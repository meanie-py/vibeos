//! atapi.c IDE command path: SMART, read/write, verify, media-status,
//! sense-buffer. Faithful transliteration (NTAPI = extern "system").

use ntkrnl_sys::ddk::{
    UCHAR, USHORT, ULONG, BOOLEAN, PVOID, PUCHAR, PUSHORT,
    SENDCMDINPARAMS, PSENDCMDINPARAMS, PSENDCMDOUTPARAMS, SENDCMDOUTPARAMS,
    PIDEREGS,
};

use crate::types::{
    PHW_DEVICE_EXTENSION, PIDE_REGISTERS_1, PIDE_REGISTERS_2,
    PCDB, PSENSE_DATA, SRB_IO_CONTROL,
    DFLAGS_DEVICE_PRESENT, DFLAGS_ATAPI_DEVICE, DFLAGS_REMOVABLE_DRIVE,
    DFLAGS_MEDIA_STATUS_ENABLED,
    IDE_STATUS_BUSY, IDE_STATUS_DRQ, IDE_STATUS_ERROR,
    IDE_COMMAND_READ, IDE_COMMAND_READ_MULTIPLE,
    IDE_COMMAND_WRITE, IDE_COMMAND_WRITE_MULTIPLE,
    IDE_COMMAND_VERIFY, IDE_COMMAND_ENABLE_MEDIA_STATUS,
    IDE_ERROR_MEDIA_CHANGE, IDE_ERROR_MEDIA_CHANGE_REQ,
    IDE_ERROR_END_OF_MEDIA, IDE_ERROR_DATA_ERROR,
    SRB_STATUS_SELECTION_TIMEOUT, SRB_STATUS_BUSY, SRB_STATUS_PENDING,
    SRB_STATUS_INVALID_REQUEST, SRB_STATUS_SUCCESS, SRB_STATUS_ERROR,
    SRB_STATUS_TIMEOUT,
    SRB_FLAGS_DATA_IN,
    SCSI_SENSE_UNIT_ATTENTION, SCSI_SENSE_NOT_READY, SCSI_SENSE_DATA_PROTECT,
    SCSI_ADSENSE_MEDIUM_CHANGED, SCSI_ADSENSE_NO_MEDIA_IN_DEVICE,
};

use crate::inlines::{
    GetBaseStatus, WriteBuffer, WaitOnBusy, WaitOnBaseBusy,
    ScsiPortReadPortUchar, ScsiPortWritePortUchar, ScsiPortStallExecution,
};

// SMART command/feature-register constants. These are not present in
// crate::types or ntkrnl_sys::ddk (the atapi driver bindgen did not pull in
// ntdddisk.h). Values transcribed verbatim from sdk/include/psdk/ntdddisk.h
// (the same header atapi.c includes). NOT fabricated — see report.
const SMART_CMD: u32 = 0xB0; // ntdddisk.h:672
const READ_ATTRIBUTE_BUFFER_SIZE: u32 = 512; // ntdddisk.h:726
const READ_ATTRIBUTES: u32 = 0xD0; // ntdddisk.h:739
const READ_THRESHOLDS: u32 = 0xD1; // ntdddisk.h:740
const ENABLE_DISABLE_AUTOSAVE: u32 = 0xD2; // ntdddisk.h:741
const SAVE_ATTRIBUTE_VALUES: u32 = 0xD3; // ntdddisk.h:742
const EXECUTE_OFFLINE_DIAGS: u32 = 0xD4; // ntdddisk.h:743
const ENABLE_SMART: u32 = 0xD8; // ntdddisk.h:746
const DISABLE_SMART: u32 = 0xD9; // ntdddisk.h:747
const RETURN_SMART_STATUS: u32 = 0xDA; // ntdddisk.h:748

