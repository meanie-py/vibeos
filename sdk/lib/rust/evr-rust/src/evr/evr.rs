// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_51D5D1B98657: [u16; 11] = [
    0x0045u16, 0x0056, 0x0052, 0x0020, 0x0049, 0x006E, 0x0070, 0x0075, 0x0074, 0x0030,
    0x0000,
];

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

static mut renderer_ops: strmbase_renderer_ops = unsafe {
    strmbase_renderer_ops {
        renderer_query_accept: Some(
            evr_query_accept
                as unsafe extern "C" fn(*mut strmbase_renderer, *const AM_MEDIA_TYPE) -> HRESULT,
        ),
        renderer_render: Some(
            evr_render
                as unsafe extern "C" fn(*mut strmbase_renderer, *mut IMediaSample) -> HRESULT,
        ),
        renderer_init_stream: Some(
            evr_init_stream as unsafe extern "C" fn(*mut strmbase_renderer) -> (),
        ),
        renderer_start_stream: Some(
            evr_start_stream as unsafe extern "C" fn(*mut strmbase_renderer) -> (),
        ),
        renderer_stop_stream: Some(
            evr_stop_stream as unsafe extern "C" fn(*mut strmbase_renderer) -> (),
        ),
        renderer_connect: Some(
            evr_connect
                as unsafe extern "C" fn(*mut strmbase_renderer, *const AM_MEDIA_TYPE) -> HRESULT,
        ),
        renderer_disconnect: Some(
            evr_disconnect as unsafe extern "C" fn(*mut strmbase_renderer) -> (),
        ),
        renderer_destroy: Some(evr_destroy as unsafe extern "C" fn(*mut strmbase_renderer) -> ()),
        renderer_query_interface: Some(
            evr_query_interface
                as unsafe extern "C" fn(
                    *mut strmbase_renderer,
                    *const IID,
                    *mut *mut ::core::ffi::c_void,
                ) -> HRESULT,
        ),
        renderer_pin_query_interface: None,
    }
};

// #define COBJMACROS -- no value, nothing to emit
// WINE_DEFAULT_DEBUG_CHANNEL(evr); -- debug channel macro, not translated here

pub const EVR_INIT_SERVICES: i32 = 0x1; /* Currently in InitServices() call. */
pub const EVR_MIXER_INITED_SERVICES: i32 = 0x2;
pub const EVR_PRESENTER_INITED_SERVICES: i32 = 0x4;

/// evr.c includes dxva2api.h under initguid.h, which DEFINES the DXVA2 IIDs for the whole
/// module; mixer.c and sample.c only reference this one, so the per-TU translation emitted it
/// nowhere (i386 link: undefined _IID_IDirectXVideoProcessorService).
#[no_mangle]
pub static IID_IDirectXVideoProcessorService: GUID = _GUID {
    Data1: 0xfc51a552,
    Data2: 0xd5e7,
    Data3: 0x11d9,
    Data4: [0xaf, 0x55, 0x00, 0x05, 0x4e, 0x43, 0xff, 0x02],
};

/// C's `struct evr`, spelled as the model did in three accessors.
pub type struct_evr = evr;

#[repr(C)]
pub struct evr {
    pub renderer: strmbase_renderer,
    pub IEVRFilterConfig_iface: IEVRFilterConfig,
    pub IAMFilterMiscFlags_iface: IAMFilterMiscFlags,
    pub IMFGetService_iface: IMFGetService,
    pub IMFVideoRenderer_iface: IMFVideoRenderer,
    pub IMediaEventSink_iface: IMediaEventSink,
    pub IMFTopologyServiceLookup_iface: IMFTopologyServiceLookup,

    pub mixer: *mut IMFTransform,
    pub presenter: *mut IMFVideoPresenter,
    pub allocator: *mut IMFVideoSampleAllocator,
    pub media_type: *mut IMFMediaType,
    pub flags: u32,
}
pub type Pevr = *mut evr;

// static const IEVRFilterConfigVtbl filter_config_vtbl -- skipped (generated vtable)
// static const IAMFilterMiscFlagsVtbl filter_misc_flags_vtbl -- skipped (generated vtable)
// static const IMFGetServiceVtbl filter_get_service_vtbl -- skipped (generated vtable)
// static const IMFVideoRendererVtbl filter_video_renderer_vtbl -- skipped (generated vtable)
// static const IMediaEventSinkVtbl filter_media_event_sink_vtbl -- skipped (generated vtable)
// static const IMFTopologyServiceLookupVtbl filter_service_lookup_vtbl -- skipped (generated vtable)

unsafe extern "C" fn evr_uninitialize(filter: *mut evr) {
    if !(*filter).mixer.is_null() {
        IMFTransform_Release((*filter).mixer);
        (*filter).mixer = core::ptr::null_mut();
    }

    if !(*filter).presenter.is_null() {
        IMFVideoPresenter_Release((*filter).presenter);
        (*filter).presenter = core::ptr::null_mut();
    }
}

