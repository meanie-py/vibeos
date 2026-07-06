use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{ExFreePool, GUID, LANGID, LPGUID, LPWSTR, _OBJECT_ATTRIBUTES, PDEVICE_OBJECT, PHANDLE, PIRP, PSHORT, PUNICODE_STRING, RtlFreeUnicodeString, RtlInitUnicodeString, RtlStringFromGUID, ZwClose, ZwCreateKey, ZwOpenKey, ZwSetValueKey};
use core::ffi::c_void;
use crate::types::*;
use crate::glue::*;
use crate::cb;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// ---- local helpers (C macros / CRT fns not in scope) ----
#[allow(non_snake_case, unused_variables)]
#[inline(always)]
unsafe fn RtlAssert(_a: PVOID, _f: PVOID, _l: u32, _m: PVOID) {}

#[allow(non_snake_case)]
#[inline(always)]
unsafe fn wcslen(s: *const u16) -> usize {
    let mut n = 0usize;
    while *s.add(n) != 0 { n += 1; }
    n
}

#[allow(non_snake_case)] #[inline(always)] fn HIBYTE(w: u16) -> u8 { (w >> 8) as u8 }
#[allow(non_snake_case)] #[inline(always)] fn LOBYTE(w: u16) -> u8 { w as u8 }

#[allow(non_snake_case)]
unsafe fn INIT_USBAUDIO_MID(guid: *mut GUID, id: u16) {
    (*guid).Data1 = 0x4e1cecd2u32.wrapping_add(id as u32);
    (*guid).Data2 = 0x1679;
    (*guid).Data3 = 0x463b;
    (*guid).Data4 = [0xa7, 0x2f, 0xa5, 0xbf, 0x64, 0xc8, 0x6e, 0xba];
}
#[allow(non_snake_case)]
unsafe fn INIT_USBAUDIO_PID(guid: *mut GUID, id: u16) {
    (*guid).Data1 = 0xabcc5a5eu32.wrapping_add(id as u32);
    (*guid).Data2 = 0xc263;
    (*guid).Data3 = 0x463b;
    (*guid).Data4 = [0xa7, 0x2f, 0xa5, 0xbf, 0x64, 0xc8, 0x6e, 0xba];
}
#[allow(non_snake_case)]
unsafe fn INIT_USBAUDIO_PRODUCT_NAME(guid: *mut GUID, vid: u16, pid: u16, str_index: u16) {
    (*guid).Data1 = 0xfc575048u32.wrapping_add(vid as u32);
    (*guid).Data2 = 0x2e08u16.wrapping_add(pid);
    (*guid).Data3 = 0x463bu16.wrapping_add(str_index);
    (*guid).Data4 = [0xa7, 0x2f, 0xa5, 0xbf, 0x64, 0xc8, 0x6e, 0xba];
}

