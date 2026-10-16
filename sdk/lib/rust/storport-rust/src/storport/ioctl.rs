use crate::prelude::*;
use ntkrnl_sys::ddk::{PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, PSTORAGE_ADAPTER_DESCRIPTOR, PSTORAGE_DESCRIPTOR_HEADER, PSTORAGE_DEVICE_DESCRIPTOR, PSTORAGE_PROPERTY_QUERY, STORAGE_DESCRIPTOR_HEADER, STORAGE_DEVICE_DESCRIPTOR, UINT32};
use core::ffi::c_void;

unsafe extern "C" fn GetFieldLength(mut Name: PUCHAR, mut MaxLength: UINT32) -> UINT32 {
    let mut Index: UINT32;
    let mut LastCharacterPosition: UINT32 = 0;

    // scan the field and return last position which contains a valid character
    Index = 0;
    while Index < MaxLength {
        if *Name.offset(Index as isize) as ::core::ffi::c_int != ' ' as i32 {
            // trim white spaces from field
            LastCharacterPosition = Index;
        }
        Index = Index.wrapping_add(1);
    }

    // convert from zero based index to length
    return LastCharacterPosition.wrapping_add(1);
}

unsafe extern "C" fn FdoDeviceControlQueryProperty(
    FdoExtension: PFDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Miniport: PMINIPORT;
    let mut AdapterDescriptor: PSTORAGE_ADAPTER_DESCRIPTOR;
    let mut Query: PSTORAGE_PROPERTY_QUERY;
    let mut Status: NTSTATUS = 0;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    Query = (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_PROPERTY_QUERY;
    Miniport = core::ptr::addr_of_mut!((*FdoExtension).Miniport);

    loop {
        /* check property type (handle only StorageAdapterProperty) */
        if (*Query).PropertyId as u32 != StorageAdapterProperty as u32 {
            if (*Query).PropertyId as u32 == StorageDeviceProperty as u32
                || (*Query).PropertyId as u32 == StorageDeviceIdProperty as u32
            {
                Status = STATUS_INVALID_DEVICE_REQUEST;
            } else {
                Status = STATUS_INVALID_PARAMETER_1;
            }

            break;
        }

        /* check query type */
        if (*Query).QueryType as u32 == PropertyExistsQuery as u32 {
            /* device property / adapter property is supported */
            Status = STATUS_SUCCESS;
            break;
        }

        if (*Query).QueryType as u32 != PropertyStandardQuery as u32 {
            /* only standard query and exists query are supported */
            Status = STATUS_INVALID_PARAMETER_2;
            break;
        }

        /* Check buffer length */
        if ((*IoStack).Parameters.DeviceIoControl.OutputBufferLength as usize)
            < core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR>()
        {
            let mut DescriptorHeader: PSTORAGE_DESCRIPTOR_HEADER =
                (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_DESCRIPTOR_HEADER;

            /* Fail request if the buffer is simply too small */
            if ((*IoStack).Parameters.DeviceIoControl.OutputBufferLength as usize)
                < core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>()
            {
                Status = STATUS_BUFFER_TOO_SMALL; /* FIXME: Is this code appropriate? */
                break;
            }

            /* Return required size */
            (*DescriptorHeader).Version =
                core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR>() as ULONG;
            (*DescriptorHeader).Size =
                core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR>() as ULONG;
            (*Irp).IoStatus.Information =
                core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as usize as ULONG_PTR;
            Status = STATUS_SUCCESS;
            break;
        }

        /* Return AdapterDescriptor */
        AdapterDescriptor = (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_ADAPTER_DESCRIPTOR;

        /* Classpnp is built for NTDDI_WIN8 and reads SrbType and AddressType past our tail */
        RtlZeroMemory!(
            AdapterDescriptor as *mut core::ffi::c_void,
            core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR>(),
        );

        (*AdapterDescriptor).Version =
            core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR>() as ULONG;
        (*AdapterDescriptor).Size =
            core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR>() as ULONG;
        (*AdapterDescriptor).MaximumTransferLength = (*Miniport).PortConfig.MaximumTransferLength;
        (*AdapterDescriptor).MaximumPhysicalPages = (*Miniport).PortConfig.NumberOfPhysicalBreaks;
        (*AdapterDescriptor).AlignmentMask = (*Miniport).PortConfig.AlignmentMask;
        (*AdapterDescriptor).AdapterUsesPio = FALSE; /* Storport requirement */
        (*AdapterDescriptor).AdapterScansDown = (*Miniport).PortConfig.AdapterScansDown;
        (*AdapterDescriptor).CommandQueueing = TRUE; /* Storport requirement */
        (*AdapterDescriptor).AcceleratedTransfer = TRUE;
        (*AdapterDescriptor).BusType = BusTypeSata as UCHAR; /* FIXME: ＲＥＡＤ　ＦＲＯＭ　ＲＥＧＩＳＴＲＹ */
        (*AdapterDescriptor).BusMajorVersion = 2 as USHORT;
        (*AdapterDescriptor).BusMinorVersion = 0 as USHORT;

        (*Irp).IoStatus.Information =
            core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR>() as usize as ULONG_PTR;
        Status = STATUS_SUCCESS;

        if !(0 != 0) {
            break;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);

    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn PortFdoDeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION = ::core::ptr::null_mut::<_IO_STACK_LOCATION>();
    let mut Status: NTSTATUS = 0;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    match (*IoStack).Parameters.DeviceIoControl.IoControlCode {
        IOCTL_STORAGE_QUERY_PROPERTY => {
            return FdoDeviceControlQueryProperty((*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION, Irp);
        }
        _ => {
            // __debugbreak();
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = 0 as ULONG_PTR;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

    return Status;
}

unsafe extern "C" fn PdoDeviceControlQueryProperty(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    const CASE_STORAGE_DEVICE_PROPERTY: ::core::ffi::c_uint = StorageDeviceProperty as ::core::ffi::c_uint;
    const CASE_STORAGE_ADAPTER_PROPERTY: ::core::ffi::c_uint = StorageAdapterProperty as ::core::ffi::c_uint;
    const CASE_POWER_OR_DEFAULT: ::core::ffi::c_uint = StorageDevicePowerProperty as ::core::ffi::c_uint;
    let mut IoStack: PIO_STACK_LOCATION = ::core::ptr::null_mut();
    let mut Query: PSTORAGE_PROPERTY_QUERY = ::core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    IoStack = IoGetCurrentIrpStackLocation(Irp);
    Query = (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_PROPERTY_QUERY;

    loop {
        /* check property type */
        if (*Query).PropertyId as ::core::ffi::c_uint != StorageDeviceProperty as ::core::ffi::c_uint
            && (*Query).PropertyId as ::core::ffi::c_uint != StorageAdapterProperty as ::core::ffi::c_uint
        {
            // only device property / adapter property are supported
            Status = STATUS_INVALID_PARAMETER_1;
            // Divergence: ioctl.c has no `break` here, so an unsupported property answered
            // an exists-query with STATUS_SUCCESS and a standard query with
            // STATUS_NOT_IMPLEMENTED (the switch's default) -- never the status it just set.
            break;
        }

        /* check query type */
        if (*Query).QueryType as ::core::ffi::c_uint == PropertyExistsQuery as ::core::ffi::c_uint {
            /* device property / adapter property is supported */
            Status = STATUS_SUCCESS;
            break;
        }

        if (*Query).QueryType as ::core::ffi::c_uint != PropertyStandardQuery as ::core::ffi::c_uint {
            /* only standard query and exists query are supported */
            Status = STATUS_INVALID_PARAMETER_2;
            break;
        }

        /* For different property types... */
        match (*Query).PropertyId as ::core::ffi::c_uint {
            CASE_STORAGE_DEVICE_PROPERTY => {
                let mut DeviceDescriptor: PSTORAGE_DEVICE_DESCRIPTOR = ::core::ptr::null_mut();
                let mut InquiryData: PINQUIRYDATA = (*PdoExtension).InquiryBuffer;
                let mut VendorIdLength: ULONG = GetFieldLength(
                    &raw mut (*InquiryData).VendorId as PUCHAR,
                    8 as UINT32,
                ) as ULONG;
                let mut ProductIdLength: ULONG = GetFieldLength(
                    &raw mut (*InquiryData).ProductId as PUCHAR,
                    16 as UINT32,
                ) as ULONG;
                let mut RevisionLength: ULONG = GetFieldLength(
                    &raw mut (*InquiryData).ProductRevisionLevel as PUCHAR,
                    4 as UINT32,
                ) as ULONG;
                let mut TotalLength: ULONG;
                let mut Dest: PUCHAR = ::core::ptr::null_mut();

                /*
                 * total length required is
                 *     sizeof(STORAGE_DEVICE_DESCRIPTOR) + (String + NullByte) + (...) + ... - 1
                 * STORAGE_DEVICE_DESCRIPTOR contains 1 byte at the end already
                 */
                TotalLength = (::core::mem::size_of::<STORAGE_DEVICE_DESCRIPTOR>() as ULONG)
                    .wrapping_add(VendorIdLength)
                    .wrapping_add(1 as ULONG)
                    .wrapping_add(ProductIdLength)
                    .wrapping_add(1 as ULONG)
                    .wrapping_add(RevisionLength)
                    .wrapping_add(1 as ULONG);

                /* Check buffer length */
                if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength < TotalLength {
                    let mut DescriptorHeader: PSTORAGE_DESCRIPTOR_HEADER =
                        (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_DESCRIPTOR_HEADER;

                    /* Fail request if the buffer is simply too small */
                    if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength
                        < ::core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as ULONG
                    {
                        Status = STATUS_BUFFER_TOO_SMALL; /* FIXME: Is this code appropriate? */
                        break;
                    }

                    /* Return required size */
                    (*DescriptorHeader).Version = TotalLength;
                    (*DescriptorHeader).Size = TotalLength;
                    (*Irp).IoStatus.Information =
                        ::core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as ULONG_PTR;
                    Status = STATUS_SUCCESS;
                    break;
                }

                /* Fill in data */
                DeviceDescriptor = (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_DEVICE_DESCRIPTOR;
                *DeviceDescriptor = STORAGE_DEVICE_DESCRIPTOR {
                    Version: ::core::mem::size_of::<STORAGE_DEVICE_DESCRIPTOR>() as ULONG,
                    Size: TotalLength,
                    DeviceType: (*InquiryData).DeviceType(),
                    DeviceTypeModifier: (*InquiryData).DeviceTypeModifier(),
                    RemovableMedia: (*InquiryData).RemovableMedia() as BOOLEAN,
                    CommandQueueing: (*InquiryData).CommandQueue() as BOOLEAN,
                    VendorIdOffset: 0,
                    ProductIdOffset: 0,
                    ProductRevisionOffset: 0,
                    SerialNumberOffset: 0,
                    BusType: BusTypeSata, /* FIXME: USE FDO BUS TYPE FROM REGISTRY */
                    RawPropertiesLength: 0,
                    RawDeviceProperties: [0; 1],
                };

                /* Arbitrary data offsets, set them cascading */
                (*DeviceDescriptor).VendorIdOffset =
                    core::mem::offset_of!(STORAGE_DEVICE_DESCRIPTOR, RawDeviceProperties) as ULONG;
                (*DeviceDescriptor).ProductIdOffset = (*DeviceDescriptor)
                    .VendorIdOffset
                    .wrapping_add(VendorIdLength)
                    .wrapping_add(1 as ULONG);
                (*DeviceDescriptor).ProductRevisionOffset = (*DeviceDescriptor)
                    .ProductIdOffset
                    .wrapping_add(ProductIdLength)
                    .wrapping_add(1 as ULONG);
                (*DeviceDescriptor).SerialNumberOffset = (*DeviceDescriptor)
                    .ProductRevisionOffset
                    .wrapping_add(RevisionLength)
                    .wrapping_add(1 as ULONG);

                /* Fill the arbitrary data */
                Dest = &raw mut (*DeviceDescriptor).RawDeviceProperties as *mut UCHAR as PUCHAR;

                RtlCopyMemory!(
                    Dest as *mut ::core::ffi::c_void,
                    &raw mut (*InquiryData).VendorId as *mut UCHAR as *const ::core::ffi::c_void,
                    VendorIdLength as usize,
                );
                *Dest.offset(VendorIdLength as isize) = 0;
                Dest = Dest.offset(VendorIdLength.wrapping_add(1 as ULONG) as isize);

                RtlCopyMemory!(
                    Dest as *mut ::core::ffi::c_void,
                    &raw mut (*InquiryData).ProductId as *mut UCHAR as *const ::core::ffi::c_void,
                    ProductIdLength as usize,
                );
                *Dest.offset(ProductIdLength as isize) = 0;
                Dest = Dest.offset(ProductIdLength.wrapping_add(1 as ULONG) as isize);

                RtlCopyMemory!(
                    Dest as *mut ::core::ffi::c_void,
                    &raw mut (*InquiryData).ProductRevisionLevel as *mut UCHAR as *const ::core::ffi::c_void,
                    RevisionLength as usize,
                );
                *Dest.offset(RevisionLength as isize) = 0;
                Dest = Dest.offset(RevisionLength.wrapping_add(1 as ULONG) as isize);

                /* FIXME: Read serial number when enumerating devices */
                *Dest = 0;

                /* TODO: Verify this doesnt overflow */
                (*Irp).IoStatus.Information = TotalLength as ULONG_PTR;
                Status = STATUS_SUCCESS;
                // C's `break;` leaves the switch; the `while (0)` tail below then leaves.
            }
            CASE_STORAGE_ADAPTER_PROPERTY => {
                IoSkipCurrentIrpStackLocation(Irp);
                return IoCallDriver((*(*PdoExtension).FdoExtension).Device, Irp);
            }
            CASE_POWER_OR_DEFAULT | _ => {
                Status = STATUS_NOT_IMPLEMENTED;
            }
        }

        if !(0 != 0) {
            break;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

    return Status;
}

unsafe extern "C" fn PdoDeviceControlScsiGetAddress(
    mut DeviceExtension: PPDO_DEVICE_EXTENSION,
    mut Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut ScsiAddress: PSCSI_ADDRESS = core::ptr::null_mut();
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut FdoDeviceExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();

    DPRINT!("PdoDeviceControlScsiGetAddress(%p %p)\n", DeviceExtension, Irp);

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    NT_ASSERT!(
        (*DeviceExtension).ExtensionType as ::core::ffi::c_uint
            == EXTENSION_TYPE_PdoExtension as ::core::ffi::c_uint,
    );
    FdoDeviceExtension = (*DeviceExtension).FdoExtension;

    loop {
        /* Check buffer length */
        if ((*IoStack).Parameters.DeviceIoControl.OutputBufferLength as usize)
            < (core::mem::size_of::<SCSI_ADDRESS>() as usize)
        {
            Status = STATUS_BUFFER_TOO_SMALL;
            (*Irp).IoStatus.Information = 0 as ULONG_PTR;
            break;
        }

        ScsiAddress = (*Irp).AssociatedIrp.SystemBuffer as PSCSI_ADDRESS;
        (*ScsiAddress).Length = core::mem::size_of::<SCSI_ADDRESS>() as ULONG;
        (*ScsiAddress).PortNumber = (*FdoDeviceExtension).ScsiPortNumber as UCHAR;
        (*ScsiAddress).PathId = (*DeviceExtension).Bus as UCHAR;
        (*ScsiAddress).TargetId = (*DeviceExtension).Target as UCHAR;
        (*ScsiAddress).Lun = (*DeviceExtension).Lun as UCHAR;

        Status = STATUS_SUCCESS;
        (*Irp).IoStatus.Information = core::mem::size_of::<SCSI_ADDRESS>() as ULONG_PTR;

        DPRINT!(
            "SCSI Address for PDO %p : (Path Target Lun)(%d %d %d)\n",
            (*DeviceExtension).Device,
            (*ScsiAddress).PathId,
            (*ScsiAddress).TargetId,
            (*ScsiAddress).Lun,
        );

        if !((0) != 0) {
            break;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn PortPdoDeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    match (*IoStack).Parameters.DeviceIoControl.IoControlCode {
        IOCTL_STORAGE_QUERY_PROPERTY => {
            return PdoDeviceControlQueryProperty(
                (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION,
                Irp,
            );
        }

        IOCTL_SCSI_GET_ADDRESS => {
            return PdoDeviceControlScsiGetAddress(
                (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION,
                Irp,
            );
        }

        _ => {
            // DbgBreakPoint();
            Status = STATUS_NOT_IMPLEMENTED;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = 0 as ULONG_PTR;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

    return Status;
}
