use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut IID_IDirect3DDeviceManager9: GUID = _GUID {
    Data1: 0xa0cade0f as ::core::ffi::c_uint,
    Data2: 0x6d5 as ::core::ffi::c_ushort,
    Data3: 0x4cf4 as ::core::ffi::c_ushort,
    Data4: [
        0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x75 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut evr_activate_funcs: activate_funcs = unsafe {
    activate_funcs {
        create_object: Some(
            evr_create_object
                as unsafe extern "C" fn(
                    *mut IMFAttributes,
                    *mut ::core::ffi::c_void,
                    *mut *mut IUnknown,
                ) -> HRESULT,
        ),
        shutdown_object: Some(
            evr_shutdown_object
                as unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut IUnknown) -> (),
        ),
        free_private: None,
    }
};

pub const EVR_SHUT_DOWN: u32 = 0x1;
pub const EVR_INIT_SERVICES: u32 = 0x2; // Currently in InitServices() call.
pub const EVR_MIXER_INITED_SERVICES: u32 = 0x4;
pub const EVR_PRESENTER_INITED_SERVICES: u32 = 0x8;

pub const EVR_STATE_STOPPED: u32 = 0;
pub const EVR_STATE_RUNNING: u32 = 1;
pub const EVR_STATE_PAUSED: u32 = 2;

pub const EVR_STREAM_PREROLLING: u32 = 0x1;
pub const EVR_STREAM_PREROLLED: u32 = 0x2;
pub const EVR_STREAM_SAMPLE_NEEDED: u32 = 0x4;
pub const EVR_STREAM_STARTED: u32 = 0x8;

#[repr(C)]
pub struct video_stream {
    pub IMFStreamSink_iface: IMFStreamSink,
    pub IMFMediaTypeHandler_iface: IMFMediaTypeHandler,
    pub IMFGetService_iface: IMFGetService,
    pub IMFAttributes_iface: IMFAttributes,
    pub refcount: LONG,
    pub id: u32,
    pub flags: u32,
    pub parent: *mut video_renderer,
    pub event_queue: *mut IMFMediaEventQueue,
    pub allocator: *mut IMFVideoSampleAllocator,
    pub attributes: *mut IMFAttributes,
    pub cs: CRITICAL_SECTION,
}

#[repr(C)]
pub struct video_renderer {
    pub IMFMediaSink_iface: IMFMediaSink,
    pub IMFMediaSinkPreroll_iface: IMFMediaSinkPreroll,
    pub IMFVideoRenderer_iface: IMFVideoRenderer,
    pub IMFClockStateSink_iface: IMFClockStateSink,
    pub IMFMediaEventGenerator_iface: IMFMediaEventGenerator,
    pub IMFGetService_iface: IMFGetService,
    pub IMFTopologyServiceLookup_iface: IMFTopologyServiceLookup,
    pub IMediaEventSink_iface: IMediaEventSink,
    pub IMFAttributes_iface: IMFAttributes,
    pub IMFQualityAdvise_iface: IMFQualityAdvise,
    pub IMFRateSupport_iface: IMFRateSupport,
    pub refcount: LONG,

    pub event_queue: *mut IMFMediaEventQueue,
    pub attributes: *mut IMFAttributes,
    pub clock: *mut IMFPresentationClock,

    pub mixer: *mut IMFTransform,
    pub presenter: *mut IMFVideoPresenter,
    pub window: HWND,
    pub device_manager: *mut IUnknown,
    pub flags: u32,
    pub state: u32,

    pub streams: *mut *mut video_stream,
    pub stream_size: usize,
    pub stream_count: usize,

    pub cs: CRITICAL_SECTION,
}

unsafe extern "C" fn impl_from_IMFMediaSink(iface: *mut IMFMediaSink) -> *mut video_renderer {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(video_renderer, IMFMediaSink_iface)) as *mut video_renderer;
}

unsafe extern "C" fn impl_from_IMFMediaSinkPreroll(
    mut iface: *mut IMFMediaSinkPreroll,
) -> *mut video_renderer {
    return (iface as *mut u8).wrapping_sub(
        core::mem::offset_of!(video_renderer, IMFMediaSinkPreroll_iface),
    ) as *mut video_renderer;
}

unsafe extern "C" fn impl_from_IMFVideoRenderer(iface: *mut IMFVideoRenderer) -> *mut video_renderer {
    return (iface as usize).wrapping_sub(
        core::mem::offset_of!(video_renderer, IMFVideoRenderer_iface),
    ) as *mut video_renderer;
}

unsafe extern "C" fn impl_from_IMFMediaEventGenerator(
    iface: *mut IMFMediaEventGenerator,
) -> *mut video_renderer {
    return (iface as *mut u8).wrapping_sub(
        core::mem::offset_of!(video_renderer, IMFMediaEventGenerator_iface),
    ) as *mut video_renderer;
}

unsafe extern "C" fn impl_from_IMFClockStateSink(iface: *mut IMFClockStateSink) -> *mut video_renderer {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(video_renderer, IMFClockStateSink_iface)) as *mut video_renderer;
}

unsafe extern "C" fn impl_from_IMFGetService(iface: *mut IMFGetService) -> *mut video_renderer {
    return (iface as *mut u8).sub(core::mem::offset_of!(video_renderer, IMFGetService_iface)) as *mut video_renderer;
}

unsafe extern "C" fn impl_from_IMFTopologyServiceLookup(
    mut iface: *mut IMFTopologyServiceLookup,
) -> *mut video_renderer {
    return (iface as *mut u8).wrapping_sub(
        core::mem::offset_of!(video_renderer, IMFTopologyServiceLookup_iface),
    ) as *mut video_renderer;
}

unsafe extern "C" fn impl_from_IMediaEventSink(iface: *mut IMediaEventSink) -> *mut video_renderer {
    return CONTAINING_RECORD!(iface, video_renderer, IMediaEventSink_iface);
}

unsafe extern "C" fn impl_from_IMFAttributes(iface: *mut IMFAttributes) -> *mut video_renderer {
    return CONTAINING_RECORD!(iface, video_renderer, IMFAttributes_iface);
}

unsafe extern "C" fn impl_from_IMFQualityAdvise(iface: *mut IMFQualityAdvise) -> *mut video_renderer {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(video_renderer, IMFQualityAdvise_iface)) as *mut video_renderer;
}

unsafe extern "C" fn impl_from_IMFRateSupport(iface: *mut IMFRateSupport) -> *mut video_renderer {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(video_renderer, IMFRateSupport_iface))
        as *mut video_renderer;
}

unsafe extern "C" fn impl_from_IMFStreamSink(iface: *mut IMFStreamSink) -> *mut video_stream {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(video_stream, IMFStreamSink_iface))
        as *mut video_stream;
}

unsafe extern "C" fn impl_from_IMFMediaTypeHandler(iface: *mut IMFMediaTypeHandler) -> *mut video_stream {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(video_stream, IMFMediaTypeHandler_iface)) as *mut video_stream;
}

unsafe extern "C" fn impl_from_stream_IMFGetService(iface: *mut IMFGetService) -> *mut video_stream {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(video_stream, IMFGetService_iface)) as *mut video_stream;
}

unsafe extern "C" fn impl_from_stream_IMFAttributes(iface: *mut IMFAttributes) -> *mut video_stream {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(video_stream, IMFAttributes_iface)) as *mut video_stream;
}

unsafe extern "C" fn video_renderer_init_presenter_services(
    renderer: *mut video_renderer,
) -> HRESULT {
    let mut lookup_client: *mut IMFTopologyServiceLookupClient =
        ::core::ptr::null_mut::<IMFTopologyServiceLookupClient>();
    let mut hr: HRESULT = 0;
    hr = IMFVideoPresenter_QueryInterface(
        (*renderer).presenter,
        core::ptr::addr_of!(IID_IMFTopologyServiceLookupClient),
        core::ptr::addr_of_mut!(lookup_client) as *mut *mut ::core::ffi::c_void,
    );
    if hr >= 0 {
        (*renderer).flags |= EVR_INIT_SERVICES as ::core::ffi::c_uint;
        hr = IMFTopologyServiceLookupClient_InitServicePointers(
            lookup_client,
            core::ptr::addr_of_mut!((*renderer).IMFTopologyServiceLookup_iface),
        );
        if hr >= 0 {
            (*renderer).flags |= EVR_PRESENTER_INITED_SERVICES as ::core::ffi::c_uint;
        }
        (*renderer).flags &= !(EVR_INIT_SERVICES as ::core::ffi::c_uint);
        IMFTopologyServiceLookupClient_Release(lookup_client);
    }
    return hr;
}

unsafe extern "C" fn video_renderer_release_presenter_services(mut renderer: *mut video_renderer) {
    let mut lookup_client: *mut IMFTopologyServiceLookupClient =
        ::core::ptr::null_mut::<IMFTopologyServiceLookupClient>();
    if (*renderer).flags
        & EVR_PRESENTER_INITED_SERVICES as ::core::ffi::c_int as ::core::ffi::c_uint
        != 0
        && IMFVideoPresenter_QueryInterface(
            (*renderer).presenter,
            &raw const IID_IMFTopologyServiceLookupClient,
            &raw mut lookup_client as *mut *mut ::core::ffi::c_void,
        ) >= 0 as HRESULT
    {
        IMFTopologyServiceLookupClient_ReleaseServicePointers(lookup_client);
        IMFTopologyServiceLookupClient_Release(lookup_client);
        (*renderer).flags &=
            !(EVR_PRESENTER_INITED_SERVICES as ::core::ffi::c_int) as ::core::ffi::c_uint;
    }
}

unsafe extern "C" fn video_renderer_release_services(mut renderer: *mut video_renderer) {
    let mut lookup_client: *mut IMFTopologyServiceLookupClient =
        ::core::ptr::null_mut::<IMFTopologyServiceLookupClient>();
    if (*renderer).flags & EVR_MIXER_INITED_SERVICES as ::core::ffi::c_uint != 0
        && IMFTransform_QueryInterface(
            (*renderer).mixer,
            &raw const IID_IMFTopologyServiceLookupClient,
            &raw mut lookup_client as *mut *mut ::core::ffi::c_void,
        ) >= 0 as HRESULT
    {
        IMFTopologyServiceLookupClient_ReleaseServicePointers(lookup_client);
        IMFTopologyServiceLookupClient_Release(lookup_client);
        (*renderer).flags &= !(EVR_MIXER_INITED_SERVICES as ::core::ffi::c_int) as ::core::ffi::c_uint;
    }
    video_renderer_release_presenter_services(renderer);
}
unsafe extern "system" fn video_stream_sink_QueryInterface(iface: *mut IMFStreamSink, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let stream: *mut video_stream = impl_from_IMFStreamSink(iface);

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFStreamSink)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFMediaEventGenerator)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = iface as *mut core::ffi::c_void;
    }
    else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFMediaTypeHandler)) != 0
    {
        *obj = core::ptr::addr_of_mut!((*stream).IMFMediaTypeHandler_iface) as *mut core::ffi::c_void;
    }
    else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFGetService)) != 0
    {
        *obj = core::ptr::addr_of_mut!((*stream).IMFGetService_iface) as *mut core::ffi::c_void;
    }
    else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAttributes)) != 0
    {
        *obj = core::ptr::addr_of_mut!((*stream).IMFAttributes_iface) as *mut core::ffi::c_void;
    }
    else
    {
        WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
        *obj = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*obj as *mut IUnknown);

    return S_OK;
}
unsafe extern "system" fn video_stream_sink_AddRef(iface: *mut IMFStreamSink) -> ULONG {
    let mut stream: *mut video_stream = impl_from_IMFStreamSink(iface);
    let mut refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*stream).refcount) as *mut core::ffi::c_long,
    ) as ULONG;
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8
                as *const core::ffi::c_char,
            b"video_stream_sink_AddRef\0" as *const u8 as *const core::ffi::c_char,
            272 as core::ffi::c_int,
            b"%p, refcount %lu.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            refcount,
        );
    }
    return refcount;
}

unsafe extern "system" fn video_stream_sink_Release(iface: *mut IMFStreamSink) -> ULONG {
    let stream: *mut video_stream = impl_from_IMFStreamSink(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*stream).refcount) as *mut ::core::ffi::c_long,
    ) as ULONG;

    if refcount == 0 {
        if !(*stream).event_queue.is_null() {
            IMFMediaEventQueue_Release((*stream).event_queue);
        }
        if !(*stream).attributes.is_null() {
            IMFAttributes_Release((*stream).attributes);
        }
        if !(*stream).allocator.is_null() {
            IMFVideoSampleAllocator_Release((*stream).allocator);
        }
        DeleteCriticalSection(core::ptr::addr_of_mut!((*stream).cs));
        free(stream as *mut ::core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn video_stream_sink_GetEvent(iface: *mut IMFStreamSink, flags: DWORD, event: *mut *mut IMFMediaEvent) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_IMFStreamSink(iface);

    return IMFMediaEventQueue_GetEvent((*stream).event_queue, flags, event);
}

unsafe extern "system" fn video_stream_sink_BeginGetEvent(iface: *mut IMFStreamSink, callback: *mut IMFAsyncCallback, state: *mut IUnknown) -> HRESULT {
    let stream: *mut video_stream = impl_from_IMFStreamSink(iface);

    return IMFMediaEventQueue_BeginGetEvent((*stream).event_queue, callback, state);
}

unsafe extern "system" fn video_stream_sink_EndGetEvent(
    iface: *mut IMFStreamSink,
    result: *mut IMFAsyncResult,
    event: *mut *mut IMFMediaEvent,
) -> HRESULT {
    let stream: *mut video_stream = impl_from_IMFStreamSink(iface);
    if ((*__wine_dbch___default).flags as core::ffi::c_int
        & ((1 as core::ffi::c_int) << (__WINE_DBCL_TRACE as core::ffi::c_int)))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0".as_ptr()
                as *const core::ffi::c_char,
            b"video_stream_sink_EndGetEvent\0".as_ptr() as *const core::ffi::c_char,
            320 as core::ffi::c_int,
            b"%p, %p, %p.\n\0".as_ptr() as *const core::ffi::c_char,
            iface,
            result,
            event,
        );
    }
    return IMFMediaEventQueue_EndGetEvent((*stream).event_queue, result, event);
}

unsafe extern "system" fn video_stream_sink_QueueEvent(iface: *mut IMFStreamSink, event_type: MediaEventType, ext_type: REFGUID, hr: HRESULT, value: *const PROPVARIANT) -> HRESULT {
    let stream: *mut video_stream = impl_from_IMFStreamSink(iface);

    return IMFMediaEventQueue_QueueEventParamVar((*stream).event_queue, event_type, ext_type, hr, value);
}

