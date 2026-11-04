use crate::d3d9::device::d3d9_null_wined3d_parent_ops;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)

static mut d3d9_vertexbuffer_wined3d_parent_ops: wined3d_parent_ops = unsafe {
    wined3d_parent_ops {
        wined3d_object_destroyed: Some(
            d3d9_vertexbuffer_wined3d_object_destroyed,
        ),
    }
};

static mut d3d9_indexbuffer_wined3d_parent_ops: wined3d_parent_ops = unsafe {
    wined3d_parent_ops {
        wined3d_object_destroyed: Some(
            d3d9_indexbuffer_wined3d_object_destroyed,
        ),
    }
};


unsafe extern "C" fn impl_from_IDirect3DVertexBuffer9(
    iface: *mut IDirect3DVertexBuffer9,
) -> *mut d3d9_vertexbuffer {
    return (iface as *mut u8).sub(core::mem::offset_of!(d3d9_vertexbuffer, IDirect3DVertexBuffer9_iface))
        as *mut d3d9_vertexbuffer;
}

unsafe extern "system" fn d3d9_vertexbuffer_QueryInterface(iface: *mut IDirect3DVertexBuffer9, riid: *const GUID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DVertexBuffer9)) != false
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DResource9)) != false
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) != false
    {
        IDirect3DVertexBuffer9_AddRef(iface);
        *out = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    WARN!("%s not implemented, returning E_NOINTERFACE.\n", debugstr_guid(riid));

    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn d3d9_vertexbuffer_AddRef(iface: *mut IDirect3DVertexBuffer9) -> ULONG {
    let buffer: *mut d3d9_vertexbuffer = impl_from_IDirect3DVertexBuffer9(iface);
    let refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*buffer).resource.refcount) as *mut core::ffi::c_long,
    ) as ULONG;
    WINE_TRACE!("%p increasing refcount to %lu.\n", iface, refcount);

    if refcount == 1 {
        IDirect3DDevice9Ex_AddRef((*buffer).parent_device);
        if !(*buffer).draw_buffer.is_null() {
            wined3d_buffer_incref((*buffer).draw_buffer);
        } else {
            wined3d_buffer_incref((*buffer).wined3d_buffer);
        }
    }

    return refcount;
}
unsafe extern "system" fn d3d9_vertexbuffer_Release(mut iface: *mut IDirect3DVertexBuffer9) -> ULONG {
    let mut buffer: *mut d3d9_vertexbuffer = impl_from_IDirect3DVertexBuffer9(iface);
    let mut refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*buffer).resource.refcount) as *mut LONG,
    ) as ULONG;
    WINE_TRACE!("%p decreasing refcount to %lu.\n", iface, refcount);
    if refcount == 0 {
        let mut draw_buffer: *mut wined3d_buffer = (*buffer).draw_buffer;
        let mut device: *mut IDirect3DDevice9Ex = (*buffer).parent_device;
        if !draw_buffer.is_null() {
            wined3d_buffer_decref(draw_buffer);
        } else {
            wined3d_buffer_decref((*buffer).wined3d_buffer);
        }
        /* Release the device last, as it may cause the device to be destroyed. */
        IDirect3DDevice9Ex_Release(device);
    }
    return refcount;
}

unsafe extern "system" fn d3d9_vertexbuffer_GetDevice(iface: *mut IDirect3DVertexBuffer9, device: *mut *mut IDirect3DDevice9) -> HRESULT {
    let mut buffer: *mut d3d9_vertexbuffer = impl_from_IDirect3DVertexBuffer9(iface);
    *device = (*buffer).parent_device as *mut IDirect3DDevice9;
    IDirect3DDevice9_AddRef(*device);
    return D3D_OK;
}

unsafe extern "system" fn d3d9_vertexbuffer_SetPrivateData(iface: *mut IDirect3DVertexBuffer9, guid: REFGUID, data: *const core::ffi::c_void, data_size: DWORD, flags: DWORD) -> HRESULT {
    let buffer: *mut d3d9_vertexbuffer = impl_from_IDirect3DVertexBuffer9(iface);

    return d3d9_resource_set_private_data(core::ptr::addr_of_mut!((*buffer).resource), guid, data, data_size, flags);
}
unsafe extern "system" fn d3d9_vertexbuffer_GetPrivateData(
    iface: *mut IDirect3DVertexBuffer9,
    guid: *const GUID,
    data: *mut core::ffi::c_void,
    data_size: *mut DWORD,
) -> HRESULT {
    let buffer: *mut d3d9_vertexbuffer = impl_from_IDirect3DVertexBuffer9(iface);

    return d3d9_resource_get_private_data(core::ptr::addr_of_mut!((*buffer).resource), guid, data, data_size);
}

