use crate::mf::main::create_activation_object;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut IID_IMMDeviceEnumerator: GUID = _GUID {
    Data1: 0xa95664d2 as ::core::ffi::c_uint,
    Data2: 0x9614 as ::core::ffi::c_ushort,
    Data3: 0x4f35 as ::core::ffi::c_ushort,
    Data4: [
        0xa7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xde as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x36 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut CLSID_MMDeviceEnumerator: GUID = _GUID {
    Data1: 0xbcde0395 as ::core::ffi::c_uint,
    Data2: 0xe52f as ::core::ffi::c_ushort,
    Data3: 0x467c as ::core::ffi::c_ushort,
    Data4: [
        0x8e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x57 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x92 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x91 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x69 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut IID_IAudioClient: GUID = _GUID {
    Data1: 0x1cb9ad4c as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xdbfa as ::core::ffi::c_ushort,
    Data3: 0x4c32 as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x78 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x68 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut IID_IAudioRenderClient: GUID = _GUID {
    Data1: 0xf294acfc as ::core::ffi::c_uint,
    Data2: 0x3146 as ::core::ffi::c_ushort,
    Data3: 0x4483 as ::core::ffi::c_ushort,
    Data4: [
        0xa7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xad as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut IID_IAudioClock: GUID = _GUID {
    Data1: 0xcd63314f as ::core::ffi::c_uint,
    Data2: 0x3fba as ::core::ffi::c_ushort,
    Data3: 0x4a1b as ::core::ffi::c_ushort,
    Data4: [
        0x81 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xef as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x35 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x87 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x28 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut IID_ISimpleAudioVolume: GUID = _GUID {
    Data1: 0x87ce5498 as ::core::ffi::c_uint,
    Data2: 0x68d6 as ::core::ffi::c_ushort,
    Data3: 0x44e5 as ::core::ffi::c_ushort,
    Data4: [
        0x92 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x15 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x83 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut IID_IAudioStreamVolume: GUID = _GUID {
    Data1: 0x93014887 as ::core::ffi::c_uint,
    Data2: 0x242d as ::core::ffi::c_ushort,
    Data3: 0x4068 as ::core::ffi::c_ushort,
    Data4: [
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x15 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x93 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut sar_activate_funcs: activate_funcs = unsafe {
    activate_funcs {
        create_object: Some(
            sar_create_object
                as unsafe extern "C" fn(
                    *mut IMFAttributes,
                    *mut ::core::ffi::c_void,
                    *mut *mut IUnknown,
                ) -> HRESULT,
        ),
        shutdown_object: Some(
            sar_shutdown_object
                as unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut IUnknown) -> (),
        ),
        free_private: None,
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(mfplat): debug channel is provided by the DPRINT1!/ERR!/WARN! macros.
// COBJMACROS: no Rust equivalent, omitted.

pub const SAR_SHUT_DOWN: u32 = 0x1;
pub const SAR_PREROLLED: u32 = 0x2;
pub const SAR_SAMPLE_REQUESTED: u32 = 0x4;
pub const SAR_SEEKING: u32 = 0x8;
pub const SAR_REPORT_SEEK_TIME: u32 = 0x10;
pub const SAR_ADDING_SILENCE: u32 = 0x20;

pub const OBJECT_TYPE_SAMPLE: i32 = 0;
pub const OBJECT_TYPE_MARKER: i32 = 1;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct queued_object_sample {
    pub sample: *mut IMFSample,
    pub frame_offset: u32,
}
pub type PQUEUED_OBJECT_SAMPLE = *mut queued_object_sample;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct queued_object_marker {
    pub type_: MFSTREAMSINK_MARKER_TYPE,
    pub context: PROPVARIANT,
}
pub type PQUEUED_OBJECT_MARKER = *mut queued_object_marker;

#[repr(C)]
pub union queued_object_u {
    pub sample: queued_object_sample,
    pub marker: queued_object_marker,
}

#[repr(C)]
pub struct queued_object {
    pub entry: list,
    pub type_: i32,
    pub u: queued_object_u,
}
pub type PQUEUED_OBJECT = *mut queued_object;

#[repr(C)]
pub struct audio_renderer_stream_config {
    pub flags: u32,
    pub session_id: GUID,
}

#[repr(C)]
pub struct audio_renderer {
    pub IMFMediaSink_iface: IMFMediaSink,
    pub IMFMediaSinkPreroll_iface: IMFMediaSinkPreroll,
    pub IMFStreamSink_iface: IMFStreamSink,
    pub IMFMediaTypeHandler_iface: IMFMediaTypeHandler,
    pub IMFClockStateSink_iface: IMFClockStateSink,
    pub IMFMediaEventGenerator_iface: IMFMediaEventGenerator,
    pub IMFGetService_iface: IMFGetService,
    pub IMFSimpleAudioVolume_iface: IMFSimpleAudioVolume,
    pub IMFAudioStreamVolume_iface: IMFAudioStreamVolume,
    pub IMFAudioPolicy_iface: IMFAudioPolicy,
    pub IMFPresentationTimeSource_iface: IMFPresentationTimeSource,
    pub render_callback: IMFAsyncCallback,
    pub refcount: i32,
    pub event_queue: *mut IMFMediaEventQueue,
    pub stream_event_queue: *mut IMFMediaEventQueue,
    pub clock: *mut IMFPresentationClock,
    pub media_type: *mut IMFMediaType,
    pub current_media_type: *mut IMFMediaType,
    pub device: *mut IMMDevice,
    pub audio_client: *mut IAudioClient,
    pub audio_render_client: *mut IAudioRenderClient,
    pub stream_volume: *mut IAudioStreamVolume,
    pub audio_volume: *mut ISimpleAudioVolume,
    pub audio_clock: *mut IAudioClock,
    pub stream_config: audio_renderer_stream_config,
    pub buffer_ready_event: HANDLE,
    pub buffer_ready_key: MFWORKITEM_KEY,
    pub frame_size: u32,
    pub queued_frames: u32,
    pub max_frames: u32,
    pub queue: list,
    pub flags: u32,
    pub cs: CRITICAL_SECTION,

    pub clock_state: MFCLOCK_STATE,
    pub sample_rate: DWORD,
    pub pts: LONGLONG,
    pub seek_pts: LONGLONG,
    pub position: UINT64,
    pub audio_clock_frequency: UINT64,
    pub rate: f32,
}

unsafe extern "C" fn release_pending_object(mut object: *mut queued_object) {
    list_remove(core::ptr::addr_of_mut!((*object).entry));
    match ((*object).type_ as ::core::ffi::c_uint) as i32 {
        OBJECT_TYPE_SAMPLE => {
            if !(*object).u.sample.sample.is_null() {
                IMFSample_Release((*object).u.sample.sample);
            }
        }
        OBJECT_TYPE_MARKER => {
            PropVariantClear(core::ptr::addr_of_mut!((*object).u.marker.context));
        }
        _ => {}
    }
    free(object as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn impl_from_IMFMediaSink(mut iface: *mut IMFMediaSink) -> *mut audio_renderer {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(audio_renderer, IMFMediaSink_iface)) as *mut audio_renderer;
}

unsafe extern "C" fn impl_from_IMFMediaSinkPreroll(iface: *mut IMFMediaSinkPreroll) -> *mut audio_renderer {
    return CONTAINING_RECORD!(iface, audio_renderer, IMFMediaSinkPreroll_iface);
}

unsafe extern "C" fn impl_from_IMFClockStateSink(iface: *mut IMFClockStateSink) -> *mut audio_renderer {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(audio_renderer, IMFClockStateSink_iface)) as *mut audio_renderer;
}

unsafe extern "C" fn impl_from_IMFMediaEventGenerator(iface: *mut IMFMediaEventGenerator) -> *mut audio_renderer {
    return CONTAINING_RECORD!(iface, audio_renderer, IMFMediaEventGenerator_iface);
}

unsafe extern "C" fn impl_from_IMFGetService(iface: *mut IMFGetService) -> *mut audio_renderer {
    return CONTAINING_RECORD!(iface, audio_renderer, IMFGetService_iface);
}

unsafe extern "C" fn impl_from_IMFSimpleAudioVolume(mut iface: *mut IMFSimpleAudioVolume) -> *mut audio_renderer {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(audio_renderer, IMFSimpleAudioVolume_iface)) as *mut audio_renderer;
}

unsafe extern "C" fn impl_from_IMFAudioStreamVolume(
    mut iface: *mut IMFAudioStreamVolume,
) -> *mut audio_renderer {
    return (iface as usize).wrapping_sub(
        core::mem::offset_of!(audio_renderer, IMFAudioStreamVolume_iface),
    ) as *mut audio_renderer;
}

unsafe extern "C" fn impl_from_IMFAudioPolicy(iface: *mut IMFAudioPolicy) -> *mut audio_renderer {
    return (iface as *mut u8).sub(core::mem::offset_of!(audio_renderer, IMFAudioPolicy_iface))
        as *mut audio_renderer;
}

unsafe extern "C" fn impl_from_IMFStreamSink(iface: *mut IMFStreamSink) -> *mut audio_renderer {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(audio_renderer, IMFStreamSink_iface)) as *mut audio_renderer;
}

unsafe extern "C" fn impl_from_IMFMediaTypeHandler(
    mut iface: *mut IMFMediaTypeHandler,
) -> *mut audio_renderer {
    return (iface as usize).wrapping_sub(
        core::mem::offset_of!(audio_renderer, IMFMediaTypeHandler_iface),
    ) as *mut audio_renderer;
}

unsafe extern "C" fn impl_from_IMFPresentationTimeSource(
    iface: *mut IMFPresentationTimeSource,
) -> *mut audio_renderer {
    return CONTAINING_RECORD!(iface, audio_renderer, IMFPresentationTimeSource_iface);
}

unsafe extern "C" fn impl_from_render_callback_IMFAsyncCallback(
    mut iface: *mut IMFAsyncCallback,
) -> *mut audio_renderer {
    return (iface as usize).wrapping_sub(
        core::mem::offset_of!(audio_renderer, render_callback),
    ) as *mut audio_renderer;
}

unsafe extern "system" fn audio_renderer_sink_QueryInterface(iface: *mut IMFMediaSink, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFMediaSink(iface);

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFMediaSink)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = iface as *mut core::ffi::c_void;
    }
    else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFMediaSinkPreroll)) != 0
    {
        *obj = core::ptr::addr_of_mut!((*renderer).IMFMediaSinkPreroll_iface) as *mut core::ffi::c_void;
    }
    else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFClockStateSink)) != 0
    {
        *obj = core::ptr::addr_of_mut!((*renderer).IMFClockStateSink_iface) as *mut core::ffi::c_void;
    }
    else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFMediaEventGenerator)) != 0
    {
        *obj = core::ptr::addr_of_mut!((*renderer).IMFMediaEventGenerator_iface) as *mut core::ffi::c_void;
    }
    else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFGetService)) != 0
    {
        *obj = core::ptr::addr_of_mut!((*renderer).IMFGetService_iface) as *mut core::ffi::c_void;
    }
    else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFPresentationTimeSource)) != 0
    {
        *obj = core::ptr::addr_of_mut!((*renderer).IMFPresentationTimeSource_iface) as *mut core::ffi::c_void;
    }
    else
    {
        WARN!("Unsupported %s.\n", debugstr_guid(riid));
        *obj = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*obj as *mut IUnknown);

    return S_OK;
}

unsafe extern "system" fn audio_renderer_sink_AddRef(iface: *mut IMFMediaSink) -> ULONG {
    let renderer: *mut audio_renderer = impl_from_IMFMediaSink(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*renderer).refcount)) as ULONG;
    TRACE!("%p, refcount %lu.\n", iface, refcount);
    return refcount;
}

