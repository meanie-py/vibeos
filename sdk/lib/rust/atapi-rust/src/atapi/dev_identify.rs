use crate::prelude::*;
use ntkrnl_sys::ddk::{PCSTR};
use core::ffi::c_void;

unsafe extern "C" fn AtaDeviceIdentifyDataEqual(
    mut IdentifyData1: PIDENTIFY_DEVICE_DATA,
    mut IdentifyData2: PIDENTIFY_DEVICE_DATA,
) -> BOOLEAN {
    if !RtlEqualMemory(
        &raw mut (*IdentifyData1).SerialNumber as *mut UCHAR as *const ::core::ffi::c_void,
        &raw mut (*IdentifyData2).SerialNumber as *mut UCHAR as *const ::core::ffi::c_void,
        RTL_FIELD_SIZE!(IDENTIFY_DEVICE_DATA, SerialNumber),
    ) {
        return FALSE;
    }
    if !RtlEqualMemory(
        &raw mut (*IdentifyData1).FirmwareRevision as *mut UCHAR as *const ::core::ffi::c_void,
        &raw mut (*IdentifyData2).FirmwareRevision as *mut UCHAR as *const ::core::ffi::c_void,
        RTL_FIELD_SIZE!(IDENTIFY_DEVICE_DATA, FirmwareRevision),
    ) {
        return FALSE;
    }
    if !RtlEqualMemory(
        &raw mut (*IdentifyData1).ModelNumber as *mut UCHAR as *const ::core::ffi::c_void,
        &raw mut (*IdentifyData2).ModelNumber as *mut UCHAR as *const ::core::ffi::c_void,
        RTL_FIELD_SIZE!(IDENTIFY_DEVICE_DATA, ModelNumber),
    ) {
        return FALSE;
    }
    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn AtaDeviceSendIdentify(
    PortData: *mut ATAPORT_PORT_DATA,
    DevExt: *mut ATAPORT_DEVICE_EXTENSION,
    Command: u8,
) -> NTSTATUS {
    let Request: *mut ATA_DEVICE_REQUEST = &raw mut (*PortData).Worker.InternalRequest;

    (*Request).Flags = REQUEST_FLAG_POLL;

    (*Request).Flags |= REQUEST_FLAG_DATA_IN | REQUEST_FLAG_HAS_LOCAL_BUFFER;
    (*Request).TimeOut = 10;
    (*Request).DataTransferLength = core::mem::size_of::<IDENTIFY_DEVICE_DATA>() as u32;

    RtlZeroMemory!(
        &raw mut (*Request).__bindgen_anon_2.TaskFile as *mut core::ffi::c_void,
        core::mem::size_of::<ATA_TASKFILE>(),
    );
    (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command = Command;

    return AtaPortSendRequest(PortData, DevExt);
}

unsafe extern "C" fn AtaDeviceSpinUp(
    PortData: PATAPORT_PORT_DATA,
    DevExt: PATAPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let Request: PATA_DEVICE_REQUEST = &raw mut (*PortData).Worker.InternalRequest;

    (*Request).Flags = 0;
    (*Request).TimeOut = 20;

    (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command = (IDE_COMMAND_SET_FEATURE) as u8;
    (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_2.Feature = (IDE_FEATURE_PUIS_SPIN_UP) as u8;

    return AtaPortSendRequest(PortData, DevExt);
}

unsafe extern "C" fn AtaDeviceIsXboxDrive(mut DevExt: PATAPORT_DEVICE_EXTENSION) -> BOOLEAN {
    static mut AtapBrokenInquiryDrive: [PCSTR; 4] = [
        b"THOMSON-DVD\0" as *const u8 as *const ::core::ffi::c_char,
        b"PHILIPS XBOX DVD DRIVE\0" as *const u8 as *const ::core::ffi::c_char,
        b"PHILIPS J5 3235C\0" as *const u8 as *const ::core::ffi::c_char,
        b"SAMSUNG DVD-ROM SDG-605B\0" as *const u8 as *const ::core::ffi::c_char,
    ];
    let mut ModelNumber: [CHAR; 26] = [0; 26];
    let mut End: PUCHAR = ::core::ptr::null_mut::<::core::ffi::c_uchar>();
    let mut i: ULONG = 0;
    End = AtaCopyIdStringUnsafe(
        &raw mut ModelNumber as *mut CHAR as PUCHAR,
        &raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData.ModelNumber as PUCHAR,
        ::core::mem::size_of::<[CHAR; 26]>() as ULONG,
    );
    *End = ANSI_NULL as ::core::ffi::c_uchar;
    i = 0 as ULONG;
    while (i as usize)
        < (::core::mem::size_of::<[PCSTR; 4]>() as usize)
            .wrapping_div(::core::mem::size_of::<PCSTR>() as usize)
    {
        if strcmp(
            &raw mut ModelNumber as *mut CHAR,
            AtapBrokenInquiryDrive[i as usize],
        ) == 0 as ::core::ffi::c_int
        {
            return TRUE as BOOLEAN;
        }
        i = i.wrapping_add(1);
    }
    return FALSE as BOOLEAN;
}

unsafe extern "C" fn AtaCreateAtapiStandardInquiryData(
    IdentifyData: PIDENTIFY_DEVICE_DATA,
    InquiryData: PINQUIRYDATA,
) {
    (*InquiryData).set_DeviceType((READ_ONLY_DIRECT_ACCESS_DEVICE) as _);
    (*InquiryData).set_RemovableMedia((AtaDevIsRemovable(IdentifyData)) as _);
    (*InquiryData).__bindgen_anon_1.Versions = 0x04;
    (*InquiryData).set_ResponseDataFormat((2) as _);
    (*InquiryData).AdditionalLength =
        (INQUIRYDATABUFFERSIZE - (RTL_SIZEOF_THROUGH_FIELD!(INQUIRYDATA, AdditionalLength)) as u32) as u8;

    AtaCopyIdStringSafe(
        (*InquiryData).VendorId.as_mut_ptr() as PCHAR,
        (*IdentifyData).ModelNumber.as_mut_ptr(),
        core::mem::size_of::<[UCHAR; 40]>() as ULONG,
        ' ' as i32 as CHAR,
    );
    AtaSwapIdString(
        (*InquiryData).VendorId.as_mut_ptr() as PVOID,
        (core::mem::size_of::<[UCHAR; 40]>() as usize / 2) as ULONG,
    );

    AtaCopyIdStringSafe(
        (*InquiryData).ProductRevisionLevel.as_mut_ptr() as PCHAR,
        (*IdentifyData).FirmwareRevision.as_mut_ptr(),
        core::mem::size_of::<[UCHAR; 4]>() as ULONG,
        ' ' as i32 as CHAR,
    );
    AtaSwapIdString(
        (*InquiryData).ProductRevisionLevel.as_mut_ptr() as PVOID,
        (core::mem::size_of::<[UCHAR; 4]>() as usize / 2) as ULONG,
    );
}

unsafe extern "C" fn AtaDeviceSendInquiry(
    PortData: PATAPORT_PORT_DATA,
    DevExt: PATAPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let Request: PATA_DEVICE_REQUEST = &raw mut (*PortData).Worker.InternalRequest;
    let mut Status: NTSTATUS;
    let Cdb: PCDB;

    (*Request).Flags = REQUEST_FLAG_DATA_IN | REQUEST_FLAG_PACKET_COMMAND | REQUEST_FLAG_HAS_LOCAL_BUFFER;
    (*Request).TimeOut = 3;
    (*Request).DataTransferLength = INQUIRYDATABUFFERSIZE;

    RtlZeroMemory!(
        &raw mut (*Request).__bindgen_anon_2.Cdb as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of_val(&(*Request).__bindgen_anon_2.Cdb),
    );
    Cdb = &raw mut (*Request).__bindgen_anon_2.Cdb as PCDB;
    (*Cdb).CDB6INQUIRY.OperationCode = (SCSIOP_INQUIRY) as u8;
    (*Cdb).CDB6INQUIRY.set_LogicalUnitNumber((*DevExt).Device.AtaScsiAddress.Lun);
    (*Cdb).CDB6INQUIRY.AllocationLength = (INQUIRYDATABUFFERSIZE) as u8;

    Status = AtaPortSendRequest(PortData, DevExt);
    if Status == STATUS_ADAPTER_HARDWARE_ERROR {
        return Status;
    }

    if !NT_SUCCESS!(Status) {
        if (*DevExt).Device.AtaScsiAddress.Lun == 0
            && AtaDeviceIsXboxDrive(DevExt) != 0
        {
            (*DevExt).Device.DeviceFlags |= DEVICE_CACHE_INQUIRY;

            AtaCreateAtapiStandardInquiryData(
                &raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData,
                ((*DevExt).Device.LocalBuffer).cast(),
            );
            Status = (DEV_STATUS_NEW_DEVICE) as i32;
        } else {
            Status = (DEV_STATUS_NO_DEVICE) as i32;
        }
    }

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn AtaPortIdentifyDevice(
    PortData: *mut ATAPORT_PORT_DATA,
    DevExt: *mut ATAPORT_DEVICE_EXTENSION,
) -> ATA_DEVICE_STATUS {
    let IdentifyData: *mut IDENTIFY_DEVICE_DATA = (*DevExt).Device.LocalBuffer as *mut IDENTIFY_DEVICE_DATA;
    let mut ConnectionStatus: ATA_CONNECTION_STATUS;
    let mut Status: NTSTATUS;
    let mut StartFromAtapi: BOOLEAN;
    let mut IsSameDevice: BOOLEAN;
    let mut Attempt: ULONG;
    let mut RetryCount: ULONG;
    let mut DeviceType: ATA_DEVICE_TYPE = core::mem::zeroed();

    _InterlockedOr(&mut (*PortData).InterruptFlags, (PORT_INT_FLAG_IGNORE_LINK_IRQ) as i32);

    ConnectionStatus = (*PortData).IdentifyDevice.unwrap_unchecked()(
        (*PortData).ChannelContext,
        ((*DevExt).Device.AtaScsiAddress.TargetId) as _,
    );

    _InterlockedAnd(&mut (*PortData).InterruptFlags, (!PORT_INT_FLAG_IGNORE_LINK_IRQ) as i32);

    if ConnectionStatus == CONN_STATUS_FAILURE {
        return DEV_STATUS_FAILED;
    }

    if ConnectionStatus == CONN_STATUS_NO_DEVICE {
        return DEV_STATUS_NO_DEVICE;
    }

    Status = AtaPortCheckDevicePowerState(PortData, DevExt);
    if Status == STATUS_ADAPTER_HARDWARE_ERROR {
        return DEV_STATUS_FAILED;
    }

    if (*DevExt).Device.DeviceFlags & DEVICE_UNINITIALIZED != 0 {
        StartFromAtapi = (ConnectionStatus == CONN_STATUS_DEV_ATAPI) as BOOLEAN;
    } else {
        StartFromAtapi = (IS_ATAPI!(&(*DevExt).Device)) as u8;
    }

    RetryCount = 0;
    while RetryCount < 2 {
        Attempt = 0;
        while Attempt < 2 {
            if StartFromAtapi != 0 {
                DeviceType = DEV_ATAPI;
            } else {
                DeviceType = DEV_ATA;
            }

            if Attempt != 0 {
                DeviceType = core::mem::transmute::<u32, ATA_DEVICE_TYPE>(
                    DeviceType as u32 ^ (DEV_ATAPI ^ DEV_ATA) as u32,
                );
            }

            Status = AtaDeviceSendIdentify(
                PortData,
                DevExt,
                if DeviceType == DEV_ATAPI {
                    (IDE_COMMAND_ATAPI_IDENTIFY) as u8
                } else {
                    (IDE_COMMAND_IDENTIFY) as u8
                },
            );
            if Status == STATUS_ADAPTER_HARDWARE_ERROR {
                return DEV_STATUS_FAILED;
            }

            if NT_SUCCESS!(Status) {
                break;
            }
            Attempt = Attempt.wrapping_add(1);
        }
        if !NT_SUCCESS!(Status) {
            return DEV_STATUS_NO_DEVICE;
        }

        if AtaDevInPuisState(IdentifyData) != 0 || AtaDevIsIdentifyDataIncomplete(IdentifyData) != 0 {
            Status = AtaDeviceSpinUp(PortData, DevExt);
            if Status == STATUS_ADAPTER_HARDWARE_ERROR {
                return DEV_STATUS_FAILED;
            }

            if !NT_SUCCESS!(Status) {
                ERR!("Failed to spin-up device\n");
            }
            RetryCount = RetryCount.wrapping_add(1);
            continue;
        }
        RetryCount = RetryCount.wrapping_add(1);
    }

    if AtaDevIsIdentifyDataValid(IdentifyData) == 0 {
        return DEV_STATUS_FAILED;
    }

    if (*DevExt).Device.DeviceFlags & DEVICE_UNINITIALIZED != 0 {
        IsSameDevice = FALSE;
    } else {
        IsSameDevice = (DeviceType == (*DevExt).DeviceType
            && AtaDeviceIdentifyDataEqual(
                &mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData,
                (*DevExt).Device.LocalBuffer as *mut IDENTIFY_DEVICE_DATA,
            ) != 0) as BOOLEAN;
    }

    RtlCopyMemory!(
        &mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData as *mut IDENTIFY_DEVICE_DATA as *mut core::ffi::c_void,
        (*DevExt).Device.LocalBuffer as *const core::ffi::c_void,
        core::mem::size_of::<IDENTIFY_DEVICE_DATA>(),
    );
    (*DevExt).DeviceType = DeviceType;

    if IsSameDevice != 0 {
        return DEV_STATUS_SAME_DEVICE;
    }

    (*DevExt).Device.__bindgen_anon_1.TransportFlags &= !(DEVICE_IS_ATAPI
        | DEVICE_HAS_CDB_INTERRUPT
        | DEVICE_NEED_DMA_DIRECTION
        | DEVICE_IS_NEC_CDR260);

    if DeviceType == DEV_ATAPI {
        (*DevExt).Device.__bindgen_anon_1.TransportFlags |= DEVICE_IS_ATAPI;

        (*DevExt).Device.__bindgen_anon_1.CdbSize = AtaDevCdbSizeInWords(&mut (*DevExt).__bindgen_anon_1.IdentifyPacketData);

        if AtaDevHasCdbInterrupt(&mut (*DevExt).__bindgen_anon_1.IdentifyPacketData) != 0 {
            (*DevExt).Device.__bindgen_anon_1.TransportFlags |= DEVICE_HAS_CDB_INTERRUPT;
        }

        if AtaDevIsDmaDirectionRequired(&mut (*DevExt).__bindgen_anon_1.IdentifyPacketData) != 0 {
            (*DevExt).Device.__bindgen_anon_1.TransportFlags |= DEVICE_NEED_DMA_DIRECTION;
        }

        if (*DevExt).__bindgen_anon_1.IdentifyPacketData.ModelNumber[0 as usize] as i32 == 'N' as i32
            && (*DevExt).__bindgen_anon_1.IdentifyPacketData.ModelNumber[1 as usize] as i32 == 'E' as i32
            && (*DevExt).__bindgen_anon_1.IdentifyPacketData.ModelNumber[2 as usize] as i32 == 'C' as i32
        {
            (*DevExt).Device.__bindgen_anon_1.TransportFlags |= DEVICE_IS_NEC_CDR260;
        }

        Status = AtaDeviceSendInquiry(PortData, DevExt);
        if Status == STATUS_ADAPTER_HARDWARE_ERROR {
            return DEV_STATUS_FAILED;
        }

        RtlCopyMemory!(
            &mut (*DevExt).InquiryData as *mut INQUIRYDATA as *mut core::ffi::c_void,
            (*DevExt).Device.LocalBuffer as *const core::ffi::c_void,
            core::mem::size_of::<INQUIRYDATA>(),
        );
    }

    return DEV_STATUS_NEW_DEVICE;
}
