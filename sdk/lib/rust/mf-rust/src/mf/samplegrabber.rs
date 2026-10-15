use crate::mf::main::create_activation_object;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)

static mut sample_grabber_activate_funcs: activate_funcs = unsafe {
    activate_funcs {
        create_object: Some(
            sample_grabber_create_object
                as unsafe extern "C" fn(
                    *mut IMFAttributes,
                    *mut ::core::ffi::c_void,
                    *mut *mut IUnknown,
                ) -> HRESULT,
        ),
        shutdown_object: Some(
            sample_grabber_shutdown_object
                as unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut IUnknown) -> (),
        ),
        free_private: Some(
            sample_grabber_free_private as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
        ),
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(mfplat) - debug channel macro, no declaration emitted.
// #define COBJMACROS - no value, no declaration emitted.

pub const SINK_STATE_STOPPED: i32 = 0;
pub const SINK_STATE_PAUSED: i32 = 1;
pub const SINK_STATE_RUNNING: i32 = 2;

pub const ITEM_TYPE_SAMPLE: i32 = 0;
pub const ITEM_TYPE_MARKER: i32 = 1;

pub const MAX_SAMPLE_QUEUE_LENGTH: u32 = 4;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct scheduled_item_marker {
    pub type_: MFSTREAMSINK_MARKER_TYPE,
    pub context: PROPVARIANT,
}
pub type Pscheduled_item_marker = *mut scheduled_item_marker;

#[repr(C)]
pub union scheduled_item_u {
    pub sample: *mut IMFSample,
    pub marker: scheduled_item_marker,
}

#[repr(C)]
pub struct scheduled_item {
    pub entry: list,
    pub type_: i32,
    pub u: scheduled_item_u,
}
pub type Pscheduled_item = *mut scheduled_item;

#[repr(C)]
pub struct sample_grabber {
    pub IMFMediaSink_iface: IMFMediaSink,
    pub IMFClockStateSink_iface: IMFClockStateSink,
    pub IMFMediaEventGenerator_iface: IMFMediaEventGenerator,
    pub IMFGetService_iface: IMFGetService,
    pub IMFRateSupport_iface: IMFRateSupport,
    pub IMFStreamSink_iface: IMFStreamSink,
    pub IMFMediaTypeHandler_iface: IMFMediaTypeHandler,
    pub timer_callback: IMFAsyncCallback,
    pub refcount: i32,
    pub callback: *mut IMFSampleGrabberSinkCallback,
    pub callback2: *mut IMFSampleGrabberSinkCallback2,
    pub media_type: *mut IMFMediaType,
    pub current_media_type: *mut IMFMediaType,
    pub is_shut_down: i32,
    pub event_queue: *mut IMFMediaEventQueue,
    pub stream_event_queue: *mut IMFMediaEventQueue,
    pub clock: *mut IMFPresentationClock,
    pub timer: *mut IMFTimer,
    pub sample_attributes: *mut IMFAttributes,
    pub items: list,
    pub cancel_key: *mut IUnknown,
    pub ignore_clock: u32,
    pub sample_time_offset: u64,
    pub rate: f32,
    pub state: i32,
    pub cs: CRITICAL_SECTION,
    pub samples_queued: u32,
    pub pending_sample_deliveries: u32,
}
pub type Psample_grabber = *mut sample_grabber;

#[repr(C)]
pub struct sample_grabber_activate_context {
    pub media_type: *mut IMFMediaType,
    pub callback: *mut IMFSampleGrabberSinkCallback,
    pub shut_down: i32,
}
pub type Psample_grabber_activate_context = *mut sample_grabber_activate_context;

// Function-pointer table. Field names and signatures are placeholders
// (no definition of struct activate_funcs appears in the preamble).
#[allow(unused_variables)]

static mut __RUSTCONV_TRACE_UNUSED: u8 = 0;

unsafe extern "C" fn sample_grabber_get_callback(
    sink: *const sample_grabber,
) -> *mut IMFSampleGrabberSinkCallback {
    return if !(*sink).callback2.is_null() {
        (*sink).callback2 as *mut IMFSampleGrabberSinkCallback
    } else {
        (*sink).callback
    };
}

unsafe extern "C" fn sample_grabber_free_private(user_context: *mut core::ffi::c_void) {
    let context: *mut sample_grabber_activate_context = user_context as *mut sample_grabber_activate_context;
    IMFMediaType_Release((*context).media_type);
    IMFSampleGrabberSinkCallback_Release((*context).callback);
    free(context as *mut core::ffi::c_void);
}

unsafe extern "C" fn impl_from_IMFMediaSink(iface: *mut IMFMediaSink) -> *mut sample_grabber {
    return (iface as *mut u8).sub(core::mem::offset_of!(sample_grabber, IMFMediaSink_iface)) as *mut sample_grabber;
}

unsafe extern "C" fn impl_from_IMFClockStateSink(mut iface: *mut IMFClockStateSink) -> *mut sample_grabber {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(sample_grabber, IMFClockStateSink_iface)) as *mut sample_grabber;
}

unsafe extern "C" fn impl_from_IMFMediaEventGenerator(
    mut iface: *mut IMFMediaEventGenerator,
) -> *mut sample_grabber {
    return (iface as *mut u8).wrapping_sub(
        core::mem::offset_of!(sample_grabber, IMFMediaEventGenerator_iface),
    ) as *mut sample_grabber;
}

unsafe extern "C" fn impl_from_IMFGetService(mut iface: *mut IMFGetService) -> *mut sample_grabber {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(sample_grabber, IMFGetService_iface)) as *mut sample_grabber;
}

unsafe extern "C" fn impl_from_IMFRateSupport(iface: *mut IMFRateSupport) -> *mut sample_grabber {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(sample_grabber, IMFRateSupport_iface)) as *mut sample_grabber;
}

unsafe extern "C" fn impl_from_IMFStreamSink(iface: *mut IMFStreamSink) -> *mut sample_grabber {
    return CONTAINING_RECORD!(iface, sample_grabber, IMFStreamSink_iface) as *mut sample_grabber;
}

unsafe extern "C" fn impl_from_IMFMediaTypeHandler(iface: *mut IMFMediaTypeHandler) -> *mut sample_grabber {
    return CONTAINING_RECORD!(iface, sample_grabber, IMFMediaTypeHandler_iface);
}

unsafe extern "C" fn impl_from_IMFAsyncCallback(iface: *mut IMFAsyncCallback) -> *mut sample_grabber {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(sample_grabber, timer_callback))
        as *mut sample_grabber;
}

unsafe extern "system" fn sample_grabber_stream_QueryInterface(iface: *mut IMFStreamSink, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFStreamSink(iface);

    if IsEqualIID!(riid, &raw const IID_IMFStreamSink) != 0 ||
            IsEqualIID!(riid, &raw const IID_IMFMediaEventGenerator) != 0 ||
            IsEqualIID!(riid, &raw const IID_IUnknown) != 0
    {
        *obj = core::ptr::addr_of_mut!((*grabber).IMFStreamSink_iface) as *mut core::ffi::c_void;
    }
    else if IsEqualIID!(riid, &raw const IID_IMFMediaTypeHandler) != 0
    {
        *obj = core::ptr::addr_of_mut!((*grabber).IMFMediaTypeHandler_iface) as *mut core::ffi::c_void;
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

unsafe extern "system" fn sample_grabber_stream_AddRef(iface: *mut IMFStreamSink) -> ULONG {
    let grabber: *mut sample_grabber = impl_from_IMFStreamSink(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*grabber).IMFMediaSink_iface));
}

unsafe extern "C" fn stream_release_pending_item(grabber: *mut sample_grabber, item: *mut scheduled_item) {
    list_remove(core::ptr::addr_of_mut!((*item).entry));
    match ((*item).type_ as ::core::ffi::c_uint) as i32 {
        ITEM_TYPE_SAMPLE => {
            IMFSample_Release((*item).u.sample);
            (*grabber).samples_queued = (*grabber).samples_queued.wrapping_sub(1);
        }
        ITEM_TYPE_MARKER => {
            PropVariantClear(core::ptr::addr_of_mut!((*item).u.marker.context));
        }
        _ => {}
    }
    free(item as *mut ::core::ffi::c_void);
}

unsafe extern "system" fn sample_grabber_stream_Release(iface: *mut IMFStreamSink) -> ULONG {
    let grabber: *mut sample_grabber = impl_from_IMFStreamSink(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*grabber).IMFMediaSink_iface));
}

unsafe extern "system" fn sample_grabber_stream_GetEvent(iface: *mut IMFStreamSink, flags: DWORD, event: *mut *mut IMFMediaEvent) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFStreamSink(iface);

    if (*grabber).is_shut_down != 0 {
        return MF_E_STREAMSINK_REMOVED;
    }

    return IMFMediaEventQueue_GetEvent((*grabber).stream_event_queue, flags, event);
}

