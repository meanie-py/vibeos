//! id_ata_part3.rs — final contiguous block of id_ata.cpp:
//! the SCSI/IO command path (IdeSendSmartCommand .. AtapiStartIo__) + DriverEntry +
//! registry / adapter-control helpers. Translated per TRANSLATION_MANIFEST.md.
//!
//! Live compile path (matching config.h / id_ata.cpp #defines, same as part2):
//!   - UNIATA_CORE          : NOT defined  (#ifndef UNIATA_CORE bodies live)
//!   - USE_REACTOS_DDK      : DEFINED       (DriverEntry takes the ReactOS branch:
//!                            MajorVersion/MinorVersion/BuildNumber set inline, CrNtInit
//!                            collapsed to a no-op, CPU_num = KeNumberProcessors)
//!   - UNIATA_INIT_CHANGERS : NOT defined   (mechanism-status changer recursion dropped)
//!   - UNIATA_DUMP_ATAPI    : NOT defined
//!   - NAVO_TEST            : NOT defined
//!   - _DEBUG               : NOT defined   (KdPrint*/SetCheckPoint/PrintNtConsole-debug dropped)
//!
//! RESET_COMPLETE_* (local #defines in id_ata.cpp):
//!   RESET_COMPLETE_CURRENT = 0, RESET_COMPLETE_ALL = 1, RESET_COMPLETE_NONE = 2

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code, unused_unsafe)]
#![allow(unused_variables, unused_mut, unused_assignments, unused_parens, unused_labels)]

use crate::types::*;
use crate::regidx::*;
use crate::*;
use ntkrnl_sys::driver_ddk::DbgPrint; // ahci markers
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{TRUE, FALSE, NT_SUCCESS};
use ntkrnl_sys::rtl::{RtlZeroMemory, RtlCopyMemory};
use crate::io_port::{
    AtapiReadPort1, AtapiReadPort2, AtapiReadPort4, AtapiReadPortEx1, AtapiReadPortEx4,
    AtapiWritePort1, AtapiWritePort2, AtapiWritePort4, AtapiWritePortEx1, AtapiWritePortEx4,
    AtapiReadBuffer2, AtapiReadBuffer4, AtapiWriteBuffer2, AtapiWriteBuffer4, UniataNanoSleep,
};
// CRT/registry helpers the ddk bindgen module doesn't carry.
use ntkrnl_sys::driver_ddk::{
    swprintf, wcslen, RTL_QUERY_REGISTRY_DIRECT, RTL_REGISTRY_ABSOLUTE,
};

// REG_DWORD (winnt) — not emitted into either bindgen module for i386.
const REG_DWORD: ULONG = 4;
// NTSTATUS codes not emitted into the driver bindgen module.
const STATUS_DEVICE_DOES_NOT_EXIST: NTSTATUS = 0xC00000C0_u32 as i32;
const STATUS_CONFLICTING_ADDRESSES: NTSTATUS = 0xC0000018u32 as NTSTATUS;

// ---- local #defines from id_ata.cpp (mirror part2) ----
const RESET_COMPLETE_CURRENT: BOOLEAN = 0x00;
const RESET_COMPLETE_ALL: BOOLEAN = 0x01;
const RESET_COMPLETE_NONE: BOOLEAN = 0x02;

// PIO0_TIMING constant; UniataGetPioTiming(LunExt) -> 0 (bm_devs_decl.h override).
const PIO0_TIMING: ULONG = 600;
#[inline(always)]
unsafe fn UniataGetPioTiming(_LunExt: PHW_LU_EXTENSION) -> ULONG { 0 }

// ---- ntdddisk.h SMART constants bindgen dropped (ntdddisk.h:656-748) ----
const SMART_CMD: UCHAR = 0xB0;
const READ_ATTRIBUTES: UCHAR = 0xD0;
const READ_THRESHOLDS: UCHAR = 0xD1;
const RETURN_SMART_STATUS: UCHAR = 0xDA;
const ID_CMD: UCHAR = 0xEC;
const ATAPI_ID_CMD: UCHAR = 0xA1;
const CAP_ATA_ID_CMD: ULONG = 1;
const CAP_ATAPI_ID_CMD: ULONG = 2;
const CAP_SMART_CMD: ULONG = 4;
const READ_ATTRIBUTE_BUFFER_SIZE: ULONG = 512; // ntdddisk.h:726
const IDENTIFY_BUFFER_SIZE: ULONG = 512;        // ntdddisk.h:727

// ---- PCI address packing (pci.h:3600) ----
const PCI_MAX_DEVICES: ULONG = 32;
const PCI_MAX_FUNCTION: ULONG = 8;

// ---- uniata version (uniata_ver.h) ----
const UNIATA_VER_MJ: ULONG = 0;
const UNIATA_VER_MN: ULONG = 47;
const UNIATA_VER_SUB_MJ: ULONG = 2;
const UNIATA_VER_SUB_MN: ULONG = 0;

// ---- WinVer (tools.h) ----
const WinVer_NT: ULONG = 0x0400;
const WinVer_2k: ULONG = 0x0500;

// uniata's OS-version globals. These lived in ros_glue.cpp, the last C++ TU in
// the driver, purely as the data home for three mutable ULONGs; DriverEntry
// overwrites all three from the running NT version anyway, so the C file was
// carrying nothing but their storage. Defining them here retired that TU and
// with it the whole ungranted UniATA C tree (2026-09-20) -- see the PROVENANCE
// note in lib.rs. The initialisers are ros_glue.cpp's verbatim: 5.2.0.
#[no_mangle]
pub static mut MajorVersion: ULONG = 0x05;
#[no_mangle]
pub static mut MinorVersion: ULONG = 0x02;
#[no_mangle]
pub static mut BuildNumber: ULONG = 0x0;

// CRT helpers used by the registry/console paths (driver_ddk + ntkrnl-sys provides
// swprintf/wcslen; _strnicmp & strncpy come from the C runtime libs).
extern "C" {
    fn _strnicmp(a: PCCHAR, b: PCCHAR, n: usize) -> core::ffi::c_int;
    fn strncpy(dst: *mut CHAR, src: *const CHAR, n: usize) -> *mut CHAR;
    // vibeos_vsnprintf — the `_vsnprintf` forwarder in drivers/storage/ide/uniata/
    // rust_varargs_shim.c. Stable Rust cannot name a `va_list` (`c_variadic`,
    // rust-lang#44930), so it holds a POINTER to the C frame's va_list and the shim
    // derefs it into the real ntoskrnl `_vsnprintf`. Buffer + NUL termination stay here.
    fn vibeos_vsnprintf(
        buf: *mut CHAR,
        n: usize,
        fmt: *const CHAR,
        ap: *mut core::ffi::c_void,
    ) -> core::ffi::c_int;
}

// Wide format string L"%2.2x" used by EncodeVendorStr's swprintf.
static WFMT_HEX2: [u16; 6] = [b'%' as u16, b'2' as u16, b'.' as u16, b'2' as u16, b'x' as u16, 0];

#[inline(always)]
unsafe fn WinVer_Id() -> ULONG {
    ((MajorVersion << 8) | MinorVersion) as ULONG
}

// ---- small numeric helpers (C min/max on these integer types) ----
#[inline(always)]
fn min_u32(a: ULONG, b: ULONG) -> ULONG { if a < b { a } else { b } }
#[inline(always)]
fn max_u32(a: ULONG, b: ULONG) -> ULONG { if a > b { a } else { b } }

// ---- ATA_REQ field accessors (bindgen buries the real fields under nested anon
//      unions: _ATA_REQ.__bindgen_anon_1(union)->bindgen_ty_1.__bindgen_anon_1(struct)) ----
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

// ===========================================================================
//  IdeReadWrite  (compiles regardless of UNIATA_CORE)
// ===========================================================================
pub unsafe extern "system" fn IdeReadWrite(
    HwDeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
    CmdAction: ULONG,
) -> ULONG {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let lChannel: UCHAR = (*Srb).PathId; // GET_CHANNEL(Srb)
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let LunExt: PHW_LU_EXTENSION;
    let AtaReq = (*Srb).SrbExtension as PATA_REQ;
    let DeviceNumber: UCHAR = (*Srb).TargetId; // GET_CDEV(Srb)
    let mut startingSector: ULONGLONG = 0;
    let mut max_bcount: ULONG = 0;
    let mut wordCount: ULONG = 0;
    let mut statusByte: UCHAR;
    let statusByte2: UCHAR;
    let cmd: UCHAR;
    let mut lba: ULONGLONG;
    let mut use_dma: BOOLEAN = FALSE as BOOLEAN;
    let fis_size: ULONG;

    (*ata_req_core(AtaReq)).Flags |= REQ_FLAG_REORDERABLE_CMD as UCHAR;
    LunExt = (*chan).lun[DeviceNumber as usize];

    if (CmdAction & CMD_ACTION_PREPARE != 0)
        && ((*ata_req_core(AtaReq)).ReqState != REQ_STATE_READY_TO_TRANSFER as UCHAR)
    {
        if (*LunExt).opt_ReadOnly != 0 && ((*Srb).SrbFlags & SRB_FLAGS_DATA_OUT != 0) {
            if (*LunExt).opt_ReadOnly == 1 {
                return SRB_STATUS_ERROR as ULONG;
            } else {
                return SRB_STATUS_SUCCESS as ULONG;
            }
        }

        // Set data buffer pointer and words left.
        (*ata_req_core(AtaReq)).Flags &= !(REQ_FLAG_DMA_OPERATION as UCHAR);

        if (*ata_req_core(AtaReq)).WordsTransfered != 0 {
            (*ata_req_core(AtaReq)).DataBuffer =
                ((*Srb).DataBuffer as PUSHORT).add((*ata_req_core(AtaReq)).WordsTransfered as usize);
            startingSector = UniAtaCalculateLBARegsBack(LunExt, (*ata_req_core(AtaReq)).lba as ULONGLONG)
                + (*ata_req_core(AtaReq)).bcount as ULONGLONG;
            (*ata_req_core(AtaReq)).bcount = ((*ata_req_core(AtaReq)).TransferLength
                - (*ata_req_core(AtaReq)).WordsTransfered * 2
                + DEV_BSIZE
                - 1)
                / DEV_BSIZE;
        } else {
            (*ata_req_core(AtaReq)).DataBuffer = (*Srb).DataBuffer as PUSHORT;
            (*ata_req_core(AtaReq)).TransferLength = (*Srb).DataTransferLength;
            // Set up 1st block.
            let cdb = (*Srb).Cdb.as_ptr() as PCDB;
            match (*Srb).Cdb[0] as ULONG {
                x if x == SCSIOP_WRITE => {
                    // FALLTHROUGH to SCSIOP_READ
                    read_cdb10(&mut startingSector, AtaReq, cdb);
                }
                x if x == SCSIOP_READ => {
                    read_cdb10(&mut startingSector, AtaReq, cdb);
                }
                x if x == SCSIOP_WRITE12 => {
                    read_cdb12(&mut startingSector, AtaReq, cdb);
                }
                x if x == SCSIOP_READ12 => {
                    read_cdb12(&mut startingSector, AtaReq, cdb);
                }
                x if x == SCSIOP_READ16 || x == SCSIOP_WRITE16 => {
                    // MOV_QD_SWP(startingSector, CDB16READWRITE.LBA)
                    startingSector = qd_swap(
                        (*cdb).CDB16READWRITE.as_ref().LBA.as_ptr(),
                    );
                    (*ata_req_core(AtaReq)).bcount = dd_swap(
                        (*cdb)
                            .CDB16READWRITE
                            .as_ref()
                            .NumOfBlocks
                            .as_ptr(),
                    );
                }
                _ => {}
            }
        }
        lba = UniAtaCalculateLBARegs(LunExt, startingSector, &mut max_bcount);

        if max_bcount != 0 {
            (*ata_req_core(AtaReq)).bcount = min_u32((*ata_req_core(AtaReq)).bcount, max_bcount);
        }
        (*ata_req_core(AtaReq)).WordsLeft = min_u32(
            (*ata_req_core(AtaReq)).TransferLength - (*ata_req_core(AtaReq)).WordsTransfered * 2,
            (*ata_req_core(AtaReq)).bcount * DEV_BSIZE,
        ) / 2;

        (*ata_req_core(AtaReq)).lba = lba as LONGLONG;
        if (*LunExt).errRetry != 0
            && lba == (*LunExt).errLastLba
            && (*ata_req_core(AtaReq)).bcount == (*LunExt).errBCount
        {
            if ((*ata_req_core(AtaReq)).retry as ULONG) < MAX_RETRIES {
                (*ata_req_core(AtaReq)).retry = (*LunExt).errRetry;
                (*ata_req_core(AtaReq)).Flags |= REQ_FLAG_FORCE_DOWNRATE as UCHAR;
            }
            (*LunExt).errRetry = 0;
        }

        // we cannot reinit Dma until previous request is completed
        if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
            UniataAhciSetupCmdPtr(AtaReq);
            if AtapiDmaSetup(
                HwDeviceExtension,
                DeviceNumber as ULONG,
                lChannel as ULONG,
                Srb,
                (*ata_req_core(AtaReq)).DataBuffer as PUCHAR,
                (*ata_req_core(AtaReq)).bcount * DEV_BSIZE,
            ) == 0
            {
                return SRB_STATUS_ERROR as ULONG;
            }
        } else if (*LunExt).LimitedTransferMode >= ATA_DMA as UCHAR {
            use_dma = TRUE as BOOLEAN;
            // this sets REQ_FLAG_DMA_OPERATION in AtaReq->Flags on success
            if AtapiDmaSetup(
                HwDeviceExtension,
                DeviceNumber as ULONG,
                lChannel as ULONG,
                Srb,
                (*ata_req_core(AtaReq)).DataBuffer as PUCHAR,
                (*ata_req_core(AtaReq)).bcount * DEV_BSIZE,
            ) == 0
            {
                use_dma = FALSE as BOOLEAN;
            }
        }

        if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
            let cmd_ptr = (*ata_req_ahci(AtaReq)).ahci_cmd_ptr;
            RtlZeroMemory(
                (*cmd_ptr).cfis.as_mut_ptr() as PVOID,
                core::mem::size_of_val(&(*ata_req_ahci_cmd0(AtaReq)).cfis),
            );

            fis_size = UniataAhciSetupFIS_H2D(
                deviceExtension,
                DeviceNumber as ULONG,
                lChannel as ULONG,
                (*cmd_ptr).cfis.as_mut_ptr(),
                if (*ata_req_core(AtaReq)).Flags as ULONG & REQ_FLAG_READ != 0 {
                    IDE_COMMAND_READ_DMA as UCHAR
                } else {
                    IDE_COMMAND_WRITE_DMA as UCHAR
                },
                lba,
                (*ata_req_core(AtaReq)).bcount as USHORT,
                0,
            );

            if fis_size == 0 {
                return SRB_STATUS_ERROR as ULONG;
            }

            (*ata_req_ahci(AtaReq)).io_cmd_flags = UniAtaAhciAdjustIoFlags(
                0,
                if (*ata_req_core(AtaReq)).Flags as ULONG & REQ_FLAG_READ != 0 {
                    0
                } else {
                    ATA_AHCI_CMD_WRITE as USHORT
                },
                fis_size,
                DeviceNumber as ULONG,
            ) as USHORT;
        }

        (*ata_req_core(AtaReq)).ReqState = REQ_STATE_READY_TO_TRANSFER as UCHAR;
    } else {
        // exec_only
        lba = (*ata_req_core(AtaReq)).lba as ULONGLONG;

        if (*ata_req_core(AtaReq)).Flags as ULONG & REQ_FLAG_DMA_OPERATION != 0 {
            use_dma = TRUE as BOOLEAN;
        }
    }
    if CmdAction & CMD_ACTION_EXEC == 0 {
        return SRB_STATUS_PENDING as ULONG;
    }

    // if this is queued request, reinit DMA and check if DMA mode is still available
    AtapiDmaReinit(deviceExtension, LunExt, AtaReq);
    if (*LunExt).TransferMode >= ATA_DMA as UCHAR {
        use_dma = TRUE as BOOLEAN;
    } else {
        (*ata_req_core(AtaReq)).Flags &= !(REQ_FLAG_DMA_OPERATION as UCHAR);
        use_dma = FALSE as BOOLEAN;
    }

    // Check if write request.
    if (*Srb).SrbFlags & SRB_FLAGS_DATA_IN != 0 {
        // Prepare read command.
        if use_dma != 0 {
            cmd = IDE_COMMAND_READ_DMA as UCHAR;
        } else if (*LunExt).MaximumBlockXfer != 0 {
            cmd = IDE_COMMAND_READ_MULTIPLE as UCHAR;
        } else {
            cmd = IDE_COMMAND_READ as UCHAR;
        }
    } else {
        // Prepare write command.
        if use_dma != 0 {
            wordCount = (*ata_req_core(AtaReq)).bcount * DEV_BSIZE / 2;
            cmd = IDE_COMMAND_WRITE_DMA as UCHAR;
        } else if (*LunExt).MaximumBlockXfer != 0 {
            wordCount = DEV_BSIZE / 2 * (*LunExt).MaximumBlockXfer as ULONG;

            if (*ata_req_core(AtaReq)).WordsLeft < wordCount {
                // Transfer only words requested.
                wordCount = (*ata_req_core(AtaReq)).WordsLeft;
            }
            cmd = IDE_COMMAND_WRITE_MULTIPLE as UCHAR;
        } else {
            wordCount = DEV_BSIZE / 2;
            cmd = IDE_COMMAND_WRITE as UCHAR;
        }
    }

    if use_dma != 0 {
        (*chan).ChannelCtrlFlags |= CTRFLAGS_DMA_OPERATION;
    } else {
        (*chan).ChannelCtrlFlags &= !CTRFLAGS_DMA_OPERATION;
    }

    if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
        // AHCI doesn't distinguish DMA and PIO
        UniataAhciBeginTransaction(HwDeviceExtension, lChannel as ULONG, DeviceNumber as ULONG, Srb);
        UniataExpectChannelInterrupt(chan, TRUE as BOOLEAN);
        InterlockedExchange(&mut (*chan).CheckIntr, CHECK_INTR_IDLE as LONG);
        return SRB_STATUS_PENDING as ULONG;
    }

    if ((*Srb).SrbFlags & SRB_FLAGS_DATA_IN != 0) || use_dma != 0 {
        if use_dma != 0 {
            AtapiDmaDBPreSync(HwDeviceExtension, chan, Srb);
            if g_opt_BochsDmaReadWorkaround != 0 && ((*Srb).SrbFlags & SRB_FLAGS_DATA_IN != 0) {
                AtapiDmaStart(HwDeviceExtension, DeviceNumber as ULONG, lChannel as ULONG, Srb);
            }
        }
        let sb2 = AtaCommand48(
            deviceExtension,
            DeviceNumber as ULONG,
            lChannel as ULONG,
            cmd,
            lba,
            (*ata_req_core(AtaReq)).bcount as USHORT,
            0,
            ATA_IMMEDIATE,
        );
        if sb2 & IDE_STATUS_ERROR as UCHAR != 0 {
            // We cannot handle errors here (some devices keep ERR from previous cmd); fail.
            statusByte = AtapiReadPort1(chan, IDX_IO1_i_Error);
            let _ = statusByte;
            return SRB_STATUS_ERROR as ULONG;
        }
        if use_dma != 0 {
            if g_opt_BochsDmaReadWorkaround == 0 || ((*Srb).SrbFlags & SRB_FLAGS_DATA_IN == 0) {
                AtapiDmaStart(HwDeviceExtension, DeviceNumber as ULONG, lChannel as ULONG, Srb);
            }
        }
        return SRB_STATUS_PENDING as ULONG;
    }

    statusByte = AtaCommand48(
        deviceExtension,
        DeviceNumber as ULONG,
        lChannel as ULONG,
        cmd,
        lba,
        (*ata_req_core(AtaReq)).bcount as USHORT,
        0,
        ATA_WAIT_INTR,
    );

    if (statusByte & IDE_STATUS_DRQ as UCHAR == 0) || statusByte == IDE_STATUS_WRONG as UCHAR {
        (*ata_req_core(AtaReq)).WordsLeft = 0;

        // Clear interrupt expecting flag.
        UniataExpectChannelInterrupt(chan, FALSE as BOOLEAN);
        InterlockedExchange(&mut (*chan).CheckIntr, CHECK_INTR_IDLE as LONG);

        // Clear current SRB.
        UniataRemoveRequest(chan, Srb);

        return if statusByte == IDE_STATUS_WRONG as UCHAR {
            SRB_STATUS_ERROR as ULONG
        } else {
            SRB_STATUS_TIMEOUT as ULONG
        };
    }

    UniataExpectChannelInterrupt(chan, TRUE as BOOLEAN);
    InterlockedExchange(&mut (*chan).CheckIntr, CHECK_INTR_IDLE as LONG);

    // Write next DEV_BSIZE/2*N words.
    if (*LunExt).DeviceFlags & DFLAGS_DWORDIO_ENABLED == 0 || (wordCount & 1 != 0) {
        AtapiWriteBuffer2(
            chan,
            IDX_IO1_o_Data,
            (*ata_req_core(AtaReq)).DataBuffer as PVOID,
            wordCount,
            UniataGetPioTiming(LunExt),
        );
    } else {
        AtapiWriteBuffer4(
            chan,
            IDX_IO1_o_Data,
            (*ata_req_core(AtaReq)).DataBuffer as PVOID,
            wordCount / 2,
            UniataGetPioTiming(LunExt),
        );
    }

    // Adjust buffer address and words left count.
    (*ata_req_core(AtaReq)).WordsLeft -= wordCount;
    (*ata_req_core(AtaReq)).DataBuffer = (*ata_req_core(AtaReq)).DataBuffer.add(wordCount as usize);
    (*ata_req_core(AtaReq)).WordsTransfered += wordCount;

    SRB_STATUS_PENDING as ULONG
}

// MOV_DD_SWP(startingSector, CDB10.LBA); MOV_SWP_DW2DD(AtaReq->bcount, CDB10.TransferBlocks)
unsafe fn read_cdb10(startingSector: &mut ULONGLONG, AtaReq: PATA_REQ, cdb: PCDB) {
    *startingSector = dd_swap((*cdb).CDB10.as_ref().LBA.as_ptr()) as ULONGLONG;
    (*ata_req_core(AtaReq)).bcount =
        RtlUshortByteSwap(*((*cdb).CDB10.as_ref().TransferBlocks.as_ptr() as *const USHORT)) as ULONG;
}
// MOV_DD_SWP(startingSector, CDB12READWRITE.LBA); MOV_DD_SWP(bcount, NumOfBlocks)
unsafe fn read_cdb12(startingSector: &mut ULONGLONG, AtaReq: PATA_REQ, cdb: PCDB) {
    *startingSector =
        dd_swap((*cdb).CDB12READWRITE.as_ref().LBA.as_ptr()) as ULONGLONG;
    (*ata_req_core(AtaReq)).bcount = dd_swap(
        (*cdb)
            .CDB12READWRITE
            .as_ref()
            .NumOfBlocks
            .as_ptr(),
    );
}
// MOV_DD_SWP target read: byte-swap a 4-byte big-endian field
#[inline(always)]
unsafe fn dd_swap(p: *const UCHAR) -> ULONG {
    RtlUlongByteSwap(*(p as *const ULONG))
}
// MOV_QD_SWP target read: an 8-byte big-endian field (two dwords swapped & reordered)
#[inline(always)]
unsafe fn qd_swap(p: *const UCHAR) -> ULONGLONG {
    let lo = RtlUlongByteSwap(*((p as *const ULONG).add(1))) as ULONGLONG;
    let hi = RtlUlongByteSwap(*(p as *const ULONG)) as ULONGLONG;
    lo | (hi << 32)
}