unsafe extern "C" fn audio_renderer_release_audio_client(renderer: *mut audio_renderer) {
    let mut obj: *mut queued_object = core::ptr::null_mut::<queued_object>();
    let mut obj2: *mut queued_object = core::ptr::null_mut::<queued_object>();
    MFCancelWorkItem((*renderer).buffer_ready_key);
    obj = ((*renderer).queue.next as *mut u8)
        .sub(core::mem::offset_of!(queued_object, entry)) as *mut queued_object;
    obj2 = ((*obj).entry.next as *mut u8)
        .sub(core::mem::offset_of!(queued_object, entry)) as *mut queued_object;
    while core::ptr::addr_of_mut!((*obj).entry) != core::ptr::addr_of_mut!((*renderer).queue) {
        release_pending_object(obj);
        obj = obj2;
        obj2 = ((*obj).entry.next as *mut u8)
            .sub(core::mem::offset_of!(queued_object, entry)) as *mut queued_object;
    }
    (*renderer).queued_frames = 0;
    (*renderer).buffer_ready_key = 0;
    if !(*renderer).audio_client.is_null() {
        IAudioClient_Stop((*renderer).audio_client);
        IAudioClient_Reset((*renderer).audio_client);
        IAudioClient_Release((*renderer).audio_client);
    }
    (*renderer).position = 0;
    (*renderer).pts = 0;
    (*renderer).audio_client = core::ptr::null_mut::<IAudioClient>();
    if !(*renderer).audio_render_client.is_null() {
        IAudioRenderClient_Release((*renderer).audio_render_client);
    }
    (*renderer).audio_render_client = core::ptr::null_mut::<IAudioRenderClient>();
    if !(*renderer).stream_volume.is_null() {
        IAudioStreamVolume_Release((*renderer).stream_volume);
    }
    (*renderer).stream_volume = core::ptr::null_mut::<IAudioStreamVolume>();
    if !(*renderer).audio_volume.is_null() {
        ISimpleAudioVolume_Release((*renderer).audio_volume);
    }
    (*renderer).audio_volume = core::ptr::null_mut::<ISimpleAudioVolume>();
    if !(*renderer).audio_clock.is_null() {
        IAudioClock_Release((*renderer).audio_clock);
    }
    (*renderer).audio_clock = core::ptr::null_mut::<IAudioClock>();
    (*renderer).flags &= !(SAR_PREROLLED as ::core::ffi::c_int) as ::core::ffi::c_uint;
}

