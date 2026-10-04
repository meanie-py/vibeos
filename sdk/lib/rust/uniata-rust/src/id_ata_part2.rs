//! id_ata_part2.rs — middle contiguous block of id_ata.cpp:
//! identify / reset / interrupt handling. Translated per TRANSLATION_MANIFEST.md.
//!
//! Live compile path assumed (matching config.h / id_ata.cpp #defines):
//!   - UNIATA_CORE              : NOT defined  (so #ifndef UNIATA_CORE bodies live)
//!   - UNIATA_USE_XXableInterrupts : NOT defined (=> RETTYPE_XXableInterrupts == VOID,
//!                                  InDpc = chan->DpcState != DPC_STATE_ISR, timer-DPC path)
//!   - UNIATA_PIO_ONLY          : NOT defined
//!   - UNIATA_INIT_CHANGERS     : NOT defined
//!   - UNIATA_DUMP_ATAPI        : NOT defined
//!   - NAVO_TEST                : NOT defined
//!   - IO_STATISTICS            : DEFINED
//!   - _DEBUG                   : NOT defined  (KdPrint*/UniataDump*/BrutePoint dropped)
//!
//! RESET_COMPLETE_* (local #defines in id_ata.cpp):
//!   RESET_COMPLETE_CURRENT = 0, RESET_COMPLETE_ALL = 1, RESET_COMPLETE_NONE = 2

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code, unused_unsafe)]
#![allow(unused_variables, unused_mut, unused_assignments, unused_parens)]

use crate::types::*;
use crate::regidx::*;
use crate::*;
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::driver_ddk::DbgPrint; // perf phase markers
use core::ffi::{c_uint, c_int};
use ntkrnl_sys::base::{TRUE, FALSE, PCVOID};
use ntkrnl_sys::interlocked::{InterlockedIncrement, InterlockedDecrement, InterlockedExchangeAdd};
use ntkrnl_sys::rtl::{RtlZeroMemory, RtlCopyMemory};
use crate::io_port::{
    AtapiReadPort1, AtapiReadPort2, AtapiReadPort4, AtapiReadPortEx1, AtapiReadPortEx4,
    AtapiWritePort1, AtapiWritePort2, AtapiWritePort4, AtapiWritePortEx1, AtapiWritePortEx4,
    AtapiReadBuffer2, AtapiReadBuffer4, AtapiWriteBuffer2, AtapiWriteBuffer4, UniataNanoSleep,
};

// ---- local #defines from id_ata.cpp ----
const RESET_COMPLETE_CURRENT: BOOLEAN = 0x00;
const RESET_COMPLETE_ALL: BOOLEAN = 0x01;
const RESET_COMPLETE_NONE: BOOLEAN = 0x02;

// PIO0_TIMING (bm_devs_decl.h) == 600. UniataGetPioTiming(LunExt) is a macro that
// expands to the constant 0 (bm_devs_decl.h override), so we inline 0 at its call sites.
const PIO0_TIMING: ULONG = 600;

#[inline(always)]
unsafe fn UniataGetPioTiming(_LunExt: PHW_LU_EXTENSION) -> ULONG { 0 }

// ---- standard interlocked ops on LONG (CheckIntr/DisableIntr/ReCheckIntr are LONG) ----
// ntkrnl-sys provides pointer-sized / Add / Inc / Dec but not the bare 32-bit Exchange /
// CompareExchange. CrNtInterlocked{ExchangeAdd,CompareExchange} == Interlocked{...}.
#[inline(always)]
unsafe fn InterlockedExchange(target: *mut LONG, value: LONG) -> LONG {
    (*(target as *const core::sync::atomic::AtomicI32))
        .swap(value, core::sync::atomic::Ordering::SeqCst)
}
#[inline(always)]
unsafe fn InterlockedCompareExchange(dest: *mut LONG, exchange: LONG, comparand: LONG) -> LONG {
    match (*(dest as *const core::sync::atomic::AtomicI32)).compare_exchange(
        comparand, exchange,
        core::sync::atomic::Ordering::SeqCst, core::sync::atomic::Ordering::SeqCst,
    ) {
        Ok(v) => v,
        Err(v) => v,
    }
}
// CrNt aliases (config.h: direct kernel calls on ReactOS-native target)
#[inline(always)]
unsafe fn CrNtInterlockedExchangeAdd(addend: *mut LONG, value: LONG) -> LONG {
    InterlockedExchangeAdd(addend, value)
}
#[inline(always)]
unsafe fn CrNtInterlockedCompareExchange(dest: *mut LONG, exchange: LONG, comparand: LONG) -> LONG {
    InterlockedCompareExchange(dest, exchange, comparand)
}

// ============================================================================
// AtapiTimerDpc / AtapiQueueTimerDpc  (#ifndef UNIATA_CORE)
// ============================================================================

pub unsafe extern "system" fn AtapiTimerDpc(
    HwDeviceExtension: PVOID,
) {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let mut HwScsiTimer: PHW_TIMER;
    let mut time = core::mem::zeroed::<LARGE_INTEGER>();
    let mut MiniportTimerValue: ULONG;
    let mut recall: BOOLEAN = FALSE;
    let mut lChannel: ULONG;
    let mut chan: PHW_CHANNEL;

    lChannel = (*deviceExtension).FirstDpcChan;
    (*deviceExtension).ActiveDpcChan = lChannel;
    if lChannel == CHAN_NOT_SPECIFIED {
        return;
    }
    chan = (*deviceExtension).chan.add(lChannel as usize);

    loop {
        HwScsiTimer = (*chan).HwScsiTimer;
        (*chan).HwScsiTimer = None;

        (*deviceExtension).FirstDpcChan = (*chan).NextDpcChan;
        if (*deviceExtension).FirstDpcChan != CHAN_NOT_SPECIFIED {
            recall = TRUE;
        }

        if let Some(f) = HwScsiTimer {
            f(HwDeviceExtension);
        }

        (*chan).NextDpcChan = CHAN_NOT_SPECIFIED;

        lChannel = (*deviceExtension).FirstDpcChan;
        (*deviceExtension).ActiveDpcChan = lChannel;
        if lChannel == CHAN_NOT_SPECIFIED {
            (*deviceExtension).FirstDpcChan = CHAN_NOT_SPECIFIED;
            (*deviceExtension).ActiveDpcChan = CHAN_NOT_SPECIFIED;
            return;
        }

        KeQuerySystemTime(&mut time);

        chan = (*deviceExtension).chan.add(lChannel as usize);
        if time.QuadPart >= (*chan).DpcTime - 10 {
            // call now
            continue;
        }
        break;
    }

    if recall != 0 {
        (*deviceExtension).ActiveDpcChan = CHAN_NOT_SPECIFIED;
        MiniportTimerValue = ((time.QuadPart - (*chan).DpcTime) / 10) as ULONG;
        if MiniportTimerValue == 0 {
            MiniportTimerValue = 1;
        }
        ScsiPortNotification(
            _SCSI_NOTIFICATION_TYPE_RequestTimerCall as SCSI_NOTIFICATION_TYPE,
            HwDeviceExtension,
            AtapiTimerDpc as PVOID,
            MiniportTimerValue,
        );
    }
}

// Wrapper for ScsiPort that implements smart Dpc queueing.
pub unsafe extern "system" fn AtapiQueueTimerDpc(
    HwDeviceExtension: PVOID,
    lChannel: ULONG,
    HwScsiTimer: PHW_TIMER,
    mut MiniportTimerValue: ULONG,
) {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let mut time = core::mem::zeroed::<LARGE_INTEGER>();
    let time2: LARGE_INTEGER;
    let mut i: ULONG;
    let mut prev_chan: PHW_CHANNEL;
    let mut chan: PHW_CHANNEL;

    KeQuerySystemTime(&mut time);
    time2 = time;
    time.QuadPart += (MiniportTimerValue as LONGLONG) * 10;

    i = (*deviceExtension).FirstDpcChan;
    chan = core::ptr::null_mut();
    prev_chan = core::ptr::null_mut();
    while i != CHAN_NOT_SPECIFIED {
        prev_chan = chan;
        chan = (*deviceExtension).chan.add(i as usize);
        if (*chan).DpcTime > time.QuadPart {
            break;
        }
        i = (*chan).NextDpcChan;
    }
    chan = (*deviceExtension).chan.add(lChannel as usize);
    if prev_chan.is_null() {
        (*deviceExtension).FirstDpcChan = lChannel;
    } else {
        (*prev_chan).NextDpcChan = lChannel;
    }
    (*chan).NextDpcChan = i;
    (*chan).HwScsiTimer = HwScsiTimer;
    (*chan).DpcTime = time.QuadPart;

    if time.QuadPart <= time2.QuadPart {
        MiniportTimerValue = 1;
    } else {
        MiniportTimerValue = ((time.QuadPart - time2.QuadPart) / 10) as ULONG;
    }

    ScsiPortNotification(
        _SCSI_NOTIFICATION_TYPE_RequestTimerCall as SCSI_NOTIFICATION_TYPE,
        HwDeviceExtension,
        AtapiTimerDpc as PVOID,
        MiniportTimerValue,
    );
}

// ============================================================================
// UniataSnapAtaRegs
// ============================================================================

pub unsafe extern "system" fn UniataSnapAtaRegs(
    chan: PHW_CHANNEL,
    DeviceNumber: ULONG,
    regs: PIDEREGS_EX,
) {
    if (*(*chan).DeviceExtension).HwFlags & UNIATA_AHCI != 0 {
        // AHCI
        UniataAhciSnapAtaRegs(chan, DeviceNumber, regs);
    } else {
        // SATA/PATA, assume drive is selected
        let mut j: ULONG;
        let mut statusByteAlt: UCHAR;
        let regs_b = regs as *mut UCHAR;

        if ((*regs).bOpFlags as u32 & ATA_FLAGS_48BIT_COMMAND) == 0 {
            j = IDX_IO1_i_Error as ULONG;
            while j <= IDX_IO1_i_Status as ULONG {
                statusByteAlt = AtapiReadPort1(chan, crate::regidx::IDX_IO1 + j as ULONG_PTR);
                *regs_b.add((j - 1) as usize) = statusByteAlt;
                j += 1;
            }
            (*regs).bOpFlags = 0;
        } else {
            (*regs).bDriveHeadReg = AtapiReadPort1(chan, IDX_IO1_i_DriveSelect);
            j = IDX_IO1_i_Error as ULONG;
            while j < IDX_IO1_i_DriveSelect as ULONG {
                statusByteAlt = AtapiReadPort1(chan, crate::regidx::IDX_IO1 + j as ULONG_PTR);
                *regs_b.add((j - 1) as usize) = statusByteAlt;
                statusByteAlt = AtapiReadPort1(chan, crate::regidx::IDX_IO1 + j as ULONG_PTR);
                *regs_b.add((j + 8 - 1) as usize) = statusByteAlt;
                j += 1;
            }
            *(*regs).__bindgen_anon_2.bCommandReg.as_mut() = AtapiReadPort1(chan, IDX_IO1_i_Status);
        }
    }
}

extern "C" {
    // CRT: case-insensitive bounded compare. AtapiStringCmp(a,b,n) -> _strnicmp(a,b,n).
    fn _strnicmp(a: PCCHAR, b: PCCHAR, n: usize) -> core::ffi::c_int;
}

// ---- ATA_REQ field accessors (bindgen buries the real fields under nested anon
//      unions: _ATA_REQ.__bindgen_anon_1(union)->bindgen_ty_1.__bindgen_anon_1(struct)) ----
#[inline(always)]
unsafe fn ata_req_core(
    r: PATA_REQ,
) -> *mut _ATA_REQ__bindgen_ty_1__bindgen_ty_1 {
    &mut (*r).__bindgen_anon_1.as_mut().__bindgen_anon_1
}
#[inline(always)]
unsafe fn ata_req_ahci(
    r: PATA_REQ,
) -> *mut _ATA_REQ__bindgen_ty_1__bindgen_ty_1__bindgen_ty_1__bindgen_ty_2 {
    (*ata_req_core(r)).__bindgen_anon_1.ahci.as_mut() as *mut _
}

// SMART IOCTL constants (ntdddisk.h; not emitted by bindgen into the uniata types).
const SMART_CMD: UCHAR = 0xB0;
const SMART_IDE_ERROR: UCHAR = 1;
const RETURN_SMART_STATUS: UCHAR = 0xDA;

const PCIConfiguration: ULONG = 4; // BUS_DATA_TYPE::PCIConfiguration

// ---- macro-expression #defines bindgen drops (bsmaster.h) ----
const ATA_MAX_LBA28: ULONGLONG = 0x0fffffff;
const ATA_MAX_IOLBA28: ULONGLONG = 0x0fffff80;
const REORDER_COST_MAX: LONGLONG = (0x1i64 << 60) - 1;
const REORDER_COST_SWITCH_RW_CD: LONGLONG = REORDER_COST_MAX / 8;

// ============================================================================
// IssueIdentify — issue IDENTIFY (EC) or ATAPI packet IDENTIFY (A1) to a device.
// ============================================================================

pub unsafe extern "system" fn IssueIdentify(
    HwDeviceExtension: PVOID,
    DeviceNumber: ULONG,
    lChannel: ULONG,
    Command: UCHAR,
    NoSetup: BOOLEAN,
) -> BOOLEAN {
    issue_identify_once(HwDeviceExtension, DeviceNumber, lChannel, Command, NoSetup, true)
}

/// SET FEATURES subcommand 07h: spin up a drive that powered up in standby.
const ATA_C_F_PUIS_SPINUP: UCHAR = 0x07;

/// IDENTIFY words that say the data is not the real thing yet (ACS-3 7.12.7):
/// word 2 = 37C8h/738Ch means the drive is in Power-Up-In-Standby and needs
/// SET FEATURES 07h before it answers properly; word 0 bit 2 means the
/// response was incomplete. Same tests as upstream's identify_funcs.h
/// (AtaDevInPuisState / AtaDevIsIdentifyDataIncomplete).
unsafe fn identify_needs_spinup(fid: *const IDENTIFY_DATA) -> bool {
    let w = fid as *const USHORT;
    let word0 = *w;
    let word2 = *w.add(2);
    word2 == 0x37C8 || word2 == 0x738C || (word0 & (1 << 2)) != 0
}

/// Word 255: low byte A5h marks an integrity checksum, and then all 512 bytes
/// must sum to 0 mod 256. No signature means the drive does not provide one,
/// which is valid (upstream AtaDevIsIdentifyDataValid).
unsafe fn identify_checksum_ok(fid: *const IDENTIFY_DATA) -> bool {
    let b = fid as *const UCHAR;
    if *b.add(510) != 0xA5 {
        return true;
    }
    let mut sum: UCHAR = 0;
    for i in 0..512 {
        sum = sum.wrapping_add(*b.add(i));
    }
    sum == 0
}

/// Send SET FEATURES 07h and give the drive the 20 seconds upstream allows
/// (dev_identify.c AtaDeviceSpinUp): a spin-up is seconds, far past the 1s
/// every other uniata command waits.
unsafe fn spin_up_puis_drive(
    deviceExtension: PHW_DEVICE_EXTENSION,
    chan: PHW_CHANNEL,
    DeviceNumber: ULONG,
    lChannel: ULONG,
    use_ahci: bool,
) -> UCHAR {
    if use_ahci {
        return UniataAhciSendPIOCommand(
            deviceExtension as PVOID, lChannel, DeviceNumber,
            core::ptr::null_mut(), core::ptr::null_mut(), 0,
            IDE_COMMAND_SET_FEATURES as UCHAR, 0, 0, ATA_C_F_PUIS_SPINUP as USHORT,
            0, ATA_WAIT_INTR as ULONG, 20000,
        );
    }
    let mut statusByte = AtaCommand(
        deviceExtension, DeviceNumber, lChannel,
        IDE_COMMAND_SET_FEATURES as UCHAR, 0, 0, 0,
        0, ATA_C_F_PUIS_SPINUP, ATA_WAIT_BASE_READY as ULONG,
    );
    // WaitOnBaseBusyLong is ~0.5s; 40 of them is upstream's 20s.
    let mut n = 0;
    while (statusByte as u32 & IDE_STATUS_BUSY) != 0 && n < 40 {
        statusByte = WaitOnBaseBusyLong(chan);
        n += 1;
    }
    statusByte
}

