use crate::evr::evr::evr;
use crate::prelude::*;

/// dxva2api.h's inline: alpha 1.0 as a DXVA2_Fixed32 (ll = 1 << 16).
unsafe fn DXVA2_Fixed32OpaqueAlpha() -> DXVA2_Fixed32 {
    let mut f32_: DXVA2_Fixed32 = core::mem::zeroed();
    f32_.__bindgen_anon_1.ll = 1 << 16;
    f32_
}

/// wine/debug.h's inline.
unsafe fn wine_dbgstr_longlong(ll: ULONGLONG) -> *const ::core::ffi::c_char {
    if ll >> 32 != 0 {
        wine_dbg_sprintf(b"%lx%08lx\0".as_ptr() as *const ::core::ffi::c_char, (ll >> 32) as u32, ll as u32)
    } else {
        wine_dbg_sprintf(b"%lx\0".as_ptr() as *const ::core::ffi::c_char, ll as u32)
    }
}
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut IID_IMFVideoMixerBitmap: GUID = _GUID {
    Data1: 0x814c7b20 as ::core::ffi::c_uint,
    Data2: 0xfdb as ::core::ffi::c_ushort,
    Data3: 0x4eec as ::core::ffi::c_ushort,
    Data4: [
        0xaf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x57 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdc as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut IID_IMFVideoProcessor: GUID = _GUID {
    Data1: 0x6ab0000c as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xfece as ::core::ffi::c_ushort,
    Data3: 0x4d1f as ::core::ffi::c_ushort,
    Data4: [
        0xa2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xac as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x57 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x35 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x30 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x65 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

// #define COBJMACROS  (preprocessor flag, no Rust declaration)
// WINE_DEFAULT_DEBUG_CHANNEL(evr)  (debug channel declaration, handled by the crate's DPRINT macros)

pub const MAX_MIXER_INPUT_SUBSTREAMS: usize = 15;

pub const MAX_MIXER_INPUT_STREAMS: usize = MAX_MIXER_INPUT_SUBSTREAMS + 1;

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct input_stream {
    pub id: u32,
    pub attributes: *mut IMFAttributes,
    pub media_type: *mut IMFMediaType,
    pub rect: MFVideoNormalizedRect,
    pub zorder: u32,
    pub aperture: MFVideoArea,
    pub sample: *mut IMFSample,
    // unsigned int sample_requested : 1;  (bitfield, emitted as its own u32 unit)
    pub sample_requested: u32,
}
pub type Pinput_stream = *mut input_stream;

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct rt_format {
    pub device: GUID,
    pub format: D3DFORMAT,
    pub media_type: *mut IMFMediaType,
}
pub type Prt_format = *mut rt_format;

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct output_stream {
    pub media_type: *mut IMFMediaType,
    pub rt_formats: *mut rt_format,
    pub rt_formats_count: u32,
}
pub type Poutput_stream = *mut output_stream;

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct video_mixer_bkgnd_color {
    pub rgba: COLORREF,
    pub ayuv: DXVA2_AYUVSample16,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct video_mixer {
    pub IMFTransform_iface: IMFTransform,
    pub IMFVideoDeviceID_iface: IMFVideoDeviceID,
    pub IMFTopologyServiceLookupClient_iface: IMFTopologyServiceLookupClient,
    pub IMFVideoMixerControl2_iface: IMFVideoMixerControl2,
    pub IMFGetService_iface: IMFGetService,
    pub IMFVideoMixerBitmap_iface: IMFVideoMixerBitmap,
    pub IMFVideoPositionMapper_iface: IMFVideoPositionMapper,
    pub IMFVideoProcessor_iface: IMFVideoProcessor,
    pub IMFAttributes_iface: IMFAttributes,
    pub IMFQualityAdvise_iface: IMFQualityAdvise,
    pub IMFClockStateSink_iface: IMFClockStateSink,
    pub IUnknown_inner: IUnknown,
    pub outer_unk: *mut IUnknown,
    pub refcount: LONG,

    pub inputs: [input_stream; MAX_MIXER_INPUT_STREAMS],
    pub input_ids: [u32; MAX_MIXER_INPUT_STREAMS],
    pub zorder: [*mut input_stream; MAX_MIXER_INPUT_STREAMS],
    pub input_count: u32,
    pub output: output_stream,

    pub device_manager: *mut IDirect3DDeviceManager9,
    pub processor: *mut IDirectXVideoProcessor,
    pub device_handle: HANDLE,

    pub event_sink: *mut IMediaEventSink,
    pub attributes: *mut IMFAttributes,
    pub internal_attributes: *mut IMFAttributes,
    pub mixing_flags: u32,
    // unsigned int is_streaming : 1;  (bitfield, emitted as its own u32 unit)
    pub is_streaming: u32,
    // unsigned int output_rendered : 1;  (bitfield, emitted as its own u32 unit)
    pub output_rendered: u32,
    pub bkgnd_color: video_mixer_bkgnd_color,
    pub lower_bound: LONGLONG,
    pub upper_bound: LONGLONG,
    pub cs: CRITICAL_SECTION,
}
pub type Pvideo_mixer = *mut video_mixer;

// static const IUnknownVtbl video_mixer_inner_vtbl        -- skipped (COM vtable)
// static const IMFTransformVtbl video_mixer_transform_vtbl -- skipped (COM vtable)
// static const IMFVideoDeviceIDVtbl video_mixer_device_id_vtbl -- skipped (COM vtable)
// static const IMFTopologyServiceLookupClientVtbl video_mixer_service_client_vtbl -- skipped (COM vtable)
// static const IMFVideoMixerControl2Vtbl video_mixer_control_vtbl -- skipped (COM vtable)
// static const IMFGetServiceVtbl video_mixer_getservice_vtbl -- skipped (COM vtable)
// static const IMFVideoMixerBitmapVtbl video_mixer_bitmap_vtbl -- skipped (COM vtable)
// static const IMFVideoPositionMapperVtbl video_mixer_position_mapper_vtbl -- skipped (COM vtable)
// static const IMFVideoProcessorVtbl video_mixer_processor_vtbl -- skipped (COM vtable)
// static const IMFAttributesVtbl video_mixer_attributes_vtbl -- skipped (COM vtable)
// static const IMFQualityAdviseVtbl video_mixer_quality_advise_vtbl -- skipped (COM vtable)
// static const IMFClockStateSinkVtbl video_mixer_clock_state_sink_vtbl -- skipped (COM vtable)

pub const __WINE_DBCL_TRACE: __wine_debug_class = 3; // __WINE_DBCL_TRACE

unsafe extern "C" fn impl_from_IUnknown(iface: *mut IUnknown) -> *mut video_mixer {
    return CONTAINING_RECORD!(iface, video_mixer, IUnknown_inner);
}

unsafe extern "C" fn impl_from_IMFTransform(iface: *mut IMFTransform) -> *mut video_mixer {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(video_mixer, IMFTransform_iface)) as *mut video_mixer;
}

unsafe extern "C" fn impl_from_IMFVideoDeviceID(iface: *mut IMFVideoDeviceID) -> *mut video_mixer {
    return (iface as *mut u8).sub(core::mem::offset_of!(video_mixer, IMFVideoDeviceID_iface)) as *mut video_mixer;
}

unsafe extern "C" fn impl_from_IMFTopologyServiceLookupClient(iface: *mut IMFTopologyServiceLookupClient) -> *mut video_mixer {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(video_mixer, IMFTopologyServiceLookupClient_iface)) as *mut video_mixer;
}

unsafe extern "C" fn impl_from_IMFVideoMixerControl2(iface: *mut IMFVideoMixerControl2) -> *mut video_mixer {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(video_mixer, IMFVideoMixerControl2_iface)) as *mut video_mixer;
}

unsafe extern "C" fn impl_from_IMFGetService(iface: *mut IMFGetService) -> *mut video_mixer {
    return CONTAINING_RECORD!(iface, video_mixer, IMFGetService_iface);
}

unsafe extern "C" fn impl_from_IMFVideoMixerBitmap(
    iface: *mut IMFVideoMixerBitmap,
) -> *mut video_mixer {
    return CONTAINING_RECORD!(iface, video_mixer, IMFVideoMixerBitmap_iface);
}

unsafe extern "C" fn impl_from_IMFVideoPositionMapper(
    iface: *mut IMFVideoPositionMapper,
) -> *mut video_mixer {
    return (iface as *mut u8).wrapping_sub(
        core::mem::offset_of!(video_mixer, IMFVideoPositionMapper_iface),
    ) as *mut video_mixer;
}

unsafe extern "C" fn impl_from_IMFVideoProcessor(iface: *mut IMFVideoProcessor) -> *mut video_mixer {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(video_mixer, IMFVideoProcessor_iface)) as *mut video_mixer;
}

unsafe extern "C" fn impl_from_IMFAttributes(iface: *mut IMFAttributes) -> *mut video_mixer {
    return CONTAINING_RECORD!(iface, video_mixer, IMFAttributes_iface);
}

unsafe extern "C" fn impl_from_IMFQualityAdvise(iface: *mut IMFQualityAdvise) -> *mut video_mixer {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(video_mixer, IMFQualityAdvise_iface)) as *mut video_mixer;
}

unsafe extern "C" fn impl_from_IMFClockStateSink(iface: *mut IMFClockStateSink) -> *mut video_mixer {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(video_mixer, IMFClockStateSink_iface)) as *mut video_mixer;
}

unsafe extern "C" fn video_mixer_compare_input_id(
    a: *const ::core::ffi::c_void,
    b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let key: *const ::core::ffi::c_uint = a as *const ::core::ffi::c_uint;
    let input: *const input_stream = b as *const input_stream;

    if *key > (*input).id {
        return 1;
    }
    if *key < (*input).id {
        return -1;
    }
    return 0;
}

unsafe extern "C" fn video_mixer_get_input(mixer: *const video_mixer, id: UINT32, stream: *mut *mut input_stream) -> HRESULT {
    *stream = bsearch(
        core::ptr::addr_of!(id) as *const core::ffi::c_void,
        core::ptr::addr_of!((*mixer).inputs) as *const input_stream as *const core::ffi::c_void,
        (*mixer).input_count as usize,
        core::mem::size_of::<input_stream>(),
        Some(
            video_mixer_compare_input_id
                as unsafe extern "C" fn(
                    *const core::ffi::c_void,
                    *const core::ffi::c_void,
                ) -> core::ffi::c_int,
        ),
    ) as *mut input_stream;
    return if !(*stream).is_null() {
        S_OK
    } else {
        MF_E_INVALIDSTREAMNUMBER
    };
}

unsafe extern "C" fn video_mixer_init_input(stream: *mut input_stream) {
    if (MFCreateAttributes(&raw mut (*stream).attributes, 1)) >= 0 {
        IMFAttributes_SetUINT32((*stream).attributes, &raw const MF_SA_REQUIRED_SAMPLE_COUNT, 1);
    }
    (*stream).rect.top = 0.0f32;
    (*stream).rect.left = (*stream).rect.top;
    (*stream).rect.bottom = 1.0f32;
    (*stream).rect.right = (*stream).rect.bottom;
}

unsafe extern "C" fn video_mixer_zorder_sort_compare(a: *const core::ffi::c_void, b: *const core::ffi::c_void) -> core::ffi::c_int {
    let left: *const input_stream = *(a as *const *const input_stream);
    let right: *const input_stream = *(b as *const *const input_stream);
    return if (*left).zorder != (*right).zorder {
        if (*left).zorder < (*right).zorder {
            -1
        } else {
            1
        }
    } else {
        0
    };
}

unsafe extern "C" fn video_mixer_update_zorder_map(mixer: *mut video_mixer) {
    let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    while i < (*mixer).input_count {
        (*mixer).zorder[i as usize] =
            (core::ptr::addr_of_mut!((*mixer).inputs) as *mut input_stream).add(i as usize);
        i = i.wrapping_add(1);
    }
    qsort(
        core::ptr::addr_of_mut!((*mixer).zorder) as *mut ::core::ffi::c_void,
        (*mixer).input_count as usize,
        core::mem::size_of::<input_stream>(),
        Some(
            video_mixer_zorder_sort_compare
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
}

unsafe extern "C" fn video_mixer_flush_input(mixer: *mut video_mixer) {
    let mut i: ::core::ffi::c_uint = 0;
    while i < (*mixer).input_count {
        if !(*mixer).inputs[i as usize].sample.is_null() {
            IMFSample_Release((*mixer).inputs[i as usize].sample);
        }
        (*mixer).inputs[i as usize].sample = ::core::ptr::null_mut::<IMFSample>();
        (*mixer).inputs[i as usize].sample_requested = 0;
        i = i.wrapping_add(1);
    }
    (*mixer).output_rendered = 0;
}

unsafe extern "C" fn video_mixer_clear_types(mut mixer: *mut video_mixer) {
    let mut i: ::core::ffi::c_uint = 0;
    while i < (*mixer).input_count {
        if !(*mixer).inputs[i as usize].media_type.is_null() {
            IMFMediaType_Release((*mixer).inputs[i as usize].media_type);
        }
        (*mixer).inputs[i as usize].media_type = core::ptr::null_mut::<IMFMediaType>();
        i += 1;
    }
    video_mixer_flush_input(mixer);
    i = 0;
    while i < (*mixer).output.rt_formats_count {
        IMFMediaType_Release((*(*mixer).output.rt_formats.offset(i as isize)).media_type);
        i += 1;
    }
    free((*mixer).output.rt_formats as *mut ::core::ffi::c_void);
    if !(*mixer).output.media_type.is_null() {
        IMFMediaType_Release((*mixer).output.media_type);
    }
    memset(
        core::ptr::addr_of_mut!((*mixer).output) as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        core::mem::size_of::<output_stream>() as size_t,
    );
}
unsafe extern "system" fn video_mixer_inner_QueryInterface(
    iface: *mut IUnknown,
    riid: REFIID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IUnknown(iface);

    if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown))) != 0) {
        *obj = iface as *mut core::ffi::c_void;
    } else if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFTransform))) != 0) {
        *obj = core::ptr::addr_of_mut!((*mixer).IMFTransform_iface) as *mut core::ffi::c_void;
    } else if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFVideoDeviceID))) != 0) {
        *obj = core::ptr::addr_of_mut!((*mixer).IMFVideoDeviceID_iface) as *mut core::ffi::c_void;
    } else if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFTopologyServiceLookupClient))) != 0) {
        *obj = core::ptr::addr_of_mut!((*mixer).IMFTopologyServiceLookupClient_iface)
            as *mut core::ffi::c_void;
    } else if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFVideoMixerControl2))) != 0)
        || ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFVideoMixerControl))) != 0)
    {
        *obj = core::ptr::addr_of_mut!((*mixer).IMFVideoMixerControl2_iface) as *mut core::ffi::c_void;
    } else if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFGetService))) != 0) {
        *obj = core::ptr::addr_of_mut!((*mixer).IMFGetService_iface) as *mut core::ffi::c_void;
    } else if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFVideoMixerBitmap))) != 0) {
        *obj = core::ptr::addr_of_mut!((*mixer).IMFVideoMixerBitmap_iface) as *mut core::ffi::c_void;
    } else if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFVideoPositionMapper))) != 0) {
        *obj = core::ptr::addr_of_mut!((*mixer).IMFVideoPositionMapper_iface)
            as *mut core::ffi::c_void;
    } else if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFVideoProcessor))) != 0) {
        *obj = core::ptr::addr_of_mut!((*mixer).IMFVideoProcessor_iface) as *mut core::ffi::c_void;
    } else if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAttributes))) != 0) {
        *obj = core::ptr::addr_of_mut!((*mixer).IMFAttributes_iface) as *mut core::ffi::c_void;
    } else if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFQualityAdvise))) != 0) {
        *obj = core::ptr::addr_of_mut!((*mixer).IMFQualityAdvise_iface) as *mut core::ffi::c_void;
    } else if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFClockStateSink))) != 0) {
        *obj = core::ptr::addr_of_mut!((*mixer).IMFClockStateSink_iface) as *mut core::ffi::c_void;
    } else {
        WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
        *obj = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*obj as *mut IUnknown);
    return S_OK;
}

