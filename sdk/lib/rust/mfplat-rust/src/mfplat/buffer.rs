use crate::mfplat::mediatype::media_type;
use crate::mfplat::sample::sample;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[repr(C)]
pub struct buffer_d3d12_list_allocator {
    pub free_value: UINT64,
    pub allocator: *mut ID3D12CommandAllocator,
}

pub type d3d12_list_allocator = buffer_d3d12_list_allocator;

pub type p_copy_image_func = Option<unsafe extern "C" fn(
    dest: *mut BYTE,
    dest_stride: LONG,
    src: *const BYTE,
    src_stride: LONG,
    width: DWORD,
    lines: DWORD,
)>;

#[repr(C)]
pub struct buffer_2d {
    pub linear_buffer: *mut BYTE,
    pub plane_size: DWORD,
    pub scanline0: *mut BYTE,
    pub width: core::ffi::c_uint,
    pub height: core::ffi::c_uint,
    pub pitch: core::ffi::c_int,
    pub locks: core::ffi::c_uint,
    pub lock_flags: MF2DBuffer_LockFlags,
    pub copy_image: p_copy_image_func,
}

#[repr(C)]
pub struct buffer_d3d9_surface {
    pub surface: *mut IDirect3DSurface9,
    pub rect: D3DLOCKED_RECT,
}

#[repr(C)]
pub struct buffer_dxgi_surface {
    pub texture: *mut ID3D11Texture2D,
    pub sub_resource_idx: core::ffi::c_uint,
    pub rb_texture: *mut ID3D11Texture2D,
    pub map_desc: D3D11_MAPPED_SUBRESOURCE,
    pub attributes: attributes,
}

#[repr(C)]
pub struct buffer_d3d12_surface {
    pub device: *mut ID3D12Device,
    pub resource: *mut ID3D12Resource,
    pub sync_obj: *mut IMFD3D12SynchronizationObject,
    pub sync_cmd: *mut IMFD3D12SynchronizationObjectCommands,
    pub layout: D3D12_PLACED_SUBRESOURCE_FOOTPRINT,
    pub data: *mut core::ffi::c_void,
    pub keep_transfer: BOOL,
    pub upload: *mut ID3D12Resource,
    pub readback: *mut ID3D12Resource,
    pub transfer: *mut ID3D12Resource,
    pub queue: *mut ID3D12CommandQueue,
    pub fence: *mut ID3D12Fence,
    pub fence_value: UINT64,
    pub alloc_index: usize,
    pub list: *mut ID3D12GraphicsCommandList,
    pub alloc: [buffer_d3d12_list_allocator; 2],
}

#[repr(C)]
pub struct buffer {
    pub IMFMediaBuffer_iface: IMFMediaBuffer,
    pub IMF2DBuffer2_iface: IMF2DBuffer2,
    pub IMFDXGIBuffer_iface: IMFDXGIBuffer,
    pub IMFGetService_iface: IMFGetService,
    pub refcount: LONG,

    pub data: *mut BYTE,
    pub max_length: DWORD,
    pub current_length: DWORD,

    pub _2d: buffer_2d,
    pub d3d9_surface: buffer_d3d9_surface,
    pub dxgi_surface: buffer_dxgi_surface,
    pub d3d12_surface: buffer_d3d12_surface,

    pub cs: CRITICAL_SECTION,
}

#[repr(C)]
pub struct legacy_buffer {
    pub IMediaBuffer_iface: IMediaBuffer,
    pub IMF2DBuffer2_iface: IMF2DBuffer2,
    pub IMFSample_iface: IMFSample,
    pub refcount: LONG,

    pub media_buffer: *mut IMFMediaBuffer,
    pub locked_data: *mut core::ffi::c_void,

    pub sample: *mut IMFSample,
    pub buffer_2d: *mut IMF2DBuffer2,
}

#[repr(C)]
pub struct buffer_wrapper {
    pub IMFMediaBuffer_iface: IMFMediaBuffer,
    pub refcount: LONG,
    pub parent: *mut IMFMediaBuffer,
    pub offset: DWORD,
    pub max_length: DWORD,
    pub current_length: DWORD,
}

#[allow(non_snake_case)]

const __WINE_DBCL_TRACE: u32 = 3; // enum position of TRACE in __wine_debug_class

unsafe extern "C" fn copy_image(
    buffer: *const buffer,
    mut dest: *mut BYTE,
    dest_stride: LONG,
    mut src: *const BYTE,
    src_stride: LONG,
    width: DWORD,
    lines: DWORD,
) {
    MFCopyImage(dest, dest_stride, src, src_stride, width, lines);

    if (*buffer)._2d.copy_image.is_some() {
        dest = dest.offset((dest_stride as DWORD).wrapping_mul(lines) as isize);
        src = src.offset((src_stride as DWORD).wrapping_mul(lines) as isize);
        (*buffer)._2d.copy_image.unwrap_unchecked()(
            dest,
            dest_stride,
            src,
            src_stride,
            width,
            lines,
        );
    }
}

unsafe extern "C" fn copy_image_nv12(
    dest: *mut BYTE,
    dest_stride: LONG,
    src: *const BYTE,
    src_stride: LONG,
    width: DWORD,
    lines: DWORD,
) {
    MFCopyImage(dest, dest_stride, src, src_stride, width, lines / 2);
}

unsafe extern "C" fn copy_image_imc1(dest: *mut BYTE, dest_stride: LONG, src: *const BYTE, src_stride: LONG, width: DWORD, lines: DWORD) {
    MFCopyImage(dest, dest_stride, src, src_stride, width / 2, lines);
}

unsafe extern "C" fn copy_image_imc2(mut dest: *mut BYTE, mut dest_stride: LONG, mut src: *const BYTE, mut src_stride: LONG, mut width: DWORD, mut lines: DWORD) {
    MFCopyImage(dest, dest_stride / 2, src, src_stride / 2, width / 2, lines);
}

unsafe extern "C" fn impl_from_IMFMediaBuffer(iface: *mut IMFMediaBuffer) -> *mut buffer {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(buffer, IMFMediaBuffer_iface)) as *mut buffer;
}

unsafe extern "C" fn impl_from_IMF2DBuffer2(iface: *mut IMF2DBuffer2) -> *mut buffer {
    return CONTAINING_RECORD!(iface, buffer, IMF2DBuffer2_iface);
}

unsafe extern "C" fn impl_from_IMFGetService(iface: *mut IMFGetService) -> *mut buffer {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(buffer, IMFGetService_iface)) as *mut buffer;
}

unsafe extern "C" fn impl_from_IMFDXGIBuffer(iface: *mut IMFDXGIBuffer) -> *mut buffer {
    return CONTAINING_RECORD!(iface, buffer, IMFDXGIBuffer_iface);
}

unsafe extern "system" fn memory_buffer_QueryInterface(iface: *mut IMFMediaBuffer, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMFMediaBuffer(iface);

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFMediaBuffer)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *out = core::ptr::addr_of_mut!((*buffer).IMFMediaBuffer_iface) as *mut core::ffi::c_void;
        IMFMediaBuffer_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported %s.\n", debugstr_guid(riid));
    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}
unsafe extern "system" fn memory_buffer_AddRef(iface: *mut IMFMediaBuffer) -> ULONG {
    let buffer: *mut buffer = impl_from_IMFMediaBuffer(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*buffer).refcount)) as ULONG;

    return refcount;
}

unsafe extern "system" fn memory_buffer_Release(iface: *mut IMFMediaBuffer) -> ULONG {
    let buffer: *mut buffer = impl_from_IMFMediaBuffer(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*buffer).refcount)) as ULONG;

    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE) as i32;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/buffer.c\0".as_ptr() as *const i8,
            b"memory_buffer_Release\0".as_ptr() as *const i8,
            184i32,
            b"%p, refcount %lu.\n\0".as_ptr() as *const i8,
            iface,
            refcount,
        );
    }

    if refcount == 0 {
        if !(*buffer).d3d9_surface.surface.is_null() {
            IDirect3DSurface9_Release((*buffer).d3d9_surface.surface);
        }
        if !(*buffer).dxgi_surface.texture.is_null() {
            ID3D11Texture2D_Release((*buffer).dxgi_surface.texture);
            if !(*buffer).dxgi_surface.rb_texture.is_null() {
                ID3D11Texture2D_Release((*buffer).dxgi_surface.rb_texture);
            }
            clear_attributes_object(core::ptr::addr_of_mut!((*buffer).dxgi_surface.attributes));
        }
        if !(*buffer).d3d12_surface.resource.is_null() {
            ID3D12Device_Release((*buffer).d3d12_surface.device);
            ID3D12Resource_Release((*buffer).d3d12_surface.resource);
            if !(*buffer).d3d12_surface.sync_obj.is_null() {
                IMFD3D12SynchronizationObject_Release((*buffer).d3d12_surface.sync_obj);
            }
            if !(*buffer).d3d12_surface.sync_cmd.is_null() {
                IMFD3D12SynchronizationObjectCommands_Release((*buffer).d3d12_surface.sync_cmd);
            }
            if !(*buffer).d3d12_surface.readback.is_null() {
                ID3D12Resource_Release((*buffer).d3d12_surface.readback);
            }
            if !(*buffer).d3d12_surface.upload.is_null() {
                ID3D12Resource_Release((*buffer).d3d12_surface.upload);
            }
            if !(*buffer).d3d12_surface.queue.is_null() {
                ID3D12CommandQueue_Release((*buffer).d3d12_surface.queue);
            }
            if !(*buffer).d3d12_surface.fence.is_null() {
                ID3D12Fence_Release((*buffer).d3d12_surface.fence);
            }
            if !(*buffer).d3d12_surface.list.is_null() {
                ID3D12GraphicsCommandList_Release((*buffer).d3d12_surface.list);
            }
            if !(*buffer).d3d12_surface.alloc[0 as usize].allocator.is_null() {
                ID3D12CommandAllocator_Release((*buffer).d3d12_surface.alloc[0 as usize].allocator);
            }
            if !(*buffer).d3d12_surface.alloc[1 as usize].allocator.is_null() {
                ID3D12CommandAllocator_Release((*buffer).d3d12_surface.alloc[1 as usize].allocator);
            }
            clear_attributes_object(core::ptr::addr_of_mut!((*buffer).dxgi_surface.attributes));
        }
        DeleteCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));
        free((*buffer)._2d.linear_buffer as *mut core::ffi::c_void);
        _aligned_free((*buffer).data as *mut core::ffi::c_void);
        free(buffer as *mut core::ffi::c_void);
    }

    refcount
}

unsafe extern "system" fn memory_buffer_Lock(iface: *mut IMFMediaBuffer, data: *mut *mut BYTE, max_length: *mut DWORD, current_length: *mut DWORD) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMFMediaBuffer(iface);

    TRACE!("%p, %p %p, %p.\n", iface, data, max_length, current_length);

    if data.is_null() {
        return E_INVALIDARG;
    }

    *data = (*buffer).data;
    if !max_length.is_null() {
        *max_length = (*buffer).max_length;
    }
    if !current_length.is_null() {
        *current_length = (*buffer).current_length;
    }

    return S_OK;
}
unsafe extern "system" fn memory_buffer_Unlock(iface: *mut IMFMediaBuffer) -> HRESULT {
    // TRACE("%p.\n", iface);
    S_OK
}

unsafe extern "system" fn memory_buffer_GetCurrentLength(
    iface: *mut IMFMediaBuffer,
    current_length: *mut DWORD,
) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMFMediaBuffer(iface);

    if current_length.is_null() {
        return E_INVALIDARG;
    }

    *current_length = (*buffer).current_length;

    return S_OK;
}

unsafe extern "system" fn memory_buffer_SetCurrentLength(iface: *mut IMFMediaBuffer, current_length: DWORD) -> HRESULT {
    let mut buffer: *mut buffer = impl_from_IMFMediaBuffer(iface);

    if current_length > (*buffer).max_length {
        return E_INVALIDARG;
    }

    (*buffer).current_length = current_length;

    return S_OK;
}

unsafe extern "system" fn memory_buffer_GetMaxLength(iface: *mut IMFMediaBuffer, max_length: *mut DWORD) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMFMediaBuffer(iface);

    if max_length.is_null() {
        return E_INVALIDARG;
    }

    *max_length = (*buffer).max_length;

    return S_OK;
}