unsafe fn issue_identify_once(
    HwDeviceExtension: PVOID,
    DeviceNumber: ULONG,
    lChannel: ULONG,
    Command: UCHAR,
    NoSetup: BOOLEAN,
    allow_spinup: bool,
) -> BOOLEAN {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let mut waitCount: ULONG = 50000;
    let mut j: ULONG;
    let mut statusByte: UCHAR;
    let mut signatureLow: UCHAR;
    let mut signatureHigh: UCHAR;
    let mut atapiDev: BOOLEAN = FALSE;
    let use_ahci: BOOLEAN;
    let LunExt: PHW_LU_EXTENSION = (*chan).lun[DeviceNumber as usize];

    use_ahci = (UniataIsSATARangeAvailable(deviceExtension, lChannel) != 0
        && ((*deviceExtension).HwFlags & UNIATA_AHCI) != 0) as BOOLEAN;

    DbgPrint(b"[UNIATA] IssueIdentify ch=%u dev=%u cmd=%x\n\0".as_ptr().cast(), lChannel, DeviceNumber, Command as ULONG); // perf phase marker
    if (*chan).ChannelCtrlFlags & CTRFLAGS_AHCI_PM != 0 {
        if (*chan).PmLunMap & (1 << DeviceNumber) != 0 {
            // OK
        } else {
            return FALSE;
        }
    } else if DeviceNumber != 0 && ((*chan).ChannelCtrlFlags & CTRFLAGS_NO_SLAVE) != 0 {
        return FALSE;
    }
    if (*LunExt).DeviceFlags & DFLAGS_HIDDEN != 0 {
        return FALSE;
    }

    if use_ahci != 0 {
        statusByte = WaitOnBusyLong(chan);
    } else {
        SelectDrive(chan, DeviceNumber);
        ScsiPortStallExecution(10);
        statusByte = WaitOnBusyLong(chan);
        // Check that the status register makes sense.
        statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
    }

    if Command as u32 == IDE_COMMAND_IDENTIFY {
        // Mask status byte ERROR bits.
        statusByte = UniataIsIdle(
            deviceExtension,
            statusByte & !((IDE_STATUS_ERROR | IDE_STATUS_INDEX) as UCHAR),
        );
        // Check if register value is reasonable.
        if statusByte as u32 != IDE_STATUS_IDLE {
            // No reset here !!!
            if UniataIsSATARangeAvailable(deviceExtension, lChannel) == 0 {
                SelectDrive(chan, DeviceNumber);
                WaitOnBusyLong(chan);

                signatureLow = AtapiReadPort1(chan, IDX_IO1_i_CylinderLow);
                signatureHigh = AtapiReadPort1(chan, IDX_IO1_i_CylinderHigh);

                if signatureLow as u32 == ATAPI_MAGIC_LSB && signatureHigh as u32 == ATAPI_MAGIC_MSB {
                    // Device is Atapi.
                    return FALSE;
                }

                // We really should wait up to 31 seconds.
                loop {
                    // Wait for Busy to drop.
                    ScsiPortStallExecution(100);
                    statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus); // GetStatus
                    if statusByte as u32 == IDE_STATUS_WRONG {
                        return FALSE;
                    }
                    let busy = (statusByte as u32 & IDE_STATUS_BUSY) != 0;
                    let wc = waitCount;
                    waitCount = waitCount.wrapping_sub(1);
                    if !(busy && wc != 0) {
                        break;
                    }
                }
                statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus

                SelectDrive(chan, DeviceNumber);
            } else {
                statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
            }
            // Another check for signature.
            signatureLow = AtapiReadPort1(chan, IDX_IO1_i_CylinderLow);
            signatureHigh = AtapiReadPort1(chan, IDX_IO1_i_CylinderHigh);

            if signatureLow as u32 == ATAPI_MAGIC_LSB && signatureHigh as u32 == ATAPI_MAGIC_MSB {
                // Device is Atapi.
                return FALSE;
            }

            statusByte = UniataIsIdle(deviceExtension, statusByte) & !(IDE_STATUS_INDEX as UCHAR);
            if statusByte as u32 != IDE_STATUS_IDLE {
                // Give up on this.
                return FALSE;
            }
        }
    } else {
        if statusByte as u32 == IDE_STATUS_WRONG {
            return FALSE;
        }
        if UniataIsSATARangeAvailable(deviceExtension, lChannel) == 0 {
            statusByte = WaitForIdleLong(chan);
        }
        atapiDev = TRUE;
    }

    if use_ahci != 0 {
        statusByte = UniataAhciSendPIOCommand(
            HwDeviceExtension,
            lChannel,
            DeviceNumber,
            core::ptr::null_mut::<SCSI_REQUEST_BLOCK>() as PSCSI_REQUEST_BLOCK,
            (&mut (*deviceExtension).FullIdentifyData) as *mut _ as PUCHAR,
            DEV_BSIZE as ULONG,
            Command,
            0,
            0,
            0,
            0, // ahci flags
            ATA_WAIT_INTR as ULONG,
            1000, // timeout 1 sec
        );
        j = 9; // AHCI is rather different, skip loop at all
    } else if (*LunExt).DeviceFlags & DFLAGS_MANUAL_CHS != 0 {
        j = 9; // don't send IDENTIFY, assume it is not supported
        RtlZeroMemory(
            (&mut (*deviceExtension).FullIdentifyData) as *mut _ as PVOID,
            core::mem::size_of_val(&(*deviceExtension).FullIdentifyData),
        );
        RtlCopyMemory(
            (&mut (*deviceExtension).FullIdentifyData) as *mut _ as PVOID,
            (&(*LunExt).IdentifyData) as *const _ as PCVOID,
            core::mem::size_of_val(&(*LunExt).IdentifyData),
        );
    } else if UniataIsSATARangeAvailable(deviceExtension, lChannel) != 0 {
        j = 4; // skip old-style checks
    } else {
        j = 0;
    }

    while j < 4 * 2 {
        // Send IDENTIFY command.
        statusByte = AtaCommand(
            deviceExtension,
            DeviceNumber,
            lChannel,
            Command,
            if j < 4 { DEV_BSIZE as USHORT } else { 0 }, // cyl
            0,
            0,
            0,
            0,
            ATA_WAIT_INTR as ULONG,
        );

        if statusByte == 0 {
            return FALSE;
        } else if statusByte as u32 & IDE_STATUS_DRQ != 0 {
            // Read status to acknowledge any interrupts generated.
            statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
            // One last check for Atapi.
            if Command as u32 == IDE_COMMAND_IDENTIFY {
                signatureLow = AtapiReadPort1(chan, IDX_IO1_i_CylinderLow);
                signatureHigh = AtapiReadPort1(chan, IDX_IO1_i_CylinderHigh);
                if signatureLow as u32 == ATAPI_MAGIC_LSB && signatureHigh as u32 == ATAPI_MAGIC_MSB {
                    // Device is Atapi.
                    return FALSE;
                }
            }
            break;
        } else {
            if Command as u32 == IDE_COMMAND_IDENTIFY {
                // Check the signature. If DRQ didn't come up it's likely Atapi.
                signatureLow = AtapiReadPort1(chan, IDX_IO1_i_CylinderLow);
                signatureHigh = AtapiReadPort1(chan, IDX_IO1_i_CylinderHigh);
                if signatureLow as u32 == ATAPI_MAGIC_LSB && signatureHigh as u32 == ATAPI_MAGIC_MSB {
                    // Device is Atapi.
                    return FALSE;
                }
            } else {
                if (statusByte as u32 & IDE_STATUS_ERROR) == 0 && (statusByte as u32 & IDE_STATUS_BUSY) != 0 {
                    break;
                }
            }
            // Device didn't respond correctly. It will be given one more chance.
            statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
            AtapiSoftReset(chan, DeviceNumber);

            AtapiDisableInterrupts(deviceExtension as PVOID, lChannel);
            AtapiEnableInterrupts(deviceExtension as PVOID, lChannel);

            statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
        }
        j += 1;
    }
    // Check for error on really stupid master devices.
    if (Command as u32 == IDE_COMMAND_IDENTIFY) && (statusByte as u32 & IDE_STATUS_ERROR) != 0 {
        return FALSE;
    }

    if use_ahci != 0 {
        // everything should already be done by controller
    } else if (*LunExt).DeviceFlags & DFLAGS_MANUAL_CHS != 0 {
        j = 9; // don't send IDENTIFY, assume it is not supported
        statusByte = WaitForDrq(chan);
        statusByte = WaitOnBusyLong(chan);
        statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
    } else {
        // Suck out 256 words.
        statusByte = WaitForDrq(chan);
        statusByte = WaitOnBusyLong(chan);

        if (statusByte as u32 & IDE_STATUS_DRQ) == 0 {
            statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
            return FALSE;
        }
        statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus

        if atapiDev != 0 || ((*LunExt).DeviceFlags & DFLAGS_DWORDIO_ENABLED) == 0 {
            // use 16bit IO
            AtapiReadBuffer2(
                chan,
                IDX_IO1_i_Data,
                (&mut (*deviceExtension).FullIdentifyData) as *mut _ as PVOID,
                256,
                PIO0_TIMING,
            );
            // Work around for some IDE/Atapi that present more than 256 bytes.
            statusByte = AtapiSuckPort2(chan);
        } else {
            // use 32bit IO
            AtapiReadBuffer4(
                chan,
                IDX_IO1_i_Data,
                (&mut (*deviceExtension).FullIdentifyData) as *mut _ as PVOID,
                256 / 2,
                PIO0_TIMING,
            );
        }

        statusByte = WaitForDrq(chan);
        statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
    }

    // Neither check existed in UniATA; both are upstream's (dev_identify.c).
    // A drive in PUIS answers IDENTIFY with placeholder data, and the C then
    // enumerated it from that. Spin it up and identify once more (only once:
    // a drive that still says PUIS keeps the old behaviour rather than loop).
    let fidp = &(*deviceExtension).FullIdentifyData as *const IDENTIFY_DATA;
    if allow_spinup && (*LunExt).DeviceFlags & DFLAGS_MANUAL_CHS == 0 && identify_needs_spinup(fidp) {
        DbgPrint(b"[UNIATA] IssueIdentify ch=%u dev=%u: PUIS/incomplete IDENTIFY, spinning up\n\0".as_ptr().cast(), lChannel, DeviceNumber);
        spin_up_puis_drive(deviceExtension, chan, DeviceNumber, lChannel, use_ahci != 0);
        return issue_identify_once(HwDeviceExtension, DeviceNumber, lChannel, Command, NoSetup, false);
    }
    if !identify_checksum_ok(fidp) {
        DbgPrint(b"[UNIATA] IssueIdentify ch=%u dev=%u: IDENTIFY checksum error\n\0".as_ptr().cast(), lChannel, DeviceNumber);
        return FALSE;
    }

    if NoSetup != 0 {
        return TRUE;
    }

    let fid = &mut (*deviceExtension).FullIdentifyData;

    if g_opt_VirtualMachine == VM_AUTO {
        if (fid.FirmwareRevision[0] == 0 || fid.FirmwareRevision[0] == (b' ' as USHORT))
            && (fid.FirmwareRevision[1] == 0 || fid.FirmwareRevision[1] == (b' ' as USHORT))
        {
            // Check for BOCHS VM signature.
            if atapiDev == 0
                && _strnicmp(
                    fid.SerialNumber.as_ptr() as PCCHAR,
                    b"XBDH00\0".as_ptr() as PCCHAR,
                    6,
                ) == 0
            {
                g_opt_VirtualMachine = VM_BOCHS;
            } else if atapiDev != 0
                && _strnicmp(
                    fid.SerialNumber.as_ptr() as PCCHAR,
                    b"XBDC00\0".as_ptr() as PCCHAR,
                    6,
                ) == 0
            {
                g_opt_VirtualMachine = VM_BOCHS;
            }
        }
    }

    (*LunExt).OrigTransferMode =
        ata_cur_mode_from_ident(fid as *mut _, crate::compat::IDENT_MODE_MAX as BOOLEAN) as UCHAR;
    (*LunExt).LimitedTransferMode = (*LunExt).OrigTransferMode;
    (*LunExt).TransferMode =
        ata_cur_mode_from_ident(fid as *mut _, crate::compat::IDENT_MODE_ACTIVE as BOOLEAN) as UCHAR;

    // Check out a few capabilities / limitations of the device.
    if fid.RemovableStatus & 1 != 0 {
        (*LunExt).DeviceFlags |= DFLAGS_REMOVABLE_DRIVE;
    }
    if use_ahci != 0 {
        // AHCI doesn't recommend using PIO and multiblock
        (*LunExt).MaximumBlockXfer = 0;
    } else if fid.MaximumBlockTransfer != 0 {
        (*LunExt).MaximumBlockXfer = fid.MaximumBlockTransfer & 0xFF;
    }
    (*LunExt).NumOfSectors = 0;
    if Command as u32 == IDE_COMMAND_IDENTIFY {
        let mut NumOfSectors: ULONGLONG;
        let mut NativeNumOfSectors: ULONGLONG;
        let mut cylinders: ULONGLONG = 0;
        let mut tmp_cylinders: ULONGLONG;

        NumOfSectors = (fid.NumberOfCylinders as ULONGLONG)
            * (fid.NumberOfHeads as ULONGLONG)
            * (fid.SectorsPerTrack as ULONGLONG);
        // Check for HDDs > 8Gb
        if fid.NumberOfCylinders == 0x3fff
            && fid.NumberOfHeads != 0
            && fid.SectorsPerTrack != 0
            && NumOfSectors < fid.UserAddressableSectors as ULONGLONG
        {
            cylinders = (fid.UserAddressableSectors as ULONGLONG)
                / ((fid.NumberOfHeads as ULONGLONG) * (fid.SectorsPerTrack as ULONGLONG));
            NumOfSectors =
                cylinders * (fid.NumberOfHeads as ULONGLONG) * (fid.SectorsPerTrack as ULONGLONG);
        }
        // Check for LBA mode
        if fid.SupportLba() != 0
            || (fid.MajorRevision != 0 && fid.UserAddressableSectors != 0)
        {
            (*LunExt).DeviceFlags |= DFLAGS_LBA_ENABLED;
        } else {
            (*LunExt).DeviceFlags |= DFLAGS_ORIG_GEOMETRY;
            // goto skip_lba_staff;
            return issue_identify_finish(
                deviceExtension, LunExt, chan, Command, atapiDev,
                NumOfSectors,
            );
        }
        // Check for LBA48 support
        if (*LunExt).DeviceFlags & DFLAGS_LBA_ENABLED != 0 {
            if fid.FeaturesSupport.Address48() != 0
                && fid.FeaturesEnabled.Address48() != 0
                && fid.NumberOfHeads != 0
                && fid.SectorsPerTrack != 0
                && fid.UserAddressableSectors48 > NumOfSectors
            {
                cylinders = fid.UserAddressableSectors48
                    / ((fid.NumberOfHeads as ULONGLONG) * (fid.SectorsPerTrack as ULONGLONG));
                NativeNumOfSectors = cylinders
                    * (fid.NumberOfHeads as ULONGLONG)
                    * (fid.SectorsPerTrack as ULONGLONG);
                if NativeNumOfSectors > NumOfSectors {
                    NumOfSectors = NativeNumOfSectors;
                }
            }

            // Check drive capacity report for LBA48-capable drives.
            if fid.FeaturesSupport.Address48() != 0 {
                let hNativeNumOfSectors: ULONG;
                statusByte = AtaCommand48(
                    deviceExtension, DeviceNumber, lChannel,
                    IDE_COMMAND_READ_NATIVE_SIZE48 as UCHAR, 0, 0, 0, ATA_WAIT_READY as ULONG,
                );
                if (statusByte as u32 & IDE_STATUS_ERROR) == 0 {
                    if use_ahci != 0 {
                        NativeNumOfSectors = (*ata_req_ahci((*chan).AhciInternalAtaReq)).in_lba;
                    } else {
                        NativeNumOfSectors = (AtapiReadPort1(chan, IDX_IO1_i_BlockNumber) as ULONG
                            | ((AtapiReadPort1(chan, IDX_IO1_i_CylinderLow) as ULONG) << 8)
                            | ((AtapiReadPort1(chan, IDX_IO1_i_CylinderHigh) as ULONG) << 16))
                            as ULONGLONG;

                        AtapiWritePort1(chan, IDX_IO2_o_Control, IDE_DC_USE_HOB as UCHAR);

                        NativeNumOfSectors |=
                            ((AtapiReadPort1(chan, IDX_IO1_i_BlockNumber) as ULONG) << 24) as ULONGLONG;
                        hNativeNumOfSectors = AtapiReadPort1(chan, IDX_IO1_i_CylinderLow) as ULONG
                            | ((AtapiReadPort1(chan, IDX_IO1_i_CylinderHigh) as ULONG) << 8);
                        // ((PULONG)&NativeNumOfSectors)[1] = hNativeNumOfSectors;
                        let p = (&mut NativeNumOfSectors as *mut ULONGLONG) as *mut ULONG;
                        *p.add(1) = hNativeNumOfSectors;
                    }

                    // see comment in C: high==low part probe
                    if (NativeNumOfSectors & 0xffffff) == ((NativeNumOfSectors >> 24) & 0xffffff) {
                        statusByte = AtaCommand48(
                            deviceExtension, DeviceNumber, lChannel,
                            IDE_COMMAND_READ_NATIVE_SIZE48 as UCHAR, 0, 0, 0, ATA_WAIT_READY as ULONG,
                        );
                        if (statusByte as u32 & IDE_STATUS_ERROR) == 0 {
                            if use_ahci != 0 {
                                NativeNumOfSectors = (*ata_req_ahci((*chan).AhciInternalAtaReq)).in_lba;
                            } else {
                                NativeNumOfSectors = (AtapiReadPort1(chan, IDX_IO1_i_BlockNumber) as ULONGLONG)
                                    | ((AtapiReadPort1(chan, IDX_IO1_i_BlockNumber) as ULONGLONG) << 24)
                                    | ((AtapiReadPort1(chan, IDX_IO1_i_CylinderLow) as ULONGLONG) << 8)
                                    | ((AtapiReadPort1(chan, IDX_IO1_i_CylinderLow) as ULONGLONG) << 32)
                                    | ((AtapiReadPort1(chan, IDX_IO1_i_CylinderHigh) as ULONGLONG) << 16)
                                    | ((AtapiReadPort1(chan, IDX_IO1_i_CylinderHigh) as ULONGLONG) << 40);
                            }
                        }
                        if (NativeNumOfSectors & 0xffffff) == ((NativeNumOfSectors >> 24) & 0xffffff) {
                            NativeNumOfSectors = 0;
                        }
                    }

                    if NumOfSectors <= ATA_MAX_LBA28 && NativeNumOfSectors > NumOfSectors {
                        statusByte = AtaCommand48(
                            deviceExtension, DeviceNumber, lChannel,
                            IDE_COMMAND_SET_NATIVE_SIZE as UCHAR, NativeNumOfSectors, 0, 0,
                            ATA_WAIT_READY as ULONG,
                        );
                        if (statusByte as u32 & IDE_STATUS_ERROR) == 0 {
                            NumOfSectors = NativeNumOfSectors;
                        }
                    }
                } // !error
            }

            if NumOfSectors < 0x2100000 {
                // check for native LBA size
                statusByte = AtaCommand(
                    deviceExtension, DeviceNumber, lChannel,
                    IDE_COMMAND_READ_NATIVE_SIZE as UCHAR, 0, IDE_USE_LBA as UCHAR, 0, 0, 0,
                    ATA_WAIT_READY as ULONG,
                );
                if (statusByte as u32 & IDE_STATUS_ERROR) == 0 {
                    if use_ahci != 0 {
                        NativeNumOfSectors = (*ata_req_ahci((*chan).AhciInternalAtaReq)).in_lba;
                    } else {
                        NativeNumOfSectors = (AtapiReadPort1(chan, IDX_IO1_i_BlockNumber) as ULONG
                            | ((AtapiReadPort1(chan, IDX_IO1_i_CylinderLow) as ULONG) << 8)
                            | ((AtapiReadPort1(chan, IDX_IO1_i_CylinderHigh) as ULONG) << 16)
                            | (((AtapiReadPort1(chan, IDX_IO1_i_DriveSelect) as ULONG) & 0xf) << 24))
                            as ULONGLONG;
                    }
                    if NativeNumOfSectors > NumOfSectors {
                        statusByte = AtaCommand48(
                            deviceExtension, DeviceNumber, lChannel,
                            IDE_COMMAND_SET_NATIVE_SIZE as UCHAR, NativeNumOfSectors, 0, 0,
                            ATA_WAIT_READY as ULONG,
                        );
                        if (statusByte as u32 & IDE_STATUS_ERROR) == 0 {
                            NumOfSectors = NativeNumOfSectors;
                        }
                    }
                }
            }

            if NumOfSectors > ATA_MAX_IOLBA28 {
                (*LunExt).DeviceFlags |= DFLAGS_LBA32plus;
            }
        } // if(LunExt->DeviceFlags & DFLAGS_LBA_ENABLED)

        // fill IdentifyData with bogus geometry
        if fid.CurrentSectorsPerTrack != 0 && fid.NumberOfCurrentHeads != 0 {
            tmp_cylinders = NumOfSectors
                / ((fid.CurrentSectorsPerTrack as ULONGLONG)
                    * (fid.NumberOfCurrentHeads as ULONGLONG));
        } else if fid.SectorsPerTrack != 0 && fid.NumberOfHeads != 0 {
            tmp_cylinders = NumOfSectors
                / ((fid.SectorsPerTrack as ULONGLONG) * (fid.NumberOfHeads as ULONGLONG));
        } else {
            tmp_cylinders = 0;
        }
        if tmp_cylinders < 0xffff || (*LunExt).opt_GeomType == GEOM_ORIG {
            // ok, we can keep original values
            if (*LunExt).opt_GeomType == GEOM_AUTO {
                (*LunExt).opt_GeomType = GEOM_ORIG;
            }
        } else {
            tmp_cylinders = NumOfSectors / (255 * 63);
            if tmp_cylinders < 0xffff {
                if (*LunExt).opt_GeomType == GEOM_AUTO {
                    (*LunExt).opt_GeomType = GEOM_STD;
                }
            } else {
                if (*LunExt).opt_GeomType == GEOM_AUTO {
                    (*LunExt).opt_GeomType = GEOM_UNIATA;
                }
            }
        }

        if (*LunExt).opt_GeomType == GEOM_STD {
            fid.SectorsPerTrack = 63;
            fid.CurrentSectorsPerTrack = 63;
            fid.NumberOfHeads = 255;
            fid.NumberOfCurrentHeads = 255;
            cylinders = NumOfSectors / (255 * 63);
        } else if (*LunExt).opt_GeomType == GEOM_UNIATA {
            while cylinders > 0xffff && fid.SectorsPerTrack < 0x80 {
                cylinders /= 2;
                fid.SectorsPerTrack = fid.SectorsPerTrack.wrapping_mul(2);
                fid.CurrentSectorsPerTrack = fid.CurrentSectorsPerTrack.wrapping_mul(2);
            }
            while cylinders > 0xffff && fid.NumberOfHeads < 0x80 {
                cylinders /= 2;
                fid.NumberOfHeads = fid.NumberOfHeads.wrapping_mul(2);
                fid.NumberOfCurrentHeads = fid.NumberOfCurrentHeads.wrapping_mul(2);
            }
            while cylinders > 0xffff && fid.SectorsPerTrack < 0x8000 {
                cylinders /= 2;
                fid.SectorsPerTrack = fid.SectorsPerTrack.wrapping_mul(2);
                fid.CurrentSectorsPerTrack = fid.CurrentSectorsPerTrack.wrapping_mul(2);
            }
            while cylinders > 0xffff && fid.NumberOfHeads < 0x8000 {
                cylinders /= 2;
                fid.NumberOfHeads = fid.NumberOfHeads.wrapping_mul(2);
                fid.NumberOfCurrentHeads = fid.NumberOfCurrentHeads.wrapping_mul(2);
            }
        }
        if cylinders == 0 {
            cylinders = tmp_cylinders;
        }
        fid.NumberOfCylinders = cylinders as USHORT;
        fid.NumberOfCurrentCylinders = cylinders as USHORT;

        // skip_lba_staff:
        if NumOfSectors != 0 {
            (*LunExt).NumOfSectors = NumOfSectors;
        }
        if fid.MajorRevision != 0 && fid.DoubleWordIo != 0 {
            (*LunExt).DeviceFlags |= DFLAGS_DWORDIO_ENABLED;
        }
    } else {
        // ATAPI
        if fid.MajorRevision != 0 && fid.DoubleWordIo != 0 {
            (*LunExt).DeviceFlags |= DFLAGS_DWORDIO_ENABLED;
        }
        // (DMADirRequired / AtapiByteCount0 are debug-only reads — dropped)
    }

    issue_identify_finish(deviceExtension, LunExt, chan, Command, atapiDev, 0)
}

// Tail shared by IssueIdentify (post-`skip_lba_staff` path). NumOfSectors is unused
// past skip_lba_staff (it was applied above), so the goto target only needs the
// remaining trailer logic that runs identically in both reach paths.
unsafe fn issue_identify_finish(
    deviceExtension: PHW_DEVICE_EXTENSION,
    LunExt: PHW_LU_EXTENSION,
    chan: PHW_CHANNEL,
    Command: UCHAR,
    _atapiDev: BOOLEAN,
    _NumOfSectors: ULONGLONG,
) -> BOOLEAN {
    let mut statusByte: UCHAR;

    ScsiPortMoveMemory(
        (&mut (*LunExt).IdentifyData) as *mut _ as PVOID,
        (&mut (*deviceExtension).FullIdentifyData) as *mut _ as PVOID,
        core::mem::size_of_val(&(*deviceExtension).FullIdentifyData) as ULONG,
    );

    InitBadBlocks(LunExt);

    if ((*LunExt).IdentifyData.DrqType() as u32 & ATAPI_DRQT_INTR) != 0
        && (Command as u32 != IDE_COMMAND_IDENTIFY)
    {
        // This device interrupts with the assertion of DRQ after Atapi Packet Command
        (*LunExt).DeviceFlags |= DFLAGS_INT_DRQ;
    }

    if Command as u32 != IDE_COMMAND_IDENTIFY {
        // ATAPI branch
        if (*LunExt).IdentifyData.DeviceType() as u32 == ATAPI_TYPE_TAPE {
            (*LunExt).DeviceFlags |= DFLAGS_TAPE_DEVICE;
        } else if (*LunExt).IdentifyData.DeviceType() as u32 == ATAPI_TYPE_CDROM
            || (*LunExt).IdentifyData.DeviceType() as u32 == ATAPI_TYPE_OPTICAL
        {
            // set CD default costs
            (*LunExt).RwSwitchCost = REORDER_COST_SWITCH_RW_CD;
            (*LunExt).RwSwitchMCost = REORDER_MCOST_SWITCH_RW_CD as LONGLONG;
            (*LunExt).SeekBackMCost = REORDER_MCOST_SEEK_BACK_CD as LONGLONG;
            statusByte = WaitForDrq(chan);
        }
    }

    statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
    let _ = statusByte;
    DbgPrint(b"[UNIATA] IssueIdentify done\n\0".as_ptr().cast()); // perf phase marker
    TRUE
}

// ============================================================================
// SetDriveParameters / UniataForgetDevice
// ============================================================================

pub unsafe extern "system" fn SetDriveParameters(
    HwDeviceExtension: PVOID,
    DeviceNumber: ULONG,
    lChannel: ULONG,
) -> BOOLEAN {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let LunExt: PHW_LU_EXTENSION = (*(*deviceExtension).chan.add(lChannel as usize)).lun[DeviceNumber as usize];
    let identifyData = &mut (*LunExt).IdentifyData;
    let mut statusByte: UCHAR;
    let _errorByte: UCHAR;

    if (*LunExt).DeviceFlags & (DFLAGS_LBA_ENABLED | DFLAGS_ORIG_GEOMETRY) != 0 {
        return TRUE;
    }

    // Send SET PARAMETER command.
    statusByte = AtaCommand(
        deviceExtension, DeviceNumber, lChannel,
        IDE_COMMAND_SET_DRIVE_PARAMETERS as UCHAR, 0,
        (identifyData.NumberOfHeads.wrapping_sub(1)) as UCHAR, 0,
        identifyData.SectorsPerTrack as UCHAR, 0, ATA_WAIT_IDLE as ULONG,
    );

    statusByte = UniataIsIdle(deviceExtension, statusByte);
    if statusByte as u32 & IDE_STATUS_ERROR != 0 {
        let _errorByte = AtapiReadPort1((*deviceExtension).chan.add(lChannel as usize), IDX_IO1_i_Error);
        return FALSE;
    }

    if statusByte as u32 == IDE_STATUS_IDLE {
        return TRUE;
    }

    FALSE
}

pub unsafe extern "system" fn UniataForgetDevice(LunExt: PHW_LU_EXTENSION) {
    // keep only DFLAGS_HIDDEN flag
    (*LunExt).DeviceFlags &= DFLAGS_HIDDEN;
    (*LunExt).AtapiReadyWaitDelay = 0;
}

// ============================================================================
// AtapiResetController / AtapiResetController__  (->HwResetBus)
// ============================================================================

pub unsafe extern "system" fn AtapiResetController(
    HwDeviceExtension: PVOID,
    PathId: ULONG,
) -> BOOLEAN {
    AtapiResetController__(HwDeviceExtension, PathId, RESET_COMPLETE_ALL)
}

