use crate::prelude::*;
use ntkrnl_sys::ddk::PCSTR;
use core::ffi::c_void;

unsafe extern "C" fn AtaDeviceSetGeometry(
    mut PortData: PATAPORT_PORT_DATA,
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut Request: PATA_DEVICE_REQUEST = &raw mut (*PortData).Worker.InternalRequest;
    let mut Status: NTSTATUS = 0;
    if IS_ATAPI!(&raw mut (*DevExt).Device) != false
        || (*DevExt).Device.DeviceFlags & DEVICE_LBA_MODE != 0
    {
        return STATUS_SUCCESS;
    }
    (*Request).Flags = REQUEST_FLAG_SET_DEVICE_REGISTER;
    (*Request).TimeOut = 5;
    RtlZeroMemory!(
        &raw mut (*Request).__bindgen_anon_2.TaskFile as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<ATA_TASKFILE>(),
    );
    (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command = (IDE_COMMAND_SET_DRIVE_PARAMETERS) as u8;
    (*Request).__bindgen_anon_2.TaskFile.SectorCount = (*DevExt).Device.SectorsPerTrack as UCHAR;
    (*Request).__bindgen_anon_2.TaskFile.DriveSelect =
        (((*DevExt).Device.Heads as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
            as UCHAR as ::core::ffi::c_int
            | (*DevExt).Device.__bindgen_anon_1.DeviceSelect as ::core::ffi::c_int) as UCHAR;
    Status = AtaPortSendRequest(PortData, DevExt);
    if !NT_SUCCESS!(Status) {
        // Diverges from the C, which passes a stray `DevExt->TransferModeSelectedBitmap`
        // between the two arguments this format names, so its `%s` reads that ULONG as a
        // string pointer. Pass only what the format consumes.
        WARN!(b"CH %lu: Failed to set geometry for '%s'\n\0".as_ptr() as *const i8,
              (*PortData).PortNumber,
              (*DevExt).FriendlyName.as_ptr());
    }
    return Status;
}

unsafe extern "C" fn AtaDeviceBuildSetTransferModeTaskFile(
    Request: PATA_DEVICE_REQUEST,
    Mode: UCHAR,
) {
    (*Request).Flags = 0;
    (*Request).TimeOut = 3;

    RtlZeroMemory!(
        &raw mut (*Request).__bindgen_anon_2.TaskFile as *mut core::ffi::c_void,
        core::mem::size_of::<ATA_TASKFILE>(),
    );
    (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command = (IDE_COMMAND_SET_FEATURE) as u8;
    (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_2.Feature = (IDE_FEATURE_SET_TRANSFER_MODE) as u8;
    (*Request).__bindgen_anon_2.TaskFile.SectorCount = Mode;
}

unsafe extern "C" fn AtaDeviceSetPioTransferMode(
    PortData: PATAPORT_PORT_DATA,
    DevExt: PATAPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let Request: PATA_DEVICE_REQUEST = &raw mut (*PortData).Worker.InternalRequest;
    let Status: NTSTATUS;
    let mut Mode: ULONG = core::mem::zeroed();

    NT_VERIFY!(
        _BitScanReverse(
            &raw mut Mode,
            (*DevExt).TransferModeSelectedBitmap & PIO_ALL,
        ) != 0,
    );

    Mode = (IDE_SET_ADVANCE_PIO_MODE!(Mode)) as u32;
    AtaDeviceBuildSetTransferModeTaskFile(Request, (Mode) as UCHAR);

    Status = AtaPortSendRequest(PortData, DevExt);
    if !NT_SUCCESS!(Status) {
        WARN!(b"CH %lu: Failed to set %lu PIO settings for '%s'\n\0".as_ptr() as *const i8,
              (*PortData).PortNumber,
              Mode,
              (*DevExt).FriendlyName.as_ptr());
    }

    return Status;
}

unsafe extern "C" fn AtaDeviceSetDmaTransferMode(
    PortData: PATAPORT_PORT_DATA,
    DevExt: PATAPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let Request: PATA_DEVICE_REQUEST = &raw mut (*PortData).Worker.InternalRequest;
    let Status: NTSTATUS;
    let mut Mode: ULONG = core::mem::zeroed();

    if _BitScanReverse(
        &raw mut Mode,
        (*DevExt).TransferModeSelectedBitmap & !PIO_ALL,
    ) == 0
    {
        return STATUS_SUCCESS;
    }

    if Mode >= UDMA_MODE!(0) {
        Mode = (IDE_SET_UDMA_MODE!(Mode - UDMA_MODE!(0))) as u32;
    } else if Mode >= MWDMA_MODE!(0) {
        Mode = (IDE_SET_MWDMA_MODE!(Mode - MWDMA_MODE!(0))) as u32;
    } else {
        Mode = (IDE_SET_SWDMA_MODE!(Mode - SWDMA_MODE!(0))) as u32;
    }
    AtaDeviceBuildSetTransferModeTaskFile(Request, (Mode) as UCHAR);

    Status = AtaPortSendRequest(PortData, DevExt);
    if NT_SUCCESS!(Status) {
        (*DevExt).Device.DeviceFlags &= !DEVICE_PIO_ONLY;
    } else {
        WARN!(b"CH %lu: Failed to set 0x%lx DMA settings for '%s'\n\0".as_ptr() as *const i8,
              (*PortData).PortNumber,
              Mode,
              (*DevExt).FriendlyName.as_ptr());

        (*DevExt).Device.DeviceFlags &= !DEVICE_NCQ;
        (*DevExt).TransferModeSelectedBitmap &= PIO_ALL;
    }

    return Status;
}

unsafe extern "C" fn AtaDeviceSetMultipleMode(
    PortData: PATAPORT_PORT_DATA,
    DevExt: PATAPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let Request: PATA_DEVICE_REQUEST = &raw mut (*PortData).Worker.InternalRequest;
    let Status: NTSTATUS;

    if IS_ATAPI!(&raw mut (*DevExt).Device) != false {
        return STATUS_SUCCESS;
    }

    (*DevExt).Device.__bindgen_anon_1.MultiSectorCount = AtaDevMaximumSectorsPerDrq(&raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData);
    if (*DevExt).Device.__bindgen_anon_1.MultiSectorCount == 0 {
        return STATUS_SUCCESS;
    }

    (*Request).Flags = 0;
    (*Request).TimeOut = 3;

    RtlZeroMemory!(
        &raw mut (*Request).__bindgen_anon_2.TaskFile as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<ATA_TASKFILE>(),
    );
    (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command = (IDE_COMMAND_SET_MULTIPLE) as u8;
    (*Request).__bindgen_anon_2.TaskFile.SectorCount = (*DevExt).Device.__bindgen_anon_1.MultiSectorCount;

    Status = AtaPortSendRequest(PortData, DevExt);
    if !NT_SUCCESS!(Status) {
        WARN!(b"CH %lu: Failed to set %u multiple mode for '%s'\n\0".as_ptr() as *const i8,
              (*PortData).PortNumber,
              (*DevExt).Device.__bindgen_anon_1.MultiSectorCount as u32,
              (*DevExt).FriendlyName.as_ptr());

        (*DevExt).Device.__bindgen_anon_1.MultiSectorCount = 0;
    }

    return Status;
}

unsafe extern "C" fn AtaDeviceFilterAcpiTaskFile(AcpiTaskFile: PATA_ACPI_TASK_FILE) -> BOOLEAN {
    match ((*AcpiTaskFile).Command as ::core::ffi::c_int) as u32 {
        IDE_COMMAND_SET_FEATURE => {
            if ((*AcpiTaskFile).Feature as ::core::ffi::c_int) as u32 == IDE_FEATURE_SET_TRANSFER_MODE {
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

unsafe extern "C" fn AtaDeviceAcpiTaskFileToInternalTaskFile(
    AcpiTaskFile: *mut ATA_ACPI_TASK_FILE,
    Request: *mut ATA_DEVICE_REQUEST,
) {
    (*Request).Flags = REQUEST_FLAG_SET_DEVICE_REGISTER;
    (*Request).TimeOut = 6;

    (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_2.Feature = (*AcpiTaskFile).Feature;
    (*Request).__bindgen_anon_2.TaskFile.SectorCount = (*AcpiTaskFile).SectorCount;
    (*Request).__bindgen_anon_2.TaskFile.LowLba = (*AcpiTaskFile).LowLba;
    (*Request).__bindgen_anon_2.TaskFile.MidLba = (*AcpiTaskFile).MidLba;
    (*Request).__bindgen_anon_2.TaskFile.HighLba = (*AcpiTaskFile).HighLba;
    (*Request).__bindgen_anon_2.TaskFile.DriveSelect = (*AcpiTaskFile).DriveSelect;
    (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command = (*AcpiTaskFile).Command;
}

unsafe extern "C" fn AtaDeviceExecuteAcpiTaskFile(
    mut PortData: PATAPORT_PORT_DATA,
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut GtfDataBuffer: PACPI_EVAL_OUTPUT_BUFFER = ((*DevExt).GtfDataBuffer).cast();
    let mut Request: PATA_DEVICE_REQUEST = &raw mut (*PortData).Worker.InternalRequest;
    let mut AcpiTaskFile: PATA_ACPI_TASK_FILE = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut i: ULONG = 0;

    if GtfDataBuffer.is_null() {
        return STATUS_SUCCESS;
    }

    ASSERT!((*GtfDataBuffer).Signature == ACPI_EVAL_OUTPUT_BUFFER_SIGNATURE);

    AcpiTaskFile = ((*GtfDataBuffer).Argument[0].__bindgen_anon_1.Data).as_mut_ptr() as PATA_ACPI_TASK_FILE;

    i = 0;
    while (i as usize)
        < ((*GtfDataBuffer).Argument[0].DataLength as usize)
            .wrapping_div(core::mem::size_of::<ATA_ACPI_TASK_FILE>())
    {
        let mut Result: PCSTR = core::ptr::null();

        if AtaDeviceFilterAcpiTaskFile(AcpiTaskFile) == 0 {
            Result = b"filtered out\0".as_ptr() as PCSTR;
        } else {
            AtaDeviceAcpiTaskFileToInternalTaskFile(AcpiTaskFile, Request);

            Status = AtaPortSendRequest(PortData, DevExt);
            if Status == STATUS_ADAPTER_HARDWARE_ERROR {
                return Status;
            }
            Result = if NT_SUCCESS!(Status) {
                b"completed\0".as_ptr() as PCSTR
            } else {
                b"aborted\0".as_ptr() as PCSTR
            };
        }

        INFO!(
            b"CH %lu: GTF[%lu]: %02x:%02x:%02x:%02x:%02x:%02x:%02x -- %s\n\0".as_ptr() as PCSTR,
            (*PortData).PortNumber,
            i,
            (*AcpiTaskFile).Feature,
            (*AcpiTaskFile).SectorCount,
            (*AcpiTaskFile).LowLba,
            (*AcpiTaskFile).MidLba,
            (*AcpiTaskFile).HighLba,
            (*AcpiTaskFile).DriveSelect,
            (*AcpiTaskFile).Command,
            Result,
        );

        AcpiTaskFile = AcpiTaskFile.add(1);
        i = i.wrapping_add(1);
    }

    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaDeviceLockSecurityModeFeatureCommands(
    PortData: PATAPORT_PORT_DATA,
    DevExt: PATAPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let Request: PATA_DEVICE_REQUEST = &raw mut (*PortData).Worker.InternalRequest;

    if IS_ATAPI!(&raw mut (*DevExt).Device) != false
        || AtapInPEMode != 0
        || AtaDevHasSecurityModeFeature(&raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData) == 0
    {
        return STATUS_SUCCESS;
    }

    (*Request).Flags = 0;
    (*Request).TimeOut = 3;

    RtlZeroMemory!(
        &raw mut (*Request).__bindgen_anon_2.TaskFile as *mut core::ffi::c_void,
        core::mem::size_of::<ATA_TASKFILE>(),
    );
    (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command = (IDE_COMMAND_SECURITY_FREEZE_LOCK) as u8;

    return AtaPortSendRequest(PortData, DevExt);
}

unsafe extern "C" fn AtaDeviceLockDeviceParameters(
    PortData: PATAPORT_PORT_DATA,
    DevExt: PATAPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let Request: PATA_DEVICE_REQUEST = core::ptr::addr_of_mut!((*PortData).Worker.InternalRequest);

    (*Request).Flags = 0;
    (*Request).TimeOut = 3;

    RtlZeroMemory!(
        core::ptr::addr_of_mut!((*Request).__bindgen_anon_2.TaskFile) as *mut core::ffi::c_void,
        core::mem::size_of::<ATA_TASKFILE>(),
    );
    (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command = (IDE_COMMAND_SET_FEATURE) as u8;
    (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_2.Feature = (IDE_FEATURE_DISABLE_REVERT_TO_POWER_ON) as u8;

    return AtaPortSendRequest(PortData, DevExt);
}

unsafe extern "C" fn AtaDeviceEnableMsnFeature(
    mut PortData: PATAPORT_PORT_DATA,
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut Request: PATA_DEVICE_REQUEST = &raw mut (*PortData).Worker.InternalRequest;
    let mut Status: NTSTATUS = 0;
    if AtaDevHasRemovableMediaStatusNotification(&raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData) == 0 {
        return STATUS_SUCCESS;
    }
    (*Request).Flags = 0;
    (*Request).TimeOut = 3;
    RtlZeroMemory!(
        &raw mut (*Request).__bindgen_anon_2.TaskFile as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<ATA_TASKFILE>(),
    );
    (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command = (IDE_COMMAND_SET_FEATURE) as u8;
    (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_2.Feature = (IDE_FEATURE_ENABLE_MSN) as u8;
    Status = AtaPortSendRequest(PortData, DevExt);
    if NT_SUCCESS!(Status) {
        (*DevExt).Device.DeviceFlags |= DEVICE_HAS_MEDIA_STATUS;
    }
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn AtaPortDeviceProcessConfig(
    PortData: PATAPORT_PORT_DATA,
    DevExt: PATAPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    (*DevExt).Device.DeviceFlags |= DEVICE_PIO_ONLY;
    (*DevExt).Device.DeviceFlags &= !(DEVICE_HAS_MEDIA_STATUS | DEVICE_SENSE_DATA_REPORTING);
    (*DevExt).Device.__bindgen_anon_1.MultiSectorCount = 0;

    Status = AtaDeviceSetGeometry(PortData, DevExt);
    if Status == STATUS_ADAPTER_HARDWARE_ERROR {
        return Status;
    }

    Status = AtaDeviceSetPioTransferMode(PortData, DevExt);
    if Status == STATUS_ADAPTER_HARDWARE_ERROR {
        return Status;
    }

    Status = AtaDeviceSetDmaTransferMode(PortData, DevExt);
    if Status == STATUS_ADAPTER_HARDWARE_ERROR {
        return Status;
    }

    Status = AtaDeviceSetMultipleMode(PortData, DevExt);
    if Status == STATUS_ADAPTER_HARDWARE_ERROR {
        return Status;
    }

    Status = AtaDeviceExecuteAcpiTaskFile(PortData, DevExt);
    if Status == STATUS_ADAPTER_HARDWARE_ERROR {
        return Status;
    }

    Status = AtaDeviceLockSecurityModeFeatureCommands(PortData, DevExt);
    if Status == STATUS_ADAPTER_HARDWARE_ERROR {
        return Status;
    }

    Status = AtaDeviceLockDeviceParameters(PortData, DevExt);
    if Status == STATUS_ADAPTER_HARDWARE_ERROR {
        return Status;
    }

    Status = AtaDeviceEnableMsnFeature(PortData, DevExt);
    if Status == STATUS_ADAPTER_HARDWARE_ERROR {
        return Status;
    }

    Status = AtaDeviceSendIdentify(
        PortData,
        DevExt,
        if IS_ATAPI!(&(*DevExt).Device) != false {
            (IDE_COMMAND_ATAPI_IDENTIFY) as u8
        } else {
            (IDE_COMMAND_IDENTIFY) as u8
        },
    );
    if NT_SUCCESS!(Status) {
        RtlCopyMemory!(
            core::ptr::addr_of_mut!((*DevExt).__bindgen_anon_1.IdentifyDeviceData) as *mut core::ffi::c_void,
            (*DevExt).Device.LocalBuffer as *const core::ffi::c_void,
            core::mem::size_of::<IDENTIFY_DEVICE_DATA>(),
        );
    }

    return Status;
}
