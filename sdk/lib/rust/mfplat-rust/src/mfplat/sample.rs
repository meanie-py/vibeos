use crate::mfplat::buffer::MFCreate2DMediaBuffer;
use crate::mfplat::buffer::MFCreateDXGISurfaceBuffer;
use crate::mfplat::buffer::MFCreateDXSurfaceBuffer;
use crate::mfplat::buffer::MFCreateMemoryBuffer;
use crate::mfplat::mediatype::MFMapDX9FormatToDXGIFormat;
use crate::mfplat::buffer::buffer;
use crate::mfplat::main::dxgi_device_manager;
use crate::mfplat::mediatype::media_type;
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

#[no_mangle]
pub static mut IID_IDirectXVideoProcessorService: GUID = _GUID {
    Data1: 0xfc51a552 as ::core::ffi::c_uint,
    Data2: 0xd5e7 as ::core::ffi::c_ushort,
    Data3: 0x11d9 as ::core::ffi::c_ushort,
    Data4: [
        0xaf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x55 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x43 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xff as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

// #define COBJMACROS: no value, nothing to emit.
// WINE_DEFAULT_DEBUG_CHANNEL(mfplat): debug-channel registration, handled by the crate's DPRINT macros.

pub const SAMPLE_PROP_HAS_DURATION: i32 = 1 << 0;
pub const SAMPLE_PROP_HAS_TIMESTAMP: i32 = 1 << 1;

#[repr(C)]
pub struct sample {
    pub attributes: attributes,
    pub IMFSample_iface: IMFSample,
    pub IMFTrackedSample_iface: IMFTrackedSample,

    pub buffers: *mut *mut IMFMediaBuffer,
    pub buffer_count: usize,
    pub capacity: usize,
    pub flags: DWORD,
    pub prop_flags: DWORD,
    pub duration: LONGLONG,
    pub timestamp: LONGLONG,

    /* Tracked sample functionality. */
    pub tracked_result: *mut IRtwqAsyncResult,
    pub tracked_refcount: LONG,
}
pub type Psample = *mut sample;

#[repr(C)]
pub struct sample_allocator_frame_desc {
    pub width: u32,
    pub height: u32,
    pub d3d9_format: D3DFORMAT,
    pub dxgi_format: DXGI_FORMAT,
    pub usage: u32,
    pub bindflags: u32,
    pub miscflags: u32,
    pub buffer_count: u32,
}

#[repr(C)]
pub struct sample_allocator {
    pub IMFVideoSampleAllocatorEx_iface: IMFVideoSampleAllocatorEx,
    pub IMFVideoSampleAllocatorCallback_iface: IMFVideoSampleAllocatorCallback,
    pub tracking_callback: IMFAsyncCallback,
    pub refcount: LONG,

    pub callback: *mut IMFVideoSampleAllocatorNotify,
    pub d3d9_device_manager: *mut IDirect3DDeviceManager9,
    pub dxgi_device_manager: *mut IMFDXGIDeviceManager,

    pub frame_desc: sample_allocator_frame_desc,

    pub attributes: *mut IMFAttributes,
    pub media_type: *mut IMFMediaType,

    pub free_sample_count: u32,
    pub cold_sample_count: u32,
    pub free_samples: list,
    pub used_samples: list,
    pub cs: CRITICAL_SECTION,
}
pub type Psample_allocator = *mut sample_allocator;

#[repr(C)]
pub struct queued_sample {
    pub entry: list,
    pub sample: *mut IMFSample,
}
pub type Pqueued_sample = *mut queued_sample;

#[repr(C)]
pub struct surface_service {
    pub dxva_service: *mut IDirectXVideoProcessorService,
    pub d3d11_device: *mut ID3D11Device,
    pub hdevice: HANDLE,
}
pub type Psurface_service = *mut surface_service;


unsafe extern "C" fn impl_from_IMFVideoSampleAllocatorEx(iface: *mut IMFVideoSampleAllocatorEx) -> *mut sample_allocator {
    return CONTAINING_RECORD!(iface, sample_allocator, IMFVideoSampleAllocatorEx_iface);
}

unsafe extern "C" fn impl_from_IMFVideoSampleAllocatorCallback(
    iface: *mut IMFVideoSampleAllocatorCallback,
) -> *mut sample_allocator {
    return CONTAINING_RECORD!(iface, sample_allocator, IMFVideoSampleAllocatorCallback_iface);
}

unsafe extern "C" fn impl_from_IMFAsyncCallback(iface: *mut IMFAsyncCallback) -> *mut sample_allocator {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(sample_allocator, tracking_callback)) as *mut sample_allocator;
}

unsafe extern "C" fn impl_from_IMFSample(iface: *mut IMFSample) -> *mut sample {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(sample, IMFSample_iface)) as *mut sample;
}

unsafe extern "C" fn impl_from_IMFTrackedSample(iface: *mut IMFTrackedSample) -> *mut sample {
    return (iface as *mut u8).sub(core::mem::offset_of!(sample, IMFTrackedSample_iface)) as *mut sample;
}

unsafe extern "system" fn sample_QueryInterface(iface: *mut IMFSample, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFSample)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAttributes)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *out = core::ptr::addr_of_mut!((*sample).IMFSample_iface) as *mut core::ffi::c_void;
    }
    else if !(*sample).IMFTrackedSample_iface.lpVtbl.is_null() && IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFTrackedSample)) != 0
    {
        *out = core::ptr::addr_of_mut!((*sample).IMFTrackedSample_iface) as *mut core::ffi::c_void;
    }
    else
    {
        WARN!("Unsupported %s.\n", debugstr_guid(riid));
        *out = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*out as *mut IUnknown);
    return S_OK;
}

unsafe extern "system" fn sample_AddRef(iface: *mut IMFSample) -> ULONG {
    let sample: *mut sample = impl_from_IMFSample(iface);
    let refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*sample).attributes.ref_) as *mut LONG,
    ) as ULONG;

    if ((*__wine_dbch___default).flags as c_int & ((1 as c_int) << (__WINE_DBCL_TRACE as c_int))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/sample.c\0".as_ptr() as *const c_char,
            b"sample_AddRef\0".as_ptr() as *const c_char,
            153 as c_int,
            b"%p, refcount %lu.\n\0".as_ptr() as *const c_char,
            iface,
            refcount,
        );
    }

    return refcount;
}

unsafe extern "C" fn release_sample_object(sample: *mut sample) {
    let mut i: size_t = 0;
    while i < (*sample).buffer_count {
        IMFMediaBuffer_Release(*(*sample).buffers.add(i as usize));
        i = i.wrapping_add(1);
    }
    clear_attributes_object(core::ptr::addr_of_mut!((*sample).attributes));
    free((*sample).buffers as *mut core::ffi::c_void);
    free(sample as *mut core::ffi::c_void);
}

unsafe extern "system" fn sample_Release(iface: *mut IMFSample) -> ULONG {
    let sample: *mut sample = impl_from_IMFSample(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*sample).attributes.ref_) as *mut core::ffi::c_long,
    ) as ULONG;

    TRACE!("%p, refcount %lu.\n", iface, refcount);

    if refcount == 0 {
        release_sample_object(sample);
    }

    return refcount;
}
unsafe extern "system" fn sample_tracked_Release(iface: *mut IMFSample) -> ULONG {
    let sample: *mut sample = impl_from_IMFSample(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*sample).attributes.ref_) as *mut LONG,
    ) as ULONG;
    let mut tracked_result: *mut IRtwqAsyncResult = core::ptr::null_mut();
    let mut hr: HRESULT;

    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));
    if !(*sample).tracked_result.is_null() && (*sample).tracked_refcount as ULONG == refcount {
        tracked_result = (*sample).tracked_result;
        (*sample).tracked_result = core::ptr::null_mut();
        (*sample).tracked_refcount = 0;

        /* Call could fail if queue system is not initialized, it's not critical. */
        hr = RtwqInvokeCallback(tracked_result);
        if hr < 0 {
            WARN!("Failed to invoke tracking callback, hr %#lx.\n", hr);
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));

    if !tracked_result.is_null() {
        IRtwqAsyncResult_Release(tracked_result);
    }

    TRACE!("%p, refcount %lu.\n", iface, refcount);

    if refcount == 0 {
        release_sample_object(sample);
    }

    return refcount;
}

unsafe extern "system" fn sample_GetItem(iface: *mut IMFSample, key: REFGUID, value: *mut PROPVARIANT) -> HRESULT {
    let mut sample: *mut sample = impl_from_IMFSample(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/sample.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"sample_GetItem\0" as *const u8 as *const core::ffi::c_char,
            217i32,
            b"%p, %s, %p.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_GetItem(core::ptr::addr_of_mut!((*sample).attributes), key, value);
}
unsafe extern "system" fn sample_GetItemType(
    mut iface: *mut IMFSample,
    key: *const GUID,
    mut type_0: *mut MF_ATTRIBUTE_TYPE,
) -> HRESULT {
    let mut sample: *mut sample = impl_from_IMFSample(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/sample.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"sample_GetItemType\0" as *const u8 as *const ::core::ffi::c_char,
            226 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            type_0,
        );
    }
    return attributes_GetItemType(&raw mut (*sample).attributes, key, type_0);
}

unsafe extern "system" fn sample_CompareItem(iface: *mut IMFSample, key: REFGUID, value: REFPROPVARIANT, result: *mut BOOL) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);

    return attributes_CompareItem(core::ptr::addr_of_mut!((*sample).attributes), key, value, result);
}

unsafe extern "system" fn sample_Compare(iface: *mut IMFSample, theirs: *mut IMFAttributes, type_: MF_ATTRIBUTES_MATCH_TYPE, result: *mut BOOL) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);

    return attributes_Compare(core::ptr::addr_of_mut!((*sample).attributes), theirs, type_, result);
}

