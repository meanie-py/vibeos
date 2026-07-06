//! id_ata.cpp — part 1: low-level ATA register primitives + id_ata.cpp file-scope globals.
//! Hand/assisted translation of the first contiguous block of id_ata.cpp
//! (AtapiRegGetStringParameterValue .. AtaSAmode), plus the module-owned global state.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code, unused_unsafe)]

use crate::types::*;
use crate::regidx::*;
use crate::*; // sibling uniata functions + ScsiPortGetBusDataByOffset
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::driver_ddk::DbgPrint; // perf phase markers
use crate::io_port::{
    AtapiReadPort1, AtapiReadPort2, AtapiReadPort4, AtapiReadPortEx1, AtapiReadPortEx4,
    AtapiWritePort1, AtapiWritePort2, AtapiWritePort4, AtapiWritePortEx1, AtapiWritePortEx4,
    AtapiReadBuffer2, AtapiReadBuffer4, AtapiWriteBuffer2, AtapiWriteBuffer4, UniataNanoSleep,
};

// Kernel helpers that are not surfaced by the bindgen `ddk` module.
use ntkrnl_sys::base::{NT_SUCCESS, TRUE, FALSE};
use ntkrnl_sys::rtl::RtlZeroMemory;
// Registry query constants live in driver_ddk / constants (not in the bindgen ddk re-export).
use ntkrnl_sys::driver_ddk::{RTL_QUERY_REGISTRY_DIRECT, RTL_REGISTRY_ABSOLUTE};
use ntkrnl_sys::constants::REG_SZ;

// ---------------------------------------------------------------------------
// id_ata.cpp file-scope global variables (roughly lines 65-112).
// NOTE: g_Perf / g_PerfDt live in crate::io_port (the perf-calibration owner);
// the AtapiReadPort*/WritePort*/ReadBuffer*/WriteBuffer* IO funcs + UniataNanoSleep
// also live there — they are NOT redefined here.
// ---------------------------------------------------------------------------

// "\n\nATAPI IDE MiniPort Driver (UniATA) v 0." UNIATA_VER_STR "\n"  (UNIATA_VER_STR = "47b")
// CHAR == c_char == i8; transmute the byte-string literal (layout-identical).
pub static ver_string: [CHAR; 46] = unsafe {
    core::mem::transmute(*b"\n\nATAPI IDE MiniPort Driver (UniATA) v 0.47b\n\0")
};
// UNIATA_COMM_PORT_VENDOR_STR "    \n" :
//   "UNIATA  " "Management Port " "47b" "    \n"
pub static uniata_comm_name: [CHAR; 33] =
    unsafe { core::mem::transmute(*b"UNIATA  Management Port 47b    \n\0") };

pub static mut SavedRegPath: UNICODE_STRING = UNICODE_STRING {
    Length: 0,
    MaximumLength: 0,
    Buffer: core::ptr::null_mut(),
};
pub static mut SavedRegPathBuffer: [WCHAR; 256] = [0; 256];

pub static mut SkipRaids: ULONG = 1;
pub static mut ForceSimplex: ULONG = 0;

// _DEBUG-only in C; harmless to always define.
pub static mut g_LogToDisplay: ULONG = 0;

pub static mut g_WaitBusyInISR: ULONG = 1;

pub static mut g_opt_WaitBusyResetCount: ULONG = 10000; // 20000
pub static mut g_opt_WaitBusyCount: ULONG = 200; // 20000
pub static mut g_opt_WaitBusyDelay: ULONG = 10; // 150
pub static mut g_opt_WaitDrqDelay: ULONG = 10; // 100
pub static mut g_opt_WaitBusyLongCount: ULONG = 2000; // 2000
pub static mut g_opt_WaitBusyLongDelay: ULONG = 250; // 250
pub static mut g_opt_MaxIsrWait: ULONG = 40;

pub static mut g_opt_DriveSelectNanoDelay: ULONG = 0; // 400; // ns

pub static mut g_opt_AtapiSendDisableIntr: BOOLEAN = 0; // 0
pub static mut g_opt_AtapiDmaRawRead: BOOLEAN = 1; // 0
pub static mut g_opt_AtapiNoDma: BOOLEAN = FALSE as BOOLEAN;
pub static mut g_opt_BochsDmaReadWorkaround: BOOLEAN = FALSE as BOOLEAN;
pub static mut hasPCI: BOOLEAN = FALSE as BOOLEAN;

pub static mut g_opt_VirtualMachine: ULONG = 0; // Auto

pub static mut InDriverEntry: BOOLEAN = TRUE as BOOLEAN;
pub static mut g_Dump: BOOLEAN = FALSE as BOOLEAN;

pub static mut g_opt_Verbose: BOOLEAN = 0;

pub static mut WinVer_WDM_Model: BOOLEAN = FALSE as BOOLEAN;
pub static mut CPU_num: ULONG = 1;

pub static mut g_foo: UCHAR = 0;

