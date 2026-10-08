use crate::prelude::*;
use ntkrnl_sys::ddk::{PMDL};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut AtapReadWriteCommandMap: [[UCHAR; 2]; 12] = [
    [
        IDE_COMMAND_WRITE_EXT as UCHAR,
        IDE_COMMAND_READ_EXT as UCHAR,
    ],
    [
        IDE_COMMAND_WRITE_MULTIPLE_EXT as UCHAR,
        IDE_COMMAND_READ_MULTIPLE_EXT as UCHAR,
    ],
    [
        IDE_COMMAND_WRITE_DMA_EXT as UCHAR,
        IDE_COMMAND_READ_DMA_EXT as UCHAR,
    ],
    [
        0 as ::core::ffi::c_int as UCHAR,
        0 as ::core::ffi::c_int as UCHAR,
    ],
    [
        IDE_COMMAND_WRITE_MULTIPLE_FUA_EXT as UCHAR,
        0 as ::core::ffi::c_int as UCHAR,
    ],
    [
        IDE_COMMAND_WRITE_DMA_FUA_EXT as UCHAR,
        0 as ::core::ffi::c_int as UCHAR,
    ],
    [IDE_COMMAND_WRITE as UCHAR, IDE_COMMAND_READ as UCHAR],
    [
        IDE_COMMAND_WRITE_MULTIPLE as UCHAR,
        IDE_COMMAND_READ_MULTIPLE as UCHAR,
    ],
    [
        IDE_COMMAND_WRITE_DMA as UCHAR,
        IDE_COMMAND_READ_DMA as UCHAR,
    ],
    [
        0 as ::core::ffi::c_int as UCHAR,
        0 as ::core::ffi::c_int as UCHAR,
    ],
    [
        0 as ::core::ffi::c_int as UCHAR,
        0 as ::core::ffi::c_int as UCHAR,
    ],
    [
        0 as ::core::ffi::c_int as UCHAR,
        0 as ::core::ffi::c_int as UCHAR,
    ],
];