// ===========================================================================
//  IdeVerify  (#ifndef UNIATA_CORE)
// ===========================================================================
pub unsafe extern "system" fn IdeVerify(
    HwDeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
) -> ULONG {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let lChannel: UCHAR = (*Srb).PathId;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let AtaReq = (*Srb).SrbExtension as PATA_REQ;
    let LunExt: PHW_LU_EXTENSION;
    let DeviceNumber: ULONG = (*Srb).TargetId as ULONG;
    let statusByte: UCHAR;
    let mut startingSector: ULONGLONG = 0;
    let mut max_bcount: ULONG = 0;
    let mut sectors: ULONGLONG;
    let endSector: ULONGLONG;
    let mut sectorCount: ULONG = 0;
    let lba: ULONGLONG;

    LunExt = (*chan).lun[DeviceNumber as usize];
    // Drive has these number sectors.
    sectors = (*LunExt).NumOfSectors as ULONG as ULONGLONG;
    if sectors == 0 {
        sectors = (*LunExt).IdentifyData.SectorsPerTrack as ULONGLONG
            * (*LunExt).IdentifyData.NumberOfHeads as ULONGLONG
            * (*LunExt).IdentifyData.NumberOfCylinders as ULONGLONG;
    }

    // Get starting sector number from CDB.
    let cdb = (*Srb).Cdb.as_ptr() as PCDB;
    match (*Srb).Cdb[0] as ULONG {
        x if x == SCSIOP_VERIFY => {
            startingSector = dd_swap((*cdb).CDB10.as_ref().LBA.as_ptr()) as ULONGLONG;
            sectorCount =
                RtlUshortByteSwap(*((*cdb).CDB10.as_ref().TransferBlocks.as_ptr() as *const USHORT)) as ULONG;
        }
        x if x == SCSIOP_VERIFY12 => {
            startingSector =
                dd_swap((*cdb).CDB12READWRITE.as_ref().LBA.as_ptr()) as ULONGLONG;
            sectorCount = dd_swap(
                (*cdb).CDB12READWRITE.as_ref().NumOfBlocks.as_ptr(),
            );
        }
        x if x == SCSIOP_VERIFY16 => {
            startingSector = qd_swap((*cdb).CDB16READWRITE.as_ref().LBA.as_ptr());
            sectorCount = dd_swap(
                (*cdb).CDB16READWRITE.as_ref().NumOfBlocks.as_ptr(),
            );
        }
        _ => {}
    }

    endSector = startingSector + sectorCount as ULONGLONG;

    if endSector > sectors {
        // Too big, round down.
        sectorCount = (sectors - startingSector - 1) as ULONG;
    } else {
        // Round up to next block.
        if sectorCount > 0xFF {
            sectorCount = 0xFF;
        }
    }

    // Set data buffer pointer and words left.
    (*ata_req_core(AtaReq)).DataBuffer = (*Srb).DataBuffer as PUSHORT;
    (*ata_req_core(AtaReq)).WordsLeft = (*Srb).DataTransferLength / 2;

    // Indicate expecting an interrupt.
    InterlockedExchange(&mut (*chan).CheckIntr, CHECK_INTR_IDLE as LONG);

    lba = UniAtaCalculateLBARegs(LunExt, startingSector, &mut max_bcount);

    statusByte = AtaCommand48(
        deviceExtension,
        (*LunExt).Lun as ULONG,
        (*Srb).PathId as ULONG,
        IDE_COMMAND_VERIFY as UCHAR,
        lba,
        sectorCount as USHORT,
        0,
        ATA_IMMEDIATE,
    );

    if statusByte & IDE_STATUS_ERROR as UCHAR == 0 {
        UniataExpectChannelInterrupt(chan, TRUE as BOOLEAN);
        return SRB_STATUS_PENDING as ULONG;
    }
    SRB_STATUS_ERROR as ULONG
}

// ===========================================================================
//  AtapiSendCommand — send ATAPI packet command to device.
//  goto restructuring: `retry:` / `make_reset:` modeled as a `'retry` loop with a
//  `do_reset` flag; the GetLba/GetLba2 fallthrough is inlined.
// ===========================================================================
pub unsafe extern "system" fn AtapiSendCommand(
    HwDeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
    CmdAction: ULONG,
) -> ULONG {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let lChannel: UCHAR = (*Srb).PathId;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let AtaReq = (*Srb).SrbExtension as PATA_REQ;
    let LunExt: PHW_LU_EXTENSION;
    let DeviceNumber: ULONG = (*Srb).TargetId as ULONG;
    let mut flags: ULONG;
    let mut statusByte: UCHAR;
    let mut statusByte0: UCHAR = 0;
    let mut byteCountLow: UCHAR;
    let mut byteCountHigh: UCHAR;
    let mut interruptReason: UCHAR;
    let mut use_dma: BOOLEAN = FALSE as BOOLEAN;
    let mut dma_reinited: BOOLEAN = FALSE as BOOLEAN;
    let mut retried: BOOLEAN = FALSE as BOOLEAN;
    let mut fis_size: ULONG;
    let mut i: ULONG;
    let mut FeatureReg: UCHAR = 0;

    LunExt = (*chan).lun[DeviceNumber as usize];

    if (*ata_req_core(AtaReq)).ReqState < REQ_STATE_PREPARE_TO_TRANSFER as UCHAR {
        (*ata_req_core(AtaReq)).ReqState = REQ_STATE_PREPARE_TO_TRANSFER as UCHAR;
    }

    if CmdAction == CMD_ACTION_PREPARE {
        match (*Srb).Cdb[0] as ULONG {
            x if x == SCSIOP_RECEIVE
                || x == SCSIOP_SEND
                || x == SCSIOP_READ
                || x == SCSIOP_WRITE
                || x == SCSIOP_READ12
                || x == SCSIOP_WRITE12
                || x == SCSIOP_READ16
                || x == SCSIOP_WRITE16 =>
            {
                // all right
            }
            x if x == SCSIOP_READ_CD || x == SCSIOP_READ_CD_MSF => {
                if (*deviceExtension).opt_AtapiDmaRawRead != 0 {
                    // all right
                } else {
                    return SRB_STATUS_BUSY as ULONG;
                }
            }
            _ => {
                return SRB_STATUS_BUSY as ULONG;
            }
        }
        // UNIATA_INIT_CHANGERS not defined -> changer pre-check dropped.
    }
    // UNIATA_INIT_CHANGERS not defined -> mechanism-status recursion dropped.

    if (CmdAction & CMD_ACTION_PREPARE != 0)
        && ((*ata_req_core(AtaReq)).ReqState != REQ_STATE_READY_TO_TRANSFER as UCHAR)
    {
        if (*LunExt).IdentifyData.AtapiCmdSize() == 0 && ((*Srb).CdbLength > 12) {
            return SRB_STATUS_INVALID_REQUEST as ULONG;
        }

        // Set data buffer pointer and words left.
        (*ata_req_core(AtaReq)).DataBuffer = (*Srb).DataBuffer as PUSHORT;
        (*ata_req_core(AtaReq)).WordsLeft = (*Srb).DataTransferLength / 2;
        (*ata_req_core(AtaReq)).TransferLength = (*Srb).DataTransferLength;
        (*ata_req_core(AtaReq)).Flags &= !(REQ_FLAG_DMA_OPERATION as UCHAR);
        // reset this to force PRD init. May be already setup by recursive SRB
        (*ata_req_core(AtaReq)).dma_entries = 0;

        let cdb = (*Srb).Cdb.as_ptr() as PCDB;
        // check if reorderable
        match (*Srb).Cdb[0] as ULONG {
            x if x == SCSIOP_READ16 || x == SCSIOP_WRITE16 => {
                (*ata_req_core(AtaReq)).bcount =
                    dd_swap((*cdb).CDB16READWRITE.as_ref().NumOfBlocks.as_ptr());
                (*ata_req_core(AtaReq)).lba =
                    qd_swap((*cdb).CDB16READWRITE.as_ref().LBA.as_ptr()) as LONGLONG;
                // GetLba2
                atapi_set_rw_flags(AtaReq, Srb);
            }
            x if x == SCSIOP_READ12 || x == SCSIOP_WRITE12 => {
                (*ata_req_core(AtaReq)).bcount =
                    dd_swap((*cdb).CDB12READWRITE.as_ref().NumOfBlocks.as_ptr());
                // GetLba
                (*ata_req_core(AtaReq)).lba =
                    dd_swap((*cdb).CDB12READWRITE.as_ref().LBA.as_ptr()) as LONGLONG;
                atapi_set_rw_flags(AtaReq, Srb);
            }
            x if x == SCSIOP_READ || x == SCSIOP_WRITE => {
                (*ata_req_core(AtaReq)).bcount =
                    RtlUshortByteSwap(*((*cdb).CDB10.as_ref().TransferBlocks.as_ptr() as *const USHORT))
                        as ULONG;
                // GetLba
                (*ata_req_core(AtaReq)).lba =
                    dd_swap((*cdb).CDB10.as_ref().LBA.as_ptr()) as LONGLONG;
                atapi_set_rw_flags(AtaReq, Srb);
            }
            _ => {
                (*ata_req_core(AtaReq)).Flags &= !(REQ_FLAG_RW_MASK as UCHAR);
                if (*ata_req_core(AtaReq)).TransferLength == 0 {
                    // assume 0-transfer
                } else if (*Srb).SrbFlags & SRB_FLAGS_DATA_OUT != 0 {
                    (*ata_req_core(AtaReq)).Flags |= REQ_FLAG_WRITE as UCHAR;
                } else if (*Srb).SrbFlags & SRB_FLAGS_DATA_IN != 0 {
                    (*ata_req_core(AtaReq)).Flags |= REQ_FLAG_READ as UCHAR;
                }
            }
        }

        // check if DMA read/write. The C control-flow with several gotos
        // (call_dma_setup / setup_dma) is flattened with a `do_setup_dma` flag.
        let mut do_setup_dma = false;
        if g_opt_AtapiNoDma != 0 {
            use_dma = FALSE as BOOLEAN;
        } else if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
            use_dma = TRUE as BOOLEAN;
            do_setup_dma = true;
        } else if (*Srb).Cdb[0] as ULONG == SCSIOP_REQUEST_SENSE {
            // no DMA setup
        } else if (*ata_req_core(AtaReq)).TransferLength != 0
            && ((*ata_req_core(AtaReq)).TransferLength & 0x0f == 0)
        {
            // try use DMA if TransferLength is 16-byte aligned
            let mut call_dma_setup = false;
            match (*Srb).Cdb[0] as ULONG {
                x if x == SCSIOP_WRITE || x == SCSIOP_WRITE12 || x == SCSIOP_WRITE16 || x == SCSIOP_SEND =>
                {
                    if (*chan).ChannelCtrlFlags & CTRFLAGS_DMA_RO != 0 {
                        // break (no dma)
                    } else if (*deviceExtension).opt_AtapiDmaReadWrite != 0 {
                        call_dma_setup = true;
                    }
                }
                x if x == SCSIOP_RECEIVE || x == SCSIOP_READ || x == SCSIOP_READ12 || x == SCSIOP_READ16 =>
                {
                    if (*deviceExtension).opt_AtapiDmaReadWrite != 0 {
                        call_dma_setup = true;
                    }
                }
                x if x == SCSIOP_READ_CD || x == SCSIOP_READ_CD_MSF => {
                    if (*deviceExtension).opt_AtapiDmaRawRead != 0 {
                        call_dma_setup = true;
                    }
                }
                _ => {
                    if (*deviceExtension).opt_AtapiDmaControlCmd != 0 {
                        if (*Srb).SrbFlags & SRB_FLAGS_DATA_IN != 0 {
                            use_dma = TRUE as BOOLEAN; // read operation
                        } else {
                            // write operation
                            if (*chan).ChannelCtrlFlags & CTRFLAGS_DMA_RO != 0 {
                                use_dma = FALSE as BOOLEAN;
                            } else {
                                use_dma = TRUE as BOOLEAN;
                            }
                        }
                    }
                }
            }
            if call_dma_setup {
                if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
                    use_dma = TRUE as BOOLEAN;
                } else if AtapiDmaSetup(
                    HwDeviceExtension,
                    DeviceNumber,
                    lChannel as ULONG,
                    Srb,
                    (*ata_req_core(AtaReq)).DataBuffer as PUCHAR,
                    (*Srb).DataTransferLength,
                ) != 0
                {
                    use_dma = TRUE as BOOLEAN;
                }
            }
            do_setup_dma = true;
        } else {
            // zero/unaligned transfer -> no DMA setup
        }

        if do_setup_dma && use_dma != 0 {
            if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
                // use_dma already TRUE
            } else if AtapiDmaSetup(
                HwDeviceExtension,
                DeviceNumber,
                lChannel as ULONG,
                Srb,
                (*ata_req_core(AtaReq)).DataBuffer as PUCHAR,
                (*Srb).DataTransferLength,
            ) == 0
            {
                use_dma = FALSE as BOOLEAN;
            }
        }

        if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
            UniataAhciSetupCmdPtr(AtaReq);

            if (*Srb).DataTransferLength == 0 {
                use_dma = FALSE as BOOLEAN;
            } else if AtapiDmaSetup(
                HwDeviceExtension,
                DeviceNumber,
                lChannel as ULONG,
                Srb,
                (*ata_req_core(AtaReq)).DataBuffer as PUCHAR,
                (*Srb).DataTransferLength,
            ) == 0
            {
                return SRB_STATUS_ERROR as ULONG;
            }
            if use_dma == 0 {
                (*ata_req_core(AtaReq)).Flags &= !(REQ_FLAG_DMA_OPERATION as UCHAR);
            } else {
                FeatureReg |= ATA_F_DMA as UCHAR;
                if (*LunExt).IdentifyData.__bindgen_anon_1.AtapiDMA.as_ref().DMADirRequired() != 0 {
                    if (*Srb).SrbFlags & SRB_FLAGS_DATA_IN != 0 {
                        FeatureReg |= ATA_F_DMAREAD as UCHAR;
                    }
                }
            }

            let cmd_ptr = (*ata_req_ahci(AtaReq)).ahci_cmd_ptr;
            RtlCopyMemory(
                (*cmd_ptr).acmd.as_mut_ptr() as PVOID,
                (*Srb).Cdb.as_ptr() as PVOID,
                (*Srb).CdbLength as usize,
            );

            fis_size = UniataAhciSetupFIS_H2D(
                deviceExtension,
                DeviceNumber,
                lChannel as ULONG,
                (*cmd_ptr).cfis.as_mut_ptr(),
                IDE_COMMAND_ATAPI_PACKET as UCHAR,
                0,
                if (*Srb).DataTransferLength >= 0x10000 {
                    0xffff
                } else {
                    (*Srb).DataTransferLength as USHORT
                },
                FeatureReg as USHORT,
            );

            if fis_size == 0 {
                return SRB_STATUS_ERROR as ULONG;
            }

            (*ata_req_ahci(AtaReq)).io_cmd_flags = UniAtaAhciAdjustIoFlags(
                0,
                ((if (*Srb).DataTransferLength != 0 && ((*Srb).SrbFlags & SRB_FLAGS_DATA_OUT != 0) {
                    ATA_AHCI_CMD_WRITE
                } else {
                    0
                }) | (ATA_AHCI_CMD_ATAPI | ATA_AHCI_CMD_PREFETCH)) as USHORT,
                fis_size,
                DeviceNumber,
            ) as USHORT;
        }
    } else {
        if (*ata_req_core(AtaReq)).Flags as ULONG & REQ_FLAG_DMA_OPERATION != 0 {
            // queued request -> reinit DMA, check if DMA still available
            AtapiDmaReinit(deviceExtension, LunExt, AtaReq);
            if (*LunExt).TransferMode >= ATA_DMA as UCHAR {
                use_dma = TRUE as BOOLEAN;
            } else {
                (*ata_req_core(AtaReq)).Flags &= !(REQ_FLAG_DMA_OPERATION as UCHAR);
                use_dma = FALSE as BOOLEAN;
            }
            dma_reinited = TRUE as BOOLEAN;
        }
    }

    if CmdAction & CMD_ACTION_EXEC == 0 {
        return SRB_STATUS_PENDING as ULONG;
    }

    if ((*Srb).Cdb[0] as ULONG == SCSIOP_REQUEST_SENSE)
        && ((*deviceExtension).HwFlags & UNIATA_SATA == 0)
    {
        use_dma = FALSE as BOOLEAN;
        (*ata_req_core(AtaReq)).Flags &= !(REQ_FLAG_DMA_OPERATION as UCHAR);
        AtapiDmaReinit(deviceExtension, LunExt, AtaReq);
    }
    if (*ata_req_core(AtaReq)).TransferLength != 0 {
        if dma_reinited == 0 {
            AtapiDmaReinit(deviceExtension, LunExt, AtaReq);
            if (*LunExt).TransferMode >= ATA_DMA as UCHAR {
                use_dma = TRUE as BOOLEAN;
            } else {
                (*ata_req_core(AtaReq)).Flags &= !(REQ_FLAG_DMA_OPERATION as UCHAR);
                use_dma = FALSE as BOOLEAN;
            }
        }
    } else {
        use_dma = FALSE as BOOLEAN;
        (*ata_req_core(AtaReq)).Flags &= !(REQ_FLAG_DMA_OPERATION as UCHAR);
        if (*deviceExtension).opt_AtapiDmaZeroTransfer == 0
            && ((*deviceExtension).HwFlags & UNIATA_SATA == 0)
        {
            AtapiDmaReinit(deviceExtension, LunExt, AtaReq);
        }
    }

    // Make sure command is to ATAPI device.
    flags = (*LunExt).DeviceFlags;
    if flags & (DFLAGS_SANYO_ATAPI_CHANGER | DFLAGS_ATAPI_CHANGER) != 0 {
        if ((*Srb).Lun as ULONG) > ((*LunExt).DiscsPresent as ULONG - 1) {
            (*ata_req_core(AtaReq)).ReqState = REQ_STATE_TRANSFER_COMPLETE as UCHAR;
            return SRB_STATUS_SELECTION_TIMEOUT as ULONG;
        }
    } else if (*Srb).Lun > 0 {
        (*ata_req_core(AtaReq)).ReqState = REQ_STATE_TRANSFER_COMPLETE as UCHAR;
        return SRB_STATUS_SELECTION_TIMEOUT as ULONG;
    }

    if flags & DFLAGS_ATAPI_DEVICE == 0 {
        (*ata_req_core(AtaReq)).ReqState = REQ_STATE_TRANSFER_COMPLETE as UCHAR;
        return SRB_STATUS_SELECTION_TIMEOUT as ULONG;
    }

    // ---- retry: / make_reset: loop ----
    'retry: loop {
        let mut do_reset = false;
        // Select device 0 or 1. Or more for PM
        SelectDrive(chan, DeviceNumber);

        // Verify that controller is ready for next command.
        statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus); // GetStatus

        if statusByte == IDE_STATUS_WRONG as UCHAR {
            do_reset = true;
        } else {
            if statusByte & IDE_STATUS_BUSY as UCHAR != 0 {
                if statusByte & IDE_STATUS_DSC as UCHAR != 0 {
                    // DSC on entry, try exec
                } else {
                    do_reset = true;
                }
            }
            if !do_reset && ((*deviceExtension).HwFlags & UNIATA_AHCI != 0) {
                // Check if command list is free
                let CI = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_CI as ULONG);
                if CI != 0 {
                    do_reset = true;
                }
            }
            if !do_reset && (statusByte & IDE_STATUS_ERROR as UCHAR != 0) {
                if (*Srb).Cdb[0] as ULONG != SCSIOP_REQUEST_SENSE {
                    (*ata_req_core(AtaReq)).ReqState = REQ_STATE_TRANSFER_COMPLETE as UCHAR;
                    return MapError(deviceExtension as PVOID, Srb);
                }
                // else continue with SCSIOP_REQUEST_SENSE
            }
        }

        if !do_reset {
            // tape DSC/restrictive check
            if (statusByte & IDE_STATUS_DSC as UCHAR == 0)
                && (flags & (DFLAGS_TAPE_DEVICE | DFLAGS_ATAPI_DEVICE) != 0)
                && (*chan).RDP != 0
            {
                ScsiPortStallExecution(200);
                (*ata_req_core(AtaReq)).ReqState = REQ_STATE_QUEUED as UCHAR;
                return SRB_STATUS_PENDING as ULONG;
            }

            if is_rdp((*Srb).Cdb[0]) {
                (*chan).RDP = TRUE as BOOLEAN;
            } else {
                (*chan).RDP = FALSE as BOOLEAN;
            }
            if statusByte & IDE_STATUS_DRQ as UCHAR != 0 {
                // Try to drain stray DRQ data.
                statusByte = AtapiSuckPort2(chan);
                if statusByte & IDE_STATUS_DRQ as UCHAR != 0 {
                    do_reset = true;
                }
            }
        }

        if do_reset {
            AtapiDisableInterrupts(deviceExtension as PVOID, lChannel as ULONG);
            AtapiSoftReset(chan, DeviceNumber);
            // Re-initialize Atapi device.
            CheckDevice(HwDeviceExtension, (*Srb).PathId as ULONG, DeviceNumber, TRUE as BOOLEAN);
            // Inform the port driver that the bus has been reset.
            ScsiPortNotification(
                _SCSI_NOTIFICATION_TYPE_ResetDetected as SCSI_NOTIFICATION_TYPE,
                HwDeviceExtension,
                0 as ULONG,
            );
            UniataExpectChannelInterrupt(chan, FALSE as BOOLEAN);
            (*chan).RDP = FALSE as BOOLEAN;
            InterlockedExchange(
                &mut (*(*deviceExtension).chan.add((*Srb).PathId as usize)).CheckIntr,
                CHECK_INTR_IDLE as LONG,
            );
            AtapiEnableInterrupts(deviceExtension as PVOID, lChannel as ULONG);
            if retried == 0 {
                retried = TRUE as BOOLEAN;
                continue 'retry;
            }
            (*ata_req_core(AtaReq)).ReqState = REQ_STATE_TRANSFER_COMPLETE as UCHAR;
            return SRB_STATUS_SELECTION_TIMEOUT as ULONG;
        }

        if flags & (DFLAGS_SANYO_ATAPI_CHANGER | DFLAGS_ATAPI_CHANGER) != 0 {
            // As the cdrom driver sets the LUN field in the cdb, it must be removed.
            (*Srb).Cdb[1] &= !0xE0;
            if ((*Srb).Cdb[0] as ULONG == SCSIOP_TEST_UNIT_READY)
                && (flags & DFLAGS_SANYO_ATAPI_CHANGER != 0)
            {
                (*Srb).Cdb[7] = (*Srb).Lun;
            }
        }

        // SETUP DMA
        if use_dma != 0 {
            (*chan).ChannelCtrlFlags |= CTRFLAGS_DMA_OPERATION;
        } else {
            (*chan).ChannelCtrlFlags &= !CTRFLAGS_DMA_OPERATION;
        }

        if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
            UniataExpectChannelInterrupt(chan, TRUE as BOOLEAN);
            UniataAhciBeginTransaction(HwDeviceExtension, lChannel as ULONG, DeviceNumber, Srb);
            return SRB_STATUS_PENDING as ULONG;
        }

        statusByte = WaitOnBusy(chan);

        if use_dma != 0 {
            FeatureReg |= ATA_F_DMA as UCHAR;
            if (*LunExt).IdentifyData.__bindgen_anon_1.AtapiDMA.as_ref().DMADirRequired() != 0 {
                if (*Srb).SrbFlags & SRB_FLAGS_DATA_IN != 0 {
                    FeatureReg |= ATA_F_DMAREAD as UCHAR;
                }
            }
        }

        // Write transfer byte count to registers.
        if (*Srb).DataTransferLength >= 0x10000 {
            byteCountLow = 0xFF;
            byteCountHigh = 0xFF;
        } else {
            byteCountLow = ((*Srb).DataTransferLength & 0xFF) as UCHAR;
            byteCountHigh = ((*Srb).DataTransferLength >> 8) as UCHAR;
        }

        if flags & DFLAGS_INT_DRQ != 0 {
            // Device interrupts when ready to receive the packet.
            UniataExpectChannelInterrupt(chan, TRUE as BOOLEAN);
            (*ata_req_core(AtaReq)).ReqState = REQ_STATE_ATAPI_EXPECTING_CMD_INTR as UCHAR;
            InterlockedExchange(&mut (*chan).CheckIntr, CHECK_INTR_IDLE as LONG);
            flags |= DFLAGS_INT_DRQ;
        } else {
            // Device quickly sets DRQ when ready to receive the packet.
            UniataExpectChannelInterrupt(chan, TRUE as BOOLEAN);
            (*ata_req_core(AtaReq)).ReqState = REQ_STATE_ATAPI_DO_NOTHING_INTR as UCHAR;
            InterlockedExchange(&mut (*chan).CheckIntr, CHECK_INTR_IDLE as LONG);

            if g_opt_AtapiSendDisableIntr != 0 {
                AtapiDisableInterrupts(deviceExtension as PVOID, lChannel as ULONG);
            }
            statusByte0 = statusByte;
        }

        AtapiWritePort1(chan, IDX_IO2_o_Control, 0);
        AtapiWritePort1(chan, IDX_ATAPI_IO1_o_Feature, FeatureReg);
        AtapiWritePort1(chan, IDX_ATAPI_IO1_o_ByteCountLow, byteCountLow);
        AtapiWritePort1(chan, IDX_ATAPI_IO1_o_ByteCountHigh, byteCountHigh);
        // Write ATAPI packet command.
        AtapiWritePort1(chan, IDX_ATAPI_IO1_o_Command, IDE_COMMAND_ATAPI_PACKET as UCHAR);

        if flags & DFLAGS_INT_DRQ != 0 {
            // Wait for interrupt and send PACKET there
            return SRB_STATUS_PENDING as ULONG;
        }

        WaitOnBusy(chan);

        statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus); // GetStatus

        // Indicate expecting an interrupt and wait for it.
        UniataExpectChannelInterrupt(chan, TRUE as BOOLEAN);

        i = 0;
        while i < 5000 {
            if g_opt_AtapiSendDisableIntr != 0 {
                statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus); // GetStatus
            } else {
                statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
            }
            interruptReason = AtapiReadPort1(chan, IDX_ATAPI_IO1_i_InterruptReason);
            if ((interruptReason & ATAPI_IR_COD as UCHAR) == ATAPI_IR_COD_Cmd as UCHAR)
                && ((statusByte & (IDE_STATUS_BUSY | IDE_STATUS_DRQ) as UCHAR)
                    == IDE_STATUS_DRQ as UCHAR)
            {
                break;
            }
            ScsiPortStallExecution(g_opt_WaitDrqDelay * 2);
            i += 1;
        }
        interruptReason = AtapiReadPort1(chan, IDX_ATAPI_IO1_i_InterruptReason);
        // Re-evaluate the loop exit condition (the last read above mirrors the C loop end).
        let cod_ok = (interruptReason & ATAPI_IR_COD as UCHAR) == ATAPI_IR_COD_Cmd as UCHAR;
        let drq_ok = (statusByte & (IDE_STATUS_BUSY | IDE_STATUS_DRQ) as UCHAR)
            == IDE_STATUS_DRQ as UCHAR;
        if !cod_ok || !drq_ok {
            if g_opt_AtapiSendDisableIntr != 0 {
                AtapiEnableInterrupts(deviceExtension as PVOID, lChannel as ULONG);
            }
            statusByte = AtapiReadPort1(chan, IDX_IO1_i_Error);
            if statusByte >> 4 != 0 {
                statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
                (*ata_req_core(AtaReq)).ReqState = REQ_STATE_TRANSFER_COMPLETE as UCHAR;
                return MapError(deviceExtension as PVOID, Srb);
            }
            // goto make_reset
            do_reset = true;
        }

        if do_reset {
            // re-run the make_reset block via loop (mirrors goto make_reset)
            AtapiDisableInterrupts(deviceExtension as PVOID, lChannel as ULONG);
            AtapiSoftReset(chan, DeviceNumber);
            CheckDevice(HwDeviceExtension, (*Srb).PathId as ULONG, DeviceNumber, TRUE as BOOLEAN);
            ScsiPortNotification(
                _SCSI_NOTIFICATION_TYPE_ResetDetected as SCSI_NOTIFICATION_TYPE,
                HwDeviceExtension,
                0 as ULONG,
            );
            UniataExpectChannelInterrupt(chan, FALSE as BOOLEAN);
            (*chan).RDP = FALSE as BOOLEAN;
            InterlockedExchange(
                &mut (*(*deviceExtension).chan.add((*Srb).PathId as usize)).CheckIntr,
                CHECK_INTR_IDLE as LONG,
            );
            AtapiEnableInterrupts(deviceExtension as PVOID, lChannel as ULONG);
            if retried == 0 {
                retried = TRUE as BOOLEAN;
                continue 'retry;
            }
            (*ata_req_core(AtaReq)).ReqState = REQ_STATE_TRANSFER_COMPLETE as UCHAR;
            return SRB_STATUS_SELECTION_TIMEOUT as ULONG;
        }

        // clear interrupt
        statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus

        if (*chan).ChannelCtrlFlags & CTRFLAGS_DMA_OPERATION != 0 {
            AtapiDmaDBPreSync(HwDeviceExtension, chan, Srb);
        }
        if g_opt_AtapiSendDisableIntr != 0 {
            AtapiEnableInterrupts(deviceExtension as PVOID, lChannel as ULONG);
        }

        // Send CDB to device.
        AtapiWriteBuffer2(
            chan,
            IDX_IO1_o_Data,
            (*Srb).Cdb.as_ptr() as PVOID,
            if (*LunExt).IdentifyData.AtapiCmdSize() != 0 { 8 } else { 6 },
            PIO0_TIMING,
        );

        statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus); // GetStatus

        // In DMA mode, do not start transfer on entry error (int may never come).
        if statusByte & IDE_STATUS_ERROR as UCHAR != 0 {
            statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
            interruptReason =
                AtapiReadPort1(chan, IDX_ATAPI_IO1_i_InterruptReason) & ATAPI_IR_Mask as UCHAR;
            let _ = interruptReason;
            (*ata_req_core(AtaReq)).ReqState = REQ_STATE_TRANSFER_COMPLETE as UCHAR;
            return MapError(deviceExtension as PVOID, Srb);
        }
        if statusByte & IDE_STATUS_DRQ as UCHAR != 0 {
            statusByte = AtapiReadPort1(chan, IDX_IO1_i_Error);
            if statusByte >> 4 != 0 {
                statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
                (*ata_req_core(AtaReq)).ReqState = REQ_STATE_TRANSFER_COMPLETE as UCHAR;
                return MapError(deviceExtension as PVOID, Srb);
            }
        }

        if (*chan).ChannelCtrlFlags & CTRFLAGS_DMA_OPERATION != 0 {
            AtapiDmaStart(HwDeviceExtension, DeviceNumber, lChannel as ULONG, Srb);
        }

        InterlockedExchange(&mut (*chan).CheckIntr, CHECK_INTR_IDLE as LONG);
        (*ata_req_core(AtaReq)).ReqState = REQ_STATE_ATAPI_EXPECTING_DATA_INTR as UCHAR;

        let _ = statusByte0;
        return SRB_STATUS_PENDING as ULONG;
    }
}

