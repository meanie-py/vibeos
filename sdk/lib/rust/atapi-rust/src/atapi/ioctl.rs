use crate::prelude::*;
use ntkrnl_sys::ddk::{ExAllocatePoolZero, _KWAIT_REASON_Executive, _IO_STATUS_BLOCK, IoBuildSynchronousFsdRequest, IoInvalidateDeviceRelations, KEVENT, KeInitializeEvent, KeWaitForSingleObject, LARGE_INTEGER, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PDEVICE_SEEK_PENALTY_DESCRIPTOR, PDEVICE_TRIM_DESCRIPTOR, PIO_STACK_LOCATION, PIRP, PSTORAGE_ACCESS_ALIGNMENT_DESCRIPTOR, PSTORAGE_ADAPTER_DESCRIPTOR, PSTORAGE_DESCRIPTOR_HEADER, PSTORAGE_DEVICE_DESCRIPTOR, PSTORAGE_PROPERTY_QUERY, STORAGE_DEVICE_DESCRIPTOR};
use core::ffi::c_void;

unsafe extern "C" fn AtaCheckPropertyQuery(
    mut Irp: PIRP,
    mut DescriptorSize: ULONG,
    mut Status: *mut NTSTATUS,
) -> BOOLEAN {
    let mut PropertyQuery: PSTORAGE_PROPERTY_QUERY =
        (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_PROPERTY_QUERY;
    let mut DescriptorHeader: PSTORAGE_DESCRIPTOR_HEADER =
        core::ptr::null_mut::<STORAGE_DESCRIPTOR_HEADER>();
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut::<IO_STACK_LOCATION>();

    PAGED_CODE!();

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    if !AtaVerifyInBuffer!(IoStack, (core::mem::size_of::<STORAGE_PROPERTY_QUERY>()) as _) {
        *Status = STATUS_INVALID_PARAMETER;
        return FALSE;
    }

    if (*PropertyQuery).QueryType != PropertyStandardQuery
        && (*PropertyQuery).QueryType != PropertyExistsQuery
    {
        *Status = STATUS_NOT_SUPPORTED;
        return FALSE;
    }

    if (*PropertyQuery).QueryType == PropertyExistsQuery {
        *Status = STATUS_SUCCESS;
        return FALSE;
    }

    if !AtaVerifyOutBuffer!(IoStack, (core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>()) as _) {
        *Status = STATUS_INFO_LENGTH_MISMATCH;
        return FALSE;
    }

    if !AtaVerifyOutBuffer!(IoStack, DescriptorSize) {
        DescriptorHeader = PropertyQuery as PSTORAGE_DESCRIPTOR_HEADER;
        (*DescriptorHeader).Version = DescriptorSize;
        (*DescriptorHeader).Size = DescriptorSize;

        (*Irp).IoStatus.Information = core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as ULONG_PTR;

        *Status = STATUS_SUCCESS;
        return FALSE;
    }

    return TRUE;
}

unsafe extern "C" fn AtaPdoQueryStorageDeviceProperty(
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
    mut Irp: PIRP,
) -> NTSTATUS {
    let mut InquiryData: PINQUIRYDATA = &raw mut (*DevExt).InquiryData;
    let mut DeviceDescriptor: PSTORAGE_DEVICE_DESCRIPTOR =
        ::core::ptr::null_mut::<_STORAGE_DEVICE_DESCRIPTOR>();
    let mut Status: NTSTATUS = 0;
    let mut DescriptorSize: ULONG = 0;
    let mut RawPropertiesLength: ULONG = 0;
    let mut Length1: ULONG = 0;
    let mut Length2: ULONG = 0;
    let mut Length3: ULONG = 0;
    PAGED_CODE!();
    Length1 = (strlen(&raw mut (*DevExt).FriendlyName as *mut CHAR) as ULONG as usize)
        .wrapping_add(::core::mem::size_of_val(&ANSI_NULL) as usize) as ULONG;
    Length2 = (strlen(&raw mut (*DevExt).RevisionNumber as *mut CHAR) as ULONG as usize)
        .wrapping_add(::core::mem::size_of_val(&ANSI_NULL) as usize) as ULONG;
    Length3 = (strlen(&raw mut (*DevExt).SerialNumber as *mut CHAR) as ULONG as usize)
        .wrapping_add(::core::mem::size_of_val(&ANSI_NULL) as usize) as ULONG;
    RawPropertiesLength = Length1.wrapping_add(Length2).wrapping_add(Length3);
    DescriptorSize = (FIELD_OFFSET!(STORAGE_DEVICE_DESCRIPTOR, RawDeviceProperties)
        .wrapping_add((RawPropertiesLength) as i32)) as u32;
    if AtaCheckPropertyQuery(Irp, DescriptorSize, &raw mut Status) == 0 {
        return Status;
    }
    DeviceDescriptor = (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_DEVICE_DESCRIPTOR;
    (*DeviceDescriptor).RawPropertiesLength = RawPropertiesLength;
    (*DeviceDescriptor).Version = ::core::mem::size_of::<_STORAGE_DEVICE_DESCRIPTOR>() as ULONG;
    (*DeviceDescriptor).Size = DescriptorSize;
    (*DeviceDescriptor).DeviceType = (*InquiryData).DeviceType();
    (*DeviceDescriptor).DeviceTypeModifier = (*InquiryData).DeviceTypeModifier();
    (*DeviceDescriptor).RemovableMedia = (*InquiryData).RemovableMedia() as BOOLEAN;
    (*DeviceDescriptor).CommandQueueing = FALSE as BOOLEAN;
    if (*(*DevExt).Device.PortData).PortFlags & PORT_FLAG_IS_AHCI as ULONG != 0 {
        (*DeviceDescriptor).BusType = BusTypeSata;
    } else {
        (*DeviceDescriptor).BusType = BusTypeAta;
    }
    (*DeviceDescriptor).VendorIdOffset = 0 as ULONG;
    (*DeviceDescriptor).ProductIdOffset = FIELD_OFFSET!(STORAGE_DEVICE_DESCRIPTOR, RawDeviceProperties) as ULONG;
    RtlCopyMemory!(
        (DeviceDescriptor as ULONG_PTR)
            .wrapping_add((*DeviceDescriptor).ProductIdOffset as ULONG_PTR) as PUCHAR
            as *mut ::core::ffi::c_void,
        &raw mut (*DevExt).FriendlyName as *mut CHAR as *const ::core::ffi::c_void,
        Length1 as size_t,
    );
    (*DeviceDescriptor).ProductRevisionOffset =
        (*DeviceDescriptor).ProductIdOffset.wrapping_add(Length1);
    RtlCopyMemory!(
        (DeviceDescriptor as ULONG_PTR)
            .wrapping_add((*DeviceDescriptor).ProductRevisionOffset as ULONG_PTR) as PUCHAR
            as *mut ::core::ffi::c_void,
        &raw mut (*DevExt).RevisionNumber as *mut CHAR as *const ::core::ffi::c_void,
        Length2 as size_t,
    );
    (*DeviceDescriptor).SerialNumberOffset = (*DeviceDescriptor)
        .ProductRevisionOffset
        .wrapping_add(Length2);
    RtlCopyMemory!(
        (DeviceDescriptor as ULONG_PTR)
            .wrapping_add((*DeviceDescriptor).SerialNumberOffset as ULONG_PTR) as PUCHAR
            as *mut ::core::ffi::c_void,
        &raw mut (*DevExt).SerialNumber as *mut CHAR as *const ::core::ffi::c_void,
        Length3 as size_t,
    );
    (*Irp).IoStatus.Information = DescriptorSize as ULONG_PTR;
    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaPdoQueryStorageAccessAlignmentProperty(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut AccessAlignmentDescriptor: PSTORAGE_ACCESS_ALIGNMENT_DESCRIPTOR =
        core::ptr::null_mut();
    let mut LogicalSectorsPerPhysicalSector: u32 = 0;
    let mut Dummy: u32 = 0;
    let mut Status: NTSTATUS = 0;

    PAGED_CODE!();

    if IS_ATAPI!(&(*DevExt).Device) != false {
        return STATUS_NOT_SUPPORTED;
    }

    if AtaCheckPropertyQuery(
        Irp,
        core::mem::size_of::<STORAGE_ACCESS_ALIGNMENT_DESCRIPTOR>() as u32,
        &mut Status,
    ) == 0
    {
        return Status;
    }

    AccessAlignmentDescriptor =
        (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_ACCESS_ALIGNMENT_DESCRIPTOR;
    (*AccessAlignmentDescriptor).Version =
        core::mem::size_of::<STORAGE_ACCESS_ALIGNMENT_DESCRIPTOR>() as u32;
    (*AccessAlignmentDescriptor).Size =
        core::mem::size_of::<STORAGE_ACCESS_ALIGNMENT_DESCRIPTOR>() as u32;
    (*AccessAlignmentDescriptor).BytesPerCacheLine = 0;
    (*AccessAlignmentDescriptor).BytesOffsetForCacheAlignment = 0;
    (*AccessAlignmentDescriptor).BytesPerLogicalSector = (*DevExt).Device.__bindgen_anon_1.SectorSize;

    LogicalSectorsPerPhysicalSector =
        AtaDevLogicalSectorsPerPhysicalSector(&mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData, &mut Dummy);

    (*AccessAlignmentDescriptor).BytesPerPhysicalSector = (*DevExt)
        .Device
        .__bindgen_anon_1.SectorSize
        .wrapping_mul(LogicalSectorsPerPhysicalSector);

    (*AccessAlignmentDescriptor).BytesOffsetForSectorAlignment = (*DevExt)
        .Device
        .__bindgen_anon_1.SectorSize
        .wrapping_mul(AtaDevLogicalSectorAlignment(&mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData));

    (*Irp).IoStatus.Information =
        core::mem::size_of::<STORAGE_ACCESS_ALIGNMENT_DESCRIPTOR>() as usize as u64;
    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaPdoQueryStorageDeviceSeekPenaltyProperty(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut PenaltyDescriptor: PDEVICE_SEEK_PENALTY_DESCRIPTOR = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut IncursSeekPenalty: BOOLEAN = 0;

    PAGED_CODE!();

    if AtaCheckPropertyQuery(Irp, core::mem::size_of::<DEVICE_SEEK_PENALTY_DESCRIPTOR>() as u32, &mut Status) == 0 {
        return Status;
    }

    if AtaDevIsRotatingDevice(&mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData) != 0 {
        IncursSeekPenalty = TRUE;
    } else if AtaDevIsSsd(&mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData) != 0 {
        IncursSeekPenalty = FALSE;
    } else {
        return STATUS_UNSUCCESSFUL;
    }

    PenaltyDescriptor = (*Irp).AssociatedIrp.SystemBuffer as PDEVICE_SEEK_PENALTY_DESCRIPTOR;
    (*PenaltyDescriptor).Version = core::mem::size_of::<DEVICE_SEEK_PENALTY_DESCRIPTOR>() as u32;
    (*PenaltyDescriptor).Size = core::mem::size_of::<DEVICE_SEEK_PENALTY_DESCRIPTOR>() as u32;
    (*PenaltyDescriptor).IncursSeekPenalty = IncursSeekPenalty;

    (*Irp).IoStatus.Information = core::mem::size_of::<DEVICE_SEEK_PENALTY_DESCRIPTOR>() as usize as ULONG_PTR;
    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaPdoQueryStorageDeviceTrimProperty(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut TrimDescriptor: PDEVICE_TRIM_DESCRIPTOR = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    PAGED_CODE!();

    if IS_ATAPI!(&(*DevExt).Device) != false {
        return STATUS_NOT_SUPPORTED;
    }

    if AtaCheckPropertyQuery(Irp, core::mem::size_of::<DEVICE_TRIM_DESCRIPTOR>() as ULONG, &mut Status) == 0 {
        return Status;
    }

    TrimDescriptor = (*Irp).AssociatedIrp.SystemBuffer as PDEVICE_TRIM_DESCRIPTOR;
    (*TrimDescriptor).Version = core::mem::size_of::<DEVICE_TRIM_DESCRIPTOR>() as ULONG;
    (*TrimDescriptor).Size = core::mem::size_of::<DEVICE_TRIM_DESCRIPTOR>() as ULONG;
    (*TrimDescriptor).TrimEnabled = (AtaDevHasTrimFunction(&(*DevExt).__bindgen_anon_1.IdentifyDeviceData)) as u8;

    (*Irp).IoStatus.Information = core::mem::size_of::<DEVICE_TRIM_DESCRIPTOR>() as ULONG_PTR;
    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaPdoHandleStorageQueryProperty(
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
    mut Irp: PIRP,
    mut IoStack: PIO_STACK_LOCATION,
    mut ForwardToFdo: PBOOLEAN,
) -> NTSTATUS {
    let mut PropertyQuery: PSTORAGE_PROPERTY_QUERY = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    PAGED_CODE!();

    if !AtaVerifyInBuffer!(IoStack, (core::mem::size_of::<STORAGE_PROPERTY_QUERY>()) as _) {
        return STATUS_BUFFER_TOO_SMALL;
    }

    PropertyQuery = (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_PROPERTY_QUERY;

    match (*PropertyQuery).PropertyId as u32 {
        StorageDeviceProperty => {
            Status = AtaPdoQueryStorageDeviceProperty(DevExt, Irp);
        }
        StorageAccessAlignmentProperty => {
            Status = AtaPdoQueryStorageAccessAlignmentProperty(DevExt, Irp);
        }
        StorageDeviceSeekPenaltyProperty => {
            Status = AtaPdoQueryStorageDeviceSeekPenaltyProperty(DevExt, Irp);
        }
        StorageDeviceTrimProperty => {
            Status = AtaPdoQueryStorageDeviceTrimProperty(DevExt, Irp);
        }
        _ => {
            *ForwardToFdo = TRUE;
            return STATUS_MORE_PROCESSING_REQUIRED;
        }
    }

    *ForwardToFdo = FALSE;
    return Status;
}

unsafe extern "C" fn AtaPdoHandleGetScsiAddress(
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
    mut Irp: PIRP,
    mut IoStack: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut ChanExt: PATAPORT_CHANNEL_EXTENSION = core::ptr::null_mut();
    let mut ScsiAddress: PSCSI_ADDRESS = core::ptr::null_mut();
    let mut AtaScsiAddress: ATA_SCSI_ADDRESS = ATA_SCSI_ADDRESS {
        c2rust_unnamed: ATA_SCSI_ADDRESS__bindgen_ty_1 {
            Lun: 0,
            TargetId: 0,
            PathId: 0,
            IsValid: 0,
        },
    };

    if !((*IoStack).Parameters.DeviceIoControl.OutputBufferLength as usize
        >= core::mem::size_of::<SCSI_ADDRESS>() as usize)
    {
        return STATUS_BUFFER_TOO_SMALL;
    }

    ChanExt = (*DevExt).Common.__bindgen_anon_1.FdoExt as PATAPORT_CHANNEL_EXTENSION;
    AtaScsiAddress = (*DevExt).Device.AtaScsiAddress;
    ScsiAddress = (*Irp).AssociatedIrp.SystemBuffer as PSCSI_ADDRESS;
    (*ScsiAddress).Length = core::mem::size_of::<SCSI_ADDRESS>() as ULONG;
    (*ScsiAddress).PortNumber = (*ChanExt).ScsiPortNumber as UCHAR;
    (*ScsiAddress).PathId = AtaScsiAddress.c2rust_unnamed.PathId;
    (*ScsiAddress).TargetId = AtaScsiAddress.c2rust_unnamed.TargetId;
    (*ScsiAddress).Lun = AtaScsiAddress.c2rust_unnamed.Lun;
    (*Irp).IoStatus.Information = core::mem::size_of::<SCSI_ADDRESS>() as usize as ULONG_PTR;
    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaPdoSendHbaControl(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    SrbControl: PSRB_IO_CONTROL,
    BufferSize: ULONG,
) -> NTSTATUS {
    let mut Srb: PSCSI_REQUEST_BLOCK;
    let mut IoStatusBlock: _IO_STATUS_BLOCK;
    let mut Irp: PIRP;
    let mut Event: KEVENT;
    let mut Status: NTSTATUS;
    let mut LargeInt: LARGE_INTEGER;
    let mut IoStack: PIO_STACK_LOCATION;

    PAGED_CODE!();

    Srb = (ExAllocatePoolZero(NonPagedPool, (core::mem::size_of::<SCSI_REQUEST_BLOCK>()) as _, ATAPORT_TAG)).cast();
    if Srb.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, FALSE);

    LargeInt.QuadPart = 1;
    Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_SCSI,
        (*DevExt).Common.Self_,
        (SrbControl).cast(),
        BufferSize,
        &mut LargeInt,
        &mut Event,
        &mut IoStatusBlock,
    );
    if Irp.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        ExFreePoolWithTag((Srb).cast(), ATAPORT_TAG);
        return Status;
    }

    (*Srb).OriginalRequest = (Irp).cast();

    (*Srb).Function = (SRB_FUNCTION_IO_CONTROL) as u8;
    (*Srb).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;

    (*Srb).TargetId = (*DevExt).Device.AtaScsiAddress.TargetId;
    (*Srb).Lun = (*DevExt).Device.AtaScsiAddress.Lun;
    (*Srb).PathId = (*DevExt).Device.AtaScsiAddress.PathId;

    (*Srb).TimeOutValue = (*SrbControl).Timeout;

    (*Srb).SrbFlags = SRB_FLAGS_NO_QUEUE_FREEZE | SRB_FLAGS_DATA_IN;

    (*Srb).DataBuffer = (SrbControl).cast();
    (*Srb).DataTransferLength = BufferSize;

    IoStack = IoGetNextIrpStackLocation(Irp);
    (*IoStack).Parameters.Scsi.Srb = Srb;

    Status = IoCallDriver((*DevExt).Common.Self_, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            &mut Event,
            _KWAIT_REASON_Executive,
            (0i32) as KPROCESSOR_MODE,
            FALSE,
            core::ptr::null_mut(),
        );
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    ExFreePoolWithTag((Srb).cast(), ATAPORT_TAG);
    return Status;
}

unsafe extern "C" fn AtaPdoHandleStorageManageDataSetAttributes(
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
    mut Irp: PIRP,
    mut IoStack: PIO_STACK_LOCATION,
) -> NTSTATUS {
    if (KeGetCurrentIrql() as ::core::ffi::c_int) as u32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
                b"drivers/storage/ide/atapi/ioctl.c\0"
                    as *const u8 as *const ::core::ffi::c_char as PVOID,
                405 as ULONG,
                ::core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }
    return (*Irp).IoStatus.__bindgen_anon_1.Status;
}

unsafe extern "C" fn AtaPdoHandleScsiMiniport(
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
    mut Irp: PIRP,
    mut IoStack: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut SrbControl: PSRB_IO_CONTROL = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut CmdBufferSize: ULONG = 0;
    let mut BufferSize: ULONG = 0;

    PAGED_CODE!();

    if !AtaVerifyInBuffer!(IoStack, (core::mem::size_of::<SRB_IO_CONTROL>()) as _) {
        return STATUS_BUFFER_TOO_SMALL;
    }

    SrbControl = (*Irp).AssociatedIrp.SystemBuffer as PSRB_IO_CONTROL;
    if (*SrbControl).HeaderLength != core::mem::size_of::<SRB_IO_CONTROL>() as ULONG {
        return STATUS_REVISION_MISMATCH;
    }

    Status = RtlULongAdd(
        (*SrbControl).Length,
        core::mem::size_of::<SRB_IO_CONTROL>() as ULONG,
        &mut BufferSize,
    );
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    CmdBufferSize = (*IoStack).Parameters.DeviceIoControl.InputBufferLength;
    CmdBufferSize = max(
        CmdBufferSize,
        (*IoStack).Parameters.DeviceIoControl.OutputBufferLength,
    );
    if CmdBufferSize < BufferSize {
        return STATUS_BUFFER_TOO_SMALL;
    }

    if RtlEqualMemory(
        &mut (*SrbControl).Signature as *mut UCHAR as *const core::ffi::c_void,
        b"SCSIDISK\0".as_ptr() as *const core::ffi::c_void,
        (core::mem::size_of::<[core::ffi::c_char; 9]>() as usize).wrapping_sub(1),
    ) != false {
        Status = AtaPdoSendHbaControl(DevExt, SrbControl, BufferSize);
    } else {
        Status = STATUS_INVALID_PARAMETER;
    }

    return Status;
}

unsafe extern "C" fn AtaPdoHandleAtaPassthrough(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let ChanExt: PATAPORT_CHANNEL_EXTENSION = (*DevExt).Common.__bindgen_anon_1.FdoExt as PATAPORT_CHANNEL_EXTENSION;

    PAGED_CODE!();

    return SptiHandleAtaPassthru(
        (*DevExt).Common.Self_,
        Irp,
        (*ChanExt).PortData.MaximumTransferLength,
        (*ChanExt).PortData.MaximumPhysicalPages,
    );
}

unsafe extern "C" fn AtaPdoHandleScsiPassthrough(
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
    mut Irp: PIRP,
    mut IoStack: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut ChanExt: PATAPORT_CHANNEL_EXTENSION = core::ptr::null_mut();

    PAGED_CODE!();

    if (*IoStack).MinorFunction as i32 == 0
        && (*DevExt).DeviceClaimed as i32 != 0
    {
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    ChanExt = (*DevExt).Common.__bindgen_anon_1.FdoExt as PATAPORT_CHANNEL_EXTENSION;

    return SptiHandleScsiPassthru(
        (*DevExt).Common.Self_,
        Irp,
        (*ChanExt).PortData.MaximumTransferLength,
        (*ChanExt).PortData.MaximumPhysicalPages,
    );
}

unsafe extern "C" fn AtaPdoDeviceControl(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;
    let mut ForwardToFdo: BOOLEAN = FALSE;

    Status = IoAcquireRemoveLock(&mut (*DevExt).Common.RemoveLock, (Irp).cast());
    if !NT_SUCCESS!(Status) {
        (*Irp).IoStatus.Information = 0;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);

        return Status;
    }

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    match (*IoStack).Parameters.DeviceIoControl.IoControlCode {
        IOCTL_STORAGE_QUERY_PROPERTY => {
            Status = AtaPdoHandleStorageQueryProperty(DevExt, Irp, IoStack, &mut ForwardToFdo);
        }
        IOCTL_SCSI_GET_ADDRESS => {
            Status = AtaPdoHandleGetScsiAddress(DevExt, Irp, IoStack);
        }
        IOCTL_SCSI_MINIPORT => {
            Status = AtaPdoHandleScsiMiniport(DevExt, Irp, IoStack);
        }
        IOCTL_STORAGE_MANAGE_DATA_SET_ATTRIBUTES => {
            Status = AtaPdoHandleStorageManageDataSetAttributes(DevExt, Irp, IoStack);
        }
        IOCTL_ATA_PASS_THROUGH | IOCTL_ATA_PASS_THROUGH_DIRECT => {
            Status = AtaPdoHandleAtaPassthrough(DevExt, Irp);
        }
        IOCTL_SCSI_PASS_THROUGH | IOCTL_SCSI_PASS_THROUGH_DIRECT => {
            Status = AtaPdoHandleScsiPassthrough(DevExt, Irp, IoStack);
        }
        IOCTL_SCSI_GET_CAPABILITIES | IOCTL_SCSI_GET_INQUIRY_DATA => {
            ForwardToFdo = TRUE;
        }
        _ => {
            Status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    if ForwardToFdo != 0 {
        let ChanExt: PATAPORT_CHANNEL_EXTENSION = ((*DevExt).Common.__bindgen_anon_1.FdoExt).cast();

        IoSkipCurrentIrpStackLocation(Irp);
        Status = IoCallDriver((*ChanExt).Common.Self_, Irp);
    } else {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);
    }

    IoReleaseRemoveLock(&mut (*DevExt).Common.RemoveLock, (Irp).cast());

    return Status;
}

unsafe extern "C" fn AtaFdoQueryStorageAdapterProperty(
    ChanExt: PATAPORT_CHANNEL_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut AdapterDescriptor: PSTORAGE_ADAPTER_DESCRIPTOR = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    PAGED_CODE!();

    if AtaCheckPropertyQuery(Irp, core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR>() as u32, &mut Status) == 0 {
        return Status;
    }

    AdapterDescriptor = (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_ADAPTER_DESCRIPTOR;

    RtlZeroMemory!(
        AdapterDescriptor as *mut core::ffi::c_void,
        core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR>(),
    );

    (*AdapterDescriptor).Version = core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR>() as u32;
    (*AdapterDescriptor).Size = core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR>() as u32;
    (*AdapterDescriptor).MaximumTransferLength = (*ChanExt).PortData.MaximumTransferLength;
    (*AdapterDescriptor).MaximumPhysicalPages = (*ChanExt).PortData.MaximumPhysicalPages;
    (*AdapterDescriptor).AlignmentMask = (*(*ChanExt).Common.Self_).AlignmentRequirement;
    (*AdapterDescriptor).AdapterUsesPio = ((*ChanExt).PortData.PortFlags != 0 && PORT_FLAG_PIO_ONLY != 0) as i32 as u8;
    (*AdapterDescriptor).AdapterScansDown = FALSE;
    (*AdapterDescriptor).CommandQueueing = FALSE;
    (*AdapterDescriptor).AcceleratedTransfer = FALSE;
    if (*ChanExt).PortData.PortFlags & PORT_FLAG_IS_AHCI != 0 {
        (*AdapterDescriptor).BusType = (BusTypeSata) as u8;
    } else {
        (*AdapterDescriptor).BusType = (BusTypeAta) as u8;
    }
    (*AdapterDescriptor).BusMajorVersion = 1;
    (*AdapterDescriptor).BusMinorVersion = 0;
    (*AdapterDescriptor).SrbType = SRB_TYPE_SCSI_REQUEST_BLOCK;
    (*AdapterDescriptor).AddressType = STORAGE_ADDRESS_TYPE_BTL8;

    (*Irp).IoStatus.Information = (core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR>() as usize) as _;
    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaFdoHandleStorageQueryProperty(
    mut ChanExt: PATAPORT_CHANNEL_EXTENSION,
    mut Irp: PIRP,
    mut IoStack: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut PropertyQuery: PSTORAGE_PROPERTY_QUERY = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    PAGED_CODE!();

    if !AtaVerifyInBuffer!(IoStack, (core::mem::size_of::<STORAGE_PROPERTY_QUERY>()) as _) {
        return STATUS_BUFFER_TOO_SMALL;
    }

    PropertyQuery = (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_PROPERTY_QUERY;

    match (*PropertyQuery).PropertyId as u32 {
        StorageAdapterProperty => {
            Status = AtaFdoQueryStorageAdapterProperty(ChanExt, Irp);
        }
        _ => {
            Status = STATUS_NOT_SUPPORTED;
        }
    }

    return Status;
}

unsafe extern "C" fn AtaFdoHandleGetScsiCapabilities(
    mut ChanExt: PATAPORT_CHANNEL_EXTENSION,
    mut Irp: PIRP,
    mut IoStack: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut Capabilities: PIO_SCSI_CAPABILITIES = core::ptr::null_mut::<_IO_SCSI_CAPABILITIES>();

    if !((*IoStack).Parameters.DeviceIoControl.OutputBufferLength as usize
        >= core::mem::size_of::<_IO_SCSI_CAPABILITIES>() as usize)
    {
        return STATUS_BUFFER_TOO_SMALL;
    }

    Capabilities = (*Irp).AssociatedIrp.SystemBuffer as PIO_SCSI_CAPABILITIES;
    (*Capabilities).Length = core::mem::size_of::<_IO_SCSI_CAPABILITIES>() as ULONG;
    (*Capabilities).MaximumTransferLength = (*ChanExt).PortData.MaximumTransferLength;
    (*Capabilities).MaximumPhysicalPages = (*ChanExt).PortData.MaximumPhysicalPages;
    (*Capabilities).SupportedAsynchronousEvents = (FALSE) as _;
    (*Capabilities).AlignmentMask = (*(*ChanExt).Common.Self_).AlignmentRequirement;
    (*Capabilities).TaggedQueuing = FALSE;
    (*Capabilities).AdapterScansDown = FALSE;
    (*Capabilities).AdapterUsesPio = ((*ChanExt).PortData.PortFlags != 0 && PORT_FLAG_PIO_ONLY != 0)
        as core::ffi::c_int as BOOLEAN;
    (*Irp).IoStatus.Information =
        core::mem::size_of::<_IO_SCSI_CAPABILITIES>() as usize as ULONG_PTR;
    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaFdoHandleGetScsiInquiryData(
    mut ChanExt: PATAPORT_CHANNEL_EXTENSION,
    mut Irp: PIRP,
    mut IoStack: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut ScsiInquiryData: PSCSI_INQUIRY_DATA = core::ptr::null_mut();
    let mut ScsiAdapterBusInfo: PSCSI_ADAPTER_BUS_INFO = core::ptr::null_mut();
    let mut PdoCount: ULONG = 0;
    let mut EntrySize: ULONG = 0;
    let mut TotalSize: ULONG = 0;
    let mut AtaScsiAddress: ATA_SCSI_ADDRESS = ATA_SCSI_ADDRESS {
        AsULONG: 0,
    };

    PdoCount = 0;
    AtaScsiAddress.AsULONG = 0;
    loop {
        let mut DevExt: PATAPORT_DEVICE_EXTENSION = core::ptr::null_mut();
        DevExt = AtaFdoFindNextDeviceByPath(ChanExt, &mut AtaScsiAddress, FALSE, core::ptr::null_mut());
        if DevExt.is_null() {
            break;
        }
        PdoCount = PdoCount.wrapping_add(1);
    }

    EntrySize = ALIGN_UP!(
        core::mem::size_of::<SCSI_INQUIRY_DATA>().wrapping_sub(1).wrapping_add((INQUIRYDATABUFFERSIZE) as _),
        core::mem::size_of::<ULONG>() as ULONG,
    );
    TotalSize = (core::mem::size_of::<SCSI_ADAPTER_BUS_INFO>().wrapping_add((EntrySize.wrapping_mul(PdoCount)) as _)) as _;

    if !AtaVerifyOutBuffer!(IoStack, TotalSize) {
        return STATUS_BUFFER_TOO_SMALL;
    }

    (*Irp).IoStatus.Information = TotalSize as ULONG_PTR;

    ScsiAdapterBusInfo = (*Irp).AssociatedIrp.SystemBuffer as PSCSI_ADAPTER_BUS_INFO;
    (*ScsiAdapterBusInfo).NumberOfBuses = 1;
    (*(&mut (*ScsiAdapterBusInfo).BusData as *mut SCSI_BUS_DATA).offset(0 as isize)).NumberOfLogicalUnits = 0;
    (*(&mut (*ScsiAdapterBusInfo).BusData as *mut SCSI_BUS_DATA).offset(0 as isize)).InitiatorBusId = 0xFF;
    (*(&mut (*ScsiAdapterBusInfo).BusData as *mut SCSI_BUS_DATA).offset(0 as isize)).InquiryDataOffset = core::mem::size_of::<SCSI_ADAPTER_BUS_INFO>() as ULONG;

    ScsiInquiryData = ScsiAdapterBusInfo.offset(1 as isize) as PSCSI_INQUIRY_DATA;

    AtaScsiAddress.AsULONG = 0;
    loop {
        let mut DevExt: PATAPORT_DEVICE_EXTENSION = core::ptr::null_mut();
        let mut InquiryData: PINQUIRYDATA = core::ptr::null_mut();

        DevExt = AtaFdoFindNextDeviceByPath(ChanExt, &mut AtaScsiAddress, FALSE, (Irp).cast());
        if DevExt.is_null() {
            break;
        }

        AtaScsiAddress = (*DevExt).Device.AtaScsiAddress;

        (*ScsiInquiryData).PathId = AtaScsiAddress.PathId;
        (*ScsiInquiryData).TargetId = AtaScsiAddress.TargetId;
        (*ScsiInquiryData).Lun = AtaScsiAddress.Lun;
        (*ScsiInquiryData).InquiryDataLength = INQUIRYDATABUFFERSIZE;
        (*ScsiInquiryData).DeviceClaimed = (*DevExt).DeviceClaimed;
        (*ScsiInquiryData).NextInquiryDataOffset =
            (ScsiInquiryData as ULONG_PTR).wrapping_add(EntrySize as ULONG_PTR).wrapping_sub(ScsiAdapterBusInfo as ULONG_PTR) as ULONG;

        RtlCopyMemory!(
            &mut (*ScsiInquiryData).InquiryData as *mut UCHAR as *mut core::ffi::c_void,
            &raw const (*DevExt).InquiryData as *const core::ffi::c_void,
            INQUIRYDATABUFFERSIZE,
        );

        InquiryData = &mut (*ScsiInquiryData).InquiryData as *mut UCHAR as PINQUIRYDATA;
        (*InquiryData).AdditionalLength =
            (INQUIRYDATABUFFERSIZE - RTL_SIZEOF_THROUGH_FIELD!(INQUIRYDATA, AdditionalLength)) as u8;

        IoReleaseRemoveLock(&mut (*DevExt).Common.RemoveLock, (Irp).cast());

        ScsiInquiryData = (ScsiInquiryData as ULONG_PTR).wrapping_add(EntrySize as ULONG_PTR) as PSCSI_INQUIRY_DATA;

        let ref mut fresh = (*(&mut (*ScsiAdapterBusInfo).BusData as *mut SCSI_BUS_DATA).offset(0 as isize)).NumberOfLogicalUnits;
        *fresh = (*fresh).wrapping_add(1);
        if *fresh as ULONG >= PdoCount {
            break;
        }
    }

    if (*(&mut (*ScsiAdapterBusInfo).BusData as *mut SCSI_BUS_DATA).offset(0 as isize)).NumberOfLogicalUnits as i32 != 0 {
        ScsiInquiryData = (ScsiInquiryData as ULONG_PTR).wrapping_sub(EntrySize as ULONG_PTR) as PSCSI_INQUIRY_DATA;
        (*ScsiInquiryData).NextInquiryDataOffset = 0;
    } else {
        (*(&mut (*ScsiAdapterBusInfo).BusData as *mut SCSI_BUS_DATA).offset(0 as isize)).InquiryDataOffset = 0;
    }

    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaFdoHandleScsiMiniport(
    mut ChanExt: PATAPORT_CHANNEL_EXTENSION,
    mut Irp: PIRP,
    mut IoStack: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut DevExt: PATAPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    PAGED_CODE!();

    DevExt = (AtaFdoFindDeviceByPath(
        (ChanExt).cast(),
        AtaMarshallScsiAddress(0, 0, 0),
        (Irp).cast(),
    )).cast();
    if DevExt.is_null() {
        return STATUS_NO_SUCH_DEVICE;
    }

    Status = AtaPdoHandleScsiMiniport(DevExt, Irp, IoStack);

    IoReleaseRemoveLock(&raw mut (*DevExt).Common.RemoveLock, (Irp).cast());

    return Status;
}

unsafe extern "C" fn AtaFdoHandleScsiPassthrough(
    ChanExt: PATAPORT_CHANNEL_EXTENSION,
    Irp: PIRP,
    IoStack: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut Spt: PSCSI_PASS_THROUGH = core::ptr::null_mut();
    let mut DevExt: PATAPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    PAGED_CODE!();

    if !AtaVerifyInBuffer!(IoStack, RTL_SIZEOF_THROUGH_FIELD!(SCSI_PASS_THROUGH, Lun)) {
        return STATUS_BUFFER_TOO_SMALL;
    }

    Spt = (*Irp).AssociatedIrp.SystemBuffer as PSCSI_PASS_THROUGH;

    DevExt = (AtaFdoFindDeviceByPath(
        (ChanExt).cast(),
        AtaMarshallScsiAddress(
            (*Spt).PathId as ULONG,
            (*Spt).TargetId as ULONG,
            (*Spt).Lun as ULONG,
        ),
        Irp as PVOID,
    )).cast();
    if DevExt.is_null() {
        return STATUS_NO_SUCH_DEVICE;
    }

    Status = AtaPdoHandleScsiPassthrough(DevExt, Irp, IoStack);

    IoReleaseRemoveLock(&raw mut (*DevExt).Common.RemoveLock, Irp as PVOID);

    return Status;
}

unsafe extern "C" fn AtaFdoDeviceControl(
    ChanExt: PATAPORT_CHANNEL_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;

    Status = IoAcquireRemoveLock(&mut (*ChanExt).Common.RemoveLock, (Irp).cast());
    if !NT_SUCCESS!(Status) {
        (*Irp).IoStatus.Information = 0;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);

        return Status;
    }

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    match (*IoStack).Parameters.DeviceIoControl.IoControlCode {
        IOCTL_STORAGE_QUERY_PROPERTY => {
            Status = AtaFdoHandleStorageQueryProperty(ChanExt, Irp, IoStack);
        }
        IOCTL_SCSI_GET_CAPABILITIES => {
            Status = AtaFdoHandleGetScsiCapabilities(ChanExt, Irp, IoStack);
        }
        IOCTL_SCSI_GET_INQUIRY_DATA => {
            Status = AtaFdoHandleGetScsiInquiryData(ChanExt, Irp, IoStack);
        }
        IOCTL_SCSI_MINIPORT => {
            Status = AtaFdoHandleScsiMiniport(ChanExt, Irp, IoStack);
        }
        IOCTL_SCSI_PASS_THROUGH | IOCTL_SCSI_PASS_THROUGH_DIRECT => {
            Status = AtaFdoHandleScsiPassthrough(ChanExt, Irp, IoStack);
        }
        IOCTL_SCSI_RESCAN_BUS => {
            IoInvalidateDeviceRelations((*ChanExt).Pdo, BusRelations);
            Status = STATUS_SUCCESS;
        }
        _ => {
            IoSkipCurrentIrpStackLocation(Irp);
            Status = IoCallDriver((*ChanExt).Common.__bindgen_anon_1.LowerDeviceObject, Irp);

            IoReleaseRemoveLock(&mut (*ChanExt).Common.RemoveLock, (Irp).cast());

            return Status;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);

    IoReleaseRemoveLock(&mut (*ChanExt).Common.RemoveLock, (Irp).cast());

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn AtaDispatchDeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    if IS_FDO!((*DeviceObject).DeviceExtension) != false {
        return AtaFdoDeviceControl(((*DeviceObject).DeviceExtension).cast(), Irp);
    } else {
        return AtaPdoDeviceControl(((*DeviceObject).DeviceExtension).cast(), Irp);
    }
}
