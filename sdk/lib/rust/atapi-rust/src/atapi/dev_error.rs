use crate::prelude::*;
use ntkrnl_sys::ddk::{ErrorCode, IO_ERROR_LOG_PACKET, IoAllocateErrorLogEntry, IoWriteErrorLogEntry, KeAcquireInterruptSpinLock, KeReleaseInterruptSpinLock, LARGE_INTEGER, PIO_ERROR_LOG_PACKET};
use core::ffi::c_void;

unsafe extern "C" fn AtaDeviceRequestSenseNeeded(Request: PATA_DEVICE_REQUEST) -> BOOLEAN {
    return (((*Request).Flags & REQUEST_FLAG_PACKET_COMMAND != 0
        && (*(*Request).Srb).SrbFlags & SRB_FLAGS_DISABLE_AUTOSENSE == 0
        && !(*(*Request).Srb).SenseInfoBuffer.is_null()
        && (*(*Request).Srb).SenseInfoBufferLength != 0) as i32) as u8;
}

unsafe extern "C" fn AtaDeviceRequestSenseNeededExt(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Request: PATA_DEVICE_REQUEST,
) -> BOOLEAN {
    return ((((*DevExt).Device.DeviceFlags & DEVICE_SENSE_DATA_REPORTING != 0)
        && ((*Request).Output.__bindgen_anon_1.Status & IDE_STATUS_INDEX as u8 != 0)
        && ((*(*Request).Srb).SrbFlags & SRB_FLAGS_DISABLE_AUTOSENSE == 0)
        && !(*(*Request).Srb).SenseInfoBuffer.is_null()
        && (*(*Request).Srb).SenseInfoBufferLength != 0) as i32) as u8;
}

unsafe extern "C" fn AtaDeviceFixedErrorNeeded(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Request: PATA_DEVICE_REQUEST,
) -> BOOLEAN {
    return ((!((*Request).Flags & REQUEST_FLAG_PACKET_COMMAND != 0)
        && !((*(*Request).Srb).SrbFlags & SRB_FLAGS_DISABLE_AUTOSENSE != 0)
        && !(*(*Request).Srb).SenseInfoBuffer.is_null()
        && (*(*Request).Srb).SenseInfoBufferLength != 0))
        as BOOLEAN;
}

unsafe extern "C" fn AtaDeviceIsDmaCrcError(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Request: PATA_DEVICE_REQUEST,
) -> BOOLEAN {
    return (((*Request).Output.__bindgen_anon_2.Error as u32 & IDE_ERROR_CRC_ERROR != 0)
        && ((*Request).Flags & REQUEST_FLAG_DMA != 0)) as BOOLEAN;
}