// LBA28 bounds (bsmaster.h DEF_U64). Not surfaced by the bindgen types module.
const ATA_MAX_IOLBA28: ULONGLONG = 0x0fffff80;
const ATA_MAX_LBA28: ULONGLONG = 0x0fffffff;

// ---------------------------------------------------------------------------
// AtapiRegGetStringParameterValue
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn AtapiRegGetStringParameterValue(
    RegistryPath: PWSTR,
    Name: PWSTR,
    Str: PWCHAR,
    MaxLen: ULONG,
) -> BOOLEAN {
    const ITEMS_TO_QUERY: usize = 2; // always 1 greater than what is searched
    let status: NTSTATUS;
    let mut parameters: [RTL_QUERY_REGISTRY_TABLE; ITEMS_TO_QUERY] = core::mem::zeroed();
    let mut ustr: UNICODE_STRING = core::mem::zeroed();

    ustr.Buffer = Str;
    ustr.MaximumLength = MaxLen as USHORT;
    ustr.Length = MaxLen as USHORT;
    RtlZeroMemory(
        parameters.as_mut_ptr() as PVOID,
        (core::mem::size_of::<RTL_QUERY_REGISTRY_TABLE>() * ITEMS_TO_QUERY) as usize,
    );

    parameters[0].Flags = RTL_QUERY_REGISTRY_DIRECT;
    parameters[0].Name = Name as PCWSTR;
    parameters[0].EntryContext = &mut ustr as *mut UNICODE_STRING as PVOID;
    parameters[0].DefaultType = REG_SZ as ULONG;
    parameters[0].DefaultData = Str as PVOID;
    parameters[0].DefaultLength = MaxLen;

    status = RtlQueryRegistryValues(
        RTL_REGISTRY_ABSOLUTE, /* | RTL_REGISTRY_OPTIONAL */
        RegistryPath as PCWSTR,
        parameters.as_mut_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS(status) {
        return FALSE as BOOLEAN;
    }

    TRUE as BOOLEAN
}

// ---------------------------------------------------------------------------
// AtapiSuckPort2
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn AtapiSuckPort2(chan: PHW_CHANNEL) -> UCHAR {
    let mut statusByte: UCHAR = 0;
    let mut i: ULONG;

    // Assume, proper drive is already seleted
    WaitOnBusyLong(chan);
    i = 0;
    while i < 0x10000 {
        // GetStatus(chan, statusByte)
        statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus);
        if (statusByte & IDE_STATUS_DRQ as UCHAR) != 0 {
            // Suck out any remaining bytes and throw away.
            AtapiReadPort2(chan, IDX_IO1_i_Data);
            UniataNanoSleep(PIO0_TIMING);
        } else {
            break;
        }
        i += 1;
    }
    // statusByte holds the value from the iteration that broke the loop
    // (the loop always runs at least once: 0 < 0x10000).
    statusByte
}

// ---------------------------------------------------------------------------
// AtapiSuckPortBuffer2
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn AtapiSuckPortBuffer2(
    chan: PHW_CHANNEL,
    mut Buffer: PUSHORT,
    mut Count: ULONG,
) -> ULONG {
    let mut statusByte: UCHAR;
    let mut i: ULONG;
    let mut retry: BOOLEAN = FALSE as BOOLEAN;

    // Assume, proper drive is already seleted
    WaitOnBusyLong(chan);
    i = 0;
    while i < Count {
        // GetStatus(chan, statusByte)
        statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus);
        if (statusByte & IDE_STATUS_DRQ as UCHAR) != 0 {
            // Suck out any remaining bytes and throw away.
            let data: USHORT = AtapiReadPort2(chan, IDX_IO1_i_Data);
            *Buffer = data;
            Count -= 1;
            Buffer = Buffer.add(1);
            UniataNanoSleep(PIO0_TIMING);
            retry = FALSE as BOOLEAN;
        } else {
            if i < Count && retry == 0 {
                WaitForDrq(chan);
                // mirrors C: set before break; value not re-read (kept for parity)
                #[allow(unused_assignments)]
                {
                    retry = TRUE as BOOLEAN;
                }
            }
            break;
        }
        i += 1;
    }
    if i != 0 {
        if i == Count {
            AtapiSuckPort2(chan);
        }
    }
    i
}

