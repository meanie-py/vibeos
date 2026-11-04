use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)

static mut d3d8_volume_wined3d_parent_ops: wined3d_parent_ops = unsafe {
    wined3d_parent_ops {
        wined3d_object_destroyed: Some(
            volume_wined3d_object_destroyed,
        ),
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(d3d8): debug channel declaration for this file
pub static DBCH_D3D8: [u8; 5] = *b"d3d8\0";

unsafe extern "C" fn impl_from_IDirect3DVolume8(iface: *mut IDirect3DVolume8) -> *mut d3d8_volume {
    return CONTAINING_RECORD!(iface, d3d8_volume, IDirect3DVolume8_iface);
}

unsafe extern "system" fn d3d8_volume_QueryInterface(iface: *mut IDirect3DVolume8, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DVolume8)) != false
            || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) != false
    {
        IDirect3DVolume8_AddRef(iface);
        *out = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    WARN!("%s not implemented, returning E_NOINTERFACE.\n", debugstr_guid(riid));

    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn d3d8_volume_AddRef(iface: *mut IDirect3DVolume8) -> ULONG {
    let mut volume: *mut d3d8_volume = impl_from_IDirect3DVolume8(iface);
    WINE_TRACE!("iface %p.\n", iface);
    WINE_TRACE!("Forwarding to %p.\n", (*volume).texture);
    return IDirect3DBaseTexture8_AddRef(core::ptr::addr_of_mut!(
        (*(*volume).texture).IDirect3DBaseTexture8_iface
    ));
}
unsafe extern "system" fn d3d8_volume_Release(iface: *mut IDirect3DVolume8) -> ULONG {
    let mut volume: *mut d3d8_volume = impl_from_IDirect3DVolume8(iface);

    WINE_TRACE!("iface %p.\n", iface);
    WINE_TRACE!("Forwarding to %p.\n", (*volume).texture);

    return IDirect3DBaseTexture8_Release(
        core::ptr::addr_of_mut!((*(*volume).texture).IDirect3DBaseTexture8_iface),
    );
}

unsafe extern "system" fn d3d8_volume_GetDevice(iface: *mut IDirect3DVolume8, device: *mut *mut IDirect3DDevice8) -> HRESULT {
    let mut volume: *mut d3d8_volume = impl_from_IDirect3DVolume8(iface);

    return IDirect3DBaseTexture8_GetDevice(core::ptr::addr_of_mut!((*(*volume).texture).IDirect3DBaseTexture8_iface), device);
}
unsafe extern "system" fn d3d8_volume_SetPrivateData(iface: *mut IDirect3DVolume8, guid: REFGUID, data: *const core::ffi::c_void, data_size: DWORD, flags: DWORD) -> HRESULT {
    let mut volume: *mut d3d8_volume = impl_from_IDirect3DVolume8(iface);

    return d3d8_resource_set_private_data(core::ptr::addr_of_mut!((*volume).resource), guid, data, data_size, flags);
}
unsafe extern "system" fn d3d8_volume_GetPrivateData(iface: *mut IDirect3DVolume8, guid: *const GUID, data: *mut ::core::ffi::c_void, data_size: *mut DWORD) -> HRESULT {
    let volume: *mut d3d8_volume = impl_from_IDirect3DVolume8(iface);

    return d3d8_resource_get_private_data(core::ptr::addr_of_mut!((*volume).resource), guid, data, data_size);
}

unsafe extern "system" fn d3d8_volume_FreePrivateData(
    iface: *mut IDirect3DVolume8,
    guid: *const GUID,
) -> HRESULT {
    let mut volume: *mut d3d8_volume = impl_from_IDirect3DVolume8(iface);
    TRACE!(
        "iface %p, guid %s.\n",
        iface,
        debugstr_guid(guid)
    );
    return d3d8_resource_free_private_data(core::ptr::addr_of_mut!((*volume).resource), guid);
}

unsafe extern "system" fn d3d8_volume_GetContainer(iface: *mut IDirect3DVolume8, riid: *const IID, container: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut volume: *mut d3d8_volume = impl_from_IDirect3DVolume8(iface);
    WINE_TRACE!("iface %p, riid %s, container %p.\n", iface, debugstr_guid(riid as *const _GUID), container);
    return IDirect3DBaseTexture8_QueryInterface(
        core::ptr::addr_of_mut!((*(*volume).texture).IDirect3DBaseTexture8_iface),
        riid,
        container,
    );
}

unsafe extern "system" fn d3d8_volume_GetDesc(iface: *mut IDirect3DVolume8, desc: *mut D3DVOLUME_DESC) -> HRESULT {
    let volume: *mut d3d8_volume = impl_from_IDirect3DVolume8(iface);
    let mut wined3d_desc: wined3d_sub_resource_desc = core::mem::zeroed();

    wined3d_mutex_lock();
    wined3d_texture_get_sub_resource_desc((*volume).wined3d_texture, (*volume).sub_resource_idx, core::ptr::addr_of_mut!(wined3d_desc));
    wined3d_mutex_unlock();

    (*desc).Format = d3dformat_from_wined3dformat(wined3d_desc.format);
    (*desc).Type = D3DRTYPE_VOLUME;
    (*desc).Usage = d3dusage_from_wined3dusage(wined3d_desc.usage, wined3d_desc.bind_flags);
    (*desc).Pool = d3dpool_from_wined3daccess(wined3d_desc.access, wined3d_desc.usage);
    (*desc).Size = wined3d_desc.size as UINT;
    (*desc).Width = wined3d_desc.width as UINT;
    (*desc).Height = wined3d_desc.height as UINT;
    (*desc).Depth = wined3d_desc.depth as UINT;

    return D3D_OK;
}

unsafe extern "system" fn d3d8_volume_LockBox(
    iface: *mut IDirect3DVolume8,
    locked_box: *mut D3DLOCKED_BOX,
    box_: *const D3DBOX,
    flags: DWORD,
) -> HRESULT {
    let mut volume: *mut d3d8_volume = impl_from_IDirect3DVolume8(iface);
    let mut map_desc: wined3d_map_desc = wined3d_map_desc {
        row_pitch: 0,
        slice_pitch: 0,
        data: core::ptr::null_mut::<core::ffi::c_void>(),
    };
    let mut hr: HRESULT = 0;

    WINE_TRACE!("iface %p, locked_box %p, box %p, flags %#lx.\n", iface, locked_box, box_, flags);

    hr = wined3d_resource_map(
        wined3d_texture_get_resource((*volume).wined3d_texture),
        (*volume).sub_resource_idx,
        core::ptr::addr_of_mut!(map_desc),
        box_ as *const wined3d_box,
        wined3dmapflags_from_d3dmapflags(flags as core::ffi::c_uint, 0 as core::ffi::c_uint)
            as uint32_t,
    );
    if hr < 0 {
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

unsafe extern "system" fn d3d8_volume_UnlockBox(iface: *mut IDirect3DVolume8) -> HRESULT {
    let volume: *mut d3d8_volume = impl_from_IDirect3DVolume8(iface);
    let mut hr: HRESULT;

    hr = wined3d_resource_unmap(wined3d_texture_get_resource((*volume).wined3d_texture), (*volume).sub_resource_idx);

    if hr == WINEDDERR_NOTLOCKED {
        return D3DERR_INVALIDCALL;
    }
    return hr;
}
unsafe extern "system" fn volume_wined3d_object_destroyed(mut parent: *mut ::core::ffi::c_void) {
    let mut volume: *mut d3d8_volume = parent as *mut d3d8_volume;
    d3d8_resource_cleanup(core::ptr::addr_of_mut!((*volume).resource));
    free(volume as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn volume_init(
    volume: *mut d3d8_volume,
    wined3d_texture: *mut wined3d_texture,
    sub_resource_idx: core::ffi::c_uint,
    parent_ops: *mut *const wined3d_parent_ops,
) {
    (*volume).IDirect3DVolume8_iface.lpVtbl =
        core::ptr::addr_of!(d3d8_volume_vtbl) as *const IDirect3DVolume8Vtbl;
    d3d8_resource_init(core::ptr::addr_of_mut!((*volume).resource));
    (*volume).resource.refcount = 0;
    (*volume).texture = wined3d_texture_get_parent(wined3d_texture) as *mut d3d8_texture;
    (*volume).wined3d_texture = wined3d_texture;
    (*volume).sub_resource_idx = sub_resource_idx;

    *parent_ops = core::ptr::addr_of!(d3d8_volume_wined3d_parent_ops);
}

// COM vtable tables — generated from volume.c by vtbl_gen.py
/// volume.c:174 `static const IDirect3DVolume8Vtbl d3d8_volume_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d8_volume_vtbl: IDirect3DVolume8Vtbl = IDirect3DVolume8Vtbl {
    QueryInterface: Some(d3d8_volume_QueryInterface),
    AddRef: Some(d3d8_volume_AddRef),
    Release: Some(d3d8_volume_Release),
    GetDevice: Some(d3d8_volume_GetDevice),
    SetPrivateData: Some(d3d8_volume_SetPrivateData),
    GetPrivateData: Some(d3d8_volume_GetPrivateData),
    FreePrivateData: Some(d3d8_volume_FreePrivateData),
    GetContainer: Some(d3d8_volume_GetContainer),
    GetDesc: Some(d3d8_volume_GetDesc),
    LockBox: Some(d3d8_volume_LockBox),
    UnlockBox: Some(d3d8_volume_UnlockBox),
};
