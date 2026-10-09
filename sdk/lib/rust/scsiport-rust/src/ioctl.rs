#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::inlines::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{LPCSTR, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, PSTORAGE_DESCRIPTOR_HEADER, PSTORAGE_DEVICE_DESCRIPTOR, PSTORAGE_PROPERTY_QUERY, STORAGE_DESCRIPTOR_HEADER, STORAGE_DEVICE_DESCRIPTOR, STORAGE_PROPERTY_QUERY, UINT32, UINT8, _STORAGE_BUS_TYPE_BusTypeScsi as BusTypeScsi, _STORAGE_PROPERTY_ID_StorageAdapterProperty as StorageAdapterProperty, _STORAGE_PROPERTY_ID_StorageDeviceIdProperty as StorageDeviceIdProperty, _STORAGE_PROPERTY_ID_StorageDeviceProperty as StorageDeviceProperty, _STORAGE_QUERY_TYPE_PropertyExistsQuery as PropertyExistsQuery, _STORAGE_QUERY_TYPE_PropertyStandardQuery as PropertyStandardQuery};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{DUMP_POINTERS, IO_SCSI_CAPABILITIES, PDUMP_POINTERS, PINQUIRYDATA, PSCSI_ADAPTER_BUS_INFO, PSCSI_ADDRESS, PSCSI_BUS_DATA, PSCSI_BUS_INFO, PSCSI_INQUIRY_DATA, PSCSI_PASS_THROUGH, PSCSI_PORT_COMMON_EXTENSION, PSCSI_PORT_DEVICE_EXTENSION, PSCSI_PORT_LUN_EXTENSION, SCSI_ADAPTER_BUS_INFO, SCSI_ADDRESS, SCSI_BUS_DATA, SCSI_INQUIRY_DATA, SCSI_PORT_DEVICE_EXTENSION, SCSI_PORT_LUN_EXTENSION, STORAGE_ADAPTER_DESCRIPTOR_WIN8, _SCSI_PORT_DEVICE_EXTENSION};
// resolve_imports.py: explicit kernel/private-type imports
// resolve_imports.py: explicit kernel/private-type imports

