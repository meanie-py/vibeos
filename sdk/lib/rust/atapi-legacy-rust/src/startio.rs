//! atapi.sys interrupt DPC + StartIo dispatch (the bulk of the miniport).
//!
//! Faithful hand-translation of `AtapiCallBack`, `AtapiInterrupt`, and
//! `AtapiStartIo` from drivers/storage/ide/atapi/atapi.c. Control flow,
//! arithmetic, side effects and quirks are preserved. The C `goto CompleteRequest`
//! in `AtapiInterrupt` is rendered by extracting the `CompleteRequest:` label body
//! into the helper `atapi_complete_request` (every `goto` site, and the
//! fall-through from the `interruptReason == 0x3` branch, invokes it identically
//! with the live `srb`/`status`/`wordCount`); the C fall-through `switch` in
//! `AtapiStartIo` is rendered with labeled blocks that emulate fall-through and
//! inner-switch `break`.

use ntkrnl_sys::ddk::{
    UCHAR, ULONG, BOOLEAN, PVOID, PUCHAR, PUSHORT,
    _KINTERRUPT_MODE_LevelSensitive,
};

use crate::types::{
    PHW_DEVICE_EXTENSION, PSCSI_REQUEST_BLOCK, PSENSE_DATA, SENSE_DATA,
    PATAPI_REGISTERS_1, PATAPI_REGISTERS_2, PIDE_REGISTERS_1, PIDE_REGISTERS_2,
    PMECHANICAL_STATUS_INFORMATION_HEADER, PSRB_IO_CONTROL, IDENTIFY_DATA2,
    IDE_STATUS_DSC, IDE_STATUS_BUSY, IDE_STATUS_ERROR, IDE_STATUS_DRQ,
    SRB_STATUS_ERROR, SRB_STATUS_DATA_OVERRUN, SRB_STATUS_SUCCESS, SRB_STATUS_PENDING,
    SRB_STATUS_BUSY, SRB_STATUS_ABORT_FAILED, SRB_STATUS_SELECTION_TIMEOUT,
    SRB_STATUS_INVALID_REQUEST,
    SRB_FLAGS_DATA_IN, SRB_FLAGS_DATA_OUT,
    SRB_FUNCTION_EXECUTE_SCSI, SRB_FUNCTION_ABORT_COMMAND, SRB_FUNCTION_RESET_BUS,
    SRB_FUNCTION_IO_CONTROL,
    DFLAGS_ATAPI_DEVICE, DFLAGS_DEVICE_PRESENT, DFLAGS_TAPE_DEVICE,
    SCSIOP_REQUEST_SENSE, SCSIOP_MECHANISM_STATUS, SCSI_SENSE_ILLEGAL_REQUEST,
    ATAPI_MODE_SENSE, SP_INTERNAL_ADAPTER_ERROR, IDENTIFY_DATA,
    IOCTL_SCSI_MINIPORT_SMART_VERSION, IOCTL_SCSI_MINIPORT_IDENTIFY,
    IOCTL_SCSI_MINIPORT_READ_SMART_ATTRIBS, IOCTL_SCSI_MINIPORT_READ_SMART_THRESHOLDS,
    IOCTL_SCSI_MINIPORT_ENABLE_SMART, IOCTL_SCSI_MINIPORT_DISABLE_SMART,
    IOCTL_SCSI_MINIPORT_RETURN_STATUS, IOCTL_SCSI_MINIPORT_ENABLE_DISABLE_AUTOSAVE,
    IOCTL_SCSI_MINIPORT_SAVE_ATTRIBUTE_VALUES, IOCTL_SCSI_MINIPORT_EXECUTE_OFFLINE_DIAGS,
    _SCSI_NOTIFICATION_TYPE_RequestComplete, _SCSI_NOTIFICATION_TYPE_NextRequest,
    _SCSI_NOTIFICATION_TYPE_RequestTimerCall,
};

use crate::inlines::{
    GetStatus, GetBaseStatus, WriteBuffer, ReadBuffer, WriteBuffer2, ReadBuffer2,
    WaitOnBusy, ScsiPortReadPortUchar, ScsiPortStallExecution, ScsiPortMoveMemory,
    ScsiPortLogError, ScsiPortNotification,
};

use crate::helpers::{AtapiStringCmp, BuildMechanismStatusSrb, BuildRequestSenseSrb};

use crate::commands::{MapError, Atapi2Scsi, AtapiSendCommand, IdeSendCommand};
use crate::io::IdeSendSmartCommand;
use crate::identify::{AtapiResetController, AtapiHwInitializeChanger};

// ---------------------------------------------------------------------------
// SCSI_NOTIFICATION_TYPE constants used by ScsiPortNotification(). bindgen
// exports these as `_SCSI_NOTIFICATION_TYPE_*`; alias to the C enumerator names.
const RequestComplete: ULONG = _SCSI_NOTIFICATION_TYPE_RequestComplete as ULONG;
const NextRequest: ULONG = _SCSI_NOTIFICATION_TYPE_NextRequest as ULONG;
const RequestTimerCall: ULONG = _SCSI_NOTIFICATION_TYPE_RequestTimerCall as ULONG;

// KINTERRUPT_MODE::LevelSensitive (== 0). bindgen exports as `_KINTERRUPT_MODE_*`.
const LevelSensitive: ULONG = _KINTERRUPT_MODE_LevelSensitive as ULONG;

