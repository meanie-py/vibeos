use crate::mfreadwrite::writer::stream;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];

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

#[no_mangle]
pub static mut MFVideoFormat_ABGR32: GUID = _GUID {
    Data1: D3DFMT_A8B8G8R8 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0x10 as ::core::ffi::c_ushort,
    Data4: [
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut readwrite_factory: IMFReadWriteClassFactory = unsafe {
    IMFReadWriteClassFactory {
        lpVtbl: &raw const readwrite_factory_vtbl,
    }
};

static mut classfactory: IClassFactory = unsafe {
    IClassFactory {
        lpVtbl: &raw const classfactoryvtbl,
    }
};

pub const STREAM_STATE_READY: i32 = 0;
pub const STREAM_STATE_EOS: i32 = 1;
pub type media_stream_state = i32;

pub const SOURCE_STATE_STOPPED: i32 = 0;
pub const SOURCE_STATE_STARTED: i32 = 1;
pub type media_source_state = i32;

pub const STREAM_FLAG_SAMPLE_REQUESTED: u32 = 0x1;
pub const STREAM_FLAG_SELECTED: u32 = 0x2;
pub const STREAM_FLAG_PRESENTED: u32 = 0x4;
pub const STREAM_FLAG_STOPPED: u32 = 0x8;

pub const SOURCE_READER_ASYNC_READ: i32 = 0;
pub const SOURCE_READER_ASYNC_SEEK: i32 = 1;
pub const SOURCE_READER_ASYNC_FLUSH: i32 = 2;
pub const SOURCE_READER_ASYNC_SAMPLE_READY: i32 = 3;
pub type source_reader_async_op = i32;

pub const SOURCE_READER_FLUSHING: u32 = 0x1;
pub const SOURCE_READER_SEEKING: u32 = 0x2;
pub const SOURCE_READER_SHUTDOWN_ON_RELEASE: u32 = 0x4;
pub const SOURCE_READER_D3D9_DEVICE_MANAGER: u32 = 0x8;
pub const SOURCE_READER_DXGI_DEVICE_MANAGER: u32 = 0x10;
pub const SOURCE_READER_HAS_DEVICE_MANAGER: u32 =
    SOURCE_READER_D3D9_DEVICE_MANAGER | SOURCE_READER_DXGI_DEVICE_MANAGER;

pub const WINE_DEFAULT_DEBUG_CHANNEL_NAME: &str = "mfplat";

#[repr(C)]
pub struct stream_response {
    pub entry: list,
    pub status: HRESULT,
    pub stream_index: DWORD,
    pub stream_flags: DWORD,
    pub timestamp: LONGLONG,
    pub sample: *mut IMFSample,
}
pub type PStreamResponse = *mut stream_response;

#[repr(C)]
pub struct transform_entry {
    pub entry: list,
    pub transform: *mut IMFTransform,
    pub min_buffer_size: u32,
    pub pending_flags: UINT32,
    pub category: GUID,
    pub hidden: BOOL,
    pub attributes_initialized: BOOL,
}
pub type PTransformEntry = *mut transform_entry;

#[repr(C)]
pub struct media_stream {
    pub stream: *mut IMFMediaStream,
    pub current: *mut IMFMediaType,
    pub transforms: list,
    pub allocator: *mut IMFVideoSampleAllocatorEx,
    pub transform_service: *mut IMFTransform,
    pub id: DWORD,
    pub index: u32,
    pub state: media_stream_state,
    pub flags: u32,
    pub requests: u32,
    pub responses: u32,
    pub last_sample_ts: LONGLONG,
    pub reader: *mut source_reader,
}
pub type PMediaStream = *mut media_stream;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct source_reader_async_command_u_read {
    pub flags: u32,
    pub stream_index: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct source_reader_async_command_u_seek {
    pub format: GUID,
    pub position: PROPVARIANT,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct source_reader_async_command_u_flush {
    pub stream_index: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct source_reader_async_command_u_sample {
    pub stream_index: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct source_reader_async_command_u_sa {
    pub stream_index: u32,
}

#[repr(C)]
pub union source_reader_async_command_u {
    pub read: source_reader_async_command_u_read,
    pub seek: source_reader_async_command_u_seek,
    pub flush: source_reader_async_command_u_flush,
    pub sample: source_reader_async_command_u_sample,
    pub sa: source_reader_async_command_u_sa,
}

#[repr(C)]
pub struct source_reader_async_command {
    pub IUnknown_iface: IUnknown,
    pub refcount: LONG,
    pub op: source_reader_async_op,
    pub u: source_reader_async_command_u,
}
pub type PSourceReaderAsyncCommand = *mut source_reader_async_command;

#[repr(C)]
pub struct source_reader {
    pub IMFSourceReaderEx_iface: IMFSourceReaderEx,
    pub source_events_callback: IMFAsyncCallback,
    pub stream_events_callback: IMFAsyncCallback,
    pub async_commands_callback: IMFAsyncCallback,
    pub refcount: LONG,
    pub public_refcount: LONG,
    pub source: *mut IMFMediaSource,
    pub descriptor: *mut IMFPresentationDescriptor,
    pub async_callback: *mut IMFSourceReaderCallback,
    pub attributes: *mut IMFAttributes,
    pub device_manager: *mut IUnknown,
    pub first_audio_stream_index: u32,
    pub first_video_stream_index: u32,
    pub stream_count: DWORD,
    pub flags: u32,
    pub queue: DWORD,
    pub source_state: media_source_state,
    pub streams: *mut media_stream,
    pub responses: list,
    pub cs: CRITICAL_SECTION,
    pub sample_event: CONDITION_VARIABLE,
    pub state_event: CONDITION_VARIABLE,
    pub stop_event: CONDITION_VARIABLE,
}
pub type PSourceReader = *mut source_reader;

unsafe extern "C" fn impl_from_IMFSourceReaderEx(iface: *mut IMFSourceReaderEx) -> *mut source_reader {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(source_reader, IMFSourceReaderEx_iface)) as *mut source_reader;
}

unsafe extern "C" fn impl_from_source_callback_IMFAsyncCallback(
    iface: *mut IMFAsyncCallback,
) -> *mut source_reader {
    return CONTAINING_RECORD!(iface, source_reader, source_events_callback);
}

unsafe extern "C" fn impl_from_stream_callback_IMFAsyncCallback(iface: *mut IMFAsyncCallback) -> *mut source_reader {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(source_reader, stream_events_callback)) as *mut source_reader;
}

unsafe extern "C" fn impl_from_async_commands_callback_IMFAsyncCallback(iface: *mut IMFAsyncCallback) -> *mut source_reader {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(source_reader, async_commands_callback)) as *mut source_reader;
}

unsafe extern "C" fn impl_from_async_command_IUnknown(iface: *mut IUnknown) -> *mut source_reader_async_command {
    return CONTAINING_RECORD!(iface, source_reader_async_command, IUnknown_iface);
}

unsafe extern "C" fn source_reader_addref(reader: *mut source_reader) -> ULONG {
    return InterlockedIncrement(core::ptr::addr_of_mut!((*reader).refcount) as *mut LONG) as ULONG;
}

unsafe extern "C" fn transform_entry_destroy(entry: *mut transform_entry) {
    IMFTransform_Release((*entry).transform);
    free(entry as *mut core::ffi::c_void);
}

unsafe extern "C" fn media_stream_destroy(stream: *mut media_stream) {
    let mut entry: *mut transform_entry = ::core::ptr::null_mut::<transform_entry>();
    let mut next: *mut transform_entry = ::core::ptr::null_mut::<transform_entry>();
    entry = ((*stream).transforms.next as *mut u8)
        .sub(core::mem::offset_of!(transform_entry, entry)) as *mut transform_entry;
    next = ((*entry).entry.next as *mut u8)
        .sub(core::mem::offset_of!(transform_entry, entry)) as *mut transform_entry;
    while core::ptr::addr_of_mut!((*entry).entry) != core::ptr::addr_of_mut!((*stream).transforms) {
        list_remove(core::ptr::addr_of_mut!((*entry).entry));
        transform_entry_destroy(entry);
        entry = next;
        next = ((*entry).entry.next as *mut u8)
            .sub(core::mem::offset_of!(transform_entry, entry)) as *mut transform_entry;
    }
    if !(*stream).transform_service.is_null() {
        IMFTransform_Release((*stream).transform_service);
    }
    if !(*stream).stream.is_null() {
        IMFMediaStream_Release((*stream).stream);
    }
    if !(*stream).current.is_null() {
        IMFMediaType_Release((*stream).current);
    }
    if !(*stream).allocator.is_null() {
        IMFVideoSampleAllocatorEx_Release((*stream).allocator);
    }
}

unsafe extern "C" fn source_reader_release(reader: *mut source_reader) -> ULONG {
    let mut refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*reader).refcount)) as ULONG;
    let mut i: ::core::ffi::c_uint;

    if refcount == 0 {
        if !(*reader).device_manager.is_null() {
            IUnknown_Release((*reader).device_manager);
        }
        if !(*reader).async_callback.is_null() {
            IMFSourceReaderCallback_Release((*reader).async_callback);
        }
        if !(*reader).descriptor.is_null() {
            IMFPresentationDescriptor_Release((*reader).descriptor);
        }
        if !(*reader).attributes.is_null() {
            IMFAttributes_Release((*reader).attributes);
        }
        IMFMediaSource_Release((*reader).source);

        i = 0 as ::core::ffi::c_uint;
        while (i as DWORD) < (*reader).stream_count {
            let mut stream: *mut media_stream = (*reader).streams.offset(i as isize) as *mut media_stream;
            media_stream_destroy(stream);
            i = i.wrapping_add(1);
        }
        source_reader_release_responses(reader, core::ptr::null_mut::<media_stream>());
        free((*reader).streams as *mut ::core::ffi::c_void);
        DeleteCriticalSection(core::ptr::addr_of_mut!((*reader).cs));
        free(reader as *mut ::core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn source_reader_async_command_QueryInterface(iface: *mut IUnknown, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0 {
        *obj = iface as *mut core::ffi::c_void;
        IUnknown_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn source_reader_async_command_AddRef(iface: *mut IUnknown) -> ULONG {
    let command: *mut source_reader_async_command = impl_from_async_command_IUnknown(iface);
    return InterlockedIncrement(core::ptr::addr_of_mut!((*command).refcount)) as ULONG;
}

unsafe extern "system" fn source_reader_async_command_Release(iface: *mut IUnknown) -> ULONG {
    let command: *mut source_reader_async_command = impl_from_async_command_IUnknown(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*command).refcount) as *mut ::core::ffi::c_long,
    ) as ULONG;

    if refcount == 0 {
        if (*command).op as ::core::ffi::c_uint
            == SOURCE_READER_ASYNC_SEEK as ::core::ffi::c_uint
        {
            PropVariantClear(core::ptr::addr_of_mut!((*command).u.seek.position));
        }
        free(command as *mut ::core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "C" fn source_reader_create_async_op(op: source_reader_async_op, ret: *mut *mut source_reader_async_command) -> HRESULT {
    let mut command: *mut source_reader_async_command = core::ptr::null_mut();
    command = calloc((1 as usize) as _, (core::mem::size_of::<source_reader_async_command>() as usize) as _) as *mut source_reader_async_command;
    if command.is_null() {
        return E_OUTOFMEMORY;
    }
    (*command).IUnknown_iface.lpVtbl = core::ptr::addr_of!(source_reader_async_command_vtbl);
    (*command).refcount = 1 as LONG;
    (*command).op = op;
    *ret = command;
    return S_OK;
}

unsafe extern "C" fn media_event_get_object(
    event: *mut IMFMediaEvent,
    riid: REFIID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut value: PROPVARIANT = core::mem::zeroed();
    let mut hr: HRESULT = 0;

    PropVariantInit!(core::ptr::addr_of_mut!(value));
    hr = IMFMediaEvent_GetValue(event, core::ptr::addr_of_mut!(value));
    if hr < 0 {
        WARN!("Failed to get event value, hr %#lx.\n", hr);
        return hr;
    }

    if value.vt != VT_UNKNOWN || value.__bindgen_anon_1.punkVal.is_null() {
        WARN!("Unexpected value type %d.\n", value.vt as core::ffi::c_uint);
        PropVariantClear(core::ptr::addr_of_mut!(value));
        return E_UNEXPECTED;
    }

    hr = IUnknown_QueryInterface(value.__bindgen_anon_1.punkVal, riid, obj);
    PropVariantClear(core::ptr::addr_of_mut!(value));
    if hr < 0 {
        WARN!("Unexpected object type.\n");
        return hr;
    }

    return hr;
}

unsafe extern "C" fn media_stream_get_id(stream: *mut IMFMediaStream, id: *mut DWORD) -> HRESULT {
    let mut sd: *mut IMFStreamDescriptor = core::ptr::null_mut();
    let mut hr: HRESULT;

    if {
        hr = IMFMediaStream_GetStreamDescriptor(stream, core::ptr::addr_of_mut!(sd));
        hr >= 0
    } {
        hr = IMFStreamDescriptor_GetStreamIdentifier(sd, id);
        IMFStreamDescriptor_Release(sd);
    }

    return hr;
}

unsafe extern "system" fn source_reader_callback_QueryInterface(
    iface: *mut IMFAsyncCallback,
    riid: REFIID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAsyncCallback)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = iface as *mut core::ffi::c_void;
        IMFAsyncCallback_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn source_reader_source_events_callback_AddRef(iface: *mut IMFAsyncCallback) -> ULONG {
    let reader: *mut source_reader = impl_from_source_callback_IMFAsyncCallback(iface);
    return source_reader_addref(reader);
}

unsafe extern "system" fn source_reader_source_events_callback_Release(iface: *mut IMFAsyncCallback) -> ULONG {
    let reader: *mut source_reader = impl_from_source_callback_IMFAsyncCallback(iface);
    return source_reader_release(reader);
}
unsafe extern "system" fn source_reader_callback_GetParameters(
    iface: *mut IMFAsyncCallback,
    flags: *mut DWORD,
    queue: *mut DWORD,
) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "C" fn source_reader_response_ready(reader: *mut source_reader, response: *mut stream_response) {
    let mut command: *mut source_reader_async_command = core::ptr::null_mut();
    let mut stream: *mut media_stream = core::ptr::null_mut();
    let mut hr: HRESULT;

    if (*response).stream_index < (*reader).stream_count {
        stream = (*reader).streams.add((*response).stream_index as usize);
    }

    if !stream.is_null() && (*stream).requests == 0 {
        return;
    }

    if !(*reader).async_callback.is_null() {
        if source_reader_create_async_op(SOURCE_READER_ASYNC_SAMPLE_READY, core::ptr::addr_of_mut!(command)) >= 0 {
            (*command).u.sample.stream_index = (*response).stream_index as UINT32;
            hr = MFPutWorkItem(
                (*reader).queue,
                core::ptr::addr_of_mut!((*reader).async_commands_callback),
                core::ptr::addr_of_mut!((*command).IUnknown_iface),
            );
            if hr < 0 {
                WARN!("Failed to submit async result, hr %#lx.\n", hr);
            }
            IUnknown_Release(core::ptr::addr_of_mut!((*command).IUnknown_iface));
        }
    } else {
        WakeAllConditionVariable(core::ptr::addr_of_mut!((*reader).sample_event));
    }

    if !stream.is_null() {
        (*stream).requests = (*stream).requests.wrapping_sub(1);
    }
}

unsafe extern "C" fn source_reader_copy_sample_buffer(src: *mut IMFSample, dst: *mut IMFSample) {
    let mut buffer: *mut IMFMediaBuffer = core::ptr::null_mut();
    let mut time: LONGLONG = 0;
    let mut flags: DWORD = 0;
    let mut hr: HRESULT = 0;

    IMFSample_CopyAllItems(src, dst as *mut IMFAttributes);

    IMFSample_SetSampleDuration(dst, 0);
    IMFSample_SetSampleTime(dst, 0);
    IMFSample_SetSampleFlags(dst, 0);

    if IMFSample_GetSampleDuration(src, core::ptr::addr_of_mut!(time)) >= 0 {
        IMFSample_SetSampleDuration(dst, time);
    }

    if IMFSample_GetSampleTime(src, core::ptr::addr_of_mut!(time)) >= 0 {
        IMFSample_SetSampleTime(dst, time);
    }

    if IMFSample_GetSampleFlags(src, core::ptr::addr_of_mut!(flags)) >= 0 {
        IMFSample_SetSampleFlags(dst, flags);
    }

    if IMFSample_ConvertToContiguousBuffer(src, core::ptr::null_mut()) >= 0 {
        if IMFSample_GetBufferByIndex(dst, 0, core::ptr::addr_of_mut!(buffer))
            >= 0
        {
            hr = IMFSample_CopyToBuffer(src, buffer);
            if hr < 0 {
                WARN!("Failed to copy a buffer, hr %#lx.\n", hr);
            }
            IMFMediaBuffer_Release(buffer);
        }
    }
}

unsafe extern "C" fn source_reader_queue_response(
    reader: *mut source_reader,
    stream: *mut media_stream,
    status: HRESULT,
    stream_flags: DWORD,
    timestamp: LONGLONG,
    sample: *mut IMFSample,
) -> HRESULT {
    let mut response: *mut stream_response;

    response = calloc(
        1,
        (core::mem::size_of::<stream_response>()) as _,
    ) as *mut stream_response;
    if response.is_null() {
        return E_OUTOFMEMORY;
    }

    (*response).status = status;
    (*response).stream_index = (*stream).index as DWORD;
    (*response).stream_flags = stream_flags;
    (*response).timestamp = timestamp;
    (*response).sample = sample;
    if !(*response).sample.is_null() {
        IMFSample_AddRef((*response).sample);
    }

    list_add_tail(&raw mut (*reader).responses, &raw mut (*response).entry);
    (*stream).responses += 1;

    source_reader_response_ready(reader, response);

    (*stream).last_sample_ts = timestamp;

    return S_OK;
}

unsafe extern "C" fn source_reader_queue_sample(
    reader: *mut source_reader,
    stream: *mut media_stream,
    flags: UINT,
    sample: *mut IMFSample,
) -> HRESULT {
    let mut timestamp: LONGLONG = 0;

    if (IMFSample_GetSampleTime(sample, core::ptr::addr_of_mut!(timestamp)) < 0) {
        WARN!("Sample time wasn't set.\n");
    }

    return source_reader_queue_response(reader, stream, S_OK, flags, timestamp, sample);
}

unsafe extern "C" fn source_reader_request_sample(reader: *mut source_reader, stream: *mut media_stream) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    if !(*stream).stream.is_null() && ((*stream).flags & STREAM_FLAG_SAMPLE_REQUESTED) == 0 {
        hr = IMFMediaStream_RequestSample((*stream).stream, core::ptr::null_mut());
        if hr < 0 {
            WARN!("Sample request failed, hr %#lx.\n", hr);
        } else {
            (*stream).flags |= STREAM_FLAG_SAMPLE_REQUESTED;
        }
    }

    return hr;
}

unsafe extern "C" fn source_reader_new_stream_handler(reader: *mut source_reader, event: *mut IMFMediaEvent) -> HRESULT {
    let mut stream: *mut IMFMediaStream = core::ptr::null_mut();
    let mut i: u32;
    let mut id: DWORD = 0;
    let mut hr: HRESULT;

    if {
        hr = media_event_get_object(event, core::ptr::addr_of!(IID_IMFMediaStream), core::ptr::addr_of_mut!(stream) as *mut *mut core::ffi::c_void);
        hr < 0
    } {
        WARN!("Failed to get stream object, hr %#lx.\n", hr);
        return hr;
    }

    if {
        hr = media_stream_get_id(stream, core::ptr::addr_of_mut!(id));
        hr < 0
    } {
        WARN!("Unidentified stream %p, hr %#lx.\n", stream, hr);
        IMFMediaStream_Release(stream);
        return hr;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*reader).cs));

    i = 0 as u32;
    while (i as DWORD) < (*reader).stream_count {
        if id == (*(*reader).streams.offset(i as isize)).id {
            if (*(*reader).streams.offset(i as isize)).stream.is_null() {
                (*(*reader).streams.offset(i as isize)).stream = stream;
                IMFMediaStream_AddRef((*(*reader).streams.offset(i as isize)).stream);
                if {
                    hr = IMFMediaStream_BeginGetEvent(stream, core::ptr::addr_of_mut!((*reader).stream_events_callback), stream as *mut IUnknown);
                    hr < 0
                } {
                    WARN!("Failed to subscribe to stream events, hr %#lx.\n", hr);
                }

                if (*(*reader).streams.offset(i as isize)).requests != 0 {
                    if {
                        source_reader_request_sample(reader, (*reader).streams.offset(i as isize))
                    } < 0
                    {
                        WakeAllConditionVariable(core::ptr::addr_of_mut!((*reader).sample_event));
                    }
                }
            }
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }

    if i as DWORD == (*reader).stream_count {
        WARN!("Stream with id %#lx was not present in presentation descriptor.\n", id);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*reader).cs));

    IMFMediaStream_Release(stream);

    return hr;
}

unsafe extern "C" fn source_reader_source_state_handler(
    mut reader: *mut source_reader,
    mut event_type: MediaEventType,
) -> HRESULT {
    EnterCriticalSection(core::ptr::addr_of_mut!((*reader).cs));

    match event_type {
        201 /* MESourceStarted */ => {
            (*reader).source_state = SOURCE_STATE_STARTED;
            (*reader).flags &= !(SOURCE_READER_SEEKING as ::core::ffi::c_uint);
        }
        207 /* MESourceStopped */ => {
            (*reader).source_state = SOURCE_STATE_STOPPED;
            (*reader).flags &= !(SOURCE_READER_SEEKING as ::core::ffi::c_uint);
        }
        203 /* MESourceSeeked */ => {
            (*reader).flags &= !(SOURCE_READER_SEEKING as ::core::ffi::c_uint);
        }
        _ => {
            WARN!("Unhandled event %ld.\n", event_type);
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*reader).cs));

    WakeAllConditionVariable(core::ptr::addr_of_mut!((*reader).state_event));
    if event_type == 207 /* MESourceStopped */ {
        WakeAllConditionVariable(core::ptr::addr_of_mut!((*reader).stop_event));
    }

    return S_OK;
}

unsafe extern "system" fn source_reader_source_events_callback_Invoke(
    iface: *mut IMFAsyncCallback,
    result: *mut IMFAsyncResult,
) -> HRESULT {
    let reader: *mut source_reader = impl_from_source_callback_IMFAsyncCallback(iface);
    let mut event_type: MediaEventType = 0;
    let mut source: *mut IMFMediaSource;
    let mut event: *mut IMFMediaEvent = core::ptr::null_mut();
    let mut hr: HRESULT;

    source = IMFAsyncResult_GetStateNoAddRef(result) as *mut IMFMediaSource;

    hr = IMFMediaSource_EndGetEvent(source, result, core::ptr::addr_of_mut!(event));
    if hr < 0 {
        return hr;
    }

    IMFMediaEvent_GetType(event, core::ptr::addr_of_mut!(event_type));

    match event_type {
        205 => {
            // MENewStream
            hr = source_reader_new_stream_handler(reader, event);
        }
        201 | 207 | 209 | 203 => {
            // MESourceStarted, MESourceStopped, MESourcePaused, MESourceSeeked
            hr = source_reader_source_state_handler(reader, event_type);
        }
        122 | 123 | 124 | 125 | 2 | 219 | 221 | 900 | 950 => {
            // MEBufferingStarted, MEBufferingStopped, MEConnectStart, MEConnectEnd,
            // MEExtendedType, MESourceCharacteristicsChanged, MESourceMetadataChanged,
            // MEContentProtectionMetadata, MEDeviceThermalStateChanged
            if !(*reader).async_callback.is_null() {
                IMFSourceReaderCallback_OnEvent(
                    (*reader).async_callback,
                    MF_SOURCE_READER_MEDIASOURCE as DWORD,
                    event,
                );
            }
        }
        _ => {
            ;
        }
    }

    if hr < 0 {
        WARN!("Failed while handling %ld event, hr %#lx.\n", event_type, hr);
    }

    IMFMediaEvent_Release(event);

    if event_type != 207 {
        // MESourceStopped
        IMFMediaSource_BeginGetEvent(source, iface, source as *mut IUnknown);
    }

    return S_OK;
}

unsafe extern "system" fn source_reader_stream_events_callback_AddRef(iface: *mut IMFAsyncCallback) -> ULONG {
    let reader: *mut source_reader = impl_from_stream_callback_IMFAsyncCallback(iface);
    return source_reader_addref(reader);
}

unsafe extern "system" fn source_reader_stream_events_callback_Release(iface: *mut IMFAsyncCallback) -> ULONG {
    let reader: *mut source_reader = impl_from_stream_callback_IMFAsyncCallback(iface);
    return source_reader_release(reader);
}

unsafe extern "C" fn source_reader_allocate_stream_sample(info: *mut MFT_OUTPUT_STREAM_INFO, out: *mut *mut IMFSample) -> HRESULT {
    let mut buffer: *mut IMFMediaBuffer = core::ptr::null_mut();
    let mut sample: *mut IMFSample = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    *out = core::ptr::null_mut();
    hr = MFCreateSample(core::ptr::addr_of_mut!(sample));
    if hr < 0 as HRESULT {
        return hr;
    }
    hr = MFCreateAlignedMemoryBuffer((*info).cbSize, (*info).cbAlignment, core::ptr::addr_of_mut!(buffer));
    if hr >= 0 as HRESULT {
        hr = IMFSample_AddBuffer(sample, buffer);
        if hr >= 0 as HRESULT {
            *out = sample;
            IMFSample_AddRef(sample);
        }
        IMFMediaBuffer_Release(buffer);
    }

    IMFSample_Release(sample);
    return hr;
}

unsafe extern "C" fn media_type_try_copy_attr(
    dst: *mut IMFMediaType,
    src: *mut IMFMediaType,
    attr: *const GUID,
    hr: *mut HRESULT,
) {
    let mut value: PROPVARIANT = core::mem::zeroed();

    PropVariantInit!(&raw mut value);
    if (*hr >= 0)
        && (IMFMediaType_GetItem(dst, attr, core::ptr::null_mut()) < 0)
        && (IMFMediaType_GetItem(src, attr, &raw mut value) >= 0)
    {
        *hr = IMFMediaType_SetItem(dst, attr, &raw mut value);
    }
    PropVariantClear(&raw mut value);
}

unsafe extern "C" fn media_type_get_uint32_pair(type_: *mut IMFMediaType, key: *const GUID, hi: *mut UINT32, lo: *mut UINT32) -> HRESULT {
    let mut data: UINT64 = 0;
    let mut hr: HRESULT = IMFMediaType_GetUINT64(type_, key, core::ptr::addr_of_mut!(data));
    if hr < 0 {
        return hr;
    }

    *hi = (data >> 32) as UINT32;
    *lo = (data & 0xffffffff) as UINT32;
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn update_media_type(dst_type: *mut IMFMediaType, src_type: *mut IMFMediaType, advanced: BOOL) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    /* propagate common video attributes */
    media_type_try_copy_attr(dst_type, src_type, &raw const MF_MT_FRAME_SIZE, &raw mut hr);
    media_type_try_copy_attr(dst_type, src_type, &raw const MF_MT_FRAME_RATE, &raw mut hr);
    media_type_try_copy_attr(dst_type, src_type, &raw const MF_MT_VIDEO_ROTATION, &raw mut hr);
    media_type_try_copy_attr(dst_type, src_type, &raw const MF_MT_FIXED_SIZE_SAMPLES, &raw mut hr);
    media_type_try_copy_attr(dst_type, src_type, &raw const MF_MT_PIXEL_ASPECT_RATIO, &raw mut hr);
    media_type_try_copy_attr(dst_type, src_type, &raw const MF_MT_ALL_SAMPLES_INDEPENDENT, &raw mut hr);
    media_type_try_copy_attr(dst_type, src_type, &raw const MF_MT_MINIMUM_DISPLAY_APERTURE, &raw mut hr);

    media_type_try_copy_attr(dst_type, src_type, &raw const MF_MT_VIDEO_CHROMA_SITING, &raw mut hr);
    media_type_try_copy_attr(dst_type, src_type, &raw const MF_MT_INTERLACE_MODE, &raw mut hr);
    media_type_try_copy_attr(dst_type, src_type, &raw const MF_MT_TRANSFER_FUNCTION, &raw mut hr);
    media_type_try_copy_attr(dst_type, src_type, &raw const MF_MT_VIDEO_PRIMARIES, &raw mut hr);
    media_type_try_copy_attr(dst_type, src_type, &raw const MF_MT_YUV_MATRIX, &raw mut hr);
    media_type_try_copy_attr(dst_type, src_type, &raw const MF_MT_VIDEO_LIGHTING, &raw mut hr);
    media_type_try_copy_attr(dst_type, src_type, &raw const MF_MT_VIDEO_NOMINAL_RANGE, &raw mut hr);

    if advanced == 0 {
        media_type_try_copy_attr(dst_type, src_type, &raw const MF_MT_DEFAULT_STRIDE, &raw mut hr);
    }

    /* propagate common audio attributes */
    media_type_try_copy_attr(dst_type, src_type, &raw const MF_MT_AUDIO_NUM_CHANNELS, &raw mut hr);
    media_type_try_copy_attr(dst_type, src_type, &raw const MF_MT_AUDIO_BLOCK_ALIGNMENT, &raw mut hr);
    media_type_try_copy_attr(dst_type, src_type, &raw const MF_MT_AUDIO_BITS_PER_SAMPLE, &raw mut hr);
    media_type_try_copy_attr(dst_type, src_type, &raw const MF_MT_AUDIO_SAMPLES_PER_SECOND, &raw mut hr);
    media_type_try_copy_attr(dst_type, src_type, &raw const MF_MT_AUDIO_AVG_BYTES_PER_SECOND, &raw mut hr);
    media_type_try_copy_attr(dst_type, src_type, &raw const MF_MT_AUDIO_CHANNEL_MASK, &raw mut hr);
    media_type_try_copy_attr(dst_type, src_type, &raw const MF_MT_AUDIO_SAMPLES_PER_BLOCK, &raw mut hr);
    media_type_try_copy_attr(dst_type, src_type, &raw const MF_MT_AUDIO_VALID_BITS_PER_SAMPLE, &raw mut hr);

    return hr;
}

unsafe extern "C" fn source_reader_push_transform_samples(
    mut reader: *mut source_reader,
    mut stream: *mut media_stream,
    mut entry: *mut transform_entry,
    mut sample: *mut IMFSample,
) -> HRESULT {
    let mut hr: HRESULT = 0;
    loop {
        hr = source_reader_pull_transform_samples(reader, stream, entry);
        if hr < 0 && hr != MF_E_TRANSFORM_NEED_MORE_INPUT {
            return hr;
        }
        hr = IMFTransform_ProcessInput((*entry).transform, 0, sample, 0);
        if hr >= 0 {
            hr = source_reader_pull_transform_samples(reader, stream, entry);
            return if hr == MF_E_TRANSFORM_NEED_MORE_INPUT {
                S_OK
            } else {
                hr
            };
        }
        if !(hr == MF_E_NOTACCEPTING) {
            break;
        }
    }
    return hr;
}

unsafe extern "C" fn set_matching_transform_output_type(
    transform: *mut IMFTransform,
    old_output_type: *mut IMFMediaType,
) -> HRESULT {
    let mut new_output_type: *mut IMFMediaType = core::ptr::null_mut();
    let mut subtype: GUID = core::mem::zeroed();
    let mut desired: GUID = core::mem::zeroed();
    let mut i: UINT = 0;
    let mut hr: HRESULT = 0;

    IMFMediaType_GetGUID(old_output_type, &raw const MF_MT_SUBTYPE, &raw mut desired);

    /* find an available output type matching the desired subtype */
    loop {
        let fresh17 = i;
        i = i.wrapping_add(1);
        hr = IMFTransform_GetOutputAvailableType(
            transform,
            0,
            fresh17,
            &raw mut new_output_type,
        );
        if !(hr >= 0) {
            break;
        }
        IMFMediaType_GetGUID(new_output_type, &raw const MF_MT_SUBTYPE, &raw mut subtype);
        if IsEqualGUID!(&raw mut subtype, &raw mut desired) != false
            && {
                hr = IMFTransform_SetOutputType(
                    transform,
                    0,
                    new_output_type,
                    0,
                );
                hr >= 0
            }
        {
            IMFMediaType_Release(new_output_type);
            return S_OK;
        }
        IMFMediaType_Release(new_output_type);
    }

    return hr;
}

unsafe extern "C" fn transform_entry_update_output_type(entry: *mut transform_entry, old_output_type: *mut IMFMediaType) -> HRESULT {
    let mut hr: HRESULT;

    if ({ hr = set_matching_transform_output_type((*entry).transform, old_output_type); hr } >= 0) {
        (*entry).pending_flags |= MF_SOURCE_READERF_CURRENTMEDIATYPECHANGED as UINT32;
    }

    return hr;
}

unsafe extern "C" fn transform_entry_update_input_type(entry: *mut transform_entry, input_type: *mut IMFMediaType) -> HRESULT {
    let mut old_output_type: *mut IMFMediaType = core::ptr::null_mut();
    let mut new_output_type: *mut IMFMediaType = core::ptr::null_mut();
    let mut hr: HRESULT;

    hr = IMFTransform_GetOutputCurrentType((*entry).transform, 0, core::ptr::addr_of_mut!(old_output_type));
    if hr < 0 {
        return hr;
    }
    hr = IMFTransform_SetInputType((*entry).transform, 0, input_type, 0);
    if hr < 0 {
        return hr;
    }

    /* check if transform output type is still valid or if we need to update it as well */
    hr = IMFTransform_GetOutputCurrentType((*entry).transform, 0, core::ptr::addr_of_mut!(new_output_type));
    if hr < 0 {
        hr = transform_entry_update_output_type(entry, old_output_type);
    } else {
        IMFMediaType_Release(new_output_type);
    }

    IMFMediaType_Release(old_output_type);
    return hr;
}

unsafe extern "C" fn transform_entry_initialize_attributes(reader: *mut source_reader, entry: *mut transform_entry) {
    let mut attributes: *mut IMFAttributes = core::ptr::null_mut();

    if (IMFTransform_GetAttributes((*entry).transform, &raw mut attributes)) >= 0 {
        if (IMFAttributes_GetItem(attributes, &raw const MF_SA_MINIMUM_OUTPUT_SAMPLE_COUNT, core::ptr::null_mut())) < 0 {
            IMFAttributes_SetUINT32(attributes, &raw const MF_SA_MINIMUM_OUTPUT_SAMPLE_COUNT, 6);
        }

        IMFAttributes_Release(attributes);
    }

    if (IMFTransform_GetOutputStreamAttributes((*entry).transform, 0, &raw mut attributes)) >= 0 {
        let mut shared: UINT32 = 0;
        let mut shared_without_mutex: UINT32 = 0;
        let mut bind_flags: UINT32 = 0;

        if (IMFAttributes_GetUINT32((*reader).attributes, &raw const MF_SA_D3D11_SHARED, &raw mut shared)) >= 0 {
            IMFAttributes_SetUINT32(attributes, &raw const MF_SA_D3D11_SHARED, shared);
        }
        if (IMFAttributes_GetUINT32((*reader).attributes, &raw const MF_SA_D3D11_SHARED_WITHOUT_MUTEX, &raw mut shared_without_mutex)) >= 0 {
            IMFAttributes_SetUINT32(attributes, &raw const MF_SA_D3D11_SHARED_WITHOUT_MUTEX, shared_without_mutex);
        }
        if (IMFAttributes_GetUINT32((*reader).attributes, &raw const MF_SOURCE_READER_D3D11_BIND_FLAGS, &raw mut bind_flags)) >= 0 {
            IMFAttributes_SetUINT32(attributes, &raw const MF_SA_D3D11_BINDFLAGS, bind_flags);
        } else if (((*reader).flags & SOURCE_READER_DXGI_DEVICE_MANAGER) != 0)
            && ((IMFAttributes_GetItem(attributes, &raw const MF_SA_D3D11_BINDFLAGS, core::ptr::null_mut())) < 0)
        {
            IMFAttributes_SetUINT32(attributes, &raw const MF_SA_D3D11_BINDFLAGS, 1024);
        }

        IMFAttributes_Release(attributes);
    }
}

unsafe extern "C" fn source_reader_pull_transform_samples(
    mut reader: *mut source_reader,
    mut stream: *mut media_stream,
    mut entry: *mut transform_entry,
) -> HRESULT {
    let mut stream_info: MFT_OUTPUT_STREAM_INFO = MFT_OUTPUT_STREAM_INFO {
        dwFlags: 0 as DWORD,
        cbSize: 0,
        cbAlignment: 0,
    };
    let mut next: *mut transform_entry = ::core::ptr::null_mut::<transform_entry>();
    let mut ptr: *mut list = ::core::ptr::null_mut::<list>();
    let mut status: DWORD = 0;
    let mut hr: HRESULT = 0;

    ptr = list_next(&raw mut (*stream).transforms, &raw mut (*entry).entry);
    if !ptr.is_null() {
        next = (ptr as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(transform_entry, entry)
                as ::core::ffi::c_ulonglong as isize),
        ) as *mut transform_entry;
    }

    if (*entry).attributes_initialized == 0 {
        transform_entry_initialize_attributes(reader, entry);
        (*entry).attributes_initialized = TRUE as BOOL;
    }

    hr = IMFTransform_GetOutputStreamInfo(
        (*entry).transform,
        0 as DWORD,
        &raw mut stream_info,
    );
    if hr < 0 as HRESULT {
        return hr;
    }
    stream_info.cbSize = max!(stream_info.cbSize, (*entry).min_buffer_size as DWORD);

    loop {
        let mut out_buffer: MFT_OUTPUT_DATA_BUFFER = MFT_OUTPUT_DATA_BUFFER {
            dwStreamID: 0 as DWORD,
            pSample: ::core::ptr::null_mut::<IMFSample>(),
            dwStatus: 0,
            pEvents: ::core::ptr::null_mut::<IMFCollection>(),
        };
        let mut media_type: *mut IMFMediaType = ::core::ptr::null_mut::<IMFMediaType>();

        if (stream_info.dwFlags
            & (MFT_OUTPUT_STREAM_PROVIDES_SAMPLES as DWORD | MFT_OUTPUT_STREAM_CAN_PROVIDE_SAMPLES as DWORD))
            == 0
            && {
                hr = source_reader_allocate_stream_sample(&raw mut stream_info, &raw mut out_buffer.pSample);
                hr < 0 as HRESULT
            }
        {
            break;
        }

        hr = IMFTransform_ProcessOutput(
            (*entry).transform,
            0 as DWORD,
            1 as DWORD,
            &raw mut out_buffer,
            &raw mut status,
        );
        if hr >= 0 as HRESULT {
            /* propagate upstream type to the transform input type */
            if ((*entry).pending_flags & MF_SOURCE_READERF_CURRENTMEDIATYPECHANGED as UINT32) != 0
                && {
                    hr = IMFTransform_GetOutputCurrentType(
                        (*entry).transform,
                        0 as DWORD,
                        &raw mut media_type,
                    );
                    hr >= 0 as HRESULT
                }
            {
                if next.is_null() {
                    hr = IMFMediaType_CopyAllItems(
                        media_type,
                        (*stream).current as *mut IMFAttributes,
                    );
                } else {
                    hr = transform_entry_update_input_type(next, media_type);
                }
                IMFMediaType_Release(media_type);
            }

            if hr < 0 as HRESULT {
                source_reader_queue_response(
                    reader,
                    stream,
                    hr,
                    MF_SOURCE_READERF_ERROR as DWORD,
                    0 as LONGLONG,
                    ::core::ptr::null_mut::<IMFSample>(),
                );
            } else if !next.is_null() {
                hr = source_reader_push_transform_samples(reader, stream, next, out_buffer.pSample);
            } else {
                hr = source_reader_queue_sample(
                    reader,
                    stream,
                    (*entry).pending_flags as UINT,
                    out_buffer.pSample,
                );
            }

            (*entry).pending_flags = 0 as UINT32;
        }

        if hr == MF_E_TRANSFORM_STREAM_CHANGE as HRESULT
            && {
                hr = IMFTransform_GetOutputCurrentType(
                    (*entry).transform,
                    0 as DWORD,
                    &raw mut media_type,
                );
                hr >= 0 as HRESULT
            }
        {
            hr = transform_entry_update_output_type(entry, media_type);
            IMFMediaType_Release(media_type);

            if hr >= 0 as HRESULT {
                hr = IMFTransform_GetOutputStreamInfo(
                    (*entry).transform,
                    0 as DWORD,
                    &raw mut stream_info,
                );
                stream_info.cbSize = max!(stream_info.cbSize, (*entry).min_buffer_size as DWORD);
            }
        }

        if !out_buffer.pSample.is_null() {
            IMFSample_Release(out_buffer.pSample);
        }
        if !out_buffer.pEvents.is_null() {
            IMFCollection_Release(out_buffer.pEvents);
        }

        if !(hr >= 0 as HRESULT && !next.is_null()) {
            break;
        }
    }

    return hr;
}

