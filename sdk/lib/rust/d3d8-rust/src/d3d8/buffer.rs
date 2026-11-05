use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)

static mut d3d8_vertexbuffer_wined3d_parent_ops: wined3d_parent_ops = unsafe {
    wined3d_parent_ops {
        wined3d_object_destroyed: Some(
            d3d8_vertexbuffer_wined3d_object_destroyed,
        ),
    }
};

static mut d3d8_indexbuffer_wined3d_parent_ops: wined3d_parent_ops = unsafe {
    wined3d_parent_ops {
        wined3d_object_destroyed: Some(
            d3d8_indexbuffer_wined3d_object_destroyed,
        ),
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(d3d8): debug channel placeholder

pub type Pwined3d_parent_ops = *mut wined3d_parent_ops;
unsafe extern "C" fn impl_from_IDirect3DVertexBuffer8(iface: *mut IDirect3DVertexBuffer8) -> *mut d3d8_vertexbuffer {
    return CONTAINING_RECORD!(iface, d3d8_vertexbuffer, IDirect3DVertexBuffer8_iface);
}

unsafe extern "system" fn d3d8_vertexbuffer_QueryInterface(iface: *mut IDirect3DVertexBuffer8, riid: REFIID, object: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DVertexBuffer8)) != false
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DResource8)) != false
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) != false
    {
        IDirect3DVertexBuffer8_AddRef(iface);
        *object = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    WARN!("%s not implemented, returning E_NOINTERFACE.\n", debugstr_guid(riid as *const _GUID));

    *object = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn d3d8_vertexbuffer_AddRef(iface: *mut IDirect3DVertexBuffer8) -> ULONG {
    let buffer: *mut d3d8_vertexbuffer = impl_from_IDirect3DVertexBuffer8(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*buffer).resource.refcount) as *mut LONG) as ULONG;

    if refcount == 1 {
        IDirect3DDevice8_AddRef((*buffer).parent_device);
        if !(*buffer).draw_buffer.is_null() {
            wined3d_buffer_incref((*buffer).draw_buffer);
        } else {
            wined3d_buffer_incref((*buffer).wined3d_buffer);
        }
    }

    return refcount;
}

unsafe extern "system" fn d3d8_vertexbuffer_Release(iface: *mut IDirect3DVertexBuffer8) -> ULONG {
    let buffer: *mut d3d8_vertexbuffer = impl_from_IDirect3DVertexBuffer8(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*buffer).resource.refcount) as *mut LONG) as ULONG;

    TRACE!("%p decreasing refcount to %lu.\n", iface, refcount);

    if refcount == 0 {
        let draw_buffer: *mut wined3d_buffer = (*buffer).draw_buffer;
        let device: *mut IDirect3DDevice8 = (*buffer).parent_device;

        if !draw_buffer.is_null() {
            wined3d_buffer_decref(draw_buffer);
        } else {
            wined3d_buffer_decref((*buffer).wined3d_buffer);
        }

        /* Release the device last, as it may cause the device to be destroyed. */
        IDirect3DDevice8_Release(device);
    }

    return refcount;
}

unsafe extern "system" fn d3d8_vertexbuffer_GetDevice(iface: *mut IDirect3DVertexBuffer8, device: *mut *mut IDirect3DDevice8) -> HRESULT {
    let mut buffer: *mut d3d8_vertexbuffer = impl_from_IDirect3DVertexBuffer8(iface);

    *device = (*buffer).parent_device;
    IDirect3DDevice8_AddRef(*device);

    return D3D_OK;
}

