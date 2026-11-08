use crate::d3dwine::utils::add_ffp_frag_shader;
use crate::d3dwine::ffp_gl::clipplane;
use crate::d3dwine::utils::compute_normal_matrix;
use crate::d3dwine::utils::debug_color;
use crate::d3dwine::utils::debug_d3dformat;
use crate::d3dwine::utils::debug_d3drenderstate;
use crate::d3dwine::utils::debug_d3dtexturestate;
use crate::d3dwine::utils::debug_d3dtstype;
use crate::d3dwine::utils::debug_ivec4;
use crate::d3dwine::utils::debug_vec4;
use crate::d3dwine::utils::find_ffp_frag_shader;
use crate::d3dwine::utils::get_identity_matrix;
use crate::d3dwine::utils::get_modelview_matrix;
use crate::d3dwine::utils::get_texture_matrix;
use crate::d3dwine::utils::invert_matrix;
use crate::d3dwine::utils::multiply_matrix;
use crate::d3dwine::device::wined3d_blend_state_create;
use crate::d3dwine::device::wined3d_blend_state_decref;
use crate::d3dwine::buffer::wined3d_buffer_decref;
use crate::d3dwine::buffer::wined3d_buffer_incref;
use crate::d3dwine::utils::wined3d_debug_feature_level;
use crate::d3dwine::device::wined3d_depth_stencil_state_create;
use crate::d3dwine::device::wined3d_depth_stencil_state_decref;
use crate::d3dwine::cs::wined3d_device_context_emit_set_clip_plane;
use crate::d3dwine::cs::wined3d_device_context_emit_set_light;
use crate::d3dwine::cs::wined3d_device_context_emit_set_light_enable;
use crate::d3dwine::cs::wined3d_device_context_emit_set_render_state;
use crate::d3dwine::cs::wined3d_device_context_emit_set_shader;
use crate::d3dwine::cs::wined3d_device_context_emit_set_texture;
use crate::d3dwine::cs::wined3d_device_context_emit_set_texture_state;
use crate::d3dwine::cs::wined3d_device_context_emit_set_transform;
use crate::d3dwine::device::wined3d_device_context_get_blend_state;
use crate::d3dwine::device::wined3d_device_context_get_depth_stencil_state;
use crate::d3dwine::cs::wined3d_device_context_push_constants;
use crate::d3dwine::device::wined3d_device_context_resolve_sub_resource;
use crate::d3dwine::device::wined3d_device_context_set_blend_state;
use crate::d3dwine::device::wined3d_device_context_set_constant_buffers;
use crate::d3dwine::cs::wined3d_device_context_set_depth_bounds;
use crate::d3dwine::device::wined3d_device_context_set_depth_stencil_state;
use crate::d3dwine::device::wined3d_device_context_set_index_buffer;
use crate::d3dwine::device::wined3d_device_context_set_rasterizer_state;
use crate::d3dwine::device::wined3d_device_context_set_samplers;
use crate::d3dwine::device::wined3d_device_context_set_scissor_rects;
use crate::d3dwine::device::wined3d_device_context_set_shader;
use crate::d3dwine::device::wined3d_device_context_set_stream_sources;
use crate::d3dwine::device::wined3d_device_context_set_vertex_declaration;
use crate::d3dwine::device::wined3d_device_context_set_viewports;
use crate::d3dwine::utils::wined3d_ffp_get_fs_settings;
use crate::d3dwine::utils::wined3d_ffp_get_vs_settings;
use crate::d3dwine::utils::wined3d_format_get_float_color_key;
use crate::d3dwine::wined3d_main::wined3d_mutex_lock;
use crate::d3dwine::wined3d_main::wined3d_mutex_unlock;
use crate::d3dwine::device::wined3d_rasterizer_state_create;
use crate::d3dwine::device::wined3d_rasterizer_state_decref;
use crate::d3dwine::view::wined3d_rendertarget_view_decref;
use crate::d3dwine::sampler::wined3d_sampler_create;
use crate::d3dwine::sampler::wined3d_sampler_decref;
use crate::d3dwine::view::wined3d_shader_resource_view_decref;
use crate::d3dwine::view::wined3d_shader_resource_view_incref;
use crate::d3dwine::texture::wined3d_texture_acquire_identity_srv;
use crate::d3dwine::texture::wined3d_texture_decref;
use crate::d3dwine::texture::wined3d_texture_incref;
use crate::d3dwine::texture::wined3d_texture_set_lod;
use crate::d3dwine::view::wined3d_unordered_access_view_decref;
use crate::d3dwine::vertexdeclaration::wined3d_vertex_declaration_decref;
use crate::d3dwine::vertexdeclaration::wined3d_vertex_declaration_incref;
use crate::prelude::*;
use core::ffi::c_void;


// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut pixel_states_render: [DWORD; 60] = [
    WINED3D_RS_ALPHABLENDENABLE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_ALPHAFUNC as ::core::ffi::c_int as DWORD,
    WINED3D_RS_ALPHAREF as ::core::ffi::c_int as DWORD,
    WINED3D_RS_ALPHATESTENABLE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_ANTIALIASEDLINEENABLE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_BLENDFACTOR as ::core::ffi::c_int as DWORD,
    WINED3D_RS_BLENDOP as ::core::ffi::c_int as DWORD,
    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int as DWORD,
    WINED3D_RS_BACK_STENCILFAIL as ::core::ffi::c_int as DWORD,
    WINED3D_RS_BACK_STENCILPASS as ::core::ffi::c_int as DWORD,
    WINED3D_RS_BACK_STENCILZFAIL as ::core::ffi::c_int as DWORD,
    WINED3D_RS_COLORWRITEENABLE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_COLORWRITEENABLE1 as ::core::ffi::c_int as DWORD,
    WINED3D_RS_COLORWRITEENABLE2 as ::core::ffi::c_int as DWORD,
    WINED3D_RS_COLORWRITEENABLE3 as ::core::ffi::c_int as DWORD,
    WINED3D_RS_DEPTHBIAS as ::core::ffi::c_int as DWORD,
    WINED3D_RS_DESTBLEND as ::core::ffi::c_int as DWORD,
    WINED3D_RS_DESTBLENDALPHA as ::core::ffi::c_int as DWORD,
    WINED3D_RS_DITHERENABLE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_FILLMODE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_FOGDENSITY as ::core::ffi::c_int as DWORD,
    WINED3D_RS_FOGEND as ::core::ffi::c_int as DWORD,
    WINED3D_RS_FOGSTART as ::core::ffi::c_int as DWORD,
    WINED3D_RS_LASTPIXEL as ::core::ffi::c_int as DWORD,
    WINED3D_RS_SCISSORTESTENABLE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_SEPARATEALPHABLENDENABLE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_SHADEMODE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_SLOPESCALEDEPTHBIAS as ::core::ffi::c_int as DWORD,
    WINED3D_RS_SRCBLEND as ::core::ffi::c_int as DWORD,
    WINED3D_RS_SRCBLENDALPHA as ::core::ffi::c_int as DWORD,
    WINED3D_RS_SRGBWRITEENABLE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_STENCILENABLE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_STENCILFAIL as ::core::ffi::c_int as DWORD,
    WINED3D_RS_STENCILFUNC as ::core::ffi::c_int as DWORD,
    WINED3D_RS_STENCILMASK as ::core::ffi::c_int as DWORD,
    WINED3D_RS_STENCILPASS as ::core::ffi::c_int as DWORD,
    WINED3D_RS_STENCILREF as ::core::ffi::c_int as DWORD,
    WINED3D_RS_STENCILWRITEMASK as ::core::ffi::c_int as DWORD,
    WINED3D_RS_STENCILZFAIL as ::core::ffi::c_int as DWORD,
    WINED3D_RS_TEXTUREFACTOR as ::core::ffi::c_int as DWORD,
    WINED3D_RS_TWOSIDEDSTENCILMODE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_WRAP0 as ::core::ffi::c_int as DWORD,
    WINED3D_RS_WRAP1 as ::core::ffi::c_int as DWORD,
    WINED3D_RS_WRAP10 as ::core::ffi::c_int as DWORD,
    WINED3D_RS_WRAP11 as ::core::ffi::c_int as DWORD,
    WINED3D_RS_WRAP12 as ::core::ffi::c_int as DWORD,
    WINED3D_RS_WRAP13 as ::core::ffi::c_int as DWORD,
    WINED3D_RS_WRAP14 as ::core::ffi::c_int as DWORD,
    WINED3D_RS_WRAP15 as ::core::ffi::c_int as DWORD,
    WINED3D_RS_WRAP2 as ::core::ffi::c_int as DWORD,
    WINED3D_RS_WRAP3 as ::core::ffi::c_int as DWORD,
    WINED3D_RS_WRAP4 as ::core::ffi::c_int as DWORD,
    WINED3D_RS_WRAP5 as ::core::ffi::c_int as DWORD,
    WINED3D_RS_WRAP6 as ::core::ffi::c_int as DWORD,
    WINED3D_RS_WRAP7 as ::core::ffi::c_int as DWORD,
    WINED3D_RS_WRAP8 as ::core::ffi::c_int as DWORD,
    WINED3D_RS_WRAP9 as ::core::ffi::c_int as DWORD,
    WINED3D_RS_ZENABLE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_ZFUNC as ::core::ffi::c_int as DWORD,
    WINED3D_RS_ZWRITEENABLE as ::core::ffi::c_int as DWORD,
];

static mut pixel_states_texture: [DWORD; 17] = [
    WINED3D_TSS_ALPHA_ARG0 as ::core::ffi::c_int as DWORD,
    WINED3D_TSS_ALPHA_ARG1 as ::core::ffi::c_int as DWORD,
    WINED3D_TSS_ALPHA_ARG2 as ::core::ffi::c_int as DWORD,
    WINED3D_TSS_ALPHA_OP as ::core::ffi::c_int as DWORD,
    WINED3D_TSS_BUMPENV_LOFFSET as ::core::ffi::c_int as DWORD,
    WINED3D_TSS_BUMPENV_LSCALE as ::core::ffi::c_int as DWORD,
    WINED3D_TSS_BUMPENV_MAT00 as ::core::ffi::c_int as DWORD,
    WINED3D_TSS_BUMPENV_MAT01 as ::core::ffi::c_int as DWORD,
    WINED3D_TSS_BUMPENV_MAT10 as ::core::ffi::c_int as DWORD,
    WINED3D_TSS_BUMPENV_MAT11 as ::core::ffi::c_int as DWORD,
    WINED3D_TSS_COLOR_ARG0 as ::core::ffi::c_int as DWORD,
    WINED3D_TSS_COLOR_ARG1 as ::core::ffi::c_int as DWORD,
    WINED3D_TSS_COLOR_ARG2 as ::core::ffi::c_int as DWORD,
    WINED3D_TSS_COLOR_OP as ::core::ffi::c_int as DWORD,
    WINED3D_TSS_RESULT_ARG as ::core::ffi::c_int as DWORD,
    WINED3D_TSS_TEXCOORD_INDEX as ::core::ffi::c_int as DWORD,
    WINED3D_TSS_TEXTURE_TRANSFORM_FLAGS as ::core::ffi::c_int as DWORD,
];

static mut pixel_states_sampler: [DWORD; 12] = [
    WINED3D_SAMP_ADDRESS_U as ::core::ffi::c_int as DWORD,
    WINED3D_SAMP_ADDRESS_V as ::core::ffi::c_int as DWORD,
    WINED3D_SAMP_ADDRESS_W as ::core::ffi::c_int as DWORD,
    WINED3D_SAMP_BORDER_COLOR as ::core::ffi::c_int as DWORD,
    WINED3D_SAMP_MAG_FILTER as ::core::ffi::c_int as DWORD,
    WINED3D_SAMP_MIN_FILTER as ::core::ffi::c_int as DWORD,
    WINED3D_SAMP_MIP_FILTER as ::core::ffi::c_int as DWORD,
    WINED3D_SAMP_MIPMAP_LOD_BIAS as ::core::ffi::c_int as DWORD,
    WINED3D_SAMP_MAX_MIP_LEVEL as ::core::ffi::c_int as DWORD,
    WINED3D_SAMP_MAX_ANISOTROPY as ::core::ffi::c_int as DWORD,
    WINED3D_SAMP_SRGB_TEXTURE as ::core::ffi::c_int as DWORD,
    WINED3D_SAMP_ELEMENT_INDEX as ::core::ffi::c_int as DWORD,
];

static mut vertex_states_render: [DWORD; 45] = [
    WINED3D_RS_ADAPTIVETESS_W as ::core::ffi::c_int as DWORD,
    WINED3D_RS_ADAPTIVETESS_X as ::core::ffi::c_int as DWORD,
    WINED3D_RS_ADAPTIVETESS_Y as ::core::ffi::c_int as DWORD,
    WINED3D_RS_ADAPTIVETESS_Z as ::core::ffi::c_int as DWORD,
    WINED3D_RS_AMBIENT as ::core::ffi::c_int as DWORD,
    WINED3D_RS_AMBIENTMATERIALSOURCE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_CLIPPING as ::core::ffi::c_int as DWORD,
    WINED3D_RS_CLIPPLANEENABLE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_COLORVERTEX as ::core::ffi::c_int as DWORD,
    WINED3D_RS_CULLMODE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_DIFFUSEMATERIALSOURCE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_EMISSIVEMATERIALSOURCE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_ENABLEADAPTIVETESSELLATION as ::core::ffi::c_int as DWORD,
    WINED3D_RS_FOGCOLOR as ::core::ffi::c_int as DWORD,
    WINED3D_RS_FOGDENSITY as ::core::ffi::c_int as DWORD,
    WINED3D_RS_FOGENABLE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_FOGEND as ::core::ffi::c_int as DWORD,
    WINED3D_RS_FOGSTART as ::core::ffi::c_int as DWORD,
    WINED3D_RS_FOGTABLEMODE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_FOGVERTEXMODE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_INDEXEDVERTEXBLENDENABLE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_LIGHTING as ::core::ffi::c_int as DWORD,
    WINED3D_RS_LOCALVIEWER as ::core::ffi::c_int as DWORD,
    WINED3D_RS_MAXTESSELLATIONLEVEL as ::core::ffi::c_int as DWORD,
    WINED3D_RS_MINTESSELLATIONLEVEL as ::core::ffi::c_int as DWORD,
    WINED3D_RS_MULTISAMPLEANTIALIAS as ::core::ffi::c_int as DWORD,
    WINED3D_RS_MULTISAMPLEMASK as ::core::ffi::c_int as DWORD,
    WINED3D_RS_NORMALDEGREE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_NORMALIZENORMALS as ::core::ffi::c_int as DWORD,
    WINED3D_RS_PATCHEDGESTYLE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_POINTSCALE_A as ::core::ffi::c_int as DWORD,
    WINED3D_RS_POINTSCALE_B as ::core::ffi::c_int as DWORD,
    WINED3D_RS_POINTSCALE_C as ::core::ffi::c_int as DWORD,
    WINED3D_RS_POINTSCALEENABLE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_POINTSIZE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_POINTSIZE_MAX as ::core::ffi::c_int as DWORD,
    WINED3D_RS_POINTSIZE_MIN as ::core::ffi::c_int as DWORD,
    WINED3D_RS_POINTSPRITEENABLE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_POSITIONDEGREE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_RANGEFOGENABLE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_SHADEMODE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_SPECULARENABLE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_SPECULARMATERIALSOURCE as ::core::ffi::c_int as DWORD,
    WINED3D_RS_TWEENFACTOR as ::core::ffi::c_int as DWORD,
    WINED3D_RS_VERTEXBLEND as ::core::ffi::c_int as DWORD,
];

static mut vertex_states_texture: [DWORD; 2] = [
    WINED3D_TSS_TEXCOORD_INDEX as ::core::ffi::c_int as DWORD,
    WINED3D_TSS_TEXTURE_TRANSFORM_FLAGS as ::core::ffi::c_int as DWORD,
];

static mut vertex_states_sampler: [DWORD; 1] =
    [WINED3D_SAMP_DMAP_OFFSET as ::core::ffi::c_int as DWORD];

// WINE_DEFAULT_DEBUG_CHANNEL(d3d)
// WINE_DECLARE_DEBUG_CHANNEL(winediag): extern channel, referenced only via the logging macros.

// Assumed values for header macros not present in this preamble:
//   WINED3D_MAX_VS_CONSTS_F = 256        -> WINED3D_BITMAP_SIZE = 8
//   WINED3D_MAX_PS_CONSTS_F = 224        -> WINED3D_BITMAP_SIZE = 7
//   WINED3D_HIGHEST_TRANSFORM_STATE = 512 -> (512 + 1) bits = 17 words
//   WINEHIGHEST_RENDER_STATE = 210       -> (210 + 1) bits = 7 words
//   WINED3D_MAX_FFP_TEXTURES = 8, WINED3D_HIGHEST_TEXTURE_STATE = 17 (given: +1 = 18)
//   WINED3D_MAX_COMBINED_SAMPLERS = 20, WINED3D_HIGHEST_SAMPLER_STATE = 13 (given: +1 = 14)
//   WINED3D_MAX_CONSTS_I = WINED3D_MAX_CONSTS_B = WINED3D_MAX_STREAMS = 16 (given)
// Rust has no bitfields: the C bitfield runs are packed into u32 words in declaration order.
//   word 0 (bits 0..31): textures:20, then indices, material, viewport, vertexDecl,
//                        pixelShader, vertexShader, scissorRect, store_stream_offset,
//                        alpha_to_coverage, lights, transforms (1 bit each)
//   word 1 (bits 0..8):  ffp_ps_constants, texture_matrices, modelview_matrices, point_scale,
//                        ffp_vs_settings, ffp_ps_settings, rasterizer_state,
//                        position_transformed, bumpenv_constants (1 bit each)



pub type PWINED3D_SAVED_STATES = *mut wined3d_saved_states;
pub type PSTAGE_STATE = *mut stage_state;
pub type PWINED3D_STATEBLOCK = *mut wined3d_stateblock;