// ---- extern declarations for #[no_mangle] functions in pin.rs ----
// These avoid pulling in crate::pin's full type universe here.
extern "system" {
    fn USBAudioPinCreate(Pin: PKSPIN, Irp: PIRP) -> NTSTATUS;
    fn USBAudioPinClose(Pin: PKSPIN, Irp: PIRP) -> NTSTATUS;
    fn USBAudioPinProcess(Pin: PKSPIN) -> NTSTATUS;
    fn USBAudioPinReset(Pin: PKSPIN);
    fn USBAudioPinSetDataFormat(
        Pin: PKSPIN,
        OldFormat: PKSDATAFORMAT,
        OldAttributeList: PKSMULTIPLE_ITEM,
        DataRange: *const KSDATARANGE,
        AttributeRange: *const KSATTRIBUTE_LIST,
    ) -> NTSTATUS;
    fn USBAudioPinSetDeviceState(Pin: PKSPIN, ToState: KSSTATE, FromState: KSSTATE) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut NodeTypeMicrophone: GUID = GUID {
    Data1: 0,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
};

pub static mut NodeTypeDesktopMicrophone: GUID = GUID {
    Data1: 0,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
};

pub static mut NodeTypePersonalMicrophone: GUID = GUID {
    Data1: 0,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
};

pub static mut NodeTypeOmmniMicrophone: GUID = GUID {
    Data1: 0,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
};

pub static mut NodeTypeArrayMicrophone: GUID = GUID {
    Data1: 0,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
};

pub static mut NodeTypeProcessingArrayMicrophone: GUID = GUID {
    Data1: 0,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
};

pub static mut NodeTypeSpeaker: GUID = GUID {
    Data1: 0,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
};

pub static mut NodeTypeHeadphonesSpeaker: GUID = GUID {
    Data1: 0,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
};

pub static mut NodeTypeHMDA: GUID = GUID {
    Data1: 0,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
};

pub static mut NodeTypeDesktopSpeaker: GUID = GUID {
    Data1: 0,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
};

pub static mut NodeTypeRoomSpeaker: GUID = GUID {
    Data1: 0,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
};

pub static mut NodeTypeCommunicationSpeaker: GUID = GUID {
    Data1: 0,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
};

pub static mut NodeTypeSubwoofer: GUID = GUID {
    Data1: 0,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
};

pub static mut NodeTypeCapture: GUID = GUID {
    Data1: 0xfb6c4281 as ::core::ffi::c_uint,
    Data2: 0x353 as ::core::ffi::c_ushort,
    Data3: 0x11d1 as ::core::ffi::c_ushort,
    Data4: [
        0x90 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x16 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut NodeTypePlayback: GUID = GUID {
    Data1: 0x6994ad04 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x93ef as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xa3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_KSCATEGORY_AUDIO: GUID = GUID {
    Data1: 0x6994ad04 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x93ef as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xa3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut StandardPinInterface: KSPIN_INTERFACE = KSIDENTIFIER {
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

pub static mut StandardPinMedium: KSPIN_MEDIUM = KSIDENTIFIER {
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

// BridgePinAudioFormat: KSDATAFORMAT__bindgen_ty_1 packed as little-endian u64 array.
// Fields: FormatSize=64, Flags=0, SampleSize=0, Reserved=0,
//   MajorFormat={0x73647561,0x0000,0x0010,[0x80,0,0,0xaa,0,0x38,0x9b,0x71]},
//   SubFormat={0x6dba3190,0x67bd,0x11cf,[0xa0,0xf7,0,0x20,0xaf,0xd1,0x56,0xe4]},
//   Specifier={0x0f6417d6,0xc318,0x11d0,[0xa4,0x3f,0,0xa0,0xc9,0x22,0x31,0x96]}
pub static mut BridgePinAudioFormat: [KSDATARANGE; 1] = [KSDATAFORMAT {
    __bindgen_anon_1: __BindgenUnionField::new(),
    Alignment: __BindgenUnionField::new(),
    bindgen_union_field: [
        0x0000_0000_0000_0040u64, // FormatSize=64, Flags=0
        0x0000_0000_0000_0000u64, // SampleSize=0, Reserved=0
        0x0010_0000_7364_7561u64, // MajorFormat: D1+D2+D3
        0x719b_3800_aa00_0080u64, // MajorFormat: D4
        0x11cf_67bd_6dba_3190u64, // SubFormat: D1+D2+D3
        0xe456_d1af_2000_f7a0u64, // SubFormat: D4
        0x11d0_c318_0f64_17d6u64, // Specifier: D1+D2+D3
        0x9631_22c9_a000_3fa4u64, // Specifier: D4
    ],
}];

static mut BridgePinAudioFormats: [PKSDATARANGE; 1] =
    [::core::ptr::null::<KSDATAFORMAT>() as *mut KSDATAFORMAT; 1];

static REFERENCE_STRING_DATA: [u16; 7] = [0x67, 0x6c, 0x6f, 0x62, 0x61, 0x6c, 0x00]; // L"global\0"
static mut ReferenceString: LPWSTR = REFERENCE_STRING_DATA.as_ptr() as *mut u16;

static mut USBAudioFilterDispatch: KSFILTER_DISPATCH = unsafe {
    _KSFILTER_DISPATCH {
        Create: Some(core::mem::transmute(
            USBAudioFilterCreate as unsafe extern "system" fn(PKSFILTER, PIRP) -> NTSTATUS,
        )),
        Close: None,
        Process: None,
        Reset: None,
    }
};

static mut UsbAudioPinDispatch: KSPIN_DISPATCH = unsafe {
    KSPIN_DISPATCH {
        Create: Some(core::mem::transmute(
            USBAudioPinCreate as unsafe extern "system" fn(PKSPIN, PIRP) -> NTSTATUS,
        )),
        Close: Some(core::mem::transmute(
            USBAudioPinClose as unsafe extern "system" fn(PKSPIN, PIRP) -> NTSTATUS,
        )),
        Process: Some(core::mem::transmute(
            USBAudioPinProcess as unsafe extern "system" fn(PKSPIN) -> NTSTATUS,
        )),
        Reset: Some(core::mem::transmute(
            USBAudioPinReset as unsafe extern "system" fn(PKSPIN),
        )),
        SetDataFormat: Some(core::mem::transmute(
            USBAudioPinSetDataFormat as unsafe extern "system" fn(PKSPIN, PKSDATAFORMAT, PKSMULTIPLE_ITEM, *const KSDATARANGE, *const KSATTRIBUTE_LIST) -> NTSTATUS,
        )),
        SetDeviceState: Some(core::mem::transmute(
            USBAudioPinSetDeviceState as unsafe extern "system" fn(PKSPIN, KSSTATE, KSSTATE) -> NTSTATUS,
        )),
        Connect: None,
        Disconnect: None,
        Clock: ::core::ptr::null::<KSCLOCK_DISPATCH>(),
        Allocator: ::core::ptr::null::<KSALLOCATOR_DISPATCH>(),
    }
};

// FilterAudioVolumeHandler fn ptr filled via bindgen_union_field at init time;
// cannot embed fn ptr in u32 in const eval. Driver uses this table after init.
pub static mut FilterAudioVolumePropertySet: [KSPROPERTY_ITEM; 1] = [KSPROPERTY_ITEM {
    PropertyId: KSPROPERTY_AUDIO_VOLUMELEVEL as ULONG,
    __bindgen_anon_1: KSPROPERTY_ITEM__bindgen_ty_1 {
        GetPropertyHandler: __BindgenUnionField::new(),
        GetSupported: __BindgenUnionField::new(),
        bindgen_union_field: 0, // filled at runtime: FilterAudioVolumeHandler
    },
    MinProperty: ::core::mem::size_of::<KSNODEPROPERTY_AUDIO_CHANNEL>() as ULONG,
    MinData: ::core::mem::size_of::<LONG>() as ULONG,
    __bindgen_anon_2: KSPROPERTY_ITEM__bindgen_ty_2 {
        SetPropertyHandler: __BindgenUnionField::new(),
        SetSupported: __BindgenUnionField::new(),
        bindgen_union_field: 0, // filled at runtime: FilterAudioVolumeHandler
    },
    Values: ::core::ptr::null::<KSPROPERTY_VALUES>(),
    RelationsCount: 0,
    Relations: ::core::ptr::null::<KSPROPERTY>(),
    SupportHandler: None,
    SerializedSize: 0,
}];

pub static mut FilterAudioMutePropertySet: [KSPROPERTY_ITEM; 1] = [KSPROPERTY_ITEM {
    PropertyId: KSPROPERTY_AUDIO_MUTE as ULONG,
    __bindgen_anon_1: KSPROPERTY_ITEM__bindgen_ty_1 {
        GetPropertyHandler: __BindgenUnionField::new(),
        GetSupported: __BindgenUnionField::new(),
        bindgen_union_field: 0, // filled at runtime: FilterAudioMuteHandler
    },
    MinProperty: ::core::mem::size_of::<KSNODEPROPERTY_AUDIO_CHANNEL>() as ULONG,
    MinData: ::core::mem::size_of::<BOOL>() as ULONG,
    __bindgen_anon_2: KSPROPERTY_ITEM__bindgen_ty_2 {
        SetPropertyHandler: __BindgenUnionField::new(),
        SetSupported: __BindgenUnionField::new(),
        bindgen_union_field: 0, // filled at runtime: FilterAudioMuteHandler
    },
    Values: ::core::ptr::null::<KSPROPERTY_VALUES>(),
    RelationsCount: 0,
    Relations: ::core::ptr::null::<KSPROPERTY>(),
    SupportHandler: None,
    SerializedSize: 0,
}];

static mut FilterAudioVolumePropertySetArray: [KSPROPERTY_SET; 1] = [KSPROPERTY_SET {
    Set: ::core::ptr::null::<GUID>(),
    PropertiesCount: 0,
    PropertyItem: ::core::ptr::null::<KSPROPERTY_ITEM>(),
    FastIoCount: 0,
    FastIoTable: ::core::ptr::null::<KSFASTPROPERTY_ITEM>(),
}; 1];

static mut FilterAudioMutePropertySetArray: [KSPROPERTY_SET; 1] = [KSPROPERTY_SET {
    Set: ::core::ptr::null::<GUID>(),
    PropertiesCount: 0,
    PropertyItem: ::core::ptr::null::<KSPROPERTY_ITEM>(),
    FastIoCount: 0,
    FastIoTable: ::core::ptr::null::<KSFASTPROPERTY_ITEM>(),
}; 1];

#[no_mangle]
pub unsafe extern "C" fn UsbAudioGetSetProperty(
    DeviceObject: PDEVICE_OBJECT,
    Request: UCHAR,
    Value: USHORT,
    Index: USHORT,
    TransferBuffer: PVOID,
    TransferBufferLength: ULONG,
    TransferFlags: ULONG,
) -> NTSTATUS {
    let Urb = AllocFunction(core::mem::size_of::<_URB_CONTROL_VENDOR_OR_CLASS_REQUEST>() as ULONG) as PURB;
    if Urb.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    {
        let hdr = (*Urb).__bindgen_anon_1.UrbHeader.as_mut();
        (*hdr).Function = 0x1b as USHORT;
        (*hdr).Length = core::mem::size_of::<_URB_CONTROL_VENDOR_OR_CLASS_REQUEST>() as USHORT;
    }
    {
        let req = (*Urb).__bindgen_anon_1.UrbControlVendorClassRequest.as_mut();
        (*req).TransferBufferLength = TransferBufferLength;
        (*req).TransferBufferMDL = core::ptr::null_mut();
        (*req).TransferBuffer = TransferBuffer;
        (*req).RequestTypeReservedBits = 0u8;
        (*req).Request = Request;
        (*req).Value = Value;
        (*req).Index = Index;
        (*req).TransferFlags = TransferFlags;
        (*req).UrbLink = core::ptr::null_mut();
    }

    let Status = SubmitUrbSync(DeviceObject, Urb);
    FreeFunction(Urb as PVOID);
    Status
}
#[no_mangle]
pub unsafe extern "C" fn FindNodeContextWithNode(
    NodeContext: PNODE_CONTEXT,
    NodeContextCount: ULONG,
    NodeId: ULONG,
) -> PNODE_CONTEXT {
    let mut Index: ULONG = 0;
    let mut NodeIndex: ULONG;
    while Index < NodeContextCount {
        NodeIndex = 0;
        while NodeIndex < (*NodeContext.offset(Index as isize)).NodeCount {
            if (*NodeContext.offset(Index as isize)).Nodes[NodeIndex as usize] == NodeId {
                return NodeContext.offset(Index as isize);
            }
            NodeIndex = NodeIndex.wrapping_add(1);
        }
        Index = Index.wrapping_add(1);
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn FilterAudioMuteHandler(
    Irp: PIRP,
    Request: PKSIDENTIFIER,
    Data: PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_INVALID_PARAMETER;

    let Filter = KsGetFilterFromIrp(Irp);
    if !Filter.is_null() {
        let Property = Request as PKSNODEPROPERTY_AUDIO_CHANNEL;
        let FilterContext = (*Filter).Context as PFILTER_CONTEXT;
        let NodeContext = FindNodeContextWithNode(
            (*(*FilterContext).DeviceExtension).NodeContext,
            (*(*FilterContext).DeviceExtension).NodeContextCount,
            (*Property).NodeProperty.NodeId,
        );
        if !NodeContext.is_null() {
            let FeatureUnitDescriptor = (*NodeContext).Descriptor as PUSB_AUDIO_CONTROL_FEATURE_UNIT_DESCRIPTOR;
            if (*Property).NodeProperty.Property.Flags & KSPROPERTY_TYPE_GET != 0 {
                Status = UsbAudioGetSetProperty(
                    (*(*FilterContext).DeviceExtension).LowerDevice,
                    0x81u8,
                    (1i32 << 8) as USHORT,
                    (((*FeatureUnitDescriptor).bUnitID as i32) << 8) as USHORT,
                    Data,
                    1u32,
                    USBD_TRANSFER_DIRECTION_IN as ULONG,
                );
                (*Irp).IoStatus.Information = core::mem::size_of::<BOOL>() as _;
            } else {
                Status = UsbAudioGetSetProperty(
                    (*(*FilterContext).DeviceExtension).LowerDevice,
                    0x01u8,
                    (1i32 << 8) as USHORT,
                    (((*FeatureUnitDescriptor).bUnitID as i32) << 8) as USHORT,
                    Data,
                    1u32,
                    USBD_TRANSFER_DIRECTION_OUT as ULONG,
                );
            }
        }
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn FilterAudioVolumeHandler(
    Irp: PIRP,
    Request: PKSIDENTIFIER,
    Data: PVOID,
) -> NTSTATUS {
    let Property: PKSNODEPROPERTY_AUDIO_CHANNEL;
    let Filter: PKSFILTER;
    let FilterContext: PFILTER_CONTEXT;
    let NodeContext: PNODE_CONTEXT;
    let FeatureUnitDescriptor: PUSB_AUDIO_CONTROL_FEATURE_UNIT_DESCRIPTOR;
    let TransferBuffer: PSHORT;
    let mut Value: LONG;
    let mut Status: NTSTATUS = STATUS_INVALID_PARAMETER;

    Filter = KsGetFilterFromIrp(Irp);

    if !Filter.is_null() {
        Property = Request as PKSNODEPROPERTY_AUDIO_CHANNEL;
        FilterContext = (*Filter).Context as PFILTER_CONTEXT;

        TransferBuffer = AllocFunction((core::mem::size_of::<USHORT>() as ULONG).wrapping_mul(3)) as PSHORT;
        if TransferBuffer.is_null() {
            RtlAssert(
                b"TransferBuffer\0".as_ptr() as PVOID,
                b"drivers/usb/usbaudio/filter.c\0".as_ptr() as PVOID,
                258u32,
                core::ptr::null_mut(),
            );
        }

        Value = *(Data as PLONG);

        NodeContext = FindNodeContextWithNode(
            (*(*FilterContext).DeviceExtension).NodeContext,
            (*(*FilterContext).DeviceExtension).NodeContextCount,
            (*Property).NodeProperty.NodeId,
        );

        if !NodeContext.is_null() {
            FeatureUnitDescriptor = (*NodeContext).Descriptor as PUSB_AUDIO_CONTROL_FEATURE_UNIT_DESCRIPTOR;

            if (*Property).NodeProperty.Property.Flags as ULONG & KSPROPERTY_TYPE_GET as ULONG != 0 {
                Status = UsbAudioGetSetProperty(
                    (*(*FilterContext).DeviceExtension).LowerDevice,
                    0x81u8,
                    (0x2i32 << 8) as USHORT,
                    (((*FeatureUnitDescriptor).bUnitID as i32) << 8) as USHORT,
                    TransferBuffer.offset(0isize) as PVOID,
                    core::mem::size_of::<USHORT>() as ULONG,
                    USBD_TRANSFER_DIRECTION_IN as ULONG,
                );

                Value = (*TransferBuffer.offset(0isize) as LONG) * 256;
                *(Data as PLONG) = Value;
                (*Irp).IoStatus.Information = core::mem::size_of::<BOOL>() as _;
            } else {
                Value /= 256;

                UsbAudioGetSetProperty(
                    (*(*FilterContext).DeviceExtension).LowerDevice,
                    0x82u8,
                    (0x2i32 << 8) as USHORT,
                    (((*FeatureUnitDescriptor).bUnitID as i32) << 8) as USHORT,
                    TransferBuffer.offset(0isize) as PVOID,
                    core::mem::size_of::<USHORT>() as ULONG,
                    USBD_TRANSFER_DIRECTION_IN as ULONG,
                );

                UsbAudioGetSetProperty(
                    (*(*FilterContext).DeviceExtension).LowerDevice,
                    0x83u8,
                    (0x2i32 << 8) as USHORT,
                    (((*FeatureUnitDescriptor).bUnitID as i32) << 8) as USHORT,
                    TransferBuffer.offset(1isize) as PVOID,
                    core::mem::size_of::<USHORT>() as ULONG,
                    USBD_TRANSFER_DIRECTION_IN as ULONG,
                );

                if *TransferBuffer.offset(0isize) as LONG > Value {
                    Value = *TransferBuffer.offset(0isize) as LONG;
                }

                if (*TransferBuffer.offset(1isize) as LONG) < Value {
                    Value = *TransferBuffer.offset(1isize) as LONG;
                }

                *TransferBuffer.offset(2isize) = Value as SHORT;

                Status = UsbAudioGetSetProperty(
                    (*(*FilterContext).DeviceExtension).LowerDevice,
                    0x01u8,
                    (0x2i32 << 8) as USHORT,
                    (((*FeatureUnitDescriptor).bUnitID as i32) << 8) as USHORT,
                    TransferBuffer.offset(2isize) as PVOID,
                    core::mem::size_of::<USHORT>() as ULONG,
                    USBD_TRANSFER_DIRECTION_OUT as ULONG,
                );

                if NT_SUCCESS(Status) {
                    (*Irp).IoStatus.Information = core::mem::size_of::<LONG>() as _;
                }
            }
        }

        FreeFunction(TransferBuffer as PVOID);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn CountTopologyComponents(
    ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
    OutDescriptorCount: PULONG,
) -> ULONG {
    let mut Descriptor: PUSB_INTERFACE_DESCRIPTOR = core::ptr::null_mut();
    let mut InterfaceHeaderDescriptor: PUSB_AUDIO_CONTROL_INTERFACE_HEADER_DESCRIPTOR = core::ptr::null_mut();
    let mut CommonDescriptor: PUSB_COMMON_DESCRIPTOR = core::ptr::null_mut();
    let mut InputTerminalDescriptor: PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR = core::ptr::null_mut();
    let mut FeatureUnitDescriptor: PUSB_AUDIO_CONTROL_FEATURE_UNIT_DESCRIPTOR = core::ptr::null_mut();
    let mut MixerUnitDescriptor: PUSB_AUDIO_CONTROL_MIXER_UNIT_DESCRIPTOR = core::ptr::null_mut();
    let mut NodeCount: ULONG = 0;
    let mut Length: ULONG = 0;
    let mut Index: ULONG = 0;
    let mut DescriptorCount: ULONG = 0;
    let mut Value: UCHAR = 0;

    Descriptor = USBD_ParseConfigurationDescriptorEx(
        ConfigurationDescriptor,
        ConfigurationDescriptor as PVOID,
        -1i32 as LONG,
        -1i32 as LONG,
        USB_DEVICE_CLASS_AUDIO as LONG,
        -1i32 as LONG,
        -1i32 as LONG,
    );

    while !Descriptor.is_null() {
        if (*Descriptor).bInterfaceSubClass as i32 == 0x01 {
            InterfaceHeaderDescriptor = USBD_ParseDescriptors(
                ConfigurationDescriptor as PVOID,
                (*ConfigurationDescriptor).wTotalLength as ULONG,
                Descriptor as PVOID,
                USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE as LONG,
            ) as PUSB_AUDIO_CONTROL_INTERFACE_HEADER_DESCRIPTOR;

            if !InterfaceHeaderDescriptor.is_null() {
                CommonDescriptor = USBD_ParseDescriptors(
                    InterfaceHeaderDescriptor as PVOID,
                    (*InterfaceHeaderDescriptor).wTotalLength as ULONG,
                    (InterfaceHeaderDescriptor as ULONG_PTR).wrapping_add(
                        (*InterfaceHeaderDescriptor).bLength as ULONG_PTR,
                    ) as PVOID,
                    USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE as LONG,
                ) as PUSB_COMMON_DESCRIPTOR;

                while !CommonDescriptor.is_null() {
                    InputTerminalDescriptor = CommonDescriptor as PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR;

                    if (*InputTerminalDescriptor).bDescriptorSubtype as i32 == 0x02
                        || (*InputTerminalDescriptor).bDescriptorSubtype as i32 == 0x03
                    {
                        NodeCount = NodeCount.wrapping_add(1);
                        DescriptorCount = DescriptorCount.wrapping_add(1);
                    } else if (*InputTerminalDescriptor).bDescriptorSubtype as i32 == 0x06 {
                        FeatureUnitDescriptor = InputTerminalDescriptor as PUSB_AUDIO_CONTROL_FEATURE_UNIT_DESCRIPTOR;
                        DescriptorCount = DescriptorCount.wrapping_add(1);

                        Value = 0;
                        Length = ((*FeatureUnitDescriptor).bLength as i32 - 7) as ULONG;
                        Index = 0;
                        while Index < Length {
                            Value |= (*FeatureUnitDescriptor).bmaControls[Index as usize];
                            Index = Index.wrapping_add(1);
                        }

                        if Value & 0x01 != 0 { NodeCount = NodeCount.wrapping_add(1); }
                        if Value & 0x02 != 0 { NodeCount = NodeCount.wrapping_add(1); }
                        if Value & 0x04 != 0 { NodeCount = NodeCount.wrapping_add(1); }
                        if Value & 0x08 != 0 { NodeCount = NodeCount.wrapping_add(1); }
                        if Value & 0x10 != 0 { NodeCount = NodeCount.wrapping_add(1); }
                        if Value & 0x20 != 0 { NodeCount = NodeCount.wrapping_add(1); }
                        if Value & 0x40 != 0 { NodeCount = NodeCount.wrapping_add(1); }
                        if Value & 0x80 != 0 { NodeCount = NodeCount.wrapping_add(1); }
                    } else if (*InputTerminalDescriptor).bDescriptorSubtype as i32 == 0x04 {
                        MixerUnitDescriptor = InputTerminalDescriptor as PUSB_AUDIO_CONTROL_MIXER_UNIT_DESCRIPTOR;
                        DescriptorCount = DescriptorCount.wrapping_add(1);
                        NodeCount = NodeCount.wrapping_add((*MixerUnitDescriptor).bNrInPins as ULONG + 1);
                    } else if (*InputTerminalDescriptor).bDescriptorSubtype as i32 == 0x05 {
                        DescriptorCount = DescriptorCount.wrapping_add(1);
                        NodeCount = NodeCount.wrapping_add(1);
                    } else {
                        DbgPrint(
                            b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as PCSTR,
                            b"CountTopologyComponents\0" as *const u8 as *const i8,
                            b"drivers/usb/usbaudio/filter.c\0" as *const u8 as *const i8,
                            394i32,
                        );
                    }

                    CommonDescriptor = (CommonDescriptor as ULONG_PTR)
                        .wrapping_add((*CommonDescriptor).bLength as ULONG_PTR)
                        as PUSB_COMMON_DESCRIPTOR;

                    if CommonDescriptor as ULONG_PTR
                        >= (InterfaceHeaderDescriptor as ULONG_PTR)
                            .wrapping_add((*InterfaceHeaderDescriptor).wTotalLength as ULONG_PTR)
                    {
                        break;
                    }
                }
            }
        }

        Descriptor = USBD_ParseConfigurationDescriptorEx(
            ConfigurationDescriptor,
            (Descriptor as ULONG_PTR)
                .wrapping_add((*Descriptor).bLength as ULONG_PTR) as PVOID,
            -1i32 as LONG,
            -1i32 as LONG,
            USB_DEVICE_CLASS_AUDIO as LONG,
            -1i32 as LONG,
            -1i32 as LONG,
        );
    }

    *OutDescriptorCount = DescriptorCount;
    NodeCount
}
#[no_mangle]
pub unsafe extern "C" fn FindNodeContextWithId(
    NodeContext: PNODE_CONTEXT,
    NodeContextCount: ULONG,
    TerminalId: UCHAR,
) -> PNODE_CONTEXT {
    let mut Index: ULONG = 0;
    let mut TerminalDescriptor: PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR;

    while Index < NodeContextCount {
        TerminalDescriptor = (*NodeContext.offset(Index as isize)).Descriptor
            as PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR;
        if (*TerminalDescriptor).bTerminalID == TerminalId {
            return NodeContext.offset(Index as isize) as PNODE_CONTEXT;
        }
        Index = Index.wrapping_add(1);
    }
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn BuildUSBAudioFilterTopology(
    Device: PKSDEVICE,
    FilterDescriptor: PKSFILTER_DESCRIPTOR,
) -> NTSTATUS {
    let DeviceExtension = (*Device).Context as PDEVICE_EXTENSION;
    let mut ControlDescriptorCount: ULONG = 0;

    let NodeCount = CountTopologyComponents(
        (*DeviceExtension).ConfigurationDescriptor,
        &mut ControlDescriptorCount,
    );

    let NodeDescriptors = AllocFunction(
        (NodeCount as usize).wrapping_mul(core::mem::size_of::<KSNODE_DESCRIPTOR>()) as ULONG,
    ) as PKSNODE_DESCRIPTOR;
    (*FilterDescriptor).NodeDescriptors = NodeDescriptors;
    if (*FilterDescriptor).NodeDescriptors.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    (*FilterDescriptor).NodeDescriptorSize = core::mem::size_of::<KSNODE_DESCRIPTOR>() as ULONG;

    let NodeContext = AllocFunction(
        (core::mem::size_of::<NODE_CONTEXT>() as usize).wrapping_mul(ControlDescriptorCount as usize)
            as ULONG,
    ) as PNODE_CONTEXT;
    (*DeviceExtension).NodeContext = NodeContext;
    if NodeContext.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    (*DeviceExtension).NodeContextCount = ControlDescriptorCount;

    let mut DescriptorCount: ULONG = 0;
    let mut StreamingTerminalIndex: ULONG = 0;
    let mut NonStreamingTerminalDescriptorCount: ULONG = 0;
    let mut TotalTerminalDescriptorCount: ULONG = 0;
    let mut StreamingTerminalPinOffset: ULONG = 0;
    let mut Index: ULONG;
    let mut Value: UCHAR;
    let mut Length: ULONG;

    let mut Descriptor = USBD_ParseConfigurationDescriptorEx(
        (*DeviceExtension).ConfigurationDescriptor,
        (*DeviceExtension).ConfigurationDescriptor as PVOID,
        -1i32 as LONG,
        -1i32 as LONG,
        USB_DEVICE_CLASS_AUDIO as LONG,
        -1i32 as LONG,
        -1i32 as LONG,
    );
    while !Descriptor.is_null() {
        if (*Descriptor).bInterfaceSubClass == 0x01 {
            let InterfaceHeaderDescriptor = USBD_ParseDescriptors(
                (*DeviceExtension).ConfigurationDescriptor as PVOID,
                (*(*DeviceExtension).ConfigurationDescriptor).wTotalLength as ULONG,
                Descriptor as PVOID,
                USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE as LONG,
            ) as PUSB_AUDIO_CONTROL_INTERFACE_HEADER_DESCRIPTOR;

            if !InterfaceHeaderDescriptor.is_null() {
                let mut CommonDescriptor = USBD_ParseDescriptors(
                    InterfaceHeaderDescriptor as PVOID,
                    (*InterfaceHeaderDescriptor).wTotalLength as ULONG,
                    (InterfaceHeaderDescriptor as ULONG_PTR)
                        .wrapping_add((*InterfaceHeaderDescriptor).bLength as ULONG_PTR)
                        as PVOID,
                    USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE as LONG,
                ) as PUSB_COMMON_DESCRIPTOR;

                while !CommonDescriptor.is_null() {
                    let InputTerminalDescriptor = CommonDescriptor as PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR;

                    if (*InputTerminalDescriptor).bDescriptorSubtype == 0x02 {
                        if (*InputTerminalDescriptor).wTerminalType as u32 == USB_AUDIO_STREAMING_TERMINAL_TYPE {
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Type = &KSNODETYPE_SRC;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Name = &KSNODETYPE_SRC;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).AutomationTable =
                                AllocFunction(core::mem::size_of::<KSAUTOMATION_TABLE>() as ULONG) as *const KSAUTOMATION_TABLE;

                            (*NodeContext.offset(DescriptorCount as isize)).Descriptor = CommonDescriptor;
                            (*NodeContext.offset(DescriptorCount as isize)).NodeCount = 1;
                            (*NodeContext.offset(DescriptorCount as isize)).Nodes[0] = (*FilterDescriptor).NodeDescriptorsCount;
                            DescriptorCount = DescriptorCount.wrapping_add(1);
                            (*FilterDescriptor).NodeDescriptorsCount = (*FilterDescriptor).NodeDescriptorsCount.wrapping_add(1);
                        } else if ((*InputTerminalDescriptor).wTerminalType & 0xFF00) == 0x200 {
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Type = &KSNODETYPE_ADC;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Name = &KSNODETYPE_ADC;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).AutomationTable =
                                AllocFunction(core::mem::size_of::<KSAUTOMATION_TABLE>() as ULONG) as *const KSAUTOMATION_TABLE;

                            (*NodeContext.offset(DescriptorCount as isize)).Descriptor = CommonDescriptor;
                            (*NodeContext.offset(DescriptorCount as isize)).NodeCount = 1;
                            (*NodeContext.offset(DescriptorCount as isize)).Nodes[0] = (*FilterDescriptor).NodeDescriptorsCount;
                            DescriptorCount = DescriptorCount.wrapping_add(1);
                            (*FilterDescriptor).NodeDescriptorsCount = (*FilterDescriptor).NodeDescriptorsCount.wrapping_add(1);
                        } else if ((*InputTerminalDescriptor).wTerminalType & 0xFF00) == 0x300 {
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Type = &KSNODETYPE_DAC;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Name = &KSNODETYPE_DAC;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).AutomationTable =
                                AllocFunction(core::mem::size_of::<KSAUTOMATION_TABLE>() as ULONG) as *const KSAUTOMATION_TABLE;

                            (*NodeContext.offset(DescriptorCount as isize)).Descriptor = CommonDescriptor;
                            (*NodeContext.offset(DescriptorCount as isize)).NodeCount = 1;
                            (*NodeContext.offset(DescriptorCount as isize)).Nodes[0] = (*FilterDescriptor).NodeDescriptorsCount;
                            DescriptorCount = DescriptorCount.wrapping_add(1);
                            (*FilterDescriptor).NodeDescriptorsCount = (*FilterDescriptor).NodeDescriptorsCount.wrapping_add(1);
                        } else {
                            // DPRINT removed
                        }
                    } else if (*InputTerminalDescriptor).bDescriptorSubtype == 0x03 {
                        if (*InputTerminalDescriptor).wTerminalType as u32 == USB_AUDIO_STREAMING_TERMINAL_TYPE {
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Type = &KSNODETYPE_SRC;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Name = &KSNODETYPE_SRC;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).AutomationTable =
                                AllocFunction(core::mem::size_of::<KSAUTOMATION_TABLE>() as ULONG) as *const KSAUTOMATION_TABLE;

                            (*NodeContext.offset(DescriptorCount as isize)).Descriptor = CommonDescriptor;
                            (*NodeContext.offset(DescriptorCount as isize)).NodeCount = 1;
                            (*NodeContext.offset(DescriptorCount as isize)).Nodes[0] = (*FilterDescriptor).NodeDescriptorsCount;
                            DescriptorCount = DescriptorCount.wrapping_add(1);
                            (*FilterDescriptor).NodeDescriptorsCount = (*FilterDescriptor).NodeDescriptorsCount.wrapping_add(1);
                        } else if ((*InputTerminalDescriptor).wTerminalType & 0xFF00) == 0x300 {
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Type = &KSNODETYPE_DAC;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Name = &KSNODETYPE_DAC;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).AutomationTable =
                                AllocFunction(core::mem::size_of::<KSAUTOMATION_TABLE>() as ULONG) as *const KSAUTOMATION_TABLE;

                            (*NodeContext.offset(DescriptorCount as isize)).Descriptor = CommonDescriptor;
                            (*NodeContext.offset(DescriptorCount as isize)).NodeCount = 1;
                            (*NodeContext.offset(DescriptorCount as isize)).Nodes[0] = (*FilterDescriptor).NodeDescriptorsCount;
                            DescriptorCount = DescriptorCount.wrapping_add(1);
                            (*FilterDescriptor).NodeDescriptorsCount = (*FilterDescriptor).NodeDescriptorsCount.wrapping_add(1);
                        } else {
                            // DPRINT removed
                        }
                    } else if (*InputTerminalDescriptor).bDescriptorSubtype == 0x06 {
                        let FeatureUnitDescriptor = CommonDescriptor as PUSB_AUDIO_CONTROL_FEATURE_UNIT_DESCRIPTOR;

                        Value = 0;
                        Length = ((*FeatureUnitDescriptor).bLength as i32 - 7) as ULONG;
                        Index = 0;
                        while Index < Length {
                            Value = (Value as i32 | (*FeatureUnitDescriptor).bmaControls[Index as usize] as i32) as UCHAR;
                            Index = Index.wrapping_add(1);
                        }

                        if Value & 0x01 != 0 {
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Type = &KSNODETYPE_MUTE;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Name = &KSNODETYPE_MUTE;
                            let AutomationTable = AllocFunction(core::mem::size_of::<KSAUTOMATION_TABLE>() as ULONG) as PKSAUTOMATION_TABLE;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).AutomationTable = AutomationTable;
                            if !AutomationTable.is_null() {
                                (*AutomationTable).PropertySets = &mut FilterAudioMutePropertySetArray as *mut KSPROPERTY_SET;
                                (*AutomationTable).PropertySetsCount = 1;
                                (*AutomationTable).PropertyItemSize = core::mem::size_of::<KSPROPERTY_ITEM>() as ULONG;
                            }

                            (*NodeContext.offset(DescriptorCount as isize)).Nodes[(*NodeContext.offset(DescriptorCount as isize)).NodeCount as usize] = (*FilterDescriptor).NodeDescriptorsCount;
                            (*NodeContext.offset(DescriptorCount as isize)).NodeCount = (*NodeContext.offset(DescriptorCount as isize)).NodeCount.wrapping_add(1);
                            (*FilterDescriptor).NodeDescriptorsCount = (*FilterDescriptor).NodeDescriptorsCount.wrapping_add(1);
                        }
                        if Value & 0x02 != 0 {
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Type = &KSNODETYPE_VOLUME;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Name = &KSNODETYPE_VOLUME;
                            let AutomationTable = AllocFunction(core::mem::size_of::<KSAUTOMATION_TABLE>() as ULONG) as PKSAUTOMATION_TABLE;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).AutomationTable = AutomationTable;
                            if !AutomationTable.is_null() {
                                (*AutomationTable).PropertySets = &mut FilterAudioVolumePropertySetArray as *mut KSPROPERTY_SET;
                                (*AutomationTable).PropertySetsCount = 1;
                                (*AutomationTable).PropertyItemSize = core::mem::size_of::<KSPROPERTY_ITEM>() as ULONG;
                            }

                            (*NodeContext.offset(DescriptorCount as isize)).Nodes[(*NodeContext.offset(DescriptorCount as isize)).NodeCount as usize] = (*FilterDescriptor).NodeDescriptorsCount;
                            (*NodeContext.offset(DescriptorCount as isize)).NodeCount = (*NodeContext.offset(DescriptorCount as isize)).NodeCount.wrapping_add(1);
                            (*FilterDescriptor).NodeDescriptorsCount = (*FilterDescriptor).NodeDescriptorsCount.wrapping_add(1);
                        }
                        if Value & 0x04 != 0 {
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Type = &KSNODETYPE_TONE;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Name = &KSNODETYPE_TONE;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).AutomationTable =
                                AllocFunction(core::mem::size_of::<KSAUTOMATION_TABLE>() as ULONG) as *const KSAUTOMATION_TABLE;

                            (*NodeContext.offset(DescriptorCount as isize)).Nodes[(*NodeContext.offset(DescriptorCount as isize)).NodeCount as usize] = (*FilterDescriptor).NodeDescriptorsCount;
                            (*NodeContext.offset(DescriptorCount as isize)).NodeCount = (*NodeContext.offset(DescriptorCount as isize)).NodeCount.wrapping_add(1);
                            (*FilterDescriptor).NodeDescriptorsCount = (*FilterDescriptor).NodeDescriptorsCount.wrapping_add(1);
                        }
                        if Value & 0x08 != 0 {
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Type = &KSNODETYPE_TONE;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Name = &KSNODETYPE_TONE;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).AutomationTable =
                                AllocFunction(core::mem::size_of::<KSAUTOMATION_TABLE>() as ULONG) as *const KSAUTOMATION_TABLE;

                            (*NodeContext.offset(DescriptorCount as isize)).Nodes[(*NodeContext.offset(DescriptorCount as isize)).NodeCount as usize] = (*FilterDescriptor).NodeDescriptorsCount;
                            (*NodeContext.offset(DescriptorCount as isize)).NodeCount = (*NodeContext.offset(DescriptorCount as isize)).NodeCount.wrapping_add(1);
                            (*FilterDescriptor).NodeDescriptorsCount = (*FilterDescriptor).NodeDescriptorsCount.wrapping_add(1);
                        }
                        if Value & 0x10 != 0 {
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Type = &KSNODETYPE_TONE;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Name = &KSNODETYPE_TONE;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).AutomationTable =
                                AllocFunction(core::mem::size_of::<KSAUTOMATION_TABLE>() as ULONG) as *const KSAUTOMATION_TABLE;

                            (*NodeContext.offset(DescriptorCount as isize)).Nodes[(*NodeContext.offset(DescriptorCount as isize)).NodeCount as usize] = (*FilterDescriptor).NodeDescriptorsCount;
                            (*NodeContext.offset(DescriptorCount as isize)).NodeCount = (*NodeContext.offset(DescriptorCount as isize)).NodeCount.wrapping_add(1);
                            (*FilterDescriptor).NodeDescriptorsCount = (*FilterDescriptor).NodeDescriptorsCount.wrapping_add(1);
                        }
                        if Value & 0x20 != 0 {
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Type = &KSNODETYPE_TONE;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Name = &KSNODETYPE_TONE;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).AutomationTable =
                                AllocFunction(core::mem::size_of::<KSAUTOMATION_TABLE>() as ULONG) as *const KSAUTOMATION_TABLE;

                            (*NodeContext.offset(DescriptorCount as isize)).Nodes[(*NodeContext.offset(DescriptorCount as isize)).NodeCount as usize] = (*FilterDescriptor).NodeDescriptorsCount;
                            (*NodeContext.offset(DescriptorCount as isize)).NodeCount = (*NodeContext.offset(DescriptorCount as isize)).NodeCount.wrapping_add(1);
                            (*FilterDescriptor).NodeDescriptorsCount = (*FilterDescriptor).NodeDescriptorsCount.wrapping_add(1);
                        }
                        if Value & 0x40 != 0 {
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Type = &KSNODETYPE_AGC;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Name = &KSNODETYPE_AGC;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).AutomationTable =
                                AllocFunction(core::mem::size_of::<KSAUTOMATION_TABLE>() as ULONG) as *const KSAUTOMATION_TABLE;

                            (*NodeContext.offset(DescriptorCount as isize)).Nodes[(*NodeContext.offset(DescriptorCount as isize)).NodeCount as usize] = (*FilterDescriptor).NodeDescriptorsCount;
                            (*NodeContext.offset(DescriptorCount as isize)).NodeCount = (*NodeContext.offset(DescriptorCount as isize)).NodeCount.wrapping_add(1);
                            (*FilterDescriptor).NodeDescriptorsCount = (*FilterDescriptor).NodeDescriptorsCount.wrapping_add(1);
                        }
                        if Value & 0x80 != 0 {
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Type = &KSNODETYPE_TONE;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Name = &KSNODETYPE_TONE;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).AutomationTable =
                                AllocFunction(core::mem::size_of::<KSAUTOMATION_TABLE>() as ULONG) as *const KSAUTOMATION_TABLE;

                            (*NodeContext.offset(DescriptorCount as isize)).Nodes[(*NodeContext.offset(DescriptorCount as isize)).NodeCount as usize] = (*FilterDescriptor).NodeDescriptorsCount;
                            (*NodeContext.offset(DescriptorCount as isize)).NodeCount = (*NodeContext.offset(DescriptorCount as isize)).NodeCount.wrapping_add(1);
                            (*FilterDescriptor).NodeDescriptorsCount = (*FilterDescriptor).NodeDescriptorsCount.wrapping_add(1);
                        }
                        (*NodeContext.offset(DescriptorCount as isize)).Descriptor = CommonDescriptor;
                        DescriptorCount = DescriptorCount.wrapping_add(1);
                    } else if (*InputTerminalDescriptor).bDescriptorSubtype == 0x04 {
                        let MixerUnitDescriptor = CommonDescriptor as PUSB_AUDIO_CONTROL_MIXER_UNIT_DESCRIPTOR;
                        Index = 0;
                        while Index < (*MixerUnitDescriptor).bNrInPins as ULONG {
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Type = &KSNODETYPE_SUPERMIX;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Name = &KSNODETYPE_SUPERMIX;
                            (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).AutomationTable =
                                AllocFunction(core::mem::size_of::<KSAUTOMATION_TABLE>() as ULONG) as *const KSAUTOMATION_TABLE;

                            (*NodeContext.offset(DescriptorCount as isize)).Nodes[(*NodeContext.offset(DescriptorCount as isize)).NodeCount as usize] = (*FilterDescriptor).NodeDescriptorsCount;
                            (*NodeContext.offset(DescriptorCount as isize)).NodeCount = (*NodeContext.offset(DescriptorCount as isize)).NodeCount.wrapping_add(1);
                            (*FilterDescriptor).NodeDescriptorsCount = (*FilterDescriptor).NodeDescriptorsCount.wrapping_add(1);
                            Index = Index.wrapping_add(1);
                        }

                        (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Type = &KSNODETYPE_SUM;
                        (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Name = &KSNODETYPE_SUM;
                        (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).AutomationTable =
                            AllocFunction(core::mem::size_of::<KSAUTOMATION_TABLE>() as ULONG) as *const KSAUTOMATION_TABLE;

                        (*NodeContext.offset(DescriptorCount as isize)).Nodes[(*NodeContext.offset(DescriptorCount as isize)).NodeCount as usize] = (*FilterDescriptor).NodeDescriptorsCount;
                        (*NodeContext.offset(DescriptorCount as isize)).NodeCount = (*NodeContext.offset(DescriptorCount as isize)).NodeCount.wrapping_add(1);
                        (*NodeContext.offset(DescriptorCount as isize)).Descriptor = CommonDescriptor;
                        DescriptorCount = DescriptorCount.wrapping_add(1);
                        (*FilterDescriptor).NodeDescriptorsCount = (*FilterDescriptor).NodeDescriptorsCount.wrapping_add(1);
                    } else if (*InputTerminalDescriptor).bDescriptorSubtype == 0x05 {
                        (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Type = &KSNODETYPE_MUX;
                        (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).Name = &KSNODETYPE_MUX;
                        (*NodeDescriptors.offset((*FilterDescriptor).NodeDescriptorsCount as isize)).AutomationTable =
                            AllocFunction(core::mem::size_of::<KSAUTOMATION_TABLE>() as ULONG) as *const KSAUTOMATION_TABLE;

                        (*NodeContext.offset(DescriptorCount as isize)).Descriptor = CommonDescriptor;
                        (*NodeContext.offset(DescriptorCount as isize)).NodeCount = 1;
                        (*NodeContext.offset(DescriptorCount as isize)).Nodes[0] = (*FilterDescriptor).NodeDescriptorsCount;
                        DescriptorCount = DescriptorCount.wrapping_add(1);
                        (*FilterDescriptor).NodeDescriptorsCount = (*FilterDescriptor).NodeDescriptorsCount.wrapping_add(1);
                    } else {
                        unimplemented!();
                    }

                    CommonDescriptor = (CommonDescriptor as ULONG_PTR)
                        .wrapping_add((*CommonDescriptor).bLength as ULONG_PTR)
                        as PUSB_COMMON_DESCRIPTOR;
                    if (CommonDescriptor as ULONG_PTR)
                        >= (InterfaceHeaderDescriptor as ULONG_PTR)
                            .wrapping_add((*InterfaceHeaderDescriptor).wTotalLength as ULONG_PTR)
                    {
                        break;
                    }
                }
            }
        }

        Descriptor = USBD_ParseConfigurationDescriptorEx(
            (*DeviceExtension).ConfigurationDescriptor,
            (Descriptor as ULONG_PTR).wrapping_add((*Descriptor).bLength as ULONG_PTR) as PVOID,
            -1i32 as LONG,
            -1i32 as LONG,
            USB_DEVICE_CLASS_AUDIO as LONG,
            -1i32 as LONG,
            -1i32 as LONG,
        );
    }

    let Connections = AllocFunction(
        (core::mem::size_of::<KSTOPOLOGY_CONNECTION>() as usize)
            .wrapping_mul((*FilterDescriptor).NodeDescriptorsCount as usize)
            .wrapping_mul(2) as ULONG,
    ) as PKSTOPOLOGY_CONNECTION;
    (*FilterDescriptor).Connections = Connections;
    if (*FilterDescriptor).Connections.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    (*FilterDescriptor).ConnectionsCount = 0;

    DescriptorCount = 0;
    StreamingTerminalIndex = 0;
    let mut NodeCount: ULONG = 0;

    CountTerminalUnits(
        (*DeviceExtension).ConfigurationDescriptor,
        &mut NonStreamingTerminalDescriptorCount,
        &mut TotalTerminalDescriptorCount,
    );
    StreamingTerminalPinOffset = TotalTerminalDescriptorCount.wrapping_sub(NonStreamingTerminalDescriptorCount);

    Descriptor = USBD_ParseConfigurationDescriptorEx(
        (*DeviceExtension).ConfigurationDescriptor,
        (*DeviceExtension).ConfigurationDescriptor as PVOID,
        -1i32 as LONG,
        -1i32 as LONG,
        USB_DEVICE_CLASS_AUDIO as LONG,
        -1i32 as LONG,
        -1i32 as LONG,
    );
    while !Descriptor.is_null() {
        if (*Descriptor).bInterfaceSubClass == 0x01 {
            let InterfaceHeaderDescriptor = USBD_ParseDescriptors(
                (*DeviceExtension).ConfigurationDescriptor as PVOID,
                (*(*DeviceExtension).ConfigurationDescriptor).wTotalLength as ULONG,
                Descriptor as PVOID,
                USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE as LONG,
            ) as PUSB_AUDIO_CONTROL_INTERFACE_HEADER_DESCRIPTOR;

            if !InterfaceHeaderDescriptor.is_null() {
                let mut CommonDescriptor = USBD_ParseDescriptors(
                    InterfaceHeaderDescriptor as PVOID,
                    (*InterfaceHeaderDescriptor).wTotalLength as ULONG,
                    (InterfaceHeaderDescriptor as ULONG_PTR)
                        .wrapping_add((*InterfaceHeaderDescriptor).bLength as ULONG_PTR)
                        as PVOID,
                    USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE as LONG,
                ) as PUSB_COMMON_DESCRIPTOR;

                while !CommonDescriptor.is_null() {
                    let InputTerminalDescriptor = CommonDescriptor as PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR;

                    if (*InputTerminalDescriptor).bDescriptorSubtype == 0x02 {
                        if (*InputTerminalDescriptor).wTerminalType as u32 == USB_AUDIO_STREAMING_TERMINAL_TYPE {
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).FromNode = KSFILTER_NODE;
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).FromNodePin = StreamingTerminalIndex;
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).ToNodePin = 1;
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).ToNode = (*NodeContext.offset(DescriptorCount as isize)).Nodes[0];
                            (*FilterDescriptor).ConnectionsCount = (*FilterDescriptor).ConnectionsCount.wrapping_add(1);
                            StreamingTerminalIndex = StreamingTerminalIndex.wrapping_add(1);
                        } else {
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).FromNode = KSFILTER_NODE;
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).FromNodePin = StreamingTerminalPinOffset;
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).ToNodePin = 1;
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).ToNode = (*NodeContext.offset(DescriptorCount as isize)).Nodes[0];
                            (*FilterDescriptor).ConnectionsCount = (*FilterDescriptor).ConnectionsCount.wrapping_add(1);
                            StreamingTerminalPinOffset = StreamingTerminalPinOffset.wrapping_add(1);
                        }
                        DescriptorCount = DescriptorCount.wrapping_add(1);
                    } else if (*InputTerminalDescriptor).bDescriptorSubtype == 0x03 {
                        let OutputTerminalDescriptor = CommonDescriptor as PUSB_AUDIO_CONTROL_OUTPUT_TERMINAL_DESCRIPTOR;
                        let PreviousNodeContext = FindNodeContextWithId(
                            NodeContext,
                            ControlDescriptorCount,
                            (*OutputTerminalDescriptor).bSourceID,
                        );
                        if !PreviousNodeContext.is_null() {
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).FromNode =
                                (*PreviousNodeContext).Nodes[((*PreviousNodeContext).NodeCount).wrapping_sub(1) as usize];
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).FromNodePin = 0;
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).ToNodePin = 1;
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).ToNode = (*NodeContext.offset(DescriptorCount as isize)).Nodes[0];
                            (*FilterDescriptor).ConnectionsCount = (*FilterDescriptor).ConnectionsCount.wrapping_add(1);
                        }

                        if (*InputTerminalDescriptor).wTerminalType as u32 == USB_AUDIO_STREAMING_TERMINAL_TYPE {
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).FromNode = (*NodeContext.offset(DescriptorCount as isize)).Nodes[0];
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).FromNodePin = 0;
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).ToNodePin = StreamingTerminalIndex;
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).ToNode = KSFILTER_NODE;
                            (*FilterDescriptor).ConnectionsCount = (*FilterDescriptor).ConnectionsCount.wrapping_add(1);
                            StreamingTerminalIndex = StreamingTerminalIndex.wrapping_add(1);
                        } else {
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).FromNode = (*NodeContext.offset(DescriptorCount as isize)).Nodes[0];
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).FromNodePin = 0;
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).ToNodePin = StreamingTerminalPinOffset;
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).ToNode = KSFILTER_NODE;
                            (*FilterDescriptor).ConnectionsCount = (*FilterDescriptor).ConnectionsCount.wrapping_add(1);
                            StreamingTerminalPinOffset = StreamingTerminalPinOffset.wrapping_add(1);
                        }
                        DescriptorCount = DescriptorCount.wrapping_add(1);
                    } else if (*InputTerminalDescriptor).bDescriptorSubtype == 0x06 {
                        let FeatureUnitDescriptor = InputTerminalDescriptor as PUSB_AUDIO_CONTROL_FEATURE_UNIT_DESCRIPTOR;
                        let PreviousNodeContext = FindNodeContextWithId(
                            NodeContext,
                            ControlDescriptorCount,
                            (*FeatureUnitDescriptor).bSourceID,
                        );
                        if !PreviousNodeContext.is_null() {
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).FromNode =
                                (*PreviousNodeContext).Nodes[((*PreviousNodeContext).NodeCount).wrapping_sub(1) as usize];
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).FromNodePin = 0;
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).ToNodePin = 1;
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).ToNode = (*NodeContext.offset(DescriptorCount as isize)).Nodes[0];
                            (*FilterDescriptor).ConnectionsCount = (*FilterDescriptor).ConnectionsCount.wrapping_add(1);
                        }

                        Index = 1;
                        while Index < (*NodeContext.offset(DescriptorCount as isize)).NodeCount {
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).FromNode =
                                (*NodeContext.offset(DescriptorCount as isize)).Nodes[Index.wrapping_sub(1) as usize];
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).FromNodePin = 0;
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).ToNodePin = 1;
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).ToNode = (*NodeContext.offset(DescriptorCount as isize)).Nodes[Index as usize];
                            (*FilterDescriptor).ConnectionsCount = (*FilterDescriptor).ConnectionsCount.wrapping_add(1);
                            Index = Index.wrapping_add(1);
                        }
                        DescriptorCount = DescriptorCount.wrapping_add(1);
                    } else if (*InputTerminalDescriptor).bDescriptorSubtype == 0x04 {
                        let MixerUnitDescriptor = InputTerminalDescriptor as PUSB_AUDIO_CONTROL_MIXER_UNIT_DESCRIPTOR;
                        Index = 0;
                        while Index < (*MixerUnitDescriptor).bNrInPins as ULONG {
                            Value = (*MixerUnitDescriptor).baSourceID[Index as usize];
                            let PreviousNodeContext = FindNodeContextWithId(
                                NodeContext,
                                ControlDescriptorCount,
                                Value,
                            );
                            if !PreviousNodeContext.is_null() {
                                (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).FromNode =
                                    (*PreviousNodeContext).Nodes[((*PreviousNodeContext).NodeCount).wrapping_sub(1) as usize];
                                (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).FromNodePin = 0;
                                (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).ToNodePin = 1;
                                (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).ToNode = (*NodeContext.offset(DescriptorCount as isize)).Nodes[Index as usize];
                                (*FilterDescriptor).ConnectionsCount = (*FilterDescriptor).ConnectionsCount.wrapping_add(1);
                            }

                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).FromNode = (*NodeContext.offset(DescriptorCount as isize)).Nodes[Index as usize];
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).FromNodePin = 0;
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).ToNodePin = 1u32.wrapping_add(Index);
                            (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).ToNode =
                                (*NodeContext.offset(DescriptorCount as isize)).Nodes[((*NodeContext.offset(DescriptorCount as isize)).NodeCount).wrapping_sub(1) as usize];
                            (*FilterDescriptor).ConnectionsCount = (*FilterDescriptor).ConnectionsCount.wrapping_add(1);
                            Index = Index.wrapping_add(1);
                        }
                        DescriptorCount = DescriptorCount.wrapping_add(1);
                    } else if (*InputTerminalDescriptor).bDescriptorSubtype == 0x05 {
                        let SelectorUnitDescriptor = InputTerminalDescriptor as PUSB_AUDIO_CONTROL_SELECTOR_UNIT_DESCRIPTOR;
                        Index = 0;
                        while Index < (*SelectorUnitDescriptor).bNrInPins as ULONG {
                            Value = (*SelectorUnitDescriptor).baSourceID[Index as usize];
                            let PreviousNodeContext = FindNodeContextWithId(
                                NodeContext,
                                ControlDescriptorCount,
                                Value,
                            );
                            if !PreviousNodeContext.is_null() {
                                (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).FromNode =
                                    (*PreviousNodeContext).Nodes[((*PreviousNodeContext).NodeCount).wrapping_sub(1) as usize];
                                (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).FromNodePin = 0;
                                (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).ToNodePin = 1;
                                (*Connections.offset((*FilterDescriptor).ConnectionsCount as isize)).ToNode = (*NodeContext.offset(DescriptorCount as isize)).Nodes[0];
                                (*FilterDescriptor).ConnectionsCount = (*FilterDescriptor).ConnectionsCount.wrapping_add(1);
                            }
                            Index = Index.wrapping_add(1);
                        }
                        DescriptorCount = DescriptorCount.wrapping_add(1);
                    } else {
                        unimplemented!();
                    }

                    CommonDescriptor = (CommonDescriptor as ULONG_PTR)
                        .wrapping_add((*CommonDescriptor).bLength as ULONG_PTR)
                        as PUSB_COMMON_DESCRIPTOR;
                    if (CommonDescriptor as ULONG_PTR)
                        >= (InterfaceHeaderDescriptor as ULONG_PTR)
                            .wrapping_add((*InterfaceHeaderDescriptor).wTotalLength as ULONG_PTR)
                    {
                        break;
                    }
                }
            }
        }

        Descriptor = USBD_ParseConfigurationDescriptorEx(
            (*DeviceExtension).ConfigurationDescriptor,
            (Descriptor as ULONG_PTR).wrapping_add((*Descriptor).bLength as ULONG_PTR) as PVOID,
            -1i32 as LONG,
            -1i32 as LONG,
            USB_DEVICE_CLASS_AUDIO as LONG,
            -1i32 as LONG,
            -1i32 as LONG,
        );
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn USBAudioFilterCreate(
    Filter: PKSFILTER,
    Irp: PIRP,
) -> NTSTATUS {
    let FilterFactory: PKSFILTERFACTORY = KsGetParent(Filter as PVOID) as PKSFILTERFACTORY;
    if FilterFactory.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let Device: PKSDEVICE = KsGetParent(FilterFactory as PVOID) as PKSDEVICE;
    if Device.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let FilterContext: PFILTER_CONTEXT =
        AllocFunction(core::mem::size_of::<FILTER_CONTEXT>() as ULONG) as PFILTER_CONTEXT;
    if FilterContext.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*FilterContext).DeviceExtension = (*Device).Context as PDEVICE_EXTENSION;
    (*FilterContext).LowerDevice = (*Device).NextDeviceObject;
    (*Filter).Context = FilterContext as PVOID;

    KsAddItemToObjectBag(
        (*Filter).Bag,
        FilterContext as PVOID,
        Some(ExFreePool),
    );

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CountTerminalUnits(
    ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
    NonStreamingTerminalDescriptorCount: PULONG,
    TotalTerminalDescriptorCount: PULONG,
) {
    let mut NonStreamingTerminalCount: ULONG = 0;
    let mut TotalTerminalCount: ULONG = 0;

    let mut Descriptor = USBD_ParseConfigurationDescriptorEx(
        ConfigurationDescriptor,
        ConfigurationDescriptor as PVOID,
        -1i32 as LONG,
        -1i32 as LONG,
        USB_DEVICE_CLASS_AUDIO as LONG,
        -1i32 as LONG,
        -1i32 as LONG,
    );

    while !Descriptor.is_null() {
        if (*Descriptor).bInterfaceSubClass as i32 == 0x01 {
            let InterfaceHeaderDescriptor = USBD_ParseDescriptors(
                ConfigurationDescriptor as PVOID,
                (*ConfigurationDescriptor).wTotalLength as ULONG,
                Descriptor as PVOID,
                USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE as LONG,
            ) as PUSB_AUDIO_CONTROL_INTERFACE_HEADER_DESCRIPTOR;

            if !InterfaceHeaderDescriptor.is_null() {
                let mut CommonDescriptor = USBD_ParseDescriptors(
                    InterfaceHeaderDescriptor as PVOID,
                    (*InterfaceHeaderDescriptor).wTotalLength as ULONG,
                    (InterfaceHeaderDescriptor as ULONG_PTR)
                        .wrapping_add((*InterfaceHeaderDescriptor).bLength as ULONG_PTR)
                        as PVOID,
                    USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE as LONG,
                ) as PUSB_COMMON_DESCRIPTOR;

                while !CommonDescriptor.is_null() {
                    let InputTerminalDescriptor = CommonDescriptor as PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR;
                    if (*InputTerminalDescriptor).bDescriptorSubtype as i32 == 0x02
                        || (*InputTerminalDescriptor).bDescriptorSubtype as i32 == 0x03
                    {
                        if (*InputTerminalDescriptor).wTerminalType as u32 != USB_AUDIO_STREAMING_TERMINAL_TYPE {
                            NonStreamingTerminalCount = NonStreamingTerminalCount.wrapping_add(1);
                        }
                        TotalTerminalCount = TotalTerminalCount.wrapping_add(1);
                    }
                    CommonDescriptor = (CommonDescriptor as ULONG_PTR)
                        .wrapping_add((*CommonDescriptor).bLength as ULONG_PTR)
                        as PUSB_COMMON_DESCRIPTOR;
                    if (CommonDescriptor as ULONG_PTR)
                        >= (InterfaceHeaderDescriptor as ULONG_PTR)
                            .wrapping_add((*InterfaceHeaderDescriptor).wTotalLength as ULONG_PTR)
                    {
                        break;
                    }
                }
            }
        } else if (*Descriptor).bInterfaceSubClass as i32 == 0x03 {
            DbgPrint(
                b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
                b"CountTerminalUnits\0".as_ptr() as *const i8,
                b"drivers/usb/usbaudio/filter.c\0".as_ptr() as *const i8,
                1011i32,
            );
        }

        Descriptor = USBD_ParseConfigurationDescriptorEx(
            ConfigurationDescriptor,
            (Descriptor as ULONG_PTR)
                .wrapping_add((*Descriptor).bLength as ULONG_PTR)
                as PVOID,
            -1i32 as LONG,
            -1i32 as LONG,
            USB_DEVICE_CLASS_AUDIO as LONG,
            -1i32 as LONG,
            -1i32 as LONG,
        );
    }

    *NonStreamingTerminalDescriptorCount = NonStreamingTerminalCount;
    *TotalTerminalDescriptorCount = TotalTerminalCount;
}
#[no_mangle]
pub unsafe extern "C" fn UsbAudioGetPinCategoryFromTerminalDescriptor(
    TerminalDescriptor: PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR,
) -> LPGUID {
    if (*TerminalDescriptor).wTerminalType as u32 == USB_AUDIO_MICROPHONE_TERMINAL_TYPE {
        return &mut NodeTypeMicrophone;
    } else if (*TerminalDescriptor).wTerminalType as u32 == USB_AUDIO_DESKTOP_MICROPHONE_TERMINAL_TYPE {
        return &mut NodeTypeDesktopMicrophone;
    } else if (*TerminalDescriptor).wTerminalType as u32 == USB_AUDIO_PERSONAL_MICROPHONE_TERMINAL_TYPE {
        return &mut NodeTypePersonalMicrophone;
    } else if (*TerminalDescriptor).wTerminalType as u32 == USB_AUDIO_OMMNI_MICROPHONE_TERMINAL_TYPE {
        return &mut NodeTypeOmmniMicrophone;
    } else if (*TerminalDescriptor).wTerminalType as u32 == USB_AUDIO_ARRAY_MICROPHONE_TERMINAL_TYPE {
        return &mut NodeTypeArrayMicrophone;
    } else if (*TerminalDescriptor).wTerminalType as u32 == USB_AUDIO_ARRAY_PROCESSING_MICROPHONE_TERMINAL_TYPE {
        return &mut NodeTypeProcessingArrayMicrophone;
    }

    if (*TerminalDescriptor).wTerminalType as u32 == USB_AUDIO_SPEAKER_TERMINAL_TYPE {
        return &mut NodeTypeSpeaker;
    } else if (*TerminalDescriptor).wTerminalType as u32 == USB_HEADPHONES_SPEAKER_TERMINAL_TYPE {
        return &mut NodeTypeHeadphonesSpeaker;
    } else if (*TerminalDescriptor).wTerminalType as u32 == USB_AUDIO_HMDA_TERMINAL_TYPE {
        return &mut NodeTypeHMDA;
    } else if (*TerminalDescriptor).wTerminalType as u32 == USB_AUDIO_DESKTOP_SPEAKER_TERMINAL_TYPE {
        return &mut NodeTypeDesktopSpeaker;
    } else if (*TerminalDescriptor).wTerminalType as u32 == USB_AUDIO_ROOM_SPEAKER_TERMINAL_TYPE {
        return &mut NodeTypeRoomSpeaker;
    } else if (*TerminalDescriptor).wTerminalType as u32 == USB_AUDIO_COMMUNICATION_SPEAKER_TERMINAL_TYPE {
        return &mut NodeTypeCommunicationSpeaker;
    } else if (*TerminalDescriptor).wTerminalType as u32 == USB_AUDIO_SUBWOOFER_TERMINAL_TYPE {
        return &mut NodeTypeSubwoofer;
    }

    if (*TerminalDescriptor).wTerminalType as u32 == USB_AUDIO_STREAMING_TERMINAL_TYPE {
        if (*TerminalDescriptor).bDescriptorSubtype as u32 == USB_AUDIO_OUTPUT_TERMINAL {
            return &mut NodeTypeCapture;
        } else if (*TerminalDescriptor).bDescriptorSubtype as u32 == USB_AUDIO_INPUT_TERMINAL {
            return &mut NodeTypePlayback;
        }
    }

    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn UsbAudioGetStreamingTerminalDescriptorByIndex(
    ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
    Index: ULONG,
) -> PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR {
    let mut Descriptor: PUSB_INTERFACE_DESCRIPTOR;
    let mut InterfaceHeaderDescriptor: PUSB_AUDIO_CONTROL_INTERFACE_HEADER_DESCRIPTOR;
    let mut CommonDescriptor: PUSB_COMMON_DESCRIPTOR;
    let mut InputTerminalDescriptor: PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR;
    let mut TerminalCount: ULONG = 0;

    Descriptor = USBD_ParseConfigurationDescriptorEx(
        ConfigurationDescriptor,
        ConfigurationDescriptor as PVOID,
        -1i32 as LONG,
        -1i32 as LONG,
        USB_DEVICE_CLASS_AUDIO as LONG,
        -1i32 as LONG,
        -1i32 as LONG,
    );
    while !Descriptor.is_null() {
        if (*Descriptor).bInterfaceSubClass as i32 == 0x01 {
            InterfaceHeaderDescriptor = USBD_ParseDescriptors(
                ConfigurationDescriptor as PVOID,
                (*ConfigurationDescriptor).wTotalLength as ULONG,
                Descriptor as PVOID,
                USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE as LONG,
            ) as PUSB_AUDIO_CONTROL_INTERFACE_HEADER_DESCRIPTOR;
            if !InterfaceHeaderDescriptor.is_null() {
                CommonDescriptor = USBD_ParseDescriptors(
                    InterfaceHeaderDescriptor as PVOID,
                    (*InterfaceHeaderDescriptor).wTotalLength as ULONG,
                    (InterfaceHeaderDescriptor as ULONG_PTR)
                        .wrapping_add((*InterfaceHeaderDescriptor).bLength as ULONG_PTR)
                        as PVOID,
                    USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE as LONG,
                ) as PUSB_COMMON_DESCRIPTOR;
                while !CommonDescriptor.is_null() {
                    InputTerminalDescriptor =
                        CommonDescriptor as PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR;
                    if (*InputTerminalDescriptor).bDescriptorSubtype as i32 == 0x02
                        || (*InputTerminalDescriptor).bDescriptorSubtype as i32 == 0x03
                    {
                        if (*InputTerminalDescriptor).wTerminalType as u32
                            == USB_AUDIO_STREAMING_TERMINAL_TYPE
                        {
                            if TerminalCount == Index {
                                return InputTerminalDescriptor;
                            }
                            TerminalCount = TerminalCount.wrapping_add(1);
                        }
                    }
                    CommonDescriptor = (CommonDescriptor as ULONG_PTR)
                        .wrapping_add((*CommonDescriptor).bLength as ULONG_PTR)
                        as PUSB_COMMON_DESCRIPTOR;
                    if CommonDescriptor as ULONG_PTR
                        >= (InterfaceHeaderDescriptor as ULONG_PTR)
                            .wrapping_add(
                                (*InterfaceHeaderDescriptor).wTotalLength as ULONG_PTR,
                            )
                    {
                        break;
                    }
                }
            }
        }
        Descriptor = USBD_ParseConfigurationDescriptorEx(
            ConfigurationDescriptor,
            (Descriptor as ULONG_PTR)
                .wrapping_add((*Descriptor).bLength as ULONG_PTR)
                as PVOID,
            -1i32 as LONG,
            -1i32 as LONG,
            USB_DEVICE_CLASS_AUDIO as LONG,
            -1i32 as LONG,
            -1i32 as LONG,
        );
    }
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn UsbAudioGetNonStreamingTerminalDescriptorByIndex(
    ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
    Index: ULONG,
) -> PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR {
    let mut Descriptor: PUSB_INTERFACE_DESCRIPTOR;
    let mut InterfaceHeaderDescriptor: PUSB_AUDIO_CONTROL_INTERFACE_HEADER_DESCRIPTOR;
    let mut CommonDescriptor: PUSB_COMMON_DESCRIPTOR;
    let mut InputTerminalDescriptor: PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR;
    let mut TerminalCount: ULONG = 0;

    Descriptor = USBD_ParseConfigurationDescriptorEx(
        ConfigurationDescriptor,
        ConfigurationDescriptor as PVOID,
        -1i32 as LONG,
        -1i32 as LONG,
        USB_DEVICE_CLASS_AUDIO as LONG,
        -1i32 as LONG,
        -1i32 as LONG,
    );
    while !Descriptor.is_null() {
        if (*Descriptor).bInterfaceSubClass as i32 == 0x01 {
            InterfaceHeaderDescriptor = USBD_ParseDescriptors(
                ConfigurationDescriptor as PVOID,
                (*ConfigurationDescriptor).wTotalLength as ULONG,
                Descriptor as PVOID,
                USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE as LONG,
            ) as PUSB_AUDIO_CONTROL_INTERFACE_HEADER_DESCRIPTOR;
            if !InterfaceHeaderDescriptor.is_null() {
                CommonDescriptor = USBD_ParseDescriptors(
                    InterfaceHeaderDescriptor as PVOID,
                    (*InterfaceHeaderDescriptor).wTotalLength as ULONG,
                    (InterfaceHeaderDescriptor as ULONG_PTR)
                        .wrapping_add((*InterfaceHeaderDescriptor).bLength as ULONG_PTR)
                        as PVOID,
                    USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE as LONG,
                );
                while !CommonDescriptor.is_null() {
                    InputTerminalDescriptor = CommonDescriptor as PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR;
                    if (*InputTerminalDescriptor).bDescriptorSubtype as i32 == 0x02
                        || (*InputTerminalDescriptor).bDescriptorSubtype as i32 == 0x03
                    {
                        if (*InputTerminalDescriptor).wTerminalType as u32 != USB_AUDIO_STREAMING_TERMINAL_TYPE {
                            if TerminalCount == Index {
                                return InputTerminalDescriptor;
                            }
                            TerminalCount = TerminalCount.wrapping_add(1);
                        }
                    }
                    CommonDescriptor = (CommonDescriptor as ULONG_PTR)
                        .wrapping_add((*CommonDescriptor).bLength as ULONG_PTR)
                        as PUSB_COMMON_DESCRIPTOR;
                    if (CommonDescriptor as ULONG_PTR)
                        >= (InterfaceHeaderDescriptor as ULONG_PTR)
                            .wrapping_add((*InterfaceHeaderDescriptor).wTotalLength as ULONG_PTR)
                    {
                        break;
                    }
                }
            }
        }
        Descriptor = USBD_ParseConfigurationDescriptorEx(
            ConfigurationDescriptor,
            (Descriptor as ULONG_PTR)
                .wrapping_add((*Descriptor).bLength as ULONG_PTR)
                as PVOID,
            -1i32 as LONG,
            -1i32 as LONG,
            USB_DEVICE_CLASS_AUDIO as LONG,
            -1i32 as LONG,
            -1i32 as LONG,
        );
    }
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn UsbAudioGetDataRanges(
    ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
    bTerminalID: UCHAR,
    OutDataRanges: *mut *mut PKSDATARANGE,
    OutDataRangesCount: PULONG,
) {
    let mut StreamingInterfaceDescriptor: PUSB_AUDIO_STREAMING_INTERFACE_DESCRIPTOR;
    let mut StreamingFormatDescriptor: PUSB_AUDIO_STREAMING_FORMAT_TYPE_DESCRIPTOR;
    let mut Descriptor: PUSB_INTERFACE_DESCRIPTOR;
    let mut DataRangeAudio: PKSDATARANGE_AUDIO;
    let mut DataRangeAudioArray: *mut PKSDATARANGE;
    let mut NumFrequency: ULONG;
    let mut DataRangeCount: ULONG;
    let mut DataRangeIndex: ULONG;
    let mut Index: ULONG;

    DataRangeCount = 0;
    Descriptor = USBD_ParseConfigurationDescriptorEx(
        ConfigurationDescriptor,
        ConfigurationDescriptor as PVOID,
        -1i32 as LONG,
        -1i32 as LONG,
        USB_DEVICE_CLASS_AUDIO as LONG,
        -1i32 as LONG,
        -1i32 as LONG,
    );
    while !Descriptor.is_null() {
        if (*Descriptor).bInterfaceSubClass as i32 == 0x02 {
            StreamingInterfaceDescriptor = USBD_ParseDescriptors(
                ConfigurationDescriptor as PVOID,
                (*ConfigurationDescriptor).wTotalLength as ULONG,
                Descriptor as PVOID,
                USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE as LONG,
            ) as PUSB_AUDIO_STREAMING_INTERFACE_DESCRIPTOR;
            if !StreamingInterfaceDescriptor.is_null() {
                if (*StreamingInterfaceDescriptor).bDescriptorSubtype as i32 != 0x01 {
                    RtlAssert(
                        b"StreamingInterfaceDescriptor->bDescriptorSubtype == 0x01\0".as_ptr() as PVOID,
                        b"drivers/usb/usbaudio/filter.c\0".as_ptr() as PVOID,
                        1178u32,
                        core::ptr::null_mut(),
                    );
                }
                if (*StreamingInterfaceDescriptor).wFormatTag as i32 != 0x01 {
                    RtlAssert(
                        b"StreamingInterfaceDescriptor->wFormatTag == WAVE_FORMAT_PCM\0".as_ptr() as PVOID,
                        b"drivers/usb/usbaudio/filter.c\0".as_ptr() as PVOID,
                        1179u32,
                        core::ptr::null_mut(),
                    );
                }
                if (*StreamingInterfaceDescriptor).bTerminalLink as i32 == bTerminalID as i32 {
                    DataRangeCount = DataRangeCount.wrapping_add(1);
                    // DPRINT removed
                }
            }
            Descriptor = StreamingInterfaceDescriptor as PUSB_INTERFACE_DESCRIPTOR;
        }
        Descriptor = USBD_ParseConfigurationDescriptorEx(
            ConfigurationDescriptor,
            (Descriptor as ULONG_PTR).wrapping_add((*Descriptor).bLength as ULONG_PTR) as PVOID,
            -1i32 as LONG,
            -1i32 as LONG,
            USB_DEVICE_CLASS_AUDIO as LONG,
            -1i32 as LONG,
            -1i32 as LONG,
        );
    }

    DataRangeAudioArray = AllocFunction((core::mem::size_of::<PVOID>() as ULONG).wrapping_mul(DataRangeCount)) as *mut PKSDATARANGE;
    if DataRangeAudioArray.is_null() {
        return;
    }

    DataRangeIndex = 0;
    Descriptor = USBD_ParseConfigurationDescriptorEx(
        ConfigurationDescriptor,
        ConfigurationDescriptor as PVOID,
        -1i32 as LONG,
        -1i32 as LONG,
        USB_DEVICE_CLASS_AUDIO as LONG,
        -1i32 as LONG,
        -1i32 as LONG,
    );
    while !Descriptor.is_null() {
        if (*Descriptor).bInterfaceSubClass as i32 == 0x02 {
            StreamingInterfaceDescriptor = USBD_ParseDescriptors(
                ConfigurationDescriptor as PVOID,
                (*ConfigurationDescriptor).wTotalLength as ULONG,
                Descriptor as PVOID,
                USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE as LONG,
            ) as PUSB_AUDIO_STREAMING_INTERFACE_DESCRIPTOR;
            if !StreamingInterfaceDescriptor.is_null() {
                if (*StreamingInterfaceDescriptor).bDescriptorSubtype as i32 != 0x01 {
                    RtlAssert(
                        b"StreamingInterfaceDescriptor->bDescriptorSubtype == 0x01\0".as_ptr() as PVOID,
                        b"drivers/usb/usbaudio/filter.c\0".as_ptr() as PVOID,
                        1207u32,
                        core::ptr::null_mut(),
                    );
                }
                if (*StreamingInterfaceDescriptor).wFormatTag as i32 != 0x01 {
                    RtlAssert(
                        b"StreamingInterfaceDescriptor->wFormatTag == WAVE_FORMAT_PCM\0".as_ptr() as PVOID,
                        b"drivers/usb/usbaudio/filter.c\0".as_ptr() as PVOID,
                        1208u32,
                        core::ptr::null_mut(),
                    );
                }
                if (*StreamingInterfaceDescriptor).bTerminalLink as i32 == bTerminalID as i32 {
                    StreamingFormatDescriptor = (StreamingInterfaceDescriptor as ULONG_PTR).wrapping_add(
                        (*StreamingInterfaceDescriptor).bLength as ULONG_PTR,
                    ) as PUSB_AUDIO_STREAMING_FORMAT_TYPE_DESCRIPTOR;
                    if (*StreamingFormatDescriptor).bDescriptorType as i32 != 0x24 {
                        RtlAssert(
                            b"StreamingFormatDescriptor->bDescriptorType == 0x24\0".as_ptr() as PVOID,
                            b"drivers/usb/usbaudio/filter.c\0".as_ptr() as PVOID,
                            1212u32,
                            core::ptr::null_mut(),
                        );
                    }
                    if (*StreamingFormatDescriptor).bDescriptorSubtype as i32 != 0x02 {
                        RtlAssert(
                            b"StreamingFormatDescriptor->bDescriptorSubtype == 0x02\0".as_ptr() as PVOID,
                            b"drivers/usb/usbaudio/filter.c\0".as_ptr() as PVOID,
                            1213u32,
                            core::ptr::null_mut(),
                        );
                    }
                    if (*StreamingFormatDescriptor).bFormatType as i32 != 0x01 {
                        RtlAssert(
                            b"StreamingFormatDescriptor->bFormatType == 0x01\0".as_ptr() as PVOID,
                            b"drivers/usb/usbaudio/filter.c\0".as_ptr() as PVOID,
                            1214u32,
                            core::ptr::null_mut(),
                        );
                    }

                    DataRangeAudio = AllocFunction(core::mem::size_of::<KSDATARANGE_AUDIO>() as ULONG) as PKSDATARANGE_AUDIO;
                    if DataRangeAudio.is_null() {
                        return;
                    }

                    {
                        let dr = (*DataRangeAudio).DataRange.__bindgen_anon_1.as_mut();
                        (*dr).FormatSize = core::mem::size_of::<KSDATARANGE_AUDIO>() as ULONG;
                        (*dr).MajorFormat = KSDATAFORMAT_TYPE_AUDIO;
                        (*dr).SubFormat = KSDATAFORMAT_SUBTYPE_PCM;
                        (*dr).Specifier = KSDATAFORMAT_SPECIFIER_WAVEFORMATEX;
                    }
                    (*DataRangeAudio).MaximumChannels = (*StreamingFormatDescriptor).bNrChannels as ULONG;
                    (*DataRangeAudio).MinimumBitsPerSample = (*StreamingFormatDescriptor).bBitResolution as ULONG;
                    (*DataRangeAudio).MaximumBitsPerSample = (*StreamingFormatDescriptor).bBitResolution as ULONG;
                    NumFrequency = (*StreamingFormatDescriptor).bSamFreqType as ULONG;
                    (*DataRangeAudio).MinimumSampleFrequency = MAXULONG;
                    (*DataRangeAudio).MaximumSampleFrequency = 0;
                    Index = 0;
                    while Index < NumFrequency {
                        let freq = ((*StreamingFormatDescriptor).tSamFreq[Index.wrapping_mul(3) as usize] as ULONG)
                            | (((*StreamingFormatDescriptor).tSamFreq[Index.wrapping_mul(3).wrapping_add(1) as usize] as ULONG) << 8)
                            | (((*StreamingFormatDescriptor).tSamFreq[Index.wrapping_mul(3).wrapping_add(2) as usize] as ULONG) << 16);
                        if freq < (*DataRangeAudio).MinimumSampleFrequency {
                            (*DataRangeAudio).MinimumSampleFrequency = freq;
                        }
                        if freq > (*DataRangeAudio).MaximumSampleFrequency {
                            (*DataRangeAudio).MaximumSampleFrequency = freq;
                        }
                        Index = Index.wrapping_add(1);
                    }
                    *DataRangeAudioArray.offset(DataRangeIndex as isize) = DataRangeAudio as PKSDATARANGE;
                    DataRangeIndex = DataRangeIndex.wrapping_add(1);
                }
            }
            Descriptor = StreamingInterfaceDescriptor as PUSB_INTERFACE_DESCRIPTOR;
        }
        Descriptor = USBD_ParseConfigurationDescriptorEx(
            ConfigurationDescriptor,
            (Descriptor as ULONG_PTR).wrapping_add((*Descriptor).bLength as ULONG_PTR) as PVOID,
            -1i32 as LONG,
            -1i32 as LONG,
            USB_DEVICE_CLASS_AUDIO as LONG,
            -1i32 as LONG,
            -1i32 as LONG,
        );
    }

    *OutDataRanges = DataRangeAudioArray;
    *OutDataRangesCount = DataRangeCount;
}
#[no_mangle]
pub unsafe extern "C" fn USBAudioPinBuildDescriptors(
    Device: PKSDEVICE,
    PinDescriptors: *mut PKSPIN_DESCRIPTOR_EX,
    PinDescriptorsCount: PULONG,
    PinDescriptorSize: PULONG,
) -> NTSTATUS {
    let DeviceExtension = (*Device).Context as PDEVICE_EXTENSION;
    let mut TotalTerminalDescriptorCount: ULONG = 0;
    let mut NonStreamingTerminalDescriptorCount: ULONG = 0;

    CountTerminalUnits(
        (*DeviceExtension).ConfigurationDescriptor,
        &mut NonStreamingTerminalDescriptorCount,
        &mut TotalTerminalDescriptorCount,
    );

    let Pins = AllocFunction(
        (core::mem::size_of::<KSPIN_DESCRIPTOR_EX>() as ULONG)
            .wrapping_mul(TotalTerminalDescriptorCount),
    ) as PKSPIN_DESCRIPTOR_EX;

    if Pins.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let mut Index: ULONG = 0;
    while Index < TotalTerminalDescriptorCount {
        if Index < TotalTerminalDescriptorCount.wrapping_sub(NonStreamingTerminalDescriptorCount) {
            let TerminalDescriptor = UsbAudioGetStreamingTerminalDescriptorByIndex(
                (*DeviceExtension).ConfigurationDescriptor,
                Index,
            );

            (*Pins.offset(Index as isize)).Dispatch = &mut UsbAudioPinDispatch;
            (*Pins.offset(Index as isize)).PinDescriptor.InterfacesCount = 1;
            (*Pins.offset(Index as isize)).PinDescriptor.Interfaces = &mut StandardPinInterface;
            (*Pins.offset(Index as isize)).PinDescriptor.MediumsCount = 1;
            (*Pins.offset(Index as isize)).PinDescriptor.Mediums = &mut StandardPinMedium;
            (*Pins.offset(Index as isize)).PinDescriptor.Category =
                UsbAudioGetPinCategoryFromTerminalDescriptor(TerminalDescriptor);
            UsbAudioGetDataRanges(
                (*DeviceExtension).ConfigurationDescriptor,
                (*TerminalDescriptor).bTerminalID,
                &mut (*Pins.offset(Index as isize)).PinDescriptor.DataRanges as *mut _ as *mut *mut PKSDATARANGE,
                &mut (*Pins.offset(Index as isize)).PinDescriptor.DataRangesCount,
            );

            if (*TerminalDescriptor).bDescriptorSubtype == USB_AUDIO_OUTPUT_TERMINAL as u8 {
                (*Pins.offset(Index as isize)).PinDescriptor.Communication = KSPIN_COMMUNICATION_BOTH;
                (*Pins.offset(Index as isize)).PinDescriptor.DataFlow = KSPIN_DATAFLOW_OUT;
                (*Pins.offset(Index as isize)).Flags =
                    (KSPIN_FLAG_PROCESS_IN_RUN_STATE_ONLY | KSFILTER_FLAG_CRITICAL_PROCESSING) as ULONG;
            } else if (*TerminalDescriptor).bDescriptorSubtype == USB_AUDIO_INPUT_TERMINAL as u8 {
                (*Pins.offset(Index as isize)).PinDescriptor.Communication = KSPIN_COMMUNICATION_SINK;
                (*Pins.offset(Index as isize)).PinDescriptor.DataFlow = KSPIN_DATAFLOW_IN;
                (*Pins.offset(Index as isize)).Flags =
                    (KSPIN_FLAG_PROCESS_IN_RUN_STATE_ONLY | KSPIN_FLAG_GENERATE_EOS_EVENTS) as ULONG;
            }

            (*Pins.offset(Index as isize)).IntersectHandler = Some(UsbAudioPinDataIntersect);
            (*Pins.offset(Index as isize)).InstancesPossible = 1;
        } else {
            let TerminalDescriptor = UsbAudioGetNonStreamingTerminalDescriptorByIndex(
                (*DeviceExtension).ConfigurationDescriptor,
                Index.wrapping_sub(
                    TotalTerminalDescriptorCount.wrapping_sub(NonStreamingTerminalDescriptorCount),
                ),
            );

            (*Pins.offset(Index as isize)).PinDescriptor.InterfacesCount = 1;
            (*Pins.offset(Index as isize)).PinDescriptor.Interfaces = &mut StandardPinInterface;
            (*Pins.offset(Index as isize)).PinDescriptor.MediumsCount = 1;
            (*Pins.offset(Index as isize)).PinDescriptor.Mediums = &mut StandardPinMedium;
            (*Pins.offset(Index as isize)).PinDescriptor.DataRanges = &mut BridgePinAudioFormats as *mut _ as *mut PKSDATARANGE;
            (*Pins.offset(Index as isize)).PinDescriptor.DataRangesCount = 1;
            (*Pins.offset(Index as isize)).PinDescriptor.Communication = KSPIN_COMMUNICATION_BRIDGE;
            (*Pins.offset(Index as isize)).PinDescriptor.Category =
                UsbAudioGetPinCategoryFromTerminalDescriptor(TerminalDescriptor);

            if (*TerminalDescriptor).bDescriptorSubtype == USB_AUDIO_INPUT_TERMINAL as u8 {
                (*Pins.offset(Index as isize)).PinDescriptor.DataFlow = KSPIN_DATAFLOW_IN;
            } else if (*TerminalDescriptor).bDescriptorSubtype == USB_AUDIO_OUTPUT_TERMINAL as u8 {
                (*Pins.offset(Index as isize)).PinDescriptor.DataFlow = KSPIN_DATAFLOW_OUT;
            }
        }

        Index = Index.wrapping_add(1);
    }

    *PinDescriptors = Pins;
    *PinDescriptorSize = core::mem::size_of::<KSPIN_DESCRIPTOR_EX>() as ULONG;
    *PinDescriptorsCount = TotalTerminalDescriptorCount;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn USBAudioGetDescriptor(
    DeviceObject: PDEVICE_OBJECT,
    DescriptorType: UCHAR,
    DescriptorLength: ULONG,
    DescriptorIndex: UCHAR,
    LanguageId: LANGID,
    OutDescriptor: *mut PVOID,
) -> NTSTATUS {
    let mut Urb: PURB;
    let Status: NTSTATUS;
    let Descriptor: PVOID;

    Descriptor = AllocFunction(DescriptorLength);
    if Descriptor.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Urb = AllocFunction(core::mem::size_of::<URB>() as ULONG) as PURB;
    if Urb.is_null() {
        FreeFunction(Descriptor);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // Inline UsbBuildGetDescriptorRequest macro
    {
        let hdr = (*Urb).__bindgen_anon_1.UrbHeader.as_mut();
        (*hdr).Function = URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE as USHORT;
        (*hdr).Length = core::mem::size_of::<URB_CONTROL_DESCRIPTOR_REQUEST>() as USHORT;
    }
    {
        let req = (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut();
        (*req).TransferBufferLength = DescriptorLength;
        (*req).TransferBufferMDL = core::ptr::null_mut();
        (*req).TransferBuffer = Descriptor;
        (*req).DescriptorType = DescriptorType;
        (*req).Index = DescriptorIndex;
        (*req).LanguageId = LanguageId;
        (*req).UrbLink = core::ptr::null_mut();
    }

    Status = SubmitUrbSync(DeviceObject, Urb);

    FreeFunction(Urb as PVOID);

    if NT_SUCCESS(Status) {
        *OutDescriptor = Descriptor;
    } else {
        FreeFunction(Descriptor);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn USBAudioGetStringDescriptor(
    DeviceObject: PDEVICE_OBJECT,
    DescriptorLength: ULONG,
    DescriptorIndex: UCHAR,
    LanguageId: LANGID,
    OutDescriptor: *mut PVOID,
) -> NTSTATUS {
    let Status = USBAudioGetDescriptor(
        DeviceObject,
        USB_STRING_DESCRIPTOR_TYPE as UCHAR,
        DescriptorLength,
        DescriptorIndex,
        LanguageId,
        OutDescriptor,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn USBAudioRegCreateMediaCategoriesKey(
    Name: PUNICODE_STRING,
    OutHandle: PHANDLE,
) -> NTSTATUS {
    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut destination_string: _UNICODE_STRING = core::mem::zeroed();
    let mut handle: HANDLE = core::ptr::null_mut();

    RtlInitUnicodeString(
        &mut destination_string,
        b"\\Registry\\Machine\\SYSTEM\\CurrentControlSet\\Control\\MediaCategories\\\0"
            .as_ptr() as PCWSTR,
    );

    object_attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    object_attributes.RootDirectory = core::ptr::null_mut();
    object_attributes.ObjectName = &mut destination_string;
    object_attributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_OPENIF | OBJ_KERNEL_HANDLE;
    object_attributes.SecurityDescriptor = core::ptr::null_mut();
    object_attributes.SecurityQualityOfService = core::ptr::null_mut();

    let mut status = ZwOpenKey(&mut handle, KEY_ALL_ACCESS, &mut object_attributes);
    if NT_SUCCESS(status) {
        object_attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        object_attributes.RootDirectory = handle;
        object_attributes.ObjectName = Name;
        object_attributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
        object_attributes.SecurityDescriptor = core::ptr::null_mut();
        object_attributes.SecurityQualityOfService = core::ptr::null_mut();

        status = ZwCreateKey(
            OutHandle,
            KEY_ALL_ACCESS,
            &mut object_attributes,
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
        );
        ZwClose(handle);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn USBAudioInitComponentId(
    Device: PKSDEVICE,
    ComponentId: PKSCOMPONENTID,
) -> NTSTATUS {
    let DeviceExtension: PDEVICE_EXTENSION = (*Device).Context as PDEVICE_EXTENSION;
    let mut DescriptionBuffer: LPWSTR = core::ptr::null_mut();
    let mut GuidString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Name: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut hKey: HANDLE = core::ptr::null_mut();
    let mut TempGuid: GUID = GUID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    };

    (*ComponentId).Component = KSCOMPONENTID_USBAUDIO;
    (*ComponentId).Version = HIBYTE((*(*DeviceExtension).DeviceDescriptor).bcdDevice) as ULONG;
    (*ComponentId).Revision = LOBYTE((*(*DeviceExtension).DeviceDescriptor).bcdDevice) as ULONG;

    INIT_USBAUDIO_MID(&mut (*ComponentId).Manufacturer, (*(*DeviceExtension).DeviceDescriptor).idVendor);
    INIT_USBAUDIO_PID(&mut (*ComponentId).Product, (*(*DeviceExtension).DeviceDescriptor).idProduct);
    INIT_USBAUDIO_PRODUCT_NAME(&mut TempGuid, (*(*DeviceExtension).DeviceDescriptor).idVendor, (*(*DeviceExtension).DeviceDescriptor).idProduct, 0);

    if (*(*DeviceExtension).DeviceDescriptor).iProduct != 0 {
        let Status = USBAudioGetStringDescriptor(
            (*DeviceExtension).LowerDevice,
            (100 * core::mem::size_of::<WCHAR>()) as ULONG,
            (*(*DeviceExtension).DeviceDescriptor).iProduct,
            0x0409,
            &mut DescriptionBuffer as *mut LPWSTR as *mut PVOID,
        );
        if NT_SUCCESS(Status) {
            let Status = RtlStringFromGUID(&TempGuid, &mut GuidString);
            if NT_SUCCESS(Status) {
                let Status = USBAudioRegCreateMediaCategoriesKey(&mut GuidString, &mut hKey);
                if NT_SUCCESS(Status) {
                    RtlInitUnicodeString(&mut Name, w!("Name").as_ptr() as PCWSTR);
                    ZwSetValueKey(
                        hKey,
                        &mut Name,
                        0,
                        REG_SZ as ULONG,
                        DescriptionBuffer as PVOID,
                        ((wcslen(DescriptionBuffer as *const u16) + 1) * core::mem::size_of::<WCHAR>()) as ULONG,
                    );
                    ZwClose(hKey);

                    INIT_USBAUDIO_PRODUCT_NAME(&mut (*ComponentId).Name, (*(*DeviceExtension).DeviceDescriptor).idVendor, (*(*DeviceExtension).DeviceDescriptor).idProduct, 0);
                }
                RtlFreeUnicodeString(&mut GuidString);
            }
            FreeFunction(DescriptionBuffer as PVOID);
        }
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn USBAudioCreateFilterContext(
    Device: PKSDEVICE,
) -> NTSTATUS {
    let FilterDescriptor: PKSFILTER_DESCRIPTOR =
        AllocFunction(core::mem::size_of::<KSFILTER_DESCRIPTOR>() as ULONG)
            as PKSFILTER_DESCRIPTOR;
    if FilterDescriptor.is_null() {
        return USBD_STATUS_INSUFFICIENT_RESOURCES as NTSTATUS;
    }

    (*FilterDescriptor).Version = KSFILTER_DESCRIPTOR_VERSION;
    (*FilterDescriptor).Flags = 0;
    (*FilterDescriptor).ReferenceGuid = &KSNAME_Filter;
    (*FilterDescriptor).Dispatch = &mut USBAudioFilterDispatch;
    (*FilterDescriptor).CategoriesCount = 1;
    (*FilterDescriptor).Categories = &mut GUID_KSCATEGORY_AUDIO;

    let ComponentId: PKSCOMPONENTID =
        AllocFunction(core::mem::size_of::<KSCOMPONENTID>() as ULONG) as PKSCOMPONENTID;
    if ComponentId.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let mut Status = USBAudioInitComponentId(Device, ComponentId);
    if !NT_SUCCESS(Status) {
        FreeFunction(ComponentId as PVOID);
        return Status;
    }
    (*FilterDescriptor).ComponentId = ComponentId;

    Status = USBAudioPinBuildDescriptors(
        Device,
        &mut (*FilterDescriptor).PinDescriptors as *mut *const KSPIN_DESCRIPTOR_EX as *mut PKSPIN_DESCRIPTOR_EX,
        &mut (*FilterDescriptor).PinDescriptorsCount,
        &mut (*FilterDescriptor).PinDescriptorSize,
    );
    if !NT_SUCCESS(Status) {
        FreeFunction(ComponentId as PVOID);
        return Status;
    }

    Status = BuildUSBAudioFilterTopology(Device, FilterDescriptor);
    if !NT_SUCCESS(Status) {
        FreeFunction(ComponentId as PVOID);
        return Status;
    }

    Status = KsCreateFilterFactory(
        (*Device).FunctionalDeviceObject,
        FilterDescriptor,
        ReferenceString,
        core::ptr::null_mut(),
        KSCREATE_ITEM_FREEONSTOP,
        None,
        None,
        core::ptr::null_mut(),
    );

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * PROJECT:     ReactOS Universal Audio Class Driver
// * LICENSE:     GPL - See COPYING in the top level directory
// * FILE:        drivers/usb/usbaudio/filter.c
// * PURPOSE:     USB Audio device driver.
// * PROGRAMMERS:
// *              Johannes Anderwald (johannes.anderwald@reactos.org)
// */
//
// #include "usbaudio.h"
//
// GUID NodeTypeMicrophone = { STATIC_KSNODETYPE_MICROPHONE };
// GUID NodeTypeDesktopMicrophone = { STATIC_KSNODETYPE_DESKTOP_MICROPHONE };
// GUID NodeTypePersonalMicrophone = { STATIC_KSNODETYPE_PERSONAL_MICROPHONE };
// GUID NodeTypeOmmniMicrophone = { STATIC_KSNODETYPE_OMNI_DIRECTIONAL_MICROPHONE };
// GUID NodeTypeArrayMicrophone = { STATIC_KSNODETYPE_MICROPHONE_ARRAY };
// GUID NodeTypeProcessingArrayMicrophone = { STATIC_KSNODETYPE_PROCESSING_MICROPHONE_ARRAY };
// GUID NodeTypeSpeaker = { STATIC_KSNODETYPE_SPEAKER };
// GUID NodeTypeHeadphonesSpeaker = { STATIC_KSNODETYPE_HEADPHONES };
// GUID NodeTypeHMDA = { STATIC_KSNODETYPE_HEAD_MOUNTED_DISPLAY_AUDIO };
// GUID NodeTypeDesktopSpeaker = { STATIC_KSNODETYPE_DESKTOP_SPEAKER };
// GUID NodeTypeRoomSpeaker = { STATIC_KSNODETYPE_ROOM_SPEAKER };
// GUID NodeTypeCommunicationSpeaker = { STATIC_KSNODETYPE_COMMUNICATION_SPEAKER };
// GUID NodeTypeSubwoofer = { STATIC_KSNODETYPE_LOW_FREQUENCY_EFFECTS_SPEAKER };
// GUID NodeTypeCapture = { STATIC_PINNAME_CAPTURE };
// GUID NodeTypePlayback = { STATIC_KSCATEGORY_AUDIO };
// GUID GUID_KSCATEGORY_AUDIO = { STATIC_KSCATEGORY_AUDIO };
//
// KSPIN_INTERFACE StandardPinInterface =
// {
//      {STATIC_KSINTERFACESETID_Standard},
//      KSINTERFACE_STANDARD_STREAMING,
//      0
// };
//
// KSPIN_MEDIUM StandardPinMedium =
// {
//      {STATIC_KSMEDIUMSETID_Standard},
//      KSMEDIUM_TYPE_ANYINSTANCE,
//      0
// };
//
// KSDATARANGE BridgePinAudioFormat[] =
// {
//     {
//         {
//             sizeof(KSDATAFORMAT),
//             0,
//             0,
//             0,
//             {STATIC_KSDATAFORMAT_TYPE_AUDIO},
//             {STATIC_KSDATAFORMAT_SUBTYPE_ANALOG},
//             {STATIC_KSDATAFORMAT_SPECIFIER_NONE}
//         }
//     }
// };
//
// static PKSDATARANGE BridgePinAudioFormats[] =
// {
//     &BridgePinAudioFormat[0]
// };
//
// static LPWSTR ReferenceString = L"global";
//
// NTSTATUS
// NTAPI
// USBAudioFilterCreate(
//     PKSFILTER Filter,
//     PIRP Irp);
//
// static KSFILTER_DISPATCH USBAudioFilterDispatch =
// {
//     USBAudioFilterCreate,
//     NULL,
//     NULL,
//     NULL
// };
//
// static KSPIN_DISPATCH UsbAudioPinDispatch =
// {
//     USBAudioPinCreate,
//     USBAudioPinClose,
//     USBAudioPinProcess,
//     USBAudioPinReset,
//     USBAudioPinSetDataFormat,
//     USBAudioPinSetDeviceState,
//     NULL,
//     NULL,
//     NULL,
//     NULL
// };
//
// NTSTATUS NTAPI FilterAudioVolumeHandler(IN PIRP Irp, IN PKSIDENTIFIER  Request, IN OUT PVOID  Data);
// NTSTATUS NTAPI FilterAudioMuteHandler(IN PIRP Irp, IN PKSIDENTIFIER  Request, IN OUT PVOID  Data);
//
// DEFINE_KSPROPERTY_TABLE_AUDIO_VOLUME(FilterAudioVolumePropertySet, FilterAudioVolumeHandler);
// DEFINE_KSPROPERTY_TABLE_AUDIO_MUTE(FilterAudioMutePropertySet, FilterAudioMuteHandler);
//
//
// static KSPROPERTY_SET FilterAudioVolumePropertySetArray[] =
// {
//     {
//         &KSPROPSETID_Audio,
//         sizeof(FilterAudioVolumePropertySet) / sizeof(KSPROPERTY_ITEM),
//         (const KSPROPERTY_ITEM*)&FilterAudioVolumePropertySet,
//         0,
//         NULL
//     }
// };
//
// static KSPROPERTY_SET FilterAudioMutePropertySetArray[] =
// {
//     {
//         &KSPROPSETID_Audio,
//         sizeof(FilterAudioMutePropertySet) / sizeof(KSPROPERTY_ITEM),
//         (const KSPROPERTY_ITEM*)&FilterAudioMutePropertySet,
//         0,
//         NULL
//     }
// };
//
// NTSTATUS
// UsbAudioGetSetProperty(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN UCHAR Request,
//     IN USHORT Value,
//     IN USHORT Index,
//     IN PVOID TransferBuffer,
//     IN ULONG TransferBufferLength,
//     IN ULONG TransferFlags)
// {
//     PURB Urb;
//     NTSTATUS Status;
//
//     /* allocate urb */
//     Urb = AllocFunction(sizeof(struct _URB_CONTROL_VENDOR_OR_CLASS_REQUEST));
//     if (!Urb)
//     {
//         /* no memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* format urb */
//     UsbBuildVendorRequest(Urb,
//         URB_FUNCTION_CLASS_INTERFACE,
//         sizeof(struct _URB_CONTROL_VENDOR_OR_CLASS_REQUEST),
//         TransferFlags,
//         0,
//         Request,
//         Value,
//         Index,
//         TransferBuffer,
//         NULL,
//         TransferBufferLength,
//         NULL);
//
//     /* submit urb */
//     Status = SubmitUrbSync(DeviceObject, Urb);
//
//     FreeFunction(Urb);
//     return Status;
// }
//
// PNODE_CONTEXT
// FindNodeContextWithNode(
//     IN PNODE_CONTEXT NodeContext,
//     IN ULONG NodeContextCount,
//     IN ULONG NodeId)
// {
//     ULONG Index, NodeIndex;
//     for (Index = 0; Index < NodeContextCount; Index++)
//     {
//         for (NodeIndex = 0; NodeIndex < NodeContext[Index].NodeCount; NodeIndex++)
//         {
//             if (NodeContext[Index].Nodes[NodeIndex] == NodeId)
//             {
//                 return &NodeContext[Index];
//             }
//         }
//     }
//     return NULL;
// }
//
//
// NTSTATUS
// NTAPI
// FilterAudioMuteHandler(
//     IN PIRP Irp,
//     IN PKSIDENTIFIER  Request,
//     IN OUT PVOID  Data)
// {
//     PKSNODEPROPERTY_AUDIO_CHANNEL Property;
//     PKSFILTER Filter;
//     PFILTER_CONTEXT FilterContext;
//     PNODE_CONTEXT NodeContext;
//     PUSB_AUDIO_CONTROL_FEATURE_UNIT_DESCRIPTOR FeatureUnitDescriptor;
//     NTSTATUS Status = STATUS_INVALID_PARAMETER;
//
//     /* get filter from irp */
//     Filter = KsGetFilterFromIrp(Irp);
//
//     if (Filter)
//     {
//         /* get property */
//         Property = (PKSNODEPROPERTY_AUDIO_CHANNEL)Request;
//
//         /* get filter context */
//         FilterContext = (PFILTER_CONTEXT)Filter->Context;
//
//         /* search for node context */
//         NodeContext = FindNodeContextWithNode(FilterContext->DeviceExtension->NodeContext, FilterContext->DeviceExtension->NodeContextCount, Property->NodeProperty.NodeId);
//         if (NodeContext)
//         {
//             FeatureUnitDescriptor = (PUSB_AUDIO_CONTROL_FEATURE_UNIT_DESCRIPTOR)NodeContext->Descriptor;
//             if (Property->NodeProperty.Property.Flags & KSPROPERTY_TYPE_GET)
//             {
//                 Status = UsbAudioGetSetProperty(FilterContext->DeviceExtension->LowerDevice, 0x81, 0x1 << 8, FeatureUnitDescriptor->bUnitID << 8, Data, 1, USBD_TRANSFER_DIRECTION_IN);
//                 Irp->IoStatus.Information = sizeof(BOOL);
//             }
//             else
//             {
//                 Status = UsbAudioGetSetProperty(FilterContext->DeviceExtension->LowerDevice, 0x01, 0x1 << 8, FeatureUnitDescriptor->bUnitID << 8, Data, 1, USBD_TRANSFER_DIRECTION_OUT);
//             }
//         }
//     }
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// FilterAudioVolumeHandler(
//     IN PIRP Irp,
//     IN PKSIDENTIFIER  Request,
//     IN OUT PVOID  Data)
// {
//     PKSNODEPROPERTY_AUDIO_CHANNEL Property;
//     PKSFILTER Filter;
//     PFILTER_CONTEXT FilterContext;
//     PNODE_CONTEXT NodeContext;
//     PUSB_AUDIO_CONTROL_FEATURE_UNIT_DESCRIPTOR FeatureUnitDescriptor;
//     PSHORT TransferBuffer;
//     LONG Value;
//     NTSTATUS Status = STATUS_INVALID_PARAMETER;
//
//
//     /* get filter from irp */
//     Filter = KsGetFilterFromIrp(Irp);
//
//     if (Filter)
//     {
//         /* get property */
//         Property = (PKSNODEPROPERTY_AUDIO_CHANNEL)Request;
//
//         /* get filter context */
//         FilterContext = (PFILTER_CONTEXT)Filter->Context;
//
//         TransferBuffer = AllocFunction(sizeof(USHORT) * 3);
//         ASSERT(TransferBuffer);
//
//         Value = *(PLONG)Data;
//
//         /* search for node context */
//         NodeContext = FindNodeContextWithNode(FilterContext->DeviceExtension->NodeContext, FilterContext->DeviceExtension->NodeContextCount, Property->NodeProperty.NodeId);
//         if (NodeContext)
//         {
//             FeatureUnitDescriptor = (PUSB_AUDIO_CONTROL_FEATURE_UNIT_DESCRIPTOR)NodeContext->Descriptor;
//             if (Property->NodeProperty.Property.Flags & KSPROPERTY_TYPE_GET)
//             {
//                 Status = UsbAudioGetSetProperty(FilterContext->DeviceExtension->LowerDevice, 0x81, 0x2 << 8, FeatureUnitDescriptor->bUnitID << 8, &TransferBuffer[0], sizeof(USHORT), USBD_TRANSFER_DIRECTION_IN);
//                 Value = (LONG)TransferBuffer[0] * 256;
//
//                 *(PLONG)Data = Value;
//                 Irp->IoStatus.Information = sizeof(BOOL);
//             }
//             else
//             {
//                 /* downscale value */
//                 Value /= 256;
//
//                 /* get minimum value */
//                 UsbAudioGetSetProperty(FilterContext->DeviceExtension->LowerDevice, 0x82, 0x2 << 8, FeatureUnitDescriptor->bUnitID << 8, &TransferBuffer[0], sizeof(USHORT), USBD_TRANSFER_DIRECTION_IN);
//
//                 /* get maximum value */
//                 UsbAudioGetSetProperty(FilterContext->DeviceExtension->LowerDevice, 0x83, 0x2 << 8, FeatureUnitDescriptor->bUnitID << 8, &TransferBuffer[1], sizeof(USHORT), USBD_TRANSFER_DIRECTION_IN);
//
//                 if (TransferBuffer[0] > Value)
//                 {
//                     /* use minimum value */
//                     Value = TransferBuffer[0];
//                 }
//
//                 if (TransferBuffer[1] < Value)
//                 {
//                     /* use maximum value */
//                     Value = TransferBuffer[1];
//                 }
//
//                 /* store value */
//                 TransferBuffer[2] = Value;
//
//                 /* set volume request */
//                 Status = UsbAudioGetSetProperty(FilterContext->DeviceExtension->LowerDevice, 0x01, 0x2 << 8, FeatureUnitDescriptor->bUnitID << 8, &TransferBuffer[2], sizeof(USHORT), USBD_TRANSFER_DIRECTION_OUT);
//                 if (NT_SUCCESS(Status))
//                 {
//                     /* store number of bytes transferred*/
//                     Irp->IoStatus.Information = sizeof(LONG);
//                 }
//             }
//         }
//
//         /* free transfer buffer */
//         FreeFunction(TransferBuffer);
//     }
//     return Status;
// }
//
//
// ULONG
// CountTopologyComponents(
//     IN PUSB_CONFIGURATION_DESCRIPTOR ConfigurationDescriptor,
//     OUT PULONG OutDescriptorCount)
// {
//     PUSB_INTERFACE_DESCRIPTOR Descriptor;
//     PUSB_AUDIO_CONTROL_INTERFACE_HEADER_DESCRIPTOR InterfaceHeaderDescriptor;
//     PUSB_COMMON_DESCRIPTOR CommonDescriptor;
//     PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR InputTerminalDescriptor;
//     PUSB_AUDIO_CONTROL_FEATURE_UNIT_DESCRIPTOR FeatureUnitDescriptor;
//     PUSB_AUDIO_CONTROL_MIXER_UNIT_DESCRIPTOR MixerUnitDescriptor;
//     ULONG NodeCount = 0, Length, Index;
//     ULONG DescriptorCount = 0;
//     UCHAR Value;
//
//     for (Descriptor = USBD_ParseConfigurationDescriptorEx(ConfigurationDescriptor, ConfigurationDescriptor, -1, -1, USB_DEVICE_CLASS_AUDIO, -1, -1);
//     Descriptor != NULL;
//         Descriptor = USBD_ParseConfigurationDescriptorEx(ConfigurationDescriptor, (PVOID)((ULONG_PTR)Descriptor + Descriptor->bLength), -1, -1, USB_DEVICE_CLASS_AUDIO, -1, -1))
//     {
//         if (Descriptor->bInterfaceSubClass == 0x01) /* AUDIO_CONTROL */
//         {
//             InterfaceHeaderDescriptor = (PUSB_AUDIO_CONTROL_INTERFACE_HEADER_DESCRIPTOR)USBD_ParseDescriptors(ConfigurationDescriptor, ConfigurationDescriptor->wTotalLength, Descriptor, USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE);
//             if (InterfaceHeaderDescriptor != NULL)
//             {
//                 CommonDescriptor = USBD_ParseDescriptors(InterfaceHeaderDescriptor, InterfaceHeaderDescriptor->wTotalLength, (PVOID)((ULONG_PTR)InterfaceHeaderDescriptor + InterfaceHeaderDescriptor->bLength), USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE);
//                 while (CommonDescriptor)
//                 {
//                     InputTerminalDescriptor = (PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR)CommonDescriptor;
//                     if (InputTerminalDescriptor->bDescriptorSubtype == 0x02 /* INPUT TERMINAL*/ || InputTerminalDescriptor->bDescriptorSubtype == 0x03 /* OUTPUT_TERMINAL*/)
//                     {
//                         NodeCount++;
//                         DescriptorCount++;
//                     }
//                     else if (InputTerminalDescriptor->bDescriptorSubtype == 0x06 /* FEATURE_UNIT*/)
//                     {
//                         FeatureUnitDescriptor = (PUSB_AUDIO_CONTROL_FEATURE_UNIT_DESCRIPTOR)InputTerminalDescriptor;
//                         DescriptorCount++;
//
//                         /* get controls from all channels*/
//                         Value = 0;
//                         Length = FeatureUnitDescriptor->bLength - 7;
//                         for (Index = 0; Index < Length; Index++)
//                         {
//                             Value |= FeatureUnitDescriptor->bmaControls[Index];
//                         }
//
//                         if (Value & 0x01) /* MUTE*/
//                             NodeCount++;
//                         if (Value & 0x02) /* VOLUME */
//                             NodeCount++;
//                         if (Value & 0x04) /* BASS */
//                             NodeCount++;
//                         if (Value & 0x08) /* MID */
//                             NodeCount++;
//                         if (Value & 0x10) /* TREBLE */
//                             NodeCount++;
//                         if (Value & 0x20) /* GRAPHIC EQUALIZER */
//                             NodeCount++;
//                         if (Value & 0x40) /* AUTOMATIC GAIN */
//                             NodeCount++;
//                         if (Value & 0x80) /* DELAY */
//                             NodeCount++;
//                     }
//                     else if (InputTerminalDescriptor->bDescriptorSubtype == 0x04 /* MIXER_UNIT */)
//                     {
//                         MixerUnitDescriptor = (PUSB_AUDIO_CONTROL_MIXER_UNIT_DESCRIPTOR)InputTerminalDescriptor;
//                         DescriptorCount++;
//                         NodeCount += MixerUnitDescriptor->bNrInPins + 1; /* KSNODETYPE_SUPERMIX for each source pin and KSNODETYPE_SUM for target */
//                     }
//                     else if (InputTerminalDescriptor->bDescriptorSubtype == 0x05 /* SELECTOR_UNIT */)
//                     {
//                         DescriptorCount++;
//                         NodeCount++;
//                     }
//                     else
//                     {
//                         UNIMPLEMENTED;
//                     }
//                     CommonDescriptor = (PUSB_COMMON_DESCRIPTOR)((ULONG_PTR)CommonDescriptor + CommonDescriptor->bLength);
//                     if ((ULONG_PTR)CommonDescriptor >= ((ULONG_PTR)InterfaceHeaderDescriptor + InterfaceHeaderDescriptor->wTotalLength))
//                         break;
//                 }
//             }
//         }
//     }
//     *OutDescriptorCount = DescriptorCount;
//     return NodeCount;
// }
//
// PNODE_CONTEXT
// FindNodeContextWithId(
//     IN PNODE_CONTEXT NodeContext,
//     IN ULONG NodeContextCount,
//     IN UCHAR TerminalId)
// {
//     ULONG Index;
//     PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR TerminalDescriptor;
//
//     for (Index = 0; Index < NodeContextCount; Index++)
//     {
//         TerminalDescriptor = (PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR)NodeContext[Index].Descriptor;
//         if (TerminalDescriptor->bTerminalID == TerminalId)
//             return &NodeContext[Index];
//     }
//     return NULL;
// }
//
// NTSTATUS
// BuildUSBAudioFilterTopology(
//     PKSDEVICE Device,
//     PKSFILTER_DESCRIPTOR FilterDescriptor)
// {
//     PDEVICE_EXTENSION DeviceExtension;
//     ULONG NodeCount, Index, DescriptorCount, StreamingTerminalIndex, NonStreamingTerminalDescriptorCount, TotalTerminalDescriptorCount, StreamingTerminalPinOffset, ControlDescriptorCount, Length;
//     UCHAR Value;
//     PUSB_INTERFACE_DESCRIPTOR Descriptor;
//     PUSB_AUDIO_CONTROL_INTERFACE_HEADER_DESCRIPTOR InterfaceHeaderDescriptor;
//     PUSB_COMMON_DESCRIPTOR CommonDescriptor;
//     PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR InputTerminalDescriptor;
//     PUSB_AUDIO_CONTROL_FEATURE_UNIT_DESCRIPTOR FeatureUnitDescriptor;
//     PUSB_AUDIO_CONTROL_MIXER_UNIT_DESCRIPTOR MixerUnitDescriptor;
//     PUSB_AUDIO_CONTROL_OUTPUT_TERMINAL_DESCRIPTOR OutputTerminalDescriptor;
//     PUSB_AUDIO_CONTROL_SELECTOR_UNIT_DESCRIPTOR SelectorUnitDescriptor;
//     PKSNODE_DESCRIPTOR NodeDescriptors;
//     PNODE_CONTEXT NodeContext, PreviousNodeContext;
//     PKSTOPOLOGY_CONNECTION Connections;
//     PKSAUTOMATION_TABLE AutomationTable;
//
//     /* get device extension */
//     DeviceExtension = Device->Context;
//
//     /* count topology nodes */
//     NodeCount = CountTopologyComponents(DeviceExtension->ConfigurationDescriptor, &ControlDescriptorCount);
//
//     /* init node descriptors*/
//     FilterDescriptor->NodeDescriptors = NodeDescriptors = AllocFunction(NodeCount * sizeof(KSNODE_DESCRIPTOR));
//     if (FilterDescriptor->NodeDescriptors == NULL)
//     {
//         /* no memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     FilterDescriptor->NodeDescriptorSize = sizeof(KSNODE_DESCRIPTOR);
//
//     DeviceExtension->NodeContext = NodeContext = AllocFunction(sizeof(NODE_CONTEXT) * ControlDescriptorCount);
//     if (!NodeContext)
//     {
//         /* no memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     DeviceExtension->NodeContextCount = ControlDescriptorCount;
//     DescriptorCount = 0;
//
//     /* first enumerate all topology nodes */
//     for (Descriptor = USBD_ParseConfigurationDescriptorEx(DeviceExtension->ConfigurationDescriptor, DeviceExtension->ConfigurationDescriptor, -1, -1, USB_DEVICE_CLASS_AUDIO, -1, -1);
//     Descriptor != NULL;
//         Descriptor = USBD_ParseConfigurationDescriptorEx(DeviceExtension->ConfigurationDescriptor, (PVOID)((ULONG_PTR)Descriptor + Descriptor->bLength), -1, -1, USB_DEVICE_CLASS_AUDIO, -1, -1))
//     {
//         if (Descriptor->bInterfaceSubClass == 0x01) /* AUDIO_CONTROL */
//         {
//             InterfaceHeaderDescriptor = (PUSB_AUDIO_CONTROL_INTERFACE_HEADER_DESCRIPTOR)USBD_ParseDescriptors(DeviceExtension->ConfigurationDescriptor, DeviceExtension->ConfigurationDescriptor->wTotalLength, Descriptor, USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE);
//             if (InterfaceHeaderDescriptor != NULL)
//             {
//                 CommonDescriptor = USBD_ParseDescriptors(InterfaceHeaderDescriptor, InterfaceHeaderDescriptor->wTotalLength, (PVOID)((ULONG_PTR)InterfaceHeaderDescriptor + InterfaceHeaderDescriptor->bLength), USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE);
//                 while (CommonDescriptor)
//                 {
//                     InputTerminalDescriptor = (PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR)CommonDescriptor;
//                     if (InputTerminalDescriptor->bDescriptorSubtype == 0x02 /* INPUT TERMINAL*/)
//                     {
//                         if (InputTerminalDescriptor->wTerminalType == USB_AUDIO_STREAMING_TERMINAL_TYPE)
//                         {
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Type = &KSNODETYPE_SRC;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Name = &KSNODETYPE_SRC;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].AutomationTable = AllocFunction(sizeof(KSAUTOMATION_TABLE));
//
//                             /* insert into node context*/
//                             NodeContext[DescriptorCount].Descriptor = CommonDescriptor;
//                             NodeContext[DescriptorCount].NodeCount = 1;
//                             NodeContext[DescriptorCount].Nodes[0] = FilterDescriptor->NodeDescriptorsCount;
//                             DescriptorCount++;
//
//                             FilterDescriptor->NodeDescriptorsCount++;
//                         }
//                         else if ((InputTerminalDescriptor->wTerminalType & 0xFF00) == 0x200)
//                         {
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Type = &KSNODETYPE_ADC;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Name = &KSNODETYPE_ADC;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].AutomationTable = AllocFunction(sizeof(KSAUTOMATION_TABLE));
//
//                             /* insert into node context*/
//                             NodeContext[DescriptorCount].Descriptor = CommonDescriptor;
//                             NodeContext[DescriptorCount].NodeCount = 1;
//                             NodeContext[DescriptorCount].Nodes[0] = FilterDescriptor->NodeDescriptorsCount;
//                             DescriptorCount++;
//
//
//                             FilterDescriptor->NodeDescriptorsCount++;
//                         }
//                         else if ((InputTerminalDescriptor->wTerminalType & 0xFF00) == 0x300)
//                         {
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Type = &KSNODETYPE_DAC;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Name = &KSNODETYPE_DAC;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].AutomationTable = AllocFunction(sizeof(KSAUTOMATION_TABLE));
//
//                             /* insert into node context*/
//                             NodeContext[DescriptorCount].Descriptor = CommonDescriptor;
//                             NodeContext[DescriptorCount].NodeCount = 1;
//                             NodeContext[DescriptorCount].Nodes[0] = FilterDescriptor->NodeDescriptorsCount;
//                             DescriptorCount++;
//
//                             FilterDescriptor->NodeDescriptorsCount++;
//                         }
//                         else
//                         {
//                             DPRINT1("Unexpected input terminal type %x\n", InputTerminalDescriptor->wTerminalType);
//                         }
//                     }
//                     else if (InputTerminalDescriptor->bDescriptorSubtype == 0x03 /* OUTPUT_TERMINAL*/)
//                     {
//                         if (InputTerminalDescriptor->wTerminalType == USB_AUDIO_STREAMING_TERMINAL_TYPE)
//                         {
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Type = &KSNODETYPE_SRC;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Name = &KSNODETYPE_SRC;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].AutomationTable = AllocFunction(sizeof(KSAUTOMATION_TABLE));
//
//                             /* insert into node context*/
//                             NodeContext[DescriptorCount].Descriptor = CommonDescriptor;
//                             NodeContext[DescriptorCount].NodeCount = 1;
//                             NodeContext[DescriptorCount].Nodes[0] = FilterDescriptor->NodeDescriptorsCount;
//                             DescriptorCount++;
//
//                             FilterDescriptor->NodeDescriptorsCount++;
//                         }
//                         else if ((InputTerminalDescriptor->wTerminalType & 0xFF00) == 0x300)
//                         {
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Type = &KSNODETYPE_DAC;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Name = &KSNODETYPE_DAC;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].AutomationTable = AllocFunction(sizeof(KSAUTOMATION_TABLE));
//
//                             /* insert into node context*/
//                             NodeContext[DescriptorCount].Descriptor = CommonDescriptor;
//                             NodeContext[DescriptorCount].NodeCount = 1;
//                             NodeContext[DescriptorCount].Nodes[0] = FilterDescriptor->NodeDescriptorsCount;
//                             DescriptorCount++;
//
//                             FilterDescriptor->NodeDescriptorsCount++;
//                         }
//                         else
//                         {
//                             DPRINT1("Unexpected output terminal type %x\n", InputTerminalDescriptor->wTerminalType);
//                         }
//                     }
//
//                     else if (InputTerminalDescriptor->bDescriptorSubtype == 0x06 /* FEATURE_UNIT*/)
//                     {
//                         FeatureUnitDescriptor = (PUSB_AUDIO_CONTROL_FEATURE_UNIT_DESCRIPTOR)CommonDescriptor;
//
//                         /* get controls from all channels*/
//                         Value = 0;
//                         Length = FeatureUnitDescriptor->bLength - 7;
//                         for (Index = 0; Index < Length; Index++)
//                         {
//                             Value |= FeatureUnitDescriptor->bmaControls[Index];
//                         }
//
//
//                         if (Value & 0x01) /* MUTE*/
//                         {
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Type = &KSNODETYPE_MUTE;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Name = &KSNODETYPE_MUTE;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].AutomationTable = AutomationTable = AllocFunction(sizeof(KSAUTOMATION_TABLE));
//                             if (AutomationTable)
//                             {
//                                 AutomationTable->PropertySets = FilterAudioMutePropertySetArray;
//                                 AutomationTable->PropertySetsCount = 1;
//                                 AutomationTable->PropertyItemSize = sizeof(KSPROPERTY_ITEM);
//                             }
//
//                             /* insert into node context*/
//                             NodeContext[DescriptorCount].Nodes[NodeContext[DescriptorCount].NodeCount] = FilterDescriptor->NodeDescriptorsCount;
//                             NodeContext[DescriptorCount].NodeCount++;
//
//                             FilterDescriptor->NodeDescriptorsCount++;
//                         }
//                         if (Value & 0x02) /* VOLUME */
//                         {
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Type = &KSNODETYPE_VOLUME;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Name = &KSNODETYPE_VOLUME;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].AutomationTable = AutomationTable = AllocFunction(sizeof(KSAUTOMATION_TABLE));
//                             if (AutomationTable)
//                             {
//                                 AutomationTable->PropertySets = FilterAudioVolumePropertySetArray;
//                                 AutomationTable->PropertySetsCount = 1;
//                                 AutomationTable->PropertyItemSize = sizeof(KSPROPERTY_ITEM);
//                             }
//
//                             /* insert into node context*/
//                             NodeContext[DescriptorCount].Nodes[NodeContext[DescriptorCount].NodeCount] = FilterDescriptor->NodeDescriptorsCount;
//                             NodeContext[DescriptorCount].NodeCount++;
//
//                             FilterDescriptor->NodeDescriptorsCount++;
//                         }
//
//                         if (Value & 0x04) /* BASS */
//                         {
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Type = &KSNODETYPE_TONE;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Name = &KSNODETYPE_TONE;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].AutomationTable = AllocFunction(sizeof(KSAUTOMATION_TABLE));
//
//                             /* insert into node context*/
//                             NodeContext[DescriptorCount].Nodes[NodeContext[DescriptorCount].NodeCount] = FilterDescriptor->NodeDescriptorsCount;
//                             NodeContext[DescriptorCount].NodeCount++;
//
//                             FilterDescriptor->NodeDescriptorsCount++;
//                         }
//
//                         if (Value & 0x08) /* MID */
//                         {
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Type = &KSNODETYPE_TONE;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Name = &KSNODETYPE_TONE;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].AutomationTable = AllocFunction(sizeof(KSAUTOMATION_TABLE));
//
//                             /* insert into node context*/
//                             NodeContext[DescriptorCount].Nodes[NodeContext[DescriptorCount].NodeCount] = FilterDescriptor->NodeDescriptorsCount;
//                             NodeContext[DescriptorCount].NodeCount++;
//
//                             FilterDescriptor->NodeDescriptorsCount++;
//                         }
//
//                         if (Value & 0x10) /* TREBLE */
//                         {
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Type = &KSNODETYPE_TONE;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Name = &KSNODETYPE_TONE;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].AutomationTable = AllocFunction(sizeof(KSAUTOMATION_TABLE));
//
//                             /* insert into node context*/
//                             NodeContext[DescriptorCount].Nodes[NodeContext[DescriptorCount].NodeCount] = FilterDescriptor->NodeDescriptorsCount;
//                             NodeContext[DescriptorCount].NodeCount++;
//
//
//                             FilterDescriptor->NodeDescriptorsCount++;
//                         }
//
//                         if (Value & 0x20) /* GRAPHIC EQUALIZER */
//                         {
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Type = &KSNODETYPE_TONE;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Name = &KSNODETYPE_TONE;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].AutomationTable = AllocFunction(sizeof(KSAUTOMATION_TABLE));
//
//                             /* insert into node context*/
//                             NodeContext[DescriptorCount].Nodes[NodeContext[DescriptorCount].NodeCount] = FilterDescriptor->NodeDescriptorsCount;
//                             NodeContext[DescriptorCount].NodeCount++;
//
//                             FilterDescriptor->NodeDescriptorsCount++;
//                         }
//
//                         if (Value & 0x40) /* AUTOMATIC GAIN */
//                         {
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Type = &KSNODETYPE_AGC;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Name = &KSNODETYPE_AGC;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].AutomationTable = AllocFunction(sizeof(KSAUTOMATION_TABLE));
//
//                             /* insert into node context*/
//                             NodeContext[DescriptorCount].Nodes[NodeContext[DescriptorCount].NodeCount] = FilterDescriptor->NodeDescriptorsCount;
//                             NodeContext[DescriptorCount].NodeCount++;
//
//
//                             FilterDescriptor->NodeDescriptorsCount++;
//                         }
//
//                         if (Value & 0x80) /* DELAY */
//                         {
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Type = &KSNODETYPE_TONE;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Name = &KSNODETYPE_TONE;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].AutomationTable = AllocFunction(sizeof(KSAUTOMATION_TABLE));
//
//                             /* insert into node context*/
//                             NodeContext[DescriptorCount].Nodes[NodeContext[DescriptorCount].NodeCount] = FilterDescriptor->NodeDescriptorsCount;
//                             NodeContext[DescriptorCount].NodeCount++;
//
//                             FilterDescriptor->NodeDescriptorsCount++;
//                         }
//                         NodeContext[DescriptorCount].Descriptor = CommonDescriptor;
//                         DescriptorCount++;
//
//                     }
//                     else if (InputTerminalDescriptor->bDescriptorSubtype == 0x04 /* MIXER_UNIT */)
//                     {
//                         MixerUnitDescriptor = (PUSB_AUDIO_CONTROL_MIXER_UNIT_DESCRIPTOR)CommonDescriptor;
//                         for (Index = 0; Index < MixerUnitDescriptor->bNrInPins; Index++)
//                         {
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Type = &KSNODETYPE_SUPERMIX;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Name = &KSNODETYPE_SUPERMIX;
//                             NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].AutomationTable = AllocFunction(sizeof(KSAUTOMATION_TABLE));
//
//                             /* insert into node context*/
//                             NodeContext[DescriptorCount].Nodes[NodeContext[DescriptorCount].NodeCount] = FilterDescriptor->NodeDescriptorsCount;
//                             NodeContext[DescriptorCount].NodeCount++;
//
//                             FilterDescriptor->NodeDescriptorsCount++;
//                         }
//
//                         NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Type = &KSNODETYPE_SUM;
//                         NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Name = &KSNODETYPE_SUM;
//                         NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].AutomationTable = AllocFunction(sizeof(KSAUTOMATION_TABLE));
//
//                         /* insert into node context*/
//                         NodeContext[DescriptorCount].Nodes[NodeContext[DescriptorCount].NodeCount] = FilterDescriptor->NodeDescriptorsCount;
//                         NodeContext[DescriptorCount].NodeCount++;
//                         NodeContext[DescriptorCount].Descriptor = CommonDescriptor;
//                         DescriptorCount++;
//
//                         FilterDescriptor->NodeDescriptorsCount++;
//                     }
//                     else if (InputTerminalDescriptor->bDescriptorSubtype == 0x05 /* SELECTOR UNIT */)
//                     {
//                         NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Type = &KSNODETYPE_MUX;
//                         NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].Name = &KSNODETYPE_MUX;
//                         NodeDescriptors[FilterDescriptor->NodeDescriptorsCount].AutomationTable = AllocFunction(sizeof(KSAUTOMATION_TABLE));
//
//                         /* insert into node context*/
//                         NodeContext[DescriptorCount].Descriptor = CommonDescriptor;
//                         NodeContext[DescriptorCount].NodeCount = 1;
//                         NodeContext[DescriptorCount].Nodes[0] = FilterDescriptor->NodeDescriptorsCount;
//                         DescriptorCount++;
//                         FilterDescriptor->NodeDescriptorsCount++;
//                     }
//                     else
//                     {
//                         UNIMPLEMENTED;
//                     }
//                     CommonDescriptor = (PUSB_COMMON_DESCRIPTOR)((ULONG_PTR)CommonDescriptor + CommonDescriptor->bLength);
//                     if ((ULONG_PTR)CommonDescriptor >= ((ULONG_PTR)InterfaceHeaderDescriptor + InterfaceHeaderDescriptor->wTotalLength))
//                         break;
//                 }
//             }
//         }
//     }
//
//     /* FIXME determine connections count*/
//     FilterDescriptor->Connections = Connections = AllocFunction(sizeof(KSTOPOLOGY_CONNECTION) * FilterDescriptor->NodeDescriptorsCount * 2);
//     if (!FilterDescriptor->Connections)
//     {
//         /* no memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     FilterDescriptor->ConnectionsCount = 0;
//
//     /* now build connections array */
//     DescriptorCount = 0;
//     StreamingTerminalIndex = 0;
//     NodeCount = 0;
//
//     CountTerminalUnits(DeviceExtension->ConfigurationDescriptor, &NonStreamingTerminalDescriptorCount, &TotalTerminalDescriptorCount);
//     StreamingTerminalPinOffset = TotalTerminalDescriptorCount - NonStreamingTerminalDescriptorCount;
//
//     for (Descriptor = USBD_ParseConfigurationDescriptorEx(DeviceExtension->ConfigurationDescriptor, DeviceExtension->ConfigurationDescriptor, -1, -1, USB_DEVICE_CLASS_AUDIO, -1, -1);
//     Descriptor != NULL;
//         Descriptor = USBD_ParseConfigurationDescriptorEx(DeviceExtension->ConfigurationDescriptor, (PVOID)((ULONG_PTR)Descriptor + Descriptor->bLength), -1, -1, USB_DEVICE_CLASS_AUDIO, -1, -1))
//     {
//         if (Descriptor->bInterfaceSubClass == 0x01) /* AUDIO_CONTROL */
//         {
//             InterfaceHeaderDescriptor = (PUSB_AUDIO_CONTROL_INTERFACE_HEADER_DESCRIPTOR)USBD_ParseDescriptors(DeviceExtension->ConfigurationDescriptor, DeviceExtension->ConfigurationDescriptor->wTotalLength, Descriptor, USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE);
//             if (InterfaceHeaderDescriptor != NULL)
//             {
//                 CommonDescriptor = USBD_ParseDescriptors(InterfaceHeaderDescriptor, InterfaceHeaderDescriptor->wTotalLength, (PVOID)((ULONG_PTR)InterfaceHeaderDescriptor + InterfaceHeaderDescriptor->bLength), USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE);
//                 while (CommonDescriptor)
//                 {
//                     InputTerminalDescriptor = (PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR)CommonDescriptor;
//                     if (InputTerminalDescriptor->bDescriptorSubtype == 0x02 /* INPUT TERMINAL*/)
//                     {
//                         if (InputTerminalDescriptor->wTerminalType == USB_AUDIO_STREAMING_TERMINAL_TYPE)
//                         {
//                              Connections[FilterDescriptor->ConnectionsCount].FromNode = KSFILTER_NODE;
//                              Connections[FilterDescriptor->ConnectionsCount].FromNodePin = StreamingTerminalIndex;
//                              Connections[FilterDescriptor->ConnectionsCount].ToNodePin = 1;
//                              Connections[FilterDescriptor->ConnectionsCount].ToNode = NodeContext[DescriptorCount].Nodes[0];
//                              FilterDescriptor->ConnectionsCount++;
//                              StreamingTerminalIndex++;
//
//                         }
//                         else
//                         {
//                             Connections[FilterDescriptor->ConnectionsCount].FromNode = KSFILTER_NODE;
//                             Connections[FilterDescriptor->ConnectionsCount].FromNodePin = StreamingTerminalPinOffset;
//                             Connections[FilterDescriptor->ConnectionsCount].ToNodePin = 1;
//                             Connections[FilterDescriptor->ConnectionsCount].ToNode = NodeContext[DescriptorCount].Nodes[0];
//                             FilterDescriptor->ConnectionsCount++;
//                             StreamingTerminalPinOffset++;
//                         }
//                         DescriptorCount++;
//                     }
//                     else if (InputTerminalDescriptor->bDescriptorSubtype == 0x03 /* OUTPUT_TERMINAL*/)
//                     {
//                         OutputTerminalDescriptor = (PUSB_AUDIO_CONTROL_OUTPUT_TERMINAL_DESCRIPTOR)CommonDescriptor;
//                         PreviousNodeContext = FindNodeContextWithId(NodeContext, ControlDescriptorCount, OutputTerminalDescriptor->bSourceID);
//                         if (PreviousNodeContext)
//                         {
//                             Connections[FilterDescriptor->ConnectionsCount].FromNode = PreviousNodeContext->Nodes[PreviousNodeContext->NodeCount - 1];
//                             Connections[FilterDescriptor->ConnectionsCount].FromNodePin = 0;
//                             Connections[FilterDescriptor->ConnectionsCount].ToNodePin = 1;
//                             Connections[FilterDescriptor->ConnectionsCount].ToNode = NodeContext[DescriptorCount].Nodes[0];
//                             FilterDescriptor->ConnectionsCount++;
//                         }
//
//                         if (InputTerminalDescriptor->wTerminalType == USB_AUDIO_STREAMING_TERMINAL_TYPE)
//                         {
//                             Connections[FilterDescriptor->ConnectionsCount].FromNode = NodeContext[DescriptorCount].Nodes[0];
//                             Connections[FilterDescriptor->ConnectionsCount].FromNodePin = 0;
//                             Connections[FilterDescriptor->ConnectionsCount].ToNodePin = StreamingTerminalIndex;
//                             Connections[FilterDescriptor->ConnectionsCount].ToNode = KSFILTER_NODE;
//                             FilterDescriptor->ConnectionsCount++;
//                             StreamingTerminalIndex++;
//                         }
//                         else
//                         {
//                             Connections[FilterDescriptor->ConnectionsCount].FromNode = NodeContext[DescriptorCount].Nodes[0];
//                             Connections[FilterDescriptor->ConnectionsCount].FromNodePin = 0;
//                             Connections[FilterDescriptor->ConnectionsCount].ToNodePin = StreamingTerminalPinOffset;
//                             Connections[FilterDescriptor->ConnectionsCount].ToNode = KSFILTER_NODE;
//                             FilterDescriptor->ConnectionsCount++;
//
//                             StreamingTerminalPinOffset++;
//                         }
//                         DescriptorCount++;
//                     }
//                     else if (InputTerminalDescriptor->bDescriptorSubtype == 0x06 /* FEATURE_UNIT*/)
//                     {
//                         FeatureUnitDescriptor = (PUSB_AUDIO_CONTROL_FEATURE_UNIT_DESCRIPTOR)InputTerminalDescriptor;
//                         PreviousNodeContext = FindNodeContextWithId(NodeContext, ControlDescriptorCount, FeatureUnitDescriptor->bSourceID);
//                         if (PreviousNodeContext)
//                         {
//                             Connections[FilterDescriptor->ConnectionsCount].FromNode = PreviousNodeContext->Nodes[PreviousNodeContext->NodeCount-1];
//                             Connections[FilterDescriptor->ConnectionsCount].FromNodePin = 0;
//                             Connections[FilterDescriptor->ConnectionsCount].ToNodePin = 1;
//                             Connections[FilterDescriptor->ConnectionsCount].ToNode = NodeContext[DescriptorCount].Nodes[0];
//                             FilterDescriptor->ConnectionsCount++;
//                         }
//                         for (Index = 1; Index < NodeContext[DescriptorCount].NodeCount; Index++)
//                         {
//                             Connections[FilterDescriptor->ConnectionsCount].FromNode = NodeContext[DescriptorCount].Nodes[Index - 1];
//                             Connections[FilterDescriptor->ConnectionsCount].FromNodePin = 0;
//                             Connections[FilterDescriptor->ConnectionsCount].ToNodePin = 1;
//                             Connections[FilterDescriptor->ConnectionsCount].ToNode = NodeContext[DescriptorCount].Nodes[Index];
//                             FilterDescriptor->ConnectionsCount++;
//                         }
//
//                         DescriptorCount++;
//                     }
//                     else if (InputTerminalDescriptor->bDescriptorSubtype == 0x04 /* MIXER_UNIT */)
//                     {
//                         MixerUnitDescriptor = (PUSB_AUDIO_CONTROL_MIXER_UNIT_DESCRIPTOR)InputTerminalDescriptor;
//                         for (Index = 0; Index < MixerUnitDescriptor->bNrInPins; Index++)
//                         {
//                             Value = MixerUnitDescriptor->baSourceID[Index];
//                             PreviousNodeContext = FindNodeContextWithId(NodeContext, ControlDescriptorCount, Value);
//                             if (PreviousNodeContext)
//                             {
//                                 Connections[FilterDescriptor->ConnectionsCount].FromNode = PreviousNodeContext->Nodes[PreviousNodeContext->NodeCount - 1];
//                                 Connections[FilterDescriptor->ConnectionsCount].FromNodePin = 0;
//                                 Connections[FilterDescriptor->ConnectionsCount].ToNodePin = 1;
//                                 Connections[FilterDescriptor->ConnectionsCount].ToNode = NodeContext[DescriptorCount].Nodes[Index];
//                                 FilterDescriptor->ConnectionsCount++;
//                             }
//
//                             Connections[FilterDescriptor->ConnectionsCount].FromNode = NodeContext[DescriptorCount].Nodes[Index];
//                             Connections[FilterDescriptor->ConnectionsCount].FromNodePin = 0;
//                             Connections[FilterDescriptor->ConnectionsCount].ToNodePin = 1 + Index;
//                             Connections[FilterDescriptor->ConnectionsCount].ToNode = NodeContext[DescriptorCount].Nodes[NodeContext[DescriptorCount].NodeCount-1];
//                             FilterDescriptor->ConnectionsCount++;
//                         }
//                         DescriptorCount++;
//                     }
//                     else if (InputTerminalDescriptor->bDescriptorSubtype == 0x05 /* SELECTOR_UNIT */)
//                     {
//                         SelectorUnitDescriptor = (PUSB_AUDIO_CONTROL_SELECTOR_UNIT_DESCRIPTOR)InputTerminalDescriptor;
//                         for (Index = 0; Index < SelectorUnitDescriptor->bNrInPins; Index++)
//                         {
//                             Value = SelectorUnitDescriptor->baSourceID[Index];
//                             PreviousNodeContext = FindNodeContextWithId(NodeContext, ControlDescriptorCount, Value);
//                             if (PreviousNodeContext)
//                             {
//                                 Connections[FilterDescriptor->ConnectionsCount].FromNode = PreviousNodeContext->Nodes[PreviousNodeContext->NodeCount - 1];
//                                 Connections[FilterDescriptor->ConnectionsCount].FromNodePin = 0;
//                                 Connections[FilterDescriptor->ConnectionsCount].ToNodePin = 1;
//                                 Connections[FilterDescriptor->ConnectionsCount].ToNode = NodeContext[DescriptorCount].Nodes[0];
//                                 FilterDescriptor->ConnectionsCount++;
//                             }
//                         }
//                         DescriptorCount++;
//                     }
//                     else
//                     {
//                         UNIMPLEMENTED;
//                     }
//                     CommonDescriptor = (PUSB_COMMON_DESCRIPTOR)((ULONG_PTR)CommonDescriptor + CommonDescriptor->bLength);
//                     if ((ULONG_PTR)CommonDescriptor >= ((ULONG_PTR)InterfaceHeaderDescriptor + InterfaceHeaderDescriptor->wTotalLength))
//                         break;
//                 }
//             }
//         }
//     }
//
//
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// USBAudioFilterCreate(
//     PKSFILTER Filter,
//     PIRP Irp)
// {
//     PKSFILTERFACTORY FilterFactory;
//     PKSDEVICE Device;
//     PFILTER_CONTEXT FilterContext;
//
//     FilterFactory = KsGetParent(Filter);
//     if (FilterFactory == NULL)
//     {
//         /* invalid parameter */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     Device = KsGetParent(FilterFactory);
//     if (Device == NULL)
//     {
//         /* invalid parameter */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* alloc filter context */
//     FilterContext = AllocFunction(sizeof(FILTER_CONTEXT));
//     if (FilterContext == NULL)
//     {
//         /* no memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* init context */
//     FilterContext->DeviceExtension = Device->Context;
//     FilterContext->LowerDevice = Device->NextDeviceObject;
//     Filter->Context = FilterContext;
//
//     DPRINT("USBAudioFilterCreate FilterContext %p LowerDevice %p DeviceExtension %p\n", FilterContext, FilterContext->LowerDevice, FilterContext->DeviceExtension);
//     KsAddItemToObjectBag(Filter->Bag, FilterContext, ExFreePool);
//     return STATUS_SUCCESS;
// }
//
//
// VOID
// NTAPI
// CountTerminalUnits(
//     IN PUSB_CONFIGURATION_DESCRIPTOR ConfigurationDescriptor,
//     OUT PULONG NonStreamingTerminalDescriptorCount,
//     OUT PULONG TotalTerminalDescriptorCount)
// {
//     PUSB_INTERFACE_DESCRIPTOR Descriptor;
//     PUSB_AUDIO_CONTROL_INTERFACE_HEADER_DESCRIPTOR InterfaceHeaderDescriptor;
//     PUSB_COMMON_DESCRIPTOR CommonDescriptor;
//     PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR InputTerminalDescriptor;
//     ULONG NonStreamingTerminalCount = 0;
//     ULONG TotalTerminalCount = 0;
//
//     for(Descriptor = USBD_ParseConfigurationDescriptorEx(ConfigurationDescriptor, ConfigurationDescriptor, -1, -1, USB_DEVICE_CLASS_AUDIO, -1, -1);
//         Descriptor != NULL;
//         Descriptor = USBD_ParseConfigurationDescriptorEx(ConfigurationDescriptor, (PVOID)((ULONG_PTR)Descriptor + Descriptor->bLength), -1, -1, USB_DEVICE_CLASS_AUDIO, -1, -1))
//     {
//         if (Descriptor->bInterfaceSubClass == 0x01) /* AUDIO_CONTROL */
//         {
//             InterfaceHeaderDescriptor = (PUSB_AUDIO_CONTROL_INTERFACE_HEADER_DESCRIPTOR)USBD_ParseDescriptors(ConfigurationDescriptor, ConfigurationDescriptor->wTotalLength, Descriptor, USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE);
//             if (InterfaceHeaderDescriptor != NULL)
//             {
//                 CommonDescriptor = USBD_ParseDescriptors(InterfaceHeaderDescriptor, InterfaceHeaderDescriptor->wTotalLength, (PVOID)((ULONG_PTR)InterfaceHeaderDescriptor + InterfaceHeaderDescriptor->bLength), USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE);
//                 while (CommonDescriptor)
//                 {
//                     InputTerminalDescriptor = (PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR)CommonDescriptor;
//                     if (InputTerminalDescriptor->bDescriptorSubtype == 0x02 /* INPUT TERMINAL*/ || InputTerminalDescriptor->bDescriptorSubtype == 0x03 /* OUTPUT_TERMINAL*/)
//                     {
//                         if (InputTerminalDescriptor->wTerminalType != USB_AUDIO_STREAMING_TERMINAL_TYPE)
//                         {
//                             NonStreamingTerminalCount++;
//                         }
//                         TotalTerminalCount++;
//                     }
//                     CommonDescriptor = (PUSB_COMMON_DESCRIPTOR)((ULONG_PTR)CommonDescriptor + CommonDescriptor->bLength);
//                     if ((ULONG_PTR)CommonDescriptor >= ((ULONG_PTR)InterfaceHeaderDescriptor + InterfaceHeaderDescriptor->wTotalLength))
//                         break;
//                 }
//             }
//         }
//         else if (Descriptor->bInterfaceSubClass == 0x03) /* MIDI_STREAMING */
//         {
//             UNIMPLEMENTED;
//         }
//     }
//     *NonStreamingTerminalDescriptorCount = NonStreamingTerminalCount;
//     *TotalTerminalDescriptorCount = TotalTerminalCount;
// }
//
// LPGUID
// UsbAudioGetPinCategoryFromTerminalDescriptor(
//     IN PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR TerminalDescriptor)
// {
//     if (TerminalDescriptor->wTerminalType == USB_AUDIO_MICROPHONE_TERMINAL_TYPE)
//         return &NodeTypeMicrophone;
//     else if (TerminalDescriptor->wTerminalType == USB_AUDIO_DESKTOP_MICROPHONE_TERMINAL_TYPE)
//         return &NodeTypeDesktopMicrophone;
//     else if (TerminalDescriptor->wTerminalType == USB_AUDIO_PERSONAL_MICROPHONE_TERMINAL_TYPE)
//         return &NodeTypePersonalMicrophone;
//     else if (TerminalDescriptor->wTerminalType == USB_AUDIO_OMMNI_MICROPHONE_TERMINAL_TYPE)
//         return &NodeTypeOmmniMicrophone;
//     else if (TerminalDescriptor->wTerminalType == USB_AUDIO_ARRAY_MICROPHONE_TERMINAL_TYPE)
//         return &NodeTypeArrayMicrophone;
//     else if (TerminalDescriptor->wTerminalType == USB_AUDIO_ARRAY_PROCESSING_MICROPHONE_TERMINAL_TYPE)
//         return &NodeTypeProcessingArrayMicrophone;
//
//     /* playback types */
//     if (TerminalDescriptor->wTerminalType == USB_AUDIO_SPEAKER_TERMINAL_TYPE)
//         return &NodeTypeSpeaker;
//     else if (TerminalDescriptor->wTerminalType == USB_HEADPHONES_SPEAKER_TERMINAL_TYPE)
//         return &NodeTypeHeadphonesSpeaker;
//     else if (TerminalDescriptor->wTerminalType == USB_AUDIO_HMDA_TERMINAL_TYPE)
//         return &NodeTypeHMDA;
//     else if (TerminalDescriptor->wTerminalType == USB_AUDIO_DESKTOP_SPEAKER_TERMINAL_TYPE)
//         return &NodeTypeDesktopSpeaker;
//     else if (TerminalDescriptor->wTerminalType == USB_AUDIO_ROOM_SPEAKER_TERMINAL_TYPE)
//         return &NodeTypeRoomSpeaker;
//     else if (TerminalDescriptor->wTerminalType == USB_AUDIO_COMMUNICATION_SPEAKER_TERMINAL_TYPE)
//         return &NodeTypeCommunicationSpeaker;
//     else if (TerminalDescriptor->wTerminalType == USB_AUDIO_SUBWOOFER_TERMINAL_TYPE)
//         return &NodeTypeSubwoofer;
//
//     if (TerminalDescriptor->wTerminalType == USB_AUDIO_STREAMING_TERMINAL_TYPE)
//     {
//         if (TerminalDescriptor->bDescriptorSubtype == USB_AUDIO_OUTPUT_TERMINAL)
//             return &NodeTypeCapture;
//         else if (TerminalDescriptor->bDescriptorSubtype == USB_AUDIO_INPUT_TERMINAL)
//             return &NodeTypePlayback;
//
//     }
//     return NULL;
// }
//
// PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR
// UsbAudioGetStreamingTerminalDescriptorByIndex(
//     IN PUSB_CONFIGURATION_DESCRIPTOR ConfigurationDescriptor,
//     IN ULONG Index)
// {
//     PUSB_INTERFACE_DESCRIPTOR Descriptor;
//     PUSB_AUDIO_CONTROL_INTERFACE_HEADER_DESCRIPTOR InterfaceHeaderDescriptor;
//     PUSB_COMMON_DESCRIPTOR CommonDescriptor;
//     PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR InputTerminalDescriptor;
//     ULONG TerminalCount = 0;
//
//     for (Descriptor = USBD_ParseConfigurationDescriptorEx(ConfigurationDescriptor, ConfigurationDescriptor, -1, -1, USB_DEVICE_CLASS_AUDIO, -1, -1);
//     Descriptor != NULL;
//         Descriptor = USBD_ParseConfigurationDescriptorEx(ConfigurationDescriptor, (PVOID)((ULONG_PTR)Descriptor + Descriptor->bLength), -1, -1, USB_DEVICE_CLASS_AUDIO, -1, -1))
//     {
//         if (Descriptor->bInterfaceSubClass == 0x01) /* AUDIO_CONTROL */
//         {
//             InterfaceHeaderDescriptor = (PUSB_AUDIO_CONTROL_INTERFACE_HEADER_DESCRIPTOR)USBD_ParseDescriptors(ConfigurationDescriptor, ConfigurationDescriptor->wTotalLength, Descriptor, USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE);
//             if (InterfaceHeaderDescriptor != NULL)
//             {
//                 CommonDescriptor = USBD_ParseDescriptors(InterfaceHeaderDescriptor, InterfaceHeaderDescriptor->wTotalLength, (PVOID)((ULONG_PTR)InterfaceHeaderDescriptor + InterfaceHeaderDescriptor->bLength), USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE);
//                 while (CommonDescriptor)
//                 {
//                     InputTerminalDescriptor = (PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR)CommonDescriptor;
//                     if (InputTerminalDescriptor->bDescriptorSubtype == 0x02 /* INPUT TERMINAL*/ || InputTerminalDescriptor->bDescriptorSubtype == 0x03 /* OUTPUT_TERMINAL*/)
//                     {
//                         if (InputTerminalDescriptor->wTerminalType == USB_AUDIO_STREAMING_TERMINAL_TYPE)
//                         {
//                             if (TerminalCount == Index)
//                             {
//                                 return InputTerminalDescriptor;
//                             }
//                             TerminalCount++;
//                         }
//                     }
//                     CommonDescriptor = (PUSB_COMMON_DESCRIPTOR)((ULONG_PTR)CommonDescriptor + CommonDescriptor->bLength);
//                     if ((ULONG_PTR)CommonDescriptor >= ((ULONG_PTR)InterfaceHeaderDescriptor + InterfaceHeaderDescriptor->wTotalLength))
//                         break;
//                 }
//             }
//         }
//     }
//     return NULL;
// }
//
// PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR
// UsbAudioGetNonStreamingTerminalDescriptorByIndex(
//     IN PUSB_CONFIGURATION_DESCRIPTOR ConfigurationDescriptor,
//     IN ULONG Index)
// {
//
//     PUSB_INTERFACE_DESCRIPTOR Descriptor;
//     PUSB_AUDIO_CONTROL_INTERFACE_HEADER_DESCRIPTOR InterfaceHeaderDescriptor;
//     PUSB_COMMON_DESCRIPTOR CommonDescriptor;
//     PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR InputTerminalDescriptor;
//     ULONG TerminalCount = 0;
//
//     for (Descriptor = USBD_ParseConfigurationDescriptorEx(ConfigurationDescriptor, ConfigurationDescriptor, -1, -1, USB_DEVICE_CLASS_AUDIO, -1, -1);
//     Descriptor != NULL;
//         Descriptor = USBD_ParseConfigurationDescriptorEx(ConfigurationDescriptor, (PVOID)((ULONG_PTR)Descriptor + Descriptor->bLength), -1, -1, USB_DEVICE_CLASS_AUDIO, -1, -1))
//     {
//         if (Descriptor->bInterfaceSubClass == 0x01) /* AUDIO_CONTROL */
//         {
//             InterfaceHeaderDescriptor = (PUSB_AUDIO_CONTROL_INTERFACE_HEADER_DESCRIPTOR)USBD_ParseDescriptors(ConfigurationDescriptor, ConfigurationDescriptor->wTotalLength, Descriptor, USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE);
//             if (InterfaceHeaderDescriptor != NULL)
//             {
//                 CommonDescriptor = USBD_ParseDescriptors(InterfaceHeaderDescriptor, InterfaceHeaderDescriptor->wTotalLength, (PVOID)((ULONG_PTR)InterfaceHeaderDescriptor + InterfaceHeaderDescriptor->bLength), USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE);
//                 while (CommonDescriptor)
//                 {
//                     InputTerminalDescriptor = (PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR)CommonDescriptor;
//                     if (InputTerminalDescriptor->bDescriptorSubtype == 0x02 /* INPUT TERMINAL*/ || InputTerminalDescriptor->bDescriptorSubtype == 0x03 /* OUTPUT_TERMINAL*/)
//                     {
//                         if (InputTerminalDescriptor->wTerminalType != USB_AUDIO_STREAMING_TERMINAL_TYPE)
//                         {
//                             if (TerminalCount == Index)
//                             {
//                                 return InputTerminalDescriptor;
//                             }
//                             TerminalCount++;
//                         }
//                     }
//                     CommonDescriptor = (PUSB_COMMON_DESCRIPTOR)((ULONG_PTR)CommonDescriptor + CommonDescriptor->bLength);
//                     if ((ULONG_PTR)CommonDescriptor >= ((ULONG_PTR)InterfaceHeaderDescriptor + InterfaceHeaderDescriptor->wTotalLength))
//                         break;
//                 }
//             }
//         }
//     }
//     return NULL;
// }
//
// VOID
// UsbAudioGetDataRanges(
//     IN PUSB_CONFIGURATION_DESCRIPTOR ConfigurationDescriptor,
//     IN UCHAR bTerminalID,
//     OUT PKSDATARANGE** OutDataRanges,
//     OUT PULONG OutDataRangesCount)
// {
//     PUSB_AUDIO_STREAMING_INTERFACE_DESCRIPTOR StreamingInterfaceDescriptor;
//     PUSB_AUDIO_STREAMING_FORMAT_TYPE_DESCRIPTOR StreamingFormatDescriptor;
//     PUSB_INTERFACE_DESCRIPTOR Descriptor;
//     PKSDATARANGE_AUDIO DataRangeAudio;
//     PKSDATARANGE *DataRangeAudioArray;
//     ULONG NumFrequency, DataRangeCount, DataRangeIndex, Index;
//
//     /* count all data ranges */
//     DataRangeCount = 0;
//     for (Descriptor = USBD_ParseConfigurationDescriptorEx(ConfigurationDescriptor, ConfigurationDescriptor, -1, -1, USB_DEVICE_CLASS_AUDIO, -1, -1);
//     Descriptor != NULL;
//         Descriptor = USBD_ParseConfigurationDescriptorEx(ConfigurationDescriptor, (PVOID)((ULONG_PTR)Descriptor + Descriptor->bLength), -1, -1, USB_DEVICE_CLASS_AUDIO, -1, -1))
//     {
//         if (Descriptor->bInterfaceSubClass == 0x02) /* AUDIO_STREAMING */
//         {
//             StreamingInterfaceDescriptor = (PUSB_AUDIO_STREAMING_INTERFACE_DESCRIPTOR)USBD_ParseDescriptors(ConfigurationDescriptor, ConfigurationDescriptor->wTotalLength, Descriptor, USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE);
//             if (StreamingInterfaceDescriptor != NULL)
//             {
//                 ASSERT(StreamingInterfaceDescriptor->bDescriptorSubtype == 0x01);
//                 ASSERT(StreamingInterfaceDescriptor->wFormatTag == WAVE_FORMAT_PCM);
//                 if (StreamingInterfaceDescriptor->bTerminalLink == bTerminalID)
//                 {
//                     DataRangeCount++;
//                     DPRINT1("StreamingInterfaceDescriptor %p TerminalID %x\n", StreamingInterfaceDescriptor, bTerminalID);
//                 }
//             }
//             Descriptor = (PUSB_INTERFACE_DESCRIPTOR)StreamingInterfaceDescriptor;
//         }
//     }
//
//     DataRangeAudioArray = AllocFunction(sizeof(PVOID) * DataRangeCount);
//     if (DataRangeAudioArray == NULL)
//     {
//         /* no memory */
//         return;
//     }
//
//     DataRangeIndex = 0;
//     for (Descriptor = USBD_ParseConfigurationDescriptorEx(ConfigurationDescriptor, ConfigurationDescriptor, -1, -1, USB_DEVICE_CLASS_AUDIO, -1, -1);
//     Descriptor != NULL;
//         Descriptor = USBD_ParseConfigurationDescriptorEx(ConfigurationDescriptor, (PVOID)((ULONG_PTR)Descriptor + Descriptor->bLength), -1, -1, USB_DEVICE_CLASS_AUDIO, -1, -1))
//     {
//         if (Descriptor->bInterfaceSubClass == 0x02) /* AUDIO_STREAMING */
//         {
//             StreamingInterfaceDescriptor = (PUSB_AUDIO_STREAMING_INTERFACE_DESCRIPTOR)USBD_ParseDescriptors(ConfigurationDescriptor, ConfigurationDescriptor->wTotalLength, Descriptor, USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE);
//             if (StreamingInterfaceDescriptor != NULL)
//             {
//                 ASSERT(StreamingInterfaceDescriptor->bDescriptorSubtype == 0x01);
//                 ASSERT(StreamingInterfaceDescriptor->wFormatTag == WAVE_FORMAT_PCM);
//                 if (StreamingInterfaceDescriptor->bTerminalLink == bTerminalID)
//                 {
//                     StreamingFormatDescriptor = (PUSB_AUDIO_STREAMING_FORMAT_TYPE_DESCRIPTOR)((ULONG_PTR)StreamingInterfaceDescriptor + StreamingInterfaceDescriptor->bLength);
//                     ASSERT(StreamingFormatDescriptor->bDescriptorType == 0x24);
//                     ASSERT(StreamingFormatDescriptor->bDescriptorSubtype == 0x02);
//                     ASSERT(StreamingFormatDescriptor->bFormatType == 0x01);
//
//                     DataRangeAudio = AllocFunction(sizeof(KSDATARANGE_AUDIO));
//                     if (DataRangeAudio == NULL)
//                     {
//                         /* no memory*/
//                         return;
//                     }
//
//                     DataRangeAudio->DataRange.FormatSize = sizeof(KSDATARANGE_AUDIO);
//                     DataRangeAudio->DataRange.MajorFormat = KSDATAFORMAT_TYPE_AUDIO;
//                     DataRangeAudio->DataRange.SubFormat = KSDATAFORMAT_SUBTYPE_PCM;
//                     DataRangeAudio->DataRange.Specifier = KSDATAFORMAT_SPECIFIER_WAVEFORMATEX;
//                     DataRangeAudio->MaximumChannels = StreamingFormatDescriptor->bNrChannels;
//                     DataRangeAudio->MinimumBitsPerSample = StreamingFormatDescriptor->bBitResolution;
//                     DataRangeAudio->MaximumBitsPerSample = StreamingFormatDescriptor->bBitResolution;
//                     NumFrequency = StreamingFormatDescriptor->bSamFreqType;
//                     DataRangeAudio->MinimumSampleFrequency = MAXULONG;
//                     DataRangeAudio->MaximumSampleFrequency = 0;
//                     for (Index = 0; Index < NumFrequency; Index++)
//                     {
//                         DataRangeAudio->MinimumSampleFrequency = min(StreamingFormatDescriptor->tSamFreq[Index * 3] | StreamingFormatDescriptor->tSamFreq[(Index * 3) + 1] << 8 | StreamingFormatDescriptor->tSamFreq[(Index * 3) + 2] << 16, DataRangeAudio->MinimumSampleFrequency);
//                         DataRangeAudio->MaximumSampleFrequency = max(StreamingFormatDescriptor->tSamFreq[Index * 3] | StreamingFormatDescriptor->tSamFreq[(Index * 3) + 1] << 8 | StreamingFormatDescriptor->tSamFreq[(Index * 3) + 2] << 16, DataRangeAudio->MaximumSampleFrequency);
//                     }
//                     DataRangeAudioArray[DataRangeIndex] = (PKSDATARANGE)DataRangeAudio;
//                     DataRangeIndex++;
//                 }
//             }
//             Descriptor = (PUSB_INTERFACE_DESCRIPTOR)StreamingInterfaceDescriptor;
//         }
//     }
//
//     *OutDataRanges = DataRangeAudioArray;
//     *OutDataRangesCount = DataRangeCount;
// }
//
//
// NTSTATUS
// USBAudioPinBuildDescriptors(
//     PKSDEVICE Device,
//     PKSPIN_DESCRIPTOR_EX *PinDescriptors,
//     PULONG PinDescriptorsCount,
//     PULONG PinDescriptorSize)
// {
//     PDEVICE_EXTENSION DeviceExtension;
//     PKSPIN_DESCRIPTOR_EX Pins;
//     ULONG TotalTerminalDescriptorCount = 0;
//     ULONG NonStreamingTerminalDescriptorCount = 0;
//     ULONG Index = 0;
//     PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR TerminalDescriptor = NULL;
//
//     /* get device extension */
//     DeviceExtension = Device->Context;
//
//     CountTerminalUnits(DeviceExtension->ConfigurationDescriptor, &NonStreamingTerminalDescriptorCount, &TotalTerminalDescriptorCount);
//     DPRINT("TotalTerminalDescriptorCount %lu NonStreamingTerminalDescriptorCount %lu\n", TotalTerminalDescriptorCount, NonStreamingTerminalDescriptorCount);
//
//     /* allocate pins */
//     Pins = AllocFunction(sizeof(KSPIN_DESCRIPTOR_EX) * TotalTerminalDescriptorCount);
//     if (!Pins)
//     {
//         /* no memory*/
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     for (Index = 0; Index < TotalTerminalDescriptorCount; Index++)
//     {
//         if (Index < (TotalTerminalDescriptorCount - NonStreamingTerminalDescriptorCount))
//         {
//             /* irp sink pins*/
//             TerminalDescriptor = UsbAudioGetStreamingTerminalDescriptorByIndex(DeviceExtension->ConfigurationDescriptor, Index);
//             ASSERT(TerminalDescriptor != NULL);
//
//             Pins[Index].Dispatch = &UsbAudioPinDispatch;
//             Pins[Index].PinDescriptor.InterfacesCount = 1;
//             Pins[Index].PinDescriptor.Interfaces = &StandardPinInterface;
//             Pins[Index].PinDescriptor.MediumsCount = 1;
//             Pins[Index].PinDescriptor.Mediums = &StandardPinMedium;
//             Pins[Index].PinDescriptor.Category = UsbAudioGetPinCategoryFromTerminalDescriptor(TerminalDescriptor);
//             UsbAudioGetDataRanges(DeviceExtension->ConfigurationDescriptor, TerminalDescriptor->bTerminalID, (PKSDATARANGE**)&Pins[Index].PinDescriptor.DataRanges, &Pins[Index].PinDescriptor.DataRangesCount);
//
//             if (TerminalDescriptor->bDescriptorSubtype == USB_AUDIO_OUTPUT_TERMINAL)
//             {
//                 Pins[Index].PinDescriptor.Communication = KSPIN_COMMUNICATION_BOTH;
//                 Pins[Index].PinDescriptor.DataFlow = KSPIN_DATAFLOW_OUT;
//
//                 /* pin flags */
//                 Pins[Index].Flags = KSPIN_FLAG_PROCESS_IN_RUN_STATE_ONLY | KSFILTER_FLAG_CRITICAL_PROCESSING;
//             }
//             else if (TerminalDescriptor->bDescriptorSubtype == USB_AUDIO_INPUT_TERMINAL)
//             {
//                 Pins[Index].PinDescriptor.Communication = KSPIN_COMMUNICATION_SINK;
//                 Pins[Index].PinDescriptor.DataFlow = KSPIN_DATAFLOW_IN;
//
//                 /* pin flags */
//                 Pins[Index].Flags = KSPIN_FLAG_PROCESS_IN_RUN_STATE_ONLY | KSPIN_FLAG_GENERATE_EOS_EVENTS;
//             }
//
//             /* data intersect handler */
//             Pins[Index].IntersectHandler = UsbAudioPinDataIntersect;
//
//             /* irp sinks / sources can be instantiated */
//             Pins[Index].InstancesPossible = 1;
//         }
//         else
//         {
//             /* bridge pins */
//             TerminalDescriptor = UsbAudioGetNonStreamingTerminalDescriptorByIndex(DeviceExtension->ConfigurationDescriptor, Index - (TotalTerminalDescriptorCount - NonStreamingTerminalDescriptorCount));
//             Pins[Index].PinDescriptor.InterfacesCount = 1;
//             Pins[Index].PinDescriptor.Interfaces = &StandardPinInterface;
//             Pins[Index].PinDescriptor.MediumsCount = 1;
//             Pins[Index].PinDescriptor.Mediums = &StandardPinMedium;
//             Pins[Index].PinDescriptor.DataRanges = BridgePinAudioFormats;
//             Pins[Index].PinDescriptor.DataRangesCount = 1;
//             Pins[Index].PinDescriptor.Communication = KSPIN_COMMUNICATION_BRIDGE;
//             Pins[Index].PinDescriptor.Category = UsbAudioGetPinCategoryFromTerminalDescriptor(TerminalDescriptor);
//
//             if (TerminalDescriptor->bDescriptorSubtype == USB_AUDIO_INPUT_TERMINAL)
//             {
//                 Pins[Index].PinDescriptor.DataFlow = KSPIN_DATAFLOW_IN;
//             }
//             else if (TerminalDescriptor->bDescriptorSubtype == USB_AUDIO_OUTPUT_TERMINAL)
//             {
//                 Pins[Index].PinDescriptor.DataFlow = KSPIN_DATAFLOW_OUT;
//             }
//         }
//
//     }
//
//     *PinDescriptors = Pins;
//     *PinDescriptorSize = sizeof(KSPIN_DESCRIPTOR_EX);
//     *PinDescriptorsCount = TotalTerminalDescriptorCount;
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// USBAudioGetDescriptor(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN UCHAR DescriptorType,
//     IN ULONG DescriptorLength,
//     IN UCHAR DescriptorIndex,
//     IN LANGID LanguageId,
//     OUT PVOID *OutDescriptor)
// {
//     PURB Urb;
//     NTSTATUS Status;
//     PVOID Descriptor;
//
//     /* sanity checks */
//     ASSERT(DeviceObject);
//     ASSERT(OutDescriptor);
//     ASSERT(DescriptorLength);
//
//     //
//     // first allocate descriptor buffer
//     //
//     Descriptor = AllocFunction(DescriptorLength);
//     if (!Descriptor)
//     {
//         /* no memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* allocate urb */
//     Urb = (PURB)AllocFunction(sizeof(URB));
//     if (!Urb)
//     {
//         /* no memory */
//         FreeFunction(Descriptor);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* initialize urb */
//     UsbBuildGetDescriptorRequest(Urb,
//         sizeof(Urb->UrbControlDescriptorRequest),
//         DescriptorType,
//         DescriptorIndex,
//         LanguageId,
//         Descriptor,
//         NULL,
//         DescriptorLength,
//         NULL);
//
//     /* submit urb */
//     Status = SubmitUrbSync(DeviceObject, Urb);
//
//     /* free urb */
//     FreeFunction(Urb);
//
//     if (NT_SUCCESS(Status))
//     {
//         /* store result */
//         *OutDescriptor = Descriptor;
//     }
//     else
//     {
//         /* failed */
//         FreeFunction(Descriptor);
//     }
//
//     /* done */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBAudioGetStringDescriptor(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN ULONG DescriptorLength,
//     IN UCHAR DescriptorIndex,
//     IN LANGID LanguageId,
//     OUT PVOID *OutDescriptor)
// {
//     NTSTATUS Status;
//
//     /* retrieve descriptor */
//     Status = USBAudioGetDescriptor(DeviceObject, USB_STRING_DESCRIPTOR_TYPE, DescriptorLength, DescriptorIndex, LanguageId, OutDescriptor);
//     if (!NT_SUCCESS(Status))
//     {
//         // failed
//         return Status;
//     }
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// USBAudioRegCreateMediaCategoriesKey(
//     IN PUNICODE_STRING Name,
//     OUT PHANDLE OutHandle)
// {
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING DestinationString;
//     HANDLE Handle;
//
//     /* initialize root name*/
//     RtlInitUnicodeString(&DestinationString, L"\\Registry\\Machine\\SYSTEM\\CurrentControlSet\\Control\\MediaCategories\\");
//
//     /* initialize object attributes */
//     InitializeObjectAttributes(&ObjectAttributes, &DestinationString, OBJ_CASE_INSENSITIVE | OBJ_OPENIF | OBJ_KERNEL_HANDLE, NULL, NULL);
//
//     /* create the key */
//     Status = ZwOpenKey(&Handle, KEY_ALL_ACCESS, &ObjectAttributes);
//     if (NT_SUCCESS(Status))
//     {
//         /* initialize object attributes */
//         InitializeObjectAttributes(&ObjectAttributes, Name, OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, Handle, NULL);
//
//         Status = ZwCreateKey(OutHandle, KEY_ALL_ACCESS, &ObjectAttributes, 0, NULL, 0, NULL);
//         ZwClose(Handle);
//
//     }
//     return Status;
// }
//
//
// NTSTATUS
// USBAudioInitComponentId(
//     PKSDEVICE Device,
//     IN PKSCOMPONENTID ComponentId)
// {
//     PDEVICE_EXTENSION DeviceExtension;
//     NTSTATUS Status;
//     LPWSTR DescriptionBuffer;
//     UNICODE_STRING GuidString;
//     UNICODE_STRING Name;
//     HANDLE hKey;
//     GUID TempGuid;
//
//     /* get device extension */
//     DeviceExtension = Device->Context;
//
//     /* init component id */
//     ComponentId->Component = KSCOMPONENTID_USBAUDIO;
//     ComponentId->Version = HIBYTE(DeviceExtension->DeviceDescriptor->bcdDevice);
//     ComponentId->Revision = LOBYTE(DeviceExtension->DeviceDescriptor->bcdDevice);
//
//     INIT_USBAUDIO_MID(&ComponentId->Manufacturer, DeviceExtension->DeviceDescriptor->idVendor);
//     INIT_USBAUDIO_PID(&ComponentId->Product, DeviceExtension->DeviceDescriptor->idProduct);
//     INIT_USBAUDIO_PRODUCT_NAME(&TempGuid, DeviceExtension->DeviceDescriptor->idVendor, DeviceExtension->DeviceDescriptor->idProduct, 0);
//
//     if (DeviceExtension->DeviceDescriptor->iProduct)
//     {
//         Status = USBAudioGetStringDescriptor(DeviceExtension->LowerDevice, 100 * sizeof(WCHAR), DeviceExtension->DeviceDescriptor->iProduct, 0x0409 /* FIXME */, (PVOID*)&DescriptionBuffer);
//         if (NT_SUCCESS(Status))
//         {
//             Status = RtlStringFromGUID(&TempGuid, &GuidString);
//             if (NT_SUCCESS(Status))
//             {
//                 Status = USBAudioRegCreateMediaCategoriesKey(&GuidString, &hKey);
//                 if (NT_SUCCESS(Status))
//                 {
//                     RtlInitUnicodeString(&Name, L"Name");
//                     ZwSetValueKey(hKey, &Name, 0, REG_SZ, DescriptionBuffer, (wcslen(DescriptionBuffer) + 1) * sizeof(WCHAR));
//                     ZwClose(hKey);
//
//                     INIT_USBAUDIO_PRODUCT_NAME(&ComponentId->Name, DeviceExtension->DeviceDescriptor->idVendor, DeviceExtension->DeviceDescriptor->idProduct, 0);
//                 }
//                 RtlFreeUnicodeString(&GuidString);
//             }
//             FreeFunction(DescriptionBuffer);
//         }
//     }
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// NTAPI
// USBAudioCreateFilterContext(
//     PKSDEVICE Device)
// {
//     PKSFILTER_DESCRIPTOR FilterDescriptor;
//     PKSCOMPONENTID ComponentId;
//     NTSTATUS Status;
//
//     /* allocate descriptor */
//     FilterDescriptor = AllocFunction(sizeof(KSFILTER_DESCRIPTOR));
//     if (!FilterDescriptor)
//     {
//         /* no memory */
//         return USBD_STATUS_INSUFFICIENT_RESOURCES as NTSTATUS;
//     }
//
//     /* init filter descriptor*/
//     FilterDescriptor->Version = KSFILTER_DESCRIPTOR_VERSION;
//     FilterDescriptor->Flags = 0;
//     FilterDescriptor->ReferenceGuid = &KSNAME_Filter;
//     FilterDescriptor->Dispatch = &USBAudioFilterDispatch;
//     FilterDescriptor->CategoriesCount = 1;
//     FilterDescriptor->Categories = &GUID_KSCATEGORY_AUDIO;
//
//     /* init component id*/
//     ComponentId = AllocFunction(sizeof(KSCOMPONENTID));
//     if (!ComponentId)
//     {
//         /* no memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     Status = USBAudioInitComponentId(Device, ComponentId);
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed*/
//         FreeFunction(ComponentId);
//         return Status;
//     }
//     FilterDescriptor->ComponentId = ComponentId;
//
//     /* build pin descriptors */
//     Status = USBAudioPinBuildDescriptors(Device, (PKSPIN_DESCRIPTOR_EX *)&FilterDescriptor->PinDescriptors, &FilterDescriptor->PinDescriptorsCount, &FilterDescriptor->PinDescriptorSize);
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed*/
//         FreeFunction(ComponentId);
//         return Status;
//     }
//
//     /* build topology */
//     Status = BuildUSBAudioFilterTopology(Device, FilterDescriptor);
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed*/
//         FreeFunction(ComponentId);
//         return Status;
//     }
//
//     /* lets create the filter */
//     Status = KsCreateFilterFactory(Device->FunctionalDeviceObject, FilterDescriptor, ReferenceString, NULL, KSCREATE_ITEM_FREEONSTOP, NULL, NULL, NULL);
//     DPRINT("KsCreateFilterFactory: %x\n", Status);
//
//     return Status;
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