extern "C" {
    fn GetLunByPath( portExt: PSCSI_PORT_DEVICE_EXTENSION, pathId: u8, targetId: u8, lun: u8, ) -> PSCSI_PORT_LUN_EXTENSION;
    fn DbgPrint(format: *const i8, ...) -> i32;
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: bool = true;
unsafe fn SpiGetInquiryData(
    DeviceExtension: PSCSI_PORT_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut InquiryDataSize: ULONG;
    let mut BusCount: ULONG;
    let mut Length: ULONG;
    let IrpStack: PIO_STACK_LOCATION;
    let mut AdapterBusInfo: PSCSI_ADAPTER_BUS_INFO;
    let mut InquiryData: PSCSI_INQUIRY_DATA;
    let Buffer: PUCHAR;

    IrpStack = IoGetCurrentIrpStackLocation(Irp);
    Buffer = (*Irp).AssociatedIrp.SystemBuffer as PUCHAR;
    BusCount = (*DeviceExtension).NumberOfBuses as ULONG;

    InquiryDataSize = ((core::mem::size_of::<SCSI_INQUIRY_DATA>() as usize)
        .wrapping_sub(1usize)
        .wrapping_add(36usize) // INQUIRYDATABUFFERSIZE
        .wrapping_add(core::mem::size_of::<ULONG>() as usize)
        .wrapping_sub(1usize) as u64
        & !(core::mem::size_of::<ULONG>() as u64).wrapping_sub(1u64)) as ULONG;

    Length = (core::mem::size_of::<SCSI_ADAPTER_BUS_INFO>() as usize).wrapping_add(
        ((BusCount as u64).wrapping_sub(1u64) as usize)
            .wrapping_mul(core::mem::size_of::<SCSI_BUS_DATA>() as usize),
    ) as ULONG;
    Length = (Length as u64).wrapping_add(
        (InquiryDataSize as u64).wrapping_mul((*DeviceExtension).TotalLUCount as u64),
    ) as ULONG;

    if (*IrpStack).Parameters.DeviceIoControl.OutputBufferLength < Length {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_BUFFER_TOO_SMALL;
        return STATUS_BUFFER_TOO_SMALL;
    }

    (*Irp).IoStatus.Information = (Length as usize) as _;

    AdapterBusInfo = Buffer as PSCSI_ADAPTER_BUS_INFO;
    (*AdapterBusInfo).NumberOfBuses = BusCount as UCHAR;

    InquiryData = Buffer
        .offset(core::mem::size_of::<SCSI_ADAPTER_BUS_INFO>() as isize)
        .offset(
            ((BusCount as u64).wrapping_sub(1u64) as usize)
                .wrapping_mul(core::mem::size_of::<SCSI_BUS_DATA>() as usize) as isize,
        ) as PSCSI_INQUIRY_DATA;

    let mut pathId: UINT8 = 0;
    while (pathId as i32) < ((*DeviceExtension).NumberOfBuses as i32) {
        let BusData: PSCSI_BUS_DATA = (&raw mut (*AdapterBusInfo).BusData as *mut SCSI_BUS_DATA)
            .offset(pathId as isize) as PSCSI_BUS_DATA;

        (*BusData).InquiryDataOffset =
            (InquiryData as PUCHAR).offset_from(Buffer) as i64 as ULONG;

        (*BusData).InitiatorBusId =
            (*(*DeviceExtension).Buses.offset(pathId as isize)).BusIdentifier;

        (*BusData).NumberOfLogicalUnits =
            (*(*DeviceExtension).Buses.offset(pathId as isize)).LogicalUnitsCount;

        let bus: PSCSI_BUS_INFO =
            (*DeviceExtension).Buses.offset(pathId as isize) as PSCSI_BUS_INFO;

        let mut lunEntry: PLIST_ENTRY = (*bus).LunsListHead.Flink as PLIST_ENTRY;
        while lunEntry != &raw mut (*bus).LunsListHead {
            let lunExt: PSCSI_PORT_LUN_EXTENSION = (lunEntry as usize).wrapping_sub(
                core::mem::offset_of!(SCSI_PORT_LUN_EXTENSION, LunEntry),
            ) as PSCSI_PORT_LUN_EXTENSION;

            (*InquiryData).PathId = (*lunExt).PathId;
            (*InquiryData).TargetId = (*lunExt).TargetId;
            (*InquiryData).Lun = (*lunExt).Lun;
            (*InquiryData).InquiryDataLength = 36; // INQUIRYDATABUFFERSIZE
            (*InquiryData).DeviceClaimed = (*lunExt).DeviceClaimed;

            (*InquiryData).NextInquiryDataOffset =
                (InquiryData as PUCHAR)
                    .offset(InquiryDataSize as isize)
                    .offset_from(Buffer) as i64 as ULONG;

            RtlCopyMemory(
                (&raw mut (*InquiryData).InquiryData) as *mut core::ffi::c_void,
                (&raw mut (*lunExt).InquiryData) as *const core::ffi::c_void,
                36usize, // INQUIRYDATABUFFERSIZE
            );

            InquiryData = (InquiryData as usize).wrapping_add(InquiryDataSize as usize) as PSCSI_INQUIRY_DATA;

            lunEntry = (*lunEntry).Flink as PLIST_ENTRY;
        }

        if (*BusData).NumberOfLogicalUnits as i32 != 0 {
            (*((InquiryData as *mut i8).offset(-(InquiryDataSize as isize)) as PSCSI_INQUIRY_DATA))
                .NextInquiryDataOffset = 0;
        } else {
            (*BusData).InquiryDataOffset = 0;
        }

        pathId = pathId.wrapping_add(1);
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    return STATUS_SUCCESS;
}

unsafe fn GetFieldLength(Name: PUCHAR, MaxLength: UINT32) -> UINT32 {
    let mut Index: UINT32;
    let mut LastCharacterPosition: UINT32 = 0;
    Index = 0;
    while Index < MaxLength {
        if *Name.add(Index as usize) != b' ' {
            LastCharacterPosition = Index;
        }
        Index = Index.wrapping_add(1);
    }
    LastCharacterPosition.wrapping_add(1)
}
unsafe fn PdoHandleQueryProperty(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let ioStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let lunExt: PSCSI_PORT_LUN_EXTENSION =
        (*DeviceObject).DeviceExtension as PSCSI_PORT_LUN_EXTENSION;
    let mut status: NTSTATUS;

    if !((*ioStack).Parameters.DeviceIoControl.InputBufferLength as usize
        >= core::mem::size_of::<STORAGE_PROPERTY_QUERY>())
    {
        ((RtlAssert))(
            b"ioStack->Parameters.DeviceIoControl.InputBufferLength >= sizeof(STORAGE_PROPERTY_QUERY)\0"
                .as_ptr()
                .cast::<i8>() as *const core::ffi::c_void,
            b"drivers/storage/port/scsiport/ioctl.c\0"
                .as_ptr()
                .cast::<i8>() as *const core::ffi::c_void,
            156,
            core::ptr::null_mut::<i8>(),
        );
    }
    if (*Irp).AssociatedIrp.SystemBuffer.is_null() {
        ((RtlAssert))(
            b"Irp->AssociatedIrp.SystemBuffer\0"
                .as_ptr()
                .cast::<i8>() as *const core::ffi::c_void,
            b"drivers/storage/port/scsiport/ioctl.c\0"
                .as_ptr()
                .cast::<i8>() as *const core::ffi::c_void,
            157,
            core::ptr::null_mut::<i8>(),
        );
    }
    if (*lunExt).Common.IsFDO != 0 {
        ((RtlAssert))(
            b"!lunExt->Common.IsFDO\0"
                .as_ptr()
                .cast::<i8>() as *const core::ffi::c_void,
            b"drivers/storage/port/scsiport/ioctl.c\0"
                .as_ptr()
                .cast::<i8>() as *const core::ffi::c_void,
            158,
            core::ptr::null_mut::<i8>(),
        );
    }

    let PropertyQuery: PSTORAGE_PROPERTY_QUERY =
        (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_PROPERTY_QUERY;

    if (*PropertyQuery).PropertyId != StorageDeviceProperty
        && (*PropertyQuery).PropertyId != StorageAdapterProperty
    {
        status = STATUS_INVALID_PARAMETER_1;
    } else if (*PropertyQuery).QueryType == PropertyExistsQuery {
        status = STATUS_SUCCESS;
    } else if (*PropertyQuery).QueryType != PropertyStandardQuery {
        status = STATUS_INVALID_PARAMETER_2;
    } else {
        match (*PropertyQuery).PropertyId {
            StorageDeviceProperty => {
                let inquiryData: PINQUIRYDATA = &raw mut (*lunExt).InquiryData;
                let FieldLengthVendor: UINT32 =
                    GetFieldLength(
                        &raw mut (*inquiryData).VendorId as *mut u8,
                        8
                    );
                let FieldLengthProduct: UINT32 =
                    GetFieldLength(
                        &raw mut (*inquiryData).ProductId as *mut u8,
                        16
                    );
                let FieldLengthRevision: UINT32 =
                    GetFieldLength(
                        &raw mut (*inquiryData).ProductRevisionLevel as *mut u8,
                        4
                    );

                let TotalLength: UINT32 = (core::mem::size_of::<STORAGE_DEVICE_DESCRIPTOR>()
                    + FieldLengthVendor as usize
                    + FieldLengthProduct as usize
                    + FieldLengthRevision as usize
                    + 3) as u32;

                if (*ioStack).Parameters.DeviceIoControl.OutputBufferLength < (TotalLength as u64) as u32 {
                    let DescriptorHeader: PSTORAGE_DESCRIPTOR_HEADER =
                        (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_DESCRIPTOR_HEADER;
                    if !((*ioStack).Parameters.DeviceIoControl.OutputBufferLength as usize
                        >= core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>())
                    {
                        ((RtlAssert))(
                            b"ioStack->Parameters.DeviceIoControl.OutputBufferLength >= sizeof(STORAGE_DESCRIPTOR_HEADER)\0"
                                .as_ptr()
                                .cast::<i8>() as *const core::ffi::c_void,
                            b"drivers/storage/port/scsiport/ioctl.c\0"
                                .as_ptr()
                                .cast::<i8>() as *const core::ffi::c_void,
                            211,
                            core::ptr::null_mut::<i8>(),
                        );
                    }
                    (*DescriptorHeader).Version = TotalLength;
                    (*DescriptorHeader).Size = TotalLength;
                    (*Irp).IoStatus.Information =
                        (core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as usize) as _;
                    status = STATUS_SUCCESS;
                } else {
                    let deviceDescriptor: PSTORAGE_DEVICE_DESCRIPTOR =
                        (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_DEVICE_DESCRIPTOR;
                    (*deviceDescriptor).Version =
                        core::mem::size_of::<STORAGE_DEVICE_DESCRIPTOR>() as u32;
                    (*deviceDescriptor).Size = TotalLength;
                    (*deviceDescriptor).DeviceType = (*inquiryData).DeviceType();
                    (*deviceDescriptor).DeviceTypeModifier = (*inquiryData).DeviceTypeModifier();
                    (*deviceDescriptor).RemovableMedia = (*inquiryData).RemovableMedia() as u8;
                    (*deviceDescriptor).CommandQueueing = (*inquiryData).CommandQueue() as u8;
                    (*deviceDescriptor).BusType = BusTypeScsi;
                    (*deviceDescriptor).VendorIdOffset =
                        core::mem::offset_of!(STORAGE_DEVICE_DESCRIPTOR, RawDeviceProperties) as u32;
                    (*deviceDescriptor).ProductIdOffset = (*deviceDescriptor).VendorIdOffset
                        + FieldLengthVendor
                        + 1;
                    (*deviceDescriptor).ProductRevisionOffset = (*deviceDescriptor).ProductIdOffset
                        + FieldLengthProduct
                        + 1;
                    (*deviceDescriptor).SerialNumberOffset = 0;
                    (*deviceDescriptor).RawPropertiesLength = FieldLengthVendor
                        + FieldLengthProduct
                        + FieldLengthRevision
                        + 3;

                    let mut Buffer: *mut u8 = &raw mut (*deviceDescriptor).RawDeviceProperties as *mut u8;
                    core::ptr::copy_nonoverlapping(
                        &(*inquiryData).VendorId as *const u8,
                        Buffer,
                        FieldLengthVendor as usize,
                    );
                    *Buffer.add(FieldLengthVendor as usize) = b'\0';
                    Buffer = Buffer.add(FieldLengthVendor as usize + 1);

                    core::ptr::copy_nonoverlapping(
                        &(*inquiryData).ProductId as *const u8,
                        Buffer,
                        FieldLengthProduct as usize,
                    );
                    *Buffer.add(FieldLengthProduct as usize) = b'\0';
                    Buffer = Buffer.add(FieldLengthProduct as usize + 1);

                    core::ptr::copy_nonoverlapping(
                        &(*inquiryData).ProductRevisionLevel as *const u8,
                        Buffer,
                        FieldLengthRevision as usize,
                    );
                    *Buffer.add(FieldLengthRevision as usize) = b'\0';
                    // Buffer += FieldLengthRevision + 1 not needed further

                    (*Irp).IoStatus.Information = (TotalLength as usize) as _;
                    status = STATUS_SUCCESS;
                }
            }
            StorageAdapterProperty => {
                IoSkipCurrentIrpStackLocation(Irp);
                return IoCallDriver((*lunExt).Common.LowerDevice, Irp);
            }
            StorageDeviceIdProperty | _ => {
                status = STATUS_NOT_IMPLEMENTED;
            }
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    return status;
}

unsafe extern "C" {
    fn SptiHandleScsiPassthru(
        DeviceObject: PDEVICE_OBJECT,
        Irp: PIRP,
        MaximumTransferLength: UINT32,
        MaximumPhysicalPages: UINT32,
    ) -> NTSTATUS;
}
unsafe fn PdoHandleScsiPassthrough(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let lunExt: PSCSI_PORT_LUN_EXTENSION = (*DeviceObject).DeviceExtension as PSCSI_PORT_LUN_EXTENSION;
    let mut portExt: PSCSI_PORT_DEVICE_EXTENSION;

    if !((KeGetCurrentIrql() as i32) < (DISPATCH_LEVEL as i32)) {
        RtlAssert(
            b"KeGetCurrentIrql() < DISPATCH_LEVEL\0".as_ptr(),
            b"drivers/storage/port/scsiport/ioctl.c\0".as_ptr(),
            300,
            core::ptr::null_mut::<u8>(),
        );
    }

    if (*lunExt).Common.IsFDO != 0 {
        RtlAssert(
            b"!lunExt->Common.IsFDO\0".as_ptr(),
            b"drivers/storage/port/scsiport/ioctl.c\0".as_ptr(),
            301,
            core::ptr::null_mut::<u8>(),
        );
    }

    if (*IoGetCurrentIrpStackLocation(Irp)).MinorFunction as i32 == 0
        && (*lunExt).DeviceClaimed as i32 != 0
    {
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    portExt = (*(*lunExt).Common.LowerDevice).DeviceExtension as PSCSI_PORT_DEVICE_EXTENSION;

    return SptiHandleScsiPassthru(
        DeviceObject,
        Irp,
        (*portExt).PortCapabilities.MaximumTransferLength,
        (*portExt).PortCapabilities.MaximumPhysicalPages,
    );
}
unsafe fn FdoHandleScsiPassthrough(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {

    let mut portExt: PSCSI_PORT_DEVICE_EXTENSION;
    let mut lunExt: PSCSI_PORT_LUN_EXTENSION;
    let mut spt: PSCSI_PASS_THROUGH;

    if !((KeGetCurrentIrql() as i32) < 2i32) {
        ((RtlAssert))(
            b"KeGetCurrentIrql() < DISPATCH_LEVEL\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            b"drivers/storage/port/scsiport/ioctl.c\0"
                .as_ptr() as *const i8 as *mut core::ffi::c_void,
            325u32,
            core::ptr::null_mut::<i8>(),
        );
    } else {
    }

    if VerifyIrpInBufferSize(
        Irp,
        (5u64 as i32 as usize).wrapping_add(core::mem::size_of::<u8>()),
    ) == 0
    {
        return STATUS_BUFFER_TOO_SMALL;
    }

    portExt = (*DeviceObject).DeviceExtension as PSCSI_PORT_DEVICE_EXTENSION;

    if (*portExt).Common.IsFDO == 0 {
        ((RtlAssert))(
            b"portExt->Common.IsFDO\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            b"drivers/storage/port/scsiport/ioctl.c\0"
                .as_ptr() as *const i8 as *mut core::ffi::c_void,
            331u32,
            core::ptr::null_mut::<i8>(),
        );
    } else {
    }

    spt = (*Irp).AssociatedIrp.SystemBuffer as PSCSI_PASS_THROUGH;
    lunExt = GetLunByPath(portExt, (*spt).PathId, (*spt).TargetId, (*spt).Lun);
    if lunExt.is_null() {
        return STATUS_NO_SUCH_DEVICE;
    }

    return PdoHandleScsiPassthrough((*lunExt).Common.DeviceObject, Irp);
}

unsafe fn FdoHandleQueryProperty(
    DeviceObject: *mut ntkrnl_sys::ddk::DEVICE_OBJECT,
    Irp: *mut ntkrnl_sys::ddk::IRP,
) -> i32 {
    let mut adapterDescriptor: *mut STORAGE_ADAPTER_DESCRIPTOR_WIN8;
    let ioStack = IoGetCurrentIrpStackLocation(Irp);
    let portExt = (*DeviceObject).DeviceExtension as *mut SCSI_PORT_DEVICE_EXTENSION;
    let mut status: i32;

    if ((*ioStack).Parameters.DeviceIoControl.InputBufferLength as usize)
        < core::mem::size_of::<STORAGE_PROPERTY_QUERY>() as usize
    {
        RtlAssert(
            b"ioStack->Parameters.DeviceIoControl.InputBufferLength >= sizeof(STORAGE_PROPERTY_QUERY)\0".as_ptr(),
            b"drivers/storage/port/scsiport/ioctl.c\0".as_ptr(),
            351,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    }
    if (*Irp).AssociatedIrp.SystemBuffer.is_null() {
        RtlAssert(
            b"Irp->AssociatedIrp.SystemBuffer\0".as_ptr(),
            b"drivers/storage/port/scsiport/ioctl.c\0".as_ptr(),
            352,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    }
    if (*portExt).Common.IsFDO == 0 {
        RtlAssert(
            b"portExt->Common.IsFDO\0".as_ptr(),
            b"drivers/storage/port/scsiport/ioctl.c\0".as_ptr(),
            353,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    }

    let PropertyQuery = (*Irp).AssociatedIrp.SystemBuffer as *mut STORAGE_PROPERTY_QUERY;

    if (*PropertyQuery).PropertyId as u32 != crate::glue::StorageAdapterProperty as u32 {
        if (*PropertyQuery).PropertyId as u32 == crate::glue::StorageDeviceProperty as u32
            || (*PropertyQuery).PropertyId as u32 == crate::glue::StorageDeviceIdProperty as u32
        {
            status = ntkrnl_sys::constants::STATUS_INVALID_DEVICE_REQUEST;
        } else {
            status = ntkrnl_sys::constants::STATUS_INVALID_PARAMETER_1;
        }
    } else if (*PropertyQuery).QueryType as u32 == crate::glue::PropertyExistsQuery as u32 {
        status = ntkrnl_sys::base::STATUS_SUCCESS;
    } else if (*PropertyQuery).QueryType as u32 != crate::glue::PropertyStandardQuery as u32 {
        status = ntkrnl_sys::constants::STATUS_INVALID_PARAMETER_2;
    } else if ((*ioStack).Parameters.DeviceIoControl.OutputBufferLength as usize)
        < core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR_WIN8>() as usize
    {
        let DescriptorHeader =
            (*Irp).AssociatedIrp.SystemBuffer as *mut STORAGE_DESCRIPTOR_HEADER;
        if ((*ioStack).Parameters.DeviceIoControl.OutputBufferLength as usize)
            < core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as usize
        {
            RtlAssert(
                b"ioStack->Parameters.DeviceIoControl.OutputBufferLength >= sizeof(STORAGE_DESCRIPTOR_HEADER)\0".as_ptr(),
                b"drivers/storage/port/scsiport/ioctl.c\0".as_ptr(),
                393,
                core::ptr::null_mut::<core::ffi::c_void>(),
            );
        }
        (*DescriptorHeader).Version =
            core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR_WIN8>() as u32;
        (*DescriptorHeader).Size =
            core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR_WIN8>() as u32;
        (*Irp).IoStatus.Information =
            (core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as usize as usize) as _; // ULONG_PTR
        status = ntkrnl_sys::base::STATUS_SUCCESS;
    } else {
        adapterDescriptor = (*Irp).AssociatedIrp.SystemBuffer as *mut STORAGE_ADAPTER_DESCRIPTOR_WIN8;
        *adapterDescriptor = STORAGE_ADAPTER_DESCRIPTOR_WIN8 {
            Version: core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR_WIN8>() as u32,
            Size: core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR_WIN8>() as u32,
            MaximumTransferLength: (*portExt).PortCapabilities.MaximumTransferLength,
            MaximumPhysicalPages: (*portExt).PortCapabilities.MaximumPhysicalPages,
            AlignmentMask: (*portExt).PortCapabilities.AlignmentMask,
            AdapterUsesPio: (*portExt).PortCapabilities.AdapterUsesPio,
            AdapterScansDown: (*portExt).PortCapabilities.AdapterScansDown,
            CommandQueueing: (*portExt).PortCapabilities.TaggedQueuing,
            AcceleratedTransfer: 1, // TRUE
            BusType: crate::glue::BusTypeScsi as u8,
            BusMajorVersion: 2u16,
            BusMinorVersion: 0u16,
            SrbType: 0,
            AddressType: 0,
        };
        (*Irp).IoStatus.Information =
            (core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR_WIN8>() as usize as usize) as _;
        status = ntkrnl_sys::base::STATUS_SUCCESS;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    ntkrnl_sys::driver_ddk::IoCompleteRequest(
        Irp,
        ntkrnl_sys::constants::IO_NO_INCREMENT as i8,
    );
    return status;
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortDeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Stack: PIO_STACK_LOCATION;
    let comExt: PSCSI_PORT_COMMON_EXTENSION = (*DeviceObject).DeviceExtension as PSCSI_PORT_COMMON_EXTENSION;
    let mut portExt: PSCSI_PORT_DEVICE_EXTENSION;
    let mut lunExt: PSCSI_PORT_LUN_EXTENSION;
    let IoControlCode: u32;
    let mut status: NTSTATUS;

    (*Irp).IoStatus.Information = 0;

    Stack = IoGetCurrentIrpStackLocation(Irp);
    IoControlCode = (*Stack).Parameters.DeviceIoControl.IoControlCode;

    match IoControlCode {
        2954240 /* IOCTL_STORAGE_QUERY_PROPERTY */ => {
            if VerifyIrpInBufferSize(Irp, core::mem::size_of::<STORAGE_PROPERTY_QUERY>() as usize) == 0 {
                status = STATUS_BUFFER_TOO_SMALL;
            } else if (*comExt).IsFDO != 0 {
                return FdoHandleQueryProperty(DeviceObject, Irp);
            } else {
                return PdoHandleQueryProperty(DeviceObject, Irp);
            }
        }
        266264 /* IOCTL_SCSI_GET_ADDRESS */ => {
            if (*comExt).IsFDO != 0 {
                status = STATUS_INVALID_DEVICE_REQUEST;
            } else {
                let address: PSCSI_ADDRESS = (*Irp).AssociatedIrp.SystemBuffer as PSCSI_ADDRESS;
                if VerifyIrpOutBufferSize(Irp, core::mem::size_of::<SCSI_ADDRESS>() as usize) == 0 {
                    status = STATUS_BUFFER_TOO_SMALL;
                } else {
                    lunExt = (*DeviceObject).DeviceExtension as PSCSI_PORT_LUN_EXTENSION;
                    portExt = (*(*comExt).LowerDevice).DeviceExtension as PSCSI_PORT_DEVICE_EXTENSION;
                    (*address).Length = core::mem::size_of::<SCSI_ADDRESS>() as u32;
                    (*address).PortNumber = (*portExt).PortNumber as u8;
                    (*address).PathId = (*lunExt).PathId;
                    (*address).TargetId = (*lunExt).TargetId;
                    (*address).Lun = (*lunExt).Lun;
                    (*Irp).IoStatus.Information = (core::mem::size_of::<SCSI_ADDRESS>() as usize) as _;
                    status = STATUS_SUCCESS;
                }
            }
        }
        266272 /* IOCTL_SCSI_GET_DUMP_POINTERS */ => {
            if (*comExt).IsFDO == 0 {
                IoSkipCurrentIrpStackLocation(Irp);
                return IoCallDriver((*comExt).LowerDevice, Irp);
            }
            let dumpPointers: PDUMP_POINTERS = (*Irp).AssociatedIrp.SystemBuffer as PDUMP_POINTERS;
            if VerifyIrpOutBufferSize(Irp, core::mem::size_of::<DUMP_POINTERS>() as usize) == 0 {
                status = STATUS_BUFFER_TOO_SMALL;
            } else {
                (*dumpPointers).DeviceObject = DeviceObject as *mut core::ffi::c_void;
                status = STATUS_SUCCESS;
                (*Irp).IoStatus.Information = (core::mem::size_of::<DUMP_POINTERS>() as usize) as _;
            }
        }
        266256 /* IOCTL_SCSI_GET_CAPABILITIES */ => {
            if (*comExt).IsFDO == 0 {
                status = STATUS_INVALID_DEVICE_REQUEST;
            } else if VerifyIrpOutBufferSize(Irp, core::mem::size_of::<IO_SCSI_CAPABILITIES>() as usize) == 0 {
                status = STATUS_BUFFER_TOO_SMALL;
            } else {
                portExt = (*DeviceObject).DeviceExtension as PSCSI_PORT_DEVICE_EXTENSION;
                RtlCopyMemory(
                    (*Irp).AssociatedIrp.SystemBuffer as *mut core::ffi::c_void,
                    &raw const (*portExt).PortCapabilities as *const core::ffi::c_void,
                    core::mem::size_of::<IO_SCSI_CAPABILITIES>() as usize,
                );
                status = STATUS_SUCCESS;
                (*Irp).IoStatus.Information = (core::mem::size_of::<IO_SCSI_CAPABILITIES>() as usize) as _;
            }
        }
        266252 /* IOCTL_SCSI_GET_INQUIRY_DATA */ => {
            if (*comExt).IsFDO == 0 {
                status = STATUS_INVALID_DEVICE_REQUEST;
            } else {
                status = SpiGetInquiryData(
                    (*DeviceObject).DeviceExtension as PSCSI_PORT_DEVICE_EXTENSION,
                    Irp,
                );
            }
        }
        315400 /* IOCTL_SCSI_MINIPORT */ => {
            if DbgPrint(
                b"(%s:%d) IOCTL_SCSI_MINIPORT unimplemented!\n\0" as *const u8 as *const i8,
                b"drivers/storage/port/scsiport/ioctl.c\0"
                    as *const u8 as *const i8,
                584i32,
            ) != 0 {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                    b"drivers/storage/port/scsiport/ioctl.c\0"
                        as *const u8 as *const i8,
                    584i32,
                );
            }
            status = STATUS_NOT_IMPLEMENTED;
        }
        315396 | 315412 /* IOCTL_SCSI_PASS_THROUGH | IOCTL_SCSI_PASS_THROUGH_DIRECT */ => {
            if (*comExt).IsFDO != 0 {
                status = FdoHandleScsiPassthrough(DeviceObject, Irp);
            } else {
                status = PdoHandleScsiPassthrough(DeviceObject, Irp);
            }
        }
        315436 | 315440 /* ATA passthrough */ => {
            if DbgPrint(
                b"(%s:%d) ATA passthrough IOCTLs not supported: 0x%lX\n\0" as *const u8 as *const i8,
                b"drivers/storage/port/scsiport/ioctl.c\0"
                    as *const u8 as *const i8,
                604i32,
                IoControlCode,
            ) != 0 {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                    b"drivers/storage/port/scsiport/ioctl.c\0"
                        as *const u8 as *const i8,
                    604i32,
                );
            }
            status = STATUS_NOT_SUPPORTED;
        }
        _ => {
            if DbgPrint(
                b"(%s:%d) unknown ioctl code: 0x%lX\n\0" as *const u8 as *const i8,
                b"drivers/storage/port/scsiport/ioctl.c\0"
                    as *const u8 as *const i8,
                609i32,
                IoControlCode,
            ) != 0 {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                    b"drivers/storage/port/scsiport/ioctl.c\0"
                        as *const u8 as *const i8,
                    609i32,
                );
            }
            status = STATUS_NOT_SUPPORTED;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    status
}