unsafe extern "C" fn transpose_matrix(out: *mut wined3d_matrix, m: *const wined3d_matrix) {
    let mut temp: wined3d_matrix = core::mem::zeroed();
    let mut i: ::core::ffi::c_uint = 0;
    while i < 4 {
        let mut j: ::core::ffi::c_uint = 0;
        while j < 4 {
            *core::ptr::addr_of_mut!(temp._11)
                .offset((4 as ::core::ffi::c_uint).wrapping_mul(j).wrapping_add(i) as isize) =
                *core::ptr::addr_of!((*m)._11)
                    .offset((4 as ::core::ffi::c_uint).wrapping_mul(i).wrapping_add(j) as isize);
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    *out = temp;
}

unsafe extern "C" fn stateblock_set_all_bits(map: *mut uint32_t, map_size: UINT) {
    let mut mask: DWORD = (1 as DWORD).wrapping_shl(map_size & 0x1f) .wrapping_sub(1 as DWORD);
    memset(map as *mut core::ffi::c_void, 0xff as core::ffi::c_int, ((map_size >> 5) as usize).wrapping_mul(core::mem::size_of::<uint32_t>()));
    if mask != 0 {
        *map.offset((map_size >> 5) as isize) = mask;
    }
}

unsafe extern "C" fn stateblock_savedstates_set_all(
    mut states: *mut wined3d_saved_states,
    mut vs_consts: DWORD,
    mut ps_consts: DWORD,
) {
    let mut i: ::core::ffi::c_uint = 0;
    (*states).set_indices(1 as uint32_t);
    (*states).set_material(1 as uint32_t);
    (*states).set_viewport(1 as uint32_t);
    (*states).set_vertexDecl(1 as uint32_t);
    (*states).set_pixelShader(1 as uint32_t);
    (*states).set_vertexShader(1 as uint32_t);
    (*states).set_scissorRect(1 as uint32_t);
    (*states).set_alpha_to_coverage(1 as uint32_t);
    (*states).set_lights(1 as uint32_t);
    (*states).set_transforms(1 as uint32_t);

    (*states).streamSource = 0xffff as uint16_t;
    (*states).streamFreq = 0xffff as uint16_t;
    (*states).set_textures(0xfffff as uint32_t);
    stateblock_set_all_bits(
        &raw mut (*states).transform as *mut uint32_t,
        ((WINED3D_HIGHEST_TRANSFORM_STATE) + (1)) as UINT,
    );
    stateblock_set_all_bits(
        &raw mut (*states).renderState as *mut uint32_t,
        ((WINEHIGHEST_RENDER_STATE) + (1)) as UINT,
    );
    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_MAX_FFP_TEXTURES as ::core::ffi::c_uint {
        (*states).textureState[i as usize] = 0x3ffff as uint32_t;
        i = i.wrapping_add(1);
    }
    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_MAX_COMBINED_SAMPLERS as ::core::ffi::c_uint {
        (*states).samplerState[i as usize] = 0x3ffe as uint16_t;
        i = i.wrapping_add(1);
    }
    (*states).clipplane = wined3d_mask_from_size(WINED3D_MAX_CLIP_DISTANCES as ::core::ffi::c_uint);
    (*states).pixelShaderConstantsB = 0xffff as uint16_t;
    (*states).pixelShaderConstantsI = 0xffff as uint16_t;
    (*states).vertexShaderConstantsB = 0xffff as uint16_t;
    (*states).vertexShaderConstantsI = 0xffff as uint16_t;

    memset(
        &raw mut (*states).ps_consts_f as *mut uint32_t as *mut ::core::ffi::c_void,
        0xff as ::core::ffi::c_int,
        ::core::mem::size_of::<[uint32_t; 7]>() as size_t,
    );
    memset(
        &raw mut (*states).vs_consts_f as *mut uint32_t as *mut ::core::ffi::c_void,
        0xff as ::core::ffi::c_int,
        ::core::mem::size_of::<[uint32_t; 8]>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_primary_dirtify_all_states(
    device: *mut wined3d_device,
    stateblock: *mut wined3d_stateblock,
) {
    let lights_tree: *mut rb_tree =
        core::ptr::addr_of_mut!((*(*stateblock).stateblock_state.light_state).lights_tree);
    let d3d_info: *const wined3d_d3d_info = core::ptr::addr_of!((*(*device).adapter).d3d_info);
    let states: *mut wined3d_saved_states = core::ptr::addr_of_mut!((*stateblock).changed);
    let mut light: *mut wined3d_light_info = core::ptr::null_mut::<wined3d_light_info>();

    stateblock_savedstates_set_all(
        states,
        (*d3d_info).limits.vs_uniform_count,
        (*d3d_info).limits.ps_uniform_count,
    );
    (*states).set_ffp_ps_constants(1 as uint32_t);
    (*states).set_texture_matrices(1 as uint32_t);
    (*states).set_modelview_matrices(1 as uint32_t);
    (*states).set_point_scale(1 as uint32_t);
    (*states).set_ffp_vs_settings(1 as uint32_t);
    (*states).set_ffp_ps_settings(1 as uint32_t);
    (*states).set_rasterizer_state(1 as uint32_t);
    (*states).set_position_transformed(1 as uint32_t);

    list_init(core::ptr::addr_of_mut!((*stateblock).changed.changed_lights));
    light = (rb_head((*lights_tree).root) as *mut core::ffi::c_char)
        .offset(-(232 as core::ffi::c_ulonglong as isize)) as *mut wined3d_light_info;
    while light
        != (core::ptr::null_mut::<core::ffi::c_char>()
            .offset(-(232 as core::ffi::c_ulonglong as isize))) as *mut wined3d_light_info
    {
        (*light).changed = true;
        list_add_tail(
            core::ptr::addr_of_mut!((*stateblock).changed.changed_lights),
            core::ptr::addr_of_mut!((*light).changed_entry),
        );
        light = (rb_next(core::ptr::addr_of_mut!((*light).entry)) as *mut core::ffi::c_char)
            .offset(-(232 as core::ffi::c_ulonglong as isize)) as *mut wined3d_light_info;
    }
}

unsafe extern "C" fn stateblock_savedstates_set_pixel(states: *mut wined3d_saved_states, num_constants: DWORD) {
    let mut texture_mask: DWORD = 0;
    let mut sampler_mask: WORD = 0;
    let mut i: ::core::ffi::c_uint;

    (*states).set_pixelShader(1 as uint32_t);

    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(pixel_states_render) as usize {
        let rs: DWORD = pixel_states_render[i as usize];
        (*states).renderState[(rs >> 5) as usize] =
            ((*states).renderState[(rs >> 5) as usize] as ::core::ffi::c_uint
                | (1 as ::core::ffi::c_uint) << (rs & 0x1f))
                as uint32_t;
        i = i.wrapping_add(1);
    }

    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(pixel_states_texture) as usize {
        texture_mask |= (1 as ::core::ffi::c_uint) << pixel_states_texture[i as usize];
        i = i.wrapping_add(1);
    }

    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_MAX_FFP_TEXTURES as ::core::ffi::c_uint {
        (*states).textureState[i as usize] = texture_mask as uint32_t;
        i = i.wrapping_add(1);
    }

    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(pixel_states_sampler) as usize {
        sampler_mask = (sampler_mask as ::core::ffi::c_uint
            | (1 as ::core::ffi::c_uint) << pixel_states_sampler[i as usize])
            as WORD;
        i = i.wrapping_add(1);
    }

    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_MAX_COMBINED_SAMPLERS as ::core::ffi::c_uint {
        (*states).samplerState[i as usize] = sampler_mask as uint16_t;
        i = i.wrapping_add(1);
    }

    (*states).pixelShaderConstantsB = 0xffff as uint16_t;
    (*states).pixelShaderConstantsI = 0xffff as uint16_t;

    memset(
        core::ptr::addr_of_mut!((*states).ps_consts_f) as *mut ::core::ffi::c_void,
        0xff as ::core::ffi::c_int,
        ::core::mem::size_of::<[uint32_t; 7]>() as size_t,
    );
}

unsafe extern "C" fn stateblock_savedstates_set_vertex(states: *mut wined3d_saved_states, num_constants: DWORD) {
    let mut texture_mask: DWORD = 0 as DWORD;
    let mut sampler_mask: WORD = 0 as WORD;
    let mut i: ::core::ffi::c_uint;

    (*states).set_vertexDecl(1 as uint32_t);
    (*states).set_vertexShader(1 as uint32_t);
    (*states).set_alpha_to_coverage(1 as uint32_t);
    (*states).set_lights(1 as uint32_t);

    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(vertex_states_render) as usize {
        let rs: DWORD = vertex_states_render[i as usize];
        (*states).renderState[(rs >> 5 as ::core::ffi::c_int) as usize] =
            ((*states).renderState[(rs >> 5 as ::core::ffi::c_int) as usize]
                as ::core::ffi::c_uint
                | (1 as ::core::ffi::c_uint)
                    << (rs & 0x1f as DWORD)) as uint32_t;
        i = i.wrapping_add(1);
    }

    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(vertex_states_texture) as usize {
        texture_mask |= (1 as DWORD) << vertex_states_texture[i as usize];
        i = i.wrapping_add(1);
    }

    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_MAX_FFP_TEXTURES as ::core::ffi::c_uint {
        (*states).textureState[i as usize] = texture_mask as uint32_t;
        i = i.wrapping_add(1);
    }

    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(vertex_states_sampler) as usize {
        sampler_mask = (sampler_mask as ::core::ffi::c_uint
            | (1 as ::core::ffi::c_uint) << vertex_states_sampler[i as usize])
            as WORD;
        i = i.wrapping_add(1);
    }

    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_MAX_COMBINED_SAMPLERS as ::core::ffi::c_uint {
        (*states).samplerState[i as usize] = sampler_mask as uint16_t;
        i = i.wrapping_add(1);
    }

    (*states).vertexShaderConstantsB = 0xffff as uint16_t;
    (*states).vertexShaderConstantsI = 0xffff as uint16_t;

    memset(
        core::ptr::addr_of_mut!((*states).vs_consts_f) as *mut ::core::ffi::c_void,
        0xff as ::core::ffi::c_int,
        ::core::mem::size_of::<[uint32_t; 8]>() as size_t,
    );
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_init_contained_states(stateblock: *mut wined3d_stateblock) {
    let mut i: ::core::ffi::c_uint;
    let mut j: ::core::ffi::c_uint;

    i = 0 as ::core::ffi::c_uint;
    while i <= ((WINEHIGHEST_RENDER_STATE as ::core::ffi::c_uint) >> (5 as ::core::ffi::c_uint)) {
        let mut map: DWORD = (*stateblock).changed.renderState[i as usize];
        j = 0 as ::core::ffi::c_uint;
        while map != 0 {
            if (map & (1 as DWORD)) != 0 {
                (*stateblock).contained_render_states[(*stateblock).num_contained_render_states as usize] =
                    (i << (5 as ::core::ffi::c_uint)) | j;
                (*stateblock).num_contained_render_states =
                    (*stateblock).num_contained_render_states.wrapping_add(1);
            }
            map >>= 1 as DWORD;
            j = j.wrapping_add(0 as ::core::ffi::c_uint + 1);
        }
        i = i.wrapping_add(1);
    }

    i = 0 as ::core::ffi::c_uint;
    while i <= ((WINED3D_HIGHEST_TRANSFORM_STATE as ::core::ffi::c_uint) >> (5 as ::core::ffi::c_uint)) {
        let mut map_0: DWORD = (*stateblock).changed.transform[i as usize];
        j = 0 as ::core::ffi::c_uint;
        while map_0 != 0 {
            if (map_0 & (1 as DWORD)) != 0 {
                (*stateblock).contained_transform_states[(*stateblock).num_contained_transform_states as usize] =
                    (i << (5 as ::core::ffi::c_uint)) | j;
                (*stateblock).num_contained_transform_states =
                    (*stateblock).num_contained_transform_states.wrapping_add(1);
            }
            map_0 >>= 1 as DWORD;
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }

    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_MAX_FFP_TEXTURES as ::core::ffi::c_uint {
        let mut map_1: DWORD = (*stateblock).changed.textureState[i as usize];
        j = 0 as ::core::ffi::c_uint;
        while map_1 != 0 {
            if (map_1 & (1 as DWORD)) != 0 {
                (*stateblock).contained_tss_states[(*stateblock).num_contained_tss_states as usize].stage = i;
                (*stateblock).contained_tss_states[(*stateblock).num_contained_tss_states as usize].state = j;
                (*stateblock).num_contained_tss_states =
                    (*stateblock).num_contained_tss_states.wrapping_add(1);
            }
            map_1 >>= 1 as DWORD;
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }

    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_MAX_COMBINED_SAMPLERS as ::core::ffi::c_uint {
        let mut map_2: DWORD = (*stateblock).changed.samplerState[i as usize] as DWORD;
        j = 0 as ::core::ffi::c_uint;
        while map_2 != 0 {
            if (map_2 & (1 as DWORD)) != 0 {
                (*stateblock).contained_sampler_states[(*stateblock).num_contained_sampler_states as usize].stage = i;
                (*stateblock).contained_sampler_states[(*stateblock).num_contained_sampler_states as usize].state = j;
                (*stateblock).num_contained_sampler_states =
                    (*stateblock).num_contained_sampler_states.wrapping_add(1);
            }
            map_2 >>= 1 as DWORD;
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn stateblock_init_lights(stateblock: *mut wined3d_stateblock, src_tree: *const rb_tree) {
    let dst_tree: *mut rb_tree = core::ptr::addr_of_mut!((*(*stateblock).stateblock_state.light_state).lights_tree);
    let mut src_light: *mut wined3d_light_info = core::ptr::null_mut::<wined3d_light_info>();
    src_light = (rb_head((*src_tree).root) as *mut core::ffi::c_char)
        .offset(-(232 as core::ffi::c_ulonglong as isize))
        as *mut wined3d_light_info;
    while src_light
        != core::ptr::null_mut::<core::ffi::c_char>()
            .offset(-(232 as core::ffi::c_ulonglong as isize))
            as *mut wined3d_light_info
    {
        let dst_light: *mut wined3d_light_info =
            malloc(core::mem::size_of::<wined3d_light_info>())
                as *mut wined3d_light_info;
        *dst_light = *src_light;
        rb_put(
            dst_tree,
            (*dst_light).OriginalIndex as ULONG_PTR as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*dst_light).entry),
        );
        (*dst_light).changed = true;
        list_add_tail(
            core::ptr::addr_of_mut!((*stateblock).changed.changed_lights),
            core::ptr::addr_of_mut!((*dst_light).changed_entry),
        );
        src_light = (rb_next(core::ptr::addr_of_mut!((*src_light).entry)) as *mut core::ffi::c_char)
            .offset(-(232 as core::ffi::c_ulonglong as isize))
            as *mut wined3d_light_info;
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_incref(
    mut stateblock: *mut wined3d_stateblock,
) -> ULONG {
    let mut refcount: ::core::ffi::c_uint = InterlockedIncrement(
        &raw mut (*stateblock).ref_ as *mut ::core::ffi::c_long as *mut ::core::ffi::c_long,
    ) as ::core::ffi::c_uint;
    WINE_TRACE!("%p increasing refcount to %u.\n", stateblock, refcount);
    return refcount as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn state_unbind_resources(mut state: *mut wined3d_state) {
    let mut uav: *mut wined3d_unordered_access_view =
        ::core::ptr::null_mut::<wined3d_unordered_access_view>();
    let mut srv: *mut wined3d_shader_resource_view =
        ::core::ptr::null_mut::<wined3d_shader_resource_view>();
    let mut decl: *mut wined3d_vertex_declaration =
        ::core::ptr::null_mut::<wined3d_vertex_declaration>();
    let mut blend_state: *mut wined3d_blend_state = ::core::ptr::null_mut::<wined3d_blend_state>();
    let mut rtv: *mut wined3d_rendertarget_view =
        ::core::ptr::null_mut::<wined3d_rendertarget_view>();
    let mut sampler: *mut wined3d_sampler = ::core::ptr::null_mut::<wined3d_sampler>();
    let mut buffer: *mut wined3d_buffer = ::core::ptr::null_mut::<wined3d_buffer>();
    let mut shader: *mut wined3d_shader = ::core::ptr::null_mut::<wined3d_shader>();
    let mut i: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_uint = 0;
    decl = (*state).vertex_declaration;
    if !decl.is_null() {
        (*state).vertex_declaration = ::core::ptr::null_mut::<wined3d_vertex_declaration>();
        wined3d_vertex_declaration_decref(decl);
    }
    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_MAX_STREAM_OUTPUT_BUFFERS as ::core::ffi::c_uint {
        buffer = (*state).stream_output[i as usize].buffer;
        if !buffer.is_null() {
            (*state).stream_output[i as usize].buffer = ::core::ptr::null_mut::<wined3d_buffer>();
            wined3d_buffer_decref(buffer);
        }
        i = i.wrapping_add(1);
    }
    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_MAX_STREAMS as ::core::ffi::c_uint {
        buffer = (*state).streams[i as usize].buffer as *mut wined3d_buffer;
        if !buffer.is_null() {
            (*state).streams[i as usize].buffer = ::core::ptr::null_mut::<wined3d_buffer>();
            wined3d_buffer_decref(buffer);
        }
        i = i.wrapping_add(1);
    }
    buffer = (*state).index_buffer;
    if !buffer.is_null() {
        (*state).index_buffer = ::core::ptr::null_mut::<wined3d_buffer>();
        wined3d_buffer_decref(buffer);
    }
    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_SHADER_TYPE_COUNT as ::core::ffi::c_uint {
        shader = (*state).shader[i as usize];
        if !shader.is_null() {
            (*state).shader[i as usize] = ::core::ptr::null_mut::<wined3d_shader>();
            wined3d_shader_decref(shader);
        }
        j = 0 as ::core::ffi::c_uint;
        while j < MAX_CONSTANT_BUFFERS as ::core::ffi::c_uint {
            buffer = (*state).cb[i as usize][j as usize].buffer;
            if !buffer.is_null() {
                (*state).cb[i as usize][j as usize].buffer =
                    ::core::ptr::null_mut::<wined3d_buffer>();
                wined3d_buffer_decref(buffer);
            }
            j = j.wrapping_add(1);
        }
        j = 0 as ::core::ffi::c_uint;
        while j < MAX_SAMPLER_OBJECTS as ::core::ffi::c_uint {
            sampler = (*state).sampler[i as usize][j as usize];
            if !sampler.is_null() {
                (*state).sampler[i as usize][j as usize] =
                    ::core::ptr::null_mut::<wined3d_sampler>();
                wined3d_sampler_decref(sampler);
            }
            j = j.wrapping_add(1);
        }
        j = 0 as ::core::ffi::c_uint;
        while j < MAX_SHADER_RESOURCE_VIEWS as ::core::ffi::c_uint {
            srv = (*state).shader_resource_view[i as usize][j as usize];
            if !srv.is_null() {
                (*state).shader_resource_view[i as usize][j as usize] =
                    ::core::ptr::null_mut::<wined3d_shader_resource_view>();
                wined3d_srv_bind_count_dec(srv);
                wined3d_shader_resource_view_decref(srv);
            }
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_PIPELINE_COUNT as ::core::ffi::c_uint {
        j = 0 as ::core::ffi::c_uint;
        while j < MAX_UNORDERED_ACCESS_VIEWS as ::core::ffi::c_uint {
            uav = (*state).unordered_access_view[i as usize][j as usize];
            if !uav.is_null() {
                (*state).unordered_access_view[i as usize][j as usize] =
                    ::core::ptr::null_mut::<wined3d_unordered_access_view>();
                wined3d_unordered_access_view_decref(uav);
            }
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    blend_state = (*state).blend_state;
    if !blend_state.is_null() {
        (*state).blend_state = ::core::ptr::null_mut::<wined3d_blend_state>();
        wined3d_blend_state_decref(blend_state);
    }
    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!((*state).fb.render_targets) as usize {
        rtv = (*state).fb.render_targets[i as usize];
        if !rtv.is_null() {
            (*state).fb.render_targets[i as usize] =
                ::core::ptr::null_mut::<wined3d_rendertarget_view>();
            wined3d_rendertarget_view_decref(rtv);
        }
        i = i.wrapping_add(1);
    }
    rtv = (*state).fb.depth_stencil;
    if !rtv.is_null() {
        (*state).fb.depth_stencil = ::core::ptr::null_mut::<wined3d_rendertarget_view>();
        wined3d_rendertarget_view_decref(rtv);
    }
}

unsafe extern "C" fn wined3d_stateblock_state_cleanup(mut state: *mut wined3d_stateblock_state) {
    let mut light: *mut wined3d_light_info = ::core::ptr::null_mut::<wined3d_light_info>();
    let mut cursor: *mut wined3d_light_info = ::core::ptr::null_mut::<wined3d_light_info>();
    let mut decl: *mut wined3d_vertex_declaration =
        ::core::ptr::null_mut::<wined3d_vertex_declaration>();
    let mut texture: *mut wined3d_texture = ::core::ptr::null_mut::<wined3d_texture>();
    let mut buffer: *mut wined3d_buffer = ::core::ptr::null_mut::<wined3d_buffer>();
    let mut shader: *mut wined3d_shader = ::core::ptr::null_mut::<wined3d_shader>();
    let mut i: ::core::ffi::c_uint = 0;
    decl = (*state).vertex_declaration as *mut wined3d_vertex_declaration;
    if !decl.is_null() {
        (*state).vertex_declaration = ::core::ptr::null_mut::<wined3d_vertex_declaration>();
        wined3d_vertex_declaration_decref(decl);
    }
    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_MAX_STREAMS as ::core::ffi::c_uint {
        buffer = (*state).streams[i as usize].buffer as *mut wined3d_buffer;
        if !buffer.is_null() {
            (*state).streams[i as usize].buffer = ::core::ptr::null_mut::<wined3d_buffer>();
            wined3d_buffer_decref(buffer);
        }
        i = i.wrapping_add(1);
    }
    buffer = (*state).index_buffer;
    if !buffer.is_null() {
        (*state).index_buffer = ::core::ptr::null_mut::<wined3d_buffer>();
        wined3d_buffer_decref(buffer);
    }
    shader = (*state).vs as *mut wined3d_shader;
    if !shader.is_null() {
        (*state).vs = ::core::ptr::null_mut::<wined3d_shader>();
        wined3d_shader_decref(shader);
    }
    shader = (*state).ps;
    if !shader.is_null() {
        (*state).ps = ::core::ptr::null_mut::<wined3d_shader>();
        wined3d_shader_decref(shader);
    }
    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_MAX_COMBINED_SAMPLERS as ::core::ffi::c_uint {
        texture = (*state).textures[i as usize] as *mut wined3d_texture;
        if !texture.is_null() {
            (*state).textures[i as usize] = ::core::ptr::null_mut::<wined3d_texture>();
            wined3d_texture_decref(texture);
        }
        i = i.wrapping_add(1);
    }
    light = (rb_postorder_head((*(*state).light_state).lights_tree.root)
        as *mut ::core::ffi::c_char)
        .offset(-(232 as ::core::ffi::c_ulonglong as isize)) as *mut wined3d_light_info;
    while light
        != ::core::ptr::null_mut::<::core::ffi::c_char>()
            .offset(-(232 as ::core::ffi::c_ulonglong as isize))
            as *mut wined3d_light_info
        && {
            cursor = (rb_postorder_next(&raw mut (*light).entry) as *mut ::core::ffi::c_char)
                .offset(-(232 as ::core::ffi::c_ulonglong as isize))
                as *mut wined3d_light_info;
            !cursor.is_null() || 1 as ::core::ffi::c_int != 0
        }
    {
        if (*light).changed {
            list_remove(&raw mut (*light).changed_entry);
        }
        rb_remove(
            &raw mut (*(*state).light_state).lights_tree,
            &raw mut (*light).entry,
        );
        free(light as *mut ::core::ffi::c_void);
        light = cursor;
    }
}
#[no_mangle]
pub unsafe extern "C" fn state_cleanup(mut state: *mut wined3d_state) {
    let mut light: *mut wined3d_light_info = core::ptr::null_mut::<wined3d_light_info>();
    let mut cursor: *mut wined3d_light_info = core::ptr::null_mut::<wined3d_light_info>();
    let mut i: core::ffi::c_uint = 0;

    if ((*state).flags & WINED3D_STATE_NO_REF as uint32_t) == 0 {
        state_unbind_resources(state);
    }

    i = 0 as core::ffi::c_uint;
    while i < WINED3D_MAX_ACTIVE_LIGHTS as core::ffi::c_uint {
        (*state).light_state.lights[i as usize] = core::ptr::null::<wined3d_light_info>();
        i = i.wrapping_add(1);
    }

    light = (rb_postorder_head((*state).light_state.lights_tree.root) as *mut core::ffi::c_char)
        .offset(-(232 as isize)) as *mut wined3d_light_info;
    while light
        != (core::ptr::null_mut::<core::ffi::c_char>().offset(-(232 as isize)))
            as *mut wined3d_light_info
    {
        cursor = (rb_postorder_next(&raw mut (*light).entry) as *mut core::ffi::c_char)
            .offset(-(232 as isize)) as *mut wined3d_light_info;

        if (*light).changed {
            list_remove(&raw mut (*light).changed_entry);
        }
        rb_remove(
            &raw mut (*state).light_state.lights_tree,
            &raw mut (*light).entry,
        );
        free(light as *mut core::ffi::c_void);

        light = cursor;
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_decref(stateblock: *mut wined3d_stateblock) -> ULONG {
    let mut refcount: ::core::ffi::c_uint = InterlockedDecrement(
        core::ptr::addr_of_mut!((*stateblock).ref_) as *mut ::core::ffi::c_long,
    ) as ::core::ffi::c_uint;

    if refcount == 0 {
        wined3d_mutex_lock();
        wined3d_stateblock_state_cleanup(core::ptr::addr_of_mut!((*stateblock).stateblock_state));
        free((stateblock).cast());
        wined3d_mutex_unlock();
    }

    return refcount as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_light_state_get_light(state: *const wined3d_light_state, idx: ::core::ffi::c_uint) -> *mut wined3d_light_info {
    let mut entry: *mut rb_entry = core::ptr::null_mut();

    if {
        entry = rb_get(core::ptr::addr_of!((*state).lights_tree), idx as usize as *mut core::ffi::c_void);
        entry.is_null()
    } {
        return core::ptr::null_mut();
    }

    return (entry as *mut u8).sub(core::mem::offset_of!(wined3d_light_info, entry)) as *mut wined3d_light_info;
}

unsafe extern "C" fn set_light_changed(stateblock: *mut wined3d_stateblock, light_info: *mut wined3d_light_info) {
    if !(*light_info).changed {
        list_add_tail(
            core::ptr::addr_of_mut!((*stateblock).changed.changed_lights),
            core::ptr::addr_of_mut!((*light_info).changed_entry),
        );
        (*light_info).changed = true;
    }
    (*stateblock).changed.set_lights(1 as uint32_t);
}

unsafe extern "C" fn wined3d_light_state_set_light(
    state: *mut wined3d_light_state,
    light_idx: core::ffi::c_uint,
    params: *const wined3d_light,
    light_info: *mut *mut wined3d_light_info,
) -> HRESULT {
    let mut constants: *mut wined3d_light_constants = core::ptr::null_mut();
    let mut object: *mut wined3d_light_info = core::ptr::null_mut();

    object = wined3d_light_state_get_light(state, light_idx);
    if object.is_null() {
        TRACE!("Adding new light.\n");
        object = calloc(
            (1 as usize) as _,
            (core::mem::size_of::<wined3d_light_info>()) as _,
        ) as *mut wined3d_light_info;
        if object.is_null() {
            ERR!("Failed to allocate light info.\n");
            return E_OUTOFMEMORY;
        }

        (*object).glIndex = -(1 as LONG);
        (*object).OriginalIndex = light_idx;
        rb_put(
            core::ptr::addr_of_mut!((*state).lights_tree),
            light_idx as ULONG_PTR as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*object).entry),
        );
    }

    constants = core::ptr::addr_of_mut!((*object).constants);

    (*object).OriginalParms = *params;

    TRACE!(
        "Light %u setting to type %#x, diffuse %s, specular %s, ambient %s, position {%.8e, %.8e, %.8e}, direction {%.8e, %.8e, %.8e}, range %.8e, falloff %.8e, theta %.8e, phi %.8e.\n",
        light_idx,
        (*params).type_ as core::ffi::c_uint,
        debug_color(core::ptr::addr_of!((*params).diffuse)),
        debug_color(core::ptr::addr_of!((*params).specular)),
        debug_color(core::ptr::addr_of!((*params).ambient)),
        (*params).position.x as core::ffi::c_double,
        (*params).position.y as core::ffi::c_double,
        (*params).position.z as core::ffi::c_double,
        (*params).direction.x as core::ffi::c_double,
        (*params).direction.y as core::ffi::c_double,
        (*params).direction.z as core::ffi::c_double,
        (*params).range as core::ffi::c_double,
        (*params).falloff as core::ffi::c_double,
        (*params).theta as core::ffi::c_double,
        (*params).phi as core::ffi::c_double,
    );

    (*constants).diffuse = (*params).diffuse;
    (*constants).specular = (*params).specular;
    (*constants).ambient = (*params).ambient;

    (*constants).range = (*params).range;
    (*constants).falloff = (*params).falloff;
    (*constants).const_att = (*params).attenuation0;
    (*constants).linear_att = (*params).attenuation1;
    (*constants).quad_att = (*params).attenuation2;
    (*constants).cos_half_theta = cosf((*params).theta / 2.0f32);
    (*constants).cos_half_phi = cosf((*params).phi / 2.0f32);

    match (*params).type_ as core::ffi::c_uint {
        WINED3D_LIGHT_POINT => {
            /* Position */
            (*constants).position.x = (*params).position.x;
            (*constants).position.y = (*params).position.y;
            (*constants).position.z = (*params).position.z;
            (*constants).position.w = 1.0f32;
            /* FIXME: Range */
        }
        WINED3D_LIGHT_DIRECTIONAL => {
            /* Direction */
            (*constants).direction.x = -(*params).direction.x;
            (*constants).direction.y = -(*params).direction.y;
            (*constants).direction.z = -(*params).direction.z;
            (*constants).direction.w = 0.0f32;
        }
        WINED3D_LIGHT_SPOT => {
            /* Position */
            (*constants).position.x = (*params).position.x;
            (*constants).position.y = (*params).position.y;
            (*constants).position.z = (*params).position.z;
            (*constants).position.w = 1.0f32;

            /* Direction */
            (*constants).direction.x = (*params).direction.x;
            (*constants).direction.y = (*params).direction.y;
            (*constants).direction.z = (*params).direction.z;
            (*constants).direction.w = 0.0f32;

            /* FIXME: Range */
        }
        WINED3D_LIGHT_PARALLELPOINT => {
            (*constants).position.x = (*params).position.x;
            (*constants).position.y = (*params).position.y;
            (*constants).position.z = (*params).position.z;
            (*constants).position.w = 1.0f32;
        }
        _ => {
            FIXME!("Unrecognized params type %#x.\n", (*params).type_ as core::ffi::c_uint);
        }
    }

    *light_info = object;
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_light_state_enable_light(
    state: *mut wined3d_light_state,
    d3d_info: *const wined3d_d3d_info,
    light_info: *mut wined3d_light_info,
    enable: BOOL,
) -> bool {
    let mut light_count: ::core::ffi::c_uint = 0;
    let mut i: ::core::ffi::c_uint = 0;

    (*light_info).enabled = enable;
    if (*light_info).enabled == 0 {
        if (*light_info).glIndex == -(1) {
            TRACE!("Light already disabled, nothing to do.\n");
            return false;
        }

        (*state).lights[(*light_info).glIndex as usize] = core::ptr::null_mut();
        (*light_info).glIndex = -(1);
        return true;
    }

    if (*light_info).glIndex != -(1) {
        TRACE!("Light already enabled, nothing to do.\n");
        return false;
    }

    /* Find a free light. */
    light_count = (*d3d_info).limits.active_light_count;
    i = 0;
    while i < light_count {
        if !(*state).lights[i as usize].is_null() {
            i = i.wrapping_add(1);
        } else {
            (*state).lights[i as usize] = light_info;
            (*light_info).glIndex = i as LONG;
            return true;
        }
    }

    /* Our tests show that Windows returns D3D_OK in this situation, even with
     * D3DCREATE_HARDWARE_VERTEXPROCESSING | D3DCREATE_PUREDEVICE devices.
     * This is consistent among ddraw, d3d8 and d3d9. GetLightEnable returns
     * TRUE * as well for those lights.
     *
     * TODO: Test how this affects rendering. */
    WARN!("Too many concurrently active lights.\n");
    return false;
}

unsafe extern "C" fn wined3d_state_record_lights(
    dst_state: *mut wined3d_light_state,
    src_state: *const wined3d_light_state,
) {
    let mut src: *const wined3d_light_info = core::ptr::null::<wined3d_light_info>();
    let mut dst: *mut wined3d_light_info = core::ptr::null_mut::<wined3d_light_info>();

    /* Lights... For a recorded state block, we just had a chain of actions
     * to perform, so we need to walk that chain and update any actions which
     * differ. */
    dst = (rb_head((*dst_state).lights_tree.root) as *mut u8).offset(-232isize) as *mut wined3d_light_info;
    while dst != (core::ptr::null_mut::<u8>().offset(-232isize)) as *mut wined3d_light_info {
        src = wined3d_light_state_get_light(src_state, (*dst).OriginalIndex) as *const wined3d_light_info;
        if !src.is_null() {
            (*dst).OriginalParms = (*src).OriginalParms;

            if (*src).glIndex == -1 && (*dst).glIndex != -1 {
                /* Light disabled. */
                (*dst_state).lights[(*dst).glIndex as usize] = core::ptr::null_mut::<wined3d_light_info>();
            } else if (*src).glIndex != -1 && (*dst).glIndex == -1 {
                /* Light enabled. */
                (*dst_state).lights[(*src).glIndex as usize] = dst;
            }
            (*dst).glIndex = (*src).glIndex;
        } else {
            /* This can happen if the light was originally created as a
             * default light for SetLightEnable() while recording. */
            WARN!(
                "Light %u in dst_state %p does not exist in src_state %p.\n",
                (*dst).OriginalIndex,
                dst_state,
                src_state
            );

            (*dst).OriginalParms = WINED3D_default_light;
            if (*dst).glIndex != -1 {
                (*dst_state).lights[(*dst).glIndex as usize] = core::ptr::null_mut::<wined3d_light_info>();
                (*dst).glIndex = -1;
            }
        }
        dst = (rb_next(core::ptr::addr_of_mut!((*dst).entry)) as *mut u8).offset(-232isize)
            as *mut wined3d_light_info;
    }
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_capture(
    stateblock: *mut wined3d_stateblock,
    device_state: *const wined3d_stateblock,
) {
    let state: *const wined3d_stateblock_state = core::ptr::addr_of!((*device_state).stateblock_state);
    let mut range: wined3d_range = wined3d_range { offset: 0, size: 0 };
    let mut i: ::core::ffi::c_uint = 0;
    let mut start: ::core::ffi::c_uint = 0;
    let mut map: uint32_t = 0;

    TRACE!("stateblock %p, device_state %p.\n", stateblock, device_state);

    if (*stateblock).changed.vertexShader() as ::core::ffi::c_int != 0
        && (*stateblock).stateblock_state.vs != (*state).vs
    {
        TRACE!(
            "Updating vertex shader from %p to %p.\n",
            (*stateblock).stateblock_state.vs,
            (*state).vs
        );
        if !(*state).vs.is_null() {
            wined3d_shader_incref((*state).vs as *mut wined3d_shader);
        }
        if !(*stateblock).stateblock_state.vs.is_null() {
            wined3d_shader_decref((*stateblock).stateblock_state.vs as *mut wined3d_shader);
        }
        (*stateblock).stateblock_state.vs = (*state).vs;
    }

    start = 0;
    loop {
        if wined3d_bitmap_get_range(
            core::ptr::addr_of_mut!((*stateblock).changed.vs_consts_f) as *mut uint32_t,
            WINED3D_MAX_VS_CONSTS_F,
            start,
            core::ptr::addr_of_mut!(range),
        ) == 0
        {
            break;
        }
        memcpy(
            (core::ptr::addr_of_mut!((*stateblock).stateblock_state.vs_consts_f) as *mut wined3d_vec4)
                .offset(range.offset as isize) as *mut ::core::ffi::c_void,
            (core::ptr::addr_of!((*state).vs_consts_f) as *const wined3d_vec4)
                .offset(range.offset as isize) as *const ::core::ffi::c_void,
            (core::mem::size_of::<wined3d_vec4>() as usize).wrapping_mul(range.size as usize),
        );
        start = range.offset.wrapping_add(range.size);
    }

    map = (*stateblock).changed.vertexShaderConstantsI as uint32_t;
    start = 0;
    loop {
        if wined3d_bitmap_get_range(
            core::ptr::addr_of_mut!(map),
            WINED3D_MAX_CONSTS_I,
            start,
            core::ptr::addr_of_mut!(range),
        ) == 0
        {
            break;
        }
        memcpy(
            (core::ptr::addr_of_mut!((*stateblock).stateblock_state.vs_consts_i) as *mut wined3d_ivec4)
                .offset(range.offset as isize) as *mut ::core::ffi::c_void,
            (core::ptr::addr_of!((*state).vs_consts_i) as *const wined3d_ivec4)
                .offset(range.offset as isize) as *const ::core::ffi::c_void,
            (core::mem::size_of::<wined3d_ivec4>() as usize).wrapping_mul(range.size as usize),
        );
        start = range.offset.wrapping_add(range.size);
    }

    map = (*stateblock).changed.vertexShaderConstantsB as uint32_t;
    start = 0;
    loop {
        if wined3d_bitmap_get_range(
            core::ptr::addr_of_mut!(map),
            WINED3D_MAX_CONSTS_B,
            start,
            core::ptr::addr_of_mut!(range),
        ) == 0
        {
            break;
        }
        memcpy(
            (core::ptr::addr_of_mut!((*stateblock).stateblock_state.vs_consts_b) as *mut BOOL)
                .offset(range.offset as isize) as *mut ::core::ffi::c_void,
            (core::ptr::addr_of!((*state).vs_consts_b) as *const BOOL)
                .offset(range.offset as isize) as *const ::core::ffi::c_void,
            (core::mem::size_of::<BOOL>() as usize).wrapping_mul(range.size as usize),
        );
        start = range.offset.wrapping_add(range.size);
    }

    start = 0;
    loop {
        if wined3d_bitmap_get_range(
            core::ptr::addr_of_mut!((*stateblock).changed.ps_consts_f) as *mut uint32_t,
            WINED3D_MAX_PS_CONSTS_F,
            start,
            core::ptr::addr_of_mut!(range),
        ) == 0
        {
            break;
        }
        memcpy(
            (core::ptr::addr_of_mut!((*stateblock).stateblock_state.ps_consts_f) as *mut wined3d_vec4)
                .offset(range.offset as isize) as *mut ::core::ffi::c_void,
            (core::ptr::addr_of!((*state).ps_consts_f) as *const wined3d_vec4)
                .offset(range.offset as isize) as *const ::core::ffi::c_void,
            (core::mem::size_of::<wined3d_vec4>() as usize).wrapping_mul(range.size as usize),
        );
        start = range.offset.wrapping_add(range.size);
    }

    map = (*stateblock).changed.pixelShaderConstantsI as uint32_t;
    start = 0;
    loop {
        if wined3d_bitmap_get_range(
            core::ptr::addr_of_mut!(map),
            WINED3D_MAX_CONSTS_I,
            start,
            core::ptr::addr_of_mut!(range),
        ) == 0
        {
            break;
        }
        memcpy(
            (core::ptr::addr_of_mut!((*stateblock).stateblock_state.ps_consts_i) as *mut wined3d_ivec4)
                .offset(range.offset as isize) as *mut ::core::ffi::c_void,
            (core::ptr::addr_of!((*state).ps_consts_i) as *const wined3d_ivec4)
                .offset(range.offset as isize) as *const ::core::ffi::c_void,
            (core::mem::size_of::<wined3d_ivec4>() as usize).wrapping_mul(range.size as usize),
        );
        start = range.offset.wrapping_add(range.size);
    }

    map = (*stateblock).changed.pixelShaderConstantsB as uint32_t;
    start = 0;
    loop {
        if wined3d_bitmap_get_range(
            core::ptr::addr_of_mut!(map),
            WINED3D_MAX_CONSTS_B,
            start,
            core::ptr::addr_of_mut!(range),
        ) == 0
        {
            break;
        }
        memcpy(
            (core::ptr::addr_of_mut!((*stateblock).stateblock_state.ps_consts_b) as *mut BOOL)
                .offset(range.offset as isize) as *mut ::core::ffi::c_void,
            (core::ptr::addr_of!((*state).ps_consts_b) as *const BOOL)
                .offset(range.offset as isize) as *const ::core::ffi::c_void,
            (core::mem::size_of::<BOOL>() as usize).wrapping_mul(range.size as usize),
        );
        start = range.offset.wrapping_add(range.size);
    }

    if (*stateblock).changed.transforms() as ::core::ffi::c_int != 0 {
        i = 0;
        while i < (*stateblock).num_contained_transform_states {
            let transform: wined3d_transform_state =
                (*stateblock).contained_transform_states[i as usize] as wined3d_transform_state;
            TRACE!("Updating transform %#x.\n", transform as ::core::ffi::c_uint);
            (*stateblock).stateblock_state.transforms[transform as usize] =
                (*state).transforms[transform as usize];
            i = i.wrapping_add(1);
        }
    }

    if (*stateblock).changed.indices() as ::core::ffi::c_int != 0
        && ((*stateblock).stateblock_state.index_buffer != (*state).index_buffer
            || (*stateblock).stateblock_state.base_vertex_index != (*state).base_vertex_index
            || (*stateblock).stateblock_state.index_format as ::core::ffi::c_uint
                != (*state).index_format as ::core::ffi::c_uint)
    {
        TRACE!(
            "Updating index buffer to %p, base vertex index to %d.\n",
            (*state).index_buffer,
            (*state).base_vertex_index
        );
        if !(*state).index_buffer.is_null() {
            wined3d_buffer_incref((*state).index_buffer);
        }
        if !(*stateblock).stateblock_state.index_buffer.is_null() {
            wined3d_buffer_decref((*stateblock).stateblock_state.index_buffer);
        }
        (*stateblock).stateblock_state.index_buffer = (*state).index_buffer;
        (*stateblock).stateblock_state.base_vertex_index = (*state).base_vertex_index;
        (*stateblock).stateblock_state.index_format = (*state).index_format;
    }

    if (*stateblock).changed.vertexDecl() as ::core::ffi::c_int != 0
        && (*stateblock).stateblock_state.vertex_declaration != (*state).vertex_declaration
    {
        TRACE!(
            "Updating vertex declaration from %p to %p.\n",
            (*stateblock).stateblock_state.vertex_declaration,
            (*state).vertex_declaration
        );
        if !(*state).vertex_declaration.is_null() {
            wined3d_vertex_declaration_incref(
                (*state).vertex_declaration as *mut wined3d_vertex_declaration,
            );
        }
        if !(*stateblock).stateblock_state.vertex_declaration.is_null() {
            wined3d_vertex_declaration_decref(
                (*stateblock).stateblock_state.vertex_declaration
                    as *mut wined3d_vertex_declaration,
            );
        }
        (*stateblock).stateblock_state.vertex_declaration = (*state).vertex_declaration;
    }

    if (*stateblock).changed.material() as ::core::ffi::c_int != 0
        && memcmp(
            core::ptr::addr_of!((*state).material) as *const ::core::ffi::c_void,
            core::ptr::addr_of_mut!((*stateblock).stateblock_state.material)
                as *const ::core::ffi::c_void,
            core::mem::size_of::<wined3d_material>() as usize,
        ) != 0
    {
        TRACE!("Updating material.\n");
        (*stateblock).stateblock_state.material = (*state).material;
    }

    if (*stateblock).changed.viewport() as ::core::ffi::c_int != 0
        && memcmp(
            core::ptr::addr_of!((*state).viewport) as *const ::core::ffi::c_void,
            core::ptr::addr_of_mut!((*stateblock).stateblock_state.viewport)
                as *const ::core::ffi::c_void,
            core::mem::size_of::<wined3d_viewport>() as usize,
        ) != 0
    {
        TRACE!("Updating viewport.\n");
        (*stateblock).stateblock_state.viewport = (*state).viewport;
    }

    if (*stateblock).changed.scissorRect() as ::core::ffi::c_int != 0
        && memcmp(
            core::ptr::addr_of!((*state).scissor_rect) as *const ::core::ffi::c_void,
            core::ptr::addr_of_mut!((*stateblock).stateblock_state.scissor_rect)
                as *const ::core::ffi::c_void,
            core::mem::size_of::<RECT>() as usize,
        ) != 0
    {
        TRACE!("Updating scissor rect.\n");
        (*stateblock).stateblock_state.scissor_rect = (*state).scissor_rect;
    }

    map = (*stateblock).changed.streamSource as uint32_t;
    while map != 0 {
        i = wined3d_bit_scan(core::ptr::addr_of_mut!(map)) as ::core::ffi::c_uint;
        if (*stateblock).stateblock_state.streams[i as usize].stride
            != (*state).streams[i as usize].stride
            || (*stateblock).stateblock_state.streams[i as usize].offset
                != (*state).streams[i as usize].offset
            || (*stateblock).stateblock_state.streams[i as usize].buffer
                != (*state).streams[i as usize].buffer
        {
            TRACE!(
                "stateblock %p, stream source %u, buffer %p, stride %u, offset %u.\n",
                stateblock,
                i,
                (*state).streams[i as usize].buffer,
                (*state).streams[i as usize].stride,
                (*state).streams[i as usize].offset
            );
            (*stateblock).stateblock_state.streams[i as usize].stride =
                (*state).streams[i as usize].stride;
            if (*stateblock).changed.store_stream_offset() as ::core::ffi::c_int != 0 {
                (*stateblock).stateblock_state.streams[i as usize].offset =
                    (*state).streams[i as usize].offset;
            }
            if !(*state).streams[i as usize].buffer.is_null() {
                wined3d_buffer_incref(
                    (*state).streams[i as usize].buffer as *mut wined3d_buffer,
                );
            }
            if !(*stateblock).stateblock_state.streams[i as usize]
                .buffer
                .is_null()
            {
                wined3d_buffer_decref(
                    (*stateblock).stateblock_state.streams[i as usize].buffer
                        as *mut wined3d_buffer,
                );
            }
            (*stateblock).stateblock_state.streams[i as usize].buffer =
                (*state).streams[i as usize].buffer;
        }
    }

    map = (*stateblock).changed.streamFreq as uint32_t;
    while map != 0 {
        i = wined3d_bit_scan(core::ptr::addr_of_mut!(map)) as ::core::ffi::c_uint;
        if (*stateblock).stateblock_state.streams[i as usize].frequency
            != (*state).streams[i as usize].frequency
            || (*stateblock).stateblock_state.streams[i as usize].flags
                != (*state).streams[i as usize].flags
        {
            TRACE!(
                "Updating stream frequency %u to %u flags to %#x.\n",
                i,
                (*state).streams[i as usize].frequency,
                (*state).streams[i as usize].flags
            );
            (*stateblock).stateblock_state.streams[i as usize].frequency =
                (*state).streams[i as usize].frequency;
            (*stateblock).stateblock_state.streams[i as usize].flags =
                (*state).streams[i as usize].flags;
        }
    }

    map = (*stateblock).changed.clipplane;
    while map != 0 {
        i = wined3d_bit_scan(core::ptr::addr_of_mut!(map)) as ::core::ffi::c_uint;
        if memcmp(
            (core::ptr::addr_of_mut!((*stateblock).stateblock_state.clip_planes) as *mut wined3d_vec4)
                .offset(i as isize) as *const ::core::ffi::c_void,
            (core::ptr::addr_of!((*state).clip_planes) as *const wined3d_vec4)
                .offset(i as isize) as *const ::core::ffi::c_void,
            core::mem::size_of::<wined3d_vec4>() as usize,
        ) != 0
        {
            TRACE!("Updating clipplane %u.\n", i);
            (*stateblock).stateblock_state.clip_planes[i as usize] =
                (*state).clip_planes[i as usize];
        }
    }

    i = 0;
    while i < (*stateblock).num_contained_render_states {
        let rs: wined3d_render_state =
            (*stateblock).contained_render_states[i as usize] as wined3d_render_state;
        TRACE!(
            "Updating render state %#x to %u.\n",
            rs as ::core::ffi::c_uint,
            (*state).rs[rs as usize]
        );
        (*stateblock).stateblock_state.rs[rs as usize] = (*state).rs[rs as usize];
        i = i.wrapping_add(1);
    }

    i = 0;
    while i < (*stateblock).num_contained_tss_states {
        let stage: ::core::ffi::c_uint = (*stateblock).contained_tss_states[i as usize].stage;
        let texture_state: ::core::ffi::c_uint =
            (*stateblock).contained_tss_states[i as usize].state;
        TRACE!(
            "Updating texturestage state %u, %u to %#x (was %#x).\n",
            stage,
            texture_state,
            (*state).texture_states[stage as usize][texture_state as usize],
            (*stateblock).stateblock_state.texture_states[stage as usize][texture_state as usize]
        );
        (*stateblock).stateblock_state.texture_states[stage as usize][texture_state as usize] =
            (*state).texture_states[stage as usize][texture_state as usize];
        i = i.wrapping_add(1);
    }

    map = (*stateblock).changed.textures() as uint32_t;
    while map != 0 {
        i = wined3d_bit_scan(core::ptr::addr_of_mut!(map)) as ::core::ffi::c_uint;
        TRACE!(
            "Updating texture %u to %p (was %p).\n",
            i,
            (*state).textures[i as usize],
            (*stateblock).stateblock_state.textures[i as usize]
        );
        if !(*state).textures[i as usize].is_null() {
            wined3d_texture_incref((*state).textures[i as usize]);
        }
        if !(*stateblock).stateblock_state.textures[i as usize].is_null() {
            wined3d_texture_decref((*stateblock).stateblock_state.textures[i as usize]);
        }
        (*stateblock).stateblock_state.textures[i as usize] = (*state).textures[i as usize];
    }

    i = 0;
    while i < (*stateblock).num_contained_sampler_states {
        let stage_0: ::core::ffi::c_uint = (*stateblock).contained_sampler_states[i as usize].stage;
        let sampler_state: ::core::ffi::c_uint =
            (*stateblock).contained_sampler_states[i as usize].state;
        TRACE!(
            "Updating sampler state %u, %u to %#x (was %#x).\n",
            stage_0,
            sampler_state,
            (*state).sampler_states[stage_0 as usize][sampler_state as usize],
            (*stateblock).stateblock_state.sampler_states[stage_0 as usize][sampler_state as usize]
        );
        (*stateblock).stateblock_state.sampler_states[stage_0 as usize][sampler_state as usize] =
            (*state).sampler_states[stage_0 as usize][sampler_state as usize];
        i = i.wrapping_add(1);
    }

    if (*stateblock).changed.pixelShader() as ::core::ffi::c_int != 0
        && (*stateblock).stateblock_state.ps != (*state).ps
    {
        if !(*state).ps.is_null() {
            wined3d_shader_incref((*state).ps);
        }
        if !(*stateblock).stateblock_state.ps.is_null() {
            wined3d_shader_decref((*stateblock).stateblock_state.ps);
        }
        (*stateblock).stateblock_state.ps = (*state).ps;
    }

    if (*stateblock).changed.lights() as ::core::ffi::c_int != 0 {
        wined3d_state_record_lights(
            (*stateblock).stateblock_state.light_state as *mut wined3d_light_state,
            (*state).light_state,
        );
    }

    if (*stateblock).changed.alpha_to_coverage() as ::core::ffi::c_int != 0 {
        (*stateblock).stateblock_state.alpha_to_coverage = (*state).alpha_to_coverage;
    }

    TRACE!("Capture done.\n");
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_apply(
    stateblock: *const wined3d_stateblock,
    device_state: *mut wined3d_stateblock,
) {
    let state: *const wined3d_stateblock_state = &raw const (*stateblock).stateblock_state;
    let mut range: wined3d_range = wined3d_range { offset: 0, size: 0 };
    let mut i: ::core::ffi::c_uint = 0;
    let mut start: ::core::ffi::c_uint = 0;
    let mut map: uint32_t = 0;
    if (*stateblock).changed.vertexShader() != 0 {
        wined3d_stateblock_set_vertex_shader(device_state, (*state).vs as *mut wined3d_shader);
    }
    if (*stateblock).changed.pixelShader() != 0 {
        wined3d_stateblock_set_pixel_shader(device_state, (*state).ps);
    }
    start = 0 as ::core::ffi::c_uint;
    while !(wined3d_bitmap_get_range(
        &raw const (*stateblock).changed.vs_consts_f as *const uint32_t,
        WINED3D_MAX_VS_CONSTS_F as ::core::ffi::c_uint,
        start,
        &raw mut range,
    ) == 0)
    {
        wined3d_stateblock_set_vs_consts_f(
            device_state,
            range.offset,
            range.size,
            (&raw const (*state).vs_consts_f as *const wined3d_vec4).offset(range.offset as isize)
                as *const wined3d_vec4,
        );
        start = range.offset.wrapping_add(range.size);
    }
    map = (*stateblock).changed.vertexShaderConstantsI as uint32_t;
    start = 0 as ::core::ffi::c_uint;
    while !(wined3d_bitmap_get_range(
        &raw mut map,
        WINED3D_MAX_CONSTS_I as ::core::ffi::c_uint,
        start,
        &raw mut range,
    ) == 0)
    {
        wined3d_stateblock_set_vs_consts_i(
            device_state,
            range.offset,
            range.size,
            (&raw const (*state).vs_consts_i as *const wined3d_ivec4).offset(range.offset as isize)
                as *const wined3d_ivec4,
        );
        start = range.offset.wrapping_add(range.size);
    }
    map = (*stateblock).changed.vertexShaderConstantsB as uint32_t;
    start = 0 as ::core::ffi::c_uint;
    while !(wined3d_bitmap_get_range(
        &raw mut map,
        WINED3D_MAX_CONSTS_B as ::core::ffi::c_uint,
        start,
        &raw mut range,
    ) == 0)
    {
        wined3d_stateblock_set_vs_consts_b(
            device_state,
            range.offset,
            range.size,
            (&raw const (*state).vs_consts_b as *const BOOL).offset(range.offset as isize)
                as *const BOOL,
        );
        start = range.offset.wrapping_add(range.size);
    }

    start = 0 as ::core::ffi::c_uint;
    while !(wined3d_bitmap_get_range(
        &raw const (*stateblock).changed.ps_consts_f as *const uint32_t,
        WINED3D_MAX_PS_CONSTS_F as ::core::ffi::c_uint,
        start,
        &raw mut range,
    ) == 0)
    {
        wined3d_stateblock_set_ps_consts_f(
            device_state,
            range.offset,
            range.size,
            (&raw const (*state).ps_consts_f as *const wined3d_vec4).offset(range.offset as isize)
                as *const wined3d_vec4,
        );
        start = range.offset.wrapping_add(range.size);
    }
    map = (*stateblock).changed.pixelShaderConstantsI as uint32_t;
    start = 0 as ::core::ffi::c_uint;
    while !(wined3d_bitmap_get_range(
        &raw mut map,
        WINED3D_MAX_CONSTS_I as ::core::ffi::c_uint,
        start,
        &raw mut range,
    ) == 0)
    {
        wined3d_stateblock_set_ps_consts_i(
            device_state,
            range.offset,
            range.size,
            (&raw const (*state).ps_consts_i as *const wined3d_ivec4).offset(range.offset as isize)
                as *const wined3d_ivec4,
        );
        start = range.offset.wrapping_add(range.size);
    }
    map = (*stateblock).changed.pixelShaderConstantsB as uint32_t;
    start = 0 as ::core::ffi::c_uint;
    while !(wined3d_bitmap_get_range(
        &raw mut map,
        WINED3D_MAX_CONSTS_B as ::core::ffi::c_uint,
        start,
        &raw mut range,
    ) == 0)
    {
        wined3d_stateblock_set_ps_consts_b(
            device_state,
            range.offset,
            range.size,
            (&raw const (*state).ps_consts_b as *const BOOL).offset(range.offset as isize)
                as *const BOOL,
        );
        start = range.offset.wrapping_add(range.size);
    }

    if (*stateblock).changed.transforms() != 0 {
        i = 0 as ::core::ffi::c_uint;
        while i < (*stateblock).num_contained_transform_states {
            let transform: wined3d_transform_state =
                (*stateblock).contained_transform_states[i as usize] as wined3d_transform_state;
            wined3d_stateblock_set_transform(
                device_state,
                transform,
                (&raw const (*state).transforms as *const wined3d_matrix).offset(transform as isize)
                    as *const wined3d_matrix,
            );
            i = i.wrapping_add(1);
        }
    }

    if (*stateblock).changed.lights() != 0 {
        let mut light: *const wined3d_light_info = ::core::ptr::null::<wined3d_light_info>();
        light = ((*stateblock).changed.changed_lights.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(wined3d_light_info, changed_entry)
                as ::core::ffi::c_ulonglong as isize),
        ) as *mut wined3d_light_info;
        while &raw const (*light).changed_entry != &raw const (*stateblock).changed.changed_lights {
            wined3d_stateblock_set_light(
                device_state,
                (*light).OriginalIndex as UINT,
                &raw const (*light).OriginalParms,
            );
            wined3d_stateblock_set_light_enable(
                device_state,
                (*light).OriginalIndex as UINT,
                ((*light).glIndex != -1) as ::core::ffi::c_int,
            );
            light = ((*light).changed_entry.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(wined3d_light_info, changed_entry)
                    as ::core::ffi::c_ulonglong as isize),
            ) as *mut wined3d_light_info;
        }
    }

    if (*stateblock).changed.alpha_to_coverage() != 0 {
        (*device_state).stateblock_state.alpha_to_coverage = (*state).alpha_to_coverage;
        (*device_state)
            .changed
            .set_alpha_to_coverage(1 as uint32_t);
    }

    /* Render states. */
    i = 0 as ::core::ffi::c_uint;
    while i < (*stateblock).num_contained_render_states {
        let rs: wined3d_render_state =
            (*stateblock).contained_render_states[i as usize] as wined3d_render_state;
        wined3d_stateblock_set_render_state(device_state, rs, (*state).rs[rs as usize]);
        i = i.wrapping_add(1);
    }

    /* Texture states. */
    i = 0 as ::core::ffi::c_uint;
    while i < (*stateblock).num_contained_tss_states {
        let stage: DWORD = (*stateblock).contained_tss_states[i as usize].stage as DWORD;
        let texture_state: DWORD = (*stateblock).contained_tss_states[i as usize].state as DWORD;
        wined3d_stateblock_set_texture_stage_state(
            device_state,
            stage as UINT,
            texture_state as wined3d_texture_stage_state,
            (*state).texture_states[stage as usize][texture_state as usize],
        );
        i = i.wrapping_add(1);
    }

    /* Sampler states. */
    i = 0 as ::core::ffi::c_uint;
    while i < (*stateblock).num_contained_sampler_states {
        let stage: DWORD = (*stateblock).contained_sampler_states[i as usize].stage as DWORD;
        let sampler_state: DWORD = (*stateblock).contained_sampler_states[i as usize].state as DWORD;
        wined3d_stateblock_set_sampler_state(
            device_state,
            stage as UINT,
            sampler_state as wined3d_sampler_state,
            (*state).sampler_states[stage as usize][sampler_state as usize],
        );
        i = i.wrapping_add(1);
    }

    if (*stateblock).changed.indices() != 0 {
        wined3d_stateblock_set_index_buffer(
            device_state,
            (*state).index_buffer,
            (*state).index_format,
        );
        wined3d_stateblock_set_base_vertex_index(device_state, (*state).base_vertex_index as INT);
    }

    if (*stateblock).changed.vertexDecl() as ::core::ffi::c_int != 0
        && !(*state).vertex_declaration.is_null()
    {
        wined3d_stateblock_set_vertex_declaration(
            device_state,
            (*state).vertex_declaration as *mut wined3d_vertex_declaration,
        );
    }

    if (*stateblock).changed.material() != 0 {
        wined3d_stateblock_set_material(device_state, &raw const (*state).material);
    }

    if (*stateblock).changed.viewport() != 0 {
        wined3d_stateblock_set_viewport(device_state, &raw const (*state).viewport);
    }

    if (*stateblock).changed.scissorRect() != 0 {
        wined3d_stateblock_set_scissor_rect(device_state, &raw const (*state).scissor_rect);
    }

    map = (*stateblock).changed.streamSource as uint32_t;
    while map != 0 {
        i = wined3d_bit_scan(&raw mut map) as ::core::ffi::c_uint;
        wined3d_stateblock_set_stream_source(
            device_state,
            i as UINT,
            (*state).streams[i as usize].buffer as *mut wined3d_buffer,
            (*state).streams[i as usize].offset,
            (*state).streams[i as usize].stride,
        );
    }

    map = (*stateblock).changed.streamFreq as uint32_t;
    while map != 0 {
        i = wined3d_bit_scan(&raw mut map) as ::core::ffi::c_uint;
        wined3d_stateblock_set_stream_source_freq(
            device_state,
            i as UINT,
            (*state).streams[i as usize].frequency | (*state).streams[i as usize].flags,
        );
    }

    map = (*stateblock).changed.textures();
    while map != 0 {
        i = wined3d_bit_scan(&raw mut map) as ::core::ffi::c_uint;
        wined3d_stateblock_set_texture(device_state, i as UINT, (*state).textures[i as usize]);
    }

    map = (*stateblock).changed.clipplane as uint32_t;
    while map != 0 {
        i = wined3d_bit_scan(&raw mut map) as ::core::ffi::c_uint;
        wined3d_stateblock_set_clip_plane(
            device_state,
            i as UINT,
            (&raw const (*state).clip_planes as *const wined3d_vec4).offset(i as isize)
                as *const wined3d_vec4,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_vertex_shader(
    mut stateblock: *mut wined3d_stateblock,
    mut shader: *mut wined3d_shader,
) {
    WINE_TRACE!("stateblock %p, shader %p.\n", stateblock, shader);
    if !shader.is_null() {
        wined3d_shader_incref(shader);
    }
    if !(*stateblock).stateblock_state.vs.is_null() {
        wined3d_shader_decref((*stateblock).stateblock_state.vs as *mut wined3d_shader);
    }
    (*stateblock).stateblock_state.vs = shader as *mut wined3d_shader;
    (*stateblock)
        .changed
        .set_vertexShader(TRUE as uint32_t as uint32_t);
    (*stateblock)
        .changed
        .set_ffp_vs_settings(1 as uint32_t as uint32_t);
}

unsafe extern "C" fn wined3d_bitmap_set_bits(mut bitmap: *mut u32, mut start: u32, mut count: u32) {
    let word_bit_count: u32 = (core::mem::size_of::<u32>() as u32).wrapping_mul(CHAR_BIT as u32);
    let shift: u32 = start.wrapping_rem(word_bit_count);
    let mut mask: u32 = 0;
    let mut last_mask: u32 = 0;
    let mut mask_size: u32 = 0;

    bitmap = bitmap.offset((start.wrapping_div(word_bit_count)) as isize);
    mask = (!(0 as u32)) << shift;
    mask_size = word_bit_count.wrapping_sub(shift);
    last_mask = ((1 as u32)
        << start.wrapping_add(count).wrapping_rem(word_bit_count))
        .wrapping_sub(1 as u32);
    if mask_size <= count {
        *bitmap |= mask;
        bitmap = bitmap.offset(1);
        count = count.wrapping_sub(mask_size);
        mask = !(0 as u32);
    }
    if count >= word_bit_count {
        memset(
            bitmap as *mut core::ffi::c_void,
            0xff as core::ffi::c_int,
            (count / word_bit_count) as usize * core::mem::size_of::<u32>(),
        );
        bitmap = bitmap.offset((count / word_bit_count) as isize);
        count = count % word_bit_count;
    }
    if count != 0 {
        *bitmap |= mask & last_mask;
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_vs_consts_f(
    stateblock: *mut wined3d_stateblock,
    start_idx: ::core::ffi::c_uint,
    count: ::core::ffi::c_uint,
    constants: *const wined3d_vec4,
) -> HRESULT {
    let d3d_info: *const wined3d_d3d_info =
        core::ptr::addr_of!((*(*(*stateblock).device).adapter).d3d_info);

    if constants.is_null()
        || !wined3d_bound_range(
            start_idx,
            count,
            (*d3d_info).limits.vs_uniform_count as ::core::ffi::c_uint,
        )
    {
        return WINED3DERR_INVALIDCALL;
    }

    memcpy(
        (core::ptr::addr_of_mut!((*stateblock).stateblock_state.vs_consts_f) as *mut wined3d_vec4)
            .offset(start_idx as isize) as *mut ::core::ffi::c_void,
        constants as *const ::core::ffi::c_void,
        (count as usize).wrapping_mul(core::mem::size_of::<wined3d_vec4>()),
    );
    wined3d_bitmap_set_bits(
        core::ptr::addr_of_mut!((*stateblock).changed.vs_consts_f) as *mut uint32_t,
        start_idx,
        count,
    );
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_vs_consts_i(
    mut stateblock: *mut wined3d_stateblock,
    mut start_idx: ::core::ffi::c_uint,
    mut count: ::core::ffi::c_uint,
    mut constants: *const wined3d_ivec4,
) -> HRESULT {
    let mut i: ::core::ffi::c_uint = 0;
    if constants.is_null() || start_idx >= WINED3D_MAX_CONSTS_I as ::core::ffi::c_uint {
        return WINED3DERR_INVALIDCALL;
    }
    if count > (WINED3D_MAX_CONSTS_I as ::core::ffi::c_uint).wrapping_sub(start_idx) {
        count = (WINED3D_MAX_CONSTS_I as ::core::ffi::c_uint).wrapping_sub(start_idx);
    }
    memcpy(
        (&raw mut (*stateblock).stateblock_state.vs_consts_i as *mut wined3d_ivec4)
            .offset(start_idx as isize) as *mut ::core::ffi::c_void,
        constants as *const ::core::ffi::c_void,
        (count as usize).wrapping_mul(::core::mem::size_of::<wined3d_ivec4>()),
    );
    i = start_idx;
    while i < count.wrapping_add(start_idx) {
        (*stateblock).changed.vertexShaderConstantsI =
            ((*stateblock).changed.vertexShaderConstantsI as ::core::ffi::c_uint
                | ((1) as ::core::ffi::c_uint) << i) as uint16_t;
        i = i.wrapping_add(1);
    }
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_vs_consts_b(
    stateblock: *mut wined3d_stateblock,
    start_idx: u32,
    mut count: u32,
    constants: *const BOOL,
) -> HRESULT {
    let mut i: u32;

    TRACE!(
        "stateblock %p, start_idx %u, count %u, constants %p.\n",
        stateblock,
        start_idx,
        count,
        constants
    );

    if constants.is_null() || start_idx >= WINED3D_MAX_CONSTS_B as u32 {
        return WINED3DERR_INVALIDCALL;
    }

    if count > (WINED3D_MAX_CONSTS_B as u32).wrapping_sub(start_idx) {
        count = (WINED3D_MAX_CONSTS_B as u32).wrapping_sub(start_idx);
    }

    memcpy(
        (core::ptr::addr_of_mut!((*stateblock).stateblock_state.vs_consts_b) as *mut BOOL)
            .add(start_idx as usize) as *mut core::ffi::c_void,
        constants as *const core::ffi::c_void,
        (count as usize).wrapping_mul(core::mem::size_of::<BOOL>()),
    );

    i = start_idx;
    while i < count.wrapping_add(start_idx) {
        (*stateblock).changed.vertexShaderConstantsB = (((*stateblock).changed.vertexShaderConstantsB
            as u32)
            | ((1 as u32) << i)) as u16;
        i = i.wrapping_add(1);
    }
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_get_vs_consts_f(
    stateblock: *mut wined3d_stateblock,
    start_idx: core::ffi::c_uint,
    count: core::ffi::c_uint,
    constants: *mut wined3d_vec4,
) -> HRESULT {
    let d3d_info: *const wined3d_d3d_info =
        core::ptr::addr_of!((*(*(*stateblock).device).adapter).d3d_info);

    if constants.is_null()
        || !wined3d_bound_range(
            start_idx,
            count,
            (*d3d_info).limits.vs_uniform_count as core::ffi::c_uint,
        )
    {
        return WINED3DERR_INVALIDCALL;
    }
    memcpy(
        constants as *mut core::ffi::c_void,
        (core::ptr::addr_of_mut!((*stateblock).stateblock_state.vs_consts_f) as *mut wined3d_vec4)
            .offset(start_idx as isize) as *const core::ffi::c_void,
        (count as usize).wrapping_mul(core::mem::size_of::<wined3d_vec4>()),
    );
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_get_vs_consts_i(
    stateblock: *mut wined3d_stateblock,
    start_idx: u32,
    mut count: u32,
    constants: *mut wined3d_ivec4,
) -> HRESULT {
    if constants.is_null() || start_idx >= (WINED3D_MAX_CONSTS_I) as u32 {
        return (WINED3DERR_INVALIDCALL) as HRESULT;
    }
    if count > ((WINED3D_MAX_CONSTS_I) as u32).wrapping_sub(start_idx) {
        count = ((WINED3D_MAX_CONSTS_I) as u32).wrapping_sub(start_idx);
    }
    memcpy(
        constants as *mut core::ffi::c_void,
        (core::ptr::addr_of_mut!((*stateblock).stateblock_state.vs_consts_i) as *mut wined3d_ivec4)
            .offset(start_idx as isize) as *mut wined3d_ivec4 as *const core::ffi::c_void,
        (count as usize).wrapping_mul(core::mem::size_of::<wined3d_ivec4>()),
    );
    return (WINED3D_OK) as HRESULT;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_get_vs_consts_b(
    stateblock: *mut wined3d_stateblock,
    start_idx: ::core::ffi::c_uint,
    mut count: ::core::ffi::c_uint,
    constants: *mut BOOL,
) -> HRESULT {
    if constants.is_null() || start_idx >= WINED3D_MAX_CONSTS_B as ::core::ffi::c_uint {
        return WINED3DERR_INVALIDCALL;
    }
    if count > (WINED3D_MAX_CONSTS_B as ::core::ffi::c_uint).wrapping_sub(start_idx) {
        count = (WINED3D_MAX_CONSTS_B as ::core::ffi::c_uint).wrapping_sub(start_idx);
    }
    memcpy(
        constants as *mut ::core::ffi::c_void,
        (core::ptr::addr_of_mut!((*stateblock).stateblock_state.vs_consts_b) as *mut BOOL)
            .offset(start_idx as isize) as *const ::core::ffi::c_void,
        (count as usize).wrapping_mul(::core::mem::size_of::<BOOL>()),
    );
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_pixel_shader(stateblock: *mut wined3d_stateblock, shader: *mut wined3d_shader) {
    if !shader.is_null() {
        wined3d_shader_incref(shader);
    }
    if !(*stateblock).stateblock_state.ps.is_null() {
        wined3d_shader_decref((*stateblock).stateblock_state.ps);
    }
    (*stateblock).stateblock_state.ps = shader;
    (*stateblock).changed.set_pixelShader((TRUE) as _);
    (*stateblock).changed.set_ffp_ps_settings((1) as _);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_ps_consts_f(
    mut stateblock: *mut wined3d_stateblock,
    mut start_idx: ::core::ffi::c_uint,
    mut count: ::core::ffi::c_uint,
    mut constants: *const wined3d_vec4,
) -> HRESULT {
    let mut d3d_info: *const wined3d_d3d_info =
        core::ptr::addr_of!((*(*(*stateblock).device).adapter).d3d_info);

    // TRACE("stateblock %p, start_idx %u, count %u, constants %p.\n", ...)
    // (trace channel output is not reproduced; no runtime effect.)

    if constants.is_null()
        || !wined3d_bound_range(
            start_idx,
            count,
            (*d3d_info).limits.ps_uniform_count as ::core::ffi::c_uint,
        )
    {
        return WINED3DERR_INVALIDCALL;
    }
    memcpy(
        (core::ptr::addr_of_mut!((*stateblock).stateblock_state.ps_consts_f) as *mut wined3d_vec4)
            .offset(start_idx as isize) as *mut wined3d_vec4 as *mut ::core::ffi::c_void,
        constants as *const ::core::ffi::c_void,
        (count as usize).wrapping_mul(::core::mem::size_of::<wined3d_vec4>()),
    );
    wined3d_bitmap_set_bits(
        core::ptr::addr_of_mut!((*stateblock).changed.ps_consts_f) as *mut uint32_t,
        start_idx,
        count,
    );
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_ps_consts_i(
    mut stateblock: *mut wined3d_stateblock,
    mut start_idx: ::core::ffi::c_uint,
    mut count: ::core::ffi::c_uint,
    mut constants: *const wined3d_ivec4,
) -> HRESULT {
    let mut i: ::core::ffi::c_uint = 0;
    WINE_TRACE!("stateblock %p, start_idx %u, count %u, constants %p.\n", stateblock, start_idx, count, constants);
    if constants.is_null() || start_idx >= WINED3D_MAX_CONSTS_I as ::core::ffi::c_uint {
        return WINED3DERR_INVALIDCALL;
    }
    if count > (WINED3D_MAX_CONSTS_I as ::core::ffi::c_uint).wrapping_sub(start_idx) {
        count = (WINED3D_MAX_CONSTS_I as ::core::ffi::c_uint).wrapping_sub(start_idx);
    }
    memcpy(
        (&raw mut (*stateblock).stateblock_state.ps_consts_i as *mut wined3d_ivec4)
            .offset(start_idx as isize) as *mut wined3d_ivec4 as *mut ::core::ffi::c_void,
        constants as *const ::core::ffi::c_void,
        (count as size_t).wrapping_mul(::core::mem::size_of::<wined3d_ivec4>() as size_t),
    );
    i = start_idx;
    while i < count.wrapping_add(start_idx) {
        (*stateblock).changed.pixelShaderConstantsI =
            ((*stateblock).changed.pixelShaderConstantsI as ::core::ffi::c_uint
                | (1 as ::core::ffi::c_uint) << i) as uint16_t;
        i = i.wrapping_add(1);
    }
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_ps_consts_b(
    mut stateblock: *mut wined3d_stateblock,
    mut start_idx: ::core::ffi::c_uint,
    mut count: ::core::ffi::c_uint,
    mut constants: *const BOOL,
) -> HRESULT {
    let mut i: ::core::ffi::c_uint = 0;
    WINE_TRACE!("stateblock %p, start_idx %u, count %u, constants %p.\n", stateblock, start_idx, count, constants);
    if constants.is_null() || start_idx >= WINED3D_MAX_CONSTS_B as ::core::ffi::c_uint {
        return WINED3DERR_INVALIDCALL;
    }
    if count > (WINED3D_MAX_CONSTS_B as ::core::ffi::c_uint).wrapping_sub(start_idx) {
        count = (WINED3D_MAX_CONSTS_B as ::core::ffi::c_uint).wrapping_sub(start_idx);
    }
    memcpy(
        (core::ptr::addr_of_mut!((*stateblock).stateblock_state.ps_consts_b) as *mut BOOL)
            .offset(start_idx as isize) as *mut BOOL as *mut ::core::ffi::c_void,
        constants as *const ::core::ffi::c_void,
        (count as usize).wrapping_mul(::core::mem::size_of::<BOOL>()),
    );
    i = start_idx;
    while i < count.wrapping_add(start_idx) {
        (*stateblock).changed.pixelShaderConstantsB =
            ((*stateblock).changed.pixelShaderConstantsB as ::core::ffi::c_uint
                | (1 as ::core::ffi::c_uint) << i) as uint16_t;
        i = i.wrapping_add(1);
    }
    return WINED3D_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_get_ps_consts_f(
    stateblock: *mut wined3d_stateblock,
    start_idx: core::ffi::c_uint,
    count: core::ffi::c_uint,
    constants: *mut wined3d_vec4,
) -> HRESULT {
    if constants.is_null()
        || wined3d_bound_range(start_idx, count, WINED3D_MAX_PS_CONSTS_F) == false
    {
        return WINED3DERR_INVALIDCALL;
    }

    memcpy(
        constants as *mut core::ffi::c_void,
        (core::ptr::addr_of_mut!((*stateblock).stateblock_state.ps_consts_f) as *mut wined3d_vec4)
            .offset(start_idx as isize) as *const core::ffi::c_void,
        (count as usize).wrapping_mul(core::mem::size_of::<wined3d_vec4>()),
    );
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_get_ps_consts_i(
    stateblock: *mut wined3d_stateblock,
    start_idx: ::core::ffi::c_uint,
    mut count: ::core::ffi::c_uint,
    constants: *mut wined3d_ivec4,
) -> HRESULT {
    if constants.is_null() || start_idx >= WINED3D_MAX_CONSTS_I {
        return WINED3DERR_INVALIDCALL;
    }

    if count > WINED3D_MAX_CONSTS_I - start_idx {
        count = WINED3D_MAX_CONSTS_I - start_idx;
    }

    memcpy(
        constants as *mut ::core::ffi::c_void,
        (core::ptr::addr_of_mut!((*stateblock).stateblock_state.ps_consts_i) as *mut wined3d_ivec4)
            .offset(start_idx as isize) as *const ::core::ffi::c_void,
        (count as size_t).wrapping_mul(core::mem::size_of::<wined3d_ivec4>() as size_t),
    );
    return WINED3D_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_get_ps_consts_b(
    mut stateblock: *mut wined3d_stateblock,
    mut start_idx: ::core::ffi::c_uint,
    mut count: ::core::ffi::c_uint,
    mut constants: *mut BOOL,
) -> HRESULT {
    if constants.is_null() || start_idx >= WINED3D_MAX_CONSTS_B as ::core::ffi::c_uint {
        return WINED3DERR_INVALIDCALL;
    }
    if count > (WINED3D_MAX_CONSTS_B as ::core::ffi::c_uint).wrapping_sub(start_idx) {
        count = (WINED3D_MAX_CONSTS_B as ::core::ffi::c_uint).wrapping_sub(start_idx);
    }
    memcpy(
        constants as *mut ::core::ffi::c_void,
        (core::ptr::addr_of_mut!((*stateblock).stateblock_state.ps_consts_b) as *mut BOOL)
            .offset(start_idx as isize) as *mut BOOL as *const ::core::ffi::c_void,
        (count as usize).wrapping_mul(::core::mem::size_of::<BOOL>() as usize),
    );
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_vertex_declaration(
    mut stateblock: *mut wined3d_stateblock,
    mut declaration: *mut wined3d_vertex_declaration,
) {
    let mut prev: *mut wined3d_vertex_declaration =
        (*stateblock).stateblock_state.vertex_declaration as *mut wined3d_vertex_declaration;
    WINE_TRACE!("stateblock %p, declaration %p.\n", stateblock, declaration);
    if !declaration.is_null() {
        wined3d_vertex_declaration_incref(declaration);
    }
    if !prev.is_null() {
        wined3d_vertex_declaration_decref(prev);
    }
    (*stateblock).stateblock_state.vertex_declaration =
        declaration as *mut wined3d_vertex_declaration;
    (*stateblock)
        .changed
        .set_vertexDecl(TRUE as uint32_t);
    (*stateblock)
        .changed
        .set_texture_matrices(TRUE as uint32_t);
    if !declaration.is_null() && !prev.is_null() {
        if (*stateblock).stateblock_state.vs.is_null() {
            if (*(*(*stateblock).device).adapter)
                .d3d_info
                .full_ffp_varyings()
                == 0
                || (*declaration).diffuse as ::core::ffi::c_int
                    != (*prev).diffuse as ::core::ffi::c_int
                || (*stateblock).stateblock_state.rs
                    [WINED3D_RS_COLORVERTEX as usize]
                    != 0
                    && (*declaration).specular as ::core::ffi::c_int
                        != (*prev).specular as ::core::ffi::c_int
                || (*declaration).normal as ::core::ffi::c_int
                    != (*prev).normal as ::core::ffi::c_int
                || (*declaration).point_size as ::core::ffi::c_int
                    != (*prev).point_size as ::core::ffi::c_int
            {
                (*stateblock)
                    .changed
                    .set_ffp_vs_settings(1 as uint32_t);
            }
        }
        if (*declaration).position_transformed as ::core::ffi::c_int
            != (*prev).position_transformed as ::core::ffi::c_int
        {
            (*stateblock)
                .changed
                .set_ffp_vs_settings(1 as uint32_t);
            (*stateblock)
                .changed
                .set_position_transformed(1 as uint32_t);
            (*stateblock)
                .changed
                .set_transforms(1 as uint32_t);
        }
    } else {
        (*stateblock)
            .changed
            .set_position_transformed(1 as uint32_t);
        (*stateblock)
            .changed
            .set_ffp_vs_settings(1 as uint32_t);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_render_state(
    mut stateblock: *mut wined3d_stateblock,
    mut state: wined3d_render_state,
    mut value: ::core::ffi::c_uint,
) {
    TRACE!(
        "stateblock %p, state %s (%#x), value %#x.\n",
        stateblock,
        debug_d3drenderstate(state),
        state as ::core::ffi::c_uint,
        value
    );

    if state as ::core::ffi::c_uint > (WINEHIGHEST_RENDER_STATE as u32) {
        WARN!("Unhandled render state %#x.\n", state as ::core::ffi::c_uint);
        return;
    }

    (*stateblock).stateblock_state.rs[state as usize] = value;
    let mut RsIdx: usize =
        ((state as ::core::ffi::c_uint) >> 5) as usize;
    (*stateblock).changed.renderState[RsIdx] = ((*stateblock).changed.renderState[RsIdx]
        as ::core::ffi::c_uint
        | (1 as ::core::ffi::c_uint)
            << ((state as ::core::ffi::c_uint) & 0x1f))
        as uint32_t;

    match state as ::core::ffi::c_uint {
        WINED3D_RS_POINTSIZE => {
            if value == WINED3D_ALPHA_TO_COVERAGE_ENABLE || value == WINED3D_ALPHA_TO_COVERAGE_DISABLE {
                (*stateblock)
                    .changed
                    .set_alpha_to_coverage(1 as uint32_t);
                (*stateblock).stateblock_state.alpha_to_coverage =
                    (value == WINED3D_ALPHA_TO_COVERAGE_ENABLE) as ::core::ffi::c_int as BOOL;
            }
            (*stateblock)
                .changed
                .set_point_scale(1 as uint32_t);
        }
        WINED3D_RS_TEXTUREFACTOR => {
            (*stateblock)
                .changed
                .set_ffp_ps_constants(1 as uint32_t);
        }
        WINED3D_RS_VERTEXBLEND => {
            (*stateblock)
                .changed
                .set_modelview_matrices(1 as uint32_t);
            (*stateblock)
                .changed
                .set_ffp_vs_settings(1 as uint32_t);
        }
        WINED3D_RS_POINTSCALEENABLE
        | WINED3D_RS_POINTSCALE_A
        | WINED3D_RS_POINTSCALE_B
        | WINED3D_RS_POINTSCALE_C => {
            (*stateblock)
                .changed
                .set_point_scale(1 as uint32_t);
        }
        WINED3D_RS_AMBIENTMATERIALSOURCE
        | WINED3D_RS_COLORVERTEX
        | WINED3D_RS_DIFFUSEMATERIALSOURCE
        | WINED3D_RS_EMISSIVEMATERIALSOURCE
        | WINED3D_RS_FOGENABLE
        | WINED3D_RS_FOGTABLEMODE
        | WINED3D_RS_FOGVERTEXMODE
        | WINED3D_RS_LIGHTING
        | WINED3D_RS_LOCALVIEWER
        | WINED3D_RS_NORMALIZENORMALS
        | WINED3D_RS_RANGEFOGENABLE
        | WINED3D_RS_SPECULARMATERIALSOURCE => {
            (*stateblock)
                .changed
                .set_ffp_vs_settings(1 as uint32_t);
        }
        WINED3D_RS_COLORKEYENABLE => {
            (*stateblock)
                .changed
                .set_ffp_ps_settings(1 as uint32_t);
        }
        WINED3D_RS_SPECULARENABLE => {
            (*stateblock)
                .changed
                .set_ffp_vs_settings(1 as uint32_t);
            (*stateblock)
                .changed
                .set_ffp_ps_constants(1 as uint32_t);
        }
        WINED3D_RS_FILLMODE
        | WINED3D_RS_CULLMODE
        | WINED3D_RS_SLOPESCALEDEPTHBIAS
        | WINED3D_RS_DEPTHBIAS
        | WINED3D_RS_SCISSORTESTENABLE
        | WINED3D_RS_ANTIALIASEDLINEENABLE => {
            (*stateblock)
                .changed
                .set_rasterizer_state(1 as uint32_t);
        }
        _ => {}
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_sampler_state(stateblock: *mut wined3d_stateblock, sampler_idx: UINT, state: wined3d_sampler_state, value: ::core::ffi::c_uint) {
    if sampler_idx as usize >= ARRAY_SIZE!((*stateblock).stateblock_state.sampler_states) as usize {
        WARN!("Invalid sampler %u.\n", sampler_idx);
        return;
    }

    (*stateblock).stateblock_state.sampler_states[sampler_idx as usize][state as usize] = value as uint32_t;
    (*stateblock).changed.samplerState[sampler_idx as usize] |= ((1 as u32) << (state as u32)) as u16;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_texture_stage_state(
    stateblock: *mut wined3d_stateblock,
    stage: UINT,
    state: wined3d_texture_stage_state,
    value: ::core::ffi::c_uint,
) {
    TRACE!(
        "stateblock %p, stage %u, state %s, value %#x.\n",
        stateblock,
        stage,
        debug_d3dtexturestate(state),
        value
    );

    if state as ::core::ffi::c_uint > WINED3D_HIGHEST_TEXTURE_STATE as ::core::ffi::c_uint {
        WARN!("Invalid state %#x passed.\n", state as ::core::ffi::c_uint);
        return;
    }

    if stage >= WINED3D_MAX_FFP_TEXTURES as UINT {
        WARN!(
            "Attempting to set stage %u which is higher than the max stage %u, ignoring.\n",
            stage,
            (WINED3D_MAX_FFP_TEXTURES as UINT - 1 as UINT)
        );
        return;
    }

    (*stateblock).stateblock_state.texture_states[stage as usize][state as usize] =
        value as uint32_t;
    (*stateblock).changed.textureState[stage as usize] = (((*stateblock).changed.textureState
        [stage as usize] as ::core::ffi::c_uint)
        | ((1 as ::core::ffi::c_uint) << (state as ::core::ffi::c_uint)))
        as uint32_t;

    let mut current_block_28: u64;
    match state as ::core::ffi::c_uint {
        WINED3D_TSS_CONSTANT => {
            (*stateblock)
                .changed
                .set_ffp_ps_constants(1 as uint32_t);
            current_block_28 = 9853141518545631134;
        }

        WINED3D_TSS_TEXCOORD_INDEX => {
            (*stateblock)
                .changed
                .set_ffp_vs_settings(1 as uint32_t);
            /* fall through */
            current_block_28 = 11080343429441928903;
        }
        WINED3D_TSS_TEXTURE_TRANSFORM_FLAGS => {
            current_block_28 = 11080343429441928903;
        }

        WINED3D_TSS_ALPHA_ARG0
        | WINED3D_TSS_ALPHA_ARG1
        | WINED3D_TSS_ALPHA_ARG2
        | WINED3D_TSS_ALPHA_OP
        | WINED3D_TSS_COLOR_ARG0
        | WINED3D_TSS_COLOR_ARG1
        | WINED3D_TSS_COLOR_ARG2
        | WINED3D_TSS_COLOR_OP
        | WINED3D_TSS_RESULT_ARG => {
            (*stateblock)
                .changed
                .set_ffp_ps_settings(1 as uint32_t);
            current_block_28 = 9853141518545631134;
        }

        WINED3D_TSS_BUMPENV_LOFFSET
        | WINED3D_TSS_BUMPENV_LSCALE
        | WINED3D_TSS_BUMPENV_MAT00
        | WINED3D_TSS_BUMPENV_MAT01
        | WINED3D_TSS_BUMPENV_MAT10
        | WINED3D_TSS_BUMPENV_MAT11 => {
            (*stateblock)
                .changed
                .set_bumpenv_constants(1 as uint32_t);
            current_block_28 = 9853141518545631134;
        }

        _ => {
            current_block_28 = 9853141518545631134;
        }
    }

    match current_block_28 {
        11080343429441928903 => {
            (*stateblock)
                .changed
                .set_texture_matrices(1 as uint32_t);
            (*stateblock)
                .changed
                .set_ffp_ps_settings(1 as uint32_t);
        }
        _ => {}
    };
}

unsafe extern "C" fn texture_binding_might_invalidate_fs_settings(
    stateblock: *const wined3d_stateblock,
    texture: *const wined3d_texture,
    prev: *const wined3d_texture,
    stage: ::core::ffi::c_uint,
) -> bool {
    let d3d_info: *const wined3d_d3d_info = &raw const (*(*(*stateblock).device).adapter).d3d_info;
    let mut old_format: *const wined3d_format = ::core::ptr::null::<wined3d_format>();
    let mut new_format: *const wined3d_format = ::core::ptr::null::<wined3d_format>();
    let mut old_usage: ::core::ffi::c_uint = 0;
    let mut new_usage: ::core::ffi::c_uint = 0;

    /* The source arguments for color and alpha ops have different meanings when
     * a NULL texture is bound. */
    if texture.is_null() {
        return !prev.is_null();
    }
    if prev.is_null() {
        return true;
    }

    old_usage = (*prev).resource.usage as ::core::ffi::c_uint;
    new_usage = (*texture).resource.usage as ::core::ffi::c_uint;
    if (*texture).resource.type_ as ::core::ffi::c_uint
        != (*prev).resource.type_ as ::core::ffi::c_uint
        || (old_usage & (WINED3DUSAGE_LEGACY_CUBEMAP as ::core::ffi::c_uint))
            != (new_usage & (WINED3DUSAGE_LEGACY_CUBEMAP as ::core::ffi::c_uint))
    {
        return true;
    }

    if stage == 0
        && (*stateblock).stateblock_state.rs[WINED3D_RS_COLORKEYENABLE as usize] != 0
        && ((*texture).color_key_flags & (WINED3D_CKEY_SRC_BLT as uint32_t)) != 0
    {
        return true;
    }

    old_format = (*prev).resource.format;
    new_format = (*texture).resource.format;

    if is_same_fixup((*old_format).color_fixup, (*new_format).color_fixup) != 0 {
        return false;
    }

    if can_use_texture_swizzle(d3d_info, new_format) != 0
        && can_use_texture_swizzle(d3d_info, old_format) != 0
    {
        return false;
    }

    return true;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_texture(
    stateblock: *mut wined3d_stateblock,
    stage: UINT,
    texture: *mut wined3d_texture,
) {
    let mut prev: *mut wined3d_texture = (*stateblock).stateblock_state.textures[stage as usize];
    WINE_TRACE!("stateblock %p, stage %u, texture %p.\n", stateblock, stage, texture);
    if stage as usize
        >= (::core::mem::size_of::<[*mut wined3d_texture; 20]>() as usize)
            .wrapping_div(::core::mem::size_of::<*mut wined3d_texture>() as usize)
    {
        WARN!("Ignoring invalid stage %u.\n", stage);
        return;
    }

    if !texture.is_null() {
        wined3d_texture_incref(texture);
    }
    if !prev.is_null() {
        wined3d_texture_decref(prev);
    }
    (*stateblock).stateblock_state.textures[stage as usize] = texture as *mut wined3d_texture;
    (*stateblock).changed.set_textures(
        (*stateblock).changed.textures()
            | ((1 as ::core::ffi::c_uint) << stage) as uint32_t,
    );

    if texture_binding_might_invalidate_fs_settings(
        stateblock,
        texture,
        prev,
        stage as ::core::ffi::c_uint,
    ) {
        (*stateblock)
            .changed
            .set_ffp_ps_settings(1 as uint32_t);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_transform(
    stateblock: *mut wined3d_stateblock,
    d3dts: wined3d_transform_state,
    matrix: *const wined3d_matrix,
) {
    (*stateblock).stateblock_state.transforms[d3dts as usize] = *matrix;
    (*stateblock).changed.transform[((d3dts as u32) >> (5 as u32)) as usize] =
        ((*stateblock).changed.transform[((d3dts as u32) >> (5 as u32)) as usize]
            as u32
            | ((1 as u32) << ((d3dts as u32) & (0x1f as u32))))
            as u32;
    (*stateblock)
        .changed
        .set_transforms((1) as u32 as u32);

    if (d3dts as u32) >= (WINED3D_TS_TEXTURE0 as u32)
        && (d3dts as u32) <= (WINED3D_TS_TEXTURE7 as u32)
    {
        (*stateblock)
            .changed
            .set_texture_matrices((1) as u32 as u32);
    } else if (d3dts as u32) == (WINED3D_TS_VIEW as u32)
        || (d3dts as u32) >= (WINED3D_TS_WORLD as u32)
    {
        (*stateblock)
            .changed
            .set_modelview_matrices((1) as u32 as u32);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_multiply_transform(
    mut stateblock: *mut wined3d_stateblock,
    mut d3dts: wined3d_transform_state,
    mut matrix: *const wined3d_matrix,
) {
    let mut mat: *mut wined3d_matrix = core::ptr::addr_of_mut!((*stateblock).stateblock_state.transforms[d3dts as usize]);
    WINE_TRACE!("stateblock %p, state %s, matrix %p.\n", stateblock, debug_d3dtstype(d3dts), matrix);
    WINE_TRACE!("%.8e %.8e %.8e %.8e\n", (*matrix)._11 as f64, (*matrix)._12 as f64, (*matrix)._13 as f64, (*matrix)._14 as f64);
    WINE_TRACE!("%.8e %.8e %.8e %.8e\n", (*matrix)._21 as f64, (*matrix)._22 as f64, (*matrix)._23 as f64, (*matrix)._24 as f64);
    WINE_TRACE!("%.8e %.8e %.8e %.8e\n", (*matrix)._31 as f64, (*matrix)._32 as f64, (*matrix)._33 as f64, (*matrix)._34 as f64);
    WINE_TRACE!("%.8e %.8e %.8e %.8e\n", (*matrix)._41 as f64, (*matrix)._42 as f64, (*matrix)._43 as f64, (*matrix)._44 as f64);
    multiply_matrix(mat, mat, matrix);
    (*stateblock).changed.transform[(d3dts as u32 >> 5) as usize] =
        ((*stateblock).changed.transform[(d3dts as u32 >> 5) as usize]
            | (1 as u32) << (d3dts as u32 & 0x1f))
            as uint32_t;
    (*stateblock).changed.set_transforms(1 as uint32_t);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_clip_plane(
    stateblock: *mut wined3d_stateblock,
    plane_idx: UINT,
    plane: *const wined3d_vec4,
) -> HRESULT {
    TRACE!("stateblock %p, plane_idx %u, plane %p.\n", stateblock, plane_idx, plane);
    if plane_idx >= (*(*(*stateblock).device).adapter).d3d_info.limits.max_clip_distances {
        TRACE!("Application has requested clipplane this device doesn't support.\n");
        return WINED3DERR_INVALIDCALL;
    }

    (*stateblock).stateblock_state.clip_planes[plane_idx as usize] = *plane;
    (*stateblock).changed.clipplane = (*stateblock).changed.clipplane
        | ((1 as uint32_t) << plane_idx);
    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_material(
    stateblock: *mut wined3d_stateblock,
    material: *const wined3d_material,
) {
    WINE_TRACE!("stateblock %p, material %p.\n", stateblock, material);
    (*stateblock).stateblock_state.material = *material;
    (*stateblock)
        .changed
        .set_material(TRUE as uint32_t);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_viewport(
    mut stateblock: *mut wined3d_stateblock,
    mut viewport: *const wined3d_viewport,
) {
    TRACE!("stateblock %p, viewport %p.\n", stateblock, viewport);
    (*stateblock).stateblock_state.viewport = *viewport;
    (*stateblock).changed.set_viewport((TRUE) as _);
    (*stateblock).changed.set_point_scale((TRUE) as _);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_scissor_rect(stateblock: *mut wined3d_stateblock, rect: *const RECT) {
    (*stateblock).stateblock_state.scissor_rect = *rect;
    (*stateblock).changed.set_scissorRect((TRUE) as _);
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_index_buffer(
    stateblock: *mut wined3d_stateblock,
    buffer: *mut wined3d_buffer,
    format_id: wined3d_format_id,
) {
    TRACE!(
        "stateblock %p, buffer %p, format %s.\n",
        stateblock,
        buffer,
        debug_d3dformat(format_id)
    );
    if !buffer.is_null() {
        wined3d_buffer_incref(buffer);
    }
    if !(*stateblock).stateblock_state.index_buffer.is_null() {
        wined3d_buffer_decref((*stateblock).stateblock_state.index_buffer);
    }
    (*stateblock).stateblock_state.index_buffer = buffer;
    (*stateblock).stateblock_state.index_format = format_id;
    (*stateblock)
        .changed
        .set_indices(TRUE as uint32_t);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_base_vertex_index(
    stateblock: *mut wined3d_stateblock,
    base_index: INT,
) {
    (*stateblock).stateblock_state.base_vertex_index = base_index;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_stream_source(
    stateblock: *mut wined3d_stateblock,
    stream_idx: UINT,
    buffer: *mut wined3d_buffer,
    offset: UINT,
    stride: UINT,
) -> HRESULT {
    let mut stream: *mut wined3d_stream_state = core::ptr::null_mut();

    if stream_idx >= WINED3D_MAX_STREAMS as UINT {
        WARN!("Stream index %u out of range.\n", stream_idx);
        return WINED3DERR_INVALIDCALL;
    }

    stream = (core::ptr::addr_of_mut!((*stateblock).stateblock_state.streams) as *mut wined3d_stream_state)
        .offset(stream_idx as isize) as *mut wined3d_stream_state;

    if !buffer.is_null() {
        wined3d_buffer_incref(buffer);
    }
    if !(*stream).buffer.is_null() {
        wined3d_buffer_decref((*stream).buffer as *mut wined3d_buffer);
    }
    (*stream).buffer = buffer as *mut wined3d_buffer;
    (*stream).stride = stride;
    (*stream).offset = offset;
    (*stateblock).changed.streamSource = ((*stateblock).changed.streamSource as u32
        | ((1 as u32) << stream_idx)) as u16;
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_stream_source_freq(
    mut stateblock: *mut wined3d_stateblock,
    mut stream_idx: UINT,
    mut divider: UINT,
) -> HRESULT {
    let mut stream: *mut wined3d_stream_state = core::ptr::null_mut::<wined3d_stream_state>();

    TRACE!(
        "stateblock %p, stream_idx %u, divider %#x.\n",
        stateblock,
        stream_idx,
        divider
    );

    if (divider & WINED3DSTREAMSOURCE_INSTANCEDATA) != 0 && (divider & WINED3DSTREAMSOURCE_INDEXEDDATA) != 0 {
        WARN!("INSTANCEDATA and INDEXEDDATA were set, returning D3DERR_INVALIDCALL.\n");
        return WINED3DERR_INVALIDCALL;
    }
    if (divider & WINED3DSTREAMSOURCE_INSTANCEDATA) != 0 && stream_idx == 0 {
        WARN!("INSTANCEDATA used on stream 0, returning D3DERR_INVALIDCALL.\n");
        return WINED3DERR_INVALIDCALL;
    }
    if divider == 0 {
        WARN!("Divider is 0, returning D3DERR_INVALIDCALL.\n");
        return WINED3DERR_INVALIDCALL;
    }

    stream = (core::ptr::addr_of_mut!((*stateblock).stateblock_state.streams) as *mut wined3d_stream_state)
        .offset(stream_idx as isize) as *mut wined3d_stream_state;
    (*stream).flags = (divider & (WINED3DSTREAMSOURCE_INSTANCEDATA | WINED3DSTREAMSOURCE_INDEXEDDATA)) as UINT;
    (*stream).frequency = divider & 0x7fffff;
    (*stateblock).changed.streamFreq = (((*stateblock).changed.streamFreq as UINT)
        | ((1 as UINT) << stream_idx)) as uint16_t;
    return WINED3D_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_light(
    mut stateblock: *mut wined3d_stateblock,
    mut light_idx: UINT,
    mut light: *const wined3d_light,
) -> HRESULT {
    let mut object: *mut wined3d_light_info = core::ptr::null_mut::<wined3d_light_info>();
    let mut hr: HRESULT = 0;

    TRACE!(
        "stateblock %p, light_idx %u, light %p.\n",
        stateblock,
        light_idx,
        light
    );

    /* Check the parameter range. Need for speed most wanted sets junk lights
     * which confuse the GL driver. */
    if light.is_null() {
        return WINED3DERR_INVALIDCALL;
    }

    match (*light).type_ as core::ffi::c_uint {
        WINED3D_LIGHT_POINT | WINED3D_LIGHT_SPOT | WINED3D_LIGHT_GLSPOT => {
            /* Incorrect attenuation values can cause the gl driver to crash.
             * Happens with Need for speed most wanted. */
            if (*light).attenuation0 < 0.0f32
                || (*light).attenuation1 < 0.0f32
                || (*light).attenuation2 < 0.0f32
            {
                WARN!("Attenuation is negative, returning WINED3DERR_INVALIDCALL.\n");
                return WINED3DERR_INVALIDCALL;
            }
        }

        WINED3D_LIGHT_DIRECTIONAL | WINED3D_LIGHT_PARALLELPOINT => {
            /* Ignores attenuation */
        }

        _ => {
            WARN!("Light type out of range, returning WINED3DERR_INVALIDCALL.\n");
            return WINED3DERR_INVALIDCALL;
        }
    }

    object = wined3d_light_state_get_light(
        (*stateblock).stateblock_state.light_state,
        light_idx as core::ffi::c_uint,
    );
    if object.is_null()
        || (*light).type_ as core::ffi::c_uint
            != (*object).OriginalParms.type_ as core::ffi::c_uint
    {
        (*stateblock)
            .changed
            .set_ffp_vs_settings(1 as uint32_t);
    }

    hr = wined3d_light_state_set_light(
        (*stateblock).stateblock_state.light_state as *mut wined3d_light_state,
        light_idx as core::ffi::c_uint,
        light,
        core::ptr::addr_of_mut!(object),
    );
    if hr >= 0 {
        set_light_changed(stateblock, object);
    }
    return hr;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_light_enable(
    stateblock: *mut wined3d_stateblock,
    light_idx: UINT,
    enable: BOOL,
) -> HRESULT {
    let light_state: *mut wined3d_light_state = (*stateblock).stateblock_state.light_state;
    let mut light_info: *mut wined3d_light_info;
    let mut hr: HRESULT;

    TRACE!(
        "stateblock %p, light_idx %u, enable %#x.\n",
        stateblock,
        light_idx,
        enable
    );

    light_info = wined3d_light_state_get_light(light_state, light_idx);
    if light_info.is_null() {
        hr = wined3d_light_state_set_light(
            light_state,
            light_idx,
            core::ptr::addr_of!(WINED3D_default_light),
            core::ptr::addr_of_mut!(light_info),
        );
        if hr < 0 {
            return hr;
        }
        set_light_changed(stateblock, light_info);
    }

    if wined3d_light_state_enable_light(
        light_state,
        core::ptr::addr_of_mut!((*(*(*stateblock).device).adapter).d3d_info),
        light_info,
        enable,
    ) {
        set_light_changed(stateblock, light_info);
    }

    (*stateblock)
        .changed
        .set_ffp_vs_settings(1 as uint32_t);
    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_get_state(stateblock: *const wined3d_stateblock) -> *const wined3d_stateblock_state {
    return core::ptr::addr_of!((*stateblock).stateblock_state);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_get_light(
    stateblock: *const wined3d_stateblock,
    light_idx: UINT,
    light: *mut wined3d_light,
    enabled: *mut BOOL,
) -> HRESULT {
    let mut light_info: *mut wined3d_light_info;

    light_info = wined3d_light_state_get_light(core::ptr::addr_of!((*stateblock).light_state), light_idx);
    if light_info.is_null() {
        return WINED3DERR_INVALIDCALL;
    }
    *light = (*light_info).OriginalParms;
    *enabled = if (*light_info).enabled != 0 { 128 } else { 0 };
    return WINED3D_OK;
}

unsafe extern "C" fn init_default_render_states(
    rs: *mut ::core::ffi::c_uint,
    d3d_info: *const wined3d_d3d_info,
) {
    // stateblock.c's function-local unions `lp` and `tmpfloat`.
    #[repr(C)]
    #[derive(Copy, Clone)]
    union C2RustUnnamed_48 {
        lp: wined3d_line_pattern,
        d: DWORD,
    }
    #[repr(C)]
    #[derive(Copy, Clone)]
    union C2RustUnnamed_47 {
        f: f32,
        d: DWORD,
    }
    let mut lp: C2RustUnnamed_48 = C2RustUnnamed_48 {
        lp: wined3d_line_pattern {
            repeat_factor: 0,
            line_pattern: 0,
        },
    };
    let mut tmpfloat: C2RustUnnamed_47 = C2RustUnnamed_47 { f: 0. };
    *rs.offset(WINED3D_RS_ZENABLE as ::core::ffi::c_int as isize) =
        WINED3D_ZB_TRUE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_FILLMODE as ::core::ffi::c_int as isize) =
        WINED3D_FILL_SOLID as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_SHADEMODE as ::core::ffi::c_int as isize) =
        WINED3D_SHADE_GOURAUD as ::core::ffi::c_uint;
    lp.lp.repeat_factor = 0 as WORD;
    lp.lp.line_pattern = 0 as WORD;
    *rs.offset(WINED3D_RS_LINEPATTERN as ::core::ffi::c_int as isize) = lp.d as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_ZWRITEENABLE as ::core::ffi::c_int as isize) =
        TRUE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_ALPHATESTENABLE as ::core::ffi::c_int as isize) =
        FALSE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_LASTPIXEL as ::core::ffi::c_int as isize) =
        TRUE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_SRCBLEND as ::core::ffi::c_int as isize) =
        WINED3D_BLEND_ONE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_DESTBLEND as ::core::ffi::c_int as isize) =
        WINED3D_BLEND_ZERO as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_CULLMODE as ::core::ffi::c_int as isize) =
        WINED3D_CULL_BACK as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_ZFUNC as ::core::ffi::c_int as isize) =
        WINED3D_CMP_LESSEQUAL as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_ALPHAFUNC as ::core::ffi::c_int as isize) =
        WINED3D_CMP_ALWAYS as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_ALPHAREF as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_DITHERENABLE as ::core::ffi::c_int as isize) =
        FALSE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_ALPHABLENDENABLE as ::core::ffi::c_int as isize) =
        FALSE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_FOGENABLE as ::core::ffi::c_int as isize) =
        FALSE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_SPECULARENABLE as ::core::ffi::c_int as isize) =
        FALSE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_ZVISIBLE as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_FOGCOLOR as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_FOGTABLEMODE as ::core::ffi::c_int as isize) =
        WINED3D_FOG_NONE as ::core::ffi::c_uint;
    tmpfloat.f = 0.0f32;
    *rs.offset(WINED3D_RS_FOGSTART as ::core::ffi::c_int as isize) =
        tmpfloat.d as ::core::ffi::c_uint;
    tmpfloat.f = 1.0f32;
    *rs.offset(WINED3D_RS_FOGEND as ::core::ffi::c_int as isize) =
        tmpfloat.d as ::core::ffi::c_uint;
    tmpfloat.f = 1.0f32;
    *rs.offset(WINED3D_RS_FOGDENSITY as ::core::ffi::c_int as isize) =
        tmpfloat.d as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_RANGEFOGENABLE as ::core::ffi::c_int as isize) =
        FALSE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_STENCILENABLE as ::core::ffi::c_int as isize) =
        FALSE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_STENCILFAIL as ::core::ffi::c_int as isize) =
        WINED3D_STENCIL_OP_KEEP as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_STENCILZFAIL as ::core::ffi::c_int as isize) =
        WINED3D_STENCIL_OP_KEEP as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_STENCILPASS as ::core::ffi::c_int as isize) =
        WINED3D_STENCIL_OP_KEEP as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_STENCILREF as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_STENCILMASK as ::core::ffi::c_int as isize) =
        0xffffffff as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_STENCILFUNC as ::core::ffi::c_int as isize) =
        WINED3D_CMP_ALWAYS as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_STENCILWRITEMASK as ::core::ffi::c_int as isize) =
        0xffffffff as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_TEXTUREFACTOR as ::core::ffi::c_int as isize) =
        0xffffffff as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_WRAP0 as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_WRAP1 as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_WRAP2 as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_WRAP3 as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_WRAP4 as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_WRAP5 as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_WRAP6 as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_WRAP7 as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_CLIPPING as ::core::ffi::c_int as isize) =
        TRUE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_LIGHTING as ::core::ffi::c_int as isize) =
        TRUE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_AMBIENT as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_FOGVERTEXMODE as ::core::ffi::c_int as isize) =
        WINED3D_FOG_NONE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_COLORVERTEX as ::core::ffi::c_int as isize) =
        TRUE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_LOCALVIEWER as ::core::ffi::c_int as isize) =
        TRUE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_NORMALIZENORMALS as ::core::ffi::c_int as isize) =
        FALSE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_DIFFUSEMATERIALSOURCE as ::core::ffi::c_int as isize) =
        WINED3D_MCS_COLOR1 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_SPECULARMATERIALSOURCE as ::core::ffi::c_int as isize) =
        WINED3D_MCS_COLOR2 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_AMBIENTMATERIALSOURCE as ::core::ffi::c_int as isize) =
        WINED3D_MCS_MATERIAL as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_EMISSIVEMATERIALSOURCE as ::core::ffi::c_int as isize) =
        WINED3D_MCS_MATERIAL as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_VERTEXBLEND as ::core::ffi::c_int as isize) =
        WINED3D_VBF_DISABLE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_CLIPPLANEENABLE as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_SOFTWAREVERTEXPROCESSING as ::core::ffi::c_int as isize) =
        FALSE as ::core::ffi::c_uint;
    tmpfloat.f = 1.0f32;
    *rs.offset(WINED3D_RS_POINTSIZE as ::core::ffi::c_int as isize) =
        tmpfloat.d as ::core::ffi::c_uint;
    tmpfloat.f = 1.0f32;
    *rs.offset(WINED3D_RS_POINTSIZE_MIN as ::core::ffi::c_int as isize) =
        tmpfloat.d as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_POINTSPRITEENABLE as ::core::ffi::c_int as isize) =
        FALSE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_POINTSCALEENABLE as ::core::ffi::c_int as isize) =
        FALSE as ::core::ffi::c_uint;
    tmpfloat.f = 1.0f32;
    *rs.offset(WINED3D_RS_POINTSCALE_A as ::core::ffi::c_int as isize) =
        tmpfloat.d as ::core::ffi::c_uint;
    tmpfloat.f = 0.0f32;
    *rs.offset(WINED3D_RS_POINTSCALE_B as ::core::ffi::c_int as isize) =
        tmpfloat.d as ::core::ffi::c_uint;
    tmpfloat.f = 0.0f32;
    *rs.offset(WINED3D_RS_POINTSCALE_C as ::core::ffi::c_int as isize) =
        tmpfloat.d as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_MULTISAMPLEANTIALIAS as ::core::ffi::c_int as isize) =
        TRUE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_MULTISAMPLEMASK as ::core::ffi::c_int as isize) =
        0xffffffff as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_PATCHEDGESTYLE as ::core::ffi::c_int as isize) =
        WINED3D_PATCH_EDGE_DISCRETE as ::core::ffi::c_uint;
    tmpfloat.f = 1.0f32;
    *rs.offset(WINED3D_RS_PATCHSEGMENTS as ::core::ffi::c_int as isize) =
        tmpfloat.d as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_DEBUGMONITORTOKEN as ::core::ffi::c_int as isize) =
        0xbaadcafe as ::core::ffi::c_uint;
    tmpfloat.f = (*d3d_info).limits.pointsize_max;
    *rs.offset(WINED3D_RS_POINTSIZE_MAX as ::core::ffi::c_int as isize) =
        tmpfloat.d as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_INDEXEDVERTEXBLENDENABLE as ::core::ffi::c_int as isize) =
        FALSE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_COLORWRITEENABLE as ::core::ffi::c_int as isize) =
        0x0000000f as ::core::ffi::c_uint;
    tmpfloat.f = 0.0f32;
    *rs.offset(WINED3D_RS_TWEENFACTOR as ::core::ffi::c_int as isize) =
        tmpfloat.d as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_BLENDOP as ::core::ffi::c_int as isize) =
        WINED3D_BLEND_OP_ADD as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_POSITIONDEGREE as ::core::ffi::c_int as isize) =
        WINED3D_DEGREE_CUBIC as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_NORMALDEGREE as ::core::ffi::c_int as isize) =
        WINED3D_DEGREE_LINEAR as ::core::ffi::c_uint;
    /* states new in d3d9 */
    *rs.offset(WINED3D_RS_SCISSORTESTENABLE as ::core::ffi::c_int as isize) =
        FALSE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_SLOPESCALEDEPTHBIAS as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    tmpfloat.f = 1.0f32;
    *rs.offset(WINED3D_RS_MINTESSELLATIONLEVEL as ::core::ffi::c_int as isize) =
        tmpfloat.d as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_MAXTESSELLATIONLEVEL as ::core::ffi::c_int as isize) =
        tmpfloat.d as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_ANTIALIASEDLINEENABLE as ::core::ffi::c_int as isize) =
        FALSE as ::core::ffi::c_uint;
    tmpfloat.f = 0.0f32;
    *rs.offset(WINED3D_RS_ADAPTIVETESS_X as ::core::ffi::c_int as isize) =
        tmpfloat.d as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_ADAPTIVETESS_Y as ::core::ffi::c_int as isize) =
        tmpfloat.d as ::core::ffi::c_uint;
    tmpfloat.f = 1.0f32;
    *rs.offset(WINED3D_RS_ADAPTIVETESS_Z as ::core::ffi::c_int as isize) =
        tmpfloat.d as ::core::ffi::c_uint;
    tmpfloat.f = 0.0f32;
    *rs.offset(WINED3D_RS_ADAPTIVETESS_W as ::core::ffi::c_int as isize) =
        tmpfloat.d as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_ENABLEADAPTIVETESSELLATION as ::core::ffi::c_int as isize) =
        FALSE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_TWOSIDEDSTENCILMODE as ::core::ffi::c_int as isize) =
        FALSE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_BACK_STENCILFAIL as ::core::ffi::c_int as isize) =
        WINED3D_STENCIL_OP_KEEP as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_BACK_STENCILZFAIL as ::core::ffi::c_int as isize) =
        WINED3D_STENCIL_OP_KEEP as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_BACK_STENCILPASS as ::core::ffi::c_int as isize) =
        WINED3D_STENCIL_OP_KEEP as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_BACK_STENCILFUNC as ::core::ffi::c_int as isize) =
        WINED3D_CMP_ALWAYS as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_COLORWRITEENABLE1 as ::core::ffi::c_int as isize) =
        0x0000000f as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_COLORWRITEENABLE2 as ::core::ffi::c_int as isize) =
        0x0000000f as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_COLORWRITEENABLE3 as ::core::ffi::c_int as isize) =
        0x0000000f as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_BLENDFACTOR as ::core::ffi::c_int as isize) =
        0xffffffff as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_SRGBWRITEENABLE as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_DEPTHBIAS as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_WRAP8 as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_WRAP9 as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_WRAP10 as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_WRAP11 as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_WRAP12 as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_WRAP13 as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_WRAP14 as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_WRAP15 as ::core::ffi::c_int as isize) =
        0 as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_SEPARATEALPHABLENDENABLE as ::core::ffi::c_int as isize) =
        FALSE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_SRCBLENDALPHA as ::core::ffi::c_int as isize) =
        WINED3D_BLEND_ONE as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_DESTBLENDALPHA as ::core::ffi::c_int as isize) =
        WINED3D_BLEND_ZERO as ::core::ffi::c_uint;
    *rs.offset(WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int as isize) =
        WINED3D_BLEND_OP_ADD as ::core::ffi::c_uint;
}

