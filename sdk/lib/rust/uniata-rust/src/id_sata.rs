//! id_sata.cpp — SATA SCR access + AHCI engine.
//! Translation of drivers/storage/ide/uniata/id_sata.cpp (all functions).
//! Also hosts the id_sata.h inline helpers (UniataAhciReadChannelPort4,
//! UniataAhciWriteChannelPort4, UniataAhciReadHostPort4/WriteHostPort4,
//! UniataAhciUlongFromRFIS, UniAtaAhciAdjustIoFlags, UniataAhciChanImplemented,
//! UniataIsSATARangeAvailable) as crate-wide fns so siblings reach them by name.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code, unused_unsafe)]

use crate::types::*;
use crate::regidx::*;
use crate::*; // sibling uniata functions + ScsiPortGetBusDataByOffset
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::driver_ddk::DbgPrint; // ahci markers
use crate::io_port::{
    AtapiReadPort1, AtapiReadPort2, AtapiReadPort4, AtapiReadPortEx1, AtapiReadPortEx4,
    AtapiWritePort1, AtapiWritePort2, AtapiWritePort4, AtapiWritePortEx1, AtapiWritePortEx4,
    AtapiReadBuffer2, AtapiReadBuffer4, AtapiWriteBuffer2, AtapiWriteBuffer4, UniataNanoSleep,
};

// ---------------------------------------------------------------------------
// bsmaster.h alignment masks (ULONGLONG #defines the bindgen tables drop) and
// a couple of derived AHCI constants.
// ---------------------------------------------------------------------------
const AHCI_CLB_ALIGNEMENT_MASK: ULONGLONG = 1024 - 1;
const AHCI_FIS_ALIGNEMENT_MASK: ULONGLONG = 256 - 1;
const AHCI_CMD_ALIGNEMENT_MASK: ULONGLONG = 128 - 1;
// AHCI_FIS_COMM_PM = 0x80 | AHCI_DEV_SEL_PM
const AHCI_FIS_COMM_PM: UCHAR = (0x80 | (AHCI_DEV_SEL_PM as u32)) as UCHAR;
// bsmaster.h LBA28 limits (DEF_U64).
const ATA_MAX_IOLBA28: ULONGLONG = 0x0fffff80;
const ATA_MAX_LBA28: ULONGLONG = 0x0fffffff;
// PCIConfiguration enumerator value of BUS_DATA_TYPE (id_probe's const is private).
const PCIConfiguration: BUS_DATA_TYPE = 4;

// ===========================================================================
//  id_sata.h inline helpers — provided here as crate-wide fns.
// ===========================================================================

#[inline]
pub unsafe extern "system" fn UniataIsSATARangeAvailable(
    deviceExtension: PHW_DEVICE_EXTENSION,
    lChannel: ULONG,
) -> BOOLEAN {
    let chan = (*deviceExtension).chan.add(lChannel as usize);
    let rt = &(*chan).RegTranslation[IDX_SATA_SStatus as usize];
    if *rt.__bindgen_anon_1.Addr.as_ref() != 0 || rt.Proc() != 0 {
        return TRUE as BOOLEAN;
    }
    FALSE as BOOLEAN
}

#[inline]
pub unsafe extern "system" fn UniataAhciReadChannelPort4(
    chan: PHW_CHANNEL,
    io_port_ndx: ULONG,
) -> ULONG {
    AtapiReadPortEx4(
        core::ptr::null_mut(),
        &(*chan).BaseIoAHCI_Port as *const _ as ULONG_PTR,
        io_port_ndx,
    )
}

#[inline]
pub unsafe extern "system" fn UniataAhciWriteChannelPort4(
    chan: PHW_CHANNEL,
    io_port_ndx: ULONG,
    data: ULONG,
) {
    AtapiWritePortEx4(
        core::ptr::null_mut(),
        &(*chan).BaseIoAHCI_Port as *const _ as ULONG_PTR,
        io_port_ndx,
        data,
    );
}

#[inline]
pub unsafe extern "system" fn UniataAhciReadHostPort4(
    deviceExtension: PHW_DEVICE_EXTENSION,
    io_port_ndx: ULONG,
) -> ULONG {
    AtapiReadPortEx4(
        core::ptr::null_mut(),
        &(*deviceExtension).BaseIoAHCI_0 as *const _ as ULONG_PTR,
        io_port_ndx,
    )
}

#[inline]
pub unsafe extern "system" fn UniataAhciWriteHostPort4(
    deviceExtension: PHW_DEVICE_EXTENSION,
    io_port_ndx: ULONG,
    data: ULONG,
) {
    AtapiWritePortEx4(
        core::ptr::null_mut(),
        &(*deviceExtension).BaseIoAHCI_0 as *const _ as ULONG_PTR,
        io_port_ndx,
        data,
    );
}

#[inline]
pub unsafe extern "system" fn UniataAhciUlongFromRFIS(RCV_FIS: PUCHAR) -> ULONG {
    ((*RCV_FIS.add(6) as ULONG) << 24)
        | ((*RCV_FIS.add(5) as ULONG) << 16)
        | ((*RCV_FIS.add(4) as ULONG) << 8)
        | (*RCV_FIS.add(12) as ULONG)
}

#[inline]
pub unsafe extern "system" fn UniAtaAhciAdjustIoFlags(
    command: UCHAR,
    ahci_flags: USHORT,
    fis_size: ULONG,
    DeviceNumber: ULONG,
) -> USHORT {
    let mut ahci_flags = ahci_flags;
    ahci_flags |= ((fis_size / core::mem::size_of::<ULONG>() as ULONG) | (DeviceNumber << 12)) as USHORT;
    if command == 0 {
        return ahci_flags;
    }
    if (AtaCommandFlags[command as usize] as ULONG & ATA_CMD_FLAG_Out) != 0 {
        ahci_flags |= ATA_AHCI_CMD_WRITE as USHORT;
    }
    ahci_flags
}

#[inline]
pub unsafe extern "system" fn UniataAhciChanImplemented(
    deviceExtension: PHW_DEVICE_EXTENSION,
    c: ULONG,
) -> BOOLEAN {
    if ((*deviceExtension).AHCI_PI & ((1u32) << c)) != 0 {
        TRUE as BOOLEAN
    } else {
        FALSE as BOOLEAN
    }
}

// ---------------------------------------------------------------------------
// ATA_REQ field accessors (bindgen buries the real fields under nested anon
// unions — see id_ata_part3.rs for the same shape).
// ---------------------------------------------------------------------------
#[inline(always)]
unsafe fn ata_req_core(r: PATA_REQ) -> *mut _ATA_REQ__bindgen_ty_1__bindgen_ty_1 {
    &mut (*r).__bindgen_anon_1.as_mut().__bindgen_anon_1
}
#[inline(always)]
unsafe fn ata_req_ahci(
    r: PATA_REQ,
) -> *mut _ATA_REQ__bindgen_ty_1__bindgen_ty_1__bindgen_ty_1__bindgen_ty_2 {
    (*ata_req_core(r)).__bindgen_anon_1.ahci.as_mut() as *mut _
}
// AtaReq->ahci_cmd0 — the embedded IDE_AHCI_CMD in the second sub-union.
#[inline(always)]
unsafe fn ata_req_ahci_cmd0(r: PATA_REQ) -> *mut IDE_AHCI_CMD {
    (*r).__bindgen_anon_1
        .as_mut()
        .__bindgen_anon_2
        .__bindgen_anon_1
        .ahci_cmd0
        .as_mut() as *mut _
}

#[inline(always)]
fn min_u32(a: ULONG, b: ULONG) -> ULONG { if a < b { a } else { b } }
#[inline(always)]
fn max_u32(a: ULONG, b: ULONG) -> ULONG { if a > b { a } else { b } }

// Set one RegTranslation[idx] entry (Addr + MemIo) — used by UniataAhciInit.
#[inline(always)]
unsafe fn ahci_set_rt(chan: PHW_CHANNEL, idx: usize, addr: ULONG_PTR, mem_io: BOOLEAN) {
    let rt = &mut (*chan).RegTranslation[idx];
    *rt.__bindgen_anon_1.Addr.as_mut() = addr;
    rt.set_MemIo(mem_io as ULONG);
}

// ===========================================================================
//  UniataSataConnect
// ===========================================================================
pub unsafe extern "system" fn UniataSataConnect(
    HwDeviceExtension: PVOID,
    lChannel: ULONG,
    pm_port: ULONG,
) -> UCHAR {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let mut SStatus: SATA_SSTATUS_REG = core::mem::zeroed();
    let mut i: ULONG;
    let Status: UCHAR;

    if UniataIsSATARangeAvailable(deviceExtension, lChannel) == 0 {
        return IDE_STATUS_IDLE as UCHAR;
    }

    /* clear SATA error register, some controllers need this */
    UniataSataWritePort4(
        chan,
        IDX_SATA_SError as ULONG,
        UniataSataReadPort4(chan, IDX_SATA_SError as ULONG, pm_port),
        pm_port,
    );
    /* wait up to 1 second for "connect well" */
    i = 0;
    while i < 100 {
        *SStatus.Reg.as_mut() = UniataSataReadPort4(chan, IDX_SATA_SStatus as ULONG, pm_port);
        let spd = SStatus.__bindgen_anon_1.as_ref().SPD();
        if spd == SStatus_SPD_Gen1 || spd == SStatus_SPD_Gen2 || spd == SStatus_SPD_Gen3 {
            let lun0 = (*chan).lun[0];
            let mode = (ATA_SA150 + (spd - 1)) as UCHAR;
            (*lun0).LimitedTransferMode = mode;
            (*lun0).PhyTransferMode = mode;
            (*lun0).TransferMode = mode;

            if (*chan).MaxTransferMode < (*lun0).TransferMode as ULONG {
                (*chan).MaxTransferMode = (*lun0).TransferMode as ULONG;
            }
            if (*deviceExtension).MaxTransferMode < (*lun0).TransferMode as ULONG {
                (*deviceExtension).MaxTransferMode = (*lun0).TransferMode as ULONG;
            }
            break;
        }
        // An EMPTY port never reports presence, and the C sat out the whole
        // second on it anyway: four empty ich9-ahci ports cost ~4s of every AHCI
        // boot. Upstream's ahci_hw.c (AtaAhciPhyCheckDevicePresence) gives up
        // once DET still says "no device" after 40ms; a device that IS there
        // (DET=1, PHY not up yet) keeps the full second to negotiate.
        if i >= 4 && SStatus.__bindgen_anon_1.as_ref().DET() == SStatus_DET_NoDev {
            return IDE_STATUS_WRONG as UCHAR;
        }
        ScsiPortStallExecution(10000);
        i += 1;
    }
    if i >= 100 {
        return IDE_STATUS_WRONG as UCHAR;
    }
    /* clear SATA error register */
    UniataSataWritePort4(
        chan,
        IDX_SATA_SError as ULONG,
        UniataSataReadPort4(chan, IDX_SATA_SError as ULONG, pm_port),
        pm_port,
    );

    Status = WaitOnBaseBusyLong(chan);
    if (Status as ULONG & IDE_STATUS_BUSY) != 0 {
        return Status;
    }
    IDE_STATUS_IDLE as UCHAR
}

// ===========================================================================
//  UniataSataPhyEnable
// ===========================================================================
pub unsafe extern "system" fn UniataSataPhyEnable(
    HwDeviceExtension: PVOID,
    lChannel: ULONG,
    pm_port: ULONG,
    doReset: BOOLEAN,
) -> UCHAR {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let mut SControl: SATA_SCONTROL_REG = core::mem::zeroed();
    let mut loop_i: i32;
    let mut retry: i32;

    if UniataIsSATARangeAvailable(deviceExtension, lChannel) == 0 {
        return IDE_STATUS_IDLE as UCHAR;
    }

    *SControl.Reg.as_mut() = UniataSataReadPort4(chan, IDX_SATA_SControl as ULONG, pm_port);
    if SControl.__bindgen_anon_1.as_ref().DET() == SControl_DET_Idle {
        if doReset == 0 {
            return UniataSataConnect(HwDeviceExtension, lChannel, pm_port);
        }
    }

    // Keep the link-speed limit across the COMRESET: the C rebuilt SControl
    // from 0 here, so a downgrade (UniataSataDowngradeLinkSpeed) could never
    // take effect.
    let spd_limit = SControl.__bindgen_anon_1.as_ref().SPD();

    retry = 0;
    while retry < 10 {
        loop_i = 0;
        while loop_i < 10 {
            *SControl.Reg.as_mut() = 0;
            SControl.__bindgen_anon_1.as_mut().set_SPD(spd_limit);
            SControl.__bindgen_anon_1.as_mut().set_DET(SControl_DET_Init);
            UniataSataWritePort4(chan, IDX_SATA_SControl as ULONG, *SControl.Reg.as_ref(), pm_port);
            ScsiPortStallExecution(100);
            *SControl.Reg.as_mut() = UniataSataReadPort4(chan, IDX_SATA_SControl as ULONG, pm_port);
            if SControl.__bindgen_anon_1.as_ref().DET() == SControl_DET_Init {
                break;
            }
            loop_i += 1;
        }
        ScsiPortStallExecution(5000);
        loop_i = 0;
        while loop_i < 10 {
            *SControl.Reg.as_mut() = 0;
            SControl.__bindgen_anon_1.as_mut().set_SPD(spd_limit);
            SControl.__bindgen_anon_1.as_mut().set_DET(SControl_DET_DoNothing);
            SControl.__bindgen_anon_1.as_mut().set_IPM(SControl_IPM_NoPartialSlumber);
            UniataSataWritePort4(chan, IDX_SATA_SControl as ULONG, *SControl.Reg.as_ref(), pm_port);
            ScsiPortStallExecution(100);
            *SControl.Reg.as_mut() = UniataSataReadPort4(chan, IDX_SATA_SControl as ULONG, pm_port);
            if SControl.__bindgen_anon_1.as_ref().DET() == SControl_DET_Idle {
                return UniataSataConnect(HwDeviceExtension, lChannel, pm_port);
            }
            loop_i += 1;
        }
        retry += 1;
    }

    IDE_STATUS_WRONG as UCHAR
}