unsafe extern "C" fn source_reader_drain_transform_samples(
    mut reader: *mut source_reader,
    mut stream: *mut media_stream,
    mut entry: *mut transform_entry,
) -> HRESULT {
    let mut next: *mut transform_entry = core::ptr::null_mut::<transform_entry>();
    let mut ptr: *mut list = core::ptr::null_mut::<list>();
    let mut hr: HRESULT = 0;
    ptr = list_next(&raw mut (*stream).transforms, &raw mut (*entry).entry);
    if !ptr.is_null() {
        next = (ptr as *mut u8).offset(
            -(core::mem::offset_of!(transform_entry, entry) as usize as isize),
        ) as *mut transform_entry;
    }
    hr = IMFTransform_ProcessMessage(
        (*entry).transform,
        MFT_MESSAGE_COMMAND_DRAIN,
        (0 as ULONG_PTR),
    );
    if hr < 0 {
        WARN!("Failed to drain transform %p, hr %#lx\n", (*entry).transform, hr);
    }
    /* MF_E_SAMPLEALLOCATOR_EMPTY can occur here if many samples are drained
     * from a transform, but it's not an issue if a later call succeeds. */
    hr = source_reader_pull_transform_samples(reader, stream, entry);
    if hr < 0 && hr != MF_E_TRANSFORM_NEED_MORE_INPUT {
        WARN!("Failed to pull pending samples, hr %#lx.\n", hr);
    }
    return if !next.is_null() {
        source_reader_drain_transform_samples(reader, stream, next)
    } else {
        S_OK
    };
}