unsafe extern "system" fn video_mixer_inner_AddRef(iface: *mut IUnknown) -> ULONG {
    let mixer: *mut video_mixer = impl_from_IUnknown(iface);
    let refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*mixer).refcount) as *mut core::ffi::c_long,
    ) as ULONG;

    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"video_mixer_inner_AddRef\0" as *const u8 as *const core::ffi::c_char,
            316 as core::ffi::c_int,
            b"%p, refcount %lu.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            refcount,
        );
    }

    return refcount;
}

unsafe extern "C" fn video_mixer_release_device_manager(mixer: *mut video_mixer) {
    if !(*mixer).processor.is_null() {
        IDirectXVideoProcessor_Release((*mixer).processor);
    }
    if !(*mixer).device_manager.is_null() {
        IDirect3DDeviceManager9_CloseDeviceHandle((*mixer).device_manager, (*mixer).device_handle);
        IDirect3DDeviceManager9_Release((*mixer).device_manager);
    }
    (*mixer).device_handle = core::ptr::null_mut();
    (*mixer).device_manager = core::ptr::null_mut();
    (*mixer).processor = core::ptr::null_mut();
}

unsafe extern "system" fn video_mixer_inner_Release(iface: *mut IUnknown) -> ULONG {
    let mixer: *mut video_mixer = impl_from_IUnknown(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*mixer).refcount) as *mut LONG) as ULONG;
    let mut i: ::core::ffi::c_uint;

    if refcount == 0 {
        i = 0;
        while i < (*mixer).input_count {
            if !(*mixer).inputs[i as usize].attributes.is_null() {
                IMFAttributes_Release((*mixer).inputs[i as usize].attributes);
            }
            i = i.wrapping_add(1);
        }
        video_mixer_clear_types(mixer);
        video_mixer_release_device_manager(mixer);
        if !(*mixer).attributes.is_null() {
            IMFAttributes_Release((*mixer).attributes);
        }
        if !(*mixer).internal_attributes.is_null() {
            IMFAttributes_Release((*mixer).internal_attributes);
        }
        DeleteCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));
        free(mixer as *mut ::core::ffi::c_void);
    }

    return refcount;
}
unsafe extern "system" fn video_mixer_transform_QueryInterface(iface: *mut IMFTransform, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFTransform(iface);
    return IUnknown_QueryInterface((*mixer).outer_unk, riid, obj);
}
unsafe extern "system" fn video_mixer_transform_AddRef(iface: *mut IMFTransform) -> ULONG {
    let mixer: *mut video_mixer = impl_from_IMFTransform(iface);
    return IUnknown_AddRef((*mixer).outer_unk);
}

unsafe extern "system" fn video_mixer_transform_Release(iface: *mut IMFTransform) -> ULONG {
    let mixer: *mut video_mixer = impl_from_IMFTransform(iface);
    return IUnknown_Release((*mixer).outer_unk);
}

unsafe extern "system" fn video_mixer_transform_GetStreamLimits(
    iface: *mut IMFTransform,
    input_minimum: *mut DWORD,
    input_maximum: *mut DWORD,
    output_minimum: *mut DWORD,
    output_maximum: *mut DWORD,
) -> HRESULT {
    TRACE!(
        "%p, %p, %p, %p, %p.\n",
        iface,
        input_minimum,
        input_maximum,
        output_minimum,
        output_maximum
    );

    *input_minimum = 1;
    *input_maximum = (MAX_MIXER_INPUT_STREAMS) as _;
    *output_minimum = 1;
    *output_maximum = 1;

    return S_OK;
}

unsafe extern "system" fn video_mixer_transform_GetStreamCount(iface: *mut IMFTransform, inputs: *mut DWORD, outputs: *mut DWORD) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFTransform(iface);
    TRACE!("%p, %p, %p.\n", iface, inputs, outputs);

    EnterCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));
    if !inputs.is_null() {
        *inputs = (*mixer).input_count as DWORD;
    }
    if !outputs.is_null() {
        *outputs = 1;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    return S_OK;
}

unsafe extern "system" fn video_mixer_transform_GetStreamIDs(
    iface: *mut IMFTransform,
    input_size: DWORD,
    inputs: *mut DWORD,
    output_size: DWORD,
    outputs: *mut DWORD,
) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFTransform(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!(
        "%p, %lu, %p, %lu, %p.\n",
        iface,
        input_size,
        inputs,
        output_size,
        outputs
    );

    EnterCriticalSection(&raw mut (*mixer).cs);
    if (*mixer).input_count as DWORD > input_size || output_size == 0 {
        hr = MF_E_BUFFERTOOSMALL;
    } else if !inputs.is_null() {
        memcpy(
            inputs as *mut core::ffi::c_void,
            &raw mut (*mixer).input_ids as *mut core::ffi::c_uint as *const core::ffi::c_void,
            ((*mixer).input_count as usize).wrapping_mul(core::mem::size_of::<DWORD>()),
        );
    }
    if !outputs.is_null() {
        *outputs = 0;
    }
    LeaveCriticalSection(&raw mut (*mixer).cs);

    return hr;
}

unsafe extern "system" fn video_mixer_transform_GetInputStreamInfo(iface: *mut IMFTransform, id: DWORD, info: *mut MFT_INPUT_STREAM_INFO) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFTransform(iface);
    let mut input: *mut input_stream = core::ptr::null_mut::<input_stream>();
    let mut hr: HRESULT;

    EnterCriticalSection(&raw mut (*mixer).cs);

    hr = video_mixer_get_input(mixer, id as ::core::ffi::c_uint, &raw mut input);
    if hr >= 0 {
        memset(
            info as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            core::mem::size_of::<MFT_INPUT_STREAM_INFO>() as size_t,
        );
        if id != 0 {
            (*info).dwFlags |= (MFT_INPUT_STREAM_REMOVABLE | MFT_INPUT_STREAM_OPTIONAL) as DWORD;
        }
    }

    LeaveCriticalSection(&raw mut (*mixer).cs);

    return hr;
}

unsafe extern "system" fn video_mixer_transform_GetOutputStreamInfo(iface: *mut IMFTransform, id: DWORD, info: *mut MFT_OUTPUT_STREAM_INFO) -> HRESULT {
    if id != 0 {
        return MF_E_INVALIDSTREAMNUMBER;
    }
    memset(info as *mut core::ffi::c_void, 0, core::mem::size_of::<MFT_OUTPUT_STREAM_INFO>());
    return S_OK;
}

unsafe extern "system" fn video_mixer_transform_GetAttributes(iface: *mut IMFTransform, attributes: *mut *mut IMFAttributes) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFTransform(iface);

    if attributes.is_null() {
        return E_POINTER;
    }

    *attributes = (*mixer).attributes;
    IMFAttributes_AddRef(*attributes);

    return S_OK;
}

unsafe extern "system" fn video_mixer_transform_GetInputStreamAttributes(
    mut iface: *mut IMFTransform,
    mut id: DWORD,
    mut attributes: *mut *mut IMFAttributes,
) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFTransform(iface);
    let mut input: *mut input_stream = core::ptr::null_mut::<input_stream>();
    let mut hr: HRESULT = 0;
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0"
                as *const u8 as *const i8,
            b"video_mixer_transform_GetInputStreamAttributes\0" as *const u8
                as *const i8,
            490 as i32,
            b"%p, %lu, %p.\n\0" as *const u8 as *const i8,
            iface,
            id,
            attributes,
        );
    }
    EnterCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));
    hr = video_mixer_get_input(mixer, id as u32, &mut input);
    if hr >= 0 {
        *attributes = (*input).attributes;
        if !(*attributes).is_null() {
            IMFAttributes_AddRef(*attributes);
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));
    return hr;
}
unsafe extern "system" fn video_mixer_transform_GetOutputStreamAttributes(
    iface: *mut IMFTransform,
    id: DWORD,
    attributes: *mut *mut IMFAttributes,
) -> HRESULT {
    // TRACE("%p, %lu, %p.\n", iface, id, attributes);
    E_NOTIMPL
}

unsafe extern "system" fn video_mixer_transform_DeleteInputStream(iface: *mut IMFTransform, id: DWORD) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFTransform(iface);
    let mut input: *mut input_stream = core::ptr::null_mut();
    let mut idx: u32 = 0;
    let mut hr: HRESULT = 0;

    if id == 0 {
        return MF_E_INVALIDSTREAMNUMBER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    /* Can't delete reference stream. */
    hr = video_mixer_get_input(mixer, id, core::ptr::addr_of_mut!(input));
    if hr >= 0 {
        (*mixer).input_count = (*mixer).input_count.wrapping_sub(1);
        idx = input.offset_from(core::ptr::addr_of_mut!((*mixer).inputs) as *mut input_stream) as core::ffi::c_longlong as u32;
        if idx < (*mixer).input_count {
            if !(*mixer).inputs[idx as usize].attributes.is_null() {
                IMFAttributes_Release((*mixer).inputs[idx as usize].attributes);
            }
            memmove(
                (core::ptr::addr_of_mut!((*mixer).inputs) as *mut input_stream).offset(idx as isize) as *mut core::ffi::c_void,
                (core::ptr::addr_of_mut!((*mixer).inputs) as *mut input_stream).offset(idx.wrapping_add(1) as isize) as *const core::ffi::c_void,
                ((*mixer).input_count.wrapping_sub(idx) as usize).wrapping_mul(core::mem::size_of::<input_stream>()),
            );
            memmove(
                (core::ptr::addr_of_mut!((*mixer).input_ids) as *mut u32).offset(idx as isize) as *mut core::ffi::c_void,
                (core::ptr::addr_of_mut!((*mixer).input_ids) as *mut u32).offset(idx.wrapping_add(1) as isize) as *const core::ffi::c_void,
                ((*mixer).input_count.wrapping_sub(idx) as usize).wrapping_mul(core::mem::size_of::<u32>()),
            );
            video_mixer_update_zorder_map(mixer);
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    return hr;
}

unsafe extern "C" fn video_mixer_add_input_sort_compare(
    a: *const ::core::ffi::c_void,
    b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let left: *const input_stream = a as *const input_stream;
    let right: *const input_stream = b as *const input_stream;
    return if (*left).id != (*right).id {
        if (*left).id < (*right).id {
            -1
        } else {
            1
        }
    } else {
        0
    };
}

unsafe extern "system" fn video_mixer_transform_AddInputStreams(
    iface: *mut IMFTransform,
    count: DWORD,
    ids: *mut DWORD,
) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFTransform(iface);
    let mut inputs: [input_stream; MAX_MIXER_INPUT_STREAMS] = {
        let _ = 0;
        core::mem::zeroed()
    };
    let mut input: *mut input_stream = core::ptr::null_mut::<input_stream>();
    let mut i: u32 = 0;
    let mut len: u32 = 0;
    let mut hr: HRESULT = S_OK;

    if ids.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(&raw mut (*mixer).cs);
    if count > (ARRAY_SIZE!((*mixer).inputs) as u32).wrapping_sub((*mixer).input_count) {
        hr = E_INVALIDARG;
    } else {
        /* Test for collisions. */
        memcpy(
            &raw mut inputs as *mut input_stream as *mut core::ffi::c_void,
            &raw mut (*mixer).inputs as *mut input_stream as *const core::ffi::c_void,
            ((*mixer).input_count as usize)
                .wrapping_mul(core::mem::size_of::<input_stream>() as usize),
        );
        i = 0;
        while i < count {
            inputs[i.wrapping_add((*mixer).input_count) as usize].id = *ids.offset(i as isize);
            i = i.wrapping_add(1);
        }

        len = (*mixer).input_count.wrapping_add(count);

        qsort(
            &raw mut inputs as *mut input_stream as *mut core::ffi::c_void,
            len as usize,
            core::mem::size_of::<input_stream>() as usize,
            Some(video_mixer_add_input_sort_compare),
        );

        i = 1;
        while i < len {
            if inputs[i.wrapping_sub(1) as usize].id
                == inputs[i as usize].id
            {
                hr = E_INVALIDARG;
                break;
            }
            i = i.wrapping_add(1);
        }

        if hr >= 0 {
            let mut zorder: u32 = (*mixer).input_count;

            i = 0;
            while i < count {
                input = bsearch(
                    ids.offset(i as isize) as *const core::ffi::c_void,
                    &raw mut inputs as *mut input_stream as *const core::ffi::c_void,
                    len as usize,
                    core::mem::size_of::<input_stream>() as usize,
                    Some(video_mixer_compare_input_id),
                ) as *mut input_stream;
                if !input.is_null() {
                    video_mixer_init_input(input);
                }
                i = i.wrapping_add(1);
            }

            memcpy(
                (&raw mut (*mixer).input_ids as *mut u32).offset((*mixer).input_count as isize)
                    as *mut u32 as *mut core::ffi::c_void,
                ids as *const core::ffi::c_void,
                (count as usize).wrapping_mul(core::mem::size_of::<DWORD>() as usize),
            );
            memcpy(
                &raw mut (*mixer).inputs as *mut input_stream as *mut core::ffi::c_void,
                &raw mut inputs as *mut input_stream as *const core::ffi::c_void,
                (len as usize).wrapping_mul(core::mem::size_of::<input_stream>() as usize),
            );
            (*mixer).input_count = (*mixer).input_count.wrapping_add(count);

            i = 0;
            while i < count {
                if video_mixer_get_input(mixer, *ids.offset(i as isize), &raw mut input)
                    >= 0
                {
                    (*input).zorder = zorder;
                }
                zorder = zorder.wrapping_add(1);
                i = i.wrapping_add(1);
            }

            video_mixer_update_zorder_map(mixer);
        }
    }
    LeaveCriticalSection(&raw mut (*mixer).cs);

    return hr;
}
unsafe extern "system" fn video_mixer_transform_GetInputAvailableType(
    iface: *mut IMFTransform,
    id: DWORD,
    index: DWORD,
    r#type: *mut *mut IMFMediaType,
) -> HRESULT {
    // TRACE("%p, %lu, %lu, %p.\n", iface, id, index, type);
    E_NOTIMPL
}
unsafe extern "system" fn video_mixer_transform_GetOutputAvailableType(iface: *mut IMFTransform, id: DWORD, index: DWORD, type_0: *mut *mut IMFMediaType) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFTransform(iface);
    let mut hr: HRESULT = S_OK;

    if id != 0 {
        return MF_E_INVALIDSTREAMNUMBER;
    }

    EnterCriticalSection(&mut (*mixer).cs);

    if (*mixer).inputs[0 as usize].media_type.is_null() {
        hr = MF_E_TRANSFORM_TYPE_NOT_SET;
    } else if index >= (*mixer).output.rt_formats_count as DWORD {
        hr = MF_E_NO_MORE_TYPES;
    } else {
        hr = MFCreateMediaType(type_0);
        if hr >= 0 {
            hr = IMFMediaType_CopyAllItems(
                (*(*mixer).output.rt_formats.offset(index as isize)).media_type,
                *type_0 as *mut IMFAttributes,
            );
        }
    }

    LeaveCriticalSection(&mut (*mixer).cs);

    return hr;
}

unsafe extern "C" fn video_mixer_init_dxva_videodesc(
    mut media_type: *mut IMFMediaType,
    mut video_desc: *mut DXVA2_VideoDesc,
) -> HRESULT {
    let mut video_format: *const MFVIDEOFORMAT = ::core::ptr::null::<MFVIDEOFORMAT>();
    let mut video_type: *mut IMFVideoMediaType = ::core::ptr::null_mut::<IMFVideoMediaType>();
    let mut is_compressed: BOOL = (TRUE as i32);
    let mut hr: HRESULT = S_OK;
    'done: {
        if (IMFMediaType_QueryInterface(
            media_type,
            &raw const IID_IMFVideoMediaType,
            &raw mut video_type as *mut *mut ::core::ffi::c_void,
        )) < 0
        {
            return MF_E_INVALIDMEDIATYPE;
        }

        video_format = IMFVideoMediaType_GetVideoFormat(video_type);
        IMFVideoMediaType_IsCompressedFormat(video_type, &raw mut is_compressed);

        if video_format.is_null()
            || (*video_format).videoInfo.dwWidth == 0
            || (*video_format).videoInfo.dwHeight == 0
            || is_compressed != 0
        {
            hr = MF_E_INVALIDMEDIATYPE;
            break 'done;
        }

        memset(
            video_desc as *mut ::core::ffi::c_void,
            0,
            ::core::mem::size_of::<DXVA2_VideoDesc>() as size_t,
        );
        (*video_desc).SampleWidth = (*video_format).videoInfo.dwWidth as UINT;
        (*video_desc).SampleHeight = (*video_format).videoInfo.dwHeight as UINT;
        (*video_desc).Format = (*video_format).surfaceInfo.Format as D3DFORMAT;
    }
    IMFVideoMediaType_Release(video_type);

    return hr;
}