// ---------------------------------------------------------------------------
// SMART / IDENTIFY IOCTL constants and structs (from sdk/include/psdk/ntdddisk.h).
// These are NOT present in the bindgen-generated atapi types nor ntkrnl-sys, so
// they are defined here from the header (values are read straight from the C
// header, NOT invented). Reported as TODO-MISSING for relocation into types.
// ---------------------------------------------------------------------------
// ntdddisk.h:671  #define ID_CMD            0xEC
const ID_CMD: UCHAR = 0xEC;
// ntdddisk.h:672  #define SMART_CMD         0xB0
const SMART_CMD: UCHAR = 0xB0;
// ntdddisk.h:701  #define SMART_IDE_ERROR   1
const SMART_IDE_ERROR: UCHAR = 1;
// ntdddisk.h:748  #define RETURN_SMART_STATUS 0xDA
const RETURN_SMART_STATUS: UCHAR = 0xDA;
// ntdddisk.h:727  #define IDENTIFY_BUFFER_SIZE 512
const IDENTIFY_BUFFER_SIZE: ULONG = 512;
// ntdddisk.h:656-658  fCapabilities bits
const CAP_ATA_ID_CMD: ULONG = 1;
const CAP_ATAPI_ID_CMD: ULONG = 2;
const CAP_SMART_CMD: ULONG = 4;

// atapi.h:280  #define IDENTIFY_DATA_SIZE sizeof(IDENTIFY_DATA)
const IDENTIFY_DATA_SIZE: ULONG = core::mem::size_of::<IDENTIFY_DATA>() as ULONG;

// ntdddisk.h:660  _GETVERSIONINPARAMS
#[repr(C)]
struct GETVERSIONINPARAMS {
    bVersion: UCHAR,
    bRevision: UCHAR,
    bReserved: UCHAR,
    bIDEDeviceMap: UCHAR,
    fCapabilities: ULONG,
    dwReserved: [ULONG; 4usize],
}
type PGETVERSIONINPARAMS = *mut GETVERSIONINPARAMS;

// ntdddisk.h:677  _IDEREGS
#[repr(C)]
#[derive(Clone, Copy)]
struct IDEREGS {
    bFeaturesReg: UCHAR,
    bSectorCountReg: UCHAR,
    bSectorNumberReg: UCHAR,
    bCylLowReg: UCHAR,
    bCylHighReg: UCHAR,
    bDriveHeadReg: UCHAR,
    bCommandReg: UCHAR,
    bReserved: UCHAR,
}

// ntdddisk.h:690  _SENDCMDINPARAMS (pshpack1)
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct SENDCMDINPARAMS {
    cBufferSize: ULONG,
    irDriveRegs: IDEREGS,
    bDriveNumber: UCHAR,
    bReserved: [UCHAR; 3usize],
    dwReserved: [ULONG; 4usize],
    bBuffer: [UCHAR; 1usize],
}
type PSENDCMDINPARAMS = *mut SENDCMDINPARAMS;

// ntdddisk.h:720  _DRIVERSTATUS
#[repr(C)]
struct DRIVERSTATUS {
    bDriverError: UCHAR,
    bIDEError: UCHAR,
    bReserved: [UCHAR; 2usize],
    dwReserved: [ULONG; 2usize],
}

// ntdddisk.h:734  _SENDCMDOUTPARAMS (pshpack1)
#[repr(C, packed)]
struct SENDCMDOUTPARAMS {
    cBufferSize: ULONG,
    DriverStatus: DRIVERSTATUS,
    bBuffer: [UCHAR; 1usize],
}
type PSENDCMDOUTPARAMS = *mut SENDCMDOUTPARAMS;

// ---------------------------------------------------------------------------

// atapi.c:3411
pub unsafe extern "system" fn AtapiCallBack(HwDeviceExtension: PVOID) {
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let srb: PSCSI_REQUEST_BLOCK = (*deviceExtension).CurrentSrb;
    let baseIoAddress2: PATAPI_REGISTERS_2;
    let statusByte: UCHAR;

    //
    // If the last command was DSC restrictive, see if it's set. If so, the device is
    // ready for a new request. Otherwise, reset the timer and come back to here later.
    //

    if !srb.is_null() && (*deviceExtension).ExpectingInterrupt == 0 {
        // DBG-only IS_RDP DebugPrint dropped.

        baseIoAddress2 = (*deviceExtension).BaseIoAddress2[((*srb).TargetId >> 1) as usize]
            as PATAPI_REGISTERS_2;
        if (*deviceExtension).RDP != 0 {
            statusByte = GetStatus(baseIoAddress2 as PIDE_REGISTERS_2);
            if (statusByte & IDE_STATUS_DSC as UCHAR) != 0 {

                ScsiPortNotification(RequestComplete, deviceExtension as PVOID, srb);

                //
                // Clear current SRB.
                //

                (*deviceExtension).CurrentSrb = core::ptr::null_mut();
                (*deviceExtension).RDP = 0;

                //
                // Ask for next request.
                //

                ScsiPortNotification(NextRequest, deviceExtension as PVOID,
                                     core::ptr::null_mut::<core::ffi::c_void>());

                return;

            } else {

                ScsiPortNotification(RequestTimerCall, HwDeviceExtension,
                                     AtapiCallBack as *const () as PVOID, 1000u32);
                return;
            }
        }
    }

    AtapiInterrupt(HwDeviceExtension);
}