pub unsafe extern "system" fn AtapiResetController__(
    HwDeviceExtension: PVOID,
    PathId: ULONG,
    CompleteType: BOOLEAN,
) -> BOOLEAN {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let mut numberChannels: ULONG;
    let mut chan: PHW_CHANNEL = core::ptr::null_mut();
    let mut i: ULONG;
    let mut j: ULONG;
    let mut MaxLuns: ULONG;
    let mut statusByte: UCHAR;
    let mut ChannelCtrlFlags: ULONG;
    let mut dma_status: UCHAR = 0;

    let slotNumber: ULONG = (*deviceExtension).slotNumber;
    let SystemIoBusNumber: ULONG = (*deviceExtension).SystemIoBusNumber;
    let VendorID: ULONG = (*deviceExtension).DevID & 0xffff;
    let ChipFlags: ULONG = (*deviceExtension).HwFlags & CHIPFLAG_MASK;
    let mut tmp16: USHORT = 0;

    if (*deviceExtension).simplexOnly == 0 && PathId != CHAN_NOT_SPECIFIED {
        // we shall reset both channels on SimplexOnly devices
        j = PathId;
        numberChannels = core::cmp::min(j + 1, (*deviceExtension).NumberChannels);
    } else {
        j = 0;
        numberChannels = (*deviceExtension).NumberChannels;
    }

    while j < numberChannels {
        chan = (*deviceExtension).chan.add(j as usize);
        MaxLuns = (*chan).NumberLuns;
        // Save control flags
        ChannelCtrlFlags = (*chan).ChannelCtrlFlags;

        if CompleteType != RESET_COMPLETE_NONE {
            while let Some(mut CurSrb) = {
                let p = UniataGetCurRequest(chan);
                if p.is_null() { None } else { Some(p) }
            } {
                let AtaReq = (*CurSrb).SrbExtension as PATA_REQ;

                i = (*CurSrb).TargetId as ULONG; // GET_CDEV
                let LunExt: PHW_LU_EXTENSION = (*chan).lun[i as usize];

                if (*CurSrb).Cdb[0] as u32 == SCSIOP_MECHANISM_STATUS {
                    if (*LunExt).DeviceFlags & DFLAGS_CHANGER_INITED == 0 {
                        (*LunExt).DeviceFlags |= DFLAGS_CHANGER_INITED;
                    }
                }
                // Check and see if we are processing an internal srb
                if !(*ata_req_core(AtaReq)).OriginalSrb.is_null() {
                    (*ata_req_core(AtaReq)).Srb = (*ata_req_core(AtaReq)).OriginalSrb;
                    (*CurSrb).SrbExtension = core::ptr::null_mut();
                    (*ata_req_core(AtaReq)).OriginalSrb = core::ptr::null_mut();
                    // NOTE: internal SRB doesn't get to SRB queue !!!
                    CurSrb = (*ata_req_core(AtaReq)).Srb;
                }

                // Remove current request from queue
                UniataRemoveRequest(chan, CurSrb);

                if !CurSrb.is_null() {
                    // Complete outstanding request with SRB_STATUS_BUS_RESET.
                    let CurPathId = (*CurSrb).PathId;
                    let TargetId = (*CurSrb).TargetId;
                    let Lun = (*CurSrb).Lun;

                    (*CurSrb).SrbStatus = ((if CompleteType == RESET_COMPLETE_ALL {
                        SRB_STATUS_BUS_RESET
                    } else {
                        SRB_STATUS_ABORTED
                    }) | SRB_STATUS_AUTOSENSE_VALID) as UCHAR;
                    (*CurSrb).ScsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;

                    if !(*CurSrb).SenseInfoBuffer.is_null() {
                        let senseBuffer = (*CurSrb).SenseInfoBuffer as PSENSE_DATA;
                        (*senseBuffer).set_ErrorCode(0x70);
                        (*senseBuffer).set_Valid(1);
                        (*senseBuffer).AdditionalSenseLength = 0xb;
                        if CompleteType == RESET_COMPLETE_ALL {
                            (*senseBuffer).set_SenseKey(SCSI_SENSE_UNIT_ATTENTION as UCHAR);
                            (*senseBuffer).AdditionalSenseCode = SCSI_ADSENSE_BUS_RESET as UCHAR;
                            (*senseBuffer).AdditionalSenseCodeQualifier = SCSI_SENSEQ_SCSI_BUS as UCHAR;
                        } else {
                            (*senseBuffer).set_SenseKey(SCSI_SENSE_ABORTED_COMMAND as UCHAR);
                            (*senseBuffer).AdditionalSenseCode = 0;
                            (*senseBuffer).AdditionalSenseCodeQualifier = 0;
                        }
                    }

                    let atapi_dev = ((*(*chan).lun[i as usize]).DeviceFlags & DFLAGS_ATAPI_DEVICE) != 0;
                    if !atapi_dev
                        && (*ata_req_core(AtaReq)).bcount != 0
                        && ((*ata_req_core(AtaReq)).retry as u32) < MAX_RETRIES
                    {
                        (*LunExt).errLastLba = (*ata_req_core(AtaReq)).lba as ULONGLONG;
                        (*LunExt).errBCount = (*ata_req_core(AtaReq)).bcount;
                        (*LunExt).errRetry = (*ata_req_core(AtaReq)).retry + 1;
                        if ((*ata_req_core(AtaReq)).Flags as u32 & REQ_FLAG_RW_MASK) == REQ_FLAG_READ
                            && (ChannelCtrlFlags & CTRFLAGS_DMA_OPERATION) != 0
                            && g_opt_VirtualMachine == VM_BOCHS
                        {
                            g_opt_BochsDmaReadWorkaround = TRUE;
                            g_opt_AtapiNoDma = TRUE;
                        }
                    } else if atapi_dev
                        && (*ata_req_core(AtaReq)).bcount != 0
                        && (*ata_req_core(AtaReq)).retry == 0
                    {
                        (*LunExt).errLastLba = (*ata_req_core(AtaReq)).lba as ULONGLONG;
                        (*LunExt).errBCount = (*ata_req_core(AtaReq)).bcount;
                        (*LunExt).errRetry = (*ata_req_core(AtaReq)).retry + 1;
                        if ((*ata_req_core(AtaReq)).Flags as u32 & REQ_FLAG_RW_MASK) == REQ_FLAG_READ
                            && (ChannelCtrlFlags & CTRFLAGS_DMA_OPERATION) != 0
                            && g_opt_VirtualMachine == VM_BOCHS
                        {
                            g_opt_AtapiNoDma = TRUE;
                        }
                    } else {
                        (*LunExt).errRetry = 0;
                    }

                    // Clear request tracking fields.
                    (*ata_req_core(AtaReq)).WordsLeft = 0;
                    (*ata_req_core(AtaReq)).DataBuffer = core::ptr::null_mut();
                    (*ata_req_core(AtaReq)).TransferLength = 0;

                    ScsiPortNotification(
                        _SCSI_NOTIFICATION_TYPE_RequestComplete as SCSI_NOTIFICATION_TYPE,
                        deviceExtension as PVOID, CurSrb,
                    );

                    // Indicate ready for next request.
                    ScsiPortNotification(
                        _SCSI_NOTIFICATION_TYPE_NextLuRequest as SCSI_NOTIFICATION_TYPE,
                        deviceExtension as PVOID, CurPathId as c_uint, TargetId as c_uint, Lun as c_uint,
                    );
                }
                if CompleteType != RESET_COMPLETE_ALL {
                    break;
                }
            } // end while()
        } // end if (CompleteType != RESET_COMPLETE_NONE)

        // Clear expecting interrupt flag.
        UniataExpectChannelInterrupt(chan, FALSE);
        (*chan).RDP = FALSE;
        (*chan).ChannelCtrlFlags = ChannelCtrlFlags & CTRFLAGS_PERMANENT;
        InterlockedExchange(&mut (*chan).CheckIntr, CHECK_INTR_IDLE as LONG);

        i = 0;
        while i < MaxLuns {
            (*(*chan).lun[i as usize]).PowerState = 0;
            i += 1;
        }
        // Reset controller
        if ChipFlags & UNIATA_AHCI != 0 {
            if UniataAhciChanImplemented(deviceExtension, j) != 0 {
                AtapiDisableInterrupts(deviceExtension as PVOID, j);
                UniataAhciReset(HwDeviceExtension, j);
            } else {
                j += 1;
                continue;
            }
        } else {
            AtapiDisableInterrupts(deviceExtension as PVOID, j);

            // Vendor reset switch. C uses `goto default_reset` from several arms and the
            // `default:` arm FALLS THROUGH into default_reset; the other arms `break;` and
            // SKIP it. Modeled with a `do_default_reset` flag set by the goto/default paths.
            let mut do_default_reset = false;
            'reset: {
                match VendorID {
                    x if x == ATA_INTEL_ID => {
                        let mut mask: ULONG;
                        let pshift: ULONG;
                        let mut timeout: ULONG;
                        if ChipFlags & UNIATA_SATA == 0 {
                            do_default_reset = true; break 'reset;
                        }
                        if UniataIsSATARangeAvailable(deviceExtension, j) == 0 {
                            do_default_reset = true; break 'reset;
                        }
                        mask = 1 << *(*(*chan).lun[0]).__bindgen_anon_1.SATA_lun_map.as_ref();
                        if MaxLuns > 1 {
                            mask |= 1 << *(*(*chan).lun[1]).__bindgen_anon_1.SATA_lun_map.as_ref();
                        }
                        // ChangePciConfig2(0x92, a & ~mask)
                        {
                            let mut a: USHORT = 0;
                            ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration as i32, SystemIoBusNumber, slotNumber, &mut a as *mut _ as PVOID, 0x92, 2);
                            a = (a as ULONG & !mask) as USHORT;
                            ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration, SystemIoBusNumber, slotNumber, &mut a as *mut _ as PVOID, 0x92, 2);
                        }
                        ScsiPortStallExecution(10);
                        {
                            let mut a: USHORT = 0;
                            ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration as i32, SystemIoBusNumber, slotNumber, &mut a as *mut _ as PVOID, 0x92, 2);
                            a = (a as ULONG | mask) as USHORT;
                            ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration, SystemIoBusNumber, slotNumber, &mut a as *mut _ as PVOID, 0x92, 2);
                        }
                        timeout = 100;
                        if ChipFlags & (I6CH | I6CH2) != 0 {
                            pshift = 8;
                        } else {
                            pshift = 4;
                        }
                        while timeout != 0 {
                            timeout -= 1;
                            ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration as i32, SystemIoBusNumber, slotNumber, &mut tmp16 as *mut _ as PVOID, 0x92, 2);
                            if (((tmp16 as ULONG) >> pshift) & mask) == mask {
                                statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
                                if statusByte as u32 != IDE_STATUS_WRONG {
                                    break;
                                }
                            }
                            ScsiPortStallExecution(10000);
                        }
                    }
                    x if x == ATA_SIS_ID => {
                        if ChipFlags & UNIATA_SATA == 0 {
                            do_default_reset = true; break 'reset;
                        }
                    }
                    x if x == ATA_NVIDIA_ID => {
                        let offs: ULONG;
                        let Channel: ULONG = (*deviceExtension).Channel + j;
                        if ChipFlags & UNIATA_SATA == 0 {
                            do_default_reset = true; break 'reset;
                        }
                        offs = if ChipFlags & NV4OFF != 0 { 0x0440 } else { 0x0010 };

                        // disable device and PHY state change interrupts
                        if ChipFlags & NVQ != 0 {
                            AtapiWritePortEx4(core::ptr::null_mut(), (&(*deviceExtension).BaseIoAddressSATA_0) as *const _ as ULONG_PTR, offs + 4,
                                AtapiReadPortEx4(core::ptr::null_mut(), (&(*deviceExtension).BaseIoAddressSATA_0) as *const _ as ULONG_PTR, offs + 4) & ((!(0x0000000d as ULONG)) << ((Channel == 0) as u32 * 16)));
                        } else {
                            AtapiWritePortEx1(core::ptr::null_mut(), (&(*deviceExtension).BaseIoAddressSATA_0) as *const _ as ULONG_PTR, offs + 1,
                                AtapiReadPortEx1(core::ptr::null_mut(), (&(*deviceExtension).BaseIoAddressSATA_0) as *const _ as ULONG_PTR, offs + 1) & ((!(0x0d as UCHAR)) << ((Channel == 0) as u32 * 4)));
                        }
                        tmp16 = UniataSataPhyEnable(HwDeviceExtension, j, 0 /* dev0 */, UNIATA_SATA_RESET_ENABLE as BOOLEAN) as USHORT;

                        // enable device and PHY state change interrupts
                        if ChipFlags & NVQ != 0 {
                            AtapiWritePortEx4(core::ptr::null_mut(), (&(*deviceExtension).BaseIoAddressSATA_0) as *const _ as ULONG_PTR, offs + 4,
                                AtapiReadPortEx4(core::ptr::null_mut(), (&(*deviceExtension).BaseIoAddressSATA_0) as *const _ as ULONG_PTR, offs + 4) | ((0x0000000d as ULONG) << ((Channel == 0) as u32 * 16)));
                        } else {
                            AtapiWritePortEx1(core::ptr::null_mut(), (&(*deviceExtension).BaseIoAddressSATA_0) as *const _ as ULONG_PTR, offs + 1,
                                AtapiReadPortEx1(core::ptr::null_mut(), (&(*deviceExtension).BaseIoAddressSATA_0) as *const _ as ULONG_PTR, offs + 1) | ((0x0d as UCHAR) << ((Channel == 0) as u32 * 4)));
                        }

                        if tmp16 as u32 != IDE_STATUS_WRONG {
                            do_default_reset = true; break 'reset;
                        }
                    }
                    x if x == ATA_SILICON_IMAGE_ID => {
                        let offset: ULONG;
                        let Channel: ULONG = (*deviceExtension).Channel + j;
                        if ChipFlags & UNIATA_SATA == 0 {
                            do_default_reset = true; break 'reset;
                        }
                        offset = ((Channel & 1) << 7) + ((Channel & 2) << 8);
                        // disable PHY state change interrupt
                        AtapiWritePortEx4(core::ptr::null_mut(), (&(*deviceExtension).BaseIoAddressSATA_0) as *const _ as ULONG_PTR, 0x148 + offset, 0);

                        UniataSataClearErr(HwDeviceExtension, j, UNIATA_SATA_IGNORE_CONNECT as BOOLEAN, 0);

                        // reset controller part for this channel
                        AtapiWritePortEx4(core::ptr::null_mut(), (&(*deviceExtension).BaseIoAddressSATA_0) as *const _ as ULONG_PTR, 0x48,
                            AtapiReadPortEx4(core::ptr::null_mut(), (&(*deviceExtension).BaseIoAddressSATA_0) as *const _ as ULONG_PTR, 0x48) | (0xc0 >> Channel));
                        ScsiPortStallExecution(1000);
                        AtapiWritePortEx4(core::ptr::null_mut(), (&(*deviceExtension).BaseIoAddressSATA_0) as *const _ as ULONG_PTR, 0x48,
                            AtapiReadPortEx4(core::ptr::null_mut(), (&(*deviceExtension).BaseIoAddressSATA_0) as *const _ as ULONG_PTR, 0x48) & !(0xc0 >> Channel));
                    }
                    x if x == ATA_PROMISE_ID => {
                        // nothing
                    }
                    _ => {
                        if ChipFlags & UNIATA_SATA != 0 {
                            UniataSataClearErr(HwDeviceExtension, j, UNIATA_SATA_IGNORE_CONNECT as BOOLEAN, 0);
                        }
                        // default: falls through into default_reset in C
                        do_default_reset = true;
                    }
                }
            }

            if do_default_reset {
                // default_reset:
                AtapiHardReset(chan, TRUE, 100000);
                AtapiDisableInterrupts(deviceExtension as PVOID, j);
                ScsiPortStallExecution(100);
                AtapiEnableInterrupts(deviceExtension as PVOID, j);
            }

            // post-switch (runs for ALL ATA arms)
            if UniataIsSATARangeAvailable(deviceExtension, j) == 0 {
                // Reset DMA engine if active
                dma_status = GetDmaStatus((*chan).DeviceExtension, (*chan).lChannel);
                if (ChannelCtrlFlags & CTRFLAGS_DMA_ACTIVE) != 0 || (dma_status as u32 & BM_STATUS_INTR) != 0 {
                    AtapiDmaDone(HwDeviceExtension, 0, j, core::ptr::null_mut());
                }
            }
        } // ATA vs AHCI

        // process connected devices
        i = 0;
        while i < MaxLuns {
            if (*(*chan).lun[i as usize]).DeviceFlags & DFLAGS_DEVICE_PRESENT == 0 {
                if ChipFlags & UNIATA_AHCI != 0 {
                    i += 1;
                    continue;
                }
                if UniataAnybodyHome(HwDeviceExtension, j, i) == 0 {
                    i += 1;
                    continue;
                }
                if CheckDevice(HwDeviceExtension, j, i, TRUE) == 0 {
                    i += 1;
                    continue;
                }
            } else {
                if ChipFlags & UNIATA_AHCI != 0 {
                    let atapi_dev = ((*(*chan).lun[i as usize]).DeviceFlags & DFLAGS_ATAPI_DEVICE) != 0;
                    if IssueIdentify(
                        HwDeviceExtension, i, j,
                        if atapi_dev { IDE_COMMAND_ATAPI_IDENTIFY as UCHAR } else { IDE_COMMAND_IDENTIFY as UCHAR },
                        FALSE,
                    ) == 0
                    {
                        UniataForgetDevice((*chan).lun[i as usize]);
                    }
                    i += 1;
                    continue;
                }
                if UniataAnybodyHome(HwDeviceExtension, j, i) == 0 {
                    UniataForgetDevice((*chan).lun[i as usize]);
                }
            }

            SelectDrive(chan, i);
            ScsiPortStallExecution(10);
            statusByte = WaitOnBusyLong(chan);
            statusByte = UniataIsIdle(deviceExtension, statusByte);
            if statusByte as u32 == IDE_STATUS_WRONG {
                UniataForgetDevice((*chan).lun[i as usize]);
            } else if ((*(*chan).lun[i as usize]).DeviceFlags & DFLAGS_ATAPI_DEVICE) != 0 {
                // Issue soft reset and issue identify.
                statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus); // GetStatus

                AtapiDisableInterrupts(deviceExtension as PVOID, j);
                AtapiSoftReset(chan, i);
                AtapiEnableInterrupts(deviceExtension as PVOID, j);

                statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus); // GetStatus

                if statusByte as u32 != IDE_STATUS_SUCCESS {
                    let mut k: ULONG = UniataAnybodyHome(deviceExtension as PVOID, j, i);
                    if k == ATA_AT_HOME_HDD {
                        // device reset in progress, perform additional wait
                        k = 30 * 1000;
                        while (AtapiReadPort1(chan, IDX_IO1_i_Status) as u32 & IDE_STATUS_BUSY) != 0 && k != 0 {
                            k -= 1;
                            ScsiPortStallExecution(150);
                        }
                        statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus); // GetStatus
                    }
                }
                if statusByte as u32 == IDE_STATUS_SUCCESS {
                    IssueIdentify(HwDeviceExtension, i, j, IDE_COMMAND_ATAPI_IDENTIFY as UCHAR, FALSE);
                }
                statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
            } else {
                // Issue identify and reinit after channel reset.
                if statusByte as u32 != IDE_STATUS_IDLE
                    && statusByte as u32 != IDE_STATUS_SUCCESS
                    && statusByte as u32 != IDE_STATUS_DRDY
                {
                    // IdeHardReset failed
                } else if IssueIdentify(HwDeviceExtension, i, j, IDE_COMMAND_IDENTIFY as UCHAR, FALSE) == 0 {
                    // IDE IssueIdentify failed
                } else if SetDriveParameters(HwDeviceExtension, i, j) == 0 {
                    // SetDriveParameters failed
                }
                statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
            }
            // force DMA mode reinit
            (*(*chan).lun[i as usize]).DeviceFlags |= DFLAGS_REINIT_DMA;
            i += 1;
        }

        // Enable interrupts, note, we can have here recursive disable
        ScsiPortStallExecution(10);
        AtapiEnableInterrupts(deviceExtension as PVOID, j);

        // Call the HwInitialize routine to setup multi-block.
        AtapiHwInitialize__(deviceExtension, j);
        j += 1;
    } // for(channel)
    ScsiPortNotification(
        _SCSI_NOTIFICATION_TYPE_NextRequest as SCSI_NOTIFICATION_TYPE,
        deviceExtension as PVOID, core::ptr::null_mut::<core::ffi::c_void>(),
    );

    TRUE
}

// helper to populate the basic ErrorCode/Valid/AdditionalSenseLength prefix
unsafe fn sense_prefix(sb: PSENSE_DATA) {
    (*sb).set_ErrorCode(0x70);
    (*sb).set_Valid(1);
    (*sb).AdditionalSenseLength = 0xb;
}

// ============================================================================
// MapError — map ATAPI/IDE errors to SRB statuses.
// ============================================================================

pub unsafe extern "system" fn MapError(
    HwDeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
) -> ULONG {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let lChannel = (*Srb).PathId as ULONG; // GET_CHANNEL
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let mut errorByte: UCHAR = 0;
    let mut srbStatus: UCHAR = SRB_STATUS_SUCCESS as UCHAR;
    let mut scsiStatus: UCHAR;
    let DeviceNumber = (*Srb).TargetId as ULONG; // GET_CDEV
    let LunExt: PHW_LU_EXTENSION = (*chan).lun[DeviceNumber as usize];

    // Read the error register.
    if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
        let AtaReq = (*Srb).SrbExtension as PATA_REQ;
        if !AtaReq.is_null() {
            errorByte = (*ata_req_ahci(AtaReq)).in_error;
        }
    } else {
        errorByte = AtapiReadPort1(chan, IDX_IO1_i_Error);
    }

    if (*LunExt).DeviceFlags & DFLAGS_ATAPI_DEVICE != 0 {
        match (errorByte >> 4) as u32 {
            SCSI_SENSE_NO_SENSE => {
                scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
                srbStatus = SRB_STATUS_ERROR as UCHAR;
            }
            x if x == 1 /* SCSI_SENSE_RECOVERED_ERROR */ => {
                scsiStatus = 0;
                srbStatus = SRB_STATUS_SUCCESS as UCHAR;
            }
            x if x == 2 /* SCSI_SENSE_NOT_READY */ => {
                scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
                srbStatus = SRB_STATUS_ERROR as UCHAR;
            }
            SCSI_SENSE_MEDIUM_ERROR => {
                scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
                srbStatus = SRB_STATUS_ERROR as UCHAR;
            }
            SCSI_SENSE_HARDWARE_ERROR => {
                scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
                srbStatus = SRB_STATUS_ERROR as UCHAR;
            }
            SCSI_SENSE_ILLEGAL_REQUEST => {
                scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
                srbStatus = SRB_STATUS_ERROR as UCHAR;
            }
            SCSI_SENSE_UNIT_ATTENTION => {
                scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
                srbStatus = SRB_STATUS_ERROR as UCHAR;
            }
            x if x == 7 /* SCSI_SENSE_DATA_PROTECT */ => {
                scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
                srbStatus = SRB_STATUS_ERROR as UCHAR;
            }
            x if x == 8 /* SCSI_SENSE_BLANK_CHECK */ => {
                scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
                srbStatus = SRB_STATUS_ERROR as UCHAR;
            }
            SCSI_SENSE_ABORTED_COMMAND => {
                scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
                srbStatus = SRB_STATUS_ERROR as UCHAR;
            }
            _ => {
                scsiStatus = 0;
                srbStatus = SRB_STATUS_ERROR as UCHAR;
            }
        }
    } else {
        scsiStatus = 0;

        // Save errorByte, to be used by SCSIOP_REQUEST_SENSE.
        (*chan).ReturningMediaStatus = errorByte;

        if errorByte as u32 & IDE_ERROR_MEDIA_CHANGE_REQ != 0 {
            scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
            srbStatus = SRB_STATUS_ERROR as UCHAR;
            if !(*Srb).SenseInfoBuffer.is_null() {
                let sb = (*Srb).SenseInfoBuffer as PSENSE_DATA;
                sense_prefix(sb);
                (*sb).set_SenseKey(SCSI_SENSE_UNIT_ATTENTION as UCHAR);
                (*sb).AdditionalSenseCode = SCSI_ADSENSE_MEDIUM_CHANGED as UCHAR;
                (*sb).AdditionalSenseCodeQualifier = 0;
                srbStatus |= SRB_STATUS_AUTOSENSE_VALID as UCHAR;
            }
        } else if errorByte as u32 & IDE_ERROR_COMMAND_ABORTED != 0 {
            srbStatus = SRB_STATUS_ABORTED as UCHAR;
            scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
            if !(*Srb).SenseInfoBuffer.is_null() {
                let sb = (*Srb).SenseInfoBuffer as PSENSE_DATA;
                sense_prefix(sb);
                (*sb).set_SenseKey(SCSI_SENSE_ABORTED_COMMAND as UCHAR);
                (*sb).AdditionalSenseCode = 0;
                (*sb).AdditionalSenseCodeQualifier = 0;
                srbStatus |= SRB_STATUS_AUTOSENSE_VALID as UCHAR;
            }
            (*LunExt).ErrorCount += 1;
        } else if errorByte as u32 & IDE_ERROR_END_OF_MEDIA != 0 {
            scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
            srbStatus = SRB_STATUS_ERROR as UCHAR;
            if !(*Srb).SenseInfoBuffer.is_null() {
                let sb = (*Srb).SenseInfoBuffer as PSENSE_DATA;
                sense_prefix(sb);
                (*sb).set_SenseKey(SCSI_SENSE_UNIT_ATTENTION as UCHAR);
                (*sb).AdditionalSenseCode = SCSI_ADSENSE_MEDIA_STATE as UCHAR;
                (*sb).AdditionalSenseCodeQualifier = SCSI_SENSEQ_END_OF_MEDIUM as UCHAR;
                (*sb).set_EndOfMedia(1);
                srbStatus |= SRB_STATUS_AUTOSENSE_VALID as UCHAR;
            }
            if (*LunExt).DeviceFlags & DFLAGS_MEDIA_STATUS_ENABLED == 0 {
                (*LunExt).ErrorCount += 1;
            }
        } else if errorByte as u32 & IDE_ERROR_ILLEGAL_LENGTH != 0 {
            srbStatus = SRB_STATUS_INVALID_REQUEST as UCHAR;
            if !(*Srb).SenseInfoBuffer.is_null() {
                let sb = (*Srb).SenseInfoBuffer as PSENSE_DATA;
                sense_prefix(sb);
                (*sb).set_SenseKey(SCSI_SENSE_ILLEGAL_REQUEST as UCHAR);
                (*sb).AdditionalSenseCode = SCSI_ADSENSE_INVALID_VALUE as UCHAR;
                (*sb).AdditionalSenseCodeQualifier = SCSI_SENSEQ_PARAM_INVALID_VALUE as UCHAR;
                (*sb).set_IncorrectLength(1);
                srbStatus |= SRB_STATUS_AUTOSENSE_VALID as UCHAR;
            }
        } else if errorByte as u32 & IDE_ERROR_BAD_BLOCK != 0 {
            srbStatus = SRB_STATUS_ERROR as UCHAR;
            scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
            if !(*Srb).SenseInfoBuffer.is_null() {
                let sb = (*Srb).SenseInfoBuffer as PSENSE_DATA;
                sense_prefix(sb);
                (*sb).set_SenseKey(SCSI_SENSE_MEDIUM_ERROR as UCHAR);
                (*sb).AdditionalSenseCode = 0;
                (*sb).AdditionalSenseCodeQualifier = 0;
                srbStatus |= SRB_STATUS_AUTOSENSE_VALID as UCHAR;
            }
        } else if errorByte as u32 & IDE_ERROR_ID_NOT_FOUND != 0 {
            srbStatus = SRB_STATUS_ERROR as UCHAR;
            scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
            if !(*Srb).SenseInfoBuffer.is_null() {
                let sb = (*Srb).SenseInfoBuffer as PSENSE_DATA;
                sense_prefix(sb);
                (*sb).set_SenseKey(SCSI_SENSE_MEDIUM_ERROR as UCHAR);
                (*sb).AdditionalSenseCode = 0;
                (*sb).AdditionalSenseCodeQualifier = 0;
                srbStatus |= SRB_STATUS_AUTOSENSE_VALID as UCHAR;
            }
            (*LunExt).ErrorCount += 1;
        } else if errorByte as u32 & IDE_ERROR_MEDIA_CHANGE != 0 {
            scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
            srbStatus = SRB_STATUS_ERROR as UCHAR;
            if !(*Srb).SenseInfoBuffer.is_null() {
                let sb = (*Srb).SenseInfoBuffer as PSENSE_DATA;
                sense_prefix(sb);
                (*sb).set_SenseKey(SCSI_SENSE_UNIT_ATTENTION as UCHAR);
                (*sb).AdditionalSenseCode = SCSI_ADSENSE_MEDIUM_CHANGED as UCHAR;
                (*sb).AdditionalSenseCodeQualifier = 0;
                srbStatus |= SRB_STATUS_AUTOSENSE_VALID as UCHAR;
            }
        } else if errorByte as u32 & IDE_ERROR_DATA_ERROR != 0 {
            scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
            srbStatus = SRB_STATUS_ERROR as UCHAR;
            if (*LunExt).DeviceFlags & DFLAGS_MEDIA_STATUS_ENABLED == 0 {
                (*LunExt).ErrorCount += 1;
            }
            if !(*Srb).SenseInfoBuffer.is_null() {
                let sb = (*Srb).SenseInfoBuffer as PSENSE_DATA;
                sense_prefix(sb);
                (*sb).set_SenseKey(SCSI_SENSE_MEDIUM_ERROR as UCHAR);
                (*sb).AdditionalSenseCode = 0;
                (*sb).AdditionalSenseCodeQualifier = 0;
                srbStatus |= SRB_STATUS_AUTOSENSE_VALID as UCHAR;
            }
        }

        if (*LunExt).ErrorCount >= MAX_ERRORS {
            (*LunExt).DeviceFlags &= !DFLAGS_DWORDIO_ENABLED;
            (*LunExt).MaximumBlockXfer = 0;

            ScsiPortLogError(
                HwDeviceExtension, Srb, (*Srb).PathId, (*Srb).TargetId, (*Srb).Lun,
                SP_BAD_FW_WARNING, 4,
            );

            // Reprogram to not use Multi-sector.
            let statusByte: UCHAR;
            if (*LunExt).DeviceFlags & DFLAGS_DEVICE_PRESENT != 0
                && ((*LunExt).DeviceFlags & (DFLAGS_ATAPI_DEVICE | DFLAGS_MANUAL_CHS)) == 0
            {
                statusByte = AtaCommand(
                    deviceExtension, DeviceNumber, lChannel,
                    IDE_COMMAND_SET_MULTIPLE as UCHAR, 0, 0, 0, 0, 0, ATA_WAIT_BASE_READY as ULONG,
                );
                if statusByte as u32 & IDE_STATUS_ERROR != 0 {
                    errorByte = AtapiReadPort1(chan, IDX_IO1_i_Error);
                    let _ = errorByte;
                    (*LunExt).MaximumBlockXfer = 0;
                }
            }
        }
    }

    // Set SCSI status to indicate a check condition.
    (*Srb).ScsiStatus = scsiStatus;
    srbStatus as ULONG
}

