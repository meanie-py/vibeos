use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn D3D8GetSWInfo() -> HRESULT {
    // FIXME("(void): stub\n");
    0
}

#[no_mangle]
pub unsafe extern "system" fn DebugSetMute() {}

#[no_mangle]
pub unsafe extern "system" fn Direct3DCreate8(sdk_version: UINT) -> *mut IDirect3D8 {
    let mut object: *mut d3d8;

    object = calloc(1, (core::mem::size_of::<d3d8>()) as _) as *mut d3d8;
    if object.is_null() {
        return core::ptr::null_mut();
    }

    if d3d8_init(object) == 0 {
        WARN!("Failed to initialize d3d8.\n");
        free(object as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    return core::ptr::addr_of_mut!((*object).IDirect3D8_iface);
}

#[no_mangle]
pub unsafe extern "system" fn ValidateVertexShader(
    vs_code: *const DWORD,
    declaration: *const DWORD,
    caps: *const D3DCAPS8,
    return_error: BOOL,
    errors: *mut *mut ::core::ffi::c_char,
) -> HRESULT {
    let mut message: *const ::core::ffi::c_char = b"\0".as_ptr() as *const ::core::ffi::c_char;
    let mut message_size: SIZE_T;
    let mut hr: HRESULT = E_FAIL;

    'done: {
        if vs_code.is_null() {
            message = b"Invalid code pointer.\n\0".as_ptr() as *const ::core::ffi::c_char;
            break 'done;
        }

        let version_code: DWORD = *vs_code;
        if !(version_code == D3DVS_VERSION!(1, 1)
            || version_code == D3DVS_VERSION!(1, 0))
        {
            message = b"Unsupported shader version.\n\0".as_ptr() as *const ::core::ffi::c_char;
            break 'done;
        }

        if !caps.is_null() && *vs_code > (*caps).VertexShaderVersion {
            message = b"Shader version not supported by caps.\n\0".as_ptr()
                as *const ::core::ffi::c_char;
            break 'done;
        }

        hr = S_OK;
    }

    if return_error == 0 {
        message = b"\0".as_ptr() as *const ::core::ffi::c_char;
    }
    message_size = (strlen(message) + 1) as _;
    if !errors.is_null() {
        *errors = HeapAlloc(
            GetProcessHeap(),
            0,
            message_size,
        ) as *mut ::core::ffi::c_char;
        if !(*errors).is_null() {
            memcpy(
                *errors as *mut ::core::ffi::c_void,
                message as *const ::core::ffi::c_void,
                message_size as usize,
            );
        }
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn ValidatePixelShader(
    ps_code: *const DWORD,
    caps: *const D3DCAPS8,
    return_error: BOOL,
    errors: *mut *mut ::core::ffi::c_char,
) -> HRESULT {
    let mut message: *const ::core::ffi::c_char = b"\0".as_ptr() as *const ::core::ffi::c_char;
    let mut message_size: SIZE_T;
    let mut hr: HRESULT = E_FAIL;

    if ps_code.is_null() {
        return E_FAIL;
    }

    'done: {
        let ver: DWORD = *ps_code;
        if !(ver == D3DPS_VERSION!(1, 4)
            || ver == D3DPS_VERSION!(1, 3)
            || ver == D3DPS_VERSION!(1, 2)
            || ver == D3DPS_VERSION!(1, 1)
            || ver == D3DPS_VERSION!(1, 0))
        {
            message = b"Unsupported shader version.\n\0".as_ptr() as *const ::core::ffi::c_char;
            break 'done;
        }

        if !caps.is_null() && *ps_code > (*caps).PixelShaderVersion {
            message = b"Shader version not supported by caps.\n\0".as_ptr()
                as *const ::core::ffi::c_char;
            break 'done;
        }

        hr = S_OK;
    }

    if return_error == 0 {
        message = b"\0".as_ptr() as *const ::core::ffi::c_char;
    }
    message_size = strlen(message).wrapping_add(1 as usize) as SIZE_T;
    if !errors.is_null() && {
        *errors = HeapAlloc(
            GetProcessHeap(),
            0,
            message_size,
        ) as *mut ::core::ffi::c_char;
        !(*errors).is_null()
    } {
        memcpy(
            *errors as *mut ::core::ffi::c_void,
            message as *const ::core::ffi::c_void,
            message_size as usize,
        );
    }

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn d3d8_resource_cleanup(resource: *mut d3d8_resource) {
    wined3d_private_store_cleanup(core::ptr::addr_of_mut!((*resource).private_store));
}
#[no_mangle]
pub unsafe extern "C" fn d3d8_resource_free_private_data(resource: *mut d3d8_resource, guid: *const GUID) -> HRESULT {
    let mut entry: *mut wined3d_private_data = core::ptr::null_mut();
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
pub unsafe extern "C" fn d3d8_resource_get_private_data(resource: *mut d3d8_resource, guid: *const GUID, data: *mut ::core::ffi::c_void, data_size: *mut DWORD) -> HRESULT {
    let mut stored_data: *const wined3d_private_data;
    let mut size_in: DWORD;
    let mut hr: HRESULT = 0;

    wined3d_mutex_lock();
    stored_data = wined3d_private_store_get_private_data(core::ptr::addr_of_mut!((*resource).private_store), guid);
    'done: {
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
        memcpy(data, core::ptr::addr_of!((*stored_data).content.data) as *const ::core::ffi::c_void, (*stored_data).size as usize);
        hr = D3D_OK;
    }

    wined3d_mutex_unlock();
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn d3d8_resource_init(resource: *mut d3d8_resource) {
    (*resource).refcount = 1 as LONG;
    wined3d_private_store_init(core::ptr::addr_of_mut!((*resource).private_store));
}
#[no_mangle]
pub unsafe extern "C" fn d3d8_resource_set_private_data(
    resource: *mut d3d8_resource,
    guid: *const GUID,
    data: *const core::ffi::c_void,
    data_size: DWORD,
    flags: DWORD,
) -> HRESULT {
    let mut hr: HRESULT;
    wined3d_mutex_lock();
    hr = wined3d_private_store_set_private_data(core::ptr::addr_of_mut!((*resource).private_store), guid, data, data_size, flags);
    wined3d_mutex_unlock();
    return hr;
}