unsafe extern "system" fn memory_1d_2d_buffer_QueryInterface(iface: *mut IMFMediaBuffer, riid: *const GUID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMFMediaBuffer(iface);

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFMediaBuffer)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *out = core::ptr::addr_of_mut!((*buffer).IMFMediaBuffer_iface) as *mut core::ffi::c_void;
    }
    else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMF2DBuffer2)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IMF2DBuffer)) != 0
    {
        *out = core::ptr::addr_of_mut!((*buffer).IMF2DBuffer2_iface) as *mut core::ffi::c_void;
    }
    else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFGetService)) != 0
    {
        *out = core::ptr::addr_of_mut!((*buffer).IMFGetService_iface) as *mut core::ffi::c_void;
    }
    else
    {
        WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
        *out = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*out as *mut IUnknown);
    return S_OK;
}
unsafe extern "system" fn memory_1d_2d_buffer_Lock(iface: *mut IMFMediaBuffer, data: *mut *mut BYTE, max_length: *mut DWORD, current_length: *mut DWORD) -> HRESULT {
    let mut buffer: *mut buffer = impl_from_IMFMediaBuffer(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!("%p, %p, %p, %p.\n", iface, data, max_length, current_length);

    if data.is_null() {
        return E_POINTER;
    }

    /* Allocate linear buffer and return it as a copy of current content. Maximum and current length are
       unrelated to 2D buffer maximum allocate length, or maintained current length. */

    EnterCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    if (*buffer)._2d.linear_buffer.is_null() && (*buffer)._2d.locks != 0 {
        hr = MF_E_INVALIDREQUEST;
    } else if (*buffer)._2d.linear_buffer.is_null() {
        (*buffer)._2d.linear_buffer = malloc((*buffer)._2d.plane_size as usize) as *mut BYTE;
        if (*buffer)._2d.linear_buffer.is_null() {
            hr = E_OUTOFMEMORY;
        }

        if hr >= 0 {
            let mut pitch: core::ffi::c_int = (*buffer)._2d.pitch;

            if pitch < 0 {
                pitch = -pitch;
            }
            copy_image(
                buffer,
                (*buffer)._2d.linear_buffer,
                (*buffer)._2d.width as LONG,
                (*buffer).data,
                pitch as LONG,
                (*buffer)._2d.width as DWORD,
                (*buffer)._2d.height as DWORD,
            );
        }
    }

    if hr >= 0 {
        (*buffer)._2d.locks = (*buffer)._2d.locks.wrapping_add(1);
        *data = (*buffer)._2d.linear_buffer;
        if !max_length.is_null() {
            *max_length = (*buffer)._2d.plane_size;
        }
        if !current_length.is_null() {
            *current_length = (*buffer)._2d.plane_size;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    return hr;
}

unsafe extern "system" fn memory_1d_2d_buffer_Unlock(iface: *mut IMFMediaBuffer) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMFMediaBuffer(iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    if !(*buffer)._2d.linear_buffer.is_null() && {
        (*buffer)._2d.locks = (*buffer)._2d.locks.wrapping_sub(1);
        (*buffer)._2d.locks == 0
    } {
        let mut pitch: i32 = (*buffer)._2d.pitch;

        if pitch < 0 {
            pitch = -pitch;
        }
        copy_image(
            buffer,
            (*buffer).data,
            pitch as LONG,
            (*buffer)._2d.linear_buffer,
            (*buffer)._2d.width as LONG,
            (*buffer)._2d.width as DWORD,
            (*buffer)._2d.height as DWORD,
        );

        free((*buffer)._2d.linear_buffer as *mut core::ffi::c_void);
        (*buffer)._2d.linear_buffer = core::ptr::null_mut::<BYTE>();
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    return S_OK;
}

unsafe extern "system" fn d3d9_surface_buffer_Lock(iface: *mut IMFMediaBuffer, data: *mut *mut BYTE, max_length: *mut DWORD, current_length: *mut DWORD) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMFMediaBuffer(iface);
    let mut hr: HRESULT = S_OK;

    if data.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    if (*buffer)._2d.linear_buffer.is_null() && (*buffer)._2d.locks != 0 {
        hr = MF_E_INVALIDREQUEST;
    } else if (*buffer)._2d.linear_buffer.is_null() {
        let mut rect: D3DLOCKED_RECT = core::mem::zeroed();

        (*buffer)._2d.linear_buffer = malloc((*buffer)._2d.plane_size as usize) as *mut BYTE;
        if (*buffer)._2d.linear_buffer.is_null() {
            hr = E_OUTOFMEMORY;
        }

        if hr >= 0 {
            hr = IDirect3DSurface9_LockRect(
                (*buffer).d3d9_surface.surface,
                core::ptr::addr_of_mut!(rect),
                core::ptr::null_mut(),
                0,
            );
            if hr >= 0 {
                copy_image(
                    buffer,
                    (*buffer)._2d.linear_buffer,
                    (*buffer)._2d.width as LONG,
                    rect.pBits as *const BYTE,
                    rect.Pitch as LONG,
                    (*buffer)._2d.width as DWORD,
                    (*buffer)._2d.height as DWORD,
                );
                IDirect3DSurface9_UnlockRect((*buffer).d3d9_surface.surface);
            }
        }
    }

    if hr >= 0 {
        (*buffer)._2d.locks = (*buffer)._2d.locks.wrapping_add(1);
        *data = (*buffer)._2d.linear_buffer;
        if !max_length.is_null() {
            *max_length = (*buffer)._2d.plane_size;
        }
        if !current_length.is_null() {
            *current_length = (*buffer)._2d.plane_size;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    return hr;
}

unsafe extern "system" fn d3d9_surface_buffer_Unlock(iface: *mut IMFMediaBuffer) -> HRESULT {
    let mut buffer: *mut buffer = impl_from_IMFMediaBuffer(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    if (*buffer)._2d.linear_buffer.is_null() {
        hr = HRESULT_FROM_WIN32!(ERROR_WAS_UNLOCKED);
    } else {
        (*buffer)._2d.locks = (*buffer)._2d.locks.wrapping_sub(1);
        if (*buffer)._2d.locks == 0 {
            let mut rect: D3DLOCKED_RECT = core::mem::zeroed();

            if ({
                hr = IDirect3DSurface9_LockRect(
                    (*buffer).d3d9_surface.surface,
                    core::ptr::addr_of_mut!(rect),
                    core::ptr::null(),
                    0,
                );
                hr
            }) >= 0
            {
                copy_image(
                    buffer,
                    rect.pBits as *mut BYTE,
                    rect.Pitch as LONG,
                    (*buffer)._2d.linear_buffer,
                    (*buffer)._2d.width as LONG,
                    (*buffer)._2d.width as DWORD,
                    (*buffer)._2d.height as DWORD,
                );
                IDirect3DSurface9_UnlockRect((*buffer).d3d9_surface.surface);
            }

            free((*buffer)._2d.linear_buffer as *mut core::ffi::c_void);
            (*buffer)._2d.linear_buffer = core::ptr::null_mut::<BYTE>();
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    return hr;
}

unsafe extern "system" fn d3d9_surface_buffer_SetCurrentLength(iface: *mut IMFMediaBuffer, current_length: DWORD) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMFMediaBuffer(iface);

    (*buffer).current_length = current_length;

    return S_OK;
}

unsafe extern "system" fn memory_2d_buffer_QueryInterface(iface: *mut IMF2DBuffer2, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMF2DBuffer2(iface);
    return IMFMediaBuffer_QueryInterface(core::ptr::addr_of_mut!((*buffer).IMFMediaBuffer_iface), riid, obj);
}

unsafe extern "system" fn memory_2d_buffer_AddRef(iface: *mut IMF2DBuffer2) -> ULONG {
    let buffer: *mut buffer = impl_from_IMF2DBuffer2(iface);
    return IMFMediaBuffer_AddRef(core::ptr::addr_of_mut!((*buffer).IMFMediaBuffer_iface));
}

unsafe extern "system" fn memory_2d_buffer_Release(iface: *mut IMF2DBuffer2) -> ULONG {
    let buffer: *mut buffer = impl_from_IMF2DBuffer2(iface);
    return IMFMediaBuffer_Release(core::ptr::addr_of_mut!((*buffer).IMFMediaBuffer_iface));
}

unsafe extern "C" fn memory_2d_buffer_lock(
    mut buffer: *mut buffer,
    mut scanline0: *mut *mut BYTE,
    mut pitch: *mut LONG,
    mut buffer_start: *mut *mut BYTE,
    mut buffer_length: *mut DWORD,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    if !(*buffer)._2d.linear_buffer.is_null() {
        hr = MF_E_UNEXPECTED;
    } else {
        (*buffer)._2d.locks = (*buffer)._2d.locks.wrapping_add(1);
        *scanline0 = (*buffer)._2d.scanline0;
        *pitch = (*buffer)._2d.pitch as LONG;
        if !buffer_start.is_null() {
            *buffer_start = (*buffer).data;
        }
        if !buffer_length.is_null() {
            *buffer_length = (*buffer).max_length;
        }
    }

    return hr;
}

unsafe extern "system" fn memory_2d_buffer_Lock2D(iface: *mut IMF2DBuffer2, scanline0: *mut *mut BYTE, pitch: *mut LONG) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMF2DBuffer2(iface);
    let mut hr: HRESULT;

    TRACE!("%p, %p, %p.\n", iface, scanline0, pitch);

    if scanline0.is_null() || pitch.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    hr = memory_2d_buffer_lock(
        buffer,
        scanline0,
        pitch,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    LeaveCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    return hr;
}

unsafe extern "system" fn memory_2d_buffer_Unlock2D(iface: *mut IMF2DBuffer2) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMF2DBuffer2(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    if (*buffer)._2d.linear_buffer.is_null() {
        if (*buffer)._2d.locks != 0 {
            (*buffer)._2d.locks = (*buffer)._2d.locks.wrapping_sub(1);
        } else {
            hr = HRESULT_FROM_WIN32!(ERROR_WAS_UNLOCKED);
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    return hr;
}

unsafe extern "system" fn memory_2d_buffer_GetScanline0AndPitch(iface: *mut IMF2DBuffer2, scanline0: *mut *mut BYTE, pitch: *mut LONG) -> HRESULT {
    let mut buffer: *mut buffer = impl_from_IMF2DBuffer2(iface);
    let mut hr: HRESULT = S_OK;

    if scanline0.is_null() || pitch.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    if !(*buffer)._2d.linear_buffer.is_null() || (*buffer)._2d.locks == 0 {
        hr = HRESULT_FROM_WIN32!(ERROR_WAS_UNLOCKED);
    } else {
        *scanline0 = (*buffer)._2d.scanline0;
        *pitch = (*buffer)._2d.pitch as LONG;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    return hr;
}
unsafe extern "system" fn memory_2d_buffer_IsContiguousFormat(iface: *mut IMF2DBuffer2, is_contiguous: *mut BOOL) -> HRESULT {
    if is_contiguous.is_null() {
        return E_POINTER;
    }

    *is_contiguous = (FALSE as i32);

    return S_OK;
}

unsafe extern "system" fn memory_2d_buffer_GetContiguousLength(iface: *mut IMF2DBuffer2, length: *mut DWORD) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMF2DBuffer2(iface);

    TRACE!("%p, %p.\n", iface, length);

    if length.is_null() {
        return E_POINTER;
    }

    *length = (*buffer)._2d.plane_size;

    return S_OK;
}

unsafe extern "system" fn memory_2d_buffer_ContiguousCopyTo(iface: *mut IMF2DBuffer2, dest_buffer: *mut BYTE, dest_length: DWORD) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMF2DBuffer2(iface);
    let mut src_scanline0: *mut BYTE = core::ptr::null_mut();
    let mut src_buffer_start: *mut BYTE = core::ptr::null_mut();
    let mut src_length: DWORD = 0;
    let mut src_pitch: LONG = 0;
    let mut hr: HRESULT;

    TRACE!("%p, %p, %lu.\n", iface, dest_buffer, dest_length);

    if dest_length < (*buffer)._2d.plane_size {
        return E_INVALIDARG;
    }

    hr = IMF2DBuffer2_Lock2DSize(
        iface,
        MF2DBuffer_LockFlags_Read,
        core::ptr::addr_of_mut!(src_scanline0),
        core::ptr::addr_of_mut!(src_pitch),
        core::ptr::addr_of_mut!(src_buffer_start),
        core::ptr::addr_of_mut!(src_length),
    );

    if hr >= 0 {
        if src_pitch < 0 {
            src_pitch = -src_pitch;
        }
        copy_image(
            buffer,
            dest_buffer,
            (*buffer)._2d.width as LONG,
            src_buffer_start,
            src_pitch,
            (*buffer)._2d.width as DWORD,
            (*buffer)._2d.height as DWORD,
        );

        if IMF2DBuffer2_Unlock2D(iface) < 0 {
            WARN!("Couldn't unlock source buffer %p, hr %#lx.\n", iface, hr);
        }
    }

    return S_OK;
}

unsafe extern "system" fn memory_2d_buffer_ContiguousCopyFrom(iface: *mut IMF2DBuffer2, src_buffer: *const BYTE, src_length: DWORD) -> HRESULT {
    let mut buffer: *mut buffer = impl_from_IMF2DBuffer2(iface);
    let mut dst_scanline0: *mut BYTE = core::ptr::null_mut();
    let mut dst_buffer_start: *mut BYTE = core::ptr::null_mut();
    let mut dst_length: DWORD = 0;
    let mut dst_pitch: LONG = 0;
    let mut hr: HRESULT = 0;

    TRACE!("%p, %p, %lu.\n", iface, src_buffer, src_length);

    if src_length < (*buffer)._2d.plane_size {
        return E_INVALIDARG;
    }

    hr = IMF2DBuffer2_Lock2DSize(
        iface,
        MF2DBuffer_LockFlags_Write,
        &raw mut dst_scanline0,
        &raw mut dst_pitch,
        &raw mut dst_buffer_start,
        &raw mut dst_length,
    );

    if hr >= 0 {
        if dst_pitch < 0 {
            dst_pitch = -dst_pitch;
        }
        copy_image(
            buffer,
            dst_buffer_start,
            dst_pitch,
            src_buffer,
            (*buffer)._2d.width as LONG,
            (*buffer)._2d.width as DWORD,
            (*buffer)._2d.height as DWORD,
        );

        if IMF2DBuffer2_Unlock2D(iface) < 0 {
            WARN!("Couldn't unlock destination buffer %p, hr %#lx.\n", iface, hr);
        }
    }

    return hr;
}

unsafe extern "system" fn memory_2d_buffer_Lock2DSize(iface: *mut IMF2DBuffer2, flags: MF2DBuffer_LockFlags, scanline0: *mut *mut BYTE, pitch: *mut LONG, buffer_start: *mut *mut BYTE, buffer_length: *mut DWORD) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMF2DBuffer2(iface);
    let mut hr: HRESULT;

    if scanline0.is_null() || pitch.is_null() || buffer_start.is_null() || buffer_length.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(&raw mut (*buffer).cs);

    hr = memory_2d_buffer_lock(buffer, scanline0, pitch, buffer_start, buffer_length);

    LeaveCriticalSection(&raw mut (*buffer).cs);

    return hr;
}
unsafe extern "system" fn memory_2d_buffer_Copy2DTo(
    iface: *mut IMF2DBuffer2,
    dest_buffer: *mut IMF2DBuffer2,
) -> HRESULT {
    // FIXME("%p, %p.\n", iface, dest_buffer);
    E_NOTIMPL
}

unsafe extern "C" fn d3d9_surface_buffer_lock(
    buffer: *mut buffer,
    flags: MF2DBuffer_LockFlags,
    scanline0: *mut *mut BYTE,
    pitch: *mut LONG,
    buffer_start: *mut *mut BYTE,
    buffer_length: *mut DWORD,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    if !(*buffer)._2d.linear_buffer.is_null() {
        hr = MF_E_UNEXPECTED;
    } else if (*buffer)._2d.locks == 0 {
        hr = IDirect3DSurface9_LockRect(
            (*buffer).d3d9_surface.surface,
            core::ptr::addr_of_mut!((*buffer).d3d9_surface.rect),
            core::ptr::null(),
            0,
        );
    } else if (*buffer)._2d.lock_flags as u32 == MF2DBuffer_LockFlags_Write as u32
        && flags as u32 != MF2DBuffer_LockFlags_Write as u32
    {
        hr = HRESULT_FROM_WIN32!(ERROR_WAS_LOCKED);
    }

    if hr >= 0 {
        if (*buffer)._2d.locks == 0 {
            (*buffer)._2d.lock_flags = flags;
        }
        (*buffer)._2d.locks = (*buffer)._2d.locks.wrapping_add(1);
        *scanline0 = (*buffer).d3d9_surface.rect.pBits as *mut BYTE;
        *pitch = (*buffer).d3d9_surface.rect.Pitch as LONG;
        if !buffer_start.is_null() {
            *buffer_start = *scanline0;
        }
        if !buffer_length.is_null() {
            *buffer_length = ((*buffer).d3d9_surface.rect.Pitch as u32)
                .wrapping_mul((*buffer)._2d.height) as DWORD;
        }
    }

    return hr;
}
unsafe extern "system" fn d3d9_surface_buffer_Lock2D(iface: *mut IMF2DBuffer2, scanline0: *mut *mut BYTE, pitch: *mut LONG) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMF2DBuffer2(iface);
    let mut hr: HRESULT;

    if scanline0.is_null() || pitch.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    hr = d3d9_surface_buffer_lock(buffer, MF2DBuffer_LockFlags_ReadWrite, scanline0, pitch, core::ptr::null_mut(), core::ptr::null_mut());

    LeaveCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    return hr;
}

unsafe extern "system" fn d3d9_surface_buffer_Unlock2D(iface: *mut IMF2DBuffer2) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMF2DBuffer2(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    if (*buffer)._2d.locks != 0 {
        (*buffer)._2d.locks = (*buffer)._2d.locks.wrapping_sub(1);
        if (*buffer)._2d.locks == 0 {
            IDirect3DSurface9_UnlockRect((*buffer).d3d9_surface.surface);
            memset(
                core::ptr::addr_of_mut!((*buffer).d3d9_surface.rect) as *mut core::ffi::c_void,
                0,
                core::mem::size_of::<D3DLOCKED_RECT>(),
            );
            (*buffer)._2d.lock_flags = 0;
        }
    } else {
        hr = HRESULT_FROM_WIN32!(ERROR_WAS_UNLOCKED);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    return hr;
}

unsafe extern "system" fn d3d9_surface_buffer_GetScanline0AndPitch(iface: *mut IMF2DBuffer2, scanline0: *mut *mut BYTE, pitch: *mut LONG) -> HRESULT {
    let mut buffer: *mut buffer = impl_from_IMF2DBuffer2(iface);
    let mut hr: HRESULT = S_OK;

    if scanline0.is_null() || pitch.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    if (*buffer)._2d.locks == 0 {
        *scanline0 = core::ptr::null_mut();
        *pitch = 0;
        hr = HRESULT_FROM_WIN32!(ERROR_WAS_UNLOCKED);
    } else {
        *scanline0 = (*buffer).d3d9_surface.rect.pBits as *mut BYTE;
        *pitch = (*buffer).d3d9_surface.rect.Pitch as LONG;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    return hr;
}
unsafe extern "system" fn d3d9_surface_buffer_Lock2DSize(iface: *mut IMF2DBuffer2, flags: MF2DBuffer_LockFlags, scanline0: *mut *mut BYTE, pitch: *mut LONG, buffer_start: *mut *mut BYTE, buffer_length: *mut DWORD) -> HRESULT {
    let mut buffer: *mut buffer = impl_from_IMF2DBuffer2(iface);
    let mut hr: HRESULT;

    if scanline0.is_null() || pitch.is_null() || buffer_start.is_null() || buffer_length.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(&raw mut (*buffer).cs);

    hr = d3d9_surface_buffer_lock(buffer, flags, scanline0, pitch, buffer_start, buffer_length);

    LeaveCriticalSection(&raw mut (*buffer).cs);

    return hr;
}

unsafe extern "system" fn memory_2d_buffer_gs_QueryInterface(iface: *mut IMFGetService, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMFGetService(iface);
    return IMFMediaBuffer_QueryInterface(core::ptr::addr_of_mut!((*buffer).IMFMediaBuffer_iface), riid, obj);
}

unsafe extern "system" fn memory_2d_buffer_gs_AddRef(iface: *mut IMFGetService) -> ULONG {
    let buffer: *mut buffer = impl_from_IMFGetService(iface);
    return IMFMediaBuffer_AddRef(core::ptr::addr_of_mut!((*buffer).IMFMediaBuffer_iface));
}

unsafe extern "system" fn memory_2d_buffer_gs_Release(iface: *mut IMFGetService) -> ULONG {
    let buffer: *mut buffer = impl_from_IMFGetService(iface);
    return IMFMediaBuffer_Release(core::ptr::addr_of_mut!((*buffer).IMFMediaBuffer_iface));
}
unsafe extern "system" fn memory_2d_buffer_gs_GetService(
    iface: *mut IMFGetService,
    service: REFGUID,
    riid: REFIID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    // TRACE("%p, %s, %s, %p.\n", iface, debugstr_guid(service), debugstr_guid(riid), obj);
    E_NOTIMPL
}

unsafe extern "system" fn d3d9_surface_buffer_gs_GetService(iface: *mut IMFGetService, service: *const GUID, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMFGetService(iface);

    if IsEqualGUID!(service, core::ptr::addr_of!(MR_BUFFER_SERVICE)) != false {
        return IDirect3DSurface9_QueryInterface((*buffer).d3d9_surface.surface, riid, obj);
    }

    return E_NOTIMPL;
}

unsafe extern "system" fn dxgi_1d_2d_buffer_QueryInterface(iface: *mut IMFMediaBuffer, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut buffer: *mut buffer = impl_from_IMFMediaBuffer(iface);

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFMediaBuffer)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *out = core::ptr::addr_of_mut!((*buffer).IMFMediaBuffer_iface) as *mut core::ffi::c_void;
    }
    else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMF2DBuffer2)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IMF2DBuffer)) != 0
    {
        *out = core::ptr::addr_of_mut!((*buffer).IMF2DBuffer2_iface) as *mut core::ffi::c_void;
    }
    else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFDXGIBuffer)) != 0
    {
        *out = core::ptr::addr_of_mut!((*buffer).IMFDXGIBuffer_iface) as *mut core::ffi::c_void;
    }
    else
    {
        WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
        *out = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*out as *mut IUnknown);
    return S_OK;
}

unsafe extern "C" fn dxgi_surface_buffer_create_readback_texture(
    buffer: *mut buffer,
) -> HRESULT {
    let mut texture_desc: D3D11_TEXTURE2D_DESC = core::mem::zeroed();
    let mut device: *mut ID3D11Device = core::ptr::null_mut();
    let mut hr: HRESULT;

    if !(*buffer).dxgi_surface.rb_texture.is_null() {
        return S_OK;
    }

    ID3D11Texture2D_GetDevice((*buffer).dxgi_surface.texture, core::ptr::addr_of_mut!(device));

    ID3D11Texture2D_GetDesc((*buffer).dxgi_surface.texture, core::ptr::addr_of_mut!(texture_desc));
    texture_desc.Usage = D3D11_USAGE_STAGING;
    texture_desc.BindFlags = 0 as UINT;
    texture_desc.CPUAccessFlags = (D3D11_CPU_ACCESS_READ | D3D11_CPU_ACCESS_WRITE) as UINT;
    texture_desc.MiscFlags = 0 as UINT;
    texture_desc.MipLevels = 1 as UINT;
    hr = ID3D11Device_CreateTexture2D(
        device,
        core::ptr::addr_of_mut!(texture_desc),
        core::ptr::null::<D3D11_SUBRESOURCE_DATA>(),
        core::ptr::addr_of_mut!((*buffer).dxgi_surface.rb_texture),
    );
    if hr < 0 {
        WARN!("Failed to create readback texture, hr %#lx.\n", hr);
    }

    ID3D11Device_Release(device);

    return hr;
}

unsafe extern "C" fn dxgi_surface_buffer_map(buffer: *mut buffer, flags: MF2DBuffer_LockFlags) -> HRESULT {
    let mut immediate_context: *mut ID3D11DeviceContext = core::ptr::null_mut();
    let mut device: *mut ID3D11Device = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    hr = dxgi_surface_buffer_create_readback_texture(buffer);
    if hr < 0 as HRESULT {
        return hr;
    }

    ID3D11Texture2D_GetDevice((*buffer).dxgi_surface.texture, core::ptr::addr_of_mut!(device));
    ID3D11Device_GetImmediateContext(device, core::ptr::addr_of_mut!(immediate_context));

    if flags == MF2DBuffer_LockFlags_Read || flags == MF2DBuffer_LockFlags_ReadWrite {
        ID3D11DeviceContext_CopySubresourceRegion(
            immediate_context,
            (*buffer).dxgi_surface.rb_texture as *mut ID3D11Resource,
            0 as UINT,
            0 as UINT,
            0 as UINT,
            0 as UINT,
            (*buffer).dxgi_surface.texture as *mut ID3D11Resource,
            (*buffer).dxgi_surface.sub_resource_idx as UINT,
            core::ptr::null::<D3D11_BOX>(),
        );
    }

    memset(
        core::ptr::addr_of_mut!((*buffer).dxgi_surface.map_desc) as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<D3D11_MAPPED_SUBRESOURCE>() as size_t,
    );
    hr = ID3D11DeviceContext_Map(
        immediate_context,
        (*buffer).dxgi_surface.rb_texture as *mut ID3D11Resource,
        0 as UINT,
        D3D11_MAP_READ_WRITE as D3D11_MAP,
        0 as UINT,
        core::ptr::addr_of_mut!((*buffer).dxgi_surface.map_desc),
    );
    if hr < 0 as HRESULT {
        WARN!("Failed to map readback texture, hr %#lx.\n", hr);
    }

    ID3D11DeviceContext_Release(immediate_context);
    ID3D11Device_Release(device);

    return hr;
}

unsafe extern "C" fn dxgi_surface_buffer_unmap(
    mut buffer: *mut buffer,
    mut flags: MF2DBuffer_LockFlags,
) {
    let mut immediate_context: *mut ID3D11DeviceContext =
        ::core::ptr::null_mut::<ID3D11DeviceContext>();
    let mut device: *mut ID3D11Device = ::core::ptr::null_mut::<ID3D11Device>();
    ID3D11Texture2D_GetDevice((*buffer).dxgi_surface.texture, &raw mut device);
    ID3D11Device_GetImmediateContext(device, &raw mut immediate_context);
    ID3D11DeviceContext_Unmap(
        immediate_context,
        (*buffer).dxgi_surface.rb_texture as *mut ID3D11Resource,
        0 as UINT,
    );
    memset(
        &raw mut (*buffer).dxgi_surface.map_desc as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<D3D11_MAPPED_SUBRESOURCE>() as size_t,
    );
    if flags as ::core::ffi::c_uint
        == MF2DBuffer_LockFlags_Write as ::core::ffi::c_int as ::core::ffi::c_uint
        || flags as ::core::ffi::c_uint
            == MF2DBuffer_LockFlags_ReadWrite as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        ID3D11DeviceContext_CopySubresourceRegion(
            immediate_context,
            (*buffer).dxgi_surface.texture as *mut ID3D11Resource,
            (*buffer).dxgi_surface.sub_resource_idx as UINT,
            0 as UINT,
            0 as UINT,
            0 as UINT,
            (*buffer).dxgi_surface.rb_texture as *mut ID3D11Resource,
            0 as UINT,
            ::core::ptr::null::<D3D11_BOX>(),
        );
    }
    ID3D11DeviceContext_Release(immediate_context);
    ID3D11Device_Release(device);
}