// atapi.c:3484
pub unsafe extern "system" fn AtapiInterrupt(HwDeviceExtension: PVOID) -> BOOLEAN {
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let srb: PSCSI_REQUEST_BLOCK = (*deviceExtension).CurrentSrb;
    let baseIoAddress1: PATAPI_REGISTERS_1;
    let baseIoAddress2: PATAPI_REGISTERS_2;
    let mut wordCount: ULONG = 0;
    let mut wordsThisInterrupt: ULONG = 256;
    let status: ULONG;
    let mut i: ULONG;
    let mut statusByte: UCHAR;
    let interruptReason: UCHAR;
    let mut atapiDev: BOOLEAN = 0;

    if !srb.is_null() {
        baseIoAddress1 = (*deviceExtension).BaseIoAddress1[((*srb).TargetId >> 1) as usize]
            as PATAPI_REGISTERS_1;
        baseIoAddress2 = (*deviceExtension).BaseIoAddress2[((*srb).TargetId >> 1) as usize]
            as PATAPI_REGISTERS_2;
    } else {
        //
        // We can only support one ATAPI IDE master on Carolina, so find
        // the base address that is non NULL and clear its interrupt before
        // returning.
        //
        // (_PPC_ branch dropped; non-PPC path below.)

        if (*deviceExtension).InterruptMode == LevelSensitive {
            if !(*deviceExtension).BaseIoAddress1[0].is_null() {
                let bia1 = (*deviceExtension).BaseIoAddress1[0] as PATAPI_REGISTERS_1;
                let _statusByte = GetBaseStatus(bia1 as PIDE_REGISTERS_1); // nolint: let-shadow (dummy status read to clear the interrupt)
            }
            if !(*deviceExtension).BaseIoAddress1[1].is_null() {
                let bia1 = (*deviceExtension).BaseIoAddress1[1] as PATAPI_REGISTERS_1;
                let _statusByte = GetBaseStatus(bia1 as PIDE_REGISTERS_1); // nolint: let-shadow (dummy status read to clear the interrupt)
            }
        }
        return 0; // FALSE
    }

    if (*deviceExtension).ExpectingInterrupt == 0 {
        return 0; // FALSE
    }

    //
    // Clear interrupt by reading status.
    //

    statusByte = GetBaseStatus(baseIoAddress1 as PIDE_REGISTERS_1);

    if (statusByte & IDE_STATUS_BUSY as UCHAR) != 0 {
        if (*deviceExtension).DriverMustPoll != 0 {

            //
            // Crashdump is polling and we got caught with busy asserted.
            // Just go away, and we will be polled again shortly.
            //

            return 1; // TRUE
        }

        //
        // Ensure BUSY is non-asserted.
        //

        i = 0;
        while i < 10 {
            statusByte = GetBaseStatus(baseIoAddress1 as PIDE_REGISTERS_1);
            if (statusByte & IDE_STATUS_BUSY as UCHAR) == 0 {
                break;
            }
            ScsiPortStallExecution(5000);
            i += 1;
        }

        if i == 10 {

            ScsiPortNotification(RequestTimerCall, HwDeviceExtension,
                                 AtapiCallBack as *const () as PVOID, 500u32);
            return 1; // TRUE
        }
    }

    //
    // Check for error conditions.
    //

    if (statusByte & IDE_STATUS_ERROR as UCHAR) != 0 {

        if (*srb).Cdb[0] != SCSIOP_REQUEST_SENSE as UCHAR {
            // Fail this request.
            status = SRB_STATUS_ERROR as ULONG;
            // goto CompleteRequest;
            return atapi_complete_request(
                deviceExtension, srb, baseIoAddress1, baseIoAddress2,
                status, wordCount, HwDeviceExtension);
        }
    }

    //
    // check reason for this interrupt.
    //

    if ((*deviceExtension).DeviceFlags[(*srb).TargetId as usize] as ULONG)
        & DFLAGS_ATAPI_DEVICE != 0 {

        interruptReason = ScsiPortReadPortUchar(&raw mut (*baseIoAddress1).InterruptReason) & 0x3;
        atapiDev = 1;
        wordsThisInterrupt = 256;

    } else {

        if (statusByte & IDE_STATUS_DRQ as UCHAR) != 0 {

            if (*deviceExtension).MaximumBlockXfer[(*srb).TargetId as usize] != 0 {
                wordsThisInterrupt =
                    256 * (*deviceExtension).MaximumBlockXfer[(*srb).TargetId as usize] as ULONG;
            }

            if (*srb).SrbFlags & SRB_FLAGS_DATA_IN != 0 {
                interruptReason = 0x2;
            } else if (*srb).SrbFlags & SRB_FLAGS_DATA_OUT != 0 {
                interruptReason = 0x0;
            } else {
                status = SRB_STATUS_ERROR as ULONG;
                // goto CompleteRequest;
                return atapi_complete_request(
                    deviceExtension, srb, baseIoAddress1, baseIoAddress2,
                    status, wordCount, HwDeviceExtension);
            }

        } else if (statusByte & IDE_STATUS_BUSY as UCHAR) != 0 {

            return 0; // FALSE

        } else {

            if (*deviceExtension).WordsLeft != 0 {

                //
                // Funky behaviour seen with PCI IDE (not all, just one).
                // The ISR hits with DRQ low, but comes up later.
                //

                let mut k: ULONG = 0;
                while k < 5000 {
                    statusByte = GetStatus(baseIoAddress2 as PIDE_REGISTERS_2);
                    if (statusByte & IDE_STATUS_DRQ as UCHAR) == 0 {
                        ScsiPortStallExecution(100);
                    } else {
                        break;
                    }
                    k += 1;
                }

                if k == 5000 {

                    // reset the controller.
                    AtapiResetController(HwDeviceExtension, (*srb).PathId as ULONG);
                    return 1; // TRUE
                } else {
                    interruptReason =
                        if (*srb).SrbFlags & SRB_FLAGS_DATA_IN != 0 { 0x2 } else { 0x0 };
                }

            } else {

                //
                // Command complete - verify, write, or the SMART enable/disable.
                // Also get_media_status
                //
                interruptReason = 0x3;
            }
        }
    }

    if interruptReason == 0x1 && (statusByte & IDE_STATUS_DRQ as UCHAR) != 0 {

        //
        // Write the packet. Send CDB to device.
        //

        WriteBuffer(baseIoAddress1 as PIDE_REGISTERS_1,
                    (*srb).Cdb.as_mut_ptr() as PUSHORT, 6);

        return 1; // TRUE

    } else if interruptReason == 0x0 && (statusByte & IDE_STATUS_DRQ as UCHAR) != 0 {

        //
        // Write the data.
        //

        if ((*deviceExtension).DeviceFlags[(*srb).TargetId as usize] as ULONG)
            & DFLAGS_ATAPI_DEVICE != 0 {

            // Pick up bytes to transfer and convert to words.
            wordCount = ScsiPortReadPortUchar(&raw mut (*baseIoAddress1).ByteCountLow) as ULONG;
            wordCount |=
                (ScsiPortReadPortUchar(&raw mut (*baseIoAddress1).ByteCountHigh) as ULONG) << 8;

            // Convert bytes to words.
            wordCount >>= 1;

            // Verify this makes sense.
            if wordCount > (*deviceExtension).WordsLeft {
                wordCount = (*deviceExtension).WordsLeft;
            }

        } else {

            // IDE path. Check if words left is at least 256.
            if (*deviceExtension).WordsLeft < wordsThisInterrupt {
                wordCount = (*deviceExtension).WordsLeft;
            } else {
                wordCount = wordsThisInterrupt;
            }
        }

        //
        // Ensure that this is a write command.
        //

        if (*srb).SrbFlags & SRB_FLAGS_DATA_OUT != 0 {

            WaitOnBusy(baseIoAddress2 as PIDE_REGISTERS_2);

            if atapiDev != 0 || (*deviceExtension).DWordIO == 0 {

                WriteBuffer(baseIoAddress1 as PIDE_REGISTERS_1,
                            (*deviceExtension).DataBuffer, wordCount);
            } else {
                // PIDE_REGISTERS_3 address3 = (PIDE_REGISTERS_3)baseIoAddress1;
                WriteBuffer2(baseIoAddress1 as PIDE_REGISTERS_1,
                             (*deviceExtension).DataBuffer as *mut ULONG, wordCount / 2);
            }
        } else {

            // Fail this request.
            status = SRB_STATUS_ERROR as ULONG;
            // goto CompleteRequest;
            return atapi_complete_request(
                deviceExtension, srb, baseIoAddress1, baseIoAddress2,
                status, wordCount, HwDeviceExtension);
        }

        //
        // Advance data buffer pointer and bytes left.
        //

        (*deviceExtension).DataBuffer = (*deviceExtension).DataBuffer.add(wordCount as usize);
        (*deviceExtension).WordsLeft -= wordCount;

        return 1; // TRUE

    } else if interruptReason == 0x2 && (statusByte & IDE_STATUS_DRQ as UCHAR) != 0 {

        if ((*deviceExtension).DeviceFlags[(*srb).TargetId as usize] as ULONG)
            & DFLAGS_ATAPI_DEVICE != 0 {

            // Pick up bytes to transfer and convert to words.
            wordCount = ScsiPortReadPortUchar(&raw mut (*baseIoAddress1).ByteCountLow) as ULONG;
            wordCount |=
                (ScsiPortReadPortUchar(&raw mut (*baseIoAddress1).ByteCountHigh) as ULONG) << 8;

            // Convert bytes to words.
            wordCount >>= 1;

            // Verify this makes sense.
            if wordCount > (*deviceExtension).WordsLeft {
                wordCount = (*deviceExtension).WordsLeft;
            }

        } else {

            // Check if words left is at least 256.
            if (*deviceExtension).WordsLeft < wordsThisInterrupt {
                wordCount = (*deviceExtension).WordsLeft;
            } else {
                wordCount = wordsThisInterrupt;
            }
        }

        //
        // Ensure that this is a read command.
        //

        if (*srb).SrbFlags & SRB_FLAGS_DATA_IN != 0 {

            WaitOnBusy(baseIoAddress2 as PIDE_REGISTERS_2);

            if atapiDev != 0 || (*deviceExtension).DWordIO == 0 {
                ReadBuffer(baseIoAddress1 as PIDE_REGISTERS_1,
                           (*deviceExtension).DataBuffer, wordCount);
            } else {
                // PIDE_REGISTERS_3 address3 = (PIDE_REGISTERS_3)baseIoAddress1;
                ReadBuffer2(baseIoAddress1 as PIDE_REGISTERS_1,
                            (*deviceExtension).DataBuffer as *mut ULONG, wordCount / 2);
            }
        } else {

            // Fail this request.
            status = SRB_STATUS_ERROR as ULONG;
            // goto CompleteRequest;
            return atapi_complete_request(
                deviceExtension, srb, baseIoAddress1, baseIoAddress2,
                status, wordCount, HwDeviceExtension);
        }

        //
        // Translate ATAPI data back to SCSI data if needed
        //

        if (*srb).Cdb[0] == ATAPI_MODE_SENSE as UCHAR
            && ((*deviceExtension).DeviceFlags[(*srb).TargetId as usize] as ULONG)
                & DFLAGS_ATAPI_DEVICE != 0 {

            // convert and adjust the wordCount
            wordCount -= Atapi2Scsi(srb, (*deviceExtension).DataBuffer as *mut i8,
                                    wordCount << 1);
        }

        //
        // Advance data buffer pointer and bytes left.
        //

        (*deviceExtension).DataBuffer = (*deviceExtension).DataBuffer.add(wordCount as usize);
        (*deviceExtension).WordsLeft -= wordCount;

        //
        // Check for read command complete.
        //

        if (*deviceExtension).WordsLeft == 0 {

            if ((*deviceExtension).DeviceFlags[(*srb).TargetId as usize] as ULONG)
                & DFLAGS_ATAPI_DEVICE != 0 {

                //
                // Work around to make many atapi devices return correct sector size
                // of 2048. Also certain devices will have sector count == 0x00, check
                // for that also.
                //

                if (*srb).Cdb[0] == 0x25
                    && (((*deviceExtension).IdentifyData[(*srb).TargetId as usize]
                            .GeneralConfiguration >> 8) & 0x1f) == 0x05 {

                    (*deviceExtension).DataBuffer =
                        (*deviceExtension).DataBuffer.sub(wordCount as usize);
                    if *(*deviceExtension).DataBuffer.add(0) == 0x00 {

                        *((*deviceExtension).DataBuffer.add(0) as *mut ULONG) = 0xFFFFFF7F;
                    }

                    *((*deviceExtension).DataBuffer.add(2) as *mut ULONG) = 0x00080000;
                    (*deviceExtension).DataBuffer =
                        (*deviceExtension).DataBuffer.add(wordCount as usize);
                }
            } else {

                //
                // Completion for IDE drives.
                //

                if (*deviceExtension).WordsLeft != 0 {
                    status = SRB_STATUS_DATA_OVERRUN as ULONG;
                } else {
                    status = SRB_STATUS_SUCCESS as ULONG;
                }

                // goto CompleteRequest;
                return atapi_complete_request(
                    deviceExtension, srb, baseIoAddress1, baseIoAddress2,
                    status, wordCount, HwDeviceExtension);
            }
        }

        return 1; // TRUE

    } else if interruptReason == 0x3 && (statusByte & IDE_STATUS_DRQ as UCHAR) == 0 {

        //
        // Command complete.
        //

        if (*deviceExtension).WordsLeft != 0 {
            status = SRB_STATUS_DATA_OVERRUN as ULONG;
        } else {
            status = SRB_STATUS_SUCCESS as ULONG;
        }

        // fall through to CompleteRequest
        return atapi_complete_request(
            deviceExtension, srb, baseIoAddress1, baseIoAddress2,
            status, wordCount, HwDeviceExtension);

    } else {

        //
        // Unexpected int.
        //

        return 0; // FALSE
    }
}