unsafe extern "system" fn video_stream_sink_GetMediaSink(iface: *mut IMFStreamSink, sink: *mut *mut IMFMediaSink) -> HRESULT {
    let stream: *mut video_stream = impl_from_IMFStreamSink(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!("%p, %p.\n", iface, sink);

    EnterCriticalSection(core::ptr::addr_of_mut!((*stream).cs));
    if (*stream).parent.is_null() {
        hr = MF_E_STREAMSINK_REMOVED;
    } else if sink.is_null() {
        hr = E_POINTER;
    } else {
        *sink = core::ptr::addr_of_mut!((*(*stream).parent).IMFMediaSink_iface);
        IMFMediaSink_AddRef(*sink);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    return hr;
}

unsafe extern "system" fn video_stream_sink_GetIdentifier(iface: *mut IMFStreamSink, id: *mut DWORD) -> HRESULT {
    let stream: *mut video_stream = impl_from_IMFStreamSink(iface);
    TRACE!("%p, %p.\n", iface, id);

    if (*stream).parent.is_null() {
        return MF_E_STREAMSINK_REMOVED;
    }

    if id.is_null() {
        return E_INVALIDARG;
    }

    *id = (*stream).id as DWORD;

    return S_OK;
}
unsafe extern "system" fn video_stream_sink_GetMediaTypeHandler(iface: *mut IMFStreamSink, handler: *mut *mut IMFMediaTypeHandler) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_IMFStreamSink(iface);

    TRACE!("%p, %p.\n", iface, handler);

    if handler.is_null() {
        return E_POINTER;
    }

    if (*stream).parent.is_null() {
        return MF_E_STREAMSINK_REMOVED;
    }

    *handler = core::ptr::addr_of_mut!((*stream).IMFMediaTypeHandler_iface);
    IMFMediaTypeHandler_AddRef(*handler);

    return S_OK;
}

unsafe extern "system" fn video_stream_sink_ProcessSample(iface: *mut IMFStreamSink, sample: *mut IMFSample) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_IMFStreamSink(iface);
    let mut timestamp: LONGLONG = 0;
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    if (*stream).parent.is_null() {
        hr = MF_E_STREAMSINK_REMOVED;
    } else if (*(*stream).parent).clock.is_null() {
        hr = MF_E_NO_CLOCK;
    } else if {
        hr = IMFSample_GetSampleTime(sample, &mut timestamp);
        hr < 0
    } {
        WARN!("No sample timestamp, hr %#lx.\n", hr);
    } else if (*(*stream).parent).state == EVR_STATE_RUNNING || (*stream).flags & EVR_STREAM_PREROLLING != 0 {
        if !((*stream).flags & EVR_STREAM_STARTED != 0) {
            IMFTransform_ProcessMessage((*(*stream).parent).mixer, MFT_MESSAGE_NOTIFY_START_OF_STREAM, (*stream).id as ULONG_PTR);
            (*stream).flags |= EVR_STREAM_STARTED;
        }

        if IMFTransform_ProcessInput((*(*stream).parent).mixer, (*stream).id as DWORD, sample, 0) >= 0 {
            loop {
                hr = IMFVideoPresenter_ProcessMessage(
                    (*(*stream).parent).presenter,
                    MFVP_MESSAGE_PROCESSINPUTNOTIFY,
                    0 as ULONG_PTR,
                );
                if !(hr == MF_E_TRANSFORM_TYPE_NOT_SET) {
                    break;
                }
                hr = IMFVideoPresenter_ProcessMessage(
                    (*(*stream).parent).presenter,
                    MFVP_MESSAGE_INVALIDATEMEDIATYPE,
                    0 as ULONG_PTR,
                );
                if hr < 0 {
                    break;
                }
            }
        }

        if (*stream).flags & EVR_STREAM_PREROLLING != 0 {
            IMFMediaEventQueue_QueueEventParamVar(
                (*stream).event_queue,
                MEStreamSinkPrerolled as MediaEventType,
                core::ptr::addr_of!(GUID_NULL),
                S_OK,
                core::ptr::null::<PROPVARIANT>(),
            );
            (*stream).flags &= !(EVR_STREAM_PREROLLING);
            (*stream).flags |= EVR_STREAM_PREROLLED;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    return hr;
}

unsafe extern "C" fn video_stream_end_of_stream(mut stream: *mut video_stream) {
    if ((*stream).flags & EVR_STREAM_STARTED) == 0 {
        return;
    }
    IMFTransform_ProcessMessage(
        (*(*stream).parent).mixer,
        MFT_MESSAGE_NOTIFY_END_OF_STREAM,
        (*stream).id as ULONG_PTR,
    );
    (*stream).flags &= !EVR_STREAM_STARTED;
}

unsafe extern "system" fn video_stream_sink_PlaceMarker(iface: *mut IMFStreamSink, marker_type: MFSTREAMSINK_MARKER_TYPE, marker_value: *const PROPVARIANT, context: *const PROPVARIANT) -> HRESULT {
    let stream: *mut video_stream = impl_from_IMFStreamSink(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*stream).cs));
    if (*stream).parent.is_null() {
        hr = MF_E_STREAMSINK_REMOVED;
    } else {
        if marker_type == MFSTREAMSINK_MARKER_ENDOFSEGMENT {
            video_stream_end_of_stream(stream);
        }
        IMFMediaEventQueue_QueueEventParamVar((*stream).event_queue, MEStreamSinkMarker as MediaEventType, &GUID_NULL, S_OK, context);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    return hr;
}

unsafe extern "system" fn video_stream_sink_Flush(iface: *mut IMFStreamSink) -> HRESULT {
    let stream: *mut video_stream = impl_from_IMFStreamSink(iface);
    let mut hr: HRESULT;

    EnterCriticalSection(core::ptr::addr_of_mut!((*stream).cs));
    if (*stream).parent.is_null() {
        hr = MF_E_STREAMSINK_REMOVED;
    } else {
        hr = IMFTransform_ProcessMessage((*(*stream).parent).mixer, MFT_MESSAGE_COMMAND_FLUSH, 0);
        if hr >= 0 {
            hr = IMFVideoPresenter_ProcessMessage((*(*stream).parent).presenter, MFVP_MESSAGE_FLUSH, 0);
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

    return hr;
}

unsafe extern "system" fn video_stream_typehandler_QueryInterface(iface: *mut IMFMediaTypeHandler, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_IMFMediaTypeHandler(iface);
    return IMFStreamSink_QueryInterface(core::ptr::addr_of_mut!((*stream).IMFStreamSink_iface), riid, obj);
}

unsafe extern "system" fn video_stream_typehandler_AddRef(iface: *mut IMFMediaTypeHandler) -> ULONG {
    let stream: *mut video_stream = impl_from_IMFMediaTypeHandler(iface);
    return IMFStreamSink_AddRef(core::ptr::addr_of_mut!((*stream).IMFStreamSink_iface));
}

unsafe extern "system" fn video_stream_typehandler_Release(iface: *mut IMFMediaTypeHandler) -> ULONG {
    let stream: *mut video_stream = impl_from_IMFMediaTypeHandler(iface);
    return IMFStreamSink_Release(core::ptr::addr_of_mut!((*stream).IMFStreamSink_iface));
}

unsafe extern "system" fn video_stream_typehandler_IsMediaTypeSupported(iface: *mut IMFMediaTypeHandler, in_type: *mut IMFMediaType, out_type: *mut *mut IMFMediaType) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_IMFMediaTypeHandler(iface);
    let mut hr: HRESULT;

    TRACE!("%p, %p, %p.\n", iface, in_type, out_type);

    if in_type.is_null() {
        return E_POINTER;
    }

    if (*stream).parent.is_null() {
        return MF_E_INVALIDMEDIATYPE;
    }

    hr = IMFTransform_SetInputType(
        (*(*stream).parent).mixer,
        (*stream).id as DWORD,
        in_type,
        MFT_SET_TYPE_TEST_ONLY as DWORD,
    );
    if hr >= 0 {
        if !out_type.is_null() {
            *out_type = core::ptr::null_mut();
        }
    }

    return hr;
}

unsafe extern "system" fn video_stream_typehandler_GetMediaTypeCount(iface: *mut IMFMediaTypeHandler, count: *mut DWORD) -> HRESULT {
    TRACE!("%p, %p.\n", iface, count);

    if count.is_null() {
        return E_POINTER;
    }

    *count = 0;

    return S_OK;
}
unsafe extern "system" fn video_stream_typehandler_GetMediaTypeByIndex(
    iface: *mut IMFMediaTypeHandler,
    index: DWORD,
    r#type: *mut *mut IMFMediaType,
) -> HRESULT {
    // TRACE("%p, %lu, %p.\n", iface, index, type);
    MF_E_NO_MORE_TYPES
}

unsafe extern "system" fn video_stream_typehandler_SetCurrentMediaType(
    iface: *mut IMFMediaTypeHandler,
    type_0: *mut IMFMediaType,
) -> HRESULT {
    let stream: *mut video_stream = impl_from_IMFMediaTypeHandler(iface);
    let mut hr: HRESULT;

    if type_0.is_null() {
        return E_POINTER;
    }

    if (*stream).parent.is_null() {
        return MF_E_STREAMSINK_REMOVED;
    }

    hr = IMFTransform_SetInputType(
        (*(*stream).parent).mixer,
        (*stream).id as DWORD,
        type_0,
        0 as DWORD,
    );
    if (hr >= 0) && (*stream).id == 0 {
        hr = IMFVideoPresenter_ProcessMessage(
            (*(*stream).parent).presenter,
            MFVP_MESSAGE_INVALIDATEMEDIATYPE,
            0 as ULONG_PTR,
        );
    }

    return hr;
}
unsafe extern "system" fn video_stream_typehandler_GetCurrentMediaType(iface: *mut IMFMediaTypeHandler, type_: *mut *mut IMFMediaType) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_IMFMediaTypeHandler(iface);

    if type_.is_null() {
        return E_POINTER;
    }

    if (*stream).parent.is_null() {
        return MF_E_STREAMSINK_REMOVED;
    }

    return IMFTransform_GetInputCurrentType((*(*stream).parent).mixer, (*stream).id, type_);
}

unsafe extern "system" fn video_stream_typehandler_GetMajorType(iface: *mut IMFMediaTypeHandler, type_0: *mut GUID) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_IMFMediaTypeHandler(iface);

    if (*stream).parent.is_null() {
        return MF_E_STREAMSINK_REMOVED;
    }

    if type_0.is_null() {
        return E_POINTER;
    }

    memcpy(
        type_0 as *mut core::ffi::c_void,
        core::ptr::addr_of!(MFMediaType_Video) as *const core::ffi::c_void,
        core::mem::size_of::<GUID>(),
    );
    return S_OK;
}

unsafe extern "system" fn video_stream_get_service_QueryInterface(iface: *mut IMFGetService, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_stream_IMFGetService(iface);
    return IMFStreamSink_QueryInterface(core::ptr::addr_of_mut!((*stream).IMFStreamSink_iface), riid, obj);
}

unsafe extern "system" fn video_stream_get_service_AddRef(iface: *mut IMFGetService) -> ULONG {
    let stream: *mut video_stream = impl_from_stream_IMFGetService(iface);
    return IMFStreamSink_AddRef(core::ptr::addr_of_mut!((*stream).IMFStreamSink_iface));
}

unsafe extern "system" fn video_stream_get_service_Release(iface: *mut IMFGetService) -> ULONG {
    let stream: *mut video_stream = impl_from_stream_IMFGetService(iface);
    return IMFStreamSink_Release(core::ptr::addr_of_mut!((*stream).IMFStreamSink_iface));
}

unsafe extern "C" fn video_stream_get_service(
    stream: *mut video_stream,
    service: REFGUID,
    riid: REFIID,
    obj: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    if IsEqualGUID!(service, core::ptr::addr_of!(MR_VIDEO_ACCELERATION_SERVICE)) != false {
        if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFVideoSampleAllocator)) != 0 {
            if (*stream).allocator.is_null() {
                hr = MFCreateVideoSampleAllocator(
                    core::ptr::addr_of!(IID_IMFVideoSampleAllocator),
                    core::ptr::addr_of_mut!((*stream).allocator) as *mut *mut ::core::ffi::c_void,
                );
                if hr >= 0 {
                    hr = IMFVideoSampleAllocator_SetDirectXManager(
                        (*stream).allocator,
                        (*(*stream).parent).device_manager,
                    );
                }
            }
            if hr >= 0 {
                hr = IMFVideoSampleAllocator_QueryInterface((*stream).allocator, riid, obj);
            }

            return hr;
        } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IDirect3DDeviceManager9)) != 0
            && !(*(*stream).parent).device_manager.is_null()
        {
            return IUnknown_QueryInterface((*(*stream).parent).device_manager, riid, obj);
        }

        return E_NOINTERFACE;
    }

    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_FIXME as ::core::ffi::c_int))) != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0".as_ptr()
                as *const ::core::ffi::c_char,
            b"video_stream_get_service\0".as_ptr() as *const ::core::ffi::c_char,
            676 as ::core::ffi::c_int,
            b"Unsupported service %s.\n\0".as_ptr() as *const ::core::ffi::c_char,
            debugstr_guid(service as *const _GUID),
        );
    }

    return E_NOTIMPL;
}
unsafe extern "system" fn video_stream_get_service_GetService(
    mut iface: *mut IMFGetService,
    service: *const GUID,
    riid: *const IID,
    mut obj: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_stream_IMFGetService(iface);
    let mut hr: HRESULT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8
                as *const ::core::ffi::c_char,
            b"video_stream_get_service_GetService\0" as *const u8 as *const ::core::ffi::c_char,
            686 as ::core::ffi::c_int,
            b"%p, %s, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(service as *const _GUID),
            debugstr_guid(riid as *const _GUID),
            obj,
        );
    }
    EnterCriticalSection(&raw mut (*stream).cs);
    if (*stream).parent.is_null() {
        hr = MF_E_STREAMSINK_REMOVED;
    } else {
        hr = video_stream_get_service(stream, service, riid, obj);
    }
    LeaveCriticalSection(&raw mut (*stream).cs);
    return hr;
}

unsafe extern "system" fn video_stream_attributes_QueryInterface(iface: *mut IMFAttributes, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);
    return IMFStreamSink_QueryInterface(core::ptr::addr_of_mut!((*stream).IMFStreamSink_iface), riid, obj);
}

unsafe extern "system" fn video_stream_attributes_AddRef(iface: *mut IMFAttributes) -> ULONG {
    let stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);
    return IMFStreamSink_AddRef(core::ptr::addr_of_mut!((*stream).IMFStreamSink_iface));
}

unsafe extern "system" fn video_stream_attributes_Release(iface: *mut IMFAttributes) -> ULONG {
    let stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);
    return IMFStreamSink_Release(core::ptr::addr_of_mut!((*stream).IMFStreamSink_iface));
}

unsafe extern "system" fn video_stream_attributes_GetItem(
    iface: *mut IMFAttributes,
    key: *const GUID,
    value: *mut PROPVARIANT,
) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8
                as *const ::core::ffi::c_char,
            b"video_stream_attributes_GetItem\0" as *const u8 as *const ::core::ffi::c_char,
            728 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            value,
        );
    }
    return IMFAttributes_GetItem((*stream).attributes, key, value);
}

unsafe extern "system" fn video_stream_attributes_GetItemType(iface: *mut IMFAttributes, key: *const GUID, type_0: *mut MF_ATTRIBUTE_TYPE) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);

    return IMFAttributes_GetItemType((*stream).attributes, key, type_0);
}

unsafe extern "system" fn video_stream_attributes_CompareItem(
    iface: *mut IMFAttributes,
    key: *const GUID,
    value: *const PROPVARIANT,
    result: *mut BOOL,
) -> HRESULT {
    let stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8
                as *const core::ffi::c_char,
            b"video_stream_attributes_CompareItem\0" as *const u8 as *const core::ffi::c_char,
            747i32,
            b"%p, %s, %p, %p.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            value,
            result,
        );
    }
    return IMFAttributes_CompareItem((*stream).attributes, key, value, result);
}

unsafe extern "system" fn video_stream_attributes_Compare(
    mut iface: *mut IMFAttributes,
    mut theirs: *mut IMFAttributes,
    mut type_0: MF_ATTRIBUTES_MATCH_TYPE,
    mut result: *mut BOOL,
) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0".as_ptr() as *const i8,
            b"video_stream_attributes_Compare\0".as_ptr() as *const i8,
            757i32,
            b"%p, %p, %d, %p.\n\0".as_ptr() as *const i8,
            iface,
            theirs,
            type_0 as u32,
            result,
        );
    }
    return IMFAttributes_Compare((*stream).attributes, theirs, type_0, result);
}

unsafe extern "system" fn video_stream_attributes_GetUINT32(
    mut iface: *mut IMFAttributes,
    key: *const GUID,
    mut value: *mut UINT32,
) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8
                as *const ::core::ffi::c_char,
            b"video_stream_attributes_GetUINT32\0" as *const u8 as *const ::core::ffi::c_char,
            766 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key),
            value,
        );
    }
    return IMFAttributes_GetUINT32((*stream).attributes, key, value);
}

