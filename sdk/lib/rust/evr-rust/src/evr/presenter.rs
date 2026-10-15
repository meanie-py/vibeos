use crate::evr::sample::create_video_sample_allocator;
use crate::evr::evr::evr;
use crate::evr::mixer::input_stream;
use crate::evr::mixer::output_stream;
use crate::prelude::*;

unsafe extern "C" {
    static IID_IQualProp: GUID;
}

/// winnt.h's InterlockedExchangePointer (an intrinsic, so no import exists).
unsafe fn InterlockedExchangePointer(target: *mut *mut core::ffi::c_void, value: *mut core::ffi::c_void) -> *mut core::ffi::c_void {
    (*(target as *const core::sync::atomic::AtomicPtr<core::ffi::c_void>)).swap(value, core::sync::atomic::Ordering::SeqCst)
}

/// wine/debug.h's inline.
unsafe fn wine_dbgstr_rect(rect: *const RECT) -> *const ::core::ffi::c_char {
    if rect.is_null() {
        return b"(null)\0".as_ptr() as *const ::core::ffi::c_char;
    }
    wine_dbg_sprintf(
        b"(%d,%d)-(%d,%d)\0".as_ptr() as *const ::core::ffi::c_char,
        (*rect).left, (*rect).top, (*rect).right, (*rect).bottom,
    )
}
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const PRESENTER_STATE_INITIAL: u32 = 0;
pub const PRESENTER_STATE_SHUT_DOWN: u32 = 1;
pub const PRESENTER_STATE_STARTED: u32 = 2;
pub const PRESENTER_STATE_STOPPED: u32 = 3;
pub const PRESENTER_STATE_PAUSED: u32 = 4;

pub const PRESENTER_MIXER_HAS_INPUT: u32 = 0x1;

pub const EVRM_STOP: u32 = WM_USER;
pub const EVRM_PRESENT: u32 = WM_USER + 1;

#[repr(C)]
pub struct sample_queue {
    pub samples: *mut *mut IMFSample,
    pub size: u32,
    pub used: u32,
    pub front: u32,
    pub back: u32,
    pub last_presented: *mut IMFSample,
    pub cs: CRITICAL_SECTION,
}

#[repr(C)]
pub struct streaming_thread {
    pub hthread: HANDLE,
    pub ready_event: HANDLE,
    pub tid: DWORD,
    pub queue: sample_queue,
}

#[repr(C)]
pub struct video_presenter_frame_stats {
    pub presented: i32,
    pub sampletime: LONGLONG,
}

#[repr(C)]
pub struct video_presenter {
    pub IMFVideoPresenter_iface: IMFVideoPresenter,
    pub IMFVideoDeviceID_iface: IMFVideoDeviceID,
    pub IMFTopologyServiceLookupClient_iface: IMFTopologyServiceLookupClient,
    pub IMFVideoDisplayControl_iface: IMFVideoDisplayControl,
    pub IMFRateSupport_iface: IMFRateSupport,
    pub IMFGetService_iface: IMFGetService,
    pub IMFVideoPositionMapper_iface: IMFVideoPositionMapper,
    pub IQualProp_iface: IQualProp,
    pub IMFQualityAdvise_iface: IMFQualityAdvise,
    pub IMFQualityAdviseLimits_iface: IMFQualityAdviseLimits,
    pub IDirect3DDeviceManager9_iface: IDirect3DDeviceManager9,
    pub allocator_cb: IMFVideoSampleAllocatorNotify,
    pub IUnknown_inner: IUnknown,
    pub outer_unk: *mut IUnknown,
    pub refcount: LONG,

    pub mixer: *mut IMFTransform,
    pub clock: *mut IMFClock,
    pub event_sink: *mut IMediaEventSink,

    pub device_manager: *mut IDirect3DDeviceManager9,
    pub swapchain: *mut IDirect3DSwapChain9,
    pub hdevice: HANDLE,

    pub allocator: *mut IMFVideoSampleAllocator,
    pub thread: streaming_thread,
    pub allocator_capacity: u32,
    pub media_type: *mut IMFMediaType,
    pub frame_time_threshold: LONGLONG,
    pub reset_token: UINT,
    pub video_window: HWND,
    pub src_rect: MFVideoNormalizedRect,
    pub dst_rect: RECT,
    pub rendering_prefs: DWORD,
    pub native_size: SIZE,
    pub native_ratio: SIZE,
    pub ar_mode: u32,
    pub state: u32,
    pub flags: u32,

    pub frame_stats: video_presenter_frame_stats,

    pub cs: CRITICAL_SECTION,
}

const MFVideoARMode_Mask: DWORD = 0x7;

const MFVideoRenderPrefs_Mask: u32 = 0x000001ff;

unsafe extern "C" fn impl_from_IUnknown(iface: *mut IUnknown) -> *mut video_presenter {
    return (iface as *mut u8).sub(core::mem::offset_of!(video_presenter, IUnknown_inner)) as *mut video_presenter;
}

unsafe extern "C" fn impl_from_IMFVideoPresenter(iface: *mut IMFVideoPresenter) -> *mut video_presenter {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(video_presenter, IMFVideoPresenter_iface)) as *mut video_presenter;
}

unsafe extern "C" fn impl_from_IMFVideoDeviceID(iface: *mut IMFVideoDeviceID) -> *mut video_presenter {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(video_presenter, IMFVideoDeviceID_iface)) as *mut video_presenter;
}

unsafe extern "C" fn impl_from_IMFTopologyServiceLookupClient(iface: *mut IMFTopologyServiceLookupClient) -> *mut video_presenter {
    return (iface as usize).wrapping_sub(
        core::mem::offset_of!(video_presenter, IMFTopologyServiceLookupClient_iface),
    ) as *mut video_presenter;
}

unsafe extern "C" fn impl_from_IMFVideoDisplayControl(
    mut iface: *mut IMFVideoDisplayControl,
) -> *mut video_presenter {
    return (iface as *mut u8).wrapping_sub(
        core::mem::offset_of!(video_presenter, IMFVideoDisplayControl_iface),
    ) as *mut video_presenter;
}

unsafe extern "C" fn impl_from_IMFRateSupport(iface: *mut IMFRateSupport) -> *mut video_presenter {
    return CONTAINING_RECORD!(iface, video_presenter, IMFRateSupport_iface);
}

unsafe extern "C" fn impl_from_IMFGetService(iface: *mut IMFGetService) -> *mut video_presenter {
    return CONTAINING_RECORD!(iface, video_presenter, IMFGetService_iface);
}

unsafe extern "C" fn impl_from_IMFVideoPositionMapper(
    iface: *mut IMFVideoPositionMapper,
) -> *mut video_presenter {
    return CONTAINING_RECORD!(iface, video_presenter, IMFVideoPositionMapper_iface);
}

unsafe extern "C" fn impl_from_IMFVideoSampleAllocatorNotify(mut iface: *mut IMFVideoSampleAllocatorNotify) -> *mut video_presenter {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(video_presenter, allocator_cb)) as *mut video_presenter;
}

unsafe extern "C" fn impl_from_IQualProp(iface: *mut IQualProp) -> *mut video_presenter {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(video_presenter, IQualProp_iface)) as *mut video_presenter;
}

unsafe extern "C" fn impl_from_IMFQualityAdvise(iface: *mut IMFQualityAdvise) -> *mut video_presenter {
    return CONTAINING_RECORD!(iface, video_presenter, IMFQualityAdvise_iface);
}

unsafe extern "C" fn impl_from_IMFQualityAdviseLimits(
    mut iface: *mut IMFQualityAdviseLimits,
) -> *mut video_presenter {
    return (iface as *mut u8).wrapping_sub(
        core::mem::offset_of!(video_presenter, IMFQualityAdviseLimits_iface),
    ) as *mut video_presenter;
}

unsafe extern "C" fn impl_from_IDirect3DDeviceManager9(iface: *mut IDirect3DDeviceManager9) -> *mut video_presenter {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(video_presenter, IDirect3DDeviceManager9_iface)) as *mut video_presenter;
}

unsafe extern "C" fn video_presenter_notify_renderer(
    presenter: *mut video_presenter,
    event: LONG,
    param1: LONG_PTR,
    param2: LONG_PTR,
) {
    if !(*presenter).event_sink.is_null() {
        IMediaEventSink_Notify((*presenter).event_sink, event, param1, param2);
    }
}

unsafe extern "C" fn get_gcd(mut a: u32, mut b: u32) -> u32 {
    let mut m: u32;

    while b != 0 {
        m = a % b;
        a = b;
        b = m;
    }

    return a;
}

unsafe extern "C" fn video_presenter_get_device(presenter: *mut video_presenter, device: *mut *mut IDirect3DDevice9) -> HRESULT {
    let mut hr: HRESULT = 0;
    if (*presenter).hdevice.is_null() {
        hr = IDirect3DDeviceManager9_OpenDeviceHandle((*presenter).device_manager, core::ptr::addr_of_mut!((*presenter).hdevice));
        if hr < 0 {
            return hr;
        }
    }

    return IDirect3DDeviceManager9_LockDevice((*presenter).device_manager, (*presenter).hdevice, device, (TRUE) as BOOL);
}

unsafe extern "C" fn video_presenter_get_native_video_size(mut presenter: *mut video_presenter) {
    let mut media_type: *mut IMFMediaType = ::core::ptr::null_mut::<IMFMediaType>();
    let mut frame_size: UINT64 = 0;

    memset(
        &raw mut (*presenter).native_size as *mut ::core::ffi::c_void,
        0,
        ::core::mem::size_of::<SIZE>() as size_t,
    );
    memset(
        &raw mut (*presenter).native_ratio as *mut ::core::ffi::c_void,
        0,
        ::core::mem::size_of::<SIZE>() as size_t,
    );

    if (*presenter).mixer.is_null() {
        return;
    }

    if IMFTransform_GetInputCurrentType((*presenter).mixer, 0, &raw mut media_type)
        < 0
    {
        return;
    }

    if IMFMediaType_GetUINT64(media_type, &raw const MF_MT_FRAME_SIZE, &raw mut frame_size)
        >= 0
    {
        let mut gcd: ::core::ffi::c_uint = 0;

        (*presenter).native_size.cx = (frame_size >> 32) as LONG;
        (*presenter).native_size.cy = frame_size as LONG;

        gcd = get_gcd(
            (*presenter).native_size.cx as ::core::ffi::c_uint,
            (*presenter).native_size.cy as ::core::ffi::c_uint,
        );
        if gcd != 0 {
            (*presenter).native_ratio.cx =
                (((*presenter).native_size.cx as ::core::ffi::c_uint) / gcd) as LONG;
            (*presenter).native_ratio.cy =
                (((*presenter).native_size.cy as ::core::ffi::c_uint) / gcd) as LONG;
        }
    }

    IMFMediaType_Release(media_type);
}

unsafe extern "C" fn video_presenter_set_allocator_callback(
    presenter: *mut video_presenter,
    notify_cb: *mut IMFVideoSampleAllocatorNotify,
) {
    let mut cb: *mut IMFVideoSampleAllocatorCallback = core::ptr::null_mut();

    IMFVideoSampleAllocator_QueryInterface(
        (*presenter).allocator,
        core::ptr::addr_of!(IID_IMFVideoSampleAllocatorCallback),
        core::ptr::addr_of_mut!(cb) as *mut *mut core::ffi::c_void,
    );
    IMFVideoSampleAllocatorCallback_SetCallback(cb, notify_cb);
    IMFVideoSampleAllocatorCallback_Release(cb);
}

unsafe extern "C" fn video_presenter_reset_media_type(presenter: *mut video_presenter) {
    if !(*presenter).media_type.is_null() {
        IMFMediaType_Release((*presenter).media_type);
    }
    (*presenter).media_type = core::ptr::null_mut();

    if !(*presenter).allocator.is_null() {
        IMFVideoSampleAllocator_UninitializeSampleAllocator((*presenter).allocator);
        video_presenter_set_allocator_callback(presenter, core::ptr::null_mut());
    }
}