// AtapiSendCommand: GetLba2 fallthrough — set reorderable + R/W direction flags.
unsafe fn atapi_set_rw_flags(AtaReq: PATA_REQ, Srb: PSCSI_REQUEST_BLOCK) {
    (*ata_req_core(AtaReq)).Flags |= REQ_FLAG_REORDERABLE_CMD as UCHAR;
    (*ata_req_core(AtaReq)).Flags &= !(REQ_FLAG_RW_MASK as UCHAR);
    let c0 = (*Srb).Cdb[0] as ULONG;
    (*ata_req_core(AtaReq)).Flags |= if c0 == SCSIOP_WRITE || c0 == SCSIOP_WRITE12 || c0 == SCSIOP_WRITE16
    {
        REQ_FLAG_WRITE as UCHAR
    } else {
        REQ_FLAG_READ as UCHAR
    };
}

// IS_RDP(OperationCode) (atapi.h)
#[inline(always)]
fn is_rdp(op: UCHAR) -> bool {
    let o = op as ULONG;
    o == SCSIOP_ERASE
        || o == SCSIOP_LOAD_UNLOAD
        || o == SCSIOP_LOCATE
        || o == SCSIOP_REWIND
        || o == SCSIOP_SPACE
        || o == SCSIOP_SEEK
        || o == SCSIOP_WRITE_FILEMARKS
}

// ===========================================================================
//  IdeSendCommand — program ATA registers for IDE disk transfer.
// ===========================================================================
pub unsafe extern "system" fn IdeSendCommand(
    HwDeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
    CmdAction: ULONG,
) -> ULONG {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let lChannel: UCHAR;
    let chan: PHW_CHANNEL;
    let cdb: PCDB;
    let LunExt: PHW_LU_EXTENSION;

    let mut statusByte: UCHAR;
    let mut errorByte: UCHAR;
    let mut status: ULONG = SRB_STATUS_INVALID_REQUEST as ULONG;
    let mut i: ULONG;
    let mut lba: ULONGLONG;
    let mut DeviceNumber: ULONG;
    let AtaReq: PATA_REQ;

    AtaReq = (*Srb).SrbExtension as PATA_REQ;

    lChannel = (*Srb).PathId;
    chan = (*deviceExtension).chan.add(lChannel as usize);
    DeviceNumber = (*Srb).TargetId as ULONG;
    LunExt = (*chan).lun[DeviceNumber as usize];

    if (*ata_req_core(AtaReq)).ReqState < REQ_STATE_PREPARE_TO_TRANSFER as UCHAR {
        (*ata_req_core(AtaReq)).ReqState = REQ_STATE_PREPARE_TO_TRANSFER as UCHAR;
    }

    cdb = (*Srb).Cdb.as_ptr() as PCDB;

    // CMD_ACTION_PREPARE gate. `goto default_no_prep` maps to returning BUSY.
    if CmdAction == CMD_ACTION_PREPARE {
        match (*Srb).Cdb[0] as ULONG {
            x if x == SCSIOP_SERVICE_ACTION16 => {
                if (*cdb).SERVICE_ACTION16.as_ref().ServiceAction() as ULONG
                    == SCSIOP_SA_READ_CAPACITY16
                {
                    // ok
                } else {
                    return SRB_STATUS_BUSY as ULONG; // default_no_prep
                }
            }
            x if x == SCSIOP_READ_CAPACITY
                || x == SCSIOP_READ
                || x == SCSIOP_WRITE
                || x == SCSIOP_READ12
                || x == SCSIOP_WRITE12
                || x == SCSIOP_READ16
                || x == SCSIOP_WRITE16
                || x == SCSIOP_REQUEST_SENSE =>
            {
                // all right
            }
            _ => {
                return SRB_STATUS_BUSY as ULONG; // default / default_no_prep
            }
        }
    }

    match (*Srb).Cdb[0] as ULONG {
        x if x == SCSIOP_INQUIRY => {
            // Filter out wrong TIDs.
            if (*Srb).Lun != 0
                || ((*Srb).PathId as ULONG >= (*deviceExtension).NumberChannels as ULONG)
                || ((*Srb).TargetId as ULONG >= (*deviceExtension).NumberLuns)
            {
                status = SRB_STATUS_SELECTION_TIMEOUT as ULONG;
            } else {
                let inquiryData = (*Srb).DataBuffer as PINQUIRYDATA;
                let identifyData: PIDENTIFY_DATA2 = &mut (*LunExt).IdentifyData;
                let mut rejected = false;

                if (*(*chan).lun[DeviceNumber as usize]).DeviceFlags & DFLAGS_DEVICE_PRESENT == 0 {
                    if CheckDevice(HwDeviceExtension, lChannel as ULONG, DeviceNumber, FALSE as BOOLEAN) == 0
                    {
                        status = SRB_STATUS_SELECTION_TIMEOUT as ULONG;
                        rejected = true;
                    }
                } else if UniataAnybodyHome(HwDeviceExtension, lChannel as ULONG, DeviceNumber) == 0 {
                    UniataForgetDevice((*chan).lun[DeviceNumber as usize]);
                    status = SRB_STATUS_SELECTION_TIMEOUT as ULONG;
                    rejected = true;
                }

                if !rejected {
                    RtlZeroMemory((*Srb).DataBuffer as PVOID, (*Srb).DataTransferLength as usize);
                    (*inquiryData).set_DeviceType(DIRECT_ACCESS_DEVICE as UCHAR);

                    if (*LunExt).DeviceFlags & DFLAGS_REMOVABLE_DRIVE != 0 {
                        (*inquiryData).set_RemovableMedia(1);
                    }
                    if (*LunExt).DeviceFlags & DFLAGS_LBA_ENABLED != 0 {
                        (*inquiryData).set_RelativeAddressing(1);
                    }
                    (*inquiryData).set_CommandQueue(1);

                    // Fill in vendor identification fields (__REACTOS__).
                    FillDeviceIdentificationString(inquiryData, identifyData);

                    // Firmware revision -> product revision.
                    i = 0;
                    while i < 4 {
                        let dst = (*inquiryData).__bindgen_anon_1.__bindgen_anon_1.as_mut().ProductRevisionLevel.as_mut_ptr().add(i as usize)
                            as *mut USHORT;
                        let src = (core::ptr::addr_of!((*identifyData).FirmwareRevision) as *const UCHAR)
                            .add(i as usize) as *const USHORT;
                        *dst = RtlUshortByteSwap(*src);
                        i += 2;
                    }

                    status = SRB_STATUS_SUCCESS as ULONG;
                }
            }
        }
        x if x == SCSIOP_REPORT_LUNS => {
            let alen: ULONG = dd_swap((*cdb).REPORT_LUNS.as_ref().AllocationLength.as_ptr());
            if alen < 16 {
                return ide_invalid_cdb(Srb);
            }
            let alen8: ULONG = 8;

            let LunInfo = (*Srb).DataBuffer as PREPORT_LUNS_INFO_HDR;
            RtlZeroMemory(LunInfo as PVOID, 16);

            // MOV_DD_SWP(LunInfo->ListLength, alen)
            *((*LunInfo).ListLength.as_mut_ptr() as *mut ULONG) = RtlUlongByteSwap(alen8);
            (*Srb).DataTransferLength = 16;
            status = SRB_STATUS_SUCCESS as ULONG;
        }
        x if x == SCSIOP_MODE_SENSE => {
            if (*cdb).MODE_SENSE.as_ref().PageCode() as ULONG == MODE_PAGE_POWER_CONDITION {
                let md = (*Srb).DataBuffer as PMODE_POWER_CONDITION_PAGE;
                if ((*cdb).MODE_SENSE.as_ref().AllocationLength as usize)
                    < core::mem::size_of::<MODE_POWER_CONDITION_PAGE>()
                {
                    status = SRB_STATUS_DATA_OVERRUN as ULONG;
                } else {
                    RtlZeroMemory(md as PVOID, core::mem::size_of::<MODE_POWER_CONDITION_PAGE>());
                    (*md).set_PageCode(MODE_PAGE_POWER_CONDITION as UCHAR);
                    (*md).PageLength = (core::mem::size_of::<MODE_POWER_CONDITION_PAGE>()
                        - core::mem::size_of::<MODE_PARAMETER_HEADER>())
                        as UCHAR;
                    (*md).Byte3.Fields.as_mut().set_Idle(
                        ((*LunExt).PowerState <= StartStop_Power_Idle as UCHAR) as UCHAR,
                    );
                    (*md).Byte3.Fields.as_mut().set_Standby(
                        ((*LunExt).PowerState == StartStop_Power_Standby as UCHAR) as UCHAR,
                    );
                    (*Srb).DataTransferLength =
                        core::mem::size_of::<MODE_POWER_CONDITION_PAGE>() as ULONG;
                    status = SRB_STATUS_SUCCESS as ULONG;
                }
            } else if (*cdb).MODE_SENSE.as_ref().PageCode() as ULONG == MODE_PAGE_CACHING {
                let md = (*Srb).DataBuffer as PMODE_CACHING_PAGE;
                if ((*cdb).MODE_SENSE.as_ref().AllocationLength as usize)
                    < core::mem::size_of::<MODE_CACHING_PAGE>()
                {
                    status = SRB_STATUS_DATA_OVERRUN as ULONG;
                } else {
                    RtlZeroMemory(md as PVOID, core::mem::size_of::<MODE_CACHING_PAGE>());
                    (*md).set_PageCode(MODE_PAGE_CACHING as UCHAR);
                    (*md).PageLength = (core::mem::size_of::<MODE_CACHING_PAGE>()
                        - core::mem::size_of::<MODE_PARAMETER_HEADER>())
                        as UCHAR;
                    (*md).set_ReadDisableCache(
                        if (*LunExt).DeviceFlags & DFLAGS_RCACHE_ENABLED != 0 { 0 } else { 1 },
                    );
                    (*md).set_WriteCacheEnable(
                        if (*LunExt).DeviceFlags & DFLAGS_WCACHE_ENABLED != 0 { 1 } else { 0 },
                    );
                    (*Srb).DataTransferLength = core::mem::size_of::<MODE_CACHING_PAGE>() as ULONG;
                    status = SRB_STATUS_SUCCESS as ULONG;
                }
            } else if (*LunExt).DeviceFlags & DFLAGS_MEDIA_STATUS_ENABLED != 0 {
                statusByte = AtaCommand(
                    deviceExtension, DeviceNumber, lChannel as ULONG,
                    IDE_COMMAND_GET_MEDIA_STATUS as UCHAR, 0, 0, 0, 0, 0, ATA_WAIT_READY,
                );
                if statusByte & IDE_STATUS_ERROR as UCHAR == 0 {
                    UniataExpectChannelInterrupt(chan, FALSE as BOOLEAN);
                    InterlockedExchange(&mut (*chan).CheckIntr, CHECK_INTR_IDLE as LONG);
                    status = SRB_STATUS_SUCCESS as ULONG;
                } else {
                    errorByte = AtapiReadPort1(chan, IDX_IO1_i_Error);
                    statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
                    UniataExpectChannelInterrupt(chan, FALSE as BOOLEAN);
                    InterlockedExchange(&mut (*chan).CheckIntr, CHECK_INTR_IDLE as LONG);
                    status = SRB_STATUS_SUCCESS as ULONG;

                    if errorByte & IDE_ERROR_DATA_ERROR as UCHAR != 0 {
                        let mdh = (*Srb).DataBuffer as PMODE_PARAMETER_HEADER;
                        (*Srb).DataTransferLength =
                            core::mem::size_of::<MODE_PARAMETER_HEADER>() as ULONG;
                        (*mdh).DeviceSpecificParameter |= MODE_DSP_WRITE_PROTECT as UCHAR;
                    }
                }
                status = SRB_STATUS_SUCCESS as ULONG;
            } else {
                status = SRB_STATUS_INVALID_REQUEST as ULONG;
            }
        }
        x if x == SCSIOP_TEST_UNIT_READY => {
            if (*(*chan).lun[DeviceNumber as usize]).DeviceFlags & DFLAGS_MEDIA_STATUS_ENABLED != 0 {
                statusByte = AtaCommand(
                    deviceExtension, DeviceNumber, lChannel as ULONG,
                    IDE_COMMAND_GET_MEDIA_STATUS as UCHAR, 0, 0, 0, 0, 0, ATA_WAIT_READY,
                );
                if statusByte & IDE_STATUS_ERROR as UCHAR == 0 {
                    UniataExpectChannelInterrupt(chan, FALSE as BOOLEAN);
                    InterlockedExchange(&mut (*chan).CheckIntr, CHECK_INTR_IDLE as LONG);
                    status = SRB_STATUS_SUCCESS as ULONG;
                } else {
                    errorByte = AtapiReadPort1(chan, IDX_IO1_i_Error);
                    if errorByte == IDE_ERROR_DATA_ERROR as UCHAR {
                        statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
                        UniataExpectChannelInterrupt(chan, FALSE as BOOLEAN);
                        InterlockedExchange(&mut (*chan).CheckIntr, CHECK_INTR_IDLE as LONG);
                        status = SRB_STATUS_SUCCESS as ULONG;
                    } else {
                        UniataExpectChannelInterrupt(chan, TRUE as BOOLEAN);
                        InterlockedExchange(&mut (*chan).CheckIntr, CHECK_INTR_IDLE as LONG);
                        status = SRB_STATUS_PENDING as ULONG;
                    }
                }
            } else {
                status = SRB_STATUS_SUCCESS as ULONG;
            }
        }
        x if x == SCSIOP_READ_CAPACITY => {
            i = DEV_BSIZE;
            RtlZeroMemory((*Srb).DataBuffer as PVOID, core::mem::size_of::<READ_CAPACITY_DATA>());
            let rc = (*Srb).DataBuffer as PREAD_CAPACITY_DATA;
            core::ptr::write_unaligned(core::ptr::addr_of_mut!((*rc).BytesPerBlock), RtlUlongByteSwap(i));

            i = (*LunExt).NumOfSectors as ULONG;
            if i == 0 {
                i = (*LunExt).IdentifyData.SectorsPerTrack as ULONG
                    * (*LunExt).IdentifyData.NumberOfHeads as ULONG
                    * (*LunExt).IdentifyData.NumberOfCylinders as ULONG;
            }
            i -= 1;
            core::ptr::write_unaligned(core::ptr::addr_of_mut!((*rc).LogicalBlockAddress), RtlUlongByteSwap(i));
            status = SRB_STATUS_SUCCESS as ULONG;
        }
        x if x == SCSIOP_SERVICE_ACTION16 => {
            if (*cdb).SERVICE_ACTION16.as_ref().ServiceAction() as ULONG == SCSIOP_SA_READ_CAPACITY16 {
                i = DEV_BSIZE;
                RtlZeroMemory(
                    (*Srb).DataBuffer as PVOID,
                    core::mem::size_of::<READ_CAPACITY16_DATA>(),
                );
                let rc = (*Srb).DataBuffer as PREAD_CAPACITY16_DATA;
                core::ptr::write_unaligned(core::ptr::addr_of_mut!((*rc).BytesPerBlock), RtlUlongByteSwap(i));

                lba = (*LunExt).NumOfSectors;
                if lba == 0 {
                    lba = (*LunExt).IdentifyData.SectorsPerTrack as ULONGLONG
                        * (*LunExt).IdentifyData.NumberOfHeads as ULONGLONG
                        * (*LunExt).IdentifyData.NumberOfCylinders as ULONGLONG;
                }
                lba -= 1;
                // MOV_QD_SWP(LogicalBlockAddress, lba)
                let dp = core::ptr::addr_of_mut!((*rc).LogicalBlockAddress) as *mut ULONG;
                core::ptr::write_unaligned(dp, RtlUlongByteSwap((lba >> 32) as ULONG));
                core::ptr::write_unaligned(dp.add(1), RtlUlongByteSwap(lba as ULONG));
                status = SRB_STATUS_SUCCESS as ULONG;
            } else {
                return ide_default_abort(Srb);
            }
        }
        x if x == SCSIOP_VERIFY || x == SCSIOP_VERIFY12 || x == SCSIOP_VERIFY16 => {
            status = IdeVerify(HwDeviceExtension, Srb);
        }
        x if x == SCSIOP_READ
            || x == SCSIOP_WRITE
            || x == SCSIOP_READ12
            || x == SCSIOP_WRITE12
            || x == SCSIOP_READ16
            || x == SCSIOP_WRITE16 =>
        {
            (*ata_req_core(AtaReq)).Flags &= !(REQ_FLAG_RW_MASK as UCHAR);
            (*ata_req_core(AtaReq)).Flags |=
                if x == SCSIOP_WRITE || x == SCSIOP_WRITE12 || x == SCSIOP_WRITE16 {
                    REQ_FLAG_WRITE as UCHAR
                } else {
                    REQ_FLAG_READ as UCHAR
                };
            status = IdeReadWrite(HwDeviceExtension, Srb, CmdAction);
        }
        x if x == SCSIOP_START_STOP_UNIT => {
            let mut command2: UCHAR = 0;
            let ss = (*cdb).START_STOP.as_ref();

            if ss.FL() != 0
                || ss.FormatLayerNumber() != 0
                || (*cdb).START_STOP.as_ref().Reserved2 != 0
                || ss.Reserved2_2() != 0
                || ss.Reserved3() != 0
            {
                return ide_invalid_cdb(Srb);
            }

            if ss.PowerConditions() != 0 {
                match ss.PowerConditions() as ULONG {
                    p if p == StartStop_Power_Idle => command2 = IDE_COMMAND_IDLE_IMMED as UCHAR,
                    p if p == StartStop_Power_Standby => command2 = IDE_COMMAND_STANDBY_IMMED as UCHAR,
                    p if p == StartStop_Power_Sleep => command2 = IDE_COMMAND_SLEEP as UCHAR,
                    _ => return ide_invalid_cdb(Srb),
                }
                (*LunExt).PowerState = ss.PowerConditions();
            } else if ss.LoadEject() == 1 {
                command2 = IDE_COMMAND_MEDIA_EJECT as UCHAR;
            } else if ss.Start() == 0 {
                command2 = IDE_COMMAND_STANDBY_IMMED as UCHAR;
            } else if (*LunExt).PowerState == StartStop_Power_Sleep as UCHAR {
                UniataUserDeviceReset(deviceExtension, LunExt, lChannel as ULONG);
                status = SRB_STATUS_SUCCESS as ULONG;
                return ide_finish(AtaReq, CmdAction, status);
            } else if (*LunExt).PowerState > StartStop_Power_Idle as UCHAR {
                command2 = IDE_COMMAND_IDLE_IMMED as UCHAR;
            } else {
                status = SRB_STATUS_SUCCESS as ULONG;
                return ide_finish(AtaReq, CmdAction, status);
            }
            if command2 != 0 {
                statusByte = WaitOnBaseBusy(chan);
                statusByte = AtaCommand(
                    deviceExtension, DeviceNumber, lChannel as ULONG, command2, 0, 0, 0, 0, 0,
                    if (*cdb).START_STOP.as_ref().Immediate() != 0 { ATA_IMMEDIATE } else { ATA_WAIT_READY },
                );
                status = if statusByte & IDE_STATUS_ERROR as UCHAR != 0 {
                    SRB_STATUS_ERROR as ULONG
                } else {
                    SRB_STATUS_SUCCESS as ULONG
                };
            } else {
                return ide_invalid_cdb(Srb);
            }
        }
        x if x == SCSIOP_MEDIUM_REMOVAL => {
            let cdb2 = (*Srb).Cdb.as_ptr() as PCDB;
            if (*LunExt).IdentifyData.Removable() != 0 {
                statusByte = WaitOnBaseBusy(chan);
                if (*cdb2).MEDIA_REMOVAL.as_ref().Prevent as ULONG == TRUE as ULONG {
                    statusByte = AtaCommand(
                        deviceExtension, DeviceNumber, lChannel as ULONG,
                        IDE_COMMAND_DOOR_LOCK as UCHAR, 0, 0, 0, 0, 0, ATA_IMMEDIATE,
                    );
                } else {
                    statusByte = AtaCommand(
                        deviceExtension, DeviceNumber, lChannel as ULONG,
                        IDE_COMMAND_DOOR_UNLOCK as UCHAR, 0, 0, 0, 0, 0, ATA_IMMEDIATE,
                    );
                }
                let _ = statusByte;
                status = SRB_STATUS_SUCCESS as ULONG;
            } else {
                status = SRB_STATUS_INVALID_REQUEST as ULONG;
            }
        }
        x if x == SCSIOP_REQUEST_SENSE => {
            if (*LunExt).DeviceFlags & DFLAGS_MEDIA_STATUS_ENABLED != 0 {
                status = IdeBuildSenseBuffer(HwDeviceExtension, Srb);
            } else {
                status = SRB_STATUS_INVALID_REQUEST as ULONG;
            }
        }
        x if x == SCSIOP_ATA_PASSTHROUGH => {
            status = ide_ata_passthrough(
                deviceExtension, HwDeviceExtension, Srb, chan, lChannel, &mut DeviceNumber,
            );
        }
        _ => {
            status = SRB_STATUS_INVALID_REQUEST as ULONG; // default / default_abort
        }
    }

    ide_finish(AtaReq, CmdAction, status)
}