/// atapi.c CompleteRequest label body (reached by `goto CompleteRequest` from
/// the interrupt-reason dispatch, and by fall-through from the `interruptReason
/// == 0x3` branch). Extracted to a helper so every `goto CompleteRequest` site
/// can invoke it identically; `srb`, `status`, `wordCount` are passed in by value
/// exactly as they stood at the goto. The original code returns TRUE at the end
/// of this label (the function's final `return TRUE;`), so this helper returns
/// BOOLEAN and callers `return` its result.
unsafe fn atapi_complete_request(
    deviceExtension: PHW_DEVICE_EXTENSION,
    mut srb: PSCSI_REQUEST_BLOCK,
    baseIoAddress1: PATAPI_REGISTERS_1,
    baseIoAddress2: PATAPI_REGISTERS_2,
    mut status: ULONG,
    wordCount: ULONG,
    HwDeviceExtension: PVOID,
) -> BOOLEAN {
    let mut i: ULONG;
    let mut statusByte: UCHAR = 0;

    //
    // Check and see if we are processing our secret (mechanism status/request sense) srb
    //
    if !(*deviceExtension).OriginalSrb.is_null() {

        let mut srbStatus: ULONG;

        if (*srb).Cdb[0] == SCSIOP_MECHANISM_STATUS as UCHAR {

            if status == SRB_STATUS_SUCCESS as ULONG {
                // Bingo!!
                AtapiHwInitializeChanger(HwDeviceExtension, (*srb).TargetId as ULONG,
                    (*srb).DataBuffer as PMECHANICAL_STATUS_INFORMATION_HEADER);

                // Get ready to issue the original srb
                srb = (*deviceExtension).OriginalSrb;
                (*deviceExtension).CurrentSrb = srb;
                (*deviceExtension).OriginalSrb = core::ptr::null_mut();

            } else {
                // failed!  Get the sense key and maybe try again
                srb = BuildRequestSenseSrb(
                    HwDeviceExtension,
                    (*(*deviceExtension).OriginalSrb).PathId as ULONG,
                    (*(*deviceExtension).OriginalSrb).TargetId as ULONG);
                (*deviceExtension).CurrentSrb = srb;
            }

            srbStatus = AtapiSendCommand(HwDeviceExtension, (*deviceExtension).CurrentSrb);
            if srbStatus == SRB_STATUS_PENDING as ULONG {
                return 1; // TRUE
            }

        } else { // srb->Cdb[0] == SCSIOP_REQUEST_SENSE)

            let senseData: PSENSE_DATA = (*srb).DataBuffer as PSENSE_DATA;

            if status == SRB_STATUS_DATA_OVERRUN as ULONG {
                // Check to see if we at least get minimum number of bytes
                if ((*srb).DataTransferLength - (*deviceExtension).WordsLeft)
                    > (core::mem::offset_of!(SENSE_DATA, AdditionalSenseLength) as ULONG
                        + core::mem::size_of_val(&(*senseData).AdditionalSenseLength) as ULONG) {
                    status = SRB_STATUS_SUCCESS as ULONG;
                }
            }

            if status == SRB_STATUS_SUCCESS as ULONG {
                if (*senseData).SenseKey() != SCSI_SENSE_ILLEGAL_REQUEST as UCHAR
                    && (*deviceExtension).MechStatusRetryCount != 0 {

                    // The sense key doesn't say the last request is illegal, so try again
                    (*deviceExtension).MechStatusRetryCount -= 1;
                    srb = BuildMechanismStatusSrb(
                        HwDeviceExtension,
                        (*(*deviceExtension).OriginalSrb).PathId as ULONG,
                        (*(*deviceExtension).OriginalSrb).TargetId as ULONG);
                    (*deviceExtension).CurrentSrb = srb;
                } else {

                    // last request was illegal.  No point trying again
                    AtapiHwInitializeChanger(HwDeviceExtension, (*srb).TargetId as ULONG,
                        core::ptr::null_mut::<core::ffi::c_void>()
                            as PMECHANICAL_STATUS_INFORMATION_HEADER);

                    // Get ready to issue the original srb
                    srb = (*deviceExtension).OriginalSrb;
                    (*deviceExtension).CurrentSrb = srb;
                    (*deviceExtension).OriginalSrb = core::ptr::null_mut();
                }

                srbStatus = AtapiSendCommand(HwDeviceExtension, (*deviceExtension).CurrentSrb);
                if srbStatus == SRB_STATUS_PENDING as ULONG {
                    return 1; // TRUE
                }
            }
        }

        // If we get here, it means AtapiSendCommand() has failed
        // Can't recover.  Pretend the original srb has failed and complete it.

        if !(*deviceExtension).OriginalSrb.is_null() {
            AtapiHwInitializeChanger(HwDeviceExtension, (*srb).TargetId as ULONG,
                core::ptr::null_mut::<core::ffi::c_void>()
                    as PMECHANICAL_STATUS_INFORMATION_HEADER);
            srb = (*deviceExtension).OriginalSrb;
            (*deviceExtension).CurrentSrb = srb;
            (*deviceExtension).OriginalSrb = core::ptr::null_mut();
        }

        // fake an error and read no data
        status = SRB_STATUS_ERROR as ULONG;
        (*srb).ScsiStatus = 0;
        (*deviceExtension).DataBuffer = (*srb).DataBuffer as PUSHORT;
        (*deviceExtension).WordsLeft = (*srb).DataTransferLength;
        (*deviceExtension).RDP = 0;

    } else if status == SRB_STATUS_ERROR as ULONG {

        //
        // Map error to specific SRB status and handle request sense.
        //

        status = MapError(deviceExtension as PVOID, srb);

        (*deviceExtension).RDP = 0;

    } else {

        //
        // Wait for busy to drop.
        //

        i = 0;
        while i < 30 {
            statusByte = GetStatus(baseIoAddress2 as PIDE_REGISTERS_2);
            if (statusByte & IDE_STATUS_BUSY as UCHAR) == 0 {
                break;
            }
            ScsiPortStallExecution(500);
            i += 1;
        }

        if i == 30 {

            // reset the controller.
            AtapiResetController(HwDeviceExtension, (*srb).PathId as ULONG);
            return 1; // TRUE
        }

        //
        // Check to see if DRQ is still up.
        //

        // NB: C uses the residual `statusByte` left by the busy-drop loop above
        // (no extra port read here). Preserve that exactly.
        if (statusByte & IDE_STATUS_DRQ as UCHAR) != 0 {

            i = 0;
            while i < 500 {
                statusByte = GetStatus(baseIoAddress2 as PIDE_REGISTERS_2);
                if (statusByte & IDE_STATUS_DRQ as UCHAR) == 0 {
                    break;
                }
                ScsiPortStallExecution(100);
                i += 1;
            }

            if i == 500 {

                // reset the controller.
                AtapiResetController(HwDeviceExtension, (*srb).PathId as ULONG);
                return 1; // TRUE
            }
        }
    }

    //
    // Clear interrupt expecting flag.
    //

    (*deviceExtension).ExpectingInterrupt = 0;

    //
    // Sanity check that there is a current request.
    //

    if !srb.is_null() {

        //
        // Set status in SRB.
        //

        (*srb).SrbStatus = status as UCHAR;

        //
        // Check for underflow.
        //

        if (*deviceExtension).WordsLeft != 0 {

            //
            // Subtract out residual words and update if filemark hit,
            // setmark hit , end of data, end of media...
            //

            if ((*deviceExtension).DeviceFlags[(*srb).TargetId as usize] as ULONG)
                & DFLAGS_TAPE_DEVICE == 0 {
                if status == SRB_STATUS_DATA_OVERRUN as ULONG {
                    (*srb).DataTransferLength -= (*deviceExtension).WordsLeft;
                } else {
                    (*srb).DataTransferLength = 0;
                }
            } else {
                (*srb).DataTransferLength -= (*deviceExtension).WordsLeft;
            }
        }

        if (*srb).Function != SRB_FUNCTION_IO_CONTROL as UCHAR {

            //
            // Indicate command complete.
            //

            if (*deviceExtension).RDP == 0 {
                ScsiPortNotification(RequestComplete, deviceExtension as PVOID, srb);
            }
        } else {

            let cmdOutParameters: PSENDCMDOUTPARAMS =
                ((*srb).DataBuffer as PUCHAR)
                    .add(core::mem::size_of::<crate::types::SRB_IO_CONTROL>())
                    as PSENDCMDOUTPARAMS;
            let mut error: UCHAR = 0;

            if status != SRB_STATUS_SUCCESS as ULONG {
                error = ScsiPortReadPortUchar((baseIoAddress1 as PUCHAR).add(1));
            }

            //
            // Build the SMART status block depending upon the completion status.
            //

            (*cmdOutParameters).cBufferSize = wordCount;
            (*cmdOutParameters).DriverStatus.bDriverError =
                if error != 0 { SMART_IDE_ERROR } else { 0 };
            (*cmdOutParameters).DriverStatus.bIDEError = error;

            //
            // If the sub-command is return smart status, jam the value from
            // cylinder low and high, into the data buffer.
            //

            if (*deviceExtension).SmartCommand == RETURN_SMART_STATUS {
                let bBuffer = (*cmdOutParameters).bBuffer.as_mut_ptr();
                *bBuffer.add(0) = RETURN_SMART_STATUS;
                *bBuffer.add(1) = ScsiPortReadPortUchar(&raw mut (*baseIoAddress1).InterruptReason);
                *bBuffer.add(2) = ScsiPortReadPortUchar(&raw mut (*baseIoAddress1).Unused1);
                *bBuffer.add(3) = ScsiPortReadPortUchar(&raw mut (*baseIoAddress1).ByteCountLow);
                *bBuffer.add(4) = ScsiPortReadPortUchar(&raw mut (*baseIoAddress1).ByteCountHigh);
                *bBuffer.add(5) = ScsiPortReadPortUchar(&raw mut (*baseIoAddress1).DriveSelect);
                *bBuffer.add(6) = SMART_CMD;
                (*cmdOutParameters).cBufferSize = 8;
            }

            //
            // Indicate command complete.
            //

            ScsiPortNotification(RequestComplete, deviceExtension as PVOID, srb);
        }

    }
    // else: DebugPrint "No SRB!" dropped.

    //
    // Indicate ready for next request.
    //

    if (*deviceExtension).RDP == 0 {

        // Clear current SRB.
        (*deviceExtension).CurrentSrb = core::ptr::null_mut();

        ScsiPortNotification(NextRequest, deviceExtension as PVOID,
                             core::ptr::null_mut::<core::ffi::c_void>());
    } else {

        ScsiPortNotification(RequestTimerCall, HwDeviceExtension,
                             AtapiCallBack as *const () as PVOID, 2000u32);
    }

    1 // TRUE
}