unsafe extern "C" fn video_presenter_set_media_type(
    presenter: *mut video_presenter,
    media_type: *mut IMFMediaType,
) -> HRESULT {
    let mut flags: DWORD = 0;
    let mut hr: HRESULT = 0;

    if media_type.is_null() {
        video_presenter_reset_media_type(presenter);
        return S_OK;
    }

    if !(*presenter).media_type.is_null()
        && IMFMediaType_IsEqual((*presenter).media_type, media_type, &raw mut flags) == S_OK
    {
        return S_OK;
    }

    video_presenter_reset_media_type(presenter);

    hr = IMFVideoSampleAllocator_InitializeSampleAllocator(
        (*presenter).allocator,
        (*presenter).allocator_capacity as DWORD,
        media_type,
    );
    if hr >= 0 {
        let mut ratio: MFRatio = MFRatio {
            Numerator: 0,
            Denominator: 0,
        };
        let mut rate: UINT64 = 0;
        let mut frametime: UINT64 = 0;

        (*presenter).media_type = media_type;
        IMFMediaType_AddRef((*presenter).media_type);

        if IMFMediaType_GetUINT64(
            (*presenter).media_type,
            &raw const MF_MT_FRAME_RATE,
            &raw mut rate,
        ) >= 0
        {
            ratio.Denominator = rate as DWORD;
            ratio.Numerator = (rate >> 32) as DWORD;
        } else {
            ratio.Denominator = 1 as DWORD;
            ratio.Numerator = 30 as DWORD;
        }

        MFFrameRateToAverageTimePerFrame(
            ratio.Numerator as UINT32,
            ratio.Denominator as UINT32,
            &raw mut frametime,
        );
        (*presenter).frame_time_threshold = (frametime / 4) as LONGLONG;
    } else {
        WARN!("Failed to initialize sample allocator, hr %#lx.\n", hr);
    }

    return hr;
}

unsafe extern "C" fn video_presenter_configure_output_type(
    mut presenter: *mut video_presenter,
    mut aperture: *const MFVideoArea,
    mut media_type: *mut IMFMediaType,
) -> HRESULT {
    let mut subtype: GUID = GUID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    };
    let mut stride: LONG = 0;
    let mut size: DWORD = 0;
    let mut hr: HRESULT = 0;

    hr = IMFMediaType_SetUINT64(
        media_type,
        &raw const MF_MT_FRAME_SIZE,
        ((*aperture).Area.cx as UINT64) << 32 | (*aperture).Area.cy as UINT64,
    );
    if hr >= 0 {
        hr = IMFMediaType_SetBlob(
            media_type,
            &raw const MF_MT_GEOMETRIC_APERTURE,
            aperture as *mut UINT8,
            ::core::mem::size_of::<MFVideoArea>() as UINT32,
        );
    }
    if hr >= 0 {
        hr = IMFMediaType_SetBlob(
            media_type,
            &raw const MF_MT_MINIMUM_DISPLAY_APERTURE,
            aperture as *mut UINT8,
            ::core::mem::size_of::<MFVideoArea>() as UINT32,
        );
    }

    if hr >= 0 {
        hr = IMFMediaType_GetGUID(media_type, &raw const MF_MT_SUBTYPE, &raw mut subtype);
    }

    if hr >= 0 {
        hr = MFGetStrideForBitmapInfoHeader(
            subtype.Data1 as DWORD,
            (*aperture).Area.cx as DWORD,
            &raw mut stride,
        );
        if hr >= 0 {
            hr = MFGetPlaneSize(
                subtype.Data1 as DWORD,
                (*aperture).Area.cx as DWORD,
                (*aperture).Area.cy as DWORD,
                &raw mut size,
            );
        }
        if hr >= 0 {
            hr = IMFMediaType_SetUINT32(
                media_type,
                &raw const MF_MT_DEFAULT_STRIDE,
                abs(stride as ::core::ffi::c_int) as UINT32,
            );
        }
        if hr >= 0 {
            hr = IMFMediaType_SetUINT32(media_type, &raw const MF_MT_SAMPLE_SIZE, size as UINT32);
        }
    }

    return hr;
}

unsafe extern "C" fn video_presenter_invalidate_media_type(presenter: *mut video_presenter) -> HRESULT {
    let mut media_type: *mut IMFMediaType = core::ptr::null_mut();
    let mut candidate_type: *mut IMFMediaType = core::ptr::null_mut();
    let mut idx: core::ffi::c_uint = 0;
    let mut rect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut hr: HRESULT;

    if (*presenter).mixer.is_null() {
        return MF_E_TRANSFORM_TYPE_NOT_SET;
    }

    hr = MFCreateMediaType(core::ptr::addr_of_mut!(media_type));
    if hr < 0 {
        return hr;
    }

    video_presenter_get_native_video_size(presenter);

    loop {
        let fresh_idx = idx;
        idx = idx.wrapping_add(1);
        hr = IMFTransform_GetOutputAvailableType(
            (*presenter).mixer,
            0 as DWORD,
            fresh_idx as DWORD,
            core::ptr::addr_of_mut!(candidate_type),
        );
        if !(hr >= 0) {
            break;
        }

        let mut aperture: MFVideoArea = core::mem::zeroed();
        let _aperture_init = 0;

        rect = (*presenter).dst_rect;
        if IsRectEmpty(core::ptr::addr_of_mut!(rect) as LPCRECT) == 0 {
            aperture.Area.cx = rect.right - rect.left;
            aperture.Area.cy = rect.bottom - rect.top;
        } else if IMFMediaType_GetBlob(
            candidate_type,
            core::ptr::addr_of!(MF_MT_GEOMETRIC_APERTURE),
            core::ptr::addr_of_mut!(aperture) as *mut UINT8,
            core::mem::size_of::<MFVideoArea>() as UINT32,
            core::ptr::null_mut(),
        ) < 0
        {
            aperture.Area.cx = (*presenter).native_size.cx;
            aperture.Area.cy = (*presenter).native_size.cy;
        }

        /* FIXME: check that d3d device supports this format */

        hr = IMFMediaType_CopyAllItems(candidate_type, media_type as *mut IMFAttributes);
        if hr < 0 {
            WARN!("Failed to clone a media type, hr %#lx.\n", hr);
        }
        IMFMediaType_Release(candidate_type);

        hr = video_presenter_configure_output_type(presenter, core::ptr::addr_of!(aperture), media_type);

        if hr >= 0 {
            hr = IMFTransform_SetOutputType(
                (*presenter).mixer,
                0 as DWORD,
                media_type,
                MFT_SET_TYPE_TEST_ONLY as DWORD,
            );
        }

        if hr >= 0 {
            hr = video_presenter_set_media_type(presenter, media_type);
        }

        if hr >= 0 {
            hr = IMFTransform_SetOutputType(
                (*presenter).mixer,
                0 as DWORD,
                media_type,
                0 as DWORD,
            );
        }

        if hr >= 0 {
            break;
        }
    }

    IMFMediaType_Release(media_type);

    hr
}

unsafe extern "C" fn video_presenter_sample_queue_init(
    presenter: *mut video_presenter,
) -> HRESULT {
    let queue: *mut sample_queue = core::ptr::addr_of_mut!((*presenter).thread.queue);

    if (*queue).size != 0 {
        return S_OK;
    }

    memset(
        queue as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<sample_queue>(),
    );
    (*queue).samples = calloc(
        ((*presenter).allocator_capacity as usize) as _,
        (core::mem::size_of::<IMFSample>()) as _,
    ) as *mut *mut IMFSample;
    if (*queue).samples.is_null() {
        return E_OUTOFMEMORY;
    }

    (*queue).size = (*presenter).allocator_capacity;
    (*queue).back = (*queue).size - 1;
    InitializeCriticalSection(core::ptr::addr_of_mut!((*queue).cs));

    return S_OK;
}

unsafe extern "C" fn video_presenter_sample_queue_push(presenter: *mut video_presenter, sample: *mut IMFSample, at_front: BOOL) {
    let queue: *mut sample_queue = core::ptr::addr_of_mut!((*presenter).thread.queue);
    let mut idx: core::ffi::c_uint = 0;

    EnterCriticalSection(core::ptr::addr_of_mut!((*queue).cs));
    if (*queue).used != (*queue).size {
        if at_front != 0 {
            (*queue).front = (*queue)
                .size
                .wrapping_add((*queue).front)
                .wrapping_sub(1)
                .wrapping_rem((*queue).size);
            idx = (*queue).front;
        } else {
            (*queue).back = (*queue)
                .back
                .wrapping_add(1)
                .wrapping_rem((*queue).size);
            idx = (*queue).back;
        }
        *(*queue).samples.offset(idx as isize) = sample;
        (*queue).used = (*queue).used.wrapping_add(1);
        IMFSample_AddRef(sample);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*queue).cs));
}

unsafe extern "C" fn video_presenter_sample_queue_pop(presenter: *mut video_presenter, sample: *mut *mut IMFSample) -> BOOL {
    let queue: *mut sample_queue = core::ptr::addr_of_mut!((*presenter).thread.queue);

    EnterCriticalSection(core::ptr::addr_of_mut!((*queue).cs));
    if (*queue).used != 0 {
        *sample = *(*queue).samples.offset((*queue).front as isize);
        (*queue).front = (*queue).front.wrapping_add(1).wrapping_rem((*queue).size);
        (*queue).used = (*queue).used.wrapping_sub(1);
    } else {
        *sample = core::ptr::null_mut::<IMFSample>();
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*queue).cs));

    return (!(*sample).is_null()) as BOOL;
}

unsafe extern "C" fn video_presenter_sample_queue_free(presenter: *mut video_presenter) {
    let queue: *mut sample_queue = core::ptr::addr_of_mut!((*presenter).thread.queue);
    let mut sample: *mut IMFSample = core::ptr::null_mut::<IMFSample>();

    while video_presenter_sample_queue_pop(presenter, core::ptr::addr_of_mut!(sample)) != 0 {
        IMFSample_Release(sample);
    }

    free((*queue).samples as *mut core::ffi::c_void);
    DeleteCriticalSection(core::ptr::addr_of_mut!((*queue).cs));
}

