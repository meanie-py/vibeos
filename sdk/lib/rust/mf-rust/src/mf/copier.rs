use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(mfplat): debug channel declaration, no Rust equivalent emitted.

pub const SAMPLE_COPIER_INPUT_TYPE_SET: i32 = 0x1;
pub const SAMPLE_COPIER_OUTPUT_TYPE_SET: i32 = 0x2;

#[repr(C)]
pub struct sample_copier {
    pub IMFTransform_iface: IMFTransform,
    pub refcount: LONG,

    pub attributes: *mut IMFAttributes,
    pub buffer_type: *mut IMFMediaType,
    pub buffer_size: DWORD,
    pub sample: *mut IMFSample,
    pub flags: DWORD,
    pub cs: CRITICAL_SECTION,
}
unsafe extern "C" fn impl_from_IMFTransform(iface: *mut IMFTransform) -> *mut sample_copier {
    return (iface as *mut u8).sub(core::mem::offset_of!(sample_copier, IMFTransform_iface)) as *mut sample_copier;
}

unsafe extern "system" fn sample_copier_transform_QueryInterface(iface: *mut IMFTransform, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, &raw const IID_IMFTransform) != 0 ||
            IsEqualIID!(riid, &raw const IID_IUnknown) != 0
    {
        *obj = iface as *mut core::ffi::c_void;
        IMFTransform_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn sample_copier_transform_AddRef(iface: *mut IMFTransform) -> ULONG {
    let transform: *mut sample_copier = impl_from_IMFTransform(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*transform).refcount)) as ULONG;

    return refcount;
}
unsafe extern "system" fn sample_copier_transform_Release(mut iface: *mut IMFTransform) -> ULONG {
    let mut transform: *mut sample_copier = impl_from_IMFTransform(iface);
    let mut refcount: ULONG = InterlockedDecrement(
        &raw mut (*transform).refcount as *mut ::core::ffi::c_long,
    ) as ULONG;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/copier.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"sample_copier_transform_Release\0" as *const u8 as *const ::core::ffi::c_char,
            85 as ::core::ffi::c_int,
            b"%p, refcount %lu.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            refcount,
        );
    }
    if refcount == 0 {
        if !(*transform).attributes.is_null() {
            IMFAttributes_Release((*transform).attributes);
        }
        if !(*transform).buffer_type.is_null() {
            IMFMediaType_Release((*transform).buffer_type);
        }
        DeleteCriticalSection(&raw mut (*transform).cs);
        free(transform as *mut ::core::ffi::c_void);
    }
    return refcount;
}

unsafe extern "system" fn sample_copier_transform_GetStreamLimits(
    iface: *mut IMFTransform,
    input_minimum: *mut DWORD,
    input_maximum: *mut DWORD,
    output_minimum: *mut DWORD,
    output_maximum: *mut DWORD,
) -> HRESULT {
    *output_maximum = 1;
    *output_minimum = *output_maximum;
    *input_maximum = *output_minimum;
    *input_minimum = *input_maximum;

    return S_OK;
}

unsafe extern "system" fn sample_copier_transform_GetStreamCount(
    mut iface: *mut IMFTransform,
    mut inputs: *mut DWORD,
    mut outputs: *mut DWORD,
) -> HRESULT {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/copier.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"sample_copier_transform_GetStreamCount\0" as *const u8 as *const ::core::ffi::c_char,
            112 as ::core::ffi::c_int,
            b"%p, %p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            inputs,
            outputs,
        );
    }
    *inputs = 1;
    *outputs = 1;
    return S_OK;
}
unsafe extern "system" fn sample_copier_transform_GetStreamIDs(
    iface: *mut IMFTransform,
    input_size: DWORD,
    inputs: *mut DWORD,
    output_size: DWORD,
    outputs: *mut DWORD,
) -> HRESULT {
    // TRACE("%p, %lu, %p, %lu, %p.\n", iface, input_size, inputs, output_size, outputs);
    E_NOTIMPL
}