unsafe extern "C" fn source_reader_flush_transform_samples(
    mut reader: *mut source_reader,
    mut stream: *mut media_stream,
    mut entry: *mut transform_entry,
) -> HRESULT {
    let mut next: *mut transform_entry = core::ptr::null_mut::<transform_entry>();
    let mut ptr: *mut list = core::ptr::null_mut::<list>();
    let mut hr: HRESULT = 0;
    ptr = list_next(&raw mut (*stream).transforms, &raw mut (*entry).entry);
    if !ptr.is_null() {
        next = (ptr as *mut core::ffi::c_char).offset(
            -(core::mem::offset_of!(transform_entry, entry)
                as core::ffi::c_ulonglong as isize),
        ) as *mut transform_entry;
    }
    hr = IMFTransform_ProcessMessage(
        (*entry).transform,
        MFT_MESSAGE_COMMAND_FLUSH,
        0 as ULONG_PTR,
    );
    if hr < 0 as HRESULT {
        WARN!("Failed to flush transform %p, hr %#lx\n", (*entry).transform, hr);
    }
    return if !next.is_null() {
        source_reader_flush_transform_samples(reader, stream, next)
    } else {
        S_OK
    };
}

unsafe extern "C" fn source_reader_notify_transform(reader: *mut source_reader, stream: *mut media_stream, entry: *mut transform_entry, message: UINT) -> HRESULT {
    let mut next: *mut transform_entry = core::ptr::null_mut();
    let mut ptr: *mut list;
    let mut hr: HRESULT;

    ptr = list_next(core::ptr::addr_of_mut!((*stream).transforms), core::ptr::addr_of_mut!((*entry).entry));
    if !ptr.is_null() {
        next = LIST_ENTRY!(ptr, transform_entry, entry);
    }

    hr = IMFTransform_ProcessMessage((*entry).transform, message as MFT_MESSAGE_TYPE, 0 as ULONG_PTR);
    if hr < 0 {
        WARN!("Failed to notify transform %p message %#x, hr %#lx\n", (*entry).transform, message, hr);
    }

    return if !next.is_null() {
        source_reader_notify_transform(reader, stream, next, message)
    } else {
        S_OK
    };
}

unsafe extern "C" fn source_reader_process_sample(
    reader: *mut source_reader,
    stream: *mut media_stream,
    sample: *mut IMFSample,
) -> HRESULT {
    let mut entry: *mut transform_entry = core::ptr::null_mut();
    let mut ptr: *mut list = core::ptr::null_mut();
    let mut hr: HRESULT;

    ptr = list_head(core::ptr::addr_of_mut!((*stream).transforms));
    if ptr.is_null() {
        return source_reader_queue_sample(reader, stream, 0 as UINT, sample);
    }
    entry = (ptr as *mut core::ffi::c_char).offset(
        -(core::mem::offset_of!(transform_entry, entry)
            as core::ffi::c_ulonglong as isize),
    ) as *mut transform_entry;

    /* It's assumed that decoder has 1 input and 1 output, both id's are 0. */
    hr = source_reader_push_transform_samples(reader, stream, entry, sample);
    if hr >= 0 {
        hr = if (*stream).requests != 0 {
            source_reader_request_sample(reader, stream)
        } else {
            S_OK
        };
    } else {
        WARN!("Transform failed to process output, hr %#lx.\n", hr);
    }

    return hr;
}

unsafe extern "C" fn source_reader_media_sample_handler(
    mut reader: *mut source_reader,
    mut stream: *mut IMFMediaStream,
    mut event: *mut IMFMediaEvent,
) -> HRESULT {
    let mut sample: *mut IMFSample = core::ptr::null_mut::<IMFSample>();
    let mut i: ::core::ffi::c_uint = 0;
    let mut id: DWORD = 0 as DWORD;
    let mut hr: HRESULT = 0;

    TRACE!("Got new sample for stream %p.\n", stream);

    hr = media_event_get_object(
        event,
        core::ptr::addr_of!(IID_IMFSample),
        &mut sample as *mut *mut IMFSample as *mut *mut ::core::ffi::c_void,
    );
    if hr < 0 as HRESULT {
        WARN!("Failed to get sample object, hr %#lx.\n", hr);
        return hr;
    }

    hr = media_stream_get_id(stream, &mut id as *mut DWORD);
    if hr < 0 as HRESULT {
        WARN!("Unidentified stream %p, hr %#lx.\n", stream, hr);
        IMFSample_Release(sample);
        return hr;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*reader).cs));

    i = 0 as ::core::ffi::c_uint;
    while (i as DWORD) < (*reader).stream_count {
        if id == (*(*reader).streams.add(i as usize)).id {
            (*(*reader).streams.add(i as usize)).flags &=
                !(STREAM_FLAG_SAMPLE_REQUESTED as ::core::ffi::c_uint);
            hr = source_reader_process_sample(
                reader,
                (*reader).streams.add(i as usize),
                sample,
            );
            break;
        }
        i = i.wrapping_add(1);
    }

    if i as DWORD == (*reader).stream_count {
        WARN!(
            "Stream with id %#lx was not present in presentation descriptor.\n",
            id
        );
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*reader).cs));

    IMFSample_Release(sample);

    return hr;
}

unsafe extern "C" fn source_reader_media_stream_state_handler(reader: *mut source_reader, stream: *mut IMFMediaStream, event: *mut IMFMediaEvent) -> HRESULT {
    let mut event_type: MediaEventType = 0;
    let mut timestamp: LONGLONG = 0;
    let mut value: PROPVARIANT = core::mem::zeroed();
    let mut ptr: *mut list = core::ptr::null_mut();
    let mut i: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;
    let mut id: DWORD = 0;

    IMFMediaEvent_GetType(event, core::ptr::addr_of_mut!(event_type));

    hr = media_stream_get_id(stream, core::ptr::addr_of_mut!(id));
    if hr < 0 {
        WARN!("Unidentified stream %p, hr %#lx.\n", stream, hr);
        return hr;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*reader).cs));

    i = 0 as ::core::ffi::c_uint;
    while (i as DWORD) < (*reader).stream_count {
        let stream: *mut media_stream = (*reader).streams.offset(i as isize);

        if id == (*stream).id {
            match event_type {
                212 => {
                    (*stream).state = STREAM_STATE_EOS;
                    (*stream).flags &= !(STREAM_FLAG_SAMPLE_REQUESTED as ::core::ffi::c_uint);

                    ptr = list_head(core::ptr::addr_of_mut!((*stream).transforms));
                    if !ptr.is_null() {
                        let entry: *mut transform_entry = LIST_ENTRY!(ptr, transform_entry, entry);
                        hr = source_reader_drain_transform_samples(reader, stream, entry);
                        if hr < 0 {
                            WARN!("Failed to drain pending samples, hr %#lx.\n", hr);
                        }
                    }

                    while (*stream).requests != 0 {
                        source_reader_queue_response(
                            reader,
                            stream,
                            S_OK,
                            MF_SOURCE_READERF_ENDOFSTREAM,
                            0,
                            core::ptr::null_mut(),
                        );
                    }
                }
                204 | 202 => {
                    (*stream).state = STREAM_STATE_READY;

                    ptr = list_head(core::ptr::addr_of_mut!((*stream).transforms));
                    if !ptr.is_null() {
                        let entry: *mut transform_entry = LIST_ENTRY!(ptr, transform_entry, entry);
                        hr = source_reader_notify_transform(reader, stream, entry, MFT_MESSAGE_NOTIFY_START_OF_STREAM);
                        if hr < 0 {
                            WARN!("Failed to notify transforms of stream start, hr %#lx.\n", hr);
                        }
                    }
                }
                208 => {
                    (*stream).flags |= STREAM_FLAG_STOPPED as ::core::ffi::c_uint;
                }
                214 => {
                    value.vt = VT_EMPTY;
                    hr = if (IMFMediaEvent_GetValue(event, core::ptr::addr_of_mut!(value)) >= 0)
                        && value.vt == (VT_I8 as u16)
                    {
                        S_OK
                    } else {
                        E_UNEXPECTED
                    };
                    timestamp = if hr >= 0 {
                        value.__bindgen_anon_1.hVal.QuadPart
                    } else {
                        0
                    };
                    PropVariantClear(core::ptr::addr_of_mut!(value));

                    source_reader_queue_response(
                        reader,
                        stream,
                        hr,
                        MF_SOURCE_READERF_STREAMTICK,
                        timestamp,
                        core::ptr::null_mut(),
                    );
                }
                _ => {}
            }

            break;
        }
        i = i.wrapping_add(1);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*reader).cs));

    if event_type == 208 /* MEStreamStopped */ {
        WakeAllConditionVariable(core::ptr::addr_of_mut!((*reader).stop_event));
    }

    return S_OK;
}

unsafe extern "system" fn source_reader_stream_events_callback_Invoke(
    mut iface: *mut IMFAsyncCallback,
    mut result: *mut IMFAsyncResult,
) -> HRESULT {
    let mut reader: *mut source_reader = impl_from_stream_callback_IMFAsyncCallback(iface);
    let mut event_type: MediaEventType = 0;
    let mut stream: *mut IMFMediaStream = core::ptr::null_mut::<IMFMediaStream>();
    let mut event: *mut IMFMediaEvent = core::ptr::null_mut::<IMFMediaEvent>();
    let mut hr: HRESULT = 0;

    TRACE!("%p, %p.\n", iface, result);

    stream = IMFAsyncResult_GetStateNoAddRef(result) as *mut IMFMediaStream;

    hr = IMFMediaStream_EndGetEvent(stream, result, core::ptr::addr_of_mut!(event));
    if hr < 0 {
        return hr;
    }

    IMFMediaEvent_GetType(event, core::ptr::addr_of_mut!(event_type));

    TRACE!("Got event %lu.\n", event_type);

    match event_type {
        213 => {
            // MEMediaSample
            hr = source_reader_media_sample_handler(reader, stream, event);
        }
        204 | 202 | 208 | 214 | 212 => {
            // MEStreamSeeked, MEStreamStarted, MEStreamStopped, MEStreamTick, MEEndOfStream
            hr = source_reader_media_stream_state_handler(reader, stream, event);
        }
        _ => {}
    }

    if hr < 0 {
        WARN!("Failed while handling %ld event, hr %#lx.\n", event_type, hr);
    }

    IMFMediaEvent_Release(event);

    if event_type != 208 {
        // MEStreamStopped
        IMFMediaStream_BeginGetEvent(stream, iface, stream as *mut IUnknown);
    }

    return S_OK;
}

unsafe extern "system" fn source_reader_async_commands_callback_AddRef(iface: *mut IMFAsyncCallback) -> ULONG {
    let reader: *mut source_reader = impl_from_async_commands_callback_IMFAsyncCallback(iface);
    return source_reader_addref(reader);
}

unsafe extern "system" fn source_reader_async_commands_callback_Release(iface: *mut IMFAsyncCallback) -> ULONG {
    let reader: *mut source_reader = impl_from_async_commands_callback_IMFAsyncCallback(iface);
    return source_reader_release(reader);
}

unsafe extern "C" fn media_stream_detach_response(
    mut reader: *mut source_reader,
    mut response: *mut stream_response,
) -> *mut stream_response {
    let mut stream: *mut media_stream = core::ptr::null_mut::<media_stream>();
    list_remove(core::ptr::addr_of_mut!((*response).entry));

    if (*response).stream_index < (*reader).stream_count {
        stream = (*reader).streams.offset((*response).stream_index as isize) as *mut media_stream;
        if (*stream).responses != 0 {
            (*stream).responses = (*stream).responses.wrapping_sub(1);
        }
    }

    return response;
}

unsafe extern "C" fn media_stream_pop_response(
    reader: *mut source_reader,
    mut stream: *mut media_stream,
) -> *mut stream_response {
    let mut response: *mut stream_response = ::core::ptr::null_mut::<stream_response>();
    let mut sample: *mut IMFSample = ::core::ptr::null_mut::<IMFSample>();
    let mut hr: HRESULT = 0;
    response = ((*reader).responses.next as *mut ::core::ffi::c_char).offset(
        -(core::mem::offset_of!(stream_response, entry) as ::core::ffi::c_ulonglong
            as isize),
    ) as *mut stream_response;
    while &raw mut (*response).entry != &raw mut (*reader).responses {
        if !stream.is_null() && (*response).stream_index != (*stream).index as DWORD {
            response = ((*response).entry.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(stream_response, entry)
                    as ::core::ffi::c_ulonglong as isize),
            ) as *mut stream_response;
        } else {
            if stream.is_null() && (*response).stream_index < (*reader).stream_count {
                stream = (*reader).streams.offset((*response).stream_index as isize)
                    as *mut media_stream;
            }
            if !(*response).sample.is_null() && !(*stream).allocator.is_null() {
                hr = IMFVideoSampleAllocatorEx_AllocateSample((*stream).allocator, &raw mut sample);
                if hr >= 0 as HRESULT {
                    source_reader_copy_sample_buffer((*response).sample, sample);
                    IMFSample_Release((*response).sample);
                    (*response).sample = sample;
                } else {
                    response = calloc(
                        (1 as size_t) as _,
                        (::core::mem::size_of::<stream_response>() as size_t) as _,
                    ) as *mut stream_response;
                    if response.is_null() {
                        return ::core::ptr::null_mut::<stream_response>();
                    }
                    (*response).status = hr;
                    (*response).stream_flags = MF_SOURCE_READERF_ERROR as DWORD;
                    return response;
                }
            }
            return media_stream_detach_response(reader, response);
        }
    }
    return ::core::ptr::null_mut::<stream_response>();
}

unsafe extern "C" fn source_reader_release_response(response: *mut stream_response) {
    if !(*response).sample.is_null() {
        IMFSample_Release((*response).sample);
    }
    free(response as *mut core::ffi::c_void);
}

unsafe extern "C" fn source_reader_get_stream_selection(reader: *const source_reader, index: DWORD, selected: *mut BOOL) -> HRESULT {
    let mut sd: *mut IMFStreamDescriptor = core::ptr::null_mut();

    if (IMFPresentationDescriptor_GetStreamDescriptorByIndex((*reader).descriptor, index, selected, &raw mut sd)) < 0 {
        return MF_E_INVALIDSTREAMNUMBER;
    }
    IMFStreamDescriptor_Release(sd);

    return S_OK;
}

