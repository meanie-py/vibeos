use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_19 {
    pub texture_state: wined3d_texture_stage_state,
    pub sampler_state: wined3d_sampler_state,
}

#[no_mangle]
pub static mut d3d8_null_wined3d_parent_ops: wined3d_parent_ops = unsafe {
    wined3d_parent_ops {
        wined3d_object_destroyed: Some(
            d3d8_null_wined3d_object_destroyed,
        ),
    }
};

static mut tss_lookup: [tss_lookup; 29] = [
    tss_lookup {
        sampler_state: (FALSE as i32),
        u: tss_lookup_u {
            texture_state: WINED3D_TSS_INVALID,
        },
    },
    tss_lookup {
        sampler_state: (FALSE as i32),
        u: tss_lookup_u {
            texture_state: WINED3D_TSS_COLOR_OP,
        },
    },
    tss_lookup {
        sampler_state: (FALSE as i32),
        u: tss_lookup_u {
            texture_state: WINED3D_TSS_COLOR_ARG1,
        },
    },
    tss_lookup {
        sampler_state: (FALSE as i32),
        u: tss_lookup_u {
            texture_state: WINED3D_TSS_COLOR_ARG2,
        },
    },
    tss_lookup {
        sampler_state: (FALSE as i32),
        u: tss_lookup_u {
            texture_state: WINED3D_TSS_ALPHA_OP,
        },
    },
    tss_lookup {
        sampler_state: (FALSE as i32),
        u: tss_lookup_u {
            texture_state: WINED3D_TSS_ALPHA_ARG1,
        },
    },
    tss_lookup {
        sampler_state: (FALSE as i32),
        u: tss_lookup_u {
            texture_state: WINED3D_TSS_ALPHA_ARG2,
        },
    },
    tss_lookup {
        sampler_state: (FALSE as i32),
        u: tss_lookup_u {
            texture_state: WINED3D_TSS_BUMPENV_MAT00,
        },
    },
    tss_lookup {
        sampler_state: (FALSE as i32),
        u: tss_lookup_u {
            texture_state: WINED3D_TSS_BUMPENV_MAT01,
        },
    },
    tss_lookup {
        sampler_state: (FALSE as i32),
        u: tss_lookup_u {
            texture_state: WINED3D_TSS_BUMPENV_MAT10,
        },
    },
    tss_lookup {
        sampler_state: (FALSE as i32),
        u: tss_lookup_u {
            texture_state: WINED3D_TSS_BUMPENV_MAT11,
        },
    },
    tss_lookup {
        sampler_state: (FALSE as i32),
        u: tss_lookup_u {
            texture_state: WINED3D_TSS_TEXCOORD_INDEX,
        },
    },
    tss_lookup {
        sampler_state: (FALSE as i32),
        u: tss_lookup_u {
            texture_state: WINED3D_TSS_INVALID,
        },
    },
    tss_lookup {
        sampler_state: (TRUE as i32),
        u: tss_lookup_u {
            sampler_state: WINED3D_SAMP_ADDRESS_U,
        },
    },
    tss_lookup {
        sampler_state: (TRUE as i32),
        u: tss_lookup_u {
            sampler_state: WINED3D_SAMP_ADDRESS_V,
        },
    },
    tss_lookup {
        sampler_state: (TRUE as i32),
        u: tss_lookup_u {
            sampler_state: WINED3D_SAMP_BORDER_COLOR,
        },
    },
    tss_lookup {
        sampler_state: (TRUE as i32),
        u: tss_lookup_u {
            sampler_state: WINED3D_SAMP_MAG_FILTER,
        },
    },
    tss_lookup {
        sampler_state: (TRUE as i32),
        u: tss_lookup_u {
            sampler_state: WINED3D_SAMP_MIN_FILTER,
        },
    },
    tss_lookup {
        sampler_state: (TRUE as i32),
        u: tss_lookup_u {
            sampler_state: WINED3D_SAMP_MIP_FILTER,
        },
    },
    tss_lookup {
        sampler_state: (TRUE as i32),
        u: tss_lookup_u {
            sampler_state: WINED3D_SAMP_MIPMAP_LOD_BIAS,
        },
    },
    tss_lookup {
        sampler_state: (TRUE as i32),
        u: tss_lookup_u {
            sampler_state: WINED3D_SAMP_MAX_MIP_LEVEL,
        },
    },
    tss_lookup {
        sampler_state: (TRUE as i32),
        u: tss_lookup_u {
            sampler_state: WINED3D_SAMP_MAX_ANISOTROPY,
        },
    },
    tss_lookup {
        sampler_state: (FALSE as i32),
        u: tss_lookup_u {
            texture_state: WINED3D_TSS_BUMPENV_LSCALE,
        },
    },
    tss_lookup {
        sampler_state: (FALSE as i32),
        u: tss_lookup_u {
            texture_state: WINED3D_TSS_BUMPENV_LOFFSET,
        },
    },
    tss_lookup {
        sampler_state: (FALSE as i32),
        u: tss_lookup_u {
            texture_state: WINED3D_TSS_TEXTURE_TRANSFORM_FLAGS,
        },
    },
    tss_lookup {
        sampler_state: (TRUE as i32),
        u: tss_lookup_u {
            sampler_state: WINED3D_SAMP_ADDRESS_W,
        },
    },
    tss_lookup {
        sampler_state: (FALSE as i32),
        u: tss_lookup_u {
            texture_state: WINED3D_TSS_COLOR_ARG0,
        },
    },
    tss_lookup {
        sampler_state: (FALSE as i32),
        u: tss_lookup_u {
            texture_state: WINED3D_TSS_ALPHA_ARG0,
        },
    },
    tss_lookup {
        sampler_state: (FALSE as i32),
        u: tss_lookup_u {
            texture_state: WINED3D_TSS_RESULT_ARG,
        },
    },
];