unsafe extern "system" fn sample_copier_transform_GetInputStreamInfo(
    iface: *mut IMFTransform,
    id: DWORD,
    info: *mut MFT_INPUT_STREAM_INFO,
) -> HRESULT {
    let transform: *mut sample_copier = impl_from_IMFTransform(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/copier.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"sample_copier_transform_GetInputStreamInfo\0" as *const u8
                as *const ::core::ffi::c_char,
            132 as ::core::ffi::c_int,
            b"%p, %lu, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            id,
            info,
        );
    }
    memset(
        info as *mut ::core::ffi::c_void,
        0,
        ::core::mem::size_of::<MFT_INPUT_STREAM_INFO>() as size_t,
    );

    EnterCriticalSection(core::ptr::addr_of_mut!((*transform).cs));
    (*info).cbSize = (*transform).buffer_size;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*transform).cs));

    return S_OK;
}
unsafe extern "system" fn sample_copier_transform_GetOutputStreamInfo(iface: *mut IMFTransform, id: DWORD, info: *mut MFT_OUTPUT_STREAM_INFO) -> HRESULT {
    let mut transform: *mut sample_copier = impl_from_IMFTransform(iface);
    TRACE!("%p, %lu, %p.\n", iface, id, info);

    memset(info as *mut core::ffi::c_void, 0, core::mem::size_of::<MFT_OUTPUT_STREAM_INFO>() as usize);

    EnterCriticalSection(core::ptr::addr_of_mut!((*transform).cs));
    (*info).cbSize = (*transform).buffer_size;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*transform).cs));

    return S_OK;
}

unsafe extern "system" fn sample_copier_transform_GetAttributes(iface: *mut IMFTransform, attributes: *mut *mut IMFAttributes) -> HRESULT {
    let mut transform: *mut sample_copier = impl_from_IMFTransform(iface);
    *attributes = (*transform).attributes;
    IMFAttributes_AddRef(*attributes);
    return S_OK;
}
unsafe extern "system" fn sample_copier_transform_GetInputStreamAttributes(
    iface: *mut IMFTransform,
    id: DWORD,
    attributes: *mut *mut IMFAttributes,
) -> HRESULT {
    // TRACE("%p, %lu, %p.\n", iface, id, attributes);
    E_NOTIMPL
}
unsafe extern "system" fn sample_copier_transform_GetOutputStreamAttributes(
    iface: *mut IMFTransform,
    id: DWORD,
    attributes: *mut *mut IMFAttributes,
) -> HRESULT {
    // TRACE("%p, %lu, %p.\n", iface, id, attributes);
    E_NOTIMPL
}
unsafe extern "system" fn sample_copier_transform_DeleteInputStream(
    iface: *mut IMFTransform,
    id: DWORD,
) -> HRESULT {
    // TRACE("%p, %lu.\n", iface, id);
    E_NOTIMPL
}
unsafe extern "system" fn sample_copier_transform_AddInputStreams(
    iface: *mut IMFTransform,
    streams: DWORD,
    ids: *mut DWORD,
) -> HRESULT {
    // TRACE("%p, %lu, %p.\n", iface, streams, ids);
    E_NOTIMPL
}

unsafe extern "system" fn sample_copier_transform_GetInputAvailableType(
    iface: *mut IMFTransform,
    id: DWORD,
    index: DWORD,
    type_: *mut *mut IMFMediaType,
) -> HRESULT {
    static mut types: [*const GUID; 2] =
        unsafe { [&raw const MFMediaType_Video, &raw const MFMediaType_Audio] };
    let mut hr: HRESULT = 0;

    if id != 0 {
        return MF_E_INVALIDSTREAMNUMBER;
    }

    if index > (ARRAY_SIZE!(types) as DWORD).wrapping_sub(1) {
        return MF_E_NO_MORE_TYPES;
    }

    hr = MFCreateMediaType(type_);
    if hr >= 0 {
        hr = IMFMediaType_SetGUID(*type_, &raw const MF_MT_MAJOR_TYPE, types[index as usize]);
    }

    return hr;
}