unsafe extern "C" fn evr_initialize(
    filter: *mut evr,
    mut mixer: *mut IMFTransform,
    mut presenter: *mut IMFVideoPresenter,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    if !mixer.is_null() {
        IMFTransform_AddRef(mixer);
    } else {
        hr = CoCreateInstance(
            core::ptr::addr_of!(CLSID_MFVideoMixer9),
            core::ptr::null_mut(),
            CLSCTX_INPROC_SERVER as DWORD,
            core::ptr::addr_of!(IID_IMFTransform),
            core::ptr::addr_of_mut!(mixer) as *mut LPVOID,
        );
        if hr < 0 {
            WARN!("Failed to create default mixer instance, hr %#lx.\n", hr);
            return hr;
        }
    }

    if !presenter.is_null() {
        IMFVideoPresenter_AddRef(presenter);
    } else {
        hr = CoCreateInstance(
            core::ptr::addr_of!(CLSID_MFVideoPresenter9),
            core::ptr::null_mut(),
            CLSCTX_INPROC_SERVER as DWORD,
            core::ptr::addr_of!(IID_IMFVideoPresenter),
            core::ptr::addr_of_mut!(presenter) as *mut LPVOID,
        );
        if hr < 0 {
            WARN!("Failed to create default presenter instance, hr %#lx.\n", hr);
            IMFTransform_Release(mixer);
            return hr;
        }
    }

    evr_uninitialize(filter);

    (*filter).mixer = mixer;
    (*filter).presenter = presenter;

    /* FIXME: configure mixer and presenter */

    return hr;
}

unsafe extern "C" fn impl_from_strmbase_renderer(iface: *mut strmbase_renderer) -> *mut evr {
    return CONTAINING_RECORD!(iface, evr, renderer);
}

unsafe extern "C" fn evr_query_interface(iface: *mut strmbase_renderer, iid: REFIID, out: *mut *mut ::core::ffi::c_void) -> HRESULT {
    let filter: *mut evr = impl_from_strmbase_renderer(iface);

    if IsEqualGUID!(iid, core::ptr::addr_of!(IID_IEVRFilterConfig)) != false {
        *out = core::ptr::addr_of_mut!((*filter).IEVRFilterConfig_iface) as *mut ::core::ffi::c_void;
    } else if IsEqualGUID!(iid, core::ptr::addr_of!(IID_IAMFilterMiscFlags)) != false {
        *out = core::ptr::addr_of_mut!((*filter).IAMFilterMiscFlags_iface) as *mut ::core::ffi::c_void;
    } else if IsEqualGUID!(iid, core::ptr::addr_of!(IID_IMFGetService)) != false {
        *out = core::ptr::addr_of_mut!((*filter).IMFGetService_iface) as *mut ::core::ffi::c_void;
    } else if IsEqualGUID!(iid, core::ptr::addr_of!(IID_IMFVideoRenderer)) != false {
        *out = core::ptr::addr_of_mut!((*filter).IMFVideoRenderer_iface) as *mut ::core::ffi::c_void;
    } else if IsEqualGUID!(iid, core::ptr::addr_of!(IID_IMediaEventSink)) != false {
        *out = core::ptr::addr_of_mut!((*filter).IMediaEventSink_iface) as *mut ::core::ffi::c_void;
    } else if IsEqualGUID!(iid, core::ptr::addr_of!(IID_IMFTopologyServiceLookup)) != false {
        *out = core::ptr::addr_of_mut!((*filter).IMFTopologyServiceLookup_iface) as *mut ::core::ffi::c_void;
    } else {
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*out as *mut IUnknown);
    return S_OK;
}

unsafe extern "C" fn evr_is_mixer_d3d_aware(filter: *const evr) -> BOOL {
    let mut attributes: *mut IMFAttributes = core::ptr::null_mut();
    let mut value: u32 = 0;
    let mut ret: BOOL = 0;

    if IMFTransform_QueryInterface(
        (*filter).mixer,
        &raw const IID_IMFAttributes,
        &raw mut attributes as *mut *mut core::ffi::c_void,
    ) < 0
    {
        return (FALSE) as BOOL;
    }

    ret = (IMFAttributes_GetUINT32(attributes, &raw const MF_SA_D3D_AWARE, &raw mut value)
        >= 0
        && value != 0) as ::core::ffi::c_int as BOOL;
    IMFAttributes_Release(attributes);
    return ret;
}

unsafe extern "C" fn evr_get_service(
    unk: *mut ::core::ffi::c_void,
    service: *const GUID,
    riid: *const IID,
    obj: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut gs: *mut IMFGetService = ::core::ptr::null_mut::<IMFGetService>();
    let mut hr: HRESULT = 0;
    hr = IUnknown_QueryInterface(
        unk as *mut IUnknown,
        &raw const IID_IMFGetService,
        &raw mut gs as *mut *mut ::core::ffi::c_void,
    );
    if hr >= 0 {
        hr = IMFGetService_GetService(gs, service, riid, obj as *mut LPVOID);
        IMFGetService_Release(gs);
    }
    return hr;
}