unsafe extern "system" fn audio_renderer_sink_Release(iface: *mut IMFMediaSink) -> ULONG {
    let renderer: *mut audio_renderer = impl_from_IMFMediaSink(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*renderer).refcount) as *mut LONG,
    ) as ULONG;

    TRACE!("%p, refcount %lu.\n", iface, refcount);

    if refcount == 0 {
        if !(*renderer).event_queue.is_null() {
            IMFMediaEventQueue_Release((*renderer).event_queue);
        }
        if !(*renderer).stream_event_queue.is_null() {
            IMFMediaEventQueue_Release((*renderer).stream_event_queue);
        }
        if !(*renderer).clock.is_null() {
            IMFPresentationClock_Release((*renderer).clock);
        }
        if !(*renderer).device.is_null() {
            IMMDevice_Release((*renderer).device);
        }
        if !(*renderer).media_type.is_null() {
            IMFMediaType_Release((*renderer).media_type);
        }
        if !(*renderer).current_media_type.is_null() {
            IMFMediaType_Release((*renderer).current_media_type);
        }
        audio_renderer_release_audio_client(renderer);
        CloseHandle((*renderer).buffer_ready_event);
        DeleteCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
        free(renderer as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn audio_renderer_sink_GetCharacteristics(iface: *mut IMFMediaSink, flags: *mut DWORD) -> HRESULT {
    let mut renderer: *mut audio_renderer = impl_from_IMFMediaSink(iface);

    if ((*renderer).flags & SAR_SHUT_DOWN) != 0 {
        return MF_E_SHUTDOWN;
    }

    *flags = (MEDIASINK_FIXED_STREAMS | MEDIASINK_CAN_PREROLL) as DWORD;

    return S_OK;
}

unsafe extern "system" fn audio_renderer_sink_AddStreamSink(iface: *mut IMFMediaSink, stream_sink_id: DWORD, media_type: *mut IMFMediaType, stream_sink: *mut *mut IMFStreamSink) -> HRESULT {
    let mut renderer: *mut audio_renderer = impl_from_IMFMediaSink(iface);

    return if ((*renderer).flags & SAR_SHUT_DOWN) != 0 { MF_E_SHUTDOWN } else { MF_E_STREAMSINKS_FIXED };
}

unsafe extern "system" fn audio_renderer_sink_RemoveStreamSink(iface: *mut IMFMediaSink, stream_sink_id: DWORD) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFMediaSink(iface);

    return if ((*renderer).flags & SAR_SHUT_DOWN) != 0 { MF_E_SHUTDOWN } else { MF_E_STREAMSINKS_FIXED };
}

unsafe extern "system" fn audio_renderer_sink_GetStreamSinkCount(iface: *mut IMFMediaSink, count: *mut DWORD) -> HRESULT {
    let mut renderer: *mut audio_renderer = impl_from_IMFMediaSink(iface);
    TRACE!("%p, %p.\n", iface, count);
    if count.is_null() {
        return E_POINTER;
    }
    if (*renderer).flags & SAR_SHUT_DOWN != 0 {
        return MF_E_SHUTDOWN;
    }
    *count = 1;
    return S_OK;
}

unsafe extern "system" fn audio_renderer_sink_GetStreamSinkByIndex(iface: *mut IMFMediaSink, index: DWORD, stream: *mut *mut IMFStreamSink) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFMediaSink(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    if (*renderer).flags & SAR_SHUT_DOWN != 0 {
        hr = MF_E_SHUTDOWN;
    } else if index > 0 {
        hr = MF_E_INVALIDINDEX;
    } else {
        *stream = core::ptr::addr_of_mut!((*renderer).IMFStreamSink_iface);
        IMFStreamSink_AddRef(*stream);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}
unsafe extern "system" fn audio_renderer_sink_GetStreamSinkById(iface: *mut IMFMediaSink, stream_sink_id: DWORD, stream: *mut *mut IMFStreamSink) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFMediaSink(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    if (*renderer).flags & SAR_SHUT_DOWN != 0 {
        hr = MF_E_SHUTDOWN;
    } else if stream_sink_id > 0 {
        hr = MF_E_INVALIDSTREAMNUMBER;
    } else {
        *stream = core::ptr::addr_of_mut!((*renderer).IMFStreamSink_iface);
        IMFStreamSink_AddRef(*stream);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "C" fn audio_renderer_set_presentation_clock(
    renderer: *mut audio_renderer,
    clock: *mut IMFPresentationClock,
) {
    if !(*renderer).clock.is_null() {
        IMFPresentationClock_RemoveClockStateSink(
            (*renderer).clock,
            core::ptr::addr_of_mut!((*renderer).IMFClockStateSink_iface),
        );
        IMFPresentationClock_Release((*renderer).clock);
    }
    (*renderer).clock = clock;
    if !(*renderer).clock.is_null() {
        IMFPresentationClock_AddRef((*renderer).clock);
        IMFPresentationClock_AddClockStateSink(
            (*renderer).clock,
            core::ptr::addr_of_mut!((*renderer).IMFClockStateSink_iface),
        );
    }
}

unsafe extern "system" fn audio_renderer_sink_SetPresentationClock(iface: *mut IMFMediaSink, clock: *mut IMFPresentationClock) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFMediaSink(iface);
    let mut time_source: *mut IMFPresentationTimeSource = core::ptr::null_mut();
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    if ((*renderer).flags & SAR_SHUT_DOWN) != 0 {
        hr = MF_E_SHUTDOWN;
    } else if clock.is_null() || {
        hr = IMFPresentationClock_GetTimeSource(clock, core::ptr::addr_of_mut!(time_source));
        hr >= 0
    } {
        if !time_source.is_null() {
            IMFPresentationTimeSource_Release(time_source);
        }
        audio_renderer_set_presentation_clock(renderer, clock);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "system" fn audio_renderer_sink_GetPresentationClock(iface: *mut IMFMediaSink, clock: *mut *mut IMFPresentationClock) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFMediaSink(iface);
    let mut hr: HRESULT = S_OK;

    if clock.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    if ((*renderer).flags & SAR_SHUT_DOWN) != 0 {
        hr = MF_E_SHUTDOWN;
    } else if !(*renderer).clock.is_null() {
        *clock = (*renderer).clock;
        IMFPresentationClock_AddRef(*clock);
    } else {
        hr = MF_E_NO_CLOCK;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "system" fn audio_renderer_sink_Shutdown(iface: *mut IMFMediaSink) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFMediaSink(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    if ((*renderer).flags & SAR_SHUT_DOWN) != 0 {
        hr = MF_E_SHUTDOWN;
    } else {
        (*renderer).flags |= SAR_SHUT_DOWN;
        IMFMediaEventQueue_Shutdown((*renderer).event_queue);
        IMFMediaEventQueue_Shutdown((*renderer).stream_event_queue);
        audio_renderer_set_presentation_clock(renderer, core::ptr::null_mut());
        audio_renderer_release_audio_client(renderer);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "C" fn audio_renderer_preroll(renderer: *mut audio_renderer) {
    let mut i: u32;
    if ((*renderer).flags & SAR_PREROLLED as u32) != 0 || (*renderer).rate == 0.0f32 {
        return;
    }
    i = 0 as u32;
    while i < 2 as u32 {
        IMFMediaEventQueue_QueueEventParamVar(
            (*renderer).stream_event_queue,
            MEStreamSinkRequestSample as MediaEventType,
            core::ptr::addr_of!(GUID_NULL),
            S_OK,
            core::ptr::null::<PROPVARIANT>(),
        );
        i = i.wrapping_add(1);
    }
    (*renderer).flags |= SAR_PREROLLED as u32;
}

unsafe extern "system" fn audio_renderer_preroll_QueryInterface(iface: *mut IMFMediaSinkPreroll, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFMediaSinkPreroll(iface);
    return IMFMediaSink_QueryInterface(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface), riid, obj);
}

unsafe extern "system" fn audio_renderer_preroll_AddRef(iface: *mut IMFMediaSinkPreroll) -> ULONG {
    let renderer: *mut audio_renderer = impl_from_IMFMediaSinkPreroll(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn audio_renderer_preroll_Release(iface: *mut IMFMediaSinkPreroll) -> ULONG {
    let renderer: *mut audio_renderer = impl_from_IMFMediaSinkPreroll(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn audio_renderer_preroll_NotifyPreroll(iface: *mut IMFMediaSinkPreroll, start_time: MFTIME) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFMediaSinkPreroll(iface);

    TRACE!("%p, %s.\n", iface, debugstr_time(start_time as LONGLONG));

    if (*renderer).flags & SAR_SHUT_DOWN != 0 {
        return MF_E_SHUTDOWN;
    }

    audio_renderer_preroll(renderer);
    return IMFMediaEventQueue_QueueEventParamVar(
        (*renderer).stream_event_queue,
        MEStreamSinkPrerolled as MediaEventType,
        &raw const GUID_NULL,
        S_OK,
        core::ptr::null::<PROPVARIANT>(),
    );
}

unsafe extern "system" fn audio_renderer_events_QueryInterface(iface: *mut IMFMediaEventGenerator, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFMediaEventGenerator(iface);
    return IMFMediaSink_QueryInterface(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface), riid, obj);
}

unsafe extern "system" fn audio_renderer_events_AddRef(iface: *mut IMFMediaEventGenerator) -> ULONG {
    let renderer: *mut audio_renderer = impl_from_IMFMediaEventGenerator(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn audio_renderer_events_Release(iface: *mut IMFMediaEventGenerator) -> ULONG {
    let renderer: *mut audio_renderer = impl_from_IMFMediaEventGenerator(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn audio_renderer_events_GetEvent(iface: *mut IMFMediaEventGenerator, flags: DWORD, event: *mut *mut IMFMediaEvent) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFMediaEventGenerator(iface);

    return IMFMediaEventQueue_GetEvent((*renderer).event_queue, flags, event);
}

unsafe extern "system" fn audio_renderer_events_BeginGetEvent(iface: *mut IMFMediaEventGenerator, callback: *mut IMFAsyncCallback, state: *mut IUnknown) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFMediaEventGenerator(iface);

    return IMFMediaEventQueue_BeginGetEvent((*renderer).event_queue, callback, state);
}

unsafe extern "system" fn audio_renderer_events_EndGetEvent(
    iface: *mut IMFMediaEventGenerator,
    result: *mut IMFAsyncResult,
    event: *mut *mut IMFMediaEvent,
) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFMediaEventGenerator(iface);

    return IMFMediaEventQueue_EndGetEvent((*renderer).event_queue, result, event);
}

unsafe extern "system" fn audio_renderer_events_QueueEvent(iface: *mut IMFMediaEventGenerator, event_type: MediaEventType, ext_type: *const GUID, hr: HRESULT, value: *const PROPVARIANT) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFMediaEventGenerator(iface);

    return IMFMediaEventQueue_QueueEventParamVar((*renderer).event_queue, event_type, ext_type, hr, value);
}

unsafe extern "system" fn audio_renderer_clock_sink_QueryInterface(iface: *mut IMFClockStateSink, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFClockStateSink(iface);
    return IMFMediaSink_QueryInterface(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface), riid, obj);
}

unsafe extern "system" fn audio_renderer_clock_sink_AddRef(iface: *mut IMFClockStateSink) -> ULONG {
    let mut renderer: *mut audio_renderer = impl_from_IMFClockStateSink(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn audio_renderer_clock_sink_Release(iface: *mut IMFClockStateSink) -> ULONG {
    let renderer: *mut audio_renderer = impl_from_IMFClockStateSink(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn audio_renderer_clock_sink_OnClockStart(iface: *mut IMFClockStateSink, systime: MFTIME, offset: LONGLONG) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFClockStateSink(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!("%p, %s, %s.\n", iface, debugstr_time(systime as LONGLONG), debugstr_time(offset));

    EnterCriticalSection(&raw mut (*renderer).cs);
    if !(*renderer).audio_client.is_null() {
        if offset != (PRESENTATION_CURRENT_POSITION as _) {
            (*renderer).seek_pts = offset;
            (*renderer).flags |= ((SAR_SEEKING as ::core::ffi::c_int)
                | (SAR_REPORT_SEEK_TIME as ::core::ffi::c_int)) as ::core::ffi::c_uint;
        }
        if (*renderer).clock_state as ::core::ffi::c_uint
            != MFCLOCK_STATE_RUNNING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*renderer).rate != 0.0f32 && {
                hr = IAudioClient_Start((*renderer).audio_client);
                hr < 0 as HRESULT
            } {
                WARN!("Failed to start audio client, hr %#lx.\n", hr);
            }
            (*renderer).clock_state = MFCLOCK_STATE_RUNNING;
        }
    } else {
        hr = MF_E_NOT_INITIALIZED;
    }

    IMFMediaEventQueue_QueueEventParamVar(
        (*renderer).stream_event_queue,
        MEStreamSinkStarted as ::core::ffi::c_int as MediaEventType,
        &raw const GUID_NULL,
        hr,
        ::core::ptr::null::<PROPVARIANT>(),
    );
    if (*renderer).rate == 0.0f32 {
        IMFMediaEventQueue_QueueEventParamVar(
            (*renderer).stream_event_queue,
            MEStreamSinkScrubSampleComplete as ::core::ffi::c_int as MediaEventType,
            &raw const GUID_NULL,
            hr,
            ::core::ptr::null::<PROPVARIANT>(),
        );
    }
    if hr >= 0 as HRESULT {
        audio_renderer_preroll(renderer);
    }
    LeaveCriticalSection(&raw mut (*renderer).cs);

    return hr;
}
unsafe extern "system" fn audio_renderer_clock_sink_OnClockStop(iface: *mut IMFClockStateSink, systime: MFTIME) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFClockStateSink(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!("%p, %s.\n", iface, debugstr_time(systime as LONGLONG));

    EnterCriticalSection(&raw mut (*renderer).cs);
    if !(*renderer).audio_client.is_null() {
        if (*renderer).clock_state != MFCLOCK_STATE_STOPPED {
            hr = IAudioClient_Stop((*renderer).audio_client);
            if hr >= 0 {
                hr = IAudioClient_Reset((*renderer).audio_client);
                if hr >= 0 {
                    (*renderer).position = 0;
                    (*renderer).pts = 0;
                } else {
                    WARN!("Failed to reset audio client, hr %#lx.\n", hr);
                }
            } else {
                WARN!("Failed to stop audio client, hr %#lx.\n", hr);
            }
            (*renderer).clock_state = MFCLOCK_STATE_STOPPED;
            (*renderer).flags &= !(SAR_PREROLLED as ::core::ffi::c_uint);
        }
    } else {
        hr = MF_E_NOT_INITIALIZED;
    }

    IMFMediaEventQueue_QueueEventParamVar(
        (*renderer).stream_event_queue,
        MEStreamSinkStopped,
        &GUID_NULL,
        hr,
        core::ptr::null_mut(),
    );
    LeaveCriticalSection(&raw mut (*renderer).cs);

    return hr;
}
unsafe extern "system" fn audio_renderer_clock_sink_OnClockPause(iface: *mut IMFClockStateSink, systime: MFTIME) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFClockStateSink(iface);
    let mut hr: HRESULT = 0;

    TRACE!("%p, %s.\n", iface, debugstr_time(systime as LONGLONG));

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    if (*renderer).clock_state == MFCLOCK_STATE_RUNNING {
        if !(*renderer).audio_client.is_null() {
            hr = IAudioClient_Stop((*renderer).audio_client);
            if hr < 0 {
                WARN!("Failed to stop audio client, hr %#lx.\n", hr);
            } else {
                hr = S_OK;
            }
            (*renderer).clock_state = MFCLOCK_STATE_PAUSED;
        } else {
            hr = MF_E_NOT_INITIALIZED;
        }

        IMFMediaEventQueue_QueueEventParamVar(
            (*renderer).stream_event_queue,
            MEStreamSinkPaused,
            &raw const GUID_NULL,
            hr,
            core::ptr::null(),
        );
    } else {
        hr = MF_E_INVALID_STATE_TRANSITION;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    hr
}
unsafe extern "system" fn audio_renderer_clock_sink_OnClockRestart(iface: *mut IMFClockStateSink, systime: MFTIME) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFClockStateSink(iface);
    let mut preroll: BOOL = (FALSE as i32);
    let mut hr: HRESULT = S_OK;

    TRACE!("%p, %s.\n", iface, debugstr_time(systime as LONGLONG));

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    if !(*renderer).audio_client.is_null() {
        preroll = ((*renderer).clock_state != MFCLOCK_STATE_RUNNING) as BOOL;
        if preroll != 0 {
            if (*renderer).rate != 0.0f32 && {
                hr = IAudioClient_Start((*renderer).audio_client);
                hr < 0
            } {
                WARN!("Failed to start audio client, hr %#lx.\n", hr);
            }
            (*renderer).clock_state = MFCLOCK_STATE_RUNNING;
        }
    } else {
        hr = MF_E_NOT_INITIALIZED;
    }

    IMFMediaEventQueue_QueueEventParamVar(
        (*renderer).stream_event_queue,
        MEStreamSinkStarted as MediaEventType,
        &raw const GUID_NULL,
        hr,
        core::ptr::null(),
    );
    if (*renderer).rate == 0.0f32 {
        IMFMediaEventQueue_QueueEventParamVar(
            (*renderer).stream_event_queue,
            MEStreamSinkScrubSampleComplete as MediaEventType,
            &raw const GUID_NULL,
            hr,
            core::ptr::null(),
        );
    }
    if preroll != 0 {
        audio_renderer_preroll(renderer);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "system" fn audio_renderer_clock_sink_OnClockSetRate(iface: *mut IMFClockStateSink, systime: MFTIME, rate: f32) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFClockStateSink(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    if rate == 0.0 || rate == 1.0 {
        (*renderer).rate = rate;
    } else {
        WARN!("%f is an unsupported rate.\n", rate as core::ffi::c_double);
        hr = MF_E_UNSUPPORTED_RATE;
    }

    if hr == S_OK {
        IMFMediaEventQueue_QueueEventParamVar(
            (*renderer).stream_event_queue,
            MEStreamSinkRateChanged as i32 as MediaEventType,
            core::ptr::addr_of!(GUID_NULL),
            hr,
            core::ptr::null::<PROPVARIANT>(),
        );
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "system" fn audio_renderer_get_service_QueryInterface(
    iface: *mut IMFGetService,
    riid: *const IID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFGetService(iface);
    return IMFMediaSink_QueryInterface(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface), riid, obj);
}

unsafe extern "system" fn audio_renderer_get_service_AddRef(iface: *mut IMFGetService) -> ULONG {
    let renderer: *mut audio_renderer = impl_from_IMFGetService(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn audio_renderer_get_service_Release(iface: *mut IMFGetService) -> ULONG {
    let renderer: *mut audio_renderer = impl_from_IMFGetService(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}
unsafe extern "system" fn audio_renderer_get_service_GetService(
    iface: *mut IMFGetService,
    service: *const GUID,
    riid: *const IID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFGetService(iface);

    *obj = core::ptr::null_mut();

    if IsEqualGUID!(service, core::ptr::addr_of!(MR_POLICY_VOLUME_SERVICE)) != false
        && IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFSimpleAudioVolume)) != 0
    {
        *obj = core::ptr::addr_of_mut!((*renderer).IMFSimpleAudioVolume_iface) as *mut core::ffi::c_void;
    } else if IsEqualGUID!(service, core::ptr::addr_of!(MR_STREAM_VOLUME_SERVICE)) != false
        && IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAudioStreamVolume)) != 0
    {
        *obj = core::ptr::addr_of_mut!((*renderer).IMFAudioStreamVolume_iface) as *mut core::ffi::c_void;
    } else if IsEqualGUID!(service, core::ptr::addr_of!(MR_AUDIO_POLICY_SERVICE)) != false
        && IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAudioPolicy)) != 0
    {
        *obj = core::ptr::addr_of_mut!((*renderer).IMFAudioPolicy_iface) as *mut core::ffi::c_void;
    } else {
        FIXME!(
            "Unsupported service %s, interface %s.\n",
            debugstr_guid(service),
            debugstr_guid(riid)
        );
    }

    if !(*obj).is_null() {
        IUnknown_AddRef(*obj as *mut IUnknown);
    }

    return if !(*obj).is_null() {
        S_OK
    } else {
        E_NOINTERFACE
    };
}

unsafe extern "system" fn audio_renderer_simple_volume_QueryInterface(iface: *mut IMFSimpleAudioVolume, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFSimpleAudioVolume)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = iface as *mut core::ffi::c_void;
        IMFSimpleAudioVolume_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn audio_renderer_simple_volume_AddRef(iface: *mut IMFSimpleAudioVolume) -> ULONG {
    let renderer: *mut audio_renderer = impl_from_IMFSimpleAudioVolume(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn audio_renderer_simple_volume_Release(iface: *mut IMFSimpleAudioVolume) -> ULONG {
    let renderer: *mut audio_renderer = impl_from_IMFSimpleAudioVolume(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn audio_renderer_simple_volume_SetMasterVolume(iface: *mut IMFSimpleAudioVolume, level: f32) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFSimpleAudioVolume(iface);
    let mut hr: HRESULT = S_OK;
    TRACE!("%p, %f.\n", iface, level as f64);
    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    if !(*renderer).audio_volume.is_null() {
        hr = ISimpleAudioVolume_SetMasterVolume((*renderer).audio_volume, level, core::ptr::null::<GUID>());
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    return hr;
}

unsafe extern "system" fn audio_renderer_simple_volume_GetMasterVolume(iface: *mut IMFSimpleAudioVolume, level: *mut f32) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFSimpleAudioVolume(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!("%p, %p.\n", iface, level);

    if level.is_null() {
        return E_POINTER;
    }

    *level = 0.0f32;

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    if !(*renderer).audio_volume.is_null() {
        hr = ISimpleAudioVolume_GetMasterVolume((*renderer).audio_volume, level);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "system" fn audio_renderer_simple_volume_SetMute(
    iface: *mut IMFSimpleAudioVolume,
    mute: BOOL,
) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFSimpleAudioVolume(iface);
    let mut hr: HRESULT = S_OK;

    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/sar.c\0".as_ptr() as *const i8,
            b"audio_renderer_simple_volume_SetMute\0".as_ptr() as *const i8,
            950i32,
            b"%p, %d.\n\0".as_ptr() as *const i8,
            iface,
            mute,
        );
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    if !(*renderer).audio_volume.is_null() {
        hr = ISimpleAudioVolume_SetMute(
            (*renderer).audio_volume,
            mute,
            core::ptr::null::<GUID>(),
        );
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "system" fn audio_renderer_simple_volume_GetMute(iface: *mut IMFSimpleAudioVolume, mute: *mut BOOL) -> HRESULT {
    let mut renderer: *mut audio_renderer = impl_from_IMFSimpleAudioVolume(iface);
    let mut hr: HRESULT = S_OK;

    if mute.is_null() {
        return E_POINTER;
    }

    *mute = (FALSE as i32);

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    if !(*renderer).audio_volume.is_null() {
        hr = ISimpleAudioVolume_GetMute((*renderer).audio_volume, mute);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "system" fn audio_renderer_stream_volume_QueryInterface(iface: *mut IMFAudioStreamVolume, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAudioStreamVolume) as *const GUID) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown) as *const GUID) != 0
    {
        *obj = iface as *mut core::ffi::c_void;
        IMFAudioStreamVolume_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn audio_renderer_stream_volume_AddRef(iface: *mut IMFAudioStreamVolume) -> ULONG {
    let renderer: *mut audio_renderer = impl_from_IMFAudioStreamVolume(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn audio_renderer_stream_volume_Release(iface: *mut IMFAudioStreamVolume) -> ULONG {
    let renderer: *mut audio_renderer = impl_from_IMFAudioStreamVolume(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn audio_renderer_stream_volume_GetChannelCount(iface: *mut IMFAudioStreamVolume, count: *mut UINT32) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFAudioStreamVolume(iface);
    let mut hr: HRESULT = S_OK;

    if count.is_null() {
        return E_POINTER;
    }

    *count = 0;

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    if !(*renderer).stream_volume.is_null() {
        hr = IAudioStreamVolume_GetChannelCount((*renderer).stream_volume, count);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "system" fn audio_renderer_stream_volume_SetChannelVolume(iface: *mut IMFAudioStreamVolume, index: UINT32, level: f32) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFAudioStreamVolume(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    if !(*renderer).stream_volume.is_null() {
        hr = IAudioStreamVolume_SetChannelVolume((*renderer).stream_volume, index, level);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "system" fn audio_renderer_stream_volume_GetChannelVolume(iface: *mut IMFAudioStreamVolume, index: UINT32, level: *mut f32) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFAudioStreamVolume(iface);
    let mut hr: HRESULT = S_OK;

    if level.is_null() {
        return E_POINTER;
    }

    *level = 0.0f32;

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    if !(*renderer).stream_volume.is_null() {
        hr = IAudioStreamVolume_GetChannelVolume((*renderer).stream_volume, index, level);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "system" fn audio_renderer_stream_volume_SetAllVolumes(iface: *mut IMFAudioStreamVolume, count: UINT32, volumes: *const f32) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFAudioStreamVolume(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!("%p, %u, %p.\n", iface, count, volumes);

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    if !(*renderer).stream_volume.is_null() {
        hr = IAudioStreamVolume_SetAllVolumes((*renderer).stream_volume, count, volumes);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "system" fn audio_renderer_stream_volume_GetAllVolumes(iface: *mut IMFAudioStreamVolume, count: UINT32, volumes: *mut f32) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFAudioStreamVolume(iface);
    let mut hr: HRESULT = S_OK;

    if volumes.is_null() {
        return E_POINTER;
    }

    if count != 0 {
        memset(
            volumes as *mut core::ffi::c_void,
            0 as core::ffi::c_int,
            core::mem::size_of::<f32>().wrapping_mul(count as usize),
        );
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    if !(*renderer).stream_volume.is_null() {
        hr = IAudioStreamVolume_GetAllVolumes((*renderer).stream_volume, count, volumes);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "system" fn audio_renderer_policy_QueryInterface(iface: *mut IMFAudioPolicy, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAudioPolicy)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = iface as *mut core::ffi::c_void;
        IMFAudioPolicy_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn audio_renderer_policy_AddRef(iface: *mut IMFAudioPolicy) -> ULONG {
    let renderer: *mut audio_renderer = impl_from_IMFAudioPolicy(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn audio_renderer_policy_Release(iface: *mut IMFAudioPolicy) -> ULONG {
    let renderer: *mut audio_renderer = impl_from_IMFAudioPolicy(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}
unsafe extern "system" fn audio_renderer_policy_SetGroupingParam(
    iface: *mut IMFAudioPolicy,
    param: REFGUID,
) -> HRESULT {
    // FIXME("%p, %s.\n", iface, debugstr_guid(param));
    E_NOTIMPL
}
unsafe extern "system" fn audio_renderer_policy_GetGroupingParam(
    iface: *mut IMFAudioPolicy,
    param: *mut GUID,
) -> HRESULT {
    // FIXME("%p, %p.\n", iface, param);
    E_NOTIMPL
}
unsafe extern "system" fn audio_renderer_policy_SetDisplayName(
    iface: *mut IMFAudioPolicy,
    name: *const WCHAR,
) -> HRESULT {
    // FIXME("%p, %s.\n", iface, debugstr_w(name));
    E_NOTIMPL
}
unsafe extern "system" fn audio_renderer_policy_GetDisplayName(
    iface: *mut IMFAudioPolicy,
    name: *mut *mut WCHAR,
) -> HRESULT {
    // FIXME("%p, %p.\n", iface, name);
    E_NOTIMPL
}
unsafe extern "system" fn audio_renderer_policy_SetIconPath(
    iface: *mut IMFAudioPolicy,
    path: *const WCHAR,
) -> HRESULT {
    // FIXME("%p, %s.\n", iface, debugstr_w(path));
    E_NOTIMPL
}
unsafe extern "system" fn audio_renderer_policy_GetIconPath(
    iface: *mut IMFAudioPolicy,
    path: *mut *mut WCHAR,
) -> HRESULT {
    // FIXME("%p, %p.\n", iface, path);
    E_NOTIMPL
}

unsafe extern "system" fn audio_renderer_time_source_QueryInterface(iface: *mut IMFPresentationTimeSource, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFPresentationTimeSource(iface);
    return IMFMediaSink_QueryInterface(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface), riid, obj);
}

unsafe extern "system" fn audio_renderer_time_source_AddRef(mut iface: *mut IMFPresentationTimeSource) -> ULONG {
    let mut renderer: *mut audio_renderer = impl_from_IMFPresentationTimeSource(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn audio_renderer_time_source_Release(iface: *mut IMFPresentationTimeSource) -> ULONG {
    let renderer: *mut audio_renderer = impl_from_IMFPresentationTimeSource(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}
unsafe extern "system" fn audio_renderer_time_source_GetClockCharacteristics(
    iface: *mut IMFPresentationTimeSource,
    flags: *mut DWORD,
) -> HRESULT {
    *flags = MFCLOCK_CHARACTERISTICS_FLAG_FREQUENCY_10MHZ;

    return S_OK;
}

unsafe extern "C" fn audio_renderer_get_correlated_time(
    renderer: *mut audio_renderer,
    clock_time: *mut LONGLONG,
    system_time: *mut MFTIME,
) -> HRESULT {
    let mut position: UINT64 = 0;
    let mut counter: UINT64 = 0;
    let mut time: LONGLONG = 0;
    let mut hr: HRESULT = 0;

    hr = IAudioClock_GetPosition((*renderer).audio_clock, core::ptr::addr_of_mut!(position), core::ptr::addr_of_mut!(counter));
    if hr < 0 {
        return hr;
    }

    time = ((*renderer).pts as INT64
        + (position.wrapping_sub((*renderer).position) as INT64) * (MFCLOCK_FREQUENCY_HNS as INT64)
            / ((*renderer).audio_clock_frequency as INT64)) as LONGLONG;
    if ((*renderer).flags & (SAR_REPORT_SEEK_TIME as u32)) != 0 {
        if time < (*renderer).seek_pts {
            time = (*renderer).seek_pts;
        } else {
            (*renderer).flags &= !((SAR_REPORT_SEEK_TIME as u32));
        }
    }

    if !clock_time.is_null() {
        *clock_time = time;
    }
    if !system_time.is_null() {
        *system_time = counter as MFTIME;
    }

    return S_OK;
}
unsafe extern "system" fn audio_renderer_time_source_GetCorrelatedTime(iface: *mut IMFPresentationTimeSource, reserved: DWORD, clock_time: *mut LONGLONG, system_time: *mut MFTIME) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFPresentationTimeSource(iface);
    let mut hr: HRESULT;

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    if (*renderer).audio_clock.is_null() {
        hr = MF_E_NOT_INITIALIZED;
    } else {
        hr = audio_renderer_get_correlated_time(renderer, clock_time, system_time);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "system" fn audio_renderer_time_source_GetContinuityKey(iface: *mut IMFPresentationTimeSource, key: *mut DWORD) -> HRESULT {
    *key = 0;

    return S_OK;
}

unsafe extern "system" fn audio_renderer_time_source_GetState(iface: *mut IMFPresentationTimeSource, reserved: DWORD, state: *mut MFCLOCK_STATE) -> HRESULT {
    let mut renderer: *mut audio_renderer = impl_from_IMFPresentationTimeSource(iface);

    *state = (*renderer).clock_state;

    return S_OK;
}

unsafe extern "system" fn audio_renderer_time_source_GetProperties(
    mut iface: *mut IMFPresentationTimeSource,
    mut props: *mut MFCLOCK_PROPERTIES,
) -> HRESULT {
    if props.is_null() {
        return E_POINTER;
    }
    memset(
        props as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<MFCLOCK_PROPERTIES>() as usize,
    );
    (*props).qwClockFrequency = MFCLOCK_FREQUENCY_HNS as UINT64;
    (*props).dwClockTolerance = MFCLOCK_TOLERANCE_UNKNOWN as DWORD;
    (*props).dwClockJitter = 1 as DWORD;

    return S_OK;
}

unsafe extern "system" fn audio_renderer_time_source_GetUnderlyingClock(iface: *mut IMFPresentationTimeSource, ppClock: *mut *mut IMFClock) -> HRESULT {
    if ppClock.is_null() {
        return E_POINTER;
    }

    *ppClock = core::ptr::null_mut();

    return MF_E_NO_CLOCK;
}

unsafe extern "C" fn sar_create_mmdevice(
    attributes: *mut IMFAttributes,
    renderer: *mut audio_renderer,
) -> HRESULT {
    let mut endpoint: *mut WCHAR = core::ptr::null_mut();
    let mut length: u32 = 0;
    let mut role: u32 = eMultimedia as u32;
    let mut devenum: *mut IMMDeviceEnumerator = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    if !attributes.is_null() {
        /* Mutually exclusive attributes. */
        if (IMFAttributes_GetItem(attributes, &raw const MF_AUDIO_RENDERER_ATTRIBUTE_ENDPOINT_ROLE, core::ptr::null_mut::<PROPVARIANT>()) >= 0)
            && (IMFAttributes_GetItem(attributes, &raw const MF_AUDIO_RENDERER_ATTRIBUTE_ENDPOINT_ID, core::ptr::null_mut::<PROPVARIANT>()) >= 0)
        {
            return E_INVALIDARG;
        }
    }

    hr = CoCreateInstance(
        &raw const CLSID_MMDeviceEnumerator,
        (core::ptr::null_mut::<IUnknown>()).cast(),
        CLSCTX_INPROC_SERVER as DWORD,
        &raw const IID_IMMDeviceEnumerator,
        &raw mut devenum as *mut LPVOID,
    );
    if hr < 0 {
        return hr;
    }

    role = eMultimedia as u32;
    if !attributes.is_null()
        && IMFAttributes_GetUINT32(attributes, &raw const MF_AUDIO_RENDERER_ATTRIBUTE_ENDPOINT_ROLE, &raw mut role) >= 0
    {
        TRACE!("Specified role %d.\n", role);
    }

    if !attributes.is_null()
        && IMFAttributes_GetAllocatedString(
            attributes,
            &raw const MF_AUDIO_RENDERER_ATTRIBUTE_ENDPOINT_ID,
            &raw mut endpoint,
            &raw mut length,
        ) >= 0
    {
        TRACE!("Specified end point %s.\n", debugstr_w(endpoint));
        hr = IMMDeviceEnumerator_GetDevice(devenum, endpoint as LPCWSTR, &raw mut (*renderer).device);
        CoTaskMemFree(endpoint as LPVOID);
    } else {
        hr = IMMDeviceEnumerator_GetDefaultAudioEndpoint(
            devenum,
            eRender,
            role as ERole,
            &raw mut (*renderer).device,
        );
    }

    /* Configuration attributes to be used later for audio client initialization. */
    if !attributes.is_null() {
        IMFAttributes_GetUINT32(
            attributes,
            &raw const MF_AUDIO_RENDERER_ATTRIBUTE_FLAGS,
            &raw mut (*renderer).stream_config.flags,
        );
        IMFAttributes_GetGUID(
            attributes,
            &raw const MF_AUDIO_RENDERER_ATTRIBUTE_SESSION_ID,
            &raw mut (*renderer).stream_config.session_id,
        );
    }

    if hr < 0 {
        hr = MF_E_NO_AUDIO_PLAYBACK_DEVICE;
    }

    IMMDeviceEnumerator_Release(devenum);

    return hr;
}

unsafe extern "system" fn audio_renderer_stream_QueryInterface(iface: *mut IMFStreamSink, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFStreamSink(iface);

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFStreamSink)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFMediaEventGenerator)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = core::ptr::addr_of_mut!((*renderer).IMFStreamSink_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFMediaTypeHandler)) != 0 {
        *obj = core::ptr::addr_of_mut!((*renderer).IMFMediaTypeHandler_iface) as *mut core::ffi::c_void;
    } else {
        WARN!("Unsupported %s.\n", debugstr_guid(riid));
        *obj = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*obj as *mut IUnknown);

    return S_OK;
}

unsafe extern "system" fn audio_renderer_stream_AddRef(iface: *mut IMFStreamSink) -> ULONG {
    let renderer: *mut audio_renderer = impl_from_IMFStreamSink(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}
unsafe extern "system" fn audio_renderer_stream_Release(iface: *mut IMFStreamSink) -> ULONG {
    let renderer: *mut audio_renderer = impl_from_IMFStreamSink(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn audio_renderer_stream_GetEvent(iface: *mut IMFStreamSink, flags: DWORD, event: *mut *mut IMFMediaEvent) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFStreamSink(iface);

    if (*renderer).flags & SAR_SHUT_DOWN != 0 {
        return MF_E_STREAMSINK_REMOVED;
    }

    return IMFMediaEventQueue_GetEvent((*renderer).stream_event_queue, flags, event);
}

unsafe extern "system" fn audio_renderer_stream_BeginGetEvent(
    iface: *mut IMFStreamSink,
    callback: *mut IMFAsyncCallback,
    state: *mut IUnknown,
) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFStreamSink(iface);
    TRACE!("%p, %p, %p.\n", iface, callback, state);
    if (*renderer).flags & SAR_SHUT_DOWN != 0 {
        return MF_E_STREAMSINK_REMOVED;
    }
    return IMFMediaEventQueue_BeginGetEvent((*renderer).stream_event_queue, callback, state);
}

unsafe extern "system" fn audio_renderer_stream_EndGetEvent(
    iface: *mut IMFStreamSink,
    result: *mut IMFAsyncResult,
    event: *mut *mut IMFMediaEvent,
) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFStreamSink(iface);

    if (*renderer).flags & SAR_SHUT_DOWN != 0 {
        return MF_E_STREAMSINK_REMOVED;
    }

    return IMFMediaEventQueue_EndGetEvent((*renderer).stream_event_queue, result, event);
}

unsafe extern "system" fn audio_renderer_stream_QueueEvent(
    iface: *mut IMFStreamSink,
    event_type: MediaEventType,
    ext_type: *const GUID,
    hr: HRESULT,
    value: *const PROPVARIANT,
) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFStreamSink(iface);

    if (*renderer).flags & SAR_SHUT_DOWN != 0 {
        return MF_E_STREAMSINK_REMOVED;
    }

    return IMFMediaEventQueue_QueueEventParamVar((*renderer).stream_event_queue, event_type, ext_type, hr, value);
}

unsafe extern "system" fn audio_renderer_stream_GetMediaSink(iface: *mut IMFStreamSink, sink: *mut *mut IMFMediaSink) -> HRESULT {
    let mut renderer: *mut audio_renderer = impl_from_IMFStreamSink(iface);

    if ((*renderer).flags & SAR_SHUT_DOWN) != 0 {
        return MF_E_STREAMSINK_REMOVED;
    }

    *sink = core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface);
    IMFMediaSink_AddRef(*sink);

    return S_OK;
}

unsafe extern "system" fn audio_renderer_stream_GetIdentifier(iface: *mut IMFStreamSink, identifier: *mut DWORD) -> HRESULT {
    let mut renderer: *mut audio_renderer = impl_from_IMFStreamSink(iface);

    if ((*renderer).flags & SAR_SHUT_DOWN) != 0 {
        return MF_E_STREAMSINK_REMOVED;
    }

    *identifier = 0;

    return S_OK;
}

unsafe extern "system" fn audio_renderer_stream_GetMediaTypeHandler(iface: *mut IMFStreamSink, handler: *mut *mut IMFMediaTypeHandler) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFStreamSink(iface);

    if handler.is_null() {
        return E_POINTER;
    }

    if ((*renderer).flags & SAR_SHUT_DOWN) != 0 {
        return MF_E_STREAMSINK_REMOVED;
    }

    *handler = core::ptr::addr_of_mut!((*renderer).IMFMediaTypeHandler_iface);
    IMFMediaTypeHandler_AddRef(*handler);

    return S_OK;
}

unsafe extern "C" fn stream_queue_sample(
    renderer: *mut audio_renderer,
    sample: *mut IMFSample,
) -> HRESULT {
    let mut sample_len: DWORD = 0;
    let mut sample_frames: DWORD = 0;
    let mut buffer_count: DWORD = 0;
    let mut object: *mut queued_object = core::ptr::null_mut::<queued_object>();
    let mut start_time: LONGLONG = 0;
    let mut hr: HRESULT = 0;

    hr = IMFSample_GetBufferCount(sample, core::ptr::addr_of_mut!(buffer_count));
    if hr < 0 {
        return hr;
    }

    if buffer_count == 0 {
        return E_INVALIDARG;
    }

    hr = IMFSample_GetTotalLength(sample, core::ptr::addr_of_mut!(sample_len));
    if hr < 0 {
        return hr;
    }

    hr = IMFSample_GetSampleTime(sample, core::ptr::addr_of_mut!(start_time));
    if hr < 0 {
        return MF_E_INVALID_TIMESTAMP;
    }

    sample_frames = sample_len.wrapping_div((*renderer).frame_size as DWORD);

    /* Discard samples that are prior to our seeking location */
    if (*renderer).flags & (SAR_SEEKING as ::core::ffi::c_uint) != 0
        && (start_time
            + sample_frames as LONGLONG * (MFCLOCK_FREQUENCY_HNS as LONGLONG)
                / (*renderer).sample_rate as LONGLONG)
            < (*renderer).seek_pts
    {
        return S_OK;
    }

    (*renderer).flags &= !(SAR_SEEKING as ::core::ffi::c_uint);

    object = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<queued_object>() as size_t) as _,
    ) as *mut queued_object;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).type_ = OBJECT_TYPE_SAMPLE;
    (*object).u.sample.sample = sample;
    IMFSample_AddRef((*object).u.sample.sample);

    list_add_tail(
        core::ptr::addr_of_mut!((*renderer).queue),
        core::ptr::addr_of_mut!((*object).entry),
    );
    (*renderer).queued_frames = ((*renderer).queued_frames as DWORD)
        .wrapping_add(sample_frames) as ::core::ffi::c_uint;

    return S_OK;
}

unsafe extern "system" fn audio_renderer_stream_ProcessSample(iface: *mut IMFStreamSink, sample: *mut IMFSample) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFStreamSink(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!("%p, %p.\n", iface, sample);

    if sample.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(&raw mut (*renderer).cs);

    if (*renderer).flags & SAR_SHUT_DOWN != 0 {
        hr = MF_E_STREAMSINK_REMOVED;
    } else {
        if (*renderer).clock_state == MFCLOCK_STATE_RUNNING {
            hr = stream_queue_sample(renderer, sample);
        }
        (*renderer).flags &= !SAR_SAMPLE_REQUESTED;

        if (*renderer).queued_frames < (*renderer).max_frames && (*renderer).clock_state == MFCLOCK_STATE_RUNNING {
            IMFMediaEventQueue_QueueEventParamVar(
                (*renderer).stream_event_queue,
                MEStreamSinkRequestSample as MediaEventType,
                &raw const GUID_NULL,
                S_OK,
                core::ptr::null::<PROPVARIANT>(),
            );
            (*renderer).flags |= SAR_SAMPLE_REQUESTED;
        }
    }

    LeaveCriticalSection(&raw mut (*renderer).cs);

    return hr;
}

unsafe extern "C" fn stream_place_marker(
    renderer: *mut audio_renderer,
    marker_type: MFSTREAMSINK_MARKER_TYPE,
    context_value: *const PROPVARIANT,
) -> HRESULT {
    let mut marker: *mut queued_object = core::ptr::null_mut::<queued_object>();
    let mut hr: HRESULT = S_OK;

    marker = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<queued_object>() as size_t) as _,
    ) as *mut queued_object;
    if marker.is_null() {
        return E_OUTOFMEMORY;
    }

    (*marker).type_ = OBJECT_TYPE_MARKER;
    (*marker).u.marker.type_ = marker_type;
    PropVariantInit!(&raw mut (*marker).u.marker.context);
    if !context_value.is_null() {
        hr = PropVariantCopy(&raw mut (*marker).u.marker.context, context_value);
    }
    if hr >= 0 {
        list_add_tail(&raw mut (*renderer).queue, &raw mut (*marker).entry);
    } else {
        release_pending_object(marker);
    }

    return hr;
}

unsafe extern "system" fn audio_renderer_stream_PlaceMarker(
    iface: *mut IMFStreamSink,
    marker_type: MFSTREAMSINK_MARKER_TYPE,
    marker_value: *const PROPVARIANT,
    context_value: *const PROPVARIANT,
) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFStreamSink(iface);
    let mut hr: HRESULT;

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    if ((*renderer).flags & SAR_SHUT_DOWN) != 0 {
        hr = MF_E_STREAMSINK_REMOVED;
    } else {
        hr = stream_place_marker(renderer, marker_type, context_value);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "system" fn audio_renderer_stream_Flush(mut iface: *mut IMFStreamSink) -> HRESULT {
    let mut renderer: *mut audio_renderer = impl_from_IMFStreamSink(iface);
    let mut obj: *mut queued_object = core::ptr::null_mut::<queued_object>();
    let mut obj2: *mut queued_object = core::ptr::null_mut::<queued_object>();
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(&raw mut (*renderer).cs);
    if (*renderer).flags & SAR_SHUT_DOWN != 0 {
        hr = MF_E_STREAMSINK_REMOVED;
    } else {
        obj = ((*renderer).queue.next as *mut u8).offset(
            -(&raw mut (*core::ptr::null_mut::<queued_object>()).entry as usize as isize),
        ) as *mut queued_object;
        obj2 = ((*obj).entry.next as *mut u8).offset(
            -(&raw mut (*core::ptr::null_mut::<queued_object>()).entry as usize as isize),
        ) as *mut queued_object;
        while &raw mut (*obj).entry != &raw mut (*renderer).queue {
            if (*obj).type_ == OBJECT_TYPE_MARKER {
                IMFMediaEventQueue_QueueEventParamVar(
                    (*renderer).stream_event_queue,
                    MEStreamSinkMarker,
                    &raw const GUID_NULL,
                    S_OK,
                    &raw mut (*obj).u.marker.context,
                );
            }
            release_pending_object(obj);
            obj = obj2;
            obj2 = ((*obj).entry.next as *mut u8).offset(
                -(&raw mut (*core::ptr::null_mut::<queued_object>()).entry as usize as isize),
            ) as *mut queued_object;
        }
        (*renderer).queued_frames = 0;
        if !(*renderer).audio_client.is_null() {
            hr = IAudioClient_Reset((*renderer).audio_client);
            if hr >= 0 {
                (*renderer).position = 0;
                (*renderer).pts = 0;
            } else {
                WARN!("Failed to reset audio client, hr %#lx.\n", hr);
            }
        }
    }
    LeaveCriticalSection(&raw mut (*renderer).cs);

    return hr;
}

unsafe extern "system" fn audio_renderer_stream_type_handler_QueryInterface(
    iface: *mut IMFMediaTypeHandler,
    riid: REFIID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFMediaTypeHandler(iface);
    return IMFStreamSink_QueryInterface(core::ptr::addr_of_mut!((*renderer).IMFStreamSink_iface), riid, obj);
}
unsafe extern "system" fn audio_renderer_stream_type_handler_AddRef(iface: *mut IMFMediaTypeHandler) -> ULONG {
    let renderer: *mut audio_renderer = impl_from_IMFMediaTypeHandler(iface);
    return IMFStreamSink_AddRef(core::ptr::addr_of_mut!((*renderer).IMFStreamSink_iface));
}

unsafe extern "system" fn audio_renderer_stream_type_handler_Release(iface: *mut IMFMediaTypeHandler) -> ULONG {
    let renderer: *mut audio_renderer = impl_from_IMFMediaTypeHandler(iface);
    return IMFStreamSink_Release(core::ptr::addr_of_mut!((*renderer).IMFStreamSink_iface));
}

unsafe extern "C" fn check_media_type(type_: *mut IMFMediaType, current: *mut IMFMediaType) -> HRESULT {
    let required_attrs: [*const GUID; 5] = [
        &raw const MF_MT_AUDIO_SAMPLES_PER_SECOND,
        &raw const MF_MT_AUDIO_NUM_CHANNELS,
        &raw const MF_MT_AUDIO_BITS_PER_SAMPLE,
        &raw const MF_MT_AUDIO_BLOCK_ALIGNMENT,
        &raw const MF_MT_AUDIO_AVG_BYTES_PER_SECOND,
    ];
    let mut value: PROPVARIANT = core::mem::zeroed();
    let mut result: BOOL = 0;
    let mut hr: HRESULT = 0;
    let mut major: GUID = core::mem::zeroed();
    let mut i: i32 = 0;

    hr = IMFMediaType_GetGUID(type_, &raw const MF_MT_MAJOR_TYPE, &raw mut major);
    if hr < 0 {
        return hr;
    }
    if IsEqualGUID!(&raw mut major, &raw const MFMediaType_Audio) == false {
        return MF_E_INVALIDMEDIATYPE;
    }

    i = 0;
    while hr >= 0 && (i as usize) < ARRAY_SIZE!(required_attrs) as usize {
        PropVariantInit!(&raw mut value);
        hr = IMFMediaType_GetItem(type_, required_attrs[i as usize], &raw mut value);
        if hr >= 0 {
            hr = IMFMediaType_CompareItem(
                current,
                required_attrs[i as usize],
                &raw mut value,
                &raw mut result,
            );
        }
        if hr >= 0 && result == 0 {
            hr = MF_E_INVALIDMEDIATYPE;
        }
        PropVariantClear(&raw mut value);
        i += 1;
    }

    if hr < 0 {
        return MF_E_INVALIDMEDIATYPE;
    }

    return S_OK;
}

unsafe extern "system" fn audio_renderer_stream_type_handler_IsMediaTypeSupported(iface: *mut IMFMediaTypeHandler, in_type: *mut IMFMediaType, out_type: *mut *mut IMFMediaType) -> HRESULT {
    let mut renderer: *mut audio_renderer = impl_from_IMFMediaTypeHandler(iface);
    let mut hr: HRESULT;

    TRACE!("%p, %p, %p.\n", iface, in_type, out_type);

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    hr = check_media_type(in_type, (*renderer).media_type);
    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "system" fn audio_renderer_stream_type_handler_GetMediaTypeCount(iface: *mut IMFMediaTypeHandler, count: *mut DWORD) -> HRESULT {
    *count = 1;

    return S_OK;
}

unsafe extern "system" fn audio_renderer_stream_type_handler_GetMediaTypeByIndex(iface: *mut IMFMediaTypeHandler, index: DWORD, media_type: *mut *mut IMFMediaType) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFMediaTypeHandler(iface);

    if index > 0 {
        return MF_E_NO_MORE_TYPES;
    }

    *media_type = (*renderer).media_type;
    IMFMediaType_AddRef(*media_type);
    return S_OK;
}

unsafe extern "C" fn audio_renderer_create_audio_client(
    renderer: *mut audio_renderer,
) -> HRESULT {
    let mut result: *mut IMFAsyncResult = core::ptr::null_mut();
    let mut flags: ::core::ffi::c_uint;
    let mut wfx: *mut WAVEFORMATEX = core::ptr::null_mut();
    let mut hr: HRESULT;

    audio_renderer_release_audio_client(renderer);

    hr = IMMDevice_Activate(
        (*renderer).device,
        &raw const IID_IAudioClient,
        CLSCTX_INPROC_SERVER as DWORD,
        core::ptr::null_mut(),
        &raw mut (*renderer).audio_client as *mut *mut ::core::ffi::c_void,
    );
    if hr < 0 as HRESULT {
        WARN!("Failed to create audio client, hr %#lx.\n", hr);
        return hr;
    }

    /* FIXME: for now always use default format. */
    hr = IAudioClient_GetMixFormat((*renderer).audio_client, &raw mut wfx);
    if hr < 0 as HRESULT {
        WARN!("Failed to get audio format, hr %#lx.\n", hr);
        return hr;
    }

    (*renderer).frame_size = (((*wfx).wBitsPerSample as ::core::ffi::c_int
        * (*wfx).nChannels as ::core::ffi::c_int)
        / 8 as ::core::ffi::c_int) as ::core::ffi::c_uint;
    (*renderer).sample_rate = (*wfx).nSamplesPerSec;

    flags = AUDCLNT_STREAMFLAGS_EVENTCALLBACK as ::core::ffi::c_uint;
    if ((*renderer).stream_config.flags & (MF_AUDIO_RENDERER_ATTRIBUTE_FLAGS_CROSSPROCESS as ::core::ffi::c_uint)) != 0 {
        flags |= AUDCLNT_STREAMFLAGS_CROSSPROCESS as ::core::ffi::c_uint;
    }
    if ((*renderer).stream_config.flags & (MF_AUDIO_RENDERER_ATTRIBUTE_FLAGS_NOPERSIST as ::core::ffi::c_uint)) != 0 {
        flags |= AUDCLNT_STREAMFLAGS_NOPERSIST as ::core::ffi::c_uint;
    }
    hr = IAudioClient_Initialize(
        (*renderer).audio_client,
        AUDCLNT_SHAREMODE_SHARED as AUDCLNT_SHAREMODE,
        flags as DWORD,
        1000000 as REFERENCE_TIME,
        0 as REFERENCE_TIME,
        wfx,
        &raw mut (*renderer).stream_config.session_id as LPCGUID,
    );
    CoTaskMemFree(wfx as LPVOID);
    if hr < 0 as HRESULT {
        WARN!("Failed to initialize audio client, hr %#lx.\n", hr);
        return hr;
    }

    hr = IAudioClient_GetService(
        (*renderer).audio_client,
        &raw const IID_IAudioStreamVolume,
        &raw mut (*renderer).stream_volume as *mut *mut ::core::ffi::c_void,
    );
    if hr < 0 as HRESULT {
        WARN!("Failed to get stream volume control, hr %#lx.\n", hr);
        return hr;
    }

    hr = IAudioClient_GetService(
        (*renderer).audio_client,
        &raw const IID_ISimpleAudioVolume,
        &raw mut (*renderer).audio_volume as *mut *mut ::core::ffi::c_void,
    );
    if hr < 0 as HRESULT {
        WARN!("Failed to get audio volume control, hr %#lx.\n", hr);
        return hr;
    }

    hr = IAudioClient_GetService(
        (*renderer).audio_client,
        &raw const IID_IAudioRenderClient,
        &raw mut (*renderer).audio_render_client as *mut *mut ::core::ffi::c_void,
    );
    if hr < 0 as HRESULT {
        WARN!("Failed to get audio render client, hr %#lx.\n", hr);
        return hr;
    }

    hr = IAudioClient_GetService(
        (*renderer).audio_client,
        &raw const IID_IAudioClock,
        &raw mut (*renderer).audio_clock as *mut *mut ::core::ffi::c_void,
    );
    if hr < 0 as HRESULT {
        WARN!("Failed to get audio clock, hr %#lx.\n", hr);
        return hr;
    }

    hr = IAudioClock_GetFrequency(
        (*renderer).audio_clock,
        &raw mut (*renderer).audio_clock_frequency,
    );
    if hr < 0 as HRESULT {
        WARN!("Failed to get audio clock frequency, hr %#lx.\n", hr);
        return hr;
    }

    hr = IAudioClient_SetEventHandle((*renderer).audio_client, (*renderer).buffer_ready_event);
    if hr < 0 as HRESULT {
        WARN!("Failed to set event handle, hr %#lx.\n", hr);
        return hr;
    }

    hr = IAudioClient_GetBufferSize((*renderer).audio_client, &raw mut (*renderer).max_frames);
    if hr < 0 as HRESULT {
        WARN!("Failed to get buffer size, hr %#lx.\n", hr);
        return hr;
    }

    hr = MFCreateAsyncResult(
        core::ptr::null_mut(),
        &raw mut (*renderer).render_callback,
        core::ptr::null_mut(),
        &raw mut result,
    );
    if hr >= 0 as HRESULT {
        hr = MFPutWaitingWorkItem(
            (*renderer).buffer_ready_event,
            0 as LONG,
            result,
            &raw mut (*renderer).buffer_ready_key,
        );
        if hr < 0 as HRESULT {
            WARN!("Failed to submit wait item, hr %#lx.\n", hr);
        }
        IMFAsyncResult_Release(result);
    }

    return hr;
}

unsafe extern "system" fn audio_renderer_stream_type_handler_SetCurrentMediaType(iface: *mut IMFMediaTypeHandler, media_type: *mut IMFMediaType) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFMediaTypeHandler(iface);
    let mut compare_result: BOOL = 0;
    let mut hr: HRESULT = S_OK;

    if media_type.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(&raw mut (*renderer).cs);
    hr = check_media_type(media_type, (*renderer).media_type);
    if hr >= 0 {
        if !(*renderer).current_media_type.is_null() {
            IMFMediaType_Release((*renderer).current_media_type);
        }
        (*renderer).current_media_type = media_type;
        IMFMediaType_AddRef((*renderer).current_media_type);

        hr = audio_renderer_create_audio_client(renderer);
        if hr >= 0 {
            if IMFMediaType_Compare(
                (*renderer).media_type,
                media_type as *mut IMFAttributes,
                MF_ATTRIBUTES_MATCH_OUR_ITEMS,
                &raw mut compare_result,
            ) >= 0
                && compare_result == 0
            {
                IMFMediaEventQueue_QueueEventParamVar(
                    (*renderer).stream_event_queue,
                    MEStreamSinkFormatInvalidated as MediaEventType,
                    &raw const GUID_NULL,
                    S_OK,
                    core::ptr::null::<PROPVARIANT>(),
                );
                audio_renderer_preroll(renderer);
            }
        }
    } else {
        hr = MF_E_INVALIDMEDIATYPE;
    }
    LeaveCriticalSection(&raw mut (*renderer).cs);

    return hr;
}

unsafe extern "system" fn audio_renderer_stream_type_handler_GetCurrentMediaType(iface: *mut IMFMediaTypeHandler, media_type: *mut *mut IMFMediaType) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFMediaTypeHandler(iface);
    let mut hr: HRESULT = S_OK;
    TRACE!("%p, %p.\n", iface, media_type);
    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    if !(*renderer).current_media_type.is_null() {
        *media_type = (*renderer).current_media_type;
        IMFMediaType_AddRef(*media_type);
    } else {
        hr = MF_E_NOT_INITIALIZED;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    return hr;
}
unsafe extern "system" fn audio_renderer_stream_type_handler_GetMajorType(iface: *mut IMFMediaTypeHandler, type_: *mut GUID) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_IMFMediaTypeHandler(iface);

    if type_.is_null() {
        return E_POINTER;
    }

    if (*renderer).flags & SAR_SHUT_DOWN != 0 {
        return MF_E_STREAMSINK_REMOVED;
    }

    memcpy(
        type_ as *mut core::ffi::c_void,
        core::ptr::addr_of!(MFMediaType_Audio) as *const core::ffi::c_void,
        core::mem::size_of::<GUID>() as usize,
    );
    return S_OK;
}

unsafe extern "C" fn audio_renderer_collect_supported_types(
    renderer: *mut audio_renderer,
) -> HRESULT {
    let mut client: *mut IAudioClient = core::ptr::null_mut();
    let mut format: *mut WAVEFORMATEX = core::ptr::null_mut();
    let mut hr: HRESULT;

    hr = MFCreateMediaType(core::ptr::addr_of_mut!((*renderer).media_type));
    if hr < (0) {
        return hr;
    }

    hr = IMMDevice_Activate(
        (*renderer).device,
        core::ptr::addr_of!(IID_IAudioClient),
        (CLSCTX_INPROC_SERVER) as DWORD,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(client) as *mut *mut core::ffi::c_void,
    );
    if hr < (0) {
        WARN!("Failed to create audio client, hr %#lx.\n", hr);
        return hr;
    }

    /* FIXME:  */

    hr = IAudioClient_GetMixFormat(client, core::ptr::addr_of_mut!(format));
    IAudioClient_Release(client);
    if hr < (0) {
        WARN!("Failed to get device audio format, hr %#lx.\n", hr);
        return hr;
    }

    hr = MFInitMediaTypeFromWaveFormatEx(
        (*renderer).media_type,
        format,
        (((*format).cbSize as usize).wrapping_add(core::mem::size_of::<WAVEFORMATEX>()))
            as UINT32,
    );
    CoTaskMemFree(format as LPVOID);
    if hr < (0) {
        WARN!("Failed to initialize media type, hr %#lx.\n", hr);
        return hr;
    }

    return hr;
}

unsafe extern "system" fn audio_renderer_render_callback_QueryInterface(iface: *mut IMFAsyncCallback, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAsyncCallback)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = iface as *mut core::ffi::c_void;
        IMFAsyncCallback_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn audio_renderer_render_callback_AddRef(mut iface: *mut IMFAsyncCallback) -> ULONG {
    let mut renderer: *mut audio_renderer = impl_from_render_callback_IMFAsyncCallback(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn audio_renderer_render_callback_Release(iface: *mut IMFAsyncCallback) -> ULONG {
    let renderer: *mut audio_renderer = impl_from_render_callback_IMFAsyncCallback(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}
unsafe extern "system" fn audio_renderer_render_callback_GetParameters(
    iface: *mut IMFAsyncCallback,
    flags: *mut DWORD,
    queue: *mut DWORD,
) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "C" fn audio_renderer_render_silence(renderer: *mut audio_renderer) {
    let mut max_frames: core::ffi::c_uint = 0;
    let mut pad_frames: core::ffi::c_uint = 0;
    let mut dst: *mut BYTE = core::ptr::null_mut();

    if (IAudioClient_GetBufferSize((*renderer).audio_client, core::ptr::addr_of_mut!(max_frames))) >= 0
    {
        if (IAudioClient_GetCurrentPadding((*renderer).audio_client, core::ptr::addr_of_mut!(pad_frames))) >= 0
        {
            max_frames = max_frames.wrapping_sub(pad_frames);
            if max_frames != 0
                && (IAudioRenderClient_GetBuffer((*renderer).audio_render_client, max_frames as UINT32, core::ptr::addr_of_mut!(dst))) >= 0
            {
                IAudioRenderClient_ReleaseBuffer((*renderer).audio_render_client, max_frames as UINT32, AUDCLNT_BUFFERFLAGS_SILENT);
            }
        }
    }
}

unsafe extern "C" fn audio_renderer_render(
    mut renderer: *mut audio_renderer,
    mut result: *mut IMFAsyncResult,
) {
    let mut src_frames: ::core::ffi::c_uint = 0;
    let mut dst_frames: ::core::ffi::c_uint = 0;
    let mut max_frames: ::core::ffi::c_uint = 0;
    let mut pad_frames: ::core::ffi::c_uint = 0;
    let mut obj: *mut queued_object = ::core::ptr::null_mut::<queued_object>();
    let mut obj2: *mut queued_object = ::core::ptr::null_mut::<queued_object>();
    let mut keep_sample: BOOL = (FALSE as i32);
    let mut buffer: *mut IMFMediaBuffer = ::core::ptr::null_mut::<IMFMediaBuffer>();
    let mut position: UINT64 = 0;
    let mut dst: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
    let mut src: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
    let mut src_len: DWORD = 0;
    let mut pts: LONGLONG = 0;
    let mut hr: HRESULT = 0;
    obj = ((*renderer).queue.next as *mut ::core::ffi::c_char).offset(
        -(&raw mut (*::core::ptr::null_mut::<queued_object>()).entry as ::core::ffi::c_ulonglong
            as isize),
    ) as *mut queued_object;
    obj2 = ((*obj).entry.next as *mut ::core::ffi::c_char).offset(
        -(&raw mut (*::core::ptr::null_mut::<queued_object>()).entry as ::core::ffi::c_ulonglong
            as isize),
    ) as *mut queued_object;
    while &raw mut (*obj).entry != &raw mut (*renderer).queue {
        if (*obj).type_ as ::core::ffi::c_uint
            == OBJECT_TYPE_MARKER as ::core::ffi::c_uint
        {
            if (*obj).u.marker.type_ as ::core::ffi::c_uint
                == MFSTREAMSINK_MARKER_ENDOFSEGMENT as ::core::ffi::c_uint
            {
                (*renderer).flags |= SAR_ADDING_SILENCE as ::core::ffi::c_uint;
            }
            IMFMediaEventQueue_QueueEventParamVar(
                (*renderer).stream_event_queue,
                MEStreamSinkMarker as ::core::ffi::c_int as MediaEventType,
                &raw const GUID_NULL,
                S_OK,
                &raw mut (*obj).u.marker.context,
            );
        } else if (*obj).type_ as ::core::ffi::c_uint
            == OBJECT_TYPE_SAMPLE as ::core::ffi::c_uint
        {
            if IMFSample_ConvertToContiguousBuffer((*obj).u.sample.sample, &raw mut buffer)
                >= 0 as HRESULT
            {
                if IMFMediaBuffer_Lock(
                    buffer,
                    &raw mut src,
                    ::core::ptr::null_mut::<DWORD>(),
                    &raw mut src_len,
                ) >= 0 as HRESULT
                {
                    src_frames = src_len.wrapping_div((*renderer).frame_size as DWORD)
                        as ::core::ffi::c_uint;
                    if src_frames != 0 {
                        if IAudioClient_GetBufferSize((*renderer).audio_client, &raw mut max_frames)
                            >= 0 as HRESULT
                        {
                            if IAudioClient_GetCurrentPadding(
                                (*renderer).audio_client,
                                &raw mut pad_frames,
                            ) >= 0 as HRESULT
                            {
                                max_frames = max_frames.wrapping_sub(pad_frames);
                                IMFSample_GetSampleTime((*obj).u.sample.sample, &raw mut pts);
                                IAudioClock_GetPosition(
                                    (*renderer).audio_clock,
                                    &raw mut position,
                                    ::core::ptr::null_mut::<UINT64>(),
                                );
                                position = position.wrapping_add(
                                    (pad_frames as UINT64)
                                        .wrapping_mul((*renderer).audio_clock_frequency)
                                        .wrapping_div((*renderer).sample_rate as UINT64),
                                );
                                src_frames = src_frames.wrapping_sub((*obj).u.sample.frame_offset);
                                dst_frames = min!(src_frames, max_frames);
                                hr = S_OK as HRESULT;
                                if dst_frames != 0 && {
                                    hr = IAudioRenderClient_GetBuffer(
                                        (*renderer).audio_render_client,
                                        dst_frames as UINT32,
                                        &raw mut dst,
                                    );
                                    hr >= 0 as HRESULT
                                } {
                                    (*renderer).flags &= !(SAR_ADDING_SILENCE as ::core::ffi::c_int)
                                        as ::core::ffi::c_uint;
                                    memcpy(
                                        dst as *mut ::core::ffi::c_void,
                                        src.offset(
                                            (*obj)
                                                .u
                                                .sample
                                                .frame_offset
                                                .wrapping_mul((*renderer).frame_size)
                                                as isize,
                                        )
                                            as *const ::core::ffi::c_void,
                                        dst_frames.wrapping_mul((*renderer).frame_size) as size_t,
                                    );
                                    IAudioRenderClient_ReleaseBuffer(
                                        (*renderer).audio_render_client,
                                        dst_frames as UINT32,
                                        0 as DWORD,
                                    );
                                    if (*obj).u.sample.frame_offset
                                        == 0 as ::core::ffi::c_uint
                                    {
                                        (*renderer).pts = pts;
                                        (*renderer).position = position;
                                    }
                                    (*obj).u.sample.frame_offset =
                                        (*obj).u.sample.frame_offset.wrapping_add(dst_frames);
                                    (*renderer).queued_frames =
                                        (*renderer).queued_frames.wrapping_sub(dst_frames);
                                }
                                keep_sample = (hr < 0 as HRESULT
                                    || src_frames > max_frames)
                                    as ::core::ffi::c_int
                                    as BOOL;
                            }
                        }
                    }
                    IMFMediaBuffer_Unlock(buffer);
                }
                IMFMediaBuffer_Release(buffer);
            }
        }
        if keep_sample != 0 {
            break;
        }
        list_remove(&raw mut (*obj).entry);
        release_pending_object(obj);
        obj = obj2;
        obj2 = ((*obj).entry.next as *mut ::core::ffi::c_char).offset(
            -(&raw mut (*::core::ptr::null_mut::<queued_object>()).entry as ::core::ffi::c_ulonglong
                as isize),
        ) as *mut queued_object;
    }
    if list_empty(&raw mut (*renderer).queue) != 0 {
        if ((*renderer).flags & SAR_SAMPLE_REQUESTED as ::core::ffi::c_uint) == 0 {
            IMFMediaEventQueue_QueueEventParamVar(
                (*renderer).stream_event_queue,
                MEStreamSinkRequestSample as ::core::ffi::c_int as MediaEventType,
                &raw const GUID_NULL,
                S_OK,
                ::core::ptr::null::<PROPVARIANT>(),
            );
            (*renderer).flags |= SAR_SAMPLE_REQUESTED as ::core::ffi::c_uint;
        }
        if ((*renderer).flags & SAR_ADDING_SILENCE as ::core::ffi::c_uint) != 0 {
            audio_renderer_render_silence(renderer);
        }
    }
    hr = MFPutWaitingWorkItem(
        (*renderer).buffer_ready_event,
        0 as LONG,
        result,
        &raw mut (*renderer).buffer_ready_key,
    );
    if hr < 0 as HRESULT {
        WARN!("Failed to submit wait item, hr %#lx.\n", hr);
    }
}

unsafe extern "system" fn audio_renderer_render_callback_Invoke(iface: *mut IMFAsyncCallback, result: *mut IMFAsyncResult) -> HRESULT {
    let renderer: *mut audio_renderer = impl_from_render_callback_IMFAsyncCallback(iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    if ((*renderer).flags & SAR_SHUT_DOWN) == 0 {
        audio_renderer_render(renderer, result);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return S_OK;
}

unsafe extern "C" fn sar_create_object(attributes: *mut IMFAttributes, user_context: *mut core::ffi::c_void, obj: *mut *mut IUnknown) -> HRESULT {
    let mut renderer: *mut audio_renderer = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    renderer = calloc(1, (core::mem::size_of::<audio_renderer>()) as _) as *mut audio_renderer;
    if renderer.is_null() {
        return E_OUTOFMEMORY;
    }

    (*renderer).IMFMediaSink_iface.lpVtbl = core::ptr::addr_of!(audio_renderer_sink_vtbl);
    (*renderer).IMFMediaSinkPreroll_iface.lpVtbl = core::ptr::addr_of!(audio_renderer_preroll_vtbl);
    (*renderer).IMFStreamSink_iface.lpVtbl = core::ptr::addr_of!(audio_renderer_stream_vtbl);
    (*renderer).IMFMediaTypeHandler_iface.lpVtbl = core::ptr::addr_of!(audio_renderer_stream_type_handler_vtbl);
    (*renderer).IMFClockStateSink_iface.lpVtbl = core::ptr::addr_of!(audio_renderer_clock_sink_vtbl);
    (*renderer).IMFMediaEventGenerator_iface.lpVtbl = core::ptr::addr_of!(audio_renderer_events_vtbl);
    (*renderer).IMFGetService_iface.lpVtbl = core::ptr::addr_of!(audio_renderer_get_service_vtbl);
    (*renderer).IMFSimpleAudioVolume_iface.lpVtbl = core::ptr::addr_of!(audio_renderer_simple_volume_vtbl);
    (*renderer).IMFAudioStreamVolume_iface.lpVtbl = core::ptr::addr_of!(audio_renderer_stream_volume_vtbl);
    (*renderer).IMFAudioPolicy_iface.lpVtbl = core::ptr::addr_of!(audio_renderer_policy_vtbl);
    (*renderer).IMFPresentationTimeSource_iface.lpVtbl = core::ptr::addr_of!(audio_renderer_time_source_vtbl);
    (*renderer).render_callback.lpVtbl = core::ptr::addr_of!(audio_renderer_render_callback_vtbl);
    (*renderer).rate = 1.0f32;
    (*renderer).refcount = 1;
    InitializeCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    (*renderer).buffer_ready_event = CreateEventW(
        core::ptr::null_mut(),
        (FALSE) as BOOL,
        (FALSE) as BOOL,
        core::ptr::null(),
    );
    list_init(core::ptr::addr_of_mut!((*renderer).queue));

    'failed: {
        hr = MFCreateEventQueue(core::ptr::addr_of_mut!((*renderer).event_queue));
        if hr < 0 {
            break 'failed;
        }

        hr = MFCreateEventQueue(core::ptr::addr_of_mut!((*renderer).stream_event_queue));
        if hr < 0 {
            break 'failed;
        }

        hr = sar_create_mmdevice(attributes, renderer);
        if hr < 0 {
            break 'failed;
        }

        hr = audio_renderer_collect_supported_types(renderer);
        if hr < 0 {
            break 'failed;
        }

        *obj = core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface) as *mut IUnknown;

        return S_OK;
    }

    IMFMediaSink_Release(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn mf_is_sar_sink(sink: *mut IMFMediaSink) -> BOOL {
    return ((*sink).lpVtbl == core::ptr::addr_of!(audio_renderer_sink_vtbl)) as i32;
}

unsafe extern "C" fn sar_shutdown_object(user_context: *mut core::ffi::c_void, obj: *mut IUnknown) {
    let mut sink: *mut IMFMediaSink = core::ptr::null_mut();
    if IUnknown_QueryInterface(
        obj,
        core::ptr::addr_of!(IID_IMFMediaSink),
        core::ptr::addr_of_mut!(sink) as *mut *mut core::ffi::c_void,
    ) >= 0
    {
        IMFMediaSink_Shutdown(sink);
        IMFMediaSink_Release(sink);
    }
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateAudioRendererActivate(activate: *mut *mut IMFActivate) -> HRESULT {
    if activate.is_null() {
        return E_POINTER;
    }

    return create_activation_object(core::ptr::null_mut(), core::ptr::addr_of!(sar_activate_funcs), activate);
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateAudioRenderer(attributes: *mut IMFAttributes, sink: *mut *mut IMFMediaSink) -> HRESULT {
    let mut object: *mut IUnknown = core::ptr::null_mut();
    let mut hr: HRESULT;

    hr = sar_create_object(attributes, core::ptr::null_mut(), &mut object);
    if hr >= 0 {
        hr = IUnknown_QueryInterface(object, core::ptr::addr_of!(IID_IMFMediaSink), sink as *mut *mut core::ffi::c_void);
        IUnknown_Release(object);
    }

    return hr;
}

// COM vtable tables — generated from sar.c by vtbl_gen.py
/// sar.c:500 `static const IMFMediaSinkVtbl audio_renderer_sink_vtbl`  (slot order: rust:types_amd64.rs)
pub static audio_renderer_sink_vtbl: IMFMediaSinkVtbl = IMFMediaSinkVtbl {
    QueryInterface: Some(audio_renderer_sink_QueryInterface),
    AddRef: Some(audio_renderer_sink_AddRef),
    Release: Some(audio_renderer_sink_Release),
    GetCharacteristics: Some(audio_renderer_sink_GetCharacteristics),
    AddStreamSink: Some(audio_renderer_sink_AddStreamSink),
    RemoveStreamSink: Some(audio_renderer_sink_RemoveStreamSink),
    GetStreamSinkCount: Some(audio_renderer_sink_GetStreamSinkCount),
    GetStreamSinkByIndex: Some(audio_renderer_sink_GetStreamSinkByIndex),
    GetStreamSinkById: Some(audio_renderer_sink_GetStreamSinkById),
    SetPresentationClock: Some(audio_renderer_sink_SetPresentationClock),
    GetPresentationClock: Some(audio_renderer_sink_GetPresentationClock),
    Shutdown: Some(audio_renderer_sink_Shutdown),
};

/// sar.c:559 `static const IMFMediaSinkPrerollVtbl audio_renderer_preroll_vtbl`  (slot order: rust:types_amd64.rs)
pub static audio_renderer_preroll_vtbl: IMFMediaSinkPrerollVtbl = IMFMediaSinkPrerollVtbl {
    QueryInterface: Some(audio_renderer_preroll_QueryInterface),
    AddRef: Some(audio_renderer_preroll_AddRef),
    Release: Some(audio_renderer_preroll_Release),
    NotifyPreroll: Some(audio_renderer_preroll_NotifyPreroll),
};

/// sar.c:624 `static const IMFMediaEventGeneratorVtbl audio_renderer_events_vtbl`  (slot order: rust:types_amd64.rs)
pub static audio_renderer_events_vtbl: IMFMediaEventGeneratorVtbl = IMFMediaEventGeneratorVtbl {
    QueryInterface: Some(audio_renderer_events_QueryInterface),
    AddRef: Some(audio_renderer_events_AddRef),
    Release: Some(audio_renderer_events_Release),
    GetEvent: Some(audio_renderer_events_GetEvent),
    BeginGetEvent: Some(audio_renderer_events_BeginGetEvent),
    EndGetEvent: Some(audio_renderer_events_EndGetEvent),
    QueueEvent: Some(audio_renderer_events_QueueEvent),
};

/// sar.c:814 `static const IMFClockStateSinkVtbl audio_renderer_clock_sink_vtbl`  (slot order: rust:types_amd64.rs)
pub static audio_renderer_clock_sink_vtbl: IMFClockStateSinkVtbl = IMFClockStateSinkVtbl {
    QueryInterface: Some(audio_renderer_clock_sink_QueryInterface),
    AddRef: Some(audio_renderer_clock_sink_AddRef),
    Release: Some(audio_renderer_clock_sink_Release),
    OnClockStart: Some(audio_renderer_clock_sink_OnClockStart),
    OnClockStop: Some(audio_renderer_clock_sink_OnClockStop),
    OnClockPause: Some(audio_renderer_clock_sink_OnClockPause),
    OnClockRestart: Some(audio_renderer_clock_sink_OnClockRestart),
    OnClockSetRate: Some(audio_renderer_clock_sink_OnClockSetRate),
};

/// sar.c:873 `static const IMFGetServiceVtbl audio_renderer_get_service_vtbl`  (slot order: rust:types_amd64.rs)
pub static audio_renderer_get_service_vtbl: IMFGetServiceVtbl = IMFGetServiceVtbl {
    QueryInterface: Some(audio_renderer_get_service_QueryInterface),
    AddRef: Some(audio_renderer_get_service_AddRef),
    Release: Some(audio_renderer_get_service_Release),
    GetService: Some(audio_renderer_get_service_GetService),
};

/// sar.c:980 `static const IMFSimpleAudioVolumeVtbl audio_renderer_simple_volume_vtbl`  (slot order: rust:types_amd64.rs)
pub static audio_renderer_simple_volume_vtbl: IMFSimpleAudioVolumeVtbl = IMFSimpleAudioVolumeVtbl {
    QueryInterface: Some(audio_renderer_simple_volume_QueryInterface),
    AddRef: Some(audio_renderer_simple_volume_AddRef),
    Release: Some(audio_renderer_simple_volume_Release),
    SetMasterVolume: Some(audio_renderer_simple_volume_SetMasterVolume),
    GetMasterVolume: Some(audio_renderer_simple_volume_GetMasterVolume),
    SetMute: Some(audio_renderer_simple_volume_SetMute),
    GetMute: Some(audio_renderer_simple_volume_GetMute),
};

/// sar.c:1112 `static const IMFAudioStreamVolumeVtbl audio_renderer_stream_volume_vtbl`  (slot order: rust:types_amd64.rs)
pub static audio_renderer_stream_volume_vtbl: IMFAudioStreamVolumeVtbl = IMFAudioStreamVolumeVtbl {
    QueryInterface: Some(audio_renderer_stream_volume_QueryInterface),
    AddRef: Some(audio_renderer_stream_volume_AddRef),
    Release: Some(audio_renderer_stream_volume_Release),
    GetChannelCount: Some(audio_renderer_stream_volume_GetChannelCount),
    SetChannelVolume: Some(audio_renderer_stream_volume_SetChannelVolume),
    GetChannelVolume: Some(audio_renderer_stream_volume_GetChannelVolume),
    SetAllVolumes: Some(audio_renderer_stream_volume_SetAllVolumes),
    GetAllVolumes: Some(audio_renderer_stream_volume_GetAllVolumes),
};

/// sar.c:1195 `static const IMFAudioPolicyVtbl audio_renderer_policy_vtbl`  (slot order: rust:types_amd64.rs)
pub static audio_renderer_policy_vtbl: IMFAudioPolicyVtbl = IMFAudioPolicyVtbl {
    QueryInterface: Some(audio_renderer_policy_QueryInterface),
    AddRef: Some(audio_renderer_policy_AddRef),
    Release: Some(audio_renderer_policy_Release),
    SetGroupingParam: Some(audio_renderer_policy_SetGroupingParam),
    GetGroupingParam: Some(audio_renderer_policy_GetGroupingParam),
    SetDisplayName: Some(audio_renderer_policy_SetDisplayName),
    GetDisplayName: Some(audio_renderer_policy_GetDisplayName),
    SetIconPath: Some(audio_renderer_policy_SetIconPath),
    GetIconPath: Some(audio_renderer_policy_GetIconPath),
};

/// sar.c:1326 `static const IMFPresentationTimeSourceVtbl audio_renderer_time_source_vtbl`  (slot order: rust:types_amd64.rs)
pub static audio_renderer_time_source_vtbl: IMFPresentationTimeSourceVtbl = IMFPresentationTimeSourceVtbl {
    QueryInterface: Some(audio_renderer_time_source_QueryInterface),
    AddRef: Some(audio_renderer_time_source_AddRef),
    Release: Some(audio_renderer_time_source_Release),
    GetClockCharacteristics: Some(audio_renderer_time_source_GetClockCharacteristics),
    GetCorrelatedTime: Some(audio_renderer_time_source_GetCorrelatedTime),
    GetContinuityKey: Some(audio_renderer_time_source_GetContinuityKey),
    GetState: Some(audio_renderer_time_source_GetState),
    GetProperties: Some(audio_renderer_time_source_GetProperties),
    GetUnderlyingClock: Some(audio_renderer_time_source_GetUnderlyingClock),
};

/// sar.c:1684 `static const IMFStreamSinkVtbl audio_renderer_stream_vtbl`  (slot order: rust:types_amd64.rs)
pub static audio_renderer_stream_vtbl: IMFStreamSinkVtbl = IMFStreamSinkVtbl {
    QueryInterface: Some(audio_renderer_stream_QueryInterface),
    AddRef: Some(audio_renderer_stream_AddRef),
    Release: Some(audio_renderer_stream_Release),
    GetEvent: Some(audio_renderer_stream_GetEvent),
    BeginGetEvent: Some(audio_renderer_stream_BeginGetEvent),
    EndGetEvent: Some(audio_renderer_stream_EndGetEvent),
    QueueEvent: Some(audio_renderer_stream_QueueEvent),
    GetMediaSink: Some(audio_renderer_stream_GetMediaSink),
    GetIdentifier: Some(audio_renderer_stream_GetIdentifier),
    GetMediaTypeHandler: Some(audio_renderer_stream_GetMediaTypeHandler),
    ProcessSample: Some(audio_renderer_stream_ProcessSample),
    PlaceMarker: Some(audio_renderer_stream_PlaceMarker),
    Flush: Some(audio_renderer_stream_Flush),
};

/// sar.c:1968 `static const IMFMediaTypeHandlerVtbl audio_renderer_stream_type_handler_vtbl`  (slot order: rust:types_amd64.rs)
pub static audio_renderer_stream_type_handler_vtbl: IMFMediaTypeHandlerVtbl = IMFMediaTypeHandlerVtbl {
    QueryInterface: Some(audio_renderer_stream_type_handler_QueryInterface),
    AddRef: Some(audio_renderer_stream_type_handler_AddRef),
    Release: Some(audio_renderer_stream_type_handler_Release),
    IsMediaTypeSupported: Some(audio_renderer_stream_type_handler_IsMediaTypeSupported),
    GetMediaTypeCount: Some(audio_renderer_stream_type_handler_GetMediaTypeCount),
    GetMediaTypeByIndex: Some(audio_renderer_stream_type_handler_GetMediaTypeByIndex),
    SetCurrentMediaType: Some(audio_renderer_stream_type_handler_SetCurrentMediaType),
    GetCurrentMediaType: Some(audio_renderer_stream_type_handler_GetCurrentMediaType),
    GetMajorType: Some(audio_renderer_stream_type_handler_GetMajorType),
};

/// sar.c:2177 `static const IMFAsyncCallbackVtbl audio_renderer_render_callback_vtbl`  (slot order: rust:types_amd64.rs)
pub static audio_renderer_render_callback_vtbl: IMFAsyncCallbackVtbl = IMFAsyncCallbackVtbl {
    QueryInterface: Some(audio_renderer_render_callback_QueryInterface),
    AddRef: Some(audio_renderer_render_callback_AddRef),
    Release: Some(audio_renderer_render_callback_Release),
    GetParameters: Some(audio_renderer_render_callback_GetParameters),
    Invoke: Some(audio_renderer_render_callback_Invoke),
};