unsafe extern "system" fn sample_GetUINT32(iface: *mut IMFSample, key: REFGUID, value: *mut UINT32) -> HRESULT {
    let mut sample: *mut sample = impl_from_IMFSample(iface);

    return attributes_GetUINT32(core::ptr::addr_of_mut!((*sample).attributes), key, value);
}

unsafe extern "system" fn sample_GetUINT64(iface: *mut IMFSample, key: REFGUID, value: *mut UINT64) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);

    return attributes_GetUINT64(core::ptr::addr_of_mut!((*sample).attributes), key, value);
}

unsafe extern "system" fn sample_GetDouble(iface: *mut IMFSample, key: *const GUID, value: *mut f64) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);
    TRACE!("%p, %s, %p.\n", iface, debugstr_attr(key), value);
    return attributes_GetDouble(core::ptr::addr_of_mut!((*sample).attributes), key, value);
}
unsafe extern "system" fn sample_GetGUID(
    mut iface: *mut IMFSample,
    key: *const GUID,
    mut value: *mut GUID,
) -> HRESULT {
    let mut sample: *mut sample = impl_from_IMFSample(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/sample.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"sample_GetGUID\0" as *const u8 as *const ::core::ffi::c_char,
            281 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
            value,
        );
    }
    return attributes_GetGUID(&raw mut (*sample).attributes, key, value);
}

unsafe extern "system" fn sample_GetStringLength(iface: *mut IMFSample, key: *const GUID, length: *mut UINT32) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);

    return attributes_GetStringLength(core::ptr::addr_of_mut!((*sample).attributes), key, length);
}

unsafe extern "system" fn sample_GetString(iface: *mut IMFSample, key: *const GUID, value: *mut WCHAR, size: UINT32, length: *mut UINT32) -> HRESULT {
    let mut sample: *mut sample = impl_from_IMFSample(iface);
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/sample.c\0".as_ptr() as *const i8,
            b"sample_GetString\0".as_ptr() as *const i8,
            299 as i32,
            b"%p, %s, %p, %u, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            value,
            size,
            length,
        );
    }
    return attributes_GetString(core::ptr::addr_of_mut!((*sample).attributes), key, value, size, length);
}

unsafe extern "system" fn sample_GetAllocatedString(iface: *mut IMFSample, key: *const GUID, value: *mut *mut WCHAR, length: *mut UINT32) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);
    TRACE!("%p, %s, %p, %p.\n", iface, debugstr_attr(key), value, length);
    return attributes_GetAllocatedString(core::ptr::addr_of_mut!((*sample).attributes), key, value, length);
}
unsafe extern "system" fn sample_GetBlobSize(iface: *mut IMFSample, key: *const GUID, size: *mut UINT32) -> HRESULT {
    let mut sample: *mut sample = impl_from_IMFSample(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/sample.c\0".as_ptr() as *const i8,
            b"sample_GetBlobSize\0".as_ptr() as *const i8,
            317i32,
            b"%p, %s, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            size,
        );
    }
    return attributes_GetBlobSize(core::ptr::addr_of_mut!((*sample).attributes), key, size);
}

unsafe extern "system" fn sample_GetBlob(iface: *mut IMFSample, key: *const GUID, buf: *mut UINT8, bufsize: UINT32, blobsize: *mut UINT32) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);
    TRACE!("%p, %s, %p, %u, %p.\n", iface, debugstr_attr(key), buf, bufsize, blobsize);
    return attributes_GetBlob(core::ptr::addr_of_mut!((*sample).attributes), key, buf, bufsize, blobsize);
}

unsafe extern "system" fn sample_GetAllocatedBlob(iface: *mut IMFSample, key: REFGUID, buf: *mut *mut UINT8, size: *mut UINT32) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);

    return attributes_GetAllocatedBlob(core::ptr::addr_of_mut!((*sample).attributes), key, buf, size);
}
unsafe extern "system" fn sample_GetUnknown(iface: *mut IMFSample, key: *const GUID, riid: *const GUID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);

    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/sample.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"sample_GetUnknown\0" as *const u8 as *const core::ffi::c_char,
            344 as core::ffi::c_int,
            b"%p, %s, %s, %p.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            debugstr_attr(key),
            debugstr_guid(riid),
            out,
        );
    }

    return attributes_GetUnknown(core::ptr::addr_of_mut!((*sample).attributes), key, riid, out);
}
unsafe extern "system" fn sample_SetItem(iface: *mut IMFSample, key: *const GUID, value: *const PROPVARIANT) -> HRESULT {
    let mut sample: *mut sample = impl_from_IMFSample(iface);

    return attributes_SetItem(core::ptr::addr_of_mut!((*sample).attributes), key, value);
}
unsafe extern "system" fn sample_DeleteItem(mut iface: *mut IMFSample, key: *const GUID) -> HRESULT {
    let mut sample: *mut sample = impl_from_IMFSample(iface);
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/sample.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"sample_DeleteItem\0" as *const u8 as *const ::core::ffi::c_char,
            362 as ::core::ffi::c_int,
            b"%p, %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_attr(key),
        );
    }
    return attributes_DeleteItem(&raw mut (*sample).attributes, key);
}

unsafe extern "system" fn sample_DeleteAllItems(iface: *mut IMFSample) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);

    return attributes_DeleteAllItems(core::ptr::addr_of_mut!((*sample).attributes));
}

unsafe extern "system" fn sample_SetUINT32(iface: *mut IMFSample, key: *const GUID, value: UINT32) -> HRESULT {
    let mut sample: *mut sample = impl_from_IMFSample(iface);
    TRACE!("%p, %s, %u.\n", iface, debugstr_attr(key), value);
    return attributes_SetUINT32(core::ptr::addr_of_mut!((*sample).attributes), key, value);
}

unsafe extern "system" fn sample_SetUINT64(
    iface: *mut IMFSample,
    key: *const GUID,
    value: UINT64,
) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);
    if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/sample.c\0".as_ptr() as *const i8,
            b"sample_SetUINT64\0".as_ptr() as *const i8,
            389,
            b"%p, %s, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            wine_dbgstr_longlong(value as ULONGLONG),
        );
    }
    return attributes_SetUINT64(core::ptr::addr_of_mut!((*sample).attributes), key, value);
}

unsafe extern "system" fn sample_SetDouble(iface: *mut IMFSample, key: *const GUID, value: f64) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);

    return attributes_SetDouble(core::ptr::addr_of_mut!((*sample).attributes), key, value);
}

unsafe extern "system" fn sample_SetGUID(iface: *mut IMFSample, key: *const GUID, value: *const GUID) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/sample.c\0" as *const u8 as *const i8,
            b"sample_SetGUID\0" as *const u8 as *const i8,
            407 as i32,
            b"%p, %s, %s.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_attr(key),
            debugstr_mf_guid(value),
        );
    }
    return attributes_SetGUID(core::ptr::addr_of_mut!((*sample).attributes), key, value);
}

unsafe extern "system" fn sample_SetString(iface: *mut IMFSample, key: *const GUID, value: *const WCHAR) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/sample.c\0".as_ptr() as *const i8,
            b"sample_SetString\0".as_ptr() as *const i8,
            416i32,
            b"%p, %s, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_attr(key),
            debugstr_w(value),
        );
    }
    return attributes_SetString(core::ptr::addr_of_mut!((*sample).attributes), key, value);
}

unsafe extern "system" fn sample_SetBlob(iface: *mut IMFSample, key: *const GUID, buf: *const UINT8, size: UINT32) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);

    return attributes_SetBlob(core::ptr::addr_of_mut!((*sample).attributes), key, buf, size);
}

unsafe extern "system" fn sample_SetUnknown(iface: *mut IMFSample, key: *const GUID, unknown: *mut IUnknown) -> HRESULT {
    let mut iface = iface;
    let mut key = key;
    let mut unknown = unknown;
    let sample: *mut sample = impl_from_IMFSample(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/sample.c\0".as_ptr() as *const core::ffi::c_char,
            b"sample_SetUnknown\0".as_ptr() as *const core::ffi::c_char,
            434i32,
            b"%p, %s, %p.\n\0".as_ptr() as *const core::ffi::c_char,
            iface,
            debugstr_attr(key),
            unknown,
        );
    }
    return attributes_SetUnknown(core::ptr::addr_of_mut!((*sample).attributes), key, unknown);
}

unsafe extern "system" fn sample_LockStore(iface: *mut IMFSample) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);
    TRACE!("%p.\n", iface);
    return attributes_LockStore(core::ptr::addr_of_mut!((*sample).attributes));
}

unsafe extern "system" fn sample_UnlockStore(iface: *mut IMFSample) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);

    return attributes_UnlockStore(core::ptr::addr_of_mut!((*sample).attributes));
}

unsafe extern "system" fn sample_GetCount(iface: *mut IMFSample, count: *mut UINT32) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);

    return attributes_GetCount(core::ptr::addr_of_mut!((*sample).attributes), count);
}

unsafe extern "system" fn sample_GetItemByIndex(iface: *mut IMFSample, index: UINT32, key: *mut GUID, value: *mut PROPVARIANT) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);

    return attributes_GetItemByIndex(core::ptr::addr_of_mut!((*sample).attributes), index, key, value);
}

unsafe extern "system" fn sample_CopyAllItems(iface: *mut IMFSample, dest: *mut IMFAttributes) -> HRESULT {
    let mut sample: *mut sample = impl_from_IMFSample(iface);

    return attributes_CopyAllItems(core::ptr::addr_of_mut!((*sample).attributes), dest);
}

unsafe extern "system" fn sample_GetSampleFlags(iface: *mut IMFSample, flags: *mut DWORD) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));
    *flags = (*sample).flags;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));

    return S_OK;
}