unsafe extern "system" fn d3d9_vertexbuffer_FreePrivateData(iface: *mut IDirect3DVertexBuffer9, guid: REFGUID) -> HRESULT {
    let mut buffer: *mut d3d9_vertexbuffer = impl_from_IDirect3DVertexBuffer9(iface);
    WINE_TRACE!("iface %p, guid %s.\n", iface, debugstr_guid(guid as *const _GUID));
    return d3d9_resource_free_private_data(core::ptr::addr_of_mut!((*buffer).resource), guid);
}
unsafe extern "system" fn d3d9_vertexbuffer_SetPriority(
    mut iface: *mut IDirect3DVertexBuffer9,
    mut priority: DWORD,
) -> DWORD {
    let mut buffer: *mut d3d9_vertexbuffer = impl_from_IDirect3DVertexBuffer9(iface);
    let mut resource: *mut wined3d_resource = core::ptr::null_mut::<wined3d_resource>();
    let mut previous: DWORD = 0;
    WINE_TRACE!("iface %p, priority %lu.\n", iface, priority);
    wined3d_mutex_lock();
    resource = wined3d_buffer_get_resource((*buffer).wined3d_buffer);
    previous = wined3d_resource_set_priority(resource, priority as u32) as DWORD;
    wined3d_mutex_unlock();
    return previous;
}
unsafe extern "system" fn d3d9_vertexbuffer_GetPriority(iface: *mut IDirect3DVertexBuffer9) -> DWORD {
    let buffer: *mut d3d9_vertexbuffer = impl_from_IDirect3DVertexBuffer9(iface);
    let mut resource: *const wined3d_resource;
    let mut priority: DWORD;

    wined3d_mutex_lock();
    resource = wined3d_buffer_get_resource((*buffer).wined3d_buffer);
    priority = wined3d_resource_get_priority(resource) as DWORD;
    wined3d_mutex_unlock();

    return priority;
}

unsafe extern "system" fn d3d9_vertexbuffer_PreLoad(iface: *mut IDirect3DVertexBuffer9) {
    let buffer: *mut d3d9_vertexbuffer = impl_from_IDirect3DVertexBuffer9(iface);
    WINE_TRACE!("iface %p.\n", iface);
    wined3d_mutex_lock();
    wined3d_resource_preload(wined3d_buffer_get_resource((*buffer).wined3d_buffer));
    wined3d_mutex_unlock();
}
unsafe extern "system" fn d3d9_vertexbuffer_GetType(
    iface: *mut IDirect3DVertexBuffer9,
) -> D3DRESOURCETYPE {
    // TRACE("iface %p.\n", iface);
    D3DRTYPE_VERTEXBUFFER
}

unsafe extern "system" fn d3d9_vertexbuffer_Lock(
    iface: *mut IDirect3DVertexBuffer9,
    offset: UINT,
    size: UINT,
    data: *mut *mut core::ffi::c_void,
    flags: DWORD,
) -> HRESULT {
    let mut buffer: *mut d3d9_vertexbuffer = impl_from_IDirect3DVertexBuffer9(iface);
    let mut wined3d_resource: *mut wined3d_resource = core::ptr::null_mut::<wined3d_resource>();
    let mut wined3d_map_desc: wined3d_map_desc = core::mem::zeroed::<wined3d_map_desc>();
    let mut wined3d_box: wined3d_box = core::mem::zeroed::<wined3d_box>();
    let mut hr: HRESULT = 0;

    WINE_TRACE!("iface %p, offset %u, size %u, data %p, flags %#lx.\n", iface, offset, size, data, flags);

    wined3d_box_set(
        &raw mut wined3d_box,
        offset as core::ffi::c_uint,
        0 as core::ffi::c_uint,
        (offset as core::ffi::c_uint).wrapping_add(size as core::ffi::c_uint),
        1 as core::ffi::c_uint,
        0 as core::ffi::c_uint,
        1 as core::ffi::c_uint,
    );

    wined3d_resource = wined3d_buffer_get_resource((*buffer).wined3d_buffer);
    hr = wined3d_resource_map(
        wined3d_resource,
        0 as core::ffi::c_uint,
        &raw mut wined3d_map_desc,
        &raw mut wined3d_box,
        wined3dmapflags_from_d3dmapflags(
            flags as core::ffi::c_uint,
            (*buffer).usage as core::ffi::c_uint,
        ) as u32,
    );
    *data = wined3d_map_desc.data;

    return hr;
}