unsafe extern "system" fn video_stream_attributes_GetUINT64(iface: *mut IMFAttributes, key: REFGUID, value: *mut UINT64) -> HRESULT {
    let stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);
    return IMFAttributes_GetUINT64((*stream).attributes, key, value);
}

unsafe extern "system" fn video_stream_attributes_GetDouble(iface: *mut IMFAttributes, key: *const GUID, value: *mut f64) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);
    TRACE!("%p, %s, %p.\n", iface, debugstr_guid(key), value);
    return IMFAttributes_GetDouble((*stream).attributes, key, value);
}
unsafe extern "system" fn video_stream_attributes_GetGUID(
    iface: *mut IMFAttributes,
    key: *const GUID,
    value: *mut GUID,
) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8
                as *const ::core::ffi::c_char,
            b"video_stream_attributes_GetGUID\0" as *const u8 as *const ::core::ffi::c_char,
            793 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            value,
        );
    }
    return IMFAttributes_GetGUID((*stream).attributes, key, value);
}

unsafe extern "system" fn video_stream_attributes_GetStringLength(iface: *mut IMFAttributes, key: *const GUID, length: *mut UINT32) -> HRESULT {
    let stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);
    TRACE!("%p, %s, %p.\n", iface, debugstr_guid(key), length);
    return IMFAttributes_GetStringLength((*stream).attributes, key, length);
}

unsafe extern "system" fn video_stream_attributes_GetString(
    mut iface: *mut IMFAttributes,
    key: *const GUID,
    mut value: *mut WCHAR,
    mut size: UINT32,
    mut length: *mut UINT32,
) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8
                as *const ::core::ffi::c_char,
            b"video_stream_attributes_GetString\0" as *const u8 as *const ::core::ffi::c_char,
            813 as ::core::ffi::c_int,
            b"%p, %s, %p, %u, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            value,
            size,
            length,
        );
    }
    return IMFAttributes_GetString((*stream).attributes, key, value as LPWSTR, size, length);
}

unsafe extern "system" fn video_stream_attributes_GetAllocatedString(iface: *mut IMFAttributes, key: *const GUID, value: *mut *mut WCHAR, length: *mut UINT32) -> HRESULT {
    let stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);

    return IMFAttributes_GetAllocatedString((*stream).attributes, key, value, length);
}

unsafe extern "system" fn video_stream_attributes_GetBlobSize(iface: *mut IMFAttributes, key: *const GUID, size: *mut UINT32) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8 as *const i8,
            b"video_stream_attributes_GetBlobSize\0" as *const u8 as *const i8,
            832i32,
            b"%p, %s, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_guid(key as *const GUID),
            size,
        );
    }
    return IMFAttributes_GetBlobSize((*stream).attributes, key, size);
}

unsafe extern "system" fn video_stream_attributes_GetBlob(
    mut iface: *mut IMFAttributes,
    key: *const GUID,
    mut buf: *mut UINT8,
    mut bufsize: UINT32,
    mut blobsize: *mut UINT32,
) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8
                as *const i8,
            b"video_stream_attributes_GetBlob\0" as *const u8 as *const i8,
            842 as i32,
            b"%p, %s, %p, %u, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_guid(key as *const _GUID),
            buf,
            bufsize,
            blobsize,
        );
    }
    return IMFAttributes_GetBlob((*stream).attributes, key, buf, bufsize, blobsize);
}

unsafe extern "system" fn video_stream_attributes_GetAllocatedBlob(
    mut iface: *mut IMFAttributes,
    key: *const GUID,
    mut buf: *mut *mut UINT8,
    mut size: *mut UINT32,
) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0".as_ptr()
                as *const core::ffi::c_char,
            b"video_stream_attributes_GetAllocatedBlob\0".as_ptr() as *const core::ffi::c_char,
            852 as core::ffi::c_int,
            b"%p, %s, %p, %p.\n\0".as_ptr() as *const core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            buf,
            size,
        );
    }
    return IMFAttributes_GetAllocatedBlob((*stream).attributes, key, buf, size);
}

unsafe extern "system" fn video_stream_attributes_GetUnknown(iface: *mut IMFAttributes, key: REFGUID, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);

    return IMFAttributes_GetUnknown((*stream).attributes, key, riid, out);
}
unsafe extern "system" fn video_stream_attributes_SetItem(iface: *mut IMFAttributes, key: REFGUID, value: REFPROPVARIANT) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);
    TRACE!("%p, %s, %p.\n", iface, debugstr_guid(key as *const GUID), value);
    return IMFAttributes_SetItem((*stream).attributes, key, value);
}

unsafe extern "system" fn video_stream_attributes_DeleteItem(iface: *mut IMFAttributes, key: *const GUID) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);
    TRACE!("%p, %s.\n", iface, debugstr_guid(key));
    return IMFAttributes_DeleteItem((*stream).attributes, key);
}
unsafe extern "system" fn video_stream_attributes_DeleteAllItems(iface: *mut IMFAttributes) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);

    return IMFAttributes_DeleteAllItems((*stream).attributes);
}
unsafe extern "system" fn video_stream_attributes_SetUINT32(iface: *mut IMFAttributes, key: *const GUID, value: UINT32) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as core::ffi::c_int & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8 as *const core::ffi::c_char,
            b"video_stream_attributes_SetUINT32\0" as *const u8 as *const core::ffi::c_char,
            897 as core::ffi::c_int,
            b"%p, %s, %u.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            value,
        );
    }
    return IMFAttributes_SetUINT32((*stream).attributes, key, value);
}
unsafe extern "system" fn video_stream_attributes_SetUINT64(iface: *mut IMFAttributes, key: *const GUID, value: UINT64) -> HRESULT {
    let stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);
    TRACE!("%p, %s, %s.\n", iface, debugstr_guid(key), wine_dbgstr_longlong(value));
    return IMFAttributes_SetUINT64((*stream).attributes, key, value);
}
unsafe extern "system" fn video_stream_attributes_SetDouble(iface: *mut IMFAttributes, key: *const GUID, value: f64) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);

    return IMFAttributes_SetDouble((*stream).attributes, key, value);
}

unsafe extern "system" fn video_stream_attributes_SetGUID(iface: *mut IMFAttributes, key: *const GUID, value: *const GUID) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);

    return IMFAttributes_SetGUID((*stream).attributes, key, value);
}

unsafe extern "system" fn video_stream_attributes_SetString(iface: *mut IMFAttributes, key: REFGUID, value: *const WCHAR) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8 as *const i8,
            b"video_stream_attributes_SetString\0" as *const u8 as *const i8,
            934i32,
            b"%p, %s, %s.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_guid(key as *const _GUID),
            debugstr_w(value),
        );
    }
    return IMFAttributes_SetString((*stream).attributes, key, value);
}

unsafe extern "system" fn video_stream_attributes_SetBlob(
    iface: *mut IMFAttributes,
    key: *const GUID,
    buf: *const UINT8,
    size: UINT32,
) -> HRESULT {
    let stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8
                as *const ::core::ffi::c_char,
            b"video_stream_attributes_SetBlob\0" as *const u8 as *const ::core::ffi::c_char,
            944 as ::core::ffi::c_int,
            b"%p, %s, %p, %u.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            buf,
            size,
        );
    }

    return IMFAttributes_SetBlob((*stream).attributes, key, buf, size);
}

unsafe extern "system" fn video_stream_attributes_SetUnknown(iface: *mut IMFAttributes, key: *const GUID, unknown: *mut IUnknown) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);

    return IMFAttributes_SetUnknown((*stream).attributes, key, unknown);
}

unsafe extern "system" fn video_stream_attributes_LockStore(iface: *mut IMFAttributes) -> HRESULT {
    let stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);

    return IMFAttributes_LockStore((*stream).attributes);
}

unsafe extern "system" fn video_stream_attributes_UnlockStore(iface: *mut IMFAttributes) -> HRESULT {
    let stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);

    return IMFAttributes_UnlockStore((*stream).attributes);
}

unsafe extern "system" fn video_stream_attributes_GetCount(iface: *mut IMFAttributes, count: *mut UINT32) -> HRESULT {
    let stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);

    return IMFAttributes_GetCount((*stream).attributes, count);
}

unsafe extern "system" fn video_stream_attributes_GetItemByIndex(iface: *mut IMFAttributes, index: UINT32, key: *mut GUID, value: *mut PROPVARIANT) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);
    TRACE!("%p, %u, %p, %p.\n", iface, index, key, value);
    return IMFAttributes_GetItemByIndex((*stream).attributes, index, key, value);
}

unsafe extern "system" fn video_stream_attributes_CopyAllItems(iface: *mut IMFAttributes, dest: *mut IMFAttributes) -> HRESULT {
    let mut stream: *mut video_stream = impl_from_stream_IMFAttributes(iface);

    return IMFAttributes_CopyAllItems((*stream).attributes, dest);
}

unsafe extern "C" fn video_renderer_is_mixer_d3d_aware(renderer: *const video_renderer) -> BOOL {
    let mut attributes: *mut IMFAttributes = core::ptr::null_mut::<IMFAttributes>();
    let mut value: core::ffi::c_uint = 0 as core::ffi::c_uint;
    let mut ret: BOOL = 0;
    if (IMFTransform_QueryInterface(
        (*renderer).mixer,
        &raw const IID_IMFAttributes,
        &raw mut attributes as *mut *mut core::ffi::c_void,
    ) < 0 as HRESULT)
    {
        return FALSE as BOOL;
    }
    ret = (IMFAttributes_GetUINT32(
        attributes,
        &raw const MF_SA_D3D_AWARE,
        &raw mut value,
    ) >= 0 as HRESULT
        && value != 0) as core::ffi::c_int as BOOL;
    IMFAttributes_Release(attributes);
    return ret;
}

unsafe extern "C" fn video_renderer_stream_create(
    mut renderer: *mut video_renderer,
    mut id: ::core::ffi::c_uint,
    mut ret: *mut *mut video_stream,
) -> HRESULT {
    let mut stream: *mut video_stream = ::core::ptr::null_mut::<video_stream>();
    let mut attributes: *mut IMFAttributes = ::core::ptr::null_mut::<IMFAttributes>();
    let mut value: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;

    stream = calloc(
        (1 as size_t) as _,
        (::core::mem::size_of::<video_stream>() as size_t) as _,
    ) as *mut video_stream;
    if stream.is_null() {
        return E_OUTOFMEMORY;
    }

    (*stream).IMFStreamSink_iface.lpVtbl = &raw const video_stream_sink_vtbl;
    (*stream).IMFMediaTypeHandler_iface.lpVtbl = &raw const video_stream_type_handler_vtbl;
    (*stream).IMFGetService_iface.lpVtbl = &raw const video_stream_get_service_vtbl;
    (*stream).IMFAttributes_iface.lpVtbl = &raw const video_stream_attributes_vtbl;
    (*stream).refcount = 1 as LONG;
    InitializeCriticalSection(&raw mut (*stream).cs);

    'failed: {
        hr = MFCreateEventQueue(&raw mut (*stream).event_queue);
        if !(hr < 0 as HRESULT) {
        } else {
            break 'failed;
        }

        hr = MFCreateAttributes(
            &raw mut (*stream).attributes,
            0 as UINT32,
        );
        if !(hr < 0 as HRESULT) {
        } else {
            break 'failed;
        }

        (*stream).parent = renderer;
        IMFMediaSink_AddRef(&raw mut (*(*stream).parent).IMFMediaSink_iface);
        (*stream).id = id;

        if video_renderer_is_mixer_d3d_aware(renderer) != 0 {
            IMFAttributes_SetUINT32(
                (*stream).attributes,
                &raw const MF_SA_D3D_AWARE,
                1 as UINT32,
            );
        }

        if IMFTransform_GetInputStreamAttributes(
            (*renderer).mixer,
            id as DWORD,
            &raw mut attributes,
        ) >= 0 as HRESULT
        {
            if IMFAttributes_GetUINT32(
                attributes,
                &raw const MF_SA_REQUIRED_SAMPLE_COUNT,
                &raw mut value,
            ) >= 0 as HRESULT
            {
                IMFAttributes_SetUINT32(
                    (*stream).attributes,
                    &raw const MF_SA_REQUIRED_SAMPLE_COUNT,
                    value as UINT32,
                );
            }
            IMFAttributes_Release(attributes);
        }

        *ret = stream;

        return S_OK;
    }

    IMFStreamSink_Release(&raw mut (*stream).IMFStreamSink_iface);

    return hr;
}
unsafe extern "system" fn video_renderer_sink_QueryInterface(
    iface: *mut IMFMediaSink,
    riid: REFIID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFMediaSink(iface);

    if IsEqualIID!(riid, &raw const IID_IMFMediaSink) != 0
        || IsEqualIID!(riid, &raw const IID_IUnknown) != 0
    {
        *obj = &raw mut (*renderer).IMFMediaSink_iface as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, &raw const IID_IMFMediaSinkPreroll) != 0 {
        *obj = &raw mut (*renderer).IMFMediaSinkPreroll_iface as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, &raw const IID_IMFVideoRenderer) != 0 {
        *obj = &raw mut (*renderer).IMFVideoRenderer_iface as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, &raw const IID_IMFMediaEventGenerator) != 0 {
        *obj = &raw mut (*renderer).IMFMediaEventGenerator_iface as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, &raw const IID_IMFClockStateSink) != 0 {
        *obj = &raw mut (*renderer).IMFClockStateSink_iface as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, &raw const IID_IMFGetService) != 0 {
        *obj = &raw mut (*renderer).IMFGetService_iface as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, &raw const IID_IMFAttributes) != 0 {
        *obj = &raw mut (*renderer).IMFAttributes_iface as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, &raw const IID_IMFQualityAdvise) != 0 {
        *obj = &raw mut (*renderer).IMFQualityAdvise_iface as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, &raw const IID_IMFRateSupport) != 0 {
        *obj = &raw mut (*renderer).IMFRateSupport_iface as *mut core::ffi::c_void;
    } else {
        WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
        *obj = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*obj as *mut IUnknown);

    return S_OK;
}

unsafe extern "system" fn video_renderer_sink_AddRef(iface: *mut IMFMediaSink) -> ULONG {
    let renderer: *mut video_renderer = impl_from_IMFMediaSink(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*renderer).refcount) as *mut LONG) as ULONG;
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        TRACE!("%p, refcount %lu.\n", iface, refcount);
    }
    return refcount;
}
unsafe extern "system" fn video_renderer_sink_Release(iface: *mut IMFMediaSink) -> ULONG {
    let renderer: *mut video_renderer = impl_from_IMFMediaSink(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*renderer).refcount) as *mut core::ffi::c_long,
    ) as ULONG;

    TRACE!("%p, refcount %lu.\n", iface, refcount);

    if refcount == 0 {
        if !(*renderer).event_queue.is_null() {
            IMFMediaEventQueue_Release((*renderer).event_queue);
        }
        if !(*renderer).mixer.is_null() {
            IMFTransform_Release((*renderer).mixer);
        }
        if !(*renderer).presenter.is_null() {
            IMFVideoPresenter_Release((*renderer).presenter);
        }
        if !(*renderer).device_manager.is_null() {
            IUnknown_Release((*renderer).device_manager);
        }
        if !(*renderer).clock.is_null() {
            IMFPresentationClock_Release((*renderer).clock);
        }
        if !(*renderer).attributes.is_null() {
            IMFAttributes_Release((*renderer).attributes);
        }
        DeleteCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
        free(renderer as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn video_renderer_sink_GetCharacteristics(iface: *mut IMFMediaSink, flags: *mut DWORD) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFMediaSink(iface);

    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8 as *const i8,
            b"video_renderer_sink_GetCharacteristics\0" as *const u8 as *const i8,
            1200 as i32,
            b"%p, %p.\n\0" as *const u8 as *const i8,
            iface,
            flags,
        );
    }

    if (*renderer).flags & EVR_SHUT_DOWN as u32 != 0 {
        return MF_E_SHUTDOWN;
    }

    *flags = (MEDIASINK_CLOCK_REQUIRED | MEDIASINK_CAN_PREROLL) as DWORD;

    return S_OK;
}