/// Cap the SATA link one generation below what it negotiated, for a link
/// throwing repeated DMA errors (upstream ahci_hw.c AtaAhciDowngradeInterface-
/// Speed, driven from dev_error.c: a bad cable usually shows up as CRC errors,
/// and a slower link is the step before giving up on DMA). Takes effect at
/// the next COMRESET, which UniataSataPhyEnable now preserves it across.
/// Upstream only lowers a limit that is ALREADY set (`!= LIMIT_NONE`), so a
/// link starting unrestricted, i.e. every link, could never be downgraded;
/// this lowers it from the negotiated speed. Returns whether it changed.
pub unsafe fn UniataSataDowngradeLinkSpeed(
    HwDeviceExtension: PVOID,
    lChannel: ULONG,
    pm_port: ULONG,
) -> bool {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    if UniataIsSATARangeAvailable(deviceExtension, lChannel) == 0 {
        return false;
    }
    let mut SStatus: SATA_SSTATUS_REG = core::mem::zeroed();
    let mut SControl: SATA_SCONTROL_REG = core::mem::zeroed();
    *SStatus.Reg.as_mut() = UniataSataReadPort4(chan, IDX_SATA_SStatus as ULONG, pm_port);
    *SControl.Reg.as_mut() = UniataSataReadPort4(chan, IDX_SATA_SControl as ULONG, pm_port);
    let cur = SStatus.__bindgen_anon_1.as_ref().SPD();
    if cur <= SStatus_SPD_Gen1 || cur > SStatus_SPD_Gen3 {
        return false;
    }
    let limit = SControl.__bindgen_anon_1.as_ref().SPD();
    let want = cur - 1; // SControl_SPD_LimGen<n> == n
    if limit != SControl_SPD_NoRestrict && limit <= want {
        return false;
    }
    SControl.__bindgen_anon_1.as_mut().set_SPD(want);
    UniataSataWritePort4(chan, IDX_SATA_SControl as ULONG, *SControl.Reg.as_ref(), pm_port);
    DbgPrint(b"[UNIATA] ch=%u: repeated DMA errors, SATA link limited to Gen%u\n\0".as_ptr().cast(),
        lChannel, want);
    true
}

// ===========================================================================
//  UniataSataClearErr
// ===========================================================================
pub unsafe extern "system" fn UniataSataClearErr(
    HwDeviceExtension: PVOID,
    lChannel: ULONG,
    do_connect: BOOLEAN,
    pm_port: ULONG,
) -> BOOLEAN {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let mut SStatus: SATA_SSTATUS_REG = core::mem::zeroed();
    let mut SError: SATA_SERROR_REG = core::mem::zeroed();

    if UniataIsSATARangeAvailable(deviceExtension, lChannel) != 0 {
        *SStatus.Reg.as_mut() = UniataSataReadPort4(chan, IDX_SATA_SStatus as ULONG, pm_port);
        *SError.Reg.as_mut() = UniataSataReadPort4(chan, IDX_SATA_SError as ULONG, pm_port);

        if *SError.Reg.as_ref() != 0 {
            /* clear error bits/interrupt */
            UniataSataWritePort4(chan, IDX_SATA_SError as ULONG, *SError.Reg.as_ref(), pm_port);

            if do_connect != 0 {
                /* if we have a connection event deal with it */
                if SError.__bindgen_anon_1.as_ref().DIAG.N() != 0 {
                    if SStatus.__bindgen_anon_1.as_ref().SPD() >= SStatus_SPD_Gen1 {
                        UniataSataEvent(deviceExtension as PVOID, lChannel, UNIATA_SATA_EVENT_ATTACH, pm_port);
                    } else {
                        UniataSataEvent(deviceExtension as PVOID, lChannel, UNIATA_SATA_EVENT_DETACH, pm_port);
                    }
                    return TRUE as BOOLEAN;
                }
            }
        }
    }
    FALSE as BOOLEAN
}

// ===========================================================================
//  UniataSataEvent
// ===========================================================================
pub unsafe extern "system" fn UniataSataEvent(
    HwDeviceExtension: PVOID,
    lChannel: ULONG,
    Action: ULONG,
    pm_port: ULONG,
) -> BOOLEAN {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let Status: UCHAR;
    let DeviceNumber: ULONG = if pm_port != 0 { 1 } else { 0 };

    if UniataIsSATARangeAvailable(deviceExtension, lChannel) == 0 {
        return FALSE as BOOLEAN;
    }

    match Action {
        x if x == UNIATA_SATA_EVENT_ATTACH => {
            Status = UniataSataConnect(HwDeviceExtension, lChannel, pm_port);
            if Status as ULONG != IDE_STATUS_IDLE {
                return FALSE as BOOLEAN;
            }
            CheckDevice(HwDeviceExtension, lChannel, DeviceNumber, FALSE as BOOLEAN);
            return TRUE as BOOLEAN;
        }
        x if x == UNIATA_SATA_EVENT_DETACH => {
            let chan = (*deviceExtension).chan.add(lChannel as usize);
            UniataForgetDevice((*chan).lun[DeviceNumber as usize]);
            return TRUE as BOOLEAN;
        }
        _ => {}
    }
    FALSE as BOOLEAN
}

// ===========================================================================
//  UniataSataReadPort4
// ===========================================================================
pub unsafe extern "system" fn UniataSataReadPort4(
    chan: PHW_CHANNEL,
    io_port_ndx: ULONG,
    pm_port: ULONG,
) -> ULONG {
    if !chan.is_null()
        && (io_port_ndx < IDX_MAX_REG as ULONG)
        && (*chan).RegTranslation[io_port_ndx as usize].Proc() != 0
    {
        let deviceExtension: PHW_DEVICE_EXTENSION = (*chan).DeviceExtension;
        let HwDeviceExtension: PVOID = deviceExtension as PVOID;
        let slotNumber: ULONG = (*deviceExtension).slotNumber;
        let SystemIoBusNumber: ULONG = (*deviceExtension).SystemIoBusNumber;
        let VendorID: ULONG = (*deviceExtension).DevID & 0xffff;
        let mut offs: ULONG;
        let p: ULONG;

        match VendorID {
            x if x == ATA_INTEL_ID => {
                p = if pm_port != 0 { 1 } else { 0 };
                if ((*deviceExtension).HwFlags & ICH5) != 0 {
                    offs = 0x50 + *(*(*chan).lun[p as usize]).__bindgen_anon_1.SATA_lun_map.as_ref() * 0x10;
                    match io_port_ndx {
                        x if x == IDX_SATA_SStatus as ULONG => offs += 0,
                        x if x == IDX_SATA_SError as ULONG => offs += 1 * 4,
                        x if x == IDX_SATA_SControl as ULONG => offs += 2 * 4,
                        _ => return !0,
                    }
                    // SetPciConfig4(0xa0, offs)
                    ScsiPortSetBusDataByOffset(
                        HwDeviceExtension, PCIConfiguration as ULONG, SystemIoBusNumber,
                        slotNumber, &mut offs as *mut _ as PVOID, 0xa0, 4,
                    );
                    // GetPciConfig4(0xa4, offs)
                    ScsiPortGetBusDataByOffset(
                        HwDeviceExtension, PCIConfiguration as i32, SystemIoBusNumber,
                        slotNumber, &mut offs as *mut _ as PVOID, 0xa4, 4,
                    );
                    return offs;
                } else if ((*deviceExtension).HwFlags & ICH7) != 0 {
                    offs = 0x100 + *(*(*chan).lun[p as usize]).__bindgen_anon_1.SATA_lun_map.as_ref() * 0x80;
                    match io_port_ndx {
                        x if x == IDX_SATA_SStatus as ULONG => offs += IDX_AHCI_P_SStatus as ULONG,
                        x if x == IDX_SATA_SError as ULONG => offs += IDX_AHCI_P_SError as ULONG,
                        x if x == IDX_SATA_SControl as ULONG => offs += IDX_AHCI_P_SControl as ULONG,
                        _ => return !0,
                    }
                    return AtapiReadPortEx4(
                        core::ptr::null_mut(),
                        &(*deviceExtension).BaseIoAddressSATA_0 as *const _ as ULONG_PTR,
                        offs,
                    );
                } else {
                    offs = (((*deviceExtension).Channel + (*chan).lChannel) * 2 + p) * 0x100;
                    match io_port_ndx {
                        x if x == IDX_SATA_SStatus as ULONG => offs += 0,
                        x if x == IDX_SATA_SControl as ULONG => offs += 1,
                        x if x == IDX_SATA_SError as ULONG => offs += 2,
                        _ => return !0,
                    }
                    AtapiWritePort4(chan, IDX_INDEXED_ADDR, offs);
                    return AtapiReadPort4(chan, IDX_INDEXED_DATA);
                }
            }
            _ => {}
        }
        return !0;
    }
    AtapiReadPort4(chan, io_port_ndx as ULONG_PTR)
}

// ===========================================================================
//  UniataSataWritePort4
// ===========================================================================
pub unsafe extern "system" fn UniataSataWritePort4(
    chan: PHW_CHANNEL,
    io_port_ndx: ULONG,
    data: ULONG,
    pm_port: ULONG,
) {
    if !chan.is_null()
        && (io_port_ndx < IDX_MAX_REG as ULONG)
        && (*chan).RegTranslation[io_port_ndx as usize].Proc() != 0
    {
        let deviceExtension: PHW_DEVICE_EXTENSION = (*chan).DeviceExtension;
        let HwDeviceExtension: PVOID = deviceExtension as PVOID;
        let slotNumber: ULONG = (*deviceExtension).slotNumber;
        let SystemIoBusNumber: ULONG = (*deviceExtension).SystemIoBusNumber;
        let VendorID: ULONG = (*deviceExtension).DevID & 0xffff;
        let mut offs: ULONG;
        let p: ULONG;

        match VendorID {
            x if x == ATA_INTEL_ID => {
                p = if pm_port != 0 { 1 } else { 0 };
                if ((*deviceExtension).HwFlags & ICH5) != 0 {
                    offs = 0x50 + *(*(*chan).lun[p as usize]).__bindgen_anon_1.SATA_lun_map.as_ref() * 0x10;
                    match io_port_ndx {
                        x if x == IDX_SATA_SStatus as ULONG => offs += 0,
                        x if x == IDX_SATA_SError as ULONG => offs += 1 * 4,
                        x if x == IDX_SATA_SControl as ULONG => offs += 2 * 4,
                        _ => return,
                    }
                    let mut datacopy = data;
                    // SetPciConfig4(0xa0, offs)
                    ScsiPortSetBusDataByOffset(
                        HwDeviceExtension, PCIConfiguration as ULONG, SystemIoBusNumber,
                        slotNumber, &mut offs as *mut _ as PVOID, 0xa0, 4,
                    );
                    // SetPciConfig4(0xa4, data)
                    ScsiPortSetBusDataByOffset(
                        HwDeviceExtension, PCIConfiguration as ULONG, SystemIoBusNumber,
                        slotNumber, &mut datacopy as *mut _ as PVOID, 0xa4, 4,
                    );
                    return;
                } else if ((*deviceExtension).HwFlags & ICH7) != 0 {
                    offs = 0x100 + *(*(*chan).lun[p as usize]).__bindgen_anon_1.SATA_lun_map.as_ref() * 0x80;
                    match io_port_ndx {
                        x if x == IDX_SATA_SStatus as ULONG => offs += IDX_AHCI_P_SStatus as ULONG,
                        x if x == IDX_SATA_SError as ULONG => offs += IDX_AHCI_P_SError as ULONG,
                        x if x == IDX_SATA_SControl as ULONG => offs += IDX_AHCI_P_SControl as ULONG,
                        _ => return,
                    }
                    AtapiWritePortEx4(
                        core::ptr::null_mut(),
                        &(*deviceExtension).BaseIoAddressSATA_0 as *const _ as ULONG_PTR,
                        offs, data,
                    );
                    return;
                } else {
                    offs = (((*deviceExtension).Channel + (*chan).lChannel) * 2 + p) * 0x100;
                    match io_port_ndx {
                        x if x == IDX_SATA_SStatus as ULONG => offs += 0,
                        x if x == IDX_SATA_SControl as ULONG => offs += 1,
                        x if x == IDX_SATA_SError as ULONG => offs += 2,
                        _ => return,
                    }
                    AtapiWritePort4(chan, IDX_INDEXED_ADDR, offs);
                    AtapiWritePort4(chan, IDX_INDEXED_DATA, data);
                }
            }
            _ => {}
        }
        return;
    }
    AtapiWritePort4(chan, io_port_ndx as ULONG_PTR, data);
}

