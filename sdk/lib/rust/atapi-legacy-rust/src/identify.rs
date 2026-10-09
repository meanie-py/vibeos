use ntkrnl_sys::ddk::{
    UCHAR, USHORT, ULONG, LONG, BOOLEAN, PVOID, PUCHAR, PUSHORT, PULONG, PCHAR,
};

use crate::types::{
    PHW_DEVICE_EXTENSION, PIDE_REGISTERS_1, PIDE_REGISTERS_2,
    PIDENTIFY_DATA2, IDENTIFY_DATA2, PMECHANICAL_STATUS_INFORMATION_HEADER,
    IDE_COMMAND_IDENTIFY, IDE_COMMAND_ATAPI_IDENTIFY, IDE_COMMAND_SET_DRIVE_PARAMETERS,
    IDE_COMMAND_SET_MULTIPLE,
    IDE_STATUS_ERROR, IDE_STATUS_INDEX, IDE_STATUS_IDLE, IDE_STATUS_BUSY, IDE_STATUS_DRQ,
    IDE_DC_RESET_CONTROLLER, IDE_DC_REENABLE_CONTROLLER,
    DFLAGS_REMOVABLE_DRIVE, DFLAGS_INT_DRQ, DFLAGS_TAPE_DEVICE,
    DFLAGS_DEVICE_PRESENT, DFLAGS_ATAPI_DEVICE, DFLAGS_CHANGER_INITED,
    DFLAGS_SANYO_ATAPI_CHANGER, DFLAGS_ATAPI_CHANGER,
    SRB_STATUS_BUS_RESET,
};

use crate::inlines::{
    GetStatus, GetBaseStatus, WriteCommand, ReadBuffer, WaitOnBusy, WaitOnBaseBusy,
    WaitForDrq, AtapiSoftReset, IdeHardReset,
    ScsiPortReadPortUchar, ScsiPortReadPortUshort, ScsiPortWritePortUchar,
    ScsiPortStallExecution, ScsiPortCompleteRequest, ScsiPortMoveMemory,
    ScsiPortNotification,
};

use crate::helpers::AtapiStringCmp;
use crate::io::IdeMediaStatus;

// NextRequest notification type (from SCSI_NOTIFICATION_TYPE). Used in
// ScsiPortNotification(NextRequest, ...). atapi.c:802
const NextRequest: ULONG = 1;

