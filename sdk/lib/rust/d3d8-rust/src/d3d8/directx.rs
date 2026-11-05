use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----

unsafe extern "C" fn impl_from_IDirect3D8(iface: *mut IDirect3D8) -> *mut d3d8 {
    return CONTAINING_RECORD!(iface, d3d8, IDirect3D8_iface);
}

unsafe extern "system" fn d3d8_QueryInterface(iface: *mut IDirect3D8, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {

    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3D8)) != false
            || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) != false
    {
        IDirect3D8_AddRef(iface);
        *out = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    WARN!("%s not implemented, returning E_NOINTERFACE.\n", debugstr_guid(riid));

    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn d3d8_AddRef(iface: *mut IDirect3D8) -> ULONG {
    let d3d8: *mut d3d8 = impl_from_IDirect3D8(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*d3d8).refcount) as *mut LONG) as ULONG;

    return refcount;
}

unsafe extern "system" fn d3d8_Release(iface: *mut IDirect3D8) -> ULONG {
    let d3d8: *mut d3d8 = impl_from_IDirect3D8(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*d3d8).refcount) as *mut core::ffi::c_long,
    ) as ULONG;

    WINE_TRACE!("%p decreasing refcount to %lu.\n", iface, refcount);

    if refcount == 0 {
        wined3d_mutex_lock();
        wined3d_decref((*d3d8).wined3d);
        wined3d_mutex_unlock();

        free((*d3d8).wined3d_outputs as *mut core::ffi::c_void);
        free(d3d8 as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn d3d8_RegisterSoftwareDevice(iface: *mut IDirect3D8, init_function: *mut core::ffi::c_void) -> HRESULT {
    let mut d3d8: *mut d3d8 = impl_from_IDirect3D8(iface);
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, init_function %p.\n", iface, init_function);
    wined3d_mutex_lock();
    hr = wined3d_register_software_device((*d3d8).wined3d, init_function);
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d8_GetAdapterCount(iface: *mut IDirect3D8) -> UINT {
    let d3d8: *mut d3d8 = impl_from_IDirect3D8(iface);
    TRACE!("iface %p.\n", iface);
    return (*d3d8).wined3d_output_count as UINT;
}
unsafe extern "system" fn d3d8_GetAdapterIdentifier(
    mut iface: *mut IDirect3D8,
    mut adapter: UINT,
    mut flags: DWORD,
    mut identifier: *mut D3DADAPTER_IDENTIFIER8,
) -> HRESULT {
    let mut d3d8: *mut d3d8 = impl_from_IDirect3D8(iface);
    let mut adapter_id: wined3d_adapter_identifier = core::mem::zeroed();
    let mut wined3d_adapter: *mut wined3d_adapter = ::core::ptr::null_mut::<wined3d_adapter>();
    let mut output_idx: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, adapter %u, flags %#lx, identifier %p.\n", iface, adapter, flags, identifier);
    output_idx = adapter as ::core::ffi::c_uint;
    if output_idx >= (*d3d8).wined3d_output_count {
        return D3DERR_INVALIDCALL;
    }
    adapter_id.driver = &raw mut (*identifier).Driver as *mut ::core::ffi::c_char;
    adapter_id.driver_size =
        ::core::mem::size_of::<[::core::ffi::c_char; 512]>() as ::core::ffi::c_uint;
    adapter_id.description = &raw mut (*identifier).Description as *mut ::core::ffi::c_char;
    adapter_id.description_size =
        ::core::mem::size_of::<[::core::ffi::c_char; 512]>() as ::core::ffi::c_uint;
    flags ^= D3DENUM_NO_WHQL_LEVEL;
    wined3d_adapter =
        wined3d_output_get_adapter(*(*d3d8).wined3d_outputs.offset(output_idx as isize));
    hr = wined3d_adapter_get_identifier(wined3d_adapter, flags as uint32_t, &raw mut adapter_id);
    if hr >= 0 {
        (*identifier).DriverVersion = adapter_id.driver_version;
        (*identifier).VendorId = adapter_id.vendor_id;
        (*identifier).DeviceId = adapter_id.device_id;
        (*identifier).SubSysId = adapter_id.subsystem_id;
        (*identifier).Revision = adapter_id.revision;
        memcpy(
            &raw mut (*identifier).DeviceIdentifier as *mut ::core::ffi::c_void,
            &raw mut adapter_id.device_identifier as *const ::core::ffi::c_void,
            ::core::mem::size_of::<GUID>() as size_t,
        );
        (*identifier).WHQLLevel = adapter_id.whql_level;
    }
    return hr;
}

unsafe extern "system" fn d3d8_GetAdapterModeCount(iface: *mut IDirect3D8, adapter: UINT) -> UINT {
    let mut d3d8: *mut d3d8 = impl_from_IDirect3D8(iface);
    let mut output_idx: ::core::ffi::c_uint = 0;
    let mut count: ::core::ffi::c_uint = 0;
    WINE_TRACE!("iface %p, adapter %u.\n", iface, adapter);
    output_idx = adapter as ::core::ffi::c_uint;
    if output_idx >= (*d3d8).wined3d_output_count {
        return 0;
    }
    wined3d_mutex_lock();
    count = wined3d_output_get_mode_count(
        *(*d3d8).wined3d_outputs.offset(output_idx as isize),
        WINED3DFMT_UNKNOWN,
        WINED3D_SCANLINE_ORDERING_UNKNOWN,
        true,
    );
    wined3d_mutex_unlock();

    return count as UINT;
}
unsafe extern "system" fn d3d8_EnumAdapterModes(iface: *mut IDirect3D8, adapter: UINT, mode_idx: UINT, mode: *mut D3DDISPLAYMODE) -> HRESULT {
    let mut d3d8: *mut d3d8 = impl_from_IDirect3D8(iface);
    let mut wined3d_mode: wined3d_display_mode = wined3d_display_mode {
        width: 0,
        height: 0,
        refresh_rate: 0,
        format_id: WINED3DFMT_UNKNOWN,
        scanline_ordering: WINED3D_SCANLINE_ORDERING_UNKNOWN,
    };
    let mut output_idx: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;

    TRACE!("iface %p, adapter %u, mode_idx %u, mode %p.\n", iface, adapter, mode_idx, mode);

    output_idx = adapter as ::core::ffi::c_uint;
    if output_idx >= (*d3d8).wined3d_output_count {
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    hr = wined3d_output_get_mode(
        *(*d3d8).wined3d_outputs.offset(output_idx as isize),
        WINED3DFMT_UNKNOWN,
        WINED3D_SCANLINE_ORDERING_UNKNOWN,
        mode_idx as ::core::ffi::c_uint,
        core::ptr::addr_of_mut!(wined3d_mode),
        true,
    );
    wined3d_mutex_unlock();

    if hr >= 0 {
        (*mode).Width = wined3d_mode.width;
        (*mode).Height = wined3d_mode.height;
        (*mode).RefreshRate = wined3d_mode.refresh_rate;
        (*mode).Format = d3dformat_from_wined3dformat(wined3d_mode.format_id);
    }

    return hr;
}
unsafe extern "system" fn d3d8_GetAdapterDisplayMode(iface: *mut IDirect3D8, adapter: UINT, mode: *mut D3DDISPLAYMODE) -> HRESULT {
    let d3d8: *mut d3d8 = impl_from_IDirect3D8(iface);
    let mut wined3d_mode: wined3d_display_mode = core::mem::zeroed();
    let mut output_idx: ::core::ffi::c_uint;
    let mut hr: HRESULT;

    output_idx = adapter as ::core::ffi::c_uint;
    if output_idx >= (*d3d8).wined3d_output_count {
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    hr = wined3d_output_get_display_mode(
        *(*d3d8).wined3d_outputs.offset(output_idx as isize),
        &raw mut wined3d_mode,
        core::ptr::null_mut(),
    );
    wined3d_mutex_unlock();

    if hr >= 0 {
        (*mode).Width = wined3d_mode.width;
        (*mode).Height = wined3d_mode.height;
        (*mode).RefreshRate = wined3d_mode.refresh_rate;
        (*mode).Format = d3dformat_from_wined3dformat(wined3d_mode.format_id);
    }

    return hr;
}

unsafe extern "system" fn d3d8_CheckDeviceType(iface: *mut IDirect3D8, adapter: UINT, device_type: D3DDEVTYPE, display_format: D3DFORMAT, backbuffer_format: D3DFORMAT, windowed: BOOL) -> HRESULT {
    let mut d3d8: *mut d3d8 = impl_from_IDirect3D8(iface);
    let mut output_idx: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, adapter %u, device_type %#x, display_format %#x, backbuffer_format %#x, windowed %#x.\n", iface, adapter, device_type as ::core::ffi::c_uint, display_format as ::core::ffi::c_uint, backbuffer_format as ::core::ffi::c_uint, windowed);
    output_idx = adapter as ::core::ffi::c_uint;
    if output_idx >= (*d3d8).wined3d_output_count {
        return D3DERR_INVALIDCALL;
    }
    if windowed == 0
        && display_format as ::core::ffi::c_uint != D3DFMT_X8R8G8B8 as ::core::ffi::c_uint
        && display_format as ::core::ffi::c_uint != D3DFMT_R5G6B5 as ::core::ffi::c_uint
    {
        return WINED3DERR_NOTAVAILABLE;
    }
    wined3d_mutex_lock();
    hr = wined3d_check_device_type(
        (*d3d8).wined3d,
        *(*d3d8).wined3d_outputs.offset(output_idx as isize),
        wined3d_device_type_from_d3d(device_type),
        wined3dformat_from_d3dformat(display_format),
        wined3dformat_from_d3dformat(backbuffer_format),
        windowed,
    );
    wined3d_mutex_unlock();
    return hr;
}

unsafe extern "system" fn d3d8_CheckDeviceFormat(
    iface: *mut IDirect3D8,
    adapter: UINT,
    device_type: D3DDEVTYPE,
    adapter_format: D3DFORMAT,
    usage: DWORD,
    resource_type: D3DRESOURCETYPE,
    format: D3DFORMAT,
) -> HRESULT {
    let mut d3d8: *mut d3d8 = impl_from_IDirect3D8(iface);
    let mut wined3d_rtype: wined3d_resource_type = core::mem::zeroed();
    let mut wined3d_adapter: *mut wined3d_adapter = core::ptr::null_mut();
    let mut bind_flags: ::core::ffi::c_uint = 0;
    let mut output_idx: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;
    let mut usage = usage;

    output_idx = adapter;
    if output_idx >= (*d3d8).wined3d_output_count {
        return D3DERR_INVALIDCALL;
    }

    if adapter_format != D3DFMT_X8R8G8B8 && adapter_format != D3DFMT_R5G6B5
        && adapter_format != D3DFMT_X1R5G5B5
    {
        WARN!("Invalid adapter format.\n");
        return if adapter_format != 0 { D3DERR_NOTAVAILABLE } else { D3DERR_INVALIDCALL };
    }

    bind_flags = wined3d_bind_flags_from_d3d8_usage(usage);
    usage = usage & (WINED3DUSAGE_MASK | WINED3DUSAGE_QUERY_MASK);
    match resource_type {
        D3DRTYPE_CUBETEXTURE => {
            usage |= WINED3DUSAGE_LEGACY_CUBEMAP;
            bind_flags |= WINED3D_BIND_SHADER_RESOURCE;
            wined3d_rtype = WINED3D_RTYPE_TEXTURE_2D;
        }
        D3DRTYPE_TEXTURE => {
            bind_flags |= WINED3D_BIND_SHADER_RESOURCE;
            wined3d_rtype = WINED3D_RTYPE_TEXTURE_2D;
        }
        D3DRTYPE_SURFACE => {
            wined3d_rtype = WINED3D_RTYPE_TEXTURE_2D;
        }
        D3DRTYPE_VOLUMETEXTURE | D3DRTYPE_VOLUME => {
            bind_flags |= WINED3D_BIND_SHADER_RESOURCE;
            wined3d_rtype = WINED3D_RTYPE_TEXTURE_3D;
        }
        D3DRTYPE_VERTEXBUFFER | D3DRTYPE_INDEXBUFFER => {
            wined3d_rtype = WINED3D_RTYPE_BUFFER;
        }
        _ => {
            FIXME!("Unhandled resource type %#x.\n", resource_type);
            return WINED3DERR_INVALIDCALL;
        }
    }

    wined3d_mutex_lock();
    wined3d_adapter = wined3d_output_get_adapter(*(*d3d8).wined3d_outputs.offset(output_idx as isize));
    if format == D3DFMT_RESZ && resource_type == D3DRTYPE_SURFACE && usage == D3DUSAGE_RENDERTARGET
    {
        let mut levels: ::core::ffi::c_uint = 0;

        hr = wined3d_check_device_multisample_type(wined3d_adapter, wined3d_device_type_from_d3d(device_type),
                WINED3DFMT_D24_UNORM_S8_UINT, (FALSE) as BOOL, WINED3D_MULTISAMPLE_NON_MASKABLE,
                core::ptr::addr_of_mut!(levels));
        if (hr >= 0) && levels == 0 {
            hr = D3DERR_NOTAVAILABLE;
        }
    } else {
        hr = wined3d_check_device_format((*d3d8).wined3d, wined3d_adapter, wined3d_device_type_from_d3d(device_type),
                wined3dformat_from_d3dformat(adapter_format), usage, bind_flags, wined3d_rtype,
                wined3dformat_from_d3dformat(format));
    }
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d8_CheckDeviceMultiSampleType(
    iface: *mut IDirect3D8,
    adapter: UINT,
    device_type: D3DDEVTYPE,
    format: D3DFORMAT,
    windowed: BOOL,
    multisample_type: D3DMULTISAMPLE_TYPE,
) -> HRESULT {
    let d3d8: *mut d3d8 = impl_from_IDirect3D8(iface);
    let mut wined3d_adapter: *mut wined3d_adapter = core::ptr::null_mut();
    let mut output_idx: core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;

    WINE_TRACE!("iface %p, adapter %u, device_type %#x, format %#x, windowed %#x, multisample_type %#x.\n", iface, adapter, device_type as core::ffi::c_uint, format as core::ffi::c_uint, windowed, multisample_type as core::ffi::c_uint);

    output_idx = adapter as core::ffi::c_uint;
    if output_idx >= (*d3d8).wined3d_output_count {
        return D3DERR_INVALIDCALL;
    }

    if multisample_type as core::ffi::c_uint > D3DMULTISAMPLE_16_SAMPLES {
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    wined3d_adapter = wined3d_output_get_adapter(*(*d3d8).wined3d_outputs.offset(output_idx as isize));
    hr = wined3d_check_device_multisample_type(
        wined3d_adapter,
        wined3d_device_type_from_d3d(device_type),
        wined3dformat_from_d3dformat(format),
        windowed,
        multisample_type as wined3d_multisample_type,
        core::ptr::null_mut::<core::ffi::c_uint>(),
    );
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d8_CheckDepthStencilMatch(
    mut iface: *mut IDirect3D8,
    mut adapter: UINT,
    mut device_type: D3DDEVTYPE,
    mut adapter_format: D3DFORMAT,
    mut rt_format: D3DFORMAT,
    mut ds_format: D3DFORMAT,
) -> HRESULT {
    let mut d3d8: *mut d3d8 = impl_from_IDirect3D8(iface);
    let mut wined3d_adapter: *mut wined3d_adapter = ::core::ptr::null_mut::<wined3d_adapter>();
    let mut output_idx: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, adapter %u, device_type %#x, adapter_format %#x, rt_format %#x, ds_format %#x.\n", iface, adapter, device_type as ::core::ffi::c_uint, adapter_format as ::core::ffi::c_uint, rt_format as ::core::ffi::c_uint, ds_format as ::core::ffi::c_uint);
    output_idx = adapter as ::core::ffi::c_uint;
    if output_idx >= (*d3d8).wined3d_output_count {
        return D3DERR_INVALIDCALL;
    }
    wined3d_mutex_lock();
    wined3d_adapter =
        wined3d_output_get_adapter(*(*d3d8).wined3d_outputs.offset(output_idx as isize));
    hr = wined3d_check_depth_stencil_match(
        wined3d_adapter,
        wined3d_device_type_from_d3d(device_type),
        wined3dformat_from_d3dformat(adapter_format),
        wined3dformat_from_d3dformat(rt_format),
        wined3dformat_from_d3dformat(ds_format),
    );
    wined3d_mutex_unlock();
    return hr;
}

unsafe extern "system" fn d3d8_GetDeviceCaps(
    mut iface: *mut IDirect3D8,
    mut adapter: UINT,
    mut device_type: D3DDEVTYPE,
    mut caps: *mut D3DCAPS8,
) -> HRESULT {
    let mut d3d8: *mut d3d8 = impl_from_IDirect3D8(iface);
    let mut wined3d_adapter: *mut wined3d_adapter = core::ptr::null_mut::<wined3d_adapter>();
    let mut wined3d_caps: wined3d_caps = core::mem::zeroed::<wined3d_caps>();
    let mut output_idx: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, adapter %u, device_type %#x, caps %p.\n", iface, adapter, device_type as ::core::ffi::c_uint, caps);
    output_idx = adapter;
    if output_idx >= (*d3d8).wined3d_output_count {
        return D3DERR_INVALIDCALL;
    }
    if caps.is_null() {
        return D3DERR_INVALIDCALL;
    }
    wined3d_mutex_lock();
    wined3d_adapter =
        wined3d_output_get_adapter(*(*d3d8).wined3d_outputs.offset(output_idx as isize));
    hr = wined3d_get_device_caps(
        wined3d_adapter,
        wined3d_device_type_from_d3d(device_type),
        &raw mut wined3d_caps,
    );
    wined3d_mutex_unlock();
    d3dcaps_from_wined3dcaps(caps, &raw mut wined3d_caps, adapter);
    return hr;
}

unsafe extern "system" fn d3d8_GetAdapterMonitor(iface: *mut IDirect3D8, adapter: UINT) -> HMONITOR {
    let d3d8: *mut d3d8 = impl_from_IDirect3D8(iface);
    let mut desc: wined3d_output_desc = core::mem::zeroed();
    let mut output_idx: UINT;
    let mut hr: HRESULT = 0;

    output_idx = adapter;
    if output_idx >= (*d3d8).wined3d_output_count {
        return core::ptr::null_mut();
    }

    wined3d_mutex_lock();
    hr = wined3d_output_get_desc(
        *(*d3d8).wined3d_outputs.add(output_idx as usize),
        core::ptr::addr_of_mut!(desc),
    );
    wined3d_mutex_unlock();

    if hr < 0 {
        WARN!("Failed to get output desc, hr %#lx.\n", hr);
        return core::ptr::null_mut();
    }

    desc.monitor
}

unsafe extern "system" fn d3d8_CreateDevice(
    iface: *mut IDirect3D8,
    adapter: UINT,
    device_type: D3DDEVTYPE,
    focus_window: HWND,
    flags: DWORD,
    parameters: *mut D3DPRESENT_PARAMETERS,
    device: *mut *mut IDirect3DDevice8,
) -> HRESULT {
    let d3d8: *mut d3d8 = impl_from_IDirect3D8(iface);
    let mut object: *mut d3d8_device;
    let mut hr: HRESULT;

    object = calloc(1, (core::mem::size_of::<d3d8_device>()) as _) as *mut d3d8_device;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = device_init(object, d3d8, (*d3d8).wined3d, adapter, device_type, focus_window, flags, parameters);
    if hr < 0 {
        WARN!("Failed to initialize device, hr %#lx.\n", hr);
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    *device = core::ptr::addr_of_mut!((*object).IDirect3DDevice8_iface);

    return D3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn d3d8_init(d3d8: *mut d3d8) -> BOOL {
    let flags: DWORD = (WINED3D_LEGACY_DEPTH_BIAS | WINED3D_HANDLE_RESTORE
            | WINED3D_PIXEL_CENTER_INTEGER | WINED3D_LEGACY_UNBOUND_RESOURCE_COLOR
            | WINED3D_NO_PRIMITIVE_RESTART | WINED3D_LEGACY_CUBEMAP_FILTERING
            | WINED3D_NO_DRAW_INDIRECT) as DWORD;
    let mut adapter_idx: ::core::ffi::c_uint;
    let mut output_idx: ::core::ffi::c_uint;
    let mut adapter_count: ::core::ffi::c_uint;
    let mut output_count: ::core::ffi::c_uint = 0;
    let mut wined3d_adapter: *mut wined3d_adapter;

    (*d3d8).IDirect3D8_iface.lpVtbl = core::ptr::addr_of!(d3d8_vtbl) as *const IDirect3D8Vtbl;
    (*d3d8).refcount = 1 as LONG;

    wined3d_mutex_lock();
    (*d3d8).wined3d = wined3d_create(flags as uint32_t);
    if (*d3d8).wined3d.is_null() {
        wined3d_mutex_unlock();
        return (FALSE) as BOOL;
    }

    adapter_count = wined3d_get_adapter_count((*d3d8).wined3d) as ::core::ffi::c_uint;
    adapter_idx = 0;
    while adapter_idx < adapter_count {
        wined3d_adapter = wined3d_get_adapter((*d3d8).wined3d, adapter_idx);
        output_count = output_count.wrapping_add(wined3d_adapter_get_output_count(wined3d_adapter));
        adapter_idx = adapter_idx.wrapping_add(1);
    }

    (*d3d8).wined3d_outputs = calloc(
        (output_count as size_t) as _,
        (::core::mem::size_of::<*mut wined3d_output>() as size_t) as _,
    ) as *mut *mut wined3d_output;
    if (*d3d8).wined3d_outputs.is_null() {
        wined3d_decref((*d3d8).wined3d);
        wined3d_mutex_unlock();
        return (FALSE) as BOOL;
    }

    (*d3d8).wined3d_output_count = 0;
    adapter_idx = 0;
    while adapter_idx < adapter_count {
        wined3d_adapter = wined3d_get_adapter((*d3d8).wined3d, adapter_idx);
        output_count = wined3d_adapter_get_output_count(wined3d_adapter);
        output_idx = 0;
        while output_idx < output_count {
            *(*d3d8)
                .wined3d_outputs
                .offset((*d3d8).wined3d_output_count as isize)
                = wined3d_adapter_get_output(wined3d_adapter, output_idx);
            (*d3d8).wined3d_output_count = (*d3d8).wined3d_output_count.wrapping_add(1);
            output_idx = output_idx.wrapping_add(1);
        }
        adapter_idx = adapter_idx.wrapping_add(1);
    }

    wined3d_mutex_unlock();
    return (TRUE) as BOOL;
}

// COM vtable tables — generated from directx.c by vtbl_gen.py
/// directx.c:452 `static const IDirect3D8Vtbl d3d8_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d8_vtbl: IDirect3D8Vtbl = IDirect3D8Vtbl {
    QueryInterface: Some(d3d8_QueryInterface),
    AddRef: Some(d3d8_AddRef),
    Release: Some(d3d8_Release),
    RegisterSoftwareDevice: Some(d3d8_RegisterSoftwareDevice),
    GetAdapterCount: Some(d3d8_GetAdapterCount),
    GetAdapterIdentifier: Some(d3d8_GetAdapterIdentifier),
    GetAdapterModeCount: Some(d3d8_GetAdapterModeCount),
    EnumAdapterModes: Some(d3d8_EnumAdapterModes),
    GetAdapterDisplayMode: Some(d3d8_GetAdapterDisplayMode),
    CheckDeviceType: Some(d3d8_CheckDeviceType),
    CheckDeviceFormat: Some(d3d8_CheckDeviceFormat),
    CheckDeviceMultiSampleType: Some(d3d8_CheckDeviceMultiSampleType),
    CheckDepthStencilMatch: Some(d3d8_CheckDepthStencilMatch),
    GetDeviceCaps: Some(d3d8_GetDeviceCaps),
    GetAdapterMonitor: Some(d3d8_GetAdapterMonitor),
    CreateDevice: Some(d3d8_CreateDevice),
};