// ============================================================================
// AtapiHwInitialize / AtapiHwInitialize__  (->HwInitialize)
// ============================================================================

pub unsafe extern "system" fn AtapiHwInitialize(HwDeviceExtension: PVOID) -> BOOLEAN {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let numberChannels = (*deviceExtension).NumberChannels;
    let mut c: ULONG;

    DbgPrint(b"[UNIATA] HwInitialize entry\n\0".as_ptr().cast()); // perf phase marker
    if WinVer_WDM_Model != 0 {
        AtapiResetController__(HwDeviceExtension, CHAN_NOT_SPECIFIED, RESET_COMPLETE_ALL);
    }
    if (*deviceExtension).MasterDev != 0 {
        (*BMList.add((*deviceExtension).DevIndex as usize)).ChanInitOk |=
            (0x01u32 << (*deviceExtension).Channel) as UCHAR;
    }

    // do extra chipset specific setups
    AtapiChipInit(HwDeviceExtension, DEVNUM_NOT_SPECIFIED, CHAN_NOT_SPECIFIED);

    c = 0;
    while c < numberChannels {
        AtapiHwInitialize__(deviceExtension, c);
        c += 1;
    }
    DbgPrint(b"[UNIATA] HwInitialize done\n\0".as_ptr().cast()); // perf phase marker
    TRUE
}

pub unsafe extern "system" fn AtapiHwInitialize__(
    deviceExtension: PHW_DEVICE_EXTENSION,
    lChannel: ULONG,
) {
    let mut i: ULONG;
    let mut statusByte: UCHAR;
    let mut errorByte: UCHAR;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let mut LunExt: PHW_LU_EXTENSION;
    let mut PreferedMode: ULONG;

    if (*deviceExtension).HwFlags & UNIATA_AHCI != 0
        && UniataAhciChanImplemented(deviceExtension, lChannel) == 0
    {
        return;
    }

    AtapiChipInit(deviceExtension as PVOID, DEVNUM_NOT_SPECIFIED, lChannel);
    FindDevices(deviceExtension as PVOID, 0, lChannel);

    i = 0;
    while i < (*chan).NumberLuns {
        LunExt = (*chan).lun[i as usize];
        // skip empty slots
        if (*LunExt).DeviceFlags & DFLAGS_DEVICE_PRESENT == 0 {
            i += 1;
            continue;
        }

        AtapiDisableInterrupts(deviceExtension as PVOID, lChannel);
        ScsiPortStallExecution(1);

        if (*LunExt).DeviceFlags & (DFLAGS_ATAPI_DEVICE | DFLAGS_MANUAL_CHS) == 0 {
            // IDE branch
            IdeMediaStatus(TRUE, deviceExtension as PVOID, lChannel, i);

            // If supported, setup Multi-block transfers.
            statusByte = AtaCommand(
                deviceExtension, i, lChannel,
                IDE_COMMAND_SET_MULTIPLE as UCHAR, 0, 0, 0,
                (*LunExt).MaximumBlockXfer as UCHAR, 0, ATA_WAIT_BASE_READY as ULONG,
            );

            if statusByte as u32 & IDE_STATUS_ERROR != 0 {
                errorByte = AtapiReadPort1(chan, IDX_IO1_i_Error);
                let _ = errorByte;
                statusByte = AtaCommand(
                    deviceExtension, i, lChannel,
                    IDE_COMMAND_SET_MULTIPLE as UCHAR, 0, 0, 0,
                    0, 0, ATA_WAIT_BASE_READY as ULONG,
                );
                if statusByte as u32 & IDE_STATUS_ERROR != 0 {
                    errorByte = AtapiReadPort1(chan, IDX_IO1_i_Error);
                    let _ = errorByte;
                }
                (*LunExt).MaximumBlockXfer = 0;
            }

            if (*LunExt).IdentifyData.MajorRevision != 0 {
                if (*LunExt).opt_ReadCacheEnable != 0 {
                    statusByte = AtaCommand(
                        deviceExtension, i, lChannel,
                        IDE_COMMAND_SET_FEATURES as UCHAR, 0, 0, 0,
                        0, ATA_C_F_ENAB_RCACHE as UCHAR, ATA_WAIT_BASE_READY as ULONG,
                    );
                    if statusByte as u32 & IDE_STATUS_ERROR != 0 {
                        (*LunExt).DeviceFlags &= !DFLAGS_RCACHE_ENABLED;
                    } else {
                        (*LunExt).DeviceFlags |= DFLAGS_RCACHE_ENABLED;
                    }
                } else {
                    statusByte = AtaCommand(
                        deviceExtension, i, lChannel,
                        IDE_COMMAND_SET_FEATURES as UCHAR, 0, 0, 0,
                        0, ATA_C_F_DIS_RCACHE as UCHAR, ATA_WAIT_BASE_READY as ULONG,
                    );
                    (*LunExt).DeviceFlags &= !DFLAGS_RCACHE_ENABLED;
                }
                if (*LunExt).IdentifyData.FeaturesSupport.WriteCache() != 0 {
                    if (*LunExt).opt_WriteCacheEnable != 0 {
                        statusByte = AtaCommand(
                            deviceExtension, i, lChannel,
                            IDE_COMMAND_SET_FEATURES as UCHAR, 0, 0, 0,
                            0, ATA_C_F_ENAB_WCACHE as UCHAR, ATA_WAIT_BASE_READY as ULONG,
                        );
                        if statusByte as u32 & IDE_STATUS_ERROR != 0 {
                            (*LunExt).DeviceFlags &= !DFLAGS_WCACHE_ENABLED;
                        } else {
                            (*LunExt).DeviceFlags |= DFLAGS_WCACHE_ENABLED;
                        }
                    } else {
                        statusByte = AtaCommand(
                            deviceExtension, i, lChannel,
                            IDE_COMMAND_SET_FEATURES as UCHAR, 0, 0, 0,
                            0, ATA_C_F_DIS_WCACHE as UCHAR, ATA_WAIT_BASE_READY as ULONG,
                        );
                        (*LunExt).DeviceFlags &= !DFLAGS_WCACHE_ENABLED;
                    }
                }

                if (*LunExt).IdentifyData.FeaturesSupport.APM() != 0 {
                    if (*LunExt).opt_AdvPowerMode != 0 {
                        statusByte = AtaCommand(
                            deviceExtension, i, lChannel,
                            IDE_COMMAND_SET_FEATURES as UCHAR, 0, 0, 0,
                            (*LunExt).opt_AdvPowerMode as UCHAR, ATA_C_F_ENAB_APM as UCHAR,
                            ATA_WAIT_BASE_READY as ULONG,
                        );
                    } else {
                        statusByte = AtaCommand(
                            deviceExtension, i, lChannel,
                            IDE_COMMAND_SET_FEATURES as UCHAR, 0, 0, 0,
                            0, ATA_C_F_DIS_APM as UCHAR, ATA_WAIT_BASE_READY as ULONG,
                        );
                    }
                }
                if (*LunExt).IdentifyData.FeaturesSupport.AutoAcoustic() != 0 {
                    if (*LunExt).opt_AcousticMode != 0 {
                        statusByte = AtaCommand(
                            deviceExtension, i, lChannel,
                            IDE_COMMAND_SET_FEATURES as UCHAR, 0, 0, 0,
                            (*LunExt).opt_AcousticMode as UCHAR, ATA_C_F_ENAB_ACOUSTIC as UCHAR,
                            ATA_WAIT_BASE_READY as ULONG,
                        );
                    } else {
                        statusByte = AtaCommand(
                            deviceExtension, i, lChannel,
                            IDE_COMMAND_SET_FEATURES as UCHAR, 0, 0, 0,
                            0, ATA_C_F_DIS_ACOUSTIC as UCHAR, ATA_WAIT_BASE_READY as ULONG,
                        );
                    }
                }
                if (*LunExt).IdentifyData.FeaturesSupport.Standby() != 0 {
                    statusByte = AtaCommand(
                        deviceExtension, i, lChannel,
                        IDE_COMMAND_IDLE as UCHAR, 0, 0, 0,
                        (*LunExt).opt_StandbyTimer as UCHAR, 0, ATA_WAIT_BASE_READY as ULONG,
                    );
                }
            }
        } else if (*LunExt).DeviceFlags & DFLAGS_CHANGER_INITED == 0 {
            // ATAPI/Changer branch
            let mut j: ULONG;
            let mut vendorId: [CCHAR; 26] = [0; 26];

            // Attempt to identify any special-case devices.
            j = 0;
            while j < 26 {
                // MOV_DW_SWP(vendorId[j], ((PUCHAR)ModelNumber)[j])
                let src = (core::ptr::addr_of!((*LunExt).IdentifyData.ModelNumber) as *const UCHAR).add(j as usize);
                let dst = (&mut vendorId[j as usize]) as *mut CCHAR as *mut USHORT;
                *dst = RtlUshortByteSwap(*(src as *const USHORT));
                j += 2;
            }

            if _strnicmp(vendorId.as_ptr() as PCCHAR, b"CD-ROM  CDR\0".as_ptr() as PCCHAR, 11) == 0 {
                // Inquiry string for older model had a '-', newer is '_'
                if vendorId[12] == (b'C' as CCHAR) {
                    (*LunExt).DeviceFlags |= DFLAGS_CHANGER_INITED | DFLAGS_SANYO_ATAPI_CHANGER;
                    (*LunExt).DiscsPresent = 3;
                }
            }
        }

        PreferedMode = (*LunExt).opt_MaxTransferMode;
        if PreferedMode == 0xffffffff || PreferedMode > (*chan).MaxTransferMode {
            PreferedMode = (*chan).MaxTransferMode;
        }

        if (*LunExt).opt_PreferedTransferMode != 0xffffffff {
            PreferedMode = core::cmp::min((*LunExt).opt_PreferedTransferMode, PreferedMode);
        }

        (*LunExt).TransferMode = PreferedMode as UCHAR;
        (*LunExt).LimitedTransferMode = (*LunExt).TransferMode;

        AtapiDmaInit__(deviceExtension, LunExt);

        (*LunExt).LimitedTransferMode = (*LunExt).TransferMode;

        // get the device ready for action.
        if (*LunExt).DeviceFlags & DFLAGS_ATAPI_DEVICE != 0 {
            let mut waitCount: ULONG;
            waitCount = 10000;
            statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus); // GetStatus
            if statusByte as u32 == IDE_STATUS_WRONG {
                waitCount = 0;
            }
            while (statusByte as u32 & IDE_STATUS_BUSY) != 0 && waitCount != 0 {
                ScsiPortStallExecution(100);
                statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus); // GetStatus
                waitCount -= 1;
            }

            // Mechanical-settle delay for real ATAPI hardware. Virtual drives
            // are ready the instant the reset completes — skip the 500ms
            // (5000 x 100us, ~880ms real under TCG) dead stall in a VM.
            if statusByte as u32 != IDE_STATUS_WRONG && g_opt_VirtualMachine <= VM_NONE {
                waitCount = 5000;
                loop {
                    ScsiPortStallExecution(100);
                    let wc = waitCount;
                    waitCount = waitCount.wrapping_sub(1);
                    if wc == 0 {
                        break;
                    }
                }
            }
        }
        statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
        let _ = statusByte;
        AtapiEnableInterrupts(deviceExtension as PVOID, lChannel);
        ScsiPortStallExecution(10);
        i += 1;
    }
}

// ============================================================================
// AtapiHwInitializeChanger  (#ifndef UNIATA_CORE)
// ============================================================================

pub unsafe extern "system" fn AtapiHwInitializeChanger(
    HwDeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
    MechanismStatus: PMECHANICAL_STATUS_INFORMATION_HEADER,
) {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let lChannel = (*Srb).PathId as ULONG; // GET_CHANNEL
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let DeviceNumber = (*Srb).TargetId as ULONG; // GET_CDEV
    let LunExt: PHW_LU_EXTENSION = (*chan).lun[DeviceNumber as usize];

    if !MechanismStatus.is_null() {
        (*LunExt).DiscsPresent = (*MechanismStatus).NumberAvailableSlots as ULONG;
        if (*LunExt).DiscsPresent > 1 {
            (*LunExt).DeviceFlags |= DFLAGS_ATAPI_CHANGER;
        }
    }
}

// ============================================================================
// AtapiParseArgumentString
// ============================================================================

pub unsafe extern "system" fn AtapiParseArgumentString(
    String: PCCH,
    KeyWord: PCCH,
) -> ULONG {
    if String.is_null() {
        return 0;
    }
    if KeyWord.is_null() {
        return 0;
    }

    // Calculate the string length.
    let mut stringLength: ULONG = 0;
    {
        let mut cptr = String;
        while *cptr != 0 {
            cptr = cptr.add(1);
            stringLength += 1;
        }
    }

    // Calculate the keyword length.
    let mut keyWordLength: ULONG = 0;
    {
        let mut kptr = KeyWord;
        while *kptr != 0 {
            kptr = kptr.add(1);
            keyWordLength += 1;
        }
    }

    if keyWordLength > stringLength {
        return 0;
    }

    // Now setup and start the compare.
    let mut cptr = String;

    'continue_search: loop {
        // The input string may start with white space. Skip it.
        while (*cptr as i32) == (' ' as i32) || (*cptr as i32) == ('\t' as i32) {
            cptr = cptr.add(1);
        }
        if *cptr == 0 {
            return 0;
        }

        let mut kptr = KeyWord;
        while {
            let c = *cptr as i32;
            let k = *kptr as i32;
            (c == k)
                || (c >= ('A' as i32) && c <= ('Z' as i32) && c + ('a' as i32 - 'A' as i32) == k)
                || (c >= ('a' as i32) && c <= ('z' as i32) && c - ('a' as i32 - 'A' as i32) == k)
        } {
            cptr = cptr.add(1);
            kptr = kptr.add(1);
            if *cptr == 0 {
                return 0;
            }
        }

        if *kptr == 0 {
            // May have a match. backup and check for blank or equals.
            while (*cptr as i32) == (' ' as i32) || (*cptr as i32) == ('\t' as i32) {
                cptr = cptr.add(1);
            }

            // Found a match. Make sure there is an equals.
            if (*cptr as i32) != ('=' as i32) {
                // Not a match so move to the next semicolon.
                while *cptr != 0 {
                    let ch = *cptr;
                    cptr = cptr.add(1);
                    if (ch as i32) == (';' as i32) {
                        continue 'continue_search;
                    }
                }
                return 0;
            }
            // Skip the equals sign.
            cptr = cptr.add(1);

            // Skip white space.
            while (*cptr as i32) == (' ' as i32) || (*cptr as i32) == ('\t' as i32) {
                cptr = cptr.add(1);
            }
            if *cptr == 0 {
                return 0;
            }
            if (*cptr as i32) == (';' as i32) {
                cptr = cptr.add(1);
                continue 'continue_search;
            }

            let mut value: ULONG = 0;
            if (*cptr as i32) == ('0' as i32)
                && ((*cptr.add(1) as i32) == ('x' as i32) || (*cptr.add(1) as i32) == ('X' as i32))
            {
                // Value is in Hex. Skip the "0x"
                cptr = cptr.add(2);
                let mut index: isize = 0;
                while *cptr.add(index as usize) != 0 {
                    let c = *cptr.add(index as usize);
                    if (c as i32) == (' ' as i32) || (c as i32) == ('\t' as i32) || (c as i32) == (';' as i32) {
                        break;
                    }
                    if (c as i32) >= ('0' as i32) && (c as i32) <= ('9' as i32) {
                        value = 16 * value + ((c as i32) - ('0' as i32)) as ULONG;
                    } else if (c as i32) >= ('a' as i32) && (c as i32) <= ('f' as i32) {
                        value = 16 * value + ((c as i32) - ('a' as i32) + 10) as ULONG;
                    } else if (c as i32) >= ('A' as i32) && (c as i32) <= ('F' as i32) {
                        value = 16 * value + ((c as i32) - ('A' as i32) + 10) as ULONG;
                    } else {
                        return 0;
                    }
                    index += 1;
                }
            } else {
                // Value is in Decimal.
                let mut index: isize = 0;
                while *cptr.add(index as usize) != 0 {
                    let c = *cptr.add(index as usize);
                    if (c as i32) == (' ' as i32) || (c as i32) == ('\t' as i32) || (c as i32) == (';' as i32) {
                        break;
                    }
                    if (c as i32) >= ('0' as i32) && (c as i32) <= ('9' as i32) {
                        value = 10 * value + ((c as i32) - ('0' as i32)) as ULONG;
                    } else {
                        return 0;
                    }
                    index += 1;
                }
            }

            return value;
        } else {
            // Not a match. check for ';' to continue search.
            while *cptr != 0 {
                let ch = *cptr;
                cptr = cptr.add(1);
                if (ch as i32) == (';' as i32) {
                    continue 'continue_search;
                }
            }
            return 0;
        }
    }
}

// ============================================================================
// AtapiCallBack__ / AtapiCallBack_X  (#ifndef UNIATA_CORE) — timer callback
//   UNIATA_USE_XXableInterrupts is NOT defined: use the AtapiEnableInterrupts +
//   AtapiQueueTimerDpc(AtapiEnableInterrupts__,1) path.
// ============================================================================

pub unsafe extern "system" fn AtapiCallBack__(
    HwDeviceExtension: PVOID,
    lChannel: UCHAR,
) {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let mut chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let mut c: ULONG;
    let mut _c: ULONG;

    let mut srb: PSCSI_REQUEST_BLOCK = UniataGetCurRequest(chan);
    let mut statusByte: UCHAR;

    // 'return_callback / return_enable_intr restructured as labeled blocks.
    'return_callback: {
        // If ISR decided to wait for BUSY/DRQ in DPC, we also get here.
        if srb.is_null() || (*chan).ExpectingInterrupt != 0 {
            (*chan).DpcState = DPC_STATE_TIMER as UCHAR;
            if AtapiInterrupt__(HwDeviceExtension, lChannel) == 0 {
                InterlockedExchange(&mut (*chan).CheckIntr, CHECK_INTR_IDLE as LONG);
            }
            break 'return_callback;
        }

        'return_enable_intr: {
            if (*chan).RDP == 0 {
                break 'return_enable_intr;
            }
            statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus); // GetStatus
            if statusByte as u32 & IDE_STATUS_DSC != 0 {
                let PathId = (*srb).PathId;
                let TargetId = (*srb).TargetId;
                let Lun = (*srb).Lun;

                AtapiDmaDBSync(chan, srb);
                UniataRemoveRequest(chan, srb);
                ScsiPortNotification(
                    _SCSI_NOTIFICATION_TYPE_RequestComplete as SCSI_NOTIFICATION_TYPE,
                    deviceExtension as PVOID, srb,
                );
                // Clear current SRB.
                if (*deviceExtension).simplexOnly == 0 {
                    srb = UniataGetCurRequest(chan);
                } else {
                    srb = core::ptr::null_mut();
                }
                (*chan).RDP = FALSE;

                ScsiPortNotification(
                    _SCSI_NOTIFICATION_TYPE_NextLuRequest as SCSI_NOTIFICATION_TYPE,
                    deviceExtension as PVOID, PathId as c_uint, TargetId as c_uint, Lun as c_uint,
                );
                ScsiPortNotification(
                    _SCSI_NOTIFICATION_TYPE_NextRequest as SCSI_NOTIFICATION_TYPE,
                    deviceExtension as PVOID, core::ptr::null_mut::<core::ffi::c_void>(),
                );

                if !srb.is_null() {
                    AtapiStartIo__(HwDeviceExtension, srb, FALSE);
                }
            } else {
                AtapiQueueTimerDpc(HwDeviceExtension, lChannel as ULONG, core::mem::transmute::<_, PHW_TIMER>(Some(AtapiCallBack_X as unsafe extern "system" fn(PVOID))), 1000);
                break 'return_callback;
            }

            // return_enable_intr:
            if CrNtInterlockedExchangeAdd(&mut (*chan).DisableIntr, 0) != 0 {
                AtapiEnableInterrupts(HwDeviceExtension, lChannel as ULONG);
                InterlockedExchange(&mut (*chan).CheckIntr, CHECK_INTR_IDLE as LONG);
                AtapiQueueTimerDpc(HwDeviceExtension, lChannel as ULONG, core::mem::transmute::<_, PHW_TIMER>(Some(AtapiEnableInterrupts__ as unsafe extern "system" fn(PVOID))), 1);
            }
        }
    }

    // return_callback:
    // Check other channel. In simplex mode no interrupts must appear on other channels.
    _c = 0;
    while _c < (*deviceExtension).NumberChannels {
        c = (_c + (*deviceExtension).FirstChannelToCheck) % (*deviceExtension).NumberChannels;
        if c == lChannel as ULONG {
            _c += 1;
            continue;
        }
        chan = (*deviceExtension).chan.add(c as usize);
        if CrNtInterlockedCompareExchange(
            &mut (*chan).CheckIntr,
            CHECK_INTR_ACTIVE as LONG,
            CHECK_INTR_DETECTED as LONG,
        ) == CHECK_INTR_DETECTED as LONG
        {
            (*chan).DpcState = DPC_STATE_ISR as UCHAR;
            if AtapiInterrupt__(HwDeviceExtension, c as UCHAR) == 0 {
                InterlockedExchange(&mut (*chan).CheckIntr, CHECK_INTR_IDLE as LONG);
            }
        }
        _c += 1;
    }
}