// IdeSendCommand tail: set req state from final status.
#[inline]
unsafe fn ide_finish(AtaReq: PATA_REQ, CmdAction: ULONG, status: ULONG) -> ULONG {
    if status == SRB_STATUS_PENDING as ULONG {
        if CmdAction & CMD_ACTION_EXEC != 0 {
            (*ata_req_core(AtaReq)).ReqState = REQ_STATE_EXPECTING_INTR as UCHAR;
        }
    } else {
        (*ata_req_core(AtaReq)).ReqState = REQ_STATE_TRANSFER_COMPLETE as UCHAR;
    }
    status
}

// `invalid_cdb:` label — fills sense buffer with ILLEGAL_REQUEST / INVALID_CDB.
unsafe fn ide_invalid_cdb(Srb: PSCSI_REQUEST_BLOCK) -> ULONG {
    let AtaReq = (*Srb).SrbExtension as PATA_REQ;
    if !(*Srb).SenseInfoBuffer.is_null() {
        let senseBuffer = (*Srb).SenseInfoBuffer as PSENSE_DATA;
        (*senseBuffer).set_ErrorCode(0x70);
        (*senseBuffer).set_Valid(1);
        (*senseBuffer).AdditionalSenseLength = 0xb;
        (*senseBuffer).set_SenseKey(SCSI_SENSE_ILLEGAL_REQUEST as UCHAR);
        (*senseBuffer).AdditionalSenseCode = SCSI_ADSENSE_INVALID_CDB as UCHAR;
        (*senseBuffer).AdditionalSenseCodeQualifier = 0;

        (*Srb).SrbStatus = SRB_STATUS_AUTOSENSE_VALID as UCHAR;
        (*Srb).ScsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
    }
    ide_finish(AtaReq, 0, SRB_STATUS_ERROR as ULONG)
}

// `default_abort:` — unsupported command.
unsafe fn ide_default_abort(Srb: PSCSI_REQUEST_BLOCK) -> ULONG {
    let AtaReq = (*Srb).SrbExtension as PATA_REQ;
    ide_finish(AtaReq, 0, SRB_STATUS_INVALID_REQUEST as ULONG)
}

// IdeSendCommand: SCSIOP_ATA_PASSTHROUGH case. `passthrough_err`/`passthrough_done`
// gotos modeled with a `do_err` flag + `'pt` block.
unsafe fn ide_ata_passthrough(
    deviceExtension: PHW_DEVICE_EXTENSION,
    HwDeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
    chan: PHW_CHANNEL,
    lChannel: UCHAR,
    DeviceNumberRef: &mut ULONG,
) -> ULONG {
    let mut DeviceNumber = *DeviceNumberRef;
    let mut regs: PIDEREGS_EX = (&mut (*Srb).Cdb[2]) as *mut UCHAR as PIDEREGS_EX;
    let mut use_dma: BOOLEAN = FALSE as BOOLEAN;
    let to_lim: ULONG;
    let mut statusByte: UCHAR;
    let mut i: ULONG;
    let status: ULONG;

    if (*(*chan).DeviceExtension).HwFlags & UNIATA_SATA != 0 {
        // lChannel = Srb->TargetId >> 1; (commented out in C)
    } else {
        DeviceNumber = max_u32(DeviceNumber, 1);
        (*regs).bDriveHeadReg &= 0x0f;
        (*regs).bDriveHeadReg |= (((DeviceNumber & 0x1) << 4) | 0xA0) as UCHAR;
    }

    if (*regs).bOpFlags & 1 == 0 {
        // execute ATA command
        if ((*regs).bOpFlags as ULONG & UNIATA_SPTI_EX_SPEC_TO) == UNIATA_SPTI_EX_SPEC_TO {
            to_lim = (*Srb).TimeOutValue;
        } else if (*Srb).TimeOutValue <= 2 {
            to_lim = (*Srb).TimeOutValue * 900;
        } else {
            to_lim = ((*Srb).TimeOutValue * 999) - 500;
        }

        AtapiDisableInterrupts(deviceExtension as PVOID, lChannel as ULONG);

        let mut do_err = false;
        let mut done = false;

        if (*(*chan).DeviceExtension).HwFlags & UNIATA_AHCI != 0 {
            // AHCI
            statusByte = UniataAhciSendPIOCommandDirect(
                deviceExtension as PVOID, lChannel as ULONG, DeviceNumber, Srb, regs, ATA_WAIT_INTR, to_lim,
            );
            if statusByte == IDE_STATUS_WRONG as UCHAR {
                do_err = true;
            } else if statusByte & (IDE_STATUS_BUSY | IDE_STATUS_ERROR) as UCHAR != 0 {
                UniataAhciAbortOperation(chan);
                do_err = true;
            } else {
                done = true; // passthrough_done
            }
        } else {
            // SATA/PATA
            if (AtaCommandFlags[(*regs).__bindgen_anon_2.bCommandReg.as_ref().clone() as usize] as ULONG
                & ATA_CMD_FLAG_DMA)
                != 0
                || ((*regs).bOpFlags as ULONG & UNIATA_SPTI_EX_USE_DMA) != 0
            {
                if (*(*chan).lun[DeviceNumber as usize]).LimitedTransferMode >= ATA_DMA as UCHAR {
                    use_dma = TRUE as BOOLEAN;
                    if AtapiDmaSetup(
                        HwDeviceExtension, DeviceNumber, lChannel as ULONG, Srb,
                        (*Srb).DataBuffer as PUCHAR,
                        ((*Srb).DataTransferLength + DEV_BSIZE - 1) & !(DEV_BSIZE - 1),
                    ) == 0
                    {
                        use_dma = FALSE as BOOLEAN;
                    }
                }
            }

            AtapiWritePort1(chan, IDX_IO1_o_DriveSelect, (*regs).bDriveHeadReg);
            ScsiPortStallExecution(10);
            if use_dma != 0 {
                AtapiDmaDBPreSync(HwDeviceExtension, chan, Srb);
            }

            let bCommandReg = (*regs).__bindgen_anon_2.bCommandReg.as_ref().clone();
            let bFeaturesReg = (*regs).__bindgen_anon_1.bFeaturesReg.as_ref().clone();
            if ((*regs).bOpFlags as ULONG & ATA_FLAGS_48BIT_COMMAND) == 0 {
                AtapiWritePort1(chan, IDX_IO1_o_Feature, bFeaturesReg);
                AtapiWritePort1(chan, IDX_IO1_o_BlockCount, (*regs).bSectorCountReg);
                AtapiWritePort1(chan, IDX_IO1_o_BlockNumber, (*regs).bSectorNumberReg);
                AtapiWritePort1(chan, IDX_IO1_o_CylinderLow, (*regs).bCylLowReg);
                AtapiWritePort1(chan, IDX_IO1_o_CylinderHigh, (*regs).bCylHighReg);
            } else {
                AtapiWritePort1(chan, IDX_IO1_o_Feature, (*regs).bFeaturesRegH);
                AtapiWritePort1(chan, IDX_IO1_o_Feature, bFeaturesReg);
                AtapiWritePort1(chan, IDX_IO1_o_BlockCount, (*regs).bSectorCountRegH);
                AtapiWritePort1(chan, IDX_IO1_o_BlockCount, (*regs).bSectorCountReg);
                AtapiWritePort1(chan, IDX_IO1_o_BlockNumber, (*regs).bSectorNumberRegH);
                AtapiWritePort1(chan, IDX_IO1_o_BlockNumber, (*regs).bSectorNumberReg);
                AtapiWritePort1(chan, IDX_IO1_o_CylinderLow, (*regs).bCylLowRegH);
                AtapiWritePort1(chan, IDX_IO1_o_CylinderLow, (*regs).bCylLowReg);
                AtapiWritePort1(chan, IDX_IO1_o_CylinderHigh, (*regs).bCylHighRegH);
                AtapiWritePort1(chan, IDX_IO1_o_CylinderHigh, (*regs).bCylHighReg);
            }
            AtapiWritePort1(chan, IDX_IO1_o_Command, bCommandReg);

            if use_dma != 0 {
                statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
                if statusByte & IDE_STATUS_ERROR as UCHAR != 0 {
                    do_err = true;
                }
                if !do_err {
                    AtapiDmaStart(HwDeviceExtension, DeviceNumber, lChannel as ULONG, Srb);
                }
            }

            if !do_err {
                ScsiPortStallExecution(1); // wait for busy to be set

                i = 0;
                while i < to_lim {
                    statusByte = WaitOnBaseBusy(chan);
                    statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus
                    if statusByte & IDE_STATUS_ERROR as UCHAR != 0 {
                        break;
                    }
                    if statusByte & IDE_STATUS_BUSY as UCHAR == 0 {
                        break;
                    }
                    i += 2;
                }
                if i >= to_lim {
                    AtapiResetController__(deviceExtension as PVOID, lChannel as ULONG, RESET_COMPLETE_NONE);
                    do_err = true;
                }
            }

            if !do_err {
                if use_dma != 0 {
                    AtapiCheckInterrupt__(deviceExtension as PVOID, lChannel);
                }
                AtapiDmaDone(deviceExtension as PVOID, DeviceNumber, lChannel as ULONG, core::ptr::null_mut());
                statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status); // GetBaseStatus

                if statusByte & (IDE_STATUS_BUSY | IDE_STATUS_ERROR) as UCHAR != 0 {
                    AtapiSuckPort2(chan);
                    do_err = true;
                } else {
                    if use_dma == 0 {
                        if statusByte & IDE_STATUS_DRQ as UCHAR != 0 {
                            if (*Srb).SrbFlags & SRB_FLAGS_DATA_IN != 0 {
                                AtapiReadBuffer2(
                                    chan, IDX_IO1_i_Data,
                                    (*Srb).DataBuffer as PVOID,
                                    (*Srb).DataTransferLength / 2, 0,
                                );
                            } else if (*Srb).SrbFlags & SRB_FLAGS_DATA_OUT != 0 {
                                AtapiWriteBuffer2(
                                    chan, IDX_IO1_o_Data,
                                    (*Srb).DataBuffer as PVOID,
                                    (*Srb).DataTransferLength / 2, 0,
                                );
                            }
                        }
                    }
                }
            }
        }

        if do_err && !done {
            // passthrough_err
            if !(*Srb).SenseInfoBuffer.is_null() {
                let senseBuffer = (*Srb).SenseInfoBuffer as PSENSE_DATA;
                (*senseBuffer).set_ErrorCode(0x70);
                (*senseBuffer).set_Valid(1);
                (*senseBuffer).AdditionalSenseLength = 0xb;
                (*senseBuffer).set_SenseKey(SCSI_SENSE_ABORTED_COMMAND as UCHAR);
                (*senseBuffer).AdditionalSenseCode = 0;
                (*senseBuffer).AdditionalSenseCodeQualifier = 0;
                (*Srb).SrbStatus = SRB_STATUS_AUTOSENSE_VALID as UCHAR;
                (*Srb).ScsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
            }
            status = SRB_STATUS_ERROR as ULONG;
        } else if done {
            status = SRB_STATUS_SUCCESS as ULONG;
        } else {
            status = SRB_STATUS_SUCCESS as ULONG;
        }

        // passthrough_done:
        AtapiEnableInterrupts(deviceExtension as PVOID, lChannel as ULONG);
        *DeviceNumberRef = DeviceNumber;
        return status;
    } else {
        // read task register
        let use48: BOOLEAN;
        regs = (*Srb).DataBuffer as PIDEREGS_EX;

        if ((*Srb).DataTransferLength as usize >= core::mem::size_of::<IDEREGS_EX>())
            && ((*regs).bOpFlags as ULONG & ATA_FLAGS_48BIT_COMMAND != 0)
        {
            use48 = TRUE as BOOLEAN;
        } else if (*Srb).DataTransferLength as usize >= core::mem::size_of::<IDEREGS>() {
            use48 = FALSE as BOOLEAN;
        } else {
            *DeviceNumberRef = DeviceNumber;
            return SRB_STATUS_DATA_OVERRUN as ULONG;
        }
        RtlZeroMemory(
            regs as PVOID,
            if use48 != 0 {
                core::mem::size_of::<IDEREGS_EX>()
            } else {
                core::mem::size_of::<IDEREGS>()
            },
        );
        (*regs).bOpFlags = if use48 != 0 { ATA_FLAGS_48BIT_COMMAND as UCHAR } else { 0 };
        UniataSnapAtaRegs(chan, 0, regs);

        *DeviceNumberRef = DeviceNumber;
        return SRB_STATUS_SUCCESS as ULONG;
    }
}

// ===========================================================================
//  IdeMediaStatus — enable/disable media status notification.
// ===========================================================================
pub unsafe extern "system" fn IdeMediaStatus(
    EnableMSN: BOOLEAN,
    HwDeviceExtension: PVOID,
    lChannel: ULONG,
    DeviceNumber: ULONG,
) {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL;
    let statusByte: UCHAR;
    let mut errorByte: UCHAR;

    chan = (*deviceExtension).chan.add(lChannel as usize);
    SelectDrive(chan, DeviceNumber);

    if EnableMSN == TRUE as BOOLEAN {
        // If supported, enable Media Status Notification support.
        if (*(*chan).lun[DeviceNumber as usize]).DeviceFlags & DFLAGS_REMOVABLE_DRIVE != 0 {
            statusByte = AtaCommand(
                deviceExtension, DeviceNumber, lChannel,
                IDE_COMMAND_SET_FEATURES as UCHAR, 0, 0, 0, 0,
                ATA_C_F_ENAB_MEDIASTAT as UCHAR, ATA_WAIT_BASE_READY,
            );

            if statusByte & IDE_STATUS_ERROR as UCHAR != 0 {
                errorByte = AtapiReadPort1(chan, IDX_IO1_i_Error);
                let _ = errorByte;
            } else {
                (*(*chan).lun[DeviceNumber as usize]).DeviceFlags |= DFLAGS_MEDIA_STATUS_ENABLED;
                (*chan).ReturningMediaStatus = 0;
            }
        }
    } else {
        // disable if previously enabled
        if (*(*chan).lun[DeviceNumber as usize]).DeviceFlags & DFLAGS_MEDIA_STATUS_ENABLED != 0 {
            let _ = AtaCommand(
                deviceExtension, DeviceNumber, lChannel,
                IDE_COMMAND_SET_FEATURES as UCHAR, 0, 0, 0, 0,
                ATA_C_F_DIS_MEDIASTAT as UCHAR, ATA_WAIT_BASE_READY,
            );
            (*(*chan).lun[DeviceNumber as usize]).DeviceFlags &= !DFLAGS_MEDIA_STATUS_ENABLED;
        }
    }
}

// ===========================================================================
//  IdeBuildSenseBuffer — artificial sense for GET_MEDIA_STATUS.
// ===========================================================================
pub unsafe extern "system" fn IdeBuildSenseBuffer(
    HwDeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
) -> ULONG {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let senseBuffer = (*Srb).DataBuffer as PSENSE_DATA;
    let ReturningMediaStatus = (*(*deviceExtension).chan.add((*Srb).PathId as usize)).ReturningMediaStatus;

    if !senseBuffer.is_null() {
        if ReturningMediaStatus & IDE_ERROR_MEDIA_CHANGE as UCHAR != 0 {
            (*senseBuffer).set_ErrorCode(0x70);
            (*senseBuffer).set_Valid(1);
            (*senseBuffer).AdditionalSenseLength = 0xb;
            (*senseBuffer).set_SenseKey(SCSI_SENSE_UNIT_ATTENTION as UCHAR);
            (*senseBuffer).AdditionalSenseCode = SCSI_ADSENSE_MEDIUM_CHANGED as UCHAR;
            (*senseBuffer).AdditionalSenseCodeQualifier = 0;
        } else if ReturningMediaStatus & IDE_ERROR_MEDIA_CHANGE_REQ as UCHAR != 0 {
            (*senseBuffer).set_ErrorCode(0x70);
            (*senseBuffer).set_Valid(1);
            (*senseBuffer).AdditionalSenseLength = 0xb;
            (*senseBuffer).set_SenseKey(SCSI_SENSE_UNIT_ATTENTION as UCHAR);
            (*senseBuffer).AdditionalSenseCode = SCSI_ADSENSE_MEDIUM_CHANGED as UCHAR;
            (*senseBuffer).AdditionalSenseCodeQualifier = 0;
        } else if ReturningMediaStatus & IDE_ERROR_END_OF_MEDIA as UCHAR != 0 {
            (*senseBuffer).set_ErrorCode(0x70);
            (*senseBuffer).set_Valid(1);
            (*senseBuffer).AdditionalSenseLength = 0xb;
            (*senseBuffer).set_SenseKey(SCSI_SENSE_NOT_READY as UCHAR);
            (*senseBuffer).AdditionalSenseCode = SCSI_ADSENSE_NO_MEDIA_IN_DEVICE as UCHAR;
            (*senseBuffer).AdditionalSenseCodeQualifier = 0;
        } else if ReturningMediaStatus & IDE_ERROR_DATA_ERROR as UCHAR != 0 {
            (*senseBuffer).set_ErrorCode(0x70);
            (*senseBuffer).set_Valid(1);
            (*senseBuffer).AdditionalSenseLength = 0xb;
            (*senseBuffer).set_SenseKey(SCSI_SENSE_DATA_PROTECT as UCHAR);
            (*senseBuffer).AdditionalSenseCode = 0;
            (*senseBuffer).AdditionalSenseCodeQualifier = 0;
        }
        return SRB_STATUS_SUCCESS as ULONG;
    }
    SRB_STATUS_ERROR as ULONG
}

// ===========================================================================
//  UniataUserDeviceReset
// ===========================================================================
pub unsafe extern "system" fn UniataUserDeviceReset(
    deviceExtension: PHW_DEVICE_EXTENSION,
    LunExt: PHW_LU_EXTENSION,
    lChannel: ULONG,
) {
    let mut i: ULONG;
    AtapiDisableInterrupts(deviceExtension as PVOID, lChannel);
    if ((*LunExt).DeviceFlags & DFLAGS_ATAPI_DEVICE != 0)
        && ((*LunExt).PowerState != StartStop_Power_Sleep as UCHAR)
    {
        AtapiSoftReset((*deviceExtension).chan.add(lChannel as usize), (*LunExt).Lun as ULONG);
    } else {
        AtapiResetController__(deviceExtension as PVOID, lChannel, RESET_COMPLETE_NONE);
        i = 0;
        while i < (*deviceExtension).NumberLuns {
            (*(*(*deviceExtension).chan.add(lChannel as usize)).lun[i as usize]).DeviceFlags |=
                DFLAGS_REINIT_DMA;
            i += 1;
        }
    }
    (*LunExt).DeviceFlags |= DFLAGS_REINIT_DMA; // force PIO/DMA reinit
    AtapiEnableInterrupts(deviceExtension as PVOID, lChannel);
}

// ===========================================================================
//  UniataNeedQueueing
// ===========================================================================
pub unsafe extern "system" fn UniataNeedQueueing(
    deviceExtension: PHW_DEVICE_EXTENSION,
    chan: PHW_CHANNEL,
    TopLevel: BOOLEAN,
) -> BOOLEAN {
    let mut PostReq: BOOLEAN = FALSE as BOOLEAN;
    if TopLevel != 0 {
        if (*chan).queue_depth > 0 {
            PostReq = TRUE as BOOLEAN;
        } else if (*deviceExtension).simplexOnly != 0 && (*deviceExtension).queue_depth > 0 {
            PostReq = TRUE as BOOLEAN;
        }
    }
    PostReq
}

// ===========================================================================
//  IdeSendSmartCommand  (#ifndef UNIATA_CORE)
// ===========================================================================
pub unsafe extern "system" fn IdeSendSmartCommand(
    HwDeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
    targetId: ULONG, // assume it is always valid
) -> ULONG {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let c: ULONG;
    let chan: PHW_CHANNEL;
    let AtaReq = (*Srb).SrbExtension as PATA_REQ;
    let cmdOutParameters =
        ((*Srb).DataBuffer as PUCHAR).add(core::mem::size_of::<SRB_IO_CONTROL>()) as PSENDCMDOUTPARAMS;
    let cmdInParameters: SENDCMDINPARAMS =
        *(((*Srb).DataBuffer as PUCHAR).add(core::mem::size_of::<SRB_IO_CONTROL>()) as PSENDCMDINPARAMS);
    let regs: *const IDEREGS = &cmdInParameters.irDriveRegs;
    let mut statusByte: UCHAR;
    let DeviceNumber: ULONG;

    if (*regs).bCommandReg != SMART_CMD {
        return SRB_STATUS_INVALID_REQUEST as ULONG;
    }

    c = targetId / (*deviceExtension).NumberLuns;
    DeviceNumber = targetId % (*deviceExtension).NumberLuns;

    chan = (*deviceExtension).chan.add(c as usize);

    (*chan).SmartCommand = (*regs).bFeaturesReg;

    // Determine which of the commands to carry out.
    match (*regs).bFeaturesReg {
        x if x == READ_ATTRIBUTES
            || x == READ_THRESHOLDS
            || x == READ_LOG_SECTOR as UCHAR
            || x == WRITE_LOG_SECTOR as UCHAR =>
        {
            if (*Srb).DataTransferLength
                < (core::mem::size_of::<SRB_IO_CONTROL>() as ULONG
                    + core::mem::size_of::<SENDCMDOUTPARAMS>() as ULONG
                    + READ_ATTRIBUTE_BUFFER_SIZE
                    - 1)
            {
                return SRB_STATUS_DATA_OVERRUN as ULONG;
            }

            statusByte = WaitOnBusy(chan);

            if statusByte & IDE_STATUS_BUSY as UCHAR != 0 {
                return SRB_STATUS_BUSY as ULONG;
            }

            // Zero the output buffer (input was saved off locally; buffers are the same).
            RtlZeroMemory(
                cmdOutParameters as PVOID,
                core::mem::size_of::<SENDCMDOUTPARAMS>() + READ_ATTRIBUTE_BUFFER_SIZE as usize - 1,
            );

            (*ata_req_core(AtaReq)).DataBuffer = (*cmdOutParameters).bBuffer.as_mut_ptr() as PUSHORT;
            (*ata_req_core(AtaReq)).WordsLeft = READ_ATTRIBUTE_BUFFER_SIZE / 2;

            statusByte = AtaCommand(
                deviceExtension,
                DeviceNumber,
                c,
                (*regs).bCommandReg,
                ((*regs).bCylLowReg as USHORT) | (((*regs).bCylHighReg as USHORT) << 8),
                0,
                (*regs).bSectorNumberReg,
                (*regs).bSectorCountReg,
                (*regs).bFeaturesReg,
                ATA_IMMEDIATE,
            );

            if statusByte & IDE_STATUS_ERROR as UCHAR == 0 {
                return SRB_STATUS_PENDING as ULONG;
            }
            SRB_STATUS_ERROR as ULONG
        }
        x if x == ENABLE_SMART as UCHAR
            || x == DISABLE_SMART as UCHAR
            || x == RETURN_SMART_STATUS
            || x == ENABLE_DISABLE_AUTOSAVE as UCHAR
            || x == EXECUTE_OFFLINE_DIAGS as UCHAR
            || x == SAVE_ATTRIBUTE_VALUES as UCHAR
            || x == AUTO_OFFLINE as UCHAR =>
        {
            statusByte = WaitOnBusy(chan);

            if statusByte & IDE_STATUS_BUSY as UCHAR != 0 {
                return SRB_STATUS_BUSY as ULONG;
            }

            RtlZeroMemory(
                cmdOutParameters as PVOID,
                core::mem::size_of::<SENDCMDOUTPARAMS>() - 1,
            );

            (*ata_req_core(AtaReq)).DataBuffer = (*cmdOutParameters).bBuffer.as_mut_ptr() as PUSHORT;
            (*ata_req_core(AtaReq)).WordsLeft = 0;

            statusByte = AtaCommand(
                deviceExtension,
                DeviceNumber,
                c,
                (*regs).bCommandReg,
                ((*regs).bCylLowReg as USHORT) | (((*regs).bCylHighReg as USHORT) << 8),
                0,
                (*regs).bSectorNumberReg,
                (*regs).bSectorCountReg,
                (*regs).bFeaturesReg,
                ATA_IMMEDIATE,
            );

            if statusByte & IDE_STATUS_ERROR as UCHAR == 0 {
                UniataExpectChannelInterrupt(chan, TRUE as BOOLEAN); // device may interrupt
                return SRB_STATUS_PENDING as ULONG;
            }
            SRB_STATUS_ERROR as ULONG
        }
        _ => SRB_STATUS_INVALID_REQUEST as ULONG,
    }
}