unsafe extern "system" fn dxgi_surface_buffer_Lock(iface: *mut IMFMediaBuffer, data: *mut *mut BYTE, max_length: *mut DWORD, current_length: *mut DWORD) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMFMediaBuffer(iface);
    let mut hr: HRESULT = S_OK;

    if data.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    if (*buffer)._2d.linear_buffer.is_null() && (*buffer)._2d.locks != 0 {
        hr = MF_E_INVALIDREQUEST;
    } else if (*buffer)._2d.linear_buffer.is_null() {
        (*buffer)._2d.linear_buffer = malloc((*buffer)._2d.plane_size as size_t) as *mut BYTE;
        if (*buffer)._2d.linear_buffer.is_null() {
            hr = E_OUTOFMEMORY;
        }

        if hr >= 0 {
            hr = dxgi_surface_buffer_map(buffer, MF2DBuffer_LockFlags_ReadWrite);
            if hr >= 0 {
                copy_image(
                    buffer,
                    (*buffer)._2d.linear_buffer,
                    (*buffer)._2d.width as LONG,
                    (*buffer).dxgi_surface.map_desc.pData as *const BYTE,
                    (*buffer).dxgi_surface.map_desc.RowPitch as LONG,
                    (*buffer)._2d.width as DWORD,
                    (*buffer)._2d.height as DWORD,
                );
            }
        }
    }

    if hr >= 0 {
        (*buffer)._2d.locks = (*buffer)._2d.locks.wrapping_add(1);
        *data = (*buffer)._2d.linear_buffer;
        if !max_length.is_null() {
            *max_length = (*buffer)._2d.plane_size;
        }
        if !current_length.is_null() {
            *current_length = (*buffer)._2d.plane_size;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    return hr;
}
unsafe extern "system" fn dxgi_surface_buffer_Unlock(iface: *mut IMFMediaBuffer) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMFMediaBuffer(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    if (*buffer)._2d.linear_buffer.is_null() {
        hr = HRESULT_FROM_WIN32!(ERROR_WAS_UNLOCKED);
    } else {
        (*buffer)._2d.locks = (*buffer)._2d.locks.wrapping_sub(1);
        if (*buffer)._2d.locks == 0 {
            copy_image(
                buffer,
                (*buffer).dxgi_surface.map_desc.pData as *mut BYTE,
                (*buffer).dxgi_surface.map_desc.RowPitch as LONG,
                (*buffer)._2d.linear_buffer,
                (*buffer)._2d.width as LONG,
                (*buffer)._2d.width as DWORD,
                (*buffer)._2d.height as DWORD,
            );
            dxgi_surface_buffer_unmap(buffer, MF2DBuffer_LockFlags_ReadWrite);

            free((*buffer)._2d.linear_buffer as *mut core::ffi::c_void);
            (*buffer)._2d.linear_buffer = core::ptr::null_mut::<BYTE>();
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    return hr;
}
unsafe extern "system" fn dxgi_surface_buffer_SetCurrentLength(iface: *mut IMFMediaBuffer, current_length: DWORD) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMFMediaBuffer(iface);

    (*buffer).current_length = current_length;

    return S_OK;
}

unsafe extern "C" fn dxgi_surface_buffer_lock(
    buffer: *mut buffer,
    flags: MF2DBuffer_LockFlags,
    scanline0: *mut *mut BYTE,
    pitch: *mut LONG,
    buffer_start: *mut *mut BYTE,
    buffer_length: *mut DWORD,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    if !(*buffer)._2d.linear_buffer.is_null() {
        hr = MF_E_UNEXPECTED;
    } else if (*buffer)._2d.locks == 0 {
        hr = dxgi_surface_buffer_map(buffer, flags);
    } else if (*buffer)._2d.lock_flags as ::core::ffi::c_uint
        == MF2DBuffer_LockFlags_Write as ::core::ffi::c_int as ::core::ffi::c_uint
        && flags as ::core::ffi::c_uint
            != MF2DBuffer_LockFlags_Write as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        hr = HRESULT_FROM_WIN32!(ERROR_WAS_LOCKED);
    }

    if hr >= 0 {
        if (*buffer)._2d.locks == 0 {
            (*buffer)._2d.lock_flags = flags;
        } else {
            (*buffer)._2d.lock_flags =
                ::core::mem::transmute::<::core::ffi::c_uint, MF2DBuffer_LockFlags>(
                    (*buffer)._2d.lock_flags as ::core::ffi::c_uint | flags as ::core::ffi::c_uint,
                );
        }
        (*buffer)._2d.locks = (*buffer)._2d.locks.wrapping_add(1);
        *scanline0 = (*buffer).dxgi_surface.map_desc.pData as *mut BYTE;
        *pitch = (*buffer).dxgi_surface.map_desc.RowPitch as LONG;
        if !buffer_start.is_null() {
            *buffer_start = *scanline0;
        }
        if !buffer_length.is_null() {
            *buffer_length = (*buffer).dxgi_surface.map_desc.DepthPitch as DWORD;
        }
    }

    return hr;
}

unsafe extern "system" fn dxgi_surface_buffer_Lock2D(iface: *mut IMF2DBuffer2, scanline0: *mut *mut BYTE, pitch: *mut LONG) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMF2DBuffer2(iface);
    let mut hr: HRESULT = 0;

    // TRACE("%p, %p, %p.\n", iface, scanline0, pitch);

    if scanline0.is_null() || pitch.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    hr = dxgi_surface_buffer_lock(buffer, MF2DBuffer_LockFlags_ReadWrite, scanline0, pitch, core::ptr::null_mut(), core::ptr::null_mut());

    LeaveCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    return hr;
}

unsafe extern "system" fn dxgi_surface_buffer_Unlock2D(iface: *mut IMF2DBuffer2) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMF2DBuffer2(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    if (*buffer)._2d.locks != 0 {
        (*buffer)._2d.locks = (*buffer)._2d.locks.wrapping_sub(1);
        if (*buffer)._2d.locks == 0 {
            dxgi_surface_buffer_unmap(buffer, (*buffer)._2d.lock_flags);
            (*buffer)._2d.lock_flags = 0;
        }
    } else {
        hr = HRESULT_FROM_WIN32!(ERROR_WAS_UNLOCKED);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    return hr;
}
unsafe extern "system" fn dxgi_surface_buffer_GetScanline0AndPitch(iface: *mut IMF2DBuffer2, scanline0: *mut *mut BYTE, pitch: *mut LONG) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMF2DBuffer2(iface);
    let mut hr: HRESULT = S_OK;

    if scanline0.is_null() || pitch.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    if (*buffer)._2d.locks == 0 {
        *scanline0 = core::ptr::null_mut();
        *pitch = 0;
        hr = HRESULT_FROM_WIN32!(ERROR_WAS_UNLOCKED);
    } else {
        *scanline0 = (*buffer).dxgi_surface.map_desc.pData as *mut BYTE;
        *pitch = (*buffer).dxgi_surface.map_desc.RowPitch as LONG;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    return hr;
}

unsafe extern "system" fn dxgi_surface_buffer_Lock2DSize(iface: *mut IMF2DBuffer2, flags: MF2DBuffer_LockFlags, scanline0: *mut *mut BYTE, pitch: *mut LONG, buffer_start: *mut *mut BYTE, buffer_length: *mut DWORD) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMF2DBuffer2(iface);
    let mut hr: HRESULT;

    if scanline0.is_null() || pitch.is_null() || buffer_start.is_null() || buffer_length.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    hr = dxgi_surface_buffer_lock(buffer, flags, scanline0, pitch, buffer_start, buffer_length);

    LeaveCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    return hr;
}

unsafe extern "system" fn dxgi_buffer_QueryInterface(iface: *mut IMFDXGIBuffer, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMFDXGIBuffer(iface);
    return IMFMediaBuffer_QueryInterface(core::ptr::addr_of_mut!((*buffer).IMFMediaBuffer_iface), riid, obj);
}

unsafe extern "system" fn dxgi_buffer_AddRef(iface: *mut IMFDXGIBuffer) -> ULONG {
    let buffer: *mut buffer = impl_from_IMFDXGIBuffer(iface);
    return IMFMediaBuffer_AddRef(core::ptr::addr_of_mut!((*buffer).IMFMediaBuffer_iface));
}
unsafe extern "system" fn dxgi_buffer_Release(iface: *mut IMFDXGIBuffer) -> ULONG {
    let buffer: *mut buffer = impl_from_IMFDXGIBuffer(iface);
    return IMFMediaBuffer_Release(core::ptr::addr_of_mut!((*buffer).IMFMediaBuffer_iface));
}

unsafe extern "system" fn dxgi_buffer_GetResource(iface: *mut IMFDXGIBuffer, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMFDXGIBuffer(iface);

    return ID3D11Texture2D_QueryInterface((*buffer).dxgi_surface.texture, riid, obj);
}

unsafe extern "system" fn dxgi_buffer_GetSubresourceIndex(iface: *mut IMFDXGIBuffer, index: *mut UINT) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMFDXGIBuffer(iface);

    if index.is_null() {
        return E_POINTER;
    }

    *index = (*buffer).dxgi_surface.sub_resource_idx;

    return S_OK;
}
unsafe extern "system" fn dxgi_buffer_GetUnknown(iface: *mut IMFDXGIBuffer, guid: REFIID, riid: REFIID, object: *mut *mut core::ffi::c_void) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMFDXGIBuffer(iface);
    let mut hr: HRESULT;

    hr = attributes_GetUnknown(core::ptr::addr_of_mut!((*buffer).dxgi_surface.attributes), guid, riid, object);

    return if hr == MF_E_ATTRIBUTENOTFOUND { MF_E_NOT_FOUND } else { hr };
}

unsafe extern "system" fn dxgi_buffer_SetUnknown(
    mut iface: *mut IMFDXGIBuffer,
    guid: *const IID,
    mut data: *mut IUnknown,
) -> HRESULT {
    let mut buffer: *mut buffer = impl_from_IMFDXGIBuffer(iface);
    let mut hr: HRESULT = S_OK;
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE) as i32;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/buffer.c\0"
                as *const u8 as *const i8,
            b"dxgi_buffer_SetUnknown\0" as *const u8 as *const i8,
            1315 as i32,
            b"%p, %s, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_guid(guid as *const _GUID),
            data,
        );
    }
    EnterCriticalSection(core::ptr::addr_of_mut!((*buffer).dxgi_surface.attributes.cs));
    if !data.is_null() {
        if attributes_GetItem(
            core::ptr::addr_of_mut!((*buffer).dxgi_surface.attributes),
            guid as *const GUID,
            core::ptr::null_mut(),
        ) >= 0
        {
            hr = HRESULT_FROM_WIN32!(ERROR_OBJECT_ALREADY_EXISTS);
        } else {
            hr = attributes_SetUnknown(
                core::ptr::addr_of_mut!((*buffer).dxgi_surface.attributes),
                guid as *const GUID,
                data,
            );
        }
    } else {
        attributes_DeleteItem(
            core::ptr::addr_of_mut!((*buffer).dxgi_surface.attributes),
            guid as *const GUID,
        );
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*buffer).dxgi_surface.attributes.cs));
    return hr;
}

unsafe extern "C" fn d3d12_surface_buffer_copy_transfer_resource(
    mut buffer: *mut buffer,
    mut flags: MF2DBuffer_LockFlags,
) -> HRESULT {
    let mut current_alloc: *mut d3d12_list_allocator = (&raw mut (*buffer).d3d12_surface.alloc
        as *mut d3d12_list_allocator)
        .offset((*buffer).d3d12_surface.alloc_index as isize)
        as *mut d3d12_list_allocator;
    let mut pre_barrier: D3D12_RESOURCE_BARRIER = D3D12_RESOURCE_BARRIER {
        Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
        __bindgen_anon_1: D3D12_RESOURCE_BARRIER__bindgen_ty_1 { Transition: D3D12_RESOURCE_TRANSITION_BARRIER {
                pResource: ::core::ptr::null_mut::<ID3D12Resource>(),
                Subresource: 0,
                StateBefore: D3D12_RESOURCE_STATE_COMMON,
                StateAfter: D3D12_RESOURCE_STATE_COMMON,
            },
        },
    };
    let mut post_barrier: D3D12_RESOURCE_BARRIER = D3D12_RESOURCE_BARRIER {
        Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
        __bindgen_anon_1: D3D12_RESOURCE_BARRIER__bindgen_ty_1 { Transition: D3D12_RESOURCE_TRANSITION_BARRIER {
                pResource: ::core::ptr::null_mut::<ID3D12Resource>(),
                Subresource: 0,
                StateBefore: D3D12_RESOURCE_STATE_COMMON,
                StateAfter: D3D12_RESOURCE_STATE_COMMON,
            },
        },
    };
    let mut res_loc: D3D12_TEXTURE_COPY_LOCATION = D3D12_TEXTURE_COPY_LOCATION {
        pResource: ::core::ptr::null_mut::<ID3D12Resource>(),
        Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
        __bindgen_anon_1: D3D12_TEXTURE_COPY_LOCATION__bindgen_ty_1 { PlacedFootprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT {
                Offset: 0,
                Footprint: D3D12_SUBRESOURCE_FOOTPRINT {
                    Format: DXGI_FORMAT_UNKNOWN,
                    Width: 0,
                    Height: 0,
                    Depth: 0,
                    RowPitch: 0,
                },
            },
        },
    };
    let mut tx_loc: D3D12_TEXTURE_COPY_LOCATION = D3D12_TEXTURE_COPY_LOCATION {
        pResource: ::core::ptr::null_mut::<ID3D12Resource>(),
        Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
        __bindgen_anon_1: D3D12_TEXTURE_COPY_LOCATION__bindgen_ty_1 { PlacedFootprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT {
                Offset: 0,
                Footprint: D3D12_SUBRESOURCE_FOOTPRINT {
                    Format: DXGI_FORMAT_UNKNOWN,
                    Width: 0,
                    Height: 0,
                    Depth: 0,
                    RowPitch: 0,
                },
            },
        },
    };
    let mut hr: HRESULT = 0;
    if (*buffer).d3d12_surface.queue.is_null() {
        let mut queue_desc: D3D12_COMMAND_QUEUE_DESC = D3D12_COMMAND_QUEUE_DESC {
            Type: D3D12_COMMAND_LIST_TYPE_COPY,
            Priority: 0,
            Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
            NodeMask: 0,
        };
        hr = ID3D12Device_CreateCommandQueue(
            (*buffer).d3d12_surface.device,
            &raw mut queue_desc,
            &raw const IID_ID3D12CommandQueue,
            &raw mut (*buffer).d3d12_surface.queue as *mut *mut ::core::ffi::c_void,
        );
        if hr < 0 {
            return hr;
        }
    }
    if (*buffer).d3d12_surface.fence.is_null() {
        hr = ID3D12Device_CreateFence(
            (*buffer).d3d12_surface.device,
            0,
            0,
            &raw const IID_ID3D12Fence,
            &raw mut (*buffer).d3d12_surface.fence as *mut *mut ::core::ffi::c_void,
        );
        if hr < 0 {
            return hr;
        }
    }
    if ID3D12Fence_GetCompletedValue((*buffer).d3d12_surface.fence)
        >= (*buffer).d3d12_surface.alloc
            [((*buffer).d3d12_surface.alloc_index == 0) as ::core::ffi::c_int as usize]
            .free_value
    {
        (*buffer).d3d12_surface.alloc_index =
            ((*buffer).d3d12_surface.alloc_index == 0) as ::core::ffi::c_int as size_t;
        current_alloc = (&raw mut (*buffer).d3d12_surface.alloc as *mut d3d12_list_allocator)
            .offset((*buffer).d3d12_surface.alloc_index as isize)
            as *mut d3d12_list_allocator as *mut d3d12_list_allocator;
        if (*current_alloc).allocator.is_null() {
            hr = ID3D12Device_CreateCommandAllocator(
                (*buffer).d3d12_surface.device,
                D3D12_COMMAND_LIST_TYPE_COPY,
                &raw const IID_ID3D12CommandAllocator,
                &raw mut (*current_alloc).allocator as *mut *mut ::core::ffi::c_void,
            );
            if hr < 0 {
                return hr;
            }
        } else {
            hr = ID3D12CommandAllocator_Reset((*current_alloc).allocator);
            if hr < 0 {
                return hr;
            }
        }
    }
    if (*buffer).d3d12_surface.list.is_null() {
        hr = ID3D12Device_CreateCommandList(
            (*buffer).d3d12_surface.device,
            0,
            D3D12_COMMAND_LIST_TYPE_COPY,
            (*current_alloc).allocator,
            ::core::ptr::null_mut::<ID3D12PipelineState>(),
            &raw const IID_ID3D12GraphicsCommandList,
            &raw mut (*buffer).d3d12_surface.list as *mut *mut ::core::ffi::c_void,
        );
        if hr < 0 {
            return hr;
        }
    } else {
        hr = ID3D12GraphicsCommandList_Reset(
            (*buffer).d3d12_surface.list,
            (*current_alloc).allocator,
            ::core::ptr::null_mut::<ID3D12PipelineState>(),
        );
        if hr < 0 {
            return hr;
        }
    }
    res_loc.pResource = (*buffer).d3d12_surface.resource;
    res_loc.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
    res_loc.__bindgen_anon_1.SubresourceIndex = (*buffer).dxgi_surface.sub_resource_idx as UINT;
    tx_loc.pResource = (*buffer).d3d12_surface.transfer;
    tx_loc.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT;
    tx_loc.__bindgen_anon_1.PlacedFootprint = (*buffer).d3d12_surface.layout;
    post_barrier.Type = D3D12_RESOURCE_BARRIER_TYPE_TRANSITION;
    pre_barrier.Type = post_barrier.Type;
    post_barrier.Flags = 0;
    pre_barrier.Flags = post_barrier.Flags;
    post_barrier.__bindgen_anon_1.Transition.pResource = (*buffer).d3d12_surface.resource;
    pre_barrier.__bindgen_anon_1.Transition.pResource =
        post_barrier.__bindgen_anon_1.Transition.pResource;
    post_barrier.__bindgen_anon_1.Transition.Subresource =
        (*buffer).dxgi_surface.sub_resource_idx as UINT;
    pre_barrier.__bindgen_anon_1.Transition.Subresource =
        post_barrier.__bindgen_anon_1.Transition.Subresource;
    post_barrier.__bindgen_anon_1.Transition.StateAfter = D3D12_RESOURCE_STATE_COMMON;
    pre_barrier.__bindgen_anon_1.Transition.StateBefore =
        post_barrier.__bindgen_anon_1.Transition.StateAfter;
    post_barrier.__bindgen_anon_1.Transition.StateBefore = if (flags as ::core::ffi::c_uint)
        == (MF2DBuffer_LockFlags_Write as ::core::ffi::c_uint)
    {
        D3D12_RESOURCE_STATE_COPY_DEST
    } else {
        D3D12_RESOURCE_STATE_COPY_SOURCE
    };
    pre_barrier.__bindgen_anon_1.Transition.StateAfter =
        post_barrier.__bindgen_anon_1.Transition.StateBefore;
    ID3D12GraphicsCommandList_ResourceBarrier(
        (*buffer).d3d12_surface.list,
        1,
        &raw mut pre_barrier,
    );
    ID3D12GraphicsCommandList_CopyTextureRegion(
        (*buffer).d3d12_surface.list,
        if (flags as ::core::ffi::c_uint) == (MF2DBuffer_LockFlags_Write as ::core::ffi::c_uint) {
            &raw mut res_loc
        } else {
            &raw mut tx_loc
        },
        0,
        0,
        0,
        if (flags as ::core::ffi::c_uint) == (MF2DBuffer_LockFlags_Write as ::core::ffi::c_uint) {
            &raw mut tx_loc
        } else {
            &raw mut res_loc
        },
        ::core::ptr::null::<D3D12_BOX>(),
    );
    ID3D12GraphicsCommandList_ResourceBarrier(
        (*buffer).d3d12_surface.list,
        1,
        &raw mut post_barrier,
    );
    hr = ID3D12GraphicsCommandList_Close((*buffer).d3d12_surface.list);
    if hr < 0 {
        return hr;
    }
    if (flags as ::core::ffi::c_uint) != (MF2DBuffer_LockFlags_Write as ::core::ffi::c_uint) {
        IMFD3D12SynchronizationObjectCommands_EnqueueResourceReadyWait(
            (*buffer).d3d12_surface.sync_cmd,
            (*buffer).d3d12_surface.queue,
        );
    }
    ID3D12CommandQueue_ExecuteCommandLists(
        (*buffer).d3d12_surface.queue,
        1,
        &raw mut (*buffer).d3d12_surface.list as *mut *mut ID3D12CommandList,
    );
    (*buffer).d3d12_surface.fence_value = (*buffer).d3d12_surface.fence_value.wrapping_add(1);
    (*current_alloc).free_value = (*buffer).d3d12_surface.fence_value;
    ID3D12CommandQueue_Signal(
        (*buffer).d3d12_surface.queue,
        (*buffer).d3d12_surface.fence,
        (*buffer).d3d12_surface.fence_value,
    );
    if (flags as ::core::ffi::c_uint) == (MF2DBuffer_LockFlags_Write as ::core::ffi::c_uint) {
        IMFD3D12SynchronizationObjectCommands_EnqueueResourceReady(
            (*buffer).d3d12_surface.sync_cmd,
            (*buffer).d3d12_surface.queue,
        );
    } else {
        IMFD3D12SynchronizationObjectCommands_EnqueueResourceRelease(
            (*buffer).d3d12_surface.sync_cmd,
            (*buffer).d3d12_surface.queue,
        );
        ID3D12Fence_SetEventOnCompletion(
            (*buffer).d3d12_surface.fence,
            (*buffer).d3d12_surface.fence_value,
            ::core::ptr::null_mut(),
        );
    }
    return S_OK;
}