unsafe extern "system" fn d3d9_vertexbuffer_Unlock(iface: *mut IDirect3DVertexBuffer9) -> HRESULT {
    let buffer: *mut d3d9_vertexbuffer = impl_from_IDirect3DVertexBuffer9(iface);

    wined3d_resource_unmap(wined3d_buffer_get_resource((*buffer).wined3d_buffer), 0);

    return D3D_OK;
}
unsafe extern "system" fn d3d9_vertexbuffer_GetDesc(
    mut iface: *mut IDirect3DVertexBuffer9,
    mut desc: *mut D3DVERTEXBUFFER_DESC,
) -> HRESULT {
    let mut buffer: *mut d3d9_vertexbuffer = impl_from_IDirect3DVertexBuffer9(iface);
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
    wined3d_resource_get_desc(wined3d_resource, &raw mut wined3d_desc);
    wined3d_mutex_unlock();
    (*desc).Format = D3DFMT_VERTEXDATA;
    (*desc).Type = D3DRTYPE_VERTEXBUFFER;
    (*desc).Usage = (*buffer).usage;
    (*desc).Pool = d3dpool_from_wined3daccess(wined3d_desc.access, wined3d_desc.usage);
    (*desc).Size = wined3d_desc.size as UINT;
    (*desc).FVF = (*buffer).fvf;
    return D3D_OK;
}
unsafe extern "system" fn d3d9_vertexbuffer_wined3d_object_destroyed(parent: *mut core::ffi::c_void) {
    let mut buffer: *mut d3d9_vertexbuffer = parent as *mut d3d9_vertexbuffer;
    if !(*buffer).draw_buffer.is_null() {
        wined3d_buffer_decref((*buffer).wined3d_buffer);
    }
    d3d9_resource_cleanup(core::ptr::addr_of_mut!((*buffer).resource));
    free(buffer as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn vertexbuffer_init(
    buffer: *mut d3d9_vertexbuffer,
    device: *mut d3d9_device,
    size: UINT,
    usage: UINT,
    fvf: DWORD,
    pool: D3DPOOL,
) -> HRESULT {
    let mut parent_ops: *const wined3d_parent_ops = &raw const d3d9_null_wined3d_parent_ops;
    let mut desc: wined3d_buffer_desc = wined3d_buffer_desc {
        byte_width: 0,
        usage: 0,
        bind_flags: 0,
        access: 0,
        misc_flags: 0,
        structure_byte_stride: 0,
    };
    let mut hr: HRESULT;

    if pool == D3DPOOL_SCRATCH {
        WARN!("Vertex buffer with D3DPOOL_SCRATCH requested.\n");
        return D3DERR_INVALIDCALL;
    }

    if pool == D3DPOOL_MANAGED && (*(*device).d3d_parent).extended != 0 {
        WARN!("Managed resources are not supported by d3d9ex devices.\n");
        return D3DERR_INVALIDCALL;
    }

    /* In d3d9, buffers can't be used as rendertarget or depth/stencil buffer. */
    if usage & ((D3DUSAGE_RENDERTARGET | D3DUSAGE_DEPTHSTENCIL) as DWORD) != 0 {
        return D3DERR_INVALIDCALL;
    }

    (*buffer).IDirect3DVertexBuffer9_iface.lpVtbl = &raw const d3d9_vertexbuffer_vtbl;
    (*buffer).fvf = fvf;
    (*buffer).usage = usage;
    d3d9_resource_init(&raw mut (*buffer).resource);

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

    if (*(*device).d3d_parent).extended == 0 {
        desc.usage |= WINED3DUSAGE_VIDMEM_ACCOUNTING;
    }

    if desc.access & WINED3D_RESOURCE_ACCESS_GPU != 0 {
        desc.bind_flags = WINED3D_BIND_VERTEX_BUFFER;
        parent_ops = &raw const d3d9_vertexbuffer_wined3d_parent_ops;
    }

    wined3d_mutex_lock();
    hr = wined3d_buffer_create(
        (*device).wined3d_device,
        &raw mut desc,
        core::ptr::null(),
        buffer as *mut core::ffi::c_void,
        parent_ops,
        &raw mut (*buffer).wined3d_buffer,
    );
    if hr >= 0 && (desc.access & WINED3D_RESOURCE_ACCESS_GPU) == 0 {
        desc.bind_flags = WINED3D_BIND_VERTEX_BUFFER;
        desc.access = WINED3D_RESOURCE_ACCESS_GPU;
        hr = wined3d_buffer_create(
            (*device).wined3d_device,
            &raw mut desc,
            core::ptr::null(),
            buffer as *mut core::ffi::c_void,
            &raw const d3d9_vertexbuffer_wined3d_parent_ops,
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

    (*buffer).parent_device = &raw mut (*device).IDirect3DDevice9Ex_iface;
    IDirect3DDevice9Ex_AddRef((*buffer).parent_device);

    return D3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn unsafe_impl_from_IDirect3DVertexBuffer9(iface: *mut IDirect3DVertexBuffer9) -> *mut d3d9_vertexbuffer {
    if iface.is_null() {
        return core::ptr::null_mut();
    }
    assert!(((*iface).lpVtbl as *const _ == core::ptr::addr_of!(d3d9_vertexbuffer_vtbl)));

    return impl_from_IDirect3DVertexBuffer9(iface);
}

unsafe extern "C" fn impl_from_IDirect3DIndexBuffer9(
    iface: *mut IDirect3DIndexBuffer9,
) -> *mut d3d9_indexbuffer {
    return CONTAINING_RECORD!(iface, d3d9_indexbuffer, IDirect3DIndexBuffer9_iface);
}

unsafe extern "system" fn d3d9_indexbuffer_QueryInterface(iface: *mut IDirect3DIndexBuffer9, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {

    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DIndexBuffer9)) != false
            || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DResource9)) != false
            || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) != false
    {
        IDirect3DIndexBuffer9_AddRef(iface);
        *out = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    WARN!("%s not implemented, returning E_NOINTERFACE.\n", debugstr_guid(riid));

    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn d3d9_indexbuffer_AddRef(iface: *mut IDirect3DIndexBuffer9) -> ULONG {
    let buffer: *mut d3d9_indexbuffer = impl_from_IDirect3DIndexBuffer9(iface);
    let refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*buffer).resource.refcount) as *mut core::ffi::c_long,
    ) as ULONG;

    if refcount == 1 {
        IDirect3DDevice9Ex_AddRef((*buffer).parent_device);
        wined3d_buffer_incref((*buffer).wined3d_buffer);
    }

    return refcount;
}