unsafe extern "system" fn d3d8_vertexbuffer_SetPrivateData(
    iface: *mut IDirect3DVertexBuffer8,
    guid: *const GUID,
    data: *const core::ffi::c_void,
    data_size: DWORD,
    flags: DWORD,
) -> HRESULT {
    let buffer: *mut d3d8_vertexbuffer = impl_from_IDirect3DVertexBuffer8(iface);
    TRACE!(
        "iface %p, guid %s, data %p, data_size %lu, flags %#lx.\n",
        iface,
        debugstr_guid(guid),
        data,
        data_size,
        flags
    );
    return d3d8_resource_set_private_data(
        core::ptr::addr_of_mut!((*buffer).resource),
        guid,
        data,
        data_size,
        flags,
    );
}
unsafe extern "system" fn d3d8_vertexbuffer_GetPrivateData(
    mut iface: *mut IDirect3DVertexBuffer8,
    guid: *const GUID,
    mut data: *mut ::core::ffi::c_void,
    mut data_size: *mut DWORD,
) -> HRESULT {
    let mut buffer: *mut d3d8_vertexbuffer = impl_from_IDirect3DVertexBuffer8(iface);
    WINE_TRACE!("iface %p, guid %s, data %p, data_size %p.\n", iface, debugstr_guid(guid as *const _GUID), data, data_size);
    return d3d8_resource_get_private_data(
        core::ptr::addr_of_mut!((*buffer).resource),
        guid,
        data,
        data_size,
    );
}

unsafe extern "system" fn d3d8_vertexbuffer_FreePrivateData(iface: *mut IDirect3DVertexBuffer8, guid: REFGUID) -> HRESULT {
    let mut buffer: *mut d3d8_vertexbuffer = impl_from_IDirect3DVertexBuffer8(iface);

    return d3d8_resource_free_private_data(&raw mut (*buffer).resource, guid);
}

unsafe extern "system" fn d3d8_vertexbuffer_SetPriority(iface: *mut IDirect3DVertexBuffer8, priority: DWORD) -> DWORD {
    let buffer: *mut d3d8_vertexbuffer = impl_from_IDirect3DVertexBuffer8(iface);
    let mut resource: *mut wined3d_resource;
    let mut previous: DWORD;

    wined3d_mutex_lock();
    resource = wined3d_buffer_get_resource((*buffer).wined3d_buffer);
    previous = wined3d_resource_set_priority(resource, priority) as DWORD;
    wined3d_mutex_unlock();

    return previous;
}
unsafe extern "system" fn d3d8_vertexbuffer_GetPriority(
    mut iface: *mut IDirect3DVertexBuffer8,
) -> DWORD {
    let mut buffer: *mut d3d8_vertexbuffer = impl_from_IDirect3DVertexBuffer8(iface);
    let mut resource: *const wined3d_resource = ::core::ptr::null::<wined3d_resource>();
    let mut priority: DWORD = 0;
    WINE_TRACE!("iface %p.\n", iface);
    wined3d_mutex_lock();
    resource = wined3d_buffer_get_resource((*buffer).wined3d_buffer);
    priority = wined3d_resource_get_priority(resource) as DWORD;
    wined3d_mutex_unlock();
    return priority;
}

unsafe extern "system" fn d3d8_vertexbuffer_PreLoad(iface: *mut IDirect3DVertexBuffer8) {
    let buffer: *mut d3d8_vertexbuffer = impl_from_IDirect3DVertexBuffer8(iface);

    WINE_TRACE!("iface %p.\n", iface);
    wined3d_mutex_lock();
    wined3d_resource_preload(wined3d_buffer_get_resource((*buffer).wined3d_buffer));
    wined3d_mutex_unlock();
}
unsafe extern "system" fn d3d8_vertexbuffer_GetType(
    iface: *mut IDirect3DVertexBuffer8,
) -> D3DRESOURCETYPE {
    // TRACE("iface %p.\n", iface);
    D3DRTYPE_VERTEXBUFFER
}