unsafe extern "system" fn sample_grabber_stream_BeginGetEvent(
    iface: *mut IMFStreamSink,
    callback: *mut IMFAsyncCallback,
    state: *mut IUnknown,
) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFStreamSink(iface);

    if (*grabber).is_shut_down != 0 {
        return MF_E_STREAMSINK_REMOVED;
    }

    return IMFMediaEventQueue_BeginGetEvent((*grabber).stream_event_queue, callback, state);
}

unsafe extern "system" fn sample_grabber_stream_EndGetEvent(
    iface: *mut IMFStreamSink,
    result: *mut IMFAsyncResult,
    event: *mut *mut IMFMediaEvent,
) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFStreamSink(iface);

    if (*grabber).is_shut_down != 0 {
        return MF_E_STREAMSINK_REMOVED;
    }

    return IMFMediaEventQueue_EndGetEvent((*grabber).stream_event_queue, result, event);
}

unsafe extern "system" fn sample_grabber_stream_QueueEvent(iface: *mut IMFStreamSink, event_type: MediaEventType, ext_type: *const GUID, hr: HRESULT, value: *const PROPVARIANT) -> HRESULT {
    let mut grabber: *mut sample_grabber = impl_from_IMFStreamSink(iface);

    if (*grabber).is_shut_down != 0 {
        return MF_E_STREAMSINK_REMOVED;
    }
    return IMFMediaEventQueue_QueueEventParamVar((*grabber).stream_event_queue, event_type, ext_type, hr, value);
}

unsafe extern "system" fn sample_grabber_stream_GetMediaSink(iface: *mut IMFStreamSink, sink: *mut *mut IMFMediaSink) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFStreamSink(iface);

    if (*grabber).is_shut_down != 0 {
        return MF_E_STREAMSINK_REMOVED;
    }

    *sink = core::ptr::addr_of_mut!((*grabber).IMFMediaSink_iface);
    IMFMediaSink_AddRef(*sink);

    return S_OK;
}

unsafe extern "system" fn sample_grabber_stream_GetIdentifier(iface: *mut IMFStreamSink, identifier: *mut DWORD) -> HRESULT {
    let mut grabber: *mut sample_grabber = impl_from_IMFStreamSink(iface);
    if (*grabber).is_shut_down != 0 {
        return MF_E_STREAMSINK_REMOVED;
    }
    *identifier = 0;
    return S_OK;
}

unsafe extern "system" fn sample_grabber_stream_GetMediaTypeHandler(
    iface: *mut IMFStreamSink,
    handler: *mut *mut IMFMediaTypeHandler,
) -> HRESULT {
    let mut grabber: *mut sample_grabber = impl_from_IMFStreamSink(iface);
    TRACE!("%p, %p.\n", iface, handler);
    if handler.is_null() {
        return E_POINTER;
    }
    if (*grabber).is_shut_down != 0 {
        return MF_E_STREAMSINK_REMOVED;
    }
    *handler = core::ptr::addr_of_mut!((*grabber).IMFMediaTypeHandler_iface);
    IMFMediaTypeHandler_AddRef(*handler);
    return S_OK;
}

unsafe extern "C" fn sample_grabber_report_sample(grabber: *mut sample_grabber, sample: *mut IMFSample, sample_delivered: *mut BOOL) -> HRESULT {
    let mut sample_time: LONGLONG = 0;
    let mut sample_duration: LONGLONG = 0;
    let mut buffer: *mut IMFMediaBuffer = core::ptr::null_mut::<IMFMediaBuffer>();
    let mut flags: DWORD = 0;
    let mut size: DWORD = 0;
    let mut major_type: GUID = GUID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    };
    let mut data: *mut BYTE = core::ptr::null_mut::<BYTE>();
    let mut hr: HRESULT = 0;

    *sample_delivered = (FALSE as i32);

    hr = IMFMediaType_GetMajorType((*grabber).media_type, &raw mut major_type);

    if hr >= 0 {
        hr = IMFSample_GetSampleTime(sample, &raw mut sample_time);
    }

    if IMFSample_GetSampleDuration(sample, &raw mut sample_duration) < 0 {
        sample_duration = 0;
    }

    if hr >= 0 {
        hr = IMFSample_GetSampleFlags(sample, &raw mut flags);
    }

    if hr >= 0 {
        if IMFSample_ConvertToContiguousBuffer(sample, &raw mut buffer) < 0 {
            return E_UNEXPECTED;
        }

        hr = IMFMediaBuffer_Lock(buffer, &raw mut data, core::ptr::null_mut::<DWORD>(), &raw mut size);
        if hr >= 0 {
            *sample_delivered = (TRUE as i32);

            if !(*grabber).callback2.is_null() {
                hr = IMFSample_CopyAllItems(sample, (*grabber).sample_attributes);
                if hr >= 0 {
                    hr = IMFSampleGrabberSinkCallback2_OnProcessSampleEx(
                        (*grabber).callback2,
                        &raw mut major_type,
                        flags,
                        sample_time,
                        sample_duration,
                        data,
                        size,
                        (*grabber).sample_attributes,
                    );
                }
            } else {
                hr = IMFSampleGrabberSinkCallback_OnProcessSample(
                    (*grabber).callback,
                    &raw mut major_type,
                    flags,
                    sample_time,
                    sample_duration,
                    data,
                    size,
                );
            }
            IMFMediaBuffer_Unlock(buffer);
        }

        IMFMediaBuffer_Release(buffer);
    }

    return hr;
}

unsafe extern "C" fn stream_schedule_sample(
    grabber: *mut sample_grabber,
    item: *mut scheduled_item,
) -> HRESULT {
    let mut sampletime: LONGLONG = 0;
    let mut hr: HRESULT = 0;

    if (*grabber).is_shut_down != 0 {
        return MF_E_STREAMSINK_REMOVED;
    }

    hr = IMFSample_GetSampleTime((*item).u.sample, core::ptr::addr_of_mut!(sampletime));
    if hr < 0 {
        return hr;
    }

    if !(*grabber).cancel_key.is_null() {
        IUnknown_Release((*grabber).cancel_key);
        (*grabber).cancel_key = core::ptr::null_mut();
    }

    hr = IMFTimer_SetTimer(
        (*grabber).timer,
        0,
        (sampletime as UINT64).wrapping_sub((*grabber).sample_time_offset) as LONGLONG,
        core::ptr::addr_of_mut!((*grabber).timer_callback),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*grabber).cancel_key),
    );
    if hr < 0 {
        (*grabber).cancel_key = core::ptr::null_mut();
    }

    hr
}

unsafe extern "C" fn stream_queue_sample(
    grabber: *mut sample_grabber,
    sample: *mut IMFSample,
) -> HRESULT {
    let mut item: *mut scheduled_item = core::ptr::null_mut();
    let mut sampletime: LONGLONG = 0;
    let mut hr: HRESULT = 0;

    hr = IMFSample_GetSampleTime(sample, core::ptr::addr_of_mut!(sampletime));
    if hr < 0 {
        return hr;
    }

    item = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<scheduled_item>() as size_t) as _,
    ) as *mut scheduled_item;
    if item.is_null() {
        return E_OUTOFMEMORY;
    }

    (*item).type_ = ITEM_TYPE_SAMPLE;
    (*item).u.sample = sample;
    IMFSample_AddRef((*item).u.sample);
    list_init(core::ptr::addr_of_mut!((*item).entry));
    let fresh15 = (*grabber).samples_queued;
    (*grabber).samples_queued = (*grabber).samples_queued.wrapping_add(1);
    if fresh15 == 0 {
        hr = stream_schedule_sample(grabber, item);
    }

    if hr >= 0 {
        list_add_tail(
            core::ptr::addr_of_mut!((*grabber).items),
            core::ptr::addr_of_mut!((*item).entry),
        );
    } else {
        stream_release_pending_item(grabber, item);
    }

    return hr;
}