unsafe extern "C" fn video_presenter_get_sample_surface(
    mut sample: *mut IMFSample,
    mut surface: *mut *mut IDirect3DSurface9,
) -> HRESULT {
    let mut buffer: *mut IMFMediaBuffer = ::core::ptr::null_mut::<IMFMediaBuffer>();
    let mut gs: *mut IMFGetService = ::core::ptr::null_mut::<IMFGetService>();
    let mut hr: HRESULT = 0;
    hr = IMFSample_GetBufferByIndex(sample, 0 as DWORD, &raw mut buffer);
    if hr < 0 as HRESULT {
        return hr;
    }
    hr = IMFMediaBuffer_QueryInterface(
        buffer,
        &raw const IID_IMFGetService,
        &raw mut gs as *mut *mut ::core::ffi::c_void,
    );
    IMFMediaBuffer_Release(buffer);
    if hr < 0 as HRESULT {
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

unsafe extern "C" fn video_presenter_sample_present(presenter: *mut video_presenter, mut sample: *mut IMFSample) {
    let mut surface: *mut IDirect3DSurface9 = core::ptr::null_mut();
    let mut backbuffer: *mut IDirect3DSurface9 = core::ptr::null_mut();
    let mut device: *mut IDirect3DDevice9 = core::ptr::null_mut();
    let queue: *mut sample_queue = core::ptr::addr_of_mut!((*presenter).thread.queue);
    let mut hr: HRESULT;

    hr = video_presenter_get_sample_surface(sample, core::ptr::addr_of_mut!(surface));
    if hr < 0 {
        WARN!("Failed to get sample surface, hr %#lx.\n", hr);
        return;
    }

    if !(*presenter).swapchain.is_null() {
        hr = IDirect3DSwapChain9_GetBackBuffer(
            (*presenter).swapchain,
            0,
            D3DBACKBUFFER_TYPE_MONO,
            core::ptr::addr_of_mut!(backbuffer),
        );
        if hr >= 0 {
            IDirect3DSwapChain9_GetDevice((*presenter).swapchain, core::ptr::addr_of_mut!(device));
            IDirect3DDevice9_StretchRect(
                device,
                surface,
                core::ptr::null::<RECT>(),
                backbuffer,
                core::ptr::null::<RECT>(),
                D3DTEXF_POINT,
            );

            IDirect3DSwapChain9_Present(
                (*presenter).swapchain,
                core::ptr::null::<RECT>(),
                core::ptr::null::<RECT>(),
                core::ptr::null_mut::<HWND__>(),
                core::ptr::null::<RGNDATA>(),
                0,
            );
            (*presenter).frame_stats.presented += 1;

            IDirect3DDevice9_Release(device);
            IDirect3DSurface9_Release(backbuffer);
        } else {
            WARN!("Failed to get a backbuffer, hr %#lx.\n", hr);
        }
    }

    IMFSample_AddRef(sample);
    sample = InterlockedExchangePointer(
        core::ptr::addr_of_mut!((*queue).last_presented) as *mut *mut core::ffi::c_void,
        sample as *mut core::ffi::c_void,
    ) as *mut IMFSample;
    if !sample.is_null() {
        IMFSample_Release(sample);
    }

    IDirect3DSurface9_Release(surface);
}

unsafe extern "C" fn video_presenter_check_queue(
    presenter: *mut video_presenter,
    next_wait: *mut ::core::ffi::c_uint,
) {
    let mut pts: LONGLONG = 0;
    let mut clocktime: LONGLONG = 0;
    let mut delta: LONGLONG = 0;
    let mut wait: ::core::ffi::c_uint = 0;
    let mut sample: *mut IMFSample = core::ptr::null_mut();
    let mut systime: MFTIME = 0;
    let mut present: BOOL = 0;
    let mut hr: HRESULT = 0;

    while video_presenter_sample_queue_pop(presenter, core::ptr::addr_of_mut!(sample)) != 0 {
        present = (TRUE as i32);
        wait = 0;

        if !(*presenter).clock.is_null() {
            clocktime = 0;
            pts = clocktime;

            hr = IMFSample_GetSampleTime(sample, core::ptr::addr_of_mut!(pts));
            if hr >= 0 {
                hr = IMFClock_GetCorrelatedTime(
                    (*presenter).clock,
                    0,
                    core::ptr::addr_of_mut!(clocktime),
                    core::ptr::addr_of_mut!(systime),
                );
            }

            delta = pts - clocktime;
            if delta > 3 * (*presenter).frame_time_threshold {
                /* Convert 100ns -> msec */
                wait = ((delta - 3 * (*presenter).frame_time_threshold)
                    / 10000) as ::core::ffi::c_uint;
                present = (FALSE as i32);
            }
        }

        if present != 0 {
            video_presenter_sample_present(presenter, sample);
        } else {
            video_presenter_sample_queue_push(presenter, sample, (TRUE) as BOOL);
        }

        IMFSample_Release(sample);

        if wait > 0 {
            break;
        }
    }

    if wait == 0 {
        wait = INFINITE;
    }

    *next_wait = wait;
}

unsafe extern "C" fn video_presenter_schedule_sample(presenter: *mut video_presenter, sample: *mut IMFSample) {
    if (*presenter).thread.tid == 0 {
        WARN!("Streaming thread hasn't been started.\n");
        return;
    }

    if !(*presenter).clock.is_null() {
        video_presenter_sample_queue_push(presenter, sample, (FALSE) as BOOL);
        PostThreadMessageW((*presenter).thread.tid, EVRM_PRESENT, 0, 0);
    } else {
        video_presenter_sample_present(presenter, sample);
    }
}

unsafe extern "C" fn video_presenter_process_input(presenter: *mut video_presenter) -> HRESULT {
    let mut buffer: MFT_OUTPUT_DATA_BUFFER = core::mem::zeroed();
    let mut hr: HRESULT = S_OK;
    let mut sample: *mut IMFSample = core::ptr::null_mut();
    let mut status: DWORD = 0;

    if (*presenter).state == PRESENTER_STATE_SHUT_DOWN as ::core::ffi::c_uint {
        return MF_E_SHUTDOWN;
    }

    if (*presenter).media_type.is_null() {
        return S_OK;
    }

    while hr == S_OK {
        let mut mixing_started: LONGLONG = 0;
        let mut mixing_finished: LONGLONG = 0;
        let mut systime: MFTIME = 0;

        if (*presenter).flags & (PRESENTER_MIXER_HAS_INPUT as ::core::ffi::c_uint) == 0 {
            break;
        }

        hr = IMFVideoSampleAllocator_AllocateSample((*presenter).allocator, core::ptr::addr_of_mut!(sample));
        if hr < 0 {
            WARN!("Failed to allocate a sample, hr %#lx.\n", hr);
            break;
        }

        memset(
            core::ptr::addr_of_mut!(buffer) as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            core::mem::size_of::<MFT_OUTPUT_DATA_BUFFER>() as usize,
        );
        buffer.pSample = sample;

        if !(*presenter).clock.is_null() {
            IMFClock_GetCorrelatedTime(
                (*presenter).clock,
                0 as DWORD,
                core::ptr::addr_of_mut!(mixing_started),
                core::ptr::addr_of_mut!(systime),
            );
        }

        hr = IMFTransform_ProcessOutput(
            (*presenter).mixer,
            0 as DWORD,
            1 as DWORD,
            core::ptr::addr_of_mut!(buffer),
            core::ptr::addr_of_mut!(status),
        );
        if hr < 0 {
            /* FIXME: failure path probably needs to handle some errors specifically */
            (*presenter).flags &= !(PRESENTER_MIXER_HAS_INPUT as ::core::ffi::c_uint);
            IMFSample_Release(sample);
            break;
        } else {
            if !(*presenter).clock.is_null() {
                let mut latency: LONGLONG = 0;

                IMFClock_GetCorrelatedTime(
                    (*presenter).clock,
                    0 as DWORD,
                    core::ptr::addr_of_mut!(mixing_finished),
                    core::ptr::addr_of_mut!(systime),
                );
                latency = mixing_finished - mixing_started;
                video_presenter_notify_renderer(
                    presenter,
                    EC_PROCESSING_LATENCY as LONG,
                    core::ptr::addr_of_mut!(latency) as LONG_PTR,
                    0 as LONG_PTR,
                );
            }

            if !buffer.pEvents.is_null() {
                IMFCollection_Release(buffer.pEvents);
            }

            video_presenter_schedule_sample(presenter, sample);

            IMFSample_Release(sample);
        }
    }

    return S_OK;
}

unsafe extern "system" fn video_presenter_streaming_thread(arg: *mut ::core::ffi::c_void) -> DWORD {
    let presenter: *mut video_presenter = arg as *mut video_presenter;
    let mut wait: ::core::ffi::c_uint = INFINITE;
    let mut stop_thread: BOOL = (FALSE as i32);
    let mut msg: MSG = core::mem::zeroed();

    PeekMessageW(core::ptr::addr_of_mut!(msg), core::ptr::null_mut(), WM_USER, WM_USER, PM_NOREMOVE);

    SetEvent((*presenter).thread.ready_event);

    while stop_thread == 0 {
        if MsgWaitForMultipleObjects(0, core::ptr::null(), (FALSE) as BOOL, wait, QS_POSTMESSAGE) == WAIT_TIMEOUT {
            video_presenter_check_queue(presenter, core::ptr::addr_of_mut!(wait));
        }
        while PeekMessageW(core::ptr::addr_of_mut!(msg), core::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
            let mut peek: BOOL = (TRUE as i32);

            match msg.message {
                EVRM_STOP => {
                    stop_thread = (TRUE as i32);
                }
                EVRM_PRESENT => {
                    if peek != 0 {
                        video_presenter_check_queue(presenter, core::ptr::addr_of_mut!(wait));
                        peek = (wait != INFINITE) as ::core::ffi::c_int as BOOL;
                    }
                }
                _ => {}
            }
        }
    }

    return 0;
}

unsafe extern "C" fn video_presenter_start_streaming(presenter: *mut video_presenter) -> HRESULT {
    let mut hr: HRESULT;

    if (*presenter).state == PRESENTER_STATE_SHUT_DOWN {
        return MF_E_SHUTDOWN;
    }

    if !(*presenter).thread.hthread.is_null() {
        return S_OK;
    }

    hr = video_presenter_sample_queue_init(presenter);
    if hr < 0 {
        return hr;
    }

    (*presenter).thread.ready_event = CreateEventW(
        core::ptr::null_mut(),
        (FALSE) as BOOL,
        (FALSE) as BOOL,
        core::ptr::null_mut(),
    );
    if (*presenter).thread.ready_event.is_null() {
        return HRESULT_FROM_WIN32!(GetLastError());
    }

    (*presenter).thread.hthread = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(video_presenter_streaming_thread),
        presenter as LPVOID,
        0,
        core::ptr::addr_of_mut!((*presenter).thread.tid),
    );
    if (*presenter).thread.hthread.is_null() {
        WARN!("Failed to create streaming thread.\n");
        CloseHandle((*presenter).thread.ready_event);
        (*presenter).thread.ready_event = core::ptr::null_mut();
        return E_FAIL;
    }

    video_presenter_set_allocator_callback(presenter, core::ptr::addr_of_mut!((*presenter).allocator_cb));

    WaitForSingleObject((*presenter).thread.ready_event, INFINITE);
    CloseHandle((*presenter).thread.ready_event);
    (*presenter).thread.ready_event = core::ptr::null_mut();

    return S_OK;
}

unsafe extern "C" fn video_presenter_end_streaming(presenter: *mut video_presenter) -> HRESULT {
    let queue: *mut sample_queue = core::ptr::addr_of_mut!((*presenter).thread.queue);
    let mut sample: *mut IMFSample = core::ptr::null_mut::<IMFSample>();
    if (*presenter).thread.hthread.is_null() {
        return S_OK;
    }
    PostThreadMessageW(
        (*presenter).thread.tid,
        (EVRM_STOP) as UINT,
        (0) as WPARAM,
        (0) as LPARAM,
    );

    WaitForSingleObject((*presenter).thread.hthread, (INFINITE) as DWORD);
    CloseHandle((*presenter).thread.hthread);

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/presenter.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"video_presenter_end_streaming\0" as *const u8 as *const ::core::ffi::c_char,
            767 as ::core::ffi::c_int,
            b"Terminated streaming thread tid %#lx.\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*presenter).thread.tid,
        );
    }

    sample = InterlockedExchangePointer(
        core::ptr::addr_of_mut!((*queue).last_presented) as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    ) as *mut IMFSample;
    if !sample.is_null() {
        IMFSample_Release(sample);
    }

    video_presenter_sample_queue_free(presenter);
    memset(
        core::ptr::addr_of_mut!((*presenter).thread) as *mut core::ffi::c_void,
        (0) as core::ffi::c_int,
        core::mem::size_of::<streaming_thread>(),
    );
    video_presenter_set_allocator_callback(presenter, core::ptr::null_mut());

    return S_OK;
}