unsafe extern "system" fn d3d8_vertexbuffer_Lock(
    iface: *mut IDirect3DVertexBuffer8,
    offset: UINT,
    size: UINT,
    data: *mut *mut BYTE,
    mut flags: DWORD,
) -> HRESULT {
    TRACE!("iface %p, offset %u, size %u, data %p, flags %#lx.\n", iface, offset, size, data, flags);
    let buffer: *mut d3d8_vertexbuffer = impl_from_IDirect3DVertexBuffer8(iface);
    let mut wined3d_resource: *mut wined3d_resource;
    let mut wined3d_map_desc: wined3d_map_desc = core::mem::zeroed();
    let mut wined3d_box: wined3d_box = core::mem::zeroed();
    let mut hr: HRESULT;

    if (*buffer).discarded {
        WARN!("Filtering out redundant discard of %p.\n", buffer);
        flags = (flags & !D3DLOCK_DISCARD) | D3DLOCK_NOOVERWRITE;
    }
    if (flags & D3DLOCK_DISCARD) != 0 {
        (*buffer).discarded = true;
    }

    wined3d_box.left = offset;
    wined3d_box.right = offset.wrapping_add(size);
    wined3d_resource = wined3d_buffer_get_resource((*buffer).wined3d_buffer);
    hr = wined3d_resource_map(
        wined3d_resource,
        0,
        core::ptr::addr_of_mut!(wined3d_map_desc),
        core::ptr::addr_of_mut!(wined3d_box),
        wined3dmapflags_from_d3dmapflags(flags, (*buffer).usage),
    );
    *data = wined3d_map_desc.data as *mut BYTE;

    return hr;
}
unsafe extern "system" fn d3d8_vertexbuffer_Unlock(mut iface: *mut IDirect3DVertexBuffer8) -> HRESULT {
    let mut buffer: *mut d3d8_vertexbuffer = impl_from_IDirect3DVertexBuffer8(iface);
    WINE_TRACE!("iface %p.\n", iface);
    wined3d_resource_unmap(
        wined3d_buffer_get_resource((*buffer).wined3d_buffer),
        0,
    );
    return D3D_OK;
}