pub unsafe extern "system" fn IssueIdentify(
    HwDeviceExtension: PVOID,
    DeviceNumber: ULONG,
    Channel: ULONG,
    Command: UCHAR,
) -> BOOLEAN {
    // atapi.c:277
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let baseIoAddress1: PIDE_REGISTERS_1 =
        (*deviceExtension).BaseIoAddress1[Channel as usize];
    let baseIoAddress2: PIDE_REGISTERS_2 =
        (*deviceExtension).BaseIoAddress2[Channel as usize];
    let mut waitCount: ULONG = 20000;
    let mut i: ULONG;
    let mut j: ULONG;
    let mut statusByte: UCHAR;
    let mut signatureLow: UCHAR;
    let mut signatureHigh: UCHAR;

    // Select device 0 or 1. atapi.c:290
    ScsiPortWritePortUchar(
        &mut (*baseIoAddress1).DriveSelect as *mut UCHAR as PUCHAR,
        ((DeviceNumber << 4) | 0xA0) as UCHAR,
    );

    // Check that the status register makes sense. atapi.c:297
    statusByte = GetBaseStatus(baseIoAddress1);

    if Command == IDE_COMMAND_IDENTIFY as UCHAR {
        // Mask status byte ERROR bits. atapi.c:305
        statusByte &= !((IDE_STATUS_ERROR | IDE_STATUS_INDEX) as UCHAR);

        // Check if register value is reasonable. atapi.c:315
        if statusByte != IDE_STATUS_IDLE as UCHAR {
            // Reset the controller. atapi.c:321
            AtapiSoftReset(baseIoAddress1, DeviceNumber);

            ScsiPortWritePortUchar(
                &mut (*baseIoAddress1).DriveSelect as *mut UCHAR as PUCHAR,
                ((DeviceNumber << 4) | 0xA0) as UCHAR,
            );

            WaitOnBusy(baseIoAddress2);

            signatureLow = ScsiPortReadPortUchar(
                &mut (*baseIoAddress1).CylinderLow as *mut UCHAR as PUCHAR,
            );
            signatureHigh = ScsiPortReadPortUchar(
                &mut (*baseIoAddress1).CylinderHigh as *mut UCHAR as PUCHAR,
            );

            if signatureLow == 0x14 && signatureHigh == 0xEB {
                // Device is Atapi. atapi.c:337
                return 0; // FALSE
            }

            // atapi.c:343
            ScsiPortWritePortUchar(
                &mut (*baseIoAddress2).AlternateStatus as *mut UCHAR as PUCHAR,
                IDE_DC_RESET_CONTROLLER as UCHAR,
            );
            ScsiPortStallExecution(500 * 1000);
            ScsiPortWritePortUchar(
                &mut (*baseIoAddress2).AlternateStatus as *mut UCHAR as PUCHAR,
                IDE_DC_REENABLE_CONTROLLER as UCHAR,
            );

            // We really should wait up to 31 seconds. atapi.c:351
            loop {
                // Wait for Busy to drop.
                ScsiPortStallExecution(100);
                statusByte = GetStatus(baseIoAddress2);

                let cond = (statusByte & IDE_STATUS_BUSY as UCHAR) != 0;
                let wc = waitCount;
                waitCount = waitCount.wrapping_sub(1);
                if !(cond && wc != 0) {
                    break;
                }
            }

            ScsiPortWritePortUchar(
                &mut (*baseIoAddress1).DriveSelect as *mut UCHAR as PUCHAR,
                ((DeviceNumber << 4) | 0xA0) as UCHAR,
            );

            // Another check for signature. atapi.c:370
            signatureLow = ScsiPortReadPortUchar(
                &mut (*baseIoAddress1).CylinderLow as *mut UCHAR as PUCHAR,
            );
            signatureHigh = ScsiPortReadPortUchar(
                &mut (*baseIoAddress1).CylinderHigh as *mut UCHAR as PUCHAR,
            );

            if signatureLow == 0x14 && signatureHigh == 0xEB {
                // Device is Atapi. atapi.c:379
                return 0; // FALSE
            }

            statusByte &= !(IDE_STATUS_INDEX as UCHAR);

            if statusByte != IDE_STATUS_IDLE as UCHAR {
                // Give up on this. atapi.c:390
                return 0; // FALSE
            }
        }
    } else {
        // atapi.c:397 DebugPrint dropped
    }

    // Load CylinderHigh and CylinderLow with number bytes to transfer. atapi.c:407
    ScsiPortWritePortUchar(
        &mut (*baseIoAddress1).CylinderHigh as *mut UCHAR as PUCHAR,
        (0x200u32 >> 8) as UCHAR,
    );
    ScsiPortWritePortUchar(
        &mut (*baseIoAddress1).CylinderLow as *mut UCHAR as PUCHAR,
        (0x200u32 & 0xFF) as UCHAR,
    );

    statusByte = 0;
    j = 0;
    while j < 2 {
        // Send IDENTIFY command. atapi.c:416
        statusByte = WaitOnBusy(baseIoAddress2);

        ScsiPortWritePortUchar(
            &mut (*baseIoAddress1).Command as *mut UCHAR as PUCHAR,
            Command,
        );

        // Wait for DRQ. atapi.c:424
        i = 0;
        while i < 4 {
            statusByte = WaitForDrq(baseIoAddress2);

            if statusByte & IDE_STATUS_DRQ as UCHAR != 0 {
                // Read status to acknowledge any interrupts generated. atapi.c:434
                statusByte = GetBaseStatus(baseIoAddress1);

                // One last check for Atapi. atapi.c:441
                signatureLow = ScsiPortReadPortUchar(
                    &mut (*baseIoAddress1).CylinderLow as *mut UCHAR as PUCHAR,
                );
                signatureHigh = ScsiPortReadPortUchar(
                    &mut (*baseIoAddress1).CylinderHigh as *mut UCHAR as PUCHAR,
                );

                if signatureLow == 0x14 && signatureHigh == 0xEB {
                    // Device is Atapi. atapi.c:450
                    return 0; // FALSE
                }

                break;
            }

            if Command == IDE_COMMAND_IDENTIFY as UCHAR {
                // Check the signature. atapi.c:462
                signatureLow = ScsiPortReadPortUchar(
                    &mut (*baseIoAddress1).CylinderLow as *mut UCHAR as PUCHAR,
                );
                signatureHigh = ScsiPortReadPortUchar(
                    &mut (*baseIoAddress1).CylinderHigh as *mut UCHAR as PUCHAR,
                );

                if signatureLow == 0x14 && signatureHigh == 0xEB {
                    // Device is Atapi. atapi.c:471
                    return 0; // FALSE
                }
            }

            statusByte = WaitOnBusy(baseIoAddress2);

            i += 1;
        }

        if i == 4 && j == 0 {
            // Device didn't respond correctly. atapi.c:478
            AtapiSoftReset(baseIoAddress1, DeviceNumber);

            statusByte = GetStatus(baseIoAddress2);
        } else {
            break;
        }

        j += 1;
    }

    // Check for error on really stupid master devices. atapi.c:509
    if (Command == IDE_COMMAND_IDENTIFY as UCHAR)
        && (statusByte & IDE_STATUS_ERROR as UCHAR != 0)
    {
        return 0; // FALSE
    }

    // Suck out 256 words. atapi.c:522
    statusByte = WaitOnBusy(baseIoAddress2);

    if !(statusByte & IDE_STATUS_DRQ as UCHAR != 0) {
        return 0; // FALSE
    }

    ReadBuffer(
        baseIoAddress1,
        &mut (*deviceExtension).FullIdentifyData as *mut _ as PUSHORT,
        256,
    );

    // Check out a few capabilities / limitations of the device. atapi.c:536
    if (*deviceExtension).FullIdentifyData.SpecialFunctionsEnabled & 1 != 0 {
        // Determine if this drive supports the MSN functions. atapi.c:547
        (*deviceExtension).DeviceFlags[((Channel * 2) + DeviceNumber) as usize] |=
            DFLAGS_REMOVABLE_DRIVE as USHORT;
    }

    if (*deviceExtension).FullIdentifyData.MaximumBlockTransfer != 0 {
        // Determine max. block transfer for this device. atapi.c:556
        (*deviceExtension).MaximumBlockXfer[((Channel * 2) + DeviceNumber) as usize] =
            ((*deviceExtension).FullIdentifyData.MaximumBlockTransfer as ULONG & 0xFF) as UCHAR;
    }

    ScsiPortMoveMemory(
        &mut (*deviceExtension).IdentifyData[((Channel * 2) + DeviceNumber) as usize]
            as *mut IDENTIFY_DATA2 as PVOID,
        &mut (*deviceExtension).FullIdentifyData as *mut _ as PVOID,
        core::mem::size_of::<IDENTIFY_DATA2>() as ULONG,
    );

    // atapi.c:562
    if (*deviceExtension).IdentifyData[((Channel * 2) + DeviceNumber) as usize]
        .GeneralConfiguration as ULONG
        & 0x20
        != 0
        && Command != IDE_COMMAND_IDENTIFY as UCHAR
    {
        // This device interrupts with the assertion of DRQ. atapi.c:570
        (*deviceExtension).DeviceFlags[((Channel * 2) + DeviceNumber) as usize] |=
            DFLAGS_INT_DRQ as USHORT;
    } else {
        // atapi.c:577 DebugPrint dropped
    }

    // atapi.c:581
    if (((*deviceExtension).IdentifyData[((Channel * 2) + DeviceNumber) as usize]
        .GeneralConfiguration as ULONG
        & 0xF00)
        == 0x100)
        && Command != IDE_COMMAND_IDENTIFY as UCHAR
    {
        // This is a tape. atapi.c:588
        (*deviceExtension).DeviceFlags[((Channel * 2) + DeviceNumber) as usize] |=
            DFLAGS_TAPE_DEVICE as USHORT;
    } else {
        // atapi.c:595 DebugPrint dropped
    }

    // Work around for some IDE that present more than 256 bytes. atapi.c:604
    WaitOnBusy(baseIoAddress2);

    i = 0;
    while i < 0x10000 {
        statusByte = GetStatus(baseIoAddress2);

        if statusByte & IDE_STATUS_DRQ as UCHAR != 0 {
            // Suck out any remaining bytes and throw away. atapi.c:616
            ScsiPortReadPortUshort(
                &mut (*baseIoAddress1).Data as *mut USHORT as PUSHORT,
            );
        } else {
            break;
        }

        i += 1;
    }

    1 // TRUE  atapi.c:629
}