unsafe extern "C" fn d3d12_surface_buffer_create_transfer_resource(buffer: *mut buffer, resource: *mut *mut ID3D12Resource, flags: MF2DBuffer_LockFlags) -> HRESULT {
    let mut heap_prop: D3D12_HEAP_PROPERTIES = D3D12_HEAP_PROPERTIES {
        Type: (if flags == MF2DBuffer_LockFlags_Write {
            D3D12_HEAP_TYPE_UPLOAD
        } else {
            D3D12_HEAP_TYPE_READBACK
        }),
        CPUPageProperty: 0,
        MemoryPoolPreference: 0,
        CreationNodeMask: 0,
        VisibleNodeMask: 0,
    };
    let mut desc: D3D12_RESOURCE_DESC = core::mem::zeroed();
    let mut footprint: D3D12_SUBRESOURCE_FOOTPRINT = (*buffer).d3d12_surface.layout.Footprint;

    memset(&mut desc as *mut D3D12_RESOURCE_DESC as *mut core::ffi::c_void, 0, core::mem::size_of::<D3D12_RESOURCE_DESC>());
    desc.Dimension = D3D12_RESOURCE_DIMENSION_BUFFER;
    desc.Alignment = 0;
    desc.Width = (footprint.RowPitch * footprint.Height) as _;
    desc.Height = 1;
    desc.DepthOrArraySize = 1;
    desc.MipLevels = 1;
    desc.Format = DXGI_FORMAT_UNKNOWN;
    desc.SampleDesc.Count = 1;
    desc.SampleDesc.Quality = 0;
    desc.Layout = D3D12_TEXTURE_LAYOUT_ROW_MAJOR;
    desc.Flags = D3D12_RESOURCE_FLAG_NONE;
    return ID3D12Device_CreateCommittedResource(
        (*buffer).d3d12_surface.device,
        &mut heap_prop as *mut D3D12_HEAP_PROPERTIES,
        D3D12_HEAP_FLAG_NONE,
        &mut desc as *mut D3D12_RESOURCE_DESC,
        if flags == MF2DBuffer_LockFlags_Write {
            D3D12_RESOURCE_STATE_GENERIC_READ
        } else {
            D3D12_RESOURCE_STATE_COPY_DEST
        },
        core::ptr::null_mut(),
        &IID_ID3D12Resource as *const GUID,
        resource as *mut *mut core::ffi::c_void,
    );
}

unsafe extern "C" fn d3d12_surface_buffer_map(
    buffer: *mut buffer,
    flags: MF2DBuffer_LockFlags,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut empty_range: D3D12_RANGE = D3D12_RANGE {
        Begin: 0,
        End: 0,
    };

    if (*buffer).d3d12_surface.transfer.is_null() {
        let transfer: *mut *mut ID3D12Resource = if flags as ::core::ffi::c_uint
            == MF2DBuffer_LockFlags_Write as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            &raw mut (*buffer).d3d12_surface.upload
        } else {
            &raw mut (*buffer).d3d12_surface.readback
        };
        if (*transfer).is_null() {
            hr = d3d12_surface_buffer_create_transfer_resource(buffer, transfer, flags);
        }
        (*buffer).d3d12_surface.transfer = *transfer;

        if hr >= 0
            && flags as ::core::ffi::c_uint
                != MF2DBuffer_LockFlags_Write as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            hr = d3d12_surface_buffer_copy_transfer_resource(buffer, flags);
        }
    }

    if hr >= 0 {
        hr = ID3D12Resource_Map(
            (*buffer).d3d12_surface.transfer,
            0,
            if flags as ::core::ffi::c_uint
                == MF2DBuffer_LockFlags_Write as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                &raw mut empty_range
            } else {
                ::core::ptr::null_mut::<D3D12_RANGE>()
            },
            &raw mut (*buffer).d3d12_surface.data,
        );
    }

    if hr < 0 {
        (*buffer).d3d12_surface.transfer = ::core::ptr::null_mut::<ID3D12Resource>();
    }
    return hr;
}

unsafe extern "C" fn d3d12_surface_buffer_unmap(
    mut buffer: *mut buffer,
    mut flags: MF2DBuffer_LockFlags,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut empty_range: D3D12_RANGE = D3D12_RANGE {
        Begin: 0,
        End: 0,
    };
    ID3D12Resource_Unmap(
        (*buffer).d3d12_surface.transfer,
        0,
        if flags as ::core::ffi::c_uint
            == MF2DBuffer_LockFlags_Write as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*buffer).d3d12_surface.keep_transfer == 0
        {
            ::core::ptr::null_mut::<D3D12_RANGE>()
        } else {
            &raw mut empty_range
        },
    );
    if flags as ::core::ffi::c_uint
        == MF2DBuffer_LockFlags_Read as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*buffer).d3d12_surface.keep_transfer = (FALSE as i32);
    } else if flags as ::core::ffi::c_uint
        == MF2DBuffer_LockFlags_ReadWrite as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*buffer).d3d12_surface.keep_transfer = (TRUE as i32);
    }
    if (*buffer).d3d12_surface.keep_transfer != 0 {
        hr = E_INVALIDARG;
    } else {
        if flags as ::core::ffi::c_uint
            == MF2DBuffer_LockFlags_Write as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            hr = d3d12_surface_buffer_copy_transfer_resource(buffer, flags);
        }
        (*buffer).d3d12_surface.transfer = ::core::ptr::null_mut::<ID3D12Resource>();
    }
    return hr;
}

unsafe extern "system" fn d3d12_surface_buffer_Lock(iface: *mut IMFMediaBuffer, data: *mut *mut BYTE, max_length: *mut DWORD, current_length: *mut DWORD) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMFMediaBuffer(iface);
    let mut hr: HRESULT = S_OK;

    if data.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    if (*buffer)._2d.linear_buffer.is_null() && (*buffer)._2d.locks != 0 {
        hr = MF_E_INVALIDREQUEST;
    } else if (*buffer)._2d.linear_buffer.is_null() {
        (*buffer)._2d.linear_buffer = malloc((*buffer)._2d.plane_size as size_t) as *mut BYTE;
        if (*buffer)._2d.linear_buffer.is_null() {
            hr = E_OUTOFMEMORY;
        }

        if hr >= 0 {
            hr = d3d12_surface_buffer_map(buffer, MF2DBuffer_LockFlags_Read);
            if hr >= 0 {
                copy_image(
                    buffer,
                    (*buffer)._2d.linear_buffer,
                    (*buffer)._2d.width as LONG,
                    (*buffer).d3d12_surface.data as *const BYTE,
                    (*buffer).d3d12_surface.layout.Footprint.RowPitch as LONG,
                    (*buffer)._2d.width as DWORD,
                    (*buffer)._2d.height as DWORD,
                );
                d3d12_surface_buffer_unmap(buffer, MF2DBuffer_LockFlags_Read);
            }
        }
    }

    if hr >= 0 {
        (*buffer)._2d.locks = (*buffer)._2d.locks.wrapping_add(1);
        *data = (*buffer)._2d.linear_buffer;
        if !max_length.is_null() {
            *max_length = (*buffer)._2d.plane_size;
        }
        if !current_length.is_null() {
            *current_length = (*buffer)._2d.plane_size;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    return hr;
}

unsafe extern "system" fn d3d12_surface_buffer_Unlock(iface: *mut IMFMediaBuffer) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMFMediaBuffer(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    if (*buffer)._2d.linear_buffer.is_null() {
        hr = HRESULT_FROM_WIN32!(ERROR_WAS_UNLOCKED);
    } else {
        (*buffer)._2d.locks = (*buffer)._2d.locks.wrapping_sub(1);
        if (*buffer)._2d.locks == 0 {
            free((*buffer)._2d.linear_buffer as *mut core::ffi::c_void);
            (*buffer)._2d.linear_buffer = core::ptr::null_mut();
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    return hr;
}

unsafe extern "system" fn d3d12_surface_buffer_GetCurrentLength(iface: *mut IMFMediaBuffer, current_length: *mut DWORD) -> HRESULT {
    if current_length.is_null() {
        return E_INVALIDARG;
    }

    *current_length = 0;
    return S_OK;
}
unsafe extern "system" fn d3d12_surface_buffer_SetCurrentLength(
    iface: *mut IMFMediaBuffer,
    current_length: DWORD,
) -> HRESULT {
    // TRACE("%p, %lu.\n", iface, current_length);
    S_OK
}

unsafe extern "C" fn d3d12_surface_buffer_lock(buffer: *mut buffer, flags: MF2DBuffer_LockFlags, scanline0: *mut *mut BYTE, pitch: *mut LONG, buffer_start: *mut *mut BYTE, buffer_length: *mut DWORD) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    if !(*buffer)._2d.linear_buffer.is_null() {
        hr = MF_E_UNEXPECTED;
    } else if (*buffer)._2d.locks == 0 {
        hr = d3d12_surface_buffer_map(buffer, flags);
    } else if (*buffer)._2d.lock_flags == MF2DBuffer_LockFlags_Write && flags != MF2DBuffer_LockFlags_Write {
        hr = HRESULT_FROM_WIN32!(ERROR_WAS_LOCKED);
    }

    if hr >= 0 {
        if (*buffer)._2d.locks == 0 {
            (*buffer)._2d.lock_flags = flags;
        }
        (*buffer)._2d.locks = (*buffer)._2d.locks.wrapping_add(1);
        *scanline0 = (*buffer).d3d12_surface.data as *mut BYTE;
        *pitch = (*buffer).d3d12_surface.layout.Footprint.RowPitch as LONG;
        if !buffer_start.is_null() {
            *buffer_start = *scanline0;
        }
        if !buffer_length.is_null() {
            *buffer_length = (*pitch as DWORD).wrapping_mul((*buffer)._2d.height);
        }
    }

    return hr;
}
unsafe extern "system" fn d3d12_surface_buffer_Lock2D(iface: *mut IMF2DBuffer2, scanline0: *mut *mut BYTE, pitch: *mut LONG) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMF2DBuffer2(iface);
    let mut hr: HRESULT;

    if scanline0.is_null() || pitch.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(&raw mut (*buffer).cs);

    hr = d3d12_surface_buffer_lock(buffer, MF2DBuffer_LockFlags_ReadWrite, scanline0, pitch, core::ptr::null_mut::<*mut BYTE>(), core::ptr::null_mut::<DWORD>());

    LeaveCriticalSection(&raw mut (*buffer).cs);

    return hr;
}

unsafe extern "system" fn d3d12_surface_buffer_Unlock2D(iface: *mut IMF2DBuffer2) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMF2DBuffer2(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    if (*buffer)._2d.locks != 0 {
        (*buffer)._2d.locks = (*buffer)._2d.locks.wrapping_sub(1);
        if (*buffer)._2d.locks == 0 {
            hr = d3d12_surface_buffer_unmap(buffer, (*buffer)._2d.lock_flags);
            (*buffer)._2d.lock_flags = 0;
        }
    } else {
        hr = HRESULT_FROM_WIN32!(ERROR_WAS_UNLOCKED);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    return hr;
}

unsafe extern "system" fn d3d12_surface_buffer_GetScanline0AndPitch(iface: *mut IMF2DBuffer2, scanline0: *mut *mut BYTE, pitch: *mut LONG) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMF2DBuffer2(iface);
    let mut hr: HRESULT = S_OK;

    if scanline0.is_null() || pitch.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    if (*buffer)._2d.locks == 0 {
        *scanline0 = core::ptr::null_mut();
        *pitch = 0;
        hr = HRESULT_FROM_WIN32!(ERROR_WAS_UNLOCKED);
    } else {
        *scanline0 = (*buffer).d3d12_surface.data as *mut BYTE;
        *pitch = (*buffer).d3d12_surface.layout.Footprint.RowPitch as LONG;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    return hr;
}

unsafe extern "system" fn d3d12_surface_buffer_Lock2DSize(
    iface: *mut IMF2DBuffer2,
    flags: MF2DBuffer_LockFlags,
    scanline0: *mut *mut BYTE,
    pitch: *mut LONG,
    buffer_start: *mut *mut BYTE,
    buffer_length: *mut DWORD,
) -> HRESULT {
    let buffer: *mut buffer = impl_from_IMF2DBuffer2(iface);
    let mut hr: HRESULT = S_OK;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE) as i32;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/buffer.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"d3d12_surface_buffer_Lock2DSize\0" as *const u8 as *const ::core::ffi::c_char,
            1742 as ::core::ffi::c_int,
            b"%p, %#x, %p, %p, %p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            flags as ::core::ffi::c_uint,
            scanline0,
            pitch,
            buffer_start,
            buffer_length,
        );
    }
    if scanline0.is_null() || pitch.is_null() || buffer_start.is_null() || buffer_length.is_null() {
        return E_POINTER;
    }
    EnterCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));
    hr = d3d12_surface_buffer_lock(buffer, flags, scanline0, pitch, buffer_start, buffer_length);
    LeaveCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));
    return hr;
}

unsafe extern "system" fn d3d12_surface_buffer_GetResource(iface: *mut IMFDXGIBuffer, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut buffer: *mut buffer = impl_from_IMFDXGIBuffer(iface);
    if (*__wine_dbch___default).flags as core::ffi::c_int & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE) as i32;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/buffer.c\0".as_ptr() as *const core::ffi::c_char,
            b"d3d12_surface_buffer_GetResource\0".as_ptr() as *const core::ffi::c_char,
            1760 as core::ffi::c_int,
            b"%p, %s, %p.\n\0".as_ptr() as *const core::ffi::c_char,
            iface,
            debugstr_guid(riid),
            obj,
        );
    }
    return ID3D12Resource_QueryInterface((*buffer).d3d12_surface.resource, riid, obj);
}