pub unsafe extern "system" fn IdeSendSmartCommand(
    HwDeviceExtension: PVOID,
    Srb: crate::types::PSCSI_REQUEST_BLOCK,
) -> ULONG {
    // atapi.c:4342
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let baseIoAddress1: PIDE_REGISTERS_1 =
        (*deviceExtension).BaseIoAddress1[((*Srb).TargetId >> 1) as usize];
    let baseIoAddress2: PIDE_REGISTERS_2 =
        (*deviceExtension).BaseIoAddress2[((*Srb).TargetId >> 1) as usize];
    let cmdOutParameters: PSENDCMDOUTPARAMS =
        (((*Srb).DataBuffer as PUCHAR).add(core::mem::size_of::<SRB_IO_CONTROL>()))
            as PSENDCMDOUTPARAMS;
    let cmdInParameters: SENDCMDINPARAMS = *((((*Srb).DataBuffer as PUCHAR)
        .add(core::mem::size_of::<SRB_IO_CONTROL>()))
        as PSENDCMDINPARAMS);
    let regs: PIDEREGS = core::ptr::addr_of!(cmdInParameters.irDriveRegs) as PIDEREGS;
    let mut i: ULONG;
    let mut statusByte: UCHAR;
    let targetId: UCHAR;

    if cmdInParameters.irDriveRegs.bCommandReg == SMART_CMD as UCHAR {
        targetId = cmdInParameters.bDriveNumber;

        //TODO optimize this check

        if (((*deviceExtension).DeviceFlags[targetId as usize] & DFLAGS_DEVICE_PRESENT as USHORT)
            == 0)
            || ((*deviceExtension).DeviceFlags[targetId as usize] & DFLAGS_ATAPI_DEVICE as USHORT)
                != 0
        {
            return SRB_STATUS_SELECTION_TIMEOUT;
        }

        (*deviceExtension).SmartCommand = cmdInParameters.irDriveRegs.bFeaturesReg;

        //
        // Determine which of the commands to carry out.
        //

        if (cmdInParameters.irDriveRegs.bFeaturesReg == READ_ATTRIBUTES as UCHAR)
            || (cmdInParameters.irDriveRegs.bFeaturesReg == READ_THRESHOLDS as UCHAR)
        {
            statusByte = WaitOnBusy(baseIoAddress2);

            if statusByte & IDE_STATUS_BUSY as UCHAR != 0 {
                return SRB_STATUS_BUSY;
            }

            //
            // Zero the output buffer as the input buffer info. has been saved off locally (the buffers are the same).
            //

            i = 0;
            while i
                < (core::mem::size_of::<SENDCMDOUTPARAMS>() as ULONG
                    + READ_ATTRIBUTE_BUFFER_SIZE
                    - 1)
            {
                *((cmdOutParameters as PUCHAR).add(i as usize)) = 0;
                i = i.wrapping_add(1);
            }

            //
            // Set data buffer pointer and words left.
            //

            (*deviceExtension).DataBuffer = (*cmdOutParameters).bBuffer.as_mut_ptr() as PUSHORT;
            (*deviceExtension).WordsLeft = READ_ATTRIBUTE_BUFFER_SIZE / 2;

            //
            // Indicate expecting an interrupt.
            //

            (*deviceExtension).ExpectingInterrupt = 1 as BOOLEAN;

            ScsiPortWritePortUchar(
                &mut (*baseIoAddress1).DriveSelect as *mut UCHAR,
                (((targetId & 0x1) << 4) | 0xA0) as UCHAR,
            );
            ScsiPortWritePortUchar(
                (baseIoAddress1 as PUCHAR).add(1),
                (*regs).bFeaturesReg,
            );
            ScsiPortWritePortUchar(
                &mut (*baseIoAddress1).BlockCount as *mut UCHAR,
                (*regs).bSectorCountReg,
            );
            ScsiPortWritePortUchar(
                &mut (*baseIoAddress1).BlockNumber as *mut UCHAR,
                (*regs).bSectorNumberReg,
            );
            ScsiPortWritePortUchar(
                &mut (*baseIoAddress1).CylinderLow as *mut UCHAR,
                (*regs).bCylLowReg,
            );
            ScsiPortWritePortUchar(
                &mut (*baseIoAddress1).CylinderHigh as *mut UCHAR,
                (*regs).bCylHighReg,
            );
            ScsiPortWritePortUchar(
                &mut (*baseIoAddress1).Command as *mut UCHAR,
                (*regs).bCommandReg,
            );

            //
            // Wait for interrupt.
            //

            return SRB_STATUS_PENDING;
        } else if (cmdInParameters.irDriveRegs.bFeaturesReg == ENABLE_SMART as UCHAR)
            || (cmdInParameters.irDriveRegs.bFeaturesReg == DISABLE_SMART as UCHAR)
            || (cmdInParameters.irDriveRegs.bFeaturesReg == RETURN_SMART_STATUS as UCHAR)
            || (cmdInParameters.irDriveRegs.bFeaturesReg == ENABLE_DISABLE_AUTOSAVE as UCHAR)
            || (cmdInParameters.irDriveRegs.bFeaturesReg == EXECUTE_OFFLINE_DIAGS as UCHAR)
            || (cmdInParameters.irDriveRegs.bFeaturesReg == SAVE_ATTRIBUTE_VALUES as UCHAR)
        {
            statusByte = WaitOnBusy(baseIoAddress2);

            if statusByte & IDE_STATUS_BUSY as UCHAR != 0 {
                return SRB_STATUS_BUSY;
            }

            //
            // Zero the output buffer as the input buffer info. has been saved off locally (the buffers are the same).
            //

            i = 0;
            while i < (core::mem::size_of::<SENDCMDOUTPARAMS>() as ULONG - 1) {
                *((cmdOutParameters as PUCHAR).add(i as usize)) = 0;
                i = i.wrapping_add(1);
            }

            //
            // Set data buffer pointer and indicate no data transfer.
            //

            (*deviceExtension).DataBuffer = (*cmdOutParameters).bBuffer.as_mut_ptr() as PUSHORT;
            (*deviceExtension).WordsLeft = 0;

            //
            // Indicate expecting an interrupt.
            //

            (*deviceExtension).ExpectingInterrupt = 1 as BOOLEAN;

            ScsiPortWritePortUchar(
                &mut (*baseIoAddress1).DriveSelect as *mut UCHAR,
                (((targetId & 0x1) << 4) | 0xA0) as UCHAR,
            );
            ScsiPortWritePortUchar(
                (baseIoAddress1 as PUCHAR).add(1),
                (*regs).bFeaturesReg,
            );
            ScsiPortWritePortUchar(
                &mut (*baseIoAddress1).BlockCount as *mut UCHAR,
                (*regs).bSectorCountReg,
            );
            ScsiPortWritePortUchar(
                &mut (*baseIoAddress1).BlockNumber as *mut UCHAR,
                (*regs).bSectorNumberReg,
            );
            ScsiPortWritePortUchar(
                &mut (*baseIoAddress1).CylinderLow as *mut UCHAR,
                (*regs).bCylLowReg,
            );
            ScsiPortWritePortUchar(
                &mut (*baseIoAddress1).CylinderHigh as *mut UCHAR,
                (*regs).bCylHighReg,
            );
            ScsiPortWritePortUchar(
                &mut (*baseIoAddress1).Command as *mut UCHAR,
                (*regs).bCommandReg,
            );

            //
            // Wait for interrupt.
            //

            return SRB_STATUS_PENDING;
        }
    }

    SRB_STATUS_INVALID_REQUEST
} // end IdeSendSmartCommand()