// ===========================================================================
//  UniAtaCalculateLBARegs
// ===========================================================================
pub unsafe extern "system" fn UniAtaCalculateLBARegs(
    LunExt: PHW_LU_EXTENSION,
    startingSector: ULONGLONG,
    max_bcount: PULONG,
) -> ULONGLONG {
    let drvSelect: UCHAR;
    let sectorNumber: UCHAR;
    let cylinder: USHORT;
    let tmp: ULONG;

    if (*LunExt).DeviceFlags & DFLAGS_LBA_ENABLED != 0 {
        *max_bcount = 0;
        if (*LunExt).LimitedTransferMode >= ATA_DMA as UCHAR {
            if !(*LunExt).DeviceExtension.is_null() {
                *max_bcount = (*(*LunExt).DeviceExtension).MaximumDmaTransferLength / DEV_BSIZE;
            }
        }
        return startingSector;
    }
    tmp = (*LunExt).IdentifyData.SectorsPerTrack as ULONG
        * (*LunExt).IdentifyData.NumberOfHeads as ULONG;
    if tmp == 0 {
        cylinder = 0;
        drvSelect = 0;
        sectorNumber = 1;
        *max_bcount = (*LunExt).IdentifyData.SectorsPerTrack as ULONG;
    } else {
        cylinder = (startingSector / tmp as ULONGLONG) as USHORT;
        drvSelect =
            ((startingSector % tmp as ULONGLONG) / (*LunExt).IdentifyData.SectorsPerTrack as ULONGLONG)
                as UCHAR;
        sectorNumber =
            ((startingSector % (*LunExt).IdentifyData.SectorsPerTrack as ULONGLONG) as UCHAR)
                .wrapping_add(1);
        *max_bcount = (*LunExt).IdentifyData.SectorsPerTrack as ULONG - sectorNumber as ULONG + 1;
    }

    (sectorNumber as ULONG & 0xff) as ULONGLONG
        | (((cylinder as ULONG & 0xffff) as ULONGLONG) << 8)
        | (((drvSelect as ULONG & 0xf) as ULONGLONG) << 24)
}

// ===========================================================================
//  UniAtaCalculateLBARegsBack
// ===========================================================================
pub unsafe extern "system" fn UniAtaCalculateLBARegsBack(
    LunExt: PHW_LU_EXTENSION,
    lba: ULONGLONG,
) -> ULONGLONG {
    let drvSelect: ULONG;
    let sectorNumber: ULONG;
    let cylinder: ULONG;
    let tmp: ULONG;

    if (*LunExt).DeviceFlags & DFLAGS_LBA_ENABLED != 0 {
        return lba;
    }
    tmp = (*LunExt).IdentifyData.SectorsPerTrack as ULONG
        * (*LunExt).IdentifyData.NumberOfHeads as ULONG;

    cylinder = (((lba >> 8) & 0xffff) as USHORT) as ULONG;
    drvSelect = (((lba >> 24) & 0xf) as UCHAR) as ULONG;
    sectorNumber = ((lba & 0xff) as UCHAR) as ULONG;

    (sectorNumber as ULONGLONG).wrapping_sub(1)
        .wrapping_add((drvSelect * (*LunExt).IdentifyData.SectorsPerTrack as ULONG) as ULONGLONG)
        .wrapping_add((cylinder as ULONGLONG) * (tmp as ULONGLONG))
}

// ===========================================================================
//  AtapiStartIo (->HwStartIo) + AtapiStartIo__
// ===========================================================================
pub unsafe extern "system" fn AtapiStartIo(
    HwDeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
) -> BOOLEAN {
    AtapiStartIo__(HwDeviceExtension, Srb, TRUE as BOOLEAN)
}

pub unsafe extern "system" fn AtapiStartIo__(
    HwDeviceExtension: PVOID,
    mut Srb: PSCSI_REQUEST_BLOCK,
    mut TopLevel: BOOLEAN,
) -> BOOLEAN {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let mut lChannel: UCHAR;
    let mut chan: PHW_CHANNEL;
    let mut LunExt: PHW_LU_EXTENSION;
    let mut status: ULONG;
    let mut DeviceNumber: ULONG;
    let mut PathId: UCHAR = 0;
    let mut TargetId: UCHAR = 0;
    let mut Lun: UCHAR = 0;
    let mut AtaReq: PATA_REQ;
    let tmpSrb: PSCSI_REQUEST_BLOCK;
    let mut PostReq: BOOLEAN;
    let mut atapiDev: BOOLEAN;
    let mut commPort: BOOLEAN;

    // Isr2DevObj must always be NULL for non-PCI
    if !(*deviceExtension).Isr2DevObj.is_null()
        && (*BMList.add((*deviceExtension).DevIndex as usize)).Isr2Enable == 0
    {
        (*BMList.add((*deviceExtension).DevIndex as usize)).Isr2Enable = TRUE as BOOLEAN;
    }

    if TopLevel != 0 && !Srb.is_null() && !(*Srb).SrbExtension.is_null() {
        UniAtaClearAtaReq((*Srb).SrbExtension);
    }

    // fetch all queued commands for the channel (if valid)
    loop {
        lChannel = (*Srb).PathId;
        chan = core::ptr::null_mut();
        LunExt = core::ptr::null_mut();
        DeviceNumber = (*Srb).TargetId as ULONG;
        commPort = FALSE as BOOLEAN;

        // flow flag: when set, jump straight to complete_req with current status.
        let mut goto_complete = false;
        status = SRB_STATUS_INVALID_REQUEST as ULONG;

        if (lChannel as ULONG == (*deviceExtension).NumberChannels as ULONG)
            && (*Srb).Lun == 0
            && (*Srb).TargetId == 0
            && (((*Srb).Function as ULONG == SRB_FUNCTION_IO_CONTROL)
                || ((*Srb).Function as ULONG == SRB_FUNCTION_EXECUTE_SCSI
                    && (*Srb).Cdb[0] as ULONG == SCSIOP_INQUIRY))
        {
            // This is our virtual device
            if (*Srb).Function as ULONG == SRB_FUNCTION_EXECUTE_SCSI {
                if ((*Srb).DataTransferLength as usize) < core::mem::size_of::<PINQUIRYDATA>() {
                    status = SRB_STATUS_DATA_OVERRUN as ULONG;
                    goto_complete = true;
                } else {
                    let inquiryData = (*Srb).DataBuffer as PINQUIRYDATA;
                    RtlZeroMemory((*Srb).DataBuffer as PVOID, (*Srb).DataTransferLength as usize);
                    (*inquiryData).set_DeviceType(COMMUNICATION_DEVICE as UCHAR);
                    // Fill in vendor identification fields.
                    RtlCopyMemory(
                        (*inquiryData).__bindgen_anon_1.__bindgen_anon_1.as_mut().VendorId.as_mut_ptr() as PVOID,
                        uniata_comm_name.as_ptr() as PVOID,
                        28,
                    );
                    status = SRB_STATUS_SUCCESS as ULONG;
                    goto_complete = true;
                }
            } else {
                commPort = TRUE as BOOLEAN;
                // Pass IOCTL request down
            }
        } else if (lChannel as ULONG >= (*deviceExtension).NumberChannels as ULONG)
            || ((*Srb).TargetId as ULONG >= (*deviceExtension).NumberLuns)
            || (*Srb).Lun != 0
        {
            if lChannel as ULONG >= (*deviceExtension).NumberChannels as ULONG {
                chan = core::ptr::null_mut();
            }
            // reject_srb
            status = SRB_STATUS_SELECTION_TIMEOUT as ULONG;
            goto_complete = true;
        } else if ((*deviceExtension).HwFlags & UNIATA_AHCI != 0)
            && UniataAhciChanImplemented(deviceExtension, lChannel as ULONG) == 0
        {
            chan = core::ptr::null_mut();
        }

        let mut do_reject = false;

        if !goto_complete {
            if commPort == 0 {
                chan = (*deviceExtension).chan.add(lChannel as usize);
                LunExt = (*chan).lun[DeviceNumber as usize];
                if LunExt.is_null() {
                    do_reject = true;
                } else {
                    atapiDev = if (*LunExt).DeviceFlags & DFLAGS_ATAPI_DEVICE != 0 {
                        TRUE as BOOLEAN
                    } else {
                        FALSE as BOOLEAN
                    };
                }
            } else {
                atapiDev = FALSE as BOOLEAN;
            }
        }

        // reject_srb closure result handling.
        if do_reject {
            status = SRB_STATUS_SELECTION_TIMEOUT as ULONG;
            goto_complete = true;
        }

        atapiDev = if !LunExt.is_null() && (*LunExt).DeviceFlags & DFLAGS_ATAPI_DEVICE != 0 {
            TRUE as BOOLEAN
        } else {
            FALSE as BOOLEAN
        };

        if !goto_complete {
            // Determine which function. The body sets `status`; some paths set
            // do_reject/goto_complete to mirror the C gotos.
            let (st, reject, complete) = atapi_startio_dispatch(
                HwDeviceExtension, deviceExtension, Srb, &mut chan, &mut LunExt,
                &mut lChannel, &mut DeviceNumber, &mut TopLevel, commPort, atapiDev,
            );
            status = st;
            if reject {
                status = SRB_STATUS_SELECTION_TIMEOUT as ULONG;
            }
            let _ = complete;
        }

        // complete_req:
        PathId = (*Srb).PathId;
        TargetId = (*Srb).TargetId;
        Lun = (*Srb).Lun;

        if status != SRB_STATUS_PENDING as ULONG {
            (*Srb).SrbStatus = status as UCHAR;

            if !chan.is_null() && !Srb.is_null() {
                AtapiDmaDBSync(chan, Srb);
            }
            UniataRemoveRequest(chan, Srb);
            ScsiPortNotification(
                _SCSI_NOTIFICATION_TYPE_RequestComplete as SCSI_NOTIFICATION_TYPE,
                deviceExtension as PVOID,
                Srb,
            );

            // Remove current Srb & get next one
            Srb = UniataGetCurRequest(chan);
            if !Srb.is_null() {
                AtaReq = (*Srb).SrbExtension as PATA_REQ;
                if (*ata_req_core(AtaReq)).ReqState > REQ_STATE_QUEUED as UCHAR {
                    // current request is under processing; do nothing here
                    Srb = core::ptr::null_mut();
                }
            }
        }

        if Srb.is_null() || status == SRB_STATUS_PENDING as ULONG {
            break;
        }
    }

    // Indicate ready for next request.
    ScsiPortNotification(
        _SCSI_NOTIFICATION_TYPE_NextRequest as SCSI_NOTIFICATION_TYPE,
        deviceExtension as PVOID,
        core::ptr::null_mut::<core::ffi::c_void>(),
    );
    ScsiPortNotification(
        _SCSI_NOTIFICATION_TYPE_NextLuRequest as SCSI_NOTIFICATION_TYPE,
        deviceExtension as PVOID,
        PathId as ULONG,
        TargetId as ULONG,
        Lun as ULONG,
    );

    TRUE as BOOLEAN
}

// AtapiStartIo__ per-SRB dispatch (the big SRB_FUNCTION switch + IO_CONTROL).
// Returns (status, reject_srb, goto_complete). `reject_srb` => SELECTION_TIMEOUT.
unsafe fn atapi_startio_dispatch(
    HwDeviceExtension: PVOID,
    deviceExtension: PHW_DEVICE_EXTENSION,
    Srb: PSCSI_REQUEST_BLOCK,
    chan_ref: &mut PHW_CHANNEL,
    LunExt_ref: &mut PHW_LU_EXTENSION,
    lChannel_ref: &mut UCHAR,
    DeviceNumber_ref: &mut ULONG,
    TopLevel_ref: &mut BOOLEAN,
    commPort: BOOLEAN,
    atapiDev: BOOLEAN,
) -> (ULONG, bool, bool) {
    let mut chan = *chan_ref;
    let mut LunExt = *LunExt_ref;
    let mut lChannel = *lChannel_ref;
    let mut DeviceNumber = *DeviceNumber_ref;
    let mut atapiDev = atapiDev;
    let mut status: ULONG;
    let mut PostReq: BOOLEAN;
    let mut AtaReq: PATA_REQ;
    let mut statusByte: UCHAR;

    macro_rules! ret_reject { () => {{
        *chan_ref = chan; *LunExt_ref = LunExt; *lChannel_ref = lChannel; *DeviceNumber_ref = DeviceNumber;
        return (SRB_STATUS_SELECTION_TIMEOUT as ULONG, true, true);
    }}; }
    macro_rules! ret { ($s:expr) => {{
        *chan_ref = chan; *LunExt_ref = LunExt; *lChannel_ref = lChannel; *DeviceNumber_ref = DeviceNumber;
        return ($s, false, true);
    }}; }

    match (*Srb).Function as ULONG {
        f if f == SRB_FUNCTION_EXECUTE_SCSI => {
            if LunExt.is_null() || ((*LunExt).DeviceFlags & DFLAGS_DEVICE_PRESENT == 0) {
                if (*Srb).Cdb[0] as ULONG == SCSIOP_ATA_PASSTHROUGH {
                    // let passthrough go
                } else if (*Srb).Cdb[0] as ULONG == SCSIOP_INQUIRY {
                    // let INQUIRY go
                } else {
                    ret!(SRB_STATUS_SELECTION_TIMEOUT as ULONG);
                }
            }

            PostReq = UniataNeedQueueing(deviceExtension, chan, *TopLevel_ref);

            if PostReq != 0 {
                if atapiDev != 0 && ((*Srb).Cdb[0] as ULONG != SCSIOP_ATA_PASSTHROUGH) {
                    status = AtapiSendCommand(HwDeviceExtension, Srb, CMD_ACTION_PREPARE);
                } else {
                    status = IdeSendCommand(HwDeviceExtension, Srb, CMD_ACTION_PREPARE);
                }
                if status == SRB_STATUS_BUSY as ULONG {
                    status = SRB_STATUS_PENDING as ULONG;
                }
                UniataQueueRequest(chan, Srb);
            } else {
                // Send command to device.
                if *TopLevel_ref != 0 {
                    AtaReq = (*Srb).SrbExtension as PATA_REQ;
                    UniataQueueRequest(chan, Srb);
                    (*ata_req_core(AtaReq)).ReqState = REQ_STATE_QUEUED as UCHAR;
                }

                if LunExt.is_null() || ((*LunExt).DeviceFlags & DFLAGS_DEVICE_PRESENT == 0) {
                    if LunExt.is_null() {
                        ret_reject!();
                    }
                    if (*Srb).Cdb[0] as ULONG == SCSIOP_INQUIRY {
                        if UniataAnybodyHome(deviceExtension as PVOID, (*chan).lChannel as ULONG, DeviceNumber) != 0 {
                            if CheckDevice(HwDeviceExtension, (*chan).lChannel as ULONG, DeviceNumber, TRUE as BOOLEAN) == 0 {
                                ret_reject!();
                            }
                        }
                        if (*LunExt).DeviceFlags & DFLAGS_DEVICE_PRESENT == 0 {
                            ret_reject!();
                        }
                    } else if (*Srb).Cdb[0] as ULONG == SCSIOP_ATA_PASSTHROUGH {
                        // allow
                    } else {
                        ret_reject!();
                    }
                }

                if atapiDev != 0 && ((*Srb).Cdb[0] as ULONG != SCSIOP_ATA_PASSTHROUGH) {
                    // __REACTOS__ broken-CD/DVD INQUIRY emulation
                    status = SRB_STATUS_BUSY as ULONG;

                    if (*Srb).Cdb[0] as ULONG == SCSIOP_INQUIRY
                        && ((*LunExt).DeviceFlags & DFLAGS_ATAPI_DEVICE != 0)
                        && ((*LunExt).IdentifyData.DeviceType() as ULONG == ATAPI_TYPE_CDROM
                            || (*LunExt).IdentifyData.DeviceType() as ULONG == ATAPI_TYPE_OPTICAL)
                        && (*LunExt).IdentifyData.ModelNumber[0] != 0
                    {
                        let mut j: ULONG = 0;
                        let mut vendorId: [CCHAR; 26] = [0; 26];
                        while j < 26 {
                            // MOV_DW_SWP(vendorId[j], ((PUCHAR)ModelNumber)[j])
                            let src = (core::ptr::addr_of!((*LunExt).IdentifyData.ModelNumber) as *const UCHAR)
                                .add(j as usize);
                            let dst = (&mut vendorId[j as usize]) as *mut CCHAR as *mut USHORT;
                            *dst = RtlUshortByteSwap(*(src as *const USHORT));
                            j += 2;
                        }

                        if _strnicmp(vendorId.as_ptr() as PCCHAR, b"THOMSON-DVD\0".as_ptr() as PCCHAR, 11) == 0
                            || _strnicmp(vendorId.as_ptr() as PCCHAR, b"PHILIPS XBOX DVD DRIVE\0".as_ptr() as PCCHAR, 22) == 0
                            || _strnicmp(vendorId.as_ptr() as PCCHAR, b"PHILIPS J5 3235C\0".as_ptr() as PCCHAR, 16) == 0
                            || _strnicmp(vendorId.as_ptr() as PCCHAR, b"SAMSUNG DVD-ROM SDG-605B\0".as_ptr() as PCCHAR, 24) == 0
                        {
                            let mut i: ULONG;
                            let inquiryData = (*Srb).DataBuffer as PINQUIRYDATA;
                            let identifyData: PIDENTIFY_DATA2 = &mut (*LunExt).IdentifyData;

                            RtlZeroMemory((*Srb).DataBuffer as PVOID, (*Srb).DataTransferLength as usize);
                            (*inquiryData).set_DeviceType(READ_ONLY_DIRECT_ACCESS_DEVICE as UCHAR);

                            if (*LunExt).DeviceFlags & DFLAGS_REMOVABLE_DRIVE != 0 {
                                (*inquiryData).set_RemovableMedia(1);
                            }
                            if (*LunExt).DeviceFlags & DFLAGS_LBA_ENABLED != 0 {
                                (*inquiryData).set_RelativeAddressing(1);
                            }
                            (*inquiryData).set_CommandQueue(1);

                            FillDeviceIdentificationString(inquiryData, identifyData);

                            i = 0;
                            while i < 4 {
                                let dst = (*inquiryData).__bindgen_anon_1.__bindgen_anon_1.as_mut()
                                    .ProductRevisionLevel.as_mut_ptr().add(i as usize) as *mut USHORT;
                                let src = (core::ptr::addr_of!((*identifyData).FirmwareRevision) as *const UCHAR)
                                    .add(i as usize) as *const USHORT;
                                *dst = RtlUshortByteSwap(*src);
                                i += 2;
                            }
                            status = SRB_STATUS_SUCCESS as ULONG;
                        }
                    }

                    if status != SRB_STATUS_SUCCESS as ULONG {
                        status = AtapiSendCommand(HwDeviceExtension, Srb, CMD_ACTION_ALL);
                    }
                } else {
                    status = IdeSendCommand(HwDeviceExtension, Srb, CMD_ACTION_ALL);
                }
            }
            *TopLevel_ref = FALSE as BOOLEAN;
            ret!(status);
        }
        f if f == SRB_FUNCTION_ABORT_COMMAND => {
            let tmpSrb = ScsiPortGetSrb(
                HwDeviceExtension, (*Srb).PathId, (*Srb).TargetId, (*Srb).Lun, (*Srb).QueueTag as LONG,
            );
            if tmpSrb != (*Srb).NextSrb || (*chan).queue_depth == 0 {
                ret!(SRB_STATUS_ABORT_FAILED as ULONG);
            }
            AtaReq = (*tmpSrb).SrbExtension as PATA_REQ;
            if (*ata_req_core(AtaReq)).ReqState > REQ_STATE_READY_TO_TRANSFER as UCHAR {
                if AtapiResetController__(deviceExtension as PVOID, lChannel as ULONG, RESET_COMPLETE_CURRENT) == 0 {
                    ScsiPortLogError(HwDeviceExtension, core::ptr::null_mut(), 0, 0, 0, SP_INTERNAL_ADAPTER_ERROR, 5 << 8);
                    status = SRB_STATUS_ERROR as ULONG;
                } else {
                    status = SRB_STATUS_SUCCESS as ULONG;
                }
            } else {
                if !(*tmpSrb).SenseInfoBuffer.is_null()
                    && (*tmpSrb).SenseInfoBufferLength as usize >= core::mem::size_of::<SENSE_DATA>()
                {
                    let senseBuffer = (*tmpSrb).SenseInfoBuffer as PSENSE_DATA;
                    (*senseBuffer).set_ErrorCode(0);
                    (*senseBuffer).set_Valid(1);
                    (*senseBuffer).AdditionalSenseLength = 0xb;
                    (*senseBuffer).set_SenseKey(SCSI_SENSE_ABORTED_COMMAND as UCHAR);
                    (*senseBuffer).AdditionalSenseCode = 0;
                    (*senseBuffer).AdditionalSenseCodeQualifier = 0;
                    (*tmpSrb).SrbStatus |= SRB_STATUS_AUTOSENSE_VALID as UCHAR;
                }
                AtapiDmaDBSync(chan, tmpSrb);
                UniataRemoveRequest(chan, tmpSrb);
                ScsiPortNotification(
                    _SCSI_NOTIFICATION_TYPE_RequestComplete as SCSI_NOTIFICATION_TYPE,
                    deviceExtension as PVOID, tmpSrb,
                );
                status = SRB_STATUS_SUCCESS as ULONG;
            }
            ret!(status);
        }
        f if f == SRB_FUNCTION_RESET_DEVICE || f == SRB_FUNCTION_RESET_LOGICAL_UNIT => {
            UniataUserDeviceReset(deviceExtension, LunExt, lChannel as ULONG);
            ret!(SRB_STATUS_SUCCESS as ULONG);
        }
        f if f == SRB_FUNCTION_RESET_BUS => {
            if AtapiResetController__(deviceExtension as PVOID, lChannel as ULONG, RESET_COMPLETE_ALL) == 0 {
                ScsiPortLogError(HwDeviceExtension, core::ptr::null_mut(), 0, 0, 0, SP_INTERNAL_ADAPTER_ERROR, 5 << 8);
                status = SRB_STATUS_ERROR as ULONG;
            } else {
                status = SRB_STATUS_SUCCESS as ULONG;
            }
            ret!(status);
        }
        f if f == SRB_FUNCTION_SHUTDOWN => {
            if LunExt.is_null() || ((*LunExt).DeviceFlags & DFLAGS_DEVICE_PRESENT == 0) {
                // no such device
            } else if atapiDev != 0 {
                // FLUSH ATAPI device - do nothing
            } else {
                AtapiDisableInterrupts(deviceExtension as PVOID, lChannel as ULONG);
                statusByte = AtaCommand(deviceExtension, DeviceNumber, (*Srb).PathId as ULONG,
                    IDE_COMMAND_FLUSH_CACHE as UCHAR, 0, 0, 0, 0, 0, ATA_WAIT_IDLE);
                if (*LunExt).DeviceFlags & DFLAGS_WCACHE_ENABLED != 0 {
                    statusByte = AtaCommand(deviceExtension, DeviceNumber, lChannel as ULONG,
                        IDE_COMMAND_SET_FEATURES as UCHAR, 0, 0, 0, 0, ATA_C_F_DIS_WCACHE as UCHAR, ATA_WAIT_BASE_READY);
                    (*LunExt).DeviceFlags &= !DFLAGS_WCACHE_ENABLED;
                    statusByte = AtaCommand(deviceExtension, DeviceNumber, lChannel as ULONG,
                        IDE_COMMAND_SET_FEATURES as UCHAR, 0, 0, 0, 0, ATA_C_F_ENAB_WCACHE as UCHAR, ATA_WAIT_BASE_READY);
                    if statusByte & IDE_STATUS_ERROR as UCHAR != 0 {
                        (*LunExt).DeviceFlags &= !DFLAGS_WCACHE_ENABLED;
                    } else {
                        (*LunExt).DeviceFlags |= DFLAGS_WCACHE_ENABLED;
                    }
                }
                AtapiEnableInterrupts(deviceExtension as PVOID, lChannel as ULONG);
            }
            ret!(SRB_STATUS_SUCCESS as ULONG);
        }
        f if f == SRB_FUNCTION_FLUSH => {
            ret!(SRB_STATUS_SUCCESS as ULONG);
        }
        f if f == SRB_FUNCTION_IO_CONTROL => {
            let (st, reject) = atapi_startio_ioctl(
                HwDeviceExtension, deviceExtension, Srb, &mut chan, &mut LunExt,
                &mut lChannel, &mut DeviceNumber, &mut atapiDev, *TopLevel_ref, commPort,
            );
            if reject {
                ret_reject!();
            }
            ret!(st);
        }
        _ => {
            ret!(SRB_STATUS_INVALID_REQUEST as ULONG);
        }
    }
}