// ---------------------------------------------------------------------------
// FillDeviceIdentificationString  (#ifdef __REACTOS__)
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn FillDeviceIdentificationString(
    InquiryData: PINQUIRYDATA,
    IdentifyData: PIDENTIFY_DATA, // PIDENTIFY_DATA2 == PIDENTIFY_DATA
) {
    let mut i: ULONG;
    let mut IdentifierLen: ULONG;
    let mut FirstWordLen: ULONG;

    // IDENTIFIER_LETTER(Identifier, i) = ((PUCHAR)Identifier)[(i) ^ 1]
    // ModelNumber is a [CHAR; N] field; take its byte base pointer.
    let model = core::ptr::addr_of!((*IdentifyData).ModelNumber) as *const UCHAR;
    let identifier_letter = |idx: ULONG| -> UCHAR { *model.add((idx ^ 1) as usize) };

    // The INQUIRYDATA union: VendorId/ProductId/ProductRevisionLevel and
    // DeviceIdentificationString overlay at the same offset. Take a byte base.
    let inq_union = core::ptr::addr_of_mut!((*InquiryData).__bindgen_anon_1) as *mut UCHAR;
    // VendorId[8] @ 0, ProductId[16] @ 8, ProductRevisionLevel[4] @ 24,
    // DeviceIdentificationString[28] @ 0.
    let vendor_id = inq_union; // offset 0
    let product_id = inq_union.add(8); // offset 8
    let dev_ident = inq_union; // offset 0

    /* We need to copy a field which is 20 chars long to two fields which are 8+16 bytes long.
     * => Try to split identifier on space char. */

    IdentifierLen = 20;
    while IdentifierLen > 0 && identifier_letter(IdentifierLen - 1) == b' ' {
        IdentifierLen -= 1;
    }

    /* Strategy 1: copy first word to VendorId if len <= 8. Copy other chars to ProductId */
    FirstWordLen = 0;
    while identifier_letter(FirstWordLen) != b' ' && FirstWordLen < IdentifierLen {
        FirstWordLen += 1;
    }
    if FirstWordLen <= 8 {
        i = 0;
        while i < FirstWordLen {
            *vendor_id.add(i as usize) = identifier_letter(i);
            i += 1;
        }
        i = FirstWordLen + 1;
        while i < IdentifierLen {
            *product_id.add((i - FirstWordLen - 1) as usize) = identifier_letter(i);
            i += 1;
        }
        return;
    }

    /* Strategy 2: copy everything to ProductId */
    if IdentifierLen <= 16 {
        i = 0;
        while i < IdentifierLen {
            *product_id.add(i as usize) = identifier_letter(i);
            i += 1;
        }
        return;
    }

    /* Strategy 3: copy first to VendorId, then to ProductId
     * MOV_DW_SWP(DeviceIdentificationString[i], ((PUCHAR)ModelNumber)[i]) :
     *   ((PUSHORT)&dst)[0] = RtlUshortByteSwap(*(PUSHORT)&src) */
    i = 0;
    while i < 24 {
        let src = (model.add(i as usize)) as *const USHORT;
        let dst = (dev_ident.add(i as usize)) as *mut USHORT;
        core::ptr::write_unaligned(dst, RtlUshortByteSwap(core::ptr::read_unaligned(src)));
        i += 2;
    }
}

// ---------------------------------------------------------------------------
// SelectDrive
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn SelectDrive(chan: PHW_CHANNEL, DeviceNumber: ULONG) -> UCHAR {
    if chan.is_null() {
        return 0;
    }
    if (*chan).last_devsel == DeviceNumber {
        return 1;
    }
    AtapiWritePort1(
        chan,
        IDX_IO1_o_DriveSelect,
        if DeviceNumber != 0 {
            IDE_DRIVE_SELECT_2 as UCHAR
        } else {
            IDE_DRIVE_SELECT_1 as UCHAR
        },
    );
    (*chan).last_devsel = if DeviceNumber != 0 { 1 } else { 0 };
    if g_opt_DriveSelectNanoDelay == 0 {
        return 2;
    }
    UniataNanoSleep(g_opt_DriveSelectNanoDelay);
    2
}

// ---------------------------------------------------------------------------
// WaitOnBusy
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn WaitOnBusy(chan: PHW_CHANNEL) -> UCHAR {
    let mut i: ULONG;
    let mut Status: UCHAR;

    // GetStatus(chan, Status)
    Status = AtapiReadPort1(chan, IDX_IO2_AltStatus);
    i = 0;
    while i < g_opt_WaitBusyCount {
        if (Status & IDE_STATUS_BUSY as UCHAR) != 0 {
            ScsiPortStallExecution(g_opt_WaitBusyDelay);
            Status = AtapiReadPort1(chan, IDX_IO2_AltStatus);
        } else {
            break;
        }
        i += 1;
    }
    Status
}

// ---------------------------------------------------------------------------
// WaitOnBusyLong
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn WaitOnBusyLong(chan: PHW_CHANNEL) -> UCHAR {
    let mut i: ULONG;
    let mut Status: UCHAR;

    Status = WaitOnBusy(chan);
    if (Status & IDE_STATUS_BUSY as UCHAR) == 0 {
        return Status;
    }
    i = 0;
    while i < g_opt_WaitBusyLongCount {
        // GetStatus(chan, Status)
        Status = AtapiReadPort1(chan, IDX_IO2_AltStatus);
        if (Status & IDE_STATUS_BUSY as UCHAR) != 0 {
            ScsiPortStallExecution(g_opt_WaitBusyLongDelay);
        } else {
            break;
        }
        i += 1;
    }
    Status
}