static mut d3d8_wined3d_device_parent_ops: wined3d_device_parent_ops = unsafe {
    wined3d_device_parent_ops {
        wined3d_device_created: Some(
            device_parent_wined3d_device_created
                as unsafe extern "C" fn(*mut wined3d_device_parent, *mut wined3d_device) -> (),
        ),
        mode_changed: Some(
            device_parent_mode_changed as unsafe extern "C" fn(*mut wined3d_device_parent) -> (),
        ),
        activate: Some(
            device_parent_activate as unsafe extern "C" fn(*mut wined3d_device_parent, BOOL) -> (),
        ),
        texture_sub_resource_created: Some(
            device_parent_texture_sub_resource_created
                as unsafe extern "C" fn(
                    *mut wined3d_device_parent,
                    wined3d_resource_type,
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    *mut *mut ::core::ffi::c_void,
                    *mut *const wined3d_parent_ops,
                ) -> HRESULT,
        ),
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(d3d8): debug-channel macro, no declaration emitted.

// Dangling "static HRESULT WINAPI" fragment in the C preamble has no declarator; nothing emitted.

#[repr(C)]
pub union tss_lookup_u {
    pub texture_state: wined3d_texture_stage_state,
    pub sampler_state: wined3d_sampler_state,
}

#[repr(C)]
pub struct tss_lookup {
    pub sampler_state: BOOL,
    pub u: tss_lookup_u,
}

// static const struct IDirect3DDevice8Vtbl d3d8_device_vtbl: COM vtable, skipped (generated by vtbl_gen.py).

static mut DRAW_INDEXED_PRIMITIVE_UNUSED_GUARD: u8 = 0;
unsafe extern "system" fn d3d8_null_wined3d_object_destroyed(parent: *mut core::ffi::c_void) {}
#[no_mangle]
pub unsafe extern "C" fn d3dformat_from_wined3dformat(mut format: wined3d_format_id) -> D3DFORMAT {
    let mut c: *mut BYTE = core::ptr::addr_of_mut!(format) as *mut BYTE;
    if isprint(*c.offset(0 as isize) as ::core::ffi::c_int) != 0
        && isprint(*c.offset(1 as isize) as ::core::ffi::c_int) != 0
        && isprint(*c.offset(2 as isize) as ::core::ffi::c_int) != 0
        && isprint(*c.offset(3 as isize) as ::core::ffi::c_int) != 0
    {
        return format as D3DFORMAT;
    }
    match format as ::core::ffi::c_uint {
        WINED3DFMT_UNKNOWN => return D3DFMT_UNKNOWN,
        WINED3DFMT_B8G8R8_UNORM => return D3DFMT_R8G8B8,
        WINED3DFMT_B8G8R8A8_UNORM => return D3DFMT_A8R8G8B8,
        WINED3DFMT_B8G8R8X8_UNORM => return D3DFMT_X8R8G8B8,
        WINED3DFMT_B5G6R5_UNORM => return D3DFMT_R5G6B5,
        WINED3DFMT_B5G5R5X1_UNORM => return D3DFMT_X1R5G5B5,
        WINED3DFMT_B5G5R5A1_UNORM => return D3DFMT_A1R5G5B5,
        WINED3DFMT_B4G4R4A4_UNORM => return D3DFMT_A4R4G4B4,
        WINED3DFMT_B2G3R3_UNORM => return D3DFMT_R3G3B2,
        WINED3DFMT_A8_UNORM => return D3DFMT_A8,
        WINED3DFMT_B2G3R3A8_UNORM => return D3DFMT_A8R3G3B2,
        WINED3DFMT_B4G4R4X4_UNORM => return D3DFMT_X4R4G4B4,
        WINED3DFMT_R10G10B10A2_UNORM => return D3DFMT_A2B10G10R10,
        WINED3DFMT_R16G16_UNORM => return D3DFMT_G16R16,
        WINED3DFMT_P8_UINT_A8_UNORM => return D3DFMT_A8P8,
        WINED3DFMT_P8_UINT => return D3DFMT_P8,
        WINED3DFMT_L8_UNORM => return D3DFMT_L8,
        WINED3DFMT_L8A8_UNORM => return D3DFMT_A8L8,
        WINED3DFMT_L4A4_UNORM => return D3DFMT_A4L4,
        WINED3DFMT_R8G8_SNORM => return D3DFMT_V8U8,
        WINED3DFMT_R5G5_SNORM_L6_UNORM => return D3DFMT_L6V5U5,
        WINED3DFMT_R8G8_SNORM_L8X8_UNORM => return D3DFMT_X8L8V8U8,
        WINED3DFMT_R8G8B8A8_SNORM => return D3DFMT_Q8W8V8U8,
        WINED3DFMT_R16G16_SNORM => return D3DFMT_V16U16,
        WINED3DFMT_R10G11B11_SNORM => return D3DFMT_W11V11U10,
        WINED3DFMT_R10G10B10_SNORM_A2_UNORM => return D3DFMT_A2W10V10U10,
        WINED3DFMT_D16_LOCKABLE => return D3DFMT_D16_LOCKABLE,
        WINED3DFMT_D32_UNORM => return D3DFMT_D32,
        WINED3DFMT_S1_UINT_D15_UNORM => return D3DFMT_D15S1,
        WINED3DFMT_D24_UNORM_S8_UINT => return D3DFMT_D24S8,
        WINED3DFMT_X8D24_UNORM => return D3DFMT_D24X8,
        WINED3DFMT_S4X4_UINT_D24_UNORM => return D3DFMT_D24X4S4,
        WINED3DFMT_D16_UNORM => return D3DFMT_D16,
        WINED3DFMT_R16_UINT => return D3DFMT_INDEX16,
        WINED3DFMT_R32_UINT => return D3DFMT_INDEX32,
        _ => {
            WINE_FIXME!("Unhandled wined3d format %#x.\n", format as ::core::ffi::c_uint);
            return D3DFMT_UNKNOWN;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3dformat_from_d3dformat(mut format: D3DFORMAT) -> wined3d_format_id {
    let mut c: *mut BYTE = core::ptr::addr_of_mut!(format) as *mut BYTE;
    if isprint(*c.offset(0 as isize) as ::core::ffi::c_int) != 0
        && isprint(*c.offset(1 as isize) as ::core::ffi::c_int) != 0
        && isprint(*c.offset(2 as isize) as ::core::ffi::c_int) != 0
        && isprint(*c.offset(3 as isize) as ::core::ffi::c_int) != 0
    {
        return format as wined3d_format_id;
    }
    match format as ::core::ffi::c_uint {
        D3DFMT_UNKNOWN => return WINED3DFMT_UNKNOWN,
        D3DFMT_R8G8B8 => return WINED3DFMT_B8G8R8_UNORM,
        D3DFMT_A8R8G8B8 => return WINED3DFMT_B8G8R8A8_UNORM,
        D3DFMT_X8R8G8B8 => return WINED3DFMT_B8G8R8X8_UNORM,
        D3DFMT_R5G6B5 => return WINED3DFMT_B5G6R5_UNORM,
        D3DFMT_X1R5G5B5 => return WINED3DFMT_B5G5R5X1_UNORM,
        D3DFMT_A1R5G5B5 => return WINED3DFMT_B5G5R5A1_UNORM,
        D3DFMT_A4R4G4B4 => return WINED3DFMT_B4G4R4A4_UNORM,
        D3DFMT_R3G3B2 => return WINED3DFMT_B2G3R3_UNORM,
        D3DFMT_A8 => return WINED3DFMT_A8_UNORM,
        D3DFMT_A8R3G3B2 => return WINED3DFMT_B2G3R3A8_UNORM,
        D3DFMT_X4R4G4B4 => return WINED3DFMT_B4G4R4X4_UNORM,
        D3DFMT_A2B10G10R10 => return WINED3DFMT_R10G10B10A2_UNORM,
        D3DFMT_G16R16 => return WINED3DFMT_R16G16_UNORM,
        D3DFMT_A8P8 => return WINED3DFMT_P8_UINT_A8_UNORM,
        D3DFMT_P8 => return WINED3DFMT_P8_UINT,
        D3DFMT_L8 => return WINED3DFMT_L8_UNORM,
        D3DFMT_A8L8 => return WINED3DFMT_L8A8_UNORM,
        D3DFMT_A4L4 => return WINED3DFMT_L4A4_UNORM,
        D3DFMT_V8U8 => return WINED3DFMT_R8G8_SNORM,
        D3DFMT_L6V5U5 => return WINED3DFMT_R5G5_SNORM_L6_UNORM,
        D3DFMT_X8L8V8U8 => return WINED3DFMT_R8G8_SNORM_L8X8_UNORM,
        D3DFMT_Q8W8V8U8 => return WINED3DFMT_R8G8B8A8_SNORM,
        D3DFMT_V16U16 => return WINED3DFMT_R16G16_SNORM,
        D3DFMT_W11V11U10 => return WINED3DFMT_R10G11B11_SNORM,
        D3DFMT_A2W10V10U10 => return WINED3DFMT_R10G10B10_SNORM_A2_UNORM,
        D3DFMT_D16_LOCKABLE => return WINED3DFMT_D16_LOCKABLE,
        D3DFMT_D32 => return WINED3DFMT_D32_UNORM,
        D3DFMT_D15S1 => return WINED3DFMT_S1_UINT_D15_UNORM,
        D3DFMT_D24S8 => return WINED3DFMT_D24_UNORM_S8_UINT,
        D3DFMT_D24X8 => return WINED3DFMT_X8D24_UNORM,
        D3DFMT_D24X4S4 => return WINED3DFMT_S4X4_UINT_D24_UNORM,
        D3DFMT_D16 => return WINED3DFMT_D16_UNORM,
        D3DFMT_INDEX16 => return WINED3DFMT_R16_UINT,
        D3DFMT_INDEX32 => return WINED3DFMT_R32_UINT,
        _ => {
            WINE_FIXME!("Unhandled D3DFORMAT %#x.\n", format as ::core::ffi::c_uint);
            return WINED3DFMT_UNKNOWN;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn wined3dmapflags_from_d3dmapflags(
    mut flags: ::core::ffi::c_uint,
    mut usage: ::core::ffi::c_uint,
) -> ::core::ffi::c_uint {
    static handled: ::core::ffi::c_uint = (D3DLOCK_NOSYSLOCK
        | D3DLOCK_NOOVERWRITE
        | D3DLOCK_DISCARD
        | D3DLOCK_NO_DIRTY_UPDATE) as ::core::ffi::c_uint;
    let mut wined3d_flags: ::core::ffi::c_uint = 0;
    wined3d_flags = flags & handled;
    if (!usage & (D3DUSAGE_WRITEONLY as ::core::ffi::c_uint)) != 0
        && (flags & ((D3DLOCK_NOOVERWRITE | D3DLOCK_DISCARD) as ::core::ffi::c_uint))
            == 0
    {
        wined3d_flags |= WINED3D_MAP_READ as ::core::ffi::c_uint;
    }
    if (flags & (D3DLOCK_READONLY as ::core::ffi::c_uint)) == 0 {
        wined3d_flags |= WINED3D_MAP_WRITE as ::core::ffi::c_uint;
    }
    if (wined3d_flags & ((WINED3D_MAP_READ | WINED3D_MAP_WRITE) as ::core::ffi::c_uint))
        == 0
    {
        wined3d_flags |= WINED3D_MAP_WRITE as ::core::ffi::c_uint;
    }
    flags &= !(handled | (D3DLOCK_READONLY as ::core::ffi::c_uint));

    if flags != 0 {
        WINE_FIXME!("Unhandled flags %#x.\n", flags);
    }
    return wined3d_flags;
}

unsafe extern "C" fn vertex_count_from_primitive_count(primitive_type: D3DPRIMITIVETYPE, primitive_count: UINT) -> UINT {
    match primitive_type as ::core::ffi::c_uint {
        D3DPT_POINTLIST => {
            return primitive_count;
        }

        D3DPT_LINELIST => {
            return primitive_count.wrapping_mul(2 as UINT);
        }

        D3DPT_LINESTRIP => {
            return primitive_count.wrapping_add(1 as UINT);
        }

        D3DPT_TRIANGLELIST => {
            return primitive_count.wrapping_mul(3 as UINT);
        }

        D3DPT_TRIANGLESTRIP | D3DPT_TRIANGLEFAN => {
            return primitive_count.wrapping_add(2 as UINT);
        }

        _ => {
            FIXME!("Unhandled primitive type %#x.\n", primitive_type);
            return 0 as UINT;
        }
    }
}

unsafe extern "C" fn d3dswapeffect_from_wined3dswapeffect(effect: wined3d_swap_effect) -> D3DSWAPEFFECT {
    match effect {
        WINED3D_SWAP_EFFECT_DISCARD => return D3DSWAPEFFECT_DISCARD,
        WINED3D_SWAP_EFFECT_SEQUENTIAL => return D3DSWAPEFFECT_FLIP,
        WINED3D_SWAP_EFFECT_COPY => return D3DSWAPEFFECT_COPY,
        WINED3D_SWAP_EFFECT_COPY_VSYNC => return D3DSWAPEFFECT_COPY_VSYNC,
        _ => {
            WINE_FIXME!("Unhandled swap effect %#x.\n", effect as ::core::ffi::c_uint);
            return D3DSWAPEFFECT_FLIP;
        }
    }
}

unsafe extern "C" fn present_parameters_from_wined3d_swapchain_desc(
    present_parameters: *mut D3DPRESENT_PARAMETERS,
    swapchain_desc: *const wined3d_swapchain_desc,
    presentation_interval: DWORD,
) {
    (*present_parameters).BackBufferWidth = (*swapchain_desc).backbuffer_width as UINT;
    (*present_parameters).BackBufferHeight = (*swapchain_desc).backbuffer_height as UINT;
    (*present_parameters).BackBufferFormat =
        d3dformat_from_wined3dformat((*swapchain_desc).backbuffer_format);
    (*present_parameters).BackBufferCount = (*swapchain_desc).backbuffer_count as UINT;
    (*present_parameters).MultiSampleType =
        d3dmultisample_type_from_wined3d((*swapchain_desc).multisample_type);
    (*present_parameters).SwapEffect =
        d3dswapeffect_from_wined3dswapeffect((*swapchain_desc).swap_effect);
    (*present_parameters).hDeviceWindow = (*swapchain_desc).device_window;
    (*present_parameters).Windowed = (*swapchain_desc).windowed;
    (*present_parameters).EnableAutoDepthStencil = (*swapchain_desc).enable_auto_depth_stencil;
    (*present_parameters).AutoDepthStencilFormat =
        d3dformat_from_wined3dformat((*swapchain_desc).auto_depth_stencil_format);
    (*present_parameters).Flags =
        ((*swapchain_desc).flags & D3DPRESENTFLAGS_MASK) as DWORD;
    (*present_parameters).FullScreen_RefreshRateInHz = (*swapchain_desc).refresh_rate as UINT;
    (*present_parameters).FullScreen_PresentationInterval = presentation_interval as UINT;
}

unsafe extern "C" fn wined3dswapeffect_from_d3dswapeffect(effect: D3DSWAPEFFECT) -> wined3d_swap_effect {
    match effect {
        D3DSWAPEFFECT_DISCARD => {
            return WINED3D_SWAP_EFFECT_DISCARD;
        }
        D3DSWAPEFFECT_FLIP => {
            return WINED3D_SWAP_EFFECT_SEQUENTIAL;
        }
        D3DSWAPEFFECT_COPY => {
            return WINED3D_SWAP_EFFECT_COPY;
        }
        D3DSWAPEFFECT_COPY_VSYNC => {
            return WINED3D_SWAP_EFFECT_COPY_VSYNC;
        }
        _ => {
            FIXME!("Unhandled swap effect %#x.\n", effect);
            return WINED3D_SWAP_EFFECT_SEQUENTIAL;
        }
    }
}

unsafe extern "C" fn wined3dswapinterval_from_d3d(mut interval: DWORD) -> wined3d_swap_interval {
    match interval {
        D3DPRESENT_INTERVAL_IMMEDIATE => return WINED3D_SWAP_INTERVAL_IMMEDIATE,
        D3DPRESENT_INTERVAL_ONE => return WINED3D_SWAP_INTERVAL_ONE,
        D3DPRESENT_INTERVAL_TWO => return WINED3D_SWAP_INTERVAL_TWO,
        D3DPRESENT_INTERVAL_THREE => return WINED3D_SWAP_INTERVAL_THREE,
        D3DPRESENT_INTERVAL_FOUR => return WINED3D_SWAP_INTERVAL_FOUR,
        D3DPRESENT_INTERVAL_DEFAULT => {}
        _ => {
            FIXME!("Unhandled presentation interval %#lx.\n", interval);
            // C falls through to the case below, which does nothing.
        }
    }
    return WINED3D_SWAP_INTERVAL_DEFAULT;
}

unsafe extern "C" fn wined3d_multisample_type_from_d3d(type_: D3DMULTISAMPLE_TYPE) -> wined3d_multisample_type {
    return type_ as wined3d_multisample_type;
}

unsafe extern "C" fn wined3d_swapchain_desc_from_d3d8(
    mut swapchain_desc: *mut wined3d_swapchain_desc,
    mut output: *mut wined3d_output,
    mut present_parameters: *const D3DPRESENT_PARAMETERS,
) -> BOOL {
    if (*present_parameters).SwapEffect == 0
        || (*present_parameters).SwapEffect > D3DSWAPEFFECT_COPY_VSYNC
    {
        WARN!(
            "Invalid swap effect %u passed.\n",
            (*present_parameters).SwapEffect
        );
        return (FALSE) as BOOL;
    }
    if (*present_parameters).BackBufferCount > 3
        || (((*present_parameters).SwapEffect == D3DSWAPEFFECT_COPY
            || (*present_parameters).SwapEffect == D3DSWAPEFFECT_COPY_VSYNC)
            && (*present_parameters).BackBufferCount > 1)
    {
        WARN!(
            "Invalid backbuffer count %u.\n",
            (*present_parameters).BackBufferCount
        );
        return (FALSE) as BOOL;
    }
    match (*present_parameters).FullScreen_PresentationInterval {
        D3DPRESENT_INTERVAL_DEFAULT
        | D3DPRESENT_INTERVAL_ONE
        | D3DPRESENT_INTERVAL_TWO
        | D3DPRESENT_INTERVAL_THREE
        | D3DPRESENT_INTERVAL_FOUR
        | D3DPRESENT_INTERVAL_IMMEDIATE => {}
        _ => {
            WARN!(
                "Invalid presentation interval %#x.\n",
                (*present_parameters).FullScreen_PresentationInterval
            );
            return (FALSE) as BOOL;
        }
    }

    (*swapchain_desc).output = output;
    (*swapchain_desc).backbuffer_width = (*present_parameters).BackBufferWidth;
    (*swapchain_desc).backbuffer_height = (*present_parameters).BackBufferHeight;
    (*swapchain_desc).backbuffer_format =
        wined3dformat_from_d3dformat((*present_parameters).BackBufferFormat);
    (*swapchain_desc).backbuffer_count = max!(
        1,
        (*present_parameters).BackBufferCount,
    );
    (*swapchain_desc).backbuffer_bind_flags = WINED3D_BIND_RENDER_TARGET;
    (*swapchain_desc).multisample_type =
        wined3d_multisample_type_from_d3d((*present_parameters).MultiSampleType);
    (*swapchain_desc).multisample_quality = 0; /* d3d9 only */
    (*swapchain_desc).swap_effect =
        wined3dswapeffect_from_d3dswapeffect((*present_parameters).SwapEffect);
    (*swapchain_desc).device_window = (*present_parameters).hDeviceWindow;
    (*swapchain_desc).windowed = (*present_parameters).Windowed;
    (*swapchain_desc).enable_auto_depth_stencil = (*present_parameters).EnableAutoDepthStencil;
    (*swapchain_desc).auto_depth_stencil_format =
        wined3dformat_from_d3dformat((*present_parameters).AutoDepthStencilFormat);
    (*swapchain_desc).flags = ((*present_parameters).Flags & D3DPRESENTFLAGS_MASK)
        | WINED3D_SWAPCHAIN_ALLOW_MODE_SWITCH;
    (*swapchain_desc).refresh_rate = (*present_parameters).FullScreen_RefreshRateInHz;
    (*swapchain_desc).auto_restore_display_mode = (TRUE as i32);

    if (*present_parameters).Flags & !D3DPRESENTFLAGS_MASK != 0 {
        FIXME!(
            "Unhandled flags %#lx.\n",
            (*present_parameters).Flags & !D3DPRESENTFLAGS_MASK
        );
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn d3dcaps_from_wined3dcaps(
    caps: *mut D3DCAPS8,
    wined3d_caps: *const wined3d_caps,
    adapter_ordinal: ::core::ffi::c_uint,
) {
    (*caps).DeviceType = (*wined3d_caps).DeviceType as D3DDEVTYPE;
    (*caps).AdapterOrdinal = adapter_ordinal as UINT;
    (*caps).Caps = (*wined3d_caps).Caps;
    (*caps).Caps2 = (*wined3d_caps).Caps2;
    (*caps).Caps3 = (*wined3d_caps).Caps3;
    (*caps).PresentationIntervals = (D3DPRESENT_INTERVAL_IMMEDIATE | D3DPRESENT_INTERVAL_ONE) as DWORD;
    (*caps).CursorCaps = (*wined3d_caps).CursorCaps;
    (*caps).DevCaps = (*wined3d_caps).DevCaps;
    (*caps).PrimitiveMiscCaps = (*wined3d_caps).PrimitiveMiscCaps;
    (*caps).RasterCaps = (*wined3d_caps).RasterCaps;
    (*caps).ZCmpCaps = (*wined3d_caps).ZCmpCaps;
    (*caps).SrcBlendCaps = (*wined3d_caps).SrcBlendCaps;
    (*caps).DestBlendCaps = (*wined3d_caps).DestBlendCaps;
    (*caps).AlphaCmpCaps = (*wined3d_caps).AlphaCmpCaps;
    (*caps).ShadeCaps = (*wined3d_caps).ShadeCaps;
    (*caps).TextureCaps = (*wined3d_caps).TextureCaps;
    (*caps).TextureFilterCaps = (*wined3d_caps).TextureFilterCaps;
    (*caps).CubeTextureFilterCaps = (*wined3d_caps).CubeTextureFilterCaps;
    (*caps).VolumeTextureFilterCaps = (*wined3d_caps).VolumeTextureFilterCaps;
    (*caps).TextureAddressCaps = (*wined3d_caps).TextureAddressCaps;
    (*caps).VolumeTextureAddressCaps = (*wined3d_caps).VolumeTextureAddressCaps;
    (*caps).LineCaps = (*wined3d_caps).LineCaps;
    (*caps).MaxTextureWidth = (*wined3d_caps).MaxTextureWidth;
    (*caps).MaxTextureHeight = (*wined3d_caps).MaxTextureHeight;
    (*caps).MaxVolumeExtent = (*wined3d_caps).MaxVolumeExtent;
    (*caps).MaxTextureRepeat = (*wined3d_caps).MaxTextureRepeat;
    (*caps).MaxTextureAspectRatio = (*wined3d_caps).MaxTextureAspectRatio;
    (*caps).MaxAnisotropy = (*wined3d_caps).MaxAnisotropy;
    (*caps).MaxVertexW = (*wined3d_caps).MaxVertexW;
    (*caps).GuardBandLeft = (*wined3d_caps).GuardBandLeft;
    (*caps).GuardBandTop = (*wined3d_caps).GuardBandTop;
    (*caps).GuardBandRight = (*wined3d_caps).GuardBandRight;
    (*caps).GuardBandBottom = (*wined3d_caps).GuardBandBottom;
    (*caps).ExtentsAdjust = (*wined3d_caps).ExtentsAdjust;
    (*caps).StencilCaps = (*wined3d_caps).StencilCaps;
    (*caps).FVFCaps = (*wined3d_caps).FVFCaps;
    (*caps).TextureOpCaps = (*wined3d_caps).TextureOpCaps;
    (*caps).MaxTextureBlendStages = (*wined3d_caps).MaxTextureBlendStages;
    (*caps).MaxSimultaneousTextures = (*wined3d_caps).MaxSimultaneousTextures;
    (*caps).VertexProcessingCaps = (*wined3d_caps).VertexProcessingCaps;
    (*caps).MaxActiveLights = (*wined3d_caps).MaxActiveLights;
    (*caps).MaxUserClipPlanes = (*wined3d_caps).MaxUserClipPlanes;
    (*caps).MaxVertexBlendMatrices = (*wined3d_caps).MaxVertexBlendMatrices;
    (*caps).MaxVertexBlendMatrixIndex = (*wined3d_caps).MaxVertexBlendMatrixIndex;
    (*caps).MaxPointSize = (*wined3d_caps).MaxPointSize;
    (*caps).MaxPrimitiveCount = (*wined3d_caps).MaxPrimitiveCount;
    (*caps).MaxVertexIndex = (*wined3d_caps).MaxVertexIndex;
    (*caps).MaxStreams = (*wined3d_caps).MaxStreams;
    (*caps).MaxStreamStride = (*wined3d_caps).MaxStreamStride;
    (*caps).VertexShaderVersion = (*wined3d_caps).VertexShaderVersion;
    (*caps).MaxVertexShaderConst = (*wined3d_caps).MaxVertexShaderConst;
    (*caps).PixelShaderVersion = (*wined3d_caps).PixelShaderVersion;
    (*caps).MaxPixelShaderValue = (*wined3d_caps).PixelShader1xMaxValue;

    (*caps).Caps2 &= (D3DCAPS2_CANCALIBRATEGAMMA | D3DCAPS2_CANRENDERWINDOWED
        | D3DCAPS2_CANMANAGERESOURCE | D3DCAPS2_DYNAMICTEXTURES | D3DCAPS2_FULLSCREENGAMMA
        | D3DCAPS2_NO2DDURING3DSCENE | D3DCAPS2_RESERVED) as DWORD;
    (*caps).Caps3 &= (D3DCAPS3_ALPHA_FULLSCREEN_FLIP_OR_DISCARD | D3DCAPS3_RESERVED) as DWORD;
    (*caps).PrimitiveMiscCaps &= (D3DPMISCCAPS_MASKZ | D3DPMISCCAPS_LINEPATTERNREP
        | D3DPMISCCAPS_CULLNONE | D3DPMISCCAPS_CULLCW | D3DPMISCCAPS_CULLCCW
        | (D3DPMISCCAPS_COLORWRITEENABLE as u32) | (D3DPMISCCAPS_CLIPPLANESCALEDPOINTS as u32)
        | (D3DPMISCCAPS_CLIPTLVERTS as u32) | (D3DPMISCCAPS_TSSARGTEMP as u32) | (D3DPMISCCAPS_BLENDOP as u32)
        | (D3DPMISCCAPS_NULLREFERENCE as u32)) as DWORD;
    (*caps).RasterCaps &= (D3DPRASTERCAPS_DITHER | D3DPRASTERCAPS_PAT | D3DPRASTERCAPS_ZTEST
        | D3DPRASTERCAPS_FOGVERTEX | D3DPRASTERCAPS_FOGTABLE | D3DPRASTERCAPS_ANTIALIASEDGES
        | D3DPRASTERCAPS_MIPMAPLODBIAS | D3DPRASTERCAPS_ZBUFFERLESSHSR
        | D3DPRASTERCAPS_FOGRANGE | D3DPRASTERCAPS_ANISOTROPY | D3DPRASTERCAPS_WBUFFER
        | D3DPRASTERCAPS_WFOG | D3DPRASTERCAPS_ZFOG | D3DPRASTERCAPS_COLORPERSPECTIVE
        | D3DPRASTERCAPS_STRETCHBLTMULTISAMPLE | WINED3DPRASTERCAPS_DEPTHBIAS) as DWORD;
    if ((*caps).RasterCaps & (WINED3DPRASTERCAPS_DEPTHBIAS as DWORD)) != 0 {
        (*caps).RasterCaps = (((*caps).RasterCaps | (D3DPRASTERCAPS_ZBIAS as DWORD))
            & !(WINED3DPRASTERCAPS_DEPTHBIAS as DWORD)) as DWORD;
    }
    (*caps).SrcBlendCaps &= (D3DPBLENDCAPS_ZERO | D3DPBLENDCAPS_ONE | D3DPBLENDCAPS_SRCCOLOR
        | D3DPBLENDCAPS_INVSRCCOLOR | D3DPBLENDCAPS_SRCALPHA | D3DPBLENDCAPS_INVSRCALPHA
        | D3DPBLENDCAPS_DESTALPHA | D3DPBLENDCAPS_INVDESTALPHA | D3DPBLENDCAPS_DESTCOLOR
        | D3DPBLENDCAPS_INVDESTCOLOR | D3DPBLENDCAPS_SRCALPHASAT | D3DPBLENDCAPS_BOTHSRCALPHA
        | D3DPBLENDCAPS_BOTHINVSRCALPHA) as DWORD;
    (*caps).DestBlendCaps &= (D3DPBLENDCAPS_ZERO | D3DPBLENDCAPS_ONE | D3DPBLENDCAPS_SRCCOLOR
        | D3DPBLENDCAPS_INVSRCCOLOR | D3DPBLENDCAPS_SRCALPHA | D3DPBLENDCAPS_INVSRCALPHA
        | D3DPBLENDCAPS_DESTALPHA | D3DPBLENDCAPS_INVDESTALPHA | D3DPBLENDCAPS_DESTCOLOR
        | D3DPBLENDCAPS_INVDESTCOLOR | D3DPBLENDCAPS_SRCALPHASAT | D3DPBLENDCAPS_BOTHSRCALPHA
        | D3DPBLENDCAPS_BOTHINVSRCALPHA) as DWORD;
    (*caps).TextureCaps &= (D3DPTEXTURECAPS_PERSPECTIVE | D3DPTEXTURECAPS_POW2 | D3DPTEXTURECAPS_ALPHA
        | D3DPTEXTURECAPS_SQUAREONLY | D3DPTEXTURECAPS_TEXREPEATNOTSCALEDBYSIZE
        | D3DPTEXTURECAPS_ALPHAPALETTE | D3DPTEXTURECAPS_NONPOW2CONDITIONAL
        | D3DPTEXTURECAPS_PROJECTED | D3DPTEXTURECAPS_CUBEMAP | D3DPTEXTURECAPS_VOLUMEMAP
        | D3DPTEXTURECAPS_MIPMAP | D3DPTEXTURECAPS_MIPVOLUMEMAP | D3DPTEXTURECAPS_MIPCUBEMAP
        | D3DPTEXTURECAPS_CUBEMAP_POW2 | D3DPTEXTURECAPS_VOLUMEMAP_POW2) as DWORD;
    (*caps).TextureFilterCaps &= (D3DPTFILTERCAPS_MINFPOINT | D3DPTFILTERCAPS_MINFLINEAR
        | D3DPTFILTERCAPS_MINFANISOTROPIC | D3DPTFILTERCAPS_MIPFPOINT
        | D3DPTFILTERCAPS_MIPFLINEAR | D3DPTFILTERCAPS_MAGFPOINT | D3DPTFILTERCAPS_MAGFLINEAR
        | D3DPTFILTERCAPS_MAGFANISOTROPIC | D3DPTFILTERCAPS_MAGFAFLATCUBIC
        | D3DPTFILTERCAPS_MAGFGAUSSIANCUBIC) as DWORD;
    (*caps).CubeTextureFilterCaps &= (D3DPTFILTERCAPS_MINFPOINT | D3DPTFILTERCAPS_MINFLINEAR
        | D3DPTFILTERCAPS_MINFANISOTROPIC | D3DPTFILTERCAPS_MIPFPOINT
        | D3DPTFILTERCAPS_MIPFLINEAR | D3DPTFILTERCAPS_MAGFPOINT | D3DPTFILTERCAPS_MAGFLINEAR
        | D3DPTFILTERCAPS_MAGFANISOTROPIC | D3DPTFILTERCAPS_MAGFAFLATCUBIC
        | D3DPTFILTERCAPS_MAGFGAUSSIANCUBIC) as DWORD;
    (*caps).VolumeTextureFilterCaps &= (D3DPTFILTERCAPS_MINFPOINT | D3DPTFILTERCAPS_MINFLINEAR
        | D3DPTFILTERCAPS_MINFANISOTROPIC | D3DPTFILTERCAPS_MIPFPOINT
        | D3DPTFILTERCAPS_MIPFLINEAR | D3DPTFILTERCAPS_MAGFPOINT | D3DPTFILTERCAPS_MAGFLINEAR
        | D3DPTFILTERCAPS_MAGFANISOTROPIC | D3DPTFILTERCAPS_MAGFAFLATCUBIC
        | D3DPTFILTERCAPS_MAGFGAUSSIANCUBIC) as DWORD;
    if ((*caps).LineCaps & (WINED3DLINECAPS_ANTIALIAS as DWORD)) != 0 {
        (*caps).RasterCaps |= D3DPRASTERCAPS_ANTIALIASEDGES as DWORD;
    }
    (*caps).LineCaps &= !(WINED3DLINECAPS_ANTIALIAS as DWORD);
    (*caps).StencilCaps &= !(WINED3DSTENCILCAPS_TWOSIDED as DWORD);
    (*caps).VertexProcessingCaps &= (D3DVTXPCAPS_TEXGEN | D3DVTXPCAPS_MATERIALSOURCE7
        | D3DVTXPCAPS_DIRECTIONALLIGHTS | D3DVTXPCAPS_POSITIONALLIGHTS | D3DVTXPCAPS_LOCALVIEWER
        | (D3DVTXPCAPS_TWEENING as u32) | (D3DVTXPCAPS_NO_VSDT_UBYTE4 as u32)) as DWORD;

    /* D3D8 doesn't support SM 2.0 or higher, so clamp to 1.x */
    if (*caps).PixelShaderVersion != 0 {
        (*caps).PixelShaderVersion = D3DPS_VERSION!(1, 4) as DWORD;
    } else {
        (*caps).PixelShaderVersion = D3DPS_VERSION!(0, 0) as DWORD;
    }
    if (*caps).VertexShaderVersion != 0 {
        (*caps).VertexShaderVersion = D3DVS_VERSION!(1, 1) as DWORD;
    } else {
        (*caps).VertexShaderVersion = D3DVS_VERSION!(0, 0) as DWORD;
    }
    (*caps).MaxVertexShaderConst = min!(D3D8_MAX_VERTEX_SHADER_CONSTANTF as DWORD, (*caps).MaxVertexShaderConst) as DWORD;
}

unsafe extern "C" fn wined3d_transform_state_from_d3d(state: D3DTRANSFORMSTATETYPE) -> wined3d_transform_state {
    return state as wined3d_transform_state;
}

unsafe extern "C" fn wined3d_render_state_from_d3d(mut state: D3DRENDERSTATETYPE) -> wined3d_render_state {
    match state {
        D3DRS_ZBIAS => {
            return WINED3D_RS_DEPTHBIAS;
        }
        D3DRS_EDGEANTIALIAS => {
            return WINED3D_RS_ANTIALIASEDLINEENABLE;
        }
        _ => {
            return state as wined3d_render_state;
        }
    };
}

unsafe extern "C" fn wined3d_primitive_type_from_d3d(type_: D3DPRIMITIVETYPE) -> wined3d_primitive_type {
    return type_ as wined3d_primitive_type;
}

unsafe extern "C" fn d3d8_allocate_handle(t: *mut d3d8_handle_table, object: *mut core::ffi::c_void, type_0: d3d8_handle_type) -> DWORD {
    let mut entry: *mut d3d8_handle_entry;
    if !(*t).free_entries.is_null() {
        let index: DWORD = (*t).free_entries.offset_from((*t).entries) as core::ffi::c_longlong as DWORD;
        entry = (*t).free_entries;
        if (*entry).type_ as core::ffi::c_uint != D3D8_HANDLE_FREE as core::ffi::c_uint {
            ERR!("Handle %lu(%p) is in the free list, but has type %#x.\n", index, entry, (*entry).type_ as core::ffi::c_uint);
            return (D3D8_INVALID_HANDLE) as DWORD;
        }
        (*t).free_entries = (*entry).object as *mut d3d8_handle_entry;
        (*entry).object = object;
        (*entry).type_ = type_0;

        return index;
    }

    if !((*t).entry_count < (*t).table_size) {
        let new_size: UINT = (*t).table_size.wrapping_add((*t).table_size >> 1);
        let mut new_entries: *mut d3d8_handle_entry;

        new_entries = realloc(
            (*t).entries as *mut core::ffi::c_void,
            (new_size as usize).wrapping_mul(core::mem::size_of::<d3d8_handle_entry>()),
        ) as *mut d3d8_handle_entry;
        if new_entries.is_null() {
            ERR!("Failed to grow the handle table.\n");
            return (D3D8_INVALID_HANDLE) as DWORD;
        }
        (*t).entries = new_entries;
        (*t).table_size = new_size;
    }

    entry = (*t).entries.offset((*t).entry_count as isize);
    (*entry).object = object;
    (*entry).type_ = type_0;
    let fresh15 = (*t).entry_count;
    (*t).entry_count = (*t).entry_count.wrapping_add(1);
    return fresh15;
}

unsafe extern "C" fn d3d8_free_handle(
    mut t: *mut d3d8_handle_table,
    mut handle: DWORD,
    mut type_0: d3d8_handle_type,
) -> *mut ::core::ffi::c_void {
    let mut entry: *mut d3d8_handle_entry = core::ptr::null_mut();
    let mut object: *mut ::core::ffi::c_void = core::ptr::null_mut();

    if handle == (D3D8_INVALID_HANDLE) as u32 || handle >= (*t).entry_count {
        WARN!("Invalid handle %lu passed.\n", handle);
        return core::ptr::null_mut();
    }

    entry = (*t).entries.add(handle as usize) as *mut d3d8_handle_entry;
    if (*entry).type_ as ::core::ffi::c_uint != type_0 as ::core::ffi::c_uint {
        WARN!(
            "Handle %lu(%p) is not of type %#x.\n",
            handle,
            entry,
            type_0 as ::core::ffi::c_uint
        );
        return core::ptr::null_mut();
    }

    object = (*entry).object;
    (*entry).object = (*t).free_entries as *mut ::core::ffi::c_void;
    (*entry).type_ = D3D8_HANDLE_FREE;
    (*t).free_entries = entry;

    return object;
}

unsafe extern "C" fn d3d8_get_object(t: *mut d3d8_handle_table, handle: DWORD, type_0: d3d8_handle_type) -> *mut core::ffi::c_void {
    let mut entry: *mut d3d8_handle_entry = core::ptr::null_mut();

    if handle == (D3D8_INVALID_HANDLE) as u32 || handle >= (*t).entry_count {
        WARN!("Invalid handle %lu passed.\n", handle);
        return core::ptr::null_mut();
    }

    entry = (*t).entries.offset(handle as isize);
    if (*entry).type_ != type_0 {
        WARN!("Handle %lu(%p) is not of type %#x.\n", handle, entry, type_0);
        return core::ptr::null_mut();
    }

    return (*entry).object;
}

unsafe extern "C" fn device_reset_viewport_state(device: *mut d3d8_device) {
    let mut vp: wined3d_viewport = core::mem::zeroed();
    let mut rect: RECT = core::mem::zeroed();

    wined3d_device_context_get_viewports(
        (*device).immediate_context,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(vp),
    );
    wined3d_stateblock_set_viewport((*device).state, core::ptr::addr_of_mut!(vp));
    wined3d_device_context_get_scissor_rects(
        (*device).immediate_context,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(rect),
    );
    wined3d_stateblock_set_scissor_rect((*device).state, core::ptr::addr_of_mut!(rect));
}

unsafe extern "system" fn d3d8_device_QueryInterface(iface: *mut IDirect3DDevice8, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DDevice8))
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown))
    {
        IDirect3DDevice8_AddRef(iface);
        *out = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    WARN!("%s not implemented, returning E_NOINTERFACE.\n", debugstr_guid(riid));

    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn d3d8_device_AddRef(iface: *mut IDirect3DDevice8) -> ULONG {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let ref_0: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*device).ref_)) as ULONG;

    return ref_0;
}
unsafe extern "system" fn d3d8_device_Release(iface: *mut IDirect3DDevice8) -> ULONG {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut ref_: ULONG;

    if (*device).in_destruction() != 0 {
        return 0;
    }

    ref_ = InterlockedDecrement(core::ptr::addr_of_mut!((*device).ref_) as *mut LONG) as ULONG;

    TRACE!("%p decreasing refcount to %lu.\n", iface, ref_);

    if ref_ == 0 {
        let parent: *mut IDirect3D8 = core::ptr::addr_of_mut!((*(*device).d3d_parent).IDirect3D8_iface);
        let mut i: u32;

        TRACE!("Releasing wined3d device %p.\n", (*device).wined3d_device);

        wined3d_mutex_lock();

        (*device).set_in_destruction((TRUE) as _);

        i = 0;
        while i < (*device).numConvertedDecls {
            d3d8_vertex_declaration_destroy(
                (*(*device).decls.offset(i as isize)).declaration as *mut d3d8_vertex_declaration,
            );
            i = i.wrapping_add(1);
        }
        free((*device).decls as *mut core::ffi::c_void);

        wined3d_streaming_buffer_cleanup(core::ptr::addr_of_mut!((*device).vertex_buffer));
        wined3d_streaming_buffer_cleanup(core::ptr::addr_of_mut!((*device).index_buffer));

        if !(*device).recording.is_null() {
            wined3d_stateblock_decref((*device).recording);
        }
        wined3d_stateblock_decref((*device).state);

        wined3d_swapchain_decref((*device).implicit_swapchain);
        wined3d_device_release_focus_window((*device).wined3d_device);
        wined3d_device_decref((*device).wined3d_device);
        free((*device).handle_table.entries as *mut core::ffi::c_void);
        free(device as *mut core::ffi::c_void);

        wined3d_mutex_unlock();

        IDirect3D8_Release(parent);
    }
    return ref_;
}

unsafe extern "system" fn d3d8_device_TestCooperativeLevel(iface: *mut IDirect3DDevice8) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);

    match ((*device).device_state) as u32 {
        D3D8_DEVICE_STATE_OK => return D3D_OK,
        D3D8_DEVICE_STATE_LOST => return D3DERR_DEVICELOST,
        D3D8_DEVICE_STATE_NOT_RESET | _ => return D3DERR_DEVICENOTRESET,
    };
}

unsafe extern "system" fn d3d8_device_GetAvailableTextureMem(iface: *mut IDirect3DDevice8) -> UINT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut ret: UINT;

    wined3d_mutex_lock();
    ret = wined3d_device_get_available_texture_mem((*device).wined3d_device);
    wined3d_mutex_unlock();

    return ret;
}

unsafe extern "system" fn d3d8_device_ResourceManagerDiscardBytes(iface: *mut IDirect3DDevice8, byte_count: DWORD) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);

    if byte_count != 0 {
        FIXME!("Byte count ignored.\n");
    }

    wined3d_mutex_lock();
    wined3d_device_evict_managed_resources((*device).wined3d_device);
    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_GetDirect3D(iface: *mut IDirect3DDevice8, d3d8: *mut *mut IDirect3D8) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);

    if d3d8.is_null() {
        return D3DERR_INVALIDCALL;
    }

    return IDirect3D8_QueryInterface(
        &raw mut (*(*device).d3d_parent).IDirect3D8_iface,
        core::ptr::addr_of!(IID_IDirect3D8),
        d3d8 as *mut *mut core::ffi::c_void,
    );
}
unsafe extern "system" fn d3d8_device_GetDeviceCaps(
    iface: *mut IDirect3DDevice8,
    caps: *mut D3DCAPS8,
) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut wined3d_caps: wined3d_caps = core::mem::zeroed();
    let mut hr: HRESULT = 0;

    if caps.is_null() {
        return D3DERR_INVALIDCALL;
    }

    WINE_TRACE!("iface %p, caps %p.\n", iface, caps);

    wined3d_mutex_lock();
    hr = wined3d_device_get_device_caps((*device).wined3d_device, &raw mut wined3d_caps);
    wined3d_mutex_unlock();

    d3dcaps_from_wined3dcaps(caps, &raw mut wined3d_caps, (*device).adapter_ordinal);

    return hr;
}