// atapi.c:5811
pub unsafe extern "system" fn AtapiStartIo(
    HwDeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
) -> BOOLEAN {
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let mut status: ULONG;

    //
    // Determine which function.
    //
    // The C `switch (Srb->Function)` falls through from ABORT_COMMAND into
    // RESET_BUS. We render the dispatch as a labeled block `'dispatch` so each
    // C `break` becomes `break 'dispatch`, an early `return FALSE` stays a
    // return, and the ABORT_COMMAND -> RESET_BUS fall-through is expressed by NOT
    // breaking out of the abort arm (it runs into the reset code).
    'dispatch: {
        let func = (*Srb).Function as ULONG;

        if func == SRB_FUNCTION_EXECUTE_SCSI as ULONG {

            //
            // Sanity check. Only one request can be outstanding on a controller.
            //

            if !(*deviceExtension).CurrentSrb.is_null() {

                (*Srb).SrbStatus = SRB_STATUS_BUSY as UCHAR;
                ScsiPortNotification(RequestComplete, deviceExtension as PVOID, Srb);
                return 0; // FALSE
            }

            //
            // Indicate that a request is active on the controller.
            //

            (*deviceExtension).CurrentSrb = Srb;

            //
            // Send command to device.
            //

            if ((*deviceExtension).DeviceFlags[(*Srb).TargetId as usize] as ULONG)
                & DFLAGS_ATAPI_DEVICE != 0 {
                status = AtapiSendCommand(HwDeviceExtension, Srb);
            } else if ((*deviceExtension).DeviceFlags[(*Srb).TargetId as usize] as ULONG)
                & DFLAGS_DEVICE_PRESENT != 0 {
                status = IdeSendCommand(HwDeviceExtension, Srb);
            } else {
                status = SRB_STATUS_SELECTION_TIMEOUT as ULONG;
            }

            break 'dispatch;

        } else if func == SRB_FUNCTION_ABORT_COMMAND as ULONG
            || func == SRB_FUNCTION_RESET_BUS as ULONG {

            // SRB_FUNCTION_ABORT_COMMAND:
            if func == SRB_FUNCTION_ABORT_COMMAND as ULONG {

                //
                // Verify that SRB to abort is still outstanding.
                //

                if (*deviceExtension).CurrentSrb.is_null() {

                    // Complete abort SRB.
                    status = SRB_STATUS_ABORT_FAILED as ULONG;
                    break 'dispatch;
                }

                //
                // Abort function indicates that a request timed out. Call reset
                // routine. Card will only be reset if status indicates something
                // is wrong. Fall through to reset code.
                //
            }

            // SRB_FUNCTION_RESET_BUS: (abort falls through to here)

            //
            // Reset Atapi and SCSI bus.
            //

            if AtapiResetController(deviceExtension as PVOID, (*Srb).PathId as ULONG) == 0 {

                //
                // Log reset failure.
                //

                ScsiPortLogError(
                    HwDeviceExtension,
                    core::ptr::null_mut(),
                    0, 0, 0,
                    SP_INTERNAL_ADAPTER_ERROR,
                    5 << 8,
                );

                status = SRB_STATUS_ERROR as ULONG;

            } else {
                status = SRB_STATUS_SUCCESS as ULONG;
            }

            break 'dispatch;

        } else if func == SRB_FUNCTION_IO_CONTROL as ULONG {

            if !(*deviceExtension).CurrentSrb.is_null() {

                (*Srb).SrbStatus = SRB_STATUS_BUSY as UCHAR;
                ScsiPortNotification(RequestComplete, deviceExtension as PVOID, Srb);
                return 0; // FALSE
            }

            //
            // Indicate that a request is active on the controller.
            //

            (*deviceExtension).CurrentSrb = Srb;

            // strlen("SCSIDISK") == 8
            let signature: &[u8; 9] = b"SCSIDISK\0";
            if AtapiStringCmp(
                (*((*Srb).DataBuffer as PSRB_IO_CONTROL)).Signature.as_mut_ptr() as *mut i8,
                signature.as_ptr() as *mut i8,
                8,
            ) != 0 {

                status = SRB_STATUS_INVALID_REQUEST as ULONG;
                break 'dispatch;
            }

            // The inner IOCTL switch. C `break` from a case breaks the INNER
            // switch (then falls to the outer `break;` after it). We render the
            // inner switch as labeled block `'ioctl` so case `break` -> `break
            // 'ioctl`, after which we `break 'dispatch` (matching the outer
            // `break;` that follows the inner switch).
            let controlCode = (*((*Srb).DataBuffer as PSRB_IO_CONTROL)).ControlCode;

            'ioctl: {
                if controlCode == IOCTL_SCSI_MINIPORT_SMART_VERSION {

                    let versionParameters: PGETVERSIONINPARAMS =
                        ((*Srb).DataBuffer as PUCHAR)
                            .add(core::mem::size_of::<crate::types::SRB_IO_CONTROL>())
                            as PGETVERSIONINPARAMS;
                    let mut deviceNumber: UCHAR;

                    //
                    // Version and revision per SMART 1.03
                    //

                    (*versionParameters).bVersion = 1;
                    (*versionParameters).bRevision = 1;
                    (*versionParameters).bReserved = 0;

                    //
                    // Indicate that support for IDE IDENTIFY, ATAPI IDENTIFY and SMART commands.
                    //

                    (*versionParameters).fCapabilities =
                        CAP_ATA_ID_CMD | CAP_ATAPI_ID_CMD | CAP_SMART_CMD;

                    //
                    // This is done because of how the IOCTL_SCSI_MINIPORT
                    // determines 'targetid's'. Disk.sys places the real target id value
                    // in the DeviceMap field.
                    //


                    if ((*deviceExtension).DeviceFlags[(*Srb).TargetId as usize] as ULONG)
                            & DFLAGS_DEVICE_PRESENT == 0
                        || ((*deviceExtension).DeviceFlags[(*Srb).TargetId as usize] as ULONG)
                            & DFLAGS_ATAPI_DEVICE != 0 {

                        status = SRB_STATUS_SELECTION_TIMEOUT as ULONG;
                        break 'ioctl;
                    }

                    //
                    // NOTE: This will only set the bit corresponding to this
                    // drive's target id. Bit mask:  Sec Pri / S M S M / 3 2 1 0
                    //

                    if (*deviceExtension).NumberChannels == 1 {
                        if (*deviceExtension).PrimaryAddress != 0 {
                            deviceNumber = (1u32 << (*Srb).TargetId) as UCHAR;
                        } else {
                            deviceNumber = (4u32 << (*Srb).TargetId) as UCHAR;
                        }
                    } else {
                        deviceNumber = (1u32 << (*Srb).TargetId) as UCHAR;
                    }

                    (*versionParameters).bIDEDeviceMap = deviceNumber;

                    status = SRB_STATUS_SUCCESS as ULONG;
                    break 'ioctl;

                } else if controlCode == IOCTL_SCSI_MINIPORT_IDENTIFY {

                    let cmdOutParameters: PSENDCMDOUTPARAMS =
                        ((*Srb).DataBuffer as PUCHAR)
                            .add(core::mem::size_of::<crate::types::SRB_IO_CONTROL>())
                            as PSENDCMDOUTPARAMS;
                    let cmdInParameters: SENDCMDINPARAMS =
                        *(((*Srb).DataBuffer as PUCHAR)
                            .add(core::mem::size_of::<crate::types::SRB_IO_CONTROL>())
                            as PSENDCMDINPARAMS);
                    let mut i: ULONG;
                    let targetId: UCHAR;

                    if cmdInParameters.irDriveRegs.bCommandReg == ID_CMD {

                        // Extract the target.
                        targetId = cmdInParameters.bDriveNumber;

                        if ((*deviceExtension).DeviceFlags[(*Srb).TargetId as usize] as ULONG)
                                & DFLAGS_DEVICE_PRESENT == 0
                            || ((*deviceExtension).DeviceFlags[(*Srb).TargetId as usize] as ULONG)
                                & DFLAGS_ATAPI_DEVICE != 0 {

                            status = SRB_STATUS_SELECTION_TIMEOUT as ULONG;
                            break 'ioctl;
                        }

                        // Zero the output buffer
                        i = 0;
                        while i < (core::mem::size_of::<SENDCMDOUTPARAMS>() as ULONG
                                    + IDENTIFY_BUFFER_SIZE - 1) {
                            *((cmdOutParameters as PUCHAR).add(i as usize)) = 0;
                            i += 1;
                        }

                        // Build status block.
                        (*cmdOutParameters).cBufferSize = IDENTIFY_BUFFER_SIZE;
                        (*cmdOutParameters).DriverStatus.bDriverError = 0;
                        (*cmdOutParameters).DriverStatus.bIDEError = 0;

                        // Extract the identify data from the device extension.
                        ScsiPortMoveMemory(
                            (*cmdOutParameters).bBuffer.as_mut_ptr() as PVOID,
                            &mut (*deviceExtension).IdentifyData[targetId as usize]
                                as *mut IDENTIFY_DATA2 as PVOID,
                            IDENTIFY_DATA_SIZE,
                        );

                        status = SRB_STATUS_SUCCESS as ULONG;

                    } else {
                        status = SRB_STATUS_INVALID_REQUEST as ULONG;
                    }
                    break 'ioctl;

                } else if controlCode == IOCTL_SCSI_MINIPORT_READ_SMART_ATTRIBS
                    || controlCode == IOCTL_SCSI_MINIPORT_READ_SMART_THRESHOLDS
                    || controlCode == IOCTL_SCSI_MINIPORT_ENABLE_SMART
                    || controlCode == IOCTL_SCSI_MINIPORT_DISABLE_SMART
                    || controlCode == IOCTL_SCSI_MINIPORT_RETURN_STATUS
                    || controlCode == IOCTL_SCSI_MINIPORT_ENABLE_DISABLE_AUTOSAVE
                    || controlCode == IOCTL_SCSI_MINIPORT_SAVE_ATTRIBUTE_VALUES
                    || controlCode == IOCTL_SCSI_MINIPORT_EXECUTE_OFFLINE_DIAGS {

                    status = IdeSendSmartCommand(HwDeviceExtension, Srb);
                    break 'ioctl;

                } else {
                    status = SRB_STATUS_INVALID_REQUEST as ULONG;
                    break 'ioctl;
                }
            } // 'ioctl

            break 'dispatch;

        } else {

            //
            // Indicate unsupported command.
            //

            status = SRB_STATUS_INVALID_REQUEST as ULONG;

            break 'dispatch;
        }
    } // 'dispatch

    //
    // Check if command complete.
    //

    if status != SRB_STATUS_PENDING as ULONG {

        //
        // Clear current SRB.
        //

        (*deviceExtension).CurrentSrb = core::ptr::null_mut();

        //
        // Set status in SRB.
        //

        (*Srb).SrbStatus = status as UCHAR;

        //
        // Indicate command complete.
        //

        ScsiPortNotification(RequestComplete, deviceExtension as PVOID, Srb);

        //
        // Indicate ready for next request.
        //

        ScsiPortNotification(NextRequest, deviceExtension as PVOID,
                             core::ptr::null_mut::<core::ffi::c_void>());
    }

    1 // TRUE
}