// AtapiStartIo__ SRB_FUNCTION_IO_CONTROL handler. Returns (status, reject).
// `wrong_buffer_size`/`invalid_request`/`do_bus_reset`/`handle_bad_ldev`/`uata_ctl_queue`
// gotos are modeled with early returns / flags.
unsafe fn atapi_startio_ioctl(
    HwDeviceExtension: PVOID,
    deviceExtension: PHW_DEVICE_EXTENSION,
    Srb: PSCSI_REQUEST_BLOCK,
    chan_ref: &mut PHW_CHANNEL,
    LunExt_ref: &mut PHW_LU_EXTENSION,
    lChannel_ref: &mut UCHAR,
    DeviceNumber_ref: &mut ULONG,
    atapiDev_ref: &mut BOOLEAN,
    TopLevel: BOOLEAN,
    commPort: BOOLEAN,
) -> (ULONG, bool) {
    let mut chan = *chan_ref;
    let mut LunExt = *LunExt_ref;
    let mut lChannel = *lChannel_ref;
    let mut DeviceNumber = *DeviceNumber_ref;
    let mut atapiDev = *atapiDev_ref;
    let len: ULONG = (*Srb).DataTransferLength;
    let sig = ((*Srb).DataBuffer as PSRB_IO_CONTROL);

    macro_rules! sync_out { () => {{
        *chan_ref = chan; *LunExt_ref = LunExt; *lChannel_ref = lChannel;
        *DeviceNumber_ref = DeviceNumber; *atapiDev_ref = atapiDev;
    }}; }
    macro_rules! wrong_buffer { () => {{ sync_out!(); return (SRB_STATUS_DATA_OVERRUN as ULONG, false); }}; }
    macro_rules! reject { () => {{ sync_out!(); return (0, true); }}; }
    macro_rules! ret { ($s:expr) => {{ sync_out!(); return ($s, false); }}; }

    let mut status: ULONG;

    if _strnicmp((*sig).Signature.as_ptr() as PCCHAR, b"SCSIDISK\0".as_ptr() as PCCHAR, "SCSIDISK".len()) == 0 {
        let mut targetId: ULONG = !0u32;

        if (len as usize) < core::mem::size_of::<SRB_IO_CONTROL>() {
            wrong_buffer!();
        }

        match (*sig).ControlCode {
            c if c == IOCTL_SCSI_MINIPORT_SMART_VERSION => {
                let versionParameters =
                    ((*Srb).DataBuffer as PUCHAR).add(core::mem::size_of::<SRB_IO_CONTROL>())
                        as PGETVERSIONINPARAMS;
                if (len as usize)
                    < core::mem::size_of::<SRB_IO_CONTROL>() + core::mem::size_of::<GETVERSIONINPARAMS>()
                {
                    wrong_buffer!();
                }
                targetId = (*versionParameters).bIDEDeviceMap as ULONG;
            }
            c if c == IOCTL_SCSI_MINIPORT_IDENTIFY
                || c == IOCTL_SCSI_MINIPORT_READ_SMART_ATTRIBS
                || c == IOCTL_SCSI_MINIPORT_READ_SMART_THRESHOLDS
                || c == IOCTL_SCSI_MINIPORT_ENABLE_SMART
                || c == IOCTL_SCSI_MINIPORT_DISABLE_SMART
                || c == IOCTL_SCSI_MINIPORT_RETURN_STATUS
                || c == IOCTL_SCSI_MINIPORT_ENABLE_DISABLE_AUTOSAVE
                || c == IOCTL_SCSI_MINIPORT_SAVE_ATTRIBUTE_VALUES
                || c == IOCTL_SCSI_MINIPORT_EXECUTE_OFFLINE_DIAGS
                || c == IOCTL_SCSI_MINIPORT_ENABLE_DISABLE_AUTO_OFFLINE
                || c == IOCTL_SCSI_MINIPORT_READ_SMART_LOG
                || c == IOCTL_SCSI_MINIPORT_WRITE_SMART_LOG =>
            {
                let cmdInParameters =
                    ((*Srb).DataBuffer as PUCHAR).add(core::mem::size_of::<SRB_IO_CONTROL>())
                        as PSENDCMDINPARAMS;
                if (len as usize)
                    < core::mem::size_of::<SRB_IO_CONTROL>() + core::mem::size_of::<SENDCMDINPARAMS>() - 1
                {
                    wrong_buffer!();
                }
                targetId = (*cmdInParameters).bDriveNumber as ULONG;
            }
            _ => {
                // invalid_request -> SRB_STATUS_INVALID_REQUEST, goto complete_req
                ret!(SRB_STATUS_INVALID_REQUEST as ULONG);
            }
        }

        // adjust (if necessary) bus address
        if targetId != !0u32 {
            if (*deviceExtension).NumberChannels == 1 {
                DeviceNumber = targetId & 0x01;
                lChannel = 0;
            } else if commPort != 0 {
                if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
                    lChannel = (targetId / 2) as UCHAR;
                    DeviceNumber = 0;
                } else {
                    lChannel = (targetId / 2) as UCHAR;
                    DeviceNumber = targetId & 0x01;
                }
            }
            if lChannel as ULONG >= (*deviceExtension).NumberChannels as ULONG
                || DeviceNumber >= (*deviceExtension).NumberLuns
            {
                reject!();
            }
            targetId = lChannel as ULONG * (*deviceExtension).NumberLuns + DeviceNumber;
            chan = (*deviceExtension).chan.add(lChannel as usize);
            LunExt = (*chan).lun[DeviceNumber as usize];
            if LunExt.is_null() {
                reject!();
            }
            atapiDev = if (*LunExt).DeviceFlags & DFLAGS_ATAPI_DEVICE != 0 { TRUE as BOOLEAN } else { FALSE as BOOLEAN };
            if (*LunExt).DeviceFlags & DFLAGS_DEVICE_PRESENT == 0 {
                reject!();
            }
        }

        match (*sig).ControlCode {
            c if c == IOCTL_SCSI_MINIPORT_SMART_VERSION => {
                let versionParameters =
                    ((*Srb).DataBuffer as PUCHAR).add(core::mem::size_of::<SRB_IO_CONTROL>())
                        as PGETVERSIONINPARAMS;
                let deviceNumberMap: UCHAR;

                (*versionParameters).bVersion = 1;
                (*versionParameters).bRevision = 1;
                (*versionParameters).bReserved = 0;
                (*versionParameters).fCapabilities =
                    (CAP_ATA_ID_CMD | CAP_ATAPI_ID_CMD | CAP_SMART_CMD) as ULONG;

                if atapiDev != 0 {
                    ret!(SRB_STATUS_INVALID_REQUEST as ULONG);
                }

                if (*(*chan).DeviceExtension).HwFlags & UNIATA_AHCI != 0 {
                    deviceNumberMap = (1u32 << lChannel) as UCHAR;
                    DeviceNumber = 0;
                } else if (*deviceExtension).NumberChannels == 1 {
                    if (*chan).PrimaryAddress != 0 {
                        deviceNumberMap = (1u32 << DeviceNumber) as UCHAR;
                    } else {
                        deviceNumberMap = (4u32 << DeviceNumber) as UCHAR;
                    }
                } else {
                    deviceNumberMap = (1u32 << (DeviceNumber + lChannel as ULONG * 2)) as UCHAR;
                }
                (*versionParameters).bIDEDeviceMap = deviceNumberMap;
                ret!(SRB_STATUS_SUCCESS as ULONG);
            }
            c if c == IOCTL_SCSI_MINIPORT_IDENTIFY => {
                let cmdOutParameters =
                    ((*Srb).DataBuffer as PUCHAR).add(core::mem::size_of::<SRB_IO_CONTROL>())
                        as PSENDCMDOUTPARAMS;
                let cmdInParameters: SENDCMDINPARAMS = *(((*Srb).DataBuffer as PUCHAR)
                    .add(core::mem::size_of::<SRB_IO_CONTROL>())
                    as PSENDCMDINPARAMS);

                let cmd = cmdInParameters.irDriveRegs.bCommandReg;
                if cmd == ID_CMD {
                    if atapiDev != 0 {
                        ret!(SRB_STATUS_INVALID_REQUEST as ULONG);
                    }
                    // FALL THROUGH to ATAPI_ID_CMD handling below
                }
                if cmd == ID_CMD || cmd == ATAPI_ID_CMD {
                    if atapiDev == 0 && cmd == ATAPI_ID_CMD {
                        ret!(SRB_STATUS_INVALID_REQUEST as ULONG);
                    }

                    let mut len2 = min_u32(
                        len,
                        (core::mem::size_of::<SENDCMDOUTPARAMS>() as ULONG - 1) + IDENTIFY_BUFFER_SIZE,
                    );
                    RtlZeroMemory(cmdOutParameters as PVOID, len2 as usize);

                    (*cmdOutParameters).cBufferSize = min_u32(
                        IDENTIFY_BUFFER_SIZE,
                        len2 - core::mem::size_of::<SENDCMDOUTPARAMS>() as ULONG + 1,
                    );
                    (*cmdOutParameters).DriverStatus.bDriverError = 0;
                    (*cmdOutParameters).DriverStatus.bIDEError = 0;

                    ScsiPortMoveMemory(
                        (*cmdOutParameters).bBuffer.as_mut_ptr() as PVOID,
                        (&mut (*LunExt).IdentifyData) as *mut _ as PVOID,
                        (*cmdOutParameters).cBufferSize,
                    );

                    if ((*cmdOutParameters).cBufferSize == IDENTIFY_BUFFER_SIZE)
                        && ((*LunExt).IdentifyData.__bindgen_anon_5.__bindgen_anon_1.as_ref().ChecksumValid() as ULONG == ATA_ChecksumValid)
                    {
                        let mut csum: CHAR = 0;
                        let mut i: ULONG = 0;
                        while i < IDENTIFY_BUFFER_SIZE - 1 {
                            csum = csum.wrapping_add(*(*cmdOutParameters).bBuffer.as_ptr().add(i as usize) as CHAR);
                            i += 1;
                        }
                        *(*cmdOutParameters).bBuffer.as_mut_ptr().add(i as usize) = (-(csum as i32)) as UCHAR;
                    }
                    let _ = len2;
                    ret!(SRB_STATUS_SUCCESS as ULONG);
                }
                ret!(SRB_STATUS_INVALID_REQUEST as ULONG);
            }
            _ => {
                // *all* remaining IOCTLs here are SMART
                if atapiDev != 0 {
                    ret!(SRB_STATUS_INVALID_REQUEST as ULONG);
                }

                let PostReq = UniataNeedQueueing(deviceExtension, chan, TopLevel);
                if PostReq != 0 || TopLevel != 0 {
                    UniataQueueRequest(chan, Srb);
                    let AtaReq = (*Srb).SrbExtension as PATA_REQ;
                    (*ata_req_core(AtaReq)).ReqState = REQ_STATE_QUEUED as UCHAR;
                }
                if PostReq != 0 {
                    ret!(SRB_STATUS_PENDING as ULONG);
                } else {
                    status = IdeSendSmartCommand(HwDeviceExtension, Srb, targetId);
                    ret!(status);
                }
            }
        }
    } else if _strnicmp((*sig).Signature.as_ptr() as PCCHAR, b"-UNIATA-\0".as_ptr() as PCCHAR, "-UNIATA-".len()) == 0 {
        let AtaCtl = (*Srb).DataBuffer as PUNIATA_CTL;
        let mut DeviceNumber2: ULONG = (*AtaCtl).addr.TargetId as ULONG;
        let bad_ldev: bool;
        let pos: ULONG = core::mem::offset_of!(_UNIATA_CTL, __bindgen_anon_1) as ULONG;

        if (len as usize) < pos as usize {
            wrong_buffer!();
        }

        if (*AtaCtl).addr.Lun != 0
            || ((*AtaCtl).addr.TargetId as ULONG >= (*deviceExtension).NumberLuns)
            || ((*AtaCtl).addr.PathId as ULONG >= (*deviceExtension).NumberChannels as ULONG)
        {
            chan = core::ptr::null_mut();
            bad_ldev = true;
            LunExt = core::ptr::null_mut();
        } else {
            bad_ldev = false;
            lChannel = (*AtaCtl).addr.PathId;
            chan = (*deviceExtension).chan.add(lChannel as usize);
            LunExt = (*chan).lun[DeviceNumber2 as usize];
        }
        DeviceNumber = DeviceNumber2;

        // check for valid LUN
        let cc = (*AtaCtl).hdr.ControlCode;
        let mut handled_bad = false;
        if cc == IOCTL_SCSI_MINIPORT_UNIATA_FIND_DEVICES || cc == IOCTL_SCSI_MINIPORT_UNIATA_RESET_DEVICE {
            if bad_ldev
                && ((*AtaCtl).addr.PathId as ULONG >= (*deviceExtension).NumberChannels as ULONG
                    || (*AtaCtl).addr.TargetId != 0xff
                    || (*AtaCtl).addr.Lun != 0)
            {
                if cc == IOCTL_SCSI_MINIPORT_UNIATA_FIND_DEVICES
                    && DeviceNumber2 < (*deviceExtension).NumberLuns
                {
                    lChannel = (*AtaCtl).addr.PathId;
                    chan = (*deviceExtension).chan.add(lChannel as usize);
                    LunExt = (*chan).lun[DeviceNumber2 as usize];
                } else {
                    reject!(); // handle_bad_ldev
                }
            } else {
                lChannel = (*AtaCtl).addr.PathId;
                chan = (*deviceExtension).chan.add(lChannel as usize);
            }
        } else if cc == IOCTL_SCSI_MINIPORT_UNIATA_DELETE_DEVICE
            || cc == IOCTL_SCSI_MINIPORT_UNIATA_SET_MAX_MODE
            || cc == IOCTL_SCSI_MINIPORT_UNIATA_GET_MODE
            || cc == IOCTL_SCSI_MINIPORT_UNIATA_RESETBB
        {
            if bad_ldev {
                reject!(); // handle_bad_ldev
            }
        }
        let _ = handled_bad;

        // check if queueing is necessary
        let mut do_queue = false;
        if cc == IOCTL_SCSI_MINIPORT_UNIATA_RESETBB {
            if (*LunExt).nBadBlocks != 0 {
                do_queue = true;
            }
        } else if cc == IOCTL_SCSI_MINIPORT_UNIATA_SET_MAX_MODE {
            if (len as usize) < pos as usize + core::mem::size_of::<SETTRANSFERMODE>() {
                wrong_buffer!();
            }
            if (*AtaCtl).__bindgen_anon_1.SetMode.as_ref().ApplyImmediately != 0 {
                do_queue = true;
            }
        } else if cc == IOCTL_SCSI_MINIPORT_UNIATA_FIND_DEVICES {
            do_queue = true;
        }
        if do_queue {
            let PostReq = UniataNeedQueueing(deviceExtension, chan, TopLevel);
            if PostReq != 0 || TopLevel != 0 {
                UniataQueueRequest(chan, Srb);
                let AtaReq = (*Srb).SrbExtension as PATA_REQ;
                (*ata_req_core(AtaReq)).ReqState = REQ_STATE_QUEUED as UCHAR;
            }
            if PostReq != 0 {
                ret!(SRB_STATUS_PENDING as ULONG);
            }
        }

        // process request
        match cc {
            c if c == IOCTL_SCSI_MINIPORT_UNIATA_FIND_DEVICES => {
                if (len as usize) < pos as usize + core::mem::size_of::<ADDREMOVEDEV>() {
                    wrong_buffer!();
                }
                if (*AtaCtl).__bindgen_anon_1.FindDelDev.as_ref().Flags & UNIATA_ADD_FLAGS_UNHIDE != 0 {
                    if (*AtaCtl).addr.TargetId != 0xff {
                        (*LunExt).DeviceFlags &= !DFLAGS_HIDDEN;
                    }
                }
                let mut i: ULONG = 0;
                while i < (*AtaCtl).__bindgen_anon_1.FindDelDev.as_ref().WaitForPhysicalLink && i < 30 {
                    ScsiPortStallExecution(1000 * 1000);
                    i += 1;
                }
                FindDevices(
                    HwDeviceExtension,
                    if (*AtaCtl).addr.TargetId == 0xff
                        && ((*AtaCtl).__bindgen_anon_1.FindDelDev.as_ref().Flags & UNIATA_ADD_FLAGS_UNHIDE != 0)
                    {
                        UNIATA_FIND_DEV_UNHIDE
                    } else {
                        0
                    },
                    (*AtaCtl).addr.PathId as ULONG,
                );
                ret!(SRB_STATUS_SUCCESS as ULONG);
            }
            c if c == IOCTL_SCSI_MINIPORT_UNIATA_DELETE_DEVICE => {
                if (len as usize) < pos as usize + core::mem::size_of::<ADDREMOVEDEV>() {
                    wrong_buffer!();
                }
                (*LunExt).DeviceFlags = 0;
                if (*AtaCtl).__bindgen_anon_1.FindDelDev.as_ref().Flags & UNIATA_REMOVE_FLAGS_HIDE != 0 {
                    UniataForgetDevice(LunExt);
                }
                let mut i: ULONG = 0;
                while i < (*AtaCtl).__bindgen_anon_1.FindDelDev.as_ref().WaitForPhysicalLink && i < 30 {
                    ScsiPortStallExecution(1000 * 1000);
                    i += 1;
                }
                ret!(SRB_STATUS_SUCCESS as ULONG);
            }
            c if c == IOCTL_SCSI_MINIPORT_UNIATA_SET_MAX_MODE => {
                if (len as usize) < pos as usize + core::mem::size_of::<SETTRANSFERMODE>() {
                    wrong_buffer!();
                }
                let sm = (*AtaCtl).__bindgen_anon_1.SetMode.as_ref();
                if sm.OrigMode != IOMODE_NOT_SPECIFIED {
                    (*LunExt).OrigTransferMode = sm.OrigMode as UCHAR;
                }
                if sm.MaxMode != IOMODE_NOT_SPECIFIED {
                    (*LunExt).LimitedTransferMode = sm.MaxMode as UCHAR;
                    if (*LunExt).LimitedTransferMode > (*LunExt).OrigTransferMode {
                        (*LunExt).LimitedTransferMode = (*LunExt).OrigTransferMode;
                    }
                }
                (*LunExt).TransferMode =
                    min_u32((*LunExt).LimitedTransferMode as ULONG, (*LunExt).OrigTransferMode as ULONG) as UCHAR;
                (*LunExt).DeviceFlags |= DFLAGS_REINIT_DMA;
                if (*AtaCtl).__bindgen_anon_1.SetMode.as_ref().ApplyImmediately != 0 {
                    AtapiDmaInit__(deviceExtension, LunExt);
                }
                ret!(SRB_STATUS_SUCCESS as ULONG);
            }
            c if c == IOCTL_SCSI_MINIPORT_UNIATA_GET_MODE => {
                if (len as usize) < pos as usize + core::mem::size_of::<GETTRANSFERMODE>() {
                    wrong_buffer!();
                }
                let gm = (*AtaCtl).__bindgen_anon_1.GetMode.as_mut();
                gm.OrigMode = (*LunExt).OrigTransferMode as ULONG;
                gm.MaxMode = (*LunExt).LimitedTransferMode as ULONG;
                gm.CurrentMode = (*LunExt).TransferMode as ULONG;
                gm.PhyMode = (*LunExt).PhyTransferMode as ULONG;
                ret!(SRB_STATUS_SUCCESS as ULONG);
            }
            c if c == IOCTL_SCSI_MINIPORT_UNIATA_GET_VERSION => {
                if (len as usize) < pos as usize + core::mem::size_of::<GETDRVVERSION>() {
                    wrong_buffer!();
                }
                let v = (*AtaCtl).__bindgen_anon_1.Version.as_mut();
                v.Length = core::mem::size_of::<GETDRVVERSION>() as ULONG;
                v.VersionMj = UNIATA_VER_MJ as USHORT;
                v.VersionMn = UNIATA_VER_MN as USHORT;
                v.SubVerMj = UNIATA_VER_SUB_MJ as USHORT;
                v.SubVerMn = UNIATA_VER_SUB_MN as USHORT;
                ret!(SRB_STATUS_SUCCESS as ULONG);
            }
            c if c == IOCTL_SCSI_MINIPORT_UNIATA_ADAPTER_INFO => {
                if (len as usize) < pos as usize + core::mem::size_of::<ADAPTERINFO>() {
                    wrong_buffer!();
                }
                let mut pos2 = pos;
                {
                    let ai = (*AtaCtl).__bindgen_anon_1.AdapterInfo.as_mut();
                    ai.HeaderLength = core::mem::size_of::<ADAPTERINFO>() as ULONG;
                    ai.DevID = (*deviceExtension).DevID;
                    ai.RevID = (*deviceExtension).RevID;
                    ai.slotNumber = (*deviceExtension).slotNumber;
                    ai.SystemIoBusNumber = (*deviceExtension).SystemIoBusNumber;
                    ai.DevIndex = (*deviceExtension).DevIndex;
                    ai.Channel = (*deviceExtension).Channel;
                    ai.HbaCtrlFlags = (*deviceExtension).HbaCtrlFlags;
                    ai.simplexOnly = (*deviceExtension).simplexOnly;
                    ai.MemIo = FALSE as BOOLEAN;
                    ai.UnknownDev = (*deviceExtension).UnknownDev;
                    ai.MasterDev = (*deviceExtension).MasterDev;
                    ai.MaxTransferMode = (*deviceExtension).MaxTransferMode;
                    ai.HwFlags = (*deviceExtension).HwFlags;
                    ai.OrigAdapterInterfaceType = (*deviceExtension).OrigAdapterInterfaceType as ULONG;
                    ai.BusInterruptLevel = (*deviceExtension).BusInterruptLevel;
                    ai.InterruptMode = (*deviceExtension).InterruptMode as ULONG;
                    ai.BusInterruptVector = (*deviceExtension).BusInterruptVector;
                    ai.NumberChannels = (*deviceExtension).NumberChannels as ULONG;
                    ai.NumberLuns = (*deviceExtension).NumberLuns as UCHAR;
                    ai.AdapterInterfaceType = (*deviceExtension).AdapterInterfaceType as ULONG;
                    if !(*deviceExtension).FullDevName.is_null() {
                        strncpy(ai.DeviceName.as_mut_ptr(), (*deviceExtension).FullDevName, 64);
                    }
                    ai.ChanInfoValid = FALSE as BOOLEAN;
                    ai.LunInfoValid = FALSE as BOOLEAN;
                    ai.ChanHeaderLengthValid = TRUE as BOOLEAN;
                    pos2 += ai.HeaderLength;
                }

                RtlZeroMemory(((AtaCtl as PCHAR).add(pos2 as usize)) as PVOID, (len - pos2) as usize);

                let nchan = (*AtaCtl).__bindgen_anon_1.AdapterInfo.as_ref().NumberChannels;
                if len as usize >= pos2 as usize + nchan as usize * core::mem::size_of::<CHANINFO>() {
                    let mut ChanInfo = ((AtaCtl as PCHAR).add(pos2 as usize)) as PCHANINFO;
                    let mut i: ULONG = 0;
                    while i < nchan {
                        let cur_chan = (*deviceExtension).chan.add(i as usize);
                        (*ChanInfo).MaxTransferMode = (*cur_chan).MaxTransferMode;
                        (*ChanInfo).ChannelCtrlFlags = (*cur_chan).ChannelCtrlFlags;
                        RtlCopyMemory(
                            (*ChanInfo).QueueStat.as_mut_ptr() as PVOID,
                            (*cur_chan).QueueStat.as_ptr() as PVOID,
                            core::mem::size_of_val(&(*ChanInfo).QueueStat),
                        );
                        (*ChanInfo).ReorderCount = (*cur_chan).ReorderCount;
                        (*ChanInfo).IntersectCount = (*cur_chan).IntersectCount;
                        (*ChanInfo).TryReorderCount = (*cur_chan).TryReorderCount;
                        (*ChanInfo).TryReorderHeadCount = (*cur_chan).TryReorderHeadCount;
                        (*ChanInfo).TryReorderTailCount = (*cur_chan).TryReorderTailCount;
                        ChanInfo = ChanInfo.add(1);
                        i += 1;
                    }
                    let ai = (*AtaCtl).__bindgen_anon_1.AdapterInfo.as_mut();
                    ai.ChanInfoValid = TRUE as BOOLEAN;
                    ai.ChanHeaderLength = core::mem::size_of::<CHANINFO>() as ULONG;
                }
                ret!(SRB_STATUS_SUCCESS as ULONG);
            }
            c if c == IOCTL_SCSI_MINIPORT_UNIATA_RESETBB => {
                ForgetBadBlocks(LunExt);
                ret!(SRB_STATUS_SUCCESS as ULONG);
            }
            c if c == IOCTL_SCSI_MINIPORT_UNIATA_RESET_DEVICE => {
                if bad_ldev {
                    // do_bus_reset
                    if AtapiResetController__(deviceExtension as PVOID, lChannel as ULONG, RESET_COMPLETE_ALL) == 0 {
                        ScsiPortLogError(HwDeviceExtension, core::ptr::null_mut(), 0, 0, 0, SP_INTERNAL_ADAPTER_ERROR, 5 << 8);
                        ret!(SRB_STATUS_ERROR as ULONG);
                    } else {
                        ret!(SRB_STATUS_SUCCESS as ULONG);
                    }
                } else {
                    UniataUserDeviceReset(deviceExtension, LunExt, (*AtaCtl).addr.PathId as ULONG);
                }
                ret!(SRB_STATUS_SUCCESS as ULONG);
            }
            _ => {
                ret!(SRB_STATUS_INVALID_REQUEST as ULONG);
            }
        }
    } else {
        ret!(SRB_STATUS_INVALID_REQUEST as ULONG);
    }
}