unsafe extern "C" fn video_mixer_append_rt_format(
    rt_formats: *mut rt_format,
    count: *mut ::core::ffi::c_uint,
    device: *const GUID,
    format: D3DFORMAT,
) {
    let mut i: ::core::ffi::c_uint = 0;
    while i < *count {
        if (*rt_formats.add(i as usize)).format == format {
            return;
        }
        i = i.wrapping_add(1);
    }
    (*rt_formats.add(*count as usize)).format = format;
    (*rt_formats.add(*count as usize)).device = *device;
    *count = (*count).wrapping_add(1);
}

unsafe extern "C" fn video_mixer_get_interlace_mode_from_video_desc(
    mut video_desc: *const DXVA2_VideoDesc,
) -> ::core::ffi::c_uint {
    match (*video_desc)
        .SampleFormat


        .__bindgen_anon_1.SampleFormat() as ::core::ffi::c_int
    {
        3 => {
            // DXVA2_SampleFieldInterleavedEvenFirst
            return MFVideoInterlace_FieldInterleavedUpperFirst as ::core::ffi::c_int
                as ::core::ffi::c_uint;
        }
        4 => {
            // DXVA2_SampleFieldInterleavedOddFirst
            return MFVideoInterlace_FieldInterleavedLowerFirst as ::core::ffi::c_int
                as ::core::ffi::c_uint;
        }
        5 => {
            // DXVA2_SampleFieldSingleEven
            return MFVideoInterlace_FieldSingleUpper as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        6 => {
            // DXVA2_SampleFieldSingleOdd
            return MFVideoInterlace_FieldSingleLower as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        _ => {
            return MFVideoInterlace_Progressive as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
    };
}

unsafe extern "C" fn mf_get_attribute_uint32(media_type: *mut IMFMediaType, key: *const GUID, value: *mut UINT32, default_value: UINT32) {
    if (IMFMediaType_GetUINT32(media_type, key, value)) < 0 {
        *value = default_value;
    }
}

unsafe extern "C" fn mf_get_attribute_uint64(mut media_type: *mut IMFMediaType, mut key: *const GUID, mut value: *mut UINT64, mut default_value: UINT64) {
    if (IMFMediaType_GetUINT64(media_type, key, value)) < 0 {
        *value = default_value;
    }
}

unsafe extern "C" fn video_mixer_collect_output_types(
    mut mixer: *mut video_mixer,
    mut video_desc: *const DXVA2_VideoDesc,
    mut media_type: *mut IMFMediaType,
    mut service: *mut IDirectXVideoProcessorService,
    mut device_count: ::core::ffi::c_uint,
    mut devices: *const GUID,
    mut flags: ::core::ffi::c_uint,
) -> HRESULT {
    let mut rt_formats: *mut rt_format = core::ptr::null_mut::<rt_format>();
    let mut ptr: *mut rt_format;
    let mut i: ::core::ffi::c_uint;
    let mut j: ::core::ffi::c_uint;
    let mut format_count: ::core::ffi::c_uint = core::mem::zeroed();
    let mut count: ::core::ffi::c_uint;
    let mut hr: HRESULT = MF_E_INVALIDMEDIATYPE;
    let mut formats: *mut D3DFORMAT = core::mem::zeroed();
    let mut subtype: GUID = core::mem::zeroed();

    count = 0;
    i = 0;
    while i < device_count {
        if (IDirectXVideoProcessorService_GetVideoProcessorRenderTargets(
            service,
            devices.offset(i as isize) as *const GUID,
            video_desc,
            core::ptr::addr_of_mut!(format_count),
            core::ptr::addr_of_mut!(formats),
        )) >= 0
        {
            ptr = realloc(
                rt_formats as *mut ::core::ffi::c_void,
                (count.wrapping_add(format_count) as size_t)
                    .wrapping_mul(core::mem::size_of::<rt_format>() as size_t),
            ) as *mut rt_format;
            if ptr.is_null() {
                hr = E_OUTOFMEMORY;
                count = 0;
                CoTaskMemFree(formats as LPVOID);
                break;
            } else {
                rt_formats = ptr;
                j = 0;
                while j < format_count {
                    video_mixer_append_rt_format(
                        rt_formats,
                        core::ptr::addr_of_mut!(count),
                        devices.offset(i as isize) as *const GUID,
                        *formats.offset(j as isize),
                    );
                    j = j.wrapping_add(1);
                }
                CoTaskMemFree(formats as LPVOID);
            }
        }
        i = i.wrapping_add(1);
    }

    if count != 0 && (flags & MFT_SET_TYPE_TEST_ONLY as ::core::ffi::c_uint) == 0 {
        let mut fixed_samples: UINT32 = 0;
        let mut interlace_mode: UINT32;
        let mut width: UINT32 = (*video_desc).SampleWidth as UINT32;
        let mut height: UINT32 = (*video_desc).SampleHeight as UINT32;
        let mut aperture: MFVideoArea = core::mem::zeroed();
        let mut par: UINT64 = 0;

        (*mixer).output.rt_formats =
            calloc((count as size_t) as _, (core::mem::size_of::<rt_format>() as size_t) as _)
                as *mut rt_format;
        if (*mixer).output.rt_formats.is_null() {
            free(rt_formats as *mut ::core::ffi::c_void);
            return E_OUTOFMEMORY;
        }

        memcpy(
            core::ptr::addr_of_mut!(subtype) as *mut ::core::ffi::c_void,
            core::ptr::addr_of!(MFVideoFormat_Base) as *const ::core::ffi::c_void,
            core::mem::size_of::<GUID>() as size_t,
        );
        memset(
            core::ptr::addr_of_mut!(aperture) as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            core::mem::size_of::<MFVideoArea>() as size_t,
        );
        if (IMFMediaType_GetBlob(
            media_type,
            core::ptr::addr_of!(MF_MT_GEOMETRIC_APERTURE),
            core::ptr::addr_of_mut!(aperture) as *mut UINT8,
            core::mem::size_of::<MFVideoArea>() as UINT32,
            core::ptr::null_mut::<UINT32>(),
        )) >= 0
        {
            width = (aperture.OffsetX.value as LONG + aperture.Area.cx) as UINT32;
            height = (aperture.OffsetY.value as LONG + aperture.Area.cy) as UINT32;
        } else {
            aperture.Area.cx = width as LONG;
            aperture.Area.cy = height as LONG;
        }

        interlace_mode = video_mixer_get_interlace_mode_from_video_desc(video_desc) as UINT32;
        mf_get_attribute_uint64(
            media_type,
            core::ptr::addr_of!(MF_MT_PIXEL_ASPECT_RATIO),
            core::ptr::addr_of_mut!(par),
            (1 as UINT64) << 32 | 1,
        );
        mf_get_attribute_uint32(
            media_type,
            core::ptr::addr_of!(MF_MT_FIXED_SIZE_SAMPLES),
            core::ptr::addr_of_mut!(fixed_samples),
            1,
        );

        i = 0;
        while i < count {
            let mut rt_media_type: *mut IMFMediaType = core::ptr::null_mut::<IMFMediaType>();
            subtype.Data1 = (*rt_formats.offset(i as isize)).format as ::core::ffi::c_uint;
            *(*mixer).output.rt_formats.offset(i as isize) =
                core::ptr::read(rt_formats.offset(i as isize));

            MFCreateMediaType(core::ptr::addr_of_mut!(rt_media_type));
            IMFMediaType_CopyAllItems(media_type, rt_media_type as *mut IMFAttributes);
            IMFMediaType_SetGUID(
                rt_media_type,
                core::ptr::addr_of!(MF_MT_SUBTYPE),
                core::ptr::addr_of_mut!(subtype),
            );
            IMFMediaType_SetUINT64(
                rt_media_type,
                core::ptr::addr_of!(MF_MT_FRAME_SIZE),
                (width as UINT64) << 32 | height as UINT64,
            );
            IMFMediaType_SetBlob(
                rt_media_type,
                core::ptr::addr_of!(MF_MT_GEOMETRIC_APERTURE),
                core::ptr::addr_of_mut!(aperture) as *const UINT8,
                core::mem::size_of::<MFVideoArea>() as UINT32,
            );
            IMFMediaType_SetBlob(
                rt_media_type,
                core::ptr::addr_of!(MF_MT_MINIMUM_DISPLAY_APERTURE),
                core::ptr::addr_of_mut!(aperture) as *const UINT8,
                core::mem::size_of::<MFVideoArea>() as UINT32,
            );
            IMFMediaType_SetUINT32(
                rt_media_type,
                core::ptr::addr_of!(MF_MT_INTERLACE_MODE),
                interlace_mode,
            );
            IMFMediaType_SetUINT64(
                rt_media_type,
                core::ptr::addr_of!(MF_MT_PIXEL_ASPECT_RATIO),
                par,
            );
            IMFMediaType_SetUINT32(
                rt_media_type,
                core::ptr::addr_of!(MF_MT_FIXED_SIZE_SAMPLES),
                fixed_samples,
            );

            (*(*mixer).output.rt_formats.offset(i as isize)).media_type = rt_media_type;
            i = i.wrapping_add(1);
        }
        (*mixer).output.rt_formats_count = count;
    }

    free(rt_formats as *mut ::core::ffi::c_void);

    return if count != 0 { S_OK } else { hr };
}

unsafe extern "C" fn video_mixer_open_device_handle(mixer: *mut video_mixer) -> HRESULT {
    IDirect3DDeviceManager9_CloseDeviceHandle((*mixer).device_manager, (*mixer).device_handle);
    (*mixer).device_handle = core::ptr::null_mut();
    return IDirect3DDeviceManager9_OpenDeviceHandle((*mixer).device_manager, core::ptr::addr_of_mut!((*mixer).device_handle));
}

unsafe extern "C" fn video_mixer_get_processor_service(mixer: *mut video_mixer, service: *mut *mut IDirectXVideoProcessorService) -> HRESULT {
    let mut hr: HRESULT;

    if (*mixer).device_handle.is_null() {
        hr = IDirect3DDeviceManager9_OpenDeviceHandle((*mixer).device_manager, core::ptr::addr_of_mut!((*mixer).device_handle));
        if hr < 0 {
            return hr;
        }
    }

    loop {
        hr = IDirect3DDeviceManager9_GetVideoService(
            (*mixer).device_manager,
            (*mixer).device_handle,
            core::ptr::addr_of!(IID_IDirectXVideoProcessorService),
            service as *mut *mut core::ffi::c_void,
        );
        if hr == DXVA2_E_NEW_VIDEO_DEVICE {
            hr = video_mixer_open_device_handle(mixer);
            if hr >= 0 {
                continue;
            }
        }
        break;
    }

    return hr;
}

unsafe extern "system" fn video_mixer_transform_SetInputType(iface: *mut IMFTransform, id: DWORD, media_type: *mut IMFMediaType, flags: DWORD) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFTransform(iface);
    let mut service: *mut IDirectXVideoProcessorService = core::ptr::null_mut::<IDirectXVideoProcessorService>();
    let mut video_desc: DXVA2_VideoDesc = core::mem::zeroed::<DXVA2_VideoDesc>();
    let mut hr: HRESULT = E_NOTIMPL;
    let mut count: core::ffi::c_uint = 0;
    let mut guids: *mut GUID = core::ptr::null_mut::<GUID>();

    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"video_mixer_transform_SetInputType\0" as *const u8 as *const core::ffi::c_char,
            861 as core::ffi::c_int,
            b"%p, %lu, %p, %#lx.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            id,
            media_type,
            flags,
        );
    }

    if media_type.is_null() && flags & MFT_SET_TYPE_TEST_ONLY as DWORD != 0 {
        return E_INVALIDARG;
    }

    EnterCriticalSection(&raw mut (*mixer).cs);

    if flags & MFT_SET_TYPE_TEST_ONLY as DWORD == 0 {
        video_mixer_clear_types(mixer);
    }

    if media_type.is_null() {
        hr = S_OK;
    } else if (*mixer).device_manager.is_null() {
        hr = MF_E_NOT_INITIALIZED;
    } else {
        hr = video_mixer_get_processor_service(mixer, &raw mut service);
        if hr >= 0 {
            hr = video_mixer_init_dxva_videodesc(media_type, &raw mut video_desc);
            if hr >= 0 {
                if id == 0 {
                    hr = IDirectXVideoProcessorService_GetVideoProcessorDeviceGuids(
                        service,
                        &raw mut video_desc,
                        &raw mut count,
                        &raw mut guids,
                    );
                    if hr >= 0 {
                        hr = video_mixer_collect_output_types(
                            mixer,
                            &raw mut video_desc,
                            media_type,
                            service,
                            count,
                            guids,
                            flags as core::ffi::c_uint,
                        );
                        if hr >= 0 && flags & MFT_SET_TYPE_TEST_ONLY as DWORD == 0 {
                            if !(*mixer).inputs[0 as usize].media_type.is_null() {
                                IMFMediaType_Release(
                                    (*mixer).inputs[0 as usize].media_type,
                                );
                            }
                            (*mixer).inputs[0 as usize].media_type = media_type;

                            if IMFMediaType_GetBlob(
                                media_type,
                                &raw const MF_MT_GEOMETRIC_APERTURE,
                                &raw mut (*mixer).inputs[0 as usize].aperture as *mut UINT8,
                                core::mem::size_of::<MFVideoArea>() as UINT32,
                                core::ptr::null_mut::<UINT32>(),
                            ) < 0
                            {
                                memset(
                                    &raw mut (*mixer).inputs[0 as usize].aperture
                                        as *mut core::ffi::c_void,
                                    0 as core::ffi::c_int,
                                    core::mem::size_of::<MFVideoArea>() as size_t,
                                );
                                (*mixer).inputs[0 as usize].aperture.Area.cx =
                                    video_desc.SampleWidth as LONG;
                                (*mixer).inputs[0 as usize].aperture.Area.cy =
                                    video_desc.SampleHeight as LONG;
                            }

                            IMFMediaType_AddRef((*mixer).inputs[0 as usize].media_type);
                        }
                        CoTaskMemFree(guids as LPVOID);
                    }
                } else {
                    if (*__wine_dbch___default).flags as core::ffi::c_int
                        & (1 as core::ffi::c_int) << __WINE_DBCL_FIXME as core::ffi::c_int
                        != 0
                    {
                        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
                        ros_dbg_log(
                            (__dbcl_0) as u32,
                            __dbch_0,
                            b"dll/win32/evr/mixer.c\0"
                                as *const u8 as *const core::ffi::c_char,
                            b"video_mixer_transform_SetInputType\0" as *const u8
                                as *const core::ffi::c_char,
                            908 as core::ffi::c_int,
                            b"Unimplemented for substreams.\n\0" as *const u8
                                as *const core::ffi::c_char,
                        );
                    }
                    hr = E_NOTIMPL;
                }
            }
            IDirectXVideoProcessorService_Release(service);
        }
    }

    LeaveCriticalSection(&raw mut (*mixer).cs);

    return hr;
}