pub unsafe extern "system" fn IdeReadWrite(
    HwDeviceExtension: PVOID,
    Srb: crate::types::PSCSI_REQUEST_BLOCK,
) -> ULONG {
    // atapi.c:4499
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let baseIoAddress1: PIDE_REGISTERS_1 =
        (*deviceExtension).BaseIoAddress1[((*Srb).TargetId >> 1) as usize];
    let baseIoAddress2: PIDE_REGISTERS_2 =
        (*deviceExtension).BaseIoAddress2[((*Srb).TargetId >> 1) as usize];
    let startingSector: ULONG;
    let mut i: ULONG;
    let mut wordCount: ULONG;
    let mut statusByte: UCHAR;
    let statusByte2: UCHAR;
    let cylinderHigh: UCHAR;
    let cylinderLow: UCHAR;
    let drvSelect: UCHAR;
    let sectorNumber: UCHAR;

    //
    // Select device 0 or 1.
    //

    ScsiPortWritePortUchar(
        &mut (*baseIoAddress1).DriveSelect as *mut UCHAR,
        ((((*Srb).TargetId & 0x1) << 4) | 0xA0) as UCHAR,
    );

    statusByte2 = WaitOnBusy(baseIoAddress2);

    if statusByte2 & IDE_STATUS_BUSY as UCHAR != 0 {
        return SRB_STATUS_BUSY;
    }

    //
    // Set data buffer pointer and words left.
    //

    (*deviceExtension).DataBuffer = (*Srb).DataBuffer as PUSHORT;
    (*deviceExtension).WordsLeft = (*Srb).DataTransferLength / 2;

    //
    // Indicate expecting an interrupt.
    //

    (*deviceExtension).ExpectingInterrupt = 1 as BOOLEAN;

    //
    // Set up sector count register. Round up to next block.
    //

    ScsiPortWritePortUchar(
        &mut (*baseIoAddress1).BlockCount as *mut UCHAR,
        (((*Srb).DataTransferLength + 0x1FF) / 0x200) as UCHAR,
    );

    //
    // Get starting sector number from CDB.
    //

    let cdb: PCDB = (*Srb).Cdb.as_mut_ptr() as PCDB;
    startingSector = ((*cdb).CDB10.as_ref().LogicalBlockByte3 as ULONG)
        | (((*cdb).CDB10.as_ref().LogicalBlockByte2 as ULONG) << 8)
        | (((*cdb).CDB10.as_ref().LogicalBlockByte1 as ULONG) << 16)
        | (((*cdb).CDB10.as_ref().LogicalBlockByte0 as ULONG) << 24);

    //
    // Set up sector number register.
    //

    sectorNumber = ((startingSector
        % (*deviceExtension).IdentifyData[(*Srb).TargetId as usize].SectorsPerTrack as ULONG)
        + 1) as UCHAR;
    ScsiPortWritePortUchar(
        &mut (*baseIoAddress1).BlockNumber as *mut UCHAR,
        sectorNumber,
    );

    //
    // Set up cylinder low register.
    //

    cylinderLow = (startingSector
        / ((*deviceExtension).IdentifyData[(*Srb).TargetId as usize].SectorsPerTrack as ULONG
            * (*deviceExtension).IdentifyData[(*Srb).TargetId as usize].NumberOfHeads as ULONG))
        as UCHAR;
    ScsiPortWritePortUchar(
        &mut (*baseIoAddress1).CylinderLow as *mut UCHAR,
        cylinderLow,
    );

    //
    // Set up cylinder high register.
    //

    cylinderHigh = ((startingSector
        / ((*deviceExtension).IdentifyData[(*Srb).TargetId as usize].SectorsPerTrack as ULONG
            * (*deviceExtension).IdentifyData[(*Srb).TargetId as usize].NumberOfHeads as ULONG))
        >> 8) as UCHAR;
    ScsiPortWritePortUchar(
        &mut (*baseIoAddress1).CylinderHigh as *mut UCHAR,
        cylinderHigh,
    );

    //
    // Set up head and drive select register.
    //

    drvSelect = (((startingSector
        / (*deviceExtension).IdentifyData[(*Srb).TargetId as usize].SectorsPerTrack as ULONG)
        % (*deviceExtension).IdentifyData[(*Srb).TargetId as usize].NumberOfHeads as ULONG)
        | (((*Srb).TargetId as ULONG & 0x1) << 4)
        | 0xA0) as UCHAR;
    ScsiPortWritePortUchar(
        &mut (*baseIoAddress1).DriveSelect as *mut UCHAR,
        drvSelect,
    );

    //
    // Check if write request.
    //

    if (*Srb).SrbFlags & SRB_FLAGS_DATA_IN != 0 {
        //
        // Send read command.
        //

        if (*deviceExtension).MaximumBlockXfer[(*Srb).TargetId as usize] != 0 {
            ScsiPortWritePortUchar(
                &mut (*baseIoAddress1).Command as *mut UCHAR,
                IDE_COMMAND_READ_MULTIPLE as UCHAR,
            );
        } else {
            ScsiPortWritePortUchar(
                &mut (*baseIoAddress1).Command as *mut UCHAR,
                IDE_COMMAND_READ as UCHAR,
            );
        }
    } else {
        //
        // Send write command.
        //

        if (*deviceExtension).MaximumBlockXfer[(*Srb).TargetId as usize] != 0 {
            wordCount =
                256 * (*deviceExtension).MaximumBlockXfer[(*Srb).TargetId as usize] as ULONG;

            if (*deviceExtension).WordsLeft < wordCount {
                //
                // Transfer only words requested.
                //

                wordCount = (*deviceExtension).WordsLeft;
            }
            ScsiPortWritePortUchar(
                &mut (*baseIoAddress1).Command as *mut UCHAR,
                IDE_COMMAND_WRITE_MULTIPLE as UCHAR,
            );
        } else {
            wordCount = 256;
            ScsiPortWritePortUchar(
                &mut (*baseIoAddress1).Command as *mut UCHAR,
                IDE_COMMAND_WRITE as UCHAR,
            );
        }

        //
        // Wait for BSY and DRQ.
        //

        statusByte = WaitOnBaseBusy(baseIoAddress1);

        if statusByte & IDE_STATUS_BUSY as UCHAR != 0 {
            return SRB_STATUS_BUSY;
        }

        i = 0;
        while i < 1000 {
            statusByte = GetBaseStatus(baseIoAddress1);
            if statusByte & IDE_STATUS_DRQ as UCHAR != 0 {
                break;
            }
            ScsiPortStallExecution(200);
            i = i.wrapping_add(1);
        }

        if statusByte & IDE_STATUS_DRQ as UCHAR == 0 {
            (*deviceExtension).WordsLeft = 0;

            //
            // Clear interrupt expecting flag.
            //

            (*deviceExtension).ExpectingInterrupt = 0 as BOOLEAN;

            //
            // Clear current SRB.
            //

            (*deviceExtension).CurrentSrb = core::ptr::null_mut();

            return SRB_STATUS_TIMEOUT;
        }

        //
        // Write next 256 words.
        //

        WriteBuffer(baseIoAddress1, (*deviceExtension).DataBuffer, wordCount);

        //
        // Adjust buffer address and words left count.
        //

        (*deviceExtension).WordsLeft -= wordCount;
        (*deviceExtension).DataBuffer = (*deviceExtension).DataBuffer.add(wordCount as usize);
    }

    //
    // Wait for interrupt.
    //

    SRB_STATUS_PENDING
} // end IdeReadWrite()