// ===========================================================================
//  BuildMechanismStatusSrb  (#ifndef UNIATA_CORE)
// ===========================================================================
pub unsafe extern "system" fn BuildMechanismStatusSrb(
    HwDeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
) -> PSCSI_REQUEST_BLOCK {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let AtaReq = (*Srb).SrbExtension as PATA_REQ;

    let srb: PSCSI_REQUEST_BLOCK = &mut (*(*deviceExtension).chan.add((*Srb).PathId as usize)).InternalSrb;

    RtlZeroMemory(srb as PVOID, core::mem::size_of::<SCSI_REQUEST_BLOCK>());

    (*srb).PathId = (*Srb).PathId;
    (*srb).TargetId = (*Srb).TargetId;
    (*srb).Function = SRB_FUNCTION_EXECUTE_SCSI as UCHAR;
    (*srb).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;

    (*srb).SrbFlags = SRB_FLAGS_DATA_IN | SRB_FLAGS_DISABLE_SYNCH_TRANSFER;
    (*srb).TimeOutValue = 4;

    (*srb).CdbLength = 6;
    (*srb).DataBuffer = (&mut (*(*deviceExtension).chan.add((*Srb).PathId as usize)).MechStatusData) as *mut _ as PVOID;
    (*srb).DataTransferLength =
        core::mem::size_of::<MECHANICAL_STATUS_INFORMATION_HEADER>() as ULONG;
    (*srb).SrbExtension = AtaReq as PVOID;

    let cdb = (*srb).Cdb.as_mut_ptr() as PCDB;
    (*cdb).MECH_STATUS.as_mut().OperationCode = SCSIOP_MECHANISM_STATUS as UCHAR;
    (*cdb).MECH_STATUS.as_mut().AllocationLength[1] =
        core::mem::size_of::<MECHANICAL_STATUS_INFORMATION_HEADER>() as UCHAR;

    srb
}

// ===========================================================================
//  BuildRequestSenseSrb
// ===========================================================================
pub unsafe extern "system" fn BuildRequestSenseSrb(
    HwDeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
) -> PSCSI_REQUEST_BLOCK {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let AtaReq = (*Srb).SrbExtension as PATA_REQ;

    let srb: PSCSI_REQUEST_BLOCK = &mut (*(*deviceExtension).chan.add((*Srb).PathId as usize)).InternalSrb;

    RtlZeroMemory(srb as PVOID, core::mem::size_of::<SCSI_REQUEST_BLOCK>());

    (*srb).PathId = (*Srb).PathId;
    (*srb).TargetId = (*Srb).TargetId;
    (*srb).Function = SRB_FUNCTION_EXECUTE_SCSI as UCHAR;
    (*srb).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;

    (*srb).SrbFlags = SRB_FLAGS_DATA_IN | SRB_FLAGS_DISABLE_SYNCH_TRANSFER;
    (*srb).TimeOutValue = 4;

    (*srb).CdbLength = 6;
    (*srb).DataBuffer = (&mut (*(*deviceExtension).chan.add((*Srb).PathId as usize)).MechStatusSense) as *mut _ as PVOID;
    (*srb).DataTransferLength = core::mem::size_of::<SENSE_DATA>() as ULONG;
    (*srb).SrbExtension = AtaReq as PVOID;

    let cdb = (*srb).Cdb.as_mut_ptr() as PCDB;
    (*cdb).CDB6INQUIRY.as_mut().OperationCode = SCSIOP_REQUEST_SENSE as UCHAR;
    (*cdb).CDB6INQUIRY.as_mut().AllocationLength = core::mem::size_of::<SENSE_DATA>() as UCHAR;

    srb
}

// ===========================================================================
//  EncodeVendorStr
// ===========================================================================
pub unsafe extern "system" fn EncodeVendorStr(
    Buffer: PWCHAR,
    Str: PUCHAR,
    Length: ULONG,
) -> ULONG {
    let mut i: ULONG = 0;
    let mut j: ULONG = 0;
    let mut a: WCHAR;

    while i < Length {
        // fix byte-order
        a = *Str.add((i ^ 0x01) as usize) as WCHAR;
        if a == 0 {
            *Buffer.add(j as usize) = 0;
            return j;
        } else if a == b' ' as WCHAR {
            *Buffer.add(j as usize) = b'_' as WCHAR;
        } else if a == b'_' as WCHAR
            || a == b'#' as WCHAR
            || a == b'\\' as WCHAR
            || a == b'"' as WCHAR
            || a == b'\'' as WCHAR
            || a < b' ' as WCHAR
            || a >= 127
        {
            *Buffer.add(j as usize) = b'#' as WCHAR;
            j += 1;
            swprintf(Buffer.add(j as usize), WFMT_HEX2.as_ptr(), a as core::ffi::c_uint);
            j += 1;
        } else {
            *Buffer.add(j as usize) = a;
        }
        i += 1;
        j += 1;
    }
    *Buffer.add(j as usize) = 0;
    j
}

// ===========================================================================
//  AtapiAdapterControl
// ===========================================================================
pub unsafe extern "system" fn AtapiAdapterControl(
    HwDeviceExtension: PVOID,
    ControlType: SCSI_ADAPTER_CONTROL_TYPE,
    Parameters: PVOID,
) -> SCSI_ADAPTER_CONTROL_STATUS {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let numberChannels: ULONG = (*deviceExtension).NumberChannels as ULONG;
    let mut c: ULONG;
    let status: NTSTATUS;

    match ControlType {
        t if t == _SCSI_ADAPTER_CONTROL_TYPE_ScsiQuerySupportedControlTypes as SCSI_ADAPTER_CONTROL_TYPE => {
            let supportedTypes: [BOOLEAN; 5] = [
                TRUE as BOOLEAN,  // ScsiQuerySupportedControlTypes
                TRUE as BOOLEAN,  // ScsiStopAdapter
                TRUE as BOOLEAN,  // ScsiRestartAdapter
                FALSE as BOOLEAN, // ScsiSetBootConfig
                FALSE as BOOLEAN, // ScsiSetRunningConfig
            ];
            let mut lim: ULONG = _SCSI_ADAPTER_CONTROL_TYPE_ScsiAdapterControlMax as ULONG;
            let pControlTypeList = Parameters as PSCSI_SUPPORTED_CONTROL_TYPE_LIST;

            if (*pControlTypeList).MaxControlType < lim {
                lim = (*pControlTypeList).MaxControlType;
            }
            let mut i: ULONG = 0;
            while i < lim {
                *(*pControlTypeList).SupportedTypeList.as_mut_ptr().add(i as usize) =
                    supportedTypes[i as usize];
                i += 1;
            }
        }
        t if t == _SCSI_ADAPTER_CONTROL_TYPE_ScsiStopAdapter as SCSI_ADAPTER_CONTROL_TYPE => {
            // Shut down all interrupts; reinit re-enables them.
            c = 0;
            while c < numberChannels {
                AtapiResetController(deviceExtension as PVOID, c);
                AtapiDisableInterrupts(deviceExtension as PVOID, c);
                c += 1;
            }
            if (*deviceExtension).AdapterInterfaceType as ULONG == _INTERFACE_TYPE_PCIBus as ULONG {
                UniataDisconnectIntr2(HwDeviceExtension);
                (*BMList.add((*deviceExtension).DevIndex as usize)).Isr2Enable = FALSE as BOOLEAN;
            }
        }
        t if t == _SCSI_ADAPTER_CONTROL_TYPE_ScsiRestartAdapter as SCSI_ADAPTER_CONTROL_TYPE => {
            AtapiChipInit(HwDeviceExtension, DEVNUM_NOT_SPECIFIED, CHAN_NOT_SPECIFIED);
            status = UniataConnectIntr2(HwDeviceExtension);
            if NT_SUCCESS(status) {
                c = 0;
                while c < numberChannels {
                    AtapiChipInit(HwDeviceExtension, DEVNUM_NOT_SPECIFIED, c);
                    FindDevices(HwDeviceExtension, 0, c);
                    AtapiEnableInterrupts(deviceExtension as PVOID, c);
                    AtapiHwInitialize__(deviceExtension, c);
                    c += 1;
                }
                if !(*deviceExtension).Isr2DevObj.is_null() {
                    (*BMList.add((*deviceExtension).DevIndex as usize)).Isr2Enable = TRUE as BOOLEAN;
                }
            }
        }
        _ => {
            return _SCSI_ADAPTER_CONTROL_STATUS_ScsiAdapterControlUnsuccessful as SCSI_ADAPTER_CONTROL_STATUS;
        }
    }

    _SCSI_ADAPTER_CONTROL_STATUS_ScsiAdapterControlSuccess as SCSI_ADAPTER_CONTROL_STATUS
}

// ===========================================================================
//  _PrintNtConsole  (sole _cdecl variadic helper)
// ===========================================================================
const DEBUG_MSG_BUFFER_SIZE: usize = 512;

/// Fixed-arity twin of the `_PrintNtConsole` C-variadic entry point in
/// `drivers/storage/ide/uniata/rust_varargs_shim.c`. The C side exists only to do the
/// `va_start` that stable Rust cannot (`c_variadic`, rust-lang#44930); `ap` is a pointer
/// to its `va_list`. All the logic — the 512-byte buffer, the NUL termination, the
/// HalDisplayString call — lives here; C only forwards the list into `_vsnprintf`.
/// Call sites keep calling `_PrintNtConsole(...)` (the extern variadic decl in types_*.rs).
#[no_mangle]
pub unsafe extern "C" fn _PrintNtConsole_v(DebugMessage: PCCH, ap: *mut core::ffi::c_void) {
    let mut dbg_print_tmp_buff: [UCHAR; DEBUG_MSG_BUFFER_SIZE] = [0; DEBUG_MSG_BUFFER_SIZE];

    vibeos_vsnprintf(
        dbg_print_tmp_buff.as_mut_ptr() as *mut CHAR,
        DEBUG_MSG_BUFFER_SIZE - 1,
        DebugMessage as *const CHAR,
        ap,
    );

    dbg_print_tmp_buff[DEBUG_MSG_BUFFER_SIZE - 1] = 0;
    HalDisplayString(dbg_print_tmp_buff.as_mut_ptr() as *const CHAR);
    // _DEBUG g_LogToDisplay stall dropped (NDEBUG build path).
}

// ---- wide string literal helper (build [u16; N] from an ASCII byte string) ----
// N MUST be > b.len() so the trailing slot stays 0 (NUL-terminated) — the wide C
// string is handed to swprintf/RtlQueryRegistryValues, which scan for the NUL. A
// too-tight N (N == b.len()) drops the terminator → the formatter walks off the end
// into garbage (BUGCHECK 0x1E in wcsnlen). The const assert makes that a build error.
const fn wlit<const N: usize>(b: &[u8]) -> [u16; N] {
    assert!(N > b.len(), "wlit: array must leave room for the NUL terminator");
    let mut out = [0u16; N];
    let mut i = 0;
    while i < b.len() {
        out[i] = b[i] as u16;
        i += 1;
    }
    out
}

// ===========================================================================
//  AtapiRegCheckDevLunValue
// ===========================================================================
pub unsafe extern "system" fn AtapiRegCheckDevLunValue(
    HwDeviceExtension: PVOID,
    NamePrefix: PCWCH,
    chan: ULONG,
    dev: ULONG,
    Name: PCWSTR,
    Default: ULONG,
) -> ULONG {
    let mut namex: [WCHAR; 160] = [0; 160];
    let mut val: ULONG = Default;

    val = AtapiRegCheckParameterValue(HwDeviceExtension, NamePrefix, Name, val);

    if chan != CHAN_NOT_SPECIFIED {
        // swprintf(namex, L"%s\\Chan_%1.1d", NamePrefix, chan)
        static FMT_CHAN: [u16; 14] = wlit::<14>(b"%s\\Chan_%1.1d");
        swprintf(namex.as_mut_ptr(), FMT_CHAN.as_ptr(), NamePrefix, chan);
        val = AtapiRegCheckParameterValue(HwDeviceExtension, namex.as_ptr(), Name, val);
        if dev != DEVNUM_NOT_SPECIFIED {
            // swprintf(namex, L"%s\\Chan_%1.1d\\%s", NamePrefix, chan, dev&1 ? L"Lun_1" : L"Lun_0")
            static FMT_LUN: [u16; 17] = wlit::<17>(b"%s\\Chan_%1.1d\\%s");
            static LUN1: [u16; 6] = wlit::<6>(b"Lun_1");
            static LUN0: [u16; 6] = wlit::<6>(b"Lun_0");
            let lun = if dev & 0x01 != 0 { LUN1.as_ptr() } else { LUN0.as_ptr() };
            swprintf(namex.as_mut_ptr(), FMT_LUN.as_ptr(), NamePrefix, chan, lun);
            val = AtapiRegCheckParameterValue(HwDeviceExtension, namex.as_ptr(), Name, val);
        }
    }
    val
}

// ===========================================================================
//  AtapiRegCheckDevValue
// ===========================================================================
pub unsafe extern "system" fn AtapiRegCheckDevValue(
    HwDeviceExtension: PVOID,
    chan: ULONG,
    dev: ULONG,
    Name: PCWSTR,
    Default: ULONG,
) -> ULONG {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let mut namex: [WCHAR; 160] = [0; 160];
    let mut namev: [WCHAR; 16] = [0; 16];
    let mut named: [WCHAR; 16] = [0; 16];
    let mut names: [WCHAR; 20] = [0; 20];

    let VendorID: ULONG;
    let DeviceID: ULONG;
    let SlotNumber: ULONG;
    let HwFlags: ULONG;

    let mut val: ULONG = Default;

    if !deviceExtension.is_null() {
        VendorID = (*deviceExtension).DevID & 0xffff;
        DeviceID = ((*deviceExtension).DevID >> 16) & 0xffff;
        SlotNumber = (*deviceExtension).slotNumber;
        HwFlags = (*deviceExtension).HwFlags;
    } else {
        VendorID = 0xffff;
        DeviceID = 0xffff;
        SlotNumber = 0xffffffff;
        HwFlags = 0;
    }

    static PARAMETERS: [u16; 11] = wlit::<11>(b"Parameters");
    static FMT_PARAMS_S: [u16; 13] = wlit::<13>(b"Parameters%s");
    static FMT_PARAMS_SS: [u16; 15] = wlit::<15>(b"Parameters%s%s");
    static FMT_PARAMS_SSS: [u16; 17] = wlit::<17>(b"Parameters%s%s%s");

    val = AtapiRegCheckDevLunValue(HwDeviceExtension, PARAMETERS.as_ptr(), chan, dev, Name, val);

    if !deviceExtension.is_null() {
        if HwFlags & UNIATA_SATA != 0 {
            static SATA: [u16; 6] = wlit::<6>(b"\\SATA");
            swprintf(namev.as_mut_ptr(), wlit::<3>(b"%s").as_ptr(), SATA.as_ptr());
            swprintf(namex.as_mut_ptr(), FMT_PARAMS_S.as_ptr(), namev.as_ptr());
            val = AtapiRegCheckDevLunValue(HwDeviceExtension, namex.as_ptr(), CHAN_NOT_SPECIFIED, dev, Name, val);
        }
        if HwFlags & UNIATA_AHCI != 0 {
            static AHCI: [u16; 6] = wlit::<6>(b"\\AHCI");
            swprintf(namev.as_mut_ptr(), wlit::<3>(b"%s").as_ptr(), AHCI.as_ptr());
            swprintf(namex.as_mut_ptr(), FMT_PARAMS_S.as_ptr(), namev.as_ptr());
            val = AtapiRegCheckDevLunValue(HwDeviceExtension, namex.as_ptr(), CHAN_NOT_SPECIFIED, dev, Name, val);
        }
        if HwFlags & (UNIATA_SATA | UNIATA_AHCI) == 0 {
            static PATA: [u16; 6] = wlit::<6>(b"\\PATA");
            swprintf(namev.as_mut_ptr(), wlit::<3>(b"%s").as_ptr(), PATA.as_ptr());
            swprintf(namex.as_mut_ptr(), FMT_PARAMS_S.as_ptr(), namev.as_ptr());
            val = AtapiRegCheckDevLunValue(HwDeviceExtension, namex.as_ptr(), CHAN_NOT_SPECIFIED, dev, Name, val);
        }

        if (*deviceExtension).AdapterInterfaceType as ULONG == _INTERFACE_TYPE_PCIBus as ULONG {
            static PCIIDE: [u16; 8] = wlit::<8>(b"\\PCIIDE");
            swprintf(namev.as_mut_ptr(), wlit::<3>(b"%s").as_ptr(), PCIIDE.as_ptr());
            swprintf(namex.as_mut_ptr(), FMT_PARAMS_S.as_ptr(), namev.as_ptr());
            val = AtapiRegCheckDevLunValue(HwDeviceExtension, namex.as_ptr(), CHAN_NOT_SPECIFIED, dev, Name, val);

            static FMT_IDE_D: [u16; 8] = wlit::<8>(b"\\IDE_%d");
            swprintf(namev.as_mut_ptr(), FMT_IDE_D.as_ptr(), (*deviceExtension).DevIndex);
            swprintf(namex.as_mut_ptr(), FMT_PARAMS_S.as_ptr(), namev.as_ptr());
            val = AtapiRegCheckDevLunValue(HwDeviceExtension, namex.as_ptr(), chan, dev, Name, val);

            static FMT_VEN: [u16; 11] = wlit::<11>(b"\\Ven_%4.4x");
            static FMT_DEV: [u16; 11] = wlit::<11>(b"\\Dev_%4.4x");
            static FMT_SLOT: [u16; 13] = wlit::<13>(b"\\Slot_%8.8x");
            swprintf(namev.as_mut_ptr(), FMT_VEN.as_ptr(), VendorID);
            swprintf(named.as_mut_ptr(), FMT_DEV.as_ptr(), DeviceID);
            swprintf(names.as_mut_ptr(), FMT_SLOT.as_ptr(), SlotNumber);

            swprintf(namex.as_mut_ptr(), FMT_PARAMS_S.as_ptr(), namev.as_ptr());
            val = AtapiRegCheckDevLunValue(HwDeviceExtension, namex.as_ptr(), chan, dev, Name, val);

            swprintf(namex.as_mut_ptr(), FMT_PARAMS_SS.as_ptr(), namev.as_ptr(), named.as_ptr());
            val = AtapiRegCheckDevLunValue(HwDeviceExtension, namex.as_ptr(), chan, dev, Name, val);

            swprintf(namex.as_mut_ptr(), FMT_PARAMS_SSS.as_ptr(), namev.as_ptr(), named.as_ptr(), names.as_ptr());
            val = AtapiRegCheckDevLunValue(HwDeviceExtension, namex.as_ptr(), chan, dev, Name, val);
        } else if (*deviceExtension).AdapterInterfaceType as ULONG == _INTERFACE_TYPE_Isa as ULONG {
            static ISAIDE: [u16; 8] = wlit::<8>(b"\\ISAIDE");
            swprintf(namev.as_mut_ptr(), wlit::<3>(b"%s").as_ptr(), ISAIDE.as_ptr());
            swprintf(namex.as_mut_ptr(), FMT_PARAMS_S.as_ptr(), namev.as_ptr());
            val = AtapiRegCheckDevLunValue(HwDeviceExtension, namex.as_ptr(), CHAN_NOT_SPECIFIED, dev, Name, val);

            static FMT_IDE_D: [u16; 8] = wlit::<8>(b"\\IDE_%d");
            swprintf(namev.as_mut_ptr(), FMT_IDE_D.as_ptr(), (*deviceExtension).DevIndex + BMListLen);
            swprintf(namex.as_mut_ptr(), FMT_PARAMS_S.as_ptr(), namev.as_ptr());
            val = AtapiRegCheckDevLunValue(HwDeviceExtension, namex.as_ptr(), chan, dev, Name, val);

            static FMT_ISA_D: [u16; 8] = wlit::<8>(b"\\ISA_%d");
            swprintf(namev.as_mut_ptr(), FMT_ISA_D.as_ptr(), (*deviceExtension).DevIndex);
            swprintf(namex.as_mut_ptr(), FMT_PARAMS_S.as_ptr(), namev.as_ptr());
            val = AtapiRegCheckDevLunValue(HwDeviceExtension, namex.as_ptr(), chan, dev, Name, val);
        } else if (*deviceExtension).AdapterInterfaceType as ULONG == _INTERFACE_TYPE_MicroChannel as ULONG {
            static MCA: [u16; 5] = wlit::<5>(b"\\MCA");
            swprintf(namev.as_mut_ptr(), wlit::<3>(b"%s").as_ptr(), MCA.as_ptr());
            swprintf(namex.as_mut_ptr(), FMT_PARAMS_S.as_ptr(), namev.as_ptr());
            val = AtapiRegCheckDevLunValue(HwDeviceExtension, namex.as_ptr(), CHAN_NOT_SPECIFIED, dev, Name, val);

            static FMT_IDE_D: [u16; 8] = wlit::<8>(b"\\IDE_%d");
            swprintf(namev.as_mut_ptr(), FMT_IDE_D.as_ptr(), (*deviceExtension).DevIndex + BMListLen + IsaCount);
            swprintf(namex.as_mut_ptr(), FMT_PARAMS_S.as_ptr(), namev.as_ptr());
            val = AtapiRegCheckDevLunValue(HwDeviceExtension, namex.as_ptr(), chan, dev, Name, val);

            static FMT_MCA_D: [u16; 8] = wlit::<8>(b"\\MCA_%d");
            swprintf(namev.as_mut_ptr(), FMT_MCA_D.as_ptr(), (*deviceExtension).DevIndex);
            swprintf(namex.as_mut_ptr(), FMT_PARAMS_S.as_ptr(), namev.as_ptr());
            val = AtapiRegCheckDevLunValue(HwDeviceExtension, namex.as_ptr(), chan, dev, Name, val);
        }
    }

    val
}