unsafe extern "C" fn video_renderer_add_stream(renderer: *mut video_renderer, id: ::core::ffi::c_uint, stream_sink: *mut *mut IMFStreamSink) -> HRESULT {
    let mut stream: *mut video_stream = ::core::ptr::null_mut();
    let mut hr: HRESULT;

    if mf_array_reserve(
        &raw mut (*renderer).streams as *mut *mut ::core::ffi::c_void,
        &raw mut (*renderer).stream_size,
        (*renderer).stream_count + 1,
        ::core::mem::size_of::<video_stream>() as size_t,
    ) == 0
    {
        return E_OUTOFMEMORY;
    }

    hr = video_renderer_stream_create(renderer, id, &raw mut stream);
    if hr >= 0 {
        if !stream_sink.is_null() {
            *stream_sink = &raw mut (*stream).IMFStreamSink_iface;
            IMFStreamSink_AddRef(*stream_sink);
        }
        let fresh15 = (*renderer).stream_count;
        (*renderer).stream_count = fresh15.wrapping_add(1);
        *(*renderer).streams.add(fresh15) = stream;
    }

    return hr;
}

unsafe extern "system" fn video_renderer_sink_AddStreamSink(
    iface: *mut IMFMediaSink,
    mut id: DWORD,
    media_type: *mut IMFMediaType,
    stream_sink: *mut *mut IMFStreamSink,
) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFMediaSink(iface);
    let mut hr: HRESULT;

    /* Rely on mixer for stream id validation. */

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    if (*renderer).flags & EVR_SHUT_DOWN != 0 {
        hr = MF_E_SHUTDOWN;
    } else {
        hr = IMFTransform_AddInputStreams(
            (*renderer).mixer,
            1,
            core::ptr::addr_of_mut!(id),
        );
        if hr >= 0 {
            hr = video_renderer_add_stream(renderer, id, stream_sink);
            if hr < 0 {
                IMFTransform_DeleteInputStream((*renderer).mixer, id);
            }
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "system" fn video_renderer_sink_RemoveStreamSink(iface: *mut IMFMediaSink, id: DWORD) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFMediaSink(iface);
    let mut hr: HRESULT = 0;
    let mut i: size_t = 0;

    /* Rely on mixer for stream id validation. */

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    if ((*renderer).flags & EVR_SHUT_DOWN) != 0 {
        hr = MF_E_SHUTDOWN;
    } else {
        hr = IMFTransform_DeleteInputStream((*renderer).mixer, id);
        if hr >= 0 {
            i = 0 as size_t;
            while i < (*renderer).stream_count {
                if (**(*renderer).streams.add(i)).id == id {
                    IMFStreamSink_Release(core::ptr::addr_of_mut!(
                        (**(*renderer).streams.add(i)).IMFStreamSink_iface
                    ));
                    *(*renderer).streams.add(i) = core::ptr::null_mut::<video_stream>();
                    if i < (*renderer).stream_count.wrapping_sub(1) {
                        memmove(
                            (*renderer).streams.add(i) as *mut core::ffi::c_void,
                            (*renderer).streams.add(i.wrapping_add(1)) as *const core::ffi::c_void,
                            (*renderer)
                                .stream_count
                                .wrapping_sub(i)
                                .wrapping_sub(1)
                                .wrapping_mul(core::mem::size_of::<video_stream>()),
                        );
                    }
                    (*renderer).stream_count = (*renderer).stream_count.wrapping_sub(1);
                    break;
                } else {
                    i = i.wrapping_add(1);
                }
            }
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "system" fn video_renderer_sink_GetStreamSinkCount(iface: *mut IMFMediaSink, count: *mut DWORD) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFMediaSink(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    if (*renderer).flags & EVR_SHUT_DOWN != 0 {
        hr = MF_E_SHUTDOWN;
    } else if count.is_null() {
        hr = E_POINTER;
    } else {
        *count = (*renderer).stream_count as DWORD;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "system" fn video_renderer_sink_GetStreamSinkByIndex(iface: *mut IMFMediaSink, index: DWORD, stream: *mut *mut IMFStreamSink) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFMediaSink(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    if ((*renderer).flags & EVR_SHUT_DOWN) != 0 {
        hr = MF_E_SHUTDOWN;
    } else if stream.is_null() {
        hr = E_POINTER;
    } else if (index as usize) >= (*renderer).stream_count {
        hr = E_INVALIDARG;
    } else {
        *stream = core::ptr::addr_of_mut!((**(*renderer).streams.add(index as usize)).IMFStreamSink_iface);
        IMFStreamSink_AddRef(*stream);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}
unsafe extern "system" fn video_renderer_sink_GetStreamSinkById(iface: *mut IMFMediaSink, id: DWORD, stream: *mut *mut IMFStreamSink) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFMediaSink(iface);
    let mut hr: HRESULT = S_OK;
    let mut i: size_t;

    TRACE!("%p, %#lx, %p.\n", iface, id, stream);

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    if (*renderer).flags & EVR_SHUT_DOWN != 0 {
        hr = MF_E_SHUTDOWN;
    } else if stream.is_null() {
        hr = E_POINTER;
    } else {
        i = 0;
        while i < (*renderer).stream_count {
            if (**(*renderer).streams.offset(i as isize)).id == id {
                break;
            }
            i = i.wrapping_add(1);
        }

        if i == (*renderer).stream_count {
            hr = MF_E_INVALIDSTREAMNUMBER;
        } else {
            *stream = core::ptr::addr_of_mut!((**(*renderer).streams.offset(i as isize)).IMFStreamSink_iface);
            IMFStreamSink_AddRef(*stream);
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "C" fn video_renderer_set_presentation_clock(renderer: *mut video_renderer, clock: *mut IMFPresentationClock) {
    if !(*renderer).clock.is_null() {
        IMFPresentationClock_RemoveClockStateSink((*renderer).clock, core::ptr::addr_of_mut!((*renderer).IMFClockStateSink_iface));
        IMFPresentationClock_Release((*renderer).clock);
    }
    video_renderer_release_presenter_services(renderer);

    (*renderer).clock = clock;
    if !(*renderer).clock.is_null() {
        IMFPresentationClock_AddRef((*renderer).clock);
        IMFPresentationClock_AddClockStateSink((*renderer).clock, core::ptr::addr_of_mut!((*renderer).IMFClockStateSink_iface));
    }
    video_renderer_init_presenter_services(renderer);
}

unsafe extern "system" fn video_renderer_sink_SetPresentationClock(iface: *mut IMFMediaSink, clock: *mut IMFPresentationClock) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFMediaSink(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    if (*renderer).flags & EVR_SHUT_DOWN != 0 {
        hr = MF_E_SHUTDOWN;
    } else {
        video_renderer_set_presentation_clock(renderer, clock);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "system" fn video_renderer_sink_GetPresentationClock(iface: *mut IMFMediaSink, clock: *mut *mut IMFPresentationClock) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFMediaSink(iface);
    let mut hr: HRESULT = S_OK;

    if clock.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    if ((*renderer).flags & EVR_SHUT_DOWN) != 0 {
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
unsafe extern "system" fn video_renderer_sink_Shutdown(iface: *mut IMFMediaSink) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFMediaSink(iface);
    let mut hr: HRESULT = S_OK;
    let mut i: size_t;

    TRACE!("%p.\n", iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    if (*renderer).flags & EVR_SHUT_DOWN != 0 {
        hr = MF_E_SHUTDOWN;
    } else {
        (*renderer).flags |= EVR_SHUT_DOWN;
        /* Detach streams from the sink. */
        i = 0;
        while i < (*renderer).stream_count {
            let mut stream: *mut video_stream = *(*renderer).streams.offset(i as isize);

            EnterCriticalSection(core::ptr::addr_of_mut!((*stream).cs));
            (*stream).parent = core::ptr::null_mut::<video_renderer>();
            LeaveCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

            IMFMediaEventQueue_Shutdown((*stream).event_queue);
            IMFStreamSink_Release(core::ptr::addr_of_mut!((*stream).IMFStreamSink_iface));
            IMFMediaSink_Release(iface);
            *(*renderer).streams.offset(i as isize) = core::ptr::null_mut::<video_stream>();
            i = i.wrapping_add(1);
        }
        free((*renderer).streams as *mut core::ffi::c_void);
        (*renderer).stream_count = 0;
        (*renderer).stream_size = 0;
        IMFMediaEventQueue_Shutdown((*renderer).event_queue);
        video_renderer_set_presentation_clock(renderer, core::ptr::null_mut::<IMFPresentationClock>());
        video_renderer_release_services(renderer);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "system" fn video_renderer_preroll_QueryInterface(iface: *mut IMFMediaSinkPreroll, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFMediaSinkPreroll(iface);
    return IMFMediaSink_QueryInterface(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface), riid, obj);
}

unsafe extern "system" fn video_renderer_preroll_AddRef(iface: *mut IMFMediaSinkPreroll) -> ULONG {
    let renderer: *mut video_renderer = impl_from_IMFMediaSinkPreroll(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}
unsafe extern "system" fn video_renderer_preroll_Release(iface: *mut IMFMediaSinkPreroll) -> ULONG {
    let renderer: *mut video_renderer = impl_from_IMFMediaSinkPreroll(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn video_renderer_preroll_NotifyPreroll(iface: *mut IMFMediaSinkPreroll, start_time: MFTIME) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFMediaSinkPreroll(iface);
    let mut hr: HRESULT = S_OK;
    let mut i: size_t;

    TRACE!("%p, %s.\n", iface, debugstr_time(start_time as LONGLONG));

    EnterCriticalSection(&raw mut (*renderer).cs);
    if (*renderer).flags & (EVR_SHUT_DOWN as ::core::ffi::c_uint) != 0 {
        hr = MF_E_SHUTDOWN;
    } else {
        i = 0 as size_t;
        while i < (*renderer).stream_count {
            let stream: *mut video_stream = *(*renderer).streams.offset(i as isize);

            EnterCriticalSection(&raw mut (*stream).cs);
            if ((*stream).flags
                & ((EVR_STREAM_PREROLLING as ::core::ffi::c_uint)
                    | (EVR_STREAM_PREROLLED as ::core::ffi::c_uint)))
                == 0
            {
                IMFMediaEventQueue_QueueEventParamVar(
                    (*stream).event_queue,
                    MEStreamSinkRequestSample as MediaEventType,
                    &raw const GUID_NULL,
                    S_OK,
                    core::ptr::null::<PROPVARIANT>(),
                );
                (*stream).flags |= EVR_STREAM_PREROLLING as ::core::ffi::c_uint;
            }
            LeaveCriticalSection(&raw mut (*stream).cs);
            i = i.wrapping_add(1);
        }
    }
    LeaveCriticalSection(&raw mut (*renderer).cs);

    return hr;
}

unsafe extern "system" fn video_renderer_QueryInterface(iface: *mut IMFVideoRenderer, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFVideoRenderer(iface);
    return IMFMediaSink_QueryInterface(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface), riid, obj);
}

unsafe extern "system" fn video_renderer_AddRef(iface: *mut IMFVideoRenderer) -> ULONG {
    let renderer: *mut video_renderer = impl_from_IMFVideoRenderer(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn video_renderer_Release(iface: *mut IMFVideoRenderer) -> ULONG {
    let renderer: *mut video_renderer = impl_from_IMFVideoRenderer(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "C" fn video_renderer_create_mixer(
    mut attributes: *mut IMFAttributes,
    mut out: *mut *mut IMFTransform,
) -> HRESULT {
    let mut flags: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut activate: *mut IMFActivate = ::core::ptr::null_mut::<IMFActivate>();
    let mut clsid: CLSID = _GUID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    };
    let mut hr: HRESULT = 0;
    if !attributes.is_null() && {
        hr = IMFAttributes_GetUnknown(
            attributes,
            &raw const MF_ACTIVATE_CUSTOM_VIDEO_MIXER_ACTIVATE,
            &raw const IID_IMFActivate,
            &raw mut activate as *mut LPVOID,
        );
        hr >= 0 as HRESULT
    } {
        IMFAttributes_GetUINT32(
            attributes,
            &raw const MF_ACTIVATE_CUSTOM_VIDEO_MIXER_FLAGS,
            &raw mut flags,
        );
        hr = IMFActivate_ActivateObject(
            activate,
            &raw const IID_IMFTransform,
            out as *mut *mut ::core::ffi::c_void,
        );
        IMFActivate_Release(activate);
        if hr >= 0 as HRESULT
            || (flags
                & MF_ACTIVATE_CUSTOM_MIXER_ALLOWFAIL as ::core::ffi::c_uint)
                == 0
        {
            return hr;
        }
    }
    if attributes.is_null()
        || IMFAttributes_GetGUID(
            attributes,
            &raw const MF_ACTIVATE_CUSTOM_VIDEO_MIXER_CLSID,
            &raw mut clsid,
        ) < 0 as HRESULT
    {
        memcpy(
            &raw mut clsid as *mut ::core::ffi::c_void,
            &raw const CLSID_MFVideoMixer9 as *const ::core::ffi::c_void,
            ::core::mem::size_of::<CLSID>() as size_t,
        );
    }
    return CoCreateInstance(
        &raw mut clsid,
        (::core::ptr::null_mut::<IUnknown>()).cast(),
        CLSCTX_INPROC_SERVER as DWORD,
        &raw const IID_IMFTransform,
        out as *mut LPVOID,
    );
}

unsafe extern "C" fn video_renderer_create_presenter(renderer: *mut video_renderer, attributes: *mut IMFAttributes, out: *mut *mut IMFVideoPresenter) -> HRESULT {
    let mut flags: ::core::ffi::c_uint = 0;
    let mut activate: *mut IMFActivate = ::core::ptr::null_mut::<IMFActivate>();
    let mut value: UINT64 = 0;
    let mut clsid: CLSID = ::core::mem::zeroed::<CLSID>();
    let mut hr: HRESULT = 0;

    if !attributes.is_null() && (IMFAttributes_GetUINT64(attributes, &raw const MF_ACTIVATE_VIDEO_WINDOW, &raw mut value)) >= 0 {
        (*renderer).window = UlongToHandle!(value as usize) as HWND;
    }

    if !attributes.is_null() && (IMFAttributes_GetUnknown(attributes, &raw const MF_ACTIVATE_CUSTOM_VIDEO_PRESENTER_ACTIVATE, &raw const IID_IMFActivate, &raw mut activate as *mut LPVOID)) >= 0 {
        IMFAttributes_GetUINT32(attributes, &raw const MF_ACTIVATE_CUSTOM_VIDEO_PRESENTER_FLAGS, &raw mut flags);
        hr = IMFActivate_ActivateObject(activate, &raw const IID_IMFVideoPresenter, out as *mut *mut ::core::ffi::c_void);
        IMFActivate_Release(activate);
        if hr >= 0 || (flags & MF_ACTIVATE_CUSTOM_PRESENTER_ALLOWFAIL as ::core::ffi::c_uint) == 0 {
            return hr;
        }
    }

    if attributes.is_null() || (IMFAttributes_GetGUID(attributes, &raw const MF_ACTIVATE_CUSTOM_VIDEO_PRESENTER_CLSID, &raw mut clsid)) < 0 {
        memcpy(&raw mut clsid as *mut ::core::ffi::c_void, &raw const CLSID_MFVideoPresenter9 as *const ::core::ffi::c_void, ::core::mem::size_of::<CLSID>() as size_t);
    }

    return CoCreateInstance(&raw mut clsid, (::core::ptr::null_mut::<IUnknown>()).cast(), CLSCTX_INPROC_SERVER as DWORD, &raw const IID_IMFVideoPresenter, out as *mut LPVOID);
}

unsafe extern "C" fn video_renderer_get_device_manager(renderer: *mut video_renderer, device_manager: *mut *mut IUnknown) -> HRESULT {
    return MFGetService((*renderer).presenter as *mut IUnknown, core::ptr::addr_of!(MR_VIDEO_RENDER_SERVICE),
                core::ptr::addr_of!(IID_IDirect3DDeviceManager9), device_manager as *mut *mut core::ffi::c_void);
}

unsafe extern "C" fn video_renderer_configure_mixer(mut renderer: *mut video_renderer) -> HRESULT {
    let mut lookup_client: *mut IMFTopologyServiceLookupClient =
        ::core::ptr::null_mut::<IMFTopologyServiceLookupClient>();
    let mut hr: HRESULT = 0;
    hr = IMFTransform_QueryInterface(
        (*renderer).mixer,
        &raw const IID_IMFTopologyServiceLookupClient,
        &raw mut lookup_client as *mut *mut ::core::ffi::c_void,
    );
    if hr >= 0 as HRESULT {
        (*renderer).flags |= EVR_INIT_SERVICES as ::core::ffi::c_int as ::core::ffi::c_uint;
        hr = IMFTopologyServiceLookupClient_InitServicePointers(
            lookup_client,
            &raw mut (*renderer).IMFTopologyServiceLookup_iface,
        );
        if hr >= 0 as HRESULT {
            (*renderer).flags |=
                EVR_MIXER_INITED_SERVICES as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        (*renderer).flags &= !(EVR_INIT_SERVICES as ::core::ffi::c_int) as ::core::ffi::c_uint;
        IMFTopologyServiceLookupClient_Release(lookup_client);
    }
    if hr >= 0 as HRESULT {
        let mut input_count: DWORD = 0;
        let mut output_count: DWORD = 0;
        let mut ids: *mut DWORD = ::core::ptr::null_mut::<DWORD>();
        let mut oids: *mut DWORD = ::core::ptr::null_mut::<DWORD>();
        let mut i: size_t = 0;

        /* Create stream sinks for inputs that mixer already has by default. */
        if IMFTransform_GetStreamCount(
            (*renderer).mixer,
            &raw mut input_count,
            &raw mut output_count,
        ) >= 0 as HRESULT
        {
            ids = calloc(
                (input_count as size_t) as _,
                (::core::mem::size_of::<DWORD>() as size_t) as _,
            ) as *mut DWORD;
            oids = calloc(
                (output_count as size_t) as _,
                (::core::mem::size_of::<DWORD>() as size_t) as _,
            ) as *mut DWORD;

            if !ids.is_null() && !oids.is_null() {
                if IMFTransform_GetStreamIDs(
                    (*renderer).mixer,
                    input_count,
                    ids,
                    output_count,
                    oids,
                ) >= 0 as HRESULT
                {
                    i = 0 as size_t;
                    while i < input_count as size_t {
                        video_renderer_add_stream(
                            renderer,
                            *ids.offset(i as isize) as ::core::ffi::c_uint,
                            ::core::ptr::null_mut::<*mut IMFStreamSink>(),
                        );
                        i = i.wrapping_add(1);
                    }
                }
            }

            free(ids as *mut ::core::ffi::c_void);
            free(oids as *mut ::core::ffi::c_void);
        }
    }

    /* Set device manager that presenter should have created. */
    if video_renderer_is_mixer_d3d_aware(renderer) != 0 {
        let mut device_manager: *mut IUnknown = ::core::ptr::null_mut::<IUnknown>();

        if video_renderer_get_device_manager(renderer, &raw mut device_manager) >= 0 as HRESULT {
            IMFTransform_ProcessMessage(
                (*renderer).mixer,
                MFT_MESSAGE_SET_D3D_MANAGER,
                device_manager as ULONG_PTR,
            );
            IUnknown_Release(device_manager);
        }
    }

    return hr;
}

unsafe extern "C" fn video_renderer_configure_presenter(renderer: *mut video_renderer) -> HRESULT {
    let mut control: *mut IMFVideoDisplayControl = core::ptr::null_mut();
    let mut rect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut hr: HRESULT;

    if (IMFVideoPresenter_QueryInterface((*renderer).presenter, core::ptr::addr_of!(IID_IMFVideoDisplayControl), core::ptr::addr_of_mut!(control) as *mut *mut core::ffi::c_void) >= 0)
    {
        GetClientRect((*renderer).window, core::ptr::addr_of_mut!(rect));

        hr = IMFVideoDisplayControl_SetVideoWindow(control, (*renderer).window);
        if hr >= 0 {
            hr = IMFVideoDisplayControl_SetVideoPosition(control, core::ptr::null(), core::ptr::addr_of_mut!(rect));
        }
        IMFVideoDisplayControl_Release(control);
    }

    hr = video_renderer_init_presenter_services(renderer);

    if video_renderer_get_device_manager(renderer, core::ptr::addr_of_mut!((*renderer).device_manager)) < 0
    {
        WARN!("Failed to get device manager from the presenter.\n");
    }

    return hr;
}

unsafe extern "C" fn video_renderer_uninitialize(mut renderer: *mut video_renderer) {
    video_renderer_release_services(renderer);

    if !(*renderer).mixer.is_null() {
        IMFTransform_Release((*renderer).mixer);
        (*renderer).mixer = core::ptr::null_mut();
    }

    if !(*renderer).presenter.is_null() {
        IMFVideoPresenter_Release((*renderer).presenter);
        (*renderer).presenter = core::ptr::null_mut();
    }

    if !(*renderer).device_manager.is_null() {
        IUnknown_Release((*renderer).device_manager);
        (*renderer).device_manager = core::ptr::null_mut();
    }
}

unsafe extern "C" fn video_renderer_initialize(renderer: *mut video_renderer, mixer: *mut IMFTransform, presenter: *mut IMFVideoPresenter) -> HRESULT {
    let mut hr: HRESULT = 0;

    (*renderer).mixer = mixer;
    IMFTransform_AddRef((*renderer).mixer);

    (*renderer).presenter = presenter;
    IMFVideoPresenter_AddRef((*renderer).presenter);

    if {
        hr = video_renderer_configure_mixer(renderer);
        hr
    } >= 0 {
        hr = video_renderer_configure_presenter(renderer);
    }

    return hr;
}

unsafe extern "C" fn video_renderer_create_mixer_and_presenter(
    renderer: *mut video_renderer,
    mixer: *mut *mut IMFTransform,
    presenter: *mut *mut IMFVideoPresenter,
) -> HRESULT {
    let mut hr: HRESULT;

    if !(*mixer).is_null() {
        IMFTransform_AddRef(*mixer);
    } else {
        hr = video_renderer_create_mixer(core::ptr::null_mut(), mixer);
        if hr < 0 {
            WARN!("Failed to create default mixer object, hr %#lx.\n", hr);
            return hr;
        }
    }

    if !(*presenter).is_null() {
        IMFVideoPresenter_AddRef(*presenter);
    } else {
        hr = video_renderer_create_presenter(renderer, core::ptr::null_mut(), presenter);
        if hr < 0 {
            WARN!("Failed to create default presenter, hr %#lx.\n", hr);
            IMFTransform_Release(*mixer);
            return hr;
        }
    }

    return S_OK;
}

unsafe extern "system" fn video_renderer_InitializeRenderer(iface: *mut IMFVideoRenderer, mut mixer: *mut IMFTransform, mut presenter: *mut IMFVideoPresenter) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFVideoRenderer(iface);
    let mut hr: HRESULT = 0;

    TRACE!("%p, %p, %p.\n", iface, mixer, presenter);

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    if ((*renderer).flags & EVR_SHUT_DOWN) != 0 {
        hr = MF_E_SHUTDOWN;
    } else {
        video_renderer_uninitialize(renderer);

        hr = video_renderer_create_mixer_and_presenter(renderer, core::ptr::addr_of_mut!(mixer), core::ptr::addr_of_mut!(presenter));
        if hr >= 0 {
            /* FIXME: check clock state */
            /* FIXME: check that streams are not initialized */

            hr = video_renderer_initialize(renderer, mixer, presenter);

            IMFTransform_Release(mixer);
            IMFVideoPresenter_Release(presenter);
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "system" fn video_renderer_events_QueryInterface(iface: *mut IMFMediaEventGenerator, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFMediaEventGenerator(iface);
    return IMFMediaSink_QueryInterface(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface), riid, obj);
}

unsafe extern "system" fn video_renderer_events_AddRef(iface: *mut IMFMediaEventGenerator) -> ULONG {
    let renderer: *mut video_renderer = impl_from_IMFMediaEventGenerator(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn video_renderer_events_Release(iface: *mut IMFMediaEventGenerator) -> ULONG {
    let renderer: *mut video_renderer = impl_from_IMFMediaEventGenerator(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}
unsafe extern "system" fn video_renderer_events_GetEvent(
    iface: *mut IMFMediaEventGenerator,
    flags: DWORD,
    event: *mut *mut IMFMediaEvent,
) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFMediaEventGenerator(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8
                as *const i8,
            b"video_renderer_events_GetEvent\0" as *const u8 as *const i8,
            1848i32,
            b"%p, %#lx, %p.\n\0" as *const u8 as *const i8,
            iface,
            flags,
            event,
        );
    }
    return IMFMediaEventQueue_GetEvent((*renderer).event_queue, flags, event);
}

unsafe extern "system" fn video_renderer_events_BeginGetEvent(
    mut iface: *mut IMFMediaEventGenerator,
    mut callback: *mut IMFAsyncCallback,
    mut state: *mut IUnknown,
) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFMediaEventGenerator(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0".as_ptr() as *const i8,
            b"video_renderer_events_BeginGetEvent\0".as_ptr() as *const i8,
            1858i32,
            b"%p, %p, %p.\n\0".as_ptr() as *const i8,
            iface,
            callback,
            state,
        );
    }
    return IMFMediaEventQueue_BeginGetEvent((*renderer).event_queue, callback, state);
}

unsafe extern "system" fn video_renderer_events_EndGetEvent(iface: *mut IMFMediaEventGenerator, result: *mut IMFAsyncResult, event: *mut *mut IMFMediaEvent) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFMediaEventGenerator(iface);

    return IMFMediaEventQueue_EndGetEvent((*renderer).event_queue, result, event);
}

unsafe extern "system" fn video_renderer_events_QueueEvent(
    iface: *mut IMFMediaEventGenerator,
    event_type: MediaEventType,
    ext_type: *const GUID,
    hr: HRESULT,
    value: *const PROPVARIANT,
) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFMediaEventGenerator(iface);
    return IMFMediaEventQueue_QueueEventParamVar((*renderer).event_queue, event_type, ext_type, hr, value);
}

unsafe extern "system" fn video_renderer_clock_sink_QueryInterface(iface: *mut IMFClockStateSink, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFClockStateSink(iface);
    return IMFMediaSink_QueryInterface(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface), riid, obj);
}

unsafe extern "system" fn video_renderer_clock_sink_AddRef(iface: *mut IMFClockStateSink) -> ULONG {
    let mut renderer: *mut video_renderer = impl_from_IMFClockStateSink(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn video_renderer_clock_sink_Release(iface: *mut IMFClockStateSink) -> ULONG {
    let renderer: *mut video_renderer = impl_from_IMFClockStateSink(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn video_renderer_clock_sink_OnClockStart(iface: *mut IMFClockStateSink, systime: MFTIME, offset: LONGLONG) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFClockStateSink(iface);
    let mut state: u32;
    let mut request_sample: u32;
    let mut i: usize;

    TRACE!("%p, %s, %s.\n", iface, debugstr_time(systime as LONGLONG), debugstr_time(offset));

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    state = (*renderer).state;

    /* Update sink state before sending sample requests, to avoid potentially receiving new sample in stopped state */
    (*renderer).state = EVR_STATE_RUNNING;

    if state == EVR_STATE_STOPPED {
        IMFTransform_ProcessMessage((*renderer).mixer, MFT_MESSAGE_NOTIFY_BEGIN_STREAMING, 0);
        IMFVideoPresenter_ProcessMessage((*renderer).presenter, MFVP_MESSAGE_BEGINSTREAMING, 0);

        i = 0;
        while i < (*renderer).stream_count {
            let stream: *mut video_stream = *(*renderer).streams.add(i);

            EnterCriticalSection(core::ptr::addr_of_mut!((*stream).cs));
            request_sample = (((*stream).flags & EVR_STREAM_PREROLLED) == 0
                || ((*stream).flags & EVR_STREAM_SAMPLE_NEEDED) != 0) as u32;
            (*stream).flags |= EVR_STREAM_PREROLLED;
            (*stream).flags &= !EVR_STREAM_SAMPLE_NEEDED;
            LeaveCriticalSection(core::ptr::addr_of_mut!((*stream).cs));

            IMFMediaEventQueue_QueueEventParamVar(
                (*stream).event_queue,
                MEStreamSinkStarted,
                core::ptr::addr_of!(GUID_NULL),
                S_OK,
                core::ptr::null_mut(),
            );
            if request_sample != 0 {
                IMFMediaEventQueue_QueueEventParamVar(
                    (*stream).event_queue,
                    MEStreamSinkRequestSample,
                    core::ptr::addr_of!(GUID_NULL),
                    S_OK,
                    core::ptr::null_mut(),
                );
            }
            i = i.wrapping_add(1);
        }
    }

    IMFVideoPresenter_OnClockStart((*renderer).presenter, systime, offset);

    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return S_OK;
}

unsafe extern "system" fn video_renderer_clock_sink_OnClockStop(iface: *mut IMFClockStateSink, systime: MFTIME) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFClockStateSink(iface);
    let mut i: size_t = 0;

    TRACE!("%p, %s.\n", iface, debugstr_time(systime));

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    IMFVideoPresenter_OnClockStop((*renderer).presenter, systime);

    IMFTransform_ProcessMessage((*renderer).mixer, MFT_MESSAGE_COMMAND_FLUSH, 0);
    IMFVideoPresenter_ProcessMessage((*renderer).presenter, MFVP_MESSAGE_FLUSH, 0);

    if (*renderer).state == EVR_STATE_RUNNING as core::ffi::c_uint
        || (*renderer).state == EVR_STATE_PAUSED as core::ffi::c_uint
    {
        IMFTransform_ProcessMessage((*renderer).mixer, MFT_MESSAGE_NOTIFY_END_STREAMING, 0);
        IMFVideoPresenter_ProcessMessage((*renderer).presenter, MFVP_MESSAGE_ENDSTREAMING, 0);

        i = 0;
        while i < (*renderer).stream_count {
            let mut stream: *mut video_stream = *(*renderer).streams.add(i);

            EnterCriticalSection(core::ptr::addr_of_mut!((*stream).cs));
            (*stream).flags &= !(EVR_STREAM_PREROLLED | EVR_STREAM_SAMPLE_NEEDED);
            LeaveCriticalSection(core::ptr::addr_of_mut!((*stream).cs));
            i = i.wrapping_add(1);
        }
        (*renderer).state = EVR_STATE_STOPPED as core::ffi::c_uint;
    }

    i = 0;
    while i < (*renderer).stream_count {
        let mut stream: *mut video_stream = *(*renderer).streams.add(i);
        IMFMediaEventQueue_QueueEventParamVar((*stream).event_queue, MEStreamSinkStopped, core::ptr::addr_of!(GUID_NULL), S_OK, core::ptr::null::<PROPVARIANT>());
        i = i.wrapping_add(1);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return S_OK;
}
unsafe extern "system" fn video_renderer_clock_sink_OnClockPause(iface: *mut IMFClockStateSink, systime: MFTIME) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFClockStateSink(iface);
    let mut i: size_t;

    TRACE!("%p, %s.\n", iface, debugstr_time(systime));

    EnterCriticalSection(&raw mut (*renderer).cs);

    IMFVideoPresenter_OnClockPause((*renderer).presenter, systime);

    i = 0;
    while i < (*renderer).stream_count {
        let stream: *mut video_stream = *(*renderer).streams.offset(i as isize);
        IMFMediaEventQueue_QueueEventParamVar(
            (*stream).event_queue,
            MEStreamSinkPaused as MediaEventType,
            &raw const GUID_NULL,
            S_OK,
            core::ptr::null(),
        );
        i = i.wrapping_add(1);
    }

    (*renderer).state = EVR_STATE_PAUSED;

    LeaveCriticalSection(&raw mut (*renderer).cs);

    return S_OK;
}

unsafe extern "system" fn video_renderer_clock_sink_OnClockRestart(iface: *mut IMFClockStateSink, systime: MFTIME) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFClockStateSink(iface);
    let mut i: size_t = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8
                as *const ::core::ffi::c_char,
            b"video_renderer_clock_sink_OnClockRestart\0" as *const u8
                as *const ::core::ffi::c_char,
            2027 as ::core::ffi::c_int,
            b"%p, %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_time(systime as LONGLONG),
        );
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    IMFVideoPresenter_OnClockRestart((*renderer).presenter, systime);

    while i < (*renderer).stream_count {
        let stream: *mut video_stream = *(*renderer).streams.add(i as usize);
        IMFMediaEventQueue_QueueEventParamVar(
            (*stream).event_queue,
            MEStreamSinkStarted as ::core::ffi::c_int as MediaEventType,
            core::ptr::addr_of!(GUID_NULL),
            S_OK,
            core::ptr::null::<PROPVARIANT>(),
        );
        i = i.wrapping_add(1);
    }
    (*renderer).state = EVR_STATE_RUNNING;

    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return S_OK;
}

unsafe extern "system" fn video_renderer_clock_sink_OnClockSetRate(iface: *mut IMFClockStateSink, systime: MFTIME, rate: f32) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFClockStateSink(iface);
    let mut sink: *mut IMFClockStateSink = core::ptr::null_mut();

    TRACE!("%p, %s, %f.\n", iface, debugstr_time(systime), rate as f64);

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    IMFVideoPresenter_OnClockSetRate((*renderer).presenter, systime, rate);
    if IMFTransform_QueryInterface(
        (*renderer).mixer,
        core::ptr::addr_of!(IID_IMFClockStateSink),
        core::ptr::addr_of_mut!(sink) as *mut *mut core::ffi::c_void,
    ) >= 0
    {
        IMFClockStateSink_OnClockSetRate(sink, systime, rate);
        IMFClockStateSink_Release(sink);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return S_OK;
}

unsafe extern "system" fn video_renderer_get_service_QueryInterface(iface: *mut IMFGetService, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFGetService(iface);
    return IMFMediaSink_QueryInterface(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface), riid, obj);
}

unsafe extern "system" fn video_renderer_get_service_AddRef(iface: *mut IMFGetService) -> ULONG {
    let renderer: *mut video_renderer = impl_from_IMFGetService(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn video_renderer_get_service_Release(iface: *mut IMFGetService) -> ULONG {
    let renderer: *mut video_renderer = impl_from_IMFGetService(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn video_renderer_get_service_GetService(iface: *mut IMFGetService, service: *const GUID, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFGetService(iface);
    let mut hr: HRESULT = E_NOINTERFACE;
    let mut gs: *mut IMFGetService = core::ptr::null_mut();

    EnterCriticalSection(&raw mut (*renderer).cs);

    if IsEqualGUID!(service, &raw const MR_VIDEO_MIXER_SERVICE) != false {
        hr = IMFTransform_QueryInterface((*renderer).mixer, &raw const IID_IMFGetService, &raw mut gs as *mut *mut core::ffi::c_void);
    } else if IsEqualGUID!(service, &raw const MR_VIDEO_RENDER_SERVICE) != false {
        hr = IMFVideoPresenter_QueryInterface((*renderer).presenter, &raw const IID_IMFGetService, &raw mut gs as *mut *mut core::ffi::c_void);
    } else if IsEqualGUID!(service, &raw const MR_VIDEO_ACCELERATION_SERVICE) != false && IsEqualIID!(riid, &raw const IID_IDirect3DDeviceManager9) != 0 {
        if !(*renderer).device_manager.is_null() {
            hr = IUnknown_QueryInterface((*renderer).device_manager, riid, obj);
        }
    } else if IsEqualGUID!(service, &raw const MF_RATE_CONTROL_SERVICE) != false && IsEqualIID!(riid, &raw const IID_IMFRateSupport) != 0 {
        hr = IMFVideoRenderer_QueryInterface(&raw mut (*renderer).IMFVideoRenderer_iface, riid, obj);
    } else {
    }

    LeaveCriticalSection(&raw mut (*renderer).cs);

    if !gs.is_null() {
        hr = IMFGetService_GetService(gs, service, riid, obj as *mut LPVOID);
        IMFGetService_Release(gs);
    }

    return hr;
}
unsafe extern "system" fn video_renderer_service_lookup_QueryInterface(iface: *mut IMFTopologyServiceLookup, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFTopologyServiceLookup)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = iface as *mut core::ffi::c_void;
        IMFTopologyServiceLookup_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn video_renderer_service_lookup_AddRef(iface: *mut IMFTopologyServiceLookup) -> ULONG {
    let renderer: *mut video_renderer = impl_from_IMFTopologyServiceLookup(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn video_renderer_service_lookup_Release(iface: *mut IMFTopologyServiceLookup) -> ULONG {
    let renderer: *mut video_renderer = impl_from_IMFTopologyServiceLookup(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn video_renderer_service_lookup_LookupService(iface: *mut IMFTopologyServiceLookup, lookup_type: MF_SERVICE_LOOKUP_TYPE, index: DWORD, service: *const GUID, riid: *const IID, objects: *mut *mut ::core::ffi::c_void, num_objects: *mut DWORD) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFTopologyServiceLookup(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    if !(((*renderer).flags & EVR_INIT_SERVICES) != 0) {
        hr = MF_E_NOTACCEPTING;
    } else if IsEqualGUID!(service, core::ptr::addr_of!(MR_VIDEO_RENDER_SERVICE)) != false {
        if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMediaEventSink)) != 0 {
            *objects = core::ptr::addr_of_mut!((*renderer).IMediaEventSink_iface) as *mut ::core::ffi::c_void;
            IUnknown_AddRef(*objects as *mut IUnknown);
        } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFClock)) != 0 {
            *objects = (*renderer).clock as *mut ::core::ffi::c_void;
            if !(*objects).is_null() {
                IUnknown_AddRef(*objects as *mut IUnknown);
            }
            hr = if !(*objects).is_null() { S_OK } else { E_NOINTERFACE };
        } else {
            hr = E_NOINTERFACE;
        }
    } else if IsEqualGUID!(service, core::ptr::addr_of!(MR_VIDEO_MIXER_SERVICE)) != false {
        if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFTransform)) != 0 {
            *objects = (*renderer).mixer as *mut ::core::ffi::c_void;
            IUnknown_AddRef(*objects as *mut IUnknown);
        } else {
            hr = E_NOINTERFACE;
        }
    } else {
        WARN!("Unsupported service %s.\n", debugstr_guid(service));
        hr = MF_E_UNSUPPORTED_SERVICE;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "system" fn video_renderer_event_sink_QueryInterface(iface: *mut IMediaEventSink, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMediaEventSink)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = iface as *mut core::ffi::c_void;
        IMediaEventSink_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn video_renderer_event_sink_AddRef(iface: *mut IMediaEventSink) -> ULONG {
    let renderer: *mut video_renderer = impl_from_IMediaEventSink(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn video_renderer_event_sink_Release(iface: *mut IMediaEventSink) -> ULONG {
    let renderer: *mut video_renderer = impl_from_IMediaEventSink(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn video_renderer_event_sink_Notify(iface: *mut IMediaEventSink, event: LONG, param1: LONG_PTR, param2: LONG_PTR) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMediaEventSink(iface);
    let mut hr: HRESULT = S_OK;
    let mut idx: ::core::ffi::c_uint;

    TRACE!("%p, %ld, %Id, %Id.\n", iface, event, param1, param2);

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    if event == (EC_SAMPLE_NEEDED as i32) {
        idx = param1 as ::core::ffi::c_uint;
        if idx as size_t >= (*renderer).stream_count {
            hr = MF_E_INVALIDSTREAMNUMBER;
        } else if (*renderer).state == EVR_STATE_RUNNING as ::core::ffi::c_uint {
            hr = IMFMediaEventQueue_QueueEventParamVar(
                (**(*renderer).streams.offset(idx as isize)).event_queue,
                MEStreamSinkRequestSample as MediaEventType,
                core::ptr::addr_of!(GUID_NULL),
                S_OK,
                core::ptr::null::<PROPVARIANT>(),
            );
        } else {
            (**(*renderer).streams.offset(idx as isize)).flags |= EVR_STREAM_SAMPLE_NEEDED as ::core::ffi::c_uint;
        }
    } else if event == (EC_DISPLAY_CHANGED as i32) {
        idx = 0 as ::core::ffi::c_uint;
        while (idx as size_t) < (*renderer).stream_count {
            IMFMediaEventQueue_QueueEventParamVar(
                (**(*renderer).streams.offset(idx as isize)).event_queue,
                MEStreamSinkDeviceChanged as MediaEventType,
                core::ptr::addr_of!(GUID_NULL),
                S_OK,
                core::ptr::null::<PROPVARIANT>(),
            );
            idx = idx.wrapping_add(1);
        }
    } else if event >= (EC_USER as i32) {
        let mut code: PROPVARIANT = core::mem::zeroed();

        code.vt = VT_I4 as VARTYPE;
        code.__bindgen_anon_1.lVal = event;
        hr = IMFMediaEventQueue_QueueEventParamVar(
            (*renderer).event_queue,
            MERendererEvent as MediaEventType,
            core::ptr::addr_of!(GUID_NULL),
            S_OK,
            core::ptr::addr_of_mut!(code),
        );
    } else {
        WARN!("Unhandled event %ld.\n", event);
        hr = MF_E_UNEXPECTED;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "system" fn video_renderer_attributes_QueryInterface(iface: *mut IMFAttributes, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFAttributes(iface);
    return IMFMediaSink_QueryInterface(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface), riid, obj);
}

unsafe extern "system" fn video_renderer_attributes_AddRef(mut iface: *mut IMFAttributes) -> ULONG {
    let mut renderer: *mut video_renderer = impl_from_IMFAttributes(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn video_renderer_attributes_Release(iface: *mut IMFAttributes) -> ULONG {
    let renderer: *mut video_renderer = impl_from_IMFAttributes(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}
unsafe extern "system" fn video_renderer_attributes_GetItem(iface: *mut IMFAttributes, key: *const GUID, value: *mut PROPVARIANT) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFAttributes(iface);
    DPRINT1!("%p, %s, %p.\n", iface, debugstr_guid(key), value);
    return IMFAttributes_GetItem((*renderer).attributes, key, value);
}

unsafe extern "system" fn video_renderer_attributes_GetItemType(
    mut iface: *mut IMFAttributes,
    key: *const GUID,
    mut type_0: *mut MF_ATTRIBUTE_TYPE,
) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8
                as *const i8,
            b"video_renderer_attributes_GetItemType\0" as *const u8 as *const i8,
            2363 as i32,
            b"%p, %s, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_guid(key as *const GUID),
            type_0,
        );
    }
    return IMFAttributes_GetItemType((*renderer).attributes, key, type_0);
}

unsafe extern "system" fn video_renderer_attributes_CompareItem(iface: *mut IMFAttributes, key: *const GUID, value: *const PROPVARIANT, result: *mut BOOL) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFAttributes(iface);

    return IMFAttributes_CompareItem((*renderer).attributes, key, value, result);
}

unsafe extern "system" fn video_renderer_attributes_Compare(
    mut iface: *mut IMFAttributes,
    mut theirs: *mut IMFAttributes,
    mut type_0: MF_ATTRIBUTES_MATCH_TYPE,
    mut result: *mut BOOL,
) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8
                as *const ::core::ffi::c_char,
            b"video_renderer_attributes_Compare\0" as *const u8 as *const ::core::ffi::c_char,
            2383 as ::core::ffi::c_int,
            b"%p, %p, %d, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            theirs,
            type_0 as ::core::ffi::c_uint,
            result,
        );
    }
    return IMFAttributes_Compare((*renderer).attributes, theirs, type_0, result);
}

unsafe extern "system" fn video_renderer_attributes_GetUINT32(
    mut iface: *mut IMFAttributes,
    key: *const GUID,
    mut value: *mut UINT32,
) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8
                as *const i8,
            b"video_renderer_attributes_GetUINT32\0" as *const u8 as *const i8,
            2392 as i32,
            b"%p, %s, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_guid(key as *const _GUID),
            value,
        );
    }
    return IMFAttributes_GetUINT32((*renderer).attributes, key, value);
}

unsafe extern "system" fn video_renderer_attributes_GetUINT64(iface: *mut IMFAttributes, key: *const GUID, value: *mut UINT64) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFAttributes(iface);

    return IMFAttributes_GetUINT64((*renderer).attributes, key, value);
}
unsafe extern "system" fn video_renderer_attributes_GetDouble(
    mut iface: *mut IMFAttributes,
    key: *const GUID,
    mut value: *mut f64,
) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32
        & ((1 as i32) << (__WINE_DBCL_TRACE as i32)))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8
                as *const i8,
            b"video_renderer_attributes_GetDouble\0" as *const u8 as *const i8,
            2410 as i32,
            b"%p, %s, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_guid(key as *const _GUID),
            value,
        );
    }
    return IMFAttributes_GetDouble((*renderer).attributes, key, value);
}
unsafe extern "system" fn video_renderer_attributes_GetGUID(iface: *mut IMFAttributes, key: *const GUID, value: *mut GUID) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFAttributes(iface);

    TRACE!("%p, %s, %p.\n", iface, debugstr_guid(key), value);

    return IMFAttributes_GetGUID((*renderer).attributes, key, value);
}

unsafe extern "system" fn video_renderer_attributes_GetStringLength(
    mut iface: *mut IMFAttributes,
    key: *const GUID,
    mut length: *mut UINT32,
) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8
                as *const ::core::ffi::c_char,
            b"video_renderer_attributes_GetStringLength\0" as *const u8
                as *const ::core::ffi::c_char,
            2429 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            length,
        );
    }
    return IMFAttributes_GetStringLength((*renderer).attributes, key, length);
}

unsafe extern "system" fn video_renderer_attributes_GetString(
    iface: *mut IMFAttributes,
    key: *const GUID,
    value: *mut WCHAR,
    size: UINT32,
    length: *mut UINT32,
) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0".as_ptr() as *const i8,
            b"video_renderer_attributes_GetString\0".as_ptr() as *const i8,
            2439i32,
            b"%p, %s, %p, %u, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key as *const GUID),
            value,
            size,
            length,
        );
    }
    return IMFAttributes_GetString((*renderer).attributes, key, value, size, length);
}
unsafe extern "system" fn video_renderer_attributes_GetAllocatedString(
    mut iface: *mut IMFAttributes,
    key: *const GUID,
    mut value: *mut *mut WCHAR,
    mut length: *mut UINT32,
) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8
                as *const ::core::ffi::c_char,
            b"video_renderer_attributes_GetAllocatedString\0" as *const u8
                as *const ::core::ffi::c_char,
            2449 as ::core::ffi::c_int,
            b"%p, %s, %p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            value,
            length,
        );
    }
    return IMFAttributes_GetAllocatedString(
        (*renderer).attributes,
        key,
        value as *mut LPWSTR,
        length,
    );
}