pub unsafe extern "system" fn IdeVerify(
    HwDeviceExtension: PVOID,
    Srb: crate::types::PSCSI_REQUEST_BLOCK,
) -> ULONG {
    // atapi.c:4743
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let baseIoAddress1: PIDE_REGISTERS_1 =
        (*deviceExtension).BaseIoAddress1[((*Srb).TargetId >> 1) as usize];
    //PIDE_REGISTERS_2     baseIoAddress2  = deviceExtension->BaseIoAddress2[Srb->TargetId >> 1];
    let startingSector: ULONG;
    let sectors: ULONG;
    let endSector: ULONG;
    let mut sectorCount: USHORT;

    //
    // Drive has these number sectors.
    //

    sectors = (*deviceExtension).IdentifyData[(*Srb).TargetId as usize].SectorsPerTrack as ULONG
        * (*deviceExtension).IdentifyData[(*Srb).TargetId as usize].NumberOfHeads as ULONG
        * (*deviceExtension).IdentifyData[(*Srb).TargetId as usize].NumberOfCylinders as ULONG;

    //
    // Get starting sector number from CDB.
    //

    let cdb: PCDB = (*Srb).Cdb.as_mut_ptr() as PCDB;
    startingSector = ((*cdb).CDB10.as_ref().LogicalBlockByte3 as ULONG)
        | (((*cdb).CDB10.as_ref().LogicalBlockByte2 as ULONG) << 8)
        | (((*cdb).CDB10.as_ref().LogicalBlockByte1 as ULONG) << 16)
        | (((*cdb).CDB10.as_ref().LogicalBlockByte0 as ULONG) << 24);

    sectorCount = (((*cdb).CDB10.as_ref().TransferBlocksMsb as USHORT) << 8
        | ((*cdb).CDB10.as_ref().TransferBlocksLsb as USHORT)) as USHORT;
    endSector = startingSector + sectorCount as ULONG;

    if endSector > sectors {
        //
        // Too big, round down.
        //

        ScsiPortWritePortUchar(
            &mut (*baseIoAddress1).BlockCount as *mut UCHAR,
            (sectors - startingSector - 1) as UCHAR,
        );
    } else {
        //
        // Set up sector count register. Round up to next block.
        //

        if sectorCount > 0xFF {
            sectorCount = 0xFF as USHORT;
        }

        ScsiPortWritePortUchar(
            &mut (*baseIoAddress1).BlockCount as *mut UCHAR,
            sectorCount as UCHAR,
        );
    }

    //
    // Set data buffer pointer and words left.
    //

    (*deviceExtension).DataBuffer = (*Srb).DataBuffer as PUSHORT;
    (*deviceExtension).WordsLeft = (*Srb).DataTransferLength / 2;

    //
    // Indicate expecting an interrupt.
    //

    (*deviceExtension).ExpectingInterrupt = 1 as BOOLEAN;

    //
    // Set up sector number register.
    //

    ScsiPortWritePortUchar(
        &mut (*baseIoAddress1).BlockNumber as *mut UCHAR,
        ((startingSector
            % (*deviceExtension).IdentifyData[(*Srb).TargetId as usize].SectorsPerTrack as ULONG)
            + 1) as UCHAR,
    );

    //
    // Set up cylinder low register.
    //

    ScsiPortWritePortUchar(
        &mut (*baseIoAddress1).CylinderLow as *mut UCHAR,
        (startingSector
            / ((*deviceExtension).IdentifyData[(*Srb).TargetId as usize].SectorsPerTrack as ULONG
                * (*deviceExtension).IdentifyData[(*Srb).TargetId as usize].NumberOfHeads
                    as ULONG)) as UCHAR,
    );

    //
    // Set up cylinder high register.
    //

    ScsiPortWritePortUchar(
        &mut (*baseIoAddress1).CylinderHigh as *mut UCHAR,
        ((startingSector
            / ((*deviceExtension).IdentifyData[(*Srb).TargetId as usize].SectorsPerTrack as ULONG
                * (*deviceExtension).IdentifyData[(*Srb).TargetId as usize].NumberOfHeads
                    as ULONG))
            >> 8) as UCHAR,
    );

    //
    // Set up head and drive select register.
    //

    ScsiPortWritePortUchar(
        &mut (*baseIoAddress1).DriveSelect as *mut UCHAR,
        (((startingSector
            / (*deviceExtension).IdentifyData[(*Srb).TargetId as usize].SectorsPerTrack as ULONG)
            % (*deviceExtension).IdentifyData[(*Srb).TargetId as usize].NumberOfHeads as ULONG)
            | (((*Srb).TargetId as ULONG & 0x1) << 4)
            | 0xA0) as UCHAR,
    );

    //
    // Send verify command.
    //

    ScsiPortWritePortUchar(
        &mut (*baseIoAddress1).Command as *mut UCHAR,
        IDE_COMMAND_VERIFY as UCHAR,
    );

    //
    // Wait for interrupt.
    //

    SRB_STATUS_PENDING
} // end IdeVerify()