unsafe extern "system" fn d3d9_indexbuffer_Release(iface: *mut IDirect3DIndexBuffer9) -> ULONG {
    let mut buffer: *mut d3d9_indexbuffer = impl_from_IDirect3DIndexBuffer9(iface);
    let mut refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*buffer).resource.refcount) as *mut ::core::ffi::c_long,
    ) as ULONG;
    WINE_TRACE!("%p decreasing refcount to %lu.\n", iface, refcount);
    if refcount == 0 {
        let mut device: *mut IDirect3DDevice9Ex = (*buffer).parent_device;
        wined3d_buffer_decref((*buffer).wined3d_buffer);
        /* Release the device last, as it may cause the device to be destroyed. */
        IDirect3DDevice9Ex_Release(device);
    }
    return refcount;
}

unsafe extern "system" fn d3d9_indexbuffer_GetDevice(iface: *mut IDirect3DIndexBuffer9, device: *mut *mut IDirect3DDevice9) -> HRESULT {
    let buffer: *mut d3d9_indexbuffer = impl_from_IDirect3DIndexBuffer9(iface);
    WINE_TRACE!("iface %p, device %p.\n", iface, device);
    *device = (*buffer).parent_device as *mut IDirect3DDevice9;
    IDirect3DDevice9_AddRef(*device);
    WINE_TRACE!("Returning device %p.\n", *device);
    return D3D_OK;
}
unsafe extern "system" fn d3d9_indexbuffer_SetPrivateData(
    mut iface: *mut IDirect3DIndexBuffer9,
    guid: *const GUID,
    mut data: *const core::ffi::c_void,
    mut data_size: DWORD,
    mut flags: DWORD,
) -> HRESULT {
    let mut buffer: *mut d3d9_indexbuffer = impl_from_IDirect3DIndexBuffer9(iface);
    WINE_TRACE!("iface %p, guid %s, data %p, data_size %lu, flags %#lx.\n", iface, debugstr_guid(guid as *const _GUID), data, data_size, flags);
    return d3d9_resource_set_private_data(
        core::ptr::addr_of_mut!((*buffer).resource),
        guid,
        data,
        data_size,
        flags,
    );
}