// ===========================================================================
//  UniataSataReadPM / UniataSataWritePM / UniataSataSoftReset
// ===========================================================================
pub unsafe extern "system" fn UniataSataReadPM(
    chan: PHW_CHANNEL,
    DeviceNumber: ULONG,
    Reg: ULONG,
    result: PULONG,
) -> BOOLEAN {
    if ((*(*chan).DeviceExtension).HwFlags & UNIATA_AHCI) != 0 {
        return UniataAhciReadPM(chan, DeviceNumber, Reg, result);
    }
    FALSE as BOOLEAN
}

pub unsafe extern "system" fn UniataSataWritePM(
    chan: PHW_CHANNEL,
    DeviceNumber: ULONG,
    Reg: ULONG,
    value: ULONG,
) -> UCHAR {
    if ((*(*chan).DeviceExtension).HwFlags & UNIATA_AHCI) != 0 {
        return UniataAhciWritePM(chan, DeviceNumber, Reg, value);
    }
    IDE_STATUS_WRONG as UCHAR
}

pub unsafe extern "system" fn UniataSataSoftReset(
    HwDeviceExtension: PVOID,
    lChannel: ULONG,
    DeviceNumber: ULONG,
) -> ULONG {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    if ((*deviceExtension).HwFlags & UNIATA_AHCI) != 0 {
        return UniataAhciSoftReset(HwDeviceExtension, lChannel, DeviceNumber);
    }
    0xffffffff
}

// ===========================================================================
//  UniataSataIdentifyPM
// ===========================================================================
pub unsafe extern "system" fn UniataSataIdentifyPM(chan: PHW_CHANNEL) {
    let mut PM_DeviceId: ULONG = 0;
    let mut PM_RevId: ULONG = 0;
    let mut PM_Ports: ULONG = 0;
    let mut i: UCHAR;

    (*chan).PmLunMap = 0;

    /* get PM vendor & product data */
    if UniataSataReadPM(chan, AHCI_DEV_SEL_PM, 0, &mut PM_DeviceId) == 0 {
        return;
    }
    /* get PM revision data */
    if UniataSataReadPM(chan, AHCI_DEV_SEL_PM, 1, &mut PM_RevId) == 0 {
        return;
    }
    /* get number of HW ports on the PM */
    if UniataSataReadPM(chan, AHCI_DEV_SEL_PM, 2, &mut PM_Ports) == 0 {
        return;
    }

    PM_Ports &= 0x0000000f;

    // Hide each PM's pseudo/config port (upstream ahci_hw.c, after FreeBSD).
    // UniATA forced 5 on 3726/4726 whatever the count, knew neither the SiI 3826
    // nor the Marvell parts, and left the SiI "XmitEarlyAck" bit (GSCR 129
    // bit 0) set, which upstream clears on both SiI 37xx/38xx parts.
    match PM_DeviceId {
        0x37261095 | 0x38261095 => {
            let mut v: ULONG = 0;
            if UniataSataReadPM(chan, AHCI_DEV_SEL_PM, 129, &mut v) != 0 && (v & 1) != 0 {
                UniataSataWritePM(chan, AHCI_DEV_SEL_PM, 129, v & !1);
            }
            if PM_Ports == 6 {
                PM_Ports = 5;
            }
        }
        0x47261095 => {
            if PM_Ports == 7 {
                PM_Ports = 5;
            }
        }
        0x57231095 | 0x57331095 | 0x57341095 | 0x57441095 => {
            if PM_Ports > 0 {
                PM_Ports -= 1;
            }
        }
        _ => {}
    }

    // reset
    i = 0;
    while (i as ULONG) < PM_Ports {
        let signature_loop: ULONG;
        let lun = (*chan).lun[i as usize];

        if UniataSataPhyEnable(
            (*chan).DeviceExtension as PVOID,
            (*chan).lChannel,
            i as ULONG,
            UNIATA_SATA_RESET_ENABLE as BOOLEAN,
        ) as ULONG
            != IDE_STATUS_IDLE
        {
            (*lun).DeviceFlags &= !DFLAGS_DEVICE_PRESENT;
            i += 1;
            continue;
        }
        ScsiPortStallExecution(1000000);

        signature_loop = UniataSataSoftReset((*chan).DeviceExtension as PVOID, (*chan).lChannel, i as ULONG);

        (*lun).DeviceFlags |= DFLAGS_DEVICE_PRESENT;
        (*chan).PmLunMap |= 1 << i;
        /* figure out whats there */
        match signature_loop >> 16 {
            0x0000 => {
                (*lun).DeviceFlags &= !DFLAGS_ATAPI_DEVICE;
                i += 1;
                continue;
            }
            0xeb14 => {
                (*lun).DeviceFlags |= DFLAGS_ATAPI_DEVICE;
                i += 1;
                continue;
            }
            _ => {}
        }
        let _ = signature_loop;
        i += 1;
    }
    let _ = PM_RevId;
}

// ===========================================================================
//  UniataAhciInit
// ===========================================================================
pub unsafe extern "system" fn UniataAhciInit(HwDeviceExtension: PVOID) -> BOOLEAN {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let mut c: ULONG;
    let mut i: ULONG;
    let mut chan: PHW_CHANNEL;
    let mut offs: ULONG;
    let BaseMemAddress: ULONG_PTR;
    let mut PI: ULONG;
    let CAP: ULONG;
    let mut CAP2: ULONG;
    let mut BOHC: ULONG;
    let mut GHC: ULONG;
    let MemIo: BOOLEAN;

    // AHCI 1.3.1 sec. 10.1.2: set GHC.AE before touching any other AHCI
    // register. UniataAhciDetect restores the firmware's GHC on exit, which on
    // an HBA that also does legacy IDE (CAP.SAM=1) can have AE=0, and the C
    // then read CAP2 and did the BIOS handoff in legacy mode. CAP2 itself only
    // exists from AHCI 1.2 on; before that the offset is reserved. Both rules
    // are upstream's (ahci_generic.c). The IE clearing below is unchanged.
    GHC = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_GHC as ULONG);
    if (GHC & AHCI_GHC_AE) == 0 {
        UniataAhciWriteHostPort4(deviceExtension, IDX_AHCI_GHC as ULONG, GHC | AHCI_GHC_AE);
    }
    CAP2 = if UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_VS as ULONG) >= 0x0001_0200 {
        UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_CAP2 as ULONG)
    } else {
        0
    };
    if (CAP2 & AHCI_CAP2_BOH) != 0 {
        BOHC = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_BOHC as ULONG);
        UniataAhciWriteHostPort4(deviceExtension, IDX_AHCI_BOHC as ULONG, BOHC | AHCI_BOHC_OOS);
        i = 0;
        while i < 50 {
            ScsiPortStallExecution(500);
            BOHC = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_BOHC as ULONG);
            if (BOHC & AHCI_BOHC_BB) != 0 {
                break;
            }
            if (BOHC & AHCI_BOHC_BOS) == 0 {
                break;
            }
            i += 1;
        }
        if (BOHC & AHCI_BOHC_BB) != 0 {
            i = 0;
            while i < 2000 {
                ScsiPortStallExecution(1000);
                BOHC = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_BOHC as ULONG);
                if (BOHC & AHCI_BOHC_BOS) == 0 {
                    break;
                }
                i += 1;
            }
        }
    }

    /* enable AHCI mode */
    GHC = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_GHC as ULONG);
    if (GHC & AHCI_GHC_AE) == 0 {
        UniataAhciWriteHostPort4(deviceExtension, IDX_AHCI_GHC as ULONG, (GHC | AHCI_GHC_AE) & !AHCI_GHC_IE);
    } else {
        UniataAhciWriteHostPort4(deviceExtension, IDX_AHCI_GHC as ULONG, GHC & !AHCI_GHC_IE);
    }
    ScsiPortStallExecution(100);

    /* read GHC again and reset AHCI controller */
    GHC = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_GHC as ULONG);
    UniataAhciWriteHostPort4(deviceExtension, IDX_AHCI_GHC as ULONG, GHC | AHCI_GHC_HR);

    i = 0;
    while i < 1000 {
        ScsiPortStallExecution(1000);
        GHC = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_GHC as ULONG);
        if (GHC & AHCI_GHC_HR) == 0 {
            break;
        }
        i += 1;
    }
    if (GHC & AHCI_GHC_HR) != 0 {
        return FALSE as BOOLEAN;
    }

    /* re-enable AHCI mode */
    GHC = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_GHC as ULONG);
    i = 0;
    while i < 5 {
        if (GHC & AHCI_GHC_AE) == 0 {
            UniataAhciWriteHostPort4(deviceExtension, IDX_AHCI_GHC as ULONG, GHC | AHCI_GHC_AE);
            ScsiPortStallExecution(1000);
            GHC = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_GHC as ULONG);
        } else {
            break;
        }
        i += 1;
    }
    if (GHC & AHCI_GHC_AE) == 0 {
        return FALSE as BOOLEAN;
    }

    CAP = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_CAP as ULONG);
    (*deviceExtension).AHCI_CAP = CAP;
    if (CAP & AHCI_CAP_S64A) != 0 {
        (*deviceExtension).Host64 = TRUE as BOOLEAN;
    }

    /* get the number of HW channels */
    PI = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_PI as ULONG);
    (*deviceExtension).AHCI_PI = PI;
    PI &= (*deviceExtension).AHCI_PI_mask;
    (*deviceExtension).AHCI_PI = PI;

    CAP2 = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_CAP2 as ULONG);
    if (CAP2 & AHCI_CAP2_BOH) != 0 {
        BOHC = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_BOHC as ULONG);
        UniataAhciWriteHostPort4(deviceExtension, IDX_AHCI_BOHC as ULONG, BOHC | AHCI_BOHC_OOS);
    }
    /* clear interrupts */
    UniataAhciWriteHostPort4(
        deviceExtension, IDX_AHCI_IS as ULONG,
        UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_IS as ULONG),
    );

    /* enable AHCI interrupts */
    UniataAhciWriteHostPort4(
        deviceExtension, IDX_AHCI_GHC as ULONG,
        UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_GHC as ULONG) | AHCI_GHC_IE,
    );

    BaseMemAddress = *(*deviceExtension).BaseIoAHCI_0.__bindgen_anon_1.Addr.as_ref();
    MemIo = (*deviceExtension).BaseIoAHCI_0.MemIo() as BOOLEAN;

    (*deviceExtension).MaxTransferMode = ATA_SA150 + (((CAP & AHCI_CAP_ISS_MASK) >> 20) - 1);

    c = 0;
    while c < (*deviceExtension).NumberChannels {
        chan = (*deviceExtension).chan.add(c as usize);
        offs = (core::mem::size_of::<IDE_AHCI_REGISTERS>() as ULONG)
            + c * (core::mem::size_of::<IDE_AHCI_PORT_REGISTERS>() as ULONG);

        (*chan).MaxTransferMode = (*deviceExtension).MaxTransferMode;

        AtapiSetupLunPtrs(chan, deviceExtension, c);

        // chan->BaseIoAHCI_Port = deviceExtension->BaseIoAHCI_0;
        core::ptr::copy_nonoverlapping(
            &(*deviceExtension).BaseIoAHCI_0 as *const IORES,
            &mut (*chan).BaseIoAHCI_Port as *mut IORES,
            1,
        );
        *(*chan).BaseIoAHCI_Port.__bindgen_anon_1.Addr.as_mut() = BaseMemAddress + offs as ULONG_PTR;

        let tfd_off = core::mem::offset_of!(_IDE_AHCI_PORT_REGISTERS, TFD) as ULONG_PTR;
        let sig_off = core::mem::offset_of!(_IDE_AHCI_PORT_REGISTERS, SIG) as ULONG_PTR;

        ahci_set_rt(chan, IDX_IO1_i_Status as usize, BaseMemAddress + offs as ULONG_PTR + tfd_off, MemIo);
        // IDX_IO2_AltStatus = IDX_IO1_i_Status
        {
            let src = core::ptr::read(&(*chan).RegTranslation[IDX_IO1_i_Status as usize] as *const IORES);
            core::ptr::write(&mut (*chan).RegTranslation[IDX_IO2_AltStatus as usize] as *mut IORES, src);
        }
        // TFD.ERR = offset_of(TFD)+1 ; SIG.{LbaLow,LbaHigh,SectorCount} are bytes of SIG.
        ahci_set_rt(chan, IDX_IO1_i_Error as usize, BaseMemAddress + offs as ULONG_PTR + tfd_off + 1, MemIo);
        ahci_set_rt(chan, IDX_IO1_i_CylinderLow as usize, BaseMemAddress + offs as ULONG_PTR + sig_off + 1, MemIo);
        ahci_set_rt(chan, IDX_IO1_i_CylinderHigh as usize, BaseMemAddress + offs as ULONG_PTR + sig_off + 2, MemIo);
        ahci_set_rt(chan, IDX_IO1_i_BlockCount as usize, BaseMemAddress + offs as ULONG_PTR + sig_off + 0, MemIo);

        UniataInitSyncBaseIO(chan);

        ahci_set_rt(chan, IDX_SATA_SStatus as usize, BaseMemAddress + offs as ULONG_PTR + IDX_AHCI_P_SStatus, MemIo);
        ahci_set_rt(chan, IDX_SATA_SError as usize, BaseMemAddress + offs as ULONG_PTR + IDX_AHCI_P_SError, MemIo);
        ahci_set_rt(chan, IDX_SATA_SControl as usize, BaseMemAddress + offs as ULONG_PTR + IDX_AHCI_P_SControl, MemIo);
        ahci_set_rt(chan, IDX_SATA_SActive as usize, BaseMemAddress + offs as ULONG_PTR + IDX_AHCI_P_ACT, MemIo);

        AtapiDmaAlloc(HwDeviceExtension, core::ptr::null_mut(), c);

        if UniataAhciChanImplemented(deviceExtension, c) == 0 {
            c += 1;
            continue;
        }

        UniataAhciResume(chan);

        (*chan).ChannelCtrlFlags |= CTRFLAGS_NO_SLAVE;
        c += 1;
    }

    TRUE as BOOLEAN
}