unsafe extern "system" fn sample_SetSampleFlags(iface: *mut IMFSample, flags: DWORD) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/sample.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"sample_SetSampleFlags\0" as *const u8 as *const ::core::ffi::c_char,
            501 as ::core::ffi::c_int,
            b"%p, %#lx.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            flags,
        );
    }
    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));
    (*sample).flags = flags;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));

    return S_OK;
}

unsafe extern "system" fn sample_GetSampleTime(iface: *mut IMFSample, timestamp: *mut LONGLONG) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!("%p, %p.\n", iface, timestamp);

    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));
    if (*sample).prop_flags & (SAMPLE_PROP_HAS_TIMESTAMP as u32) != 0 {
        *timestamp = (*sample).timestamp;
    } else {
        hr = MF_E_NO_SAMPLE_TIMESTAMP;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));

    return hr;
}

unsafe extern "system" fn sample_SetSampleTime(iface: *mut IMFSample, timestamp: LONGLONG) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));
    (*sample).timestamp = timestamp;
    (*sample).prop_flags |= (SAMPLE_PROP_HAS_TIMESTAMP) as u32;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));

    return S_OK;
}

unsafe extern "system" fn sample_GetSampleDuration(iface: *mut IMFSample, duration: *mut LONGLONG) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));
    if (*sample).prop_flags & (SAMPLE_PROP_HAS_DURATION as u32) != 0 {
        *duration = (*sample).duration;
    } else {
        hr = MF_E_NO_SAMPLE_DURATION;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));

    return hr;
}

unsafe extern "system" fn sample_SetSampleDuration(iface: *mut IMFSample, duration: LONGLONG) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));
    (*sample).duration = duration;
    (*sample).prop_flags |= (SAMPLE_PROP_HAS_DURATION) as u32;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));

    return S_OK;
}
unsafe extern "system" fn sample_GetBufferCount(iface: *mut IMFSample, count: *mut DWORD) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);

    if count.is_null() {
        return E_INVALIDARG;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));
    *count = (*sample).buffer_count as DWORD;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));

    return S_OK;
}

unsafe extern "system" fn sample_GetBufferByIndex(iface: *mut IMFSample, index: DWORD, buffer: *mut *mut IMFMediaBuffer) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!("%p, %lu, %p.\n", iface, index, buffer);

    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));
    if (index as usize) < (*sample).buffer_count {
        *buffer = *(*sample).buffers.offset(index as isize);
        IMFMediaBuffer_AddRef(*buffer);
    } else {
        hr = E_INVALIDARG;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));

    return hr;
}

unsafe extern "C" fn sample_get_total_length(sample: *mut sample) -> ::core::ffi::c_uint {
    let mut total_length: DWORD = 0;
    let mut length: DWORD;
    let mut i: size_t;
    i = 0;
    while i < (*sample).buffer_count {
        length = 0;
        if (IMFMediaBuffer_GetCurrentLength(*(*sample).buffers.offset(i as isize), core::ptr::addr_of_mut!(length))) >= 0 {
            total_length = total_length.wrapping_add(length);
        }
        i = i.wrapping_add(1);
    }

    return total_length as ::core::ffi::c_uint;
}

unsafe extern "C" fn sample_copy_to_buffer(sample: *mut sample, buffer: *mut IMFMediaBuffer) -> HRESULT {
    let mut total_length: DWORD;
    let mut dst_length: DWORD;
    let mut dst_current_length: DWORD;
    let mut src_max_length: DWORD;
    let mut current_length: DWORD;
    let mut src_ptr: *mut BYTE;
    let mut dst_ptr: *mut BYTE;
    let mut locked: BOOL;
    let mut hr: HRESULT;
    let mut i: usize;

    total_length = sample_get_total_length(sample) as DWORD;
    dst_current_length = (0) as DWORD;

    dst_ptr = core::ptr::null_mut();
    current_length = (0) as DWORD;
    dst_length = current_length;
    hr = IMFMediaBuffer_Lock(
        buffer,
        core::ptr::addr_of_mut!(dst_ptr),
        core::ptr::addr_of_mut!(dst_length),
        core::ptr::addr_of_mut!(current_length),
    );
    locked = (hr >= (0) as HRESULT) as BOOL;
    if locked != 0 {
        if dst_length < total_length {
            hr = (MF_E_BUFFERTOOSMALL) as HRESULT;
        } else if !dst_ptr.is_null() {
            i = (0) as usize;
            while i < (*sample).buffer_count && (hr >= (0) as HRESULT) {
                src_ptr = core::ptr::null_mut();
                current_length = (0) as DWORD;
                src_max_length = current_length;
                hr = IMFMediaBuffer_Lock(
                    *(*sample).buffers.add(i),
                    core::ptr::addr_of_mut!(src_ptr),
                    core::ptr::addr_of_mut!(src_max_length),
                    core::ptr::addr_of_mut!(current_length),
                );
                if hr >= (0) as HRESULT {
                    if !src_ptr.is_null() {
                        if current_length > dst_length {
                            hr = (MF_E_BUFFERTOOSMALL) as HRESULT;
                        } else if current_length != 0 {
                            memcpy(
                                dst_ptr as *mut core::ffi::c_void,
                                src_ptr as *const core::ffi::c_void,
                                current_length as usize,
                            );
                            dst_length = dst_length.wrapping_sub(current_length);
                            dst_current_length = dst_current_length.wrapping_add(current_length);
                            dst_ptr = dst_ptr.add(current_length as usize);
                        }
                    }
                    IMFMediaBuffer_Unlock(*(*sample).buffers.add(i));
                }
                i = i.wrapping_add(1);
            }
        }
    }

    if IMFMediaBuffer_SetCurrentLength(buffer, dst_current_length) < (0) as HRESULT {
        WARN!("Failed to set buffer length.\n");
    }

    if locked != 0 {
        IMFMediaBuffer_Unlock(buffer);
    }

    return hr;
}
unsafe extern "system" fn sample_ConvertToContiguousBuffer(iface: *mut IMFSample, buffer: *mut *mut IMFMediaBuffer) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);
    let mut total_length: core::ffi::c_uint = 0;
    let mut i: core::ffi::c_uint = 0;
    let mut dest_buffer: *mut IMFMediaBuffer = core::ptr::null_mut();
    let mut hr: HRESULT = S_OK;

    TRACE!("%p, %p.\n", iface, buffer);

    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));

    if (*sample).buffer_count == 0 {
        hr = E_UNEXPECTED;
    } else if (*sample).buffer_count > 1 {
        total_length = sample_get_total_length(sample);
        hr = MFCreateMemoryBuffer(total_length as DWORD, core::ptr::addr_of_mut!(dest_buffer));
        if hr >= 0 {
            hr = sample_copy_to_buffer(sample, dest_buffer);
            if hr >= 0 {
                i = 0;
                while (i as size_t) < (*sample).buffer_count {
                    IMFMediaBuffer_Release(*(*sample).buffers.offset(i as isize));
                    i = i.wrapping_add(1);
                }

                *(*sample).buffers.offset(0 as isize) = dest_buffer;
                IMFMediaBuffer_AddRef(*(*sample).buffers.offset(0 as isize));

                (*sample).buffer_count = 1;
            }
            IMFMediaBuffer_Release(dest_buffer);
        }
    }

    if (hr >= 0) && !buffer.is_null() {
        *buffer = *(*sample).buffers.offset(0 as isize);
        IMFMediaBuffer_AddRef(*buffer);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));

    return hr;
}
unsafe extern "system" fn sample_AddBuffer(iface: *mut IMFSample, buffer: *mut IMFMediaBuffer) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);
    let mut hr: HRESULT = S_OK;
    TRACE!("%p, %p.\n", iface, buffer);

    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));
    if mf_array_reserve(
        core::ptr::addr_of_mut!((*sample).buffers) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*sample).capacity),
        (*sample).buffer_count.wrapping_add(1),
        core::mem::size_of::<IMFMediaBuffer>() as size_t,
    ) == 0
    {
        hr = E_OUTOFMEMORY;
    } else {
        let fresh15 = (*sample).buffer_count;
        (*sample).buffer_count = (*sample).buffer_count.wrapping_add(1);
        *(*sample).buffers.add(fresh15) = buffer;
        IMFMediaBuffer_AddRef(buffer);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));

    return hr;
}

unsafe extern "system" fn sample_RemoveBufferByIndex(iface: *mut IMFSample, index: DWORD) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));
    if (index as usize) < (*sample).buffer_count {
        IMFMediaBuffer_Release(*(*sample).buffers.offset(index as isize));
        if (index as usize) < (*sample).buffer_count.wrapping_sub(1) {
            memmove(
                (*sample).buffers.offset(index as isize) as *mut core::ffi::c_void,
                (*sample).buffers.offset(index.wrapping_add(1) as isize)
                    as *const core::ffi::c_void,
                (*sample)
                    .buffer_count
                    .wrapping_sub(index as usize)
                    .wrapping_sub(1)
                    .wrapping_mul(core::mem::size_of::<IMFMediaBuffer>()),
            );
        }
        (*sample).buffer_count = (*sample).buffer_count.wrapping_sub(1);
    } else {
        hr = E_INVALIDARG;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));

    return hr;
}
unsafe extern "system" fn sample_RemoveAllBuffers(iface: *mut IMFSample) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);
    let mut i: size_t;

    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));
    i = 0;
    while i < (*sample).buffer_count {
        IMFMediaBuffer_Release(*(*sample).buffers.add(i));
        i += 1;
    }
    (*sample).buffer_count = 0;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));

    return S_OK;
}

