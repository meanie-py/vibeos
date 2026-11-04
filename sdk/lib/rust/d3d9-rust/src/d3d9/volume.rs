use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)

static mut d3d9_volume_wined3d_parent_ops: wined3d_parent_ops = unsafe {
    wined3d_parent_ops {
        wined3d_object_destroyed: Some(
            volume_wined3d_object_destroyed,
        ),
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(d3d9): debug channel declaration is provided by the crate's debug macros.

unsafe extern "C" fn impl_from_IDirect3DVolume9(iface: *mut IDirect3DVolume9) -> *mut d3d9_volume {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(d3d9_volume, IDirect3DVolume9_iface)) as *mut d3d9_volume;
}

unsafe extern "system" fn d3d9_volume_QueryInterface(iface: *mut IDirect3DVolume9, riid: *const IID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DVolume9)) != false
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) != false
    {
        IDirect3DVolume9_AddRef(iface);
        *out = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    WARN!("%s not implemented, returning E_NOINTERFACE.\n", debugstr_guid(riid));

    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn d3d9_volume_AddRef(iface: *mut IDirect3DVolume9) -> ULONG {
    let mut volume: *mut d3d9_volume = impl_from_IDirect3DVolume9(iface);
    WINE_TRACE!("iface %p.\n", iface);
    WINE_TRACE!("Forwarding to %p.\n", (*volume).texture);
    return IDirect3DBaseTexture9_AddRef(core::ptr::addr_of_mut!(
        (*(*volume).texture).IDirect3DBaseTexture9_iface
    ));
}

unsafe extern "system" fn d3d9_volume_Release(iface: *mut IDirect3DVolume9) -> ULONG {
    let mut volume: *mut d3d9_volume = impl_from_IDirect3DVolume9(iface);

    TRACE!("iface %p.\n", iface);
    TRACE!("Forwarding to %p.\n", (*volume).texture);

    return IDirect3DBaseTexture9_Release(core::ptr::addr_of_mut!((*(*volume).texture).IDirect3DBaseTexture9_iface));
}

unsafe extern "system" fn d3d9_volume_GetDevice(iface: *mut IDirect3DVolume9, device: *mut *mut IDirect3DDevice9) -> HRESULT {
    let mut volume: *mut d3d9_volume = impl_from_IDirect3DVolume9(iface);

    return IDirect3DBaseTexture9_GetDevice(
        core::ptr::addr_of_mut!((*(*volume).texture).IDirect3DBaseTexture9_iface),
        device,
    );
}

unsafe extern "system" fn d3d9_volume_SetPrivateData(iface: *mut IDirect3DVolume9, guid: REFGUID, data: *const core::ffi::c_void, data_size: DWORD, flags: DWORD) -> HRESULT {
    let volume: *mut d3d9_volume = impl_from_IDirect3DVolume9(iface);
    WINE_TRACE!("iface %p, guid %s, data %p, data_size %lu, flags %#lx.\n", iface, debugstr_guid(guid as *const _GUID), data, data_size, flags);
    return d3d9_resource_set_private_data(
        core::ptr::addr_of_mut!((*volume).resource),
        guid,
        data,
        data_size,
        flags,
    );
}

unsafe extern "system" fn d3d9_volume_GetPrivateData(
    iface: *mut IDirect3DVolume9,
    guid: REFGUID,
    data: *mut core::ffi::c_void,
    data_size: *mut DWORD,
) -> HRESULT {
    let volume: *mut d3d9_volume = impl_from_IDirect3DVolume9(iface);
    WINE_TRACE!("iface %p, guid %s, data %p, data_size %p.\n", iface, debugstr_guid(guid as *const _GUID), data, data_size);
    return d3d9_resource_get_private_data(
        core::ptr::addr_of_mut!((*volume).resource),
        guid,
        data,
        data_size,
    );
}

unsafe extern "system" fn d3d9_volume_FreePrivateData(
    iface: *mut IDirect3DVolume9,
    guid: *const GUID,
) -> HRESULT {
    let volume: *mut d3d9_volume = impl_from_IDirect3DVolume9(iface);
    WINE_TRACE!("iface %p, guid %s.\n", iface, debugstr_guid(guid));
    return d3d9_resource_free_private_data(core::ptr::addr_of_mut!((*volume).resource), guid);
}

unsafe extern "system" fn d3d9_volume_GetContainer(iface: *mut IDirect3DVolume9, riid: REFIID, container: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut volume: *mut d3d9_volume = impl_from_IDirect3DVolume9(iface);

    return IDirect3DBaseTexture9_QueryInterface(core::ptr::addr_of_mut!((*(*volume).texture).IDirect3DBaseTexture9_iface), riid, container);
}

unsafe extern "system" fn d3d9_volume_GetDesc(iface: *mut IDirect3DVolume9, desc: *mut D3DVOLUME_DESC) -> HRESULT {
    let mut volume: *mut d3d9_volume = impl_from_IDirect3DVolume9(iface);
    let mut wined3d_desc: wined3d_sub_resource_desc = core::mem::zeroed();

    wined3d_mutex_lock();
    wined3d_texture_get_sub_resource_desc((*volume).wined3d_texture, (*volume).sub_resource_idx, core::ptr::addr_of_mut!(wined3d_desc));
    wined3d_mutex_unlock();

    (*desc).Format = d3dformat_from_wined3dformat(wined3d_desc.format);
    (*desc).Type = D3DRTYPE_VOLUME;
    (*desc).Usage = d3dusage_from_wined3dusage(wined3d_desc.usage, wined3d_desc.bind_flags);
    (*desc).Pool = d3dpool_from_wined3daccess(wined3d_desc.access, wined3d_desc.usage);
    (*desc).Width = wined3d_desc.width as UINT;
    (*desc).Height = wined3d_desc.height as UINT;
    (*desc).Depth = wined3d_desc.depth as UINT;

    return D3D_OK;
}

unsafe extern "system" fn d3d9_volume_LockBox(
    iface: *mut IDirect3DVolume9,
    locked_box: *mut D3DLOCKED_BOX,
    box_0: *const D3DBOX,
    flags: DWORD,
) -> HRESULT {
    let volume: *mut d3d9_volume = impl_from_IDirect3DVolume9(iface);
    let mut map_desc: wined3d_map_desc = wined3d_map_desc {
        row_pitch: 0,
        slice_pitch: 0,
        data: core::ptr::null_mut::<core::ffi::c_void>(),
    };
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, locked_box %p, box %p, flags %#lx.\n", iface, locked_box, box_0, flags);
    if ({
        hr = wined3d_resource_map(
            wined3d_texture_get_resource((*volume).wined3d_texture),
            (*volume).sub_resource_idx,
            core::ptr::addr_of_mut!(map_desc),
            box_0 as *const wined3d_box,
            wined3dmapflags_from_d3dmapflags(flags, 0) as uint32_t,
        );
        hr
    } < 0)
    {
        map_desc.data = core::ptr::null_mut();
    }

    (*locked_box).RowPitch = map_desc.row_pitch as INT;
    (*locked_box).SlicePitch = map_desc.slice_pitch as INT;
    (*locked_box).pBits = map_desc.data;

    if hr == E_INVALIDARG {
        return D3DERR_INVALIDCALL;
    }
    return hr;
}
unsafe extern "system" fn d3d9_volume_UnlockBox(iface: *mut IDirect3DVolume9) -> HRESULT {
    let volume: *mut d3d9_volume = impl_from_IDirect3DVolume9(iface);
    let mut hr: HRESULT;

    TRACE!("iface %p.\n", iface);

    hr = wined3d_resource_unmap(
        wined3d_texture_get_resource((*volume).wined3d_texture),
        (*volume).sub_resource_idx,
    );

    if hr == WINEDDERR_NOTLOCKED {
        return D3DERR_INVALIDCALL;
    }
    return hr;
}
unsafe extern "system" fn volume_wined3d_object_destroyed(parent: *mut core::ffi::c_void) {
    let volume: *mut d3d9_volume = parent as *mut d3d9_volume;
    d3d9_resource_cleanup(core::ptr::addr_of_mut!((*volume).resource));
    free(volume as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn volume_init(
    volume: *mut d3d9_volume,
    wined3d_texture: *mut wined3d_texture,
    sub_resource_idx: ::core::ffi::c_uint,
    parent_ops: *mut *const wined3d_parent_ops,
) {
    (*volume).IDirect3DVolume9_iface.lpVtbl =
        core::ptr::addr_of!(d3d9_volume_vtbl) as *const IDirect3DVolume9Vtbl;
    d3d9_resource_init(core::ptr::addr_of_mut!((*volume).resource));
    (*volume).resource.refcount = 0;
    (*volume).texture = wined3d_texture_get_parent(wined3d_texture) as *mut d3d9_texture;
    (*volume).wined3d_texture = wined3d_texture;
    (*volume).sub_resource_idx = sub_resource_idx;

    *parent_ops = core::ptr::addr_of!(d3d9_volume_wined3d_parent_ops);
}

// COM vtable tables — generated from volume.c by vtbl_gen.py
/// volume.c:175 `static const IDirect3DVolume9Vtbl d3d9_volume_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d9_volume_vtbl: IDirect3DVolume9Vtbl = IDirect3DVolume9Vtbl {
    QueryInterface: Some(d3d9_volume_QueryInterface),
    AddRef: Some(d3d9_volume_AddRef),
    Release: Some(d3d9_volume_Release),
    GetDevice: Some(d3d9_volume_GetDevice),
    SetPrivateData: Some(d3d9_volume_SetPrivateData),
    GetPrivateData: Some(d3d9_volume_GetPrivateData),
    FreePrivateData: Some(d3d9_volume_FreePrivateData),
    GetContainer: Some(d3d9_volume_GetContainer),
    GetDesc: Some(d3d9_volume_GetDesc),
    LockBox: Some(d3d9_volume_LockBox),
    UnlockBox: Some(d3d9_volume_UnlockBox),
};