unsafe extern "C" fn AtaReadWriteCommand(
    mut Request: PATA_DEVICE_REQUEST,
    mut Device: PATAPORT_IO_CONTEXT,
) -> UCHAR {
    let mut CmdEntry: ULONG = 0;
    if (*Request).Flags & REQUEST_FLAG_LBA48 != 0 {
        if (*Device).DeviceFlags & DEVICE_PIO_FOR_LBA48_XFER != 0 {
            (*Request).Flags &= !REQUEST_FLAG_DMA;
        }
        CmdEntry = 0;
    } else {
        CmdEntry = 6;
    }
    if (*Request).Flags & REQUEST_FLAG_FUA != 0 {
        CmdEntry = CmdEntry.wrapping_add(3);
    }
    if (*Request).Flags & REQUEST_FLAG_DMA != 0 {
        CmdEntry = CmdEntry.wrapping_add(2);
    } else if (*Request).Flags & REQUEST_FLAG_READ_WRITE_MULTIPLE != 0 {
        CmdEntry = CmdEntry.wrapping_add(1);
    }
    return AtapReadWriteCommandMap[CmdEntry as usize][(if (*Request).Flags
        & REQUEST_FLAG_DATA_IN
        != 0
    {
        1
    } else {
        0
    }) as usize];
}
#[no_mangle]
pub unsafe extern "C" fn AtaReqDmaTransferToPioTransfer(
    Request: PATA_DEVICE_REQUEST,
) -> BOOLEAN {
    let Device: PATAPORT_IO_CONTEXT = (*Request).Device as PATAPORT_IO_CONTEXT;

    if (*Request).Flags & REQUEST_FLAG_PACKET_COMMAND != 0 {
        (*Request).Flags &= !REQUEST_DMA_FLAGS;
        return TRUE;
    }

    if (*Request).Flags & REQUEST_FLAG_READ_WRITE != 0 {
        (*Request).Flags &= !REQUEST_DMA_FLAGS;

        (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command = AtaReadWriteCommand(Request, Device);
        if (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command == 0 {
            (*Request).Flags |= REQUEST_DMA_FLAGS;
            return FALSE;
        }

        return TRUE;
    }

    return FALSE;
}

unsafe extern "C" fn AtaReqLbaFromTaskFile(Request: PATA_DEVICE_REQUEST) -> ULONG64 {
    let TaskFile: PATA_TASKFILE = &raw mut (*Request).__bindgen_anon_2.TaskFile;
    let Device: PATAPORT_IO_CONTEXT = (*Request).Device as PATAPORT_IO_CONTEXT;
    let mut Lba: ULONG64;

    if (*Device).DeviceFlags & DEVICE_LBA_MODE != 0 {
        Lba = ((*TaskFile).LowLba as ULONG64) << 0
            | ((*TaskFile).MidLba as ULONG64) << 8
            | ((*TaskFile).HighLba as ULONG64) << 16;

        if (*Request).Flags & REQUEST_FLAG_LBA48 != 0 {
            Lba |= ((*TaskFile).LowLbaEx as ULONG64) << 24
                | ((*TaskFile).MidLbaEx as ULONG64) << 32
                | ((*TaskFile).HighLbaEx as ULONG64) << 40;
        } else {
            Lba |= (((*TaskFile).DriveSelect as i32 & 0x0F as i32) as ULONG64)
                << 24;
        }
    } else {
        let mut Cylinder: ULONG;
        let mut Head: ULONG;
        let mut Sector: ULONG;

        Cylinder = (*TaskFile).LowLba as ULONG;
        Head = ((*TaskFile).DriveSelect as i32 & 0x0F as i32) as ULONG;
        Sector = (((*TaskFile).HighLba as ULONG64) << 8
            | (*TaskFile).MidLba as ULONG64) as ULONG;

        Lba = Cylinder
            .wrapping_mul((*Device).Heads as ULONG)
            .wrapping_add(Head)
            .wrapping_mul((*Device).SectorsPerTrack as ULONG)
            .wrapping_add(Sector.wrapping_sub(1 as ULONG)) as ULONG64;
    }

    return Lba;
}

unsafe extern "C" fn AtaReqTranslateFixedError(
    Request: PATA_DEVICE_REQUEST,
    SK: PUCHAR,
    ASK: PUCHAR,
    ASCQ: PUCHAR,
) -> BOOLEAN {
    if (*Request).Output.__bindgen_anon_1.Status as u32 & IDE_STATUS_DEVICE_FAULT != 0 {
        *SK = SCSI_SENSE_HARDWARE_ERROR as u8;
        *ASK = SCSI_ADSENSE_INTERNAL_TARGET_FAILURE as u8;
        *ASCQ = SCSI_SENSEQ_INTERNAL_TARGET_FAILURE as u8;
    } else if (*Request).Output.__bindgen_anon_2.Error as u32 & IDE_ERROR_DATA_ERROR != 0 {
        if (*Request).Flags & REQUEST_FLAG_DATA_OUT as ULONG != 0 {
            *SK = SCSI_SENSE_DATA_PROTECT as u8;
            *ASK = SCSI_ADSENSE_WRITE_PROTECT as u8;
            *ASCQ = 0 as u8;
        } else {
            *SK = SCSI_SENSE_MEDIUM_ERROR as u8;
            *ASK = SCSI_ADSENSE_UNRECOVERED_ERROR as u8;
            *ASCQ = SCSI_SENSEQ_UNRECOVERED_READ_ERROR as u8;
        }
    } else if (*Request).Output.__bindgen_anon_2.Error as u32 & IDE_ERROR_ID_NOT_FOUND != 0 {
        *SK = SCSI_SENSE_ILLEGAL_REQUEST as u8;
        *ASK = SCSI_ADSENSE_ILLEGAL_BLOCK as u8;
        *ASCQ = SCSI_SENSEQ_LOGICAL_ADDRESS_OUT_OF_RANGE as u8;
    } else if (*Request).Output.__bindgen_anon_2.Error as u32 & IDE_ERROR_CRC_ERROR != 0 {
        *SK = SCSI_SENSE_HARDWARE_ERROR as u8;
        *ASK = SCSI_ADSENSE_LUN_COMMUNICATION as u8;
        *ASCQ = SCSI_SESNEQ_COMM_CRC_ERROR as u8;
    } else if (*Request).Output.__bindgen_anon_2.Error as u32 & IDE_ERROR_MEDIA_CHANGE != 0 {
        *SK = SCSI_SENSE_UNIT_ATTENTION as u8;
        *ASK = SCSI_ADSENSE_MEDIUM_CHANGED as u8;
        *ASCQ = 0 as u8;
    } else if (*Request).Output.__bindgen_anon_2.Error as u32 & IDE_ERROR_MEDIA_CHANGE_REQ != 0 {
        *SK = SCSI_SENSE_UNIT_ATTENTION as u8;
        *ASK = SCSI_ADSENSE_OPERATOR_REQUEST as u8;
        *ASCQ = SCSI_SENSEQ_MEDIUM_REMOVAL as u8;
    } else if (*Request).Output.__bindgen_anon_2.Error as u32 & IDE_ERROR_END_OF_MEDIA != 0 {
        *SK = SCSI_SENSE_NOT_READY as u8;
        *ASK = SCSI_ADSENSE_NO_MEDIA_IN_DEVICE as u8;
        *ASCQ = 0 as u8;
    } else if (*Request).Output.__bindgen_anon_2.Error as u32 & IDE_ERROR_ADDRESS_NOT_FOUND != 0 {
        *SK = SCSI_SENSE_MEDIUM_ERROR as u8;
        *ASK = SCSI_ADSENSE_ADDRESS_MARK_NOT_FOUND_FOR_DATA_FIELD as u8;
        *ASCQ = 0 as u8;
    } else if (*Request).Output.__bindgen_anon_2.Error as u32 & IDE_ERROR_COMMAND_ABORTED != 0 {
        *SK = SCSI_SENSE_ABORTED_COMMAND as u8;
        *ASK = SCSI_ADSENSE_NO_SENSE as u8;
        *ASCQ = 0 as u8;
    } else {
        *SK = SCSI_SENSE_ABORTED_COMMAND as u8;
        *ASK = SCSI_ADSENSE_NO_SENSE as u8;
        *ASCQ = 0 as u8;
        return FALSE as BOOLEAN;
    }
    return TRUE as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn AtaReqSetFixedAtaSenseData(
    Request: PATA_DEVICE_REQUEST,
) -> UCHAR {
    let mut SenseCode: SCSI_SENSE_CODE = { let mut __lit: SCSI_SENSE_CODE = core::mem::zeroed(); __lit.__bindgen_anon_1.SrbStatus = 0; __lit.__bindgen_anon_1.SenseKey = 0; __lit.__bindgen_anon_1.AdditionalSenseCode = 0; __lit.__bindgen_anon_1.AdditionalSenseCodeQualifier = 0; __lit };
    let mut SrbStatus: UCHAR;
    let mut SK: UCHAR = core::mem::zeroed();
    let mut ASK: UCHAR = core::mem::zeroed();
    let mut ASCQ: UCHAR = core::mem::zeroed();

    if ((AtaReqTranslateFixedError(
        Request,
        &mut SK,
        &mut ASK,
        &mut ASCQ,
    )) == 0) {
        return (*Request).SrbStatus;
    }

    SenseCode.SrbStatus = (*Request).SrbStatus;
    SenseCode.SenseKey = SK;
    SenseCode.AdditionalSenseCode = ASK;
    SenseCode.AdditionalSenseCodeQualifier = ASCQ;

    SrbStatus = AtaReqSetFixedSenseData((*Request).Srb, SenseCode);
    AtaReqSetLbaInformation((*Request).Srb, AtaReqLbaFromTaskFile(Request));

    return SrbStatus;
}

unsafe extern "C" fn AtaReqCopySatlBuffer(
    mut Request: PATA_DEVICE_REQUEST,
    mut Buffer: PVOID,
    mut Length: ULONG,
) -> ULONG {
    let mut Srb: PSCSI_REQUEST_BLOCK = (*Request).Srb;
    let mut Mdl: PMDL;
    let mut BaseAddress: PVOID;
    let mut Offset: ULONG_PTR;
    let mut BytesCount: ULONG;

    Mdl = (*(*Request).Irp).MdlAddress;

    BaseAddress = MmGetSystemAddressForMdlSafe(Mdl, HighPagePriority);
    if BaseAddress.is_null() {
        return SRB_STATUS_INSUFFICIENT_RESOURCES;
    }

    if Length < (*Srb).DataTransferLength {
        BytesCount = Length;
        (*Request).SrbStatus = (SRB_STATUS_DATA_OVERRUN) as u8;
    } else {
        BytesCount = (*Srb).DataTransferLength;
        (*Request).SrbStatus = (SRB_STATUS_SUCCESS) as u8;
    }
    (*Request).DataTransferLength = BytesCount;

    Offset = (BaseAddress as ULONG_PTR)
        .wrapping_add((*Srb).DataBuffer as ULONG_PTR)
        .wrapping_sub(MmGetMdlVirtualAddress(Mdl) as ULONG_PTR);

    RtlCopyMemory!(Offset as PVOID, Buffer, BytesCount);
    return (*Request).SrbStatus as ULONG;
}

unsafe extern "C" fn AtaReqTerminateInvalidOpCode(mut Srb: PSCSI_REQUEST_BLOCK) -> UCHAR {
    let mut SenseCode: SCSI_SENSE_CODE = { let mut __lit: SCSI_SENSE_CODE = core::mem::zeroed(); __lit.__bindgen_anon_1.SrbStatus = (SRB_STATUS_INVALID_REQUEST) as u8; __lit.__bindgen_anon_1.SenseKey = (SCSI_SENSE_ILLEGAL_REQUEST) as u8; __lit.__bindgen_anon_1.AdditionalSenseCode = (SCSI_ADSENSE_ILLEGAL_COMMAND) as u8; __lit.__bindgen_anon_1.AdditionalSenseCodeQualifier = 0; __lit };
    return AtaReqSetFixedSenseData(Srb, SenseCode);
}

unsafe extern "C" fn AtaReqTerminateInvalidField(Srb: PSCSI_REQUEST_BLOCK) -> UCHAR {
    let mut SenseCode: SCSI_SENSE_CODE = { let mut __lit: SCSI_SENSE_CODE = core::mem::zeroed(); __lit.__bindgen_anon_1.SrbStatus = (SRB_STATUS_INVALID_REQUEST) as u8; __lit.__bindgen_anon_1.SenseKey = (SCSI_SENSE_ILLEGAL_REQUEST) as u8; __lit.__bindgen_anon_1.AdditionalSenseCode = (SCSI_ADSENSE_INVALID_CDB) as u8; __lit.__bindgen_anon_1.AdditionalSenseCodeQualifier = 0; __lit };

    return AtaReqSetFixedSenseData(Srb, SenseCode);
}
#[no_mangle]
pub unsafe extern "C" fn AtaReqTerminateInvalidFieldParameter(
    Srb: PSCSI_REQUEST_BLOCK,
) -> u8 {
    let mut SenseCode: SCSI_SENSE_CODE = { let mut __lit: SCSI_SENSE_CODE = core::mem::zeroed(); __lit.__bindgen_anon_1.SrbStatus = (SRB_STATUS_INVALID_REQUEST) as u8; __lit.__bindgen_anon_1.SenseKey = (SCSI_SENSE_ILLEGAL_REQUEST) as u8; __lit.__bindgen_anon_1.AdditionalSenseCode = (SCSI_ADSENSE_INVALID_FIELD_PARAMETER_LIST) as u8; __lit.__bindgen_anon_1.AdditionalSenseCodeQualifier = 0; __lit };

    return AtaReqSetFixedSenseData(Srb, SenseCode);
}

unsafe extern "C" fn AtaReqTerminateInvalidRange(mut Srb: PSCSI_REQUEST_BLOCK) -> UCHAR {
    let mut SenseCode: SCSI_SENSE_CODE = { let mut __lit: SCSI_SENSE_CODE = core::mem::zeroed(); __lit.__bindgen_anon_1.SrbStatus = 0; __lit.__bindgen_anon_1.SenseKey = 0; __lit.__bindgen_anon_1.AdditionalSenseCode = 0; __lit.__bindgen_anon_1.AdditionalSenseCodeQualifier = 0; __lit };
    SenseCode.SrbStatus = (SRB_STATUS_INVALID_REQUEST) as u8;
    SenseCode.SenseKey = (SCSI_SENSE_ILLEGAL_REQUEST) as u8;
    SenseCode.AdditionalSenseCode = (SCSI_ADSENSE_ILLEGAL_BLOCK) as u8;
    SenseCode.AdditionalSenseCodeQualifier = 0;
    return AtaReqSetFixedSenseData(Srb, SenseCode);
}

unsafe extern "C" fn AtaReqBuildIdentifyCommand(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Request: PATA_DEVICE_REQUEST,
) -> BOOLEAN {
    (*Request).Flags = REQUEST_FLAG_DATA_IN | REQUEST_FLAG_EXCLUSIVE | REQUEST_FLAG_POLL;
    (*Request).DataBuffer = &raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData as PVOID;
    (*Request).DataTransferLength = core::mem::size_of::<IDENTIFY_DEVICE_DATA>() as ULONG;

    if AtaReqAllocateMdl(Request) == 0 {
        return FALSE;
    }

    RtlZeroMemory!(
        &raw mut (*Request).__bindgen_anon_2.TaskFile as *mut core::ffi::c_void,
        core::mem::size_of::<ATA_TASKFILE>(),
    );
    (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command = (IDE_COMMAND_IDENTIFY) as u8;

    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn AtaReqBuildReadLogTaskFile(
    Request: PATA_DEVICE_REQUEST,
    LogAddress: UCHAR,
    PageNumber: UCHAR,
    LogPageCount: USHORT,
) {
    let TaskFile: PATA_TASKFILE = &raw mut (*Request).__bindgen_anon_2.TaskFile;

    (*Request).Flags = REQUEST_FLAG_DATA_IN | REQUEST_FLAG_LBA48;
    (*Request).DataTransferLength = (LogPageCount as u32) * IDE_GP_LOG_SECTOR_SIZE;

    (*TaskFile).__bindgen_anon_2.Feature = 0;
    (*TaskFile).FeatureEx = 0;

    (*TaskFile).SectorCount = LogPageCount as UCHAR;
    (*TaskFile).SectorCountEx = (LogPageCount >> 8) as UCHAR;

    (*TaskFile).LowLba = LogAddress;
    (*TaskFile).MidLba = PageNumber;
    (*TaskFile).HighLba = 0;
    (*TaskFile).LowLbaEx = 0;
    (*TaskFile).MidLbaEx = 0;
    (*TaskFile).HighLbaEx = 0;
    (*TaskFile).__bindgen_anon_1.Command = (IDE_COMMAND_READ_LOG_EXT) as u8;
}

unsafe extern "C" fn AtaReqBuildLbaTaskFile(
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
    mut Request: PATA_DEVICE_REQUEST,
    mut Lba: ULONG64,
    mut SectorCount: ULONG,
) -> BOOLEAN {
    let mut TaskFile: PATA_TASKFILE = &raw mut (*Request).__bindgen_anon_2.TaskFile;
    let mut DriveSelect: UCHAR = 0;

    if SectorCount > ATA_MAX_SECTORS_PER_IO {
        return FALSE;
    }

    (*Request).Flags |= REQUEST_FLAG_SET_DEVICE_REGISTER;

    if (*DevExt).Device.DeviceFlags & DEVICE_LBA_MODE != 0 {
        DriveSelect = (IDE_LBA_MODE) as u8;

        (*TaskFile).__bindgen_anon_2.Feature = 0;
        (*TaskFile).SectorCount = SectorCount as UCHAR;
        (*TaskFile).LowLba = Lba as UCHAR;
        (*TaskFile).MidLba = (Lba >> 8) as UCHAR;
        (*TaskFile).HighLba = (Lba >> 16) as UCHAR;

        if (*DevExt).Device.DeviceFlags & DEVICE_LBA48 != 0
            && ((*Request).Flags & REQUEST_FLAG_FUA != 0
                || AtaCommandUseLba48(Lba, SectorCount) != 0)
        {
            if Lba.wrapping_add(SectorCount as ULONG64) > ATA_MAX_LBA_48 {
                return FALSE;
            }

            (*TaskFile).FeatureEx = 0;
            (*TaskFile).SectorCountEx = (SectorCount >> 8) as UCHAR;
            (*TaskFile).LowLbaEx = (Lba >> 24) as UCHAR;
            (*TaskFile).MidLbaEx = (Lba >> 32) as UCHAR;
            (*TaskFile).HighLbaEx = (Lba >> 40) as UCHAR;

            (*Request).Flags |= REQUEST_FLAG_LBA48;
        } else {
            if Lba.wrapping_add(SectorCount as ULONG64) > ATA_MAX_LBA_28 {
                return FALSE;
            }

            DriveSelect = (DriveSelect as ULONG64
                | (Lba >> 24) & 0x0F as ULONG64)
                as UCHAR;
        }
    } else {
        let mut ChsTemp: ULONG = 0;
        let mut Cylinder: ULONG = 0;
        let mut Head: ULONG = 0;
        let mut Sector: ULONG = 0;

        if Lba.wrapping_add(SectorCount as ULONG64) > ATA_MAX_LBA_28 {
            return FALSE;
        }

        ChsTemp = (Lba as ULONG).wrapping_div((*DevExt).Device.SectorsPerTrack as ULONG);

        Cylinder = ChsTemp.wrapping_div((*DevExt).Device.Heads as ULONG);
        Head = ChsTemp.wrapping_rem((*DevExt).Device.Heads as ULONG);
        Sector = (Lba as ULONG)
            .wrapping_rem((*DevExt).Device.SectorsPerTrack as ULONG)
            .wrapping_add(1);

        if Cylinder > 65535
            || Head > 15
            || Sector > 255
        {
            return FALSE;
        }

        (*TaskFile).__bindgen_anon_2.Feature = 0;
        (*TaskFile).SectorCount = SectorCount as UCHAR;
        (*TaskFile).LowLba = Sector as UCHAR;
        (*TaskFile).MidLba = Cylinder as UCHAR;
        (*TaskFile).HighLba = (Cylinder >> 8) as UCHAR;

        DriveSelect = Head as UCHAR;
    }

    (*TaskFile).DriveSelect = ((*DevExt).Device.__bindgen_anon_1.DeviceSelect as ULONG | DriveSelect as ULONG) as UCHAR;

    return TRUE;
}

unsafe extern "C" fn AtaReqBuildNcqReadWriteTaskFile(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Request: PATA_DEVICE_REQUEST,
    Lba: ULONG64,
    SectorCount: ULONG,
) -> BOOLEAN {
    let TaskFile: PATA_TASKFILE = &raw mut (*Request).__bindgen_anon_2.TaskFile;

    if SectorCount > ATA_MAX_SECTORS_PER_IO {
        return FALSE;
    }

    if Lba.wrapping_add(SectorCount as ULONG64) > ATA_MAX_LBA_48 {
        return FALSE;
    }

    (*Request).Flags |= REQUEST_FLAG_NCQ
        | REQUEST_DMA_FLAGS
        | REQUEST_FLAG_LBA48
        | REQUEST_FLAG_SET_DEVICE_REGISTER;

    (*TaskFile).__bindgen_anon_2.Feature = SectorCount as UCHAR;
    (*TaskFile).FeatureEx = (SectorCount >> 8) as UCHAR;

    (*TaskFile).SectorCount = 0;
    (*TaskFile).SectorCountEx = 0;

    (*TaskFile).LowLba = Lba as UCHAR;
    (*TaskFile).MidLba = (Lba >> 8) as UCHAR;
    (*TaskFile).HighLba = (Lba >> 16) as UCHAR;
    (*TaskFile).LowLbaEx = (Lba >> 24) as UCHAR;
    (*TaskFile).MidLbaEx = (Lba >> 32) as UCHAR;
    (*TaskFile).HighLbaEx = (Lba >> 40) as UCHAR;

    (*TaskFile).DriveSelect = (IDE_LBA_MODE) as u8;
    if (*Request).Flags & REQUEST_FLAG_FUA != 0 {
        (*TaskFile).DriveSelect |= (IDE_DEVICE_FUA_NCQ) as u8;
    }

    if (*Request).Flags & REQUEST_FLAG_DATA_OUT != 0 {
        (*TaskFile).__bindgen_anon_1.Command = (IDE_COMMAND_WRITE_FPDMA_QUEUED) as u8;
    } else {
        (*TaskFile).__bindgen_anon_1.Command = (IDE_COMMAND_READ_FPDMA_QUEUED) as u8;
    }

    return TRUE;
}

unsafe extern "C" fn AtaReqScsiReadWrite(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Request: PATA_DEVICE_REQUEST,
    Srb: PSCSI_REQUEST_BLOCK,
) -> UCHAR {
    let mut Cdb: PCDB;
    let mut Lba: ULONG64;
    let mut SectorCount: ULONG;

    (*Request).Flags = REQUEST_FLAG_READ_WRITE;

    Cdb = ((*Srb).Cdb).as_mut_ptr() as PCDB;
    match ((*Cdb).AsByte[0] as i32) as u32 {
        SCSIOP_READ6 | SCSIOP_WRITE6 => {
            Lba = (CdbGetLogicalBlockAddress6(Cdb)) as _;
        }
        SCSIOP_READ | SCSIOP_WRITE => {
            Lba = (CdbGetLogicalBlockAddress10(Cdb)) as _;
            if (*Cdb).CDB10.ForceUnitAccess() != 0 {
                (*Request).Flags |= REQUEST_FLAG_FUA;
            }
        }
        SCSIOP_READ12 | SCSIOP_WRITE12 => {
            Lba = (CdbGetLogicalBlockAddress12(Cdb)) as _;
            if (*Cdb).CDB12.ForceUnitAccess() != 0 {
                (*Request).Flags |= REQUEST_FLAG_FUA;
            }
        }
        SCSIOP_READ16 | SCSIOP_WRITE16 => {
            Lba = CdbGetLogicalBlockAddress16(Cdb);
            if (*Cdb).CDB16.ForceUnitAccess() != 0 {
                (*Request).Flags |= REQUEST_FLAG_FUA;
            }
        }
        _ => {
            ASSERT!(FALSE);
            UNREACHABLE!();
        }
    }

    if (*Cdb).AsByte[0] & 0x02 != 0 {
        (*Request).Flags |= REQUEST_FLAG_DATA_OUT;
    } else {
        (*Request).Flags |= REQUEST_FLAG_DATA_IN;
    }

    if (*DevExt).Device.DeviceFlags & DEVICE_NCQ == 0 {
        (*Request).Flags &= !REQUEST_FLAG_FUA;
    }

    SectorCount = (*Request).DataTransferLength.wrapping_add(
        (*DevExt).Device.__bindgen_anon_1.SectorSize.wrapping_sub(1),
    );
    SectorCount = SectorCount.wrapping_div((*DevExt).Device.__bindgen_anon_1.SectorSize);

    if (*DevExt).Device.DeviceFlags & DEVICE_NCQ != 0 {
        if AtaReqBuildNcqReadWriteTaskFile(DevExt, Request, Lba, SectorCount) == 0 {
            return AtaReqTerminateInvalidRange(Srb);
        }
    } else {
        if AtaReqBuildLbaTaskFile(DevExt, Request, Lba, SectorCount) == 0 {
            return AtaReqTerminateInvalidRange(Srb);
        }

        if (*DevExt).Device.DeviceFlags & DEVICE_PIO_ONLY == 0
            && SRB_GET_FLAGS!(Srb) & (SRB_FLAG_PIO_RETRY) as ULONG_PTR == 0
        {
            (*Request).Flags |= REQUEST_DMA_FLAGS;
        } else if (*DevExt).Device.__bindgen_anon_1.MultiSectorCount != 0 {
            (*Request).Flags |= REQUEST_FLAG_READ_WRITE_MULTIPLE;
        }

        (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command = AtaReadWriteCommand(Request, &mut (*DevExt).Device);
        if (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command == 0 {
            return AtaReqTerminateInvalidField(Srb);
        }
    }

    return (SRB_STATUS_PENDING) as u8;
}

unsafe extern "C" fn AtaReqScsiSynchronizeCache(
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
    mut Request: PATA_DEVICE_REQUEST,
    mut Srb: PSCSI_REQUEST_BLOCK,
) -> UCHAR {
    let mut Command: UCHAR = 0;
    if AtaDevIsVolatileWriteCacheEnabled(&raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData) == 0 {
        return (SRB_STATUS_SUCCESS) as u8;
    }
    Command = AtaDeviceGetFlushCacheCommand((DevExt).cast());
    if Command as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        return (SRB_STATUS_SUCCESS) as u8;
    }
    (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command = Command;
    return (SRB_STATUS_PENDING) as u8;
}

unsafe extern "C" fn AtaReqScsiVerify(
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
    mut Request: PATA_DEVICE_REQUEST,
    mut Srb: PSCSI_REQUEST_BLOCK,
) -> UCHAR {
    let mut Cdb: PCDB = core::ptr::null_mut();
    let mut Lba: ULONG64 = 0;
    let mut VerificationLength: ULONG = 0;

    Cdb = ((*Srb).Cdb).as_mut_ptr() as PCDB;

    if (*Cdb).VERIFY16.ByteCheck() != 0
        || (*Cdb).VERIFY16.BlockVerify() != 0
    {
        return AtaReqTerminateInvalidField(Srb);
    }

    match (*Cdb).AsByte[0 as usize] as u32 {
        SCSIOP_VERIFY => {
            Lba = CdbGetLogicalBlockAddress10(Cdb) as ULONG64;
            VerificationLength = CdbGetTransferLength10(Cdb) as ULONG;
        }
        SCSIOP_VERIFY12 => {
            Lba = CdbGetLogicalBlockAddress12(Cdb) as ULONG64;
            VerificationLength = CdbGetTransferLength12(Cdb);
        }
        SCSIOP_VERIFY16 => {
            Lba = CdbGetLogicalBlockAddress16(Cdb);
            VerificationLength = CdbGetTransferLength16(Cdb);
        }
        _ => {
            ASSERT!(FALSE);
            UNREACHABLE!();
        }
    }

    if AtaReqBuildLbaTaskFile(DevExt, Request, Lba, VerificationLength) == 0 {
        return AtaReqTerminateInvalidRange(Srb);
    }

    if (*Request).Flags & REQUEST_FLAG_LBA48 != 0 {
        (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command = IDE_COMMAND_VERIFY_EXT as UCHAR;
    } else {
        (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command = IDE_COMMAND_VERIFY as UCHAR;
    }

    return SRB_STATUS_PENDING as UCHAR;
}

unsafe extern "C" fn AtaReqCompleteReadCapacity(mut Request: PATA_DEVICE_REQUEST) -> ATA_COMPLETION_ACTION {
    let mut DevExt: PATAPORT_DEVICE_EXTENSION;
    let mut Srb: PSCSI_REQUEST_BLOCK = (*Request).Srb;
    let mut Length: ULONG;

    DevExt = CONTAINING_RECORD!((*Request).Device, ATAPORT_DEVICE_EXTENSION, Device);

    AtaDeviceSetAddressingMode(DevExt);

    if (*DevExt).Device.TotalSectors == 1 {
        let mut SenseCode: SCSI_SENSE_CODE = core::mem::zeroed();

        SenseCode.SrbStatus = (SRB_STATUS_ERROR) as u8;
        SenseCode.SenseKey = (SCSI_SENSE_NOT_READY) as u8;
        SenseCode.AdditionalSenseCode = (SCSI_ADSENSE_NO_MEDIA_IN_DEVICE) as u8;
        SenseCode.AdditionalSenseCodeQualifier = 0;

        (*Request).SrbStatus = AtaReqSetFixedSenseData(Srb, SenseCode);
        return COMPLETE_IRP;
    }

    if ((*Srb).Cdb[0 as usize] as i32) as u32 == SCSIOP_READ_CAPACITY {
        let mut CapacityData: PREAD_CAPACITY_DATA;
        let mut MaximumLba: ULONG;

        MaximumLba = min(
            (*DevExt).Device.TotalSectors.wrapping_sub(1),
            MAXULONG as ULONG64,
        ) as ULONG;

        CapacityData = (*DevExt).Device.LocalBuffer as PREAD_CAPACITY_DATA;
        (*CapacityData).LogicalBlockAddress = RtlUlongByteSwap(MaximumLba);
        (*CapacityData).BytesPerBlock = RtlUlongByteSwap((*DevExt).Device.__bindgen_anon_1.SectorSize);

        Length = core::mem::size_of::<READ_CAPACITY_DATA>() as ULONG;
    } else {
        let mut CapacityData: PREAD_CAPACITY16_DATA;
        let mut LogicalPerPhysicalExponent: ULONG = core::mem::zeroed();
        let mut LogicalSectorsPerPhysicalSector: ULONG;
        let mut LogicalSectorAlignment: ULONG;
        let mut LowestAlignedBlock: ULONG;
        let mut MaximumLba: ULONG64;

        LogicalSectorAlignment = AtaDevLogicalSectorAlignment(&raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData);
        if LogicalSectorAlignment != 0 {
            LogicalSectorsPerPhysicalSector = AtaDevLogicalSectorsPerPhysicalSector(
                &raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData,
                &raw mut LogicalPerPhysicalExponent,
            );

            LowestAlignedBlock = LogicalSectorsPerPhysicalSector.wrapping_sub(LogicalSectorAlignment);
            LowestAlignedBlock %= LogicalSectorsPerPhysicalSector;
        } else {
            LowestAlignedBlock = 0;
        }

        MaximumLba = (*DevExt).Device.TotalSectors.wrapping_sub(1);

        CapacityData = (*DevExt).Device.LocalBuffer as PREAD_CAPACITY16_DATA;

        RtlZeroMemory!(
            CapacityData as *mut core::ffi::c_void,
            core::mem::size_of::<READ_CAPACITY16_DATA>(),
        );
        (*CapacityData).LogicalBlockAddress.QuadPart = (RtlUlonglongByteSwap(MaximumLba)) as i64;
        (*CapacityData).BytesPerBlock = RtlUlongByteSwap((*DevExt).Device.__bindgen_anon_1.SectorSize);
        (*CapacityData).set_LogicalPerPhysicalExponent((LogicalPerPhysicalExponent as UCHAR) as _);
        (*CapacityData).set_LowestAlignedBlock_MSB(((LowestAlignedBlock >> 8) as UCHAR) as _);
        (*CapacityData).LowestAlignedBlock_LSB = LowestAlignedBlock as UCHAR;

        if AtaDevHasTrimFunction(&raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData) != 0 {
            if AtaDevHasDratFunction(&raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData) != 0 {
                (*CapacityData).set_LBPME((1) as _);

                if AtaDevHasRzatFunction(&raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData) != 0 {
                    (*CapacityData).set_LBPRZ((1) as _);
                }
            }
        }

        Length = CdbGetAllocationLength16((*Srb).Cdb.as_ptr() as PCDB);
    }

    AtaReqCopySatlBuffer(Request, (*DevExt).Device.LocalBuffer, Length);
    return COMPLETE_IRP;
}

unsafe extern "C" fn AtaReqScsiReadCapacity(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Request: PATA_DEVICE_REQUEST,
    Srb: PSCSI_REQUEST_BLOCK,
) -> UCHAR {
    if ((*Srb).Cdb[0 as usize] as i32) as u32 == SCSIOP_READ_CAPACITY {
        if ((*Srb).DataTransferLength as usize) < core::mem::size_of::<READ_CAPACITY_DATA>() {
            return AtaReqTerminateInvalidField(Srb);
        }
    } else if (*Srb).DataTransferLength
        < CdbGetAllocationLength16(core::ptr::addr_of_mut!((*Srb).Cdb) as *mut UCHAR as PCDB)
    {
        return AtaReqTerminateInvalidField(Srb);
    }
    if AtaReqBuildIdentifyCommand(DevExt, Request) == 0 {
        return (SRB_STATUS_INSUFFICIENT_RESOURCES) as u8;
    }
    (*Request).Complete = Some(
        AtaReqCompleteReadCapacity
            as unsafe extern "C" fn(PATA_DEVICE_REQUEST) -> ATA_COMPLETION_ACTION,
    ) as PREQUEST_COMPLETION_ROUTINE;
    return (SRB_STATUS_PENDING) as u8;
}

unsafe extern "C" fn AtaReqControlModePage(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Buffer: PVOID,
) -> ULONG {
    let PageData: PMODE_CONTROL_PAGE = Buffer as PMODE_CONTROL_PAGE;

    (*PageData).set_PageCode((MODE_PAGE_CONTROL) as _);
    (*PageData).PageLength = (core::mem::size_of::<_MODE_CONTROL_PAGE>() as usize)
        .wrapping_sub(RTL_SIZEOF_THROUGH_FIELD!(MODE_CONTROL_PAGE, PageLength)) as UCHAR;

    (*PageData).set_QERR((0) as _);
    (*PageData).set_QueueAlgorithmModifier((1) as _);
    (*PageData).BusyTimeoutPeriod[0] = 0xFF;
    (*PageData).BusyTimeoutPeriod[1] = 0xFF;
    (*PageData).set_D_SENSE((((*DevExt).Device.DeviceFlags & DEVICE_DESCRIPTOR_SENSE != 0) as UCHAR) as _);

    return core::mem::size_of::<_MODE_CONTROL_PAGE>() as ULONG;
}

unsafe extern "C" fn AtaReqControlExtensionModePage(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Buffer: PVOID,
) -> ULONG {
    let PageData: PMODE_CONTROL_EXTENSION_PAGE = Buffer as PMODE_CONTROL_EXTENSION_PAGE;

    (*PageData).set_SubPageFormat((1) as _);
    (*PageData).SubPageCode = 0x00;
    (*PageData).set_PageCode((MODE_PAGE_CONTROL) as _);
    (*PageData).PageLength[1] =
        (core::mem::size_of::<_MODE_CONTROL_EXTENSION_PAGE>() as usize)
            .wrapping_sub(RTL_SIZEOF_THROUGH_FIELD!(MODE_CONTROL_EXTENSION_PAGE, PageLength)) as UCHAR;

    return core::mem::size_of::<_MODE_CONTROL_EXTENSION_PAGE>() as ULONG;
}

unsafe extern "C" fn AtaReqRwErrorRecoveryModePage(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Buffer: PVOID,
) -> ULONG {
    let PageData: PMODE_READ_WRITE_RECOVERY_PAGE = Buffer as PMODE_READ_WRITE_RECOVERY_PAGE;

    (*PageData).set_PageCode((MODE_PAGE_ERROR_RECOVERY) as u8);
    (*PageData).PageLength =
        (core::mem::size_of::<_MODE_READ_WRITE_RECOVERY_PAGE>() as usize)
            .wrapping_sub(RTL_SIZEOF_THROUGH_FIELD!(MODE_READ_WRITE_RECOVERY_PAGE, PageLength)) as UCHAR;

    (*PageData).set_AWRE(1);

    return core::mem::size_of::<_MODE_READ_WRITE_RECOVERY_PAGE>() as ULONG;
}

unsafe extern "C" fn AtaReqCachingModePage(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Buffer: PVOID,
) -> ULONG {
    let PageData: PMODE_CACHING_PAGE_SPC5 = Buffer as PMODE_CACHING_PAGE_SPC5;

    (*PageData).set_PageCode((MODE_PAGE_CACHING) as _);
    (*PageData).PageLength = (core::mem::size_of::<_MODE_CACHING_PAGE_SPC5>()
        - RTL_SIZEOF_THROUGH_FIELD!(MODE_CACHING_PAGE_SPC5, PageLength)) as UCHAR;

    if AtaDevIsVolatileWriteCacheEnabled(&raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData) != 0 {
        (*PageData).set_WriteCacheEnable((1) as _);
    }

    if AtaDevIsReadLookAHeadEnabled(&raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData) != 0 {
        (*PageData).set_DisableReadAHead((1) as _);
    }

    return core::mem::size_of::<_MODE_CACHING_PAGE_SPC5>() as ULONG;
}

unsafe extern "C" fn AtaReqInformationalExceptionsControlModePage(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Buffer: PVOID,
) -> ULONG {
    let PageData: PMODE_INFO_EXCEPTIONS = Buffer as PMODE_INFO_EXCEPTIONS;

    // C: `PageData->PageCode = ...`. The model called the raw bitfield-unit `set` with the page
    // code as the bit WIDTH (28 bits of a 1-byte unit): an index-out-of-bounds panic.
    (*PageData).set_PageCode(MODE_PAGE_FAULT_REPORTING as UCHAR);
    (*PageData).PageLength =
        (core::mem::size_of::<MODE_INFO_EXCEPTIONS>() as usize)
            .wrapping_sub(RTL_SIZEOF_THROUGH_FIELD!(MODE_INFO_EXCEPTIONS, PageLength)) as UCHAR;

    (*PageData).set_ReportMethod(6);

    // Diverges from the C, which returns (and sizes PageLength by) `sizeof(PageData)`, the
    // POINTER: 8 or 4 bytes for a 12-byte page, so the next page overlapped this one.
    return core::mem::size_of::<MODE_INFO_EXCEPTIONS>() as ULONG;
}

unsafe extern "C" fn AtaReqCompleteModeSense(mut Request: PATA_DEVICE_REQUEST) -> ATA_COMPLETION_ACTION {
    let mut DevExt: PATAPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut PageCode: UCHAR = 0;
    let mut ModeDataLength: USHORT = 0;
    let mut Buffer: PUCHAR = core::ptr::null_mut();
    let mut Is6ByteCommand: BOOLEAN = 0;
    let mut IsWriteProtected: BOOLEAN = 0;
    let mut Cdb: PCDB = core::ptr::null_mut();
    let mut ModeHeader: _HEADER = _HEADER {
        Header6: core::ptr::null_mut(),
    };

    DevExt = CONTAINING_RECORD!(
        (*Request).Device,
        ATAPORT_DEVICE_EXTENSION,
        Device,
    );

    RtlZeroMemory!(
        (*DevExt).Device.LocalBuffer as *mut core::ffi::c_void,
        ATA_LOCAL_BUFFER_SIZE,
    );

    Cdb = (*Request).Srb as *mut _ as PCDB;
    Cdb = &raw mut (*(*Request).Srb).Cdb as *mut UCHAR as PCDB;
    Is6ByteCommand = (((*Cdb).AsByte[0 as usize] as i32) as u32
        == SCSIOP_MODE_SENSE) as BOOLEAN;

    Buffer = (*DevExt).Device.LocalBuffer as PUCHAR;

    if (*Request).__bindgen_anon_3.__bindgen_anon_1.TranslationState != 0 {
        IsWriteProtected = (((*Request).Output.__bindgen_anon_1.Status as u32 & IDE_STATUS_ERROR) != 0
            && ((*Request).Output.__bindgen_anon_2.Error as u32 & IDE_ERROR_COMMAND_ABORTED) == 0
            && ((*Request).Output.__bindgen_anon_2.Error as u32 & IDE_ERROR_WRITE_PROTECT) != 0) as BOOLEAN;
    } else {
        IsWriteProtected = FALSE;
    }

    ModeHeader.Header6 = Buffer as PMODE_PARAMETER_HEADER;

    if Is6ByteCommand != 0 {
        if IsWriteProtected != 0 {
            (*ModeHeader.Header6).DeviceSpecificParameter =
                ((*ModeHeader.Header6).DeviceSpecificParameter as i32 | (MODE_DSP_WRITE_PROTECT) as i32) as UCHAR;
        }
        if (*DevExt).Device.DeviceFlags & DEVICE_HAS_FUA != 0 {
            (*ModeHeader.Header6).DeviceSpecificParameter =
                ((*ModeHeader.Header6).DeviceSpecificParameter as i32 | (MODE_DSP_FUA_SUPPORTED) as i32) as UCHAR;
        }
        // C: `Buffer += sizeof(*ModeHeader.Header6);` -- once (the model wrote a second,
        // pointer-sized advance that skipped 8 bytes of the mode data).
        Buffer = Buffer.offset(core::mem::size_of::<_MODE_PARAMETER_HEADER>() as isize);
    } else {
        if IsWriteProtected != 0 {
            (*ModeHeader.Header10).DeviceSpecificParameter =
                ((*ModeHeader.Header10).DeviceSpecificParameter as i32 | (MODE_DSP_WRITE_PROTECT) as i32) as UCHAR;
        }
        if (*DevExt).Device.DeviceFlags & DEVICE_HAS_FUA != 0 {
            (*ModeHeader.Header10).DeviceSpecificParameter =
                ((*ModeHeader.Header10).DeviceSpecificParameter as i32 | (MODE_DSP_FUA_SUPPORTED) as i32) as UCHAR;
        }
        Buffer = Buffer.offset(core::mem::size_of::<_MODE_PARAMETER_HEADER10>() as isize);
    }

    if (*Cdb).MODE_SENSE.Dbd() == 0 {
        let mut FormatDescriptor: PFORMAT_DESCRIPTOR = core::ptr::null_mut();

        if Is6ByteCommand != 0 {
            (*ModeHeader.Header6).BlockDescriptorLength =
                core::mem::size_of::<FORMAT_DESCRIPTOR>() as UCHAR;
        } else {
            (*ModeHeader.Header10).BlockDescriptorLength[0 as usize] =
                (core::mem::size_of::<FORMAT_DESCRIPTOR>() >> 8) as UCHAR;
            (*ModeHeader.Header10).BlockDescriptorLength[1 as usize] =
                core::mem::size_of::<FORMAT_DESCRIPTOR>() as UCHAR;
        }

        FormatDescriptor = Buffer as PFORMAT_DESCRIPTOR;
        (*FormatDescriptor).BlockLength[0 as usize] =
            ((*DevExt).Device.__bindgen_anon_1.SectorSize >> 16) as UCHAR;
        (*FormatDescriptor).BlockLength[1 as usize] =
            ((*DevExt).Device.__bindgen_anon_1.SectorSize >> 8) as UCHAR;
        (*FormatDescriptor).BlockLength[2 as usize] =
            (*DevExt).Device.__bindgen_anon_1.SectorSize as UCHAR;

        Buffer = Buffer.offset(core::mem::size_of::<FORMAT_DESCRIPTOR>() as isize);
    }

    PageCode = (*Cdb).MODE_SENSE.PageCode();

    if (PageCode as i32) as u32 == MODE_PAGE_CONTROL || (PageCode as i32) as u32 == MODE_SENSE_RETURN_ALL {
        let mut SubPageCode: UCHAR = (*Cdb).MODE_SENSE.Reserved3;

        if SubPageCode as i32 == 0x00 || SubPageCode as i32 == 0xFF {
            Buffer = Buffer.offset(AtaReqControlModePage(DevExt, Buffer as PVOID) as isize);
        }
        if SubPageCode as i32 == 0x01 || SubPageCode as i32 == 0xFF {
            Buffer = Buffer.offset(AtaReqControlExtensionModePage(DevExt, Buffer as PVOID) as isize);
        }
    }
    if (PageCode as i32) as u32 == MODE_PAGE_ERROR_RECOVERY || (PageCode as i32) as u32 == MODE_SENSE_RETURN_ALL {
        Buffer = Buffer.offset(AtaReqRwErrorRecoveryModePage(DevExt, Buffer as PVOID) as isize);
    }
    if (PageCode as i32) as u32 == MODE_PAGE_CACHING || (PageCode as i32) as u32 == MODE_SENSE_RETURN_ALL {
        Buffer = Buffer.offset(AtaReqCachingModePage(DevExt, Buffer as PVOID) as isize);
    }
    if (PageCode as i32) as u32 == MODE_PAGE_FAULT_REPORTING || (PageCode as i32) as u32 == MODE_SENSE_RETURN_ALL {
        Buffer = Buffer.offset(AtaReqInformationalExceptionsControlModePage(DevExt, Buffer as PVOID) as isize);
    }

    ModeDataLength = Buffer.offset_from((*DevExt).Device.LocalBuffer as PUCHAR) as USHORT;

    if Is6ByteCommand != 0 {
        ModeDataLength = (ModeDataLength as u64).wrapping_sub(
            RTL_SIZEOF_THROUGH_FIELD!(MODE_PARAMETER_HEADER, MediumType) as u64,
        ) as USHORT;
        (*ModeHeader.Header6).ModeDataLength = ModeDataLength as UCHAR;
    } else {
        ModeDataLength = (ModeDataLength as u64).wrapping_sub(
            RTL_SIZEOF_THROUGH_FIELD!(MODE_PARAMETER_HEADER10, MediumType) as u64,
        ) as USHORT;
        (*ModeHeader.Header10).ModeDataLength[0 as usize] =
            (ModeDataLength as i32 >> 8) as UCHAR;
        (*ModeHeader.Header10).ModeDataLength[1 as usize] =
            ModeDataLength as UCHAR;
    }

    AtaReqCopySatlBuffer(
        Request,
        (*DevExt).Device.LocalBuffer,
        Buffer.offset_from((*DevExt).Device.LocalBuffer as PUCHAR) as ULONG,
    );
    return COMPLETE_IRP;
}

unsafe extern "C" fn AtaReqScsiModeSense(
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
    mut Request: PATA_DEVICE_REQUEST,
    mut Srb: PSCSI_REQUEST_BLOCK,
    mut Is6ByteCommand: BOOLEAN,
) -> UCHAR {
    let mut Cdb: PCDB = &raw mut (*Srb).Cdb as *mut UCHAR as PCDB;
    if ((*Cdb).MODE_SENSE.Pc() as ::core::ffi::c_int) as u32 == MODE_SENSE_SAVED_VALUES {
        return AtaReqTerminateInvalidField(Srb);
    }
    match ((*Cdb).MODE_SENSE.PageCode() as ::core::ffi::c_int) as u32 {
        MODE_PAGE_FAULT_REPORTING => {
            if AtaDevHasSmartFeature(&raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData) == 0 {
                return AtaReqTerminateInvalidField(Srb);
            }
        }
        MODE_PAGE_CONTROL => {
            let mut SubPageCode: UCHAR = (*Cdb).MODE_SENSE.Reserved3;
            if SubPageCode as ::core::ffi::c_int != 0x00
                && SubPageCode as ::core::ffi::c_int != 0x01
                && SubPageCode as ::core::ffi::c_int != 0xFF
            {
                return AtaReqTerminateInvalidField(Srb);
            }
        }
        MODE_PAGE_ERROR_RECOVERY
        | MODE_PAGE_POWER_CONDITION
        | MODE_PAGE_CACHING
        | MODE_SENSE_RETURN_ALL => {}
        _ => return AtaReqTerminateInvalidField(Srb),
    }
    if AtaReqBuildIdentifyCommand(DevExt, Request) == 0 {
        return (SRB_STATUS_INSUFFICIENT_RESOURCES) as u8;
    }
    (*Request).Complete = Some(
        AtaReqCompleteModeSense
            as unsafe extern "C" fn(PATA_DEVICE_REQUEST) -> ATA_COMPLETION_ACTION,
    ) as PREQUEST_COMPLETION_ROUTINE;
    return (SRB_STATUS_PENDING) as u8;
}

unsafe extern "C" fn AtaReqScsiModeSelect(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Request: PATA_DEVICE_REQUEST,
    Srb: PSCSI_REQUEST_BLOCK,
    Is6ByteCommand: BOOLEAN,
) -> UCHAR {
    return AtaReqTerminateInvalidField(Srb);
}
#[no_mangle]
pub unsafe extern "C" fn AtaCreateStandardInquiryData(
    DevExt: PATAPORT_DEVICE_EXTENSION,
) {
    let IdentifyData: PIDENTIFY_DEVICE_DATA = &raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData;
    let InquiryData: PINQUIRYDATA = &raw mut (*DevExt).InquiryData;
    let mut Offset: ULONG;
    let mut VersionDescriptor: PUCHAR;

    PAGED_CODE!();

    (*InquiryData).set_DeviceType((DIRECT_ACCESS_DEVICE) as _);
    (*InquiryData).set_RemovableMedia((AtaDevIsRemovable(IdentifyData)) as _);
    (*InquiryData).__bindgen_anon_1.Versions = 0x07;
    (*InquiryData).set_ResponseDataFormat((2) as _);

    if (*DevExt).Device.DeviceFlags & DEVICE_NCQ != 0 {
        (*InquiryData).set_CommandQueue((1) as _);
    }

    RtlCopyMemory!(
        &raw mut (*InquiryData).VendorId as *mut UCHAR as *mut core::ffi::c_void,
        b"ATA     \0".as_ptr() as *const core::ffi::c_void,
        RTL_FIELD_SIZE!(INQUIRYDATA, VendorId) as u32,
    );

    AtaCopyIdStringUnsafe(
        &raw mut (*InquiryData).ProductId as PUCHAR,
        &raw mut (*IdentifyData).ModelNumber as PUCHAR,
        RTL_FIELD_SIZE!(INQUIRYDATA, ProductId) as u32,
    );

    if (*IdentifyData).FirmwareRevision[4 as usize] as i32 == ' ' as i32
        && (*IdentifyData).FirmwareRevision[5 as usize] as i32 == ' ' as i32
        && (*IdentifyData).FirmwareRevision[6 as usize] as i32 == ' ' as i32
        && (*IdentifyData).FirmwareRevision[7 as usize] as i32 == ' ' as i32
    {
        Offset = 0;
    } else {
        Offset = 4;
    }

    AtaCopyIdStringUnsafe(
        &raw mut (*InquiryData).ProductRevisionLevel as PUCHAR,
        (&raw mut (*IdentifyData).FirmwareRevision as *mut UCHAR).offset(Offset as isize) as PUCHAR,
        RTL_FIELD_SIZE!(INQUIRYDATA, ProductRevisionLevel) as u32,
    );

    VersionDescriptor = (&raw mut (*InquiryData).Reserved3 as *mut UCHAR)
        .offset(3 as isize) as PUCHAR;

    *VersionDescriptor = 0x00;
    VersionDescriptor = VersionDescriptor.offset(1);
    *VersionDescriptor = 0xA0;
    VersionDescriptor = VersionDescriptor.offset(1);

    *VersionDescriptor = 0x05;
    VersionDescriptor = VersionDescriptor.offset(1);
    *VersionDescriptor = 0xC0;
    VersionDescriptor = VersionDescriptor.offset(1);

    *VersionDescriptor = 0x06;
    VersionDescriptor = VersionDescriptor.offset(1);
    *VersionDescriptor = 0x00;
    VersionDescriptor = VersionDescriptor.offset(1);

    if AtaDevHasIeee1667(IdentifyData) != 0 {
        *VersionDescriptor = 0xFF;
        VersionDescriptor = VersionDescriptor.offset(1);
        *VersionDescriptor = 0xC0;
        VersionDescriptor = VersionDescriptor.offset(1);
    }

    if (*IdentifyData).MajorRevision as i32 != 0xFFFF as i32 {
        if (*IdentifyData).MajorRevision as i32
            & (1 << 8)
            != 0
        {
            *VersionDescriptor = 0x16;
            VersionDescriptor = VersionDescriptor.offset(1);
            *VersionDescriptor = 0x23;
            VersionDescriptor = VersionDescriptor.offset(1);
        } else if (*IdentifyData).MajorRevision as i32
            & (1 << 7)
            != 0
        {
            if (*IdentifyData).MinorRevision as i32 == 0x1D as i32 {
                *VersionDescriptor = 0x16;
                VersionDescriptor = VersionDescriptor.offset(1);
                *VersionDescriptor = 0x1C;
                VersionDescriptor = VersionDescriptor.offset(1);
            } else {
                *VersionDescriptor = 0x16;
                VersionDescriptor = VersionDescriptor.offset(1);
                *VersionDescriptor = 0x00;
                VersionDescriptor = VersionDescriptor.offset(1);
            }
        } else if (*IdentifyData).MajorRevision as i32
            & (1 << 6)
            != 0
        {
            if (*IdentifyData).MinorRevision as i32 == 0x22 as i32 {
                *VersionDescriptor = 0x15;
                VersionDescriptor = VersionDescriptor.offset(1);
                *VersionDescriptor = 0xFD;
                VersionDescriptor = VersionDescriptor.offset(1);
            } else {
                *VersionDescriptor = 0x15;
                VersionDescriptor = VersionDescriptor.offset(1);
                *VersionDescriptor = 0xE0;
                VersionDescriptor = VersionDescriptor.offset(1);
            }
        }
    }

    (*InquiryData).AdditionalLength = (VersionDescriptor as usize)
        .wrapping_sub(InquiryData as usize)
        .wrapping_sub(RTL_SIZEOF_THROUGH_FIELD!(INQUIRYDATA, AdditionalLength))
        as UCHAR;
}

unsafe extern "C" fn AtaReqScsiReportLuns(
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
    mut Request: PATA_DEVICE_REQUEST,
    mut Srb: PSCSI_REQUEST_BLOCK,
) -> ULONG {
    let mut Cdb: PCDB = &raw mut (*Srb).Cdb as *mut UCHAR as PCDB;
    let mut LunList: PLUN_LIST = core::ptr::null_mut();
    let mut Length: ULONG = 0;
    let mut LunListLength: ULONG = 0;

    Length = (((*Cdb).REPORT_LUNS.AllocationLength[0] as i32)
        << 24
        | ((*Cdb).REPORT_LUNS.AllocationLength[1] as i32)
            << 16
        | ((*Cdb).REPORT_LUNS.AllocationLength[2] as i32)
            << 8
        | (*Cdb).REPORT_LUNS.AllocationLength[3] as i32)
        as ULONG;

    if (*Srb).DataTransferLength < Length {
        return AtaReqTerminateInvalidField(Srb) as ULONG;
    }

    LunList = (*DevExt).Device.LocalBuffer as PLUN_LIST;
    LunListLength = RTL_FIELD_SIZE!(LUN_LIST, Lun) as ULONG;

    RtlZeroMemory!(
        LunList as *mut core::ffi::c_void,
        core::mem::size_of::<_LUN_LIST>(),
    );

    (*LunList).LunListLength[0] =
        (LunListLength >> 24) as UCHAR;
    (*LunList).LunListLength[1] =
        (LunListLength >> 16) as UCHAR;
    (*LunList).LunListLength[2] =
        (LunListLength >> 8) as UCHAR;
    (*LunList).LunListLength[3] = LunListLength as UCHAR;

    return AtaReqCopySatlBuffer(Request, (*DevExt).Device.LocalBuffer, Length);
}

unsafe extern "C" fn AtaReqScsiInquirySupportedPages(
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
) -> ULONG {
    let mut SupportedPages: PVPD_SUPPORTED_PAGES_PAGE =
        (*DevExt).Device.LocalBuffer as PVPD_SUPPORTED_PAGES_PAGE;
    let mut i: ULONG = 0;

    RtlZeroMemory!(
        SupportedPages as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<VPD_SUPPORTED_PAGES_PAGE>(),
    );

    let fresh0 = i;
    i = i.wrapping_add(1);
    *(&raw mut (*SupportedPages).SupportedPageList as *mut UCHAR).offset(fresh0 as isize) =
        VPD_SUPPORTED_PAGES as UCHAR;

    let fresh1 = i;
    i = i.wrapping_add(1);
    *(&raw mut (*SupportedPages).SupportedPageList as *mut UCHAR).offset(fresh1 as isize) =
        VPD_SERIAL_NUMBER as UCHAR;

    let fresh2 = i;
    i = i.wrapping_add(1);
    *(&raw mut (*SupportedPages).SupportedPageList as *mut UCHAR).offset(fresh2 as isize) =
        VPD_DEVICE_IDENTIFIERS as UCHAR;

    let fresh3 = i;
    i = i.wrapping_add(1);
    *(&raw mut (*SupportedPages).SupportedPageList as *mut UCHAR).offset(fresh3 as isize) =
        VPD_BLOCK_LIMITS as UCHAR;

    let fresh4 = i;
    i = i.wrapping_add(1);
    *(&raw mut (*SupportedPages).SupportedPageList as *mut UCHAR).offset(fresh4 as isize) =
        VPD_BLOCK_DEVICE_CHARACTERISTICS as UCHAR;

    let fresh5 = i;
    i = i.wrapping_add(1);
    *(&raw mut (*SupportedPages).SupportedPageList as *mut UCHAR).offset(fresh5 as isize) =
        VPD_LOGICAL_BLOCK_PROVISIONING as UCHAR;

    (*SupportedPages).PageLength = i as UCHAR;

    return (::core::mem::size_of::<VPD_SUPPORTED_PAGES_PAGE>() as usize).wrapping_add(i as usize)
        as ULONG;
}

unsafe extern "C" fn AtaReqScsiInquirySerialNumber(
    DevExt: PATAPORT_DEVICE_EXTENSION,
) -> ULONG {
    let SerialNumberPage: PVPD_SERIAL_NUMBER_PAGE =
        (*DevExt).Device.LocalBuffer as PVPD_SERIAL_NUMBER_PAGE;

    RtlZeroMemory!(
        SerialNumberPage as *mut core::ffi::c_void,
        FIELD_OFFSET!(VPD_SERIAL_NUMBER_PAGE, SerialNumber),
    );
    (*SerialNumberPage).PageLength =
        RTL_FIELD_SIZE!(IDENTIFY_DEVICE_DATA, SerialNumber) as UCHAR;

    AtaCopyIdStringUnsafe(
        &raw mut (*SerialNumberPage).SerialNumber as PUCHAR,
        &raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData.SerialNumber as PUCHAR,
        RTL_FIELD_SIZE!(IDENTIFY_DEVICE_DATA, SerialNumber) as ULONG,
    );

    return (FIELD_OFFSET!(VPD_SERIAL_NUMBER_PAGE, SerialNumber) as usize)
        .wrapping_add(RTL_FIELD_SIZE!(IDENTIFY_DEVICE_DATA, SerialNumber) as usize)
        as ULONG;
}

unsafe extern "C" fn AtaReqScsiInquiryDeviceIdentifiers(
    DevExt: PATAPORT_DEVICE_EXTENSION,
) -> ULONG {
    let mut DeviceIdentificationPage: PVPD_IDENTIFICATION_PAGE =
        (*DevExt).Device.LocalBuffer as PVPD_IDENTIFICATION_PAGE;
    let mut Descriptor: PVPD_IDENTIFICATION_DESCRIPTOR;
    let mut PageLength: UCHAR;

    PageLength = (FIELD_OFFSET!(VPD_IDENTIFICATION_PAGE, Descriptors)
        + FIELD_OFFSET!(VPD_IDENTIFICATION_DESCRIPTOR, Identifier)) as UCHAR;

    if AtaDevHasWorldWideName(&mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData) != 0 {
        PageLength = PageLength.wrapping_add(RTL_FIELD_SIZE!(IDENTIFY_DEVICE_DATA, WorldWideName) as UCHAR);
    } else {
        PageLength = PageLength
            .wrapping_add(RTL_FIELD_SIZE!(INQUIRYDATA, VendorId) as UCHAR)
            .wrapping_add(RTL_FIELD_SIZE!(IDENTIFY_DEVICE_DATA, ModelNumber) as UCHAR)
            .wrapping_add(RTL_FIELD_SIZE!(IDENTIFY_DEVICE_DATA, SerialNumber) as UCHAR);
    }

    RtlZeroMemory!(
        DeviceIdentificationPage as *mut core::ffi::c_void,
        PageLength as usize,
    );
    (*DeviceIdentificationPage).PageLength = PageLength.wrapping_sub(
        RTL_SIZEOF_THROUGH_FIELD!(VPD_IDENTIFICATION_PAGE, PageLength) as UCHAR,
    );

    Descriptor = (*DeviceIdentificationPage).Descriptors.as_mut_ptr()
        as *mut _ as PVPD_IDENTIFICATION_DESCRIPTOR;

    if AtaDevHasWorldWideName(&mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData) != 0 {
        (*Descriptor).set_CodeSet((VpdCodeSetBinary) as _);
        (*Descriptor).set_IdentifierType((VpdIdentifierTypeFCPHName) as _);
        (*Descriptor).IdentifierLength =
            RTL_FIELD_SIZE!(IDENTIFY_DEVICE_DATA, WorldWideName) as UCHAR;

        AtaCopyIdStringUnsafe(
            (*Descriptor).Identifier.as_mut_ptr(),
            core::ptr::addr_of_mut!((*DevExt).__bindgen_anon_1.IdentifyDeviceData.WorldWideName) as PUCHAR,
            RTL_FIELD_SIZE!(IDENTIFY_DEVICE_DATA, WorldWideName) as ULONG,
        );
    } else {
        let mut Identifier: PUCHAR;

        (*Descriptor).set_CodeSet((VpdCodeSetAscii) as _);
        (*Descriptor).set_IdentifierType((VpdIdentifierTypeVendorId) as _);
        (*Descriptor).IdentifierLength = RTL_FIELD_SIZE!(INQUIRYDATA, VendorId) as UCHAR
            + RTL_FIELD_SIZE!(IDENTIFY_DEVICE_DATA, ModelNumber) as UCHAR
            + RTL_FIELD_SIZE!(IDENTIFY_DEVICE_DATA, SerialNumber) as UCHAR;

        Identifier = (*Descriptor).Identifier.as_mut_ptr();

        RtlCopyMemory!(
            Identifier as *mut core::ffi::c_void,
            b"ATA     \0".as_ptr() as *const core::ffi::c_void,
            RTL_FIELD_SIZE!(INQUIRYDATA, VendorId) as usize,
        );
        Identifier = Identifier.add(RTL_FIELD_SIZE!(INQUIRYDATA, VendorId) as usize);

        AtaCopyIdStringUnsafe(
            Identifier,
            (*DevExt).__bindgen_anon_1.IdentifyDeviceData.ModelNumber.as_mut_ptr(),
            RTL_FIELD_SIZE!(IDENTIFY_DEVICE_DATA, ModelNumber) as ULONG,
        );
        Identifier = Identifier.add(RTL_FIELD_SIZE!(IDENTIFY_DEVICE_DATA, ModelNumber) as usize);

        AtaCopyIdStringUnsafe(
            Identifier,
            (*DevExt).__bindgen_anon_1.IdentifyDeviceData.SerialNumber.as_mut_ptr(),
            RTL_FIELD_SIZE!(IDENTIFY_DEVICE_DATA, SerialNumber) as ULONG,
        );
    }

    return PageLength as ULONG
        + RTL_SIZEOF_THROUGH_FIELD!(VPD_IDENTIFICATION_PAGE, PageLength) as ULONG;
}

unsafe extern "C" fn AtaReqScsiInquiryBlockLimits(mut DevExt: PATAPORT_DEVICE_EXTENSION) -> ULONG {
    let mut BlockLimitsPage: PVPD_BLOCK_LIMITS_PAGE =
        (*DevExt).Device.LocalBuffer as PVPD_BLOCK_LIMITS_PAGE;
    let mut PageLength: USHORT = 0;
    PageLength = (::core::mem::size_of::<_VPD_BLOCK_LIMITS_PAGE>() as usize).wrapping_sub(
        RTL_SIZEOF_THROUGH_FIELD!(VPD_BLOCK_LIMITS_PAGE, PageLength) as usize,
    ) as USHORT;
    RtlZeroMemory!(
        BlockLimitsPage as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<_VPD_BLOCK_LIMITS_PAGE>() as usize,
    );
    (*BlockLimitsPage).PageLength[0 as usize] =
        (PageLength as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as UCHAR;
    (*BlockLimitsPage).PageLength[1 as usize] = PageLength as UCHAR;
    return ::core::mem::size_of::<_VPD_BLOCK_LIMITS_PAGE>() as ULONG;
}

unsafe extern "C" fn AtaReqScsiInquiryBlockDeviceCharacteristics(
    DevExt: PATAPORT_DEVICE_EXTENSION,
) -> ULONG {
    let Characteristics: PVPD_BLOCK_DEVICE_CHARACTERISTICS_PAGE =
        (*DevExt).Device.LocalBuffer as PVPD_BLOCK_DEVICE_CHARACTERISTICS_PAGE;
    let mut MediumRotationRate: USHORT;

    RtlZeroMemory!(
        Characteristics as *mut core::ffi::c_void,
        core::mem::size_of::<VPD_BLOCK_DEVICE_CHARACTERISTICS_PAGE>(),
    );
    (*Characteristics).PageLength = (core::mem::size_of::<VPD_BLOCK_DEVICE_CHARACTERISTICS_PAGE>()
        - RTL_SIZEOF_THROUGH_FIELD!(VPD_BLOCK_DEVICE_CHARACTERISTICS_PAGE, PageLength)) as UCHAR;

    MediumRotationRate = AtaDevMediumRotationRate(&mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData);
    (*Characteristics).MediumRotationRateMsb = (MediumRotationRate >> 8) as UCHAR;
    (*Characteristics).MediumRotationRateLsb = MediumRotationRate as UCHAR;
    (*Characteristics).set_NominalFormFactor((AtaDevNominalFormFactor(&mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData)) as _);
    (*Characteristics).set_ZONED((AtaDevZonedCapabilities(&mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData)) as _);
    return core::mem::size_of::<VPD_BLOCK_DEVICE_CHARACTERISTICS_PAGE>() as ULONG;
}

unsafe extern "C" fn AtaReqScsiInquiryLogicalBlockProvisioning(
    DevExt: PATAPORT_DEVICE_EXTENSION,
) -> ULONG {
    let LogicalBlockProvisioningPage: PVPD_LOGICAL_BLOCK_PROVISIONING_PAGE =
        (*DevExt).Device.LocalBuffer as PVPD_LOGICAL_BLOCK_PROVISIONING_PAGE;

    RtlZeroMemory!(
        LogicalBlockProvisioningPage as *mut core::ffi::c_void,
        FIELD_OFFSET!(VPD_LOGICAL_BLOCK_PROVISIONING_PAGE, ProvisioningGroupDescr),
    );
    (*LogicalBlockProvisioningPage).PageLength[1] =
        (FIELD_OFFSET!(VPD_LOGICAL_BLOCK_PROVISIONING_PAGE, ProvisioningGroupDescr)
            - (RTL_SIZEOF_THROUGH_FIELD!(VPD_LOGICAL_BLOCK_PROVISIONING_PAGE, PageLength)) as i32) as UCHAR;

    if AtaDevHasTrimFunction(&raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData) != 0 {
        if AtaDevHasDratFunction(&raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData) != 0 {
            (*LogicalBlockProvisioningPage).set_ANC_SUP(1);
        }

        if AtaDevHasRzatFunction(&raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData) != 0 {
            (*LogicalBlockProvisioningPage).set_LBPRZ(1);
        }
    }

    return FIELD_OFFSET!(VPD_LOGICAL_BLOCK_PROVISIONING_PAGE, ProvisioningGroupDescr) as ULONG;
}

unsafe extern "C" fn AtaReqScsiInquiry(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Request: PATA_DEVICE_REQUEST,
    Srb: PSCSI_REQUEST_BLOCK,
) -> UCHAR {
    let mut Buffer: PVPD_SUPPORTED_PAGES_PAGE = core::ptr::null_mut();
    let Cdb: PCDB = ((*Srb).Cdb).as_mut_ptr() as PCDB;
    let mut Length: ULONG = 0;

    if (*Cdb).CDB6INQUIRY3.EnableVitalProductData() == 0 {
        if (*Cdb).CDB6INQUIRY3.PageCode != 0 {
            return AtaReqTerminateInvalidField(Srb);
        }
        return AtaReqCopySatlBuffer(
            Request,
            core::ptr::addr_of_mut!((*DevExt).InquiryData) as PVOID,
            core::mem::size_of::<INQUIRYDATA>() as ULONG,
        ) as UCHAR;
    }

    match ((*Cdb).CDB6INQUIRY3.PageCode as i32) as u32 {
        VPD_SUPPORTED_PAGES => {
            Length = AtaReqScsiInquirySupportedPages(DevExt);
        }
        VPD_SERIAL_NUMBER => {
            Length = AtaReqScsiInquirySerialNumber(DevExt);
        }
        VPD_DEVICE_IDENTIFIERS => {
            Length = AtaReqScsiInquiryDeviceIdentifiers(DevExt);
        }
        VPD_EXTENDED_INQUIRY_DATA => {
            return AtaReqTerminateInvalidField(Srb);
        }
        VPD_ATA_INFORMATION => {
            return AtaReqTerminateInvalidField(Srb);
        }
        VPD_BLOCK_LIMITS => {
            Length = AtaReqScsiInquiryBlockLimits(DevExt);
        }
        VPD_BLOCK_DEVICE_CHARACTERISTICS => {
            Length = AtaReqScsiInquiryBlockDeviceCharacteristics(DevExt);
        }
        VPD_LOGICAL_BLOCK_PROVISIONING => {
            Length = AtaReqScsiInquiryLogicalBlockProvisioning(DevExt);
        }
        _ => {
            return AtaReqTerminateInvalidField(Srb);
        }
    }

    Buffer = (*DevExt).Device.LocalBuffer as PVPD_SUPPORTED_PAGES_PAGE;
    (*Buffer).set_DeviceType(((*DevExt).InquiryData.DeviceType()) as _);
    (*Buffer).set_DeviceTypeQualifier(((*DevExt).InquiryData.DeviceTypeQualifier()) as _);
    (*Buffer).PageCode = (*Cdb).CDB6INQUIRY3.PageCode;

    return AtaReqCopySatlBuffer(Request, (*DevExt).Device.LocalBuffer, Length) as UCHAR;
}

unsafe extern "C" fn AtaReqScsiTestUnitReady(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Request: PATA_DEVICE_REQUEST,
    Srb: PSCSI_REQUEST_BLOCK,
) -> UCHAR {
    (SRB_STATUS_SUCCESS) as u8
}

unsafe extern "C" fn AtaReqScsiMediumRemoval(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Request: PATA_DEVICE_REQUEST,
    Srb: PSCSI_REQUEST_BLOCK,
) -> UCHAR {
    let Cdb: PCDB = ((*Srb).Cdb).as_mut_ptr() as PCDB;

    if AtaDevHasRemovableMediaFeature(&raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData) != 0 {
        if (*Cdb).MEDIA_REMOVAL.Prevent() != 0 {
            (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command = (IDE_COMMAND_DOOR_LOCK) as u8;
        } else {
            (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command = (IDE_COMMAND_DOOR_UNLOCK) as u8;
        }
        return (SRB_STATUS_PENDING) as u8;
    }

    return (SRB_STATUS_SUCCESS) as u8;
}

unsafe extern "C" fn AtaReqScsiStartStopUnit(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Request: PATA_DEVICE_REQUEST,
    Srb: PSCSI_REQUEST_BLOCK,
) -> UCHAR {
    let Cdb: PCDB = ((*Srb).Cdb).as_mut_ptr() as PCDB;

    if (*Cdb).START_STOP.LoadEject() != 0 {
        (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command = (IDE_COMMAND_MEDIA_EJECT) as u8;
        return (SRB_STATUS_PENDING) as u8;
    }

    return (SRB_STATUS_SUCCESS) as u8;
}

unsafe extern "C" fn AtaReqScsiRequestSense(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Request: PATA_DEVICE_REQUEST,
    Srb: PSCSI_REQUEST_BLOCK,
) -> UCHAR {
    let mut SenseCode: SCSI_SENSE_CODE = { let mut __lit: SCSI_SENSE_CODE = core::mem::zeroed(); __lit.__bindgen_anon_1.SrbStatus = (SRB_STATUS_SUCCESS) as u8; __lit.__bindgen_anon_1.SenseKey = (SCSI_SENSE_NO_SENSE) as u8; __lit.__bindgen_anon_1.AdditionalSenseCode = (SCSI_ADSENSE_NO_SENSE) as u8; __lit.__bindgen_anon_1.AdditionalSenseCodeQualifier = 0; __lit };
    return AtaReqSetFixedSenseData(Srb, SenseCode);
}

unsafe extern "C" fn AtaReqFilterPassThrough(mut TaskFile: PATA_TASKFILE) -> BOOLEAN {
    match ((*TaskFile).__bindgen_anon_1.Command as ::core::ffi::c_int) as u32 {
        IDE_COMMAND_SET_FEATURE => {
            if ((*TaskFile).__bindgen_anon_2.Feature as ::core::ffi::c_int) as u32 == IDE_FEATURE_SET_TRANSFER_MODE {
                return FALSE;
            }
        }
        IDE_COMMAND_SET_DRIVE_PARAMETERS | IDE_COMMAND_SET_MULTIPLE => {
            return FALSE;
        }
        _ => {}
    }
    return TRUE;
}

unsafe extern "C" fn AtaReqSetAtaStatusSenseData(
    mut Request: PATA_DEVICE_REQUEST,
    mut SK: UCHAR,
    mut ASK: UCHAR,
    mut ASCQ: UCHAR,
) {
    let mut TaskFile: PATA_TASKFILE = &raw mut (*Request).Output;
    let mut Srb: PSCSI_REQUEST_BLOCK = (*Request).Srb;
    let mut SenseData: PDESCRIPTOR_SENSE_DATA = ::core::ptr::null_mut::<_DESCRIPTOR_SENSE_DATA>();
    let mut BufferLength: ULONG = 0;
    let mut Descriptor: PSCSI_SENSE_DESCRIPTOR_ATA_STATUS_RETURN =
        ::core::ptr::null_mut::<_SCSI_SENSE_DESCRIPTOR_ATA_STATUS_RETURN>();
    (*Srb).ScsiStatus = SCSISTAT_CHECK_CONDITION as UCHAR;
    BufferLength = FIELD_OFFSET!(DESCRIPTOR_SENSE_DATA, DescriptorBuffer) as ULONG;
    BufferLength = (BufferLength as ::core::ffi::c_ulonglong).wrapping_add(::core::mem::size_of::<
        _SCSI_SENSE_DESCRIPTOR_ATA_STATUS_RETURN,
    >() as usize
        as ::core::ffi::c_ulonglong) as ULONG as ULONG;
    if ((*Srb).SenseInfoBufferLength as ULONG) < BufferLength {
        return;
    }
    SenseData = (*Srb).SenseInfoBuffer as PDESCRIPTOR_SENSE_DATA;
    RtlZeroMemory!(
        SenseData as *mut ::core::ffi::c_void,
        BufferLength as usize,
    );
    (*SenseData).set_ErrorCode((SCSI_SENSE_ERRORCODE_DESCRIPTOR_CURRENT as UCHAR) as _);
    (*SenseData).set_SenseKey((SK as UCHAR) as _);
    (*SenseData).AdditionalSenseCode = ASK;
    (*SenseData).AdditionalSenseCodeQualifier = ASCQ;
    (*SenseData).AdditionalSenseLength = (BufferLength as usize).wrapping_sub(
        RTL_SIZEOF_THROUGH_FIELD!(DESCRIPTOR_SENSE_DATA, AdditionalSenseLength) as usize,
    ) as UCHAR;
    Descriptor = (&raw mut (*SenseData).DescriptorBuffer as *mut UCHAR)
        .offset(0 as isize) as *mut UCHAR
        as PSCSI_SENSE_DESCRIPTOR_ATA_STATUS_RETURN;
    (*Descriptor).Header.DescriptorType = SCSI_SENSE_DESCRIPTOR_TYPE_ATA_STATUS_RETURN as UCHAR;
    (*Descriptor).Header.AdditionalLength = 0x0C as UCHAR;
    (*Descriptor).Error = (*TaskFile).__bindgen_anon_2.Error;
    (*Descriptor).Device = (*TaskFile).DriveSelect;
    (*Descriptor).Status = (*TaskFile).__bindgen_anon_1.Status;
    (*Descriptor).SectorCount7_0 = (*TaskFile).SectorCount;
    (*Descriptor).LbaLow7_0 = (*TaskFile).LowLba;
    (*Descriptor).LbaMid7_0 = (*TaskFile).MidLba;
    (*Descriptor).LbaHigh7_0 = (*TaskFile).HighLba;
    (*Descriptor).LbaLow15_8 =
        ((*TaskFile).DriveSelect as ::core::ffi::c_int & 0x0F as ::core::ffi::c_int) as UCHAR;
    if (*Request).Flags & REQUEST_FLAG_LBA48 as ULONG != 0 {
        (*Descriptor).set_Extend((1 as UCHAR) as _);
        (*Descriptor).SectorCount15_8 = (*TaskFile).SectorCountEx;
        (*Descriptor).LbaLow15_8 = (*TaskFile).LowLbaEx;
        (*Descriptor).LbaMid15_8 = (*TaskFile).MidLba;
        (*Descriptor).LbaHigh15_8 = (*TaskFile).HighLbaEx;
    }
    (*Request).SrbStatus =
        ((*Request).SrbStatus as ::core::ffi::c_int | SRB_STATUS_AUTOSENSE_VALID as ::core::ffi::c_int) as UCHAR;
}

unsafe extern "C" fn AtaReqCompleteAtaPassThrough(Request: PATA_DEVICE_REQUEST) -> ATA_COMPLETION_ACTION {
    let Cdb: PCDB = ((*(*Request).Srb).Cdb).as_mut_ptr() as PCDB;

    if (*Request).Flags & REQUEST_FLAG_HAS_TASK_FILE == 0 {
        return COMPLETE_IRP;
    }

    if (*Cdb).ATA_PASSTHROUGH16.CkCond() == 1
        || ((*Request).SrbStatus) as u32 != SRB_STATUS_SUCCESS
    {
        let mut SK: UCHAR = 0;
        let mut ASK: UCHAR = 0;
        let mut ASCQ: UCHAR = 0;

        if ((*Request).SrbStatus) as u32 != SRB_STATUS_SUCCESS {
            AtaReqTranslateFixedError(
                Request,
                &raw mut SK,
                &raw mut ASK,
                &raw mut ASCQ,
            );
        } else {
            SK = (SCSI_SENSE_RECOVERED_ERROR) as u8;
            ASK = (SCSI_ADSENSE_NO_SENSE) as u8;
            ASCQ = (SCSI_SENSEQ_ATA_PASS_THROUGH_INFORMATION_AVAILABLE) as u8;
        }

        AtaReqSetAtaStatusSenseData(
            Request,
            SK,
            ASK,
            ASCQ,
        );
    }

    return COMPLETE_IRP;
}

unsafe extern "C" fn AtaReqScsiAtaPassThrough(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Request: PATA_DEVICE_REQUEST,
    Srb: PSCSI_REQUEST_BLOCK,
) -> UCHAR {
    let TaskFile: PATA_TASKFILE = &raw mut (*Request).__bindgen_anon_2.TaskFile;
    let Cdb: PCDB;

    (*Request).Flags = REQUEST_FLAG_SET_DEVICE_REGISTER
        | REQUEST_FLAG_SAVE_TASK_FILE
        | REQUEST_FLAG_PASSTHROUGH
        | ((*Srb).SrbFlags & SRB_FLAGS_UNSPECIFIED_DIRECTION);

    Cdb = ((*Srb).Cdb).as_mut_ptr() as *mut UCHAR as PCDB;
    match ((*Cdb).ATA_PASSTHROUGH16.Protocol() as i32) as u32 {
        ATA_PASSTHROUGH_PROTOCOL_NON_DATA => {
            if (*Cdb).ATA_PASSTHROUGH16.TLength() != 0 {
                return AtaReqTerminateInvalidField(Srb);
            }
        }
        ATA_PASSTHROUGH_PROTOCOL_PIO_DATA_IN => {
            if (*Cdb).ATA_PASSTHROUGH16.TDir() == 0 {
                return AtaReqTerminateInvalidField(Srb);
            }
        }
        ATA_PASSTHROUGH_PROTOCOL_PIO_DATA_OUT => {
            if (*Cdb).ATA_PASSTHROUGH16.TDir() != 0 {
                return AtaReqTerminateInvalidField(Srb);
            }
        }
        ATA_PASSTHROUGH_PROTOCOL_DMA | ATA_PASSTHROUGH_PROTOCOL_UDMA_DATA_IN | ATA_PASSTHROUGH_PROTOCOL_UDMA_DATA_OUT => {
            if (*DevExt).Device.DeviceFlags & DEVICE_PIO_ONLY != 0 {
                return AtaReqTerminateInvalidField(Srb);
            }
            (*Request).Flags |= REQUEST_DMA_FLAGS;
        }
        ATA_PASSTHROUGH_PROTOCOL_NCQ => {
            if (*DevExt).Device.DeviceFlags & DEVICE_NCQ == 0 {
                return AtaReqTerminateInvalidField(Srb);
            }
            (*Request).Flags |= REQUEST_FLAG_NCQ | REQUEST_DMA_FLAGS;
        }
        _ => return AtaReqTerminateInvalidField(Srb),
    }

    if (*Cdb).ATA_PASSTHROUGH16.TLength() == 0
        && (*Srb).SrbFlags & SRB_FLAGS_UNSPECIFIED_DIRECTION != 0
    {
        return AtaReqTerminateInvalidField(Srb);
    }

    if ((*Srb).Cdb[0 as usize] as i32) as u32 == SCSIOP_ATA_PASSTHROUGH12 {
        (*TaskFile).__bindgen_anon_2.Feature = (*Cdb).ATA_PASSTHROUGH12.Features;
        (*TaskFile).SectorCount = (*Cdb).ATA_PASSTHROUGH12.SectorCount;
        (*TaskFile).LowLba = (*Cdb).ATA_PASSTHROUGH12.LbaLow;
        (*TaskFile).MidLba = (*Cdb).ATA_PASSTHROUGH12.LbaMid;
        (*TaskFile).HighLba = (*Cdb).ATA_PASSTHROUGH12.LbaHigh;
        (*TaskFile).DriveSelect = (*Cdb).ATA_PASSTHROUGH12.Device;
        (*TaskFile).__bindgen_anon_1.Command = (*Cdb).ATA_PASSTHROUGH12.Command;
    } else {
        (*TaskFile).__bindgen_anon_2.Feature = (*Cdb).ATA_PASSTHROUGH16.Features7_0;
        (*TaskFile).SectorCount = (*Cdb).ATA_PASSTHROUGH16.SectorCount7_0;
        (*TaskFile).LowLba = (*Cdb).ATA_PASSTHROUGH16.LbaLow7_0;
        (*TaskFile).MidLba = (*Cdb).ATA_PASSTHROUGH16.LbaMid7_0;
        (*TaskFile).HighLba = (*Cdb).ATA_PASSTHROUGH16.LbaHigh7_0;
        (*TaskFile).DriveSelect = (*Cdb).ATA_PASSTHROUGH16.Device;
        (*TaskFile).__bindgen_anon_1.Command = (*Cdb).ATA_PASSTHROUGH16.Command;

        if (*Cdb).ATA_PASSTHROUGH16.Extend() != 0 {
            (*TaskFile).FeatureEx = (*Cdb).ATA_PASSTHROUGH16.Features15_8;
            (*TaskFile).SectorCountEx = (*Cdb).ATA_PASSTHROUGH16.SectorCount15_8;
            (*TaskFile).LowLbaEx = (*Cdb).ATA_PASSTHROUGH16.LbaLow15_8;
            (*TaskFile).MidLbaEx = (*Cdb).ATA_PASSTHROUGH16.LbaMid15_8;
            (*TaskFile).HighLbaEx = (*Cdb).ATA_PASSTHROUGH16.LbaHigh15_8;

            (*Request).Flags |= REQUEST_FLAG_LBA48;
        }
    }

    if AtaReqFilterPassThrough(TaskFile) == 0 {
        return AtaReqTerminateInvalidField(Srb);
    }

    if (*Request).Flags & REQUEST_FLAG_NCQ != 0 {
        (*TaskFile).SectorCount = ((*TaskFile).SectorCount as i32 & !0xF8) as UCHAR;
    }

    (*TaskFile).DriveSelect = ((*TaskFile).DriveSelect as i32 & (!IDE_DRIVE_SELECT_SLAVE) as i32) as UCHAR;
    (*TaskFile).DriveSelect = ((*TaskFile).DriveSelect as i32
        | (*DevExt).Device.__bindgen_anon_1.DeviceSelect as i32 & (IDE_DRIVE_SELECT_SLAVE) as i32) as UCHAR;

    (*Request).Complete = Some(
        AtaReqCompleteAtaPassThrough
            as unsafe extern "C" fn(PATA_DEVICE_REQUEST) -> ATA_COMPLETION_ACTION,
    ) as PREQUEST_COMPLETION_ROUTINE;
    return (SRB_STATUS_PENDING) as u8;
}

unsafe extern "C" fn AtaReqExecuteScsiAta(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Request: PATA_DEVICE_REQUEST,
    Srb: PSCSI_REQUEST_BLOCK,
) -> UCHAR {
    match (*Srb).Cdb[0] as u32 {
        SCSIOP_REPORT_LUNS => return (AtaReqScsiReportLuns(DevExt, Request, Srb)) as u8,
        SCSIOP_INQUIRY => return AtaReqScsiInquiry(DevExt, Request, Srb),
        SCSIOP_TEST_UNIT_READY => return AtaReqScsiTestUnitReady(DevExt, Request, Srb),
        SCSIOP_MEDIUM_REMOVAL => return AtaReqScsiMediumRemoval(DevExt, Request, Srb),
        SCSIOP_START_STOP_UNIT => return AtaReqScsiStartStopUnit(DevExt, Request, Srb),
        SCSIOP_REQUEST_SENSE => return AtaReqScsiRequestSense(DevExt, Request, Srb),
        SCSIOP_MODE_SENSE | SCSIOP_MODE_SENSE10 => {
            return AtaReqScsiModeSense(
                DevExt,
                Request,
                Srb,
                ((*Srb).Cdb[0] as u32 == SCSIOP_MODE_SENSE) as BOOLEAN,
            );
        }
        SCSIOP_MODE_SELECT | SCSIOP_MODE_SELECT10 => {
            return AtaReqScsiModeSelect(
                DevExt,
                Request,
                Srb,
                ((*Srb).Cdb[0] as u32 == SCSIOP_MODE_SELECT) as BOOLEAN,
            );
        }
        SCSIOP_SERVICE_ACTION_IN16 => {
            let ServiceAction: UCHAR = (*(&raw mut (*Srb).Cdb as *mut UCHAR as PCDB))
                .READ_CAPACITY16
                .ServiceAction();
            if ServiceAction as u32 != SERVICE_ACTION_READ_CAPACITY16 {
                // fall through to default
            } else {
                return AtaReqScsiReadCapacity(DevExt, Request, Srb);
            }
        }
        SCSIOP_READ_CAPACITY => return AtaReqScsiReadCapacity(DevExt, Request, Srb),
        SCSIOP_READ6
        | SCSIOP_WRITE6
        | SCSIOP_READ
        | SCSIOP_WRITE
        | SCSIOP_READ12
        | SCSIOP_WRITE12
        | SCSIOP_READ16
        | SCSIOP_WRITE16 => return AtaReqScsiReadWrite(DevExt, Request, Srb),
        SCSIOP_SYNCHRONIZE_CACHE | SCSIOP_SYNCHRONIZE_CACHE16 => {
            return AtaReqScsiSynchronizeCache(DevExt, Request, Srb);
        }
        SCSIOP_VERIFY | SCSIOP_VERIFY12 | SCSIOP_VERIFY16 => {
            return AtaReqScsiVerify(DevExt, Request, Srb);
        }
        SCSIOP_ATA_PASSTHROUGH12 | SCSIOP_ATA_PASSTHROUGH16 => {
            return AtaReqScsiAtaPassThrough(DevExt, Request, Srb);
        }
        _ => {}
    }

    INFO!("Unknown command %02x\n", (*Srb).Cdb[0]);

    return AtaReqTerminateInvalidOpCode(Srb);
}

unsafe extern "C" fn AtaReqPreparePacketCommand(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Request: PATA_DEVICE_REQUEST,
    Srb: PSCSI_REQUEST_BLOCK,
) -> UCHAR {
    (*Request).Flags = REQUEST_FLAG_PACKET_COMMAND |
        (*Srb).SrbFlags & SRB_FLAGS_UNSPECIFIED_DIRECTION;

    if (*Srb).SrbFlags & SRB_FLAGS_UNSPECIFIED_DIRECTION != 0
        && AtaPacketCommandUseDma((*Srb).Cdb[0 as usize]) != 0
        && (*DevExt).Device.DeviceFlags & DEVICE_PIO_ONLY == 0
        && SRB_GET_FLAGS!(Srb) & (SRB_FLAG_PIO_RETRY) as ULONG_PTR == 0
    {
        (*Request).Flags |= REQUEST_DMA_FLAGS;
    }

    RtlCopyMemory!(
        (*Request).__bindgen_anon_2.Cdb.as_mut_ptr() as *mut core::ffi::c_void,
        (*Srb).Cdb.as_ptr() as *const core::ffi::c_void,
        RTL_FIELD_SIZE!(SCSI_REQUEST_BLOCK, Cdb),
    );

    return (SRB_STATUS_PENDING) as u8;
}

unsafe extern "C" fn AtaReqExecuteScsiAtapi(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Request: PATA_DEVICE_REQUEST,
    Srb: PSCSI_REQUEST_BLOCK,
) -> UCHAR {
    match (*Srb).Cdb[0 as usize] as u32 {
        SCSIOP_ATA_PASSTHROUGH12 | SCSIOP_ATA_PASSTHROUGH16 => {
            return AtaReqScsiAtaPassThrough(DevExt, Request, Srb);
        }
        SCSIOP_GET_CONFIGURATION => {
            if IsNEC_98 != 0 {
                return AtaReqTerminateInvalidOpCode(Srb);
            }
        }
        SCSIOP_INQUIRY => {
            if (*DevExt).Device.DeviceFlags & DEVICE_CACHE_INQUIRY != 0 {
                return AtaReqCopySatlBuffer(
                    Request,
                    &raw mut (*DevExt).InquiryData as PVOID,
                    core::mem::size_of::<INQUIRYDATA>() as ULONG,
                ) as UCHAR;
            }
        }
        _ => {}
    }
    return AtaReqPreparePacketCommand(DevExt, Request, Srb);
}
#[no_mangle]
pub unsafe extern "C" fn AtaReqExecuteScsi(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Request: PATA_DEVICE_REQUEST,
    Srb: PSCSI_REQUEST_BLOCK,
) -> UCHAR {
    if IS_ATAPI!(&(*DevExt).Device) != false {
        return AtaReqExecuteScsiAtapi(DevExt, Request, Srb);
    } else {
        return AtaReqExecuteScsiAta(DevExt, Request, Srb);
    }
}