unsafe extern "system" fn d3d8_device_GetDisplayMode(iface: *mut IDirect3DDevice8, mode: *mut D3DDISPLAYMODE) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut wined3d_mode: wined3d_display_mode = core::mem::zeroed();
    let mut hr: HRESULT;

    TRACE!("iface %p, mode %p.\n", iface, mode);

    wined3d_mutex_lock();
    hr = wined3d_device_get_display_mode(
        (*device).wined3d_device,
        0,
        core::ptr::addr_of_mut!(wined3d_mode),
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

unsafe extern "system" fn d3d8_device_GetCreationParameters(
    iface: *mut IDirect3DDevice8,
    parameters: *mut D3DDEVICE_CREATION_PARAMETERS,
) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);

    TRACE!("iface %p, parameters %p.\n", iface, parameters);

    wined3d_mutex_lock();
    wined3d_device_get_creation_parameters(
        (*device).wined3d_device,
        parameters as *mut wined3d_device_creation_parameters,
    );
    wined3d_mutex_unlock();

    (*parameters).AdapterOrdinal = (*device).adapter_ordinal as UINT;
    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_SetCursorProperties(
    iface: *mut IDirect3DDevice8,
    hotspot_x: UINT,
    hotspot_y: UINT,
    bitmap: *mut IDirect3DSurface8,
) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let bitmap_impl: *mut d3d8_surface = unsafe_impl_from_IDirect3DSurface8(bitmap);
    let mut surface_desc: D3DSURFACE_DESC = core::mem::zeroed();
    let mut mode: D3DDISPLAYMODE = core::mem::zeroed();
    let mut hr: HRESULT = 0;

    if bitmap.is_null() {
        WARN!("No cursor bitmap, returning D3DERR_INVALIDCALL.\n");
        return D3DERR_INVALIDCALL;
    }

    hr = IDirect3DSurface8_GetDesc(bitmap, core::ptr::addr_of_mut!(surface_desc));
    if hr < 0 {
        WARN!("Failed to get surface description, hr %#lx.\n", hr);
        return hr;
    }

    hr = IDirect3D8_GetAdapterDisplayMode(
        core::ptr::addr_of_mut!((*(*device).d3d_parent).IDirect3D8_iface),
        (*device).adapter_ordinal,
        core::ptr::addr_of_mut!(mode),
    );
    if hr < 0 {
        WARN!("Failed to get device display mode, hr %#lx.\n", hr);
        return hr;
    }

    if surface_desc.Width > mode.Width || surface_desc.Height > mode.Height {
        WARN!(
            "Surface dimension %ux%u exceeds display mode %ux%u.\n",
            surface_desc.Width,
            surface_desc.Height,
            mode.Width,
            mode.Height
        );
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    hr = wined3d_device_set_cursor_properties(
        (*device).wined3d_device,
        hotspot_x,
        hotspot_y,
        (*bitmap_impl).wined3d_texture,
        (*bitmap_impl).sub_resource_idx,
    );
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d8_device_SetCursorPosition(
    iface: *mut IDirect3DDevice8,
    x: UINT,
    y: UINT,
    flags: DWORD,
) {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    WINE_TRACE!("iface %p, x %u, y %u, flags %#lx.\n", iface, x, y, flags);
    wined3d_mutex_lock();
    wined3d_device_set_cursor_position(
        (*device).wined3d_device,
        x as ::core::ffi::c_int,
        y as ::core::ffi::c_int,
        flags,
    );
    wined3d_mutex_unlock();
}

unsafe extern "system" fn d3d8_device_ShowCursor(iface: *mut IDirect3DDevice8, show: BOOL) -> BOOL {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut ret: BOOL = 0;

    WINE_TRACE!("iface %p, show %#x.\n", iface, show);

    wined3d_mutex_lock();
    ret = wined3d_device_show_cursor((*device).wined3d_device, show);
    wined3d_mutex_unlock();

    return ret;
}
unsafe extern "system" fn d3d8_device_CreateAdditionalSwapChain(iface: *mut IDirect3DDevice8, present_parameters: *mut D3DPRESENT_PARAMETERS, swapchain: *mut *mut IDirect3DSwapChain8) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut desc: wined3d_swapchain_desc = wined3d_swapchain_desc {
        output: core::ptr::null_mut::<wined3d_output>(),
        backbuffer_width: 0,
        backbuffer_height: 0,
        backbuffer_format: WINED3DFMT_UNKNOWN,
        backbuffer_count: 0,
        backbuffer_bind_flags: 0,
        multisample_type: WINED3D_MULTISAMPLE_NONE,
        multisample_quality: 0,
        swap_effect: WINED3D_SWAP_EFFECT_DISCARD,
        device_window: core::ptr::null_mut::<HWND__>(),
        windowed: 0,
        enable_auto_depth_stencil: 0,
        auto_depth_stencil_format: WINED3DFMT_UNKNOWN,
        flags: 0,
        refresh_rate: 0,
        auto_restore_display_mode: 0,
    };
    let mut object: *mut d3d8_swapchain = core::ptr::null_mut::<d3d8_swapchain>();
    let mut swap_interval: u32;
    let mut output_idx: u32;
    let mut i: u32;
    let mut count: u32;
    let mut hr: HRESULT = 0;

    if (*present_parameters).Windowed == 0 {
        WARN!("Trying to create an additional fullscreen swapchain, returning D3DERR_INVALIDCALL.\n");
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    count = wined3d_device_get_swapchain_count((*device).wined3d_device) as u32;
    i = 0 as u32;
    while i < count {
        let mut wined3d_swapchain: *mut wined3d_swapchain = core::ptr::null_mut::<wined3d_swapchain>();

        wined3d_swapchain = wined3d_device_get_swapchain((*device).wined3d_device, i as UINT);
        wined3d_swapchain_get_desc(wined3d_swapchain, core::ptr::addr_of_mut!(desc));

        if desc.windowed == 0 {
            wined3d_mutex_unlock();
            WARN!("Trying to create an additional swapchain in fullscreen mode, returning D3DERR_INVALIDCALL.\n");
            return D3DERR_INVALIDCALL;
        }
        i = i.wrapping_add(1);
    }
    wined3d_mutex_unlock();

    output_idx = (*device).adapter_ordinal;
    if wined3d_swapchain_desc_from_d3d8(
        core::ptr::addr_of_mut!(desc),
        *(*(*device).d3d_parent).wined3d_outputs.offset(output_idx as isize),
        present_parameters,
    ) == 0
    {
        return D3DERR_INVALIDCALL;
    }
    swap_interval = wined3dswapinterval_from_d3d((*present_parameters).FullScreen_PresentationInterval as DWORD) as u32;
    hr = d3d8_swapchain_create(device, core::ptr::addr_of_mut!(desc), swap_interval, core::ptr::addr_of_mut!(object));
    if hr >= 0 {
        *swapchain = core::ptr::addr_of_mut!((*object).IDirect3DSwapChain8_iface);
    }
    present_parameters_from_wined3d_swapchain_desc(
        present_parameters,
        core::ptr::addr_of_mut!(desc),
        (*present_parameters).FullScreen_PresentationInterval as DWORD,
    );

    return hr;
}

unsafe extern "C" fn reset_enum_callback(resource: *mut wined3d_resource) -> HRESULT {
    let mut vertex_buffer: *mut d3d8_vertexbuffer = core::ptr::null_mut();
    let mut index_buffer: *mut d3d8_indexbuffer = core::ptr::null_mut();
    let mut desc: wined3d_resource_desc = core::mem::zeroed();
    let mut texture: *mut IDirect3DBaseTexture8 = core::ptr::null_mut();
    let mut surface: *mut d3d8_surface = core::ptr::null_mut();
    let mut parent: *mut IUnknown = core::ptr::null_mut();

    wined3d_resource_get_desc(resource, &raw mut desc);
    if (desc.access & WINED3D_RESOURCE_ACCESS_CPU) != 0 || (desc.usage & WINED3DUSAGE_MANAGED) != 0 {
        return D3D_OK;
    }

    if desc.resource_type != WINED3D_RTYPE_TEXTURE_2D {
        if (desc.bind_flags & WINED3D_BIND_VERTEX_BUFFER) != 0 {
            vertex_buffer = wined3d_resource_get_parent(resource) as *mut d3d8_vertexbuffer;
            if !vertex_buffer.is_null() && !(*vertex_buffer).draw_buffer.is_null() {
                return D3D_OK;
            }
        }
        if (desc.bind_flags & WINED3D_BIND_INDEX_BUFFER) != 0 {
            index_buffer = wined3d_resource_get_parent(resource) as *mut d3d8_indexbuffer;
            if !index_buffer.is_null() && (*index_buffer).sysmem != false {
                return D3D_OK;
            }
        }

        WARN!("Resource %p in pool D3DPOOL_DEFAULT blocks the Reset call.\n", resource);
        return D3DERR_DEVICELOST;
    }

    parent = wined3d_resource_get_parent(resource) as *mut IUnknown;
    if !parent.is_null()
        && (IUnknown_QueryInterface(
            parent,
            &raw const IID_IDirect3DBaseTexture8,
            &raw mut texture as *mut *mut core::ffi::c_void,
        ) >= 0)
    {
        IDirect3DBaseTexture8_Release(texture);
        WARN!(
            "Texture %p (resource %p) in pool D3DPOOL_DEFAULT blocks the Reset call.\n",
            texture,
            resource
        );
        return D3DERR_DEVICELOST;
    }

    surface = wined3d_texture_get_sub_resource_parent(
        wined3d_texture_from_resource(resource),
        0,
    ) as *mut d3d8_surface;
    if surface.is_null() || (*surface).resource.refcount == 0 {
        return D3D_OK;
    }

    WARN!("Surface %p in pool D3DPOOL_DEFAULT blocks the Reset call.\n", surface);
    return D3DERR_DEVICELOST;
}

unsafe extern "system" fn d3d8_device_Reset(
    iface: *mut IDirect3DDevice8,
    present_parameters: *mut D3DPRESENT_PARAMETERS,
) -> HRESULT {
    let mut swapchain_desc: wined3d_swapchain_desc = core::mem::zeroed();
    let mut old_swapchain_desc: wined3d_swapchain_desc = core::mem::zeroed();
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut implicit_swapchain: *mut d3d8_swapchain = core::ptr::null_mut();
    let mut output_idx: u32;
    let mut hr: HRESULT;

    if ((*device).device_state) as u32 == D3D8_DEVICE_STATE_LOST {
        WARN!("App not active, returning D3DERR_DEVICELOST.\n");
        return D3DERR_DEVICELOST;
    }
    output_idx = (*device).adapter_ordinal;
    if wined3d_swapchain_desc_from_d3d8(
        core::ptr::addr_of_mut!(swapchain_desc),
        *(*(*device).d3d_parent).wined3d_outputs.offset(output_idx as isize),
        present_parameters,
    ) == 0
    {
        return D3DERR_INVALIDCALL;
    }
    swapchain_desc.flags =
        (swapchain_desc.flags as u32 | WINED3D_SWAPCHAIN_IMPLICIT) as u32;

    wined3d_mutex_lock();

    wined3d_streaming_buffer_cleanup(core::ptr::addr_of_mut!((*device).vertex_buffer));
    wined3d_streaming_buffer_cleanup(core::ptr::addr_of_mut!((*device).index_buffer));

    wined3d_swapchain_get_desc(
        (*device).implicit_swapchain,
        core::ptr::addr_of_mut!(old_swapchain_desc),
    );

    if !(*device).recording.is_null() {
        wined3d_stateblock_decref((*device).recording);
    }
    (*device).recording = core::ptr::null_mut();
    (*device).update_state = (*device).state;
    wined3d_stateblock_reset((*device).state);

    /* wined3d_device_reset() may recreate swapchain textures.
     * We do not need to remove the reference to the wined3d swapchain from the
     * old d3d8 surfaces: we will fail the reset if they have 0 references,
     * and therefore they are not actually holding a reference to the wined3d
     * swapchain, and will not do anything with it when they are destroyed. */

    hr = wined3d_device_reset(
        (*device).wined3d_device,
        core::ptr::addr_of_mut!(swapchain_desc),
        core::ptr::null(),
        Some(reset_enum_callback as unsafe extern "C" fn(*mut wined3d_resource) -> HRESULT),
        (TRUE) as BOOL,
    );
    if hr >= 0 {
        let mut rtv: *mut wined3d_rendertarget_view = core::ptr::null_mut();
        let mut surface: *mut d3d8_surface = core::ptr::null_mut();
        let mut i: u32;

        (*present_parameters).BackBufferCount = swapchain_desc.backbuffer_count as UINT;
        implicit_swapchain =
            wined3d_swapchain_get_parent((*device).implicit_swapchain) as *mut d3d8_swapchain;
        (*implicit_swapchain).swap_interval = wined3dswapinterval_from_d3d(
            (*present_parameters).FullScreen_PresentationInterval as DWORD,
        ) as u32;
        wined3d_stateblock_set_render_state(
            (*device).state,
            WINED3D_RS_POINTSIZE_MIN,
            0,
        );
        wined3d_stateblock_set_render_state(
            (*device).state,
            WINED3D_RS_ZENABLE,
            (swapchain_desc.enable_auto_depth_stencil != 0) as i32 as u32,
        );
        device_reset_viewport_state(device);

        /* FIXME: This should be the new backbuffer count, but we don't support
         * changing the backbuffer count in wined3d yet. */
        i = 0;
        while i < old_swapchain_desc.backbuffer_count {
            let backbuffer: *mut wined3d_texture =
                wined3d_swapchain_get_back_buffer((*device).implicit_swapchain, i as UINT);

            surface = d3d8_surface_create(
                backbuffer,
                0,
                core::ptr::addr_of_mut!((*device).IDirect3DDevice8_iface) as *mut IUnknown,
            );
            if !surface.is_null() {
                (*surface).parent_device = core::ptr::addr_of_mut!((*device).IDirect3DDevice8_iface);
            }
            i = i.wrapping_add(1);
        }

        rtv = wined3d_device_context_get_depth_stencil_view((*device).immediate_context);
        if !rtv.is_null() {
            let resource: *mut wined3d_resource = wined3d_rendertarget_view_get_resource(rtv);

            surface = d3d8_surface_create(
                wined3d_texture_from_resource(resource),
                0,
                core::ptr::addr_of_mut!((*device).IDirect3DDevice8_iface) as *mut IUnknown,
            );
            if !surface.is_null() {
                (*surface).parent_device = core::ptr::addr_of_mut!((*device).IDirect3DDevice8_iface);
            }
        }

        (*device).device_state = (D3D8_DEVICE_STATE_OK) as i32;
    } else {
        (*device).device_state = (D3D8_DEVICE_STATE_NOT_RESET) as i32;
    }
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d8_device_Present(
    mut iface: *mut IDirect3DDevice8,
    mut src_rect: *const RECT,
    mut dst_rect: *const RECT,
    mut dst_window_override: HWND,
    mut dirty_region: *const RGNDATA,
) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut implicit_swapchain: *mut d3d8_swapchain = core::ptr::null_mut();
    WINE_TRACE!("iface %p, src_rect %s, dst_rect %s, dst_window_override %p, dirty_region %p.\n", iface, wine_dbgstr_rect(src_rect), wine_dbgstr_rect(dst_rect), dst_window_override, dirty_region);
    implicit_swapchain =
        wined3d_swapchain_get_parent((*device).implicit_swapchain) as *mut d3d8_swapchain;
    return IDirect3DSwapChain8_Present(
        core::ptr::addr_of_mut!((*implicit_swapchain).IDirect3DSwapChain8_iface),
        src_rect,
        dst_rect,
        dst_window_override,
        dirty_region,
    );
}

unsafe extern "system" fn d3d8_device_GetBackBuffer(
    iface: *mut IDirect3DDevice8,
    backbuffer_idx: UINT,
    backbuffer_type: D3DBACKBUFFER_TYPE,
    backbuffer: *mut *mut IDirect3DSurface8,
) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut wined3d_texture: *mut wined3d_texture = core::ptr::null_mut::<wined3d_texture>();
    let mut surface_impl: *mut d3d8_surface = core::ptr::null_mut::<d3d8_surface>();

    WINE_TRACE!("iface %p, backbuffer_idx %u, backbuffer_type %#x, backbuffer %p.\n", iface, backbuffer_idx, backbuffer_type as core::ffi::c_uint, backbuffer);

    /* backbuffer_type is ignored by native. */

    /* No need to check for backbuffer == NULL, Windows crashes in that case. */
    wined3d_mutex_lock();

    wined3d_texture = wined3d_swapchain_get_back_buffer((*device).implicit_swapchain, backbuffer_idx);
    if wined3d_texture.is_null() {
        wined3d_mutex_unlock();
        *backbuffer = core::ptr::null_mut::<IDirect3DSurface8>();
        return D3DERR_INVALIDCALL;
    }

    surface_impl = wined3d_texture_get_sub_resource_parent(wined3d_texture, 0)
        as *mut d3d8_surface;
    *backbuffer = core::ptr::addr_of_mut!((*surface_impl).IDirect3DSurface8_iface);
    IDirect3DSurface8_AddRef(*backbuffer);

    wined3d_mutex_unlock();
    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_GetRasterStatus(
    iface: *mut IDirect3DDevice8,
    raster_status: *mut D3DRASTER_STATUS,
) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut hr: HRESULT = 0;

    wined3d_mutex_lock();
    hr = wined3d_device_get_raster_status(
        (*device).wined3d_device,
        0,
        raster_status as *mut wined3d_raster_status,
    );
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d8_device_SetGammaRamp(iface: *mut IDirect3DDevice8, flags: DWORD, ramp: *const D3DGAMMARAMP) {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);

    /* Note: D3DGAMMARAMP is compatible with struct wined3d_gamma_ramp. */
    wined3d_mutex_lock();
    wined3d_device_set_gamma_ramp((*device).wined3d_device, 0 as UINT, flags, ramp as *const wined3d_gamma_ramp);
    wined3d_mutex_unlock();
}

