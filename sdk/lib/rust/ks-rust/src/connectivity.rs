use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{ACCESS_MASK, GUID, IoGetCurrentIrpStackLocation, LPGUID, LPWSTR, _OBJECT_ATTRIBUTES, PHANDLE, PIO_STACK_LOCATION, PIRP, PKEY_VALUE_PARTIAL_INFORMATION, RtlAppendUnicodeStringToString, RtlFreeUnicodeString, RtlStringFromGUID, ZwClose, ZwOpenKey, ZwQueryValueKey};
use core::ffi::c_void;

// ---- ks-rust integration glue (connectivity.rs-local): imports, aliases, helper shims ----
// Shared allocator helpers (defined #[no_mangle] in misc.rs).
use crate::misc::{AllocateItem, FreeItem, KspCopyCreateRequest};
// Cross-module Ks* handler (defined #[no_mangle] in topology.rs).
use crate::topology::KspCreateObjectType;
// _KEY_VALUE_INFORMATION_CLASS member (bindgen names it <EnumTag>_<member>).
use ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as KeyValuePartialInformation;
// KSPROPERTY_PIN enum-member bare aliases (bindgen names them <EnumTag>_<member>; CTYPES via glue).
use crate::types::KSPROPERTY_PIN_KSPROPERTY_PIN_DATAFLOW as KSPROPERTY_PIN_DATAFLOW;
use crate::types::KSPROPERTY_PIN_KSPROPERTY_PIN_DATARANGES as KSPROPERTY_PIN_DATARANGES;
use crate::types::KSPROPERTY_PIN_KSPROPERTY_PIN_CONSTRAINEDDATARANGES as KSPROPERTY_PIN_CONSTRAINEDDATARANGES;
use crate::types::KSPROPERTY_PIN_KSPROPERTY_PIN_INTERFACES as KSPROPERTY_PIN_INTERFACES;
use crate::types::KSPROPERTY_PIN_KSPROPERTY_PIN_MEDIUMS as KSPROPERTY_PIN_MEDIUMS;
use crate::types::KSPROPERTY_PIN_KSPROPERTY_PIN_COMMUNICATION as KSPROPERTY_PIN_COMMUNICATION;
use crate::types::KSPROPERTY_PIN_KSPROPERTY_PIN_CATEGORY as KSPROPERTY_PIN_CATEGORY;
use crate::types::KSPROPERTY_PIN_KSPROPERTY_PIN_NAME as KSPROPERTY_PIN_NAME;
use crate::types::KSPROPERTY_PIN_KSPROPERTY_PIN_PROPOSEDATAFORMAT as KSPROPERTY_PIN_PROPOSEDATAFORMAT;

// IsEqualGUIDAligned: guiddef inline GUID equality.
#[inline(always)]
unsafe fn IsEqualGUIDAligned(a: *const GUID, b: *const GUID) -> bool {
    (*a).Data1 == (*b).Data1
        && (*a).Data2 == (*b).Data2
        && (*a).Data3 == (*b).Data3
        && (*a).Data4 == (*b).Data4
}

// RTL_CONSTANT_STRING(L!("...")) helper: wide static -> _UNICODE_STRING (lengths in bytes).
#[inline(always)]
unsafe fn rtl_const_string(w: &'static [u16]) -> _UNICODE_STRING {
    _UNICODE_STRING {
        Length: ((w.len() - 1) * 2) as u16,
        MaximumLength: (w.len() * 2) as u16,
        Buffer: w.as_ptr() as *mut u16,
    }
}

// Audio format types (ksmedia.h pshpack1 / mmreg.h WAVEFORMATEX) — used only in this module.
#[repr(C, packed)]
pub struct WAVEFORMATEX {
    pub wFormatTag: u16,
    pub nChannels: u16,
    pub nSamplesPerSec: u32,
    pub nAvgBytesPerSec: u32,
    pub nBlockAlign: u16,
    pub wBitsPerSample: u16,
    pub cbSize: u16,
}
#[repr(C, packed)]
pub struct KSDATAFORMAT_WAVEFORMATEX {
    pub DataFormat: KSDATAFORMAT,
    pub WaveFormatEx: WAVEFORMATEX,
}
pub type PKSDATAFORMAT_WAVEFORMATEX = *mut KSDATAFORMAT_WAVEFORMATEX;
#[repr(C)]
pub struct KSDATARANGE_AUDIO {
    pub DataRange: KSDATARANGE,
    pub MaximumChannels: ULONG,
    pub MinimumBitsPerSample: ULONG,
    pub MaximumBitsPerSample: ULONG,
    pub MinimumSampleFrequency: ULONG,
    pub MaximumSampleFrequency: ULONG,
}
pub type PKSDATARANGE_AUDIO = *mut KSDATARANGE_AUDIO;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const KSSTRING_Pin: [::core::ffi::c_ushort; 39] = unsafe {
    ::core::mem::transmute::<
        [u8; 78],
        [::core::ffi::c_ushort; 39],
    >(
        *b"{\x001\x004\x006\0F\x001\0A\08\x000\0-\x004\x007\09\x001\0-\x001\x001\0D\x000\0-\0A\x005\0D\x006\0-\x002\08\0D\0B\x000\x004\0C\x001\x000\x000\x000\x000\0}\0\0\0",
    )
};

pub static mut StandardPinInterface: KSPIN_INTERFACE = KSPIN_INTERFACE {
    Set: GUID {
        Data1: 0x1a8766a0 as ::core::ffi::c_long as ::core::ffi::c_uint,
        Data2: 0x62ce as ::core::ffi::c_ushort,
        Data3: 0x11cf as ::core::ffi::c_ushort,
        Data4: [
            0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0xd6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0x28 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0xdb as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0x4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        ],
    },
    Id: KSINTERFACE_STANDARD_STREAMING as ::core::ffi::c_int as ULONG,
    Flags: 0 as ULONG,
};

pub static mut StandardPinMedium: KSPIN_MEDIUM = KSPIN_MEDIUM {
    Set: GUID {
        Data1: 0x4747b320 as ::core::ffi::c_long as ::core::ffi::c_uint,
        Data2: 0x62ce as ::core::ffi::c_ushort,
        Data3: 0x11cf as ::core::ffi::c_ushort,
        Data4: [
            0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0xd6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0x28 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0xdb as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0x4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        ],
    },
    Id: KSMEDIUM_TYPE_ANYINSTANCE as ULONG,
    Flags: 0 as ULONG,
};