unsafe extern "C" fn sample_grabber_stream_request_sample(grabber: *mut sample_grabber) {
    IMFStreamSink_QueueEvent(
        core::ptr::addr_of_mut!((*grabber).IMFStreamSink_iface),
        MEStreamSinkRequestSample,
        &GUID_NULL,
        S_OK,
        core::ptr::null_mut(),
    );
    (*grabber).pending_sample_deliveries = (*grabber).pending_sample_deliveries.wrapping_add(1);
}
unsafe extern "system" fn sample_grabber_stream_ProcessSample(
    iface: *mut IMFStreamSink,
    sample: *mut IMFSample,
) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFStreamSink(iface);
    let mut sample_delivered: BOOL = (FALSE as i32);
    let mut sampletime: LONGLONG = 0;
    let mut hr: HRESULT = S_OK;

    TRACE!("%p, %p.\n", iface, sample);

    if sample.is_null() {
        return S_OK;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*grabber).cs));

    if (*grabber).is_shut_down != 0 {
        hr = MF_E_STREAMSINK_REMOVED;
    } else if (*grabber).state as ::core::ffi::c_uint
        != SINK_STATE_STOPPED as ::core::ffi::c_uint
    {
        if (*grabber).pending_sample_deliveries != 0 {
            (*grabber).pending_sample_deliveries =
                (*grabber).pending_sample_deliveries.wrapping_sub(1);
        }

        hr = IMFSample_GetSampleTime(sample, core::ptr::addr_of_mut!(sampletime));

        if hr >= 0 {
            if (*grabber).ignore_clock != 0 {
                hr = sample_grabber_report_sample(
                    grabber,
                    sample,
                    core::ptr::addr_of_mut!(sample_delivered),
                );
                if sample_delivered != 0 {
                    sample_grabber_stream_request_sample(grabber);
                }
            } else {
                hr = stream_queue_sample(grabber, sample);
            }
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*grabber).cs));

    return hr;
}

unsafe extern "C" fn sample_grabber_stream_report_marker(
    grabber: *mut sample_grabber,
    context: *const PROPVARIANT,
    hr: HRESULT,
) {
    IMFStreamSink_QueueEvent(
        core::ptr::addr_of_mut!((*grabber).IMFStreamSink_iface),
        MEStreamSinkMarker as i32 as MediaEventType,
        core::ptr::addr_of!(GUID_NULL),
        hr,
        context,
    );
}

unsafe extern "C" fn stream_place_marker(
    grabber: *mut sample_grabber,
    marker_type: MFSTREAMSINK_MARKER_TYPE,
    context_value: *const PROPVARIANT,
) -> HRESULT {
    let mut item: *mut scheduled_item = core::ptr::null_mut();
    let mut hr: HRESULT = S_OK;

    if list_empty(core::ptr::addr_of_mut!((*grabber).items)) != 0 {
        sample_grabber_stream_report_marker(grabber, context_value, S_OK);
        return S_OK;
    }

    item = calloc(
        (1 as usize) as _,
        (core::mem::size_of::<scheduled_item>()) as _,
    ) as *mut scheduled_item;
    if item.is_null() {
        return E_OUTOFMEMORY;
    }

    (*item).type_ = ITEM_TYPE_MARKER;
    (*item).u.marker.type_ = marker_type;
    list_init(core::ptr::addr_of_mut!((*item).entry));
    PropVariantInit!(core::ptr::addr_of_mut!((*item).u.marker.context));
    if !context_value.is_null() {
        hr = PropVariantCopy(
            core::ptr::addr_of_mut!((*item).u.marker.context),
            context_value,
        );
    }
    if hr >= 0 {
        list_add_tail(
            core::ptr::addr_of_mut!((*grabber).items),
            core::ptr::addr_of_mut!((*item).entry),
        );
    } else {
        stream_release_pending_item(grabber, item);
    }

    hr
}

unsafe extern "system" fn sample_grabber_stream_PlaceMarker(
    iface: *mut IMFStreamSink,
    marker_type: MFSTREAMSINK_MARKER_TYPE,
    marker_value: *const PROPVARIANT,
    context_value: *const PROPVARIANT,
) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFStreamSink(iface);
    let mut hr: HRESULT = S_OK;
    if (*__wine_dbch___default).flags as u32
        & (1u32) << __WINE_DBCL_TRACE as u32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/samplegrabber.c\0"
                as *const u8 as *const i8,
            b"sample_grabber_stream_PlaceMarker\0" as *const u8 as *const i8,
            503i32,
            b"%p, %d, %p, %p.\n\0" as *const u8 as *const i8,
            iface,
            marker_type as u32,
            marker_value,
            context_value,
        );
    }
    EnterCriticalSection(core::ptr::addr_of_mut!((*grabber).cs));
    if (*grabber).is_shut_down != 0 {
        hr = MF_E_STREAMSINK_REMOVED;
    } else if (*grabber).state as u32 == (SINK_STATE_RUNNING as u32) {
        hr = stream_place_marker(grabber, marker_type, context_value);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*grabber).cs));
    return hr;
}

unsafe extern "system" fn sample_grabber_stream_Flush(iface: *mut IMFStreamSink) -> HRESULT {
    let mut grabber: *mut sample_grabber = impl_from_IMFStreamSink(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*grabber).cs));

    if (*grabber).is_shut_down != 0 {
        hr = MF_E_STREAMSINK_REMOVED;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*grabber).cs));

    return hr;
}
unsafe extern "system" fn sample_grabber_stream_type_handler_QueryInterface(
    iface: *mut IMFMediaTypeHandler,
    riid: *const GUID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFMediaTypeHandler(iface);
    return IMFStreamSink_QueryInterface(core::ptr::addr_of_mut!((*grabber).IMFStreamSink_iface), riid, obj);
}

unsafe extern "system" fn sample_grabber_stream_type_handler_AddRef(
    iface: *mut IMFMediaTypeHandler,
) -> ULONG {
    let grabber: *mut sample_grabber = impl_from_IMFMediaTypeHandler(iface);
    return IMFStreamSink_AddRef(core::ptr::addr_of_mut!((*grabber).IMFStreamSink_iface));
}

unsafe extern "system" fn sample_grabber_stream_type_handler_Release(iface: *mut IMFMediaTypeHandler) -> ULONG {
    let grabber: *mut sample_grabber = impl_from_IMFMediaTypeHandler(iface);
    return IMFStreamSink_Release(core::ptr::addr_of_mut!((*grabber).IMFStreamSink_iface));
}

unsafe extern "C" fn sample_grabber_stream_is_media_type_supported(grabber: *mut sample_grabber, in_type: *mut IMFMediaType) -> HRESULT {
    let supported_flags: DWORD = (MF_MEDIATYPE_EQUAL_MAJOR_TYPES | MF_MEDIATYPE_EQUAL_FORMAT_TYPES | MF_MEDIATYPE_EQUAL_FORMAT_DATA) as DWORD;
    let mut flags: DWORD = 0;

    if (*grabber).is_shut_down != 0 {
        return MF_E_STREAMSINK_REMOVED;
    }

    if in_type.is_null() {
        return E_POINTER;
    }

    if IMFMediaType_IsEqual((*grabber).media_type, in_type, core::ptr::addr_of_mut!(flags)) == S_OK {
        return S_OK;
    }

    return if (flags & supported_flags) == supported_flags { S_OK } else { MF_E_INVALIDMEDIATYPE };
}

unsafe extern "system" fn sample_grabber_stream_type_handler_IsMediaTypeSupported(iface: *mut IMFMediaTypeHandler, in_type: *mut IMFMediaType, out_type: *mut *mut IMFMediaType) -> HRESULT {
    let mut grabber: *mut sample_grabber = impl_from_IMFMediaTypeHandler(iface);

    return sample_grabber_stream_is_media_type_supported(grabber, in_type);
}

unsafe extern "system" fn sample_grabber_stream_type_handler_GetMediaTypeCount(iface: *mut IMFMediaTypeHandler, count: *mut DWORD) -> HRESULT {
    if count.is_null() {
        return E_POINTER;
    }

    *count = 0;

    return S_OK;
}

unsafe extern "system" fn sample_grabber_stream_type_handler_GetMediaTypeByIndex(
    iface: *mut IMFMediaTypeHandler,
    index: DWORD,
    media_type: *mut *mut IMFMediaType,
) -> HRESULT {
    if media_type.is_null() {
        return E_POINTER;
    }

    return MF_E_NO_MORE_TYPES;
}

unsafe extern "system" fn sample_grabber_stream_type_handler_SetCurrentMediaType(
    iface: *mut IMFMediaTypeHandler,
    media_type: *mut IMFMediaType,
) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFMediaTypeHandler(iface);
    let mut hr: HRESULT;

    hr = sample_grabber_stream_is_media_type_supported(grabber, media_type);
    if hr < 0 {
        return hr;
    }

    IMFMediaType_Release((*grabber).current_media_type);
    (*grabber).current_media_type = media_type;
    IMFMediaType_AddRef((*grabber).current_media_type);

    return S_OK;
}

unsafe extern "system" fn sample_grabber_stream_type_handler_GetCurrentMediaType(
    iface: *mut IMFMediaTypeHandler,
    media_type: *mut *mut IMFMediaType,
) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFMediaTypeHandler(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!("%p, %p.\n", iface, media_type);

    if media_type.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*grabber).cs));

    if (*grabber).is_shut_down != 0 {
        hr = MF_E_STREAMSINK_REMOVED;
    } else {
        *media_type = (*grabber).current_media_type;
        IMFMediaType_AddRef(*media_type);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*grabber).cs));

    return hr;
}