unsafe extern "system" fn video_mixer_transform_SetOutputType(iface: *mut IMFTransform, id: DWORD, type_0: *mut IMFMediaType, flags: DWORD) -> HRESULT {
    let equality_flags: c_uint = (MF_MEDIATYPE_EQUAL_MAJOR_TYPES | MF_MEDIATYPE_EQUAL_FORMAT_TYPES) as c_uint;
    let mut mixer: *mut video_mixer = impl_from_IMFTransform(iface);
    let mut hr: HRESULT = MF_E_INVALIDMEDIATYPE;
    let mut is_compressed: BOOL = (TRUE as i32);
    let mut compare_flags: DWORD;
    let mut i: c_uint;

    if (*__wine_dbch___default).flags as c_int & (1 as c_int) << __WINE_DBCL_TRACE as c_int != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0" as *const u8 as *const c_char,
            b"video_mixer_transform_SetOutputType\0" as *const u8 as *const c_char,
            930 as c_int,
            b"%p, %lu, %p, %#lx.\n\0" as *const u8 as *const c_char,
            iface,
            id,
            type_0,
            flags,
        );
    }

    if id != 0 {
        return MF_E_INVALIDSTREAMNUMBER;
    }

    if type_0.is_null() {
        return E_INVALIDARG;
    }

    if (IMFMediaType_IsCompressedFormat(type_0, &raw mut is_compressed) < 0) || is_compressed != 0 {
        return MF_E_INVALIDMEDIATYPE;
    }

    EnterCriticalSection(&raw mut (*mixer).cs);

    i = 0;
    while i < (*mixer).output.rt_formats_count {
        compare_flags = 0;
        if IMFMediaType_IsEqual(
            type_0,
            (*(*mixer).output.rt_formats.add(i as usize)).media_type,
            &raw mut compare_flags,
        ) < 0
        {
            i = i.wrapping_add(1);
            continue;
        }

        if (compare_flags & equality_flags as DWORD) == equality_flags as DWORD {
            hr = S_OK;
            break;
        }
        i = i.wrapping_add(1);
    }

    if (hr >= 0) && (flags & MFT_SET_TYPE_TEST_ONLY) == 0 {
        let mut service: *mut IDirectXVideoProcessorService = core::ptr::null_mut::<IDirectXVideoProcessorService>();

        hr = video_mixer_get_processor_service(mixer, &raw mut service);
        if hr >= 0 {
            let mut video_desc: DXVA2_VideoDesc = core::mem::zeroed::<DXVA2_VideoDesc>();
            let mut subtype: GUID = GUID {
                Data1: 0,
                Data2: 0,
                Data3: 0,
                Data4: [0; 8],
            };
            let mut rt_format: D3DFORMAT;

            if !(*mixer).processor.is_null() {
                IDirectXVideoProcessor_Release((*mixer).processor);
            }
            (*mixer).processor = core::ptr::null_mut::<IDirectXVideoProcessor>();

            video_mixer_init_dxva_videodesc((*mixer).inputs[0 as usize].media_type, &raw mut video_desc);
            IMFMediaType_GetGUID(type_0, &raw const MF_MT_SUBTYPE, &raw mut subtype);
            rt_format = subtype.Data1 as D3DFORMAT;

            hr = IDirectXVideoProcessorService_CreateVideoProcessor(
                service,
                &raw mut (*(*mixer).output.rt_formats.add(i as usize)).device,
                &raw mut video_desc,
                rt_format,
                MAX_MIXER_INPUT_SUBSTREAMS as UINT,
                &raw mut (*mixer).processor,
            );
            if hr >= 0 {
                if !(*mixer).output.media_type.is_null() {
                    IMFMediaType_Release((*mixer).output.media_type);
                }
                (*mixer).output.media_type = type_0;
                IMFMediaType_AddRef((*mixer).output.media_type);
            }

            IDirectXVideoProcessorService_Release(service);
        }
    }

    LeaveCriticalSection(&raw mut (*mixer).cs);

    return hr;
}

unsafe extern "system" fn video_mixer_transform_GetInputCurrentType(iface: *mut IMFTransform, id: DWORD, type_: *mut *mut IMFMediaType) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFTransform(iface);
    let mut stream: *mut input_stream = core::ptr::null_mut();
    let mut hr: HRESULT;

    EnterCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    hr = video_mixer_get_input(mixer, id, core::ptr::addr_of_mut!(stream));
    if hr >= 0 {
        if (*stream).media_type.is_null() {
            hr = MF_E_TRANSFORM_TYPE_NOT_SET;
        } else {
            *type_ = (*stream).media_type;
            IMFMediaType_AddRef(*type_);
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    return hr;
}

unsafe extern "system" fn video_mixer_transform_GetOutputCurrentType(iface: *mut IMFTransform, id: DWORD, type_0: *mut *mut IMFMediaType) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFTransform(iface);
    let mut hr: HRESULT = S_OK;
    if id != 0 {
        return MF_E_INVALIDSTREAMNUMBER;
    }
    EnterCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));
    if (*mixer).output.media_type.is_null() {
        hr = MF_E_TRANSFORM_TYPE_NOT_SET;
    } else {
        *type_0 = (*mixer).output.media_type;
        IMFMediaType_AddRef(*type_0);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));
    return hr;
}

unsafe extern "system" fn video_mixer_transform_GetInputStatus(
    iface: *mut IMFTransform,
    id: DWORD,
    status: *mut DWORD,
) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFTransform(iface);
    let mut stream: *mut input_stream = core::ptr::null_mut();
    let mut hr: HRESULT;

    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"video_mixer_transform_GetInputStatus\0" as *const u8 as *const core::ffi::c_char,
            1049 as core::ffi::c_int,
            b"%p, %lu, %p.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            id,
            status,
        );
    }

    if status.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    if (*mixer).output.media_type.is_null() {
        hr = MF_E_TRANSFORM_TYPE_NOT_SET;
    } else {
        hr = video_mixer_get_input(mixer, id as core::ffi::c_uint, core::ptr::addr_of_mut!(stream));
        if hr >= 0 {
            *status = (if !(*stream).sample.is_null() {
                0
            } else {
                MFT_INPUT_STATUS_ACCEPT_DATA
            }) as DWORD;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    return hr;
}

unsafe extern "system" fn video_mixer_transform_GetOutputStatus(
    iface: *mut IMFTransform,
    status: *mut DWORD,
) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFTransform(iface);
    let mut hr: HRESULT = S_OK;
    let mut i: u32;

    TRACE!("%p, %p.\n", iface, status);

    if status.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    if (*mixer).output.media_type.is_null() {
        hr = MF_E_TRANSFORM_TYPE_NOT_SET;
    } else {
        *status = MFT_OUTPUT_STATUS_SAMPLE_READY as DWORD;
        i = 0 as u32;
        while i < (*mixer).input_count {
            if (*mixer).inputs[i as usize].sample.is_null() {
                *status = 0 as DWORD;
                break;
            }
            i = i.wrapping_add(1);
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    return hr;
}
unsafe extern "system" fn video_mixer_transform_SetOutputBounds(iface: *mut IMFTransform, lower: LONGLONG, upper: LONGLONG) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFTransform(iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    (*mixer).lower_bound = lower;
    (*mixer).upper_bound = upper;

    LeaveCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    return S_OK;
}
unsafe extern "system" fn video_mixer_transform_ProcessEvent(
    iface: *mut IMFTransform,
    id: DWORD,
    event: *mut IMFMediaEvent,
) -> HRESULT {
    // FIXME("%p, %lu, %p.\n", iface, id, event);
    E_NOTIMPL
}

unsafe extern "C" fn video_mixer_request_sample(mut mixer: *mut video_mixer, mut idx: ::core::ffi::c_uint) {
    if (*mixer).event_sink.is_null() || (*mixer).inputs[idx as usize].sample_requested as ::core::ffi::c_int != 0 {
        return;
    }
    IMediaEventSink_Notify(
        (*mixer).event_sink,
        EC_SAMPLE_NEEDED as LONG,
        idx as LONG_PTR,
        0 as LONG_PTR,
    );
    (*mixer).inputs[idx as usize].sample_requested = 1 as ::core::ffi::c_uint;
}

unsafe extern "system" fn video_mixer_transform_ProcessMessage(
    iface: *mut IMFTransform,
    message: MFT_MESSAGE_TYPE,
    param: ULONG_PTR,
) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFTransform(iface);
    let mut hr: HRESULT = S_OK;
    let mut i: core::ffi::c_uint = 0;

    EnterCriticalSection(&raw mut (*mixer).cs);

    match message as core::ffi::c_uint {
        MFT_MESSAGE_SET_D3D_MANAGER => {
            video_mixer_release_device_manager(mixer);
            if param != 0 {
                hr = IUnknown_QueryInterface(
                    param as *mut IUnknown,
                    &raw const IID_IDirect3DDeviceManager9,
                    &raw mut (*mixer).device_manager as *mut *mut core::ffi::c_void,
                );
            }
        }

        MFT_MESSAGE_COMMAND_FLUSH => {
            video_mixer_flush_input(mixer);
        }

        MFT_MESSAGE_NOTIFY_BEGIN_STREAMING | MFT_MESSAGE_NOTIFY_END_STREAMING => {
            if (*mixer).is_streaming != 0 {
                video_mixer_flush_input(mixer);
            } else {
                i = 0 as core::ffi::c_uint;
                while i < (*mixer).input_count {
                    video_mixer_request_sample(mixer, i);
                    i = i.wrapping_add(1);
                }
            }

            (*mixer).is_streaming =
                (message as core::ffi::c_uint == MFT_MESSAGE_NOTIFY_BEGIN_STREAMING as core::ffi::c_uint)
                    as core::ffi::c_int as core::ffi::c_uint;
        }

        MFT_MESSAGE_COMMAND_DRAIN => {}

        _ => {
            WARN!("Message not handled %d.\n", message as core::ffi::c_uint);
            hr = E_NOTIMPL;
        }
    }

    LeaveCriticalSection(&raw mut (*mixer).cs);

    return hr;
}

unsafe extern "system" fn video_mixer_transform_ProcessInput(iface: *mut IMFTransform, id: DWORD, sample: *mut IMFSample, flags: DWORD) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFTransform(iface);
    let mut input: *mut input_stream = core::ptr::null_mut::<input_stream>();
    let mut hr: HRESULT = 0;
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0" as *const u8 as *const i8,
            b"video_mixer_transform_ProcessInput\0" as *const u8 as *const i8,
            1190 as i32,
            b"%p, %lu, %p, %#lx.\n\0" as *const u8 as *const i8,
            iface,
            id,
            sample,
            flags,
        );
    }
    if sample.is_null() {
        return E_POINTER;
    }
    EnterCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));
    hr = video_mixer_get_input(mixer, id as u32, &mut input);
    if hr >= 0 {
        if (*input).media_type.is_null() || (*mixer).output.media_type.is_null() {
            hr = MF_E_TRANSFORM_TYPE_NOT_SET;
        } else if !(*input).sample.is_null() && (*mixer).output_rendered == 0 {
            hr = MF_E_NOTACCEPTING;
        } else {
            if !(*input).sample.is_null() && (*mixer).output_rendered as i32 != 0 {
                video_mixer_flush_input(mixer);
            }
            (*mixer).is_streaming = 1 as u32;
            (*input).sample_requested = 0 as u32;
            (*input).sample = sample;
            IMFSample_AddRef((*input).sample);
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));
    return hr;
}

unsafe extern "C" fn video_mixer_get_sample_surface(sample: *mut IMFSample, surface: *mut *mut IDirect3DSurface9) -> HRESULT {
    let mut buffer: *mut IMFMediaBuffer = core::ptr::null_mut::<IMFMediaBuffer>();
    let mut gs: *mut IMFGetService = core::ptr::null_mut::<IMFGetService>();
    let mut hr: HRESULT = 0;

    hr = IMFSample_GetBufferByIndex(sample, 0 as DWORD, &raw mut buffer);
    if hr < 0 {
        return hr;
    }

    hr = IMFMediaBuffer_QueryInterface(
        buffer,
        &raw const IID_IMFGetService,
        &raw mut gs as *mut *mut core::ffi::c_void,
    );
    IMFMediaBuffer_Release(buffer);
    if hr < 0 {
        return hr;
    }

    hr = IMFGetService_GetService(
        gs,
        &raw const MR_BUFFER_SERVICE,
        &raw const IID_IDirect3DSurface9,
        surface as *mut LPVOID,
    );
    IMFGetService_Release(gs);
    return hr;
}

unsafe extern "C" fn video_mixer_get_d3d_device(mixer: *mut video_mixer, device: *mut *mut IDirect3DDevice9) -> HRESULT {
    let mut hr: HRESULT;

    loop {
        hr = IDirect3DDeviceManager9_LockDevice((*mixer).device_manager, (*mixer).device_handle, device, (TRUE) as BOOL);
        if hr == DXVA2_E_NEW_VIDEO_DEVICE {
            hr = video_mixer_open_device_handle(mixer);
            if hr >= 0 {
                continue;
            }
        }
        break;
    }

    return hr;
}

unsafe extern "C" fn video_mixer_rect_needs_scaling(scale: *const MFVideoNormalizedRect) -> BOOL {
    return ((*scale).left != 0.0f32
        || (*scale).top != 0.0f32
        || (*scale).right != 1.0f32
        || (*scale).bottom != 1.0f32) as BOOL;
}

unsafe extern "C" fn video_mixer_scale_rect(
    mut rect: *mut RECT,
    mut width: ::core::ffi::c_uint,
    mut height: ::core::ffi::c_uint,
    mut scale: *const MFVideoNormalizedRect,
) {
    if video_mixer_rect_needs_scaling(scale) != 0 {
        (*rect).left = (width as ::core::ffi::c_float * (*scale).left) as LONG;
        (*rect).right = (width as ::core::ffi::c_float * (*scale).right) as LONG;
        (*rect).top = (height as ::core::ffi::c_float * (*scale).top) as LONG;
        (*rect).bottom = (height as ::core::ffi::c_float * (*scale).bottom) as LONG;
    } else {
        SetRect(
            rect as LPRECT,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            width as ::core::ffi::c_int,
            height as ::core::ffi::c_int,
        );
    };
}

unsafe extern "C" fn video_mixer_correct_aspect_ratio(src: *const RECT, dst: *mut RECT) {
    let mut src_width: ::core::ffi::c_uint = ((*src).right - (*src).left) as ::core::ffi::c_uint;
    let mut src_height: ::core::ffi::c_uint = ((*src).bottom - (*src).top) as ::core::ffi::c_uint;
    let mut dst_width: ::core::ffi::c_uint = ((*dst).right - (*dst).left) as ::core::ffi::c_uint;
    let mut dst_height: ::core::ffi::c_uint = ((*dst).bottom - (*dst).top) as ::core::ffi::c_uint;
    if src_width.wrapping_mul(dst_height) > dst_width.wrapping_mul(src_height) {
        let mut dst_center: ::core::ffi::c_uint =
            (((*dst).top + (*dst).bottom) / 2 as LONG) as ::core::ffi::c_uint;
        let mut scaled_height: ::core::ffi::c_uint =
            src_height.wrapping_mul(dst_width).wrapping_div(src_width);
        (*dst).top = dst_center
            .wrapping_sub(scaled_height.wrapping_div(2 as ::core::ffi::c_uint))
            as LONG;
        (*dst).bottom = ((*dst).top as ::core::ffi::c_uint)
            .wrapping_add(scaled_height as ::core::ffi::c_uint) as LONG;
    } else if src_width.wrapping_mul(dst_height) < dst_width.wrapping_mul(src_height) {
        let mut dst_center: ::core::ffi::c_uint =
            (((*dst).left + (*dst).right) / 2 as LONG) as ::core::ffi::c_uint;
        let mut scaled_width: ::core::ffi::c_uint =
            src_width.wrapping_mul(dst_height).wrapping_div(src_height);
        (*dst).left = dst_center
            .wrapping_sub(scaled_width.wrapping_div(2 as ::core::ffi::c_uint))
            as LONG;
        (*dst).right = ((*dst).left as ::core::ffi::c_uint)
            .wrapping_add(scaled_width as ::core::ffi::c_uint) as LONG;
    }
}

