use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ANSI_STRING, IoMarkIrpPending, IoStartPacket, LPCSTR, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, PSTORAGE_DESCRIPTOR_HEADER, PSTORAGE_DEVICE_DESCRIPTOR, PSTORAGE_PROPERTY_QUERY, RtlInitUnicodeString, RtlUnicodeStringToAnsiString, STORAGE_DESCRIPTOR_HEADER, STORAGE_DEVICE_DESCRIPTOR, STORAGE_PROPERTY_QUERY, PCUNICODE_STRING, PDRIVER_CANCEL};
use ntkrnl_sys::ddk::_STORAGE_BUS_TYPE_BusTypeUsb as BusTypeUsb;
use ntkrnl_sys::ddk::_STORAGE_PROPERTY_ID_StorageDeviceProperty as StorageDeviceProperty;
use ntkrnl_sys::ddk::_STORAGE_PROPERTY_ID_StorageAdapterProperty as StorageAdapterProperty;
use ntkrnl_sys::ddk::_STORAGE_QUERY_TYPE_PropertyStandardQuery as PropertyStandardQuery;
use ntkrnl_sys::ddk::_STORAGE_QUERY_TYPE_PropertyExistsQuery as PropertyExistsQuery;
use ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation;
use core::ffi::c_void;

extern "system" {
    fn USBSTOR_CancelIo(DeviceObject: PDEVICE_OBJECT, Irp: PIRP);
}
unsafe extern "C" {
    fn USBSTOR_QueueAddIrp(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> BOOLEAN;
}
unsafe extern "C" {
    fn USBSTOR_QueueRelease(DeviceObject: PDEVICE_OBJECT);
}
unsafe extern "C" {
    fn USBSTOR_QueueWaitForPendingRequests(DeviceObject: PDEVICE_OBJECT);
}
// wcslen (CRT) and SptiHandleScsiPassthru (sptilib) are both cdecl C (`_wcslen` / no `@16`),
// NOT NTAPI/stdcall — they must NOT live in the extern "system" block above (that would emit
// `_wcslen@4` / `_SptiHandleScsiPassthru@16`, undefined at link). sptilib.c declares
// SptiHandleScsiPassthru with no NTAPI ⇒ cdecl; wcslen resolves from the ntoskrnl import lib.
extern "C" {
    fn wcslen(s: *const WCHAR) -> usize;
    fn SptiHandleScsiPassthru(
        DeviceObject: PDEVICE_OBJECT, Irp: PIRP, MaximumTransferLength: ULONG, MaximumPhysicalPages: ULONG,
    ) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
pub const USBSTOR_DEFAULT_MAX_PHYS_PAGES: u32 = USBSTOR_DEFAULT_MAX_TRANSFER_LENGTH / PAGE_SIZE as u32 + 1;
unsafe fn IsRequestValid(Irp: PIRP) -> BOOLEAN {
    const SRB_FLAGS_DATA_IN: ULONG = 0x00000040;
    const SRB_FLAGS_DATA_OUT: ULONG = 0x00000080;
    const SRB_FLAGS_UNSPECIFIED_DIRECTION: ULONG = SRB_FLAGS_DATA_IN | SRB_FLAGS_DATA_OUT;
    const USBSTOR_DEFAULT_MAX_TRANSFER_LENGTH: ULONG = 0x10000;

    let mut TransferLength: ULONG;
    let IoStack: PIO_STACK_LOCATION;
    let Srb: PSCSI_REQUEST_BLOCK;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    Srb = (*IoStack).Parameters.Scsi.Srb as PSCSI_REQUEST_BLOCK;

    if (*Srb).SrbFlags & (SRB_FLAGS_DATA_IN | SRB_FLAGS_DATA_OUT) != 0 {
        if (*Srb).SrbFlags & SRB_FLAGS_UNSPECIFIED_DIRECTION == SRB_FLAGS_UNSPECIFIED_DIRECTION {
            return 0;
        }

        TransferLength = (*Srb).DataTransferLength;

        if (*Irp).MdlAddress.is_null() {
            return 0;
        }

        if TransferLength == 0 {
            return 0;
        }

        if TransferLength > USBSTOR_DEFAULT_MAX_TRANSFER_LENGTH {
            return 0;
        }
    } else {
        if (*Srb).DataTransferLength != 0 {
            return 0;
        }

        if !(*Srb).DataBuffer.is_null() {
            return 0;
        }

        if !(*Irp).MdlAddress.is_null() {
            return 0;
        }
    }

    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_HandleInternalDeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {

    const SRB_FUNCTION_EXECUTE_SCSI: i32 = 0x00;
    const SRB_FUNCTION_CLAIM_DEVICE: i32 = 0x01;
    const SRB_FUNCTION_RELEASE_DEVICE: i32 = 0x06;
    const SRB_FUNCTION_SHUTDOWN: i32 = 0x07;
    const SRB_FUNCTION_FLUSH: i32 = 0x08;
    const SRB_FUNCTION_RELEASE_QUEUE: i32 = 0x04;
    const SRB_FUNCTION_FLUSH_QUEUE: i32 = 0x15;

    const SCSIOP_MODE_SENSE: i32 = 0x1A;

    const SRB_STATUS_PENDING: u32 = 0x00;
    const SRB_STATUS_SUCCESS: u32 = 0x01;
    const SRB_STATUS_BUSY: u32 = 0x05;
    const SRB_STATUS_ERROR: u32 = 0x04;

    const STATUS_DEVICE_BUSY: NTSTATUS = 0x80000011u32 as NTSTATUS;

    let IoStack: PIO_STACK_LOCATION;
    let Request: PSCSI_REQUEST_BLOCK;
    let PDODeviceExtension: PPDO_DEVICE_EXTENSION;
    let mut Status: NTSTATUS;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    Request = (*IoStack).Parameters.Others.Argument1 as PSCSI_REQUEST_BLOCK;
    // ASSERT(Request);
    PDODeviceExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;
    // ASSERT(PDODeviceExtension->Common.IsFDO == FALSE);

    match (*Request).Function as i32 {
        SRB_FUNCTION_EXECUTE_SCSI => {
            if IsRequestValid(Irp) == 0 {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                if (*Request).Cdb[0] as i32 == SCSIOP_MODE_SENSE {
                    // FIXME SCSIOP_MODE_SENSE
                }

                IoMarkIrpPending(Irp);
                (*Request).SrbStatus = SRB_STATUS_PENDING as UCHAR;

                if USBSTOR_QueueAddIrp((*PDODeviceExtension).LowerDeviceObject, Irp) == 0 {
                    IoStartPacket(
                        (*PDODeviceExtension).LowerDeviceObject,
                        Irp,
                        (*Request).__bindgen_anon_1.QueueSortKey.as_mut() as *mut ULONG,
                        core::mem::transmute::<
                            unsafe extern "system" fn(PDEVICE_OBJECT, PIRP),
                            PDRIVER_CANCEL,
                        >(USBSTOR_CancelIo),
                    );
                }

                return STATUS_PENDING;
            }
        }
        SRB_FUNCTION_RELEASE_DEVICE => {
            // ASSERT(PDODeviceExtension->Claimed == TRUE);
            (*PDODeviceExtension).Claimed = 0 as BOOLEAN;
            Status = STATUS_SUCCESS;
        }
        SRB_FUNCTION_CLAIM_DEVICE => {
            if (*PDODeviceExtension).Claimed != 0 {
                Status = STATUS_DEVICE_BUSY;
                (*Request).SrbStatus = SRB_STATUS_BUSY as UCHAR;
            } else {
                (*PDODeviceExtension).Claimed = 1 as BOOLEAN;
                (*Request).DataBuffer = DeviceObject as PVOID;
                Status = STATUS_SUCCESS;
            }
        }
        SRB_FUNCTION_RELEASE_QUEUE => {
            USBSTOR_QueueRelease((*PDODeviceExtension).LowerDeviceObject);
            (*Request).SrbStatus = SRB_STATUS_SUCCESS as UCHAR;
            Status = STATUS_SUCCESS;
        }
        SRB_FUNCTION_SHUTDOWN | SRB_FUNCTION_FLUSH | SRB_FUNCTION_FLUSH_QUEUE => {
            USBSTOR_QueueWaitForPendingRequests((*PDODeviceExtension).LowerDeviceObject);
            (*Request).SrbStatus = SRB_STATUS_SUCCESS as UCHAR;
            Status = STATUS_SUCCESS;
        }
        _ => {
            Status = STATUS_NOT_SUPPORTED;
            (*Request).SrbStatus = SRB_STATUS_ERROR as UCHAR;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_GetFieldLength(mut Name: PUCHAR, mut MaxLength: ULONG) -> ULONG {
    let mut Index: ULONG;
    let mut LastCharacterPosition: ULONG = 0 as ULONG;

    // scan the field and return last position which contains a valid character
    Index = 0 as ULONG;
    while Index < MaxLength {
        if *Name.offset(Index as isize) as i32 != ' ' as i32 {
            // trim white spaces from field
            LastCharacterPosition = Index;
        }
        Index = Index.wrapping_add(1);
    }

    // convert from zero based index to length
    return LastCharacterPosition.wrapping_add(1 as ULONG);
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_HandleQueryProperty(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {

    let mut IoStack: PIO_STACK_LOCATION;
    let mut PropertyQuery: PSTORAGE_PROPERTY_QUERY;
    let mut DescriptorHeader: PSTORAGE_DESCRIPTOR_HEADER;
    let mut AdapterDescriptor: PSTORAGE_ADAPTER_DESCRIPTOR_WIN8;
    let mut FieldLengthVendor: ULONG;
    let mut FieldLengthProduct: ULONG;
    let mut FieldLengthRevision: ULONG;
    let mut TotalLength: ULONG;
    let mut FieldLengthSerialNumber: ULONG;
    let mut PDODeviceExtension: PPDO_DEVICE_EXTENSION;
    let mut InquiryData: PINQUIRYDATA;
    let mut DeviceDescriptor: PSTORAGE_DEVICE_DESCRIPTOR;
    let mut Buffer: PUCHAR;
    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION;
    let mut SerialNumber: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut AnsiString: ANSI_STRING = ANSI_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Status: NTSTATUS;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    PropertyQuery = (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_PROPERTY_QUERY;

    // check property type
    if (*PropertyQuery).PropertyId != StorageDeviceProperty
        && (*PropertyQuery).PropertyId != StorageAdapterProperty
    {
        return STATUS_INVALID_PARAMETER_1;
    }

    // check query type
    if (*PropertyQuery).QueryType == PropertyExistsQuery {
        return STATUS_SUCCESS;
    }

    if (*PropertyQuery).QueryType != PropertyStandardQuery {
        return STATUS_INVALID_PARAMETER_2;
    }

    // check if it is a device property
    if (*PropertyQuery).PropertyId == StorageDeviceProperty {
        PDODeviceExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;

        FDODeviceExtension =
            (*(*PDODeviceExtension).LowerDeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;

        InquiryData = core::ptr::addr_of_mut!((*PDODeviceExtension).InquiryData) as PINQUIRYDATA;

        // compute extra parameters length
        FieldLengthVendor =
            USBSTOR_GetFieldLength(core::ptr::addr_of_mut!((*InquiryData).VendorId) as PUCHAR, 8);
        FieldLengthProduct =
            USBSTOR_GetFieldLength(core::ptr::addr_of_mut!((*InquiryData).ProductId) as PUCHAR, 16);
        FieldLengthRevision = USBSTOR_GetFieldLength(
            core::ptr::addr_of_mut!((*InquiryData).ProductRevisionLevel) as PUCHAR,
            4,
        );

        if !(*FDODeviceExtension).SerialNumber.is_null() {
            FieldLengthSerialNumber = wcslen(
                core::ptr::addr_of_mut!((*(*FDODeviceExtension).SerialNumber).bString) as *mut WCHAR,
            ) as ULONG;
        } else {
            FieldLengthSerialNumber = 0;
        }

        TotalLength = (core::mem::size_of::<STORAGE_DEVICE_DESCRIPTOR>() as ULONG)
            .wrapping_add(FieldLengthVendor)
            .wrapping_add(FieldLengthProduct)
            .wrapping_add(FieldLengthRevision)
            .wrapping_add(FieldLengthSerialNumber)
            .wrapping_add(3);

        // check if output buffer is long enough
        if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength < TotalLength {
            DescriptorHeader = (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_DESCRIPTOR_HEADER;

            // return required size
            (*DescriptorHeader).Version = TotalLength;
            (*DescriptorHeader).Size = TotalLength;

            (*Irp).IoStatus.Information =
                core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as _;
            return STATUS_SUCCESS;
        }

        // initialize the device descriptor
        DeviceDescriptor = (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_DEVICE_DESCRIPTOR;

        (*DeviceDescriptor).Version = core::mem::size_of::<STORAGE_DEVICE_DESCRIPTOR>() as ULONG;
        (*DeviceDescriptor).Size = TotalLength;
        (*DeviceDescriptor).DeviceType = (*InquiryData).DeviceType();
        (*DeviceDescriptor).DeviceTypeModifier = (*InquiryData).DeviceTypeModifier();
        (*DeviceDescriptor).RemovableMedia = (*InquiryData).RemovableMedia() as BOOLEAN;
        (*DeviceDescriptor).CommandQueueing = 0 as BOOLEAN;
        (*DeviceDescriptor).BusType = BusTypeUsb;
        (*DeviceDescriptor).VendorIdOffset = (core::mem::size_of::<STORAGE_DEVICE_DESCRIPTOR>()
            as ULONG)
            .wrapping_sub(core::mem::size_of::<UCHAR>() as ULONG);
        (*DeviceDescriptor).ProductIdOffset = (*DeviceDescriptor)
            .VendorIdOffset
            .wrapping_add(FieldLengthVendor)
            .wrapping_add(1);
        (*DeviceDescriptor).ProductRevisionOffset = (*DeviceDescriptor)
            .ProductIdOffset
            .wrapping_add(FieldLengthProduct)
            .wrapping_add(1);
        (*DeviceDescriptor).SerialNumberOffset = if FieldLengthSerialNumber > 0 {
            (*DeviceDescriptor)
                .ProductRevisionOffset
                .wrapping_add(FieldLengthRevision)
                .wrapping_add(1)
        } else {
            0
        };
        (*DeviceDescriptor).RawPropertiesLength = FieldLengthVendor
            .wrapping_add(FieldLengthProduct)
            .wrapping_add(FieldLengthRevision)
            .wrapping_add(FieldLengthSerialNumber)
            .wrapping_add(3)
            .wrapping_add(if FieldLengthSerialNumber > 0 { 1 } else { 0 });

        // copy descriptors
        Buffer = (DeviceDescriptor as usize)
            .wrapping_add(core::mem::size_of::<STORAGE_DEVICE_DESCRIPTOR>())
            .wrapping_sub(core::mem::size_of::<UCHAR>()) as PUCHAR;

        RtlCopyMemory(
            Buffer as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*InquiryData).VendorId) as *const core::ffi::c_void,
            FieldLengthVendor as usize,
        );
        *Buffer.offset(FieldLengthVendor as isize) = b'\0';
        Buffer = Buffer.offset(FieldLengthVendor.wrapping_add(1) as isize);

        RtlCopyMemory(
            Buffer as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*InquiryData).ProductId) as *const core::ffi::c_void,
            FieldLengthProduct as usize,
        );
        *Buffer.offset(FieldLengthProduct as isize) = b'\0';
        Buffer = Buffer.offset(FieldLengthProduct.wrapping_add(1) as isize);

        RtlCopyMemory(
            Buffer as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*InquiryData).ProductRevisionLevel) as *const core::ffi::c_void,
            FieldLengthRevision as usize,
        );
        *Buffer.offset(FieldLengthRevision as isize) = b'\0';
        Buffer = Buffer.offset(FieldLengthRevision.wrapping_add(1) as isize);

        if FieldLengthSerialNumber != 0 {
            RtlInitUnicodeString(
                core::ptr::addr_of_mut!(SerialNumber),
                core::ptr::addr_of_mut!((*(*FDODeviceExtension).SerialNumber).bString) as *mut WCHAR
                    as PCWSTR,
            );

            AnsiString.Buffer = Buffer as PCHAR;
            AnsiString.Length = 0 as USHORT;
            AnsiString.MaximumLength =
                (FieldLengthSerialNumber as usize).wrapping_mul(core::mem::size_of::<WCHAR>())
                    as USHORT;

            Status = RtlUnicodeStringToAnsiString(
                core::ptr::addr_of_mut!(AnsiString),
                core::ptr::addr_of_mut!(SerialNumber) as PCUNICODE_STRING,
                0 as BOOLEAN,
            );
            let _ = Status;
        }

        (*Irp).IoStatus.Information = TotalLength as _;
        return STATUS_SUCCESS;
    } else {
        // adapter property query request

        if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength
            < core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR_WIN8>() as ULONG
        {
            // buffer too small
            DescriptorHeader = (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_DESCRIPTOR_HEADER;

            // return required size
            (*DescriptorHeader).Version =
                core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR_WIN8>() as ULONG;
            (*DescriptorHeader).Size =
                core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR_WIN8>() as ULONG;

            (*Irp).IoStatus.Information =
                core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as _;
            return STATUS_SUCCESS;
        }

        // get adapter descriptor, information is returned in the same buffer
        AdapterDescriptor =
            (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_ADAPTER_DESCRIPTOR_WIN8;

        // fill out descriptor
        *AdapterDescriptor = _STORAGE_ADAPTER_DESCRIPTOR_WIN8 {
            Version: core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR_WIN8>() as ULONG,
            Size: core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR_WIN8>() as ULONG,
            MaximumTransferLength: USBSTOR_DEFAULT_MAX_TRANSFER_LENGTH as ULONG,
            MaximumPhysicalPages: USBSTOR_DEFAULT_MAX_PHYS_PAGES as ULONG,
            AlignmentMask: 0,
            AdapterUsesPio: 0,
            AdapterScansDown: 0,
            CommandQueueing: 0,
            AcceleratedTransfer: 0,
            BusType: BusTypeUsb as UCHAR,
            BusMajorVersion: 2 as USHORT,
            BusMinorVersion: 0 as USHORT,
            SrbType: 0,
            AddressType: 0,
        };

        // store returned length
        (*Irp).IoStatus.Information =
            core::mem::size_of::<STORAGE_ADAPTER_DESCRIPTOR_WIN8>() as _;

        return STATUS_SUCCESS;
    }
}
#[no_mangle]
pub unsafe extern "system" fn USBSTOR_HandleDeviceControl(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    const IOCTL_STORAGE_QUERY_PROPERTY: u32 = 2954240;
    const IOCTL_SCSI_PASS_THROUGH: u32 = 315396;
    const IOCTL_SCSI_PASS_THROUGH_DIRECT: u32 = 315412;
    const IOCTL_STORAGE_GET_MEDIA_SERIAL_NUMBER: u32 = 2952208;
    const IOCTL_SCSI_GET_CAPABILITIES: u32 = 266256;
    const IOCTL_SCSI_GET_INQUIRY_DATA: u32 = 266252;
    const IOCTL_SCSI_GET_ADDRESS: u32 = 266264;
    const USBSTOR_DEFAULT_MAX_TRANSFER_LENGTH: u32 = 0x10000;
    const USB_STOR_TAG: u32 = 0x73627375;
    const MAX_LUN: i32 = 0xF;

    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut PDODeviceExtension: PPDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut BusInfo: PSCSI_ADAPTER_BUS_INFO = core::ptr::null_mut();
    let mut ScsiInquiryData: PSCSI_INQUIRY_DATA = core::ptr::null_mut();
    let mut InquiryData: PINQUIRYDATA = core::ptr::null_mut();

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    match (*IoStack).Parameters.DeviceIoControl.IoControlCode {
        IOCTL_STORAGE_QUERY_PROPERTY => {
            Status = USBSTOR_HandleQueryProperty(DeviceObject, Irp);
        }
        IOCTL_SCSI_PASS_THROUGH | IOCTL_SCSI_PASS_THROUGH_DIRECT => {
            PDODeviceExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;

            if (*IoGetCurrentIrpStackLocation(Irp)).MinorFunction as i32 == 0
                && (*PDODeviceExtension).Claimed as i32 != 0
            {
                Status = STATUS_INVALID_DEVICE_REQUEST;
            } else {
                Status = SptiHandleScsiPassthru(
                    DeviceObject,
                    Irp,
                    USBSTOR_DEFAULT_MAX_TRANSFER_LENGTH as ULONG,
                    (USBSTOR_DEFAULT_MAX_TRANSFER_LENGTH / 4096 + 1) as ULONG,
                );
            }
        }
        IOCTL_STORAGE_GET_MEDIA_SERIAL_NUMBER => {
            Status = STATUS_NOT_SUPPORTED;
        }
        IOCTL_SCSI_GET_CAPABILITIES => {
            let mut Capabilities: PIO_SCSI_CAPABILITIES = core::ptr::null_mut();

            if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength as usize
                == core::mem::size_of::<PVOID>()
            {
                let fresh1 = &mut *((*Irp).AssociatedIrp.SystemBuffer as *mut PVOID);
                *fresh1 = ExAllocatePoolWithTag(
                    NonPagedPool,
                    core::mem::size_of::<IO_SCSI_CAPABILITIES>() as SIZE_T,
                    USB_STOR_TAG as ULONG,
                );
                Capabilities = *fresh1 as PIO_SCSI_CAPABILITIES;
                (*Irp).IoStatus.Information = core::mem::size_of::<PVOID>() as _;
            } else {
                Capabilities = (*Irp).AssociatedIrp.SystemBuffer as PIO_SCSI_CAPABILITIES;
                (*Irp).IoStatus.Information =
                    core::mem::size_of::<IO_SCSI_CAPABILITIES>() as _;
            }

            if !Capabilities.is_null() {
                (*Capabilities).MaximumTransferLength =
                    USBSTOR_DEFAULT_MAX_TRANSFER_LENGTH as ULONG;
                (*Capabilities).MaximumPhysicalPages =
                    (USBSTOR_DEFAULT_MAX_TRANSFER_LENGTH / 4096 + 1) as ULONG;
                (*Capabilities).SupportedAsynchronousEvents = 0 as ULONG;
                (*Capabilities).AlignmentMask = 0 as ULONG;
                (*Capabilities).TaggedQueuing = 0 as BOOLEAN;
                (*Capabilities).AdapterScansDown = 0 as BOOLEAN;
                (*Capabilities).AdapterUsesPio = 0 as BOOLEAN;
                Status = STATUS_SUCCESS;
            } else {
                Status = STATUS_INSUFFICIENT_RESOURCES;
            }
        }
        IOCTL_SCSI_GET_INQUIRY_DATA => {
            PDODeviceExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;

            BusInfo = (*Irp).AssociatedIrp.SystemBuffer as PSCSI_ADAPTER_BUS_INFO;
            ScsiInquiryData = BusInfo.offset(1) as PSCSI_INQUIRY_DATA;
            InquiryData =
                core::ptr::addr_of_mut!((*ScsiInquiryData).InquiryData) as *mut UCHAR
                    as PINQUIRYDATA;

            (*BusInfo).NumberOfBuses = 1 as UCHAR;
            (*(core::ptr::addr_of_mut!((*BusInfo).BusData) as *mut SCSI_BUS_DATA).offset(0))
                .NumberOfLogicalUnits = 1 as UCHAR;
            (*(core::ptr::addr_of_mut!((*BusInfo).BusData) as *mut SCSI_BUS_DATA).offset(0))
                .InitiatorBusId = 0 as UCHAR;
            (*(core::ptr::addr_of_mut!((*BusInfo).BusData) as *mut SCSI_BUS_DATA).offset(0))
                .InquiryDataOffset = core::mem::size_of::<SCSI_ADAPTER_BUS_INFO>() as ULONG;

            (*ScsiInquiryData).PathId = 0 as UCHAR;
            (*ScsiInquiryData).TargetId = 0 as UCHAR;
            (*ScsiInquiryData).Lun = ((*PDODeviceExtension).LUN as i32 & MAX_LUN) as UCHAR;
            (*ScsiInquiryData).DeviceClaimed = (*PDODeviceExtension).Claimed;
            (*ScsiInquiryData).InquiryDataLength = core::mem::size_of::<INQUIRYDATA>() as ULONG;
            (*ScsiInquiryData).NextInquiryDataOffset = 0 as ULONG;

            RtlZeroMemory(
                InquiryData as *mut core::ffi::c_void,
                core::mem::size_of::<INQUIRYDATA>(),
            );
            RtlCopyMemory(
                InquiryData as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!((*PDODeviceExtension).InquiryData) as *const core::ffi::c_void,
                core::mem::size_of::<[UCHAR; 36]>(),
            );

            *(*InquiryData).__bindgen_anon_1.Versions.as_mut() = 0x04 as UCHAR;
            (*InquiryData).set_ResponseDataFormat(0x02 as UCHAR);

            (*Irp).IoStatus.Information = (core::mem::size_of::<SCSI_ADAPTER_BUS_INFO>())
                .wrapping_add(core::mem::size_of::<SCSI_INQUIRY_DATA>())
                .wrapping_add(core::mem::size_of::<INQUIRYDATA>())
                .wrapping_sub(1) as _;
            Status = STATUS_SUCCESS;
        }
        IOCTL_SCSI_GET_ADDRESS => {
            let mut Address: PSCSI_ADDRESS =
                (*Irp).AssociatedIrp.SystemBuffer as PSCSI_ADDRESS;

            (*Address).Length = core::mem::size_of::<SCSI_ADDRESS>() as ULONG;
            (*Address).PortNumber = 0 as UCHAR;
            (*Address).PathId = 0 as UCHAR;
            (*Address).TargetId = 0 as UCHAR;
            (*Address).Lun =
                ((*((*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION)).LUN as i32
                    & MAX_LUN) as UCHAR;
            (*Irp).IoStatus.Information = core::mem::size_of::<SCSI_ADDRESS>() as _;

            Status = STATUS_SUCCESS;
        }
        _ => {
            Status = STATUS_NOT_SUPPORTED;
        }
    }

    return Status;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Universal Serial Bus Bulk Storage Driver
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     USB block storage device driver.
//  * COPYRIGHT:   2005-2006 James Tabor
//  *              2011-2012 Michael Martin (michael.martin@reactos.org)
//  *              2011-2013 Johannes Anderwald (johannes.anderwald@reactos.org)
//  *              2017 Vadim Galyant
//  */
//
// #include "usbstor.h"
// #include <sptilib.h>
//
// #define NDEBUG
// #include <debug.h>
//
// // See CORE-10515 and CORE-10755
// #define USBSTOR_DEFAULT_MAX_PHYS_PAGES    (USBSTOR_DEFAULT_MAX_TRANSFER_LENGTH / PAGE_SIZE + 1)
//
// static
// BOOLEAN
// IsRequestValid(PIRP Irp)
// {
//     ULONG TransferLength;
//     PIO_STACK_LOCATION IoStack;
//     PSCSI_REQUEST_BLOCK Srb;
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     Srb = IoStack->Parameters.Scsi.Srb;
//
//     if (Srb->SrbFlags & (SRB_FLAGS_DATA_IN | SRB_FLAGS_DATA_OUT))
//     {
//         if ((Srb->SrbFlags & SRB_FLAGS_UNSPECIFIED_DIRECTION) == SRB_FLAGS_UNSPECIFIED_DIRECTION)
//         {
//             DPRINT1("IsRequestValid: Invalid Srb. Srb->SrbFlags - %X\n", Srb->SrbFlags);
//             return FALSE;
//         }
//
//         TransferLength = Srb->DataTransferLength;
//
//         if (Irp->MdlAddress == NULL)
//         {
//             DPRINT1("IsRequestValid: Invalid Srb. Irp->MdlAddress == NULL\n");
//             return FALSE;
//         }
//
//         if (TransferLength == 0)
//         {
//             DPRINT1("IsRequestValid: Invalid Srb. TransferLength == 0\n");
//             return FALSE;
//         }
//
//         if (TransferLength > USBSTOR_DEFAULT_MAX_TRANSFER_LENGTH)
//         {
//             DPRINT1("IsRequestValid: Invalid Srb. TransferLength > 0x10000\n");
//             return FALSE;
//         }
//     }
//     else
//     {
//         if (Srb->DataTransferLength)
//         {
//             DPRINT1("IsRequestValid: Invalid Srb. Srb->DataTransferLength != 0\n");
//             return FALSE;
//         }
//
//         if (Srb->DataBuffer)
//         {
//             DPRINT1("IsRequestValid: Invalid Srb. Srb->DataBuffer != NULL\n");
//             return FALSE;
//         }
//
//         if (Irp->MdlAddress)
//         {
//             DPRINT1("IsRequestValid: Invalid Srb. Irp->MdlAddress != NULL\n");
//             return FALSE;
//         }
//     }
//
//     return TRUE;
// }
//
// NTSTATUS
// USBSTOR_HandleInternalDeviceControl(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     PSCSI_REQUEST_BLOCK Request;
//     PPDO_DEVICE_EXTENSION PDODeviceExtension;
//     NTSTATUS Status;
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     Request = (PSCSI_REQUEST_BLOCK)IoStack->Parameters.Others.Argument1;
//     ASSERT(Request);
//     PDODeviceExtension = (PPDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT(PDODeviceExtension->Common.IsFDO == FALSE);
//
//     switch(Request->Function)
//     {
//         case SRB_FUNCTION_EXECUTE_SCSI:
//         {
//             DPRINT("SRB_FUNCTION_EXECUTE_SCSI\n");
//
//             if (!IsRequestValid(Irp))
//             {
//                 Status = STATUS_INVALID_PARAMETER;
//                 break;
//             }
//
//             if (Request->Cdb[0] == SCSIOP_MODE_SENSE)
//             {
//                 DPRINT("USBSTOR_Scsi: SRB_FUNCTION_EXECUTE_SCSI - FIXME SCSIOP_MODE_SENSE\n");
//                 // FIXME Get from registry WriteProtect for StorageDevicePolicies;
//                 // L"\\Registry\\Machine\\System\\CurrentControlSet\\Control\\StorageDevicePolicies"
//                 // QueryTable[0].Name = L"WriteProtect"
//             }
//
//             IoMarkIrpPending(Irp);
//             Request->SrbStatus = SRB_STATUS_PENDING;
//
//             // add the request
//             if (!USBSTOR_QueueAddIrp(PDODeviceExtension->LowerDeviceObject, Irp))
//             {
//                 IoStartPacket(PDODeviceExtension->LowerDeviceObject, Irp, &Request->QueueSortKey, USBSTOR_CancelIo);
//             }
//
//             return STATUS_PENDING;
//         }
//         case SRB_FUNCTION_RELEASE_DEVICE:
//         {
//             DPRINT1("SRB_FUNCTION_RELEASE_DEVICE\n");
//             ASSERT(PDODeviceExtension->Claimed == TRUE);
//
//             // release claim
//             PDODeviceExtension->Claimed = FALSE;
//             Status = STATUS_SUCCESS;
//             break;
//         }
//         case SRB_FUNCTION_CLAIM_DEVICE:
//         {
//             DPRINT1("SRB_FUNCTION_CLAIM_DEVICE\n");
//
//             // check if the device has been claimed
//             if (PDODeviceExtension->Claimed)
//             {
//                 // device has already been claimed
//                 Status = STATUS_DEVICE_BUSY;
//                 Request->SrbStatus = SRB_STATUS_BUSY;
//                 break;
//             }
//
//             // claim device
//             PDODeviceExtension->Claimed = TRUE;
//
//             // output device object
//             Request->DataBuffer = DeviceObject;
//
//             Status = STATUS_SUCCESS;
//             break;
//         }
//         case SRB_FUNCTION_RELEASE_QUEUE:
//         {
//             DPRINT1("SRB_FUNCTION_RELEASE_QUEUE\n");
//
//             USBSTOR_QueueRelease(PDODeviceExtension->LowerDeviceObject);
//
//             Request->SrbStatus = SRB_STATUS_SUCCESS;
//             Status = STATUS_SUCCESS;
//             break;
//         }
//
//         case SRB_FUNCTION_SHUTDOWN:
//         case SRB_FUNCTION_FLUSH:
//         case SRB_FUNCTION_FLUSH_QUEUE:
//         {
//             DPRINT1("SRB_FUNCTION_FLUSH / SRB_FUNCTION_FLUSH_QUEUE / SRB_FUNCTION_SHUTDOWN\n");
//
//             // wait for pending requests to finish
//             USBSTOR_QueueWaitForPendingRequests(PDODeviceExtension->LowerDeviceObject);
//
//             Request->SrbStatus = SRB_STATUS_SUCCESS;
//             Status = STATUS_SUCCESS;
//             break;
//         }
//         default:
//         {
//             //
//             // not supported
//             //
//             Status = STATUS_NOT_SUPPORTED;
//             Request->SrbStatus = SRB_STATUS_ERROR;
//         }
//     }
//
//     Irp->IoStatus.Status = Status;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     return Status;
// }
//
// ULONG
// USBSTOR_GetFieldLength(
//     IN PUCHAR Name,
//     IN ULONG MaxLength)
// {
//     ULONG Index;
//     ULONG LastCharacterPosition = 0;
//
//     // scan the field and return last position which contains a valid character
//     for(Index = 0; Index < MaxLength; Index++)
//     {
//         if (Name[Index] != ' ')
//         {
//             // trim white spaces from field
//             LastCharacterPosition = Index;
//         }
//     }
//
//     // convert from zero based index to length
//     return LastCharacterPosition + 1;
// }
//
// NTSTATUS
// USBSTOR_HandleQueryProperty(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     PSTORAGE_PROPERTY_QUERY PropertyQuery;
//     PSTORAGE_DESCRIPTOR_HEADER DescriptorHeader;
//     PSTORAGE_ADAPTER_DESCRIPTOR_WIN8 AdapterDescriptor;
//     ULONG FieldLengthVendor, FieldLengthProduct, FieldLengthRevision, TotalLength, FieldLengthSerialNumber;
//     PPDO_DEVICE_EXTENSION PDODeviceExtension;
//     PINQUIRYDATA InquiryData;
//     PSTORAGE_DEVICE_DESCRIPTOR DeviceDescriptor;
//     PUCHAR Buffer;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//     UNICODE_STRING SerialNumber;
//     ANSI_STRING AnsiString;
//     NTSTATUS Status;
//
//     DPRINT("USBSTOR_HandleQueryProperty\n");
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     ASSERT(IoStack->Parameters.DeviceIoControl.InputBufferLength >= sizeof(STORAGE_PROPERTY_QUERY));
//     ASSERT(Irp->AssociatedIrp.SystemBuffer);
//
//     PropertyQuery = (PSTORAGE_PROPERTY_QUERY)Irp->AssociatedIrp.SystemBuffer;
//
//     // check property type
//     if (PropertyQuery->PropertyId != StorageDeviceProperty &&
//         PropertyQuery->PropertyId != StorageAdapterProperty)
//     {
//         // only device property / adapter property are supported
//         return STATUS_INVALID_PARAMETER_1;
//     }
//
//     // check query type
//     if (PropertyQuery->QueryType == PropertyExistsQuery)
//     {
//         // device property / adapter property is supported
//         return STATUS_SUCCESS;
//     }
//
//     if (PropertyQuery->QueryType != PropertyStandardQuery)
//     {
//         // only standard query and exists query are supported
//         return STATUS_INVALID_PARAMETER_2;
//     }
//
//     // check if it is a device property
//     if (PropertyQuery->PropertyId == StorageDeviceProperty)
//     {
//         DPRINT("USBSTOR_HandleQueryProperty StorageDeviceProperty OutputBufferLength %lu\n", IoStack->Parameters.DeviceIoControl.OutputBufferLength);
//
//         PDODeviceExtension = (PPDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//         ASSERT(PDODeviceExtension);
//         ASSERT(PDODeviceExtension->Common.IsFDO == FALSE);
//
//         FDODeviceExtension = (PFDO_DEVICE_EXTENSION)PDODeviceExtension->LowerDeviceObject->DeviceExtension;
//         ASSERT(FDODeviceExtension);
//         ASSERT(FDODeviceExtension->Common.IsFDO);
//
//         InquiryData = (PINQUIRYDATA)&PDODeviceExtension->InquiryData;
//
//         // compute extra parameters length
//         FieldLengthVendor = USBSTOR_GetFieldLength(InquiryData->VendorId, 8);
//         FieldLengthProduct = USBSTOR_GetFieldLength(InquiryData->ProductId, 16);
//         FieldLengthRevision = USBSTOR_GetFieldLength(InquiryData->ProductRevisionLevel, 4);
//
//         if (FDODeviceExtension->SerialNumber)
//         {
//             FieldLengthSerialNumber = wcslen(FDODeviceExtension->SerialNumber->bString);
//         }
//         else
//         {
//             FieldLengthSerialNumber = 0;
//         }
//
//         // total length required is sizeof(STORAGE_DEVICE_DESCRIPTOR) + FieldLength + 4 extra null bytes - 1
//         // -1 due STORAGE_DEVICE_DESCRIPTOR contains one byte length of parameter data
//         TotalLength = sizeof(STORAGE_DEVICE_DESCRIPTOR) + FieldLengthVendor + FieldLengthProduct + FieldLengthRevision + FieldLengthSerialNumber + 3;
//
//         // check if output buffer is long enough
//         if (IoStack->Parameters.DeviceIoControl.OutputBufferLength < TotalLength)
//         {
//             // buffer too small
//             DescriptorHeader = (PSTORAGE_DESCRIPTOR_HEADER)Irp->AssociatedIrp.SystemBuffer;
//             ASSERT(IoStack->Parameters.DeviceIoControl.OutputBufferLength >= sizeof(STORAGE_DESCRIPTOR_HEADER));
//
//             // return required size
//             DescriptorHeader->Version = TotalLength;
//             DescriptorHeader->Size = TotalLength;
//
//             Irp->IoStatus.Information = sizeof(STORAGE_DESCRIPTOR_HEADER);
//             return STATUS_SUCCESS;
//         }
//
//         // initialize the device descriptor
//         DeviceDescriptor = (PSTORAGE_DEVICE_DESCRIPTOR)Irp->AssociatedIrp.SystemBuffer;
//
//         DeviceDescriptor->Version = sizeof(STORAGE_DEVICE_DESCRIPTOR);
//         DeviceDescriptor->Size = TotalLength;
//         DeviceDescriptor->DeviceType = InquiryData->DeviceType;
//         DeviceDescriptor->DeviceTypeModifier = InquiryData->DeviceTypeModifier;
//         DeviceDescriptor->RemovableMedia = InquiryData->RemovableMedia;
//         DeviceDescriptor->CommandQueueing = FALSE;
//         DeviceDescriptor->BusType = BusTypeUsb;
//         DeviceDescriptor->VendorIdOffset = sizeof(STORAGE_DEVICE_DESCRIPTOR) - sizeof(UCHAR);
//         DeviceDescriptor->ProductIdOffset = DeviceDescriptor->VendorIdOffset + FieldLengthVendor + 1;
//         DeviceDescriptor->ProductRevisionOffset = DeviceDescriptor->ProductIdOffset + FieldLengthProduct + 1;
//         DeviceDescriptor->SerialNumberOffset = (FieldLengthSerialNumber > 0 ? DeviceDescriptor->ProductRevisionOffset + FieldLengthRevision + 1 : 0);
//         DeviceDescriptor->RawPropertiesLength = FieldLengthVendor + FieldLengthProduct + FieldLengthRevision + FieldLengthSerialNumber + 3 + (FieldLengthSerialNumber > 0 ? + 1 : 0);
//
//         // copy descriptors
//         Buffer = (PUCHAR)((ULONG_PTR)DeviceDescriptor + sizeof(STORAGE_DEVICE_DESCRIPTOR) - sizeof(UCHAR));
//
//         RtlCopyMemory(Buffer, InquiryData->VendorId, FieldLengthVendor);
//         Buffer[FieldLengthVendor] = '\0';
//         Buffer += FieldLengthVendor + 1;
//
//         RtlCopyMemory(Buffer, InquiryData->ProductId, FieldLengthProduct);
//         Buffer[FieldLengthProduct] = '\0';
//         Buffer += FieldLengthProduct + 1;
//
//         RtlCopyMemory(Buffer, InquiryData->ProductRevisionLevel, FieldLengthRevision);
//         Buffer[FieldLengthRevision] = '\0';
//         Buffer += FieldLengthRevision + 1;
//
//         if (FieldLengthSerialNumber)
//         {
//             RtlInitUnicodeString(&SerialNumber, FDODeviceExtension->SerialNumber->bString);
//
//             AnsiString.Buffer = (PCHAR)Buffer;
//             AnsiString.Length = 0;
//             AnsiString.MaximumLength = FieldLengthSerialNumber * sizeof(WCHAR);
//
//             Status = RtlUnicodeStringToAnsiString(&AnsiString, &SerialNumber, FALSE);
//             ASSERT(Status == STATUS_SUCCESS);
//         }
//
//         DPRINT("Vendor %s\n", (LPCSTR)((ULONG_PTR)DeviceDescriptor + DeviceDescriptor->VendorIdOffset));
//         DPRINT("Product %s\n", (LPCSTR)((ULONG_PTR)DeviceDescriptor + DeviceDescriptor->ProductIdOffset));
//         DPRINT("Revision %s\n", (LPCSTR)((ULONG_PTR)DeviceDescriptor + DeviceDescriptor->ProductRevisionOffset));
//         DPRINT("Serial %s\n", (LPCSTR)((ULONG_PTR)DeviceDescriptor + DeviceDescriptor->SerialNumberOffset));
//
//         Irp->IoStatus.Information = TotalLength;
//         return STATUS_SUCCESS;
//     }
//     else
//     {
//         // adapter property query request
//
//         DPRINT("USBSTOR_HandleQueryProperty StorageAdapterProperty OutputBufferLength %lu\n", IoStack->Parameters.DeviceIoControl.OutputBufferLength);
//
//         if (IoStack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(STORAGE_ADAPTER_DESCRIPTOR_WIN8))
//         {
//             // buffer too small
//             DescriptorHeader = (PSTORAGE_DESCRIPTOR_HEADER)Irp->AssociatedIrp.SystemBuffer;
//             ASSERT(IoStack->Parameters.DeviceIoControl.OutputBufferLength >= sizeof(STORAGE_DESCRIPTOR_HEADER));
//
//             // return required size
//             DescriptorHeader->Version = sizeof(STORAGE_ADAPTER_DESCRIPTOR_WIN8);
//             DescriptorHeader->Size = sizeof(STORAGE_ADAPTER_DESCRIPTOR_WIN8);
//
//             Irp->IoStatus.Information = sizeof(STORAGE_DESCRIPTOR_HEADER);
//             return STATUS_SUCCESS;
//         }
//
//         // get adapter descriptor, information is returned in the same buffer
//         AdapterDescriptor = Irp->AssociatedIrp.SystemBuffer;
//
//         // fill out descriptor
//         // NOTE: STORAGE_ADAPTER_DESCRIPTOR_WIN8 may vary in size, so it's important to zero out
//         // all unused fields
//         *AdapterDescriptor = (STORAGE_ADAPTER_DESCRIPTOR_WIN8) {
//             .Version = sizeof(STORAGE_ADAPTER_DESCRIPTOR_WIN8),
//             .Size = sizeof(STORAGE_ADAPTER_DESCRIPTOR_WIN8),
//             .MaximumTransferLength = USBSTOR_DEFAULT_MAX_TRANSFER_LENGTH,
//             .MaximumPhysicalPages = USBSTOR_DEFAULT_MAX_PHYS_PAGES,
//             .BusType = BusTypeUsb,
//             .BusMajorVersion = 2, //FIXME verify
//             .BusMinorVersion = 0 //FIXME
//         };
//
//         // __debugbreak();
//
//         // store returned length
//         Irp->IoStatus.Information = sizeof(STORAGE_ADAPTER_DESCRIPTOR_WIN8);
//
//         return STATUS_SUCCESS;
//     }
// }
//
// NTSTATUS
// USBSTOR_HandleDeviceControl(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status;
//     PPDO_DEVICE_EXTENSION PDODeviceExtension;
//     PSCSI_ADAPTER_BUS_INFO BusInfo;
//     PSCSI_INQUIRY_DATA ScsiInquiryData;
//     PINQUIRYDATA InquiryData;
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     switch (IoStack->Parameters.DeviceIoControl.IoControlCode)
//     {
//         case IOCTL_STORAGE_QUERY_PROPERTY:
//             Status = USBSTOR_HandleQueryProperty(DeviceObject, Irp);
//             break;
//         case IOCTL_SCSI_PASS_THROUGH:
//         case IOCTL_SCSI_PASS_THROUGH_DIRECT:
//         {
//             PDODeviceExtension = DeviceObject->DeviceExtension;
//
//             ASSERT(KeGetCurrentIrql() < DISPATCH_LEVEL);
//             ASSERT(PDODeviceExtension);
//             ASSERT(PDODeviceExtension->Common.IsFDO == FALSE);
//
//             /* Skip requests that bypassed the class driver. See also cdrom!RequestHandleScsiPassThrough */
//             if ((IoGetCurrentIrpStackLocation(Irp)->MinorFunction == 0) && PDODeviceExtension->Claimed)
//             {
//                 Status = STATUS_INVALID_DEVICE_REQUEST;
//                 break;
//             }
//
//             Status = SptiHandleScsiPassthru(DeviceObject,
//                                             Irp,
//                                             USBSTOR_DEFAULT_MAX_TRANSFER_LENGTH,
//                                             USBSTOR_DEFAULT_MAX_PHYS_PAGES);
//             break;
//         }
//         case IOCTL_STORAGE_GET_MEDIA_SERIAL_NUMBER:
//             DPRINT1("USBSTOR_HandleDeviceControl IOCTL_STORAGE_GET_MEDIA_SERIAL_NUMBER NOT implemented\n");
//             Status = STATUS_NOT_SUPPORTED;
//             break;
//         case IOCTL_SCSI_GET_CAPABILITIES:
//         {
//             PIO_SCSI_CAPABILITIES Capabilities;
//
//             // Legacy port capability query
//             if (IoStack->Parameters.DeviceIoControl.OutputBufferLength == sizeof(PVOID))
//             {
//                 Capabilities = *((PVOID *)Irp->AssociatedIrp.SystemBuffer) = ExAllocatePoolWithTag(NonPagedPool,
//                                                                                                    sizeof(IO_SCSI_CAPABILITIES),
//                                                                                                    USB_STOR_TAG);
//                 Irp->IoStatus.Information = sizeof(PVOID);
//             }
//             else
//             {
//                 Capabilities = Irp->AssociatedIrp.SystemBuffer;
//                 Irp->IoStatus.Information = sizeof(IO_SCSI_CAPABILITIES);
//             }
//
//             if (Capabilities)
//             {
//                 Capabilities->MaximumTransferLength = USBSTOR_DEFAULT_MAX_TRANSFER_LENGTH;
//                 Capabilities->MaximumPhysicalPages = USBSTOR_DEFAULT_MAX_PHYS_PAGES;
//                 Capabilities->SupportedAsynchronousEvents = 0;
//                 Capabilities->AlignmentMask = 0;
//                 Capabilities->TaggedQueuing = FALSE;
//                 Capabilities->AdapterScansDown = FALSE;
//                 Capabilities->AdapterUsesPio = FALSE;
//                 Status = STATUS_SUCCESS;
//             }
//             else
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//             }
//
//             break;
//         }
//         case IOCTL_SCSI_GET_INQUIRY_DATA:
//         {
//             PDODeviceExtension = (PPDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//             ASSERT(PDODeviceExtension);
//             ASSERT(PDODeviceExtension->Common.IsFDO == FALSE);
//
//             // get parameters
//             BusInfo = Irp->AssociatedIrp.SystemBuffer;
//             ScsiInquiryData = (PSCSI_INQUIRY_DATA)(BusInfo + 1);
//             InquiryData = (PINQUIRYDATA)ScsiInquiryData->InquiryData;
//
//
//             BusInfo->NumberOfBuses = 1;
//             BusInfo->BusData[0].NumberOfLogicalUnits = 1; //FIXME
//             BusInfo->BusData[0].InitiatorBusId = 0;
//             BusInfo->BusData[0].InquiryDataOffset = sizeof(SCSI_ADAPTER_BUS_INFO);
//
//             ScsiInquiryData->PathId = 0;
//             ScsiInquiryData->TargetId = 0;
//             ScsiInquiryData->Lun = PDODeviceExtension->LUN & MAX_LUN;
//             ScsiInquiryData->DeviceClaimed = PDODeviceExtension->Claimed;
//             ScsiInquiryData->InquiryDataLength = sizeof(INQUIRYDATA);
//             ScsiInquiryData->NextInquiryDataOffset = 0;
//
//             // Note: INQUIRYDATA structure is larger than INQUIRYDATABUFFERSIZE
//             RtlZeroMemory(InquiryData, sizeof(INQUIRYDATA));
//             RtlCopyMemory(InquiryData, &PDODeviceExtension->InquiryData, sizeof(PDODeviceExtension->InquiryData));
//
//             InquiryData->Versions = 0x04;
//             InquiryData->ResponseDataFormat = 0x02; // some devices set this to 1
//
//             Irp->IoStatus.Information = sizeof(SCSI_ADAPTER_BUS_INFO) + sizeof(SCSI_INQUIRY_DATA) + sizeof(INQUIRYDATA) - 1;
//             Status = STATUS_SUCCESS;
//
//             break;
//         }
//         case IOCTL_SCSI_GET_ADDRESS:
//         {
//             PSCSI_ADDRESS Address = Irp->AssociatedIrp.SystemBuffer;
//
//             Address->Length = sizeof(SCSI_ADDRESS);
//             Address->PortNumber = 0;
//             Address->PathId = 0;
//             Address->TargetId = 0;
//             Address->Lun = (((PPDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension)->LUN & MAX_LUN);
//             Irp->IoStatus.Information = sizeof(SCSI_ADDRESS);
//
//             Status = STATUS_SUCCESS;
//
//             break;
//         }
//         default:
//             DPRINT("USBSTOR_HandleDeviceControl IoControl %x not supported\n", IoStack->Parameters.DeviceIoControl.IoControlCode);
//             Status = STATUS_NOT_SUPPORTED;
//             break;
//     }
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================