unsafe extern "system" fn d3d8_device_GetGammaRamp(iface: *mut IDirect3DDevice8, ramp: *mut D3DGAMMARAMP) {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    wined3d_mutex_lock();
    wined3d_device_get_gamma_ramp((*device).wined3d_device, 0, ramp as *mut wined3d_gamma_ramp);
    wined3d_mutex_unlock();
}
unsafe extern "system" fn d3d8_device_CreateTexture(
    mut iface: *mut IDirect3DDevice8,
    mut width: UINT,
    mut height: UINT,
    mut levels: UINT,
    mut usage: DWORD,
    mut format: D3DFORMAT,
    mut pool: D3DPOOL,
    mut texture: *mut *mut IDirect3DTexture8,
) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut object: *mut d3d8_texture = ::core::ptr::null_mut::<d3d8_texture>();
    let mut i: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;

    TRACE!(
        "iface %p, width %u, height %u, levels %u, usage %#lx, format %#x, pool %#x, texture %p.\n",
        iface,
        width,
        height,
        levels,
        usage,
        format as ::core::ffi::c_uint,
        pool as ::core::ffi::c_uint,
        texture,
    );

    if format as u64 == 0 {
        return D3DERR_INVALIDCALL;
    }

    *texture = ::core::ptr::null_mut::<IDirect3DTexture8>();
    object = calloc(
        1,
        (::core::mem::size_of::<d3d8_texture>() as size_t) as _,
    ) as *mut d3d8_texture;
    if object.is_null() {
        return D3DERR_OUTOFVIDEOMEMORY;
    }

    hr = d3d8_texture_2d_init(
        object,
        device,
        width as ::core::ffi::c_uint,
        height as ::core::ffi::c_uint,
        levels as ::core::ffi::c_uint,
        usage,
        format,
        pool,
    );
    if hr < 0 {
        WARN!("Failed to initialize texture, hr %#lx.\n", hr);
        free(object as *mut ::core::ffi::c_void);
        return hr;
    }

    levels = wined3d_texture_get_level_count((*object).wined3d_texture) as UINT;
    i = 0;
    while i < levels {
        if d3d8_surface_create(
            (*object).wined3d_texture,
            i,
            &raw mut (*object).IDirect3DBaseTexture8_iface as *mut IUnknown,
        )
        .is_null()
        {
            IDirect3DTexture8_Release((&raw mut (*object).IDirect3DBaseTexture8_iface).cast());
            return E_OUTOFMEMORY;
        }
        i = i.wrapping_add(1);
    }

    TRACE!("Created texture %p.\n", object);

    *texture = &raw mut (*object).IDirect3DBaseTexture8_iface as *mut IDirect3DTexture8;

    return D3D_OK;
}
unsafe extern "system" fn d3d8_device_CreateVolumeTexture(
    iface: *mut IDirect3DDevice8,
    width: UINT,
    height: UINT,
    depth: UINT,
    levels: UINT,
    usage: DWORD,
    format: D3DFORMAT,
    pool: D3DPOOL,
    texture: *mut *mut IDirect3DVolumeTexture8,
) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut object: *mut d3d8_texture;
    let mut hr: HRESULT;

    if format == 0 {
        return D3DERR_INVALIDCALL;
    }

    *texture = core::ptr::null_mut();
    object = calloc(1, (core::mem::size_of::<d3d8_texture>() as usize) as _) as *mut d3d8_texture;
    if object.is_null() {
        return D3DERR_OUTOFVIDEOMEMORY;
    }

    hr = d3d8_texture_3d_init(object, device, width, height, depth, levels, usage, format, pool);
    if hr < 0 {
        WARN!("Failed to initialize volume texture, hr %#lx.\n", hr);
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    *texture = core::ptr::addr_of_mut!((*object).IDirect3DBaseTexture8_iface) as *mut IDirect3DVolumeTexture8;

    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_CreateCubeTexture(
    iface: *mut IDirect3DDevice8,
    edge_length: UINT,
    mut levels: UINT,
    usage: DWORD,
    format: D3DFORMAT,
    pool: D3DPOOL,
    texture: *mut *mut IDirect3DCubeTexture8,
) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut object: *mut d3d8_texture;
    let mut i: ::core::ffi::c_uint;
    let mut hr: HRESULT;

    if format == 0 {
        return D3DERR_INVALIDCALL;
    }

    *texture = core::ptr::null_mut();
    object = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<d3d8_texture>() as size_t) as _,
    ) as *mut d3d8_texture;
    if object.is_null() {
        return D3DERR_OUTOFVIDEOMEMORY;
    }

    hr = d3d8_texture_cube_init(object, device, edge_length, levels, usage, format, pool);
    if hr < 0 {
        WARN!("Failed to initialize cube texture, hr %#lx.\n", hr);
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    levels = wined3d_texture_get_level_count((*object).wined3d_texture) as UINT;
    i = 0;
    while i < levels.wrapping_mul(6) {
        if d3d8_surface_create(
            (*object).wined3d_texture,
            i,
            core::ptr::addr_of_mut!((*object).IDirect3DBaseTexture8_iface) as *mut IUnknown,
        )
        .is_null()
        {
            IDirect3DTexture8_Release((core::ptr::addr_of_mut!((*object).IDirect3DBaseTexture8_iface)).cast());
            return E_OUTOFMEMORY;
        }
        i = i.wrapping_add(1);
    }

    *texture = core::ptr::addr_of_mut!((*object).IDirect3DBaseTexture8_iface)
        as *mut IDirect3DCubeTexture8;

    return hr;
}

unsafe extern "system" fn d3d8_device_CreateVertexBuffer(iface: *mut IDirect3DDevice8, size: UINT, usage: DWORD, fvf: DWORD, pool: D3DPOOL, buffer: *mut *mut IDirect3DVertexBuffer8) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut object: *mut d3d8_vertexbuffer = core::ptr::null_mut();
    let mut hr: HRESULT;

    object = calloc((1 as size_t) as _, (core::mem::size_of::<d3d8_vertexbuffer>() as size_t) as _) as *mut d3d8_vertexbuffer;
    if object.is_null() {
        return D3DERR_OUTOFVIDEOMEMORY;
    }

    hr = vertexbuffer_init(object, device, size, usage, fvf, pool);
    if hr < 0 {
        WARN!("Failed to initialize vertex buffer, hr %#lx.\n", hr);
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    *buffer = core::ptr::addr_of_mut!((*object).IDirect3DVertexBuffer8_iface);

    return D3D_OK;
}
unsafe extern "system" fn d3d8_device_CreateIndexBuffer(
    iface: *mut IDirect3DDevice8,
    size: UINT,
    usage: DWORD,
    format: D3DFORMAT,
    pool: D3DPOOL,
    buffer: *mut *mut IDirect3DIndexBuffer8,
) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut object: *mut d3d8_indexbuffer = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    object = calloc((1 as size_t) as _, (core::mem::size_of::<d3d8_indexbuffer>() as size_t) as _) as *mut d3d8_indexbuffer;
    if object.is_null() {
        return D3DERR_OUTOFVIDEOMEMORY;
    }

    hr = indexbuffer_init(object, device, size, usage, format, pool);
    if hr < 0 {
        WARN!("Failed to initialize index buffer, hr %#lx.\n", hr);
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    *buffer = core::ptr::addr_of_mut!((*object).IDirect3DIndexBuffer8_iface);

    return D3D_OK;
}

unsafe extern "C" fn d3d8_device_create_surface(device: *mut d3d8_device, format: wined3d_format_id, multisample_type: wined3d_multisample_type, bind_flags: ::core::ffi::c_uint, access: ::core::ffi::c_uint, width: ::core::ffi::c_uint, height: ::core::ffi::c_uint, surface: *mut *mut IDirect3DSurface8) -> HRESULT {
    let mut desc: wined3d_resource_desc = core::mem::zeroed();
    let mut surface_impl: *mut d3d8_surface = core::ptr::null_mut();
    let mut texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut hr: HRESULT;

    desc.resource_type = WINED3D_RTYPE_TEXTURE_2D;
    desc.format = format;
    desc.multisample_type = multisample_type;
    desc.multisample_quality = 0;
    desc.usage = 0;
    desc.bind_flags = bind_flags;
    desc.access = access;
    desc.width = width;
    desc.height = height;
    desc.depth = 1;
    desc.size = 0;

    wined3d_mutex_lock();

    hr = wined3d_texture_create(
        (*device).wined3d_device,
        core::ptr::addr_of_mut!(desc),
        1,
        1,
        0,
        core::ptr::null(),
        core::ptr::null_mut(),
        core::ptr::addr_of!(d3d8_null_wined3d_parent_ops),
        core::ptr::addr_of_mut!(texture),
    );
    if hr < 0 {
        wined3d_mutex_unlock();
        WARN!("Failed to create texture, hr %#lx.\n", hr);
        return hr;
    }

    surface_impl = d3d8_surface_create(texture, 0, core::ptr::null_mut());
    if surface_impl.is_null() {
        wined3d_texture_decref(texture);
        wined3d_mutex_unlock();
        return E_OUTOFMEMORY;
    }
    (*surface_impl).parent_device = core::ptr::addr_of_mut!((*device).IDirect3DDevice8_iface);
    *surface = core::ptr::addr_of_mut!((*surface_impl).IDirect3DSurface8_iface);
    IDirect3DSurface8_AddRef(*surface);
    wined3d_texture_decref(texture);

    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_CreateRenderTarget(
    iface: *mut IDirect3DDevice8,
    width: UINT,
    height: UINT,
    format: D3DFORMAT,
    multisample_type: D3DMULTISAMPLE_TYPE,
    lockable: BOOL,
    surface: *mut *mut IDirect3DSurface8,
) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut access: u32 = WINED3D_RESOURCE_ACCESS_GPU;

    TRACE!(
        "iface %p, width %u, height %u, format %#x, multisample_type %#x, lockable %#x, surface %p.\n",
        iface,
        width,
        height,
        format as u32,
        multisample_type as u32,
        lockable,
        surface
    );

    if format as u64 == 0 {
        return D3DERR_INVALIDCALL;
    }

    *surface = core::ptr::null_mut();
    if lockable != 0 {
        access |= WINED3D_RESOURCE_ACCESS_MAP_R | WINED3D_RESOURCE_ACCESS_MAP_W;
    }

    return d3d8_device_create_surface(
        device,
        wined3dformat_from_d3dformat(format),
        wined3d_multisample_type_from_d3d(multisample_type),
        WINED3D_BIND_RENDER_TARGET as u32,
        access,
        width,
        height,
        surface,
    );
}

unsafe extern "system" fn d3d8_device_CreateDepthStencilSurface(
    mut iface: *mut IDirect3DDevice8,
    mut width: UINT,
    mut height: UINT,
    mut format: D3DFORMAT,
    mut multisample_type: D3DMULTISAMPLE_TYPE,
    mut surface: *mut *mut IDirect3DSurface8,
) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    TRACE!(
        "iface %p, width %u, height %u, format %#x, multisample_type %#x, surface %p.\n",
        iface,
        width,
        height,
        format,
        multisample_type,
        surface
    );
    if format as u64 == 0 {
        return D3DERR_INVALIDCALL;
    }
    *surface = core::ptr::null_mut::<IDirect3DSurface8>();
    return d3d8_device_create_surface(
        device,
        wined3dformat_from_d3dformat(format),
        wined3d_multisample_type_from_d3d(multisample_type),
        WINED3D_BIND_DEPTH_STENCIL as u32,
        WINED3D_RESOURCE_ACCESS_GPU,
        width,
        height,
        surface,
    );
}
unsafe extern "system" fn d3d8_device_CreateImageSurface(
    mut iface: *mut IDirect3DDevice8,
    mut width: UINT,
    mut height: UINT,
    mut format: D3DFORMAT,
    mut surface: *mut *mut IDirect3DSurface8,
) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut access: ::core::ffi::c_uint =
        WINED3D_RESOURCE_ACCESS_CPU | WINED3D_RESOURCE_ACCESS_MAP_R | WINED3D_RESOURCE_ACCESS_MAP_W;
    WINE_TRACE!("iface %p, width %u, height %u, format %#x, surface %p.\n", iface, width, height, format as ::core::ffi::c_uint, surface);
    *surface = ::core::ptr::null_mut::<IDirect3DSurface8>();
    return d3d8_device_create_surface(
        device,
        wined3dformat_from_d3dformat(format),
        WINED3D_MULTISAMPLE_NONE,
        0,
        access,
        width,
        height,
        surface,
    );
}

unsafe extern "system" fn d3d8_device_CopyRects(
    iface: *mut IDirect3DDevice8,
    src_surface: *mut IDirect3DSurface8,
    src_rects: *const RECT,
    rect_count: UINT,
    dst_surface: *mut IDirect3DSurface8,
    dst_points: *const POINT,
) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let src: *mut d3d8_surface = unsafe_impl_from_IDirect3DSurface8(src_surface);
    let dst: *mut d3d8_surface = unsafe_impl_from_IDirect3DSurface8(dst_surface);
    let mut src_format: wined3d_format_id = core::mem::zeroed();
    let mut dst_format: wined3d_format_id = core::mem::zeroed();
    let mut wined3d_desc: wined3d_sub_resource_desc = core::mem::zeroed();
    let mut src_w: UINT;
    let mut src_h: UINT;

    wined3d_mutex_lock();
    wined3d_texture_get_sub_resource_desc(
        (*src).wined3d_texture,
        (*src).sub_resource_idx,
        core::ptr::addr_of_mut!(wined3d_desc),
    );
    if (wined3d_desc.bind_flags & WINED3D_BIND_DEPTH_STENCIL) != 0 {
        WARN!("Source %p is a depth stencil surface, returning D3DERR_INVALIDCALL.\n", src_surface);
        wined3d_mutex_unlock();
        return D3DERR_INVALIDCALL;
    }
    src_format = wined3d_desc.format;
    src_w = wined3d_desc.width as UINT;
    src_h = wined3d_desc.height as UINT;

    wined3d_texture_get_sub_resource_desc(
        (*dst).wined3d_texture,
        (*dst).sub_resource_idx,
        core::ptr::addr_of_mut!(wined3d_desc),
    );
    if (wined3d_desc.bind_flags & WINED3D_BIND_DEPTH_STENCIL) != 0 {
        WARN!("Destination %p is a depth stencil surface, returning D3DERR_INVALIDCALL.\n", dst_surface);
        wined3d_mutex_unlock();
        return D3DERR_INVALIDCALL;
    }
    dst_format = wined3d_desc.format;

    /* Check that the source and destination formats match */
    if src_format as core::ffi::c_uint != dst_format as core::ffi::c_uint {
        WARN!(
            "Source %p format must match the destination %p format, returning D3DERR_INVALIDCALL.\n",
            src_surface,
            dst_surface
        );
        wined3d_mutex_unlock();
        return D3DERR_INVALIDCALL;
    }

    /* Quick if complete copy ... */
    if rect_count == 0 && src_rects.is_null() && dst_points.is_null() {
        let mut rect: RECT = RECT {
            left: 0 as LONG,
            top: 0 as LONG,
            right: src_w as LONG,
            bottom: src_h as LONG,
        };
        wined3d_device_context_blt(
            (*device).immediate_context,
            (*dst).wined3d_texture,
            (*dst).sub_resource_idx,
            core::ptr::addr_of_mut!(rect),
            (*src).wined3d_texture,
            (*src).sub_resource_idx,
            core::ptr::addr_of_mut!(rect),
            0 as core::ffi::c_uint,
            core::ptr::null::<wined3d_blt_fx>(),
            WINED3D_TEXF_POINT,
        );
    } else {
        let mut i: core::ffi::c_uint;
        /* Copy rect by rect */
        if !src_rects.is_null() && !dst_points.is_null() {
            i = 0 as core::ffi::c_uint;
            while i < rect_count {
                let w: UINT = ((*src_rects.offset(i as isize)).right
                    - (*src_rects.offset(i as isize)).left) as UINT;
                let h: UINT = ((*src_rects.offset(i as isize)).bottom
                    - (*src_rects.offset(i as isize)).top) as UINT;
                let mut dst_rect: RECT = RECT {
                    left: (*dst_points.offset(i as isize)).x,
                    top: (*dst_points.offset(i as isize)).y,
                    right: ((*dst_points.offset(i as isize)).x as UINT).wrapping_add(w) as LONG,
                    bottom: ((*dst_points.offset(i as isize)).y as UINT).wrapping_add(h) as LONG,
                };

                wined3d_device_context_blt(
                    (*device).immediate_context,
                    (*dst).wined3d_texture,
                    (*dst).sub_resource_idx,
                    core::ptr::addr_of_mut!(dst_rect),
                    (*src).wined3d_texture,
                    (*src).sub_resource_idx,
                    src_rects.offset(i as isize),
                    0 as core::ffi::c_uint,
                    core::ptr::null::<wined3d_blt_fx>(),
                    WINED3D_TEXF_POINT,
                );
                i = i.wrapping_add(1);
            }
        } else {
            i = 0 as core::ffi::c_uint;
            while i < rect_count {
                let w: UINT = ((*src_rects.offset(i as isize)).right
                    - (*src_rects.offset(i as isize)).left) as UINT;
                let h: UINT = ((*src_rects.offset(i as isize)).bottom
                    - (*src_rects.offset(i as isize)).top) as UINT;
                let mut dst_rect: RECT = RECT {
                    left: 0 as LONG,
                    top: 0 as LONG,
                    right: w as LONG,
                    bottom: h as LONG,
                };

                wined3d_device_context_blt(
                    (*device).immediate_context,
                    (*dst).wined3d_texture,
                    (*dst).sub_resource_idx,
                    core::ptr::addr_of_mut!(dst_rect),
                    (*src).wined3d_texture,
                    (*src).sub_resource_idx,
                    src_rects.offset(i as isize),
                    0 as core::ffi::c_uint,
                    core::ptr::null::<wined3d_blt_fx>(),
                    WINED3D_TEXF_POINT,
                );
                i = i.wrapping_add(1);
            }
        }
    }
    wined3d_mutex_unlock();

    return WINED3D_OK;
}

