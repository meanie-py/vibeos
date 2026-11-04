use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)

static mut d3d9_vertexshader_wined3d_parent_ops: wined3d_parent_ops = unsafe {
    wined3d_parent_ops {
        wined3d_object_destroyed: Some(
            d3d9_vertexshader_wined3d_object_destroyed,
        ),
    }
};

static mut d3d9_pixelshader_wined3d_parent_ops: wined3d_parent_ops = unsafe {
    wined3d_parent_ops {
        wined3d_object_destroyed: Some(
            d3d9_pixelshader_wined3d_object_destroyed,
        ),
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(d3d9): debug channel "d3d9" (macro-generated, no direct Rust equivalent here)

unsafe extern "C" fn impl_from_IDirect3DVertexShader9(
    mut iface: *mut IDirect3DVertexShader9,
) -> *mut d3d9_vertexshader {
    return (iface as *mut u8).sub(core::mem::offset_of!(d3d9_vertexshader, IDirect3DVertexShader9_iface))
        as *mut d3d9_vertexshader;
}

unsafe extern "system" fn d3d9_vertexshader_QueryInterface(iface: *mut IDirect3DVertexShader9, riid: *const IID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DVertexShader9))
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown))
    {
        IDirect3DVertexShader9_AddRef(iface);
        *out = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    WARN!("%s not implemented, returning E_NOINTERFACE.\n", debugstr_guid(riid));

    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}
unsafe extern "system" fn d3d9_vertexshader_AddRef(mut iface: *mut IDirect3DVertexShader9) -> ULONG {
    let mut shader: *mut d3d9_vertexshader = impl_from_IDirect3DVertexShader9(iface);
    let mut refcount: ULONG = InterlockedIncrement(
        &raw mut (*shader).refcount as *mut ::core::ffi::c_long as *mut ::core::ffi::c_long,
    ) as ULONG;
    WINE_TRACE!("%p increasing refcount to %lu.\n", iface, refcount);
    if refcount == 1 {
        IDirect3DDevice9Ex_AddRef((*shader).parent_device);
        wined3d_shader_incref((*shader).wined3d_shader);
    }
    return refcount;
}

unsafe extern "system" fn d3d9_vertexshader_Release(iface: *mut IDirect3DVertexShader9) -> ULONG {
    let shader: *mut d3d9_vertexshader = impl_from_IDirect3DVertexShader9(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*shader).refcount) as *mut core::ffi::c_long,
    ) as ULONG;

    WINE_TRACE!("%p decreasing refcount to %lu.\n", iface, refcount);

    if refcount == 0 {
        let device: *mut IDirect3DDevice9Ex = (*shader).parent_device;
        wined3d_shader_decref((*shader).wined3d_shader);
        /* Release the device last, as it may cause the device to be destroyed. */
        IDirect3DDevice9Ex_Release(device);
    }

    return refcount;
}

unsafe extern "system" fn d3d9_vertexshader_GetDevice(iface: *mut IDirect3DVertexShader9, device: *mut *mut IDirect3DDevice9) -> HRESULT {
    let mut shader: *mut d3d9_vertexshader = impl_from_IDirect3DVertexShader9(iface);

    *device = (*shader).parent_device as *mut IDirect3DDevice9;
    IDirect3DDevice9_AddRef(*device);

    return D3D_OK;
}

