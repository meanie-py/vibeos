// commands.rs — faithful C→Rust transliteration of drivers/storage/ide/atapi/atapi.c
// Functions: MapError, Scsi2Atapi, Atapi2Scsi, AtapiSendCommand, IdeSendCommand.
// DebugPrint/DPRINT calls are DEBUG-ONLY and dropped.

use ntkrnl_sys::ddk::{UCHAR, USHORT, ULONG, PVOID, PUCHAR, PUSHORT};

use crate::types::{
    PHW_DEVICE_EXTENSION, PIDE_REGISTERS_1, PIDE_REGISTERS_2,
    PATAPI_REGISTERS_1, PATAPI_REGISTERS_2,
    PSCSI_REQUEST_BLOCK, PCDB,
    PSENSE_DATA,
    PMODE_PARAMETER_HEADER, PMODE_PARAMETER_HEADER_10, MODE_PARAMETER_HEADER, MODE_PARAMETER_HEADER_10,
    PMODE_SENSE_10, PMODE_SELECT_10,
    PIDENTIFY_DATA2,
    PINQUIRYDATA,
    PREAD_CAPACITY_DATA,
    // constants
    SRB_STATUS_ERROR, SRB_STATUS_SUCCESS, SRB_STATUS_ABORTED, SRB_STATUS_AUTOSENSE_VALID,
    SRB_STATUS_INVALID_REQUEST, SRB_STATUS_PENDING, SRB_STATUS_BUSY, SRB_STATUS_BUS_RESET,
    SRB_STATUS_SELECTION_TIMEOUT,
    SCSISTAT_CHECK_CONDITION,
    SCSI_SENSE_NO_SENSE, SCSI_SENSE_RECOVERED_ERROR, SCSI_SENSE_NOT_READY, SCSI_SENSE_MEDIUM_ERROR,
    SCSI_SENSE_HARDWARE_ERROR, SCSI_SENSE_ILLEGAL_REQUEST, SCSI_SENSE_UNIT_ATTENTION,
    SCSI_SENSE_DATA_PROTECT, SCSI_SENSE_BLANK_CHECK, SCSI_SENSE_ABORTED_COMMAND,
    SCSI_ADSENSE_MEDIUM_CHANGED,
    DFLAGS_ATAPI_DEVICE, DFLAGS_MEDIA_STATUS_ENABLED, DFLAGS_DEVICE_PRESENT,
    DFLAGS_CHANGER_INITED, DFLAGS_SANYO_ATAPI_CHANGER, DFLAGS_ATAPI_CHANGER,
    DFLAGS_TAPE_DEVICE, DFLAGS_INT_DRQ, DFLAGS_REMOVABLE_DRIVE,
    IDE_ERROR_MEDIA_CHANGE_REQ, IDE_ERROR_COMMAND_ABORTED, IDE_ERROR_END_OF_MEDIA,
    IDE_ERROR_ILLEGAL_LENGTH, IDE_ERROR_BAD_BLOCK, IDE_ERROR_ID_NOT_FOUND,
    IDE_ERROR_MEDIA_CHANGE, IDE_ERROR_DATA_ERROR,
    IDE_STATUS_ERROR, IDE_STATUS_BUSY, IDE_STATUS_DSC, IDE_STATUS_DRQ,
    IDE_COMMAND_SET_MULTIPLE, IDE_COMMAND_ATAPI_IDENTIFY, IDE_COMMAND_ATAPI_PACKET,
    IDE_COMMAND_GET_MEDIA_STATUS, IDE_COMMAND_MEDIA_EJECT,
    MAX_ERRORS, SP_BAD_FW_WARNING,
    MAXIMUM_CDB_SIZE,
    ATAPI_MODE_SENSE, ATAPI_MODE_SELECT, ATAPI_FORMAT_UNIT,
    SCSIOP_MODE_SENSE, SCSIOP_MODE_SELECT, SCSIOP_FORMAT_UNIT, SCSIOP_REQUEST_SENSE,
    SCSIOP_TEST_UNIT_READY, SCSIOP_INQUIRY, SCSIOP_READ_CAPACITY, SCSIOP_VERIFY,
    SCSIOP_READ, SCSIOP_WRITE, SCSIOP_START_STOP_UNIT,
    DIRECT_ACCESS_DEVICE, MODE_DSP_WRITE_PROTECT,
    SCSI_ADSENSE_NO_MEDIA_IN_DEVICE,
    _SCSI_NOTIFICATION_TYPE_ResetDetected,
};

use crate::inlines::{
    ScsiPortReadPortUchar, ScsiPortReadPortUshort, ScsiPortWritePortUchar,
    ScsiPortStallExecution, ScsiPortMoveMemory, ScsiPortLogError, ScsiPortNotification,
    GetStatus, GetBaseStatus, WaitOnBusy, WaitOnBaseBusy, WaitForDrq,
    WriteBuffer, AtapiSoftReset, IS_RDP,
};

use crate::helpers::{AtapiZeroMemory, BuildMechanismStatusSrb};

// Cross-module callees (defined in sibling modules; resolved at crate root):
use crate::{
    IssueIdentify, AtapiHwInitializeChanger,
    IdeVerify, IdeReadWrite, IdeBuildSenseBuffer,
};