pub unsafe extern "system" fn AtapiCallBack_X(HwDeviceExtension: PVOID) {
    let dev = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    AtapiCallBack__(HwDeviceExtension, (*dev).ActiveDpcChan as UCHAR);
}

// ============================================================================
// AtapiInterrupt — the ISR. Returns TRUE if expecting an interrupt.
// ============================================================================

pub unsafe extern "system" fn AtapiInterrupt(HwDeviceExtension: PVOID) -> BOOLEAN {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let mut c: ULONG;
    let mut _c: ULONG;
    let mut status: BOOLEAN = FALSE;
    let mut c_state: ULONG;
    let mut i_res: ULONG = 0;
    let mut pass: ULONG;
    let hIS: ULONG;
    let mut checked: ULONG;

    if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
        hIS = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_IS as ULONG);
        if hIS == 0 {
            return FALSE;
        }
        checked = !hIS;
        checked |= !(*deviceExtension).AHCI_PI;
    } else {
        checked = 0;
    }

    // clear this flag now, it can be set again in sub-calls
    (*deviceExtension).ExpectingInterrupt = 0;

    pass = 0;
    while pass < 2 {
        if status != 0 && pass != 0 {
            break;
        }
        _c = 0;
        while _c < (*deviceExtension).NumberChannels {
            c = (_c + (*deviceExtension).FirstChannelToCheck) % (*deviceExtension).NumberChannels;

            if (checked >> c) & 0x01 != 0 {
                _c += 1;
                continue;
            }
            // check non-empty and expecting interrupt channels first
            if pass == 0 && (*(*deviceExtension).chan.add(c as usize)).ExpectingInterrupt == 0 {
                _c += 1;
                continue;
            }
            checked |= (1u32) << c;

            if CrNtInterlockedExchangeAdd(&mut (*(*deviceExtension).chan.add(c as usize)).DisableIntr, 0) != 0 {
                _c += 1;
                continue;
            }
            // lock channel.
            loop {
                c_state = CrNtInterlockedCompareExchange(
                    &mut (*(*deviceExtension).chan.add(c as usize)).CheckIntr,
                    CHECK_INTR_ACTIVE as LONG, CHECK_INTR_DETECTED as LONG,
                ) as ULONG;
                if c_state == CHECK_INTR_IDLE {
                    c_state = CrNtInterlockedCompareExchange(
                        &mut (*(*deviceExtension).chan.add(c as usize)).CheckIntr,
                        CHECK_INTR_ACTIVE as LONG, CHECK_INTR_IDLE as LONG,
                    ) as ULONG;
                }
                if c_state != CHECK_INTR_CHECK {
                    break;
                }
            }
            // check if already serviced
            if c_state == CHECK_INTR_ACTIVE {
                _c += 1;
                continue;
            }

            if c_state == CHECK_INTR_DETECTED || {
                i_res = AtapiCheckInterrupt__(deviceExtension as PVOID, c as UCHAR) as ULONG;
                i_res != 0
            } {
                if i_res == INTERRUPT_REASON_UNEXPECTED {
                    InterlockedExchange(&mut (*(*deviceExtension).chan.add(c as usize)).CheckIntr, CHECK_INTR_IDLE as LONG);
                    status = TRUE;
                    _c += 1;
                    continue;
                }
                // disable interrupts on other channel of legacy mode
                if (*deviceExtension).simplexOnly != 0 {
                    AtapiDisableInterrupts(deviceExtension as PVOID, (c == 0) as ULONG);
                }

                (*(*deviceExtension).chan.add(c as usize)).DpcState = DPC_STATE_ISR as UCHAR;
                if AtapiInterrupt__(HwDeviceExtension, c as UCHAR) != 0 {
                    (*deviceExtension).LastInterruptedChannel = c as UCHAR;
                    status = TRUE;
                } else {
                    InterlockedExchange(&mut (*(*deviceExtension).chan.add(c as usize)).CheckIntr, CHECK_INTR_IDLE as LONG);
                }

                if (*deviceExtension).simplexOnly != 0 {
                    AtapiEnableInterrupts(deviceExtension as PVOID, (c == 0) as ULONG);
                }
            } else {
                InterlockedExchange(&mut (*(*deviceExtension).chan.add(c as usize)).CheckIntr, CHECK_INTR_IDLE as LONG);
            }
            _c += 1;
        }
        pass += 1;
    }
    if status != 0 {
        (*deviceExtension).FirstChannelToCheck += 1;
        if (*deviceExtension).FirstChannelToCheck >= (*deviceExtension).NumberChannels {
            (*deviceExtension).FirstChannelToCheck = 0;
        }
    }
    status
}

// ============================================================================
// AtapiInterrupt2  (#ifndef UNIATA_CORE) — secondary ISR
// ============================================================================

pub unsafe extern "system" fn AtapiInterrupt2(
    _Interrupt: PKINTERRUPT,
    Isr2HwDeviceExtension: PVOID,
) -> BOOLEAN {
    let Isr2DeviceExtension = Isr2HwDeviceExtension as PISR2_DEVICE_EXTENSION;
    let deviceExtension = (*Isr2DeviceExtension).HwDeviceExtension;
    let mut c: ULONG;
    let mut status: BOOLEAN = FALSE;
    let mut c_count: ULONG = 0;
    let mut i_res: ULONG;
    let hIS: ULONG;
    let mut checked: ULONG;

    // we should never get here for ISA/MCA
    if (*BMList.add((*deviceExtension).DevIndex as usize)).Isr2Enable == 0 {
        return FALSE;
    }

    if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
        hIS = UniataAhciReadHostPort4(deviceExtension, IDX_AHCI_IS as ULONG);
        if hIS == 0 {
            return FALSE;
        }
        checked = !hIS;
        checked |= !(*deviceExtension).AHCI_PI;
    } else {
        checked = 0;
    }
    if (*deviceExtension).ExpectingInterrupt == 0 {
        (*deviceExtension).ExpectingInterrupt += 1;
        return FALSE;
    }

    c = 0;
    while c < (*deviceExtension).NumberChannels {
        if (checked >> c) & 0x01 != 0 {
            c += 1;
            continue;
        }
        checked |= (1u32) << c;

        if CrNtInterlockedExchangeAdd(&mut (*(*deviceExtension).chan.add(c as usize)).DisableIntr, 0) != 0 {
            c += 1;
            continue;
        }

        if CrNtInterlockedCompareExchange(
            &mut (*(*deviceExtension).chan.add(c as usize)).CheckIntr,
            CHECK_INTR_CHECK as LONG, CHECK_INTR_IDLE as LONG,
        ) != CHECK_INTR_IDLE as LONG
        {
            c += 1;
            continue;
        }

        c_count += 1;
        i_res = AtapiCheckInterrupt__(deviceExtension as PVOID, c as UCHAR) as ULONG;
        if i_res != 0 {
            if i_res == INTERRUPT_REASON_UNEXPECTED {
                InterlockedExchange(&mut (*(*deviceExtension).chan.add(c as usize)).CheckIntr, CHECK_INTR_IDLE as LONG);
                return TRUE;
            }
            status = TRUE;
            InterlockedExchange(&mut (*(*deviceExtension).chan.add(c as usize)).CheckIntr, CHECK_INTR_DETECTED as LONG);
        } else {
            InterlockedExchange(&mut (*(*deviceExtension).chan.add(c as usize)).CheckIntr, CHECK_INTR_IDLE as LONG);
        }
        c += 1;
    }
    if status != 0 && c_count != (*deviceExtension).NumberChannels {
        InterlockedExchange(&mut (*deviceExtension).ReCheckIntr, CHECK_INTR_DETECTED as LONG);
    } else {
        status = FALSE;
    }
    status
}

// RETTYPE_XXableInterrupts == VOID (UNIATA_USE_XXableInterrupts not defined)
pub unsafe extern "system" fn AtapiInterruptDpc(HwDeviceExtension: PVOID) {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let mut c: ULONG;

    c = 0;
    while c < (*deviceExtension).NumberChannels {
        if (*(*deviceExtension).chan.add(c as usize)).ChannelCtrlFlags & CTRFLAGS_DPC_REQ == 0 {
            if CrNtInterlockedCompareExchange(
                &mut (*(*deviceExtension).chan.add(c as usize)).CheckIntr,
                CHECK_INTR_ACTIVE as LONG, CHECK_INTR_DETECTED as LONG,
            ) != CHECK_INTR_DETECTED as LONG
            {
                c += 1;
                continue;
            }
        } else {
            (*(*deviceExtension).chan.add(c as usize)).ChannelCtrlFlags &= !CTRFLAGS_DPC_REQ;
        }
        (*(*deviceExtension).chan.add(c as usize)).DpcState = DPC_STATE_DPC as UCHAR;
        if AtapiInterrupt__(HwDeviceExtension, c as UCHAR) == 0 {
            InterlockedExchange(&mut (*(*deviceExtension).chan.add(c as usize)).CheckIntr, CHECK_INTR_IDLE as LONG);
        }
        c += 1;
    }
}

// RETTYPE_XXableInterrupts == VOID
pub unsafe extern "system" fn AtapiEnableInterrupts__(HwDeviceExtension: PVOID) {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let mut c: ULONG;
    let mut chan: PHW_CHANNEL = core::ptr::null_mut();

    c = 0;
    while c < (*deviceExtension).NumberChannels {
        chan = (*deviceExtension).chan.add(c as usize);

        if (*chan).ChannelCtrlFlags & CTRFLAGS_ENABLE_INTR_REQ != 0 {
            (*chan).ChannelCtrlFlags &= !CTRFLAGS_ENABLE_INTR_REQ;
            AtapiEnableInterrupts(HwDeviceExtension, c);
            InterlockedExchange(&mut (*chan).CheckIntr, CHECK_INTR_IDLE as LONG);

            if (*deviceExtension).simplexOnly != 0 {
                break;
            }
        } else {
            if CrNtInterlockedCompareExchange(
                &mut (*chan).CheckIntr,
                CHECK_INTR_ACTIVE as LONG, CHECK_INTR_DETECTED as LONG,
            ) != CHECK_INTR_DETECTED as LONG
            {
                c += 1;
                continue;
            }
            (*chan).DpcState = DPC_STATE_ISR as UCHAR;
            if AtapiInterrupt__(HwDeviceExtension, c as UCHAR) == 0 {
                InterlockedExchange(&mut (*chan).CheckIntr, CHECK_INTR_IDLE as LONG);
            }
        }
        c += 1;
    }
    // In simplex mode next command must be sent to device here
    if (*deviceExtension).simplexOnly != 0 && !chan.is_null() {
        let srb: PSCSI_REQUEST_BLOCK;
        chan = UniataGetNextChannel(chan);
        if !chan.is_null() {
            srb = UniataGetCurRequest(chan);
        } else {
            srb = core::ptr::null_mut();
        }
        if !srb.is_null() {
            AtapiStartIo__(HwDeviceExtension, srb, FALSE);
        }
    }
}

pub unsafe extern "system" fn AtapiEnableInterrupts(HwDeviceExtension: PVOID, c: ULONG) {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL;

    if c >= (*deviceExtension).NumberChannels {
        return;
    }
    if (*deviceExtension).HwFlags & UNIATA_AHCI != 0
        && UniataAhciChanImplemented(deviceExtension, c) == 0
    {
        return;
    }

    chan = (*deviceExtension).chan.add(c as usize);
    if InterlockedDecrement(&mut (*chan).DisableIntr) == 0 {
        if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
            UniataAhciWriteChannelPort4(
                chan, IDX_AHCI_P_IE as ULONG,
                ATA_AHCI_P_IX_CPD | ATA_AHCI_P_IX_TFE | ATA_AHCI_P_IX_HBF
                    | ATA_AHCI_P_IX_HBD | ATA_AHCI_P_IX_INF | ATA_AHCI_P_IX_IF | ATA_AHCI_P_IX_OF
                    | ATA_AHCI_P_IX_PRC | ATA_AHCI_P_IX_PC
                    | ATA_AHCI_P_IX_PRC | ATA_AHCI_P_IX_PC
                    | ATA_AHCI_P_IX_DI
                    | ATA_AHCI_P_IX_DP | ATA_AHCI_P_IX_UF | ATA_AHCI_P_IX_SDB
                    | ATA_AHCI_P_IX_DS | ATA_AHCI_P_IX_PS | ATA_AHCI_P_IX_DHR,
            );
        } else {
            AtapiWritePort1(chan, IDX_IO2_o_Control, (0 | IDE_DC_A_4BIT) as UCHAR);
        }
        (*chan).ChannelCtrlFlags &= !CTRFLAGS_INTR_DISABLED;
    } else {
        if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
            UniataAhciWriteChannelPort4(chan, IDX_AHCI_P_IE as ULONG, 0);
        } else {
            AtapiWritePort1(chan, IDX_IO2_o_Control, IDE_DC_DISABLE_INTERRUPTS as UCHAR);
        }
    }
}

pub unsafe extern "system" fn AtapiDisableInterrupts(HwDeviceExtension: PVOID, c: ULONG) {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL;
    if c >= (*deviceExtension).NumberChannels {
        return;
    }
    chan = (*deviceExtension).chan.add(c as usize);
    // mark channel as busy
    if InterlockedIncrement(&mut (*chan).DisableIntr) != 0 {
        if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
            UniataAhciWriteChannelPort4(chan, IDX_AHCI_P_IE as ULONG, 0);
        } else {
            AtapiWritePort1(chan, IDX_IO2_o_Control, IDE_DC_DISABLE_INTERRUPTS as UCHAR);
        }
        (*chan).ChannelCtrlFlags |= CTRFLAGS_INTR_DISABLED;
    }
}

pub unsafe extern "system" fn UniataExpectChannelInterrupt(
    chan: *mut _HW_CHANNEL,
    Expecting: BOOLEAN,
) {
    (*chan).ExpectingInterrupt = Expecting;
    if Expecting != 0 {
        (*(*chan).DeviceExtension).ExpectingInterrupt += 1;
    } else if (*(*chan).DeviceExtension).ExpectingInterrupt != 0 {
        (*(*chan).DeviceExtension).ExpectingInterrupt -= 1;
    }
}

// ============================================================================
// AtapiCheckInterrupt__ — check hardware for interrupt state
// ============================================================================