unsafe extern "system" fn video_presenter_inner_QueryInterface(iface: *mut IUnknown, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IUnknown(iface);

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0 {
        *obj = iface as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFClockStateSink)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFVideoPresenter)) != 0
    {
        *obj = core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFVideoDeviceID)) != 0 {
        *obj = core::ptr::addr_of_mut!((*presenter).IMFVideoDeviceID_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFTopologyServiceLookupClient)) != 0 {
        *obj = core::ptr::addr_of_mut!((*presenter).IMFTopologyServiceLookupClient_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFVideoDisplayControl)) != 0 {
        *obj = core::ptr::addr_of_mut!((*presenter).IMFVideoDisplayControl_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFRateSupport)) != 0 {
        *obj = core::ptr::addr_of_mut!((*presenter).IMFRateSupport_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFGetService)) != 0 {
        *obj = core::ptr::addr_of_mut!((*presenter).IMFGetService_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFVideoPositionMapper)) != 0 {
        *obj = core::ptr::addr_of_mut!((*presenter).IMFVideoPositionMapper_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IQualProp)) != 0 {
        *obj = core::ptr::addr_of_mut!((*presenter).IQualProp_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFQualityAdvise)) != 0 {
        *obj = core::ptr::addr_of_mut!((*presenter).IMFQualityAdvise_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFQualityAdviseLimits)) != 0 {
        *obj = core::ptr::addr_of_mut!((*presenter).IMFQualityAdviseLimits_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IDirect3DDeviceManager9)) != 0 {
        *obj = core::ptr::addr_of_mut!((*presenter).IDirect3DDeviceManager9_iface) as *mut core::ffi::c_void;
    } else {
        WARN!("Unimplemented interface %s.\n", debugstr_guid(riid));
        *obj = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef((*obj) as *mut IUnknown);
    return S_OK;
}

unsafe extern "system" fn video_presenter_inner_AddRef(iface: *mut IUnknown) -> ULONG {
    let presenter: *mut video_presenter = impl_from_IUnknown(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*presenter).refcount)) as ULONG;

    return refcount;
}

unsafe extern "C" fn video_presenter_clear_container(presenter: *mut video_presenter) {
    if !(*presenter).clock.is_null() {
        IMFClock_Release((*presenter).clock);
    }
    if !(*presenter).mixer.is_null() {
        IMFTransform_Release((*presenter).mixer);
    }
    if !(*presenter).event_sink.is_null() {
        IMediaEventSink_Release((*presenter).event_sink);
    }
    (*presenter).clock = core::ptr::null_mut();
    (*presenter).mixer = core::ptr::null_mut();
    (*presenter).event_sink = core::ptr::null_mut();
}

unsafe extern "system" fn video_presenter_inner_Release(iface: *mut IUnknown) -> ULONG {
    let presenter: *mut video_presenter = impl_from_IUnknown(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*presenter).refcount)) as ULONG;

    if refcount == 0 {
        video_presenter_end_streaming(presenter);
        video_presenter_clear_container(presenter);
        video_presenter_reset_media_type(presenter);
        DeleteCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));
        if !(*presenter).swapchain.is_null() {
            IDirect3DSwapChain9_Release((*presenter).swapchain);
        }
        if !(*presenter).device_manager.is_null() {
            IDirect3DDeviceManager9_CloseDeviceHandle((*presenter).device_manager, (*presenter).hdevice);
            IDirect3DDeviceManager9_Release((*presenter).device_manager);
        }
        if !(*presenter).allocator.is_null() {
            IMFVideoSampleAllocator_Release((*presenter).allocator);
        }
        free(presenter as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn video_presenter_QueryInterface(iface: *mut IMFVideoPresenter, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut presenter: *mut video_presenter = impl_from_IMFVideoPresenter(iface);
    return IUnknown_QueryInterface((*presenter).outer_unk, riid, obj);
}

unsafe extern "system" fn video_presenter_AddRef(iface: *mut IMFVideoPresenter) -> ULONG {
    let presenter: *mut video_presenter = impl_from_IMFVideoPresenter(iface);
    return IUnknown_AddRef((*presenter).outer_unk);
}

unsafe extern "system" fn video_presenter_Release(iface: *mut IMFVideoPresenter) -> ULONG {
    let presenter: *mut video_presenter = impl_from_IMFVideoPresenter(iface);
    return IUnknown_Release((*presenter).outer_unk);
}

unsafe extern "system" fn video_presenter_OnClockStart(iface: *mut IMFVideoPresenter, systime: MFTIME, offset: LONGLONG) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFVideoPresenter(iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));
    (*presenter).state = PRESENTER_STATE_STARTED;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    return S_OK;
}

unsafe extern "system" fn video_presenter_OnClockStop(iface: *mut IMFVideoPresenter, systime: MFTIME) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFVideoPresenter(iface);

    TRACE!("%p, %s.\n", iface, debugstr_time(systime as REFERENCE_TIME));

    EnterCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));
    (*presenter).state = PRESENTER_STATE_STOPPED;
    (*presenter).frame_stats.presented = 0;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    return S_OK;
}

unsafe extern "system" fn video_presenter_OnClockPause(iface: *mut IMFVideoPresenter, systime: MFTIME) -> HRESULT {
    let mut presenter: *mut video_presenter = impl_from_IMFVideoPresenter(iface);

    EnterCriticalSection(&raw mut (*presenter).cs);
    (*presenter).state = PRESENTER_STATE_PAUSED;
    LeaveCriticalSection(&raw mut (*presenter).cs);

    return S_OK;
}

unsafe extern "system" fn video_presenter_OnClockRestart(iface: *mut IMFVideoPresenter, systime: MFTIME) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFVideoPresenter(iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));
    (*presenter).state = PRESENTER_STATE_STARTED as ::core::ffi::c_uint;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    return S_OK;
}
unsafe extern "system" fn video_presenter_OnClockSetRate(
    iface: *mut IMFVideoPresenter,
    systime: MFTIME,
    rate: f32,
) -> HRESULT {
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_FIXME as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/presenter.c\0" as *const u8 as *const i8,
            b"video_presenter_OnClockSetRate\0" as *const u8 as *const i8,
            976i32,
            b"%p, %s, %f.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_time(systime as REFERENCE_TIME),
            rate as f64,
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn video_presenter_ProcessMessage(iface: *mut IMFVideoPresenter, message: MFVP_MESSAGE_TYPE, param: ULONG_PTR) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFVideoPresenter(iface);
    let mut hr: HRESULT;

    EnterCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    match message {
        MFVP_MESSAGE_INVALIDATEMEDIATYPE => {
            if (*presenter).state == PRESENTER_STATE_SHUT_DOWN {
                hr = MF_E_SHUTDOWN;
            } else if (*presenter).mixer.is_null() {
                hr = MF_E_INVALIDREQUEST;
            } else {
                hr = video_presenter_invalidate_media_type(presenter);
            }
        }
        MFVP_MESSAGE_BEGINSTREAMING => {
            hr = video_presenter_start_streaming(presenter);
        }
        MFVP_MESSAGE_ENDSTREAMING => {
            hr = video_presenter_end_streaming(presenter);
        }
        MFVP_MESSAGE_PROCESSINPUTNOTIFY => {
            (*presenter).flags |= PRESENTER_MIXER_HAS_INPUT;
            hr = video_presenter_process_input(presenter);
        }
        _ => {
            FIXME!("Unsupported message %u.\n", message);
            hr = E_NOTIMPL;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    return hr;
}

unsafe extern "system" fn video_presenter_GetCurrentMediaType(
    iface: *mut IMFVideoPresenter,
    media_type: *mut *mut IMFVideoMediaType,
) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFVideoPresenter(iface);
    let mut hr: HRESULT = 0;

    TRACE!("%p, %p.\n", iface, media_type);

    EnterCriticalSection(&raw mut (*presenter).cs);

    if (*presenter).state == PRESENTER_STATE_SHUT_DOWN {
        hr = MF_E_SHUTDOWN;
    } else if (*presenter).media_type.is_null() {
        hr = MF_E_NOT_INITIALIZED;
    } else {
        hr = IMFMediaType_QueryInterface(
            (*presenter).media_type,
            &raw const IID_IMFVideoMediaType,
            media_type as *mut *mut ::core::ffi::c_void,
        );
    }

    LeaveCriticalSection(&raw mut (*presenter).cs);

    return hr;
}

unsafe extern "system" fn video_presenter_device_id_QueryInterface(iface: *mut IMFVideoDeviceID, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFVideoDeviceID(iface);
    return IMFVideoPresenter_QueryInterface(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface), riid, obj);
}

unsafe extern "system" fn video_presenter_device_id_AddRef(iface: *mut IMFVideoDeviceID) -> ULONG {
    let presenter: *mut video_presenter = impl_from_IMFVideoDeviceID(iface);
    return IMFVideoPresenter_AddRef(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface));
}

unsafe extern "system" fn video_presenter_device_id_Release(iface: *mut IMFVideoDeviceID) -> ULONG {
    let presenter: *mut video_presenter = impl_from_IMFVideoDeviceID(iface);
    return IMFVideoPresenter_Release(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface));
}

unsafe extern "system" fn video_presenter_device_id_GetDeviceID(iface: *mut IMFVideoDeviceID, device_id: *mut IID) -> HRESULT {
    if device_id.is_null() {
        return E_POINTER;
    }

    memcpy(
        device_id as *mut core::ffi::c_void,
        core::ptr::addr_of!(IID_IDirect3DDevice9) as *const core::ffi::c_void,
        core::mem::size_of::<IID>() as usize,
    );

    return S_OK;
}

unsafe extern "system" fn video_presenter_service_client_QueryInterface(
    iface: *mut IMFTopologyServiceLookupClient,
    riid: *const IID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFTopologyServiceLookupClient(iface);
    return IMFVideoPresenter_QueryInterface(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface), riid, obj);
}

unsafe extern "system" fn video_presenter_service_client_AddRef(
    iface: *mut IMFTopologyServiceLookupClient,
) -> ULONG {
    let presenter: *mut video_presenter = impl_from_IMFTopologyServiceLookupClient(iface);
    return IMFVideoPresenter_AddRef(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface));
}

unsafe extern "system" fn video_presenter_service_client_Release(iface: *mut IMFTopologyServiceLookupClient) -> ULONG {
    let presenter: *mut video_presenter = impl_from_IMFTopologyServiceLookupClient(iface);
    return IMFVideoPresenter_Release(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface));
}

unsafe extern "C" fn video_presenter_set_mixer_rect(presenter: *mut video_presenter) {
    let mut attributes: *mut IMFAttributes = core::ptr::null_mut::<IMFAttributes>();
    let mut hr: HRESULT = 0;

    if (*presenter).mixer.is_null() {
        return;
    }

    if IMFTransform_GetAttributes((*presenter).mixer, &raw mut attributes) >= 0 {
        hr = IMFAttributes_SetBlob(
            attributes,
            &raw const VIDEO_ZOOM_RECT,
            &raw mut (*presenter).src_rect as *const UINT8,
            core::mem::size_of::<MFVideoNormalizedRect>() as UINT32,
        );
        if hr < 0 {
            WARN!("Failed to set zoom rectangle attribute, hr %#lx.\n", hr);
        }
        IMFAttributes_Release(attributes);
    }
}

unsafe extern "C" fn video_presenter_attach_mixer(presenter: *mut video_presenter, service_lookup: *mut IMFTopologyServiceLookup) -> HRESULT {
    let mut device_id: *mut IMFVideoDeviceID = core::ptr::null_mut();
    let mut id: GUID = GUID { Data1: 0, Data2: 0, Data3: 0, Data4: [0; 8] };
    let mut count: DWORD;
    let mut hr: HRESULT;

    count = 1;
    hr = IMFTopologyServiceLookup_LookupService(service_lookup, MF_SERVICE_LOOKUP_GLOBAL, 0,
            core::ptr::addr_of!(MR_VIDEO_MIXER_SERVICE), core::ptr::addr_of!(IID_IMFTransform), core::ptr::addr_of_mut!((*presenter).mixer) as *mut *mut core::ffi::c_void, core::ptr::addr_of_mut!(count));
    if hr < 0 {
        WARN!("Failed to get mixer interface, hr %#lx.\n", hr);
        return hr;
    }

    hr = IMFTransform_QueryInterface((*presenter).mixer, core::ptr::addr_of!(IID_IMFVideoDeviceID), core::ptr::addr_of_mut!(device_id) as *mut *mut core::ffi::c_void);
    if hr >= 0 {
        hr = IMFVideoDeviceID_GetDeviceID(device_id, core::ptr::addr_of_mut!(id));
        if hr >= 0 {
            if IsEqualGUID!(core::ptr::addr_of!(id), core::ptr::addr_of!(IID_IDirect3DDevice9)) == false {
                hr = MF_E_INVALIDREQUEST;
            }
        }

        IMFVideoDeviceID_Release(device_id);
    }

    if hr < 0 {
        IMFTransform_Release((*presenter).mixer);
        (*presenter).mixer = core::ptr::null_mut();
    }

    video_presenter_set_mixer_rect(presenter);
    video_presenter_get_native_video_size(presenter);

    return hr;
}

