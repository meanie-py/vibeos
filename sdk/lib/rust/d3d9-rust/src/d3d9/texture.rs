use crate::d3d9::device::d3d9_null_wined3d_parent_ops;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)

static mut d3d9_texture_wined3d_parent_ops: wined3d_parent_ops = unsafe {
    wined3d_parent_ops {
        wined3d_object_destroyed: Some(
            d3d9_texture_wined3d_object_destroyed,
        ),
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(d3d9): debug channel declaration, provided by the crate's DPRINT macros.

static d3d9_texture_3d_PreLoad_dummy: u8 = 0;

unsafe extern "C" fn d3d9_texture_incref(mut texture: *mut d3d9_texture) -> ULONG {
    let mut ref_: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*texture).resource.refcount) as *mut i32,
    ) as ULONG;

    TRACE!(
        "%p increasing refcount to %lu.\n",
        texture,
        ref_
    );

    if ref_ == 1 {
        let mut surface: *mut d3d9_surface = core::ptr::null_mut();

        IDirect3DDevice9Ex_AddRef(
            core::ptr::addr_of_mut!((*(*texture).parent_device).IDirect3DDevice9Ex_iface),
        );
        wined3d_mutex_lock();
        surface = ((*texture).rtv_list.next as *mut u8)
            .offset(-(core::mem::offset_of!(d3d9_surface, rtv_entry) as isize))
            as *mut d3d9_surface;
        while core::ptr::addr_of_mut!((*surface).rtv_entry)
            != core::ptr::addr_of_mut!((*texture).rtv_list)
        {
            wined3d_rendertarget_view_incref((*surface).wined3d_rtv);
            surface = ((*surface).rtv_entry.next as *mut u8)
                .offset(-(core::mem::offset_of!(d3d9_surface, rtv_entry) as isize))
                as *mut d3d9_surface;
        }
        wined3d_texture_incref(d3d9_texture_get_draw_texture(texture));
        wined3d_mutex_unlock();
    }

    return ref_;
}

unsafe extern "C" fn d3d9_texture_decref(texture: *mut d3d9_texture) -> ULONG {
    let ref_0: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*texture).resource.refcount)) as ULONG;
    WINE_TRACE!("%p decreasing refcount to %lu.\n", texture, ref_0);
    if ref_0 == 0 {
        let parent_device: *mut IDirect3DDevice9Ex =
            core::ptr::addr_of_mut!((*(*texture).parent_device).IDirect3DDevice9Ex_iface);
        let mut surface: *mut d3d9_surface = core::ptr::null_mut::<d3d9_surface>();

        wined3d_mutex_lock();

        surface = CONTAINING_RECORD!(
            (*texture).rtv_list.next,
            d3d9_surface,
            rtv_entry,
        );
        while core::ptr::addr_of_mut!((*surface).rtv_entry)
            != core::ptr::addr_of_mut!((*texture).rtv_list)
        {
            wined3d_rendertarget_view_decref((*surface).wined3d_rtv);
            surface = CONTAINING_RECORD!(
                (*surface).rtv_entry.next,
                d3d9_surface,
                rtv_entry,
            );
        }

        wined3d_texture_decref(d3d9_texture_get_draw_texture(texture));
        wined3d_mutex_unlock();

        /* Release the device last, as it may cause the device to be destroyed. */
        IDirect3DDevice9Ex_Release(parent_device);
    }
    return ref_0;
}

unsafe extern "C" fn d3d9_texture_preload(texture: *mut d3d9_texture) {
    wined3d_mutex_lock();
    wined3d_resource_preload(wined3d_texture_get_resource(d3d9_texture_get_draw_texture(texture)));
    wined3d_mutex_unlock();
}

unsafe extern "C" fn impl_from_IDirect3DTexture9(iface: *mut IDirect3DTexture9) -> *mut d3d9_texture {
    return (iface as *mut u8).wrapping_sub(
        core::mem::offset_of!(d3d9_texture, IDirect3DBaseTexture9_iface),
    ) as *mut d3d9_texture;
}

unsafe extern "C" fn impl_from_IDirect3DCubeTexture9(iface: *mut IDirect3DCubeTexture9) -> *mut d3d9_texture {
    return ((iface as *mut u8).wrapping_sub(core::mem::offset_of!(d3d9_texture, IDirect3DBaseTexture9_iface)))
        as *mut d3d9_texture;
}

