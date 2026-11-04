use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_19 {
    pub usage: BYTE,
    pub usage_idx: BYTE,
}

static mut wined3d_type_sizes: [size_t; 17] = [
        (1 as usize).wrapping_mul(::core::mem::size_of::<::core::ffi::c_float>() as usize),
        (2 as usize).wrapping_mul(::core::mem::size_of::<::core::ffi::c_float>() as usize),
        (3 as usize).wrapping_mul(::core::mem::size_of::<::core::ffi::c_float>() as usize),
        (4 as usize).wrapping_mul(::core::mem::size_of::<::core::ffi::c_float>() as usize),
        (4 as usize).wrapping_mul(::core::mem::size_of::<BYTE>() as usize),
        (4 as usize).wrapping_mul(::core::mem::size_of::<BYTE>() as usize),
        (2 as usize).wrapping_mul(::core::mem::size_of::<::core::ffi::c_short>() as usize),
        (4 as usize).wrapping_mul(::core::mem::size_of::<::core::ffi::c_short>() as usize),
        (4 as usize).wrapping_mul(::core::mem::size_of::<BYTE>() as usize),
        (2 as usize).wrapping_mul(::core::mem::size_of::<::core::ffi::c_short>() as usize),
        (4 as usize).wrapping_mul(::core::mem::size_of::<::core::ffi::c_short>() as usize),
        (2 as usize).wrapping_mul(::core::mem::size_of::<::core::ffi::c_short>() as usize),
        (4 as usize).wrapping_mul(::core::mem::size_of::<::core::ffi::c_short>() as usize),
        (3 as usize).wrapping_mul(::core::mem::size_of::<::core::ffi::c_short>() as usize),
        (3 as usize).wrapping_mul(::core::mem::size_of::<::core::ffi::c_short>() as usize),
        (2 as usize).wrapping_mul(::core::mem::size_of::<::core::ffi::c_short>() as usize),
        (4 as usize).wrapping_mul(::core::mem::size_of::<::core::ffi::c_short>() as usize),
    ];

static mut wined3d_format_lookup: [wined3d_format_id; 17] = [
    WINED3DFMT_R32_FLOAT,
    WINED3DFMT_R32G32_FLOAT,
    WINED3DFMT_R32G32B32_FLOAT,
    WINED3DFMT_R32G32B32A32_FLOAT,
    WINED3DFMT_B8G8R8A8_UNORM,
    WINED3DFMT_R8G8B8A8_UINT,
    WINED3DFMT_R16G16_SINT,
    WINED3DFMT_R16G16B16A16_SINT,
    WINED3DFMT_R8G8B8A8_UNORM,
    WINED3DFMT_R16G16_SNORM,
    WINED3DFMT_R16G16B16A16_SNORM,
    WINED3DFMT_R16G16_UNORM,
    WINED3DFMT_R16G16B16A16_UNORM,
    WINED3DFMT_R10G10B10X2_UINT,
    WINED3DFMT_R10G10B10X2_SNORM,
    WINED3DFMT_R16G16_FLOAT,
    WINED3DFMT_R16G16B16A16_FLOAT,
];