unsafe extern "C" fn video_mixer_render(mixer: *mut video_mixer, rt: *mut IDirect3DSurface9) {
    let mut samples: [DXVA2_VideoSample; MAX_MIXER_INPUT_STREAMS] = core::mem::zeroed();
    let _sample_init = 0;
    let mut params: DXVA2_VideoProcessBltParams = core::mem::zeroed();
    let mut zoom_rect: MFVideoNormalizedRect = MFVideoNormalizedRect {
        left: 0.,
        top: 0.,
        right: 0.,
        bottom: 0.,
    };
    let mut stream: *mut input_stream = core::ptr::null_mut::<input_stream>();
    let mut aperture: MFVideoArea = core::mem::zeroed();
    let mut hr: HRESULT = S_OK;
    let mut i: ::core::ffi::c_uint = 0;

    if IMFAttributes_GetBlob(
        (*mixer).attributes,
        &raw const VIDEO_ZOOM_RECT,
        &raw mut zoom_rect as *mut UINT8,
        ::core::mem::size_of::<MFVideoNormalizedRect>() as UINT32,
        core::ptr::null_mut::<UINT32>(),
    ) < 0
    {
        zoom_rect.top = 0.0f32;
        zoom_rect.left = zoom_rect.top;
        zoom_rect.bottom = 1.0f32;
        zoom_rect.right = zoom_rect.bottom;
    }

    if IMFMediaType_GetBlob(
        (*mixer).output.media_type,
        &raw const MF_MT_GEOMETRIC_APERTURE,
        &raw mut aperture as *mut UINT8,
        ::core::mem::size_of::<MFVideoArea>() as UINT32,
        core::ptr::null_mut::<UINT32>(),
    ) < 0
    {
        aperture = (*mixer).inputs[0 as usize].aperture;
    }

    SetRect(
        &raw mut params.TargetRect,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        aperture.Area.cx as ::core::ffi::c_int,
        aperture.Area.cy as ::core::ffi::c_int,
    );
    OffsetRect(
        &raw mut params.TargetRect,
        aperture.OffsetX.value as ::core::ffi::c_int,
        aperture.OffsetY.value as ::core::ffi::c_int,
    );

    i = 0 as ::core::ffi::c_uint;
    while i < (*mixer).input_count {
        let sample: *mut DXVA2_VideoSample =
            (&raw mut samples as *mut DXVA2_VideoSample).offset(i as isize) as *mut DXVA2_VideoSample;
        let mut surface: *mut IDirect3DSurface9 = core::ptr::null_mut::<IDirect3DSurface9>();

        stream = (*mixer).zorder[i as usize];

        hr = video_mixer_get_sample_surface((*stream).sample, &raw mut surface);
        if hr < 0
        {
            WARN!(
                "Failed to get source surface for stream %u, hr %#lx.\n",
                i,
                hr
            );
            break;
        }

        /* Full input frame corrected to full destination rectangle. */

        video_mixer_scale_rect(
            &raw mut (*sample).SrcRect,
            (*stream).aperture.Area.cx as ::core::ffi::c_uint,
            (*stream).aperture.Area.cy as ::core::ffi::c_uint,
            &raw mut zoom_rect,
        );
        OffsetRect(
            &raw mut (*sample).SrcRect,
            (*stream).aperture.OffsetX.value as ::core::ffi::c_int,
            (*stream).aperture.OffsetY.value as ::core::ffi::c_int,
        );
        CopyRect(
            &raw mut (*sample).DstRect,
            &raw mut params.TargetRect as LPCRECT,
        );
        video_mixer_correct_aspect_ratio(
            &raw mut (*sample).SrcRect,
            &raw mut (*sample).DstRect,
        );

        if video_mixer_rect_needs_scaling(&raw mut (*stream).rect) != 0 {
            WARN!(
                "Ignoring stream %u rectangle %s.\n",
                (*stream).id,
                debugstr_normalized_rect(&raw mut (*stream).rect)
            );
        }

        (*sample)
            .SampleFormat
            .__bindgen_anon_1
            .set_SampleFormat(
                (if (*stream).id == 0 as ::core::ffi::c_uint {
                    DXVA2_SampleProgressiveFrame as ::core::ffi::c_int
                } else {
                    DXVA2_SampleSubStream as ::core::ffi::c_int
                }) as UINT as UINT,
            );
        (*sample).SrcSurface = surface;
        (*sample).PlanarAlpha = DXVA2_Fixed32OpaqueAlpha();
        i = i.wrapping_add(1);
    }

    if hr >= 0 {
        params.BackgroundColor = (*mixer).bkgnd_color.ayuv;
        params.Alpha = DXVA2_Fixed32OpaqueAlpha();

        hr = IDirectXVideoProcessor_VideoProcessBlt(
            (*mixer).processor,
            rt,
            &raw mut params,
            &raw mut samples as *mut DXVA2_VideoSample,
            (*mixer).input_count as UINT,
            core::ptr::null_mut::<HANDLE>(),
        );
        if hr < 0
        {
            WARN!("Failed to process samples, hr %#lx.\n", hr);
        }
    }

    i = 0 as ::core::ffi::c_uint;
    while i < (*mixer).input_count {
        if !samples[i as usize].SrcSurface.is_null() {
            IDirect3DSurface9_Release(samples[i as usize].SrcSurface);
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn video_mixer_get_sample_desired_time(sample: *mut IMFSample, timestamp: *mut LONGLONG, duration: *mut LONGLONG) -> HRESULT {
    let mut desired: *mut IMFDesiredSample = core::ptr::null_mut();
    let mut hr: HRESULT;
    *duration = 0;
    *timestamp = 0;
    hr = IMFSample_QueryInterface(sample, core::ptr::addr_of!(IID_IMFDesiredSample), core::ptr::addr_of_mut!(desired) as *mut *mut core::ffi::c_void);
    if hr >= 0 {
        hr = IMFDesiredSample_GetDesiredSampleTimeAndDuration(desired, timestamp, duration);
        IMFDesiredSample_Release(desired);
    }

    return hr;
}

unsafe extern "C" fn video_mixer_has_input(mixer: *const video_mixer) -> BOOL {
    let mut i: ::core::ffi::c_uint = 0;
    while i < (*mixer).input_count {
        if (*mixer).inputs[i as usize].sample.is_null() {
            return (FALSE) as BOOL;
        }
        i = i.wrapping_add(1);
    }
    return (TRUE) as BOOL;
}

unsafe extern "system" fn video_mixer_transform_ProcessOutput(
    iface: *mut IMFTransform,
    flags: DWORD,
    count: DWORD,
    buffers: *mut MFT_OUTPUT_DATA_BUFFER,
    status: *mut DWORD,
) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFTransform(iface);
    let mut timestamp: LONGLONG = 0;
    let mut duration: LONGLONG = 0;
    let mut surface: *mut IDirect3DSurface9 = core::ptr::null_mut();
    let mut device: *mut IDirect3DDevice9 = core::ptr::null_mut();
    let mut repaint: BOOL = (FALSE as i32);
    let mut i: u32;
    let mut hr: HRESULT;

    TRACE!("%p, %#lx, %lu, %p, %p.\n", iface, flags, count, buffers, status);

    if buffers.is_null() || count == 0 || count > 1 || (*buffers).pSample.is_null() {
        return E_INVALIDARG;
    }

    if (*buffers).dwStreamID != 0 {
        return MF_E_INVALIDSTREAMNUMBER;
    }

    *status = 0;

    EnterCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    hr = video_mixer_get_sample_surface((*buffers).pSample, core::ptr::addr_of_mut!(surface));
    if hr >= 0 {
        if (*mixer).is_streaming != 0 {
            /* Desired timestamp is ignored, duration is required to be non-zero but is not used either. */
            if video_mixer_get_sample_desired_time(
                (*buffers).pSample,
                core::ptr::addr_of_mut!(timestamp),
                core::ptr::addr_of_mut!(duration),
            ) >= 0
            {
                repaint = (duration != 0) as BOOL;
                if repaint == 0 {
                    WARN!("Unexpected sample duration.\n");
                    hr = E_INVALIDARG;
                }
            }

            /* Not enough input, or no new input. */
            if hr >= 0
                && (video_mixer_has_input(mixer) == 0
                    || (repaint == 0 && (*mixer).output_rendered != 0))
            {
                hr = MF_E_TRANSFORM_NEED_MORE_INPUT;
            }

            if hr >= 0 {
                video_mixer_render(mixer, surface);

                duration = 0;
                timestamp = duration;
                if IMFSample_GetSampleTime(
                    (*mixer).inputs[0 as usize].sample,
                    core::ptr::addr_of_mut!(timestamp),
                ) >= 0
                {
                    IMFSample_GetSampleDuration(
                        (*mixer).inputs[0 as usize].sample,
                        core::ptr::addr_of_mut!(duration),
                    );
                    IMFSample_SetSampleTime((*buffers).pSample, timestamp);
                    IMFSample_SetSampleDuration((*buffers).pSample, duration);
                }
                (*mixer).output_rendered = 1;
            }

            if hr >= 0 && repaint == 0 {
                i = 0;
                while i < (*mixer).input_count {
                    video_mixer_request_sample(mixer, i);
                    i = i.wrapping_add(1);
                }
            }
        } else {
            if video_mixer_get_sample_desired_time(
                (*buffers).pSample,
                core::ptr::addr_of_mut!(timestamp),
                core::ptr::addr_of_mut!(duration),
            ) >= 0
            {
                hr = video_mixer_get_d3d_device(mixer, core::ptr::addr_of_mut!(device));
                if hr >= 0 {
                    IDirect3DDevice9_ColorFill(device, surface, core::ptr::null(), 0);
                    IDirect3DDeviceManager9_UnlockDevice(
                        (*mixer).device_manager,
                        (*mixer).device_handle,
                        (FALSE) as BOOL,
                    );
                    IDirect3DDevice9_Release(device);
                }
            } else {
                hr = MF_E_TRANSFORM_NEED_MORE_INPUT;
            }
        }
        IDirect3DSurface9_Release(surface);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    return hr;
}

unsafe extern "system" fn video_mixer_device_id_QueryInterface(iface: *mut IMFVideoDeviceID, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFVideoDeviceID(iface);
    return IMFTransform_QueryInterface(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface), riid, obj);
}

unsafe extern "system" fn video_mixer_device_id_AddRef(iface: *mut IMFVideoDeviceID) -> ULONG {
    let mixer: *mut video_mixer = impl_from_IMFVideoDeviceID(iface);
    return IMFTransform_AddRef(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface));
}

unsafe extern "system" fn video_mixer_device_id_Release(iface: *mut IMFVideoDeviceID) -> ULONG {
    let mut mixer: *mut video_mixer = impl_from_IMFVideoDeviceID(iface);
    return IMFTransform_Release(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface));
}

unsafe extern "system" fn video_mixer_device_id_GetDeviceID(iface: *mut IMFVideoDeviceID, device_id: *mut IID) -> HRESULT {
    if device_id.is_null() {
        return E_POINTER;
    }

    memcpy(
        device_id as *mut core::ffi::c_void,
        core::ptr::addr_of!(IID_IDirect3DDevice9) as *const core::ffi::c_void,
        core::mem::size_of::<IID>(),
    );

    return S_OK;
}

unsafe extern "system" fn video_mixer_service_client_QueryInterface(
    iface: *mut IMFTopologyServiceLookupClient,
    riid: *const IID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFTopologyServiceLookupClient(iface);
    return IMFTransform_QueryInterface(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface), riid, obj);
}

unsafe extern "system" fn video_mixer_service_client_AddRef(iface: *mut IMFTopologyServiceLookupClient) -> ULONG {
    let mut mixer: *mut video_mixer = impl_from_IMFTopologyServiceLookupClient(iface);
    return IMFTransform_AddRef(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface));
}

unsafe extern "system" fn video_mixer_service_client_Release(iface: *mut IMFTopologyServiceLookupClient) -> ULONG {
    let mixer: *mut video_mixer = impl_from_IMFTopologyServiceLookupClient(iface);
    return IMFTransform_Release(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface));
}

unsafe extern "system" fn video_mixer_service_client_InitServicePointers(
    iface: *mut IMFTopologyServiceLookupClient,
    service_lookup: *mut IMFTopologyServiceLookup,
) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFTopologyServiceLookupClient(iface);
    let mut count: DWORD = 0;
    let mut hr: HRESULT = 0;

    if service_lookup.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(&raw mut (*mixer).cs);

    count = 1;
    hr = IMFTopologyServiceLookup_LookupService(
        service_lookup,
        MF_SERVICE_LOOKUP_GLOBAL,
        0,
        &raw const MR_VIDEO_RENDER_SERVICE,
        &raw const IID_IMediaEventSink,
        &raw mut (*mixer).event_sink as *mut *mut ::core::ffi::c_void,
        &raw mut count,
    );
    if hr < 0 {
        WARN!("Failed to get renderer event sink, hr %#lx.\n", hr);
    }

    LeaveCriticalSection(&raw mut (*mixer).cs);

    return hr;
}

unsafe extern "system" fn video_mixer_service_client_ReleaseServicePointers(
    iface: *mut IMFTopologyServiceLookupClient,
) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFTopologyServiceLookupClient(iface);

    TRACE!("%p.\n", iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    if !(*mixer).event_sink.is_null() {
        IMediaEventSink_Release((*mixer).event_sink);
    }
    (*mixer).event_sink = core::ptr::null_mut();

    LeaveCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    return S_OK;
}

unsafe extern "system" fn video_mixer_control_QueryInterface(iface: *mut IMFVideoMixerControl2, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFVideoMixerControl2(iface);
    return IMFTransform_QueryInterface(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface), riid, obj);
}

unsafe extern "system" fn video_mixer_control_AddRef(iface: *mut IMFVideoMixerControl2) -> ULONG {
    let mut mixer: *mut video_mixer = impl_from_IMFVideoMixerControl2(iface);
    return IMFTransform_AddRef(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface));
}

unsafe extern "system" fn video_mixer_control_Release(iface: *mut IMFVideoMixerControl2) -> ULONG {
    let mixer: *mut video_mixer = impl_from_IMFVideoMixerControl2(iface);
    return IMFTransform_Release(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface));
}

unsafe extern "system" fn video_mixer_control_SetStreamZOrder(iface: *mut IMFVideoMixerControl2, id: DWORD, zorder: DWORD) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFVideoMixerControl2(iface);
    let mut stream: *mut input_stream = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    /* Can't change reference stream. */
    if id == 0 && zorder != 0 {
        return E_INVALIDARG;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    if zorder >= (*mixer).input_count {
        hr = E_INVALIDARG;
    } else {
        hr = video_mixer_get_input(mixer, id, core::ptr::addr_of_mut!(stream));
        if hr >= 0 {
            /* Lowest zorder only applies to reference stream. */
            if id != 0 && zorder == 0 {
                hr = MF_E_INVALIDREQUEST;
            } else if (*stream).zorder != zorder {
                (*stream).zorder = zorder;
                video_mixer_update_zorder_map(mixer);
            }
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    return hr;
}

unsafe extern "system" fn video_mixer_control_GetStreamZOrder(iface: *mut IMFVideoMixerControl2, id: DWORD, zorder: *mut DWORD) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFVideoMixerControl2(iface);
    let mut stream: *mut input_stream = core::ptr::null_mut();
    let mut hr: HRESULT;

    if zorder.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    hr = video_mixer_get_input(mixer, id, core::ptr::addr_of_mut!(stream));
    if hr >= 0 {
        *zorder = (*stream).zorder;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    return hr;
}

unsafe extern "system" fn video_mixer_control_SetStreamOutputRect(
    iface: *mut IMFVideoMixerControl2,
    id: DWORD,
    rect: *const MFVideoNormalizedRect,
) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFVideoMixerControl2(iface);
    let mut stream: *mut input_stream = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    if ((*__wine_dbch___default).flags as i32) & ((1i32) << (__WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0".as_ptr() as *const i8,
            b"video_mixer_control_SetStreamOutputRect\0".as_ptr() as *const i8,
            1699i32,
            b"%p, %lu, %s.\n\0".as_ptr() as *const i8,
            iface,
            id,
            debugstr_normalized_rect(rect),
        );
    }

    if rect.is_null() {
        return E_POINTER;
    }

    if (*rect).left > (*rect).right
        || (*rect).top > (*rect).bottom
        || (*rect).left < 0.0f32
        || (*rect).top < 0.0f32
        || (*rect).right > 1.0f32
        || (*rect).bottom > 1.0f32
    {
        return E_INVALIDARG;
    }

    EnterCriticalSection(&raw mut (*mixer).cs);

    hr = video_mixer_get_input(mixer, id, &raw mut stream);
    if hr >= 0 {
        (*stream).rect = *rect;
    }

    LeaveCriticalSection(&raw mut (*mixer).cs);

    return hr;
}

unsafe extern "system" fn video_mixer_control_GetStreamOutputRect(iface: *mut IMFVideoMixerControl2, id: DWORD, rect: *mut MFVideoNormalizedRect) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFVideoMixerControl2(iface);
    let mut stream: *mut input_stream = core::mem::zeroed();
    let mut hr: HRESULT;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"video_mixer_control_GetStreamOutputRect\0" as *const u8 as *const ::core::ffi::c_char,
            1727 as ::core::ffi::c_int,
            b"%p, %lu, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            id,
            rect,
        );
    }

    if rect.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    hr = video_mixer_get_input(mixer, id as ::core::ffi::c_uint, core::ptr::addr_of_mut!(stream));
    if hr >= 0 {
        *rect = (*stream).rect;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    return hr;
}
unsafe extern "system" fn video_mixer_control_SetMixingPrefs(
    iface: *mut IMFVideoMixerControl2,
    flags: DWORD,
) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFVideoMixerControl2(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0".as_ptr() as *const i8,
            b"video_mixer_control_SetMixingPrefs\0".as_ptr() as *const i8,
            1746i32,
            b"%p, %#lx.\n\0".as_ptr() as *const i8,
            iface,
            flags,
        );
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));
    (*mixer).mixing_flags = flags;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    return S_OK;
}

