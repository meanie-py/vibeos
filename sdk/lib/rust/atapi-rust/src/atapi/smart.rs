use crate::prelude::*;
use ntkrnl_sys::ddk::{IDEREGS, PGETVERSIONINPARAMS, PIDEREGS, PSENDCMDINPARAMS, PSENDCMDOUTPARAMS};
use core::ffi::c_void;

unsafe extern "C" fn AtaIdeRegsToTaskFile(
    DevExt: *mut ATAPORT_DEVICE_EXTENSION,
    IdeRegs: *mut IDEREGS,
    Request: *mut ATA_DEVICE_REQUEST,
) {
    let TaskFile: PATA_TASKFILE = &raw mut (*Request).__bindgen_anon_2.TaskFile;

    (*TaskFile).__bindgen_anon_2.Feature = (*IdeRegs).bFeaturesReg;
    (*TaskFile).SectorCount = (*IdeRegs).bSectorCountReg;
    (*TaskFile).LowLba = (*IdeRegs).bSectorNumberReg;
    (*TaskFile).MidLba = (*IdeRegs).bCylLowReg;
    (*TaskFile).HighLba = (*IdeRegs).bCylHighReg;
    (*TaskFile).__bindgen_anon_1.Command = (IDE_COMMAND_SMART) as u8;

    (*TaskFile).DriveSelect = (*IdeRegs).bDriveHeadReg & (!IDE_DRIVE_SELECT_SLAVE) as u8;
    (*TaskFile).DriveSelect |= (*DevExt).Device.__bindgen_anon_1.DeviceSelect & (IDE_DRIVE_SELECT_SLAVE) as u8;
}

unsafe extern "C" fn AtaTaskFileToIdeRegs(
    Request: *mut ATA_DEVICE_REQUEST,
    IdeRegs: *mut IDEREGS,
) {
    let TaskFile: PATA_TASKFILE = &raw mut (*Request).Output;
    (*IdeRegs).bFeaturesReg = (*TaskFile).__bindgen_anon_2.Feature;
    (*IdeRegs).bSectorCountReg = (*TaskFile).SectorCount;
    (*IdeRegs).bSectorNumberReg = (*TaskFile).LowLba;
    (*IdeRegs).bCylLowReg = (*TaskFile).MidLba;
    (*IdeRegs).bCylHighReg = (*TaskFile).HighLba;
    (*IdeRegs).bCommandReg = (*TaskFile).__bindgen_anon_1.Command;
    (*IdeRegs).bDriveHeadReg = (*TaskFile).DriveSelect;
}