unsafe extern "system" fn d3d8_device_UpdateTexture(
    iface: *mut IDirect3DDevice8,
    src_texture: *mut IDirect3DBaseTexture8,
    dst_texture: *mut IDirect3DBaseTexture8,
) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut src_impl: *mut d3d8_texture;
    let mut dst_impl: *mut d3d8_texture;
    let mut hr: HRESULT;

    TRACE!(
        "iface %p, src_texture %p, dst_texture %p.\n",
        iface,
        src_texture,
        dst_texture
    );

    src_impl = unsafe_impl_from_IDirect3DBaseTexture8(src_texture);
    dst_impl = unsafe_impl_from_IDirect3DBaseTexture8(dst_texture);

    if !(*src_impl).draw_texture.is_null() || !(*dst_impl).draw_texture.is_null() {
        WARN!("Source or destination is managed; returning D3DERR_INVALIDCALL.\n");
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    hr = wined3d_device_update_texture(
        (*device).wined3d_device,
        (*src_impl).wined3d_texture,
        (*dst_impl).wined3d_texture,
    );
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d8_device_GetFrontBuffer(iface: *mut IDirect3DDevice8, dst_surface: *mut IDirect3DSurface8) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let dst_impl: *mut d3d8_surface = unsafe_impl_from_IDirect3DSurface8(dst_surface);
    let mut hr: HRESULT;

    if dst_surface.is_null() {
        WARN!("Invalid destination surface passed.\n");
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    hr = wined3d_swapchain_get_front_buffer_data(
        (*device).implicit_swapchain,
        (*dst_impl).wined3d_texture,
        (*dst_impl).sub_resource_idx,
    );
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d8_device_SetRenderTarget(
    iface: *mut IDirect3DDevice8,
    render_target: *mut IDirect3DSurface8,
    depth_stencil: *mut IDirect3DSurface8,
) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let rt_impl: *mut d3d8_surface = unsafe_impl_from_IDirect3DSurface8(render_target);
    let ds_impl: *mut d3d8_surface = unsafe_impl_from_IDirect3DSurface8(depth_stencil);
    let mut original_dsv: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut rtv: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut hr: HRESULT = D3D_OK;

    if !rt_impl.is_null() && d3d8_surface_get_device(rt_impl) != device {
        WARN!("Render target surface does not match device.\n");
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();

    if !ds_impl.is_null() {
        let mut ds_desc: wined3d_sub_resource_desc = core::mem::zeroed();
        let mut rt_desc: wined3d_sub_resource_desc = core::mem::zeroed();
        let mut original_rtv: *mut wined3d_rendertarget_view = core::ptr::null_mut();
        let mut original_surface: *mut d3d8_surface = core::ptr::null_mut();

        /* If no render target is passed in check the size against the current RT */
        if render_target.is_null() {
            original_rtv = wined3d_device_context_get_rendertarget_view((*device).immediate_context, 0);
            if original_rtv.is_null() {
                wined3d_mutex_unlock();
                return D3DERR_NOTFOUND;
            }
            original_surface = wined3d_rendertarget_view_get_sub_resource_parent(original_rtv) as *mut d3d8_surface;
            wined3d_texture_get_sub_resource_desc(
                (*original_surface).wined3d_texture,
                (*original_surface).sub_resource_idx,
                core::ptr::addr_of_mut!(rt_desc),
            );
        } else {
            wined3d_texture_get_sub_resource_desc(
                (*rt_impl).wined3d_texture,
                (*rt_impl).sub_resource_idx,
                core::ptr::addr_of_mut!(rt_desc),
            );
        }

        wined3d_texture_get_sub_resource_desc(
            (*ds_impl).wined3d_texture,
            (*ds_impl).sub_resource_idx,
            core::ptr::addr_of_mut!(ds_desc),
        );

        if ds_desc.width < rt_desc.width || ds_desc.height < rt_desc.height {
            WARN!("Depth stencil is smaller than the render target, returning D3DERR_INVALIDCALL\n");
            wined3d_mutex_unlock();
            return D3DERR_INVALIDCALL;
        }
        if ds_desc.multisample_type != rt_desc.multisample_type
            || ds_desc.multisample_quality != rt_desc.multisample_quality
        {
            WARN!("Multisample settings do not match, returning D3DERR_INVALIDCALL\n");
            wined3d_mutex_unlock();
            return D3DERR_INVALIDCALL;
        }
    }

    original_dsv = wined3d_device_context_get_depth_stencil_view((*device).immediate_context);
    rtv = if !ds_impl.is_null() {
        d3d8_surface_acquire_rendertarget_view(ds_impl)
    } else {
        core::ptr::null_mut()
    };
    hr = wined3d_device_context_set_depth_stencil_view((*device).immediate_context, rtv);
    d3d8_surface_release_rendertarget_view(ds_impl, rtv);
    if hr >= 0 {
        rtv = if !render_target.is_null() {
            d3d8_surface_acquire_rendertarget_view(rt_impl)
        } else {
            core::ptr::null_mut()
        };
        if !render_target.is_null() {
            hr = wined3d_device_context_set_rendertarget_views(
                (*device).immediate_context,
                0,
                1,
                core::ptr::addr_of_mut!(rtv),
                (TRUE) as BOOL,
            );
            if hr >= 0 {
                device_reset_viewport_state(device);
            } else {
                wined3d_device_context_set_depth_stencil_view((*device).immediate_context, original_dsv);
            }
        }
        d3d8_surface_release_rendertarget_view(rt_impl, rtv);
        wined3d_stateblock_depth_buffer_changed((*device).state);
    }

    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d8_device_GetRenderTarget(
    iface: *mut IDirect3DDevice8,
    render_target: *mut *mut IDirect3DSurface8,
) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut wined3d_rtv: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut surface_impl: *mut d3d8_surface = core::ptr::null_mut();
    let mut hr: HRESULT;

    if render_target.is_null() {
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    wined3d_rtv = wined3d_device_context_get_rendertarget_view((*device).immediate_context, 0);
    if !wined3d_rtv.is_null() {
        /* We want the sub resource parent here, since the view itself may be
         * internal to wined3d and may not have a parent. */
        surface_impl = wined3d_rendertarget_view_get_sub_resource_parent(wined3d_rtv) as *mut d3d8_surface;
        *render_target = core::ptr::addr_of_mut!((*surface_impl).IDirect3DSurface8_iface);
        IDirect3DSurface8_AddRef(*render_target);
        hr = D3D_OK;
    } else {
        ERR!("Failed to get wined3d render target.\n");
        *render_target = core::ptr::null_mut();
        hr = D3DERR_NOTFOUND;
    }
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d8_device_GetDepthStencilSurface(
    mut iface: *mut IDirect3DDevice8,
    mut depth_stencil: *mut *mut IDirect3DSurface8,
) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut wined3d_dsv: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut surface_impl: *mut d3d8_surface = core::ptr::null_mut();
    let mut hr: HRESULT = D3D_OK;

    TRACE!("iface %p, depth_stencil %p.\n", iface, depth_stencil);

    if depth_stencil.is_null() {
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    wined3d_dsv = wined3d_device_context_get_depth_stencil_view((*device).immediate_context);
    if !wined3d_dsv.is_null() {
        /* We want the sub resource parent here, since the view itself may be
         * internal to wined3d and may not have a parent. */
        surface_impl = wined3d_rendertarget_view_get_sub_resource_parent(wined3d_dsv) as *mut d3d8_surface;
        *depth_stencil = &raw mut (*surface_impl).IDirect3DSurface8_iface;
        IDirect3DSurface8_AddRef(*depth_stencil);
    } else {
        hr = D3DERR_NOTFOUND;
        *depth_stencil = core::ptr::null_mut();
    }
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d8_device_BeginScene(iface: *mut IDirect3DDevice8) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut hr: HRESULT;

    wined3d_mutex_lock();
    hr = wined3d_device_begin_scene((*device).wined3d_device);
    wined3d_mutex_unlock();

    return hr;
}
#[no_mangle]
pub unsafe extern "system" fn d3d8_device_EndScene(iface: *mut IDirect3DDevice8) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut hr: HRESULT;

    wined3d_mutex_lock();
    hr = wined3d_device_end_scene((*device).wined3d_device);
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d8_device_Clear(iface: *mut IDirect3DDevice8, mut rect_count: DWORD, rects: *const D3DRECT, flags: DWORD, color: D3DCOLOR, z: f32, stencil: DWORD) -> HRESULT {
    let c: wined3d_color = wined3d_color {
        r: ((color >> 16) & 0xff) as f32 / 255.0f32,
        g: ((color >> 8) & 0xff) as f32 / 255.0f32,
        b: (color & 0xff) as f32 / 255.0f32,
        a: ((color >> 24) & 0xff) as f32 / 255.0f32,
    };
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut hr: HRESULT = 0;

    TRACE!("iface %p, rect_count %lu, rects %p, flags %#lx, color 0x%08lx, z %.8e, stencil %lu.\n", iface, rect_count, rects, flags, color, z as f64, stencil);

    if rect_count != 0 && rects.is_null() {
        WARN!("count %lu with NULL rects.\n", rect_count);
        rect_count = 0 as DWORD;
    }

    wined3d_mutex_lock();
    wined3d_stateblock_apply_clear_state((*device).state, (*device).wined3d_device);
    hr = wined3d_device_clear((*device).wined3d_device, rect_count, rects as *const RECT, flags, core::ptr::addr_of!(c), z, stencil);
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d8_device_SetTransform(
    iface: *mut IDirect3DDevice8,
    state: D3DTRANSFORMSTATETYPE,
    matrix: *const D3DMATRIX,
) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);

    /* Note: D3DMATRIX is compatible with struct wined3d_matrix. */
    wined3d_mutex_lock();
    wined3d_stateblock_set_transform(
        (*device).update_state,
        wined3d_transform_state_from_d3d(state),
        matrix as *const wined3d_matrix,
    );
    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_GetTransform(
    mut iface: *mut IDirect3DDevice8,
    mut state: D3DTRANSFORMSTATETYPE,
    mut matrix: *mut D3DMATRIX,
) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    TRACE!(
        "iface %p, state %#x, matrix %p.\n",
        iface,
        state as u32,
        matrix
    );
    /* Note: D3DMATRIX is compatible with struct wined3d_matrix. */
    wined3d_mutex_lock();
    memcpy(
        matrix as *mut core::ffi::c_void,
        (core::ptr::addr_of!((*(*device).stateblock_state).transforms) as *const wined3d_matrix)
            .offset(state as isize) as *const core::ffi::c_void,
        core::mem::size_of::<D3DMATRIX>() as usize,
    );
    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d8_device_MultiplyTransform(
    iface: *mut IDirect3DDevice8,
    state: D3DTRANSFORMSTATETYPE,
    matrix: *const D3DMATRIX,
) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);

    /* Note: D3DMATRIX is compatible with struct wined3d_matrix. */
    wined3d_mutex_lock();
    wined3d_stateblock_multiply_transform(
        (*device).state,
        wined3d_transform_state_from_d3d(state),
        matrix as *const wined3d_matrix,
    );
    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d8_device_SetViewport(
    iface: *mut IDirect3DDevice8,
    viewport: *const D3DVIEWPORT8,
) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut rt_desc: wined3d_sub_resource_desc = core::mem::zeroed();
    let mut rtv: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut surface: *mut d3d8_surface = core::ptr::null_mut();
    let mut vp: wined3d_viewport = core::mem::zeroed();

    TRACE!("iface %p, viewport %p.\n", iface, viewport);

    wined3d_mutex_lock();
    rtv = wined3d_device_context_get_rendertarget_view((*device).immediate_context, 0);
    if rtv.is_null() {
        wined3d_mutex_unlock();
        return D3DERR_NOTFOUND;
    }
    surface = wined3d_rendertarget_view_get_sub_resource_parent(rtv) as *mut d3d8_surface;
    wined3d_texture_get_sub_resource_desc(
        (*surface).wined3d_texture,
        (*surface).sub_resource_idx,
        core::ptr::addr_of_mut!(rt_desc),
    );

    if (*viewport).X > rt_desc.width
        || (*viewport).Width > (rt_desc.width as DWORD).wrapping_sub((*viewport).X)
        || (*viewport).Y > rt_desc.height
        || (*viewport).Height > (rt_desc.height as DWORD).wrapping_sub((*viewport).Y)
    {
        WARN!("Invalid viewport, returning D3DERR_INVALIDCALL.\n");
        wined3d_mutex_unlock();
        return D3DERR_INVALIDCALL;
    }

    vp.x = (*viewport).X as f32;
    vp.y = (*viewport).Y as f32;
    vp.width = (*viewport).Width as f32;
    vp.height = (*viewport).Height as f32;
    vp.min_z = (*viewport).MinZ;
    vp.max_z = (*viewport).MaxZ;

    wined3d_stateblock_set_viewport((*device).update_state, core::ptr::addr_of_mut!(vp));
    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_GetViewport(
    iface: *mut IDirect3DDevice8,
    viewport: *mut D3DVIEWPORT8,
) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut wined3d_viewport: wined3d_viewport = wined3d_viewport {
        x: 0.,
        y: 0.,
        width: 0.,
        height: 0.,
        min_z: 0.,
        max_z: 0.,
    };

    wined3d_mutex_lock();
    wined3d_viewport = (*(*device).stateblock_state).viewport;
    wined3d_mutex_unlock();

    (*viewport).X = wined3d_viewport.x as DWORD;
    (*viewport).Y = wined3d_viewport.y as DWORD;
    (*viewport).Width = wined3d_viewport.width as DWORD;
    (*viewport).Height = wined3d_viewport.height as DWORD;
    (*viewport).MinZ = wined3d_viewport.min_z;
    (*viewport).MaxZ = wined3d_viewport.max_z;

    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_SetMaterial(iface: *mut IDirect3DDevice8, material: *const D3DMATERIAL8) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);

    TRACE!("iface %p, material %p.\n", iface, material);
    wined3d_mutex_lock();
    wined3d_stateblock_set_material((*device).update_state, material as *const wined3d_material);
    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_GetMaterial(
    mut iface: *mut IDirect3DDevice8,
    mut material: *mut D3DMATERIAL8,
) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    WINE_TRACE!("iface %p, material %p.\n", iface, material);
    wined3d_mutex_lock();
    memcpy(
        material as *mut ::core::ffi::c_void,
        &raw const (*(*device).stateblock_state).material as *const ::core::ffi::c_void,
        ::core::mem::size_of::<D3DMATERIAL8>() as size_t,
    );
    wined3d_mutex_unlock();
    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_SetLight(iface: *mut IDirect3DDevice8, index: DWORD, light: *const D3DLIGHT8) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut hr: HRESULT;

    /* Note: D3DLIGHT8 is compatible with struct wined3d_light. */
    TRACE!("iface %p, index %lu, light %p.\n", iface, index, light);
    wined3d_mutex_lock();
    hr = wined3d_stateblock_set_light((*device).update_state, index as UINT, light as *const wined3d_light);
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d8_device_GetLight(
    mut iface: *mut IDirect3DDevice8,
    mut index: DWORD,
    mut light: *mut D3DLIGHT8,
) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut enabled: BOOL = 0;
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, index %lu, light %p.\n", iface, index, light);
    wined3d_mutex_lock();
    hr = wined3d_stateblock_get_light(
        (*device).state,
        index as UINT,
        light as *mut wined3d_light,
        &raw mut enabled,
    );
    wined3d_mutex_unlock();
    return hr;
}
unsafe extern "system" fn d3d8_device_LightEnable(
    mut iface: *mut IDirect3DDevice8,
    mut index: DWORD,
    mut enable: BOOL,
) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, index %lu, enable %#x.\n", iface, index, enable);
    wined3d_mutex_lock();
    hr = wined3d_stateblock_set_light_enable((*device).update_state, index as UINT, enable);
    wined3d_mutex_unlock();
    return hr;
}
unsafe extern "system" fn d3d8_device_GetLightEnable(
    mut iface: *mut IDirect3DDevice8,
    mut index: DWORD,
    mut enabled: *mut BOOL,
) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut light: wined3d_light = wined3d_light {
        type_: 0 as wined3d_light_type,
        diffuse: wined3d_color {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 0.,
        },
        specular: wined3d_color {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 0.,
        },
        ambient: wined3d_color {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 0.,
        },
        position: wined3d_vec3 {
            x: 0.,
            y: 0.,
            z: 0.,
        },
        direction: wined3d_vec3 {
            x: 0.,
            y: 0.,
            z: 0.,
        },
        range: 0.,
        falloff: 0.,
        attenuation0: 0.,
        attenuation1: 0.,
        attenuation2: 0.,
        theta: 0.,
        phi: 0.,
    };
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, index %lu, enabled %p.\n", iface, index, enabled);
    wined3d_mutex_lock();
    hr = wined3d_stateblock_get_light((*device).state, index as UINT, core::ptr::addr_of_mut!(light), enabled);
    wined3d_mutex_unlock();
    return hr;
}

unsafe extern "system" fn d3d8_device_SetClipPlane(iface: *mut IDirect3DDevice8, index: DWORD, plane: *const f32) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut hr: HRESULT;

    wined3d_mutex_lock();
    hr = wined3d_stateblock_set_clip_plane((*device).update_state, index as UINT, plane as *const wined3d_vec4);
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d8_device_GetClipPlane(iface: *mut IDirect3DDevice8, index: DWORD, plane: *mut f32) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);

    if index >= (*device).max_user_clip_planes {
        return WINED3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    let src: *const wined3d_vec4 = (&raw const (*(*device).stateblock_state).clip_planes as *const wined3d_vec4).offset(index as isize);
    memcpy(
        plane as *mut core::ffi::c_void,
        src as *const core::ffi::c_void,
        core::mem::size_of::<wined3d_vec4>() as size_t,
    );
    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "C" fn resolve_depth_buffer(mut device: *mut d3d8_device) {
    let mut state: *const wined3d_stateblock_state = (*device).stateblock_state;
    let mut wined3d_dsv: *mut wined3d_rendertarget_view =
        ::core::ptr::null_mut::<wined3d_rendertarget_view>();
    let mut dst_resource: *mut wined3d_resource = ::core::ptr::null_mut::<wined3d_resource>();
    let mut dst_texture: *mut wined3d_texture = ::core::ptr::null_mut::<wined3d_texture>();
    let mut desc: wined3d_resource_desc = core::mem::zeroed::<wined3d_resource_desc>();
    let mut d3d8_dsv: *mut d3d8_surface = ::core::ptr::null_mut::<d3d8_surface>();
    dst_texture = (*state).textures[0 as usize] as *mut wined3d_texture;
    if dst_texture.is_null() {
        return;
    }
    dst_resource = wined3d_texture_get_resource(dst_texture);
    wined3d_resource_get_desc(dst_resource, &raw mut desc);
    if desc.format as ::core::ffi::c_uint
        != WINED3DFMT_D24_UNORM_S8_UINT as ::core::ffi::c_uint
        && desc.format as ::core::ffi::c_uint
            != WINED3DFMT_X8D24_UNORM as ::core::ffi::c_uint
        && desc.format as ::core::ffi::c_uint
            != WINED3DFMT_DF16 as ::core::ffi::c_uint
        && desc.format as ::core::ffi::c_uint
            != WINED3DFMT_DF24 as ::core::ffi::c_uint
        && desc.format as ::core::ffi::c_uint
            != WINED3DFMT_INTZ as ::core::ffi::c_uint
    {
        return;
    }
    wined3d_dsv = wined3d_device_context_get_depth_stencil_view((*device).immediate_context);
    if wined3d_dsv.is_null() {
        return;
    }
    d3d8_dsv = wined3d_rendertarget_view_get_sub_resource_parent(wined3d_dsv) as *mut d3d8_surface;
    wined3d_device_context_resolve_sub_resource(
        (*device).immediate_context,
        dst_resource,
        0 as ::core::ffi::c_uint,
        wined3d_rendertarget_view_get_resource(wined3d_dsv),
        (*d3d8_dsv).sub_resource_idx,
        desc.format,
    );
}
unsafe extern "system" fn d3d8_device_SetRenderState(
    iface: *mut IDirect3DDevice8,
    state: D3DRENDERSTATETYPE,
    value: DWORD,
) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);

    TRACE!("iface %p, state %#x, value %#lx.\n", iface, state as u32, value);

    wined3d_mutex_lock();
    wined3d_stateblock_set_render_state(
        (*device).update_state,
        wined3d_render_state_from_d3d(state),
        value as u32,
    );
    if state as u32 == D3DRS_POINTSIZE && value == D3D8_RESZ_CODE {
        resolve_depth_buffer(device);
    }
    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d8_device_GetRenderState(iface: *mut IDirect3DDevice8, state: D3DRENDERSTATETYPE, value: *mut DWORD) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut device_state: *const wined3d_stateblock_state = core::ptr::null();

    wined3d_mutex_lock();
    device_state = (*device).stateblock_state;
    match state as u32 {
        D3DRS_ZBIAS => {
            *value = (*device_state).rs[WINED3D_RS_DEPTHBIAS as usize] as DWORD;
        }
        _ => {
            *value = (*device_state).rs[state as usize] as DWORD;
        }
    }
    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_BeginStateBlock(iface: *mut IDirect3DDevice8) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut stateblock: *mut wined3d_stateblock = core::ptr::null_mut();
    let mut hr: HRESULT;

    wined3d_mutex_lock();
    if !(*device).recording.is_null() {
        wined3d_mutex_unlock();
        WARN!("Trying to begin a stateblock while recording, returning D3DERR_INVALIDCALL.\n");
        return D3DERR_INVALIDCALL;
    }

    hr = wined3d_stateblock_create(
        (*device).wined3d_device,
        core::ptr::null_mut(),
        WINED3D_SBT_RECORDED,
        core::ptr::addr_of_mut!(stateblock),
    );
    if hr >= 0 {
        (*device).recording = stateblock;
        (*device).update_state = stateblock;
    }
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d8_device_EndStateBlock(iface: *mut IDirect3DDevice8, token: *mut DWORD) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let stateblock: *mut wined3d_stateblock;

    /* Tell wineD3D to endstateblock before anything else (in case we run out
     * of memory later and cause locking problems)
     */
    wined3d_mutex_lock();
    if (*device).recording.is_null() {
        wined3d_mutex_unlock();
        WARN!("Trying to end a stateblock, but no stateblock is being recorded.\n");
        return D3DERR_INVALIDCALL;
    }
    stateblock = (*device).recording;
    wined3d_stateblock_init_contained_states(stateblock);
    (*device).recording = core::ptr::null_mut();
    (*device).update_state = (*device).state;

    *token = d3d8_allocate_handle(core::ptr::addr_of_mut!((*device).handle_table), (stateblock).cast(), D3D8_HANDLE_SB);
    wined3d_mutex_unlock();

    if *token == (D3D8_INVALID_HANDLE) as u32 {
        ERR!("Failed to create a handle\n");
        wined3d_mutex_lock();
        wined3d_stateblock_decref(stateblock);
        wined3d_mutex_unlock();
        return E_FAIL;
    }
    *token = (*token).wrapping_add(1);

    return D3D_OK;
}
unsafe extern "system" fn d3d8_device_ApplyStateBlock(iface: *mut IDirect3DDevice8, token: DWORD) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut state: *const wined3d_stateblock_state = core::ptr::null();
    let mut vertex_buffer: *mut d3d8_vertexbuffer = core::ptr::null_mut();
    let mut stateblock: *mut wined3d_stateblock = core::ptr::null_mut();
    let mut index_buffer: *mut d3d8_indexbuffer = core::ptr::null_mut();
    let mut wined3d_buffer: *mut wined3d_buffer = core::ptr::null_mut();
    let mut i: ::core::ffi::c_uint = 0;

    if token == 0 {
        return D3D_OK;
    }

    wined3d_mutex_lock();
    if !(*device).recording.is_null() {
        wined3d_mutex_unlock();
        WARN!("Trying to apply stateblock while recording, returning D3DERR_INVALIDCALL.\n");
        return D3DERR_INVALIDCALL;
    }
    stateblock = d3d8_get_object(
        core::ptr::addr_of_mut!((*device).handle_table),
        token.wrapping_sub(1),
        D3D8_HANDLE_SB,
    ) as *mut wined3d_stateblock;
    if stateblock.is_null() {
        WARN!("Invalid handle (%#lx) passed.\n", token);
        wined3d_mutex_unlock();
        return D3DERR_INVALIDCALL;
    }
    wined3d_stateblock_apply(stateblock, (*device).state);
    (*device).set_sysmem_vb(0 as DWORD);
    state = (*device).stateblock_state;
    i = 0 as ::core::ffi::c_uint;
    while i < D3D8_MAX_STREAMS as ::core::ffi::c_uint {
        wined3d_buffer = (*state).streams[i as usize].buffer as *mut wined3d_buffer;
        if !(wined3d_buffer.is_null() || {
            vertex_buffer = wined3d_buffer_get_parent(wined3d_buffer) as *mut d3d8_vertexbuffer;
            vertex_buffer.is_null()
        }) {
            if !(*vertex_buffer).draw_buffer.is_null() {
                (*device).set_sysmem_vb(
                    (*device).sysmem_vb() | ((1 as ::core::ffi::c_uint) << i) as DWORD,
                );
            }
        }
        i = i.wrapping_add(1);
    }
    wined3d_buffer = (*state).index_buffer as *mut wined3d_buffer;
    let ib_ok: ::core::ffi::c_int = (!wined3d_buffer.is_null()
        && {
            index_buffer = wined3d_buffer_get_parent(wined3d_buffer) as *mut d3d8_indexbuffer;
            !index_buffer.is_null()
        }
        && (*index_buffer).sysmem as ::core::ffi::c_int != 0) as ::core::ffi::c_int;
    (*device).set_sysmem_ib(ib_ok as DWORD);
    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d8_device_CaptureStateBlock(iface: *mut IDirect3DDevice8, token: DWORD) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut stateblock: *mut wined3d_stateblock = core::ptr::null_mut();

    wined3d_mutex_lock();
    if !(*device).recording.is_null() {
        wined3d_mutex_unlock();
        WARN!("Trying to capture stateblock while recording, returning D3DERR_INVALIDCALL.\n");
        return D3DERR_INVALIDCALL;
    }
    stateblock = d3d8_get_object(
        core::ptr::addr_of_mut!((*device).handle_table),
        token.wrapping_sub(1),
        D3D8_HANDLE_SB,
    ) as *mut wined3d_stateblock;
    if stateblock.is_null() {
        WARN!("Invalid handle (%#lx) passed.\n", token);
        wined3d_mutex_unlock();
        return D3DERR_INVALIDCALL;
    }
    wined3d_stateblock_capture(stateblock, (*device).state);
    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_DeleteStateBlock(iface: *mut IDirect3DDevice8, token: DWORD) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut stateblock: *mut wined3d_stateblock;

    TRACE!("iface %p, token %#lx.\n", iface, token);
    wined3d_mutex_lock();
    stateblock = d3d8_free_handle(
        core::ptr::addr_of_mut!((*device).handle_table),
        token.wrapping_sub(1),
        D3D8_HANDLE_SB,
    ) as *mut wined3d_stateblock;

    if stateblock.is_null() {
        WARN!("Invalid handle (%#lx) passed.\n", token);
        wined3d_mutex_unlock();
        return D3DERR_INVALIDCALL;
    }

    if wined3d_stateblock_decref(stateblock) != 0 {
        ERR!("Stateblock %p has references left, this shouldn't happen.\n", stateblock);
    }
    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_CreateStateBlock(iface: *mut IDirect3DDevice8, type_: D3DSTATEBLOCKTYPE, handle: *mut DWORD) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut stateblock: *mut wined3d_stateblock = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    if type_ != D3DSBT_ALL
        && type_ != D3DSBT_PIXELSTATE
        && type_ != D3DSBT_VERTEXSTATE
    {
        WARN!("Unexpected stateblock type, returning D3DERR_INVALIDCALL\n");
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    if !(*device).recording.is_null() {
        wined3d_mutex_unlock();
        WARN!("Trying to create a stateblock while recording, returning D3DERR_INVALIDCALL.\n");
        return D3DERR_INVALIDCALL;
    }
    hr = wined3d_stateblock_create(
        (*device).wined3d_device,
        (*device).state,
        type_ as wined3d_stateblock_type,
        core::ptr::addr_of_mut!(stateblock),
    );
    if (hr) < 0 {
        wined3d_mutex_unlock();
        ERR!("Failed to create the state block, hr %#lx.\n", hr);
        return hr;
    }

    *handle = d3d8_allocate_handle(
        core::ptr::addr_of_mut!((*device).handle_table),
        stateblock as *mut core::ffi::c_void,
        D3D8_HANDLE_SB,
    );
    wined3d_mutex_unlock();

    if *handle == (D3D8_INVALID_HANDLE) as u32 {
        ERR!("Failed to allocate a handle.\n");
        wined3d_mutex_lock();
        wined3d_stateblock_decref(stateblock);
        wined3d_mutex_unlock();
        return E_FAIL;
    }
    *handle = (*handle).wrapping_add(1);

    return hr;
}