unsafe extern "C" fn source_reader_start_source(reader: *mut source_reader) -> HRESULT {
    let mut selected: BOOL = 0;
    let mut selection_changed: BOOL = (FALSE as i32);
    let mut position: PROPVARIANT = core::mem::zeroed();
    let mut hr: HRESULT = S_OK;
    let mut i: ::core::ffi::c_uint;

    i = 0 as ::core::ffi::c_uint;
    while (i as DWORD) < (*reader).stream_count {
        source_reader_get_stream_selection(reader, i as DWORD, &mut selected);
        if selected != 0 {
            (*(*reader).streams.offset(i as isize)).flags |=
                (STREAM_FLAG_SELECTED as ::core::ffi::c_int) as ::core::ffi::c_uint;
        } else {
            (*(*reader).streams.offset(i as isize)).flags &=
                !((STREAM_FLAG_SELECTED as ::core::ffi::c_int) as ::core::ffi::c_uint);
        }
        i = i.wrapping_add(1);
    }

    if ((*reader).source_state as ::core::ffi::c_uint)
        == (SOURCE_STATE_STARTED as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        i = 0 as ::core::ffi::c_uint;
        while (i as DWORD) < (*reader).stream_count {
            selection_changed = ((((*(*reader).streams.offset(i as isize)).flags
                & (STREAM_FLAG_SELECTED as ::core::ffi::c_int as ::core::ffi::c_uint))
                != 0) as ::core::ffi::c_int
                ^ (((*(*reader).streams.offset(i as isize)).flags
                    & (STREAM_FLAG_PRESENTED as ::core::ffi::c_int as ::core::ffi::c_uint))
                    != 0) as ::core::ffi::c_int) as BOOL;
            if selection_changed != 0 {
                break;
            }
            i = i.wrapping_add(1);
        }
    }

    position.__bindgen_anon_1.hVal.QuadPart = 0 as LONGLONG;
    if ((*reader).source_state as ::core::ffi::c_uint)
        != (SOURCE_STATE_STARTED as ::core::ffi::c_int as ::core::ffi::c_uint)
        || selection_changed != 0
    {
        position.vt = ((if ((*reader).source_state as ::core::ffi::c_uint)
            == (SOURCE_STATE_STARTED as ::core::ffi::c_int as ::core::ffi::c_uint)
        {
            VT_EMPTY as ::core::ffi::c_int
        } else {
            VT_I8 as ::core::ffi::c_int
        }) as VARTYPE);

        /* Update cached stream selection if descriptor was accepted. */
        hr = IMFMediaSource_Start(
            (*reader).source,
            (*reader).descriptor,
            &GUID_NULL,
            &mut position,
        );
        if hr >= 0 {
            i = 0 as ::core::ffi::c_uint;
            while (i as DWORD) < (*reader).stream_count {
                if (*(*reader).streams.offset(i as isize)).flags
                    & (STREAM_FLAG_SELECTED as ::core::ffi::c_int as ::core::ffi::c_uint)
                    != 0
                {
                    (*(*reader).streams.offset(i as isize)).flags |=
                        (STREAM_FLAG_PRESENTED as ::core::ffi::c_int) as ::core::ffi::c_uint;
                }
                i = i.wrapping_add(1);
            }
        }
    }

    return hr;
}

unsafe extern "C" fn source_reader_got_response_for_stream(reader: *mut source_reader, stream: *mut media_stream) -> BOOL {
    let mut response: *mut stream_response = core::ptr::null_mut::<stream_response>();
    response = ((*reader).responses.next as *mut u8)
        .sub(core::mem::offset_of!(stream_response, entry)) as *mut stream_response;
    while core::ptr::addr_of_mut!((*response).entry) != core::ptr::addr_of_mut!((*reader).responses) {
        if (*response).stream_index == (*stream).index as DWORD {
            return (TRUE) as BOOL;
        }
        response = ((*response).entry.next as *mut u8)
            .sub(core::mem::offset_of!(stream_response, entry)) as *mut stream_response;
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn source_reader_get_read_result(
    mut reader: *mut source_reader,
    mut stream: *mut media_stream,
    mut flags: DWORD,
    mut status: *mut HRESULT,
    mut stream_index: *mut DWORD,
    mut stream_flags: *mut DWORD,
    mut timestamp: *mut LONGLONG,
    mut sample: *mut *mut IMFSample,
) -> BOOL {
    let mut response: *mut stream_response = core::ptr::null_mut::<stream_response>();
    let mut request_sample: BOOL = (FALSE as i32);
    let mut ptr: *mut list = core::ptr::null_mut::<list>();

    response = media_stream_pop_response(reader, stream);
    if !response.is_null() {
        *status = (*response).status;
        *stream_index = (*stream).index as DWORD;
        *stream_flags = (*response).stream_flags;
        *timestamp = (*response).timestamp;
        *sample = (*response).sample;
        if !(*sample).is_null() {
            IMFSample_AddRef(*sample);
            if (*stream).state as ::core::ffi::c_uint
                == STREAM_STATE_EOS as ::core::ffi::c_uint
                && {
                    ptr = list_head(core::ptr::addr_of_mut!((*stream).transforms));
                    !ptr.is_null()
                }
            {
                let mut entry: *mut transform_entry = (ptr as *mut ::core::ffi::c_char).offset(
                    -(core::mem::offset_of!(transform_entry, entry) as isize),
                ) as *mut transform_entry;
                /* Try to drain another sample in case the decoder emits many while draining.
                 * Draining one at a time prevents exhaustion of the allocator in some cases. */
                source_reader_drain_transform_samples(reader, stream, entry);
            }
        }

        source_reader_release_response(response);
    } else {
        *status = S_OK;
        *stream_index = (*stream).index as DWORD;
        *timestamp = 0;
        *sample = core::ptr::null_mut::<IMFSample>();

        if (*stream).state as ::core::ffi::c_uint
            == STREAM_STATE_EOS as ::core::ffi::c_uint
        {
            *stream_flags = MF_SOURCE_READERF_ENDOFSTREAM;
        } else {
            request_sample = ((flags & MF_SOURCE_READER_CONTROLF_DRAIN) == 0) as ::core::ffi::c_int as BOOL;
            *stream_flags = 0;
        }
    }

    return (request_sample == 0) as ::core::ffi::c_int;
}

unsafe extern "C" fn source_reader_get_next_selected_stream(
    mut reader: *mut source_reader,
    mut stream_index: *mut DWORD,
) -> HRESULT {
    let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut first_selected: ::core::ffi::c_uint = !(0 as ::core::ffi::c_uint);
    let mut selected: BOOL = 0;
    let mut stream_drained: BOOL = 0;
    let mut min_ts: LONGLONG = (MAXLONGLONG as _);
    i = 0 as ::core::ffi::c_uint;
    while (i as DWORD) < (*reader).stream_count {
        stream_drained = ((*(*reader).streams.offset(i as isize)).state as ::core::ffi::c_uint
            == STREAM_STATE_EOS as ::core::ffi::c_uint
            && (*(*reader).streams.offset(i as isize)).responses == 0)
            as ::core::ffi::c_int as BOOL;
        selected = (source_reader_get_stream_selection(reader, i as DWORD, &raw mut selected)
            >= 0 as HRESULT
            && selected != 0) as ::core::ffi::c_int as BOOL;
        if selected != 0 {
            if first_selected == !(0 as ::core::ffi::c_uint) {
                first_selected = i;
            }
            if stream_drained == 0
                && (*(*reader).streams.offset(i as isize)).last_sample_ts < min_ts
            {
                min_ts = (*(*reader).streams.offset(i as isize)).last_sample_ts;
                *stream_index = i as DWORD;
            }
        }
        i = i.wrapping_add(1);
    }
    if first_selected != !(0 as ::core::ffi::c_uint) {
        if min_ts == (MAXLONGLONG as _) {
            *stream_index = first_selected as DWORD;
        }
    }
    return if first_selected == !(0 as ::core::ffi::c_uint) {
        MF_E_MEDIA_SOURCE_NO_STREAMS_SELECTED
    } else {
        S_OK
    };
}

unsafe extern "C" fn source_reader_get_stream_read_index(reader: *mut source_reader, index: ::core::ffi::c_uint, stream_index: *mut DWORD) -> HRESULT {
    let mut selected: BOOL = 0;
    let mut hr: HRESULT = 0;

    match index {
        MF_SOURCE_READER_FIRST_VIDEO_STREAM => {
            *stream_index = (*reader).first_video_stream_index as DWORD;
        }
        MF_SOURCE_READER_FIRST_AUDIO_STREAM => {
            *stream_index = (*reader).first_audio_stream_index as DWORD;
        }
        MF_SOURCE_READER_ANY_STREAM => {
            return source_reader_get_next_selected_stream(reader, stream_index);
        }
        _ => {
            *stream_index = index as DWORD;
        }
    }

    /* Can't read from deselected streams. */
    hr = source_reader_get_stream_selection(reader, *stream_index, core::ptr::addr_of_mut!(selected));
    if hr >= 0 && selected == 0 {
        hr = MF_E_INVALIDREQUEST;
    }

    return hr;
}

unsafe extern "C" fn source_reader_release_responses(reader: *mut source_reader, stream: *mut media_stream) {
    let mut ptr: *mut stream_response = ::core::ptr::null_mut::<stream_response>();
    let mut next: *mut stream_response = ::core::ptr::null_mut::<stream_response>();
    ptr = ((*reader).responses.next as *mut ::core::ffi::c_char).offset(
        -(core::mem::offset_of!(stream_response, entry) as isize),
    ) as *mut stream_response;
    next = ((*ptr).entry.next as *mut ::core::ffi::c_char).offset(
        -(core::mem::offset_of!(stream_response, entry) as isize),
    ) as *mut stream_response;
    while &raw mut (*ptr).entry != &raw mut (*reader).responses {
        if !(!stream.is_null()
            && (*stream).index as DWORD != (*ptr).stream_index
            && (*ptr).stream_index != MF_SOURCE_READER_FIRST_VIDEO_STREAM as DWORD
            && (*ptr).stream_index != MF_SOURCE_READER_FIRST_AUDIO_STREAM as DWORD
            && (*ptr).stream_index != MF_SOURCE_READER_ANY_STREAM as DWORD)
        {
            media_stream_detach_response(reader, ptr);
            source_reader_release_response(ptr);
        }
        ptr = next;
        next = ((*ptr).entry.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(stream_response, entry) as isize),
        ) as *mut stream_response;
    }
}

unsafe extern "C" fn source_reader_flush_stream(reader: *mut source_reader, stream_index: DWORD) {
    let mut stream: *mut media_stream = (*reader).streams.offset(stream_index as isize);
    let mut ptr: *mut list = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    source_reader_release_responses(reader, stream);

    ptr = list_head(core::ptr::addr_of_mut!((*stream).transforms));
    if !ptr.is_null() {
        let mut entry: *mut transform_entry = (ptr as *mut core::ffi::c_char)
            .offset(-(core::mem::offset_of!(transform_entry, entry) as isize))
            as *mut transform_entry;
        hr = source_reader_flush_transform_samples(reader, stream, entry);
        if hr < 0 {
            WARN!("Failed to drain pending samples, hr %#lx.\n", hr);
        }
    }

    (*stream).requests = 0;
}

unsafe extern "C" fn source_reader_flush(
    mut reader: *mut source_reader,
    mut index: ::core::ffi::c_uint,
) -> HRESULT {
    let mut stream_index: ::core::ffi::c_uint;
    let mut hr: HRESULT = S_OK;
    if index == MF_SOURCE_READER_ALL_STREAMS {
        stream_index = 0 as ::core::ffi::c_uint;
        while (stream_index as DWORD) < (*reader).stream_count {
            source_reader_flush_stream(reader, stream_index as DWORD);
            stream_index = stream_index.wrapping_add(1);
        }
    } else {
        match index {
            MF_SOURCE_READER_FIRST_VIDEO_STREAM => {
                stream_index = (*reader).first_video_stream_index;
            }
            MF_SOURCE_READER_FIRST_AUDIO_STREAM => {
                stream_index = (*reader).first_audio_stream_index;
            }
            _ => {
                stream_index = index;
            }
        }
        if (stream_index as DWORD) < (*reader).stream_count {
            source_reader_flush_stream(reader, stream_index as DWORD);
        } else {
            hr = MF_E_INVALIDSTREAMNUMBER;
        }
    }
    return hr;
}
unsafe extern "system" fn source_reader_async_commands_callback_Invoke(
    mut iface: *mut IMFAsyncCallback,
    mut result: *mut IMFAsyncResult,
) -> HRESULT {
    let mut reader: *mut source_reader = impl_from_async_commands_callback_IMFAsyncCallback(iface);
    let mut stream: *mut media_stream = ::core::ptr::null_mut::<media_stream>();
    let mut stub_stream: media_stream = core::mem::zeroed::<media_stream>();
    stub_stream.requests = 1 as ::core::ffi::c_uint;
    let mut command: *mut source_reader_async_command =
        ::core::ptr::null_mut::<source_reader_async_command>();
    let mut response: *mut stream_response = ::core::ptr::null_mut::<stream_response>();
    let mut stream_index: DWORD = 0;
    let mut stream_flags: DWORD = 0;
    let mut report_sample: BOOL = (FALSE as i32);
    let mut sample: *mut IMFSample = ::core::ptr::null_mut::<IMFSample>();
    let mut timestamp: LONGLONG = 0 as LONGLONG;
    let mut hr: HRESULT = 0;
    let mut status: HRESULT = 0;
    let mut state: *mut IUnknown = ::core::ptr::null_mut::<IUnknown>();
    hr = IMFAsyncResult_GetState(result, &raw mut state);
    if hr < 0 as HRESULT {
        return hr;
    }
    command = impl_from_async_command_IUnknown(state);
    match ((*command).op as ::core::ffi::c_uint) as i32 {
        SOURCE_READER_ASYNC_READ => {
            EnterCriticalSection(&raw mut (*reader).cs);
            if {
                hr = source_reader_start_source(reader);
                hr
            } >= 0 as HRESULT
            {
                if {
                    hr = source_reader_get_stream_read_index(
                        reader,
                        (*command).u.read.stream_index,
                        &raw mut stream_index,
                    );
                    hr
                } >= 0 as HRESULT
                {
                    stream = (*reader).streams.offset(stream_index as isize) as *mut media_stream;
                    report_sample = source_reader_get_read_result(
                        reader,
                        stream,
                        (*command).u.read.flags as DWORD,
                        &raw mut status,
                        &raw mut stream_index,
                        &raw mut stream_flags,
                        &raw mut timestamp,
                        &raw mut sample,
                    );
                    if report_sample == 0 {
                        (*stream).requests = (*stream).requests.wrapping_add(1);
                        source_reader_request_sample(reader, stream);
                        /* FIXME: set error stream/reader state on request failure */
                    }
                } else {
                    stub_stream.index = (*command).u.read.stream_index;
                    if (stub_stream.index as DWORD) < (*reader).stream_count {
                        let ref mut fresh16 =
                            (*(*reader).streams.offset(stub_stream.index as isize)).requests;
                        *fresh16 = (*fresh16).wrapping_add(1);
                    }
                    if hr == MF_E_MEDIA_SOURCE_NO_STREAMS_SELECTED as HRESULT {
                        hr = MF_E_INVALIDREQUEST as HRESULT;
                    }
                    source_reader_queue_response(
                        reader,
                        &raw mut stub_stream,
                        hr,
                        MF_SOURCE_READERF_ERROR as DWORD,
                        0 as LONGLONG,
                        ::core::ptr::null_mut::<IMFSample>(),
                    );
                }
            }
            LeaveCriticalSection(&raw mut (*reader).cs);
            if report_sample != 0 {
                IMFSourceReaderCallback_OnReadSample(
                    (*reader).async_callback,
                    status,
                    stream_index,
                    stream_flags,
                    timestamp,
                    sample,
                );
            }
            if !sample.is_null() {
                IMFSample_Release(sample);
            }
        }
        SOURCE_READER_ASYNC_SEEK => {
            EnterCriticalSection(&raw mut (*reader).cs);
            if IMFMediaSource_Start(
                (*reader).source,
                (*reader).descriptor,
                &raw mut (*command).u.seek.format,
                &raw mut (*command).u.seek.position,
            ) < 0 as HRESULT
            {
                (*reader).flags &= !(SOURCE_READER_SEEKING as ::core::ffi::c_uint);
            }
            LeaveCriticalSection(&raw mut (*reader).cs);
        }
        SOURCE_READER_ASYNC_SAMPLE_READY => {
            EnterCriticalSection(&raw mut (*reader).cs);
            if ((*command).u.sample.stream_index as DWORD) < (*reader).stream_count {
                stream = (*reader)
                    .streams
                    .offset((*command).u.sample.stream_index as isize)
                    as *mut media_stream;
            }
            response = media_stream_pop_response(reader, stream);
            LeaveCriticalSection(&raw mut (*reader).cs);
            if !response.is_null() {
                IMFSourceReaderCallback_OnReadSample(
                    (*reader).async_callback,
                    (*response).status,
                    (*response).stream_index,
                    (*response).stream_flags,
                    (*response).timestamp,
                    (*response).sample,
                );
                source_reader_release_response(response);
            }
        }
        SOURCE_READER_ASYNC_FLUSH => {
            EnterCriticalSection(&raw mut (*reader).cs);
            source_reader_flush(reader, (*command).u.flush.stream_index);
            (*reader).flags &= !(SOURCE_READER_FLUSHING as ::core::ffi::c_uint);
            LeaveCriticalSection(&raw mut (*reader).cs);
            IMFSourceReaderCallback_OnFlush(
                (*reader).async_callback,
                (*command).u.flush.stream_index as DWORD,
            );
        }
        _ => {}
    }
    IUnknown_Release(state);
    return S_OK as HRESULT;
}

unsafe extern "system" fn src_reader_QueryInterface(iface: *mut IMFSourceReaderEx, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let reader: *mut source_reader = impl_from_IMFSourceReaderEx(iface);

    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) != false
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IMFSourceReader)) != false
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IMFSourceReaderEx)) != false
    {
        *out = core::ptr::addr_of_mut!((*reader).IMFSourceReaderEx_iface) as *mut core::ffi::c_void;
    } else {
        *out = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*out as *mut IUnknown);
    return S_OK;
}

unsafe extern "system" fn src_reader_AddRef(iface: *mut IMFSourceReaderEx) -> ULONG {
    let reader: *mut source_reader = impl_from_IMFSourceReaderEx(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*reader).public_refcount) as *mut LONG) as ULONG;

    TRACE!("%p, refcount %lu.\n", iface, refcount);

    return refcount;
}

unsafe extern "C" fn source_reader_is_source_stopped(reader: *const source_reader) -> BOOL {
    let mut i: ::core::ffi::c_uint = 0;
    if (*reader).source_state as ::core::ffi::c_uint
        != SOURCE_STATE_STOPPED as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return (FALSE) as BOOL;
    }
    i = 0 as ::core::ffi::c_uint;
    while (i as DWORD) < (*reader).stream_count {
        if !(*(*reader).streams.offset(i as isize)).stream.is_null()
            && ((*(*reader).streams.offset(i as isize)).flags
                & STREAM_FLAG_STOPPED as ::core::ffi::c_uint)
                == 0
        {
            return (FALSE) as BOOL;
        }
        i = i.wrapping_add(1);
    }
    return (TRUE) as BOOL;
}

unsafe extern "system" fn src_reader_Release(iface: *mut IMFSourceReaderEx) -> ULONG {
    let reader: *mut source_reader = impl_from_IMFSourceReaderEx(iface);
    let refcount: ULONG = InterlockedDecrement(&raw mut (*reader).public_refcount as *mut c_long) as ULONG;
    let mut i: ::core::ffi::c_uint = 0;

    if refcount == 0 {
        if (*reader).flags & SOURCE_READER_SHUTDOWN_ON_RELEASE != 0 {
            IMFMediaSource_Shutdown((*reader).source);
        } else if IMFMediaSource_Stop((*reader).source) >= 0 {
            EnterCriticalSection(&raw mut (*reader).cs);

            while source_reader_is_source_stopped(reader) == 0 {
                SleepConditionVariableCS(
                    &raw mut (*reader).stop_event,
                    &raw mut (*reader).cs,
                    INFINITE,
                );
            }

            LeaveCriticalSection(&raw mut (*reader).cs);
        }

        i = 0;
        while (i as DWORD) < (*reader).stream_count {
            let stream: *mut media_stream = (*reader).streams.offset(i as isize) as *mut media_stream;
            let mut callback: *mut IMFVideoSampleAllocatorCallback =
                ::core::ptr::null_mut::<IMFVideoSampleAllocatorCallback>();

            'body: {
                if (*stream).allocator.is_null() {
                    break 'body;
                }

                if IMFVideoSampleAllocatorEx_QueryInterface(
                    (*stream).allocator,
                    &raw const IID_IMFVideoSampleAllocatorCallback,
                    &raw mut callback as *mut *mut ::core::ffi::c_void,
                ) >= 0
                {
                    IMFVideoSampleAllocatorCallback_SetCallback(callback, ::core::ptr::null_mut());
                    IMFVideoSampleAllocatorCallback_Release(callback);
                }
            }
            i = i.wrapping_add(1);
        }

        MFUnlockWorkQueue((*reader).queue);
        source_reader_release(reader);
    }

    refcount
}