unsafe extern "system" fn d3d9_indexbuffer_GetPrivateData(iface: *mut IDirect3DIndexBuffer9, guid: *const GUID, data: *mut core::ffi::c_void, data_size: *mut DWORD) -> HRESULT {
    let buffer: *mut d3d9_indexbuffer = impl_from_IDirect3DIndexBuffer9(iface);

    return d3d9_resource_get_private_data(core::ptr::addr_of_mut!((*buffer).resource), guid, data, data_size);
}
unsafe extern "system" fn d3d9_indexbuffer_FreePrivateData(iface: *mut IDirect3DIndexBuffer9, guid: REFGUID) -> HRESULT {
    let buffer: *mut d3d9_indexbuffer = impl_from_IDirect3DIndexBuffer9(iface);

    return d3d9_resource_free_private_data(core::ptr::addr_of_mut!((*buffer).resource), guid);
}

unsafe extern "system" fn d3d9_indexbuffer_SetPriority(iface: *mut IDirect3DIndexBuffer9, priority: DWORD) -> DWORD {
    let mut buffer: *mut d3d9_indexbuffer = impl_from_IDirect3DIndexBuffer9(iface);
    let mut resource: *mut wined3d_resource;
    let mut previous: DWORD;

    wined3d_mutex_lock();
    resource = wined3d_buffer_get_resource((*buffer).wined3d_buffer);
    previous = wined3d_resource_set_priority(resource, priority) as DWORD;
    wined3d_mutex_unlock();

    return previous;
}

unsafe extern "system" fn d3d9_indexbuffer_GetPriority(iface: *mut IDirect3DIndexBuffer9) -> DWORD {
    let buffer: *mut d3d9_indexbuffer = impl_from_IDirect3DIndexBuffer9(iface);
    let mut resource: *const wined3d_resource;
    let mut priority: DWORD;

    wined3d_mutex_lock();
    resource = wined3d_buffer_get_resource((*buffer).wined3d_buffer);
    priority = wined3d_resource_get_priority(resource) as DWORD;
    wined3d_mutex_unlock();

    return priority;
}

unsafe extern "system" fn d3d9_indexbuffer_PreLoad(iface: *mut IDirect3DIndexBuffer9) {
    let buffer: *mut d3d9_indexbuffer = impl_from_IDirect3DIndexBuffer9(iface);
    WINE_TRACE!("iface %p.\n", iface);
    wined3d_mutex_lock();
    wined3d_resource_preload(wined3d_buffer_get_resource((*buffer).wined3d_buffer));
    wined3d_mutex_unlock();
}
unsafe extern "system" fn d3d9_indexbuffer_GetType(
    iface: *mut IDirect3DIndexBuffer9,
) -> D3DRESOURCETYPE {
    // TRACE("iface %p.\n", iface);
    D3DRTYPE_INDEXBUFFER
}