pub unsafe extern "system" fn IdeMediaStatus(
    EnableMSN: BOOLEAN,
    HwDeviceExtension: PVOID,
    Channel: ULONG,
) {
    // atapi.c:5655
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let baseIoAddress: PIDE_REGISTERS_1 =
        (*deviceExtension).BaseIoAddress1[(Channel >> 1) as usize];
    let mut statusByte: UCHAR;
    let _errorByte: UCHAR;

    if EnableMSN != 0 as BOOLEAN {
        //
        // If supported enable Media Status Notification support
        //

        if (*deviceExtension).DeviceFlags[Channel as usize] & DFLAGS_REMOVABLE_DRIVE as USHORT != 0
        {
            //
            // enable
            //
            ScsiPortWritePortUchar((baseIoAddress as PUCHAR).add(1), 0x95 as UCHAR);
            ScsiPortWritePortUchar(
                &mut (*baseIoAddress).Command as *mut UCHAR,
                IDE_COMMAND_ENABLE_MEDIA_STATUS as UCHAR,
            );

            statusByte = WaitOnBaseBusy(baseIoAddress);

            if statusByte & IDE_STATUS_ERROR as UCHAR != 0 {
                //
                // Read the error register.
                //
                _errorByte = ScsiPortReadPortUchar((baseIoAddress as PUCHAR).add(1));
            } else {
                (*deviceExtension).DeviceFlags[Channel as usize] |=
                    DFLAGS_MEDIA_STATUS_ENABLED as USHORT;
                (*deviceExtension).ReturningMediaStatus = 0;
            }
        }
    } else {
        // end if EnableMSN != FALSE

        //
        // disable if previously enabled
        //
        if (*deviceExtension).DeviceFlags[Channel as usize] & DFLAGS_MEDIA_STATUS_ENABLED as USHORT
            != 0
        {
            ScsiPortWritePortUchar((baseIoAddress as PUCHAR).add(1), 0x31 as UCHAR);
            ScsiPortWritePortUchar(
                &mut (*baseIoAddress).Command as *mut UCHAR,
                IDE_COMMAND_ENABLE_MEDIA_STATUS as UCHAR,
            );

            statusByte = WaitOnBaseBusy(baseIoAddress);
            let _ = statusByte;
            (*deviceExtension).DeviceFlags[Channel as usize] &=
                !(DFLAGS_MEDIA_STATUS_ENABLED as USHORT);
        }
    }
}