unsafe extern "system" fn sample_grabber_stream_type_handler_GetMajorType(iface: *mut IMFMediaTypeHandler, type_: *mut GUID) -> HRESULT {
    let mut grabber: *mut sample_grabber = impl_from_IMFMediaTypeHandler(iface);
    let mut hr: HRESULT;

    if type_.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*grabber).cs));

    if (*grabber).is_shut_down != 0 {
        hr = MF_E_STREAMSINK_REMOVED;
    } else {
        hr = IMFMediaType_GetMajorType((*grabber).current_media_type, type_);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*grabber).cs));

    return hr;
}

unsafe extern "system" fn sample_grabber_stream_timer_callback_QueryInterface(iface: *mut IMFAsyncCallback, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAsyncCallback)) != 0 || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0 {
        *obj = iface as *mut core::ffi::c_void;
        IMFAsyncCallback_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn sample_grabber_stream_timer_callback_AddRef(iface: *mut IMFAsyncCallback) -> ULONG {
    let mut grabber: *mut sample_grabber = impl_from_IMFAsyncCallback(iface);
    return IMFStreamSink_AddRef(core::ptr::addr_of_mut!((*grabber).IMFStreamSink_iface));
}

unsafe extern "system" fn sample_grabber_stream_timer_callback_Release(iface: *mut IMFAsyncCallback) -> ULONG {
    let mut grabber: *mut sample_grabber = impl_from_IMFAsyncCallback(iface);
    return IMFStreamSink_Release(core::ptr::addr_of_mut!((*grabber).IMFStreamSink_iface));
}

unsafe extern "system" fn sample_grabber_stream_timer_callback_GetParameters(iface: *mut IMFAsyncCallback, flags: *mut DWORD, queue: *mut DWORD) -> HRESULT {
    *flags = 0 as DWORD;
    *queue = MFASYNC_CALLBACK_QUEUE_MULTITHREADED as DWORD;
    return S_OK;
}
unsafe extern "system" fn sample_grabber_stream_timer_callback_Invoke(
    mut iface: *mut IMFAsyncCallback,
    mut result: *mut IMFAsyncResult,
) -> HRESULT {
    let mut grabber: *mut sample_grabber = impl_from_IMFAsyncCallback(iface);
    let mut sample_reported: BOOL = (FALSE as i32);
    let mut sample_delivered: BOOL = (FALSE as i32);
    let mut item: *mut scheduled_item = ::core::ptr::null_mut::<scheduled_item>();
    let mut item2: *mut scheduled_item = ::core::ptr::null_mut::<scheduled_item>();
    let mut hr: HRESULT = 0;

    EnterCriticalSection(&raw mut (*grabber).cs);

    item = ((*grabber).items.next as *mut ::core::ffi::c_char).offset(
        -(&raw mut (*::core::ptr::null_mut::<scheduled_item>()).entry as ::core::ffi::c_ulonglong
            as isize),
    ) as *mut scheduled_item;
    item2 = ((*item).entry.next as *mut ::core::ffi::c_char).offset(
        -(&raw mut (*::core::ptr::null_mut::<scheduled_item>()).entry as ::core::ffi::c_ulonglong
            as isize),
    ) as *mut scheduled_item;
    while &raw mut (*item).entry != &raw mut (*grabber).items {
        if (*item).type_ as ::core::ffi::c_uint
            == ITEM_TYPE_MARKER as ::core::ffi::c_uint
        {
            sample_grabber_stream_report_marker(grabber, &raw mut (*item).u.marker.context, S_OK);
            stream_release_pending_item(grabber, item);
        } else if (*item).type_ as ::core::ffi::c_uint
            == ITEM_TYPE_SAMPLE as ::core::ffi::c_uint
        {
            if sample_reported == 0 {
                hr = sample_grabber_report_sample(
                    grabber,
                    (*item).u.sample,
                    &raw mut sample_delivered,
                );
                if hr < 0 {
                    WARN!("Failed to report a sample, hr %#lx.\n", hr);
                }
                stream_release_pending_item(grabber, item);
                sample_reported = (TRUE as i32);
                if sample_delivered != 0 {
                    sample_grabber_stream_request_sample(grabber);
                }
            } else {
                hr = stream_schedule_sample(grabber, item);
                if hr < 0 {
                    WARN!("Failed to schedule a sample, hr %#lx.\n", hr);
                }
                break;
            }
        }
        item = item2;
        item2 = ((*item).entry.next as *mut ::core::ffi::c_char).offset(
            -(&raw mut (*::core::ptr::null_mut::<scheduled_item>()).entry
                as ::core::ffi::c_ulonglong as isize),
        ) as *mut scheduled_item;
    }

    LeaveCriticalSection(&raw mut (*grabber).cs);

    return S_OK;
}

unsafe extern "system" fn sample_grabber_sink_QueryInterface(iface: *mut IMFMediaSink, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFMediaSink(iface);

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFMediaSink)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = core::ptr::addr_of_mut!((*grabber).IMFMediaSink_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFClockStateSink)) != 0 {
        *obj = core::ptr::addr_of_mut!((*grabber).IMFClockStateSink_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFMediaEventGenerator)) != 0 {
        *obj = core::ptr::addr_of_mut!((*grabber).IMFMediaEventGenerator_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFGetService)) != 0 {
        *obj = core::ptr::addr_of_mut!((*grabber).IMFGetService_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFRateSupport)) != 0 {
        *obj = core::ptr::addr_of_mut!((*grabber).IMFRateSupport_iface) as *mut core::ffi::c_void;
    } else {
        WARN!(b"Unsupported %s.\n\0".as_ptr() as *const i8, debugstr_guid(riid));
        *obj = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*obj as *mut IUnknown);

    return S_OK;
}

unsafe extern "system" fn sample_grabber_sink_AddRef(iface: *mut IMFMediaSink) -> ULONG {
    let grabber: *mut sample_grabber = impl_from_IMFMediaSink(iface);
    let refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*grabber).refcount) as *mut i32,
    ) as ULONG;

    TRACE!("%p, refcount %lu.\n", iface, refcount);

    return refcount;
}

unsafe extern "C" fn sample_grabber_release_pending_items(grabber: *mut sample_grabber) {
    let mut item: *mut scheduled_item;
    let mut next_item: *mut scheduled_item;
    item = ((*grabber).items.next as *mut u8)
        .sub(core::mem::offset_of!(scheduled_item, entry)) as *mut scheduled_item;
    next_item = ((*item).entry.next as *mut u8)
        .sub(core::mem::offset_of!(scheduled_item, entry)) as *mut scheduled_item;
    while core::ptr::addr_of_mut!((*item).entry) != core::ptr::addr_of_mut!((*grabber).items) {
        stream_release_pending_item(grabber, item);
        item = next_item;
        next_item = ((*item).entry.next as *mut u8)
            .sub(core::mem::offset_of!(scheduled_item, entry)) as *mut scheduled_item;
    }
}