unsafe extern "C" fn AtaReqCompleteSmartIoControl(
    mut Request: PATA_DEVICE_REQUEST,
) -> ATA_COMPLETION_ACTION {
    let mut CmdOut: PSENDCMDOUTPARAMS = core::ptr::null_mut();
    CmdOut = ((*(*Request).Srb).DataBuffer as *mut u8)
        .offset(core::mem::size_of::<SRB_IO_CONTROL>() as isize)
        as PSENDCMDOUTPARAMS;
    if ((*Request).SrbStatus) as u32 == SRB_STATUS_SUCCESS {
        (*CmdOut).DriverStatus.bDriverError = (SMART_NO_ERROR) as u8;
        (*CmdOut).DriverStatus.bIDEError = 0;
    } else {
        (*CmdOut).DriverStatus.bDriverError = (SMART_IDE_ERROR) as u8;
        (*CmdOut).DriverStatus.bIDEError = (*Request).Output.__bindgen_anon_2.Error;
    }
    if (*Request).Flags & REQUEST_FLAG_SAVE_TASK_FILE != 0 {
        if (*Request).Flags & REQUEST_FLAG_HAS_TASK_FILE != 0 {
            let mut IdeRegs: PIDEREGS = &raw mut (*CmdOut).bBuffer as PIDEREGS;
            AtaTaskFileToIdeRegs(Request, IdeRegs);
        } else {
            (*CmdOut).DriverStatus.bDriverError = (SMART_IDE_ERROR) as u8;
            (*CmdOut).DriverStatus.bIDEError = (*Request).Output.__bindgen_anon_2.Error;
        }
    }
    return COMPLETE_IRP;
}
#[no_mangle]
pub unsafe extern "C" fn AtaReqSmartIoControl(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Request: PATA_DEVICE_REQUEST,
    Srb: PSCSI_REQUEST_BLOCK,
) -> UCHAR {
    union _PARAMS {
        CmdIn: PSENDCMDINPARAMS,
        CmdOut: PSENDCMDOUTPARAMS,
    }

    let mut Buffer: _PARAMS = _PARAMS {
        CmdIn: core::ptr::null_mut(),
    };

    if (*Srb).DataTransferLength
        < (core::mem::size_of::<SRB_IO_CONTROL>() + core::mem::size_of::<*mut SENDCMDINPARAMS>() - 1) as u32
    {
        return (SRB_STATUS_INVALID_REQUEST) as u8;
    }

    Buffer.CmdIn = ((*Srb).DataBuffer as *mut u8).add(core::mem::size_of::<SRB_IO_CONTROL>()) as PSENDCMDINPARAMS;

    if ((*Buffer.CmdIn).irDriveRegs.bCommandReg) as u32 != SMART_CMD {
        return (SRB_STATUS_INVALID_REQUEST) as u8;
    }

    match ((*Buffer.CmdIn).irDriveRegs.bFeaturesReg) as u32 {
        READ_ATTRIBUTES => {
            (*Request).DataTransferLength = READ_ATTRIBUTE_BUFFER_SIZE;
            (*Request).Flags = REQUEST_FLAG_DATA_IN;
        }
        READ_THRESHOLDS => {
            (*Request).DataTransferLength = READ_THRESHOLD_BUFFER_SIZE;
            (*Request).Flags = REQUEST_FLAG_DATA_IN;
        }
        SMART_READ_LOG => {
            (*Request).DataTransferLength = (*Buffer.CmdIn).irDriveRegs.bSectorCountReg as u32 * SMART_LOG_SECTOR_SIZE;
            (*Request).Flags = REQUEST_FLAG_DATA_IN;
        }
        SMART_WRITE_LOG => {
            (*Request).DataTransferLength = (*Buffer.CmdIn).irDriveRegs.bSectorCountReg as u32 * SMART_LOG_SECTOR_SIZE;
            (*Request).Flags = REQUEST_FLAG_DATA_OUT;
        }
        RETURN_SMART_STATUS => {
            (*Request).DataTransferLength = core::mem::size_of::<IDEREGS>() as u32;
            (*Request).Flags = REQUEST_FLAG_SAVE_TASK_FILE;
        }
        EXECUTE_OFFLINE_DIAGS => {
            let Subcommand: UCHAR = (*Buffer.CmdIn).irDriveRegs.bSectorNumberReg;
            if (Subcommand) as u32 == SMART_SHORT_SELFTEST_CAPTIVE || (Subcommand) as u32 == SMART_EXTENDED_SELFTEST_CAPTIVE {
                return (SRB_STATUS_INVALID_REQUEST) as u8;
            }
            (*Request).DataTransferLength = 0;
            (*Request).Flags = 0;
        }
        ENABLE_DISABLE_AUTOSAVE
        | SAVE_ATTRIBUTE_VALUES
        | ENABLE_SMART
        | DISABLE_SMART
        | ENABLE_DISABLE_AUTO_OFFLINE => {
            (*Request).DataTransferLength = 0;
            (*Request).Flags = 0;
        }
        _ => return (SRB_STATUS_INVALID_REQUEST) as u8,
    }

    if (*Request).Flags & (REQUEST_FLAG_DATA_IN | REQUEST_FLAG_SAVE_TASK_FILE) != 0 {
        if (*Srb).DataTransferLength
            < (core::mem::size_of::<SRB_IO_CONTROL>() + core::mem::size_of::<*mut SENDCMDOUTPARAMS>() - 1 + (*Request).DataTransferLength as usize) as u32
        {
            return (SRB_STATUS_INVALID_REQUEST) as u8;
        }
        (*Request).DataBuffer = (*Buffer.CmdOut).bBuffer.as_mut_ptr() as *mut core::ffi::c_void;
    } else if (*Request).Flags & REQUEST_FLAG_DATA_OUT != 0 {
        if (*Srb).DataTransferLength
            < (core::mem::size_of::<SRB_IO_CONTROL>() + core::mem::size_of::<*mut SENDCMDINPARAMS>() - 1 + (*Request).DataTransferLength as usize) as u32
        {
            return (SRB_STATUS_INVALID_REQUEST) as u8;
        }
        (*Request).DataBuffer = (*Buffer.CmdIn).bBuffer.as_mut_ptr() as *mut core::ffi::c_void;
    }

    (*Request).Complete = Some(AtaReqCompleteSmartIoControl);

    AtaIdeRegsToTaskFile(DevExt, &mut (*Buffer.CmdIn).irDriveRegs, Request);

    return (SRB_STATUS_PENDING) as u8;
}
#[no_mangle]
pub unsafe extern "C" fn AtaPdoHandleMiniportIdentify(
    DevExt: *mut ATAPORT_DEVICE_EXTENSION,
    Srb: *mut SCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    union _PARAMS {
        CmdIn: PSENDCMDINPARAMS,
        CmdOut: PSENDCMDOUTPARAMS,
    }
    let mut Buffer: _PARAMS = core::mem::zeroed();

    PAGED_CODE!();

    if (*Srb).DataTransferLength
        < ((core::mem::size_of::<SRB_IO_CONTROL>()
            + core::mem::size_of::<SENDCMDOUTPARAMS>()
            - 1
            + (IDENTIFY_BUFFER_SIZE) as usize)) as u32
    {
        (*Srb).SrbStatus = (SRB_STATUS_INVALID_REQUEST) as u8;
        return STATUS_BUFFER_TOO_SMALL;
    }

    Buffer.CmdIn = ((*Srb).DataBuffer as *mut u8)
        .add(core::mem::size_of::<SRB_IO_CONTROL>()) as PSENDCMDINPARAMS;

    if IS_ATAPI!(&(*DevExt).Device)
        || ((*Buffer.CmdIn).irDriveRegs.bCommandReg) as u32 != ID_CMD
    {
        (*Srb).SrbStatus = (SRB_STATUS_INVALID_REQUEST) as u8;
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    (*Buffer.CmdOut).cBufferSize = IDENTIFY_BUFFER_SIZE;
    (*Buffer.CmdOut).DriverStatus.bDriverError = 0;
    (*Buffer.CmdOut).DriverStatus.bIDEError = 0;

    RtlCopyMemory!(
        (*Buffer.CmdOut).bBuffer.as_mut_ptr() as *mut core::ffi::c_void,
        &(*DevExt).__bindgen_anon_1.IdentifyDeviceData as *const _ as *const core::ffi::c_void,
        IDENTIFY_BUFFER_SIZE,
    );

    (*Srb).SrbStatus = (SRB_STATUS_SUCCESS) as u8;
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn AtaPdoHandleMiniportSmartVersion(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Srb: PSCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    let ChanExt: PATAPORT_CHANNEL_EXTENSION = ((*DevExt).Common.__bindgen_anon_1.FdoExt).cast();
    let PortData: PATAPORT_PORT_DATA = &raw mut (*ChanExt).PortData;
    let mut VersionParameters: PGETVERSIONINPARAMS;
    let mut i: ULONG;

    PAGED_CODE!();

    if (*Srb).DataTransferLength < ((core::mem::size_of::<GETVERSIONINPARAMS>() + core::mem::size_of::<SRB_IO_CONTROL>())) as _ {
        (*Srb).SrbStatus = (SRB_STATUS_INVALID_REQUEST) as u8;
        return STATUS_BUFFER_TOO_SMALL;
    }

    VersionParameters = ((*Srb).DataBuffer as *mut u8).add(core::mem::size_of::<SRB_IO_CONTROL>()) as PGETVERSIONINPARAMS;

    (*VersionParameters).bVersion = 1;
    (*VersionParameters).bRevision = 1;
    (*VersionParameters).bReserved = 0;

    (*VersionParameters).fCapabilities = (CAP_ATA_ID_CMD | CAP_ATAPI_ID_CMD | CAP_SMART_CMD);

    (*VersionParameters).bIDEDeviceMap = 0;

    i = 0;
    while i < MAX_IDE_DEVICE {
        let mut DevExtInner: PATAPORT_DEVICE_EXTENSION;
        let mut DeviceFlag: ULONG;

        DevExtInner = (AtaFdoFindDeviceByPath(
            (ChanExt).cast(),
            AtaMarshallScsiAddress((*PortData).PortNumber, i, 0),
            (Srb).cast(),
        )).cast();
        if DevExtInner.is_null() {
            i = i.wrapping_add(1);
            continue;
        }

        DeviceFlag = 1 << i;
        if (*PortData).PortNumber != 0 {
            DeviceFlag <<= 2;
        }
        if IS_ATAPI!(&raw mut (*DevExtInner).Device) {
            DeviceFlag <<= 4;
        }
        (*VersionParameters).bIDEDeviceMap |= DeviceFlag as u8;

        IoReleaseRemoveLock(&raw mut (*DevExtInner).Common.RemoveLock, (Srb).cast());

        i = i.wrapping_add(1);
    }

    (*Srb).SrbStatus = (SRB_STATUS_SUCCESS) as u8;
    return STATUS_SUCCESS;
}