pub unsafe extern "system" fn SetDriveParameters(
    HwDeviceExtension: PVOID,
    DeviceNumber: ULONG,
    Channel: ULONG,
) -> BOOLEAN {
    // atapi.c:661
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let baseIoAddress1: PIDE_REGISTERS_1 =
        (*deviceExtension).BaseIoAddress1[Channel as usize];
    let baseIoAddress2: PIDE_REGISTERS_2 =
        (*deviceExtension).BaseIoAddress2[Channel as usize];
    let identifyData: PIDENTIFY_DATA2 =
        &mut (*deviceExtension).IdentifyData[((Channel * 2) + DeviceNumber) as usize]
            as *mut IDENTIFY_DATA2 as PIDENTIFY_DATA2;
    let mut i: ULONG;
    let mut statusByte: UCHAR;

    // Set up registers for SET PARAMETER command. atapi.c:680
    ScsiPortWritePortUchar(
        &mut (*baseIoAddress1).DriveSelect as *mut UCHAR as PUCHAR,
        (((DeviceNumber << 4) | 0xA0) | ((*identifyData).NumberOfHeads as ULONG - 1)) as UCHAR,
    );

    ScsiPortWritePortUchar(
        &mut (*baseIoAddress1).BlockCount as *mut UCHAR as PUCHAR,
        (*identifyData).SectorsPerTrack as UCHAR,
    );

    // Send SET PARAMETER command. atapi.c:690
    ScsiPortWritePortUchar(
        &mut (*baseIoAddress1).Command as *mut UCHAR as PUCHAR,
        IDE_COMMAND_SET_DRIVE_PARAMETERS as UCHAR,
    );

    // Wait for up to 30 milliseconds for ERROR or command complete. atapi.c:697
    i = 0;
    while i < 30 * 1000 {
        let _errorByte: UCHAR;

        statusByte = GetStatus(baseIoAddress2);

        if statusByte & IDE_STATUS_ERROR as UCHAR != 0 {
            _errorByte = ScsiPortReadPortUchar(
                (baseIoAddress1 as PUCHAR).add(1),
            );
            return 0; // FALSE  atapi.c:710
        } else if (statusByte & !(IDE_STATUS_INDEX as UCHAR)) == IDE_STATUS_IDLE as UCHAR {
            break;
        } else {
            ScsiPortStallExecution(100);
        }

        i += 1;
    }

    // Check for timeout. atapi.c:722
    if i == 30 * 1000 {
        0 // FALSE
    } else {
        1 // TRUE
    }
}