unsafe extern "system" fn src_reader_GetStreamSelection(iface: *mut IMFSourceReaderEx, mut index: DWORD, selected: *mut BOOL) -> HRESULT {
    let reader: *mut source_reader = impl_from_IMFSourceReaderEx(iface);

    match index {
        MF_SOURCE_READER_FIRST_VIDEO_STREAM => {
            index = (*reader).first_video_stream_index as DWORD;
        }
        MF_SOURCE_READER_FIRST_AUDIO_STREAM => {
            index = (*reader).first_audio_stream_index as DWORD;
        }
        _ => {}
    }

    return source_reader_get_stream_selection(reader, index, selected);
}

unsafe extern "system" fn src_reader_SetStreamSelection(iface: *mut IMFSourceReaderEx, index: DWORD, selection: BOOL) -> HRESULT {
    let mut index = index;
    let mut selection = selection;
    let reader: *mut source_reader = impl_from_IMFSourceReaderEx(iface);
    let mut hr: HRESULT = S_OK;
    let mut selection_changed: BOOL = (FALSE as i32);
    let mut selected: BOOL = 0;
    let mut i: ::core::ffi::c_uint;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfreadwrite/reader.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"src_reader_SetStreamSelection\0" as *const u8 as *const ::core::ffi::c_char,
            1775 as ::core::ffi::c_int,
            b"%p, %#lx, %d.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            index,
            selection,
        );
    }

    selection = (selection != 0) as ::core::ffi::c_int as BOOL;

    EnterCriticalSection(core::ptr::addr_of_mut!((*reader).cs));

    if index == MF_SOURCE_READER_ALL_STREAMS {
        i = 0;
        while (i as DWORD) < (*reader).stream_count {
            if selection_changed == 0 {
                source_reader_get_stream_selection(reader, i as DWORD, core::ptr::addr_of_mut!(selected));
                selection_changed = ((selected ^ selection) != 0) as ::core::ffi::c_int as BOOL;
            }

            if selection != 0 {
                IMFPresentationDescriptor_SelectStream((*reader).descriptor, i as DWORD);
            } else {
                IMFPresentationDescriptor_DeselectStream((*reader).descriptor, i as DWORD);
            }
            i = i.wrapping_add(1);
        }
    } else {
        match index {
            MF_SOURCE_READER_FIRST_VIDEO_STREAM => {
                index = (*reader).first_video_stream_index as DWORD;
            }
            MF_SOURCE_READER_FIRST_AUDIO_STREAM => {
                index = (*reader).first_audio_stream_index as DWORD;
            }
            _ => {}
        }

        source_reader_get_stream_selection(reader, index, core::ptr::addr_of_mut!(selected));
        selection_changed = ((selected ^ selection) != 0) as ::core::ffi::c_int as BOOL;

        if selection != 0 {
            hr = IMFPresentationDescriptor_SelectStream((*reader).descriptor, index);
        } else {
            hr = IMFPresentationDescriptor_DeselectStream((*reader).descriptor, index);
        }
    }

    if selection_changed != 0 {
        i = 0;
        while (i as DWORD) < (*reader).stream_count {
            (*(*reader).streams.offset(i as isize)).last_sample_ts = 0;
            i = i.wrapping_add(1);
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*reader).cs));

    return if hr >= 0 {
        S_OK
    } else {
        MF_E_INVALIDSTREAMNUMBER
    };
}

unsafe extern "C" fn source_reader_get_native_media_type(mut reader: *mut source_reader, mut index: DWORD, mut type_index: DWORD, mut type_0: *mut *mut IMFMediaType) -> HRESULT {
    let mut handler: *mut IMFMediaTypeHandler = core::ptr::null_mut::<IMFMediaTypeHandler>();
    let mut sd: *mut IMFStreamDescriptor = core::ptr::null_mut::<IMFStreamDescriptor>();
    let mut src_type: *mut IMFMediaType = core::ptr::null_mut::<IMFMediaType>();
    let mut selected: BOOL = 0;
    let mut hr: HRESULT = 0;

    match index {
        MF_SOURCE_READER_FIRST_VIDEO_STREAM => {
            index = (*reader).first_video_stream_index as DWORD;
        }
        MF_SOURCE_READER_FIRST_AUDIO_STREAM => {
            index = (*reader).first_audio_stream_index as DWORD;
        }
        _ => {}
    }

    if IMFPresentationDescriptor_GetStreamDescriptorByIndex((*reader).descriptor, index, core::ptr::addr_of_mut!(selected), core::ptr::addr_of_mut!(sd)) < 0 {
        return MF_E_INVALIDSTREAMNUMBER;
    }

    hr = IMFStreamDescriptor_GetMediaTypeHandler(sd, core::ptr::addr_of_mut!(handler));
    IMFStreamDescriptor_Release(sd);
    if hr < 0 {
        return hr;
    }

    if type_index == MF_SOURCE_READER_CURRENT_TYPE_INDEX {
        hr = IMFMediaTypeHandler_GetCurrentMediaType(handler, core::ptr::addr_of_mut!(src_type));
    } else {
        hr = IMFMediaTypeHandler_GetMediaTypeByIndex(handler, type_index, core::ptr::addr_of_mut!(src_type));
    }
    IMFMediaTypeHandler_Release(handler);

    if hr >= 0 {
        hr = MFCreateMediaType(type_0);
        if hr >= 0 {
            hr = IMFMediaType_CopyAllItems(src_type, *type_0 as *mut IMFAttributes);
        }
        IMFMediaType_Release(src_type);
    }

    return hr;
}
unsafe extern "system" fn src_reader_GetNativeMediaType(
    iface: *mut IMFSourceReaderEx,
    index: DWORD,
    type_index: DWORD,
    type_: *mut *mut IMFMediaType,
) -> HRESULT {
    let reader: *mut source_reader = impl_from_IMFSourceReaderEx(iface);

    TRACE!(
        "%p, %#lx, %#lx, %p.\n",
        iface,
        index,
        type_index,
        type_
    );

    return source_reader_get_native_media_type(reader, index, type_index, type_);
}

unsafe extern "system" fn src_reader_GetCurrentMediaType(iface: *mut IMFSourceReaderEx, mut index: DWORD, type_: *mut *mut IMFMediaType) -> HRESULT {
    let reader: *mut source_reader = impl_from_IMFSourceReaderEx(iface);
    let mut hr: HRESULT;

    if index == MF_SOURCE_READER_FIRST_VIDEO_STREAM {
        index = (*reader).first_video_stream_index as DWORD;
    } else if index == MF_SOURCE_READER_FIRST_AUDIO_STREAM {
        index = (*reader).first_audio_stream_index as DWORD;
    } else {
        ;
    }

    if index >= (*reader).stream_count {
        return MF_E_INVALIDSTREAMNUMBER;
    }

    hr = MFCreateMediaType(type_);
    if hr < 0 {
        return hr;
    }

    EnterCriticalSection(&raw mut (*reader).cs);

    hr = IMFMediaType_CopyAllItems(
        (*(*reader).streams.offset(index as isize)).current,
        *type_ as *mut IMFAttributes,
    );

    LeaveCriticalSection(&raw mut (*reader).cs);

    return hr;
}

unsafe extern "C" fn source_reader_get_source_type_handler(reader: *mut source_reader, index: DWORD, handler: *mut *mut IMFMediaTypeHandler) -> HRESULT {
    let mut sd: *mut IMFStreamDescriptor = core::ptr::null_mut();
    let mut selected: BOOL = 0;
    let mut hr: HRESULT = 0;

    hr = IMFPresentationDescriptor_GetStreamDescriptorByIndex((*reader).descriptor, index, core::ptr::addr_of_mut!(selected), core::ptr::addr_of_mut!(sd));
    if hr < 0 as HRESULT {
        return hr;
    }

    hr = IMFStreamDescriptor_GetMediaTypeHandler(sd, handler);
    IMFStreamDescriptor_Release(sd);

    return hr;
}

unsafe extern "C" fn source_reader_set_compatible_media_type(reader: *mut source_reader, index: DWORD, type_0: *mut IMFMediaType) -> HRESULT {
    let stream: *mut media_stream = (*reader).streams.offset(index as isize) as *mut media_stream;
    let mut entry: *mut transform_entry = core::ptr::null_mut::<transform_entry>();
    let mut next: *mut transform_entry = core::ptr::null_mut::<transform_entry>();
    let mut type_handler: *mut IMFMediaTypeHandler = core::ptr::null_mut::<IMFMediaTypeHandler>();
    let mut native_type: *mut IMFMediaType = core::ptr::null_mut::<IMFMediaType>();
    let mut type_set: BOOL = (FALSE as i32);
    let mut i: core::ffi::c_uint = 0;
    let mut flags: DWORD = 0;
    let mut hr: HRESULT = 0;

    hr = IMFMediaType_IsEqual(type_0, (*stream).current, core::ptr::addr_of_mut!(flags));
    if hr < 0 {
        return hr;
    }

    if !(flags & MF_MEDIATYPE_EQUAL_MAJOR_TYPES as DWORD != 0) {
        return MF_E_INVALIDMEDIATYPE;
    }

    /* No need for a decoder or type change. */
    if flags & MF_MEDIATYPE_EQUAL_FORMAT_DATA as DWORD != 0 {
        return S_OK;
    }

    if !(*stream).transform_service.is_null() {
        IMFTransform_Release((*stream).transform_service);
        (*stream).transform_service = core::ptr::null_mut::<IMFTransform>();
    }

    entry = ((*stream).transforms.next as *mut u8).offset(
        -(core::mem::offset_of!(transform_entry, entry) as isize),
    ) as *mut transform_entry;
    next = ((*entry).entry.next as *mut u8).offset(
        -(core::mem::offset_of!(transform_entry, entry) as isize),
    ) as *mut transform_entry;
    while core::ptr::addr_of_mut!((*entry).entry) != core::ptr::addr_of_mut!((*stream).transforms) {
        list_remove(core::ptr::addr_of_mut!((*entry).entry));
        transform_entry_destroy(entry);
        entry = next;
        next = ((*entry).entry.next as *mut u8).offset(
            -(core::mem::offset_of!(transform_entry, entry) as isize),
        ) as *mut transform_entry;
    }

    hr = source_reader_get_source_type_handler(reader, index, core::ptr::addr_of_mut!(type_handler));
    if hr < 0 {
        return hr;
    }

    while type_set == 0 && {
        let fresh21 = i;
        i = i.wrapping_add(1);
        IMFMediaTypeHandler_GetMediaTypeByIndex(type_handler, fresh21 as DWORD, core::ptr::addr_of_mut!(native_type))
            == S_OK
    } {
        let compare_flags: DWORD = MF_MEDIATYPE_EQUAL_MAJOR_TYPES as DWORD | MF_MEDIATYPE_EQUAL_FORMAT_DATA as DWORD;
        if IMFMediaType_IsEqual(native_type, type_0, core::ptr::addr_of_mut!(flags)) >= 0
            && (flags & compare_flags) == compare_flags
        {
            type_set = (IMFMediaTypeHandler_SetCurrentMediaType(type_handler, native_type) >= 0) as core::ffi::c_int as BOOL;
            if type_set != 0 {
                IMFMediaType_CopyAllItems(native_type, (*stream).current as *mut IMFAttributes);
            }
        }

        IMFMediaType_Release(native_type);
    }

    IMFMediaTypeHandler_Release(type_handler);

    return if type_set != 0 { S_OK } else { S_FALSE };
}

unsafe extern "C" fn source_reader_create_sample_allocator_attributes(reader: *const source_reader, attributes: *mut *mut IMFAttributes) -> HRESULT {
    let mut shared: UINT32 = 0;
    let mut shared_without_mutex: UINT32 = 0;
    let mut hr: HRESULT;

    hr = MFCreateAttributes(attributes, 1);
    if hr < 0 {
        return hr;
    }

    IMFAttributes_GetUINT32((*reader).attributes, core::ptr::addr_of!(MF_SA_D3D11_SHARED), core::ptr::addr_of_mut!(shared));
    IMFAttributes_GetUINT32((*reader).attributes, core::ptr::addr_of!(MF_SA_D3D11_SHARED_WITHOUT_MUTEX), core::ptr::addr_of_mut!(shared_without_mutex));

    if shared_without_mutex != 0 {
        hr = IMFAttributes_SetUINT32(*attributes, core::ptr::addr_of!(MF_SA_D3D11_SHARED_WITHOUT_MUTEX), TRUE);
    } else if shared != 0 {
        hr = IMFAttributes_SetUINT32(*attributes, core::ptr::addr_of!(MF_SA_D3D11_SHARED), TRUE);
    }

    return hr;
}

unsafe extern "C" fn source_reader_setup_sample_allocator(reader: *mut source_reader, index: core::ffi::c_uint) -> HRESULT {
    let stream: *mut media_stream = (*reader).streams.add(index as usize);
    let mut attributes: *mut IMFAttributes = core::ptr::null_mut();
    let mut major: GUID = core::mem::zeroed();
    let mut hr: HRESULT = 0;

    IMFMediaType_GetMajorType((*stream).current, &raw mut major);
    if IsEqualGUID!(&raw const major, &raw const MFMediaType_Video) == false {
        return S_OK;
    }

    if ((*reader).flags & SOURCE_READER_HAS_DEVICE_MANAGER) == 0 {
        return S_OK;
    }

    if (*stream).allocator.is_null() {
        hr = MFCreateVideoSampleAllocatorEx(
            &raw const IID_IMFVideoSampleAllocatorEx,
            &raw mut (*stream).allocator as *mut *mut core::ffi::c_void,
        );
        if hr < 0 {
            WARN!("Failed to create sample allocator, hr %#lx.\n", hr);
            return hr;
        }
    }

    IMFVideoSampleAllocatorEx_UninitializeSampleAllocator((*stream).allocator);
    hr = IMFVideoSampleAllocatorEx_SetDirectXManager((*stream).allocator, (*reader).device_manager);
    if hr < 0 {
        WARN!("Failed to set device manager, hr %#lx.\n", hr);
        return hr;
    }

    hr = source_reader_create_sample_allocator_attributes(reader, &raw mut attributes);
    if hr < 0 {
        WARN!("Failed to create allocator attributes, hr %#lx.\n", hr);
    }

    hr = IMFVideoSampleAllocatorEx_InitializeSampleAllocatorEx(
        (*stream).allocator,
        2,
        8,
        attributes,
        (*stream).current,
    );
    if hr < 0 {
        WARN!("Failed to initialize sample allocator, hr %#lx.\n", hr);
    }

    if !attributes.is_null() {
        IMFAttributes_Release(attributes);
    }

    return hr;
}

unsafe extern "C" fn source_reader_allow_video_processor(reader: *mut source_reader, advanced: *mut BOOL) -> BOOL {
    let mut value: UINT32 = 0;

    *advanced = FALSE as BOOL;
    if (*reader).attributes.is_null() {
        return FALSE as BOOL;
    }

    if ((IMFAttributes_GetUINT32((*reader).attributes, &raw const MF_SOURCE_READER_ENABLE_ADVANCED_VIDEO_PROCESSING, &raw mut value)) >= 0) {
        *advanced = value as BOOL;
    }
    if ((IMFAttributes_GetUINT32((*reader).attributes, &raw const MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING, &raw mut value)) >= 0) {
        return (value != 0 || *advanced != 0) as BOOL;
    }

    return *advanced;
}

unsafe extern "C" fn mediatype_set_uint32(
    mut mediatype: *mut IMFMediaType,
    mut attr: *const GUID,
    mut value: ::core::ffi::c_uint,
    mut hr: *mut HRESULT,
) {
    if *hr >= 0 {
        *hr = IMFMediaType_SetUINT32(mediatype, attr, value as UINT32);
    }
}

unsafe extern "C" fn mediatype_get_stride_and_sample_size(mediatype: *mut IMFMediaType, stride: *mut LONG, sample_size: *mut DWORD, hr: *mut HRESULT) {
    let mut frame_size: UINT64 = 0;
    let mut subtype: GUID = core::mem::zeroed();

    if *hr >= 0 {
        *hr = IMFMediaType_GetGUID(mediatype, core::ptr::addr_of!(MF_MT_SUBTYPE), core::ptr::addr_of_mut!(subtype));
    }

    if *hr >= 0 {
        *hr = IMFMediaType_GetUINT64(mediatype, core::ptr::addr_of!(MF_MT_FRAME_SIZE), core::ptr::addr_of_mut!(frame_size));
    }

    if *hr >= 0 {
        *hr = MFGetStrideForBitmapInfoHeader(subtype.Data1 as DWORD, (frame_size >> 32) as DWORD, stride);
    }

    if *hr >= 0 {
        *hr = MFGetPlaneSize(subtype.Data1 as DWORD, (frame_size >> 32) as DWORD, (frame_size & 0xffffffff) as DWORD, sample_size);
    }
}

unsafe extern "C" fn set_default_video_attributes(reader: *mut source_reader, output_type: *mut IMFMediaType) -> HRESULT {
    let mut sample_size: DWORD = 0;
    let mut compressed: BOOL = 0;
    let mut stride: LONG = 0;
    let mut hr: HRESULT = 0;

    hr = IMFMediaType_IsCompressedFormat(output_type, &raw mut compressed);
    if hr < 0 {
        return hr;
    }

    if compressed == 0 {
        mediatype_get_stride_and_sample_size(output_type, &raw mut stride, &raw mut sample_size, &raw mut hr);

        mediatype_set_uint32(output_type, &raw const MF_MT_COMPRESSED, compressed as ::core::ffi::c_uint, &raw mut hr);
        mediatype_set_uint32(output_type, &raw const MF_MT_DEFAULT_STRIDE, abs(stride as ::core::ffi::c_int) as ::core::ffi::c_uint, &raw mut hr);
        mediatype_set_uint32(output_type, &raw const MF_MT_SAMPLE_SIZE, sample_size as ::core::ffi::c_uint, &raw mut hr);
    }

    return hr;
}

unsafe extern "C" fn apply_pixel_aspect_ratio(dst_type: *mut IMFMediaType, src_type: *mut IMFMediaType) -> HRESULT {
    let mut dst_frame_size: UINT64 = 0;
    let mut par_w: UINT32 = 0;
    let mut par_h: UINT32 = 0;
    let mut w: UINT32 = 0;
    let mut h: UINT32 = 0;
    if media_type_get_uint32_pair(src_type, &raw const MF_MT_FRAME_SIZE, &raw mut w, &raw mut h) >= 0
        && IMFMediaType_GetUINT64(dst_type, &raw const MF_MT_FRAME_SIZE, &raw mut dst_frame_size) < 0
        && media_type_get_uint32_pair(src_type, &raw const MF_MT_PIXEL_ASPECT_RATIO, &raw mut par_w, &raw mut par_h) >= 0
    {
        /* Video processor should rectify the aspect ratio so the output has aspect ratio of 1:1.
         * We are doing this here by explicitly changing the target frame size. */

        let mut new_h: UINT64 = h as UINT64;
        let mut new_w: UINT64 = w as UINT64;
        if par_w < par_h {
            new_h = h.wrapping_mul(par_h).wrapping_div(par_w) as UINT64;
        } else {
            new_w = w.wrapping_mul(par_w).wrapping_div(par_h) as UINT64;
        }

        new_h = new_h.wrapping_add(1) & !1;
        new_w = new_w.wrapping_add(1) & !1;

        return IMFMediaType_SetUINT64(
            dst_type,
            &raw const MF_MT_FRAME_SIZE,
            (new_w << 32).wrapping_add(new_h),
        );
    }
    return S_OK;
}