unsafe extern "system" fn sample_GetTotalLength(iface: *mut IMFSample, total_length: *mut DWORD) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));
    *total_length = sample_get_total_length(sample);
    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));

    return S_OK;
}
unsafe extern "system" fn sample_CopyToBuffer(iface: *mut IMFSample, buffer: *mut IMFMediaBuffer) -> HRESULT {
    let sample: *mut sample = impl_from_IMFSample(iface);
    let mut total_length: DWORD = 0;
    let mut dst_length: DWORD = 0;
    let mut dst_current_length: DWORD = 0;
    let mut src_max_length: DWORD = 0;
    let mut current_length: DWORD = 0;
    let mut src_ptr: *mut BYTE = core::ptr::null_mut();
    let mut dst_ptr: *mut BYTE = core::ptr::null_mut();
    let mut buffer2d: *mut IMF2DBuffer = core::ptr::null_mut();
    let mut locked: BOOL = (FALSE as i32);
    let mut hr: HRESULT = E_FAIL;
    let mut i: size_t = 0;

    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));

    total_length = sample_get_total_length(sample);
    dst_current_length = 0;

    'done: {
        if (*sample).buffer_count == 1
            && ((IMFMediaBuffer_QueryInterface(
                buffer,
                core::ptr::addr_of!(IID_IMF2DBuffer),
                core::ptr::addr_of_mut!(buffer2d) as *mut *mut core::ffi::c_void,
            )) >= 0)
        {
            if ((IMFMediaBuffer_GetCurrentLength(
                *(*sample).buffers.add(0 as usize),
                core::ptr::addr_of_mut!(current_length),
            )) >= 0)
                && ((IMF2DBuffer_GetContiguousLength(buffer2d, core::ptr::addr_of_mut!(dst_length)))
                    >= 0)
                && current_length == dst_length
                && ((IMFMediaBuffer_Lock(
                    *(*sample).buffers.add(0 as usize),
                    core::ptr::addr_of_mut!(src_ptr),
                    core::ptr::addr_of_mut!(src_max_length),
                    core::ptr::addr_of_mut!(current_length),
                )) >= 0)
            {
                hr = IMF2DBuffer_ContiguousCopyFrom(buffer2d, src_ptr, current_length);
                IMFMediaBuffer_Unlock(*(*sample).buffers.add(0 as usize));
            }
            IMF2DBuffer_Release(buffer2d);
            if (hr) >= 0 {
                dst_current_length = current_length;
                break 'done;
            }
        }

        dst_ptr = core::ptr::null_mut();
        current_length = 0;
        dst_length = current_length;
        hr = IMFMediaBuffer_Lock(
            buffer,
            core::ptr::addr_of_mut!(dst_ptr),
            core::ptr::addr_of_mut!(dst_length),
            core::ptr::addr_of_mut!(current_length),
        );
        locked = (hr >= 0) as BOOL;
        if locked == 0 {
            break 'done;
        }

        if dst_length < total_length {
            hr = MF_E_BUFFERTOOSMALL;
            break 'done;
        }

        if dst_ptr.is_null() {
            break 'done;
        }

        i = 0 as size_t;
        while i < (*sample).buffer_count && (hr >= 0) {
            'body: {
                src_ptr = core::ptr::null_mut();
                current_length = 0;
                src_max_length = current_length;

                hr = IMFMediaBuffer_Lock(
                    *(*sample).buffers.add(i as usize),
                    core::ptr::addr_of_mut!(src_ptr),
                    core::ptr::addr_of_mut!(src_max_length),
                    core::ptr::addr_of_mut!(current_length),
                );
                if hr < 0 {
                    break 'body;
                }

                if !src_ptr.is_null() {
                    if current_length > dst_length {
                        hr = MF_E_BUFFERTOOSMALL;
                    } else if current_length != 0 {
                        memcpy(
                            dst_ptr as *mut core::ffi::c_void,
                            src_ptr as *const core::ffi::c_void,
                            current_length as size_t,
                        );
                        dst_length = dst_length.wrapping_sub(current_length);
                        dst_current_length = dst_current_length.wrapping_add(current_length);
                        dst_ptr = dst_ptr.offset(current_length as isize);
                    }
                }
                IMFMediaBuffer_Unlock(*(*sample).buffers.add(i as usize));
            }
            i = i.wrapping_add(1);
        }
    }

    IMFMediaBuffer_SetCurrentLength(buffer, dst_current_length);

    if locked != 0 {
        IMFMediaBuffer_Unlock(buffer);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));

    return hr;
}

unsafe extern "system" fn tracked_sample_QueryInterface(iface: *mut IMFTrackedSample, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let sample: *mut sample = impl_from_IMFTrackedSample(iface);
    return IMFSample_QueryInterface(core::ptr::addr_of_mut!((*sample).IMFSample_iface), riid, obj);
}

unsafe extern "system" fn tracked_sample_AddRef(mut iface: *mut IMFTrackedSample) -> ULONG {
    let mut sample: *mut sample = impl_from_IMFTrackedSample(iface);
    return IMFSample_AddRef(core::ptr::addr_of_mut!((*sample).IMFSample_iface));
}

unsafe extern "system" fn tracked_sample_Release(iface: *mut IMFTrackedSample) -> ULONG {
    let mut sample: *mut sample = impl_from_IMFTrackedSample(iface);
    return IMFSample_Release(core::ptr::addr_of_mut!((*sample).IMFSample_iface));
}

unsafe extern "system" fn tracked_sample_SetAllocator(
    iface: *mut IMFTrackedSample,
    sample_allocator: *mut IMFAsyncCallback,
    state: *mut IUnknown,
) -> HRESULT {
    let sample: *mut sample = impl_from_IMFTrackedSample(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!("%p, %p, %p.\n", iface, sample_allocator, state);

    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));

    if !(*sample).tracked_result.is_null() {
        hr = MF_E_NOTACCEPTING;
    } else {
        hr = RtwqCreateAsyncResult(
            iface as *mut IUnknown,
            sample_allocator as *mut IRtwqAsyncCallback,
            state,
            core::ptr::addr_of_mut!((*sample).tracked_result),
        );
        if hr >= 0 {
            /* Account for additional refcount brought by 'state' object. This threshold is used
               on Release() to invoke tracker callback.  */
            (*sample).tracked_refcount = 1;
            if state == core::ptr::addr_of_mut!((*sample).IMFTrackedSample_iface) as *mut IUnknown
                || state == core::ptr::addr_of_mut!((*sample).IMFSample_iface) as *mut IUnknown
            {
                (*sample).tracked_refcount += 1;
            }
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).attributes.cs));

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateSample(sample: *mut *mut IMFSample) -> HRESULT {
    let mut object: *mut sample;
    let mut hr: HRESULT;

    object = calloc(1, (core::mem::size_of::<sample>()) as _) as *mut sample;

    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = init_attributes_object(core::ptr::addr_of_mut!((*object).attributes), 0);
    if hr < 0 {
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    (*object).IMFSample_iface.lpVtbl = core::ptr::addr_of!(samplevtbl);

    *sample = core::ptr::addr_of_mut!((*object).IMFSample_iface);

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateTrackedSample(sample: *mut *mut IMFTrackedSample) -> HRESULT {
    let mut object: *mut sample;
    let mut hr: HRESULT;

    object = calloc(1, (core::mem::size_of::<sample>()) as _) as *mut sample;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = init_attributes_object(core::ptr::addr_of_mut!((*object).attributes), 0);
    if hr < 0 {
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    (*object).IMFSample_iface.lpVtbl = core::ptr::addr_of!(sample_tracked_vtbl);
    (*object).IMFTrackedSample_iface.lpVtbl = core::ptr::addr_of!(tracked_sample_vtbl);

    *sample = core::ptr::addr_of_mut!((*object).IMFTrackedSample_iface);

    return S_OK;
}

unsafe extern "system" fn sample_allocator_QueryInterface(iface: *mut IMFVideoSampleAllocatorEx, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocatorEx(iface);

    if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFVideoSampleAllocatorEx))) != 0)
        || ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFVideoSampleAllocator))) != 0)
        || ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown))) != 0)
    {
        *obj = core::ptr::addr_of_mut!((*allocator).IMFVideoSampleAllocatorEx_iface) as *mut core::ffi::c_void;
    } else if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFVideoSampleAllocatorCallback))) != 0) {
        *obj = core::ptr::addr_of_mut!((*allocator).IMFVideoSampleAllocatorCallback_iface) as *mut core::ffi::c_void;
    } else {
        WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
        *obj = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*obj as *mut IUnknown);
    return S_OK;
}
unsafe extern "system" fn sample_allocator_AddRef(iface: *mut IMFVideoSampleAllocatorEx) -> ULONG {
    let allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocatorEx(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*allocator).refcount) as _) as ULONG;

    return refcount;
}

unsafe extern "C" fn sample_allocator_release_samples(allocator: *mut sample_allocator) {
    let mut iter: *mut queued_sample = core::ptr::null_mut();
    let mut iter2: *mut queued_sample = core::ptr::null_mut();
    iter = ((*allocator).free_samples.next as *mut u8)
        .offset(-(core::mem::offset_of!(queued_sample, entry) as isize)) as *mut queued_sample;
    iter2 = ((*iter).entry.next as *mut u8)
        .offset(-(core::mem::offset_of!(queued_sample, entry) as isize)) as *mut queued_sample;
    while core::ptr::addr_of_mut!((*iter).entry) != core::ptr::addr_of_mut!((*allocator).free_samples) {
        list_remove(core::ptr::addr_of_mut!((*iter).entry));
        IMFSample_Release((*iter).sample);
        free(iter as *mut core::ffi::c_void);
        iter = iter2;
        iter2 = ((*iter).entry.next as *mut u8)
            .offset(-(core::mem::offset_of!(queued_sample, entry) as isize)) as *mut queued_sample;
    }

    iter = ((*allocator).used_samples.next as *mut u8)
        .offset(-(core::mem::offset_of!(queued_sample, entry) as isize)) as *mut queued_sample;
    iter2 = ((*iter).entry.next as *mut u8)
        .offset(-(core::mem::offset_of!(queued_sample, entry) as isize)) as *mut queued_sample;
    while core::ptr::addr_of_mut!((*iter).entry) != core::ptr::addr_of_mut!((*allocator).used_samples) {
        list_remove(core::ptr::addr_of_mut!((*iter).entry));
        free(iter as *mut core::ffi::c_void);
        iter = iter2;
        iter2 = ((*iter).entry.next as *mut u8)
            .offset(-(core::mem::offset_of!(queued_sample, entry) as isize)) as *mut queued_sample;
    }

    (*allocator).free_sample_count = 0;
    (*allocator).cold_sample_count = 0;
}