pub unsafe extern "system" fn AtapiCheckInterrupt__(
    HwDeviceExtension: PVOID,
    c: UCHAR, // logical channel
) -> BOOLEAN {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(c as usize);
    let LunExt: PHW_LU_EXTENSION;

    let VendorID: ULONG = (*deviceExtension).DevID & 0xffff;
    let ChipType: ULONG = (*deviceExtension).HwFlags & CHIPTYPE_MASK;

    let mut status: ULONG;
    let mut pr_status: ULONG;
    let mut dma_status: UCHAR = 0;
    let mut reg8: UCHAR = 0;
    let mut reg32: ULONG;
    let mut statusByte: UCHAR = 0;
    let slotNumber: ULONG = (*deviceExtension).slotNumber;
    let SystemIoBusNumber: ULONG = (*deviceExtension).SystemIoBusNumber;
    let ChipFlags: ULONG = (*deviceExtension).HwFlags & CHIPFLAG_MASK;
    let Channel: UCHAR;
    let lChannel: UCHAR;
    let mut DmaTransfer: BOOLEAN = FALSE;
    let mut OurInterrupt: BOOLEAN = FALSE;
    let mut StatusValid: BOOLEAN = FALSE;
    let mut interruptReason: UCHAR;
    let mut EarlyIntr: BOOLEAN = FALSE;
    let mut SingleBlockIntr: BOOLEAN = FALSE;

    lChannel = c;
    Channel = ((*deviceExtension).Channel + lChannel as ULONG) as UCHAR;
    LunExt = (*chan).lun[(*chan).cur_cdev as usize];

    if (ChipFlags & UNIATA_AHCI) != 0 && UniataIsSATARangeAvailable(deviceExtension, lChannel as ULONG) != 0 {
        if UniataAhciChanImplemented(deviceExtension, lChannel as ULONG) == 0 {
            return OurInterrupt;
        }
        OurInterrupt = UniataAhciStatus(HwDeviceExtension, lChannel as ULONG, DEVNUM_NOT_SPECIFIED);
        if OurInterrupt as u32 == INTERRUPT_REASON_UNEXPECTED
            && (*LunExt).DeviceFlags & DFLAGS_ATAPI_DEVICE != 0
        {
            UniataAhciWaitCommandReady(chan, 2 /* ms */);
            statusByte = UniataAhciWaitReady(chan, 0 /* immediate */) as UCHAR;
            if statusByte as u32 & IDE_STATUS_BUSY == 0 {
                OurInterrupt = INTERRUPT_REASON_OUR as BOOLEAN;
            } else if (statusByte as u32 & (IDE_STATUS_BUSY | IDE_STATUS_DRDY))
                == (IDE_STATUS_BUSY | IDE_STATUS_DRDY)
            {
                OurInterrupt = INTERRUPT_REASON_OUR as BOOLEAN;
            } else if statusByte as u32 & IDE_STATUS_ERROR != 0 {
                OurInterrupt = INTERRUPT_REASON_OUR as BOOLEAN;
            } else {
                OurInterrupt = INTERRUPT_REASON_OUR as BOOLEAN;
            }
        }
        return OurInterrupt;
    }

    if (*chan).ChannelCtrlFlags & CTRFLAGS_DMA_ACTIVE != 0 {
        DmaTransfer = TRUE;
    } else {
        dma_status = GetDmaStatus(deviceExtension, lChannel as ULONG);
    }

    // 'gen_check (check_unknown:) and 'skip_dma (skip_dma_stat_check:) modeled as labeled
    // blocks. The vendor switch jumps via these. Early returns become `return`.
    let mut skip_generic = false; // set when arm does `goto skip_dma_stat_check`

    'gen_check: {
        // do controller-specific interrupt servicing
        if (*deviceExtension).UnknownDev != 0 {
            break 'gen_check; // goto check_unknown
        }

        match VendorID {
            x if x == ATA_PROMISE_ID => {
                match ChipType {
                    t if t == PROLD || t == PRNEW => {
                        status = AtapiReadPortEx4(chan, (&(*deviceExtension).BaseIoAddressBM_0) as *const _ as ULONG_PTR, 0x1c);
                        if DmaTransfer == 0 {
                            // break out of inner switch only
                        } else if status & (if Channel != 0 { 0x00004000 } else { 0x00000400 }) == 0 {
                            return INTERRUPT_REASON_IGNORE as BOOLEAN;
                        }
                    }
                    t if t == PRTX => {
                        AtapiWritePort1(chan, IDX_BM_DeviceSpecific0, 0x0b);
                        status = AtapiReadPort1(chan, IDX_BM_DeviceSpecific1) as ULONG;
                        if DmaTransfer == 0 {
                        } else if status & 0x20 == 0 {
                            return INTERRUPT_REASON_IGNORE as BOOLEAN;
                        }
                    }
                    t if t == PRMIO => {
                        let stat_reg: ULONG = if ChipFlags & PRG2 != 0 { 0x60 } else { 0x6c };
                        status = AtapiReadPortEx4(chan, (&(*deviceExtension).BaseIoAddressBM_0) as *const _ as ULONG_PTR, 0x40);
                        AtapiWritePortEx4(chan, (&(*deviceExtension).BaseIoAddressBM_0) as *const _ as ULONG_PTR, 0x40, status);

                        if status & (1 << (Channel + 1)) != 0 {
                            // our
                        } else {
                            return INTERRUPT_REASON_IGNORE as BOOLEAN;
                        }

                        if ChipFlags & UNIATA_SATA != 0 {
                            pr_status = AtapiReadPortEx4(chan, (&(*deviceExtension).BaseIoAddressBM_0) as *const _ as ULONG_PTR, stat_reg);
                            AtapiWritePortEx4(chan, (&(*deviceExtension).BaseIoAddressBM_0) as *const _ as ULONG_PTR, stat_reg, pr_status & (0x11 << Channel));
                            if pr_status & (0x11 << Channel) != 0 {
                                UniataSataEvent(deviceExtension as PVOID, lChannel as ULONG, UNIATA_SATA_EVENT_DETACH as ULONG, 0);
                            }
                            if status & (0x01 << Channel) == 0 {
                                UniataSataEvent(deviceExtension as PVOID, lChannel as ULONG, UNIATA_SATA_EVENT_ATTACH as ULONG, 0);
                            }
                            if UniataSataClearErr(HwDeviceExtension, c as ULONG, UNIATA_SATA_DO_CONNECT as BOOLEAN, 0) != 0 {
                                OurInterrupt = INTERRUPT_REASON_UNEXPECTED as BOOLEAN;
                            } else {
                                return INTERRUPT_REASON_IGNORE as BOOLEAN;
                            }
                            AtapiWritePort4(chan, IDX_BM_DeviceSpecific0, 0x00000001);
                        }
                    }
                    _ => {}
                }
            }
            x if x == ATA_NVIDIA_ID => {
                if (ChipFlags & UNIATA_SATA) == 0 || (ChipFlags & NVGEN) != 0 {
                    // break
                } else {
                    let offs: ULONG = if ChipFlags & NV4OFF != 0 { 0x0440 } else { 0x0010 };
                    let shift: ULONG = (Channel as ULONG) * (if ChipFlags & NVQ != 0 { 4 } else { 16 });

                    if ChipFlags & NVQ != 0 {
                        pr_status = AtapiReadPortEx4(chan, (&(*deviceExtension).BaseIoAddressSATA_0) as *const _ as ULONG_PTR, offs);
                        AtapiWritePortEx4(chan, (&(*deviceExtension).BaseIoAddressSATA_0) as *const _ as ULONG_PTR, offs, (0x0fu32 << shift) | 0x00f000f0);
                    } else {
                        pr_status = AtapiReadPortEx1(chan, (&(*deviceExtension).BaseIoAddressSATA_0) as *const _ as ULONG_PTR, offs) as ULONG;
                        AtapiWritePortEx1(chan, (&(*deviceExtension).BaseIoAddressSATA_0) as *const _ as ULONG_PTR, offs, (0x0f << shift) as UCHAR);
                    }

                    if (pr_status & (0x0cu32 << shift)) == (0x04u32 << shift) {
                        UniataSataEvent(deviceExtension as PVOID, lChannel as ULONG, UNIATA_SATA_EVENT_ATTACH as ULONG, 0);
                    }
                    if (pr_status & (0x08u32 << shift)) != 0
                        && !((pr_status & (0x04u32 << shift)) != 0
                            && UniataSataReadPort4(chan, IDX_SATA_SStatus as ULONG, 0) != 0)
                    {
                        UniataSataEvent(deviceExtension as PVOID, lChannel as ULONG, UNIATA_SATA_EVENT_DETACH as ULONG, 0);
                    }
                    if pr_status & (0x01u32 << shift) == 0 {
                        if UniataSataClearErr(HwDeviceExtension, c as ULONG, UNIATA_SATA_DO_CONNECT as BOOLEAN, 0) != 0 {
                            OurInterrupt = INTERRUPT_REASON_UNEXPECTED as BOOLEAN;
                        } else {
                            return INTERRUPT_REASON_IGNORE as BOOLEAN;
                        }
                    }
                }
            }
            x if x == ATA_ATI_ID || x == ATA_SILICON_IMAGE_ID => {
                // ATI falls into SiI only when ChipType==SIIMIO; otherwise break.
                if VendorID == ATA_ATI_ID && ChipType != SIIMIO {
                    // break
                } else if ChipType == SIIMIO {
                    reg32 = AtapiReadPort4(chan, IDX_BM_DeviceSpecific0);
                    if reg32 == 0xffffffff {
                        return INTERRUPT_REASON_IGNORE as BOOLEAN;
                    }
                    if reg32 & (BM_DS0_SII_DMA_SATA_IRQ | BM_DS0_SII_DMA_COMPLETE | BM_DS0_SII_IRQ | BM_DS0_SII_DMA_ENABLE | BM_DS0_SII_DMA_ERROR) == 0 {
                        return INTERRUPT_REASON_IGNORE as BOOLEAN;
                    }
                    if ChipFlags & UNIATA_SATA != 0 {
                        if reg32 & (BM_DS0_SII_DMA_SATA_IRQ | BM_DS0_SII_IRQ) != 0 {
                            if UniataSataClearErr(HwDeviceExtension, c as ULONG, UNIATA_SATA_DO_CONNECT as BOOLEAN, 0) != 0 {
                                OurInterrupt = INTERRUPT_REASON_UNEXPECTED as BOOLEAN;
                            }
                        }
                    }
                    if DmaTransfer == 0 {
                        // break
                    } else {
                        dma_status = GetDmaStatus(deviceExtension, lChannel as ULONG);
                        if dma_status as u32 & BM_STATUS_INTR == 0 {
                            return OurInterrupt;
                        }
                        AtapiWritePort1(chan, IDX_BM_Status, dma_status & !(BM_STATUS_ERR as UCHAR));
                        skip_generic = true; // goto skip_dma_stat_check
                    }
                } else {
                    // SiI non-MIO path
                    if (*deviceExtension).HwFlags & SIIINTR == 0 {
                        // break
                    } else {
                        // GetPciConfig1(0x71, reg8)
                        ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration as i32, SystemIoBusNumber, slotNumber, &mut reg8 as *mut _ as PVOID, 0x71, 1);
                        if reg8 & (if Channel != 0 { 0x08 } else { 0x04 }) == 0 {
                            return INTERRUPT_REASON_IGNORE as BOOLEAN;
                        }
                        if DmaTransfer == 0 {
                            OurInterrupt = INTERRUPT_REASON_UNEXPECTED as BOOLEAN;
                        }
                        // SetPciConfig1(0x71, ...)
                        let mut a: UCHAR = if Channel != 0 { 0x08 } else { 0x04 };
                        ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration, SystemIoBusNumber, slotNumber, &mut a as *mut _ as PVOID, 0x71, 1);
                    }
                }
            }
            x if x == ATA_ACARD_ID => {
                if DmaTransfer == 0 {
                    // break
                } else {
                    dma_status = GetDmaStatus(deviceExtension, lChannel as ULONG);
                    if dma_status as u32 & BM_STATUS_INTR == 0 {
                        return INTERRUPT_REASON_IGNORE as BOOLEAN;
                    }
                    AtapiWritePort1(chan, IDX_BM_Status, dma_status | BM_STATUS_INTR as UCHAR);
                    ScsiPortStallExecution(1);
                    AtapiWritePort1(chan, IDX_BM_Command,
                        AtapiReadPort1(chan, IDX_BM_Command) & !(BM_COMMAND_START_STOP as UCHAR));
                    skip_generic = true; // goto skip_dma_stat_check
                }
            }
            x if x == ATA_INTEL_ID => {
                if UniataIsSATARangeAvailable(deviceExtension, lChannel as ULONG) != 0 {
                    if ChipFlags & UNIATA_AHCI != 0 {
                    } else if ChipFlags & UNIATA_SATA != 0 {
                        if UniataSataClearErr(HwDeviceExtension, c as ULONG, UNIATA_SATA_DO_CONNECT as BOOLEAN, 0) != 0 {
                            OurInterrupt = INTERRUPT_REASON_UNEXPECTED as BOOLEAN;
                        }
                        if (*chan).ChannelCtrlFlags & CTRFLAGS_NO_SLAVE == 0 {
                            if UniataSataClearErr((*chan).DeviceExtension as PVOID, (*chan).lChannel, UNIATA_SATA_IGNORE_CONNECT as BOOLEAN, 1) != 0 {
                                OurInterrupt = INTERRUPT_REASON_UNEXPECTED as BOOLEAN;
                            }
                        }
                    }
                }
            }
            _ => {
                if UniataIsSATARangeAvailable(deviceExtension, lChannel as ULONG) != 0 {
                    if ChipFlags & UNIATA_AHCI != 0 {
                    } else if ChipFlags & UNIATA_SATA != 0 {
                        if UniataSataClearErr(HwDeviceExtension, c as ULONG, UNIATA_SATA_DO_CONNECT as BOOLEAN, 0) != 0 {
                            OurInterrupt = INTERRUPT_REASON_UNEXPECTED as BOOLEAN;
                        }
                    }
                }
            }
        }
    } // 'gen_check end == arrival of check_unknown: label

    if !skip_generic {
        // check_unknown: (generic check)
        if DmaTransfer != 0 {
            dma_status = GetDmaStatus(deviceExtension, lChannel as ULONG);
            if dma_status as u32 & BM_STATUS_INTR == 0 {
                if dma_status as u32 & BM_STATUS_ERR != 0 {
                    OurInterrupt = INTERRUPT_REASON_UNEXPECTED as BOOLEAN;
                } else {
                    statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus); // GetStatus
                    StatusValid = 1;
                    if statusByte as u32 & IDE_STATUS_ERROR != 0 {
                        OurInterrupt = INTERRUPT_REASON_UNEXPECTED as BOOLEAN;
                    } else if (statusByte as u32 & IDE_STATUS_DSC) != 0
                        && ((*LunExt).DeviceFlags & DFLAGS_ATAPI_DEVICE) != 0
                        && (dma_status as u32 == BM_STATUS_ACTIVE)
                    {
                        if (*LunExt).TransferMode as u32 >= ATA_SDMA
                            && (*LunExt).TransferMode as u32 <= ATA_WDMA2
                        {
                            SingleBlockIntr = TRUE;
                        }
                    } else {
                        return INTERRUPT_REASON_IGNORE as BOOLEAN;
                    }
                }
            }
        } else {
            if dma_status as u32 & BM_STATUS_INTR != 0 {
                AtapiDmaDone(deviceExtension as PVOID, DEVNUM_NOT_SPECIFIED, lChannel as ULONG, core::ptr::null_mut());
                OurInterrupt = INTERRUPT_REASON_UNEXPECTED as BOOLEAN;
            }
        }
    }

    // skip_dma_stat_check:
    if (ChipFlags & UNIATA_SATA) == 0 && (*chan).ExpectingInterrupt != 0 {
        ScsiPortStallExecution(1);
    }

    // if drive is busy it didn't interrupt
    if StatusValid == 0 {
        statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus); // GetStatus
    }
    if statusByte as u32 == IDE_STATUS_WRONG {
        // interrupt from empty controller ?
    } else if statusByte as u32 & IDE_STATUS_BUSY != 0 {
        if (*chan).ExpectingInterrupt == 0 {
            return OurInterrupt;
        }
        if (*LunExt).DeviceFlags & DFLAGS_ATAPI_DEVICE != 0 {
            // ATAPI additional check
        } else {
            return INTERRUPT_REASON_IGNORE as BOOLEAN;
        }
        if (statusByte as u32 & !(IDE_STATUS_DRQ | IDE_STATUS_INDEX))
            != (IDE_STATUS_BUSY | IDE_STATUS_DRDY | IDE_STATUS_DSC)
        {
            return INTERRUPT_REASON_IGNORE as BOOLEAN;
        }
        if (*LunExt).DeviceFlags & DFLAGS_INT_DRQ == 0 && (statusByte as u32 & IDE_STATUS_DRQ) != 0 {
            return INTERRUPT_REASON_IGNORE as BOOLEAN;
        }

        EarlyIntr = TRUE;

        if dma_status as u32 & BM_STATUS_INTR != 0 {
            statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
            if dma_status as u32 & BM_STATUS_ACTIVE == 0 {
                AtapiDmaDone(deviceExtension as PVOID, DEVNUM_NOT_SPECIFIED, lChannel as ULONG, core::ptr::null_mut());
            }
            return INTERRUPT_REASON_OUR as BOOLEAN;
        }

        if g_WaitBusyInISR != 0 {
            statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus); // GetStatus
            reg8 = AtapiReadPort1(chan, IDX_IO1_i_Error);
            let _ = reg8;
            if statusByte as u32 & IDE_STATUS_BUSY != 0 {
                return INTERRUPT_REASON_IGNORE as BOOLEAN;
            }
        }
    }

    // clear interrupt and get status
    statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
    if statusByte as u32 == IDE_STATUS_WRONG {
        // interrupt from empty controller ?
    } else if statusByte as u32 & (IDE_STATUS_DRQ | IDE_STATUS_DRDY) == 0 {
        return OurInterrupt;
    }

    // #ifndef UNIATA_PIO_ONLY
    if DmaTransfer != 0 {
        if SingleBlockIntr == 0 && (EarlyIntr == 0 || g_WaitBusyInISR != 0) {
            dma_status = AtapiDmaDone(HwDeviceExtension, DEVNUM_NOT_SPECIFIED, lChannel as ULONG, core::ptr::null_mut());
        } else {
            let srb: PSCSI_REQUEST_BLOCK = UniataGetCurRequest(chan);
            let AtaReq: PATA_REQ = if !srb.is_null() { (*srb).SrbExtension as PATA_REQ } else { core::ptr::null_mut() };
            if !AtaReq.is_null() {
                (*ata_req_core(AtaReq)).ReqState = if SingleBlockIntr != 0 {
                    REQ_STATE_ATAPI_EXPECTING_DATA_INTR2 as UCHAR
                } else {
                    REQ_STATE_EARLY_INTR as UCHAR
                };
            }
        }
    }

    if (*chan).ExpectingInterrupt == 0 {
        if (*LunExt).DeviceFlags & DFLAGS_ATAPI_DEVICE != 0 {
            // ATAPI additional check
        } else {
            return OurInterrupt;
        }
        interruptReason = AtapiReadPort1(chan, IDX_ATAPI_IO1_i_InterruptReason) & ATAPI_IR_Mask as UCHAR;
        let _ = interruptReason;
        return OurInterrupt;
    }

    INTERRUPT_REASON_OUR as BOOLEAN
}

// ============================================================================
// AtapiInterrupt__ — the interrupt service worker.
//   Live config: UNIATA_USE_XXableInterrupts OFF, UNIATA_CORE OFF, UNIATA_PIO_ONLY OFF,
//   UNIATA_INIT_CHANGERS OFF, UNIATA_DUMP_ATAPI OFF, IO_STATISTICS ON.
//   The dense goto graph is restructured into a `'sm` state machine: each C label is a
//   `Pc` variant; `goto L` becomes `pc = Pc::L; continue 'sm`. InDpc = (DpcState != ISR).
// ============================================================================

#[derive(Clone, Copy, PartialEq)]
enum Pc {
    ServiceInterrupt,
    PostToDpc,
    CallTimerDpc,
    CallTimerDpc2,
    TryDpcWait,
    ContinueErr,
    ContinuePio,
    ContinueReadDrq,
    PioWaitDrq0,
    CompleteRequest,
    CompleteRdp,
    IntrCompleteReq,
    PioWaitBusy,
    PioWaitDrq,
    IntrPrepareResetController,
    EnqueueNextReq,
    ReenqueueReq,
    ReturnEnableIntr,
    Done,
}