unsafe extern "C" fn AtaDeviceHandleRequestSense(
    PortData: PATAPORT_PORT_DATA,
    DevExt: PATAPORT_DEVICE_EXTENSION,
) {
    let Request: PATA_DEVICE_REQUEST = &raw mut (*PortData).Worker.InternalRequest;
    let FailedRequest: PATA_DEVICE_REQUEST = (*PortData).Worker.FailedRequest;
    let Srb: PSCSI_REQUEST_BLOCK = (*FailedRequest).Srb;
    let SenseData: PSENSE_DATA;
    let Success: BOOLEAN;

    if IS_ATAPI!(&(*DevExt).Device) != false {
        Success = ((((*Request).SrbStatus) as u32 == SRB_STATUS_SUCCESS)
            || (((*Request).SrbStatus) as u32 == SRB_STATUS_DATA_OVERRUN)) as u8;
    } else {
        Success = ((((*Request).SrbStatus) as u32 == SRB_STATUS_SUCCESS)
            && ((*Request).Flags & REQUEST_FLAG_HAS_TASK_FILE) != 0
            && ((*Request).__bindgen_anon_2.TaskFile.HighLba != 0)) as u8;
    }

    if Success == 0 {
        if IS_ATAPI!(&(*DevExt).Device) != false {
            (*Srb).ScsiStatus = (SCSISTAT_GOOD) as u8;
            (*FailedRequest).SrbStatus = (SRB_STATUS_REQUEST_SENSE_FAILED) as u8;
        } else {
            (*FailedRequest).SrbStatus = AtaReqSetFixedAtaSenseData(FailedRequest);
        }
        return;
    }

    SenseData = ((*Srb).SenseInfoBuffer).cast();

    if IS_ATAPI!(&(*DevExt).Device) != false {
        RtlCopyMemory!(
            (*Srb).SenseInfoBuffer as *mut core::ffi::c_void,
            (*DevExt).Device.LocalBuffer as *const core::ffi::c_void,
            min((*Srb).SenseInfoBufferLength, ATA_LOCAL_BUFFER_SIZE) as usize,
        );

        (*Srb).ScsiStatus = (SCSISTAT_CHECK_CONDITION) as u8;
        (*FailedRequest).SrbStatus = (SRB_STATUS_ERROR | SRB_STATUS_AUTOSENSE_VALID) as u8;
    } else {
        let mut SenseCode: SCSI_SENSE_CODE = core::mem::zeroed();

        SenseCode.SrbStatus = (SRB_STATUS_ERROR) as u8;
        SenseCode.SenseKey = (*Request).__bindgen_anon_2.TaskFile.HighLba;
        SenseCode.AdditionalSenseCode = (*Request).__bindgen_anon_2.TaskFile.MidLba;
        SenseCode.AdditionalSenseCodeQualifier = (*Request).__bindgen_anon_2.TaskFile.LowLba;
        (*FailedRequest).SrbStatus = AtaReqSetFixedSenseData(Srb, SenseCode);
    }

    if RTL_CONTAINS_FIELD(
        SenseData,
        (*Srb).SenseInfoBufferLength,
        AdditionalSenseCodeQualifier,
    ) != 0
    {
    }
}

unsafe extern "C" fn AtaDeviceSendRequestSense(
    PortData: PATAPORT_PORT_DATA,
    DevExt: PATAPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let Request: PATA_DEVICE_REQUEST = &raw mut (*PortData).Worker.InternalRequest;
    let FailedRequest: PATA_DEVICE_REQUEST = (*PortData).Worker.FailedRequest;
    let Status: NTSTATUS;
    let Cdb: PCDB;

    ASSERT_REQUEST!(!FailedRequest.is_null());

    (*Request).Flags = REQUEST_FLAG_DATA_IN |
                       REQUEST_FLAG_PACKET_COMMAND |
                       REQUEST_FLAG_HAS_LOCAL_BUFFER;
    (*Request).TimeOut = 3;
    (*Request).DataTransferLength = ((*(*FailedRequest).Srb).SenseInfoBufferLength) as _;

    Cdb = &raw mut (*Request).__bindgen_anon_2.Cdb as *mut UCHAR as PCDB;
    (*Cdb).CDB6INQUIRY.OperationCode = (SCSIOP_REQUEST_SENSE) as u8;
    (*Cdb).CDB6INQUIRY._bitfield_1.set_LogicalUnitNumber(0);
    (*Cdb).CDB6INQUIRY._bitfield_1.set_Reserved1(0);
    (*Cdb).CDB6INQUIRY.PageCode = 0;
    (*Cdb).CDB6INQUIRY.IReserved = 0;
    (*Cdb).CDB6INQUIRY.AllocationLength = (*Request).DataTransferLength as UCHAR;
    (*Cdb).CDB6INQUIRY.Control = 0;

    Status = AtaPortSendRequest(PortData, DevExt);

    AtaDeviceHandleRequestSense(PortData, DevExt);
    return Status;
}