// ===========================================================================
//  UniAtaAhciValidateVersion
// ===========================================================================
pub unsafe extern "system" fn UniAtaAhciValidateVersion(
    deviceExtension: PHW_DEVICE_EXTENSION,
    version: ULONG,
    Strict: BOOLEAN,
) -> BOOLEAN {
    match version {
        0x00000000 | 0xffffffff => {
            return FALSE as BOOLEAN;
        }
        0x00000905 | 0x00010000 | 0x00010100 | 0x00010200 | 0x00010300 | 0x00010301 => {}
        _ => {
            // L"CheckAhciRevision"
            static W_CHECK_AHCI_REV: [u16; 18] = [
                b'C' as u16, b'h' as u16, b'e' as u16, b'c' as u16, b'k' as u16, b'A' as u16,
                b'h' as u16, b'c' as u16, b'i' as u16, b'R' as u16, b'e' as u16, b'v' as u16,
                b'i' as u16, b's' as u16, b'i' as u16, b'o' as u16, b'n' as u16, 0,
            ];
            if AtapiRegCheckDevValue(
                deviceExtension as PVOID, CHAN_NOT_SPECIFIED, DEVNUM_NOT_SPECIFIED,
                W_CHECK_AHCI_REV.as_ptr() as PCWSTR, Strict as ULONG,
            ) != 0
            {
                return FALSE as BOOLEAN;
            }
        }
    }
    TRUE as BOOLEAN
}

// ===========================================================================
//  UniataAhciDetect
// ===========================================================================
pub unsafe extern "system" fn UniataAhciDetect(
    HwDeviceExtension: PVOID,
    pciData: crate::id_probe::PPCI_COMMON_CONFIG,
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
) -> BOOLEAN {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let SystemIoBusNumber: ULONG = (*deviceExtension).SystemIoBusNumber;
    let version: ULONG;
    let mut i: ULONG;
    let mut n: ULONG;
    let mut PI: ULONG;
    let CAP: ULONG;
    let CAP2: ULONG;
    let mut GHC: ULONG;
    let GHC0: ULONG;
    let mut NumberChannels: ULONG;
    let BaseMemAddress: ULONG_PTR;
    let mut MemIo: BOOLEAN = FALSE as BOOLEAN;
    let mut found: BOOLEAN = FALSE as BOOLEAN;
    let mut BarId: ULONG = 5;

    // L"IgnoreAhci"
    static W_IGNORE_AHCI: [u16; 11] = [
        b'I' as u16, b'g' as u16, b'n' as u16, b'o' as u16, b'r' as u16, b'e' as u16,
        b'A' as u16, b'h' as u16, b'c' as u16, b'i' as u16, 0,
    ];
    // L"Exclude"
    static W_EXCLUDE: [u16; 8] = [
        b'E' as u16, b'x' as u16, b'c' as u16, b'l' as u16, b'u' as u16, b'd' as u16, b'e' as u16, 0,
    ];
    // L"PortMask"
    static W_PORTMASK: [u16; 9] = [
        b'P' as u16, b'o' as u16, b'r' as u16, b't' as u16, b'M' as u16, b'a' as u16, b's' as u16, b'k' as u16, 0,
    ];
    // L"IgnoreAhciPM"
    static W_IGNORE_AHCI_PM: [u16; 13] = [
        b'I' as u16, b'g' as u16, b'n' as u16, b'o' as u16, b'r' as u16, b'e' as u16,
        b'A' as u16, b'h' as u16, b'c' as u16, b'i' as u16, b'P' as u16, b'M' as u16, 0,
    ];

    if AtapiRegCheckDevValue(
        deviceExtension as PVOID, CHAN_NOT_SPECIFIED, DEVNUM_NOT_SPECIFIED,
        W_IGNORE_AHCI.as_ptr() as PCWSTR, 0,
    ) != 0
    {
        return FALSE as BOOLEAN;
    }
    match (*deviceExtension).DevID {
        0xa01c0031 => {
            BarId = 0;
        }
        _ => {}
    }
    BaseMemAddress = AtapiGetIoRange(
        HwDeviceExtension, ConfigInfo, pciData, SystemIoBusNumber, BarId, 0, 0x10,
    );
    DbgPrint(b"[UNIATA] AhciDetect: bar%u va=%p raw=%08x\n\0".as_ptr().cast(),
        BarId, BaseMemAddress as PVOID,
        (*pciData).u.type0.as_ref().BaseAddresses[BarId as usize]); // ahci marker
    if BaseMemAddress == 0 {
        return FALSE as BOOLEAN;
    }
    // (*ConfigInfo->AccessRanges)[BarId].RangeInMemory
    {
        let ar = ((*ConfigInfo).AccessRanges as *mut ACCESS_RANGE).add(BarId as usize);
        if (*ar).RangeInMemory != 0 {
            MemIo = TRUE as BOOLEAN;
        }
    }
    *(*deviceExtension).BaseIoAHCI_0.__bindgen_anon_1.Addr.as_mut() = BaseMemAddress;
    (*deviceExtension).BaseIoAHCI_0.set_MemIo(MemIo as ULONG);

    GHC = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_GHC as ULONG);
    if (GHC & AHCI_GHC_HR) != 0 {
        return FALSE as BOOLEAN;
    }

    /* check AHCI mode. Save state and try enable */
    GHC = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_GHC as ULONG);
    GHC0 = GHC;

    version = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_VS as ULONG);

    // single-exit (exit_detect) restructured via a labeled block.
    'detect: {
        if (GHC & AHCI_GHC_AE) == 0 {
            if UniAtaAhciValidateVersion(deviceExtension, version, FALSE as BOOLEAN) == 0 {
                break 'detect;
            }
            UniataAhciWriteHostPort4(deviceExtension, IDX_AHCI_GHC as ULONG, (GHC | AHCI_GHC_AE) & !AHCI_GHC_IE);
            GHC = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_GHC as ULONG);
            if (GHC & AHCI_GHC_AE) == 0 {
                break 'detect;
            }
        }

        CAP = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_CAP as ULONG);
        CAP2 = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_CAP2 as ULONG);
        let _ = CAP2;

        /* get the number of HW channels */
        PI = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_PI as ULONG);
        (*deviceExtension).AHCI_PI = PI;
        (*deviceExtension).AHCI_PI_mask = PI;

        i = PI;
        n = 0;
        while i != 0 {
            if AtapiRegCheckDevValue(
                deviceExtension as PVOID, n, DEVNUM_NOT_SPECIFIED,
                W_EXCLUDE.as_ptr() as PCWSTR, 0,
            ) != 0
            {
                (*deviceExtension).AHCI_PI &= !((1u32) << n);
                (*deviceExtension).AHCI_PI_mask &= !((1u32) << n);
            }
            n += 1;
            i >>= 1;
        }
        (*deviceExtension).AHCI_PI_mask = AtapiRegCheckDevValue(
            deviceExtension as PVOID, CHAN_NOT_SPECIFIED, DEVNUM_NOT_SPECIFIED,
            W_PORTMASK.as_ptr() as PCWSTR, (*deviceExtension).AHCI_PI_mask,
        );

        i = PI;
        n = 0;
        while i != 0 {
            n += 1;
            i >>= 1;
        }
        NumberChannels = max_u32((CAP & AHCI_CAP_NOP_MASK) + 1, n);

        if PI == 0 && ((CAP & AHCI_CAP_NOP_MASK) + 1) != 0 {
            n = NumberChannels;
            (*deviceExtension).AHCI_PI = (1u32 << n) - 1;

            if (*deviceExtension).AHCI_PI_mask != 0 {
                PI = (*deviceExtension).AHCI_PI_mask;
            } else {
                PI = (1u32 << n) - 1;
                (*deviceExtension).AHCI_PI = PI;
                (*deviceExtension).AHCI_PI_mask = (1u32 << n) - 1;
            }
            UniataAhciWriteHostPort4(deviceExtension, IDX_AHCI_PI as ULONG, PI);
        }

        match (*deviceExtension).DevID {
            0x2361197b => {
                NumberChannels = 1;
            }
            x if x == ATA_M88SE6111 => {
                NumberChannels = 1;
            }
            x if x == ATA_M88SE6121 => {
                NumberChannels = min_u32(NumberChannels, 2);
            }
            x if x == ATA_M88SE6141 || x == ATA_M88SE6145 || x == ATA_M88SE9123 => {
                NumberChannels = min_u32(NumberChannels, 4);
            }
            _ => {}
        }

        if NumberChannels == 0 {
            found = FALSE as BOOLEAN;
            break 'detect;
        }

        if (CAP & AHCI_CAP_SPM) != 0 {
            if AtapiRegCheckDevValue(
                deviceExtension as PVOID, CHAN_NOT_SPECIFIED, DEVNUM_NOT_SPECIFIED,
                W_IGNORE_AHCI_PM.as_ptr() as PCWSTR, 1,
            ) != 0
            {
                (*deviceExtension).NumberLuns = 1;
            } else {
                (*deviceExtension).NumberLuns = SATA_MAX_PM_UNITS;
            }
        } else {
            (*deviceExtension).NumberLuns = 1;
        }

        if UniAtaAhciValidateVersion(deviceExtension, version, TRUE as BOOLEAN) == 0 {
            break 'detect;
        }

        (*deviceExtension).HwFlags |= UNIATA_SATA | UNIATA_AHCI;
        if (*deviceExtension).NumberChannels < NumberChannels {
            (*deviceExtension).NumberChannels = NumberChannels;
        }
        (*deviceExtension).DmaSegmentLength = 0x3fffff + 1; // 4MB
        (*deviceExtension).DmaSegmentAlignmentMask = !0; // no restrictions

        (*deviceExtension).BusMaster = DMA_MODE_AHCI as BOOLEAN;
        (*deviceExtension).MaxTransferMode = max_u32(
            (*deviceExtension).MaxTransferMode,
            ATA_SA150 + (((CAP & AHCI_CAP_ISS_MASK) >> 20) - 1),
        );

        found = TRUE as BOOLEAN;
    } // 'detect

    // exit_detect:
    UniataAhciWriteHostPort4(deviceExtension, IDX_AHCI_GHC as ULONG, GHC0);
    let _ = MemIo;
    DbgPrint(b"[UNIATA] AhciDetect: exit found=%u GHC=%08x chans=%u\n\0".as_ptr().cast(),
        found as ULONG, GHC, (*deviceExtension).NumberChannels); // ahci marker
    found
}