unsafe extern "system" fn video_presenter_service_client_InitServicePointers(iface: *mut IMFTopologyServiceLookupClient, service_lookup: *mut IMFTopologyServiceLookup) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFTopologyServiceLookupClient(iface);
    let mut hr: HRESULT = S_OK;
    let mut count: DWORD = 0;

    TRACE!("%p, %p.\n", iface, service_lookup);

    if service_lookup.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(&raw mut (*presenter).cs);

    if (*presenter).state == PRESENTER_STATE_STARTED || (*presenter).state == PRESENTER_STATE_PAUSED {
        hr = MF_E_INVALIDREQUEST;
    } else {
        video_presenter_clear_container(presenter);

        count = 1;
        IMFTopologyServiceLookup_LookupService(
            service_lookup,
            MF_SERVICE_LOOKUP_GLOBAL,
            0,
            &raw const MR_VIDEO_RENDER_SERVICE,
            &raw const IID_IMFClock,
            &raw mut (*presenter).clock as *mut *mut core::ffi::c_void,
            &raw mut count,
        );

        hr = video_presenter_attach_mixer(presenter, service_lookup);

        if hr >= 0 {
            count = 1;
            hr = IMFTopologyServiceLookup_LookupService(
                service_lookup,
                MF_SERVICE_LOOKUP_GLOBAL,
                0,
                &raw const MR_VIDEO_RENDER_SERVICE,
                &raw const IID_IMediaEventSink,
                &raw mut (*presenter).event_sink as *mut *mut core::ffi::c_void,
                &raw mut count,
            );
            if hr < 0 {
                WARN!("Failed to get renderer event sink, hr %#lx.\n", hr);
            }
        }

        if hr >= 0 {
            (*presenter).state = PRESENTER_STATE_STOPPED;
        }
    }

    LeaveCriticalSection(&raw mut (*presenter).cs);

    return hr;
}

unsafe extern "system" fn video_presenter_service_client_ReleaseServicePointers(iface: *mut IMFTopologyServiceLookupClient) -> HRESULT {
    let mut presenter: *mut video_presenter = impl_from_IMFTopologyServiceLookupClient(iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    (*presenter).state = PRESENTER_STATE_SHUT_DOWN;
    video_presenter_clear_container(presenter);

    LeaveCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    return S_OK;
}

unsafe extern "system" fn video_presenter_control_QueryInterface(iface: *mut IMFVideoDisplayControl, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFVideoDisplayControl(iface);
    return IMFVideoPresenter_QueryInterface(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface), riid, obj);
}

unsafe extern "system" fn video_presenter_control_AddRef(iface: *mut IMFVideoDisplayControl) -> ULONG {
    let mut presenter: *mut video_presenter = impl_from_IMFVideoDisplayControl(iface);
    return IMFVideoPresenter_AddRef(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface));
}

unsafe extern "system" fn video_presenter_control_Release(iface: *mut IMFVideoDisplayControl) -> ULONG {
    let mut presenter: *mut video_presenter = impl_from_IMFVideoDisplayControl(iface);
    return IMFVideoPresenter_Release(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface));
}

unsafe extern "system" fn video_presenter_control_GetNativeVideoSize(iface: *mut IMFVideoDisplayControl, video_size: *mut SIZE, aspect_ratio: *mut SIZE) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFVideoDisplayControl(iface);
    let mut hr: HRESULT = S_OK;

    if video_size.is_null() && aspect_ratio.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    if (*presenter).state == PRESENTER_STATE_SHUT_DOWN {
        hr = MF_E_SHUTDOWN;
    } else {
        if !video_size.is_null() {
            *video_size = (*presenter).native_size;
        }
        if !aspect_ratio.is_null() {
            *aspect_ratio = (*presenter).native_ratio;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    return hr;
}

unsafe extern "system" fn video_presenter_control_GetIdealVideoSize(iface: *mut IMFVideoDisplayControl, min_size: *mut SIZE, max_size: *mut SIZE) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFVideoDisplayControl(iface);
    let mut hr: HRESULT;

    EnterCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    if (*presenter).state == PRESENTER_STATE_SHUT_DOWN {
        hr = MF_E_SHUTDOWN;
    } else {
        hr = E_NOTIMPL;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    return hr;
}

unsafe extern "system" fn video_presenter_control_SetVideoPosition(
    iface: *mut IMFVideoDisplayControl,
    src_rect: *const MFVideoNormalizedRect,
    dst_rect: *const RECT,
) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFVideoDisplayControl(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!(
        "%p, %s, %s.\n",
        iface,
        debugstr_normalized_rect(src_rect),
        wine_dbgstr_rect(dst_rect)
    );

    if src_rect.is_null() && dst_rect.is_null() {
        return E_POINTER;
    }

    if !src_rect.is_null()
        && ((*src_rect).left < 0.0f32
            || (*src_rect).top < 0.0f32
            || (*src_rect).right > 1.0f32
            || (*src_rect).bottom > 1.0f32
            || (*src_rect).left > (*src_rect).right
            || (*src_rect).top > (*src_rect).bottom)
    {
        return E_INVALIDARG;
    }

    if !dst_rect.is_null()
        && ((*dst_rect).left > (*dst_rect).right || (*dst_rect).top > (*dst_rect).bottom)
    {
        return E_INVALIDARG;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));
    if (*presenter).video_window.is_null() {
        hr = E_POINTER;
    } else {
        if !src_rect.is_null() {
            if memcmp(
                core::ptr::addr_of_mut!((*presenter).src_rect) as *const core::ffi::c_void,
                src_rect as *const core::ffi::c_void,
                core::mem::size_of::<MFVideoNormalizedRect>() as size_t,
            ) != 0
            {
                (*presenter).src_rect = *src_rect;
                video_presenter_set_mixer_rect(presenter);
            }
        }
        if !dst_rect.is_null()
            && EqualRect(
                dst_rect,
                core::ptr::addr_of_mut!((*presenter).dst_rect),
            ) == 0
        {
            (*presenter).dst_rect = *dst_rect;
            hr = video_presenter_invalidate_media_type(presenter);
            /* Mixer's input type hasn't been configured yet, this is not an error. */
            if hr == MF_E_TRANSFORM_TYPE_NOT_SET {
                hr = S_OK;
            }
            /* FIXME: trigger repaint */
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    return hr;
}

unsafe extern "system" fn video_presenter_control_GetVideoPosition(iface: *mut IMFVideoDisplayControl, src_rect: *mut MFVideoNormalizedRect, dst_rect: *mut RECT) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFVideoDisplayControl(iface);

    if src_rect.is_null() || dst_rect.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));
    *src_rect = (*presenter).src_rect;
    *dst_rect = (*presenter).dst_rect;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    return S_OK;
}

unsafe extern "system" fn video_presenter_control_SetAspectRatioMode(iface: *mut IMFVideoDisplayControl, mode: DWORD) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFVideoDisplayControl(iface);

    if mode & !MFVideoARMode_Mask != 0 {
        return E_INVALIDARG;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));
    (*presenter).ar_mode = mode as u32;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    return S_OK;
}

unsafe extern "system" fn video_presenter_control_GetAspectRatioMode(
    iface: *mut IMFVideoDisplayControl,
    mode: *mut DWORD,
) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFVideoDisplayControl(iface);

    if mode.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));
    *mode = (*presenter).ar_mode as DWORD;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    return S_OK;
}

unsafe extern "C" fn video_presenter_create_swapchain(presenter: *mut video_presenter) -> HRESULT {
    let mut present_params: D3DPRESENT_PARAMETERS = core::mem::zeroed();
    let mut d3d_device: *mut IDirect3DDevice9 = core::ptr::null_mut();
    let mut hr: HRESULT;

    if {
        hr = video_presenter_get_device(presenter, &raw mut d3d_device);
        hr
    } >= 0
    {
        present_params.hDeviceWindow = (*presenter).video_window;
        present_params.Windowed = (TRUE as i32);
        present_params.SwapEffect = D3DSWAPEFFECT_COPY;
        present_params.Flags = D3DPRESENTFLAG_VIDEO;
        present_params.PresentationInterval = D3DPRESENT_INTERVAL_IMMEDIATE;
        hr = IDirect3DDevice9_CreateAdditionalSwapChain(
            d3d_device,
            &raw mut present_params,
            &raw mut (*presenter).swapchain,
        );

        IDirect3DDevice9_Release(d3d_device);
        IDirect3DDeviceManager9_UnlockDevice(
            (*presenter).device_manager,
            (*presenter).hdevice,
            (FALSE) as BOOL,
        );
    }

    return hr;
}

unsafe extern "system" fn video_presenter_control_SetVideoWindow(iface: *mut IMFVideoDisplayControl, window: HWND) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFVideoDisplayControl(iface);
    let mut hr: HRESULT = S_OK;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/presenter.c\0".as_ptr() as *const ::core::ffi::c_char,
            b"video_presenter_control_SetVideoWindow\0".as_ptr() as *const ::core::ffi::c_char,
            1439 as ::core::ffi::c_int,
            b"%p, %p.\n\0".as_ptr() as *const ::core::ffi::c_char,
            iface,
            window,
        );
    }

    if IsWindow(window) == 0 {
        return E_INVALIDARG;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));
    if (*presenter).video_window != window {
        if !(*presenter).swapchain.is_null() {
            IDirect3DSwapChain9_Release((*presenter).swapchain);
        }
        (*presenter).swapchain = core::ptr::null_mut::<IDirect3DSwapChain9>();
        (*presenter).video_window = window;
        hr = video_presenter_create_swapchain(presenter);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    return hr;
}

unsafe extern "system" fn video_presenter_control_GetVideoWindow(iface: *mut IMFVideoDisplayControl, window: *mut HWND) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFVideoDisplayControl(iface);
    if window.is_null() {
        return E_POINTER;
    }
    EnterCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));
    *window = (*presenter).video_window;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));
    return S_OK;
}

unsafe extern "system" fn video_presenter_control_RepaintVideo(iface: *mut IMFVideoDisplayControl) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFVideoDisplayControl(iface);
    let mut hr: HRESULT;

    EnterCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    if (*presenter).state == PRESENTER_STATE_SHUT_DOWN {
        hr = MF_E_SHUTDOWN;
    } else {
        hr = E_NOTIMPL;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    return hr;
}