pub unsafe extern "system" fn AtapiInterrupt__(
    HwDeviceExtension: PVOID,
    c: UCHAR,
) -> BOOLEAN {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let mut chan: PHW_CHANNEL = (*deviceExtension).chan.add(c as usize);
    let mut srb: PSCSI_REQUEST_BLOCK = UniataGetCurRequest(chan);
    let mut AtaReq: PATA_REQ = if !srb.is_null() { (*srb).SrbExtension as PATA_REQ } else { core::ptr::null_mut() };

    let mut wordCount: ULONG = 0;
    let mut wordsThisInterrupt: ULONG = DEV_BSIZE / 2;
    let mut status: ULONG = SRB_STATUS_SUCCESS;
    let mut dma_status: UCHAR = 0;
    let mut i: ULONG;
    let mut k: ULONG;
    let mut statusByte: UCHAR = 0;
    let mut interruptReason: UCHAR = 0;

    let mut atapiDev: BOOLEAN = FALSE;
    let lChannel: UCHAR;
    let mut DeviceNumber: UCHAR = 0;
    let mut DmaTransfer: BOOLEAN = FALSE;
    let mut error: UCHAR = 0;
    let mut TimerValue: ULONG = 1000;
    let mut TotalTimerValue: ULONG = 0;
    let mut InDpc: BOOLEAN = ((*chan).DpcState != DPC_STATE_ISR as UCHAR) as BOOLEAN;
    let mut UseDpc: BOOLEAN = (*deviceExtension).UseDpc;
    let mut DataOverrun: BOOLEAN = FALSE;
    let mut NoStartIo: BOOLEAN = TRUE;
    let mut NoRetry: BOOLEAN = FALSE;

    let mut PathId: UCHAR = 0;
    let mut TargetId: UCHAR = 0;
    let mut Lun: UCHAR = 0;
    let mut OldReqState: UCHAR = REQ_STATE_NONE as UCHAR;
    let mut LunExt: PHW_LU_EXTENSION = core::ptr::null_mut();

    lChannel = c;

    if (*chan).ChannelCtrlFlags & CTRFLAGS_DMA_ACTIVE != 0
        || (!AtaReq.is_null() && (*ata_req_core(AtaReq)).Flags as u32 & REQ_FLAG_DMA_OPERATION != 0)
        || (*deviceExtension).HwFlags & UNIATA_AHCI != 0
    {
        DmaTransfer = TRUE;
    }

    // entry control-flow (pre-state-machine straight-line section)
    let mut first_pc: Pc = Pc::ServiceInterrupt;
    'entry: {
        if !srb.is_null() {
            PathId = (*srb).PathId;
            TargetId = (*srb).TargetId;
            Lun = (*srb).Lun;
        } else {
            PathId = c;
            TargetId = 0;
            Lun = 0;
            first_pc = Pc::EnqueueNextReq;
            break 'entry;
        }

        DeviceNumber = TargetId;
        LunExt = (*chan).lun[DeviceNumber as usize];
        atapiDev = ((*LunExt).DeviceFlags & DFLAGS_ATAPI_DEVICE != 0) as BOOLEAN;

        if InDpc != 0 {
            first_pc = Pc::ServiceInterrupt;
            break 'entry;
        }

        if DmaTransfer != 0 {
            dma_status = GetDmaStatus(deviceExtension, lChannel as ULONG);
        }

        if (*chan).ExpectingInterrupt == 0 {
            return FALSE;
        }

        // change request state
        if !AtaReq.is_null() {
            OldReqState = (*ata_req_core(AtaReq)).ReqState;
            (*ata_req_core(AtaReq)).ReqState = REQ_STATE_PROCESSING_INTR as UCHAR;
        }

        if DmaTransfer != 0 {
            first_pc = Pc::ServiceInterrupt;
            break 'entry;
        }

        match OldReqState as u32 {
            REQ_STATE_ATAPI_EXPECTING_CMD_INTR
            | REQ_STATE_ATAPI_EXPECTING_DATA_INTR
            | REQ_STATE_ATAPI_EXPECTING_DATA_INTR2
            | REQ_STATE_DPC_WAIT_BUSY0
            | REQ_STATE_DPC_WAIT_BUSY1 => {
                first_pc = Pc::ServiceInterrupt;
                break 'entry;
            }
            REQ_STATE_ATAPI_DO_NOTHING_INTR => {
                return TRUE;
            }
            _ => {}
        }

        if (DmaTransfer == 0 && atapiDev == 0) || (*deviceExtension).DriverMustPoll != 0 {
            UseDpc = FALSE;
        }

        if UseDpc == 0 {
            first_pc = Pc::ServiceInterrupt;
            break 'entry;
        }
        // UNIATA_USE_XXableInterrupts not defined -> goto ServiceInterrupt
        first_pc = Pc::ServiceInterrupt;
    }

    let mut pc = first_pc;
    'sm: loop {
        match pc {
            Pc::ServiceInterrupt => {
                if !AtaReq.is_null() && InDpc != 0 {
                    match (*ata_req_core(AtaReq)).ReqState as u32 {
                        REQ_STATE_DPC_WAIT_DRQ0 => { pc = Pc::PioWaitDrq0; continue 'sm; }
                        REQ_STATE_DPC_WAIT_BUSY => { pc = Pc::PioWaitBusy; continue 'sm; }
                        REQ_STATE_DPC_WAIT_DRQ => { pc = Pc::PioWaitDrq; continue 'sm; }
                        REQ_STATE_DPC_WAIT_DRQ_ERR => { pc = Pc::ContinueErr; continue 'sm; }
                        REQ_STATE_DPC_WAIT_BUSY0 | REQ_STATE_DPC_WAIT_BUSY1 => {}
                        _ => {}
                    }
                }

                // clear interrupt and get status
                if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
                    UniataAhciEndTransaction(HwDeviceExtension, lChannel as ULONG, DeviceNumber as ULONG, srb);
                    statusByte = ((*ata_req_ahci(AtaReq)).in_status & IDE_STATUS_MASK) as UCHAR;

                    if (*chan).AhciLastIS
                        & !(ATA_AHCI_P_IX_DHR | ATA_AHCI_P_IX_PS | ATA_AHCI_P_IX_DS | ATA_AHCI_P_IX_SDB)
                        != 0
                    {
                        if (*chan).AhciLastIS & !ATA_AHCI_P_IX_OF != 0 {
                            if (*chan).AhciLastIS == ATA_AHCI_P_IX_INF
                                && statusByte as u32 & IDE_STATUS_ERROR == 0
                                && (*chan).AhciLastSError == 0
                                && !srb.is_null()
                                && (*srb).SrbFlags & SRB_FLAGS_DATA_IN != 0
                            {
                                // assume underflow, continue
                            } else {
                                statusByte |= IDE_STATUS_ERROR as UCHAR;
                                (*ata_req_ahci(AtaReq)).in_serror = (*chan).AhciLastSError;
                                if (*chan).AhciLastSError
                                    & (ATA_SE_HANDSHAKE_ERR | ATA_SE_LINKSEQ_ERR | ATA_SE_TRANSPORT_ERR | ATA_SE_UNKNOWN_FIS)
                                    != 0
                                {
                                    NoRetry = TRUE;
                                }
                            }
                        } else {
                            if UniataAhciAbortOperation(chan) == 0 {
                                // need UniataAhciReset
                            }
                            UniataAhciWaitCommandReady(chan, 10);
                        }
                    }
                } else {
                    statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
                }

                if UseDpc == 0 {
                    InDpc = TRUE;
                }

                if atapiDev == 0 {
                    // IDE
                    if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
                    } else if statusByte as u32 & IDE_STATUS_BUSY != 0 {
                        if (*deviceExtension).DriverMustPoll != 0 {
                            pc = Pc::ReturnEnableIntr; continue 'sm;
                        }
                        pc = Pc::TryDpcWait; continue 'sm;
                    }
                } else {
                    // ATAPI
                    if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
                    } else {
                        interruptReason = AtapiReadPort1(chan, IDX_ATAPI_IO1_i_InterruptReason) & ATAPI_IR_Mask as UCHAR;
                    }

                    if statusByte as u32 & IDE_STATUS_BUSY != 0 {
                        if (*LunExt).AtapiReadyWaitDelay != 0
                            && (*LunExt).AtapiReadyWaitDelay > g_opt_MaxIsrWait
                            && InDpc == 0
                            && UseDpc != 0
                        {
                            TimerValue = (*LunExt).AtapiReadyWaitDelay;
                            (*ata_req_core(AtaReq)).ReqState = REQ_STATE_DPC_WAIT_BUSY0 as UCHAR;
                            pc = Pc::CallTimerDpc2; continue 'sm;
                        }
                        TimerValue = 10;
                        let mut to_dpc = false;
                        k = 20;
                        while k != 0 {
                            statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
                            if statusByte as u32 & IDE_STATUS_BUSY == 0 {
                                break;
                            }
                            TotalTimerValue += TimerValue;
                            if k <= 1 {
                                if InDpc == 0 {
                                    TimerValue = 100;
                                    (*ata_req_core(AtaReq)).ReqState = REQ_STATE_DPC_WAIT_BUSY0 as UCHAR;
                                } else {
                                    TimerValue = 1000;
                                    (*ata_req_core(AtaReq)).ReqState = REQ_STATE_DPC_WAIT_BUSY1 as UCHAR;
                                }
                                if UseDpc != 0 {
                                    if (*LunExt).AtapiReadyWaitDelay == 0 {
                                        (*LunExt).AtapiReadyWaitDelay = TotalTimerValue * 2 / 3;
                                    }
                                    to_dpc = true;
                                    break;
                                }
                            }
                            ScsiPortStallExecution(TimerValue);
                            TimerValue += 10;
                            k -= 1;
                        }
                        if to_dpc {
                            pc = Pc::CallTimerDpc2; continue 'sm;
                        }
                        if (*LunExt).AtapiReadyWaitDelay == 0 {
                            (*LunExt).AtapiReadyWaitDelay = TotalTimerValue * 2 / 3;
                        }
                        if statusByte as u32 & IDE_STATUS_BUSY != 0 {
                            pc = Pc::TryDpcWait; continue 'sm;
                        }
                    }
                }

                if !AtaReq.is_null() && DmaTransfer != 0 && (*deviceExtension).HwFlags & UNIATA_AHCI == 0 {
                    match OldReqState as u32 {
                        REQ_STATE_EARLY_INTR | REQ_STATE_DPC_WAIT_BUSY0 => {
                            if (*chan).ChannelCtrlFlags & CTRFLAGS_DMA_ACTIVE != 0 {
                                dma_status = AtapiDmaDone(HwDeviceExtension, DEVNUM_NOT_SPECIFIED, lChannel as ULONG, core::ptr::null_mut());
                            }
                        }
                        _ => {}
                    }
                }

                // Check for error conditions.
                if (statusByte as u32 & IDE_STATUS_ERROR != 0) || (dma_status as u32 & BM_STATUS_ERR != 0) {
                    if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
                        error = (*ata_req_ahci(AtaReq)).in_error;
                        if UniataAhciAbortOperation(chan) == 0 {}
                        UniataAhciWaitCommandReady(chan, 10);
                        UniataAhciStatus(HwDeviceExtension, lChannel as ULONG, DEVNUM_NOT_SPECIFIED);
                        if NoRetry != 0 {
                            (*ata_req_core(AtaReq)).retry += MAX_RETRIES as UCHAR;
                            if error == 0 && statusByte as u32 & IDE_STATUS_ERROR != 0 {
                                error |= IDE_STATUS_ERROR as UCHAR;
                            }
                        }
                    } else {
                        error = AtapiReadPort1(chan, IDX_IO1_i_Error);
                    }

                    if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
                    } else if atapiDev == 0 {
                        ScsiPortStallExecution(100);
                    } else {
                        ScsiPortStallExecution(10);
                    }
                    pc = Pc::ContinueErr; continue 'sm;
                } else if (*ata_req_core(AtaReq)).Flags as u32 & REQ_FLAG_FORCE_DOWNRATE_LBA48 != 0 {
                    (*deviceExtension).HbaCtrlFlags |= HBAFLAGS_DMA_DISABLED_LBA48;
                } else if (*ata_req_core(AtaReq)).Flags as u32 & REQ_FLAG_FORCE_DOWNRATE != 0 {
                    // IO_STATISTICS on
                    (*LunExt).RecoverCount[(*ata_req_core(AtaReq)).retry as usize] += 1;
                    if (*LunExt).RecoverCount[(*ata_req_core(AtaReq)).retry as usize] >= (*LunExt).BlockIoCount / 3
                        || ((*deviceExtension).HwFlags & UNIATA_NO80CHK) != 0
                    {
                        (*LunExt).LimitedTransferMode = (*LunExt).TransferMode;
                    }
                }
                // IO_STATISTICS
                if (*ata_req_core(AtaReq)).bcount != 0 {
                    (*LunExt).BlockIoCount += 1;
                }
                (*LunExt).IoCount += 1;

                pc = Pc::ContinuePio; continue 'sm;
            }

            Pc::ContinueErr => {
                for kk in (1..=(if atapiDev != 0 { 0 } else { 200u32 })).rev() {
                    let _ = kk;
                    statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
                    if statusByte as u32 & IDE_STATUS_DRQ == 0 {
                        ScsiPortStallExecution(50);
                    } else {
                        break;
                    }
                }

                if atapiDev == 0 {
                    (*ata_req_core(AtaReq)).retry += 1;
                    if ((*ata_req_core(AtaReq)).retry as u32) < MAX_RETRIES {
                        // IO_STATISTICS
                        (*LunExt).ModeErrorCount[(*ata_req_core(AtaReq)).retry as usize] += 1;
                        if DmaTransfer != 0 {
                            if ((*ata_req_core(AtaReq)).retry as u32) < MAX_RETRIES {
                                if (*deviceExtension).HwFlags & UNIATA_AHCI == 0 {
                                    (*ata_req_core(AtaReq)).Flags |= REQ_FLAG_FORCE_DOWNRATE as UCHAR;
                                }
                                (*ata_req_core(AtaReq)).ReqState = REQ_STATE_QUEUED as UCHAR;
                                pc = Pc::ReenqueueReq; continue 'sm;
                            }
                        } else {
                            if (*ata_req_core(AtaReq)).Flags as u32 & REQ_FLAG_FORCE_DOWNRATE == 0 {
                                (*ata_req_core(AtaReq)).retry += 1;
                            }
                        }
                    }
                } else {
                    interruptReason = AtapiReadPort1(chan, IDX_ATAPI_IO1_i_InterruptReason) & ATAPI_IR_Mask as UCHAR;

                    if UniataIsSATARangeAvailable(deviceExtension, lChannel as ULONG) != 0 {
                        if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
                        } else if (*deviceExtension).HwFlags & UNIATA_SATA != 0 {
                            UniataSataClearErr(HwDeviceExtension, lChannel as ULONG, UNIATA_SATA_IGNORE_CONNECT as BOOLEAN, 0);
                        }
                    }

                    if DmaTransfer != 0
                        && ((*(*chan).lun[DeviceNumber as usize]).TransferMode as u32 > ATA_UDMA2)
                        && ((error >> 4) as u32 == SCSI_SENSE_HARDWARE_ERROR)
                    {
                        if ((*ata_req_core(AtaReq)).retry as u32) < MAX_RETRIES {
                            (*ata_req_core(AtaReq)).Flags &= !(REQ_FLAG_DMA_OPERATION as UCHAR);
                            (*ata_req_core(AtaReq)).Flags |= REQ_FLAG_FORCE_DOWNRATE as UCHAR;
                            (*ata_req_core(AtaReq)).ReqState = REQ_STATE_QUEUED as UCHAR;
                            pc = Pc::ReenqueueReq; continue 'sm;
                        }
                    } else {
                        if (*ata_req_core(AtaReq)).Flags as u32 & REQ_FLAG_FORCE_DOWNRATE == 0 {
                            (*ata_req_core(AtaReq)).retry += 1;
                        }
                    }
                }

                if (*srb).Cdb[0] as u32 != SCSIOP_REQUEST_SENSE {
                    status = SRB_STATUS_ERROR;
                    pc = Pc::CompleteRequest; continue 'sm;
                }
                // else continue to continue_PIO
                pc = Pc::ContinuePio; continue 'sm;
            }

            Pc::ContinuePio => {
                if atapiDev != 0 {
                    interruptReason = AtapiReadPort1(chan, IDX_ATAPI_IO1_i_InterruptReason) & ATAPI_IR_Mask as UCHAR;
                    if DmaTransfer != 0 {
                        wordsThisInterrupt = DEV_BSIZE / 2 * 512;
                    } else {
                        wordsThisInterrupt = DEV_BSIZE / 2;
                    }
                } else {
                    if DmaTransfer != 0 {
                        statusByte |= IDE_STATUS_DRQ as UCHAR;
                    }
                    if statusByte as u32 & IDE_STATUS_DRQ != 0 {
                        if DmaTransfer != 0 {
                            wordsThisInterrupt = DEV_BSIZE / 2 * 512;
                        } else if (*LunExt).MaximumBlockXfer != 0 {
                            wordsThisInterrupt = DEV_BSIZE / 2 * (*LunExt).MaximumBlockXfer as ULONG;
                        }
                        if (*srb).SrbFlags & SRB_FLAGS_DATA_IN != 0 {
                            interruptReason = ATAPI_IR_IO_toHost as UCHAR;
                        } else if (*srb).SrbFlags & SRB_FLAGS_DATA_OUT != 0 {
                            interruptReason = ATAPI_IR_IO_toDev as UCHAR;
                        } else {
                            status = SRB_STATUS_ERROR;
                            pc = Pc::CompleteRequest; continue 'sm;
                        }
                    } else if statusByte as u32 & IDE_STATUS_BUSY != 0 {
                        return FALSE;
                    } else {
                        if (*ata_req_core(AtaReq)).WordsLeft != 0 {
                            pc = Pc::PioWaitDrq0; continue 'sm;
                        } else {
                            interruptReason = (ATAPI_IR_IO_toHost | ATAPI_IR_COD_Cmd) as UCHAR;
                        }
                    }
                }
                pc = interrupt_dispatch_reason(
                    deviceExtension, &mut chan, &mut srb, &mut AtaReq, lChannel, DeviceNumber,
                    atapiDev, DmaTransfer, &mut statusByte, &mut interruptReason,
                    &mut wordCount, wordsThisInterrupt, &mut status, &mut DataOverrun,
                    &mut error, OldReqState, LunExt,
                );
                continue 'sm;
            }

            Pc::ContinueReadDrq => {
                pc = interrupt_read_drq(
                    deviceExtension, chan, srb, AtaReq, lChannel, DeviceNumber, atapiDev,
                    DmaTransfer, &mut statusByte, &mut wordCount, wordsThisInterrupt,
                    &mut status, &mut DataOverrun, LunExt, HwDeviceExtension,
                );
                continue 'sm;
            }

            Pc::PioWaitDrq0 => {
                let mut reset = false;
                let mut to_dpc = false;
                k = 0;
                while k < 5000 {
                    statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
                    if statusByte as u32 & IDE_STATUS_DRQ != 0 {
                        break;
                    }
                    if InDpc == 0 {
                        (*ata_req_core(AtaReq)).ReqState = REQ_STATE_DPC_WAIT_DRQ0 as UCHAR;
                        TimerValue = 100;
                        to_dpc = true;
                        break;
                    }
                    ScsiPortStallExecution(100);
                    k += 1;
                }
                if to_dpc { pc = Pc::PostToDpc; continue 'sm; }
                if k == 5000 {
                    reset = true;
                }
                if reset {
                    pc = Pc::IntrPrepareResetController; continue 'sm;
                } else {
                    interruptReason = if (*srb).SrbFlags & SRB_FLAGS_DATA_IN != 0 {
                        ATAPI_IR_IO_toHost as UCHAR
                    } else {
                        ATAPI_IR_IO_toDev as UCHAR
                    };
                }
                pc = interrupt_dispatch_reason(
                    deviceExtension, &mut chan, &mut srb, &mut AtaReq, lChannel, DeviceNumber,
                    atapiDev, DmaTransfer, &mut statusByte, &mut interruptReason,
                    &mut wordCount, wordsThisInterrupt, &mut status, &mut DataOverrun,
                    &mut error, OldReqState, LunExt,
                );
                continue 'sm;
            }

            Pc::IntrPrepareResetController => {
                AtapiResetController__(HwDeviceExtension, lChannel as ULONG, RESET_COMPLETE_CURRENT);
                pc = Pc::ReturnEnableIntr; continue 'sm;
            }

            Pc::PostToDpc => {
                if OldReqState as u32 != REQ_STATE_DPC_WAIT_BUSY1 {
                    UniataExpectChannelInterrupt(chan, FALSE);
                    AtapiDisableInterrupts(deviceExtension as PVOID, lChannel as ULONG);
                }
                (*chan).ChannelCtrlFlags |= CTRFLAGS_DPC_REQ;
                AtapiQueueTimerDpc(HwDeviceExtension, c as ULONG, core::mem::transmute::<_, PHW_TIMER>(Some(AtapiInterruptDpc as unsafe extern "system" fn(PVOID))), TimerValue);
                return TRUE;
            }

            Pc::CallTimerDpc => {
                (*ata_req_core(AtaReq)).ReqState = REQ_STATE_PROCESSING_INTR as UCHAR;
                pc = Pc::CallTimerDpc2; continue 'sm;
            }

            Pc::CallTimerDpc2 => {
                if InDpc == 0 && OldReqState as u32 != REQ_STATE_DPC_WAIT_BUSY1 {
                    AtapiDisableInterrupts(deviceExtension as PVOID, lChannel as ULONG);
                }
                AtapiQueueTimerDpc(HwDeviceExtension, c as ULONG, core::mem::transmute::<_, PHW_TIMER>(Some(AtapiCallBack_X as unsafe extern "system" fn(PVOID))), TimerValue);
                return TRUE;
            }

            Pc::TryDpcWait => {
                k = if InDpc != 0 && UseDpc != 0 { 1000 } else { 2 };
                i = 0;
                while i < k {
                    statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
                    if statusByte as u32 & IDE_STATUS_BUSY == 0 {
                        break;
                    }
                    ScsiPortStallExecution(10);
                    i += 1;
                }
                if InDpc == 0 && UseDpc != 0 && i == 2 {
                    TimerValue = 50;
                    (*ata_req_core(AtaReq)).ReqState = REQ_STATE_DPC_WAIT_BUSY0 as UCHAR;
                    pc = Pc::PostToDpc; continue 'sm;
                } else if InDpc != 0 && i == k {
                    pc = Pc::IntrPrepareResetController; continue 'sm;
                }
                // fall through to the AtaReq DMA-done check + error handling: re-enter
                // the post-BUSY portion of ServiceInterrupt. In C this is straight-line
                // after the BUSY block; we re-run the tail by jumping to a shared point.
                pc = service_after_busy(
                    deviceExtension, chan, &mut srb, AtaReq, lChannel, DeviceNumber, atapiDev,
                    DmaTransfer, &mut statusByte, &mut dma_status, &mut error, &mut NoRetry,
                    OldReqState, LunExt, HwDeviceExtension,
                );
                continue 'sm;
            }

            Pc::PioWaitBusy => {
                let mut reset = false;
                let mut to_dpc = false;
                i = 0;
                while i < 5 * 30 {
                    statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
                    if statusByte as u32 & IDE_STATUS_BUSY == 0 {
                        break;
                    }
                    if InDpc == 0 {
                        (*ata_req_core(AtaReq)).ReqState = REQ_STATE_DPC_WAIT_BUSY as UCHAR;
                        TimerValue = 200;
                        to_dpc = true;
                        break;
                    }
                    ScsiPortStallExecution(100);
                    i += 1;
                }
                if to_dpc { pc = Pc::PostToDpc; continue 'sm; }
                if i == 5 * 30 { reset = true; }
                if reset { pc = Pc::IntrPrepareResetController; continue 'sm; }

                if statusByte as u32 & IDE_STATUS_DRQ != 0 {
                    if !AtaReq.is_null() && (*ata_req_core(AtaReq)).WordsLeft != 0 {
                        pc = Pc::ContinuePio; continue 'sm;
                    }
                }
                if (statusByte as u32 & IDE_STATUS_DRQ) != 0
                    && (*LunExt).DeviceFlags & (DFLAGS_ATAPI_DEVICE | DFLAGS_TAPE_DEVICE | DFLAGS_LBA_ENABLED) != 0
                {
                    pc = Pc::PioWaitDrq; continue 'sm;
                }
                if AtapiDmaPioSync(HwDeviceExtension, srb, (*srb).DataBuffer as PUCHAR, (*srb).DataTransferLength) == 0 {
                    // can't sync
                }
                pc = complete_after_pio(
                    deviceExtension, chan, &mut srb, AtaReq, &mut status, &mut wordCount,
                    PathId, TargetId, Lun, LunExt, &mut error, &mut NoStartIo, &mut UseDpc,
                    HwDeviceExtension, c,
                );
                continue 'sm;
            }

            Pc::PioWaitDrq => {
                let mut reset = false;
                let mut to_dpc = false;
                i = 0;
                while i < 200 {
                    statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
                    if statusByte as u32 & IDE_STATUS_DRQ == 0 {
                        break;
                    }
                    if InDpc == 0 {
                        (*ata_req_core(AtaReq)).ReqState = REQ_STATE_DPC_WAIT_DRQ as UCHAR;
                        TimerValue = 100;
                        to_dpc = true;
                        break;
                    }
                    ScsiPortStallExecution(100);
                    i += 1;
                }
                if to_dpc { pc = Pc::PostToDpc; continue 'sm; }
                if i == 200 { reset = true; }
                if reset { pc = Pc::IntrPrepareResetController; continue 'sm; }

                if AtapiDmaPioSync(HwDeviceExtension, srb, (*srb).DataBuffer as PUCHAR, (*srb).DataTransferLength) == 0 {}
                pc = complete_after_pio(
                    deviceExtension, chan, &mut srb, AtaReq, &mut status, &mut wordCount,
                    PathId, TargetId, Lun, LunExt, &mut error, &mut NoStartIo, &mut UseDpc,
                    HwDeviceExtension, c,
                );
                continue 'sm;
            }

            Pc::CompleteRequest => {
                pc = interrupt_complete_request(
                    deviceExtension, chan, &mut srb, AtaReq, &mut status, &mut wordCount,
                    PathId, TargetId, Lun, LunExt, atapiDev, DmaTransfer, InDpc, &mut error,
                    &mut NoStartIo, &mut UseDpc, &mut TimerValue, OldReqState, HwDeviceExtension, c,
                );
                continue 'sm;
            }

            Pc::CompleteRdp | Pc::IntrCompleteReq => {
                // handled inside interrupt_complete_request; should not reach here directly.
                pc = Pc::ReturnEnableIntr; continue 'sm;
            }

            Pc::EnqueueNextReq => {
                srb = UniataGetCurRequest(chan);
                pc = Pc::ReenqueueReq; continue 'sm;
            }

            Pc::ReenqueueReq => {
                if srb.is_null() {
                    ScsiPortNotification(
                        _SCSI_NOTIFICATION_TYPE_NextRequest as SCSI_NOTIFICATION_TYPE,
                        deviceExtension as PVOID, core::ptr::null_mut::<core::ffi::c_void>(),
                    );
                } else {
                    ScsiPortNotification(
                        _SCSI_NOTIFICATION_TYPE_NextLuRequest as SCSI_NOTIFICATION_TYPE,
                        deviceExtension as PVOID, PathId as c_uint, TargetId as c_uint, Lun as c_uint,
                    );
                    if (*deviceExtension).simplexOnly == 0 {
                        AtapiStartIo__(HwDeviceExtension, srb, FALSE);
                    }
                }
                if (*deviceExtension).simplexOnly != 0 {
                    NoStartIo = FALSE;
                }
                pc = Pc::ReturnEnableIntr; continue 'sm;
            }

            Pc::ReturnEnableIntr => {
                (*deviceExtension).ExpectingInterrupt = TRUE as ULONG;
                if UseDpc != 0 {
                    if CrNtInterlockedExchangeAdd(&mut (*chan).DisableIntr, 0) != 0 {
                        AtapiEnableInterrupts(HwDeviceExtension, c as ULONG);
                        InterlockedExchange(&mut (*chan).CheckIntr, CHECK_INTR_IDLE as LONG);
                        AtapiQueueTimerDpc(HwDeviceExtension, lChannel as ULONG, core::mem::transmute::<_, PHW_TIMER>(Some(AtapiEnableInterrupts__ as unsafe extern "system" fn(PVOID))), 1);
                    }
                }
                InterlockedExchange(&mut (*chan).CheckIntr, CHECK_INTR_IDLE as LONG);

                if UseDpc == 0 && NoStartIo == 0 {
                    chan = UniataGetNextChannel(chan);
                    if !chan.is_null() {
                        srb = UniataGetCurRequest(chan);
                    } else {
                        srb = core::ptr::null_mut();
                    }
                    if !srb.is_null() {
                        AtapiStartIo__(HwDeviceExtension, srb, FALSE);
                    }
                }
                return TRUE;
            }

            Pc::Done => return TRUE,
        }
    }
}

// ---- AtapiInterrupt__ block helpers (each returns the next Pc) ----

// The big interruptReason dispatch (AHCI / write-CDB / toDev / toHost / complete / unexpected).
unsafe fn interrupt_dispatch_reason(
    deviceExtension: PHW_DEVICE_EXTENSION,
    chan_ref: &mut PHW_CHANNEL,
    srb_ref: &mut PSCSI_REQUEST_BLOCK,
    AtaReq_ref: &mut PATA_REQ,
    lChannel: UCHAR,
    DeviceNumber: UCHAR,
    atapiDev: BOOLEAN,
    DmaTransfer: BOOLEAN,
    statusByte: &mut UCHAR,
    interruptReason: &mut UCHAR,
    wordCount: &mut ULONG,
    _wordsThisInterrupt: ULONG,
    status: &mut ULONG,
    DataOverrun: &mut BOOLEAN,
    _error: &mut UCHAR,
    OldReqState: UCHAR,
    LunExt: PHW_LU_EXTENSION,
) -> Pc {
    let chan = *chan_ref;
    let srb = *srb_ref;
    let AtaReq = *AtaReq_ref;
    let HwDeviceExtension = deviceExtension as PVOID;

    if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
        if (*ata_req_core(AtaReq)).WordsTransfered >= (*ata_req_core(AtaReq)).WordsLeft {
            (*ata_req_core(AtaReq)).WordsLeft = 0;
        } else {
            (*ata_req_core(AtaReq)).WordsLeft -= (*ata_req_core(AtaReq)).WordsTransfered;
        }
        *status = SRB_STATUS_SUCCESS;
        (*chan).ChannelCtrlFlags &= !CTRFLAGS_DMA_OPERATION;
        return Pc::CompleteRequest;
    } else if *interruptReason as u32 == ATAPI_IR_COD_Cmd && (*statusByte as u32 & IDE_STATUS_DRQ) != 0 {
        if (*chan).ChannelCtrlFlags & CTRFLAGS_DMA_OPERATION != 0 {
            AtapiDmaDBPreSync(HwDeviceExtension, chan, srb);
        }
        // Write the packet (CDB).
        AtapiWriteBuffer2(
            chan, IDX_IO1_o_Data, (*srb).Cdb.as_ptr() as PVOID,
            if (*LunExt).IdentifyData.AtapiCmdSize() != 0 { 8 } else { 6 },
            PIO0_TIMING,
        );
        (*ata_req_core(AtaReq)).ReqState = REQ_STATE_ATAPI_EXPECTING_DATA_INTR as UCHAR;
        if (*chan).ChannelCtrlFlags & CTRFLAGS_DMA_OPERATION != 0 {
            AtapiDmaStart(HwDeviceExtension, DeviceNumber as ULONG, lChannel as ULONG, srb);
        }
        return Pc::ReturnEnableIntr;
    } else if *interruptReason as u32 == ATAPI_IR_IO_toDev && (*statusByte as u32 & IDE_STATUS_DRQ) != 0 {
        // Write the data.
        if atapiDev != 0 {
            *wordCount = AtapiReadPort1(chan, IDX_ATAPI_IO1_i_ByteCountLow) as ULONG;
            *wordCount |= (AtapiReadPort1(chan, IDX_ATAPI_IO1_i_ByteCountHigh) as ULONG) << 8;
            *wordCount >>= 1;
            if *wordCount > (*ata_req_core(AtaReq)).WordsLeft {
                *wordCount = (*ata_req_core(AtaReq)).WordsLeft;
                *DataOverrun = TRUE;
            }
        } else {
            if (*ata_req_core(AtaReq)).WordsLeft < _wordsThisInterrupt {
                *wordCount = (*ata_req_core(AtaReq)).WordsLeft;
            } else {
                *wordCount = _wordsThisInterrupt;
            }
        }

        if DmaTransfer != 0 && (*chan).ChannelCtrlFlags & CTRFLAGS_DMA_OPERATION != 0 {
            if (*ata_req_core(AtaReq)).ReqState as u32 == REQ_STATE_ATAPI_EXPECTING_DATA_INTR2 {
                if (*ata_req_core(AtaReq)).WordsLeft > *wordCount {
                    (*ata_req_core(AtaReq)).WordsLeft -= *wordCount;
                    (*ata_req_core(AtaReq)).WordsTransfered += *wordCount;
                    (*ata_req_core(AtaReq)).ReqState = REQ_STATE_ATAPI_EXPECTING_DATA_INTR as UCHAR;
                    return Pc::ReturnEnableIntr;
                }
                AtapiDmaDone(HwDeviceExtension, DEVNUM_NOT_SPECIFIED, lChannel as ULONG, core::ptr::null_mut());
            }
            (*ata_req_core(AtaReq)).WordsTransfered = (*ata_req_core(AtaReq)).WordsLeft;
            (*ata_req_core(AtaReq)).WordsLeft = 0;
            *status = SRB_STATUS_SUCCESS;
            (*chan).ChannelCtrlFlags &= !CTRFLAGS_DMA_OPERATION;
            return Pc::CompleteRequest;
        }

        if (*srb).SrbFlags & SRB_FLAGS_DATA_OUT != 0 {
            *statusByte = WaitOnBusy(chan);
            if (*LunExt).DeviceFlags & DFLAGS_DWORDIO_ENABLED == 0 || (*wordCount & 1) != 0 {
                AtapiWriteBuffer2(chan, IDX_IO1_o_Data, (*ata_req_core(AtaReq)).DataBuffer as PVOID, *wordCount, UniataGetPioTiming(LunExt));
            } else {
                AtapiWriteBuffer4(chan, IDX_IO1_o_Data, (*ata_req_core(AtaReq)).DataBuffer as PVOID, *wordCount / 2, UniataGetPioTiming(LunExt));
            }
        } else {
            *status = SRB_STATUS_ERROR;
            if *wordCount == 0 && atapiDev != 0 && (*srb).Cdb[0] as u32 != SCSIOP_REQUEST_SENSE {
                AtapiDisableInterrupts(deviceExtension as PVOID, lChannel as ULONG);
                AtapiSoftReset(chan, DeviceNumber as ULONG);
                AtapiEnableInterrupts(deviceExtension as PVOID, lChannel as ULONG);
                *status = SRB_STATUS_BUS_RESET;
                (*ata_req_core(AtaReq)).ReqState = REQ_STATE_TRANSFER_COMPLETE as UCHAR;
            }
            return Pc::CompleteRequest;
        }
        (*ata_req_core(AtaReq)).DataBuffer = (*ata_req_core(AtaReq)).DataBuffer.add(*wordCount as usize);
        (*ata_req_core(AtaReq)).WordsLeft -= *wordCount;
        (*ata_req_core(AtaReq)).WordsTransfered += *wordCount;
        if atapiDev != 0 {
            (*ata_req_core(AtaReq)).ReqState = REQ_STATE_ATAPI_EXPECTING_DATA_INTR as UCHAR;
        }
        return Pc::ReturnEnableIntr;
    } else if *interruptReason as u32 == ATAPI_IR_IO_toHost && (*statusByte as u32 & IDE_STATUS_DRQ) != 0 {
        return Pc::ContinueReadDrq;
    } else if *interruptReason as u32 == (ATAPI_IR_IO_toHost | ATAPI_IR_COD_Cmd) && (*statusByte as u32 & IDE_STATUS_DRQ) == 0 {
        // Command complete.
        if DmaTransfer != 0 {
            (*ata_req_core(AtaReq)).WordsTransfered += (*ata_req_core(AtaReq)).WordsLeft;
            (*ata_req_core(AtaReq)).WordsLeft = 0;
        } else {
            *wordCount = (*ata_req_core(AtaReq)).WordsLeft;
            (*ata_req_core(AtaReq)).DataBuffer = (*ata_req_core(AtaReq)).DataBuffer.add(*wordCount as usize);
            (*ata_req_core(AtaReq)).WordsLeft -= *wordCount;
            (*ata_req_core(AtaReq)).WordsTransfered += *wordCount;
        }
        *status = SRB_STATUS_SUCCESS;
        return Pc::CompleteRequest;
    } else {
        // Unexpected int.
        if g_opt_VirtualMachine == VM_QEMU {
            if *interruptReason as u32 == ATAPI_IR_IO_toDev
                && (*statusByte as u32 & IDE_STATUS_DRQ) == 0
                && DmaTransfer == 0
            {
                *statusByte = WaitForDrq(chan);
                if *statusByte as u32 & IDE_STATUS_DRQ != 0 {
                    return Pc::ContinuePio;
                }
            }
        }
        if OldReqState as u32 == REQ_STATE_DPC_WAIT_BUSY0
            && (*ata_req_core(AtaReq)).WordsLeft == 0
        {
            *status = SRB_STATUS_SUCCESS;
            (*chan).ChannelCtrlFlags &= !CTRFLAGS_DMA_OPERATION;
            return Pc::CompleteRequest;
        }
        return Pc::ReturnEnableIntr;
    }
}

// continue_read_drq: block — PIO/DMA read path.
unsafe fn interrupt_read_drq(
    deviceExtension: PHW_DEVICE_EXTENSION,
    chan: PHW_CHANNEL,
    srb: PSCSI_REQUEST_BLOCK,
    AtaReq: PATA_REQ,
    lChannel: UCHAR,
    _DeviceNumber: UCHAR,
    atapiDev: BOOLEAN,
    DmaTransfer: BOOLEAN,
    statusByte: &mut UCHAR,
    wordCount: &mut ULONG,
    wordsThisInterrupt: ULONG,
    status: &mut ULONG,
    DataOverrun: &mut BOOLEAN,
    LunExt: PHW_LU_EXTENSION,
    HwDeviceExtension: PVOID,
) -> Pc {
    let mut i: ULONG;

    if atapiDev != 0 {
        *wordCount = (AtapiReadPort1(chan, IDX_ATAPI_IO1_i_ByteCountLow) as ULONG)
            | ((AtapiReadPort1(chan, IDX_ATAPI_IO1_i_ByteCountHigh) as ULONG) << 8);
        *wordCount >>= 1;
        if *wordCount > (*ata_req_core(AtaReq)).WordsLeft {
            *wordCount = (*ata_req_core(AtaReq)).WordsLeft;
            *DataOverrun = TRUE;
        }
    } else {
        if (*ata_req_core(AtaReq)).WordsLeft < wordsThisInterrupt {
            *wordCount = (*ata_req_core(AtaReq)).WordsLeft;
        } else {
            *wordCount = wordsThisInterrupt;
        }
    }

    if DmaTransfer != 0 && (*chan).ChannelCtrlFlags & CTRFLAGS_DMA_OPERATION != 0 {
        if (*ata_req_core(AtaReq)).ReqState as u32 == REQ_STATE_ATAPI_EXPECTING_DATA_INTR2 {
            if (*ata_req_core(AtaReq)).WordsLeft > *wordCount {
                (*ata_req_core(AtaReq)).WordsLeft -= *wordCount;
                (*ata_req_core(AtaReq)).WordsTransfered += *wordCount;
                (*ata_req_core(AtaReq)).ReqState = REQ_STATE_ATAPI_EXPECTING_DATA_INTR as UCHAR;
                return Pc::ReturnEnableIntr;
            }
            AtapiDmaDone(HwDeviceExtension, DEVNUM_NOT_SPECIFIED, lChannel as ULONG, core::ptr::null_mut());
        }
        (*ata_req_core(AtaReq)).WordsTransfered = (*ata_req_core(AtaReq)).WordsLeft;
        (*ata_req_core(AtaReq)).WordsLeft = 0;
        *status = SRB_STATUS_SUCCESS;
        (*chan).ChannelCtrlFlags &= !CTRFLAGS_DMA_OPERATION;
        return Pc::CompleteRequest;
    }

    if (*srb).SrbFlags & SRB_FLAGS_DATA_IN != 0 {
        *statusByte = WaitOnBusy(chan);

        if *wordCount & 1 != 0 && atapiDev != 0 {
            // unaligned ATAPI; fall through to 16-bit read below
        } else if (*LunExt).DeviceFlags & DFLAGS_DWORDIO_ENABLED != 0 {
            AtapiReadBuffer4(chan, IDX_IO1_i_Data, (*ata_req_core(AtaReq)).DataBuffer as PVOID, *wordCount / 2, UniataGetPioTiming(LunExt));
            (*ata_req_core(AtaReq)).DataBuffer = (*ata_req_core(AtaReq)).DataBuffer.add((*wordCount & !1) as usize);
            (*ata_req_core(AtaReq)).WordsLeft -= *wordCount & !1;
            (*ata_req_core(AtaReq)).WordsTransfered += *wordCount & !1;
            *wordCount &= 1;
        }
        if *wordCount != 0 {
            AtapiReadBuffer2(chan, IDX_IO1_i_Data, (*ata_req_core(AtaReq)).DataBuffer as PVOID, *wordCount, UniataGetPioTiming(LunExt));
        }

        *statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus

        if *DataOverrun != 0 {
            AtapiSuckPort2(chan);
            *statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
        }

        if *statusByte as u32 & IDE_STATUS_BUSY != 0 {
            i = 0;
            while i < 2 {
                ScsiPortStallExecution(10);
                *statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
                if *statusByte as u32 & IDE_STATUS_BUSY == 0 {
                    break;
                }
                i += 1;
            }
        }
    } else {
        *status = SRB_STATUS_ERROR;
        return Pc::CompleteRequest;
    }

    (*ata_req_core(AtaReq)).DataBuffer = (*ata_req_core(AtaReq)).DataBuffer.add(*wordCount as usize);
    (*ata_req_core(AtaReq)).WordsLeft -= *wordCount;
    (*ata_req_core(AtaReq)).WordsTransfered += *wordCount;

    if (*ata_req_core(AtaReq)).WordsLeft == 0 {
        if atapiDev != 0 {
            if (*LunExt).IdentifyData.DeviceType() as u32 == ATAPI_TYPE_CDROM {
                if (*srb).Cdb[0] as u32 == SCSIOP_READ_CAPACITY {
                    (*ata_req_core(AtaReq)).DataBuffer =
                        (*ata_req_core(AtaReq)).DataBuffer.sub((*ata_req_core(AtaReq)).WordsTransfered as usize);
                    let db = (*ata_req_core(AtaReq)).DataBuffer;
                    if *db == 0x00 {
                        *((db as *mut ULONG)) = 0xFFFFFF7F;
                    }
                    *((db.add(2) as *mut ULONG)) = 0x00080000;
                    (*ata_req_core(AtaReq)).DataBuffer =
                        (*ata_req_core(AtaReq)).DataBuffer.add((*ata_req_core(AtaReq)).WordsTransfered as usize);
                }
                // UNIATA_INIT_CHANGERS not defined:
                else if (*srb).Cdb[0] as u32 == SCSIOP_MECHANISM_STATUS {
                    AtapiHwInitializeChanger(
                        HwDeviceExtension, srb,
                        (*srb).DataBuffer as PMECHANICAL_STATUS_INFORMATION_HEADER,
                    );
                    (*LunExt).DeviceFlags |= DFLAGS_CHANGER_INITED;
                }
            }
            *statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus); // GetStatus
            if *statusByte as u32 & IDE_STATUS_BUSY == 0 {
                *status = SRB_STATUS_SUCCESS;
                return Pc::CompleteRequest;
            }
        } else {
            *status = SRB_STATUS_SUCCESS;
            return Pc::CompleteRequest;
        }
    } else {
        if atapiDev != 0 {
            (*ata_req_core(AtaReq)).ReqState = REQ_STATE_ATAPI_EXPECTING_DATA_INTR as UCHAR;
            *statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus); // GetStatus
            if *statusByte as u32 & IDE_STATUS_BUSY == 0 {
                *status = SRB_STATUS_SUCCESS;
                return Pc::CompleteRequest;
            }
        } else {
            if atapiDev == 0
                && *DataOverrun == 0
                && (*srb).SrbFlags & SRB_FLAGS_DATA_IN != 0
                && ((*statusByte as u32 & !IDE_STATUS_INDEX) == (IDE_STATUS_IDLE | IDE_STATUS_DRQ))
            {
                return Pc::ContinueReadDrq;
            }
        }
    }
    let _ = deviceExtension;
    Pc::ReturnEnableIntr
}