unsafe extern "C" fn AtaDeviceSendRequestSenseExt(
    PortData: PATAPORT_PORT_DATA,
    DevExt: PATAPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let Request: PATA_DEVICE_REQUEST = &raw mut (*PortData).Worker.InternalRequest;
    let mut Status: NTSTATUS;

    (*Request).Flags = REQUEST_FLAG_SAVE_TASK_FILE | REQUEST_FLAG_LBA48;
    (*Request).TimeOut = 3;

    RtlZeroMemory!(
        &raw mut (*Request).__bindgen_anon_2.TaskFile as *mut core::ffi::c_void,
        core::mem::size_of::<ATA_TASKFILE>(),
    );
    (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command = (IDE_COMMAND_REQUEST_SENSE_DATA_EXT) as u8;

    Status = AtaPortSendRequest(PortData, DevExt);

    AtaDeviceHandleRequestSense(PortData, DevExt);
    return Status;
}

unsafe extern "C" fn AtaDeviceSendReadNcqCommandErrorLog(
    PortData: PATAPORT_PORT_DATA,
    DevExt: PATAPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let Request: PATA_DEVICE_REQUEST = &raw mut (*PortData).Worker.InternalRequest;

    AtaReqBuildReadLogTaskFile(
        Request,
        (IDE_GP_LOG_NCQ_COMMAND_ERROR_ADDRESS) as UCHAR,
        0,
        1,
    );
    (*Request).Flags |= REQUEST_FLAG_HAS_LOCAL_BUFFER;
    (*Request).TimeOut = 3;

    return AtaPortSendRequest(PortData, DevExt);
}

unsafe extern "C" fn AtaDeviceSaveLogPageTaskFile(
    LogPage: *mut GP_LOG_NCQ_COMMAND_ERROR,
    Request: *mut ATA_DEVICE_REQUEST,
) {
    let TaskFile: PATA_TASKFILE = &raw mut (*Request).Output;

    (*TaskFile).SectorCount = (*LogPage).Count7_0;
    (*TaskFile).LowLba = (*LogPage).LBA7_0;
    (*TaskFile).MidLba = (*LogPage).LBA15_8;
    (*TaskFile).HighLba = (*LogPage).LBA23_16;
    (*TaskFile).DriveSelect = (*LogPage).Device;

    if (*Request).Flags & REQUEST_FLAG_LBA48 != 0 {
        (*TaskFile).FeatureEx = 0;
        (*TaskFile).SectorCountEx = (*LogPage).Count15_8;
        (*TaskFile).LowLbaEx = (*LogPage).LBA31_24;
        (*TaskFile).MidLbaEx = (*LogPage).LBA39_32;
        (*TaskFile).HighLbaEx = (*LogPage).LBA47_40;
    }
}

unsafe extern "C" fn AtaDeviceLogEvent(
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
    mut ErrorValue: ATA_ERROR_LOG_VALUE,
) {
    let mut ChanExt: PATAPORT_CHANNEL_EXTENSION = ((*DevExt).Common.__bindgen_anon_1.FdoExt).cast();
    let mut ErrorCode: ULONG = 0;
    let mut FinalStatus: ULONG = 0;
    let mut LogEntry: PIO_ERROR_LOG_PACKET = core::ptr::null_mut();
    let mut Size: UCHAR = 0;

    Size = FIELD_OFFSET!(IO_ERROR_LOG_PACKET, DumpData) as UCHAR;
    LogEntry = (IoAllocateErrorLogEntry(((*ChanExt).Common.Self_).cast(), Size)).cast();
    if LogEntry.is_null() {
        return;
    }

    match ErrorValue {
        EVENT_CODE_TIMEOUT => {
            ErrorCode = (IO_ERR_TIMEOUT) as u32;
            FinalStatus = (STATUS_IO_TIMEOUT) as u32;
        }
        EVENT_CODE_CRC_ERROR => {
            ErrorCode = (IO_ERR_PARITY) as u32;
            FinalStatus = (STATUS_IO_DEVICE_ERROR) as u32;
        }
        _ => {
            ErrorCode = (IO_ERR_CONTROLLER_ERROR) as u32;
            FinalStatus = (STATUS_IO_DEVICE_ERROR) as u32;
        }
    }

    RtlZeroMemory!(LogEntry, Size as usize);
    (*LogEntry).FinalStatus = (FinalStatus) as i32;
    (*LogEntry).ErrorCode = (ErrorCode) as i32;
    (*LogEntry).MajorFunctionCode = (IRP_MJ_SCSI) as u8;
    (*LogEntry).UniqueErrorValue = ErrorValue;

    IoWriteErrorLogEntry((LogEntry).cast());
}

unsafe extern "C" fn AtaDeviceDowngradeTransferSpeed(
    PortData: PATAPORT_PORT_DATA,
    DevExt: PATAPORT_DEVICE_EXTENSION,
) {
    let mut Mode: ULONG = 0;
    let mut AllowedModesMask: ULONG;

    if ((_BitScanReverse(
        &mut Mode,
        (*DevExt).TransferModeSelectedBitmap & !PIO_ALL,
    )) == 0) {
        return;
    }

    AllowedModesMask = ((1 << Mode) - 1);

    if ((*DevExt).TransferModeSupportedBitmap & UDMA_ALL) != 0
        && (AllowedModesMask & UDMA_ALL) == 0
    {
        if ((*PortData).PortFlags & PORT_FLAG_PIO_VIA_DMA) != 0
            && !IS_ATAPI!(&(*DevExt).Device)
        {
            return;
        }
        AllowedModesMask &= PIO_ALL;
    } else if (((*DevExt).TransferModeSupportedBitmap & MWDMA_ALL) != 0
        && (AllowedModesMask & MWDMA_ALL) == 0)
        || ((*DevExt).TransferModeSupportedBitmap & SWDMA_ALL) != 0
            && (AllowedModesMask & SWDMA_ALL) == 0
    {
        AllowedModesMask &= PIO_ALL;
    }

    if ((*DevExt).TransferModeSupportedBitmap & !PIO_ALL) != 0
        && (AllowedModesMask & !PIO_ALL) == 0
    {
        AtaDeviceLogEvent(DevExt, EVENT_CODE_DMA_DISABLE);
    }

    (*DevExt).TransferModeAllowedMask &= AllowedModesMask;

    _InterlockedOr(
        &mut (*PortData).Worker.EventsPending,
        (ACTION_PORT_TIMING | ACTION_DEVICE_CONFIG) as i32,
    );
    _InterlockedOr(
        &mut (*DevExt).Worker.EventsPending,
        (ACTION_DEVICE_CONFIG) as i32,
    );

    (*PortData).Worker.Flags |= WORKER_FLAG_NEED_RESCAN;
}

unsafe extern "C" fn AtaDeviceAnalyzeDmaError(
    mut PortData: PATAPORT_PORT_DATA,
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
    mut Request: PATA_DEVICE_REQUEST,
    mut ErrorValue: ATA_ERROR_LOG_VALUE,
) {
    let mut CurrentTime: LARGE_INTEGER = core::mem::zeroed();
    let mut TimeDifferenceMs: LARGE_INTEGER = core::mem::zeroed();

    if (*Request).Flags & REQUEST_FLAG_PASSTHROUGH != 0 {
        return;
    }

    KeQuerySystemTime(&mut CurrentTime);
    TimeDifferenceMs.QuadPart = (CurrentTime.QuadPart - (*DevExt).LastDmaErrorTime.QuadPart) / 10000;
    (*DevExt).LastDmaErrorTime.QuadPart = CurrentTime.QuadPart;

    AtaDeviceLogEvent(DevExt, ErrorValue);

    if TimeDifferenceMs.QuadPart >= (10 * 60000) {
        return;
    }

    if (*DevExt).Device.DeviceFlags & DEVICE_NCQ != 0 {
        (*DevExt).Device.DeviceFlags &= !DEVICE_NCQ;
        AtaDeviceLogEvent(DevExt, EVENT_CODE_NCQ_DISABLE);
        return;
    }

    if (*PortData).DowngradeInterfaceSpeed((*PortData).ChannelContext) != 0 {
        AtaDeviceLogEvent(DevExt, EVENT_CODE_DOWNSHIFT);
        return;
    }

    AtaDeviceDowngradeTransferSpeed(PortData, DevExt);
}

unsafe extern "C" fn AtaDeviceCompleteFailedRequest(PortData: PATAPORT_PORT_DATA) {
    let Request: PATA_DEVICE_REQUEST = (*PortData).Worker.FailedRequest;
    let mut SrbStatus: UCHAR;
    let mut RetryCount: SIZE_T;
    let mut OldIrql: KIRQL;

    ASSERT_REQUEST!(Request);

    SrbStatus = SRB_STATUS((*Request).SrbStatus);

    RetryCount = SRB_GET_FLAGS!((*Request).Srb) & (SRB_FLAG_RETRY_COUNT_MASK) as _;
    RetryCount = RetryCount.wrapping_add(1);
    SRB_CLEAR_FLAGS!((*Request).Srb, SRB_FLAG_RETRY_COUNT_MASK);
    SRB_SET_FLAGS!((*Request).Srb, RetryCount);

    if RetryCount > 3 {
    } else {
        match (SrbStatus) as u32 {
            SRB_STATUS_BUS_RESET | SRB_STATUS_TIMEOUT => {
                if (*PortData).PortFlags & PORT_FLAG_PIO_VIA_DMA == 0 {
                    SRB_SET_FLAGS!((*Request).Srb, (SRB_FLAG_PIO_RETRY) as _);
                }
                SrbStatus = (SRB_STATUS_BUSY) as u8;
            }
            SRB_STATUS_REQUEST_SENSE_FAILED => {
                SrbStatus = (SRB_STATUS_BUSY) as u8;
            }
            _ => {
                if (*(*Request).Irp).Flags & (IRP_SYNCHRONOUS_PAGING_IO | IRP_PAGING_IO) != 0 {
                    SrbStatus = (SRB_STATUS_BUSY) as u8;
                }
            }
        }
    }

    SrbStatus = SrbStatus | ((*Request).SrbStatus & ((SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as u8);
    (*Request).SrbStatus = SrbStatus;

    if SRB_STATUS(SrbStatus) == SRB_STATUS_BUSY {
        return;
    }

    (*Request).__bindgen_anon_3.__bindgen_anon_1.InternalState = (REQUEST_STATE_FREEZE_QUEUE) as u8;

    OldIrql = KeAcquireInterruptSpinLock((*PortData).InterruptObject);

    (*PortData).Worker.PausedSlotsBitmap &= !(1 << (*Request).Slot);

    KeReleaseInterruptSpinLock((*PortData).InterruptObject, OldIrql);
}

unsafe extern "C" fn AtaDeviceNcqRecovery(
    mut PortData: PATAPORT_PORT_DATA,
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut FailedRequest: PATA_DEVICE_REQUEST = core::ptr::null_mut();
    let mut LogPage: PGP_LOG_NCQ_COMMAND_ERROR = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut i: ULONG = 0;
    let mut Crc: UCHAR = 0;

    'failure: {
        Status = AtaDeviceSendReadNcqCommandErrorLog(PortData, DevExt);
        if Status == STATUS_ADAPTER_HARDWARE_ERROR {
            return Status;
        }
        if !NT_SUCCESS!(Status) {
            break 'failure;
        }

        LogPage = (*DevExt).Device.LocalBuffer as *mut GP_LOG_NCQ_COMMAND_ERROR;

        if (*LogPage).NonQueuedCmd() != 0 {
            break 'failure;
        }

        Crc = 0;
        i = 0;
        while i < IDE_GP_LOG_SECTOR_SIZE {
            Crc = Crc.wrapping_add(*((LogPage as *const u8).add(i as usize)));
            i = i.wrapping_add(1);
        }
        if Crc != 0 {
            break 'failure;
        }

        if ((*PortData).Worker.PausedSlotsBitmap
            & ((1 as u32) << ((*LogPage).NcqTag() as u32)))
            == 0
        {
            break 'failure;
        }

        FailedRequest = (*PortData).Slots[(*LogPage).NcqTag() as usize];
        ASSERT_REQUEST!(!FailedRequest.is_null());
        (*PortData).Worker.FailedRequest = FailedRequest;

        (*FailedRequest).SrbStatus = (SRB_STATUS_ERROR) as u8;
        (*FailedRequest).Output.__bindgen_anon_1.Status = (*LogPage).Status;
        (*FailedRequest).Output.__bindgen_anon_2.Error = (*LogPage).Error;
        if ((*FailedRequest).Flags & (REQUEST_FLAG_SAVE_TASK_FILE as u32)) != 0 {
            AtaDeviceSaveLogPageTaskFile(LogPage, FailedRequest);
        }

        if ((*LogPage).SenseKey != 0)
            && AtaDevHasNcqAutosense(core::ptr::addr_of_mut!((*DevExt).__bindgen_anon_1.IdentifyDeviceData)) != 0
        {
            let mut SenseCode: SCSI_SENSE_CODE = core::mem::zeroed();
            SenseCode.SrbStatus = (*FailedRequest).SrbStatus;
            SenseCode.SenseKey = (*LogPage).SenseKey;
            SenseCode.AdditionalSenseCode = (*LogPage).ASC;
            SenseCode.AdditionalSenseCodeQualifier = (*LogPage).ASCQ;
            (*FailedRequest).SrbStatus = AtaReqSetFixedSenseData((*FailedRequest).Srb, SenseCode);

            AtaReqSetLbaInformation(
                (*FailedRequest).Srb,
                ((*((LogPage as *const u8).add(17 as usize)) as u64)
                    << (0 as u32))
                    | ((*((LogPage as *const u8).add(18 as usize)) as u64)
                        << (8 as u32))
                    | ((*((LogPage as *const u8).add(19 as usize)) as u64)
                        << (16 as u32))
                    | ((*((LogPage as *const u8).add(20 as usize)) as u64)
                        << (24 as u32))
                    | ((*((LogPage as *const u8).add(21 as usize)) as u64)
                        << (32 as u32))
                    | ((*((LogPage as *const u8).add(22 as usize)) as u64)
                        << (48 as u32)),
            );
        } else if AtaDeviceRequestSenseNeededExt(DevExt, FailedRequest) != 0 {
            Status = AtaDeviceSendRequestSenseExt(PortData, DevExt);
            if Status == STATUS_ADAPTER_HARDWARE_ERROR {
                return Status;
            }
        } else {
            (*FailedRequest).SrbStatus = AtaReqSetFixedAtaSenseData(FailedRequest);
        }

        AtaDeviceCompleteFailedRequest(PortData);
        return STATUS_SUCCESS;
    }

    _InterlockedOr(
        core::ptr::addr_of_mut!((*PortData).Worker.EventsPending),
        (ACTION_PORT_RESET) as i32,
    );
    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaDeviceGenericRecovery(
    mut PortData: PATAPORT_PORT_DATA,
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut Request: PATA_DEVICE_REQUEST = (*PortData).Worker.FailedRequest;
    let mut Status: NTSTATUS = 0;
    match ((*Request).SrbStatus as ::core::ffi::c_int) as u32 {
        SRB_STATUS_BUS_RESET | SRB_STATUS_TIMEOUT => {
            if (*Request).Flags & REQUEST_FLAG_DMA != 0 {
                let mut ErrorValue: ATA_ERROR_LOG_VALUE = 0 as ATA_ERROR_LOG_VALUE;
                if ((*Request).SrbStatus as ::core::ffi::c_int) as u32 == SRB_STATUS_TIMEOUT {
                    ErrorValue = EVENT_CODE_TIMEOUT;
                } else {
                    ErrorValue = EVENT_CODE_BAD_STATE;
                }
                AtaDeviceAnalyzeDmaError(PortData, DevExt, Request, ErrorValue);
            }
        }
        SRB_STATUS_ERROR => {
            if AtaDeviceIsDmaCrcError(DevExt, Request) != 0 {
                AtaDeviceAnalyzeDmaError(PortData, DevExt, Request, EVENT_CODE_CRC_ERROR);
            } else if AtaDeviceRequestSenseNeeded(Request) != 0 {
                Status = AtaDeviceSendRequestSense(PortData, DevExt);
            } else if AtaDeviceRequestSenseNeededExt(DevExt, Request) != 0 {
                Status = AtaDeviceSendRequestSenseExt(PortData, DevExt);
            } else if AtaDeviceFixedErrorNeeded(DevExt, Request) != 0 {
                (*Request).SrbStatus = AtaReqSetFixedAtaSenseData(Request);
            }
        }
        _ => {}
    }
    if Status == STATUS_ADAPTER_HARDWARE_ERROR {
        return Status;
    }
    AtaDeviceCompleteFailedRequest(PortData);
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn AtaPortDeviceProcessError(
    PortData: *mut ATAPORT_PORT_DATA,
    DevExt: *mut ATAPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let FailedRequest: *mut ATA_DEVICE_REQUEST = (*PortData).Worker.FailedRequest;
    let Status: NTSTATUS;

    ASSERT_REQUEST!(!FailedRequest.is_null());

    if (*FailedRequest).Flags & REQUEST_FLAG_NCQ != 0 {
        Status = AtaDeviceNcqRecovery(PortData, DevExt);
    } else {
        Status = AtaDeviceGenericRecovery(PortData, DevExt);
    }

    return Status;
}