unsafe extern "system" fn d3d8_vertexbuffer_GetDesc(
    mut iface: *mut IDirect3DVertexBuffer8,
    mut desc: *mut D3DVERTEXBUFFER_DESC,
) -> HRESULT {
    let mut buffer: *mut d3d8_vertexbuffer = impl_from_IDirect3DVertexBuffer8(iface);
    let mut wined3d_desc: wined3d_resource_desc = wined3d_resource_desc {
        resource_type: WINED3D_RTYPE_NONE,
        format: WINED3DFMT_UNKNOWN,
        multisample_type: WINED3D_MULTISAMPLE_NONE,
        multisample_quality: 0,
        usage: 0,
        bind_flags: 0,
        access: 0,
        width: 0,
        height: 0,
        depth: 0,
        size: 0,
    };
    let mut wined3d_resource: *mut wined3d_resource = core::ptr::null_mut::<wined3d_resource>();
    WINE_TRACE!("iface %p, desc %p.\n", iface, desc);
    wined3d_mutex_lock();
    wined3d_resource = wined3d_buffer_get_resource((*buffer).wined3d_buffer);
    wined3d_resource_get_desc(wined3d_resource, core::ptr::addr_of_mut!(wined3d_desc));
    wined3d_mutex_unlock();

    (*desc).Format = D3DFMT_VERTEXDATA;
    (*desc).Type = D3DRTYPE_VERTEXBUFFER;
    (*desc).Usage = (*buffer).usage;
    (*desc).Pool = d3dpool_from_wined3daccess(wined3d_desc.access, wined3d_desc.usage);
    (*desc).Size = wined3d_desc.size as UINT;
    (*desc).FVF = (*buffer).fvf;

    return D3D_OK;
}
unsafe extern "system" fn d3d8_vertexbuffer_wined3d_object_destroyed(parent: *mut core::ffi::c_void) {
    let buffer: *mut d3d8_vertexbuffer = parent as *mut d3d8_vertexbuffer;
    if !(*buffer).draw_buffer.is_null() {
        wined3d_buffer_decref((*buffer).wined3d_buffer);
    }
    d3d8_resource_cleanup(core::ptr::addr_of_mut!((*buffer).resource));
    free(buffer as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn vertexbuffer_init(
    mut buffer: *mut d3d8_vertexbuffer,
    mut device: *mut d3d8_device,
    mut size: UINT,
    mut usage: DWORD,
    mut fvf: DWORD,
    mut pool: D3DPOOL,
) -> HRESULT {
    let mut parent_ops: *const wined3d_parent_ops = &raw const d3d8_null_wined3d_parent_ops;
    let mut desc: wined3d_buffer_desc = wined3d_buffer_desc {
        byte_width: 0,
        usage: 0,
        bind_flags: 0,
        access: 0,
        misc_flags: 0,
        structure_byte_stride: 0,
    };
    let mut hr: HRESULT = 0;

    if pool == D3DPOOL_SCRATCH {
        WARN!("Vertex buffer with D3DPOOL_SCRATCH requested.\n");
        return D3DERR_INVALIDCALL;
    }

    /* In d3d8, buffers can't be used as rendertarget or depth/stencil buffer. */
    if (usage & (D3DUSAGE_RENDERTARGET | D3DUSAGE_DEPTHSTENCIL)) != 0 {
        return D3DERR_INVALIDCALL;
    }

    (*buffer).IDirect3DVertexBuffer8_iface.lpVtbl = &raw const Direct3DVertexBuffer8_Vtbl;
    d3d8_resource_init(&raw mut (*buffer).resource);
    (*buffer).fvf = fvf;
    (*buffer).usage = usage;

    desc.byte_width = size;
    desc.usage = wined3d_usage_from_d3d(pool, usage);
    desc.bind_flags = 0;
    desc.access = wined3daccess_from_d3dpool(pool, usage) | map_access_from_usage(usage);
    /* Buffers are always readable. */
    if pool != D3DPOOL_DEFAULT {
        desc.access |= WINED3D_RESOURCE_ACCESS_MAP_R | WINED3D_RESOURCE_ACCESS_MAP_W;
    }
    desc.misc_flags = 0;
    desc.structure_byte_stride = 0;

    if (desc.access & WINED3D_RESOURCE_ACCESS_GPU) != 0 {
        desc.bind_flags = WINED3D_BIND_VERTEX_BUFFER;
        parent_ops = &raw const d3d8_vertexbuffer_wined3d_parent_ops;
    }

    wined3d_mutex_lock();
    hr = wined3d_buffer_create(
        (*device).wined3d_device,
        &raw mut desc,
        ::core::ptr::null(),
        buffer as *mut ::core::ffi::c_void,
        parent_ops,
        &raw mut (*buffer).wined3d_buffer,
    );
    if (hr >= 0) && ((desc.access & WINED3D_RESOURCE_ACCESS_GPU) == 0) {
        desc.bind_flags = WINED3D_BIND_VERTEX_BUFFER;
        desc.access = WINED3D_RESOURCE_ACCESS_GPU;
        hr = wined3d_buffer_create(
            (*device).wined3d_device,
            &raw mut desc,
            ::core::ptr::null(),
            buffer as *mut ::core::ffi::c_void,
            &raw const d3d8_vertexbuffer_wined3d_parent_ops,
            &raw mut (*buffer).draw_buffer,
        );
        if hr < 0 {
            wined3d_buffer_decref((*buffer).wined3d_buffer);
        }
    }
    wined3d_mutex_unlock();
    if hr < 0 {
        WARN!("Failed to create wined3d buffer, hr %#lx.\n", hr);
        return hr;
    }

    (*buffer).parent_device = &raw mut (*device).IDirect3DDevice8_iface;
    IDirect3DDevice8_AddRef((*buffer).parent_device);

    return D3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn unsafe_impl_from_IDirect3DVertexBuffer8(iface: *mut IDirect3DVertexBuffer8) -> *mut d3d8_vertexbuffer {
    if iface.is_null() {
        return core::ptr::null_mut();
    }
    assert!(
        (*iface).lpVtbl as *const _ == core::ptr::addr_of!(Direct3DVertexBuffer8_Vtbl),
    );

    return impl_from_IDirect3DVertexBuffer8(iface);
}

unsafe extern "C" fn impl_from_IDirect3DIndexBuffer8(iface: *mut IDirect3DIndexBuffer8) -> *mut d3d8_indexbuffer {
    return CONTAINING_RECORD!(iface, d3d8_indexbuffer, IDirect3DIndexBuffer8_iface) as *mut d3d8_indexbuffer;
}

unsafe extern "system" fn d3d8_indexbuffer_QueryInterface(iface: *mut IDirect3DIndexBuffer8, riid: REFIID, object: *mut *mut core::ffi::c_void) -> HRESULT {

    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DIndexBuffer8))
            || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DResource8))
            || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown))
    {
        IDirect3DIndexBuffer8_AddRef(iface);
        *object = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    WARN!("%s not implemented, returning E_NOINTERFACE.\n", debugstr_guid(riid));

    *object = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn d3d8_indexbuffer_AddRef(iface: *mut IDirect3DIndexBuffer8) -> ULONG {
    let buffer: *mut d3d8_indexbuffer = impl_from_IDirect3DIndexBuffer8(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*buffer).resource.refcount) as *mut LONG) as ULONG;

    if refcount == 1 {
        IDirect3DDevice8_AddRef((*buffer).parent_device);
        wined3d_buffer_incref((*buffer).wined3d_buffer);
    }

    return refcount;
}