unsafe extern "system" fn d3d8_device_SetClipStatus(iface: *mut IDirect3DDevice8, clip_status: *const D3DCLIPSTATUS8) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut hr: HRESULT = 0;

    TRACE!("iface %p, clip_status %p.\n", iface, clip_status);

    /* FIXME: Verify that D3DCLIPSTATUS8 ~= struct wined3d_clip_status. */

    wined3d_mutex_lock();
    hr = wined3d_device_set_clip_status((*device).wined3d_device, clip_status as *const wined3d_clip_status);
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d8_device_GetClipStatus(iface: *mut IDirect3DDevice8, clip_status: *mut D3DCLIPSTATUS8) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut hr: HRESULT;

    wined3d_mutex_lock();
    hr = wined3d_device_get_clip_status((*device).wined3d_device, clip_status as *mut wined3d_clip_status);
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d8_device_GetTexture(iface: *mut IDirect3DDevice8, stage: DWORD, texture: *mut *mut IDirect3DBaseTexture8) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut wined3d_texture: *mut wined3d_texture = core::ptr::null_mut::<wined3d_texture>();
    let mut texture_impl: *mut d3d8_texture = core::ptr::null_mut::<d3d8_texture>();

    TRACE!("iface %p, stage %lu, texture %p.\n", iface, stage, texture);

    if texture.is_null() {
        return D3DERR_INVALIDCALL;
    }

    if stage >= (WINED3D_MAX_FRAGMENT_SAMPLERS as DWORD) {
        WARN!("Ignoring invalid stage %lu.\n", stage);
        *texture = core::ptr::null_mut::<IDirect3DBaseTexture8>();
        return D3D_OK;
    }

    wined3d_mutex_lock();
    wined3d_texture = (*(*device).stateblock_state).textures[stage as usize] as *mut wined3d_texture;
    if !wined3d_texture.is_null() {
        texture_impl = wined3d_texture_get_parent(wined3d_texture) as *mut d3d8_texture;
        *texture = core::ptr::addr_of_mut!((*texture_impl).IDirect3DBaseTexture8_iface);
        IDirect3DBaseTexture8_AddRef(*texture);
    } else {
        *texture = core::ptr::null_mut::<IDirect3DBaseTexture8>();
    }
    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_SetTexture(iface: *mut IDirect3DDevice8, stage: DWORD, texture: *mut IDirect3DBaseTexture8) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut texture_impl: *mut d3d8_texture;

    texture_impl = unsafe_impl_from_IDirect3DBaseTexture8(texture);

    wined3d_mutex_lock();
    wined3d_stateblock_set_texture((*device).update_state, stage,
            if !texture_impl.is_null() { d3d8_texture_get_draw_texture(texture_impl) } else { core::ptr::null_mut() });
    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d8_device_GetTextureStageState(
    iface: *mut IDirect3DDevice8,
    stage: DWORD,
    state: D3DTEXTURESTAGESTATETYPE,
    value: *mut DWORD,
) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut device_state: *const wined3d_stateblock_state = core::ptr::null();
    let l: *const tss_lookup;

    if state as usize >= ARRAY_SIZE!(tss_lookup) as usize {
        WARN!("Invalid state %#x.\n", state);
        return D3D_OK;
    }

    l = (core::ptr::addr_of!(tss_lookup) as *const tss_lookup).add(state as usize);

    // Divergence: the C indexes both arrays unchecked -- a stage past the state block reads
    // its neighbour, and the two unused slots (0 and 12) map to WINED3D_TSS_INVALID (~0u),
    // so GetTextureStageState(0, 0) reads texture_states[0][0xffffffff] and the C oracle
    // faults (modrun's d3d8 device case). Both are invalid here, like an invalid state.
    device_state = (*device).stateblock_state;
    let ok = if (*l).sampler_state != 0 {
        let s = &(*device_state).sampler_states;
        (stage as usize) < s.len() && ((*l).u.sampler_state as usize) < s[0].len()
    } else {
        let s = &(*device_state).texture_states;
        (stage as usize) < s.len() && ((*l).u.texture_state as usize) < s[0].len()
    };
    if !ok {
        WARN!("Invalid stage %u or state %#x.\n", stage, state);
        return D3D_OK;
    }

    wined3d_mutex_lock();
    if (*l).sampler_state != 0 {
        *value =
            (*device_state).sampler_states[stage as usize][(*l).u.sampler_state as usize] as DWORD;
    } else {
        *value =
            (*device_state).texture_states[stage as usize][(*l).u.texture_state as usize] as DWORD;
    }
    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d8_device_SetTextureStageState(
    mut iface: *mut IDirect3DDevice8,
    mut stage: DWORD,
    mut type_0: D3DTEXTURESTAGESTATETYPE,
    mut value: DWORD,
) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut l: *const tss_lookup;

    if type_0 as usize >= ARRAY_SIZE!(tss_lookup) as usize {
        WARN!("Invalid type %#x passed.\n", type_0);
        return D3D_OK;
    }

    l = (core::ptr::addr_of!(tss_lookup) as *const tss_lookup).add(type_0 as usize) as *const tss_lookup;

    wined3d_mutex_lock();
    if (*l).sampler_state != 0 {
        wined3d_stateblock_set_sampler_state(
            (*device).update_state,
            stage as UINT,
            (*l).u.sampler_state,
            value as core::ffi::c_uint,
        );
    } else {
        wined3d_stateblock_set_texture_stage_state(
            (*device).update_state,
            stage as UINT,
            (*l).u.texture_state,
            value as core::ffi::c_uint,
        );
    }
    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_ValidateDevice(iface: *mut IDirect3DDevice8, pass_count: *mut DWORD) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut hr: HRESULT;

    wined3d_mutex_lock();
    hr = wined3d_device_validate_device((*device).wined3d_device, (*device).stateblock_state, pass_count);
    wined3d_mutex_unlock();

    /* In d3d8, texture filters are not validated, so errors concerning
     * unsupported ones are ignored here. */
    if hr == WINED3DERR_UNSUPPORTEDTEXTUREFILTER {
        WARN!("Ignoring invalid texture filter settings.\n");
        *pass_count = 1;
        return D3D_OK;
    }

    return hr;
}
unsafe extern "system" fn d3d8_device_GetInfo(
    iface: *mut IDirect3DDevice8,
    info_id: DWORD,
    info: *mut core::ffi::c_void,
    info_size: DWORD,
) -> HRESULT {
    TRACE!(
        "iface %p, info_id %#lx, info %p, info_size %lu.\n",
        iface,
        info_id,
        info,
        info_size
    );

    if info_id < 4 {
        return E_FAIL;
    }
    return S_FALSE;
}
unsafe extern "system" fn d3d8_device_SetPaletteEntries(
    iface: *mut IDirect3DDevice8,
    palette_idx: UINT,
    entries: *const PALETTEENTRY,
) -> HRESULT {
    WARN!("iface %p, palette_idx %u, entries %p unimplemented\n", iface, palette_idx, entries);
    D3D_OK
}
unsafe extern "system" fn d3d8_device_GetPaletteEntries(
    iface: *mut IDirect3DDevice8,
    palette_idx: UINT,
    entries: *mut PALETTEENTRY,
) -> HRESULT {
    // FIXME("iface %p, palette_idx %u, entries %p unimplemented.\n", iface, palette_idx, entries);
    D3DERR_INVALIDCALL
}
unsafe extern "system" fn d3d8_device_SetCurrentTexturePalette(
    iface: *mut IDirect3DDevice8,
    palette_idx: UINT,
) -> HRESULT {
    WARN!("iface %p, palette_idx %u unimplemented.\n", iface, palette_idx);
    D3D_OK
}
unsafe extern "system" fn d3d8_device_GetCurrentTexturePalette(
    iface: *mut IDirect3DDevice8,
    palette_idx: *mut UINT,
) -> HRESULT {
    // FIXME("iface %p, palette_idx %p unimplemented.\n", iface, palette_idx);
    D3DERR_INVALIDCALL
}

unsafe extern "C" fn d3d8_device_upload_sysmem_vertex_buffers(
    device: *mut d3d8_device,
    start_vertex: ::core::ffi::c_uint,
    vertex_count: ::core::ffi::c_uint,
) {
    let state: *const wined3d_stateblock_state = (*device).stateblock_state;
    let mut wined3d_decl: *mut wined3d_vertex_declaration;
    let mut box_: wined3d_box = wined3d_box {
        left: 0,
        top: 0,
        right: 0,
        bottom: 1,
        front: 0,
        back: 1,
    };
    let mut stream: *const wined3d_stream_state;
    let mut d3d8_buffer: *mut d3d8_vertexbuffer;
    let mut dst_resource: *mut wined3d_resource;
    let mut decl: *mut d3d8_vertex_declaration;
    let mut dst_buffer: *mut wined3d_buffer;
    let mut desc: wined3d_resource_desc = core::mem::zeroed();
    let mut stride: ::core::ffi::c_uint;
    let mut map: ::core::ffi::c_uint;
    let mut hr: HRESULT;

    if (*device).sysmem_vb() == 0 {
        return;
    }

    wined3d_decl = (*state).vertex_declaration as *mut wined3d_vertex_declaration;
    if wined3d_decl.is_null() {
        return;
    }

    decl = wined3d_vertex_declaration_get_parent(wined3d_decl) as *mut d3d8_vertex_declaration;
    map = ((*decl).stream_map & (*device).sysmem_vb()) as ::core::ffi::c_uint;
    while map != 0 {
        stream = ((&raw const (*state).streams) as *const wined3d_stream_state)
            .add(wined3d_bit_scan(core::ptr::addr_of_mut!(map)) as usize);
        dst_buffer = (*stream).buffer as *mut wined3d_buffer;
        stride = (*stream).stride as ::core::ffi::c_uint;

        d3d8_buffer = wined3d_buffer_get_parent(dst_buffer) as *mut d3d8_vertexbuffer;
        dst_resource = wined3d_buffer_get_resource(dst_buffer);
        wined3d_resource_get_desc(dst_resource, core::ptr::addr_of_mut!(desc));
        box_.left = ((*stream).offset as ::core::ffi::c_uint)
            .wrapping_add(start_vertex.wrapping_mul(stride)) as UINT;
        box_.right = min!(
            (box_.left as ::core::ffi::c_uint).wrapping_add(vertex_count.wrapping_mul(stride)),
            desc.size,
        ) as UINT;
        hr = wined3d_device_context_copy_sub_resource_region(
            (*device).immediate_context,
            dst_resource,
            0,
            box_.left,
            0,
            0,
            wined3d_buffer_get_resource((*d3d8_buffer).wined3d_buffer),
            0,
            core::ptr::addr_of_mut!(box_),
            0,
        );
        if hr < 0 {
            ERR!("Failed to update buffer.\n");
        }
    }
}

unsafe extern "C" fn d3d8_device_upload_sysmem_index_buffer(
    device: *mut d3d8_device,
    start_idx: *mut core::ffi::c_uint,
    idx_count: core::ffi::c_uint,
) -> HRESULT {
    let state: *const wined3d_stateblock_state = (*device).stateblock_state;
    let mut src_offset: core::ffi::c_uint = 0;
    let mut idx_size: core::ffi::c_uint = 0;
    let mut buffer_size: core::ffi::c_uint = 0;
    let mut pos: core::ffi::c_uint = 0;
    let mut resource_desc: wined3d_resource_desc = core::mem::zeroed();
    let mut index_buffer: *mut wined3d_resource = core::ptr::null_mut();
    let mut map_desc: wined3d_map_desc = core::mem::zeroed();
    let mut box_: wined3d_box = core::mem::zeroed();
    let mut hr: HRESULT = 0;

    if (*device).sysmem_ib() == 0 {
        return S_OK;
    }
    index_buffer = wined3d_buffer_get_resource((*state).index_buffer);
    wined3d_resource_get_desc(index_buffer, core::ptr::addr_of_mut!(resource_desc));
    idx_size = if (*state).index_format == WINED3DFMT_R16_UINT {
        2
    } else {
        4
    };

    src_offset = (*start_idx).wrapping_mul(idx_size);
    buffer_size = min!(
        idx_count.wrapping_mul(idx_size),
        resource_desc.size.wrapping_sub(src_offset),
    );

    wined3d_box_set(
        core::ptr::addr_of_mut!(box_),
        src_offset,
        0,
        buffer_size,
        1,
        0,
        1,
    );
    hr = wined3d_resource_map(
        index_buffer,
        0,
        core::ptr::addr_of_mut!(map_desc),
        core::ptr::addr_of_mut!(box_),
        WINED3D_MAP_READ,
    );
    if hr < 0 {
        ERR!("Failed to map index buffer, hr %#lx.\n", hr);
        return hr;
    }
    wined3d_streaming_buffer_upload(
        (*device).wined3d_device,
        core::ptr::addr_of_mut!((*device).index_buffer),
        map_desc.data,
        buffer_size,
        idx_size,
        core::ptr::addr_of_mut!(pos),
    );
    wined3d_resource_unmap(index_buffer, 0);

    wined3d_device_context_set_index_buffer(
        (*device).immediate_context,
        (*device).index_buffer.buffer,
        (*state).index_format,
        pos,
    );
    *start_idx = 0;
    return S_OK;
}

unsafe extern "C" fn d3d8_device_upload_managed_textures(device: *mut d3d8_device) {
    let state: *const wined3d_stateblock_state = (*device).stateblock_state;
    let mut i: ::core::ffi::c_uint = 0;
    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_MAX_FRAGMENT_SAMPLERS as ::core::ffi::c_uint {
        'body: {
            let wined3d_texture: *mut wined3d_texture = (*state).textures[i as usize];
            let mut d3d8_texture: *mut d3d8_texture = ::core::ptr::null_mut::<d3d8_texture>();

            if wined3d_texture.is_null() {
                break 'body;
            }
            d3d8_texture = wined3d_texture_get_parent(wined3d_texture) as *mut d3d8_texture;
            if !(*d3d8_texture).draw_texture.is_null() {
                wined3d_device_update_texture(
                    (*device).wined3d_device,
                    (*d3d8_texture).wined3d_texture,
                    (*d3d8_texture).draw_texture,
                );
            }
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "system" fn d3d8_device_DrawPrimitive(
    iface: *mut IDirect3DDevice8,
    primitive_type: D3DPRIMITIVETYPE,
    start_vertex: UINT,
    primitive_count: UINT,
) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut vb: *mut d3d8_vertexbuffer = core::ptr::null_mut();
    let mut vertex_count: u32;
    let mut i: u32;

    WINE_TRACE!("iface %p, primitive_type %#x, start_vertex %u, primitive_count %u.\n", iface, primitive_type as u32, start_vertex, primitive_count);

    vertex_count = vertex_count_from_primitive_count(primitive_type, primitive_count) as u32;
    wined3d_mutex_lock();
    d3d8_device_upload_managed_textures(device);
    d3d8_device_upload_sysmem_vertex_buffers(device, start_vertex, vertex_count);
    wined3d_device_context_set_primitive_type(
        (*device).immediate_context,
        wined3d_primitive_type_from_d3d(primitive_type),
        0,
    );
    wined3d_device_apply_stateblock((*device).wined3d_device, (*device).state);
    wined3d_device_context_draw(
        (*device).immediate_context,
        start_vertex,
        vertex_count,
        0,
        0,
    );

    i = 0;
    while i < ARRAY_SIZE!((*(*device).stateblock_state).streams) {
        if !(*(*device).stateblock_state).streams[i as usize]
            .buffer
            .is_null()
        {
            vb = wined3d_buffer_get_parent((*(*device).stateblock_state).streams[i as usize].buffer)
                as *mut d3d8_vertexbuffer;
            (*vb).discarded = false;
        }
        i = i.wrapping_add(1);
    }

    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_DrawIndexedPrimitive(
    iface: *mut IDirect3DDevice8,
    primitive_type: D3DPRIMITIVETYPE,
    min_vertex_idx: UINT,
    vertex_count: UINT,
    mut start_idx: UINT,
    primitive_count: UINT,
) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut vb: *mut d3d8_vertexbuffer = core::ptr::null_mut();
    let mut ib: *mut d3d8_indexbuffer = core::ptr::null_mut();
    let mut index_count: UINT;
    let mut i: UINT;
    let mut base_vertex_index: core::ffi::c_int;
    let mut hr: HRESULT;

    index_count = vertex_count_from_primitive_count(primitive_type, primitive_count);
    wined3d_mutex_lock();
    if (*(*device).stateblock_state).index_buffer.is_null() {
        wined3d_mutex_unlock();
        WARN!("Index buffer not set, returning D3D_OK.\n");
        return D3D_OK;
    }
    base_vertex_index = (*(*device).stateblock_state).base_vertex_index;
    d3d8_device_upload_managed_textures(device);
    d3d8_device_upload_sysmem_vertex_buffers(
        device,
        (base_vertex_index as UINT).wrapping_add(min_vertex_idx),
        vertex_count,
    );
    wined3d_device_context_set_primitive_type(
        (*device).immediate_context,
        wined3d_primitive_type_from_d3d(primitive_type),
        0,
    );
    wined3d_device_apply_stateblock((*device).wined3d_device, (*device).state);
    hr = d3d8_device_upload_sysmem_index_buffer(device, core::ptr::addr_of_mut!(start_idx), index_count);
    if hr < 0 {
        wined3d_mutex_unlock();
        return hr;
    }
    wined3d_device_context_draw_indexed(
        (*device).immediate_context,
        base_vertex_index,
        start_idx,
        index_count,
        0,
        0,
    );

    ib = wined3d_buffer_get_parent((*(*device).stateblock_state).index_buffer) as *mut d3d8_indexbuffer;
    (*ib).discarded = false;
    i = 0;
    while i < ARRAY_SIZE!((*(*device).stateblock_state).streams) {
        if !(*(*device).stateblock_state).streams[i as usize]
            .buffer
            .is_null()
        {
            vb = wined3d_buffer_get_parent((*(*device).stateblock_state).streams[i as usize].buffer)
                as *mut d3d8_vertexbuffer;
            (*vb).discarded = false;
        }
        i = i.wrapping_add(1);
    }

    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_DrawPrimitiveUP(
    iface: *mut IDirect3DDevice8,
    primitive_type: D3DPRIMITIVETYPE,
    primitive_count: UINT,
    data: *const core::ffi::c_void,
    stride: UINT,
) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut vtx_count: UINT = vertex_count_from_primitive_count(primitive_type, primitive_count);
    let mut size: UINT = vtx_count.wrapping_mul(stride);
    let mut vb_pos: UINT = 0;
    let mut hr: HRESULT = 0;

    if primitive_count == 0 || stride == 0 {
        WARN!("primitive_count or stride is 0, returning D3D_OK.\n");
        return D3D_OK;
    }

    wined3d_mutex_lock();

    'done: {
        hr = wined3d_streaming_buffer_upload(
            (*device).wined3d_device,
            core::ptr::addr_of_mut!((*device).vertex_buffer),
            data,
            size,
            stride,
            core::ptr::addr_of_mut!(vb_pos),
        );
        if hr < 0 {
            break 'done;
        }

        hr = wined3d_stateblock_set_stream_source(
            (*device).state,
            0,
            (*device).vertex_buffer.buffer,
            0,
            stride,
        );
        if hr < 0 {
            break 'done;
        }

        d3d8_device_upload_managed_textures(device);
        wined3d_device_context_set_primitive_type(
            (*device).immediate_context,
            wined3d_primitive_type_from_d3d(primitive_type),
            0,
        );
        wined3d_device_apply_stateblock((*device).wined3d_device, (*device).state);
        wined3d_device_context_draw(
            (*device).immediate_context,
            vb_pos / stride,
            vtx_count,
            0,
            0,
        );
        wined3d_stateblock_set_stream_source(
            (*device).state,
            0,
            core::ptr::null_mut(),
            0,
            0,
        );
    }

    wined3d_mutex_unlock();
    return hr;
}
unsafe extern "system" fn d3d8_device_DrawIndexedPrimitiveUP(iface: *mut IDirect3DDevice8, primitive_type: D3DPRIMITIVETYPE, min_vertex_idx: UINT, vertex_count: UINT, primitive_count: UINT, index_data: *const core::ffi::c_void, index_format: D3DFORMAT, vertex_data: *const core::ffi::c_void, vertex_stride: UINT) -> HRESULT {
    let idx_count: UINT = vertex_count_from_primitive_count(primitive_type, primitive_count);
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let idx_fmt_size: UINT = if index_format == D3DFMT_INDEX16 { 2 } else { 4 };
    let mut base_vertex_idx: UINT = 0;
    let mut vb_pos: UINT = 0;
    let mut ib_pos: UINT = 0;
    let vtx_size: UINT = vertex_count.wrapping_mul(vertex_stride);
    let idx_size: UINT = idx_count.wrapping_mul(idx_fmt_size);
    let mut hr: HRESULT = 0;

    TRACE!("iface %p, primitive_type %#x, min_vertex_idx %u, vertex_count %u, primitive_count %u, index_data %p, index_format %#x, vertex_data %p, vertex_stride %u.\n", iface, primitive_type as u32, min_vertex_idx, vertex_count, primitive_count, index_data, index_format as u32, vertex_data, vertex_stride);

    if primitive_count == 0 || vertex_stride == 0 {
        WARN!("primitive_count or vertex_stride is 0, returning D3D_OK.\n");
        return D3D_OK;
    }

    wined3d_mutex_lock();

    'done: {
        hr = wined3d_streaming_buffer_upload(
            (*device).wined3d_device,
            &raw mut (*device).vertex_buffer,
            (vertex_data as *mut core::ffi::c_char).offset(min_vertex_idx.wrapping_mul(vertex_stride) as isize) as *const core::ffi::c_void,
            vtx_size,
            vertex_stride,
            &raw mut vb_pos,
        );
        if hr < 0 {
            break 'done;
        }

        hr = wined3d_streaming_buffer_upload(
            (*device).wined3d_device,
            &raw mut (*device).index_buffer,
            index_data,
            idx_size,
            idx_fmt_size,
            &raw mut ib_pos,
        );
        if hr < 0 {
            break 'done;
        }

        hr = wined3d_stateblock_set_stream_source((*device).state, 0, (*device).vertex_buffer.buffer, 0, vertex_stride);
        if hr < 0 {
            break 'done;
        }

        wined3d_stateblock_set_index_buffer(
            (*device).state,
            (*device).index_buffer.buffer,
            wined3dformat_from_d3dformat(index_format),
        );
        base_vertex_idx = vb_pos.wrapping_div(vertex_stride).wrapping_sub(min_vertex_idx);
        wined3d_stateblock_set_base_vertex_index((*device).state, base_vertex_idx as INT);

        d3d8_device_upload_managed_textures(device);
        wined3d_device_context_set_primitive_type(
            (*device).immediate_context,
            wined3d_primitive_type_from_d3d(primitive_type),
            0,
        );
        wined3d_device_apply_stateblock((*device).wined3d_device, (*device).state);
        wined3d_device_context_draw_indexed(
            (*device).immediate_context,
            base_vertex_idx as INT,
            ib_pos.wrapping_div(idx_fmt_size),
            idx_count,
            0,
            0,
        );

        wined3d_stateblock_set_stream_source(
            (*device).state,
            0,
            core::ptr::null_mut(),
            0,
            0,
        );
        wined3d_stateblock_set_index_buffer(
            (*device).state,
            core::ptr::null_mut(),
            WINED3DFMT_UNKNOWN,
        );
        wined3d_stateblock_set_base_vertex_index((*device).state, 0);
    }

    wined3d_mutex_unlock();
    return hr;
}
unsafe extern "system" fn d3d8_device_ProcessVertices(iface: *mut IDirect3DDevice8, src_start_idx: UINT, dst_idx: UINT, vertex_count: UINT, dst_buffer: *mut IDirect3DVertexBuffer8, flags: DWORD) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut dst: *mut d3d8_vertexbuffer = unsafe_impl_from_IDirect3DVertexBuffer8(dst_buffer);
    let mut state: *const wined3d_stateblock_state = core::ptr::null();
    let mut stream: *const wined3d_stream_state = core::ptr::null();
    let mut d3d8_buffer: *mut d3d8_vertexbuffer = core::ptr::null_mut();
    let mut i: u32;
    let mut map: u32;
    let mut hr: HRESULT;

    TRACE!("iface %p, src_start_idx %u, dst_idx %u, vertex_count %u, dst_buffer %p, flags %#lx.\n",
        iface, src_start_idx, dst_idx, vertex_count, dst_buffer, flags);

    wined3d_mutex_lock();

    state = (*device).stateblock_state;

    /* Note that an alternative approach would be to simply create these
     * buffers with WINED3D_RESOURCE_ACCESS_MAP_R and update them here like we
     * do for draws. In some regards that would be easier, but it seems less
     * than optimal to upload data to the GPU only to subsequently download it
     * again. */
    map = (*device).sysmem_vb();
    while map != 0 {
        i = wined3d_bit_scan(core::ptr::addr_of_mut!(map)) as u32;
        stream = (core::ptr::addr_of!((*state).streams) as *const wined3d_stream_state).offset(i as isize);

        d3d8_buffer = wined3d_buffer_get_parent((*stream).buffer) as *mut d3d8_vertexbuffer;
        if (wined3d_stateblock_set_stream_source((*device).state,
                i as UINT, (*d3d8_buffer).wined3d_buffer, (*stream).offset, (*stream).stride) < 0)
        {
            ERR!("Failed to set stream source.\n");
        }
    }

    hr = wined3d_device_process_vertices((*device).wined3d_device, (*device).state, src_start_idx, dst_idx,
            vertex_count, (*dst).wined3d_buffer, core::ptr::null_mut(), flags, (*dst).fvf);

    map = (*device).sysmem_vb();
    while map != 0 {
        i = wined3d_bit_scan(core::ptr::addr_of_mut!(map)) as u32;
        stream = (core::ptr::addr_of!((*state).streams) as *const wined3d_stream_state).offset(i as isize);

        d3d8_buffer = wined3d_buffer_get_parent((*stream).buffer) as *mut d3d8_vertexbuffer;
        if (wined3d_stateblock_set_stream_source((*device).state,
                i as UINT, (*d3d8_buffer).draw_buffer, (*stream).offset, (*stream).stride) < 0)
        {
            ERR!("Failed to set stream source.\n");
        }
    }

    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d8_device_CreateVertexShader(iface: *mut IDirect3DDevice8, declaration: *const DWORD, byte_code: *const DWORD, shader: *mut DWORD, usage: DWORD) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut object: *mut d3d8_vertex_shader;
    let mut shader_handle: DWORD;
    let mut handle: DWORD;
    let mut hr: HRESULT;

    object = calloc(1, (core::mem::size_of::<d3d8_vertex_shader>()) as _) as *mut d3d8_vertex_shader;
    if object.is_null()
    {
        *shader = 0;
        return E_OUTOFMEMORY;
    }

    wined3d_mutex_lock();
    handle = d3d8_allocate_handle(core::ptr::addr_of_mut!((*device).handle_table), object as *mut core::ffi::c_void, D3D8_HANDLE_VS);
    wined3d_mutex_unlock();
    if handle == (D3D8_INVALID_HANDLE) as u32
    {
        ERR!("Failed to allocate vertex shader handle.\n");
        free(object as *mut core::ffi::c_void);
        *shader = 0;
        return E_OUTOFMEMORY;
    }

    shader_handle = handle.wrapping_add(VS_HIGHESTFIXEDFXF).wrapping_add(1);

    hr = d3d8_vertex_shader_init(object, device, declaration, byte_code, shader_handle, usage);
    if hr < 0
    {
        WARN!("Failed to initialize vertex shader, hr %#lx.\n", hr);
        wined3d_mutex_lock();
        d3d8_free_handle(core::ptr::addr_of_mut!((*device).handle_table), handle, D3D8_HANDLE_VS);
        wined3d_mutex_unlock();
        free(object as *mut core::ffi::c_void);
        *shader = 0;
        return hr;
    }

    *shader = shader_handle;

    return D3D_OK;
}