unsafe extern "system" fn video_mixer_control_GetMixingPrefs(iface: *mut IMFVideoMixerControl2, flags: *mut DWORD) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFVideoMixerControl2(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0" as *const u8 as *const i8,
            b"video_mixer_control_GetMixingPrefs\0" as *const u8 as *const i8,
            1759i32,
            b"%p, %p.\n\0" as *const u8 as *const i8,
            iface,
            flags,
        );
    }
    if flags.is_null() {
        return E_POINTER;
    }
    EnterCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));
    *flags = (*mixer).mixing_flags as DWORD;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));
    return S_OK;
}

unsafe extern "system" fn video_mixer_getservice_QueryInterface(iface: *mut IMFGetService, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFGetService(iface);
    return IMFTransform_QueryInterface(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface), riid, obj);
}

unsafe extern "system" fn video_mixer_getservice_AddRef(iface: *mut IMFGetService) -> ULONG {
    let mixer: *mut video_mixer = impl_from_IMFGetService(iface);
    return IMFTransform_AddRef(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface));
}

unsafe extern "system" fn video_mixer_getservice_Release(iface: *mut IMFGetService) -> ULONG {
    let mixer: *mut video_mixer = impl_from_IMFGetService(iface);
    return IMFTransform_Release(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface));
}

unsafe extern "system" fn video_mixer_getservice_GetService(
    iface: *mut IMFGetService,
    service: *const GUID,
    riid: *const IID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if IsEqualGUID!(service, &raw const MR_VIDEO_MIXER_SERVICE) != false {
        if IsEqualIID!(riid, &raw const IID_IMFVideoMixerBitmap) != 0
            || IsEqualIID!(riid, &raw const IID_IMFVideoProcessor) != 0
            || IsEqualIID!(riid, &raw const IID_IMFVideoPositionMapper) != 0
            || IsEqualIID!(riid, &raw const IID_IMFVideoMixerControl) != 0
            || IsEqualIID!(riid, &raw const IID_IMFVideoMixerControl2) != 0
        {
            return IMFGetService_QueryInterface(iface, riid, obj);
        }

        return E_NOINTERFACE;
    }

    return MF_E_UNSUPPORTED_SERVICE;
}

unsafe extern "system" fn video_mixer_bitmap_QueryInterface(iface: *mut IMFVideoMixerBitmap, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFVideoMixerBitmap(iface);
    return IMFTransform_QueryInterface(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface), riid, obj);
}

unsafe extern "system" fn video_mixer_bitmap_AddRef(iface: *mut IMFVideoMixerBitmap) -> ULONG {
    let mixer: *mut video_mixer = impl_from_IMFVideoMixerBitmap(iface);
    return IMFTransform_AddRef(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface));
}

unsafe extern "system" fn video_mixer_bitmap_Release(iface: *mut IMFVideoMixerBitmap) -> ULONG {
    let mut mixer: *mut video_mixer = impl_from_IMFVideoMixerBitmap(iface);
    return IMFTransform_Release(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface));
}
unsafe extern "system" fn video_mixer_bitmap_SetAlphaBitmap(
    iface: *mut IMFVideoMixerBitmap,
    bitmap: *const MFVideoAlphaBitmap,
) -> HRESULT {
    // FIXME("%p, %p.\n", iface, bitmap);
    E_NOTIMPL
}
unsafe extern "system" fn video_mixer_bitmap_ClearAlphaBitmap(
    iface: *mut IMFVideoMixerBitmap,
) -> HRESULT {
    // FIXME("%p.\n", iface);
    E_NOTIMPL
}
unsafe extern "system" fn video_mixer_bitmap_UpdateAlphaBitmapParameters(
    iface: *mut IMFVideoMixerBitmap,
    params: *const MFVideoAlphaBitmapParams,
) -> HRESULT {
    // FIXME("%p, %p.\n", iface, params);
    E_NOTIMPL
}
unsafe extern "system" fn video_mixer_bitmap_GetAlphaBitmapParameters(
    iface: *mut IMFVideoMixerBitmap,
    params: *mut MFVideoAlphaBitmapParams,
) -> HRESULT {
    // FIXME("%p, %p.\n", iface, params);
    E_NOTIMPL
}

unsafe extern "system" fn video_mixer_position_mapper_QueryInterface(iface: *mut IMFVideoPositionMapper, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFVideoPositionMapper(iface);
    return IMFTransform_QueryInterface(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface), riid, obj);
}

unsafe extern "system" fn video_mixer_position_mapper_AddRef(iface: *mut IMFVideoPositionMapper) -> ULONG {
    let mixer: *mut video_mixer = impl_from_IMFVideoPositionMapper(iface);
    return IMFTransform_AddRef(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface));
}

unsafe extern "system" fn video_mixer_position_mapper_Release(iface: *mut IMFVideoPositionMapper) -> ULONG {
    let mut mixer: *mut video_mixer = impl_from_IMFVideoPositionMapper(iface);
    return IMFTransform_Release(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface));
}
unsafe extern "system" fn video_mixer_position_mapper_MapOutputCoordinateToInputStream(
    iface: *mut IMFVideoPositionMapper,
    x_out: FLOAT,
    y_out: FLOAT,
    output_stream: DWORD,
    input_stream: DWORD,
    x_in: *mut FLOAT,
    y_in: *mut FLOAT,
) -> HRESULT {
    // FIXME("%p, %f, %f, %lu, %lu, %p, %p.\n", iface, x_out, y_out, output_stream, input_stream, x_in, y_in);
    E_NOTIMPL
}

unsafe extern "system" fn video_mixer_processor_QueryInterface(iface: *mut IMFVideoProcessor, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFVideoProcessor(iface);
    return IMFTransform_QueryInterface(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface), riid, obj);
}

unsafe extern "system" fn video_mixer_processor_AddRef(iface: *mut IMFVideoProcessor) -> ULONG {
    let mut mixer: *mut video_mixer = impl_from_IMFVideoProcessor(iface);
    return IMFTransform_AddRef(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface));
}

unsafe extern "system" fn video_mixer_processor_Release(iface: *mut IMFVideoProcessor) -> ULONG {
    let mixer: *mut video_mixer = impl_from_IMFVideoProcessor(iface);
    return IMFTransform_Release(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface));
}

unsafe extern "system" fn video_mixer_processor_GetAvailableVideoProcessorModes(
    iface: *mut IMFVideoProcessor,
    count: *mut UINT,
    modes: *mut *mut GUID,
) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFVideoProcessor(iface);
    let mut service: *mut IDirectXVideoProcessorService = core::ptr::null_mut();
    let mut video_desc: DXVA2_VideoDesc = core::mem::zeroed::<DXVA2_VideoDesc>();
    let mut hr: HRESULT;

    EnterCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    if (*mixer).inputs[0 as usize].media_type.is_null()
        || (*mixer).output.media_type.is_null()
    {
        hr = MF_E_TRANSFORM_TYPE_NOT_SET as HRESULT;
    } else if {
        hr = video_mixer_get_processor_service(mixer, core::ptr::addr_of_mut!(service));
        hr >= 0 as HRESULT
    } {
        video_mixer_init_dxva_videodesc(
            (*mixer).inputs[0 as usize].media_type,
            core::ptr::addr_of_mut!(video_desc),
        );
        hr = IDirectXVideoProcessorService_GetVideoProcessorDeviceGuids(
            service,
            core::ptr::addr_of_mut!(video_desc),
            count,
            modes,
        );
        IDirectXVideoProcessorService_Release(service);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*mixer).cs));

    return hr;
}

unsafe extern "system" fn video_mixer_processor_GetVideoProcessorCaps(
    mut iface: *mut IMFVideoProcessor,
    mut mode: *mut GUID,
    mut caps: *mut DXVA2_VideoProcessorCaps,
) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFVideoProcessor(iface);
    let mut service: *mut IDirectXVideoProcessorService = core::ptr::null_mut::<IDirectXVideoProcessorService>();
    let mut video_desc: DXVA2_VideoDesc = core::mem::zeroed();
    let mut subtype: GUID = core::mem::zeroed();
    let mut hr: HRESULT = 0;
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"video_mixer_processor_GetVideoProcessorCaps\0" as *const u8
                as *const ::core::ffi::c_char,
            1978 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(mode),
            caps,
        );
    }

    EnterCriticalSection(&raw mut (*mixer).cs);

    if (*mixer).inputs[0 as usize].media_type.is_null()
        || (*mixer).output.media_type.is_null()
    {
        hr = MF_E_TRANSFORM_TYPE_NOT_SET;
    } else if {
        hr = video_mixer_get_processor_service(mixer, &raw mut service);
        hr >= 0
    } {
        video_mixer_init_dxva_videodesc(
            (*mixer).inputs[0 as usize].media_type,
            &raw mut video_desc,
        );
        IMFMediaType_GetGUID(
            (*mixer).output.media_type,
            &raw const MF_MT_SUBTYPE,
            &raw mut subtype,
        );

        hr = IDirectXVideoProcessorService_GetVideoProcessorCaps(
            service,
            mode,
            &raw mut video_desc,
            subtype.Data1 as D3DFORMAT,
            caps,
        );
        IDirectXVideoProcessorService_Release(service);
    }

    LeaveCriticalSection(&raw mut (*mixer).cs);

    return hr;
}
unsafe extern "system" fn video_mixer_processor_GetVideoProcessorMode(
    iface: *mut IMFVideoProcessor,
    mode: *mut GUID,
) -> HRESULT {
    // FIXME("%p, %p.\n", iface, mode);
    E_NOTIMPL
}
unsafe extern "system" fn video_mixer_processor_SetVideoProcessorMode(
    iface: *mut IMFVideoProcessor,
    mode: *mut GUID,
) -> HRESULT {
    // FIXME("%p, %s.\n", iface, debugstr_guid(mode));
    E_NOTIMPL
}
unsafe extern "system" fn video_mixer_processor_GetProcAmpRange(
    iface: *mut IMFVideoProcessor,
    prop: DWORD,
    range: *mut DXVA2_ValueRange,
) -> HRESULT {
    // FIXME("%p, %#lx, %p.\n", iface, prop, range);
    E_NOTIMPL
}
unsafe extern "system" fn video_mixer_processor_GetProcAmpValues(
    iface: *mut IMFVideoProcessor,
    flags: DWORD,
    values: *mut DXVA2_ProcAmpValues,
) -> HRESULT {
    // FIXME("%p, %#lx, %p.\n", iface, flags, values);
    E_NOTIMPL
}
unsafe extern "system" fn video_mixer_processor_SetProcAmpValues(
    iface: *mut IMFVideoProcessor,
    flags: DWORD,
    values: *mut DXVA2_ProcAmpValues,
) -> HRESULT {
    // FIXME("%p, %#lx, %p.\n", iface, flags, values);
    E_NOTIMPL
}
unsafe extern "system" fn video_mixer_processor_GetFilteringRange(
    iface: *mut IMFVideoProcessor,
    prop: DWORD,
    range: *mut DXVA2_ValueRange,
) -> HRESULT {
    // FIXME("%p, %#lx, %p.\n", iface, prop, range);
    E_NOTIMPL
}
unsafe extern "system" fn video_mixer_processor_GetFilteringValue(
    iface: *mut IMFVideoProcessor,
    prop: DWORD,
    value: *mut DXVA2_Fixed32,
) -> HRESULT {
    // FIXME("%p, %#lx, %p.\n", iface, prop, value);
    E_NOTIMPL
}
unsafe extern "system" fn video_mixer_processor_SetFilteringValue(
    iface: *mut IMFVideoProcessor,
    prop: DWORD,
    value: *mut DXVA2_Fixed32,
) -> HRESULT {
    // FIXME("%p, %#lx, %p.\n", iface, prop, value);
    E_NOTIMPL
}

unsafe extern "system" fn video_mixer_processor_GetBackgroundColor(iface: *mut IMFVideoProcessor, color: *mut COLORREF) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFVideoProcessor(iface);

    if color.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(&raw mut (*mixer).cs);
    *color = (*mixer).bkgnd_color.rgba;
    LeaveCriticalSection(&raw mut (*mixer).cs);

    return S_OK;
}

unsafe extern "C" fn video_mixer_rgb_to_ycbcr(
    mut rgb: COLORREF,
    mut ayuv: *mut DXVA2_AYUVSample16,
) {
    let mut y: ::core::ffi::c_int = 0;
    let mut cb: ::core::ffi::c_int = 0;
    let mut cr: ::core::ffi::c_int = 0;
    let mut r: ::core::ffi::c_int = 0;
    let mut g: ::core::ffi::c_int = 0;
    let mut b: ::core::ffi::c_int = 0;

    r = GetRValue!(rgb) as ::core::ffi::c_int;
    g = GetGValue!(rgb) as ::core::ffi::c_int;
    b = GetBValue!(rgb) as ::core::ffi::c_int;
    /* Coefficients according to SDTV ITU-R BT.601 */
    y = (77 * r
        + 150 * g
        + 29 * b
        + 128)
        / 256
        + 16;
    cb = (-44 * r
        - 87 * g
        + 131 * b
        + 128)
        / 256
        + 128;
    cr = (131 * r
        - 110 * g
        - 21 * b
        + 128)
        / 256
        + 128;

    (*ayuv).Y = (y * 0x100) as USHORT;
    (*ayuv).Cb = (cb * 0x100) as USHORT;
    (*ayuv).Cr = (cr * 0x100) as USHORT;
    (*ayuv).Alpha = 0xffff as USHORT;
}

unsafe extern "system" fn video_mixer_processor_SetBackgroundColor(iface: *mut IMFVideoProcessor, color: COLORREF) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFVideoProcessor(iface);
    TRACE!("%p, %#lx.\n", iface, color);
    EnterCriticalSection(&raw mut (*mixer).cs);
    if (*mixer).bkgnd_color.rgba != color {
        video_mixer_rgb_to_ycbcr(color, &raw mut (*mixer).bkgnd_color.ayuv);
        (*mixer).bkgnd_color.rgba = color;
    }
    LeaveCriticalSection(&raw mut (*mixer).cs);
    return S_OK;
}

unsafe extern "system" fn video_mixer_attributes_QueryInterface(iface: *mut IMFAttributes, riid: *const IID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFAttributes(iface);
    return IMFTransform_QueryInterface(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface), riid, out);
}

unsafe extern "system" fn video_mixer_attributes_AddRef(iface: *mut IMFAttributes) -> ULONG {
    let mut mixer: *mut video_mixer = impl_from_IMFAttributes(iface);
    return IMFTransform_AddRef(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface));
}