unsafe extern "system" fn sample_copier_transform_GetOutputAvailableType(
    iface: *mut IMFTransform,
    id: DWORD,
    index: DWORD,
    type_: *mut *mut IMFMediaType,
) -> HRESULT {
    let transform: *mut sample_copier = impl_from_IMFTransform(iface);
    let mut cloned_type: *mut IMFMediaType = core::ptr::null_mut();
    let mut hr: HRESULT = S_OK;

    // TRACE("%p, %lu, %lu, %p.\n", iface, id, index, type); (debug channel gated, not emitted)
    let _ = index;

    EnterCriticalSection(core::ptr::addr_of_mut!((*transform).cs));
    if !(*transform).buffer_type.is_null() {
        hr = MFCreateMediaType(core::ptr::addr_of_mut!(cloned_type));
        if hr >= 0 {
            hr = IMFMediaType_CopyAllItems(
                (*transform).buffer_type,
                cloned_type as *mut IMFAttributes,
            );
        }
    } else if id != 0 {
        hr = MF_E_INVALIDSTREAMNUMBER;
    } else {
        hr = MF_E_NO_MORE_TYPES;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*transform).cs));

    if hr >= 0 {
        *type_ = cloned_type;
    } else if !cloned_type.is_null() {
        IMFMediaType_Release(cloned_type);
    }

    return hr;
}

unsafe extern "C" fn sample_copier_get_buffer_size(mut type_0: *mut IMFMediaType, mut size: *mut UINT32) -> HRESULT {
    let mut major: GUID = core::mem::zeroed::<GUID>();
    let mut subtype: GUID = core::mem::zeroed::<GUID>();
    let mut frame_size: UINT64 = 0;
    let mut hr: HRESULT = 0;

    *size = 0;

    hr = IMFMediaType_GetMajorType(type_0, &raw mut major);
    if hr < 0 {
        return hr;
    }

    if IsEqualGUID!(&raw const major, &raw const MFMediaType_Video) != false {
        hr = IMFMediaType_GetGUID(type_0, &raw const MF_MT_SUBTYPE, &raw mut subtype);
        if hr >= 0 {
            hr = IMFMediaType_GetUINT64(type_0, &raw const MF_MT_FRAME_SIZE, &raw mut frame_size);
            if hr >= 0 {
                hr = MFCalculateImageSize(
                    &raw mut subtype,
                    (frame_size >> 32) as UINT32,
                    frame_size as UINT32,
                    size,
                );
                if hr < 0 {
                    WARN!("Failed to get image size for video format %s.\n", debugstr_guid(&raw mut subtype));
                }
            }
        }
    } else if IsEqualGUID!(&raw const major, &raw const MFMediaType_Audio) != false {
        hr = E_NOTIMPL;
    }

    return hr;
}

unsafe extern "C" fn sample_copier_set_media_type(
    transform: *mut sample_copier,
    input: BOOL,
    id: DWORD,
    type_0: *mut IMFMediaType,
    flags: DWORD,
) -> HRESULT {
    let mut buffer_size: UINT32 = 0;
    let mut hr: HRESULT = S_OK;

    if id != 0 {
        return MF_E_INVALIDSTREAMNUMBER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*transform).cs));
    if !type_0.is_null() {
        hr = sample_copier_get_buffer_size(type_0, core::ptr::addr_of_mut!(buffer_size));
        if (flags & (MFT_SET_TYPE_TEST_ONLY as DWORD)) == 0 && hr >= 0 {
            if (*transform).buffer_type.is_null() {
                hr = MFCreateMediaType(core::ptr::addr_of_mut!((*transform).buffer_type));
            }
            if hr >= 0 {
                hr = IMFMediaType_CopyAllItems(
                    type_0,
                    (*transform).buffer_type as *mut IMFAttributes,
                );
            }
            if hr >= 0 {
                (*transform).buffer_size = buffer_size as DWORD;
            }

            if hr >= 0 {
                if input != 0 {
                    (*transform).flags |= SAMPLE_COPIER_INPUT_TYPE_SET as DWORD;
                    (*transform).flags &= !(SAMPLE_COPIER_OUTPUT_TYPE_SET as DWORD);
                } else {
                    (*transform).flags |= SAMPLE_COPIER_OUTPUT_TYPE_SET as DWORD;
                }
            }
        }
    } else if !(*transform).buffer_type.is_null() {
        IMFMediaType_Release((*transform).buffer_type);
        (*transform).buffer_type = core::ptr::null_mut::<IMFMediaType>();
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*transform).cs));

    return hr;
}