unsafe extern "C" fn impl_from_IDirect3DVolumeTexture9(
    mut iface: *mut IDirect3DVolumeTexture9,
) -> *mut d3d9_texture {
    return (iface as *mut u8).wrapping_sub(
        core::mem::offset_of!(d3d9_texture, IDirect3DBaseTexture9_iface),
    ) as *mut d3d9_texture;
}
#[no_mangle]
pub unsafe extern "C" fn d3d9_texture_gen_auto_mipmap(texture: *mut d3d9_texture) {
    if (*texture).flags as ::core::ffi::c_int & (D3D9_TEXTURE_MIPMAP_DIRTY as i32) == 0 {
        return;
    }
    if !(*texture).draw_texture.is_null() {
        wined3d_device_update_texture(
            (*(*texture).parent_device).wined3d_device,
            (*texture).wined3d_texture,
            (*texture).draw_texture,
        );
    }
    wined3d_device_context_generate_mipmaps(
        (*(*texture).parent_device).immediate_context,
        wined3d_texture_acquire_identity_srv(d3d9_texture_get_draw_texture(texture)),
    );
    (*texture).flags &= (!D3D9_TEXTURE_MIPMAP_DIRTY) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn d3d9_texture_flag_auto_gen_mipmap(texture: *mut d3d9_texture) {
    if ((*texture).usage & (D3DUSAGE_AUTOGENMIPMAP as u32)) != 0 {
        (*texture).flags |= (D3D9_TEXTURE_MIPMAP_DIRTY as i32);
    }
}

unsafe extern "system" fn d3d9_texture_2d_QueryInterface(iface: *mut IDirect3DTexture9, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {

    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DTexture9)) != false
            || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DBaseTexture9)) != false
            || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DResource9)) != false
            || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) != false
    {
        IDirect3DTexture9_AddRef(iface);
        *out = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    WARN!("%s not implemented, returning E_NOINTERFACE.\n", debugstr_guid(riid));

    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn d3d9_texture_2d_AddRef(iface: *mut IDirect3DTexture9) -> ULONG {
    let texture: *mut d3d9_texture = impl_from_IDirect3DTexture9(iface);

    return d3d9_texture_incref(texture);
}
unsafe extern "system" fn d3d9_texture_2d_Release(iface: *mut IDirect3DTexture9) -> ULONG {
    let texture: *mut d3d9_texture = impl_from_IDirect3DTexture9(iface);

    return d3d9_texture_decref(texture);
}
unsafe extern "system" fn d3d9_texture_2d_GetDevice(
    mut iface: *mut IDirect3DTexture9,
    mut device: *mut *mut IDirect3DDevice9,
) -> HRESULT {
    let mut texture: *mut d3d9_texture = impl_from_IDirect3DTexture9(iface);
    WINE_TRACE!("iface %p, device %p.\n", iface, device);
    *device = core::ptr::addr_of_mut!((*(*texture).parent_device).IDirect3DDevice9Ex_iface)
        as *mut IDirect3DDevice9;
    IDirect3DDevice9_AddRef(*device);
    WINE_TRACE!("Returning device %p.\n", *device);
    return D3D_OK;
}

unsafe extern "system" fn d3d9_texture_2d_SetPrivateData(iface: *mut IDirect3DTexture9, guid: *const GUID, data: *const core::ffi::c_void, data_size: DWORD, flags: DWORD) -> HRESULT {
    let texture: *mut d3d9_texture = impl_from_IDirect3DTexture9(iface);

    return d3d9_resource_set_private_data(core::ptr::addr_of_mut!((*texture).resource), guid, data, data_size, flags);
}
unsafe extern "system" fn d3d9_texture_2d_GetPrivateData(
    mut iface: *mut IDirect3DTexture9,
    guid: *const GUID,
    mut data: *mut ::core::ffi::c_void,
    mut data_size: *mut DWORD,
) -> HRESULT {
    let mut texture: *mut d3d9_texture = impl_from_IDirect3DTexture9(iface);
    WINE_TRACE!("iface %p, guid %s, data %p, data_size %p.\n", iface, debugstr_guid(guid as *const _GUID), data, data_size);
    return d3d9_resource_get_private_data(
        core::ptr::addr_of_mut!((*texture).resource),
        guid,
        data,
        data_size,
    );
}
unsafe extern "system" fn d3d9_texture_2d_FreePrivateData(
    mut iface: *mut IDirect3DTexture9,
    guid: *const GUID,
) -> HRESULT {
    let mut texture: *mut d3d9_texture = impl_from_IDirect3DTexture9(iface);
    WINE_TRACE!("iface %p, guid %s.\n", iface, debugstr_guid(guid as *const _GUID));
    return d3d9_resource_free_private_data(&raw mut (*texture).resource, guid);
}

unsafe extern "system" fn d3d9_texture_2d_SetPriority(
    iface: *mut IDirect3DTexture9,
    priority: DWORD,
) -> DWORD {
    let mut texture: *mut d3d9_texture = impl_from_IDirect3DTexture9(iface);
    let mut resource: *mut wined3d_resource = core::ptr::null_mut();
    let mut ret: DWORD = 0;
    WINE_TRACE!("iface %p, priority %lu.\n", iface, priority);
    wined3d_mutex_lock();
    resource = wined3d_texture_get_resource(d3d9_texture_get_draw_texture(texture));
    ret = wined3d_resource_set_priority(resource, priority as u32) as DWORD;
    wined3d_mutex_unlock();
    return ret;
}

unsafe extern "system" fn d3d9_texture_2d_GetPriority(iface: *mut IDirect3DTexture9) -> DWORD {
    let mut texture: *mut d3d9_texture = impl_from_IDirect3DTexture9(iface);
    let mut resource: *const wined3d_resource = core::ptr::null();
    let mut ret: DWORD = 0;

    wined3d_mutex_lock();
    resource = wined3d_texture_get_resource(d3d9_texture_get_draw_texture(texture));
    ret = wined3d_resource_get_priority(resource) as DWORD;
    wined3d_mutex_unlock();

    return ret;
}

unsafe extern "system" fn d3d9_texture_2d_PreLoad(iface: *mut IDirect3DTexture9) {
    let texture: *mut d3d9_texture = impl_from_IDirect3DTexture9(iface);

    WINE_TRACE!("iface %p.\n", iface);

    d3d9_texture_preload(texture);
}
unsafe extern "system" fn d3d9_texture_2d_GetType(
    iface: *mut IDirect3DTexture9,
) -> D3DRESOURCETYPE {
    // TRACE("iface %p.\n", iface);
    D3DRTYPE_TEXTURE
}

unsafe extern "system" fn d3d9_texture_2d_SetLOD(iface: *mut IDirect3DTexture9, lod: DWORD) -> DWORD {
    let texture: *mut d3d9_texture = impl_from_IDirect3DTexture9(iface);
    let mut ret: DWORD;

    WINE_TRACE!("iface %p, lod %lu.\n", iface, lod);

    wined3d_mutex_lock();
    ret = wined3d_stateblock_set_texture_lod(
        (*(*texture).parent_device).state,
        d3d9_texture_get_draw_texture(texture),
        lod,
    ) as DWORD;
    wined3d_mutex_unlock();

    return ret;
}

unsafe extern "system" fn d3d9_texture_2d_GetLOD(mut iface: *mut IDirect3DTexture9) -> DWORD {
    let mut texture: *mut d3d9_texture = impl_from_IDirect3DTexture9(iface);
    let mut ret: DWORD = 0;
    WINE_TRACE!("iface %p.\n", iface);
    wined3d_mutex_lock();
    ret = wined3d_texture_get_lod(d3d9_texture_get_draw_texture(texture)) as DWORD;
    wined3d_mutex_unlock();
    return ret;
}

unsafe extern "system" fn d3d9_texture_2d_GetLevelCount(iface: *mut IDirect3DTexture9) -> DWORD {
    let texture: *mut d3d9_texture = impl_from_IDirect3DTexture9(iface);
    let mut ret: DWORD;

    if (*texture).usage & (D3DUSAGE_AUTOGENMIPMAP as u32) != 0 {
        return 1;
    }

    wined3d_mutex_lock();
    ret = wined3d_texture_get_level_count((*texture).wined3d_texture) as DWORD;
    wined3d_mutex_unlock();

    return ret;
}

unsafe extern "system" fn d3d9_texture_2d_SetAutoGenFilterType(iface: *mut IDirect3DTexture9, filter_type: D3DTEXTUREFILTERTYPE) -> HRESULT {
    let mut texture: *mut d3d9_texture = impl_from_IDirect3DTexture9(iface);

    if filter_type == D3DTEXF_NONE {
        WARN!("Invalid filter type D3DTEXF_NONE specified.\n");
        return D3DERR_INVALIDCALL;
    }
    if ((*texture).usage & (D3DUSAGE_AUTOGENMIPMAP as u32)) == 0 {
        WARN!("Called on a texture without the D3DUSAGE_AUTOGENMIPMAP flag.\n");
    } else if filter_type != D3DTEXF_LINEAR {
        FIXME!("Unsupported filter type %u.\n", filter_type);
    }

    (*texture).autogen_filter_type = filter_type;
    return D3D_OK;
}

unsafe extern "system" fn d3d9_texture_2d_GetAutoGenFilterType(iface: *mut IDirect3DTexture9) -> D3DTEXTUREFILTERTYPE {
    let texture: *mut d3d9_texture = impl_from_IDirect3DTexture9(iface);

    if ((*texture).usage & (D3DUSAGE_AUTOGENMIPMAP as u32)) == 0 {
        WARN!("Called on a texture without the D3DUSAGE_AUTOGENMIPMAP flag.\n");
    }

    return (*texture).autogen_filter_type;
}
unsafe extern "system" fn d3d9_texture_2d_GenerateMipSubLevels(iface: *mut IDirect3DTexture9) {
    let mut texture: *mut d3d9_texture = impl_from_IDirect3DTexture9(iface);

    wined3d_mutex_lock();
    d3d9_texture_gen_auto_mipmap(texture);
    wined3d_mutex_unlock();
}

unsafe extern "system" fn d3d9_texture_2d_GetLevelDesc(iface: *mut IDirect3DTexture9, level: UINT, desc: *mut D3DSURFACE_DESC) -> HRESULT {
    let texture: *mut d3d9_texture = impl_from_IDirect3DTexture9(iface);
    let mut wined3d_desc: wined3d_sub_resource_desc = core::mem::zeroed();
    let mut hr: HRESULT;

    if ((*texture).usage & (D3DUSAGE_AUTOGENMIPMAP as u32)) != 0 && level != 0 {
        WARN!("D3DUSAGE_AUTOGENMIPMAP textures have only one accessible level.\n");
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    hr = wined3d_texture_get_sub_resource_desc((*texture).wined3d_texture, level, core::ptr::addr_of_mut!(wined3d_desc));
    if hr >= 0 {
        (*desc).Format = d3dformat_from_wined3dformat(wined3d_desc.format);
        (*desc).Type = D3DRTYPE_SURFACE;
        (*desc).Usage = (*texture).usage;
        (*desc).Pool = d3dpool_from_wined3daccess(wined3d_desc.access, wined3d_desc.usage);
        (*desc).MultiSampleType = d3dmultisample_type_from_wined3d(wined3d_desc.multisample_type);
        (*desc).MultiSampleQuality = wined3d_desc.multisample_quality;
        (*desc).Width = wined3d_desc.width;
        (*desc).Height = wined3d_desc.height;
    }
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d9_texture_2d_GetSurfaceLevel(iface: *mut IDirect3DTexture9, level: UINT, surface: *mut *mut IDirect3DSurface9) -> HRESULT {
    let texture: *mut d3d9_texture = impl_from_IDirect3DTexture9(iface);
    let mut surface_impl: *mut d3d9_surface = core::ptr::null_mut();
    TRACE!("iface %p, level %u, surface %p.\n", iface, level, surface);
    if (*texture).usage & (D3DUSAGE_AUTOGENMIPMAP as u32) != 0 && level != 0 {
        WARN!("D3DUSAGE_AUTOGENMIPMAP textures have only one accessible level.\n");
        return D3DERR_INVALIDCALL;
    }
    wined3d_mutex_lock();
    surface_impl = wined3d_texture_get_sub_resource_parent((*texture).wined3d_texture, level) as *mut d3d9_surface;
    if surface_impl.is_null() {
        wined3d_mutex_unlock();
        return D3DERR_INVALIDCALL;
    }
    *surface = core::ptr::addr_of_mut!((*surface_impl).IDirect3DSurface9_iface);
    IDirect3DSurface9_AddRef(*surface);
    wined3d_mutex_unlock();
    return D3D_OK;
}

unsafe extern "system" fn d3d9_texture_2d_LockRect(
    iface: *mut IDirect3DTexture9,
    level: UINT,
    locked_rect: *mut D3DLOCKED_RECT,
    rect: *const RECT,
    flags: DWORD,
) -> HRESULT {
    let texture: *mut d3d9_texture = impl_from_IDirect3DTexture9(iface);
    let mut surface_impl: *mut d3d9_surface;
    let mut hr: HRESULT;

    if ((*texture).usage & (D3DUSAGE_AUTOGENMIPMAP as u32)) != 0 && level != 0 {
        WARN!("D3DUSAGE_AUTOGENMIPMAP textures have only one accessible level.\n");
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    surface_impl = wined3d_texture_get_sub_resource_parent((*texture).wined3d_texture, level) as *mut d3d9_surface;
    if surface_impl.is_null() {
        hr = D3DERR_INVALIDCALL;
    } else {
        hr = IDirect3DSurface9_LockRect(
            core::ptr::addr_of_mut!((*surface_impl).IDirect3DSurface9_iface),
            locked_rect,
            rect,
            flags,
        );
    }
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d9_texture_2d_UnlockRect(iface: *mut IDirect3DTexture9, level: UINT) -> HRESULT {
    let texture: *mut d3d9_texture = impl_from_IDirect3DTexture9(iface);
    let mut surface_impl: *mut d3d9_surface;
    let mut hr: HRESULT;

    if (*texture).usage & (D3DUSAGE_AUTOGENMIPMAP as u32) != 0 && level != 0 {
        WARN!("D3DUSAGE_AUTOGENMIPMAP textures have only one accessible level.\n");
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    surface_impl = wined3d_texture_get_sub_resource_parent((*texture).wined3d_texture, level) as *mut d3d9_surface;
    if surface_impl.is_null() {
        hr = D3DERR_INVALIDCALL;
    } else {
        hr = IDirect3DSurface9_UnlockRect(&raw mut (*surface_impl).IDirect3DSurface9_iface);
    }
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d9_texture_2d_AddDirtyRect(iface: *mut IDirect3DTexture9, dirty_rect: *const RECT) -> HRESULT {
    let texture: *mut d3d9_texture = impl_from_IDirect3DTexture9(iface);
    let mut hr: HRESULT = 0;

    wined3d_mutex_lock();
    if dirty_rect.is_null() {
        hr = wined3d_texture_add_dirty_region((*texture).wined3d_texture, 0, core::ptr::null());
    } else {
        let mut dirty_region: wined3d_box = core::mem::zeroed();
        wined3d_box_set(
            core::ptr::addr_of_mut!(dirty_region),
            (*dirty_rect).left as u32,
            (*dirty_rect).top as u32,
            (*dirty_rect).right as u32,
            (*dirty_rect).bottom as u32,
            0,
            1,
        );
        hr = wined3d_texture_add_dirty_region(
            (*texture).wined3d_texture,
            0,
            core::ptr::addr_of_mut!(dirty_region),
        );
    }
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d9_texture_cube_QueryInterface(iface: *mut IDirect3DCubeTexture9, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DCubeTexture9))
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DBaseTexture9))
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DResource9))
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown))
    {
        IDirect3DCubeTexture9_AddRef(iface);
        *out = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    WARN!("%s not implemented, returning E_NOINTERFACE.\n", debugstr_guid(riid));

    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn d3d9_texture_cube_AddRef(iface: *mut IDirect3DCubeTexture9) -> ULONG {
    let texture: *mut d3d9_texture = impl_from_IDirect3DCubeTexture9(iface);

    return d3d9_texture_incref(texture);
}

unsafe extern "system" fn d3d9_texture_cube_Release(iface: *mut IDirect3DCubeTexture9) -> ULONG {
    let texture: *mut d3d9_texture = impl_from_IDirect3DCubeTexture9(iface);

    return d3d9_texture_decref(texture);
}
unsafe extern "system" fn d3d9_texture_cube_GetDevice(
    iface: *mut IDirect3DCubeTexture9,
    device: *mut *mut IDirect3DDevice9,
) -> HRESULT {
    let texture: *mut d3d9_texture = impl_from_IDirect3DCubeTexture9(iface);

    *device = core::ptr::addr_of_mut!((*(*texture).parent_device).IDirect3DDevice9Ex_iface)
        as *mut IDirect3DDevice9;
    IDirect3DDevice9_AddRef(*device);

    return D3D_OK;
}
unsafe extern "system" fn d3d9_texture_cube_SetPrivateData(
    iface: *mut IDirect3DCubeTexture9,
    guid: *const GUID,
    data: *const core::ffi::c_void,
    data_size: DWORD,
    flags: DWORD,
) -> HRESULT {
    let texture: *mut d3d9_texture = impl_from_IDirect3DCubeTexture9(iface);
    WINE_TRACE!("iface %p, guid %s, data %p, data_size %lu, flags %#lx.\n", iface, debugstr_guid(guid as *const _GUID), data, data_size, flags);
    return d3d9_resource_set_private_data(
        core::ptr::addr_of_mut!((*texture).resource),
        guid,
        data,
        data_size,
        flags,
    );
}
unsafe extern "system" fn d3d9_texture_cube_GetPrivateData(
    iface: *mut IDirect3DCubeTexture9,
    guid: *const GUID,
    data: *mut core::ffi::c_void,
    data_size: *mut DWORD,
) -> HRESULT {
    let texture: *mut d3d9_texture = impl_from_IDirect3DCubeTexture9(iface);
    WINE_TRACE!("iface %p, guid %s, data %p, data_size %p.\n", iface, debugstr_guid(guid as *const _GUID), data, data_size);
    return d3d9_resource_get_private_data(core::ptr::addr_of_mut!((*texture).resource), guid, data, data_size);
}

unsafe extern "system" fn d3d9_texture_cube_FreePrivateData(iface: *mut IDirect3DCubeTexture9, guid: *const GUID) -> HRESULT {
    let texture: *mut d3d9_texture = impl_from_IDirect3DCubeTexture9(iface);

    return d3d9_resource_free_private_data(&raw mut (*texture).resource, guid);
}

unsafe extern "system" fn d3d9_texture_cube_SetPriority(iface: *mut IDirect3DCubeTexture9, priority: DWORD) -> DWORD {
    let mut texture: *mut d3d9_texture = impl_from_IDirect3DCubeTexture9(iface);
    let mut resource: *mut wined3d_resource = core::ptr::null_mut();
    let mut ret: DWORD = 0;
    WINE_TRACE!("iface %p, priority %lu.\n", iface, priority);
    wined3d_mutex_lock();
    resource = wined3d_texture_get_resource(d3d9_texture_get_draw_texture(texture));
    ret = wined3d_resource_set_priority(resource, priority as u32) as DWORD;
    wined3d_mutex_unlock();

    return ret;
}

unsafe extern "system" fn d3d9_texture_cube_GetPriority(iface: *mut IDirect3DCubeTexture9) -> DWORD {
    let texture: *mut d3d9_texture = impl_from_IDirect3DCubeTexture9(iface);
    let resource: *const wined3d_resource;
    let ret: DWORD;

    TRACE!("iface %p.\n", iface);

    wined3d_mutex_lock();
    resource = wined3d_texture_get_resource(d3d9_texture_get_draw_texture(texture));
    ret = wined3d_resource_get_priority(resource);
    wined3d_mutex_unlock();

    return ret;
}
unsafe extern "system" fn d3d9_texture_cube_PreLoad(iface: *mut IDirect3DCubeTexture9) {
    let texture: *mut d3d9_texture = impl_from_IDirect3DCubeTexture9(iface);

    WINE_TRACE!("iface %p.\n", iface);

    d3d9_texture_preload(texture);
}
unsafe extern "system" fn d3d9_texture_cube_GetType(
    iface: *mut IDirect3DCubeTexture9,
) -> D3DRESOURCETYPE {
    // TRACE("iface %p.\n", iface);
    D3DRTYPE_CUBETEXTURE
}

unsafe extern "system" fn d3d9_texture_cube_SetLOD(iface: *mut IDirect3DCubeTexture9, lod: DWORD) -> DWORD {
    let mut texture: *mut d3d9_texture = impl_from_IDirect3DCubeTexture9(iface);
    let mut ret: DWORD;

    WINE_TRACE!("iface %p, lod %lu.\n", iface, lod);
    wined3d_mutex_lock();
    ret = wined3d_stateblock_set_texture_lod(
        (*(*texture).parent_device).state,
        d3d9_texture_get_draw_texture(texture),
        lod as u32,
    ) as DWORD;
    wined3d_mutex_unlock();

    return ret;
}

unsafe extern "system" fn d3d9_texture_cube_GetLOD(iface: *mut IDirect3DCubeTexture9) -> DWORD {
    let mut texture: *mut d3d9_texture = impl_from_IDirect3DCubeTexture9(iface);
    let mut ret: DWORD = 0;

    wined3d_mutex_lock();
    ret = wined3d_texture_get_lod(d3d9_texture_get_draw_texture(texture)) as DWORD;
    wined3d_mutex_unlock();

    return ret;
}

unsafe extern "system" fn d3d9_texture_cube_GetLevelCount(iface: *mut IDirect3DCubeTexture9) -> DWORD {
    let mut texture: *mut d3d9_texture = impl_from_IDirect3DCubeTexture9(iface);
    let mut ret: DWORD = 0;
    WINE_TRACE!("iface %p.\n", iface);
    if (*texture).usage & (D3DUSAGE_AUTOGENMIPMAP as u32) != 0 {
        return 1;
    }
    wined3d_mutex_lock();
    ret = wined3d_texture_get_level_count((*texture).wined3d_texture) as DWORD;
    wined3d_mutex_unlock();
    return ret;
}
unsafe extern "system" fn d3d9_texture_cube_SetAutoGenFilterType(
    iface: *mut IDirect3DCubeTexture9,
    filter_type: D3DTEXTUREFILTERTYPE,
) -> HRESULT {
    let texture: *mut d3d9_texture = impl_from_IDirect3DCubeTexture9(iface);

    if filter_type == D3DTEXF_NONE {
        WARN!("Invalid filter type D3DTEXF_NONE specified.\n");
        return D3DERR_INVALIDCALL;
    }
    if ((*texture).usage & (D3DUSAGE_AUTOGENMIPMAP as u32)) == 0 {
        WARN!("Called on a texture without the D3DUSAGE_AUTOGENMIPMAP flag.\n");
    } else if filter_type != D3DTEXF_LINEAR {
        FIXME!("Unsupported filter type %u.\n", filter_type);
    }

    (*texture).autogen_filter_type = filter_type;
    return D3D_OK;
}
unsafe extern "system" fn d3d9_texture_cube_GetAutoGenFilterType(iface: *mut IDirect3DCubeTexture9) -> D3DTEXTUREFILTERTYPE {
    let mut texture: *mut d3d9_texture = impl_from_IDirect3DCubeTexture9(iface);

    if (*texture).usage & (D3DUSAGE_AUTOGENMIPMAP as u32) == 0 {
        WARN!("Called on a texture without the D3DUSAGE_AUTOGENMIPMAP flag.\n");
    }

    return (*texture).autogen_filter_type;
}

unsafe extern "system" fn d3d9_texture_cube_GenerateMipSubLevels(iface: *mut IDirect3DCubeTexture9) {
    let mut texture: *mut d3d9_texture = impl_from_IDirect3DCubeTexture9(iface);

    TRACE!("iface %p.\n", iface);
    wined3d_mutex_lock();
    d3d9_texture_gen_auto_mipmap(texture);
    wined3d_mutex_unlock();
}

unsafe extern "system" fn d3d9_texture_cube_GetLevelDesc(iface: *mut IDirect3DCubeTexture9, level: UINT, desc: *mut D3DSURFACE_DESC) -> HRESULT {
    let texture: *mut d3d9_texture = impl_from_IDirect3DCubeTexture9(iface);
    let mut wined3d_desc: wined3d_sub_resource_desc = core::mem::zeroed();
    let mut level_count: DWORD;
    let mut hr: HRESULT;

    TRACE!("iface %p, level %u, desc %p.\n", iface, level, desc);

    if ((*texture).usage & (D3DUSAGE_AUTOGENMIPMAP as u32)) != 0 && level != 0 {
        WARN!("D3DUSAGE_AUTOGENMIPMAP textures have only one accessible level.\n");
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    level_count = wined3d_texture_get_level_count((*texture).wined3d_texture) as DWORD;
    if level >= level_count {
        wined3d_mutex_unlock();
        return D3DERR_INVALIDCALL;
    }

    hr = wined3d_texture_get_sub_resource_desc(
        (*texture).wined3d_texture,
        level as ::core::ffi::c_uint,
        &raw mut wined3d_desc,
    );
    if hr >= 0 {
        (*desc).Format = d3dformat_from_wined3dformat(wined3d_desc.format);
        (*desc).Type = D3DRTYPE_SURFACE;
        (*desc).Usage = (*texture).usage;
        (*desc).Pool = d3dpool_from_wined3daccess(wined3d_desc.access, wined3d_desc.usage);
        (*desc).MultiSampleType = d3dmultisample_type_from_wined3d(wined3d_desc.multisample_type);
        (*desc).MultiSampleQuality = wined3d_desc.multisample_quality as DWORD;
        (*desc).Width = wined3d_desc.width as UINT;
        (*desc).Height = wined3d_desc.height as UINT;
    }
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d9_texture_cube_GetCubeMapSurface(
    mut iface: *mut IDirect3DCubeTexture9,
    mut face: D3DCUBEMAP_FACES,
    mut level: UINT,
    mut surface: *mut *mut IDirect3DSurface9,
) -> HRESULT {
    let mut texture: *mut d3d9_texture = impl_from_IDirect3DCubeTexture9(iface);
    let mut surface_impl: *mut d3d9_surface = core::ptr::null_mut();
    let mut sub_resource_idx: UINT = 0;
    let mut level_count: DWORD = 0;

    TRACE!(
        "iface %p, face %#x, level %u, surface %p.\n",
        iface,
        face as u32,
        level,
        surface
    );

    if ((*texture).usage & (D3DUSAGE_AUTOGENMIPMAP as u32)) != 0 && level != 0 {
        WARN!("D3DUSAGE_AUTOGENMIPMAP textures have only one accessible level.\n");
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    level_count = wined3d_texture_get_level_count((*texture).wined3d_texture) as DWORD;
    if level >= level_count {
        wined3d_mutex_unlock();
        return D3DERR_INVALIDCALL;
    }

    sub_resource_idx = (level_count as UINT)
        .wrapping_mul(face as UINT)
        .wrapping_add(level);
    surface_impl = wined3d_texture_get_sub_resource_parent(
        (*texture).wined3d_texture,
        sub_resource_idx as u32,
    ) as *mut d3d9_surface;
    if surface_impl.is_null() {
        wined3d_mutex_unlock();
        return D3DERR_INVALIDCALL;
    }

    *surface = core::ptr::addr_of_mut!((*surface_impl).IDirect3DSurface9_iface);
    IDirect3DSurface9_AddRef(*surface);
    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d9_texture_cube_LockRect(
    iface: *mut IDirect3DCubeTexture9,
    face: D3DCUBEMAP_FACES,
    level: UINT,
    locked_rect: *mut D3DLOCKED_RECT,
    rect: *const RECT,
    flags: DWORD,
) -> HRESULT {
    let texture: *mut d3d9_texture = impl_from_IDirect3DCubeTexture9(iface);
    let mut surface_impl: *mut d3d9_surface;
    let mut sub_resource_idx: UINT;
    let mut hr: HRESULT;

    if (*texture).usage & (D3DUSAGE_AUTOGENMIPMAP as u32) != 0 && level != 0 {
        WARN!("D3DUSAGE_AUTOGENMIPMAP textures have only one accessible level.\n");
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    sub_resource_idx = (wined3d_texture_get_level_count((*texture).wined3d_texture) as UINT)
        .wrapping_mul(face as UINT)
        .wrapping_add(level);
    surface_impl = wined3d_texture_get_sub_resource_parent(
        (*texture).wined3d_texture,
        sub_resource_idx as ::core::ffi::c_uint,
    ) as *mut d3d9_surface;
    if surface_impl.is_null() {
        hr = D3DERR_INVALIDCALL;
    } else {
        hr = IDirect3DSurface9_LockRect(
            core::ptr::addr_of_mut!((*surface_impl).IDirect3DSurface9_iface),
            locked_rect,
            rect,
            flags,
        );
    }
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d9_texture_cube_UnlockRect(
    iface: *mut IDirect3DCubeTexture9,
    face: D3DCUBEMAP_FACES,
    level: UINT,
) -> HRESULT {
    let texture: *mut d3d9_texture = impl_from_IDirect3DCubeTexture9(iface);
    let mut surface_impl: *mut d3d9_surface;
    let mut sub_resource_idx: UINT;
    let mut hr: HRESULT;

    if (*texture).usage & (D3DUSAGE_AUTOGENMIPMAP as u32) != 0 && level != 0 {
        WARN!("D3DUSAGE_AUTOGENMIPMAP textures have only one accessible level.\n");
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    sub_resource_idx = (wined3d_texture_get_level_count((*texture).wined3d_texture) as UINT)
        .wrapping_mul(face as UINT)
        .wrapping_add(level);
    surface_impl = wined3d_texture_get_sub_resource_parent((*texture).wined3d_texture, sub_resource_idx) as *mut d3d9_surface;
    if surface_impl.is_null() {
        hr = D3DERR_INVALIDCALL;
    } else {
        hr = IDirect3DSurface9_UnlockRect(core::ptr::addr_of_mut!((*surface_impl).IDirect3DSurface9_iface));
    }
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d9_texture_cube_AddDirtyRect(
    iface: *mut IDirect3DCubeTexture9,
    face: D3DCUBEMAP_FACES,
    dirty_rect: *const RECT,
) -> HRESULT {
    let texture: *mut d3d9_texture = impl_from_IDirect3DCubeTexture9(iface);
    let mut hr: HRESULT = 0;

    wined3d_mutex_lock();
    if dirty_rect.is_null() {
        hr = wined3d_texture_add_dirty_region(
            (*texture).wined3d_texture,
            face as UINT,
            core::ptr::null(),
        );
    } else {
        let mut dirty_region: wined3d_box = core::mem::zeroed();

        wined3d_box_set(
            core::ptr::addr_of_mut!(dirty_region),
            (*dirty_rect).left as UINT,
            (*dirty_rect).top as UINT,
            (*dirty_rect).right as UINT,
            (*dirty_rect).bottom as UINT,
            0,
            1,
        );
        hr = wined3d_texture_add_dirty_region(
            (*texture).wined3d_texture,
            face as UINT,
            core::ptr::addr_of_mut!(dirty_region),
        );
    }
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d9_texture_3d_QueryInterface(iface: *mut IDirect3DVolumeTexture9, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DVolumeTexture9))
        != false
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DBaseTexture9)) != false
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DResource9)) != false
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) != false
    {
        IDirect3DVolumeTexture9_AddRef(iface);
        *out = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    WARN!("%s not implemented, returning E_NOINTERFACE.\n", debugstr_guid(riid));

    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn d3d9_texture_3d_AddRef(iface: *mut IDirect3DVolumeTexture9) -> ULONG {
    let texture: *mut d3d9_texture = impl_from_IDirect3DVolumeTexture9(iface);

    return d3d9_texture_incref(texture);
}

unsafe extern "system" fn d3d9_texture_3d_Release(iface: *mut IDirect3DVolumeTexture9) -> ULONG {
    let texture: *mut d3d9_texture = impl_from_IDirect3DVolumeTexture9(iface);

    return d3d9_texture_decref(texture);
}

unsafe extern "system" fn d3d9_texture_3d_GetDevice(
    iface: *mut IDirect3DVolumeTexture9,
    device: *mut *mut IDirect3DDevice9,
) -> HRESULT {
    let texture: *mut d3d9_texture = impl_from_IDirect3DVolumeTexture9(iface);
    WINE_TRACE!("iface %p, device %p.\n", iface, device);

    *device = core::ptr::addr_of_mut!((*(*texture).parent_device).IDirect3DDevice9Ex_iface)
        as *mut IDirect3DDevice9;
    IDirect3DDevice9_AddRef(*device);

    WINE_TRACE!("Returning device %p.\n", *device);

    return D3D_OK;
}
unsafe extern "system" fn d3d9_texture_3d_SetPrivateData(
    iface: *mut IDirect3DVolumeTexture9,
    guid: REFGUID,
    data: *const core::ffi::c_void,
    data_size: DWORD,
    flags: DWORD,
) -> HRESULT {
    let texture: *mut d3d9_texture = impl_from_IDirect3DVolumeTexture9(iface);
    TRACE!(
        "iface %p, guid %s, data %p, data_size %lu, flags %#lx.\n",
        iface,
        debugstr_guid(guid as *const _),
        data,
        data_size,
        flags
    );

    return d3d9_resource_set_private_data(
        core::ptr::addr_of_mut!((*texture).resource),
        guid,
        data,
        data_size,
        flags,
    );
}

unsafe extern "system" fn d3d9_texture_3d_GetPrivateData(iface: *mut IDirect3DVolumeTexture9, guid: REFGUID, data: *mut core::ffi::c_void, data_size: *mut DWORD) -> HRESULT {
    let mut texture: *mut d3d9_texture = impl_from_IDirect3DVolumeTexture9(iface);
    TRACE!("iface %p, guid %s, data %p, data_size %p.\n", iface, debugstr_guid(guid as *const _GUID), data, data_size);

    return d3d9_resource_get_private_data(&raw mut (*texture).resource, guid, data, data_size);
}

unsafe extern "system" fn d3d9_texture_3d_FreePrivateData(iface: *mut IDirect3DVolumeTexture9, guid: *const GUID) -> HRESULT {
    let texture: *mut d3d9_texture = impl_from_IDirect3DVolumeTexture9(iface);

    WINE_TRACE!("iface %p, guid %s.\n", iface, debugstr_guid(guid as *const _GUID));

    return d3d9_resource_free_private_data(core::ptr::addr_of_mut!((*texture).resource), guid);
}

unsafe extern "system" fn d3d9_texture_3d_SetPriority(
    mut iface: *mut IDirect3DVolumeTexture9,
    mut priority: DWORD,
) -> DWORD {
    let mut texture: *mut d3d9_texture = impl_from_IDirect3DVolumeTexture9(iface);
    let mut resource: *mut wined3d_resource = ::core::ptr::null_mut::<wined3d_resource>();
    let mut ret: DWORD = 0;
    WINE_TRACE!("iface %p, priority %lu.\n", iface, priority);
    wined3d_mutex_lock();
    resource = wined3d_texture_get_resource(d3d9_texture_get_draw_texture(texture));
    ret = wined3d_resource_set_priority(resource, priority as ::core::ffi::c_uint) as DWORD;
    wined3d_mutex_unlock();
    return ret;
}

unsafe extern "system" fn d3d9_texture_3d_GetPriority(mut iface: *mut IDirect3DVolumeTexture9) -> DWORD {
    let mut texture: *mut d3d9_texture = impl_from_IDirect3DVolumeTexture9(iface);
    let mut resource: *const wined3d_resource = core::ptr::null::<wined3d_resource>();
    let mut ret: DWORD = 0;
    WINE_TRACE!("iface %p.\n", iface);
    wined3d_mutex_lock();
    resource = wined3d_texture_get_resource(d3d9_texture_get_draw_texture(texture));
    ret = wined3d_resource_get_priority(resource) as DWORD;
    wined3d_mutex_unlock();
    return ret;
}

unsafe extern "system" fn d3d9_texture_3d_PreLoad(iface: *mut IDirect3DVolumeTexture9) {
    let texture: *mut d3d9_texture = impl_from_IDirect3DVolumeTexture9(iface);

    WINE_TRACE!("iface %p.\n", iface);

    d3d9_texture_preload(texture);
}
unsafe extern "system" fn d3d9_texture_3d_GetType(
    iface: *mut IDirect3DVolumeTexture9,
) -> D3DRESOURCETYPE {
    // TRACE("iface %p.\n", iface);
    D3DRTYPE_VOLUMETEXTURE
}

unsafe extern "system" fn d3d9_texture_3d_SetLOD(iface: *mut IDirect3DVolumeTexture9, lod: DWORD) -> DWORD {
    let texture: *mut d3d9_texture = impl_from_IDirect3DVolumeTexture9(iface);
    let mut ret: DWORD;

    TRACE!("iface %p, lod %lu.\n", iface, lod);

    wined3d_mutex_lock();
    ret = wined3d_stateblock_set_texture_lod(
        (*(*texture).parent_device).state,
        d3d9_texture_get_draw_texture(texture),
        lod as core::ffi::c_uint,
    ) as DWORD;
    wined3d_mutex_unlock();

    return ret;
}
unsafe extern "system" fn d3d9_texture_3d_GetLOD(iface: *mut IDirect3DVolumeTexture9) -> DWORD {
    let texture: *mut d3d9_texture = impl_from_IDirect3DVolumeTexture9(iface);
    let mut ret: DWORD;

    // TRACE("iface %p.\n", iface);

    wined3d_mutex_lock();
    ret = wined3d_texture_get_lod(d3d9_texture_get_draw_texture(texture)) as DWORD;
    wined3d_mutex_unlock();

    return ret;
}

unsafe extern "system" fn d3d9_texture_3d_GetLevelCount(iface: *mut IDirect3DVolumeTexture9) -> DWORD {
    let mut texture: *mut d3d9_texture = impl_from_IDirect3DVolumeTexture9(iface);
    let mut ret: DWORD = 0;
    WINE_TRACE!("iface %p.\n", iface);
    wined3d_mutex_lock();
    ret = wined3d_texture_get_level_count((*texture).wined3d_texture) as DWORD;
    wined3d_mutex_unlock();
    return ret;
}
unsafe extern "system" fn d3d9_texture_3d_SetAutoGenFilterType(
    iface: *mut IDirect3DVolumeTexture9,
    filter_type: D3DTEXTUREFILTERTYPE,
) -> HRESULT {
    // TRACE("iface %p, filter_type %#x.\n", iface, filter_type);
    D3DERR_INVALIDCALL
}
unsafe extern "system" fn d3d9_texture_3d_GetAutoGenFilterType(
    iface: *mut IDirect3DVolumeTexture9,
) -> D3DTEXTUREFILTERTYPE {
    // TRACE("iface %p.\n", iface);
    D3DTEXF_NONE
}
unsafe extern "system" fn d3d9_texture_3d_GenerateMipSubLevels(
    iface: *mut IDirect3DVolumeTexture9,
) {
    // TRACE("iface %p.\n", iface);
}

unsafe extern "system" fn d3d9_texture_3d_GetLevelDesc(
    iface: *mut IDirect3DVolumeTexture9,
    level: UINT,
    desc: *mut D3DVOLUME_DESC,
) -> HRESULT {
    let texture: *mut d3d9_texture = impl_from_IDirect3DVolumeTexture9(iface);
    let mut wined3d_desc: wined3d_sub_resource_desc = core::mem::zeroed();
    let mut hr: HRESULT;

    WINE_TRACE!("iface %p, level %u, desc %p.\n", iface, level, desc);

    wined3d_mutex_lock();
    hr = wined3d_texture_get_sub_resource_desc((*texture).wined3d_texture, level as u32, &raw mut wined3d_desc);
    if hr >= 0 {
        (*desc).Format = d3dformat_from_wined3dformat(wined3d_desc.format);
        (*desc).Type = D3DRTYPE_VOLUME;
        (*desc).Usage = (*texture).usage;
        (*desc).Pool = d3dpool_from_wined3daccess(wined3d_desc.access, wined3d_desc.usage);
        (*desc).Width = wined3d_desc.width as UINT;
        (*desc).Height = wined3d_desc.height as UINT;
        (*desc).Depth = wined3d_desc.depth as UINT;
    }
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d9_texture_3d_GetVolumeLevel(
    iface: *mut IDirect3DVolumeTexture9,
    level: UINT,
    volume: *mut *mut IDirect3DVolume9,
) -> HRESULT {
    let texture: *mut d3d9_texture = impl_from_IDirect3DVolumeTexture9(iface);
    let mut volume_impl: *mut d3d9_volume = core::ptr::null_mut();

    wined3d_mutex_lock();
    volume_impl = wined3d_texture_get_sub_resource_parent((*texture).wined3d_texture, level) as *mut d3d9_volume;
    if volume_impl.is_null() {
        wined3d_mutex_unlock();
        return D3DERR_INVALIDCALL;
    }

    *volume = core::ptr::addr_of_mut!((*volume_impl).IDirect3DVolume9_iface);
    IDirect3DVolume9_AddRef(*volume);
    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d9_texture_3d_LockBox(
    iface: *mut IDirect3DVolumeTexture9,
    level: UINT,
    locked_box: *mut D3DLOCKED_BOX,
    box_: *const D3DBOX,
    flags: DWORD,
) -> HRESULT {
    let texture: *mut d3d9_texture = impl_from_IDirect3DVolumeTexture9(iface);
    let mut volume_impl: *mut d3d9_volume = core::ptr::null_mut();
    let mut hr: HRESULT;

    wined3d_mutex_lock();
    volume_impl = wined3d_texture_get_sub_resource_parent((*texture).wined3d_texture, level) as *mut d3d9_volume;
    if volume_impl.is_null() {
        hr = D3DERR_INVALIDCALL;
    } else {
        hr = IDirect3DVolume9_LockBox(
            core::ptr::addr_of_mut!((*volume_impl).IDirect3DVolume9_iface),
            locked_box,
            box_,
            flags,
        );
    }
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d9_texture_3d_UnlockBox(iface: *mut IDirect3DVolumeTexture9, level: UINT) -> HRESULT {
    let mut texture: *mut d3d9_texture = impl_from_IDirect3DVolumeTexture9(iface);
    let mut volume_impl: *mut d3d9_volume = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    wined3d_mutex_lock();
    volume_impl = wined3d_texture_get_sub_resource_parent((*texture).wined3d_texture, level) as *mut d3d9_volume;
    if volume_impl.is_null() {
        hr = D3DERR_INVALIDCALL;
    } else {
        hr = IDirect3DVolume9_UnlockBox(core::ptr::addr_of_mut!((*volume_impl).IDirect3DVolume9_iface));
    }
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d9_texture_3d_AddDirtyBox(iface: *mut IDirect3DVolumeTexture9, dirty_box: *const D3DBOX) -> HRESULT {
    let texture: *mut d3d9_texture = impl_from_IDirect3DVolumeTexture9(iface);
    let mut hr: HRESULT;

    wined3d_mutex_lock();
    hr = wined3d_texture_add_dirty_region((*texture).wined3d_texture, 0, dirty_box as *const wined3d_box);
    wined3d_mutex_unlock();

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn unsafe_impl_from_IDirect3DBaseTexture9(mut iface: *mut IDirect3DBaseTexture9) -> *mut d3d9_texture {
    if iface.is_null() {
        return core::ptr::null_mut();
    }

    if (*iface).lpVtbl != core::ptr::addr_of!(d3d9_texture_2d_vtbl) as *const IDirect3DBaseTexture9Vtbl
        && (*iface).lpVtbl != core::ptr::addr_of!(d3d9_texture_cube_vtbl) as *const IDirect3DBaseTexture9Vtbl
        && (*iface).lpVtbl != core::ptr::addr_of!(d3d9_texture_3d_vtbl) as *const IDirect3DBaseTexture9Vtbl
    {
        WARN!("%p is not a valid IDirect3DBaseTexture9 interface.\n", iface);
        return core::ptr::null_mut();
    }

    return (iface as usize).wrapping_sub(
        core::mem::offset_of!(d3d9_texture, IDirect3DBaseTexture9_iface),
    ) as *mut d3d9_texture;
}
unsafe extern "system" fn d3d9_texture_wined3d_object_destroyed(parent: *mut ::core::ffi::c_void) {
    let mut texture: *mut d3d9_texture = parent as *mut d3d9_texture;
    if !(*texture).draw_texture.is_null() {
        wined3d_texture_decref((*texture).wined3d_texture);
    }
    d3d9_resource_cleanup(core::ptr::addr_of_mut!((*texture).resource));
    free(texture as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn d3d9_texture_init(texture: *mut d3d9_texture, device: *mut d3d9_device, desc: *const wined3d_resource_desc, pool: D3DPOOL, usage: DWORD, layer_count: ::core::ffi::c_uint, level_count: ::core::ffi::c_uint) -> HRESULT {
    let mut wined3d_adapter: *mut wined3d_adapter = core::ptr::null_mut();
    let mut output_idx: ::core::ffi::c_uint;
    let mut flags: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT;
    let mut level_count = level_count;

    if pool == D3DPOOL_MANAGED && (*(*device).d3d_parent).extended != 0 {
        WARN!("Managed resources are not supported by d3d9ex devices.\n");
        return D3DERR_INVALIDCALL;
    }

    d3d9_resource_init(&raw mut (*texture).resource);
    list_init(&raw mut (*texture).rtv_list);
    (*texture).usage = usage;

    if (*desc).resource_type == WINED3D_RTYPE_TEXTURE_2D && is_gdi_compat_wined3dformat((*desc).format) != 0 {
        flags |= WINED3D_TEXTURE_CREATE_GET_DC;
    }
    if pool == D3DPOOL_SYSTEMMEM || pool == D3DPOOL_MANAGED {
        flags |= WINED3D_TEXTURE_CREATE_RECORD_DIRTY_REGIONS;
    }

    if (usage & (D3DUSAGE_WRITEONLY as DWORD)) != 0 {
        WARN!("Texture can't be created with the D3DUSAGE_WRITEONLY flags, returning D3DERR_INVALIDCALL.\n");
        return D3DERR_INVALIDCALL;
    }
    if (usage & (D3DUSAGE_AUTOGENMIPMAP as DWORD)) != 0 {
        if (*desc).resource_type == WINED3D_RTYPE_TEXTURE_3D {
            WARN!("Texture can't be created with the D3DUSAGE_WRITEONLY flags, returning D3DERR_INVALIDCALL.\n");
            return D3DERR_INVALIDCALL;
        }

        if pool == D3DPOOL_SYSTEMMEM {
            WARN!("D3DUSAGE_AUTOGENMIPMAP texture can't be in D3DPOOL_SYSTEMMEM, returning D3DERR_INVALIDCALL.\n");
            return D3DERR_INVALIDCALL;
        }

        if level_count != 0 && level_count != 1 {
            WARN!("D3DUSAGE_AUTOGENMIPMAP texture with %u level_count, returning D3DERR_INVALIDCALL.\n", level_count);
            return D3DERR_INVALIDCALL;
        }
        wined3d_mutex_lock();
        output_idx = (*device).adapter_ordinal;
        wined3d_adapter = wined3d_output_get_adapter(
            *(*(*device).d3d_parent).wined3d_outputs.offset(output_idx as isize),
        );
        hr = wined3d_check_device_format(
            (*(*device).d3d_parent).wined3d,
            wined3d_adapter,
            WINED3D_DEVICE_TYPE_HAL,
            WINED3DFMT_B8G8R8A8_UNORM,
            WINED3DUSAGE_QUERY_GENMIPMAP,
            WINED3D_BIND_SHADER_RESOURCE,
            WINED3D_RTYPE_TEXTURE_2D,
            (*desc).format,
        );
        wined3d_mutex_unlock();
        if hr == D3D_OK {
            flags |= WINED3D_TEXTURE_CREATE_GENERATE_MIPMAPS;
            level_count = 0;
        } else {
            WARN!("Mipmap autogeneration is not supported on wined3d format %#x, creating a texture with a single level.\n", (*desc).format);
            level_count = 1;
        }
        (*texture).autogen_filter_type = D3DTEXF_LINEAR;
    } else {
        (*texture).autogen_filter_type = D3DTEXF_NONE;
    }
    if level_count == 0 {
        level_count = wined3d_log2i(max!(max!((*desc).width, (*desc).height), (*desc).depth)) + 1;
    }

    wined3d_mutex_lock();
    if pool == D3DPOOL_MANAGED {
        let mut managed_desc: wined3d_resource_desc = *desc;

        managed_desc.access = WINED3D_RESOURCE_ACCESS_CPU | WINED3D_RESOURCE_ACCESS_MAP_R | WINED3D_RESOURCE_ACCESS_MAP_W;
        managed_desc.bind_flags = 0;
        hr = wined3d_texture_create(
            (*device).wined3d_device,
            &raw mut managed_desc,
            layer_count,
            level_count,
            flags,
            core::ptr::null_mut(),
            (texture).cast(),
            &raw const d3d9_null_wined3d_parent_ops,
            &raw mut (*texture).wined3d_texture,
        );
        if hr < 0 {
            wined3d_mutex_unlock();
            WARN!("Failed to create sysmem texture, hr %#lx.\n", hr);
            return hr;
        }

        managed_desc.access = WINED3D_RESOURCE_ACCESS_GPU;
        managed_desc.bind_flags = (*desc).bind_flags;
        flags &= WINED3D_TEXTURE_CREATE_GENERATE_MIPMAPS;
        hr = wined3d_texture_create(
            (*device).wined3d_device,
            &raw mut managed_desc,
            layer_count,
            level_count,
            flags,
            core::ptr::null_mut(),
            (texture).cast(),
            &raw const d3d9_texture_wined3d_parent_ops,
            &raw mut (*texture).draw_texture,
        );
        if hr < 0 {
            wined3d_texture_decref((*texture).wined3d_texture);
            wined3d_mutex_unlock();
            WARN!("Failed to create draw texture, hr %#lx.\n", hr);
            return hr;
        }
    } else {
        hr = wined3d_texture_create(
            (*device).wined3d_device,
            desc,
            layer_count,
            level_count,
            flags,
            core::ptr::null_mut(),
            (texture).cast(),
            &raw const d3d9_texture_wined3d_parent_ops,
            &raw mut (*texture).wined3d_texture,
        );
        if hr < 0 {
            wined3d_mutex_unlock();
            WARN!("Failed to create wined3d texture, hr %#lx.\n", hr);
            return hr;
        }
    }
    wined3d_mutex_unlock();

    (*texture).parent_device = device;
    IDirect3DDevice9Ex_AddRef(&raw mut (*(*texture).parent_device).IDirect3DDevice9Ex_iface);

    return D3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn d3d9_texture_2d_init(
    texture: *mut d3d9_texture,
    device: *mut d3d9_device,
    width: ::core::ffi::c_uint,
    height: ::core::ffi::c_uint,
    level_count: ::core::ffi::c_uint,
    usage: DWORD,
    format: D3DFORMAT,
    pool: D3DPOOL,
) -> HRESULT {
    let mut desc: wined3d_resource_desc = core::mem::zeroed();

    desc.resource_type = WINED3D_RTYPE_TEXTURE_2D;
    desc.format = wined3dformat_from_d3dformat(format);
    desc.multisample_type = WINED3D_MULTISAMPLE_NONE;
    desc.multisample_quality = 0;
    desc.usage = wined3d_usage_from_d3d(pool, usage);
    desc.bind_flags = wined3d_bind_flags_from_d3d9_usage(usage) | WINED3D_BIND_SHADER_RESOURCE;
    desc.access = wined3daccess_from_d3dpool(pool, usage);
    desc.width = width;
    desc.height = height;
    desc.depth = 1;
    desc.size = 0;

    if (*(*device).d3d_parent).extended == 0 {
        desc.usage |= WINED3DUSAGE_VIDMEM_ACCOUNTING;
    }

    (*texture).IDirect3DBaseTexture9_iface.lpVtbl =
        core::ptr::addr_of!(d3d9_texture_2d_vtbl) as *const IDirect3DBaseTexture9Vtbl;
    return d3d9_texture_init(
        texture,
        device,
        &mut desc,
        pool,
        usage,
        1,
        level_count,
    );
}
#[no_mangle]
pub unsafe extern "C" fn d3d9_texture_cube_init(texture: *mut d3d9_texture, device: *mut d3d9_device, edge_length: core::ffi::c_uint, level_count: core::ffi::c_uint, usage: DWORD, format: D3DFORMAT, pool: D3DPOOL) -> HRESULT {
    let mut desc: wined3d_resource_desc = core::mem::zeroed();

    desc.resource_type = WINED3D_RTYPE_TEXTURE_2D;
    desc.format = wined3dformat_from_d3dformat(format);
    desc.multisample_type = WINED3D_MULTISAMPLE_NONE;
    desc.multisample_quality = 0;
    desc.usage = wined3d_usage_from_d3d(pool, usage) | WINED3DUSAGE_LEGACY_CUBEMAP;
    desc.bind_flags = wined3d_bind_flags_from_d3d9_usage(usage) | WINED3D_BIND_SHADER_RESOURCE;
    desc.access = wined3daccess_from_d3dpool(pool, usage);
    desc.width = edge_length;
    desc.height = edge_length;
    desc.depth = 1;
    desc.size = 0;

    if (*(*device).d3d_parent).extended == 0 {
        desc.usage |= WINED3DUSAGE_VIDMEM_ACCOUNTING;
    }

    (*texture).IDirect3DBaseTexture9_iface.lpVtbl = core::ptr::addr_of!(d3d9_texture_cube_vtbl) as *const IDirect3DBaseTexture9Vtbl;
    return d3d9_texture_init(texture, device, &mut desc, pool, usage, 6, level_count);
}
#[no_mangle]
pub unsafe extern "C" fn d3d9_texture_3d_init(
    texture: *mut d3d9_texture,
    device: *mut d3d9_device,
    width: ::core::ffi::c_uint,
    height: ::core::ffi::c_uint,
    depth: ::core::ffi::c_uint,
    level_count: ::core::ffi::c_uint,
    usage: DWORD,
    format: D3DFORMAT,
    pool: D3DPOOL,
) -> HRESULT {
    let mut desc: wined3d_resource_desc = core::mem::zeroed();

    /* In d3d9, 3D textures can't be used as rendertarget or depth/stencil buffer. */
    if (usage & ((D3DUSAGE_RENDERTARGET | D3DUSAGE_DEPTHSTENCIL) as DWORD)) != 0 {
        return D3DERR_INVALIDCALL;
    }

    desc.resource_type = WINED3D_RTYPE_TEXTURE_3D;
    desc.format = wined3dformat_from_d3dformat(format);
    desc.multisample_type = WINED3D_MULTISAMPLE_NONE;
    desc.multisample_quality = 0;
    desc.usage = wined3d_usage_from_d3d(pool, usage);
    desc.bind_flags = wined3d_bind_flags_from_d3d9_usage(usage) | WINED3D_BIND_SHADER_RESOURCE;
    desc.access = wined3daccess_from_d3dpool(pool, usage);
    desc.width = width;
    desc.height = height;
    desc.depth = depth;
    desc.size = 0;

    if (*(*device).d3d_parent).extended == 0 {
        desc.usage |= WINED3DUSAGE_VIDMEM_ACCOUNTING;
    }

    (*texture).IDirect3DBaseTexture9_iface.lpVtbl =
        core::ptr::addr_of!(d3d9_texture_3d_vtbl) as *const IDirect3DBaseTexture9Vtbl;
    return d3d9_texture_init(
        texture,
        device,
        core::ptr::addr_of_mut!(desc),
        pool,
        usage,
        1,
        level_count,
    );
}

// COM vtable tables — generated from texture.c by vtbl_gen.py
/// texture.c:457 `static const IDirect3DTexture9Vtbl d3d9_texture_2d_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d9_texture_2d_vtbl: IDirect3DTexture9Vtbl = IDirect3DTexture9Vtbl {
    QueryInterface: Some(d3d9_texture_2d_QueryInterface),
    AddRef: Some(d3d9_texture_2d_AddRef),
    Release: Some(d3d9_texture_2d_Release),
    GetDevice: Some(d3d9_texture_2d_GetDevice),
    SetPrivateData: Some(d3d9_texture_2d_SetPrivateData),
    GetPrivateData: Some(d3d9_texture_2d_GetPrivateData),
    FreePrivateData: Some(d3d9_texture_2d_FreePrivateData),
    SetPriority: Some(d3d9_texture_2d_SetPriority),
    GetPriority: Some(d3d9_texture_2d_GetPriority),
    PreLoad: Some(d3d9_texture_2d_PreLoad),
    GetType: Some(d3d9_texture_2d_GetType),
    SetLOD: Some(d3d9_texture_2d_SetLOD),
    GetLOD: Some(d3d9_texture_2d_GetLOD),
    GetLevelCount: Some(d3d9_texture_2d_GetLevelCount),
    SetAutoGenFilterType: Some(d3d9_texture_2d_SetAutoGenFilterType),
    GetAutoGenFilterType: Some(d3d9_texture_2d_GetAutoGenFilterType),
    GenerateMipSubLevels: Some(d3d9_texture_2d_GenerateMipSubLevels),
    GetLevelDesc: Some(d3d9_texture_2d_GetLevelDesc),
    GetSurfaceLevel: Some(d3d9_texture_2d_GetSurfaceLevel),
    LockRect: Some(d3d9_texture_2d_LockRect),
    UnlockRect: Some(d3d9_texture_2d_UnlockRect),
    AddDirtyRect: Some(d3d9_texture_2d_AddDirtyRect),
};

/// texture.c:858 `static const IDirect3DCubeTexture9Vtbl d3d9_texture_cube_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d9_texture_cube_vtbl: IDirect3DCubeTexture9Vtbl = IDirect3DCubeTexture9Vtbl {
    QueryInterface: Some(d3d9_texture_cube_QueryInterface),
    AddRef: Some(d3d9_texture_cube_AddRef),
    Release: Some(d3d9_texture_cube_Release),
    GetDevice: Some(d3d9_texture_cube_GetDevice),
    SetPrivateData: Some(d3d9_texture_cube_SetPrivateData),
    GetPrivateData: Some(d3d9_texture_cube_GetPrivateData),
    FreePrivateData: Some(d3d9_texture_cube_FreePrivateData),
    SetPriority: Some(d3d9_texture_cube_SetPriority),
    GetPriority: Some(d3d9_texture_cube_GetPriority),
    PreLoad: Some(d3d9_texture_cube_PreLoad),
    GetType: Some(d3d9_texture_cube_GetType),
    SetLOD: Some(d3d9_texture_cube_SetLOD),
    GetLOD: Some(d3d9_texture_cube_GetLOD),
    GetLevelCount: Some(d3d9_texture_cube_GetLevelCount),
    SetAutoGenFilterType: Some(d3d9_texture_cube_SetAutoGenFilterType),
    GetAutoGenFilterType: Some(d3d9_texture_cube_GetAutoGenFilterType),
    GenerateMipSubLevels: Some(d3d9_texture_cube_GenerateMipSubLevels),
    GetLevelDesc: Some(d3d9_texture_cube_GetLevelDesc),
    GetCubeMapSurface: Some(d3d9_texture_cube_GetCubeMapSurface),
    LockRect: Some(d3d9_texture_cube_LockRect),
    UnlockRect: Some(d3d9_texture_cube_UnlockRect),
    AddDirtyRect: Some(d3d9_texture_cube_AddDirtyRect),
};

/// texture.c:1174 `static const IDirect3DVolumeTexture9Vtbl d3d9_texture_3d_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d9_texture_3d_vtbl: IDirect3DVolumeTexture9Vtbl = IDirect3DVolumeTexture9Vtbl {
    QueryInterface: Some(d3d9_texture_3d_QueryInterface),
    AddRef: Some(d3d9_texture_3d_AddRef),
    Release: Some(d3d9_texture_3d_Release),
    GetDevice: Some(d3d9_texture_3d_GetDevice),
    SetPrivateData: Some(d3d9_texture_3d_SetPrivateData),
    GetPrivateData: Some(d3d9_texture_3d_GetPrivateData),
    FreePrivateData: Some(d3d9_texture_3d_FreePrivateData),
    SetPriority: Some(d3d9_texture_3d_SetPriority),
    GetPriority: Some(d3d9_texture_3d_GetPriority),
    PreLoad: Some(d3d9_texture_3d_PreLoad),
    GetType: Some(d3d9_texture_3d_GetType),
    SetLOD: Some(d3d9_texture_3d_SetLOD),
    GetLOD: Some(d3d9_texture_3d_GetLOD),
    GetLevelCount: Some(d3d9_texture_3d_GetLevelCount),
    SetAutoGenFilterType: Some(d3d9_texture_3d_SetAutoGenFilterType),
    GetAutoGenFilterType: Some(d3d9_texture_3d_GetAutoGenFilterType),
    GenerateMipSubLevels: Some(d3d9_texture_3d_GenerateMipSubLevels),
    GetLevelDesc: Some(d3d9_texture_3d_GetLevelDesc),
    GetVolumeLevel: Some(d3d9_texture_3d_GetVolumeLevel),
    LockBox: Some(d3d9_texture_3d_LockBox),
    UnlockBox: Some(d3d9_texture_3d_UnlockBox),
    AddDirtyBox: Some(d3d9_texture_3d_AddDirtyBox),
};