// ===========================================================================
//  UniataAhciStatus
// ===========================================================================
pub unsafe extern "system" fn UniataAhciStatus(
    HwDeviceExtension: PVOID,
    lChannel: ULONG,
    DeviceNumber: ULONG,
) -> UCHAR {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let Channel: ULONG = (*deviceExtension).Channel + lChannel;
    let mut hIS: ULONG;
    let CI: ULONG;
    let ACT: ULONG;
    let mut IS: AHCI_IS_REG = core::mem::zeroed();
    let mut SStatus: SATA_SSTATUS_REG = core::mem::zeroed();
    let mut SError: SATA_SERROR_REG = core::mem::zeroed();
    let tag: ULONG = 0;
    let _ = DeviceNumber;

    hIS = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_IS as ULONG);
    hIS &= 1 << Channel;
    if hIS == 0 {
        return INTERRUPT_REASON_IGNORE as UCHAR;
    }
    *IS.Reg.as_mut() = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_IS as ULONG);
    CI = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_CI as ULONG);
    ACT = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_ACT as ULONG);
    *SStatus.Reg.as_mut() = AtapiReadPort4(chan, IDX_SATA_SStatus);
    *SError.Reg.as_mut() = AtapiReadPort4(chan, IDX_SATA_SError);

    /* clear interrupt(s) */
    // Port first, host last (AHCI 1.3.1 sec. 5.6.4, upstream ahci_io.c): IS.IPS
    // re-asserts while the port's PxIS is still set, so the C's host-first
    // order left a pending port bit behind and drew a second, spurious
    // interrupt on real HBAs. (QEMU recomputes IS from PxIS, so it never showed.)
    UniataAhciWriteChannelPort4(chan, IDX_AHCI_P_IS as ULONG, *IS.Reg.as_ref());
    AtapiWritePort4(chan, IDX_SATA_SError, *SError.Reg.as_ref());
    UniataAhciWriteHostPort4(deviceExtension, IDX_AHCI_IS as ULONG, hIS);

    /* do we have cold connect surprise */
    if IS.__bindgen_anon_1.as_ref().CPDS() != 0 {}

    /* check for and handle connect events */
    if IS.__bindgen_anon_1.as_ref().PCS() != 0 {
        UniataSataEvent(HwDeviceExtension, lChannel, UNIATA_SATA_EVENT_ATTACH, 0);
    }
    if IS.__bindgen_anon_1.as_ref().PRCS() != 0 {
        UniataSataEvent(HwDeviceExtension, lChannel, UNIATA_SATA_EVENT_DETACH, 0);
    }
    (*chan).AhciCompleteCI = ((*chan).AhciPrevCI ^ CI) & (*chan).AhciPrevCI; // only 1->0 states
    (*chan).AhciPrevCI = CI;
    (*chan).AhciLastSError = *SError.Reg.as_ref();
    (*chan).AhciLastIS = *IS.Reg.as_ref();
    let _ = ACT;
    if (CI & (1 << tag)) != 0 {
        if (*IS.Reg.as_ref()
            & (ATA_AHCI_P_IX_OF | ATA_AHCI_P_IX_INF | ATA_AHCI_P_IX_IF
                | ATA_AHCI_P_IX_HBD | ATA_AHCI_P_IX_HBF | ATA_AHCI_P_IX_TFE))
            != 0
        {
            // unexpected error
        } else {
            return INTERRUPT_REASON_UNEXPECTED as UCHAR;
        }
    }
    INTERRUPT_REASON_OUR as UCHAR
}

// ===========================================================================
//  UniataAhciSnapAtaRegs
// ===========================================================================
pub unsafe extern "system" fn UniataAhciSnapAtaRegs(
    chan: PHW_CHANNEL,
    DeviceNumber: ULONG,
    regs: PIDEREGS_EX,
) {
    let TFD: ULONG;
    let SIG: ULONG;
    let _ = DeviceNumber;

    (*regs).bDriveHeadReg = IDE_DRIVE_SELECT_1 as UCHAR;
    TFD = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_TFD as ULONG);
    *(*regs).__bindgen_anon_2.bCommandReg.as_mut() = (TFD & 0xff) as UCHAR;
    *(*regs).__bindgen_anon_1.bFeaturesReg.as_mut() = ((TFD >> 8) & 0xff) as UCHAR;

    SIG = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_SIG as ULONG);
    (*regs).bSectorCountReg = (SIG & 0xff) as UCHAR;
    (*regs).bSectorNumberReg = ((SIG >> 8) & 0xff) as UCHAR;
    (*regs).bCylLowReg = ((SIG >> 16) & 0xff) as UCHAR;
    (*regs).bCylHighReg = ((SIG >> 24) & 0xff) as UCHAR;
    (*regs).bOpFlags = 0;
}

// ===========================================================================
//  UniataAhciSetupFIS_H2D
// ===========================================================================
pub unsafe extern "system" fn UniataAhciSetupFIS_H2D(
    deviceExtension: PHW_DEVICE_EXTENSION,
    DeviceNumber: ULONG,
    lChannel: ULONG,
    fis: PUCHAR,
    command: UCHAR,
    lba: ULONGLONG,
    count: USHORT,
    feature: USHORT,
) -> ULONG {
    let mut command = command;
    let need48: BOOLEAN;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let plba = &lba as *const ULONGLONG as *const UCHAR;

    RtlZeroMemory(fis as PVOID, 20);

    *fis.add(0) = AHCI_FIS_TYPE_ATA_H2D as UCHAR;
    *fis.add(1) = 0x80 | ((DeviceNumber as UCHAR) & 0x0f); /* command FIS (PM in low nibble) */
    *fis.add(IDX_AHCI_o_DriveSelect as usize) = (IDE_DRIVE_SELECT_1
        | (if (AtaCommandFlags[command as usize] as ULONG
            & (ATA_CMD_FLAG_LBAIOsupp | ATA_CMD_FLAG_48))
            != 0
        {
            IDE_USE_LBA
        } else {
            0
        })) as UCHAR;
    *fis.add(IDX_AHCI_o_Control as usize) = IDE_DC_A_4BIT as UCHAR;

    if command as ULONG == IDE_COMMAND_ATAPI_PACKET {
        *fis.add(IDX_AHCI_o_Command as usize) = IDE_COMMAND_ATAPI_PACKET as UCHAR;
        if (feature as ULONG & ATA_F_DMA) != 0 {
            *fis.add(IDX_AHCI_o_Feature as usize) = (feature & 0xff) as UCHAR;
        } else {
            *fis.add(IDX_AHCI_o_CylinderLow as usize) = (count & 0xff) as UCHAR;
            *fis.add(IDX_AHCI_o_CylinderHigh as usize) = ((count >> 8) & 0xff) as UCHAR;
        }
    } else {
        if ((AtaCommandFlags[command as usize] as ULONG
            & (ATA_CMD_FLAG_LBAIOsupp | ATA_CMD_FLAG_FUA))
            == ATA_CMD_FLAG_LBAIOsupp)
            && CheckIfBadBlock((*chan).lun[DeviceNumber as usize], lba, count as ULONG) != 0
        {
            return 0;
        }

        // UniAta_need_lba48(command, lba, count, supp48)
        let supp48 = (*(*chan).lun[DeviceNumber as usize])
            .IdentifyData.FeaturesSupport.Address48() as ULONG;
        need48 = ahci_need_lba48(command, lba, count as ULONG, supp48);

        if need48 != 0 {
            if (AtaCommandFlags[command as usize] as ULONG & ATA_CMD_FLAG_48supp) != 0 {
                command = AtaCommands48[command as usize];
            } else {
                return 0;
            }
        }

        *fis.add(IDX_AHCI_o_Command as usize) = command;
        *fis.add(IDX_AHCI_o_Feature as usize) = feature as UCHAR;

        *fis.add(IDX_AHCI_o_BlockNumber as usize) = *plba.add(0);
        *fis.add(IDX_AHCI_o_CylinderLow as usize) = *plba.add(1);
        *fis.add(IDX_AHCI_o_CylinderHigh as usize) = *plba.add(2);

        *fis.add(IDX_AHCI_o_BlockCount as usize) = (count & 0xff) as UCHAR;

        if need48 != 0 {
            *fis.add(IDX_AHCI_o_Control as usize) |= IDE_DC_USE_HOB as UCHAR;

            *fis.add(IDX_AHCI_o_BlockNumberExp as usize) = *plba.add(3);
            *fis.add(IDX_AHCI_o_CylinderLowExp as usize) = *plba.add(4);
            *fis.add(IDX_AHCI_o_CylinderHighExp as usize) = *plba.add(5);

            *fis.add(IDX_AHCI_o_BlockCountExp as usize) = ((count >> 8) & 0xff) as UCHAR;

            *fis.add(IDX_AHCI_o_FeatureExp as usize) = ((feature >> 8) & 0xff) as UCHAR;

            (*chan).ChannelCtrlFlags |= CTRFLAGS_LBA48;
        } else {
            *fis.add(IDX_AHCI_o_DriveSelect as usize) |= *plba.add(3) & 0x0f;
            (*chan).ChannelCtrlFlags &= !CTRFLAGS_LBA48;
        }
    }

    20
}

// helper: UniAta_need_lba48 macro from atapi.h
#[inline(always)]
unsafe fn ahci_need_lba48(command: UCHAR, lba: ULONGLONG, count: ULONG, supp48: ULONG) -> BOOLEAN {
    let cond = (((AtaCommandFlags[command as usize] as ULONG & ATA_CMD_FLAG_LBAIOsupp) != 0)
        && (supp48 != 0)
        && (((lba + count as ULONGLONG) >= ATA_MAX_IOLBA28) || (count > 256)))
        || (lba > ATA_MAX_LBA28)
        || (count > 255);
    if cond { TRUE as BOOLEAN } else { FALSE as BOOLEAN }
}

// ===========================================================================
//  UniataAhciSetupFIS_H2D_Direct
// ===========================================================================
pub unsafe extern "system" fn UniataAhciSetupFIS_H2D_Direct(
    deviceExtension: PHW_DEVICE_EXTENSION,
    DeviceNumber: ULONG,
    lChannel: ULONG,
    fis: PUCHAR,
    regs: PIDEREGS_EX,
) -> ULONG {
    let need48: BOOLEAN;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let mut command: UCHAR;

    command = (*regs).__bindgen_anon_2.bCommandReg.as_ref().clone();

    RtlZeroMemory(fis as PVOID, 20);

    *fis.add(0) = AHCI_FIS_TYPE_ATA_H2D as UCHAR;
    *fis.add(1) = 0x80 | ((DeviceNumber as UCHAR) & 0x0f);
    *fis.add(IDX_AHCI_o_DriveSelect as usize) = (IDE_DRIVE_SELECT_1
        | (if (AtaCommandFlags[command as usize] as ULONG
            & (ATA_CMD_FLAG_LBAIOsupp | ATA_CMD_FLAG_48))
            != 0
        {
            IDE_USE_LBA
        } else {
            0
        })) as UCHAR;
    *fis.add(IDX_AHCI_o_Control as usize) = IDE_DC_A_4BIT as UCHAR;

    if command as ULONG == IDE_COMMAND_ATAPI_PACKET {
        return 0;
    } else {
        need48 = ((((*regs).bOpFlags as ULONG & ATA_FLAGS_48BIT_COMMAND) != 0)
            && (*(*chan).lun[DeviceNumber as usize])
                .IdentifyData.FeaturesSupport.Address48() != 0) as BOOLEAN;

        if need48 != 0 {
            if (AtaCommandFlags[command as usize] as ULONG & ATA_CMD_FLAG_48supp) != 0 {
                command = AtaCommands48[command as usize];
            } else {
                return 0;
            }
        }

        *fis.add(IDX_AHCI_o_Command as usize) = command;
        *fis.add(IDX_AHCI_o_Feature as usize) = (*regs).__bindgen_anon_1.bFeaturesReg.as_ref().clone();

        *fis.add(IDX_AHCI_o_BlockNumber as usize) = (*regs).bSectorNumberReg;
        *fis.add(IDX_AHCI_o_CylinderLow as usize) = (*regs).bCylLowReg;
        *fis.add(IDX_AHCI_o_CylinderHigh as usize) = (*regs).bCylHighReg;

        *fis.add(IDX_AHCI_o_BlockCount as usize) = (*regs).bSectorCountReg;

        if need48 != 0 {
            *fis.add(IDX_AHCI_o_Control as usize) |= IDE_DC_USE_HOB as UCHAR;

            *fis.add(IDX_AHCI_o_BlockNumberExp as usize) = (*regs).bSectorNumberRegH;
            *fis.add(IDX_AHCI_o_CylinderLowExp as usize) = (*regs).bCylLowRegH;
            *fis.add(IDX_AHCI_o_CylinderHighExp as usize) = (*regs).bCylHighRegH;

            *fis.add(IDX_AHCI_o_BlockCountExp as usize) = (*regs).bSectorCountRegH;

            *fis.add(IDX_AHCI_o_FeatureExp as usize) = (*regs).bFeaturesRegH;

            (*chan).ChannelCtrlFlags |= CTRFLAGS_LBA48;
        } else {
            (*chan).ChannelCtrlFlags &= !CTRFLAGS_LBA48;
        }
        *fis.add(IDX_AHCI_o_DriveSelect as usize) |= (*regs).bDriveHeadReg & 0x0f;
    }

    20
}

// ===========================================================================
//  UniataAhciWaitCommandReady
// ===========================================================================
pub unsafe extern "system" fn UniataAhciWaitCommandReady(
    chan: PHW_CHANNEL,
    timeout: ULONG,
) -> UCHAR {
    let mut IS: AHCI_IS_REG = core::mem::zeroed();
    let mut CI: ULONG = 0;
    let mut i: ULONG;
    let mut SError: ULONG;
    let tag: ULONG = 0;

    let timeout = timeout * 5;

    i = 0;
    while i < timeout {
        CI = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_CI as ULONG);
        if ((CI >> tag) & 0x01) == 0 {
            break;
        }
        *IS.Reg.as_mut() = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_IS as ULONG);
        if *IS.Reg.as_ref() != 0 {
            break;
        }
        SError = AtapiReadPort4(chan, IDX_SATA_SError);
        if SError != 0 {
            i = timeout;
            break;
        }
        ScsiPortStallExecution(200);
        i += 1;
    }
    let _ = CI;

    /* clear interrupt(s) */
    *IS.Reg.as_mut() = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_IS as ULONG);
    UniataAhciWriteChannelPort4(chan, IDX_AHCI_P_IS as ULONG, *IS.Reg.as_ref());

    if timeout != 0 && (i >= timeout) {
        return IDE_STATUS_WRONG as UCHAR;
    }

    IDE_STATUS_IDLE as UCHAR
}