unsafe extern "C" fn source_reader_create_transform(
    reader: *mut source_reader,
    decoder: BOOL,
    allow_processor: BOOL,
    input_type: *mut IMFMediaType,
    output_type: *mut IMFMediaType,
    out: *mut *mut transform_entry,
) -> HRESULT {
    let mut in_type: MFT_REGISTER_TYPE_INFO = core::mem::zeroed();
    let mut out_type: MFT_REGISTER_TYPE_INFO = core::mem::zeroed();
    let mut entry: *mut transform_entry = ::core::ptr::null_mut::<transform_entry>();
    let mut activates: *mut *mut IMFActivate = ::core::ptr::null_mut::<*mut IMFActivate>();
    let mut category: GUID = core::mem::zeroed();
    let mut transform: *mut IMFTransform = ::core::ptr::null_mut::<IMFTransform>();
    let mut i: UINT = 0;
    let mut count: UINT = 0;
    let mut hr: HRESULT = 0;

    hr = IMFMediaType_GetMajorType(input_type, &raw mut in_type.guidMajorType);
    if hr < 0
        || {
            hr = IMFMediaType_GetGUID(
                input_type,
                &raw const MF_MT_SUBTYPE,
                &raw mut in_type.guidSubtype,
            );
            hr < 0
        }
    {
        return hr;
    }
    hr = IMFMediaType_GetMajorType(output_type, &raw mut out_type.guidMajorType);
    if hr < 0
        || {
            hr = IMFMediaType_GetGUID(
                output_type,
                &raw const MF_MT_SUBTYPE,
                &raw mut out_type.guidSubtype,
            );
            hr < 0
        }
    {
        return hr;
    }

    if IsEqualGUID!(&raw const out_type.guidMajorType, &raw const MFMediaType_Video) != false {
        category = if decoder != 0 {
            MFT_CATEGORY_VIDEO_DECODER
        } else {
            MFT_CATEGORY_VIDEO_PROCESSOR
        };
    } else if IsEqualGUID!(&raw const out_type.guidMajorType, &raw const MFMediaType_Audio) != false {
        category = if decoder != 0 {
            MFT_CATEGORY_AUDIO_DECODER
        } else {
            MFT_CATEGORY_AUDIO_EFFECT
        };
    } else {
        return MF_E_TOPO_CODEC_NOT_FOUND;
    }

    entry = calloc(
        1,
        (::core::mem::size_of::<transform_entry>() as size_t) as _,
    ) as *mut transform_entry;
    if entry.is_null() {
        return E_OUTOFMEMORY;
    }
    list_init(&raw mut (*entry).entry);
    (*entry).category = category;

    if IsEqualGUID!(&raw const out_type.guidMajorType, &raw const MFMediaType_Audio) != false
        && IMFMediaType_GetUINT32(
            output_type,
            &raw const MF_MT_AUDIO_BLOCK_ALIGNMENT,
            &raw mut (*entry).min_buffer_size,
        ) >= 0
    {
        let mut bytes_per_second: UINT32 = 0;

        if IMFMediaType_GetUINT32(
            output_type,
            &raw const MF_MT_AUDIO_AVG_BYTES_PER_SECOND,
            &raw mut bytes_per_second,
        ) >= 0
        {
            (*entry).min_buffer_size = max!((*entry).min_buffer_size, bytes_per_second);
        }
    }

    if IsEqualGUID!(&raw const out_type.guidMajorType, &raw const MFMediaType_Video) != false
        && IsEqualGUID!(&raw const out_type.guidSubtype, &raw const MFVideoFormat_ABGR32) != false
        && IsEqualGUID!(&raw const category, &raw const MFT_CATEGORY_VIDEO_PROCESSOR) != false
    {
        /* The video processor isn't registered for MFVideoFormat_ABGR32, and native even only supports that format when
         * D3D-enabled, we still want to instantiate a video processor in such case, so fixup the subtype for MFTEnumEx.
         */
        WARN!("Fixing up MFVideoFormat_ABGR32 subtype for the video processor\n");
        out_type.guidSubtype = MFVideoFormat_RGB32;
    }

    count = 0;
    hr = MFTEnumEx(
        category,
        0,
        &raw mut in_type,
        if allow_processor != 0 {
            ::core::ptr::null_mut::<MFT_REGISTER_TYPE_INFO>()
        } else {
            &raw mut out_type
        },
        &raw mut activates,
        &raw mut count,
    );
    if hr >= 0 {
        if count == 0 {
            free(entry as *mut ::core::ffi::c_void);
            return MF_E_TOPO_CODEC_NOT_FOUND;
        }

        i = 0;
        while i < count {
            let mut attributes: *mut IMFAttributes = ::core::ptr::null_mut::<IMFAttributes>();
            let mut media_type: *mut IMFMediaType = ::core::ptr::null_mut::<IMFMediaType>();

            hr = IMFActivate_ActivateObject(
                *activates.offset(i as isize),
                &raw const IID_IMFTransform,
                &raw mut transform as *mut *mut ::core::ffi::c_void,
            );
            if !(hr < 0) {
                if (*reader).device_manager.is_null()
                    || IMFTransform_GetAttributes(transform, &raw mut attributes) < 0
                {
                    (*entry).attributes_initialized = (TRUE as i32);
                } else {
                    let mut d3d_aware: UINT32 = FALSE;

                    if (*reader).flags & SOURCE_READER_DXGI_DEVICE_MANAGER as ::core::ffi::c_uint != 0 {
                        if IMFAttributes_GetUINT32(
                            attributes,
                            &raw const MF_SA_D3D11_AWARE,
                            &raw mut d3d_aware,
                        ) >= 0
                            && d3d_aware != 0
                        {
                            IMFTransform_ProcessMessage(
                                transform,
                                MFT_MESSAGE_SET_D3D_MANAGER,
                                (*reader).device_manager as ULONG_PTR,
                            );
                        }
                    } else if (*reader).flags & SOURCE_READER_D3D9_DEVICE_MANAGER as ::core::ffi::c_uint != 0 {
                        if IMFAttributes_GetUINT32(
                            attributes,
                            &raw const MF_SA_D3D_AWARE,
                            &raw mut d3d_aware,
                        ) >= 0
                            && d3d_aware != 0
                        {
                            IMFTransform_ProcessMessage(
                                transform,
                                MFT_MESSAGE_SET_D3D_MANAGER,
                                (*reader).device_manager as ULONG_PTR,
                            );
                        }
                    }

                    (*entry).attributes_initialized = (d3d_aware == 0) as ::core::ffi::c_int as BOOL;
                    IMFAttributes_Release(attributes);
                }

                if IMFTransform_SetInputType(
                    transform,
                    0,
                    input_type,
                    0,
                ) >= 0
                    && {
                        hr = IMFTransform_GetInputCurrentType(
                            transform,
                            0,
                            &raw mut media_type,
                        );
                        hr >= 0
                    }
                {
                    let mut enable_advanced: BOOL = 0;
                    let mut output_type_copy: *mut IMFMediaType =
                        ::core::ptr::null_mut::<IMFMediaType>();

                    hr = MFCreateMediaType(&raw mut output_type_copy);
                    if hr >= 0 {
                        hr = IMFMediaType_CopyAllItems(
                            output_type,
                            output_type_copy as *mut IMFAttributes,
                        );
                    }

                    source_reader_allow_video_processor(reader, &raw mut enable_advanced);

                    if hr >= 0
                        && (enable_advanced == 0 || {
                            hr = apply_pixel_aspect_ratio(output_type_copy, media_type);
                            hr >= 0
                        })
                        && {
                            hr = update_media_type(
                                output_type_copy,
                                media_type,
                                enable_advanced,
                            );
                            hr >= 0
                        }
                        && {
                            hr = IMFTransform_SetOutputType(
                                transform,
                                0,
                                output_type_copy,
                                0,
                            );
                            hr < 0
                        }
                        && {
                            hr = set_matching_transform_output_type(transform, output_type_copy);
                            hr < 0
                        }
                        && allow_processor != 0
                        && {
                            hr = IMFTransform_GetOutputAvailableType(
                                transform,
                                0,
                                0,
                                &raw mut media_type,
                            );
                            hr >= 0
                        }
                    {
                        let mut converter: *mut transform_entry =
                            ::core::ptr::null_mut::<transform_entry>();

                        if {
                            hr = IMFTransform_SetOutputType(
                                transform,
                                0,
                                media_type,
                                0,
                            );
                            hr >= 0
                        } && (enable_advanced == 0 || {
                            hr = apply_pixel_aspect_ratio(output_type_copy, media_type);
                            hr >= 0
                        }) && {
                            hr = update_media_type(
                                output_type_copy,
                                media_type,
                                enable_advanced,
                            );
                            hr >= 0
                        } && (enable_advanced != 0 || {
                            hr = set_default_video_attributes(reader, output_type_copy);
                            hr >= 0
                        }) && {
                            hr = source_reader_create_transform(
                                reader,
                                (FALSE) as BOOL,
                                (FALSE) as BOOL,
                                media_type,
                                output_type_copy,
                                &raw mut converter,
                            );
                            hr >= 0
                        } {
                            list_add_tail(
                                &raw mut (*entry).entry,
                                &raw mut (*converter).entry,
                            );
                        }

                        IMFMediaType_Release(media_type);
                    }

                    if !output_type_copy.is_null() {
                        IMFMediaType_Release(output_type_copy);
                    }

                    if hr >= 0 {
                        (*entry).transform = transform;
                        *out = entry;
                        break;
                    }
                }

                IMFTransform_Release(transform);
            }
            i = i.wrapping_add(1);
        }

        i = 0;
        while i < count {
            IMFActivate_Release(*activates.offset(i as isize));
            i = i.wrapping_add(1);
        }
        CoTaskMemFree(activates as LPVOID);
    }

    if hr < 0 {
        free(entry as *mut ::core::ffi::c_void);
    }
    return hr;
}

unsafe extern "C" fn source_reader_create_decoder_for_stream(reader: *mut source_reader, index: DWORD, mut output_type: *mut IMFMediaType) -> HRESULT {
    let mut enable_advanced: BOOL = (FALSE as i32);
    let mut allow_processor: BOOL = (TRUE as i32);
    let stream: *mut media_stream = (*reader).streams.offset(index as isize);
    let mut input_type: *mut IMFMediaType = core::ptr::null_mut();
    let mut i: u32 = 0;
    let mut major: GUID = GUID { Data1: 0, Data2: 0, Data3: 0, Data4: [0; 8] };
    let mut hr: HRESULT;

    if (IMFMediaType_GetMajorType(output_type, core::ptr::addr_of_mut!(major)) >= 0)
        && IsEqualGUID!(core::ptr::addr_of_mut!(major), core::ptr::addr_of!(MFMediaType_Video)) != false
    {
        allow_processor = source_reader_allow_video_processor(reader, core::ptr::addr_of_mut!(enable_advanced));
    }

    loop {
        let fresh20 = i;
        i = i.wrapping_add(1);
        hr = source_reader_get_native_media_type(reader, index, fresh20 as DWORD, core::ptr::addr_of_mut!(input_type));
        if !(hr >= 0) {
            break;
        }

        let mut entry: *mut transform_entry = core::ptr::null_mut();

        /* first, try to append a single processor, then try again with a decoder and a processor */
        if (allow_processor != 0 && {
            hr = source_reader_create_transform(reader, (FALSE) as BOOL, (FALSE) as BOOL, input_type, output_type, core::ptr::addr_of_mut!(entry));
            hr >= 0
        }) || {
            hr = source_reader_create_transform(reader, (TRUE) as BOOL, allow_processor, input_type, output_type, core::ptr::addr_of_mut!(entry));
            hr >= 0
        } {
            let ptr: *mut list = list_head(core::ptr::addr_of_mut!((*entry).entry));
            let service: *mut transform_entry = if !ptr.is_null() {
                (ptr as *mut u8).sub(core::mem::offset_of!(transform_entry, entry)) as *mut transform_entry
            } else {
                entry
            };
            let mut type_handler: *mut IMFMediaTypeHandler = core::ptr::null_mut();

            if enable_advanced != 0 {
                /* when advanced video processing is enabled, converters are exposed as stream transform service */
                (*stream).transform_service = (*service).transform;
                IMFTransform_AddRef((*stream).transform_service);
            } else {
                /* when advanced video processing is disabled, only decoders are exposed as stream transform service */
                if IsEqualGUID!(core::ptr::addr_of_mut!((*entry).category), core::ptr::addr_of!(MFT_CATEGORY_AUDIO_DECODER)) != false
                    || IsEqualGUID!(core::ptr::addr_of_mut!((*entry).category), core::ptr::addr_of!(MFT_CATEGORY_VIDEO_DECODER)) != false
                {
                    (*stream).transform_service = (*entry).transform;
                    IMFTransform_AddRef((*stream).transform_service);

                    /* converters are hidden from the stream transforms */
                    if service != entry {
                        (*service).hidden = (TRUE as i32);
                    }
                } else {
                    /* converters are hidden from the stream transforms */
                    (*entry).hidden = (TRUE as i32);
                }
            }

            /* move any additional transforms that have been created */
            list_move_head(core::ptr::addr_of_mut!((*stream).transforms), core::ptr::addr_of_mut!((*entry).entry));
            list_add_head(core::ptr::addr_of_mut!((*stream).transforms), core::ptr::addr_of_mut!((*entry).entry));

            if source_reader_get_source_type_handler(reader, index, core::ptr::addr_of_mut!(type_handler)) >= 0 {
                if ({ hr = IMFMediaTypeHandler_SetCurrentMediaType(type_handler, input_type); hr }) < 0 {
                    WARN!("Failed to set current input media type, hr %#lx\n", hr);
                }
                IMFMediaTypeHandler_Release(type_handler);
            }

            if ({ hr = IMFTransform_GetOutputCurrentType((*service).transform, 0, core::ptr::addr_of_mut!(output_type)); hr }) < 0 {
                WARN!("Failed to get decoder output media type, hr %#lx\n", hr);
            } else {
                IMFMediaType_CopyAllItems(output_type, (*stream).current as *mut IMFAttributes);
                IMFMediaType_Release(output_type);
            }

            IMFMediaType_Release(input_type);
            return S_OK;
        }

        IMFMediaType_Release(input_type);
    }

    return MF_E_TOPO_CODEC_NOT_FOUND;
}

unsafe extern "system" fn src_reader_SetCurrentMediaType(
    mut iface: *mut IMFSourceReaderEx,
    mut index: DWORD,
    mut reserved: *mut DWORD,
    mut type_0: *mut IMFMediaType,
) -> HRESULT {
    let mut reader: *mut source_reader = impl_from_IMFSourceReaderEx(iface);
    let mut hr: HRESULT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfreadwrite/reader.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"src_reader_SetCurrentMediaType\0" as *const u8 as *const ::core::ffi::c_char,
            2369 as ::core::ffi::c_int,
            b"%p, %#lx, %p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            index,
            reserved,
            type_0,
        );
    }
    match index {
        MF_SOURCE_READER_FIRST_VIDEO_STREAM => {
            index = (*reader).first_video_stream_index as DWORD;
        }
        MF_SOURCE_READER_FIRST_AUDIO_STREAM => {
            index = (*reader).first_audio_stream_index as DWORD;
        }
        _ => {}
    }
    if index >= (*reader).stream_count {
        return MF_E_INVALIDSTREAMNUMBER;
    }
    EnterCriticalSection(core::ptr::addr_of_mut!((*reader).cs));
    hr = source_reader_set_compatible_media_type(reader, index, type_0);
    if hr == S_FALSE {
        hr = source_reader_create_decoder_for_stream(reader, index, type_0);
    }
    if hr >= 0 {
        hr = source_reader_setup_sample_allocator(reader, index as ::core::ffi::c_uint);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*reader).cs));
    return hr;
}

unsafe extern "system" fn src_reader_SetCurrentPosition(iface: *mut IMFSourceReaderEx, format: *const GUID, position: *const PROPVARIANT) -> HRESULT {
    let reader: *mut source_reader = impl_from_IMFSourceReaderEx(iface);
    let mut command: *mut source_reader_async_command = core::ptr::null_mut();
    let mut i: u32 = 0;
    let mut flags: DWORD = 0;
    let mut hr: HRESULT = 0;

    TRACE!("%p, %s, %p.\n", iface, debugstr_guid(format), position);

    hr = IMFMediaSource_GetCharacteristics((*reader).source, core::ptr::addr_of_mut!(flags));
    if hr < 0 {
        return hr;
    }

    if (flags & MFMEDIASOURCE_CAN_SEEK) == 0 {
        return MF_E_INVALIDREQUEST;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*reader).cs));

    /* Check if we got pending requests. */
    i = 0;
    while (i as DWORD) < (*reader).stream_count {
        if (*(*reader).streams.offset(i as isize)).requests != 0 {
            hr = MF_E_INVALIDREQUEST;
            break;
        }
        i += 1;
    }

    if hr >= 0 {
        (*reader).flags |= SOURCE_READER_SEEKING;

        i = 0;
        while (i as DWORD) < (*reader).stream_count {
            (*(*reader).streams.offset(i as isize)).last_sample_ts = 0;
            i += 1;
        }

        if !(*reader).async_callback.is_null() {
            hr = source_reader_create_async_op(SOURCE_READER_ASYNC_SEEK, core::ptr::addr_of_mut!(command));
            if hr >= 0 {
                (*command).u.seek.format = *format;
                PropVariantCopy(core::ptr::addr_of_mut!((*command).u.seek.position), position);

                hr = MFPutWorkItem(
                    (*reader).queue,
                    core::ptr::addr_of_mut!((*reader).async_commands_callback),
                    core::ptr::addr_of_mut!((*command).IUnknown_iface),
                );
                IUnknown_Release(core::ptr::addr_of_mut!((*command).IUnknown_iface));
            }
        } else {
            if IMFMediaSource_Start((*reader).source, (*reader).descriptor, format, position) >= 0 {
                while ((*reader).flags & SOURCE_READER_SEEKING) != 0 {
                    SleepConditionVariableCS(
                        core::ptr::addr_of_mut!((*reader).state_event),
                        core::ptr::addr_of_mut!((*reader).cs),
                        INFINITE,
                    );
                }
            }
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*reader).cs));

    return hr;
}