static mut wined3d_usage_lookup: [C2RustUnnamed_19; 17] = [
    C2RustUnnamed_19 {
        usage: WINED3D_DECL_USAGE_POSITION as ::core::ffi::c_int as BYTE,
        usage_idx: 0 as BYTE,
    },
    C2RustUnnamed_19 {
        usage: WINED3D_DECL_USAGE_BLEND_WEIGHT as ::core::ffi::c_int as BYTE,
        usage_idx: 0 as BYTE,
    },
    C2RustUnnamed_19 {
        usage: WINED3D_DECL_USAGE_BLEND_INDICES as ::core::ffi::c_int as BYTE,
        usage_idx: 0 as BYTE,
    },
    C2RustUnnamed_19 {
        usage: WINED3D_DECL_USAGE_NORMAL as ::core::ffi::c_int as BYTE,
        usage_idx: 0 as BYTE,
    },
    C2RustUnnamed_19 {
        usage: WINED3D_DECL_USAGE_PSIZE as ::core::ffi::c_int as BYTE,
        usage_idx: 0 as BYTE,
    },
    C2RustUnnamed_19 {
        usage: WINED3D_DECL_USAGE_COLOR as ::core::ffi::c_int as BYTE,
        usage_idx: 0 as BYTE,
    },
    C2RustUnnamed_19 {
        usage: WINED3D_DECL_USAGE_COLOR as ::core::ffi::c_int as BYTE,
        usage_idx: 1 as BYTE,
    },
    C2RustUnnamed_19 {
        usage: WINED3D_DECL_USAGE_TEXCOORD as ::core::ffi::c_int as BYTE,
        usage_idx: 0 as BYTE,
    },
    C2RustUnnamed_19 {
        usage: WINED3D_DECL_USAGE_TEXCOORD as ::core::ffi::c_int as BYTE,
        usage_idx: 1 as BYTE,
    },
    C2RustUnnamed_19 {
        usage: WINED3D_DECL_USAGE_TEXCOORD as ::core::ffi::c_int as BYTE,
        usage_idx: 2 as BYTE,
    },
    C2RustUnnamed_19 {
        usage: WINED3D_DECL_USAGE_TEXCOORD as ::core::ffi::c_int as BYTE,
        usage_idx: 3 as BYTE,
    },
    C2RustUnnamed_19 {
        usage: WINED3D_DECL_USAGE_TEXCOORD as ::core::ffi::c_int as BYTE,
        usage_idx: 4 as BYTE,
    },
    C2RustUnnamed_19 {
        usage: WINED3D_DECL_USAGE_TEXCOORD as ::core::ffi::c_int as BYTE,
        usage_idx: 5 as BYTE,
    },
    C2RustUnnamed_19 {
        usage: WINED3D_DECL_USAGE_TEXCOORD as ::core::ffi::c_int as BYTE,
        usage_idx: 6 as BYTE,
    },
    C2RustUnnamed_19 {
        usage: WINED3D_DECL_USAGE_TEXCOORD as ::core::ffi::c_int as BYTE,
        usage_idx: 7 as BYTE,
    },
    C2RustUnnamed_19 {
        usage: WINED3D_DECL_USAGE_POSITION as ::core::ffi::c_int as BYTE,
        usage_idx: 1 as BYTE,
    },
    C2RustUnnamed_19 {
        usage: WINED3D_DECL_USAGE_NORMAL as ::core::ffi::c_int as BYTE,
        usage_idx: 1 as BYTE,
    },
];

