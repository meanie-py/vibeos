use crate::d3dwine::context_gl::context_create_wgl_attribs;
use crate::d3dwine::context_gl::context_gl_resource_released;
use crate::d3dwine::utils::debug_glerror;
use crate::d3dwine::context_gl::dispatch_compute;
use crate::d3dwine::context_gl::draw_primitive;
use crate::d3dwine::gl_compat::install_gl_compat_wrapper;
use crate::d3dwine::directx::wined3d_adapter_cleanup;
use crate::d3dwine::utils::wined3d_adapter_gl_init_format_info;
use crate::d3dwine::directx::wined3d_adapter_init;
use crate::d3dwine::buffer::wined3d_buffer_cleanup;
use crate::d3dwine::buffer::wined3d_buffer_gl_binding_from_bind_flags;
use crate::d3dwine::buffer::wined3d_buffer_gl_init;
use crate::d3dwine::utils::wined3d_caps_gl_ctx_test_filling_convention;
use crate::d3dwine::utils::wined3d_caps_gl_ctx_test_viewport_subpixel_bits;
use crate::d3dwine::context_gl::wined3d_context_gl_acquire;
use crate::d3dwine::context_gl::wined3d_context_gl_copy_bo_address;
use crate::d3dwine::context_gl::wined3d_context_gl_destroy_bo;
use crate::d3dwine::context_gl::wined3d_context_gl_flush_bo_address;
use crate::d3dwine::context_gl::wined3d_context_gl_get_current;
use crate::d3dwine::context_gl::wined3d_context_gl_map_bo_address;
use crate::d3dwine::context_gl::wined3d_context_gl_release;
use crate::d3dwine::context_gl::wined3d_context_gl_unmap_bo_address;
use crate::d3dwine::cs::wined3d_cs_destroy_object;
use crate::d3dwine::cs::wined3d_cs_init_object;
use crate::d3dwine::cs::wined3d_cs_map_bo_address;
use crate::d3dwine::utils::wined3d_debug_view_desc;
use crate::d3dwine::device::wined3d_device_cleanup;
use crate::d3dwine::device::wined3d_device_decref;
use crate::d3dwine::device::wined3d_device_destroy_default_samplers;
use crate::d3dwine::device::wined3d_device_gl_create_bo;
use crate::d3dwine::device::wined3d_device_gl_create_primary_opengl_context_cs;
use crate::d3dwine::device::wined3d_device_gl_delete_opengl_contexts_cs;
use crate::d3dwine::device::wined3d_device_incref;
use crate::d3dwine::device::wined3d_device_init;
use crate::d3dwine::directx::wined3d_driver_info_init;
use crate::d3dwine::directx::wined3d_get_gpu_description;
use crate::d3dwine::utils::wined3d_get_primary_adapter_luid;
use crate::d3dwine::directx::wined3d_get_user_override_gpu_description;
use crate::d3dwine::directx::wined3d_gpu_from_feature_level;
use crate::d3dwine::query::wined3d_query_gl_create;
use crate::d3dwine::query::wined3d_query_gl_destroy_buffer_object;
use crate::d3dwine::utils::wined3d_release_dc;
use crate::d3dwine::view::wined3d_rendertarget_view_cleanup;
use crate::d3dwine::view::wined3d_rendertarget_view_gl_init;
use crate::d3dwine::resource::wined3d_resource_gl_storage_flags;
use crate::d3dwine::sampler::wined3d_sampler_gl_init;
use crate::d3dwine::view::wined3d_shader_resource_view_cleanup;
use crate::d3dwine::view::wined3d_shader_resource_view_gl_generate_mipmap;
use crate::d3dwine::view::wined3d_shader_resource_view_gl_init;
use crate::d3dwine::swapchain::wined3d_swapchain_gl_cleanup;
use crate::d3dwine::swapchain::wined3d_swapchain_gl_init;
use crate::d3dwine::texture::wined3d_texture_cleanup;
use crate::d3dwine::texture::wined3d_texture_gl_init;
use crate::d3dwine::texture::wined3d_texture_sub_resources_destroyed;
use crate::d3dwine::view::wined3d_unordered_access_view_cleanup;
use crate::d3dwine::view::wined3d_unordered_access_view_gl_clear;
use crate::d3dwine::view::wined3d_unordered_access_view_gl_init;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_53 {
    pub extension: wined3d_gl_extension,
    pub min_gl_version: DWORD,
}

pub static RUSTCONV_WSTR_0D54D2A9857B: [u16; 162] = [
    0x0052u16, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x0020, 0x0044, 0x0069,
    0x0072, 0x0065, 0x0063, 0x0074, 0x0058, 0x0020, 0x004E, 0x0065, 0x0065, 0x0064,
    0x0073, 0x0020, 0x0061, 0x0074, 0x0020, 0x006C, 0x0065, 0x0061, 0x0073, 0x0074,
    0x0020, 0x004F, 0x0070, 0x0065, 0x006E, 0x0047, 0x004C, 0x0020, 0x0032, 0x002E,
    0x0030, 0x0020, 0x006F, 0x0072, 0x0020, 0x0062, 0x0065, 0x0074, 0x0074, 0x0065,
    0x0072, 0x000A, 0x000A, 0x0049, 0x006E, 0x0073, 0x0074, 0x0061, 0x006C, 0x006C,
    0x0020, 0x0061, 0x0020, 0x0064, 0x0069, 0x0073, 0x0070, 0x006C, 0x0061, 0x0079,
    0x0020, 0x0064, 0x0072, 0x0069, 0x0076, 0x0065, 0x0072, 0x0020, 0x0074, 0x0068,
    0x0061, 0x0074, 0x0020, 0x0070, 0x0072, 0x006F, 0x0076, 0x0069, 0x0064, 0x0065,
    0x0073, 0x0020, 0x0061, 0x006E, 0x0020, 0x004F, 0x0070, 0x0065, 0x006E, 0x0047,
    0x004C, 0x0020, 0x0049, 0x0043, 0x0044, 0x0020, 0x0074, 0x006F, 0x0020, 0x0065,
    0x006E, 0x0061, 0x0062, 0x006C, 0x0065, 0x0020, 0x0033, 0x0044, 0x0020, 0x0061,
    0x0063, 0x0063, 0x0065, 0x006C, 0x0065, 0x0072, 0x0061, 0x0074, 0x0069, 0x006F,
    0x006E, 0x002E, 0x0020, 0x004F, 0x0072, 0x0020, 0x0069, 0x006E, 0x0073, 0x0074,
    0x0061, 0x006C, 0x006C, 0x0020, 0x004D, 0x0065, 0x0073, 0x0061, 0x0033, 0x0044,
    0x0020, 0x0066, 0x0072, 0x006F, 0x006D, 0x0020, 0x0052, 0x0041, 0x0050, 0x0050,
    0x0053, 0x0000,
];
pub static RUSTCONV_WSTR_C1AB1E21E959: [u16; 29] = [
    0x0052u16, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x0020, 0x002D, 0x0020,
    0x006E, 0x006F, 0x0020, 0x0033, 0x0044, 0x0020, 0x0061, 0x0063, 0x0063, 0x0065,
    0x006C, 0x0065, 0x0072, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000,
];

use core::ffi::c_void;

extern "system" {
    fn wglGetProcAddress(lpszProc: LPCSTR) -> PROC;
    fn MessageBoxW(hWnd: HWND, lpText: *const u16, lpCaption: *const u16, uType: u32) -> i32;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_52 {
    pub card_vendor: wined3d_pci_vendor,
    pub description: *const ::core::ffi::c_char,
    pub gl_vendor_selection: *const gl_vendor_selection,
    pub gl_vendor_count: ::core::ffi::c_uint,
}

static mut gl_extension_map: [wined3d_extension_map; 179] = [
    wined3d_extension_map {
        extension_string: b"GL_APPLE_fence\0" as *const u8 as *const ::core::ffi::c_char,
        extension: APPLE_FENCE,
    },
    wined3d_extension_map {
        extension_string: b"GL_APPLE_float_pixels\0" as *const u8 as *const ::core::ffi::c_char,
        extension: APPLE_FLOAT_PIXELS,
    },
    wined3d_extension_map {
        extension_string: b"GL_APPLE_flush_buffer_range\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: APPLE_FLUSH_BUFFER_RANGE,
    },
    wined3d_extension_map {
        extension_string: b"GL_APPLE_flush_render\0" as *const u8 as *const ::core::ffi::c_char,
        extension: APPLE_FLUSH_RENDER,
    },
    wined3d_extension_map {
        extension_string: b"GL_APPLE_rgb_422\0" as *const u8 as *const ::core::ffi::c_char,
        extension: APPLE_RGB_422,
    },
    wined3d_extension_map {
        extension_string: b"GL_APPLE_ycbcr_422\0" as *const u8 as *const ::core::ffi::c_char,
        extension: APPLE_YCBCR_422,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_base_instance\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_BASE_INSTANCE,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_blend_func_extended\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_BLEND_FUNC_EXTENDED,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_buffer_storage\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_BUFFER_STORAGE,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_clear_buffer_object\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_CLEAR_BUFFER_OBJECT,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_clear_texture\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_CLEAR_TEXTURE,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_clip_control\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_CLIP_CONTROL,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_color_buffer_float\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_COLOR_BUFFER_FLOAT,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_compute_shader\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_COMPUTE_SHADER,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_conservative_depth\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_CONSERVATIVE_DEPTH,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_copy_buffer\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_COPY_BUFFER,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_copy_image\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_COPY_IMAGE,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_cull_distance\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_CULL_DISTANCE,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_debug_output\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_DEBUG_OUTPUT,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_depth_buffer_float\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_DEPTH_BUFFER_FLOAT,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_depth_clamp\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_DEPTH_CLAMP,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_depth_texture\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_DEPTH_TEXTURE,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_derivative_control\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_DERIVATIVE_CONTROL,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_draw_buffers\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_DRAW_BUFFERS,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_draw_buffers_blend\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_DRAW_BUFFERS_BLEND,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_draw_elements_base_vertex\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_DRAW_ELEMENTS_BASE_VERTEX,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_draw_indirect\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_DRAW_INDIRECT,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_draw_instanced\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_DRAW_INSTANCED,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_ES2_compatibility\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_ES2_COMPATIBILITY,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_ES3_compatibility\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_ES3_COMPATIBILITY,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_explicit_attrib_location\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_EXPLICIT_ATTRIB_LOCATION,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_fragment_coord_conventions\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_FRAGMENT_COORD_CONVENTIONS,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_fragment_layer_viewport\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_FRAGMENT_LAYER_VIEWPORT,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_fragment_program\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_FRAGMENT_PROGRAM,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_fragment_program_shadow\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_FRAGMENT_PROGRAM_SHADOW,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_fragment_shader\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_FRAGMENT_SHADER,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_framebuffer_no_attachments\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_FRAMEBUFFER_NO_ATTACHMENTS,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_framebuffer_object\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_FRAMEBUFFER_OBJECT,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_framebuffer_sRGB\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_FRAMEBUFFER_SRGB,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_geometry_shader4\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_GEOMETRY_SHADER4,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_gpu_shader5\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_GPU_SHADER5,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_half_float_pixel\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_HALF_FLOAT_PIXEL,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_half_float_vertex\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_HALF_FLOAT_VERTEX,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_instanced_arrays\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_INSTANCED_ARRAYS,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_internalformat_query\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_INTERNALFORMAT_QUERY,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_internalformat_query2\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_INTERNALFORMAT_QUERY2,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_map_buffer_alignment\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_MAP_BUFFER_ALIGNMENT,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_map_buffer_range\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_MAP_BUFFER_RANGE,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_multisample\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_MULTISAMPLE,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_multitexture\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_MULTITEXTURE,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_occlusion_query\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_OCCLUSION_QUERY,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_pipeline_statistics_query\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_PIPELINE_STATISTICS_QUERY,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_pixel_buffer_object\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_PIXEL_BUFFER_OBJECT,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_point_parameters\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_POINT_PARAMETERS,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_point_sprite\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_POINT_SPRITE,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_polygon_offset_clamp\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_POLYGON_OFFSET_CLAMP,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_provoking_vertex\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_PROVOKING_VERTEX,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_query_buffer_object\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_QUERY_BUFFER_OBJECT,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_sample_shading\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_SAMPLE_SHADING,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_sampler_objects\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_SAMPLER_OBJECTS,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_seamless_cube_map\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_SEAMLESS_CUBE_MAP,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_shader_atomic_counters\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_SHADER_ATOMIC_COUNTERS,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_shader_bit_encoding\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_SHADER_BIT_ENCODING,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_shader_image_load_store\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_SHADER_IMAGE_LOAD_STORE,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_shader_image_size\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_SHADER_IMAGE_SIZE,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_shader_stencil_export\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_SHADER_STENCIL_EXPORT,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_shader_storage_buffer_object\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_SHADER_STORAGE_BUFFER_OBJECT,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_shader_texture_image_samples\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_SHADER_TEXTURE_IMAGE_SAMPLES,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_shader_texture_lod\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_SHADER_TEXTURE_LOD,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_shader_viewport_layer_array\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_SHADER_VIEWPORT_LAYER_ARRAY,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_shading_language_100\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_SHADING_LANGUAGE_100,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_shading_language_420pack\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_SHADING_LANGUAGE_420PACK,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_shading_language_packing\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_SHADING_LANGUAGE_PACKING,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_shadow\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_SHADOW,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_stencil_texturing\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_STENCIL_TEXTURING,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_sync\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_SYNC,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_tessellation_shader\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_TESSELLATION_SHADER,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_border_clamp\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_BORDER_CLAMP,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_buffer_object\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_BUFFER_OBJECT,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_buffer_range\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_BUFFER_RANGE,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_compression\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_COMPRESSION,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_compression_bptc\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_COMPRESSION_BPTC,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_compression_rgtc\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_COMPRESSION_RGTC,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_cube_map\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_CUBE_MAP,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_cube_map_array\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_CUBE_MAP_ARRAY,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_env_combine\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_ENV_COMBINE,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_env_dot3\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_ENV_DOT3,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_filter_anisotropic\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_FILTER_ANISOTROPIC,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_float\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_FLOAT,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_gather\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_GATHER,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_mirrored_repeat\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_MIRRORED_REPEAT,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_mirror_clamp_to_edge\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_MIRROR_CLAMP_TO_EDGE,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_multisample\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_MULTISAMPLE,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_non_power_of_two\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_NON_POWER_OF_TWO,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_query_levels\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_QUERY_LEVELS,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_rg\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_RG,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_rgb10_a2ui\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_RGB10_A2UI,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_storage\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_STORAGE,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_storage_multisample\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_STORAGE_MULTISAMPLE,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_swizzle\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_SWIZZLE,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_view\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_VIEW,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_timer_query\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_TIMER_QUERY,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_transform_feedback2\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_TRANSFORM_FEEDBACK2,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_transform_feedback3\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_TRANSFORM_FEEDBACK3,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_uniform_buffer_object\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_UNIFORM_BUFFER_OBJECT,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_vertex_array_bgra\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_VERTEX_ARRAY_BGRA,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_vertex_buffer_object\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_VERTEX_BUFFER_OBJECT,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_vertex_program\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_VERTEX_PROGRAM,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_vertex_shader\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_VERTEX_SHADER,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_vertex_type_10f_11f_11f_rev\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_VERTEX_TYPE_10F_11F_11F_REV,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_vertex_type_2_10_10_10_rev\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_VERTEX_TYPE_2_10_10_10_REV,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_viewport_array\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_VIEWPORT_ARRAY,
    },
    wined3d_extension_map {
        extension_string: b"GL_ARB_texture_barrier\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_BARRIER,
    },
    wined3d_extension_map {
        extension_string: b"GL_ATI_fragment_shader\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ATI_FRAGMENT_SHADER,
    },
    wined3d_extension_map {
        extension_string: b"GL_ATI_separate_stencil\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ATI_SEPARATE_STENCIL,
    },
    wined3d_extension_map {
        extension_string: b"GL_ATI_texture_compression_3dc\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ATI_TEXTURE_COMPRESSION_3DC,
    },
    wined3d_extension_map {
        extension_string: b"GL_ATI_texture_env_combine3\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ATI_TEXTURE_ENV_COMBINE3,
    },
    wined3d_extension_map {
        extension_string: b"GL_ATI_texture_mirror_once\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ATI_TEXTURE_MIRROR_ONCE,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_blend_color\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_BLEND_COLOR,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_blend_equation_separate\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: EXT_BLEND_EQUATION_SEPARATE,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_blend_func_separate\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: EXT_BLEND_FUNC_SEPARATE,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_blend_minmax\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_BLEND_MINMAX,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_blend_subtract\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_BLEND_SUBTRACT,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_depth_bounds_test\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_DEPTH_BOUNDS_TEST,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_draw_buffers2\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_DRAW_BUFFERS2,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_fog_coord\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_FOG_COORD,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_framebuffer_blit\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_FRAMEBUFFER_BLIT,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_framebuffer_multisample\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: EXT_FRAMEBUFFER_MULTISAMPLE,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_framebuffer_multisample_blit_scaled\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: EXT_FRAMEBUFFER_MULTISAMPLE_BLIT_SCALED,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_framebuffer_object\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_FRAMEBUFFER_OBJECT,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_memory_object\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_MEMORY_OBJECT,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_gpu_program_parameters\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: EXT_GPU_PROGRAM_PARAMETERS,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_gpu_shader4\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_GPU_SHADER4,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_packed_depth_stencil\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: EXT_PACKED_DEPTH_STENCIL,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_packed_float\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_PACKED_FLOAT,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_point_parameters\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_POINT_PARAMETERS,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_polygon_offset_clamp\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_POLYGON_OFFSET_CLAMP,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_provoking_vertex\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_PROVOKING_VERTEX,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_secondary_color\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_SECONDARY_COLOR,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_shader_integer_mix\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_SHADER_INTEGER_MIX,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_stencil_two_side\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_STENCIL_TWO_SIDE,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_stencil_wrap\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_STENCIL_WRAP,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_texture3D\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_TEXTURE3D,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_texture_array\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_TEXTURE_ARRAY,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_texture_compression_rgtc\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: EXT_TEXTURE_COMPRESSION_RGTC,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_texture_compression_s3tc\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: EXT_TEXTURE_COMPRESSION_S3TC,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_texture_env_combine\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: EXT_TEXTURE_ENV_COMBINE,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_texture_env_dot3\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_TEXTURE_ENV_DOT3,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_texture_filter_anisotropic\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_FILTER_ANISOTROPIC,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_texture_integer\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_TEXTURE_INTEGER,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_texture_lod_bias\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_TEXTURE_LOD_BIAS,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_texture_mirror_clamp\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: EXT_TEXTURE_MIRROR_CLAMP,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_texture_shadow_lod\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_TEXTURE_SHADOW_LOD,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_texture_shared_exponent\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: EXT_TEXTURE_SHARED_EXPONENT,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_texture_snorm\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_TEXTURE_SNORM,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_texture_sRGB\0" as *const u8 as *const ::core::ffi::c_char,
        extension: EXT_TEXTURE_SRGB,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_texture_sRGB_decode\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: EXT_TEXTURE_SRGB_DECODE,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_texture_swizzle\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_TEXTURE_SWIZZLE,
    },
    wined3d_extension_map {
        extension_string: b"GL_EXT_vertex_array_bgra\0" as *const u8 as *const ::core::ffi::c_char,
        extension: ARB_VERTEX_ARRAY_BGRA,
    },
    wined3d_extension_map {
        extension_string: b"GL_NV_fence\0" as *const u8 as *const ::core::ffi::c_char,
        extension: NV_FENCE,
    },
    wined3d_extension_map {
        extension_string: b"GL_NV_fog_distance\0" as *const u8 as *const ::core::ffi::c_char,
        extension: NV_FOG_DISTANCE,
    },
    wined3d_extension_map {
        extension_string: b"GL_NV_fragment_program\0" as *const u8 as *const ::core::ffi::c_char,
        extension: NV_FRAGMENT_PROGRAM,
    },
    wined3d_extension_map {
        extension_string: b"GL_NV_fragment_program2\0" as *const u8 as *const ::core::ffi::c_char,
        extension: NV_FRAGMENT_PROGRAM2,
    },
    wined3d_extension_map {
        extension_string: b"GL_NV_fragment_program_option\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: NV_FRAGMENT_PROGRAM_OPTION,
    },
    wined3d_extension_map {
        extension_string: b"GL_NV_half_float\0" as *const u8 as *const ::core::ffi::c_char,
        extension: NV_HALF_FLOAT,
    },
    wined3d_extension_map {
        extension_string: b"GL_NV_light_max_exponent\0" as *const u8 as *const ::core::ffi::c_char,
        extension: NV_LIGHT_MAX_EXPONENT,
    },
    wined3d_extension_map {
        extension_string: b"GL_NV_point_sprite\0" as *const u8 as *const ::core::ffi::c_char,
        extension: NV_POINT_SPRITE,
    },
    wined3d_extension_map {
        extension_string: b"GL_NV_register_combiners\0" as *const u8 as *const ::core::ffi::c_char,
        extension: NV_REGISTER_COMBINERS,
    },
    wined3d_extension_map {
        extension_string: b"GL_NV_register_combiners2\0" as *const u8 as *const ::core::ffi::c_char,
        extension: NV_REGISTER_COMBINERS2,
    },
    wined3d_extension_map {
        extension_string: b"GL_NV_texgen_reflection\0" as *const u8 as *const ::core::ffi::c_char,
        extension: NV_TEXGEN_REFLECTION,
    },
    wined3d_extension_map {
        extension_string: b"GL_NV_texture_env_combine4\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: NV_TEXTURE_ENV_COMBINE4,
    },
    wined3d_extension_map {
        extension_string: b"GL_NV_texture_shader\0" as *const u8 as *const ::core::ffi::c_char,
        extension: NV_TEXTURE_SHADER,
    },
    wined3d_extension_map {
        extension_string: b"GL_NV_texture_shader2\0" as *const u8 as *const ::core::ffi::c_char,
        extension: NV_TEXTURE_SHADER2,
    },
    wined3d_extension_map {
        extension_string: b"GL_NV_vertex_program\0" as *const u8 as *const ::core::ffi::c_char,
        extension: NV_VERTEX_PROGRAM,
    },
    wined3d_extension_map {
        extension_string: b"GL_NV_vertex_program1_1\0" as *const u8 as *const ::core::ffi::c_char,
        extension: NV_VERTEX_PROGRAM1_1,
    },
    wined3d_extension_map {
        extension_string: b"GL_NV_vertex_program2\0" as *const u8 as *const ::core::ffi::c_char,
        extension: NV_VERTEX_PROGRAM2,
    },
    wined3d_extension_map {
        extension_string: b"GL_NV_vertex_program2_option\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: NV_VERTEX_PROGRAM2_OPTION,
    },
    wined3d_extension_map {
        extension_string: b"GL_NV_vertex_program3\0" as *const u8 as *const ::core::ffi::c_char,
        extension: NV_VERTEX_PROGRAM3,
    },
    wined3d_extension_map {
        extension_string: b"GL_NV_texture_barrier\0" as *const u8 as *const ::core::ffi::c_char,
        extension: NV_TEXTURE_BARRIER,
    },
];

static mut wgl_extension_map: [wined3d_extension_map; 4] = [
    wined3d_extension_map {
        extension_string: b"WGL_ARB_pixel_format\0" as *const u8 as *const ::core::ffi::c_char,
        extension: WGL_ARB_PIXEL_FORMAT,
    },
    wined3d_extension_map {
        extension_string: b"WGL_EXT_swap_control\0" as *const u8 as *const ::core::ffi::c_char,
        extension: WGL_EXT_SWAP_CONTROL,
    },
    wined3d_extension_map {
        extension_string: b"WGL_WINE_pixel_format_passthrough\0" as *const u8
            as *const ::core::ffi::c_char,
        extension: WGL_WINE_PIXEL_FORMAT_PASSTHROUGH,
    },
    wined3d_extension_map {
        extension_string: b"WGL_WINE_query_renderer\0" as *const u8 as *const ::core::ffi::c_char,
        extension: WGL_WINE_QUERY_RENDERER,
    },
];

static mut cards_intel: [wined3d_renderer_table; 49] = [
    wined3d_renderer_table {
        renderer: b"Iris Pro Graphics P580\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_IPP580_1,
    },
    wined3d_renderer_table {
        renderer: b"Skylake\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_HD520_1,
    },
    wined3d_renderer_table {
        renderer: b"Iris Pro P6300\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_IPP6300,
    },
    wined3d_renderer_table {
        renderer: b"Iris Pro 6200\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_IP6200,
    },
    wined3d_renderer_table {
        renderer: b"Iris 6100\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_I6100,
    },
    wined3d_renderer_table {
        renderer: b"Iris(TM) Graphics 6100\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_I6100,
    },
    wined3d_renderer_table {
        renderer: b"Iris Pro 5200\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_IP5200_1,
    },
    wined3d_renderer_table {
        renderer: b"Iris 5100\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_I5100_1,
    },
    wined3d_renderer_table {
        renderer: b"HD Graphics 5000\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_HD5000_1,
    },
    wined3d_renderer_table {
        renderer: b"Haswell Mobile\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_HWM,
    },
    wined3d_renderer_table {
        renderer: b"Iris OpenGL Engine\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_HWM,
    },
    wined3d_renderer_table {
        renderer: b"Ivybridge Server\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_IVBS,
    },
    wined3d_renderer_table {
        renderer: b"Ivybridge Mobile\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_IVBM,
    },
    wined3d_renderer_table {
        renderer: b"Ivybridge Desktop\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_IVBD,
    },
    wined3d_renderer_table {
        renderer: b"HD Graphics 4000\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_IVBD,
    },
    wined3d_renderer_table {
        renderer: b"Sandybridge Server\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_SNBS,
    },
    wined3d_renderer_table {
        renderer: b"Sandybridge Mobile\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_SNBM,
    },
    wined3d_renderer_table {
        renderer: b"Sandybridge Desktop\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_SNBD,
    },
    wined3d_renderer_table {
        renderer: b"Ironlake Mobile\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_ILKM,
    },
    wined3d_renderer_table {
        renderer: b"Ironlake Desktop\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_ILKD,
    },
    wined3d_renderer_table {
        renderer: b"B43\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_B43,
    },
    wined3d_renderer_table {
        renderer: b"G41\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_G41,
    },
    wined3d_renderer_table {
        renderer: b"G45\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_G45,
    },
    wined3d_renderer_table {
        renderer: b"Q45\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_Q45,
    },
    wined3d_renderer_table {
        renderer: b"Integrated Graphics Device\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_IGD,
    },
    wined3d_renderer_table {
        renderer: b"GM45\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_GM45,
    },
    wined3d_renderer_table {
        renderer: b"965GME\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_965GME,
    },
    wined3d_renderer_table {
        renderer: b"965GM\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_965GM,
    },
    wined3d_renderer_table {
        renderer: b"X3100\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_965GM,
    },
    wined3d_renderer_table {
        renderer: b"946GZ\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_946GZ,
    },
    wined3d_renderer_table {
        renderer: b"965G\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_965G,
    },
    wined3d_renderer_table {
        renderer: b"965Q\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_965Q,
    },
    wined3d_renderer_table {
        renderer: b"Pineview M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_PNVM,
    },
    wined3d_renderer_table {
        renderer: b"Pineview G\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_PNVG,
    },
    wined3d_renderer_table {
        renderer: b"IGD\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_PNVG,
    },
    wined3d_renderer_table {
        renderer: b"Q33\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_Q33,
    },
    wined3d_renderer_table {
        renderer: b"G33\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_G33,
    },
    wined3d_renderer_table {
        renderer: b"Q35\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_Q35,
    },
    wined3d_renderer_table {
        renderer: b"945GME\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_945GME,
    },
    wined3d_renderer_table {
        renderer: b"945GM\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_945GM,
    },
    wined3d_renderer_table {
        renderer: b"GMA 950\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_945GM,
    },
    wined3d_renderer_table {
        renderer: b"945G\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_945G,
    },
    wined3d_renderer_table {
        renderer: b"915GM\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_915GM,
    },
    wined3d_renderer_table {
        renderer: b"E7221G\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_E7221G,
    },
    wined3d_renderer_table {
        renderer: b"915G\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_915G,
    },
    wined3d_renderer_table {
        renderer: b"865G\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_865G,
    },
    wined3d_renderer_table {
        renderer: b"845G\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_845G,
    },
    wined3d_renderer_table {
        renderer: b"855GM\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_855GM,
    },
    wined3d_renderer_table {
        renderer: b"830M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_INTEL_830M,
    },
];

static mut cards_amd_binary: [wined3d_renderer_table; 79] = [
    wined3d_renderer_table {
        renderer: b"RX 480\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_RX_480,
    },
    wined3d_renderer_table {
        renderer: b"RX 460\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_RX_460,
    },
    wined3d_renderer_table {
        renderer: b"R9 Fury Series\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_R9_FURY,
    },
    wined3d_renderer_table {
        renderer: b"HD 7900\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD7900,
    },
    wined3d_renderer_table {
        renderer: b"HD 7800\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD7800,
    },
    wined3d_renderer_table {
        renderer: b"HD 7700\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD7700,
    },
    wined3d_renderer_table {
        renderer: b"HD 6970\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6900,
    },
    wined3d_renderer_table {
        renderer: b"HD 6900\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6900,
    },
    wined3d_renderer_table {
        renderer: b"HD 6800\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6800,
    },
    wined3d_renderer_table {
        renderer: b"HD 6770M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6600M,
    },
    wined3d_renderer_table {
        renderer: b"HD 6750M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6600M,
    },
    wined3d_renderer_table {
        renderer: b"HD 6700\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6700,
    },
    wined3d_renderer_table {
        renderer: b"HD 6670\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6600,
    },
    wined3d_renderer_table {
        renderer: b"HD 6630M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6600M,
    },
    wined3d_renderer_table {
        renderer: b"HD 6600M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6600M,
    },
    wined3d_renderer_table {
        renderer: b"HD 6600\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6600,
    },
    wined3d_renderer_table {
        renderer: b"HD 6570\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6600,
    },
    wined3d_renderer_table {
        renderer: b"HD 6500M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6600M,
    },
    wined3d_renderer_table {
        renderer: b"HD 6500\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6600,
    },
    wined3d_renderer_table {
        renderer: b"HD 6480G\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6480G,
    },
    wined3d_renderer_table {
        renderer: b"HD 6400\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6400,
    },
    wined3d_renderer_table {
        renderer: b"HD 6300\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6300,
    },
    wined3d_renderer_table {
        renderer: b"HD 6200\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6300,
    },
    wined3d_renderer_table {
        renderer: b"HD 5870\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD5800,
    },
    wined3d_renderer_table {
        renderer: b"HD 5850\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD5800,
    },
    wined3d_renderer_table {
        renderer: b"HD 5800\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD5800,
    },
    wined3d_renderer_table {
        renderer: b"HD 5770\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD5700,
    },
    wined3d_renderer_table {
        renderer: b"HD 5750\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD5700,
    },
    wined3d_renderer_table {
        renderer: b"HD 5700\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD5700,
    },
    wined3d_renderer_table {
        renderer: b"HD 5670\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD5600,
    },
    wined3d_renderer_table {
        renderer: b"HD 5570\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD5600,
    },
    wined3d_renderer_table {
        renderer: b"HD 5550\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD5600,
    },
    wined3d_renderer_table {
        renderer: b"HD 5450\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD5400,
    },
    wined3d_renderer_table {
        renderer: b"HD 5000\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD5600,
    },
    wined3d_renderer_table {
        renderer: b"HD 4890\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD4800,
    },
    wined3d_renderer_table {
        renderer: b"HD 4870\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD4800,
    },
    wined3d_renderer_table {
        renderer: b"HD 4850\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD4800,
    },
    wined3d_renderer_table {
        renderer: b"HD 4830\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD4800,
    },
    wined3d_renderer_table {
        renderer: b"HD 4800\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD4800,
    },
    wined3d_renderer_table {
        renderer: b"HD 4770\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD4700,
    },
    wined3d_renderer_table {
        renderer: b"HD 4700\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD4700,
    },
    wined3d_renderer_table {
        renderer: b"HD 4670\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD4600,
    },
    wined3d_renderer_table {
        renderer: b"HD 4650\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD4600,
    },
    wined3d_renderer_table {
        renderer: b"HD 4600\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD4600,
    },
    wined3d_renderer_table {
        renderer: b"HD 4550\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD4350,
    },
    wined3d_renderer_table {
        renderer: b"HD 4350\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD4350,
    },
    wined3d_renderer_table {
        renderer: b"HD 4200M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD4200M,
    },
    wined3d_renderer_table {
        renderer: b"HD 3300\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD3200,
    },
    wined3d_renderer_table {
        renderer: b"HD 3200\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD3200,
    },
    wined3d_renderer_table {
        renderer: b"HD 3100\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD3200,
    },
    wined3d_renderer_table {
        renderer: b"HD 3870\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD2900,
    },
    wined3d_renderer_table {
        renderer: b"HD 3850\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD2900,
    },
    wined3d_renderer_table {
        renderer: b"HD 2900\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD2900,
    },
    wined3d_renderer_table {
        renderer: b"HD 3830\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD2600,
    },
    wined3d_renderer_table {
        renderer: b"HD 3690\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD2600,
    },
    wined3d_renderer_table {
        renderer: b"HD 3650\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD2600,
    },
    wined3d_renderer_table {
        renderer: b"HD 2600\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD2600,
    },
    wined3d_renderer_table {
        renderer: b"HD 3470\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD2350,
    },
    wined3d_renderer_table {
        renderer: b"HD 3450\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD2350,
    },
    wined3d_renderer_table {
        renderer: b"HD 3430\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD2350,
    },
    wined3d_renderer_table {
        renderer: b"HD 3400\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD2350,
    },
    wined3d_renderer_table {
        renderer: b"HD 2400\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD2350,
    },
    wined3d_renderer_table {
        renderer: b"HD 2350\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD2350,
    },
    wined3d_renderer_table {
        renderer: b"X1950\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X1600,
    },
    wined3d_renderer_table {
        renderer: b"X1900\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X1600,
    },
    wined3d_renderer_table {
        renderer: b"X1800\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X1600,
    },
    wined3d_renderer_table {
        renderer: b"X1650\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X1600,
    },
    wined3d_renderer_table {
        renderer: b"X1600\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X1600,
    },
    wined3d_renderer_table {
        renderer: b"HD 2300\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X700,
    },
    wined3d_renderer_table {
        renderer: b"X2500\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X700,
    },
    wined3d_renderer_table {
        renderer: b"X2300\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X700,
    },
    wined3d_renderer_table {
        renderer: b"X1550\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X700,
    },
    wined3d_renderer_table {
        renderer: b"X1450\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X700,
    },
    wined3d_renderer_table {
        renderer: b"X1400\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X700,
    },
    wined3d_renderer_table {
        renderer: b"X1300\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X700,
    },
    wined3d_renderer_table {
        renderer: b"X850\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X700,
    },
    wined3d_renderer_table {
        renderer: b"X800\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X700,
    },
    wined3d_renderer_table {
        renderer: b"X700\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X700,
    },
    wined3d_renderer_table {
        renderer: b"Radeon Xpress\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_XPRESS_200M,
    },
];

static mut cards_redhat: [wined3d_renderer_table; 1] = [wined3d_renderer_table {
    renderer: b"virgl\0" as *const u8 as *const ::core::ffi::c_char,
    id: CARD_REDHAT_VIRGL,
}];

static mut cards_vmware: [wined3d_renderer_table; 1] = [wined3d_renderer_table {
    renderer: b"SVGA3D\0" as *const u8 as *const ::core::ffi::c_char,
    id: CARD_VMWARE_SVGA3D,
}];

static mut cards_amd_mesa: [wined3d_renderer_table; 74] = [
    wined3d_renderer_table {
        renderer: b"NAVI10\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_RX_NAVI_10,
    },
    wined3d_renderer_table {
        renderer: b"NAVI14\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_RX_NAVI_14,
    },
    wined3d_renderer_table {
        renderer: b"POLARIS10\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_RX_480,
    },
    wined3d_renderer_table {
        renderer: b"POLARIS11\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_RX_460,
    },
    wined3d_renderer_table {
        renderer: b"FIJI\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_R9_FURY,
    },
    wined3d_renderer_table {
        renderer: b"TONGA\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_R9_285,
    },
    wined3d_renderer_table {
        renderer: b"HAWAII\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_R9_290,
    },
    wined3d_renderer_table {
        renderer: b"KAVERI\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_R7,
    },
    wined3d_renderer_table {
        renderer: b"KABINI\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_R3,
    },
    wined3d_renderer_table {
        renderer: b"BONAIRE\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD8770,
    },
    wined3d_renderer_table {
        renderer: b"OLAND\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD8670,
    },
    wined3d_renderer_table {
        renderer: b"HAINAN\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD8600M,
    },
    wined3d_renderer_table {
        renderer: b"TAHITI\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD7900,
    },
    wined3d_renderer_table {
        renderer: b"PITCAIRN\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD7800,
    },
    wined3d_renderer_table {
        renderer: b"CAPE VERDE\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD7700,
    },
    wined3d_renderer_table {
        renderer: b"ARUBA\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD7660D,
    },
    wined3d_renderer_table {
        renderer: b"CAYMAN\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6900,
    },
    wined3d_renderer_table {
        renderer: b"BARTS\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6800,
    },
    wined3d_renderer_table {
        renderer: b"TURKS\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6600,
    },
    wined3d_renderer_table {
        renderer: b"SUMO2\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6410D,
    },
    wined3d_renderer_table {
        renderer: b"SUMO\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6550D,
    },
    wined3d_renderer_table {
        renderer: b"CAICOS\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6400,
    },
    wined3d_renderer_table {
        renderer: b"PALM\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD6300,
    },
    wined3d_renderer_table {
        renderer: b"HEMLOCK\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD5900,
    },
    wined3d_renderer_table {
        renderer: b"CYPRESS\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD5800,
    },
    wined3d_renderer_table {
        renderer: b"JUNIPER\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD5700,
    },
    wined3d_renderer_table {
        renderer: b"REDWOOD\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD5600,
    },
    wined3d_renderer_table {
        renderer: b"CEDAR\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD5400,
    },
    wined3d_renderer_table {
        renderer: b"R700\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD4800,
    },
    wined3d_renderer_table {
        renderer: b"RV790\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD4800,
    },
    wined3d_renderer_table {
        renderer: b"RV770\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD4800,
    },
    wined3d_renderer_table {
        renderer: b"RV740\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD4700,
    },
    wined3d_renderer_table {
        renderer: b"RV730\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD4600,
    },
    wined3d_renderer_table {
        renderer: b"RV710\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD4350,
    },
    wined3d_renderer_table {
        renderer: b"RS880\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD4200M,
    },
    wined3d_renderer_table {
        renderer: b"RS780\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD3200,
    },
    wined3d_renderer_table {
        renderer: b"R680\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD2900,
    },
    wined3d_renderer_table {
        renderer: b"R600\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD2900,
    },
    wined3d_renderer_table {
        renderer: b"RV670\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD3850,
    },
    wined3d_renderer_table {
        renderer: b"RV635\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD2600,
    },
    wined3d_renderer_table {
        renderer: b"RV630\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD2600,
    },
    wined3d_renderer_table {
        renderer: b"RV620\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD2350,
    },
    wined3d_renderer_table {
        renderer: b"RV610\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_HD2350,
    },
    wined3d_renderer_table {
        renderer: b"R580\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X1600,
    },
    wined3d_renderer_table {
        renderer: b"R520\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X1600,
    },
    wined3d_renderer_table {
        renderer: b"RV570\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X1600,
    },
    wined3d_renderer_table {
        renderer: b"RV560\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X1600,
    },
    wined3d_renderer_table {
        renderer: b"RV535\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X1600,
    },
    wined3d_renderer_table {
        renderer: b"RV530\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X1600,
    },
    wined3d_renderer_table {
        renderer: b"RV516\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X700,
    },
    wined3d_renderer_table {
        renderer: b"RV515\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X700,
    },
    wined3d_renderer_table {
        renderer: b"R481\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X700,
    },
    wined3d_renderer_table {
        renderer: b"R480\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X700,
    },
    wined3d_renderer_table {
        renderer: b"R430\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X700,
    },
    wined3d_renderer_table {
        renderer: b"R423\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X700,
    },
    wined3d_renderer_table {
        renderer: b"R420\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X700,
    },
    wined3d_renderer_table {
        renderer: b"R410\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X700,
    },
    wined3d_renderer_table {
        renderer: b"RV410\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_X700,
    },
    wined3d_renderer_table {
        renderer: b"RS740\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_XPRESS_200M,
    },
    wined3d_renderer_table {
        renderer: b"RS690\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_XPRESS_200M,
    },
    wined3d_renderer_table {
        renderer: b"RS600\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_XPRESS_200M,
    },
    wined3d_renderer_table {
        renderer: b"RS485\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_XPRESS_200M,
    },
    wined3d_renderer_table {
        renderer: b"RS482\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_XPRESS_200M,
    },
    wined3d_renderer_table {
        renderer: b"RS480\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_XPRESS_200M,
    },
    wined3d_renderer_table {
        renderer: b"RS400\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_XPRESS_200M,
    },
    wined3d_renderer_table {
        renderer: b"RC410\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_XPRESS_200M,
    },
    wined3d_renderer_table {
        renderer: b"R360\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_9500,
    },
    wined3d_renderer_table {
        renderer: b"R350\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_9500,
    },
    wined3d_renderer_table {
        renderer: b"R300\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_9500,
    },
    wined3d_renderer_table {
        renderer: b"RV380\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_9500,
    },
    wined3d_renderer_table {
        renderer: b"RV370\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_9500,
    },
    wined3d_renderer_table {
        renderer: b"RV360\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_9500,
    },
    wined3d_renderer_table {
        renderer: b"RV351\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_9500,
    },
    wined3d_renderer_table {
        renderer: b"RV350\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_AMD_RADEON_9500,
    },
];

static mut cards_nvidia_mesa: [wined3d_renderer_table; 69] = [
    wined3d_renderer_table {
        renderer: b"NV124\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX970,
    },
    wined3d_renderer_table {
        renderer: b"NV120\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX980TI,
    },
    wined3d_renderer_table {
        renderer: b"NV118\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_840M,
    },
    wined3d_renderer_table {
        renderer: b"NV117\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX750,
    },
    wined3d_renderer_table {
        renderer: b"NV108\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT740M,
    },
    wined3d_renderer_table {
        renderer: b"NV106\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT720,
    },
    wined3d_renderer_table {
        renderer: b"NVF1\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX780TI,
    },
    wined3d_renderer_table {
        renderer: b"NVF0\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX780,
    },
    wined3d_renderer_table {
        renderer: b"NVE6\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX770M,
    },
    wined3d_renderer_table {
        renderer: b"NVE4\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX680,
    },
    wined3d_renderer_table {
        renderer: b"NVD9\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT520,
    },
    wined3d_renderer_table {
        renderer: b"NVD7\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_820M,
    },
    wined3d_renderer_table {
        renderer: b"NVCF\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX550,
    },
    wined3d_renderer_table {
        renderer: b"NVCE\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX560,
    },
    wined3d_renderer_table {
        renderer: b"NVC8\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX570,
    },
    wined3d_renderer_table {
        renderer: b"NVC4\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX460,
    },
    wined3d_renderer_table {
        renderer: b"NVC3\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT440,
    },
    wined3d_renderer_table {
        renderer: b"NVC1\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT420,
    },
    wined3d_renderer_table {
        renderer: b"NVC0\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX480,
    },
    wined3d_renderer_table {
        renderer: b"NVAF\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT320M,
    },
    wined3d_renderer_table {
        renderer: b"NVAC\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_8200,
    },
    wined3d_renderer_table {
        renderer: b"NVAA\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_8200,
    },
    wined3d_renderer_table {
        renderer: b"NVA8\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_210,
    },
    wined3d_renderer_table {
        renderer: b"NVA5\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT220,
    },
    wined3d_renderer_table {
        renderer: b"NVA3\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT240,
    },
    wined3d_renderer_table {
        renderer: b"NVA0\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX280,
    },
    wined3d_renderer_table {
        renderer: b"NV98\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_9200,
    },
    wined3d_renderer_table {
        renderer: b"NV96\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_9400GT,
    },
    wined3d_renderer_table {
        renderer: b"NV94\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_9600GT,
    },
    wined3d_renderer_table {
        renderer: b"NV92\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_9800GT,
    },
    wined3d_renderer_table {
        renderer: b"NV86\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_8500GT,
    },
    wined3d_renderer_table {
        renderer: b"NV84\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_8600GT,
    },
    wined3d_renderer_table {
        renderer: b"NV50\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_8800GTX,
    },
    wined3d_renderer_table {
        renderer: b"NV68\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_6200,
    },
    wined3d_renderer_table {
        renderer: b"NV67\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_6200,
    },
    wined3d_renderer_table {
        renderer: b"NV63\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_6200,
    },
    wined3d_renderer_table {
        renderer: b"NV4E\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_6200,
    },
    wined3d_renderer_table {
        renderer: b"NV4C\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_6200,
    },
    wined3d_renderer_table {
        renderer: b"NV4B\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_7600,
    },
    wined3d_renderer_table {
        renderer: b"NV4A\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_6200,
    },
    wined3d_renderer_table {
        renderer: b"NV49\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_7800GT,
    },
    wined3d_renderer_table {
        renderer: b"NV47\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_7800GT,
    },
    wined3d_renderer_table {
        renderer: b"NV46\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_7400,
    },
    wined3d_renderer_table {
        renderer: b"NV45\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_6800,
    },
    wined3d_renderer_table {
        renderer: b"NV44\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_6200,
    },
    wined3d_renderer_table {
        renderer: b"NV43\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_6600GT,
    },
    wined3d_renderer_table {
        renderer: b"NV42\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_6800,
    },
    wined3d_renderer_table {
        renderer: b"NV41\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_6800,
    },
    wined3d_renderer_table {
        renderer: b"NV40\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_6800,
    },
    wined3d_renderer_table {
        renderer: b"NV38\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCEFX_5800,
    },
    wined3d_renderer_table {
        renderer: b"NV36\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCEFX_5800,
    },
    wined3d_renderer_table {
        renderer: b"NV35\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCEFX_5800,
    },
    wined3d_renderer_table {
        renderer: b"NV34\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCEFX_5200,
    },
    wined3d_renderer_table {
        renderer: b"NV31\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCEFX_5600,
    },
    wined3d_renderer_table {
        renderer: b"NV30\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCEFX_5800,
    },
    wined3d_renderer_table {
        renderer: b"nv28\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE4_TI4200,
    },
    wined3d_renderer_table {
        renderer: b"nv25\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE4_TI4200,
    },
    wined3d_renderer_table {
        renderer: b"nv20\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE3,
    },
    wined3d_renderer_table {
        renderer: b"nv1F\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE4_MX,
    },
    wined3d_renderer_table {
        renderer: b"nv1A\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE2,
    },
    wined3d_renderer_table {
        renderer: b"nv18\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE4_MX,
    },
    wined3d_renderer_table {
        renderer: b"nv17\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE4_MX,
    },
    wined3d_renderer_table {
        renderer: b"nv16\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE2,
    },
    wined3d_renderer_table {
        renderer: b"nv15\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE2,
    },
    wined3d_renderer_table {
        renderer: b"nv11\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE2_MX,
    },
    wined3d_renderer_table {
        renderer: b"nv10\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE,
    },
    wined3d_renderer_table {
        renderer: b"nv05\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_RIVA_TNT2,
    },
    wined3d_renderer_table {
        renderer: b"nv04\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_RIVA_TNT,
    },
    wined3d_renderer_table {
        renderer: b"nv03\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_RIVA_128,
    },
];

static mut cards_nvidia_binary: [wined3d_renderer_table; 196] = [
    wined3d_renderer_table {
        renderer: b"RTX 4090\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX4090,
    },
    wined3d_renderer_table {
        renderer: b"RTX 4080 SUPER\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX4080SUPER,
    },
    wined3d_renderer_table {
        renderer: b"RTX 4080\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX4080,
    },
    wined3d_renderer_table {
        renderer: b"RTX 4070 Ti SUPER\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX4070TISUPER,
    },
    wined3d_renderer_table {
        renderer: b"RTX 4070 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX4070TI,
    },
    wined3d_renderer_table {
        renderer: b"RTX 4070 SUPER\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX4070SUPER,
    },
    wined3d_renderer_table {
        renderer: b"RTX 4070\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX4070,
    },
    wined3d_renderer_table {
        renderer: b"RTX 4060 Ti 16GB\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX4060TI16G,
    },
    wined3d_renderer_table {
        renderer: b"RTX 4060 Ti 8GB\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX4060TI8G,
    },
    wined3d_renderer_table {
        renderer: b"RTX 4060\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX4060,
    },
    wined3d_renderer_table {
        renderer: b"Tesla T4\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_TESLA_T4,
    },
    wined3d_renderer_table {
        renderer: b"Ampere A10\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_AMPERE_A10,
    },
    wined3d_renderer_table {
        renderer: b"RTX 3090 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX3090TI,
    },
    wined3d_renderer_table {
        renderer: b"RTX 3090\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX3090,
    },
    wined3d_renderer_table {
        renderer: b"RTX 3080 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX3080TI,
    },
    wined3d_renderer_table {
        renderer: b"RTX 3080 12GB\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX3080_12GB,
    },
    wined3d_renderer_table {
        renderer: b"RTX 3080 10GB LHR\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX3080_10GB_LHR,
    },
    wined3d_renderer_table {
        renderer: b"RTX 3080 10GB\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX3080_10GB,
    },
    wined3d_renderer_table {
        renderer: b"RTX 3070 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX3070TI,
    },
    wined3d_renderer_table {
        renderer: b"RTX 3070 mobile\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX3070_MOBILE,
    },
    wined3d_renderer_table {
        renderer: b"RTX 3070 LHR\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX3070_LHR,
    },
    wined3d_renderer_table {
        renderer: b"RTX 3070\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX3070,
    },
    wined3d_renderer_table {
        renderer: b"RTX 3060 Ti GA104 LHR\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX3060TI_GA104_LHR,
    },
    wined3d_renderer_table {
        renderer: b"RTX 3060 Ti GA104\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX3060TI_GA104,
    },
    wined3d_renderer_table {
        renderer: b"RTX 3060 Ti GA103\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX3060TI_GA103,
    },
    wined3d_renderer_table {
        renderer: b"RTX 3060 LHR\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX3060_LHR,
    },
    wined3d_renderer_table {
        renderer: b"RTX 3060\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX3060,
    },
    wined3d_renderer_table {
        renderer: b"RTX 3050\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX3050,
    },
    wined3d_renderer_table {
        renderer: b"RTX 2080 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX2080TI,
    },
    wined3d_renderer_table {
        renderer: b"RTX 2080\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX2080,
    },
    wined3d_renderer_table {
        renderer: b"RTX 2070\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX2070,
    },
    wined3d_renderer_table {
        renderer: b"RTX 2060\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_RTX2060,
    },
    wined3d_renderer_table {
        renderer: b"GTX 1660 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX1660TI,
    },
    wined3d_renderer_table {
        renderer: b"GTX 1660 SUPER\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX1660SUPER,
    },
    wined3d_renderer_table {
        renderer: b"GTX 1650 SUPER\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX1650SUPER,
    },
    wined3d_renderer_table {
        renderer: b"GTX 1650\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX1650,
    },
    wined3d_renderer_table {
        renderer: b"TITAN V\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_TITANV,
    },
    wined3d_renderer_table {
        renderer: b"TITAN X (Pascal)\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_TITANX_PASCAL,
    },
    wined3d_renderer_table {
        renderer: b"GTX 1080 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX1080TI,
    },
    wined3d_renderer_table {
        renderer: b"GTX 1080\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX1080,
    },
    wined3d_renderer_table {
        renderer: b"GTX 1070\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX1070,
    },
    wined3d_renderer_table {
        renderer: b"GTX 1070M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX1070M,
    },
    wined3d_renderer_table {
        renderer: b"GTX 1060 3GB\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX1060_3GB,
    },
    wined3d_renderer_table {
        renderer: b"GTX 1060\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX1060,
    },
    wined3d_renderer_table {
        renderer: b"GTX 1050 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX1050TI,
    },
    wined3d_renderer_table {
        renderer: b"GTX 1050\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX1050,
    },
    wined3d_renderer_table {
        renderer: b"GT 1030\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT1030,
    },
    wined3d_renderer_table {
        renderer: b"GTX 980 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX980TI,
    },
    wined3d_renderer_table {
        renderer: b"GTX 980\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX980,
    },
    wined3d_renderer_table {
        renderer: b"GTX 970M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX970M,
    },
    wined3d_renderer_table {
        renderer: b"GTX 970\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX970,
    },
    wined3d_renderer_table {
        renderer: b"GTX TITAN X\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTXTITANX,
    },
    wined3d_renderer_table {
        renderer: b"GTX 960M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX960M,
    },
    wined3d_renderer_table {
        renderer: b"GTX 960\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX960,
    },
    wined3d_renderer_table {
        renderer: b"GTX 950M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX950M,
    },
    wined3d_renderer_table {
        renderer: b"GTX 950\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX950,
    },
    wined3d_renderer_table {
        renderer: b"GeForce 940M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_940M,
    },
    wined3d_renderer_table {
        renderer: b"GTX 880M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX880M,
    },
    wined3d_renderer_table {
        renderer: b"GTX 870M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX870M,
    },
    wined3d_renderer_table {
        renderer: b"GTX 860M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX860M,
    },
    wined3d_renderer_table {
        renderer: b"GTX 850M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX850M,
    },
    wined3d_renderer_table {
        renderer: b"GeForce 845M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_845M,
    },
    wined3d_renderer_table {
        renderer: b"GeForce 840M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_840M,
    },
    wined3d_renderer_table {
        renderer: b"GeForce 830M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_830M,
    },
    wined3d_renderer_table {
        renderer: b"GeForce 820M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_820M,
    },
    wined3d_renderer_table {
        renderer: b"GTX 780 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX780TI,
    },
    wined3d_renderer_table {
        renderer: b"GTX TITAN Black\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTXTITANB,
    },
    wined3d_renderer_table {
        renderer: b"GTX TITAN Z\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTXTITANZ,
    },
    wined3d_renderer_table {
        renderer: b"GTX TITAN\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTXTITAN,
    },
    wined3d_renderer_table {
        renderer: b"GTX 780\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX780,
    },
    wined3d_renderer_table {
        renderer: b"GTX 770M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX770M,
    },
    wined3d_renderer_table {
        renderer: b"GTX 770\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX770,
    },
    wined3d_renderer_table {
        renderer: b"GTX 765M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX765M,
    },
    wined3d_renderer_table {
        renderer: b"GTX 760 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX760TI,
    },
    wined3d_renderer_table {
        renderer: b"GTX 760\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX760,
    },
    wined3d_renderer_table {
        renderer: b"GTX 750 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX750TI,
    },
    wined3d_renderer_table {
        renderer: b"GTX 750\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX750,
    },
    wined3d_renderer_table {
        renderer: b"GT 750M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT750M,
    },
    wined3d_renderer_table {
        renderer: b"GT 740M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT740M,
    },
    wined3d_renderer_table {
        renderer: b"GT 730M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT730M,
    },
    wined3d_renderer_table {
        renderer: b"GT 730\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT730,
    },
    wined3d_renderer_table {
        renderer: b"GTX 690\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX690,
    },
    wined3d_renderer_table {
        renderer: b"GTX 680\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX680,
    },
    wined3d_renderer_table {
        renderer: b"GTX 675MX\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX675MX_1,
    },
    wined3d_renderer_table {
        renderer: b"GTX 670MX\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX670MX,
    },
    wined3d_renderer_table {
        renderer: b"GTX 670\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX670,
    },
    wined3d_renderer_table {
        renderer: b"GTX 660 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX660TI,
    },
    wined3d_renderer_table {
        renderer: b"GTX 660M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX660M,
    },
    wined3d_renderer_table {
        renderer: b"GTX 660\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX660,
    },
    wined3d_renderer_table {
        renderer: b"GTX 650 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX650TI,
    },
    wined3d_renderer_table {
        renderer: b"GTX 650\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX650,
    },
    wined3d_renderer_table {
        renderer: b"GT 650M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT650M,
    },
    wined3d_renderer_table {
        renderer: b"GT 640M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT640M,
    },
    wined3d_renderer_table {
        renderer: b"GT 630M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT630M,
    },
    wined3d_renderer_table {
        renderer: b"GT 630\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT630,
    },
    wined3d_renderer_table {
        renderer: b"GT 610\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT610,
    },
    wined3d_renderer_table {
        renderer: b"GTX 580\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX580,
    },
    wined3d_renderer_table {
        renderer: b"GTX 570\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX570,
    },
    wined3d_renderer_table {
        renderer: b"GTX 560 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX560TI,
    },
    wined3d_renderer_table {
        renderer: b"GTX 560M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX560M,
    },
    wined3d_renderer_table {
        renderer: b"GTX 560\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX560,
    },
    wined3d_renderer_table {
        renderer: b"GT 555M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT555M,
    },
    wined3d_renderer_table {
        renderer: b"GTX 550 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX550,
    },
    wined3d_renderer_table {
        renderer: b"GT 540M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT540M,
    },
    wined3d_renderer_table {
        renderer: b"GT 525M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT525M,
    },
    wined3d_renderer_table {
        renderer: b"GT 520\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT520,
    },
    wined3d_renderer_table {
        renderer: b"GTX 480\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX480,
    },
    wined3d_renderer_table {
        renderer: b"GTX 470\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX470,
    },
    wined3d_renderer_table {
        renderer: b"GTX 465\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX465,
    },
    wined3d_renderer_table {
        renderer: b"GTX 460M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX460M,
    },
    wined3d_renderer_table {
        renderer: b"GTX 460\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX460,
    },
    wined3d_renderer_table {
        renderer: b"GTS 450\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTS450,
    },
    wined3d_renderer_table {
        renderer: b"GT 440\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT440,
    },
    wined3d_renderer_table {
        renderer: b"GT 430\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT430,
    },
    wined3d_renderer_table {
        renderer: b"GT 425M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT425M,
    },
    wined3d_renderer_table {
        renderer: b"GT 420\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT420,
    },
    wined3d_renderer_table {
        renderer: b"410M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_410M,
    },
    wined3d_renderer_table {
        renderer: b"GT 330\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT330,
    },
    wined3d_renderer_table {
        renderer: b"GTS 360M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTS350M,
    },
    wined3d_renderer_table {
        renderer: b"GTS 350M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTS350M,
    },
    wined3d_renderer_table {
        renderer: b"GT 330M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT325M,
    },
    wined3d_renderer_table {
        renderer: b"GT 325M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT325M,
    },
    wined3d_renderer_table {
        renderer: b"GT 320M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT320M,
    },
    wined3d_renderer_table {
        renderer: b"320M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_320M,
    },
    wined3d_renderer_table {
        renderer: b"315M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_315M,
    },
    wined3d_renderer_table {
        renderer: b"GTX 295\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX280,
    },
    wined3d_renderer_table {
        renderer: b"GTX 285\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX280,
    },
    wined3d_renderer_table {
        renderer: b"GTX 280\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX280,
    },
    wined3d_renderer_table {
        renderer: b"GTX 275\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX275,
    },
    wined3d_renderer_table {
        renderer: b"GTX 260\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTX260,
    },
    wined3d_renderer_table {
        renderer: b"GTS 250\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GTS250,
    },
    wined3d_renderer_table {
        renderer: b"GT 240\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT240,
    },
    wined3d_renderer_table {
        renderer: b"GT 220\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_GT220,
    },
    wined3d_renderer_table {
        renderer: b"GeForce 310\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_210,
    },
    wined3d_renderer_table {
        renderer: b"GeForce 305\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_210,
    },
    wined3d_renderer_table {
        renderer: b"GeForce 210\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_210,
    },
    wined3d_renderer_table {
        renderer: b"G 210\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_210,
    },
    wined3d_renderer_table {
        renderer: b"GTS 150\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_9800GT,
    },
    wined3d_renderer_table {
        renderer: b"9800\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_9800GT,
    },
    wined3d_renderer_table {
        renderer: b"9700M GT\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_9700MGT,
    },
    wined3d_renderer_table {
        renderer: b"GT 140\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_9600GT,
    },
    wined3d_renderer_table {
        renderer: b"9600\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_9600GT,
    },
    wined3d_renderer_table {
        renderer: b"GT 130\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_9500GT,
    },
    wined3d_renderer_table {
        renderer: b"GT 120\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_9500GT,
    },
    wined3d_renderer_table {
        renderer: b"9500\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_9500GT,
    },
    wined3d_renderer_table {
        renderer: b"9400M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_9400M,
    },
    wined3d_renderer_table {
        renderer: b"9400\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_9400GT,
    },
    wined3d_renderer_table {
        renderer: b"9300\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_9300,
    },
    wined3d_renderer_table {
        renderer: b"9200\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_9200,
    },
    wined3d_renderer_table {
        renderer: b"9100\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_9200,
    },
    wined3d_renderer_table {
        renderer: b"G 100\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_9200,
    },
    wined3d_renderer_table {
        renderer: b"8800 GTX\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_8800GTX,
    },
    wined3d_renderer_table {
        renderer: b"8800\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_8800GTS,
    },
    wined3d_renderer_table {
        renderer: b"8600M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_8600MGT,
    },
    wined3d_renderer_table {
        renderer: b"8600 M\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_8600MGT,
    },
    wined3d_renderer_table {
        renderer: b"8700\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_8600GT,
    },
    wined3d_renderer_table {
        renderer: b"8600\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_8600GT,
    },
    wined3d_renderer_table {
        renderer: b"8500\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_8500GT,
    },
    wined3d_renderer_table {
        renderer: b"8400\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_8400GS,
    },
    wined3d_renderer_table {
        renderer: b"8300\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_8300GS,
    },
    wined3d_renderer_table {
        renderer: b"8200\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_8200,
    },
    wined3d_renderer_table {
        renderer: b"8100\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_8200,
    },
    wined3d_renderer_table {
        renderer: b"Quadro FX 5\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_7800GT,
    },
    wined3d_renderer_table {
        renderer: b"Quadro FX 4\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_7800GT,
    },
    wined3d_renderer_table {
        renderer: b"7950\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_7800GT,
    },
    wined3d_renderer_table {
        renderer: b"7900\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_7800GT,
    },
    wined3d_renderer_table {
        renderer: b"7800\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_7800GT,
    },
    wined3d_renderer_table {
        renderer: b"7700\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_7600,
    },
    wined3d_renderer_table {
        renderer: b"7600\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_7600,
    },
    wined3d_renderer_table {
        renderer: b"7400\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_7400,
    },
    wined3d_renderer_table {
        renderer: b"7300\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_7300,
    },
    wined3d_renderer_table {
        renderer: b"6800\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_6800,
    },
    wined3d_renderer_table {
        renderer: b"6700\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_6600GT,
    },
    wined3d_renderer_table {
        renderer: b"6610\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_6600GT,
    },
    wined3d_renderer_table {
        renderer: b"6600\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE_6600GT,
    },
    wined3d_renderer_table {
        renderer: b"Quadro FX\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCEFX_5800,
    },
    wined3d_renderer_table {
        renderer: b"5950\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCEFX_5800,
    },
    wined3d_renderer_table {
        renderer: b"5900\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCEFX_5800,
    },
    wined3d_renderer_table {
        renderer: b"5800\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCEFX_5800,
    },
    wined3d_renderer_table {
        renderer: b"5750\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCEFX_5600,
    },
    wined3d_renderer_table {
        renderer: b"5700\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCEFX_5600,
    },
    wined3d_renderer_table {
        renderer: b"5650\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCEFX_5600,
    },
    wined3d_renderer_table {
        renderer: b"5600\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCEFX_5600,
    },
    wined3d_renderer_table {
        renderer: b"5500\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCEFX_5200,
    },
    wined3d_renderer_table {
        renderer: b"5300\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCEFX_5200,
    },
    wined3d_renderer_table {
        renderer: b"5250\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCEFX_5200,
    },
    wined3d_renderer_table {
        renderer: b"5200\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCEFX_5200,
    },
    wined3d_renderer_table {
        renderer: b"5100\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCEFX_5200,
    },
    wined3d_renderer_table {
        renderer: b"Quadro4\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE4_TI4200,
    },
    wined3d_renderer_table {
        renderer: b"GeForce4 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE4_TI4200,
    },
    wined3d_renderer_table {
        renderer: b"GeForce4 MX\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE4_MX,
    },
    wined3d_renderer_table {
        renderer: b"Quadro2 MXR\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE2_MX,
    },
    wined3d_renderer_table {
        renderer: b"GeForce2 MX\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE2_MX,
    },
    wined3d_renderer_table {
        renderer: b"Quadro2\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE2,
    },
    wined3d_renderer_table {
        renderer: b"GeForce2\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_GEFORCE2,
    },
    wined3d_renderer_table {
        renderer: b"TNT2\0" as *const u8 as *const ::core::ffi::c_char,
        id: CARD_NVIDIA_RIVA_TNT2,
    },
];

static mut redhat_gl_vendor_table: [gl_vendor_selection; 1] = [gl_vendor_selection {
        gl_vendor: GL_VENDOR_MESA,
        description: b"Red Hat driver\0" as *const u8 as *const ::core::ffi::c_char,
        cards: &raw const cards_redhat as *const wined3d_renderer_table,
        cards_size: (::core::mem::size_of::<[wined3d_renderer_table; 1]>() as size_t)
            .wrapping_div(::core::mem::size_of::<wined3d_renderer_table>() as size_t),
    }];

static mut vmware_gl_vendor_table: [gl_vendor_selection; 1] = [gl_vendor_selection {
        gl_vendor: GL_VENDOR_MESA,
        description: b"VMware driver\0" as *const u8 as *const ::core::ffi::c_char,
        cards: &raw const cards_vmware as *const wined3d_renderer_table,
        cards_size: (::core::mem::size_of::<[wined3d_renderer_table; 1]>() as size_t)
            .wrapping_div(::core::mem::size_of::<wined3d_renderer_table>() as size_t),
    }];

static mut nvidia_gl_vendor_table: [gl_vendor_selection; 3] = [
        gl_vendor_selection {
            gl_vendor: GL_VENDOR_APPLE,
            description: b"Apple OSX NVidia binary driver\0" as *const u8
                as *const ::core::ffi::c_char,
            cards: &raw const cards_nvidia_binary as *const wined3d_renderer_table,
            cards_size: (::core::mem::size_of::<[wined3d_renderer_table; 196]>() as size_t)
                .wrapping_div(::core::mem::size_of::<wined3d_renderer_table>() as size_t),
        },
        gl_vendor_selection {
            gl_vendor: GL_VENDOR_MESA,
            description: b"Mesa Nouveau driver\0" as *const u8 as *const ::core::ffi::c_char,
            cards: &raw const cards_nvidia_mesa as *const wined3d_renderer_table,
            cards_size: (::core::mem::size_of::<[wined3d_renderer_table; 69]>() as size_t)
                .wrapping_div(::core::mem::size_of::<wined3d_renderer_table>() as size_t),
        },
        gl_vendor_selection {
            gl_vendor: GL_VENDOR_NVIDIA,
            description: b"NVIDIA binary driver\0" as *const u8 as *const ::core::ffi::c_char,
            cards: &raw const cards_nvidia_binary as *const wined3d_renderer_table,
            cards_size: (::core::mem::size_of::<[wined3d_renderer_table; 196]>() as size_t)
                .wrapping_div(::core::mem::size_of::<wined3d_renderer_table>() as size_t),
        },
    ];

static mut amd_gl_vendor_table: [gl_vendor_selection; 3] = [
        gl_vendor_selection {
            gl_vendor: GL_VENDOR_APPLE,
            description: b"Apple OSX AMD/ATI binary driver\0" as *const u8
                as *const ::core::ffi::c_char,
            cards: &raw const cards_amd_binary as *const wined3d_renderer_table,
            cards_size: (::core::mem::size_of::<[wined3d_renderer_table; 79]>() as size_t)
                .wrapping_div(::core::mem::size_of::<wined3d_renderer_table>() as size_t),
        },
        gl_vendor_selection {
            gl_vendor: GL_VENDOR_FGLRX,
            description: b"AMD/ATI binary driver\0" as *const u8 as *const ::core::ffi::c_char,
            cards: &raw const cards_amd_binary as *const wined3d_renderer_table,
            cards_size: (::core::mem::size_of::<[wined3d_renderer_table; 79]>() as size_t)
                .wrapping_div(::core::mem::size_of::<wined3d_renderer_table>() as size_t),
        },
        gl_vendor_selection {
            gl_vendor: GL_VENDOR_MESA,
            description: b"Mesa AMD/ATI driver\0" as *const u8 as *const ::core::ffi::c_char,
            cards: &raw const cards_amd_mesa as *const wined3d_renderer_table,
            cards_size: (::core::mem::size_of::<[wined3d_renderer_table; 74]>() as size_t)
                .wrapping_div(::core::mem::size_of::<wined3d_renderer_table>() as size_t),
        },
    ];

static mut intel_gl_vendor_table: [gl_vendor_selection; 2] = [
        gl_vendor_selection {
            gl_vendor: GL_VENDOR_APPLE,
            description: b"Apple OSX Intel binary driver\0" as *const u8
                as *const ::core::ffi::c_char,
            cards: &raw const cards_intel as *const wined3d_renderer_table,
            cards_size: (::core::mem::size_of::<[wined3d_renderer_table; 49]>() as size_t)
                .wrapping_div(::core::mem::size_of::<wined3d_renderer_table>() as size_t),
        },
        gl_vendor_selection {
            gl_vendor: GL_VENDOR_MESA,
            description: b"Mesa Intel driver\0" as *const u8 as *const ::core::ffi::c_char,
            cards: &raw const cards_intel as *const wined3d_renderer_table,
            cards_size: (::core::mem::size_of::<[wined3d_renderer_table; 49]>() as size_t)
                .wrapping_div(::core::mem::size_of::<wined3d_renderer_table>() as size_t),
        },
    ];

static mut card_vendor_table: [C2RustUnnamed_52; 5] = [
        C2RustUnnamed_52 {
            card_vendor: HW_VENDOR_AMD,
            description: b"AMD\0" as *const u8 as *const ::core::ffi::c_char,
            gl_vendor_selection: &raw const amd_gl_vendor_table as *const gl_vendor_selection,
            gl_vendor_count: (::core::mem::size_of::<[gl_vendor_selection; 3]>() as usize)
                .wrapping_div(::core::mem::size_of::<gl_vendor_selection>() as usize)
                as ::core::ffi::c_uint,
        },
        C2RustUnnamed_52 {
            card_vendor: HW_VENDOR_NVIDIA,
            description: b"NVIDIA\0" as *const u8 as *const ::core::ffi::c_char,
            gl_vendor_selection: &raw const nvidia_gl_vendor_table as *const gl_vendor_selection,
            gl_vendor_count: (::core::mem::size_of::<[gl_vendor_selection; 3]>() as usize)
                .wrapping_div(::core::mem::size_of::<gl_vendor_selection>() as usize)
                as ::core::ffi::c_uint,
        },
        C2RustUnnamed_52 {
            card_vendor: HW_VENDOR_REDHAT,
            description: b"Red Hat\0" as *const u8 as *const ::core::ffi::c_char,
            gl_vendor_selection: &raw const redhat_gl_vendor_table as *const gl_vendor_selection,
            gl_vendor_count: (::core::mem::size_of::<[gl_vendor_selection; 1]>() as usize)
                .wrapping_div(::core::mem::size_of::<gl_vendor_selection>() as usize)
                as ::core::ffi::c_uint,
        },
        C2RustUnnamed_52 {
            card_vendor: HW_VENDOR_VMWARE,
            description: b"VMware\0" as *const u8 as *const ::core::ffi::c_char,
            gl_vendor_selection: &raw const vmware_gl_vendor_table as *const gl_vendor_selection,
            gl_vendor_count: (::core::mem::size_of::<[gl_vendor_selection; 1]>() as usize)
                .wrapping_div(::core::mem::size_of::<gl_vendor_selection>() as usize)
                as ::core::ffi::c_uint,
        },
        C2RustUnnamed_52 {
            card_vendor: HW_VENDOR_INTEL,
            description: b"Intel\0" as *const u8 as *const ::core::ffi::c_char,
            gl_vendor_selection: &raw const intel_gl_vendor_table as *const gl_vendor_selection,
            gl_vendor_count: (::core::mem::size_of::<[gl_vendor_selection; 2]>() as usize)
                .wrapping_div(::core::mem::size_of::<gl_vendor_selection>() as usize)
                as ::core::ffi::c_uint,
        },
    ];

static mut wined3d_adapter_gl_ops: wined3d_adapter_ops = unsafe {
    wined3d_adapter_ops {
        adapter_destroy: Some(
            adapter_gl_destroy as unsafe extern "C" fn(*mut wined3d_adapter) -> (),
        ),
        adapter_create_device: Some(
            adapter_gl_create_device
                as unsafe extern "C" fn(
                    *mut wined3d,
                    *const wined3d_adapter,
                    wined3d_device_type,
                    HWND,
                    ::core::ffi::c_uint,
                    BYTE,
                    *const wined3d_feature_level,
                    ::core::ffi::c_uint,
                    *mut wined3d_device_parent,
                    *mut *mut wined3d_device,
                ) -> HRESULT,
        ),
        adapter_destroy_device: Some(
            adapter_gl_destroy_device as unsafe extern "C" fn(*mut wined3d_device) -> (),
        ),
        adapter_acquire_context: Some(
            adapter_gl_acquire_context
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                ) -> *mut wined3d_context,
        ),
        adapter_release_context: Some(
            adapter_gl_release_context as unsafe extern "C" fn(*mut wined3d_context) -> (),
        ),
        adapter_get_wined3d_caps: Some(
            adapter_gl_get_wined3d_caps
                as unsafe extern "C" fn(*const wined3d_adapter, *mut wined3d_caps) -> (),
        ),
        adapter_check_format: Some(
            adapter_gl_check_format
                as unsafe extern "C" fn(
                    *const wined3d_adapter,
                    *const wined3d_format,
                    *const wined3d_format,
                    *const wined3d_format,
                ) -> BOOL,
        ),
        adapter_init_3d: Some(
            adapter_gl_init_3d as unsafe extern "C" fn(*mut wined3d_device) -> HRESULT,
        ),
        adapter_uninit_3d: Some(
            adapter_gl_uninit_3d as unsafe extern "C" fn(*mut wined3d_device) -> (),
        ),
        adapter_map_bo_address: Some(
            adapter_gl_map_bo_address
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *const wined3d_bo_address,
                    size_t,
                    uint32_t,
                ) -> *mut ::core::ffi::c_void,
        ),
        adapter_unmap_bo_address: Some(
            adapter_gl_unmap_bo_address
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *const wined3d_bo_address,
                    ::core::ffi::c_uint,
                    *const wined3d_range,
                ) -> (),
        ),
        adapter_copy_bo_address: Some(
            adapter_gl_copy_bo_address
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *const wined3d_bo_address,
                    *const wined3d_bo_address,
                    ::core::ffi::c_uint,
                    *const wined3d_range,
                    uint32_t,
                ) -> (),
        ),
        adapter_flush_bo_address: Some(
            adapter_gl_flush_bo_address
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *const wined3d_const_bo_address,
                    size_t,
                ) -> (),
        ),
        adapter_alloc_bo: Some(
            adapter_gl_alloc_bo
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    *mut wined3d_resource,
                    ::core::ffi::c_uint,
                    *mut wined3d_bo_address,
                ) -> bool,
        ),
        adapter_destroy_bo: Some(
            adapter_gl_destroy_bo
                as unsafe extern "C" fn(*mut wined3d_context, *mut wined3d_bo) -> (),
        ),
        adapter_create_swapchain: Some(
            adapter_gl_create_swapchain
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    *const wined3d_swapchain_desc,
                    *mut wined3d_swapchain_state_parent,
                    *mut ::core::ffi::c_void,
                    *const wined3d_parent_ops,
                    *mut *mut wined3d_swapchain,
                ) -> HRESULT,
        ),
        adapter_destroy_swapchain: Some(
            adapter_gl_destroy_swapchain as unsafe extern "C" fn(*mut wined3d_swapchain) -> (),
        ),
        adapter_create_buffer: Some(
            adapter_gl_create_buffer
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    *const wined3d_buffer_desc,
                    *const wined3d_sub_resource_data,
                    *mut ::core::ffi::c_void,
                    *const wined3d_parent_ops,
                    *mut *mut wined3d_buffer,
                ) -> HRESULT,
        ),
        adapter_destroy_buffer: Some(
            adapter_gl_destroy_buffer as unsafe extern "C" fn(*mut wined3d_buffer) -> (),
        ),
        adapter_create_texture: Some(
            adapter_gl_create_texture
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    *const wined3d_resource_desc,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    uint32_t,
                    *mut ::core::ffi::c_void,
                    *const wined3d_parent_ops,
                    *mut *mut wined3d_texture,
                ) -> HRESULT,
        ),
        adapter_destroy_texture: Some(
            adapter_gl_destroy_texture as unsafe extern "C" fn(*mut wined3d_texture) -> (),
        ),
        adapter_create_rendertarget_view: Some(
            adapter_gl_create_rendertarget_view
                as unsafe extern "C" fn(
                    *const wined3d_view_desc,
                    *mut wined3d_resource,
                    *mut ::core::ffi::c_void,
                    *const wined3d_parent_ops,
                    *mut *mut wined3d_rendertarget_view,
                ) -> HRESULT,
        ),
        adapter_destroy_rendertarget_view: Some(
            adapter_gl_destroy_rendertarget_view
                as unsafe extern "C" fn(*mut wined3d_rendertarget_view) -> (),
        ),
        adapter_create_shader_resource_view: Some(
            adapter_gl_create_shader_resource_view
                as unsafe extern "C" fn(
                    *const wined3d_view_desc,
                    *mut wined3d_resource,
                    *mut ::core::ffi::c_void,
                    *const wined3d_parent_ops,
                    *mut *mut wined3d_shader_resource_view,
                ) -> HRESULT,
        ),
        adapter_destroy_shader_resource_view: Some(
            adapter_gl_destroy_shader_resource_view
                as unsafe extern "C" fn(*mut wined3d_shader_resource_view) -> (),
        ),
        adapter_create_unordered_access_view: Some(
            adapter_gl_create_unordered_access_view
                as unsafe extern "C" fn(
                    *const wined3d_view_desc,
                    *mut wined3d_resource,
                    *mut ::core::ffi::c_void,
                    *const wined3d_parent_ops,
                    *mut *mut wined3d_unordered_access_view,
                ) -> HRESULT,
        ),
        adapter_destroy_unordered_access_view: Some(
            adapter_gl_destroy_unordered_access_view
                as unsafe extern "C" fn(*mut wined3d_unordered_access_view) -> (),
        ),
        adapter_create_sampler: Some(
            adapter_gl_create_sampler
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    *const wined3d_sampler_desc,
                    *mut ::core::ffi::c_void,
                    *const wined3d_parent_ops,
                    *mut *mut wined3d_sampler,
                ) -> HRESULT,
        ),
        adapter_destroy_sampler: Some(
            adapter_gl_destroy_sampler as unsafe extern "C" fn(*mut wined3d_sampler) -> (),
        ),
        adapter_create_query: Some(
            adapter_gl_create_query
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    wined3d_query_type,
                    *mut ::core::ffi::c_void,
                    *const wined3d_parent_ops,
                    *mut *mut wined3d_query,
                ) -> HRESULT,
        ),
        adapter_destroy_query: Some(
            adapter_gl_destroy_query as unsafe extern "C" fn(*mut wined3d_query) -> (),
        ),
        adapter_flush_context: Some(
            adapter_gl_flush_context as unsafe extern "C" fn(*mut wined3d_context) -> (),
        ),
        adapter_draw_primitive: Some(
            draw_primitive
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    *const wined3d_state,
                    *const wined3d_draw_parameters,
                ) -> (),
        ),
        adapter_dispatch_compute: Some(
            dispatch_compute
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    *const wined3d_state,
                    *const wined3d_dispatch_parameters,
                ) -> (),
        ),
        adapter_clear_uav: Some(
            adapter_gl_clear_uav
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *mut wined3d_unordered_access_view,
                    *const wined3d_uvec4,
                    bool,
                ) -> (),
        ),
        adapter_generate_mipmap: Some(
            adapter_gl_generate_mipmap
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *mut wined3d_shader_resource_view,
                ) -> (),
        ),
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(d3d) / WINE_DECLARE_DEBUG_CHANNEL(...): function-like macros, emitted by the caller.

pub const GL_VENDOR_UNKNOWN: wined3d_gl_vendor = 0;
pub const GL_VENDOR_APPLE: wined3d_gl_vendor = 1;
pub const GL_VENDOR_FGLRX: wined3d_gl_vendor = 2;
pub const GL_VENDOR_MESA: wined3d_gl_vendor = 3;
pub const GL_VENDOR_NVIDIA: wined3d_gl_vendor = 4;

#[repr(C)]
pub struct wined3d_extension_map {
    pub extension_string: *const i8,
    pub extension: wined3d_gl_extension,
}
pub type PWined3d_extension_map = *mut wined3d_extension_map;

// gl_extension_map, wgl_extension_map: c2rust file-scope data, not emitted here.

#[repr(C)]
pub struct wined3d_renderer_table {
    pub renderer: *const i8,
    pub id: wined3d_pci_device,
}
pub type PWined3d_renderer_table = *mut wined3d_renderer_table;

unsafe impl Sync for wined3d_renderer_table {}

// cards_amd_binary: c2rust file-scope data, not emitted here.

#[repr(C)]
pub struct gl_vendor_selection {
    pub gl_vendor: wined3d_gl_vendor,
    pub description: *const i8,
    pub cards: *const wined3d_renderer_table,
    pub cards_size: usize,
}
pub type Pgl_vendor_selection = *mut gl_vendor_selection;

unsafe impl Sync for gl_vendor_selection {}

// amd_gl_vendor_table: c2rust file-scope data, not emitted here.
// card_vendor_table (anonymous struct): c2rust file-scope data, not emitted here.

#[repr(C)]
pub struct wined3d_view_gl_destroy_ctx {
    pub device: *mut wined3d_device,
    pub gl_view: *const wined3d_gl_view,
    pub bo_user: *mut wined3d_bo_user,
    pub counter_bo: *mut wined3d_bo_gl,
    pub object: *mut core::ffi::c_void,
    pub free: *mut wined3d_view_gl_destroy_ctx,
}
pub type Pwined3d_view_gl_destroy_ctx = *mut wined3d_view_gl_destroy_ctx;

// wined3d_adapter_gl_ops: c2rust file-scope data, not emitted here.

unsafe extern "C" fn wined3d_caps_gl_ctx_destroy(ctx: *const wined3d_caps_gl_ctx) {
    let gl_info: *const wined3d_gl_info = (*ctx).gl_info;

    /* Both glDeleteProgram and glDeleteBuffers silently ignore 0 IDs but
     * this function might be called before the relevant function pointers
     * in gl_info are initialized. */
    if (*ctx).test_program_id != 0 || (*ctx).test_vbo != 0 {
        ((*gl_info).gl_ops.ext.p_glDeleteProgram.unwrap_unchecked())((*ctx).test_program_id);
        ((*gl_info).gl_ops.ext.p_glDeleteBuffers.unwrap_unchecked())(
            1 as GLsizei,
            core::ptr::addr_of!((*ctx).test_vbo),
        );
    }

    if wglMakeCurrent(core::ptr::null_mut(), core::ptr::null_mut()) == 0 {
        ERR!("Failed to disable caps GL context.\n");
    }

    if wglDeleteContext((*ctx).gl_ctx) == 0 {
        let err: u32 = GetLastError() as u32;
        ERR!("wglDeleteContext(%p) failed, last error %#x.\n", (*ctx).gl_ctx, err);
    }

    wined3d_release_dc((*ctx).wnd, (*ctx).dc);
    DestroyWindow((*ctx).wnd);

    if !(*ctx).restore_gl_ctx.is_null()
        && wglMakeCurrent((*ctx).restore_dc, (*ctx).restore_gl_ctx) == 0
    {
        ERR!("Failed to restore previous GL context.\n");
    }
}

unsafe extern "C" fn wined3d_caps_gl_ctx_create_attribs(
    caps_gl_ctx: *mut wined3d_caps_gl_ctx,
    gl_info: *mut wined3d_gl_info,
) -> BOOL {
    let mut new_ctx: HGLRC;

    (*gl_info).p_wglCreateContextAttribsARB =
        core::mem::transmute::<
            *mut core::ffi::c_void,
            Option<unsafe extern "system" fn(HDC, HGLRC, *const i32) -> HGLRC>,
        >(core::mem::transmute::<PROC, *mut core::ffi::c_void>(
            wglGetProcAddress(b"wglCreateContextAttribsARB\0".as_ptr() as LPCSTR),
        ));
    if (*gl_info).p_wglCreateContextAttribsARB.is_none() {
        return (TRUE) as BOOL;
    }

    new_ctx = context_create_wgl_attribs(gl_info, (*caps_gl_ctx).dc, core::ptr::null_mut());
    if new_ctx.is_null() {
        (*gl_info).p_wglCreateContextAttribsARB = None;
        return (FALSE) as BOOL;
    }

    if wglMakeCurrent((*caps_gl_ctx).dc, new_ctx) == 0 {
        ERR!(
            "Failed to make new context current, last error %#lx.\n",
            GetLastError()
        );
        if wglDeleteContext(new_ctx) == 0 {
            ERR!(
                "Failed to delete new context, last error %#lx.\n",
                GetLastError()
            );
        }
        (*gl_info).p_wglCreateContextAttribsARB = None;
        return (TRUE) as BOOL;
    }

    if wglDeleteContext((*caps_gl_ctx).gl_ctx) == 0 {
        ERR!(
            "Failed to delete old context, last error %#lx.\n",
            GetLastError()
        );
    }
    (*caps_gl_ctx).gl_ctx = new_ctx;

    return (TRUE) as BOOL;
}

unsafe extern "C" fn wined3d_caps_gl_ctx_create(
    adapter_gl: *mut wined3d_adapter_gl,
    ctx: *mut wined3d_caps_gl_ctx,
) -> BOOL {
    let mut pfd: PIXELFORMATDESCRIPTOR = core::mem::zeroed();
    let mut iPixelFormat: core::ffi::c_int;

    (*ctx).restore_dc = wglGetCurrentDC();
    (*ctx).restore_gl_ctx = wglGetCurrentContext();

    /* We need a fake window as a hdc retrieved using GetDC(0) can't be used for much GL purposes. */
    (*ctx).wnd = CreateWindowA!(
        WINED3D_OPENGL_WINDOW_CLASS_NAME.as_ptr(),
        b"WineD3D fake window\0".as_ptr() as *const i8,
        WS_OVERLAPPEDWINDOW,
        10,
        10,
        10,
        10,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    'fail: {
        if (*ctx).wnd.is_null() {
            ERR!("Failed to create a window.\n");
            break 'fail;
        }

        (*ctx).dc = GetDC((*ctx).wnd);
        if (*ctx).dc.is_null() {
            ERR!("Failed to get a DC.\n");
            break 'fail;
        }

        /* PixelFormat selection */
        ZeroMemory!(
            &raw mut pfd as *mut core::ffi::c_void,
            core::mem::size_of::<PIXELFORMATDESCRIPTOR>(),
        );
        pfd.nSize = core::mem::size_of::<PIXELFORMATDESCRIPTOR>() as WORD;
        pfd.nVersion = 1 as WORD;
        pfd.dwFlags = (PFD_SUPPORT_OPENGL as DWORD)
            | (PFD_DOUBLEBUFFER as DWORD)
            | (PFD_DRAW_TO_WINDOW as DWORD); /* PFD_GENERIC_ACCELERATED */
        pfd.iPixelType = PFD_TYPE_RGBA as BYTE;
        pfd.cColorBits = 32 as BYTE;
        pfd.iLayerType = PFD_MAIN_PLANE as BYTE;

        iPixelFormat = ChoosePixelFormat((*ctx).dc, &raw mut pfd);
        if iPixelFormat == 0 {
            /* If this happens something is very wrong as ChoosePixelFormat barely fails. */
            ERR!("Failed to find a suitable pixel format.\n");
            break 'fail;
        }
        DescribePixelFormat(
            (*ctx).dc,
            iPixelFormat,
            core::mem::size_of::<PIXELFORMATDESCRIPTOR>() as UINT,
            &raw mut pfd,
        );
        SetPixelFormat((*ctx).dc, iPixelFormat, &raw mut pfd);

        /* Create a GL context. */
        (*ctx).gl_ctx = wglCreateContext((*ctx).dc);
        if (*ctx).gl_ctx.is_null() {
            WARN!("Failed to create default context for capabilities initialization.\n");
            break 'fail;
        }

        /* Make it the current GL context. */
        if wglMakeCurrent((*ctx).dc, (*ctx).gl_ctx) == 0 {
            ERR!("Failed to make caps GL context current.\n");
            break 'fail;
        }

        (*ctx).gl_info = &raw mut (*adapter_gl).gl_info;
        return (TRUE) as BOOL;
    }

    // fail:
    if !(*ctx).gl_ctx.is_null() {
        wglDeleteContext((*ctx).gl_ctx);
    }
    (*ctx).gl_ctx = core::ptr::null_mut();
    if !(*ctx).dc.is_null() {
        ReleaseDC((*ctx).wnd, (*ctx).dc);
    }
    (*ctx).dc = core::ptr::null_mut();
    if !(*ctx).wnd.is_null() {
        DestroyWindow((*ctx).wnd);
    }
    (*ctx).wnd = core::ptr::null_mut();
    if !(*ctx).restore_gl_ctx.is_null()
        && wglMakeCurrent((*ctx).restore_dc, (*ctx).restore_gl_ctx) == 0
    {
        ERR!("Failed to restore previous GL context.\n");
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn test_arb_vs_offset_limit(gl_info: *const wined3d_gl_info) -> BOOL {
    let mut prog: GLuint = 0;
    let mut ret: BOOL = (FALSE as i32);
    let testcode: [u8; 153] = *b"!!ARBvp1.0\nPARAM C[66] = { program.env[0..65] };\nADDRESS A0;PARAM zero = {0.0, 0.0, 0.0, 0.0};\nARL A0.x, zero.x;\nMOV result.position, C[A0.x + 65];\nEND\n\0";

    while (*gl_info).gl_ops.gl.p_glGetError.unwrap_unchecked()() != 0 {}
    ((*gl_info).gl_ops.ext.p_glGenProgramsARB.unwrap_unchecked())(1, &raw mut prog);
    if prog == 0 {
        ERR!("Failed to create an ARB offset limit test program\n");
    }
    ((*gl_info).gl_ops.ext.p_glBindProgramARB.unwrap_unchecked())(GL_VERTEX_PROGRAM_ARB, prog);
    ((*gl_info).gl_ops.ext.p_glProgramStringARB.unwrap_unchecked())(
        GL_VERTEX_PROGRAM_ARB,
        GL_PROGRAM_FORMAT_ASCII_ARB,
        (strlen(&raw const testcode as *const ::core::ffi::c_char)) as i32,
        &raw const testcode as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
    );
    if (*gl_info).gl_ops.gl.p_glGetError.unwrap_unchecked()() != 0 {
        ret = (TRUE as i32);
    } else {
        TRACE!("OpenGL implementation allows offsets > 63\n");
    }

    ((*gl_info).gl_ops.ext.p_glBindProgramARB.unwrap_unchecked())(GL_VERTEX_PROGRAM_ARB, 0);
    ((*gl_info).gl_ops.ext.p_glDeleteProgramsARB.unwrap_unchecked())(1, &raw mut prog);
    checkGLcall!(gl_info, "ARB vp offset limit test cleanup");

    return ret;
}

unsafe extern "C" fn match_amd_r300_to_500(
    gl_info: *const wined3d_gl_info,
    ctx: *mut wined3d_caps_gl_ctx,
    gl_renderer: *const ::core::ffi::c_char,
    gl_vendor: wined3d_gl_vendor,
    card_vendor: wined3d_pci_vendor,
    device: wined3d_pci_device,
) -> BOOL {
    return (card_vendor as ::core::ffi::c_uint == HW_VENDOR_AMD as ::core::ffi::c_uint
        && (device as ::core::ffi::c_uint == CARD_AMD_RADEON_9500 as ::core::ffi::c_uint
            || device as ::core::ffi::c_uint == CARD_AMD_RADEON_X700 as ::core::ffi::c_uint
            || device as ::core::ffi::c_uint == CARD_AMD_RADEON_X1600 as ::core::ffi::c_uint))
        as ::core::ffi::c_int;
}

unsafe extern "C" fn match_geforce5(
    gl_info: *const wined3d_gl_info,
    ctx: *mut wined3d_caps_gl_ctx,
    gl_renderer: *const ::core::ffi::c_char,
    gl_vendor: wined3d_gl_vendor,
    card_vendor: wined3d_pci_vendor,
    device: wined3d_pci_device,
) -> BOOL {
    return (card_vendor as ::core::ffi::c_uint == HW_VENDOR_NVIDIA as ::core::ffi::c_uint
        && (device as ::core::ffi::c_uint == CARD_NVIDIA_GEFORCEFX_5200 as ::core::ffi::c_uint
            || device as ::core::ffi::c_uint == CARD_NVIDIA_GEFORCEFX_5600 as ::core::ffi::c_uint
            || device as ::core::ffi::c_uint == CARD_NVIDIA_GEFORCEFX_5800 as ::core::ffi::c_uint))
        as BOOL;
}

unsafe extern "C" fn match_apple(
    gl_info: *const wined3d_gl_info,
    ctx: *mut wined3d_caps_gl_ctx,
    gl_renderer: *const ::core::ffi::c_char,
    gl_vendor: wined3d_gl_vendor,
    card_vendor: wined3d_pci_vendor,
    device: wined3d_pci_device,
) -> BOOL {
    return (gl_vendor as ::core::ffi::c_uint
        == GL_VENDOR_APPLE as ::core::ffi::c_uint) as BOOL;
}

unsafe extern "C" fn test_pbo_functionality(gl_info: *mut wined3d_gl_info) {
    let mut texture: GLuint = 0;
    let mut pbo: GLuint = 0;
    static pattern: [c_uint; 16] = [
        0x00000000 as c_uint, 0x000000ff as c_uint,
        0x0000ff00 as c_uint, 0x40ff0000 as c_uint,
        0x80ffffff as c_uint, 0x40ffff00 as c_uint,
        0x00ff00ff as c_uint, 0x0000ffff as c_uint,
        0x00ffff00 as c_uint, 0x00ff00ff as c_uint,
        0x0000ffff as c_uint, 0x000000ff as c_uint,
        0x80ff00ff as c_uint, 0x0000ffff as c_uint,
        0x00ff00ff as c_uint, 0x40ff00ff as c_uint,
    ];
    let mut check: [c_uint; 16] = [0; 16];

    /* No PBO -> No point in testing them. */
    if (*gl_info).supported[ARB_PIXEL_BUFFER_OBJECT as usize] == 0 {
        return;
    }

    while (*gl_info).gl_ops.gl.p_glGetError.unwrap_unchecked()() != 0 {}
    (*gl_info).gl_ops.gl.p_glGenTextures.unwrap_unchecked()(
        1 as GLsizei,
        &raw mut texture,
    );
    (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        texture,
    );

    (*gl_info).gl_ops.gl.p_glTexParameteri.unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        GL_TEXTURE_MAX_LEVEL as GLenum,
        0 as GLint,
    );
    (*gl_info).gl_ops.gl.p_glTexImage2D.unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        0 as GLint,
        GL_RGBA8 as GLint,
        4 as GLsizei,
        4 as GLsizei,
        0 as GLint,
        GL_BGRA as GLenum,
        GL_UNSIGNED_INT_8_8_8_8_REV as GLenum,
        0 as *const c_void,
    );
    checkGLcall!(gl_info, b"Specifying the PBO test texture\0".as_ptr() as *const c_char);

    ((*gl_info).gl_ops.ext.p_glGenBuffers.unwrap_unchecked())(1 as GLsizei, &raw mut pbo);
    ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(GL_PIXEL_UNPACK_BUFFER as GLenum, pbo);
    ((*gl_info).gl_ops.ext.p_glBufferData.unwrap_unchecked())(
        GL_PIXEL_UNPACK_BUFFER as GLenum,
        core::mem::size_of::<[c_uint; 16]>() as GLsizeiptr,
        &raw const pattern as *const c_uint as *const c_void,
        GL_STREAM_DRAW as GLenum,
    );
    checkGLcall!(gl_info, b"Specifying the PBO test pbo\0".as_ptr() as *const c_char);

    (*gl_info).gl_ops.gl.p_glTexSubImage2D.unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        0 as GLint,
        0 as GLint,
        0 as GLint,
        4 as GLsizei,
        4 as GLsizei,
        GL_BGRA as GLenum,
        GL_UNSIGNED_INT_8_8_8_8_REV as GLenum,
        core::ptr::null::<c_void>(),
    );
    checkGLcall!(gl_info, b"Loading the PBO test texture\0".as_ptr() as *const c_char);

    ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(
        GL_PIXEL_UNPACK_BUFFER as GLenum,
        0 as GLuint,
    );

    (*gl_info).gl_ops.gl.p_glFinish.unwrap_unchecked()(); /* just to be sure */

    memset(
        &raw mut check as *mut c_uint as *mut c_void,
        0 as c_int,
        core::mem::size_of::<[c_uint; 16]>() as size_t,
    );
    (*gl_info).gl_ops.gl.p_glGetTexImage.unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        0 as GLint,
        GL_BGRA as GLenum,
        GL_UNSIGNED_INT_8_8_8_8_REV as GLenum,
        &raw mut check as *mut c_uint as *mut c_void,
    );
    checkGLcall!(gl_info, b"Reading back the PBO test texture\0".as_ptr() as *const c_char);

    (*gl_info).gl_ops.gl.p_glDeleteTextures.unwrap_unchecked()(
        1 as GLsizei,
        &raw mut texture,
    );
    ((*gl_info).gl_ops.ext.p_glDeleteBuffers.unwrap_unchecked())(1 as GLsizei, &raw mut pbo);
    checkGLcall!(gl_info, b"PBO test cleanup\0".as_ptr() as *const c_char);

    if memcmp(
        &raw mut check as *mut c_uint as *const c_void,
        &raw const pattern as *const c_uint as *const c_void,
        core::mem::size_of::<[c_uint; 16]>() as size_t,
    ) != 0
    {
        WINE_WARN_!(__wine_dbch_d3d_perf,
            b"PBO test failed, read back data doesn't match original.\nDisabling PBOs. This may result in slower performance.\n\0"
                .as_ptr() as *const c_char,
        );
        (*gl_info).supported[ARB_PIXEL_BUFFER_OBJECT as usize] = FALSE as BOOL;
    } else { WINE_TRACE!("PBO test successful.\n"); }
}

unsafe extern "C" fn match_dx10_capable(
    gl_info: *const wined3d_gl_info,
    ctx: *mut wined3d_caps_gl_ctx,
    gl_renderer: *const ::core::ffi::c_char,
    gl_vendor: wined3d_gl_vendor,
    card_vendor: wined3d_pci_vendor,
    device: wined3d_pci_device,
) -> BOOL {
    if (*gl_info).limits.glsl_varyings > 44 as ::core::ffi::c_uint {
        return 1;
    } else {
        return 0;
    }
}

unsafe extern "C" fn match_not_dx10_capable(
    gl_info: *const wined3d_gl_info,
    ctx: *mut wined3d_caps_gl_ctx,
    gl_renderer: *const ::core::ffi::c_char,
    gl_vendor: wined3d_gl_vendor,
    card_vendor: wined3d_pci_vendor,
    device: wined3d_pci_device,
) -> BOOL {
    return (match_dx10_capable(gl_info, ctx, gl_renderer, gl_vendor, card_vendor, device) == 0) as BOOL;
}

unsafe extern "C" fn match_allows_spec_alpha(
    gl_info: *const wined3d_gl_info,
    ctx: *mut wined3d_caps_gl_ctx,
    gl_renderer: *const ::core::ffi::c_char,
    gl_vendor: wined3d_gl_vendor,
    card_vendor: wined3d_pci_vendor,
    device: wined3d_pci_device,
) -> BOOL {
    let mut error: GLenum = 0;
    let mut data: [DWORD; 16] = [0; 16];

    if (*gl_info).supported[EXT_SECONDARY_COLOR as usize] == 0
        || (*gl_info).supported[WINED3D_GL_LEGACY_CONTEXT as usize] == 0
    {
        return (FALSE) as BOOL;
    }

    while (*gl_info)
        .gl_ops
        .gl
        .p_glGetError
        .unwrap_unchecked()()
        != 0
    {}

    (*gl_info)
        .gl_ops
        .ext
        .p_glSecondaryColorPointerEXT
        .unwrap_unchecked()(
        4 as GLint,
        GL_UNSIGNED_BYTE as GLenum,
        4 as GLsizei,
        &raw mut data as *mut DWORD as *const ::core::ffi::c_void,
    );

    error = (*gl_info)
        .gl_ops
        .gl
        .p_glGetError
        .unwrap_unchecked()();

    if error == GL_NO_ERROR as GLenum {
        WINE_TRACE!("GL Implementation accepts 4 component specular color pointers\n");
        return (TRUE) as BOOL;
    } else {
        WINE_TRACE!("GL implementation does not accept 4 component specular colors, error %s\n", debug_glerror(error));
        return (FALSE) as BOOL;
    }
}

unsafe extern "C" fn match_broken_nv_clip(
    gl_info: *const wined3d_gl_info,
    ctx: *mut wined3d_caps_gl_ctx,
    gl_renderer: *const ::core::ffi::c_char,
    gl_vendor: wined3d_gl_vendor,
    card_vendor: wined3d_pci_vendor,
    device: wined3d_pci_device,
) -> BOOL {
    let mut prog: GLuint = 0;
    let mut ret: BOOL = (FALSE as i32);
    let mut pos: GLint = 0;
    let testcode: *const ::core::ffi::c_char =
        b"!!ARBvp1.0\nOPTION NV_vertex_program2;\nMOV result.clip[0], 0.0;\nMOV result.position, 0.0;\nEND\n\0"
            .as_ptr() as *const ::core::ffi::c_char;

    if (*gl_info).supported[NV_VERTEX_PROGRAM2_OPTION as usize] == 0 {
        return (FALSE) as BOOL;
    }

    while (*gl_info).gl_ops.gl.p_glGetError.unwrap_unchecked()() != 0 {}

    ((*gl_info).gl_ops.ext.p_glGenProgramsARB.unwrap_unchecked())(
        1,
        core::ptr::addr_of_mut!(prog),
    );
    if prog == 0 {
        ERR!("Failed to create the NVvp clip test program\n");
        return (FALSE) as BOOL;
    }
    ((*gl_info).gl_ops.ext.p_glBindProgramARB.unwrap_unchecked())(GL_VERTEX_PROGRAM_ARB, prog);
    ((*gl_info).gl_ops.ext.p_glProgramStringARB.unwrap_unchecked())(
        GL_VERTEX_PROGRAM_ARB,
        GL_PROGRAM_FORMAT_ASCII_ARB,
        strlen(testcode) as GLsizei,
        testcode as *const ::core::ffi::c_void,
    );
    (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
        GL_PROGRAM_ERROR_POSITION_ARB,
        core::ptr::addr_of_mut!(pos),
    );
    if pos != -(1) {
        WARN!("GL_NV_vertex_program2_option result.clip[] test failed\n");

        ret = (TRUE as i32);
        while (*gl_info).gl_ops.gl.p_glGetError.unwrap_unchecked()() != 0 {}
    } else {
        TRACE!("GL_NV_vertex_program2_option result.clip[] test passed\n");
    }

    ((*gl_info).gl_ops.ext.p_glBindProgramARB.unwrap_unchecked())(GL_VERTEX_PROGRAM_ARB, 0);
    ((*gl_info).gl_ops.ext.p_glDeleteProgramsARB.unwrap_unchecked())(
        1,
        core::ptr::addr_of_mut!(prog),
    );
    checkGLcall!(gl_info, "GL_NV_vertex_program2_option result.clip[] test cleanup");

    return ret;
}

unsafe extern "C" fn match_fbo_tex_update(
    mut gl_info: *const wined3d_gl_info,
    mut ctx: *mut wined3d_caps_gl_ctx,
    mut gl_renderer: *const ::core::ffi::c_char,
    mut gl_vendor: wined3d_gl_vendor,
    mut card_vendor: wined3d_pci_vendor,
    mut device: wined3d_pci_device,
) -> BOOL {
    let mut data: [::core::ffi::c_char; 4 * 4 * 4] =
        [0; 4 * 4 * 4];
    let mut tex: GLuint = 0;
    let mut fbo: GLuint = 0;
    let mut status: GLenum = 0;

    memset(
        &raw mut data as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        0xcc as ::core::ffi::c_int,
        ::core::mem::size_of::<[::core::ffi::c_char; 4 * 4 * 4]>() as size_t,
    );

    (*gl_info)
        .gl_ops
        .gl
        .p_glGenTextures
        .unwrap_unchecked()(1 as GLsizei, &raw mut tex);
    (*gl_info)
        .gl_ops
        .gl
        .p_glBindTexture
        .unwrap_unchecked()(GL_TEXTURE_2D as GLenum, tex);
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        GL_TEXTURE_MIN_FILTER as GLenum,
        (GL_NEAREST as i32),
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        GL_TEXTURE_MAG_FILTER as GLenum,
        (GL_NEAREST as i32),
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexImage2D
        .unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        0 as GLint,
        GL_RGBA8 as GLint,
        4 as GLsizei,
        4 as GLsizei,
        0 as GLint,
        GL_BGRA as GLenum,
        GL_UNSIGNED_INT_8_8_8_8_REV as GLenum,
        ::core::ptr::null::<::core::ffi::c_void>(),
    );
    checkGLcall!(gl_info, "glTexImage2D");

    (*gl_info)
        .fbo_ops
        .glGenFramebuffers
        .unwrap_unchecked()(1 as GLsizei, &raw mut fbo);
    (*gl_info)
        .fbo_ops
        .glBindFramebuffer
        .unwrap_unchecked()(GL_FRAMEBUFFER as GLenum, fbo);
    (*gl_info)
        .fbo_ops
        .glFramebufferTexture2D
        .unwrap_unchecked()(
        GL_FRAMEBUFFER as GLenum,
        GL_COLOR_ATTACHMENT0 as GLenum,
        GL_TEXTURE_2D as GLenum,
        tex,
        0 as GLint,
    );
    checkGLcall!(gl_info, "glFramebufferTexture2D");

    status = (*gl_info)
        .fbo_ops
        .glCheckFramebufferStatus
        .unwrap_unchecked()(GL_FRAMEBUFFER as GLenum);
    if status != GL_FRAMEBUFFER_COMPLETE {
        ERR!("FBO status %#x\n", status);
    }
    checkGLcall!(gl_info, "glCheckFramebufferStatus");

    memset(
        &raw mut data as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        0x11 as ::core::ffi::c_int,
        ::core::mem::size_of::<[::core::ffi::c_char; 4 * 4 * 4]>() as size_t,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexSubImage2D
        .unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        0 as GLint,
        0 as GLint,
        0 as GLint,
        4 as GLsizei,
        4 as GLsizei,
        GL_BGRA as GLenum,
        GL_UNSIGNED_INT_8_8_8_8_REV as GLenum,
        &raw mut data as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
    );
    checkGLcall!(gl_info, "glTexSubImage2D");

    (*gl_info)
        .gl_ops
        .gl
        .p_glClearColor
        .unwrap_unchecked()(0.996f32, 0.729f32, 0.745f32, 0.792f32);
    (*gl_info)
        .gl_ops
        .gl
        .p_glClear
        .unwrap_unchecked()(GL_COLOR_BUFFER_BIT as GLbitfield);
    checkGLcall!(gl_info, "glClear");

    (*gl_info)
        .gl_ops
        .gl
        .p_glGetTexImage
        .unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        0 as GLint,
        GL_BGRA as GLenum,
        GL_UNSIGNED_INT_8_8_8_8_REV as GLenum,
        &raw mut data as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
    );
    checkGLcall!(gl_info, "glGetTexImage");

    (*gl_info)
        .fbo_ops
        .glFramebufferTexture2D
        .unwrap_unchecked()(
        GL_FRAMEBUFFER as GLenum,
        GL_COLOR_ATTACHMENT0 as GLenum,
        GL_TEXTURE_2D as GLenum,
        0 as GLuint,
        0 as GLint,
    );
    (*gl_info)
        .fbo_ops
        .glBindFramebuffer
        .unwrap_unchecked()(GL_FRAMEBUFFER as GLenum, 0 as GLuint);
    (*gl_info)
        .gl_ops
        .gl
        .p_glBindTexture
        .unwrap_unchecked()(GL_TEXTURE_2D as GLenum, 0 as GLuint);
    checkGLcall!(gl_info, "glBindTexture");

    (*gl_info)
        .fbo_ops
        .glDeleteFramebuffers
        .unwrap_unchecked()(1 as GLsizei, &raw mut fbo);
    (*gl_info)
        .gl_ops
        .gl
        .p_glDeleteTextures
        .unwrap_unchecked()(1 as GLsizei, &raw mut tex);
    checkGLcall!(gl_info, "glDeleteTextures");

    return (*(&raw mut data as *mut ::core::ffi::c_char as *mut DWORD)
        == 0x11111111 as DWORD) as BOOL;
}

unsafe extern "C" fn match_broken_rgba16(
    gl_info: *const wined3d_gl_info,
    ctx: *mut wined3d_caps_gl_ctx,
    gl_renderer: *const core::ffi::c_char,
    gl_vendor: wined3d_gl_vendor,
    card_vendor: wined3d_pci_vendor,
    device: wined3d_pci_device,
) -> BOOL {
    /* GL_RGBA16 uses GL_RGBA8 internally on Geforce 7 and older cards.
     * This leads to graphical bugs in Half Life 2 and Unreal engine games. */
    let mut tex: GLuint = 0;
    let mut size: GLint = 0;

    (*gl_info).gl_ops.gl.p_glGenTextures.unwrap_unchecked()(
        1 as GLsizei,
        core::ptr::addr_of_mut!(tex),
    );
    (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        tex,
    );
    (*gl_info).gl_ops.gl.p_glTexImage2D.unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        0 as GLint,
        GL_RGBA16 as GLint,
        4 as GLsizei,
        4 as GLsizei,
        0 as GLint,
        GL_RGBA as GLenum,
        GL_UNSIGNED_SHORT as GLenum,
        core::ptr::null::<core::ffi::c_void>(),
    );
    checkGLcall!(gl_info, b"glTexImage2D\0".as_ptr() as *const core::ffi::c_char);

    (*gl_info).gl_ops.gl.p_glGetTexLevelParameteriv.unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        0 as GLint,
        GL_TEXTURE_RED_SIZE as GLenum,
        core::ptr::addr_of_mut!(size),
    );
    checkGLcall!(gl_info, b"glGetTexLevelParameteriv\0".as_ptr() as *const core::ffi::c_char);

    (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        0 as GLuint,
    );
    checkGLcall!(gl_info, b"glBindTexture\0".as_ptr() as *const core::ffi::c_char);
    (*gl_info).gl_ops.gl.p_glDeleteTextures.unwrap_unchecked()(
        1 as GLsizei,
        core::ptr::addr_of_mut!(tex),
    );
    checkGLcall!(gl_info, b"glDeleteTextures\0".as_ptr() as *const core::ffi::c_char);

    return (size < 16) as BOOL;
}

unsafe extern "C" fn match_fglrx(
    gl_info: *const wined3d_gl_info,
    ctx: *mut wined3d_caps_gl_ctx,
    gl_renderer: *const ::core::ffi::c_char,
    gl_vendor: wined3d_gl_vendor,
    card_vendor: wined3d_pci_vendor,
    device: wined3d_pci_device,
) -> BOOL {
    return (gl_vendor as ::core::ffi::c_uint == GL_VENDOR_FGLRX as ::core::ffi::c_uint) as BOOL;
}

unsafe extern "C" fn match_r200(
    mut gl_info: *const wined3d_gl_info,
    mut ctx: *mut wined3d_caps_gl_ctx,
    mut gl_renderer: *const ::core::ffi::c_char,
    mut gl_vendor: wined3d_gl_vendor,
    mut card_vendor: wined3d_pci_vendor,
    mut device: wined3d_pci_device,
) -> BOOL {
    if card_vendor as ::core::ffi::c_uint
        != HW_VENDOR_AMD as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return FALSE as BOOL;
    }
    if device as ::core::ffi::c_uint
        == CARD_AMD_RADEON_8500 as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return TRUE as BOOL;
    }
    return FALSE as BOOL;
}

unsafe extern "C" fn match_broken_arb_fog(
    gl_info: *const wined3d_gl_info,
    ctx: *mut wined3d_caps_gl_ctx,
    gl_renderer: *const ::core::ffi::c_char,
    gl_vendor: wined3d_gl_vendor,
    card_vendor: wined3d_pci_vendor,
    device: wined3d_pci_device,
) -> BOOL {
    let mut data: [::core::ffi::c_uint; 4] = [0; 4];
    let mut tex: GLuint = 0;
    let mut fbo: GLuint = 0;
    let mut status: GLenum = 0;
    let mut color: [::core::ffi::c_float; 4] = [0.0f32, 1.0f32, 0.0f32, 0.0f32];
    let mut prog: GLuint = 0;
    let mut err_pos: GLint = 0;
    static mut program_code: [::core::ffi::c_char; 79] = unsafe {
        ::core::mem::transmute::<[u8; 79], [::core::ffi::c_char; 79]>(
            *b"!!ARBfp1.0\nOPTION ARB_fog_linear;\nMOV result.color, {1.0, 0.0, 0.0, 0.0};\nEND\n\0",
        )
    };

    if (*gl_info).supported[ARB_FRAGMENT_PROGRAM as usize] == 0 {
        return (FALSE) as BOOL;
    }
    if (*gl_info).supported[WINED3D_GL_LEGACY_CONTEXT as usize] == 0 {
        return (FALSE) as BOOL;
    }

    (*gl_info)
        .gl_ops
        .gl
        .p_glGenTextures
        .unwrap_unchecked()(1 as GLsizei, &raw mut tex);
    (*gl_info)
        .gl_ops
        .gl
        .p_glBindTexture
        .unwrap_unchecked()(GL_TEXTURE_2D as GLenum, tex);
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        GL_TEXTURE_MIN_FILTER as GLenum,
        GL_NEAREST as GLint,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        GL_TEXTURE_MAG_FILTER as GLenum,
        GL_NEAREST as GLint,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexImage2D
        .unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        0 as GLint,
        GL_RGB8 as GLint,
        4 as GLsizei,
        1 as GLsizei,
        0 as GLint,
        GL_BGRA as GLenum,
        GL_UNSIGNED_INT_8_8_8_8_REV as GLenum,
        ::core::ptr::null::<::core::ffi::c_void>(),
    );
    checkGLcall!(gl_info, b"glTexImage2D\0".as_ptr() as *const ::core::ffi::c_char);

    (*gl_info)
        .fbo_ops
        .glGenFramebuffers
        .unwrap_unchecked()(1 as GLsizei, &raw mut fbo);
    (*gl_info)
        .fbo_ops
        .glBindFramebuffer
        .unwrap_unchecked()(GL_FRAMEBUFFER as GLenum, fbo);
    (*gl_info)
        .fbo_ops
        .glFramebufferTexture2D
        .unwrap_unchecked()(
        GL_FRAMEBUFFER as GLenum,
        GL_COLOR_ATTACHMENT0 as GLenum,
        GL_TEXTURE_2D as GLenum,
        tex,
        0 as GLint,
    );
    checkGLcall!(gl_info, b"glFramebufferTexture2D\0".as_ptr() as *const ::core::ffi::c_char);

    status = (*gl_info)
        .fbo_ops
        .glCheckFramebufferStatus
        .unwrap_unchecked()(GL_FRAMEBUFFER as GLenum);
    if status != GL_FRAMEBUFFER_COMPLETE as GLenum {
        ERR!("FBO status %#x\n", status);
    }
    checkGLcall!(gl_info, b"glCheckFramebufferStatus\0".as_ptr() as *const ::core::ffi::c_char);

    (*gl_info)
        .gl_ops
        .gl
        .p_glClearColor
        .unwrap_unchecked()(0.0f32, 0.0f32, 1.0f32, 0.0f32);
    (*gl_info)
        .gl_ops
        .gl
        .p_glClear
        .unwrap_unchecked()(GL_COLOR_BUFFER_BIT as GLbitfield);
    checkGLcall!(gl_info, b"glClear\0".as_ptr() as *const ::core::ffi::c_char);
    (*gl_info)
        .gl_ops
        .gl
        .p_glViewport
        .unwrap_unchecked()(
        0 as GLint,
        0 as GLint,
        4 as GLsizei,
        1 as GLsizei,
    );
    checkGLcall!(gl_info, b"glViewport\0".as_ptr() as *const ::core::ffi::c_char);

    (*gl_info)
        .gl_ops
        .gl
        .p_glEnable
        .unwrap_unchecked()(GL_FOG as GLenum);
    (*gl_info)
        .gl_ops
        .gl
        .p_glFogf
        .unwrap_unchecked()(GL_FOG_START as GLenum, 0.5f32);
    (*gl_info)
        .gl_ops
        .gl
        .p_glFogf
        .unwrap_unchecked()(GL_FOG_END as GLenum, 0.5f32);
    (*gl_info)
        .gl_ops
        .gl
        .p_glFogi
        .unwrap_unchecked()(
        GL_FOG_MODE as GLenum,
        GL_LINEAR as GLint,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glHint
        .unwrap_unchecked()(
        GL_FOG_HINT as GLenum,
        GL_NICEST as GLenum,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glFogfv
        .unwrap_unchecked()(
        GL_FOG_COLOR as GLenum,
        ::core::ptr::addr_of_mut!(color) as *mut ::core::ffi::c_float,
    );
    checkGLcall!(gl_info, b"fog setup\0".as_ptr() as *const ::core::ffi::c_char);

    ((*gl_info).gl_ops.ext.p_glGenProgramsARB.unwrap_unchecked())(1 as GLsizei, &raw mut prog);
    ((*gl_info).gl_ops.ext.p_glBindProgramARB.unwrap_unchecked())(GL_FRAGMENT_PROGRAM_ARB as GLenum, prog);
    ((*gl_info).gl_ops.ext.p_glProgramStringARB.unwrap_unchecked())(
        GL_FRAGMENT_PROGRAM_ARB as GLenum,
        GL_PROGRAM_FORMAT_ASCII_ARB as GLenum,
        strlen(::core::ptr::addr_of!(program_code) as *const ::core::ffi::c_char) as GLsizei,
        ::core::ptr::addr_of!(program_code) as *const ::core::ffi::c_void,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glEnable
        .unwrap_unchecked()(GL_FRAGMENT_PROGRAM_ARB as GLenum);
    checkGLcall!(gl_info, b"Test fragment program setup\0".as_ptr() as *const ::core::ffi::c_char);

    (*gl_info)
        .gl_ops
        .gl
        .p_glGetIntegerv
        .unwrap_unchecked()(GL_PROGRAM_ERROR_POSITION_ARB as GLenum, &raw mut err_pos);
    if err_pos != -(1 as GLint) {
        let mut error_str: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        error_str = (*gl_info)
            .gl_ops
            .gl
            .p_glGetString
            .unwrap_unchecked()(GL_PROGRAM_ERROR_STRING_ARB as GLenum)
            as *const ::core::ffi::c_char;
        WINE_FIXME!("Fog test program error at position %d: %s\n\n", err_pos, debugstr_a(error_str));
    }

    (*gl_info)
        .gl_ops
        .gl
        .p_glBegin
        .unwrap_unchecked()(GL_TRIANGLE_STRIP as GLenum);
    (*gl_info)
        .gl_ops
        .gl
        .p_glVertex3f
        .unwrap_unchecked()(-1.0f32, -1.0f32, 0.0f32);
    (*gl_info)
        .gl_ops
        .gl
        .p_glVertex3f
        .unwrap_unchecked()(1.0f32, -1.0f32, 1.0f32);
    (*gl_info)
        .gl_ops
        .gl
        .p_glVertex3f
        .unwrap_unchecked()(-1.0f32, 1.0f32, 0.0f32);
    (*gl_info)
        .gl_ops
        .gl
        .p_glVertex3f
        .unwrap_unchecked()(1.0f32, 1.0f32, 1.0f32);
    (*gl_info)
        .gl_ops
        .gl
        .p_glEnd
        .unwrap_unchecked()();
    checkGLcall!(gl_info, b"ARBfp fog test draw\0".as_ptr() as *const ::core::ffi::c_char);

    (*gl_info)
        .gl_ops
        .gl
        .p_glGetTexImage
        .unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        0 as GLint,
        GL_BGRA as GLenum,
        GL_UNSIGNED_INT_8_8_8_8_REV as GLenum,
        ::core::ptr::addr_of_mut!(data) as *mut ::core::ffi::c_void,
    );
    checkGLcall!(gl_info, b"glGetTexImage\0".as_ptr() as *const ::core::ffi::c_char);
    data[0] &= 0x00ffffff;
    data[1] &= 0x00ffffff;
    data[2] &= 0x00ffffff;
    data[3] &= 0x00ffffff;

    (*gl_info)
        .fbo_ops
        .glBindFramebuffer
        .unwrap_unchecked()(GL_FRAMEBUFFER as GLenum, 0 as GLuint);
    (*gl_info)
        .gl_ops
        .gl
        .p_glBindTexture
        .unwrap_unchecked()(GL_TEXTURE_2D as GLenum, 0 as GLuint);

    (*gl_info)
        .fbo_ops
        .glDeleteFramebuffers
        .unwrap_unchecked()(1 as GLsizei, &raw mut fbo);
    (*gl_info)
        .gl_ops
        .gl
        .p_glDeleteTextures
        .unwrap_unchecked()(1 as GLsizei, &raw mut tex);
    (*gl_info)
        .gl_ops
        .gl
        .p_glDisable
        .unwrap_unchecked()(GL_FOG as GLenum);
    ((*gl_info).gl_ops.ext.p_glBindProgramARB.unwrap_unchecked())(GL_FRAGMENT_PROGRAM_ARB as GLenum, 0 as GLuint);
    (*gl_info)
        .gl_ops
        .gl
        .p_glDisable
        .unwrap_unchecked()(GL_FRAGMENT_PROGRAM_ARB as GLenum);
    ((*gl_info).gl_ops.ext.p_glDeleteProgramsARB.unwrap_unchecked())(1 as GLsizei, &raw mut prog);
    checkGLcall!(gl_info, b"ARBfp fog test teardown\0".as_ptr() as *const ::core::ffi::c_char);

    WINE_TRACE!("Fog test data: %08x %08x %08x %08x\n", data[0], data[3], data[3], data[3]);
    return (data[0] != 0x00ff0000
        || data[3] != 0x0000ff00) as ::core::ffi::c_int;
}

unsafe extern "C" fn match_broken_viewport_subpixel_bits(
    gl_info: *const wined3d_gl_info,
    ctx: *mut wined3d_caps_gl_ctx,
    gl_renderer: *const core::ffi::c_char,
    gl_vendor: wined3d_gl_vendor,
    card_vendor: wined3d_pci_vendor,
    device: wined3d_pci_device,
) -> BOOL {
    if (*gl_info).supported[ARB_VIEWPORT_ARRAY as usize] == 0 {
        return (FALSE) as BOOL;
    }
    return (wined3d_caps_gl_ctx_test_viewport_subpixel_bits(ctx) == 0) as i32;
}

unsafe extern "C" fn match_no_independent_bit_depths(
    gl_info: *const wined3d_gl_info,
    ctx: *mut wined3d_caps_gl_ctx,
    gl_renderer: *const ::core::ffi::c_char,
    gl_vendor: wined3d_gl_vendor,
    card_vendor: wined3d_pci_vendor,
    device: wined3d_pci_device,
) -> BOOL {
    let mut tex: [GLuint; 2] = [0; 2];
    let mut fbo: GLuint = 0;
    let mut status: GLenum = 0;

    /* ARB_framebuffer_object allows implementation-dependent internal format
     * restrictions. The EXT extension explicitly calls out an error in the
     * relevant case. */
    if (*gl_info).supported[(ARB_FRAMEBUFFER_OBJECT) as usize] == 0 {
        return (TRUE) as BOOL;
    }

    (*gl_info)
        .gl_ops
        .gl
        .p_glGenTextures
        .unwrap_unchecked()(2, &raw mut tex as *mut GLuint);

    (*gl_info)
        .gl_ops
        .gl
        .p_glBindTexture
        .unwrap_unchecked()(
        GL_TEXTURE_2D,
        tex[0],
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        GL_TEXTURE_2D,
        GL_TEXTURE_MIN_FILTER,
        (GL_NEAREST as i32),
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        GL_TEXTURE_2D,
        GL_TEXTURE_MAG_FILTER,
        (GL_NEAREST as i32),
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexImage2D
        .unwrap_unchecked()(
        GL_TEXTURE_2D,
        0,
        (GL_RGBA8 as i32),
        4,
        1,
        0,
        GL_RGBA,
        GL_UNSIGNED_INT_8_8_8_8,
        ::core::ptr::null::<::core::ffi::c_void>(),
    );

    (*gl_info)
        .gl_ops
        .gl
        .p_glBindTexture
        .unwrap_unchecked()(
        GL_TEXTURE_2D,
        tex[1],
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        GL_TEXTURE_2D,
        GL_TEXTURE_MIN_FILTER,
        (GL_NEAREST as i32),
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        GL_TEXTURE_2D,
        GL_TEXTURE_MAG_FILTER,
        (GL_NEAREST as i32),
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexImage2D
        .unwrap_unchecked()(
        GL_TEXTURE_2D,
        0,
        (GL_RGB5 as i32),
        4,
        1,
        0,
        GL_RGB,
        GL_UNSIGNED_SHORT_5_6_5,
        ::core::ptr::null::<::core::ffi::c_void>(),
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glBindTexture
        .unwrap_unchecked()(GL_TEXTURE_2D, 0);

    (*gl_info)
        .fbo_ops
        .glGenFramebuffers
        .unwrap_unchecked()(1, &raw mut fbo);
    (*gl_info)
        .fbo_ops
        .glBindFramebuffer
        .unwrap_unchecked()(GL_DRAW_FRAMEBUFFER, fbo);
    (*gl_info)
        .fbo_ops
        .glFramebufferTexture2D
        .unwrap_unchecked()(
        GL_DRAW_FRAMEBUFFER,
        GL_COLOR_ATTACHMENT0,
        GL_TEXTURE_2D,
        tex[0],
        0,
    );
    (*gl_info)
        .fbo_ops
        .glFramebufferTexture2D
        .unwrap_unchecked()(
        GL_DRAW_FRAMEBUFFER,
        GL_COLOR_ATTACHMENT1,
        GL_TEXTURE_2D,
        tex[1],
        0,
    );

    status = (*gl_info)
        .fbo_ops
        .glCheckFramebufferStatus
        .unwrap_unchecked()(GL_DRAW_FRAMEBUFFER);

    (*gl_info)
        .fbo_ops
        .glBindFramebuffer
        .unwrap_unchecked()(GL_DRAW_FRAMEBUFFER, 0);
    (*gl_info)
        .fbo_ops
        .glDeleteFramebuffers
        .unwrap_unchecked()(1, &raw mut fbo);
    (*gl_info)
        .gl_ops
        .gl
        .p_glDeleteTextures
        .unwrap_unchecked()(2, &raw mut tex as *mut GLuint);
    checkGLcall!(gl_info,
        b"testing multiple framebuffer attachments with different bit depths\0".as_ptr()
            as *const ::core::ffi::c_char,
    );

    return (status != GL_FRAMEBUFFER_COMPLETE) as ::core::ffi::c_int;
}

unsafe extern "C" fn quirk_apple_glsl_constants(gl_info: *mut wined3d_gl_info) {
    (*gl_info).reserved_glsl_constants = max!((*gl_info).reserved_glsl_constants, 12);
}

unsafe extern "C" fn quirk_amd_dx9(mut gl_info: *mut wined3d_gl_info) {
    if (*gl_info).supported[ARB_TEXTURE_NON_POWER_OF_TWO as usize] != 0 {
        WINE_TRACE!("GL_ARB_texture_non_power_of_two advertised on R500 or earlier card, removing.\n");
        (*gl_info).supported[WINED3D_GL_NORMALIZED_TEXRECT as usize] = TRUE as BOOL;
        (*gl_info).supported[ARB_TEXTURE_NON_POWER_OF_TWO as usize] = FALSE as BOOL;
    }
}

unsafe extern "C" fn quirk_no_np2(gl_info: *mut wined3d_gl_info) {
    (*gl_info).supported[ARB_TEXTURE_NON_POWER_OF_TWO as usize] = (FALSE as i32);
}

unsafe extern "C" fn quirk_clip_varying(gl_info: *mut wined3d_gl_info) {
    (*gl_info).quirks |= WINED3D_QUIRK_GLSL_CLIP_VARYING as u32;
}

unsafe extern "C" fn quirk_allows_specular_alpha(gl_info: *mut wined3d_gl_info) {
    (*gl_info).quirks |= WINED3D_QUIRK_ALLOWS_SPECULAR_ALPHA as u32;
}

unsafe extern "C" fn quirk_disable_nvvp_clip(gl_info: *mut wined3d_gl_info) {
    (*gl_info).quirks |= WINED3D_QUIRK_NV_CLIP_BROKEN;
}

unsafe extern "C" fn quirk_fbo_tex_update(gl_info: *mut wined3d_gl_info) {
    (*gl_info).quirks |= WINED3D_QUIRK_FBO_TEX_UPDATE as u32;
}

unsafe extern "C" fn quirk_broken_rgba16(gl_info: *mut wined3d_gl_info) {
    (*gl_info).quirks |= WINED3D_QUIRK_BROKEN_RGBA16 as u32;
}

unsafe extern "C" fn quirk_infolog_spam(gl_info: *mut wined3d_gl_info) {
    (*gl_info).quirks |= WINED3D_QUIRK_INFO_LOG_SPAM;
}

unsafe extern "C" fn quirk_limited_tex_filtering(gl_info: *mut wined3d_gl_info) {
    (*gl_info).quirks |= WINED3D_QUIRK_LIMITED_TEX_FILTERING;
}

unsafe extern "C" fn quirk_r200_constants(gl_info: *mut wined3d_gl_info) {
    (*gl_info).reserved_arb_constants = max!((*gl_info).reserved_arb_constants, 1);
}

unsafe extern "C" fn quirk_broken_arb_fog(gl_info: *mut wined3d_gl_info) {
    (*gl_info).quirks |= WINED3D_QUIRK_BROKEN_ARB_FOG as u32;
}

unsafe extern "C" fn quirk_broken_viewport_subpixel_bits(mut gl_info: *mut wined3d_gl_info) {
    (*gl_info).limits.viewport_subpixel_bits = 0 as ::core::ffi::c_uint;
    if (*gl_info).supported[ARB_CLIP_CONTROL as usize] != 0 {
        TRACE!("Disabling ARB_clip_control.\n");
        (*gl_info).supported[ARB_CLIP_CONTROL as usize] = FALSE as BOOL;
    }
}

unsafe extern "C" fn quirk_no_independent_bit_depths(gl_info: *mut wined3d_gl_info) {
    (*gl_info).quirks |= WINED3D_QUIRK_NO_INDEPENDENT_BIT_DEPTHS as u32;
}

unsafe extern "C" fn query_gpu_description(
    mut gl_info: *const wined3d_gl_info,
    mut vram_bytes: *mut UINT64,
) -> *const wined3d_gpu_description {
    let mut gpu_description: *const wined3d_gpu_description =
        ::core::ptr::null::<wined3d_gpu_description>();
    let mut gpu_description_override: *const wined3d_gpu_description =
        ::core::ptr::null::<wined3d_gpu_description>();
    let mut vendor: wined3d_pci_vendor = PCI_VENDOR_NONE as wined3d_pci_vendor;
    let mut device: wined3d_pci_device = PCI_DEVICE_NONE as wined3d_pci_device;
    let mut value: GLuint = 0;
    if (*gl_info).supported[WGL_WINE_QUERY_RENDERER as ::core::ffi::c_int as usize] != 0 {
        if (*gl_info)
            .gl_ops
            .ext
            .p_wglQueryCurrentRendererIntegerWINE
            .unwrap_unchecked()(
            WGL_RENDERER_VENDOR_ID_WINE as GLenum,
            &raw mut value,
        ) != 0
        {
            vendor = value as wined3d_pci_vendor;
        }
        if (*gl_info)
            .gl_ops
            .ext
            .p_wglQueryCurrentRendererIntegerWINE
            .unwrap_unchecked()(
            WGL_RENDERER_DEVICE_ID_WINE as GLenum,
            &raw mut value,
        ) != 0
        {
            device = value as wined3d_pci_device;
        }
        if (*gl_info)
            .gl_ops
            .ext
            .p_wglQueryCurrentRendererIntegerWINE
            .unwrap_unchecked()(
            WGL_RENDERER_VIDEO_MEMORY_WINE as GLenum,
            &raw mut value,
        ) != 0
        {
            *vram_bytes = (value as UINT64)
                .wrapping_mul(1024 as UINT64)
                .wrapping_mul(1024 as UINT64);
        }
        WINE_TRACE!("Card reports vendor PCI ID 0x%04x, device PCI ID 0x%04x, 0x%s bytes of video memory.\n", vendor as ::core::ffi::c_uint, device as ::core::ffi::c_uint, wine_dbgstr_longlong(*vram_bytes));
        gpu_description = wined3d_get_gpu_description(vendor, device);
    }
    gpu_description_override = wined3d_get_user_override_gpu_description(vendor, device);
    if !gpu_description_override.is_null() {
        gpu_description = gpu_description_override;
    }
    return gpu_description;
}

unsafe extern "C" fn fixup_extensions(
    gl_info: *mut wined3d_gl_info,
    ctx: *mut wined3d_caps_gl_ctx,
    gl_renderer: *const ::core::ffi::c_char,
    gl_vendor: wined3d_gl_vendor,
) {
    let card_vendor: wined3d_pci_vendor = (*(*ctx).gpu_description).vendor;
    let device: wined3d_pci_device = (*(*ctx).gpu_description).device;
    let mut i: ::core::ffi::c_uint;
    static mut quirk_table: [driver_quirk; 14] = unsafe {
        [
            driver_quirk {
                match_: Some(match_amd_r300_to_500),
                apply: Some(quirk_amd_dx9),
                description: b"AMD normalised texrect quirk\0".as_ptr() as *const ::core::ffi::c_char,
            },
            driver_quirk {
                match_: Some(match_apple),
                apply: Some(quirk_apple_glsl_constants),
                description: b"Apple GLSL uniform override\0".as_ptr() as *const ::core::ffi::c_char,
            },
            driver_quirk {
                match_: Some(match_geforce5),
                apply: Some(quirk_no_np2),
                description: b"Geforce 5 NP2 disable\0".as_ptr() as *const ::core::ffi::c_char,
            },
            driver_quirk {
                match_: Some(match_dx10_capable),
                apply: Some(quirk_clip_varying),
                description: b"Reserved varying for gl_ClipPos\0".as_ptr() as *const ::core::ffi::c_char,
            },
            driver_quirk {
                match_: Some(match_allows_spec_alpha),
                apply: Some(quirk_allows_specular_alpha),
                description: b"Allow specular alpha quirk\0".as_ptr() as *const ::core::ffi::c_char,
            },
            driver_quirk {
                match_: Some(match_broken_nv_clip),
                apply: Some(quirk_disable_nvvp_clip),
                description: b"Apple NV_vertex_program clip bug quirk\0".as_ptr() as *const ::core::ffi::c_char,
            },
            driver_quirk {
                match_: Some(match_fbo_tex_update),
                apply: Some(quirk_fbo_tex_update),
                description: b"FBO rebind for attachment updates\0".as_ptr() as *const ::core::ffi::c_char,
            },
            driver_quirk {
                match_: Some(match_broken_rgba16),
                apply: Some(quirk_broken_rgba16),
                description: b"True RGBA16 is not available\0".as_ptr() as *const ::core::ffi::c_char,
            },
            driver_quirk {
                match_: Some(match_fglrx),
                apply: Some(quirk_infolog_spam),
                description: b"Not printing GLSL infolog\0".as_ptr() as *const ::core::ffi::c_char,
            },
            driver_quirk {
                match_: Some(match_not_dx10_capable),
                apply: Some(quirk_limited_tex_filtering),
                description: b"Texture filtering, blending and VTF support is limited\0".as_ptr() as *const ::core::ffi::c_char,
            },
            driver_quirk {
                match_: Some(match_r200),
                apply: Some(quirk_r200_constants),
                description: b"r200 vertex shader constants\0".as_ptr() as *const ::core::ffi::c_char,
            },
            driver_quirk {
                match_: Some(match_broken_arb_fog),
                apply: Some(quirk_broken_arb_fog),
                description: b"ARBfp fogstart == fogend workaround\0".as_ptr() as *const ::core::ffi::c_char,
            },
            driver_quirk {
                match_: Some(match_broken_viewport_subpixel_bits),
                apply: Some(quirk_broken_viewport_subpixel_bits),
                description: b"NVIDIA viewport subpixel bits bug\0".as_ptr() as *const ::core::ffi::c_char,
            },
            driver_quirk {
                match_: Some(match_no_independent_bit_depths),
                apply: Some(quirk_no_independent_bit_depths),
                description: b"No support for MRT with independent bit depths\0".as_ptr() as *const ::core::ffi::c_char,
            },
        ]
    };

    /*
    * WINE's timing here is completely broken,
    * This allows us to fallback to the NO_3D fallback though.
    */
    if (*gl_info).supported[EXT_FRAMEBUFFER_OBJECT as usize] == 0
        || (*gl_info).supported[ARB_VERTEX_SHADER as usize] == 0
        || (*gl_info).supported[ARB_FRAGMENT_SHADER as usize] == 0
    {
        ERR!("Driver \"%s\" is missing extensions wined3d requires; skipping quirk detection.\n", gl_renderer);
        return;
    }

    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(quirk_table) as usize {
        if !(quirk_table[i as usize]
            .match_
            .unwrap_unchecked()(
            gl_info,
            ctx,
            gl_renderer,
            gl_vendor,
            card_vendor,
            device,
        ) == 0)
        {
            TRACE!("Applying driver quirk \"%s\".\n", quirk_table[i as usize].description);
            quirk_table[i as usize]
                .apply
                .unwrap_unchecked()(gl_info);
        }
        i = i.wrapping_add(1);
    }

    /* Find out if PBOs work as they are supposed to. */
    test_pbo_functionality(gl_info);
}

unsafe extern "C" fn wined3d_parse_gl_version(gl_version: *const core::ffi::c_char) -> core::ffi::c_uint {
    let mut ptr: *const core::ffi::c_char = gl_version;
    let mut major: core::ffi::c_int;
    let mut minor: core::ffi::c_int;

    major = atoi(ptr);
    if major <= 0 {
        ERR!("Invalid OpenGL major version %d.\n", major);
    }

    while isdigit(*ptr as core::ffi::c_int) != 0 {
        ptr = ptr.offset(1);
    }
    let fresh27 = ptr;
    ptr = ptr.offset(1);
    if *fresh27 as core::ffi::c_int != '.' as i32 {
        ERR!("Invalid OpenGL version string %s.\n", debugstr_a(gl_version));
    }

    minor = atoi(ptr);

    return MAKEDWORD_VERSION!(major, minor);
}

unsafe extern "C" fn wined3d_guess_gl_vendor(
    mut gl_info: *const wined3d_gl_info,
    mut gl_vendor_string: *const ::core::ffi::c_char,
    mut gl_renderer: *const ::core::ffi::c_char,
    mut gl_version: *const ::core::ffi::c_char,
) -> wined3d_gl_vendor {
    if (*gl_info).supported[APPLE_FLUSH_RENDER as usize] != 0
        && ((*gl_info).supported[APPLE_YCBCR_422 as usize] != 0
            || (*gl_info).supported[APPLE_RGB_422 as usize] != 0)
    {
        return GL_VENDOR_APPLE;
    }
    if !strstr(
        gl_vendor_string,
        b"NVIDIA\0" as *const u8 as *const ::core::ffi::c_char,
    )
    .is_null()
    {
        return GL_VENDOR_NVIDIA;
    }
    if !strstr(
        gl_vendor_string,
        b"ATI\0" as *const u8 as *const ::core::ffi::c_char,
    )
    .is_null()
    {
        return GL_VENDOR_FGLRX;
    }
    if !strstr(
        gl_vendor_string,
        b"Mesa\0" as *const u8 as *const ::core::ffi::c_char,
    )
    .is_null()
        || !strstr(
            gl_vendor_string,
            b"X.Org\0" as *const u8 as *const ::core::ffi::c_char,
        )
        .is_null()
        || !strstr(
            gl_vendor_string,
            b"Advanced Micro Devices, Inc.\0" as *const u8 as *const ::core::ffi::c_char,
        )
        .is_null()
        || !strstr(
            gl_vendor_string,
            b"DRI R300 Project\0" as *const u8 as *const ::core::ffi::c_char,
        )
        .is_null()
        || !strstr(
            gl_vendor_string,
            b"Tungsten Graphics, Inc\0" as *const u8 as *const ::core::ffi::c_char,
        )
        .is_null()
        || !strstr(
            gl_vendor_string,
            b"VMware, Inc.\0" as *const u8 as *const ::core::ffi::c_char,
        )
        .is_null()
        || !strstr(
            gl_vendor_string,
            b"Red Hat\0" as *const u8 as *const ::core::ffi::c_char,
        )
        .is_null()
        || !strstr(
            gl_vendor_string,
            b"Intel\0" as *const u8 as *const ::core::ffi::c_char,
        )
        .is_null()
        || !strstr(
            gl_renderer,
            b"Mesa\0" as *const u8 as *const ::core::ffi::c_char,
        )
        .is_null()
        || !strstr(
            gl_renderer,
            b"Gallium\0" as *const u8 as *const ::core::ffi::c_char,
        )
        .is_null()
        || !strstr(
            gl_renderer,
            b"Intel\0" as *const u8 as *const ::core::ffi::c_char,
        )
        .is_null()
        || !strstr(
            gl_version,
            b"Mesa\0" as *const u8 as *const ::core::ffi::c_char,
        )
        .is_null()
    {
        return GL_VENDOR_MESA;
    }
    if ((((*__wine_dbch___default).flags as ::core::ffi::c_int)
        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_FIXME as ::core::ffi::c_int)))
        != 0)
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/directx/wine/wined3d/adapter_gl.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"wined3d_guess_gl_vendor\0" as *const u8 as *const ::core::ffi::c_char,
            1204 as ::core::ffi::c_int,
            b"Received unrecognized GL_VENDOR %s. Returning GL_VENDOR_UNKNOWN.\n\0"
                as *const u8 as *const ::core::ffi::c_char,
            debugstr_a(gl_vendor_string),
        );
    }
    return GL_VENDOR_UNKNOWN;
}

unsafe extern "C" fn wined3d_guess_card_vendor(
    gl_vendor_string: *const ::core::ffi::c_char,
    gl_renderer: *const ::core::ffi::c_char,
) -> wined3d_pci_vendor {
    if !strstr(gl_vendor_string, b"NVIDIA\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        || !strstr(gl_vendor_string, b"Nouveau\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        || !strstr(gl_vendor_string, b"nouveau\0".as_ptr() as *const ::core::ffi::c_char).is_null()
    {
        return HW_VENDOR_NVIDIA;
    }

    if !strstr(gl_vendor_string, b"ATI\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        || !strstr(gl_vendor_string, b"Advanced Micro Devices, Inc.\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        || !strstr(gl_vendor_string, b"X.Org R300 Project\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        || !strstr(gl_vendor_string, b"AMD\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        || !strstr(gl_renderer, b"AMD\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        || !strstr(gl_renderer, b"FirePro\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        || !strstr(gl_renderer, b"Radeon\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        || !strstr(gl_renderer, b"R100\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        || !strstr(gl_renderer, b"R200\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        || !strstr(gl_renderer, b"R300\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        || !strstr(gl_renderer, b"R600\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        || !strstr(gl_renderer, b"R700\0".as_ptr() as *const ::core::ffi::c_char).is_null()
    {
        return HW_VENDOR_AMD;
    }

    if !strstr(gl_vendor_string, b"Intel(R)\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        /* Intel switched from Intel(R) to Intel® recently, so just match Intel. */
        || !strstr(gl_renderer, b"Intel\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        || !strstr(gl_renderer, b"i915\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        || !strstr(gl_vendor_string, b"Intel Inc.\0".as_ptr() as *const ::core::ffi::c_char).is_null()
    {
        return HW_VENDOR_INTEL;
    }

    if !strstr(gl_vendor_string, b"Red Hat\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        || !strstr(gl_renderer, b"virgl\0".as_ptr() as *const ::core::ffi::c_char).is_null()
    {
        return HW_VENDOR_REDHAT;
    }

    if !strstr(gl_renderer, b"SVGA3D\0".as_ptr() as *const ::core::ffi::c_char).is_null()
    {
        return HW_VENDOR_VMWARE;
    }

    if !strstr(gl_vendor_string, b"Mesa\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        || !strstr(gl_vendor_string, b"Brian Paul\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        || !strstr(gl_vendor_string, b"Tungsten Graphics, Inc\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        || !strstr(gl_vendor_string, b"VMware, Inc.\0".as_ptr() as *const ::core::ffi::c_char).is_null()
    {
        return HW_VENDOR_SOFTWARE;
    }

    WINE_FIXME!("Received unrecognized GL_VENDOR %s. Returning HW_VENDOR_NVIDIA.\n", debugstr_a(gl_vendor_string));

    return HW_VENDOR_NVIDIA;
}

unsafe extern "C" fn feature_level_from_caps(
    gl_info: *const wined3d_gl_info,
    shader_caps: *const shader_caps,
    fragment_caps: *const fragment_caps,
) -> wined3d_feature_level {
    let mut shader_model: ::core::ffi::c_uint;

    shader_model = min!((*shader_caps).vs_version, (*shader_caps).ps_version);
    shader_model = min!(shader_model, max!((*shader_caps).gs_version, 3));
    shader_model = min!(shader_model, max!((*shader_caps).hs_version, 4));
    shader_model = min!(shader_model, max!((*shader_caps).ds_version, 4));

    if (*gl_info).supported[WINED3D_GL_VERSION_3_2 as usize] != 0
        && (*gl_info).supported[ARB_POLYGON_OFFSET_CLAMP as usize] != 0
        && (*gl_info).supported[ARB_SAMPLER_OBJECTS as usize] != 0
    {
        if shader_model >= 5
            && (*gl_info).supported[ARB_DRAW_INDIRECT as usize] != 0
            && (*gl_info).supported[ARB_TEXTURE_COMPRESSION_BPTC as usize] != 0
        {
            return WINED3D_FEATURE_LEVEL_11_1;
        }

        if shader_model >= 4 {
            if (*gl_info).supported[ARB_TEXTURE_CUBE_MAP_ARRAY as usize] != 0
                && (*gl_info).supported[ARB_DRAW_BUFFERS_BLEND as usize] != 0
            {
                return WINED3D_FEATURE_LEVEL_10_1;
            }
            return WINED3D_FEATURE_LEVEL_10;
        }
    }

    if shader_model >= 3
        && (*gl_info).limits.texture_size >= 4096
        && (*gl_info).limits.buffers >= 4
    {
        return WINED3D_FEATURE_LEVEL_9_3;
    }
    if shader_model >= 2 {
        if (*gl_info).supported[ARB_OCCLUSION_QUERY as usize] != 0
            && (*gl_info).supported[ARB_TEXTURE_MIRROR_CLAMP_TO_EDGE as usize] != 0
            && (*gl_info).supported[EXT_BLEND_EQUATION_SEPARATE as usize] != 0
            && (*gl_info).supported[EXT_BLEND_FUNC_SEPARATE as usize] != 0
        {
            return WINED3D_FEATURE_LEVEL_9_2;
        }

        return WINED3D_FEATURE_LEVEL_9_1;
    }
    if shader_model >= 1 {
        return WINED3D_FEATURE_LEVEL_8;
    }

    if (*fragment_caps).TextureOpCaps & WINED3DTEXOPCAPS_DOTPRODUCT3 != 0 {
        return WINED3D_FEATURE_LEVEL_7;
    }
    if (*fragment_caps).max_textures > 1 {
        return WINED3D_FEATURE_LEVEL_6;
    }

    return WINED3D_FEATURE_LEVEL_5;
}

unsafe extern "C" fn select_card_handler(
    mut table: *const gl_vendor_selection,
    mut table_size: ::core::ffi::c_uint,
    mut gl_vendor: wined3d_gl_vendor,
    mut gl_renderer: *const ::core::ffi::c_char,
) -> wined3d_pci_device {
    let mut i: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_uint = 0;
    i = 0 as ::core::ffi::c_uint;
    while i < table_size {
        if (*table.offset(i as isize)).gl_vendor as ::core::ffi::c_uint
            != gl_vendor as ::core::ffi::c_uint
        {
            i = i.wrapping_add(1);
        } else {
            WINE_TRACE!("Applying card selector \"%s\".\n", (*table.offset(i as isize)).description);
            j = 0 as ::core::ffi::c_uint;
            while (j as size_t) < (*table.offset(i as isize)).cards_size {
                if !strstr(
                    gl_renderer,
                    (*(*table.offset(i as isize)).cards.offset(j as isize)).renderer,
                )
                .is_null()
                {
                    return (*(*table.offset(i as isize)).cards.offset(j as isize)).id;
                }
                j = j.wrapping_add(1);
            }
            return PCI_DEVICE_NONE as wined3d_pci_device;
        }
    }
    WINE_FIXME!("Couldn't find a suitable card selector for GL vendor %04x (using GL_RENDERER %s)\n", gl_vendor as ::core::ffi::c_uint, debugstr_a(gl_renderer));
    return PCI_DEVICE_NONE as wined3d_pci_device;
}

unsafe extern "C" fn wined3d_guess_card(
    mut feature_level: wined3d_feature_level,
    mut gl_renderer: *const ::core::ffi::c_char,
    mut gl_vendor: *mut wined3d_gl_vendor,
    mut card_vendor: *mut wined3d_pci_vendor,
) -> wined3d_pci_device {
    let mut device: wined3d_pci_device = CARD_WINE;
    let mut i: ::core::ffi::c_uint = 0;
    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(card_vendor_table) as usize {
        if card_vendor_table[i as usize].card_vendor as ::core::ffi::c_uint
            != *card_vendor as ::core::ffi::c_uint
        {
            i = i.wrapping_add(1);
        } else {
            WINE_TRACE!("Applying card selector \"%s\".\n", card_vendor_table[i as usize].description);
            device = select_card_handler(
                card_vendor_table[i as usize].gl_vendor_selection,
                card_vendor_table[i as usize].gl_vendor_count,
                *gl_vendor,
                gl_renderer,
            );
            if device as ::core::ffi::c_uint
                != PCI_DEVICE_NONE as ::core::ffi::c_uint
            {
                return device;
            }
            WINE_TRACE!("Unrecognized renderer %s, falling back to default.\n", debugstr_a(gl_renderer));
            return wined3d_gpu_from_feature_level(card_vendor, feature_level);
        }
    }
    WINE_FIXME!("No card selector available for card vendor %04x (using GL_RENDERER %s).\n", *card_vendor as ::core::ffi::c_uint, debugstr_a(gl_renderer));
    return wined3d_gpu_from_feature_level(card_vendor, feature_level);
}

unsafe extern "C" fn parse_extension_string(
    gl_info: *mut wined3d_gl_info,
    mut extensions: *const ::core::ffi::c_char,
    map: *const wined3d_extension_map,
    entry_count: UINT,
) {
    while *extensions != 0 {
        let mut start: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut len: size_t = 0;
        let mut i: UINT = 0;
        while isspace(*extensions as ::core::ffi::c_int) != 0 {
            extensions = extensions.offset(1);
        }
        start = extensions;
        while isspace(*extensions as ::core::ffi::c_int) == 0
            && *extensions as ::core::ffi::c_int != 0
        {
            extensions = extensions.offset(1);
        }
        len = extensions.offset_from(start) as ::core::ffi::c_longlong as size_t;
        if len == 0 {
            continue;
        }
        WINE_TRACE!("- %s.\n", debugstr_an(start, len as ::core::ffi::c_int));
        i = 0;
        while i < entry_count {
            if len == strlen((*map.offset(i as isize)).extension_string)
                && memcmp(
                    start as *const ::core::ffi::c_void,
                    (*map.offset(i as isize)).extension_string as *const ::core::ffi::c_void,
                    len,
                ) == 0
            {
                WINE_TRACE!(" FOUND: %s support.\n", (*map.offset(i as isize)).extension_string);
                (*gl_info).supported[(*map.offset(i as isize)).extension as usize] =
                    TRUE as BOOL;
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
    }
}

unsafe extern "C" fn enumerate_gl_extensions(
    gl_info: *mut wined3d_gl_info,
    map: *const wined3d_extension_map,
    map_entries_count: ::core::ffi::c_uint,
) {
    let mut gl_extension_name: *const ::core::ffi::c_char = core::ptr::null();
    let mut i: ::core::ffi::c_uint;
    let mut j: ::core::ffi::c_uint;
    let mut extensions_count: GLint = 0;

    (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
        GL_NUM_EXTENSIONS,
        core::ptr::addr_of_mut!(extensions_count),
    );
    i = 0;
    while i < extensions_count as ::core::ffi::c_uint {
        gl_extension_name = ((*gl_info).gl_ops.ext.p_glGetStringi.unwrap_unchecked())(GL_EXTENSIONS, i)
            as *const ::core::ffi::c_char;
        j = 0;
        while j < map_entries_count {
            if strcmp(
                gl_extension_name,
                (*map.offset(j as isize)).extension_string,
            ) == 0
            {
                (*gl_info).supported[(*map.offset(j as isize)).extension as usize] =
                    (TRUE as i32);
                break;
            } else {
                j = j.wrapping_add(1);
            }
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn load_gl_funcs(gl_info: *mut wined3d_gl_info) {
    macro_rules! use_gl {
        ($field:ident, $name:literal) => {
            (*gl_info).gl_ops.ext.$field =
                ::core::mem::transmute(wglGetProcAddress(concat!($name, "\0").as_ptr() as LPCSTR));
        };
    }
    macro_rules! map_gl {
        ($core:ident, $ext:ident) => {
            if (*gl_info).gl_ops.ext.$core.is_none() {
                (*gl_info).gl_ops.ext.$core = ::core::mem::transmute((*gl_info).gl_ops.ext.$ext);
            }
        };
    }

    /* GL_APPLE_fence */
    use_gl!(p_glDeleteFencesAPPLE, "glDeleteFencesAPPLE");
    use_gl!(p_glFinishFenceAPPLE, "glFinishFenceAPPLE");
    use_gl!(p_glFinishObjectAPPLE, "glFinishObjectAPPLE");
    use_gl!(p_glGenFencesAPPLE, "glGenFencesAPPLE");
    use_gl!(p_glIsFenceAPPLE, "glIsFenceAPPLE");
    use_gl!(p_glSetFenceAPPLE, "glSetFenceAPPLE");
    use_gl!(p_glTestFenceAPPLE, "glTestFenceAPPLE");
    use_gl!(p_glTestObjectAPPLE, "glTestObjectAPPLE");
    /* GL_APPLE_flush_buffer_range */
    use_gl!(p_glBufferParameteriAPPLE, "glBufferParameteriAPPLE");
    use_gl!(p_glFlushMappedBufferRangeAPPLE, "glFlushMappedBufferRangeAPPLE");
    /* GL_ARB_base_instance */
    use_gl!(p_glDrawArraysInstancedBaseInstance, "glDrawArraysInstancedBaseInstance");
    use_gl!(p_glDrawElementsInstancedBaseVertexBaseInstance, "glDrawElementsInstancedBaseVertexBaseInstance");
    /* GL_ARB_blend_func_extended */
    use_gl!(p_glBindFragDataLocationIndexed, "glBindFragDataLocationIndexed");
    use_gl!(p_glGetFragDataIndex, "glGetFragDataIndex");
    /* GL_ARB_buffer_storage */
    use_gl!(p_glBufferStorage, "glBufferStorage");
    /* GL_ARB_clear_buffer_object */
    use_gl!(p_glClearBufferData, "glClearBufferData");
    use_gl!(p_glClearBufferSubData, "glClearBufferSubData");
    /* GL_ARB_clear_texture */
    use_gl!(p_glClearTexImage, "glClearTexImage");
    use_gl!(p_glClearTexSubImage, "glClearTexSubImage");
    /* GL_ARB_clip_control */
    use_gl!(p_glClipControl, "glClipControl");
    /* GL_ARB_color_buffer_float */
    use_gl!(p_glClampColorARB, "glClampColorARB");
    /* GL_ARB_compute_shader */
    use_gl!(p_glDispatchCompute, "glDispatchCompute");
    use_gl!(p_glDispatchComputeIndirect, "glDispatchComputeIndirect");
    /* GL_ARB_copy_buffer */
    use_gl!(p_glCopyBufferSubData, "glCopyBufferSubData");
    /* GL_ARB_copy_image */
    use_gl!(p_glCopyImageSubData, "glCopyImageSubData");
    /* GL_ARB_debug_output */
    use_gl!(p_glDebugMessageCallbackARB, "glDebugMessageCallbackARB");
    use_gl!(p_glDebugMessageControlARB, "glDebugMessageControlARB");
    use_gl!(p_glDebugMessageInsertARB, "glDebugMessageInsertARB");
    use_gl!(p_glGetDebugMessageLogARB, "glGetDebugMessageLogARB");
    /* GL_ARB_draw_buffers */
    use_gl!(p_glDrawBuffersARB, "glDrawBuffersARB");
    /* GL_ARB_draw_buffers_blend */
    use_gl!(p_glBlendEquationiARB, "glBlendEquationiARB");
    use_gl!(p_glBlendEquationSeparateiARB, "glBlendEquationSeparateiARB");
    use_gl!(p_glBlendFunciARB, "glBlendFunciARB");
    use_gl!(p_glBlendFuncSeparateiARB, "glBlendFuncSeparateiARB");
    /* GL_ARB_draw_elements_base_vertex */
    use_gl!(p_glDrawElementsBaseVertex, "glDrawElementsBaseVertex");
    use_gl!(p_glDrawElementsInstancedBaseVertex, "glDrawElementsInstancedBaseVertex");
    use_gl!(p_glDrawRangeElementsBaseVertex, "glDrawRangeElementsBaseVertex");
    use_gl!(p_glMultiDrawElementsBaseVertex, "glMultiDrawElementsBaseVertex");
    /* GL_ARB_draw_indirect */
    use_gl!(p_glDrawArraysIndirect, "glDrawArraysIndirect");
    use_gl!(p_glDrawElementsIndirect, "glDrawElementsIndirect");
    /* GL_ARB_draw_instanced */
    use_gl!(p_glDrawArraysInstancedARB, "glDrawArraysInstancedARB");
    use_gl!(p_glDrawElementsInstancedARB, "glDrawElementsInstancedARB");
    /* GL_ARB_ES2_compatibility */
    use_gl!(p_glReleaseShaderCompiler, "glReleaseShaderCompiler");
    use_gl!(p_glShaderBinary, "glShaderBinary");
    use_gl!(p_glGetShaderPrecisionFormat, "glGetShaderPrecisionFormat");
    use_gl!(p_glDepthRangef, "glDepthRangef");
    use_gl!(p_glClearDepthf, "glClearDepthf");
    /* GL_ARB_framebuffer_no_attachments */
    use_gl!(p_glFramebufferParameteri, "glFramebufferParameteri");
    /* GL_ARB_framebuffer_object */
    use_gl!(p_glBindFramebuffer, "glBindFramebuffer");
    use_gl!(p_glBindRenderbuffer, "glBindRenderbuffer");
    use_gl!(p_glBlitFramebuffer, "glBlitFramebuffer");
    use_gl!(p_glCheckFramebufferStatus, "glCheckFramebufferStatus");
    use_gl!(p_glDeleteFramebuffers, "glDeleteFramebuffers");
    use_gl!(p_glDeleteRenderbuffers, "glDeleteRenderbuffers");
    use_gl!(p_glFramebufferRenderbuffer, "glFramebufferRenderbuffer");
    use_gl!(p_glFramebufferTexture, "glFramebufferTexture");
    use_gl!(p_glFramebufferTexture1D, "glFramebufferTexture1D");
    use_gl!(p_glFramebufferTexture2D, "glFramebufferTexture2D");
    use_gl!(p_glFramebufferTexture3D, "glFramebufferTexture3D");
    use_gl!(p_glFramebufferTextureLayer, "glFramebufferTextureLayer");
    use_gl!(p_glGenFramebuffers, "glGenFramebuffers");
    use_gl!(p_glGenRenderbuffers, "glGenRenderbuffers");
    use_gl!(p_glGenerateMipmap, "glGenerateMipmap");
    use_gl!(p_glGetFramebufferAttachmentParameteriv, "glGetFramebufferAttachmentParameteriv");
    use_gl!(p_glGetRenderbufferParameteriv, "glGetRenderbufferParameteriv");
    use_gl!(p_glIsFramebuffer, "glIsFramebuffer");
    use_gl!(p_glIsRenderbuffer, "glIsRenderbuffer");
    use_gl!(p_glRenderbufferStorage, "glRenderbufferStorage");
    use_gl!(p_glRenderbufferStorageMultisample, "glRenderbufferStorageMultisample");
    /* GL_ARB_geometry_shader4 */
    use_gl!(p_glFramebufferTextureARB, "glFramebufferTextureARB");
    use_gl!(p_glFramebufferTextureFaceARB, "glFramebufferTextureFaceARB");
    use_gl!(p_glFramebufferTextureLayerARB, "glFramebufferTextureLayerARB");
    use_gl!(p_glProgramParameteriARB, "glProgramParameteriARB");
    /* GL_ARB_instanced_arrays */
    use_gl!(p_glVertexAttribDivisorARB, "glVertexAttribDivisorARB");
    /* GL_ARB_internalformat_query */
    use_gl!(p_glGetInternalformativ, "glGetInternalformativ");
    /* GL_ARB_internalformat_query2 */
    use_gl!(p_glGetInternalformati64v, "glGetInternalformati64v");
    /* GL_ARB_map_buffer_range */
    use_gl!(p_glFlushMappedBufferRange, "glFlushMappedBufferRange");
    use_gl!(p_glMapBufferRange, "glMapBufferRange");
    /* GL_ARB_multisample */
    use_gl!(p_glSampleCoverageARB, "glSampleCoverageARB");
    /* GL_ARB_multitexture */
    use_gl!(p_glActiveTextureARB, "glActiveTextureARB");
    use_gl!(p_glClientActiveTextureARB, "glClientActiveTextureARB");
    use_gl!(p_glMultiTexCoord1fARB, "glMultiTexCoord1fARB");
    use_gl!(p_glMultiTexCoord1fvARB, "glMultiTexCoord1fvARB");
    use_gl!(p_glMultiTexCoord2fARB, "glMultiTexCoord2fARB");
    use_gl!(p_glMultiTexCoord2fvARB, "glMultiTexCoord2fvARB");
    use_gl!(p_glMultiTexCoord2svARB, "glMultiTexCoord2svARB");
    use_gl!(p_glMultiTexCoord3fARB, "glMultiTexCoord3fARB");
    use_gl!(p_glMultiTexCoord3fvARB, "glMultiTexCoord3fvARB");
    use_gl!(p_glMultiTexCoord4fARB, "glMultiTexCoord4fARB");
    use_gl!(p_glMultiTexCoord4fvARB, "glMultiTexCoord4fvARB");
    use_gl!(p_glMultiTexCoord4svARB, "glMultiTexCoord4svARB");
    /* GL_ARB_occlusion_query */
    use_gl!(p_glBeginQueryARB, "glBeginQueryARB");
    use_gl!(p_glDeleteQueriesARB, "glDeleteQueriesARB");
    use_gl!(p_glEndQueryARB, "glEndQueryARB");
    use_gl!(p_glGenQueriesARB, "glGenQueriesARB");
    use_gl!(p_glGetQueryivARB, "glGetQueryivARB");
    use_gl!(p_glGetQueryObjectivARB, "glGetQueryObjectivARB");
    use_gl!(p_glGetQueryObjectuivARB, "glGetQueryObjectuivARB");
    use_gl!(p_glIsQueryARB, "glIsQueryARB");
    /* GL_ARB_point_parameters */
    use_gl!(p_glPointParameterfARB, "glPointParameterfARB");
    use_gl!(p_glPointParameterfvARB, "glPointParameterfvARB");
    /* GL_ARB_polgyon_offset_clamp */
    use_gl!(p_glPolygonOffsetClamp, "glPolygonOffsetClamp");
    /* GL_ARB_provoking_vertex */
    use_gl!(p_glProvokingVertex, "glProvokingVertex");
    /* GL_ARB_sample_shading */
    use_gl!(p_glMinSampleShadingARB, "glMinSampleShadingARB");
    /* GL_ARB_sampler_objects */
    use_gl!(p_glGenSamplers, "glGenSamplers");
    use_gl!(p_glDeleteSamplers, "glDeleteSamplers");
    use_gl!(p_glIsSampler, "glIsSampler");
    use_gl!(p_glBindSampler, "glBindSampler");
    use_gl!(p_glSamplerParameteri, "glSamplerParameteri");
    use_gl!(p_glSamplerParameterf, "glSamplerParameterf");
    use_gl!(p_glSamplerParameteriv, "glSamplerParameteriv");
    use_gl!(p_glSamplerParameterfv, "glSamplerParameterfv");
    use_gl!(p_glSamplerParameterIiv, "glSamplerParameterIiv");
    use_gl!(p_glSamplerParameterIuiv, "glSamplerParameterIuiv");
    use_gl!(p_glGetSamplerParameteriv, "glGetSamplerParameteriv");
    use_gl!(p_glGetSamplerParameterfv, "glGetSamplerParameterfv");
    use_gl!(p_glGetSamplerParameterIiv, "glGetSamplerParameterIiv");
    use_gl!(p_glGetSamplerParameterIuiv, "glGetSamplerParameterIuiv");
    /* GL_ARB_shader_atomic_counters */
    use_gl!(p_glGetActiveAtomicCounterBufferiv, "glGetActiveAtomicCounterBufferiv");
    /* GL_ARB_shader_image_load_store */
    use_gl!(p_glBindImageTexture, "glBindImageTexture");
    use_gl!(p_glMemoryBarrier, "glMemoryBarrier");
    /* GL_ARB_shader_objects */
    use_gl!(p_glAttachObjectARB, "glAttachObjectARB");
    use_gl!(p_glBindAttribLocationARB, "glBindAttribLocationARB");
    use_gl!(p_glCompileShaderARB, "glCompileShaderARB");
    use_gl!(p_glCreateProgramObjectARB, "glCreateProgramObjectARB");
    use_gl!(p_glCreateShaderObjectARB, "glCreateShaderObjectARB");
    use_gl!(p_glDeleteObjectARB, "glDeleteObjectARB");
    use_gl!(p_glDetachObjectARB, "glDetachObjectARB");
    use_gl!(p_glGetActiveUniformARB, "glGetActiveUniformARB");
    use_gl!(p_glGetAttachedObjectsARB, "glGetAttachedObjectsARB");
    use_gl!(p_glGetAttribLocationARB, "glGetAttribLocationARB");
    use_gl!(p_glGetHandleARB, "glGetHandleARB");
    use_gl!(p_glGetInfoLogARB, "glGetInfoLogARB");
    use_gl!(p_glGetObjectParameterfvARB, "glGetObjectParameterfvARB");
    use_gl!(p_glGetObjectParameterivARB, "glGetObjectParameterivARB");
    use_gl!(p_glGetShaderSourceARB, "glGetShaderSourceARB");
    use_gl!(p_glGetUniformLocationARB, "glGetUniformLocationARB");
    use_gl!(p_glGetUniformfvARB, "glGetUniformfvARB");
    use_gl!(p_glGetUniformivARB, "glGetUniformivARB");
    use_gl!(p_glLinkProgramARB, "glLinkProgramARB");
    use_gl!(p_glShaderSourceARB, "glShaderSourceARB");
    use_gl!(p_glUniform1fARB, "glUniform1fARB");
    use_gl!(p_glUniform1fvARB, "glUniform1fvARB");
    use_gl!(p_glUniform1iARB, "glUniform1iARB");
    use_gl!(p_glUniform1ivARB, "glUniform1ivARB");
    use_gl!(p_glUniform2fARB, "glUniform2fARB");
    use_gl!(p_glUniform2fvARB, "glUniform2fvARB");
    use_gl!(p_glUniform2iARB, "glUniform2iARB");
    use_gl!(p_glUniform2ivARB, "glUniform2ivARB");
    use_gl!(p_glUniform3fARB, "glUniform3fARB");
    use_gl!(p_glUniform3fvARB, "glUniform3fvARB");
    use_gl!(p_glUniform3iARB, "glUniform3iARB");
    use_gl!(p_glUniform3ivARB, "glUniform3ivARB");
    use_gl!(p_glUniform4fARB, "glUniform4fARB");
    use_gl!(p_glUniform4fvARB, "glUniform4fvARB");
    use_gl!(p_glUniform4iARB, "glUniform4iARB");
    use_gl!(p_glUniform4ivARB, "glUniform4ivARB");
    use_gl!(p_glUniformMatrix2fvARB, "glUniformMatrix2fvARB");
    use_gl!(p_glUniformMatrix3fvARB, "glUniformMatrix3fvARB");
    use_gl!(p_glUniformMatrix4fvARB, "glUniformMatrix4fvARB");
    use_gl!(p_glUseProgramObjectARB, "glUseProgramObjectARB");
    use_gl!(p_glValidateProgramARB, "glValidateProgramARB");
    /* GL_ARB_shader_storage_buffer_object */
    use_gl!(p_glShaderStorageBlockBinding, "glShaderStorageBlockBinding");
    /* GL_ARB_sync */
    use_gl!(p_glClientWaitSync, "glClientWaitSync");
    use_gl!(p_glDeleteSync, "glDeleteSync");
    use_gl!(p_glFenceSync, "glFenceSync");
    use_gl!(p_glGetInteger64v, "glGetInteger64v");
    use_gl!(p_glGetSynciv, "glGetSynciv");
    use_gl!(p_glIsSync, "glIsSync");
    use_gl!(p_glWaitSync, "glWaitSync");
    /* GL_ARB_tessellation_shader */
    use_gl!(p_glPatchParameteri, "glPatchParameteri");
    use_gl!(p_glPatchParameterfv, "glPatchParameterfv");
    /* GL_ARB_texture_buffer_object */
    use_gl!(p_glTexBufferARB, "glTexBufferARB");
    /* GL_ARB_texture_buffer_range */
    use_gl!(p_glTexBufferRange, "glTexBufferRange");
    /* GL_ARB_texture_compression */
    use_gl!(p_glCompressedTexImage2DARB, "glCompressedTexImage2DARB");
    use_gl!(p_glCompressedTexImage3DARB, "glCompressedTexImage3DARB");
    use_gl!(p_glCompressedTexSubImage2DARB, "glCompressedTexSubImage2DARB");
    use_gl!(p_glCompressedTexSubImage3DARB, "glCompressedTexSubImage3DARB");
    use_gl!(p_glGetCompressedTexImageARB, "glGetCompressedTexImageARB");
    /* GL_ARB_texture_multisample */
    use_gl!(p_glGetMultisamplefv, "glGetMultisamplefv");
    use_gl!(p_glSampleMaski, "glSampleMaski");
    use_gl!(p_glTexImage2DMultisample, "glTexImage2DMultisample");
    use_gl!(p_glTexImage3DMultisample, "glTexImage3DMultisample");
    /* GL_ARB_texture_storage */
    use_gl!(p_glTexStorage1D, "glTexStorage1D");
    use_gl!(p_glTexStorage2D, "glTexStorage2D");
    use_gl!(p_glTexStorage3D, "glTexStorage3D");
    /* GL_ARB_texture_storage_multisample */
    use_gl!(p_glTexStorage2DMultisample, "glTexStorage2DMultisample");
    use_gl!(p_glTexStorage3DMultisample, "glTexStorage3DMultisample");
    /* GL_ARB_texture_view */
    use_gl!(p_glTextureView, "glTextureView");
    /* GL_ARB_timer_query */
    use_gl!(p_glQueryCounter, "glQueryCounter");
    use_gl!(p_glGetQueryObjectui64v, "glGetQueryObjectui64v");
    /* GL_ARB_transform_feedback2 */
    use_gl!(p_glBindTransformFeedback, "glBindTransformFeedback");
    use_gl!(p_glDeleteTransformFeedbacks, "glDeleteTransformFeedbacks");
    use_gl!(p_glDrawTransformFeedback, "glDrawTransformFeedback");
    use_gl!(p_glGenTransformFeedbacks, "glGenTransformFeedbacks");
    use_gl!(p_glIsTransformFeedback, "glIsTransformFeedback");
    use_gl!(p_glPauseTransformFeedback, "glPauseTransformFeedback");
    use_gl!(p_glResumeTransformFeedback, "glResumeTransformFeedback");
    /* GL_ARB_transform_feedback3 */
    use_gl!(p_glBeginQueryIndexed, "glBeginQueryIndexed");
    use_gl!(p_glDrawTransformFeedbackStream, "glDrawTransformFeedbackStream");
    use_gl!(p_glEndQueryIndexed, "glEndQueryIndexed");
    use_gl!(p_glGetQueryIndexediv, "glGetQueryIndexediv");
    /* GL_ARB_uniform_buffer_object */
    use_gl!(p_glBindBufferBase, "glBindBufferBase");
    use_gl!(p_glBindBufferRange, "glBindBufferRange");
    use_gl!(p_glGetActiveUniformBlockName, "glGetActiveUniformBlockName");
    use_gl!(p_glGetActiveUniformBlockiv, "glGetActiveUniformBlockiv");
    use_gl!(p_glGetActiveUniformName, "glGetActiveUniformName");
    use_gl!(p_glGetActiveUniformsiv, "glGetActiveUniformsiv");
    use_gl!(p_glGetIntegeri_v, "glGetIntegeri_v");
    use_gl!(p_glGetUniformBlockIndex, "glGetUniformBlockIndex");
    use_gl!(p_glGetUniformIndices, "glGetUniformIndices");
    use_gl!(p_glUniformBlockBinding, "glUniformBlockBinding");
    /* GL_ARB_vertex_buffer_object */
    use_gl!(p_glBindBufferARB, "glBindBufferARB");
    use_gl!(p_glBufferDataARB, "glBufferDataARB");
    use_gl!(p_glBufferSubDataARB, "glBufferSubDataARB");
    use_gl!(p_glDeleteBuffersARB, "glDeleteBuffersARB");
    use_gl!(p_glGenBuffersARB, "glGenBuffersARB");
    use_gl!(p_glGetBufferParameterivARB, "glGetBufferParameterivARB");
    use_gl!(p_glGetBufferPointervARB, "glGetBufferPointervARB");
    use_gl!(p_glGetBufferSubDataARB, "glGetBufferSubDataARB");
    use_gl!(p_glIsBufferARB, "glIsBufferARB");
    use_gl!(p_glMapBufferARB, "glMapBufferARB");
    use_gl!(p_glUnmapBufferARB, "glUnmapBufferARB");
    /* GL_ARB_vertex_program */
    use_gl!(p_glBindProgramARB, "glBindProgramARB");
    use_gl!(p_glDeleteProgramsARB, "glDeleteProgramsARB");
    use_gl!(p_glDisableVertexAttribArrayARB, "glDisableVertexAttribArrayARB");
    use_gl!(p_glEnableVertexAttribArrayARB, "glEnableVertexAttribArrayARB");
    use_gl!(p_glGenProgramsARB, "glGenProgramsARB");
    use_gl!(p_glGetProgramivARB, "glGetProgramivARB");
    use_gl!(p_glProgramEnvParameter4fvARB, "glProgramEnvParameter4fvARB");
    use_gl!(p_glProgramLocalParameter4fvARB, "glProgramLocalParameter4fvARB");
    use_gl!(p_glProgramStringARB, "glProgramStringARB");
    use_gl!(p_glVertexAttrib1dARB, "glVertexAttrib1dARB");
    use_gl!(p_glVertexAttrib1dvARB, "glVertexAttrib1dvARB");
    use_gl!(p_glVertexAttrib1fARB, "glVertexAttrib1fARB");
    use_gl!(p_glVertexAttrib1fvARB, "glVertexAttrib1fvARB");
    use_gl!(p_glVertexAttrib1sARB, "glVertexAttrib1sARB");
    use_gl!(p_glVertexAttrib1svARB, "glVertexAttrib1svARB");
    use_gl!(p_glVertexAttrib2dARB, "glVertexAttrib2dARB");
    use_gl!(p_glVertexAttrib2dvARB, "glVertexAttrib2dvARB");
    use_gl!(p_glVertexAttrib2fARB, "glVertexAttrib2fARB");
    use_gl!(p_glVertexAttrib2fvARB, "glVertexAttrib2fvARB");
    use_gl!(p_glVertexAttrib2sARB, "glVertexAttrib2sARB");
    use_gl!(p_glVertexAttrib2svARB, "glVertexAttrib2svARB");
    use_gl!(p_glVertexAttrib3dARB, "glVertexAttrib3dARB");
    use_gl!(p_glVertexAttrib3dvARB, "glVertexAttrib3dvARB");
    use_gl!(p_glVertexAttrib3fARB, "glVertexAttrib3fARB");
    use_gl!(p_glVertexAttrib3fvARB, "glVertexAttrib3fvARB");
    use_gl!(p_glVertexAttrib3sARB, "glVertexAttrib3sARB");
    use_gl!(p_glVertexAttrib3svARB, "glVertexAttrib3svARB");
    use_gl!(p_glVertexAttrib4NbvARB, "glVertexAttrib4NbvARB");
    use_gl!(p_glVertexAttrib4NivARB, "glVertexAttrib4NivARB");
    use_gl!(p_glVertexAttrib4NsvARB, "glVertexAttrib4NsvARB");
    use_gl!(p_glVertexAttrib4NubARB, "glVertexAttrib4NubARB");
    use_gl!(p_glVertexAttrib4NubvARB, "glVertexAttrib4NubvARB");
    use_gl!(p_glVertexAttrib4NuivARB, "glVertexAttrib4NuivARB");
    use_gl!(p_glVertexAttrib4NusvARB, "glVertexAttrib4NusvARB");
    use_gl!(p_glVertexAttrib4bvARB, "glVertexAttrib4bvARB");
    use_gl!(p_glVertexAttrib4dARB, "glVertexAttrib4dARB");
    use_gl!(p_glVertexAttrib4dvARB, "glVertexAttrib4dvARB");
    use_gl!(p_glVertexAttrib4fARB, "glVertexAttrib4fARB");
    use_gl!(p_glVertexAttrib4fvARB, "glVertexAttrib4fvARB");
    use_gl!(p_glVertexAttrib4ivARB, "glVertexAttrib4ivARB");
    use_gl!(p_glVertexAttrib4sARB, "glVertexAttrib4sARB");
    use_gl!(p_glVertexAttrib4svARB, "glVertexAttrib4svARB");
    use_gl!(p_glVertexAttrib4ubvARB, "glVertexAttrib4ubvARB");
    use_gl!(p_glVertexAttrib4uivARB, "glVertexAttrib4uivARB");
    use_gl!(p_glVertexAttrib4usvARB, "glVertexAttrib4usvARB");
    use_gl!(p_glVertexAttribPointerARB, "glVertexAttribPointerARB");
    /* GL_ARB_viewport_array */
    use_gl!(p_glDepthRangeArrayv, "glDepthRangeArrayv");
    use_gl!(p_glDepthRangeIndexed, "glDepthRangeIndexed");
    use_gl!(p_glGetDoublei_v, "glGetDoublei_v");
    use_gl!(p_glGetFloati_v, "glGetFloati_v");
    use_gl!(p_glScissorArrayv, "glScissorArrayv");
    use_gl!(p_glScissorIndexed, "glScissorIndexed");
    use_gl!(p_glScissorIndexedv, "glScissorIndexedv");
    use_gl!(p_glViewportArrayv, "glViewportArrayv");
    use_gl!(p_glViewportIndexedf, "glViewportIndexedf");
    use_gl!(p_glViewportIndexedfv, "glViewportIndexedfv");
    /* GL_ARB_texture_barrier */
    use_gl!(p_glTextureBarrier, "glTextureBarrier");
    /* GL_ATI_fragment_shader */
    use_gl!(p_glAlphaFragmentOp1ATI, "glAlphaFragmentOp1ATI");
    use_gl!(p_glAlphaFragmentOp2ATI, "glAlphaFragmentOp2ATI");
    use_gl!(p_glAlphaFragmentOp3ATI, "glAlphaFragmentOp3ATI");
    use_gl!(p_glBeginFragmentShaderATI, "glBeginFragmentShaderATI");
    use_gl!(p_glBindFragmentShaderATI, "glBindFragmentShaderATI");
    use_gl!(p_glColorFragmentOp1ATI, "glColorFragmentOp1ATI");
    use_gl!(p_glColorFragmentOp2ATI, "glColorFragmentOp2ATI");
    use_gl!(p_glColorFragmentOp3ATI, "glColorFragmentOp3ATI");
    use_gl!(p_glDeleteFragmentShaderATI, "glDeleteFragmentShaderATI");
    use_gl!(p_glEndFragmentShaderATI, "glEndFragmentShaderATI");
    use_gl!(p_glGenFragmentShadersATI, "glGenFragmentShadersATI");
    use_gl!(p_glPassTexCoordATI, "glPassTexCoordATI");
    use_gl!(p_glSampleMapATI, "glSampleMapATI");
    use_gl!(p_glSetFragmentShaderConstantATI, "glSetFragmentShaderConstantATI");
    /* GL_ATI_separate_stencil */
    use_gl!(p_glStencilOpSeparateATI, "glStencilOpSeparateATI");
    use_gl!(p_glStencilFuncSeparateATI, "glStencilFuncSeparateATI");
    /* GL_EXT_blend_color */
    use_gl!(p_glBlendColorEXT, "glBlendColorEXT");
    /* GL_EXT_blend_equation_separate */
    use_gl!(p_glBlendFuncSeparateEXT, "glBlendFuncSeparateEXT");
    /* GL_EXT_blend_func_separate */
    use_gl!(p_glBlendEquationSeparateEXT, "glBlendEquationSeparateEXT");
    /* GL_EXT_blend_minmax */
    use_gl!(p_glBlendEquationEXT, "glBlendEquationEXT");
    /* GL_EXT_depth_bounds_test */
    use_gl!(p_glDepthBoundsEXT, "glDepthBoundsEXT");
    /* GL_EXT_draw_buffers2 */
    use_gl!(p_glColorMaskIndexedEXT, "glColorMaskIndexedEXT");
    use_gl!(p_glDisableIndexedEXT, "glDisableIndexedEXT");
    use_gl!(p_glEnableIndexedEXT, "glEnableIndexedEXT");
    use_gl!(p_glGetBooleanIndexedvEXT, "glGetBooleanIndexedvEXT");
    use_gl!(p_glGetIntegerIndexedvEXT, "glGetIntegerIndexedvEXT");
    use_gl!(p_glIsEnabledIndexedEXT, "glIsEnabledIndexedEXT");
    /* GL_EXT_fog_coord */
    use_gl!(p_glFogCoordPointerEXT, "glFogCoordPointerEXT");
    use_gl!(p_glFogCoorddEXT, "glFogCoorddEXT");
    use_gl!(p_glFogCoorddvEXT, "glFogCoorddvEXT");
    use_gl!(p_glFogCoordfEXT, "glFogCoordfEXT");
    use_gl!(p_glFogCoordfvEXT, "glFogCoordfvEXT");
    /* GL_EXT_framebuffer_blit */
    use_gl!(p_glBlitFramebufferEXT, "glBlitFramebufferEXT");
    /* GL_EXT_framebuffer_multisample */
    use_gl!(p_glRenderbufferStorageMultisampleEXT, "glRenderbufferStorageMultisampleEXT");
    /* GL_EXT_framebuffer_object */
    use_gl!(p_glBindFramebufferEXT, "glBindFramebufferEXT");
    use_gl!(p_glBindRenderbufferEXT, "glBindRenderbufferEXT");
    use_gl!(p_glCheckFramebufferStatusEXT, "glCheckFramebufferStatusEXT");
    use_gl!(p_glDeleteFramebuffersEXT, "glDeleteFramebuffersEXT");
    use_gl!(p_glDeleteRenderbuffersEXT, "glDeleteRenderbuffersEXT");
    use_gl!(p_glFramebufferRenderbufferEXT, "glFramebufferRenderbufferEXT");
    use_gl!(p_glFramebufferTexture1DEXT, "glFramebufferTexture1DEXT");
    use_gl!(p_glFramebufferTexture2DEXT, "glFramebufferTexture2DEXT");
    use_gl!(p_glFramebufferTexture3DEXT, "glFramebufferTexture3DEXT");
    use_gl!(p_glGenFramebuffersEXT, "glGenFramebuffersEXT");
    use_gl!(p_glGenRenderbuffersEXT, "glGenRenderbuffersEXT");
    use_gl!(p_glGenerateMipmapEXT, "glGenerateMipmapEXT");
    use_gl!(p_glGetFramebufferAttachmentParameterivEXT, "glGetFramebufferAttachmentParameterivEXT");
    use_gl!(p_glGetRenderbufferParameterivEXT, "glGetRenderbufferParameterivEXT");
    use_gl!(p_glIsFramebufferEXT, "glIsFramebufferEXT");
    use_gl!(p_glIsRenderbufferEXT, "glIsRenderbufferEXT");
    use_gl!(p_glRenderbufferStorageEXT, "glRenderbufferStorageEXT");
    /* GL_EXT_gpu_program_parameters */
    use_gl!(p_glProgramEnvParameters4fvEXT, "glProgramEnvParameters4fvEXT");
    use_gl!(p_glProgramLocalParameters4fvEXT, "glProgramLocalParameters4fvEXT");
    /* GL_EXT_gpu_shader4 */
    use_gl!(p_glBindFragDataLocationEXT, "glBindFragDataLocationEXT");
    use_gl!(p_glGetFragDataLocationEXT, "glGetFragDataLocationEXT");
    use_gl!(p_glGetUniformuivEXT, "glGetUniformuivEXT");
    use_gl!(p_glGetVertexAttribIivEXT, "glGetVertexAttribIivEXT");
    use_gl!(p_glGetVertexAttribIuivEXT, "glGetVertexAttribIuivEXT");
    use_gl!(p_glUniform1uiEXT, "glUniform1uiEXT");
    use_gl!(p_glUniform1uivEXT, "glUniform1uivEXT");
    use_gl!(p_glUniform2uiEXT, "glUniform2uiEXT");
    use_gl!(p_glUniform2uivEXT, "glUniform2uivEXT");
    use_gl!(p_glUniform3uiEXT, "glUniform3uiEXT");
    use_gl!(p_glUniform3uivEXT, "glUniform3uivEXT");
    use_gl!(p_glUniform4uiEXT, "glUniform4uiEXT");
    use_gl!(p_glUniform4uivEXT, "glUniform4uivEXT");
    use_gl!(p_glVertexAttribI1iEXT, "glVertexAttribI1iEXT");
    use_gl!(p_glVertexAttribI1ivEXT, "glVertexAttribI1ivEXT");
    use_gl!(p_glVertexAttribI1uiEXT, "glVertexAttribI1uiEXT");
    use_gl!(p_glVertexAttribI1uivEXT, "glVertexAttribI1uivEXT");
    use_gl!(p_glVertexAttribI2iEXT, "glVertexAttribI2iEXT");
    use_gl!(p_glVertexAttribI2ivEXT, "glVertexAttribI2ivEXT");
    use_gl!(p_glVertexAttribI2uiEXT, "glVertexAttribI2uiEXT");
    use_gl!(p_glVertexAttribI2uivEXT, "glVertexAttribI2uivEXT");
    use_gl!(p_glVertexAttribI3iEXT, "glVertexAttribI3iEXT");
    use_gl!(p_glVertexAttribI3ivEXT, "glVertexAttribI3ivEXT");
    use_gl!(p_glVertexAttribI3uiEXT, "glVertexAttribI3uiEXT");
    use_gl!(p_glVertexAttribI3uivEXT, "glVertexAttribI3uivEXT");
    use_gl!(p_glVertexAttribI4bvEXT, "glVertexAttribI4bvEXT");
    use_gl!(p_glVertexAttribI4iEXT, "glVertexAttribI4iEXT");
    use_gl!(p_glVertexAttribI4ivEXT, "glVertexAttribI4ivEXT");
    use_gl!(p_glVertexAttribI4svEXT, "glVertexAttribI4svEXT");
    use_gl!(p_glVertexAttribI4ubvEXT, "glVertexAttribI4ubvEXT");
    use_gl!(p_glVertexAttribI4uiEXT, "glVertexAttribI4uiEXT");
    use_gl!(p_glVertexAttribI4uivEXT, "glVertexAttribI4uivEXT");
    use_gl!(p_glVertexAttribI4usvEXT, "glVertexAttribI4usvEXT");
    use_gl!(p_glVertexAttribIPointerEXT, "glVertexAttribIPointerEXT");
    /* GL_EXT_memory_object */
    use_gl!(p_glGetUnsignedBytei_vEXT, "glGetUnsignedBytei_vEXT");
    use_gl!(p_glGetUnsignedBytevEXT, "glGetUnsignedBytevEXT");
    /* GL_EXT_point_parameters */
    use_gl!(p_glPointParameterfEXT, "glPointParameterfEXT");
    use_gl!(p_glPointParameterfvEXT, "glPointParameterfvEXT");
    /* GL_EXT_polgyon_offset_clamp */
    use_gl!(p_glPolygonOffsetClampEXT, "glPolygonOffsetClampEXT");
    /* GL_EXT_provoking_vertex */
    use_gl!(p_glProvokingVertexEXT, "glProvokingVertexEXT");
    /* GL_EXT_secondary_color */
    use_gl!(p_glSecondaryColor3fEXT, "glSecondaryColor3fEXT");
    use_gl!(p_glSecondaryColor3fvEXT, "glSecondaryColor3fvEXT");
    use_gl!(p_glSecondaryColor3ubEXT, "glSecondaryColor3ubEXT");
    use_gl!(p_glSecondaryColor3ubvEXT, "glSecondaryColor3ubvEXT");
    use_gl!(p_glSecondaryColorPointerEXT, "glSecondaryColorPointerEXT");
    /* GL_EXT_stencil_two_side */
    use_gl!(p_glActiveStencilFaceEXT, "glActiveStencilFaceEXT");
    /* GL_EXT_texture3D */
    use_gl!(p_glTexImage3D, "glTexImage3D");
    use_gl!(p_glTexImage3DEXT, "glTexImage3DEXT");
    use_gl!(p_glTexSubImage3D, "glTexSubImage3D");
    use_gl!(p_glTexSubImage3DEXT, "glTexSubImage3DEXT");
    /* GL_NV_fence */
    use_gl!(p_glDeleteFencesNV, "glDeleteFencesNV");
    use_gl!(p_glFinishFenceNV, "glFinishFenceNV");
    use_gl!(p_glGenFencesNV, "glGenFencesNV");
    use_gl!(p_glGetFenceivNV, "glGetFenceivNV");
    use_gl!(p_glIsFenceNV, "glIsFenceNV");
    use_gl!(p_glSetFenceNV, "glSetFenceNV");
    use_gl!(p_glTestFenceNV, "glTestFenceNV");
    /* GL_NV_half_float */
    use_gl!(p_glColor3hNV, "glColor3hNV");
    use_gl!(p_glColor3hvNV, "glColor3hvNV");
    use_gl!(p_glColor4hNV, "glColor4hNV");
    use_gl!(p_glColor4hvNV, "glColor4hvNV");
    use_gl!(p_glFogCoordhNV, "glFogCoordhNV");
    use_gl!(p_glFogCoordhvNV, "glFogCoordhvNV");
    use_gl!(p_glMultiTexCoord1hNV, "glMultiTexCoord1hNV");
    use_gl!(p_glMultiTexCoord1hvNV, "glMultiTexCoord1hvNV");
    use_gl!(p_glMultiTexCoord2hNV, "glMultiTexCoord2hNV");
    use_gl!(p_glMultiTexCoord2hvNV, "glMultiTexCoord2hvNV");
    use_gl!(p_glMultiTexCoord3hNV, "glMultiTexCoord3hNV");
    use_gl!(p_glMultiTexCoord3hvNV, "glMultiTexCoord3hvNV");
    use_gl!(p_glMultiTexCoord4hNV, "glMultiTexCoord4hNV");
    use_gl!(p_glMultiTexCoord4hvNV, "glMultiTexCoord4hvNV");
    use_gl!(p_glNormal3hNV, "glNormal3hNV");
    use_gl!(p_glNormal3hvNV, "glNormal3hvNV");
    use_gl!(p_glSecondaryColor3hNV, "glSecondaryColor3hNV");
    use_gl!(p_glSecondaryColor3hvNV, "glSecondaryColor3hvNV");
    use_gl!(p_glTexCoord1hNV, "glTexCoord1hNV");
    use_gl!(p_glTexCoord1hvNV, "glTexCoord1hvNV");
    use_gl!(p_glTexCoord2hNV, "glTexCoord2hNV");
    use_gl!(p_glTexCoord2hvNV, "glTexCoord2hvNV");
    use_gl!(p_glTexCoord3hNV, "glTexCoord3hNV");
    use_gl!(p_glTexCoord3hvNV, "glTexCoord3hvNV");
    use_gl!(p_glTexCoord4hNV, "glTexCoord4hNV");
    use_gl!(p_glTexCoord4hvNV, "glTexCoord4hvNV");
    use_gl!(p_glVertex2hNV, "glVertex2hNV");
    use_gl!(p_glVertex2hvNV, "glVertex2hvNV");
    use_gl!(p_glVertex3hNV, "glVertex3hNV");
    use_gl!(p_glVertex3hvNV, "glVertex3hvNV");
    use_gl!(p_glVertex4hNV, "glVertex4hNV");
    use_gl!(p_glVertex4hvNV, "glVertex4hvNV");
    use_gl!(p_glVertexAttrib1hNV, "glVertexAttrib1hNV");
    use_gl!(p_glVertexAttrib1hvNV, "glVertexAttrib1hvNV");
    use_gl!(p_glVertexAttrib2hNV, "glVertexAttrib2hNV");
    use_gl!(p_glVertexAttrib2hvNV, "glVertexAttrib2hvNV");
    use_gl!(p_glVertexAttrib3hNV, "glVertexAttrib3hNV");
    use_gl!(p_glVertexAttrib3hvNV, "glVertexAttrib3hvNV");
    use_gl!(p_glVertexAttrib4hNV, "glVertexAttrib4hNV");
    use_gl!(p_glVertexAttrib4hvNV, "glVertexAttrib4hvNV");
    use_gl!(p_glVertexAttribs1hvNV, "glVertexAttribs1hvNV");
    use_gl!(p_glVertexAttribs2hvNV, "glVertexAttribs2hvNV");
    use_gl!(p_glVertexAttribs3hvNV, "glVertexAttribs3hvNV");
    use_gl!(p_glVertexAttribs4hvNV, "glVertexAttribs4hvNV");
    use_gl!(p_glVertexWeighthNV, "glVertexWeighthNV");
    use_gl!(p_glVertexWeighthvNV, "glVertexWeighthvNV");
    /* GL_NV_point_sprite */
    use_gl!(p_glPointParameteriNV, "glPointParameteriNV");
    use_gl!(p_glPointParameterivNV, "glPointParameterivNV");
    /* GL_NV_register_combiners */
    use_gl!(p_glCombinerInputNV, "glCombinerInputNV");
    use_gl!(p_glCombinerOutputNV, "glCombinerOutputNV");
    use_gl!(p_glCombinerParameterfNV, "glCombinerParameterfNV");
    use_gl!(p_glCombinerParameterfvNV, "glCombinerParameterfvNV");
    use_gl!(p_glCombinerParameteriNV, "glCombinerParameteriNV");
    use_gl!(p_glCombinerParameterivNV, "glCombinerParameterivNV");
    use_gl!(p_glFinalCombinerInputNV, "glFinalCombinerInputNV");
    /* GL_NV_texture_barrier */
    use_gl!(p_glTextureBarrierNV, "glTextureBarrierNV");
    /* WGL extensions */
    use_gl!(p_wglChoosePixelFormatARB, "wglChoosePixelFormatARB");
    use_gl!(p_wglGetExtensionsStringARB, "wglGetExtensionsStringARB");
    use_gl!(p_wglGetPixelFormatAttribfvARB, "wglGetPixelFormatAttribfvARB");
    use_gl!(p_wglGetPixelFormatAttribivARB, "wglGetPixelFormatAttribivARB");
    use_gl!(p_wglQueryCurrentRendererIntegerWINE, "wglQueryCurrentRendererIntegerWINE");
    use_gl!(p_wglQueryCurrentRendererStringWINE, "wglQueryCurrentRendererStringWINE");
    use_gl!(p_wglQueryRendererIntegerWINE, "wglQueryRendererIntegerWINE");
    use_gl!(p_wglQueryRendererStringWINE, "wglQueryRendererStringWINE");
    use_gl!(p_wglSetPixelFormatWINE, "wglSetPixelFormatWINE");
    use_gl!(p_wglSwapIntervalEXT, "wglSwapIntervalEXT");

    /* Newer core functions */
    use_gl!(p_glActiveTexture, "glActiveTexture");
    use_gl!(p_glAttachShader, "glAttachShader");
    use_gl!(p_glBeginQuery, "glBeginQuery");
    use_gl!(p_glBeginTransformFeedback, "glBeginTransformFeedback");
    use_gl!(p_glBindAttribLocation, "glBindAttribLocation");
    use_gl!(p_glBindBuffer, "glBindBuffer");
    use_gl!(p_glBindFragDataLocation, "glBindFragDataLocation");
    use_gl!(p_glBindVertexArray, "glBindVertexArray");
    use_gl!(p_glBlendColor, "glBlendColor");
    use_gl!(p_glBlendEquation, "glBlendEquation");
    use_gl!(p_glBlendEquationi, "glBlendEquationi");
    use_gl!(p_glBlendEquationSeparate, "glBlendEquationSeparate");
    use_gl!(p_glBlendEquationSeparatei, "glBlendEquationSeparatei");
    use_gl!(p_glBlendFunci, "glBlendFunci");
    use_gl!(p_glBlendFuncSeparate, "glBlendFuncSeparate");
    use_gl!(p_glBlendFuncSeparatei, "glBlendFuncSeparatei");
    use_gl!(p_glBufferData, "glBufferData");
    use_gl!(p_glBufferSubData, "glBufferSubData");
    use_gl!(p_glColorMaski, "glColorMaski");
    use_gl!(p_glCompileShader, "glCompileShader");
    use_gl!(p_glCompressedTexImage2D, "glCompressedTexImage2D");
    use_gl!(p_glCompressedTexImage3D, "glCompressedTexImage3D");
    use_gl!(p_glCompressedTexSubImage2D, "glCompressedTexSubImage2D");
    use_gl!(p_glCompressedTexSubImage3D, "glCompressedTexSubImage3D");
    use_gl!(p_glCreateProgram, "glCreateProgram");
    use_gl!(p_glCreateShader, "glCreateShader");
    use_gl!(p_glDebugMessageCallback, "glDebugMessageCallback");
    use_gl!(p_glDebugMessageControl, "glDebugMessageControl");
    use_gl!(p_glDebugMessageInsert, "glDebugMessageInsert");
    use_gl!(p_glDeleteBuffers, "glDeleteBuffers");
    use_gl!(p_glDeleteProgram, "glDeleteProgram");
    use_gl!(p_glDeleteQueries, "glDeleteQueries");
    use_gl!(p_glDeleteShader, "glDeleteShader");
    use_gl!(p_glDeleteVertexArrays, "glDeleteVertexArrays");
    use_gl!(p_glDetachShader, "glDetachShader");
    use_gl!(p_glDisablei, "glDisablei");
    use_gl!(p_glDisableVertexAttribArray, "glDisableVertexAttribArray");
    use_gl!(p_glDrawArraysInstanced, "glDrawArraysInstanced");
    use_gl!(p_glDrawBuffers, "glDrawBuffers");
    use_gl!(p_glDrawElementsInstanced, "glDrawElementsInstanced");
    use_gl!(p_glEnablei, "glEnablei");
    use_gl!(p_glEnableVertexAttribArray, "glEnableVertexAttribArray");
    use_gl!(p_glEndQuery, "glEndQuery");
    use_gl!(p_glEndTransformFeedback, "glEndTransformFeedback");
    use_gl!(p_glFramebufferTexture, "glFramebufferTexture");
    use_gl!(p_glGenBuffers, "glGenBuffers");
    use_gl!(p_glGenQueries, "glGenQueries");
    use_gl!(p_glGenVertexArrays, "glGenVertexArrays");
    use_gl!(p_glGetActiveUniform, "glGetActiveUniform");
    use_gl!(p_glGetAttachedShaders, "glGetAttachedShaders");
    use_gl!(p_glGetAttribLocation, "glGetAttribLocation");
    use_gl!(p_glGetBooleani_v, "glGetBooleani_v");
    use_gl!(p_glGetBufferSubData, "glGetBufferSubData");
    use_gl!(p_glGetCompressedTexImage, "glGetCompressedTexImage");
    use_gl!(p_glGetDebugMessageLog, "glGetDebugMessageLog");
    use_gl!(p_glGetIntegeri_v, "glGetIntegeri_v");
    use_gl!(p_glGetProgramInfoLog, "glGetProgramInfoLog");
    use_gl!(p_glGetProgramiv, "glGetProgramiv");
    use_gl!(p_glGetQueryiv, "glGetQueryiv");
    use_gl!(p_glGetQueryObjectuiv, "glGetQueryObjectuiv");
    use_gl!(p_glGetShaderInfoLog, "glGetShaderInfoLog");
    use_gl!(p_glGetShaderiv, "glGetShaderiv");
    use_gl!(p_glGetShaderSource, "glGetShaderSource");
    use_gl!(p_glGetStringi, "glGetStringi");
    use_gl!(p_glGetTextureLevelParameteriv, "glGetTextureLevelParameteriv");
    use_gl!(p_glGetTextureParameteriv, "glGetTextureParameteriv");
    use_gl!(p_glGetUniformfv, "glGetUniformfv");
    use_gl!(p_glGetUniformiv, "glGetUniformiv");
    use_gl!(p_glGetUniformLocation, "glGetUniformLocation");
    use_gl!(p_glIsEnabledi, "glIsEnabledi");
    use_gl!(p_glLinkProgram, "glLinkProgram");
    use_gl!(p_glMapBuffer, "glMapBuffer");
    use_gl!(p_glMinSampleShading, "glMinSampleShading");
    use_gl!(p_glPointParameteri, "glPointParameteri");
    use_gl!(p_glPointParameteriv, "glPointParameteriv");
    use_gl!(p_glShaderSource, "glShaderSource");
    use_gl!(p_glStencilFuncSeparate, "glStencilFuncSeparate");
    use_gl!(p_glStencilOpSeparate, "glStencilOpSeparate");
    use_gl!(p_glTexBuffer, "glTexBuffer");
    use_gl!(p_glTexImage3D, "glTexImage3D");
    use_gl!(p_glTexSubImage3D, "glTexSubImage3D");
    use_gl!(p_glTransformFeedbackVaryings, "glTransformFeedbackVaryings");
    use_gl!(p_glUniform1f, "glUniform1f");
    use_gl!(p_glUniform1fv, "glUniform1fv");
    use_gl!(p_glUniform1i, "glUniform1i");
    use_gl!(p_glUniform1iv, "glUniform1iv");
    use_gl!(p_glUniform2f, "glUniform2f");
    use_gl!(p_glUniform2fv, "glUniform2fv");
    use_gl!(p_glUniform2i, "glUniform2i");
    use_gl!(p_glUniform2iv, "glUniform2iv");
    use_gl!(p_glUniform3f, "glUniform3f");
    use_gl!(p_glUniform3fv, "glUniform3fv");
    use_gl!(p_glUniform3i, "glUniform3i");
    use_gl!(p_glUniform3iv, "glUniform3iv");
    use_gl!(p_glUniform4f, "glUniform4f");
    use_gl!(p_glUniform4fv, "glUniform4fv");
    use_gl!(p_glUniform4i, "glUniform4i");
    use_gl!(p_glUniform4iv, "glUniform4iv");
    use_gl!(p_glUniformMatrix2fv, "glUniformMatrix2fv");
    use_gl!(p_glUniformMatrix3fv, "glUniformMatrix3fv");
    use_gl!(p_glUniformMatrix4fv, "glUniformMatrix4fv");
    use_gl!(p_glUnmapBuffer, "glUnmapBuffer");
    use_gl!(p_glUseProgram, "glUseProgram");
    use_gl!(p_glValidateProgram, "glValidateProgram");
    use_gl!(p_glVertexAttrib1f, "glVertexAttrib1f");
    use_gl!(p_glVertexAttrib1fv, "glVertexAttrib1fv");
    use_gl!(p_glVertexAttrib2f, "glVertexAttrib2f");
    use_gl!(p_glVertexAttrib2fv, "glVertexAttrib2fv");
    use_gl!(p_glVertexAttrib3f, "glVertexAttrib3f");
    use_gl!(p_glVertexAttrib3fv, "glVertexAttrib3fv");
    use_gl!(p_glVertexAttrib4f, "glVertexAttrib4f");
    use_gl!(p_glVertexAttrib4fv, "glVertexAttrib4fv");
    use_gl!(p_glVertexAttrib4Nsv, "glVertexAttrib4Nsv");
    use_gl!(p_glVertexAttrib4Nub, "glVertexAttrib4Nub");
    use_gl!(p_glVertexAttrib4Nubv, "glVertexAttrib4Nubv");
    use_gl!(p_glVertexAttrib4Nusv, "glVertexAttrib4Nusv");
    use_gl!(p_glVertexAttrib4sv, "glVertexAttrib4sv");
    use_gl!(p_glVertexAttrib4ubv, "glVertexAttrib4ubv");
    use_gl!(p_glVertexAttribDivisor, "glVertexAttribDivisor");
    use_gl!(p_glVertexAttribIPointer, "glVertexAttribIPointer");
    use_gl!(p_glVertexAttribPointer, "glVertexAttribPointer");

    /* Mapping of core entry points onto extension entry points */
    map_gl!(p_glActiveTexture, p_glActiveTextureARB);
    map_gl!(p_glAttachShader, p_glAttachObjectARB);
    map_gl!(p_glBeginQuery, p_glBeginQueryARB);
    map_gl!(p_glBindAttribLocation, p_glBindAttribLocationARB);
    map_gl!(p_glBindBuffer, p_glBindBufferARB);
    map_gl!(p_glBindFragDataLocation, p_glBindFragDataLocationEXT);
    map_gl!(p_glBlendColor, p_glBlendColorEXT);
    map_gl!(p_glBlendEquation, p_glBlendEquationEXT);
    map_gl!(p_glBlendEquationi, p_glBlendEquationiARB);
    map_gl!(p_glBlendEquationSeparate, p_glBlendEquationSeparateEXT);
    map_gl!(p_glBlendEquationSeparatei, p_glBlendEquationSeparateiARB);
    map_gl!(p_glBlendFunci, p_glBlendFunciARB);
    map_gl!(p_glBlendFuncSeparate, p_glBlendFuncSeparateEXT);
    map_gl!(p_glBlendFuncSeparatei, p_glBlendFuncSeparateiARB);
    map_gl!(p_glBufferData, p_glBufferDataARB);
    map_gl!(p_glBufferSubData, p_glBufferSubDataARB);
    map_gl!(p_glColorMaski, p_glColorMaskIndexedEXT);
    map_gl!(p_glCompileShader, p_glCompileShaderARB);
    map_gl!(p_glCompressedTexImage2D, p_glCompressedTexImage2DARB);
    map_gl!(p_glCompressedTexImage3D, p_glCompressedTexImage3DARB);
    map_gl!(p_glCompressedTexSubImage2D, p_glCompressedTexSubImage2DARB);
    map_gl!(p_glCompressedTexSubImage3D, p_glCompressedTexSubImage3DARB);
    map_gl!(p_glCreateProgram, p_glCreateProgramObjectARB);
    map_gl!(p_glCreateShader, p_glCreateShaderObjectARB);
    map_gl!(p_glDebugMessageCallback, p_glDebugMessageCallbackARB);
    map_gl!(p_glDebugMessageControl, p_glDebugMessageControlARB);
    map_gl!(p_glDebugMessageInsert, p_glDebugMessageInsertARB);
    map_gl!(p_glDeleteBuffers, p_glDeleteBuffersARB);
    map_gl!(p_glDeleteProgram, p_glDeleteObjectARB);
    map_gl!(p_glDeleteQueries, p_glDeleteQueriesARB);
    map_gl!(p_glDeleteShader, p_glDeleteObjectARB);
    map_gl!(p_glDetachShader, p_glDetachObjectARB);
    map_gl!(p_glDisablei, p_glDisableIndexedEXT);
    map_gl!(p_glDisableVertexAttribArray, p_glDisableVertexAttribArrayARB);
    map_gl!(p_glDrawArraysInstanced, p_glDrawArraysInstancedARB);
    map_gl!(p_glDrawBuffers, p_glDrawBuffersARB);
    map_gl!(p_glDrawElementsInstanced, p_glDrawElementsInstancedARB);
    map_gl!(p_glEnablei, p_glEnableIndexedEXT);
    map_gl!(p_glEnableVertexAttribArray, p_glEnableVertexAttribArrayARB);
    map_gl!(p_glEndQuery, p_glEndQueryARB);
    map_gl!(p_glFramebufferTexture, p_glFramebufferTextureARB);
    map_gl!(p_glGenBuffers, p_glGenBuffersARB);
    map_gl!(p_glGenQueries, p_glGenQueriesARB);
    map_gl!(p_glGetActiveUniform, p_glGetActiveUniformARB);
    map_gl!(p_glGetAttachedShaders, p_glGetAttachedObjectsARB);
    map_gl!(p_glGetAttribLocation, p_glGetAttribLocationARB);
    map_gl!(p_glGetBooleani_v, p_glGetBooleanIndexedvEXT);
    map_gl!(p_glGetBufferSubData, p_glGetBufferSubDataARB);
    map_gl!(p_glGetCompressedTexImage, p_glGetCompressedTexImageARB);
    map_gl!(p_glGetDebugMessageLog, p_glGetDebugMessageLogARB);
    map_gl!(p_glGetIntegeri_v, p_glGetIntegerIndexedvEXT);
    map_gl!(p_glGetProgramInfoLog, p_glGetInfoLogARB);
    map_gl!(p_glGetProgramiv, p_glGetObjectParameterivARB);
    map_gl!(p_glGetQueryiv, p_glGetQueryivARB);
    map_gl!(p_glGetQueryObjectuiv, p_glGetQueryObjectuivARB);
    map_gl!(p_glGetShaderInfoLog, p_glGetInfoLogARB);
    map_gl!(p_glGetShaderiv, p_glGetObjectParameterivARB);
    map_gl!(p_glGetShaderSource, p_glGetShaderSourceARB);
    map_gl!(p_glGetUniformfv, p_glGetUniformfvARB);
    map_gl!(p_glGetUniformiv, p_glGetUniformivARB);
    map_gl!(p_glGetUniformLocation, p_glGetUniformLocationARB);
    map_gl!(p_glIsEnabledi, p_glIsEnabledIndexedEXT);
    map_gl!(p_glLinkProgram, p_glLinkProgramARB);
    map_gl!(p_glMapBuffer, p_glMapBufferARB);
    map_gl!(p_glMinSampleShading, p_glMinSampleShadingARB);
    map_gl!(p_glPolygonOffsetClamp, p_glPolygonOffsetClampEXT);
    map_gl!(p_glShaderSource, p_glShaderSourceARB);
    map_gl!(p_glTexBuffer, p_glTexBufferARB);
    map_gl!(p_glTexImage3D, p_glTexImage3DEXT);
    map_gl!(p_glTexSubImage3D, p_glTexSubImage3DEXT);
    map_gl!(p_glUniform1f, p_glUniform1fARB);
    map_gl!(p_glUniform1fv, p_glUniform1fvARB);
    map_gl!(p_glUniform1i, p_glUniform1iARB);
    map_gl!(p_glUniform1iv, p_glUniform1ivARB);
    map_gl!(p_glUniform2f, p_glUniform2fARB);
    map_gl!(p_glUniform2fv, p_glUniform2fvARB);
    map_gl!(p_glUniform2i, p_glUniform2iARB);
    map_gl!(p_glUniform2iv, p_glUniform2ivARB);
    map_gl!(p_glUniform3f, p_glUniform3fARB);
    map_gl!(p_glUniform3fv, p_glUniform3fvARB);
    map_gl!(p_glUniform3i, p_glUniform3iARB);
    map_gl!(p_glUniform3iv, p_glUniform3ivARB);
    map_gl!(p_glUniform4f, p_glUniform4fARB);
    map_gl!(p_glUniform4fv, p_glUniform4fvARB);
    map_gl!(p_glUniform4i, p_glUniform4iARB);
    map_gl!(p_glUniform4iv, p_glUniform4ivARB);
    map_gl!(p_glUniformMatrix2fv, p_glUniformMatrix2fvARB);
    map_gl!(p_glUniformMatrix3fv, p_glUniformMatrix3fvARB);
    map_gl!(p_glUniformMatrix4fv, p_glUniformMatrix4fvARB);
    map_gl!(p_glUnmapBuffer, p_glUnmapBufferARB);
    map_gl!(p_glUseProgram, p_glUseProgramObjectARB);
    map_gl!(p_glValidateProgram, p_glValidateProgramARB);
    map_gl!(p_glVertexAttrib1f, p_glVertexAttrib1fARB);
    map_gl!(p_glVertexAttrib1fv, p_glVertexAttrib1fvARB);
    map_gl!(p_glVertexAttrib2f, p_glVertexAttrib2fARB);
    map_gl!(p_glVertexAttrib2fv, p_glVertexAttrib2fvARB);
    map_gl!(p_glVertexAttrib3f, p_glVertexAttrib3fARB);
    map_gl!(p_glVertexAttrib3fv, p_glVertexAttrib3fvARB);
    map_gl!(p_glVertexAttrib4f, p_glVertexAttrib4fARB);
    map_gl!(p_glVertexAttrib4fv, p_glVertexAttrib4fvARB);
    map_gl!(p_glVertexAttrib4Nsv, p_glVertexAttrib4NsvARB);
    map_gl!(p_glVertexAttrib4Nub, p_glVertexAttrib4NubARB);
    map_gl!(p_glVertexAttrib4Nubv, p_glVertexAttrib4NubvARB);
    map_gl!(p_glVertexAttrib4Nusv, p_glVertexAttrib4NusvARB);
    map_gl!(p_glVertexAttrib4sv, p_glVertexAttrib4svARB);
    map_gl!(p_glVertexAttrib4ubv, p_glVertexAttrib4ubvARB);
    map_gl!(p_glVertexAttribDivisor, p_glVertexAttribDivisorARB);
    map_gl!(p_glVertexAttribIPointer, p_glVertexAttribIPointerEXT);
    map_gl!(p_glVertexAttribPointer, p_glVertexAttribPointerARB);
}

unsafe extern "C" fn wined3d_adapter_init_limits(mut gl_info: *mut wined3d_gl_info) {
    let mut i: ::core::ffi::c_uint = 0;
    let mut sampler_count: ::core::ffi::c_uint = 0;
    let mut gl_max: GLint = 0;
    (*gl_info).limits.buffers = 1;
    (*gl_info).limits.ffp_textures = 0;
    (*gl_info).limits.texture_coords = 0;
    i = 0;
    while i < WINED3D_SHADER_TYPE_COUNT {
        (*gl_info).limits.uniform_blocks[i as usize] = 0;
        (*gl_info).limits.samplers[i as usize] = 0;
        i = i.wrapping_add(1);
    }
    (*gl_info).limits.samplers[WINED3D_SHADER_TYPE_PIXEL as usize] = 1;
    (*gl_info).limits.combined_samplers =
        (*gl_info).limits.samplers[WINED3D_SHADER_TYPE_PIXEL as usize];
    (*gl_info).limits.graphics_samplers = (*gl_info).limits.combined_samplers;
    (*gl_info).limits.vertex_attribs = 16;
    (*gl_info).limits.texture_buffer_offset_alignment = 1;
    (*gl_info).limits.glsl_vs_float_constants = 0;
    (*gl_info).limits.glsl_ps_float_constants = 0;
    (*gl_info).limits.arb_vs_float_constants = 0;
    (*gl_info).limits.arb_vs_native_constants = 0;
    (*gl_info).limits.arb_vs_instructions = 0;
    (*gl_info).limits.arb_vs_temps = 0;
    (*gl_info).limits.arb_ps_float_constants = 0;
    (*gl_info).limits.arb_ps_local_constants = 0;
    (*gl_info).limits.arb_ps_instructions = 0;
    (*gl_info).limits.arb_ps_temps = 0;

    (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
        GL_MAX_CLIP_DISTANCES as GLenum,
        &raw mut gl_max,
    );
    (*gl_info).limits.user_clip_distances =
        min!(WINED3D_MAX_CLIP_DISTANCES, (gl_max) as u32) as ::core::ffi::c_uint;

    if (*gl_info).supported[WINED3D_GL_LEGACY_CONTEXT as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_LIGHTS as GLenum,
            &raw mut gl_max,
        );
        (*gl_info).limits.lights = gl_max as ::core::ffi::c_uint;
    }

    (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
        GL_MAX_TEXTURE_SIZE as GLenum,
        &raw mut gl_max,
    );
    (*gl_info).limits.texture_size = gl_max as ::core::ffi::c_uint;

    if (*gl_info).supported[ARB_MAP_BUFFER_ALIGNMENT as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MIN_MAP_BUFFER_ALIGNMENT as GLenum,
            &raw mut gl_max,
        );
    } else {
        WINE_WARN_!(__wine_dbch_d3d_perf, "Driver doesn't guarantee a minimum buffer map alignment.\n");
    }
    if (*gl_info).supported[NV_REGISTER_COMBINERS as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_GENERAL_COMBINERS_NV as GLenum,
            &raw mut gl_max,
        );
        (*gl_info).limits.general_combiners = gl_max as ::core::ffi::c_uint;
    }
    if (*gl_info).supported[ARB_DRAW_BUFFERS as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_DRAW_BUFFERS_ARB as GLenum,
            &raw mut gl_max,
        );
        (*gl_info).limits.buffers =
            min!(WINED3D_MAX_RENDER_TARGETS, (gl_max) as u32) as ::core::ffi::c_uint;
    }
    if (*gl_info).supported[ARB_MULTITEXTURE as usize] != 0 {
        if (*gl_info).supported[WINED3D_GL_LEGACY_CONTEXT as usize] != 0 {
            (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
                GL_MAX_TEXTURE_UNITS_ARB as GLenum,
                &raw mut gl_max,
            );
            (*gl_info).limits.ffp_textures =
                min!(WINED3D_MAX_FFP_TEXTURES, (gl_max) as u32) as ::core::ffi::c_uint;

            if (*gl_info).supported[ARB_FRAGMENT_PROGRAM as usize] != 0 {
                (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
                    GL_MAX_TEXTURE_COORDS_ARB as GLenum,
                    &raw mut gl_max,
                );
                (*gl_info).limits.texture_coords =
                    min!(WINED3D_MAX_FFP_TEXTURES, (gl_max) as u32) as ::core::ffi::c_uint;
            } else {
                (*gl_info).limits.texture_coords = (*gl_info).limits.ffp_textures;
            }
        }

        if (*gl_info).supported[ARB_FRAGMENT_PROGRAM as usize] != 0
            || (*gl_info).supported[ARB_FRAGMENT_SHADER as usize] != 0
        {
            (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
                GL_MAX_TEXTURE_IMAGE_UNITS as GLenum,
                &raw mut gl_max,
            );
            (*gl_info).limits.samplers[WINED3D_SHADER_TYPE_PIXEL as usize] =
                gl_max as ::core::ffi::c_uint;
        } else {
            (*gl_info).limits.samplers[WINED3D_SHADER_TYPE_PIXEL as usize] =
                (*gl_info).limits.ffp_textures;
        }

        if (*gl_info).supported[ARB_VERTEX_SHADER as usize] != 0 {
            let mut vertex_sampler_count: ::core::ffi::c_uint = 0;

            (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
                GL_MAX_VERTEX_TEXTURE_IMAGE_UNITS_ARB as GLenum,
                &raw mut gl_max,
            );
            (*gl_info).limits.samplers[WINED3D_SHADER_TYPE_VERTEX as usize] =
                gl_max as ::core::ffi::c_uint;
            vertex_sampler_count =
                (*gl_info).limits.samplers[WINED3D_SHADER_TYPE_VERTEX as usize];
            (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
                GL_MAX_COMBINED_TEXTURE_IMAGE_UNITS_ARB as GLenum,
                &raw mut gl_max,
            );
            (*gl_info).limits.combined_samplers = gl_max as ::core::ffi::c_uint;
            (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
                GL_MAX_VERTEX_ATTRIBS_ARB as GLenum,
                &raw mut gl_max,
            );
            (*gl_info).limits.vertex_attribs = gl_max as ::core::ffi::c_uint;

            /* Loading GLSL sampler uniforms is much simpler if we can assume
             * that the sampler setup is known at shader link time. In a
             * vertex shader + pixel shader combination this isn't an issue
             * because then the sampler setup only depends on the two shaders.
             * If a pixel shader is used with fixed-function vertex processing
             * we're fine too because fixed-function vertex processing doesn't
             * use any samplers. If fixed-function fragment processing is used
             * we have to make sure that all vertex sampler setups are valid
             * together with all possible fixed-function fragment processing
             * setups. This is true if vsamplers + WINED3D_MAX_FFP_TEXTURES <= max_samplers.
             * This is true on all Direct3D 9 cards that support vertex
             * texture fetch (GeForce 6 and GeForce 7 cards). Direct3D 9
             * Radeon cards do not support vertex texture fetch. Direct3D 10
             * cards have 128 samplers, and Direct3D 9 is limited to 8
             * fixed-function texture stages and 4 vertex samplers.
             * Direct3D 10 does not have a fixed-function pipeline anymore.
             *
             * So this is just a check to check that our assumption holds
             * true. If not, write a warning and reduce the number of vertex
             * samplers or probably disable vertex texture fetch. */
            if vertex_sampler_count != 0
                && (*gl_info).limits.combined_samplers < 12
                && WINED3D_MAX_FFP_TEXTURES + vertex_sampler_count
                    > (*gl_info).limits.combined_samplers
            {
                if (*gl_info).limits.combined_samplers > WINED3D_MAX_FFP_TEXTURES {
                    vertex_sampler_count =
                        (*gl_info).limits.combined_samplers - WINED3D_MAX_FFP_TEXTURES;
                } else {
                    vertex_sampler_count = 0;
                }
                (*gl_info).limits.samplers[WINED3D_SHADER_TYPE_VERTEX as usize] =
                    vertex_sampler_count;
            }
        } else {
            (*gl_info).limits.combined_samplers =
                (*gl_info).limits.samplers[WINED3D_SHADER_TYPE_PIXEL as usize];
        }
    } else {
        (*gl_info).limits.ffp_textures = 1;
        (*gl_info).limits.texture_coords = 1;
    }

    if (*gl_info).supported[EXT_TEXTURE3D as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_3D_TEXTURE_SIZE_EXT as GLenum,
            &raw mut gl_max,
        );
        (*gl_info).limits.texture3d_size = gl_max as ::core::ffi::c_uint;
    }
    if (*gl_info).supported[ARB_TEXTURE_FILTER_ANISOTROPIC as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_TEXTURE_MAX_ANISOTROPY as GLenum,
            &raw mut gl_max,
        );
        (*gl_info).limits.anisotropy = gl_max as ::core::ffi::c_uint;
    }
    if (*gl_info).supported[ARB_FRAGMENT_PROGRAM as usize] != 0 {
        ((*gl_info).gl_ops.ext.p_glGetProgramivARB.unwrap_unchecked())(
            GL_FRAGMENT_PROGRAM_ARB,
            GL_MAX_PROGRAM_ENV_PARAMETERS_ARB,
            &raw mut gl_max,
        );
        (*gl_info).limits.arb_ps_float_constants = gl_max as ::core::ffi::c_uint;

        ((*gl_info).gl_ops.ext.p_glGetProgramivARB.unwrap_unchecked())(
            GL_FRAGMENT_PROGRAM_ARB,
            GL_MAX_PROGRAM_NATIVE_PARAMETERS_ARB,
            &raw mut gl_max,
        );
        (*gl_info).limits.arb_ps_native_constants = gl_max as ::core::ffi::c_uint;

        ((*gl_info).gl_ops.ext.p_glGetProgramivARB.unwrap_unchecked())(
            GL_FRAGMENT_PROGRAM_ARB,
            GL_MAX_PROGRAM_NATIVE_TEMPORARIES_ARB,
            &raw mut gl_max,
        );
        (*gl_info).limits.arb_ps_temps = gl_max as ::core::ffi::c_uint;

        ((*gl_info).gl_ops.ext.p_glGetProgramivARB.unwrap_unchecked())(
            GL_FRAGMENT_PROGRAM_ARB,
            GL_MAX_PROGRAM_NATIVE_INSTRUCTIONS_ARB,
            &raw mut gl_max,
        );
        (*gl_info).limits.arb_ps_instructions = gl_max as ::core::ffi::c_uint;

        ((*gl_info).gl_ops.ext.p_glGetProgramivARB.unwrap_unchecked())(
            GL_FRAGMENT_PROGRAM_ARB,
            GL_MAX_PROGRAM_LOCAL_PARAMETERS_ARB,
            &raw mut gl_max,
        );
        (*gl_info).limits.arb_ps_local_constants = gl_max as ::core::ffi::c_uint;
    }
    if (*gl_info).supported[ARB_VERTEX_PROGRAM as usize] != 0 {
        ((*gl_info).gl_ops.ext.p_glGetProgramivARB.unwrap_unchecked())(
            GL_VERTEX_PROGRAM_ARB,
            GL_MAX_PROGRAM_ENV_PARAMETERS_ARB,
            &raw mut gl_max,
        );
        (*gl_info).limits.arb_vs_float_constants = gl_max as ::core::ffi::c_uint;

        ((*gl_info).gl_ops.ext.p_glGetProgramivARB.unwrap_unchecked())(
            GL_VERTEX_PROGRAM_ARB,
            GL_MAX_PROGRAM_NATIVE_PARAMETERS_ARB,
            &raw mut gl_max,
        );
        (*gl_info).limits.arb_vs_native_constants = gl_max as ::core::ffi::c_uint;

        ((*gl_info).gl_ops.ext.p_glGetProgramivARB.unwrap_unchecked())(
            GL_VERTEX_PROGRAM_ARB,
            GL_MAX_PROGRAM_NATIVE_TEMPORARIES_ARB,
            &raw mut gl_max,
        );
        (*gl_info).limits.arb_vs_temps = gl_max as ::core::ffi::c_uint;

        ((*gl_info).gl_ops.ext.p_glGetProgramivARB.unwrap_unchecked())(
            GL_VERTEX_PROGRAM_ARB,
            GL_MAX_PROGRAM_NATIVE_INSTRUCTIONS_ARB,
            &raw mut gl_max,
        );
        (*gl_info).limits.arb_vs_instructions = gl_max as ::core::ffi::c_uint;
    }
    if (*gl_info).supported[ARB_VERTEX_SHADER as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_VERTEX_UNIFORM_COMPONENTS_ARB as GLenum,
            &raw mut gl_max,
        );
        (*gl_info).limits.glsl_vs_float_constants =
            (gl_max / 4) as ::core::ffi::c_uint;

        if (*gl_info).supported[ARB_UNIFORM_BUFFER_OBJECT as usize] != 0 {
            (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
                GL_MAX_VERTEX_UNIFORM_BLOCKS as GLenum,
                &raw mut gl_max,
            );
            (*gl_info).limits.uniform_blocks[WINED3D_SHADER_TYPE_VERTEX as usize] =
                min!(gl_max, (WINED3D_MAX_CBS as i32)) as ::core::ffi::c_uint;
        }
    }
    if (*gl_info).supported[ARB_TESSELLATION_SHADER as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_TESS_CONTROL_UNIFORM_BLOCKS as GLenum,
            &raw mut gl_max,
        );
        (*gl_info).limits.uniform_blocks[WINED3D_SHADER_TYPE_HULL as usize] =
            min!(gl_max, (WINED3D_MAX_CBS as i32)) as ::core::ffi::c_uint;

        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_TESS_CONTROL_TEXTURE_IMAGE_UNITS as GLenum,
            &raw mut gl_max,
        );
        (*gl_info).limits.samplers[WINED3D_SHADER_TYPE_HULL as usize] =
            gl_max as ::core::ffi::c_uint;

        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_TESS_EVALUATION_UNIFORM_BLOCKS as GLenum,
            &raw mut gl_max,
        );
        (*gl_info).limits.uniform_blocks[WINED3D_SHADER_TYPE_DOMAIN as usize] =
            min!(gl_max, (WINED3D_MAX_CBS as i32)) as ::core::ffi::c_uint;

        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_TESS_EVALUATION_TEXTURE_IMAGE_UNITS as GLenum,
            &raw mut gl_max,
        );
        (*gl_info).limits.samplers[WINED3D_SHADER_TYPE_DOMAIN as usize] =
            gl_max as ::core::ffi::c_uint;
    }
    if (*gl_info).supported[WINED3D_GL_VERSION_3_2 as usize] != 0
        && (*gl_info).supported[ARB_UNIFORM_BUFFER_OBJECT as usize] != 0
    {
        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_GEOMETRY_UNIFORM_BLOCKS as GLenum,
            &raw mut gl_max,
        );
        (*gl_info).limits.uniform_blocks[WINED3D_SHADER_TYPE_GEOMETRY as usize] =
            min!(gl_max, (WINED3D_MAX_CBS as i32)) as ::core::ffi::c_uint;

        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_GEOMETRY_TEXTURE_IMAGE_UNITS as GLenum,
            &raw mut gl_max,
        );
        (*gl_info).limits.samplers[WINED3D_SHADER_TYPE_GEOMETRY as usize] =
            gl_max as ::core::ffi::c_uint;
    }
    if (*gl_info).supported[ARB_FRAGMENT_SHADER as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_FRAGMENT_UNIFORM_COMPONENTS_ARB as GLenum,
            &raw mut gl_max,
        );
        (*gl_info).limits.glsl_ps_float_constants =
            (gl_max / 4) as ::core::ffi::c_uint;

        if (*gl_info).supported[WINED3D_GL_LEGACY_CONTEXT as usize] != 0 {
            (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
                GL_MAX_VARYING_FLOATS_ARB as GLenum,
                &raw mut gl_max,
            );
            (*gl_info).limits.glsl_varyings = gl_max as ::core::ffi::c_uint;
        } else {
            (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
                GL_MAX_FRAGMENT_INPUT_COMPONENTS as GLenum,
                &raw mut gl_max,
            );
            (*gl_info).limits.glsl_varyings =
                (gl_max - 4) as ::core::ffi::c_uint;
        }

        if (*gl_info).supported[ARB_UNIFORM_BUFFER_OBJECT as usize] != 0 {
            (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
                GL_MAX_FRAGMENT_UNIFORM_BLOCKS as GLenum,
                &raw mut gl_max,
            );
            (*gl_info).limits.uniform_blocks[WINED3D_SHADER_TYPE_PIXEL as usize] =
                min!(gl_max, (WINED3D_MAX_CBS as i32)) as ::core::ffi::c_uint;
        }
    }
    if (*gl_info).supported[ARB_COMPUTE_SHADER as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_COMPUTE_UNIFORM_BLOCKS as GLenum,
            &raw mut gl_max,
        );
        (*gl_info).limits.uniform_blocks[WINED3D_SHADER_TYPE_COMPUTE as usize] =
            min!(gl_max, (WINED3D_MAX_CBS as i32)) as ::core::ffi::c_uint;

        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_COMPUTE_TEXTURE_IMAGE_UNITS as GLenum,
            &raw mut gl_max,
        );
        (*gl_info).limits.samplers[WINED3D_SHADER_TYPE_COMPUTE as usize] =
            gl_max as ::core::ffi::c_uint;
    }
    if (*gl_info).supported[ARB_UNIFORM_BUFFER_OBJECT as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_COMBINED_UNIFORM_BLOCKS as GLenum,
            &raw mut gl_max,
        );

        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_UNIFORM_BUFFER_BINDINGS as GLenum,
            &raw mut gl_max,
        );
    }
    if (*gl_info).supported[ARB_TEXTURE_BUFFER_RANGE as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_TEXTURE_BUFFER_OFFSET_ALIGNMENT as GLenum,
            &raw mut gl_max,
        );
        (*gl_info).limits.texture_buffer_offset_alignment = gl_max as ::core::ffi::c_uint;
    }
    if (*gl_info).supported[ARB_SHADER_ATOMIC_COUNTERS as usize] != 0 {
        let mut max_fragment_buffers: GLint = 0;
        let mut max_combined_buffers: GLint = 0;
        let mut max_bindings: GLint = 0;
        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_FRAGMENT_ATOMIC_COUNTER_BUFFERS as GLenum,
            &raw mut max_fragment_buffers,
        );

        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_COMBINED_ATOMIC_COUNTER_BUFFERS as GLenum,
            &raw mut max_combined_buffers,
        );

        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_ATOMIC_COUNTER_BUFFER_BINDINGS as GLenum,
            &raw mut max_bindings,
        );

        if max_fragment_buffers < (MAX_UNORDERED_ACCESS_VIEWS as GLint)
            || max_combined_buffers < (MAX_UNORDERED_ACCESS_VIEWS as GLint)
            || max_bindings < (MAX_UNORDERED_ACCESS_VIEWS as GLint)
        {
            WARN!("Disabling ARB_shader_atomic_counters.\n");
            (*gl_info).supported[ARB_SHADER_ATOMIC_COUNTERS as usize] =
                (FALSE as i32);
        }
    }
    if (*gl_info).supported[ARB_TRANSFORM_FEEDBACK3 as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_VERTEX_STREAMS as GLenum,
            &raw mut gl_max,
        );
    }

    if (*gl_info).supported[NV_LIGHT_MAX_EXPONENT as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glGetFloatv.unwrap_unchecked()(
            GL_MAX_SHININESS_NV as GLenum,
            &raw mut (*gl_info).limits.shininess,
        );
    } else {
        (*gl_info).limits.shininess = 128.0f32;
    }

    if (*gl_info).supported[ARB_FRAMEBUFFER_OBJECT as usize] != 0
        || (*gl_info).supported[EXT_FRAMEBUFFER_MULTISAMPLE as usize] != 0
    {
        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_SAMPLES as GLenum,
            &raw mut gl_max,
        );
        (*gl_info).limits.samples = gl_max as ::core::ffi::c_uint;
    }

    if (*gl_info).supported[ARB_FRAMEBUFFER_NO_ATTACHMENTS as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_FRAMEBUFFER_WIDTH as GLenum,
            &raw mut gl_max,
        );
        (*gl_info).limits.framebuffer_width = gl_max as ::core::ffi::c_uint;
        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
            GL_MAX_FRAMEBUFFER_HEIGHT as GLenum,
            &raw mut gl_max,
        );
        (*gl_info).limits.framebuffer_height = gl_max as ::core::ffi::c_uint;
    } else {
        (*gl_info).limits.framebuffer_width = (*gl_info).limits.texture_size;
        (*gl_info).limits.framebuffer_height = (*gl_info).limits.texture_size;
    }

    (*gl_info).limits.samplers[WINED3D_SHADER_TYPE_PIXEL as usize] =
        min!((*gl_info).limits.samplers[WINED3D_SHADER_TYPE_PIXEL as usize], MAX_GL_FRAGMENT_SAMPLERS)
            as ::core::ffi::c_uint;
    sampler_count = 0;
    i = 0;
    while i < WINED3D_SHADER_TYPE_GRAPHICS_COUNT {
        sampler_count = sampler_count.wrapping_add((*gl_info).limits.samplers[i as usize]);
        i = i.wrapping_add(1);
    }
    if (*gl_info).supported[WINED3D_GL_VERSION_3_2 as usize] != 0
        && (*gl_info).limits.combined_samplers < sampler_count
    {
        /* The minimum value for GL_MAX_COMBINED_TEXTURE_IMAGE_UNITS in OpenGL
         * 3.2 is 48 (16 per stage). When tessellation shaders are supported
         * the minimum value is increased to 80. */
        WARN!(
            "Graphics pipeline sampler count %u is greater than combined sampler count %u.\n",
            sampler_count,
            (*gl_info).limits.combined_samplers
        );
        i = 0;
        while i < WINED3D_SHADER_TYPE_GRAPHICS_COUNT {
            (*gl_info).limits.samplers[i as usize] = min!(
                (*gl_info).limits.samplers[i as usize],
                16,
            ) as ::core::ffi::c_uint;
            i = i.wrapping_add(1);
        }
    }

    /* A majority of OpenGL implementations allow us to statically partition
     * the set of texture bindings into six separate sets. */
    (*gl_info).limits.graphics_samplers = (*gl_info).limits.combined_samplers;
    sampler_count = 0;
    i = 0;
    while i < WINED3D_SHADER_TYPE_COUNT {
        sampler_count = sampler_count.wrapping_add((*gl_info).limits.samplers[i as usize]);
        i = i.wrapping_add(1);
    }
    if (*gl_info).limits.combined_samplers >= sampler_count {
        (*gl_info).limits.graphics_samplers = (*gl_info).limits.graphics_samplers.wrapping_sub(
            (*gl_info).limits.samplers[WINED3D_SHADER_TYPE_COMPUTE as usize],
        );
    }
}

unsafe extern "C" fn wined3d_adapter_init_gl_caps(
    adapter_gl: *mut wined3d_adapter_gl,
    caps_gl_ctx: *mut wined3d_caps_gl_ctx,
    wined3d_creation_flags: ::core::ffi::c_uint,
) -> BOOL {
    static core_extensions: [C2RustUnnamed_53; 113] = [
        C2RustUnnamed_53 { extension: EXT_TEXTURE3D, min_gl_version: MAKEDWORD_VERSION!(1, 2) },
        C2RustUnnamed_53 { extension: ARB_MULTISAMPLE, min_gl_version: MAKEDWORD_VERSION!(1, 3) },
        C2RustUnnamed_53 { extension: ARB_MULTITEXTURE, min_gl_version: MAKEDWORD_VERSION!(1, 3) },
        C2RustUnnamed_53 { extension: ARB_TEXTURE_BORDER_CLAMP, min_gl_version: MAKEDWORD_VERSION!(1, 3) },
        C2RustUnnamed_53 { extension: ARB_TEXTURE_COMPRESSION, min_gl_version: MAKEDWORD_VERSION!(1, 3) },
        C2RustUnnamed_53 { extension: ARB_TEXTURE_CUBE_MAP, min_gl_version: MAKEDWORD_VERSION!(1, 3) },
        C2RustUnnamed_53 { extension: ARB_DEPTH_TEXTURE, min_gl_version: MAKEDWORD_VERSION!(1, 4) },
        C2RustUnnamed_53 { extension: ARB_POINT_PARAMETERS, min_gl_version: MAKEDWORD_VERSION!(1, 4) },
        C2RustUnnamed_53 { extension: ARB_SHADOW, min_gl_version: MAKEDWORD_VERSION!(1, 4) },
        C2RustUnnamed_53 { extension: ARB_TEXTURE_MIRRORED_REPEAT, min_gl_version: MAKEDWORD_VERSION!(1, 4) },
        C2RustUnnamed_53 { extension: EXT_BLEND_COLOR, min_gl_version: MAKEDWORD_VERSION!(1, 4) },
        C2RustUnnamed_53 { extension: EXT_BLEND_FUNC_SEPARATE, min_gl_version: MAKEDWORD_VERSION!(1, 4) },
        C2RustUnnamed_53 { extension: EXT_BLEND_MINMAX, min_gl_version: MAKEDWORD_VERSION!(1, 4) },
        C2RustUnnamed_53 { extension: EXT_BLEND_SUBTRACT, min_gl_version: MAKEDWORD_VERSION!(1, 4) },
        C2RustUnnamed_53 { extension: EXT_STENCIL_WRAP, min_gl_version: MAKEDWORD_VERSION!(1, 4) },
        C2RustUnnamed_53 { extension: NV_POINT_SPRITE, min_gl_version: MAKEDWORD_VERSION!(1, 4) },
        C2RustUnnamed_53 { extension: ARB_OCCLUSION_QUERY, min_gl_version: MAKEDWORD_VERSION!(1, 5) },
        C2RustUnnamed_53 { extension: ARB_VERTEX_BUFFER_OBJECT, min_gl_version: MAKEDWORD_VERSION!(1, 5) },
        C2RustUnnamed_53 { extension: ARB_DRAW_BUFFERS, min_gl_version: MAKEDWORD_VERSION!(2, 0) },
        C2RustUnnamed_53 { extension: ARB_FRAGMENT_SHADER, min_gl_version: MAKEDWORD_VERSION!(2, 0) },
        C2RustUnnamed_53 { extension: ARB_SHADING_LANGUAGE_100, min_gl_version: MAKEDWORD_VERSION!(2, 0) },
        C2RustUnnamed_53 { extension: ARB_TEXTURE_NON_POWER_OF_TWO, min_gl_version: MAKEDWORD_VERSION!(2, 0) },
        C2RustUnnamed_53 { extension: ARB_VERTEX_SHADER, min_gl_version: MAKEDWORD_VERSION!(2, 0) },
        C2RustUnnamed_53 { extension: EXT_BLEND_EQUATION_SEPARATE, min_gl_version: MAKEDWORD_VERSION!(2, 0) },
        C2RustUnnamed_53 { extension: ARB_PIXEL_BUFFER_OBJECT, min_gl_version: MAKEDWORD_VERSION!(2, 1) },
        C2RustUnnamed_53 { extension: EXT_TEXTURE_SRGB, min_gl_version: MAKEDWORD_VERSION!(2, 1) },
        C2RustUnnamed_53 { extension: ARB_COLOR_BUFFER_FLOAT, min_gl_version: MAKEDWORD_VERSION!(3, 0) },
        C2RustUnnamed_53 { extension: ARB_DEPTH_BUFFER_FLOAT, min_gl_version: MAKEDWORD_VERSION!(3, 0) },
        C2RustUnnamed_53 { extension: ARB_FRAMEBUFFER_OBJECT, min_gl_version: MAKEDWORD_VERSION!(3, 0) },
        C2RustUnnamed_53 { extension: ARB_FRAMEBUFFER_SRGB, min_gl_version: MAKEDWORD_VERSION!(3, 0) },
        C2RustUnnamed_53 { extension: ARB_HALF_FLOAT_PIXEL, min_gl_version: MAKEDWORD_VERSION!(3, 0) },
        C2RustUnnamed_53 { extension: ARB_HALF_FLOAT_VERTEX, min_gl_version: MAKEDWORD_VERSION!(3, 0) },
        C2RustUnnamed_53 { extension: ARB_MAP_BUFFER_RANGE, min_gl_version: MAKEDWORD_VERSION!(3, 0) },
        C2RustUnnamed_53 { extension: ARB_TEXTURE_COMPRESSION_RGTC, min_gl_version: MAKEDWORD_VERSION!(3, 0) },
        C2RustUnnamed_53 { extension: ARB_TEXTURE_FLOAT, min_gl_version: MAKEDWORD_VERSION!(3, 0) },
        C2RustUnnamed_53 { extension: ARB_TEXTURE_RG, min_gl_version: MAKEDWORD_VERSION!(3, 0) },
        C2RustUnnamed_53 { extension: EXT_DRAW_BUFFERS2, min_gl_version: MAKEDWORD_VERSION!(3, 0) },
        C2RustUnnamed_53 { extension: EXT_PACKED_FLOAT, min_gl_version: MAKEDWORD_VERSION!(3, 0) },
        C2RustUnnamed_53 { extension: EXT_TEXTURE_ARRAY, min_gl_version: MAKEDWORD_VERSION!(3, 0) },
        C2RustUnnamed_53 { extension: EXT_TEXTURE_INTEGER, min_gl_version: MAKEDWORD_VERSION!(3, 0) },
        C2RustUnnamed_53 { extension: EXT_TEXTURE_SHARED_EXPONENT, min_gl_version: MAKEDWORD_VERSION!(3, 0) },
        C2RustUnnamed_53 { extension: ARB_COPY_BUFFER, min_gl_version: MAKEDWORD_VERSION!(3, 1) },
        C2RustUnnamed_53 { extension: ARB_DRAW_INSTANCED, min_gl_version: MAKEDWORD_VERSION!(3, 1) },
        C2RustUnnamed_53 { extension: ARB_TEXTURE_BUFFER_OBJECT, min_gl_version: MAKEDWORD_VERSION!(3, 1) },
        C2RustUnnamed_53 { extension: ARB_UNIFORM_BUFFER_OBJECT, min_gl_version: MAKEDWORD_VERSION!(3, 1) },
        C2RustUnnamed_53 { extension: EXT_TEXTURE_SNORM, min_gl_version: MAKEDWORD_VERSION!(3, 1) },
        C2RustUnnamed_53 { extension: ARB_DEPTH_CLAMP, min_gl_version: MAKEDWORD_VERSION!(3, 2) },
        C2RustUnnamed_53 { extension: ARB_DRAW_ELEMENTS_BASE_VERTEX, min_gl_version: MAKEDWORD_VERSION!(3, 2) },
        C2RustUnnamed_53 { extension: ARB_FRAGMENT_COORD_CONVENTIONS, min_gl_version: MAKEDWORD_VERSION!(3, 2) },
        C2RustUnnamed_53 { extension: ARB_PROVOKING_VERTEX, min_gl_version: MAKEDWORD_VERSION!(3, 2) },
        C2RustUnnamed_53 { extension: ARB_SEAMLESS_CUBE_MAP, min_gl_version: MAKEDWORD_VERSION!(3, 2) },
        C2RustUnnamed_53 { extension: ARB_SYNC, min_gl_version: MAKEDWORD_VERSION!(3, 2) },
        C2RustUnnamed_53 { extension: ARB_TEXTURE_MULTISAMPLE, min_gl_version: MAKEDWORD_VERSION!(3, 2) },
        C2RustUnnamed_53 { extension: ARB_VERTEX_ARRAY_BGRA, min_gl_version: MAKEDWORD_VERSION!(3, 2) },
        C2RustUnnamed_53 { extension: ARB_BLEND_FUNC_EXTENDED, min_gl_version: MAKEDWORD_VERSION!(3, 3) },
        C2RustUnnamed_53 { extension: ARB_EXPLICIT_ATTRIB_LOCATION, min_gl_version: MAKEDWORD_VERSION!(3, 3) },
        C2RustUnnamed_53 { extension: ARB_INSTANCED_ARRAYS, min_gl_version: MAKEDWORD_VERSION!(3, 3) },
        C2RustUnnamed_53 { extension: ARB_SAMPLER_OBJECTS, min_gl_version: MAKEDWORD_VERSION!(3, 3) },
        C2RustUnnamed_53 { extension: ARB_SHADER_BIT_ENCODING, min_gl_version: MAKEDWORD_VERSION!(3, 3) },
        C2RustUnnamed_53 { extension: ARB_TEXTURE_RGB10_A2UI, min_gl_version: MAKEDWORD_VERSION!(3, 3) },
        C2RustUnnamed_53 { extension: ARB_TEXTURE_SWIZZLE, min_gl_version: MAKEDWORD_VERSION!(3, 3) },
        C2RustUnnamed_53 { extension: ARB_TIMER_QUERY, min_gl_version: MAKEDWORD_VERSION!(3, 3) },
        C2RustUnnamed_53 { extension: ARB_VERTEX_TYPE_2_10_10_10_REV, min_gl_version: MAKEDWORD_VERSION!(3, 3) },
        C2RustUnnamed_53 { extension: ARB_DRAW_BUFFERS_BLEND, min_gl_version: MAKEDWORD_VERSION!(4, 0) },
        C2RustUnnamed_53 { extension: ARB_DRAW_INDIRECT, min_gl_version: MAKEDWORD_VERSION!(4, 0) },
        C2RustUnnamed_53 { extension: ARB_GPU_SHADER5, min_gl_version: MAKEDWORD_VERSION!(4, 0) },
        C2RustUnnamed_53 { extension: ARB_SAMPLE_SHADING, min_gl_version: MAKEDWORD_VERSION!(4, 0) },
        C2RustUnnamed_53 { extension: ARB_TESSELLATION_SHADER, min_gl_version: MAKEDWORD_VERSION!(4, 0) },
        C2RustUnnamed_53 { extension: ARB_TEXTURE_CUBE_MAP_ARRAY, min_gl_version: MAKEDWORD_VERSION!(4, 0) },
        C2RustUnnamed_53 { extension: ARB_TEXTURE_GATHER, min_gl_version: MAKEDWORD_VERSION!(4, 0) },
        C2RustUnnamed_53 { extension: ARB_TRANSFORM_FEEDBACK2, min_gl_version: MAKEDWORD_VERSION!(4, 0) },
        C2RustUnnamed_53 { extension: ARB_TRANSFORM_FEEDBACK3, min_gl_version: MAKEDWORD_VERSION!(4, 0) },
        C2RustUnnamed_53 { extension: ARB_ES2_COMPATIBILITY, min_gl_version: MAKEDWORD_VERSION!(4, 1) },
        C2RustUnnamed_53 { extension: ARB_VIEWPORT_ARRAY, min_gl_version: MAKEDWORD_VERSION!(4, 1) },
        C2RustUnnamed_53 { extension: ARB_BASE_INSTANCE, min_gl_version: MAKEDWORD_VERSION!(4, 2) },
        C2RustUnnamed_53 { extension: ARB_CONSERVATIVE_DEPTH, min_gl_version: MAKEDWORD_VERSION!(4, 2) },
        C2RustUnnamed_53 { extension: ARB_INTERNALFORMAT_QUERY, min_gl_version: MAKEDWORD_VERSION!(4, 2) },
        C2RustUnnamed_53 { extension: ARB_MAP_BUFFER_ALIGNMENT, min_gl_version: MAKEDWORD_VERSION!(4, 2) },
        C2RustUnnamed_53 { extension: ARB_SHADER_ATOMIC_COUNTERS, min_gl_version: MAKEDWORD_VERSION!(4, 2) },
        C2RustUnnamed_53 { extension: ARB_SHADER_IMAGE_LOAD_STORE, min_gl_version: MAKEDWORD_VERSION!(4, 2) },
        C2RustUnnamed_53 { extension: ARB_SHADING_LANGUAGE_420PACK, min_gl_version: MAKEDWORD_VERSION!(4, 2) },
        C2RustUnnamed_53 { extension: ARB_SHADING_LANGUAGE_PACKING, min_gl_version: MAKEDWORD_VERSION!(4, 2) },
        C2RustUnnamed_53 { extension: ARB_TEXTURE_COMPRESSION_BPTC, min_gl_version: MAKEDWORD_VERSION!(4, 2) },
        C2RustUnnamed_53 { extension: ARB_TEXTURE_STORAGE, min_gl_version: MAKEDWORD_VERSION!(4, 2) },
        C2RustUnnamed_53 { extension: ARB_CLEAR_BUFFER_OBJECT, min_gl_version: MAKEDWORD_VERSION!(4, 3) },
        C2RustUnnamed_53 { extension: ARB_COMPUTE_SHADER, min_gl_version: MAKEDWORD_VERSION!(4, 3) },
        C2RustUnnamed_53 { extension: ARB_COPY_IMAGE, min_gl_version: MAKEDWORD_VERSION!(4, 3) },
        C2RustUnnamed_53 { extension: ARB_DEBUG_OUTPUT, min_gl_version: MAKEDWORD_VERSION!(4, 3) },
        C2RustUnnamed_53 { extension: ARB_ES3_COMPATIBILITY, min_gl_version: MAKEDWORD_VERSION!(4, 3) },
        C2RustUnnamed_53 { extension: ARB_FRAGMENT_LAYER_VIEWPORT, min_gl_version: MAKEDWORD_VERSION!(4, 3) },
        C2RustUnnamed_53 { extension: ARB_FRAMEBUFFER_NO_ATTACHMENTS, min_gl_version: MAKEDWORD_VERSION!(4, 3) },
        C2RustUnnamed_53 { extension: ARB_INTERNALFORMAT_QUERY2, min_gl_version: MAKEDWORD_VERSION!(4, 3) },
        C2RustUnnamed_53 { extension: ARB_SHADER_IMAGE_SIZE, min_gl_version: MAKEDWORD_VERSION!(4, 3) },
        C2RustUnnamed_53 { extension: ARB_SHADER_STORAGE_BUFFER_OBJECT, min_gl_version: MAKEDWORD_VERSION!(4, 3) },
        C2RustUnnamed_53 { extension: ARB_STENCIL_TEXTURING, min_gl_version: MAKEDWORD_VERSION!(4, 3) },
        C2RustUnnamed_53 { extension: ARB_TEXTURE_BUFFER_RANGE, min_gl_version: MAKEDWORD_VERSION!(4, 3) },
        C2RustUnnamed_53 { extension: ARB_TEXTURE_QUERY_LEVELS, min_gl_version: MAKEDWORD_VERSION!(4, 3) },
        C2RustUnnamed_53 { extension: ARB_TEXTURE_STORAGE_MULTISAMPLE, min_gl_version: MAKEDWORD_VERSION!(4, 3) },
        C2RustUnnamed_53 { extension: ARB_TEXTURE_VIEW, min_gl_version: MAKEDWORD_VERSION!(4, 3) },
        C2RustUnnamed_53 { extension: ARB_BUFFER_STORAGE, min_gl_version: MAKEDWORD_VERSION!(4, 4) },
        C2RustUnnamed_53 { extension: ARB_CLEAR_TEXTURE, min_gl_version: MAKEDWORD_VERSION!(4, 4) },
        C2RustUnnamed_53 { extension: ARB_QUERY_BUFFER_OBJECT, min_gl_version: MAKEDWORD_VERSION!(4, 4) },
        C2RustUnnamed_53 { extension: ARB_TEXTURE_MIRROR_CLAMP_TO_EDGE, min_gl_version: MAKEDWORD_VERSION!(4, 4) },
        C2RustUnnamed_53 { extension: ARB_VERTEX_TYPE_10F_11F_11F_REV, min_gl_version: MAKEDWORD_VERSION!(4, 4) },
        C2RustUnnamed_53 { extension: ARB_CLIP_CONTROL, min_gl_version: MAKEDWORD_VERSION!(4, 5) },
        C2RustUnnamed_53 { extension: ARB_CULL_DISTANCE, min_gl_version: MAKEDWORD_VERSION!(4, 5) },
        C2RustUnnamed_53 { extension: ARB_DERIVATIVE_CONTROL, min_gl_version: MAKEDWORD_VERSION!(4, 5) },
        C2RustUnnamed_53 { extension: ARB_SHADER_TEXTURE_IMAGE_SAMPLES, min_gl_version: MAKEDWORD_VERSION!(4, 5) },
        C2RustUnnamed_53 { extension: ARB_TEXTURE_BARRIER, min_gl_version: MAKEDWORD_VERSION!(4, 5) },
        C2RustUnnamed_53 { extension: EXT_SHADER_INTEGER_MIX, min_gl_version: MAKEDWORD_VERSION!(4, 5) },
        C2RustUnnamed_53 { extension: ARB_PIPELINE_STATISTICS_QUERY, min_gl_version: MAKEDWORD_VERSION!(4, 6) },
        C2RustUnnamed_53 { extension: ARB_POLYGON_OFFSET_CLAMP, min_gl_version: MAKEDWORD_VERSION!(4, 6) },
        C2RustUnnamed_53 { extension: ARB_TEXTURE_FILTER_ANISOTROPIC, min_gl_version: MAKEDWORD_VERSION!(4, 6) },
    ];
    let mut gl_vendor_str: *const ::core::ffi::c_char = ::core::ptr::null();
    let mut gl_renderer_str: *const ::core::ffi::c_char = ::core::ptr::null();
    let mut gl_version_str: *const ::core::ffi::c_char = ::core::ptr::null();
    let gl_info: *mut wined3d_gl_info = &raw mut (*adapter_gl).gl_info;
    let adapter: *mut wined3d_adapter = &raw mut (*adapter_gl).a;
    let mut gl_version: ::core::ffi::c_uint = 0;
    let mut gl_ext_emul_mask: DWORD = 0;
    let mut WGL_Extensions: *const ::core::ffi::c_char = ::core::ptr::null();
    let mut gl_vendor: wined3d_gl_vendor = GL_VENDOR_UNKNOWN;
    let mut context_profile: GLint = 0;
    let mut i: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_uint = 0;
    let mut hdc: HDC = core::ptr::null_mut();

    gl_renderer_str = (*gl_info).gl_ops.gl.p_glGetString.unwrap_unchecked()(GL_RENDERER as GLenum) as *const ::core::ffi::c_char;
    TRACE!("GL_RENDERER: %s.\n", debugstr_a(gl_renderer_str));
    if gl_renderer_str.is_null() {
        ERR!("Received a NULL GL_RENDERER.\n");
        return (FALSE) as BOOL;
    }

    gl_vendor_str = (*gl_info).gl_ops.gl.p_glGetString.unwrap_unchecked()(GL_VENDOR as GLenum) as *const ::core::ffi::c_char;
    TRACE!("GL_VENDOR: %s.\n", debugstr_a(gl_vendor_str));
    if gl_vendor_str.is_null() {
        ERR!("Received a NULL GL_VENDOR.\n");
        return (FALSE) as BOOL;
    }

    /* Parse the GL_VERSION field into major and minor information */
    gl_version_str = (*gl_info).gl_ops.gl.p_glGetString.unwrap_unchecked()(GL_VERSION as GLenum) as *const ::core::ffi::c_char;
    TRACE!("GL_VERSION: %s.\n", debugstr_a(gl_version_str));
    if gl_version_str.is_null() {
        ERR!("Received a NULL GL_VERSION.\n");
        return (FALSE) as BOOL;
    }
    gl_version = wined3d_parse_gl_version(gl_version_str);

    load_gl_funcs(gl_info);

    memset(
        &raw mut (*gl_info).supported as *mut BOOL as *mut ::core::ffi::c_void,
        0,
        ::core::mem::size_of::<[BOOL; 187]>() as size_t,
    );
    (*gl_info).supported[WINED3D_GL_EXT_NONE as usize] = (TRUE as i32);

    if gl_version >= MAKEDWORD_VERSION!(3, 2) {
        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(GL_CONTEXT_PROFILE_MASK as GLenum, &raw mut context_profile);
        checkGLcall!(gl_info, "Querying context profile");
    }
    if context_profile & (GL_CONTEXT_CORE_PROFILE_BIT as GLint) != 0 {
        TRACE!("Got a core profile context.\n");
    } else {
        (*gl_info).supported[WINED3D_GL_LEGACY_CONTEXT as usize] = (TRUE as i32);
    }

    TRACE!("GL extensions reported:\n");
    if (*gl_info).supported[WINED3D_GL_LEGACY_CONTEXT as usize] != 0 {
        let gl_extensions: *const ::core::ffi::c_char = (*gl_info).gl_ops.gl.p_glGetString.unwrap_unchecked()(GL_EXTENSIONS as GLenum) as *const ::core::ffi::c_char;

        if gl_extensions.is_null() {
            ERR!("Received a NULL GL_EXTENSIONS.\n");
            return (FALSE) as BOOL;
        }
        parse_extension_string(gl_info, gl_extensions, &raw const gl_extension_map as *const wined3d_extension_map, ARRAY_SIZE!(gl_extension_map) as UINT);
    } else {
        enumerate_gl_extensions(gl_info, &raw const gl_extension_map as *const wined3d_extension_map, ARRAY_SIZE!(gl_extension_map) as ::core::ffi::c_uint);
    }

    hdc = wglGetCurrentDC();
    /* Not all GL drivers might offer WGL extensions e.g. VirtualBox. */
    if (*gl_info).gl_ops.ext.p_wglGetExtensionsStringARB.is_some() {
        WGL_Extensions = ((*gl_info).gl_ops.ext.p_wglGetExtensionsStringARB.unwrap_unchecked())(hdc) as *const ::core::ffi::c_char;
    }
    if WGL_Extensions.is_null() {
        WARN!("WGL extensions not supported.\n");
    } else {
        parse_extension_string(gl_info, WGL_Extensions, &raw const wgl_extension_map as *const wined3d_extension_map, ARRAY_SIZE!(wgl_extension_map) as UINT);
    }

    i = 0;
    while (i as usize) < ARRAY_SIZE!(core_extensions) as usize {
        if (*gl_info).supported[core_extensions[i as usize].extension as usize] == 0
            && gl_version >= core_extensions[i as usize].min_gl_version
        {
            j = 0;
            while (j as usize) < ARRAY_SIZE!(gl_extension_map) as usize {
                if gl_extension_map[j as usize].extension as ::core::ffi::c_uint
                    == core_extensions[i as usize].extension as ::core::ffi::c_uint
                {
                    break;
                }
                j = j.wrapping_add(1);
            }

            if (j as usize) < ARRAY_SIZE!(gl_extension_map) as usize {
                TRACE!("GL CORE: %s support.\n", gl_extension_map[j as usize].extension_string);
                (*gl_info).supported[core_extensions[i as usize].extension as usize] = (TRUE as i32);
            } else {
                FIXME!("GL extension %u not in the GL extensions map.\n", core_extensions[i as usize].extension as ::core::ffi::c_uint);
            }
        }
        i = i.wrapping_add(1);
    }

    if (*gl_info).supported[EXT_BLEND_MINMAX as usize] != 0 || (*gl_info).supported[EXT_BLEND_SUBTRACT as usize] != 0 {
        (*gl_info).supported[WINED3D_GL_BLEND_EQUATION as usize] = (TRUE as i32);
    }

    if gl_version >= MAKEDWORD_VERSION!(2, 0) {
        (*gl_info).supported[WINED3D_GL_VERSION_2_0 as usize] = (TRUE as i32);
        /* We want to use the core APIs for two-sided stencil in GL 2.0. */
        (*gl_info).supported[EXT_STENCIL_TWO_SIDE as usize] = (FALSE as i32);
    }
    if gl_version >= MAKEDWORD_VERSION!(3, 2) {
        (*gl_info).supported[WINED3D_GL_VERSION_3_2 as usize] = (TRUE as i32);
    }

    /* All the points are actually point sprites in core contexts, the APIs from
     * ARB_point_sprite are not supported anymore. */
    if (*gl_info).supported[WINED3D_GL_LEGACY_CONTEXT as usize] == 0 {
        (*gl_info).supported[ARB_POINT_SPRITE as usize] = (FALSE as i32);
    }

    if (*gl_info).supported[APPLE_FENCE as usize] != 0 {
        /* GL_NV_fence and GL_APPLE_fence provide the same functionality basically.
         * The apple extension interacts with some other apple exts. Disable the NV
         * extension if the apple one is support to prevent confusion in other parts
         * of the code. */
        (*gl_info).supported[NV_FENCE as usize] = (FALSE as i32);
    }
    if (*gl_info).supported[APPLE_FLOAT_PIXELS as usize] != 0 {
        /* GL_APPLE_float_pixels == GL_ARB_texture_float + GL_ARB_half_float_pixel */
        if (*gl_info).supported[ARB_TEXTURE_FLOAT as usize] == 0 {
            TRACE!(" IMPLIED: GL_ARB_texture_float support (by GL_APPLE_float_pixels).\n");
            (*gl_info).supported[ARB_TEXTURE_FLOAT as usize] = (TRUE as i32);
        }
        if (*gl_info).supported[ARB_HALF_FLOAT_PIXEL as usize] == 0 {
            TRACE!(" IMPLIED: GL_ARB_half_float_pixel support (by GL_APPLE_float_pixels).\n");
            (*gl_info).supported[ARB_HALF_FLOAT_PIXEL as usize] = (TRUE as i32);
        }
    }
    if (*gl_info).supported[ARB_MAP_BUFFER_RANGE as usize] != 0 {
        /* GL_ARB_map_buffer_range and GL_APPLE_flush_buffer_range provide the same
         * functionality. Prefer the ARB extension */
        (*gl_info).supported[APPLE_FLUSH_BUFFER_RANGE as usize] = (FALSE as i32);
    }
    if (*gl_info).supported[ARB_TEXTURE_CUBE_MAP as usize] != 0 {
        (*gl_info).supported[NV_TEXGEN_REFLECTION as usize] = (TRUE as i32);
    }
    if (*gl_info).supported[EXT_TEXTURE_COMPRESSION_RGTC as usize] == 0 && (*gl_info).supported[ARB_TEXTURE_COMPRESSION_RGTC as usize] != 0 {
        (*gl_info).supported[EXT_TEXTURE_COMPRESSION_RGTC as usize] = (TRUE as i32);
    }
    if (*gl_info).supported[ARB_TEXTURE_COMPRESSION_RGTC as usize] == 0 && (*gl_info).supported[EXT_TEXTURE_COMPRESSION_RGTC as usize] != 0 {
        (*gl_info).supported[ARB_TEXTURE_COMPRESSION_RGTC as usize] = (TRUE as i32);
    }
    if (*gl_info).supported[ARB_TEXTURE_COMPRESSION_RGTC as usize] != 0 && (*gl_info).supported[ARB_TEXTURE_RG as usize] == 0 {
        (*gl_info).supported[ARB_TEXTURE_COMPRESSION_RGTC as usize] = (FALSE as i32);
    }
    if (*gl_info).supported[NV_TEXTURE_SHADER2 as usize] != 0 {
        if (*gl_info).supported[NV_REGISTER_COMBINERS as usize] != 0 {
            /* Also disable ATI_FRAGMENT_SHADER if register combiners and texture_shader2
             * are supported. The nv extensions provide the same functionality as the
             * ATI one, and a bit more(signed pixelformats). */
            (*gl_info).supported[ATI_FRAGMENT_SHADER as usize] = (FALSE as i32);
        }
    }
    if (*gl_info).supported[ATI_FRAGMENT_SHADER as usize] != 0 {
        /* Disable NV_register_combiners and fragment shader if this is supported. */
        (*gl_info).supported[NV_REGISTER_COMBINERS as usize] = (FALSE as i32);
        (*gl_info).supported[NV_REGISTER_COMBINERS2 as usize] = (FALSE as i32);
        (*gl_info).supported[NV_TEXTURE_SHADER as usize] = (FALSE as i32);
        (*gl_info).supported[NV_TEXTURE_SHADER2 as usize] = (FALSE as i32);
    }
    if (*gl_info).supported[NV_HALF_FLOAT as usize] != 0 {
        /* GL_ARB_half_float_vertex is a subset of GL_NV_half_float. */
        (*gl_info).supported[ARB_HALF_FLOAT_VERTEX as usize] = (TRUE as i32);
    }
    if (*gl_info).supported[ARB_FRAMEBUFFER_SRGB as usize] != 0 && (*gl_info).supported[EXT_TEXTURE_SRGB_DECODE as usize] == 0 {
        /* Current wined3d sRGB infrastructure requires EXT_texture_sRGB_decode
         * for GL_ARB_framebuffer_sRGB support (without EXT_texture_sRGB_decode
         * we never render to sRGB surfaces). */
        (*gl_info).supported[ARB_FRAMEBUFFER_SRGB as usize] = (FALSE as i32);
    }
    if (*gl_info).supported[ARB_OCCLUSION_QUERY as usize] != 0 {
        let mut counter_bits: GLint = 0;

        ((*gl_info).gl_ops.ext.p_glGetQueryiv.unwrap_unchecked())(GL_SAMPLES_PASSED as GLenum, GL_QUERY_COUNTER_BITS as GLenum, &raw mut counter_bits);
        if counter_bits == 0 {
            (*gl_info).supported[ARB_OCCLUSION_QUERY as usize] = (FALSE as i32);
        }
    }
    if (*gl_info).supported[ARB_TIMER_QUERY as usize] != 0 {
        let mut counter_bits: GLint = 0;

        ((*gl_info).gl_ops.ext.p_glGetQueryiv.unwrap_unchecked())(GL_TIMESTAMP as GLenum, GL_QUERY_COUNTER_BITS as GLenum, &raw mut counter_bits);
        if counter_bits == 0 {
            (*gl_info).supported[ARB_TIMER_QUERY as usize] = (FALSE as i32);
        }
    }
    if gl_version >= MAKEDWORD_VERSION!(3, 0) {
        let mut counter_bits: GLint = 0;

        (*gl_info).supported[WINED3D_GL_PRIMITIVE_QUERY as usize] = (TRUE as i32);

        ((*gl_info).gl_ops.ext.p_glGetQueryiv.unwrap_unchecked())(GL_PRIMITIVES_GENERATED as GLenum, GL_QUERY_COUNTER_BITS as GLenum, &raw mut counter_bits);
        if counter_bits == 0 {
            (*gl_info).supported[WINED3D_GL_PRIMITIVE_QUERY as usize] = (FALSE as i32);
        }

        ((*gl_info).gl_ops.ext.p_glGetQueryiv.unwrap_unchecked())(GL_TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN as GLenum, GL_QUERY_COUNTER_BITS as GLenum, &raw mut counter_bits);
        if counter_bits == 0 {
            (*gl_info).supported[WINED3D_GL_PRIMITIVE_QUERY as usize] = (FALSE as i32);
        }
    }
    if (*gl_info).supported[ARB_VIEWPORT_ARRAY as usize] != 0 {
        let mut subpixel_bits: GLint = 0;

        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(GL_VIEWPORT_SUBPIXEL_BITS as GLenum, &raw mut subpixel_bits);
        TRACE!("Viewport supports %d subpixel bits.\n", subpixel_bits);
        if subpixel_bits < 8 && (*gl_info).supported[ARB_CLIP_CONTROL as usize] != 0 {
            TRACE!("Disabling ARB_clip_control because viewport subpixel bits < 8.\n");
            (*gl_info).supported[ARB_CLIP_CONTROL as usize] = (FALSE as i32);
        }
        (*gl_info).limits.viewport_subpixel_bits = subpixel_bits as ::core::ffi::c_uint;
    }
    if (*gl_info).supported[ARB_CLIP_CONTROL as usize] != 0 && (*gl_info).supported[ARB_VIEWPORT_ARRAY as usize] == 0 {
        /* When using ARB_clip_control we need the float viewport parameters
         * introduced by ARB_viewport_array to take care of the shifted pixel
         * coordinates. */
        (*gl_info).supported[ARB_CLIP_CONTROL as usize] = (FALSE as i32);
    }
    if (*gl_info).supported[ARB_STENCIL_TEXTURING as usize] != 0 && (*gl_info).supported[ARB_TEXTURE_SWIZZLE as usize] == 0 {
        /* The stencil value needs to be placed in the green channel.  */
        (*gl_info).supported[ARB_STENCIL_TEXTURING as usize] = (FALSE as i32);
    }
    if (*gl_info).supported[ATI_TEXTURE_MIRROR_ONCE as usize] == 0 && (*gl_info).supported[EXT_TEXTURE_MIRROR_CLAMP as usize] != 0 {
        (*gl_info).supported[ATI_TEXTURE_MIRROR_ONCE as usize] = (TRUE as i32);
    }
    if (*gl_info).supported[ARB_TEXTURE_MIRROR_CLAMP_TO_EDGE as usize] == 0 && (*gl_info).supported[ATI_TEXTURE_MIRROR_ONCE as usize] != 0 {
        (*gl_info).supported[ARB_TEXTURE_MIRROR_CLAMP_TO_EDGE as usize] = (TRUE as i32);
    }
    if (*gl_info).supported[ARB_TEXTURE_STORAGE as usize] != 0 && (*gl_info).supported[APPLE_YCBCR_422 as usize] != 0 {
        /* AFAIK APPLE_ycbcr_422 is only available in legacy contexts so we shouldn't ever hit this. */
        ERR!("Disabling APPLE_ycbcr_422 because of ARB_texture_storage.\n");
        (*gl_info).supported[APPLE_YCBCR_422 as usize] = (FALSE as i32);
    }
    if (*gl_info).supported[ARB_DRAW_INDIRECT as usize] != 0 && (*gl_info).supported[ARB_BASE_INSTANCE as usize] == 0 {
        /* If ARB_base_instance is not supported the baseInstance field
         * in indirect draw parameters must be 0 or behavior is undefined.
         */
        WARN!("Disabling ARB_draw_indirect because ARB_base_instance is not supported.\n");
        (*gl_info).supported[ARB_DRAW_INDIRECT as usize] = (FALSE as i32);
    }
    if (*gl_info).supported[ARB_TEXTURE_MULTISAMPLE as usize] != 0 && wined3d_settings.multisample_textures == 0 {
        (*gl_info).supported[ARB_TEXTURE_MULTISAMPLE as usize] = (FALSE as i32);
    }
    if (*gl_info).supported[ARB_TEXTURE_MULTISAMPLE as usize] != 0 && (*gl_info).supported[ARB_TEXTURE_STORAGE_MULTISAMPLE as usize] == 0 {
        WARN!("Disabling ARB_texture_multisample because immutable storage is not supported.\n");
        (*gl_info).supported[ARB_TEXTURE_MULTISAMPLE as usize] = (FALSE as i32);
    }
    if (*gl_info).supported[ARB_FRAMEBUFFER_OBJECT as usize] != 0 && (*gl_info).supported[EXT_PACKED_DEPTH_STENCIL as usize] == 0 {
        /* Disable the extension if the immutable storage is missing. */
        (*gl_info).supported[EXT_PACKED_DEPTH_STENCIL as usize] = (TRUE as i32);
    }

    wined3d_adapter_init_limits(gl_info);

    if (*gl_info).supported[ARB_VERTEX_PROGRAM as usize] != 0 && test_arb_vs_offset_limit(gl_info) != 0 {
        (*gl_info).quirks |= WINED3D_QUIRK_ARB_VS_OFFSET_LIMIT;
    }

    if (*gl_info).supported[ARB_SHADING_LANGUAGE_100 as usize] != 0 {
        let str_: *const ::core::ffi::c_char = (*gl_info).gl_ops.gl.p_glGetString.unwrap_unchecked()(GL_SHADING_LANGUAGE_VERSION_ARB as GLenum) as *const ::core::ffi::c_char;
        let mut major: ::core::ffi::c_uint = 0;
        let mut minor: ::core::ffi::c_uint = 0;
        TRACE!("GLSL version string: %s.\n", debugstr_a(str_));

        /* The format of the GLSL version string is "major.minor[.release] [vendor info]". */
        sscanf(str_, b"%u.%u\0".as_ptr() as *const ::core::ffi::c_char, &raw mut major, &raw mut minor);
        (*gl_info).glsl_version = MAKEDWORD_VERSION!(major, minor);
        if (*gl_info).glsl_version > MAKEDWORD_VERSION!(1, 30)
            && gl_version < MAKEDWORD_VERSION!(3, 0)
        {
            WARN!("OpenGL version %u.%u too low, limiting GLSL version to 1.30.\n",
                    gl_version >> 16, gl_version & 0xffff);
            (*gl_info).glsl_version = MAKEDWORD_VERSION!(1, 30);
        }
        if (*gl_info).glsl_version >= MAKEDWORD_VERSION!(1, 30) {
            (*gl_info).supported[WINED3D_GLSL_130 as usize] = (TRUE as i32);
        }
    }

    checkGLcall!(gl_info, "extension detection");

    (*adapter).shader_backend = &raw const glsl_shader_backend;
    (*adapter).vertex_pipe = &raw const glsl_vertex_pipe;
    (*adapter).fragment_pipe = &raw const glsl_fragment_pipe;

    if (*gl_info).supported[ARB_FRAMEBUFFER_OBJECT as usize] != 0 {
        (*gl_info).fbo_ops.glIsRenderbuffer = (*gl_info).gl_ops.ext.p_glIsRenderbuffer;
        (*gl_info).fbo_ops.glBindRenderbuffer = (*gl_info).gl_ops.ext.p_glBindRenderbuffer;
        (*gl_info).fbo_ops.glDeleteRenderbuffers = (*gl_info).gl_ops.ext.p_glDeleteRenderbuffers;
        (*gl_info).fbo_ops.glGenRenderbuffers = (*gl_info).gl_ops.ext.p_glGenRenderbuffers;
        (*gl_info).fbo_ops.glRenderbufferStorage = (*gl_info).gl_ops.ext.p_glRenderbufferStorage;
        (*gl_info).fbo_ops.glRenderbufferStorageMultisample = (*gl_info).gl_ops.ext.p_glRenderbufferStorageMultisample;
        (*gl_info).fbo_ops.glGetRenderbufferParameteriv = (*gl_info).gl_ops.ext.p_glGetRenderbufferParameteriv;
        (*gl_info).fbo_ops.glIsFramebuffer = (*gl_info).gl_ops.ext.p_glIsFramebuffer;
        (*gl_info).fbo_ops.glBindFramebuffer = (*gl_info).gl_ops.ext.p_glBindFramebuffer;
        (*gl_info).fbo_ops.glDeleteFramebuffers = (*gl_info).gl_ops.ext.p_glDeleteFramebuffers;
        (*gl_info).fbo_ops.glGenFramebuffers = (*gl_info).gl_ops.ext.p_glGenFramebuffers;
        (*gl_info).fbo_ops.glCheckFramebufferStatus = (*gl_info).gl_ops.ext.p_glCheckFramebufferStatus;
        (*gl_info).fbo_ops.glFramebufferTexture1D = (*gl_info).gl_ops.ext.p_glFramebufferTexture1D;
        (*gl_info).fbo_ops.glFramebufferTexture2D = (*gl_info).gl_ops.ext.p_glFramebufferTexture2D;
        (*gl_info).fbo_ops.glFramebufferTexture3D = (*gl_info).gl_ops.ext.p_glFramebufferTexture3D;
        (*gl_info).fbo_ops.glFramebufferTextureLayer = (*gl_info).gl_ops.ext.p_glFramebufferTextureLayer;
        (*gl_info).fbo_ops.glFramebufferRenderbuffer = (*gl_info).gl_ops.ext.p_glFramebufferRenderbuffer;
        (*gl_info).fbo_ops.glGetFramebufferAttachmentParameteriv = (*gl_info).gl_ops.ext.p_glGetFramebufferAttachmentParameteriv;
        (*gl_info).fbo_ops.glBlitFramebuffer = (*gl_info).gl_ops.ext.p_glBlitFramebuffer;
        (*gl_info).fbo_ops.glGenerateMipmap = (*gl_info).gl_ops.ext.p_glGenerateMipmap;
        (*gl_info).fbo_ops.glFramebufferTexture = (*gl_info).gl_ops.ext.p_glFramebufferTexture;

        (*gl_info).supported[EXT_FRAMEBUFFER_OBJECT as usize] = (TRUE as i32);
    } else {
        if (*gl_info).supported[EXT_FRAMEBUFFER_OBJECT as usize] != 0 {
            (*gl_info).fbo_ops.glIsRenderbuffer = (*gl_info).gl_ops.ext.p_glIsRenderbufferEXT;
            (*gl_info).fbo_ops.glBindRenderbuffer = (*gl_info).gl_ops.ext.p_glBindRenderbufferEXT;
            (*gl_info).fbo_ops.glDeleteRenderbuffers = (*gl_info).gl_ops.ext.p_glDeleteRenderbuffersEXT;
            (*gl_info).fbo_ops.glGenRenderbuffers = (*gl_info).gl_ops.ext.p_glGenRenderbuffersEXT;
            (*gl_info).fbo_ops.glRenderbufferStorage = (*gl_info).gl_ops.ext.p_glRenderbufferStorageEXT;
            (*gl_info).fbo_ops.glGetRenderbufferParameteriv = (*gl_info).gl_ops.ext.p_glGetRenderbufferParameterivEXT;
            (*gl_info).fbo_ops.glIsFramebuffer = (*gl_info).gl_ops.ext.p_glIsFramebufferEXT;
            (*gl_info).fbo_ops.glBindFramebuffer = (*gl_info).gl_ops.ext.p_glBindFramebufferEXT;
            (*gl_info).fbo_ops.glDeleteFramebuffers = (*gl_info).gl_ops.ext.p_glDeleteFramebuffersEXT;
            (*gl_info).fbo_ops.glGenFramebuffers = (*gl_info).gl_ops.ext.p_glGenFramebuffersEXT;
            (*gl_info).fbo_ops.glCheckFramebufferStatus = (*gl_info).gl_ops.ext.p_glCheckFramebufferStatusEXT;
            (*gl_info).fbo_ops.glFramebufferTexture1D = (*gl_info).gl_ops.ext.p_glFramebufferTexture1DEXT;
            (*gl_info).fbo_ops.glFramebufferTexture2D = (*gl_info).gl_ops.ext.p_glFramebufferTexture2DEXT;
            (*gl_info).fbo_ops.glFramebufferTexture3D = (*gl_info).gl_ops.ext.p_glFramebufferTexture3DEXT;
            (*gl_info).fbo_ops.glFramebufferRenderbuffer = (*gl_info).gl_ops.ext.p_glFramebufferRenderbufferEXT;
            (*gl_info).fbo_ops.glGetFramebufferAttachmentParameteriv = (*gl_info).gl_ops.ext.p_glGetFramebufferAttachmentParameterivEXT;
            (*gl_info).fbo_ops.glGenerateMipmap = (*gl_info).gl_ops.ext.p_glGenerateMipmapEXT;
        }

        if (*gl_info).supported[ARB_GEOMETRY_SHADER4 as usize] != 0 {
            (*gl_info).fbo_ops.glFramebufferTexture = (*gl_info).gl_ops.ext.p_glFramebufferTextureARB;
            (*gl_info).fbo_ops.glFramebufferTextureLayer = (*gl_info).gl_ops.ext.p_glFramebufferTextureLayerARB;
        }
        if (*gl_info).supported[EXT_FRAMEBUFFER_BLIT as usize] != 0 {
            (*gl_info).fbo_ops.glBlitFramebuffer = (*gl_info).gl_ops.ext.p_glBlitFramebufferEXT;
        }
        if (*gl_info).supported[EXT_FRAMEBUFFER_MULTISAMPLE as usize] != 0 {
            (*gl_info).fbo_ops.glRenderbufferStorageMultisample = (*gl_info).gl_ops.ext.p_glRenderbufferStorageMultisampleEXT;
        }
    }

    (*gl_info).wrap_lookup[(WINED3D_TADDRESS_WRAP as ::core::ffi::c_int - WINED3D_TADDRESS_WRAP as ::core::ffi::c_int) as usize] = (GL_REPEAT as i32);
    (*gl_info).wrap_lookup[(WINED3D_TADDRESS_MIRROR as ::core::ffi::c_int - WINED3D_TADDRESS_WRAP as ::core::ffi::c_int) as usize] =
        (if (*gl_info).supported[ARB_TEXTURE_MIRRORED_REPEAT as usize] != 0 { GL_MIRRORED_REPEAT_ARB } else { GL_REPEAT }) as GLint;
    (*gl_info).wrap_lookup[(WINED3D_TADDRESS_CLAMP as ::core::ffi::c_int - WINED3D_TADDRESS_WRAP as ::core::ffi::c_int) as usize] = (GL_CLAMP_TO_EDGE as i32);
    (*gl_info).wrap_lookup[(WINED3D_TADDRESS_BORDER as ::core::ffi::c_int - WINED3D_TADDRESS_WRAP as ::core::ffi::c_int) as usize] =
        (if (*gl_info).supported[ARB_TEXTURE_BORDER_CLAMP as usize] != 0 { GL_CLAMP_TO_BORDER_ARB } else { GL_REPEAT }) as GLint;
    (*gl_info).wrap_lookup[(WINED3D_TADDRESS_MIRROR_ONCE as ::core::ffi::c_int - WINED3D_TADDRESS_WRAP as ::core::ffi::c_int) as usize] =
        (if (*gl_info).supported[ARB_TEXTURE_MIRROR_CLAMP_TO_EDGE as usize] != 0 { GL_MIRROR_CLAMP_TO_EDGE } else { GL_REPEAT }) as GLint;

    if (*gl_info).supported[WINED3D_GL_LEGACY_CONTEXT as usize] == 0 {
        let mut vao: GLuint = 0;

        ((*gl_info).gl_ops.ext.p_glGenVertexArrays.unwrap_unchecked())(1 as GLsizei, &raw mut vao);
        ((*gl_info).gl_ops.ext.p_glBindVertexArray.unwrap_unchecked())(vao);
        checkGLcall!(gl_info, "creating VAO");
    }

    gl_vendor = wined3d_guess_gl_vendor(gl_info, gl_vendor_str, gl_renderer_str, gl_version_str);
    TRACE!("Guessed GL vendor %#x.\n", gl_vendor as ::core::ffi::c_uint);

    (*caps_gl_ctx).gpu_description = query_gpu_description(gl_info, &raw mut (*caps_gl_ctx).vram_bytes);
    if (*caps_gl_ctx).gpu_description.is_null() {
        let mut feature_level: wined3d_feature_level = WINED3D_FEATURE_LEVEL_NONE;
        let mut fragment_caps: fragment_caps = fragment_caps {
            PrimitiveMiscCaps: 0,
            TextureOpCaps: 0,
            max_blend_stages: 0,
            max_textures: 0,
        };
        let mut vendor: wined3d_pci_vendor = HW_VENDOR_SOFTWARE;
        let mut device: wined3d_pci_device = CARD_WINE;
        let mut shader_caps: shader_caps = shader_caps {
            vs_version: 0,
            hs_version: 0,
            ds_version: 0,
            gs_version: 0,
            ps_version: 0,
            cs_version: 0,
            vs_uniform_count: 0,
            ps_uniform_count: 0,
            ps_1x_max_value: 0.,
            varying_count: 0,
            wined3d_caps: 0,
        };

        (*(*adapter).shader_backend).shader_get_caps.unwrap_unchecked()(adapter, &raw mut shader_caps);
        (*(*adapter).fragment_pipe).get_caps.unwrap_unchecked()(adapter, &raw mut fragment_caps);
        feature_level = feature_level_from_caps(gl_info, &raw mut shader_caps, &raw mut fragment_caps);

        vendor = wined3d_guess_card_vendor(gl_vendor_str, gl_renderer_str);
        TRACE!("Guessed vendor PCI ID 0x%04x.\n", vendor as ::core::ffi::c_uint);

        device = wined3d_guess_card(feature_level, gl_renderer_str, &raw mut gl_vendor, &raw mut vendor);
        TRACE!("Guessed device PCI ID 0x%04x.\n", device as ::core::ffi::c_uint);

        (*caps_gl_ctx).gpu_description = wined3d_get_gpu_description(vendor, device);
        if (*caps_gl_ctx).gpu_description.is_null() {
            ERR!("Card %04x:%04x not found in driver DB.\n", vendor as ::core::ffi::c_uint, device as ::core::ffi::c_uint);
            return (FALSE) as BOOL;
        }
    }
    fixup_extensions(gl_info, caps_gl_ctx, gl_renderer_str, gl_vendor);

    (*adapter).vram_bytes_used = 0;
    TRACE!("Emulating 0x%s bytes of video ram.\n", wine_dbgstr_longlong((*caps_gl_ctx).vram_bytes as ULONGLONG));

    if (*gl_info).supported[EXT_MEMORY_OBJECT as usize] != 0 {
        let mut device_count: GLint = 0;

        (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(GL_NUM_DEVICE_UUIDS_EXT as GLenum, &raw mut device_count);
        if device_count > 0 {
            if device_count > 1 {
                FIXME!("A set of %d devices is not supported.\n", device_count);
            }

            ((*gl_info).gl_ops.ext.p_glGetUnsignedBytevEXT.unwrap_unchecked())(GL_DRIVER_UUID_EXT as GLenum, &raw mut (*adapter).driver_uuid as *mut GLubyte);
            ((*gl_info).gl_ops.ext.p_glGetUnsignedBytei_vEXT.unwrap_unchecked())(GL_DEVICE_UUID_EXT as GLenum, 0 as GLuint, &raw mut (*adapter).device_uuid as *mut GLubyte);

            TRACE!("Driver UUID: %s, device UUID %s.\n", debugstr_guid(&raw mut (*adapter).driver_uuid), debugstr_guid(&raw mut (*adapter).device_uuid));
        } else {
            WARN!("Unexpected device count %d.\n", device_count);
        }
    }

    gl_ext_emul_mask = ((*(*adapter).vertex_pipe).vp_get_emul_mask.unwrap_unchecked()(adapter)
        | (*(*adapter).fragment_pipe).get_emul_mask.unwrap_unchecked()(adapter)) as DWORD;
    if gl_ext_emul_mask & GL_EXT_EMUL_ARB_MULTITEXTURE as DWORD != 0 {
        install_gl_compat_wrapper(gl_info, ARB_MULTITEXTURE);
    }
    if gl_ext_emul_mask & GL_EXT_EMUL_EXT_FOG_COORD as DWORD != 0 {
        install_gl_compat_wrapper(gl_info, EXT_FOG_COORD);
    }

    return (TRUE) as BOOL;
}

unsafe extern "system" fn invalid_generic_attrib_func(idx: GLuint, data: *const core::ffi::c_void) {
    ERR!("Invalid attribute function called.\n");
    DebugBreak();
}

unsafe extern "system" fn generic_d3dcolor(idx: GLuint, data: *const core::ffi::c_void) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl_get_current()).gl_info;
    let color: DWORD = *(data as *const DWORD);

    ((*gl_info).gl_ops.ext.p_glVertexAttrib4Nub.unwrap_unchecked())(
        idx,
        (D3DCOLOR_B_R!(color)) as GLubyte,
        (D3DCOLOR_B_G!(color)) as GLubyte,
        (D3DCOLOR_B_B!(color)) as GLubyte,
        (D3DCOLOR_B_A!(color)) as GLubyte,
    );
}

unsafe extern "system" fn generic_short2n(idx: GLuint, data: *const ::core::ffi::c_void) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl_get_current()).gl_info;
    let s: [GLshort; 4] = [
        *(data as *const GLshort).offset(0 as isize),
        *(data as *const GLshort).offset(1 as isize),
        0 as GLshort,
        1 as GLshort,
    ];

    (*gl_info)
        .gl_ops
        .ext
        .p_glVertexAttrib4Nsv
        .unwrap_unchecked()(idx, s.as_ptr());
}

unsafe extern "system" fn generic_ushort2n(idx: GLuint, data: *const core::ffi::c_void) {
    let s: [GLushort; 4] = [
        *(data as *const GLushort).offset(0 as isize),
        *(data as *const GLushort).offset(1 as isize),
        0 as GLushort,
        1 as GLushort,
    ];
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl_get_current()).gl_info;

    (*gl_info)
        .gl_ops
        .ext
        .p_glVertexAttrib4Nusv
        .unwrap_unchecked()(idx, core::ptr::addr_of!(s) as *const GLushort);
}

unsafe extern "system" fn generic_float16_2(idx: GLuint, data: *const core::ffi::c_void) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl_get_current()).gl_info;
    let x: f32 = float_16_to_32((data as *const u16).add(0));
    let y: f32 = float_16_to_32((data as *const u16).add(1));

    (*gl_info)
        .gl_ops
        .ext
        .p_glVertexAttrib2f
        .unwrap_unchecked()(idx, x as GLfloat, y as GLfloat);
}

unsafe extern "system" fn generic_float16_4(idx: GLuint, data: *const core::ffi::c_void) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl_get_current()).gl_info;
    let x: f32 = float_16_to_32((data as *const u16).offset(0 as isize));
    let y: f32 = float_16_to_32((data as *const u16).offset(1 as isize));
    let z: f32 = float_16_to_32((data as *const u16).offset(2 as isize));
    let w: f32 = float_16_to_32((data as *const u16).offset(3 as isize));

    (*gl_info)
        .gl_ops
        .ext
        .p_glVertexAttrib4f
        .unwrap_unchecked()(
        idx,
        x as GLfloat,
        y as GLfloat,
        z as GLfloat,
        w as GLfloat,
    );
}

unsafe extern "C" fn wined3d_adapter_init_ffp_attrib_ops(mut adapter_gl: *mut wined3d_adapter_gl) {
    let mut gl_info: *mut wined3d_gl_info = &raw mut (*adapter_gl).gl_info;
    let mut ops: *mut wined3d_ffp_attrib_ops = &raw mut (*gl_info).ffp_attrib_ops;
    let mut i: ::core::ffi::c_uint = 0;
    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_FFP_EMIT_COUNT as ::core::ffi::c_uint {
        (*ops).generic[i as usize] = Some(invalid_generic_attrib_func);
        i = i.wrapping_add(1);
    }
    (*ops).generic[WINED3D_FFP_EMIT_FLOAT1 as usize] =
        ::core::mem::transmute::<
            Option<unsafe extern "system" fn(GLuint, *const GLfloat) -> ()>,
            wined3d_generic_attrib_func,
        >((*gl_info).gl_ops.ext.p_glVertexAttrib1fv);
    (*ops).generic[WINED3D_FFP_EMIT_FLOAT2 as usize] =
        ::core::mem::transmute::<
            Option<unsafe extern "system" fn(GLuint, *const GLfloat) -> ()>,
            wined3d_generic_attrib_func,
        >((*gl_info).gl_ops.ext.p_glVertexAttrib2fv);
    (*ops).generic[WINED3D_FFP_EMIT_FLOAT3 as usize] =
        ::core::mem::transmute::<
            Option<unsafe extern "system" fn(GLuint, *const GLfloat) -> ()>,
            wined3d_generic_attrib_func,
        >((*gl_info).gl_ops.ext.p_glVertexAttrib3fv);
    (*ops).generic[WINED3D_FFP_EMIT_FLOAT4 as usize] =
        ::core::mem::transmute::<
            Option<unsafe extern "system" fn(GLuint, *const GLfloat) -> ()>,
            wined3d_generic_attrib_func,
        >((*gl_info).gl_ops.ext.p_glVertexAttrib4fv);
    if (*gl_info).supported[ARB_VERTEX_ARRAY_BGRA as usize] != 0 {
        (*ops).generic[WINED3D_FFP_EMIT_D3DCOLOR as usize] = Some(generic_d3dcolor);
    } else {
        (*ops).generic[WINED3D_FFP_EMIT_D3DCOLOR as usize] =
            ::core::mem::transmute::<
                Option<unsafe extern "system" fn(GLuint, *const GLubyte) -> ()>,
                wined3d_generic_attrib_func,
            >((*gl_info).gl_ops.ext.p_glVertexAttrib4Nubv);
    }
    (*ops).generic[WINED3D_FFP_EMIT_UBYTE4 as usize] =
        ::core::mem::transmute::<
            Option<unsafe extern "system" fn(GLuint, *const GLubyte) -> ()>,
            wined3d_generic_attrib_func,
        >((*gl_info).gl_ops.ext.p_glVertexAttrib4ubv);
    (*ops).generic[WINED3D_FFP_EMIT_SHORT2 as usize] =
        ::core::mem::transmute::<
            Option<unsafe extern "system" fn(GLuint, *const GLshort) -> ()>,
            wined3d_generic_attrib_func,
        >((*gl_info).gl_ops.ext.p_glVertexAttrib2sv);
    (*ops).generic[WINED3D_FFP_EMIT_SHORT4 as usize] =
        ::core::mem::transmute::<
            Option<unsafe extern "system" fn(GLuint, *const GLshort) -> ()>,
            wined3d_generic_attrib_func,
        >((*gl_info).gl_ops.ext.p_glVertexAttrib4sv);
    (*ops).generic[WINED3D_FFP_EMIT_UBYTE4N as usize] =
        ::core::mem::transmute::<
            Option<unsafe extern "system" fn(GLuint, *const GLubyte) -> ()>,
            wined3d_generic_attrib_func,
        >((*gl_info).gl_ops.ext.p_glVertexAttrib4Nubv);
    (*ops).generic[WINED3D_FFP_EMIT_SHORT2N as usize] =
        Some(generic_short2n);
    (*ops).generic[WINED3D_FFP_EMIT_SHORT4N as usize] =
        ::core::mem::transmute::<
            Option<unsafe extern "system" fn(GLuint, *const GLshort) -> ()>,
            wined3d_generic_attrib_func,
        >((*gl_info).gl_ops.ext.p_glVertexAttrib4Nsv);
    (*ops).generic[WINED3D_FFP_EMIT_USHORT2N as usize] =
        Some(generic_ushort2n);
    (*ops).generic[WINED3D_FFP_EMIT_USHORT4N as usize] =
        ::core::mem::transmute::<
            Option<unsafe extern "system" fn(GLuint, *const GLushort) -> ()>,
            wined3d_generic_attrib_func,
        >((*gl_info).gl_ops.ext.p_glVertexAttrib4Nusv);
    if (*gl_info).supported[NV_HALF_FLOAT as usize] != 0
        && (*gl_info).supported[NV_VERTEX_PROGRAM as usize] != 0
    {
        (*ops).generic[WINED3D_FFP_EMIT_FLOAT16_2 as usize] =
            ::core::mem::transmute::<
                Option<unsafe extern "system" fn(GLuint, *const GLhalfNV) -> ()>,
                wined3d_generic_attrib_func,
            >((*gl_info).gl_ops.ext.p_glVertexAttrib2hvNV);
        (*ops).generic[WINED3D_FFP_EMIT_FLOAT16_4 as usize] =
            ::core::mem::transmute::<
                Option<unsafe extern "system" fn(GLuint, *const GLhalfNV) -> ()>,
                wined3d_generic_attrib_func,
            >((*gl_info).gl_ops.ext.p_glVertexAttrib4hvNV);
    } else {
        (*ops).generic[WINED3D_FFP_EMIT_FLOAT16_2 as usize] = Some(generic_float16_2);
        (*ops).generic[WINED3D_FFP_EMIT_FLOAT16_4 as usize] = Some(generic_float16_4);
    };
}

unsafe extern "C" fn wined3d_adapter_init_fb_cfgs(
    mut adapter_gl: *mut wined3d_adapter_gl,
    mut dc: HDC,
) {
    let mut gl_info: *const wined3d_gl_info = &raw mut (*adapter_gl).gl_info;
    let mut i: ::core::ffi::c_int = 0;
    if (*gl_info).supported[WGL_ARB_PIXEL_FORMAT as usize] != 0 {
        let mut attrib_count: UINT = 0 as UINT;
        let mut cfg_count: GLint = 0;
        let mut attribs: [::core::ffi::c_int; 12] = [0; 12];
        let mut values: [::core::ffi::c_int; 12] = [0; 12];
        let mut attribute: ::core::ffi::c_int = 0;
        attribute = (WGL_NUMBER_PIXEL_FORMATS_ARB as i32);
        ((*gl_info).gl_ops.ext.p_wglGetPixelFormatAttribivARB.unwrap_unchecked())(
            dc,
            0,
            0,
            1,
            &raw mut attribute,
            &raw mut cfg_count,
        );
        (*adapter_gl).pixel_formats = calloc(
            (cfg_count as size_t) as _,
            (::core::mem::size_of::<wined3d_pixel_format>() as size_t) as _,
        ) as *mut wined3d_pixel_format;
        attribs[attrib_count as usize] = (WGL_RED_BITS_ARB as i32);
        attrib_count = attrib_count.wrapping_add(1);
        attribs[attrib_count as usize] = (WGL_GREEN_BITS_ARB as i32);
        attrib_count = attrib_count.wrapping_add(1);
        attribs[attrib_count as usize] = (WGL_BLUE_BITS_ARB as i32);
        attrib_count = attrib_count.wrapping_add(1);
        attribs[attrib_count as usize] = (WGL_ALPHA_BITS_ARB as i32);
        attrib_count = attrib_count.wrapping_add(1);
        attribs[attrib_count as usize] = (WGL_COLOR_BITS_ARB as i32);
        attrib_count = attrib_count.wrapping_add(1);
        attribs[attrib_count as usize] = (WGL_DEPTH_BITS_ARB as i32);
        attrib_count = attrib_count.wrapping_add(1);
        attribs[attrib_count as usize] = (WGL_STENCIL_BITS_ARB as i32);
        attrib_count = attrib_count.wrapping_add(1);
        attribs[attrib_count as usize] = (WGL_DRAW_TO_WINDOW_ARB as i32);
        attrib_count = attrib_count.wrapping_add(1);
        attribs[attrib_count as usize] = (WGL_PIXEL_TYPE_ARB as i32);
        attrib_count = attrib_count.wrapping_add(1);
        attribs[attrib_count as usize] = (WGL_DOUBLE_BUFFER_ARB as i32);
        attrib_count = attrib_count.wrapping_add(1);
        attribs[attrib_count as usize] = (WGL_AUX_BUFFERS_ARB as i32);
        attrib_count = attrib_count.wrapping_add(1);
        attribs[attrib_count as usize] = (WGL_SWAP_METHOD_ARB as i32);
        attrib_count = attrib_count.wrapping_add(1);

        i = 0 as ::core::ffi::c_int;
        (*adapter_gl).pixel_format_count = 0 as ::core::ffi::c_uint;
        while i < cfg_count {
            let mut cfg: *mut wined3d_pixel_format = (*adapter_gl)
                .pixel_formats
                .offset((*adapter_gl).pixel_format_count as isize)
                as *mut wined3d_pixel_format;
            let mut format_id: ::core::ffi::c_int = i + 1 as ::core::ffi::c_int;
            if !(((*gl_info).gl_ops.ext.p_wglGetPixelFormatAttribivARB.unwrap_unchecked())(
                dc,
                format_id,
                0,
                attrib_count,
                &raw mut attribs as *mut ::core::ffi::c_int,
                &raw mut values as *mut ::core::ffi::c_int,
            ) == 0)
            {
                (*cfg).iPixelFormat = format_id;
                (*cfg).redSize = values[0 as usize];
                (*cfg).greenSize = values[1 as usize];
                (*cfg).blueSize = values[2 as usize];
                (*cfg).alphaSize = values[3 as usize];
                (*cfg).colorSize = values[4 as usize];
                (*cfg).depthSize = values[5 as usize];
                (*cfg).stencilSize = values[6 as usize];
                (*cfg).windowDrawable = values[7 as usize] as BOOL;
                (*cfg).iPixelType = values[8 as usize];
                (*cfg).doubleBuffer = values[9 as usize] as BOOL;
                (*cfg).auxBuffers = values[10 as usize];
                (*cfg).swap_method = values[11 as usize];

                (*cfg).numSamples = 0;
                /* Check multisample support. */
                if (*gl_info).supported[ARB_MULTISAMPLE as usize] != 0 {
                    let mut attribs_0: [::core::ffi::c_int; 2] =
                        [(WGL_SAMPLE_BUFFERS_ARB as i32), (WGL_SAMPLES_ARB as i32)];
                    let mut values_0: [::core::ffi::c_int; 2] =
                        [0; 2];
                    if ((*gl_info).gl_ops.ext.p_wglGetPixelFormatAttribivARB.unwrap_unchecked())(
                        dc,
                        format_id,
                        0,
                        2,
                        &raw mut attribs_0 as *mut ::core::ffi::c_int,
                        &raw mut values_0 as *mut ::core::ffi::c_int,
                    ) != 0
                    {
                        /* values[0] = WGL_SAMPLE_BUFFERS_ARB which tells whether
                         * multisampling is supported. values[1] = number of
                         * multisample buffers. */
                        if values_0[0 as usize] != 0 {
                            (*cfg).numSamples = values_0[1 as usize];
                        }
                    }
                }
                WINE_TRACE!("iPixelFormat=%d, iPixelType=%#x, doubleBuffer=%d, RGBA=%d/%d/%d/%d, depth=%d, stencil=%d, samples=%d, windowDrawable=%d\n", (*cfg).iPixelFormat, (*cfg).iPixelType, (*cfg).doubleBuffer, (*cfg).redSize, (*cfg).greenSize, (*cfg).blueSize, (*cfg).alphaSize, (*cfg).depthSize, (*cfg).stencilSize, (*cfg).numSamples, (*cfg).windowDrawable);
                (*adapter_gl).pixel_format_count = (*adapter_gl).pixel_format_count.wrapping_add(1);
            }
            i += 1;
        }
    } else {
        let mut cfg_count_0: ::core::ffi::c_int = 0;
        cfg_count_0 = DescribePixelFormat(
            dc,
            0,
            0,
            core::ptr::null_mut(),
        );
        (*adapter_gl).pixel_formats = calloc(
            (cfg_count_0 as size_t) as _,
            (::core::mem::size_of::<wined3d_pixel_format>() as size_t) as _,
        ) as *mut wined3d_pixel_format;
        i = 0 as ::core::ffi::c_int;
        (*adapter_gl).pixel_format_count = 0 as ::core::ffi::c_uint;
        while i < cfg_count_0 {
            let mut cfg_0: *mut wined3d_pixel_format = (*adapter_gl)
                .pixel_formats
                .offset((*adapter_gl).pixel_format_count as isize)
                as *mut wined3d_pixel_format;
            let mut pfd: PIXELFORMATDESCRIPTOR = tagPIXELFORMATDESCRIPTOR {
                nSize: 0,
                nVersion: 0,
                dwFlags: 0,
                iPixelType: 0,
                cColorBits: 0,
                cRedBits: 0,
                cRedShift: 0,
                cGreenBits: 0,
                cGreenShift: 0,
                cBlueBits: 0,
                cBlueShift: 0,
                cAlphaBits: 0,
                cAlphaShift: 0,
                cAccumBits: 0,
                cAccumRedBits: 0,
                cAccumGreenBits: 0,
                cAccumBlueBits: 0,
                cAccumAlphaBits: 0,
                cDepthBits: 0,
                cStencilBits: 0,
                cAuxBuffers: 0,
                iLayerType: 0,
                bReserved: 0,
                dwLayerMask: 0,
                dwVisibleMask: 0,
                dwDamageMask: 0,
            };
            let mut format_id_0: ::core::ffi::c_int =
                i + 1 as ::core::ffi::c_int;
            if !(DescribePixelFormat(
                dc,
                format_id_0,
                ::core::mem::size_of::<PIXELFORMATDESCRIPTOR>() as UINT,
                &raw mut pfd,
            ) == 0)
            {
                /* We only want HW acceleration using an OpenGL ICD driver.
                 * PFD_GENERIC_FORMAT = slow OpenGL 1.1 GDI software rendering.
                 * PFD_GENERIC_ACCELERATED = partial hw acceleration using a MCD
                 * driver (e.g. 3dfx minigl). */
                if pfd.dwFlags & (PFD_GENERIC_FORMAT | PFD_GENERIC_ACCELERATED) as DWORD != 0 {
                    WINE_TRACE!("Skipping format %d because it isn't ICD accelerated.\n", format_id_0);
                } else {
                    (*cfg_0).iPixelFormat = format_id_0;
                    (*cfg_0).redSize = pfd.cRedBits as ::core::ffi::c_int;
                    (*cfg_0).greenSize = pfd.cGreenBits as ::core::ffi::c_int;
                    (*cfg_0).blueSize = pfd.cBlueBits as ::core::ffi::c_int;
                    (*cfg_0).alphaSize = pfd.cAlphaBits as ::core::ffi::c_int;
                    (*cfg_0).colorSize = pfd.cColorBits as ::core::ffi::c_int;
                    (*cfg_0).depthSize = pfd.cDepthBits as ::core::ffi::c_int;
                    (*cfg_0).stencilSize = pfd.cStencilBits as ::core::ffi::c_int;
                    (*cfg_0).windowDrawable = (if pfd.dwFlags & PFD_DRAW_TO_WINDOW as DWORD != 0 {
                        1
                    } else {
                        0
                    }) as BOOL;
                    (*cfg_0).iPixelType = if pfd.iPixelType as ::core::ffi::c_int
                        == (PFD_TYPE_RGBA as i32)
                    {
                        (WGL_TYPE_RGBA_ARB as i32)
                    } else {
                        (WGL_TYPE_COLORINDEX_ARB as i32)
                    };
                    (*cfg_0).doubleBuffer = (if pfd.dwFlags & PFD_DOUBLEBUFFER as DWORD != 0 {
                        1
                    } else {
                        0
                    }) as BOOL;
                    (*cfg_0).auxBuffers = pfd.cAuxBuffers as ::core::ffi::c_int;
                    (*cfg_0).swap_method = (WGL_SWAP_UNDEFINED_ARB as i32);
                    (*cfg_0).numSamples = 0;
                    WINE_TRACE!("iPixelFormat=%d, iPixelType=%#x, doubleBuffer=%d, RGBA=%d/%d/%d/%d, depth=%d, stencil=%d, windowDrawable=%d\n", (*cfg_0).iPixelFormat, (*cfg_0).iPixelType, (*cfg_0).doubleBuffer, (*cfg_0).redSize, (*cfg_0).greenSize, (*cfg_0).blueSize, (*cfg_0).alphaSize, (*cfg_0).depthSize, (*cfg_0).stencilSize, (*cfg_0).windowDrawable);
                    (*adapter_gl).pixel_format_count =
                        (*adapter_gl).pixel_format_count.wrapping_add(1);
                }
            }
            i += 1;
        }
    };
}

unsafe extern "C" fn adapter_gl_destroy(adapter: *mut wined3d_adapter) {
    let adapter_gl: *mut wined3d_adapter_gl = wined3d_adapter_gl(adapter);

    free((*adapter_gl).pixel_formats as *mut ::core::ffi::c_void);
    wined3d_adapter_cleanup(adapter);
    free(adapter_gl as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn adapter_gl_create_device(
    wined3d: *mut wined3d,
    adapter: *const wined3d_adapter,
    device_type: wined3d_device_type,
    focus_window: HWND,
    flags: ::core::ffi::c_uint,
    surface_alignment: BYTE,
    levels: *const wined3d_feature_level,
    level_count: ::core::ffi::c_uint,
    device_parent: *mut wined3d_device_parent,
    device: *mut *mut wined3d_device,
) -> HRESULT {
    let mut device_gl: *mut wined3d_device_gl = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    device_gl = calloc(1, (core::mem::size_of::<wined3d_device_gl>()) as _)
        as *mut wined3d_device_gl;
    if device_gl.is_null() {
        return E_OUTOFMEMORY;
    }

    (*device_gl).current_fence_id = 1;

    hr = wined3d_device_init(
        &raw mut (*device_gl).d,
        wined3d,
        (*adapter).ordinal,
        device_type,
        focus_window,
        flags,
        surface_alignment,
        levels,
        level_count,
        &raw const (*wined3d_adapter_gl_const(adapter)).gl_info.supported as *const BOOL,
        device_parent,
    );
    if hr < 0 {
        WARN!("Failed to initialize device, hr %#lx.\n", hr);
        free(device_gl as *mut core::ffi::c_void);
        return hr;
    }

    wined3d_lock_init!(
        &raw mut (*device_gl).allocator_cs,
        "wined3d_device_gl.allocator_cs",
    );

    *device = &raw mut (*device_gl).d;
    return WINED3D_OK;
}

unsafe extern "C" fn adapter_gl_destroy_device(device: *mut wined3d_device) {
    let device_gl: *mut wined3d_device_gl = wined3d_device_gl(device);

    wined3d_device_cleanup(core::ptr::addr_of_mut!((*device_gl).d));
    wined3d_lock_cleanup(core::ptr::addr_of_mut!((*device_gl).allocator_cs));

    free((*device_gl).retired_blocks as *mut core::ffi::c_void);
    free(device_gl as *mut core::ffi::c_void);
}

unsafe extern "C" fn adapter_gl_acquire_context(
    device: *mut wined3d_device,
    texture: *mut wined3d_texture,
    sub_resource_idx: u32,
) -> *mut wined3d_context {
    wined3d_context_gl_acquire(device, texture, sub_resource_idx)
}

unsafe extern "C" fn adapter_gl_release_context(mut context: *mut wined3d_context) {
    return wined3d_context_gl_release(wined3d_context_gl(context));
}

unsafe extern "C" fn adapter_gl_get_wined3d_caps(adapter: *const wined3d_adapter, caps: *mut wined3d_caps) {
    let gl_info: *const wined3d_gl_info = core::ptr::addr_of!((*wined3d_adapter_gl_const(adapter)).gl_info);
    let d3d_info: *const wined3d_d3d_info = core::ptr::addr_of!((*adapter).d3d_info);

    (*caps).ddraw_caps.dds_caps |= (WINEDDSCAPS_BACKBUFFER
        | WINEDDSCAPS_COMPLEX
        | WINEDDSCAPS_FRONTBUFFER
        | WINEDDSCAPS_3DDEVICE
        | WINEDDSCAPS_VIDEOMEMORY
        | WINEDDSCAPS_OWNDC
        | WINEDDSCAPS_LOCALVIDMEM
        | WINEDDSCAPS_NONLOCALVIDMEM) as DWORD;

    (*caps).ddraw_caps.caps |= WINEDDCAPS_3D as DWORD;

    if (*gl_info).supported[ARB_FRAMEBUFFER_OBJECT as usize] != 0
        || (*gl_info).supported[EXT_FRAMEBUFFER_OBJECT as usize] != 0
    {
        (*caps).Caps2 |= WINED3DCAPS2_CANGENMIPMAP as DWORD;
    }

    if (*gl_info).supported[WINED3D_GL_BLEND_EQUATION as usize] != 0 {
        (*caps).PrimitiveMiscCaps |= WINED3DPMISCCAPS_BLENDOP as DWORD;
    }
    if (*gl_info).supported[EXT_BLEND_EQUATION_SEPARATE as usize] != 0
        && (*gl_info).supported[EXT_BLEND_FUNC_SEPARATE as usize] != 0
    {
        (*caps).PrimitiveMiscCaps |= WINED3DPMISCCAPS_SEPARATEALPHABLEND as DWORD;
    }
    if (*gl_info).supported[EXT_DRAW_BUFFERS2 as usize] != 0 {
        (*caps).PrimitiveMiscCaps |= WINED3DPMISCCAPS_INDEPENDENTWRITEMASKS as DWORD;
    }
    if (*gl_info).supported[ARB_FRAMEBUFFER_SRGB as usize] != 0 {
        (*caps).PrimitiveMiscCaps |= WINED3DPMISCCAPS_POSTBLENDSRGBCONVERT as DWORD;
    }
    if (!(*gl_info).quirks & (WINED3D_QUIRK_NO_INDEPENDENT_BIT_DEPTHS as u32)) != 0 {
        (*caps).PrimitiveMiscCaps |= WINED3DPMISCCAPS_MRTINDEPENDENTBITDEPTHS as DWORD;
    }

    if (*gl_info).supported[ARB_SAMPLER_OBJECTS as usize] != 0
        || (*gl_info).supported[EXT_TEXTURE_LOD_BIAS as usize] != 0
    {
        (*caps).RasterCaps |= WINED3DPRASTERCAPS_MIPMAPLODBIAS as DWORD;
    }

    if (*gl_info).supported[ARB_TEXTURE_FILTER_ANISOTROPIC as usize] != 0 {
        (*caps).RasterCaps |= WINED3DPRASTERCAPS_ANISOTROPY as DWORD;

        (*caps).TextureFilterCaps |= (WINED3DPTFILTERCAPS_MAGFANISOTROPIC
            | WINED3DPTFILTERCAPS_MINFANISOTROPIC) as DWORD;
    }

    if (*gl_info).supported[ARB_BLEND_FUNC_EXTENDED as usize] != 0 {
        (*caps).DestBlendCaps |= WINED3DPBLENDCAPS_SRCALPHASAT as DWORD;
    }

    if (*gl_info).supported[EXT_BLEND_COLOR as usize] != 0 {
        (*caps).SrcBlendCaps |= WINED3DPBLENDCAPS_BLENDFACTOR as DWORD;
        (*caps).DestBlendCaps |= WINED3DPBLENDCAPS_BLENDFACTOR as DWORD;
    }

    if (*gl_info).supported[EXT_TEXTURE3D as usize] != 0 {
        (*caps).TextureCaps |= (WINED3DPTEXTURECAPS_VOLUMEMAP
            | WINED3DPTEXTURECAPS_MIPVOLUMEMAP) as DWORD;
        if (*d3d_info).unconditional_npot() == 0 {
            (*caps).TextureCaps |= WINED3DPTEXTURECAPS_VOLUMEMAP_POW2 as DWORD;
        }

        (*caps).VolumeTextureFilterCaps |= (WINED3DPTFILTERCAPS_MAGFLINEAR
            | WINED3DPTFILTERCAPS_MAGFPOINT
            | WINED3DPTFILTERCAPS_MINFLINEAR
            | WINED3DPTFILTERCAPS_MINFPOINT
            | WINED3DPTFILTERCAPS_MIPFLINEAR
            | WINED3DPTFILTERCAPS_MIPFPOINT
            | WINED3DPTFILTERCAPS_LINEAR
            | WINED3DPTFILTERCAPS_LINEARMIPLINEAR
            | WINED3DPTFILTERCAPS_LINEARMIPNEAREST
            | WINED3DPTFILTERCAPS_MIPLINEAR
            | WINED3DPTFILTERCAPS_MIPNEAREST
            | WINED3DPTFILTERCAPS_NEAREST) as DWORD;

        (*caps).VolumeTextureAddressCaps |= (WINED3DPTADDRESSCAPS_INDEPENDENTUV
            | WINED3DPTADDRESSCAPS_CLAMP
            | WINED3DPTADDRESSCAPS_WRAP) as DWORD;

        if (*gl_info).supported[ARB_TEXTURE_BORDER_CLAMP as usize] != 0 {
            (*caps).VolumeTextureAddressCaps |= WINED3DPTADDRESSCAPS_BORDER as DWORD;
        }
        if (*gl_info).supported[ARB_TEXTURE_MIRRORED_REPEAT as usize] != 0 {
            (*caps).VolumeTextureAddressCaps |= WINED3DPTADDRESSCAPS_MIRROR as DWORD;
        }
        if (*gl_info).supported[ARB_TEXTURE_MIRROR_CLAMP_TO_EDGE as usize] != 0 {
            (*caps).VolumeTextureAddressCaps |= WINED3DPTADDRESSCAPS_MIRRORONCE as DWORD;
        }

        (*caps).MaxVolumeExtent = (*gl_info).limits.texture3d_size as DWORD;
    }

    if (*gl_info).supported[ARB_TEXTURE_CUBE_MAP as usize] != 0 {
        (*caps).TextureCaps |= (WINED3DPTEXTURECAPS_CUBEMAP
            | WINED3DPTEXTURECAPS_MIPCUBEMAP) as DWORD;
        if (*d3d_info).unconditional_npot() == 0 {
            (*caps).TextureCaps |= WINED3DPTEXTURECAPS_CUBEMAP_POW2 as DWORD;
        }

        (*caps).CubeTextureFilterCaps |= (WINED3DPTFILTERCAPS_MAGFLINEAR
            | WINED3DPTFILTERCAPS_MAGFPOINT
            | WINED3DPTFILTERCAPS_MINFLINEAR
            | WINED3DPTFILTERCAPS_MINFPOINT
            | WINED3DPTFILTERCAPS_MIPFLINEAR
            | WINED3DPTFILTERCAPS_MIPFPOINT
            | WINED3DPTFILTERCAPS_LINEAR
            | WINED3DPTFILTERCAPS_LINEARMIPLINEAR
            | WINED3DPTFILTERCAPS_LINEARMIPNEAREST
            | WINED3DPTFILTERCAPS_MIPLINEAR
            | WINED3DPTFILTERCAPS_MIPNEAREST
            | WINED3DPTFILTERCAPS_NEAREST) as DWORD;

        if (*gl_info).supported[ARB_TEXTURE_FILTER_ANISOTROPIC as usize] != 0 {
            (*caps).CubeTextureFilterCaps |= (WINED3DPTFILTERCAPS_MAGFANISOTROPIC
                | WINED3DPTFILTERCAPS_MINFANISOTROPIC) as DWORD;
        }
    }

    if (*gl_info).supported[ARB_TEXTURE_BORDER_CLAMP as usize] != 0 {
        (*caps).TextureAddressCaps |= WINED3DPTADDRESSCAPS_BORDER as DWORD;
    }
    if (*gl_info).supported[ARB_TEXTURE_MIRRORED_REPEAT as usize] != 0 {
        (*caps).TextureAddressCaps |= WINED3DPTADDRESSCAPS_MIRROR as DWORD;
    }
    if (*gl_info).supported[ARB_TEXTURE_MIRROR_CLAMP_TO_EDGE as usize] != 0 {
        (*caps).TextureAddressCaps |= WINED3DPTADDRESSCAPS_MIRRORONCE as DWORD;
    }

    if (*gl_info).supported[EXT_STENCIL_WRAP as usize] != 0 {
        (*caps).StencilCaps |= (WINED3DSTENCILCAPS_DECR
            | WINED3DSTENCILCAPS_INCR) as DWORD;
    }

    if (*gl_info).supported[WINED3D_GL_VERSION_2_0 as usize] != 0
        || (*gl_info).supported[EXT_STENCIL_TWO_SIDE as usize] != 0
        || (*gl_info).supported[ATI_SEPARATE_STENCIL as usize] != 0
    {
        (*caps).StencilCaps |= WINED3DSTENCILCAPS_TWOSIDED as DWORD;
    }

    (*caps).MaxAnisotropy = (*gl_info).limits.anisotropy as DWORD;

    if (*caps).VertexShaderVersion >= 3 as DWORD {
        (*caps).MaxVertexShader30InstructionSlots =
            max!((*caps).MaxVertexShader30InstructionSlots, (*gl_info).limits.arb_vs_instructions) as DWORD;
    }
    if (*caps).VertexShaderVersion >= 2 as DWORD {
        (*caps).VS20Caps.temp_count = max!((*caps).VS20Caps.temp_count, ((*gl_info).limits.arb_vs_temps) as i32) as INT;

        if (*gl_info).supported[ARB_HALF_FLOAT_VERTEX as usize] != 0 {
            (*caps).DeclTypes |= (WINED3DDTCAPS_FLOAT16_2 | WINED3DDTCAPS_FLOAT16_4) as DWORD;
        }
    }

    if (*caps).PixelShaderVersion >= 3 as DWORD {
        (*caps).MaxPixelShader30InstructionSlots =
            max!((*caps).MaxPixelShader30InstructionSlots, (*gl_info).limits.arb_ps_instructions) as DWORD;
    }
    if (*caps).PixelShaderVersion >= 2 as DWORD {
        (*caps).PS20Caps.temp_count = max!((*caps).PS20Caps.temp_count, ((*gl_info).limits.arb_ps_temps) as i32) as INT;
    }
}

unsafe extern "C" fn adapter_gl_check_format(
    adapter: *const wined3d_adapter,
    adapter_format: *const wined3d_format,
    rt_format: *const wined3d_format,
    ds_format: *const wined3d_format,
) -> BOOL {
    (TRUE as i32)
}

unsafe extern "C" fn adapter_gl_init_3d(device: *mut wined3d_device) -> HRESULT {
    wined3d_cs_init_object(
        (*device).cs as *mut wined3d_cs,
        Some(
            wined3d_device_gl_create_primary_opengl_context_cs
                as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
        ),
        wined3d_device_gl(device) as *mut ::core::ffi::c_void,
    );
    wined3d_cs_finish((*device).cs as *mut wined3d_cs, WINED3D_CS_QUEUE_DEFAULT);
    if (*device).context_count == 0 {
        return E_FAIL;
    }
    return WINED3D_OK;
}

unsafe extern "C" fn adapter_gl_uninit_3d(device: *mut wined3d_device) {
    WINE_TRACE!("device %p.\n", device);
    wined3d_device_destroy_default_samplers(device);
    wined3d_cs_destroy_object(
        (*device).cs as *mut wined3d_cs,
        Some(
            wined3d_device_gl_delete_opengl_contexts_cs
                as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
        ),
        wined3d_device_gl(device) as *mut ::core::ffi::c_void,
    );
    wined3d_cs_finish((*device).cs as *mut wined3d_cs, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn adapter_gl_map_bo_address(
    context: *mut wined3d_context,
    data: *const wined3d_bo_address,
    size: size_t,
    map_flags: uint32_t,
) -> *mut core::ffi::c_void {
    return wined3d_context_gl_map_bo_address(wined3d_context_gl(context), data, size, map_flags);
}

unsafe extern "C" fn adapter_gl_unmap_bo_address(
    context: *mut wined3d_context,
    data: *const wined3d_bo_address,
    range_count: u32,
    ranges: *const wined3d_range,
) {
    wined3d_context_gl_unmap_bo_address(wined3d_context_gl(context), data, range_count, ranges);
}

unsafe extern "C" fn adapter_gl_copy_bo_address(
    mut context: *mut wined3d_context,
    mut dst: *const wined3d_bo_address,
    mut src: *const wined3d_bo_address,
    mut range_count: ::core::ffi::c_uint,
    mut ranges: *const wined3d_range,
    mut map_flags: uint32_t,
) {
    wined3d_context_gl_copy_bo_address(
        wined3d_context_gl(context),
        dst,
        src,
        range_count,
        ranges,
        map_flags,
    );
}

unsafe extern "C" fn adapter_gl_flush_bo_address(
    context: *mut wined3d_context,
    data: *const wined3d_const_bo_address,
    size: size_t,
) {
    wined3d_context_gl_flush_bo_address(wined3d_context_gl(context), data, size);
}

unsafe extern "C" fn adapter_gl_alloc_bo(
    device: *mut wined3d_device,
    resource: *mut wined3d_resource,
    sub_resource_idx: ::core::ffi::c_uint,
    addr: *mut wined3d_bo_address,
) -> bool {
    let gl_info: *const wined3d_gl_info =
        core::ptr::addr_of!((*wined3d_adapter_gl((*device).adapter)).gl_info);
    let device_gl: *mut wined3d_device_gl = wined3d_device_gl(device);
    let mut bo_gl: *mut wined3d_bo_gl = core::ptr::null_mut();
    let mut binding: GLenum;
    let mut usage: GLenum;
    let mut coherent: bool = true;
    let mut flags: GLbitfield;
    let mut size: GLsizeiptr;

    wined3d_not_from_cs((*device).cs);
    assert!((*device).context_count != 0);

    if (*resource).type_ as ::core::ffi::c_uint == WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint
    {
        size = (*resource).size as GLsizeiptr;
        binding = wined3d_buffer_gl_binding_from_bind_flags(gl_info, (*resource).bind_flags as uint32_t);
        usage = GL_STATIC_DRAW as GLenum;
        flags = wined3d_resource_gl_storage_flags(resource);
        if ((*resource).usage & WINED3DUSAGE_DYNAMIC as uint32_t) != 0 {
            usage = GL_STREAM_DRAW as GLenum;
            coherent = false;
        }
    } else {
        let texture: *mut wined3d_texture = texture_from_resource(resource);

        size = (*(*texture).sub_resources.offset(sub_resource_idx as isize)).size as GLsizeiptr;
        binding = GL_PIXEL_UNPACK_BUFFER as GLenum;
        usage = GL_STREAM_DRAW as GLenum;
        flags = (GL_MAP_READ_BIT | GL_MAP_WRITE_BIT | GL_CLIENT_STORAGE_BIT)
            as GLbitfield;
    }

    bo_gl = malloc(core::mem::size_of::<wined3d_bo_gl>() as size_t)
        as *mut wined3d_bo_gl;
    if bo_gl.is_null() {
        return false;
    }

    if !wined3d_device_gl_create_bo(
        device_gl,
        core::ptr::null_mut(),
        size,
        binding,
        usage,
        coherent,
        flags,
        bo_gl,
    ) {
        free(bo_gl as *mut ::core::ffi::c_void);
        return false;
    }

    if !(*bo_gl).memory.is_null() {
        let chunk: *mut wined3d_allocator_chunk_gl =
            wined3d_allocator_chunk_gl((*(*bo_gl).memory).chunk);

        wined3d_allocator_chunk_gl_lock(chunk);

        (*bo_gl).b.map_ptr = (*chunk).c.map_ptr;
        if !(*bo_gl).b.map_ptr.is_null() {
            (*chunk).c.map_count = (*chunk).c.map_count.wrapping_add(1);
        }

        wined3d_allocator_chunk_gl_unlock(chunk);
    }

    (*addr).buffer_object = core::ptr::addr_of_mut!((*bo_gl).b);
    (*addr).addr = core::ptr::null_mut();

    if (*bo_gl).b.map_ptr.is_null() {
        WINE_WARN_!(__wine_dbch_d3d_perf,
            "BO %p (chunk %p) is not persistently mapped.\n",
            bo_gl,
            if !(*bo_gl).memory.is_null() {
                (*(*bo_gl).memory).chunk
            } else {
                core::ptr::null_mut()
            }
        );
        wined3d_cs_map_bo_address(
            (*device).cs as *mut wined3d_cs,
            addr,
            (*resource).size as size_t,
            (WINED3D_MAP_WRITE | WINED3D_MAP_DISCARD) as ::core::ffi::c_uint,
        );
    }

    return true;
}

unsafe extern "C" fn adapter_gl_destroy_bo(context: *mut wined3d_context, bo: *mut wined3d_bo) {
    wined3d_context_gl_destroy_bo(wined3d_context_gl(context), wined3d_bo_gl(bo));
}

unsafe extern "C" fn adapter_gl_create_swapchain(
    device: *mut wined3d_device,
    desc: *const wined3d_swapchain_desc,
    state_parent: *mut wined3d_swapchain_state_parent,
    parent: *mut ::core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    swapchain: *mut *mut wined3d_swapchain,
) -> HRESULT {
    let mut swapchain_gl: *mut wined3d_swapchain_gl = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    swapchain_gl = calloc(
        (1 as usize) as _,
        (core::mem::size_of::<wined3d_swapchain_gl>()) as _,
    ) as *mut wined3d_swapchain_gl;
    if swapchain_gl.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = wined3d_swapchain_gl_init(swapchain_gl, device, desc, state_parent, parent, parent_ops);
    if hr < 0 {
        WARN!("Failed to initialise swapchain, hr %#lx.\n", hr);
        free(swapchain_gl as *mut ::core::ffi::c_void);
        return hr;
    }

    *swapchain = core::ptr::addr_of_mut!((*swapchain_gl).s);

    return hr;
}

unsafe extern "C" fn adapter_gl_destroy_swapchain(swapchain: *mut wined3d_swapchain) {
    let swapchain_gl: *mut wined3d_swapchain_gl = wined3d_swapchain_gl(swapchain);
    wined3d_swapchain_gl_cleanup(swapchain_gl);
    free(swapchain_gl as *mut core::ffi::c_void);
}

unsafe extern "C" fn adapter_gl_create_buffer(
    device: *mut wined3d_device,
    desc: *const wined3d_buffer_desc,
    data: *const wined3d_sub_resource_data,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    buffer: *mut *mut wined3d_buffer,
) -> HRESULT {
    let mut buffer_gl: *mut wined3d_buffer_gl = core::ptr::null_mut();
    let mut hr: HRESULT;

    buffer_gl = calloc(1, (core::mem::size_of::<wined3d_buffer_gl>() as size_t) as _) as *mut wined3d_buffer_gl;
    if buffer_gl.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = wined3d_buffer_gl_init(buffer_gl, device, desc, data, parent, parent_ops);
    if hr < 0 {
        WARN!("Failed to initialise buffer, hr %#lx.\n", hr);
        free(buffer_gl as *mut core::ffi::c_void);
        return hr;
    }

    *buffer = core::ptr::addr_of_mut!((*buffer_gl).b);

    return hr;
}

unsafe extern "C" fn adapter_gl_destroy_buffer(buffer: *mut wined3d_buffer) {
    let buffer_gl: *mut wined3d_buffer_gl = wined3d_buffer_gl(buffer);
    let device: *mut wined3d_device = (*buffer_gl).b.resource.device;
    let swapchain_count: core::ffi::c_uint = (*device).swapchain_count as core::ffi::c_uint;
    WINE_TRACE!("buffer_gl %p.\n", buffer_gl);

    /* Take a reference to the device, in case releasing the buffer would
     * cause the device to be destroyed. However, swapchain resources don't
     * take a reference to the device, and we wouldn't want to increment the
     * refcount on a device that's in the process of being destroyed. */
    if swapchain_count != 0 {
        wined3d_device_incref(device);
    }
    wined3d_buffer_cleanup(&raw mut (*buffer_gl).b);
    wined3d_cs_destroy_object(
        (*device).cs as *mut wined3d_cs,
        Some(free as unsafe extern "C" fn(*mut core::ffi::c_void) -> ()),
        buffer_gl as *mut core::ffi::c_void,
    );
    if swapchain_count != 0 {
        wined3d_device_decref(device);
    }
}

unsafe extern "C" fn adapter_gl_create_texture(
    device: *mut wined3d_device,
    desc: *const wined3d_resource_desc,
    layer_count: ::core::ffi::c_uint,
    level_count: ::core::ffi::c_uint,
    flags: uint32_t,
    parent: *mut ::core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    texture: *mut *mut wined3d_texture,
) -> HRESULT {
    let mut texture_gl: *mut wined3d_texture_gl = core::ptr::null_mut();
    let mut hr: HRESULT;

    texture_gl = wined3d_texture_allocate_object_memory(
        core::mem::size_of::<wined3d_texture_gl>() as SIZE_T,
        level_count as SIZE_T,
        layer_count as SIZE_T,
    ) as *mut wined3d_texture_gl;
    if texture_gl.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = wined3d_texture_gl_init(
        texture_gl,
        device,
        desc,
        layer_count,
        level_count,
        flags,
        parent,
        parent_ops,
    );
    if hr < 0 {
        WARN!("Failed to initialise texture, hr %#lx.\n", hr);
        free(texture_gl as *mut ::core::ffi::c_void);
        return hr;
    }

    *texture = core::ptr::addr_of_mut!((*texture_gl).t);

    return hr;
}

unsafe extern "C" fn adapter_gl_destroy_texture(mut texture: *mut wined3d_texture) {
    let mut texture_gl: *mut wined3d_texture_gl = wined3d_texture_gl(texture);
    let mut device: *mut wined3d_device = (*texture_gl).t.resource.device;
    let mut swapchain_count: ::core::ffi::c_uint = (*device).swapchain_count as ::core::ffi::c_uint;
    WINE_TRACE!("texture_gl %p.\n", texture_gl);
    if swapchain_count != 0 {
        wined3d_device_incref(device);
    }
    wined3d_texture_sub_resources_destroyed(texture);
    (*(*texture).resource.parent_ops)
        .wined3d_object_destroyed
        .unwrap_unchecked()((*texture).resource.parent);
    wined3d_texture_cleanup(core::ptr::addr_of_mut!((*texture_gl).t));
    wined3d_cs_destroy_object(
        (*device).cs as *mut wined3d_cs,
        Some(free as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        texture_gl as *mut ::core::ffi::c_void,
    );
    if swapchain_count != 0 {
        wined3d_device_decref(device);
    }
}

unsafe extern "C" fn adapter_gl_create_rendertarget_view(
    desc: *const wined3d_view_desc,
    resource: *mut wined3d_resource,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    view: *mut *mut wined3d_rendertarget_view,
) -> HRESULT {
    let mut view_gl: *mut wined3d_rendertarget_view_gl = core::ptr::null_mut();
    let mut hr: HRESULT;

    TRACE!(
        "desc %s, resource %p, parent %p, parent_ops %p, view %p.\n",
        wined3d_debug_view_desc(desc, resource),
        resource,
        parent,
        parent_ops,
        view
    );

    view_gl = calloc(
        1,
        (core::mem::size_of::<wined3d_rendertarget_view_gl>()) as _,
    ) as *mut wined3d_rendertarget_view_gl;
    if view_gl.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = wined3d_rendertarget_view_gl_init(view_gl, desc, resource, parent, parent_ops);
    if hr < 0 {
        WARN!("Failed to initialise view, hr %#lx.\n", hr);
        free(view_gl as *mut core::ffi::c_void);
        return hr;
    }

    TRACE!("Created render target view %p.\n", view_gl);

    *view = core::ptr::addr_of_mut!((*view_gl).v);
    return hr;
}

unsafe extern "C" fn wined3d_view_gl_destroy_object(object: *mut ::core::ffi::c_void) {
    let ctx: *mut wined3d_view_gl_destroy_ctx = object as *mut wined3d_view_gl_destroy_ctx;
    let mut gl_info: *const wined3d_gl_info = core::ptr::null::<wined3d_gl_info>();
    let mut context: *mut wined3d_context = core::ptr::null_mut::<wined3d_context>();
    let mut device: *mut wined3d_device = core::ptr::null_mut::<wined3d_device>();
    let mut counter_id: GLuint;

    TRACE!("ctx %p.\n", ctx);

    device = (*ctx).device;

    counter_id = if !(*ctx).counter_bo.is_null() {
        (*(*ctx).counter_bo).id
    } else {
        0
    };
    if (*(*ctx).gl_view).name != 0 || counter_id != 0 {
        context = context_acquire(
            device,
            core::ptr::null_mut::<wined3d_texture>(),
            0,
        );
        gl_info = (*wined3d_context_gl(context)).gl_info;
        if (*(*ctx).gl_view).name != 0 {
            context_gl_resource_released(device, (*(*ctx).gl_view).name, (FALSE) as BOOL);
            (*gl_info)
                .gl_ops
                .gl
                .p_glDeleteTextures
                .unwrap_unchecked()(
                1,
                &raw const (*(*ctx).gl_view).name,
            );
        }
        if counter_id != 0 {
            wined3d_context_gl_destroy_bo(wined3d_context_gl(context), (*ctx).counter_bo);
        }
        checkGLcall!(gl_info, "delete resources");
        context_release(context);
    }
    if !(*ctx).bo_user.is_null() && (*(*ctx).bo_user).valid != false {
        list_remove(&raw mut (*(*ctx).bo_user).entry);
    }

    free((*ctx).object);
    free((*ctx).free as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn wined3d_view_gl_destroy(
    mut device: *mut wined3d_device,
    mut gl_view: *const wined3d_gl_view,
    mut bo_user: *mut wined3d_bo_user,
    mut counter_bo: *mut wined3d_bo_gl,
    mut object: *mut ::core::ffi::c_void,
) {
    let mut ctx: *mut wined3d_view_gl_destroy_ctx = core::ptr::null_mut::<wined3d_view_gl_destroy_ctx>();
    let mut c: wined3d_view_gl_destroy_ctx = wined3d_view_gl_destroy_ctx {
        device: core::ptr::null_mut::<wined3d_device>(),
        gl_view: core::ptr::null::<wined3d_gl_view>(),
        bo_user: core::ptr::null_mut::<wined3d_bo_user>(),
        counter_bo: core::ptr::null_mut::<wined3d_bo_gl>(),
        object: core::ptr::null_mut::<::core::ffi::c_void>(),
        free: core::ptr::null_mut::<wined3d_view_gl_destroy_ctx>(),
    };
    ctx = malloc(core::mem::size_of::<wined3d_view_gl_destroy_ctx>() as size_t)
        as *mut wined3d_view_gl_destroy_ctx;
    if ctx.is_null() {
        ctx = core::ptr::addr_of_mut!(c);
    }
    (*ctx).device = device;
    (*ctx).gl_view = gl_view;
    (*ctx).bo_user = bo_user;
    (*ctx).counter_bo = counter_bo;
    (*ctx).object = object;
    (*ctx).free = if ctx != core::ptr::addr_of_mut!(c) {
        ctx
    } else {
        core::ptr::null_mut::<wined3d_view_gl_destroy_ctx>()
    };
    wined3d_cs_destroy_object(
        (*device).cs as *mut wined3d_cs,
        Some(
            wined3d_view_gl_destroy_object as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
        ),
        ctx as *mut ::core::ffi::c_void,
    );
    if ctx == core::ptr::addr_of_mut!(c) {
        wined3d_cs_finish((*device).cs as *mut wined3d_cs, WINED3D_CS_QUEUE_DEFAULT);
    }
}

unsafe extern "C" fn adapter_gl_destroy_rendertarget_view(view: *mut wined3d_rendertarget_view) {
    let view_gl: *mut wined3d_rendertarget_view_gl = wined3d_rendertarget_view_gl(view);
    let resource: *mut wined3d_resource = (*view_gl).v.resource;

    TRACE!("view_gl %p.\n", view_gl);

    wined3d_rendertarget_view_cleanup(core::ptr::addr_of_mut!((*view_gl).v));
    wined3d_view_gl_destroy(
        (*resource).device,
        core::ptr::addr_of_mut!((*view_gl).gl_view),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        (view_gl).cast(),
    );
}

unsafe extern "C" fn adapter_gl_create_shader_resource_view(
    desc: *const wined3d_view_desc,
    resource: *mut wined3d_resource,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    view: *mut *mut wined3d_shader_resource_view,
) -> HRESULT {
    let mut view_gl: *mut wined3d_shader_resource_view_gl =
        core::ptr::null_mut::<wined3d_shader_resource_view_gl>();
    let mut hr: HRESULT = 0;

    view_gl = calloc(
        1,
        (core::mem::size_of::<wined3d_shader_resource_view_gl>()) as _,
    ) as *mut wined3d_shader_resource_view_gl;
    if view_gl.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = wined3d_shader_resource_view_gl_init(view_gl, desc, resource, parent, parent_ops);
    if hr < 0 {
        WARN!("Failed to initialise view, hr %#lx.\n", hr);
        free(view_gl as *mut core::ffi::c_void);
        return hr;
    }

    *view = &raw mut (*view_gl).v;

    return hr;
}

unsafe extern "C" fn adapter_gl_destroy_shader_resource_view(view: *mut wined3d_shader_resource_view) {
    let mut view_gl: *mut wined3d_shader_resource_view_gl = wined3d_shader_resource_view_gl(view);
    let mut resource: *mut wined3d_resource = (*view_gl).v.resource;
    WINE_TRACE!("view_gl %p.\n", view_gl);
    wined3d_shader_resource_view_cleanup(core::ptr::addr_of_mut!((*view_gl).v));
    wined3d_view_gl_destroy(
        (*resource).device,
        core::ptr::addr_of_mut!((*view_gl).gl_view),
        core::ptr::addr_of_mut!((*view_gl).bo_user),
        core::ptr::null_mut::<wined3d_bo_gl>(),
        view_gl as *mut core::ffi::c_void,
    );
}

unsafe extern "C" fn adapter_gl_create_unordered_access_view(
    desc: *const wined3d_view_desc,
    resource: *mut wined3d_resource,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    view: *mut *mut wined3d_unordered_access_view,
) -> HRESULT {
    let mut view_gl: *mut wined3d_unordered_access_view_gl = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    TRACE!(
        "desc %s, resource %p, parent %p, parent_ops %p, view %p.\n",
        wined3d_debug_view_desc(desc, resource),
        resource,
        parent,
        parent_ops,
        view
    );

    view_gl = calloc(
        (1 as usize) as _,
        (core::mem::size_of::<wined3d_unordered_access_view_gl>() as usize) as _,
    ) as *mut wined3d_unordered_access_view_gl;
    if view_gl.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = wined3d_unordered_access_view_gl_init(view_gl, desc, resource, parent, parent_ops);
    if hr < 0 {
        WARN!("Failed to initialise view, hr %#lx.\n", hr);
        free(view_gl as *mut core::ffi::c_void);
        return hr;
    }

    TRACE!("Created unordered access view %p.\n", view_gl);

    *view = core::ptr::addr_of_mut!((*view_gl).v);

    return hr;
}

unsafe extern "C" fn adapter_gl_destroy_unordered_access_view(
    mut view: *mut wined3d_unordered_access_view,
) {
    let mut view_gl: *mut wined3d_unordered_access_view_gl = wined3d_unordered_access_view_gl(view);
    let mut resource: *mut wined3d_resource = (*view_gl).v.resource;
    WINE_TRACE!("view_gl %p.\n", view_gl);
    wined3d_unordered_access_view_cleanup(&raw mut (*view_gl).v);
    wined3d_view_gl_destroy(
        (*resource).device,
        &raw mut (*view_gl).gl_view,
        &raw mut (*view_gl).bo_user,
        &raw mut (*view_gl).counter_bo,
        view_gl as *mut ::core::ffi::c_void,
    );
}

unsafe extern "C" fn adapter_gl_create_sampler(
    mut device: *mut wined3d_device,
    mut desc: *const wined3d_sampler_desc,
    mut parent: *mut core::ffi::c_void,
    mut parent_ops: *const wined3d_parent_ops,
    mut sampler: *mut *mut wined3d_sampler,
) -> HRESULT {
    let mut sampler_gl: *mut wined3d_sampler_gl = core::ptr::null_mut();

    sampler_gl = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<wined3d_sampler_gl>() as size_t) as _,
    ) as *mut wined3d_sampler_gl;
    if sampler_gl.is_null() {
        return E_OUTOFMEMORY;
    }

    wined3d_sampler_gl_init(sampler_gl, device, desc, parent, parent_ops);

    *sampler = core::ptr::addr_of_mut!((*sampler_gl).s);

    return WINED3D_OK;
}

unsafe extern "C" fn wined3d_sampler_gl_destroy_object(object: *mut core::ffi::c_void) {
    let sampler_gl: *mut wined3d_sampler_gl = object as *mut wined3d_sampler_gl;
    let mut gl_info: *const wined3d_gl_info = core::ptr::null();
    let mut context: *mut wined3d_context = core::ptr::null_mut();

    if (*sampler_gl).name != 0 {
        context = context_acquire((*sampler_gl).s.device, core::ptr::null_mut(), 0);
        gl_info = (*wined3d_context_gl(context)).gl_info;
        ((*gl_info).gl_ops.ext.p_glDeleteSamplers.unwrap_unchecked())(1, &raw mut (*sampler_gl).name);
        context_release(context);
    }

    free(sampler_gl as *mut core::ffi::c_void);
}

unsafe extern "C" fn adapter_gl_destroy_sampler(mut sampler: *mut wined3d_sampler) {
    let mut sampler_gl: *mut wined3d_sampler_gl = wined3d_sampler_gl(sampler);
    TRACE!("sampler_gl %p.\n", sampler_gl);
    wined3d_cs_destroy_object(
        (*(*sampler).device).cs as *mut wined3d_cs,
        Some(
            wined3d_sampler_gl_destroy_object
                as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
        ),
        sampler_gl as *mut ::core::ffi::c_void,
    );
}

unsafe extern "C" fn adapter_gl_create_query(
    device: *mut wined3d_device,
    r#type: wined3d_query_type,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    query: *mut *mut wined3d_query,
) -> HRESULT {
    // TRACE("device %p, type %#x, parent %p, parent_ops %p, query %p.\n", device, type, parent, parent_ops, query);
    wined3d_query_gl_create(device, r#type, parent, parent_ops, query)
}

unsafe extern "C" fn wined3d_query_gl_destroy_object(mut object: *mut ::core::ffi::c_void) {
    let mut query: *mut wined3d_query = object as *mut wined3d_query;
    WINE_TRACE!("query %p.\n", query);
    if (*query).buffer_object != 0 {
        let mut context: *mut wined3d_context = ::core::ptr::null_mut::<wined3d_context>();
        context = context_acquire(
            (*query).device,
            ::core::ptr::null_mut::<wined3d_texture>(),
            0 as ::core::ffi::c_uint,
        );
        wined3d_query_gl_destroy_buffer_object(wined3d_context_gl(context), query);
        context_release(context);
    }
    (*(*query).query_ops)
        .query_destroy
        .unwrap_unchecked()(query);
}

unsafe extern "C" fn adapter_gl_destroy_query(query: *mut wined3d_query) {
    wined3d_cs_destroy_object((*(*query).device).cs, Some(wined3d_query_gl_destroy_object), query as *mut core::ffi::c_void);
}

unsafe extern "C" fn adapter_gl_flush_context(mut context: *mut wined3d_context) {
    let mut context_gl: *mut wined3d_context_gl = wined3d_context_gl(context);
    if (*context_gl).valid() != 0 {
        (*(*context_gl).gl_info)
            .gl_ops
            .gl
            .p_glFlush
            .unwrap_unchecked()();
    }
}

unsafe extern "C" fn adapter_gl_clear_uav(
    mut context: *mut wined3d_context,
    mut view: *mut wined3d_unordered_access_view,
    mut clear_value: *const wined3d_uvec4,
    mut fp: bool,
) {
    wined3d_unordered_access_view_gl_clear(
        wined3d_unordered_access_view_gl(view),
        clear_value,
        wined3d_context_gl(context),
        fp,
    );
}

unsafe extern "C" fn adapter_gl_generate_mipmap(context: *mut wined3d_context, view: *mut wined3d_shader_resource_view) {
    TRACE!("context %p, view %p.\n", context, view);
    wined3d_shader_resource_view_gl_generate_mipmap(wined3d_shader_resource_view_gl(view), wined3d_context_gl(context));
}

unsafe extern "C" fn wined3d_adapter_gl_init_d3d_info(
    mut adapter_gl: *mut wined3d_adapter_gl,
    mut wined3d_creation_flags: uint32_t,
) {
    let mut gl_info: *const wined3d_gl_info = &raw mut (*adapter_gl).gl_info;
    let mut d3d_info: *mut wined3d_d3d_info = &raw mut (*adapter_gl).a.d3d_info;
    let mut vertex_caps: wined3d_vertex_caps = core::mem::zeroed();
    let mut shader_caps: shader_caps = core::mem::zeroed();
    let mut f: [GLfloat; 2 as usize] = [0.; 2 as usize];

    (*(*adapter_gl).a.shader_backend)
        .shader_get_caps
        .unwrap_unchecked()(&raw mut (*adapter_gl).a, &raw mut shader_caps);
    (*(*adapter_gl).a.vertex_pipe)
        .vp_get_caps
        .unwrap_unchecked()(&raw mut (*adapter_gl).a, &raw mut vertex_caps);
    (*adapter_gl).a.misc_state_template =
        &raw const misc_state_template_gl as *const wined3d_state_entry_template;
    (*(*adapter_gl).a.fragment_pipe)
        .get_caps
        .unwrap_unchecked()(
        &raw mut (*adapter_gl).a,
        &raw mut (*d3d_info).ffp_fragment_caps,
    );

    (*d3d_info).limits.vs_version = shader_caps.vs_version;
    (*d3d_info).limits.hs_version = shader_caps.hs_version;
    (*d3d_info).limits.ds_version = shader_caps.ds_version;
    (*d3d_info).limits.gs_version = shader_caps.gs_version;
    (*d3d_info).limits.ps_version = shader_caps.ps_version;
    (*d3d_info).limits.cs_version = shader_caps.cs_version;
    (*d3d_info).limits.vs_uniform_count = shader_caps.vs_uniform_count as DWORD;
    (*d3d_info).limits.ps_uniform_count = shader_caps.ps_uniform_count as DWORD;
    (*d3d_info).limits.varying_count = shader_caps.varying_count;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        TRACE!(
            "Max texture stages: %u.\n",
            (*d3d_info).ffp_fragment_caps.max_blend_stages,
        );
    }

    (*d3d_info).limits.ffp_vertex_blend_matrices =
        vertex_caps.max_vertex_blend_matrices as ::core::ffi::c_uint;
    (*d3d_info).limits.active_light_count = vertex_caps.max_active_lights as ::core::ffi::c_uint;
    (*d3d_info).limits.max_rt_count = (*gl_info).limits.buffers;
    (*d3d_info).limits.max_clip_distances = (*gl_info).limits.user_clip_distances;
    (*d3d_info).limits.texture_size = (*gl_info).limits.texture_size;
    (*d3d_info).limits.sample_count = (*gl_info).limits.samples;

    (*gl_info)
        .gl_ops
        .gl
        .p_glGetFloatv
        .unwrap_unchecked()(
        (if (*gl_info).supported[WINED3D_GL_LEGACY_CONTEXT as usize] != 0 {
            GL_ALIASED_POINT_SIZE_RANGE
        } else {
            GL_POINT_SIZE_RANGE
        }) as GLenum,
        &raw mut f as *mut GLfloat,
    );
    (*d3d_info).limits.pointsize_max =
        f[1 as usize] as ::core::ffi::c_float;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        TRACE!(
            "Maximum point size support - max point size %.8e.\n",
            f[1 as usize] as ::core::ffi::c_double,
        );
    }

    (*d3d_info).wined3d_creation_flags = wined3d_creation_flags;
    (*d3d_info).set_emulated_flatshading(vertex_caps.emulated_flatshading as uint32_t as uint32_t);
    (*d3d_info).set_ffp_alpha_test(
        ((*gl_info).supported[WINED3D_GL_LEGACY_CONTEXT as usize] != 0)
            as ::core::ffi::c_int as uint32_t as uint32_t,
    );
    (*d3d_info).set_shader_double_precision(
        ((shader_caps.wined3d_caps as ::core::ffi::c_uint & (WINED3D_SHADER_CAP_DOUBLE_PRECISION as ::core::ffi::c_uint)) != 0)
            as ::core::ffi::c_int as uint32_t as uint32_t,
    );
    (*d3d_info).set_shader_output_interpolation(
        ((shader_caps.wined3d_caps as ::core::ffi::c_uint & (WINED3D_SHADER_CAP_OUTPUT_INTERPOLATION as ::core::ffi::c_uint))
            != 0) as ::core::ffi::c_int as uint32_t as uint32_t,
    );
    (*d3d_info).set_viewport_array_index_any_shader(
        ((*gl_info).supported[ARB_SHADER_VIEWPORT_LAYER_ARRAY as usize] != 0)
            as ::core::ffi::c_int as uint32_t as uint32_t,
    );
    (*d3d_info).set_stencil_export(
        ((*gl_info).supported[ARB_SHADER_STENCIL_EXPORT as usize] != 0)
            as ::core::ffi::c_int as uint32_t as uint32_t,
    );
    (*d3d_info).set_unconditional_npot(
        ((*gl_info).supported[ARB_TEXTURE_NON_POWER_OF_TWO as usize] != 0)
            as ::core::ffi::c_int as uint32_t as uint32_t,
    );
    (*d3d_info).set_draw_base_vertex_offset(
        ((*gl_info).supported[ARB_DRAW_ELEMENTS_BASE_VERTEX as usize] != 0)
            as ::core::ffi::c_int as uint32_t as uint32_t,
    );
    (*d3d_info).set_vertex_bgra(
        ((*gl_info).supported[ARB_VERTEX_ARRAY_BGRA as usize] != 0)
            as ::core::ffi::c_int as uint32_t as uint32_t,
    );
    (*d3d_info).set_texture_swizzle(
        ((*gl_info).supported[ARB_TEXTURE_SWIZZLE as usize] != 0)
            as ::core::ffi::c_int as uint32_t as uint32_t,
    );
    (*d3d_info).set_srgb_read_control(
        ((*gl_info).supported[EXT_TEXTURE_SRGB_DECODE as usize] != 0)
            as ::core::ffi::c_int as uint32_t as uint32_t,
    );
    (*d3d_info).set_srgb_write_control(
        ((*gl_info).supported[ARB_FRAMEBUFFER_SRGB as usize] != 0)
            as ::core::ffi::c_int as uint32_t as uint32_t,
    );
    (*d3d_info).set_clip_control(
        ((*gl_info).supported[ARB_CLIP_CONTROL as usize] != 0)
            as ::core::ffi::c_int as uint32_t as uint32_t,
    );
    (*d3d_info).set_full_ffp_varyings(
        ((shader_caps.wined3d_caps as ::core::ffi::c_uint & (WINED3D_SHADER_CAP_FULL_FFP_VARYINGS as ::core::ffi::c_uint))
            != 0) as ::core::ffi::c_int as uint32_t as uint32_t,
    );
    (*d3d_info).set_scaled_resolve(
        ((*gl_info).supported[EXT_FRAMEBUFFER_MULTISAMPLE_BLIT_SCALED as usize] != 0)
            as ::core::ffi::c_int as uint32_t as uint32_t,
    );
    (*d3d_info).set_pbo(
        ((*gl_info).supported[ARB_PIXEL_BUFFER_OBJECT as usize] != 0)
            as ::core::ffi::c_int as uint32_t as uint32_t,
    );
    (*d3d_info).set_subpixel_viewport(
        ((*gl_info).limits.viewport_subpixel_bits >= 8 as ::core::ffi::c_uint)
            as ::core::ffi::c_int as uint32_t as uint32_t,
    );
    (*d3d_info).set_fences(wined3d_fence_supported(gl_info) as uint32_t as uint32_t);
    (*d3d_info).feature_level = feature_level_from_caps(
        gl_info,
        &raw mut shader_caps,
        &raw mut (*d3d_info).ffp_fragment_caps,
    );
    (*d3d_info).filling_convention_offset = (*gl_info).filling_convention_offset;
    (*d3d_info).set_persistent_map(
        ((*gl_info).supported[ARB_BUFFER_STORAGE as usize] != 0)
            as ::core::ffi::c_int as uint32_t as uint32_t,
    );
    (*d3d_info).set_ffp_hlsl(wined3d_settings.ffp_hlsl as uint32_t as uint32_t);

    if (*gl_info).supported[ARB_TEXTURE_MULTISAMPLE as usize] != 0 {
        (*d3d_info).multisample_draw_location = WINED3D_LOCATION_TEXTURE_RGB as DWORD;
    } else {
        (*d3d_info).multisample_draw_location = WINED3D_LOCATION_RB_MULTISAMPLE as DWORD;
    };
}

unsafe extern "C" fn wined3d_adapter_find_fill_offset(ctx: *mut wined3d_caps_gl_ctx) -> f32 {
    static test_array: [f32; 6] = [
        0.0f32,
        -1.0f32 / 1024.0f32,
        -1.0f32 / 512.0f32,
        -1.0f32 / 256.0f32,
        -1.0f32 / 128.0f32,
        -1.0f32 / 64.0f32,
    ];
    let mut upper: u32 = ARRAY_SIZE!(test_array) as u32;
    let mut lower: u32 = 0;
    let mut test: u32;
    let mut value: f32 = 0.0;

    while upper != lower {
        test = upper.wrapping_add(lower) / 2;
        value = test_array[test as usize];

        if wined3d_caps_gl_ctx_test_filling_convention(ctx, value) != false {
            upper = test;
        } else {
            lower = test.wrapping_add(1);
        }
    }

    if (upper as usize) < ARRAY_SIZE!(test_array) as usize {
        value = test_array[upper as usize];
        if value != 0.0 {
            WARN!(
                "Using a filling convention fixup offset of -1/%f.\n",
                (-1.0f32 / value) as f64
            );
        } else {
            TRACE!("No need for a filling convention offset.\n");
        }

        return value;
    }

    /* This value was used unconditionally before the dynamic test function was
     * introduced. */
    return -1.0f32 / 64.0f32;
}

unsafe extern "system" fn wined3d_no_3d_warning_proc(param: *mut core::ffi::c_void) -> DWORD {
    MessageBoxW(core::ptr::null_mut(),
            RUSTCONV_WSTR_0D54D2A9857B.as_ptr(),
            RUSTCONV_WSTR_C1AB1E21E959.as_ptr(),
            (MB_OK | MB_ICONWARNING | MB_SETFOREGROUND | MB_TOPMOST) as UINT);
    return 0 as DWORD;
}

unsafe extern "C" fn wined3d_warn_no_3d() {
    static mut warned: LONG = 0;
    let mut thread: HANDLE = core::ptr::null_mut();
    let prev: LONG = match (*(core::ptr::addr_of_mut!(warned) as *const core::sync::atomic::AtomicI32))
        .compare_exchange(
            0,
            1,
            core::sync::atomic::Ordering::SeqCst,
            core::sync::atomic::Ordering::SeqCst,
        ) {
        Ok(v) => v,
        Err(v) => v,
    };
    if prev != 0 {
        return;
    }
    thread = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(wined3d_no_3d_warning_proc),
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
    );
    if !thread.is_null() {
        CloseHandle(thread);
    }
}

unsafe extern "C" fn wined3d_adapter_gl_init(
    adapter_gl: *mut wined3d_adapter_gl,
    ordinal: u32,
    wined3d_creation_flags: u32,
) -> BOOL {
    let supported_gl_versions: [u32; 3] = [
        MAKEDWORD_VERSION!(4, 4),
        MAKEDWORD_VERSION!(3, 2),
        MAKEDWORD_VERSION!(1, 0),
    ];
    let driver_info: *mut wined3d_driver_info = core::ptr::addr_of_mut!((*adapter_gl).a.driver_info);
    let gl_info: *mut wined3d_gl_info = core::ptr::addr_of_mut!((*adapter_gl).gl_info);
    let mut caps_gl_ctx: wined3d_caps_gl_ctx = core::mem::zeroed();
    let mut primary_luid: LUID = core::mem::zeroed();
    let mut luid: *mut LUID = core::ptr::null_mut();
    let mut i: u32;

    let required_extensions = [
        (ARB_FRAGMENT_SHADER, b"ARB_fragment_shader\0".as_ptr() as *const i8),
        (ARB_SHADING_LANGUAGE_100, b"ARB_shading_language_100\0".as_ptr() as *const i8),
        (ARB_VERTEX_SHADER, b"ARB_vertex_shader\0".as_ptr() as *const i8),
        (EXT_FRAMEBUFFER_OBJECT, b"EXT_framebuffer_object\0".as_ptr() as *const i8),
    ];

    if ordinal == 0
        && wined3d_get_primary_adapter_luid(core::ptr::addr_of_mut!(primary_luid)) != 0
    {
        luid = core::ptr::addr_of_mut!(primary_luid);
    }

    if wined3d_adapter_init(
        core::ptr::addr_of_mut!((*adapter_gl).a),
        ordinal,
        luid,
        core::ptr::addr_of!(wined3d_adapter_gl_ops),
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    /* Dynamically load all GL core functions */
    {
        let mod_gl: HMODULE = GetModuleHandleA(b"opengl32.dll\0".as_ptr() as *const i8);
        // wgl_driver.h ALL_WGL_FUNCS with `USE_GL_FUNC(f)` -> `gl_ops.gl.p_##f = GetProcAddress(mod_gl, #f)`.
        (*gl_info).gl_ops.gl.p_glAccum = core::mem::transmute(GetProcAddress(mod_gl, b"glAccum\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glAlphaFunc = core::mem::transmute(GetProcAddress(mod_gl, b"glAlphaFunc\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glAreTexturesResident = core::mem::transmute(GetProcAddress(mod_gl, b"glAreTexturesResident\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glArrayElement = core::mem::transmute(GetProcAddress(mod_gl, b"glArrayElement\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glBegin = core::mem::transmute(GetProcAddress(mod_gl, b"glBegin\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glBindTexture = core::mem::transmute(GetProcAddress(mod_gl, b"glBindTexture\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glBitmap = core::mem::transmute(GetProcAddress(mod_gl, b"glBitmap\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glBlendFunc = core::mem::transmute(GetProcAddress(mod_gl, b"glBlendFunc\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glCallList = core::mem::transmute(GetProcAddress(mod_gl, b"glCallList\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glCallLists = core::mem::transmute(GetProcAddress(mod_gl, b"glCallLists\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glClear = core::mem::transmute(GetProcAddress(mod_gl, b"glClear\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glClearAccum = core::mem::transmute(GetProcAddress(mod_gl, b"glClearAccum\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glClearColor = core::mem::transmute(GetProcAddress(mod_gl, b"glClearColor\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glClearDepth = core::mem::transmute(GetProcAddress(mod_gl, b"glClearDepth\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glClearIndex = core::mem::transmute(GetProcAddress(mod_gl, b"glClearIndex\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glClearStencil = core::mem::transmute(GetProcAddress(mod_gl, b"glClearStencil\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glClipPlane = core::mem::transmute(GetProcAddress(mod_gl, b"glClipPlane\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor3b = core::mem::transmute(GetProcAddress(mod_gl, b"glColor3b\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor3bv = core::mem::transmute(GetProcAddress(mod_gl, b"glColor3bv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor3d = core::mem::transmute(GetProcAddress(mod_gl, b"glColor3d\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor3dv = core::mem::transmute(GetProcAddress(mod_gl, b"glColor3dv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor3f = core::mem::transmute(GetProcAddress(mod_gl, b"glColor3f\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor3fv = core::mem::transmute(GetProcAddress(mod_gl, b"glColor3fv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor3i = core::mem::transmute(GetProcAddress(mod_gl, b"glColor3i\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor3iv = core::mem::transmute(GetProcAddress(mod_gl, b"glColor3iv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor3s = core::mem::transmute(GetProcAddress(mod_gl, b"glColor3s\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor3sv = core::mem::transmute(GetProcAddress(mod_gl, b"glColor3sv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor3ub = core::mem::transmute(GetProcAddress(mod_gl, b"glColor3ub\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor3ubv = core::mem::transmute(GetProcAddress(mod_gl, b"glColor3ubv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor3ui = core::mem::transmute(GetProcAddress(mod_gl, b"glColor3ui\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor3uiv = core::mem::transmute(GetProcAddress(mod_gl, b"glColor3uiv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor3us = core::mem::transmute(GetProcAddress(mod_gl, b"glColor3us\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor3usv = core::mem::transmute(GetProcAddress(mod_gl, b"glColor3usv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor4b = core::mem::transmute(GetProcAddress(mod_gl, b"glColor4b\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor4bv = core::mem::transmute(GetProcAddress(mod_gl, b"glColor4bv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor4d = core::mem::transmute(GetProcAddress(mod_gl, b"glColor4d\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor4dv = core::mem::transmute(GetProcAddress(mod_gl, b"glColor4dv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor4f = core::mem::transmute(GetProcAddress(mod_gl, b"glColor4f\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor4fv = core::mem::transmute(GetProcAddress(mod_gl, b"glColor4fv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor4i = core::mem::transmute(GetProcAddress(mod_gl, b"glColor4i\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor4iv = core::mem::transmute(GetProcAddress(mod_gl, b"glColor4iv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor4s = core::mem::transmute(GetProcAddress(mod_gl, b"glColor4s\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor4sv = core::mem::transmute(GetProcAddress(mod_gl, b"glColor4sv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor4ub = core::mem::transmute(GetProcAddress(mod_gl, b"glColor4ub\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor4ubv = core::mem::transmute(GetProcAddress(mod_gl, b"glColor4ubv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor4ui = core::mem::transmute(GetProcAddress(mod_gl, b"glColor4ui\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor4uiv = core::mem::transmute(GetProcAddress(mod_gl, b"glColor4uiv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor4us = core::mem::transmute(GetProcAddress(mod_gl, b"glColor4us\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColor4usv = core::mem::transmute(GetProcAddress(mod_gl, b"glColor4usv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColorMask = core::mem::transmute(GetProcAddress(mod_gl, b"glColorMask\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColorMaterial = core::mem::transmute(GetProcAddress(mod_gl, b"glColorMaterial\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glColorPointer = core::mem::transmute(GetProcAddress(mod_gl, b"glColorPointer\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glCopyPixels = core::mem::transmute(GetProcAddress(mod_gl, b"glCopyPixels\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glCopyTexImage1D = core::mem::transmute(GetProcAddress(mod_gl, b"glCopyTexImage1D\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glCopyTexImage2D = core::mem::transmute(GetProcAddress(mod_gl, b"glCopyTexImage2D\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glCopyTexSubImage1D = core::mem::transmute(GetProcAddress(mod_gl, b"glCopyTexSubImage1D\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glCopyTexSubImage2D = core::mem::transmute(GetProcAddress(mod_gl, b"glCopyTexSubImage2D\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glCullFace = core::mem::transmute(GetProcAddress(mod_gl, b"glCullFace\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glDeleteLists = core::mem::transmute(GetProcAddress(mod_gl, b"glDeleteLists\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glDeleteTextures = core::mem::transmute(GetProcAddress(mod_gl, b"glDeleteTextures\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glDepthFunc = core::mem::transmute(GetProcAddress(mod_gl, b"glDepthFunc\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glDepthMask = core::mem::transmute(GetProcAddress(mod_gl, b"glDepthMask\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glDepthRange = core::mem::transmute(GetProcAddress(mod_gl, b"glDepthRange\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glDisable = core::mem::transmute(GetProcAddress(mod_gl, b"glDisable\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glDisableClientState = core::mem::transmute(GetProcAddress(mod_gl, b"glDisableClientState\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glDrawArrays = core::mem::transmute(GetProcAddress(mod_gl, b"glDrawArrays\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glDrawBuffer = core::mem::transmute(GetProcAddress(mod_gl, b"glDrawBuffer\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glDrawElements = core::mem::transmute(GetProcAddress(mod_gl, b"glDrawElements\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glDrawPixels = core::mem::transmute(GetProcAddress(mod_gl, b"glDrawPixels\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glEdgeFlag = core::mem::transmute(GetProcAddress(mod_gl, b"glEdgeFlag\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glEdgeFlagPointer = core::mem::transmute(GetProcAddress(mod_gl, b"glEdgeFlagPointer\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glEdgeFlagv = core::mem::transmute(GetProcAddress(mod_gl, b"glEdgeFlagv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glEnable = core::mem::transmute(GetProcAddress(mod_gl, b"glEnable\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glEnableClientState = core::mem::transmute(GetProcAddress(mod_gl, b"glEnableClientState\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glEnd = core::mem::transmute(GetProcAddress(mod_gl, b"glEnd\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glEndList = core::mem::transmute(GetProcAddress(mod_gl, b"glEndList\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glEvalCoord1d = core::mem::transmute(GetProcAddress(mod_gl, b"glEvalCoord1d\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glEvalCoord1dv = core::mem::transmute(GetProcAddress(mod_gl, b"glEvalCoord1dv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glEvalCoord1f = core::mem::transmute(GetProcAddress(mod_gl, b"glEvalCoord1f\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glEvalCoord1fv = core::mem::transmute(GetProcAddress(mod_gl, b"glEvalCoord1fv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glEvalCoord2d = core::mem::transmute(GetProcAddress(mod_gl, b"glEvalCoord2d\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glEvalCoord2dv = core::mem::transmute(GetProcAddress(mod_gl, b"glEvalCoord2dv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glEvalCoord2f = core::mem::transmute(GetProcAddress(mod_gl, b"glEvalCoord2f\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glEvalCoord2fv = core::mem::transmute(GetProcAddress(mod_gl, b"glEvalCoord2fv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glEvalMesh1 = core::mem::transmute(GetProcAddress(mod_gl, b"glEvalMesh1\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glEvalMesh2 = core::mem::transmute(GetProcAddress(mod_gl, b"glEvalMesh2\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glEvalPoint1 = core::mem::transmute(GetProcAddress(mod_gl, b"glEvalPoint1\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glEvalPoint2 = core::mem::transmute(GetProcAddress(mod_gl, b"glEvalPoint2\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glFeedbackBuffer = core::mem::transmute(GetProcAddress(mod_gl, b"glFeedbackBuffer\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glFinish = core::mem::transmute(GetProcAddress(mod_gl, b"glFinish\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glFlush = core::mem::transmute(GetProcAddress(mod_gl, b"glFlush\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glFogf = core::mem::transmute(GetProcAddress(mod_gl, b"glFogf\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glFogfv = core::mem::transmute(GetProcAddress(mod_gl, b"glFogfv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glFogi = core::mem::transmute(GetProcAddress(mod_gl, b"glFogi\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glFogiv = core::mem::transmute(GetProcAddress(mod_gl, b"glFogiv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glFrontFace = core::mem::transmute(GetProcAddress(mod_gl, b"glFrontFace\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glFrustum = core::mem::transmute(GetProcAddress(mod_gl, b"glFrustum\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGenLists = core::mem::transmute(GetProcAddress(mod_gl, b"glGenLists\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGenTextures = core::mem::transmute(GetProcAddress(mod_gl, b"glGenTextures\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetBooleanv = core::mem::transmute(GetProcAddress(mod_gl, b"glGetBooleanv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetClipPlane = core::mem::transmute(GetProcAddress(mod_gl, b"glGetClipPlane\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetDoublev = core::mem::transmute(GetProcAddress(mod_gl, b"glGetDoublev\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetError = core::mem::transmute(GetProcAddress(mod_gl, b"glGetError\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetFloatv = core::mem::transmute(GetProcAddress(mod_gl, b"glGetFloatv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetIntegerv = core::mem::transmute(GetProcAddress(mod_gl, b"glGetIntegerv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetLightfv = core::mem::transmute(GetProcAddress(mod_gl, b"glGetLightfv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetLightiv = core::mem::transmute(GetProcAddress(mod_gl, b"glGetLightiv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetMapdv = core::mem::transmute(GetProcAddress(mod_gl, b"glGetMapdv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetMapfv = core::mem::transmute(GetProcAddress(mod_gl, b"glGetMapfv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetMapiv = core::mem::transmute(GetProcAddress(mod_gl, b"glGetMapiv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetMaterialfv = core::mem::transmute(GetProcAddress(mod_gl, b"glGetMaterialfv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetMaterialiv = core::mem::transmute(GetProcAddress(mod_gl, b"glGetMaterialiv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetPixelMapfv = core::mem::transmute(GetProcAddress(mod_gl, b"glGetPixelMapfv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetPixelMapuiv = core::mem::transmute(GetProcAddress(mod_gl, b"glGetPixelMapuiv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetPixelMapusv = core::mem::transmute(GetProcAddress(mod_gl, b"glGetPixelMapusv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetPointerv = core::mem::transmute(GetProcAddress(mod_gl, b"glGetPointerv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetPolygonStipple = core::mem::transmute(GetProcAddress(mod_gl, b"glGetPolygonStipple\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetString = core::mem::transmute(GetProcAddress(mod_gl, b"glGetString\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetTexEnvfv = core::mem::transmute(GetProcAddress(mod_gl, b"glGetTexEnvfv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetTexEnviv = core::mem::transmute(GetProcAddress(mod_gl, b"glGetTexEnviv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetTexGendv = core::mem::transmute(GetProcAddress(mod_gl, b"glGetTexGendv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetTexGenfv = core::mem::transmute(GetProcAddress(mod_gl, b"glGetTexGenfv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetTexGeniv = core::mem::transmute(GetProcAddress(mod_gl, b"glGetTexGeniv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetTexImage = core::mem::transmute(GetProcAddress(mod_gl, b"glGetTexImage\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetTexLevelParameterfv = core::mem::transmute(GetProcAddress(mod_gl, b"glGetTexLevelParameterfv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetTexLevelParameteriv = core::mem::transmute(GetProcAddress(mod_gl, b"glGetTexLevelParameteriv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetTexParameterfv = core::mem::transmute(GetProcAddress(mod_gl, b"glGetTexParameterfv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glGetTexParameteriv = core::mem::transmute(GetProcAddress(mod_gl, b"glGetTexParameteriv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glHint = core::mem::transmute(GetProcAddress(mod_gl, b"glHint\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glIndexMask = core::mem::transmute(GetProcAddress(mod_gl, b"glIndexMask\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glIndexPointer = core::mem::transmute(GetProcAddress(mod_gl, b"glIndexPointer\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glIndexd = core::mem::transmute(GetProcAddress(mod_gl, b"glIndexd\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glIndexdv = core::mem::transmute(GetProcAddress(mod_gl, b"glIndexdv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glIndexf = core::mem::transmute(GetProcAddress(mod_gl, b"glIndexf\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glIndexfv = core::mem::transmute(GetProcAddress(mod_gl, b"glIndexfv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glIndexi = core::mem::transmute(GetProcAddress(mod_gl, b"glIndexi\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glIndexiv = core::mem::transmute(GetProcAddress(mod_gl, b"glIndexiv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glIndexs = core::mem::transmute(GetProcAddress(mod_gl, b"glIndexs\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glIndexsv = core::mem::transmute(GetProcAddress(mod_gl, b"glIndexsv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glIndexub = core::mem::transmute(GetProcAddress(mod_gl, b"glIndexub\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glIndexubv = core::mem::transmute(GetProcAddress(mod_gl, b"glIndexubv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glInitNames = core::mem::transmute(GetProcAddress(mod_gl, b"glInitNames\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glInterleavedArrays = core::mem::transmute(GetProcAddress(mod_gl, b"glInterleavedArrays\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glIsEnabled = core::mem::transmute(GetProcAddress(mod_gl, b"glIsEnabled\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glIsList = core::mem::transmute(GetProcAddress(mod_gl, b"glIsList\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glIsTexture = core::mem::transmute(GetProcAddress(mod_gl, b"glIsTexture\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glLightModelf = core::mem::transmute(GetProcAddress(mod_gl, b"glLightModelf\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glLightModelfv = core::mem::transmute(GetProcAddress(mod_gl, b"glLightModelfv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glLightModeli = core::mem::transmute(GetProcAddress(mod_gl, b"glLightModeli\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glLightModeliv = core::mem::transmute(GetProcAddress(mod_gl, b"glLightModeliv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glLightf = core::mem::transmute(GetProcAddress(mod_gl, b"glLightf\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glLightfv = core::mem::transmute(GetProcAddress(mod_gl, b"glLightfv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glLighti = core::mem::transmute(GetProcAddress(mod_gl, b"glLighti\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glLightiv = core::mem::transmute(GetProcAddress(mod_gl, b"glLightiv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glLineStipple = core::mem::transmute(GetProcAddress(mod_gl, b"glLineStipple\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glLineWidth = core::mem::transmute(GetProcAddress(mod_gl, b"glLineWidth\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glListBase = core::mem::transmute(GetProcAddress(mod_gl, b"glListBase\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glLoadIdentity = core::mem::transmute(GetProcAddress(mod_gl, b"glLoadIdentity\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glLoadMatrixd = core::mem::transmute(GetProcAddress(mod_gl, b"glLoadMatrixd\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glLoadMatrixf = core::mem::transmute(GetProcAddress(mod_gl, b"glLoadMatrixf\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glLoadName = core::mem::transmute(GetProcAddress(mod_gl, b"glLoadName\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glLogicOp = core::mem::transmute(GetProcAddress(mod_gl, b"glLogicOp\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glMap1d = core::mem::transmute(GetProcAddress(mod_gl, b"glMap1d\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glMap1f = core::mem::transmute(GetProcAddress(mod_gl, b"glMap1f\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glMap2d = core::mem::transmute(GetProcAddress(mod_gl, b"glMap2d\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glMap2f = core::mem::transmute(GetProcAddress(mod_gl, b"glMap2f\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glMapGrid1d = core::mem::transmute(GetProcAddress(mod_gl, b"glMapGrid1d\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glMapGrid1f = core::mem::transmute(GetProcAddress(mod_gl, b"glMapGrid1f\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glMapGrid2d = core::mem::transmute(GetProcAddress(mod_gl, b"glMapGrid2d\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glMapGrid2f = core::mem::transmute(GetProcAddress(mod_gl, b"glMapGrid2f\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glMaterialf = core::mem::transmute(GetProcAddress(mod_gl, b"glMaterialf\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glMaterialfv = core::mem::transmute(GetProcAddress(mod_gl, b"glMaterialfv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glMateriali = core::mem::transmute(GetProcAddress(mod_gl, b"glMateriali\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glMaterialiv = core::mem::transmute(GetProcAddress(mod_gl, b"glMaterialiv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glMatrixMode = core::mem::transmute(GetProcAddress(mod_gl, b"glMatrixMode\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glMultMatrixd = core::mem::transmute(GetProcAddress(mod_gl, b"glMultMatrixd\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glMultMatrixf = core::mem::transmute(GetProcAddress(mod_gl, b"glMultMatrixf\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glNewList = core::mem::transmute(GetProcAddress(mod_gl, b"glNewList\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glNormal3b = core::mem::transmute(GetProcAddress(mod_gl, b"glNormal3b\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glNormal3bv = core::mem::transmute(GetProcAddress(mod_gl, b"glNormal3bv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glNormal3d = core::mem::transmute(GetProcAddress(mod_gl, b"glNormal3d\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glNormal3dv = core::mem::transmute(GetProcAddress(mod_gl, b"glNormal3dv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glNormal3f = core::mem::transmute(GetProcAddress(mod_gl, b"glNormal3f\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glNormal3fv = core::mem::transmute(GetProcAddress(mod_gl, b"glNormal3fv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glNormal3i = core::mem::transmute(GetProcAddress(mod_gl, b"glNormal3i\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glNormal3iv = core::mem::transmute(GetProcAddress(mod_gl, b"glNormal3iv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glNormal3s = core::mem::transmute(GetProcAddress(mod_gl, b"glNormal3s\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glNormal3sv = core::mem::transmute(GetProcAddress(mod_gl, b"glNormal3sv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glNormalPointer = core::mem::transmute(GetProcAddress(mod_gl, b"glNormalPointer\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glOrtho = core::mem::transmute(GetProcAddress(mod_gl, b"glOrtho\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glPassThrough = core::mem::transmute(GetProcAddress(mod_gl, b"glPassThrough\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glPixelMapfv = core::mem::transmute(GetProcAddress(mod_gl, b"glPixelMapfv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glPixelMapuiv = core::mem::transmute(GetProcAddress(mod_gl, b"glPixelMapuiv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glPixelMapusv = core::mem::transmute(GetProcAddress(mod_gl, b"glPixelMapusv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glPixelStoref = core::mem::transmute(GetProcAddress(mod_gl, b"glPixelStoref\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glPixelStorei = core::mem::transmute(GetProcAddress(mod_gl, b"glPixelStorei\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glPixelTransferf = core::mem::transmute(GetProcAddress(mod_gl, b"glPixelTransferf\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glPixelTransferi = core::mem::transmute(GetProcAddress(mod_gl, b"glPixelTransferi\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glPixelZoom = core::mem::transmute(GetProcAddress(mod_gl, b"glPixelZoom\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glPointSize = core::mem::transmute(GetProcAddress(mod_gl, b"glPointSize\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glPolygonMode = core::mem::transmute(GetProcAddress(mod_gl, b"glPolygonMode\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glPolygonOffset = core::mem::transmute(GetProcAddress(mod_gl, b"glPolygonOffset\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glPolygonStipple = core::mem::transmute(GetProcAddress(mod_gl, b"glPolygonStipple\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glPopAttrib = core::mem::transmute(GetProcAddress(mod_gl, b"glPopAttrib\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glPopClientAttrib = core::mem::transmute(GetProcAddress(mod_gl, b"glPopClientAttrib\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glPopMatrix = core::mem::transmute(GetProcAddress(mod_gl, b"glPopMatrix\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glPopName = core::mem::transmute(GetProcAddress(mod_gl, b"glPopName\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glPrioritizeTextures = core::mem::transmute(GetProcAddress(mod_gl, b"glPrioritizeTextures\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glPushAttrib = core::mem::transmute(GetProcAddress(mod_gl, b"glPushAttrib\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glPushClientAttrib = core::mem::transmute(GetProcAddress(mod_gl, b"glPushClientAttrib\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glPushMatrix = core::mem::transmute(GetProcAddress(mod_gl, b"glPushMatrix\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glPushName = core::mem::transmute(GetProcAddress(mod_gl, b"glPushName\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos2d = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos2d\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos2dv = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos2dv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos2f = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos2f\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos2fv = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos2fv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos2i = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos2i\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos2iv = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos2iv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos2s = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos2s\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos2sv = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos2sv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos3d = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos3d\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos3dv = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos3dv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos3f = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos3f\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos3fv = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos3fv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos3i = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos3i\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos3iv = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos3iv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos3s = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos3s\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos3sv = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos3sv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos4d = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos4d\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos4dv = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos4dv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos4f = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos4f\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos4fv = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos4fv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos4i = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos4i\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos4iv = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos4iv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos4s = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos4s\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRasterPos4sv = core::mem::transmute(GetProcAddress(mod_gl, b"glRasterPos4sv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glReadBuffer = core::mem::transmute(GetProcAddress(mod_gl, b"glReadBuffer\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glReadPixels = core::mem::transmute(GetProcAddress(mod_gl, b"glReadPixels\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRectd = core::mem::transmute(GetProcAddress(mod_gl, b"glRectd\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRectdv = core::mem::transmute(GetProcAddress(mod_gl, b"glRectdv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRectf = core::mem::transmute(GetProcAddress(mod_gl, b"glRectf\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRectfv = core::mem::transmute(GetProcAddress(mod_gl, b"glRectfv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRecti = core::mem::transmute(GetProcAddress(mod_gl, b"glRecti\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRectiv = core::mem::transmute(GetProcAddress(mod_gl, b"glRectiv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRects = core::mem::transmute(GetProcAddress(mod_gl, b"glRects\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRectsv = core::mem::transmute(GetProcAddress(mod_gl, b"glRectsv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRenderMode = core::mem::transmute(GetProcAddress(mod_gl, b"glRenderMode\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRotated = core::mem::transmute(GetProcAddress(mod_gl, b"glRotated\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glRotatef = core::mem::transmute(GetProcAddress(mod_gl, b"glRotatef\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glScaled = core::mem::transmute(GetProcAddress(mod_gl, b"glScaled\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glScalef = core::mem::transmute(GetProcAddress(mod_gl, b"glScalef\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glScissor = core::mem::transmute(GetProcAddress(mod_gl, b"glScissor\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glSelectBuffer = core::mem::transmute(GetProcAddress(mod_gl, b"glSelectBuffer\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glShadeModel = core::mem::transmute(GetProcAddress(mod_gl, b"glShadeModel\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glStencilFunc = core::mem::transmute(GetProcAddress(mod_gl, b"glStencilFunc\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glStencilMask = core::mem::transmute(GetProcAddress(mod_gl, b"glStencilMask\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glStencilOp = core::mem::transmute(GetProcAddress(mod_gl, b"glStencilOp\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord1d = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord1d\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord1dv = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord1dv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord1f = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord1f\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord1fv = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord1fv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord1i = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord1i\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord1iv = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord1iv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord1s = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord1s\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord1sv = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord1sv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord2d = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord2d\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord2dv = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord2dv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord2f = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord2f\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord2fv = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord2fv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord2i = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord2i\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord2iv = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord2iv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord2s = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord2s\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord2sv = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord2sv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord3d = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord3d\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord3dv = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord3dv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord3f = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord3f\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord3fv = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord3fv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord3i = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord3i\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord3iv = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord3iv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord3s = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord3s\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord3sv = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord3sv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord4d = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord4d\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord4dv = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord4dv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord4f = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord4f\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord4fv = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord4fv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord4i = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord4i\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord4iv = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord4iv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord4s = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord4s\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoord4sv = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoord4sv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexCoordPointer = core::mem::transmute(GetProcAddress(mod_gl, b"glTexCoordPointer\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexEnvf = core::mem::transmute(GetProcAddress(mod_gl, b"glTexEnvf\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexEnvfv = core::mem::transmute(GetProcAddress(mod_gl, b"glTexEnvfv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexEnvi = core::mem::transmute(GetProcAddress(mod_gl, b"glTexEnvi\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexEnviv = core::mem::transmute(GetProcAddress(mod_gl, b"glTexEnviv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexGend = core::mem::transmute(GetProcAddress(mod_gl, b"glTexGend\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexGendv = core::mem::transmute(GetProcAddress(mod_gl, b"glTexGendv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexGenf = core::mem::transmute(GetProcAddress(mod_gl, b"glTexGenf\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexGenfv = core::mem::transmute(GetProcAddress(mod_gl, b"glTexGenfv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexGeni = core::mem::transmute(GetProcAddress(mod_gl, b"glTexGeni\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexGeniv = core::mem::transmute(GetProcAddress(mod_gl, b"glTexGeniv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexImage1D = core::mem::transmute(GetProcAddress(mod_gl, b"glTexImage1D\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexImage2D = core::mem::transmute(GetProcAddress(mod_gl, b"glTexImage2D\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexParameterf = core::mem::transmute(GetProcAddress(mod_gl, b"glTexParameterf\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexParameterfv = core::mem::transmute(GetProcAddress(mod_gl, b"glTexParameterfv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexParameteri = core::mem::transmute(GetProcAddress(mod_gl, b"glTexParameteri\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexParameteriv = core::mem::transmute(GetProcAddress(mod_gl, b"glTexParameteriv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexSubImage1D = core::mem::transmute(GetProcAddress(mod_gl, b"glTexSubImage1D\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTexSubImage2D = core::mem::transmute(GetProcAddress(mod_gl, b"glTexSubImage2D\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTranslated = core::mem::transmute(GetProcAddress(mod_gl, b"glTranslated\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glTranslatef = core::mem::transmute(GetProcAddress(mod_gl, b"glTranslatef\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex2d = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex2d\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex2dv = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex2dv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex2f = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex2f\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex2fv = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex2fv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex2i = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex2i\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex2iv = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex2iv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex2s = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex2s\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex2sv = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex2sv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex3d = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex3d\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex3dv = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex3dv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex3f = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex3f\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex3fv = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex3fv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex3i = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex3i\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex3iv = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex3iv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex3s = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex3s\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex3sv = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex3sv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex4d = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex4d\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex4dv = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex4dv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex4f = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex4f\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex4fv = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex4fv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex4i = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex4i\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex4iv = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex4iv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex4s = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex4s\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertex4sv = core::mem::transmute(GetProcAddress(mod_gl, b"glVertex4sv\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glVertexPointer = core::mem::transmute(GetProcAddress(mod_gl, b"glVertexPointer\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.gl.p_glViewport = core::mem::transmute(GetProcAddress(mod_gl, b"glViewport\0".as_ptr() as *const i8));
        (*gl_info).gl_ops.wgl.p_wglSwapBuffers = core::mem::transmute(
            GetProcAddress(mod_gl, b"wglSwapBuffers\0".as_ptr() as *const i8),
        );
        (*gl_info).gl_ops.wgl.p_wglGetPixelFormat = core::mem::transmute(
            GetProcAddress(mod_gl, b"wglGetPixelFormat\0".as_ptr() as *const i8),
        );
    }

    (*gl_info).p_glEnableWINE = (*gl_info).gl_ops.gl.p_glEnable;
    (*gl_info).p_glDisableWINE = (*gl_info).gl_ops.gl.p_glDisable;

    if wined3d_caps_gl_ctx_create(adapter_gl, core::ptr::addr_of_mut!(caps_gl_ctx)) == 0 {
        ERR!("Failed to get a GL context for adapter %p.\n", adapter_gl);
        return (FALSE) as BOOL;
    }

    i = 0;
    while (i as usize) < ARRAY_SIZE!(supported_gl_versions) as usize {
        if supported_gl_versions[i as usize] <= wined3d_settings.max_gl_version {
            break;
        }
        i += 1;
    }
    if (i as usize) == ARRAY_SIZE!(supported_gl_versions) as usize {
        WINE_ERR_!(__wine_dbch_winediag,
            "Requested invalid GL version %u.%u.\n",
            wined3d_settings.max_gl_version >> 16,
            wined3d_settings.max_gl_version & 0xffff
        );
        i = (ARRAY_SIZE!(supported_gl_versions) as u32).wrapping_sub(1 as u32);
    }

    while (i as usize) < ARRAY_SIZE!(supported_gl_versions) as usize {
        (*gl_info).selected_gl_version = supported_gl_versions[i as usize];

        if wined3d_caps_gl_ctx_create_attribs(core::ptr::addr_of_mut!(caps_gl_ctx), gl_info) != 0 {
            break;
        }

        WARN!(
            "Couldn't create an OpenGL %u.%u context, trying fallback to a lower version.\n",
            supported_gl_versions[i as usize] >> 16,
            supported_gl_versions[i as usize] & 0xffff
        );
        i += 1;
    }

    if wined3d_adapter_init_gl_caps(adapter_gl, core::ptr::addr_of_mut!(caps_gl_ctx), wined3d_creation_flags) == 0 {
        ERR!("Failed to initialize GL caps for adapter %p.\n", adapter_gl);
        wined3d_caps_gl_ctx_destroy(core::ptr::addr_of_mut!(caps_gl_ctx));
        return (FALSE) as BOOL;
    }

    let mut i_0: u32 = 0;
    while (i_0 as usize) < ARRAY_SIZE!(required_extensions) as usize {
        if (*gl_info).supported[required_extensions[i_0 as usize].0 as usize] == 0 {
            ERR!(
                "Required extension %s is not supported.\n",
                required_extensions[i_0 as usize].1
            );
            wined3d_warn_no_3d();
            wined3d_caps_gl_ctx_destroy(core::ptr::addr_of_mut!(caps_gl_ctx));
            return (FALSE) as BOOL;
        }
        i_0 += 1;
    }

    if (*gl_info).supported[ARB_TEXTURE_NON_POWER_OF_TWO as usize] == 0
        && (*gl_info).supported[WINED3D_GL_NORMALIZED_TEXRECT as usize] == 0
    {
        ERR!("Required extension ARB_texture_non_power_of_two is not supported.\n");
        wined3d_warn_no_3d();
        wined3d_caps_gl_ctx_destroy(core::ptr::addr_of_mut!(caps_gl_ctx));
        return (FALSE) as BOOL;
    }

    if (*gl_info).glsl_version < MAKEDWORD_VERSION!(1, 20) {
        ERR!(
            "GLSL version %s is too low; 1.20 is required.\n",
            (*gl_info)
                .gl_ops
                .gl
                .p_glGetString
                .unwrap_unchecked()(GL_SHADING_LANGUAGE_VERSION_ARB as GLenum)
                as *const i8
        );
        wined3d_warn_no_3d();
        wined3d_caps_gl_ctx_destroy(core::ptr::addr_of_mut!(caps_gl_ctx));
        return (FALSE) as BOOL;
    }

    (*gl_info).filling_convention_offset =
        wined3d_adapter_find_fill_offset(core::ptr::addr_of_mut!(caps_gl_ctx));

    wined3d_adapter_gl_init_d3d_info(adapter_gl, wined3d_creation_flags);

    if wined3d_driver_info_init(
        driver_info,
        caps_gl_ctx.gpu_description,
        (*adapter_gl).a.d3d_info.feature_level,
        caps_gl_ctx.vram_bytes,
        0,
    ) == false
    {
        wined3d_caps_gl_ctx_destroy(core::ptr::addr_of_mut!(caps_gl_ctx));
        return (FALSE) as BOOL;
    }

    wined3d_adapter_init_fb_cfgs(adapter_gl, caps_gl_ctx.dc);
    /* We haven't found any suitable formats. This should only happen in
     * case of GDI software rendering, which is pretty useless anyway. */
    if (*adapter_gl).pixel_format_count == 0 {
        WARN!("No suitable pixel formats found.\n");
        wined3d_caps_gl_ctx_destroy(core::ptr::addr_of_mut!(caps_gl_ctx));
        free((*adapter_gl).pixel_formats as *mut core::ffi::c_void);
        return (FALSE) as BOOL;
    }

    if wined3d_adapter_gl_init_format_info(
        core::ptr::addr_of_mut!((*adapter_gl).a),
        core::ptr::addr_of_mut!(caps_gl_ctx),
    ) == 0
    {
        ERR!("Failed to initialize GL format info.\n");
        wined3d_caps_gl_ctx_destroy(core::ptr::addr_of_mut!(caps_gl_ctx));
        free((*adapter_gl).pixel_formats as *mut core::ffi::c_void);
        return (FALSE) as BOOL;
    }

    wined3d_caps_gl_ctx_destroy(core::ptr::addr_of_mut!(caps_gl_ctx));

    wined3d_adapter_init_ffp_attrib_ops(adapter_gl);

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_adapter_gl_create(mut ordinal: ::core::ffi::c_uint, mut wined3d_creation_flags: ::core::ffi::c_uint) -> *mut wined3d_adapter {
    let mut adapter: *mut wined3d_adapter_gl = core::ptr::null_mut();

    adapter = calloc(1, (core::mem::size_of::<wined3d_adapter_gl>()) as _) as *mut wined3d_adapter_gl;
    if adapter.is_null() {
        return core::ptr::null_mut();
    }

    if wined3d_adapter_gl_init(adapter, ordinal, wined3d_creation_flags) == 0 {
        if (*adapter).a.kmt_adapter != 0 {
            wined3d_adapter_cleanup(core::ptr::addr_of_mut!((*adapter).a));
        }
        free(adapter as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    WINE_TRACE!("Created adapter %p.\n", adapter);

    return core::ptr::addr_of_mut!((*adapter).a);
}