unsafe extern "system" fn sample_grabber_sink_Release(iface: *mut IMFMediaSink) -> ULONG {
    let grabber: *mut sample_grabber = impl_from_IMFMediaSink(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*grabber).refcount) as *mut i32) as ULONG;

    if refcount == 0 {
        if !(*grabber).callback.is_null() {
            IMFSampleGrabberSinkCallback_Release((*grabber).callback);
        }
        if !(*grabber).callback2.is_null() {
            IMFSampleGrabberSinkCallback2_Release((*grabber).callback2);
        }
        if !(*grabber).current_media_type.is_null() {
            IMFMediaType_Release((*grabber).current_media_type);
        }
        IMFMediaType_Release((*grabber).media_type);
        if !(*grabber).event_queue.is_null() {
            IMFMediaEventQueue_Release((*grabber).event_queue);
        }
        if !(*grabber).clock.is_null() {
            IMFPresentationClock_Release((*grabber).clock);
        }
        if !(*grabber).timer.is_null() {
            IMFTimer_Release((*grabber).timer);
        }
        if !(*grabber).cancel_key.is_null() {
            IUnknown_Release((*grabber).cancel_key);
        }
        if !(*grabber).stream_event_queue.is_null() {
            IMFMediaEventQueue_Shutdown((*grabber).stream_event_queue);
            IMFMediaEventQueue_Release((*grabber).stream_event_queue);
        }
        if !(*grabber).sample_attributes.is_null() {
            IMFAttributes_Release((*grabber).sample_attributes);
        }
        sample_grabber_release_pending_items(grabber);
        DeleteCriticalSection(core::ptr::addr_of_mut!((*grabber).cs));
        free(grabber as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn sample_grabber_sink_GetCharacteristics(iface: *mut IMFMediaSink, flags: *mut DWORD) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFMediaSink(iface);

    if (*grabber).is_shut_down != 0 {
        return MF_E_SHUTDOWN;
    }
    *flags = MEDIASINK_FIXED_STREAMS as DWORD;
    if (*grabber).ignore_clock != 0 {
        *flags |= MEDIASINK_RATELESS as DWORD;
    }

    return S_OK;
}
unsafe extern "system" fn sample_grabber_sink_AddStreamSink(iface: *mut IMFMediaSink, stream_sink_id: DWORD, media_type: *mut IMFMediaType, stream_sink: *mut *mut IMFStreamSink) -> HRESULT {
    let mut grabber: *mut sample_grabber = impl_from_IMFMediaSink(iface);

    return if (*grabber).is_shut_down != 0 { MF_E_SHUTDOWN } else { MF_E_STREAMSINKS_FIXED };
}

unsafe extern "system" fn sample_grabber_sink_RemoveStreamSink(iface: *mut IMFMediaSink, stream_sink_id: DWORD) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFMediaSink(iface);

    let _ = stream_sink_id;

    return if (*grabber).is_shut_down != 0 { MF_E_SHUTDOWN } else { MF_E_STREAMSINKS_FIXED };
}
unsafe extern "system" fn sample_grabber_sink_GetStreamSinkCount(iface: *mut IMFMediaSink, count: *mut DWORD) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFMediaSink(iface);

    if (*grabber).is_shut_down != 0 {
        return MF_E_SHUTDOWN;
    }

    *count = 1;

    return S_OK;
}

unsafe extern "system" fn sample_grabber_sink_GetStreamSinkByIndex(iface: *mut IMFMediaSink, index: DWORD, stream: *mut *mut IMFStreamSink) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFMediaSink(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*grabber).cs));

    if (*grabber).is_shut_down != 0 {
        hr = MF_E_SHUTDOWN;
    } else if index > 0 {
        hr = MF_E_INVALIDINDEX;
    } else {
        *stream = core::ptr::addr_of_mut!((*grabber).IMFStreamSink_iface);
        IMFStreamSink_AddRef(*stream);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*grabber).cs));

    return hr;
}

unsafe extern "system" fn sample_grabber_sink_GetStreamSinkById(iface: *mut IMFMediaSink, stream_sink_id: DWORD, stream: *mut *mut IMFStreamSink) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFMediaSink(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*grabber).cs));

    if (*grabber).is_shut_down != 0 {
        hr = MF_E_SHUTDOWN;
    } else if stream_sink_id > 0 {
        hr = MF_E_INVALIDSTREAMNUMBER;
    } else {
        *stream = core::ptr::addr_of_mut!((*grabber).IMFStreamSink_iface);
        IMFStreamSink_AddRef(*stream);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*grabber).cs));

    return hr;
}

unsafe extern "C" fn sample_grabber_cancel_timer(grabber: *mut sample_grabber) {
    if !(*grabber).timer.is_null() && !(*grabber).cancel_key.is_null() {
        IMFTimer_CancelTimer((*grabber).timer, (*grabber).cancel_key);
        IUnknown_Release((*grabber).cancel_key);
        (*grabber).cancel_key = core::ptr::null_mut();
    }
}

unsafe extern "C" fn sample_grabber_set_presentation_clock(
    grabber: *mut sample_grabber,
    clock: *mut IMFPresentationClock,
) -> HRESULT {
    let mut hr: HRESULT = 0;

    hr = IMFSampleGrabberSinkCallback_OnSetPresentationClock(sample_grabber_get_callback(grabber), clock);
    if hr < 0 {
        return hr;
    }

    if !(*grabber).clock.is_null() {
        sample_grabber_cancel_timer(grabber);
        IMFPresentationClock_RemoveClockStateSink(
            (*grabber).clock,
            core::ptr::addr_of_mut!((*grabber).IMFClockStateSink_iface),
        );
        IMFPresentationClock_Release((*grabber).clock);
        if !(*grabber).timer.is_null() {
            IMFTimer_Release((*grabber).timer);
            (*grabber).timer = core::ptr::null_mut::<IMFTimer>();
        }
    }

    (*grabber).clock = clock;
    if !(*grabber).clock.is_null() {
        IMFPresentationClock_AddRef((*grabber).clock);
        IMFPresentationClock_AddClockStateSink(
            (*grabber).clock,
            core::ptr::addr_of_mut!((*grabber).IMFClockStateSink_iface),
        );
        if IMFPresentationClock_QueryInterface(
            (*grabber).clock,
            core::ptr::addr_of!(IID_IMFTimer),
            core::ptr::addr_of_mut!((*grabber).timer) as *mut *mut core::ffi::c_void,
        ) < 0
        {
            WARN!("Failed to get IMFTimer interface.\n");
            (*grabber).timer = core::ptr::null_mut::<IMFTimer>();
        }
    }

    return hr;
}

unsafe extern "system" fn sample_grabber_sink_SetPresentationClock(iface: *mut IMFMediaSink, clock: *mut IMFPresentationClock) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFMediaSink(iface);
    let mut hr: HRESULT;

    EnterCriticalSection(core::ptr::addr_of_mut!((*grabber).cs));

    if (*grabber).is_shut_down != 0 {
        hr = MF_E_SHUTDOWN;
    } else {
        hr = sample_grabber_set_presentation_clock(grabber, clock);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*grabber).cs));

    return hr;
}

unsafe extern "system" fn sample_grabber_sink_GetPresentationClock(iface: *mut IMFMediaSink, clock: *mut *mut IMFPresentationClock) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFMediaSink(iface);
    let mut hr: HRESULT = S_OK;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/samplegrabber.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"sample_grabber_sink_GetPresentationClock\0" as *const u8
                as *const ::core::ffi::c_char,
            1055 as ::core::ffi::c_int,
            b"%p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            clock,
        );
    }
    if clock.is_null() {
        return E_POINTER;
    }
    EnterCriticalSection(core::ptr::addr_of_mut!((*grabber).cs));
    if !(*grabber).clock.is_null() {
        *clock = (*grabber).clock;
        IMFPresentationClock_AddRef(*clock);
    } else {
        hr = MF_E_NO_CLOCK;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*grabber).cs));
    return hr;
}

unsafe extern "system" fn sample_grabber_sink_Shutdown(iface: *mut IMFMediaSink) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFMediaSink(iface);
    let mut hr: HRESULT;

    TRACE!("%p.\n", iface);

    EnterCriticalSection(&raw mut (*grabber).cs);

    if (*grabber).is_shut_down != 0 {
        hr = MF_E_SHUTDOWN;
    } else {
        (*grabber).is_shut_down = (TRUE as i32);
        sample_grabber_release_pending_items(grabber);
        hr = sample_grabber_set_presentation_clock(grabber, core::ptr::null_mut::<IMFPresentationClock>());
        if hr >= 0 {
            hr = IMFSampleGrabberSinkCallback_OnShutdown(sample_grabber_get_callback(grabber));
        }
        if hr >= 0 {
            IMFMediaType_Release((*grabber).current_media_type);
            (*grabber).current_media_type = core::ptr::null_mut::<IMFMediaType>();
            IMFMediaEventQueue_Shutdown((*grabber).stream_event_queue);
            IMFMediaEventQueue_Shutdown((*grabber).event_queue);
        }
    }

    LeaveCriticalSection(&raw mut (*grabber).cs);

    return hr;
}

unsafe extern "system" fn sample_grabber_clock_sink_QueryInterface(iface: *mut IMFClockStateSink, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut grabber: *mut sample_grabber = impl_from_IMFClockStateSink(iface);
    return IMFMediaSink_QueryInterface(core::ptr::addr_of_mut!((*grabber).IMFMediaSink_iface), riid, obj);
}

unsafe extern "system" fn sample_grabber_clock_sink_AddRef(iface: *mut IMFClockStateSink) -> ULONG {
    let grabber: *mut sample_grabber = impl_from_IMFClockStateSink(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*grabber).IMFMediaSink_iface));
}

unsafe extern "system" fn sample_grabber_clock_sink_Release(iface: *mut IMFClockStateSink) -> ULONG {
    let mut grabber: *mut sample_grabber = impl_from_IMFClockStateSink(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*grabber).IMFMediaSink_iface));
}

