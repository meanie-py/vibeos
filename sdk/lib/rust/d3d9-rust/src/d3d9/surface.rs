use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)

static mut d3d9_surface_wined3d_parent_ops: wined3d_parent_ops = unsafe {
    wined3d_parent_ops {
        wined3d_object_destroyed: Some(
            surface_wined3d_object_destroyed,
        ),
    }
};

static mut d3d9_view_wined3d_parent_ops: wined3d_parent_ops = unsafe {
    wined3d_parent_ops {
        wined3d_object_destroyed: Some(
            view_wined3d_object_destroyed,
        ),
    }
};


unsafe extern "C" fn impl_from_IDirect3DSurface9(iface: *mut IDirect3DSurface9) -> *mut d3d9_surface {
    return (iface as *mut u8).sub(core::mem::offset_of!(d3d9_surface, IDirect3DSurface9_iface)) as *mut d3d9_surface;
}

unsafe extern "system" fn d3d9_surface_QueryInterface(iface: *mut IDirect3DSurface9, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DSurface9))
        != false
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DResource9)) != false
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) != false
    {
        IDirect3DSurface9_AddRef(iface);
        *out = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    WARN!(
        "%s not implemented, returning E_NOINTERFACE.\n",
        debugstr_guid(riid)
    );

    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn d3d9_surface_AddRef(iface: *mut IDirect3DSurface9) -> ULONG {
    let surface: *mut d3d9_surface = impl_from_IDirect3DSurface9(iface);
    let mut refcount: ULONG;

    if !(*surface).texture.is_null() {
        return IDirect3DBaseTexture9_AddRef(core::ptr::addr_of_mut!((*(*surface).texture).IDirect3DBaseTexture9_iface));
    }

    refcount = InterlockedIncrement(core::ptr::addr_of_mut!((*surface).resource.refcount) as *mut i32) as ULONG;

    if refcount == 1 {
        if !(*surface).parent_device.is_null() {
            IDirect3DDevice9Ex_AddRef((*surface).parent_device);
        }
        if !(*surface).wined3d_rtv.is_null() {
            wined3d_rendertarget_view_incref((*surface).wined3d_rtv);
        }
        wined3d_texture_incref((*surface).wined3d_texture);
        if !(*surface).swapchain.is_null() {
            wined3d_swapchain_incref((*surface).swapchain);
        }
    }

    return refcount;
}

unsafe extern "system" fn d3d9_surface_Release(iface: *mut IDirect3DSurface9) -> ULONG {
    let surface: *mut d3d9_surface = impl_from_IDirect3DSurface9(iface);
    let mut refcount: ULONG;

    if !(*surface).texture.is_null() {
        TRACE!(
            "Forwarding to %p.\n",
            (*surface).texture,
        );
        return IDirect3DBaseTexture9_Release(&raw mut (*(*surface).texture).IDirect3DBaseTexture9_iface);
    }

    if (*surface).resource.refcount == 0 {
        WARN!("Surface does not have any references.\n");
        return 0;
    }

    refcount = InterlockedDecrement(&raw mut (*surface).resource.refcount as *mut LONG) as ULONG;
    TRACE!(
        "%p decreasing refcount to %lu.\n",
        iface,
        refcount,
    );

    if refcount == 0 {
        let parent_device: *mut IDirect3DDevice9Ex = (*surface).parent_device;

        if !(*surface).wined3d_rtv.is_null() {
            wined3d_rendertarget_view_decref((*surface).wined3d_rtv);
        }
        if !(*surface).swapchain.is_null() {
            wined3d_swapchain_decref((*surface).swapchain);
        }
        /* Releasing the texture may free the d3d9 object, so do not access it
         * after releasing the texture. */
        wined3d_texture_decref((*surface).wined3d_texture);

        /* Release the device last, as it may cause the device to be destroyed. */
        if !parent_device.is_null() {
            IDirect3DDevice9Ex_Release(parent_device);
        }
    }

    return refcount;
}

unsafe extern "system" fn d3d9_surface_GetDevice(
    iface: *mut IDirect3DSurface9,
    device: *mut *mut IDirect3DDevice9,
) -> HRESULT {
    let surface: *mut d3d9_surface = impl_from_IDirect3DSurface9(iface);

    if !(*surface).texture.is_null() {
        return IDirect3DBaseTexture9_GetDevice(
            core::ptr::addr_of_mut!((*(*surface).texture).IDirect3DBaseTexture9_iface),
            device,
        );
    }

    *device = (*surface).parent_device as *mut IDirect3DDevice9;
    IDirect3DDevice9_AddRef(*device);

    return D3D_OK;
}
unsafe extern "system" fn d3d9_surface_SetPrivateData(
    iface: *mut IDirect3DSurface9,
    guid: *const GUID,
    data: *const core::ffi::c_void,
    data_size: DWORD,
    flags: DWORD,
) -> HRESULT {
    let surface: *mut d3d9_surface = impl_from_IDirect3DSurface9(iface);
    WINE_TRACE!("iface %p, guid %s, data %p, data_size %lu, flags %#lx.\n", iface, debugstr_guid(guid as *const _GUID), data, data_size, flags);
    return d3d9_resource_set_private_data(
        core::ptr::addr_of_mut!((*surface).resource),
        guid,
        data,
        data_size,
        flags,
    );
}

unsafe extern "system" fn d3d9_surface_GetPrivateData(iface: *mut IDirect3DSurface9, guid: *const GUID, data: *mut core::ffi::c_void, data_size: *mut DWORD) -> HRESULT {
    let mut surface: *mut d3d9_surface = impl_from_IDirect3DSurface9(iface);

    return d3d9_resource_get_private_data(core::ptr::addr_of_mut!((*surface).resource), guid, data, data_size);
}
unsafe extern "system" fn d3d9_surface_FreePrivateData(iface: *mut IDirect3DSurface9, guid: REFGUID) -> HRESULT {
    let surface: *mut d3d9_surface = impl_from_IDirect3DSurface9(iface);
    WINE_TRACE!("iface %p, guid %s.\n", iface, debugstr_guid(guid as *const GUID));
    return d3d9_resource_free_private_data(core::ptr::addr_of_mut!((*surface).resource), guid);
}
unsafe extern "system" fn d3d9_surface_SetPriority(
    iface: *mut IDirect3DSurface9,
    priority: DWORD,
) -> DWORD {
    // TRACE("iface %p, priority %lu. Ignored on surfaces.\n", iface, priority);
    0
}
unsafe extern "system" fn d3d9_surface_GetPriority(iface: *mut IDirect3DSurface9) -> DWORD {
    // TRACE("iface %p. Ignored on surfaces.\n", iface);
    0
}

unsafe extern "system" fn d3d9_surface_PreLoad(mut iface: *mut IDirect3DSurface9) {
    let mut surface: *mut d3d9_surface = impl_from_IDirect3DSurface9(iface);
    WINE_TRACE!("iface %p.\n", iface);
    wined3d_mutex_lock();
    wined3d_resource_preload(wined3d_texture_get_resource((*surface).wined3d_texture));
    wined3d_mutex_unlock();
}
unsafe extern "system" fn d3d9_surface_GetType(
    iface: *mut IDirect3DSurface9,
) -> D3DRESOURCETYPE {
    // TRACE("iface %p.\n", iface);
    D3DRTYPE_SURFACE
}

unsafe extern "system" fn d3d9_surface_GetContainer(iface: *mut IDirect3DSurface9, riid: REFIID, container: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut surface: *mut d3d9_surface = impl_from_IDirect3DSurface9(iface);
    let mut hr: HRESULT = 0;

    if (*surface).container.is_null() {
        return E_NOINTERFACE;
    }

    hr = IUnknown_QueryInterface((*surface).container, riid, container);

    return hr;
}
unsafe extern "system" fn d3d9_surface_GetDesc(iface: *mut IDirect3DSurface9, desc: *mut D3DSURFACE_DESC) -> HRESULT {
    let surface: *mut d3d9_surface = impl_from_IDirect3DSurface9(iface);
    let mut wined3d_desc: wined3d_sub_resource_desc = core::mem::zeroed();

    wined3d_mutex_lock();
    wined3d_texture_get_sub_resource_desc((*surface).wined3d_texture, (*surface).sub_resource_idx, core::ptr::addr_of_mut!(wined3d_desc));
    wined3d_mutex_unlock();

    (*desc).Format = d3dformat_from_wined3dformat(wined3d_desc.format);
    (*desc).Type = D3DRTYPE_SURFACE;
    (*desc).Usage = d3dusage_from_wined3dusage(wined3d_desc.usage, wined3d_desc.bind_flags);
    (*desc).Pool = d3dpool_from_wined3daccess(wined3d_desc.access, wined3d_desc.usage);
    (*desc).MultiSampleType = d3dmultisample_type_from_wined3d(wined3d_desc.multisample_type);
    (*desc).MultiSampleQuality = wined3d_desc.multisample_quality as DWORD;
    (*desc).Width = wined3d_desc.width as UINT;
    (*desc).Height = wined3d_desc.height as UINT;

    return D3D_OK;
}

unsafe extern "system" fn d3d9_surface_LockRect(
    mut iface: *mut IDirect3DSurface9,
    mut locked_rect: *mut D3DLOCKED_RECT,
    mut rect: *const RECT,
    mut flags: DWORD,
) -> HRESULT {
    let mut surface: *mut d3d9_surface = impl_from_IDirect3DSurface9(iface);
    let mut box_0: wined3d_box = wined3d_box {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        front: 0,
        back: 0,
    };
    let mut map_desc: wined3d_map_desc = wined3d_map_desc {
        row_pitch: 0,
        slice_pitch: 0,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, locked_rect %p, rect %s, flags %#lx.\n", iface, locked_rect, wine_dbgstr_rect(rect), flags);
    if !rect.is_null() {
        wined3d_box_set(
            core::ptr::addr_of_mut!(box_0),
            (*rect).left as ::core::ffi::c_uint,
            (*rect).top as ::core::ffi::c_uint,
            (*rect).right as ::core::ffi::c_uint,
            (*rect).bottom as ::core::ffi::c_uint,
            0,
            1,
        );
    }
    hr = wined3d_resource_map(
        wined3d_texture_get_resource((*surface).wined3d_texture),
        (*surface).sub_resource_idx,
        core::ptr::addr_of_mut!(map_desc),
        if !rect.is_null() {
            core::ptr::addr_of_mut!(box_0)
        } else {
            ::core::ptr::null_mut::<wined3d_box>()
        },
        wined3dmapflags_from_d3dmapflags(flags, 0) as uint32_t,
    );
    if (hr >= 0) {
        (*locked_rect).Pitch = map_desc.row_pitch as INT;
        (*locked_rect).pBits = map_desc.data;
    }
    if hr == E_INVALIDARG {
        return D3DERR_INVALIDCALL;
    }
    return hr;
}

unsafe extern "system" fn d3d9_surface_UnlockRect(iface: *mut IDirect3DSurface9) -> HRESULT {
    let surface: *mut d3d9_surface = impl_from_IDirect3DSurface9(iface);
    let mut hr: HRESULT;

    wined3d_mutex_lock();
    hr = wined3d_resource_unmap(
        wined3d_texture_get_resource((*surface).wined3d_texture),
        (*surface).sub_resource_idx,
    );
    if hr >= 0 && !(*surface).texture.is_null() {
        d3d9_texture_flag_auto_gen_mipmap((*surface).texture);
    }
    wined3d_mutex_unlock();

    if hr == WINEDDERR_NOTLOCKED {
        let mut type_: D3DRESOURCETYPE;
        if !(*surface).texture.is_null() {
            type_ = IDirect3DBaseTexture9_GetType(core::ptr::addr_of_mut!(
                (*(*surface).texture).IDirect3DBaseTexture9_iface
            ));
        } else {
            type_ = D3DRTYPE_SURFACE;
        }
        hr = if type_ == D3DRTYPE_TEXTURE {
            D3D_OK
        } else {
            D3DERR_INVALIDCALL
        };
    }
    return hr;
}
unsafe extern "system" fn d3d9_surface_GetDC(iface: *mut IDirect3DSurface9, dc: *mut HDC) -> HRESULT {
    let surface: *mut d3d9_surface = impl_from_IDirect3DSurface9(iface);
    let mut hr: HRESULT;

    wined3d_mutex_lock();
    hr = wined3d_texture_get_dc((*surface).wined3d_texture, (*surface).sub_resource_idx, dc);
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d9_surface_ReleaseDC(iface: *mut IDirect3DSurface9, dc: HDC) -> HRESULT {
    let surface: *mut d3d9_surface = impl_from_IDirect3DSurface9(iface);
    let mut hr: HRESULT;

    TRACE!("iface %p, dc %p.\n", iface, dc);
    wined3d_mutex_lock();
    hr = wined3d_texture_release_dc((*surface).wined3d_texture, (*surface).sub_resource_idx, dc);
    if (hr >= 0) && !(*surface).texture.is_null() {
        d3d9_texture_flag_auto_gen_mipmap((*surface).texture);
    }
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn surface_wined3d_object_destroyed(parent: *mut core::ffi::c_void) {
    let surface: *mut d3d9_surface = parent as *mut d3d9_surface;
    d3d9_resource_cleanup(core::ptr::addr_of_mut!((*surface).resource));
    free(surface as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn d3d9_surface_create(
    wined3d_texture: *mut wined3d_texture,
    sub_resource_idx: ::core::ffi::c_uint,
    container: *mut IUnknown,
) -> *mut d3d9_surface {
    let mut texture: *mut IDirect3DBaseTexture9 = core::ptr::null_mut();
    let mut surface: *mut d3d9_surface = core::ptr::null_mut();

    surface = calloc(1, (core::mem::size_of::<d3d9_surface>()) as _) as *mut d3d9_surface;
    if surface.is_null() {
        return core::ptr::null_mut();
    }

    (*surface).IDirect3DSurface9_iface.lpVtbl = &raw const d3d9_surface_vtbl as *const IDirect3DSurface9Vtbl;
    d3d9_resource_init(&raw mut (*surface).resource);
    (*surface).resource.refcount = 0;
    list_init(&raw mut (*surface).rtv_entry);
    (*surface).container = container;
    (*surface).wined3d_texture = wined3d_texture;
    (*surface).sub_resource_idx = sub_resource_idx;
    (*surface).swapchain = wined3d_texture_get_swapchain(wined3d_texture);

    if !(*surface).container.is_null()
        && IUnknown_QueryInterface(
            (*surface).container,
            &raw const IID_IDirect3DBaseTexture9,
            &raw mut texture as *mut *mut ::core::ffi::c_void,
        ) >= 0
    {
        (*surface).texture = unsafe_impl_from_IDirect3DBaseTexture9(texture);
        IDirect3DBaseTexture9_Release(texture);
    }

    wined3d_texture_set_sub_resource_parent(
        wined3d_texture,
        sub_resource_idx,
        surface as *mut ::core::ffi::c_void,
        &raw const d3d9_surface_wined3d_parent_ops,
    );

    WINE_TRACE!("Created surface %p.\n", surface);

    return surface;
}
unsafe extern "system" fn view_wined3d_object_destroyed(parent: *mut core::ffi::c_void) {
    let mut surface: *mut d3d9_surface = parent as *mut d3d9_surface;
    (*surface).wined3d_rtv = core::ptr::null_mut();
    list_remove(core::ptr::addr_of_mut!((*surface).rtv_entry));
}
#[no_mangle]
pub unsafe extern "C" fn d3d9_surface_get_device(surface: *const d3d9_surface) -> *mut d3d9_device {
    let mut device: *mut IDirect3DDevice9Ex = core::ptr::null_mut::<IDirect3DDevice9Ex>();
    device = if !(*surface).texture.is_null() {
        core::ptr::addr_of_mut!((*(*(*surface).texture).parent_device).IDirect3DDevice9Ex_iface)
    } else {
        (*surface).parent_device
    };
    return impl_from_IDirect3DDevice9Ex(device);
}
#[no_mangle]
pub unsafe extern "C" fn d3d9_surface_acquire_rendertarget_view(surface: *mut d3d9_surface) -> *mut wined3d_rendertarget_view {
    let mut hr: HRESULT = 0;

    /* The surface reference count can be equal to 0 when this function is
     * called. In order to properly manage the render target view reference
     * count, we temporarily increment the surface reference count. */
    d3d9_surface_AddRef(core::ptr::addr_of_mut!((*surface).IDirect3DSurface9_iface));

    if !(*surface).wined3d_rtv.is_null() {
        return (*surface).wined3d_rtv;
    }

    hr = wined3d_rendertarget_view_create_from_sub_resource(
        (*surface).wined3d_texture,
        (*surface).sub_resource_idx,
        surface as *mut core::ffi::c_void,
        core::ptr::addr_of!(d3d9_view_wined3d_parent_ops),
        core::ptr::addr_of_mut!((*surface).wined3d_rtv),
    );
    if hr < 0 {
        ERR!("Failed to create rendertarget view, hr %#lx.\n", hr);
        d3d9_surface_Release(core::ptr::addr_of_mut!((*surface).IDirect3DSurface9_iface));
        return core::ptr::null_mut();
    }

    if !(*surface).texture.is_null() {
        list_add_head(
            core::ptr::addr_of_mut!((*(*surface).texture).rtv_list),
            core::ptr::addr_of_mut!((*surface).rtv_entry),
        );
    }

    return (*surface).wined3d_rtv;
}
#[no_mangle]
pub unsafe extern "C" fn d3d9_surface_release_rendertarget_view(
    surface: *mut d3d9_surface,
    rtv: *mut wined3d_rendertarget_view,
) {
    if !rtv.is_null() {
        d3d9_surface_Release(core::ptr::addr_of_mut!((*surface).IDirect3DSurface9_iface));
    }
}
#[no_mangle]
pub unsafe extern "C" fn unsafe_impl_from_IDirect3DSurface9(iface: *mut IDirect3DSurface9) -> *mut d3d9_surface {
    if iface.is_null() {
        return core::ptr::null_mut();
    }
    ASSERT!((*iface).lpVtbl == core::ptr::addr_of!(d3d9_surface_vtbl));

    return impl_from_IDirect3DSurface9(iface);
}

// COM vtable tables — generated from surface.c by vtbl_gen.py
/// surface.c:320 `static const IDirect3DSurface9Vtbl d3d9_surface_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d9_surface_vtbl: IDirect3DSurface9Vtbl = IDirect3DSurface9Vtbl {
    QueryInterface: Some(d3d9_surface_QueryInterface),
    AddRef: Some(d3d9_surface_AddRef),
    Release: Some(d3d9_surface_Release),
    GetDevice: Some(d3d9_surface_GetDevice),
    SetPrivateData: Some(d3d9_surface_SetPrivateData),
    GetPrivateData: Some(d3d9_surface_GetPrivateData),
    FreePrivateData: Some(d3d9_surface_FreePrivateData),
    SetPriority: Some(d3d9_surface_SetPriority),
    GetPriority: Some(d3d9_surface_GetPriority),
    PreLoad: Some(d3d9_surface_PreLoad),
    GetType: Some(d3d9_surface_GetType),
    GetContainer: Some(d3d9_surface_GetContainer),
    GetDesc: Some(d3d9_surface_GetDesc),
    LockRect: Some(d3d9_surface_LockRect),
    UnlockRect: Some(d3d9_surface_UnlockRect),
    GetDC: Some(d3d9_surface_GetDC),
    ReleaseDC: Some(d3d9_surface_ReleaseDC),
};