unsafe extern "C" fn evr_init_services(filter: *mut evr) -> HRESULT {
    let mut lookup_client: *mut IMFTopologyServiceLookupClient =
        core::ptr::null_mut::<IMFTopologyServiceLookupClient>();
    let mut hr: HRESULT = 0;

    if {
        hr = IMFTransform_QueryInterface(
            (*filter).mixer,
            &raw const IID_IMFTopologyServiceLookupClient,
            &raw mut lookup_client as *mut *mut ::core::ffi::c_void,
        );
        hr >= 0
    } {
        (*filter).flags |= EVR_INIT_SERVICES as ::core::ffi::c_uint;
        if {
            hr = IMFTopologyServiceLookupClient_InitServicePointers(
                lookup_client,
                &raw mut (*filter).IMFTopologyServiceLookup_iface,
            );
            hr >= 0
        } {
            (*filter).flags |= EVR_MIXER_INITED_SERVICES as ::core::ffi::c_uint;
        }
        (*filter).flags &= !(EVR_INIT_SERVICES as ::core::ffi::c_int) as ::core::ffi::c_uint;
        IMFTopologyServiceLookupClient_Release(lookup_client);
    }

    if hr < 0 {
        return hr;
    }

    /* Set device manager that presenter should have created. */
    if evr_is_mixer_d3d_aware(filter) != 0 {
        let mut device_manager: *mut IUnknown = core::ptr::null_mut::<IUnknown>();
        let mut gs: *mut IMFGetService = core::ptr::null_mut::<IMFGetService>();

        if IUnknown_QueryInterface(
            ((*filter).presenter).cast(),
            &raw const IID_IMFGetService,
            &raw mut gs as *mut *mut ::core::ffi::c_void,
        ) >= 0
        {
            if IMFGetService_GetService(
                gs,
                &raw const MR_VIDEO_RENDER_SERVICE,
                &raw const IID_IDirect3DDeviceManager9,
                &raw mut device_manager as *mut LPVOID,
            ) >= 0
            {
                IMFTransform_ProcessMessage(
                    (*filter).mixer,
                    MFT_MESSAGE_SET_D3D_MANAGER,
                    device_manager as ULONG_PTR,
                );
                IUnknown_Release(device_manager);
            }

            IMFGetService_Release(gs);
        }
    }

    if {
        hr = IMFVideoPresenter_QueryInterface(
            (*filter).presenter,
            &raw const IID_IMFTopologyServiceLookupClient,
            &raw mut lookup_client as *mut *mut ::core::ffi::c_void,
        );
        hr >= 0
    } {
        (*filter).flags |= EVR_INIT_SERVICES as ::core::ffi::c_uint;
        if {
            hr = IMFTopologyServiceLookupClient_InitServicePointers(
                lookup_client,
                &raw mut (*filter).IMFTopologyServiceLookup_iface,
            );
            hr >= 0
        } {
            (*filter).flags |= EVR_PRESENTER_INITED_SERVICES as ::core::ffi::c_uint;
        }
        (*filter).flags &= !(EVR_INIT_SERVICES as ::core::ffi::c_int) as ::core::ffi::c_uint;
        IMFTopologyServiceLookupClient_Release(lookup_client);
    }

    return hr;
}

unsafe extern "C" fn evr_release_services(filter: *mut evr) {
    let mut lookup_client: *mut IMFTopologyServiceLookupClient =
        ::core::ptr::null_mut::<IMFTopologyServiceLookupClient>();
    if ((*filter).flags & (EVR_MIXER_INITED_SERVICES as ::core::ffi::c_uint)) != 0
        && (IMFTransform_QueryInterface(
            (*filter).mixer,
            &raw const IID_IMFTopologyServiceLookupClient,
            &raw mut lookup_client as *mut *mut ::core::ffi::c_void,
        ) >= (0 as HRESULT))
    {
        IMFTopologyServiceLookupClient_ReleaseServicePointers(lookup_client);
        IMFTopologyServiceLookupClient_Release(lookup_client);
        (*filter).flags &= !(EVR_MIXER_INITED_SERVICES as ::core::ffi::c_uint);
    }
    if ((*filter).flags & (EVR_PRESENTER_INITED_SERVICES as ::core::ffi::c_uint)) != 0
        && (IMFVideoPresenter_QueryInterface(
            (*filter).presenter,
            &raw const IID_IMFTopologyServiceLookupClient,
            &raw mut lookup_client as *mut *mut ::core::ffi::c_void,
        ) >= (0 as HRESULT))
    {
        IMFTopologyServiceLookupClient_ReleaseServicePointers(lookup_client);
        IMFTopologyServiceLookupClient_Release(lookup_client);
        (*filter).flags &= !(EVR_PRESENTER_INITED_SERVICES as ::core::ffi::c_uint);
    }
}

unsafe extern "C" fn evr_set_input_type(filter: *mut evr, media_type: *mut IMFMediaType) {
    if !(*filter).media_type.is_null() {
        IMFMediaType_Release((*filter).media_type);
    }
    (*filter).media_type = media_type;
    if !(*filter).media_type.is_null() {
        IMFMediaType_AddRef((*filter).media_type);
    }
    if media_type.is_null() && !(*filter).allocator.is_null() {
        IMFVideoSampleAllocator_UninitializeSampleAllocator((*filter).allocator);
    }
}

unsafe extern "C" fn evr_test_input_type(filter: *mut evr, mt: *const AM_MEDIA_TYPE, ret: *mut *mut IMFMediaType) -> HRESULT {
    let mut media_type: *mut IMFMediaType = core::ptr::null_mut();
    let mut hr: HRESULT = S_OK;

    if (*filter).presenter.is_null() {
        hr = evr_initialize(filter, core::ptr::null_mut(), core::ptr::null_mut());
    }

    if hr >= 0 {
        hr = evr_init_services(filter);
    }

    if hr >= 0 {
        hr = MFCreateMediaType(core::ptr::addr_of_mut!(media_type));
    }

    if hr >= 0 {
        hr = MFInitMediaTypeFromAMMediaType(media_type, mt);
        if hr >= 0 {
            /* TODO: some pin -> mixer input mapping is necessary to test the substreams. */
            hr = IMFTransform_SetInputType((*filter).mixer, 0 as DWORD, media_type, MFT_SET_TYPE_TEST_ONLY as DWORD);
            if hr >= 0 {
                if !ret.is_null() {
                    *ret = media_type;
                    IMFMediaType_AddRef(*ret);
                }
            }
        }

        IMFMediaType_Release(media_type);
    }

    return hr;
}