unsafe extern "system" fn d3d8_indexbuffer_Release(iface: *mut IDirect3DIndexBuffer8) -> ULONG {
    let buffer: *mut d3d8_indexbuffer = impl_from_IDirect3DIndexBuffer8(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*buffer).resource.refcount) as *mut core::ffi::c_long,
    ) as ULONG;

    if refcount == 0 {
        let device: *mut IDirect3DDevice8 = (*buffer).parent_device;

        wined3d_buffer_decref((*buffer).wined3d_buffer);

        /* Release the device last, as it may cause the device to be destroyed. */
        IDirect3DDevice8_Release(device);
    }

    return refcount;
}

unsafe extern "system" fn d3d8_indexbuffer_GetDevice(iface: *mut IDirect3DIndexBuffer8, device: *mut *mut IDirect3DDevice8) -> HRESULT {
    let buffer: *mut d3d8_indexbuffer = impl_from_IDirect3DIndexBuffer8(iface);

    *device = (*buffer).parent_device;
    IDirect3DDevice8_AddRef(*device);

    return D3D_OK;
}

unsafe extern "system" fn d3d8_indexbuffer_SetPrivateData(
    iface: *mut IDirect3DIndexBuffer8,
    guid: *const GUID,
    data: *const core::ffi::c_void,
    data_size: DWORD,
    flags: DWORD,
) -> HRESULT {
    let buffer: *mut d3d8_indexbuffer = impl_from_IDirect3DIndexBuffer8(iface);
    TRACE!(
        "iface %p, guid %s, data %p, data_size %lu, flags %#lx.\n",
        iface,
        debugstr_guid(guid),
        data,
        data_size,
        flags
    );

    return d3d8_resource_set_private_data(
        core::ptr::addr_of_mut!((*buffer).resource),
        guid,
        data,
        data_size,
        flags,
    );
}
unsafe extern "system" fn d3d8_indexbuffer_GetPrivateData(
    mut iface: *mut IDirect3DIndexBuffer8,
    guid: *const GUID,
    mut data: *mut ::core::ffi::c_void,
    mut data_size: *mut DWORD,
) -> HRESULT {
    let mut buffer: *mut d3d8_indexbuffer = impl_from_IDirect3DIndexBuffer8(iface);
    WINE_TRACE!("iface %p, guid %s, data %p, data_size %p.\n", iface, debugstr_guid(guid as *const _GUID), data, data_size);
    return d3d8_resource_get_private_data(
        core::ptr::addr_of_mut!((*buffer).resource),
        guid,
        data,
        data_size,
    );
}
unsafe extern "system" fn d3d8_indexbuffer_FreePrivateData(iface: *mut IDirect3DIndexBuffer8, guid: *const GUID) -> HRESULT {
    let mut buffer: *mut d3d8_indexbuffer = impl_from_IDirect3DIndexBuffer8(iface);
    WINE_TRACE!("iface %p, guid %s.\n", iface, debugstr_guid(guid as *const _GUID));
    return d3d8_resource_free_private_data(core::ptr::addr_of_mut!((*buffer).resource), guid);
}