unsafe extern "C" fn source_reader_read_sample(
    mut reader: *mut source_reader,
    mut index: DWORD,
    mut flags: DWORD,
    mut actual_index: *mut DWORD,
    mut stream_flags: *mut DWORD,
    mut timestamp: *mut LONGLONG,
    mut sample: *mut *mut IMFSample,
) -> HRESULT {
    let mut stream: *mut media_stream = core::ptr::null_mut::<media_stream>();
    let mut actual_index_tmp: DWORD = 0;
    let mut timestamp_tmp: LONGLONG = 0;
    let mut stream_index: DWORD = 0;
    let mut hr: HRESULT = S_OK;

    if stream_flags.is_null() || sample.is_null() {
        return E_POINTER;
    }

    *sample = core::ptr::null_mut::<IMFSample>();

    if timestamp.is_null() {
        timestamp = core::ptr::addr_of_mut!(timestamp_tmp);
    }

    if actual_index.is_null() {
        actual_index = core::ptr::addr_of_mut!(actual_index_tmp);
    }

    hr = source_reader_start_source(reader);
    if hr >= 0 {
        hr = source_reader_get_stream_read_index(reader, index, core::ptr::addr_of_mut!(stream_index));
        if hr >= 0 {
            *actual_index = stream_index;

            stream = (*reader).streams.offset(stream_index as isize) as *mut media_stream;

            if source_reader_get_read_result(
                reader,
                stream,
                flags,
                core::ptr::addr_of_mut!(hr),
                actual_index,
                stream_flags,
                timestamp,
                sample,
            ) == 0
            {
                while source_reader_got_response_for_stream(reader, stream) == 0
                    && (*stream).state as ::core::ffi::c_uint
                        != (STREAM_STATE_EOS as ::core::ffi::c_int) as ::core::ffi::c_uint
                {
                    (*stream).requests = (*stream).requests.wrapping_add(1);
                    hr = source_reader_request_sample(reader, stream);
                    if hr < 0 {
                        WARN!("Failed to request a sample, hr %#lx.\n", hr);
                    }
                    if !(*stream).stream.is_null()
                        && ((*stream).flags & (STREAM_FLAG_SAMPLE_REQUESTED as ::core::ffi::c_uint)) == 0
                    {
                        *stream_flags = MF_SOURCE_READERF_ERROR;
                        *timestamp = 0;
                        break;
                    }
                    SleepConditionVariableCS(
                        core::ptr::addr_of_mut!((*reader).sample_event),
                        core::ptr::addr_of_mut!((*reader).cs),
                        INFINITE,
                    );
                }
                if hr >= 0 {
                    source_reader_get_read_result(
                        reader,
                        stream,
                        flags,
                        core::ptr::addr_of_mut!(hr),
                        actual_index,
                        stream_flags,
                        timestamp,
                        sample,
                    );
                }
            }
        } else {
            *actual_index = index;
            *stream_flags = MF_SOURCE_READERF_ERROR;
            *timestamp = 0;
        }
    }

    TRACE!(
        "Stream %lu, got sample %p, flags %#lx.\n",
        *actual_index,
        *sample,
        *stream_flags
    );

    return hr;
}

unsafe extern "C" fn source_reader_read_sample_async(reader: *mut source_reader, index: ::core::ffi::c_uint, flags: ::core::ffi::c_uint, actual_index: *mut DWORD, stream_flags: *mut DWORD, timestamp: *mut LONGLONG, sample: *mut *mut IMFSample) -> HRESULT {
    let mut command: *mut source_reader_async_command = core::ptr::null_mut();
    let mut hr: HRESULT;

    if !actual_index.is_null() || !stream_flags.is_null() || !timestamp.is_null() || !sample.is_null() {
        return E_INVALIDARG;
    }

    if ((*reader).flags & SOURCE_READER_FLUSHING as ::core::ffi::c_uint) != 0 {
        hr = MF_E_NOTACCEPTING;
    } else {
        hr = source_reader_create_async_op(SOURCE_READER_ASYNC_READ, &raw mut command);
        if hr >= 0 {
            (*command).u.read.stream_index = index;
            (*command).u.read.flags = flags;

            hr = MFPutWorkItem(
                (*reader).queue,
                &raw mut (*reader).async_commands_callback,
                &raw mut (*command).IUnknown_iface,
            );
            IUnknown_Release(&raw mut (*command).IUnknown_iface);
        }
    }

    return hr;
}

unsafe extern "system" fn src_reader_ReadSample(iface: *mut IMFSourceReaderEx, index: DWORD, flags: DWORD, actual_index: *mut DWORD, stream_flags: *mut DWORD, timestamp: *mut LONGLONG, sample: *mut *mut IMFSample) -> HRESULT {
    let reader: *mut source_reader = impl_from_IMFSourceReaderEx(iface);
    let mut hr: HRESULT = 0;

    TRACE!("%p, %#lx, %#lx, %p, %p, %p, %p\n", iface, index, flags, actual_index, stream_flags, timestamp, sample);

    EnterCriticalSection(core::ptr::addr_of_mut!((*reader).cs));

    while ((*reader).flags & SOURCE_READER_SEEKING) != 0
    {
        SleepConditionVariableCS(core::ptr::addr_of_mut!((*reader).state_event), core::ptr::addr_of_mut!((*reader).cs), INFINITE as DWORD);
    }

    if !(*reader).async_callback.is_null() {
        hr = source_reader_read_sample_async(reader, index as u32, flags as u32, actual_index, stream_flags, timestamp, sample);
    } else {
        hr = source_reader_read_sample(reader, index, flags, actual_index, stream_flags, timestamp, sample);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*reader).cs));

    return hr;
}

unsafe extern "C" fn source_reader_flush_async(reader: *mut source_reader, index: ::core::ffi::c_uint) -> HRESULT {
    let mut command: *mut source_reader_async_command = core::ptr::null_mut();
    let mut stream_index: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;

    if ((*reader).flags & SOURCE_READER_FLUSHING) != 0 {
        return MF_E_INVALIDREQUEST;
    }

    if index == MF_SOURCE_READER_FIRST_VIDEO_STREAM {
        stream_index = (*reader).first_video_stream_index;
    } else if index == MF_SOURCE_READER_FIRST_AUDIO_STREAM {
        stream_index = (*reader).first_audio_stream_index;
    } else {
        stream_index = index;
    }

    (*reader).flags |= SOURCE_READER_FLUSHING;

    if stream_index != MF_SOURCE_READER_ALL_STREAMS && stream_index >= (*reader).stream_count {
        return MF_E_INVALIDSTREAMNUMBER;
    }

    hr = source_reader_create_async_op(SOURCE_READER_ASYNC_FLUSH, core::ptr::addr_of_mut!(command));
    if hr < 0 {
        return hr;
    }

    (*command).u.flush.stream_index = stream_index;

    hr = MFPutWorkItem(
        (*reader).queue,
        core::ptr::addr_of_mut!((*reader).async_commands_callback),
        core::ptr::addr_of_mut!((*command).IUnknown_iface),
    );
    IUnknown_Release(core::ptr::addr_of_mut!((*command).IUnknown_iface));

    return hr;
}

unsafe extern "system" fn src_reader_Flush(iface: *mut IMFSourceReaderEx, index: DWORD) -> HRESULT {
    let reader: *mut source_reader = impl_from_IMFSourceReaderEx(iface);
    let mut hr: HRESULT;

    EnterCriticalSection(&raw mut (*reader).cs);

    if !(*reader).async_callback.is_null() {
        hr = source_reader_flush_async(reader, index);
    } else {
        hr = source_reader_flush(reader, index);
    }

    LeaveCriticalSection(&raw mut (*reader).cs);

    return hr;
}

unsafe extern "system" fn src_reader_GetServiceForStream(iface: *mut IMFSourceReaderEx, mut index: DWORD, service: REFGUID, riid: REFIID, object: *mut *mut core::ffi::c_void) -> HRESULT {
    let reader: *mut source_reader = impl_from_IMFSourceReaderEx(iface);
    let mut obj: *mut IUnknown = core::ptr::null_mut();
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(&raw mut (*reader).cs);

    match index {
        MF_SOURCE_READER_MEDIASOURCE => {
            obj = (*reader).source as *mut IUnknown;
        }
        _ => {
            if index == MF_SOURCE_READER_FIRST_VIDEO_STREAM {
                index = (*reader).first_video_stream_index as DWORD;
            } else if index == MF_SOURCE_READER_FIRST_AUDIO_STREAM {
                index = (*reader).first_audio_stream_index as DWORD;
            }

            if index >= (*reader).stream_count {
                hr = MF_E_INVALIDSTREAMNUMBER;
            } else {
                obj = (*(*reader).streams.offset(index as isize)).transform_service as *mut IUnknown;
                if obj.is_null() {
                    hr = E_NOINTERFACE;
                }
            }
        }
    }

    if !obj.is_null() {
        IUnknown_AddRef(obj);
    }

    LeaveCriticalSection(&raw mut (*reader).cs);

    if !obj.is_null() {
        if IsEqualGUID!(service, &GUID_NULL) {
            hr = IUnknown_QueryInterface(obj, riid, object);
        } else {
            let mut gs: *mut IMFGetService = core::ptr::null_mut();

            hr = IUnknown_QueryInterface(obj, &raw const IID_IMFGetService, &raw mut gs as *mut *mut core::ffi::c_void);
            if hr >= 0 {
                hr = IMFGetService_GetService(gs, service, riid, object);
                IMFGetService_Release(gs);
            }
        }
    }

    if !obj.is_null() {
        IUnknown_Release(obj);
    }

    hr
}

unsafe extern "system" fn src_reader_GetPresentationAttribute(
    iface: *mut IMFSourceReaderEx,
    mut index: DWORD,
    guid: *const GUID,
    value: *mut PROPVARIANT,
) -> HRESULT {
    let reader: *mut source_reader = impl_from_IMFSourceReaderEx(iface);
    let mut sd: *mut IMFStreamDescriptor = core::ptr::null_mut();
    let mut selected: BOOL = 0;
    let mut hr: HRESULT;

    match index {
        MF_SOURCE_READER_MEDIASOURCE => {
            if IsEqualGUID!(guid, &raw const MF_SOURCE_READER_MEDIASOURCE_CHARACTERISTICS) != false {
                let mut flags: DWORD = 0;

                hr = IMFMediaSource_GetCharacteristics((*reader).source, &raw mut flags);
                if hr < 0 {
                    return hr;
                }

                (*value).vt = VT_UI4 as VARTYPE;
                (*value).__bindgen_anon_1.ulVal = flags as ULONG;
                return S_OK;
            } else {
                return IMFPresentationDescriptor_GetItem((*reader).descriptor, guid, value);
            }
        }
        MF_SOURCE_READER_FIRST_VIDEO_STREAM => {
            index = (*reader).first_video_stream_index as DWORD;
        }
        MF_SOURCE_READER_FIRST_AUDIO_STREAM => {
            index = (*reader).first_audio_stream_index as DWORD;
        }
        _ => {}
    }

    hr = IMFPresentationDescriptor_GetStreamDescriptorByIndex(
        (*reader).descriptor,
        index,
        &raw mut selected,
        &raw mut sd,
    );
    if hr < 0 {
        return hr;
    }

    hr = IMFStreamDescriptor_GetItem(sd, guid, value);
    IMFStreamDescriptor_Release(sd);

    return hr;
}
unsafe extern "system" fn src_reader_SetNativeMediaType(
    iface: *mut IMFSourceReaderEx,
    stream_index: DWORD,
    media_type: *mut IMFMediaType,
    stream_flags: *mut DWORD,
) -> HRESULT {
    // FIXME("%p, %#lx, %p, %p.\n", iface, stream_index, media_type, stream_flags);
    E_NOTIMPL
}
unsafe extern "system" fn src_reader_AddTransformForStream(
    iface: *mut IMFSourceReaderEx,
    stream_index: DWORD,
    transform: *mut IUnknown,
) -> HRESULT {
    // FIXME("%p, %#lx, %p.\n", iface, stream_index, transform);
    E_NOTIMPL
}
unsafe extern "system" fn src_reader_RemoveAllTransformsForStream(
    iface: *mut IMFSourceReaderEx,
    stream_index: DWORD,
) -> HRESULT {
    // FIXME("%p, %#lx.\n", iface, stream_index);
    E_NOTIMPL
}

unsafe extern "C" fn get_transform_at_index(stream: *mut media_stream, mut index: UINT) -> *mut transform_entry {
    let mut entry: *mut transform_entry;

    entry = ((*stream).transforms.next as *mut u8).offset(
        -(core::mem::offset_of!(transform_entry, entry) as isize),
    ) as *mut transform_entry;
    while !(core::ptr::addr_of_mut!((*entry).entry) == core::ptr::addr_of_mut!((*stream).transforms)) {
        if (*entry).hidden == 0 && {
            let fresh19 = index;
            index = index.wrapping_sub(1);
            fresh19 == 0
        } {
            return entry;
        }
        entry = ((*entry).entry.next as *mut u8).offset(
            -(core::mem::offset_of!(transform_entry, entry) as isize),
        ) as *mut transform_entry;
    }
    return core::ptr::null_mut();
}

unsafe extern "system" fn src_reader_GetTransformForStream(
    iface: *mut IMFSourceReaderEx,
    mut stream_index: DWORD,
    mut transform_index: DWORD,
    category: *mut GUID,
    transform: *mut *mut IMFTransform,
) -> HRESULT {
    let reader: *mut source_reader = impl_from_IMFSourceReaderEx(iface);
    let mut entry: *mut transform_entry = core::ptr::null_mut();
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
            b"dll/win32/mfreadwrite/reader.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"src_reader_GetTransformForStream\0" as *const u8 as *const core::ffi::c_char,
            2783 as core::ffi::c_int,
            b"%p, %#lx, %#lx, %p, %p.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            stream_index,
            transform_index,
            category,
            transform,
        );
    }

    if transform.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*reader).cs));

    if stream_index == MF_SOURCE_READER_FIRST_VIDEO_STREAM as DWORD {
        stream_index = (*reader).first_video_stream_index as DWORD;
    } else if stream_index == MF_SOURCE_READER_FIRST_AUDIO_STREAM as DWORD {
        stream_index = (*reader).first_audio_stream_index as DWORD;
    }

    if stream_index >= (*reader).stream_count {
        hr = MF_E_INVALIDSTREAMNUMBER;
    } else {
        entry = get_transform_at_index(
            (*reader).streams.offset(stream_index as isize) as *mut media_stream,
            transform_index as UINT,
        );
        if entry.is_null() {
            hr = MF_E_INVALIDINDEX;
        } else {
            if !category.is_null() {
                *category = (*entry).category;
            }
            *transform = (*entry).transform;
            IMFTransform_AddRef(*transform);
            hr = S_OK;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*reader).cs));

    return hr;
}

unsafe extern "C" fn reader_get_first_stream_index(descriptor: *mut IMFPresentationDescriptor, major: *const GUID) -> DWORD {
    let mut count: DWORD = 0;
    let mut i: DWORD = 0;
    let mut selected: BOOL = 0;
    let mut hr: HRESULT = 0;
    let mut guid: GUID = core::mem::zeroed();

    if IMFPresentationDescriptor_GetStreamDescriptorCount(descriptor, core::ptr::addr_of_mut!(count)) < (0 as HRESULT) {
        return MF_SOURCE_READER_INVALID_STREAM_INDEX as DWORD;
    }

    i = 0 as DWORD;
    while i < count {
        'body: {
            let mut handler: *mut IMFMediaTypeHandler = core::ptr::null_mut();
            let mut sd: *mut IMFStreamDescriptor = core::ptr::null_mut();

            if IMFPresentationDescriptor_GetStreamDescriptorByIndex(descriptor, i, core::ptr::addr_of_mut!(selected), core::ptr::addr_of_mut!(sd)) >= (0 as HRESULT) {
                hr = IMFStreamDescriptor_GetMediaTypeHandler(sd, core::ptr::addr_of_mut!(handler));
                IMFStreamDescriptor_Release(sd);
                if hr >= (0 as HRESULT) {
                    hr = IMFMediaTypeHandler_GetMajorType(handler, core::ptr::addr_of_mut!(guid));
                    IMFMediaTypeHandler_Release(handler);
                    if hr < (0 as HRESULT) {
                        WARN!("Failed to get stream major type, hr %#lx.\n", hr);
                        break 'body;
                    }

                    if IsEqualGUID!(core::ptr::addr_of!(guid), major) != false {
                        return i;
                    }
                }
            }
        }
        i = i.wrapping_add(1);
    }

    return MF_SOURCE_READER_INVALID_STREAM_INDEX as DWORD;
}