unsafe extern "system" fn sample_copier_transform_SetInputType(
    iface: *mut IMFTransform,
    id: DWORD,
    type_: *mut IMFMediaType,
    flags: DWORD,
) -> HRESULT {
    let transform: *mut sample_copier = impl_from_IMFTransform(iface);

    return sample_copier_set_media_type(transform, (TRUE) as BOOL, id, type_, flags);
}

unsafe extern "system" fn sample_copier_transform_SetOutputType(iface: *mut IMFTransform, id: DWORD, type_: *mut IMFMediaType, flags: DWORD) -> HRESULT {
    let transform: *mut sample_copier = impl_from_IMFTransform(iface);
    return sample_copier_set_media_type(transform, (FALSE) as BOOL, id, type_, flags);
}

unsafe extern "C" fn sample_copier_get_current_type(
    transform: *mut sample_copier,
    id: DWORD,
    flags: DWORD,
    ret: *mut *mut IMFMediaType,
) -> HRESULT {
    let mut cloned_type: *mut IMFMediaType = core::ptr::null_mut();
    let mut hr: HRESULT;

    if id != 0 {
        return MF_E_INVALIDSTREAMNUMBER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*transform).cs));
    if (*transform).flags & flags != 0 {
        if {
            hr = MFCreateMediaType(core::ptr::addr_of_mut!(cloned_type));
            hr
        } >= 0
        {
            hr = IMFMediaType_CopyAllItems(
                (*transform).buffer_type,
                cloned_type as *mut IMFAttributes,
            );
        }
    } else {
        hr = MF_E_TRANSFORM_TYPE_NOT_SET;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*transform).cs));

    if hr >= 0 {
        *ret = cloned_type;
    } else if !cloned_type.is_null() {
        IMFMediaType_Release(cloned_type);
    }

    return hr;
}

unsafe extern "system" fn sample_copier_transform_GetInputCurrentType(iface: *mut IMFTransform, id: DWORD, type_0: *mut *mut IMFMediaType) -> HRESULT {
    let transform: *mut sample_copier = impl_from_IMFTransform(iface);

    return sample_copier_get_current_type(transform, id, (SAMPLE_COPIER_INPUT_TYPE_SET) as DWORD, type_0);
}

unsafe extern "system" fn sample_copier_transform_GetOutputCurrentType(iface: *mut IMFTransform, id: DWORD, type_: *mut *mut IMFMediaType) -> HRESULT {
    let transform: *mut sample_copier = impl_from_IMFTransform(iface);

    return sample_copier_get_current_type(transform, id, SAMPLE_COPIER_OUTPUT_TYPE_SET as DWORD, type_);
}

unsafe extern "system" fn sample_copier_transform_GetInputStatus(iface: *mut IMFTransform, id: DWORD, flags: *mut DWORD) -> HRESULT {
    let transform: *mut sample_copier = impl_from_IMFTransform(iface);
    let mut hr: HRESULT = S_OK;

    if id != 0 {
        return MF_E_INVALIDSTREAMNUMBER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*transform).cs));
    if ((*transform).flags & (SAMPLE_COPIER_INPUT_TYPE_SET as u32)) == 0 {
        hr = MF_E_TRANSFORM_TYPE_NOT_SET;
    } else {
        *flags = if !(*transform).sample.is_null() {
            0
        } else {
            MFT_INPUT_STATUS_ACCEPT_DATA
        } as DWORD;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*transform).cs));

    return hr;
}