// ---------------------------------------------------------------------------
// WaitOnBaseBusy
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn WaitOnBaseBusy(chan: PHW_CHANNEL) -> UCHAR {
    let mut i: ULONG;
    let mut Status: UCHAR = IDE_STATUS_WRONG as UCHAR;
    i = 0;
    while i < g_opt_WaitBusyCount {
        // GetBaseStatus(chan, Status)
        Status = AtapiReadPort1(chan, IDX_IO1_i_Status);
        if (Status & IDE_STATUS_BUSY as UCHAR) != 0 {
            ScsiPortStallExecution(g_opt_WaitBusyDelay);
        } else {
            break;
        }
        i += 1;
    }
    Status
}

// ---------------------------------------------------------------------------
// WaitOnBaseBusyLong
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn WaitOnBaseBusyLong(chan: PHW_CHANNEL) -> UCHAR {
    let mut i: ULONG;
    let mut Status: UCHAR;

    Status = WaitOnBaseBusy(chan);
    if (Status & IDE_STATUS_BUSY as UCHAR) == 0 {
        return Status;
    }
    i = 0;
    while i < 2000 {
        // GetBaseStatus(chan, Status)
        Status = AtapiReadPort1(chan, IDX_IO1_i_Status);
        if (Status & IDE_STATUS_BUSY as UCHAR) != 0 {
            ScsiPortStallExecution(250);
        } else {
            break;
        }
        i += 1;
    }
    Status
}

// ---------------------------------------------------------------------------
// UniataIsIdle
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn UniataIsIdle(
    deviceExtension: *mut _HW_DEVICE_EXTENSION,
    Status: UCHAR,
) -> UCHAR {
    let Status2: UCHAR;

    if Status == IDE_STATUS_WRONG as UCHAR {
        return IDE_STATUS_WRONG as UCHAR;
    }
    if (Status & IDE_STATUS_BUSY as UCHAR) != 0 {
        return Status;
    }
    if UniataIsSATARangeAvailable(deviceExtension, 0) != 0 {
        if (Status & (IDE_STATUS_BUSY as UCHAR | IDE_STATUS_ERROR as UCHAR)) != 0 {
            return Status;
        }
    } else {
        Status2 = Status & !(IDE_STATUS_ERROR as UCHAR | IDE_STATUS_INDEX as UCHAR);
        if (Status & IDE_STATUS_BUSY as UCHAR) != 0
            || (Status2 != IDE_STATUS_IDLE as UCHAR && Status2 != IDE_STATUS_DRDY as UCHAR)
        {
            return Status;
        }
    }
    IDE_STATUS_IDLE as UCHAR
}

// ---------------------------------------------------------------------------
// WaitForIdleLong
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn WaitForIdleLong(chan: PHW_CHANNEL) -> UCHAR {
    let mut i: ULONG;
    let mut Status: UCHAR = 0;
    let mut Status2: UCHAR;
    i = 0;
    while i < 20000 {
        // GetStatus(chan, Status)
        Status = AtapiReadPort1(chan, IDX_IO2_AltStatus);
        Status2 = UniataIsIdle((*chan).DeviceExtension, Status);
        if Status2 == IDE_STATUS_WRONG as UCHAR {
            // no drive ?
            break;
        } else if (Status2 & IDE_STATUS_BUSY as UCHAR) != 0 {
            ScsiPortStallExecution(10);
        } else {
            break;
        }
        i += 1;
    }
    Status
}

// ---------------------------------------------------------------------------
// WaitForDrq
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn WaitForDrq(chan: PHW_CHANNEL) -> UCHAR {
    let mut i: ULONG;
    let mut Status: UCHAR = 0;
    i = 0;
    while i < 1000 {
        // GetStatus(chan, Status)
        Status = AtapiReadPort1(chan, IDX_IO2_AltStatus);
        if (Status & IDE_STATUS_BUSY as UCHAR) != 0 {
            ScsiPortStallExecution(g_opt_WaitDrqDelay);
        } else if (Status & IDE_STATUS_DRQ as UCHAR) != 0 {
            break;
        } else {
            ScsiPortStallExecution(g_opt_WaitDrqDelay * 2);
        }
        i += 1;
    }
    Status
}

// ---------------------------------------------------------------------------
// WaitShortForDrq
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn WaitShortForDrq(chan: PHW_CHANNEL) -> UCHAR {
    let mut i: ULONG;
    let mut Status: UCHAR = 0;
    i = 0;
    while i < 2 {
        // GetStatus(chan, Status)
        Status = AtapiReadPort1(chan, IDX_IO2_AltStatus);
        if (Status & IDE_STATUS_BUSY as UCHAR) != 0 {
            ScsiPortStallExecution(g_opt_WaitDrqDelay);
        } else if (Status & IDE_STATUS_DRQ as UCHAR) != 0 {
            break;
        } else {
            ScsiPortStallExecution(g_opt_WaitDrqDelay);
        }
        i += 1;
    }
    Status
}