unsafe extern "C" fn evr_connect(iface: *mut strmbase_renderer, mt: *const AM_MEDIA_TYPE) -> HRESULT {
    let filter: *mut evr = impl_from_strmbase_renderer(iface);
    let mut allocator: *mut IMFVideoSampleAllocator = core::ptr::null_mut();
    let mut media_type: *mut IMFMediaType = core::ptr::null_mut();
    let mut device_manager: *mut IUnknown = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    hr = evr_test_input_type(filter, mt, &raw mut media_type);
    if hr >= 0 {
        hr = IMFTransform_SetInputType((*filter).mixer, 0, media_type, 0);
        if hr >= 0 {
            hr = IMFVideoPresenter_ProcessMessage((*filter).presenter, MFVP_MESSAGE_INVALIDATEMEDIATYPE, 0);
        }

        hr = MFCreateVideoSampleAllocator(&raw const IID_IMFVideoSampleAllocator, &raw mut allocator as *mut *mut ::core::ffi::c_void);
        if hr >= 0 {
            hr = evr_get_service(
                (*filter).presenter as *mut ::core::ffi::c_void,
                &raw const MR_VIDEO_RENDER_SERVICE,
                &raw const IID_IDirect3DDeviceManager9,
                &raw mut device_manager as *mut *mut ::core::ffi::c_void,
            );
            if hr >= 0 {
                hr = IMFVideoSampleAllocator_SetDirectXManager(allocator, device_manager);
                if hr >= 0 {
                    hr = IMFVideoSampleAllocator_InitializeSampleAllocator(allocator, 2, media_type);
                    if hr >= 0 {
                        (*filter).allocator = allocator;
                        IMFVideoSampleAllocator_AddRef((*filter).allocator);
                    }
                }
                IUnknown_Release(device_manager);
            }

            IMFVideoSampleAllocator_Release(allocator);
        }

        if hr >= 0 {
            evr_set_input_type(filter, media_type);
        }

        IMFMediaType_Release(media_type);
    }

    return hr;
}

unsafe extern "C" fn evr_disconnect(iface: *mut strmbase_renderer) {
    let filter: *mut evr = impl_from_strmbase_renderer(iface);

    if !(*filter).mixer.is_null() {
        IMFTransform_SetInputType(
            (*filter).mixer,
            0 as DWORD,
            core::ptr::null_mut(),
            0 as DWORD,
        );
    }
    evr_set_input_type(filter, core::ptr::null_mut());
    evr_release_services(filter);
}

unsafe extern "C" fn evr_destroy(iface: *mut strmbase_renderer) {
    let filter: *mut evr = impl_from_strmbase_renderer(iface);

    evr_uninitialize(filter);
    evr_set_input_type(filter, core::ptr::null_mut());
    if !(*filter).allocator.is_null() {
        IMFVideoSampleAllocator_Release((*filter).allocator);
    }
    strmbase_renderer_cleanup(core::ptr::addr_of_mut!((*filter).renderer));
    free(filter as *mut core::ffi::c_void);
}

unsafe extern "C" fn evr_copy_sample_buffer(
    filter: *mut evr,
    subtype: *const GUID,
    input_sample: *mut IMediaSample,
    sample: *mut *mut IMFSample,
) -> HRESULT {
    let mut surface: *mut IDirect3DSurface9 = core::ptr::null_mut();
    let mut locked_rect: D3DLOCKED_RECT = core::mem::zeroed();
    let mut buffer: *mut IMFMediaBuffer = core::ptr::null_mut();
    let mut frame_size: UINT64 = 0;
    let mut width: UINT32 = 0;
    let mut lines: UINT32 = 0;
    let mut src_stride: LONG = 0;
    let mut hr: HRESULT = 0;
    let mut src: *mut BYTE = core::ptr::null_mut();
    let mut dst_uv: *mut BYTE = core::ptr::null_mut();

    hr = IMFMediaType_GetUINT32(
        (*filter).media_type,
        core::ptr::addr_of!(MF_MT_DEFAULT_STRIDE),
        core::ptr::addr_of_mut!(src_stride) as *mut UINT32,
    );
    if hr < 0 as HRESULT {
        WARN!("Unknown input buffer stride.\n");
        return hr;
    }
    IMFMediaType_GetUINT64(
        (*filter).media_type,
        core::ptr::addr_of!(MF_MT_FRAME_SIZE),
        core::ptr::addr_of_mut!(frame_size),
    );
    width = (frame_size >> 32) as UINT32;
    lines = frame_size as UINT32;

    hr = IMediaSample_GetPointer(input_sample, core::ptr::addr_of_mut!(src));
    if hr < 0 as HRESULT {
        WARN!("Failed to get pointer to sample data, hr %#lx.\n", hr);
        return hr;
    }

    hr = IMFVideoSampleAllocator_AllocateSample((*filter).allocator, sample);
    if hr < 0 as HRESULT {
        WARN!("Failed to allocate a sample, hr %#lx.\n", hr);
        return hr;
    }

    hr = IMFSample_GetBufferByIndex(*sample, 0 as DWORD, core::ptr::addr_of_mut!(buffer));
    if hr >= 0 as HRESULT {
        hr = evr_get_service(
            buffer as *mut core::ffi::c_void,
            core::ptr::addr_of!(MR_BUFFER_SERVICE),
            core::ptr::addr_of!(IID_IDirect3DSurface9),
            core::ptr::addr_of_mut!(surface) as *mut *mut core::ffi::c_void,
        );
        if hr >= 0 as HRESULT {
            hr = IDirect3DSurface9_LockRect(
                surface,
                core::ptr::addr_of_mut!(locked_rect),
                core::ptr::null_mut(),
                D3DLOCK_DISCARD as DWORD,
            );
            if hr >= 0 as HRESULT {
                if src_stride < 0 as LONG {
                    src = src.offset(
                        ((-src_stride) as usize)
                            .wrapping_mul(lines.wrapping_sub(1 as UINT32) as usize)
                            as isize,
                    );
                }

                if IsEqualGUID!(subtype, core::ptr::addr_of!(MFVideoFormat_YUY2)) != false {
                    width = width.wrapping_add(1 as UINT32)
                        & !(1 as UINT32);
                    MFCopyImage(
                        locked_rect.pBits as *mut BYTE,
                        locked_rect.Pitch as LONG,
                        src,
                        src_stride,
                        width.wrapping_mul(2 as UINT32) as DWORD,
                        lines as DWORD,
                    );
                } else if IsEqualGUID!(subtype, core::ptr::addr_of!(MFVideoFormat_NV12)) != false {
                    /* Width and height must be rounded up to even. */
                    width = width.wrapping_add(1 as UINT32)
                        & !(1 as UINT32);
                    lines = lines.wrapping_add(1 as UINT32)
                        & !(1 as UINT32);

                    /* Y plane */
                    MFCopyImage(
                        locked_rect.pBits as *mut BYTE,
                        locked_rect.Pitch as LONG,
                        src,
                        src_stride,
                        width as DWORD,
                        lines as DWORD,
                    );

                    /* UV plane */
                    dst_uv = (locked_rect.pBits as *mut BYTE)
                        .offset(lines.wrapping_mul(locked_rect.Pitch as UINT32) as isize);
                    MFCopyImage(
                        dst_uv,
                        locked_rect.Pitch as LONG,
                        src.offset(
                            (lines as usize).wrapping_mul(src_stride as usize) as isize,
                        ),
                        src_stride,
                        width as DWORD,
                        (lines / (2 as UINT32)) as DWORD,
                    );
                } else {
                    /* All other formats are 32-bit, single plane. */
                    MFCopyImage(
                        locked_rect.pBits as *mut BYTE,
                        locked_rect.Pitch as LONG,
                        src,
                        src_stride,
                        width.wrapping_mul(4 as UINT32) as DWORD,
                        lines as DWORD,
                    );
                }
                IDirect3DSurface9_UnlockRect(surface);
            }

            IDirect3DSurface9_Release(surface);
        }
        IMFMediaBuffer_Release(buffer);
    }

    if hr < 0 as HRESULT {
        IMFSample_Release(*sample);
        *sample = core::ptr::null_mut();
    }

    return hr;
}