unsafe extern "system" fn video_presenter_control_GetCurrentImage(
    iface: *mut IMFVideoDisplayControl,
    header: *mut BITMAPINFOHEADER,
    dib: *mut *mut BYTE,
    dib_size: *mut DWORD,
    timestamp: *mut LONGLONG,
) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFVideoDisplayControl(iface);
    let queue: *mut sample_queue = core::ptr::addr_of_mut!((*presenter).thread.queue);
    let mut readback: *mut IDirect3DSurface9 = core::ptr::null_mut();
    let mut surface: *mut IDirect3DSurface9 = core::ptr::null_mut();
    let mut surface_desc: D3DSURFACE_DESC = core::mem::zeroed();
    let mut mapped_rect: D3DLOCKED_RECT = core::mem::zeroed();
    let mut device: *mut IDirect3DDevice9 = core::ptr::null_mut();
    let mut sample: *mut IMFSample = core::ptr::null_mut();
    let mut stride: LONG = 0;
    let mut hr: HRESULT = 0;

    TRACE!(
        "%p, %p, %p, %p, %p.\n",
        iface,
        header,
        dib,
        dib_size,
        timestamp
    );

    EnterCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    sample = InterlockedExchangePointer(
        core::ptr::addr_of_mut!((*queue).last_presented) as *mut *mut ::core::ffi::c_void,
        core::ptr::null_mut(),
    ) as *mut IMFSample;
    if sample.is_null() {
        hr = MF_E_INVALIDREQUEST;
    } else {
        hr = video_presenter_get_sample_surface(sample, core::ptr::addr_of_mut!(surface));
        if hr >= 0 {
            IDirect3DSurface9_GetDevice(surface, core::ptr::addr_of_mut!(device));
            IDirect3DSurface9_GetDesc(surface, core::ptr::addr_of_mut!(surface_desc));

            if surface_desc.Format as ::core::ffi::c_uint
                != D3DFMT_X8R8G8B8 as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                FIXME!(
                    "Unexpected surface format %d.\n",
                    surface_desc.Format as ::core::ffi::c_uint
                );
                hr = E_FAIL;
            }

            if hr >= 0 {
                hr = IDirect3DDevice9_CreateOffscreenPlainSurface(
                    device,
                    surface_desc.Width,
                    surface_desc.Height,
                    D3DFMT_X8R8G8B8,
                    D3DPOOL_SYSTEMMEM,
                    core::ptr::addr_of_mut!(readback),
                    core::ptr::null_mut(),
                );
                if hr < 0 {
                    WARN!("Failed to create readback surface, hr %#lx.\n", hr);
                }
            }

            if hr >= 0 {
                hr = IDirect3DDevice9_GetRenderTargetData(device, surface, readback);
            }

            if hr >= 0 {
                MFGetStrideForBitmapInfoHeader(
                    D3DFMT_X8R8G8B8,
                    surface_desc.Width,
                    core::ptr::addr_of_mut!(stride),
                );
                *dib_size = (abs(stride as ::core::ffi::c_int) as UINT)
                    .wrapping_mul(surface_desc.Height) as DWORD;
                *dib = CoTaskMemAlloc(*dib_size as SIZE_T) as *mut BYTE;
                if (*dib).is_null() {
                    hr = E_OUTOFMEMORY;
                }
            }

            if hr >= 0 {
                hr = IDirect3DSurface9_LockRect(
                    readback,
                    core::ptr::addr_of_mut!(mapped_rect),
                    core::ptr::null(),
                    D3DLOCK_READONLY,
                );
                if hr >= 0 {
                    hr = MFCopyImage(
                        if stride < 0 {
                            (*dib)
                                .offset(*dib_size as isize)
                                .offset(stride as isize)
                        } else {
                            *dib
                        },
                        stride,
                        mapped_rect.pBits as *const BYTE,
                        mapped_rect.Pitch as LONG,
                        abs(stride as ::core::ffi::c_int) as DWORD,
                        surface_desc.Height as DWORD,
                    );
                    IDirect3DSurface9_UnlockRect(readback);
                }
            }

            memset(
                header as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<BITMAPINFOHEADER>() as size_t,
            );
            (*header).biSize = ::core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
            (*header).biWidth = surface_desc.Width as LONG;
            (*header).biHeight = surface_desc.Height as LONG;
            (*header).biPlanes = 1 as WORD;
            (*header).biBitCount = 32 as WORD;
            (*header).biSizeImage = *dib_size;
            IMFSample_GetSampleTime(sample, timestamp);

            if !readback.is_null() {
                IDirect3DSurface9_Release(readback);
            }
            IDirect3DSurface9_Release(surface);

            IDirect3DDevice9_Release(device);
        }
    }

    if !sample.is_null() {
        IMFSample_Release(sample);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    return hr;
}
unsafe extern "system" fn video_presenter_control_SetBorderColor(
    iface: *mut IMFVideoDisplayControl,
    color: COLORREF,
) -> HRESULT {
    // FIXME("%p, %#lx.\n", iface, color);
    E_NOTIMPL
}
unsafe extern "system" fn video_presenter_control_GetBorderColor(
    iface: *mut IMFVideoDisplayControl,
    color: *mut COLORREF,
) -> HRESULT {
    // FIXME("%p, %p.\n", iface, color);
    E_NOTIMPL
}

unsafe extern "system" fn video_presenter_control_SetRenderingPrefs(
    iface: *mut IMFVideoDisplayControl,
    flags: DWORD,
) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFVideoDisplayControl(iface);

    if flags & !MFVideoRenderPrefs_Mask != 0 {
        return E_INVALIDARG;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));
    (*presenter).rendering_prefs = flags;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    return S_OK;
}

unsafe extern "system" fn video_presenter_control_GetRenderingPrefs(iface: *mut IMFVideoDisplayControl, flags: *mut DWORD) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFVideoDisplayControl(iface);

    if flags.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));
    *flags = (*presenter).rendering_prefs;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    return S_OK;
}
unsafe extern "system" fn video_presenter_control_SetFullscreen(
    iface: *mut IMFVideoDisplayControl,
    fullscreen: BOOL,
) -> HRESULT {
    // FIXME("%p, %d.\n", iface, fullscreen);
    E_NOTIMPL
}
unsafe extern "system" fn video_presenter_control_GetFullscreen(
    iface: *mut IMFVideoDisplayControl,
    fullscreen: *mut BOOL,
) -> HRESULT {
    // FIXME("%p, %p.\n", iface, fullscreen);
    E_NOTIMPL
}

unsafe extern "system" fn video_presenter_rate_support_QueryInterface(iface: *mut IMFRateSupport, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFRateSupport(iface);
    return IMFVideoPresenter_QueryInterface(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface), riid, obj);
}

unsafe extern "system" fn video_presenter_rate_support_AddRef(iface: *mut IMFRateSupport) -> ULONG {
    let presenter: *mut video_presenter = impl_from_IMFRateSupport(iface);
    return IMFVideoPresenter_AddRef(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface));
}

unsafe extern "system" fn video_presenter_rate_support_Release(iface: *mut IMFRateSupport) -> ULONG {
    let presenter: *mut video_presenter = impl_from_IMFRateSupport(iface);
    return IMFVideoPresenter_Release(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface));
}
unsafe extern "system" fn video_presenter_rate_support_GetSlowestRate(
    iface: *mut IMFRateSupport,
    direction: MFRATE_DIRECTION,
    thin: BOOL,
    rate: *mut f32,
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
            b"dll/win32/evr/presenter.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"video_presenter_rate_support_GetSlowestRate\0" as *const u8
                as *const ::core::ffi::c_char,
            1683 as ::core::ffi::c_int,
            b"%p, %d, %d, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            direction as ::core::ffi::c_uint,
            thin,
            rate,
        );
    }
    *rate = 0.0f32;
    return S_OK;
}
unsafe extern "system" fn video_presenter_rate_support_GetFastestRate(
    iface: *mut IMFRateSupport,
    direction: MFRATE_DIRECTION,
    thin: BOOL,
    rate: *mut FLOAT,
) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn video_presenter_rate_support_IsRateSupported(
    iface: *mut IMFRateSupport,
    thin: BOOL,
    rate: FLOAT,
    nearest_supported_rate: *mut FLOAT,
) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "system" fn video_presenter_getservice_QueryInterface(iface: *mut IMFGetService, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut presenter: *mut video_presenter = impl_from_IMFGetService(iface);
    return IMFVideoPresenter_QueryInterface(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface), riid, obj);
}

unsafe extern "system" fn video_presenter_getservice_AddRef(iface: *mut IMFGetService) -> ULONG {
    let presenter: *mut video_presenter = impl_from_IMFGetService(iface);
    return IMFVideoPresenter_AddRef(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface));
}

unsafe extern "system" fn video_presenter_getservice_Release(iface: *mut IMFGetService) -> ULONG {
    let mut presenter: *mut video_presenter = impl_from_IMFGetService(iface);
    return IMFVideoPresenter_Release(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface));
}

unsafe extern "system" fn video_presenter_getservice_GetService(iface: *mut IMFGetService, service: *const GUID, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFGetService(iface);

    if IsEqualGUID!(&MR_VIDEO_ACCELERATION_SERVICE, service) != false {
        return IDirect3DDeviceManager9_QueryInterface((*presenter).device_manager, riid, obj);
    }

    if IsEqualGUID!(&MR_VIDEO_RENDER_SERVICE, service) != false {
        return IMFVideoPresenter_QueryInterface(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface), riid, obj);
    }

    return MF_E_UNSUPPORTED_SERVICE;
}

unsafe extern "system" fn video_presenter_position_mapper_QueryInterface(iface: *mut IMFVideoPositionMapper, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFVideoPositionMapper(iface);
    return IMFVideoPresenter_QueryInterface(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface), riid, obj);
}

unsafe extern "system" fn video_presenter_position_mapper_AddRef(iface: *mut IMFVideoPositionMapper) -> ULONG {
    let presenter: *mut video_presenter = impl_from_IMFVideoPositionMapper(iface);
    return IMFVideoPresenter_AddRef(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface));
}

unsafe extern "system" fn video_presenter_position_mapper_Release(iface: *mut IMFVideoPositionMapper) -> ULONG {
    let presenter: *mut video_presenter = impl_from_IMFVideoPositionMapper(iface);
    return IMFVideoPresenter_Release(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface));
}
unsafe extern "system" fn video_presenter_position_mapper_MapOutputCoordinateToInputStream(
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
unsafe extern "system" fn video_presenter_allocator_cb_QueryInterface(
    iface: *mut IMFVideoSampleAllocatorNotify,
    riid: REFIID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFVideoSampleAllocatorNotify))
        != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = iface as *mut core::ffi::c_void;
        IMFVideoSampleAllocatorNotify_AddRef(iface);
        return S_OK;
    }

    WARN!(
        "Unsupported interface %s.\n",
        debugstr_guid(riid)
    );
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn video_presenter_allocator_cb_AddRef(iface: *mut IMFVideoSampleAllocatorNotify) -> ULONG {
    let mut presenter: *mut video_presenter = impl_from_IMFVideoSampleAllocatorNotify(iface);
    return IMFVideoPresenter_AddRef(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface));
}

unsafe extern "system" fn video_presenter_allocator_cb_Release(iface: *mut IMFVideoSampleAllocatorNotify) -> ULONG {
    let presenter: *mut video_presenter = impl_from_IMFVideoSampleAllocatorNotify(iface);
    return IMFVideoPresenter_Release(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface));
}

unsafe extern "system" fn video_presenter_allocator_cb_NotifyRelease(iface: *mut IMFVideoSampleAllocatorNotify) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFVideoSampleAllocatorNotify(iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));
    video_presenter_process_input(presenter);
    LeaveCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    return S_OK;
}

unsafe extern "system" fn video_presenter_qualprop_QueryInterface(iface: *mut IQualProp, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut presenter: *mut video_presenter = impl_from_IQualProp(iface);
    return IMFVideoPresenter_QueryInterface(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface), riid, obj);
}

unsafe extern "system" fn video_presenter_qualprop_AddRef(iface: *mut IQualProp) -> ULONG {
    let presenter: *mut video_presenter = impl_from_IQualProp(iface);
    return IMFVideoPresenter_AddRef(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface));
}

unsafe extern "system" fn video_presenter_qualprop_Release(iface: *mut IQualProp) -> ULONG {
    let presenter: *mut video_presenter = impl_from_IQualProp(iface);
    return IMFVideoPresenter_Release(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface));
}
unsafe extern "system" fn video_presenter_qualprop_get_FramesDroppedInRenderer(
    iface: *mut IQualProp,
    frames: *mut i32,
) -> HRESULT {
    // FIXME("%p, %p stub.\n", iface, frames);
    E_NOTIMPL
}