// ---------------------------------------------------------------------------
// AtapiSoftReset
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn AtapiSoftReset(chan: PHW_CHANNEL, DeviceNumber: ULONG) {
    let mut i: ULONG;
    let mut dma_status: UCHAR = 0;
    let statusByte0: UCHAR;
    let mut statusByte2: UCHAR;

    DbgPrint(b"[UNIATA] SoftReset ch=%u dev=%u\n\0".as_ptr().cast(), (*chan).lChannel, DeviceNumber); // perf phase marker
    if ((*(*chan).DeviceExtension).HwFlags & UNIATA_AHCI) != 0 {
        UniataAhciSoftReset(
            (*chan).DeviceExtension as PVOID,
            (*chan).lChannel,
            DeviceNumber,
        );
        return;
    }

    // GetBaseStatus(chan, statusByte2) — value discarded; the port read is the side effect
    let _ = AtapiReadPort1(chan, IDX_IO1_i_Status);
    SelectDrive(chan, DeviceNumber);
    if ((*(*chan).lun[DeviceNumber as usize]).DeviceFlags & DFLAGS_MANUAL_CHS) != 0 {
        // For ESDI/MFM
        ScsiPortStallExecution(10000);
        i = 0;
        while i < 1000 {
            ScsiPortStallExecution(999);
            i += 1;
        }
    } else {
        ScsiPortStallExecution(500);
        // GetBaseStatus(chan, statusByte2)
        statusByte2 = AtapiReadPort1(chan, IDX_IO1_i_Status);
        statusByte0 = statusByte2;
        AtapiWritePort1(chan, IDX_IO1_o_Command, IDE_COMMAND_ATAPI_RESET as UCHAR);

        // Do not wait for BUSY assertion if it was initially set, jump to
        // BUSY release wait loop
        if (statusByte0 & IDE_STATUS_BUSY as UCHAR) == 0 {
            // QEMU's IDE model (same lineage as Bochs) completes ATAPI RESET
            // instantly and never asserts BUSY — the 100ms wait below (10000
            // x 10us stalls + port reads, ~167ms real under TCG) is pure loss.
            // Measured: 12 soft resets/boot x 167ms = ~2.0s of amd64 boot.
            if g_opt_VirtualMachine == VM_BOCHS || g_opt_VirtualMachine == VM_QEMU {
                i = 100;
            } else {
                i = 10 * 1000;
            }
            statusByte2 = AtapiReadPort1(chan, IDX_IO1_i_Status);
            while (statusByte2 & IDE_STATUS_BUSY as UCHAR) == 0 && {
                let old = i;
                i = i.wrapping_sub(1);
                old != 0
            } {
                if (statusByte0 & IDE_STATUS_ERROR as UCHAR) == 0
                    && (statusByte2 & IDE_STATUS_ERROR as UCHAR) != 0
                {
                    break;
                }
                ScsiPortStallExecution(10);
                statusByte2 = AtapiReadPort1(chan, IDX_IO1_i_Status);
            }
        }

        i = 30 * 1000;
        while (AtapiReadPort1(chan, IDX_IO1_i_Status) & IDE_STATUS_BUSY as UCHAR) != 0 && {
            let old = i;
            i = i.wrapping_sub(1);
            old != 0
        } {
            ScsiPortStallExecution(30);
        }
        (*(*chan).lun[DeviceNumber as usize]).DeviceFlags |= DFLAGS_REINIT_DMA;
    }

    (*chan).last_devsel = 0xFFFFFFFF; // -1 : make sure proper drive would be selected
    SelectDrive(chan, DeviceNumber);
    WaitOnBusy(chan);
    // GetBaseStatus(chan, statusByte2) — value discarded; the port read is the side effect
    let _ = AtapiReadPort1(chan, IDX_IO1_i_Status);
    ScsiPortStallExecution(500);

    // GetBaseStatus(chan, statusByte2)
    statusByte2 = AtapiReadPort1(chan, IDX_IO1_i_Status);
    if !chan.is_null() && !(*chan).DeviceExtension.is_null() {
        // GetDmaStatus(de, c)
        let de = (*chan).DeviceExtension;
        let c = (*chan).lChannel;
        dma_status = if (*de).BusMaster as ULONG == DMA_MODE_BM {
            AtapiReadPort1((*de).chan.add(c as usize), IDX_BM_Status)
        } else {
            0
        };
    }
    if (dma_status & BM_STATUS_INTR as UCHAR) != 0 {
        // bullshit, we have DMA interrupt, but had never initiate DMA operation
        AtapiDmaDone(
            (*chan).DeviceExtension as PVOID,
            DeviceNumber,
            (*chan).lChannel,
            core::ptr::null_mut(),
        );
        // GetBaseStatus(chan, statusByte2)
        statusByte2 = AtapiReadPort1(chan, IDX_IO1_i_Status);
    }
    if ((*(*chan).DeviceExtension).HwFlags & UNIATA_SATA) != 0 {
        UniataSataClearErr(
            (*chan).DeviceExtension as PVOID,
            (*chan).lChannel,
            UNIATA_SATA_IGNORE_CONNECT as BOOLEAN,
            DeviceNumber,
        );
    }
    let _ = statusByte2;
    DbgPrint(b"[UNIATA] SoftReset ch=%u dev=%u done\n\0".as_ptr().cast(), (*chan).lChannel, DeviceNumber); // perf phase marker
}