unsafe extern "system" fn d3d9_vertexshader_GetFunction(iface: *mut IDirect3DVertexShader9, data: *mut core::ffi::c_void, data_size: *mut UINT) -> HRESULT {
    let shader: *mut d3d9_vertexshader = impl_from_IDirect3DVertexShader9(iface);
    let hr: HRESULT;

    wined3d_mutex_lock();
    hr = wined3d_shader_get_byte_code((*shader).wined3d_shader, data, data_size);
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d9_vertexshader_wined3d_object_destroyed(
    parent: *mut core::ffi::c_void,
) {
    free(parent);
}
#[no_mangle]
pub unsafe extern "C" fn vertexshader_init(
    shader: *mut d3d9_vertexshader,
    device: *mut d3d9_device,
    byte_code: *const DWORD,
) -> HRESULT {
    let mut desc: wined3d_shader_desc = wined3d_shader_desc {
        byte_code: core::ptr::null::<DWORD>(),
        byte_code_size: 0,
    };
    let mut hr: HRESULT = 0;

    (*shader).refcount = 1;
    (*shader).IDirect3DVertexShader9_iface.lpVtbl =
        core::ptr::addr_of!(d3d9_vertexshader_vtbl) as *const IDirect3DVertexShader9Vtbl;

    desc.byte_code = byte_code;
    desc.byte_code_size = !(0 as usize);

    wined3d_mutex_lock();
    hr = wined3d_shader_create_vs(
        (*device).wined3d_device,
        core::ptr::addr_of_mut!(desc),
        shader as *mut core::ffi::c_void,
        core::ptr::addr_of!(d3d9_vertexshader_wined3d_parent_ops),
        core::ptr::addr_of_mut!((*shader).wined3d_shader),
    );
    wined3d_mutex_unlock();

    if hr < 0 {
        WARN!("Failed to create wined3d vertex shader, hr %#lx.\n", hr);
        return hr;
    }

    (*shader).parent_device = core::ptr::addr_of_mut!((*device).IDirect3DDevice9Ex_iface);
    IDirect3DDevice9Ex_AddRef((*shader).parent_device);

    return D3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn unsafe_impl_from_IDirect3DVertexShader9(iface: *mut IDirect3DVertexShader9) -> *mut d3d9_vertexshader {
    if iface.is_null() {
        return core::ptr::null_mut();
    }
    if (*iface).lpVtbl != core::ptr::addr_of!(d3d9_vertexshader_vtbl) as *const _ {
        WARN!("Vertex shader %p with the wrong vtbl %p\n", iface, (*iface).lpVtbl);
    }

    return impl_from_IDirect3DVertexShader9(iface);
}

unsafe extern "C" fn impl_from_IDirect3DPixelShader9(
    iface: *mut IDirect3DPixelShader9,
) -> *mut d3d9_pixelshader {
    return CONTAINING_RECORD!(iface, d3d9_pixelshader, IDirect3DPixelShader9_iface);
}

unsafe extern "system" fn d3d9_pixelshader_QueryInterface(iface: *mut IDirect3DPixelShader9, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DPixelShader9)) != false
            || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) != false
    {
        IDirect3DPixelShader9_AddRef(iface);
        *out = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    WARN!("%s not implemented, returning E_NOINTERFACE.\n", debugstr_guid(riid));

    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn d3d9_pixelshader_AddRef(mut iface: *mut IDirect3DPixelShader9) -> ULONG {
    let mut shader: *mut d3d9_pixelshader = impl_from_IDirect3DPixelShader9(iface);
    let mut refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*shader).refcount) as *mut ::core::ffi::c_long,
    ) as ULONG;
    WINE_TRACE!("%p increasing refcount to %lu.\n", iface, refcount);
    if refcount == 1 {
        IDirect3DDevice9Ex_AddRef((*shader).parent_device);
        wined3d_shader_incref((*shader).wined3d_shader);
    }
    return refcount;
}

unsafe extern "system" fn d3d9_pixelshader_Release(iface: *mut IDirect3DPixelShader9) -> ULONG {
    let shader: *mut d3d9_pixelshader = impl_from_IDirect3DPixelShader9(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*shader).refcount) as *mut i32) as ULONG;

    TRACE!("%p decreasing refcount to %lu.\n", iface, refcount);

    if refcount == 0 {
        let device: *mut IDirect3DDevice9Ex = (*shader).parent_device;
        wined3d_shader_decref((*shader).wined3d_shader);
        /* Release the device last, as it may cause the device to be destroyed. */
        IDirect3DDevice9Ex_Release(device);
    }

    return refcount;
}

unsafe extern "system" fn d3d9_pixelshader_GetDevice(iface: *mut IDirect3DPixelShader9, device: *mut *mut IDirect3DDevice9) -> HRESULT {
    let shader: *mut d3d9_pixelshader = impl_from_IDirect3DPixelShader9(iface);

    *device = (*shader).parent_device as *mut IDirect3DDevice9;
    IDirect3DDevice9_AddRef(*device);

    return D3D_OK;
}