// ===========================================================================
//  UniataAhciSendCommand
// ===========================================================================
pub unsafe extern "system" fn UniataAhciSendCommand(
    HwDeviceExtension: PVOID,
    lChannel: ULONG,
    DeviceNumber: ULONG,
    ahci_flags: USHORT,
    timeout: ULONG,
) -> UCHAR {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let tag: ULONG = 0;

    let AHCI_CL: PIDE_AHCI_CMD_LIST = &mut (*(*chan).AhciCtlBlock).cmd_list[tag as usize];

    (*AHCI_CL).prd_length = 0;
    (*AHCI_CL).cmd_flags = UniAtaAhciAdjustIoFlags(0, ahci_flags, 20, DeviceNumber);

    (*AHCI_CL).bytecount = 0;
    (*AHCI_CL).cmd_table_phys = (*chan).AHCI_CTL_PhAddr
        + core::mem::offset_of!(_IDE_AHCI_CHANNEL_CTL_BLOCK, cmd) as ULONGLONG;
    // alignment check (diagnostic only)

    UniataAhciWriteChannelPort4(chan, IDX_AHCI_P_CI as ULONG, 1 << tag);

    UniataAhciWaitCommandReady(chan, timeout)
}

// ===========================================================================
//  UniataAhciSendPIOCommand
// ===========================================================================
pub unsafe extern "system" fn UniataAhciSendPIOCommand(
    HwDeviceExtension: PVOID,
    lChannel: ULONG,
    DeviceNumber: ULONG,
    Srb: PSCSI_REQUEST_BLOCK,
    data: PUCHAR,
    length: ULONG,
    command: UCHAR,
    lba: ULONGLONG,
    bcount: USHORT,
    feature: USHORT,
    ahci_flags: USHORT,
    wait_flags: ULONG,
    timeout: ULONG,
) -> UCHAR {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let statusByte: UCHAR;
    let AtaReq: PATA_REQ;
    let fis_size: ULONG;
    let AHCI_CMD: PIDE_AHCI_CMD;
    let mut Srb = Srb;
    let mut ahci_flags = ahci_flags;
    let _ = bcount;

    if Srb.is_null() {
        Srb = BuildAhciInternalSrb(HwDeviceExtension, DeviceNumber, lChannel, data, length);
        if Srb.is_null() {
            return IDE_STATUS_WRONG as UCHAR;
        }
    }
    AtaReq = (*Srb).SrbExtension as PATA_REQ;

    AHCI_CMD = (*ata_req_ahci(AtaReq)).ahci_cmd_ptr;

    fis_size = UniataAhciSetupFIS_H2D(
        deviceExtension, DeviceNumber, lChannel,
        &mut (*AHCI_CMD).cfis[0] as *mut UCHAR,
        command, lba, bcount, feature,
    );

    if fis_size == 0 {
        return IDE_STATUS_WRONG as UCHAR;
    }

    ahci_flags = UniAtaAhciAdjustIoFlags(command, ahci_flags, fis_size, DeviceNumber);

    if !data.is_null() {
        if (ahci_flags as ULONG & ATA_AHCI_CMD_WRITE) != 0 {
            (*ata_req_core(AtaReq)).Flags &= !(REQ_FLAG_READ as UCHAR);
            (*Srb).SrbFlags |= SRB_FLAGS_DATA_OUT;
        } else {
            (*ata_req_core(AtaReq)).Flags |= REQ_FLAG_READ as UCHAR;
            (*Srb).SrbFlags |= SRB_FLAGS_DATA_IN;
        }
        if AtapiDmaSetup(HwDeviceExtension, DeviceNumber, lChannel, Srb, data, length) == 0 {
            return IDE_STATUS_WRONG as UCHAR;
        }
    }

    (*ata_req_ahci(AtaReq)).io_cmd_flags = ahci_flags;

    UniataAhciBeginTransaction(HwDeviceExtension, lChannel, DeviceNumber, Srb);

    if wait_flags == ATA_IMMEDIATE {
        statusByte = 0;
    } else {
        statusByte = UniataAhciWaitCommandReady(chan, timeout);
        UniataAhciStatus(HwDeviceExtension, lChannel, DeviceNumber);
        UniataAhciEndTransaction(HwDeviceExtension, lChannel, DeviceNumber, Srb);
    }

    statusByte
}

// ===========================================================================
//  UniataAhciSendPIOCommandDirect
// ===========================================================================
pub unsafe extern "system" fn UniataAhciSendPIOCommandDirect(
    HwDeviceExtension: PVOID,
    lChannel: ULONG,
    DeviceNumber: ULONG,
    Srb: PSCSI_REQUEST_BLOCK,
    regs: PIDEREGS_EX,
    wait_flags: ULONG,
    timeout: ULONG,
) -> UCHAR {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let statusByte: UCHAR;
    let AtaReq: PATA_REQ;
    let fis_size: ULONG;
    let AHCI_CMD: PIDE_AHCI_CMD;
    let mut ahci_flags: USHORT = 0;

    if Srb.is_null() {
        return IDE_STATUS_WRONG as UCHAR;
    }
    AtaReq = (*Srb).SrbExtension as PATA_REQ;

    AHCI_CMD = (*ata_req_ahci(AtaReq)).ahci_cmd_ptr;
    if AHCI_CMD.is_null() {
        return IDE_STATUS_WRONG as UCHAR;
    }

    if (*Srb).DataTransferLength != 0 {
        if ((*Srb).SrbFlags & SRB_FLAGS_DATA_OUT) != 0 {
            ahci_flags |= ATA_AHCI_CMD_WRITE as USHORT;
            (*ata_req_core(AtaReq)).Flags &= !(REQ_FLAG_READ as UCHAR);
        } else {
            (*ata_req_core(AtaReq)).Flags |= REQ_FLAG_READ as UCHAR;
        }
    }

    fis_size = UniataAhciSetupFIS_H2D_Direct(
        deviceExtension, DeviceNumber, lChannel,
        &mut (*AHCI_CMD).cfis[0] as *mut UCHAR,
        regs,
    );

    if fis_size == 0 {
        return IDE_STATUS_WRONG as UCHAR;
    }

    ahci_flags = UniAtaAhciAdjustIoFlags(
        (*regs).__bindgen_anon_2.bCommandReg.as_ref().clone(), ahci_flags, fis_size, DeviceNumber,
    );

    if (*Srb).DataTransferLength != 0 {
        if AtapiDmaSetup(
            HwDeviceExtension, DeviceNumber, lChannel, Srb,
            (*Srb).DataBuffer as PUCHAR, (*Srb).DataTransferLength,
        ) == 0
        {
            return IDE_STATUS_WRONG as UCHAR;
        }
    }

    (*ata_req_ahci(AtaReq)).io_cmd_flags = ahci_flags;

    UniataAhciBeginTransaction(HwDeviceExtension, lChannel, DeviceNumber, Srb);

    if wait_flags == ATA_IMMEDIATE {
        statusByte = 0;
    } else {
        statusByte = UniataAhciWaitCommandReady(chan, timeout);
        UniataAhciStatus(HwDeviceExtension, lChannel, DeviceNumber);
        UniataAhciEndTransaction(HwDeviceExtension, lChannel, DeviceNumber, Srb);
    }

    statusByte
}

// ===========================================================================
//  UniataAhciAbortOperation
// ===========================================================================
pub unsafe extern "system" fn UniataAhciAbortOperation(chan: PHW_CHANNEL) -> BOOLEAN {
    if UniataAhciStop(chan) == 0 {
        return FALSE as BOOLEAN;
    }
    if UniataAhciStopFR(chan) == 0 {
        return FALSE as BOOLEAN;
    }
    if UniataAhciCLO(chan) == 0 {
        return FALSE as BOOLEAN;
    }
    UniataAhciStartFR(chan);
    // AHCI 1.3.1 sec. 10.3.1: PxCMD.ST must not be set while PxTFD has BSY or
    // DRQ. CLO clears them only when CAP.SCLO says the HBA can, so the C could
    // restart a port whose device was still busy. Upstream's fatal-error path
    // (ahci_hw.c AtaAhciHandleFatalError, "Busy TFD") escalates to a port reset
    // instead; UniataAhciHardReset is that COMRESET, and it restarts the port
    // itself once the device is ready.
    let TFD = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_TFD as ULONG);
    if (TFD & (IDE_STATUS_BUSY | IDE_STATUS_DRQ)) != 0 {
        let mut signature: ULONG = 0;
        return (UniataAhciHardReset((*chan).DeviceExtension as PVOID, (*chan).lChannel, &mut signature) == 0)
            as BOOLEAN;
    }
    UniataAhciStart(chan);

    TRUE as BOOLEAN
}

// ===========================================================================
//  UniataAhciSoftReset
// ===========================================================================
pub unsafe extern "system" fn UniataAhciSoftReset(
    HwDeviceExtension: PVOID,
    lChannel: ULONG,
    DeviceNumber: ULONG,
) -> ULONG {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);

    let AHCI_CMD: PIDE_AHCI_CMD = &mut (*(*chan).AhciCtlBlock).cmd;
    let RCV_FIS: PUCHAR = &mut (*(*chan).AhciCtlBlock).rcv_fis.rfis[0] as *mut UCHAR;

    /* kick controller into sane state */
    if UniataAhciAbortOperation(chan) == 0 {
        return 0xffffffff;
    }

    /* pull reset active */
    RtlZeroMemory(
        (*AHCI_CMD).cfis.as_mut_ptr() as PVOID,
        core::mem::size_of_val(&(*AHCI_CMD).cfis),
    );
    (*AHCI_CMD).cfis[0] = AHCI_FIS_TYPE_ATA_H2D as UCHAR;
    (*AHCI_CMD).cfis[1] = (DeviceNumber as UCHAR) & 0x0f;
    (*AHCI_CMD).cfis[15] = (IDE_DC_A_4BIT | IDE_DC_RESET_CONTROLLER) as UCHAR;

    if UniataAhciSendCommand(
        HwDeviceExtension, lChannel, DeviceNumber,
        (ATA_AHCI_CMD_RESET | ATA_AHCI_CMD_CLR_BUSY) as USHORT, 100,
    ) as ULONG
        == IDE_STATUS_WRONG
    {
        return 0xffffffff;
    }
    ScsiPortStallExecution(50);

    /* pull reset inactive */
    RtlZeroMemory(
        (*AHCI_CMD).cfis.as_mut_ptr() as PVOID,
        core::mem::size_of_val(&(*AHCI_CMD).cfis),
    );
    (*AHCI_CMD).cfis[0] = AHCI_FIS_TYPE_ATA_H2D as UCHAR;
    (*AHCI_CMD).cfis[1] = (DeviceNumber as UCHAR) & 0x0f;
    (*AHCI_CMD).cfis[15] = IDE_DC_A_4BIT as UCHAR;
    if UniataAhciSendCommand(HwDeviceExtension, lChannel, DeviceNumber, 0, 3000) as ULONG
        == IDE_STATUS_WRONG
    {
        return 0xffffffff;
    }

    UniataAhciWaitReady(chan, 1);

    if ((*deviceExtension).HwFlags & UNIATA_AHCI_ALT_SIG) != 0 {
        let signature: ULONG;
        signature = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_SIG as ULONG);
        return signature;
    }

    UniataAhciUlongFromRFIS(RCV_FIS)
}

// ===========================================================================
//  UniataAhciWaitReady
// ===========================================================================
pub unsafe extern "system" fn UniataAhciWaitReady(chan: PHW_CHANNEL, timeout: ULONG) -> ULONG {
    let mut TFD: ULONG;
    let mut i: ULONG;

    TFD = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_TFD as ULONG);
    i = 0;
    while i < timeout && (TFD & (IDE_STATUS_DRQ | IDE_STATUS_BUSY)) != 0 {
        ScsiPortStallExecution(1000);
        TFD = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_TFD as ULONG);
        i += 1;
    }

    TFD
}

// ===========================================================================
//  UniataAhciHardReset
// ===========================================================================
pub unsafe extern "system" fn UniataAhciHardReset(
    HwDeviceExtension: PVOID,
    lChannel: ULONG,
    signature: PULONG,
) -> ULONG {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let TFD: ULONG;

    *signature = 0xffffffff;

    UniataAhciStop(chan);
    if UniataSataPhyEnable(HwDeviceExtension, lChannel, 0, UNIATA_SATA_RESET_ENABLE as BOOLEAN) as ULONG
        == IDE_STATUS_WRONG
    {
        return IDE_STATUS_WRONG;
    }

    /* Wait for clearing busy status. */
    TFD = UniataAhciWaitReady(chan, 15000);
    if (TFD & (IDE_STATUS_DRQ | IDE_STATUS_BUSY)) != 0 {
        return TFD;
    }

    *signature = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_SIG as ULONG);

    UniataAhciStart(chan);

    0
}