unsafe extern "C" fn evr_render(iface: *mut strmbase_renderer, input_sample: *mut IMediaSample) -> HRESULT {
    let filter: *mut evr = impl_from_strmbase_renderer(iface);
    let mut hr: HRESULT = E_NOTIMPL;
    let mut subtype: GUID = GUID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    };
    let mut sample: *mut IMFSample = core::ptr::null_mut();

    if (*filter).media_type.is_null() {
        WARN!("Media type wasn't set.\n");
        return E_UNEXPECTED;
    }

    IMFMediaType_GetGUID(
        (*filter).media_type,
        core::ptr::addr_of!(MF_MT_SUBTYPE),
        core::ptr::addr_of_mut!(subtype),
    );

    if IsEqualGUID!(core::ptr::addr_of_mut!(subtype), core::ptr::addr_of!(MFVideoFormat_ARGB32)) != false
        || IsEqualGUID!(core::ptr::addr_of_mut!(subtype), core::ptr::addr_of!(MFVideoFormat_RGB32)) != false
        || IsEqualGUID!(core::ptr::addr_of_mut!(subtype), core::ptr::addr_of!(MFVideoFormat_YUY2)) != false
        || IsEqualGUID!(core::ptr::addr_of_mut!(subtype), core::ptr::addr_of!(MFVideoFormat_NV12)) != false
    {
        hr = evr_copy_sample_buffer(
            filter,
            core::ptr::addr_of_mut!(subtype),
            input_sample,
            core::ptr::addr_of_mut!(sample),
        );
        if hr >= 0 {
            if IMFTransform_ProcessInput(
                (*filter).mixer,
                0,
                sample,
                0,
            ) >= 0
            {
                IMFVideoPresenter_ProcessMessage(
                    (*filter).presenter,
                    MFVP_MESSAGE_PROCESSINPUTNOTIFY,
                    0,
                );
            }

            IMFSample_Release(sample);
        }
    } else {
        FIXME!(
            "Unhandled input type %s.\n",
            debugstr_guid(core::ptr::addr_of_mut!(subtype))
        );
    }

    return hr;
}

unsafe extern "C" fn evr_query_accept(iface: *mut strmbase_renderer, mt: *const AM_MEDIA_TYPE) -> HRESULT {
    let mut filter: *mut evr = impl_from_strmbase_renderer(iface);
    let mut hr: HRESULT;

    EnterCriticalSection(core::ptr::addr_of_mut!((*filter).renderer.filter.filter_cs));

    hr = evr_test_input_type(filter, mt, core::ptr::null_mut());
    evr_release_services(filter);

    LeaveCriticalSection(core::ptr::addr_of_mut!((*filter).renderer.filter.filter_cs));

    return hr;
}

unsafe extern "C" fn evr_init_stream(iface: *mut strmbase_renderer) {
    let filter: *mut evr = impl_from_strmbase_renderer(iface);

    if (*filter).mixer.is_null() {
        return;
    }

    if (IMFTransform_ProcessMessage((*filter).mixer, MFT_MESSAGE_NOTIFY_BEGIN_STREAMING, 0)) >= 0 {
        IMFVideoPresenter_ProcessMessage((*filter).presenter, MFVP_MESSAGE_BEGINSTREAMING, 0);
    }
}

unsafe extern "C" fn evr_start_stream(iface: *mut strmbase_renderer) {
    let filter: *mut evr = impl_from_strmbase_renderer(iface);

    if !(*filter).mixer.is_null() {
        IMFTransform_ProcessMessage((*filter).mixer, MFT_MESSAGE_NOTIFY_START_OF_STREAM, 0);
    }
}