unsafe extern "system" fn d3d9_pixelshader_GetFunction(
    iface: *mut IDirect3DPixelShader9,
    data: *mut ::core::ffi::c_void,
    data_size: *mut UINT,
) -> HRESULT {
    let shader: *mut d3d9_pixelshader = impl_from_IDirect3DPixelShader9(iface);
    let mut hr: HRESULT = 0;

    WINE_TRACE!("iface %p, data %p, data_size %p.\n", iface, data, data_size);

    wined3d_mutex_lock();
    hr = wined3d_shader_get_byte_code((*shader).wined3d_shader, data, data_size);
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d9_pixelshader_wined3d_object_destroyed(
    parent: *mut core::ffi::c_void,
) {
    free(parent);
}
#[no_mangle]
pub unsafe extern "C" fn pixelshader_init(
    shader: *mut d3d9_pixelshader,
    device: *mut d3d9_device,
    byte_code: *const DWORD,
) -> HRESULT {
    let mut desc: wined3d_shader_desc = wined3d_shader_desc {
        byte_code: core::ptr::null::<DWORD>(),
        byte_code_size: 0,
    };
    let mut hr: HRESULT = 0;

    (*shader).refcount = 1 as LONG;
    (*shader).IDirect3DPixelShader9_iface.lpVtbl =
        core::ptr::addr_of!(d3d9_pixelshader_vtbl) as *const IDirect3DPixelShader9Vtbl;

    desc.byte_code = byte_code;
    desc.byte_code_size = !(0 as usize);

    wined3d_mutex_lock();
    hr = wined3d_shader_create_ps(
        (*device).wined3d_device,
        core::ptr::addr_of_mut!(desc),
        shader as *mut core::ffi::c_void,
        core::ptr::addr_of!(d3d9_pixelshader_wined3d_parent_ops),
        core::ptr::addr_of_mut!((*shader).wined3d_shader),
    );
    wined3d_mutex_unlock();

    if hr < 0 {
        WARN!("Failed to created wined3d pixel shader, hr %#lx.\n", hr);
        return hr;
    }

    (*shader).parent_device = core::ptr::addr_of_mut!((*device).IDirect3DDevice9Ex_iface);
    IDirect3DDevice9Ex_AddRef((*shader).parent_device);

    return D3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn unsafe_impl_from_IDirect3DPixelShader9(iface: *mut IDirect3DPixelShader9) -> *mut d3d9_pixelshader {
    if iface.is_null() {
        return core::ptr::null_mut();
    }
    if (*iface).lpVtbl != core::ptr::addr_of!(d3d9_pixelshader_vtbl) as _ {
        WARN!("Pixel shader %p with the wrong vtbl %p\n", iface, (*iface).lpVtbl);
    }

    return impl_from_IDirect3DPixelShader9(iface);
}

// COM vtable tables — generated from shader.c by vtbl_gen.py
/// shader.c:109 `static const IDirect3DVertexShader9Vtbl d3d9_vertexshader_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d9_vertexshader_vtbl: IDirect3DVertexShader9Vtbl = IDirect3DVertexShader9Vtbl {
    QueryInterface: Some(d3d9_vertexshader_QueryInterface),
    AddRef: Some(d3d9_vertexshader_AddRef),
    Release: Some(d3d9_vertexshader_Release),
    GetDevice: Some(d3d9_vertexshader_GetDevice),
    GetFunction: Some(d3d9_vertexshader_GetFunction),
};

/// shader.c:252 `static const IDirect3DPixelShader9Vtbl d3d9_pixelshader_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d9_pixelshader_vtbl: IDirect3DPixelShader9Vtbl = IDirect3DPixelShader9Vtbl {
    QueryInterface: Some(d3d9_pixelshader_QueryInterface),
    AddRef: Some(d3d9_pixelshader_AddRef),
    Release: Some(d3d9_pixelshader_Release),
    GetDevice: Some(d3d9_pixelshader_GetDevice),
    GetFunction: Some(d3d9_pixelshader_GetFunction),
};