unsafe extern "C" fn memory_buffer_init(buffer: *mut buffer, max_length: DWORD, alignment: DWORD, vtbl: *const IMFMediaBufferVtbl) -> HRESULT {
    let mut alignment = alignment;
    if alignment < MF_16_BYTE_ALIGNMENT {
        alignment = MF_16_BYTE_ALIGNMENT;
    }
    alignment = alignment.wrapping_add(1);

    if alignment & alignment.wrapping_sub(1) != 0 {
        alignment = alignment.wrapping_sub(1);
        alignment |= alignment >> 1;
        alignment |= alignment >> 2;
        alignment |= alignment >> 4;
        alignment |= alignment >> 8;
        alignment |= alignment >> 16;
        alignment = alignment.wrapping_add(1);
    }

    (*buffer).data = _aligned_malloc(max_length as usize, alignment as usize) as *mut BYTE;
    if (*buffer).data.is_null() {
        return E_OUTOFMEMORY;
    }
    memset(
        (*buffer).data as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        max_length as usize,
    );

    (*buffer).IMFMediaBuffer_iface.lpVtbl = vtbl;
    (*buffer).refcount = 1;
    (*buffer).max_length = max_length;
    (*buffer).current_length = 0;
    InitializeCriticalSection(core::ptr::addr_of_mut!((*buffer).cs));

    return S_OK;
}

unsafe extern "C" fn create_1d_buffer(
    mut max_length: DWORD,
    mut alignment: DWORD,
    mut buffer: *mut *mut IMFMediaBuffer,
) -> HRESULT {
    let mut object: *mut buffer = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    if buffer.is_null() {
        return E_POINTER;
    }

    *buffer = core::ptr::null_mut();

    object = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<buffer>() as size_t) as _,
    ) as *mut buffer;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = memory_buffer_init(
        object,
        max_length,
        alignment,
        core::ptr::addr_of!(memory_1d_buffer_vtbl),
    );
    if hr < 0 {
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    *buffer = core::ptr::addr_of_mut!((*object).IMFMediaBuffer_iface) as *mut IMFMediaBuffer;

    return S_OK;
}

unsafe extern "C" fn get_2d_buffer_copy_func(mut fourcc: DWORD) -> p_copy_image_func {
    // Case labels are MAKEFOURCC values, written out as their numeric equivalents:
    // IMC1=826494281, IMC3=860048713, IMC2=843271497, IMC4=876825929,
    // NV11=825316942, YV12=842094169, I420=808596553, IYUV=1448433993,
    // NV12=842094158, P010=808530000
    match fourcc {
        826494281 /* MAKEFOURCC('I','M','C','1') */ | 860048713 /* MAKEFOURCC('I','M','C','3') */ => {
            return Some(
                copy_image_imc1
                    as unsafe extern "C" fn(*mut BYTE, LONG, *const BYTE, LONG, DWORD, DWORD) -> (),
            );
        }

        843271497 /* MAKEFOURCC('I','M','C','2') */
        | 876825929 /* MAKEFOURCC('I','M','C','4') */
        | 825316942 /* MAKEFOURCC('N','V','1','1') */
        | 842094169 /* MAKEFOURCC('Y','V','1','2') */
        | 808596553 /* MAKEFOURCC('I','4','2','0') */
        | 1448433993 /* MAKEFOURCC('I','Y','U','V') */ => {
            return Some(
                copy_image_imc2
                    as unsafe extern "C" fn(*mut BYTE, LONG, *const BYTE, LONG, DWORD, DWORD) -> (),
            );
        }

        842094158 /* MAKEFOURCC('N','V','1','2') */ | 808530000 /* MAKEFOURCC('P','0','1','0') */ => {
            return Some(
                copy_image_nv12
                    as unsafe extern "C" fn(*mut BYTE, LONG, *const BYTE, LONG, DWORD, DWORD) -> (),
            );
        }

        _ => return None,
    }
}

unsafe extern "C" fn create_2d_buffer(
    width: DWORD,
    height: DWORD,
    fourcc: DWORD,
    bottom_up: BOOL,
    buffer: *mut *mut IMFMediaBuffer,
) -> HRESULT {
    let mut stride: ::core::ffi::c_uint = 0;
    let mut max_length: ::core::ffi::c_uint = 0;
    let mut row_alignment: ::core::ffi::c_uint = 0;
    let mut object: *mut buffer = ::core::ptr::null_mut::<buffer>();
    let mut plane_size: DWORD = 0;
    let mut subtype: GUID = core::mem::zeroed();
    let mut is_yuv: BOOL = 0;
    let mut hr: HRESULT = 0;
    let mut pitch: ::core::ffi::c_int = 0;

    if buffer.is_null() {
        return E_POINTER;
    }

    *buffer = ::core::ptr::null_mut::<IMFMediaBuffer>();

    memcpy(
        core::ptr::addr_of_mut!(subtype) as *mut ::core::ffi::c_void,
        core::ptr::addr_of!(MFVideoFormat_Base) as *const ::core::ffi::c_void,
        ::core::mem::size_of::<GUID>() as usize,
    );
    subtype.Data1 = fourcc as ::core::ffi::c_uint;

    stride = mf_format_get_stride(
        core::ptr::addr_of_mut!(subtype),
        width as ::core::ffi::c_uint,
        core::ptr::addr_of_mut!(is_yuv),
    );
    if stride == 0 {
        return MF_E_INVALIDMEDIATYPE;
    }

    if is_yuv != 0 && bottom_up != 0 {
        return MF_E_INVALIDMEDIATYPE;
    }

    match fourcc {
        826494281 /* MAKEFOURCC('I','M','C','1') */
        | 860048713 /* MAKEFOURCC('I','M','C','3') */ => {
            plane_size = (stride as DWORD)
                .wrapping_mul(height)
                .wrapping_mul(2);
        }
        843271497 /* MAKEFOURCC('I','M','C','2') */
        | 876825929 /* MAKEFOURCC('I','M','C','4') */
        | 825316942 /* MAKEFOURCC('N','V','1','1') */
        | 842094169 /* MAKEFOURCC('Y','V','1','2') */
        | 808596553 /* MAKEFOURCC('I','4','2','0') */
        | 1448433993 /* MAKEFOURCC('I','Y','U','V') */ => {
            plane_size = (stride
                .wrapping_mul(3)
                .wrapping_div(2) as DWORD)
                .wrapping_mul(height);
        }
        842094158 /* MAKEFOURCC('N','V','1','2') */ => {
            plane_size = (stride as DWORD)
                .wrapping_mul(height)
                .wrapping_mul(3)
                .wrapping_div(2);
        }
        808530000 /* MAKEFOURCC('P','0','1','0') */ => {
            plane_size = width
                .wrapping_mul(height)
                .wrapping_mul(3);
        }
        _ => {
            plane_size = (stride as DWORD).wrapping_mul(height);
        }
    }

    object = calloc(
        (1 as usize) as _,
        (::core::mem::size_of::<buffer>() as usize) as _,
    ) as *mut buffer;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    match fourcc {
        826494281 /* MAKEFOURCC('I','M','C','1') */
        | 843271497 /* MAKEFOURCC('I','M','C','2') */
        | 860048713 /* MAKEFOURCC('I','M','C','3') */
        | 876825929 /* MAKEFOURCC('I','M','C','4') */
        | 842094169 /* MAKEFOURCC('Y','V','1','2') */
        | 825316942 /* MAKEFOURCC('N','V','1','1') */
        | 808596553 /* MAKEFOURCC('I','4','2','0') */
        | 1448433993 /* MAKEFOURCC('I','Y','U','V') */ => {
            row_alignment = MF_128_BYTE_ALIGNMENT as ::core::ffi::c_uint;
        }
        _ => {
            row_alignment = MF_64_BYTE_ALIGNMENT as ::core::ffi::c_uint;
        }
    }

    pitch = ALIGN_SIZE!(stride, row_alignment) as ::core::ffi::c_int;

    match fourcc {
        826494281 /* MAKEFOURCC('I','M','C','1') */
        | 860048713 /* MAKEFOURCC('I','M','C','3') */ => {
            max_length = (pitch as DWORD)
                .wrapping_mul(height)
                .wrapping_mul(2) as ::core::ffi::c_uint;
        }
        842094158 /* MAKEFOURCC('N','V','1','2') */
        | 842094169 /* MAKEFOURCC('Y','V','1','2') */
        | 843271497 /* MAKEFOURCC('I','M','C','2') */
        | 876825929 /* MAKEFOURCC('I','M','C','4') */
        | 825316942 /* MAKEFOURCC('N','V','1','1') */
        | 808596553 /* MAKEFOURCC('I','4','2','0') */
        | 1448433993 /* MAKEFOURCC('I','Y','U','V') */
        | 808530000 /* MAKEFOURCC('P','0','1','0') */ => {
            max_length = (pitch as DWORD)
                .wrapping_mul(height)
                .wrapping_mul(3)
                .wrapping_div(2) as ::core::ffi::c_uint;
        }
        _ => {
            max_length = (pitch as DWORD).wrapping_mul(height) as ::core::ffi::c_uint;
        }
    }

    hr = memory_buffer_init(
        object,
        max_length as DWORD,
        row_alignment as DWORD,
        core::ptr::addr_of!(memory_1d_2d_buffer_vtbl),
    );
    if hr < 0 {
        free(object as *mut ::core::ffi::c_void);
        return hr;
    }

    (*object).IMF2DBuffer2_iface.lpVtbl = core::ptr::addr_of!(memory_2d_buffer_vtbl);
    (*object).IMFGetService_iface.lpVtbl = core::ptr::addr_of!(memory_2d_buffer_gs_vtbl);
    (*object)._2d.plane_size = plane_size;
    (*object)._2d.width = stride;
    (*object)._2d.height = height as ::core::ffi::c_uint;
    (*object)._2d.pitch = if bottom_up != 0 { -pitch } else { pitch };
    (*object)._2d.scanline0 = if bottom_up != 0 {
        (*object).data.offset(
            (pitch as ::core::ffi::c_uint)
                .wrapping_mul((*object)._2d.height.wrapping_sub(1 as ::core::ffi::c_uint))
                as isize,
        )
    } else {
        (*object).data
    };
    (*object)._2d.copy_image = get_2d_buffer_copy_func(fourcc);

    *buffer = core::ptr::addr_of_mut!((*object).IMFMediaBuffer_iface) as *mut IMFMediaBuffer;

    return S_OK;
}

unsafe extern "C" fn create_d3d9_surface_buffer(surface: *mut IUnknown, bottom_up: BOOL, buffer: *mut *mut IMFMediaBuffer) -> HRESULT {
    let mut object: *mut buffer = core::ptr::null_mut();
    let mut desc: D3DSURFACE_DESC = core::mem::zeroed();
    let mut stride: u32 = 0;
    let mut subtype: GUID = core::mem::zeroed();
    let mut is_yuv: BOOL = 0;

    IDirect3DSurface9_GetDesc(surface as *mut IDirect3DSurface9, core::ptr::addr_of_mut!(desc));

    TRACE!("format %#x, %u x %u.\n", desc.Format, desc.Width, desc.Height);

    memcpy(
        core::ptr::addr_of_mut!(subtype) as *mut core::ffi::c_void,
        core::ptr::addr_of!(MFVideoFormat_Base) as *const core::ffi::c_void,
        core::mem::size_of::<GUID>() as usize,
    );
    subtype.Data1 = desc.Format as u32;

    stride = mf_format_get_stride(
        core::ptr::addr_of_mut!(subtype),
        desc.Width as u32,
        core::ptr::addr_of_mut!(is_yuv),
    );
    if stride == 0 {
        return MF_E_INVALIDMEDIATYPE;
    }

    object = calloc((1 as usize) as _, (core::mem::size_of::<buffer>() as usize) as _) as *mut buffer;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).IMFMediaBuffer_iface.lpVtbl = core::ptr::addr_of!(d3d9_surface_1d_buffer_vtbl);
    (*object).IMF2DBuffer2_iface.lpVtbl = core::ptr::addr_of!(d3d9_surface_buffer_vtbl);
    (*object).IMFGetService_iface.lpVtbl = core::ptr::addr_of!(d3d9_surface_buffer_gs_vtbl);
    (*object).refcount = 1;
    InitializeCriticalSection(core::ptr::addr_of_mut!((*object).cs));
    (*object).d3d9_surface.surface = surface as *mut IDirect3DSurface9;
    IUnknown_AddRef(surface);

    MFGetPlaneSize(
        desc.Format as DWORD,
        desc.Width as DWORD,
        desc.Height as DWORD,
        core::ptr::addr_of_mut!((*object)._2d.plane_size),
    );
    (*object)._2d.width = stride;
    (*object)._2d.height = desc.Height;
    (*object).max_length = (*object)._2d.plane_size;
    (*object)._2d.copy_image = get_2d_buffer_copy_func(desc.Format as DWORD);

    *buffer = core::ptr::addr_of_mut!((*object).IMFMediaBuffer_iface);

    return S_OK;
}

unsafe extern "C" fn map_dxgi_format_to_dx9_format(format: DXGI_FORMAT) -> D3DFORMAT {
    if format == DXGI_FORMAT_R8_UINT {
        return MFMapDXGIFormatToDX9Format(DXGI_FORMAT_R8_UNORM) as D3DFORMAT;
    }

    return MFMapDXGIFormatToDX9Format(format) as D3DFORMAT;
}

unsafe extern "C" fn create_dxgi_surface_buffer(
    surface: *mut IUnknown,
    sub_resource_idx: ::core::ffi::c_uint,
    bottom_up: BOOL,
    buffer: *mut *mut IMFMediaBuffer,
) -> HRESULT {
    let mut object: *mut buffer = core::ptr::null_mut();
    let mut desc: D3D11_TEXTURE2D_DESC = core::mem::zeroed();
    let mut texture: *mut ID3D11Texture2D = core::ptr::null_mut();
    let mut stride: u32 = 0;
    let mut format: D3DFORMAT = core::mem::zeroed();
    let mut subtype: GUID = core::mem::zeroed();
    let mut is_yuv: BOOL = 0;
    let mut hr: HRESULT = 0;

    hr = IUnknown_QueryInterface(
        surface,
        core::ptr::addr_of!(IID_ID3D11Texture2D),
        core::ptr::addr_of_mut!(texture) as *mut *mut core::ffi::c_void,
    );
    if hr < 0 {
        WARN!("Failed to get texture interface, hr %#lx.\n", hr);
        return hr;
    }

    ID3D11Texture2D_GetDesc(texture, core::ptr::addr_of_mut!(desc));

    memcpy(
        core::ptr::addr_of_mut!(subtype) as *mut core::ffi::c_void,
        core::ptr::addr_of!(MFVideoFormat_Base) as *const core::ffi::c_void,
        core::mem::size_of::<GUID>() as usize,
    );
    format = map_dxgi_format_to_dx9_format(desc.Format);
    subtype.Data1 = format as u32;

    stride = mf_format_get_stride(
        core::ptr::addr_of_mut!(subtype),
        desc.Width,
        core::ptr::addr_of_mut!(is_yuv),
    );
    if stride == 0 {
        ID3D11Texture2D_Release(texture);
        return MF_E_INVALIDMEDIATYPE;
    }

    object = calloc(
        (1 as usize) as _,
        (core::mem::size_of::<buffer>() as usize) as _,
    ) as *mut buffer;
    if object.is_null() {
        ID3D11Texture2D_Release(texture);
        return E_OUTOFMEMORY;
    }

    (*object).IMFMediaBuffer_iface.lpVtbl = core::ptr::addr_of!(dxgi_surface_1d_buffer_vtbl);
    (*object).IMF2DBuffer2_iface.lpVtbl = core::ptr::addr_of!(dxgi_surface_buffer_vtbl);
    (*object).IMFDXGIBuffer_iface.lpVtbl = core::ptr::addr_of!(dxgi_buffer_vtbl);
    (*object).refcount = 1 as LONG;
    InitializeCriticalSection(core::ptr::addr_of_mut!((*object).cs));
    (*object).dxgi_surface.texture = texture;
    (*object).dxgi_surface.sub_resource_idx = sub_resource_idx;

    MFGetPlaneSize(
        format as DWORD,
        desc.Width as DWORD,
        desc.Height as DWORD,
        core::ptr::addr_of_mut!((*object)._2d.plane_size),
    );
    (*object)._2d.width = stride;
    (*object)._2d.height = desc.Height;
    (*object).max_length = (*object)._2d.plane_size;
    (*object)._2d.copy_image = get_2d_buffer_copy_func(format as DWORD);

    hr = init_attributes_object(
        core::ptr::addr_of_mut!((*object).dxgi_surface.attributes),
        0 as UINT32,
    );
    if hr < 0 {
        IMFMediaBuffer_Release(core::ptr::addr_of_mut!((*object).IMFMediaBuffer_iface));
        return hr;
    }

    *buffer = core::ptr::addr_of_mut!((*object).IMFMediaBuffer_iface);

    return S_OK;
}