unsafe extern "C" fn init_default_texture_state(
    i: ::core::ffi::c_uint,
    stage: *mut uint32_t,
) {
    *stage.offset(WINED3D_TSS_COLOR_OP as ::core::ffi::c_int as isize) = (if i != 0 {
        WINED3D_TOP_DISABLE as ::core::ffi::c_int
    } else {
        WINED3D_TOP_MODULATE as ::core::ffi::c_int
    }) as uint32_t;
    *stage.offset(WINED3D_TSS_COLOR_ARG1 as ::core::ffi::c_int as isize) =
        WINED3DTA_TEXTURE as uint32_t;
    *stage.offset(WINED3D_TSS_COLOR_ARG2 as ::core::ffi::c_int as isize) =
        WINED3DTA_CURRENT as uint32_t;
    *stage.offset(WINED3D_TSS_ALPHA_OP as ::core::ffi::c_int as isize) = (if i != 0 {
        WINED3D_TOP_DISABLE as ::core::ffi::c_int
    } else {
        WINED3D_TOP_SELECT_ARG1 as ::core::ffi::c_int
    }) as uint32_t;
    *stage.offset(WINED3D_TSS_ALPHA_ARG1 as ::core::ffi::c_int as isize) =
        WINED3DTA_TEXTURE as uint32_t;
    *stage.offset(WINED3D_TSS_ALPHA_ARG2 as ::core::ffi::c_int as isize) =
        WINED3DTA_CURRENT as uint32_t;
    *stage.offset(WINED3D_TSS_BUMPENV_MAT00 as ::core::ffi::c_int as isize) =
        0 as uint32_t;
    *stage.offset(WINED3D_TSS_BUMPENV_MAT01 as ::core::ffi::c_int as isize) =
        0 as uint32_t;
    *stage.offset(WINED3D_TSS_BUMPENV_MAT10 as ::core::ffi::c_int as isize) =
        0 as uint32_t;
    *stage.offset(WINED3D_TSS_BUMPENV_MAT11 as ::core::ffi::c_int as isize) =
        0 as uint32_t;
    *stage.offset(WINED3D_TSS_TEXCOORD_INDEX as ::core::ffi::c_int as isize) = i as uint32_t;
    *stage.offset(WINED3D_TSS_BUMPENV_LSCALE as ::core::ffi::c_int as isize) =
        0 as uint32_t;
    *stage.offset(WINED3D_TSS_BUMPENV_LOFFSET as ::core::ffi::c_int as isize) =
        0 as uint32_t;
    *stage.offset(WINED3D_TSS_TEXTURE_TRANSFORM_FLAGS as ::core::ffi::c_int as isize) =
        WINED3D_TTFF_DISABLE as ::core::ffi::c_int as uint32_t;
    *stage.offset(WINED3D_TSS_COLOR_ARG0 as ::core::ffi::c_int as isize) =
        WINED3DTA_CURRENT as uint32_t;
    *stage.offset(WINED3D_TSS_ALPHA_ARG0 as ::core::ffi::c_int as isize) =
        WINED3DTA_CURRENT as uint32_t;
    *stage.offset(WINED3D_TSS_RESULT_ARG as ::core::ffi::c_int as isize) =
        WINED3DTA_CURRENT as uint32_t;
}