pub unsafe extern "system" fn AtapiResetController(
    HwDeviceExtension: PVOID,
    PathId: ULONG,
) -> BOOLEAN {
    // atapi.c:756
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let numberChannels: ULONG = (*deviceExtension).NumberChannels;
    let mut baseIoAddress1: PIDE_REGISTERS_1;
    let mut baseIoAddress2: PIDE_REGISTERS_2;
    let mut result: BOOLEAN;
    let mut i: ULONG;
    let mut j: ULONG;
    let mut statusByte: UCHAR;

    let _ = PathId; // PathId unused beyond signature (matches C)

    // Check and see if we are processing an internal srb. atapi.c:769
    if !(*deviceExtension).OriginalSrb.is_null() {
        (*deviceExtension).CurrentSrb = (*deviceExtension).OriginalSrb;
        (*deviceExtension).OriginalSrb = core::ptr::null_mut();
    }

    // Check if request is in progress. atapi.c:778
    if !(*deviceExtension).CurrentSrb.is_null() {
        // Complete outstanding request with SRB_STATUS_BUS_RESET. atapi.c:784
        ScsiPortCompleteRequest(
            deviceExtension as PVOID,
            (*(*deviceExtension).CurrentSrb).PathId,
            (*(*deviceExtension).CurrentSrb).TargetId,
            (*(*deviceExtension).CurrentSrb).Lun,
            SRB_STATUS_BUS_RESET as UCHAR,
        );

        // Clear request tracking fields. atapi.c:794
        (*deviceExtension).CurrentSrb = core::ptr::null_mut();
        (*deviceExtension).WordsLeft = 0;
        (*deviceExtension).DataBuffer = core::ptr::null_mut();

        // Indicate ready for next request. atapi.c:802
        ScsiPortNotification(NextRequest, deviceExtension as PVOID, core::ptr::null_mut::<core::ffi::c_void>());
    }

    // Clear expecting interrupt flag. atapi.c:811
    (*deviceExtension).ExpectingInterrupt = 0; // FALSE
    (*deviceExtension).RDP = 0; // FALSE

    j = 0;
    while j < numberChannels {
        baseIoAddress1 = (*deviceExtension).BaseIoAddress1[j as usize];
        baseIoAddress2 = (*deviceExtension).BaseIoAddress2[j as usize];

        // Do special processing for ATAPI and IDE disk devices. atapi.c:823
        i = 0;
        while i < 2 {
            // Check if device present. atapi.c:829
            if (*deviceExtension).DeviceFlags[(i + (j * 2)) as usize]
                & DFLAGS_DEVICE_PRESENT as USHORT
                != 0
            {
                // Check for ATAPI disk. atapi.c:835
                if (*deviceExtension).DeviceFlags[(i + (j * 2)) as usize]
                    & DFLAGS_ATAPI_DEVICE as USHORT
                    != 0
                {
                    // Issue soft reset and issue identify. atapi.c:841
                    GetStatus(baseIoAddress2);

                    AtapiSoftReset(baseIoAddress1, i);

                    statusByte = GetStatus(baseIoAddress2);

                    if statusByte == 0x0 {
                        IssueIdentify(
                            HwDeviceExtension,
                            i,
                            j,
                            IDE_COMMAND_ATAPI_IDENTIFY as UCHAR,
                        );
                    } else {
                        // atapi.c:858 DebugPrint dropped
                    }
                } else {
                    // Write IDE reset controller bits. atapi.c:869
                    result = IdeHardReset(baseIoAddress2);

                    if result == 0 {
                        return 0; // FALSE
                    }

                    // Set disk geometry parameters. atapi.c:879
                    if SetDriveParameters(HwDeviceExtension, i, j) == 0 {
                        // atapi.c:883 DebugPrint dropped
                    }
                }
            }

            i += 1;
        }

        j += 1;
    }

    // Call the HwInitialize routine to setup multi-block. atapi.c:895
    AtapiHwInitialize(HwDeviceExtension);

    1 // TRUE  atapi.c:897
}

