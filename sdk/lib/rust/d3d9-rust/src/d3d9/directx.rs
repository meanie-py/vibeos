use crate::d3d9::d3d9_main::Direct3DCreate9Ex;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(d3d9): debug-channel macro, expanded to its channel-name static.

// Dangling `static HRESULT WINAPI` fragments (no name, no body) have no translatable declaration; omitted.
// `static const struct IDirect3D9ExVtbl d3d9_vtbl` is a COM vtable table, skipped per vtbl_gen.py.

unsafe extern "C" fn impl_from_IDirect3D9Ex(iface: *mut IDirect3D9Ex) -> *mut d3d9 {
    return CONTAINING_RECORD!(iface, d3d9, IDirect3D9Ex_iface);
}

unsafe extern "system" fn d3d9_QueryInterface(iface: *mut IDirect3D9Ex, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let d3d9: *mut d3d9 = impl_from_IDirect3D9Ex(iface);

    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3D9))
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown))
    {
        IDirect3D9Ex_AddRef(core::ptr::addr_of_mut!((*d3d9).IDirect3D9Ex_iface));
        *out = core::ptr::addr_of_mut!((*d3d9).IDirect3D9Ex_iface) as *mut core::ffi::c_void;
        return S_OK;
    }

    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3D9Ex)) {
        if (*d3d9).extended == 0 {
            WARN!("Application asks for IDirect3D9Ex, but this instance wasn't created with Direct3DCreate9Ex.\n");
            *out = core::ptr::null_mut();
            return E_NOINTERFACE;
        }

        IDirect3D9Ex_AddRef(core::ptr::addr_of_mut!((*d3d9).IDirect3D9Ex_iface));
        *out = core::ptr::addr_of_mut!((*d3d9).IDirect3D9Ex_iface) as *mut core::ffi::c_void;
        return S_OK;
    }

    WARN!("%s not implemented, returning E_NOINTERFACE.\n", debugstr_guid(riid));

    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}
unsafe extern "system" fn d3d9_AddRef(mut iface: *mut IDirect3D9Ex) -> ULONG {
    let mut d3d9: *mut d3d9 = impl_from_IDirect3D9Ex(iface);
    let mut refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*d3d9).refcount) as *mut ::core::ffi::c_long,
    ) as ULONG;
    WINE_TRACE!("%p increasing refcount to %lu.\n", iface, refcount);
    return refcount;
}

unsafe extern "system" fn d3d9_Release(iface: *mut IDirect3D9Ex) -> ULONG {
    let mut d3d9: *mut d3d9 = impl_from_IDirect3D9Ex(iface);
    let mut refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*d3d9).refcount) as *mut LONG,
    ) as ULONG;
    WINE_TRACE!("%p decreasing refcount to %lu.\n", iface, refcount);
    if refcount == 0 {
        wined3d_decref((*d3d9).wined3d);

        free((*d3d9).wined3d_outputs as *mut core::ffi::c_void);
        free(d3d9 as *mut core::ffi::c_void);
    }
    return refcount;
}