unsafe extern "system" fn video_renderer_attributes_GetBlobSize(
    mut iface: *mut IMFAttributes,
    key: *const GUID,
    mut size: *mut UINT32,
) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0".as_ptr() as *const i8,
            b"video_renderer_attributes_GetBlobSize\0".as_ptr() as *const i8,
            2458i32,
            b"%p, %s, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key as *const _GUID),
            size,
        );
    }
    return IMFAttributes_GetBlobSize((*renderer).attributes, key, size);
}

unsafe extern "system" fn video_renderer_attributes_GetBlob(iface: *mut IMFAttributes, key: REFGUID, buf: *mut UINT8, bufsize: UINT32, blobsize: *mut UINT32) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFAttributes(iface);

    return IMFAttributes_GetBlob((*renderer).attributes, key, buf, bufsize, blobsize);
}

unsafe extern "system" fn video_renderer_attributes_GetAllocatedBlob(
    iface: *mut IMFAttributes,
    key: *const GUID,
    buf: *mut *mut UINT8,
    size: *mut UINT32,
) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFAttributes(iface);

    TRACE!(
        "%p, %s, %p, %p.\n",
        iface,
        debugstr_guid(key),
        buf,
        size
    );

    return IMFAttributes_GetAllocatedBlob((*renderer).attributes, key, buf, size);
}