unsafe extern "system" fn d3d9_indexbuffer_Lock(
    iface: *mut IDirect3DIndexBuffer9,
    offset: UINT,
    size: UINT,
    data: *mut *mut core::ffi::c_void,
    flags: DWORD,
) -> HRESULT {
    let buffer: *mut d3d9_indexbuffer = impl_from_IDirect3DIndexBuffer9(iface);
    let mut wined3d_resource: *mut wined3d_resource = core::ptr::null_mut::<wined3d_resource>();
    let mut wined3d_map_desc: wined3d_map_desc = wined3d_map_desc {
        row_pitch: 0,
        slice_pitch: 0,
        data: core::ptr::null_mut::<core::ffi::c_void>(),
    };
    let mut wined3d_box: wined3d_box = wined3d_box {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        front: 0,
        back: 0,
    };
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, offset %u, size %u, data %p, flags %#lx.\n", iface, offset, size, data, flags);
    wined3d_box_set(
        core::ptr::addr_of_mut!(wined3d_box),
        offset as core::ffi::c_uint,
        0 as core::ffi::c_uint,
        (offset as core::ffi::c_uint).wrapping_add(size as core::ffi::c_uint),
        1 as core::ffi::c_uint,
        0 as core::ffi::c_uint,
        1 as core::ffi::c_uint,
    );
    wined3d_resource = wined3d_buffer_get_resource((*buffer).wined3d_buffer);
    hr = wined3d_resource_map(
        wined3d_resource,
        0 as core::ffi::c_uint,
        core::ptr::addr_of_mut!(wined3d_map_desc),
        core::ptr::addr_of_mut!(wined3d_box),
        wined3dmapflags_from_d3dmapflags(
            flags as core::ffi::c_uint,
            (*buffer).usage as core::ffi::c_uint,
        ) as uint32_t,
    );
    *data = wined3d_map_desc.data;
    return hr;
}

