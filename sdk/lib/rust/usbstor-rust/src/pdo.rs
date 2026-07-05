use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ANSI_STRING, _BUS_QUERY_ID_TYPE_BusQueryCompatibleIDs as BusQueryCompatibleIDs, _BUS_QUERY_ID_TYPE_BusQueryDeviceID as BusQueryDeviceID, _BUS_QUERY_ID_TYPE_BusQueryHardwareIDs as BusQueryHardwareIDs, _BUS_QUERY_ID_TYPE_BusQueryInstanceID as BusQueryInstanceID, DEVICE_RELATIONS, ExAllocatePoolUninitialized, _KWAIT_REASON_Executive, IoCreateDevice, IoDeleteDevice, IoForwardIrpSynchronously, IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, KEVENT, KeInitializeEvent, KeWaitForSingleObject, LPCSTR, LPWSTR, PCSZ, PDEVICE_CAPABILITIES, PDEVICE_OBJECT, PDEVICE_RELATIONS, PIO_STACK_LOCATION, PIRP, PKEVENT, PMDL, RtlAnsiStringToUnicodeString, RtlInitAnsiString, _EVENT_TYPE_SynchronizationEvent, _DEVICE_RELATION_TYPE_TargetDeviceRelation as TargetDeviceRelation, UINT32};
use core::ffi::c_void;