unsafe extern "C" fn create_d3d12_surface_buffer(
    surface: *mut IUnknown,
    sub_resource_idx: ::core::ffi::c_uint,
    bottom_up: BOOL,
    buffer: *mut *mut IMFMediaBuffer,
) -> HRESULT {
    let mut object: *mut buffer = core::ptr::null_mut();
    let mut device: *mut ID3D12Device = core::ptr::null_mut();
    let mut resource: *mut ID3D12Resource = core::ptr::null_mut();
    let mut desc: D3D12_RESOURCE_DESC = core::mem::zeroed();
    let mut layout: D3D12_PLACED_SUBRESOURCE_FOOTPRINT = core::mem::zeroed();
    let mut total_bytes: UINT64 = 0;
    let mut stride: ::core::ffi::c_uint = 0;
    let mut format: D3DFORMAT = 0;
    let mut subtype: GUID = core::mem::zeroed();
    let mut is_yuv: BOOL = 0;
    let mut hr: HRESULT = 0;

    let _ = bottom_up;

    'end: {
        hr = IUnknown_QueryInterface(
            surface,
            core::ptr::addr_of!(IID_ID3D12Resource),
            core::ptr::addr_of_mut!(resource) as *mut *mut core::ffi::c_void,
        );
        if hr < 0 {
            WARN!("Failed to get resource interface, hr %#lx.\n", hr);
            break 'end;
        }

        hr = ID3D12Resource_GetDevice(
            resource,
            core::ptr::addr_of!(IID_ID3D12Device),
            core::ptr::addr_of_mut!(device) as *mut *mut core::ffi::c_void,
        );
        if hr < 0 {
            break 'end;
        }

        ID3D12Resource_GetDesc(resource, core::ptr::addr_of_mut!(desc));
        ID3D12Device_GetCopyableFootprints(
            device,
            core::ptr::addr_of_mut!(desc),
            sub_resource_idx,
            1,
            0,
            core::ptr::addr_of_mut!(layout),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(total_bytes),
        );

        if desc.Dimension as ::core::ffi::c_uint
            != D3D12_RESOURCE_DIMENSION_TEXTURE2D as ::core::ffi::c_uint
        {
            hr = MF_E_INVALIDMEDIATYPE;
            break 'end;
        }

        memcpy(
            core::ptr::addr_of_mut!(subtype) as *mut core::ffi::c_void,
            core::ptr::addr_of!(MFVideoFormat_Base) as *const core::ffi::c_void,
            core::mem::size_of::<GUID>(),
        );
        format = MFMapDXGIFormatToDX9Format(layout.Footprint.Format) as D3DFORMAT;
        subtype.Data1 = format as DWORD;

        stride = mf_format_get_stride(
            core::ptr::addr_of_mut!(subtype),
            layout.Footprint.Width,
            core::ptr::addr_of_mut!(is_yuv),
        );
        if stride == 0 {
            hr = MF_E_INVALIDMEDIATYPE;
            break 'end;
        }

        object = calloc(
            1,
            (core::mem::size_of::<buffer>()) as _,
        ) as *mut buffer;
        if object.is_null() {
            hr = E_OUTOFMEMORY;
            break 'end;
        }

        (*object).IMFMediaBuffer_iface.lpVtbl = core::ptr::addr_of!(d3d12_surface_1d_buffer_vtbl);
        (*object).IMF2DBuffer2_iface.lpVtbl = core::ptr::addr_of!(d3d12_surface_buffer_vtbl);
        (*object).IMFDXGIBuffer_iface.lpVtbl = core::ptr::addr_of!(d3d12_surface_dxgi_buffer_vtbl);
        (*object).refcount = 1;
        InitializeCriticalSection(core::ptr::addr_of_mut!((*object).cs));
        (*object).d3d12_surface.layout = layout;
        (*object).d3d12_surface.device = device;
        (*object).d3d12_surface.resource = resource;
        (*object).dxgi_surface.sub_resource_idx = sub_resource_idx;

        device = core::ptr::null_mut();
        resource = core::ptr::null_mut();

        MFGetPlaneSize(
            format,
            layout.Footprint.Width,
            layout.Footprint.Height,
            core::ptr::addr_of_mut!((*object)._2d.plane_size),
        );
        (*object)._2d.width = stride;
        (*object)._2d.height = layout.Footprint.Height;
        (*object)._2d.copy_image = get_2d_buffer_copy_func(format);

        (*object).max_length = total_bytes as DWORD;

        hr = init_attributes_object(
            core::ptr::addr_of_mut!((*object).dxgi_surface.attributes),
            0,
        );
        if hr < 0 {
            break 'end;
        }

        hr = MFCreateD3D12SynchronizationObject(
            (*object).d3d12_surface.device,
            core::ptr::addr_of!(IID_IMFD3D12SynchronizationObject),
            core::ptr::addr_of_mut!((*object).d3d12_surface.sync_obj) as *mut *mut core::ffi::c_void,
        );
        if hr < 0 {
            break 'end;
        }

        hr = IMFD3D12SynchronizationObject_QueryInterface(
            (*object).d3d12_surface.sync_obj,
            core::ptr::addr_of!(IID_IMFD3D12SynchronizationObjectCommands),
            core::ptr::addr_of_mut!((*object).d3d12_surface.sync_cmd) as *mut *mut core::ffi::c_void,
        );
        if hr < 0 {
            break 'end;
        }

        IMFDXGIBuffer_SetUnknown(
            core::ptr::addr_of_mut!((*object).IMFDXGIBuffer_iface),
            &MF_D3D12_SYNCHRONIZATION_OBJECT,
            (*object).d3d12_surface.sync_obj as *mut IUnknown,
        );
        IMFDXGIBuffer_SetUnknown(
            core::ptr::addr_of_mut!((*object).IMFDXGIBuffer_iface),
            &MF_D3D12_SYNCHRONIZATION_OBJECT,
            (*object).d3d12_surface.sync_cmd as *mut IUnknown,
        );

        *buffer = core::ptr::addr_of_mut!((*object).IMFMediaBuffer_iface);
        hr = S_OK;
    }

    if !device.is_null() {
        ID3D12Device_Release(device);
    }
    if !resource.is_null() {
        ID3D12Resource_Release(resource);
    }
    if (hr < 0) && !object.is_null() {
        IMFMediaBuffer_Release(core::ptr::addr_of_mut!((*object).IMFMediaBuffer_iface));
    }
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateMemoryBuffer(
    mut max_length: DWORD,
    mut buffer: *mut *mut IMFMediaBuffer,
) -> HRESULT {
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE) as i32;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/buffer.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MFCreateMemoryBuffer\0" as *const u8 as *const ::core::ffi::c_char,
            2204 as ::core::ffi::c_int,
            b"%lu, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            max_length,
            buffer,
        );
    }
    return create_1d_buffer(max_length, MF_1_BYTE_ALIGNMENT as DWORD, buffer);
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateAlignedMemoryBuffer(
    max_length: DWORD,
    alignment: DWORD,
    buffer: *mut *mut IMFMediaBuffer,
) -> HRESULT {
    // TRACE("%lu, %lu, %p.\n", max_length, alignment, buffer);
    create_1d_buffer(max_length, alignment, buffer)
}

#[no_mangle]
pub unsafe extern "system" fn MFCreate2DMediaBuffer(
    mut width: DWORD,
    mut height: DWORD,
    mut fourcc: DWORD,
    mut bottom_up: BOOL,
    mut buffer: *mut *mut IMFMediaBuffer,
) -> HRESULT {
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut core::ffi::c_void = __wine_dbch___default as *mut core::ffi::c_void;
        let __dbcl: u32 = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            (__dbch).cast(),
            b"dll/win32/mfplat/buffer.c\0"
                as *const u8 as *const i8,
            b"MFCreate2DMediaBuffer\0" as *const u8 as *const i8,
            2224,
            b"%lu, %lu, %s, %d, %p.\n\0" as *const u8 as *const i8,
            width,
            height,
            mf_debugstr_fourcc(fourcc),
            bottom_up,
            buffer,
        );
    }
    return create_2d_buffer(width, height, fourcc, bottom_up, buffer);
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateDXSurfaceBuffer(riid: REFIID, surface: *mut IUnknown, bottom_up: BOOL, buffer: *mut *mut IMFMediaBuffer) -> HRESULT {
    DPRINT1!("%s, %p, %d, %p.\n", debugstr_guid(riid), surface, bottom_up, buffer);

    if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IDirect3DSurface9))) == 0) {
        return E_INVALIDARG;
    }

    return create_d3d9_surface_buffer(surface, bottom_up, buffer);
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateDXGISurfaceBuffer(
    riid: REFIID,
    surface: *mut IUnknown,
    subresource: UINT,
    bottom_up: BOOL,
    buffer: *mut *mut IMFMediaBuffer,
) -> HRESULT {
    TRACE!(
        "%s, %p, %u, %d, %p.\n",
        debugstr_guid(riid as *const GUID),
        surface,
        subresource,
        bottom_up,
        buffer
    );
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_ID3D11Texture2D)) != 0 {
        return create_dxgi_surface_buffer(surface, subresource, bottom_up, buffer);
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_ID3D12Resource)) != 0 {
        return create_d3d12_surface_buffer(surface, subresource, bottom_up, buffer);
    } else {
        return E_INVALIDARG;
    }
}

unsafe extern "C" fn buffer_get_aligned_length(mut length: u32, mut alignment: u32) -> u32 {
    length = length.wrapping_add(alignment).wrapping_div(alignment);
    length = length.wrapping_mul(alignment);

    return length;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateMediaBufferFromMediaType(
    media_type: *mut IMFMediaType,
    duration: LONGLONG,
    min_length: DWORD,
    mut alignment: DWORD,
    buffer: *mut *mut IMFMediaBuffer,
) -> HRESULT {
    let mut length: UINT32 = 0;
    let mut block_alignment: UINT32;
    let mut avg_length: LONGLONG;
    let mut major: GUID = GUID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    };
    let mut subtype: GUID = GUID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    };
    let mut frame_size: UINT64 = core::mem::zeroed();
    let mut is_yuv: BOOL = core::mem::zeroed();
    let mut hr: HRESULT;

    TRACE!(
        "%p, %s, %lu, %lu, %p.\n",
        media_type,
        debugstr_time(duration),
        min_length,
        alignment,
        buffer
    );

    if media_type.is_null() {
        return E_INVALIDARG;
    }

    hr = IMFMediaType_GetMajorType(media_type, &raw mut major);
    if hr < 0 {
        return hr;
    }

    if IsEqualGUID!(&raw const major, &raw const MFMediaType_Audio) {
        block_alignment = 0;
        if IMFMediaType_GetUINT32(
            media_type,
            &raw const MF_MT_AUDIO_BLOCK_ALIGNMENT,
            &raw mut block_alignment,
        ) < 0
        {
            WARN!("Block alignment was not specified.\n");
        }

        alignment = max!(16, alignment);

        if block_alignment != 0 {
            avg_length = 0;

            if duration != 0 {
                length = 0;
                if IMFMediaType_GetUINT32(
                    media_type,
                    &raw const MF_MT_AUDIO_AVG_BYTES_PER_SECOND,
                    &raw mut length,
                ) >= 0
                {
                    /* 100 ns -> 1 s */
                    avg_length = length as LONGLONG * duration
                        / ((10 * 1000 * 1000) as LONGLONG);
                }
            }

            length = buffer_get_aligned_length(
                (avg_length + 1) as UINT32,
                alignment,
            );
            length = buffer_get_aligned_length(length, block_alignment);
        } else {
            length = 0;
        }

        length = max!(length, min_length);

        return create_1d_buffer(
            length,
            alignment - 1,
            buffer,
        );
    }

    if IsEqualGUID!(&raw const major, &raw const MFMediaType_Video) {
        hr = IMFMediaType_GetGUID(
            media_type,
            &raw const MF_MT_SUBTYPE,
            &raw mut subtype,
        );
        if hr >= 0
            && ({
                hr = IMFMediaType_GetUINT64(
                    media_type,
                    &raw const MF_MT_FRAME_SIZE,
                    &raw mut frame_size,
                );
                hr >= 0
            })
            && mf_format_get_stride(
                &raw mut subtype,
                (frame_size >> 32) as u32,
                &raw mut is_yuv,
            ) != 0
        {
            let mut bottom_up: BOOL = (FALSE as i32);
            let mut stride: UINT32 = 0;

            if is_yuv == 0
                && IMFMediaType_GetUINT32(
                    media_type,
                    &raw const MF_MT_DEFAULT_STRIDE,
                    &raw mut stride,
                ) >= 0
            {
                bottom_up = ((stride as i32) < 0) as BOOL;
            }

            hr = create_2d_buffer(
                (frame_size >> 32) as DWORD,
                frame_size as UINT32,
                subtype.Data1,
                bottom_up,
                buffer,
            );
            if hr >= 0 {
                return hr;
            }
        }

        if min_length == 0 {
            return if hr < 0 {
                hr
            } else {
                E_INVALIDARG
            };
        }

        alignment = max!(16, alignment);
        return create_1d_buffer(
            min_length,
            alignment - 1,
            buffer,
        );
    }

    FIXME!(
        "Major type %s is not supported.\n",
        debugstr_guid(&raw const major)
    );
    return E_NOTIMPL;
}

unsafe extern "C" fn legacy_buffer_from_IMediaBuffer(iface: *mut IMediaBuffer) -> *mut legacy_buffer {
    return (iface as usize).wrapping_sub(
        core::mem::offset_of!(legacy_buffer, IMediaBuffer_iface),
    ) as *mut legacy_buffer;
}

unsafe extern "system" fn legacy_buffer_QueryInterface(iface: *mut IMediaBuffer, iid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMediaBuffer(iface);

    if IsEqualGUID!(iid, core::ptr::addr_of!(IID_IUnknown))
        || IsEqualGUID!(iid, core::ptr::addr_of!(IID_IMediaBuffer))
    {
        IMediaBuffer_AddRef(core::ptr::addr_of_mut!((*buffer).IMediaBuffer_iface));
        *obj = core::ptr::addr_of_mut!((*buffer).IMediaBuffer_iface) as *mut core::ffi::c_void;
        return S_OK;
    }

    if !(*buffer).sample.is_null()
        && IsEqualGUID!(iid, core::ptr::addr_of!(IID_IMFSample))
    {
        IMFSample_AddRef(core::ptr::addr_of_mut!((*buffer).IMFSample_iface));
        *obj = core::ptr::addr_of_mut!((*buffer).IMFSample_iface) as *mut core::ffi::c_void;
        return S_OK;
    }

    if !(*buffer).buffer_2d.is_null()
        && (IsEqualGUID!(iid, core::ptr::addr_of!(IID_IMF2DBuffer))
            || IsEqualGUID!(iid, core::ptr::addr_of!(IID_IMF2DBuffer2)))
    {
        IMF2DBuffer2_AddRef(core::ptr::addr_of_mut!((*buffer).IMF2DBuffer2_iface));
        *obj = core::ptr::addr_of_mut!((*buffer).IMF2DBuffer2_iface) as *mut core::ffi::c_void;
        return S_OK;
    }

    WARN!("%s not implemented, returning E_NOINTERFACE.\n", debugstr_guid(iid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn legacy_buffer_AddRef(iface: *mut IMediaBuffer) -> ULONG {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMediaBuffer(iface);
    let refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*buffer).refcount) as *mut ::core::ffi::c_long,
    ) as ULONG;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE) as i32;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/buffer.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"legacy_buffer_AddRef\0" as *const u8 as *const ::core::ffi::c_char,
            2403 as ::core::ffi::c_int,
            b"%p, refcount %lu.\n\0" as *const u8 as *const ::core::ffi::c_char,
            buffer,
            refcount,
        );
    }
    return refcount;
}

unsafe extern "system" fn legacy_buffer_Release(iface: *mut IMediaBuffer) -> ULONG {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMediaBuffer(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*buffer).refcount)) as ULONG;

    if refcount == 0 {
        if !(*buffer).locked_data.is_null() {
            IMFMediaBuffer_Unlock((*buffer).media_buffer);
        }
        if !(*buffer).sample.is_null() {
            IMFSample_Release((*buffer).sample);
        }
        if !(*buffer).buffer_2d.is_null() {
            IMF2DBuffer2_Release((*buffer).buffer_2d);
        }
        IMFMediaBuffer_Release((*buffer).media_buffer);
        free(buffer as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn legacy_buffer_SetLength(iface: *mut IMediaBuffer, len: DWORD) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMediaBuffer(iface);
    return IMFMediaBuffer_SetCurrentLength((*buffer).media_buffer, len);
}

unsafe extern "system" fn legacy_buffer_GetMaxLength(iface: *mut IMediaBuffer, len: *mut DWORD) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMediaBuffer(iface);
    return IMFMediaBuffer_GetMaxLength((*buffer).media_buffer, len);
}

unsafe extern "system" fn legacy_buffer_GetBufferAndLength(iface: *mut IMediaBuffer, data: *mut *mut BYTE, len: *mut DWORD) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMediaBuffer(iface);
    let hr: HRESULT;

    *data = InterlockedCompareExchangePointer(
        core::ptr::addr_of_mut!((*buffer).locked_data) as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) as *mut BYTE;
    if !(*data).is_null() {
        return IMFMediaBuffer_GetCurrentLength((*buffer).media_buffer, len);
    }
    hr = IMFMediaBuffer_Lock((*buffer).media_buffer, data, core::ptr::null_mut(), len);
    if hr >= 0 {
        InterlockedExchangePointer(
            core::ptr::addr_of_mut!((*buffer).locked_data) as *mut *mut core::ffi::c_void,
            *data as *mut core::ffi::c_void,
        );
    }
    return hr;
}

unsafe extern "C" fn legacy_buffer_from_IMF2DBuffer2(mut iface: *mut IMF2DBuffer2) -> *mut legacy_buffer {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(legacy_buffer, IMF2DBuffer2_iface)) as *mut legacy_buffer;
}

unsafe extern "system" fn legacy_buffer_IMF2DBuffer2_QueryInterface(iface: *mut IMF2DBuffer2, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMF2DBuffer2(iface);
    return IMediaBuffer_QueryInterface(core::ptr::addr_of_mut!((*buffer).IMediaBuffer_iface), riid, obj);
}

unsafe extern "system" fn legacy_buffer_IMF2DBuffer2_AddRef(iface: *mut IMF2DBuffer2) -> ULONG {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMF2DBuffer2(iface);
    return IMediaBuffer_AddRef(core::ptr::addr_of_mut!((*buffer).IMediaBuffer_iface));
}

unsafe extern "system" fn legacy_buffer_IMF2DBuffer2_Release(iface: *mut IMF2DBuffer2) -> ULONG {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMF2DBuffer2(iface);
    return IMediaBuffer_Release(core::ptr::addr_of_mut!((*buffer).IMediaBuffer_iface));
}
unsafe extern "system" fn legacy_buffer_IMF2DBuffer2_Lock2D(iface: *mut IMF2DBuffer2, scanline0: *mut *mut BYTE, pitch: *mut LONG) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMF2DBuffer2(iface);
    return IMF2DBuffer2_Lock2D((*buffer).buffer_2d, scanline0, pitch);
}

unsafe extern "system" fn legacy_buffer_IMF2DBuffer2_Unlock2D(iface: *mut IMF2DBuffer2) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMF2DBuffer2(iface);
    return IMF2DBuffer2_Unlock2D((*buffer).buffer_2d);
}

unsafe extern "system" fn legacy_buffer_IMF2DBuffer2_GetScanline0AndPitch(iface: *mut IMF2DBuffer2, scanline0: *mut *mut BYTE, pitch: *mut LONG) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMF2DBuffer2(iface);
    return IMF2DBuffer2_GetScanline0AndPitch((*buffer).buffer_2d, scanline0, pitch);
}

unsafe extern "system" fn legacy_buffer_IMF2DBuffer2_IsContiguousFormat(iface: *mut IMF2DBuffer2, is_contiguous: *mut BOOL) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMF2DBuffer2(iface);
    return IMF2DBuffer2_IsContiguousFormat((*buffer).buffer_2d, is_contiguous);
}

unsafe extern "system" fn legacy_buffer_IMF2DBuffer2_GetContiguousLength(iface: *mut IMF2DBuffer2, length: *mut DWORD) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMF2DBuffer2(iface);
    return IMF2DBuffer2_GetContiguousLength((*buffer).buffer_2d, length);
}

unsafe extern "system" fn legacy_buffer_IMF2DBuffer2_ContiguousCopyTo(iface: *mut IMF2DBuffer2, dest_buffer: *mut BYTE, dest_length: DWORD) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMF2DBuffer2(iface);
    return IMF2DBuffer2_ContiguousCopyTo((*buffer).buffer_2d, dest_buffer, dest_length);
}

unsafe extern "system" fn legacy_buffer_IMF2DBuffer2_ContiguousCopyFrom(iface: *mut IMF2DBuffer2, src_buffer: *const BYTE, src_length: DWORD) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMF2DBuffer2(iface);
    return IMF2DBuffer2_ContiguousCopyFrom((*buffer).buffer_2d, src_buffer, src_length);
}
unsafe extern "system" fn legacy_buffer_IMF2DBuffer2_Lock2DSize(
    iface: *mut IMF2DBuffer2,
    flags: MF2DBuffer_LockFlags,
    scanline0: *mut *mut BYTE,
    pitch: *mut LONG,
    buffer_start: *mut *mut BYTE,
    buffer_length: *mut DWORD,
) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMF2DBuffer2(iface);
    return IMF2DBuffer2_Lock2DSize((*buffer).buffer_2d, flags, scanline0, pitch, buffer_start, buffer_length);
}

