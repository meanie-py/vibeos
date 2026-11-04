use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut d3d8_vertexshader_wined3d_parent_ops: wined3d_parent_ops = unsafe {
    wined3d_parent_ops {
        wined3d_object_destroyed: Some(
            d3d8_vertexshader_wined3d_object_destroyed,
        ),
    }
};

static mut d3d8_pixelshader_wined3d_parent_ops: wined3d_parent_ops = unsafe {
    wined3d_parent_ops {
        wined3d_object_destroyed: Some(
            d3d8_pixelshader_wined3d_object_destroyed,
        ),
    }
};

pub const __WINE_DEFAULT_DEBUG_CHANNEL_D3D8: &[u8] = b"d3d8\0";

unsafe extern "system" fn d3d8_vertexshader_wined3d_object_destroyed(parent: *mut core::ffi::c_void) {
    let shader: *mut d3d8_vertex_shader = parent as *mut d3d8_vertex_shader;
    d3d8_vertex_declaration_destroy((*shader).vertex_declaration);
    free(shader as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn d3d8_vertex_shader_destroy(shader: *mut d3d8_vertex_shader) {
    if !(*shader).wined3d_shader.is_null() {
        wined3d_mutex_lock();
        wined3d_shader_decref((*shader).wined3d_shader);
        wined3d_mutex_unlock();
    } else {
        d3d8_vertexshader_wined3d_object_destroyed(shader as *mut core::ffi::c_void);
    }
}

unsafe extern "C" fn d3d8_vertexshader_create_vertexdeclaration(
    device: *mut d3d8_device,
    declaration: *const DWORD,
    shader_handle: DWORD,
    decl_ptr: *mut *mut d3d8_vertex_declaration,
) -> HRESULT {
    let mut object: *mut d3d8_vertex_declaration = core::ptr::null_mut();
    let mut hr: HRESULT;

    object = calloc(
        (1 as usize) as _,
        (core::mem::size_of::<d3d8_vertex_declaration>()) as _,
    ) as *mut d3d8_vertex_declaration;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = d3d8_vertex_declaration_init(object, device, declaration, shader_handle);
    if hr < 0 {
        WARN!("Failed to initialize vertex declaration, hr %#lx.\n", hr);
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    *decl_ptr = object;

    return D3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn d3d8_vertex_shader_init(
    shader: *mut d3d8_vertex_shader,
    device: *mut d3d8_device,
    declaration: *const DWORD,
    byte_code: *const DWORD,
    shader_handle: DWORD,
    usage: DWORD,
) -> HRESULT {
    let mut token: *const DWORD = declaration;
    let mut hr: HRESULT;

    /* Test if the vertex declaration is valid. */
    while D3DVSD_END!() != *token {
        let token_type: DWORD = ((*token & D3DVSD_TOKENTYPEMASK) >> D3DVSD_TOKENTYPESHIFT);

        if token_type == D3DVSD_TOKEN_STREAMDATA && (((token_type & 0x10000000)) == 0) {
            let type_: DWORD = (*token & D3DVSD_DATATYPEMASK) >> D3DVSD_DATATYPESHIFT;
            let reg: DWORD = (*token & D3DVSD_VERTEXREGMASK) >> D3DVSD_VERTEXREGSHIFT;

            if reg == D3DVSDE_NORMAL && type_ != D3DVSDT_FLOAT3 && byte_code.is_null() {
                WARN!("Attempt to use a non-FLOAT3 normal with the fixed-function function\n");
                return D3DERR_INVALIDCALL;
            }
        }
        token = token.offset(parse_token(token) as isize);
    }

    hr = d3d8_vertexshader_create_vertexdeclaration(
        device,
        declaration,
        shader_handle,
        core::ptr::addr_of_mut!((*shader).vertex_declaration),
    );
    if hr < 0 {
        WARN!("Failed to create vertex declaration, hr %#lx.\n", hr);
        return hr;
    }

    if !byte_code.is_null() {
        let mut desc: wined3d_shader_desc = wined3d_shader_desc {
            byte_code: core::ptr::null::<DWORD>(),
            byte_code_size: 0,
        };

        if usage != 0 {
            FIXME!("Usage %#lx not implemented.\n", usage);
        }

        desc.byte_code = byte_code;
        desc.byte_code_size = !(0 as size_t);

        wined3d_mutex_lock();
        hr = wined3d_shader_create_vs(
            (*device).wined3d_device,
            core::ptr::addr_of_mut!(desc),
            shader as *mut core::ffi::c_void,
            core::ptr::addr_of!(d3d8_vertexshader_wined3d_parent_ops),
            core::ptr::addr_of_mut!((*shader).wined3d_shader),
        );
        wined3d_mutex_unlock();
        if hr < 0 {
            WARN!("Failed to create wined3d vertex shader, hr %#lx.\n", hr);
            d3d8_vertex_declaration_destroy((*shader).vertex_declaration);
            return hr;
        }

        load_local_constants(declaration, (*shader).wined3d_shader);
    }

    return D3D_OK;
}
unsafe extern "system" fn d3d8_pixelshader_wined3d_object_destroyed(
    parent: *mut core::ffi::c_void,
) {
    free(parent);
}
#[no_mangle]
pub unsafe extern "C" fn d3d8_pixel_shader_destroy(shader: *mut d3d8_pixel_shader) {
    TRACE!("shader %p.\n", shader);
    wined3d_mutex_lock();
    wined3d_shader_decref((*shader).wined3d_shader);
    wined3d_mutex_unlock();
}
#[no_mangle]
pub unsafe extern "C" fn d3d8_pixel_shader_init(
    shader: *mut d3d8_pixel_shader,
    device: *mut d3d8_device,
    byte_code: *const DWORD,
    shader_handle: DWORD,
) -> HRESULT {
    let mut desc: wined3d_shader_desc = core::mem::zeroed();
    let mut hr: HRESULT;

    (*shader).handle = shader_handle;

    desc.byte_code = byte_code;
    desc.byte_code_size = !(0 as usize);

    wined3d_mutex_lock();
    hr = wined3d_shader_create_ps(
        (*device).wined3d_device,
        core::ptr::addr_of_mut!(desc),
        shader as *mut core::ffi::c_void,
        core::ptr::addr_of!(d3d8_pixelshader_wined3d_parent_ops),
        core::ptr::addr_of_mut!((*shader).wined3d_shader),
    );
    wined3d_mutex_unlock();
    if hr < 0 {
        WARN!("Failed to create wined3d pixel shader, hr %#lx.\n", hr);
        return hr;
    }

    D3D_OK
}