static mut d3d8_vertexdeclaration_wined3d_parent_ops: wined3d_parent_ops = unsafe {
    wined3d_parent_ops {
        wined3d_object_destroyed: Some(
            d3d8_vertexdeclaration_wined3d_object_destroyed,
        ),
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(d3d8);
pub const WINE_DEFAULT_DEBUG_CHANNEL_NAME: &str = "d3d8";

#[repr(C)]
pub struct wined3d_usage_lookup_t {
    pub usage: BYTE,
    pub usage_idx: BYTE,
}

unsafe extern "C" fn debug_d3dvsdt_type(
    mut d3dvsdt_type: D3DVSDT_TYPE,
) -> *const ::core::ffi::c_char {
    let _ = (
        D3DVSDT_FLOAT1,
        D3DVSDT_FLOAT2,
        D3DVSDT_FLOAT3,
        D3DVSDT_FLOAT4,
        D3DVSDT_D3DCOLOR,
        D3DVSDT_UBYTE4,
        D3DVSDT_SHORT2,
        D3DVSDT_SHORT4,
    );
    match d3dvsdt_type as ::core::ffi::c_uint {
        D3DVSDT_FLOAT1 => return b"D3DVSDT_FLOAT1\0" as *const u8 as *const ::core::ffi::c_char,
        D3DVSDT_FLOAT2 => return b"D3DVSDT_FLOAT2\0" as *const u8 as *const ::core::ffi::c_char,
        D3DVSDT_FLOAT3 => return b"D3DVSDT_FLOAT3\0" as *const u8 as *const ::core::ffi::c_char,
        D3DVSDT_FLOAT4 => return b"D3DVSDT_FLOAT4\0" as *const u8 as *const ::core::ffi::c_char,
        D3DVSDT_D3DCOLOR => return b"D3DVSDT_D3DCOLOR\0" as *const u8 as *const ::core::ffi::c_char,
        D3DVSDT_UBYTE4 => return b"D3DVSDT_UBYTE4\0" as *const u8 as *const ::core::ffi::c_char,
        D3DVSDT_SHORT2 => return b"D3DVSDT_SHORT2\0" as *const u8 as *const ::core::ffi::c_char,
        D3DVSDT_SHORT4 => return b"D3DVSDT_SHORT4\0" as *const u8 as *const ::core::ffi::c_char,
        _ => {
            WINE_FIXME!("Unrecognized D3DVSDT_TYPE %#x\n", d3dvsdt_type as ::core::ffi::c_uint);
            return b"unrecognized\0" as *const u8 as *const ::core::ffi::c_char;
        }
    };
}

unsafe extern "C" fn debug_d3dvsde_register(
    mut d3dvsde_register: D3DVSDE_REGISTER,
) -> *const ::core::ffi::c_char {
    match d3dvsde_register {
        D3DVSDE_POSITION => return concat!(stringify!(D3DVSDE_POSITION), "\0").as_ptr() as *const ::core::ffi::c_char,
        D3DVSDE_BLENDWEIGHT => return concat!(stringify!(D3DVSDE_BLENDWEIGHT), "\0").as_ptr() as *const ::core::ffi::c_char,
        D3DVSDE_BLENDINDICES => return concat!(stringify!(D3DVSDE_BLENDINDICES), "\0").as_ptr() as *const ::core::ffi::c_char,
        D3DVSDE_NORMAL => return concat!(stringify!(D3DVSDE_NORMAL), "\0").as_ptr() as *const ::core::ffi::c_char,
        D3DVSDE_PSIZE => return concat!(stringify!(D3DVSDE_PSIZE), "\0").as_ptr() as *const ::core::ffi::c_char,
        D3DVSDE_DIFFUSE => return concat!(stringify!(D3DVSDE_DIFFUSE), "\0").as_ptr() as *const ::core::ffi::c_char,
        D3DVSDE_SPECULAR => return concat!(stringify!(D3DVSDE_SPECULAR), "\0").as_ptr() as *const ::core::ffi::c_char,
        D3DVSDE_TEXCOORD0 => return concat!(stringify!(D3DVSDE_TEXCOORD0), "\0").as_ptr() as *const ::core::ffi::c_char,
        D3DVSDE_TEXCOORD1 => return concat!(stringify!(D3DVSDE_TEXCOORD1), "\0").as_ptr() as *const ::core::ffi::c_char,
        D3DVSDE_TEXCOORD2 => return concat!(stringify!(D3DVSDE_TEXCOORD2), "\0").as_ptr() as *const ::core::ffi::c_char,
        D3DVSDE_TEXCOORD3 => return concat!(stringify!(D3DVSDE_TEXCOORD3), "\0").as_ptr() as *const ::core::ffi::c_char,
        D3DVSDE_TEXCOORD4 => return concat!(stringify!(D3DVSDE_TEXCOORD4), "\0").as_ptr() as *const ::core::ffi::c_char,
        D3DVSDE_TEXCOORD5 => return concat!(stringify!(D3DVSDE_TEXCOORD5), "\0").as_ptr() as *const ::core::ffi::c_char,
        D3DVSDE_TEXCOORD6 => return concat!(stringify!(D3DVSDE_TEXCOORD6), "\0").as_ptr() as *const ::core::ffi::c_char,
        D3DVSDE_TEXCOORD7 => return concat!(stringify!(D3DVSDE_TEXCOORD7), "\0").as_ptr() as *const ::core::ffi::c_char,
        D3DVSDE_POSITION2 => return concat!(stringify!(D3DVSDE_POSITION2), "\0").as_ptr() as *const ::core::ffi::c_char,
        D3DVSDE_NORMAL2 => return concat!(stringify!(D3DVSDE_NORMAL2), "\0").as_ptr() as *const ::core::ffi::c_char,
        _ => {
            FIXME!("Unrecognized D3DVSDE_REGISTER %#x\n", d3dvsde_register);
            return b"unrecognized\0".as_ptr() as *const ::core::ffi::c_char;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn parse_token(pToken: *const DWORD) -> size_t {
    let token: DWORD = *pToken;
    let mut tokenlen: size_t = 1 as size_t;
    match (token & D3DVSD_TOKENTYPEMASK as DWORD) >> D3DVSD_TOKENTYPESHIFT as DWORD {
        D3DVSD_TOKEN_NOP => {
            TRACE!(" 0x%08lx NOP()\n", token);
        }
        D3DVSD_TOKEN_STREAM => {
            if token & D3DVSD_STREAMTESSMASK as DWORD != 0 {
            } else {
            }
        }
        D3DVSD_TOKEN_STREAMDATA => {
            if token & 0x10000000 as DWORD != 0 {
            } else {
                let type_: DWORD = (token & D3DVSD_DATATYPEMASK as DWORD) >> D3DVSD_DATATYPESHIFT as DWORD;
                let reg: DWORD = (token & D3DVSD_VERTEXREGMASK as DWORD) >> D3DVSD_VERTEXREGSHIFT as DWORD;
            }
        }
        D3DVSD_TOKEN_TESSELLATOR => {
            if token & 0x10000000 as DWORD != 0 {
                let type_: DWORD = (token & D3DVSD_DATATYPEMASK as DWORD) >> D3DVSD_DATATYPESHIFT as DWORD;
                let reg: DWORD = (token & D3DVSD_VERTEXREGMASK as DWORD) >> D3DVSD_VERTEXREGSHIFT as DWORD;
            } else {
                let type_: DWORD = (token & D3DVSD_DATATYPEMASK as DWORD) >> D3DVSD_DATATYPESHIFT as DWORD;
                let regout: DWORD = (token & D3DVSD_VERTEXREGMASK as DWORD) >> D3DVSD_VERTEXREGSHIFT as DWORD;
                let regin: DWORD = (token & D3DVSD_VERTEXREGINMASK as DWORD) >> D3DVSD_VERTEXREGINSHIFT as DWORD;
            }
        }
        D3DVSD_TOKEN_CONSTMEM => {
            {
                let count: DWORD = (token & D3DVSD_CONSTCOUNTMASK as DWORD) >> D3DVSD_CONSTCOUNTSHIFT as DWORD;
                tokenlen = (4 as DWORD)
                    .wrapping_mul(count)
                    .wrapping_add(1 as DWORD) as size_t;
            }
        }
        D3DVSD_TOKEN_EXT => {
            {
                let count: DWORD = (token & D3DVSD_CONSTCOUNTMASK as DWORD) >> D3DVSD_CONSTCOUNTSHIFT as DWORD;
                let extinfo: DWORD = (token & D3DVSD_EXTINFOMASK as DWORD) >> D3DVSD_EXTINFOSHIFT as DWORD;
                tokenlen = count.wrapping_add(1 as DWORD) as size_t;
            }
        }
        D3DVSD_TOKEN_END => {
            TRACE!(" 0x%08lx END()\n", token);
        }
        _ => {
            TRACE!(" 0x%08lx UNKNOWN\n", token);
        }
    }
    return tokenlen;
}
#[no_mangle]
pub unsafe extern "C" fn load_local_constants(d3d8_elements: *const DWORD, wined3d_vertex_shader: *mut wined3d_shader) {
    let mut token: *const DWORD = d3d8_elements;
    while *token != D3DVSD_END!() {
        if ((*token & D3DVSD_TOKENTYPEMASK) >> D3DVSD_TOKENTYPESHIFT) == D3DVSD_TOKEN_CONSTMEM {
            let count: DWORD = (*token & D3DVSD_CONSTCOUNTMASK) >> D3DVSD_CONSTCOUNTSHIFT;
            let constant_idx: DWORD = (*token & D3DVSD_CONSTADDRESSMASK) >> D3DVSD_CONSTADDRESSSHIFT;
            let mut hr: HRESULT = 0;

            if ((TRACE_ON!(__wine_dbch_d3d8)) != 0) {
                let mut i: DWORD = 0;
                while i < count {
                    i = i.wrapping_add(1);
                }
            }
            hr = wined3d_shader_set_local_constants_float(
                wined3d_vertex_shader,
                constant_idx,
                (token as *const f32).offset(1 as isize),
                count,
            );
            if hr < 0 {
                ERR!("Failed setting shader constants\n");
            }
        }

        token = token.offset(parse_token(token) as isize);
    }
}

unsafe extern "C" fn convert_to_wined3d_declaration(
    d3d8_elements: *const DWORD,
    d3d8_elements_size: *mut DWORD,
    wined3d_elements: *mut *mut wined3d_vertex_element,
    stream_map: *mut DWORD,
) -> UINT {
    let mut element: *mut wined3d_vertex_element;
    let mut token: *const DWORD = d3d8_elements;
    let mut token_type: D3DVSD_TOKENTYPE;
    let mut element_count: ::core::ffi::c_uint = 0;
    let mut stream: WORD = 0;
    let mut offset: ::core::ffi::c_int = 0;

    *stream_map = 0;
    /* 128 should be enough for anyone... */
    *wined3d_elements = calloc(
        128,
        (core::mem::size_of::<wined3d_vertex_element>()) as _,
    ) as *mut wined3d_vertex_element;
    while D3DVSD_END!() != *token {
        token_type = ((*token & D3DVSD_TOKENTYPEMASK) >> D3DVSD_TOKENTYPESHIFT) as D3DVSD_TOKENTYPE;

        if token_type == D3DVSD_TOKEN_STREAM && (*token & D3DVSD_STREAMTESSMASK) == 0 {
            stream = ((*token & D3DVSD_STREAMNUMBERMASK) >> D3DVSD_STREAMNUMBERSHIFT) as WORD;
            offset = 0;
        } else if token_type == D3DVSD_TOKEN_STREAMDATA && (*token & D3DVSD_DATALOADTYPEMASK) == 0 {
            let type_: DWORD = (*token & D3DVSD_DATATYPEMASK) >> D3DVSD_DATATYPESHIFT;
            let reg: DWORD = (*token & D3DVSD_VERTEXREGMASK) >> D3DVSD_VERTEXREGSHIFT;

            element = (*wined3d_elements).offset(element_count as isize);
            element_count = element_count.wrapping_add(1);
            (*element).format = wined3d_format_lookup[type_ as usize];
            (*element).input_slot = stream as _;
            (*element).offset = offset as _;
            (*element).output_slot = reg as _;
            (*element).input_slot_class = WINED3D_INPUT_PER_VERTEX_DATA;
            (*element).instance_data_step_rate = 0;
            (*element).method = WINED3D_DECL_METHOD_DEFAULT as BYTE;
            (*element).usage = wined3d_usage_lookup[reg as usize].usage;
            (*element).usage_idx = wined3d_usage_lookup[reg as usize].usage_idx;

            *stream_map |= (1 as DWORD) << stream;

            offset = (offset as usize).wrapping_add(wined3d_type_sizes[type_ as usize]) as ::core::ffi::c_int;
        } else if token_type == D3DVSD_TOKEN_STREAMDATA && (*token & D3DVSD_DATALOADTYPEMASK) != 0 {
            offset = (offset as usize).wrapping_add(
                core::mem::size_of::<DWORD>().wrapping_mul(
                    ((*token & D3DVSD_SKIPCOUNTMASK) >> D3DVSD_SKIPCOUNTSHIFT) as usize,
                ),
            ) as ::core::ffi::c_int;
        }

        if element_count >= 127 {
            ERR!("More than 127 elements?\n");
            break;
        } else {
            token = token.offset(parse_token(token) as isize);
        }
    }

    token = token.offset(1);
    *d3d8_elements_size = (token.offset_from(d3d8_elements) as usize).wrapping_mul(core::mem::size_of::<DWORD>()) as DWORD;

    return element_count as UINT;
}
unsafe extern "system" fn d3d8_vertexdeclaration_wined3d_object_destroyed(parent: *mut core::ffi::c_void) {
    let declaration: *mut d3d8_vertex_declaration = parent as *mut d3d8_vertex_declaration;
    free((*declaration).elements as *mut core::ffi::c_void);
    free(declaration as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn d3d8_vertex_declaration_destroy(declaration: *mut d3d8_vertex_declaration) {
    DPRINT1!("declaration %p.\n", declaration);
    wined3d_vertex_declaration_decref((*declaration).wined3d_vertex_declaration);
}
#[no_mangle]
pub unsafe extern "C" fn d3d8_vertex_declaration_init(
    declaration: *mut d3d8_vertex_declaration,
    device: *mut d3d8_device,
    elements: *const DWORD,
    shader_handle: DWORD,
) -> HRESULT {
    let mut wined3d_elements: *mut wined3d_vertex_element = core::ptr::null_mut();
    let mut wined3d_element_count: UINT;
    let mut hr: HRESULT;

    (*declaration).shader_handle = shader_handle;

    wined3d_element_count = convert_to_wined3d_declaration(
        elements,
        core::ptr::addr_of_mut!((*declaration).elements_size),
        core::ptr::addr_of_mut!(wined3d_elements),
        core::ptr::addr_of_mut!((*declaration).stream_map),
    );
    (*declaration).elements = malloc((*declaration).elements_size as size_t) as *mut DWORD;
    if (*declaration).elements.is_null() {
        ERR!("Failed to allocate vertex declaration elements memory.\n");
        free(wined3d_elements as *mut core::ffi::c_void);
        return E_OUTOFMEMORY;
    }

    memcpy(
        (*declaration).elements as *mut core::ffi::c_void,
        elements as *const core::ffi::c_void,
        (*declaration).elements_size as size_t,
    );

    wined3d_mutex_lock();
    hr = wined3d_vertex_declaration_create(
        (*device).wined3d_device,
        wined3d_elements,
        wined3d_element_count,
        declaration as *mut core::ffi::c_void,
        core::ptr::addr_of!(d3d8_vertexdeclaration_wined3d_parent_ops),
        core::ptr::addr_of_mut!((*declaration).wined3d_vertex_declaration),
    );
    wined3d_mutex_unlock();
    free(wined3d_elements as *mut core::ffi::c_void);
    if hr < 0 {
        WARN!("Failed to create wined3d vertex declaration, hr %#lx.\n", hr);
        free((*declaration).elements as *mut core::ffi::c_void);
        if hr == E_INVALIDARG {
            hr = E_FAIL;
        }
        return hr;
    }

    return D3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn d3d8_vertex_declaration_init_fvf(
    declaration: *mut d3d8_vertex_declaration,
    device: *mut d3d8_device,
    fvf: DWORD,
) -> HRESULT {
    let mut hr: HRESULT;

    (*declaration).elements = core::ptr::null_mut();
    (*declaration).elements_size = 0;
    (*declaration).stream_map = 1;
    (*declaration).shader_handle = fvf;

    hr = wined3d_vertex_declaration_create_from_fvf(
        (*device).wined3d_device,
        fvf,
        declaration as *mut core::ffi::c_void,
        core::ptr::addr_of!(d3d8_vertexdeclaration_wined3d_parent_ops),
        core::ptr::addr_of_mut!((*declaration).wined3d_vertex_declaration),
    );
    if hr < 0 {
        WARN!("Failed to create wined3d vertex declaration, hr %#lx.\n", hr);
        if hr == E_INVALIDARG {
            hr = E_FAIL;
        }
        return hr;
    }

    return D3D_OK;
}