unsafe extern "system" fn sample_copier_transform_GetOutputStatus(iface: *mut IMFTransform, flags: *mut DWORD) -> HRESULT {
    let transform: *mut sample_copier = impl_from_IMFTransform(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*transform).cs));
    if ((*transform).flags & (SAMPLE_COPIER_OUTPUT_TYPE_SET as u32)) == 0 {
        hr = MF_E_TRANSFORM_TYPE_NOT_SET;
    } else {
        *flags = if !(*transform).sample.is_null() { MFT_OUTPUT_STATUS_SAMPLE_READY } else { 0 };
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*transform).cs));

    return hr;
}
unsafe extern "system" fn sample_copier_transform_SetOutputBounds(
    mut iface: *mut IMFTransform,
    mut lower: LONGLONG,
    mut upper: LONGLONG,
) -> HRESULT {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/copier.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"sample_copier_transform_SetOutputBounds\0" as *const u8 as *const ::core::ffi::c_char,
            427 as ::core::ffi::c_int,
            b"%p, %s, %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_time(lower),
            debugstr_time(upper),
        );
    }
    return E_NOTIMPL;
}
unsafe extern "system" fn sample_copier_transform_ProcessEvent(
    iface: *mut IMFTransform,
    id: DWORD,
    event: *mut IMFMediaEvent,
) -> HRESULT {
    // FIXME("%p, %lu, %p.\n", iface, id, event);
    E_NOTIMPL
}

unsafe extern "system" fn sample_copier_transform_ProcessMessage(iface: *mut IMFTransform, message: MFT_MESSAGE_TYPE, param: ULONG_PTR) -> HRESULT {
    let transform: *mut sample_copier = impl_from_IMFTransform(iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*transform).cs));

    if message == MFT_MESSAGE_COMMAND_FLUSH {
        if !(*transform).sample.is_null() {
            IMFSample_Release((*transform).sample);
            (*transform).sample = core::ptr::null_mut();
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*transform).cs));

    return S_OK;
}

unsafe extern "system" fn sample_copier_transform_ProcessInput(iface: *mut IMFTransform, id: DWORD, sample: *mut IMFSample, flags: DWORD) -> HRESULT {
    let transform: *mut sample_copier = impl_from_IMFTransform(iface);
    let mut hr: HRESULT = S_OK;

    if id != 0 {
        return MF_E_INVALIDSTREAMNUMBER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*transform).cs));
    if (*transform).buffer_type.is_null() {
        hr = MF_E_TRANSFORM_TYPE_NOT_SET;
    } else if !(*transform).sample.is_null() {
        hr = MF_E_NOTACCEPTING;
    } else {
        (*transform).sample = sample;
        IMFSample_AddRef((*transform).sample);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*transform).cs));

    return hr;
}

unsafe extern "system" fn sample_copier_transform_ProcessOutput(
    iface: *mut IMFTransform,
    flags: DWORD,
    count: DWORD,
    buffers: *mut MFT_OUTPUT_DATA_BUFFER,
    status: *mut DWORD,
) -> HRESULT {
    let transform: *mut sample_copier = impl_from_IMFTransform(iface);
    let mut buffer: *mut IMFMediaBuffer = core::ptr::null_mut();
    let mut sample_flags: DWORD = 0;
    let mut hr: HRESULT = S_OK;
    let mut time: LONGLONG = 0;

    TRACE!(
        "%p, %#lx, %lu, %p, %p.\n",
        iface,
        flags,
        count,
        buffers,
        status
    );

    if count != 1 {
        return E_INVALIDARG;
    }

    (*buffers).dwStatus = 0;
    *status = (*buffers).dwStatus;

    EnterCriticalSection(core::ptr::addr_of_mut!((*transform).cs));
    if !((*transform).flags & (SAMPLE_COPIER_OUTPUT_TYPE_SET as u32) != 0) {
        hr = MF_E_TRANSFORM_TYPE_NOT_SET;
    } else if (*transform).sample.is_null() {
        hr = MF_E_TRANSFORM_NEED_MORE_INPUT;
    } else {
        IMFSample_CopyAllItems(
            (*transform).sample,
            (*buffers).pSample as *mut IMFAttributes,
        );

        if IMFSample_GetSampleDuration((*transform).sample, core::ptr::addr_of_mut!(time)) >= 0 {
            IMFSample_SetSampleDuration((*buffers).pSample, time);
        }

        if IMFSample_GetSampleTime((*transform).sample, core::ptr::addr_of_mut!(time)) >= 0 {
            IMFSample_SetSampleTime((*buffers).pSample, time);
        }

        if IMFSample_GetSampleFlags((*transform).sample, core::ptr::addr_of_mut!(sample_flags)) >= 0 {
            IMFSample_SetSampleFlags((*buffers).pSample, sample_flags);
        }

        if IMFSample_ConvertToContiguousBuffer((*transform).sample, core::ptr::null_mut()) >= 0 {
            if IMFSample_GetBufferByIndex(
                (*buffers).pSample,
                0,
                core::ptr::addr_of_mut!(buffer),
            ) >= 0
            {
                if IMFSample_CopyToBuffer((*transform).sample, buffer) < 0 {
                    hr = MF_E_UNEXPECTED;
                }
                IMFMediaBuffer_Release(buffer);
            }
        }

        IMFSample_Release((*transform).sample);
        (*transform).sample = core::ptr::null_mut();
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*transform).cs));

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn mf_is_sample_copier_transform(transform: *mut IMFTransform) -> BOOL {
    return (((*transform).lpVtbl as *const IMFTransformVtbl) == core::ptr::addr_of!(sample_copier_transform_vtbl)) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateSampleCopierMFT(transform: *mut *mut IMFTransform) -> HRESULT {
    let mut object: *mut sample_copier;
    let mut hr: HRESULT;

    object = calloc(1, (core::mem::size_of::<sample_copier>()) as _) as *mut sample_copier;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).IMFTransform_iface.lpVtbl = &raw const sample_copier_transform_vtbl;
    (*object).refcount = 1;
    InitializeCriticalSection(core::ptr::addr_of_mut!((*object).cs));

    hr = MFCreateAttributes(core::ptr::addr_of_mut!((*object).attributes), 0);
    if hr < 0 {
        IMFTransform_Release(core::ptr::addr_of_mut!((*object).IMFTransform_iface));
        return hr;
    }

    IMFAttributes_SetUINT32((*object).attributes, &raw const MFT_SUPPORT_DYNAMIC_FORMAT_CHANGE, 1);

    *transform = core::ptr::addr_of_mut!((*object).IMFTransform_iface);

    return S_OK;
}