unsafe extern "system" fn video_renderer_attributes_GetUnknown(iface: *mut IMFAttributes, key: *const GUID, riid: *const IID, out: *mut *mut ::core::ffi::c_void) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFAttributes(iface);

    return IMFAttributes_GetUnknown((*renderer).attributes, key, riid, out);
}

unsafe extern "system" fn video_renderer_attributes_SetItem(iface: *mut IMFAttributes, key: *const GUID, value: *const PROPVARIANT) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32) & ((1 as i32) << (__WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0".as_ptr() as *const i8,
            b"video_renderer_attributes_SetItem\0".as_ptr() as *const i8,
            2496 as i32,
            b"%p, %s, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key),
            value,
        );
    }
    return IMFAttributes_SetItem((*renderer).attributes, key, value);
}

unsafe extern "system" fn video_renderer_attributes_DeleteItem(iface: *mut IMFAttributes, key: *const GUID) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFAttributes(iface);

    return IMFAttributes_DeleteItem((*renderer).attributes, key);
}

unsafe extern "system" fn video_renderer_attributes_DeleteAllItems(iface: *mut IMFAttributes) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFAttributes(iface);

    return IMFAttributes_DeleteAllItems((*renderer).attributes);
}

unsafe extern "system" fn video_renderer_attributes_SetUINT32(iface: *mut IMFAttributes, key: *const GUID, value: UINT32) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFAttributes(iface);

    return IMFAttributes_SetUINT32((*renderer).attributes, key, value);
}