unsafe extern "C" fn sample_grabber_set_state(
    grabber: *mut sample_grabber,
    state: sink_state,
    systime: MFTIME,
    offset: LONGLONG,
) -> HRESULT {
    static mut events: [DWORD; 3] = {
        let mut a = [0 as DWORD; 3];
        a[SINK_STATE_STOPPED as usize] = MEStreamSinkStopped as DWORD;
        a[SINK_STATE_PAUSED as usize] = MEStreamSinkPaused as DWORD;
        a[SINK_STATE_RUNNING as usize] = MEStreamSinkStarted as DWORD;
        a
    };
    let mut do_callback: BOOL = FALSE as BOOL;
    let mut required_requests: byte = 0;
    let mut hr: HRESULT = S_OK as HRESULT;
    let mut i: ::core::ffi::c_uint = 0;

    EnterCriticalSection(&raw mut (*grabber).cs);

    if (*grabber).is_shut_down == 0 {
        if state as ::core::ffi::c_uint == SINK_STATE_PAUSED as ::core::ffi::c_uint
            && (*grabber).state as ::core::ffi::c_uint
                == SINK_STATE_STOPPED as ::core::ffi::c_uint
        {
            hr = MF_E_INVALID_STATE_TRANSITION as HRESULT;
        } else {
            if state as ::core::ffi::c_uint == SINK_STATE_STOPPED as ::core::ffi::c_uint {
                sample_grabber_cancel_timer(grabber);
                sample_grabber_release_pending_items(grabber);
                (*grabber).pending_sample_deliveries = 0 as UINT32;
            } else if state as ::core::ffi::c_uint
                == SINK_STATE_RUNNING as ::core::ffi::c_uint
                && ((*grabber).state as ::core::ffi::c_uint
                    != SINK_STATE_RUNNING as ::core::ffi::c_uint
                    || offset != PRESENTATION_CURRENT_POSITION as LONGLONG)
            {
                if offset != PRESENTATION_CURRENT_POSITION as LONGLONG {
                    sample_grabber_cancel_timer(grabber);
                    sample_grabber_release_pending_items(grabber);
                }

                required_requests = (MAX_SAMPLE_QUEUE_LENGTH as UINT32)
                    .wrapping_sub((*grabber).pending_sample_deliveries)
                    as byte;

                i = 0 as ::core::ffi::c_uint;
                while i < required_requests as ::core::ffi::c_uint {
                    sample_grabber_stream_request_sample(grabber);
                    i = i.wrapping_add(1);
                }
            }

            do_callback = (state as ::core::ffi::c_uint != (*grabber).state as ::core::ffi::c_uint
                || state as ::core::ffi::c_uint
                    != SINK_STATE_PAUSED as ::core::ffi::c_uint)
                as ::core::ffi::c_int as BOOL;
            if do_callback != 0 {
                if (*grabber).rate == 0.0f32
                    && state as ::core::ffi::c_uint
                        == SINK_STATE_RUNNING as ::core::ffi::c_uint
                {
                    IMFStreamSink_QueueEvent(
                        &raw mut (*grabber).IMFStreamSink_iface,
                        MEStreamSinkScrubSampleComplete as ::core::ffi::c_int as MediaEventType,
                        &raw const GUID_NULL,
                        S_OK as HRESULT,
                        ::core::ptr::null::<PROPVARIANT>(),
                    );
                }

                IMFStreamSink_QueueEvent(
                    &raw mut (*grabber).IMFStreamSink_iface,
                    events[state as usize],
                    &raw const GUID_NULL,
                    S_OK as HRESULT,
                    ::core::ptr::null::<PROPVARIANT>(),
                );
            }
            (*grabber).state = (state) as i32;
        }
    }

    LeaveCriticalSection(&raw mut (*grabber).cs);

    if do_callback != 0 {
        match (state as ::core::ffi::c_uint) as i32 {
            SINK_STATE_STOPPED => {
                hr = IMFSampleGrabberSinkCallback_OnClockStop(sample_grabber_get_callback(grabber), systime);
            }
            SINK_STATE_PAUSED => {
                hr = IMFSampleGrabberSinkCallback_OnClockPause(sample_grabber_get_callback(grabber), systime);
            }
            SINK_STATE_RUNNING => {
                hr = IMFSampleGrabberSinkCallback_OnClockStart(
                    sample_grabber_get_callback(grabber),
                    systime,
                    offset,
                );
            }
            _ => {}
        }
    }

    return hr;
}

unsafe extern "system" fn sample_grabber_clock_sink_OnClockStart(iface: *mut IMFClockStateSink, systime: MFTIME, offset: LONGLONG) -> HRESULT {
    let mut grabber: *mut sample_grabber = impl_from_IMFClockStateSink(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/samplegrabber.c\0".as_ptr() as *const i8,
            b"sample_grabber_clock_sink_OnClockStart\0".as_ptr() as *const i8,
            1221i32,
            b"%p, %s, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_time(systime as LONGLONG),
            debugstr_time(offset),
        );
    }
    return sample_grabber_set_state(grabber, (SINK_STATE_RUNNING) as sink_state, systime, offset);
}

unsafe extern "system" fn sample_grabber_clock_sink_OnClockStop(iface: *mut IMFClockStateSink, systime: MFTIME) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFClockStateSink(iface);

    return sample_grabber_set_state(grabber, (SINK_STATE_STOPPED) as sink_state, systime, 0);
}

unsafe extern "system" fn sample_grabber_clock_sink_OnClockPause(iface: *mut IMFClockStateSink, systime: MFTIME) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFClockStateSink(iface);
    TRACE!("%p, %s.\n", iface, debugstr_time(systime as LONGLONG));
    return sample_grabber_set_state(grabber, (SINK_STATE_PAUSED) as sink_state, systime, 0);
}

unsafe extern "system" fn sample_grabber_clock_sink_OnClockRestart(iface: *mut IMFClockStateSink, systime: MFTIME) -> HRESULT {
    let mut grabber: *mut sample_grabber = impl_from_IMFClockStateSink(iface);
    if ((*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/samplegrabber.c\0".as_ptr() as *const i8,
            b"sample_grabber_clock_sink_OnClockRestart\0".as_ptr() as *const i8,
            1248 as i32,
            b"%p, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_time(systime as i64),
        );
    }
    return sample_grabber_set_state(grabber, (SINK_STATE_RUNNING) as sink_state, systime, (PRESENTATION_CURRENT_POSITION) as LONGLONG);
}

unsafe extern "system" fn sample_grabber_clock_sink_OnClockSetRate(iface: *mut IMFClockStateSink, systime: MFTIME, rate: f32) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFClockStateSink(iface);
    let mut hr: HRESULT = S_OK;
    TRACE!("%p, %s, %f.\n", iface, debugstr_time(systime as LONGLONG), rate as f64);
    EnterCriticalSection(core::ptr::addr_of_mut!((*grabber).cs));
    if (*grabber).is_shut_down != 0 {
        hr = MF_E_SHUTDOWN;
    } else {
        IMFStreamSink_QueueEvent(
            core::ptr::addr_of_mut!((*grabber).IMFStreamSink_iface),
            MEStreamSinkRateChanged as MediaEventType,
            core::ptr::addr_of!(GUID_NULL),
            S_OK,
            core::ptr::null::<PROPVARIANT>(),
        );
        (*grabber).rate = rate;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*grabber).cs));
    if hr >= 0 {
        hr = IMFSampleGrabberSinkCallback_OnClockSetRate(sample_grabber_get_callback(grabber), systime, rate);
    }
    return hr;
}

unsafe extern "system" fn sample_grabber_events_QueryInterface(iface: *mut IMFMediaEventGenerator, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFMediaEventGenerator(iface);
    return IMFMediaSink_QueryInterface(core::ptr::addr_of_mut!((*grabber).IMFMediaSink_iface), riid, obj);
}

unsafe extern "system" fn sample_grabber_events_AddRef(iface: *mut IMFMediaEventGenerator) -> ULONG {
    let grabber: *mut sample_grabber = impl_from_IMFMediaEventGenerator(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*grabber).IMFMediaSink_iface));
}

unsafe extern "system" fn sample_grabber_events_Release(iface: *mut IMFMediaEventGenerator) -> ULONG {
    let grabber: *mut sample_grabber = impl_from_IMFMediaEventGenerator(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*grabber).IMFMediaSink_iface));
}
unsafe extern "system" fn sample_grabber_events_GetEvent(iface: *mut IMFMediaEventGenerator, flags: DWORD, event: *mut *mut IMFMediaEvent) -> HRESULT {
    let mut grabber: *mut sample_grabber = impl_from_IMFMediaEventGenerator(iface);
    TRACE!("%p, %#lx, %p.\n", iface, flags, event);
    return IMFMediaEventQueue_GetEvent((*grabber).event_queue, flags, event);
}