pub static mut KSDATAFORMAT_SUBTYPE_BDA_MPEG2_TRANSPORT: GUID = GUID {
    Data1: 0xf4aeb342 as ::core::ffi::c_uint,
    Data2: 0x329 as ::core::ffi::c_ushort,
    Data3: 0x4fdd as ::core::ffi::c_ushort,
    Data4: [
        0xa8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xff as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x49 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x26 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x78 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[repr(C)]
pub struct KSPIN_INTERFACE {
    pub Set: GUID,
    pub Id: u32,
    pub Flags: u32,
}
pub type PKSPIN_INTERFACE = *mut KSPIN_INTERFACE;

#[repr(C)]
pub struct KSPIN_MEDIUM {
    pub Set: GUID,
    pub Id: u32,
    pub Flags: u32,
}
pub type PKSPIN_MEDIUM = *mut KSPIN_MEDIUM;

#[no_mangle]
pub unsafe extern "system" fn KsCreatePin(
    FilterHandle: HANDLE,
    Connect: PKSPIN_CONNECT,
    DesiredAccess: ACCESS_MASK,
    ConnectionHandle: PHANDLE,
) -> NTSTATUS {
    let mut ConnectSize: UINT = core::mem::size_of::<KSPIN_CONNECT>() as UINT;
    let Format: PKSDATAFORMAT_WAVEFORMATEX = Connect.offset(1) as PKSDATAFORMAT_WAVEFORMATEX;
    // DataFormat.FormatSize is at offset 0; WaveFormatEx.cbSize lives in a packed struct (read unaligned).
    let format_size: ULONG = core::ptr::read_unaligned(Format as *const ULONG);
    let cb_size: u16 = core::ptr::read_unaligned(core::ptr::addr_of!((*Format).WaveFormatEx.cbSize));
    if format_size == core::mem::size_of::<KSDATAFORMAT>() as ULONG
        || format_size as usize
            == (core::mem::size_of::<KSDATAFORMAT>() as usize)
                .wrapping_add(core::mem::size_of::<WAVEFORMATEX>() as usize)
                .wrapping_add(cb_size as usize)
    {
        ConnectSize = ConnectSize.wrapping_add(format_size as UINT);
    }
    KspCreateObjectType(
        FilterHandle,
        KSSTRING_Pin.as_ptr() as LPWSTR,
        Connect as PVOID,
        ConnectSize,
        DesiredAccess,
        ConnectionHandle,
    )
}
#[no_mangle]
pub unsafe extern "C" fn KspValidateConnectRequest(
    Irp: PIRP,
    DescriptorsCount: ULONG,
    Descriptors: PVOID,
    DescriptorSize: ULONG,
    Connect: *mut PKSPIN_CONNECT,
) -> NTSTATUS {
    let mut ConnectDetails: PKSPIN_CONNECT = core::ptr::null_mut();
    let mut Interface: PKSPIN_INTERFACE = core::ptr::null_mut();
    let mut Medium: PKSPIN_MEDIUM = core::ptr::null_mut();
    let mut Size: ULONG;
    let mut Status: NTSTATUS;
    let mut Index: ULONG;
    let mut Count: ULONG;
    let mut Found: BOOLEAN;
    let mut Descriptor: PKSPIN_DESCRIPTOR;
    let mut GuidString2: _UNICODE_STRING = core::mem::zeroed();

    if Connect.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    Size = core::mem::size_of::<KSPIN_CONNECT>() as ULONG;

    Status = KspCopyCreateRequest(
        Irp,
        KSSTRING_Pin.as_ptr() as LPWSTR,
        &mut Size,
        &mut ConnectDetails as *mut PKSPIN_CONNECT as *mut PVOID,
    );

    if !NT_SUCCESS(Status) {
        return Status;
    }

    if (*ConnectDetails).PinId >= DescriptorsCount {
        FreeItem(ConnectDetails as PVOID);
        return STATUS_INVALID_PARAMETER;
    }

    if DescriptorSize as usize == core::mem::size_of::<KSPIN_DESCRIPTOR>() as usize {
        Descriptor = (Descriptors as usize)
            .wrapping_add(
                core::mem::size_of::<KSPIN_DESCRIPTOR>() as usize
                    * (*ConnectDetails).PinId as usize,
            ) as PKSPIN_DESCRIPTOR;
    } else {
        Descriptor = &mut (*((Descriptors as usize)
            .wrapping_add(DescriptorSize as usize * (*ConnectDetails).PinId as usize)
            as PKSPIN_DESCRIPTOR_EX))
        .PinDescriptor as PKSPIN_DESCRIPTOR;
    }

    if (*Descriptor).InterfacesCount != 0 && !(*Descriptor).Interfaces.is_null() {
        Count = (*Descriptor).InterfacesCount;
        Interface = (*Descriptor).Interfaces as PKSPIN_INTERFACE;
    } else {
        Count = 1;
        Interface = &mut StandardPinInterface as PKSPIN_INTERFACE;
    }

    Found = FALSE;
    Index = 0;
    RtlStringFromGUID(&mut (*ConnectDetails).Interface.Set, &mut GuidString2);

    loop {
        let mut GuidString: _UNICODE_STRING = core::mem::zeroed();
        RtlStringFromGUID(&mut (*Interface.offset(Index as isize)).Set, &mut GuidString);
        RtlFreeUnicodeString(&mut GuidString);

        if IsEqualGUIDAligned(
            &mut (*Interface.offset(Index as isize)).Set,
            &mut (*ConnectDetails).Interface.Set,
        ) && (*Interface.offset(Index as isize)).Id == (*ConnectDetails).Interface.Id
        {
            Found = TRUE;
            break;
        }

        Index += 1;
        if Index >= Count {
            break;
        }
    }

    RtlFreeUnicodeString(&mut GuidString2);

    if Found == FALSE {
        FreeItem(ConnectDetails as PVOID);
        return STATUS_NO_MATCH;
    }

    if (*Descriptor).MediumsCount != 0 && !(*Descriptor).Mediums.is_null() {
        Count = (*Descriptor).MediumsCount;
        Medium = (*Descriptor).Mediums as PKSPIN_MEDIUM;
    } else {
        Count = 1;
        Medium = &mut StandardPinMedium as PKSPIN_MEDIUM;
    }

    Found = FALSE;
    Index = 0;
    RtlStringFromGUID(&mut (*ConnectDetails).Medium.Set, &mut GuidString2);

    loop {
        let mut GuidString: _UNICODE_STRING = core::mem::zeroed();
        RtlStringFromGUID(&mut (*Medium.offset(Index as isize)).Set, &mut GuidString);
        RtlFreeUnicodeString(&mut GuidString);

        if IsEqualGUIDAligned(
            &mut (*Medium.offset(Index as isize)).Set,
            &mut (*ConnectDetails).Medium.Set,
        ) && (*Medium.offset(Index as isize)).Id == (*ConnectDetails).Medium.Id
        {
            Found = TRUE;
            break;
        }

        Index += 1;
        if Index >= Count {
            break;
        }
    }

    RtlFreeUnicodeString(&mut GuidString2);

    if Found == FALSE {
        FreeItem(ConnectDetails as PVOID);
        return STATUS_NO_MATCH;
    }

    *Connect = ConnectDetails;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KsValidateConnectRequest(
    Irp: PIRP,
    DescriptorsCount: ULONG,
    Descriptor: *mut KSPIN_DESCRIPTOR,
    Connect: *mut PKSPIN_CONNECT,
) -> NTSTATUS {
    return KspValidateConnectRequest(
        Irp,
        DescriptorsCount,
        Descriptor as PVOID,
        core::mem::size_of::<KSPIN_DESCRIPTOR>() as ULONG,
        Connect,
    );
}
#[no_mangle]
pub unsafe extern "C" fn KspReadMediaCategory(
    Category: LPGUID,
    OutInformation: *mut PKEY_VALUE_PARTIAL_INFORMATION,
) -> NTSTATUS {
    let MediaPath: _UNICODE_STRING = rtl_const_string(crate::L!("\\REGISTRY\\MACHINE\\SYSTEM\\CurrentControlSet\\Control\\MediaCategories\\"));
    let mut Name: _UNICODE_STRING = rtl_const_string(crate::L!("Name"));
    let mut GuidString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Path: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut hKey: HANDLE = core::ptr::null_mut();
    let mut Size: ULONG = 0;
    let mut KeyInfo: PKEY_VALUE_PARTIAL_INFORMATION = core::ptr::null_mut();

    Status = RtlStringFromGUID(Category, &mut GuidString);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Path.Length = 0;
    Path.MaximumLength = MediaPath.MaximumLength + GuidString.MaximumLength;
    Path.Buffer = AllocateItem(NonPagedPool, Path.MaximumLength as usize) as PWCH;
    if Path.Buffer.is_null() {
        RtlFreeUnicodeString(&mut GuidString);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlAppendUnicodeStringToString(&mut Path, &MediaPath);
    RtlAppendUnicodeStringToString(&mut Path, &GuidString);
    RtlFreeUnicodeString(&mut GuidString);

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut Path;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwOpenKey(&mut hKey, GENERIC_READ, &mut ObjectAttributes);
    FreeItem(Path.Buffer as PVOID);

    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ZwQueryValueKey(hKey, &mut Name, KeyValuePartialInformation, core::ptr::null_mut(), 0, &mut Size);
    if !NT_SUCCESS(Status) && Status != STATUS_BUFFER_TOO_SMALL {
        ZwClose(hKey);
        return Status;
    }

    KeyInfo = AllocateItem(NonPagedPool, Size as usize) as PKEY_VALUE_PARTIAL_INFORMATION;
    if KeyInfo.is_null() {
        ZwClose(hKey);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = ZwQueryValueKey(hKey, &mut Name, KeyValuePartialInformation, KeyInfo as PVOID, Size, &mut Size);
    ZwClose(hKey);

    if !NT_SUCCESS(Status) {
        FreeItem(KeyInfo as PVOID);
        return Status;
    }

    *OutInformation = KeyInfo;
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn KspPinPropertyHandler(
    Irp: PIRP,
    Property: PKSPROPERTY,
    Data: PVOID,
    DescriptorsCount: ULONG,
    Descriptors: *const KSPIN_DESCRIPTOR,
    DescriptorSize: ULONG,
) -> NTSTATUS {
    let mut Pin: *mut KSP_PIN;
    let mut Item: *mut KSMULTIPLE_ITEM;
    let IoStack: PIO_STACK_LOCATION;
    let mut Size: ULONG;
    let mut Index: ULONG;
    let Buffer: PVOID;
    let mut WaveFormatOut: *mut PKSDATARANGE_AUDIO;
    let mut WaveFormatIn: PKSDATAFORMAT_WAVEFORMATEX;
    let mut KeyInfo: PKEY_VALUE_PARTIAL_INFORMATION = core::ptr::null_mut();
    let Descriptor: *const KSPIN_DESCRIPTOR;
    let mut Status: NTSTATUS;
    let mut Count: ULONG;
    let mut DataRanges: *const PKSDATARANGE;
    let mut Guid: LPGUID;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    Buffer = Data;

    Pin = Property as *mut KSP_PIN;

    if (*Property).Id != KSPROPERTY_PIN_CTYPES {
        if (*Pin).PinId >= DescriptorsCount {
            return STATUS_INVALID_PARAMETER;
        }
    } else {
        *(Buffer as *mut ULONG) = DescriptorsCount;
        (*Irp).IoStatus.Information = core::mem::size_of::<ULONG>() as _;
        return STATUS_SUCCESS;
    }

    if DescriptorSize == core::mem::size_of::<KSPIN_DESCRIPTOR>() as ULONG {
        Descriptor = &*Descriptors.offset((*Pin).PinId as isize) as *const KSPIN_DESCRIPTOR;
    } else {
        Descriptor = &(*((Descriptors as ULONG_PTR).wrapping_add((*Pin).PinId as usize * DescriptorSize as usize) as *const KSPIN_DESCRIPTOR_EX)).PinDescriptor;
    }

    Status = STATUS_NOT_SUPPORTED;

    match (*Property).Id {
        KSPROPERTY_PIN_DATAFLOW => {
            Size = core::mem::size_of::<KSPIN_DATAFLOW>() as ULONG;
            if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength < Size {
                (*Irp).IoStatus.Information = Size as _;
                Status = STATUS_BUFFER_TOO_SMALL;
            } else {
                *(Buffer as *mut KSPIN_DATAFLOW) = (*Descriptor).DataFlow;
                (*Irp).IoStatus.Information = core::mem::size_of::<KSPIN_DATAFLOW>() as _;
                Status = STATUS_SUCCESS;
            }
        }

        KSPROPERTY_PIN_DATARANGES | KSPROPERTY_PIN_CONSTRAINEDDATARANGES => {
            Size = core::mem::size_of::<KSMULTIPLE_ITEM>() as ULONG;

            if (*Property).Id == KSPROPERTY_PIN_DATARANGES || (*Descriptor).__bindgen_anon_1.__bindgen_anon_1.as_ref().ConstrainedDataRangesCount == 0 {
                DataRanges = (*Descriptor).DataRanges;
                Count = (*Descriptor).DataRangesCount;
            } else {
                DataRanges = (*Descriptor).__bindgen_anon_1.__bindgen_anon_1.as_ref().ConstrainedDataRanges as *const PKSDATARANGE;
                Count = (*Descriptor).__bindgen_anon_1.__bindgen_anon_1.as_ref().ConstrainedDataRangesCount;
            }

            Index = 0;
            while Index < Count {
                Size = Size.wrapping_add(((*(*DataRanges.offset(Index as isize))).__bindgen_anon_1.as_ref().FormatSize + 0x7) & !0x7);
                Index = Index.wrapping_add(1);
            }

            if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength == 0 {
                (*Irp).IoStatus.Information = Size as _;
                Status = STATUS_BUFFER_OVERFLOW;
            } else {
                Item = Buffer as *mut KSMULTIPLE_ITEM;

                if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength == core::mem::size_of::<ULONG>() as ULONG {
                    (*Item).Size = Size;
                    (*Irp).IoStatus.Information = core::mem::size_of::<ULONG>() as _;
                    Status = STATUS_SUCCESS;
                } else {
                    (*Item).Size = Size;
                    (*Item).Count = Count;

                    if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength == core::mem::size_of::<KSMULTIPLE_ITEM>() as ULONG {
                        (*Irp).IoStatus.Information = core::mem::size_of::<KSMULTIPLE_ITEM>() as _;
                        Status = STATUS_SUCCESS;
                    } else {
                        let mut DataPtr = Item.offset(1) as *mut u8;

                        Index = 0;
                        while Index < Count {
                            let mut GuidString: _UNICODE_STRING = core::mem::zeroed();
                            let dr = *DataRanges.offset(Index as isize);
                            RtlStringFromGUID(&(*dr).__bindgen_anon_1.as_ref().MajorFormat, &mut GuidString);
                            RtlStringFromGUID(&(*dr).__bindgen_anon_1.as_ref().SubFormat, &mut GuidString);
                            RtlStringFromGUID(&(*dr).__bindgen_anon_1.as_ref().Specifier, &mut GuidString);
                            RtlStringFromGUID(&(*dr).__bindgen_anon_1.as_ref().Specifier, &mut GuidString);

                            let format_size = (*dr).__bindgen_anon_1.as_ref().FormatSize;
                            core::ptr::copy_nonoverlapping(
                                dr as *const u8,
                                DataPtr,
                                format_size as usize,
                            );
                            DataPtr = DataPtr.offset(format_size as isize);
                            DataPtr = (((DataPtr as ULONG_PTR).wrapping_add(7)) & !(7 as ULONG_PTR)) as *mut u8;
                            Index = Index.wrapping_add(1);
                        }

                        Status = STATUS_SUCCESS;
                        (*Irp).IoStatus.Information = Size as _;
                    }
                }
            }
        }

        KSPROPERTY_PIN_INTERFACES => {
            if !(*Descriptor).Interfaces.is_null() {
                return KsHandleSizedListQuery(Irp, (*Descriptor).InterfacesCount, core::mem::size_of::<KSPIN_MEDIUM>() as ULONG, (*Descriptor).Interfaces as *const core::ffi::c_void);
            } else {
                return KsHandleSizedListQuery(Irp, 1, core::mem::size_of::<KSPIN_INTERFACE>() as ULONG, &StandardPinInterface as *const _ as *const core::ffi::c_void);
            }
        }

        KSPROPERTY_PIN_MEDIUMS => {
            if (*Descriptor).MediumsCount != 0 {
                return KsHandleSizedListQuery(Irp, (*Descriptor).MediumsCount, core::mem::size_of::<KSPIN_MEDIUM>() as ULONG, (*Descriptor).Mediums as *const core::ffi::c_void);
            } else {
                return KsHandleSizedListQuery(Irp, 1, core::mem::size_of::<KSPIN_MEDIUM>() as ULONG, &StandardPinMedium as *const _ as *const core::ffi::c_void);
            }
        }

        KSPROPERTY_PIN_COMMUNICATION => {
            Size = core::mem::size_of::<KSPIN_COMMUNICATION>() as ULONG;
            if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength < Size {
                (*Irp).IoStatus.Information = Size as _;
                Status = STATUS_BUFFER_TOO_SMALL;
            } else {
                *(Buffer as *mut KSPIN_COMMUNICATION) = (*Descriptor).Communication;
                Status = STATUS_SUCCESS;
                (*Irp).IoStatus.Information = Size as _;
            }
        }

        KSPROPERTY_PIN_CATEGORY => {
            if (*Descriptor).Category.is_null() {
                return STATUS_NOT_FOUND;
            }

            Size = core::mem::size_of::<GUID>() as ULONG;
            if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength < Size {
                (*Irp).IoStatus.Information = Size as _;
                Status = STATUS_BUFFER_TOO_SMALL;
            } else {
                core::ptr::copy_nonoverlapping(
                    (*Descriptor).Category as *const u8,
                    Buffer as *mut u8,
                    core::mem::size_of::<GUID>(),
                );
                Status = STATUS_SUCCESS;
                (*Irp).IoStatus.Information = Size as _;
            }
        }

        KSPROPERTY_PIN_NAME => {
            if !(*Descriptor).Name.is_null() {
                Guid = (*Descriptor).Name as LPGUID;
            } else {
                Guid = (*Descriptor).Category as LPGUID;
            }

            if Guid.is_null() {
                return STATUS_NOT_FOUND;
            }

            Status = KspReadMediaCategory(Guid, &mut KeyInfo);
            if !NT_SUCCESS(Status) {
                (*Irp).IoStatus.Information = 0;
            } else {
                (*Irp).IoStatus.Information = ((*KeyInfo).DataLength as usize).wrapping_add(core::mem::size_of::<WCHAR>()) as _;

                if ((*KeyInfo).DataLength as usize).wrapping_add(core::mem::size_of::<WCHAR>()) > (*IoStack).Parameters.DeviceIoControl.OutputBufferLength as usize {
                    Status = STATUS_BUFFER_OVERFLOW;
                    FreeItem(KeyInfo as PVOID);
                } else {
                    core::ptr::copy_nonoverlapping(
                        &(*KeyInfo).Data as *const _ as *const u8,
                        (*Irp).AssociatedIrp.SystemBuffer as *mut u8,
                        (*KeyInfo).DataLength as usize,
                    );
                    *((*Irp).AssociatedIrp.SystemBuffer as *mut WCHAR).offset((*KeyInfo).DataLength as isize / core::mem::size_of::<WCHAR>() as isize) = 0;
                    FreeItem(KeyInfo as PVOID);
                }
            }
        }

        KSPROPERTY_PIN_PROPOSEDATAFORMAT => {
            Size = core::mem::size_of::<KSDATAFORMAT>() as ULONG;
            if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength < Size {
                (*Irp).IoStatus.Information = Size as _;
                Status = STATUS_BUFFER_TOO_SMALL;
            } else if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength != core::mem::size_of::<KSDATAFORMAT_WAVEFORMATEX>() as ULONG {
                Status = STATUS_NOT_IMPLEMENTED;
                (*Irp).IoStatus.Information = 0;
            } else {
                WaveFormatIn = Buffer as PKSDATAFORMAT_WAVEFORMATEX;
                if (*Descriptor).DataRanges.is_null() || (*Descriptor).DataRangesCount == 0 {
                    Status = STATUS_UNSUCCESSFUL;
                    (*Irp).IoStatus.Information = 0;
                } else {
                    WaveFormatOut = (*Descriptor).DataRanges as *mut PKSDATARANGE_AUDIO;
                    Index = 0;
                    while Index < (*Descriptor).DataRangesCount {
                        let wfo = *WaveFormatOut.offset(Index as isize);
                        if (*wfo).DataRange.__bindgen_anon_1.as_ref().FormatSize != core::mem::size_of::<KSDATARANGE_AUDIO>() as ULONG {
                            Index = Index.wrapping_add(1);
                            continue;
                        }

                        let in_samples_per_sec = core::ptr::read_unaligned(core::ptr::addr_of!((*WaveFormatIn).WaveFormatEx.nSamplesPerSec));
                        let in_bits_per_sample = core::ptr::read_unaligned(core::ptr::addr_of!((*WaveFormatIn).WaveFormatEx.wBitsPerSample));
                        let in_channels = core::ptr::read_unaligned(core::ptr::addr_of!((*WaveFormatIn).WaveFormatEx.nChannels));
                        if (*wfo).MinimumSampleFrequency > in_samples_per_sec ||
                            (*wfo).MaximumSampleFrequency < in_samples_per_sec ||
                            (*wfo).MinimumBitsPerSample > in_bits_per_sample as ULONG ||
                            (*wfo).MaximumBitsPerSample < in_bits_per_sample as ULONG ||
                            (*wfo).MaximumChannels < in_channels as ULONG
                        {
                            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NO_MATCH;
                            (*Irp).IoStatus.Information = 0;
                            return STATUS_NO_MATCH;
                        } else {
                            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
                            (*Irp).IoStatus.Information = 0;
                            return STATUS_SUCCESS;
                        }
                    }
                    Status = STATUS_NO_MATCH;
                    (*Irp).IoStatus.Information = 0;
                }
            }
        }

        _ => {
            Status = STATUS_NOT_IMPLEMENTED;
            (*Irp).IoStatus.Information = 0;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsPinPropertyHandler(
    Irp: PIRP,
    Property: PKSPROPERTY,
    Data: PVOID,
    DescriptorsCount: ULONG,
    Descriptor: *const KSPIN_DESCRIPTOR,
) -> NTSTATUS {
    return KspPinPropertyHandler(
        Irp,
        Property,
        Data,
        DescriptorsCount,
        Descriptor,
        core::mem::size_of::<KSPIN_DESCRIPTOR>() as ULONG,
    );
}

#[no_mangle]
pub unsafe extern "system" fn KsPinDataIntersectionEx(
    Irp: PIRP,
    Pin: PKSP_PIN,
    Data: PVOID,
    DescriptorsCount: ULONG,
    Descriptor: *const KSPIN_DESCRIPTOR,
    DescriptorSize: ULONG,
    IntersectHandler: PFNKSINTERSECTHANDLEREX,
    HandlerContext: PVOID,
) -> NTSTATUS {
    STATUS_UNSUCCESSFUL
}

#[no_mangle]
pub unsafe extern "system" fn KsPinDataIntersection(
    Irp: PIRP,
    Pin: PKSP_PIN,
    Data: PVOID,
    DescriptorsCount: ULONG,
    _Descriptor: *const KSPIN_DESCRIPTOR,
    IntersectHandler: PFNKSINTERSECTHANDLER,
) -> NTSTATUS {
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let Size = (core::mem::size_of::<KSP_PIN>() as ULONG)
        .wrapping_add(core::mem::size_of::<KSMULTIPLE_ITEM>() as ULONG)
        .wrapping_add(core::mem::size_of::<KSDATARANGE>() as ULONG);

    if (*IoStack).Parameters.DeviceIoControl.InputBufferLength < Size {
        (*Irp).IoStatus.Information = Size as _;
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_BUFFER_TOO_SMALL;
        return STATUS_BUFFER_TOO_SMALL;
    }

    if (*Pin).PinId >= DescriptorsCount {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
        (*Irp).IoStatus.Information = 0;
        return STATUS_INVALID_PARAMETER;
    }

    let Item = Pin.add(1) as *mut KSMULTIPLE_ITEM;
    let mut DataRange = Item.add(1) as *mut KSDATARANGE;

    let mut Index: ULONG = 0;
    while Index < (*Item).Count {
        let Status = IntersectHandler.expect("non-null function pointer")(Irp, Pin, DataRange as PKSDATARANGE, Data);
        if NT_SUCCESS(Status) {
            if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength < (*DataRange).__bindgen_anon_1.as_ref().FormatSize {
                (*Irp).IoStatus.Information = (*DataRange).__bindgen_anon_1.as_ref().FormatSize as _;
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_BUFFER_TOO_SMALL;
                return STATUS_BUFFER_TOO_SMALL;
            }
            core::ptr::copy_nonoverlapping(
                DataRange as *const u8,
                (*Irp).UserBuffer as *mut u8,
                core::mem::size_of::<KSDATARANGE>(),
            );
            (*Irp).IoStatus.Information = core::mem::size_of::<KSDATARANGE>() as _;
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            return STATUS_SUCCESS;
        }
        Index = Index.wrapping_add(1);
        DataRange = DataRange.add(1);
    }

    (*Irp).IoStatus.Information = 0;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NO_MATCH;
    STATUS_NO_MATCH
}

#[no_mangle]
pub unsafe extern "system" fn KsHandleSizedListQuery(
    Irp: PIRP,
    DataItemsCount: ULONG,
    DataItemSize: ULONG,
    DataItems: *const core::ffi::c_void,
) -> NTSTATUS {
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let Size = DataItemSize.wrapping_mul(DataItemsCount).wrapping_add(size_of::<KSMULTIPLE_ITEM>() as ULONG);
    let Item = (*Irp).AssociatedIrp.SystemBuffer as PKSMULTIPLE_ITEM;

    if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength == 0 {
        (*Irp).IoStatus.Information = Size as _;
        return STATUS_BUFFER_OVERFLOW;
    }

    if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength == size_of::<ULONG>() as ULONG {
        (*Item).Size = Size;
        (*Irp).IoStatus.Information = size_of::<ULONG>() as _;
        return STATUS_SUCCESS;
    }

    if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength < size_of::<KSMULTIPLE_ITEM>() as ULONG {
        return STATUS_BUFFER_TOO_SMALL;
    }

    (*Item).Count = DataItemsCount;
    (*Item).Size = DataItemSize;

    if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength == size_of::<KSMULTIPLE_ITEM>() as ULONG {
        (*Irp).IoStatus.Information = size_of::<KSMULTIPLE_ITEM>() as _;
        return STATUS_SUCCESS;
    }

    if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength >= Size {
        core::ptr::copy_nonoverlapping(
            DataItems,
            Item.offset(1) as *mut core::ffi::c_void,
            (DataItemSize.wrapping_mul(DataItemsCount)) as usize,
        );
        (*Irp).IoStatus.Information = Size as _;
        STATUS_SUCCESS
    } else {
        STATUS_BUFFER_TOO_SMALL
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel Streaming
//  * FILE:            drivers/ksfilter/ks/connectivity.c
//  * PURPOSE:         KS Pin functions
//  * PROGRAMMER:      Johannes Anderwald
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// KSPIN_INTERFACE StandardPinInterface =
// {
//     {STATIC_KSINTERFACESETID_Standard},
//     KSINTERFACE_STANDARD_STREAMING,
//     0
// };
//
// KSPIN_MEDIUM StandardPinMedium =
// {
//     {STATIC_KSMEDIUMSETID_Standard},
//     KSMEDIUM_TYPE_ANYINSTANCE,
//     0
// };
//
// const GUID KSDATAFORMAT_SUBTYPE_BDA_MPEG2_TRANSPORT = {0xf4aeb342, 0x0329, 0x4fdd, {0xa8, 0xfd, 0x4a, 0xff, 0x49, 0x26, 0xc9, 0x78}};
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsCreatePin(
//     IN  HANDLE FilterHandle,
//     IN  PKSPIN_CONNECT Connect,
//     IN  ACCESS_MASK DesiredAccess,
//     OUT PHANDLE ConnectionHandle)
// {
//     UINT ConnectSize = sizeof(KSPIN_CONNECT);
//
//     PKSDATAFORMAT_WAVEFORMATEX Format = (PKSDATAFORMAT_WAVEFORMATEX)(Connect + 1);
//     if (Format->DataFormat.FormatSize == sizeof(KSDATAFORMAT) ||
//         Format->DataFormat.FormatSize == sizeof(KSDATAFORMAT) + sizeof(WAVEFORMATEX) + Format->WaveFormatEx.cbSize)
//     {
//         ConnectSize += Format->DataFormat.FormatSize;
//     }
//
//     return KspCreateObjectType(FilterHandle,
//                                KSSTRING_Pin,
//                                (PVOID)Connect,
//                                ConnectSize,
//                                DesiredAccess,
//                                ConnectionHandle);
// }
//
// NTSTATUS
// KspValidateConnectRequest(
//     IN PIRP Irp,
//     IN ULONG DescriptorsCount,
//     IN PVOID Descriptors,
//     IN ULONG DescriptorSize,
//     OUT PKSPIN_CONNECT* Connect)
// {
//     PKSPIN_CONNECT ConnectDetails;
//     PKSPIN_INTERFACE Interface;
//     PKSPIN_MEDIUM Medium;
//     ULONG Size;
//     NTSTATUS Status;
//     ULONG Index;
//     ULONG Count;
//     BOOLEAN Found;
//     PKSPIN_DESCRIPTOR Descriptor;
//     UNICODE_STRING GuidString2;
//
//     /* did the caller miss the connect parameter */
//     if (!Connect)
//         return STATUS_INVALID_PARAMETER;
//
//     /* set create param  size */
//     Size = sizeof(KSPIN_CONNECT);
//
//     /* fetch create parameters */
//     Status = KspCopyCreateRequest(Irp,
//                                   KSSTRING_Pin,
//                                   &Size,
//                                   (PVOID*)&ConnectDetails);
//
//     /* check for success */
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* is pin id out of bounds */
//     if (ConnectDetails->PinId >= DescriptorsCount)
//     {
//         FreeItem(ConnectDetails);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if (DescriptorSize == sizeof(KSPIN_DESCRIPTOR))
//     {
//         /* standard pin descriptor */
//         Descriptor = (PKSPIN_DESCRIPTOR)((ULONG_PTR)Descriptors + sizeof(KSPIN_DESCRIPTOR) * ConnectDetails->PinId);
//     }
//     else
//     {
//         /* extended / variable pin descriptor */
//         Descriptor = &((PKSPIN_DESCRIPTOR_EX)((ULONG_PTR)Descriptors + DescriptorSize * ConnectDetails->PinId))->PinDescriptor;
//     }
//
//
//     /* does the pin have interface details filled in */
//     if (Descriptor->InterfacesCount && Descriptor->Interfaces)
//     {
//         /* use provided pin interface count */
//         Count = Descriptor->InterfacesCount;
//         Interface = (PKSPIN_INTERFACE)Descriptor->Interfaces;
//     }
//     else
//     {
//         /* use standard pin interface */
//         Count = 1;
//         Interface = &StandardPinInterface;
//     }
//
//     /* now check the interface */
//     Found = FALSE;
//     Index = 0;
//     RtlStringFromGUID(&ConnectDetails->Interface.Set, &GuidString2);
//     do
//     {
//         UNICODE_STRING GuidString;
//         RtlStringFromGUID(&Interface[Index].Set, &GuidString);
//
//         DPRINT("Driver Interface %S Id %u\n", GuidString.Buffer, Interface[Index].Id);
//         DPRINT("Connect Interface %S Id %u\n", GuidString2.Buffer, ConnectDetails->Interface.Id);
//
//         RtlFreeUnicodeString(&GuidString);
//
//         if (IsEqualGUIDAligned(&Interface[Index].Set, &ConnectDetails->Interface.Set) &&
//                                Interface[Index].Id == ConnectDetails->Interface.Id)
//         {
//             /* found a matching interface */
//             Found = TRUE;
//             break;
//         }
//         /* iterate to next interface */
//         Index++;
//     }while(Index < Count);
//     RtlFreeUnicodeString(&GuidString2);
//
//     if (!Found)
//     {
//         /* pin doesnt support this interface */
//         FreeItem(ConnectDetails);
//         return STATUS_NO_MATCH;
//     }
//
//     /* does the pin have medium details filled in */
//     if (Descriptor->MediumsCount && Descriptor->Mediums)
//     {
//         /* use provided pin interface count */
//         Count = Descriptor->MediumsCount;
//         Medium = (PKSPIN_MEDIUM)Descriptor->Mediums;
//     }
//     else
//     {
//         /* use standard pin interface */
//         Count = 1;
//         Medium = &StandardPinMedium;
//     }
//
//     /* now check the interface */
//     Found = FALSE;
//     Index = 0;
//     RtlStringFromGUID(&ConnectDetails->Medium.Set, &GuidString2);
//     do
//     {
//         UNICODE_STRING GuidString;
//         RtlStringFromGUID(&Medium[Index].Set, &GuidString);
//
//         DPRINT("Driver Medium %S Id %u\n", GuidString.Buffer, Medium[Index].Id);
//         DPRINT("Connect Medium %S Id %u\n", GuidString2.Buffer, ConnectDetails->Medium.Id);
//
//         RtlFreeUnicodeString(&GuidString);
//
//         if (IsEqualGUIDAligned(&Medium[Index].Set, &ConnectDetails->Medium.Set) &&
//                                Medium[Index].Id == ConnectDetails->Medium.Id)
//         {
//             /* found a matching interface */
//             Found = TRUE;
//             break;
//         }
//
//         /* iterate to next medium */
//         Index++;
//     }while(Index < Count);
//     RtlFreeUnicodeString(&GuidString2);
//
//     if (!Found)
//     {
//         /* pin doesnt support this medium */
//         FreeItem(ConnectDetails);
//         return STATUS_NO_MATCH;
//     }
//
//     /// FIXME
//     /// implement format checking
//
//     *Connect = ConnectDetails;
//     return STATUS_SUCCESS;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsValidateConnectRequest(
//     IN  PIRP Irp,
//     IN  ULONG DescriptorsCount,
//     IN  KSPIN_DESCRIPTOR* Descriptor,
//     OUT PKSPIN_CONNECT* Connect)
// {
//     return KspValidateConnectRequest(Irp, DescriptorsCount, Descriptor, sizeof(KSPIN_DESCRIPTOR), Connect);
// }
//
// NTSTATUS
// KspReadMediaCategory(
//     IN LPGUID Category,
//     PKEY_VALUE_PARTIAL_INFORMATION *OutInformation)
// {
//     UNICODE_STRING MediaPath = RTL_CONSTANT_STRING(L!("\\REGISTRY\\MACHINE\\SYSTEM\\CurrentControlSet\\Control\\MediaCategories\\"));
//     UNICODE_STRING Name = RTL_CONSTANT_STRING(L!("Name"));
//     UNICODE_STRING GuidString, Path;
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE hKey;
//     ULONG Size;
//     PKEY_VALUE_PARTIAL_INFORMATION KeyInfo;
//
//     /* convert the guid to string */
//     Status = RtlStringFromGUID(Category, &GuidString);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* allocate buffer for the registry key */
//     Path.Length = 0;
//     Path.MaximumLength = MediaPath.MaximumLength + GuidString.MaximumLength;
//     Path.Buffer = AllocateItem(NonPagedPool, Path.MaximumLength);
//     if (!Path.Buffer)
//     {
//         /* not enough memory */
//         RtlFreeUnicodeString(&GuidString);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlAppendUnicodeStringToString(&Path, &MediaPath);
//     RtlAppendUnicodeStringToString(&Path, &GuidString);
//
//     /* free guid string */
//     RtlFreeUnicodeString(&GuidString);
//
//     /* initialize object attributes */
//     InitializeObjectAttributes(&ObjectAttributes, &Path, OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, NULL, NULL);
//
//     /* open the key */
//     Status = ZwOpenKey(&hKey, GENERIC_READ, &ObjectAttributes);
//
//     DPRINT("ZwOpenKey() status 0x%08lx %wZ\n", Status, &Path);
//
//     /* free path buffer */
//     FreeItem(Path.Buffer);
//
//     /* check for success */
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwOpenKey() failed with status 0x%08lx\n", Status);
//         return Status;
//     }
//
//     /* query the name size */
//     Status = ZwQueryValueKey(hKey, &Name, KeyValuePartialInformation, NULL, 0, &Size);
//     if (!NT_SUCCESS(Status) && Status != STATUS_BUFFER_TOO_SMALL)
//     {
//         /* failed to query for name key */
//         ZwClose(hKey);
//         return Status;
//     }
//
//     /* allocate buffer to read key info */
//     KeyInfo = (PKEY_VALUE_PARTIAL_INFORMATION) AllocateItem(NonPagedPool, Size);
//     if (!KeyInfo)
//     {
//         /* not enough memory */
//         ZwClose(hKey);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* now read the info */
//     Status = ZwQueryValueKey(hKey, &Name, KeyValuePartialInformation, (PVOID)KeyInfo, Size, &Size);
//
//     /* close the key */
//     ZwClose(hKey);
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed to read key */
//         FreeItem(KeyInfo);
//         return Status;
//     }
//
//     /* store key information */
//     *OutInformation = KeyInfo;
//     return Status;
// }
//
// KSDDKAPI
// NTSTATUS
// NTAPI
// KspPinPropertyHandler(
//     IN  PIRP Irp,
//     IN  PKSPROPERTY Property,
//     IN  OUT PVOID Data,
//     IN  ULONG DescriptorsCount,
//     IN  const KSPIN_DESCRIPTOR* Descriptors,
//     IN  ULONG DescriptorSize)
// {
//     KSP_PIN * Pin;
//     KSMULTIPLE_ITEM * Item;
//     PIO_STACK_LOCATION IoStack;
//     ULONG Size, Index;
//     PVOID Buffer;
//     PKSDATARANGE_AUDIO *WaveFormatOut;
//     PKSDATAFORMAT_WAVEFORMATEX WaveFormatIn;
//     PKEY_VALUE_PARTIAL_INFORMATION KeyInfo;
//     const KSPIN_DESCRIPTOR *Descriptor;
//     NTSTATUS Status = STATUS_NOT_SUPPORTED;
//     ULONG Count;
//     const PKSDATARANGE* DataRanges;
//     LPGUID Guid;
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     Buffer = Data;
//
//     //DPRINT("KsPinPropertyHandler Irp %p Property %p Data %p DescriptorsCount %u Descriptor %p OutputLength %u Id %u\n", Irp, Property, Data, DescriptorsCount, Descriptor, IoStack->Parameters.DeviceIoControl.OutputBufferLength, Property->Id);
//
//     /* convert to PKSP_PIN */
//     Pin = (KSP_PIN*)Property;
//
//     if (Property->Id != KSPROPERTY_PIN_CTYPES)
//     {
//         if (Pin->PinId >= DescriptorsCount)
//         {
//             /* invalid parameter */
//             return STATUS_INVALID_PARAMETER;
//         }
//     }
//     else
//     {
//         (*(PULONG)Buffer) = DescriptorsCount;
//         Irp->IoStatus.Information = sizeof(ULONG);
//         return STATUS_SUCCESS;
//     }
//
//
//     if (DescriptorSize == sizeof(KSPIN_DESCRIPTOR))
//     {
//         /* it is simple pin descriptor */
//         Descriptor = &Descriptors[Pin->PinId];
//     }
//     else
//     {
//         /* get offset to pin descriptor */
//         Descriptor = &(((PKSPIN_DESCRIPTOR_EX)((ULONG_PTR)Descriptors + Pin->PinId * DescriptorSize))->PinDescriptor);
//     }
//
//     switch(Property->Id)
//     {
//         case KSPROPERTY_PIN_DATAFLOW:
//
//             Size = sizeof(KSPIN_DATAFLOW);
//             if (IoStack->Parameters.DeviceIoControl.OutputBufferLength < Size)
//             {
//                 Irp->IoStatus.Information = Size;
//                 Status = STATUS_BUFFER_TOO_SMALL;
//                 break;
//             }
//
//             *((KSPIN_DATAFLOW*)Buffer) = Descriptor->DataFlow;
//             Irp->IoStatus.Information = sizeof(KSPIN_DATAFLOW);
//             Status = STATUS_SUCCESS;
//             break;
//
//         case KSPROPERTY_PIN_DATARANGES:
//         case KSPROPERTY_PIN_CONSTRAINEDDATARANGES:
//
//             Size = sizeof(KSMULTIPLE_ITEM);
//             DPRINT("Id %lu PinId %lu DataRangesCount %lu ConstrainedDataRangesCount %lu\n", Property->Id, Pin->PinId, Descriptor->DataRangesCount, Descriptor->ConstrainedDataRangesCount);
//
//             if (Property->Id == KSPROPERTY_PIN_DATARANGES || Descriptor->ConstrainedDataRangesCount == 0)
//             {
//                 DataRanges = Descriptor->DataRanges;
//                 Count = Descriptor->DataRangesCount;
//             }
//             else
//             {
//                 DataRanges = Descriptor->ConstrainedDataRanges;
//                 Count = Descriptor->ConstrainedDataRangesCount;
//             }
//
//             for (Index = 0; Index < Count; Index++)
//             {
//                 Size += ((DataRanges[Index]->FormatSize + 0x7) & ~0x7);
//             }
//
//             if (IoStack->Parameters.DeviceIoControl.OutputBufferLength == 0)
//             {
//                 /* buffer too small */
//                 Irp->IoStatus.Information = Size;
//                 Status = STATUS_BUFFER_OVERFLOW;
//                 break;
//             }
//
//             Item = (KSMULTIPLE_ITEM*)Buffer;
//
//             if (IoStack->Parameters.DeviceIoControl.OutputBufferLength == sizeof(ULONG))
//             {
//                 /* store the result size */
//                 Item->Size = Size;
//                 Irp->IoStatus.Information = sizeof(ULONG);
//                 Status = STATUS_SUCCESS;
//                 break;
//             }
//
//             /* store descriptor size */
//             Item->Size = Size;
//             Item->Count = Count;
//
//             if (IoStack->Parameters.DeviceIoControl.OutputBufferLength == sizeof(KSMULTIPLE_ITEM))
//             {
//                 Irp->IoStatus.Information = sizeof(KSMULTIPLE_ITEM);
//                 Status = STATUS_SUCCESS;
//                 break;
//             }
//
//             /* now copy all dataranges */
//             Data = (PUCHAR)(Item +1);
//
//             /* alignment assert */
//             ASSERT(((ULONG_PTR)Data & 0x7) == 0);
//
//             for (Index = 0; Index < Count; Index++)
//             {
//                 UNICODE_STRING GuidString;
//                 /* convert the guid to string */
//                 RtlStringFromGUID(&DataRanges[Index]->MajorFormat, &GuidString);
//                 DPRINT("Index %lu MajorFormat %S\n", Index, GuidString.Buffer);
//                 RtlStringFromGUID(&DataRanges[Index]->SubFormat, &GuidString);
//                 DPRINT("Index %lu SubFormat %S\n", Index, GuidString.Buffer);
//                 RtlStringFromGUID(&DataRanges[Index]->Specifier, &GuidString);
//                 DPRINT("Index %lu Specifier %S\n", Index, GuidString.Buffer);
//                 RtlStringFromGUID(&DataRanges[Index]->Specifier, &GuidString);
//                 DPRINT("Index %lu FormatSize %lu Flags %lu SampleSize %lu Reserved %lu KSDATAFORMAT %lu\n", Index,
//                        DataRanges[Index]->FormatSize, DataRanges[Index]->Flags, DataRanges[Index]->SampleSize, DataRanges[Index]->Reserved, sizeof(KSDATAFORMAT));
//
//                 RtlMoveMemory(Data, DataRanges[Index], DataRanges[Index]->FormatSize);
//                 Data = ((PUCHAR)Data + DataRanges[Index]->FormatSize);
//                 /* alignment assert */
//                 ASSERT(((ULONG_PTR)Data & 0x7) == 0);
//                 Data = (PVOID)(((ULONG_PTR)Data + 0x7) & ~0x7);
//             }
//
//             Status = STATUS_SUCCESS;
//             Irp->IoStatus.Information = Size;
//             break;
//         case KSPROPERTY_PIN_INTERFACES:
//
//             if (Descriptor->Interfaces)
//             {
//                 /* use mediums provided by driver */
//                 return KsHandleSizedListQuery(Irp, Descriptor->InterfacesCount, sizeof(KSPIN_MEDIUM), Descriptor->Interfaces);
//             }
//             else
//             {
//                 /* use standard medium */
//                 return KsHandleSizedListQuery(Irp, 1, sizeof(KSPIN_INTERFACE), &StandardPinInterface);
//             }
//             break;
//
//         case KSPROPERTY_PIN_MEDIUMS:
//
//             if (Descriptor->MediumsCount)
//             {
//                 /* use mediums provided by driver */
//                 return KsHandleSizedListQuery(Irp, Descriptor->MediumsCount, sizeof(KSPIN_MEDIUM), Descriptor->Mediums);
//             }
//             else
//             {
//                 /* use standard medium */
//                 return KsHandleSizedListQuery(Irp, 1, sizeof(KSPIN_MEDIUM), &StandardPinMedium);
//             }
//             break;
//
//         case KSPROPERTY_PIN_COMMUNICATION:
//
//             Size = sizeof(KSPIN_COMMUNICATION);
//             if (IoStack->Parameters.DeviceIoControl.OutputBufferLength < Size)
//             {
//                 Irp->IoStatus.Information = Size;
//                 Status = STATUS_BUFFER_TOO_SMALL;
//                 break;
//             }
//
//             *((KSPIN_COMMUNICATION*)Buffer) = Descriptor->Communication;
//
//             Status = STATUS_SUCCESS;
//             Irp->IoStatus.Information = Size;
//             break;
//
//         case KSPROPERTY_PIN_CATEGORY:
//
//             if (!Descriptor->Category)
//             {
//                 /* no pin category */
//                 return STATUS_NOT_FOUND;
//             }
//
//             /* check size */
//             Size = sizeof(GUID);
//             if (IoStack->Parameters.DeviceIoControl.OutputBufferLength < Size)
//             {
//                 /* buffer too small */
//                 Irp->IoStatus.Information = Size;
//                 Status = STATUS_BUFFER_TOO_SMALL;
//                 break;
//             }
//
//             /* copy category guid */
//             RtlMoveMemory(Buffer, Descriptor->Category, sizeof(GUID));
//
//             /* save result */
//             Status = STATUS_SUCCESS;
//             Irp->IoStatus.Information = Size;
//             break;
//
//         case KSPROPERTY_PIN_NAME:
//
//             if (Descriptor->Name)
//             {
//                 /* use pin name */
//                 Guid = (LPGUID)Descriptor->Name;
//             }
//             else
//             {
//                 /* use pin category as fallback */
//                 Guid = (LPGUID)Descriptor->Category;
//             }
//
//             if (!Guid)
//             {
//                 /* no friendly name available */
//                 return STATUS_NOT_FOUND;
//             }
//
//             /* read friendly name category name */
//             Status = KspReadMediaCategory(Guid, &KeyInfo);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* failed to read category */
//                 Irp->IoStatus.Information = 0;
//                 break;
//             }
//
//             /* store required length */
//             Irp->IoStatus.Information = KeyInfo->DataLength + sizeof(WCHAR);
//
//             /* check if buffer is too small */
//             if (KeyInfo->DataLength + sizeof(WCHAR) > IoStack->Parameters.DeviceIoControl.OutputBufferLength)
//             {
//                 /* buffer too small */
//                 Status = STATUS_BUFFER_OVERFLOW;
//                 FreeItem(KeyInfo);
//                 break;
//             }
//
//             /* copy result */
//             RtlMoveMemory(Irp->AssociatedIrp.SystemBuffer, &KeyInfo->Data, KeyInfo->DataLength);
//
//             /* null terminate name */
//             ((LPWSTR)Irp->AssociatedIrp.SystemBuffer)[KeyInfo->DataLength / sizeof(WCHAR)] = L'\0';
//
//             /* free key info */
//             FreeItem(KeyInfo);
//             break;
//         case KSPROPERTY_PIN_PROPOSEDATAFORMAT:
//             Size = sizeof(KSDATAFORMAT);
//             if (IoStack->Parameters.DeviceIoControl.OutputBufferLength < Size)
//             {
//                 Irp->IoStatus.Information = Size;
//                 Status = STATUS_BUFFER_TOO_SMALL;
//                 break;
//             }
//             if (IoStack->Parameters.DeviceIoControl.OutputBufferLength != sizeof(KSDATAFORMAT_WAVEFORMATEX))
//             {
//                 UNIMPLEMENTED;
//                 Status = STATUS_NOT_IMPLEMENTED;
//                 Irp->IoStatus.Information = 0;
//                 break;
//             }
//
//             WaveFormatIn = (PKSDATAFORMAT_WAVEFORMATEX)Buffer;
//             if (!Descriptor->DataRanges || !Descriptor->DataRangesCount)
//             {
//                 Status = STATUS_UNSUCCESSFUL;
//                 Irp->IoStatus.Information = 0;
//                 break;
//             }
//             WaveFormatOut = (PKSDATARANGE_AUDIO*)Descriptor->DataRanges;
//             for(Index = 0; Index < Descriptor->DataRangesCount; Index++)
//             {
//                 if (WaveFormatOut[Index]->DataRange.FormatSize != sizeof(KSDATARANGE_AUDIO))
//                 {
//                     UNIMPLEMENTED;
//                     continue;
//                 }
//
//                 if (WaveFormatOut[Index]->MinimumSampleFrequency > WaveFormatIn->WaveFormatEx.nSamplesPerSec ||
//                     WaveFormatOut[Index]->MaximumSampleFrequency < WaveFormatIn->WaveFormatEx.nSamplesPerSec ||
//                     WaveFormatOut[Index]->MinimumBitsPerSample > WaveFormatIn->WaveFormatEx.wBitsPerSample ||
//                     WaveFormatOut[Index]->MaximumBitsPerSample < WaveFormatIn->WaveFormatEx.wBitsPerSample ||
//                     WaveFormatOut[Index]->MaximumChannels < WaveFormatIn->WaveFormatEx.nChannels)
//                 {
//                     Irp->IoStatus.Status = STATUS_NO_MATCH;
//                     Irp->IoStatus.Information = 0;
//                     return STATUS_NO_MATCH;
//                 }
//                 else
//                 {
//                     Irp->IoStatus.Status = STATUS_SUCCESS;
//                     Irp->IoStatus.Information = 0;
//                     return STATUS_SUCCESS;
//                 }
//             }
//             Status = STATUS_NO_MATCH;
//             Irp->IoStatus.Information = 0;
//             break;
//         default:
//             DPRINT1("Unhandled property request %x\n", Property->Id);
//             Status = STATUS_NOT_IMPLEMENTED;
//             Irp->IoStatus.Information = 0;
//     }
//
//     return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsPinPropertyHandler(
//     IN  PIRP Irp,
//     IN  PKSPROPERTY Property,
//     IN  OUT PVOID Data,
//     IN  ULONG DescriptorsCount,
//     IN  const KSPIN_DESCRIPTOR* Descriptor)
// {
//     return KspPinPropertyHandler(Irp, Property, Data, DescriptorsCount, Descriptor, sizeof(KSPIN_DESCRIPTOR));
// }
//
// /*
//     @unimplemented
// */
// KSDDKAPI NTSTATUS NTAPI
// KsPinDataIntersectionEx(
//     IN  PIRP Irp,
//     IN  PKSP_PIN Pin,
//     OUT PVOID Data,
//     IN  ULONG DescriptorsCount,
//     IN  const KSPIN_DESCRIPTOR* Descriptor,
//     IN  ULONG DescriptorSize,
//     IN  PFNKSINTERSECTHANDLEREX IntersectHandler OPTIONAL,
//     IN  PVOID HandlerContext OPTIONAL)
// {
//     UNIMPLEMENTED;
//     return STATUS_UNSUCCESSFUL;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsPinDataIntersection(
//     IN  PIRP Irp,
//     IN  PKSP_PIN Pin,
//     OUT PVOID Data,
//     IN  ULONG DescriptorsCount,
//     IN  const KSPIN_DESCRIPTOR* Descriptor,
//     IN  PFNKSINTERSECTHANDLER IntersectHandler)
// {
//     KSMULTIPLE_ITEM * Item;
//     KSDATARANGE * DataRange;
//     PIO_STACK_LOCATION IoStack;
//     ULONG Size;
//     ULONG Index;
//     NTSTATUS Status;
//
//     /* get current irp stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* calculate minimum data size */
//     Size = sizeof(KSP_PIN) + sizeof(KSMULTIPLE_ITEM) + sizeof(KSDATARANGE);
//     if (IoStack->Parameters.DeviceIoControl.InputBufferLength < Size)
//     {
//         /* buffer too small */
//         Irp->IoStatus.Information = Size;
//         Irp->IoStatus.Status = STATUS_BUFFER_TOO_SMALL;
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//     /* is pin id out of bounds */
//     if (Pin->PinId >= DescriptorsCount)
//     {
//         /* it is */
//         Irp->IoStatus.Status = STATUS_INVALID_PARAMETER;
//         Irp->IoStatus.Information = 0;
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* get start item */
//     Item = (KSMULTIPLE_ITEM*)(Pin + 1);
//     /* get first data range */
//     DataRange = (KSDATARANGE*)(Item + 1);
//     /* iterate through all data ranges */
//     for(Index = 0; Index < Item->Count; Index++, DataRange++)
//     {
//         /* call intersect handler */
//         Status = IntersectHandler(Irp, Pin, DataRange, Data);
//         if (NT_SUCCESS(Status))
//         {
//             if (IoStack->Parameters.DeviceIoControl.OutputBufferLength < DataRange->FormatSize)
//             {
//                 /* buffer is too small */
//                 Irp->IoStatus.Information = DataRange->FormatSize;
//                 Irp->IoStatus.Status = STATUS_BUFFER_TOO_SMALL;
//                 return STATUS_BUFFER_TOO_SMALL;
//             }
//             RtlMoveMemory(Irp->UserBuffer, DataRange, sizeof(KSDATARANGE));
//             Irp->IoStatus.Information = sizeof(KSDATARANGE);
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             return STATUS_SUCCESS;
//         }
//
//     }
//
//     Irp->IoStatus.Information = 0;
//     Irp->IoStatus.Status = STATUS_NO_MATCH;
//     return STATUS_NO_MATCH;
// }
//
// /*
//     @implemented
// */
//
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsHandleSizedListQuery(
//     IN  PIRP Irp,
//     IN  ULONG DataItemsCount,
//     IN  ULONG DataItemSize,
//     IN  const VOID* DataItems)
// {
//     ULONG Size;
//     PIO_STACK_LOCATION IoStack;
//     PKSMULTIPLE_ITEM Item;
//
//     /* get current irp stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* calculate size */
//     Size = DataItemSize * DataItemsCount + sizeof(KSMULTIPLE_ITEM);
//
//     /* get multiple item */
//     Item = (PKSMULTIPLE_ITEM)Irp->AssociatedIrp.SystemBuffer;
//
//     if (IoStack->Parameters.DeviceIoControl.OutputBufferLength == 0)
//     {
//         /* buffer too small */
//         Irp->IoStatus.Information = Size;
//
//         return STATUS_BUFFER_OVERFLOW;
//     }
//
//     if (IoStack->Parameters.DeviceIoControl.OutputBufferLength == sizeof(ULONG))
//     {
//         /* store just the size */
//         Item->Size = Size;
//         Irp->IoStatus.Information = sizeof(ULONG);
//
//         return STATUS_SUCCESS;
//     }
//
//
//     if (IoStack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(KSMULTIPLE_ITEM))
//     {
//         /* buffer too small */
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     Item->Count = DataItemsCount;
//     Item->Size = DataItemSize;
//
//     if (IoStack->Parameters.DeviceIoControl.OutputBufferLength == sizeof(KSMULTIPLE_ITEM))
//     {
//         /* buffer can only hold the length descriptor */
//         Irp->IoStatus.Information = sizeof(KSMULTIPLE_ITEM);
//         return STATUS_SUCCESS;
//     }
//
//     if (IoStack->Parameters.DeviceIoControl.OutputBufferLength >= Size)
//     {
//         /* copy items */
//         RtlMoveMemory((PVOID)(Item + 1), DataItems, DataItemSize * DataItemsCount);
//         /* store result */
//         Irp->IoStatus.Information = Size;
//         /* done */
//         return STATUS_SUCCESS;
//     }
//     else
//     {
//         /* buffer too small */
//         return STATUS_BUFFER_TOO_SMALL;
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