unsafe extern "C" fn d3d8_device_get_fvf_declaration(
    device: *mut d3d8_device,
    fvf: DWORD,
) -> *mut d3d8_vertex_declaration {
    let mut d3d8_declaration: *mut d3d8_vertex_declaration = core::ptr::null_mut();
    let mut convertedDecls: *mut FvfToDecl = (*device).decls;
    let mut p: i32;
    let mut low: i32;
    let mut high: i32;
    let mut hr: HRESULT;

    TRACE!("Searching for declaration for fvf %#lx... \n", fvf);

    low = 0;
    high = (*device).numConvertedDecls.wrapping_sub(1) as i32;
    while low <= high {
        p = (low + high) >> 1;

        TRACE!("%d \n", p);

        if (*convertedDecls.offset(p as isize)).fvf == fvf {
            TRACE!("found %p\n", (*convertedDecls.offset(p as isize)).declaration);
            return (*convertedDecls.offset(p as isize)).declaration as *mut d3d8_vertex_declaration;
        }

        if (*convertedDecls.offset(p as isize)).fvf < fvf {
            low = p + 1;
        } else {
            high = p - 1;
        }
    }

    TRACE!("not found. Creating and inserting at position %d.\n", low);

    d3d8_declaration = malloc(core::mem::size_of::<d3d8_vertex_declaration>() as size_t)
        as *mut d3d8_vertex_declaration;
    if d3d8_declaration.is_null() {
        return core::ptr::null_mut();
    }

    hr = d3d8_vertex_declaration_init_fvf(d3d8_declaration, device, fvf);
    if hr < 0 {
        WARN!("Failed to initialize vertex declaration, hr %#lx.\n", hr);
        free(d3d8_declaration as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    if (*device).declArraySize == (*device).numConvertedDecls {
        let mut grow: UINT = (*device).declArraySize.wrapping_div(2);

        convertedDecls = realloc(
            convertedDecls as *mut core::ffi::c_void,
            (core::mem::size_of::<FvfToDecl>() as size_t)
                .wrapping_mul((*device).numConvertedDecls.wrapping_add(grow) as size_t),
        ) as *mut FvfToDecl;
        if convertedDecls.is_null() {
            d3d8_vertex_declaration_destroy(d3d8_declaration);
            return core::ptr::null_mut();
        }
        (*device).decls = convertedDecls;
        (*device).declArraySize = (*device).declArraySize.wrapping_add(grow);
    }

    memmove(
        convertedDecls.offset(low as isize).offset(1) as *mut core::ffi::c_void,
        convertedDecls.offset(low as isize) as *const core::ffi::c_void,
        (core::mem::size_of::<FvfToDecl>() as size_t)
            .wrapping_mul((*device).numConvertedDecls.wrapping_sub(low as UINT) as size_t),
    );
    (*convertedDecls.offset(low as isize)).declaration = d3d8_declaration;
    (*convertedDecls.offset(low as isize)).fvf = fvf;
    (*device).numConvertedDecls = (*device).numConvertedDecls.wrapping_add(1);

    TRACE!(
        "Returning %p. %u decls in array.\n",
        d3d8_declaration,
        (*device).numConvertedDecls
    );

    return d3d8_declaration;
}

unsafe extern "system" fn d3d8_device_SetVertexShader(iface: *mut IDirect3DDevice8, shader: DWORD) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut shader_impl: *mut d3d8_vertex_shader = core::ptr::null_mut();

    if VS_HIGHESTFIXEDFXF >= shader {
        wined3d_mutex_lock();
        wined3d_stateblock_set_vertex_declaration(
            (*device).update_state,
            (*d3d8_device_get_fvf_declaration(device, shader)).wined3d_vertex_declaration,
        );
        wined3d_stateblock_set_vertex_shader((*device).update_state, core::ptr::null_mut());
        wined3d_mutex_unlock();

        return D3D_OK;
    }

    wined3d_mutex_lock();
    shader_impl = d3d8_get_object(
        core::ptr::addr_of_mut!((*device).handle_table),
        shader.wrapping_sub(VS_HIGHESTFIXEDFXF + 1),
        D3D8_HANDLE_VS,
    ) as *mut d3d8_vertex_shader;
    if shader_impl.is_null() {
        WARN!("Invalid handle (%#lx) passed.\n", shader);
        wined3d_mutex_unlock();

        return D3DERR_INVALIDCALL;
    }

    wined3d_stateblock_set_vertex_declaration(
        (*device).update_state,
        (*(*shader_impl).vertex_declaration).wined3d_vertex_declaration,
    );
    wined3d_stateblock_set_vertex_shader((*device).update_state, (*shader_impl).wined3d_shader);
    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_GetVertexShader(
    iface: *mut IDirect3DDevice8,
    shader: *mut DWORD,
) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut wined3d_declaration: *mut wined3d_vertex_declaration =
        core::ptr::null_mut::<wined3d_vertex_declaration>();
    let mut d3d8_declaration: *mut d3d8_vertex_declaration =
        core::ptr::null_mut::<d3d8_vertex_declaration>();

    WINE_TRACE!("iface %p, shader %p.\n", iface, shader);

    wined3d_mutex_lock();
    wined3d_declaration =
        (*(*device).stateblock_state).vertex_declaration as *mut wined3d_vertex_declaration;
    if !wined3d_declaration.is_null() {
        d3d8_declaration = wined3d_vertex_declaration_get_parent(wined3d_declaration)
            as *mut d3d8_vertex_declaration;
        *shader = (*d3d8_declaration).shader_handle;
    } else {
        *shader = 0;
    }
    wined3d_mutex_unlock();

    WINE_TRACE!("Returning %#lx.\n", *shader);

    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_DeleteVertexShader(iface: *mut IDirect3DDevice8, shader: DWORD) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut shader_impl: *mut d3d8_vertex_shader = core::ptr::null_mut();

    TRACE!("iface %p, shader %#lx.\n", iface, shader);

    wined3d_mutex_lock();
    shader_impl = d3d8_free_handle(
        core::ptr::addr_of_mut!((*device).handle_table),
        shader.wrapping_sub(VS_HIGHESTFIXEDFXF + 1),
        D3D8_HANDLE_VS,
    ) as *mut d3d8_vertex_shader;
    if shader_impl.is_null() {
        WARN!("Invalid handle (%#lx) passed.\n", shader);
        wined3d_mutex_unlock();

        return D3DERR_INVALIDCALL;
    }

    if !(*shader_impl).wined3d_shader.is_null()
        && (*(*device).stateblock_state).vs == (*shader_impl).wined3d_shader
    {
        IDirect3DDevice8_SetVertexShader(iface, 0);
    }

    wined3d_mutex_unlock();

    d3d8_vertex_shader_destroy(shader_impl);

    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_SetVertexShaderConstant(
    iface: *mut IDirect3DDevice8,
    start_register: DWORD,
    data: *const core::ffi::c_void,
    count: DWORD,
) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut hr: HRESULT;

    if start_register.wrapping_add(count) > D3D8_MAX_VERTEX_SHADER_CONSTANTF {
        WARN!(
            "Trying to access %lu constants, but d3d8 only supports %u.\n",
            start_register.wrapping_add(count),
            D3D8_MAX_VERTEX_SHADER_CONSTANTF
        );
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    hr = wined3d_stateblock_set_vs_consts_f(
        (*device).update_state,
        start_register,
        count,
        data as *const wined3d_vec4,
    );
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d8_device_GetVertexShaderConstant(
    iface: *mut IDirect3DDevice8,
    start_idx: DWORD,
    constants: *mut core::ffi::c_void,
    count: DWORD,
) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut hr: HRESULT;

    TRACE!(
        "iface %p, start_idx %lu, constants %p, count %lu.\n",
        iface,
        start_idx,
        constants,
        count
    );
    wined3d_mutex_lock();
    hr = wined3d_stateblock_get_vs_consts_f((*device).state, start_idx, count, (constants).cast());
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d8_device_GetVertexShaderDeclaration(
    iface: *mut IDirect3DDevice8,
    shader: DWORD,
    data: *mut ::core::ffi::c_void,
    data_size: *mut DWORD,
) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut declaration: *mut d3d8_vertex_declaration = core::ptr::null_mut();
    let mut shader_impl: *mut d3d8_vertex_shader = core::ptr::null_mut();

    TRACE!(
        "iface %p, shader %#lx, data %p, data_size %p.\n",
        iface,
        shader,
        data,
        data_size
    );
    wined3d_mutex_lock();
    shader_impl = d3d8_get_object(
        core::ptr::addr_of_mut!((*device).handle_table),
        shader.wrapping_sub(VS_HIGHESTFIXEDFXF + 1),
        D3D8_HANDLE_VS,
    ) as *mut d3d8_vertex_shader;
    wined3d_mutex_unlock();

    if shader_impl.is_null() {
        WARN!("Invalid handle (%#lx) passed.\n", shader);
        return D3DERR_INVALIDCALL;
    }
    declaration = (*shader_impl).vertex_declaration;

    if data.is_null() {
        *data_size = (*declaration).elements_size;
        return D3D_OK;
    }

    /* MSDN claims that if *data_size is smaller than the required size
     * we should write the required size and return D3DERR_MOREDATA.
     * That's not actually true. */
    if *data_size < (*declaration).elements_size {
        return D3DERR_INVALIDCALL;
    }

    memcpy(
        data,
        (*declaration).elements as *const ::core::ffi::c_void,
        (*declaration).elements_size as usize,
    );

    return D3D_OK;
}
unsafe extern "system" fn d3d8_device_GetVertexShaderFunction(
    iface: *mut IDirect3DDevice8,
    shader: DWORD,
    data: *mut core::ffi::c_void,
    data_size: *mut DWORD,
) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut shader_impl: *mut d3d8_vertex_shader = core::ptr::null_mut();
    let mut hr: HRESULT;

    wined3d_mutex_lock();
    shader_impl = d3d8_get_object(
        core::ptr::addr_of_mut!((*device).handle_table),
        shader.wrapping_sub(VS_HIGHESTFIXEDFXF + 1),
        D3D8_HANDLE_VS,
    ) as *mut d3d8_vertex_shader;
    if shader_impl.is_null() {
        WARN!("Invalid handle (%#lx) passed.\n", shader);
        wined3d_mutex_unlock();

        return D3DERR_INVALIDCALL;
    }

    if (*shader_impl).wined3d_shader.is_null() {
        wined3d_mutex_unlock();
        *data_size = 0;
        return D3D_OK;
    }

    hr = wined3d_shader_get_byte_code(
        (*shader_impl).wined3d_shader,
        data,
        data_size as *mut u32,
    );
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d8_device_SetIndices(
    mut iface: *mut IDirect3DDevice8,
    mut buffer: *mut IDirect3DIndexBuffer8,
    mut base_vertex_idx: UINT,
) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut ib: *mut d3d8_indexbuffer = unsafe_impl_from_IDirect3DIndexBuffer8(buffer);
    let mut wined3d_buffer: *mut wined3d_buffer = ::core::ptr::null_mut::<wined3d_buffer>();
    WINE_TRACE!("iface %p, buffer %p, base_vertex_idx %u.\n", iface, buffer, base_vertex_idx);
    if ib.is_null() {
        wined3d_buffer = ::core::ptr::null_mut::<wined3d_buffer>();
    } else {
        wined3d_buffer = (*ib).wined3d_buffer;
    }
    wined3d_mutex_lock();
    wined3d_stateblock_set_base_vertex_index((*device).update_state, base_vertex_idx as INT);
    wined3d_stateblock_set_index_buffer(
        (*device).update_state,
        wined3d_buffer,
        (if !ib.is_null() {
            (*ib).format as ::core::ffi::c_uint
        } else {
            WINED3DFMT_UNKNOWN as ::core::ffi::c_uint
        }) as wined3d_format_id,
    );
    if (*device).recording.is_null() {
        (*device).set_sysmem_ib(((!ib.is_null() && (*ib).sysmem as ::core::ffi::c_int != 0)
            as ::core::ffi::c_int as DWORD as DWORD) as _);
    }
    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_GetIndices(iface: *mut IDirect3DDevice8, buffer: *mut *mut IDirect3DIndexBuffer8, base_vertex_index: *mut UINT) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut state: *const wined3d_stateblock_state = core::ptr::null();
    let mut wined3d_buffer: *mut wined3d_buffer = core::ptr::null_mut();
    let mut buffer_impl: *mut d3d8_indexbuffer = core::ptr::null_mut();

    if buffer.is_null() {
        return D3DERR_INVALIDCALL;
    }

    /* The case from UINT to INT is safe because d3d8 will never set negative values */
    wined3d_mutex_lock();
    state = (*device).stateblock_state;
    *base_vertex_index = (*state).base_vertex_index as UINT;
    wined3d_buffer = (*state).index_buffer;
    if !wined3d_buffer.is_null() {
        buffer_impl = wined3d_buffer_get_parent(wined3d_buffer) as *mut d3d8_indexbuffer;
        *buffer = core::ptr::addr_of_mut!((*buffer_impl).IDirect3DIndexBuffer8_iface);
        IDirect3DIndexBuffer8_AddRef(*buffer);
    } else {
        *buffer = core::ptr::null_mut();
    }
    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_CreatePixelShader(
    iface: *mut IDirect3DDevice8,
    byte_code: *const DWORD,
    shader: *mut DWORD,
) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut object: *mut d3d8_pixel_shader;
    let mut shader_handle: DWORD;
    let mut handle: DWORD;
    let mut hr: HRESULT;

    if shader.is_null() {
        return D3DERR_INVALIDCALL;
    }

    object = calloc(1, (core::mem::size_of::<d3d8_pixel_shader>()) as _) as *mut d3d8_pixel_shader;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    wined3d_mutex_lock();
    handle = d3d8_allocate_handle(core::ptr::addr_of_mut!((*device).handle_table), object as *mut core::ffi::c_void, D3D8_HANDLE_PS);
    wined3d_mutex_unlock();
    if handle == (D3D8_INVALID_HANDLE) as u32 {
        ERR!("Failed to allocate pixel shader handle.\n");
        free(object as *mut core::ffi::c_void);
        return E_OUTOFMEMORY;
    }

    shader_handle = handle.wrapping_add(VS_HIGHESTFIXEDFXF).wrapping_add(1);

    hr = d3d8_pixel_shader_init(object, device, byte_code, shader_handle);
    if hr < 0 {
        WARN!("Failed to initialize pixel shader, hr %#lx.\n", hr);
        wined3d_mutex_lock();
        d3d8_free_handle(core::ptr::addr_of_mut!((*device).handle_table), handle, D3D8_HANDLE_PS);
        wined3d_mutex_unlock();
        free(object as *mut core::ffi::c_void);
        *shader = 0;
        return hr;
    }

    *shader = shader_handle;

    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_SetPixelShader(iface: *mut IDirect3DDevice8, shader: DWORD) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut shader_impl: *mut d3d8_pixel_shader = core::ptr::null_mut();

    TRACE!("iface %p, shader %#lx.\n", iface, shader);

    wined3d_mutex_lock();

    if shader == 0 {
        wined3d_stateblock_set_pixel_shader((*device).update_state, core::ptr::null_mut());
        wined3d_mutex_unlock();
        return D3D_OK;
    }

    shader_impl = d3d8_get_object(
        core::ptr::addr_of_mut!((*device).handle_table),
        shader.wrapping_sub(VS_HIGHESTFIXEDFXF.wrapping_add(1)),
        D3D8_HANDLE_PS,
    ) as *mut d3d8_pixel_shader;

    if shader_impl.is_null() {
        WARN!("Invalid handle (%#lx) passed.\n", shader);
        wined3d_mutex_unlock();
        return D3DERR_INVALIDCALL;
    }

    TRACE!("Setting shader %p.\n", shader_impl);

    wined3d_stateblock_set_pixel_shader((*device).update_state, (*shader_impl).wined3d_shader);
    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d8_device_GetPixelShader(iface: *mut IDirect3DDevice8, shader: *mut DWORD) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut object: *mut wined3d_shader = core::ptr::null_mut::<wined3d_shader>();

    if shader.is_null() {
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    object = (*(*device).stateblock_state).ps;
    if !object.is_null() {
        let mut d3d8_shader: *mut d3d8_pixel_shader = core::ptr::null_mut::<d3d8_pixel_shader>();
        d3d8_shader = wined3d_shader_get_parent(object) as *mut d3d8_pixel_shader;
        *shader = (*d3d8_shader).handle;
    } else {
        *shader = 0;
    }
    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_DeletePixelShader(iface: *mut IDirect3DDevice8, shader: DWORD) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut shader_impl: *mut d3d8_pixel_shader = core::ptr::null_mut();

    wined3d_mutex_lock();

    shader_impl = d3d8_free_handle(
        core::ptr::addr_of_mut!((*device).handle_table),
        shader.wrapping_sub(VS_HIGHESTFIXEDFXF.wrapping_add(1)),
        D3D8_HANDLE_PS,
    ) as *mut d3d8_pixel_shader;
    if shader_impl.is_null() {
        WARN!("Invalid handle (%#lx) passed.\n", shader);
        wined3d_mutex_unlock();
        return D3DERR_INVALIDCALL;
    }

    if (*(*device).stateblock_state).ps == (*shader_impl).wined3d_shader {
        IDirect3DDevice8_SetPixelShader(iface, 0 as DWORD);
    }

    wined3d_mutex_unlock();

    d3d8_pixel_shader_destroy(shader_impl);

    return D3D_OK;
}

unsafe extern "system" fn d3d8_device_SetPixelShaderConstant(
    mut iface: *mut IDirect3DDevice8,
    mut start_register: DWORD,
    mut data: *const ::core::ffi::c_void,
    mut count: DWORD,
) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, start_register %lu, data %p, count %lu.\n", iface, start_register, data, count);
    wined3d_mutex_lock();
    hr = wined3d_stateblock_set_ps_consts_f(
        (*device).update_state,
        start_register as ::core::ffi::c_uint,
        count as ::core::ffi::c_uint,
        data as *const wined3d_vec4,
    );
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d8_device_GetPixelShaderConstant(iface: *mut IDirect3DDevice8, start_idx: DWORD, constants: *mut core::ffi::c_void, count: DWORD) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut hr: HRESULT;

    if !wined3d_bound_range(start_idx, count, D3D8_MAX_PIXEL_SHADER_CONSTANTF) {
        return WINED3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    hr = wined3d_stateblock_get_ps_consts_f((*device).state, start_idx, count, (constants).cast());
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d8_device_GetPixelShaderFunction(
    mut iface: *mut IDirect3DDevice8,
    mut shader: DWORD,
    mut data: *mut core::ffi::c_void,
    mut data_size: *mut DWORD,
) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut shader_impl: *mut d3d8_pixel_shader = core::ptr::null_mut::<d3d8_pixel_shader>();
    let mut hr: HRESULT = 0;

    TRACE!("iface %p, shader %#lx, data %p, data_size %p.\n", iface, shader, data, data_size);

    wined3d_mutex_lock();
    shader_impl = d3d8_get_object(
        core::ptr::addr_of_mut!((*device).handle_table),
        shader.wrapping_sub(VS_HIGHESTFIXEDFXF + 1),
        D3D8_HANDLE_PS,
    ) as *mut d3d8_pixel_shader;
    if shader_impl.is_null() {
        WARN!("Invalid handle (%#lx) passed.\n", shader);
        wined3d_mutex_unlock();

        return D3DERR_INVALIDCALL;
    }

    hr = wined3d_shader_get_byte_code((*shader_impl).wined3d_shader, data, data_size as *mut UINT);
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d8_device_DrawRectPatch(
    iface: *mut IDirect3DDevice8,
    handle: UINT,
    segment_count: *const f32,
    patch_info: *const D3DRECTPATCH_INFO,
) -> HRESULT {
    // FIXME("iface %p, handle %#x, segment_count %p, patch_info %p unimplemented.\n", iface, handle, segment_count, patch_info);
    D3D_OK
}
unsafe extern "system" fn d3d8_device_DrawTriPatch(
    iface: *mut IDirect3DDevice8,
    handle: UINT,
    segment_count: *const f32,
    patch_info: *const D3DTRIPATCH_INFO,
) -> HRESULT {
    // FIXME("iface %p, handle %#x, segment_count %p, patch_info %p unimplemented.\n", iface, handle, segment_count, patch_info);
    D3D_OK
}
unsafe extern "system" fn d3d8_device_DeletePatch(
    iface: *mut IDirect3DDevice8,
    handle: UINT,
) -> HRESULT {
    // FIXME("iface %p, handle %#x unimplemented.\n", iface, handle);
    D3DERR_INVALIDCALL
}

unsafe extern "system" fn d3d8_device_SetStreamSource(
    iface: *mut IDirect3DDevice8,
    stream_idx: UINT,
    buffer: *mut IDirect3DVertexBuffer8,
    mut stride: UINT,
) -> HRESULT {
    let device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let buffer_impl: *mut d3d8_vertexbuffer = unsafe_impl_from_IDirect3DVertexBuffer8(buffer);
    let mut state: *const wined3d_stateblock_state = core::ptr::null();
    let mut wined3d_buffer: *mut wined3d_buffer = core::ptr::null_mut();
    let mut hr: HRESULT;

    TRACE!(
        "iface %p, stream_idx %u, buffer %p, stride %u.\n",
        iface,
        stream_idx,
        buffer,
        stride
    );

    if stream_idx as usize
        >= (core::mem::size_of::<[wined3d_stream_state; 16]>() as usize)
            .wrapping_div(core::mem::size_of::<wined3d_stream_state>() as usize)
    {
        WARN!("Stream index %u out of range.\n", stream_idx);
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();

    if buffer_impl.is_null() {
        wined3d_buffer = core::ptr::null_mut();
        state = (*device).stateblock_state;
        stride = (*state).streams[stream_idx as usize].stride;
    } else if !(*buffer_impl).draw_buffer.is_null() {
        wined3d_buffer = (*buffer_impl).draw_buffer;
    } else {
        wined3d_buffer = (*buffer_impl).wined3d_buffer;
    }

    hr = wined3d_stateblock_set_stream_source(
        (*device).update_state,
        stream_idx,
        wined3d_buffer,
        0,
        stride,
    );
    if hr >= 0 && (*device).recording.is_null() {
        if !buffer_impl.is_null() && !(*buffer_impl).draw_buffer.is_null() {
            (*device).set_sysmem_vb(((*device).sysmem_vb() | (((1 as DWORD) << stream_idx) as DWORD)) as _);
        } else {
            (*device).set_sysmem_vb(((*device).sysmem_vb() & (!((1 as DWORD) << stream_idx))) as _);
        }
    }

    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d8_device_GetStreamSource(
    iface: *mut IDirect3DDevice8,
    stream_idx: UINT,
    buffer: *mut *mut IDirect3DVertexBuffer8,
    stride: *mut UINT,
) -> HRESULT {
    let mut device: *mut d3d8_device = impl_from_IDirect3DDevice8(iface);
    let mut state: *const wined3d_stateblock_state = core::ptr::null();
    let mut stream: *const wined3d_stream_state = core::ptr::null();
    let mut buffer_impl: *mut d3d8_vertexbuffer = core::ptr::null_mut();

    TRACE!(
        "iface %p, stream_idx %u, buffer %p, stride %p.\n",
        iface,
        stream_idx,
        buffer,
        stride
    );

    if buffer.is_null() {
        return D3DERR_INVALIDCALL;
    }

    if (stream_idx as usize)
        >= (core::mem::size_of::<[wined3d_stream_state; 16]>() / core::mem::size_of::<wined3d_stream_state>())
    {
        WARN!("Stream index %u out of range.\n", stream_idx);
        return D3DERR_INVALIDCALL;
    }

    wined3d_mutex_lock();
    state = (*device).stateblock_state;
    stream = (core::ptr::addr_of!((*state).streams) as *const wined3d_stream_state)
        .offset(stream_idx as isize) as *const wined3d_stream_state;
    if !(*stream).buffer.is_null() {
        buffer_impl = wined3d_buffer_get_parent((*stream).buffer) as *mut d3d8_vertexbuffer;
        *buffer = core::ptr::addr_of_mut!((*buffer_impl).IDirect3DVertexBuffer8_iface);
        IDirect3DVertexBuffer8_AddRef(*buffer);
    } else {
        *buffer = core::ptr::null_mut();
    }
    *stride = (*stream).stride;
    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "C" fn device_from_device_parent(
    mut device_parent: *mut wined3d_device_parent,
) -> *mut d3d8_device {
    return (device_parent as usize)
        .wrapping_sub(core::mem::offset_of!(d3d8_device, device_parent))
        as *mut d3d8_device;
}

unsafe extern "C" fn device_parent_wined3d_device_created(
    device_parent: *mut wined3d_device_parent,
    device: *mut wined3d_device,
) {
    // TRACE("device_parent %p, device %p\n", device_parent, device);
}

unsafe extern "C" fn device_parent_mode_changed(device_parent: *mut wined3d_device_parent) {
    // TRACE("device_parent %p.\n", device_parent);
}

unsafe extern "C" fn device_parent_activate(device_parent: *mut wined3d_device_parent, activate: BOOL) {
    let device: *mut d3d8_device = device_from_device_parent(device_parent);
    if activate == 0 {
        let dest: *mut i32 = core::ptr::addr_of_mut!((*device).device_state) as *mut i32;
        let _ = core::sync::atomic::AtomicI32::from_ptr(dest).compare_exchange(
            D3D8_DEVICE_STATE_OK as i32,
            D3D8_DEVICE_STATE_LOST as i32,
            core::sync::atomic::Ordering::SeqCst,
            core::sync::atomic::Ordering::SeqCst,
        );
    } else {
        let dest: *mut i32 = core::ptr::addr_of_mut!((*device).device_state) as *mut i32;
        let _ = core::sync::atomic::AtomicI32::from_ptr(dest).compare_exchange(
            D3D8_DEVICE_STATE_LOST as i32,
            D3D8_DEVICE_STATE_NOT_RESET as i32,
            core::sync::atomic::Ordering::SeqCst,
            core::sync::atomic::Ordering::SeqCst,
        );
    }
}

unsafe extern "C" fn device_parent_texture_sub_resource_created(
    device_parent: *mut wined3d_device_parent,
    type_: wined3d_resource_type,
    wined3d_texture: *mut wined3d_texture,
    sub_resource_idx: u32,
    parent: *mut *mut core::ffi::c_void,
    parent_ops: *mut *const wined3d_parent_ops,
) -> HRESULT {
    if type_ as u32 == WINED3D_RTYPE_TEXTURE_3D {
        let mut d3d_volume: *mut d3d8_volume = core::ptr::null_mut::<d3d8_volume>();

        d3d_volume = calloc(
            (1 as size_t) as _,
            (core::mem::size_of::<d3d8_volume>() as size_t) as _,
        ) as *mut d3d8_volume;
        if d3d_volume.is_null() {
            return E_OUTOFMEMORY;
        }

        volume_init(d3d_volume, wined3d_texture, sub_resource_idx, parent_ops);
        *parent = d3d_volume as *mut core::ffi::c_void;
    } else if type_ as u32 != WINED3D_RTYPE_TEXTURE_2D {
        ERR!("Unhandled resource type %#x.\n", type_ as u32);
        return E_FAIL;
    }

    return D3D_OK;
}

unsafe extern "C" fn setup_fpu() {
    #[cfg(target_arch = "x86")]
    {
        let mut cw: WORD = 0;
        core::arch::asm!("fnstcw word ptr [{0}]", in(reg) core::ptr::addr_of_mut!(cw));
        cw = (cw & !0xf3f) | 0x3f;
        core::arch::asm!("fldcw word ptr [{0}]", in(reg) core::ptr::addr_of!(cw));
    }
    #[cfg(target_arch = "x86_64")]
    {
        let mut cw: WORD = 0;
        core::arch::asm!("fnstcw word ptr [{0}]", in(reg) core::ptr::addr_of_mut!(cw));
        cw = (cw & !0xf3f) | 0x3f;
        core::arch::asm!("fldcw word ptr [{0}]", in(reg) core::ptr::addr_of!(cw));
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        WINE_FIXME!("FPU setup not implemented for this platform.\n");
    }
}
#[no_mangle]
pub unsafe extern "C" fn device_init(
    device: *mut d3d8_device,
    parent: *mut d3d8,
    wined3d: *mut wined3d,
    adapter: UINT,
    device_type: D3DDEVTYPE,
    mut focus_window: HWND,
    flags: DWORD,
    parameters: *mut D3DPRESENT_PARAMETERS,
) -> HRESULT {
    let mut swapchain_desc: wined3d_swapchain_desc = core::mem::zeroed();
    let mut wined3d_swapchain: *mut wined3d_swapchain = core::ptr::null_mut();
    let mut wined3d_adapter: *mut wined3d_adapter = core::ptr::null_mut();
    let mut wined3d_output: *mut wined3d_output = core::ptr::null_mut();
    let mut d3d_swapchain: *mut d3d8_swapchain = core::ptr::null_mut();
    let mut caps: wined3d_caps = core::mem::zeroed();
    let mut output_idx: u32;
    let mut hr: HRESULT;

    static feature_levels: [wined3d_feature_level; 4] = [
        WINED3D_FEATURE_LEVEL_8,
        WINED3D_FEATURE_LEVEL_7,
        WINED3D_FEATURE_LEVEL_6,
        WINED3D_FEATURE_LEVEL_5,
    ];

    output_idx = adapter;
    if output_idx >= (*parent).wined3d_output_count {
        return D3DERR_INVALIDCALL;
    }

    (*device).IDirect3DDevice8_iface.lpVtbl =
        core::ptr::addr_of!(d3d8_device_vtbl) as *const IDirect3DDevice8Vtbl;
    (*device).device_parent.ops =
        core::ptr::addr_of!(d3d8_wined3d_device_parent_ops) as *const wined3d_device_parent_ops;
    (*device).adapter_ordinal = adapter;
    (*device).ref_ = 1 as LONG;
    (*device).handle_table.entries = calloc(
        D3D8_INITIAL_HANDLE_TABLE_SIZE,
        (core::mem::size_of::<d3d8_handle_entry>()) as _,
    ) as *mut d3d8_handle_entry;
    if (*device).handle_table.entries.is_null() {
        ERR!("Failed to allocate handle table memory.\n");
        return E_OUTOFMEMORY;
    }
    (*device).handle_table.table_size = D3D8_INITIAL_HANDLE_TABLE_SIZE;

    if (flags & (D3DCREATE_FPU_PRESERVE as DWORD)) == 0 {
        setup_fpu();
    }

    wined3d_mutex_lock();
    wined3d_output = *(*parent).wined3d_outputs.offset(output_idx as isize);
    wined3d_adapter = wined3d_output_get_adapter(wined3d_output);
    hr = wined3d_device_create(
        wined3d,
        wined3d_adapter,
        wined3d_device_type_from_d3d(device_type),
        focus_window,
        flags,
        4,
        core::ptr::addr_of!(feature_levels) as *const wined3d_feature_level,
        ARRAY_SIZE!(feature_levels) as u32,
        core::ptr::addr_of_mut!((*device).device_parent),
        core::ptr::addr_of_mut!((*device).wined3d_device),
    );
    if hr < 0 {
        WARN!("Failed to create wined3d device, hr %#lx.\n", hr);
        wined3d_mutex_unlock();
        free((*device).handle_table.entries as *mut core::ffi::c_void);
        return hr;
    }

    (*device).immediate_context = wined3d_device_get_immediate_context((*device).wined3d_device);
    wined3d_get_device_caps(
        wined3d_adapter,
        wined3d_device_type_from_d3d(device_type),
        core::ptr::addr_of_mut!(caps),
    );
    (*device).max_user_clip_planes = caps.MaxUserClipPlanes as u32;
    (*device).vs_uniform_count = caps.MaxVertexShaderConst as u32;

    hr = wined3d_stateblock_create(
        (*device).wined3d_device,
        core::ptr::null_mut(),
        WINED3D_SBT_PRIMARY,
        core::ptr::addr_of_mut!((*device).state),
    );
    if hr < 0 {
        ERR!("Failed to create primary stateblock, hr %#lx.\n", hr);
        wined3d_device_decref((*device).wined3d_device);
        wined3d_mutex_unlock();
        return hr;
    }
    (*device).stateblock_state = wined3d_stateblock_get_state((*device).state);
    (*device).update_state = (*device).state;

    wined3d_streaming_buffer_init(
        core::ptr::addr_of_mut!((*device).vertex_buffer),
        WINED3D_BIND_VERTEX_BUFFER,
    );
    wined3d_streaming_buffer_init(
        core::ptr::addr_of_mut!((*device).index_buffer),
        WINED3D_BIND_INDEX_BUFFER,
    );

    if (*parameters).Windowed == 0 {
        if focus_window.is_null() {
            focus_window = (*parameters).hDeviceWindow;
        }
        hr = wined3d_device_acquire_focus_window((*device).wined3d_device, focus_window);
        if hr < 0 {
            ERR!("Failed to acquire focus window, hr %#lx.\n", hr);
            wined3d_device_decref((*device).wined3d_device);
            wined3d_mutex_unlock();
            free((*device).handle_table.entries as *mut core::ffi::c_void);
            return hr;
        }
    }

    if (flags & (D3DCREATE_MULTITHREADED as DWORD)) != 0 {
        wined3d_device_set_multithreaded((*device).wined3d_device);
    }

    if wined3d_swapchain_desc_from_d3d8(
        core::ptr::addr_of_mut!(swapchain_desc),
        wined3d_output,
        parameters,
    ) == 0
    {
        wined3d_device_release_focus_window((*device).wined3d_device);
        wined3d_device_decref((*device).wined3d_device);
        wined3d_mutex_unlock();
        free((*device).handle_table.entries as *mut core::ffi::c_void);
        return D3DERR_INVALIDCALL;
    }
    swapchain_desc.flags |= WINED3D_SWAPCHAIN_IMPLICIT;

    hr = d3d8_swapchain_create(
        device,
        core::ptr::addr_of_mut!(swapchain_desc),
        wined3dswapinterval_from_d3d((*parameters).FullScreen_PresentationInterval),
        core::ptr::addr_of_mut!(d3d_swapchain),
    );
    if hr < 0 {
        WARN!("Failed to create implicit swapchain, hr %#lx.\n", hr);
        wined3d_device_release_focus_window((*device).wined3d_device);
        wined3d_device_decref((*device).wined3d_device);
        wined3d_mutex_unlock();
        free((*device).handle_table.entries as *mut core::ffi::c_void);
        return hr;
    }

    wined3d_swapchain = (*d3d_swapchain).wined3d_swapchain;
    wined3d_swapchain_incref(wined3d_swapchain);
    IDirect3DSwapChain8_Release(core::ptr::addr_of_mut!((*d3d_swapchain).IDirect3DSwapChain8_iface));

    wined3d_stateblock_set_render_state(
        (*device).state,
        WINED3D_RS_ZENABLE,
        (swapchain_desc.enable_auto_depth_stencil != 0) as u32,
    );
    wined3d_stateblock_set_render_state((*device).state, WINED3D_RS_POINTSIZE_MIN, 0);
    device_reset_viewport_state(device);
    wined3d_mutex_unlock();

    present_parameters_from_wined3d_swapchain_desc(
        parameters,
        core::ptr::addr_of_mut!(swapchain_desc),
        (*parameters).FullScreen_PresentationInterval,
    );

    'err: {
        (*device).declArraySize = 16;
        (*device).decls = malloc(
            ((*device).declArraySize as usize).wrapping_mul(core::mem::size_of::<FvfToDecl>()),
        ) as *mut FvfToDecl;
        if (*device).decls.is_null() {
            ERR!("Failed to allocate FVF vertex declaration map memory.\n");
            hr = E_OUTOFMEMORY;
            break 'err;
        }

        (*device).implicit_swapchain = wined3d_swapchain;

        (*device).d3d_parent = parent;
        IDirect3D8_AddRef(core::ptr::addr_of_mut!((*parent).IDirect3D8_iface));

        return D3D_OK;
    }

    wined3d_mutex_lock();
    wined3d_swapchain_decref(wined3d_swapchain);
    wined3d_device_release_focus_window((*device).wined3d_device);
    wined3d_device_decref((*device).wined3d_device);
    wined3d_mutex_unlock();
    free((*device).handle_table.entries as *mut core::ffi::c_void);
    return hr;
}

// COM vtable tables — generated from device.c by vtbl_gen.py
/// device.c:3486 `static const IDirect3DDevice8Vtbl d3d8_device_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d8_device_vtbl: IDirect3DDevice8Vtbl = IDirect3DDevice8Vtbl {
    QueryInterface: Some(d3d8_device_QueryInterface),
    AddRef: Some(d3d8_device_AddRef),
    Release: Some(d3d8_device_Release),
    TestCooperativeLevel: Some(d3d8_device_TestCooperativeLevel),
    GetAvailableTextureMem: Some(d3d8_device_GetAvailableTextureMem),
    ResourceManagerDiscardBytes: Some(d3d8_device_ResourceManagerDiscardBytes),
    GetDirect3D: Some(d3d8_device_GetDirect3D),
    GetDeviceCaps: Some(d3d8_device_GetDeviceCaps),
    GetDisplayMode: Some(d3d8_device_GetDisplayMode),
    GetCreationParameters: Some(d3d8_device_GetCreationParameters),
    SetCursorProperties: Some(d3d8_device_SetCursorProperties),
    SetCursorPosition: Some(d3d8_device_SetCursorPosition),
    ShowCursor: Some(d3d8_device_ShowCursor),
    CreateAdditionalSwapChain: Some(d3d8_device_CreateAdditionalSwapChain),
    Reset: Some(d3d8_device_Reset),
    Present: Some(d3d8_device_Present),
    GetBackBuffer: Some(d3d8_device_GetBackBuffer),
    GetRasterStatus: Some(d3d8_device_GetRasterStatus),
    SetGammaRamp: Some(d3d8_device_SetGammaRamp),
    GetGammaRamp: Some(d3d8_device_GetGammaRamp),
    CreateTexture: Some(d3d8_device_CreateTexture),
    CreateVolumeTexture: Some(d3d8_device_CreateVolumeTexture),
    CreateCubeTexture: Some(d3d8_device_CreateCubeTexture),
    CreateVertexBuffer: Some(d3d8_device_CreateVertexBuffer),
    CreateIndexBuffer: Some(d3d8_device_CreateIndexBuffer),
    CreateRenderTarget: Some(d3d8_device_CreateRenderTarget),
    CreateDepthStencilSurface: Some(d3d8_device_CreateDepthStencilSurface),
    CreateImageSurface: Some(d3d8_device_CreateImageSurface),
    CopyRects: Some(d3d8_device_CopyRects),
    UpdateTexture: Some(d3d8_device_UpdateTexture),
    GetFrontBuffer: Some(d3d8_device_GetFrontBuffer),
    SetRenderTarget: Some(d3d8_device_SetRenderTarget),
    GetRenderTarget: Some(d3d8_device_GetRenderTarget),
    GetDepthStencilSurface: Some(d3d8_device_GetDepthStencilSurface),
    BeginScene: Some(d3d8_device_BeginScene),
    EndScene: Some(d3d8_device_EndScene),
    Clear: Some(d3d8_device_Clear),
    SetTransform: Some(d3d8_device_SetTransform),
    GetTransform: Some(d3d8_device_GetTransform),
    MultiplyTransform: Some(d3d8_device_MultiplyTransform),
    SetViewport: Some(d3d8_device_SetViewport),
    GetViewport: Some(d3d8_device_GetViewport),
    SetMaterial: Some(d3d8_device_SetMaterial),
    GetMaterial: Some(d3d8_device_GetMaterial),
    SetLight: Some(d3d8_device_SetLight),
    GetLight: Some(d3d8_device_GetLight),
    LightEnable: Some(d3d8_device_LightEnable),
    GetLightEnable: Some(d3d8_device_GetLightEnable),
    SetClipPlane: Some(d3d8_device_SetClipPlane),
    GetClipPlane: Some(d3d8_device_GetClipPlane),
    SetRenderState: Some(d3d8_device_SetRenderState),
    GetRenderState: Some(d3d8_device_GetRenderState),
    BeginStateBlock: Some(d3d8_device_BeginStateBlock),
    EndStateBlock: Some(d3d8_device_EndStateBlock),
    ApplyStateBlock: Some(d3d8_device_ApplyStateBlock),
    CaptureStateBlock: Some(d3d8_device_CaptureStateBlock),
    DeleteStateBlock: Some(d3d8_device_DeleteStateBlock),
    CreateStateBlock: Some(d3d8_device_CreateStateBlock),
    SetClipStatus: Some(d3d8_device_SetClipStatus),
    GetClipStatus: Some(d3d8_device_GetClipStatus),
    GetTexture: Some(d3d8_device_GetTexture),
    SetTexture: Some(d3d8_device_SetTexture),
    GetTextureStageState: Some(d3d8_device_GetTextureStageState),
    SetTextureStageState: Some(d3d8_device_SetTextureStageState),
    ValidateDevice: Some(d3d8_device_ValidateDevice),
    GetInfo: Some(d3d8_device_GetInfo),
    SetPaletteEntries: Some(d3d8_device_SetPaletteEntries),
    GetPaletteEntries: Some(d3d8_device_GetPaletteEntries),
    SetCurrentTexturePalette: Some(d3d8_device_SetCurrentTexturePalette),
    GetCurrentTexturePalette: Some(d3d8_device_GetCurrentTexturePalette),
    DrawPrimitive: Some(d3d8_device_DrawPrimitive),
    DrawIndexedPrimitive: Some(d3d8_device_DrawIndexedPrimitive),
    DrawPrimitiveUP: Some(d3d8_device_DrawPrimitiveUP),
    DrawIndexedPrimitiveUP: Some(d3d8_device_DrawIndexedPrimitiveUP),
    ProcessVertices: Some(d3d8_device_ProcessVertices),
    CreateVertexShader: Some(d3d8_device_CreateVertexShader),
    SetVertexShader: Some(d3d8_device_SetVertexShader),
    GetVertexShader: Some(d3d8_device_GetVertexShader),
    DeleteVertexShader: Some(d3d8_device_DeleteVertexShader),
    SetVertexShaderConstant: Some(d3d8_device_SetVertexShaderConstant),
    GetVertexShaderConstant: Some(d3d8_device_GetVertexShaderConstant),
    GetVertexShaderDeclaration: Some(d3d8_device_GetVertexShaderDeclaration),
    GetVertexShaderFunction: Some(d3d8_device_GetVertexShaderFunction),
    SetStreamSource: Some(d3d8_device_SetStreamSource),
    GetStreamSource: Some(d3d8_device_GetStreamSource),
    SetIndices: Some(d3d8_device_SetIndices),
    GetIndices: Some(d3d8_device_GetIndices),
    CreatePixelShader: Some(d3d8_device_CreatePixelShader),
    SetPixelShader: Some(d3d8_device_SetPixelShader),
    GetPixelShader: Some(d3d8_device_GetPixelShader),
    DeletePixelShader: Some(d3d8_device_DeletePixelShader),
    SetPixelShaderConstant: Some(d3d8_device_SetPixelShaderConstant),
    GetPixelShaderConstant: Some(d3d8_device_GetPixelShaderConstant),
    GetPixelShaderFunction: Some(d3d8_device_GetPixelShaderFunction),
    DrawRectPatch: Some(d3d8_device_DrawRectPatch),
    DrawTriPatch: Some(d3d8_device_DrawTriPatch),
    DeletePatch: Some(d3d8_device_DeletePatch),
};