unsafe extern "C" fn evr_stop_stream(iface: *mut strmbase_renderer) {
    let filter: *mut evr = impl_from_strmbase_renderer(iface);

    if (*filter).mixer.is_null() {
        return;
    }

    if (IMFTransform_ProcessMessage((*filter).mixer, MFT_MESSAGE_NOTIFY_END_OF_STREAM, 0)) >= 0
    {
        if (IMFVideoPresenter_ProcessMessage((*filter).presenter, MFVP_MESSAGE_ENDSTREAMING, 0)) >= 0
        {
            IMFTransform_ProcessMessage((*filter).mixer, MFT_MESSAGE_NOTIFY_END_STREAMING, 0);
        }
    }
}

unsafe extern "C" fn impl_from_IEVRFilterConfig(iface: *mut IEVRFilterConfig) -> *mut evr {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(evr, IEVRFilterConfig_iface)) as *mut evr;
}

unsafe extern "system" fn filter_config_QueryInterface(iface: *mut IEVRFilterConfig, iid: *const GUID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let filter: *mut evr = impl_from_IEVRFilterConfig(iface);
    return IUnknown_QueryInterface((*filter).renderer.filter.outer_unk, iid, out);
}

unsafe extern "system" fn filter_config_AddRef(iface: *mut IEVRFilterConfig) -> ULONG {
    let filter: *mut evr = impl_from_IEVRFilterConfig(iface);
    return IUnknown_AddRef((*filter).renderer.filter.outer_unk);
}
unsafe extern "system" fn filter_config_Release(iface: *mut IEVRFilterConfig) -> ULONG {
    let filter: *mut struct_evr = impl_from_IEVRFilterConfig(iface);
    return IUnknown_Release((*filter).renderer.filter.outer_unk);
}
unsafe extern "system" fn filter_config_SetNumberOfStreams(iface: *mut IEVRFilterConfig, count: DWORD) -> HRESULT {
    let filter: *mut evr = impl_from_IEVRFilterConfig(iface);

    FIXME!("filter %p, count %lu, stub!\n", filter, count);

    return E_NOTIMPL;
}

unsafe extern "system" fn filter_config_GetNumberOfStreams(iface: *mut IEVRFilterConfig, count: *mut DWORD) -> HRESULT {
    let filter: *mut evr = impl_from_IEVRFilterConfig(iface);
    FIXME!("filter %p, count %p, stub!\n", filter, count);
    return E_NOTIMPL;
}

unsafe extern "C" fn impl_from_IAMFilterMiscFlags(iface: *mut IAMFilterMiscFlags) -> *mut evr {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(evr, IAMFilterMiscFlags_iface)) as *mut evr;
}

unsafe extern "system" fn filter_misc_flags_QueryInterface(iface: *mut IAMFilterMiscFlags, iid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let filter: *mut evr = impl_from_IAMFilterMiscFlags(iface);
    return IUnknown_QueryInterface((*filter).renderer.filter.outer_unk, iid, out);
}

unsafe extern "system" fn filter_misc_flags_AddRef(iface: *mut IAMFilterMiscFlags) -> ULONG {
    let filter: *mut evr = impl_from_IAMFilterMiscFlags(iface);
    return IUnknown_AddRef((*filter).renderer.filter.outer_unk);
}
unsafe extern "system" fn filter_misc_flags_Release(iface: *mut IAMFilterMiscFlags) -> ULONG {
    let filter: *mut evr = impl_from_IAMFilterMiscFlags(iface);
    return IUnknown_Release((*filter).renderer.filter.outer_unk);
}
unsafe extern "system" fn filter_misc_flags_GetMiscFlags(
    iface: *mut IAMFilterMiscFlags,
) -> ULONG {
    // TRACE("%p.\n", iface);
    AM_FILTER_MISC_FLAGS_IS_RENDERER
}

unsafe extern "C" fn impl_from_IMFGetService(iface: *mut IMFGetService) -> *mut evr {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(evr, IMFGetService_iface)) as *mut evr;
}

unsafe extern "system" fn filter_get_service_QueryInterface(iface: *mut IMFGetService, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let filter: *mut evr = impl_from_IMFGetService(iface);
    return IUnknown_QueryInterface((*filter).renderer.filter.outer_unk, riid, obj);
}

unsafe extern "system" fn filter_get_service_AddRef(iface: *mut IMFGetService) -> ULONG {
    let filter: *mut evr = impl_from_IMFGetService(iface);
    return IUnknown_AddRef((*filter).renderer.filter.outer_unk);
}

unsafe extern "system" fn filter_get_service_Release(iface: *mut IMFGetService) -> ULONG {
    let filter: *mut evr = impl_from_IMFGetService(iface);
    return IUnknown_Release((*filter).renderer.filter.outer_unk);
}