unsafe extern "system" fn legacy_buffer_IMF2DBuffer2_Copy2DTo(iface: *mut IMF2DBuffer2, dest_buffer: *mut IMF2DBuffer2) -> HRESULT {
    let mut buffer: *mut legacy_buffer = legacy_buffer_from_IMF2DBuffer2(iface);
    return IMF2DBuffer2_Copy2DTo((*buffer).buffer_2d, dest_buffer);
}

unsafe extern "C" fn legacy_buffer_from_IMFSample(iface: *mut IMFSample) -> *mut legacy_buffer {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(legacy_buffer, IMFSample_iface)) as *mut legacy_buffer;
}

unsafe extern "system" fn legacy_buffer_IMFSample_QueryInterface(iface: *mut IMFSample, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMediaBuffer_QueryInterface(core::ptr::addr_of_mut!((*buffer).IMediaBuffer_iface), riid, obj);
}

unsafe extern "system" fn legacy_buffer_IMFSample_AddRef(iface: *mut IMFSample) -> ULONG {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMediaBuffer_AddRef(core::ptr::addr_of_mut!((*buffer).IMediaBuffer_iface));
}

unsafe extern "system" fn legacy_buffer_IMFSample_Release(iface: *mut IMFSample) -> ULONG {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMediaBuffer_Release(core::ptr::addr_of_mut!((*buffer).IMediaBuffer_iface));
}

unsafe extern "system" fn legacy_buffer_IMFSample_GetItem(iface: *mut IMFSample, key: REFGUID, value: *mut PROPVARIANT) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_GetItem((*buffer).sample, key, value);
}

unsafe extern "system" fn legacy_buffer_IMFSample_GetItemType(iface: *mut IMFSample, key: *const GUID, type_0: *mut MF_ATTRIBUTE_TYPE) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_GetItemType((*buffer).sample, key, type_0);
}

unsafe extern "system" fn legacy_buffer_IMFSample_CompareItem(iface: *mut IMFSample, key: REFGUID, value: REFPROPVARIANT, result: *mut BOOL) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_CompareItem((*buffer).sample, key, value, result);
}

unsafe extern "system" fn legacy_buffer_IMFSample_Compare(
    iface: *mut IMFSample,
    theirs: *mut IMFAttributes,
    type_0: MF_ATTRIBUTES_MATCH_TYPE,
    result: *mut BOOL,
) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_Compare((*buffer).sample, theirs, type_0, result);
}

unsafe extern "system" fn legacy_buffer_IMFSample_GetUINT32(iface: *mut IMFSample, key: REFGUID, value: *mut UINT32) -> HRESULT {
    let mut buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_GetUINT32((*buffer).sample, key, value);
}

unsafe extern "system" fn legacy_buffer_IMFSample_GetUINT64(iface: *mut IMFSample, key: *const GUID, value: *mut UINT64) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_GetUINT64((*buffer).sample, key, value);
}

unsafe extern "system" fn legacy_buffer_IMFSample_GetDouble(iface: *mut IMFSample, key: REFGUID, value: *mut f64) -> HRESULT {
    let mut buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_GetDouble((*buffer).sample, key, value);
}

unsafe extern "system" fn legacy_buffer_IMFSample_GetGUID(iface: *mut IMFSample, key: *const GUID, value: *mut GUID) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_GetGUID((*buffer).sample, key, value);
}

unsafe extern "system" fn legacy_buffer_IMFSample_GetStringLength(iface: *mut IMFSample, key: *const GUID, length: *mut UINT32) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_GetStringLength((*buffer).sample, key, length);
}

unsafe extern "system" fn legacy_buffer_IMFSample_GetString(iface: *mut IMFSample, key: REFGUID, value: *mut WCHAR, size: UINT32, length: *mut UINT32) -> HRESULT {
    let mut buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_GetString((*buffer).sample, key, value, size, length);
}

unsafe extern "system" fn legacy_buffer_IMFSample_GetAllocatedString(iface: *mut IMFSample, key: *const GUID, value: *mut *mut WCHAR, length: *mut UINT32) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_GetAllocatedString((*buffer).sample, key, value, length);
}

unsafe extern "system" fn legacy_buffer_IMFSample_GetBlobSize(iface: *mut IMFSample, key: *const GUID, size: *mut UINT32) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_GetBlobSize((*buffer).sample, key, size);
}

unsafe extern "system" fn legacy_buffer_IMFSample_GetBlob(iface: *mut IMFSample, key: REFGUID, buf: *mut UINT8, bufsize: UINT32, blobsize: *mut UINT32) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_GetBlob((*buffer).sample, key, buf, bufsize, blobsize);
}

unsafe extern "system" fn legacy_buffer_IMFSample_GetAllocatedBlob(iface: *mut IMFSample, key: REFGUID, buf: *mut *mut UINT8, size: *mut UINT32) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_GetAllocatedBlob((*buffer).sample, key, buf, size);
}
unsafe extern "system" fn legacy_buffer_IMFSample_GetUnknown(
    mut iface: *mut IMFSample,
    key: *const GUID,
    riid: *const IID,
    mut out: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_GetUnknown((*buffer).sample, key, riid, out as *mut LPVOID);
}

unsafe extern "system" fn legacy_buffer_IMFSample_SetItem(iface: *mut IMFSample, key: *const GUID, value: *const PROPVARIANT) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_SetItem((*buffer).sample, key, value);
}

unsafe extern "system" fn legacy_buffer_IMFSample_DeleteItem(iface: *mut IMFSample, key: *const GUID) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_DeleteItem((*buffer).sample, key);
}
unsafe extern "system" fn legacy_buffer_IMFSample_DeleteAllItems(iface: *mut IMFSample) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_DeleteAllItems((*buffer).sample);
}

unsafe extern "system" fn legacy_buffer_IMFSample_SetUINT32(iface: *mut IMFSample, key: REFGUID, value: UINT32) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_SetUINT32((*buffer).sample, key, value);
}

unsafe extern "system" fn legacy_buffer_IMFSample_SetUINT64(iface: *mut IMFSample, key: *const GUID, value: UINT64) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_SetUINT64((*buffer).sample, key, value);
}

unsafe extern "system" fn legacy_buffer_IMFSample_SetDouble(iface: *mut IMFSample, key: *const GUID, value: f64) -> HRESULT {
    let mut buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_SetDouble((*buffer).sample, key, value);
}

unsafe extern "system" fn legacy_buffer_IMFSample_SetGUID(iface: *mut IMFSample, key: REFGUID, value: REFGUID) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_SetGUID((*buffer).sample, key, value);
}

unsafe extern "system" fn legacy_buffer_IMFSample_SetString(iface: *mut IMFSample, key: *const GUID, value: *const WCHAR) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_SetString((*buffer).sample, key, value);
}

unsafe extern "system" fn legacy_buffer_IMFSample_SetBlob(iface: *mut IMFSample, key: REFGUID, buf: *const UINT8, size: UINT32) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_SetBlob((*buffer).sample, key, buf, size);
}

unsafe extern "system" fn legacy_buffer_IMFSample_SetUnknown(iface: *mut IMFSample, key: *const GUID, unknown: *mut IUnknown) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_SetUnknown((*buffer).sample, key, unknown);
}

unsafe extern "system" fn legacy_buffer_IMFSample_LockStore(iface: *mut IMFSample) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_LockStore((*buffer).sample);
}

unsafe extern "system" fn legacy_buffer_IMFSample_UnlockStore(iface: *mut IMFSample) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_UnlockStore((*buffer).sample);
}

unsafe extern "system" fn legacy_buffer_IMFSample_GetCount(iface: *mut IMFSample, count: *mut UINT32) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_GetCount((*buffer).sample, count);
}

unsafe extern "system" fn legacy_buffer_IMFSample_GetItemByIndex(iface: *mut IMFSample, index: UINT32, key: *mut GUID, value: *mut PROPVARIANT) -> HRESULT {
    let mut buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_GetItemByIndex((*buffer).sample, index, key, value);
}

unsafe extern "system" fn legacy_buffer_IMFSample_CopyAllItems(iface: *mut IMFSample, dest: *mut IMFAttributes) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_CopyAllItems((*buffer).sample, dest);
}

unsafe extern "system" fn legacy_buffer_IMFSample_GetSampleFlags(iface: *mut IMFSample, flags: *mut DWORD) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_GetSampleFlags((*buffer).sample, flags);
}

unsafe extern "system" fn legacy_buffer_IMFSample_SetSampleFlags(iface: *mut IMFSample, flags: DWORD) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_SetSampleFlags((*buffer).sample, flags);
}

unsafe extern "system" fn legacy_buffer_IMFSample_GetSampleTime(iface: *mut IMFSample, timestamp: *mut LONGLONG) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_GetSampleTime((*buffer).sample, timestamp);
}

unsafe extern "system" fn legacy_buffer_IMFSample_SetSampleTime(iface: *mut IMFSample, timestamp: LONGLONG) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_SetSampleTime((*buffer).sample, timestamp);
}

unsafe extern "system" fn legacy_buffer_IMFSample_GetSampleDuration(iface: *mut IMFSample, duration: *mut LONGLONG) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_GetSampleDuration((*buffer).sample, duration);
}

unsafe extern "system" fn legacy_buffer_IMFSample_SetSampleDuration(iface: *mut IMFSample, duration: LONGLONG) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_SetSampleDuration((*buffer).sample, duration);
}

unsafe extern "system" fn legacy_buffer_IMFSample_GetBufferCount(iface: *mut IMFSample, count: *mut DWORD) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_GetBufferCount((*buffer).sample, count);
}

unsafe extern "system" fn legacy_buffer_IMFSample_GetBufferByIndex(iface: *mut IMFSample, index: DWORD, out: *mut *mut IMFMediaBuffer) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_GetBufferByIndex((*buffer).sample, index, out);
}

unsafe extern "system" fn legacy_buffer_IMFSample_ConvertToContiguousBuffer(iface: *mut IMFSample, out: *mut *mut IMFMediaBuffer) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_ConvertToContiguousBuffer((*buffer).sample, out);
}

unsafe extern "system" fn legacy_buffer_IMFSample_AddBuffer(iface: *mut IMFSample, media_buffer: *mut IMFMediaBuffer) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_AddBuffer((*buffer).sample, media_buffer);
}

unsafe extern "system" fn legacy_buffer_IMFSample_RemoveBufferByIndex(iface: *mut IMFSample, index: DWORD) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_RemoveBufferByIndex((*buffer).sample, index);
}

unsafe extern "system" fn legacy_buffer_IMFSample_RemoveAllBuffers(iface: *mut IMFSample) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_RemoveAllBuffers((*buffer).sample);
}

unsafe extern "system" fn legacy_buffer_IMFSample_GetTotalLength(iface: *mut IMFSample, total_length: *mut DWORD) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_GetTotalLength((*buffer).sample, total_length);
}

unsafe extern "system" fn legacy_buffer_IMFSample_CopyToBuffer(iface: *mut IMFSample, dest: *mut IMFMediaBuffer) -> HRESULT {
    let buffer: *mut legacy_buffer = legacy_buffer_from_IMFSample(iface);
    return IMFSample_CopyToBuffer((*buffer).sample, dest);
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateLegacyMediaBufferOnMFMediaBuffer(
    mut sample: *mut IMFSample,
    mut media_buffer: *mut IMFMediaBuffer,
    mut offset: DWORD,
    mut obj: *mut *mut IMediaBuffer,
) -> HRESULT {
    let mut buffer: *mut legacy_buffer = core::ptr::null_mut::<legacy_buffer>();
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        TRACE!(
            "sample %p, media_buffer %p, offset %lu, out %p\n",
            sample,
            media_buffer,
            offset,
            obj
        );
    }
    if obj.is_null() {
        return E_INVALIDARG;
    }
    if offset != 0 {
        if (*__wine_dbch___default).flags as i32
            & (1 as i32) << __WINE_DBCL_FIXME as i32
            != 0
        {
            FIXME!("Not implemented\n");
        }
        return E_NOTIMPL;
    }
    buffer = calloc(
        1,
        (core::mem::size_of::<legacy_buffer>() as size_t) as _,
    ) as *mut legacy_buffer;
    if buffer.is_null() {
        return E_OUTOFMEMORY;
    }
    (*buffer).IMediaBuffer_iface.lpVtbl = &raw const legacy_buffer_vtbl;
    (*buffer).IMFSample_iface.lpVtbl = &raw const legacy_buffer_IMFSample_vtbl;
    (*buffer).IMF2DBuffer2_iface.lpVtbl = &raw const legacy_buffer_IMF2DBuffer2_vtbl;
    (*buffer).refcount = 1;

    (*buffer).media_buffer = media_buffer;
    IMFMediaBuffer_AddRef((*buffer).media_buffer);
    (*buffer).sample = sample;
    if !(*buffer).sample.is_null() {
        IMFSample_AddRef((*buffer).sample);
    }
    if IMFMediaBuffer_QueryInterface(
        media_buffer,
        &raw const IID_IMF2DBuffer2,
        &raw mut (*buffer).buffer_2d as *mut *mut core::ffi::c_void,
    ) < 0
    {
        (*buffer).buffer_2d = core::ptr::null_mut::<IMF2DBuffer2>();
    }
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        TRACE!("created %p\n", buffer);
    }
    *obj = &raw mut (*buffer).IMediaBuffer_iface;
    return S_OK;
}

unsafe extern "C" fn impl_from_wrapper_IMFMediaBuffer(mut iface: *mut IMFMediaBuffer) -> *mut buffer_wrapper {
    return (iface as *mut u8).sub(core::mem::offset_of!(buffer_wrapper, IMFMediaBuffer_iface)) as *mut buffer_wrapper;
}

unsafe extern "system" fn buffer_wrapper_QueryInterface(iface: *mut IMFMediaBuffer, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let wrapper: *mut buffer_wrapper = impl_from_wrapper_IMFMediaBuffer(iface);

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFMediaBuffer)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *out = core::ptr::addr_of_mut!((*wrapper).IMFMediaBuffer_iface) as *mut core::ffi::c_void;
        IMFMediaBuffer_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported %s.\n", debugstr_guid(riid));
    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}
unsafe extern "system" fn buffer_wrapper_AddRef(mut iface: *mut IMFMediaBuffer) -> ULONG {
    let mut wrapper: *mut buffer_wrapper = impl_from_wrapper_IMFMediaBuffer(iface);
    let mut refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*wrapper).refcount) as *mut LONG,
    ) as ULONG;
    if ((((*__wine_dbch___default).flags as i32) & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0) {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE) as i32;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/buffer.c\0"
                as *const u8 as *const i8,
            b"buffer_wrapper_AddRef\0" as *const u8 as *const i8,
            2974 as i32,
            b"%p, refcount %lu.\n\0" as *const u8 as *const i8,
            iface,
            refcount,
        );
    }
    return refcount;
}
unsafe extern "system" fn buffer_wrapper_Release(iface: *mut IMFMediaBuffer) -> ULONG {
    let wrapper: *mut buffer_wrapper = impl_from_wrapper_IMFMediaBuffer(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*wrapper).refcount) as *mut i32) as ULONG;

    if refcount == 0 {
        IMFMediaBuffer_Release((*wrapper).parent);
        free(wrapper as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn buffer_wrapper_Lock(iface: *mut IMFMediaBuffer, data: *mut *mut BYTE, max_length: *mut DWORD, current_length: *mut DWORD) -> HRESULT {
    let wrapper: *mut buffer_wrapper = impl_from_wrapper_IMFMediaBuffer(iface);
    let mut parent_data: *mut BYTE = core::ptr::null_mut();
    let mut hr: HRESULT;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE) as i32;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/buffer.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"buffer_wrapper_Lock\0" as *const u8 as *const ::core::ffi::c_char,
            3001 as ::core::ffi::c_int,
            b"%p, %p, %p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            data,
            max_length,
            current_length,
        );
    }
    if data.is_null() {
        return E_INVALIDARG;
    }
    hr = IMFMediaBuffer_Lock(
        (*wrapper).parent,
        core::ptr::addr_of_mut!(parent_data),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<DWORD>(),
    );
    if hr < 0 {
        return hr;
    }
    *data = parent_data.offset((*wrapper).offset as isize);
    if !max_length.is_null() {
        *max_length = (*wrapper).max_length;
    }
    if !current_length.is_null() {
        *current_length = (*wrapper).current_length;
    }
    return S_OK;
}

unsafe extern "system" fn buffer_wrapper_Unlock(iface: *mut IMFMediaBuffer) -> HRESULT {
    let mut wrapper: *mut buffer_wrapper = impl_from_wrapper_IMFMediaBuffer(iface);

    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE) as i32;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/buffer.c\0"
                as *const u8 as *const i8,
            b"buffer_wrapper_Unlock\0" as *const u8 as *const i8,
            3022 as i32,
            b"%p.\n\0" as *const u8 as *const i8,
            iface,
        );
    }

    return IMFMediaBuffer_Unlock((*wrapper).parent);
}

unsafe extern "system" fn buffer_wrapper_GetCurrentLength(iface: *mut IMFMediaBuffer, current_length: *mut DWORD) -> HRESULT {
    let wrapper: *mut buffer_wrapper = impl_from_wrapper_IMFMediaBuffer(iface);

    TRACE!("%p, %p.\n", iface, current_length);

    if current_length.is_null() {
        return E_INVALIDARG;
    }

    *current_length = (*wrapper).current_length;

    return S_OK;
}

unsafe extern "system" fn buffer_wrapper_SetCurrentLength(
    iface: *mut IMFMediaBuffer,
    current_length: DWORD,
) -> HRESULT {
    let wrapper: *mut buffer_wrapper = impl_from_wrapper_IMFMediaBuffer(iface);

    if current_length > (*wrapper).max_length {
        return E_INVALIDARG;
    }

    (*wrapper).current_length = current_length;

    return S_OK;
}

unsafe extern "system" fn buffer_wrapper_GetMaxLength(iface: *mut IMFMediaBuffer, max_length: *mut DWORD) -> HRESULT {
    let wrapper: *mut buffer_wrapper = impl_from_wrapper_IMFMediaBuffer(iface);

    if max_length.is_null() {
        return E_INVALIDARG;
    }

    *max_length = (*wrapper).max_length;

    return S_OK;
}
#[no_mangle]
pub unsafe extern "system" fn MFCreateMediaBufferWrapper(buffer: *mut IMFMediaBuffer, offset: DWORD, length: DWORD, out: *mut *mut IMFMediaBuffer) -> HRESULT {
    let mut wrapper: *mut buffer_wrapper = core::ptr::null_mut();
    let mut parent_length: DWORD = 0;
    let mut hr: HRESULT;

    if out.is_null() || buffer.is_null() {
        return E_POINTER;
    }

    hr = IMFMediaBuffer_GetCurrentLength(buffer, core::ptr::addr_of_mut!(parent_length));
    if hr < 0 {
        return hr;
    }

    if offset > parent_length || length > parent_length.wrapping_sub(offset) {
        return E_OUTOFMEMORY;
    }

    wrapper = calloc((1 as size_t) as _, (core::mem::size_of::<buffer_wrapper>() as size_t) as _) as *mut buffer_wrapper;
    if wrapper.is_null() {
        return E_OUTOFMEMORY;
    }

    (*wrapper).IMFMediaBuffer_iface.lpVtbl = core::ptr::addr_of!(buffer_wrapper_vtbl);
    (*wrapper).refcount = 1;
    (*wrapper).parent = buffer;
    IMFMediaBuffer_AddRef(buffer);
    (*wrapper).offset = offset;
    (*wrapper).max_length = length;
    (*wrapper).current_length = length;

    *out = core::ptr::addr_of_mut!((*wrapper).IMFMediaBuffer_iface);

    return S_OK;
}