unsafe extern "system" fn video_renderer_attributes_SetUINT64(iface: *mut IMFAttributes, key: *const GUID, value: UINT64) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0".as_ptr() as *const i8,
            b"video_renderer_attributes_SetUINT64\0".as_ptr() as *const i8,
            2532 as i32,
            b"%p, %s, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key as *const GUID),
            wine_dbgstr_longlong(value as ULONGLONG),
        );
    }
    return IMFAttributes_SetUINT64((*renderer).attributes, key, value);
}

unsafe extern "system" fn video_renderer_attributes_SetDouble(iface: *mut IMFAttributes, key: *const GUID, value: f64) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFAttributes(iface);

    return IMFAttributes_SetDouble((*renderer).attributes, key, value);
}

unsafe extern "system" fn video_renderer_attributes_SetGUID(iface: *mut IMFAttributes, key: *const GUID, value: *const GUID) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFAttributes(iface);

    return IMFAttributes_SetGUID((*renderer).attributes, key, value);
}

unsafe extern "system" fn video_renderer_attributes_SetString(
    iface: *mut IMFAttributes,
    key: *const GUID,
    value: *const WCHAR,
) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFAttributes(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0".as_ptr() as *const i8,
            b"video_renderer_attributes_SetString\0".as_ptr() as *const i8,
            2560i32,
            b"%p, %s, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key as *const _GUID),
            debugstr_w(value),
        );
    }
    return IMFAttributes_SetString((*renderer).attributes, key, value as LPCWSTR);
}

unsafe extern "system" fn video_renderer_attributes_SetBlob(
    iface: *mut IMFAttributes,
    key: *const GUID,
    buf: *const UINT8,
    size: UINT32,
) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFAttributes(iface);

    TRACE!(
        "%p, %s, %p, %u.\n",
        iface,
        debugstr_guid(key),
        buf,
        size
    );

    return IMFAttributes_SetBlob((*renderer).attributes, key, buf, size);
}

unsafe extern "system" fn video_renderer_attributes_SetUnknown(
    mut iface: *mut IMFAttributes,
    key: *const GUID,
    mut unknown: *mut IUnknown,
) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8
                as *const ::core::ffi::c_char,
            b"video_renderer_attributes_SetUnknown\0" as *const u8 as *const ::core::ffi::c_char,
            2580 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            unknown,
        );
    }
    return IMFAttributes_SetUnknown((*renderer).attributes, key, unknown);
}

unsafe extern "system" fn video_renderer_attributes_LockStore(iface: *mut IMFAttributes) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFAttributes(iface);

    return IMFAttributes_LockStore((*renderer).attributes);
}
unsafe extern "system" fn video_renderer_attributes_UnlockStore(iface: *mut IMFAttributes) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFAttributes(iface);

    return IMFAttributes_UnlockStore((*renderer).attributes);
}

unsafe extern "system" fn video_renderer_attributes_GetCount(iface: *mut IMFAttributes, count: *mut UINT32) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFAttributes(iface);

    return IMFAttributes_GetCount((*renderer).attributes, count);
}

unsafe extern "system" fn video_renderer_attributes_GetItemByIndex(
    mut iface: *mut IMFAttributes,
    mut index: UINT32,
    mut key: *mut GUID,
    mut value: *mut PROPVARIANT,
) -> HRESULT {
    let mut renderer: *mut video_renderer = impl_from_IMFAttributes(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8
                as *const ::core::ffi::c_char,
            b"video_renderer_attributes_GetItemByIndex\0" as *const u8
                as *const ::core::ffi::c_char,
            2617 as ::core::ffi::c_int,
            b"%p, %u, %p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            index,
            key,
            value,
        );
    }
    return IMFAttributes_GetItemByIndex((*renderer).attributes, index, key, value);
}

unsafe extern "system" fn video_renderer_attributes_CopyAllItems(iface: *mut IMFAttributes, dest: *mut IMFAttributes) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFAttributes(iface);

    return IMFAttributes_CopyAllItems((*renderer).attributes, dest);
}
unsafe extern "system" fn video_renderer_quality_advise_QueryInterface(iface: *mut IMFQualityAdvise, riid: *const IID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFQualityAdvise(iface);
    return IMFMediaSink_QueryInterface(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface), riid, out);
}

unsafe extern "system" fn video_renderer_quality_advise_AddRef(iface: *mut IMFQualityAdvise) -> ULONG {
    let renderer: *mut video_renderer = impl_from_IMFQualityAdvise(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "system" fn video_renderer_quality_Release(iface: *mut IMFQualityAdvise) -> ULONG {
    let renderer: *mut video_renderer = impl_from_IMFQualityAdvise(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}
unsafe extern "system" fn video_renderer_quality_advise_SetDropMode(
    iface: *mut IMFQualityAdvise,
    mode: MF_QUALITY_DROP_MODE,
) -> HRESULT {
    // FIXME("%p, %u.\n", iface, mode);
    E_NOTIMPL
}
unsafe extern "system" fn video_renderer_quality_advise_SetQualityLevel(
    iface: *mut IMFQualityAdvise,
    level: MF_QUALITY_LEVEL,
) -> HRESULT {
    // FIXME("%p, %u.\n", iface, level);
    E_NOTIMPL
}
unsafe extern "system" fn video_renderer_quality_advise_GetDropMode(
    iface: *mut IMFQualityAdvise,
    mode: *mut MF_QUALITY_DROP_MODE,
) -> HRESULT {
    // FIXME("%p, %p.\n", iface, mode);
    E_NOTIMPL
}
unsafe extern "system" fn video_renderer_quality_advise_GetQualityLevel(
    iface: *mut IMFQualityAdvise,
    level: *mut MF_QUALITY_LEVEL,
) -> HRESULT {
    // FIXME("%p, %p.\n", iface, level);
    E_NOTIMPL
}
unsafe extern "system" fn video_renderer_quality_advise_DropTime(
    iface: *mut IMFQualityAdvise,
    interval: LONGLONG,
) -> HRESULT {
    // FIXME("%p, %s.\n", iface, wine_dbgstr_longlong(interval));
    E_NOTIMPL
}

unsafe extern "system" fn video_renderer_rate_support_QueryInterface(
    iface: *mut IMFRateSupport,
    riid: *const IID,
    out: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFRateSupport(iface);
    return IMFMediaSink_QueryInterface(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface), riid, out);
}

unsafe extern "system" fn video_renderer_rate_support_AddRef(iface: *mut IMFRateSupport) -> ULONG {
    let renderer: *mut video_renderer = impl_from_IMFRateSupport(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}
unsafe extern "system" fn video_renderer_rate_support_Release(iface: *mut IMFRateSupport) -> ULONG {
    let renderer: *mut video_renderer = impl_from_IMFRateSupport(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*renderer).IMFMediaSink_iface));
}

unsafe extern "C" fn video_renderer_is_main_stream_configured(renderer: *const video_renderer) -> BOOL {
    let mut media_type: *mut IMFMediaType = core::ptr::null_mut();
    let mut hr: HRESULT;

    hr = IMFTransform_GetInputCurrentType((*renderer).mixer, 0, core::ptr::addr_of_mut!(media_type));
    if hr >= 0 {
        IMFMediaType_Release(media_type);
    }

    return (hr >= 0) as BOOL;
}

unsafe extern "system" fn video_renderer_rate_support_GetSlowestRate(
    iface: *mut IMFRateSupport,
    direction: MFRATE_DIRECTION,
    thin: BOOL,
    rate: *mut f32,
) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFRateSupport(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    if (*renderer).flags & EVR_SHUT_DOWN != 0 {
        hr = MF_E_SHUTDOWN;
    } else if rate.is_null() {
        hr = E_POINTER;
    } else {
        *rate = 0.0f32;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}
unsafe extern "system" fn video_renderer_rate_support_GetFastestRate(iface: *mut IMFRateSupport, direction: MFRATE_DIRECTION, thin: BOOL, rate: *mut f32) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFRateSupport(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    if (*renderer).flags & EVR_SHUT_DOWN != 0 {
        hr = MF_E_SHUTDOWN;
    } else if rate.is_null() {
        hr = E_POINTER;
    } else if video_renderer_is_main_stream_configured(renderer) != 0 {
        *rate = if direction == MFRATE_FORWARD { FLT_MAX } else { -FLT_MAX };
    } else {
        hr = MF_E_INVALIDREQUEST;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}
unsafe extern "system" fn video_renderer_rate_support_IsRateSupported(
    iface: *mut IMFRateSupport,
    thin: BOOL,
    rate: f32,
    nearest_rate: *mut f32,
) -> HRESULT {
    let renderer: *mut video_renderer = impl_from_IMFRateSupport(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));
    if ((*renderer).flags & EVR_SHUT_DOWN) != 0 {
        hr = MF_E_SHUTDOWN;
    } else {
        if thin == 0 && video_renderer_is_main_stream_configured(renderer) == 0 {
            hr = MF_E_INVALIDREQUEST;
        }

        if !nearest_rate.is_null() {
            *nearest_rate = rate;
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*renderer).cs));

    return hr;
}

unsafe extern "C" fn evr_create_object(
    mut attributes: *mut IMFAttributes,
    mut user_context: *mut ::core::ffi::c_void,
    mut obj: *mut *mut IUnknown,
) -> HRESULT {
    let mut object: *mut video_renderer = core::ptr::null_mut();
    let mut presenter: *mut IMFVideoPresenter = core::ptr::null_mut();
    let mut mixer: *mut IMFTransform = core::ptr::null_mut();
    let mut hr: HRESULT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/evr.c\0" as *const u8
                as *const ::core::ffi::c_char,
            b"evr_create_object\0" as *const u8 as *const ::core::ffi::c_char,
            2853 as ::core::ffi::c_int,
            b"%p, %p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            attributes,
            user_context,
            obj,
        );
    }

    object = calloc(
        1,
        (core::mem::size_of::<video_renderer>()) as _,
    ) as *mut video_renderer;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).IMFMediaSink_iface.lpVtbl = core::ptr::addr_of!(video_renderer_sink_vtbl);
    (*object).IMFMediaSinkPreroll_iface.lpVtbl = core::ptr::addr_of!(video_renderer_preroll_vtbl);
    (*object).IMFVideoRenderer_iface.lpVtbl = core::ptr::addr_of!(video_renderer_vtbl);
    (*object).IMFMediaEventGenerator_iface.lpVtbl = core::ptr::addr_of!(video_renderer_events_vtbl);
    (*object).IMFClockStateSink_iface.lpVtbl = core::ptr::addr_of!(video_renderer_clock_sink_vtbl);
    (*object).IMFGetService_iface.lpVtbl = core::ptr::addr_of!(video_renderer_get_service_vtbl);
    (*object).IMFTopologyServiceLookup_iface.lpVtbl =
        core::ptr::addr_of!(video_renderer_service_lookup_vtbl);
    (*object).IMediaEventSink_iface.lpVtbl = core::ptr::addr_of!(media_event_sink_vtbl);
    (*object).IMFAttributes_iface.lpVtbl = core::ptr::addr_of!(video_renderer_attributes_vtbl);
    (*object).IMFQualityAdvise_iface.lpVtbl = core::ptr::addr_of!(video_renderer_quality_advise_vtbl);
    (*object).IMFRateSupport_iface.lpVtbl = core::ptr::addr_of!(video_renderer_rate_support_vtbl);
    (*object).refcount = 1;
    InitializeCriticalSection(core::ptr::addr_of_mut!((*object).cs));

    'failed: {
        hr = MFCreateEventQueue(core::ptr::addr_of_mut!((*object).event_queue));
        if hr < 0 {
            break 'failed;
        }

        hr = MFCreateAttributes(
            core::ptr::addr_of_mut!((*object).attributes),
            0,
        );
        if hr < 0 {
            break 'failed;
        }

        /* Create mixer and presenter. */
        hr = video_renderer_create_mixer(attributes, core::ptr::addr_of_mut!(mixer));
        if hr < 0 {
            break 'failed;
        }

        hr = video_renderer_create_presenter(object, attributes, core::ptr::addr_of_mut!(presenter));
        if hr < 0 {
            break 'failed;
        }

        hr = video_renderer_initialize(object, mixer, presenter);
        if hr < 0 {
            break 'failed;
        }

        IMFTransform_Release(mixer);
        IMFVideoPresenter_Release(presenter);

        /* Default attributes */
        IMFAttributes_SetUINT32(
            (*object).attributes,
            core::ptr::addr_of!(EVRConfig_ForceBob),
            0,
        );
        IMFAttributes_SetUINT32(
            (*object).attributes,
            core::ptr::addr_of!(EVRConfig_AllowDropToBob),
            0,
        );
        IMFAttributes_SetUINT32(
            (*object).attributes,
            core::ptr::addr_of!(EVRConfig_ForceThrottle),
            0,
        );
        IMFAttributes_SetUINT32(
            (*object).attributes,
            core::ptr::addr_of!(EVRConfig_AllowDropToThrottle),
            0,
        );
        IMFAttributes_SetUINT32(
            (*object).attributes,
            core::ptr::addr_of!(EVRConfig_ForceHalfInterlace),
            0,
        );
        IMFAttributes_SetUINT32(
            (*object).attributes,
            core::ptr::addr_of!(EVRConfig_AllowDropToHalfInterlace),
            0,
        );
        IMFAttributes_SetUINT32(
            (*object).attributes,
            core::ptr::addr_of!(EVRConfig_ForceScaling),
            0,
        );
        IMFAttributes_SetUINT32(
            (*object).attributes,
            core::ptr::addr_of!(EVRConfig_AllowScaling),
            0,
        );
        IMFAttributes_SetUINT32(
            (*object).attributes,
            core::ptr::addr_of!(EVRConfig_ForceBatching),
            0,
        );
        IMFAttributes_SetUINT32(
            (*object).attributes,
            core::ptr::addr_of!(EVRConfig_AllowBatching),
            0,
        );

        *obj = core::ptr::addr_of_mut!((*object).IMFMediaSink_iface) as *mut IUnknown;

        return S_OK;
    }

    /* failed: */
    if !mixer.is_null() {
        IMFTransform_Release(mixer);
    }

    if !presenter.is_null() {
        IMFVideoPresenter_Release(presenter);
    }

    video_renderer_release_services(object);
    IMFMediaSink_Release(core::ptr::addr_of_mut!((*object).IMFMediaSink_iface));

    return hr;
}