unsafe extern "system" fn filter_get_service_GetService(
    mut iface: *mut IMFGetService,
    mut service: *const GUID,
    mut riid: *const IID,
    mut obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut filter: *mut evr = impl_from_IMFGetService(iface);
    let mut hr: HRESULT = E_NOINTERFACE;
    let mut gs: *mut IMFGetService = core::ptr::null_mut();

    TRACE!(
        "iface %p, service %s, riid %s, obj %p.\n",
        iface,
        debugstr_guid(service),
        debugstr_guid(riid),
        obj
    );

    EnterCriticalSection(&raw mut (*filter).renderer.filter.filter_cs);

    if IsEqualGUID!(service, &raw const MR_VIDEO_RENDER_SERVICE) != false {
        if (*filter).presenter.is_null() {
            hr = evr_initialize(
                filter,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
        }

        if !(*filter).presenter.is_null() {
            hr = IMFVideoPresenter_QueryInterface(
                (*filter).presenter,
                &raw const IID_IMFGetService,
                &raw mut gs as *mut *mut core::ffi::c_void,
            );
        }
    } else {
        FIXME!("Unsupported service %s.\n", debugstr_guid(service));
    }

    LeaveCriticalSection(&raw mut (*filter).renderer.filter.filter_cs);

    if !gs.is_null() {
        hr = IMFGetService_GetService(gs, service, riid, obj as *mut LPVOID);
        IMFGetService_Release(gs);
    }

    return hr;
}

unsafe extern "C" fn impl_from_IMFVideoRenderer(iface: *mut IMFVideoRenderer) -> *mut evr {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(evr, IMFVideoRenderer_iface)) as *mut evr;
}
unsafe extern "system" fn filter_video_renderer_QueryInterface(iface: *mut IMFVideoRenderer, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let filter: *mut evr = impl_from_IMFVideoRenderer(iface);
    return IUnknown_QueryInterface((*filter).renderer.filter.outer_unk, riid, obj);
}

unsafe extern "system" fn filter_video_renderer_AddRef(iface: *mut IMFVideoRenderer) -> ULONG {
    let filter: *mut evr = impl_from_IMFVideoRenderer(iface);
    return IUnknown_AddRef((*filter).renderer.filter.outer_unk);
}

unsafe extern "system" fn filter_video_renderer_Release(iface: *mut IMFVideoRenderer) -> ULONG {
    let filter: *mut struct_evr = impl_from_IMFVideoRenderer(iface);
    return IUnknown_Release((*filter).renderer.filter.outer_unk);
}

unsafe extern "system" fn filter_video_renderer_InitializeRenderer(
    mut iface: *mut IMFVideoRenderer,
    mut mixer: *mut IMFTransform,
    mut presenter: *mut IMFVideoPresenter,
) -> HRESULT {
    let mut filter: *mut evr = impl_from_IMFVideoRenderer(iface);
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
            b"dll/win32/evr/evr.c\0" as *const u8
                as *const ::core::ffi::c_char,
            b"filter_video_renderer_InitializeRenderer\0" as *const u8
                as *const ::core::ffi::c_char,
            707 as ::core::ffi::c_int,
            b"iface %p, mixer %p, presenter %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            mixer,
            presenter,
        );
    }
    EnterCriticalSection(&raw mut (*filter).renderer.filter.filter_cs);
    hr = evr_initialize(filter, mixer, presenter);
    LeaveCriticalSection(&raw mut (*filter).renderer.filter.filter_cs);
    return hr;
}

unsafe extern "C" fn impl_from_IMediaEventSink(mut iface: *mut IMediaEventSink) -> *mut evr {
    return (iface as *mut u8).sub(core::mem::offset_of!(evr, IMediaEventSink_iface)) as *mut evr;
}

unsafe extern "system" fn filter_media_event_sink_QueryInterface(iface: *mut IMediaEventSink, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let filter: *mut evr = impl_from_IMediaEventSink(iface);
    return IUnknown_QueryInterface((*filter).renderer.filter.outer_unk, riid, obj);
}

unsafe extern "system" fn filter_media_event_sink_AddRef(iface: *mut IMediaEventSink) -> ULONG {
    let filter: *mut struct_evr = impl_from_IMediaEventSink(iface);
    return IUnknown_AddRef((*filter).renderer.filter.outer_unk);
}

unsafe extern "system" fn filter_media_event_sink_Release(iface: *mut IMediaEventSink) -> ULONG {
    let filter: *mut evr = impl_from_IMediaEventSink(iface);
    return IUnknown_Release((*filter).renderer.filter.outer_unk);
}
unsafe extern "system" fn filter_media_event_sink_Notify(
    iface: *mut IMediaEventSink,
    event: LONG,
    param1: LONG_PTR,
    param2: LONG_PTR,
) -> HRESULT {
    // FIXME("iface %p, event %ld, param1 %Id, param2 %Id.\n", iface, event, param1, param2);
    E_NOTIMPL
}

unsafe extern "C" fn impl_from_IMFTopologyServiceLookup(iface: *mut IMFTopologyServiceLookup) -> *mut evr {
    return (iface as *mut u8).sub(core::mem::offset_of!(evr, IMFTopologyServiceLookup_iface)) as *mut evr;
}

unsafe extern "system" fn filter_service_lookup_QueryInterface(iface: *mut IMFTopologyServiceLookup, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let filter: *mut evr = impl_from_IMFTopologyServiceLookup(iface);
    return IUnknown_QueryInterface((*filter).renderer.filter.outer_unk, riid, obj);
}
unsafe extern "system" fn filter_service_lookup_AddRef(iface: *mut IMFTopologyServiceLookup) -> ULONG {
    let filter: *mut evr = impl_from_IMFTopologyServiceLookup(iface);
    return IUnknown_AddRef((*filter).renderer.filter.outer_unk);
}

unsafe extern "system" fn filter_service_lookup_Release(iface: *mut IMFTopologyServiceLookup) -> ULONG {
    let filter: *mut evr = impl_from_IMFTopologyServiceLookup(iface);
    return IUnknown_Release((*filter).renderer.filter.outer_unk);
}