unsafe extern "system" fn video_presenter_qualprop_get_FramesDrawn(iface: *mut IQualProp, frames: *mut i32) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IQualProp(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!("%p, %p.\n", iface, frames);

    EnterCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    let st: u32 = (*presenter).state;
    if st == PRESENTER_STATE_STARTED || st == PRESENTER_STATE_PAUSED {
        if !frames.is_null() {
            *frames = (*presenter).frame_stats.presented;
        } else {
            hr = E_POINTER;
        }
    } else {
        hr = E_NOTIMPL;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*presenter).cs));

    return hr;
}
unsafe extern "system" fn video_presenter_qualprop_get_AvgFrameRate(
    iface: *mut IQualProp,
    avg_frame_rate: *mut i32,
) -> HRESULT {
    // FIXME("%p, %p stub.\n", iface, avg_frame_rate);
    E_NOTIMPL
}
unsafe extern "system" fn video_presenter_qualprop_get_Jitter(
    iface: *mut IQualProp,
    jitter: *mut i32,
) -> HRESULT {
    // FIXME("%p, %p stub.\n", iface, jitter);
    E_NOTIMPL
}
unsafe extern "system" fn video_presenter_qualprop_get_AvgSyncOffset(
    iface: *mut IQualProp,
    offset: *mut i32,
) -> HRESULT {
    // FIXME("%p, %p stub.\n", iface, offset);
    E_NOTIMPL
}
unsafe extern "system" fn video_presenter_qualprop_get_DevSyncOffset(
    iface: *mut IQualProp,
    devoffset: *mut i32,
) -> HRESULT {
    // FIXME("%p, %p stub.\n", iface, devoffset);
    E_NOTIMPL
}

unsafe extern "system" fn video_presenter_quality_advise_QueryInterface(iface: *mut IMFQualityAdvise, riid: *const IID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut presenter: *mut video_presenter = impl_from_IMFQualityAdvise(iface);
    return IMFVideoPresenter_QueryInterface(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface), riid, out);
}

unsafe extern "system" fn video_presenter_quality_advise_AddRef(iface: *mut IMFQualityAdvise) -> ULONG {
    let presenter: *mut video_presenter = impl_from_IMFQualityAdvise(iface);
    return IMFVideoPresenter_AddRef(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface));
}

unsafe extern "system" fn video_presenter_quality_advise_Release(iface: *mut IMFQualityAdvise) -> ULONG {
    let presenter: *mut video_presenter = impl_from_IMFQualityAdvise(iface);
    return IMFVideoPresenter_Release(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface));
}
unsafe extern "system" fn video_presenter_quality_advise_SetDropMode(
    iface: *mut IMFQualityAdvise,
    mode: MF_QUALITY_DROP_MODE,
) -> HRESULT {
    // FIXME("%p, %u.\n", iface, mode);
    E_NOTIMPL
}
unsafe extern "system" fn video_presenter_quality_advise_SetQualityLevel(
    iface: *mut IMFQualityAdvise,
    level: MF_QUALITY_LEVEL,
) -> HRESULT {
    // FIXME("%p, %u.\n", iface, level);
    E_NOTIMPL
}
unsafe extern "system" fn video_presenter_quality_advise_GetDropMode(
    iface: *mut IMFQualityAdvise,
    mode: *mut MF_QUALITY_DROP_MODE,
) -> HRESULT {
    // FIXME("%p, %p.\n", iface, mode);
    E_NOTIMPL
}
unsafe extern "system" fn video_presenter_quality_advise_GetQualityLevel(
    iface: *mut IMFQualityAdvise,
    level: *mut MF_QUALITY_LEVEL,
) -> HRESULT {
    // FIXME("%p, %p.\n", iface, level);
    E_NOTIMPL
}
unsafe extern "system" fn video_presenter_quality_advise_DropTime(
    iface: *mut IMFQualityAdvise,
    interval: LONGLONG,
) -> HRESULT {
    // FIXME("%p, %s.\n", iface, wine_dbgstr_longlong(interval));
    E_NOTIMPL
}

unsafe extern "system" fn video_presenter_device_manager_QueryInterface(iface: *mut IDirect3DDeviceManager9, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IDirect3DDeviceManager9(iface);
    return IMFVideoPresenter_QueryInterface(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface), riid, obj);
}

unsafe extern "system" fn video_presenter_device_manager_AddRef(iface: *mut IDirect3DDeviceManager9) -> ULONG {
    let presenter: *mut video_presenter = impl_from_IDirect3DDeviceManager9(iface);
    return IMFVideoPresenter_AddRef(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface));
}

unsafe extern "system" fn video_presenter_device_manager_Release(iface: *mut IDirect3DDeviceManager9) -> ULONG {
    let mut presenter: *mut video_presenter = impl_from_IDirect3DDeviceManager9(iface);
    return IMFVideoPresenter_Release(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface));
}

unsafe extern "system" fn video_presenter_device_manager_ResetDevice(iface: *mut IDirect3DDeviceManager9, device: *mut IDirect3DDevice9, token: UINT) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IDirect3DDeviceManager9(iface);
    return IDirect3DDeviceManager9_ResetDevice((*presenter).device_manager, device, token);
}

unsafe extern "system" fn video_presenter_device_manager_OpenDeviceHandle(iface: *mut IDirect3DDeviceManager9, hdevice: *mut HANDLE) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IDirect3DDeviceManager9(iface);
    return IDirect3DDeviceManager9_OpenDeviceHandle((*presenter).device_manager, hdevice);
}

unsafe extern "system" fn video_presenter_device_manager_CloseDeviceHandle(
    iface: *mut IDirect3DDeviceManager9,
    hdevice: HANDLE,
) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IDirect3DDeviceManager9(iface);
    return IDirect3DDeviceManager9_CloseDeviceHandle((*presenter).device_manager, hdevice);
}

unsafe extern "system" fn video_presenter_device_manager_TestDevice(iface: *mut IDirect3DDeviceManager9, hdevice: HANDLE) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IDirect3DDeviceManager9(iface);
    return IDirect3DDeviceManager9_TestDevice((*presenter).device_manager, hdevice);
}

unsafe extern "system" fn video_presenter_device_manager_LockDevice(iface: *mut IDirect3DDeviceManager9, hdevice: HANDLE, device: *mut *mut IDirect3DDevice9, block: BOOL) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IDirect3DDeviceManager9(iface);
    return IDirect3DDeviceManager9_LockDevice((*presenter).device_manager, hdevice, device, block);
}

unsafe extern "system" fn video_presenter_device_manager_UnlockDevice(iface: *mut IDirect3DDeviceManager9, hdevice: HANDLE, savestate: BOOL) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IDirect3DDeviceManager9(iface);
    return IDirect3DDeviceManager9_UnlockDevice((*presenter).device_manager, hdevice, savestate);
}
unsafe extern "system" fn video_presenter_device_manager_GetVideoService(
    iface: *mut IDirect3DDeviceManager9,
    hdevice: HANDLE,
    riid: *const IID,
    service: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IDirect3DDeviceManager9(iface);
    return IDirect3DDeviceManager9_GetVideoService((*presenter).device_manager, hdevice, riid, service);
}
unsafe extern "system" fn video_presenter_qa_limits_QueryInterface(iface: *mut IMFQualityAdviseLimits, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let presenter: *mut video_presenter = impl_from_IMFQualityAdviseLimits(iface);
    return IMFVideoPresenter_QueryInterface(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface), riid, obj);
}

unsafe extern "system" fn video_presenter_qa_limits_AddRef(mut iface: *mut IMFQualityAdviseLimits) -> ULONG {
    let mut presenter: *mut video_presenter = impl_from_IMFQualityAdviseLimits(iface);
    return IMFVideoPresenter_AddRef(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface));
}

unsafe extern "system" fn video_presenter_qa_limits_Release(iface: *mut IMFQualityAdviseLimits) -> ULONG {
    let presenter: *mut video_presenter = impl_from_IMFQualityAdviseLimits(iface);
    return IMFVideoPresenter_Release(core::ptr::addr_of_mut!((*presenter).IMFVideoPresenter_iface));
}
unsafe extern "system" fn video_presenter_qa_limits_GetMaximumDropMode(
    iface: *mut IMFQualityAdviseLimits,
    mode: *mut MF_QUALITY_DROP_MODE,
) -> HRESULT {
    // FIXME("%p, %p.\n", iface, mode);
    E_NOTIMPL
}
unsafe extern "system" fn video_presenter_qa_limits_GetMinimumQualityLevel(
    iface: *mut IMFQualityAdviseLimits,
    level: *mut MF_QUALITY_LEVEL,
) -> HRESULT {
    // FIXME("%p, %p.\n", iface, level);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateVideoPresenter(
    owner: *mut IUnknown,
    riid_device: *const IID,
    riid: *const IID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    *obj = core::ptr::null_mut();

    if IsEqualIID!(riid_device, core::ptr::addr_of!(IID_IDirect3DDevice9)) == 0 {
        return E_INVALIDARG;
    }

    return CoCreateInstance(
        core::ptr::addr_of!(CLSID_MFVideoPresenter9),
        owner as LPUNKNOWN,
        CLSCTX_INPROC_SERVER as DWORD,
        riid,
        obj as *mut LPVOID,
    );
}

unsafe extern "C" fn video_presenter_init_d3d(presenter: *mut video_presenter) -> HRESULT {
    let mut present_params: D3DPRESENT_PARAMETERS = core::mem::zeroed();
    let mut device: *mut IDirect3DDevice9 = core::mem::zeroed();
    let mut d3d: *mut IDirect3D9;
    let mut hr: HRESULT;

    d3d = Direct3DCreate9(D3D_SDK_VERSION);
    if d3d.is_null() {
        WARN!("Failed to initialize d3d9.\n");
        return E_FAIL;
    }

    present_params.BackBufferCount = 1;
    present_params.SwapEffect = D3DSWAPEFFECT_COPY;
    present_params.hDeviceWindow = GetDesktopWindow();
    present_params.Windowed = (TRUE as i32);
    present_params.Flags = D3DPRESENTFLAG_VIDEO;
    present_params.PresentationInterval = D3DPRESENT_INTERVAL_IMMEDIATE;
    hr = IDirect3D9_CreateDevice(
        d3d,
        D3DADAPTER_DEFAULT,
        D3DDEVTYPE_HAL,
        GetDesktopWindow(),
        (D3DCREATE_HARDWARE_VERTEXPROCESSING | D3DCREATE_MULTITHREADED) as DWORD,
        core::ptr::addr_of_mut!(present_params),
        core::ptr::addr_of_mut!(device),
    );

    IDirect3D9_Release(d3d);

    if hr < 0 {
        WARN!("Failed to create d3d device, hr %#lx.\n", hr);
        return hr;
    }

    hr = IDirect3DDeviceManager9_ResetDevice((*presenter).device_manager, device, (*presenter).reset_token);
    IDirect3DDevice9_Release(device);
    if hr < 0 {
        WARN!("Failed to set new device for the manager, hr %#lx.\n", hr);
    }

    hr = create_video_sample_allocator(
        (FALSE) as BOOL,
        core::ptr::addr_of!(IID_IMFVideoSampleAllocator),
        core::ptr::addr_of_mut!((*presenter).allocator) as *mut *mut core::ffi::c_void,
    );
    if hr >= 0 {
        hr = IMFVideoSampleAllocator_SetDirectXManager(
            (*presenter).allocator,
            (*presenter).device_manager as *mut IUnknown,
        );
    }

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn evr_presenter_create(outer: *mut IUnknown, out: *mut *mut ::core::ffi::c_void) -> HRESULT {
    let mut object: *mut video_presenter;
    let mut hr: HRESULT;

    *out = core::ptr::null_mut();

    object = calloc(1, (core::mem::size_of::<video_presenter>()) as _) as *mut video_presenter;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).IMFVideoPresenter_iface.lpVtbl = core::ptr::addr_of!(video_presenter_vtbl);
    (*object).IMFVideoDeviceID_iface.lpVtbl = core::ptr::addr_of!(video_presenter_device_id_vtbl);
    (*object).IMFTopologyServiceLookupClient_iface.lpVtbl = core::ptr::addr_of!(video_presenter_service_client_vtbl);
    (*object).IMFVideoDisplayControl_iface.lpVtbl = core::ptr::addr_of!(video_presenter_control_vtbl);
    (*object).IMFRateSupport_iface.lpVtbl = core::ptr::addr_of!(video_presenter_rate_support_vtbl);
    (*object).IMFGetService_iface.lpVtbl = core::ptr::addr_of!(video_presenter_getservice_vtbl);
    (*object).IMFVideoPositionMapper_iface.lpVtbl = core::ptr::addr_of!(video_presenter_position_mapper_vtbl);
    (*object).IQualProp_iface.lpVtbl = core::ptr::addr_of!(video_presenter_qualprop_vtbl);
    (*object).IMFQualityAdvise_iface.lpVtbl = core::ptr::addr_of!(video_presenter_quality_advise_vtbl);
    (*object).IMFQualityAdviseLimits_iface.lpVtbl = core::ptr::addr_of!(video_presenter_qa_limits_vtbl);
    (*object).allocator_cb.lpVtbl = core::ptr::addr_of!(video_presenter_allocator_cb_vtbl);
    (*object).IUnknown_inner.lpVtbl = core::ptr::addr_of!(video_presenter_inner_vtbl);
    (*object).IDirect3DDeviceManager9_iface.lpVtbl = core::ptr::addr_of!(video_presenter_device_manager_vtbl);
    (*object).outer_unk = if !outer.is_null() {
        outer
    } else {
        core::ptr::addr_of_mut!((*object).IUnknown_inner)
    };
    (*object).refcount = 1;
    (*object).src_rect.bottom = 1.0f32;
    (*object).src_rect.right = (*object).src_rect.bottom;
    // MFVideoARMode_PreservePicture (0x4) | MFVideoARMode_PreservePixel (0x2)
    (*object).ar_mode = 0x4 | 0x2;
    (*object).allocator_capacity = 3;
    InitializeCriticalSection(core::ptr::addr_of_mut!((*object).cs));

    'failed: {
        hr = DXVA2CreateDirect3DDeviceManager9(
            core::ptr::addr_of_mut!((*object).reset_token),
            core::ptr::addr_of_mut!((*object).device_manager),
        );
        if hr < 0 {
            break 'failed;
        }

        hr = video_presenter_init_d3d(object);
        if hr < 0 {
            WARN!("Failed to initialize d3d device, hr %#lx.\n", hr);
            break 'failed;
        }

        *out = core::ptr::addr_of_mut!((*object).IUnknown_inner) as *mut core::ffi::c_void;

        return S_OK;
    }

    IUnknown_Release(core::ptr::addr_of_mut!((*object).IUnknown_inner));

    return hr;
}