unsafe extern "system" fn d3d8_indexbuffer_SetPriority(
    mut iface: *mut IDirect3DIndexBuffer8,
    mut priority: DWORD,
) -> DWORD {
    let mut buffer: *mut d3d8_indexbuffer = impl_from_IDirect3DIndexBuffer8(iface);
    let mut resource: *mut wined3d_resource = core::ptr::null_mut::<wined3d_resource>();
    let mut previous: DWORD = 0;

    TRACE!("iface %p, priority %lu.\n", iface, priority);
    wined3d_mutex_lock();
    resource = wined3d_buffer_get_resource((*buffer).wined3d_buffer);
    previous = wined3d_resource_set_priority(resource, priority as ::core::ffi::c_uint) as DWORD;
    wined3d_mutex_unlock();

    return previous;
}
unsafe extern "system" fn d3d8_indexbuffer_GetPriority(iface: *mut IDirect3DIndexBuffer8) -> DWORD {
    let buffer: *mut d3d8_indexbuffer = impl_from_IDirect3DIndexBuffer8(iface);
    let resource: *const wined3d_resource;
    let priority: DWORD;

    wined3d_mutex_lock();
    resource = wined3d_buffer_get_resource((*buffer).wined3d_buffer);
    priority = wined3d_resource_get_priority(resource) as DWORD;
    wined3d_mutex_unlock();

    return priority;
}

unsafe extern "system" fn d3d8_indexbuffer_PreLoad(iface: *mut IDirect3DIndexBuffer8) {
    let buffer: *mut d3d8_indexbuffer = impl_from_IDirect3DIndexBuffer8(iface);
    WINE_TRACE!("iface %p.\n", iface);
    wined3d_mutex_lock();
    wined3d_resource_preload(wined3d_buffer_get_resource((*buffer).wined3d_buffer));
    wined3d_mutex_unlock();
}
unsafe extern "system" fn d3d8_indexbuffer_GetType(
    iface: *mut IDirect3DIndexBuffer8,
) -> D3DRESOURCETYPE {
    // TRACE("iface %p.\n", iface);
    D3DRTYPE_INDEXBUFFER
}
unsafe extern "system" fn d3d8_indexbuffer_Lock(iface: *mut IDirect3DIndexBuffer8, offset: UINT, size: UINT, data: *mut *mut BYTE, mut flags: DWORD) -> HRESULT {
    let buffer: *mut d3d8_indexbuffer = impl_from_IDirect3DIndexBuffer8(iface);
    let mut wined3d_resource: *mut wined3d_resource;
    let mut wined3d_map_desc: wined3d_map_desc = core::mem::zeroed();
    let mut wined3d_box: wined3d_box = core::mem::zeroed();
    let mut hr: HRESULT;

    if (*buffer).discarded {
        WARN!("Filtering out redundant discard of %p.\n", buffer);
        flags = (flags & !D3DLOCK_DISCARD) | D3DLOCK_NOOVERWRITE;
    }
    if (flags & D3DLOCK_DISCARD) != 0 {
        (*buffer).discarded = true;
    }

    wined3d_box.left = offset;
    wined3d_box.right = offset.wrapping_add(size);
    wined3d_resource = wined3d_buffer_get_resource((*buffer).wined3d_buffer);
    hr = wined3d_resource_map(
        wined3d_resource,
        0,
        core::ptr::addr_of_mut!(wined3d_map_desc),
        core::ptr::addr_of_mut!(wined3d_box),
        wined3dmapflags_from_d3dmapflags(flags, (*buffer).usage),
    );
    *data = wined3d_map_desc.data as *mut BYTE;

    return hr;
}

unsafe extern "system" fn d3d8_indexbuffer_Unlock(iface: *mut IDirect3DIndexBuffer8) -> HRESULT {
    let buffer: *mut d3d8_indexbuffer = impl_from_IDirect3DIndexBuffer8(iface);
    TRACE!("iface %p.\n", iface);
    wined3d_resource_unmap(wined3d_buffer_get_resource((*buffer).wined3d_buffer), 0);
    return D3D_OK;
}