// ===========================================================================
//  AtapiRegCheckParameterValue
// ===========================================================================
pub unsafe extern "system" fn AtapiRegCheckParameterValue(
    HwDeviceExtension: PVOID,
    PathSuffix: PCWSTR,
    Name: PCWSTR,
    Default: ULONG,
) -> ULONG {
    let status: NTSTATUS;
    let mut zero: LONG = Default as LONG;
    let mut parameters: [RTL_QUERY_REGISTRY_TABLE; 2] = core::mem::zeroed();
    let mut doRun: LONG = Default as LONG;
    let RegistryPath: PUNICODE_STRING = &raw mut SavedRegPath;
    let mut paramPath: UNICODE_STRING = core::mem::zeroed();

    if g_Dump != 0 {
        return Default;
    }

    paramPath.Length = 0;
    paramPath.MaximumLength =
        ((*RegistryPath).Length as ULONG + (wcslen(PathSuffix) as ULONG + 2) * core::mem::size_of::<WCHAR>() as ULONG) as USHORT;
    paramPath.Buffer = ExAllocatePool(0 as POOL_TYPE, paramPath.MaximumLength as SIZE_T) as PWCHAR;
    if paramPath.Buffer.is_null() {
        return Default;
    }

    RtlZeroMemory(paramPath.Buffer as PVOID, paramPath.MaximumLength as usize);
    RtlAppendUnicodeToString(&mut paramPath, (*RegistryPath).Buffer as PCWSTR);
    static BACKSLASH: [u16; 2] = wlit::<2>(b"\\");
    RtlAppendUnicodeToString(&mut paramPath, BACKSLASH.as_ptr());
    // REGRTL_STR_PTYPE is empty for ReactOS DDK.
    RtlAppendUnicodeToString(&mut paramPath, PathSuffix);

    RtlZeroMemory(
        parameters.as_mut_ptr() as PVOID,
        core::mem::size_of::<RTL_QUERY_REGISTRY_TABLE>() * 2,
    );

    parameters[0].Flags = RTL_QUERY_REGISTRY_DIRECT;
    parameters[0].Name = Name;
    parameters[0].EntryContext = &mut doRun as *mut LONG as PVOID;
    parameters[0].DefaultType = REG_DWORD;
    parameters[0].DefaultData = &mut zero as *mut LONG as PVOID;
    parameters[0].DefaultLength = core::mem::size_of::<ULONG>() as ULONG;

    status = RtlQueryRegistryValues(
        RTL_REGISTRY_ABSOLUTE,
        paramPath.Buffer as PCWSTR,
        parameters.as_mut_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    ExFreePool(paramPath.Buffer as PVOID);

    if !NT_SUCCESS(status) {
        doRun = Default as LONG;
    }

    doRun as ULONG
}

// ===========================================================================
//  DriverEntry — the sole cohab export.
// ===========================================================================
#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PVOID,
    RegistryPath: PVOID,
) -> NTSTATUS {
    use crate::io_port::{g_Perf, g_PerfDt};

    let mut hwInitializationData: HW_INITIALIZATION_DATA_COMMON = core::mem::zeroed();
    let mut adapterCount: ULONG = 0;
    let mut i: ULONG;
    let mut c: ULONG;
    let mut alt: ULONG;
    let mut pref_alt: ULONG;
    let mut statusToReturn: ULONG;
    let mut newStatus: ULONG;
    let RegistryPath = RegistryPath as PUNICODE_STRING;
    let mut ReEnter: BOOLEAN = FALSE as BOOLEAN;

    let GlobalConfig: PCONFIGURATION_INFORMATION = IoGetConfigurationInformation();
    let mut PrimaryClaimed: BOOLEAN = FALSE as BOOLEAN;
    let mut SecondaryClaimed: BOOLEAN = FALSE as BOOLEAN;
    let mut IgnoreIsaCompatiblePci: BOOLEAN = FALSE as BOOLEAN;
    let mut IgnoreNativePci: BOOLEAN = FALSE as BOOLEAN;

    let mut t0: LARGE_INTEGER = core::mem::zeroed();
    let mut t1: LARGE_INTEGER = core::mem::zeroed();

    statusToReturn = 0xffffffff;

    RtlZeroMemory(
        (&mut hwInitializationData) as *mut _ as PVOID,
        core::mem::size_of::<HW_INITIALIZATION_DATA_COMMON>(),
    );

    // Set size of hwInitializationData.
    hwInitializationData.comm.HwInitializationDataSize =
        (core::mem::size_of::<HW_INITIALIZATION_DATA>()
            + if WinVer_Id() <= WinVer_NT { 0 } else { core::mem::size_of::<HW_INITIALIZATION_DATA_2K>() })
            as ULONG;

    // Set entry points (siblings are extern "system"; wrap with transmute to satisfy
    // the bindgen fn-ptr Option types — ABI is identical per arch).
    hwInitializationData.comm.HwInitialize = Some(core::mem::transmute(AtapiHwInitialize as usize));
    hwInitializationData.comm.HwResetBus = Some(core::mem::transmute(AtapiResetController as usize));
    hwInitializationData.comm.HwStartIo = Some(core::mem::transmute(AtapiStartIo as usize));
    hwInitializationData.comm.HwInterrupt = Some(core::mem::transmute(AtapiInterrupt as usize));

    hwInitializationData.comm.DeviceExtensionSize = core::mem::size_of::<HW_DEVICE_EXTENSION>() as ULONG;
    hwInitializationData.comm.SpecificLuExtensionSize = core::mem::size_of::<HW_LU_EXTENSION>() as ULONG;
    hwInitializationData.comm.SrbExtensionSize = core::mem::size_of::<ATA_REQ>() as ULONG;

    hwInitializationData.comm.MapBuffers = TRUE as BOOLEAN;

    hwInitializationData.comm.NumberOfAccessRanges = 2;
    hwInitializationData.comm.HwFindAdapter = Some(core::mem::transmute(AtapiReadArgumentString as usize));
    ScsiPortInitialize(
        DriverObject,
        RegistryPath as PVOID,
        &mut hwInitializationData.comm,
        &mut adapterCount as *mut ULONG as PVOID,
    );

    if g_Dump == 0 {
        // Connect_DbgPrint() dropped.
        static L_PRINTLOGO: [u16; 10] = wlit::<10>(b"PrintLogo");
        static L_IGNOREISAPCI: [u16; 23] = wlit::<23>(b"IgnoreIsaCompatiblePci");
        static L_IGNORENATIVEPCI: [u16; 16] = wlit::<16>(b"IgnoreNativePci");
        g_opt_Verbose = AtapiRegCheckDevValue(
            core::ptr::null_mut(), CHAN_NOT_SPECIFIED, DEVNUM_NOT_SPECIFIED, L_PRINTLOGO.as_ptr(), 0,
        ) as BOOLEAN;
        if g_opt_Verbose != 0 {
            static MSG: [u8; 38] = *b"Universal ATA driver v 0.47b (built)\n\0";
            _PrintNtConsole(MSG.as_ptr() as PCCH);
        }
        IgnoreIsaCompatiblePci = if AtapiRegCheckDevValue(
            core::ptr::null_mut(), CHAN_NOT_SPECIFIED, DEVNUM_NOT_SPECIFIED, L_IGNOREISAPCI.as_ptr(),
            IgnoreIsaCompatiblePci as ULONG,
        ) != 0 { TRUE as BOOLEAN } else { FALSE as BOOLEAN };
        IgnoreNativePci = if AtapiRegCheckDevValue(
            core::ptr::null_mut(), CHAN_NOT_SPECIFIED, DEVNUM_NOT_SPECIFIED, L_IGNORENATIVEPCI.as_ptr(),
            IgnoreNativePci as ULONG,
        ) != 0 { TRUE as BOOLEAN } else { FALSE as BOOLEAN };
    }

    if SavedDriverObject.is_null() {
        SavedDriverObject = DriverObject as PDRIVER_OBJECT;
        // USE_REACTOS_DDK path
        MajorVersion = 0x04;
        MinorVersion = 0x01;
        BuildNumber = 1;
        CPU_num = KeNumberProcessors as ULONG;

        KeQuerySystemTime(&mut t0);
        loop {
            KeQuerySystemTime(&mut t1);
            if t0.QuadPart != t1.QuadPart {
                break;
            }
        }
        t0 = t1;
        g_Perf = 0;
        loop {
            KeQuerySystemTime(&mut t1);
            g_Perf += 1;
            if t0.QuadPart != t1.QuadPart {
                break;
            }
        }
        g_PerfDt = ((t1.QuadPart - t0.QuadPart) / 10) as ULONG;
    } else {
        ReEnter = TRUE as BOOLEAN;
    }

    // (re)read bad block list
    InitBadBlocks(core::ptr::null_mut());

    if ReEnter == 0 {
        // get registry path to settings
        RtlCopyMemory(
            (&raw mut SavedRegPath) as *mut _ as PVOID,
            RegistryPath as PVOID,
            core::mem::size_of::<UNICODE_STRING>(),
        );
        SavedRegPath.Buffer = SavedRegPathBuffer.as_mut_ptr() as PWCHAR;
        SavedRegPath.Length = min_u32((*RegistryPath).Length as ULONG, 255 * core::mem::size_of::<WCHAR>() as ULONG) as USHORT;
        SavedRegPath.MaximumLength = (255 * core::mem::size_of::<WCHAR>()) as USHORT;
        RtlCopyMemory(
            SavedRegPath.Buffer as PVOID,
            (*RegistryPath).Buffer as PVOID,
            SavedRegPath.Length as usize,
        );
        *((SavedRegPath.Buffer as *mut WCHAR).add(SavedRegPath.Length as usize / core::mem::size_of::<WCHAR>())) = 0;
    }

    if WinVer_Id() >= WinVer_2k {
        static L_PNPIF: [u16; 27] = wlit::<27>(b"Parameters\\PnpInterface");
        static L_ONE: [u16; 2] = wlit::<2>(b"1");
        static L_FIVE: [u16; 2] = wlit::<2>(b"5");
        if AtapiRegCheckParameterValue(core::ptr::null_mut(), L_PNPIF.as_ptr(), L_ONE.as_ptr(), 0) != 0 {
            WinVer_WDM_Model = TRUE as BOOLEAN;
        }
        if AtapiRegCheckParameterValue(core::ptr::null_mut(), L_PNPIF.as_ptr(), L_FIVE.as_ptr(), 0) != 0 {
            WinVer_WDM_Model = TRUE as BOOLEAN;
        }
    }

    static L_SKIPRAIDS: [u16; 10] = wlit::<10>(b"SkipRaids");
    static L_FORCESIMPLEX: [u16; 13] = wlit::<13>(b"ForceSimplex");
    SkipRaids = AtapiRegCheckDevValue(core::ptr::null_mut(), CHAN_NOT_SPECIFIED, DEVNUM_NOT_SPECIFIED, L_SKIPRAIDS.as_ptr(), 1);
    ForceSimplex = AtapiRegCheckDevValue(core::ptr::null_mut(), CHAN_NOT_SPECIFIED, DEVNUM_NOT_SPECIFIED, L_FORCESIMPLEX.as_ptr(), 0);

    // Set PnP-specific API
    if WinVer_Id() > WinVer_NT {
        hwInitializationData.comm.NeedPhysicalAddresses = TRUE as BOOLEAN;
        hwInitializationData.w2k.HwAdapterControl = Some(core::mem::transmute(AtapiAdapterControl as usize));
    }

    if ReEnter == 0 {
        static L_VMT: [u16; 19] = wlit::<19>(b"VirtualMachineType");
        static L_VBOX: [u16; 11] = wlit::<11>(b"VirtualBox");
        g_opt_VirtualMachine = AtapiRegCheckDevValue(core::ptr::null_mut(), CHAN_NOT_SPECIFIED, DEVNUM_NOT_SPECIFIED, L_VMT.as_ptr(), g_opt_VirtualMachine);
        if g_opt_VirtualMachine > VM_MAX_KNOWN {
            g_opt_VirtualMachine = 0;
        }
        if AtapiRegCheckDevValue(core::ptr::null_mut(), CHAN_NOT_SPECIFIED, DEVNUM_NOT_SPECIFIED, L_VBOX.as_ptr(), (g_opt_VirtualMachine == VM_VBOX) as ULONG) != 0 {
            g_opt_VirtualMachine = VM_VBOX;
        }
        if IgnoreIsaCompatiblePci == 0 || IgnoreNativePci == 0 {
            UniataEnumBusMasterController(DriverObject, RegistryPath as PVOID);
        }

        match g_opt_VirtualMachine {
            v if v == VM_VBOX => {
                g_opt_WaitBusyCount = 20000;
                g_opt_WaitBusyDelay = 150;
                g_opt_WaitDrqDelay = 100;
                g_opt_WaitBusyLongCount = 20000;
                g_opt_MaxIsrWait = 200;
                g_opt_AtapiSendDisableIntr = FALSE as BOOLEAN;
                g_opt_AtapiDmaRawRead = FALSE as BOOLEAN;
            }
            v if v == VM_BOCHS => {
                g_opt_AtapiNoDma = TRUE as BOOLEAN;
            }
            _ => {}
        }

        if hasPCI == 0 {
            g_opt_WaitBusyResetCount = 20000;
            g_opt_WaitBusyCount = 20000;
            g_opt_WaitBusyDelay = 150;
            g_opt_WaitDrqDelay = 100;
            g_opt_WaitBusyLongCount = 20000;
            g_opt_MaxIsrWait = 200;
            g_opt_DriveSelectNanoDelay = 400;
        }
        if g_opt_VirtualMachine > VM_NONE {
            g_opt_DriveSelectNanoDelay = 0;
        }
        if CPU_num > 1 && g_opt_VirtualMachine <= VM_NONE {
            // The nIEN-gated ATAPI send is an SMP workaround for real
            // controllers whose INTRQ stays pending across a device-side
            // mask; QEMU/Bochs-family IDE models check nIEN only at raise
            // time and never replay a suppressed IRQ, so this path loses
            // packet-command completions (smp-bringup 2026-07-10: -smp 2
            // lost the boot CD's INQUIRY -> no \SystemRoot -> 0x6B).
            g_opt_AtapiSendDisableIntr = TRUE as BOOLEAN;
        }

        static L_WBC: [u16; 14] = wlit::<14>(b"WaitBusyCount");
        static L_WBD: [u16; 14] = wlit::<14>(b"WaitBusyDelay");
        static L_WDD: [u16; 13] = wlit::<13>(b"WaitDrqDelay");
        static L_WBLC: [u16; 18] = wlit::<18>(b"WaitBusyLongCount");
        static L_WBLD: [u16; 18] = wlit::<18>(b"WaitBusyLongDelay");
        static L_ASDI: [u16; 21] = wlit::<21>(b"AtapiSendDisableIntr");
        static L_ADRR: [u16; 16] = wlit::<16>(b"AtapiDmaRawRead");
        static L_AND: [u16; 11] = wlit::<11>(b"AtapiNoDma");
        static L_MIW: [u16; 11] = wlit::<11>(b"MaxIsrWait");
        static L_DSND: [u16; 21] = wlit::<21>(b"DriveSelectNanoDelay");
        g_opt_WaitBusyCount = AtapiRegCheckDevValue(core::ptr::null_mut(), CHAN_NOT_SPECIFIED, DEVNUM_NOT_SPECIFIED, L_WBC.as_ptr(), g_opt_WaitBusyCount);
        g_opt_WaitBusyDelay = AtapiRegCheckDevValue(core::ptr::null_mut(), CHAN_NOT_SPECIFIED, DEVNUM_NOT_SPECIFIED, L_WBD.as_ptr(), g_opt_WaitBusyDelay);
        g_opt_WaitDrqDelay = AtapiRegCheckDevValue(core::ptr::null_mut(), CHAN_NOT_SPECIFIED, DEVNUM_NOT_SPECIFIED, L_WDD.as_ptr(), g_opt_WaitDrqDelay);
        g_opt_WaitBusyLongCount = AtapiRegCheckDevValue(core::ptr::null_mut(), CHAN_NOT_SPECIFIED, DEVNUM_NOT_SPECIFIED, L_WBLC.as_ptr(), g_opt_WaitBusyLongCount);
        g_opt_WaitBusyLongDelay = AtapiRegCheckDevValue(core::ptr::null_mut(), CHAN_NOT_SPECIFIED, DEVNUM_NOT_SPECIFIED, L_WBLD.as_ptr(), g_opt_WaitBusyLongDelay);
        g_opt_AtapiSendDisableIntr = if AtapiRegCheckDevValue(core::ptr::null_mut(), CHAN_NOT_SPECIFIED, DEVNUM_NOT_SPECIFIED, L_ASDI.as_ptr(), g_opt_AtapiSendDisableIntr as ULONG) != 0 { TRUE as BOOLEAN } else { FALSE as BOOLEAN };
        g_opt_AtapiDmaRawRead = if AtapiRegCheckDevValue(core::ptr::null_mut(), CHAN_NOT_SPECIFIED, DEVNUM_NOT_SPECIFIED, L_ADRR.as_ptr(), g_opt_AtapiDmaRawRead as ULONG) != 0 { TRUE as BOOLEAN } else { FALSE as BOOLEAN };
        g_opt_AtapiNoDma = if AtapiRegCheckDevValue(core::ptr::null_mut(), CHAN_NOT_SPECIFIED, DEVNUM_NOT_SPECIFIED, L_AND.as_ptr(), g_opt_AtapiNoDma as ULONG) != 0 { TRUE as BOOLEAN } else { FALSE as BOOLEAN };
        g_opt_MaxIsrWait = AtapiRegCheckDevValue(core::ptr::null_mut(), CHAN_NOT_SPECIFIED, DEVNUM_NOT_SPECIFIED, L_MIW.as_ptr(), g_opt_MaxIsrWait);
        g_opt_DriveSelectNanoDelay = AtapiRegCheckDevValue(core::ptr::null_mut(), CHAN_NOT_SPECIFIED, DEVNUM_NOT_SPECIFIED, L_DSND.as_ptr(), g_opt_DriveSelectNanoDelay);
    }

    // Look for legacy ISA-bridged PCI IDE controller (onboard)
    {
        let mut k: ULONG = 0;
        while k < BMListLen {
            DbgPrint(b"[UNIATA] BMList[%u]: %04x:%04x master=%u known=%u slot=%08x\n\0".as_ptr().cast(),
                k, (*BMList.add(k as usize)).nVendorId, (*BMList.add(k as usize)).nDeviceId,
                (*BMList.add(k as usize)).MasterDev as ULONG,
                (*BMList.add(k as usize)).Known as ULONG,
                (*BMList.add(k as usize)).slotNumber); // ahci marker
            k += 1;
        }
    }
    i = 0;
    'isa_pci: while i < BMListLen {
        if (*BMList.add(i as usize)).MasterDev == 0 {
            break;
        }
        if IgnoreIsaCompatiblePci != 0 {
            break;
        }
        if ReEnter != 0 {
            if (*BMList.add(i as usize)).ChanInitOk & 0x03 != 0 {
                statusToReturn = STATUS_SUCCESS as ULONG;
                newStatus = STATUS_SUCCESS as ULONG;
                let _ = newStatus;
            }
            i += 1;
            continue 'isa_pci;
        }

        if (*GlobalConfig).AtDiskPrimaryAddressClaimed != 0 {
            PrimaryClaimed = TRUE as BOOLEAN;
        }
        if (*GlobalConfig).AtDiskSecondaryAddressClaimed != 0 {
            SecondaryClaimed = TRUE as BOOLEAN;
        }
        pref_alt = 0;

        if WinVer_WDM_Model == 0 && PrimaryClaimed == 0 && SecondaryClaimed == 0 && g_Dump == 0
            && ((*BMList.add(i as usize)).ChanInitOk & 0x80 == 0)
        {
            newStatus = UniataClaimLegacyPCIIDE(i) as ULONG;
            if (newStatus as NTSTATUS == STATUS_CONFLICTING_ADDRESSES) && (WinVer_Id() <= WinVer_NT) {
                newStatus = STATUS_SUCCESS as ULONG;
                pref_alt = 1;
            }
            if newStatus != STATUS_SUCCESS as ULONG {
                break;
            }
        }

        if g_opt_Verbose != 0 {
            static MSG: [u8; 47] = *b"Init standard Dual-channel PCI ATA controller:\0";
            _PrintNtConsole(MSG.as_ptr() as PCCH);
        }

        alt = 0;
        while alt < (if WinVer_WDM_Model != 0 { 1 } else { 2 }) {
            c = 0;
            while c < 2 {
                static L_IGNOREISAPCI: [u16; 23] = wlit::<23>(b"IgnoreIsaCompatiblePci");
                if AtapiRegCheckDevValue(core::ptr::null_mut(), c, DEVNUM_NOT_SPECIFIED, L_IGNOREISAPCI.as_ptr(), 0) != 0 {
                    break;
                }
                if c == 0 {
                    if PrimaryClaimed != 0 {
                        c += 1;
                        continue;
                    }
                } else if c == 1 {
                    if SecondaryClaimed != 0 {
                        c += 1;
                        continue;
                    }
                }

                if WinVer_Id() < WinVer_2k {
                    if c == 0 {
                        (*GlobalConfig).AtDiskPrimaryAddressClaimed = FALSE as BOOLEAN;
                    } else if c == 1 {
                        (*GlobalConfig).AtDiskSecondaryAddressClaimed = FALSE as BOOLEAN;
                    }
                }
                if WinVer_WDM_Model == 0 {
                    hwInitializationData.comm.HwFindAdapter = Some(core::mem::transmute(UniataFindBusMasterController as usize));
                } else {
                    hwInitializationData.comm.HwFindAdapter = if c == 0 {
                        Some(core::mem::transmute(UniataFindCompatBusMasterController1 as usize))
                    } else {
                        Some(core::mem::transmute(UniataFindCompatBusMasterController2 as usize))
                    };
                }
                hwInitializationData.comm.NumberOfAccessRanges = 6;
                hwInitializationData.comm.AdapterInterfaceType = _INTERFACE_TYPE_Isa as INTERFACE_TYPE;

                if WinVer_WDM_Model == 0 {
                    (*BMList.add(i as usize)).channel = c as UCHAR as CHAR;
                }

                newStatus = ScsiPortInitialize(
                    DriverObject,
                    RegistryPath as PVOID,
                    &mut hwInitializationData.comm,
                    (i | (if (alt ^ pref_alt) != 0 { 0x80000000 } else { 0 })) as ULONG_PTR as PVOID,
                );
                if newStatus < statusToReturn {
                    statusToReturn = newStatus;
                }
                if newStatus == STATUS_SUCCESS as ULONG {
                    if WinVer_Id() < WinVer_2k {
                        (*BMList.add(i as usize)).ChanInitOk |= (0x01 << c) as UCHAR;
                    }
                }
                c += 1;
            }
            if (*BMList.add(i as usize)).ChanInitOk & 0x03 != 0 {
                break;
            }
            alt += 1;
        }
        if g_opt_Verbose != 0 {
            if (*BMList.add(i as usize)).ChanInitOk & 0x03 != 0 {
                static OK: [u8; 6] = *b"  OK\n\0";
                _PrintNtConsole(OK.as_ptr() as PCCH);
            } else {
                static FAILED: [u8; 10] = *b"  failed\n\0";
                _PrintNtConsole(FAILED.as_ptr() as PCCH);
            }
        }
        i += 1;
    }

    // Look for PCI IDE controller
    while i < BMListLen {
        if IgnoreNativePci != 0 {
            break;
        }
        hwInitializationData.comm.HwFindAdapter = Some(core::mem::transmute(UniataFindBusMasterController as usize));
        hwInitializationData.comm.NumberOfAccessRanges = 6;
        hwInitializationData.comm.AdapterInterfaceType = _INTERFACE_TYPE_PCIBus as INTERFACE_TYPE;

        hwInitializationData.comm.VendorId = (*BMList.add(i as usize)).VendorId as PVOID;
        hwInitializationData.comm.VendorIdLength = (*BMList.add(i as usize)).VendorIdLength as USHORT;
        hwInitializationData.comm.DeviceId = (*BMList.add(i as usize)).DeviceId as PVOID;
        hwInitializationData.comm.DeviceIdLength = (*BMList.add(i as usize)).DeviceIdLength as USHORT;

        (*BMList.add(i as usize)).channel = 0;

        newStatus = ScsiPortInitialize(
            DriverObject,
            RegistryPath as PVOID,
            &mut hwInitializationData.comm,
            i as ULONG_PTR as PVOID,
        );
        DbgPrint(b"[UNIATA] native init BMList[%u]=%04x:%04x st=%08x altinit=%u\n\0".as_ptr().cast(),
            i, (*BMList.add(i as usize)).nVendorId, (*BMList.add(i as usize)).nDeviceId,
            newStatus, (*BMList.add(i as usize)).NeedAltInit as ULONG); // ahci marker
        if newStatus == STATUS_DEVICE_DOES_NOT_EXIST as ULONG && (*BMList.add(i as usize)).NeedAltInit != 0 {
            hwInitializationData.comm.AdapterInterfaceType = _INTERFACE_TYPE_Isa as INTERFACE_TYPE;
            newStatus = ScsiPortInitialize(
                DriverObject,
                RegistryPath as PVOID,
                &mut hwInitializationData.comm,
                (i | 0x80000000) as ULONG_PTR as PVOID,
            );
            DbgPrint(b"[UNIATA] native init alt BMList[%u] st=%08x\n\0".as_ptr().cast(),
                i, newStatus); // ahci marker
        }
        if newStatus < statusToReturn {
            statusToReturn = newStatus;
        }
        if g_opt_Verbose != 0 {
            if newStatus == STATUS_SUCCESS as ULONG {
                static OK: [u8; 6] = *b"  OK\n\0";
                _PrintNtConsole(OK.as_ptr() as PCCH);
            } else {
                static FAILED: [u8; 10] = *b"  failed\n\0";
                _PrintNtConsole(FAILED.as_ptr() as PCCH);
            }
        }
        i += 1;
    }

    hwInitializationData.comm.VendorId = core::ptr::null_mut();
    hwInitializationData.comm.VendorIdLength = 0;
    hwInitializationData.comm.DeviceId = core::ptr::null_mut();
    hwInitializationData.comm.DeviceIdLength = 0;

    if BMListLen == 0 {
        hwInitializationData.comm.SrbExtensionSize = core::mem::size_of::<ATA_REQ>() as ULONG;
    }

    hwInitializationData.comm.NumberOfAccessRanges = 2;
    hwInitializationData.comm.HwFindAdapter = Some(core::mem::transmute(AtapiFindIsaController as usize));

    static L_IGNOREISA: [u16; 10] = wlit::<10>(b"IgnoreIsa");
    static L_IGNOREMCA: [u16; 10] = wlit::<10>(b"IgnoreMca");
    if AtapiRegCheckDevValue(core::ptr::null_mut(), CHAN_NOT_SPECIFIED, DEVNUM_NOT_SPECIFIED, L_IGNOREISA.as_ptr(), 0) == 0 {
        hwInitializationData.comm.AdapterInterfaceType = _INTERFACE_TYPE_Isa as INTERFACE_TYPE;
        adapterCount = 0;
        newStatus = ScsiPortInitialize(
            DriverObject, RegistryPath as PVOID, &mut hwInitializationData.comm,
            &mut adapterCount as *mut ULONG as PVOID,
        );
        if newStatus < statusToReturn {
            statusToReturn = newStatus;
        }
    }
    if AtapiRegCheckDevValue(core::ptr::null_mut(), CHAN_NOT_SPECIFIED, DEVNUM_NOT_SPECIFIED, L_IGNOREMCA.as_ptr(), 0) == 0 {
        hwInitializationData.comm.AdapterInterfaceType = _INTERFACE_TYPE_MicroChannel as INTERFACE_TYPE;
        adapterCount = 0;
        newStatus = ScsiPortInitialize(
            DriverObject, RegistryPath as PVOID, &mut hwInitializationData.comm,
            &mut adapterCount as *mut ULONG as PVOID,
        );
        if newStatus < statusToReturn {
            statusToReturn = newStatus;
        }
    }
    InDriverEntry = FALSE as BOOLEAN;

    statusToReturn as NTSTATUS
}

// USE_REACTOS_DDK: KeNumberProcessors is an exported PCCHAR-ish counter. It is a
// kernel DATA export — needs raw-dylib so the reference goes through the import thunk
// (__imp_) instead of expecting a local definition (i386 cross-DLL data import).
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static KeNumberProcessors: CCHAR;
}