unsafe extern "C" fn sample_allocator_set_media_type(allocator: *mut sample_allocator, media_type: *mut IMFMediaType) {
    let mut frame_size: UINT64 = 0;
    let mut subtype: GUID = GUID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    };

    if media_type.is_null() {
        if !(*allocator).media_type.is_null() {
            IMFMediaType_Release((*allocator).media_type);
        }
        (*allocator).media_type = core::ptr::null_mut();
        return;
    }

    /* Check if type is the same. */
    IMFMediaType_GetUINT64(media_type, core::ptr::addr_of!(MF_MT_FRAME_SIZE), core::ptr::addr_of_mut!(frame_size));
    IMFMediaType_GetGUID(media_type, core::ptr::addr_of!(MF_MT_SUBTYPE), core::ptr::addr_of_mut!(subtype));

    if frame_size == (((*allocator).frame_desc.width as UINT64) << 32 | (*allocator).frame_desc.height as UINT64)
        && subtype.Data1 == (*allocator).frame_desc.d3d9_format as core::ffi::c_uint
    {
        return;
    }

    if !(*allocator).media_type.is_null() {
        IMFMediaType_Release((*allocator).media_type);
    }
    (*allocator).media_type = media_type;
    if !(*allocator).media_type.is_null() {
        IMFMediaType_AddRef((*allocator).media_type);
    }
}

unsafe extern "C" fn sample_allocator_set_attributes(
    allocator: *mut sample_allocator,
    attributes: *mut IMFAttributes,
) {
    if !(*allocator).attributes.is_null() {
        IMFAttributes_Release((*allocator).attributes);
    }
    (*allocator).attributes = attributes;
    if !(*allocator).attributes.is_null() {
        IMFAttributes_AddRef((*allocator).attributes);
    }
}

unsafe extern "system" fn sample_allocator_Release(iface: *mut IMFVideoSampleAllocatorEx) -> ULONG {
    let allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocatorEx(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*allocator).refcount) as *mut LONG) as ULONG;

    if refcount == 0 {
        if !(*allocator).callback.is_null() {
            IMFVideoSampleAllocatorNotify_Release((*allocator).callback);
        }
        if !(*allocator).d3d9_device_manager.is_null() {
            IDirect3DDeviceManager9_Release((*allocator).d3d9_device_manager);
        }
        if !(*allocator).dxgi_device_manager.is_null() {
            IMFDXGIDeviceManager_Release((*allocator).dxgi_device_manager);
        }
        sample_allocator_set_media_type(allocator, core::ptr::null_mut());
        sample_allocator_set_attributes(allocator, core::ptr::null_mut());
        sample_allocator_release_samples(allocator);
        DeleteCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));
        free(allocator as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn sample_allocator_SetDirectXManager(iface: *mut IMFVideoSampleAllocatorEx, manager: *mut IUnknown) -> HRESULT {
    let mut allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocatorEx(iface);
    let mut d3d9_device_manager: *mut IDirect3DDeviceManager9 = core::ptr::null_mut();
    let mut dxgi_device_manager: *mut IMFDXGIDeviceManager = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    if !manager.is_null() {
        hr = IUnknown_QueryInterface(
            manager,
            &raw const IID_IMFDXGIDeviceManager,
            &raw mut dxgi_device_manager as *mut *mut core::ffi::c_void,
        );
        if hr < 0 {
            hr = IUnknown_QueryInterface(
                manager,
                &raw const IID_IDirect3DDeviceManager9,
                &raw mut d3d9_device_manager as *mut *mut core::ffi::c_void,
            );
        }

        if hr < 0 {
            return hr;
        }
    }

    EnterCriticalSection(&raw mut (*allocator).cs);

    if !(*allocator).d3d9_device_manager.is_null() {
        IDirect3DDeviceManager9_Release((*allocator).d3d9_device_manager);
    }
    if !(*allocator).dxgi_device_manager.is_null() {
        IMFDXGIDeviceManager_Release((*allocator).dxgi_device_manager);
    }
    (*allocator).d3d9_device_manager = core::ptr::null_mut();
    (*allocator).dxgi_device_manager = core::ptr::null_mut();

    if !dxgi_device_manager.is_null() {
        (*allocator).dxgi_device_manager = dxgi_device_manager;
    } else if !d3d9_device_manager.is_null() {
        (*allocator).d3d9_device_manager = d3d9_device_manager;
    }

    LeaveCriticalSection(&raw mut (*allocator).cs);

    return S_OK;
}
unsafe extern "system" fn sample_allocator_UninitializeSampleAllocator(iface: *mut IMFVideoSampleAllocatorEx) -> HRESULT {
    let mut allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocatorEx(iface);

    TRACE!("%p.\n", iface);

    EnterCriticalSection(&raw mut (*allocator).cs);

    sample_allocator_release_samples(allocator);
    sample_allocator_set_media_type(allocator, core::ptr::null_mut::<IMFMediaType>());
    sample_allocator_set_attributes(allocator, core::ptr::null_mut::<IMFAttributes>());
    memset(
        &raw mut (*allocator).frame_desc as *mut core::ffi::c_void,
        0,
        core::mem::size_of_val(&(*allocator).frame_desc),
    );

    LeaveCriticalSection(&raw mut (*allocator).cs);

    return S_OK;
}

unsafe extern "C" fn sample_allocator_get_surface_service(mut allocator: *mut sample_allocator, mut service: *mut surface_service) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    memset(service as *mut ::core::ffi::c_void, 0, core::mem::size_of::<surface_service>());

    if !(*allocator).d3d9_device_manager.is_null() {
        if {
            hr = IDirect3DDeviceManager9_OpenDeviceHandle((*allocator).d3d9_device_manager, &raw mut (*service).hdevice);
            hr >= 0
        } {
            if {
                hr = IDirect3DDeviceManager9_GetVideoService(
                    (*allocator).d3d9_device_manager,
                    (*service).hdevice,
                    &raw const IID_IDirectXVideoProcessorService,
                    &raw mut (*service).dxva_service as *mut *mut ::core::ffi::c_void,
                );
                hr < 0
            } {
                WARN!("Failed to get DXVA processor service, hr %#lx.\n", hr);
                IDirect3DDeviceManager9_CloseDeviceHandle((*allocator).d3d9_device_manager, (*service).hdevice);
            }
        }
    } else if !(*allocator).dxgi_device_manager.is_null() {
        if {
            hr = IMFDXGIDeviceManager_OpenDeviceHandle((*allocator).dxgi_device_manager, &raw mut (*service).hdevice);
            hr >= 0
        } {
            if {
                hr = IMFDXGIDeviceManager_GetVideoService(
                    (*allocator).dxgi_device_manager,
                    (*service).hdevice,
                    &raw const IID_ID3D11Device,
                    &raw mut (*service).d3d11_device as *mut *mut ::core::ffi::c_void,
                );
                hr < 0
            } {
                WARN!("Failed to get D3D11 device, hr %#lx.\n", hr);
                IMFDXGIDeviceManager_CloseDeviceHandle((*allocator).dxgi_device_manager, (*service).hdevice);
            }
        }
    }

    if hr < 0 {
        memset(service as *mut ::core::ffi::c_void, 0, core::mem::size_of::<surface_service>());
    }

    return hr;
}

unsafe extern "C" fn sample_allocator_release_surface_service(
    mut allocator: *mut sample_allocator,
    mut service: *mut surface_service,
) {
    if !(*service).dxva_service.is_null() {
        IDirectXVideoProcessorService_Release((*service).dxva_service);
    }
    if !(*service).d3d11_device.is_null() {
        ID3D11Device_Release((*service).d3d11_device);
    }

    if !(*allocator).d3d9_device_manager.is_null() {
        IDirect3DDeviceManager9_CloseDeviceHandle((*allocator).d3d9_device_manager, (*service).hdevice);
    } else if !(*allocator).dxgi_device_manager.is_null() {
        IMFDXGIDeviceManager_CloseDeviceHandle((*allocator).dxgi_device_manager, (*service).hdevice);
    }
}