// ===========================================================================
//  UniataAhciReset
// ===========================================================================
pub unsafe extern "system" fn UniataAhciReset(HwDeviceExtension: PVOID, lChannel: ULONG) {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let CAP: ULONG;
    let mut signature: ULONG = 0;
    let mut i: ULONG;
    let VendorID: ULONG = (*deviceExtension).DevID & 0xffff;

    /* Disable port interrupts */
    UniataAhciWriteChannelPort4(chan, IDX_AHCI_P_IE as ULONG, 0);

    if UniataAhciHardReset(HwDeviceExtension, lChannel, &mut signature) != 0 {
        DbgPrint(b"[UNIATA] AhciReset ch=%u: HardReset FAILED sig=%08x\n\0".as_ptr().cast(),
            lChannel, signature); // ahci marker
        i = 0;
        while i < (*deviceExtension).NumberLuns {
            UniataForgetDevice((*chan).lun[i as usize]);
            i += 1;
        }

        /* enable wanted port interrupts */
        UniataAhciWriteChannelPort4(
            chan, IDX_AHCI_P_IE as ULONG,
            ATA_AHCI_P_IX_CPD | ATA_AHCI_P_IX_PRC | ATA_AHCI_P_IX_PC,
        );
        return;
    }

    /* enable wanted port interrupts */
    UniataAhciWriteChannelPort4(
        chan, IDX_AHCI_P_IE as ULONG,
        ATA_AHCI_P_IX_CPD | ATA_AHCI_P_IX_TFE | ATA_AHCI_P_IX_HBF
            | ATA_AHCI_P_IX_HBD | ATA_AHCI_P_IX_IF | ATA_AHCI_P_IX_OF
            | (ATA_AHCI_P_IX_PRC | ATA_AHCI_P_IX_PC)
            | ATA_AHCI_P_IX_DP | ATA_AHCI_P_IX_UF | ATA_AHCI_P_IX_SDB
            | ATA_AHCI_P_IX_DS | ATA_AHCI_P_IX_PS | ATA_AHCI_P_IX_DHR,
    );

    CAP = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_CAP as ULONG);
    if (CAP & AHCI_CAP_SPM) != 0 && (VendorID != ATA_MARVELL_ID) {
        signature = UniataAhciSoftReset(HwDeviceExtension, lChannel, AHCI_DEV_SEL_PM);
        if signature == 0xffffffff {
            signature = UniataAhciSoftReset(HwDeviceExtension, lChannel, 0);
        }
    } else {
        signature = UniataAhciSoftReset(HwDeviceExtension, lChannel, 0);
    }

    DbgPrint(b"[UNIATA] AhciReset ch=%u: sig=%08x\n\0".as_ptr().cast(),
        lChannel, signature); // ahci marker
    (*(*chan).lun[0]).DeviceFlags &=
        !(DFLAGS_ATAPI_DEVICE | DFLAGS_DEVICE_PRESENT | CTRFLAGS_AHCI_PM);
    match signature >> 16 {
        0x0000 => {
            (*(*chan).lun[0]).DeviceFlags |= DFLAGS_DEVICE_PRESENT;
            (*chan).PmLunMap = 0;
        }
        0x9669 => {
            if (*deviceExtension).NumberLuns > 1 {
                (*chan).ChannelCtrlFlags |= CTRFLAGS_AHCI_PM;
                UniataSataIdentifyPM(chan);
            }
        }
        0xeb14 => {
            (*(*chan).lun[0]).DeviceFlags |= DFLAGS_ATAPI_DEVICE | DFLAGS_DEVICE_PRESENT;
            (*chan).PmLunMap = 0;
        }
        _ => {
            (*(*chan).lun[0]).DeviceFlags |= DFLAGS_DEVICE_PRESENT;
            (*chan).PmLunMap = 0;
        }
    }
}

// ===========================================================================
//  UniataAhciStartFR / StopFR / Start / CLO / Stop
// ===========================================================================
pub unsafe extern "system" fn UniataAhciStartFR(chan: PHW_CHANNEL) {
    let CMD: ULONG;
    CMD = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_CMD as ULONG);
    UniataAhciWriteChannelPort4(chan, IDX_AHCI_P_CMD as ULONG, CMD | ATA_AHCI_P_CMD_FRE);
    UniataAhciReadChannelPort4(chan, IDX_AHCI_P_CMD as ULONG); /* flush */
}

pub unsafe extern "system" fn UniataAhciStopFR(chan: PHW_CHANNEL) -> BOOLEAN {
    let mut CMD: ULONG;
    let mut i: ULONG;

    CMD = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_CMD as ULONG);
    UniataAhciWriteChannelPort4(chan, IDX_AHCI_P_CMD as ULONG, CMD & !ATA_AHCI_P_CMD_FRE);

    i = 0;
    while i < 1000 {
        CMD = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_CMD as ULONG);
        if (CMD & ATA_AHCI_P_CMD_FR) == 0 {
            return TRUE as BOOLEAN;
        }
        ScsiPortStallExecution(1000);
        i += 1;
    }
    FALSE as BOOLEAN
}

pub unsafe extern "system" fn UniataAhciStart(chan: PHW_CHANNEL) {
    let IS: ULONG;
    let CMD: ULONG;
    let mut SError: SATA_SERROR_REG = core::mem::zeroed();

    /* clear SATA error register */
    *SError.Reg.as_mut() = AtapiReadPort4(chan, IDX_SATA_SError);
    // PxSERR is write-1-to-clear, and the C only READ it, despite the comment
    // above. Stale link bits then survive into UniataAhciWaitCommandReady, which
    // treats any nonzero SError as a timeout. Upstream's ahci_hw.c clears it
    // before every command issue and after PHY ready ("Required for a D2H FIS").
    AtapiWritePort4(chan, IDX_SATA_SError, *SError.Reg.as_ref());

    /* clear any interrupts pending on this channel */
    IS = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_IS as ULONG);
    UniataAhciWriteChannelPort4(chan, IDX_AHCI_P_IS as ULONG, IS);

    CMD = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_CMD as ULONG);
    UniataAhciWriteChannelPort4(
        chan, IDX_AHCI_P_CMD as ULONG,
        CMD | ATA_AHCI_P_CMD_ST
            | (if ((*chan).ChannelCtrlFlags & CTRFLAGS_AHCI_PM) != 0 { ATA_AHCI_P_CMD_PMA } else { 0 }),
    );
    UniataAhciReadChannelPort4(chan, IDX_AHCI_P_CMD as ULONG); /* flush */
}

pub unsafe extern "system" fn UniataAhciCLO(chan: PHW_CHANNEL) -> BOOLEAN {
    let CAP: ULONG;
    let mut CMD: ULONG;
    let mut i: ULONG;

    CAP = (*(*chan).DeviceExtension).AHCI_CAP;
    if (CAP & AHCI_CAP_SCLO) == 0 {
        return TRUE as BOOLEAN;
    }
    CMD = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_CMD as ULONG);
    CMD |= ATA_AHCI_P_CMD_CLO;
    UniataAhciWriteChannelPort4(chan, IDX_AHCI_P_CMD as ULONG, CMD);

    i = 0;
    while i < 1000 {
        CMD = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_CMD as ULONG);
        if (CMD & ATA_AHCI_P_CMD_CLO) == 0 {
            return TRUE as BOOLEAN;
        }
        ScsiPortStallExecution(1000);
        i += 1;
    }
    FALSE as BOOLEAN
}

pub unsafe extern "system" fn UniataAhciStop(chan: PHW_CHANNEL) -> BOOLEAN {
    let mut CMD: ULONG;
    let mut i: ULONG;

    CMD = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_CMD as ULONG);
    CMD &= !ATA_AHCI_P_CMD_ST;
    UniataAhciWriteChannelPort4(chan, IDX_AHCI_P_CMD as ULONG, CMD);

    i = 0;
    while i < 1000 {
        CMD = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_CMD as ULONG);
        if (CMD & ATA_AHCI_P_CMD_CR) == 0 {
            return TRUE as BOOLEAN;
        }
        ScsiPortStallExecution(1000);
        i += 1;
    }
    FALSE as BOOLEAN
}

// ===========================================================================
//  UniataAhciBeginTransaction
// ===========================================================================
pub unsafe extern "system" fn UniataAhciBeginTransaction(
    HwDeviceExtension: PVOID,
    lChannel: ULONG,
    DeviceNumber: ULONG,
    Srb: PSCSI_REQUEST_BLOCK,
) -> UCHAR {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let mut CMD: ULONG;
    let mut CMD0: ULONG;
    let AtaReq: PATA_REQ = (*Srb).SrbExtension as PATA_REQ;
    let tag: ULONG = 0;

    let AHCI_CL: PIDE_AHCI_CMD_LIST = &mut (*(*chan).AhciCtlBlock).cmd_list[tag as usize];

    if (*Srb).DataTransferLength != 0
        && ((*ata_req_core(AtaReq)).dma_entries == 0
            || (*ata_req_core(AtaReq)).dma_entries >= 0xffff)
    {
        return 0;
    }

    (*AHCI_CL).prd_length = (*ata_req_core(AtaReq)).dma_entries as USHORT;
    (*AHCI_CL).cmd_flags = (*ata_req_ahci(AtaReq)).io_cmd_flags;
    (*AHCI_CL).bytecount = 0;
    if (*ata_req_ahci(AtaReq)).ahci_base64 != 0 {
        (*AHCI_CL).cmd_table_phys = (*ata_req_ahci(AtaReq)).ahci_base64;
    } else if !(*ata_req_ahci(AtaReq)).ahci_cmd_ptr.is_null() {
        RtlCopyMemory(
            &mut (*(*chan).AhciCtlBlock).cmd as *mut IDE_AHCI_CMD as PVOID,
            (*ata_req_ahci(AtaReq)).ahci_cmd_ptr as PVOID,
            core::mem::offset_of!(_IDE_AHCI_CMD, prd_tab)
                + (*AHCI_CL).prd_length as usize * core::mem::size_of::<IDE_AHCI_PRD_ENTRY>(),
        );
        (*AHCI_CL).cmd_table_phys = (*chan).AHCI_CTL_PhAddr
            + core::mem::offset_of!(_IDE_AHCI_CHANNEL_CTL_BLOCK, cmd) as ULONGLONG;
    } else {
        return 0;
    }
    if ((*AHCI_CL).cmd_table_phys & AHCI_CMD_ALIGNEMENT_MASK) != 0 {
        return 0;
    }

    CMD = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_CMD as ULONG);
    CMD0 = CMD;
    // switch controller to ATAPI mode for ATA_PACKET commands only
    // ATAPI_DEVICE(chan, DeviceNumber)
    let is_atapi = ((*(*chan).lun[DeviceNumber as usize]).DeviceFlags & DFLAGS_ATAPI_DEVICE) != 0;
    let cmd_ptr = (*ata_req_ahci(AtaReq)).ahci_cmd_ptr;
    if is_atapi && (*cmd_ptr).cfis[2] as ULONG == IDE_COMMAND_ATAPI_PACKET {
        CMD |= ATA_AHCI_P_CMD_ATAPI;
    } else {
        CMD &= !ATA_AHCI_P_CMD_ATAPI;
    }
    if CMD0 != CMD {
        UniataAhciWriteChannelPort4(chan, IDX_AHCI_P_CMD as ULONG, CMD);
        CMD0 = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_CMD as ULONG); /* flush */
    }

    /* issue command to controller */
    UniataAhciWriteChannelPort4(chan, IDX_AHCI_P_CI as ULONG, 0x01 << tag);
    (*chan).AhciPrevCI |= 0x01 << tag;

    CMD |= ATA_AHCI_P_CMD_ST
        | (if ((*chan).ChannelCtrlFlags & CTRFLAGS_AHCI_PM) != 0 { ATA_AHCI_P_CMD_PMA } else { 0 });
    if CMD != CMD0 {
        UniataAhciWriteChannelPort4(chan, IDX_AHCI_P_CMD as ULONG, CMD);
        CMD0 = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_CMD as ULONG); /* flush */
    }
    let _ = CMD0;

    if !is_atapi {
        if (*cmd_ptr).cfis[2] as ULONG == IDE_COMMAND_ATAPI_RESET {
            let mut TFD: ULONG = 0;
            let mut i: ULONG = 0;

            while i < 1000000 {
                TFD = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_TFD as ULONG);
                if (TFD & IDE_STATUS_BUSY) == 0 {
                    break;
                }
                i += 1;
            }
            (*ata_req_ahci(AtaReq)).in_status = TFD;

            return IDE_STATUS_SUCCESS as UCHAR;
        }
    }

    DbgPrint(b"[UNIATA] AhciBeginTx ch=%u: GHC=%08x hIS=%08x PxIE=%08x PxIS=%08x CI=%08x\n\0"
        .as_ptr().cast(),
        lChannel,
        UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_GHC as ULONG),
        UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_IS as ULONG),
        UniataAhciReadChannelPort4(chan, IDX_AHCI_P_IE as ULONG),
        UniataAhciReadChannelPort4(chan, IDX_AHCI_P_IS as ULONG),
        UniataAhciReadChannelPort4(chan, IDX_AHCI_P_CI as ULONG)); // ahci marker

    IDE_STATUS_IDLE as UCHAR
}