unsafe extern "C" fn evr_shutdown_object(user_context: *mut core::ffi::c_void, obj: *mut IUnknown) {
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
pub unsafe extern "system" fn MFCreateVideoRendererActivate(hwnd: HWND, activate: *mut *mut IMFActivate) -> HRESULT {
    let mut hr: HRESULT;

    if activate.is_null() {
        return E_POINTER;
    }

    hr = create_activation_object(core::ptr::null_mut(), core::ptr::addr_of!(evr_activate_funcs), activate);
    if hr >= 0 {
        IMFActivate_SetUINT64(*activate, core::ptr::addr_of!(MF_ACTIVATE_VIDEO_WINDOW), hwnd as UINT64);
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateVideoRenderer(riid: *const IID, renderer: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut obj: *mut IUnknown = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    *renderer = core::ptr::null_mut();

    hr = evr_create_object(core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(obj));
    if hr >= 0 {
        hr = IUnknown_QueryInterface(obj, riid, renderer);
        IUnknown_Release(obj);
    }

    return hr;
}

// COM vtable tables — generated from evr.c by vtbl_gen.py
/// evr.c:488 `static const IMFStreamSinkVtbl video_stream_sink_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_stream_sink_vtbl: IMFStreamSinkVtbl = IMFStreamSinkVtbl {
    QueryInterface: Some(video_stream_sink_QueryInterface),
    AddRef: Some(video_stream_sink_AddRef),
    Release: Some(video_stream_sink_Release),
    GetEvent: Some(video_stream_sink_GetEvent),
    BeginGetEvent: Some(video_stream_sink_BeginGetEvent),
    EndGetEvent: Some(video_stream_sink_EndGetEvent),
    QueueEvent: Some(video_stream_sink_QueueEvent),
    GetMediaSink: Some(video_stream_sink_GetMediaSink),
    GetIdentifier: Some(video_stream_sink_GetIdentifier),
    GetMediaTypeHandler: Some(video_stream_sink_GetMediaTypeHandler),
    ProcessSample: Some(video_stream_sink_ProcessSample),
    PlaceMarker: Some(video_stream_sink_PlaceMarker),
    Flush: Some(video_stream_sink_Flush),
};

/// evr.c:618 `static const IMFMediaTypeHandlerVtbl video_stream_type_handler_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_stream_type_handler_vtbl: IMFMediaTypeHandlerVtbl = IMFMediaTypeHandlerVtbl {
    QueryInterface: Some(video_stream_typehandler_QueryInterface),
    AddRef: Some(video_stream_typehandler_AddRef),
    Release: Some(video_stream_typehandler_Release),
    IsMediaTypeSupported: Some(video_stream_typehandler_IsMediaTypeSupported),
    GetMediaTypeCount: Some(video_stream_typehandler_GetMediaTypeCount),
    GetMediaTypeByIndex: Some(video_stream_typehandler_GetMediaTypeByIndex),
    SetCurrentMediaType: Some(video_stream_typehandler_SetCurrentMediaType),
    GetCurrentMediaType: Some(video_stream_typehandler_GetCurrentMediaType),
    GetMajorType: Some(video_stream_typehandler_GetMajorType),
};

/// evr.c:698 `static const IMFGetServiceVtbl video_stream_get_service_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_stream_get_service_vtbl: IMFGetServiceVtbl = IMFGetServiceVtbl {
    QueryInterface: Some(video_stream_get_service_QueryInterface),
    AddRef: Some(video_stream_get_service_AddRef),
    Release: Some(video_stream_get_service_Release),
    GetService: Some(video_stream_get_service_GetService),
};

/// evr.c:1005 `static const IMFAttributesVtbl video_stream_attributes_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_stream_attributes_vtbl: IMFAttributesVtbl = IMFAttributesVtbl {
    QueryInterface: Some(video_stream_attributes_QueryInterface),
    AddRef: Some(video_stream_attributes_AddRef),
    Release: Some(video_stream_attributes_Release),
    GetItem: Some(video_stream_attributes_GetItem),
    GetItemType: Some(video_stream_attributes_GetItemType),
    CompareItem: Some(video_stream_attributes_CompareItem),
    Compare: Some(video_stream_attributes_Compare),
    GetUINT32: Some(video_stream_attributes_GetUINT32),
    GetUINT64: Some(video_stream_attributes_GetUINT64),
    GetDouble: Some(video_stream_attributes_GetDouble),
    GetGUID: Some(video_stream_attributes_GetGUID),
    GetStringLength: Some(video_stream_attributes_GetStringLength),
    GetString: Some(video_stream_attributes_GetString),
    GetAllocatedString: Some(video_stream_attributes_GetAllocatedString),
    GetBlobSize: Some(video_stream_attributes_GetBlobSize),
    GetBlob: Some(video_stream_attributes_GetBlob),
    GetAllocatedBlob: Some(video_stream_attributes_GetAllocatedBlob),
    GetUnknown: Some(video_stream_attributes_GetUnknown),
    SetItem: Some(video_stream_attributes_SetItem),
    DeleteItem: Some(video_stream_attributes_DeleteItem),
    DeleteAllItems: Some(video_stream_attributes_DeleteAllItems),
    SetUINT32: Some(video_stream_attributes_SetUINT32),
    SetUINT64: Some(video_stream_attributes_SetUINT64),
    SetDouble: Some(video_stream_attributes_SetDouble),
    SetGUID: Some(video_stream_attributes_SetGUID),
    SetString: Some(video_stream_attributes_SetString),
    SetBlob: Some(video_stream_attributes_SetBlob),
    SetUnknown: Some(video_stream_attributes_SetUnknown),
    LockStore: Some(video_stream_attributes_LockStore),
    UnlockStore: Some(video_stream_attributes_UnlockStore),
    GetCount: Some(video_stream_attributes_GetCount),
    GetItemByIndex: Some(video_stream_attributes_GetItemByIndex),
    CopyAllItems: Some(video_stream_attributes_CopyAllItems),
};

/// evr.c:1480 `static const IMFMediaSinkVtbl video_renderer_sink_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_renderer_sink_vtbl: IMFMediaSinkVtbl = IMFMediaSinkVtbl {
    QueryInterface: Some(video_renderer_sink_QueryInterface),
    AddRef: Some(video_renderer_sink_AddRef),
    Release: Some(video_renderer_sink_Release),
    GetCharacteristics: Some(video_renderer_sink_GetCharacteristics),
    AddStreamSink: Some(video_renderer_sink_AddStreamSink),
    RemoveStreamSink: Some(video_renderer_sink_RemoveStreamSink),
    GetStreamSinkCount: Some(video_renderer_sink_GetStreamSinkCount),
    GetStreamSinkByIndex: Some(video_renderer_sink_GetStreamSinkByIndex),
    GetStreamSinkById: Some(video_renderer_sink_GetStreamSinkById),
    SetPresentationClock: Some(video_renderer_sink_SetPresentationClock),
    GetPresentationClock: Some(video_renderer_sink_GetPresentationClock),
    Shutdown: Some(video_renderer_sink_Shutdown),
};

/// evr.c:1546 `static const IMFMediaSinkPrerollVtbl video_renderer_preroll_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_renderer_preroll_vtbl: IMFMediaSinkPrerollVtbl = IMFMediaSinkPrerollVtbl {
    QueryInterface: Some(video_renderer_preroll_QueryInterface),
    AddRef: Some(video_renderer_preroll_AddRef),
    Release: Some(video_renderer_preroll_Release),
    NotifyPreroll: Some(video_renderer_preroll_NotifyPreroll),
};

/// evr.c:1818 `static const IMFVideoRendererVtbl video_renderer_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_renderer_vtbl: IMFVideoRendererVtbl = IMFVideoRendererVtbl {
    QueryInterface: Some(video_renderer_QueryInterface),
    AddRef: Some(video_renderer_AddRef),
    Release: Some(video_renderer_Release),
    InitializeRenderer: Some(video_renderer_InitializeRenderer),
};

/// evr.c:1883 `static const IMFMediaEventGeneratorVtbl video_renderer_events_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_renderer_events_vtbl: IMFMediaEventGeneratorVtbl = IMFMediaEventGeneratorVtbl {
    QueryInterface: Some(video_renderer_events_QueryInterface),
    AddRef: Some(video_renderer_events_AddRef),
    Release: Some(video_renderer_events_Release),
    GetEvent: Some(video_renderer_events_GetEvent),
    BeginGetEvent: Some(video_renderer_events_BeginGetEvent),
    EndGetEvent: Some(video_renderer_events_EndGetEvent),
    QueueEvent: Some(video_renderer_events_QueueEvent),
};

/// evr.c:2066 `static const IMFClockStateSinkVtbl video_renderer_clock_sink_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_renderer_clock_sink_vtbl: IMFClockStateSinkVtbl = IMFClockStateSinkVtbl {
    QueryInterface: Some(video_renderer_clock_sink_QueryInterface),
    AddRef: Some(video_renderer_clock_sink_AddRef),
    Release: Some(video_renderer_clock_sink_Release),
    OnClockStart: Some(video_renderer_clock_sink_OnClockStart),
    OnClockStop: Some(video_renderer_clock_sink_OnClockStop),
    OnClockPause: Some(video_renderer_clock_sink_OnClockPause),
    OnClockRestart: Some(video_renderer_clock_sink_OnClockRestart),
    OnClockSetRate: Some(video_renderer_clock_sink_OnClockSetRate),
};

/// evr.c:2139 `static const IMFGetServiceVtbl video_renderer_get_service_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_renderer_get_service_vtbl: IMFGetServiceVtbl = IMFGetServiceVtbl {
    QueryInterface: Some(video_renderer_get_service_QueryInterface),
    AddRef: Some(video_renderer_get_service_AddRef),
    Release: Some(video_renderer_get_service_Release),
    GetService: Some(video_renderer_get_service_GetService),
};

/// evr.c:2234 `static const IMFTopologyServiceLookupVtbl video_renderer_service_lookup_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_renderer_service_lookup_vtbl: IMFTopologyServiceLookupVtbl = IMFTopologyServiceLookupVtbl {
    QueryInterface: Some(video_renderer_service_lookup_QueryInterface),
    AddRef: Some(video_renderer_service_lookup_AddRef),
    Release: Some(video_renderer_service_lookup_Release),
    LookupService: Some(video_renderer_service_lookup_LookupService),
};

/// evr.c:2324 `static const IMediaEventSinkVtbl media_event_sink_vtbl`  (slot order: rust:types_amd64.rs)
pub static media_event_sink_vtbl: IMediaEventSinkVtbl = IMediaEventSinkVtbl {
    QueryInterface: Some(video_renderer_event_sink_QueryInterface),
    AddRef: Some(video_renderer_event_sink_AddRef),
    Release: Some(video_renderer_event_sink_Release),
    Notify: Some(video_renderer_event_sink_Notify),
};

/// evr.c:2631 `static const IMFAttributesVtbl video_renderer_attributes_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_renderer_attributes_vtbl: IMFAttributesVtbl = IMFAttributesVtbl {
    QueryInterface: Some(video_renderer_attributes_QueryInterface),
    AddRef: Some(video_renderer_attributes_AddRef),
    Release: Some(video_renderer_attributes_Release),
    GetItem: Some(video_renderer_attributes_GetItem),
    GetItemType: Some(video_renderer_attributes_GetItemType),
    CompareItem: Some(video_renderer_attributes_CompareItem),
    Compare: Some(video_renderer_attributes_Compare),
    GetUINT32: Some(video_renderer_attributes_GetUINT32),
    GetUINT64: Some(video_renderer_attributes_GetUINT64),
    GetDouble: Some(video_renderer_attributes_GetDouble),
    GetGUID: Some(video_renderer_attributes_GetGUID),
    GetStringLength: Some(video_renderer_attributes_GetStringLength),
    GetString: Some(video_renderer_attributes_GetString),
    GetAllocatedString: Some(video_renderer_attributes_GetAllocatedString),
    GetBlobSize: Some(video_renderer_attributes_GetBlobSize),
    GetBlob: Some(video_renderer_attributes_GetBlob),
    GetAllocatedBlob: Some(video_renderer_attributes_GetAllocatedBlob),
    GetUnknown: Some(video_renderer_attributes_GetUnknown),
    SetItem: Some(video_renderer_attributes_SetItem),
    DeleteItem: Some(video_renderer_attributes_DeleteItem),
    DeleteAllItems: Some(video_renderer_attributes_DeleteAllItems),
    SetUINT32: Some(video_renderer_attributes_SetUINT32),
    SetUINT64: Some(video_renderer_attributes_SetUINT64),
    SetDouble: Some(video_renderer_attributes_SetDouble),
    SetGUID: Some(video_renderer_attributes_SetGUID),
    SetString: Some(video_renderer_attributes_SetString),
    SetBlob: Some(video_renderer_attributes_SetBlob),
    SetUnknown: Some(video_renderer_attributes_SetUnknown),
    LockStore: Some(video_renderer_attributes_LockStore),
    UnlockStore: Some(video_renderer_attributes_UnlockStore),
    GetCount: Some(video_renderer_attributes_GetCount),
    GetItemByIndex: Some(video_renderer_attributes_GetItemByIndex),
    CopyAllItems: Some(video_renderer_attributes_CopyAllItems),
};

/// evr.c:2725 `static const IMFQualityAdviseVtbl video_renderer_quality_advise_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_renderer_quality_advise_vtbl: IMFQualityAdviseVtbl = IMFQualityAdviseVtbl {
    QueryInterface: Some(video_renderer_quality_advise_QueryInterface),
    AddRef: Some(video_renderer_quality_advise_AddRef),
    Release: Some(video_renderer_quality_Release),
    SetDropMode: Some(video_renderer_quality_advise_SetDropMode),
    SetQualityLevel: Some(video_renderer_quality_advise_SetQualityLevel),
    GetDropMode: Some(video_renderer_quality_advise_GetDropMode),
    GetQualityLevel: Some(video_renderer_quality_advise_GetQualityLevel),
    DropTime: Some(video_renderer_quality_advise_DropTime),
};

/// evr.c:2836 `static const IMFRateSupportVtbl video_renderer_rate_support_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_renderer_rate_support_vtbl: IMFRateSupportVtbl = IMFRateSupportVtbl {
    QueryInterface: Some(video_renderer_rate_support_QueryInterface),
    AddRef: Some(video_renderer_rate_support_AddRef),
    Release: Some(video_renderer_rate_support_Release),
    GetSlowestRate: Some(video_renderer_rate_support_GetSlowestRate),
    GetFastestRate: Some(video_renderer_rate_support_GetFastestRate),
    IsRateSupported: Some(video_renderer_rate_support_IsRateSupported),
};