unsafe extern "C" fn init_default_sampler_states(states: *mut [u32; 14]) {
    let mut i: u32 = 0;
    while i < WINED3D_MAX_COMBINED_SAMPLERS {
        (*states.offset(i as isize))[WINED3D_SAMP_ADDRESS_U as usize] = WINED3D_TADDRESS_WRAP;
        (*states.offset(i as isize))[WINED3D_SAMP_ADDRESS_V as usize] = WINED3D_TADDRESS_WRAP;
        (*states.offset(i as isize))[WINED3D_SAMP_ADDRESS_W as usize] = WINED3D_TADDRESS_WRAP;
        (*states.offset(i as isize))[WINED3D_SAMP_BORDER_COLOR as usize] = 0;
        (*states.offset(i as isize))[WINED3D_SAMP_MAG_FILTER as usize] = WINED3D_TEXF_POINT;
        (*states.offset(i as isize))[WINED3D_SAMP_MIN_FILTER as usize] = WINED3D_TEXF_POINT;
        (*states.offset(i as isize))[WINED3D_SAMP_MIP_FILTER as usize] = WINED3D_TEXF_NONE;
        (*states.offset(i as isize))[WINED3D_SAMP_MIPMAP_LOD_BIAS as usize] = 0;
        (*states.offset(i as isize))[WINED3D_SAMP_MAX_MIP_LEVEL as usize] = 0;
        (*states.offset(i as isize))[WINED3D_SAMP_MAX_ANISOTROPY as usize] = 1;
        (*states.offset(i as isize))[WINED3D_SAMP_SRGB_TEXTURE as usize] = 0;
        /* TODO: Indicates which element of a multielement texture to use. */
        (*states.offset(i as isize))[WINED3D_SAMP_ELEMENT_INDEX as usize] = 0;
        /* TODO: Vertex offset in the presampled displacement map. */
        (*states.offset(i as isize))[WINED3D_SAMP_DMAP_OFFSET as usize] = 0;
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn state_init_default(
    mut state: *mut wined3d_state,
    mut d3d_info: *const wined3d_d3d_info,
) {
    let mut identity: wined3d_matrix = core::mem::zeroed();
    let mut i: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_uint = 0;
    WINE_TRACE!("state %p, d3d_info %p.\n", state, d3d_info);
    get_identity_matrix(&raw mut identity);
    (*state).primitive_type = WINED3D_PT_UNDEFINED;
    (*state).patch_vertex_count = 0;
    (*state).transforms[WINED3D_TS_PROJECTION as usize] = identity;
    (*state).transforms[WINED3D_TS_VIEW as usize] = identity;
    i = 0 as ::core::ffi::c_uint;
    while i < 256 as ::core::ffi::c_uint {
        (*state).transforms[WINED3D_TS_WORLD_MATRIX!(i) as usize] = identity;
        i = i.wrapping_add(1);
    }
    init_default_render_states(
        &raw mut (*state).render_states as *mut ::core::ffi::c_uint,
        d3d_info,
    );
    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_MAX_FFP_TEXTURES as ::core::ffi::c_uint {
        WINE_TRACE!("Setting up default texture states for texture Stage %u.\n", i);
        (*state).transforms[(WINED3D_TS_TEXTURE0 as ::core::ffi::c_uint).wrapping_add(i) as usize] =
            identity;
        init_default_texture_state(
            i,
            &raw mut (*state).texture_states[i as usize] as *mut uint32_t,
        );
        i = i.wrapping_add(1);
    }
    (*state).blend_factor.r = 1.0f32;
    (*state).blend_factor.g = 1.0f32;
    (*state).blend_factor.b = 1.0f32;
    (*state).blend_factor.a = 1.0f32;
    (*state).sample_mask = 0xffffffff as uint32_t;
    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_MAX_STREAMS as ::core::ffi::c_uint {
        (*state).streams[i as usize].frequency = 1 as UINT;
        i = i.wrapping_add(1);
    }
    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_SHADER_TYPE_COUNT as ::core::ffi::c_uint {
        j = 0 as ::core::ffi::c_uint;
        while j < MAX_CONSTANT_BUFFERS as ::core::ffi::c_uint {
            (*state).cb[i as usize][j as usize].size =
                (WINED3D_MAX_CONSTANT_BUFFER_SIZE * 16) as ::core::ffi::c_uint;
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn lights_compare(key: *const core::ffi::c_void, entry: *const rb_entry) -> i32 {
    let light: *const wined3d_light_info = (entry as *const u8)
        .sub(core::mem::offset_of!(wined3d_light_info, entry)) as *const wined3d_light_info;
    let original_index: u32 = key as usize as u32;

    return wined3d_uint32_compare((*light).OriginalIndex, original_index);
}
#[no_mangle]
pub unsafe extern "C" fn state_init(
    state: *mut wined3d_state,
    d3d_info: *const wined3d_d3d_info,
    flags: uint32_t,
    feature_level: wined3d_feature_level,
) {
    (*state).feature_level = feature_level;
    (*state).flags = flags;

    rb_init(
        &raw mut (*state).light_state.lights_tree,
        Some(
            lights_compare
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const rb_entry,
                ) -> ::core::ffi::c_int,
        ),
    );

    if (flags & (WINED3D_STATE_INIT_DEFAULT as uint32_t)) != 0 {
        state_init_default(state, d3d_info);
    }
}

unsafe extern "C" fn wined3d_select_feature_level(
    adapter: *const wined3d_adapter,
    levels: *const wined3d_feature_level,
    level_count: core::ffi::c_uint,
    selected_level: *mut wined3d_feature_level,
) -> bool {
    let d3d_info: *const wined3d_d3d_info = core::ptr::addr_of!((*adapter).d3d_info);
    let mut i: core::ffi::c_uint = 0;

    while i < level_count {
        if *levels.add(i as usize) as u32 != 0
            && (*d3d_info).feature_level as u32 >= *levels.add(i as usize) as u32
        {
            *selected_level = *levels.add(i as usize);
            return true;
        }
        i = i.wrapping_add(1);
    }

    WINE_FIXME_!(__wine_dbch_winediag, "None of the requested D3D feature levels is supported on this GPU with the current shader backend.\n");
    return false;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_state_create(
    mut device: *mut wined3d_device,
    mut levels: *const wined3d_feature_level,
    mut level_count: ::core::ffi::c_uint,
    mut state: *mut *mut wined3d_state,
) -> HRESULT {
    let mut feature_level: wined3d_feature_level = WINED3D_FEATURE_LEVEL_NONE;
    let mut object: *mut wined3d_state = ::core::ptr::null_mut::<wined3d_state>();
    WINE_TRACE!("device %p, levels %p, level_count %u, state %p.\n", device, levels, level_count, state);
    if !wined3d_select_feature_level(
        (*device).adapter,
        levels,
        level_count,
        &raw mut feature_level,
    ) {
        return E_FAIL;
    }
    WINE_TRACE!("Selected feature level %s.\n", wined3d_debug_feature_level(feature_level));
    object = calloc(
        (1 as size_t) as _,
        (::core::mem::size_of::<wined3d_state>() as size_t) as _,
    ) as *mut wined3d_state;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }
    state_init(
        object,
        &raw mut (*(*device).adapter).d3d_info,
        WINED3D_STATE_INIT_DEFAULT as uint32_t,
        feature_level,
    );
    *state = object;
    return S_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_state_get_feature_level(
    state: *const wined3d_state,
) -> wined3d_feature_level {
    // TRACE("state %p.\n", state);
    (*state).feature_level
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_state_destroy(state: *mut wined3d_state) {
    TRACE!("state %p.\n", state);
    state_cleanup(state);
    free(state as *mut core::ffi::c_void);
}

unsafe extern "C" fn stateblock_state_init_default(
    state: *mut wined3d_stateblock_state,
    d3d_info: *const wined3d_d3d_info,
) {
    let mut identity: wined3d_matrix = core::mem::zeroed();
    let mut i: u32;

    get_identity_matrix(core::ptr::addr_of_mut!(identity));

    (*state).transforms[WINED3D_TS_PROJECTION as usize] = identity;
    (*state).transforms[WINED3D_TS_VIEW as usize] = identity;
    i = 0 as u32;
    while i < 256 as u32 {
        (*state).transforms[WINED3D_TS_WORLD_MATRIX!(i) as usize] = identity;
        i = i.wrapping_add(1);
    }

    init_default_render_states(
        core::ptr::addr_of_mut!((*state).rs) as *mut u32,
        d3d_info,
    );

    i = 0 as u32;
    while i < WINED3D_MAX_FFP_TEXTURES as u32 {
        (*state).transforms[(WINED3D_TS_TEXTURE0 as u32).wrapping_add(i) as usize] = identity;
        init_default_texture_state(
            i,
            core::ptr::addr_of_mut!((*state).texture_states[i as usize]) as *mut u32,
        );
        i = i.wrapping_add(1);
    }

    init_default_sampler_states(core::ptr::addr_of_mut!((*state).sampler_states) as *mut _);

    i = 0 as u32;
    while i < WINED3D_MAX_STREAMS as u32 {
        (*state).streams[i as usize].frequency = 1;
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn wined3d_stateblock_state_init(state: *mut wined3d_stateblock_state, device: *const wined3d_device, flags: uint32_t) {
    rb_init(
        core::ptr::addr_of_mut!((*(*state).light_state).lights_tree),
        Some(
            lights_compare
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const rb_entry,
                ) -> ::core::ffi::c_int,
        ),
    );

    if (flags & (WINED3D_STATE_INIT_DEFAULT as uint32_t)) != 0 {
        stateblock_state_init_default(state, core::ptr::addr_of!((*(*device).adapter).d3d_info));
    }
}

unsafe extern "C" fn wined3d_stateblock_invalidate_initial_states(stateblock: *mut wined3d_stateblock) {
    (*stateblock).changed.set_ffp_ps_constants((1) as uint32_t);
    (*stateblock).changed.set_lights((1) as uint32_t);
    (*stateblock).changed.set_texture_matrices((1) as uint32_t);
    (*stateblock).changed.set_material((1) as uint32_t);
    (*stateblock).changed.set_transforms((1) as uint32_t);
    memset(
        core::ptr::addr_of_mut!((*stateblock).changed.transform) as *mut ::core::ffi::c_void,
        (0xff) as ::core::ffi::c_int,
        ::core::mem::size_of::<[uint32_t; 16]>() as size_t,
    );
    (*stateblock).changed.set_modelview_matrices((1) as uint32_t);
    (*stateblock).changed.set_point_scale((1) as uint32_t);
    (*stateblock).changed.set_ffp_vs_settings((1) as uint32_t);
    (*stateblock).changed.set_ffp_ps_settings((1) as uint32_t);
    (*stateblock).changed.set_bumpenv_constants((1) as uint32_t);
}

unsafe extern "C" fn stateblock_init(
    stateblock: *mut wined3d_stateblock,
    device_state: *const wined3d_stateblock,
    device: *mut wined3d_device,
    type_: wined3d_stateblock_type,
) -> HRESULT {
    let d3d_info: *const wined3d_d3d_info = core::ptr::addr_of!((*(*device).adapter).d3d_info);
    (*stateblock).ref_ = 1 as LONG;
    (*stateblock).device = device;
    (*stateblock).stateblock_state.light_state =
        core::ptr::addr_of_mut!((*stateblock).light_state) as *mut wined3d_light_state;
    wined3d_stateblock_state_init(
        core::ptr::addr_of_mut!((*stateblock).stateblock_state),
        device,
        (if (type_ as ::core::ffi::c_uint) == WINED3D_SBT_PRIMARY as ::core::ffi::c_uint {
            WINED3D_STATE_INIT_DEFAULT as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as uint32_t,
    );

    (*stateblock).changed.set_store_stream_offset(1 as uint32_t);
    list_init(core::ptr::addr_of_mut!((*stateblock).changed.changed_lights));

    if (type_ as ::core::ffi::c_uint) == WINED3D_SBT_PRIMARY as ::core::ffi::c_uint {
        wined3d_stateblock_invalidate_initial_states(stateblock);
    }

    if (type_ as ::core::ffi::c_uint) == WINED3D_SBT_RECORDED as ::core::ffi::c_uint
        || (type_ as ::core::ffi::c_uint) == WINED3D_SBT_PRIMARY as ::core::ffi::c_uint
    {
        return WINED3D_OK;
    }

    WINE_TRACE!("Updating changed flags appropriate for type %#x.\n", type_ as ::core::ffi::c_uint);

    match type_ as ::core::ffi::c_uint {
        WINED3D_SBT_ALL => {
            stateblock_init_lights(
                stateblock,
                core::ptr::addr_of!((*(*device_state).stateblock_state.light_state).lights_tree),
            );
            stateblock_savedstates_set_all(
                core::ptr::addr_of_mut!((*stateblock).changed),
                (*d3d_info).limits.vs_uniform_count,
                (*d3d_info).limits.ps_uniform_count,
            );
        }
        WINED3D_SBT_PIXEL_STATE => {
            stateblock_savedstates_set_pixel(
                core::ptr::addr_of_mut!((*stateblock).changed),
                (*d3d_info).limits.ps_uniform_count,
            );
        }
        WINED3D_SBT_VERTEX_STATE => {
            stateblock_init_lights(
                stateblock,
                core::ptr::addr_of!((*(*device_state).stateblock_state.light_state).lights_tree),
            );
            stateblock_savedstates_set_vertex(
                core::ptr::addr_of_mut!((*stateblock).changed),
                (*d3d_info).limits.vs_uniform_count,
            );
        }
        _ => {
            WINE_FIXME!("Unrecognized state block type %#x.\n", type_ as ::core::ffi::c_uint);
        }
    }

    wined3d_stateblock_init_contained_states(stateblock);
    wined3d_stateblock_capture(stateblock, device_state);

    (*stateblock).changed.set_store_stream_offset(0 as uint32_t);

    return WINED3D_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_create(
    device: *mut wined3d_device,
    device_state: *const wined3d_stateblock,
    type_: wined3d_stateblock_type,
    stateblock: *mut *mut wined3d_stateblock,
) -> HRESULT {
    let mut object: *mut wined3d_stateblock;
    let mut hr: HRESULT;

    object = calloc(
        1,
        (core::mem::size_of::<wined3d_stateblock>()) as _,
    ) as *mut wined3d_stateblock;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = stateblock_init(object, device_state, device, type_);
    if hr < 0 {
        WARN!("Failed to initialize stateblock, hr %#lx.\n", hr);
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    *stateblock = object;

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_reset(stateblock: *mut wined3d_stateblock) {
    wined3d_stateblock_state_cleanup(core::ptr::addr_of_mut!((*stateblock).stateblock_state));
    memset(
        core::ptr::addr_of_mut!((*stateblock).stateblock_state) as *mut core::ffi::c_void,
        0 as i32,
        core::mem::size_of::<wined3d_stateblock_state>() as usize,
    );
    (*stateblock).stateblock_state.light_state =
        core::ptr::addr_of_mut!((*stateblock).light_state) as *mut wined3d_light_state;
    wined3d_stateblock_state_init(
        core::ptr::addr_of_mut!((*stateblock).stateblock_state),
        (*stateblock).device,
        WINED3D_STATE_INIT_DEFAULT as u32,
    );
    wined3d_stateblock_invalidate_initial_states(stateblock);
}

unsafe extern "C" fn wined3d_device_set_base_vertex_index(
    mut device: *mut wined3d_device,
    mut base_index: ::core::ffi::c_int,
) {
    WINE_TRACE!("device %p, base_index %d.\n", device, base_index);
    (*(*(*device).cs).c.state).base_vertex_index = base_index;
}

unsafe extern "C" fn wined3d_device_set_vs_consts_b(
    device: *mut wined3d_device,
    start_idx: u32,
    count: u32,
    constants: *const BOOL,
) {
    let mut i: u32 = 0;

    if TRACE_ON!(__wine_dbch_d3d) != 0 {
        i = 0;
        while i < count {
            TRACE!("Set BOOL constant %u to %#x.\n", start_idx.wrapping_add(i), *constants.add(i as usize));
            i = i.wrapping_add(1);
        }
    }

    wined3d_device_context_push_constants(
        core::ptr::addr_of_mut!((*(*device).cs).c),
        WINED3D_PUSH_CONSTANTS_VS_B,
        WINED3D_SHADER_CONST_VS_B,
        start_idx,
        count,
        constants as *const core::ffi::c_void,
    );
}

unsafe extern "C" fn wined3d_device_set_vs_consts_i(
    device: *mut wined3d_device,
    start_idx: ::core::ffi::c_uint,
    count: ::core::ffi::c_uint,
    constants: *const wined3d_ivec4,
) {
    let mut i: ::core::ffi::c_uint = 0;

    if ((TRACE_ON!(__wine_dbch_d3d)) != 0) {
        i = 0;
        while i < count {
            TRACE!(
                "Set ivec4 constant %u to %s.\n",
                start_idx.wrapping_add(i),
                debug_ivec4(constants.offset(i as isize) as *const wined3d_ivec4)
            );
            i = i.wrapping_add(1);
        }
    }

    wined3d_device_context_push_constants(
        core::ptr::addr_of_mut!((*(*device).cs).c),
        WINED3D_PUSH_CONSTANTS_VS_I,
        WINED3D_SHADER_CONST_VS_I as u32,
        start_idx,
        count,
        constants as *const ::core::ffi::c_void,
    );
}

unsafe extern "C" fn wined3d_device_set_vs_consts_f(
    device: *mut wined3d_device,
    start_idx: u32,
    count: u32,
    constants: *const wined3d_vec4,
) {
    let mut i: u32;

    if ((TRACE_ON!(__wine_dbch_d3d)) != 0)
    {
        i = 0;
        while i < count {
            TRACE!("Set vec4 constant %u to %s.\n", start_idx.wrapping_add(i), debug_vec4(constants.add(i as usize)));
            i = i.wrapping_add(1);
        }
    }

    wined3d_device_context_push_constants(core::ptr::addr_of_mut!((*(*device).cs).c), WINED3D_PUSH_CONSTANTS_VS_F,
            WINED3D_SHADER_CONST_VS_F, start_idx, count, constants as *const core::ffi::c_void);
}

unsafe extern "C" fn wined3d_device_set_ps_consts_b(device: *mut wined3d_device, start_idx: ::core::ffi::c_uint, count: ::core::ffi::c_uint, constants: *const BOOL) {
    let mut i: ::core::ffi::c_uint;

    if TRACE_ON!(__wine_dbch_d3d) != 0 {
        i = 0;
        while i < count {
            TRACE!("Set BOOL constant %u to %#x.\n", start_idx.wrapping_add(i), *constants.offset(i as isize));
            i = i.wrapping_add(1);
        }
    }

    wined3d_device_context_push_constants(core::ptr::addr_of_mut!((*(*device).cs).c), WINED3D_PUSH_CONSTANTS_PS_B, WINED3D_SHADER_CONST_PS_B, start_idx, count, constants as *const core::ffi::c_void);
}

unsafe extern "C" fn wined3d_device_set_ps_consts_i(device: *mut wined3d_device, start_idx: u32, count: u32, constants: *const wined3d_ivec4) {
    let mut i: u32;

    if TRACE_ON!(__wine_dbch_d3d) != 0 {
        i = 0;
        while i < count {
            TRACE!("Set ivec4 constant %u to %s.\n", start_idx.wrapping_add(i), debug_ivec4(constants.add(i as usize)));
            i = i.wrapping_add(1);
        }
    }

    wined3d_device_context_push_constants(core::ptr::addr_of_mut!((*(*device).cs).c), WINED3D_PUSH_CONSTANTS_PS_I,
            WINED3D_SHADER_CONST_PS_I, start_idx, count, constants as *const core::ffi::c_void);
}

unsafe extern "C" fn wined3d_device_set_ps_consts_f(
    device: *mut wined3d_device,
    start_idx: u32,
    count: u32,
    constants: *const wined3d_vec4,
) {
    let mut i: u32;

    if ((TRACE_ON!(__wine_dbch_d3d)) != 0) {
        i = 0;
        while i < count {
            TRACE!(
                "Set vec4 constant %u to %s.\n",
                start_idx.wrapping_add(i),
                debug_vec4(constants.offset(i as isize) as *const wined3d_vec4)
            );
            i = i.wrapping_add(1);
        }
    }

    wined3d_device_context_push_constants(
        core::ptr::addr_of_mut!((*(*device).cs).c),
        WINED3D_PUSH_CONSTANTS_PS_F,
        WINED3D_SHADER_CONST_PS_F,
        start_idx,
        count,
        constants as *const core::ffi::c_void,
    );
}

unsafe extern "C" fn wined3d_device_context_set_light(context: *mut wined3d_device_context, light_idx: ::core::ffi::c_uint, light: *const wined3d_light) {
    let mut object: *mut wined3d_light_info = core::ptr::null_mut();

    if ((wined3d_light_state_set_light(core::ptr::addr_of_mut!((*(*context).state).light_state), light_idx, light, core::ptr::addr_of_mut!(object))) < 0) {
        return;
    }

    wined3d_device_context_emit_set_light(context, object);
}

unsafe extern "C" fn wined3d_device_set_light_enable(
    device: *mut wined3d_device,
    light_idx: core::ffi::c_uint,
    enable: bool,
) {
    let light_state: *mut wined3d_light_state =
        core::ptr::addr_of_mut!((*(*(*device).cs).c.state).light_state);
    let mut light_info: *mut wined3d_light_info = core::ptr::null_mut();

    /* Special case - enabling an undefined light creates one with a strict set of parameters. */
    light_info = wined3d_light_state_get_light(light_state, light_idx);
    if light_info.is_null() {
        /* Light enabled requested but light not defined, so defining one! */
        wined3d_device_context_set_light(
            core::ptr::addr_of_mut!((*(*device).cs).c),
            light_idx,
            core::ptr::addr_of!(WINED3D_default_light),
        );

        light_info = wined3d_light_state_get_light(light_state, light_idx);
        if light_info.is_null() {
            ERR!("Adding default lights has failed dismally.\n");
            return;
        }
    }

    if wined3d_light_state_enable_light(
        light_state,
        core::ptr::addr_of_mut!((*(*device).adapter).d3d_info),
        light_info,
        enable as BOOL,
    ) {
        wined3d_device_context_emit_set_light_enable(
            core::ptr::addr_of_mut!((*(*device).cs).c),
            light_idx,
            enable as BOOL,
        );
    }
}

unsafe extern "C" fn wined3d_device_set_clip_plane(
    device: *mut wined3d_device,
    plane_idx: u32,
    plane: *const wined3d_vec4,
) {
    let clip_planes: *mut wined3d_vec4 =
        core::ptr::addr_of_mut!((*(*(*device).cs).c.state).clip_planes) as *mut wined3d_vec4;

    if memcmp(
        clip_planes.offset(plane_idx as isize) as *const core::ffi::c_void,
        plane as *const core::ffi::c_void,
        core::mem::size_of::<wined3d_vec4>(),
    ) == 0
    {
        return;
    }

    *clip_planes.offset(plane_idx as isize) = *plane;

    wined3d_device_context_emit_set_clip_plane(core::ptr::addr_of_mut!((*(*device).cs).c), plane_idx, plane);
}

unsafe extern "C" fn resolve_depth_buffer(device: *mut wined3d_device) {
    let state: *const wined3d_state = (*(*device).cs).c.state;
    let mut src_view: *mut wined3d_rendertarget_view;
    let mut dst_resource: *mut wined3d_resource;
    let mut dst_texture: *mut wined3d_texture;

    dst_texture = wined3d_state_get_ffp_texture(state, 0);
    if dst_texture.is_null() {
        return;
    }
    dst_resource = core::ptr::addr_of_mut!((*dst_texture).resource);
    if (*(*dst_resource).format).depth_size == 0 {
        return;
    }
    src_view = (*state).fb.depth_stencil;
    if src_view.is_null() {
        return;
    }

    wined3d_device_context_resolve_sub_resource(
        core::ptr::addr_of_mut!((*(*device).cs).c),
        dst_resource,
        0,
        (*src_view).resource,
        (*src_view).sub_resource_idx,
        (*(*dst_resource).format).id,
    );
}

unsafe extern "C" fn wined3d_device_set_render_state(
    mut device: *mut wined3d_device,
    mut state: wined3d_render_state,
    mut value: ::core::ffi::c_uint,
) {
    if value == (*(*(*device).cs).c.state).render_states[state as usize] {
        WINE_TRACE!("Application is setting the old value over, nothing to do.\n");
    } else {
        (*(*(*device).cs).c.state).render_states[state as usize] = value;
        wined3d_device_context_emit_set_render_state(&raw mut (*(*device).cs).c, state, value);
    }
    if state as ::core::ffi::c_uint
        == WINED3D_RS_POINTSIZE as ::core::ffi::c_int as ::core::ffi::c_uint
        && value == WINED3D_RESZ_CODE as ::core::ffi::c_uint
    {
        WINE_TRACE!("RESZ multisampled depth buffer resolve triggered.\n");
        resolve_depth_buffer(device);
    }
}

unsafe extern "C" fn wined3d_device_set_texture_stage_state(
    device: *mut wined3d_device,
    stage: ::core::ffi::c_uint,
    state: wined3d_texture_stage_state,
    value: uint32_t,
) {
    if value == (*(*(*device).cs).c.state).texture_states[stage as usize][state as usize] {
        return;
    }

    (*(*(*device).cs).c.state).texture_states[stage as usize][state as usize] = value;

    wined3d_device_context_emit_set_texture_state(
        core::ptr::addr_of_mut!((*(*device).cs).c),
        stage,
        state,
        value,
    );
}

unsafe extern "C" fn wined3d_device_set_texture(
    mut device: *mut wined3d_device,
    mut stage: ::core::ffi::c_uint,
    mut texture: *mut wined3d_texture,
) {
    let mut shader_type: wined3d_shader_type = WINED3D_SHADER_TYPE_PIXEL;
    let mut srv: *mut wined3d_shader_resource_view = ::core::ptr::null_mut::<wined3d_shader_resource_view>();
    let mut prev: *mut wined3d_shader_resource_view = ::core::ptr::null_mut::<wined3d_shader_resource_view>();
    let mut state: *mut wined3d_state = (*(*device).cs).c.state;
    WINE_TRACE!("device %p, stage %u, texture %p.\n", device, stage, texture);
    if stage >= WINED3D_VERTEX_SAMPLER_OFFSET as ::core::ffi::c_uint {
        shader_type = WINED3D_SHADER_TYPE_VERTEX;
        stage = stage.wrapping_sub(WINED3D_VERTEX_SAMPLER_OFFSET as ::core::ffi::c_uint);
    }
    if !texture.is_null() && {
        srv = wined3d_texture_acquire_identity_srv(texture);
        srv.is_null()
    } {
        return;
    }
    prev = (*state).shader_resource_view[shader_type as usize][stage as usize];
    WINE_TRACE!("Previous texture %p.\n", prev);
    if srv == prev {
        WINE_TRACE!("App is setting the same texture again, nothing to do.\n");
        return;
    }
    (*state).shader_resource_view[shader_type as usize][stage as usize] = srv;
    if !srv.is_null() {
        wined3d_shader_resource_view_incref(srv);
    }
    wined3d_device_context_emit_set_texture(
        core::ptr::addr_of_mut!((*(*device).cs).c),
        shader_type,
        stage,
        srv,
    );
    if !prev.is_null() {
        wined3d_shader_resource_view_decref(prev);
    }
}

unsafe extern "C" fn wined3d_device_set_transform(
    device: *mut wined3d_device,
    state: wined3d_transform_state,
    matrix: *const wined3d_matrix,
) {
    if memcmp(
        core::ptr::addr_of_mut!((*(*(*device).cs).c.state).transforms[state as usize]) as *const core::ffi::c_void,
        matrix as *const core::ffi::c_void,
        core::mem::size_of::<wined3d_matrix>(),
    ) == 0
    {
        return;
    }

    (*(*(*device).cs).c.state).transforms[state as usize] = *matrix;
    wined3d_device_context_emit_set_transform(core::ptr::addr_of_mut!((*(*device).cs).c), state, matrix);
}

unsafe extern "C" fn get_texture_address_mode(
    texture: *const wined3d_texture,
    t: wined3d_texture_address,
) -> wined3d_texture_address {
    if (t as u32) < WINED3D_TADDRESS_WRAP as u32 || (t as u32) > WINED3D_TADDRESS_MIRROR_ONCE as u32 {
        WINE_FIXME!("Unrecognized or unsupported texture address mode %#x.\n", t as u32);
        return WINED3D_TADDRESS_WRAP;
    }

    /* Cubemaps are always set to clamp, regardless of the sampler state. */
    if (*texture).resource.usage & WINED3DUSAGE_LEGACY_CUBEMAP as u32 != 0
        || (*texture).flags & WINED3D_TEXTURE_COND_NP2 as u32 != 0
            && t as u32 == WINED3D_TADDRESS_WRAP as u32
    {
        return WINED3D_TADDRESS_CLAMP;
    }

    return t;
}

unsafe extern "C" fn sampler_desc_from_sampler_states(
    desc: *mut wined3d_sampler_desc,
    sampler_states: *const uint32_t,
    texture: *const wined3d_texture,
) {
    let d3d_info: *const wined3d_d3d_info =
        core::ptr::addr_of!((*(*(*texture).resource.device).adapter).d3d_info);

    (*desc).address_u = get_texture_address_mode(
        texture,
        *sampler_states.add(WINED3D_SAMP_ADDRESS_U as usize) as wined3d_texture_address,
    );
    (*desc).address_v = get_texture_address_mode(
        texture,
        *sampler_states.add(WINED3D_SAMP_ADDRESS_V as usize) as wined3d_texture_address,
    );
    (*desc).address_w = get_texture_address_mode(
        texture,
        *sampler_states.add(WINED3D_SAMP_ADDRESS_W as usize) as wined3d_texture_address,
    );
    wined3d_color_from_d3dcolor(
        core::ptr::addr_of_mut!((*desc).border_color) as *mut wined3d_color,
        *sampler_states.add(WINED3D_SAMP_BORDER_COLOR as usize) as DWORD,
    );
    if *sampler_states.add(WINED3D_SAMP_MAG_FILTER as usize) > WINED3D_TEXF_ANISOTROPIC as uint32_t {
        FIXME!(
            "Unrecognized or unsupported WINED3D_SAMP_MAG_FILTER %#x.\n",
            *sampler_states.add(WINED3D_SAMP_MAG_FILTER as usize)
        );
    }
    (*desc).mag_filter = min!(
        max!(
            *sampler_states.add(WINED3D_SAMP_MAG_FILTER as usize),
            WINED3D_TEXF_POINT as uint32_t,
        ),
        WINED3D_TEXF_LINEAR as uint32_t,
    ) as wined3d_texture_filter_type;
    if *sampler_states.add(WINED3D_SAMP_MIN_FILTER as usize) > WINED3D_TEXF_ANISOTROPIC as uint32_t {
        FIXME!(
            "Unrecognized or unsupported WINED3D_SAMP_MIN_FILTER %#x.\n",
            *sampler_states.add(WINED3D_SAMP_MIN_FILTER as usize)
        );
    }
    (*desc).min_filter = min!(
        max!(
            *sampler_states.add(WINED3D_SAMP_MIN_FILTER as usize),
            WINED3D_TEXF_POINT as uint32_t,
        ),
        WINED3D_TEXF_LINEAR as uint32_t,
    ) as wined3d_texture_filter_type;
    if *sampler_states.add(WINED3D_SAMP_MIP_FILTER as usize) > WINED3D_TEXF_ANISOTROPIC as uint32_t {
        FIXME!(
            "Unrecognized or unsupported WINED3D_SAMP_MIP_FILTER %#x.\n",
            *sampler_states.add(WINED3D_SAMP_MIP_FILTER as usize)
        );
    }
    (*desc).mip_filter = min!(
        max!(
            *sampler_states.add(WINED3D_SAMP_MIP_FILTER as usize),
            WINED3D_TEXF_NONE as uint32_t,
        ),
        WINED3D_TEXF_LINEAR as uint32_t,
    ) as wined3d_texture_filter_type;
    (*desc).lod_bias = int_to_float(
        *sampler_states.add(WINED3D_SAMP_MIPMAP_LOD_BIAS as usize),
    );
    (*desc).min_lod = -1000.0f32;
    (*desc).max_lod = 1000.0f32;

    /* The LOD is already clamped to texture->level_count in wined3d_stateblock_set_texture_lod(). */
    if (*texture).flags & (WINED3D_TEXTURE_COND_NP2 as uint32_t) != 0 {
        (*desc).mip_base_level = 0 as core::ffi::c_uint;
    } else if (*desc).mip_filter as core::ffi::c_uint
        == WINED3D_TEXF_NONE as core::ffi::c_uint
    {
        (*desc).mip_base_level = (*texture).lod;
    } else {
        (*desc).mip_base_level = min!(
            max!(
                *sampler_states.add(WINED3D_SAMP_MAX_MIP_LEVEL as usize),
                (*texture).lod as uint32_t,
            ),
            ((*texture).level_count as uint32_t).wrapping_sub(1 as uint32_t),
        ) as core::ffi::c_uint;
    }

    (*desc).max_anisotropy = *sampler_states.add(WINED3D_SAMP_MAX_ANISOTROPY as usize) as core::ffi::c_uint;
    if (*sampler_states.add(WINED3D_SAMP_MAG_FILTER as usize) != WINED3D_TEXF_ANISOTROPIC as uint32_t
        && *sampler_states.add(WINED3D_SAMP_MIN_FILTER as usize) != WINED3D_TEXF_ANISOTROPIC as uint32_t
        && *sampler_states.add(WINED3D_SAMP_MIP_FILTER as usize) != WINED3D_TEXF_ANISOTROPIC as uint32_t)
        || (*texture).flags & (WINED3D_TEXTURE_COND_NP2 as uint32_t) != 0
    {
        (*desc).max_anisotropy = 1 as core::ffi::c_uint;
    }
    (*desc).compare = ((*texture).resource.format_caps
        & (WINED3D_FORMAT_CAP_SHADOW as core::ffi::c_uint)) as BOOL;
    (*desc).comparison_func = WINED3D_CMP_LESSEQUAL as _;

    /* Only use the LSB of the WINED3D_SAMP_SRGB_TEXTURE value. This matches
     * the behaviour of the AMD Windows driver.
     *
     * Might & Magic: Heroes VI - Shades of Darkness sets
     * WINED3D_SAMP_SRGB_TEXTURE to a large value that looks like a
     * pointer—presumably by accident—and expects sRGB decoding to be
     * disabled. */
    (*desc).srgb_decode = (*sampler_states.add(WINED3D_SAMP_SRGB_TEXTURE as usize)
        & (0x1 as uint32_t)) as BOOL;

    if (*texture).resource.format_caps & (WINED3D_FORMAT_CAP_FILTERING as core::ffi::c_uint) == 0 {
        (*desc).mag_filter = WINED3D_TEXF_POINT as wined3d_texture_filter_type;
        (*desc).min_filter = WINED3D_TEXF_POINT as wined3d_texture_filter_type;
        (*desc).mip_filter = WINED3D_TEXF_NONE as wined3d_texture_filter_type;
    }

    if (*texture).flags & (WINED3D_TEXTURE_COND_NP2 as uint32_t) != 0 {
        (*desc).mip_filter = WINED3D_TEXF_NONE as wined3d_texture_filter_type;
        if (*d3d_info).unconditional_npot() == 0 {
            (*desc).min_filter = WINED3D_TEXF_POINT as wined3d_texture_filter_type;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_apply_clear_state(stateblock: *mut wined3d_stateblock, device: *mut wined3d_device) {
    let state: *const wined3d_stateblock_state = &raw const (*stateblock).stateblock_state;
    let context: *mut wined3d_device_context = &raw mut (*(*device).cs).c;

    /* Clear state depends on the viewport, scissor rect, and scissor enable. */

    if (*stateblock).changed.viewport() != 0 {
        wined3d_device_context_set_viewports(context, 1 as ::core::ffi::c_uint, &raw const (*state).viewport);
    }
    if (*stateblock).changed.scissorRect() != 0 {
        wined3d_device_context_set_scissor_rects(context, 1 as ::core::ffi::c_uint, &raw const (*state).scissor_rect);
    }

    if (*stateblock).changed.rasterizer_state() != 0 {
        let mut rasterizer_state: *mut wined3d_rasterizer_state = core::ptr::null_mut();
        let mut desc: wined3d_rasterizer_state_desc = core::mem::zeroed();
        let mut entry: *mut wine_rb_entry = core::ptr::null_mut();

        memset(
            &raw mut desc as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            core::mem::size_of::<wined3d_rasterizer_state_desc>() as size_t,
        );
        desc.fill_mode = (*state).rs[WINED3D_RS_FILLMODE as usize] as wined3d_fill_mode;
        desc.cull_mode = (*state).rs[WINED3D_RS_CULLMODE as usize] as wined3d_cull;
        desc.depth_bias = int_to_float((*state).rs[WINED3D_RS_DEPTHBIAS as usize]);
        desc.scale_bias = int_to_float((*state).rs[WINED3D_RS_SLOPESCALEDEPTHBIAS as usize]);
        desc.depth_clip = TRUE as BOOL;
        desc.scissor = (*state).rs[WINED3D_RS_SCISSORTESTENABLE as usize] as BOOL;
        desc.line_antialias = (*state).rs[WINED3D_RS_ANTIALIASEDLINEENABLE as usize] as BOOL;

        entry = wine_rb_get(
            &raw mut (*device).rasterizer_states,
            &raw mut desc as *const ::core::ffi::c_void,
        );
        if !entry.is_null() {
            rasterizer_state = WINE_RB_ENTRY_VALUE!(entry, wined3d_rasterizer_state, entry);
            wined3d_device_context_set_rasterizer_state(context, rasterizer_state);
        } else if wined3d_rasterizer_state_create(
            device,
            &raw mut desc,
            core::ptr::null_mut::<::core::ffi::c_void>(),
            &raw const wined3d_null_parent_ops,
            &raw mut rasterizer_state,
        ) >= 0 as ::core::ffi::c_int
        {
            wined3d_device_context_set_rasterizer_state(context, rasterizer_state);
            if wine_rb_put(
                &raw mut (*device).rasterizer_states,
                &raw mut desc as *const ::core::ffi::c_void,
                &raw mut (*rasterizer_state).entry,
            ) == -1 as ::core::ffi::c_int
            {
                ERR!("Failed to insert rasterizer state.\n");
                wined3d_rasterizer_state_decref(rasterizer_state);
            }
        }
    }

    if wined3d_bitmap_is_set(
        &raw mut (*stateblock).changed.renderState as *mut uint32_t,
        WINED3D_RS_SRGBWRITEENABLE as ::core::ffi::c_uint,
    ) != 0
    {
        wined3d_device_set_render_state(
            device,
            WINED3D_RS_SRGBWRITEENABLE,
            (*state).rs[WINED3D_RS_SRGBWRITEENABLE as usize],
        );
    }
}

unsafe extern "C" fn get_ffp_vertex_shader(device: *mut wined3d_device, state: *const wined3d_state) -> *mut wined3d_shader {
    let mut dummy_stream_info: wined3d_stream_info = core::mem::zeroed();
    let mut settings: wined3d_ffp_vs_settings = core::mem::zeroed();
    let mut entry: *const wine_rb_entry;
    let mut vs: *mut wined3d_ffp_vs;

    wined3d_ffp_get_vs_settings(
        state,
        &raw mut dummy_stream_info,
        &raw mut (*(*device).adapter).d3d_info,
        &raw mut settings,
    );

    entry = wine_rb_get(
        &raw mut (*device).ffp_vertex_shaders,
        &raw mut settings as *const core::ffi::c_void,
    );
    if !entry.is_null() {
        return (*WINE_RB_ENTRY_VALUE!(entry, wined3d_ffp_vs, entry.entry)).shader;
    }

    vs = malloc(core::mem::size_of::<wined3d_ffp_vs>() as usize) as *mut wined3d_ffp_vs;
    if vs.is_null() {
        return core::ptr::null_mut();
    }

    (*vs).entry.settings = settings;
    if wined3d_shader_create_ffp_vs(device, &raw mut settings, &raw mut (*vs).shader) < 0 {
        free(vs as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    if wine_rb_put(
        &raw mut (*device).ffp_vertex_shaders,
        &raw mut (*vs).entry.settings as *const core::ffi::c_void,
        &raw mut (*vs).entry.entry,
    ) == -1
    {
        ERR!("Failed to insert FFP vertex shader.\n");
    }

    return (*vs).shader;
}

unsafe extern "C" fn get_ffp_pixel_shader(mut device: *mut wined3d_device, state: *const wined3d_state) -> *mut wined3d_shader {
    let mut settings: ffp_frag_settings = core::mem::zeroed();
    let mut desc: *const ffp_frag_desc = core::ptr::null();
    let mut ps: *mut wined3d_ffp_ps = core::ptr::null_mut();

    wined3d_ffp_get_fs_settings(state, core::ptr::addr_of_mut!((*(*device).adapter).d3d_info), core::ptr::addr_of_mut!(settings));

    desc = find_ffp_frag_shader(core::ptr::addr_of_mut!((*device).ffp_pixel_shaders), core::ptr::addr_of_mut!(settings));
    if !desc.is_null() {
        return (*CONTAINING_RECORD!(desc, wined3d_ffp_ps, entry)).shader;
    }

    ps = malloc(core::mem::size_of::<wined3d_ffp_ps>()) as *mut wined3d_ffp_ps;
    if ps.is_null() {
        return core::ptr::null_mut();
    }

    (*ps).entry.settings = settings;
    if (wined3d_shader_create_ffp_ps(device, core::ptr::addr_of_mut!(settings), core::ptr::addr_of_mut!((*ps).shader))) < 0 {
        free(ps as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }
    add_ffp_frag_shader(core::ptr::addr_of_mut!((*device).ffp_pixel_shaders), core::ptr::addr_of_mut!((*ps).entry));

    (*ps).shader
}

unsafe extern "C" fn bind_push_constant_buffer(
    device: *mut wined3d_device,
    type_: wined3d_push_constants,
    shader_type: wined3d_shader_type,
    shader_binding: u32,
) {
    let mut state: wined3d_constant_buffer_state = wined3d_constant_buffer_state {
        buffer: core::ptr::null_mut(),
        offset: 0,
        size: 0,
    };

    if (*(*device).adapter).d3d_info.gpu_push_constants() == 0
        || (*device).push_constants[type_ as usize].is_null()
    {
        return;
    }

    state.buffer = (*device).push_constants[type_ as usize];
    state.offset = 0;
    state.size = (*(*device).push_constants[type_ as usize]).resource.size as u32;
    wined3d_device_context_set_constant_buffers(
        core::ptr::addr_of_mut!((*(*device).cs).c),
        shader_type,
        shader_binding,
        1,
        core::ptr::addr_of_mut!(state),
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_apply_stateblock(
    device: *mut wined3d_device,
    stateblock: *mut wined3d_stateblock,
) {
    let mut set_blend_state: bool = false;
    let mut set_depth_stencil_state: bool = false;

    let state: *const wined3d_stateblock_state = core::ptr::addr_of!((*stateblock).stateblock_state);
    let word_bit_count: ::core::ffi::c_uint = (::core::mem::size_of::<DWORD>() as ::core::ffi::c_uint) * CHAR_BIT;
    let changed: *mut wined3d_saved_states = core::ptr::addr_of_mut!((*stateblock).changed);
    let context: *mut wined3d_device_context = core::ptr::addr_of_mut!((*(*device).cs).c);
    let mut i: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_uint = 0;
    let mut start: ::core::ffi::c_uint = 0;
    let mut idx: ::core::ffi::c_uint = 0;
    let mut set_depth_bounds: bool = false;
    let mut range: wined3d_range = wined3d_range { offset: 0, size: 0 };
    let mut map: uint32_t = 0;

    wined3d_stateblock_apply_clear_state(stateblock, device);

    if (*changed).vertexShader() != 0 {
        /* Clip planes are affected by the view matrix, but only if not using
         * vertex shaders. */
        (*changed).clipplane = wined3d_mask_from_size(WINED3D_MAX_CLIP_DISTANCES);
    }

    start = 0;
    loop {
        if wined3d_bitmap_get_range(core::ptr::addr_of_mut!((*changed).vs_consts_f) as *mut uint32_t, WINED3D_MAX_VS_CONSTS_F, start, core::ptr::addr_of_mut!(range)) == 0 {
            break;
        }

        wined3d_device_set_vs_consts_f(device, range.offset, range.size, (core::ptr::addr_of!((*state).vs_consts_f) as *const wined3d_vec4).offset(range.offset as isize));
        start = range.offset.wrapping_add(range.size);
    }

    map = (*changed).vertexShaderConstantsI as uint32_t;
    start = 0;
    loop {
        if wined3d_bitmap_get_range(core::ptr::addr_of_mut!(map), WINED3D_MAX_CONSTS_I, start, core::ptr::addr_of_mut!(range)) == 0 {
            break;
        }

        wined3d_device_set_vs_consts_i(device, range.offset, range.size, (core::ptr::addr_of!((*state).vs_consts_i) as *const wined3d_ivec4).offset(range.offset as isize));
        start = range.offset.wrapping_add(range.size);
    }

    map = (*changed).vertexShaderConstantsB as uint32_t;
    start = 0;
    loop {
        if wined3d_bitmap_get_range(core::ptr::addr_of_mut!(map), WINED3D_MAX_CONSTS_B, start, core::ptr::addr_of_mut!(range)) == 0 {
            break;
        }

        wined3d_device_set_vs_consts_b(device, range.offset, range.size, (core::ptr::addr_of!((*state).vs_consts_b) as *const BOOL).offset(range.offset as isize));
        start = range.offset.wrapping_add(range.size);
    }

    start = 0;
    loop {
        if wined3d_bitmap_get_range(core::ptr::addr_of_mut!((*changed).ps_consts_f) as *mut uint32_t, WINED3D_MAX_PS_CONSTS_F, start, core::ptr::addr_of_mut!(range)) == 0 {
            break;
        }

        wined3d_device_set_ps_consts_f(device, range.offset, range.size, (core::ptr::addr_of!((*state).ps_consts_f) as *const wined3d_vec4).offset(range.offset as isize));
        start = range.offset.wrapping_add(range.size);
    }

    map = (*changed).pixelShaderConstantsI as uint32_t;
    start = 0;
    loop {
        if wined3d_bitmap_get_range(core::ptr::addr_of_mut!(map), WINED3D_MAX_CONSTS_I, start, core::ptr::addr_of_mut!(range)) == 0 {
            break;
        }

        wined3d_device_set_ps_consts_i(device, range.offset, range.size, (core::ptr::addr_of!((*state).ps_consts_i) as *const wined3d_ivec4).offset(range.offset as isize));
        start = range.offset.wrapping_add(range.size);
    }

    map = (*changed).pixelShaderConstantsB as uint32_t;
    start = 0;
    loop {
        if wined3d_bitmap_get_range(core::ptr::addr_of_mut!(map), WINED3D_MAX_CONSTS_B, start, core::ptr::addr_of_mut!(range)) == 0 {
            break;
        }

        wined3d_device_set_ps_consts_b(device, range.offset, range.size, (core::ptr::addr_of!((*state).ps_consts_b) as *const BOOL).offset(range.offset as isize));
        start = range.offset.wrapping_add(range.size);
    }

    i = 0;
    while (i as usize) < ARRAY_SIZE!((*changed).renderState) as usize {
        map = (*changed).renderState[i as usize];
        while map != 0 {
            j = wined3d_bit_scan(core::ptr::addr_of_mut!(map)) as ::core::ffi::c_uint;
            idx = i * word_bit_count + j;

            match idx {
                WINED3D_RS_BLENDFACTOR
                | WINED3D_RS_MULTISAMPLEMASK
                | WINED3D_RS_ALPHABLENDENABLE
                | WINED3D_RS_SRCBLEND
                | WINED3D_RS_DESTBLEND
                | WINED3D_RS_BLENDOP
                | WINED3D_RS_SEPARATEALPHABLENDENABLE
                | WINED3D_RS_SRCBLENDALPHA
                | WINED3D_RS_DESTBLENDALPHA
                | WINED3D_RS_BLENDOPALPHA
                | WINED3D_RS_COLORWRITEENABLE
                | WINED3D_RS_COLORWRITEENABLE1
                | WINED3D_RS_COLORWRITEENABLE2
                | WINED3D_RS_COLORWRITEENABLE3 => {
                    set_blend_state = true;
                }

                WINED3D_RS_BACK_STENCILFAIL
                | WINED3D_RS_BACK_STENCILFUNC
                | WINED3D_RS_BACK_STENCILPASS
                | WINED3D_RS_BACK_STENCILZFAIL
                | WINED3D_RS_STENCILENABLE
                | WINED3D_RS_STENCILFAIL
                | WINED3D_RS_STENCILFUNC
                | WINED3D_RS_STENCILREF
                | WINED3D_RS_STENCILMASK
                | WINED3D_RS_STENCILPASS
                | WINED3D_RS_STENCILWRITEMASK
                | WINED3D_RS_STENCILZFAIL
                | WINED3D_RS_TWOSIDEDSTENCILMODE
                | WINED3D_RS_ZENABLE
                | WINED3D_RS_ZFUNC
                | WINED3D_RS_ZWRITEENABLE => {
                    set_depth_stencil_state = true;
                }

                WINED3D_RS_ADAPTIVETESS_X
                | WINED3D_RS_ADAPTIVETESS_Z
                | WINED3D_RS_ADAPTIVETESS_W => {
                    set_depth_bounds = true;
                }

                WINED3D_RS_AMBIENT => {
                    (*changed).set_lights(1 as uint32_t);
                }

                WINED3D_RS_FILLMODE
                | WINED3D_RS_CULLMODE
                | WINED3D_RS_SLOPESCALEDEPTHBIAS
                | WINED3D_RS_DEPTHBIAS
                | WINED3D_RS_SCISSORTESTENABLE
                | WINED3D_RS_ANTIALIASEDLINEENABLE
                | WINED3D_RS_ADAPTIVETESS_Y
                | WINED3D_RS_POINTSCALEENABLE
                | WINED3D_RS_POINTSCALE_A
                | WINED3D_RS_POINTSCALE_B
                | WINED3D_RS_POINTSCALE_C
                | WINED3D_RS_TEXTUREFACTOR
                | WINED3D_RS_ALPHAREF => {}

                WINED3D_RS_ANTIALIAS => {
                    if (*state).rs[WINED3D_RS_ANTIALIAS as usize] != 0 {
                        FIXME!("Antialias not supported yet.\n");
                    }
                }

                WINED3D_RS_TEXTUREPERSPECTIVE => {}

                WINED3D_RS_WRAPU => {
                    if (*state).rs[WINED3D_RS_WRAPU as usize] != 0 {
                        FIXME!("Render state WINED3D_RS_WRAPU not implemented yet.\n");
                    }
                }

                WINED3D_RS_WRAPV => {
                    if (*state).rs[WINED3D_RS_WRAPV as usize] != 0 {
                        FIXME!("Render state WINED3D_RS_WRAPV not implemented yet.\n");
                    }
                }

                WINED3D_RS_MONOENABLE => {
                    if (*state).rs[WINED3D_RS_MONOENABLE as usize] != 0 {
                        FIXME!("Render state WINED3D_RS_MONOENABLE not implemented yet.\n");
                    }
                }

                WINED3D_RS_ROP2 => {
                    if (*state).rs[WINED3D_RS_ROP2 as usize] != 0 {
                        FIXME!("Render state WINED3D_RS_ROP2 not implemented yet.\n");
                    }
                }

                WINED3D_RS_PLANEMASK => {
                    if (*state).rs[WINED3D_RS_PLANEMASK as usize] != 0 {
                        FIXME!("Render state WINED3D_RS_PLANEMASK not implemented yet.\n");
                    }
                }

                WINED3D_RS_LASTPIXEL => {
                    if (*state).rs[WINED3D_RS_LASTPIXEL as usize] == 0 {
                        static mut warned: bool = false;
                        if !warned {
                            /* Last Pixel Drawing Disabled, not handled yet. */
                            warned = true;
                        }
                    }
                }

                WINED3D_RS_ZVISIBLE => {
                    if (*state).rs[WINED3D_RS_ZVISIBLE as usize] != 0 {
                        FIXME!("WINED3D_RS_ZVISIBLE not implemented.\n");
                    }
                }

                WINED3D_RS_SUBPIXEL => {
                    if (*state).rs[WINED3D_RS_SUBPIXEL as usize] != 0 {
                        FIXME!("Render state WINED3D_RS_SUBPIXEL not implemented yet.\n");
                    }
                }

                WINED3D_RS_SUBPIXELX => {
                    if (*state).rs[WINED3D_RS_SUBPIXELX as usize] != 0 {
                        FIXME!("Render state WINED3D_RS_SUBPIXELX not implemented yet.\n");
                    }
                }

                WINED3D_RS_STIPPLEDALPHA => {
                    if (*state).rs[WINED3D_RS_STIPPLEDALPHA as usize] != 0 {
                        FIXME!("Stippled Alpha not supported yet.\n");
                    }
                }

                WINED3D_RS_STIPPLEENABLE => {
                    if (*state).rs[WINED3D_RS_STIPPLEENABLE as usize] != 0 {
                        FIXME!("Render state WINED3D_RS_STIPPLEENABLE not implemented yet.\n");
                    }
                }

                WINED3D_RS_MIPMAPLODBIAS => {
                    if (*state).rs[WINED3D_RS_MIPMAPLODBIAS as usize] != 0 {
                        FIXME!("Render state WINED3D_RS_MIPMAPLODBIAS not implemented yet.\n");
                    }
                }

                WINED3D_RS_ANISOTROPY => {
                    if (*state).rs[WINED3D_RS_ANISOTROPY as usize] != 0 {
                        FIXME!("Render state WINED3D_RS_ANISOTROPY not implemented yet.\n");
                    }
                }

                WINED3D_RS_FLUSHBATCH => {
                    if (*state).rs[WINED3D_RS_FLUSHBATCH as usize] != 0 {
                        FIXME!("Render state WINED3D_RS_FLUSHBATCH not implemented yet.\n");
                    }
                }

                WINED3D_RS_TRANSLUCENTSORTINDEPENDENT => {
                    if (*state).rs[WINED3D_RS_TRANSLUCENTSORTINDEPENDENT as usize] != 0 {
                        FIXME!("Render state WINED3D_RS_TRANSLUCENTSORTINDEPENDENT not implemented yet.\n");
                    }
                }

                WINED3D_RS_WRAP0
                | WINED3D_RS_WRAP1
                | WINED3D_RS_WRAP2
                | WINED3D_RS_WRAP3
                | WINED3D_RS_WRAP4
                | WINED3D_RS_WRAP5
                | WINED3D_RS_WRAP6
                | WINED3D_RS_WRAP7
                | WINED3D_RS_WRAP8
                | WINED3D_RS_WRAP9
                | WINED3D_RS_WRAP10
                | WINED3D_RS_WRAP11
                | WINED3D_RS_WRAP12
                | WINED3D_RS_WRAP13
                | WINED3D_RS_WRAP14
                | WINED3D_RS_WRAP15 => {
                    static mut once: ::core::ffi::c_uint = 0;

                    if (*state).rs[idx as usize] != 0 && {
                        let fresh = once;
                        once = once.wrapping_add(1);
                        fresh == 0
                    } {
                        FIXME!("(WINED3D_RS_WRAP0) Texture wrapping not yet supported.\n");
                    }
                }

                WINED3D_RS_EXTENTS => {
                    if (*state).rs[WINED3D_RS_EXTENTS as usize] != 0 {
                        FIXME!("Render state WINED3D_RS_EXTENTS not implemented yet.\n");
                    }
                }

                WINED3D_RS_COLORKEYBLENDENABLE => {
                    if (*state).rs[WINED3D_RS_COLORKEYBLENDENABLE as usize] != 0 {
                        FIXME!("Render state WINED3D_RS_COLORKEYBLENDENABLE not implemented yet.\n");
                    }
                }

                WINED3D_RS_SOFTWAREVERTEXPROCESSING => {
                    static mut once: ::core::ffi::c_uint = 0;

                    if (*state).rs[WINED3D_RS_SOFTWAREVERTEXPROCESSING as usize] != 0 && {
                        let fresh = once;
                        once = once.wrapping_add(1);
                        fresh == 0
                    } {
                        FIXME!("Software vertex processing not implemented.\n");
                    }
                }

                WINED3D_RS_PATCHEDGESTYLE => {
                    if (*state).rs[WINED3D_RS_PATCHEDGESTYLE as usize] != WINED3D_PATCH_EDGE_DISCRETE as uint32_t {
                        FIXME!("WINED3D_RS_PATCHEDGESTYLE %#x not yet implemented.\n",
                                (*state).rs[WINED3D_RS_PATCHEDGESTYLE as usize]);
                    }
                }

                WINED3D_RS_PATCHSEGMENTS => {
                    #[repr(C)]
                    union TmpValue {
                        d: uint32_t,
                        f: f32,
                    }
                    let mut tmpvalue: TmpValue = TmpValue { d: 0 };
                    tmpvalue.f = 1.0f32;

                    if (*state).rs[WINED3D_RS_PATCHSEGMENTS as usize] != tmpvalue.d {
                        static mut displayed: bool = false;

                        tmpvalue.d = (*state).rs[WINED3D_RS_PATCHSEGMENTS as usize] as uint32_t;
                        if !displayed {
                            FIXME!("(WINED3D_RS_PATCHSEGMENTS,%f) not yet implemented.\n", tmpvalue.f as core::ffi::c_double);
                        }

                        displayed = true;
                    }
                }

                WINED3D_RS_DEBUGMONITORTOKEN => {
                    WARN!("token: %#x.\n", (*state).rs[WINED3D_RS_DEBUGMONITORTOKEN as usize]);
                }

                WINED3D_RS_INDEXEDVERTEXBLENDENABLE => {}

                WINED3D_RS_TWEENFACTOR => {}

                WINED3D_RS_POSITIONDEGREE => {
                    if (*state).rs[WINED3D_RS_POSITIONDEGREE as usize] != WINED3D_DEGREE_CUBIC as uint32_t {
                        FIXME!("WINED3D_RS_POSITIONDEGREE %#x not yet implemented.\n",
                                (*state).rs[WINED3D_RS_POSITIONDEGREE as usize]);
                    }
                }

                WINED3D_RS_NORMALDEGREE => {
                    if (*state).rs[WINED3D_RS_NORMALDEGREE as usize] != WINED3D_DEGREE_LINEAR as uint32_t {
                        FIXME!("WINED3D_RS_NORMALDEGREE %#x not yet implemented.\n",
                                (*state).rs[WINED3D_RS_NORMALDEGREE as usize]);
                    }
                }

                WINED3D_RS_MINTESSELLATIONLEVEL => {}

                WINED3D_RS_MAXTESSELLATIONLEVEL => {}

                WINED3D_RS_ENABLEADAPTIVETESSELLATION => {
                    if (*state).rs[WINED3D_RS_ENABLEADAPTIVETESSELLATION as usize] != 0 {
                        FIXME!("WINED3D_RS_ENABLEADAPTIVETESSELLATION %#x not yet implemented.\n",
                                (*state).rs[WINED3D_RS_ENABLEADAPTIVETESSELLATION as usize]);
                    }
                }

                _ => {
                    wined3d_device_set_render_state(device, idx, (*state).rs[idx as usize]);
                }
            }
        }
        i = i.wrapping_add(1);
    }

    if set_blend_state
        || (*changed).alpha_to_coverage() != 0
        || wined3d_bitmap_is_set(core::ptr::addr_of_mut!((*changed).renderState) as *mut uint32_t, WINED3D_RS_ADAPTIVETESS_Y) != 0
    {
        let mut blend_state: *mut wined3d_blend_state = core::ptr::null_mut();
        let mut desc: wined3d_blend_state_desc = core::mem::zeroed();
        let mut entry: *mut wine_rb_entry = core::ptr::null_mut();
        let mut colour: wined3d_color = core::mem::zeroed();
        let mut sample_mask: ::core::ffi::c_uint = 0;

        memset(core::ptr::addr_of_mut!(desc) as *mut ::core::ffi::c_void, 0 as ::core::ffi::c_int, ::core::mem::size_of::<wined3d_blend_state_desc>() as usize);
        desc.alpha_to_coverage = (*state).alpha_to_coverage;
        desc.independent = FALSE as BOOL;
        if (*state).rs[WINED3D_RS_ADAPTIVETESS_Y as usize] == WINED3DFMT_ATOC as uint32_t {
            desc.alpha_to_coverage = TRUE as BOOL;
        }
        desc.rt[0 as usize].enable = (*state).rs[WINED3D_RS_ALPHABLENDENABLE as usize] as BOOL;
        desc.rt[0 as usize].src = (*state).rs[WINED3D_RS_SRCBLEND as usize] as wined3d_blend;
        desc.rt[0 as usize].dst = (*state).rs[WINED3D_RS_DESTBLEND as usize] as wined3d_blend;
        desc.rt[0 as usize].op = (*state).rs[WINED3D_RS_BLENDOP as usize] as wined3d_blend_op;
        if (*state).rs[WINED3D_RS_SEPARATEALPHABLENDENABLE as usize] != 0 {
            desc.rt[0 as usize].src_alpha = (*state).rs[WINED3D_RS_SRCBLENDALPHA as usize] as wined3d_blend;
            desc.rt[0 as usize].dst_alpha = (*state).rs[WINED3D_RS_DESTBLENDALPHA as usize] as wined3d_blend;
            desc.rt[0 as usize].op_alpha = (*state).rs[WINED3D_RS_BLENDOPALPHA as usize] as wined3d_blend_op;
        } else {
            desc.rt[0 as usize].src_alpha = (*state).rs[WINED3D_RS_SRCBLEND as usize] as wined3d_blend;
            desc.rt[0 as usize].dst_alpha = (*state).rs[WINED3D_RS_DESTBLEND as usize] as wined3d_blend;
            desc.rt[0 as usize].op_alpha = (*state).rs[WINED3D_RS_BLENDOP as usize] as wined3d_blend_op;
        }
        desc.rt[0 as usize].writemask = (*state).rs[WINED3D_RS_COLORWRITEENABLE as usize];
        desc.rt[1 as usize].writemask = (*state).rs[WINED3D_RS_COLORWRITEENABLE1 as usize];
        desc.rt[2 as usize].writemask = (*state).rs[WINED3D_RS_COLORWRITEENABLE2 as usize];
        desc.rt[3 as usize].writemask = (*state).rs[WINED3D_RS_COLORWRITEENABLE3 as usize];
        if desc.rt[1 as usize].writemask != desc.rt[0 as usize].writemask
            || desc.rt[2 as usize].writemask != desc.rt[0 as usize].writemask
            || desc.rt[3 as usize].writemask != desc.rt[0 as usize].writemask
        {
            desc.independent = TRUE as BOOL;
            i = 1;
            while i < 4 {
                desc.rt[i as usize].enable = desc.rt[0 as usize].enable;
                desc.rt[i as usize].src = desc.rt[0 as usize].src;
                desc.rt[i as usize].dst = desc.rt[0 as usize].dst;
                desc.rt[i as usize].op = desc.rt[0 as usize].op;
                desc.rt[i as usize].src_alpha = desc.rt[0 as usize].src_alpha;
                desc.rt[i as usize].dst_alpha = desc.rt[0 as usize].dst_alpha;
                desc.rt[i as usize].op_alpha = desc.rt[0 as usize].op_alpha;
                i = i.wrapping_add(1);
            }
        }

        if wined3d_bitmap_is_set(core::ptr::addr_of_mut!((*changed).renderState) as *mut uint32_t, WINED3D_RS_BLENDFACTOR) != 0 {
            wined3d_color_from_d3dcolor(core::ptr::addr_of_mut!(colour), (*state).rs[WINED3D_RS_BLENDFACTOR as usize]);
        } else {
            wined3d_device_context_get_blend_state(context, core::ptr::addr_of_mut!(colour), core::ptr::addr_of_mut!(sample_mask));
        }

        entry = wine_rb_get(core::ptr::addr_of_mut!((*device).blend_states), core::ptr::addr_of!(desc) as *const ::core::ffi::c_void);
        if !entry.is_null() {
            blend_state = (entry as *mut ::core::ffi::c_char)
                .offset(-(core::mem::offset_of!(wined3d_blend_state, entry) as isize))
                as *mut wined3d_blend_state;
            wined3d_device_context_set_blend_state(context, blend_state, core::ptr::addr_of_mut!(colour),
                    (*state).rs[WINED3D_RS_MULTISAMPLEMASK as usize]);
        } else if wined3d_blend_state_create(device, core::ptr::addr_of_mut!(desc), core::ptr::null_mut(),
                core::ptr::addr_of!(wined3d_null_parent_ops), core::ptr::addr_of_mut!(blend_state)) >= 0 {
            wined3d_device_context_set_blend_state(context, blend_state, core::ptr::addr_of_mut!(colour),
                    (*state).rs[WINED3D_RS_MULTISAMPLEMASK as usize]);
            if wine_rb_put(core::ptr::addr_of_mut!((*device).blend_states), core::ptr::addr_of!(desc) as *const ::core::ffi::c_void,
                    core::ptr::addr_of_mut!((*blend_state).entry)) == -1 {
                ERR!("Failed to insert blend state.\n");
                wined3d_blend_state_decref(blend_state);
            }
        }
    }

    if set_depth_stencil_state {
        let mut depth_stencil_state: *mut wined3d_depth_stencil_state = core::ptr::null_mut();
        let mut desc: wined3d_depth_stencil_state_desc = core::mem::zeroed();
        let mut entry: *mut wine_rb_entry = core::ptr::null_mut();
        let mut stencil_ref: ::core::ffi::c_uint = 0;

        memset(core::ptr::addr_of_mut!(desc) as *mut ::core::ffi::c_void, 0 as ::core::ffi::c_int, ::core::mem::size_of::<wined3d_depth_stencil_state_desc>() as usize);
        match (*state).rs[WINED3D_RS_ZENABLE as usize] {
            WINED3D_ZB_FALSE => {
                desc.depth = FALSE as BOOL;
            }

            WINED3D_ZB_USEW => {
                FIXME!("W buffer is not well handled.\n");
                desc.depth = TRUE as BOOL;
            }

            WINED3D_ZB_TRUE => {
                desc.depth = TRUE as BOOL;
            }

            _ => {
                FIXME!("Unrecognized depth buffer type %#x.\n", (*state).rs[WINED3D_RS_ZENABLE as usize]);
            }
        }
        desc.depth_write = (*state).rs[WINED3D_RS_ZWRITEENABLE as usize] as BOOL;
        desc.depth_func = (*state).rs[WINED3D_RS_ZFUNC as usize] as wined3d_cmp_func;
        desc.stencil = (*state).rs[WINED3D_RS_STENCILENABLE as usize] as BOOL;
        desc.stencil_read_mask = (*state).rs[WINED3D_RS_STENCILMASK as usize];
        desc.stencil_write_mask = (*state).rs[WINED3D_RS_STENCILWRITEMASK as usize];
        desc.front.fail_op = (*state).rs[WINED3D_RS_STENCILFAIL as usize] as wined3d_stencil_op;
        desc.front.depth_fail_op = (*state).rs[WINED3D_RS_STENCILZFAIL as usize] as wined3d_stencil_op;
        desc.front.pass_op = (*state).rs[WINED3D_RS_STENCILPASS as usize] as wined3d_stencil_op;
        desc.front.func = (*state).rs[WINED3D_RS_STENCILFUNC as usize] as wined3d_cmp_func;

        if (*state).rs[WINED3D_RS_TWOSIDEDSTENCILMODE as usize] != 0 {
            desc.back.fail_op = (*state).rs[WINED3D_RS_BACK_STENCILFAIL as usize] as wined3d_stencil_op;
            desc.back.depth_fail_op = (*state).rs[WINED3D_RS_BACK_STENCILZFAIL as usize] as wined3d_stencil_op;
            desc.back.pass_op = (*state).rs[WINED3D_RS_BACK_STENCILPASS as usize] as wined3d_stencil_op;
            desc.back.func = (*state).rs[WINED3D_RS_BACK_STENCILFUNC as usize] as wined3d_cmp_func;
        } else {
            desc.back = desc.front;
        }

        if wined3d_bitmap_is_set(core::ptr::addr_of_mut!((*changed).renderState) as *mut uint32_t, WINED3D_RS_STENCILREF) != 0 {
            stencil_ref = (*state).rs[WINED3D_RS_STENCILREF as usize];
        } else {
            wined3d_device_context_get_depth_stencil_state(context, core::ptr::addr_of_mut!(stencil_ref));
        }

        entry = wine_rb_get(core::ptr::addr_of_mut!((*device).depth_stencil_states), core::ptr::addr_of!(desc) as *const ::core::ffi::c_void);
        if !entry.is_null() {
            depth_stencil_state = (entry as *mut ::core::ffi::c_char)
                .offset(-(core::mem::offset_of!(wined3d_depth_stencil_state, entry) as isize))
                as *mut wined3d_depth_stencil_state;
            wined3d_device_context_set_depth_stencil_state(context, depth_stencil_state, stencil_ref);
        } else if wined3d_depth_stencil_state_create(device, core::ptr::addr_of_mut!(desc), core::ptr::null_mut(),
                core::ptr::addr_of!(wined3d_null_parent_ops), core::ptr::addr_of_mut!(depth_stencil_state)) >= 0 {
            wined3d_device_context_set_depth_stencil_state(context, depth_stencil_state, stencil_ref);
            if wine_rb_put(core::ptr::addr_of_mut!((*device).depth_stencil_states), core::ptr::addr_of!(desc) as *const ::core::ffi::c_void,
                    core::ptr::addr_of_mut!((*depth_stencil_state).entry)) == -1 {
                ERR!("Failed to insert depth/stencil state.\n");
                wined3d_depth_stencil_state_decref(depth_stencil_state);
            }
        }
    }

    if set_depth_bounds {
        wined3d_device_context_set_depth_bounds(context,
                (*state).rs[WINED3D_RS_ADAPTIVETESS_X as usize] == WINED3DFMT_NVDB as uint32_t,
                int_to_float((*state).rs[WINED3D_RS_ADAPTIVETESS_Z as usize]),
                int_to_float((*state).rs[WINED3D_RS_ADAPTIVETESS_W as usize]));
    }

    i = 0;
    while (i as usize) < ARRAY_SIZE!((*changed).textureState) as usize {
        map = (*changed).textureState[i as usize];
        while map != 0 {
            j = wined3d_bit_scan(core::ptr::addr_of_mut!(map)) as ::core::ffi::c_uint;

            match j {
                WINED3D_TSS_BUMPENV_LOFFSET
                | WINED3D_TSS_BUMPENV_LSCALE
                | WINED3D_TSS_BUMPENV_MAT00
                | WINED3D_TSS_BUMPENV_MAT01
                | WINED3D_TSS_BUMPENV_MAT10
                | WINED3D_TSS_BUMPENV_MAT11
                | WINED3D_TSS_CONSTANT => {}

                _ => {
                    wined3d_device_set_texture_stage_state(device, i, j, (*state).texture_states[i as usize][j as usize]);
                }
            }
        }
        i = i.wrapping_add(1);
    }

    i = 0;
    while (i as usize) < ARRAY_SIZE!((*changed).samplerState) as usize {
        'body: {
            let mut shader_type: wined3d_shader_type = WINED3D_SHADER_TYPE_PIXEL as wined3d_shader_type;
            let mut desc: wined3d_sampler_desc = core::mem::zeroed();
            let mut texture: *mut wined3d_texture = core::ptr::null_mut();
            let mut sampler: *mut wined3d_sampler = core::ptr::null_mut();
            let mut bind_index: ::core::ffi::c_uint = i;
            let mut entry: *mut wine_rb_entry = core::ptr::null_mut();

            if (*changed).samplerState[i as usize] == 0
                && ((*changed).textures() as ::core::ffi::c_uint
                    & ((1 as ::core::ffi::c_uint) << i)) == 0
            {
                break 'body;
            }

            texture = (*state).textures[i as usize] as *mut wined3d_texture;
            if texture.is_null() {
                break 'body;
            }

            memset(core::ptr::addr_of_mut!(desc) as *mut ::core::ffi::c_void, 0 as ::core::ffi::c_int, ::core::mem::size_of::<wined3d_sampler_desc>() as usize);
            sampler_desc_from_sampler_states(core::ptr::addr_of_mut!(desc),
                    (core::ptr::addr_of!((*state).sampler_states) as *const [uint32_t; 14]).offset(i as isize) as *const uint32_t,
                    texture);

            if i >= WINED3D_VERTEX_SAMPLER_OFFSET as ::core::ffi::c_uint {
                shader_type = WINED3D_SHADER_TYPE_VERTEX as wined3d_shader_type;
                bind_index = bind_index.wrapping_sub(WINED3D_VERTEX_SAMPLER_OFFSET as ::core::ffi::c_uint);
            }

            entry = wine_rb_get(core::ptr::addr_of_mut!((*device).samplers), core::ptr::addr_of!(desc) as *const ::core::ffi::c_void);
            if !entry.is_null() {
                sampler = (entry as *mut ::core::ffi::c_char)
                    .offset(-(core::mem::offset_of!(wined3d_sampler, entry) as isize))
                    as *mut wined3d_sampler;

                wined3d_device_context_set_samplers(context, shader_type, bind_index, 1 as ::core::ffi::c_uint, core::ptr::addr_of_mut!(sampler));
            } else if wined3d_sampler_create(device, core::ptr::addr_of_mut!(desc), core::ptr::null_mut(),
                    core::ptr::addr_of!(wined3d_null_parent_ops), core::ptr::addr_of_mut!(sampler)) >= 0 {
                wined3d_device_context_set_samplers(context, shader_type, bind_index, 1 as ::core::ffi::c_uint, core::ptr::addr_of_mut!(sampler));

                if wine_rb_put(core::ptr::addr_of_mut!((*device).samplers), core::ptr::addr_of!(desc) as *const ::core::ffi::c_void,
                        core::ptr::addr_of_mut!((*sampler).entry)) == -1 {
                    ERR!("Failed to insert sampler.\n");
                    wined3d_sampler_decref(sampler);
                }
            }
        }
        i = i.wrapping_add(1);
    }

    if !(*state).vertex_declaration.is_null()
        && (*(*state).vertex_declaration).position_transformed != false
    {
        /* We reuse the projection matrix to undo the translation between
         * clip coordinates and pixel coordinates. */
        if (*changed).position_transformed() != 0 || (*changed).viewport() != 0 {
            let x: f32 = (*state).viewport.x;
            let y: f32 = (*state).viewport.y;
            let w: f32 = (*state).viewport.width;
            let h: f32 = (*state).viewport.height;
            let x_scale: f32 = 2.0f32 / w;
            let x_offset: f32 = (-(2.0f32 * x) - w) / w;
            let y_scale: f32 = 2.0f32 / -h;
            let y_offset: f32 = (-(2.0f32 * y) - h) / -h;
            let depth: bool = (*state).rs[WINED3D_RS_ZENABLE as usize] != 0
                && !(*(*context).state).fb.depth_stencil.is_null();
            let z_scale: f32 = if depth { 1.0f32 } else { 0.0f32 };
            let matrix: wined3d_matrix = wined3d_matrix {
                _11: x_scale,
                _12: 0.0f32,
                _13: 0.0f32,
                _14: 0.0f32,
                _21: 0.0f32,
                _22: y_scale,
                _23: 0.0f32,
                _24: 0.0f32,
                _31: 0.0f32,
                _32: 0.0f32,
                _33: z_scale,
                _34: 0.0f32,
                _41: x_offset,
                _42: y_offset,
                _43: 0.0f32,
                _44: 1.0f32,
            };

            wined3d_device_context_push_constants(context, WINED3D_PUSH_CONSTANTS_VS_FFP as ::core::ffi::c_uint, WINED3D_SHADER_CONST_FFP_PROJ as uint32_t,
                    core::mem::offset_of!(wined3d_ffp_vs_constants, projection_matrix) as ::core::ffi::c_uint,
                    ::core::mem::size_of::<wined3d_matrix>() as ::core::ffi::c_uint,
                    core::ptr::addr_of!(matrix) as *const ::core::ffi::c_void);
        }

        if wined3d_bitmap_is_set(core::ptr::addr_of_mut!((*changed).transform) as *mut uint32_t, WINED3D_TS_PROJECTION) != 0 {
            /* wined3d_ffp_vs_settings.ortho_fog still needs the
             * device state to be set. */
            wined3d_device_set_transform(device, WINED3D_TS_PROJECTION as wined3d_transform_state,
                    (core::ptr::addr_of!((*state).transforms) as *const wined3d_matrix).offset(WINED3D_TS_PROJECTION as isize));
        }
    } else if (*changed).transforms() != 0 {
        if wined3d_bitmap_is_set(core::ptr::addr_of_mut!((*changed).transform) as *mut uint32_t, WINED3D_TS_VIEW) != 0 {
            (*changed).set_lights(1 as uint32_t);
            (*changed).clipplane = wined3d_mask_from_size(WINED3D_MAX_CLIP_DISTANCES);
        }

        if wined3d_bitmap_is_set(core::ptr::addr_of_mut!((*changed).transform) as *mut uint32_t, WINED3D_TS_PROJECTION) != 0
            || (*changed).position_transformed() != 0
        {
            let dst_matrix: *const wined3d_matrix = (core::ptr::addr_of!((*state).transforms) as *const wined3d_matrix)
                .offset(WINED3D_TS_PROJECTION as isize);
            wined3d_device_context_push_constants(context,
                    WINED3D_PUSH_CONSTANTS_VS_FFP as ::core::ffi::c_uint, WINED3D_SHADER_CONST_FFP_PROJ as uint32_t,
                    core::mem::offset_of!(wined3d_ffp_vs_constants, projection_matrix) as ::core::ffi::c_uint,
                    core::mem::size_of_val(&(*state).transforms[WINED3D_TS_PROJECTION as usize]) as ::core::ffi::c_uint,
                    dst_matrix as *const ::core::ffi::c_void);
            /* wined3d_ffp_vs_settings.ortho_fog and vs_compile_args.ortho_fog
             * still need the device state to be set. */
            wined3d_device_set_transform(device, WINED3D_TS_PROJECTION as wined3d_transform_state,
                    (core::ptr::addr_of!((*state).transforms) as *const wined3d_matrix).offset(WINED3D_TS_PROJECTION as isize));
        }

        /* Clip planes are affected by the view matrix. */
        (*changed).clipplane = wined3d_mask_from_size(WINED3D_MAX_CLIP_DISTANCES);
    }

    if (*changed).indices() != 0 {
        wined3d_device_context_set_index_buffer(context, (*state).index_buffer, (*state).index_format, 0 as ::core::ffi::c_uint);
    }
    wined3d_device_set_base_vertex_index(device, (*state).base_vertex_index);
    if (*changed).vertexDecl() != 0 {
        wined3d_device_context_set_vertex_declaration(context, (*state).vertex_declaration as *mut wined3d_vertex_declaration);
    }

    map = ((*changed).streamSource as uint32_t) | ((*changed).streamFreq as uint32_t);
    while map != 0 {
        i = wined3d_bit_scan(core::ptr::addr_of_mut!(map)) as ::core::ffi::c_uint;
        wined3d_device_context_set_stream_sources(context, i, 1 as ::core::ffi::c_uint,
                (core::ptr::addr_of!((*state).streams) as *const wined3d_stream_state).offset(i as isize));
    }

    map = (*changed).textures();
    while map != 0 {
        let mut float_key: [wined3d_color; 2 as usize] = core::mem::zeroed();
        let mut texture: *mut wined3d_texture = core::ptr::null_mut();

        i = wined3d_bit_scan(core::ptr::addr_of_mut!(map)) as ::core::ffi::c_uint;
        texture = (*state).textures[i as usize] as *mut wined3d_texture;
        wined3d_device_set_texture(device, i, texture);

        if i == 0 && !texture.is_null() && ((*texture).color_key_flags & (WINED3D_CKEY_SRC_BLT as uint32_t)) != 0 {
            wined3d_format_get_float_color_key((*texture).resource.format,
                    core::ptr::addr_of_mut!((*texture).src_blt_color_key),
                    core::ptr::addr_of_mut!(float_key) as *mut wined3d_color);
            wined3d_device_context_push_constants(context,
                    WINED3D_PUSH_CONSTANTS_PS_FFP as ::core::ffi::c_uint, WINED3D_SHADER_CONST_FFP_COLOR_KEY as uint32_t,
                    core::mem::offset_of!(wined3d_ffp_ps_constants, color_key) as ::core::ffi::c_uint,
                    core::mem::size_of_val(&float_key) as ::core::ffi::c_uint,
                    core::ptr::addr_of!(float_key) as *const ::core::ffi::c_void);
        }
    }

    map = (*changed).clipplane;
    while map != 0 {
        i = wined3d_bit_scan(core::ptr::addr_of_mut!(map)) as ::core::ffi::c_uint;

        /* In Direct3D, clipping is done based on the position as transformed
         * by the world (model) matrix, but not the view matrix.
         *
         * GL and Vulkan do not distinguish the world and view, so we pass them
         * as a single uniform. That means, however, that we need to unapply the
         * view matrix from the clip planes that we are applying. We do this by
         * multiplying by the transpose of the inverse of the view matrix.
         *
         * This works mathematically (c = clip plane, p = position):
         *
         * clip distance = dot((V⁻¹)ᵀc, VMp)
         *               = ((V⁻¹)ᵀc)ᵀVMp
         *               = cᵀV⁻¹VMp
         *               = cᵀMp
         *               = dot(c, Mp)
         */

        if (*state).vs.is_null() {
            let mut matrix: wined3d_matrix = core::mem::zeroed();
            let mut plane: wined3d_vec4 = core::mem::zeroed();

            invert_matrix(core::ptr::addr_of_mut!(matrix),
                    (core::ptr::addr_of!((*state).transforms) as *const wined3d_matrix).offset(WINED3D_TS_VIEW as isize));
            transpose_matrix(core::ptr::addr_of_mut!(matrix), core::ptr::addr_of_mut!(matrix));
            wined3d_vec4_transform(core::ptr::addr_of_mut!(plane),
                    (core::ptr::addr_of!((*state).clip_planes) as *const wined3d_vec4).offset(i as isize),
                    core::ptr::addr_of_mut!(matrix));
            wined3d_device_set_clip_plane(device, i, core::ptr::addr_of_mut!(plane));
        } else {
            wined3d_device_set_clip_plane(device, i,
                    (core::ptr::addr_of!((*state).clip_planes) as *const wined3d_vec4).offset(i as isize));
        }
    }

    if (*changed).material() != 0 {
        wined3d_device_context_push_constants(context, WINED3D_PUSH_CONSTANTS_VS_FFP as ::core::ffi::c_uint, WINED3D_SHADER_CONST_FFP_MATERIAL as uint32_t,
                core::mem::offset_of!(wined3d_ffp_vs_constants, material) as ::core::ffi::c_uint,
                ::core::mem::size_of::<wined3d_material>() as ::core::ffi::c_uint,
                core::ptr::addr_of!((*state).material) as *const ::core::ffi::c_void);
    }

    if (*changed).lights() != 0 {
        let mut point_idx: ::core::ffi::c_uint;
        let mut spot_idx: ::core::ffi::c_uint;
        let mut directional_idx: ::core::ffi::c_uint;
        let mut parallel_point_idx: ::core::ffi::c_uint;
        let mut point_count: ::core::ffi::c_uint = 0;
        let mut spot_count: ::core::ffi::c_uint = 0;
        let mut directional_count: ::core::ffi::c_uint = 0;
        let mut constants: wined3d_ffp_light_constants = core::mem::zeroed();
        let mut light: *mut wined3d_light_info;
        let mut cursor: *mut wined3d_light_info;

        /* LIST_FOR_EACH_ENTRY_SAFE(light, cursor, &changed->changed_lights, ...) */
        light = ((*changed).changed_lights.next as *mut ::core::ffi::c_char)
            .offset(-(core::mem::offset_of!(wined3d_light_info, changed_entry) as isize))
            as *mut wined3d_light_info;
        cursor = ((*light).changed_entry.next as *mut ::core::ffi::c_char)
            .offset(-(core::mem::offset_of!(wined3d_light_info, changed_entry) as isize))
            as *mut wined3d_light_info;
        while core::ptr::addr_of_mut!((*light).changed_entry) != core::ptr::addr_of_mut!((*changed).changed_lights) {
            wined3d_device_context_set_light(context, (*light).OriginalIndex, core::ptr::addr_of_mut!((*light).OriginalParms));
            wined3d_device_set_light_enable(device, (*light).OriginalIndex, (*light).glIndex != -(1 as ::core::ffi::c_int));
            list_remove(core::ptr::addr_of_mut!((*light).changed_entry));
            (*light).changed = false;

            light = cursor;
            cursor = ((*cursor).changed_entry.next as *mut ::core::ffi::c_char)
                .offset(-(core::mem::offset_of!(wined3d_light_info, changed_entry) as isize))
                as *mut wined3d_light_info;
        }

        wined3d_color_from_d3dcolor(core::ptr::addr_of_mut!(constants.ambient), (*state).rs[WINED3D_RS_AMBIENT as usize]);

        let mut i_0: ::core::ffi::c_uint = 0;
        while i_0 < WINED3D_MAX_ACTIVE_LIGHTS {
            'body: {
                if (*(*state).light_state).lights[i_0 as usize].is_null() {
                    break 'body;
                }

                match (*(*(*state).light_state).lights[i_0 as usize]).OriginalParms.type_ as ::core::ffi::c_uint {
                    WINED3D_LIGHT_POINT => {
                        point_count += 1;
                    }
                    WINED3D_LIGHT_SPOT => {
                        spot_count += 1;
                    }
                    WINED3D_LIGHT_DIRECTIONAL => {
                        directional_count += 1;
                    }
                    WINED3D_LIGHT_PARALLELPOINT => {}
                    _ => {
                        FIXME!("Unhandled light type %#x.\n",
                                (*(*(*state).light_state).lights[i_0 as usize]).OriginalParms.type_ as ::core::ffi::c_uint);
                    }
                }
            }
            i_0 = i_0.wrapping_add(1);
        }

        point_idx = 0;
        spot_idx = point_idx + point_count;
        directional_idx = spot_idx + spot_count;
        parallel_point_idx = directional_idx + directional_count;

        let mut i_1: ::core::ffi::c_uint = 0;
        while i_1 < WINED3D_MAX_ACTIVE_LIGHTS {
            'body: {
                let light_info: *const wined3d_light_info = (*(*state).light_state).lights[i_1 as usize];
                let light_constants: *const wined3d_light_constants;
                let type_0: wined3d_light_type;
                let idx_0: ::core::ffi::c_uint;

                if light_info.is_null() {
                    break 'body;
                }
                type_0 = (*light_info).OriginalParms.type_;
                light_constants = core::ptr::addr_of!((*(*(*state).light_state).lights[i_1 as usize]).constants);

                idx_0 = match type_0 as ::core::ffi::c_uint {
                    WINED3D_LIGHT_POINT => {
                        let v = point_idx;
                        point_idx += 1;
                        v
                    }
                    WINED3D_LIGHT_SPOT => {
                        let v = spot_idx;
                        spot_idx += 1;
                        v
                    }
                    WINED3D_LIGHT_DIRECTIONAL => {
                        let v = directional_idx;
                        directional_idx += 1;
                        v
                    }
                    WINED3D_LIGHT_PARALLELPOINT => {
                        let v = parallel_point_idx;
                        parallel_point_idx += 1;
                        v
                    }
                    _ => {
                        FIXME!("Unhandled light type %#x.\n", type_0 as ::core::ffi::c_uint);
                        break 'body;
                    }
                };

                constants.lights[idx_0 as usize] = *light_constants;

                let dst: *mut wined3d_light_constants = (core::ptr::addr_of_mut!(constants.lights) as *mut wined3d_light_constants)
                    .offset(idx_0 as isize);

                if type_0 as ::core::ffi::c_uint != WINED3D_LIGHT_DIRECTIONAL as ::core::ffi::c_uint {
                    wined3d_vec4_transform(core::ptr::addr_of_mut!((*dst).position),
                            core::ptr::addr_of!((*light_constants).position),
                            (core::ptr::addr_of!((*state).transforms) as *const wined3d_matrix).offset(WINED3D_TS_VIEW as isize));
                }
                if type_0 as ::core::ffi::c_uint == WINED3D_LIGHT_SPOT as ::core::ffi::c_uint
                    || type_0 as ::core::ffi::c_uint == WINED3D_LIGHT_DIRECTIONAL as ::core::ffi::c_uint
                {
                    wined3d_vec4_transform(core::ptr::addr_of_mut!((*dst).direction),
                            core::ptr::addr_of!((*light_constants).direction),
                            (core::ptr::addr_of!((*state).transforms) as *const wined3d_matrix).offset(WINED3D_TS_VIEW as isize));
                }
            }
            i_1 = i_1.wrapping_add(1);
        }

        wined3d_device_context_push_constants(context, WINED3D_PUSH_CONSTANTS_VS_FFP as ::core::ffi::c_uint, WINED3D_SHADER_CONST_FFP_LIGHTS as uint32_t,
                core::mem::offset_of!(wined3d_ffp_vs_constants, light) as ::core::ffi::c_uint,
                ::core::mem::size_of::<wined3d_ffp_light_constants>() as ::core::ffi::c_uint,
                core::ptr::addr_of!(constants) as *const ::core::ffi::c_void);
    }

    if (*changed).modelview_matrices() != 0 {
        let mut matrices: wined3d_ffp_vs_modelview_matrices = core::mem::zeroed();

        get_modelview_matrix(state, 0 as ::core::ffi::c_uint,
                (core::ptr::addr_of_mut!(matrices.modelview_matrices) as *mut wined3d_matrix).offset(0 as isize));

        if (*state).rs[WINED3D_RS_VERTEXBLEND as usize] != 0 {
            i = 1;
            while i < MAX_VERTEX_BLENDS {
                get_modelview_matrix(state, i,
                        (core::ptr::addr_of_mut!(matrices.modelview_matrices) as *mut wined3d_matrix).offset(i as isize));
                i = i.wrapping_add(1);
            }

            wined3d_device_context_push_constants(context, WINED3D_PUSH_CONSTANTS_VS_FFP as ::core::ffi::c_uint,
                    (WINED3D_SHADER_CONST_FFP_MODELVIEW | WINED3D_SHADER_CONST_FFP_VERTEXBLEND) as uint32_t,
                    core::mem::offset_of!(wined3d_ffp_vs_constants, modelview) as ::core::ffi::c_uint,
                    ::core::mem::size_of::<wined3d_ffp_vs_modelview_matrices>() as ::core::ffi::c_uint,
                    core::ptr::addr_of!(matrices) as *const ::core::ffi::c_void);
        } else {
            compute_normal_matrix(core::ptr::addr_of_mut!(matrices.not_blended.normal_matrix),
                    ((*(*device).adapter).d3d_info.wined3d_creation_flags & (WINED3D_LEGACY_FFP_LIGHTING as uint32_t)) as BOOL,
                    core::ptr::addr_of_mut!(matrices.not_blended.modelview_matrix));

            wined3d_device_context_push_constants(context, WINED3D_PUSH_CONSTANTS_VS_FFP as ::core::ffi::c_uint,
                    WINED3D_SHADER_CONST_FFP_MODELVIEW as uint32_t,
                    core::mem::offset_of!(wined3d_ffp_vs_constants, modelview) as ::core::ffi::c_uint,
                    core::mem::size_of_val(&matrices.not_blended) as ::core::ffi::c_uint,
                    core::ptr::addr_of!(matrices) as *const ::core::ffi::c_void);
        }
    }

    if (*changed).texture_matrices() != 0 {
        let mut constants: wined3d_ffp_vs_constants = core::mem::zeroed();

        i = 0;
        while i < WINED3D_MAX_FFP_TEXTURES {
            get_texture_matrix(state, i,
                    (core::ptr::addr_of_mut!(constants.texture_matrices) as *mut wined3d_matrix).offset(i as isize));
            i = i.wrapping_add(1);
        }
        wined3d_device_context_push_constants(context,
                WINED3D_PUSH_CONSTANTS_VS_FFP as ::core::ffi::c_uint, WINED3D_SHADER_CONST_FFP_TEXMATRIX as uint32_t,
                core::mem::offset_of!(wined3d_ffp_vs_constants, texture_matrices) as ::core::ffi::c_uint,
                core::mem::size_of_val(&constants.texture_matrices) as ::core::ffi::c_uint,
                core::ptr::addr_of!(constants.texture_matrices) as *const ::core::ffi::c_void);
    }

    if (*changed).point_scale() != 0 {
        let mut constants: wined3d_ffp_point_constants = core::mem::zeroed();

        constants.size = int_to_float((*state).rs[WINED3D_RS_POINTSIZE as usize]);

        if (*state).rs[WINED3D_RS_POINTSCALEENABLE as usize] != 0 {
            let scale_factor: f32 = (*state).viewport.height * (*state).viewport.height;

            constants.scale_const = int_to_float((*state).rs[WINED3D_RS_POINTSCALE_A as usize]) / scale_factor;
            constants.scale_linear = int_to_float((*state).rs[WINED3D_RS_POINTSCALE_B as usize]) / scale_factor;
            constants.scale_quad = int_to_float((*state).rs[WINED3D_RS_POINTSCALE_C as usize]) / scale_factor;
        } else {
            constants.scale_const = 1.0f32;
            constants.scale_linear = 0.0f32;
            constants.scale_quad = 0.0f32;
        }

        wined3d_device_context_push_constants(context, WINED3D_PUSH_CONSTANTS_VS_FFP as ::core::ffi::c_uint, WINED3D_SHADER_CONST_VS_POINTSIZE as uint32_t,
                core::mem::offset_of!(wined3d_ffp_vs_constants, point) as ::core::ffi::c_uint,
                ::core::mem::size_of::<wined3d_ffp_point_constants>() as ::core::ffi::c_uint,
                core::ptr::addr_of!(constants) as *const ::core::ffi::c_void);
    }

    if (*changed).bumpenv_constants() != 0 {
        let mut constants: wined3d_ffp_bumpenv_constants = core::mem::zeroed();

        let mut i_2: ::core::ffi::c_uint = 0;
        while i_2 < WINED3D_MAX_FFP_TEXTURES {
            constants.matrices[i_2 as usize]._00 = int_to_float((*state).texture_states[i_2 as usize][WINED3D_TSS_BUMPENV_MAT00 as usize]);
            constants.matrices[i_2 as usize]._01 = int_to_float((*state).texture_states[i_2 as usize][WINED3D_TSS_BUMPENV_MAT01 as usize]);
            constants.matrices[i_2 as usize]._10 = int_to_float((*state).texture_states[i_2 as usize][WINED3D_TSS_BUMPENV_MAT10 as usize]);
            constants.matrices[i_2 as usize]._11 = int_to_float((*state).texture_states[i_2 as usize][WINED3D_TSS_BUMPENV_MAT11 as usize]);
            constants.loffset[i_2 as usize] = int_to_float((*state).texture_states[i_2 as usize][WINED3D_TSS_BUMPENV_LOFFSET as usize]);
            constants.lscale[i_2 as usize] = int_to_float((*state).texture_states[i_2 as usize][WINED3D_TSS_BUMPENV_LSCALE as usize]);
            i_2 = i_2.wrapping_add(1);
        }

        wined3d_device_context_push_constants(context, WINED3D_PUSH_CONSTANTS_PS_FFP as ::core::ffi::c_uint, WINED3D_SHADER_CONST_PS_BUMP_ENV as uint32_t,
                core::mem::offset_of!(wined3d_ffp_ps_constants, bumpenv) as ::core::ffi::c_uint,
                ::core::mem::size_of::<wined3d_ffp_bumpenv_constants>() as ::core::ffi::c_uint,
                core::ptr::addr_of!(constants) as *const ::core::ffi::c_void);
    }

    if (*changed).ffp_ps_constants() != 0 {
        static specular_enabled: wined3d_color = wined3d_color { r: 1.0f32, g: 1.0f32, b: 1.0f32, a: 0.0f32 };
        static specular_disabled: wined3d_color = wined3d_color { r: 0.0f32, g: 0.0f32, b: 0.0f32, a: 0.0f32 };
        let mut constants: wined3d_ffp_ps_constants = core::mem::zeroed();

        i = 0;
        while i < WINED3D_MAX_FFP_TEXTURES {
            wined3d_color_from_d3dcolor(
                    (core::ptr::addr_of_mut!(constants.texture_constants) as *mut wined3d_color).offset(i as isize),
                    (*state).texture_states[i as usize][WINED3D_TSS_CONSTANT as usize]);
            i = i.wrapping_add(1);
        }

        wined3d_color_from_d3dcolor(core::ptr::addr_of_mut!(constants.texture_factor), (*state).rs[WINED3D_RS_TEXTUREFACTOR as usize]);

        constants.specular_enable = if (*state).rs[WINED3D_RS_SPECULARENABLE as usize] != 0 {
            specular_enabled
        } else {
            specular_disabled
        };

        wined3d_device_context_push_constants(context, WINED3D_PUSH_CONSTANTS_PS_FFP as ::core::ffi::c_uint,
                WINED3D_SHADER_CONST_FFP_PS as uint32_t, 0 as uint32_t,
                core::mem::offset_of!(wined3d_ffp_ps_constants, color_key) as ::core::ffi::c_uint,
                core::ptr::addr_of!(constants) as *const ::core::ffi::c_void);
    }

    if wined3d_bitmap_is_set(core::ptr::addr_of_mut!((*changed).renderState) as *mut uint32_t, WINED3D_RS_ALPHAREF) != 0 {
        let f: f32 = ((*state).rs[WINED3D_RS_ALPHAREF as usize] & (0xff as ::core::ffi::c_uint)) as f32 / 255.0f32;

        wined3d_device_context_push_constants(context,
                WINED3D_PUSH_CONSTANTS_PS_FFP as ::core::ffi::c_uint, WINED3D_SHADER_CONST_PS_ALPHA_TEST as uint32_t,
                core::mem::offset_of!(wined3d_ffp_ps_constants, alpha_test_ref) as ::core::ffi::c_uint,
                ::core::mem::size_of::<f32>() as ::core::ffi::c_uint,
                core::ptr::addr_of!(f) as *const ::core::ffi::c_void);
    }

    if (*changed).vertexShader() != 0 {
        wined3d_device_context_set_shader(context, WINED3D_SHADER_TYPE_VERTEX as wined3d_shader_type, (*state).vs as *mut wined3d_shader);
        bind_push_constant_buffer(device, WINED3D_PUSH_CONSTANTS_VS_F as wined3d_push_constants,
                WINED3D_SHADER_TYPE_VERTEX as wined3d_shader_type, VKD3D_SHADER_D3DBC_FLOAT_CONSTANT_REGISTER as ::core::ffi::c_uint);
        bind_push_constant_buffer(device, WINED3D_PUSH_CONSTANTS_VS_I as wined3d_push_constants,
                WINED3D_SHADER_TYPE_VERTEX as wined3d_shader_type, VKD3D_SHADER_D3DBC_INT_CONSTANT_REGISTER as ::core::ffi::c_uint);
        bind_push_constant_buffer(device, WINED3D_PUSH_CONSTANTS_VS_B as wined3d_push_constants,
                WINED3D_SHADER_TYPE_VERTEX as wined3d_shader_type, VKD3D_SHADER_D3DBC_BOOL_CONSTANT_REGISTER as ::core::ffi::c_uint);
    }
    if (*changed).pixelShader() != 0 {
        wined3d_device_context_set_shader(context, WINED3D_SHADER_TYPE_PIXEL as wined3d_shader_type, (*state).ps as *mut wined3d_shader);
        bind_push_constant_buffer(device, WINED3D_PUSH_CONSTANTS_PS_F as wined3d_push_constants,
                WINED3D_SHADER_TYPE_PIXEL as wined3d_shader_type, VKD3D_SHADER_D3DBC_FLOAT_CONSTANT_REGISTER as ::core::ffi::c_uint);
        bind_push_constant_buffer(device, WINED3D_PUSH_CONSTANTS_PS_I as wined3d_push_constants,
                WINED3D_SHADER_TYPE_PIXEL as wined3d_shader_type, VKD3D_SHADER_D3DBC_INT_CONSTANT_REGISTER as ::core::ffi::c_uint);
        bind_push_constant_buffer(device, WINED3D_PUSH_CONSTANTS_PS_B as wined3d_push_constants,
                WINED3D_SHADER_TYPE_PIXEL as wined3d_shader_type, VKD3D_SHADER_D3DBC_BOOL_CONSTANT_REGISTER as ::core::ffi::c_uint);
    }

    /* XXX: We don't invalidate HLSL shaders for every field contained in
     * wined3d_ffp_vs_settings / ffp_frag_settings; only the ones that the HLSL
     * FFP pipeline cares about. The rest should eventually be removed from
     * those structs and left only in vs_compile_args / ps_compile_args. */

    if (*changed).ffp_vs_settings() != 0
        && ((*state).vs.is_null()
            || (*state).vertex_declaration.is_null()
            || (*(*state).vertex_declaration).position_transformed != false)
    {
        if (*(*device).adapter).d3d_info.ffp_hlsl() != 0 {
            let shader: *mut wined3d_shader = get_ffp_vertex_shader(device, (*(*device).cs).c.state);

            wined3d_device_context_set_shader(context, WINED3D_SHADER_TYPE_VERTEX as wined3d_shader_type, shader);
            bind_push_constant_buffer(device, WINED3D_PUSH_CONSTANTS_VS_FFP as wined3d_push_constants,
                    WINED3D_SHADER_TYPE_VERTEX as wined3d_shader_type, VKD3D_SHADER_D3DBC_FLOAT_CONSTANT_REGISTER as ::core::ffi::c_uint);
        } else {
            /* Force invalidation of the vertex shader. */
            wined3d_device_context_emit_set_shader(context, WINED3D_SHADER_TYPE_VERTEX as wined3d_shader_type, core::ptr::null_mut());
        }
    }

    if (*changed).ffp_ps_settings() != 0 && (*state).ps.is_null() {
        if (*(*device).adapter).d3d_info.ffp_hlsl() != 0 {
            let shader_0: *mut wined3d_shader = get_ffp_pixel_shader(device, (*(*device).cs).c.state);

            wined3d_device_context_set_shader(context, WINED3D_SHADER_TYPE_PIXEL as wined3d_shader_type, shader_0);
            bind_push_constant_buffer(device, WINED3D_PUSH_CONSTANTS_PS_FFP as wined3d_push_constants,
                    WINED3D_SHADER_TYPE_PIXEL as wined3d_shader_type, VKD3D_SHADER_D3DBC_FLOAT_CONSTANT_REGISTER as ::core::ffi::c_uint);
        } else {
            /* Force invalidation of the pixel shader. */
            wined3d_device_context_emit_set_shader(context, WINED3D_SHADER_TYPE_PIXEL as wined3d_shader_type, core::ptr::null_mut());
        }
    }

    bind_push_constant_buffer(device, WINED3D_PUSH_CONSTANTS_PS_FFP as wined3d_push_constants,
            WINED3D_SHADER_TYPE_PIXEL as wined3d_shader_type, WINED3D_FFP_CONSTANTS_EXTRA_REGISTER as ::core::ffi::c_uint);

    assert!(list_empty(core::ptr::addr_of_mut!((*stateblock).changed.changed_lights)) != 0);
    memset(core::ptr::addr_of_mut!((*stateblock).changed) as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int, ::core::mem::size_of::<wined3d_saved_states>() as usize);
    list_init(core::ptr::addr_of_mut!((*stateblock).changed.changed_lights));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_set_texture_lod(
    mut stateblock: *mut wined3d_stateblock,
    mut texture: *mut wined3d_texture,
    mut lod: ::core::ffi::c_uint,
) -> ::core::ffi::c_uint {
    let mut old: ::core::ffi::c_uint = 0;
    old = wined3d_texture_set_lod(texture, lod);
    if old != lod {
        let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
        while i < WINED3D_MAX_COMBINED_SAMPLERS as ::core::ffi::c_uint {
            if (*stateblock).stateblock_state.textures[i as usize] == texture {
                (*stateblock).changed.set_textures(
                    (*stateblock).changed.textures()
                        | ((1 as ::core::ffi::c_uint) << i) as uint32_t,
                );
            }
            i = i.wrapping_add(1);
        }
    }
    return old;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_texture_changed(
    mut stateblock: *mut wined3d_stateblock,
    mut texture: *const wined3d_texture,
) {
    let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    while i < WINED3D_MAX_COMBINED_SAMPLERS as ::core::ffi::c_uint {
        if (*stateblock).stateblock_state.textures[i as usize] == texture as *mut wined3d_texture {
            (*stateblock).changed.set_textures(
                (*stateblock).changed.textures()
                    | ((1 as ::core::ffi::c_uint) << i) as uint32_t,
            );
        }
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stateblock_depth_buffer_changed(
    mut stateblock: *mut wined3d_stateblock,
) {
    let mut decl: *mut wined3d_vertex_declaration =
        (*stateblock).stateblock_state.vertex_declaration as *mut wined3d_vertex_declaration;
    if !decl.is_null() && (*decl).position_transformed as ::core::ffi::c_int != 0 {
        (*stateblock)
            .changed
            .set_position_transformed(1);
    }
}