// ---------------------------------------------------------------------------
// AtapiHardReset
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn AtapiHardReset(
    chan: *mut _HW_CHANNEL,
    DisableInterrupts: BOOLEAN,
    Delay: ULONG,
) {
    AtapiWritePort1(
        chan,
        IDX_IO2_o_Control,
        (IDE_DC_RESET_CONTROLLER
            | (if DisableInterrupts != 0 {
                IDE_DC_DISABLE_INTERRUPTS
            } else {
                0
            })) as UCHAR,
    );
    (*chan).last_devsel = 0xFFFFFFFF; // -1
    ScsiPortStallExecution(Delay);
    AtapiWritePort1(chan, IDX_IO2_o_Control, IDE_DC_REENABLE_CONTROLLER as UCHAR);
}

// ---------------------------------------------------------------------------
// AtaCommand48 — Send command to device. Translate to 48-LBA form if required.
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn AtaCommand48(
    deviceExtension: PHW_DEVICE_EXTENSION,
    DeviceNumber: ULONG,
    lChannel: ULONG,
    mut command: UCHAR,
    lba: ULONGLONG,
    count: USHORT,
    feature: USHORT,
    wait_flags: ULONG,
) -> UCHAR {
    // C: chan = &deviceExtension->chan[lChannel]. deviceExtension->chan is a
    // PHW_CHANNEL base pointer, so element lChannel is chan.add(lChannel).
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let mut statusByte: UCHAR;
    let mut i: ULONG;
    let plba: PUCHAR;

    if ((*deviceExtension).HwFlags & UNIATA_AHCI) != 0 {
        statusByte = UniataAhciSendPIOCommand(
            deviceExtension as PVOID,
            lChannel,
            DeviceNumber,
            core::ptr::null_mut(), // (PSCSI_REQUEST_BLOCK)NULL
            core::ptr::null_mut(),
            0,
            command,
            lba,
            count,
            feature,
            0, /* ahci flags */
            wait_flags,
            1000, /* timeout 1 sec */
        );
        return statusByte;
    }

    SelectDrive(chan, DeviceNumber);

    statusByte = WaitOnBusy(chan);

    /* ready to issue command ? */
    if (statusByte & IDE_STATUS_BUSY as UCHAR) != 0 {
        return statusByte;
    }
    // We should not check ERROR condition here.

    //>>>>>> NV: 2006/08/03
    if (AtaCommandFlags[command as usize] as ULONG
        & (ATA_CMD_FLAG_LBAIOsupp | ATA_CMD_FLAG_FUA))
        == ATA_CMD_FLAG_LBAIOsupp
        && CheckIfBadBlock((*chan).lun[DeviceNumber as usize], lba, count as ULONG) != 0
    {
        return IDE_STATUS_ERROR as UCHAR;
    }
    //<<<<<< NV:  2006/08/03

    /* only use 48bit addressing if needed because of the overhead */
    // UniAta_need_lba48(command, lba, count, supp48)
    let supp48 = (*(*chan).lun[DeviceNumber as usize])
        .IdentifyData
        .FeaturesSupport
        .Address48();
    let need_lba48 = (((AtaCommandFlags[command as usize] as ULONG & ATA_CMD_FLAG_LBAIOsupp) != 0)
        && (supp48 != 0)
        && ((lba + count as ULONGLONG) >= ATA_MAX_IOLBA28 || count as ULONG > 256))
        || (lba > ATA_MAX_LBA28)
        || (count as ULONG > 255);

    if need_lba48 {
        /* translate command into 48bit version */
        if (AtaCommandFlags[command as usize] as ULONG & ATA_CMD_FLAG_48supp) != 0 {
            command = AtaCommands48[command as usize];
        } else {
            return (-1i32) as UCHAR;
        }

        (*chan).ChannelCtrlFlags |= CTRFLAGS_LBA48;
        plba = &lba as *const ULONGLONG as PUCHAR;

        AtapiWritePort1(chan, IDX_IO1_o_Feature, (feature >> 8) as UCHAR);
        AtapiWritePort1(chan, IDX_IO1_o_Feature, feature as UCHAR);
        AtapiWritePort1(chan, IDX_IO1_o_BlockCount, (count >> 8) as UCHAR);
        AtapiWritePort1(chan, IDX_IO1_o_BlockCount, count as UCHAR);
        AtapiWritePort1(chan, IDX_IO1_o_BlockNumber, *plba.add(3));
        AtapiWritePort1(chan, IDX_IO1_o_BlockNumber, *plba.add(0));
        AtapiWritePort1(chan, IDX_IO1_o_CylinderLow, *plba.add(4));
        AtapiWritePort1(chan, IDX_IO1_o_CylinderLow, *plba.add(1));
        AtapiWritePort1(chan, IDX_IO1_o_CylinderHigh, *plba.add(5));
        AtapiWritePort1(chan, IDX_IO1_o_CylinderHigh, *plba.add(2));

        AtapiWritePort1(
            chan,
            IDX_IO1_o_DriveSelect,
            (IDE_USE_LBA | (if DeviceNumber != 0 { IDE_DRIVE_2 } else { IDE_DRIVE_1 })) as UCHAR,
        );
    } else {
        plba = &lba as *const ULONGLONG as PUCHAR; //ktp
        (*chan).ChannelCtrlFlags &= !CTRFLAGS_LBA48;

        AtapiWritePort1(chan, IDX_IO1_o_Feature, feature as UCHAR);
        AtapiWritePort1(chan, IDX_IO1_o_BlockCount, count as UCHAR);
        AtapiWritePort1(chan, IDX_IO1_o_BlockNumber, *plba.add(0));
        AtapiWritePort1(chan, IDX_IO1_o_CylinderLow, *plba.add(1));
        AtapiWritePort1(chan, IDX_IO1_o_CylinderHigh, *plba.add(2));
        if ((*(*chan).lun[DeviceNumber as usize]).DeviceFlags & DFLAGS_LBA_ENABLED) != 0 {
            AtapiWritePort1(
                chan,
                IDX_IO1_o_DriveSelect,
                ((*plba.add(3) & 0xf)
                    | IDE_USE_LBA as UCHAR
                    | (if DeviceNumber != 0 {
                        IDE_DRIVE_SELECT_2 as UCHAR
                    } else {
                        IDE_DRIVE_SELECT_1 as UCHAR
                    })),
            );
        } else {
            AtapiWritePort1(
                chan,
                IDX_IO1_o_DriveSelect,
                ((*plba.add(3) & 0xf)
                    | (if DeviceNumber != 0 {
                        IDE_DRIVE_SELECT_2 as UCHAR
                    } else {
                        IDE_DRIVE_SELECT_1 as UCHAR
                    })),
            );
        }
    }

    // write command code to device
    AtapiWritePort1(chan, IDX_IO1_o_Command, command);

    match wait_flags {
        x if x == ATA_WAIT_INTR => {
            // caller requested wait for interrupt
            i = 0;
            while i < 4 {
                WaitOnBusy(chan);
                statusByte = WaitForDrq(chan);
                if (statusByte & IDE_STATUS_DRQ as UCHAR) != 0 {
                    break;
                }
                ScsiPortStallExecution(500);
                i += 1;
            }
            return statusByte;
        }
        x if x == ATA_WAIT_IDLE => {
            // caller requested wait for entering Wait state
            i = 0;
            while i < 30 * 1000 {
                // GetStatus(chan, statusByte)
                statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus);
                statusByte = UniataIsIdle(deviceExtension, statusByte);
                if statusByte == IDE_STATUS_WRONG as UCHAR {
                    break;
                } else if (statusByte & IDE_STATUS_ERROR as UCHAR) != 0 {
                    break;
                } else if (statusByte & IDE_STATUS_BUSY as UCHAR) != 0 {
                    ScsiPortStallExecution(100);
                } else if (statusByte & !(IDE_STATUS_INDEX as UCHAR)) == IDE_STATUS_IDLE as UCHAR {
                    break;
                } else {
                    if UniataIsSATARangeAvailable(deviceExtension, lChannel) != 0 {
                        break;
                    }
                    ScsiPortStallExecution(100);
                }
                i += 1;
            }
        }
        x if x == ATA_WAIT_READY => {
            statusByte = WaitOnBusyLong(chan);
        }
        x if x == ATA_WAIT_BASE_READY => {
            statusByte = WaitOnBaseBusyLong(chan);
        }
        x if x == ATA_IMMEDIATE => {
            // GetStatus(chan, statusByte)
            statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus);
            // In C, `statusByte == (IDLE|ERROR)` breaks out of the switch early,
            // skipping the interrupt-expect / CheckIntr / SUCCESS assignment below.
            let mut early_break = false;
            if (statusByte & IDE_STATUS_ERROR as UCHAR) != 0 {
                if statusByte == (IDE_STATUS_IDLE as UCHAR | IDE_STATUS_ERROR as UCHAR) {
                    early_break = true;
                } else {
                    statusByte &= !(IDE_STATUS_ERROR as UCHAR);
                }
            }
            if !early_break {
                UniataExpectChannelInterrupt(chan, TRUE as BOOLEAN);
                // !!!!!
                interlocked_exchange_i32(
                    &mut (*chan).CheckIntr as *mut LONG,
                    CHECK_INTR_IDLE as LONG,
                );

                statusByte = IDE_STATUS_SUCCESS as UCHAR;
            }
        }
        _ => {}
    }

    statusByte
}