// ===========================================================================
//  UniataAhciEndTransaction
// ===========================================================================
pub unsafe extern "system" fn UniataAhciEndTransaction(
    HwDeviceExtension: PVOID,
    lChannel: ULONG,
    DeviceNumber: ULONG,
    Srb: PSCSI_REQUEST_BLOCK,
) -> UCHAR {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let CI: ULONG;
    let ACT: ULONG;
    let AtaReq: PATA_REQ = (*Srb).SrbExtension as PATA_REQ;
    let TFD: ULONG;
    let RCV_FIS: PUCHAR = &mut (*(*chan).AhciCtlBlock).rcv_fis.rfis[0] as *mut UCHAR;
    let tag: ULONG = 0;
    let _ = DeviceNumber;
    let AHCI_CL: PIDE_AHCI_CMD_LIST = &mut (*(*chan).AhciCtlBlock).cmd_list[tag as usize];

    TFD = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_TFD as ULONG);

    if (TFD & IDE_STATUS_ERROR) != 0 {
        (*ata_req_ahci(AtaReq)).in_error = (TFD >> 8) as UCHAR;
    } else {
        (*ata_req_ahci(AtaReq)).in_error = 0;
    }
    (*ata_req_ahci(AtaReq)).in_status = TFD;

    (*ata_req_ahci(AtaReq)).in_bcount = (*RCV_FIS.add(12) as ULONG) | ((*RCV_FIS.add(13) as ULONG) << 8);
    (*ata_req_ahci(AtaReq)).in_lba = (*RCV_FIS.add(4) as ULONGLONG)
        | ((*RCV_FIS.add(5) as ULONGLONG) << 8)
        | ((*RCV_FIS.add(6) as ULONGLONG) << 16);
    if ((*chan).ChannelCtrlFlags & CTRFLAGS_LBA48) != 0 {
        (*ata_req_ahci(AtaReq)).in_lba |= ((*RCV_FIS.add(8) as ULONGLONG) << 24)
            | ((*RCV_FIS.add(9) as ULONGLONG) << 32)
            | ((*RCV_FIS.add(10) as ULONGLONG) << 40);
    } else {
        (*ata_req_ahci(AtaReq)).in_lba |= ((*RCV_FIS.add(8) as ULONGLONG) << 24)
            | ((*RCV_FIS.add(9) as ULONGLONG) << 32)
            | (((*RCV_FIS.add(7) as ULONGLONG) & 0x0f) << 24);
    }
    (*ata_req_core(AtaReq)).WordsTransfered = (*AHCI_CL).bytecount / 2;

    ACT = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_ACT as ULONG);
    CI = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_CI as ULONG);
    let _ = ACT;
    if (CI & (1 << tag)) != 0 {
        if UniataAhciAbortOperation(chan) == 0 {
            // need RESET
        }
        (*chan).AhciPrevCI = CI & !((1u32) << tag);
    } else {
        (*chan).AhciPrevCI &= !((1u32) << tag);
        RtlZeroMemory(AHCI_CL as PVOID, core::mem::size_of::<IDE_AHCI_CMD_LIST>());
    }

    0
}

// ===========================================================================
//  UniataAhciResume
// ===========================================================================
pub unsafe extern "system" fn UniataAhciResume(chan: PHW_CHANNEL) {
    let mut base: ULONGLONG;

    /* Disable port interrupts */
    UniataAhciWriteChannelPort4(chan, IDX_AHCI_P_IE as ULONG, 0);

    /* setup work areas */
    base = (*chan).AHCI_CTL_PhAddr;
    if base == 0 {
        return;
    }
    UniataAhciWriteChannelPort4(chan, IDX_AHCI_P_CLB as ULONG, (base & 0xffffffff) as ULONG);
    UniataAhciWriteChannelPort4(chan, IDX_AHCI_P_CLB as ULONG + 4, ((base >> 32) & 0xffffffff) as ULONG);

    base = (*chan).AHCI_CTL_PhAddr
        + core::mem::offset_of!(_IDE_AHCI_CHANNEL_CTL_BLOCK, rcv_fis) as ULONGLONG;
    UniataAhciWriteChannelPort4(chan, IDX_AHCI_P_FB as ULONG, (base & 0xffffffff) as ULONG);
    UniataAhciWriteChannelPort4(chan, IDX_AHCI_P_FB as ULONG + 4, ((base >> 32) & 0xffffffff) as ULONG);

    /* activate the channel and power/spin up device */
    UniataAhciWriteChannelPort4(
        chan, IDX_AHCI_P_CMD as ULONG,
        ATA_AHCI_P_CMD_ACTIVE | ATA_AHCI_P_CMD_POD | ATA_AHCI_P_CMD_SUD
            | (if ((*chan).ChannelCtrlFlags & CTRFLAGS_AHCI_PM) != 0 { ATA_AHCI_P_CMD_ALPE } else { 0 })
            | (if ((*chan).ChannelCtrlFlags & CTRFLAGS_AHCI_PM2) != 0 { ATA_AHCI_P_CMD_ASP } else { 0 }),
    );
    UniataAhciReadChannelPort4(chan, IDX_AHCI_P_CMD as ULONG); /* flush */

    UniataAhciStartFR(chan);
    UniataAhciStart(chan);
}

// ===========================================================================
//  UniataAhciReadPM
// ===========================================================================
pub unsafe extern "system" fn UniataAhciReadPM(
    chan: PHW_CHANNEL,
    DeviceNumber: ULONG,
    Reg: ULONG,
    result: PULONG,
) -> BOOLEAN {
    let mut Reg = Reg;
    let AHCI_CMD: PIDE_AHCI_CMD = &mut (*(*chan).AhciCtlBlock).cmd;
    let RCV_FIS: PUCHAR = &mut (*(*chan).AhciCtlBlock).rcv_fis.rfis[0] as *mut UCHAR;

    if DeviceNumber == DEVNUM_NOT_SPECIFIED {
        *result = UniataSataReadPort4(chan, Reg, 0);
        return TRUE as BOOLEAN;
    }
    if DeviceNumber < AHCI_DEV_SEL_PM {
        match Reg {
            x if x == IDX_SATA_SStatus as ULONG => Reg = 0,
            x if x == IDX_SATA_SError as ULONG => Reg = 1,
            x if x == IDX_SATA_SControl as ULONG => Reg = 2,
            _ => return FALSE as BOOLEAN,
        }
    }

    RtlZeroMemory(
        (*AHCI_CMD).cfis.as_mut_ptr() as PVOID,
        core::mem::size_of_val(&(*AHCI_CMD).cfis),
    );
    (*AHCI_CMD).cfis[0] = AHCI_FIS_TYPE_ATA_H2D as UCHAR;
    (*AHCI_CMD).cfis[1] = AHCI_FIS_COMM_PM;
    (*AHCI_CMD).cfis[2] = IDE_COMMAND_READ_PM as UCHAR;
    (*AHCI_CMD).cfis[3] = Reg as UCHAR;
    (*AHCI_CMD).cfis[7] = (IDE_USE_LBA | DeviceNumber) as UCHAR;
    (*AHCI_CMD).cfis[15] = IDE_DC_A_4BIT as UCHAR;

    if UniataAhciSendCommand((*chan).DeviceExtension as PVOID, (*chan).lChannel, DeviceNumber, 0, 10) as ULONG
        == IDE_STATUS_WRONG
    {
        return FALSE as BOOLEAN;
    }

    *result = UniataAhciUlongFromRFIS(RCV_FIS);
    TRUE as BOOLEAN
}

// ===========================================================================
//  UniataAhciWritePM
// ===========================================================================
pub unsafe extern "system" fn UniataAhciWritePM(
    chan: PHW_CHANNEL,
    DeviceNumber: ULONG,
    Reg: ULONG,
    value: ULONG,
) -> UCHAR {
    let mut Reg = Reg;
    let TFD: ULONG;
    let AHCI_CMD: PIDE_AHCI_CMD = &mut (*(*chan).AhciCtlBlock).cmd;

    if DeviceNumber == DEVNUM_NOT_SPECIFIED {
        UniataSataWritePort4(chan, Reg, value, 0);
        return 0;
    }
    if DeviceNumber < AHCI_DEV_SEL_PM {
        match Reg {
            x if x == IDX_SATA_SStatus as ULONG => Reg = 0,
            x if x == IDX_SATA_SError as ULONG => Reg = 1,
            x if x == IDX_SATA_SControl as ULONG => Reg = 2,
            _ => return IDE_STATUS_WRONG as UCHAR,
        }
    }

    RtlZeroMemory(
        (*AHCI_CMD).cfis.as_mut_ptr() as PVOID,
        core::mem::size_of_val(&(*AHCI_CMD).cfis),
    );
    (*AHCI_CMD).cfis[0] = AHCI_FIS_TYPE_ATA_H2D as UCHAR;
    (*AHCI_CMD).cfis[1] = AHCI_FIS_COMM_PM;
    (*AHCI_CMD).cfis[2] = IDE_COMMAND_WRITE_PM as UCHAR;
    (*AHCI_CMD).cfis[3] = Reg as UCHAR;
    (*AHCI_CMD).cfis[7] = (IDE_USE_LBA | DeviceNumber) as UCHAR;

    (*AHCI_CMD).cfis[12] = (value & 0xff) as UCHAR;
    (*AHCI_CMD).cfis[4] = ((value >> 8) & 0xff) as UCHAR;
    (*AHCI_CMD).cfis[5] = ((value >> 16) & 0xff) as UCHAR;
    (*AHCI_CMD).cfis[6] = ((value >> 24) & 0xff) as UCHAR;

    (*AHCI_CMD).cfis[15] = IDE_DC_A_4BIT as UCHAR;

    if UniataAhciSendCommand((*chan).DeviceExtension as PVOID, (*chan).lChannel, DeviceNumber, 0, 100) as ULONG
        == IDE_STATUS_WRONG
    {
        return IDE_STATUS_WRONG as UCHAR;
    }

    TFD = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_TFD as ULONG);

    (TFD >> 8) as UCHAR
}

// ===========================================================================
//  UniataAhciSetupCmdPtr
// ===========================================================================
pub unsafe extern "system" fn UniataAhciSetupCmdPtr(AtaReq: PATA_REQ) {
    // C: prd_base = &AtaReq->ahci_cmd0;
    //    prd_base64 = (prd_base + max(FIELD_OFFSET(ATA_REQ, ahci_cmd0),
    //                  AHCI_CMD_ALIGNEMENT_MASK+1)) & ~AHCI_CMD_ALIGNEMENT_MASK;
    // ahci_cmd0 lives in the second sub-union (__bindgen_anon_2); use its offset as
    // the FIELD_OFFSET(ATA_REQ, ahci_cmd0) proxy.
    let prd_base: ULONGLONG = ata_req_ahci_cmd0(AtaReq) as ULONGLONG;
    let fo_ahci_cmd0: ULONGLONG = core::mem::offset_of!(_ATA_REQ, __bindgen_anon_1) as ULONGLONG;
    let prd_base64: ULONGLONG =
        (prd_base + max_u64(fo_ahci_cmd0, AHCI_CMD_ALIGNEMENT_MASK + 1)) & !AHCI_CMD_ALIGNEMENT_MASK;

    (*ata_req_ahci(AtaReq)).ahci_cmd_ptr = prd_base64 as PIDE_AHCI_CMD;
}

#[inline(always)]
fn max_u64(a: ULONGLONG, b: ULONGLONG) -> ULONGLONG { if a > b { a } else { b } }

// ===========================================================================
//  BuildAhciInternalSrb
// ===========================================================================
pub unsafe extern "system" fn BuildAhciInternalSrb(
    HwDeviceExtension: PVOID,
    DeviceNumber: ULONG,
    lChannel: ULONG,
    Buffer: PUCHAR,
    Length: ULONG,
) -> PSCSI_REQUEST_BLOCK {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let srb: PSCSI_REQUEST_BLOCK;
    let AtaReq: PATA_REQ = (*chan).AhciInternalAtaReq;

    if AtaReq.is_null() {
        return core::ptr::null_mut();
    }

    UniAtaClearAtaReq(AtaReq as PVOID);

    srb = (*chan).AhciInternalSrb;

    RtlZeroMemory(srb as PVOID, core::mem::size_of::<SCSI_REQUEST_BLOCK>());

    (*srb).PathId = lChannel as UCHAR;
    (*srb).TargetId = DeviceNumber as UCHAR;
    (*srb).Function = SRB_FUNCTION_EXECUTE_SCSI as UCHAR;
    (*srb).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;

    // Set timeout to 4 seconds.
    (*srb).TimeOutValue = 4;

    (*srb).CdbLength = 6;
    (*srb).DataBuffer = Buffer as PVOID;
    (*srb).DataTransferLength = Length;
    (*srb).SrbExtension = AtaReq as PVOID;

    (*ata_req_core(AtaReq)).Srb = srb;
    (*ata_req_core(AtaReq)).DataBuffer = Buffer as PUSHORT;
    (*ata_req_core(AtaReq)).TransferLength = Length;

    srb
}