unsafe extern "system" fn filter_service_lookup_LookupService(iface: *mut IMFTopologyServiceLookup, lookup_type: MF_SERVICE_LOOKUP_TYPE, index: DWORD, service: REFGUID, riid: REFIID, objects: *mut *mut ::core::ffi::c_void, num_objects: *mut DWORD) -> HRESULT {
    let filter: *mut evr = impl_from_IMFTopologyServiceLookup(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*filter).renderer.filter.filter_cs));

    if ((*filter).flags & (EVR_INIT_SERVICES as u32)) == 0
    {
        hr = MF_E_NOTACCEPTING;
    }
    else if IsEqualGUID!(service, &MR_VIDEO_RENDER_SERVICE)
    {
        if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMediaEventSink))) != 0)
        {
            *objects = core::ptr::addr_of_mut!((*filter).IMediaEventSink_iface) as *mut ::core::ffi::c_void;
            IUnknown_AddRef(*objects as *mut IUnknown);
        }
        else
        {
            WARN!("Unsupported interface %s for MR_VIDEO_RENDER_SERVICE.\n", debugstr_guid(riid));
            hr = E_NOINTERFACE;
        }
    }
    else if IsEqualGUID!(service, &MR_VIDEO_MIXER_SERVICE)
    {
        hr = IMFTransform_QueryInterface((*filter).mixer, riid, objects);
    }
    else
    {
        WARN!("Unsupported service %s.\n", debugstr_guid(service));
        hr = MF_E_UNSUPPORTED_SERVICE;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*filter).renderer.filter.filter_cs));

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn evr_filter_create(outer: *mut IUnknown, out: *mut *mut ::core::ffi::c_void) -> HRESULT {
    let mut object: *mut evr = core::ptr::null_mut::<evr>();

    object = calloc(1, (core::mem::size_of::<evr>()) as _) as *mut evr;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    strmbase_renderer_init(
        &raw mut (*object).renderer,
        outer,
        &raw const CLSID_EnhancedVideoRenderer,
        RUSTCONV_WSTR_51D5D1B98657.as_ptr(),
        &raw const renderer_ops,
    );
    (*object).IEVRFilterConfig_iface.lpVtbl = &raw const filter_config_vtbl;
    (*object).IAMFilterMiscFlags_iface.lpVtbl = &raw const filter_misc_flags_vtbl;
    (*object).IMFGetService_iface.lpVtbl = &raw const filter_get_service_vtbl;
    (*object).IMFVideoRenderer_iface.lpVtbl = &raw const filter_video_renderer_vtbl;
    (*object).IMediaEventSink_iface.lpVtbl = &raw const filter_media_event_sink_vtbl;
    (*object).IMFTopologyServiceLookup_iface.lpVtbl = &raw const filter_service_lookup_vtbl;

    TRACE!("Created EVR %p.\n", object);

    *out = &raw mut (*object).renderer.filter.IUnknown_inner as *mut core::ffi::c_void;
    return S_OK;
}

// COM vtable tables — generated from evr.c by vtbl_gen.py
/// evr.c:566 `static const IEVRFilterConfigVtbl filter_config_vtbl`  (slot order: rust:types_amd64.rs)
pub static filter_config_vtbl: IEVRFilterConfigVtbl = IEVRFilterConfigVtbl {
    QueryInterface: Some(filter_config_QueryInterface),
    AddRef: Some(filter_config_AddRef),
    Release: Some(filter_config_Release),
    SetNumberOfStreams: Some(filter_config_SetNumberOfStreams),
    GetNumberOfStreams: Some(filter_config_GetNumberOfStreams),
};

/// evr.c:605 `static const IAMFilterMiscFlagsVtbl filter_misc_flags_vtbl`  (slot order: rust:types_amd64.rs)
pub static filter_misc_flags_vtbl: IAMFilterMiscFlagsVtbl = IAMFilterMiscFlagsVtbl {
    QueryInterface: Some(filter_misc_flags_QueryInterface),
    AddRef: Some(filter_misc_flags_AddRef),
    Release: Some(filter_misc_flags_Release),
    GetMiscFlags: Some(filter_misc_flags_GetMiscFlags),
};

/// evr.c:670 `static const IMFGetServiceVtbl filter_get_service_vtbl`  (slot order: rust:types_amd64.rs)
pub static filter_get_service_vtbl: IMFGetServiceVtbl = IMFGetServiceVtbl {
    QueryInterface: Some(filter_get_service_QueryInterface),
    AddRef: Some(filter_get_service_AddRef),
    Release: Some(filter_get_service_Release),
    GetService: Some(filter_get_service_GetService),
};

/// evr.c:718 `static const IMFVideoRendererVtbl filter_video_renderer_vtbl`  (slot order: rust:types_amd64.rs)
pub static filter_video_renderer_vtbl: IMFVideoRendererVtbl = IMFVideoRendererVtbl {
    QueryInterface: Some(filter_video_renderer_QueryInterface),
    AddRef: Some(filter_video_renderer_AddRef),
    Release: Some(filter_video_renderer_Release),
    InitializeRenderer: Some(filter_video_renderer_InitializeRenderer),
};

/// evr.c:756 `static const IMediaEventSinkVtbl filter_media_event_sink_vtbl`  (slot order: rust:types_amd64.rs)
pub static filter_media_event_sink_vtbl: IMediaEventSinkVtbl = IMediaEventSinkVtbl {
    QueryInterface: Some(filter_media_event_sink_QueryInterface),
    AddRef: Some(filter_media_event_sink_AddRef),
    Release: Some(filter_media_event_sink_Release),
    Notify: Some(filter_media_event_sink_Notify),
};

/// evr.c:830 `static const IMFTopologyServiceLookupVtbl filter_service_lookup_vtbl`  (slot order: rust:types_amd64.rs)
pub static filter_service_lookup_vtbl: IMFTopologyServiceLookupVtbl = IMFTopologyServiceLookupVtbl {
    QueryInterface: Some(filter_service_lookup_QueryInterface),
    AddRef: Some(filter_service_lookup_AddRef),
    Release: Some(filter_service_lookup_Release),
    LookupService: Some(filter_service_lookup_LookupService),
};