// COM vtable tables — generated from presenter.c by vtbl_gen.py
/// presenter.c:896 `static const IUnknownVtbl video_presenter_inner_vtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static video_presenter_inner_vtbl: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: Some(video_presenter_inner_QueryInterface),
    AddRef: Some(video_presenter_inner_AddRef),
    Release: Some(video_presenter_inner_Release),
};

/// presenter.c:1045 `static const IMFVideoPresenterVtbl video_presenter_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_presenter_vtbl: IMFVideoPresenterVtbl = IMFVideoPresenterVtbl {
    QueryInterface: Some(video_presenter_QueryInterface),
    AddRef: Some(video_presenter_AddRef),
    Release: Some(video_presenter_Release),
    OnClockStart: Some(video_presenter_OnClockStart),
    OnClockStop: Some(video_presenter_OnClockStop),
    OnClockPause: Some(video_presenter_OnClockPause),
    OnClockRestart: Some(video_presenter_OnClockRestart),
    OnClockSetRate: Some(video_presenter_OnClockSetRate),
    ProcessMessage: Some(video_presenter_ProcessMessage),
    GetCurrentMediaType: Some(video_presenter_GetCurrentMediaType),
};

/// presenter.c:1089 `static const IMFVideoDeviceIDVtbl video_presenter_device_id_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_presenter_device_id_vtbl: IMFVideoDeviceIDVtbl = IMFVideoDeviceIDVtbl {
    QueryInterface: Some(video_presenter_device_id_QueryInterface),
    AddRef: Some(video_presenter_device_id_AddRef),
    Release: Some(video_presenter_device_id_Release),
    GetDeviceID: Some(video_presenter_device_id_GetDeviceID),
};

/// presenter.c:1237 `static const IMFTopologyServiceLookupClientVtbl video_presenter_service_client_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_presenter_service_client_vtbl: IMFTopologyServiceLookupClientVtbl = IMFTopologyServiceLookupClientVtbl {
    QueryInterface: Some(video_presenter_service_client_QueryInterface),
    AddRef: Some(video_presenter_service_client_AddRef),
    Release: Some(video_presenter_service_client_Release),
    InitServicePointers: Some(video_presenter_service_client_InitServicePointers),
    ReleaseServicePointers: Some(video_presenter_service_client_ReleaseServicePointers),
};

/// presenter.c:1639 `static const IMFVideoDisplayControlVtbl video_presenter_control_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_presenter_control_vtbl: IMFVideoDisplayControlVtbl = IMFVideoDisplayControlVtbl {
    QueryInterface: Some(video_presenter_control_QueryInterface),
    AddRef: Some(video_presenter_control_AddRef),
    Release: Some(video_presenter_control_Release),
    GetNativeVideoSize: Some(video_presenter_control_GetNativeVideoSize),
    GetIdealVideoSize: Some(video_presenter_control_GetIdealVideoSize),
    SetVideoPosition: Some(video_presenter_control_SetVideoPosition),
    GetVideoPosition: Some(video_presenter_control_GetVideoPosition),
    SetAspectRatioMode: Some(video_presenter_control_SetAspectRatioMode),
    GetAspectRatioMode: Some(video_presenter_control_GetAspectRatioMode),
    SetVideoWindow: Some(video_presenter_control_SetVideoWindow),
    GetVideoWindow: Some(video_presenter_control_GetVideoWindow),
    RepaintVideo: Some(video_presenter_control_RepaintVideo),
    GetCurrentImage: Some(video_presenter_control_GetCurrentImage),
    SetBorderColor: Some(video_presenter_control_SetBorderColor),
    GetBorderColor: Some(video_presenter_control_GetBorderColor),
    SetRenderingPrefs: Some(video_presenter_control_SetRenderingPrefs),
    GetRenderingPrefs: Some(video_presenter_control_GetRenderingPrefs),
    SetFullscreen: Some(video_presenter_control_SetFullscreen),
    GetFullscreen: Some(video_presenter_control_GetFullscreen),
};

/// presenter.c:1702 `static const IMFRateSupportVtbl video_presenter_rate_support_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_presenter_rate_support_vtbl: IMFRateSupportVtbl = IMFRateSupportVtbl {
    QueryInterface: Some(video_presenter_rate_support_QueryInterface),
    AddRef: Some(video_presenter_rate_support_AddRef),
    Release: Some(video_presenter_rate_support_Release),
    GetSlowestRate: Some(video_presenter_rate_support_GetSlowestRate),
    GetFastestRate: Some(video_presenter_rate_support_GetFastestRate),
    IsRateSupported: Some(video_presenter_rate_support_IsRateSupported),
};

/// presenter.c:1747 `static const IMFGetServiceVtbl video_presenter_getservice_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_presenter_getservice_vtbl: IMFGetServiceVtbl = IMFGetServiceVtbl {
    QueryInterface: Some(video_presenter_getservice_QueryInterface),
    AddRef: Some(video_presenter_getservice_AddRef),
    Release: Some(video_presenter_getservice_Release),
    GetService: Some(video_presenter_getservice_GetService),
};

/// presenter.c:1781 `static const IMFVideoPositionMapperVtbl video_presenter_position_mapper_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_presenter_position_mapper_vtbl: IMFVideoPositionMapperVtbl = IMFVideoPositionMapperVtbl {
    QueryInterface: Some(video_presenter_position_mapper_QueryInterface),
    AddRef: Some(video_presenter_position_mapper_AddRef),
    Release: Some(video_presenter_position_mapper_Release),
    MapOutputCoordinateToInputStream: Some(video_presenter_position_mapper_MapOutputCoordinateToInputStream),
};

/// presenter.c:1828 `static const IMFVideoSampleAllocatorNotifyVtbl video_presenter_allocator_cb_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_presenter_allocator_cb_vtbl: IMFVideoSampleAllocatorNotifyVtbl = IMFVideoSampleAllocatorNotifyVtbl {
    QueryInterface: Some(video_presenter_allocator_cb_QueryInterface),
    AddRef: Some(video_presenter_allocator_cb_AddRef),
    Release: Some(video_presenter_allocator_cb_Release),
    NotifyRelease: Some(video_presenter_allocator_cb_NotifyRelease),
};

/// presenter.c:1914 `static const IQualPropVtbl video_presenter_qualprop_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_presenter_qualprop_vtbl: IQualPropVtbl = IQualPropVtbl {
    QueryInterface: Some(video_presenter_qualprop_QueryInterface),
    AddRef: Some(video_presenter_qualprop_AddRef),
    Release: Some(video_presenter_qualprop_Release),
    get_FramesDroppedInRenderer: Some(video_presenter_qualprop_get_FramesDroppedInRenderer),
    get_FramesDrawn: Some(video_presenter_qualprop_get_FramesDrawn),
    get_AvgFrameRate: Some(video_presenter_qualprop_get_AvgFrameRate),
    get_Jitter: Some(video_presenter_qualprop_get_Jitter),
    get_AvgSyncOffset: Some(video_presenter_qualprop_get_AvgSyncOffset),
    get_DevSyncOffset: Some(video_presenter_qualprop_get_DevSyncOffset),
};

/// presenter.c:1984 `static const IMFQualityAdviseVtbl video_presenter_quality_advise_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_presenter_quality_advise_vtbl: IMFQualityAdviseVtbl = IMFQualityAdviseVtbl {
    QueryInterface: Some(video_presenter_quality_advise_QueryInterface),
    AddRef: Some(video_presenter_quality_advise_AddRef),
    Release: Some(video_presenter_quality_advise_Release),
    SetDropMode: Some(video_presenter_quality_advise_SetDropMode),
    SetQualityLevel: Some(video_presenter_quality_advise_SetQualityLevel),
    GetDropMode: Some(video_presenter_quality_advise_GetDropMode),
    GetQualityLevel: Some(video_presenter_quality_advise_GetQualityLevel),
    DropTime: Some(video_presenter_quality_advise_DropTime),
};

/// presenter.c:2061 `static const IDirect3DDeviceManager9Vtbl video_presenter_device_manager_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_presenter_device_manager_vtbl: IDirect3DDeviceManager9Vtbl = IDirect3DDeviceManager9Vtbl {
    QueryInterface: Some(video_presenter_device_manager_QueryInterface),
    AddRef: Some(video_presenter_device_manager_AddRef),
    Release: Some(video_presenter_device_manager_Release),
    ResetDevice: Some(video_presenter_device_manager_ResetDevice),
    OpenDeviceHandle: Some(video_presenter_device_manager_OpenDeviceHandle),
    CloseDeviceHandle: Some(video_presenter_device_manager_CloseDeviceHandle),
    TestDevice: Some(video_presenter_device_manager_TestDevice),
    LockDevice: Some(video_presenter_device_manager_LockDevice),
    UnlockDevice: Some(video_presenter_device_manager_UnlockDevice),
    GetVideoService: Some(video_presenter_device_manager_GetVideoService),
};

/// presenter.c:2107 `static const IMFQualityAdviseLimitsVtbl video_presenter_qa_limits_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_presenter_qa_limits_vtbl: IMFQualityAdviseLimitsVtbl = IMFQualityAdviseLimitsVtbl {
    QueryInterface: Some(video_presenter_qa_limits_QueryInterface),
    AddRef: Some(video_presenter_qa_limits_AddRef),
    Release: Some(video_presenter_qa_limits_Release),
    GetMaximumDropMode: Some(video_presenter_qa_limits_GetMaximumDropMode),
    GetMinimumQualityLevel: Some(video_presenter_qa_limits_GetMinimumQualityLevel),
};