// COM vtable tables — generated from copier.c by vtbl_gen.py
/// copier.c:538 `static const IMFTransformVtbl sample_copier_transform_vtbl`  (slot order: rust:types_amd64.rs)
pub static sample_copier_transform_vtbl: IMFTransformVtbl = IMFTransformVtbl {
    QueryInterface: Some(sample_copier_transform_QueryInterface),
    AddRef: Some(sample_copier_transform_AddRef),
    Release: Some(sample_copier_transform_Release),
    GetStreamLimits: Some(sample_copier_transform_GetStreamLimits),
    GetStreamCount: Some(sample_copier_transform_GetStreamCount),
    GetStreamIDs: Some(sample_copier_transform_GetStreamIDs),
    GetInputStreamInfo: Some(sample_copier_transform_GetInputStreamInfo),
    GetOutputStreamInfo: Some(sample_copier_transform_GetOutputStreamInfo),
    GetAttributes: Some(sample_copier_transform_GetAttributes),
    GetInputStreamAttributes: Some(sample_copier_transform_GetInputStreamAttributes),
    GetOutputStreamAttributes: Some(sample_copier_transform_GetOutputStreamAttributes),
    DeleteInputStream: Some(sample_copier_transform_DeleteInputStream),
    AddInputStreams: Some(sample_copier_transform_AddInputStreams),
    GetInputAvailableType: Some(sample_copier_transform_GetInputAvailableType),
    GetOutputAvailableType: Some(sample_copier_transform_GetOutputAvailableType),
    SetInputType: Some(sample_copier_transform_SetInputType),
    SetOutputType: Some(sample_copier_transform_SetOutputType),
    GetInputCurrentType: Some(sample_copier_transform_GetInputCurrentType),
    GetOutputCurrentType: Some(sample_copier_transform_GetOutputCurrentType),
    GetInputStatus: Some(sample_copier_transform_GetInputStatus),
    GetOutputStatus: Some(sample_copier_transform_GetOutputStatus),
    SetOutputBounds: Some(sample_copier_transform_SetOutputBounds),
    ProcessEvent: Some(sample_copier_transform_ProcessEvent),
    ProcessMessage: Some(sample_copier_transform_ProcessMessage),
    ProcessInput: Some(sample_copier_transform_ProcessInput),
    ProcessOutput: Some(sample_copier_transform_ProcessOutput),
};