unsafe extern "system" fn sample_grabber_events_BeginGetEvent(
    iface: *mut IMFMediaEventGenerator,
    callback: *mut IMFAsyncCallback,
    state: *mut IUnknown,
) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFMediaEventGenerator(iface);

    return IMFMediaEventQueue_BeginGetEvent((*grabber).event_queue, callback, state);
}

unsafe extern "system" fn sample_grabber_events_EndGetEvent(
    iface: *mut IMFMediaEventGenerator,
    result: *mut IMFAsyncResult,
    event: *mut *mut IMFMediaEvent,
) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFMediaEventGenerator(iface);

    TRACE!("%p, %p, %p.\n", iface, result, event);

    return IMFMediaEventQueue_EndGetEvent((*grabber).event_queue, result, event);
}

unsafe extern "system" fn sample_grabber_events_QueueEvent(
    iface: *mut IMFMediaEventGenerator,
    event_type: MediaEventType,
    ext_type: *const GUID,
    hr: HRESULT,
    value: *const PROPVARIANT,
) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFMediaEventGenerator(iface);

    return IMFMediaEventQueue_QueueEventParamVar(
        (*grabber).event_queue,
        event_type,
        ext_type,
        hr,
        value,
    );
}

unsafe extern "system" fn sample_grabber_getservice_QueryInterface(iface: *mut IMFGetService, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFGetService(iface);
    return IMFMediaSink_QueryInterface(core::ptr::addr_of_mut!((*grabber).IMFMediaSink_iface), riid, obj);
}

unsafe extern "system" fn sample_grabber_getservice_AddRef(iface: *mut IMFGetService) -> ULONG {
    let grabber: *mut sample_grabber = impl_from_IMFGetService(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*grabber).IMFMediaSink_iface));
}

unsafe extern "system" fn sample_grabber_getservice_Release(iface: *mut IMFGetService) -> ULONG {
    let grabber: *mut sample_grabber = impl_from_IMFGetService(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*grabber).IMFMediaSink_iface));
}

unsafe extern "system" fn sample_grabber_getservice_GetService(iface: *mut IMFGetService, service: *const GUID, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualGUID!(service, &MF_RATE_CONTROL_SERVICE) != false {
        if IsEqualIID!(riid, &raw const IID_IMFRateSupport) != 0 {
            return IMFGetService_QueryInterface(iface, riid, obj);
        }

        return E_NOINTERFACE;
    }

    return MF_E_UNSUPPORTED_SERVICE;
}

unsafe extern "system" fn sample_grabber_rate_support_QueryInterface(iface: *mut IMFRateSupport, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let grabber: *mut sample_grabber = impl_from_IMFRateSupport(iface);
    return IMFMediaSink_QueryInterface(core::ptr::addr_of_mut!((*grabber).IMFMediaSink_iface), riid, obj);
}

unsafe extern "system" fn sample_grabber_rate_support_AddRef(iface: *mut IMFRateSupport) -> ULONG {
    let grabber: *mut sample_grabber = impl_from_IMFRateSupport(iface);
    return IMFMediaSink_AddRef(core::ptr::addr_of_mut!((*grabber).IMFMediaSink_iface));
}

unsafe extern "system" fn sample_grabber_rate_support_Release(iface: *mut IMFRateSupport) -> ULONG {
    let grabber: *mut sample_grabber = impl_from_IMFRateSupport(iface);
    return IMFMediaSink_Release(core::ptr::addr_of_mut!((*grabber).IMFMediaSink_iface));
}

unsafe extern "system" fn sample_grabber_rate_support_GetSlowestRate(iface: *mut IMFRateSupport, direction: MFRATE_DIRECTION, thin: BOOL, rate: *mut f32) -> HRESULT {
    *rate = 0.0f32;

    return S_OK;
}
unsafe extern "system" fn sample_grabber_rate_support_GetFastestRate(iface: *mut IMFRateSupport, direction: MFRATE_DIRECTION, thin: BOOL, rate: *mut f32) -> HRESULT {
    *rate = if direction == MFRATE_REVERSE { -FLT_MAX } else { FLT_MAX };

    return S_OK;
}

unsafe extern "system" fn sample_grabber_rate_support_IsRateSupported(
    iface: *mut IMFRateSupport,
    thin: BOOL,
    rate: f32,
    ret_rate: *mut f32,
) -> HRESULT {
    if !ret_rate.is_null() {
        *ret_rate = rate;
    }

    return S_OK;
}

unsafe extern "C" fn sample_grabber_create_object(mut attributes: *mut IMFAttributes, mut user_context: *mut ::core::ffi::c_void, mut obj: *mut *mut IUnknown) -> HRESULT {
    let mut context: *mut sample_grabber_activate_context = user_context as *mut sample_grabber_activate_context;
    let mut object: *mut sample_grabber = core::ptr::null_mut::<sample_grabber>();
    let mut hr: HRESULT = 0;
    let mut guid: GUID = core::mem::zeroed::<GUID>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/samplegrabber.c\0" as *const u8 as *const ::core::ffi::c_char,
            b"sample_grabber_create_object\0" as *const u8 as *const ::core::ffi::c_char,
            1468 as ::core::ffi::c_int,
            b"%p, %p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            attributes,
            user_context,
            obj,
        );
    }
    if (*context).shut_down != 0 {
        return MF_E_SHUTDOWN;
    }
    if (IMFMediaType_GetMajorType((*context).media_type, &raw mut guid)) < 0 {
        return MF_E_INVALIDMEDIATYPE;
    }
    object = calloc(1, (core::mem::size_of::<sample_grabber>()) as _) as *mut sample_grabber;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }
    (*object).IMFMediaSink_iface.lpVtbl = &raw const sample_grabber_sink_vtbl;
    (*object).IMFClockStateSink_iface.lpVtbl = &raw const sample_grabber_clock_sink_vtbl;
    (*object).IMFMediaEventGenerator_iface.lpVtbl = &raw const sample_grabber_sink_events_vtbl;
    (*object).IMFGetService_iface.lpVtbl = &raw const sample_grabber_getservice_vtbl;
    (*object).IMFRateSupport_iface.lpVtbl = &raw const sample_grabber_rate_support_vtbl;
    (*object).IMFStreamSink_iface.lpVtbl = &raw const sample_grabber_stream_vtbl;
    (*object).IMFMediaTypeHandler_iface.lpVtbl = &raw const sample_grabber_stream_type_handler_vtbl;
    (*object).timer_callback.lpVtbl = &raw const sample_grabber_stream_timer_callback_vtbl;
    (*object).refcount = 1;
    (*object).rate = 1.0f32;
    if (IMFSampleGrabberSinkCallback_QueryInterface(
        (*context).callback,
        &raw const IID_IMFSampleGrabberSinkCallback2,
        &raw mut (*object).callback2 as *mut *mut ::core::ffi::c_void,
    )) < 0
    {
        (*object).callback = (*context).callback;
        IMFSampleGrabberSinkCallback_AddRef((*object).callback);
    }
    (*object).media_type = (*context).media_type;
    IMFMediaType_AddRef((*object).media_type);
    (*object).current_media_type = (*context).media_type;
    IMFMediaType_AddRef((*object).current_media_type);
    IMFAttributes_GetUINT32(attributes, &raw const MF_SAMPLEGRABBERSINK_IGNORE_CLOCK, &raw mut (*object).ignore_clock);
    IMFAttributes_GetUINT64(attributes, &raw const MF_SAMPLEGRABBERSINK_SAMPLE_TIME_OFFSET, &raw mut (*object).sample_time_offset);
    list_init(&raw mut (*object).items);
    InitializeCriticalSection(&raw mut (*object).cs);
    'failed: {
        hr = MFCreateEventQueue(&raw mut (*object).stream_event_queue);
        if hr < 0 {
            break 'failed;
        }
        hr = MFCreateAttributes(&raw mut (*object).sample_attributes, 0);
        if hr < 0 {
            break 'failed;
        }
        hr = MFCreateEventQueue(&raw mut (*object).event_queue);
        if hr < 0 {
            break 'failed;
        }
        *obj = &raw mut (*object).IMFMediaSink_iface as *mut IUnknown;
        if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int != 0 {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
            ros_dbg_log(
                (__dbcl_0) as u32,
                __dbch_0,
                b"dll/win32/mf/samplegrabber.c\0" as *const u8 as *const ::core::ffi::c_char,
                b"sample_grabber_create_object\0" as *const u8 as *const ::core::ffi::c_char,
                1516 as ::core::ffi::c_int,
                b"Created %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
                *obj,
            );
        }
        return S_OK;
    }
    IMFMediaSink_Release(&raw mut (*object).IMFMediaSink_iface);
    return hr;
}