extern "C" {
    fn sprintf(buf: *mut ::core::ffi::c_char, fmt: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn strlen(s: *const ::core::ffi::c_char) -> usize;
}
extern "system" {
    fn RtlAssert(FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut CHAR);
}
extern "C" {
    fn RtlStringCchPrintfW( pszDest: NTSTRSAFE_PWSTR, cchDest: size_t, pszFormat: NTSTRSAFE_PCWSTR, ... ) -> NTSTATUS;
}
extern "system" {
    fn IoAllocateMdl( VirtualAddress: PVOID, Length: ULONG, SecondaryBuffer: BOOLEAN, ChargeQuota: BOOLEAN, Irp: PIRP, ) -> PMDL;
    fn IoFreeMdl(Mdl: PMDL);
    fn MmBuildMdlForNonPagedPool(MemoryDescriptorList: PMDL);
    fn IoAllocateIrp(StackSize: i8, ChargeQuota: BOOLEAN) -> PIRP;
    fn IoFreeIrp(Irp: PIRP);
}
extern "C" {
    fn IoSetCompletionRoutine( Irp: PIRP, CompletionRoutine: Option< unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS, >, Context: PVOID, InvokeOnSuccess: BOOLEAN, InvokeOnError: BOOLEAN, InvokeOnCancel: BOOLEAN, );
    fn DbgPrint(Format: PCSTR, ...) -> u32;
}

// ---- file-scope preamble (translated) ----
// #define NDEBUG

const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016u32 as NTSTATUS;
// USB_STOR_TAG 'sbsu' (usbstor.h:11) => ('s'<<24)|('b'<<16)|('s'<<8)|'u'
const USB_STOR_TAG: u32 = 0x73627375;
// IRP_MN minor codes absent from driver_ddk (xdk/iotypes.h) — typed u8 to match KIO_STACK_LOCATION.MinorFunction
const IRP_MN_QUERY_CAPABILITIES: u8 = 0x09;
const IRP_MN_QUERY_DEVICE_TEXT: u8 = 0x0C;
const IRP_MN_QUERY_ID: u8 = 0x13;
unsafe fn USBSTOR_GetDeviceType(InquiryData: PINQUIRYDATA) -> LPCSTR {
    const DIRECT_ACCESS_DEVICE: u32 = 0x00;
    const SEQUENTIAL_ACCESS_DEVICE: u32 = 0x01;
    const WRITE_ONCE_READ_MULTIPLE_DEVICE: u32 = 0x04;
    const READ_ONLY_DIRECT_ACCESS_DEVICE: u32 = 0x05;
    const OPTICAL_DEVICE: u32 = 0x07;
    const MEDIUM_CHANGER: u32 = 0x08;
    match (*InquiryData).DeviceType() as u32 {
        DIRECT_ACCESS_DEVICE => return b"Disk\0".as_ptr() as LPCSTR,
        SEQUENTIAL_ACCESS_DEVICE => return b"Sequential\0".as_ptr() as LPCSTR,
        WRITE_ONCE_READ_MULTIPLE_DEVICE => return b"Worm\0".as_ptr() as LPCSTR,
        READ_ONLY_DIRECT_ACCESS_DEVICE => return b"CdRom\0".as_ptr() as LPCSTR,
        OPTICAL_DEVICE => return b"Optical\0".as_ptr() as LPCSTR,
        MEDIUM_CHANGER => return b"Changer\0".as_ptr() as LPCSTR,
        _ => return b"Other\0".as_ptr() as LPCSTR,
    }
}
unsafe fn USBSTOR_GetGenericType(InquiryData: PINQUIRYDATA) -> LPCSTR {
    match (*InquiryData).DeviceType() as u32 {
        DIRECT_ACCESS_DEVICE => return b"GenDisk\0".as_ptr() as LPCSTR,
        SEQUENTIAL_ACCESS_DEVICE => return b"GenSequential\0".as_ptr() as LPCSTR,
        WRITE_ONCE_READ_MULTIPLE_DEVICE => return b"GenWorm\0".as_ptr() as LPCSTR,
        READ_ONLY_DIRECT_ACCESS_DEVICE => return b"GenCdRom\0".as_ptr() as LPCSTR,
        OPTICAL_DEVICE => return b"GenOptical\0".as_ptr() as LPCSTR,
        MEDIUM_CHANGER => return b"GenChanger\0".as_ptr() as LPCSTR,
        _ => return b"UsbstorOther\0".as_ptr() as LPCSTR,
    };
}
unsafe fn CopyField(mut Name: PUCHAR, mut Buffer: PCHAR, mut MaxLength: ULONG) -> ULONG {
    let mut Index: ULONG;
    Index = 0 as ULONG;
    while Index < MaxLength {
        if *Name.offset(Index as isize) as ::core::ffi::c_int <= ' ' as i32
            || *Name.offset(Index as isize) as ::core::ffi::c_int >= 0x7f as ::core::ffi::c_int
            || *Name.offset(Index as isize) as ::core::ffi::c_int == ',' as i32
        {
            *Buffer.offset(Index as isize) = '_' as i32 as CHAR;
        } else {
            *Buffer.offset(Index as isize) = *Name.offset(Index as isize) as CHAR;
        }
        Index = Index.wrapping_add(1);
    }
    return MaxLength;
}
unsafe fn CopyFieldTruncate(
    mut Name: PUCHAR,
    mut Buffer: PCHAR,
    mut MaxLength: ULONG,
) -> ULONG {
    let mut Index: ULONG = 0;
    Index = 0 as ULONG;
    while Index < MaxLength {
        if *Name.offset(Index as isize) as core::ffi::c_int == '\0' as i32 {
            break;
        } else if *Name.offset(Index as isize) as core::ffi::c_int <= ' ' as i32
            || *Name.offset(Index as isize) as core::ffi::c_int >= 0x7f as core::ffi::c_int
            || *Name.offset(Index as isize) as core::ffi::c_int == ',' as i32
        {
            *Buffer.offset(Index as isize) = ' ' as i32 as CHAR;
        } else {
            *Buffer.offset(Index as isize) = *Name.offset(Index as isize) as CHAR;
        }
        Index = Index.wrapping_add(1);
    }
    return Index;
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_PdoHandleQueryDeviceText(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut DeviceExtension: PPDO_DEVICE_EXTENSION;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut LocalBuffer: [CHAR; 26] = [0; 26];
    let mut Offset: UINT32 = 0 as UINT32;
    let mut InquiryData: PINQUIRYDATA;
    let mut AnsiString: ANSI_STRING = ANSI_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut DeviceDescription: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    DeviceExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;
    InquiryData = core::ptr::addr_of_mut!((*DeviceExtension).InquiryData) as PINQUIRYDATA;

    match (*IoStack).Parameters.QueryDeviceText.DeviceTextType as u32 {
        0 | 1 => {
            Offset = (Offset as ULONG).wrapping_add(CopyFieldTruncate(
                core::ptr::addr_of_mut!((*InquiryData).VendorId) as PUCHAR,
                (core::ptr::addr_of_mut!(LocalBuffer) as *mut CHAR).offset(Offset as isize) as PCHAR,
                core::mem::size_of::<[UCHAR; 8]>() as ULONG,
            ) as ULONG) as UINT32;
            let fresh0 = Offset;
            Offset = Offset.wrapping_add(1);
            LocalBuffer[fresh0 as usize] = ' ' as i32 as CHAR;
            Offset = (Offset as ULONG).wrapping_add(CopyFieldTruncate(
                core::ptr::addr_of_mut!((*InquiryData).ProductId) as PUCHAR,
                (core::ptr::addr_of_mut!(LocalBuffer) as *mut CHAR).offset(Offset as isize) as PCHAR,
                core::mem::size_of::<[UCHAR; 16]>() as ULONG,
            ) as ULONG) as UINT32;
            let fresh1 = Offset;
            Offset = Offset.wrapping_add(1);
            LocalBuffer[fresh1 as usize] = '\0' as i32 as CHAR;

            RtlInitAnsiString(core::ptr::addr_of_mut!(AnsiString), core::ptr::addr_of_mut!(LocalBuffer) as PCSZ);

            DeviceDescription.Length = 0 as USHORT;
            DeviceDescription.MaximumLength =
                (Offset as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as USHORT;
            DeviceDescription.Buffer = ExAllocatePoolWithTag(
                PagedPool,
                DeviceDescription.MaximumLength as SIZE_T,
                USB_STOR_TAG as ULONG,
            ) as PWCH;
            if DeviceDescription.Buffer.is_null() {
                (*Irp).IoStatus.Information = 0 as _;
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            RtlAnsiStringToUnicodeString(
                core::ptr::addr_of_mut!(DeviceDescription),
                core::ptr::addr_of_mut!(AnsiString),
                FALSE as BOOLEAN,
            );

            (*Irp).IoStatus.Information = DeviceDescription.Buffer as _;
            return STATUS_SUCCESS;
        }
        _ => {
            (*Irp).IoStatus.Information = 0 as _;
            return (*Irp).IoStatus.__bindgen_anon_1.Status;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_PdoHandleQueryDeviceId(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {

    let mut DeviceExtension: PPDO_DEVICE_EXTENSION;
    let mut Status: NTSTATUS;
    let mut Buffer: [CHAR; 100] = [0; 100];
    let mut DeviceType: LPCSTR;
    let mut Offset: ULONG = 0;
    let mut InquiryData: PINQUIRYDATA;
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();
    let mut DeviceId: _UNICODE_STRING = core::mem::zeroed();

    DeviceExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;
    InquiryData = core::ptr::addr_of_mut!((*DeviceExtension).InquiryData) as PINQUIRYDATA;

    DeviceType = USBSTOR_GetDeviceType(InquiryData);

    // lets create device string
    Offset = sprintf(
        (Buffer.as_mut_ptr() as *mut CHAR).offset(Offset as isize) as *mut ::core::ffi::c_char,
        b"USBSTOR\\\0".as_ptr() as *const ::core::ffi::c_char,
    ) as ULONG;
    Offset = Offset.wrapping_add(sprintf(
        (Buffer.as_mut_ptr() as *mut CHAR).offset(Offset as isize) as *mut ::core::ffi::c_char,
        DeviceType as *const ::core::ffi::c_char,
    ) as ULONG);
    Offset = Offset.wrapping_add(sprintf(
        (Buffer.as_mut_ptr() as *mut CHAR).offset(Offset as isize) as *mut ::core::ffi::c_char,
        b"&Ven_\0".as_ptr() as *const ::core::ffi::c_char,
    ) as ULONG);
    Offset = Offset.wrapping_add(CopyField(
        core::ptr::addr_of_mut!((*InquiryData).VendorId) as PUCHAR,
        (Buffer.as_mut_ptr() as *mut CHAR).offset(Offset as isize) as PCHAR,
        8 as ULONG,
    ));
    Offset = Offset.wrapping_add(sprintf(
        (Buffer.as_mut_ptr() as *mut CHAR).offset(Offset as isize) as *mut ::core::ffi::c_char,
        b"&Prod_\0".as_ptr() as *const ::core::ffi::c_char,
    ) as ULONG);
    Offset = Offset.wrapping_add(CopyField(
        core::ptr::addr_of_mut!((*InquiryData).ProductId) as PUCHAR,
        (Buffer.as_mut_ptr() as *mut CHAR).offset(Offset as isize) as PCHAR,
        16 as ULONG,
    ));
    Offset = Offset.wrapping_add(sprintf(
        (Buffer.as_mut_ptr() as *mut CHAR).offset(Offset as isize) as *mut ::core::ffi::c_char,
        b"&Rev_\0".as_ptr() as *const ::core::ffi::c_char,
    ) as ULONG);
    Offset = Offset.wrapping_add(CopyField(
        core::ptr::addr_of_mut!((*InquiryData).ProductRevisionLevel) as PUCHAR,
        (Buffer.as_mut_ptr() as *mut CHAR).offset(Offset as isize) as PCHAR,
        4 as ULONG,
    ));

    RtlInitAnsiString(core::ptr::addr_of_mut!(AnsiString), Buffer.as_mut_ptr() as *mut CHAR as PCSZ);

    // allocate DeviceId string
    DeviceId.Length = 0;
    DeviceId.MaximumLength = ((strlen(Buffer.as_mut_ptr() as *mut CHAR as *const ::core::ffi::c_char) as usize)
        .wrapping_add(1)
        .wrapping_mul(core::mem::size_of::<WCHAR>())) as USHORT;
    DeviceId.Buffer = ExAllocatePoolWithTag(
        PagedPool,
        DeviceId.MaximumLength as SIZE_T,
        USB_STOR_TAG as ULONG,
    ) as PWCH;
    if DeviceId.Buffer.is_null() {
        (*Irp).IoStatus.Information = 0 as _;
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = RtlAnsiStringToUnicodeString(
        core::ptr::addr_of_mut!(DeviceId),
        core::ptr::addr_of_mut!(AnsiString),
        FALSE as BOOLEAN,
    );

    if NT_SUCCESS(Status) {
        (*Irp).IoStatus.Information = DeviceId.Buffer as _;
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_ConvertToUnicodeString(
    mut Buffer: *mut CHAR,
    mut ResultBufferLength: ULONG,
    mut ResultBufferOffset: ULONG,
    mut ResultBuffer: LPWSTR,
    mut NewResultBufferOffset: PULONG,
) {
    let mut DeviceString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    let mut AnsiString: ANSI_STRING = ANSI_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<CHAR>(),
    };
    let mut Status: NTSTATUS;

    // ASSERT(ResultBufferLength);
    // ASSERT(ResultBufferLength > ResultBufferOffset);

    // DPRINT(...) - NDEBUG, omitted

    // construct destination string
    DeviceString.Buffer = ResultBuffer.offset(ResultBufferOffset as isize) as *mut WCHAR as PWCH;
    DeviceString.Length = 0 as USHORT;
    DeviceString.MaximumLength = (ResultBufferLength.wrapping_sub(ResultBufferOffset) as usize)
        .wrapping_mul(core::mem::size_of::<WCHAR>())
        as USHORT;

    // initialize source string
    RtlInitAnsiString(core::ptr::addr_of_mut!(AnsiString), Buffer as PCSZ);

    Status = RtlAnsiStringToUnicodeString(
        core::ptr::addr_of_mut!(DeviceString),
        core::ptr::addr_of_mut!(AnsiString),
        FALSE as BOOLEAN,
    );
    // ASSERT(Status == STATUS_SUCCESS);
    let _ = Status;

    // subtract consumed bytes
    ResultBufferLength = ResultBufferLength.wrapping_sub(
        (DeviceString.Length as usize)
            .wrapping_add(core::mem::size_of::<WCHAR>())
            .wrapping_div(core::mem::size_of::<WCHAR>()) as ULONG,
    );
    ResultBufferOffset = ResultBufferOffset.wrapping_add(
        (DeviceString.Length as usize)
            .wrapping_add(core::mem::size_of::<WCHAR>())
            .wrapping_div(core::mem::size_of::<WCHAR>()) as ULONG,
    );
    let _ = ResultBufferLength;
    let _ = Buffer;

    *NewResultBufferOffset = ResultBufferOffset;
}

#[no_mangle]
pub unsafe extern "C" fn USBSTOR_PdoHandleQueryHardwareId(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    const USB_STOR_TAG: ULONG = 0x73627375; // 'sbsu'
    const UNICODE_NULL: WCHAR = 0;

    let mut PDODeviceExtension: PPDO_DEVICE_EXTENSION =
        ::core::ptr::null_mut::<PDO_DEVICE_EXTENSION>();
    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION =
        ::core::ptr::null_mut::<FDO_DEVICE_EXTENSION>();
    let mut GenericType: LPCSTR = ::core::ptr::null::<CHAR>();
    let mut DeviceType: LPCSTR = ::core::ptr::null::<CHAR>();
    let mut Buffer: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut Id1: [CHAR; 50] = [0; 50];
    let mut Id2: [CHAR; 50] = [0; 50];
    let mut Id3: [CHAR; 50] = [0; 50];
    let mut Id4: [CHAR; 50] = [0; 50];
    let mut Id5: [CHAR; 50] = [0; 50];
    let mut Id6: [CHAR; 50] = [0; 50];
    let mut Id7: [CHAR; 50] = [0; 50];
    let mut Id1Length: ULONG = 0;
    let mut Id2Length: ULONG = 0;
    let mut Id3Length: ULONG = 0;
    let mut Id4Length: ULONG = 0;
    let mut Id5Length: ULONG = 0;
    let mut Id6Length: ULONG = 0;
    let mut Id7Length: ULONG = 0;
    let mut Offset: ULONG = 0;
    let mut TotalLength: ULONG = 0;
    let mut Length: ULONG = 0;
    let mut InquiryData: PINQUIRYDATA = ::core::ptr::null_mut::<INQUIRYDATA>();

    PDODeviceExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;
    FDODeviceExtension =
        (*(*PDODeviceExtension).LowerDeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    if (*FDODeviceExtension).DeviceDescriptor.is_null() {
        RtlAssert(
            b"FDODeviceExtension->DeviceDescriptor\0" as *const u8 as *const ::core::ffi::c_char
                as PVOID,
            b"pdo.c\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            282 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    InquiryData = core::ptr::addr_of_mut!((*PDODeviceExtension).InquiryData) as PINQUIRYDATA;

    DeviceType = USBSTOR_GetDeviceType(InquiryData);
    GenericType = USBSTOR_GetGenericType(InquiryData);

    if GenericType.is_null() {
        RtlAssert(
            b"GenericType\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"pdo.c\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            288 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    // generate id 1
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(Id1) as *mut CHAR,
        0,
        core::mem::size_of::<[CHAR; 50]>(),
    );
    Offset = 0 as ULONG;
    Offset = sprintf(
        (core::ptr::addr_of_mut!(Id1) as *mut CHAR).offset(Offset as isize) as *mut ::core::ffi::c_char,
        b"USBSTOR\\\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ULONG;
    Offset = Offset.wrapping_add(sprintf(
        (core::ptr::addr_of_mut!(Id1) as *mut CHAR).offset(Offset as isize) as *mut ::core::ffi::c_char,
        DeviceType as *const ::core::ffi::c_char,
    ) as ULONG);
    Offset = Offset.wrapping_add(CopyField(
        core::ptr::addr_of_mut!((*InquiryData).VendorId) as PUCHAR,
        (core::ptr::addr_of_mut!(Id1) as *mut CHAR).offset(Offset as isize) as PCHAR,
        8 as ULONG,
    ));
    Offset = Offset.wrapping_add(CopyField(
        core::ptr::addr_of_mut!((*InquiryData).ProductId) as PUCHAR,
        (core::ptr::addr_of_mut!(Id1) as *mut CHAR).offset(Offset as isize) as PCHAR,
        16 as ULONG,
    ));
    Offset = Offset.wrapping_add(CopyField(
        core::ptr::addr_of_mut!((*InquiryData).ProductRevisionLevel) as PUCHAR,
        (core::ptr::addr_of_mut!(Id1) as *mut CHAR).offset(Offset as isize) as PCHAR,
        4 as ULONG,
    ));
    Id1Length = (strlen(core::ptr::addr_of_mut!(Id1) as *mut CHAR) as u64).wrapping_add(1) as ULONG;

    // generate id 2
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(Id2) as *mut CHAR,
        0,
        core::mem::size_of::<[CHAR; 50]>(),
    );
    Offset = 0 as ULONG;
    Offset = sprintf(
        (core::ptr::addr_of_mut!(Id2) as *mut CHAR).offset(Offset as isize) as *mut ::core::ffi::c_char,
        b"USBSTOR\\\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ULONG;
    Offset = Offset.wrapping_add(sprintf(
        (core::ptr::addr_of_mut!(Id2) as *mut CHAR).offset(Offset as isize) as *mut ::core::ffi::c_char,
        DeviceType as *const ::core::ffi::c_char,
    ) as ULONG);
    Offset = Offset.wrapping_add(CopyField(
        core::ptr::addr_of_mut!((*InquiryData).VendorId) as PUCHAR,
        (core::ptr::addr_of_mut!(Id2) as *mut CHAR).offset(Offset as isize) as PCHAR,
        8 as ULONG,
    ));
    Offset = Offset.wrapping_add(CopyField(
        core::ptr::addr_of_mut!((*InquiryData).ProductId) as PUCHAR,
        (core::ptr::addr_of_mut!(Id2) as *mut CHAR).offset(Offset as isize) as PCHAR,
        16 as ULONG,
    ));
    Id2Length = (strlen(core::ptr::addr_of_mut!(Id2) as *mut CHAR) as u64).wrapping_add(1) as ULONG;

    // generate id 3
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(Id3) as *mut CHAR,
        0,
        core::mem::size_of::<[CHAR; 50]>(),
    );
    Offset = 0 as ULONG;
    Offset = sprintf(
        (core::ptr::addr_of_mut!(Id3) as *mut CHAR).offset(Offset as isize) as *mut ::core::ffi::c_char,
        b"USBSTOR\\\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ULONG;
    Offset = Offset.wrapping_add(sprintf(
        (core::ptr::addr_of_mut!(Id3) as *mut CHAR).offset(Offset as isize) as *mut ::core::ffi::c_char,
        DeviceType as *const ::core::ffi::c_char,
    ) as ULONG);
    Offset = Offset.wrapping_add(CopyField(
        core::ptr::addr_of_mut!((*InquiryData).VendorId) as PUCHAR,
        (core::ptr::addr_of_mut!(Id3) as *mut CHAR).offset(Offset as isize) as PCHAR,
        8 as ULONG,
    ));
    Id3Length = (strlen(core::ptr::addr_of_mut!(Id3) as *mut CHAR) as u64).wrapping_add(1) as ULONG;

    // generate id 4
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(Id4) as *mut CHAR,
        0,
        core::mem::size_of::<[CHAR; 50]>(),
    );
    Offset = 0 as ULONG;
    Offset = sprintf(
        (core::ptr::addr_of_mut!(Id4) as *mut CHAR).offset(Offset as isize) as *mut ::core::ffi::c_char,
        b"USBSTOR\\\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ULONG;
    Offset = Offset.wrapping_add(sprintf(
        (core::ptr::addr_of_mut!(Id4) as *mut CHAR).offset(Offset as isize) as *mut ::core::ffi::c_char,
        DeviceType as *const ::core::ffi::c_char,
    ) as ULONG);
    Offset = Offset.wrapping_add(CopyField(
        core::ptr::addr_of_mut!((*InquiryData).VendorId) as PUCHAR,
        (core::ptr::addr_of_mut!(Id4) as *mut CHAR).offset(Offset as isize) as PCHAR,
        8 as ULONG,
    ));
    Offset = Offset.wrapping_add(CopyField(
        core::ptr::addr_of_mut!((*InquiryData).ProductId) as PUCHAR,
        (core::ptr::addr_of_mut!(Id4) as *mut CHAR).offset(Offset as isize) as PCHAR,
        16 as ULONG,
    ));
    Offset = Offset.wrapping_add(CopyField(
        core::ptr::addr_of_mut!((*InquiryData).ProductRevisionLevel) as PUCHAR,
        (core::ptr::addr_of_mut!(Id4) as *mut CHAR).offset(Offset as isize) as PCHAR,
        1 as ULONG,
    ));
    Id4Length = (strlen(core::ptr::addr_of_mut!(Id4) as *mut CHAR) as u64).wrapping_add(1) as ULONG;

    // generate id 5
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(Id5) as *mut CHAR,
        0,
        core::mem::size_of::<[CHAR; 50]>(),
    );
    Offset = 0 as ULONG;
    Offset = sprintf(
        (core::ptr::addr_of_mut!(Id5) as *mut CHAR).offset(Offset as isize) as *mut ::core::ffi::c_char,
        DeviceType as *const ::core::ffi::c_char,
    ) as ULONG;
    Offset = Offset.wrapping_add(CopyField(
        core::ptr::addr_of_mut!((*InquiryData).VendorId) as PUCHAR,
        (core::ptr::addr_of_mut!(Id5) as *mut CHAR).offset(Offset as isize) as PCHAR,
        8 as ULONG,
    ));
    Offset = Offset.wrapping_add(CopyField(
        core::ptr::addr_of_mut!((*InquiryData).ProductId) as PUCHAR,
        (core::ptr::addr_of_mut!(Id5) as *mut CHAR).offset(Offset as isize) as PCHAR,
        16 as ULONG,
    ));
    Offset = Offset.wrapping_add(CopyField(
        core::ptr::addr_of_mut!((*InquiryData).ProductRevisionLevel) as PUCHAR,
        (core::ptr::addr_of_mut!(Id5) as *mut CHAR).offset(Offset as isize) as PCHAR,
        1 as ULONG,
    ));
    Id5Length = (strlen(core::ptr::addr_of_mut!(Id5) as *mut CHAR) as u64).wrapping_add(1) as ULONG;

    // generate id 6
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(Id6) as *mut CHAR,
        0,
        core::mem::size_of::<[CHAR; 50]>(),
    );
    Offset = 0 as ULONG;
    Offset = sprintf(
        (core::ptr::addr_of_mut!(Id6) as *mut CHAR).offset(Offset as isize) as *mut ::core::ffi::c_char,
        b"USBSTOR\\\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ULONG;
    Offset = Offset.wrapping_add(sprintf(
        (core::ptr::addr_of_mut!(Id6) as *mut CHAR).offset(Offset as isize) as *mut ::core::ffi::c_char,
        GenericType as *const ::core::ffi::c_char,
    ) as ULONG);
    Id6Length = (strlen(core::ptr::addr_of_mut!(Id6) as *mut CHAR) as u64).wrapping_add(1) as ULONG;

    // generate id 7
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(Id7) as *mut CHAR,
        0,
        core::mem::size_of::<[CHAR; 50]>(),
    );
    Offset = 0 as ULONG;
    Offset = sprintf(
        (core::ptr::addr_of_mut!(Id7) as *mut CHAR).offset(Offset as isize) as *mut ::core::ffi::c_char,
        GenericType as *const ::core::ffi::c_char,
    ) as ULONG;
    Id7Length = (strlen(core::ptr::addr_of_mut!(Id7) as *mut CHAR) as u64).wrapping_add(1) as ULONG;

    TotalLength = Id1Length
        .wrapping_add(Id2Length)
        .wrapping_add(Id3Length)
        .wrapping_add(Id4Length)
        .wrapping_add(Id5Length)
        .wrapping_add(Id6Length)
        .wrapping_add(Id7Length)
        .wrapping_add(1 as ULONG);

    Buffer = ExAllocatePoolWithTag(
        PagedPool,
        (TotalLength as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        USB_STOR_TAG,
    ) as LPWSTR;
    if Buffer.is_null() {
        (*Irp).IoStatus.Information = 0 as _;
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Offset = 0 as ULONG;
    Length = TotalLength;

    USBSTOR_ConvertToUnicodeString(
        core::ptr::addr_of_mut!(Id1) as *mut CHAR,
        Length,
        Offset,
        Buffer,
        core::ptr::addr_of_mut!(Offset),
    );
    USBSTOR_ConvertToUnicodeString(
        core::ptr::addr_of_mut!(Id2) as *mut CHAR,
        Length,
        Offset,
        Buffer,
        core::ptr::addr_of_mut!(Offset),
    );
    USBSTOR_ConvertToUnicodeString(
        core::ptr::addr_of_mut!(Id3) as *mut CHAR,
        Length,
        Offset,
        Buffer,
        core::ptr::addr_of_mut!(Offset),
    );
    USBSTOR_ConvertToUnicodeString(
        core::ptr::addr_of_mut!(Id4) as *mut CHAR,
        Length,
        Offset,
        Buffer,
        core::ptr::addr_of_mut!(Offset),
    );
    USBSTOR_ConvertToUnicodeString(
        core::ptr::addr_of_mut!(Id5) as *mut CHAR,
        Length,
        Offset,
        Buffer,
        core::ptr::addr_of_mut!(Offset),
    );
    USBSTOR_ConvertToUnicodeString(
        core::ptr::addr_of_mut!(Id6) as *mut CHAR,
        Length,
        Offset,
        Buffer,
        core::ptr::addr_of_mut!(Offset),
    );
    USBSTOR_ConvertToUnicodeString(
        core::ptr::addr_of_mut!(Id7) as *mut CHAR,
        Length,
        Offset,
        Buffer,
        core::ptr::addr_of_mut!(Offset),
    );

    *Buffer.offset(Offset as isize) = UNICODE_NULL;

    if !(Offset.wrapping_add(1 as ULONG) == Length) {
        RtlAssert(
            b"Offset + 1 == Length\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"pdo.c\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            387 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    (*Irp).IoStatus.Information = Buffer as _;
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_PdoHandleQueryCompatibleId(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {

    let mut PDODeviceExtension: PPDO_DEVICE_EXTENSION;
    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION;
    let mut Buffer: [CHAR; 100] = [0; 100];
    let mut Length: ULONG;
    let mut Offset: ULONG = 0;
    let mut InstanceId: LPWSTR;
    let mut DeviceType: LPCSTR;

    PDODeviceExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;
    FDODeviceExtension =
        (*(*PDODeviceExtension).LowerDeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    // ASSERT(FDODeviceExtension->DeviceDescriptor);
    let _ = &(*FDODeviceExtension).DeviceDescriptor;
    DeviceType = USBSTOR_GetDeviceType(
        core::ptr::addr_of_mut!((*PDODeviceExtension).InquiryData) as PINQUIRYDATA,
    );

    // format instance id
    Length = (sprintf(
        Buffer.as_mut_ptr(),
        b"USBSTOR\\%s\0".as_ptr() as *const CHAR,
        DeviceType,
    ) + 1) as ULONG;
    // +1 for terminating \0 and another +1 for \0 at the end of REG_MULTI_SZ
    Length = (Length as ULONG).wrapping_add(
        (sprintf(
            Buffer.as_mut_ptr().offset(Length as isize),
            b"USBSTOR\\%s\0".as_ptr() as *const CHAR,
            b"RAW\0".as_ptr() as *const CHAR,
        ) + 2) as ULONG,
    );

    InstanceId = ExAllocatePoolWithTag(
        PagedPool,
        (Length as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        USB_STOR_TAG as ULONG,
    ) as LPWSTR;
    if InstanceId.is_null() {
        (*Irp).IoStatus.Information = 0 as _;
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    USBSTOR_ConvertToUnicodeString(
        Buffer.as_mut_ptr() as *mut CHAR,
        Length,
        0 as ULONG,
        InstanceId,
        core::ptr::addr_of_mut!(Offset),
    );
    USBSTOR_ConvertToUnicodeString(
        Buffer.as_mut_ptr().offset(Offset as isize) as *mut CHAR,
        Length,
        Offset,
        InstanceId,
        core::ptr::addr_of_mut!(Offset),
    );

    *InstanceId.offset(Offset as isize) = UNICODE_NULL; // finish the REG_MULTI_SZ

    // DPRINT("USBSTOR_PdoHandleQueryCompatibleId %S\n", InstanceId);

    (*Irp).IoStatus.Information = InstanceId as _;
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_PdoHandleQueryInstanceId(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {

    let mut PDODeviceExtension: PPDO_DEVICE_EXTENSION;
    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION;
    let mut Descriptor: PUSB_STRING_DESCRIPTOR;
    let mut CharCount: ULONG;
    let mut InstanceId: LPWSTR;
    let mut Status: NTSTATUS;

    PDODeviceExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;
    FDODeviceExtension =
        (*(*PDODeviceExtension).LowerDeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;

    Descriptor = (*FDODeviceExtension).SerialNumber;
    if !Descriptor.is_null()
        && (*Descriptor).bLength as usize
            >= core::mem::size_of::<USB_COMMON_DESCRIPTOR>() + core::mem::size_of::<WCHAR>()
    {
        CharCount = (((*Descriptor).bLength as usize - core::mem::size_of::<USB_COMMON_DESCRIPTOR>())
            / core::mem::size_of::<WCHAR>()
            + (2 - 1)
            + (2 - 1)
            + core::mem::size_of::<CHAR>()) as ULONG;
    } else {
        CharCount = ((9 - 1) + (2 - 1) + (2 - 1) + core::mem::size_of::<CHAR>()) as ULONG;
    }

    InstanceId = ExAllocatePoolUninitialized(
        PagedPool,
        ((CharCount as SIZE_T) * (core::mem::size_of::<WCHAR>() as SIZE_T)) as _,
        USB_STOR_TAG as ULONG,
    ) as LPWSTR;
    if InstanceId.is_null() {
        (*Irp).IoStatus.Information = 0 as _;
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    if !Descriptor.is_null()
        && (*Descriptor).bLength as usize
            >= core::mem::size_of::<USB_COMMON_DESCRIPTOR>() + core::mem::size_of::<WCHAR>()
    {
        Status = RtlStringCchPrintfW(
            InstanceId as NTSTRSAFE_PWSTR,
            CharCount as size_t,
            core::mem::transmute::<&[u16; 6], NTSTRSAFE_PCWSTR>(&[
                0x25, 0x73, 0x26, 0x25, 0x78, 0x00,
            ]),
            core::ptr::addr_of_mut!((*Descriptor).bString) as *mut WCHAR,
            (*PDODeviceExtension).LUN as core::ffi::c_int,
        );
    } else {
        Status = RtlStringCchPrintfW(
            InstanceId as NTSTRSAFE_PWSTR,
            CharCount as size_t,
            core::mem::transmute::<&[u16; 9], NTSTRSAFE_PCWSTR>(&[
                0x25, 0x30, 0x34, 0x6c, 0x75, 0x26, 0x25, 0x78, 0x00,
            ]),
            (*FDODeviceExtension).InstanceCount,
            (*PDODeviceExtension).LUN as core::ffi::c_int,
        );
    }

    if !(Status >= 0 as core::ffi::c_long) {
        RtlAssert(
            b"NT_SUCCESS(Status)\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbstor/pdo.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            491 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    (*Irp).IoStatus.Information = InstanceId as _;
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_PdoHandleDeviceRelations(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    const USB_STOR_TAG: u32 = 0x73627375; // 'sbsu'

    let mut DeviceRelations: PDEVICE_RELATIONS = core::ptr::null_mut();
    let mut IoStack: PIO_STACK_LOCATION;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    // check if relation type is BusRelations
    if (*IoStack).Parameters.QueryDeviceRelations.Type as u32 != TargetDeviceRelation as i32 as u32 {
        // PDO handles only target device relation
        return (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    DeviceRelations = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<DEVICE_RELATIONS>(),
        USB_STOR_TAG,
    ) as PDEVICE_RELATIONS;
    if DeviceRelations.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // initialize device relations
    (*DeviceRelations).Count = 1;
    *(core::ptr::addr_of_mut!((*DeviceRelations).Objects) as *mut PDEVICE_OBJECT).offset(0) =
        DeviceObject;
    ObReferenceObject(DeviceObject as *mut core::ffi::c_void);

    (*Irp).IoStatus.Information = DeviceRelations as _;
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_PdoHandlePnp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let mut DeviceExtension: PPDO_DEVICE_EXTENSION;
    let mut Status: NTSTATUS;
    let mut Caps: PDEVICE_CAPABILITIES;
    let mut bDelete: ULONG;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    DeviceExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;
    // ASSERT(DeviceExtension->Common.IsFDO == FALSE);

    match (*IoStack).MinorFunction {
        IRP_MN_QUERY_DEVICE_RELATIONS => {
            Status = USBSTOR_PdoHandleDeviceRelations(DeviceObject, Irp);
        }
        IRP_MN_QUERY_DEVICE_TEXT => {
            Status = USBSTOR_PdoHandleQueryDeviceText(DeviceObject, Irp);
        }
        IRP_MN_QUERY_ID => {
            if (*IoStack).Parameters.QueryId.IdType as u32 == BusQueryDeviceID as u32 {
                Status = USBSTOR_PdoHandleQueryDeviceId(DeviceObject, Irp);
            } else if (*IoStack).Parameters.QueryId.IdType as u32 == BusQueryHardwareIDs as u32 {
                Status = USBSTOR_PdoHandleQueryHardwareId(DeviceObject, Irp);
            } else if (*IoStack).Parameters.QueryId.IdType as u32 == BusQueryInstanceID as u32 {
                Status = USBSTOR_PdoHandleQueryInstanceId(DeviceObject, Irp);
            } else if (*IoStack).Parameters.QueryId.IdType as u32 == BusQueryCompatibleIDs as u32 {
                Status = USBSTOR_PdoHandleQueryCompatibleId(DeviceObject, Irp);
            } else {
                // DPRINT1("USBSTOR_PdoHandlePnp: IRP_MN_QUERY_ID IdType %x unimplemented\n", IoStack->Parameters.QueryId.IdType);
                Status = STATUS_NOT_SUPPORTED;
                (*Irp).IoStatus.Information = 0 as _;
            }
        }
        IRP_MN_REMOVE_DEVICE => {
            // DPRINT("IRP_MN_REMOVE_DEVICE\n");
            if !(*(*DeviceExtension).PDODeviceObject).is_null() {
                *(*DeviceExtension).PDODeviceObject = core::ptr::null_mut();
                bDelete = 1 as ULONG;
            } else {
                // device object already marked for deletion
                bDelete = 0 as ULONG;
            }

            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

            if bDelete != 0 {
                IoDeleteDevice(DeviceObject);
            }
            return STATUS_SUCCESS;
        }
        IRP_MN_QUERY_CAPABILITIES => {
            // just forward irp to lower device
            Status = STATUS_UNSUCCESSFUL;

            if IoForwardIrpSynchronously((*DeviceExtension).LowerDeviceObject, Irp) != 0 {
                Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
                // ASSERT(Status == STATUS_SUCCESS);

                if NT_SUCCESS(Status) {
                    // check if no unique id
                    Caps = (*IoStack).Parameters.DeviceCapabilities.Capabilities;
                    (*Caps).set_UniqueID(0 as ULONG); // no unique id is supported
                    (*Caps).set_Removable(1 as ULONG); //FIXME
                }
            }
        }
        IRP_MN_QUERY_REMOVE_DEVICE | IRP_MN_QUERY_STOP_DEVICE => {
            if (*DeviceExtension).Claimed != 0 {
                Status = STATUS_UNSUCCESSFUL;
                // DPRINT1("[USBSTOR] Request %x fails because device is still claimed\n", IoStack->MinorFunction);
            } else {
                Status = STATUS_SUCCESS;
            }
        }
        IRP_MN_START_DEVICE => {
            // no-op for PDO
            Status = STATUS_SUCCESS;
        }
        IRP_MN_SURPRISE_REMOVAL => {
            Status = STATUS_SUCCESS;
        }
        _ => {
            // do nothing
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }
    }

    if Status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBSTOR_SyncCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Ctx: PVOID,
) -> NTSTATUS {
    KeSetEvent(Ctx as PKEVENT as PVOID, IO_NO_INCREMENT as i32, 0 as u8);
    return STATUS_MORE_PROCESSING_REQUIRED;
}
unsafe fn USBSTOR_SendInternalCdb(
    PdoDevice: PDEVICE_OBJECT,
    Cdb: PCDB,
    CdbLength: UCHAR,
    TimeOutValue: ULONG,
    OutDataBuffer: PVOID,
    OutDataTransferLength: PULONG,
) -> NTSTATUS {

    const SENSE_BUFFER_SIZE: ULONG = 18;
    const USB_STOR_TAG: ULONG = 0x73627375; // 'sbsu'
    const SRB_FUNCTION_EXECUTE_SCSI: u32 = 0x00;
    const SRB_FLAGS_DATA_IN: u32 = 0x00000040;
    const SRB_FLAGS_NO_QUEUE_FREEZE: u32 = 0x00000100;
    const SRB_STATUS_SUCCESS: i32 = 0x01;
    const SRB_STATUS_DATA_OVERRUN: i32 = 0x12;
    const SRB_STATUS_AUTOSENSE_VALID: i32 = 0x80;
    const SRB_STATUS_QUEUE_FROZEN: i32 = 0x40;

    let mut Srb: PSCSI_REQUEST_BLOCK;
    let SenseBuffer: PSENSE_DATA;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Irp: PIRP = core::ptr::null_mut();
    let mut Mdl: PMDL = core::ptr::null_mut();
    let mut ix: ULONG = 0;
    let mut Status: NTSTATUS = STATUS_INSUFFICIENT_RESOURCES;
    let mut SrbStatus: UCHAR;

    Srb = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<SCSI_REQUEST_BLOCK>(),
        USB_STOR_TAG,
    ) as PSCSI_REQUEST_BLOCK;

    if !Srb.is_null() {
        SenseBuffer =
            ExAllocatePoolWithTag(NonPagedPool, SENSE_BUFFER_SIZE as usize, USB_STOR_TAG)
                as PSENSE_DATA;

        if !SenseBuffer.is_null() {
            Mdl = IoAllocateMdl(
                OutDataBuffer,
                *OutDataTransferLength,
                0,
                0,
                core::ptr::null_mut(),
            );

            if Mdl.is_null() {
                ExFreePoolWithTag(SenseBuffer as PVOID, USB_STOR_TAG);
                ExFreePoolWithTag(Srb as PVOID, USB_STOR_TAG);
                return Status;
            }

            MmBuildMdlForNonPagedPool(Mdl);

            loop {
                Irp = IoAllocateIrp((*PdoDevice).StackSize, 0);

                if Irp.is_null() {
                    break;
                }

                IoStack = IoGetNextIrpStackLocation(Irp);
                (*IoStack).MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL as UCHAR;
                (*IoStack).Parameters.Scsi.Srb = Srb as *mut ntkrnl_sys::ddk::_SCSI_REQUEST_BLOCK;

                RtlZeroMemory(
                    Srb as PVOID,
                    core::mem::size_of::<SCSI_REQUEST_BLOCK>(),
                );

                (*Srb).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
                (*Srb).Function = SRB_FUNCTION_EXECUTE_SCSI as UCHAR;
                (*Srb).CdbLength = CdbLength;
                (*Srb).SenseInfoBufferLength = SENSE_BUFFER_SIZE as UCHAR;
                (*Srb).SrbFlags = (SRB_FLAGS_DATA_IN | SRB_FLAGS_NO_QUEUE_FREEZE) as ULONG;
                (*Srb).DataTransferLength = *OutDataTransferLength;
                (*Srb).TimeOutValue = TimeOutValue;
                (*Srb).DataBuffer = OutDataBuffer;
                (*Srb).SenseInfoBuffer = SenseBuffer as PVOID;

                RtlCopyMemory(
                    core::ptr::addr_of_mut!((*Srb).Cdb) as PVOID,
                    Cdb as PVOID,
                    CdbLength as usize,
                );

                (*Irp).MdlAddress = Mdl;

                KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_SynchronizationEvent, 0);

                IoSetCompletionRoutine(
                    Irp,
                    Some(USBSTOR_SyncCompletionRoutine),
                    core::ptr::addr_of_mut!(Event) as PVOID,
                    1,
                    1,
                    1,
                );

                if IoCallDriver(PdoDevice, Irp) == STATUS_PENDING {
                    KeWaitForSingleObject(
                        core::ptr::addr_of_mut!(Event) as PVOID,
                        _KWAIT_REASON_Executive,
                        0i32 as _,
                        0,
                        core::ptr::null_mut(),
                    );
                }

                SrbStatus = ((*Srb).SrbStatus as i32
                    & !(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN))
                    as UCHAR;

                IoFreeIrp(Irp);
                Irp = core::ptr::null_mut();

                if SrbStatus as i32 == SRB_STATUS_SUCCESS
                    || SrbStatus as i32 == SRB_STATUS_DATA_OVERRUN
                {
                    Status = STATUS_SUCCESS;
                    *OutDataTransferLength = (*Srb).DataTransferLength;
                    break;
                }

                Status = STATUS_UNSUCCESSFUL;

                ix = ix.wrapping_add(1);
                if !(ix < 3) {
                    break;
                }
            }

            if !Mdl.is_null() {
                IoFreeMdl(Mdl);
            }

            ExFreePoolWithTag(SenseBuffer as PVOID, USB_STOR_TAG);
        }

        ExFreePoolWithTag(Srb as PVOID, USB_STOR_TAG);
    }

    return Status;
}
unsafe fn USBSTOR_FillInquiryData(PDODeviceObject: PDEVICE_OBJECT) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_INSUFFICIENT_RESOURCES;
    let PDODeviceExtension: PPDO_DEVICE_EXTENSION =
        (*PDODeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;
    let mut Cdb: CDB = core::mem::zeroed();
    let mut DataTransferLength: ULONG = INQUIRYDATABUFFERSIZE as ULONG;
    let InquiryData: PINQUIRYDATA =
        core::ptr::addr_of_mut!((*PDODeviceExtension).InquiryData) as PINQUIRYDATA;

    RtlZeroMemory(
        core::ptr::addr_of_mut!(Cdb) as PVOID,
        core::mem::size_of::<CDB>() as _,
    );
    Cdb.CDB6INQUIRY.as_mut().OperationCode = SCSIOP_INQUIRY as UCHAR;
    Cdb.CDB6INQUIRY.as_mut().AllocationLength = INQUIRYDATABUFFERSIZE as UCHAR;

    Status = USBSTOR_SendInternalCdb(
        PDODeviceObject,
        core::ptr::addr_of_mut!(Cdb),
        CDB6GENERIC_LENGTH as UCHAR,
        20 as ULONG,
        InquiryData as PVOID,
        core::ptr::addr_of_mut!(DataTransferLength),
    );

    if !NT_SUCCESS(Status) {
        return Status;
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_CreatePDO(
    DeviceObject: PDEVICE_OBJECT,
    LUN: UCHAR,
) -> NTSTATUS {
    const FILE_DEVICE_MASS_STORAGE: u32 = 0x0000002d;
    const DIRECT_ACCESS_DEVICE: i32 = 0x00;
    const READ_ONLY_DIRECT_ACCESS_DEVICE: i32 = 0x05;

    let mut PDO: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut PDODeviceExtension: PPDO_DEVICE_EXTENSION;
    let FDODeviceExtension: PFDO_DEVICE_EXTENSION;
    let InquiryData: PINQUIRYDATA;

    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;

    // create child device object
    Status = IoCreateDevice(
        (*DeviceObject).DriverObject,
        core::mem::size_of::<PDO_DEVICE_EXTENSION>() as u32,
        core::ptr::null_mut(),
        FILE_DEVICE_MASS_STORAGE,
        (FILE_AUTOGENERATED_DEVICE_NAME | FILE_DEVICE_SECURE_OPEN) as u32,
        0,
        core::ptr::addr_of_mut!(PDO),
    );
    if !NT_SUCCESS(Status) {
        if DbgPrint(
            b"(%s:%d) Failed to create PDO, status %x\n\0".as_ptr() as PCSTR,
            b"pdo.c\0".as_ptr() as *const core::ffi::c_char,
            880,
            Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"pdo.c\0".as_ptr() as *const core::ffi::c_char,
                880,
            );
        }
        return Status;
    }

    // patch the stack size
    (*PDO).StackSize = (*DeviceObject).StackSize;

    PDODeviceExtension = (*PDO).DeviceExtension as PPDO_DEVICE_EXTENSION;
    InquiryData = core::ptr::addr_of_mut!((*PDODeviceExtension).InquiryData) as PINQUIRYDATA;

    // initialize device extension
    RtlZeroMemory(
        PDODeviceExtension as *mut core::ffi::c_void,
        core::mem::size_of::<PDO_DEVICE_EXTENSION>(),
    );
    (*PDODeviceExtension).Common.IsFDO = 0;
    (*PDODeviceExtension).LowerDeviceObject = DeviceObject;
    (*PDODeviceExtension).PDODeviceObject =
        (core::ptr::addr_of_mut!((*FDODeviceExtension).ChildPDO) as *mut PDEVICE_OBJECT)
            .offset(LUN as isize) as *mut PDEVICE_OBJECT;
    (*PDODeviceExtension).Self_ = PDO;
    (*PDODeviceExtension).LUN = LUN;

    (*PDO).Flags |= DO_DIRECT_IO as u32;

    // device is initialized
    (*PDO).Flags &= !(DO_DEVICE_INITIALIZING as u32);

    // output device object
    (*FDODeviceExtension).ChildPDO[LUN as usize] = PDO;

    // send inquiry command by irp
    Status = USBSTOR_FillInquiryData(PDO);

    if !NT_SUCCESS(Status) {
        return Status;
    }

    if (*InquiryData).DeviceType() as i32 != DIRECT_ACCESS_DEVICE
        && (*InquiryData).DeviceType() as i32 != READ_ONLY_DIRECT_ACCESS_DEVICE
    {
        return STATUS_NOT_SUPPORTED;
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
//  *              2019 Victor Perevertkin (victor.perevertkin@reactos.org)
//  */
//
// #include "usbstor.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// static
// LPCSTR
// USBSTOR_GetDeviceType(
//     IN PINQUIRYDATA InquiryData)
// {
//     switch (InquiryData->DeviceType)
//     {
//         case DIRECT_ACCESS_DEVICE:
//             return "Disk";
//         case SEQUENTIAL_ACCESS_DEVICE:
//             // sequential device, i.e magnetic tape
//             return "Sequential";
//         case WRITE_ONCE_READ_MULTIPLE_DEVICE:
//             return "Worm";
//         case READ_ONLY_DIRECT_ACCESS_DEVICE:
//             return "CdRom";
//         case OPTICAL_DEVICE:
//             return "Optical";
//         case MEDIUM_CHANGER:
//             return "Changer";
//         default:
//             return "Other";
//     }
// }
//
// static
// LPCSTR
// USBSTOR_GetGenericType(
//     IN PINQUIRYDATA InquiryData)
// {
//     switch (InquiryData->DeviceType)
//     {
//         case DIRECT_ACCESS_DEVICE:
//             return "GenDisk";
//         case SEQUENTIAL_ACCESS_DEVICE:
//             // sequential device, i.e magnetic tape
//             return "GenSequential";
//         case WRITE_ONCE_READ_MULTIPLE_DEVICE:
//             return "GenWorm";
//         case READ_ONLY_DIRECT_ACCESS_DEVICE:
//             return "GenCdRom";
//         case OPTICAL_DEVICE:
//             return "GenOptical";
//         case MEDIUM_CHANGER:
//             return "GenChanger";
//         default:
//             return "UsbstorOther";
//     }
// }
//
// static
// ULONG
// CopyField(
//     IN PUCHAR Name,
//     IN PCHAR Buffer,
//     IN ULONG MaxLength)
// {
//     ULONG Index;
//
//     for (Index = 0; Index < MaxLength; Index++)
//     {
//         if (Name[Index] <= ' ' || Name[Index] >= 0x7F /* last printable ascii character */ ||  Name[Index] == ',')
//         {
//             // convert to underscore
//             Buffer[Index] = '_';
//         }
//         else
//         {
//             // just copy character
//             Buffer[Index] = Name[Index];
//         }
//     }
//
//     return MaxLength;
// }
//
// static
// ULONG
// CopyFieldTruncate(
//     IN PUCHAR Name,
//     IN PCHAR Buffer,
//     IN ULONG MaxLength)
// {
//     ULONG Index;
//
//     for (Index = 0; Index < MaxLength; Index++)
//     {
//         if (Name[Index] == '\0')
//         {
//             break;
//         }
//         else if (Name[Index] <= ' ' || Name[Index] >= 0x7F /* last printable ascii character */ ||  Name[Index] == ',')
//         {
//             // convert to underscore
//             Buffer[Index] = ' ';
//         }
//         else
//         {
//             // just copy character
//             Buffer[Index] = Name[Index];
//         }
//     }
//
//     return Index;
// }
//
// NTSTATUS
// USBSTOR_PdoHandleQueryDeviceText(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     PPDO_DEVICE_EXTENSION DeviceExtension;
//     PIO_STACK_LOCATION IoStack;
//     CHAR LocalBuffer[26];
//     UINT32 Offset = 0;
//     PINQUIRYDATA InquiryData;
//     ANSI_STRING AnsiString;
//     UNICODE_STRING DeviceDescription;
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     DeviceExtension = (PPDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     InquiryData = (PINQUIRYDATA)&DeviceExtension->InquiryData;
//
//     switch (IoStack->Parameters.QueryDeviceText.DeviceTextType)
//     {
//         case DeviceTextDescription:
//         case DeviceTextLocationInformation:
//         {
//             DPRINT("USBSTOR_PdoHandleQueryDeviceText\n");
//
//             Offset += CopyFieldTruncate(InquiryData->VendorId, &LocalBuffer[Offset], sizeof(InquiryData->VendorId));
//             LocalBuffer[Offset++] = ' ';
//             Offset += CopyFieldTruncate(InquiryData->ProductId, &LocalBuffer[Offset], sizeof(InquiryData->ProductId));
//             LocalBuffer[Offset++] = '\0';
//
//             RtlInitAnsiString(&AnsiString, (PCSZ)&LocalBuffer);
//
//             DeviceDescription.Length = 0;
//             DeviceDescription.MaximumLength = (USHORT)(Offset * sizeof(WCHAR));
//             DeviceDescription.Buffer = ExAllocatePoolWithTag(PagedPool, DeviceDescription.MaximumLength, USB_STOR_TAG);
//             if (!DeviceDescription.Buffer)
//             {
//                 Irp->IoStatus.Information = 0;
//                 return STATUS_INSUFFICIENT_RESOURCES;
//             }
//
//             RtlAnsiStringToUnicodeString(&DeviceDescription, &AnsiString, FALSE);
//
//             Irp->IoStatus.Information = (ULONG_PTR)DeviceDescription.Buffer;
//             return STATUS_SUCCESS;
//         }
//         default:
//         {
//             Irp->IoStatus.Information = 0;
//             return Irp->IoStatus.Status;
//         }
//     }
// }
//
// NTSTATUS
// USBSTOR_PdoHandleQueryDeviceId(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     PPDO_DEVICE_EXTENSION DeviceExtension;
//     NTSTATUS Status;
//     CHAR Buffer[100] = {0};
//     LPCSTR DeviceType;
//     ULONG Offset = 0;
//     PINQUIRYDATA InquiryData;
//     ANSI_STRING AnsiString;
//     UNICODE_STRING DeviceId;
//
//     DeviceExtension = (PPDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     InquiryData = (PINQUIRYDATA)&DeviceExtension->InquiryData;
//
//     DeviceType = USBSTOR_GetDeviceType(InquiryData);
//
//     // lets create device string
//     Offset = sprintf(&Buffer[Offset], "USBSTOR\\");
//     Offset += sprintf(&Buffer[Offset], DeviceType);
//     Offset += sprintf(&Buffer[Offset], "&Ven_");
//     Offset += CopyField(InquiryData->VendorId, &Buffer[Offset], 8);
//     Offset += sprintf(&Buffer[Offset], "&Prod_");
//     Offset += CopyField(InquiryData->ProductId, &Buffer[Offset], 16);
//     Offset += sprintf(&Buffer[Offset], "&Rev_");
//     Offset += CopyField(InquiryData->ProductRevisionLevel, &Buffer[Offset], 4);
//
//     RtlInitAnsiString(&AnsiString, (PCSZ)Buffer);
//
//     // allocate DeviceId string
//     DeviceId.Length = 0;
//     DeviceId.MaximumLength = (USHORT)((strlen((PCHAR)Buffer) + 1) * sizeof(WCHAR));
//     DeviceId.Buffer = ExAllocatePoolWithTag(PagedPool, DeviceId.MaximumLength, USB_STOR_TAG);
//     if (!DeviceId.Buffer)
//     {
//         Irp->IoStatus.Information = 0;
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     Status = RtlAnsiStringToUnicodeString(&DeviceId, &AnsiString, FALSE);
//
//     if (NT_SUCCESS(Status))
//     {
//         Irp->IoStatus.Information = (ULONG_PTR)DeviceId.Buffer;
//     }
//
//     DPRINT("DeviceId %wZ Status %x\n", &DeviceId, Status);
//
//     return Status;
// }
//
// VOID
// USBSTOR_ConvertToUnicodeString(
//     IN CHAR * Buffer,
//     IN ULONG ResultBufferLength,
//     IN ULONG ResultBufferOffset,
//     OUT LPWSTR ResultBuffer,
//     OUT PULONG NewResultBufferOffset)
// {
//     UNICODE_STRING DeviceString;
//     ANSI_STRING AnsiString;
//     NTSTATUS Status;
//
//     ASSERT(ResultBufferLength);
//     ASSERT(ResultBufferLength > ResultBufferOffset);
//
//     DPRINT("ResultBufferOffset %lu ResultBufferLength %lu Buffer %s Length %lu\n", ResultBufferOffset, ResultBufferLength, Buffer, strlen(Buffer));
//
//     // construct destination string
//     DeviceString.Buffer = &ResultBuffer[ResultBufferOffset];
//     DeviceString.Length = 0;
//     DeviceString.MaximumLength = (ResultBufferLength - ResultBufferOffset) * sizeof(WCHAR);
//
//     // initialize source string
//     RtlInitAnsiString(&AnsiString, Buffer);
//
//     Status = RtlAnsiStringToUnicodeString(&DeviceString, &AnsiString, FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//
//     // subtract consumed bytes
//     ResultBufferLength -= (DeviceString.Length + sizeof(WCHAR)) / sizeof(WCHAR);
//     ResultBufferOffset += (DeviceString.Length + sizeof(WCHAR)) / sizeof(WCHAR);
//
//     *NewResultBufferOffset = ResultBufferOffset;
// }
//
// NTSTATUS
// USBSTOR_PdoHandleQueryHardwareId(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp)
// {
//     PPDO_DEVICE_EXTENSION PDODeviceExtension;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//     LPCSTR GenericType, DeviceType;
//     LPWSTR Buffer;
//     CHAR Id1[50], Id2[50], Id3[50], Id4[50], Id5[50], Id6[50], Id7[50];
//     ULONG Id1Length, Id2Length, Id3Length, Id4Length, Id5Length, Id6Length, Id7Length;
//     ULONG Offset, TotalLength, Length;
//     PINQUIRYDATA InquiryData;
//
//     PDODeviceExtension = (PPDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)PDODeviceExtension->LowerDeviceObject->DeviceExtension;
//     ASSERT(FDODeviceExtension->DeviceDescriptor);
//     InquiryData = (PINQUIRYDATA)&PDODeviceExtension->InquiryData;
//
//     DeviceType = USBSTOR_GetDeviceType(InquiryData);
//     GenericType = USBSTOR_GetGenericType(InquiryData);
//
//     ASSERT(GenericType);
//
//     // generate id 1
//     // USBSTOR\SCSIType_VendorId(8)_ProductId(16)_Revision(4)
//     RtlZeroMemory(Id1, sizeof(Id1));
//     Offset = 0;
//     Offset = sprintf(&Id1[Offset], "USBSTOR\\");
//     Offset += sprintf(&Id1[Offset], DeviceType);
//     Offset += CopyField(InquiryData->VendorId, &Id1[Offset], 8);
//     Offset += CopyField(InquiryData->ProductId, &Id1[Offset], 16);
//     Offset += CopyField(InquiryData->ProductRevisionLevel, &Id1[Offset], 4);
//     Id1Length = strlen(Id1) + 1;
//     DPRINT("USBSTOR_PdoHandleQueryHardwareId HardwareId1 %s\n", Id1);
//
//     // generate id 2
//     // USBSTOR\SCSIType_VendorId(8)_ProductId(16)
//     RtlZeroMemory(Id2, sizeof(Id2));
//     Offset = 0;
//     Offset = sprintf(&Id2[Offset], "USBSTOR\\");
//     Offset += sprintf(&Id2[Offset], DeviceType);
//     Offset += CopyField(InquiryData->VendorId, &Id2[Offset], 8);
//     Offset += CopyField(InquiryData->ProductId, &Id2[Offset], 16);
//     Id2Length = strlen(Id2) + 1;
//     DPRINT("USBSTOR_PdoHandleQueryHardwareId HardwareId2 %s\n", Id2);
//
//     // generate id 3
//     // USBSTOR\SCSIType_VendorId(8)
//     RtlZeroMemory(Id3, sizeof(Id3));
//     Offset = 0;
//     Offset = sprintf(&Id3[Offset], "USBSTOR\\");
//     Offset += sprintf(&Id3[Offset], DeviceType);
//     Offset += CopyField(InquiryData->VendorId, &Id3[Offset], 8);
//     Id3Length = strlen(Id3) + 1;
//     DPRINT("USBSTOR_PdoHandleQueryHardwareId HardwareId3 %s\n", Id3);
//
//     // generate id 4
//     // USBSTOR\SCSIType_VendorId(8)_ProductId(16)_Revision(1)
//     RtlZeroMemory(Id4, sizeof(Id4));
//     Offset = 0;
//     Offset = sprintf(&Id4[Offset], "USBSTOR\\");
//     Offset += sprintf(&Id4[Offset], DeviceType);
//     Offset += CopyField(InquiryData->VendorId, &Id4[Offset], 8);
//     Offset += CopyField(InquiryData->ProductId, &Id4[Offset], 16);
//     Offset += CopyField(InquiryData->ProductRevisionLevel, &Id4[Offset], 1);
//     Id4Length = strlen(Id4) + 1;
//     DPRINT("USBSTOR_PdoHandleQueryHardwareId HardwareId4 %s\n", Id4);
//
//     // generate id 5
//     // SCSIType_VendorId(8)_ProductId(16)_Revision(1)
//     RtlZeroMemory(Id5, sizeof(Id5));
//     Offset = 0;
//     Offset = sprintf(&Id5[Offset], DeviceType);
//     Offset += CopyField(InquiryData->VendorId, &Id5[Offset], 8);
//     Offset += CopyField(InquiryData->ProductId, &Id5[Offset], 16);
//     Offset += CopyField(InquiryData->ProductRevisionLevel, &Id5[Offset], 1);
//     Id5Length = strlen(Id5) + 1;
//     DPRINT("USBSTOR_PdoHandleQueryHardwareId HardwareId5 %s\n", Id5);
//
//     // generate id 6
//     // USBSTOR\SCSIType
//     RtlZeroMemory(Id6, sizeof(Id6));
//     Offset = 0;
//     Offset = sprintf(&Id6[Offset], "USBSTOR\\");
//     Offset += sprintf(&Id6[Offset], GenericType);
//     Id6Length = strlen(Id6) + 1;
//     DPRINT("USBSTOR_PdoHandleQueryHardwareId HardwareId6 %s\n", Id6);
//
//     // generate id 7
//     // SCSIType
//     RtlZeroMemory(Id7, sizeof(Id7));
//     Offset = 0;
//     Offset = sprintf(&Id7[Offset], GenericType);
//     Id7Length = strlen(Id7) + 1;
//     DPRINT("USBSTOR_PdoHandleQueryHardwareId HardwareId7 %s\n", Id7);
//
//     // last +1 is for terminating \0 of REG_MULTI_SZ
//     TotalLength = Id1Length + Id2Length + Id3Length + Id4Length + Id5Length + Id6Length + Id7Length + 1;
//
//     Buffer = ExAllocatePoolWithTag(PagedPool, TotalLength * sizeof(WCHAR), USB_STOR_TAG);
//     if (!Buffer)
//     {
//         Irp->IoStatus.Information = 0;
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     // reset offset
//     Offset = 0;
//     Length = TotalLength;
//
//     USBSTOR_ConvertToUnicodeString(Id1, Length, Offset, Buffer, &Offset);
//     USBSTOR_ConvertToUnicodeString(Id2, Length, Offset, Buffer, &Offset);
//     USBSTOR_ConvertToUnicodeString(Id3, Length, Offset, Buffer, &Offset);
//     USBSTOR_ConvertToUnicodeString(Id4, Length, Offset, Buffer, &Offset);
//     USBSTOR_ConvertToUnicodeString(Id5, Length, Offset, Buffer, &Offset);
//     USBSTOR_ConvertToUnicodeString(Id6, Length, Offset, Buffer, &Offset);
//     USBSTOR_ConvertToUnicodeString(Id7, Length, Offset, Buffer, &Offset);
//
//     Buffer[Offset] = UNICODE_NULL; // finish the REG_MULTI_SZ
//
//     ASSERT(Offset + 1 == Length);
//
//     Irp->IoStatus.Information = (ULONG_PTR)Buffer;
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// USBSTOR_PdoHandleQueryCompatibleId(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp)
// {
//     PPDO_DEVICE_EXTENSION PDODeviceExtension;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//     CHAR Buffer[100] = {0};
//     ULONG Length, Offset;
//     LPWSTR InstanceId;
//     LPCSTR DeviceType;
//
//     PDODeviceExtension = (PPDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)PDODeviceExtension->LowerDeviceObject->DeviceExtension;
//     ASSERT(FDODeviceExtension->DeviceDescriptor);
//     DeviceType = USBSTOR_GetDeviceType((PINQUIRYDATA)&PDODeviceExtension->InquiryData);
//
//     // format instance id
//     Length = sprintf(Buffer, "USBSTOR\\%s", DeviceType) + 1;
//     // +1 for terminating \0 and another +1 for \0 at the end of REG_MULTI_SZ
//     Length += sprintf(&Buffer[Length], "USBSTOR\\%s", "RAW") + 2;
//
//     InstanceId = ExAllocatePoolWithTag(PagedPool, Length * sizeof(WCHAR), USB_STOR_TAG);
//     if (!InstanceId)
//     {
//         Irp->IoStatus.Information = 0;
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     USBSTOR_ConvertToUnicodeString(Buffer, Length, 0, InstanceId, &Offset);
//     USBSTOR_ConvertToUnicodeString(&Buffer[Offset], Length, Offset, InstanceId, &Offset);
//
//     InstanceId[Offset] = UNICODE_NULL; // finish the REG_MULTI_SZ
//
//     DPRINT("USBSTOR_PdoHandleQueryCompatibleId %S\n", InstanceId);
//
//     Irp->IoStatus.Information = (ULONG_PTR)InstanceId;
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// USBSTOR_PdoHandleQueryInstanceId(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp)
// {
//     PPDO_DEVICE_EXTENSION PDODeviceExtension;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//     PUSB_STRING_DESCRIPTOR Descriptor;
//     ULONG CharCount;
//     LPWSTR InstanceId;
//     NTSTATUS Status;
//
//     PDODeviceExtension = DeviceObject->DeviceExtension;
//     FDODeviceExtension = PDODeviceExtension->LowerDeviceObject->DeviceExtension;
//
//     Descriptor = FDODeviceExtension->SerialNumber;
//     if (Descriptor && (Descriptor->bLength >= sizeof(USB_COMMON_DESCRIPTOR) + sizeof(WCHAR)))
//     {
//         /* Format the serial number descriptor only if supported by the device */
//         CharCount = (Descriptor->bLength - sizeof(USB_COMMON_DESCRIPTOR)) / sizeof(WCHAR) +
//                     (sizeof("&") - 1) +
//                     (sizeof("F") - 1) + // LUN: 1 char (MAX_LUN)
//                     sizeof(ANSI_NULL);
//     }
//     else
//     {
//         /* Use the instance count and LUN as a fallback */
//         CharCount = (sizeof("99999999") - 1) + // Instance Count: 8 chars
//                     (sizeof("&") - 1) +
//                     (sizeof("F") - 1) + // LUN: 1 char (MAX_LUN)
//                     sizeof(ANSI_NULL);
//     }
//
//     InstanceId = ExAllocatePoolUninitialized(PagedPool, CharCount * sizeof(WCHAR), USB_STOR_TAG);
//     if (!InstanceId)
//     {
//         Irp->IoStatus.Information = 0;
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     if (Descriptor && (Descriptor->bLength >= sizeof(USB_COMMON_DESCRIPTOR) + sizeof(WCHAR)))
//     {
//         Status = RtlStringCchPrintfW(InstanceId,
//                                      CharCount,
//                                      L"%s&%x",
//                                      Descriptor->bString,
//                                      PDODeviceExtension->LUN);
//     }
//     else
//     {
//         Status = RtlStringCchPrintfW(InstanceId,
//                                      CharCount,
//                                      L"%04lu&%x",
//                                      FDODeviceExtension->InstanceCount,
//                                      PDODeviceExtension->LUN);
//     }
//
//     /* This should not happen */
//     ASSERT(NT_SUCCESS(Status));
//
//     DPRINT("USBSTOR_PdoHandleQueryInstanceId '%S'\n", InstanceId);
//
//     Irp->IoStatus.Information = (ULONG_PTR)InstanceId;
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// USBSTOR_PdoHandleDeviceRelations(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp)
// {
//     PDEVICE_RELATIONS DeviceRelations;
//     PIO_STACK_LOCATION IoStack;
//
//     DPRINT("USBSTOR_PdoHandleDeviceRelations\n");
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     // check if relation type is BusRelations
//     if (IoStack->Parameters.QueryDeviceRelations.Type != TargetDeviceRelation)
//     {
//         // PDO handles only target device relation
//         return Irp->IoStatus.Status;
//     }
//
//     DeviceRelations = ExAllocatePoolWithTag(PagedPool, sizeof(DEVICE_RELATIONS), USB_STOR_TAG);
//     if (!DeviceRelations)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     // initialize device relations
//     DeviceRelations->Count = 1;
//     DeviceRelations->Objects[0] = DeviceObject;
//     ObReferenceObject(DeviceObject);
//
//     Irp->IoStatus.Information = (ULONG_PTR)DeviceRelations;
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// USBSTOR_PdoHandlePnp(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     PPDO_DEVICE_EXTENSION DeviceExtension;
//     NTSTATUS Status;
//     PDEVICE_CAPABILITIES Caps;
//     ULONG bDelete;
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     DeviceExtension = (PPDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT(DeviceExtension->Common.IsFDO == FALSE);
//
//     switch(IoStack->MinorFunction)
//     {
//        case IRP_MN_QUERY_DEVICE_RELATIONS:
//        {
//            Status = USBSTOR_PdoHandleDeviceRelations(DeviceObject, Irp);
//            break;
//        }
//        case IRP_MN_QUERY_DEVICE_TEXT:
//        {
//            Status = USBSTOR_PdoHandleQueryDeviceText(DeviceObject, Irp);
//            break;
//        }
//        case IRP_MN_QUERY_ID:
//        {
//            if (IoStack->Parameters.QueryId.IdType == BusQueryDeviceID)
//            {
//                Status = USBSTOR_PdoHandleQueryDeviceId(DeviceObject, Irp);
//                break;
//            }
//            else if (IoStack->Parameters.QueryId.IdType == BusQueryHardwareIDs)
//            {
//                Status = USBSTOR_PdoHandleQueryHardwareId(DeviceObject, Irp);
//                break;
//            }
//            else if (IoStack->Parameters.QueryId.IdType == BusQueryInstanceID)
//            {
//                Status = USBSTOR_PdoHandleQueryInstanceId(DeviceObject, Irp);
//                break;
//            }
//            else if (IoStack->Parameters.QueryId.IdType == BusQueryCompatibleIDs)
//            {
//                Status = USBSTOR_PdoHandleQueryCompatibleId(DeviceObject, Irp);
//                break;
//            }
//
//            DPRINT1("USBSTOR_PdoHandlePnp: IRP_MN_QUERY_ID IdType %x unimplemented\n", IoStack->Parameters.QueryId.IdType);
//            Status = STATUS_NOT_SUPPORTED;
//            Irp->IoStatus.Information = 0;
//            break;
//        }
//        case IRP_MN_REMOVE_DEVICE:
//        {
//            DPRINT("IRP_MN_REMOVE_DEVICE\n");
//
//            if(*DeviceExtension->PDODeviceObject != NULL)
//            {
//                *DeviceExtension->PDODeviceObject = NULL;
//                bDelete = TRUE;
//            }
//            else
//            {
//                // device object already marked for deletion
//                bDelete = FALSE;
//            }
//
//            Irp->IoStatus.Status = STATUS_SUCCESS;
//            IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//            if (bDelete)
//            {
//                IoDeleteDevice(DeviceObject);
//            }
//            return STATUS_SUCCESS;
//        }
//        case IRP_MN_QUERY_CAPABILITIES:
//        {
//            // just forward irp to lower device
//             Status = STATUS_UNSUCCESSFUL;
//
//             if (IoForwardIrpSynchronously(DeviceExtension->LowerDeviceObject, Irp))
//             {
//                 Status = Irp->IoStatus.Status;
//                 ASSERT(Status == STATUS_SUCCESS);
//
//                 if (NT_SUCCESS(Status))
//                 {
//                     // check if no unique id
//                     Caps = (PDEVICE_CAPABILITIES)IoStack->Parameters.DeviceCapabilities.Capabilities;
//                     Caps->UniqueID = FALSE; // no unique id is supported
//                     Caps->Removable = TRUE; //FIXME
//                 }
//             }
//            break;
//        }
//        case IRP_MN_QUERY_REMOVE_DEVICE:
//        case IRP_MN_QUERY_STOP_DEVICE:
//        {
//            if (DeviceExtension->Claimed)
//            {
//                Status = STATUS_UNSUCCESSFUL;
//                DPRINT1("[USBSTOR] Request %x fails because device is still claimed\n", IoStack->MinorFunction);
//            }
//            else
//                Status = STATUS_SUCCESS;
//            break;
//        }
//        case IRP_MN_START_DEVICE:
//        {
//            // no-op for PDO
//            Status = STATUS_SUCCESS;
//            break;
//        }
//        case IRP_MN_SURPRISE_REMOVAL:
//        {
//            Status = STATUS_SUCCESS;
//            break;
//        }
//        default:
//         {
//             // do nothing
//             Status = Irp->IoStatus.Status;
//         }
//     }
//
//     if (Status != STATUS_PENDING)
//     {
//         Irp->IoStatus.Status = Status;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBSTOR_SyncCompletionRoutine(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp,
//     IN PVOID Ctx)
// {
//     KeSetEvent((PKEVENT)Ctx, IO_NO_INCREMENT, FALSE);
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
// /*
// * @name USBSTOR_SendInternalCdb
// *
// * Issues an internal SCSI request to device.
// * The request is sent in a synchronous way.
// */
// static
// NTSTATUS
// USBSTOR_SendInternalCdb(
//     IN PDEVICE_OBJECT PdoDevice,
//     IN PCDB Cdb,
//     IN UCHAR CdbLength,
//     IN ULONG TimeOutValue,
//     OUT PVOID OutDataBuffer,
//     OUT PULONG OutDataTransferLength)
// {
//     PSCSI_REQUEST_BLOCK Srb;
//     PSENSE_DATA SenseBuffer;
//     PIO_STACK_LOCATION IoStack;
//     KEVENT Event;
//     PIRP Irp = NULL;
//     PMDL Mdl = NULL;
//     ULONG ix = 0;
//     NTSTATUS Status = STATUS_INSUFFICIENT_RESOURCES;
//     UCHAR SrbStatus;
//
//     DPRINT("USBSTOR_SendInternalCdb SCSIOP %x\n", Cdb->CDB6GENERIC.OperationCode);
//
//     Srb = ExAllocatePoolWithTag(NonPagedPool,
//                                 sizeof(SCSI_REQUEST_BLOCK),
//                                 USB_STOR_TAG);
//
//     if (Srb)
//     {
//         SenseBuffer = ExAllocatePoolWithTag(NonPagedPool,
//                                             SENSE_BUFFER_SIZE,
//                                             USB_STOR_TAG);
//
//         if (SenseBuffer)
//         {
//             Mdl = IoAllocateMdl(OutDataBuffer,
//                     *OutDataTransferLength,
//                     FALSE,
//                     FALSE,
//                     NULL);
//
//             if (!Mdl)
//             {
//                 ExFreePoolWithTag(SenseBuffer, USB_STOR_TAG);
//                 ExFreePoolWithTag(Srb, USB_STOR_TAG);
//                 return Status;
//             }
//
//             MmBuildMdlForNonPagedPool(Mdl);
//
//             // make 3 attempts - the device may be in STALL state after the first one
//             do
//             {
//                 Irp = IoAllocateIrp(PdoDevice->StackSize, FALSE);
//
//                 if (!Irp)
//                 {
//                     break;
//                 }
//
//                 IoStack = IoGetNextIrpStackLocation(Irp);
//                 IoStack->MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
//                 IoStack->Parameters.Scsi.Srb = Srb;
//
//                 RtlZeroMemory(Srb, sizeof(SCSI_REQUEST_BLOCK));
//
//                 Srb->Length = sizeof(SCSI_REQUEST_BLOCK);
//                 Srb->Function = SRB_FUNCTION_EXECUTE_SCSI;
//                 Srb->CdbLength = CdbLength;
//                 Srb->SenseInfoBufferLength = SENSE_BUFFER_SIZE;
//                 Srb->SrbFlags = SRB_FLAGS_DATA_IN | SRB_FLAGS_NO_QUEUE_FREEZE;
//                 Srb->DataTransferLength = *OutDataTransferLength;
//                 Srb->TimeOutValue = TimeOutValue;
//                 Srb->DataBuffer = OutDataBuffer;
//                 Srb->SenseInfoBuffer = SenseBuffer;
//
//                 RtlCopyMemory(Srb->Cdb, Cdb, CdbLength);
//
//                 Irp->MdlAddress = Mdl;
//
//                 KeInitializeEvent(&Event, SynchronizationEvent, FALSE);
//
//                 IoSetCompletionRoutine(Irp,
//                                        USBSTOR_SyncCompletionRoutine,
//                                        &Event,
//                                        TRUE,
//                                        TRUE,
//                                        TRUE);
//
//                 if (IoCallDriver(PdoDevice, Irp) == STATUS_PENDING)
//                 {
//                     KeWaitForSingleObject(&Event,
//                                           Executive,
//                                           KernelMode,
//                                           FALSE,
//                                           NULL);
//                 }
//
//                 SrbStatus = SRB_STATUS(Srb->SrbStatus);
//
//                 IoFreeIrp(Irp);
//                 Irp = NULL;
//
//                 if (SrbStatus == SRB_STATUS_SUCCESS ||
//                      SrbStatus == SRB_STATUS_DATA_OVERRUN)
//                 {
//                     Status = STATUS_SUCCESS;
//                     *OutDataTransferLength = Srb->DataTransferLength;
//                     break;
//                 }
//
//                 Status = STATUS_UNSUCCESSFUL;
//
//                 ++ix;
//             } while (ix < 3);
//
//             if (Mdl)
//             {
//                 IoFreeMdl(Mdl);
//             }
//
//             ExFreePoolWithTag(SenseBuffer, USB_STOR_TAG);
//         }
//
//         ExFreePoolWithTag(Srb, USB_STOR_TAG);
//     }
//
//     return Status;
// }
//
// /*
// * @name USBSTOR_FillInquiryData
// *
// * Sends a SCSI Inquiry request and fills in the PDODeviceExtension->InquiryData field with a result.
// */
// static
// NTSTATUS
// USBSTOR_FillInquiryData(
//     IN PDEVICE_OBJECT PDODeviceObject)
// {
//     NTSTATUS Status = STATUS_INSUFFICIENT_RESOURCES;
//     PPDO_DEVICE_EXTENSION PDODeviceExtension = (PPDO_DEVICE_EXTENSION)PDODeviceObject->DeviceExtension;
//     CDB Cdb;
//     ULONG DataTransferLength = INQUIRYDATABUFFERSIZE;
//     PINQUIRYDATA InquiryData = (PINQUIRYDATA)&PDODeviceExtension->InquiryData;
//
//     RtlZeroMemory(&Cdb, sizeof(Cdb));
//     Cdb.CDB6INQUIRY.OperationCode = SCSIOP_INQUIRY;
//     Cdb.CDB6INQUIRY.AllocationLength = INQUIRYDATABUFFERSIZE;
//
//     Status = USBSTOR_SendInternalCdb(PDODeviceObject, &Cdb, CDB6GENERIC_LENGTH, 20, InquiryData, &DataTransferLength);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("USBSTOR_FillInquiryData failed with %x\n", Status);
//         return Status;
//     }
//
//     DPRINT("DeviceType %x\n", InquiryData->DeviceType);
//     DPRINT("DeviceTypeModifier %x\n", InquiryData->DeviceTypeModifier);
//     DPRINT("RemovableMedia %x\n", InquiryData->RemovableMedia);
//     DPRINT("Version %x\n", InquiryData->Versions);
//     DPRINT("Format %x\n", InquiryData->ResponseDataFormat);
//     DPRINT("Length %x\n", InquiryData->AdditionalLength);
//     DPRINT("Reserved %p\n", InquiryData->Reserved);
//     DPRINT("VendorId %c%c%c%c%c%c%c%c\n", InquiryData->VendorId[0], InquiryData->VendorId[1], InquiryData->VendorId[2], InquiryData->VendorId[3], InquiryData->VendorId[4], InquiryData->VendorId[5], InquiryData->VendorId[6], InquiryData->VendorId[7]);
//     DPRINT("ProductId %c%c%c%c%c%c%c%c%c%c%c%c%c%c%c%c\n", InquiryData->ProductId[0], InquiryData->ProductId[1], InquiryData->ProductId[2], InquiryData->ProductId[3],
//                                                            InquiryData->ProductId[4], InquiryData->ProductId[5], InquiryData->ProductId[6], InquiryData->ProductId[7],
//                                                            InquiryData->ProductId[8], InquiryData->ProductId[9], InquiryData->ProductId[10], InquiryData->ProductId[11],
//                                                            InquiryData->ProductId[12], InquiryData->ProductId[13], InquiryData->ProductId[14], InquiryData->ProductId[15]);
//
//     DPRINT("Revision %c%c%c%c\n", InquiryData->ProductRevisionLevel[0], InquiryData->ProductRevisionLevel[1], InquiryData->ProductRevisionLevel[2], InquiryData->ProductRevisionLevel[3]);
//
//     return Status;
// }
//
// NTSTATUS
// USBSTOR_CreatePDO(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN UCHAR LUN)
// {
//     PDEVICE_OBJECT PDO;
//     NTSTATUS Status;
//     PPDO_DEVICE_EXTENSION PDODeviceExtension;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//     PINQUIRYDATA InquiryData;
//
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
//     // create child device object
//     Status = IoCreateDevice(DeviceObject->DriverObject, sizeof(PDO_DEVICE_EXTENSION), NULL, FILE_DEVICE_MASS_STORAGE, FILE_AUTOGENERATED_DEVICE_NAME | FILE_DEVICE_SECURE_OPEN, FALSE, &PDO);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create PDO, status %x\n", Status);
//         return Status;
//     }
//
//     // patch the stack size
//     PDO->StackSize = DeviceObject->StackSize;
//
//     PDODeviceExtension = (PPDO_DEVICE_EXTENSION)PDO->DeviceExtension;
//     InquiryData = (PINQUIRYDATA)&PDODeviceExtension->InquiryData;
//
//     // initialize device extension
//     RtlZeroMemory(PDODeviceExtension, sizeof(PDO_DEVICE_EXTENSION));
//     PDODeviceExtension->Common.IsFDO = FALSE;
//     PDODeviceExtension->LowerDeviceObject = DeviceObject;
//     PDODeviceExtension->PDODeviceObject = &FDODeviceExtension->ChildPDO[LUN];
//     PDODeviceExtension->Self = PDO;
//     PDODeviceExtension->LUN = LUN;
//
//     PDO->Flags |= DO_DIRECT_IO;
//
//     // device is initialized
//     PDO->Flags &= ~DO_DEVICE_INITIALIZING;
//
//     // output device object
//     FDODeviceExtension->ChildPDO[LUN] = PDO;
//
//     // send inquiry command by irp
//     Status = USBSTOR_FillInquiryData(PDO);
//
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     if (InquiryData->DeviceType != DIRECT_ACCESS_DEVICE &&
//         InquiryData->DeviceType != READ_ONLY_DIRECT_ACCESS_DEVICE)
//     {
//         return STATUS_NOT_SUPPORTED;
//     }
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================