pub unsafe extern "system" fn MapError(
    HwDeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
) -> ULONG {
    // atapi.c:927
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let baseIoAddress1: PIDE_REGISTERS_1 =
        (*deviceExtension).BaseIoAddress1[((*Srb).TargetId >> 1) as usize];
    // `i` (atapi.c:931) is only used inside the MAX_ERRORS reprogram loop, which
    // lives in map_error_finish() in this transliteration.
    let errorByte: UCHAR;
    let mut srbStatus: UCHAR = SRB_STATUS_ERROR as UCHAR;
    let scsiStatus: UCHAR;

    // atapi.c:940 — Read the error register.
    errorByte = ScsiPortReadPortUchar((baseIoAddress1 as PUCHAR).add(1));

    if ((*deviceExtension).DeviceFlags[(*Srb).TargetId as usize] as ULONG)
        & (DFLAGS_ATAPI_DEVICE as ULONG)
        != 0
    {
        // atapi.c:947
        match (errorByte >> 4) as ULONG {
            x if x == SCSI_SENSE_NO_SENSE as ULONG => {
                scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
                srbStatus = SRB_STATUS_ERROR as UCHAR;
            }
            x if x == SCSI_SENSE_RECOVERED_ERROR as ULONG => {
                scsiStatus = 0;
                srbStatus = SRB_STATUS_SUCCESS as UCHAR;
            }
            x if x == SCSI_SENSE_NOT_READY as ULONG => {
                scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
                srbStatus = SRB_STATUS_ERROR as UCHAR;
            }
            x if x == SCSI_SENSE_MEDIUM_ERROR as ULONG => {
                scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
                srbStatus = SRB_STATUS_ERROR as UCHAR;
            }
            x if x == SCSI_SENSE_HARDWARE_ERROR as ULONG => {
                scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
                srbStatus = SRB_STATUS_ERROR as UCHAR;
            }
            x if x == SCSI_SENSE_ILLEGAL_REQUEST as ULONG => {
                scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
                srbStatus = SRB_STATUS_ERROR as UCHAR;
            }
            x if x == SCSI_SENSE_UNIT_ATTENTION as ULONG => {
                scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
                srbStatus = SRB_STATUS_ERROR as UCHAR;
            }
            x if x == SCSI_SENSE_DATA_PROTECT as ULONG => {
                scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
                srbStatus = SRB_STATUS_ERROR as UCHAR;
            }
            x if x == SCSI_SENSE_BLANK_CHECK as ULONG => {
                scsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
                srbStatus = SRB_STATUS_ERROR as UCHAR;
            }
            x if x == SCSI_SENSE_ABORTED_COMMAND as ULONG => {
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

        // atapi.c:1044 — Save errorByte, to be used by SCSIOP_REQUEST_SENSE.
        (*deviceExtension).ReturningMediaStatus = errorByte;

        if (errorByte & IDE_ERROR_MEDIA_CHANGE_REQ as UCHAR) != 0 {
            // atapi.c:1046
            let scsiStatus_local = SCSISTAT_CHECK_CONDITION as UCHAR;
            srbStatus = SRB_STATUS_ERROR as UCHAR;
            return map_error_finish(deviceExtension, Srb, baseIoAddress1, srbStatus, scsiStatus_local, errorByte);
        } else if (errorByte & IDE_ERROR_COMMAND_ABORTED as UCHAR) != 0 {
            // atapi.c:1052
            srbStatus = SRB_STATUS_ABORTED as UCHAR;
            let scsiStatus_local = SCSISTAT_CHECK_CONDITION as UCHAR;

            if !(*Srb).SenseInfoBuffer.is_null() {
                let senseBuffer: PSENSE_DATA = (*Srb).SenseInfoBuffer as PSENSE_DATA;

                (*senseBuffer).set_ErrorCode(0x70);
                (*senseBuffer).set_Valid(1);
                (*senseBuffer).AdditionalSenseLength = 0xb;
                (*senseBuffer).set_SenseKey(SCSI_SENSE_ABORTED_COMMAND as UCHAR);
                (*senseBuffer).AdditionalSenseCode = 0;
                (*senseBuffer).AdditionalSenseCodeQualifier = 0;

                srbStatus |= SRB_STATUS_AUTOSENSE_VALID as UCHAR;
            }

            (*deviceExtension).ErrorCount += 1;

            return map_error_finish(deviceExtension, Srb, baseIoAddress1, srbStatus, scsiStatus_local, errorByte);
        } else if (errorByte & IDE_ERROR_END_OF_MEDIA as UCHAR) != 0 {
            // atapi.c:1074
            let scsiStatus_local = SCSISTAT_CHECK_CONDITION as UCHAR;
            srbStatus = SRB_STATUS_ERROR as UCHAR;

            if ((*deviceExtension).DeviceFlags[(*Srb).TargetId as usize] as ULONG)
                & (DFLAGS_MEDIA_STATUS_ENABLED as ULONG)
                == 0
            {
                (*deviceExtension).ErrorCount += 1;
            }

            return map_error_finish(deviceExtension, Srb, baseIoAddress1, srbStatus, scsiStatus_local, errorByte);
        } else if (errorByte & IDE_ERROR_ILLEGAL_LENGTH as UCHAR) != 0 {
            // atapi.c:1084
            srbStatus = SRB_STATUS_INVALID_REQUEST as UCHAR;
            return map_error_finish(deviceExtension, Srb, baseIoAddress1, srbStatus, scsiStatus, errorByte);
        } else if (errorByte & IDE_ERROR_BAD_BLOCK as UCHAR) != 0 {
            // atapi.c:1090
            srbStatus = SRB_STATUS_ERROR as UCHAR;
            let scsiStatus_local = SCSISTAT_CHECK_CONDITION as UCHAR;
            if !(*Srb).SenseInfoBuffer.is_null() {
                let senseBuffer: PSENSE_DATA = (*Srb).SenseInfoBuffer as PSENSE_DATA;

                (*senseBuffer).set_ErrorCode(0x70);
                (*senseBuffer).set_Valid(1);
                (*senseBuffer).AdditionalSenseLength = 0xb;
                (*senseBuffer).set_SenseKey(SCSI_SENSE_MEDIUM_ERROR as UCHAR);
                (*senseBuffer).AdditionalSenseCode = 0;
                (*senseBuffer).AdditionalSenseCodeQualifier = 0;

                srbStatus |= SRB_STATUS_AUTOSENSE_VALID as UCHAR;
            }
            return map_error_finish(deviceExtension, Srb, baseIoAddress1, srbStatus, scsiStatus_local, errorByte);
        } else if (errorByte & IDE_ERROR_ID_NOT_FOUND as UCHAR) != 0 {
            // atapi.c:1110
            srbStatus = SRB_STATUS_ERROR as UCHAR;
            let scsiStatus_local = SCSISTAT_CHECK_CONDITION as UCHAR;

            if !(*Srb).SenseInfoBuffer.is_null() {
                let senseBuffer: PSENSE_DATA = (*Srb).SenseInfoBuffer as PSENSE_DATA;

                (*senseBuffer).set_ErrorCode(0x70);
                (*senseBuffer).set_Valid(1);
                (*senseBuffer).AdditionalSenseLength = 0xb;
                (*senseBuffer).set_SenseKey(SCSI_SENSE_MEDIUM_ERROR as UCHAR);
                (*senseBuffer).AdditionalSenseCode = 0;
                (*senseBuffer).AdditionalSenseCodeQualifier = 0;

                srbStatus |= SRB_STATUS_AUTOSENSE_VALID as UCHAR;
            }

            (*deviceExtension).ErrorCount += 1;
            return map_error_finish(deviceExtension, Srb, baseIoAddress1, srbStatus, scsiStatus_local, errorByte);
        } else if (errorByte & IDE_ERROR_MEDIA_CHANGE as UCHAR) != 0 {
            // atapi.c:1133
            let scsiStatus_local = SCSISTAT_CHECK_CONDITION as UCHAR;
            srbStatus = SRB_STATUS_ERROR as UCHAR;

            if !(*Srb).SenseInfoBuffer.is_null() {
                let senseBuffer: PSENSE_DATA = (*Srb).SenseInfoBuffer as PSENSE_DATA;

                (*senseBuffer).set_ErrorCode(0x70);
                (*senseBuffer).set_Valid(1);
                (*senseBuffer).AdditionalSenseLength = 0xb;
                (*senseBuffer).set_SenseKey(SCSI_SENSE_UNIT_ATTENTION as UCHAR);
                (*senseBuffer).AdditionalSenseCode = SCSI_ADSENSE_MEDIUM_CHANGED as UCHAR;
                (*senseBuffer).AdditionalSenseCodeQualifier = 0;

                srbStatus |= SRB_STATUS_AUTOSENSE_VALID as UCHAR;
            }
            return map_error_finish(deviceExtension, Srb, baseIoAddress1, srbStatus, scsiStatus_local, errorByte);
        } else if (errorByte & IDE_ERROR_DATA_ERROR as UCHAR) != 0 {
            // atapi.c:1154
            let scsiStatus_local = SCSISTAT_CHECK_CONDITION as UCHAR;
            srbStatus = SRB_STATUS_ERROR as UCHAR;

            if ((*deviceExtension).DeviceFlags[(*Srb).TargetId as usize] as ULONG)
                & (DFLAGS_MEDIA_STATUS_ENABLED as ULONG)
                == 0
            {
                (*deviceExtension).ErrorCount += 1;
            }

            if !(*Srb).SenseInfoBuffer.is_null() {
                let senseBuffer: PSENSE_DATA = (*Srb).SenseInfoBuffer as PSENSE_DATA;

                (*senseBuffer).set_ErrorCode(0x70);
                (*senseBuffer).set_Valid(1);
                (*senseBuffer).AdditionalSenseLength = 0xb;
                (*senseBuffer).set_SenseKey(SCSI_SENSE_MEDIUM_ERROR as UCHAR);
                (*senseBuffer).AdditionalSenseCode = 0;
                (*senseBuffer).AdditionalSenseCodeQualifier = 0;

                srbStatus |= SRB_STATUS_AUTOSENSE_VALID as UCHAR;
            }
            return map_error_finish(deviceExtension, Srb, baseIoAddress1, srbStatus, scsiStatus_local, errorByte);
        }

        // None of the IDE error branches matched: still run the MAX_ERRORS check
        // and the common tail (atapi.c:1184).
        return map_error_finish(deviceExtension, Srb, baseIoAddress1, srbStatus, scsiStatus, errorByte);
    }

    // atapi.c:1273 (ATAPI device path tail)
    (*Srb).ScsiStatus = scsiStatus;

    srbStatus as ULONG
}

// Common tail for the IDE (non-ATAPI) path: MAX_ERRORS handling (atapi.c:1184)
// followed by the shared exit (atapi.c:1273).
unsafe fn map_error_finish(
    deviceExtension: PHW_DEVICE_EXTENSION,
    Srb: PSCSI_REQUEST_BLOCK,
    baseIoAddress1: PIDE_REGISTERS_1,
    srbStatus: UCHAR,
    scsiStatus: UCHAR,
    mut errorByte: UCHAR,
) -> ULONG {
    let mut i: ULONG;

    if (*deviceExtension).ErrorCount >= MAX_ERRORS {
        // atapi.c:1184
        (*deviceExtension).DWordIO = 0; // FALSE
        (*deviceExtension).MaximumBlockXfer[(*Srb).TargetId as usize] = 0;

        // atapi.c:1195 — Log the error.
        ScsiPortLogError(
            deviceExtension as PVOID,
            Srb,
            (*Srb).PathId,
            (*Srb).TargetId,
            (*Srb).Lun,
            SP_BAD_FW_WARNING as ULONG,
            4,
        );

        // atapi.c:1206 — Reprogram to not use Multi-sector.
        i = 0;
        while i < 4 {
            let mut statusByte: UCHAR;

            if ((*deviceExtension).DeviceFlags[i as usize] as ULONG)
                & (DFLAGS_DEVICE_PRESENT as ULONG)
                != 0
                && ((*deviceExtension).DeviceFlags[i as usize] as ULONG)
                    & (DFLAGS_ATAPI_DEVICE as ULONG)
                    == 0
            {
                // Select the device.
                ScsiPortWritePortUchar(
                    &mut (*baseIoAddress1).DriveSelect,
                    (((i & 0x1) << 4) | 0xA0) as UCHAR,
                );

                // Setup sector count.
                ScsiPortWritePortUchar(&mut (*baseIoAddress1).BlockCount, 0);

                // Issue the command.
                ScsiPortWritePortUchar(
                    &mut (*baseIoAddress1).Command,
                    IDE_COMMAND_SET_MULTIPLE as UCHAR,
                );

                // Wait for busy to drop.
                statusByte = WaitOnBaseBusy(baseIoAddress1);

                if (statusByte & IDE_STATUS_ERROR as UCHAR) != 0 {
                    // Read the error register.
                    errorByte = ScsiPortReadPortUchar((baseIoAddress1 as PUCHAR).add(1));
                    let _ = errorByte;

                    (*deviceExtension).MaximumBlockXfer[i as usize] = 0;
                }
            }

            i += 1;
        }
    }

    // atapi.c:1273
    (*Srb).ScsiStatus = scsiStatus;

    srbStatus as ULONG
}

pub unsafe extern "system" fn Atapi2Scsi(
    Srb: PSCSI_REQUEST_BLOCK,
    DataBuffer: *mut core::ffi::c_char,
    mut ByteCount: ULONG,
) -> ULONG {
    // atapi.c:3365
    let mut bytesAdjust: ULONG = 0;

    if (*Srb).Cdb[0] == ATAPI_MODE_SENSE as UCHAR {
        // atapi.c:3369
        let header_10: PMODE_PARAMETER_HEADER_10 = DataBuffer as PMODE_PARAMETER_HEADER_10;
        let header: PMODE_PARAMETER_HEADER = DataBuffer as PMODE_PARAMETER_HEADER;

        (*header).ModeDataLength = (*header_10).ModeDataLengthLsb;
        (*header).MediumType = (*header_10).MediumType;

        // ATAPI Mode Parameter Header doesn't have these fields.
        (*header).DeviceSpecificParameter = (*header_10).Reserved[0];
        (*header).BlockDescriptorLength = (*header_10).Reserved[1];

        // atapi.c:3382 — ByteCount is unsigned; underflow wraps (faithful).
        ByteCount = ByteCount.wrapping_sub(core::mem::size_of::<MODE_PARAMETER_HEADER_10>() as ULONG);
        if ByteCount > 0 {
            ScsiPortMoveMemory(
                (DataBuffer as *mut u8).add(core::mem::size_of::<MODE_PARAMETER_HEADER>()) as PVOID,
                (DataBuffer as *mut u8).add(core::mem::size_of::<MODE_PARAMETER_HEADER_10>()) as PVOID,
                ByteCount,
            );
        }

        // atapi.c:3393 — Change ATAPI_MODE_SENSE opcode back to SCSIOP_MODE_SENSE.
        (*Srb).Cdb[0] = SCSIOP_MODE_SENSE as UCHAR;

        bytesAdjust = (core::mem::size_of::<MODE_PARAMETER_HEADER_10>()
            - core::mem::size_of::<MODE_PARAMETER_HEADER>()) as ULONG;
    }

    // atapi.c:3405 — Convert to words.
    bytesAdjust >> 1
}

pub unsafe extern "system" fn Scsi2Atapi(Srb: PSCSI_REQUEST_BLOCK) {
    // atapi.c:4937
    (*Srb).CdbLength = 12;

    match (*Srb).Cdb[0] as ULONG {
        x if x == SCSIOP_MODE_SENSE as ULONG => {
            // atapi.c:4940
            let modeSense10: PMODE_SENSE_10 = (*Srb).Cdb.as_mut_ptr() as PMODE_SENSE_10;
            let cdb: PCDB = (*Srb).Cdb.as_mut_ptr() as PCDB;
            let PageCode: UCHAR = (*cdb).MODE_SENSE.as_ref().PageCode();
            let Length: UCHAR = (*cdb).MODE_SENSE.as_ref().AllocationLength;

            AtapiZeroMemory((*Srb).Cdb.as_mut_ptr(), MAXIMUM_CDB_SIZE);

            (*modeSense10).OperationCode = ATAPI_MODE_SENSE as UCHAR;
            (*modeSense10).set_PageCode(PageCode);
            (*modeSense10).ParameterListLengthMsb = 0;
            (*modeSense10).ParameterListLengthLsb = Length;
        }
        x if x == SCSIOP_MODE_SELECT as ULONG => {
            // atapi.c:4954
            let modeSelect10: PMODE_SELECT_10 = (*Srb).Cdb.as_mut_ptr() as PMODE_SELECT_10;
            let cdb: PCDB = (*Srb).Cdb.as_mut_ptr() as PCDB;
            let Length: UCHAR = (*cdb).MODE_SELECT.as_ref().ParameterListLength;

            // Zero the original cdb
            AtapiZeroMemory((*Srb).Cdb.as_mut_ptr(), MAXIMUM_CDB_SIZE);

            (*modeSelect10).OperationCode = ATAPI_MODE_SELECT as UCHAR;
            (*modeSelect10).set_PFBit(1);
            (*modeSelect10).ParameterListLengthMsb = 0;
            (*modeSelect10).ParameterListLengthLsb = Length;
        }
        x if x == SCSIOP_FORMAT_UNIT as ULONG => {
            // atapi.c:4971
            (*Srb).Cdb[0] = ATAPI_FORMAT_UNIT as UCHAR;
        }
        _ => {}
    }
}

pub unsafe extern "system" fn AtapiSendCommand(
    HwDeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
) -> ULONG {
    // atapi.c:5002
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let baseIoAddress1: PATAPI_REGISTERS_1 =
        (*deviceExtension).BaseIoAddress1[((*Srb).TargetId >> 1) as usize] as PATAPI_REGISTERS_1;
    let baseIoAddress2: PATAPI_REGISTERS_2 =
        (*deviceExtension).BaseIoAddress2[((*Srb).TargetId >> 1) as usize] as PATAPI_REGISTERS_2;
    let mut i: ULONG;
    let flags: ULONG;
    let mut statusByte: UCHAR;
    let mut byteCountLow: UCHAR;
    let mut byteCountHigh: UCHAR;

    // atapi.c:5017 — secret mechanism-status SRB to init changer data.
    if ((*deviceExtension).DeviceFlags[(*Srb).TargetId as usize] as ULONG)
        & (DFLAGS_CHANGER_INITED as ULONG)
        == 0
        && (*deviceExtension).OriginalSrb.is_null()
    {
        let srbStatus: ULONG;

        (*deviceExtension).DeviceFlags[(*Srb).TargetId as usize] |= DFLAGS_CHANGER_INITED as USHORT;

        (*deviceExtension).MechStatusRetryCount = 3;
        (*deviceExtension).CurrentSrb = BuildMechanismStatusSrb(
            HwDeviceExtension,
            (*Srb).PathId as ULONG,
            (*Srb).TargetId as ULONG,
        );
        (*deviceExtension).OriginalSrb = Srb;

        srbStatus = AtapiSendCommand(HwDeviceExtension, (*deviceExtension).CurrentSrb);
        if srbStatus == SRB_STATUS_PENDING as ULONG {
            return srbStatus;
        } else {
            (*deviceExtension).CurrentSrb = (*deviceExtension).OriginalSrb;
            (*deviceExtension).OriginalSrb = core::ptr::null_mut();
            AtapiHwInitializeChanger(
                HwDeviceExtension,
                (*Srb).TargetId as ULONG,
                core::ptr::null_mut(),
            );
            // fall out
        }
    }

    // atapi.c:5058 — Make sure command is to ATAPI device.
    flags = (*deviceExtension).DeviceFlags[(*Srb).TargetId as usize] as ULONG;
    if flags & ((DFLAGS_SANYO_ATAPI_CHANGER as ULONG) | (DFLAGS_ATAPI_CHANGER as ULONG)) != 0 {
        if ((*Srb).Lun as ULONG) > ((*deviceExtension).DiscsPresent[(*Srb).TargetId as usize] - 1) {
            // Indicate no device found at this address.
            return SRB_STATUS_SELECTION_TIMEOUT as ULONG;
        }
    } else if (*Srb).Lun > 0 {
        return SRB_STATUS_SELECTION_TIMEOUT as ULONG;
    }

    if flags & (DFLAGS_ATAPI_DEVICE as ULONG) == 0 {
        return SRB_STATUS_SELECTION_TIMEOUT as ULONG;
    }

    // atapi.c:5080 — Select device 0 or 1.
    ScsiPortWritePortUchar(
        &mut (*baseIoAddress1).DriveSelect,
        ((((*Srb).TargetId & 0x1) << 4) | 0xA0) as UCHAR,
    );

    // atapi.c:5087 — Verify controller ready.
    statusByte = GetStatus(baseIoAddress2 as PIDE_REGISTERS_2);

    if (statusByte & IDE_STATUS_BUSY as UCHAR) != 0 {
        // atapi.c:5093
        return SRB_STATUS_BUSY as ULONG;
    }

    if (statusByte & IDE_STATUS_ERROR as UCHAR) != 0 {
        // atapi.c:5101
        if (*Srb).Cdb[0] != SCSIOP_REQUEST_SENSE as UCHAR {
            // Read the error reg. to clear it and fail this request.
            return MapError(deviceExtension as PVOID, Srb);
        }
    }

    // atapi.c:5121 — Restrictive Delayed Process commands (QIC-157).
    if (statusByte & IDE_STATUS_DSC as UCHAR) == 0
        && (flags & (DFLAGS_TAPE_DEVICE as ULONG)) != 0
        && (*deviceExtension).RDP != 0
    {
        ScsiPortStallExecution(1000);
        return SRB_STATUS_BUSY as ULONG;
    }

    if IS_RDP((*Srb).Cdb[0]) {
        // atapi.c:5128
        (*deviceExtension).RDP = 1; // TRUE
    } else {
        (*deviceExtension).RDP = 0; // FALSE
    }

    if (statusByte & IDE_STATUS_DRQ as UCHAR) != 0 {
        // atapi.c:5141 — drain stray DRQ data.
        i = 0;
        while i < 0x10000 {
            statusByte = GetStatus(baseIoAddress2 as PIDE_REGISTERS_2);

            if (statusByte & IDE_STATUS_DRQ as UCHAR) != 0 {
                ScsiPortReadPortUshort(&mut (*baseIoAddress1).Data);
            } else {
                break;
            }

            i += 1;
        }

        if i == 0x10000 {
            // atapi.c:5166
            AtapiSoftReset(baseIoAddress1 as PIDE_REGISTERS_1, (*Srb).TargetId as ULONG);

            // Re-initialize Atapi device.
            IssueIdentify(
                HwDeviceExtension,
                ((*Srb).TargetId & 0x1) as ULONG,
                ((*Srb).TargetId >> 1) as ULONG,
                IDE_COMMAND_ATAPI_IDENTIFY as UCHAR,
            );

            // Inform the port driver that the bus has been reset.
            ScsiPortNotification(
                _SCSI_NOTIFICATION_TYPE_ResetDetected as ULONG,
                HwDeviceExtension,
                core::ptr::null_mut::<core::ffi::c_void>(),
            );

            // Clean up device extension fields that AtapiStartIo won't.
            (*deviceExtension).ExpectingInterrupt = 0; // FALSE
            (*deviceExtension).RDP = 0; // FALSE

            return SRB_STATUS_BUS_RESET as ULONG;
        }
    }

    if flags & ((DFLAGS_SANYO_ATAPI_CHANGER as ULONG) | (DFLAGS_ATAPI_CHANGER as ULONG)) != 0 {
        // atapi.c:5204 — strip LUN from cdb byte 1.
        (*Srb).Cdb[1] &= !0xE0u32 as UCHAR;

        if (*Srb).Cdb[0] == SCSIOP_TEST_UNIT_READY as UCHAR
            && (flags & (DFLAGS_SANYO_ATAPI_CHANGER as ULONG)) != 0
        {
            // Torisan changer. TUR's are overloaded to be platter switches.
            (*Srb).Cdb[7] = (*Srb).Lun;
        }
    }

    // atapi.c:5227 — Convert SCSI to ATAPI commands if needed.
    match (*Srb).Cdb[0] as ULONG {
        x if x == SCSIOP_MODE_SENSE as ULONG
            || x == SCSIOP_MODE_SELECT as ULONG
            || x == SCSIOP_FORMAT_UNIT as ULONG =>
        {
            if flags & (DFLAGS_TAPE_DEVICE as ULONG) == 0 {
                Scsi2Atapi(Srb);
            }
        }
        _ => {}
    }

    // atapi.c:5242 — Set data buffer pointer and words left.
    (*deviceExtension).DataBuffer = (*Srb).DataBuffer as PUSHORT;
    (*deviceExtension).WordsLeft = (*Srb).DataTransferLength / 2;

    statusByte = WaitOnBusy(baseIoAddress2 as PIDE_REGISTERS_2);

    // atapi.c:5251 — Write transfer byte count to registers.
    byteCountLow = ((*Srb).DataTransferLength & 0xFF) as UCHAR;
    byteCountHigh = ((*Srb).DataTransferLength >> 8) as UCHAR;

    if (*Srb).DataTransferLength >= 0x10000 {
        byteCountLow = 0xFF;
        byteCountHigh = 0xFF;
    }

    ScsiPortWritePortUchar(&mut (*baseIoAddress1).ByteCountLow, byteCountLow);
    ScsiPortWritePortUchar(&mut (*baseIoAddress1).ByteCountHigh, byteCountHigh);

    ScsiPortWritePortUchar((baseIoAddress1 as PUCHAR).add(1), 0);

    if flags & (DFLAGS_INT_DRQ as ULONG) != 0 {
        // atapi.c:5264 — device interrupts when ready to receive the packet.
        ScsiPortWritePortUchar(
            &mut (*baseIoAddress1).Command,
            IDE_COMMAND_ATAPI_PACKET as UCHAR,
        );

        (*deviceExtension).ExpectingInterrupt = 1; // TRUE

        return SRB_STATUS_PENDING as ULONG;
    } else {
        // atapi.c:5283 — Write ATAPI packet command.
        ScsiPortWritePortUchar(
            &mut (*baseIoAddress1).Command,
            IDE_COMMAND_ATAPI_PACKET as UCHAR,
        );

        // Wait for DRQ.
        statusByte = WaitOnBusy(baseIoAddress2 as PIDE_REGISTERS_2);
        statusByte = WaitForDrq(baseIoAddress2 as PIDE_REGISTERS_2);

        if (statusByte & IDE_STATUS_DRQ as UCHAR) == 0 {
            // atapi.c:5299
            return SRB_STATUS_ERROR as ULONG;
        }
    }

    // atapi.c:5312 — Need to read status register.
    statusByte = GetBaseStatus(baseIoAddress1 as PIDE_REGISTERS_1);
    let _ = statusByte;

    // atapi.c:5318 — Send CDB to device.
    statusByte = WaitOnBusy(baseIoAddress2 as PIDE_REGISTERS_2);
    let _ = statusByte;

    WriteBuffer(
        baseIoAddress1 as PIDE_REGISTERS_1,
        (*Srb).Cdb.as_mut_ptr() as PUSHORT,
        6,
    );

    // atapi.c:5328 — Indicate expecting an interrupt and wait for it.
    (*deviceExtension).ExpectingInterrupt = 1; // TRUE

    SRB_STATUS_PENDING as ULONG
}

pub unsafe extern "system" fn IdeSendCommand(
    HwDeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
) -> ULONG {
    // atapi.c:5358
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let baseIoAddress1: PIDE_REGISTERS_1 =
        (*deviceExtension).BaseIoAddress1[((*Srb).TargetId >> 1) as usize];
    let baseIoAddress2: PIDE_REGISTERS_2 =
        (*deviceExtension).BaseIoAddress2[((*Srb).TargetId >> 1) as usize];
    let cdb: PCDB;

    let mut statusByte: UCHAR;
    let mut errorByte: UCHAR;
    let status: ULONG;
    let mut i: ULONG;
    let modeData: PMODE_PARAMETER_HEADER;

    // atapi.c:5376
    'switch_block: {
        match (*Srb).Cdb[0] as ULONG {
            x if x == SCSIOP_INQUIRY as ULONG => {
                // atapi.c:5377
                if (*Srb).Lun != 0
                    || ((*deviceExtension).DeviceFlags[(*Srb).TargetId as usize] as ULONG)
                        & (DFLAGS_DEVICE_PRESENT as ULONG)
                        == 0
                {
                    // Indicate no device found at this address.
                    status = SRB_STATUS_SELECTION_TIMEOUT as ULONG;
                } else {
                    let inquiryData: PINQUIRYDATA = (*Srb).DataBuffer as PINQUIRYDATA;
                    let identifyData: PIDENTIFY_DATA2 =
                        &mut (*deviceExtension).IdentifyData[(*Srb).TargetId as usize]
                            as PIDENTIFY_DATA2;

                    // Zero INQUIRY data structure.
                    i = 0;
                    while i < (*Srb).DataTransferLength {
                        *((*Srb).DataBuffer as PUCHAR).add(i as usize) = 0;
                        i += 1;
                    }

                    // Standard IDE interface only supports disks.
                    (*inquiryData).set_DeviceType(DIRECT_ACCESS_DEVICE as UCHAR);

                    // Set the removable bit, if applicable.
                    if ((*deviceExtension).DeviceFlags[(*Srb).TargetId as usize] as ULONG)
                        & (DFLAGS_REMOVABLE_DRIVE as ULONG)
                        != 0
                    {
                        (*inquiryData).set_RemovableMedia(1);
                    }

                    // Fill in vendor identification fields.
                    let modelNumber: PUCHAR = (*identifyData).ModelNumber.as_mut_ptr() as PUCHAR;
                    i = 0;
                    while i < 8 {
                        (*inquiryData).VendorId[i as usize] = *modelNumber.add((i + 1) as usize);
                        (*inquiryData).VendorId[(i + 1) as usize] = *modelNumber.add(i as usize);
                        i += 2;
                    }

                    i = 0;
                    while i < 12 {
                        (*inquiryData).ProductId[i as usize] =
                            *modelNumber.add((i + 8 + 1) as usize);
                        (*inquiryData).ProductId[(i + 1) as usize] =
                            *modelNumber.add((i + 8) as usize);
                        i += 2;
                    }

                    // Initialize unused portion of product id.
                    i = 0;
                    while i < 4 {
                        (*inquiryData).ProductId[(12 + i) as usize] = b' ';
                        i += 1;
                    }

                    // Move firmware revision.
                    let firmwareRevision: PUCHAR =
                        (*identifyData).FirmwareRevision.as_mut_ptr() as PUCHAR;
                    i = 0;
                    while i < 4 {
                        (*inquiryData).ProductRevisionLevel[i as usize] =
                            *firmwareRevision.add((i + 1) as usize);
                        (*inquiryData).ProductRevisionLevel[(i + 1) as usize] =
                            *firmwareRevision.add(i as usize);
                        i += 2;
                    }

                    status = SRB_STATUS_SUCCESS as ULONG;
                }
            }
            x if x == SCSIOP_MODE_SENSE as ULONG => {
                // atapi.c:5464
                if ((*deviceExtension).DeviceFlags[(*Srb).TargetId as usize] as ULONG)
                    & (DFLAGS_MEDIA_STATUS_ENABLED as ULONG)
                    != 0
                {
                    ScsiPortWritePortUchar(
                        &mut (*baseIoAddress1).DriveSelect,
                        ((((*Srb).TargetId & 0x1) << 4) | 0xA0) as UCHAR,
                    );
                    ScsiPortWritePortUchar(
                        &mut (*baseIoAddress1).Command,
                        IDE_COMMAND_GET_MEDIA_STATUS as UCHAR,
                    );
                    statusByte = WaitOnBusy(baseIoAddress2);

                    if (statusByte & IDE_STATUS_ERROR as UCHAR) == 0 {
                        // no error, media not protected
                        (*deviceExtension).ExpectingInterrupt = 0; // FALSE
                    } else {
                        // error occured, handle locally, clear interrupt
                        errorByte = ScsiPortReadPortUchar((baseIoAddress1 as PUCHAR).add(1));

                        statusByte = GetBaseStatus(baseIoAddress1);
                        let _ = statusByte;
                        (*deviceExtension).ExpectingInterrupt = 0; // FALSE

                        if (errorByte & IDE_ERROR_DATA_ERROR as UCHAR) != 0 {
                            // media is write-protected
                            modeData = (*Srb).DataBuffer as PMODE_PARAMETER_HEADER;

                            (*Srb).DataTransferLength =
                                core::mem::size_of::<MODE_PARAMETER_HEADER>() as ULONG;
                            (*modeData).DeviceSpecificParameter |= MODE_DSP_WRITE_PROTECT as UCHAR;
                        }
                    }
                    status = SRB_STATUS_SUCCESS as ULONG;
                } else {
                    status = SRB_STATUS_INVALID_REQUEST as ULONG;
                }
            }
            x if x == SCSIOP_TEST_UNIT_READY as ULONG => {
                // atapi.c:5516
                if ((*deviceExtension).DeviceFlags[(*Srb).TargetId as usize] as ULONG)
                    & (DFLAGS_MEDIA_STATUS_ENABLED as ULONG)
                    != 0
                {
                    // Select device 0 or 1.
                    ScsiPortWritePortUchar(
                        &mut (*baseIoAddress1).DriveSelect,
                        ((((*Srb).TargetId & 0x1) << 4) | 0xA0) as UCHAR,
                    );
                    ScsiPortWritePortUchar(
                        &mut (*baseIoAddress1).Command,
                        IDE_COMMAND_GET_MEDIA_STATUS as UCHAR,
                    );

                    statusByte = WaitOnBusy(baseIoAddress2);

                    if (statusByte & IDE_STATUS_ERROR as UCHAR) == 0 {
                        (*deviceExtension).ExpectingInterrupt = 0; // FALSE
                        status = SRB_STATUS_SUCCESS as ULONG;
                    } else {
                        errorByte = ScsiPortReadPortUchar((baseIoAddress1 as PUCHAR).add(1));
                        if errorByte == IDE_ERROR_DATA_ERROR as UCHAR {
                            // write-protect bit is sticky; ignore this error
                            statusByte = GetBaseStatus(baseIoAddress1);
                            let _ = statusByte;
                            (*deviceExtension).ExpectingInterrupt = 0; // FALSE
                            status = SRB_STATUS_SUCCESS as ULONG;
                        } else {
                            // Request sense buffer to be built
                            (*deviceExtension).ExpectingInterrupt = 1; // TRUE
                            status = SRB_STATUS_PENDING as ULONG;
                        }
                    }
                } else {
                    status = SRB_STATUS_SUCCESS as ULONG;
                }
            }
            x if x == SCSIOP_READ_CAPACITY as ULONG => {
                // atapi.c:5566 — Claim 512 byte blocks (big-endian).
                (*((*Srb).DataBuffer as PREAD_CAPACITY_DATA)).BytesPerBlock = 0x20000;

                // Calculate last sector.
                i = ((*deviceExtension).IdentifyData[(*Srb).TargetId as usize].NumberOfHeads as ULONG
                    * (*deviceExtension).IdentifyData[(*Srb).TargetId as usize].NumberOfCylinders
                        as ULONG
                    * (*deviceExtension).IdentifyData[(*Srb).TargetId as usize].SectorsPerTrack
                        as ULONG)
                    .wrapping_sub(1);

                let ip: PUCHAR = &mut i as *mut ULONG as PUCHAR;
                (*((*Srb).DataBuffer as PREAD_CAPACITY_DATA)).LogicalBlockAddress =
                    ((*ip.add(0) as ULONG) << 24)
                        | ((*ip.add(1) as ULONG) << 16)
                        | ((*ip.add(2) as ULONG) << 8)
                        | (*ip.add(3) as ULONG);

                status = SRB_STATUS_SUCCESS as ULONG;
            }
            x if x == SCSIOP_VERIFY as ULONG => {
                // atapi.c:5598
                status = IdeVerify(HwDeviceExtension, Srb);
            }
            x if x == SCSIOP_READ as ULONG || x == SCSIOP_WRITE as ULONG => {
                // atapi.c:5603
                status = IdeReadWrite(HwDeviceExtension, Srb);
            }
            x if x == SCSIOP_START_STOP_UNIT as ULONG => {
                // atapi.c:5610
                cdb = (*Srb).Cdb.as_mut_ptr() as PCDB;

                if (*cdb).START_STOP.as_ref().LoadEject() == 1 {
                    // Eject media, first select device 0 or 1.
                    ScsiPortWritePortUchar(
                        &mut (*baseIoAddress1).DriveSelect,
                        ((((*Srb).TargetId & 0x1) << 4) | 0xA0) as UCHAR,
                    );
                    ScsiPortWritePortUchar(
                        &mut (*baseIoAddress1).Command,
                        IDE_COMMAND_MEDIA_EJECT as UCHAR,
                    );
                }
                status = SRB_STATUS_SUCCESS as ULONG;
            }
            x if x == SCSIOP_REQUEST_SENSE as ULONG => {
                // atapi.c:5630 — falls through to default if MSN not enabled.
                if ((*deviceExtension).DeviceFlags[(*Srb).TargetId as usize] as ULONG)
                    & (DFLAGS_MEDIA_STATUS_ENABLED as ULONG)
                    != 0
                {
                    status = IdeBuildSenseBuffer(HwDeviceExtension, Srb);
                    break 'switch_block;
                }
                // C fall-through into default:
                status = SRB_STATUS_INVALID_REQUEST as ULONG;
            }
            _ => {
                // atapi.c:5639 default
                status = SRB_STATUS_INVALID_REQUEST as ULONG;
            }
        }
    } // end 'switch_block (label is target of break for the SCSIOP_REQUEST_SENSE early path)

    // atapi.c:5649
    status
}