unsafe extern "C" fn sample_grabber_shutdown_object(user_context: *mut core::ffi::c_void, obj: *mut IUnknown) {
    let mut context: *mut sample_grabber_activate_context = user_context as *mut sample_grabber_activate_context;
    (*context).shut_down = TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateSampleGrabberSinkActivate(
    media_type: *mut IMFMediaType,
    callback: *mut IMFSampleGrabberSinkCallback,
    activate: *mut *mut IMFActivate,
) -> HRESULT {
    let mut context: *mut sample_grabber_activate_context =
        core::ptr::null_mut::<sample_grabber_activate_context>();
    let mut hr: HRESULT;

    if media_type.is_null() || callback.is_null() || activate.is_null() {
        return E_POINTER;
    }

    context = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<sample_grabber_activate_context>() as size_t) as _,
    ) as *mut sample_grabber_activate_context;
    if context.is_null() {
        return E_OUTOFMEMORY;
    }

    (*context).media_type = media_type;
    IMFMediaType_AddRef((*context).media_type);
    (*context).callback = callback;
    IMFSampleGrabberSinkCallback_AddRef((*context).callback);

    hr = create_activation_object(
        context as *mut core::ffi::c_void,
        core::ptr::addr_of!(sample_grabber_activate_funcs),
        activate,
    );
    if hr < 0 {
        sample_grabber_free_private(context as *mut core::ffi::c_void);
    }

    return hr;
}

// COM vtable tables — generated from samplegrabber.c by vtbl_gen.py
/// samplegrabber.c:534 `static const IMFStreamSinkVtbl sample_grabber_stream_vtbl`  (slot order: rust:types_amd64.rs)
pub static sample_grabber_stream_vtbl: IMFStreamSinkVtbl = IMFStreamSinkVtbl {
    QueryInterface: Some(sample_grabber_stream_QueryInterface),
    AddRef: Some(sample_grabber_stream_AddRef),
    Release: Some(sample_grabber_stream_Release),
    GetEvent: Some(sample_grabber_stream_GetEvent),
    BeginGetEvent: Some(sample_grabber_stream_BeginGetEvent),
    EndGetEvent: Some(sample_grabber_stream_EndGetEvent),
    QueueEvent: Some(sample_grabber_stream_QueueEvent),
    GetMediaSink: Some(sample_grabber_stream_GetMediaSink),
    GetIdentifier: Some(sample_grabber_stream_GetIdentifier),
    GetMediaTypeHandler: Some(sample_grabber_stream_GetMediaTypeHandler),
    ProcessSample: Some(sample_grabber_stream_ProcessSample),
    PlaceMarker: Some(sample_grabber_stream_PlaceMarker),
    Flush: Some(sample_grabber_stream_Flush),
};

/// samplegrabber.c:689 `static const IMFMediaTypeHandlerVtbl sample_grabber_stream_type_handler_vtbl`  (slot order: rust:types_amd64.rs)
pub static sample_grabber_stream_type_handler_vtbl: IMFMediaTypeHandlerVtbl = IMFMediaTypeHandlerVtbl {
    QueryInterface: Some(sample_grabber_stream_type_handler_QueryInterface),
    AddRef: Some(sample_grabber_stream_type_handler_AddRef),
    Release: Some(sample_grabber_stream_type_handler_Release),
    IsMediaTypeSupported: Some(sample_grabber_stream_type_handler_IsMediaTypeSupported),
    GetMediaTypeCount: Some(sample_grabber_stream_type_handler_GetMediaTypeCount),
    GetMediaTypeByIndex: Some(sample_grabber_stream_type_handler_GetMediaTypeByIndex),
    SetCurrentMediaType: Some(sample_grabber_stream_type_handler_SetCurrentMediaType),
    GetCurrentMediaType: Some(sample_grabber_stream_type_handler_GetCurrentMediaType),
    GetMajorType: Some(sample_grabber_stream_type_handler_GetMajorType),
};

/// samplegrabber.c:778 `static const IMFAsyncCallbackVtbl sample_grabber_stream_timer_callback_vtbl`  (slot order: rust:types_amd64.rs)
pub static sample_grabber_stream_timer_callback_vtbl: IMFAsyncCallbackVtbl = IMFAsyncCallbackVtbl {
    QueryInterface: Some(sample_grabber_stream_timer_callback_QueryInterface),
    AddRef: Some(sample_grabber_stream_timer_callback_AddRef),
    Release: Some(sample_grabber_stream_timer_callback_Release),
    GetParameters: Some(sample_grabber_stream_timer_callback_GetParameters),
    Invoke: Some(sample_grabber_stream_timer_callback_Invoke),
};

/// samplegrabber.c:1106 `static const IMFMediaSinkVtbl sample_grabber_sink_vtbl`  (slot order: rust:types_amd64.rs)
pub static sample_grabber_sink_vtbl: IMFMediaSinkVtbl = IMFMediaSinkVtbl {
    QueryInterface: Some(sample_grabber_sink_QueryInterface),
    AddRef: Some(sample_grabber_sink_AddRef),
    Release: Some(sample_grabber_sink_Release),
    GetCharacteristics: Some(sample_grabber_sink_GetCharacteristics),
    AddStreamSink: Some(sample_grabber_sink_AddStreamSink),
    RemoveStreamSink: Some(sample_grabber_sink_RemoveStreamSink),
    GetStreamSinkCount: Some(sample_grabber_sink_GetStreamSinkCount),
    GetStreamSinkByIndex: Some(sample_grabber_sink_GetStreamSinkByIndex),
    GetStreamSinkById: Some(sample_grabber_sink_GetStreamSinkById),
    SetPresentationClock: Some(sample_grabber_sink_SetPresentationClock),
    GetPresentationClock: Some(sample_grabber_sink_GetPresentationClock),
    Shutdown: Some(sample_grabber_sink_Shutdown),
};

/// samplegrabber.c:1335 `static const IMFMediaEventGeneratorVtbl sample_grabber_sink_events_vtbl`  (slot order: rust:types_amd64.rs)
pub static sample_grabber_sink_events_vtbl: IMFMediaEventGeneratorVtbl = IMFMediaEventGeneratorVtbl {
    QueryInterface: Some(sample_grabber_events_QueryInterface),
    AddRef: Some(sample_grabber_events_AddRef),
    Release: Some(sample_grabber_events_Release),
    GetEvent: Some(sample_grabber_events_GetEvent),
    BeginGetEvent: Some(sample_grabber_events_BeginGetEvent),
    EndGetEvent: Some(sample_grabber_events_EndGetEvent),
    QueueEvent: Some(sample_grabber_events_QueueEvent),
};

/// samplegrabber.c:1346 `static const IMFClockStateSinkVtbl sample_grabber_clock_sink_vtbl`  (slot order: rust:types_amd64.rs)
pub static sample_grabber_clock_sink_vtbl: IMFClockStateSinkVtbl = IMFClockStateSinkVtbl {
    QueryInterface: Some(sample_grabber_clock_sink_QueryInterface),
    AddRef: Some(sample_grabber_clock_sink_AddRef),
    Release: Some(sample_grabber_clock_sink_Release),
    OnClockStart: Some(sample_grabber_clock_sink_OnClockStart),
    OnClockStop: Some(sample_grabber_clock_sink_OnClockStop),
    OnClockPause: Some(sample_grabber_clock_sink_OnClockPause),
    OnClockRestart: Some(sample_grabber_clock_sink_OnClockRestart),
    OnClockSetRate: Some(sample_grabber_clock_sink_OnClockSetRate),
};

/// samplegrabber.c:1394 `static const IMFGetServiceVtbl sample_grabber_getservice_vtbl`  (slot order: rust:types_amd64.rs)
pub static sample_grabber_getservice_vtbl: IMFGetServiceVtbl = IMFGetServiceVtbl {
    QueryInterface: Some(sample_grabber_getservice_QueryInterface),
    AddRef: Some(sample_grabber_getservice_AddRef),
    Release: Some(sample_grabber_getservice_Release),
    GetService: Some(sample_grabber_getservice_GetService),
};

/// samplegrabber.c:1451 `static const IMFRateSupportVtbl sample_grabber_rate_support_vtbl`  (slot order: rust:types_amd64.rs)
pub static sample_grabber_rate_support_vtbl: IMFRateSupportVtbl = IMFRateSupportVtbl {
    QueryInterface: Some(sample_grabber_rate_support_QueryInterface),
    AddRef: Some(sample_grabber_rate_support_AddRef),
    Release: Some(sample_grabber_rate_support_Release),
    GetSlowestRate: Some(sample_grabber_rate_support_GetSlowestRate),
    GetFastestRate: Some(sample_grabber_rate_support_GetFastestRate),
    IsRateSupported: Some(sample_grabber_rate_support_IsRateSupported),
};