pub unsafe extern "system" fn AtapiHwInitialize(HwDeviceExtension: PVOID) -> BOOLEAN {
    // atapi.c:1302
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let mut baseIoAddress: PIDE_REGISTERS_1;
    let mut i: ULONG;
    let mut statusByte: UCHAR;
    let mut _errorByte: UCHAR;

    i = 0;
    while i < 4 {
        if (*deviceExtension).DeviceFlags[i as usize] & DFLAGS_DEVICE_PRESENT as USHORT != 0 {
            if !((*deviceExtension).DeviceFlags[i as usize] & DFLAGS_ATAPI_DEVICE as USHORT != 0) {
                // Enable media status notification. atapi.c:1317
                baseIoAddress = (*deviceExtension).BaseIoAddress1[(i >> 1) as usize];

                IdeMediaStatus(1 /* TRUE */, HwDeviceExtension, i);

                // If supported, setup Multi-block transfers. atapi.c:1324
                if (*deviceExtension).MaximumBlockXfer[i as usize] != 0 {
                    // Select the device. atapi.c:1330
                    ScsiPortWritePortUchar(
                        &mut (*baseIoAddress).DriveSelect as *mut UCHAR as PUCHAR,
                        (((i & 0x1) << 4) | 0xA0) as UCHAR,
                    );

                    // Setup sector count to reflect the # of blocks. atapi.c:1337
                    ScsiPortWritePortUchar(
                        &mut (*baseIoAddress).BlockCount as *mut UCHAR as PUCHAR,
                        (*deviceExtension).MaximumBlockXfer[i as usize],
                    );

                    // Issue the command. atapi.c:1344
                    ScsiPortWritePortUchar(
                        &mut (*baseIoAddress).Command as *mut UCHAR as PUCHAR,
                        IDE_COMMAND_SET_MULTIPLE as UCHAR,
                    );

                    // Wait for busy to drop. atapi.c:1351
                    statusByte = WaitOnBaseBusy(baseIoAddress);

                    // Check for errors. atapi.c:1358
                    if statusByte & IDE_STATUS_ERROR as UCHAR != 0 {
                        // Read the error register. atapi.c:1364
                        _errorByte = ScsiPortReadPortUchar((baseIoAddress as PUCHAR).add(1));

                        // Adjust the devExt. value, if necessary. atapi.c:1374
                        (*deviceExtension).MaximumBlockXfer[i as usize] = 0;
                    } else {
                        // atapi.c:1377 DebugPrint dropped
                    }
                }
            } else if !((*deviceExtension).DeviceFlags[i as usize]
                & DFLAGS_CHANGER_INITED as USHORT
                != 0)
            {
                // atapi.c:1383
                let mut j: ULONG;
                let mut vendorId: [UCHAR; 26] = [0; 26];

                // Attempt to identify any special-case devices. atapi.c:1392
                j = 0;
                while j < 13 {
                    // Build a buffer based on the identify data. atapi.c:1398
                    let modelPtr = core::ptr::addr_of!(
                        (*deviceExtension).IdentifyData[i as usize].ModelNumber
                    ) as PUCHAR;
                    vendorId[j as usize] = *modelPtr.add((j + 1) as usize);
                    vendorId[(j + 1) as usize] = *modelPtr.add(j as usize);

                    j += 2;
                }

                if AtapiStringCmp(
                    vendorId.as_mut_ptr() as PCHAR,
                    b"CD-ROM  CDR\0".as_ptr() as PCHAR,
                    11,
                ) == 0
                {
                    // Inquiry string for older model had a '-', newer is '_'. atapi.c:1408
                    if vendorId[12] == b'C' {
                        // Torisan changer. Set the bit. atapi.c:1415
                        (*deviceExtension).DeviceFlags[i as usize] |=
                            (DFLAGS_CHANGER_INITED | DFLAGS_SANYO_ATAPI_CHANGER) as USHORT;
                        (*deviceExtension).DiscsPresent[i as usize] = 3;
                    }
                }
            }

            // We need to get our device ready for action. atapi.c:1435
            if (*deviceExtension).DeviceFlags[i as usize] & DFLAGS_ATAPI_DEVICE as USHORT != 0 {
                let baseIoAddress2: PIDE_REGISTERS_2 =
                    (*deviceExtension).BaseIoAddress2[(i >> 1) as usize];
                let mut waitCount: ULONG;

                // 10000 * 100us = 1s. atapi.c:1442
                waitCount = 10000;
                statusByte = GetStatus(baseIoAddress2);
                while (statusByte & IDE_STATUS_BUSY as UCHAR != 0) && waitCount != 0 {
                    // Wait for Busy to drop. atapi.c:1448
                    ScsiPortStallExecution(100);
                    statusByte = GetStatus(baseIoAddress2);
                    waitCount -= 1;
                }

                // 5000 * 100us = 0.5s. atapi.c:1454
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

        i += 1;
    }

    1 // TRUE  atapi.c:1462
}

pub unsafe extern "system" fn AtapiHwInitializeChanger(
    HwDeviceExtension: PVOID,
    TargetId: ULONG,
    MechanismStatus: PMECHANICAL_STATUS_INFORMATION_HEADER,
) {
    // atapi.c:1474
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;

    if !MechanismStatus.is_null() {
        (*deviceExtension).DiscsPresent[TargetId as usize] =
            (*MechanismStatus).NumberAvailableSlots as ULONG;
        if (*deviceExtension).DiscsPresent[TargetId as usize] > 1 {
            (*deviceExtension).DeviceFlags[TargetId as usize] |= DFLAGS_ATAPI_CHANGER as USHORT;
        }
    }
}

pub unsafe extern "system" fn FindDevices(
    HwDeviceExtension: PVOID,
    mut AtapiOnly: BOOLEAN,
    Channel: ULONG,
) -> BOOLEAN {
    // atapi.c:1515
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let baseIoAddress1: PIDE_REGISTERS_1 =
        (*deviceExtension).BaseIoAddress1[Channel as usize];
    let baseIoAddress2: PIDE_REGISTERS_2 =
        (*deviceExtension).BaseIoAddress2[Channel as usize];
    let mut deviceResponded: BOOLEAN = 0; // FALSE
    let mut skipSetParameters: BOOLEAN = 0; // FALSE
    let mut waitCount: ULONG = 10000;
    let mut deviceNumber: ULONG;
    let mut i: ULONG;
    let mut signatureLow: UCHAR;
    let mut signatureHigh: UCHAR;
    let mut statusByte: UCHAR;

    // Clear expecting interrupt flag and current SRB field. atapi.c:1531
    (*deviceExtension).ExpectingInterrupt = 0; // FALSE
    (*deviceExtension).CurrentSrb = core::ptr::null_mut();

    // Search for devices. atapi.c:1538
    deviceNumber = 0;
    while deviceNumber < 2 {
        // Select the device. atapi.c:1544
        ScsiPortWritePortUchar(
            &mut (*baseIoAddress1).DriveSelect as *mut UCHAR as PUCHAR,
            ((deviceNumber << 4) | 0xA0) as UCHAR,
        );

        // Check here for some SCSI adapters that incorporate IDE emulation. atapi.c:1551
        statusByte = GetStatus(baseIoAddress2);
        if statusByte == 0xFF {
            deviceNumber += 1;
            continue;
        }

        AtapiSoftReset(baseIoAddress1, deviceNumber);
        WaitOnBusy(baseIoAddress2);

        signatureLow = ScsiPortReadPortUchar(
            &mut (*baseIoAddress1).CylinderLow as *mut UCHAR as PUCHAR,
        );
        signatureHigh = ScsiPortReadPortUchar(
            &mut (*baseIoAddress1).CylinderHigh as *mut UCHAR as PUCHAR,
        );

        // 'atapiIssueId' is a label jumped to from the IDE-identify-failed path
        // below. We model the goto with a flag-driven inner loop so the
        // ATAPI-identify block can be re-entered. atapi.c:1562 / atapi.c:1570
        let mut do_atapi_issue: bool = signatureLow == 0x14 && signatureHigh == 0xEB;

        if !do_atapi_issue {
            // Issue IDE Identify. atapi.c:1624
            if IssueIdentify(
                HwDeviceExtension,
                deviceNumber,
                Channel,
                IDE_COMMAND_IDENTIFY as UCHAR,
            ) != 0
            {
                // IDE drive found. atapi.c:1638
                (*deviceExtension).DeviceFlags[(deviceNumber + (Channel * 2)) as usize] |=
                    DFLAGS_DEVICE_PRESENT as USHORT;

                if AtapiOnly == 0 {
                    deviceResponded = 1; // TRUE
                }

                // Indicate IDE - not ATAPI device. atapi.c:1648
                (*deviceExtension).DeviceFlags[(deviceNumber + (Channel * 2)) as usize] &=
                    !(DFLAGS_ATAPI_DEVICE as USHORT);
            } else {
                // Look to see if an Atapi device is present. atapi.c:1657
                AtapiSoftReset(baseIoAddress1, deviceNumber);

                WaitOnBusy(baseIoAddress2);

                signatureLow = ScsiPortReadPortUchar(
                    &mut (*baseIoAddress1).CylinderLow as *mut UCHAR as PUCHAR,
                );
                signatureHigh = ScsiPortReadPortUchar(
                    &mut (*baseIoAddress1).CylinderHigh as *mut UCHAR as PUCHAR,
                );

                if signatureLow == 0x14 && signatureHigh == 0xEB {
                    do_atapi_issue = true; // goto atapiIssueId
                }
            }
        }

        if do_atapi_issue {
            // atapiIssueId: atapi.c:1570
            if (*deviceExtension).DriverMustPoll == 0 {
                // Issue ATAPI packet identify command. atapi.c:1578
                if IssueIdentify(
                    HwDeviceExtension,
                    deviceNumber,
                    Channel,
                    IDE_COMMAND_ATAPI_IDENTIFY as UCHAR,
                ) != 0
                {
                    // Indicate ATAPI device. atapi.c:1591
                    (*deviceExtension).DeviceFlags[(deviceNumber + (Channel * 2)) as usize] |=
                        DFLAGS_ATAPI_DEVICE as USHORT;
                    (*deviceExtension).DeviceFlags[(deviceNumber + (Channel * 2)) as usize] |=
                        DFLAGS_DEVICE_PRESENT as USHORT;

                    deviceResponded = 1; // TRUE

                    statusByte = GetStatus(baseIoAddress2);
                    if statusByte & IDE_STATUS_ERROR as UCHAR != 0 {
                        AtapiSoftReset(baseIoAddress1, deviceNumber);
                    }
                } else {
                    // Indicate no working device. atapi.c:1612
                    (*deviceExtension).DeviceFlags[(deviceNumber + (Channel * 2)) as usize] &=
                        !(DFLAGS_DEVICE_PRESENT as USHORT);
                }
            }
        }

        deviceNumber += 1;
    }

    i = 0;
    while i < 2 {
        if ((*deviceExtension).DeviceFlags[(i + (Channel * 2)) as usize]
            & DFLAGS_DEVICE_PRESENT as USHORT
            != 0)
            && (!((*deviceExtension).DeviceFlags[(i + (Channel * 2)) as usize]
                & DFLAGS_ATAPI_DEVICE as USHORT
                != 0))
            && deviceResponded != 0
        {
            // ESDI device garbage geometry hack. atapi.c:1682
            if (*deviceExtension).IdentifyData[i as usize].SectorsPerTrack == 0x35
                && (*deviceExtension).IdentifyData[i as usize].NumberOfHeads == 0x07
            {
                // Change these values to something reasonable. atapi.c:1694
                (*deviceExtension).IdentifyData[i as usize].SectorsPerTrack = 0x34;
                (*deviceExtension).IdentifyData[i as usize].NumberOfHeads = 0x0E;
            }

            if (*deviceExtension).IdentifyData[i as usize].SectorsPerTrack == 0x35
                && (*deviceExtension).IdentifyData[i as usize].NumberOfHeads == 0x0F
            {
                // atapi.c:1712
                (*deviceExtension).IdentifyData[i as usize].SectorsPerTrack = 0x34;
                (*deviceExtension).IdentifyData[i as usize].NumberOfHeads = 0x0F;
            }

            if (*deviceExtension).IdentifyData[i as usize].SectorsPerTrack == 0x36
                && (*deviceExtension).IdentifyData[i as usize].NumberOfHeads == 0x07
            {
                // atapi.c:1731
                (*deviceExtension).IdentifyData[i as usize].SectorsPerTrack = 0x3F;
                (*deviceExtension).IdentifyData[i as usize].NumberOfHeads = 0x10;
                skipSetParameters = 1; // TRUE
            }

            if skipSetParameters == 0 {
                WaitOnBusy(baseIoAddress2);

                // Select the device. atapi.c:1747
                ScsiPortWritePortUchar(
                    &mut (*baseIoAddress1).DriveSelect as *mut UCHAR as PUCHAR,
                    ((i << 4) | 0xA0) as UCHAR,
                );

                statusByte = GetStatus(baseIoAddress2);

                if statusByte & IDE_STATUS_ERROR as UCHAR != 0 {
                    // Reset the device. atapi.c:1761
                    ScsiPortWritePortUchar(
                        &mut (*baseIoAddress2).AlternateStatus as *mut UCHAR as PUCHAR,
                        IDE_DC_RESET_CONTROLLER as UCHAR,
                    );
                    ScsiPortStallExecution(500 * 1000);
                    ScsiPortWritePortUchar(
                        &mut (*baseIoAddress2).AlternateStatus as *mut UCHAR as PUCHAR,
                        IDE_DC_REENABLE_CONTROLLER as UCHAR,
                    );
                    ScsiPortWritePortUchar(
                        &mut (*baseIoAddress1).DriveSelect as *mut UCHAR as PUCHAR,
                        ((i << 4) | 0xA0) as UCHAR,
                    );

                    loop {
                        // Wait for Busy to drop. atapi.c:1767
                        ScsiPortStallExecution(100);
                        statusByte = GetStatus(baseIoAddress2);

                        let cond = (statusByte & IDE_STATUS_BUSY as UCHAR) != 0;
                        let wc = waitCount;
                        waitCount = waitCount.wrapping_sub(1);
                        if !(cond && wc != 0) {
                            break;
                        }
                    }
                }

                WaitOnBusy(baseIoAddress2);

                // Use the IDENTIFY data to set drive parameters. atapi.c:1789
                if SetDriveParameters(HwDeviceExtension, i, Channel) == 0 {
                    // Don't use this device as writes could cause corruption. atapi.c:1799
                    // Diverges from the C, which indexes `i + Channel` here and
                    // `deviceNumber + Channel * 2` below. deviceNumber is 2 once the probe loop
                    // above has run, so on channel 1 the C read past DeviceFlags[4] and the
                    // Rust panicked (a bugcheck); on channel 0 both read the wrong drive's
                    // flags. This loop's drive is `i`.
                    (*deviceExtension).DeviceFlags[(i + (Channel * 2)) as usize] = 0;
                    i += 1;
                    continue;
                }
                if (*deviceExtension).DeviceFlags[(i + (Channel * 2)) as usize]
                    & DFLAGS_REMOVABLE_DRIVE as USHORT
                    != 0
                {
                    // Pick up ALL IDE removable drives. atapi.c:1809
                    AtapiOnly = 0; // FALSE
                }

                // Indicate that a device was found. atapi.c:1817
                if AtapiOnly == 0 {
                    deviceResponded = 1; // TRUE
                }
            }
        }

        i += 1;
    }

    // Make sure master device is selected on exit. atapi.c:1828
    ScsiPortWritePortUchar(
        &mut (*baseIoAddress1).DriveSelect as *mut UCHAR as PUCHAR,
        0xA0,
    );

    // Reset the controller. atapi.c:1838
    if deviceResponded == 0 {
        ScsiPortWritePortUchar(
            &mut (*baseIoAddress2).AlternateStatus as *mut UCHAR as PUCHAR,
            IDE_DC_RESET_CONTROLLER as UCHAR,
        );
        ScsiPortStallExecution(50 * 1000);
        ScsiPortWritePortUchar(
            &mut (*baseIoAddress2).AlternateStatus as *mut UCHAR as PUCHAR,
            IDE_DC_REENABLE_CONTROLLER as UCHAR,
        );
    }

    deviceResponded // atapi.c:1845
}