unsafe extern "C" fn sample_allocator_allocate_sample(
    mut allocator: *mut sample_allocator,
    mut service: *const surface_service,
    mut queued_sample: *mut *mut queued_sample,
) -> HRESULT {
    let mut tracked_sample: *mut IMFTrackedSample = core::ptr::null_mut();
    let mut buffer: *mut IMFMediaBuffer = core::ptr::null_mut();
    let mut sample: *mut IMFSample = core::ptr::null_mut();
    let mut i: core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;

    hr = MFCreateTrackedSample(core::ptr::addr_of_mut!(tracked_sample));
    if hr < 0 {
        return hr;
    }

    IMFTrackedSample_QueryInterface(
        tracked_sample,
        core::ptr::addr_of!(IID_IMFSample),
        core::ptr::addr_of_mut!(sample) as *mut *mut core::ffi::c_void,
    );
    IMFTrackedSample_Release(tracked_sample);

    i = 0 as core::ffi::c_uint;
    while i < (*allocator).frame_desc.buffer_count {
        if !(*service).dxva_service.is_null() {
            let mut surface: *mut IDirect3DSurface9 = core::ptr::null_mut();

            hr = IDirectXVideoProcessorService_CreateSurface(
                (*service).dxva_service,
                (*allocator).frame_desc.width as UINT,
                (*allocator).frame_desc.height as UINT,
                0 as UINT,
                (*allocator).frame_desc.d3d9_format,
                D3DPOOL_DEFAULT as UINT,
                0 as DWORD,
                DXVA2_VideoProcessorRenderTarget as core::ffi::c_int as DWORD,
                core::ptr::addr_of_mut!(surface),
                core::ptr::null_mut::<HANDLE>(),
            );
            if hr >= 0 {
                hr = MFCreateDXSurfaceBuffer(
                    core::ptr::addr_of!(IID_IDirect3DSurface9),
                    surface as *mut IUnknown,
                    FALSE as BOOL,
                    core::ptr::addr_of_mut!(buffer),
                );
                IDirect3DSurface9_Release(surface);
            }
        } else if !(*service).d3d11_device.is_null() {
            let mut desc: D3D11_TEXTURE2D_DESC = core::mem::zeroed();
            let mut texture: *mut ID3D11Texture2D = core::ptr::null_mut();

            desc.Width = (*allocator).frame_desc.width as UINT;
            desc.Height = (*allocator).frame_desc.height as UINT;
            desc.MipLevels = 1 as UINT;
            desc.ArraySize = 1 as UINT;
            desc.Format = (*allocator).frame_desc.dxgi_format;
            desc.SampleDesc.Count = 1 as UINT;
            desc.SampleDesc.Quality = 0 as UINT;
            desc.Usage = (*allocator).frame_desc.usage as D3D11_USAGE;
            desc.BindFlags = (*allocator).frame_desc.bindflags as UINT;
            if desc.Usage as core::ffi::c_uint == D3D11_USAGE_DYNAMIC as core::ffi::c_uint {
                desc.CPUAccessFlags = D3D11_CPU_ACCESS_WRITE as UINT;
            } else if desc.Usage as core::ffi::c_uint
                == D3D11_USAGE_STAGING as core::ffi::c_uint
            {
                desc.CPUAccessFlags = (D3D11_CPU_ACCESS_WRITE | D3D11_CPU_ACCESS_READ) as UINT;
            }
            desc.MiscFlags = (*allocator).frame_desc.miscflags as UINT;

            hr = ID3D11Device_CreateTexture2D(
                (*service).d3d11_device,
                core::ptr::addr_of_mut!(desc),
                core::ptr::null::<D3D11_SUBRESOURCE_DATA>(),
                core::ptr::addr_of_mut!(texture),
            );
            if hr >= 0 {
                hr = MFCreateDXGISurfaceBuffer(
                    core::ptr::addr_of!(IID_ID3D11Texture2D),
                    texture as *mut IUnknown,
                    0 as UINT,
                    FALSE as BOOL,
                    core::ptr::addr_of_mut!(buffer),
                );
                ID3D11Texture2D_Release(texture);
            }
        } else {
            hr = MFCreate2DMediaBuffer(
                (*allocator).frame_desc.width as DWORD,
                (*allocator).frame_desc.height as DWORD,
                (*allocator).frame_desc.d3d9_format as DWORD,
                FALSE as BOOL,
                core::ptr::addr_of_mut!(buffer),
            );
        }

        if hr >= 0 {
            hr = IMFSample_AddBuffer(sample, buffer);
            IMFMediaBuffer_Release(buffer);
        }

        i = i.wrapping_add(1);
    }

    if hr < 0 {
        IMFSample_Release(sample);
        return hr;
    }

    *queued_sample =
        malloc(core::mem::size_of::<queued_sample>() as usize) as *mut queued_sample;
    if (*queued_sample).is_null() {
        IMFSample_Release(sample);
        return E_OUTOFMEMORY;
    }
    (**queued_sample).sample = sample;

    return hr;
}

unsafe extern "C" fn sample_allocator_initialize(
    allocator: *mut sample_allocator,
    mut sample_count: ::core::ffi::c_uint,
    mut max_sample_count: ::core::ffi::c_uint,
    attributes: *mut IMFAttributes,
    media_type: *mut IMFMediaType,
) -> HRESULT {
    let mut service: surface_service = surface_service {
        dxva_service: core::ptr::null_mut::<IDirectXVideoProcessorService>(),
        d3d11_device: core::ptr::null_mut::<ID3D11Device>(),
        hdevice: core::ptr::null_mut::<core::ffi::c_void>(),
    };
    let mut sample: *mut queued_sample = core::ptr::null_mut::<queued_sample>();
    let mut dxgi_format: DXGI_FORMAT = DXGI_FORMAT_UNKNOWN;
    let mut i: ::core::ffi::c_uint = 0;
    let mut value: ::core::ffi::c_uint = 0;
    let mut major: GUID = core::mem::zeroed();
    let mut subtype: GUID = core::mem::zeroed();
    let mut frame_size: UINT64 = 0;
    let mut usage: UINT32 = 0;
    let mut hr: HRESULT = 0;

    hr = IMFMediaType_GetMajorType(media_type, &raw mut major);
    if hr < 0 {
        return hr;
    }

    if !IsEqualGUID!(&raw mut major, &raw const MFMediaType_Video) {
        return MF_E_INVALIDMEDIATYPE;
    }

    hr = IMFMediaType_GetUINT64(media_type, &raw const MF_MT_FRAME_SIZE, &raw mut frame_size);
    if hr < 0 {
        return hr;
    }

    hr = IMFMediaType_GetGUID(media_type, &raw const MF_MT_SUBTYPE, &raw mut subtype);
    if hr < 0 {
        return hr;
    }

    if sample_count > max_sample_count {
        return E_INVALIDARG;
    }

    usage = D3D11_USAGE_DEFAULT;
    if !attributes.is_null() {
        IMFAttributes_GetUINT32(
            attributes,
            &raw const MF_SA_BUFFERS_PER_SAMPLE,
            &raw mut (*allocator).frame_desc.buffer_count,
        );
        IMFAttributes_GetUINT32(attributes, &raw const MF_SA_D3D11_USAGE, &raw mut usage);
    }

    if usage == D3D11_USAGE_IMMUTABLE || usage > D3D11_USAGE_STAGING {
        return E_INVALIDARG;
    }

    dxgi_format = MFMapDX9FormatToDXGIFormat(subtype.Data1 as DWORD);

    (*allocator).frame_desc.bindflags = 0;
    (*allocator).frame_desc.miscflags = 0;
    (*allocator).frame_desc.usage = D3D11_USAGE_DEFAULT;

    if dxgi_format as ::core::ffi::c_uint == DXGI_FORMAT_B8G8R8A8_UNORM
        || dxgi_format as ::core::ffi::c_uint == DXGI_FORMAT_B8G8R8X8_UNORM
    {
        (*allocator).frame_desc.usage = usage as ::core::ffi::c_uint;
        if (*allocator).frame_desc.usage == D3D11_USAGE_DEFAULT {
            (*allocator).frame_desc.bindflags = D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_RENDER_TARGET;
        } else if (*allocator).frame_desc.usage == D3D11_USAGE_DYNAMIC {
            (*allocator).frame_desc.bindflags = D3D11_BIND_SHADER_RESOURCE;
        }
    }

    if !attributes.is_null() {
        IMFAttributes_GetUINT32(
            attributes,
            &raw const MF_SA_D3D11_BINDFLAGS,
            &raw mut (*allocator).frame_desc.bindflags,
        );
        if IMFAttributes_GetUINT32(attributes, &raw const MF_SA_D3D11_SHARED, &raw mut value)
            >= 0
            && value != 0
        {
            (*allocator).frame_desc.miscflags |= D3D11_RESOURCE_MISC_SHARED_KEYEDMUTEX;
        }
        if IMFAttributes_GetUINT32(attributes, &raw const MF_SA_D3D11_SHARED_WITHOUT_MUTEX, &raw mut value)
            >= 0
            && value != 0
        {
            (*allocator).frame_desc.miscflags |= D3D11_RESOURCE_MISC_SHARED;
        }
    }

    sample_allocator_set_media_type(allocator, media_type);
    sample_allocator_set_attributes(allocator, attributes);

    sample_count = max!(1, sample_count);
    max_sample_count = max!(1, max_sample_count);

    (*allocator).frame_desc.d3d9_format = subtype.Data1 as D3DFORMAT;
    (*allocator).frame_desc.dxgi_format = dxgi_format;
    (*allocator).frame_desc.width = (frame_size >> 32) as ::core::ffi::c_uint;
    (*allocator).frame_desc.height = frame_size as ::core::ffi::c_uint;
    (*allocator).frame_desc.buffer_count = max!(
        1,
        (*allocator).frame_desc.buffer_count,
    );

    hr = sample_allocator_get_surface_service(allocator, &raw mut service);
    if hr < 0 {
        return hr;
    }

    sample_allocator_release_samples(allocator);

    i = 0;
    while i < sample_count {
        hr = sample_allocator_allocate_sample(allocator, &raw mut service, &raw mut sample);
        if hr >= 0 {
            list_add_tail(
                &raw mut (*allocator).free_samples,
                &raw mut (*sample).entry,
            );
            (*allocator).free_sample_count = (*allocator).free_sample_count.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    (*allocator).cold_sample_count = max_sample_count.wrapping_sub((*allocator).free_sample_count);

    sample_allocator_release_surface_service(allocator, &raw mut service);

    return hr;
}

unsafe extern "system" fn sample_allocator_InitializeSampleAllocator(iface: *mut IMFVideoSampleAllocatorEx, sample_count: DWORD, media_type: *mut IMFMediaType) -> HRESULT {
    let allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocatorEx(iface);
    let mut hr: HRESULT;

    if sample_count == 0 {
        return E_INVALIDARG;
    }
    if media_type.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));

    hr = sample_allocator_initialize(allocator, sample_count, sample_count, core::ptr::null_mut(), media_type);

    LeaveCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));

    return hr;
}