unsafe extern "system" fn d3d9_indexbuffer_Unlock(iface: *mut IDirect3DIndexBuffer9) -> HRESULT {
    let buffer: *mut d3d9_indexbuffer = impl_from_IDirect3DIndexBuffer9(iface);
    wined3d_resource_unmap(wined3d_buffer_get_resource((*buffer).wined3d_buffer), 0);
    return D3D_OK;
}
unsafe extern "system" fn d3d9_indexbuffer_GetDesc(
    mut iface: *mut IDirect3DIndexBuffer9,
    mut desc: *mut D3DINDEXBUFFER_DESC,
) -> HRESULT {
    let mut buffer: *mut d3d9_indexbuffer = impl_from_IDirect3DIndexBuffer9(iface);
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
unsafe extern "system" fn d3d9_indexbuffer_wined3d_object_destroyed(parent: *mut core::ffi::c_void) {
    let mut buffer: *mut d3d9_indexbuffer = parent as *mut d3d9_indexbuffer;
    d3d9_resource_cleanup(core::ptr::addr_of_mut!((*buffer).resource));
    free(buffer as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn indexbuffer_init(
    buffer: *mut d3d9_indexbuffer,
    device: *mut d3d9_device,
    size: UINT,
    usage: DWORD,
    format: D3DFORMAT,
    pool: D3DPOOL,
) -> HRESULT {
    let mut desc: wined3d_buffer_desc = core::mem::zeroed();
    let mut hr: HRESULT;

    if pool == D3DPOOL_SCRATCH {
        return D3DERR_INVALIDCALL;
    }

    if pool == D3DPOOL_MANAGED && (*(*device).d3d_parent).extended != 0 {
        WARN!("Managed resources are not supported by d3d9ex devices.\n");
        return D3DERR_INVALIDCALL;
    }

    /* In d3d9, buffers can't be used as rendertarget or depth/stencil buffer. */
    if usage & ((D3DUSAGE_RENDERTARGET | D3DUSAGE_DEPTHSTENCIL) as DWORD) != 0 {
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

    if (*(*device).d3d_parent).extended == 0 {
        desc.usage |= WINED3DUSAGE_VIDMEM_ACCOUNTING;
    }

    if desc.access & WINED3D_RESOURCE_ACCESS_GPU != 0 {
        desc.bind_flags = WINED3D_BIND_INDEX_BUFFER;
    }

    (*buffer).IDirect3DIndexBuffer9_iface.lpVtbl =
        core::ptr::addr_of!(d3d9_indexbuffer_vtbl) as *const IDirect3DIndexBuffer9Vtbl;
    (*buffer).format = wined3dformat_from_d3dformat(format);
    (*buffer).usage = usage;
    (*buffer).sysmem = (((desc.access & WINED3D_RESOURCE_ACCESS_GPU == 0) as BOOL) != 0);
    d3d9_resource_init(core::ptr::addr_of_mut!((*buffer).resource));

    wined3d_mutex_lock();
    hr = wined3d_buffer_create(
        (*device).wined3d_device,
        core::ptr::addr_of_mut!(desc),
        core::ptr::null(),
        buffer as *mut core::ffi::c_void,
        core::ptr::addr_of!(d3d9_indexbuffer_wined3d_parent_ops),
        core::ptr::addr_of_mut!((*buffer).wined3d_buffer),
    );
    wined3d_mutex_unlock();
    if hr < 0 {
        WARN!("Failed to create wined3d buffer, hr %#lx.\n", hr);
        return hr;
    }

    (*buffer).parent_device = core::ptr::addr_of_mut!((*device).IDirect3DDevice9Ex_iface);
    IDirect3DDevice9Ex_AddRef((*buffer).parent_device);

    return D3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn unsafe_impl_from_IDirect3DIndexBuffer9(iface: *mut IDirect3DIndexBuffer9) -> *mut d3d9_indexbuffer {
    if iface.is_null() {
        return core::ptr::null_mut();
    }
    assert!((*iface).lpVtbl == core::ptr::addr_of!(d3d9_indexbuffer_vtbl) as *const _);

    return impl_from_IDirect3DIndexBuffer9(iface);
}

// COM vtable tables — generated from buffer.c by vtbl_gen.py
/// buffer.c:241 `static const IDirect3DVertexBuffer9Vtbl d3d9_vertexbuffer_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d9_vertexbuffer_vtbl: IDirect3DVertexBuffer9Vtbl = IDirect3DVertexBuffer9Vtbl {
    QueryInterface: Some(d3d9_vertexbuffer_QueryInterface),
    AddRef: Some(d3d9_vertexbuffer_AddRef),
    Release: Some(d3d9_vertexbuffer_Release),
    GetDevice: Some(d3d9_vertexbuffer_GetDevice),
    SetPrivateData: Some(d3d9_vertexbuffer_SetPrivateData),
    GetPrivateData: Some(d3d9_vertexbuffer_GetPrivateData),
    FreePrivateData: Some(d3d9_vertexbuffer_FreePrivateData),
    SetPriority: Some(d3d9_vertexbuffer_SetPriority),
    GetPriority: Some(d3d9_vertexbuffer_GetPriority),
    PreLoad: Some(d3d9_vertexbuffer_PreLoad),
    GetType: Some(d3d9_vertexbuffer_GetType),
    Lock: Some(d3d9_vertexbuffer_Lock),
    Unlock: Some(d3d9_vertexbuffer_Unlock),
    GetDesc: Some(d3d9_vertexbuffer_GetDesc),
};

/// buffer.c:563 `static const IDirect3DIndexBuffer9Vtbl d3d9_indexbuffer_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d9_indexbuffer_vtbl: IDirect3DIndexBuffer9Vtbl = IDirect3DIndexBuffer9Vtbl {
    QueryInterface: Some(d3d9_indexbuffer_QueryInterface),
    AddRef: Some(d3d9_indexbuffer_AddRef),
    Release: Some(d3d9_indexbuffer_Release),
    GetDevice: Some(d3d9_indexbuffer_GetDevice),
    SetPrivateData: Some(d3d9_indexbuffer_SetPrivateData),
    GetPrivateData: Some(d3d9_indexbuffer_GetPrivateData),
    FreePrivateData: Some(d3d9_indexbuffer_FreePrivateData),
    SetPriority: Some(d3d9_indexbuffer_SetPriority),
    GetPriority: Some(d3d9_indexbuffer_GetPriority),
    PreLoad: Some(d3d9_indexbuffer_PreLoad),
    GetType: Some(d3d9_indexbuffer_GetType),
    Lock: Some(d3d9_indexbuffer_Lock),
    Unlock: Some(d3d9_indexbuffer_Unlock),
    GetDesc: Some(d3d9_indexbuffer_GetDesc),
};