unsafe extern "system" fn d3d8_indexbuffer_GetDesc(
    mut iface: *mut IDirect3DIndexBuffer8,
    mut desc: *mut D3DINDEXBUFFER_DESC,
) -> HRESULT {
    let mut buffer: *mut d3d8_indexbuffer = impl_from_IDirect3DIndexBuffer8(iface);
    let mut wined3d_desc: wined3d_resource_desc = wined3d_resource_desc {
        resource_type: WINED3D_RTYPE_NONE,
        format: WINED3DFMT_UNKNOWN,
        multisample_type: WINED3D_MULTISAMPLE_NONE,
        multisample_quality: 0,
        usage: 0,
        bind_flags: 0,
        access: 0,
        width: 0,
        height: 0,
        depth: 0,
        size: 0,
    };
    let mut wined3d_resource: *mut wined3d_resource = ::core::ptr::null_mut::<wined3d_resource>();
    WINE_TRACE!("iface %p, desc %p.\n", iface, desc);
    wined3d_mutex_lock();
    wined3d_resource = wined3d_buffer_get_resource((*buffer).wined3d_buffer);
    wined3d_resource_get_desc(wined3d_resource, core::ptr::addr_of_mut!(wined3d_desc));
    wined3d_mutex_unlock();
    (*desc).Format = d3dformat_from_wined3dformat((*buffer).format);
    (*desc).Type = D3DRTYPE_INDEXBUFFER;
    (*desc).Usage = (*buffer).usage;
    (*desc).Pool = d3dpool_from_wined3daccess(wined3d_desc.access, wined3d_desc.usage);
    (*desc).Size = wined3d_desc.size as UINT;
    return D3D_OK;
}
unsafe extern "system" fn d3d8_indexbuffer_wined3d_object_destroyed(
    mut parent: *mut core::ffi::c_void,
) {
    let mut buffer: *mut d3d8_indexbuffer = parent as *mut d3d8_indexbuffer;
    d3d8_resource_cleanup(core::ptr::addr_of_mut!((*buffer).resource));
    free(buffer as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn indexbuffer_init(buffer: *mut d3d8_indexbuffer, device: *mut d3d8_device, size: UINT, usage: DWORD, format: D3DFORMAT, pool: D3DPOOL) -> HRESULT {
    let mut desc: wined3d_buffer_desc = core::mem::zeroed();
    let mut hr: HRESULT = 0;

    if pool == D3DPOOL_SCRATCH {
        return D3DERR_INVALIDCALL;
    }

    /* In d3d8, buffers can't be used as rendertarget or depth/stencil buffer. */
    if usage & (D3DUSAGE_RENDERTARGET | D3DUSAGE_DEPTHSTENCIL) != 0 {
        return D3DERR_INVALIDCALL;
    }

    desc.byte_width = size;
    desc.usage = wined3d_usage_from_d3d(pool, usage) | WINED3DUSAGE_STATICDECL;
    desc.bind_flags = 0;
    desc.access = wined3daccess_from_d3dpool(pool, usage) | map_access_from_usage(usage);
    /* Buffers are always readable. */
    if pool != D3DPOOL_DEFAULT {
        desc.access |= WINED3D_RESOURCE_ACCESS_MAP_R | WINED3D_RESOURCE_ACCESS_MAP_W;
    }
    desc.misc_flags = 0;
    desc.structure_byte_stride = 0;

    if desc.access & WINED3D_RESOURCE_ACCESS_GPU != 0 {
        desc.bind_flags = WINED3D_BIND_INDEX_BUFFER;
    }

    (*buffer).IDirect3DIndexBuffer8_iface.lpVtbl = core::ptr::addr_of!(d3d8_indexbuffer_vtbl) as *const IDirect3DIndexBuffer8Vtbl;
    d3d8_resource_init(core::ptr::addr_of_mut!((*buffer).resource));
    (*buffer).format = wined3dformat_from_d3dformat(format);
    (*buffer).usage = usage;
    (*buffer).sysmem = ((((desc.access & WINED3D_RESOURCE_ACCESS_GPU) == 0) as BOOL) != 0);

    wined3d_mutex_lock();
    hr = wined3d_buffer_create(
        (*device).wined3d_device,
        core::ptr::addr_of_mut!(desc),
        core::ptr::null(),
        (buffer).cast(),
        core::ptr::addr_of!(d3d8_indexbuffer_wined3d_parent_ops),
        core::ptr::addr_of_mut!((*buffer).wined3d_buffer),
    );
    wined3d_mutex_unlock();
    if hr < 0 {
        WARN!("Failed to create wined3d buffer, hr %#lx.\n", hr);
        return hr;
    }

    (*buffer).parent_device = core::ptr::addr_of_mut!((*device).IDirect3DDevice8_iface);
    IDirect3DDevice8_AddRef((*buffer).parent_device);

    return D3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn unsafe_impl_from_IDirect3DIndexBuffer8(mut iface: *mut IDirect3DIndexBuffer8) -> *mut d3d8_indexbuffer {
    if iface.is_null() {
        return core::ptr::null_mut();
    }
    assert!((*iface).lpVtbl == core::ptr::addr_of!(d3d8_indexbuffer_vtbl));

    return impl_from_IDirect3DIndexBuffer8(iface);
}

// COM vtable tables — generated from buffer.c by vtbl_gen.py
/// buffer.c:248 `static const IDirect3DVertexBuffer8Vtbl Direct3DVertexBuffer8_Vtbl`  (slot order: rust:types_amd64.rs)
pub static Direct3DVertexBuffer8_Vtbl: IDirect3DVertexBuffer8Vtbl = IDirect3DVertexBuffer8Vtbl {
    QueryInterface: Some(d3d8_vertexbuffer_QueryInterface),
    AddRef: Some(d3d8_vertexbuffer_AddRef),
    Release: Some(d3d8_vertexbuffer_Release),
    GetDevice: Some(d3d8_vertexbuffer_GetDevice),
    SetPrivateData: Some(d3d8_vertexbuffer_SetPrivateData),
    GetPrivateData: Some(d3d8_vertexbuffer_GetPrivateData),
    FreePrivateData: Some(d3d8_vertexbuffer_FreePrivateData),
    SetPriority: Some(d3d8_vertexbuffer_SetPriority),
    GetPriority: Some(d3d8_vertexbuffer_GetPriority),
    PreLoad: Some(d3d8_vertexbuffer_PreLoad),
    GetType: Some(d3d8_vertexbuffer_GetType),
    Lock: Some(d3d8_vertexbuffer_Lock),
    Unlock: Some(d3d8_vertexbuffer_Unlock),
    GetDesc: Some(d3d8_vertexbuffer_GetDesc),
};

/// buffer.c:571 `static const IDirect3DIndexBuffer8Vtbl d3d8_indexbuffer_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d8_indexbuffer_vtbl: IDirect3DIndexBuffer8Vtbl = IDirect3DIndexBuffer8Vtbl {
    QueryInterface: Some(d3d8_indexbuffer_QueryInterface),
    AddRef: Some(d3d8_indexbuffer_AddRef),
    Release: Some(d3d8_indexbuffer_Release),
    GetDevice: Some(d3d8_indexbuffer_GetDevice),
    SetPrivateData: Some(d3d8_indexbuffer_SetPrivateData),
    GetPrivateData: Some(d3d8_indexbuffer_GetPrivateData),
    FreePrivateData: Some(d3d8_indexbuffer_FreePrivateData),
    SetPriority: Some(d3d8_indexbuffer_SetPriority),
    GetPriority: Some(d3d8_indexbuffer_GetPriority),
    PreLoad: Some(d3d8_indexbuffer_PreLoad),
    GetType: Some(d3d8_indexbuffer_GetType),
    Lock: Some(d3d8_indexbuffer_Lock),
    Unlock: Some(d3d8_indexbuffer_Unlock),
    GetDesc: Some(d3d8_indexbuffer_GetDesc),
};