// ---------------------------------------------------------------------------
// AtaCommand — wrapper for AtaCommand48().
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn AtaCommand(
    deviceExtension: PHW_DEVICE_EXTENSION,
    DeviceNumber: ULONG,
    lChannel: ULONG,
    command: UCHAR,
    cylinder: USHORT,
    head: UCHAR,
    sector: UCHAR,
    count: UCHAR,
    feature: UCHAR,
    wait_flags: ULONG,
) -> UCHAR {
    if ((*deviceExtension).HwFlags & UNIATA_AHCI) == 0 {
        AtaCommand48(
            deviceExtension,
            DeviceNumber,
            lChannel,
            command,
            (sector as ULONG | ((cylinder as ULONG) << 8) | (((head & 0x0f) as ULONG) << 24))
                as ULONGLONG,
            count as USHORT,
            feature as USHORT,
            wait_flags,
        )
    } else {
        UniataAhciSendPIOCommand(
            deviceExtension as PVOID,
            lChannel,
            DeviceNumber,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            command,
            (sector as ULONG | ((cylinder as ULONG) << 8) | (((head & 0x0f) as ULONG) << 24))
                as ULONGLONG,
            count as USHORT,
            feature as USHORT,
            0, /* ahci flags */
            wait_flags,
            1000, /* timeout 1 sec */
        )
    }
}