unsafe extern "C" fn create_source_reader_from_source(
    source: *mut IMFMediaSource,
    attributes: *mut IMFAttributes,
    shutdown_on_release: BOOL,
    riid: REFIID,
    out: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut object: *mut source_reader = core::ptr::null_mut();
    let mut i: core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;

    object = calloc(1, (core::mem::size_of::<source_reader>() as size_t) as _) as *mut source_reader;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).IMFSourceReaderEx_iface.lpVtbl = &raw const srcreader_vtbl;
    (*object).source_events_callback.lpVtbl = &raw const source_events_callback_vtbl;
    (*object).stream_events_callback.lpVtbl = &raw const stream_events_callback_vtbl;
    (*object).async_commands_callback.lpVtbl = &raw const async_commands_callback_vtbl;
    (*object).public_refcount = 1;
    (*object).refcount = 1;
    list_init(&raw mut (*object).responses);
    if shutdown_on_release != 0 {
        (*object).flags |= SOURCE_READER_SHUTDOWN_ON_RELEASE;
    }
    (*object).source = source;
    IMFMediaSource_AddRef((*object).source);
    InitializeCriticalSection(&raw mut (*object).cs);
    InitializeConditionVariable(&raw mut (*object).sample_event);
    InitializeConditionVariable(&raw mut (*object).state_event);
    InitializeConditionVariable(&raw mut (*object).stop_event);

    'failed: {
        hr = IMFMediaSource_CreatePresentationDescriptor((*object).source, &raw mut (*object).descriptor);
        if hr < 0 {
            break 'failed;
        }

        hr = IMFPresentationDescriptor_GetStreamDescriptorCount((*object).descriptor, &raw mut (*object).stream_count);
        if hr < 0 {
            break 'failed;
        }

        (*object).streams = calloc(((*object).stream_count as size_t) as _, (core::mem::size_of::<media_stream>() as size_t) as _) as *mut media_stream;
        if (*object).streams.is_null() {
            hr = E_OUTOFMEMORY;
            break 'failed;
        }

        /* Set initial current media types. */
        i = 0;
        while i < (*object).stream_count {
            let mut handler: *mut IMFMediaTypeHandler = core::ptr::null_mut();
            let mut sd: *mut IMFStreamDescriptor = core::ptr::null_mut();
            let mut src_type: *mut IMFMediaType = core::ptr::null_mut();
            let mut selected: BOOL = 0;

            list_init(&raw mut (*(*object).streams.offset(i as isize)).transforms);

            hr = MFCreateMediaType(&raw mut (*(*object).streams.offset(i as isize)).current);
            if hr < 0 {
                break;
            }

            hr = IMFPresentationDescriptor_GetStreamDescriptorByIndex((*object).descriptor, i, &raw mut selected, &raw mut sd);
            if hr < 0 {
                break;
            }

            hr = IMFStreamDescriptor_GetStreamIdentifier(sd, &raw mut (*(*object).streams.offset(i as isize)).id);
            if hr < 0 {
                WARN!("Failed to get stream identifier, hr %#lx.\n", hr);
            }

            hr = IMFStreamDescriptor_GetMediaTypeHandler(sd, &raw mut handler);
            IMFStreamDescriptor_Release(sd);
            if hr < 0 {
                break;
            }

            hr = IMFMediaTypeHandler_GetMediaTypeByIndex(handler, 0, &raw mut src_type);
            IMFMediaTypeHandler_Release(handler);
            if hr < 0 {
                break;
            }

            hr = IMFMediaType_CopyAllItems(src_type, (*(*object).streams.offset(i as isize)).current as *mut IMFAttributes);
            IMFMediaType_Release(src_type);
            if hr < 0 {
                break;
            }

            (*(*object).streams.offset(i as isize)).reader = object;
            (*(*object).streams.offset(i as isize)).index = i;

            i = i.wrapping_add(1);
        }

        if hr < 0 {
            break 'failed;
        }

        /* At least one major type has to be set. */
        (*object).first_audio_stream_index = reader_get_first_stream_index((*object).descriptor, &raw const MFMediaType_Audio) as core::ffi::c_uint;
        (*object).first_video_stream_index = reader_get_first_stream_index((*object).descriptor, &raw const MFMediaType_Video) as core::ffi::c_uint;

        if (*object).first_audio_stream_index == MF_SOURCE_READER_INVALID_STREAM_INDEX
            && (*object).first_video_stream_index == MF_SOURCE_READER_INVALID_STREAM_INDEX
        {
            hr = MF_E_ATTRIBUTENOTFOUND;
        }

        hr = IMFMediaSource_BeginGetEvent(
            (*object).source,
            &raw mut (*object).source_events_callback,
            (*object).source as *mut IUnknown,
        );
        if hr < 0 {
            break 'failed;
        }

        if !attributes.is_null() {
            (*object).attributes = attributes;
            IMFAttributes_AddRef((*object).attributes);

            IMFAttributes_GetUnknown(
                attributes,
                &raw const MF_SOURCE_READER_ASYNC_CALLBACK,
                &raw const IID_IMFSourceReaderCallback,
                &raw mut (*object).async_callback as *mut LPVOID,
            );
            if !(*object).async_callback.is_null() {
                TRACE!("Using async callback %p.\n", (*object).async_callback);
            }

            IMFAttributes_GetUnknown(
                attributes,
                &raw const MF_SOURCE_READER_D3D_MANAGER,
                &raw const IID_IUnknown,
                &raw mut (*object).device_manager as *mut LPVOID,
            );
            if !(*object).device_manager.is_null() {
                let mut unk: *mut IUnknown = core::ptr::null_mut();

                if IUnknown_QueryInterface((*object).device_manager, &raw const IID_IMFDXGIDeviceManager, &raw mut unk as *mut *mut core::ffi::c_void) >= 0 {
                    (*object).flags |= SOURCE_READER_DXGI_DEVICE_MANAGER;
                } else if IUnknown_QueryInterface((*object).device_manager, &raw const IID_IDirect3DDeviceManager9, &raw mut unk as *mut *mut core::ffi::c_void) >= 0 {
                    (*object).flags |= SOURCE_READER_D3D9_DEVICE_MANAGER;
                }

                if ((*object).flags & (SOURCE_READER_HAS_DEVICE_MANAGER)) == 0 {
                    WARN!("Unknown device manager.\n");
                    IUnknown_Release((*object).device_manager);
                    (*object).device_manager = core::ptr::null_mut();
                }

                if !unk.is_null() {
                    IUnknown_Release(unk);
                }
            }
        }

        hr = MFLockSharedWorkQueue(RUSTCONV_WSTR_96A296D224F2.as_ptr(), 0, core::ptr::null_mut(), &raw mut (*object).queue);
        if hr < 0 {
            WARN!("Failed to acquired shared queue, hr %#lx.\n", hr);
        }

        if hr >= 0 {
            hr = IMFSourceReaderEx_QueryInterface(&raw mut (*object).IMFSourceReaderEx_iface, riid, out);
        }
    }

    // failed:
    IMFSourceReaderEx_Release(&raw mut (*object).IMFSourceReaderEx_iface);
    return hr;
}

unsafe extern "C" fn create_source_reader_from_stream(
    mut stream: *mut IMFByteStream,
    mut attributes: *mut IMFAttributes,
    riid: *const IID,
    mut out: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut props: *mut IPropertyStore = ::core::ptr::null_mut::<IPropertyStore>();
    let mut resolver: *mut IMFSourceResolver = ::core::ptr::null_mut::<IMFSourceResolver>();
    let mut obj_type: MF_OBJECT_TYPE = MF_OBJECT_MEDIASOURCE;
    let mut source: *mut IMFMediaSource = ::core::ptr::null_mut::<IMFMediaSource>();
    let mut hr: HRESULT = 0;
    hr = MFCreateSourceResolver(&raw mut resolver);
    if hr < 0 {
        return hr;
    }
    if !attributes.is_null() {
        IMFAttributes_GetUnknown(
            attributes,
            &raw const MF_SOURCE_READER_MEDIASOURCE_CONFIG,
            &raw const IID_IPropertyStore,
            &raw mut props as *mut LPVOID,
        );
    }
    hr = IMFSourceResolver_CreateObjectFromByteStream(
        resolver,
        stream,
        ::core::ptr::null::<WCHAR>(),
        (MF_RESOLUTION_MEDIASOURCE as ::core::ffi::c_int
            | MF_RESOLUTION_CONTENT_DOES_NOT_HAVE_TO_MATCH_EXTENSION_OR_MIME_TYPE as ::core::ffi::c_int) as DWORD,
        props,
        &raw mut obj_type,
        &raw mut source as *mut *mut IUnknown,
    );
    IMFSourceResolver_Release(resolver);
    if !props.is_null() {
        IPropertyStore_Release(props);
    }
    if hr < 0 {
        return hr;
    }
    hr = create_source_reader_from_source(source, attributes, (TRUE) as BOOL, riid, out);
    IMFMediaSource_Release(source);
    return hr;
}

unsafe extern "C" fn create_source_reader_from_url(url: *const WCHAR, attributes: *mut IMFAttributes, riid: *const IID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut props: *mut IPropertyStore = core::ptr::null_mut();
    let mut resolver: *mut IMFSourceResolver = core::mem::zeroed();
    let mut object: *mut IUnknown = core::ptr::null_mut();
    let mut obj_type: MF_OBJECT_TYPE = 0;
    let mut source: *mut IMFMediaSource = core::ptr::null_mut();
    let mut hr: HRESULT;

    hr = MFCreateSourceResolver(core::ptr::addr_of_mut!(resolver));
    if hr < 0 {
        return hr;
    }

    if !attributes.is_null() {
        IMFAttributes_GetUnknown(
            attributes,
            core::ptr::addr_of!(MF_SOURCE_READER_MEDIASOURCE_CONFIG),
            core::ptr::addr_of!(IID_IPropertyStore),
            core::ptr::addr_of_mut!(props) as *mut LPVOID,
        );
    }

    hr = IMFSourceResolver_CreateObjectFromURL(
        resolver,
        url,
        MF_RESOLUTION_MEDIASOURCE,
        props,
        core::ptr::addr_of_mut!(obj_type),
        core::ptr::addr_of_mut!(object),
    );
    if hr >= 0 {
        match obj_type {
            MF_OBJECT_BYTESTREAM => {
                hr = IMFSourceResolver_CreateObjectFromByteStream(
                    resolver,
                    object as *mut IMFByteStream,
                    core::ptr::null::<WCHAR>(),
                    MF_RESOLUTION_MEDIASOURCE,
                    props,
                    core::ptr::addr_of_mut!(obj_type),
                    core::ptr::addr_of_mut!(source) as *mut *mut IUnknown,
                );
            }
            MF_OBJECT_MEDIASOURCE => {
                source = object as *mut IMFMediaSource;
                IMFMediaSource_AddRef(source);
            }
            _ => {
                WARN!("Unknown object type %d.\n", obj_type);
                hr = E_UNEXPECTED;
            }
        }
        IUnknown_Release(object);
    }

    IMFSourceResolver_Release(resolver);
    if !props.is_null() {
        IPropertyStore_Release(props);
    }
    if hr < 0 {
        return hr;
    }

    hr = create_source_reader_from_source(source, attributes, (TRUE) as BOOL, riid, out);
    IMFMediaSource_Release(source);
    return hr;
}

unsafe extern "C" fn create_source_reader_from_object(unk: *mut IUnknown, attributes: *mut IMFAttributes, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut source: *mut IMFMediaSource = core::ptr::null_mut();
    let mut stream: *mut IMFByteStream = core::ptr::null_mut();
    let mut hr: HRESULT;

    hr = IUnknown_QueryInterface(unk, core::ptr::addr_of!(IID_IMFMediaSource), &mut source as *mut *mut IMFMediaSource as *mut *mut core::ffi::c_void);
    if hr < 0 {
        hr = IUnknown_QueryInterface(unk, core::ptr::addr_of!(IID_IMFByteStream), &mut stream as *mut *mut IMFByteStream as *mut *mut core::ffi::c_void);
    }

    if !source.is_null() {
        let mut disconnect: UINT32 = 0;

        if !attributes.is_null() {
            IMFAttributes_GetUINT32(attributes, core::ptr::addr_of!(MF_SOURCE_READER_DISCONNECT_MEDIASOURCE_ON_SHUTDOWN), &mut disconnect as *mut UINT32);
        }
        hr = create_source_reader_from_source(source, attributes, (disconnect == 0) as core::ffi::c_int, riid, out);
    } else if !stream.is_null() {
        hr = create_source_reader_from_stream(stream, attributes, riid, out);
    }

    if !source.is_null() {
        IMFMediaSource_Release(source);
    }
    if !stream.is_null() {
        IMFByteStream_Release(stream);
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateSourceReaderFromByteStream(
    stream: *mut IMFByteStream,
    attributes: *mut IMFAttributes,
    reader: *mut *mut IMFSourceReader,
) -> HRESULT {
    return create_source_reader_from_object(
        stream as *mut IUnknown,
        attributes,
        core::ptr::addr_of!(IID_IMFSourceReader) as *const GUID,
        reader as *mut *mut core::ffi::c_void,
    );
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateSourceReaderFromMediaSource(
    source: *mut IMFMediaSource,
    attributes: *mut IMFAttributes,
    reader: *mut *mut IMFSourceReader,
) -> HRESULT {
    return create_source_reader_from_object(
        source as *mut IUnknown,
        attributes,
        core::ptr::addr_of!(IID_IMFSourceReader),
        reader as *mut *mut core::ffi::c_void,
    );
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateSourceReaderFromURL(url: *const WCHAR, attributes: *mut IMFAttributes, reader: *mut *mut IMFSourceReader) -> HRESULT {
    TRACE!("%s, %p, %p.\n", debugstr_w(url), attributes, reader);

    return create_source_reader_from_url(url, attributes, core::ptr::addr_of!(IID_IMFSourceReader), reader as *mut *mut core::ffi::c_void);
}

unsafe extern "system" fn readwrite_factory_QueryInterface(iface: *mut IMFReadWriteClassFactory, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFReadWriteClassFactory)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *out = iface as *mut core::ffi::c_void;
        IMFReadWriteClassFactory_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}
unsafe extern "system" fn readwrite_factory_AddRef(
    iface: *mut IMFReadWriteClassFactory,
) -> ULONG {
    2
}
unsafe extern "system" fn readwrite_factory_Release(
    iface: *mut IMFReadWriteClassFactory,
) -> ULONG {
    1
}

unsafe extern "system" fn readwrite_factory_CreateInstanceFromURL(iface: *mut IMFReadWriteClassFactory, clsid: *const GUID, url: *const u16, attributes: *mut IMFAttributes, riid: *const GUID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualGUID!(clsid, core::ptr::addr_of!(CLSID_MFSourceReader)) != false {
        return create_source_reader_from_url(url, attributes, core::ptr::addr_of!(IID_IMFSourceReader), out);
    } else if IsEqualGUID!(clsid, core::ptr::addr_of!(CLSID_MFSinkWriter)) != false {
        return create_sink_writer_from_url(url, core::ptr::null_mut(), attributes, riid, out);
    }

    return E_NOTIMPL;
}

unsafe extern "system" fn readwrite_factory_CreateInstanceFromObject(iface: *mut IMFReadWriteClassFactory, clsid: *const GUID, unk: *mut IUnknown, attributes: *mut IMFAttributes, riid: *const GUID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut hr: HRESULT;

    if IsEqualGUID!(clsid, core::ptr::addr_of!(CLSID_MFSourceReader)) != false {
        return create_source_reader_from_object(unk, attributes, riid, out);
    } else if IsEqualGUID!(clsid, core::ptr::addr_of!(CLSID_MFSinkWriter)) != false {
        let mut stream: *mut IMFByteStream = core::ptr::null_mut();
        let mut sink: *mut IMFMediaSink = core::ptr::null_mut();

        hr = IUnknown_QueryInterface(unk, core::ptr::addr_of!(IID_IMFByteStream), core::ptr::addr_of_mut!(stream) as *mut *mut core::ffi::c_void);
        if hr < 0 {
            hr = IUnknown_QueryInterface(unk, core::ptr::addr_of!(IID_IMFMediaSink), core::ptr::addr_of_mut!(sink) as *mut *mut core::ffi::c_void);
        }

        if !stream.is_null() {
            hr = create_sink_writer_from_url(core::ptr::null_mut(), stream, attributes, riid, out);
        } else if !sink.is_null() {
            hr = create_sink_writer_from_sink(sink, attributes, riid, out);
        }

        if !sink.is_null() {
            IMFMediaSink_Release(sink);
        }
        if !stream.is_null() {
            IMFByteStream_Release(stream);
        }

        return hr;
    } else {
        WARN!("Unsupported class %s.\n", debugstr_guid(clsid));
        *out = core::ptr::null_mut();
        return E_FAIL;
    }
}

unsafe extern "system" fn classfactory_QueryInterface(iface: *mut IClassFactory, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IClassFactory)) != false ||
            IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) != false
    {
        IClassFactory_AddRef(iface);
        *out = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    WARN!("interface %s not implemented\n", debugstr_guid(riid));
    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}
unsafe extern "system" fn classfactory_AddRef(iface: *mut IClassFactory) -> ULONG {
    2
}
unsafe extern "system" fn classfactory_Release(iface: *mut IClassFactory) -> ULONG {
    1
}

unsafe extern "system" fn classfactory_CreateInstance(iface: *mut IClassFactory, outer: *mut IUnknown, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    TRACE!("%p, %s, %p.\n", outer, debugstr_guid(riid), out);
    *out = core::ptr::null_mut();
    if !outer.is_null() {
        return CLASS_E_NOAGGREGATION;
    }
    return IMFReadWriteClassFactory_QueryInterface(core::ptr::addr_of_mut!(readwrite_factory), riid, out);
}
unsafe extern "system" fn classfactory_LockServer(
    iface: *mut IClassFactory,
    dolock: BOOL,
) -> HRESULT {
    // FIXME("%d.\n", dolock);
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(clsid: *const GUID, riid: *const GUID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualGUID!(clsid, core::ptr::addr_of!(CLSID_MFReadWriteClassFactory)) != false {
        return IClassFactory_QueryInterface(core::ptr::addr_of_mut!(classfactory), riid, out);
    }
    WARN!("Unsupported class %s.\n", debugstr_guid(clsid));
    *out = core::ptr::null_mut();
    return CLASS_E_CLASSNOTAVAILABLE;
}

// COM vtable tables — generated from reader.c by vtbl_gen.py
/// reader.c:305 `static const IUnknownVtbl source_reader_async_command_vtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static source_reader_async_command_vtbl: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: Some(source_reader_async_command_QueryInterface),
    AddRef: Some(source_reader_async_command_AddRef),
    Release: Some(source_reader_async_command_Release),
};

/// reader.c:667 `static const IMFAsyncCallbackVtbl source_events_callback_vtbl`  (slot order: rust:types_amd64.rs)
pub static source_events_callback_vtbl: IMFAsyncCallbackVtbl = IMFAsyncCallbackVtbl {
    QueryInterface: Some(source_reader_callback_QueryInterface),
    AddRef: Some(source_reader_source_events_callback_AddRef),
    Release: Some(source_reader_source_events_callback_Release),
    GetParameters: Some(source_reader_callback_GetParameters),
    Invoke: Some(source_reader_source_events_callback_Invoke),
};

/// reader.c:1205 `static const IMFAsyncCallbackVtbl stream_events_callback_vtbl`  (slot order: rust:types_amd64.rs)
pub static stream_events_callback_vtbl: IMFAsyncCallbackVtbl = IMFAsyncCallbackVtbl {
    QueryInterface: Some(source_reader_callback_QueryInterface),
    AddRef: Some(source_reader_stream_events_callback_AddRef),
    Release: Some(source_reader_stream_events_callback_Release),
    GetParameters: Some(source_reader_callback_GetParameters),
    Invoke: Some(source_reader_stream_events_callback_Invoke),
};

/// reader.c:1642 `static const IMFAsyncCallbackVtbl async_commands_callback_vtbl`  (slot order: rust:types_amd64.rs)
pub static async_commands_callback_vtbl: IMFAsyncCallbackVtbl = IMFAsyncCallbackVtbl {
    QueryInterface: Some(source_reader_callback_QueryInterface),
    AddRef: Some(source_reader_async_commands_callback_AddRef),
    Release: Some(source_reader_async_commands_callback_Release),
    GetParameters: Some(source_reader_callback_GetParameters),
    Invoke: Some(source_reader_async_commands_callback_Invoke),
};

/// reader.c:2813 `static const IMFSourceReaderExVtbl srcreader_vtbl`  (slot order: rust:types_amd64.rs)
pub static srcreader_vtbl: IMFSourceReaderExVtbl = IMFSourceReaderExVtbl {
    QueryInterface: Some(src_reader_QueryInterface),
    AddRef: Some(src_reader_AddRef),
    Release: Some(src_reader_Release),
    GetStreamSelection: Some(src_reader_GetStreamSelection),
    SetStreamSelection: Some(src_reader_SetStreamSelection),
    GetNativeMediaType: Some(src_reader_GetNativeMediaType),
    GetCurrentMediaType: Some(src_reader_GetCurrentMediaType),
    SetCurrentMediaType: Some(src_reader_SetCurrentMediaType),
    SetCurrentPosition: Some(src_reader_SetCurrentPosition),
    ReadSample: Some(src_reader_ReadSample),
    Flush: Some(src_reader_Flush),
    GetServiceForStream: Some(src_reader_GetServiceForStream),
    GetPresentationAttribute: Some(src_reader_GetPresentationAttribute),
    SetNativeMediaType: Some(src_reader_SetNativeMediaType),
    AddTransformForStream: Some(src_reader_AddTransformForStream),
    RemoveAllTransformsForStream: Some(src_reader_RemoveAllTransformsForStream),
    GetTransformForStream: Some(src_reader_GetTransformForStream),
};

/// reader.c:3235 `static const IMFReadWriteClassFactoryVtbl readwrite_factory_vtbl`  (slot order: rust:types_amd64.rs)
pub static readwrite_factory_vtbl: IMFReadWriteClassFactoryVtbl = IMFReadWriteClassFactoryVtbl {
    QueryInterface: Some(readwrite_factory_QueryInterface),
    AddRef: Some(readwrite_factory_AddRef),
    Release: Some(readwrite_factory_Release),
    CreateInstanceFromURL: Some(readwrite_factory_CreateInstanceFromURL),
    CreateInstanceFromObject: Some(readwrite_factory_CreateInstanceFromObject),
};

/// reader.c:3291 `static const IClassFactoryVtbl classfactoryvtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static classfactoryvtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: Some(classfactory_QueryInterface),
    AddRef: Some(classfactory_AddRef),
    Release: Some(classfactory_Release),
    CreateInstance: Some(classfactory_CreateInstance),
    LockServer: Some(classfactory_LockServer),
};