unsafe extern "system" fn video_mixer_attributes_Release(iface: *mut IMFAttributes) -> ULONG {
    let mixer: *mut video_mixer = impl_from_IMFAttributes(iface);
    return IMFTransform_Release(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface));
}

unsafe extern "system" fn video_mixer_attributes_GetItem(iface: *mut IMFAttributes, key: REFGUID, value: *mut PROPVARIANT) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFAttributes(iface);
    TRACE!("%p, %s, %p.\n", iface, debugstr_guid(key), value);
    return IMFAttributes_GetItem((*mixer).internal_attributes, key, value);
}
unsafe extern "system" fn video_mixer_attributes_GetItemType(iface: *mut IMFAttributes, key: REFGUID, type_: *mut MF_ATTRIBUTE_TYPE) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0".as_ptr() as *const i8,
            b"video_mixer_attributes_GetItemType\0".as_ptr() as *const i8,
            2153i32,
            b"%p, %s, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key as *const _GUID),
            type_,
        );
    }
    return IMFAttributes_GetItemType((*mixer).internal_attributes, key, type_);
}

unsafe extern "system" fn video_mixer_attributes_CompareItem(
    iface: *mut IMFAttributes,
    key: *const GUID,
    value: *const PROPVARIANT,
    result: *mut BOOL,
) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0" as *const u8
                as *const i8,
            b"video_mixer_attributes_CompareItem\0" as *const u8 as *const i8,
            2163 as i32,
            b"%p, %s, %p, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_guid(key as *const _GUID),
            value,
            result,
        );
    }
    return IMFAttributes_CompareItem((*mixer).internal_attributes, key, value, result);
}

unsafe extern "system" fn video_mixer_attributes_Compare(
    mut iface: *mut IMFAttributes,
    mut theirs: *mut IMFAttributes,
    mut match_type: MF_ATTRIBUTES_MATCH_TYPE,
    mut ret: *mut BOOL,
) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0".as_ptr() as *const i8,
            b"video_mixer_attributes_Compare\0".as_ptr() as *const i8,
            2173i32,
            b"%p, %p, %d, %p.\n\0".as_ptr() as *const i8,
            iface,
            theirs,
            match_type as u32,
            ret,
        );
    }
    return IMFAttributes_Compare((*mixer).internal_attributes, theirs, match_type, ret);
}

unsafe extern "system" fn video_mixer_attributes_GetUINT32(iface: *mut IMFAttributes, key: REFGUID, value: *mut UINT32) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFAttributes(iface);

    return IMFAttributes_GetUINT32((*mixer).internal_attributes, key, value);
}

unsafe extern "system" fn video_mixer_attributes_GetUINT64(iface: *mut IMFAttributes, key: REFGUID, value: *mut UINT64) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFAttributes(iface);

    return IMFAttributes_GetUINT64((*mixer).internal_attributes, key, value);
}

unsafe extern "system" fn video_mixer_attributes_GetDouble(iface: *mut IMFAttributes, key: *const GUID, value: *mut f64) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFAttributes(iface);

    return IMFAttributes_GetDouble((*mixer).internal_attributes, key, value);
}

unsafe extern "system" fn video_mixer_attributes_GetGUID(iface: *mut IMFAttributes, key: REFGUID, value: *mut GUID) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFAttributes(iface);

    return IMFAttributes_GetGUID((*mixer).internal_attributes, key, value);
}

unsafe extern "system" fn video_mixer_attributes_GetStringLength(iface: *mut IMFAttributes, key: REFGUID, length: *mut UINT32) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFAttributes(iface);

    return IMFAttributes_GetStringLength((*mixer).internal_attributes, key, length);
}

unsafe extern "system" fn video_mixer_attributes_GetString(
    iface: *mut IMFAttributes,
    key: *const GUID,
    value: *mut WCHAR,
    size: UINT32,
    length: *mut UINT32,
) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFAttributes(iface);

    return IMFAttributes_GetString(
        (*mixer).internal_attributes,
        key,
        value,
        size,
        length,
    );
}

unsafe extern "system" fn video_mixer_attributes_GetAllocatedString(iface: *mut IMFAttributes, key: *const GUID, value: *mut *mut WCHAR, length: *mut UINT32) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFAttributes(iface);

    return IMFAttributes_GetAllocatedString((*mixer).internal_attributes, key, value, length);
}

unsafe extern "system" fn video_mixer_attributes_GetBlobSize(iface: *mut IMFAttributes, key: REFGUID, size: *mut UINT32) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFAttributes(iface);

    return IMFAttributes_GetBlobSize((*mixer).internal_attributes, key, size);
}

unsafe extern "system" fn video_mixer_attributes_GetBlob(iface: *mut IMFAttributes, key: *const GUID, buf: *mut UINT8, bufsize: UINT32, blobsize: *mut UINT32) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFAttributes(iface);

    return IMFAttributes_GetBlob((*mixer).internal_attributes, key, buf, bufsize, blobsize);
}
unsafe extern "system" fn video_mixer_attributes_GetAllocatedBlob(iface: *mut IMFAttributes, key: REFGUID, buf: *mut *mut UINT8, size: *mut UINT32) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFAttributes(iface);

    return IMFAttributes_GetAllocatedBlob((*mixer).internal_attributes, key, buf, size);
}

unsafe extern "system" fn video_mixer_attributes_GetUnknown(
    mut iface: *mut IMFAttributes,
    key: *const GUID,
    riid: *const IID,
    mut out: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"video_mixer_attributes_GetUnknown\0" as *const u8 as *const ::core::ffi::c_char,
            2275 as ::core::ffi::c_int,
            b"%p, %s, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            debugstr_guid(riid as *const _GUID),
            out,
        );
    }
    return IMFAttributes_GetUnknown(
        (*mixer).internal_attributes,
        key,
        riid,
        out as *mut LPVOID,
    );
}

unsafe extern "system" fn video_mixer_attributes_SetItem(iface: *mut IMFAttributes, key: *const GUID, value: *const PROPVARIANT) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0" as *const u8 as *const i8,
            b"video_mixer_attributes_SetItem\0" as *const u8 as *const i8,
            2284i32,
            b"%p, %s, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_guid(key as *const _GUID),
            value,
        );
    }
    return IMFAttributes_SetItem((*mixer).internal_attributes, key, value);
}

unsafe extern "system" fn video_mixer_attributes_DeleteItem(iface: *mut IMFAttributes, key: *const GUID) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFAttributes(iface);

    return IMFAttributes_DeleteItem((*mixer).internal_attributes, key);
}

unsafe extern "system" fn video_mixer_attributes_DeleteAllItems(iface: *mut IMFAttributes) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFAttributes(iface);
    return IMFAttributes_DeleteAllItems((*mixer).internal_attributes);
}

unsafe extern "system" fn video_mixer_attributes_SetUINT32(iface: *mut IMFAttributes, key: *const GUID, value: UINT32) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFAttributes(iface);

    return IMFAttributes_SetUINT32((*mixer).internal_attributes, key, value);
}

unsafe extern "system" fn video_mixer_attributes_SetUINT64(
    iface: *mut IMFAttributes,
    key: *const GUID,
    value: UINT64,
) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0".as_ptr() as *const i8,
            b"video_mixer_attributes_SetUINT64\0".as_ptr() as *const i8,
            2320i32,
            b"%p, %s, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key as *const GUID),
            wine_dbgstr_longlong(value as u64),
        );
    }
    return IMFAttributes_SetUINT64((*mixer).internal_attributes, key, value);
}

unsafe extern "system" fn video_mixer_attributes_SetDouble(iface: *mut IMFAttributes, key: *const GUID, value: f64) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0".as_ptr() as *const i8,
            b"video_mixer_attributes_SetDouble\0".as_ptr() as *const i8,
            2329i32,
            b"%p, %s, %f.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key as *const GUID),
            value,
        );
    }
    return IMFAttributes_SetDouble((*mixer).internal_attributes, key, value);
}

unsafe extern "system" fn video_mixer_attributes_SetGUID(iface: *mut IMFAttributes, key: *const GUID, value: *const GUID) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0".as_ptr() as *const i8,
            b"video_mixer_attributes_SetGUID\0".as_ptr() as *const i8,
            2338i32,
            b"%p, %s, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key as *const _GUID),
            debugstr_guid(value as *const _GUID),
        );
    }
    return IMFAttributes_SetGUID((*mixer).internal_attributes, key, value);
}

unsafe extern "system" fn video_mixer_attributes_SetString(iface: *mut IMFAttributes, key: REFGUID, value: *const WCHAR) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0".as_ptr() as *const i8,
            b"video_mixer_attributes_SetString\0".as_ptr() as *const i8,
            2347i32,
            b"%p, %s, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key as *const GUID),
            debugstr_w(value),
        );
    }
    return IMFAttributes_SetString((*mixer).internal_attributes, key, value as LPCWSTR);
}

unsafe extern "system" fn video_mixer_attributes_SetBlob(iface: *mut IMFAttributes, key: *const GUID, buf: *const UINT8, size: UINT32) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"video_mixer_attributes_SetBlob\0" as *const u8 as *const ::core::ffi::c_char,
            2356 as ::core::ffi::c_int,
            b"%p, %s, %p, %u.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            buf,
            size,
        );
    }
    return IMFAttributes_SetBlob((*mixer).internal_attributes, key, buf, size);
}
unsafe extern "system" fn video_mixer_attributes_SetUnknown(iface: *mut IMFAttributes, key: *const GUID, unknown: *mut IUnknown) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0".as_ptr() as *const i8,
            b"video_mixer_attributes_SetUnknown\0".as_ptr() as *const i8,
            2365i32,
            b"%p, %s, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key as *const _GUID),
            unknown,
        );
    }
    return IMFAttributes_SetUnknown((*mixer).internal_attributes, key, unknown);
}

unsafe extern "system" fn video_mixer_attributes_LockStore(iface: *mut IMFAttributes) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFAttributes(iface);

    return IMFAttributes_LockStore((*mixer).internal_attributes);
}

unsafe extern "system" fn video_mixer_attributes_UnlockStore(iface: *mut IMFAttributes) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFAttributes(iface);

    return IMFAttributes_UnlockStore((*mixer).internal_attributes);
}
unsafe extern "system" fn video_mixer_attributes_GetCount(
    mut iface: *mut IMFAttributes,
    mut count: *mut UINT32,
) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"video_mixer_attributes_GetCount\0" as *const u8 as *const ::core::ffi::c_char,
            2392 as ::core::ffi::c_int,
            b"%p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            count,
        );
    }
    return IMFAttributes_GetCount((*mixer).internal_attributes, count);
}

unsafe extern "system" fn video_mixer_attributes_GetItemByIndex(iface: *mut IMFAttributes, index: UINT32, key: *mut GUID, value: *mut PROPVARIANT) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFAttributes(iface);

    return IMFAttributes_GetItemByIndex((*mixer).internal_attributes, index, key, value);
}
unsafe extern "system" fn video_mixer_attributes_CopyAllItems(
    mut iface: *mut IMFAttributes,
    mut dest: *mut IMFAttributes,
) -> HRESULT {
    let mut mixer: *mut video_mixer = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"video_mixer_attributes_CopyAllItems\0" as *const u8 as *const ::core::ffi::c_char,
            2411 as ::core::ffi::c_int,
            b"%p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            dest,
        );
    }
    return IMFAttributes_CopyAllItems((*mixer).internal_attributes, dest);
}

unsafe extern "system" fn video_mixer_quality_advise_QueryInterface(iface: *mut IMFQualityAdvise, riid: *const IID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFQualityAdvise(iface);
    return IMFTransform_QueryInterface(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface), riid, out);
}

unsafe extern "system" fn video_mixer_quality_advise_AddRef(iface: *mut IMFQualityAdvise) -> ULONG {
    let mut mixer: *mut video_mixer = impl_from_IMFQualityAdvise(iface);
    return IMFTransform_AddRef(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface));
}

unsafe extern "system" fn video_mixer_quality_advise_Release(iface: *mut IMFQualityAdvise) -> ULONG {
    let mut mixer: *mut video_mixer = impl_from_IMFQualityAdvise(iface);
    return IMFTransform_Release(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface));
}
unsafe extern "system" fn video_mixer_quality_advise_SetDropMode(
    iface: *mut IMFQualityAdvise,
    mode: MF_QUALITY_DROP_MODE,
) -> HRESULT {
    // FIXME("%p, %u.\n", iface, mode);
    E_NOTIMPL
}
unsafe extern "system" fn video_mixer_quality_advise_SetQualityLevel(
    iface: *mut IMFQualityAdvise,
    level: MF_QUALITY_LEVEL,
) -> HRESULT {
    // FIXME("%p, %u.\n", iface, level);
    E_NOTIMPL
}
unsafe extern "system" fn video_mixer_quality_advise_GetDropMode(
    iface: *mut IMFQualityAdvise,
    mode: *mut MF_QUALITY_DROP_MODE,
) -> HRESULT {
    // FIXME("%p, %p.\n", iface, mode);
    E_NOTIMPL
}
unsafe extern "system" fn video_mixer_quality_advise_GetQualityLevel(
    iface: *mut IMFQualityAdvise,
    level: *mut MF_QUALITY_LEVEL,
) -> HRESULT {
    // FIXME("%p, %p.\n", iface, level);
    E_NOTIMPL
}
unsafe extern "system" fn video_mixer_quality_advise_DropTime(
    iface: *mut IMFQualityAdvise,
    interval: LONGLONG,
) -> HRESULT {
    // FIXME("%p, %s.\n", iface, wine_dbgstr_longlong(interval));
    E_NOTIMPL
}

unsafe extern "system" fn video_mixer_clock_state_sink_QueryInterface(iface: *mut IMFClockStateSink, riid: *const IID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let mixer: *mut video_mixer = impl_from_IMFClockStateSink(iface);
    return IMFTransform_QueryInterface(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface), riid, out);
}

unsafe extern "system" fn video_mixer_clock_state_sink_AddRef(iface: *mut IMFClockStateSink) -> ULONG {
    let mut mixer: *mut video_mixer = impl_from_IMFClockStateSink(iface);
    return IMFTransform_AddRef(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface));
}