pub unsafe extern "system" fn IdeBuildSenseBuffer(
    HwDeviceExtension: PVOID,
    Srb: crate::types::PSCSI_REQUEST_BLOCK,
) -> ULONG {
    // atapi.c:5737
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let senseBuffer: PSENSE_DATA = (*Srb).DataBuffer as PSENSE_DATA;

    if !senseBuffer.is_null() {
        if (*deviceExtension).ReturningMediaStatus & IDE_ERROR_MEDIA_CHANGE as UCHAR != 0 {
            (*senseBuffer).set_ErrorCode(0x70);
            (*senseBuffer).set_Valid(1);
            (*senseBuffer).AdditionalSenseLength = 0xb;
            (*senseBuffer).set_SenseKey(SCSI_SENSE_UNIT_ATTENTION as UCHAR);
            (*senseBuffer).AdditionalSenseCode = SCSI_ADSENSE_MEDIUM_CHANGED as UCHAR;
            (*senseBuffer).AdditionalSenseCodeQualifier = 0;
        } else if (*deviceExtension).ReturningMediaStatus & IDE_ERROR_MEDIA_CHANGE_REQ as UCHAR != 0
        {
            (*senseBuffer).set_ErrorCode(0x70);
            (*senseBuffer).set_Valid(1);
            (*senseBuffer).AdditionalSenseLength = 0xb;
            (*senseBuffer).set_SenseKey(SCSI_SENSE_UNIT_ATTENTION as UCHAR);
            (*senseBuffer).AdditionalSenseCode = SCSI_ADSENSE_MEDIUM_CHANGED as UCHAR;
            (*senseBuffer).AdditionalSenseCodeQualifier = 0;
        } else if (*deviceExtension).ReturningMediaStatus & IDE_ERROR_END_OF_MEDIA as UCHAR != 0 {
            (*senseBuffer).set_ErrorCode(0x70);
            (*senseBuffer).set_Valid(1);
            (*senseBuffer).AdditionalSenseLength = 0xb;
            (*senseBuffer).set_SenseKey(SCSI_SENSE_NOT_READY as UCHAR);
            (*senseBuffer).AdditionalSenseCode = SCSI_ADSENSE_NO_MEDIA_IN_DEVICE as UCHAR;
            (*senseBuffer).AdditionalSenseCodeQualifier = 0;
        } else if (*deviceExtension).ReturningMediaStatus & IDE_ERROR_DATA_ERROR as UCHAR != 0 {
            (*senseBuffer).set_ErrorCode(0x70);
            (*senseBuffer).set_Valid(1);
            (*senseBuffer).AdditionalSenseLength = 0xb;
            (*senseBuffer).set_SenseKey(SCSI_SENSE_DATA_PROTECT as UCHAR);
            (*senseBuffer).AdditionalSenseCode = 0;
            (*senseBuffer).AdditionalSenseCodeQualifier = 0;
        }
        return SRB_STATUS_SUCCESS;
    }
    SRB_STATUS_ERROR
} // End of IdeBuildSenseBuffer