unsafe extern "C" fn sample_allocator_track_sample(allocator: *mut sample_allocator, sample: *mut IMFSample) -> HRESULT {
    let mut tracked_sample: *mut IMFTrackedSample = core::ptr::null_mut();
    let mut hr: HRESULT;
    if {
        hr = IMFSample_QueryInterface(
            sample,
            core::ptr::addr_of!(IID_IMFTrackedSample),
            core::ptr::addr_of_mut!(tracked_sample) as *mut *mut core::ffi::c_void,
        );
        hr >= 0
    } {
        hr = IMFTrackedSample_SetAllocator(
            tracked_sample,
            core::ptr::addr_of_mut!((*allocator).tracking_callback),
            core::ptr::null_mut(),
        );
        IMFTrackedSample_Release(tracked_sample);
    }

    return hr;
}

unsafe extern "system" fn sample_allocator_AllocateSample(
    mut iface: *mut IMFVideoSampleAllocatorEx,
    mut out: *mut *mut IMFSample,
) -> HRESULT {
    let mut allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocatorEx(iface);
    let mut sample: *mut queued_sample = ::core::ptr::null_mut::<queued_sample>();
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
            b"dll/win32/mfplat/sample.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"sample_allocator_AllocateSample\0" as *const u8 as *const ::core::ffi::c_char,
            1553 as ::core::ffi::c_int,
            b"%p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            out,
        );
    }

    EnterCriticalSection(&raw mut (*allocator).cs);

    if list_empty(&raw mut (*allocator).free_samples) != 0
        && list_empty(&raw mut (*allocator).used_samples) != 0
    {
        hr = MF_E_NOT_INITIALIZED;
    } else if list_empty(&raw mut (*allocator).free_samples) != 0
        && (*allocator).cold_sample_count == 0
    {
        hr = MF_E_SAMPLEALLOCATOR_EMPTY;
    } else if list_empty(&raw mut (*allocator).free_samples) == 0 {
        let mut head: *mut list = list_head(&raw mut (*allocator).free_samples);

        sample = (head as *mut ::core::ffi::c_char).offset(
            -(&raw mut (*::core::ptr::null_mut::<queued_sample>()).entry as ::core::ffi::c_ulonglong
                as isize),
        ) as *mut queued_sample;

        if {
            hr = sample_allocator_track_sample(allocator, (*sample).sample);
            hr
        } >= 0
        {
            list_remove(head);
            list_add_tail(&raw mut (*allocator).used_samples, head);
            (*allocator).free_sample_count = (*allocator).free_sample_count.wrapping_sub(1);

            /* Reference counter is not increased when sample is returned, so next release could trigger
               tracking condition. This is balanced by incremented reference counter when sample is returned
               back to the free list. */
            *out = (*sample).sample;
        }
    } else {
        let mut service: surface_service = surface_service {
            dxva_service: ::core::ptr::null_mut::<IDirectXVideoProcessorService>(),
            d3d11_device: ::core::ptr::null_mut::<ID3D11Device>(),
            hdevice: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };

        if {
            hr = sample_allocator_get_surface_service(allocator, &raw mut service);
            hr
        } >= 0
        {
            if {
                hr = sample_allocator_allocate_sample(allocator, &raw mut service, &raw mut sample);
                hr
            } >= 0
            {
                if {
                    hr = sample_allocator_track_sample(allocator, (*sample).sample);
                    hr
                } >= 0
                {
                    list_add_tail(&raw mut (*allocator).used_samples, &raw mut (*sample).entry);
                    (*allocator).cold_sample_count = (*allocator).cold_sample_count.wrapping_sub(1);

                    *out = (*sample).sample;
                } else {
                    IMFSample_Release((*sample).sample);
                    free(sample as *mut ::core::ffi::c_void);
                }
            }

            sample_allocator_release_surface_service(allocator, &raw mut service);
        }
    }

    LeaveCriticalSection(&raw mut (*allocator).cs);

    return hr;
}

unsafe extern "system" fn sample_allocator_InitializeSampleAllocatorEx(iface: *mut IMFVideoSampleAllocatorEx, initial_sample_count: DWORD, max_sample_count: DWORD, attributes: *mut IMFAttributes, media_type: *mut IMFMediaType) -> HRESULT {
    let allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocatorEx(iface);
    let mut hr: HRESULT;

    if media_type.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));

    hr = sample_allocator_initialize(allocator, initial_sample_count, max_sample_count, attributes, media_type);

    LeaveCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));

    return hr;
}

unsafe extern "system" fn sample_allocator_callback_QueryInterface(iface: *mut IMFVideoSampleAllocatorCallback, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocatorCallback(iface);
    return IMFVideoSampleAllocatorEx_QueryInterface(core::ptr::addr_of_mut!((*allocator).IMFVideoSampleAllocatorEx_iface), riid, obj);
}

unsafe extern "system" fn sample_allocator_callback_AddRef(iface: *mut IMFVideoSampleAllocatorCallback) -> ULONG {
    let mut allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocatorCallback(iface);
    return IMFVideoSampleAllocatorEx_AddRef(core::ptr::addr_of_mut!((*allocator).IMFVideoSampleAllocatorEx_iface));
}

unsafe extern "system" fn sample_allocator_callback_Release(iface: *mut IMFVideoSampleAllocatorCallback) -> ULONG {
    let allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocatorCallback(iface);
    return IMFVideoSampleAllocatorEx_Release(core::ptr::addr_of_mut!((*allocator).IMFVideoSampleAllocatorEx_iface));
}

unsafe extern "system" fn sample_allocator_callback_SetCallback(
    iface: *mut IMFVideoSampleAllocatorCallback,
    callback: *mut IMFVideoSampleAllocatorNotify,
) -> HRESULT {
    let allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocatorCallback(iface);

    TRACE!("%p, %p.\n", iface, callback);

    EnterCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));
    if !(*allocator).callback.is_null() {
        IMFVideoSampleAllocatorNotify_Release((*allocator).callback);
    }
    (*allocator).callback = callback;
    if !(*allocator).callback.is_null() {
        IMFVideoSampleAllocatorNotify_AddRef((*allocator).callback);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));

    return S_OK;
}

unsafe extern "system" fn sample_allocator_callback_GetFreeSampleCount(
    iface: *mut IMFVideoSampleAllocatorCallback,
    count: *mut LONG,
) -> HRESULT {
    let allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocatorCallback(iface);

    TRACE!("%p, %p.\n", iface, count);

    if count.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));
    *count = (*allocator).free_sample_count as LONG;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));

    return S_OK;
}