unsafe extern "system" fn video_mixer_clock_state_sink_Release(iface: *mut IMFClockStateSink) -> ULONG {
    let mut mixer: *mut video_mixer = impl_from_IMFClockStateSink(iface);
    return IMFTransform_Release(core::ptr::addr_of_mut!((*mixer).IMFTransform_iface));
}
unsafe extern "system" fn video_mixer_clock_state_sink_OnClockStart(
    iface: *mut IMFClockStateSink,
    systime: MFTIME,
    offset: LONGLONG,
) -> HRESULT {
    // FIXME("%p.\n", iface);
    E_NOTIMPL
}
unsafe extern "system" fn video_mixer_clock_state_sink_OnClockStop(
    iface: *mut IMFClockStateSink,
    systime: MFTIME,
) -> HRESULT {
    // FIXME("%p.\n", iface);
    E_NOTIMPL
}
unsafe extern "system" fn video_mixer_clock_state_sink_OnClockPause(
    iface: *mut IMFClockStateSink,
    systime: MFTIME,
) -> HRESULT {
    // FIXME("%p.\n", iface);
    E_NOTIMPL
}
unsafe extern "system" fn video_mixer_clock_state_sink_OnClockRestart(
    iface: *mut IMFClockStateSink,
    systime: MFTIME,
) -> HRESULT {
    // FIXME("%p.\n", iface);
    E_NOTIMPL
}
unsafe extern "system" fn video_mixer_clock_state_sink_OnClockSetRate(
    iface: *mut IMFClockStateSink,
    systime: MFTIME,
    rate: FLOAT,
) -> HRESULT {
    // FIXME("%p, %f.\n", iface, rate);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateVideoMixer(
    owner: *mut IUnknown,
    riid_device: *const IID,
    riid: *const IID,
    obj: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/mixer.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MFCreateVideoMixer\0" as *const u8 as *const ::core::ffi::c_char,
            2591 as ::core::ffi::c_int,
            b"%p, %s, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            owner,
            debugstr_guid(riid_device as *const _GUID),
            debugstr_guid(riid as *const _GUID),
            obj,
        );
    }

    *obj = core::ptr::null_mut();

    if IsEqualIID!(riid_device, core::ptr::addr_of!(IID_IDirect3DDevice9) as *const IID) == 0 {
        return E_INVALIDARG;
    }

    return CoCreateInstance(
        core::ptr::addr_of!(CLSID_MFVideoMixer9),
        owner,
        CLSCTX_INPROC_SERVER,
        riid,
        obj,
    );
}
#[no_mangle]
pub unsafe extern "C" fn evr_mixer_create(mut outer: *mut IUnknown, mut out: *mut *mut ::core::ffi::c_void) -> HRESULT {
    let mut object: *mut video_mixer = core::ptr::null_mut::<video_mixer>();
    let mut rect: MFVideoNormalizedRect = MFVideoNormalizedRect {
        left: 0.0,
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
    };
    let mut hr: HRESULT = 0;
    object = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<video_mixer>() as size_t) as _,
    ) as *mut video_mixer;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }
    (*object).IMFTransform_iface.lpVtbl = &raw const video_mixer_transform_vtbl;
    (*object).IMFVideoDeviceID_iface.lpVtbl = &raw const video_mixer_device_id_vtbl;
    (*object).IMFTopologyServiceLookupClient_iface.lpVtbl = &raw const video_mixer_service_client_vtbl;
    (*object).IMFVideoMixerControl2_iface.lpVtbl = &raw const video_mixer_control_vtbl;
    (*object).IMFGetService_iface.lpVtbl = &raw const video_mixer_getservice_vtbl;
    (*object).IMFVideoMixerBitmap_iface.lpVtbl = &raw const video_mixer_bitmap_vtbl;
    (*object).IMFVideoPositionMapper_iface.lpVtbl = &raw const video_mixer_position_mapper_vtbl;
    (*object).IMFVideoProcessor_iface.lpVtbl = &raw const video_mixer_processor_vtbl;
    (*object).IMFAttributes_iface.lpVtbl = &raw const video_mixer_attributes_vtbl;
    (*object).IMFQualityAdvise_iface.lpVtbl = &raw const video_mixer_quality_advise_vtbl;
    (*object).IMFClockStateSink_iface.lpVtbl = &raw const video_mixer_clock_state_sink_vtbl;
    (*object).IUnknown_inner.lpVtbl = &raw const video_mixer_inner_vtbl;
    (*object).outer_unk = if !outer.is_null() {
        outer
    } else {
        &raw mut (*object).IUnknown_inner
    };
    (*object).refcount = 1;
    (*object).input_count = 1;
    (*object).lower_bound = MFT_OUTPUT_BOUND_LOWER_UNBOUNDED;
    (*object).upper_bound = (MFT_OUTPUT_BOUND_UPPER_UNBOUNDED as _);
    video_mixer_init_input(
        (&raw mut (*object).inputs as *mut input_stream).offset(0 as isize)
            as *mut input_stream,
    );
    video_mixer_update_zorder_map(object);
    video_mixer_rgb_to_ycbcr(
        (*object).bkgnd_color.rgba,
        &raw mut (*object).bkgnd_color.ayuv,
    );
    InitializeCriticalSection(&raw mut (*object).cs);
    hr = MFCreateAttributes(
        &raw mut (*object).attributes,
        0 as UINT32,
    );
    if hr < 0 as HRESULT {
        IUnknown_Release(&raw mut (*object).IUnknown_inner);
        return hr;
    }
    hr = MFCreateAttributes(
        &raw mut (*object).internal_attributes,
        0 as UINT32,
    );
    if hr < 0 as HRESULT {
        IUnknown_Release(&raw mut (*object).IUnknown_inner);
        return hr;
    }

    /* Default attributes configuration. */

    rect.top = 0.0f32;
    rect.left = rect.top;
    rect.bottom = 1.0f32;
    rect.right = rect.bottom;
    IMFAttributes_SetBlob(
        (*object).attributes,
        &raw const VIDEO_ZOOM_RECT,
        &raw const rect as *const UINT8,
        core::mem::size_of::<MFVideoNormalizedRect>() as UINT32,
    );

    IMFAttributes_SetUINT32(
        (*object).internal_attributes,
        &raw const MF_SA_D3D_AWARE,
        1 as UINT32,
    );

    *out = &raw mut (*object).IUnknown_inner as *mut core::ffi::c_void;

    return S_OK;
}

// COM vtable tables — generated from mixer.c by vtbl_gen.py
/// mixer.c:363 `static const IUnknownVtbl video_mixer_inner_vtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static video_mixer_inner_vtbl: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: Some(video_mixer_inner_QueryInterface),
    AddRef: Some(video_mixer_inner_AddRef),
    Release: Some(video_mixer_inner_Release),
};

/// mixer.c:1481 `static const IMFTransformVtbl video_mixer_transform_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_mixer_transform_vtbl: IMFTransformVtbl = IMFTransformVtbl {
    QueryInterface: Some(video_mixer_transform_QueryInterface),
    AddRef: Some(video_mixer_transform_AddRef),
    Release: Some(video_mixer_transform_Release),
    GetStreamLimits: Some(video_mixer_transform_GetStreamLimits),
    GetStreamCount: Some(video_mixer_transform_GetStreamCount),
    GetStreamIDs: Some(video_mixer_transform_GetStreamIDs),
    GetInputStreamInfo: Some(video_mixer_transform_GetInputStreamInfo),
    GetOutputStreamInfo: Some(video_mixer_transform_GetOutputStreamInfo),
    GetAttributes: Some(video_mixer_transform_GetAttributes),
    GetInputStreamAttributes: Some(video_mixer_transform_GetInputStreamAttributes),
    GetOutputStreamAttributes: Some(video_mixer_transform_GetOutputStreamAttributes),
    DeleteInputStream: Some(video_mixer_transform_DeleteInputStream),
    AddInputStreams: Some(video_mixer_transform_AddInputStreams),
    GetInputAvailableType: Some(video_mixer_transform_GetInputAvailableType),
    GetOutputAvailableType: Some(video_mixer_transform_GetOutputAvailableType),
    SetInputType: Some(video_mixer_transform_SetInputType),
    SetOutputType: Some(video_mixer_transform_SetOutputType),
    GetInputCurrentType: Some(video_mixer_transform_GetInputCurrentType),
    GetOutputCurrentType: Some(video_mixer_transform_GetOutputCurrentType),
    GetInputStatus: Some(video_mixer_transform_GetInputStatus),
    GetOutputStatus: Some(video_mixer_transform_GetOutputStatus),
    SetOutputBounds: Some(video_mixer_transform_SetOutputBounds),
    ProcessEvent: Some(video_mixer_transform_ProcessEvent),
    ProcessMessage: Some(video_mixer_transform_ProcessMessage),
    ProcessInput: Some(video_mixer_transform_ProcessInput),
    ProcessOutput: Some(video_mixer_transform_ProcessOutput),
};

/// mixer.c:1541 `static const IMFVideoDeviceIDVtbl video_mixer_device_id_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_mixer_device_id_vtbl: IMFVideoDeviceIDVtbl = IMFVideoDeviceIDVtbl {
    QueryInterface: Some(video_mixer_device_id_QueryInterface),
    AddRef: Some(video_mixer_device_id_AddRef),
    Release: Some(video_mixer_device_id_Release),
    GetDeviceID: Some(video_mixer_device_id_GetDeviceID),
};

/// mixer.c:1611 `static const IMFTopologyServiceLookupClientVtbl video_mixer_service_client_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_mixer_service_client_vtbl: IMFTopologyServiceLookupClientVtbl = IMFTopologyServiceLookupClientVtbl {
    QueryInterface: Some(video_mixer_service_client_QueryInterface),
    AddRef: Some(video_mixer_service_client_AddRef),
    Release: Some(video_mixer_service_client_Release),
    InitServicePointers: Some(video_mixer_service_client_InitServicePointers),
    ReleaseServicePointers: Some(video_mixer_service_client_ReleaseServicePointers),
};

/// mixer.c:1771 `static const IMFVideoMixerControl2Vtbl video_mixer_control_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_mixer_control_vtbl: IMFVideoMixerControl2Vtbl = IMFVideoMixerControl2Vtbl {
    QueryInterface: Some(video_mixer_control_QueryInterface),
    AddRef: Some(video_mixer_control_AddRef),
    Release: Some(video_mixer_control_Release),
    SetStreamZOrder: Some(video_mixer_control_SetStreamZOrder),
    GetStreamZOrder: Some(video_mixer_control_GetStreamZOrder),
    SetStreamOutputRect: Some(video_mixer_control_SetStreamOutputRect),
    GetStreamOutputRect: Some(video_mixer_control_GetStreamOutputRect),
    SetMixingPrefs: Some(video_mixer_control_SetMixingPrefs),
    GetMixingPrefs: Some(video_mixer_control_GetMixingPrefs),
};

/// mixer.c:1825 `static const IMFGetServiceVtbl video_mixer_getservice_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_mixer_getservice_vtbl: IMFGetServiceVtbl = IMFGetServiceVtbl {
    QueryInterface: Some(video_mixer_getservice_QueryInterface),
    AddRef: Some(video_mixer_getservice_AddRef),
    Release: Some(video_mixer_getservice_Release),
    GetService: Some(video_mixer_getservice_GetService),
};

/// mixer.c:1880 `static const IMFVideoMixerBitmapVtbl video_mixer_bitmap_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_mixer_bitmap_vtbl: IMFVideoMixerBitmapVtbl = IMFVideoMixerBitmapVtbl {
    QueryInterface: Some(video_mixer_bitmap_QueryInterface),
    AddRef: Some(video_mixer_bitmap_AddRef),
    Release: Some(video_mixer_bitmap_Release),
    SetAlphaBitmap: Some(video_mixer_bitmap_SetAlphaBitmap),
    ClearAlphaBitmap: Some(video_mixer_bitmap_ClearAlphaBitmap),
    UpdateAlphaBitmapParameters: Some(video_mixer_bitmap_UpdateAlphaBitmapParameters),
    GetAlphaBitmapParameters: Some(video_mixer_bitmap_GetAlphaBitmapParameters),
};

/// mixer.c:1917 `static const IMFVideoPositionMapperVtbl video_mixer_position_mapper_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_mixer_position_mapper_vtbl: IMFVideoPositionMapperVtbl = IMFVideoPositionMapperVtbl {
    QueryInterface: Some(video_mixer_position_mapper_QueryInterface),
    AddRef: Some(video_mixer_position_mapper_AddRef),
    Release: Some(video_mixer_position_mapper_Release),
    MapOutputCoordinateToInputStream: Some(video_mixer_position_mapper_MapOutputCoordinateToInputStream),
};

/// mixer.c:2103 `static const IMFVideoProcessorVtbl video_mixer_processor_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_mixer_processor_vtbl: IMFVideoProcessorVtbl = IMFVideoProcessorVtbl {
    QueryInterface: Some(video_mixer_processor_QueryInterface),
    AddRef: Some(video_mixer_processor_AddRef),
    Release: Some(video_mixer_processor_Release),
    GetAvailableVideoProcessorModes: Some(video_mixer_processor_GetAvailableVideoProcessorModes),
    GetVideoProcessorCaps: Some(video_mixer_processor_GetVideoProcessorCaps),
    GetVideoProcessorMode: Some(video_mixer_processor_GetVideoProcessorMode),
    SetVideoProcessorMode: Some(video_mixer_processor_SetVideoProcessorMode),
    GetProcAmpRange: Some(video_mixer_processor_GetProcAmpRange),
    GetProcAmpValues: Some(video_mixer_processor_GetProcAmpValues),
    SetProcAmpValues: Some(video_mixer_processor_SetProcAmpValues),
    GetFilteringRange: Some(video_mixer_processor_GetFilteringRange),
    GetFilteringValue: Some(video_mixer_processor_GetFilteringValue),
    SetFilteringValue: Some(video_mixer_processor_SetFilteringValue),
    GetBackgroundColor: Some(video_mixer_processor_GetBackgroundColor),
    SetBackgroundColor: Some(video_mixer_processor_SetBackgroundColor),
};

/// mixer.c:2416 `static const IMFAttributesVtbl video_mixer_attributes_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_mixer_attributes_vtbl: IMFAttributesVtbl = IMFAttributesVtbl {
    QueryInterface: Some(video_mixer_attributes_QueryInterface),
    AddRef: Some(video_mixer_attributes_AddRef),
    Release: Some(video_mixer_attributes_Release),
    GetItem: Some(video_mixer_attributes_GetItem),
    GetItemType: Some(video_mixer_attributes_GetItemType),
    CompareItem: Some(video_mixer_attributes_CompareItem),
    Compare: Some(video_mixer_attributes_Compare),
    GetUINT32: Some(video_mixer_attributes_GetUINT32),
    GetUINT64: Some(video_mixer_attributes_GetUINT64),
    GetDouble: Some(video_mixer_attributes_GetDouble),
    GetGUID: Some(video_mixer_attributes_GetGUID),
    GetStringLength: Some(video_mixer_attributes_GetStringLength),
    GetString: Some(video_mixer_attributes_GetString),
    GetAllocatedString: Some(video_mixer_attributes_GetAllocatedString),
    GetBlobSize: Some(video_mixer_attributes_GetBlobSize),
    GetBlob: Some(video_mixer_attributes_GetBlob),
    GetAllocatedBlob: Some(video_mixer_attributes_GetAllocatedBlob),
    GetUnknown: Some(video_mixer_attributes_GetUnknown),
    SetItem: Some(video_mixer_attributes_SetItem),
    DeleteItem: Some(video_mixer_attributes_DeleteItem),
    DeleteAllItems: Some(video_mixer_attributes_DeleteAllItems),
    SetUINT32: Some(video_mixer_attributes_SetUINT32),
    SetUINT64: Some(video_mixer_attributes_SetUINT64),
    SetDouble: Some(video_mixer_attributes_SetDouble),
    SetGUID: Some(video_mixer_attributes_SetGUID),
    SetString: Some(video_mixer_attributes_SetString),
    SetBlob: Some(video_mixer_attributes_SetBlob),
    SetUnknown: Some(video_mixer_attributes_SetUnknown),
    LockStore: Some(video_mixer_attributes_LockStore),
    UnlockStore: Some(video_mixer_attributes_UnlockStore),
    GetCount: Some(video_mixer_attributes_GetCount),
    GetItemByIndex: Some(video_mixer_attributes_GetItemByIndex),
    CopyAllItems: Some(video_mixer_attributes_CopyAllItems),
};

/// mixer.c:2506 `static const IMFQualityAdviseVtbl video_mixer_quality_advise_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_mixer_quality_advise_vtbl: IMFQualityAdviseVtbl = IMFQualityAdviseVtbl {
    QueryInterface: Some(video_mixer_quality_advise_QueryInterface),
    AddRef: Some(video_mixer_quality_advise_AddRef),
    Release: Some(video_mixer_quality_advise_Release),
    SetDropMode: Some(video_mixer_quality_advise_SetDropMode),
    SetQualityLevel: Some(video_mixer_quality_advise_SetQualityLevel),
    GetDropMode: Some(video_mixer_quality_advise_GetDropMode),
    GetQualityLevel: Some(video_mixer_quality_advise_GetQualityLevel),
    DropTime: Some(video_mixer_quality_advise_DropTime),
};

/// mixer.c:2577 `static const IMFClockStateSinkVtbl video_mixer_clock_state_sink_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_mixer_clock_state_sink_vtbl: IMFClockStateSinkVtbl = IMFClockStateSinkVtbl {
    QueryInterface: Some(video_mixer_clock_state_sink_QueryInterface),
    AddRef: Some(video_mixer_clock_state_sink_AddRef),
    Release: Some(video_mixer_clock_state_sink_Release),
    OnClockStart: Some(video_mixer_clock_state_sink_OnClockStart),
    OnClockStop: Some(video_mixer_clock_state_sink_OnClockStop),
    OnClockPause: Some(video_mixer_clock_state_sink_OnClockPause),
    OnClockRestart: Some(video_mixer_clock_state_sink_OnClockRestart),
    OnClockSetRate: Some(video_mixer_clock_state_sink_OnClockSetRate),
};