// ---------------------------------------------------------------------------
// AtaPio2Mode
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn AtaPio2Mode(pio: LONG) -> LONG {
    match pio {
        0 => ATA_PIO0 as LONG,
        1 => ATA_PIO1 as LONG,
        2 => ATA_PIO2 as LONG,
        3 => ATA_PIO3 as LONG,
        4 => ATA_PIO4 as LONG,
        5 => ATA_PIO5 as LONG,
        _ => ATA_PIO as LONG,
    }
}

// ---------------------------------------------------------------------------
// AtaPioMode
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn AtaPioMode(ident: PIDENTIFY_DATA) -> LONG {
    if (*ident).PioTimingsValid() != 0 {
        if ((*ident).AdvancedPIOModes() & AdvancedPIOModes_5 as USHORT) != 0 {
            return 5;
        }
        if ((*ident).AdvancedPIOModes() & AdvancedPIOModes_4 as USHORT) != 0 {
            return 4;
        }
        if ((*ident).AdvancedPIOModes() & AdvancedPIOModes_3 as USHORT) != 0 {
            return 3;
        }
    }
    if (*ident).PioCycleTimingMode == 2 {
        return 2;
    }
    if (*ident).PioCycleTimingMode == 1 {
        return 1;
    }
    if (*ident).PioCycleTimingMode == 0 {
        return 0;
    }
    IOMODE_NOT_SPECIFIED as LONG
}

// ---------------------------------------------------------------------------
// AtaWmode
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn AtaWmode(ident: PIDENTIFY_DATA) -> LONG {
    if ((*ident).MultiWordDMASupport() & 0x04) != 0 {
        return 2;
    }
    if ((*ident).MultiWordDMASupport() & 0x02) != 0 {
        return 1;
    }
    if ((*ident).MultiWordDMASupport() & 0x01) != 0 {
        return 0;
    }
    IOMODE_NOT_SPECIFIED as LONG
}

// ---------------------------------------------------------------------------
// AtaUmode
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn AtaUmode(ident: PIDENTIFY_DATA) -> LONG {
    if (*ident).UdmaModesValid() == 0 {
        return IOMODE_NOT_SPECIFIED as LONG;
    }
    if ((*ident).UltraDMASupport() & 0x40) != 0 {
        return 6;
    }
    if ((*ident).UltraDMASupport() & 0x20) != 0 {
        return 5;
    }
    if ((*ident).UltraDMASupport() & 0x10) != 0 {
        return 4;
    }
    if ((*ident).UltraDMASupport() & 0x08) != 0 {
        return 3;
    }
    if ((*ident).UltraDMASupport() & 0x04) != 0 {
        return 2;
    }
    if ((*ident).UltraDMASupport() & 0x02) != 0 {
        return 1;
    }
    if ((*ident).UltraDMASupport() & 0x01) != 0 {
        return 0;
    }
    IOMODE_NOT_SPECIFIED as LONG
}

// ---------------------------------------------------------------------------
// AtaSAmode
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn AtaSAmode(ident: PIDENTIFY_DATA) -> LONG {
    if (*ident).SataCapabilities == 0 || (*ident).SataCapabilities == 0xffff {
        return IOMODE_NOT_SPECIFIED as LONG;
    }
    if ((*ident).SataCapabilities & ATA_SATA_GEN3 as USHORT) != 0 {
        return ATA_SA600 as LONG;
    } else if ((*ident).SataCapabilities & ATA_SATA_GEN2 as USHORT) != 0 {
        return ATA_SA300 as LONG;
    } else if ((*ident).SataCapabilities & ATA_SATA_GEN1 as USHORT) != 0 {
        return ATA_SA150 as LONG;
    }
    IOMODE_NOT_SPECIFIED as LONG
}

// ---------------------------------------------------------------------------
// Local helper: InterlockedExchange (32-bit LONG). The bindgen ddk module does
// not surface a plain InterlockedExchange; implement via a raw atomic swap to
// avoid a link-time dependency on an MSVC-style intrinsic.
// ---------------------------------------------------------------------------
#[inline(always)]
unsafe fn interlocked_exchange_i32(target: *mut LONG, value: LONG) -> LONG {
    let a = &*(target as *const core::sync::atomic::AtomicI32);
    a.swap(value, core::sync::atomic::Ordering::SeqCst)
}