// Shared tail after the BUSY-wait section (reached when try_dpc_wait falls through):
// DMA-done switch + error check + downrate + IO_STATISTICS, ending in ContinueErr/ContinuePio.
unsafe fn service_after_busy(
    deviceExtension: PHW_DEVICE_EXTENSION,
    chan: PHW_CHANNEL,
    _srb_ref: &mut PSCSI_REQUEST_BLOCK,
    AtaReq: PATA_REQ,
    lChannel: UCHAR,
    _DeviceNumber: UCHAR,
    atapiDev: BOOLEAN,
    DmaTransfer: BOOLEAN,
    statusByte: &mut UCHAR,
    dma_status: &mut UCHAR,
    error: &mut UCHAR,
    NoRetry: &mut BOOLEAN,
    OldReqState: UCHAR,
    LunExt: PHW_LU_EXTENSION,
    HwDeviceExtension: PVOID,
) -> Pc {
    if !AtaReq.is_null() && DmaTransfer != 0 && (*deviceExtension).HwFlags & UNIATA_AHCI == 0 {
        match OldReqState as u32 {
            REQ_STATE_EARLY_INTR | REQ_STATE_DPC_WAIT_BUSY0 => {
                if (*chan).ChannelCtrlFlags & CTRFLAGS_DMA_ACTIVE != 0 {
                    *dma_status = AtapiDmaDone(HwDeviceExtension, DEVNUM_NOT_SPECIFIED, lChannel as ULONG, core::ptr::null_mut());
                }
            }
            _ => {}
        }
    }

    if (*statusByte as u32 & IDE_STATUS_ERROR != 0) || (*dma_status as u32 & BM_STATUS_ERR != 0) {
        if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
            *error = (*ata_req_ahci(AtaReq)).in_error;
            if UniataAhciAbortOperation(chan) == 0 {}
            UniataAhciWaitCommandReady(chan, 10);
            UniataAhciStatus(HwDeviceExtension, lChannel as ULONG, DEVNUM_NOT_SPECIFIED);
            if *NoRetry != 0 {
                (*ata_req_core(AtaReq)).retry += MAX_RETRIES as UCHAR;
                if *error == 0 && *statusByte as u32 & IDE_STATUS_ERROR != 0 {
                    *error |= IDE_STATUS_ERROR as UCHAR;
                }
            }
        } else {
            *error = AtapiReadPort1(chan, IDX_IO1_i_Error);
        }
        if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
        } else if atapiDev == 0 {
            ScsiPortStallExecution(100);
        } else {
            ScsiPortStallExecution(10);
        }
        return Pc::ContinueErr;
    } else if (*ata_req_core(AtaReq)).Flags as u32 & REQ_FLAG_FORCE_DOWNRATE_LBA48 != 0 {
        (*deviceExtension).HbaCtrlFlags |= HBAFLAGS_DMA_DISABLED_LBA48;
    } else if (*ata_req_core(AtaReq)).Flags as u32 & REQ_FLAG_FORCE_DOWNRATE != 0 {
        (*LunExt).RecoverCount[(*ata_req_core(AtaReq)).retry as usize] += 1;
        if (*LunExt).RecoverCount[(*ata_req_core(AtaReq)).retry as usize] >= (*LunExt).BlockIoCount / 3
            || ((*deviceExtension).HwFlags & UNIATA_NO80CHK) != 0
        {
            (*LunExt).LimitedTransferMode = (*LunExt).TransferMode;
        }
    }
    if (*ata_req_core(AtaReq)).bcount != 0 {
        (*LunExt).BlockIoCount += 1;
    }
    (*LunExt).IoCount += 1;
    Pc::ContinuePio
}

// CompleteRequest: items 1-3 (underrun, OriginalSrb resend, MapError) then dispatch:
//  - PIO completion -> Pc::PioWaitBusy (waits, then calls complete_after_pio)
//  - otherwise -> complete_after_pio (the SRB-completion tail)
unsafe fn interrupt_complete_request(
    deviceExtension: PHW_DEVICE_EXTENSION,
    chan: PHW_CHANNEL,
    srb_ref: &mut PSCSI_REQUEST_BLOCK,
    AtaReq: PATA_REQ,
    status: &mut ULONG,
    wordCount: &mut ULONG,
    PathId: UCHAR,
    TargetId: UCHAR,
    Lun: UCHAR,
    LunExt: PHW_LU_EXTENSION,
    _atapiDev: BOOLEAN,
    DmaTransfer: BOOLEAN,
    _InDpc: BOOLEAN,
    error: &mut UCHAR,
    NoStartIo: &mut BOOLEAN,
    UseDpc: &mut BOOLEAN,
    _TimerValue: &mut ULONG,
    _OldReqState: UCHAR,
    HwDeviceExtension: PVOID,
    c: UCHAR,
) -> Pc {
    let mut srb = *srb_ref;

    if (*ata_req_core(AtaReq)).WordsLeft != 0 && *status == SRB_STATUS_SUCCESS {
        *status = SRB_STATUS_DATA_OVERRUN;
    }

    if !(*ata_req_core(AtaReq)).OriginalSrb.is_null() {
        // UNIATA_INIT_CHANGERS not defined: only SCSIOP_REQUEST_SENSE reaches here usefully.
        if (*srb).Cdb[0] as u32 == SCSIOP_MECHANISM_STATUS {
            // ERROR path in non-changer build; fall to error-complete below.
        } else {
            // srb->Cdb[0] == SCSIOP_REQUEST_SENSE
            if *status == SRB_STATUS_DATA_OVERRUN {
                let add_off = core::mem::offset_of!(SENSE_DATA, AdditionalSenseLength) as ULONG;
                if ((*srb).DataTransferLength - (*ata_req_core(AtaReq)).WordsLeft)
                    > (add_off + core::mem::size_of::<UCHAR>() as ULONG)
                {
                    *status = SRB_STATUS_SUCCESS;
                }
            }

            if *status == SRB_STATUS_SUCCESS {
                // Get ready to issue the original srb
                srb = (*ata_req_core(AtaReq)).OriginalSrb;
                (*ata_req_core(AtaReq)).Srb = srb;
                (*ata_req_core(AtaReq)).OriginalSrb = core::ptr::null_mut();

                let srbStatus = AtapiSendCommand(HwDeviceExtension, srb, CMD_ACTION_ALL as ULONG);
                if srbStatus == SRB_STATUS_PENDING {
                    *srb_ref = srb;
                    return Pc::ReturnEnableIntr;
                }
            }
        }

        // AtapiSendCommand failed: pretend the original srb failed and complete it.
        if !(*ata_req_core(AtaReq)).OriginalSrb.is_null() {
            srb = (*ata_req_core(AtaReq)).OriginalSrb;
            (*ata_req_core(AtaReq)).Srb = srb;
            (*ata_req_core(AtaReq)).OriginalSrb = core::ptr::null_mut();
        }
        *status = SRB_STATUS_ERROR;
        (*srb).ScsiStatus = 0;
        (*ata_req_core(AtaReq)).DataBuffer = (*srb).DataBuffer as PUSHORT;
        (*ata_req_core(AtaReq)).WordsLeft = (*srb).DataTransferLength;
        (*chan).RDP = FALSE;
    } else if *status == SRB_STATUS_ERROR {
        *status = MapError(deviceExtension as PVOID, srb);
        (*chan).RDP = FALSE;
    } else if DmaTransfer == 0 {
        // PIO completion -> wait BUSY in the state machine
        *srb_ref = srb;
        return Pc::PioWaitBusy;
    }

    *srb_ref = srb;
    complete_after_pio(
        deviceExtension, chan, srb_ref, AtaReq, status, wordCount,
        PathId, TargetId, Lun, LunExt, error, NoStartIo, UseDpc, HwDeviceExtension, c,
    )
}

// The SRB-completion tail (C: from `UniataExpectChannelInterrupt(chan, FALSE)` after the
// PIO_wait section through enqueue_next_req). Returns the next Pc.
unsafe fn complete_after_pio(
    deviceExtension: PHW_DEVICE_EXTENSION,
    chan: PHW_CHANNEL,
    srb_ref: &mut PSCSI_REQUEST_BLOCK,
    AtaReq: PATA_REQ,
    status: &mut ULONG,
    wordCount: &mut ULONG,
    PathId: UCHAR,
    TargetId: UCHAR,
    Lun: UCHAR,
    LunExt: PHW_LU_EXTENSION,
    error: &mut UCHAR,
    _NoStartIo: &mut BOOLEAN,
    _UseDpc: &mut BOOLEAN,
    HwDeviceExtension: PVOID,
    _c: UCHAR,
) -> Pc {
    let mut srb = *srb_ref;
    let mut i: ULONG;

    UniataExpectChannelInterrupt(chan, FALSE);
    InterlockedExchange(&mut (*chan).CheckIntr, CHECK_INTR_IDLE as LONG);

    if !srb.is_null() {
        (*srb).SrbStatus = *status as UCHAR;

        if (*ata_req_core(AtaReq)).WordsLeft != 0 {
            if (*LunExt).DeviceFlags & DFLAGS_TAPE_DEVICE == 0 {
                if *status == SRB_STATUS_DATA_OVERRUN {
                    (*srb).DataTransferLength -= (*ata_req_core(AtaReq)).WordsLeft * 2;
                } else {
                    (*srb).DataTransferLength = 0;
                }
            } else {
                (*srb).DataTransferLength -= (*ata_req_core(AtaReq)).WordsLeft * 2;
            }
        }
        if *status == SRB_STATUS_SUCCESS {
            if (*LunExt).DeviceFlags & DFLAGS_ATAPI_DEVICE == 0
                && (*ata_req_core(AtaReq)).WordsTransfered * 2 < (*ata_req_core(AtaReq)).TransferLength
            {
                (*ata_req_core(AtaReq)).Flags &= !(REQ_FLAG_DMA_OPERATION as UCHAR);
                (*ata_req_core(AtaReq)).ReqState = REQ_STATE_PREPARE_TO_NEXT as UCHAR;
                *srb_ref = srb;
                return Pc::ReenqueueReq;
            }
        }

        if (*srb).Function as u32 != SRB_FUNCTION_IO_CONTROL {
            // CompleteRDP / IntrCompleteReq (RequestComplete)
            complete_srb(deviceExtension, chan, srb, AtaReq, *status);
        } else {
            // IOCTL completion
            if *status != SRB_STATUS_SUCCESS {
                *error = AtapiReadPort1(chan, IDX_IO1_i_Error);
            }
            if _strnicmp(
                (*srb).DataBuffer as PCCHAR,
                b"SCSIDISK\0".as_ptr() as PCCHAR,
                "SCSIDISK".len(),
            ) == 0
            {
                let cmdOutParameters =
                    ((*srb).DataBuffer as *mut UCHAR).add(core::mem::size_of::<SRB_IO_CONTROL>()) as PSENDCMDOUTPARAMS;
                (*cmdOutParameters).cBufferSize = *wordCount;
                (*cmdOutParameters).DriverStatus.bDriverError = if *error != 0 { SMART_IDE_ERROR } else { 0 };
                (*cmdOutParameters).DriverStatus.bIDEError = *error;

                if (*chan).SmartCommand == RETURN_SMART_STATUS {
                    let regs = (&mut (*cmdOutParameters).bBuffer) as *mut [UCHAR; 1] as PIDEREGS_EX;
                    (*regs).bOpFlags = 0;
                    UniataSnapAtaRegs(chan, 0, regs);
                    *(*regs).__bindgen_anon_2.bCommandReg.as_mut() = SMART_CMD;
                    *(*regs).__bindgen_anon_1.bFeaturesReg.as_mut() = RETURN_SMART_STATUS;
                    (*cmdOutParameters).cBufferSize = 8;
                }
                (*chan).SmartCommand = 0;
            }
            // Indicate command complete (IntrCompleteReq).
            complete_srb(deviceExtension, chan, srb, AtaReq, *status);
        }
    }

    if (*chan).RDP != 0 {
        i = 0;
        while i < 5 {
            let sb = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
            if sb as u32 & IDE_STATUS_BUSY == 0 {
                (*chan).RDP = FALSE;
                // CompleteRDP: re-run the complete path (already completed above);
                // RDP is cleared, continue to enqueue.
                break;
            } else if sb as u32 & IDE_STATUS_DSC != 0 {
                (*chan).RDP = FALSE;
                break;
            }
            ScsiPortStallExecution(50);
            i += 1;
        }
    }
    if (*chan).RDP != 0 {
        // RequestTimerCall 2000 -> CallTimerDpc path (caller has TimerValue context lost;
        // emulate with AtapiQueueTimerDpc(AtapiCallBack_X, 2000)).
        AtapiQueueTimerDpc(HwDeviceExtension, (*chan).lChannel, core::mem::transmute::<_, PHW_TIMER>(Some(AtapiCallBack_X as unsafe extern "system" fn(PVOID))), 2000);
        return Pc::ReturnEnableIntr;
    }

    // enqueue_next_req:
    srb = UniataGetCurRequest(chan);
    *srb_ref = srb;
    let _ = (PathId, TargetId, Lun);
    Pc::ReenqueueReq
}

// IntrCompleteReq: set AutoSense, sync DMA buffers, remove + complete the SRB.
unsafe fn complete_srb(
    deviceExtension: PHW_DEVICE_EXTENSION,
    chan: PHW_CHANNEL,
    srb: PSCSI_REQUEST_BLOCK,
    _AtaReq: PATA_REQ,
    status: ULONG,
) {
    if status == SRB_STATUS_SUCCESS
        && !(*srb).SenseInfoBuffer.is_null()
        && (*srb).SenseInfoBufferLength as usize >= core::mem::size_of::<SENSE_DATA>()
    {
        let senseBuffer = (*srb).SenseInfoBuffer as PSENSE_DATA;
        (*senseBuffer).set_ErrorCode(0);
        (*senseBuffer).set_Valid(1);
        (*senseBuffer).AdditionalSenseLength = 0xb;
        (*senseBuffer).set_SenseKey(0);
        (*senseBuffer).AdditionalSenseCode = 0;
        (*senseBuffer).AdditionalSenseCodeQualifier = 0;
        (*srb).SrbStatus |= SRB_STATUS_AUTOSENSE_VALID as UCHAR;
    }
    AtapiDmaDBSync(chan, srb);
    UniataRemoveRequest(chan, srb);
    ScsiPortNotification(
        _SCSI_NOTIFICATION_TYPE_RequestComplete as SCSI_NOTIFICATION_TYPE,
        deviceExtension as PVOID, srb,
    );
}
