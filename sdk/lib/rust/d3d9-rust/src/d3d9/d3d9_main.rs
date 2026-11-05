use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut D3DPERF_event_level: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

static mut shader_validator: IDirect3DShaderValidator9 = unsafe {
    IDirect3DShaderValidator9 {
        vtbl: &raw const shader_validator_vtbl,
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(d3d9); (debug channel macro, no Rust declaration emitted)

pub type shader_validator_cb = Option<unsafe extern "system" fn(
    file: *const i8,
    line: i32,
    arg3: DWORD_PTR,
    message_id: DWORD_PTR,
    message: *const i8,
    context: *mut core::ffi::c_void,
) -> HRESULT>;

#[repr(C)]
pub struct IDirect3DShaderValidator9 {
    pub vtbl: *const IDirect3DShaderValidator9Vtbl,
}

pub type PIDirect3DShaderValidator9 = *mut IDirect3DShaderValidator9;

#[repr(C)]
pub struct IDirect3DShaderValidator9Vtbl {
    pub QueryInterface: unsafe extern "system" fn(
        iface: *mut IDirect3DShaderValidator9,
        iid: REFIID,
        out: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(iface: *mut IDirect3DShaderValidator9) -> ULONG,
    pub Release: unsafe extern "system" fn(iface: *mut IDirect3DShaderValidator9) -> ULONG,
    pub Begin: unsafe extern "system" fn(
        iface: *mut IDirect3DShaderValidator9,
        callback: shader_validator_cb,
        context: *mut core::ffi::c_void,
        arg3: DWORD_PTR,
    ) -> HRESULT,
    pub Instruction: unsafe extern "system" fn(
        iface: *mut IDirect3DShaderValidator9,
        file: *const i8,
        line: i32,
        tokens: *const DWORD,
        token_count: u32,
    ) -> HRESULT,
    pub End: unsafe extern "system" fn(iface: *mut IDirect3DShaderValidator9) -> HRESULT,
}

#[no_mangle]
pub unsafe extern "system" fn DebugSetMute() {}

#[no_mangle]
pub unsafe extern "system" fn Direct3DCreate9(sdk_version: UINT) -> *mut IDirect3D9 {
    let mut object: *mut d3d9;

    object = calloc(1, (core::mem::size_of::<d3d9>()) as _) as *mut d3d9;
    if object.is_null() {
        return core::ptr::null_mut::<IDirect3D9>();
    }

    if d3d9_init(object, (FALSE) as BOOL, (FALSE) as BOOL) == 0 {
        WARN!("Failed to initialize d3d9.\n");
        free(object as *mut core::ffi::c_void);
        return core::ptr::null_mut::<IDirect3D9>();
    }

    core::ptr::addr_of_mut!((*object).IDirect3D9Ex_iface) as *mut IDirect3D9
}

#[no_mangle]
pub unsafe extern "system" fn Direct3DCreate9Ex(sdk_version: UINT, d3d9ex: *mut *mut IDirect3D9Ex) -> HRESULT {
    let mut object: *mut d3d9 = core::ptr::null_mut();

    object = calloc((1 as size_t) as _, (core::mem::size_of::<d3d9>() as size_t) as _) as *mut d3d9;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    if d3d9_init(object, (TRUE) as BOOL, (FALSE) as BOOL) == 0 {
        WARN!("Failed to initialize d3d9.\n");
        free(object as *mut core::ffi::c_void);
        return D3DERR_NOTAVAILABLE;
    }

    *d3d9ex = core::ptr::addr_of_mut!((*object).IDirect3D9Ex_iface);

    return D3D_OK;
}

#[no_mangle]
pub unsafe extern "system" fn Direct3DCreate9On12(
    sdk_version: UINT,
    d3d9on12_args: *mut D3D9ON12_ARGS,
    d3d9on12_args_count: UINT,
) -> *mut IDirect3D9 {
    let mut object: *mut d3d9;
    let mut d3d9on12: BOOL = (TRUE as i32);

    object = calloc((1 as size_t) as _, (core::mem::size_of::<d3d9>() as size_t) as _) as *mut d3d9;
    if object.is_null() {
        return core::ptr::null_mut::<IDirect3D9>();
    }

    if d3d9on12_args.is_null() || (*d3d9on12_args).Enable9On12 == 0 || d3d9on12_args_count == 0 {
        d3d9on12 = (FALSE as i32);
    }

    if d3d9_init(object, (TRUE) as BOOL, d3d9on12) == 0 {
        WARN!("Failed to initialize d3d9.\n");
        free(object as *mut core::ffi::c_void);
        return core::ptr::null_mut::<IDirect3D9>();
    }

    return core::ptr::addr_of_mut!((*object).IDirect3D9Ex_iface) as *mut IDirect3D9;
}

unsafe extern "system" fn shader_validator_QueryInterface(iface: *mut IDirect3DShaderValidator9, iid: *const GUID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    WARN!("%s not implemented, returning E_NOINTERFACE.\n", debugstr_guid(iid));
    *out = core::ptr::null_mut();

    return E_NOINTERFACE;
}
unsafe extern "system" fn shader_validator_AddRef(
    iface: *mut IDirect3DShaderValidator9,
) -> ULONG {
    // TRACE("iface %p.\n", iface);
    2
}
unsafe extern "system" fn shader_validator_Release(
    iface: *mut IDirect3DShaderValidator9,
) -> ULONG {
    // TRACE("iface %p.\n", iface);
    1
}
unsafe extern "system" fn shader_validator_Begin(
    iface: *mut IDirect3DShaderValidator9,
    callback: shader_validator_cb,
    context: *mut core::ffi::c_void,
    arg3: DWORD_PTR,
) -> HRESULT {
    WARN!("iface %p, callback %p, context %p, arg3 %#Ix, stub!\n", iface, callback, context, arg3);
    S_OK
}

unsafe extern "system" fn shader_validator_Instruction(
    iface: *mut IDirect3DShaderValidator9,
    file: *const core::ffi::c_char,
    line: i32,
    tokens: *const DWORD,
    token_count: u32,
) -> HRESULT {
    WARN!("iface %p, file %s, line %u, tokens %p, token_count %u, stub!\n",
            iface, debugstr_a(file), line, tokens, token_count);

    return S_OK;
}
unsafe extern "system" fn shader_validator_End(
    iface: *mut IDirect3DShaderValidator9,
) -> HRESULT {
    WARN!("iface %p, stub!\n", iface);
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn Direct3DShaderValidatorCreate9() -> *mut IDirect3DShaderValidator9 {
    WINE_FIXME!("Returning stub validator %p.\n", core::ptr::addr_of_mut!(shader_validator));
    return core::ptr::addr_of_mut!(shader_validator);
}

#[no_mangle]
pub unsafe extern "system" fn D3DPERF_BeginEvent(color: D3DCOLOR, name: *const WCHAR) -> ::core::ffi::c_int {
    WINE_TRACE!("color 0x%08lx, name %s.\n", color, debugstr_w(name));
    let fresh15 = D3DPERF_event_level;
    D3DPERF_event_level = D3DPERF_event_level + 1;
    return fresh15;
}

#[no_mangle]
pub unsafe extern "system" fn D3DPERF_EndEvent() -> core::ffi::c_int {
    WINE_TRACE!("(void) : stub\n");
    D3DPERF_event_level -= 1;
    return D3DPERF_event_level;
}

#[no_mangle]
pub unsafe extern "system" fn D3DPERF_GetStatus() -> DWORD {
    // FIXME("(void) : stub\n");
    0
}

#[no_mangle]
pub unsafe extern "system" fn D3DPERF_SetOptions(options: DWORD) {
    // FIXME("options %#lx, stub!\n", options);
}

#[no_mangle]
pub unsafe extern "system" fn D3DPERF_QueryRepeatFrame() -> BOOL {
    // FIXME("(void) : stub\n");
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn D3DPERF_SetMarker(color: D3DCOLOR, name: *const WCHAR) {
    // FIXME("color 0x%08lx, name %s stub!\n", color, debugstr_w(name));
}

#[no_mangle]
pub unsafe extern "system" fn D3DPERF_SetRegion(color: D3DCOLOR, name: *const WCHAR) {
    // FIXME("color 0x%08lx, name %s stub!\n", color, debugstr_w(name));
}
#[no_mangle]
pub unsafe extern "C" fn d3d9_resource_cleanup(resource: *mut d3d9_resource) {
    wined3d_private_store_cleanup(core::ptr::addr_of_mut!((*resource).private_store));
}
#[no_mangle]
pub unsafe extern "C" fn d3d9_resource_free_private_data(resource: *mut d3d9_resource, guid: *const GUID) -> HRESULT {
    let mut entry: *mut wined3d_private_data = core::ptr::null_mut::<wined3d_private_data>();

    wined3d_mutex_lock();
    entry = wined3d_private_store_get_private_data(core::ptr::addr_of_mut!((*resource).private_store), guid);
    if entry.is_null() {
        wined3d_mutex_unlock();
        return D3DERR_NOTFOUND;
    }

    wined3d_private_store_free_private_data(core::ptr::addr_of_mut!((*resource).private_store), entry);
    wined3d_mutex_unlock();

    return D3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn d3d9_resource_get_private_data(
    resource: *mut d3d9_resource,
    guid: *const GUID,
    data: *mut ::core::ffi::c_void,
    data_size: *mut DWORD,
) -> HRESULT {
    let mut stored_data: *const wined3d_private_data = core::ptr::null();
    let mut size_in: DWORD = 0;
    let mut hr: HRESULT = 0;

    wined3d_mutex_lock();
    'done: {
        stored_data = wined3d_private_store_get_private_data(core::ptr::addr_of_mut!((*resource).private_store), guid);
        if stored_data.is_null() {
            hr = D3DERR_NOTFOUND;
            break 'done;
        }

        size_in = *data_size;
        *data_size = (*stored_data).size;
        if data.is_null() {
            hr = D3D_OK;
            break 'done;
        }
        if size_in < (*stored_data).size {
            hr = D3DERR_MOREDATA;
            break 'done;
        }

        if ((*stored_data).flags & WINED3DSPD_IUNKNOWN) != 0 {
            IUnknown_AddRef((*stored_data).content.object);
        }
        memcpy(
            data,
            core::ptr::addr_of!((*stored_data).content.data) as *const ::core::ffi::c_void,
            (*stored_data).size as usize,
        );
        hr = D3D_OK;
    }
    wined3d_mutex_unlock();
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn d3d9_resource_init(resource: *mut d3d9_resource) {
    (*resource).refcount = 1;
    wined3d_private_store_init(core::ptr::addr_of_mut!((*resource).private_store));
}
#[no_mangle]
pub unsafe extern "C" fn d3d9_resource_set_private_data(resource: *mut d3d9_resource, guid: *const GUID, data: *const core::ffi::c_void, data_size: DWORD, flags: DWORD) -> HRESULT {
    let mut hr: HRESULT;
    wined3d_mutex_lock();
    hr = wined3d_private_store_set_private_data(core::ptr::addr_of_mut!((*resource).private_store), guid, data, data_size, flags);
    wined3d_mutex_unlock();
    return hr;
}

// COM vtable tables — generated from d3d9_main.c by vtbl_gen.py
/// d3d9_main.c:203 `static const IDirect3DShaderValidator9Vtbl shader_validator_vtbl`  (slot order: c-header:d3d9_main.c)
pub static shader_validator_vtbl: IDirect3DShaderValidator9Vtbl = IDirect3DShaderValidator9Vtbl {
    QueryInterface: shader_validator_QueryInterface,
    AddRef: shader_validator_AddRef,
    Release: shader_validator_Release,
    Begin: shader_validator_Begin,
    Instruction: shader_validator_Instruction,
    End: shader_validator_End,
};