unsafe extern "system" fn sample_allocator_tracking_callback_QueryInterface(
    iface: *mut IMFAsyncCallback,
    riid: *const GUID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
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

unsafe extern "system" fn sample_allocator_tracking_callback_AddRef(iface: *mut IMFAsyncCallback) -> ULONG {
    let mut allocator: *mut sample_allocator = impl_from_IMFAsyncCallback(iface);
    return IMFVideoSampleAllocatorEx_AddRef(core::ptr::addr_of_mut!((*allocator).IMFVideoSampleAllocatorEx_iface));
}

unsafe extern "system" fn sample_allocator_tracking_callback_Release(iface: *mut IMFAsyncCallback) -> ULONG {
    let allocator: *mut sample_allocator = impl_from_IMFAsyncCallback(iface);
    return IMFVideoSampleAllocatorEx_Release(core::ptr::addr_of_mut!((*allocator).IMFVideoSampleAllocatorEx_iface));
}
unsafe extern "system" fn sample_allocator_tracking_callback_GetParameters(
    iface: *mut IMFAsyncCallback,
    flags: *mut DWORD,
    queue: *mut DWORD,
) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "system" fn sample_allocator_tracking_callback_Invoke(iface: *mut IMFAsyncCallback, result: *mut IMFAsyncResult) -> HRESULT {
    let allocator: *mut sample_allocator = impl_from_IMFAsyncCallback(iface);
    let mut iter: *mut queued_sample = core::ptr::null_mut();
    let mut object: *mut IUnknown = core::ptr::null_mut();
    let mut sample: *mut IMFSample = core::ptr::null_mut();
    let mut hr: HRESULT;

    if (IMFAsyncResult_GetObject(result, core::ptr::addr_of_mut!(object))) < 0 {
        return E_UNEXPECTED;
    }

    hr = IUnknown_QueryInterface(object, core::ptr::addr_of!(IID_IMFSample), core::ptr::addr_of_mut!(sample) as *mut *mut core::ffi::c_void);
    IUnknown_Release(object);
    if hr < 0 {
        return E_UNEXPECTED;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));

    iter = ((*allocator).used_samples.next as *mut u8).sub(core::mem::offset_of!(queued_sample, entry)) as *mut queued_sample;
    while core::ptr::addr_of_mut!((*iter).entry) != core::ptr::addr_of_mut!((*allocator).used_samples) {
        if sample == (*iter).sample {
            list_remove(core::ptr::addr_of_mut!((*iter).entry));
            list_add_tail(core::ptr::addr_of_mut!((*allocator).free_samples), core::ptr::addr_of_mut!((*iter).entry));
            IMFSample_AddRef((*iter).sample);
            (*allocator).free_sample_count = (*allocator).free_sample_count.wrapping_add(1);
            break;
        } else {
            iter = ((*iter).entry.next as *mut u8).sub(core::mem::offset_of!(queued_sample, entry)) as *mut queued_sample;
        }
    }

    IMFSample_Release(sample);

    if !(*allocator).callback.is_null() {
        IMFVideoSampleAllocatorNotify_NotifyRelease((*allocator).callback);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateVideoSampleAllocatorEx(riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut object: *mut sample_allocator;
    let mut hr: HRESULT;

    object = calloc(1, (core::mem::size_of::<sample_allocator>()) as _) as *mut sample_allocator;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).IMFVideoSampleAllocatorEx_iface.lpVtbl = &raw const sample_allocator_vtbl;
    (*object).IMFVideoSampleAllocatorCallback_iface.lpVtbl = &raw const sample_allocator_callback_vtbl;
    (*object).tracking_callback.lpVtbl = &raw const sample_allocator_tracking_callback_vtbl;
    (*object).refcount = 1;
    list_init(&raw mut (*object).used_samples);
    list_init(&raw mut (*object).free_samples);
    InitializeCriticalSection(&raw mut (*object).cs);

    hr = IMFVideoSampleAllocatorEx_QueryInterface(&raw mut (*object).IMFVideoSampleAllocatorEx_iface, riid, obj);
    IMFVideoSampleAllocatorEx_Release(&raw mut (*object).IMFVideoSampleAllocatorEx_iface);

    return hr;
}

// COM vtable tables — generated from sample.c by vtbl_gen.py
/// sample.c:879 `static const IMFSampleVtbl samplevtbl`  (slot order: rust:types_amd64.rs)
pub static samplevtbl: IMFSampleVtbl = IMFSampleVtbl {
    QueryInterface: Some(sample_QueryInterface),
    AddRef: Some(sample_AddRef),
    Release: Some(sample_Release),
    GetItem: Some(sample_GetItem),
    GetItemType: Some(sample_GetItemType),
    CompareItem: Some(sample_CompareItem),
    Compare: Some(sample_Compare),
    GetUINT32: Some(sample_GetUINT32),
    GetUINT64: Some(sample_GetUINT64),
    GetDouble: Some(sample_GetDouble),
    GetGUID: Some(sample_GetGUID),
    GetStringLength: Some(sample_GetStringLength),
    GetString: Some(sample_GetString),
    GetAllocatedString: Some(sample_GetAllocatedString),
    GetBlobSize: Some(sample_GetBlobSize),
    GetBlob: Some(sample_GetBlob),
    GetAllocatedBlob: Some(sample_GetAllocatedBlob),
    GetUnknown: Some(sample_GetUnknown),
    SetItem: Some(sample_SetItem),
    DeleteItem: Some(sample_DeleteItem),
    DeleteAllItems: Some(sample_DeleteAllItems),
    SetUINT32: Some(sample_SetUINT32),
    SetUINT64: Some(sample_SetUINT64),
    SetDouble: Some(sample_SetDouble),
    SetGUID: Some(sample_SetGUID),
    SetString: Some(sample_SetString),
    SetBlob: Some(sample_SetBlob),
    SetUnknown: Some(sample_SetUnknown),
    LockStore: Some(sample_LockStore),
    UnlockStore: Some(sample_UnlockStore),
    GetCount: Some(sample_GetCount),
    GetItemByIndex: Some(sample_GetItemByIndex),
    CopyAllItems: Some(sample_CopyAllItems),
    GetSampleFlags: Some(sample_GetSampleFlags),
    SetSampleFlags: Some(sample_SetSampleFlags),
    GetSampleTime: Some(sample_GetSampleTime),
    SetSampleTime: Some(sample_SetSampleTime),
    GetSampleDuration: Some(sample_GetSampleDuration),
    SetSampleDuration: Some(sample_SetSampleDuration),
    GetBufferCount: Some(sample_GetBufferCount),
    GetBufferByIndex: Some(sample_GetBufferByIndex),
    ConvertToContiguousBuffer: Some(sample_ConvertToContiguousBuffer),
    AddBuffer: Some(sample_AddBuffer),
    RemoveBufferByIndex: Some(sample_RemoveBufferByIndex),
    RemoveAllBuffers: Some(sample_RemoveAllBuffers),
    GetTotalLength: Some(sample_GetTotalLength),
    CopyToBuffer: Some(sample_CopyToBuffer),
};

/// sample.c:981 `static const IMFTrackedSampleVtbl tracked_sample_vtbl`  (slot order: rust:types_amd64.rs)
pub static tracked_sample_vtbl: IMFTrackedSampleVtbl = IMFTrackedSampleVtbl {
    QueryInterface: Some(tracked_sample_QueryInterface),
    AddRef: Some(tracked_sample_AddRef),
    Release: Some(tracked_sample_Release),
    SetAllocator: Some(tracked_sample_SetAllocator),
};

/// sample.c:989 `static const IMFSampleVtbl sample_tracked_vtbl`  (slot order: rust:types_amd64.rs)
pub static sample_tracked_vtbl: IMFSampleVtbl = IMFSampleVtbl {
    QueryInterface: Some(sample_QueryInterface),
    AddRef: Some(sample_AddRef),
    Release: Some(sample_tracked_Release),
    GetItem: Some(sample_GetItem),
    GetItemType: Some(sample_GetItemType),
    CompareItem: Some(sample_CompareItem),
    Compare: Some(sample_Compare),
    GetUINT32: Some(sample_GetUINT32),
    GetUINT64: Some(sample_GetUINT64),
    GetDouble: Some(sample_GetDouble),
    GetGUID: Some(sample_GetGUID),
    GetStringLength: Some(sample_GetStringLength),
    GetString: Some(sample_GetString),
    GetAllocatedString: Some(sample_GetAllocatedString),
    GetBlobSize: Some(sample_GetBlobSize),
    GetBlob: Some(sample_GetBlob),
    GetAllocatedBlob: Some(sample_GetAllocatedBlob),
    GetUnknown: Some(sample_GetUnknown),
    SetItem: Some(sample_SetItem),
    DeleteItem: Some(sample_DeleteItem),
    DeleteAllItems: Some(sample_DeleteAllItems),
    SetUINT32: Some(sample_SetUINT32),
    SetUINT64: Some(sample_SetUINT64),
    SetDouble: Some(sample_SetDouble),
    SetGUID: Some(sample_SetGUID),
    SetString: Some(sample_SetString),
    SetBlob: Some(sample_SetBlob),
    SetUnknown: Some(sample_SetUnknown),
    LockStore: Some(sample_LockStore),
    UnlockStore: Some(sample_UnlockStore),
    GetCount: Some(sample_GetCount),
    GetItemByIndex: Some(sample_GetItemByIndex),
    CopyAllItems: Some(sample_CopyAllItems),
    GetSampleFlags: Some(sample_GetSampleFlags),
    SetSampleFlags: Some(sample_SetSampleFlags),
    GetSampleTime: Some(sample_GetSampleTime),
    SetSampleTime: Some(sample_SetSampleTime),
    GetSampleDuration: Some(sample_GetSampleDuration),
    SetSampleDuration: Some(sample_SetSampleDuration),
    GetBufferCount: Some(sample_GetBufferCount),
    GetBufferByIndex: Some(sample_GetBufferByIndex),
    ConvertToContiguousBuffer: Some(sample_ConvertToContiguousBuffer),
    AddBuffer: Some(sample_AddBuffer),
    RemoveBufferByIndex: Some(sample_RemoveBufferByIndex),
    RemoveAllBuffers: Some(sample_RemoveAllBuffers),
    GetTotalLength: Some(sample_GetTotalLength),
    CopyToBuffer: Some(sample_CopyToBuffer),
};

/// sample.c:1630 `static const IMFVideoSampleAllocatorExVtbl sample_allocator_vtbl`  (slot order: rust:types_amd64.rs)
pub static sample_allocator_vtbl: IMFVideoSampleAllocatorExVtbl = IMFVideoSampleAllocatorExVtbl {
    QueryInterface: Some(sample_allocator_QueryInterface),
    AddRef: Some(sample_allocator_AddRef),
    Release: Some(sample_allocator_Release),
    SetDirectXManager: Some(sample_allocator_SetDirectXManager),
    UninitializeSampleAllocator: Some(sample_allocator_UninitializeSampleAllocator),
    InitializeSampleAllocator: Some(sample_allocator_InitializeSampleAllocator),
    AllocateSample: Some(sample_allocator_AllocateSample),
    InitializeSampleAllocatorEx: Some(sample_allocator_InitializeSampleAllocatorEx),
};

/// sample.c:1696 `static const IMFVideoSampleAllocatorCallbackVtbl sample_allocator_callback_vtbl`  (slot order: rust:types_amd64.rs)
pub static sample_allocator_callback_vtbl: IMFVideoSampleAllocatorCallbackVtbl = IMFVideoSampleAllocatorCallbackVtbl {
    QueryInterface: Some(sample_allocator_callback_QueryInterface),
    AddRef: Some(sample_allocator_callback_AddRef),
    Release: Some(sample_allocator_callback_Release),
    SetCallback: Some(sample_allocator_callback_SetCallback),
    GetFreeSampleCount: Some(sample_allocator_callback_GetFreeSampleCount),
};

/// sample.c:1779 `static const IMFAsyncCallbackVtbl sample_allocator_tracking_callback_vtbl`  (slot order: rust:types_amd64.rs)
pub static sample_allocator_tracking_callback_vtbl: IMFAsyncCallbackVtbl = IMFAsyncCallbackVtbl {
    QueryInterface: Some(sample_allocator_tracking_callback_QueryInterface),
    AddRef: Some(sample_allocator_tracking_callback_AddRef),
    Release: Some(sample_allocator_tracking_callback_Release),
    GetParameters: Some(sample_allocator_tracking_callback_GetParameters),
    Invoke: Some(sample_allocator_tracking_callback_Invoke),
};