// COM vtable tables — generated from buffer.c by vtbl_gen.py
/// buffer.c:288 `static const IMFMediaBufferVtbl memory_1d_buffer_vtbl`  (slot order: rust:types_amd64.rs)
pub static memory_1d_buffer_vtbl: IMFMediaBufferVtbl = IMFMediaBufferVtbl {
    QueryInterface: Some(memory_buffer_QueryInterface),
    AddRef: Some(memory_buffer_AddRef),
    Release: Some(memory_buffer_Release),
    Lock: Some(memory_buffer_Lock),
    Unlock: Some(memory_buffer_Unlock),
    GetCurrentLength: Some(memory_buffer_GetCurrentLength),
    SetCurrentLength: Some(memory_buffer_SetCurrentLength),
    GetMaxLength: Some(memory_buffer_GetMaxLength),
};

/// buffer.c:405 `static const IMFMediaBufferVtbl memory_1d_2d_buffer_vtbl`  (slot order: rust:types_amd64.rs)
pub static memory_1d_2d_buffer_vtbl: IMFMediaBufferVtbl = IMFMediaBufferVtbl {
    QueryInterface: Some(memory_1d_2d_buffer_QueryInterface),
    AddRef: Some(memory_buffer_AddRef),
    Release: Some(memory_buffer_Release),
    Lock: Some(memory_1d_2d_buffer_Lock),
    Unlock: Some(memory_1d_2d_buffer_Unlock),
    GetCurrentLength: Some(memory_buffer_GetCurrentLength),
    SetCurrentLength: Some(memory_buffer_SetCurrentLength),
    GetMaxLength: Some(memory_buffer_GetMaxLength),
};

/// buffer.c:507 `static const IMFMediaBufferVtbl d3d9_surface_1d_buffer_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d9_surface_1d_buffer_vtbl: IMFMediaBufferVtbl = IMFMediaBufferVtbl {
    QueryInterface: Some(memory_1d_2d_buffer_QueryInterface),
    AddRef: Some(memory_buffer_AddRef),
    Release: Some(memory_buffer_Release),
    Lock: Some(d3d9_surface_buffer_Lock),
    Unlock: Some(d3d9_surface_buffer_Unlock),
    GetCurrentLength: Some(memory_buffer_GetCurrentLength),
    SetCurrentLength: Some(d3d9_surface_buffer_SetCurrentLength),
    GetMaxLength: Some(memory_buffer_GetMaxLength),
};

/// buffer.c:735 `static const IMF2DBuffer2Vtbl memory_2d_buffer_vtbl`  (slot order: rust:types_amd64.rs)
pub static memory_2d_buffer_vtbl: IMF2DBuffer2Vtbl = IMF2DBuffer2Vtbl {
    QueryInterface: Some(memory_2d_buffer_QueryInterface),
    AddRef: Some(memory_2d_buffer_AddRef),
    Release: Some(memory_2d_buffer_Release),
    Lock2D: Some(memory_2d_buffer_Lock2D),
    Unlock2D: Some(memory_2d_buffer_Unlock2D),
    GetScanline0AndPitch: Some(memory_2d_buffer_GetScanline0AndPitch),
    IsContiguousFormat: Some(memory_2d_buffer_IsContiguousFormat),
    GetContiguousLength: Some(memory_2d_buffer_GetContiguousLength),
    ContiguousCopyTo: Some(memory_2d_buffer_ContiguousCopyTo),
    ContiguousCopyFrom: Some(memory_2d_buffer_ContiguousCopyFrom),
    Lock2DSize: Some(memory_2d_buffer_Lock2DSize),
    Copy2DTo: Some(memory_2d_buffer_Copy2DTo),
};

/// buffer.c:873 `static const IMF2DBuffer2Vtbl d3d9_surface_buffer_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d9_surface_buffer_vtbl: IMF2DBuffer2Vtbl = IMF2DBuffer2Vtbl {
    QueryInterface: Some(memory_2d_buffer_QueryInterface),
    AddRef: Some(memory_2d_buffer_AddRef),
    Release: Some(memory_2d_buffer_Release),
    Lock2D: Some(d3d9_surface_buffer_Lock2D),
    Unlock2D: Some(d3d9_surface_buffer_Unlock2D),
    GetScanline0AndPitch: Some(d3d9_surface_buffer_GetScanline0AndPitch),
    IsContiguousFormat: Some(memory_2d_buffer_IsContiguousFormat),
    GetContiguousLength: Some(memory_2d_buffer_GetContiguousLength),
    ContiguousCopyTo: Some(memory_2d_buffer_ContiguousCopyTo),
    ContiguousCopyFrom: Some(memory_2d_buffer_ContiguousCopyFrom),
    Lock2DSize: Some(d3d9_surface_buffer_Lock2DSize),
    Copy2DTo: Some(memory_2d_buffer_Copy2DTo),
};

/// buffer.c:914 `static const IMFGetServiceVtbl memory_2d_buffer_gs_vtbl`  (slot order: rust:types_amd64.rs)
pub static memory_2d_buffer_gs_vtbl: IMFGetServiceVtbl = IMFGetServiceVtbl {
    QueryInterface: Some(memory_2d_buffer_gs_QueryInterface),
    AddRef: Some(memory_2d_buffer_gs_AddRef),
    Release: Some(memory_2d_buffer_gs_Release),
    GetService: Some(memory_2d_buffer_gs_GetService),
};

/// buffer.c:936 `static const IMFGetServiceVtbl d3d9_surface_buffer_gs_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d9_surface_buffer_gs_vtbl: IMFGetServiceVtbl = IMFGetServiceVtbl {
    QueryInterface: Some(memory_2d_buffer_gs_QueryInterface),
    AddRef: Some(memory_2d_buffer_gs_AddRef),
    Release: Some(memory_2d_buffer_gs_Release),
    GetService: Some(d3d9_surface_buffer_gs_GetService),
};

/// buffer.c:1334 `static const IMFMediaBufferVtbl dxgi_surface_1d_buffer_vtbl`  (slot order: rust:types_amd64.rs)
pub static dxgi_surface_1d_buffer_vtbl: IMFMediaBufferVtbl = IMFMediaBufferVtbl {
    QueryInterface: Some(dxgi_1d_2d_buffer_QueryInterface),
    AddRef: Some(memory_buffer_AddRef),
    Release: Some(memory_buffer_Release),
    Lock: Some(dxgi_surface_buffer_Lock),
    Unlock: Some(dxgi_surface_buffer_Unlock),
    GetCurrentLength: Some(memory_buffer_GetCurrentLength),
    SetCurrentLength: Some(dxgi_surface_buffer_SetCurrentLength),
    GetMaxLength: Some(memory_buffer_GetMaxLength),
};

/// buffer.c:1346 `static const IMF2DBuffer2Vtbl dxgi_surface_buffer_vtbl`  (slot order: rust:types_amd64.rs)
pub static dxgi_surface_buffer_vtbl: IMF2DBuffer2Vtbl = IMF2DBuffer2Vtbl {
    QueryInterface: Some(memory_2d_buffer_QueryInterface),
    AddRef: Some(memory_2d_buffer_AddRef),
    Release: Some(memory_2d_buffer_Release),
    Lock2D: Some(dxgi_surface_buffer_Lock2D),
    Unlock2D: Some(dxgi_surface_buffer_Unlock2D),
    GetScanline0AndPitch: Some(dxgi_surface_buffer_GetScanline0AndPitch),
    IsContiguousFormat: Some(memory_2d_buffer_IsContiguousFormat),
    GetContiguousLength: Some(memory_2d_buffer_GetContiguousLength),
    ContiguousCopyTo: Some(memory_2d_buffer_ContiguousCopyTo),
    ContiguousCopyFrom: Some(memory_2d_buffer_ContiguousCopyFrom),
    Lock2DSize: Some(dxgi_surface_buffer_Lock2DSize),
    Copy2DTo: Some(memory_2d_buffer_Copy2DTo),
};

/// buffer.c:1362 `static const IMFDXGIBufferVtbl dxgi_buffer_vtbl`  (slot order: rust:types_amd64.rs)
pub static dxgi_buffer_vtbl: IMFDXGIBufferVtbl = IMFDXGIBufferVtbl {
    QueryInterface: Some(dxgi_buffer_QueryInterface),
    AddRef: Some(dxgi_buffer_AddRef),
    Release: Some(dxgi_buffer_Release),
    GetResource: Some(dxgi_buffer_GetResource),
    GetSubresourceIndex: Some(dxgi_buffer_GetSubresourceIndex),
    GetUnknown: Some(dxgi_buffer_GetUnknown),
    SetUnknown: Some(dxgi_buffer_SetUnknown),
};

/// buffer.c:1765 `static const IMFMediaBufferVtbl d3d12_surface_1d_buffer_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d12_surface_1d_buffer_vtbl: IMFMediaBufferVtbl = IMFMediaBufferVtbl {
    QueryInterface: Some(dxgi_1d_2d_buffer_QueryInterface),
    AddRef: Some(memory_buffer_AddRef),
    Release: Some(memory_buffer_Release),
    Lock: Some(d3d12_surface_buffer_Lock),
    Unlock: Some(d3d12_surface_buffer_Unlock),
    GetCurrentLength: Some(d3d12_surface_buffer_GetCurrentLength),
    SetCurrentLength: Some(d3d12_surface_buffer_SetCurrentLength),
    GetMaxLength: Some(memory_buffer_GetMaxLength),
};

/// buffer.c:1777 `static const IMF2DBuffer2Vtbl d3d12_surface_buffer_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d12_surface_buffer_vtbl: IMF2DBuffer2Vtbl = IMF2DBuffer2Vtbl {
    QueryInterface: Some(memory_2d_buffer_QueryInterface),
    AddRef: Some(memory_2d_buffer_AddRef),
    Release: Some(memory_2d_buffer_Release),
    Lock2D: Some(d3d12_surface_buffer_Lock2D),
    Unlock2D: Some(d3d12_surface_buffer_Unlock2D),
    GetScanline0AndPitch: Some(d3d12_surface_buffer_GetScanline0AndPitch),
    IsContiguousFormat: Some(memory_2d_buffer_IsContiguousFormat),
    GetContiguousLength: Some(memory_2d_buffer_GetContiguousLength),
    ContiguousCopyTo: Some(memory_2d_buffer_ContiguousCopyTo),
    ContiguousCopyFrom: Some(memory_2d_buffer_ContiguousCopyFrom),
    Lock2DSize: Some(d3d12_surface_buffer_Lock2DSize),
    Copy2DTo: Some(memory_2d_buffer_Copy2DTo),
};

/// buffer.c:1793 `static const IMFDXGIBufferVtbl d3d12_surface_dxgi_buffer_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d12_surface_dxgi_buffer_vtbl: IMFDXGIBufferVtbl = IMFDXGIBufferVtbl {
    QueryInterface: Some(dxgi_buffer_QueryInterface),
    AddRef: Some(dxgi_buffer_AddRef),
    Release: Some(dxgi_buffer_Release),
    GetResource: Some(d3d12_surface_buffer_GetResource),
    GetSubresourceIndex: Some(dxgi_buffer_GetSubresourceIndex),
    GetUnknown: Some(dxgi_buffer_GetUnknown),
    SetUnknown: Some(dxgi_buffer_SetUnknown),
};

/// buffer.c:2458 `static const IMediaBufferVtbl legacy_buffer_vtbl`  (slot order: rust:types_amd64.rs)
pub static legacy_buffer_vtbl: IMediaBufferVtbl = IMediaBufferVtbl {
    QueryInterface: Some(legacy_buffer_QueryInterface),
    AddRef: Some(legacy_buffer_AddRef),
    Release: Some(legacy_buffer_Release),
    SetLength: Some(legacy_buffer_SetLength),
    GetMaxLength: Some(legacy_buffer_GetMaxLength),
    GetBufferAndLength: Some(legacy_buffer_GetBufferAndLength),
};

/// buffer.c:2546 `static const IMF2DBuffer2Vtbl legacy_buffer_IMF2DBuffer2_vtbl`  (slot order: rust:types_amd64.rs)
pub static legacy_buffer_IMF2DBuffer2_vtbl: IMF2DBuffer2Vtbl = IMF2DBuffer2Vtbl {
    QueryInterface: Some(legacy_buffer_IMF2DBuffer2_QueryInterface),
    AddRef: Some(legacy_buffer_IMF2DBuffer2_AddRef),
    Release: Some(legacy_buffer_IMF2DBuffer2_Release),
    Lock2D: Some(legacy_buffer_IMF2DBuffer2_Lock2D),
    Unlock2D: Some(legacy_buffer_IMF2DBuffer2_Unlock2D),
    GetScanline0AndPitch: Some(legacy_buffer_IMF2DBuffer2_GetScanline0AndPitch),
    IsContiguousFormat: Some(legacy_buffer_IMF2DBuffer2_IsContiguousFormat),
    GetContiguousLength: Some(legacy_buffer_IMF2DBuffer2_GetContiguousLength),
    ContiguousCopyTo: Some(legacy_buffer_IMF2DBuffer2_ContiguousCopyTo),
    ContiguousCopyFrom: Some(legacy_buffer_IMF2DBuffer2_ContiguousCopyFrom),
    Lock2DSize: Some(legacy_buffer_IMF2DBuffer2_Lock2DSize),
    Copy2DTo: Some(legacy_buffer_IMF2DBuffer2_Copy2DTo),
};

/// buffer.c:2850 `static const IMFSampleVtbl legacy_buffer_IMFSample_vtbl`  (slot order: rust:types_amd64.rs)
pub static legacy_buffer_IMFSample_vtbl: IMFSampleVtbl = IMFSampleVtbl {
    QueryInterface: Some(legacy_buffer_IMFSample_QueryInterface),
    AddRef: Some(legacy_buffer_IMFSample_AddRef),
    Release: Some(legacy_buffer_IMFSample_Release),
    GetItem: Some(legacy_buffer_IMFSample_GetItem),
    GetItemType: Some(legacy_buffer_IMFSample_GetItemType),
    CompareItem: Some(legacy_buffer_IMFSample_CompareItem),
    Compare: Some(legacy_buffer_IMFSample_Compare),
    GetUINT32: Some(legacy_buffer_IMFSample_GetUINT32),
    GetUINT64: Some(legacy_buffer_IMFSample_GetUINT64),
    GetDouble: Some(legacy_buffer_IMFSample_GetDouble),
    GetGUID: Some(legacy_buffer_IMFSample_GetGUID),
    GetStringLength: Some(legacy_buffer_IMFSample_GetStringLength),
    GetString: Some(legacy_buffer_IMFSample_GetString),
    GetAllocatedString: Some(legacy_buffer_IMFSample_GetAllocatedString),
    GetBlobSize: Some(legacy_buffer_IMFSample_GetBlobSize),
    GetBlob: Some(legacy_buffer_IMFSample_GetBlob),
    GetAllocatedBlob: Some(legacy_buffer_IMFSample_GetAllocatedBlob),
    GetUnknown: Some(legacy_buffer_IMFSample_GetUnknown),
    SetItem: Some(legacy_buffer_IMFSample_SetItem),
    DeleteItem: Some(legacy_buffer_IMFSample_DeleteItem),
    DeleteAllItems: Some(legacy_buffer_IMFSample_DeleteAllItems),
    SetUINT32: Some(legacy_buffer_IMFSample_SetUINT32),
    SetUINT64: Some(legacy_buffer_IMFSample_SetUINT64),
    SetDouble: Some(legacy_buffer_IMFSample_SetDouble),
    SetGUID: Some(legacy_buffer_IMFSample_SetGUID),
    SetString: Some(legacy_buffer_IMFSample_SetString),
    SetBlob: Some(legacy_buffer_IMFSample_SetBlob),
    SetUnknown: Some(legacy_buffer_IMFSample_SetUnknown),
    LockStore: Some(legacy_buffer_IMFSample_LockStore),
    UnlockStore: Some(legacy_buffer_IMFSample_UnlockStore),
    GetCount: Some(legacy_buffer_IMFSample_GetCount),
    GetItemByIndex: Some(legacy_buffer_IMFSample_GetItemByIndex),
    CopyAllItems: Some(legacy_buffer_IMFSample_CopyAllItems),
    GetSampleFlags: Some(legacy_buffer_IMFSample_GetSampleFlags),
    SetSampleFlags: Some(legacy_buffer_IMFSample_SetSampleFlags),
    GetSampleTime: Some(legacy_buffer_IMFSample_GetSampleTime),
    SetSampleTime: Some(legacy_buffer_IMFSample_SetSampleTime),
    GetSampleDuration: Some(legacy_buffer_IMFSample_GetSampleDuration),
    SetSampleDuration: Some(legacy_buffer_IMFSample_SetSampleDuration),
    GetBufferCount: Some(legacy_buffer_IMFSample_GetBufferCount),
    GetBufferByIndex: Some(legacy_buffer_IMFSample_GetBufferByIndex),
    ConvertToContiguousBuffer: Some(legacy_buffer_IMFSample_ConvertToContiguousBuffer),
    AddBuffer: Some(legacy_buffer_IMFSample_AddBuffer),
    RemoveBufferByIndex: Some(legacy_buffer_IMFSample_RemoveBufferByIndex),
    RemoveAllBuffers: Some(legacy_buffer_IMFSample_RemoveAllBuffers),
    GetTotalLength: Some(legacy_buffer_IMFSample_GetTotalLength),
    CopyToBuffer: Some(legacy_buffer_IMFSample_CopyToBuffer),
};

/// buffer.c:3069 `static const IMFMediaBufferVtbl buffer_wrapper_vtbl`  (slot order: rust:types_amd64.rs)
pub static buffer_wrapper_vtbl: IMFMediaBufferVtbl = IMFMediaBufferVtbl {
    QueryInterface: Some(buffer_wrapper_QueryInterface),
    AddRef: Some(buffer_wrapper_AddRef),
    Release: Some(buffer_wrapper_Release),
    Lock: Some(buffer_wrapper_Lock),
    Unlock: Some(buffer_wrapper_Unlock),
    GetCurrentLength: Some(buffer_wrapper_GetCurrentLength),
    SetCurrentLength: Some(buffer_wrapper_SetCurrentLength),
    GetMaxLength: Some(buffer_wrapper_GetMaxLength),
};