unsafe extern "system" fn d3d9_RegisterSoftwareDevice(iface: *mut IDirect3D9Ex, init_function: *mut core::ffi::c_void) -> HRESULT {
    let mut d3d9: *mut d3d9 = impl_from_IDirect3D9Ex(iface);
    let mut hr: HRESULT;

    wined3d_mutex_lock();
    hr = wined3d_register_software_device((*d3d9).wined3d, init_function);
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d9_GetAdapterCount(mut iface: *mut IDirect3D9Ex) -> UINT {
    let mut d3d9: *mut d3d9 = impl_from_IDirect3D9Ex(iface);
    WINE_TRACE!("iface %p.\n", iface);
    return (*d3d9).wined3d_output_count as UINT;
}
unsafe extern "system" fn d3d9_GetAdapterIdentifier(
    iface: *mut IDirect3D9Ex,
    adapter: UINT,
    flags: DWORD,
    identifier: *mut D3DADAPTER_IDENTIFIER9,
) -> HRESULT {
    let d3d9: *mut d3d9 = impl_from_IDirect3D9Ex(iface);
    let mut adapter_id: wined3d_adapter_identifier = core::mem::zeroed();
    let mut wined3d_adapter: *mut wined3d_adapter = core::ptr::null_mut::<wined3d_adapter>();
    let mut output_desc: wined3d_output_desc = core::mem::zeroed();
    let mut output_idx: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;

    TRACE!(
        "iface %p, adapter %u, flags %#lx, identifier %p.\n",
        iface,
        adapter,
        flags,
        identifier
    );

    output_idx = adapter as ::core::ffi::c_uint;
    if output_idx >= (*d3d9).wined3d_output_count {
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    hr = wined3d_output_get_desc(
        *(*d3d9).wined3d_outputs.offset(output_idx as isize),
        &raw mut output_desc,
    );
    if hr < 0 {
        wined3d_mutex_unlock();
        WARN!("Failed to get output description, hr %#lx.\n", hr);
        return hr;
    }
    WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        &raw mut output_desc.device_name as *mut WCHAR as LPCWSTR,
        -(1 as ::core::ffi::c_int),
        &raw mut (*identifier).DeviceName as LPSTR,
        core::mem::size_of::<[::core::ffi::c_char; 32]>() as ::core::ffi::c_int,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    );

    adapter_id.driver = &raw mut (*identifier).Driver as *mut ::core::ffi::c_char;
    adapter_id.driver_size =
        core::mem::size_of::<[::core::ffi::c_char; 512]>() as ::core::ffi::c_uint;
    adapter_id.description = &raw mut (*identifier).Description as *mut ::core::ffi::c_char;
    adapter_id.description_size =
        core::mem::size_of::<[::core::ffi::c_char; 512]>() as ::core::ffi::c_uint;

    wined3d_adapter =
        wined3d_output_get_adapter(*(*d3d9).wined3d_outputs.offset(output_idx as isize));
    hr = wined3d_adapter_get_identifier(wined3d_adapter, flags, &raw mut adapter_id);
    if hr >= 0 {
        (*identifier).DriverVersion = adapter_id.driver_version;
        (*identifier).VendorId = adapter_id.vendor_id;
        (*identifier).DeviceId = adapter_id.device_id;
        (*identifier).SubSysId = adapter_id.subsystem_id;
        (*identifier).Revision = adapter_id.revision;
        memcpy(
            &raw mut (*identifier).DeviceIdentifier as *mut ::core::ffi::c_void,
            &raw mut adapter_id.device_identifier as *const ::core::ffi::c_void,
            core::mem::size_of::<GUID>() as size_t,
        );
        (*identifier).WHQLLevel = adapter_id.whql_level;
    }
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d9_GetAdapterModeCount(iface: *mut IDirect3D9Ex, adapter: UINT, format: D3DFORMAT) -> UINT {
    let d3d9: *mut d3d9 = impl_from_IDirect3D9Ex(iface);
    let mut output_idx: UINT = 0;
    let mut count: UINT = 0;
    output_idx = adapter;
    if output_idx >= (*d3d9).wined3d_output_count {
        return 0;
    }
    /* Others than that not supported by d3d9, but reported by wined3d for ddraw. Filter them out. */
    if format != D3DFMT_X8R8G8B8 && format != D3DFMT_R5G6B5 {
        return 0;
    }
    wined3d_mutex_lock();
    count = wined3d_output_get_mode_count(
        *(*d3d9).wined3d_outputs.offset(output_idx as isize),
        wined3dformat_from_d3dformat(format),
        WINED3D_SCANLINE_ORDERING_UNKNOWN,
        true,
    );
    wined3d_mutex_unlock();
    return count;
}
unsafe extern "system" fn d3d9_EnumAdapterModes(
    mut iface: *mut IDirect3D9Ex,
    mut adapter: UINT,
    mut format: D3DFORMAT,
    mut mode_idx: UINT,
    mut mode: *mut D3DDISPLAYMODE,
) -> HRESULT {
    let mut d3d9: *mut d3d9 = impl_from_IDirect3D9Ex(iface);
    let mut wined3d_mode: wined3d_display_mode = core::mem::zeroed();
    let mut output_idx: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, adapter %u, format %#x, mode_idx %u, mode %p.\n", iface, adapter, format as ::core::ffi::c_uint, mode_idx, mode);

    output_idx = adapter;
    if output_idx >= (*d3d9).wined3d_output_count {
        return D3DERR_INVALIDCALL;
    }

    if format != D3DFMT_X8R8G8B8 && format != D3DFMT_R5G6B5 {
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    hr = wined3d_output_get_mode(
        *(*d3d9).wined3d_outputs.offset(output_idx as isize),
        wined3dformat_from_d3dformat(format),
        WINED3D_SCANLINE_ORDERING_UNKNOWN,
        mode_idx,
        &raw mut wined3d_mode,
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
unsafe extern "system" fn d3d9_GetAdapterDisplayMode(iface: *mut IDirect3D9Ex, adapter: UINT, mode: *mut D3DDISPLAYMODE) -> HRESULT {
    let mut d3d9: *mut d3d9 = impl_from_IDirect3D9Ex(iface);
    let mut wined3d_mode: wined3d_display_mode = wined3d_display_mode {
        width: 0,
        height: 0,
        refresh_rate: 0,
        format_id: WINED3DFMT_UNKNOWN,
        scanline_ordering: WINED3D_SCANLINE_ORDERING_UNKNOWN,
    };
    let mut output_idx: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, adapter %u, mode %p.\n", iface, adapter, mode);
    output_idx = adapter as ::core::ffi::c_uint;
    if output_idx >= (*d3d9).wined3d_output_count {
        return D3DERR_INVALIDCALL;
    }
    wined3d_mutex_lock();
    hr = wined3d_output_get_display_mode(
        *(*d3d9).wined3d_outputs.offset(output_idx as isize),
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
unsafe extern "system" fn d3d9_CheckDeviceType(
    mut iface: *mut IDirect3D9Ex,
    mut adapter: UINT,
    mut device_type: D3DDEVTYPE,
    mut display_format: D3DFORMAT,
    mut backbuffer_format: D3DFORMAT,
    mut windowed: BOOL,
) -> HRESULT {
    let mut d3d9: *mut d3d9 = impl_from_IDirect3D9Ex(iface);
    let mut output_idx: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, adapter %u, device_type %#x, display_format %#x, backbuffer_format %#x, windowed %#x.\n", iface, adapter, device_type as ::core::ffi::c_uint, display_format as ::core::ffi::c_uint, backbuffer_format as ::core::ffi::c_uint, windowed);
    output_idx = adapter as ::core::ffi::c_uint;
    if output_idx >= (*d3d9).wined3d_output_count {
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
        (*d3d9).wined3d,
        *(*d3d9).wined3d_outputs.offset(output_idx as isize),
        wined3d_device_type_from_d3d(device_type),
        wined3dformat_from_d3dformat(display_format),
        wined3dformat_from_d3dformat(backbuffer_format),
        windowed,
    );
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d9_CheckDeviceFormat(mut iface: *mut IDirect3D9Ex, mut adapter: UINT, mut device_type: D3DDEVTYPE, mut adapter_format: D3DFORMAT, mut usage: DWORD, mut resource_type: D3DRESOURCETYPE, mut format: D3DFORMAT) -> HRESULT {
    let mut d3d9: *mut d3d9 = impl_from_IDirect3D9Ex(iface);
    let mut wined3d_rtype: wined3d_resource_type = WINED3D_RTYPE_NONE;
    let mut wined3d_adapter: *mut wined3d_adapter = ::core::ptr::null_mut::<wined3d_adapter>();
    let mut bind_flags: ::core::ffi::c_uint = 0;
    let mut output_idx: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;

    TRACE!("iface %p, adapter %u, device_type %#x, adapter_format %#x, usage %#lx, resource_type %#x, format %#x.\n", iface, adapter, device_type as ::core::ffi::c_uint, adapter_format as ::core::ffi::c_uint, usage, resource_type as ::core::ffi::c_uint, format as ::core::ffi::c_uint);

    output_idx = adapter as ::core::ffi::c_uint;
    if output_idx >= (*d3d9).wined3d_output_count {
        return D3DERR_INVALIDCALL;
    }

    if adapter_format as ::core::ffi::c_uint != D3DFMT_X8R8G8B8 as ::core::ffi::c_uint
        && adapter_format as ::core::ffi::c_uint != D3DFMT_R5G6B5 as ::core::ffi::c_uint
        && adapter_format as ::core::ffi::c_uint != D3DFMT_X1R5G5B5 as ::core::ffi::c_uint
    {
        WARN!("Invalid adapter format.\n");
        return if adapter_format as ::core::ffi::c_uint != 0 {
            D3DERR_NOTAVAILABLE
        } else {
            D3DERR_INVALIDCALL
        };
    }

    bind_flags = wined3d_bind_flags_from_d3d9_usage(usage);
    usage = usage & (WINED3DUSAGE_MASK | WINED3DUSAGE_QUERY_MASK) as DWORD;
    match resource_type as ::core::ffi::c_uint {
        D3DRTYPE_CUBETEXTURE => {
            usage |= WINED3DUSAGE_LEGACY_CUBEMAP as DWORD;
            bind_flags |= WINED3D_BIND_SHADER_RESOURCE as ::core::ffi::c_uint;
            wined3d_rtype = WINED3D_RTYPE_TEXTURE_2D;
        }
        D3DRTYPE_TEXTURE => {
            bind_flags |= WINED3D_BIND_SHADER_RESOURCE as ::core::ffi::c_uint;
            wined3d_rtype = WINED3D_RTYPE_TEXTURE_2D;
        }
        D3DRTYPE_SURFACE => {
            wined3d_rtype = WINED3D_RTYPE_TEXTURE_2D;
        }
        D3DRTYPE_VOLUMETEXTURE | D3DRTYPE_VOLUME => {
            bind_flags |= WINED3D_BIND_SHADER_RESOURCE as ::core::ffi::c_uint;
            wined3d_rtype = WINED3D_RTYPE_TEXTURE_3D;
        }
        D3DRTYPE_VERTEXBUFFER | D3DRTYPE_INDEXBUFFER => {
            wined3d_rtype = WINED3D_RTYPE_BUFFER;
        }
        _ => {
            FIXME!("Unhandled resource type %#x.\n", resource_type as ::core::ffi::c_uint);
            return WINED3DERR_INVALIDCALL;
        }
    }

    wined3d_mutex_lock();
    wined3d_adapter = wined3d_output_get_adapter(*(*d3d9).wined3d_outputs.offset(output_idx as isize));
    if format as ::core::ffi::c_uint == D3DFMT_RESZ as ::core::ffi::c_uint
        && resource_type as ::core::ffi::c_uint == D3DRTYPE_SURFACE as ::core::ffi::c_uint
        && usage as ::core::ffi::c_ulong == D3DUSAGE_RENDERTARGET as ::core::ffi::c_ulong
    {
        let mut levels: ::core::ffi::c_uint = 0;

        hr = wined3d_check_device_multisample_type(
            wined3d_adapter,
            wined3d_device_type_from_d3d(device_type),
            WINED3DFMT_D24_UNORM_S8_UINT,
            (FALSE) as BOOL,
            WINED3D_MULTISAMPLE_NON_MASKABLE,
            ::core::ptr::addr_of_mut!(levels),
        );
        if hr >= 0 && levels == 0 {
            hr = D3DERR_NOTAVAILABLE;
        }
    } else {
        hr = wined3d_check_device_format(
            (*d3d9).wined3d,
            wined3d_adapter,
            wined3d_device_type_from_d3d(device_type),
            wined3dformat_from_d3dformat(adapter_format),
            usage as uint32_t,
            bind_flags,
            wined3d_rtype,
            wined3dformat_from_d3dformat(format),
        );
    }
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d9_CheckDeviceMultiSampleType(
    mut iface: *mut IDirect3D9Ex,
    mut adapter: UINT,
    mut device_type: D3DDEVTYPE,
    mut format: D3DFORMAT,
    mut windowed: BOOL,
    mut multisample_type: D3DMULTISAMPLE_TYPE,
    mut levels: *mut DWORD,
) -> HRESULT {
    let mut d3d9: *mut d3d9 = impl_from_IDirect3D9Ex(iface);
    let mut wined3d_adapter: *mut wined3d_adapter = ::core::ptr::null_mut::<wined3d_adapter>();
    let mut output_idx: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, adapter %u, device_type %#x, format %#x, windowed %#x, multisample_type %#x, levels %p.\n", iface, adapter, device_type as ::core::ffi::c_uint, format as ::core::ffi::c_uint, windowed, multisample_type as ::core::ffi::c_uint, levels);
    output_idx = adapter as ::core::ffi::c_uint;
    if output_idx >= (*d3d9).wined3d_output_count {
        return D3DERR_INVALIDCALL;
    }
    if multisample_type as ::core::ffi::c_uint > D3DMULTISAMPLE_16_SAMPLES {
        return D3DERR_INVALIDCALL;
    }
    wined3d_mutex_lock();
    wined3d_adapter =
        wined3d_output_get_adapter(*(*d3d9).wined3d_outputs.offset(output_idx as isize));
    hr = wined3d_check_device_multisample_type(
        wined3d_adapter,
        wined3d_device_type_from_d3d(device_type),
        wined3dformat_from_d3dformat(format),
        windowed,
        wined3d_multisample_type_from_d3d(multisample_type),
        levels as *mut ::core::ffi::c_uint,
    );
    wined3d_mutex_unlock();
    if hr == WINED3DERR_NOTAVAILABLE && !levels.is_null() {
        *levels = 1;
    }
    return hr;
}

unsafe extern "system" fn d3d9_CheckDepthStencilMatch(
    iface: *mut IDirect3D9Ex,
    adapter: UINT,
    device_type: D3DDEVTYPE,
    adapter_format: D3DFORMAT,
    rt_format: D3DFORMAT,
    ds_format: D3DFORMAT,
) -> HRESULT {
    let d3d9: *mut d3d9 = impl_from_IDirect3D9Ex(iface);
    let mut wined3d_adapter: *mut wined3d_adapter = core::ptr::null_mut();
    let mut output_idx: u32;
    let mut hr: HRESULT = 0;

    TRACE!(
        "iface %p, adapter %u, device_type %#x, adapter_format %#x, rt_format %#x, ds_format %#x.\n",
        iface,
        adapter,
        device_type as u32,
        adapter_format as u32,
        rt_format as u32,
        ds_format as u32
    );

    output_idx = adapter;
    if output_idx >= (*d3d9).wined3d_output_count {
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    wined3d_adapter = wined3d_output_get_adapter(*(*d3d9).wined3d_outputs.offset(output_idx as isize));
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
unsafe extern "system" fn d3d9_CheckDeviceFormatConversion(
    mut iface: *mut IDirect3D9Ex,
    mut adapter: UINT,
    mut device_type: D3DDEVTYPE,
    mut src_format: D3DFORMAT,
    mut dst_format: D3DFORMAT,
) -> HRESULT {
    let mut d3d9: *mut d3d9 = impl_from_IDirect3D9Ex(iface);
    let mut output_idx: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, adapter %u, device_type %#x, src_format %#x, dst_format %#x.\n", iface, adapter, device_type as ::core::ffi::c_uint, src_format as ::core::ffi::c_uint, dst_format as ::core::ffi::c_uint);
    output_idx = adapter as ::core::ffi::c_uint;
    if output_idx >= (*d3d9).wined3d_output_count {
        return D3DERR_INVALIDCALL;
    }
    if src_format as ::core::ffi::c_uint == dst_format as ::core::ffi::c_uint {
        return S_OK;
    }
    wined3d_mutex_lock();
    hr = wined3d_check_device_format_conversion(
        *(*d3d9).wined3d_outputs.offset(output_idx as isize),
        wined3d_device_type_from_d3d(device_type),
        wined3dformat_from_d3dformat(src_format),
        wined3dformat_from_d3dformat(dst_format),
    );
    wined3d_mutex_unlock();
    return hr;
}

unsafe extern "system" fn d3d9_GetDeviceCaps(
    mut iface: *mut IDirect3D9Ex,
    mut adapter: UINT,
    mut device_type: D3DDEVTYPE,
    mut caps: *mut D3DCAPS9,
) -> HRESULT {
    let mut d3d9: *mut d3d9 = impl_from_IDirect3D9Ex(iface);
    let mut wined3d_adapter: *mut wined3d_adapter = core::ptr::null_mut::<wined3d_adapter>();
    let mut wined3d_caps: wined3d_caps = core::mem::zeroed::<wined3d_caps>();
    let mut output_idx: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, adapter %u, device_type %#x, caps %p.\n", iface, adapter, device_type as ::core::ffi::c_uint, caps);
    output_idx = adapter as ::core::ffi::c_uint;
    if output_idx >= (*d3d9).wined3d_output_count {
        return D3DERR_INVALIDCALL;
    }
    if caps.is_null() {
        return D3DERR_INVALIDCALL;
    }
    memset(
        caps as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<D3DCAPS9>() as size_t,
    );
    wined3d_mutex_lock();
    wined3d_adapter = wined3d_output_get_adapter(
        *(*d3d9).wined3d_outputs.offset(output_idx as isize),
    );
    hr = wined3d_get_device_caps(
        wined3d_adapter,
        wined3d_device_type_from_d3d(device_type),
        core::ptr::addr_of_mut!(wined3d_caps),
    );
    wined3d_mutex_unlock();
    d3d9_caps_from_wined3dcaps(
        d3d9,
        adapter as ::core::ffi::c_uint,
        caps,
        core::ptr::addr_of_mut!(wined3d_caps),
    );
    return hr;
}

unsafe extern "system" fn d3d9_GetAdapterMonitor(iface: *mut IDirect3D9Ex, adapter: UINT) -> HMONITOR {
    let d3d9: *mut d3d9 = impl_from_IDirect3D9Ex(iface);
    let mut desc: wined3d_output_desc = core::mem::zeroed();
    let mut output_idx: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;

    TRACE!("iface %p, adapter %u.\n", iface, adapter);

    output_idx = adapter as ::core::ffi::c_uint;
    if output_idx >= (*d3d9).wined3d_output_count {
        return core::ptr::null_mut();
    }

    wined3d_mutex_lock();
    hr = wined3d_output_get_desc(*(*d3d9).wined3d_outputs.offset(output_idx as isize), core::ptr::addr_of_mut!(desc));
    wined3d_mutex_unlock();

    if hr < 0 {
        WARN!("Failed to get output desc, hr %#lx.\n", hr);
        return core::ptr::null_mut();
    }

    return desc.monitor;
}
#[no_mangle]
pub unsafe extern "system" fn d3d9_CreateDevice(
    iface: *mut IDirect3D9Ex,
    adapter: UINT,
    device_type: D3DDEVTYPE,
    focus_window: HWND,
    flags: DWORD,
    parameters: *mut D3DPRESENT_PARAMETERS,
    device: *mut *mut IDirect3DDevice9,
) -> HRESULT {
    let d3d9: *mut d3d9 = impl_from_IDirect3D9Ex(iface);
    let mut object: *mut d3d9_device = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    object = calloc(1, (core::mem::size_of::<d3d9_device>()) as _)
        as *mut d3d9_device;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = device_init(
        object,
        d3d9,
        (*d3d9).wined3d,
        adapter,
        device_type,
        focus_window,
        flags,
        parameters,
        core::ptr::null_mut(),
    );
    if hr < 0 {
        WARN!("Failed to initialize device, hr %#lx.\n", hr);
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    *device = core::ptr::addr_of_mut!((*object).IDirect3DDevice9Ex_iface) as *mut IDirect3DDevice9;

    return D3D_OK;
}
unsafe extern "system" fn d3d9_GetAdapterModeCountEx(
    mut iface: *mut IDirect3D9Ex,
    mut adapter: UINT,
    mut filter: *const D3DDISPLAYMODEFILTER,
) -> UINT {
    let mut d3d9: *mut d3d9 = impl_from_IDirect3D9Ex(iface);
    let mut output_idx: ::core::ffi::c_uint = 0;
    let mut count: ::core::ffi::c_uint = 0;
    WINE_TRACE!("iface %p, adapter %u, filter %p.\n", iface, adapter, filter);
    output_idx = adapter as ::core::ffi::c_uint;
    if output_idx >= (*d3d9).wined3d_output_count {
        return 0;
    }
    if (*filter).Format as ::core::ffi::c_uint != D3DFMT_X8R8G8B8 as ::core::ffi::c_uint
        && (*filter).Format as ::core::ffi::c_uint != D3DFMT_R5G6B5 as ::core::ffi::c_uint
    {
        return 0;
    }
    wined3d_mutex_lock();
    count = wined3d_output_get_mode_count(
        *(*d3d9).wined3d_outputs.offset(output_idx as isize),
        wined3dformat_from_d3dformat((*filter).Format),
        wined3d_scanline_ordering_from_d3d((*filter).ScanLineOrdering),
        true,
    );
    wined3d_mutex_unlock();
    return count as UINT;
}
unsafe extern "system" fn d3d9_EnumAdapterModesEx(
    iface: *mut IDirect3D9Ex,
    adapter: UINT,
    filter: *const D3DDISPLAYMODEFILTER,
    mode_idx: UINT,
    mode: *mut D3DDISPLAYMODEEX,
) -> HRESULT {
    let d3d9: *mut d3d9 = impl_from_IDirect3D9Ex(iface);
    let mut wined3d_mode: wined3d_display_mode = core::mem::zeroed();
    let mut output_idx: ::core::ffi::c_uint;
    let mut hr: HRESULT;

    WINE_TRACE!("iface %p, adapter %u, filter %p, mode_idx %u, mode %p.\n", iface, adapter, filter, mode_idx, mode);

    output_idx = adapter as ::core::ffi::c_uint;
    if output_idx >= (*d3d9).wined3d_output_count {
        return D3DERR_INVALIDCALL;
    }

    if (*filter).Format as ::core::ffi::c_uint != D3DFMT_X8R8G8B8 as ::core::ffi::c_uint
        && (*filter).Format as ::core::ffi::c_uint != D3DFMT_R5G6B5 as ::core::ffi::c_uint
    {
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    hr = wined3d_output_get_mode(
        *(*d3d9).wined3d_outputs.offset(output_idx as isize),
        wined3dformat_from_d3dformat((*filter).Format),
        wined3d_scanline_ordering_from_d3d((*filter).ScanLineOrdering),
        mode_idx as ::core::ffi::c_uint,
        &raw mut wined3d_mode,
        true,
    );
    wined3d_mutex_unlock();

    if hr >= 0 {
        (*mode).Width = wined3d_mode.width;
        (*mode).Height = wined3d_mode.height;
        (*mode).RefreshRate = wined3d_mode.refresh_rate;
        (*mode).Format = d3dformat_from_wined3dformat(wined3d_mode.format_id);
        (*mode).ScanLineOrdering = d3dscanlineordering_from_wined3d(wined3d_mode.scanline_ordering);
    }

    return hr;
}

unsafe extern "system" fn d3d9_GetAdapterDisplayModeEx(
    iface: *mut IDirect3D9Ex,
    adapter: UINT,
    mode: *mut D3DDISPLAYMODEEX,
    rotation: *mut D3DDISPLAYROTATION,
) -> HRESULT {
    let mut d3d9: *mut d3d9 = impl_from_IDirect3D9Ex(iface);
    let mut wined3d_mode: wined3d_display_mode = core::mem::zeroed();
    let mut output_idx: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;

    TRACE!(
        "iface %p, adapter %u, mode %p, rotation %p.\n",
        iface,
        adapter,
        mode,
        rotation
    );

    output_idx = adapter as ::core::ffi::c_uint;
    if output_idx >= (*d3d9).wined3d_output_count {
        return D3DERR_INVALIDCALL;
    }

    if (*mode).Size as usize != ::core::mem::size_of::<D3DDISPLAYMODEEX>() as usize {
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    hr = wined3d_output_get_display_mode(
        *(*d3d9).wined3d_outputs.offset(output_idx as isize),
        &raw mut wined3d_mode,
        rotation as *mut wined3d_display_rotation,
    );
    wined3d_mutex_unlock();

    if hr >= 0 {
        (*mode).Width = wined3d_mode.width;
        (*mode).Height = wined3d_mode.height;
        (*mode).RefreshRate = wined3d_mode.refresh_rate;
        (*mode).Format = d3dformat_from_wined3dformat(wined3d_mode.format_id);
        (*mode).ScanLineOrdering = d3dscanlineordering_from_wined3d(wined3d_mode.scanline_ordering);
    }

    return hr;
}
#[no_mangle]
pub unsafe extern "system" fn d3d9_CreateDeviceEx(
    iface: *mut IDirect3D9Ex,
    adapter: UINT,
    device_type: D3DDEVTYPE,
    focus_window: HWND,
    flags: DWORD,
    parameters: *mut D3DPRESENT_PARAMETERS,
    mode: *mut D3DDISPLAYMODEEX,
    device: *mut *mut IDirect3DDevice9Ex,
) -> HRESULT {
    let d3d9: *mut d3d9 = impl_from_IDirect3D9Ex(iface);
    let mut object: *mut d3d9_device;
    let mut hr: HRESULT;

    object = calloc(1, (core::mem::size_of::<d3d9_device>()) as _) as *mut d3d9_device;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = device_init(object, d3d9, (*d3d9).wined3d, adapter, device_type, focus_window, flags, parameters, mode);
    if hr < 0 {
        WARN!("Failed to initialize device, hr %#lx.\n", hr);
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    *device = core::ptr::addr_of_mut!((*object).IDirect3DDevice9Ex_iface);

    return D3D_OK;
}
unsafe extern "system" fn d3d9_GetAdapterLUID(
    mut iface: *mut IDirect3D9Ex,
    mut adapter: UINT,
    mut luid: *mut LUID,
) -> HRESULT {
    let mut d3d9: *mut d3d9 = impl_from_IDirect3D9Ex(iface);
    let mut adapter_id: wined3d_adapter_identifier = core::mem::zeroed();
    let mut wined3d_adapter: *mut wined3d_adapter = core::ptr::null_mut::<wined3d_adapter>();
    let mut output_idx: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, adapter %u, luid %p.\n", iface, adapter, luid);
    output_idx = adapter as ::core::ffi::c_uint;
    if output_idx >= (*d3d9).wined3d_output_count {
        return D3DERR_INVALIDCALL;
    }
    adapter_id.driver_size = 0;
    adapter_id.description_size = 0;
    wined3d_adapter =
        wined3d_output_get_adapter(*(*d3d9).wined3d_outputs.offset(output_idx as isize));
    hr = wined3d_adapter_get_identifier(wined3d_adapter, 0 as uint32_t, &raw mut adapter_id);
    if hr >= 0 {
        *luid = adapter_id.adapter_luid;
    }
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn d3d9_init(
    d3d9: *mut d3d9,
    extended: BOOL,
    d3d9on12: BOOL,
) -> BOOL {
    let mut flags: DWORD = (WINED3D_PRESENT_CONVERSION
        | WINED3D_HANDLE_RESTORE
        | WINED3D_PIXEL_CENTER_INTEGER
        | WINED3D_SRGB_READ_WRITE_CONTROL
        | WINED3D_LEGACY_UNBOUND_RESOURCE_COLOR
        | WINED3D_NO_PRIMITIVE_RESTART
        | WINED3D_LEGACY_CUBEMAP_FILTERING
        | WINED3D_NORMALIZED_DEPTH_BIAS
        | WINED3D_NO_DRAW_INDIRECT) as DWORD;
    let mut adapter_idx: ::core::ffi::c_uint = 0;
    let mut output_idx: ::core::ffi::c_uint = 0;
    let mut adapter_count: ::core::ffi::c_uint = 0;
    let mut output_count: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut wined3d_adapter: *mut wined3d_adapter = ::core::ptr::null_mut::<wined3d_adapter>();

    if extended != 0 {
        flags |= WINED3D_RESTORE_MODE_ON_ACTIVATE as DWORD;
    }

    (*d3d9).IDirect3D9Ex_iface.lpVtbl =
        core::ptr::addr_of!(d3d9_vtbl) as *const IDirect3D9ExVtbl;
    (*d3d9).refcount = 1 as LONG;

    wined3d_mutex_lock();
    (*d3d9).wined3d = wined3d_create(flags);
    if (*d3d9).wined3d.is_null() {
        wined3d_mutex_unlock();
        return FALSE as BOOL;
    }

    adapter_count = wined3d_get_adapter_count((*d3d9).wined3d) as ::core::ffi::c_uint;
    adapter_idx = 0 as ::core::ffi::c_uint;
    while adapter_idx < adapter_count {
        wined3d_adapter = wined3d_get_adapter((*d3d9).wined3d, adapter_idx);
        output_count = output_count.wrapping_add(wined3d_adapter_get_output_count(wined3d_adapter));
        adapter_idx = adapter_idx.wrapping_add(1);
    }

    (*d3d9).wined3d_outputs = calloc(
        (output_count as usize) as _,
        (::core::mem::size_of::<wined3d_output>() as usize) as _,
    ) as *mut *mut wined3d_output;
    if (*d3d9).wined3d_outputs.is_null() {
        wined3d_decref((*d3d9).wined3d);
        wined3d_mutex_unlock();
        return FALSE as BOOL;
    }

    (*d3d9).wined3d_output_count = 0 as ::core::ffi::c_uint;
    adapter_idx = 0 as ::core::ffi::c_uint;
    while adapter_idx < adapter_count {
        wined3d_adapter = wined3d_get_adapter((*d3d9).wined3d, adapter_idx);
        output_count = wined3d_adapter_get_output_count(wined3d_adapter);
        output_idx = 0 as ::core::ffi::c_uint;
        while output_idx < output_count {
            let slot: *mut *mut wined3d_output = (*d3d9)
                .wined3d_outputs
                .offset((*d3d9).wined3d_output_count as isize);
            *slot = wined3d_adapter_get_output(wined3d_adapter, output_idx);
            (*d3d9).wined3d_output_count = (*d3d9).wined3d_output_count.wrapping_add(1);
            output_idx = output_idx.wrapping_add(1);
        }
        adapter_idx = adapter_idx.wrapping_add(1);
    }

    wined3d_mutex_unlock();
    (*d3d9).extended = extended;
    (*d3d9).d3d9on12 = d3d9on12;

    return TRUE as BOOL;
}

// COM vtable tables — generated from directx.c by vtbl_gen.py
/// directx.c:657 `static const IDirect3D9ExVtbl d3d9_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d9_vtbl: IDirect3D9ExVtbl = IDirect3D9ExVtbl {
    QueryInterface: Some(d3d9_QueryInterface),
    AddRef: Some(d3d9_AddRef),
    Release: Some(d3d9_Release),
    RegisterSoftwareDevice: Some(d3d9_RegisterSoftwareDevice),
    GetAdapterCount: Some(d3d9_GetAdapterCount),
    GetAdapterIdentifier: Some(d3d9_GetAdapterIdentifier),
    GetAdapterModeCount: Some(d3d9_GetAdapterModeCount),
    EnumAdapterModes: Some(d3d9_EnumAdapterModes),
    GetAdapterDisplayMode: Some(d3d9_GetAdapterDisplayMode),
    CheckDeviceType: Some(d3d9_CheckDeviceType),
    CheckDeviceFormat: Some(d3d9_CheckDeviceFormat),
    CheckDeviceMultiSampleType: Some(d3d9_CheckDeviceMultiSampleType),
    CheckDepthStencilMatch: Some(d3d9_CheckDepthStencilMatch),
    CheckDeviceFormatConversion: Some(d3d9_CheckDeviceFormatConversion),
    GetDeviceCaps: Some(d3d9_GetDeviceCaps),
    GetAdapterMonitor: Some(d3d9_GetAdapterMonitor),
    CreateDevice: Some(d3d9_CreateDevice),
    GetAdapterModeCountEx: Some(d3d9_GetAdapterModeCountEx),
    EnumAdapterModesEx: Some(d3d9_EnumAdapterModesEx),
    GetAdapterDisplayModeEx: Some(d3d9_GetAdapterDisplayModeEx),
    CreateDeviceEx: Some(d3d9_CreateDeviceEx),
    GetAdapterLUID: Some(d3d9_GetAdapterLUID),
};
