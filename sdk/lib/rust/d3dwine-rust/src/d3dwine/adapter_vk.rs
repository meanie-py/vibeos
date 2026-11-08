use crate::d3dwine::directx::adapter_adjust_mapped_memory;
use crate::d3dwine::utils::debug_uvec4;
use crate::d3dwine::device::device_context_add;
use crate::d3dwine::device::device_context_remove;
use crate::d3dwine::ffp_gl::state_nop;
use crate::d3dwine::buffer::vk_buffer_usage_from_bind_flags;
use crate::d3dwine::buffer::vk_memory_type_from_access_flags;
use crate::d3dwine::directx::wined3d_adapter_cleanup;
use crate::d3dwine::directx::wined3d_adapter_init;
use crate::d3dwine::utils::wined3d_adapter_vk_init_format_info;
use crate::d3dwine::utils::wined3d_allocator_chunk_cleanup;
use crate::d3dwine::utils::wined3d_allocator_chunk_init;
use crate::d3dwine::context_vk::wined3d_allocator_chunk_vk_map;
use crate::d3dwine::context_vk::wined3d_allocator_chunk_vk_unmap;
use crate::d3dwine::utils::wined3d_allocator_cleanup;
use crate::d3dwine::utils::wined3d_allocator_init;
use crate::d3dwine::utils::wined3d_array_reserve;
use crate::d3dwine::buffer::wined3d_buffer_cleanup;
use crate::d3dwine::buffer::wined3d_buffer_vk_init;
use crate::d3dwine::context_vk::wined3d_context_vk_allocate_vram_chunk_memory;
use crate::d3dwine::context_vk::wined3d_context_vk_apply_compute_state;
use crate::d3dwine::context_vk::wined3d_context_vk_apply_draw_state;
use crate::d3dwine::context_vk::wined3d_context_vk_cleanup;
use crate::d3dwine::context_vk::wined3d_context_vk_create_bo;
use crate::d3dwine::context_vk::wined3d_context_vk_destroy_bo;
use crate::d3dwine::context_vk::wined3d_context_vk_destroy_vk_buffer_view;
use crate::d3dwine::context_vk::wined3d_context_vk_destroy_vk_image_view;
use crate::d3dwine::context_vk::wined3d_context_vk_destroy_vk_sampler;
use crate::d3dwine::context_vk::wined3d_context_vk_end_current_render_pass;
use crate::d3dwine::context_vk::wined3d_context_vk_get_command_buffer;
use crate::d3dwine::context_vk::wined3d_context_vk_init;
use crate::d3dwine::context_vk::wined3d_context_vk_submit_command_buffer;
use crate::d3dwine::context_vk::wined3d_context_vk_wait_command_buffer;
use crate::d3dwine::surface::wined3d_cpu_blitter_create;
use crate::d3dwine::cs::wined3d_cs_destroy_object;
use crate::d3dwine::utils::wined3d_debug_vkresult;
use crate::d3dwine::device::wined3d_device_cleanup;
use crate::d3dwine::device::wined3d_device_create_default_samplers;
use crate::d3dwine::device::wined3d_device_decref;
use crate::d3dwine::device::wined3d_device_destroy_default_samplers;
use crate::d3dwine::device::wined3d_device_incref;
use crate::d3dwine::device::wined3d_device_init;
use crate::d3dwine::device::wined3d_device_vk_create_null_resources;
use crate::d3dwine::device::wined3d_device_vk_create_null_views;
use crate::d3dwine::device::wined3d_device_vk_destroy_null_resources;
use crate::d3dwine::device::wined3d_device_vk_destroy_null_views;
use crate::d3dwine::view::wined3d_device_vk_uav_clear_state_cleanup;
use crate::d3dwine::view::wined3d_device_vk_uav_clear_state_init;
use crate::d3dwine::directx::wined3d_driver_info_init;
use crate::d3dwine::wined3d_main::wined3d_get_app_name;
use crate::d3dwine::directx::wined3d_get_gpu_description;
use crate::d3dwine::utils::wined3d_get_primary_adapter_luid;
use crate::d3dwine::directx::wined3d_get_user_override_gpu_description;
use crate::d3dwine::query::wined3d_query_vk_create;
use crate::d3dwine::view::wined3d_rendertarget_view_cleanup;
use crate::d3dwine::view::wined3d_rendertarget_view_vk_init;
use crate::d3dwine::sampler::wined3d_sampler_vk_init;
use crate::d3dwine::view::wined3d_shader_resource_view_cleanup;
use crate::d3dwine::view::wined3d_shader_resource_view_vk_generate_mipmap;
use crate::d3dwine::view::wined3d_shader_resource_view_vk_init;
use crate::d3dwine::shader_spirv::wined3d_spirv_fragment_pipe_init_vk;
use crate::d3dwine::shader_spirv::wined3d_spirv_shader_backend_init_vk;
use crate::d3dwine::shader_spirv::wined3d_spirv_vertex_pipe_init_vk;
use crate::d3dwine::swapchain::wined3d_swapchain_vk_cleanup;
use crate::d3dwine::swapchain::wined3d_swapchain_vk_init;
use crate::d3dwine::texture::wined3d_texture_cleanup;
use crate::d3dwine::texture::wined3d_texture_sub_resources_destroyed;
use crate::d3dwine::texture::wined3d_texture_vk_init;
use crate::d3dwine::view::wined3d_unordered_access_view_cleanup;
use crate::d3dwine::view::wined3d_unordered_access_view_vk_clear;
use crate::d3dwine::view::wined3d_unordered_access_view_vk_init;
use crate::d3dwine::texture::wined3d_vk_blitter_create;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_80073828D8A0: [u16; 10] = [0x006Eu16, 0x0074, 0x0064, 0x006C, 0x006C, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_55 {
    pub name: *const ::core::ffi::c_char,
    pub core_since_version: ::core::ffi::c_uint,
    pub required: BOOL,
}

pub static VK_KHR_SURFACE_EXTENSION_NAME: [::core::ffi::c_char; 15] =
    unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"VK_KHR_surface\0") };

pub const VK_KHR_SWAPCHAIN_EXTENSION_NAME: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"VK_KHR_swapchain\0")
};

pub static VK_KHR_WIN32_SURFACE_EXTENSION_NAME: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"VK_KHR_win32_surface\0")
};

pub const VK_KHR_SAMPLER_MIRROR_CLAMP_TO_EDGE_EXTENSION_NAME: [::core::ffi::c_char; 36] = unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
        *b"VK_KHR_sampler_mirror_clamp_to_edge\0",
    )
};

pub const VK_EXT_TRANSFORM_FEEDBACK_EXTENSION_NAME: [::core::ffi::c_char; 26] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"VK_EXT_transform_feedback\0")
};

pub static VK_KHR_GET_PHYSICAL_DEVICE_PROPERTIES_2_EXTENSION_NAME: [::core::ffi::c_char; 39] = unsafe {
    ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
        *b"VK_KHR_get_physical_device_properties2\0",
    )
};

pub const VK_KHR_SHADER_DRAW_PARAMETERS_EXTENSION_NAME: [::core::ffi::c_char; 30] = unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
        *b"VK_KHR_shader_draw_parameters\0",
    )
};


pub static VK_KHR_EXTERNAL_MEMORY_EXTENSION_NAME: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"VK_KHR_external_memory\0")
};


pub const VK_EXT_SHADER_STENCIL_EXPORT_EXTENSION_NAME: [::core::ffi::c_char; 29] = unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"VK_EXT_shader_stencil_export\0",
    )
};

pub const VK_EXT_VERTEX_ATTRIBUTE_DIVISOR_EXTENSION_NAME: [::core::ffi::c_char; 32] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
        *b"VK_EXT_vertex_attribute_divisor\0",
    )
};

pub const VK_EXT_HOST_QUERY_RESET_EXTENSION_NAME: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"VK_EXT_host_query_reset\0")
};

pub const VK_EXT_EXTENDED_DYNAMIC_STATE_EXTENSION_NAME: [::core::ffi::c_char; 30] = unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
        *b"VK_EXT_extended_dynamic_state\0",
    )
};

pub const VK_EXT_EXTENDED_DYNAMIC_STATE_2_EXTENSION_NAME: [::core::ffi::c_char; 31] = unsafe {
    ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
        *b"VK_EXT_extended_dynamic_state2\0",
    )
};

pub const VK_EXT_EXTENDED_DYNAMIC_STATE_3_EXTENSION_NAME: [::core::ffi::c_char; 31] = unsafe {
    ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
        *b"VK_EXT_extended_dynamic_state3\0",
    )
};

static mut misc_state_template_vk: [wined3d_state_entry_template; 34] = [
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_VERTEX as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_VERTEX as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_VERTEX as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_VERTEX as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int)
                as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_LINEPATTERN as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_LINEPATTERN as ::core::ffi::c_int as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_DITHERENABLE as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_DITHERENABLE as ::core::ffi::c_int
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_MULTISAMPLEANTIALIAS as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_MULTISAMPLEANTIALIAS as ::core::ffi::c_int
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: 0 as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: 0,
                apply: None,
            },
            extension: 0,
        },
    ];

static mut wined3d_allocator_vk_ops: wined3d_allocator_ops = unsafe {
    wined3d_allocator_ops {
        allocator_create_chunk: Some(
            wined3d_allocator_vk_create_chunk
                as unsafe extern "C" fn(
                    *mut wined3d_allocator,
                    *mut wined3d_context,
                    ::core::ffi::c_uint,
                    size_t,
                ) -> *mut wined3d_allocator_chunk,
        ),
        allocator_destroy_chunk: Some(
            wined3d_allocator_vk_destroy_chunk
                as unsafe extern "C" fn(*mut wined3d_allocator_chunk) -> (),
        ),
    }
};

static mut wined3d_adapter_vk_ops: wined3d_adapter_ops = unsafe {
    wined3d_adapter_ops {
        adapter_destroy: Some(
            adapter_vk_destroy as unsafe extern "C" fn(*mut wined3d_adapter) -> (),
        ),
        adapter_create_device: Some(
            adapter_vk_create_device
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
            adapter_vk_destroy_device as unsafe extern "C" fn(*mut wined3d_device) -> (),
        ),
        adapter_acquire_context: Some(
            adapter_vk_acquire_context
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                ) -> *mut wined3d_context,
        ),
        adapter_release_context: Some(
            adapter_vk_release_context as unsafe extern "C" fn(*mut wined3d_context) -> (),
        ),
        adapter_get_wined3d_caps: Some(
            adapter_vk_get_wined3d_caps
                as unsafe extern "C" fn(*const wined3d_adapter, *mut wined3d_caps) -> (),
        ),
        adapter_check_format: Some(
            adapter_vk_check_format
                as unsafe extern "C" fn(
                    *const wined3d_adapter,
                    *const wined3d_format,
                    *const wined3d_format,
                    *const wined3d_format,
                ) -> BOOL,
        ),
        adapter_init_3d: Some(
            adapter_vk_init_3d as unsafe extern "C" fn(*mut wined3d_device) -> HRESULT,
        ),
        adapter_uninit_3d: Some(
            adapter_vk_uninit_3d as unsafe extern "C" fn(*mut wined3d_device) -> (),
        ),
        adapter_map_bo_address: Some(
            adapter_vk_map_bo_address
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *const wined3d_bo_address,
                    size_t,
                    uint32_t,
                ) -> *mut ::core::ffi::c_void,
        ),
        adapter_unmap_bo_address: Some(
            adapter_vk_unmap_bo_address
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *const wined3d_bo_address,
                    ::core::ffi::c_uint,
                    *const wined3d_range,
                ) -> (),
        ),
        adapter_copy_bo_address: Some(
            adapter_vk_copy_bo_address
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
            adapter_vk_flush_bo_address
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *const wined3d_const_bo_address,
                    size_t,
                ) -> (),
        ),
        adapter_alloc_bo: Some(
            adapter_vk_alloc_bo
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    *mut wined3d_resource,
                    ::core::ffi::c_uint,
                    *mut wined3d_bo_address,
                ) -> bool,
        ),
        adapter_destroy_bo: Some(
            adapter_vk_destroy_bo
                as unsafe extern "C" fn(*mut wined3d_context, *mut wined3d_bo) -> (),
        ),
        adapter_create_swapchain: Some(
            adapter_vk_create_swapchain
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
            adapter_vk_destroy_swapchain as unsafe extern "C" fn(*mut wined3d_swapchain) -> (),
        ),
        adapter_create_buffer: Some(
            adapter_vk_create_buffer
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
            adapter_vk_destroy_buffer as unsafe extern "C" fn(*mut wined3d_buffer) -> (),
        ),
        adapter_create_texture: Some(
            adapter_vk_create_texture
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
            adapter_vk_destroy_texture as unsafe extern "C" fn(*mut wined3d_texture) -> (),
        ),
        adapter_create_rendertarget_view: Some(
            adapter_vk_create_rendertarget_view
                as unsafe extern "C" fn(
                    *const wined3d_view_desc,
                    *mut wined3d_resource,
                    *mut ::core::ffi::c_void,
                    *const wined3d_parent_ops,
                    *mut *mut wined3d_rendertarget_view,
                ) -> HRESULT,
        ),
        adapter_destroy_rendertarget_view: Some(
            adapter_vk_destroy_rendertarget_view
                as unsafe extern "C" fn(*mut wined3d_rendertarget_view) -> (),
        ),
        adapter_create_shader_resource_view: Some(
            adapter_vk_create_shader_resource_view
                as unsafe extern "C" fn(
                    *const wined3d_view_desc,
                    *mut wined3d_resource,
                    *mut ::core::ffi::c_void,
                    *const wined3d_parent_ops,
                    *mut *mut wined3d_shader_resource_view,
                ) -> HRESULT,
        ),
        adapter_destroy_shader_resource_view: Some(
            adapter_vk_destroy_shader_resource_view
                as unsafe extern "C" fn(*mut wined3d_shader_resource_view) -> (),
        ),
        adapter_create_unordered_access_view: Some(
            adapter_vk_create_unordered_access_view
                as unsafe extern "C" fn(
                    *const wined3d_view_desc,
                    *mut wined3d_resource,
                    *mut ::core::ffi::c_void,
                    *const wined3d_parent_ops,
                    *mut *mut wined3d_unordered_access_view,
                ) -> HRESULT,
        ),
        adapter_destroy_unordered_access_view: Some(
            adapter_vk_destroy_unordered_access_view
                as unsafe extern "C" fn(*mut wined3d_unordered_access_view) -> (),
        ),
        adapter_create_sampler: Some(
            adapter_vk_create_sampler
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    *const wined3d_sampler_desc,
                    *mut ::core::ffi::c_void,
                    *const wined3d_parent_ops,
                    *mut *mut wined3d_sampler,
                ) -> HRESULT,
        ),
        adapter_destroy_sampler: Some(
            adapter_vk_destroy_sampler as unsafe extern "C" fn(*mut wined3d_sampler) -> (),
        ),
        adapter_create_query: Some(
            adapter_vk_create_query
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    wined3d_query_type,
                    *mut ::core::ffi::c_void,
                    *const wined3d_parent_ops,
                    *mut *mut wined3d_query,
                ) -> HRESULT,
        ),
        adapter_destroy_query: Some(
            adapter_vk_destroy_query as unsafe extern "C" fn(*mut wined3d_query) -> (),
        ),
        adapter_flush_context: Some(
            adapter_vk_flush_context as unsafe extern "C" fn(*mut wined3d_context) -> (),
        ),
        adapter_draw_primitive: Some(
            adapter_vk_draw_primitive
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    *const wined3d_state,
                    *const wined3d_draw_parameters,
                ) -> (),
        ),
        adapter_dispatch_compute: Some(
            adapter_vk_dispatch_compute
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    *const wined3d_state,
                    *const wined3d_dispatch_parameters,
                ) -> (),
        ),
        adapter_clear_uav: Some(
            adapter_vk_clear_uav
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *mut wined3d_unordered_access_view,
                    *const wined3d_uvec4,
                    bool,
                ) -> (),
        ),
        adapter_generate_mipmap: Some(
            adapter_vk_generate_mipmap
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *mut wined3d_shader_resource_view,
                ) -> (),
        ),
    }
};

static mut vulkan_instance_extensions: [C2RustUnnamed_55; 4] = [
    C2RustUnnamed_55 {
        name: VK_KHR_GET_PHYSICAL_DEVICE_PROPERTIES_2_EXTENSION_NAME.as_ptr(),
        core_since_version: VK_API_VERSION_1_1 as ::core::ffi::c_uint,
        required: (FALSE as i32),
    },
    C2RustUnnamed_55 {
        name: VK_KHR_EXTERNAL_MEMORY_EXTENSION_NAME.as_ptr(),
        core_since_version: VK_API_VERSION_1_1 as ::core::ffi::c_uint,
        required: (FALSE as i32),
    },
    C2RustUnnamed_55 {
        name: VK_KHR_SURFACE_EXTENSION_NAME.as_ptr(),
        core_since_version: !(0 as ::core::ffi::c_uint),
        required: (TRUE as i32),
    },
    C2RustUnnamed_55 {
        name: VK_KHR_WIN32_SURFACE_EXTENSION_NAME.as_ptr(),
        core_since_version: !(0 as ::core::ffi::c_uint),
        required: (TRUE as i32),
    },
];

#[repr(C)]
pub struct wined3d_physical_device_info {
    pub dynamic_state_features: VkPhysicalDeviceExtendedDynamicStateFeaturesEXT,
    pub dynamic_state2_features: VkPhysicalDeviceExtendedDynamicState2FeaturesEXT,
    pub dynamic_state3_features: VkPhysicalDeviceExtendedDynamicState3FeaturesEXT,
    pub host_query_reset_features: VkPhysicalDeviceHostQueryResetFeatures,
    pub draw_parameters_features: VkPhysicalDeviceShaderDrawParametersFeatures,
    pub xfb_features: VkPhysicalDeviceTransformFeedbackFeaturesEXT,
    pub vertex_divisor_features: VkPhysicalDeviceVertexAttributeDivisorFeaturesEXT,

    pub features2: VkPhysicalDeviceFeatures2,
}

#[repr(C)]
pub struct wined3d_view_vk_destroy_ctx {
    pub device_vk: *mut wined3d_device_vk,
    pub vk_buffer_view: *mut VkBufferView,
    pub vk_image_view: *mut VkImageView,
    pub bo_user: *mut wined3d_bo_user,
    pub vk_counter_bo: *mut wined3d_bo_vk,
    pub vk_counter_view: *mut VkBufferView,
    pub command_buffer_id: *mut u64,
    pub object: *mut core::ffi::c_void,
    pub free: *mut wined3d_view_vk_destroy_ctx,
}

// The C source leaves the element type of the anonymous `static const struct`
// (used by wined3d_adapter_vk_ops / vulkan_instance_extensions) unnamed.
// Named here so the table type used by the translated functions exists.
#[repr(C)]
pub struct wined3d_adapter_vk_extension {
    pub name: *const i8,
    pub core_since_version: u32,
    pub required: i32,
}

unsafe extern "C" fn wined3d_adapter_vk_const(adapter: *const wined3d_adapter) -> *const wined3d_adapter_vk {
    return CONTAINING_RECORD!(adapter, wined3d_adapter_vk, a) as *const wined3d_adapter_vk;
}

unsafe extern "C" fn debug_vk_version(version: uint32_t) -> *const ::core::ffi::c_char {
    return wine_dbg_sprintf(
        b"%u.%u.%u\0".as_ptr() as *const ::core::ffi::c_char,
        VK_VERSION_MAJOR!(version),
        VK_VERSION_MINOR!(version),
        VK_VERSION_PATCH!(version),
    );
}

unsafe extern "C" fn hresult_from_vk_result(vr: VkResult) -> HRESULT {
    match vr {
        VK_SUCCESS => {
            return S_OK;
        }
        VK_ERROR_OUT_OF_HOST_MEMORY => {
            WARN!("Out of host memory.\n");
            return E_OUTOFMEMORY;
        }
        VK_ERROR_OUT_OF_DEVICE_MEMORY => {
            WARN!("Out of device memory.\n");
            return E_OUTOFMEMORY;
        }
        VK_ERROR_DEVICE_LOST => {
            WARN!("Device lost.\n");
            return E_FAIL;
        }
        VK_ERROR_EXTENSION_NOT_PRESENT => {
            WARN!("Extension not present.\n");
            return E_FAIL;
        }
        _ => {
            FIXME!("Unhandled VkResult %d.\n", vr);
            return E_FAIL;
        }
    }
}

unsafe extern "C" fn wined3d_load_vulkan(vk_info: *mut wined3d_vk_info) -> BOOL {
    let vk_ops: *mut vulkan_ops = core::ptr::addr_of_mut!((*vk_info).vk_ops);
    (*vk_info).vulkan_lib = LoadLibraryA(b"winevulkan.dll\0".as_ptr() as LPCSTR) as HMODULE;
    if (*vk_info).vulkan_lib.is_null() {
        (*vk_info).vulkan_lib = LoadLibraryA(b"vulkan-1.dll\0".as_ptr() as LPCSTR) as HMODULE;
    }
    if (*vk_info).vulkan_lib.is_null() {
        WARN!("Failed to load vulkan-1.dll.\n");
        return (FALSE) as BOOL;
    }
    (*vk_ops).vkGetInstanceProcAddr =
        ::core::mem::transmute::<*mut ::core::ffi::c_void, PFN_vkGetInstanceProcAddr>(
            ::core::mem::transmute::<FARPROC, *mut ::core::ffi::c_void>(GetProcAddress(
                (*vk_info).vulkan_lib as HINSTANCE,
                b"vkGetInstanceProcAddr\0".as_ptr() as LPCSTR,
            )),
        );
    if (*vk_ops).vkGetInstanceProcAddr.is_none() {
        FreeLibrary((*vk_info).vulkan_lib);
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn wined3d_unload_vulkan(mut vk_info: *mut wined3d_vk_info) {
    if !(*vk_info).vulkan_lib.is_null() {
        FreeLibrary((*vk_info).vulkan_lib);
        (*vk_info).vulkan_lib = core::ptr::null_mut();
    }
}

unsafe extern "C" fn adapter_vk_destroy(adapter: *mut wined3d_adapter) {
    let adapter_vk: *mut wined3d_adapter_vk = wined3d_adapter_vk(adapter);
    let vk_info: *mut wined3d_vk_info = core::ptr::addr_of_mut!((*adapter_vk).vk_info);

    ((*vk_info).vk_ops.vkDestroyInstance.unwrap_unchecked())((*vk_info).instance, core::ptr::null_mut());
    wined3d_unload_vulkan(vk_info);
    wined3d_adapter_cleanup(core::ptr::addr_of_mut!((*adapter_vk).a));
    free((*adapter_vk).device_extensions as *mut core::ffi::c_void);
    free(adapter_vk as *mut core::ffi::c_void);
}

unsafe extern "C" fn wined3d_select_vulkan_queue_family(
    adapter_vk: *const wined3d_adapter_vk,
    queue_family_index: *mut uint32_t,
    timestamp_bits: *mut uint32_t,
) -> HRESULT {
    let physical_device: VkPhysicalDevice = (*adapter_vk).physical_device;
    let vk_info: *const wined3d_vk_info = core::ptr::addr_of!((*adapter_vk).vk_info);
    let mut queue_properties: *mut VkQueueFamilyProperties = core::ptr::null_mut();
    let mut count: uint32_t = 0;
    let mut i: uint32_t = 0;

    ((*vk_info).vk_ops.vkGetPhysicalDeviceQueueFamilyProperties.unwrap_unchecked())(physical_device, core::ptr::addr_of_mut!(count), core::ptr::null_mut());

    queue_properties = calloc(
        (count as size_t) as _,
        (core::mem::size_of::<VkQueueFamilyProperties>() as size_t) as _,
    ) as *mut VkQueueFamilyProperties;

    if queue_properties.is_null() {
        return E_OUTOFMEMORY;
    }

    ((*vk_info).vk_ops.vkGetPhysicalDeviceQueueFamilyProperties.unwrap_unchecked())(physical_device, core::ptr::addr_of_mut!(count), queue_properties);

    i = 0;
    while i < count {
        if (*queue_properties.offset(i as isize)).queueFlags & VK_QUEUE_GRAPHICS_BIT != 0 {
            *queue_family_index = i;
            *timestamp_bits = (*queue_properties.offset(i as isize)).timestampValidBits;
            free(queue_properties as *mut core::ffi::c_void);
            return WINED3D_OK;
        }
        i = i.wrapping_add(1);
    }
    free(queue_properties as *mut core::ffi::c_void);

    WARN!("Failed to find graphics queue.\n");
    return E_FAIL;
}

unsafe extern "C" fn wined3d_disable_vulkan_features(info: *mut wined3d_physical_device_info) {
    let dynamic_state3: *mut VkPhysicalDeviceExtendedDynamicState3FeaturesEXT = core::ptr::addr_of_mut!((*info).dynamic_state3_features);
    let features: *mut VkPhysicalDeviceFeatures = core::ptr::addr_of_mut!((*info).features2.features);

    (*features).depthBounds = VK_FALSE;
    (*features).wideLines = VK_FALSE;
    (*features).alphaToOne = VK_FALSE;
    (*features).textureCompressionETC2 = VK_FALSE;
    (*features).textureCompressionASTC_LDR = VK_FALSE;
    (*features).shaderStorageImageMultisample = VK_FALSE;
    (*features).shaderUniformBufferArrayDynamicIndexing = VK_FALSE;
    (*features).shaderSampledImageArrayDynamicIndexing = VK_FALSE;
    (*features).shaderStorageBufferArrayDynamicIndexing = VK_FALSE;
    (*features).shaderStorageImageArrayDynamicIndexing = VK_FALSE;
    (*features).shaderInt64 = VK_FALSE;
    (*features).shaderInt16 = VK_FALSE;
    (*features).shaderResourceResidency = VK_FALSE;
    (*features).shaderResourceMinLod = VK_FALSE;
    (*features).shaderTessellationAndGeometryPointSize = VK_FALSE;
    (*features).sparseBinding = VK_FALSE;
    (*features).sparseResidencyBuffer = VK_FALSE;
    (*features).sparseResidencyImage2D = VK_FALSE;
    (*features).sparseResidencyImage3D = VK_FALSE;
    (*features).sparseResidency2Samples = VK_FALSE;
    (*features).sparseResidency4Samples = VK_FALSE;
    (*features).sparseResidency8Samples = VK_FALSE;
    (*features).sparseResidency16Samples = VK_FALSE;
    (*features).sparseResidencyAliased = VK_FALSE;
    (*features).inheritedQueries = VK_FALSE;

    (*dynamic_state3).extendedDynamicState3AlphaToOneEnable = VK_FALSE;
    (*dynamic_state3).extendedDynamicState3ColorBlendAdvanced = VK_FALSE;
    (*dynamic_state3).extendedDynamicState3ConservativeRasterizationMode = VK_FALSE;
    (*dynamic_state3).extendedDynamicState3CoverageModulationMode = VK_FALSE;
    (*dynamic_state3).extendedDynamicState3CoverageModulationTable = VK_FALSE;
    (*dynamic_state3).extendedDynamicState3CoverageModulationTableEnable = VK_FALSE;
    (*dynamic_state3).extendedDynamicState3CoverageReductionMode = VK_FALSE;
    (*dynamic_state3).extendedDynamicState3CoverageToColorEnable = VK_FALSE;
    (*dynamic_state3).extendedDynamicState3CoverageToColorLocation = VK_FALSE;
    (*dynamic_state3).extendedDynamicState3DepthClipEnable = VK_FALSE;
    (*dynamic_state3).extendedDynamicState3DepthClipNegativeOneToOne = VK_FALSE;
    (*dynamic_state3).extendedDynamicState3ExtraPrimitiveOverestimationSize = VK_FALSE;
    (*dynamic_state3).extendedDynamicState3LineRasterizationMode = VK_FALSE;
    (*dynamic_state3).extendedDynamicState3LineStippleEnable = VK_FALSE;
    (*dynamic_state3).extendedDynamicState3LogicOpEnable = VK_FALSE;
    (*dynamic_state3).extendedDynamicState3PolygonMode = VK_FALSE;
    (*dynamic_state3).extendedDynamicState3ProvokingVertexMode = VK_FALSE;
    (*dynamic_state3).extendedDynamicState3RasterizationStream = VK_FALSE;
    (*dynamic_state3).extendedDynamicState3RepresentativeFragmentTestEnable = VK_FALSE;
    (*dynamic_state3).extendedDynamicState3SampleLocationsEnable = VK_FALSE;
    (*dynamic_state3).extendedDynamicState3ShadingRateImageEnable = VK_FALSE;
    (*dynamic_state3).extendedDynamicState3TessellationDomainOrigin = VK_FALSE;
    (*dynamic_state3).extendedDynamicState3ViewportWScalingEnable = VK_FALSE;
    (*dynamic_state3).extendedDynamicState3ViewportSwizzle = VK_FALSE;
}

unsafe extern "C" fn wined3d_allocator_vk_create_chunk(
    mut allocator: *mut wined3d_allocator,
    mut context: *mut wined3d_context,
    mut memory_type: ::core::ffi::c_uint,
    mut chunk_size: size_t,
) -> *mut wined3d_allocator_chunk {
    let mut context_vk: *mut wined3d_context_vk = wined3d_context_vk(context);
    let mut chunk_vk: *mut wined3d_allocator_chunk_vk = ::core::ptr::null_mut::<wined3d_allocator_chunk_vk>();

    chunk_vk = malloc(core::mem::size_of::<wined3d_allocator_chunk_vk>() as size_t)
        as *mut wined3d_allocator_chunk_vk;
    if chunk_vk.is_null() {
        return ::core::ptr::null_mut::<wined3d_allocator_chunk>();
    }

    if wined3d_allocator_chunk_init(&raw mut (*chunk_vk).c, allocator) == false {
        free(chunk_vk as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<wined3d_allocator_chunk>();
    }

    (*chunk_vk).vk_memory = wined3d_context_vk_allocate_vram_chunk_memory(context_vk, memory_type, chunk_size);
    if (*chunk_vk).vk_memory == 0 {
        wined3d_allocator_chunk_cleanup(&raw mut (*chunk_vk).c);
        free(chunk_vk as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<wined3d_allocator_chunk>();
    }

    list_add_head(
        &raw mut (*(*allocator).pools.offset(memory_type as isize)).chunks,
        &raw mut (*chunk_vk).c.entry,
    );

    return &raw mut (*chunk_vk).c;
}

unsafe extern "C" fn wined3d_allocator_vk_destroy_chunk(chunk: *mut wined3d_allocator_chunk) {
    let mut chunk_vk: *mut wined3d_allocator_chunk_vk = wined3d_allocator_chunk_vk(chunk);
    let mut vk_info: *const wined3d_vk_info;
    let mut device_vk: *mut wined3d_device_vk;

    device_vk = wined3d_device_vk_from_allocator((*chunk_vk).c.allocator);
    vk_info = core::ptr::addr_of_mut!((*device_vk).vk_info);

    if !(*chunk_vk).c.map_ptr.is_null() {
        TRACE!("chunk %p.\n", chunk);
        ((*vk_info).vk_ops.vkUnmapMemory.unwrap_unchecked())((*device_vk).vk_device, (*chunk_vk).vk_memory);
        adapter_adjust_mapped_memory((*device_vk).d.adapter, (-(WINED3D_ALLOCATOR_CHUNK_SIZE as i32)) as ssize_t);
    }
    ((*vk_info).vk_ops.vkFreeMemory.unwrap_unchecked())((*device_vk).vk_device, (*chunk_vk).vk_memory, core::ptr::null::<VkAllocationCallbacks>());
    TRACE!("Freed memory 0x%s.\n", wine_dbgstr_longlong((*chunk_vk).vk_memory as ULONGLONG));
    wined3d_allocator_chunk_cleanup(core::ptr::addr_of_mut!((*chunk_vk).c));
    free(chunk_vk as *mut core::ffi::c_void);
}

unsafe extern "C" fn add_structure(features2: *mut VkPhysicalDeviceFeatures2, s: *mut core::ffi::c_void) {
    let mut base: *mut VkBaseOutStructure = s as *mut VkBaseOutStructure;

    (*base).pNext = (*features2).pNext as *mut VkBaseOutStructure;
    (*features2).pNext = base as *mut core::ffi::c_void;
}

unsafe extern "C" fn get_physical_device_info(adapter_vk: *const wined3d_adapter_vk, info: *mut wined3d_physical_device_info) {
    let vertex_divisor_features: *mut VkPhysicalDeviceVertexAttributeDivisorFeaturesEXT = core::ptr::addr_of_mut!((*info).vertex_divisor_features);
    let dynamic_state3_features: *mut VkPhysicalDeviceExtendedDynamicState3FeaturesEXT = core::ptr::addr_of_mut!((*info).dynamic_state3_features);
    let dynamic_state2_features: *mut VkPhysicalDeviceExtendedDynamicState2FeaturesEXT = core::ptr::addr_of_mut!((*info).dynamic_state2_features);
    let draw_parameters_features: *mut VkPhysicalDeviceShaderDrawParametersFeatures = core::ptr::addr_of_mut!((*info).draw_parameters_features);
    let dynamic_state_features: *mut VkPhysicalDeviceExtendedDynamicStateFeaturesEXT = core::ptr::addr_of_mut!((*info).dynamic_state_features);
    let host_query_reset_features: *mut VkPhysicalDeviceHostQueryResetFeatures = core::ptr::addr_of_mut!((*info).host_query_reset_features);
    let xfb_features: *mut VkPhysicalDeviceTransformFeedbackFeaturesEXT = core::ptr::addr_of_mut!((*info).xfb_features);
    let physical_device: VkPhysicalDevice = (*adapter_vk).physical_device;
    let vk_info: *const wined3d_vk_info = core::ptr::addr_of!((*adapter_vk).vk_info);
    let features2: *mut VkPhysicalDeviceFeatures2 = core::ptr::addr_of_mut!((*info).features2);

    memset(
        info as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<wined3d_physical_device_info>() as usize,
    );

    (*draw_parameters_features).sType = VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SHADER_DRAW_PARAMETERS_FEATURES;
    if (*vk_info).api_version as uint32_t >= VK_API_VERSION_1_1 {
        add_structure(features2, draw_parameters_features as *mut ::core::ffi::c_void);
    } else {
        (*draw_parameters_features).shaderDrawParameters =
            (*vk_info).supported[WINED3D_VK_KHR_SHADER_DRAW_PARAMETERS as usize] as VkBool32;
    }

    (*xfb_features).sType = VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_TRANSFORM_FEEDBACK_FEATURES_EXT;
    if (*vk_info).supported[WINED3D_VK_EXT_TRANSFORM_FEEDBACK as usize] != 0 {
        add_structure(features2, xfb_features as *mut ::core::ffi::c_void);
    }

    (*vertex_divisor_features).sType = VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_FEATURES_EXT;
    if (*vk_info).supported[WINED3D_VK_EXT_VERTEX_ATTRIBUTE_DIVISOR as usize] != 0 {
        add_structure(features2, vertex_divisor_features as *mut ::core::ffi::c_void);
    }

    (*host_query_reset_features).sType = VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_HOST_QUERY_RESET_FEATURES;
    if (*vk_info).supported[WINED3D_VK_EXT_HOST_QUERY_RESET as usize] != 0 {
        add_structure(features2, host_query_reset_features as *mut ::core::ffi::c_void);
    }

    (*dynamic_state3_features).sType = VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTENDED_DYNAMIC_STATE_3_FEATURES_EXT;
    if (*vk_info).supported[WINED3D_VK_EXT_EXTENDED_DYNAMIC_STATE3 as usize] != 0 {
        add_structure(features2, dynamic_state3_features as *mut ::core::ffi::c_void);
    }

    (*dynamic_state2_features).sType = VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTENDED_DYNAMIC_STATE_2_FEATURES_EXT;
    if (*vk_info).supported[WINED3D_VK_EXT_EXTENDED_DYNAMIC_STATE2 as usize] != 0 {
        add_structure(features2, dynamic_state2_features as *mut ::core::ffi::c_void);
    }

    (*dynamic_state_features).sType = VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTENDED_DYNAMIC_STATE_FEATURES_EXT;
    if (*vk_info).supported[WINED3D_VK_EXT_EXTENDED_DYNAMIC_STATE as usize] != 0 {
        add_structure(features2, dynamic_state_features as *mut ::core::ffi::c_void);
    }

    (*features2).sType = VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_FEATURES_2;

    if (*vk_info).vk_ops.vkGetPhysicalDeviceFeatures2.is_some() {
        ((*vk_info).vk_ops.vkGetPhysicalDeviceFeatures2.unwrap_unchecked())(physical_device, features2);
    } else {
        ((*vk_info).vk_ops.vkGetPhysicalDeviceFeatures.unwrap_unchecked())(physical_device, core::ptr::addr_of_mut!((*features2).features));
    }
}

unsafe extern "C" fn adapter_vk_create_device(
    wined3d: *mut wined3d,
    adapter: *const wined3d_adapter,
    device_type: wined3d_device_type,
    focus_window: HWND,
    flags: core::ffi::c_uint,
    surface_alignment: BYTE,
    levels: *const wined3d_feature_level,
    level_count: core::ffi::c_uint,
    device_parent: *mut wined3d_device_parent,
    device: *mut *mut wined3d_device,
) -> HRESULT {
    let adapter_vk: *const wined3d_adapter_vk = wined3d_adapter_vk_const(adapter);
    let vk_info: *const wined3d_vk_info = core::ptr::addr_of!((*adapter_vk).vk_info);
    let mut physical_device_info: wined3d_physical_device_info = core::mem::zeroed();
    static PRIORITIES: [f32; 1] = [1.0f32];
    let mut device_vk: *mut wined3d_device_vk = core::ptr::null_mut();
    let mut vk_device: VkDevice = core::ptr::null_mut();
    let mut queue_info: VkDeviceQueueCreateInfo = core::mem::zeroed();
    let mut physical_device: VkPhysicalDevice = core::ptr::null_mut();
    let mut device_info: VkDeviceCreateInfo = core::mem::zeroed();
    let mut queue_family_index: uint32_t = 0;
    let mut timestamp_bits: uint32_t = 0;
    let mut vr: VkResult = core::mem::zeroed();
    let mut hr: HRESULT = 0;

    device_vk = calloc(
        1,
        (core::mem::size_of::<wined3d_device_vk>() as size_t) as _,
    ) as *mut wined3d_device_vk;
    if device_vk.is_null() {
        return E_OUTOFMEMORY;
    }

    'fail: {
        hr = wined3d_select_vulkan_queue_family(
            adapter_vk,
            core::ptr::addr_of_mut!(queue_family_index),
            core::ptr::addr_of_mut!(timestamp_bits),
        );
        if hr < 0 {
            break 'fail;
        }

        physical_device = (*adapter_vk).physical_device;

        get_physical_device_info(adapter_vk, core::ptr::addr_of_mut!(physical_device_info));
        wined3d_disable_vulkan_features(core::ptr::addr_of_mut!(physical_device_info));

        queue_info.sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO;
        queue_info.pNext = core::ptr::null();
        queue_info.flags = 0;
        queue_info.queueFamilyIndex = queue_family_index;
        queue_info.queueCount = ARRAY_SIZE!(PRIORITIES) as uint32_t;
        queue_info.pQueuePriorities = PRIORITIES.as_ptr();

        device_info.sType = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO;
        device_info.pNext = physical_device_info.features2.pNext;
        device_info.flags = 0;
        device_info.queueCreateInfoCount = 1;
        device_info.pQueueCreateInfos = core::ptr::addr_of!(queue_info);
        device_info.enabledLayerCount = 0;
        device_info.ppEnabledLayerNames = core::ptr::null();
        device_info.enabledExtensionCount = (*adapter_vk).device_extension_count as uint32_t;
        device_info.ppEnabledExtensionNames = (*adapter_vk).device_extensions;
        device_info.pEnabledFeatures = core::ptr::addr_of!(physical_device_info.features2.features);

        vr = ((*vk_info).vk_ops.vkCreateDevice.unwrap_unchecked())(
            physical_device,
            core::ptr::addr_of_mut!(device_info),
            core::ptr::null(),
            core::ptr::addr_of_mut!(vk_device),
        );
        if (vr as i32) < 0 {
            WARN!("Failed to create Vulkan device, vr %s.\n", wined3d_debug_vkresult(vr));
            vk_device = core::ptr::null_mut();
            hr = hresult_from_vk_result(vr);
            break 'fail;
        }

        (*device_vk).vk_device = vk_device;
        ((*vk_info).vk_ops.vkGetDeviceQueue.unwrap_unchecked())(
            vk_device,
            queue_family_index,
            0,
            core::ptr::addr_of_mut!((*device_vk).vk_queue),
        );
        (*device_vk).vk_queue_family_index = queue_family_index;
        (*device_vk).timestamp_bits = timestamp_bits;

        (*device_vk).vk_info = *vk_info;

        // VK_DEVICE_FUNCS(): expanded inline from VK_DEVICE_PFN / VK_DEVICE_EXT_PFN.
        (*device_vk).vk_info.vk_ops.vkAllocateCommandBuffers = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkAllocateCommandBuffers>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkAllocateCommandBuffers\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkAllocateCommandBuffers.is_none() {
            WARN!("Could not get device proc addr for 'vkAllocateCommandBuffers'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkAllocateDescriptorSets = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkAllocateDescriptorSets>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkAllocateDescriptorSets\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkAllocateDescriptorSets.is_none() {
            WARN!("Could not get device proc addr for 'vkAllocateDescriptorSets'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkAllocateMemory = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkAllocateMemory>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkAllocateMemory\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkAllocateMemory.is_none() {
            WARN!("Could not get device proc addr for 'vkAllocateMemory'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkBeginCommandBuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkBeginCommandBuffer>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkBeginCommandBuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkBeginCommandBuffer.is_none() {
            WARN!("Could not get device proc addr for 'vkBeginCommandBuffer'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkBindBufferMemory = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkBindBufferMemory>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkBindBufferMemory\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkBindBufferMemory.is_none() {
            WARN!("Could not get device proc addr for 'vkBindBufferMemory'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkBindImageMemory = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkBindImageMemory>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkBindImageMemory\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkBindImageMemory.is_none() {
            WARN!("Could not get device proc addr for 'vkBindImageMemory'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdBeginQuery = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdBeginQuery>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdBeginQuery\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdBeginQuery.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdBeginQuery'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdBeginRenderPass = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdBeginRenderPass>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdBeginRenderPass\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdBeginRenderPass.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdBeginRenderPass'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdBindDescriptorSets = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdBindDescriptorSets>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdBindDescriptorSets\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdBindDescriptorSets.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdBindDescriptorSets'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdBindIndexBuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdBindIndexBuffer>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdBindIndexBuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdBindIndexBuffer.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdBindIndexBuffer'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdBindPipeline = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdBindPipeline>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdBindPipeline\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdBindPipeline.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdBindPipeline'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdBindVertexBuffers = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdBindVertexBuffers>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdBindVertexBuffers\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdBindVertexBuffers.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdBindVertexBuffers'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdBlitImage = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdBlitImage>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdBlitImage\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdBlitImage.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdBlitImage'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdClearAttachments = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdClearAttachments>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdClearAttachments\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdClearAttachments.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdClearAttachments'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdClearColorImage = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdClearColorImage>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdClearColorImage\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdClearColorImage.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdClearColorImage'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdClearDepthStencilImage = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdClearDepthStencilImage>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdClearDepthStencilImage\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdClearDepthStencilImage.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdClearDepthStencilImage'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdCopyBuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdCopyBuffer>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdCopyBuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdCopyBuffer.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdCopyBuffer'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdCopyBufferToImage = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdCopyBufferToImage>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdCopyBufferToImage\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdCopyBufferToImage.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdCopyBufferToImage'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdCopyImage = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdCopyImage>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdCopyImage\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdCopyImage.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdCopyImage'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdCopyImageToBuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdCopyImageToBuffer>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdCopyImageToBuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdCopyImageToBuffer.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdCopyImageToBuffer'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdCopyQueryPoolResults = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdCopyQueryPoolResults>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdCopyQueryPoolResults\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdCopyQueryPoolResults.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdCopyQueryPoolResults'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdDispatch = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdDispatch>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdDispatch\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdDispatch.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdDispatch'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdDispatchIndirect = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdDispatchIndirect>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdDispatchIndirect\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdDispatchIndirect.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdDispatchIndirect'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdDraw = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdDraw>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdDraw\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdDraw.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdDraw'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdDrawIndexed = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdDrawIndexed>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdDrawIndexed\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdDrawIndexed.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdDrawIndexed'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdDrawIndexedIndirect = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdDrawIndexedIndirect>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdDrawIndexedIndirect\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdDrawIndexedIndirect.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdDrawIndexedIndirect'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdDrawIndirect = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdDrawIndirect>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdDrawIndirect\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdDrawIndirect.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdDrawIndirect'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdEndQuery = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdEndQuery>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdEndQuery\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdEndQuery.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdEndQuery'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdEndRenderPass = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdEndRenderPass>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdEndRenderPass\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdEndRenderPass.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdEndRenderPass'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdExecuteCommands = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdExecuteCommands>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdExecuteCommands\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdExecuteCommands.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdExecuteCommands'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdFillBuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdFillBuffer>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdFillBuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdFillBuffer.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdFillBuffer'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdNextSubpass = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdNextSubpass>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdNextSubpass\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdNextSubpass.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdNextSubpass'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdPipelineBarrier = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdPipelineBarrier>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdPipelineBarrier\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdPipelineBarrier.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdPipelineBarrier'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdPushConstants = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdPushConstants>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdPushConstants\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdPushConstants.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdPushConstants'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdResetEvent = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdResetEvent>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdResetEvent\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdResetEvent.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdResetEvent'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdResetQueryPool = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdResetQueryPool>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdResetQueryPool\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdResetQueryPool.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdResetQueryPool'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdResolveImage = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdResolveImage>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdResolveImage\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdResolveImage.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdResolveImage'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdSetBlendConstants = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetBlendConstants>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetBlendConstants\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdSetBlendConstants.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdSetBlendConstants'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdSetDepthBias = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetDepthBias>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetDepthBias\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdSetDepthBias.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdSetDepthBias'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdSetDepthBounds = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetDepthBounds>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetDepthBounds\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdSetDepthBounds.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdSetDepthBounds'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdSetEvent = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetEvent>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetEvent\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdSetEvent.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdSetEvent'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdSetLineWidth = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetLineWidth>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetLineWidth\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdSetLineWidth.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdSetLineWidth'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdSetScissor = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetScissor>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetScissor\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdSetScissor.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdSetScissor'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdSetStencilCompareMask = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetStencilCompareMask>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetStencilCompareMask\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdSetStencilCompareMask.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdSetStencilCompareMask'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdSetStencilReference = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetStencilReference>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetStencilReference\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdSetStencilReference.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdSetStencilReference'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdSetStencilWriteMask = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetStencilWriteMask>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetStencilWriteMask\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdSetStencilWriteMask.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdSetStencilWriteMask'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdSetViewport = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetViewport>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetViewport\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdSetViewport.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdSetViewport'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdUpdateBuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdUpdateBuffer>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdUpdateBuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdUpdateBuffer.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdUpdateBuffer'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdWaitEvents = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdWaitEvents>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdWaitEvents\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdWaitEvents.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdWaitEvents'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCmdWriteTimestamp = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdWriteTimestamp>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdWriteTimestamp\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCmdWriteTimestamp.is_none() {
            WARN!("Could not get device proc addr for 'vkCmdWriteTimestamp'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCreateBuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateBuffer>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCreateBuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCreateBuffer.is_none() {
            WARN!("Could not get device proc addr for 'vkCreateBuffer'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCreateBufferView = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateBufferView>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCreateBufferView\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCreateBufferView.is_none() {
            WARN!("Could not get device proc addr for 'vkCreateBufferView'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCreateCommandPool = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateCommandPool>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCreateCommandPool\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCreateCommandPool.is_none() {
            WARN!("Could not get device proc addr for 'vkCreateCommandPool'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCreateComputePipelines = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateComputePipelines>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCreateComputePipelines\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCreateComputePipelines.is_none() {
            WARN!("Could not get device proc addr for 'vkCreateComputePipelines'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCreateDescriptorPool = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateDescriptorPool>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCreateDescriptorPool\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCreateDescriptorPool.is_none() {
            WARN!("Could not get device proc addr for 'vkCreateDescriptorPool'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCreateDescriptorSetLayout = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateDescriptorSetLayout>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCreateDescriptorSetLayout\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCreateDescriptorSetLayout.is_none() {
            WARN!("Could not get device proc addr for 'vkCreateDescriptorSetLayout'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCreateEvent = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateEvent>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCreateEvent\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCreateEvent.is_none() {
            WARN!("Could not get device proc addr for 'vkCreateEvent'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCreateFence = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateFence>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCreateFence\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCreateFence.is_none() {
            WARN!("Could not get device proc addr for 'vkCreateFence'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCreateFramebuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateFramebuffer>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCreateFramebuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCreateFramebuffer.is_none() {
            WARN!("Could not get device proc addr for 'vkCreateFramebuffer'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCreateGraphicsPipelines = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateGraphicsPipelines>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCreateGraphicsPipelines\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCreateGraphicsPipelines.is_none() {
            WARN!("Could not get device proc addr for 'vkCreateGraphicsPipelines'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCreateImage = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateImage>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCreateImage\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCreateImage.is_none() {
            WARN!("Could not get device proc addr for 'vkCreateImage'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCreateImageView = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateImageView>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCreateImageView\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCreateImageView.is_none() {
            WARN!("Could not get device proc addr for 'vkCreateImageView'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCreatePipelineCache = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreatePipelineCache>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCreatePipelineCache\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCreatePipelineCache.is_none() {
            WARN!("Could not get device proc addr for 'vkCreatePipelineCache'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCreatePipelineLayout = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreatePipelineLayout>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCreatePipelineLayout\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCreatePipelineLayout.is_none() {
            WARN!("Could not get device proc addr for 'vkCreatePipelineLayout'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCreateQueryPool = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateQueryPool>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCreateQueryPool\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCreateQueryPool.is_none() {
            WARN!("Could not get device proc addr for 'vkCreateQueryPool'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCreateRenderPass = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateRenderPass>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCreateRenderPass\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCreateRenderPass.is_none() {
            WARN!("Could not get device proc addr for 'vkCreateRenderPass'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCreateSampler = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateSampler>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCreateSampler\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCreateSampler.is_none() {
            WARN!("Could not get device proc addr for 'vkCreateSampler'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCreateSemaphore = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateSemaphore>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCreateSemaphore\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCreateSemaphore.is_none() {
            WARN!("Could not get device proc addr for 'vkCreateSemaphore'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCreateShaderModule = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateShaderModule>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCreateShaderModule\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCreateShaderModule.is_none() {
            WARN!("Could not get device proc addr for 'vkCreateShaderModule'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkDestroyBuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyBuffer>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkDestroyBuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkDestroyBuffer.is_none() {
            WARN!("Could not get device proc addr for 'vkDestroyBuffer'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkDestroyBufferView = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyBufferView>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkDestroyBufferView\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkDestroyBufferView.is_none() {
            WARN!("Could not get device proc addr for 'vkDestroyBufferView'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkDestroyCommandPool = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyCommandPool>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkDestroyCommandPool\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkDestroyCommandPool.is_none() {
            WARN!("Could not get device proc addr for 'vkDestroyCommandPool'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkDestroyDescriptorPool = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyDescriptorPool>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkDestroyDescriptorPool\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkDestroyDescriptorPool.is_none() {
            WARN!("Could not get device proc addr for 'vkDestroyDescriptorPool'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkDestroyDescriptorSetLayout = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyDescriptorSetLayout>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkDestroyDescriptorSetLayout\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkDestroyDescriptorSetLayout.is_none() {
            WARN!("Could not get device proc addr for 'vkDestroyDescriptorSetLayout'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkDestroyDevice = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyDevice>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkDestroyDevice\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkDestroyDevice.is_none() {
            WARN!("Could not get device proc addr for 'vkDestroyDevice'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkDestroyEvent = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyEvent>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkDestroyEvent\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkDestroyEvent.is_none() {
            WARN!("Could not get device proc addr for 'vkDestroyEvent'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkDestroyFence = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyFence>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkDestroyFence\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkDestroyFence.is_none() {
            WARN!("Could not get device proc addr for 'vkDestroyFence'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkDestroyFramebuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyFramebuffer>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkDestroyFramebuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkDestroyFramebuffer.is_none() {
            WARN!("Could not get device proc addr for 'vkDestroyFramebuffer'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkDestroyImage = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyImage>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkDestroyImage\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkDestroyImage.is_none() {
            WARN!("Could not get device proc addr for 'vkDestroyImage'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkDestroyImageView = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyImageView>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkDestroyImageView\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkDestroyImageView.is_none() {
            WARN!("Could not get device proc addr for 'vkDestroyImageView'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkDestroyPipeline = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyPipeline>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkDestroyPipeline\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkDestroyPipeline.is_none() {
            WARN!("Could not get device proc addr for 'vkDestroyPipeline'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkDestroyPipelineCache = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyPipelineCache>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkDestroyPipelineCache\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkDestroyPipelineCache.is_none() {
            WARN!("Could not get device proc addr for 'vkDestroyPipelineCache'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkDestroyPipelineLayout = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyPipelineLayout>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkDestroyPipelineLayout\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkDestroyPipelineLayout.is_none() {
            WARN!("Could not get device proc addr for 'vkDestroyPipelineLayout'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkDestroyQueryPool = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyQueryPool>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkDestroyQueryPool\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkDestroyQueryPool.is_none() {
            WARN!("Could not get device proc addr for 'vkDestroyQueryPool'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkDestroyRenderPass = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyRenderPass>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkDestroyRenderPass\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkDestroyRenderPass.is_none() {
            WARN!("Could not get device proc addr for 'vkDestroyRenderPass'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkDestroySampler = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroySampler>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkDestroySampler\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkDestroySampler.is_none() {
            WARN!("Could not get device proc addr for 'vkDestroySampler'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkDestroySemaphore = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroySemaphore>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkDestroySemaphore\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkDestroySemaphore.is_none() {
            WARN!("Could not get device proc addr for 'vkDestroySemaphore'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkDestroyShaderModule = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyShaderModule>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkDestroyShaderModule\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkDestroyShaderModule.is_none() {
            WARN!("Could not get device proc addr for 'vkDestroyShaderModule'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkDeviceWaitIdle = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDeviceWaitIdle>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkDeviceWaitIdle\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkDeviceWaitIdle.is_none() {
            WARN!("Could not get device proc addr for 'vkDeviceWaitIdle'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkEndCommandBuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkEndCommandBuffer>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkEndCommandBuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkEndCommandBuffer.is_none() {
            WARN!("Could not get device proc addr for 'vkEndCommandBuffer'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkFlushMappedMemoryRanges = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkFlushMappedMemoryRanges>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkFlushMappedMemoryRanges\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkFlushMappedMemoryRanges.is_none() {
            WARN!("Could not get device proc addr for 'vkFlushMappedMemoryRanges'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkFreeCommandBuffers = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkFreeCommandBuffers>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkFreeCommandBuffers\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkFreeCommandBuffers.is_none() {
            WARN!("Could not get device proc addr for 'vkFreeCommandBuffers'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkFreeDescriptorSets = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkFreeDescriptorSets>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkFreeDescriptorSets\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkFreeDescriptorSets.is_none() {
            WARN!("Could not get device proc addr for 'vkFreeDescriptorSets'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkFreeMemory = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkFreeMemory>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkFreeMemory\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkFreeMemory.is_none() {
            WARN!("Could not get device proc addr for 'vkFreeMemory'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkGetBufferMemoryRequirements = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetBufferMemoryRequirements>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkGetBufferMemoryRequirements\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkGetBufferMemoryRequirements.is_none() {
            WARN!("Could not get device proc addr for 'vkGetBufferMemoryRequirements'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkGetDeviceMemoryCommitment = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetDeviceMemoryCommitment>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkGetDeviceMemoryCommitment\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkGetDeviceMemoryCommitment.is_none() {
            WARN!("Could not get device proc addr for 'vkGetDeviceMemoryCommitment'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkGetDeviceQueue = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetDeviceQueue>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkGetDeviceQueue\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkGetDeviceQueue.is_none() {
            WARN!("Could not get device proc addr for 'vkGetDeviceQueue'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkGetEventStatus = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetEventStatus>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkGetEventStatus\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkGetEventStatus.is_none() {
            WARN!("Could not get device proc addr for 'vkGetEventStatus'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkGetFenceStatus = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetFenceStatus>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkGetFenceStatus\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkGetFenceStatus.is_none() {
            WARN!("Could not get device proc addr for 'vkGetFenceStatus'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkGetImageMemoryRequirements = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetImageMemoryRequirements>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkGetImageMemoryRequirements\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkGetImageMemoryRequirements.is_none() {
            WARN!("Could not get device proc addr for 'vkGetImageMemoryRequirements'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkGetImageSparseMemoryRequirements = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetImageSparseMemoryRequirements>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkGetImageSparseMemoryRequirements\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkGetImageSparseMemoryRequirements.is_none() {
            WARN!("Could not get device proc addr for 'vkGetImageSparseMemoryRequirements'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkGetImageSubresourceLayout = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetImageSubresourceLayout>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkGetImageSubresourceLayout\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkGetImageSubresourceLayout.is_none() {
            WARN!("Could not get device proc addr for 'vkGetImageSubresourceLayout'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkGetPipelineCacheData = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetPipelineCacheData>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkGetPipelineCacheData\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkGetPipelineCacheData.is_none() {
            WARN!("Could not get device proc addr for 'vkGetPipelineCacheData'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkGetQueryPoolResults = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetQueryPoolResults>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkGetQueryPoolResults\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkGetQueryPoolResults.is_none() {
            WARN!("Could not get device proc addr for 'vkGetQueryPoolResults'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkGetRenderAreaGranularity = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetRenderAreaGranularity>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkGetRenderAreaGranularity\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkGetRenderAreaGranularity.is_none() {
            WARN!("Could not get device proc addr for 'vkGetRenderAreaGranularity'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkInvalidateMappedMemoryRanges = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkInvalidateMappedMemoryRanges>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkInvalidateMappedMemoryRanges\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkInvalidateMappedMemoryRanges.is_none() {
            WARN!("Could not get device proc addr for 'vkInvalidateMappedMemoryRanges'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkMapMemory = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkMapMemory>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkMapMemory\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkMapMemory.is_none() {
            WARN!("Could not get device proc addr for 'vkMapMemory'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkMergePipelineCaches = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkMergePipelineCaches>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkMergePipelineCaches\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkMergePipelineCaches.is_none() {
            WARN!("Could not get device proc addr for 'vkMergePipelineCaches'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkQueueBindSparse = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkQueueBindSparse>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkQueueBindSparse\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkQueueBindSparse.is_none() {
            WARN!("Could not get device proc addr for 'vkQueueBindSparse'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkQueueSubmit = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkQueueSubmit>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkQueueSubmit\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkQueueSubmit.is_none() {
            WARN!("Could not get device proc addr for 'vkQueueSubmit'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkQueueWaitIdle = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkQueueWaitIdle>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkQueueWaitIdle\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkQueueWaitIdle.is_none() {
            WARN!("Could not get device proc addr for 'vkQueueWaitIdle'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkResetCommandBuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkResetCommandBuffer>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkResetCommandBuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkResetCommandBuffer.is_none() {
            WARN!("Could not get device proc addr for 'vkResetCommandBuffer'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkResetCommandPool = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkResetCommandPool>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkResetCommandPool\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkResetCommandPool.is_none() {
            WARN!("Could not get device proc addr for 'vkResetCommandPool'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkResetDescriptorPool = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkResetDescriptorPool>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkResetDescriptorPool\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkResetDescriptorPool.is_none() {
            WARN!("Could not get device proc addr for 'vkResetDescriptorPool'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkResetEvent = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkResetEvent>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkResetEvent\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkResetEvent.is_none() {
            WARN!("Could not get device proc addr for 'vkResetEvent'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkResetFences = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkResetFences>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkResetFences\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkResetFences.is_none() {
            WARN!("Could not get device proc addr for 'vkResetFences'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkSetEvent = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkSetEvent>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkSetEvent\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkSetEvent.is_none() {
            WARN!("Could not get device proc addr for 'vkSetEvent'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkUnmapMemory = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkUnmapMemory>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkUnmapMemory\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkUnmapMemory.is_none() {
            WARN!("Could not get device proc addr for 'vkUnmapMemory'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkUpdateDescriptorSets = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkUpdateDescriptorSets>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkUpdateDescriptorSets\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkUpdateDescriptorSets.is_none() {
            WARN!("Could not get device proc addr for 'vkUpdateDescriptorSets'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkWaitForFences = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkWaitForFences>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkWaitForFences\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkWaitForFences.is_none() {
            WARN!("Could not get device proc addr for 'vkWaitForFences'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        // VK_DEVICE_EXT_PFN entries (unchecked)
        (*device_vk).vk_info.vk_ops.vkCmdSetDepthCompareOpEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetDepthCompareOpEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetDepthCompareOpEXT\0".as_ptr() as *const core::ffi::c_char));
        (*device_vk).vk_info.vk_ops.vkCmdSetDepthTestEnableEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetDepthTestEnableEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetDepthTestEnableEXT\0".as_ptr() as *const core::ffi::c_char));
        (*device_vk).vk_info.vk_ops.vkCmdSetDepthWriteEnableEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetDepthWriteEnableEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetDepthWriteEnableEXT\0".as_ptr() as *const core::ffi::c_char));
        (*device_vk).vk_info.vk_ops.vkCmdSetPrimitiveTopologyEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetPrimitiveTopologyEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetPrimitiveTopologyEXT\0".as_ptr() as *const core::ffi::c_char));
        (*device_vk).vk_info.vk_ops.vkCmdSetStencilOpEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetStencilOpEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetStencilOpEXT\0".as_ptr() as *const core::ffi::c_char));
        (*device_vk).vk_info.vk_ops.vkCmdSetStencilTestEnableEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetStencilTestEnableEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetStencilTestEnableEXT\0".as_ptr() as *const core::ffi::c_char));
        (*device_vk).vk_info.vk_ops.vkCmdSetPatchControlPointsEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetPatchControlPointsEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetPatchControlPointsEXT\0".as_ptr() as *const core::ffi::c_char));
        (*device_vk).vk_info.vk_ops.vkCmdSetPrimitiveRestartEnableEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetPrimitiveRestartEnableEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetPrimitiveRestartEnableEXT\0".as_ptr() as *const core::ffi::c_char));
        (*device_vk).vk_info.vk_ops.vkCmdSetAlphaToCoverageEnableEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetAlphaToCoverageEnableEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetAlphaToCoverageEnableEXT\0".as_ptr() as *const core::ffi::c_char));
        (*device_vk).vk_info.vk_ops.vkCmdSetColorBlendEnableEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetColorBlendEnableEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetColorBlendEnableEXT\0".as_ptr() as *const core::ffi::c_char));
        (*device_vk).vk_info.vk_ops.vkCmdSetColorBlendEquationEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetColorBlendEquationEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetColorBlendEquationEXT\0".as_ptr() as *const core::ffi::c_char));
        (*device_vk).vk_info.vk_ops.vkCmdSetColorWriteMaskEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetColorWriteMaskEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetColorWriteMaskEXT\0".as_ptr() as *const core::ffi::c_char));
        (*device_vk).vk_info.vk_ops.vkCmdSetCullModeEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetCullModeEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetCullModeEXT\0".as_ptr() as *const core::ffi::c_char));
        (*device_vk).vk_info.vk_ops.vkCmdSetDepthBiasEnableEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetDepthBiasEnableEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetDepthBiasEnableEXT\0".as_ptr() as *const core::ffi::c_char));
        (*device_vk).vk_info.vk_ops.vkCmdSetDepthClampEnableEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetDepthClampEnableEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetDepthClampEnableEXT\0".as_ptr() as *const core::ffi::c_char));
        (*device_vk).vk_info.vk_ops.vkCmdSetFrontFaceEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetFrontFaceEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetFrontFaceEXT\0".as_ptr() as *const core::ffi::c_char));
        (*device_vk).vk_info.vk_ops.vkCmdSetRasterizationSamplesEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetRasterizationSamplesEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetRasterizationSamplesEXT\0".as_ptr() as *const core::ffi::c_char));
        (*device_vk).vk_info.vk_ops.vkCmdSetRasterizerDiscardEnableEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetRasterizerDiscardEnableEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetRasterizerDiscardEnableEXT\0".as_ptr() as *const core::ffi::c_char));
        (*device_vk).vk_info.vk_ops.vkCmdSetSampleMaskEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetSampleMaskEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdSetSampleMaskEXT\0".as_ptr() as *const core::ffi::c_char));
        (*device_vk).vk_info.vk_ops.vkCmdBeginQueryIndexedEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdBeginQueryIndexedEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdBeginQueryIndexedEXT\0".as_ptr() as *const core::ffi::c_char));
        (*device_vk).vk_info.vk_ops.vkCmdBeginTransformFeedbackEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdBeginTransformFeedbackEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdBeginTransformFeedbackEXT\0".as_ptr() as *const core::ffi::c_char));
        (*device_vk).vk_info.vk_ops.vkCmdBindTransformFeedbackBuffersEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdBindTransformFeedbackBuffersEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdBindTransformFeedbackBuffersEXT\0".as_ptr() as *const core::ffi::c_char));
        (*device_vk).vk_info.vk_ops.vkCmdEndQueryIndexedEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdEndQueryIndexedEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdEndQueryIndexedEXT\0".as_ptr() as *const core::ffi::c_char));
        (*device_vk).vk_info.vk_ops.vkCmdEndTransformFeedbackEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdEndTransformFeedbackEXT>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCmdEndTransformFeedbackEXT\0".as_ptr() as *const core::ffi::c_char));
        // VK_DEVICE_PFN KHR entries (checked)
        (*device_vk).vk_info.vk_ops.vkAcquireNextImageKHR = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkAcquireNextImageKHR>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkAcquireNextImageKHR\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkAcquireNextImageKHR.is_none() {
            WARN!("Could not get device proc addr for 'vkAcquireNextImageKHR'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkCreateSwapchainKHR = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateSwapchainKHR>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkCreateSwapchainKHR\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkCreateSwapchainKHR.is_none() {
            WARN!("Could not get device proc addr for 'vkCreateSwapchainKHR'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkDestroySwapchainKHR = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroySwapchainKHR>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkDestroySwapchainKHR\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkDestroySwapchainKHR.is_none() {
            WARN!("Could not get device proc addr for 'vkDestroySwapchainKHR'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkGetSwapchainImagesKHR = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetSwapchainImagesKHR>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkGetSwapchainImagesKHR\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkGetSwapchainImagesKHR.is_none() {
            WARN!("Could not get device proc addr for 'vkGetSwapchainImagesKHR'.\n");
            hr = E_FAIL;
            break 'fail;
        }
        (*device_vk).vk_info.vk_ops.vkQueuePresentKHR = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkQueuePresentKHR>((*vk_info).vk_ops.vkGetDeviceProcAddr.unwrap_unchecked()(vk_device, b"vkQueuePresentKHR\0".as_ptr() as *const core::ffi::c_char));
        if (*device_vk).vk_info.vk_ops.vkQueuePresentKHR.is_none() {
            WARN!("Could not get device proc addr for 'vkQueuePresentKHR'.\n");
            hr = E_FAIL;
            break 'fail;
        }

        if wined3d_allocator_init(
            core::ptr::addr_of_mut!((*device_vk).allocator),
            (*adapter_vk).memory_properties.memoryTypeCount as size_t,
            core::ptr::addr_of!(wined3d_allocator_vk_ops),
        ) == false
        {
            WARN!("Failed to initialise allocator.\n");
            hr = E_FAIL;
            break 'fail;
        }

        hr = wined3d_device_init(
            core::ptr::addr_of_mut!((*device_vk).d),
            wined3d,
            (*adapter).ordinal,
            device_type,
            focus_window,
            flags,
            surface_alignment,
            levels,
            level_count,
            core::ptr::addr_of!((*vk_info).supported) as *const BOOL,
            device_parent,
        );
        if hr < 0 {
            WARN!("Failed to initialize device, hr %#lx.\n", hr);
            wined3d_allocator_cleanup(core::ptr::addr_of_mut!((*device_vk).allocator));
            break 'fail;
        }

        wined3d_lock_init!(
            core::ptr::addr_of_mut!((*device_vk).allocator_cs),
            b"wined3d_device_vk.allocator_cs\0".as_ptr() as *const core::ffi::c_char,
        );

        *device = core::ptr::addr_of_mut!((*device_vk).d) as *mut wined3d_device;

        return WINED3D_OK;
    }

    // fail:
    ((*vk_info).vk_ops.vkDestroyDevice.unwrap_unchecked())(vk_device, core::ptr::null());
    free(device_vk as *mut core::ffi::c_void);
    return hr;
}

unsafe extern "C" fn adapter_vk_destroy_device(device: *mut wined3d_device) {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk(device);
    let vk_info: *const wined3d_vk_info = core::ptr::addr_of!((*device_vk).vk_info);

    wined3d_device_cleanup(core::ptr::addr_of_mut!((*device_vk).d));
    wined3d_allocator_cleanup(core::ptr::addr_of_mut!((*device_vk).allocator));

    wined3d_lock_cleanup(core::ptr::addr_of_mut!((*device_vk).allocator_cs));

    (*vk_info)
        .vk_ops
        .vkDestroyDevice
        .unwrap_unchecked()(
        (*device_vk).vk_device,
        core::ptr::null::<VkAllocationCallbacks>(),
    );
    free(device_vk as *mut core::ffi::c_void);
}

unsafe extern "C" fn adapter_vk_acquire_context(
    device: *mut wined3d_device,
    texture: *mut wined3d_texture,
    sub_resource_idx: ::core::ffi::c_uint,
) -> *mut wined3d_context {
    wined3d_from_cs((*device).cs);

    if (*device).context_count == 0 {
        return core::ptr::null_mut();
    }

    return &raw mut (*wined3d_device_vk(device)).context_vk.c;
}

unsafe extern "C" fn adapter_vk_release_context(context: *mut wined3d_context) {
    // TRACE("context %p.\n", context);
}

unsafe extern "C" fn adapter_vk_get_wined3d_caps(adapter: *const wined3d_adapter, caps: *mut wined3d_caps) {
    let adapter_vk: *const wined3d_adapter_vk = wined3d_adapter_vk_const(adapter);
    let limits: *const VkPhysicalDeviceLimits = &raw const (*adapter_vk).device_limits;
    let sampler_anisotropy: bool = (*limits).maxSamplerAnisotropy > 1.0f32;
    let vk_info: *const wined3d_vk_info = &raw const (*adapter_vk).vk_info;
    (*caps).ddraw_caps.dds_caps |= (WINEDDSCAPS_BACKBUFFER
        | WINEDDSCAPS_COMPLEX
        | WINEDDSCAPS_FRONTBUFFER
        | WINEDDSCAPS_3DDEVICE
        | WINEDDSCAPS_VIDEOMEMORY
        | WINEDDSCAPS_OWNDC
        | WINEDDSCAPS_LOCALVIDMEM
        | WINEDDSCAPS_NONLOCALVIDMEM) as DWORD;
    (*caps).ddraw_caps.caps |= WINEDDCAPS_3D as DWORD;
    (*caps).Caps2 |= WINED3DCAPS2_CANGENMIPMAP as DWORD;
    (*caps).PrimitiveMiscCaps |= (WINED3DPMISCCAPS_BLENDOP
        | WINED3DPMISCCAPS_INDEPENDENTWRITEMASKS
        | WINED3DPMISCCAPS_MRTINDEPENDENTBITDEPTHS
        | WINED3DPMISCCAPS_POSTBLENDSRGBCONVERT
        | WINED3DPMISCCAPS_SEPARATEALPHABLEND) as DWORD;
    (*caps).RasterCaps |= WINED3DPRASTERCAPS_MIPMAPLODBIAS as DWORD;
    if sampler_anisotropy {
        (*caps).RasterCaps |= WINED3DPRASTERCAPS_ANISOTROPY as DWORD;
        (*caps).TextureFilterCaps |= (WINED3DPTFILTERCAPS_MAGFANISOTROPIC
            | WINED3DPTFILTERCAPS_MINFANISOTROPIC) as DWORD;
        (*caps).MaxAnisotropy = (*limits).maxSamplerAnisotropy as DWORD;
    }
    (*caps).SrcBlendCaps |= WINED3DPBLENDCAPS_BLENDFACTOR as DWORD;
    (*caps).DestBlendCaps |= (WINED3DPBLENDCAPS_BLENDFACTOR
        | WINED3DPBLENDCAPS_SRCALPHASAT) as DWORD;
    (*caps).TextureCaps |= (WINED3DPTEXTURECAPS_VOLUMEMAP
        | WINED3DPTEXTURECAPS_MIPVOLUMEMAP
        | WINED3DPTEXTURECAPS_VOLUMEMAP_POW2) as DWORD;
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
    (*caps).VolumeTextureAddressCaps |= (WINED3DPTADDRESSCAPS_BORDER
        | WINED3DPTADDRESSCAPS_MIRROR) as DWORD;
    if (*vk_info).supported[WINED3D_VK_KHR_SAMPLER_MIRROR_CLAMP_TO_EDGE as usize] != 0 {
        (*caps).VolumeTextureAddressCaps |= WINED3DPTADDRESSCAPS_MIRRORONCE as DWORD;
    }
    (*caps).MaxVolumeExtent = (*limits).maxImageDimension3D as DWORD;
    (*caps).TextureCaps |= (WINED3DPTEXTURECAPS_CUBEMAP
        | WINED3DPTEXTURECAPS_MIPCUBEMAP
        | WINED3DPTEXTURECAPS_CUBEMAP_POW2) as DWORD;
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
    if sampler_anisotropy {
        (*caps).CubeTextureFilterCaps |= (WINED3DPTFILTERCAPS_MAGFANISOTROPIC
            | WINED3DPTFILTERCAPS_MINFANISOTROPIC) as DWORD;
    }
    (*caps).TextureAddressCaps |= (WINED3DPTADDRESSCAPS_BORDER
        | WINED3DPTADDRESSCAPS_MIRROR) as DWORD;
    if (*vk_info).supported[WINED3D_VK_KHR_SAMPLER_MIRROR_CLAMP_TO_EDGE as usize] != 0 {
        (*caps).TextureAddressCaps |= WINED3DPTADDRESSCAPS_MIRRORONCE as DWORD;
    }
    (*caps).StencilCaps |= (WINED3DSTENCILCAPS_DECR
        | WINED3DSTENCILCAPS_INCR
        | WINED3DSTENCILCAPS_TWOSIDED) as DWORD;
    (*caps).DeclTypes |= (WINED3DDTCAPS_FLOAT16_2 | WINED3DDTCAPS_FLOAT16_4) as DWORD;
    (*caps).MaxPixelShader30InstructionSlots = WINED3DMAX30SHADERINSTRUCTIONS as DWORD;
    (*caps).MaxVertexShader30InstructionSlots = WINED3DMAX30SHADERINSTRUCTIONS as DWORD;
    (*caps).PS20Caps.temp_count = WINED3DPS20_MAX_NUMTEMPS as INT;
    (*caps).VS20Caps.temp_count = WINED3DVS20_MAX_NUMTEMPS as INT;
}

unsafe extern "C" fn adapter_vk_check_format(
    adapter: *const wined3d_adapter,
    adapter_format: *const wined3d_format,
    rt_format: *const wined3d_format,
    ds_format: *const wined3d_format,
) -> BOOL {
    (TRUE as i32)
}

unsafe extern "C" fn adapter_vk_init_3d(device: *mut wined3d_device) -> HRESULT {
    let mut context_vk: *mut wined3d_context_vk = core::ptr::null_mut();
    let mut device_vk: *mut wined3d_device_vk = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    device_vk = wined3d_device_vk(device);
    context_vk = core::ptr::addr_of_mut!((*device_vk).context_vk);
    hr = wined3d_context_vk_init(context_vk, *(*device).swapchains.add(0 as usize));
    if hr < 0 {
        WARN!("Failed to initialise context.\n");
        return hr;
    }

    hr = (*(*device).shader_backend).shader_alloc_private.unwrap_unchecked()(
        device,
        (*(*device).adapter).vertex_pipe,
        (*(*device).adapter).fragment_pipe,
    );
    if hr < 0 {
        ERR!("Failed to allocate shader private data, hr %#lx.\n", hr);
        wined3d_context_vk_cleanup(context_vk);
        return hr;
    }

    if device_context_add(device, core::ptr::addr_of_mut!((*context_vk).c)) == 0 {
        ERR!("Failed to add the newly created context to the context list.\n");
        (*(*device).shader_backend).shader_free_private.unwrap_unchecked()(
            device,
            core::ptr::null_mut(),
        );
        wined3d_context_vk_cleanup(context_vk);
        return E_FAIL;
    }

    (*device_vk).d.blitter = wined3d_cpu_blitter_create();
    if (*device_vk).d.blitter.is_null() {
        ERR!("Failed to create CPU blitter.\n");
        device_context_remove(device, core::ptr::addr_of_mut!((*context_vk).c));
        (*(*device).shader_backend).shader_free_private.unwrap_unchecked()(
            device,
            core::ptr::null_mut(),
        );
        wined3d_context_vk_cleanup(context_vk);
        return E_FAIL;
    }
    wined3d_vk_blitter_create(core::ptr::addr_of_mut!((*device_vk).d.blitter));

    wined3d_device_create_default_samplers(device, core::ptr::addr_of_mut!((*context_vk).c));
    wined3d_device_vk_create_null_resources(device_vk, context_vk);
    wined3d_device_vk_create_null_views(device_vk, context_vk);
    if (*(*device).adapter).d3d_info.feature_level as u32 >= WINED3D_FEATURE_LEVEL_11 as u32 {
        wined3d_device_vk_uav_clear_state_init(device_vk);
    }

    return WINED3D_OK;
}

unsafe extern "C" fn adapter_vk_uninit_3d_cs(mut object: *mut ::core::ffi::c_void) {
    let mut device_vk: *mut wined3d_device_vk = object as *mut wined3d_device_vk;
    let mut context_vk: *mut wined3d_context_vk = ::core::ptr::null_mut::<wined3d_context_vk>();
    let mut device: *mut wined3d_device = ::core::ptr::null_mut::<wined3d_device>();
    let mut shader: *mut wined3d_shader = ::core::ptr::null_mut::<wined3d_shader>();
    WINE_TRACE!("device_vk %p.\n", device_vk);
    context_vk = &raw mut (*device_vk).context_vk;
    device = &raw mut (*device_vk).d;
    shader = ((*device).shaders.next as *mut ::core::ffi::c_char).offset(
        -(core::mem::offset_of!(wined3d_shader, shader_list_entry)
            as ::core::ffi::c_ulonglong as isize),
    ) as *mut wined3d_shader;
    while &raw mut (*shader).shader_list_entry != &raw mut (*device).shaders {
        (*(*device).shader_backend)
            .shader_destroy
            .unwrap_unchecked()(shader);
        shader = ((*shader).shader_list_entry.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(wined3d_shader, shader_list_entry)
                as ::core::ffi::c_ulonglong as isize),
        ) as *mut wined3d_shader;
    }
    if (*(*device).adapter).d3d_info.feature_level as ::core::ffi::c_uint
        >= WINED3D_FEATURE_LEVEL_11 as ::core::ffi::c_uint
    {
        wined3d_device_vk_uav_clear_state_cleanup(device_vk);
    }
    (*(*(*device).blitter).ops)
        .blitter_destroy
        .unwrap_unchecked()(
        (*device).blitter,
        ::core::ptr::null_mut::<wined3d_context>(),
    );
    (*(*device).shader_backend)
        .shader_free_private
        .unwrap_unchecked()(device, &raw mut (*context_vk).c);
    wined3d_device_vk_destroy_null_views(device_vk, context_vk);
    wined3d_device_vk_destroy_null_resources(device_vk, context_vk);
}

unsafe extern "C" fn adapter_vk_uninit_3d(device: *mut wined3d_device) {
    let mut context_vk: *mut wined3d_context_vk = core::ptr::null_mut::<wined3d_context_vk>();
    let mut device_vk: *mut wined3d_device_vk = core::ptr::null_mut::<wined3d_device_vk>();
    TRACE!("device %p.\n", device);
    device_vk = wined3d_device_vk(device);
    context_vk = core::ptr::addr_of_mut!((*device_vk).context_vk);

    wined3d_device_destroy_default_samplers(device);
    wined3d_cs_destroy_object(
        (*device).cs as *mut wined3d_cs,
        Some(adapter_vk_uninit_3d_cs as unsafe extern "C" fn(*mut core::ffi::c_void) -> ()),
        device_vk as *mut core::ffi::c_void,
    );
    wined3d_cs_finish((*device).cs as *mut wined3d_cs, WINED3D_CS_QUEUE_DEFAULT);

    device_context_remove(device, core::ptr::addr_of_mut!((*context_vk).c));
    wined3d_context_vk_cleanup(context_vk);
}

unsafe extern "C" fn wined3d_bo_vk_map(
    bo: *mut wined3d_bo_vk,
    context_vk: *mut wined3d_context_vk,
) -> *mut ::core::ffi::c_void {
    let mut vk_info: *const wined3d_vk_info;
    let mut device_vk: *mut wined3d_device_vk;
    let mut slab: *mut wined3d_bo_slab_vk;
    let mut vr: VkResult;

    if !(*bo).b.map_ptr.is_null() {
        return (*bo).b.map_ptr;
    }

    vk_info = (*context_vk).vk_info;
    device_vk = wined3d_device_vk((*context_vk).c.device);

    slab = (*bo).slab as *mut wined3d_bo_slab_vk;
    if !slab.is_null() {
        (*bo).b.map_ptr = wined3d_bo_slab_vk_map(slab, context_vk);
        if (*bo).b.map_ptr.is_null() {
            ERR!("Failed to map slab.\n");
            return core::ptr::null_mut();
        }
    } else if !(*bo).memory.is_null() {
        let chunk_vk: *mut wined3d_allocator_chunk_vk =
            wined3d_allocator_chunk_vk((*(*bo).memory).chunk);

        (*bo).b.map_ptr = wined3d_allocator_chunk_vk_map(chunk_vk, context_vk);
        if (*bo).b.map_ptr.is_null() {
            ERR!("Failed to map chunk.\n");
            return core::ptr::null_mut();
        }
    } else {
        vr = ((*vk_info).vk_ops.vkMapMemory.unwrap_unchecked())(
            (*device_vk).vk_device,
            (*bo).vk_memory,
            0,
            (VK_WHOLE_SIZE) as u64,
            0,
            core::ptr::addr_of_mut!((*bo).b.map_ptr),
        );
        if (vr as ::core::ffi::c_int) < 0 {
            ERR!("Failed to map memory, vr %s.\n", wined3d_debug_vkresult(vr));
            return core::ptr::null_mut();
        }

        adapter_adjust_mapped_memory((*device_vk).d.adapter, (*bo).size as ssize_t);
    }

    return (*bo).b.map_ptr;
}

unsafe extern "C" fn wined3d_bo_vk_unmap(
    bo: *mut wined3d_bo_vk,
    context_vk: *mut wined3d_context_vk,
) {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let mut vk_info: *const wined3d_vk_info = core::ptr::null();
    let mut slab: *mut wined3d_bo_slab_vk = core::ptr::null_mut();

    /* This may race with the client thread, but it's not a hard limit anyway. */
    if (*(*device_vk).d.adapter).mapped_size <= MAX_PERSISTENT_MAPPED_BYTES as ssize_t {
        return;
    }

    wined3d_device_bo_map_lock((*context_vk).c.device);
    /* The mapping is still in use by the client (viz. for an accelerated
     * NOOVERWRITE map). The client will trigger another unmap request when the
     * d3d application requests to unmap the BO. */
    if (*bo).b.client_map_count != 0 {
        wined3d_device_bo_map_unlock((*context_vk).c.device);
        return;
    }
    (*bo).b.map_ptr = core::ptr::null_mut();
    wined3d_device_bo_map_unlock((*context_vk).c.device);

    slab = (*bo).slab as *mut wined3d_bo_slab_vk;
    if !slab.is_null() {
        wined3d_bo_slab_vk_unmap(slab, context_vk);
        return;
    }

    if !(*bo).memory.is_null() {
        wined3d_allocator_chunk_vk_unmap(wined3d_allocator_chunk_vk((*(*bo).memory).chunk), context_vk);
        return;
    }

    vk_info = (*context_vk).vk_info;
    ((*vk_info).vk_ops.vkUnmapMemory.unwrap_unchecked())((*device_vk).vk_device, (*bo).vk_memory);
    adapter_adjust_mapped_memory((*device_vk).d.adapter, (*bo).size.wrapping_neg() as ssize_t);
}

unsafe extern "C" fn wined3d_bo_slab_vk_lock(
    mut slab_vk: *mut wined3d_bo_slab_vk,
    mut context_vk: *mut wined3d_context_vk,
) {
    let _ = slab_vk;
    wined3d_device_vk_allocator_lock(wined3d_device_vk((*context_vk).c.device));
}

unsafe extern "C" fn wined3d_bo_slab_vk_unlock(
    slab_vk: *mut wined3d_bo_slab_vk,
    context_vk: *mut wined3d_context_vk,
) {
    let _ = slab_vk;
    wined3d_device_vk_allocator_unlock(wined3d_device_vk((*context_vk).c.device));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_bo_slab_vk_map(slab_vk: *mut wined3d_bo_slab_vk, context_vk: *mut wined3d_context_vk) -> *mut ::core::ffi::c_void {
    let mut map_ptr: *mut ::core::ffi::c_void = core::ptr::null_mut();

    wined3d_bo_slab_vk_lock(slab_vk, context_vk);

    if (*slab_vk).map_ptr.is_null() && {
        (*slab_vk).map_ptr = wined3d_bo_vk_map(core::ptr::addr_of_mut!((*slab_vk).bo), context_vk);
        (*slab_vk).map_ptr.is_null()
    } {
        wined3d_bo_slab_vk_unlock(slab_vk, context_vk);
        ERR!("Failed to map slab.\n");
        return core::ptr::null_mut();
    }

    (*slab_vk).map_count = (*slab_vk).map_count.wrapping_add(1);
    map_ptr = (*slab_vk).map_ptr;

    wined3d_bo_slab_vk_unlock(slab_vk, context_vk);

    return map_ptr;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_bo_slab_vk_unmap(slab_vk: *mut wined3d_bo_slab_vk, context_vk: *mut wined3d_context_vk) {
    wined3d_bo_slab_vk_lock(slab_vk, context_vk);

    (*slab_vk).map_count = (*slab_vk).map_count.wrapping_sub(1);
    if (*slab_vk).map_count != 0 {
        wined3d_bo_slab_vk_unlock(slab_vk, context_vk);
        return;
    }

    wined3d_bo_vk_unmap(core::ptr::addr_of_mut!((*slab_vk).bo), context_vk);
    (*slab_vk).map_ptr = core::ptr::null_mut();

    wined3d_bo_slab_vk_unlock(slab_vk, context_vk);
}
#[no_mangle]
pub unsafe extern "C" fn vk_access_mask_from_buffer_usage(
    mut usage: VkBufferUsageFlags,
) -> VkAccessFlags {
    let mut flags: VkAccessFlags = 0 as VkAccessFlags;
    if usage & VK_BUFFER_USAGE_VERTEX_BUFFER_BIT as VkBufferUsageFlags != 0 {
        flags |= VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT as VkAccessFlags;
    }
    if usage & VK_BUFFER_USAGE_INDEX_BUFFER_BIT as VkBufferUsageFlags != 0 {
        flags |= VK_ACCESS_INDEX_READ_BIT as VkAccessFlags;
    }
    if usage & VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT as VkBufferUsageFlags != 0 {
        flags |= VK_ACCESS_UNIFORM_READ_BIT as VkAccessFlags;
    }
    if usage & VK_BUFFER_USAGE_UNIFORM_TEXEL_BUFFER_BIT as VkBufferUsageFlags != 0 {
        flags |= VK_ACCESS_SHADER_READ_BIT as VkAccessFlags;
    }
    if usage & VK_BUFFER_USAGE_STORAGE_TEXEL_BUFFER_BIT as VkBufferUsageFlags != 0 {
        flags |= (VK_ACCESS_SHADER_READ_BIT as VkAccessFlags) | (VK_ACCESS_SHADER_WRITE_BIT as VkAccessFlags);
    }
    if usage & VK_BUFFER_USAGE_INDIRECT_BUFFER_BIT as VkBufferUsageFlags != 0 {
        flags |= VK_ACCESS_INDIRECT_COMMAND_READ_BIT as VkAccessFlags;
    }
    if usage & VK_BUFFER_USAGE_TRANSFORM_FEEDBACK_BUFFER_BIT_EXT as VkBufferUsageFlags != 0 {
        flags |= VK_ACCESS_TRANSFORM_FEEDBACK_WRITE_BIT_EXT as VkAccessFlags;
    }
    if usage & VK_BUFFER_USAGE_TRANSFORM_FEEDBACK_COUNTER_BUFFER_BIT_EXT as VkBufferUsageFlags != 0 {
        flags |= (VK_ACCESS_TRANSFORM_FEEDBACK_COUNTER_READ_BIT_EXT as VkAccessFlags) | (VK_ACCESS_TRANSFORM_FEEDBACK_COUNTER_WRITE_BIT_EXT as VkAccessFlags);
    }
    if usage & VK_BUFFER_USAGE_TRANSFER_SRC_BIT as VkBufferUsageFlags != 0 {
        flags |= VK_ACCESS_TRANSFER_READ_BIT as VkAccessFlags;
    }
    if usage & VK_BUFFER_USAGE_TRANSFER_DST_BIT as VkBufferUsageFlags != 0 {
        flags |= VK_ACCESS_TRANSFER_WRITE_BIT as VkAccessFlags;
    }
    return flags;
}
#[no_mangle]
pub unsafe extern "C" fn vk_pipeline_stage_mask_from_buffer_usage(usage: VkBufferUsageFlags) -> VkPipelineStageFlags {
    let mut flags: VkPipelineStageFlags = 0 as VkPipelineStageFlags;

    if usage & ((VK_BUFFER_USAGE_VERTEX_BUFFER_BIT | VK_BUFFER_USAGE_INDEX_BUFFER_BIT) as VkBufferUsageFlags) != 0 {
        flags |= VK_PIPELINE_STAGE_VERTEX_INPUT_BIT as VkPipelineStageFlags;
    }
    if usage & ((VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT | VK_BUFFER_USAGE_UNIFORM_TEXEL_BUFFER_BIT | VK_BUFFER_USAGE_STORAGE_TEXEL_BUFFER_BIT) as VkBufferUsageFlags) != 0 {
        flags |= (VK_PIPELINE_STAGE_VERTEX_SHADER_BIT | VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT
            | VK_PIPELINE_STAGE_COMPUTE_SHADER_BIT | VK_PIPELINE_STAGE_GEOMETRY_SHADER_BIT
            | VK_PIPELINE_STAGE_TESSELLATION_EVALUATION_SHADER_BIT
            | VK_PIPELINE_STAGE_TESSELLATION_CONTROL_SHADER_BIT) as VkPipelineStageFlags;
    }
    if usage & (VK_BUFFER_USAGE_INDIRECT_BUFFER_BIT as VkBufferUsageFlags) != 0 {
        flags |= VK_PIPELINE_STAGE_DRAW_INDIRECT_BIT as VkPipelineStageFlags;
    }
    if usage & ((VK_BUFFER_USAGE_TRANSFORM_FEEDBACK_BUFFER_BIT_EXT | VK_BUFFER_USAGE_TRANSFORM_FEEDBACK_COUNTER_BUFFER_BIT_EXT) as VkBufferUsageFlags) != 0 {
        flags |= VK_PIPELINE_STAGE_TRANSFORM_FEEDBACK_BIT_EXT as VkPipelineStageFlags;
    }
    if usage & ((VK_BUFFER_USAGE_TRANSFER_SRC_BIT | VK_BUFFER_USAGE_TRANSFER_DST_BIT) as VkBufferUsageFlags) != 0 {
        flags |= VK_PIPELINE_STAGE_TRANSFER_BIT as VkPipelineStageFlags;
    }

    return flags;
}

unsafe extern "C" fn adapter_vk_map_bo_address(
    mut context: *mut wined3d_context,
    mut data: *const wined3d_bo_address,
    mut size: size_t,
    mut map_flags: uint32_t,
) -> *mut ::core::ffi::c_void {
    let mut context_vk: *mut wined3d_context_vk = wined3d_context_vk(context);
    let mut vk_info: *const wined3d_vk_info = core::ptr::null();
    let mut device_vk: *mut wined3d_device_vk = core::ptr::null_mut();
    let mut vk_command_buffer: VkCommandBuffer = core::ptr::null_mut();
    let mut vk_barrier: VkBufferMemoryBarrier = core::mem::zeroed();
    let mut bo_user: *mut wined3d_bo_user = core::ptr::null_mut();
    let mut bo: *mut wined3d_bo_vk = core::ptr::null_mut();
    let mut tmp: wined3d_bo_vk = core::mem::zeroed();
    let mut range: VkMappedMemoryRange = core::mem::zeroed();
    let mut map_ptr: *mut ::core::ffi::c_void = core::ptr::null_mut();

    if (*data).buffer_object.is_null() {
        return (*data).addr as *mut ::core::ffi::c_void;
    }
    bo = wined3d_bo_vk((*data).buffer_object);

    vk_info = (*context_vk).vk_info;
    device_vk = wined3d_device_vk((*context).device);

    'map: {
        if (map_flags & WINED3D_MAP_NOOVERWRITE) != 0 {
            break 'map;
        }

        if ((map_flags & WINED3D_MAP_DISCARD) != 0)
            && (*bo).command_buffer_id > (*context_vk).completed_command_buffer_id
        {
            if wined3d_context_vk_create_bo(
                context_vk,
                (*bo).size,
                (*bo).usage,
                (*bo).memory_type,
                &raw mut tmp,
            ) != 0
            {
                let mut host_synced: bool = (*bo).host_synced;

                // LIST_FOR_EACH_ENTRY(bo_user, &bo->b.users, struct wined3d_bo_user, entry)
                bo_user = ((*bo).b.users.next as *mut u8).offset(
                    -(core::mem::offset_of!(wined3d_bo_user, entry) as isize),
                ) as *mut wined3d_bo_user;
                while &raw mut (*bo_user).entry != &raw mut (*bo).b.users {
                    (*bo_user).valid = false;
                    bo_user = ((*bo_user).entry.next as *mut u8).offset(
                        -(core::mem::offset_of!(wined3d_bo_user, entry) as isize),
                    ) as *mut wined3d_bo_user;
                }
                list_init(&raw mut (*bo).b.users);

                wined3d_context_vk_destroy_bo(context_vk, bo);
                *bo = tmp;
                (*bo).host_synced = host_synced;
                list_init(&raw mut (*bo).b.users);

                break 'map;
            }

            ERR!("Failed to create new buffer object.\n");
        }

        if (map_flags & WINED3D_MAP_READ) != 0 {
            if !(*bo).host_synced {
                vk_command_buffer = wined3d_context_vk_get_command_buffer(context_vk);
                if vk_command_buffer.is_null() {
                    ERR!("Failed to get command buffer.\n");
                    return core::ptr::null_mut();
                }

                wined3d_context_vk_end_current_render_pass(context_vk);

                vk_barrier.sType = VK_STRUCTURE_TYPE_BUFFER_MEMORY_BARRIER;
                vk_barrier.pNext = core::ptr::null();
                vk_barrier.srcAccessMask = vk_access_mask_from_buffer_usage((*bo).usage);
                vk_barrier.dstAccessMask = VK_ACCESS_HOST_READ_BIT;
                vk_barrier.srcQueueFamilyIndex = (VK_QUEUE_FAMILY_IGNORED) as u32;
                vk_barrier.dstQueueFamilyIndex = (VK_QUEUE_FAMILY_IGNORED) as u32;
                vk_barrier.buffer = (*bo).vk_buffer;
                vk_barrier.offset = ((*bo).b.buffer_offset as usize)
                    .wrapping_add((*data).addr as usize) as VkDeviceSize;
                vk_barrier.size = size as VkDeviceSize;
                ((*vk_info).vk_ops.vkCmdPipelineBarrier.unwrap_unchecked())(
                    vk_command_buffer,
                    VK_PIPELINE_STAGE_ALL_COMMANDS_BIT,
                    VK_PIPELINE_STAGE_HOST_BIT,
                    0,
                    0,
                    core::ptr::null(),
                    1,
                    &raw mut vk_barrier,
                    0,
                    core::ptr::null(),
                );

                wined3d_context_vk_reference_bo(context_vk, bo);
            }

            if !(*bo).b.coherent {
                range.sType = VK_STRUCTURE_TYPE_MAPPED_MEMORY_RANGE;
                range.pNext = core::ptr::null();
                range.memory = (*bo).vk_memory;
                range.offset = ((*bo).b.memory_offset as usize)
                    .wrapping_add((*data).addr as usize) as VkDeviceSize;
                range.size = size as VkDeviceSize;
                ((*vk_info).vk_ops.vkInvalidateMappedMemoryRanges.unwrap_unchecked())(
                    (*device_vk).vk_device,
                    1,
                    &raw mut range,
                );
            }
        }

        if (*bo).command_buffer_id == (*context_vk).current_command_buffer.id {
            wined3d_context_vk_submit_command_buffer(
                context_vk,
                0,
                core::ptr::null(),
                core::ptr::null(),
                0,
                core::ptr::null(),
            );
        }
        wined3d_context_vk_wait_command_buffer(context_vk, (*bo).command_buffer_id);
    }

    // map:
    map_ptr = wined3d_bo_vk_map(bo, context_vk);
    if map_ptr.is_null() {
        ERR!("Failed to map bo.\n");
        return core::ptr::null_mut();
    }

    return (map_ptr as *mut u8)
        .offset((*bo).b.memory_offset as isize)
        .offset((*data).addr as usize as isize) as *mut ::core::ffi::c_void;
}

unsafe extern "C" fn flush_bo_range(
    mut context_vk: *mut wined3d_context_vk,
    mut bo: *mut wined3d_bo_vk,
    mut offset: ::core::ffi::c_uint,
    mut size: ::core::ffi::c_uint,
) {
    let mut device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let mut vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut range: VkMappedMemoryRange = VkMappedMemoryRange {
        sType: VK_STRUCTURE_TYPE_MAPPED_MEMORY_RANGE,
        pNext: ::core::ptr::null::<::core::ffi::c_void>(),
        memory: 0,
        offset: 0,
        size: 0,
    };
    range.sType = VK_STRUCTURE_TYPE_MAPPED_MEMORY_RANGE;
    range.pNext = ::core::ptr::null::<::core::ffi::c_void>();
    range.memory = (*bo).vk_memory;
    range.offset = (*bo).b.memory_offset.wrapping_add(offset as size_t) as VkDeviceSize;
    range.size = size as VkDeviceSize;
    ((*vk_info).vk_ops.vkFlushMappedMemoryRanges.unwrap_unchecked())((*device_vk).vk_device, 1, &raw mut range);
}

unsafe extern "C" fn adapter_vk_unmap_bo_address(context: *mut wined3d_context, data: *const wined3d_bo_address, range_count: ::core::ffi::c_uint, ranges: *const wined3d_range) {
    let mut context_vk: *mut wined3d_context_vk = wined3d_context_vk(context);
    let mut bo: *mut wined3d_bo_vk = ::core::ptr::null_mut::<wined3d_bo_vk>();
    let mut i: ::core::ffi::c_uint = 0;

    if (*data).buffer_object.is_null() {
        return;
    }
    bo = wined3d_bo_vk((*data).buffer_object);

    assert!(!(*bo).b.map_ptr.is_null());

    if !(*bo).b.coherent {
        i = 0;
        while i < range_count {
            flush_bo_range(
                context_vk,
                bo,
                (*ranges.offset(i as isize)).offset,
                (*ranges.offset(i as isize)).size,
            );
            i = i.wrapping_add(1);
        }
    }

    wined3d_bo_vk_unmap(bo, context_vk);
}
#[no_mangle]
pub unsafe extern "C" fn adapter_vk_copy_bo_address(context: *mut wined3d_context, dst: *const wined3d_bo_address, src: *const wined3d_bo_address, range_count: ::core::ffi::c_uint, ranges: *const wined3d_range, map_flags: uint32_t) {
    let context_vk: *mut wined3d_context_vk = wined3d_context_vk(context);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut staging_bo: wined3d_bo_vk = core::mem::zeroed();
    let mut src_bo: *mut wined3d_bo_vk = core::ptr::null_mut();
    let mut dst_bo: *mut wined3d_bo_vk = core::ptr::null_mut();
    let mut src_access_mask: VkAccessFlags = 0;
    let mut dst_access_mask: VkAccessFlags = 0;
    let mut vk_barrier: [VkBufferMemoryBarrier; 2] = core::mem::zeroed();
    let mut range: *const wined3d_range = core::ptr::null();
    let mut staging: wined3d_bo_address = core::mem::zeroed();
    let mut vk_command_buffer: VkCommandBuffer = core::ptr::null_mut();
    let mut dst_ptr: *mut uint8_t = core::ptr::null_mut();
    let mut src_ptr: *mut uint8_t = core::ptr::null_mut();
    let mut region: VkBufferCopy = core::mem::zeroed();
    let mut size: size_t = 0;
    let mut i: core::ffi::c_uint;

    src_bo = if !(*src).buffer_object.is_null() {
        wined3d_bo_vk((*src).buffer_object)
    } else {
        core::ptr::null_mut()
    };
    dst_bo = if !(*dst).buffer_object.is_null() {
        wined3d_bo_vk((*dst).buffer_object)
    } else {
        core::ptr::null_mut()
    };

    if !src_bo.is_null() && !dst_bo.is_null() {
        vk_command_buffer = wined3d_context_vk_get_command_buffer(context_vk);
        if vk_command_buffer.is_null() {
            ERR!("Failed to get command buffer.\n");
            return;
        }

        wined3d_context_vk_end_current_render_pass(context_vk);

        src_access_mask = vk_access_mask_from_buffer_usage((*src_bo).usage);
        dst_access_mask = vk_access_mask_from_buffer_usage((*dst_bo).usage);

        vk_barrier[0].sType = VK_STRUCTURE_TYPE_BUFFER_MEMORY_BARRIER;
        vk_barrier[0].pNext = core::ptr::null();
        vk_barrier[0].srcQueueFamilyIndex = (VK_QUEUE_FAMILY_IGNORED) as u32;
        vk_barrier[0].dstQueueFamilyIndex = (VK_QUEUE_FAMILY_IGNORED) as u32;
        vk_barrier[0].buffer = (*src_bo).vk_buffer;

        vk_barrier[1].sType = VK_STRUCTURE_TYPE_BUFFER_MEMORY_BARRIER;
        vk_barrier[1].pNext = core::ptr::null();
        vk_barrier[1].srcQueueFamilyIndex = (VK_QUEUE_FAMILY_IGNORED) as u32;
        vk_barrier[1].dstQueueFamilyIndex = (VK_QUEUE_FAMILY_IGNORED) as u32;
        vk_barrier[1].buffer = (*dst_bo).vk_buffer;

        i = 0;
        while i < range_count {
            range = ranges.add(i as usize);

            region.srcOffset = ((*src_bo).b.buffer_offset as u64)
                .wrapping_add((*src).addr as u64)
                .wrapping_add((*range).offset as u64) as VkDeviceSize;
            region.dstOffset = ((*dst_bo).b.buffer_offset as u64)
                .wrapping_add((*dst).addr as u64)
                .wrapping_add((*range).offset as u64) as VkDeviceSize;
            region.size = (*range).size as VkDeviceSize;

            vk_barrier[0].offset = region.srcOffset;
            vk_barrier[0].size = region.size;

            vk_barrier[1].offset = region.dstOffset;
            vk_barrier[1].size = region.size;

            vk_barrier[0].srcAccessMask = src_access_mask;
            vk_barrier[0].dstAccessMask = VK_ACCESS_TRANSFER_READ_BIT;

            vk_barrier[1].srcAccessMask = dst_access_mask;
            vk_barrier[1].dstAccessMask = VK_ACCESS_TRANSFER_WRITE_BIT;

            ((*vk_info).vk_ops.vkCmdPipelineBarrier.unwrap_unchecked())(vk_command_buffer, VK_PIPELINE_STAGE_ALL_COMMANDS_BIT,
                    VK_PIPELINE_STAGE_TRANSFER_BIT, 0, 0, core::ptr::null(), 2, vk_barrier.as_mut_ptr(), 0, core::ptr::null());

            ((*vk_info).vk_ops.vkCmdCopyBuffer.unwrap_unchecked())(vk_command_buffer, (*src_bo).vk_buffer, (*dst_bo).vk_buffer, 1, &mut region);

            vk_barrier[0].srcAccessMask = VK_ACCESS_TRANSFER_READ_BIT;
            vk_barrier[0].dstAccessMask = src_access_mask;

            vk_barrier[1].srcAccessMask = VK_ACCESS_TRANSFER_WRITE_BIT;
            vk_barrier[1].dstAccessMask = dst_access_mask;

            ((*vk_info).vk_ops.vkCmdPipelineBarrier.unwrap_unchecked())(vk_command_buffer, VK_PIPELINE_STAGE_TRANSFER_BIT,
                    VK_PIPELINE_STAGE_ALL_COMMANDS_BIT, 0, 0, core::ptr::null(), 2, vk_barrier.as_mut_ptr(), 0, core::ptr::null());

            i = i.wrapping_add(1);
        }

        wined3d_context_vk_reference_bo(context_vk, src_bo);
        wined3d_context_vk_reference_bo(context_vk, dst_bo);

        return;
    }

    i = 0;
    while i < range_count {
        size = max!(
            size,
            (*ranges.add(i as usize)).offset.wrapping_add((*ranges.add(i as usize)).size) as size_t,
        );
        i = i.wrapping_add(1);
    }

    if !src_bo.is_null() && ((*src_bo).memory_type & VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT) == 0 {
        if wined3d_context_vk_create_bo(context_vk, size as VkDeviceSize, VK_BUFFER_USAGE_TRANSFER_DST_BIT,
                VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT, &mut staging_bo) == 0
        {
            ERR!("Failed to create staging bo.\n");
            return;
        }

        staging.buffer_object = &mut staging_bo.b;
        staging.addr = core::ptr::null_mut();
        adapter_vk_copy_bo_address(context, &mut staging, src, range_count, ranges, WINED3D_MAP_WRITE);
        adapter_vk_copy_bo_address(context, dst, &mut staging, range_count, ranges, WINED3D_MAP_WRITE);

        wined3d_context_vk_destroy_bo(context_vk, &mut staging_bo);

        return;
    }

    if !dst_bo.is_null()
        && (((*dst_bo).memory_type & VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT) == 0
            || ((map_flags & WINED3D_MAP_DISCARD) == 0
                && (*dst_bo).command_buffer_id > (*context_vk).completed_command_buffer_id))
    {
        if wined3d_context_vk_create_bo(context_vk, size as VkDeviceSize, VK_BUFFER_USAGE_TRANSFER_SRC_BIT,
                VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT, &mut staging_bo) == 0
        {
            ERR!("Failed to create staging bo.\n");
            return;
        }

        staging.buffer_object = &mut staging_bo.b;
        staging.addr = core::ptr::null_mut();
        adapter_vk_copy_bo_address(context, &mut staging, src, range_count, ranges, WINED3D_MAP_WRITE);
        adapter_vk_copy_bo_address(context, dst, &mut staging, range_count, ranges, WINED3D_MAP_WRITE);

        wined3d_context_vk_destroy_bo(context_vk, &mut staging_bo);

        return;
    }

    src_ptr = adapter_vk_map_bo_address(context, src, size, WINED3D_MAP_READ) as *mut uint8_t;
    dst_ptr = adapter_vk_map_bo_address(context, dst, size, map_flags) as *mut uint8_t;

    i = 0;
    while i < range_count {
        memcpy(
            dst_ptr.add((*ranges.add(i as usize)).offset as usize) as *mut core::ffi::c_void,
            src_ptr.add((*ranges.add(i as usize)).offset as usize) as *const core::ffi::c_void,
            (*ranges.add(i as usize)).size as size_t,
        );
        i = i.wrapping_add(1);
    }

    adapter_vk_unmap_bo_address(context, dst, range_count, ranges);
    adapter_vk_unmap_bo_address(context, src, 0, core::ptr::null());
}

unsafe extern "C" fn adapter_vk_flush_bo_address(
    mut context: *mut wined3d_context,
    mut data: *const wined3d_const_bo_address,
    mut size: size_t,
) {
    let mut context_vk: *mut wined3d_context_vk = wined3d_context_vk(context);
    let mut bo: *mut wined3d_bo = core::ptr::null_mut::<wined3d_bo>();

    bo = (*data).buffer_object;
    if bo.is_null() {
        return;
    }

    flush_bo_range(
        context_vk,
        wined3d_bo_vk(bo),
        (*data).addr as uintptr_t as core::ffi::c_uint,
        size as core::ffi::c_uint,
    );
}

unsafe extern "C" fn adapter_vk_alloc_bo(
    device: *mut wined3d_device,
    resource: *mut wined3d_resource,
    sub_resource_idx: ::core::ffi::c_uint,
    addr: *mut wined3d_bo_address,
) -> bool {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk(device);
    let context_vk: *mut wined3d_context_vk = core::ptr::addr_of_mut!((*device_vk).context_vk);
    let mut memory_type: VkMemoryPropertyFlags = 0;
    let mut buffer_usage: VkBufferUsageFlags = 0;
    let mut bo_vk: *mut wined3d_bo_vk = core::ptr::null_mut();
    let mut size: VkDeviceSize = 0;

    wined3d_not_from_cs((*device).cs);
    assert!((*device).context_count != 0);

    if (*resource).type_ == WINED3D_RTYPE_BUFFER {
        buffer_usage = vk_buffer_usage_from_bind_flags((*resource).bind_flags as uint32_t);
        memory_type = vk_memory_type_from_access_flags((*resource).access as uint32_t, (*resource).usage);
        size = (*resource).size as VkDeviceSize;
    } else {
        let texture: *mut wined3d_texture = texture_from_resource(resource);

        buffer_usage = VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_TRANSFER_SRC_BIT;
        memory_type = VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT;
        size = (*(*texture).sub_resources.offset(sub_resource_idx as isize)).size as VkDeviceSize;
    }

    bo_vk = malloc(core::mem::size_of::<wined3d_bo_vk>() as size_t) as *mut wined3d_bo_vk;
    if bo_vk.is_null() {
        return false;
    }

    if wined3d_context_vk_create_bo(context_vk, size, buffer_usage, memory_type, bo_vk) == 0 {
        WARN!("Failed to create Vulkan buffer.\n");
        free(bo_vk as *mut ::core::ffi::c_void);
        return false;
    }

    if (*bo_vk).b.map_ptr.is_null() {
        if wined3d_bo_vk_map(bo_vk, context_vk).is_null() {
            ERR!("Failed to map bo.\n");
        }
    }

    (*addr).buffer_object = core::ptr::addr_of_mut!((*bo_vk).b);
    (*addr).addr = core::ptr::null_mut();
    return true;
}

unsafe extern "C" fn adapter_vk_destroy_bo(
    mut context: *mut wined3d_context,
    mut bo: *mut wined3d_bo,
) {
    wined3d_context_vk_destroy_bo(wined3d_context_vk(context), wined3d_bo_vk(bo));
}

unsafe extern "C" fn adapter_vk_create_swapchain(
    device: *mut wined3d_device,
    desc: *const wined3d_swapchain_desc,
    state_parent: *mut wined3d_swapchain_state_parent,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    swapchain: *mut *mut wined3d_swapchain,
) -> HRESULT {
    let mut swapchain_vk: *mut wined3d_swapchain_vk;
    let mut hr: HRESULT;

    swapchain_vk = calloc(1, (core::mem::size_of::<wined3d_swapchain_vk>()) as _) as *mut wined3d_swapchain_vk;
    if swapchain_vk.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = wined3d_swapchain_vk_init(swapchain_vk, device, desc, state_parent, parent, parent_ops);
    if hr < 0 {
        WARN!("Failed to initialise swapchain, hr %#lx.\n", hr);
        free(swapchain_vk as *mut core::ffi::c_void);
        return hr;
    }

    *swapchain = core::ptr::addr_of_mut!((*swapchain_vk).s);

    return hr;
}

unsafe extern "C" fn adapter_vk_destroy_swapchain(swapchain: *mut wined3d_swapchain) {
    let swapchain_vk: *mut wined3d_swapchain_vk = wined3d_swapchain_vk(swapchain);
    wined3d_swapchain_vk_cleanup(swapchain_vk);
    free(swapchain_vk as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_adapter_vk_get_memory_type_index(adapter_vk: *const wined3d_adapter_vk, memory_type_mask: uint32_t, flags: VkMemoryPropertyFlags) -> ::core::ffi::c_uint {
    let memory_info: *const VkPhysicalDeviceMemoryProperties = core::ptr::addr_of!((*adapter_vk).memory_properties);
    let mut i: ::core::ffi::c_uint;

    i = 0;
    while i < (*memory_info).memoryTypeCount {
        if !((memory_type_mask & (1 << i)) != 0) {
            i = i.wrapping_add(1);
            continue;
        }
        if ((*memory_info).memoryTypes[i as usize].propertyFlags & flags) == flags {
            return i;
        }
        i = i.wrapping_add(1);
    }

    return !0;
}

unsafe extern "C" fn adapter_vk_create_buffer(
    device: *mut wined3d_device,
    desc: *const wined3d_buffer_desc,
    data: *const wined3d_sub_resource_data,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    buffer: *mut *mut wined3d_buffer,
) -> HRESULT {
    let mut buffer_vk: *mut wined3d_buffer_vk = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    buffer_vk = calloc(1, (core::mem::size_of::<wined3d_buffer_vk>()) as _) as *mut wined3d_buffer_vk;
    if buffer_vk.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = wined3d_buffer_vk_init(buffer_vk, device, desc, data, parent, parent_ops);
    if hr < 0 {
        WARN!("Failed to initialise buffer, hr %#lx.\n", hr);
        free(buffer_vk as *mut core::ffi::c_void);
        return hr;
    }

    *buffer = core::ptr::addr_of_mut!((*buffer_vk).b);

    return hr;
}

unsafe extern "C" fn adapter_vk_destroy_buffer(buffer: *mut wined3d_buffer) {
    let buffer_vk: *mut wined3d_buffer_vk = wined3d_buffer_vk(buffer);
    let device: *mut wined3d_device = (*buffer_vk).b.resource.device;
    let swapchain_count: core::ffi::c_uint = (*device).swapchain_count as core::ffi::c_uint;

    /* Take a reference to the device, in case releasing the buffer would
     * cause the device to be destroyed. However, swapchain resources don't
     * take a reference to the device, and we wouldn't want to increment the
     * refcount on a device that's in the process of being destroyed. */
    if swapchain_count != 0 {
        wined3d_device_incref(device);
    }
    wined3d_buffer_cleanup(core::ptr::addr_of_mut!((*buffer_vk).b));
    wined3d_cs_destroy_object((*device).cs, Some(free), buffer_vk as *mut core::ffi::c_void);
    if swapchain_count != 0 {
        wined3d_device_decref(device);
    }
}

unsafe extern "C" fn adapter_vk_create_texture(
    mut device: *mut wined3d_device,
    mut desc: *const wined3d_resource_desc,
    mut layer_count: ::core::ffi::c_uint,
    mut level_count: ::core::ffi::c_uint,
    mut flags: uint32_t,
    mut parent: *mut ::core::ffi::c_void,
    mut parent_ops: *const wined3d_parent_ops,
    mut texture: *mut *mut wined3d_texture,
) -> HRESULT {
    let mut texture_vk: *mut wined3d_texture_vk = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    TRACE!(
        "device %p, desc %p, layer_count %u, level_count %u, flags %#x, parent %p, parent_ops %p, texture %p.\n",
        device,
        desc,
        layer_count,
        level_count,
        flags,
        parent,
        parent_ops,
        texture
    );

    texture_vk = wined3d_texture_allocate_object_memory(
        core::mem::size_of::<wined3d_texture_vk>() as SIZE_T,
        level_count as SIZE_T,
        layer_count as SIZE_T,
    ) as *mut wined3d_texture_vk;
    if texture_vk.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = wined3d_texture_vk_init(
        texture_vk,
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
        free(texture_vk as *mut ::core::ffi::c_void);
        return hr;
    }

    TRACE!("Created texture %p.\n", texture_vk);

    *texture = core::ptr::addr_of_mut!((*texture_vk).t);

    return hr;
}

unsafe extern "C" fn adapter_vk_destroy_texture(texture: *mut wined3d_texture) {
    let texture_vk: *mut wined3d_texture_vk = wined3d_texture_vk(texture);
    let device: *mut wined3d_device = (*texture_vk).t.resource.device;
    let swapchain_count: core::ffi::c_uint = (*device).swapchain_count as core::ffi::c_uint;

    if swapchain_count != 0 {
        wined3d_device_incref(device);
    }

    wined3d_texture_sub_resources_destroyed(texture);
    (*(*texture).resource.parent_ops)
        .wined3d_object_destroyed
        .unwrap_unchecked()((*texture).resource.parent);

    wined3d_texture_cleanup(core::ptr::addr_of_mut!((*texture_vk).t));
    wined3d_cs_destroy_object(
        (*device).cs as *mut wined3d_cs,
        Some(free as unsafe extern "C" fn(*mut core::ffi::c_void) -> ()),
        texture_vk as *mut core::ffi::c_void,
    );

    if swapchain_count != 0 {
        wined3d_device_decref(device);
    }
}

unsafe extern "C" fn adapter_vk_create_rendertarget_view(
    desc: *const wined3d_view_desc,
    resource: *mut wined3d_resource,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    view: *mut *mut wined3d_rendertarget_view,
) -> HRESULT {
    let mut view_vk: *mut wined3d_rendertarget_view_vk = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    view_vk = calloc(
        1,
        (core::mem::size_of::<wined3d_rendertarget_view_vk>()) as _,
    ) as *mut wined3d_rendertarget_view_vk;
    if view_vk.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = wined3d_rendertarget_view_vk_init(view_vk, desc, resource, parent, parent_ops);
    if hr < 0 {
        WARN!("Failed to initialise view, hr %#lx.\n", hr);
        free(view_vk as *mut core::ffi::c_void);
        return hr;
    }

    *view = core::ptr::addr_of_mut!((*view_vk).v);

    return hr;
}

unsafe extern "C" fn wined3d_view_vk_destroy_object(object: *mut core::ffi::c_void) {
    let ctx: *mut wined3d_view_vk_destroy_ctx = object as *mut wined3d_view_vk_destroy_ctx;
    let mut vk_info: *const wined3d_vk_info;
    let mut device_vk: *mut wined3d_device_vk;
    let mut context: *mut wined3d_context;

    device_vk = (*ctx).device_vk;
    vk_info = core::ptr::addr_of!((*wined3d_adapter_vk((*device_vk).d.adapter)).vk_info);
    context = context_acquire(
        core::ptr::addr_of_mut!((*device_vk).d),
        core::ptr::null_mut(),
        0,
    );

    if !(*ctx).vk_buffer_view.is_null() {
        if !context.is_null() {
            wined3d_context_vk_destroy_vk_buffer_view(
                wined3d_context_vk(context),
                *(*ctx).vk_buffer_view,
                *(*ctx).command_buffer_id,
            );
        } else {
            ((*vk_info).vk_ops.vkDestroyBufferView.unwrap_unchecked())(
                (*device_vk).vk_device,
                *(*ctx).vk_buffer_view,
                core::ptr::null(),
            );
        }
    }
    if !(*ctx).vk_image_view.is_null() {
        if !context.is_null() {
            wined3d_context_vk_destroy_vk_image_view(
                wined3d_context_vk(context),
                *(*ctx).vk_image_view,
                *(*ctx).command_buffer_id,
            );
        } else {
            ((*vk_info).vk_ops.vkDestroyImageView.unwrap_unchecked())(
                (*device_vk).vk_device,
                *(*ctx).vk_image_view,
                core::ptr::null(),
            );
        }
    }
    if !(*ctx).bo_user.is_null() && (*(*ctx).bo_user).valid != false {
        list_remove(core::ptr::addr_of_mut!((*(*ctx).bo_user).entry));
    }
    if !(*ctx).vk_counter_bo.is_null() && (*(*ctx).vk_counter_bo).vk_buffer != 0 {
        wined3d_context_vk_destroy_bo(wined3d_context_vk(context), (*ctx).vk_counter_bo);
    }
    if !(*ctx).vk_counter_view.is_null() {
        if !context.is_null() {
            wined3d_context_vk_destroy_vk_buffer_view(
                wined3d_context_vk(context),
                *(*ctx).vk_counter_view,
                *(*ctx).command_buffer_id,
            );
        } else {
            ((*vk_info).vk_ops.vkDestroyBufferView.unwrap_unchecked())(
                (*device_vk).vk_device,
                *(*ctx).vk_counter_view,
                core::ptr::null(),
            );
        }
    }

    if !context.is_null() {
        context_release(context);
    }

    free((*ctx).object);
    free((*ctx).free as *mut core::ffi::c_void);
}

unsafe extern "C" fn wined3d_view_vk_destroy(
    mut device: *mut wined3d_device,
    mut vk_buffer_view: *mut VkBufferView,
    mut vk_image_view: *mut VkImageView,
    mut bo_user: *mut wined3d_bo_user,
    mut vk_counter_bo: *mut wined3d_bo_vk,
    mut vk_counter_view: *mut VkBufferView,
    mut command_buffer_id: *mut uint64_t,
    mut view_vk: *mut ::core::ffi::c_void,
) {
    let mut ctx: *mut wined3d_view_vk_destroy_ctx =
        ::core::ptr::null_mut::<wined3d_view_vk_destroy_ctx>();
    let mut c: wined3d_view_vk_destroy_ctx = wined3d_view_vk_destroy_ctx {
        device_vk: ::core::ptr::null_mut::<wined3d_device_vk>(),
        vk_buffer_view: ::core::ptr::null_mut::<VkBufferView>(),
        vk_image_view: ::core::ptr::null_mut::<VkImageView>(),
        bo_user: ::core::ptr::null_mut::<wined3d_bo_user>(),
        vk_counter_bo: ::core::ptr::null_mut::<wined3d_bo_vk>(),
        vk_counter_view: ::core::ptr::null_mut::<VkBufferView>(),
        command_buffer_id: ::core::ptr::null_mut::<uint64_t>(),
        object: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        free: ::core::ptr::null_mut::<wined3d_view_vk_destroy_ctx>(),
    };
    ctx = malloc(core::mem::size_of::<wined3d_view_vk_destroy_ctx>() as size_t)
        as *mut wined3d_view_vk_destroy_ctx;
    if ctx.is_null() {
        ctx = core::ptr::addr_of_mut!(c);
    }
    (*ctx).device_vk = wined3d_device_vk(device);
    (*ctx).vk_buffer_view = vk_buffer_view;
    (*ctx).vk_image_view = vk_image_view;
    (*ctx).bo_user = bo_user;
    (*ctx).vk_counter_bo = vk_counter_bo;
    (*ctx).vk_counter_view = vk_counter_view;
    (*ctx).command_buffer_id = command_buffer_id;
    (*ctx).object = view_vk;
    (*ctx).free = if ctx != core::ptr::addr_of_mut!(c) {
        ctx
    } else {
        ::core::ptr::null_mut::<wined3d_view_vk_destroy_ctx>()
    };
    wined3d_cs_destroy_object(
        (*device).cs as *mut wined3d_cs,
        Some(
            wined3d_view_vk_destroy_object as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
        ),
        ctx as *mut ::core::ffi::c_void,
    );
    if ctx == core::ptr::addr_of_mut!(c) {
        wined3d_cs_finish((*device).cs as *mut wined3d_cs, WINED3D_CS_QUEUE_DEFAULT);
    }
}

unsafe extern "C" fn adapter_vk_destroy_rendertarget_view(view: *mut wined3d_rendertarget_view) {
    let view_vk: *mut wined3d_rendertarget_view_vk = wined3d_rendertarget_view_vk(view);
    let resource: *mut wined3d_resource = (*view_vk).v.resource;

    wined3d_rendertarget_view_cleanup(&raw mut (*view_vk).v);
    wined3d_view_vk_destroy(
        (*resource).device,
        core::ptr::null_mut(),
        &raw mut (*view_vk).vk_image_view,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &raw mut (*view_vk).command_buffer_id,
        view_vk as *mut core::ffi::c_void,
    );
}

unsafe extern "C" fn adapter_vk_create_shader_resource_view(
    desc: *const wined3d_view_desc,
    resource: *mut wined3d_resource,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    view: *mut *mut wined3d_shader_resource_view,
) -> HRESULT {
    let mut view_vk: *mut wined3d_shader_resource_view_vk = core::ptr::null_mut();
    let mut hr: HRESULT;

    view_vk = calloc(
        1,
        (core::mem::size_of::<wined3d_shader_resource_view_vk>()) as _,
    ) as *mut wined3d_shader_resource_view_vk;
    if view_vk.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = wined3d_shader_resource_view_vk_init(view_vk, desc, resource, parent, parent_ops);
    if hr < 0 {
        WARN!("Failed to initialise view, hr %#lx.\n", hr);
        free(view_vk as *mut core::ffi::c_void);
        return hr;
    }

    *view = core::ptr::addr_of_mut!((*view_vk).v);

    return hr;
}

unsafe extern "C" fn adapter_vk_destroy_shader_resource_view(
    mut view: *mut wined3d_shader_resource_view,
) {
    let mut srv_vk: *mut wined3d_shader_resource_view_vk = wined3d_shader_resource_view_vk(view);
    let mut resource: *mut wined3d_resource = (*srv_vk).v.resource;
    let mut view_vk: *mut wined3d_view_vk = &raw mut (*srv_vk).view_vk;
    let mut vk_buffer_view: *mut VkBufferView = core::ptr::null_mut::<VkBufferView>();
    let mut vk_image_view: *mut VkImageView = core::ptr::null_mut::<VkImageView>();

    TRACE!("srv_vk %p.\n", srv_vk);

    if (*resource).type_ as core::ffi::c_uint == WINED3D_RTYPE_BUFFER as core::ffi::c_uint {
        vk_buffer_view = &raw mut (*view_vk).u.vk_buffer_view;
    } else {
        vk_image_view = &raw mut (*view_vk).u.vk_image_info.imageView;
    }
    wined3d_shader_resource_view_cleanup(&raw mut (*srv_vk).v);
    wined3d_view_vk_destroy(
        (*resource).device,
        vk_buffer_view,
        vk_image_view,
        &raw mut (*view_vk).bo_user,
        core::ptr::null_mut::<wined3d_bo_vk>(),
        core::ptr::null_mut::<VkBufferView>(),
        &raw mut (*view_vk).command_buffer_id,
        srv_vk as *mut core::ffi::c_void,
    );
}

unsafe extern "C" fn adapter_vk_create_unordered_access_view(
    desc: *const wined3d_view_desc,
    resource: *mut wined3d_resource,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    view: *mut *mut wined3d_unordered_access_view,
) -> HRESULT {
    let mut view_vk: *mut wined3d_unordered_access_view_vk;
    let mut hr: HRESULT;

    view_vk = calloc(
        1,
        (core::mem::size_of::<wined3d_unordered_access_view_vk>()) as _,
    ) as *mut wined3d_unordered_access_view_vk;
    if view_vk.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = wined3d_unordered_access_view_vk_init(view_vk, desc, resource, parent, parent_ops);
    if hr < 0 {
        WARN!("Failed to initialise view, hr %#lx.\n", hr);
        free(view_vk as *mut core::ffi::c_void);
        return hr;
    }

    *view = core::ptr::addr_of_mut!((*view_vk).v);

    return hr;
}

unsafe extern "C" fn adapter_vk_destroy_unordered_access_view(view: *mut wined3d_unordered_access_view) {
    let uav_vk: *mut wined3d_unordered_access_view_vk = wined3d_unordered_access_view_vk(view);
    let resource: *mut wined3d_resource = (*uav_vk).v.resource;
    let view_vk: *mut wined3d_view_vk = core::ptr::addr_of_mut!((*uav_vk).view_vk);
    let mut vk_buffer_view: *mut VkBufferView = core::ptr::null_mut();
    let mut vk_image_view: *mut VkImageView = core::ptr::null_mut();

    if (*resource).type_ == WINED3D_RTYPE_BUFFER {
        vk_buffer_view = core::ptr::addr_of_mut!((*view_vk).u.vk_buffer_view);
    } else {
        vk_image_view = core::ptr::addr_of_mut!((*view_vk).u.vk_image_info.imageView);
    }
    wined3d_unordered_access_view_cleanup(core::ptr::addr_of_mut!((*uav_vk).v));
    wined3d_view_vk_destroy(
        (*resource).device,
        vk_buffer_view,
        vk_image_view,
        core::ptr::addr_of_mut!((*view_vk).bo_user),
        core::ptr::addr_of_mut!((*uav_vk).counter_bo),
        core::ptr::addr_of_mut!((*uav_vk).vk_counter_view),
        core::ptr::addr_of_mut!((*view_vk).command_buffer_id),
        uav_vk as *mut core::ffi::c_void,
    );
}

unsafe extern "C" fn adapter_vk_create_sampler(
    mut device: *mut wined3d_device,
    mut desc: *const wined3d_sampler_desc,
    mut parent: *mut ::core::ffi::c_void,
    mut parent_ops: *const wined3d_parent_ops,
    mut sampler: *mut *mut wined3d_sampler,
) -> HRESULT {
    let mut sampler_vk: *mut wined3d_sampler_vk = ::core::ptr::null_mut::<wined3d_sampler_vk>();
    WINE_TRACE!("device %p, desc %p, parent %p, parent_ops %p, sampler %p.\n", device, desc, parent, parent_ops, sampler);
    sampler_vk = calloc(
        (1 as size_t) as _,
        (::core::mem::size_of::<wined3d_sampler_vk>() as size_t) as _,
    ) as *mut wined3d_sampler_vk;
    if sampler_vk.is_null() {
        return E_OUTOFMEMORY;
    }
    wined3d_sampler_vk_init(sampler_vk, device, desc, parent, parent_ops);
    WINE_TRACE!("Created sampler %p.\n", sampler_vk);
    *sampler = core::ptr::addr_of_mut!((*sampler_vk).s);
    return WINED3D_OK;
}

unsafe extern "C" fn wined3d_sampler_vk_destroy_object(object: *mut core::ffi::c_void) {
    let sampler_vk: *mut wined3d_sampler_vk = object as *mut wined3d_sampler_vk;
    let mut context_vk: *mut wined3d_context_vk = core::ptr::null_mut();

    TRACE!("sampler_vk %p.\n", sampler_vk);

    context_vk = wined3d_context_vk(context_acquire((*sampler_vk).s.device, core::ptr::null_mut(), 0));

    wined3d_context_vk_destroy_vk_sampler(context_vk, (*sampler_vk).vk_image_info.sampler, (*sampler_vk).command_buffer_id);
    free(sampler_vk as *mut core::ffi::c_void);

    context_release(core::ptr::addr_of_mut!((*context_vk).c));
}

unsafe extern "C" fn adapter_vk_destroy_sampler(mut sampler: *mut wined3d_sampler) {
    let mut sampler_vk: *mut wined3d_sampler_vk = wined3d_sampler_vk(sampler);
    WINE_TRACE!("sampler_vk %p.\n", sampler_vk);
    wined3d_cs_destroy_object(
        (*(*sampler).device).cs as *mut wined3d_cs,
        Some(
            wined3d_sampler_vk_destroy_object
                as unsafe extern "C" fn(*mut core::ffi::c_void) -> (),
        ),
        sampler_vk as *mut core::ffi::c_void,
    );
}

unsafe extern "C" fn adapter_vk_create_query(
    device: *mut wined3d_device,
    r#type: wined3d_query_type,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    query: *mut *mut wined3d_query,
) -> HRESULT {
    // TRACE("device %p, type %#x, parent %p, parent_ops %p, query %p.\n", device, type, parent, parent_ops, query);
    wined3d_query_vk_create(device, r#type, parent, parent_ops, query)
}

unsafe extern "C" fn wined3d_query_vk_destroy_object(mut object: *mut ::core::ffi::c_void) {
    let mut query_vk: *mut wined3d_query_vk = object as *mut wined3d_query_vk;
    WINE_TRACE!("query_vk %p.\n", query_vk);
    (*(*query_vk).q.query_ops)
        .query_destroy
        .unwrap_unchecked()(core::ptr::addr_of_mut!((*query_vk).q));
}

unsafe extern "C" fn adapter_vk_destroy_query(query: *mut wined3d_query) {
    let query_vk: *mut wined3d_query_vk = wined3d_query_vk(query);

    wined3d_cs_destroy_object((*(*query).device).cs as *mut wined3d_cs, Some(wined3d_query_vk_destroy_object), query_vk as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn adapter_vk_flush_context(mut context: *mut wined3d_context) {
    let mut context_vk: *mut wined3d_context_vk = wined3d_context_vk(context);
    WINE_TRACE!("context_vk %p.\n", context_vk);
    wined3d_context_vk_submit_command_buffer(
        context_vk,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
    );
}

unsafe extern "C" fn adapter_vk_draw_primitive(
    device: *mut wined3d_device,
    state: *const wined3d_state,
    parameters: *const wined3d_draw_parameters,
) {
    let mut indirect_vk: *mut wined3d_buffer_vk = core::ptr::null_mut();
    let mut vk_info: *const wined3d_vk_info = core::ptr::null();
    let mut context_vk: *mut wined3d_context_vk = core::ptr::null_mut();
    let mut vk_command_buffer: VkCommandBuffer = core::ptr::null_mut();
    let mut instance_count: u32 = 0;

    TRACE!("device %p, state %p, parameters %p.\n", device, state, parameters);

    context_vk = wined3d_context_vk(context_acquire(device, core::ptr::null_mut(), 0));
    vk_info = (*context_vk).vk_info;

    if (*parameters).indirect != 0 {
        indirect_vk = wined3d_buffer_vk((*parameters).u.indirect.buffer);
    }

    vk_command_buffer = wined3d_context_vk_apply_draw_state(
        context_vk,
        state,
        indirect_vk,
        (*parameters).indexed != 0,
    );
    if vk_command_buffer.is_null() {
        ERR!("Failed to apply draw state.\n");
        context_release(&raw mut (*context_vk).c);
        return;
    }

    if (*context_vk).c.transform_feedback_active() != 0 {
        wined3d_context_vk_reference_bo(context_vk, &raw mut (*context_vk).vk_so_counter_bo);
        if (*context_vk).c.transform_feedback_paused() != 0 {
            ((*vk_info).vk_ops.vkCmdBeginTransformFeedbackEXT.unwrap_unchecked())(
                vk_command_buffer,
                0,
                ARRAY_SIZE!((*context_vk).vk_so_counters),
                &raw mut (*context_vk).vk_so_counters as *mut VkBuffer,
                &raw mut (*context_vk).vk_so_offsets as *mut VkDeviceSize,
            );
        } else {
            ((*vk_info).vk_ops.vkCmdBeginTransformFeedbackEXT.unwrap_unchecked())(
                vk_command_buffer,
                0,
                0,
                core::ptr::null::<VkBuffer>(),
                core::ptr::null::<VkDeviceSize>(),
            );
        }
    }

    if (*parameters).indirect != 0 {
        let bo: *mut wined3d_bo_vk = wined3d_bo_vk((*indirect_vk).b.buffer_object);

        wined3d_context_vk_reference_bo(context_vk, bo);
        if (*parameters).indexed != 0 {
            ((*vk_info).vk_ops.vkCmdDrawIndexedIndirect.unwrap_unchecked())(
                vk_command_buffer,
                (*bo).vk_buffer,
                ((*bo).b.buffer_offset as VkDeviceSize)
                    .wrapping_add((*parameters).u.indirect.offset as VkDeviceSize),
                1,
                core::mem::size_of::<VkDrawIndexedIndirectCommand>() as u32,
            );
        } else {
            ((*vk_info).vk_ops.vkCmdDrawIndirect.unwrap_unchecked())(
                vk_command_buffer,
                (*bo).vk_buffer,
                ((*bo).b.buffer_offset as VkDeviceSize)
                    .wrapping_add((*parameters).u.indirect.offset as VkDeviceSize),
                1,
                core::mem::size_of::<VkDrawIndirectCommand>() as u32,
            );
        }
    } else {
        instance_count = (*parameters).u.direct.instance_count as u32;
        if instance_count == 0 {
            instance_count = 1;
        }

        if (*parameters).indexed != 0 {
            ((*vk_info).vk_ops.vkCmdDrawIndexed.unwrap_unchecked())(
                vk_command_buffer,
                (*parameters).u.direct.index_count as u32,
                instance_count,
                (*parameters).u.direct.start_idx as u32,
                (*parameters).u.direct.base_vertex_idx as i32,
                (*parameters).u.direct.start_instance as u32,
            );
        } else {
            ((*vk_info).vk_ops.vkCmdDraw.unwrap_unchecked())(
                vk_command_buffer,
                (*parameters).u.direct.index_count as u32,
                instance_count,
                (*parameters).u.direct.start_idx as u32,
                (*parameters).u.direct.start_instance as u32,
            );
        }
    }

    if (*context_vk).c.transform_feedback_active() != 0 {
        ((*vk_info).vk_ops.vkCmdEndTransformFeedbackEXT.unwrap_unchecked())(
            vk_command_buffer,
            0,
            ARRAY_SIZE!((*context_vk).vk_so_counters),
            &raw mut (*context_vk).vk_so_counters as *mut VkBuffer,
            &raw mut (*context_vk).vk_so_offsets as *mut VkDeviceSize,
        );
        (*context_vk).c.set_transform_feedback_paused(1 as DWORD);
        (*context_vk).c.set_transform_feedback_active(0 as DWORD);
    }

    (*context_vk).command_buffer_work_count = (*context_vk).command_buffer_work_count.wrapping_add(1);

    context_release(&raw mut (*context_vk).c);
}

unsafe extern "C" fn adapter_vk_dispatch_compute(
    device: *mut wined3d_device,
    state: *const wined3d_state,
    parameters: *const wined3d_dispatch_parameters,
) {
    let mut indirect_vk: *mut wined3d_buffer_vk = core::ptr::null_mut();
    let mut vk_info: *const wined3d_vk_info = core::ptr::null();
    let mut context_vk: *mut wined3d_context_vk = core::ptr::null_mut();
    let mut vk_command_buffer: VkCommandBuffer = core::ptr::null_mut();

    TRACE!(
        "device %p, state %p, parameters %p.\n",
        device,
        state,
        parameters
    );

    context_vk = wined3d_context_vk(context_acquire(
        device,
        core::ptr::null_mut(),
        0,
    ));
    vk_info = (*context_vk).vk_info;

    if (*parameters).indirect != 0 {
        indirect_vk = wined3d_buffer_vk((*parameters).u.indirect.buffer);
    }

    vk_command_buffer = wined3d_context_vk_apply_compute_state(context_vk, state, indirect_vk);
    if vk_command_buffer.is_null() {
        ERR!("Failed to apply compute state.\n");
        context_release(core::ptr::addr_of_mut!((*context_vk).c));
        return;
    }

    if (*parameters).indirect != 0 {
        let bo: *mut wined3d_bo_vk = wined3d_bo_vk((*indirect_vk).b.buffer_object);

        wined3d_context_vk_reference_bo(context_vk, bo);
        ((*vk_info).vk_ops.vkCmdDispatchIndirect.unwrap_unchecked())(
            vk_command_buffer,
            (*bo).vk_buffer,
            ((*bo).b.buffer_offset as VkDeviceSize)
                .wrapping_add((*parameters).u.indirect.offset as VkDeviceSize),
        );
    } else {
        let direct: *const wined3d_direct_dispatch_parameters =
            core::ptr::addr_of!((*parameters).u.direct);

        ((*vk_info).vk_ops.vkCmdDispatch.unwrap_unchecked())(
            vk_command_buffer,
            (*direct).group_count_x as uint32_t,
            (*direct).group_count_y as uint32_t,
            (*direct).group_count_z as uint32_t,
        );
    }

    ((*vk_info).vk_ops.vkCmdPipelineBarrier.unwrap_unchecked())(
        vk_command_buffer,
        VK_PIPELINE_STAGE_COMPUTE_SHADER_BIT,
        VK_PIPELINE_STAGE_ALL_GRAPHICS_BIT,
        0,
        0,
        core::ptr::null(),
        0,
        core::ptr::null(),
        0,
        core::ptr::null(),
    );

    (*context_vk).command_buffer_work_count =
        (*context_vk).command_buffer_work_count.wrapping_add(1);

    context_release(core::ptr::addr_of_mut!((*context_vk).c));
}

unsafe extern "C" fn adapter_vk_clear_uav(
    mut context: *mut wined3d_context,
    mut view: *mut wined3d_unordered_access_view,
    mut clear_value: *const wined3d_uvec4,
    mut fp: bool,
) {
    TRACE!(
        "context %p, view %p, clear_value %s.\n",
        context,
        view,
        debug_uvec4(clear_value)
    );
    wined3d_unordered_access_view_vk_clear(
        wined3d_unordered_access_view_vk(view),
        clear_value,
        wined3d_context_vk(context),
        fp,
    );
}

unsafe extern "C" fn adapter_vk_generate_mipmap(
    context: *mut wined3d_context,
    view: *mut wined3d_shader_resource_view,
) {
    wined3d_shader_resource_view_vk_generate_mipmap(
        wined3d_shader_resource_view_vk(view),
        wined3d_context_vk(context),
    );
}

unsafe extern "C" fn wined3d_get_wine_vk_version() -> ::core::ffi::c_uint {
    let wine_get_version_opt: Option<unsafe extern "C" fn() -> *const ::core::ffi::c_char> =
        ::core::mem::transmute::<
            *mut ::core::ffi::c_void,
            Option<unsafe extern "C" fn() -> *const ::core::ffi::c_char>,
        >(::core::mem::transmute::<FARPROC, *mut ::core::ffi::c_void>(
            GetProcAddress(
                GetModuleHandleW(RUSTCONV_WSTR_80073828D8A0.as_ptr()),
                b"wine_get_version\0".as_ptr() as LPCSTR,
            ),
        ));
    if wine_get_version_opt.is_none() {
        return VK_MAKE_VERSION!(1, 0, 0);
    }
    let wine_get_version = wine_get_version_opt.unwrap_unchecked();
    let mut ptr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut major: ::core::ffi::c_int = 0;
    let mut minor: ::core::ffi::c_int = 0;

    ptr = wine_get_version();
    major = atoi(ptr);

    while isdigit(*ptr as ::core::ffi::c_int) != 0 {
        ptr = ptr.offset(1);
    }
    if *ptr as ::core::ffi::c_int == '.' as i32 {
        ptr = ptr.offset(1);
    }

    minor = atoi(ptr);

    return VK_MAKE_VERSION!(
        major as ::core::ffi::c_uint,
        minor as ::core::ffi::c_uint,
        0,
    );
}

unsafe extern "C" fn enable_vulkan_instance_extensions(
    extension_count: *mut uint32_t,
    enabled_extensions: *mut *const ::core::ffi::c_char,
    vk_info: *const wined3d_vk_info,
) -> BOOL {
    let mut pfn_vkEnumerateInstanceExtensionProperties: PFN_vkEnumerateInstanceExtensionProperties =
        None;
    let mut extensions: *mut VkExtensionProperties = core::ptr::null_mut();
    let mut success: BOOL = (FALSE as i32);
    let mut found: BOOL = 0;
    let mut i: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_uint = 0;
    let mut count: ::core::ffi::c_uint = 0;
    let mut vr: VkResult = VK_SUCCESS;

    *extension_count = 0;

    pfn_vkEnumerateInstanceExtensionProperties = ::core::mem::transmute::<
        *mut ::core::ffi::c_void,
        PFN_vkEnumerateInstanceExtensionProperties,
    >(::core::mem::transmute::<
        PFN_vkVoidFunction,
        *mut ::core::ffi::c_void,
    >(((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked())(
        core::ptr::null_mut::<VkInstance_T>(),
        b"vkEnumerateInstanceExtensionProperties\0" as *const u8 as *const ::core::ffi::c_char,
    )));

    'done: {
        if pfn_vkEnumerateInstanceExtensionProperties.is_none() {
            WARN!("Failed to get 'vkEnumerateInstanceExtensionProperties'.\n");
            break 'done;
        }

        vr = (pfn_vkEnumerateInstanceExtensionProperties.unwrap_unchecked())(
            core::ptr::null::<::core::ffi::c_char>(),
            core::ptr::addr_of_mut!(count),
            core::ptr::null_mut::<VkExtensionProperties>(),
        );
        if (vr as ::core::ffi::c_int) < 0 {
            WARN!(
                "Failed to count instance extensions, vr %s.\n",
                wined3d_debug_vkresult(vr)
            );
            break 'done;
        }

        extensions = calloc(
            (count as size_t) as _,
            (::core::mem::size_of::<VkExtensionProperties>() as size_t) as _,
        ) as *mut VkExtensionProperties;
        if extensions.is_null() {
            WARN!("Out of memory.\n");
            break 'done;
        }

        vr = (pfn_vkEnumerateInstanceExtensionProperties.unwrap_unchecked())(
            core::ptr::null::<::core::ffi::c_char>(),
            core::ptr::addr_of_mut!(count),
            extensions,
        );
        if (vr as ::core::ffi::c_int) < 0 {
            WARN!(
                "Failed to enumerate extensions, vr %s.\n",
                wined3d_debug_vkresult(vr)
            );
            break 'done;
        }

        TRACE!("Vulkan instance extensions reported:\n");
        i = 0;
        while i < count {
            TRACE!(
                "    - %s.\n",
                debugstr_a(core::ptr::addr_of_mut!(
                    (*extensions.offset(i as isize)).extensionName
                ) as *mut ::core::ffi::c_char)
            );
            i = i.wrapping_add(1);
        }

        i = 0;
        while (i as usize) < ARRAY_SIZE!(vulkan_instance_extensions) as usize {
            if !(vulkan_instance_extensions[i as usize].core_since_version
                <= (*vk_info).api_version)
            {
                j = 0;
                found = (FALSE as i32);
                while j < count {
                    if strcmp(
                        core::ptr::addr_of_mut!((*extensions.offset(j as isize)).extensionName)
                            as *mut ::core::ffi::c_char,
                        vulkan_instance_extensions[i as usize].name,
                    ) == 0
                    {
                        found = (TRUE as i32);
                        break;
                    } else {
                        j = j.wrapping_add(1);
                    }
                }
                if found != 0 {
                    TRACE!(
                        "Enabling instance extension '%s'.\n",
                        vulkan_instance_extensions[i as usize].name
                    );
                    let fresh17 = *extension_count;
                    *extension_count = (*extension_count).wrapping_add(1);
                    *enabled_extensions.offset(fresh17 as isize) =
                        vulkan_instance_extensions[i as usize].name;
                } else if found == 0 && vulkan_instance_extensions[i as usize].required != 0 {
                    WARN!(
                        "Required extension '%s' is not available.\n",
                        vulkan_instance_extensions[i as usize].name
                    );
                    break 'done;
                }
            }
            i = i.wrapping_add(1);
        }
        success = (TRUE as i32);
    }

    free(extensions as *mut ::core::ffi::c_void);
    return success;
}

unsafe extern "C" fn wined3d_init_vulkan(vk_info: *mut wined3d_vk_info) -> BOOL {
    let mut enabled_instance_extensions: [*const ::core::ffi::c_char; (unsafe { ARRAY_SIZE!(vulkan_instance_extensions) }) as usize] =
        [::core::ptr::null::<::core::ffi::c_char>(); (unsafe { ARRAY_SIZE!(vulkan_instance_extensions) }) as usize];
    let mut pfn_vkEnumerateInstanceVersion: PFN_vkEnumerateInstanceVersion = None;
    let vk_ops: *mut vulkan_ops = &raw mut (*vk_info).vk_ops;
    let mut instance: VkInstance = core::ptr::null_mut();
    let mut instance_info: VkInstanceCreateInfo = core::mem::zeroed();
    let mut app_info: VkApplicationInfo = core::mem::zeroed();
    let mut api_version: uint32_t = 0;
    let mut app_name: [::core::ffi::c_char; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut vr: VkResult;

    if wined3d_load_vulkan(vk_info) == 0 {
        return (FALSE) as BOOL;
    }

    'fail: {
        (*vk_ops).vkCreateInstance = ::core::mem::transmute::<*mut ::core::ffi::c_void, PFN_vkCreateInstance>(
            ::core::mem::transmute::<PFN_vkVoidFunction, *mut ::core::ffi::c_void>(((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked())(
                    ::core::ptr::null_mut::<VkInstance_T>(),
                    b"vkCreateInstance\0".as_ptr() as *const ::core::ffi::c_char,
                )),
        );
        if (*vk_ops).vkCreateInstance.is_none() {
            ERR!("Failed to get 'vkCreateInstance'.\n");
            break 'fail;
        }

        (*vk_info).api_version = VK_API_VERSION_1_0;
        pfn_vkEnumerateInstanceVersion = ::core::mem::transmute::<*mut ::core::ffi::c_void, PFN_vkEnumerateInstanceVersion>(
            ::core::mem::transmute::<PFN_vkVoidFunction, *mut ::core::ffi::c_void>(((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked())(
                    ::core::ptr::null_mut::<VkInstance_T>(),
                    b"vkEnumerateInstanceVersion\0".as_ptr() as *const ::core::ffi::c_char,
                )),
        );
        if pfn_vkEnumerateInstanceVersion.is_some()
            && (pfn_vkEnumerateInstanceVersion.unwrap_unchecked())(&raw mut api_version) == VK_SUCCESS
        {
            if api_version >= VK_API_VERSION_1_1 {
                (*vk_info).api_version = VK_API_VERSION_1_1;
            }
        }

        memset(
            &raw mut app_info as *mut ::core::ffi::c_void,
            0,
            ::core::mem::size_of::<VkApplicationInfo>() as usize,
        );
        app_info.sType = VK_STRUCTURE_TYPE_APPLICATION_INFO;
        if wined3d_get_app_name(
            &raw mut app_name as *mut ::core::ffi::c_char,
            ARRAY_SIZE!(app_name) as ::core::ffi::c_uint,
        ) != 0
        {
            app_info.pApplicationName = &raw mut app_name as *mut ::core::ffi::c_char;
        }
        app_info.pEngineName = b"Damavand\0".as_ptr() as *const ::core::ffi::c_char;
        app_info.engineVersion = wined3d_get_wine_vk_version() as uint32_t;
        app_info.apiVersion = (*vk_info).api_version as uint32_t;

        memset(
            &raw mut instance_info as *mut ::core::ffi::c_void,
            0,
            ::core::mem::size_of::<VkInstanceCreateInfo>() as usize,
        );
        instance_info.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO;
        instance_info.pApplicationInfo = &raw mut app_info;
        instance_info.ppEnabledExtensionNames =
            &raw mut enabled_instance_extensions as *mut *const ::core::ffi::c_char;
        if enable_vulkan_instance_extensions(
            &raw mut instance_info.enabledExtensionCount,
            &raw mut enabled_instance_extensions as *mut *const ::core::ffi::c_char,
            vk_info,
        ) == 0
        {
            break 'fail;
        }

        memset(
            &raw mut (*vk_info).supported as *mut BOOL as *mut ::core::ffi::c_void,
            0,
            ::core::mem::size_of::<[BOOL; 11]>() as usize,
        );
        (*vk_info).supported[WINED3D_VK_EXT_NONE as usize] = (TRUE as i32);

        vr = ((*vk_info).vk_ops.vkCreateInstance.unwrap_unchecked())(&raw mut instance_info, ::core::ptr::null::<VkAllocationCallbacks>(), &raw mut instance);
        if (vr as ::core::ffi::c_int) < 0 {
            WARN!(
                "Failed to create Vulkan instance, vr %s.\n",
                wined3d_debug_vkresult(vr)
            );
            break 'fail;
        }

        TRACE!("Created Vulkan instance %p.\n", instance);

        // VK_INSTANCE_FUNCS() then VK_DEVICE_FUNCS(), with VK_*_PFN bound to LOAD_INSTANCE_PFN
        // and VK_*_EXT_PFN to LOAD_INSTANCE_OPT_PFN (adapter_vk.c), generated from wined3d_vk.h.
        (*vk_ops).vkCreateDevice = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateDevice>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCreateDevice\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCreateDevice.is_none() {
            WARN!("Could not get instance proc addr for 'vkCreateDevice'.\n");
            break 'fail;
        }
        (*vk_ops).vkDestroyInstance = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyInstance>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkDestroyInstance\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkDestroyInstance.is_none() {
            WARN!("Could not get instance proc addr for 'vkDestroyInstance'.\n");
            break 'fail;
        }
        (*vk_ops).vkEnumerateDeviceExtensionProperties = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkEnumerateDeviceExtensionProperties>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkEnumerateDeviceExtensionProperties\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkEnumerateDeviceExtensionProperties.is_none() {
            WARN!("Could not get instance proc addr for 'vkEnumerateDeviceExtensionProperties'.\n");
            break 'fail;
        }
        (*vk_ops).vkEnumerateDeviceLayerProperties = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkEnumerateDeviceLayerProperties>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkEnumerateDeviceLayerProperties\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkEnumerateDeviceLayerProperties.is_none() {
            WARN!("Could not get instance proc addr for 'vkEnumerateDeviceLayerProperties'.\n");
            break 'fail;
        }
        (*vk_ops).vkEnumeratePhysicalDevices = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkEnumeratePhysicalDevices>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkEnumeratePhysicalDevices\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkEnumeratePhysicalDevices.is_none() {
            WARN!("Could not get instance proc addr for 'vkEnumeratePhysicalDevices'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetDeviceProcAddr = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetDeviceProcAddr>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetDeviceProcAddr\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetDeviceProcAddr.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetDeviceProcAddr'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetPhysicalDeviceFeatures = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetPhysicalDeviceFeatures>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetPhysicalDeviceFeatures\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetPhysicalDeviceFeatures.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetPhysicalDeviceFeatures'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetPhysicalDeviceFormatProperties = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetPhysicalDeviceFormatProperties>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetPhysicalDeviceFormatProperties\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetPhysicalDeviceFormatProperties.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetPhysicalDeviceFormatProperties'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetPhysicalDeviceImageFormatProperties = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetPhysicalDeviceImageFormatProperties>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetPhysicalDeviceImageFormatProperties\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetPhysicalDeviceImageFormatProperties.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetPhysicalDeviceImageFormatProperties'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetPhysicalDeviceMemoryProperties = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetPhysicalDeviceMemoryProperties>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetPhysicalDeviceMemoryProperties\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetPhysicalDeviceMemoryProperties.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetPhysicalDeviceMemoryProperties'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetPhysicalDeviceProperties = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetPhysicalDeviceProperties>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetPhysicalDeviceProperties\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetPhysicalDeviceProperties.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetPhysicalDeviceProperties'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetPhysicalDeviceQueueFamilyProperties = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetPhysicalDeviceQueueFamilyProperties>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetPhysicalDeviceQueueFamilyProperties\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetPhysicalDeviceQueueFamilyProperties.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetPhysicalDeviceQueueFamilyProperties'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetPhysicalDeviceSparseImageFormatProperties = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetPhysicalDeviceSparseImageFormatProperties>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetPhysicalDeviceSparseImageFormatProperties\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetPhysicalDeviceSparseImageFormatProperties.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetPhysicalDeviceSparseImageFormatProperties'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetPhysicalDeviceFeatures2 = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetPhysicalDeviceFeatures2>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetPhysicalDeviceFeatures2\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkGetPhysicalDeviceProperties2 = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetPhysicalDeviceProperties2>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetPhysicalDeviceProperties2\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkDestroySurfaceKHR = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroySurfaceKHR>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkDestroySurfaceKHR\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkDestroySurfaceKHR.is_none() {
            WARN!("Could not get instance proc addr for 'vkDestroySurfaceKHR'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetPhysicalDeviceSurfaceCapabilitiesKHR = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetPhysicalDeviceSurfaceCapabilitiesKHR>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetPhysicalDeviceSurfaceCapabilitiesKHR\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetPhysicalDeviceSurfaceCapabilitiesKHR.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetPhysicalDeviceSurfaceCapabilitiesKHR'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetPhysicalDeviceSurfaceFormatsKHR = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetPhysicalDeviceSurfaceFormatsKHR>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetPhysicalDeviceSurfaceFormatsKHR\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetPhysicalDeviceSurfaceFormatsKHR.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetPhysicalDeviceSurfaceFormatsKHR'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetPhysicalDeviceSurfacePresentModesKHR = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetPhysicalDeviceSurfacePresentModesKHR>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetPhysicalDeviceSurfacePresentModesKHR\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetPhysicalDeviceSurfacePresentModesKHR.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetPhysicalDeviceSurfacePresentModesKHR'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetPhysicalDeviceSurfaceSupportKHR = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetPhysicalDeviceSurfaceSupportKHR>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetPhysicalDeviceSurfaceSupportKHR\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetPhysicalDeviceSurfaceSupportKHR.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetPhysicalDeviceSurfaceSupportKHR'.\n");
            break 'fail;
        }
        (*vk_ops).vkCreateWin32SurfaceKHR = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateWin32SurfaceKHR>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCreateWin32SurfaceKHR\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCreateWin32SurfaceKHR.is_none() {
            WARN!("Could not get instance proc addr for 'vkCreateWin32SurfaceKHR'.\n");
            break 'fail;
        }
        (*vk_ops).vkResetQueryPoolEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkResetQueryPoolEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkResetQueryPoolEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkAllocateCommandBuffers = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkAllocateCommandBuffers>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkAllocateCommandBuffers\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkAllocateCommandBuffers.is_none() {
            WARN!("Could not get instance proc addr for 'vkAllocateCommandBuffers'.\n");
            break 'fail;
        }
        (*vk_ops).vkAllocateDescriptorSets = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkAllocateDescriptorSets>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkAllocateDescriptorSets\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkAllocateDescriptorSets.is_none() {
            WARN!("Could not get instance proc addr for 'vkAllocateDescriptorSets'.\n");
            break 'fail;
        }
        (*vk_ops).vkAllocateMemory = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkAllocateMemory>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkAllocateMemory\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkAllocateMemory.is_none() {
            WARN!("Could not get instance proc addr for 'vkAllocateMemory'.\n");
            break 'fail;
        }
        (*vk_ops).vkBeginCommandBuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkBeginCommandBuffer>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkBeginCommandBuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkBeginCommandBuffer.is_none() {
            WARN!("Could not get instance proc addr for 'vkBeginCommandBuffer'.\n");
            break 'fail;
        }
        (*vk_ops).vkBindBufferMemory = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkBindBufferMemory>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkBindBufferMemory\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkBindBufferMemory.is_none() {
            WARN!("Could not get instance proc addr for 'vkBindBufferMemory'.\n");
            break 'fail;
        }
        (*vk_ops).vkBindImageMemory = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkBindImageMemory>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkBindImageMemory\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkBindImageMemory.is_none() {
            WARN!("Could not get instance proc addr for 'vkBindImageMemory'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdBeginQuery = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdBeginQuery>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdBeginQuery\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdBeginQuery.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdBeginQuery'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdBeginRenderPass = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdBeginRenderPass>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdBeginRenderPass\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdBeginRenderPass.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdBeginRenderPass'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdBindDescriptorSets = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdBindDescriptorSets>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdBindDescriptorSets\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdBindDescriptorSets.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdBindDescriptorSets'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdBindIndexBuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdBindIndexBuffer>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdBindIndexBuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdBindIndexBuffer.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdBindIndexBuffer'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdBindPipeline = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdBindPipeline>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdBindPipeline\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdBindPipeline.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdBindPipeline'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdBindVertexBuffers = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdBindVertexBuffers>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdBindVertexBuffers\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdBindVertexBuffers.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdBindVertexBuffers'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdBlitImage = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdBlitImage>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdBlitImage\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdBlitImage.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdBlitImage'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdClearAttachments = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdClearAttachments>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdClearAttachments\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdClearAttachments.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdClearAttachments'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdClearColorImage = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdClearColorImage>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdClearColorImage\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdClearColorImage.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdClearColorImage'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdClearDepthStencilImage = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdClearDepthStencilImage>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdClearDepthStencilImage\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdClearDepthStencilImage.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdClearDepthStencilImage'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdCopyBuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdCopyBuffer>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdCopyBuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdCopyBuffer.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdCopyBuffer'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdCopyBufferToImage = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdCopyBufferToImage>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdCopyBufferToImage\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdCopyBufferToImage.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdCopyBufferToImage'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdCopyImage = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdCopyImage>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdCopyImage\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdCopyImage.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdCopyImage'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdCopyImageToBuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdCopyImageToBuffer>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdCopyImageToBuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdCopyImageToBuffer.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdCopyImageToBuffer'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdCopyQueryPoolResults = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdCopyQueryPoolResults>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdCopyQueryPoolResults\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdCopyQueryPoolResults.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdCopyQueryPoolResults'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdDispatch = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdDispatch>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdDispatch\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdDispatch.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdDispatch'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdDispatchIndirect = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdDispatchIndirect>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdDispatchIndirect\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdDispatchIndirect.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdDispatchIndirect'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdDraw = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdDraw>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdDraw\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdDraw.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdDraw'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdDrawIndexed = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdDrawIndexed>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdDrawIndexed\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdDrawIndexed.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdDrawIndexed'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdDrawIndexedIndirect = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdDrawIndexedIndirect>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdDrawIndexedIndirect\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdDrawIndexedIndirect.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdDrawIndexedIndirect'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdDrawIndirect = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdDrawIndirect>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdDrawIndirect\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdDrawIndirect.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdDrawIndirect'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdEndQuery = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdEndQuery>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdEndQuery\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdEndQuery.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdEndQuery'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdEndRenderPass = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdEndRenderPass>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdEndRenderPass\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdEndRenderPass.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdEndRenderPass'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdExecuteCommands = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdExecuteCommands>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdExecuteCommands\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdExecuteCommands.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdExecuteCommands'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdFillBuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdFillBuffer>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdFillBuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdFillBuffer.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdFillBuffer'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdNextSubpass = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdNextSubpass>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdNextSubpass\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdNextSubpass.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdNextSubpass'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdPipelineBarrier = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdPipelineBarrier>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdPipelineBarrier\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdPipelineBarrier.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdPipelineBarrier'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdPushConstants = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdPushConstants>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdPushConstants\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdPushConstants.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdPushConstants'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdResetEvent = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdResetEvent>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdResetEvent\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdResetEvent.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdResetEvent'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdResetQueryPool = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdResetQueryPool>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdResetQueryPool\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdResetQueryPool.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdResetQueryPool'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdResolveImage = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdResolveImage>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdResolveImage\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdResolveImage.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdResolveImage'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdSetBlendConstants = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetBlendConstants>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetBlendConstants\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdSetBlendConstants.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdSetBlendConstants'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdSetDepthBias = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetDepthBias>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetDepthBias\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdSetDepthBias.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdSetDepthBias'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdSetDepthBounds = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetDepthBounds>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetDepthBounds\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdSetDepthBounds.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdSetDepthBounds'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdSetEvent = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetEvent>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetEvent\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdSetEvent.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdSetEvent'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdSetLineWidth = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetLineWidth>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetLineWidth\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdSetLineWidth.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdSetLineWidth'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdSetScissor = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetScissor>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetScissor\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdSetScissor.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdSetScissor'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdSetStencilCompareMask = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetStencilCompareMask>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetStencilCompareMask\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdSetStencilCompareMask.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdSetStencilCompareMask'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdSetStencilReference = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetStencilReference>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetStencilReference\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdSetStencilReference.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdSetStencilReference'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdSetStencilWriteMask = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetStencilWriteMask>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetStencilWriteMask\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdSetStencilWriteMask.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdSetStencilWriteMask'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdSetViewport = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetViewport>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetViewport\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdSetViewport.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdSetViewport'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdUpdateBuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdUpdateBuffer>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdUpdateBuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdUpdateBuffer.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdUpdateBuffer'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdWaitEvents = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdWaitEvents>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdWaitEvents\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdWaitEvents.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdWaitEvents'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdWriteTimestamp = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdWriteTimestamp>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdWriteTimestamp\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCmdWriteTimestamp.is_none() {
            WARN!("Could not get instance proc addr for 'vkCmdWriteTimestamp'.\n");
            break 'fail;
        }
        (*vk_ops).vkCreateBuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateBuffer>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCreateBuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCreateBuffer.is_none() {
            WARN!("Could not get instance proc addr for 'vkCreateBuffer'.\n");
            break 'fail;
        }
        (*vk_ops).vkCreateBufferView = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateBufferView>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCreateBufferView\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCreateBufferView.is_none() {
            WARN!("Could not get instance proc addr for 'vkCreateBufferView'.\n");
            break 'fail;
        }
        (*vk_ops).vkCreateCommandPool = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateCommandPool>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCreateCommandPool\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCreateCommandPool.is_none() {
            WARN!("Could not get instance proc addr for 'vkCreateCommandPool'.\n");
            break 'fail;
        }
        (*vk_ops).vkCreateComputePipelines = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateComputePipelines>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCreateComputePipelines\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCreateComputePipelines.is_none() {
            WARN!("Could not get instance proc addr for 'vkCreateComputePipelines'.\n");
            break 'fail;
        }
        (*vk_ops).vkCreateDescriptorPool = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateDescriptorPool>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCreateDescriptorPool\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCreateDescriptorPool.is_none() {
            WARN!("Could not get instance proc addr for 'vkCreateDescriptorPool'.\n");
            break 'fail;
        }
        (*vk_ops).vkCreateDescriptorSetLayout = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateDescriptorSetLayout>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCreateDescriptorSetLayout\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCreateDescriptorSetLayout.is_none() {
            WARN!("Could not get instance proc addr for 'vkCreateDescriptorSetLayout'.\n");
            break 'fail;
        }
        (*vk_ops).vkCreateEvent = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateEvent>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCreateEvent\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCreateEvent.is_none() {
            WARN!("Could not get instance proc addr for 'vkCreateEvent'.\n");
            break 'fail;
        }
        (*vk_ops).vkCreateFence = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateFence>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCreateFence\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCreateFence.is_none() {
            WARN!("Could not get instance proc addr for 'vkCreateFence'.\n");
            break 'fail;
        }
        (*vk_ops).vkCreateFramebuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateFramebuffer>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCreateFramebuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCreateFramebuffer.is_none() {
            WARN!("Could not get instance proc addr for 'vkCreateFramebuffer'.\n");
            break 'fail;
        }
        (*vk_ops).vkCreateGraphicsPipelines = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateGraphicsPipelines>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCreateGraphicsPipelines\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCreateGraphicsPipelines.is_none() {
            WARN!("Could not get instance proc addr for 'vkCreateGraphicsPipelines'.\n");
            break 'fail;
        }
        (*vk_ops).vkCreateImage = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateImage>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCreateImage\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCreateImage.is_none() {
            WARN!("Could not get instance proc addr for 'vkCreateImage'.\n");
            break 'fail;
        }
        (*vk_ops).vkCreateImageView = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateImageView>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCreateImageView\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCreateImageView.is_none() {
            WARN!("Could not get instance proc addr for 'vkCreateImageView'.\n");
            break 'fail;
        }
        (*vk_ops).vkCreatePipelineCache = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreatePipelineCache>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCreatePipelineCache\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCreatePipelineCache.is_none() {
            WARN!("Could not get instance proc addr for 'vkCreatePipelineCache'.\n");
            break 'fail;
        }
        (*vk_ops).vkCreatePipelineLayout = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreatePipelineLayout>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCreatePipelineLayout\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCreatePipelineLayout.is_none() {
            WARN!("Could not get instance proc addr for 'vkCreatePipelineLayout'.\n");
            break 'fail;
        }
        (*vk_ops).vkCreateQueryPool = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateQueryPool>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCreateQueryPool\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCreateQueryPool.is_none() {
            WARN!("Could not get instance proc addr for 'vkCreateQueryPool'.\n");
            break 'fail;
        }
        (*vk_ops).vkCreateRenderPass = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateRenderPass>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCreateRenderPass\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCreateRenderPass.is_none() {
            WARN!("Could not get instance proc addr for 'vkCreateRenderPass'.\n");
            break 'fail;
        }
        (*vk_ops).vkCreateSampler = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateSampler>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCreateSampler\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCreateSampler.is_none() {
            WARN!("Could not get instance proc addr for 'vkCreateSampler'.\n");
            break 'fail;
        }
        (*vk_ops).vkCreateSemaphore = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateSemaphore>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCreateSemaphore\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCreateSemaphore.is_none() {
            WARN!("Could not get instance proc addr for 'vkCreateSemaphore'.\n");
            break 'fail;
        }
        (*vk_ops).vkCreateShaderModule = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateShaderModule>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCreateShaderModule\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCreateShaderModule.is_none() {
            WARN!("Could not get instance proc addr for 'vkCreateShaderModule'.\n");
            break 'fail;
        }
        (*vk_ops).vkDestroyBuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyBuffer>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkDestroyBuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkDestroyBuffer.is_none() {
            WARN!("Could not get instance proc addr for 'vkDestroyBuffer'.\n");
            break 'fail;
        }
        (*vk_ops).vkDestroyBufferView = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyBufferView>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkDestroyBufferView\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkDestroyBufferView.is_none() {
            WARN!("Could not get instance proc addr for 'vkDestroyBufferView'.\n");
            break 'fail;
        }
        (*vk_ops).vkDestroyCommandPool = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyCommandPool>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkDestroyCommandPool\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkDestroyCommandPool.is_none() {
            WARN!("Could not get instance proc addr for 'vkDestroyCommandPool'.\n");
            break 'fail;
        }
        (*vk_ops).vkDestroyDescriptorPool = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyDescriptorPool>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkDestroyDescriptorPool\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkDestroyDescriptorPool.is_none() {
            WARN!("Could not get instance proc addr for 'vkDestroyDescriptorPool'.\n");
            break 'fail;
        }
        (*vk_ops).vkDestroyDescriptorSetLayout = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyDescriptorSetLayout>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkDestroyDescriptorSetLayout\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkDestroyDescriptorSetLayout.is_none() {
            WARN!("Could not get instance proc addr for 'vkDestroyDescriptorSetLayout'.\n");
            break 'fail;
        }
        (*vk_ops).vkDestroyDevice = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyDevice>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkDestroyDevice\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkDestroyDevice.is_none() {
            WARN!("Could not get instance proc addr for 'vkDestroyDevice'.\n");
            break 'fail;
        }
        (*vk_ops).vkDestroyEvent = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyEvent>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkDestroyEvent\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkDestroyEvent.is_none() {
            WARN!("Could not get instance proc addr for 'vkDestroyEvent'.\n");
            break 'fail;
        }
        (*vk_ops).vkDestroyFence = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyFence>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkDestroyFence\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkDestroyFence.is_none() {
            WARN!("Could not get instance proc addr for 'vkDestroyFence'.\n");
            break 'fail;
        }
        (*vk_ops).vkDestroyFramebuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyFramebuffer>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkDestroyFramebuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkDestroyFramebuffer.is_none() {
            WARN!("Could not get instance proc addr for 'vkDestroyFramebuffer'.\n");
            break 'fail;
        }
        (*vk_ops).vkDestroyImage = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyImage>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkDestroyImage\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkDestroyImage.is_none() {
            WARN!("Could not get instance proc addr for 'vkDestroyImage'.\n");
            break 'fail;
        }
        (*vk_ops).vkDestroyImageView = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyImageView>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkDestroyImageView\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkDestroyImageView.is_none() {
            WARN!("Could not get instance proc addr for 'vkDestroyImageView'.\n");
            break 'fail;
        }
        (*vk_ops).vkDestroyPipeline = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyPipeline>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkDestroyPipeline\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkDestroyPipeline.is_none() {
            WARN!("Could not get instance proc addr for 'vkDestroyPipeline'.\n");
            break 'fail;
        }
        (*vk_ops).vkDestroyPipelineCache = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyPipelineCache>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkDestroyPipelineCache\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkDestroyPipelineCache.is_none() {
            WARN!("Could not get instance proc addr for 'vkDestroyPipelineCache'.\n");
            break 'fail;
        }
        (*vk_ops).vkDestroyPipelineLayout = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyPipelineLayout>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkDestroyPipelineLayout\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkDestroyPipelineLayout.is_none() {
            WARN!("Could not get instance proc addr for 'vkDestroyPipelineLayout'.\n");
            break 'fail;
        }
        (*vk_ops).vkDestroyQueryPool = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyQueryPool>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkDestroyQueryPool\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkDestroyQueryPool.is_none() {
            WARN!("Could not get instance proc addr for 'vkDestroyQueryPool'.\n");
            break 'fail;
        }
        (*vk_ops).vkDestroyRenderPass = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyRenderPass>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkDestroyRenderPass\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkDestroyRenderPass.is_none() {
            WARN!("Could not get instance proc addr for 'vkDestroyRenderPass'.\n");
            break 'fail;
        }
        (*vk_ops).vkDestroySampler = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroySampler>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkDestroySampler\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkDestroySampler.is_none() {
            WARN!("Could not get instance proc addr for 'vkDestroySampler'.\n");
            break 'fail;
        }
        (*vk_ops).vkDestroySemaphore = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroySemaphore>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkDestroySemaphore\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkDestroySemaphore.is_none() {
            WARN!("Could not get instance proc addr for 'vkDestroySemaphore'.\n");
            break 'fail;
        }
        (*vk_ops).vkDestroyShaderModule = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroyShaderModule>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkDestroyShaderModule\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkDestroyShaderModule.is_none() {
            WARN!("Could not get instance proc addr for 'vkDestroyShaderModule'.\n");
            break 'fail;
        }
        (*vk_ops).vkDeviceWaitIdle = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDeviceWaitIdle>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkDeviceWaitIdle\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkDeviceWaitIdle.is_none() {
            WARN!("Could not get instance proc addr for 'vkDeviceWaitIdle'.\n");
            break 'fail;
        }
        (*vk_ops).vkEndCommandBuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkEndCommandBuffer>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkEndCommandBuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkEndCommandBuffer.is_none() {
            WARN!("Could not get instance proc addr for 'vkEndCommandBuffer'.\n");
            break 'fail;
        }
        (*vk_ops).vkFlushMappedMemoryRanges = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkFlushMappedMemoryRanges>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkFlushMappedMemoryRanges\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkFlushMappedMemoryRanges.is_none() {
            WARN!("Could not get instance proc addr for 'vkFlushMappedMemoryRanges'.\n");
            break 'fail;
        }
        (*vk_ops).vkFreeCommandBuffers = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkFreeCommandBuffers>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkFreeCommandBuffers\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkFreeCommandBuffers.is_none() {
            WARN!("Could not get instance proc addr for 'vkFreeCommandBuffers'.\n");
            break 'fail;
        }
        (*vk_ops).vkFreeDescriptorSets = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkFreeDescriptorSets>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkFreeDescriptorSets\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkFreeDescriptorSets.is_none() {
            WARN!("Could not get instance proc addr for 'vkFreeDescriptorSets'.\n");
            break 'fail;
        }
        (*vk_ops).vkFreeMemory = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkFreeMemory>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkFreeMemory\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkFreeMemory.is_none() {
            WARN!("Could not get instance proc addr for 'vkFreeMemory'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetBufferMemoryRequirements = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetBufferMemoryRequirements>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetBufferMemoryRequirements\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetBufferMemoryRequirements.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetBufferMemoryRequirements'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetDeviceMemoryCommitment = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetDeviceMemoryCommitment>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetDeviceMemoryCommitment\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetDeviceMemoryCommitment.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetDeviceMemoryCommitment'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetDeviceQueue = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetDeviceQueue>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetDeviceQueue\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetDeviceQueue.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetDeviceQueue'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetEventStatus = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetEventStatus>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetEventStatus\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetEventStatus.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetEventStatus'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetFenceStatus = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetFenceStatus>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetFenceStatus\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetFenceStatus.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetFenceStatus'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetImageMemoryRequirements = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetImageMemoryRequirements>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetImageMemoryRequirements\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetImageMemoryRequirements.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetImageMemoryRequirements'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetImageSparseMemoryRequirements = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetImageSparseMemoryRequirements>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetImageSparseMemoryRequirements\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetImageSparseMemoryRequirements.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetImageSparseMemoryRequirements'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetImageSubresourceLayout = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetImageSubresourceLayout>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetImageSubresourceLayout\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetImageSubresourceLayout.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetImageSubresourceLayout'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetPipelineCacheData = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetPipelineCacheData>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetPipelineCacheData\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetPipelineCacheData.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetPipelineCacheData'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetQueryPoolResults = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetQueryPoolResults>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetQueryPoolResults\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetQueryPoolResults.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetQueryPoolResults'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetRenderAreaGranularity = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetRenderAreaGranularity>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetRenderAreaGranularity\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetRenderAreaGranularity.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetRenderAreaGranularity'.\n");
            break 'fail;
        }
        (*vk_ops).vkInvalidateMappedMemoryRanges = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkInvalidateMappedMemoryRanges>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkInvalidateMappedMemoryRanges\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkInvalidateMappedMemoryRanges.is_none() {
            WARN!("Could not get instance proc addr for 'vkInvalidateMappedMemoryRanges'.\n");
            break 'fail;
        }
        (*vk_ops).vkMapMemory = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkMapMemory>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkMapMemory\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkMapMemory.is_none() {
            WARN!("Could not get instance proc addr for 'vkMapMemory'.\n");
            break 'fail;
        }
        (*vk_ops).vkMergePipelineCaches = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkMergePipelineCaches>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkMergePipelineCaches\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkMergePipelineCaches.is_none() {
            WARN!("Could not get instance proc addr for 'vkMergePipelineCaches'.\n");
            break 'fail;
        }
        (*vk_ops).vkQueueBindSparse = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkQueueBindSparse>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkQueueBindSparse\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkQueueBindSparse.is_none() {
            WARN!("Could not get instance proc addr for 'vkQueueBindSparse'.\n");
            break 'fail;
        }
        (*vk_ops).vkQueueSubmit = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkQueueSubmit>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkQueueSubmit\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkQueueSubmit.is_none() {
            WARN!("Could not get instance proc addr for 'vkQueueSubmit'.\n");
            break 'fail;
        }
        (*vk_ops).vkQueueWaitIdle = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkQueueWaitIdle>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkQueueWaitIdle\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkQueueWaitIdle.is_none() {
            WARN!("Could not get instance proc addr for 'vkQueueWaitIdle'.\n");
            break 'fail;
        }
        (*vk_ops).vkResetCommandBuffer = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkResetCommandBuffer>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkResetCommandBuffer\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkResetCommandBuffer.is_none() {
            WARN!("Could not get instance proc addr for 'vkResetCommandBuffer'.\n");
            break 'fail;
        }
        (*vk_ops).vkResetCommandPool = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkResetCommandPool>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkResetCommandPool\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkResetCommandPool.is_none() {
            WARN!("Could not get instance proc addr for 'vkResetCommandPool'.\n");
            break 'fail;
        }
        (*vk_ops).vkResetDescriptorPool = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkResetDescriptorPool>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkResetDescriptorPool\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkResetDescriptorPool.is_none() {
            WARN!("Could not get instance proc addr for 'vkResetDescriptorPool'.\n");
            break 'fail;
        }
        (*vk_ops).vkResetEvent = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkResetEvent>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkResetEvent\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkResetEvent.is_none() {
            WARN!("Could not get instance proc addr for 'vkResetEvent'.\n");
            break 'fail;
        }
        (*vk_ops).vkResetFences = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkResetFences>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkResetFences\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkResetFences.is_none() {
            WARN!("Could not get instance proc addr for 'vkResetFences'.\n");
            break 'fail;
        }
        (*vk_ops).vkSetEvent = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkSetEvent>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkSetEvent\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkSetEvent.is_none() {
            WARN!("Could not get instance proc addr for 'vkSetEvent'.\n");
            break 'fail;
        }
        (*vk_ops).vkUnmapMemory = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkUnmapMemory>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkUnmapMemory\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkUnmapMemory.is_none() {
            WARN!("Could not get instance proc addr for 'vkUnmapMemory'.\n");
            break 'fail;
        }
        (*vk_ops).vkUpdateDescriptorSets = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkUpdateDescriptorSets>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkUpdateDescriptorSets\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkUpdateDescriptorSets.is_none() {
            WARN!("Could not get instance proc addr for 'vkUpdateDescriptorSets'.\n");
            break 'fail;
        }
        (*vk_ops).vkWaitForFences = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkWaitForFences>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkWaitForFences\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkWaitForFences.is_none() {
            WARN!("Could not get instance proc addr for 'vkWaitForFences'.\n");
            break 'fail;
        }
        (*vk_ops).vkCmdSetDepthCompareOpEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetDepthCompareOpEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetDepthCompareOpEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkCmdSetDepthTestEnableEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetDepthTestEnableEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetDepthTestEnableEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkCmdSetDepthWriteEnableEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetDepthWriteEnableEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetDepthWriteEnableEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkCmdSetPrimitiveTopologyEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetPrimitiveTopologyEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetPrimitiveTopologyEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkCmdSetStencilOpEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetStencilOpEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetStencilOpEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkCmdSetStencilTestEnableEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetStencilTestEnableEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetStencilTestEnableEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkCmdSetPatchControlPointsEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetPatchControlPointsEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetPatchControlPointsEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkCmdSetPrimitiveRestartEnableEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetPrimitiveRestartEnableEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetPrimitiveRestartEnableEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkCmdSetAlphaToCoverageEnableEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetAlphaToCoverageEnableEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetAlphaToCoverageEnableEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkCmdSetColorBlendEnableEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetColorBlendEnableEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetColorBlendEnableEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkCmdSetColorBlendEquationEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetColorBlendEquationEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetColorBlendEquationEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkCmdSetColorWriteMaskEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetColorWriteMaskEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetColorWriteMaskEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkCmdSetCullModeEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetCullModeEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetCullModeEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkCmdSetDepthBiasEnableEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetDepthBiasEnableEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetDepthBiasEnableEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkCmdSetDepthClampEnableEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetDepthClampEnableEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetDepthClampEnableEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkCmdSetFrontFaceEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetFrontFaceEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetFrontFaceEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkCmdSetRasterizationSamplesEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetRasterizationSamplesEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetRasterizationSamplesEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkCmdSetRasterizerDiscardEnableEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetRasterizerDiscardEnableEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetRasterizerDiscardEnableEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkCmdSetSampleMaskEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdSetSampleMaskEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdSetSampleMaskEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkCmdBeginQueryIndexedEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdBeginQueryIndexedEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdBeginQueryIndexedEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkCmdBeginTransformFeedbackEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdBeginTransformFeedbackEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdBeginTransformFeedbackEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkCmdBindTransformFeedbackBuffersEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdBindTransformFeedbackBuffersEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdBindTransformFeedbackBuffersEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkCmdEndQueryIndexedEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdEndQueryIndexedEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdEndQueryIndexedEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkCmdEndTransformFeedbackEXT = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCmdEndTransformFeedbackEXT>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCmdEndTransformFeedbackEXT\0".as_ptr() as *const core::ffi::c_char));
        (*vk_ops).vkAcquireNextImageKHR = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkAcquireNextImageKHR>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkAcquireNextImageKHR\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkAcquireNextImageKHR.is_none() {
            WARN!("Could not get instance proc addr for 'vkAcquireNextImageKHR'.\n");
            break 'fail;
        }
        (*vk_ops).vkCreateSwapchainKHR = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkCreateSwapchainKHR>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkCreateSwapchainKHR\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkCreateSwapchainKHR.is_none() {
            WARN!("Could not get instance proc addr for 'vkCreateSwapchainKHR'.\n");
            break 'fail;
        }
        (*vk_ops).vkDestroySwapchainKHR = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkDestroySwapchainKHR>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkDestroySwapchainKHR\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkDestroySwapchainKHR.is_none() {
            WARN!("Could not get instance proc addr for 'vkDestroySwapchainKHR'.\n");
            break 'fail;
        }
        (*vk_ops).vkGetSwapchainImagesKHR = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkGetSwapchainImagesKHR>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkGetSwapchainImagesKHR\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkGetSwapchainImagesKHR.is_none() {
            WARN!("Could not get instance proc addr for 'vkGetSwapchainImagesKHR'.\n");
            break 'fail;
        }
        (*vk_ops).vkQueuePresentKHR = core::mem::transmute::<PFN_vkVoidFunction, PFN_vkQueuePresentKHR>((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked()(instance, b"vkQueuePresentKHR\0".as_ptr() as *const core::ffi::c_char));
        if (*vk_ops).vkQueuePresentKHR.is_none() {
            WARN!("Could not get instance proc addr for 'vkQueuePresentKHR'.\n");
            break 'fail;
        }

        if (*vk_ops).vkGetPhysicalDeviceProperties2.is_none() {
            (*vk_ops).vkGetPhysicalDeviceProperties2 = ::core::mem::transmute::<*mut ::core::ffi::c_void, PFN_vkGetPhysicalDeviceProperties2>(
                ::core::mem::transmute::<PFN_vkVoidFunction, *mut ::core::ffi::c_void>(((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked())(
                        instance,
                        b"vkGetPhysicalDeviceProperties2KHR\0".as_ptr() as *const ::core::ffi::c_char,
                    )),
            );
        }
        if (*vk_ops).vkGetPhysicalDeviceFeatures2.is_none() {
            (*vk_ops).vkGetPhysicalDeviceFeatures2 = ::core::mem::transmute::<*mut ::core::ffi::c_void, PFN_vkGetPhysicalDeviceFeatures2>(
                ::core::mem::transmute::<PFN_vkVoidFunction, *mut ::core::ffi::c_void>(((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked())(
                        instance,
                        b"vkGetPhysicalDeviceFeatures2KHR\0".as_ptr() as *const ::core::ffi::c_char,
                    )),
            );
        }

        (*vk_info).instance = instance;

        return (TRUE) as BOOL;
    }

    // fail:
    if (*vk_ops).vkDestroyInstance.is_some() {
        ((*vk_info).vk_ops.vkDestroyInstance.unwrap_unchecked())(instance, ::core::ptr::null_mut::<VkAllocationCallbacks>());
    }
    wined3d_unload_vulkan(vk_info);
    return (FALSE) as BOOL;
}

unsafe extern "C" fn get_vulkan_physical_device(vk_info: *mut wined3d_vk_info) -> VkPhysicalDevice {
    let mut physical_devices: [VkPhysicalDevice; 1] = core::mem::zeroed();
    let mut count: uint32_t = 0;
    let mut vr: VkResult;

    vr = ((*vk_info).vk_ops.vkEnumeratePhysicalDevices.unwrap_unchecked())((*vk_info).instance, core::ptr::addr_of_mut!(count), core::ptr::null_mut());
    if vr < 0 {
        WARN!("Failed to enumerate physical devices, vr %s.\n", wined3d_debug_vkresult(vr));
        return core::ptr::null_mut();
    }
    if count == 0 {
        WARN!("No physical device.\n");
        return core::ptr::null_mut();
    }
    if count > 1 {
        /* TODO: Create wined3d_adapter for each device. */

        count = 1;
    }

    vr = ((*vk_info).vk_ops.vkEnumeratePhysicalDevices.unwrap_unchecked())((*vk_info).instance, core::ptr::addr_of_mut!(count), core::ptr::addr_of_mut!(physical_devices) as *mut VkPhysicalDevice);
    if vr < 0 {
        WARN!("Failed to get physical devices, vr %s.\n", wined3d_debug_vkresult(vr));
        return core::ptr::null_mut();
    }

    return physical_devices[0];
}

unsafe extern "C" fn guess_display_driver(mut vendor: wined3d_pci_vendor) -> wined3d_display_driver {
    match vendor as ::core::ffi::c_uint {
        HW_VENDOR_AMD => return DRIVER_AMD_RX,
        HW_VENDOR_INTEL => return DRIVER_INTEL_HD4000,
        HW_VENDOR_NVIDIA => return DRIVER_NVIDIA_GEFORCE8,
        _ => return DRIVER_WINE,
    };
}

unsafe extern "C" fn adapter_vk_init_driver_info(
    mut adapter_vk: *mut wined3d_adapter_vk,
    mut properties: *const VkPhysicalDeviceProperties,
) -> bool {
    let mut memory_properties: *const VkPhysicalDeviceMemoryProperties =
        &raw mut (*adapter_vk).memory_properties;
    let mut gpu_description: *const wined3d_gpu_description =
        ::core::ptr::null::<wined3d_gpu_description>();
    let mut description: wined3d_gpu_description = wined3d_gpu_description {
        vendor: HW_VENDOR_SOFTWARE,
        device: CARD_WINE,
        description: ::core::ptr::null::<::core::ffi::c_char>(),
        driver: DRIVER_AMD_RAGE_128PRO,
        vidmem: 0,
    };
    let mut vram_bytes: UINT64 = 0;
    let mut sysmem_bytes: UINT64 = 0;
    let mut type_0: *const VkMemoryType = ::core::ptr::null::<VkMemoryType>();
    let mut heap: *const VkMemoryHeap = ::core::ptr::null::<VkMemoryHeap>();
    let mut i: ::core::ffi::c_uint = 0;
    WINE_TRACE!("Device name: %s.\n", debugstr_a(&raw const (*properties).deviceName as *const ::core::ffi::c_char));
    WINE_TRACE!("Vendor ID: 0x%04x, Device ID: 0x%04x.\n", (*properties).vendorID, (*properties).deviceID);
    WINE_TRACE!("Driver version: %#x.\n", (*properties).driverVersion);
    WINE_TRACE!("API version: %s.\n", debug_vk_version((*properties).apiVersion));
    i = 0 as ::core::ffi::c_uint;
    vram_bytes = 0 as UINT64;
    sysmem_bytes = 0 as UINT64;
    while (i as uint32_t) < (*memory_properties).memoryHeapCount {
        heap = (&raw const (*memory_properties).memoryHeaps as *const VkMemoryHeap)
            .offset(i as isize) as *const VkMemoryHeap;
        WINE_TRACE!("Memory heap [%u]: flags %#x, size 0x%s.\n", i, (*heap).flags, wine_dbgstr_longlong((*heap).size as ULONGLONG));
        if (*heap).flags
            & VK_MEMORY_HEAP_DEVICE_LOCAL_BIT as ::core::ffi::c_int as VkMemoryHeapFlags
            != 0
        {
            vram_bytes = (vram_bytes as ::core::ffi::c_ulonglong)
                .wrapping_add((*heap).size as ::core::ffi::c_ulonglong)
                as UINT64 as UINT64;
        } else {
            sysmem_bytes = (sysmem_bytes as ::core::ffi::c_ulonglong)
                .wrapping_add((*heap).size as ::core::ffi::c_ulonglong)
                as UINT64 as UINT64;
        }
        i = i.wrapping_add(1);
    }
    WINE_TRACE!("Total device memory: 0x%s.\n", wine_dbgstr_longlong(vram_bytes as ULONGLONG));
    WINE_TRACE!("Total shared system memory: 0x%s.\n", wine_dbgstr_longlong(sysmem_bytes as ULONGLONG));
    i = 0 as ::core::ffi::c_uint;
    while (i as uint32_t) < (*memory_properties).memoryTypeCount {
        type_0 = (&raw const (*memory_properties).memoryTypes as *const VkMemoryType)
            .offset(i as isize) as *const VkMemoryType;
        WINE_TRACE!("Memory type [%u]: flags %#x, heap %u.\n", i, (*type_0).propertyFlags, (*type_0).heapIndex);
        i = i.wrapping_add(1);
    }
    gpu_description = wined3d_get_user_override_gpu_description(
        (*properties).vendorID as wined3d_pci_vendor,
        (*properties).deviceID as wined3d_pci_device,
    );
    if gpu_description.is_null() {
        gpu_description = wined3d_get_gpu_description(
            (*properties).vendorID as wined3d_pci_vendor,
            (*properties).deviceID as wined3d_pci_device,
        );
    }
    if gpu_description.is_null() {
        WINE_FIXME!("Failed to retrieve GPU description for device %s %04x:%04x.\n", debugstr_a( &raw const (*properties).deviceName as *const ::core::ffi::c_char, ), (*properties).vendorID, (*properties).deviceID);
        description.vendor = (*properties).vendorID as wined3d_pci_vendor;
        description.device = (*properties).deviceID as wined3d_pci_device;
        description.description = &raw const (*properties).deviceName as *const ::core::ffi::c_char;
        description.driver = guess_display_driver((*properties).vendorID as wined3d_pci_vendor);
        description.vidmem = vram_bytes as ::core::ffi::c_uint;
        gpu_description = &raw mut description;
    }
    return wined3d_driver_info_init(
        &raw mut (*adapter_vk).a.driver_info,
        gpu_description,
        (*adapter_vk).a.d3d_info.feature_level,
        vram_bytes,
        sysmem_bytes,
    );
}

unsafe extern "C" fn feature_level_9_2_supported(info: *const wined3d_physical_device_info) -> bool {
    return (*info).features2.features.occlusionQueryPrecise != 0;
}

unsafe extern "C" fn feature_level_9_3_supported(info: *const wined3d_physical_device_info, shader_model: ::core::ffi::c_uint) -> bool {
    return shader_model >= 3
        && (*info).features2.features.independentBlend != 0;
}

unsafe extern "C" fn feature_level_10_supported(info: *const wined3d_physical_device_info, shader_model: ::core::ffi::c_uint) -> bool {
    return shader_model >= 4
        && (*info).features2.features.multiViewport != 0
        && (*info).features2.features.geometryShader != 0
        && (*info).features2.features.depthClamp != 0
        && (*info).features2.features.depthBiasClamp != 0
        && (*info).features2.features.pipelineStatisticsQuery != 0
        && (*info).features2.features.shaderClipDistance != 0
        && (*info).features2.features.shaderCullDistance != 0
        && (*info).draw_parameters_features.shaderDrawParameters != 0
        && (*info).vertex_divisor_features.vertexAttributeInstanceRateDivisor != 0
        && (*info).vertex_divisor_features.vertexAttributeInstanceRateZeroDivisor != 0;
}

unsafe extern "C" fn feature_level_10_1_supported(info: *const wined3d_physical_device_info, shader_model: core::ffi::c_uint) -> bool {
    return (*info).features2.features.imageCubeArray != 0;
}

unsafe extern "C" fn feature_level_11_supported(info: *const wined3d_physical_device_info, shader_model: core::ffi::c_uint) -> bool {
    return shader_model >= 5
        && (*info).features2.features.multiDrawIndirect != 0
        && (*info).features2.features.drawIndirectFirstInstance != 0
        && (*info).features2.features.fragmentStoresAndAtomics != 0
        && (*info).features2.features.shaderImageGatherExtended != 0
        && (*info).features2.features.tessellationShader != 0;
}

unsafe extern "C" fn feature_level_11_1_supported(info: *const wined3d_physical_device_info) -> bool {
    return (*info).features2.features.vertexPipelineStoresAndAtomics != 0;
}

unsafe extern "C" fn feature_level_from_caps(info: *const wined3d_physical_device_info, shader_caps: *const shader_caps) -> wined3d_feature_level {
    let mut shader_model: u32;

    shader_model = min!((*shader_caps).vs_version, (*shader_caps).ps_version);
    shader_model = min!(shader_model, max!((*shader_caps).gs_version, 3));
    shader_model = min!(shader_model, max!((*shader_caps).hs_version, 4));
    shader_model = min!(shader_model, max!((*shader_caps).ds_version, 4));

    if shader_model == 0 {
        return WINED3D_FEATURE_LEVEL_7;
    }

    if shader_model <= 1 {
        return WINED3D_FEATURE_LEVEL_8;
    }

    if !feature_level_9_2_supported(info) {
        return WINED3D_FEATURE_LEVEL_9_1;
    }

    if !feature_level_9_3_supported(info, shader_model) {
        return WINED3D_FEATURE_LEVEL_9_2;
    }

    if !feature_level_10_supported(info, shader_model) {
        return WINED3D_FEATURE_LEVEL_9_3;
    }

    if !feature_level_10_1_supported(info, shader_model) {
        return WINED3D_FEATURE_LEVEL_10;
    }

    if !feature_level_11_supported(info, shader_model) {
        return WINED3D_FEATURE_LEVEL_10_1;
    }

    if !feature_level_11_1_supported(info) {
        return WINED3D_FEATURE_LEVEL_11;
    }

    return WINED3D_FEATURE_LEVEL_11_1;
}

unsafe extern "C" fn wined3d_adapter_vk_init_d3d_info(
    mut adapter_vk: *mut wined3d_adapter_vk,
    mut wined3d_creation_flags: uint32_t,
) {
    let mut dynamic_state3: *const VkPhysicalDeviceExtendedDynamicState3FeaturesEXT =
        ::core::ptr::null::<VkPhysicalDeviceExtendedDynamicState3FeaturesEXT>();
    let mut d3d_info: *mut wined3d_d3d_info = &raw mut (*adapter_vk).a.d3d_info;
    let mut vk_info: *mut wined3d_vk_info = &raw mut (*adapter_vk).vk_info;
    let mut device_info: wined3d_physical_device_info = core::mem::zeroed();
    let mut vertex_caps: wined3d_vertex_caps = core::mem::zeroed();
    let mut sample_counts_mask: ::core::ffi::c_uint = 0;
    let mut shader_caps: shader_caps = core::mem::zeroed();

    get_physical_device_info(adapter_vk, &raw mut device_info);

    if device_info.dynamic_state_features.extendedDynamicState == 0 {
        (*adapter_vk).vk_info.supported[(WINED3D_VK_EXT_EXTENDED_DYNAMIC_STATE) as usize] = (FALSE as i32);
    }

    if device_info.host_query_reset_features.hostQueryReset == 0 {
        (*adapter_vk).vk_info.supported[(WINED3D_VK_EXT_HOST_QUERY_RESET) as usize] = (FALSE as i32);
    }

    if device_info.xfb_features.transformFeedback == 0 {
        (*adapter_vk).vk_info.supported[(WINED3D_VK_EXT_TRANSFORM_FEEDBACK) as usize] = (FALSE as i32);
    }

    (*(*adapter_vk).a.shader_backend)
        .shader_get_caps
        .unwrap_unchecked()(&raw mut (*adapter_vk).a, &raw mut shader_caps);
    (*(*adapter_vk).a.vertex_pipe)
        .vp_get_caps
        .unwrap_unchecked()(&raw mut (*adapter_vk).a, &raw mut vertex_caps);
    (*(*adapter_vk).a.fragment_pipe)
        .get_caps
        .unwrap_unchecked()(
        &raw mut (*adapter_vk).a,
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
    (*d3d_info).limits.ffp_vertex_blend_matrices =
        vertex_caps.max_vertex_blend_matrices as ::core::ffi::c_uint;
    (*d3d_info).limits.active_light_count = vertex_caps.max_active_lights as ::core::ffi::c_uint;

    (*d3d_info).limits.max_rt_count = WINED3D_MAX_RENDER_TARGETS as ::core::ffi::c_uint;
    (*d3d_info).limits.max_clip_distances = WINED3D_MAX_CLIP_DISTANCES as ::core::ffi::c_uint;
    (*d3d_info).limits.texture_size =
        (*adapter_vk).device_limits.maxImageDimension2D as ::core::ffi::c_uint;
    (*d3d_info).limits.pointsize_max =
        (*adapter_vk).device_limits.pointSizeRange[1 as usize];
    sample_counts_mask = ((*adapter_vk).device_limits.framebufferColorSampleCounts
        | (*adapter_vk).device_limits.framebufferDepthSampleCounts
        | (*adapter_vk).device_limits.framebufferStencilSampleCounts
        | (*adapter_vk)
            .device_limits
            .framebufferNoAttachmentsSampleCounts) as ::core::ffi::c_uint;
    (*d3d_info).limits.sample_count =
        (1 as ::core::ffi::c_uint) << wined3d_log2i(sample_counts_mask);

    (*d3d_info).wined3d_creation_flags = wined3d_creation_flags;

    (*d3d_info).set_emulated_flatshading(vertex_caps.emulated_flatshading as uint32_t as uint32_t);
    (*d3d_info).set_ffp_alpha_test(0);
    (*d3d_info).set_shader_double_precision(
        (shader_caps.wined3d_caps as ::core::ffi::c_uint & WINED3D_SHADER_CAP_DOUBLE_PRECISION as ::core::ffi::c_uint != 0)
            as ::core::ffi::c_int as uint32_t as uint32_t,
    );
    (*d3d_info).set_shader_output_interpolation(
        (shader_caps.wined3d_caps as ::core::ffi::c_uint & WINED3D_SHADER_CAP_OUTPUT_INTERPOLATION as ::core::ffi::c_uint
            != 0) as ::core::ffi::c_int as uint32_t as uint32_t,
    );
    (*d3d_info).set_viewport_array_index_any_shader(0);
    (*d3d_info).set_stencil_export(
        (*vk_info).supported[(WINED3D_VK_EXT_SHADER_STENCIL_EXPORT) as usize] as uint32_t as uint32_t,
    );
    (*d3d_info).set_unconditional_npot(1);
    (*d3d_info).set_draw_base_vertex_offset(1);
    (*d3d_info).set_vertex_bgra(1);
    (*d3d_info).set_texture_swizzle(1);
    (*d3d_info).set_srgb_read_control(0);
    (*d3d_info).set_srgb_write_control(0);
    (*d3d_info).set_clip_control(1);
    (*d3d_info).set_full_ffp_varyings(
        (shader_caps.wined3d_caps as ::core::ffi::c_uint & WINED3D_SHADER_CAP_FULL_FFP_VARYINGS as ::core::ffi::c_uint
            != 0) as ::core::ffi::c_int as uint32_t as uint32_t,
    );
    (*d3d_info).set_scaled_resolve(0);
    (*d3d_info).set_pbo(1);
    (*d3d_info).feature_level =
        feature_level_from_caps(&raw mut device_info, &raw mut shader_caps);
    (*d3d_info).set_subpixel_viewport(1);
    (*d3d_info).set_fences(1);
    (*d3d_info).set_persistent_map(1);
    (*d3d_info).set_gpu_push_constants(1);
    (*d3d_info).set_ffp_hlsl(1);

    /* Like GL, Vulkan doesn't explicitly specify a filling convention and only mandates that a
     * shared edge of two adjacent triangles generate a fragment for exactly one of the triangles.
     *
     * However, every Vulkan implementation we have seen so far uses a top-left rule. Hardware
     * that differs either predates Vulkan (d3d9 class HW, GeForce 9xxx) or behaves the way we
     * want in Vulkan (MacOS Radeon driver through MoltenVK). */
    (*d3d_info).filling_convention_offset = 0.0f32;

    (*d3d_info).multisample_draw_location = WINED3D_LOCATION_TEXTURE_RGB as DWORD;

    (*vk_info).multiple_viewports = device_info.features2.features.multiViewport != 0;
    (*vk_info).dynamic_state2 = device_info.dynamic_state2_features.extendedDynamicState2 != 0;
    (*vk_info).dynamic_patch_vertex_count = device_info
        .dynamic_state2_features
        .extendedDynamicState2PatchControlPoints
        != 0;

    dynamic_state3 = &raw const device_info.dynamic_state3_features;
    (*vk_info).dynamic_multisample_state =
        (*dynamic_state3).extendedDynamicState3RasterizationSamples != 0
            && (*dynamic_state3).extendedDynamicState3AlphaToCoverageEnable != 0
            && (*dynamic_state3).extendedDynamicState3SampleMask != 0;
    (*vk_info).dynamic_blend_state = (*dynamic_state3).extendedDynamicState3ColorBlendEnable != 0
        && (*dynamic_state3).extendedDynamicState3ColorBlendEquation != 0
        && (*dynamic_state3).extendedDynamicState3ColorWriteMask != 0;
    /* Rasterizer state needs EDS2, for rasterizer discard, and EDS1, for cull mode and front face. */
    (*vk_info).dynamic_rasterizer_state = (*dynamic_state3).extendedDynamicState3DepthClampEnable
        != 0
        && (*vk_info).dynamic_state2 as ::core::ffi::c_int != 0
        && (*adapter_vk).vk_info.supported[(WINED3D_VK_EXT_EXTENDED_DYNAMIC_STATE) as usize] != 0;
}

unsafe extern "C" fn wined3d_adapter_vk_init_device_extensions(
    adapter_vk: *mut wined3d_adapter_vk,
) -> bool {
    struct InfoEntry {
        name: *const core::ffi::c_char,
        core_since_version: core::ffi::c_uint,
        required: bool,
    }
    struct MapEntry {
        name: *const core::ffi::c_char,
        extension: core::ffi::c_int,
    }

    let physical_device: VkPhysicalDevice = (*adapter_vk).physical_device;
    let vk_info: *mut wined3d_vk_info = core::ptr::addr_of_mut!((*adapter_vk).vk_info);
    let mut count: core::ffi::c_uint = 0;
    let mut enable_count: core::ffi::c_uint = 0;
    let mut i: core::ffi::c_uint = 0;
    let mut j: core::ffi::c_uint = 0;
    let mut enabled_extensions: *mut *const core::ffi::c_char = core::ptr::null_mut();
    let mut extensions: *mut VkExtensionProperties = core::ptr::null_mut();
    let mut found: bool = false;
    let mut success: bool = false;
    let mut enable_size: SIZE_T = 0;
    let mut vr: VkResult = VK_SUCCESS;

    let info: [InfoEntry; 12] = [
        InfoEntry { name: (VK_EXT_EXTENDED_DYNAMIC_STATE_EXTENSION_NAME).as_ptr(), core_since_version: VK_API_VERSION_1_3, required: false },
        InfoEntry { name: (VK_EXT_EXTENDED_DYNAMIC_STATE_2_EXTENSION_NAME).as_ptr(), core_since_version: VK_API_VERSION_1_3, required: false },
        InfoEntry { name: (VK_EXT_EXTENDED_DYNAMIC_STATE_3_EXTENSION_NAME).as_ptr(), core_since_version: !(0), required: false },
        InfoEntry { name: (VK_EXT_HOST_QUERY_RESET_EXTENSION_NAME).as_ptr(), core_since_version: VK_API_VERSION_1_2, required: false },
        InfoEntry { name: (VK_EXT_SHADER_STENCIL_EXPORT_EXTENSION_NAME).as_ptr(), core_since_version: !(0), required: false },
        InfoEntry { name: (VK_EXT_TRANSFORM_FEEDBACK_EXTENSION_NAME).as_ptr(), core_since_version: !(0), required: false },
        InfoEntry { name: (VK_EXT_VERTEX_ATTRIBUTE_DIVISOR_EXTENSION_NAME).as_ptr(), core_since_version: !(0), required: false },
        InfoEntry { name: ((VK_KHR_MAINTENANCE_1_EXTENSION_NAME).as_ptr()).cast(), core_since_version: VK_API_VERSION_1_1, required: true },
        InfoEntry { name: ((VK_KHR_MAINTENANCE_2_EXTENSION_NAME).as_ptr()).cast(), core_since_version: VK_API_VERSION_1_1, required: false },
        InfoEntry { name: (VK_KHR_SAMPLER_MIRROR_CLAMP_TO_EDGE_EXTENSION_NAME).as_ptr(), core_since_version: VK_API_VERSION_1_2, required: false },
        InfoEntry { name: (VK_KHR_SHADER_DRAW_PARAMETERS_EXTENSION_NAME).as_ptr(), core_since_version: VK_API_VERSION_1_1, required: false },
        InfoEntry { name: (VK_KHR_SWAPCHAIN_EXTENSION_NAME).as_ptr(), core_since_version: !(0), required: true },
    ];

    let map: [MapEntry; 10] = [
        MapEntry { name: (VK_EXT_EXTENDED_DYNAMIC_STATE_EXTENSION_NAME).as_ptr(), extension: (WINED3D_VK_EXT_EXTENDED_DYNAMIC_STATE) as i32 },
        MapEntry { name: (VK_EXT_EXTENDED_DYNAMIC_STATE_2_EXTENSION_NAME).as_ptr(), extension: (WINED3D_VK_EXT_EXTENDED_DYNAMIC_STATE2) as i32 },
        MapEntry { name: (VK_EXT_EXTENDED_DYNAMIC_STATE_3_EXTENSION_NAME).as_ptr(), extension: (WINED3D_VK_EXT_EXTENDED_DYNAMIC_STATE3) as i32 },
        MapEntry { name: (VK_EXT_HOST_QUERY_RESET_EXTENSION_NAME).as_ptr(), extension: (WINED3D_VK_EXT_HOST_QUERY_RESET) as i32 },
        MapEntry { name: (VK_EXT_SHADER_STENCIL_EXPORT_EXTENSION_NAME).as_ptr(), extension: (WINED3D_VK_EXT_SHADER_STENCIL_EXPORT) as i32 },
        MapEntry { name: (VK_EXT_TRANSFORM_FEEDBACK_EXTENSION_NAME).as_ptr(), extension: (WINED3D_VK_EXT_TRANSFORM_FEEDBACK) as i32 },
        MapEntry { name: (VK_EXT_VERTEX_ATTRIBUTE_DIVISOR_EXTENSION_NAME).as_ptr(), extension: (WINED3D_VK_EXT_VERTEX_ATTRIBUTE_DIVISOR) as i32 },
        MapEntry { name: ((VK_KHR_MAINTENANCE_2_EXTENSION_NAME).as_ptr()).cast(), extension: (WINED3D_VK_KHR_MAINTENANCE2) as i32 },
        MapEntry { name: (VK_KHR_SAMPLER_MIRROR_CLAMP_TO_EDGE_EXTENSION_NAME).as_ptr(), extension: (WINED3D_VK_KHR_SAMPLER_MIRROR_CLAMP_TO_EDGE) as i32 },
        MapEntry { name: (VK_KHR_SHADER_DRAW_PARAMETERS_EXTENSION_NAME).as_ptr(), extension: (WINED3D_VK_KHR_SHADER_DRAW_PARAMETERS) as i32 },
    ];

    'done: {
        vr = ((*vk_info).vk_ops.vkEnumerateDeviceExtensionProperties.unwrap_unchecked())(
            physical_device,
            core::ptr::null::<core::ffi::c_char>(),
            core::ptr::addr_of_mut!(count),
            core::ptr::null_mut::<VkExtensionProperties>(),
        );
        if (vr as core::ffi::c_int) < 0 {
            ERR!("Failed to enumerate device extensions, vr %s.\n", wined3d_debug_vkresult(vr));
            return false;
        }

        extensions = calloc(
            (count as usize) as _,
            (core::mem::size_of::<VkExtensionProperties>()) as _,
        ) as *mut VkExtensionProperties;
        if extensions.is_null() {
            ERR!("Failed to allocate extension properties array.\n");
            return false;
        }

        vr = ((*vk_info).vk_ops.vkEnumerateDeviceExtensionProperties.unwrap_unchecked())(
            physical_device,
            core::ptr::null::<core::ffi::c_char>(),
            core::ptr::addr_of_mut!(count),
            extensions,
        );
        if (vr as core::ffi::c_int) < 0 {
            ERR!("Failed to enumerate device extensions, vr %s.\n", wined3d_debug_vkresult(vr));
            break 'done;
        }

        i = 0;
        while i < count {
            i = i.wrapping_add(1);
        }

        i = 0;
        enable_count = 0;
        while (i as usize) < ARRAY_SIZE!(info) as usize {
            if !(info[i as usize].core_since_version <= (*vk_info).api_version) {
                j = 0;
                found = false;
                while j < count {
                    if strcmp(
                        core::ptr::addr_of_mut!((*extensions.offset(j as isize)).extensionName)
                            as *mut core::ffi::c_char,
                        info[i as usize].name,
                    ) == 0
                    {
                        found = true;
                        break;
                    } else {
                        j = j.wrapping_add(1);
                    }
                }

                if !found {
                    if info[i as usize].required {
                        WARN!("Required extension '%s' is not available.\n", info[i as usize].name);
                        break 'done;
                    }
                } else {
                    if wined3d_array_reserve(
                        core::ptr::addr_of_mut!(enabled_extensions) as *mut *mut core::ffi::c_void,
                        core::ptr::addr_of_mut!(enable_size),
                        (enable_count + 1) as SIZE_T,
                        (core::mem::size_of::<*const core::ffi::c_char>()) as SIZE_T,
                    ) == 0
                    {
                        ERR!("Failed to allocate enabled extensions array.\n");
                        break 'done;
                    } else {
                        let fresh15 = enable_count;
                        enable_count = enable_count.wrapping_add(1);
                        *enabled_extensions.offset(fresh15 as isize) = info[i as usize].name;
                    }
                }
            }
            i = i.wrapping_add(1);
        }

        success = true;

        i = 0;
        while (i as usize) < ARRAY_SIZE!(map) as usize {
            j = 0;
            while j < enable_count {
                if strcmp(
                    *enabled_extensions.offset(j as isize),
                    map[i as usize].name,
                ) == 0
                {
                    (*vk_info).supported[map[i as usize].extension as usize] = (TRUE as i32);
                    break;
                } else {
                    j = j.wrapping_add(1);
                }
            }
            i = i.wrapping_add(1);
        }
    }

    if success {
        (*adapter_vk).device_extension_count = enable_count;
        (*adapter_vk).device_extensions = enabled_extensions;
    } else {
        free(enabled_extensions as *mut core::ffi::c_void);
    }
    free(extensions as *mut core::ffi::c_void);
    return success;
}

unsafe extern "C" fn wined3d_adapter_vk_init(
    mut adapter_vk: *mut wined3d_adapter_vk,
    mut ordinal: u32,
    mut wined3d_creation_flags: u32,
) -> BOOL {
    let vk_info: *mut wined3d_vk_info = core::ptr::addr_of_mut!((*adapter_vk).vk_info);
    let adapter: *mut wined3d_adapter = core::ptr::addr_of_mut!((*adapter_vk).a);
    let mut id_properties: VkPhysicalDeviceIDProperties = core::mem::zeroed();
    let mut properties2: VkPhysicalDeviceProperties2 = core::mem::zeroed();
    let mut primary_luid: LUID = core::mem::zeroed();
    let mut luid: *mut LUID = core::ptr::null_mut();

    if wined3d_init_vulkan(vk_info) == 0 {
        WARN!("Failed to initialize Vulkan.\n");
        return (FALSE) as BOOL;
    }

    'fail_vulkan: {
        (*adapter_vk).physical_device = get_vulkan_physical_device(vk_info);
        if (*adapter_vk).physical_device.is_null() {
            break 'fail_vulkan;
        }

        if !wined3d_adapter_vk_init_device_extensions(adapter_vk) {
            break 'fail_vulkan;
        }

        'fail: {
            memset(
                core::ptr::addr_of_mut!(id_properties) as *mut core::ffi::c_void,
                0,
                core::mem::size_of::<VkPhysicalDeviceIDProperties>() as usize,
            );
            id_properties.sType = VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_ID_PROPERTIES;
            properties2.sType = VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROPERTIES_2;
            properties2.pNext = core::ptr::addr_of_mut!(id_properties) as *mut core::ffi::c_void;

            if (*vk_info).vk_ops.vkGetPhysicalDeviceProperties2.is_some() {
                ((*vk_info).vk_ops.vkGetPhysicalDeviceProperties2.unwrap_unchecked())(
                    (*adapter_vk).physical_device,
                    core::ptr::addr_of_mut!(properties2),
                );
            } else {
                ((*vk_info).vk_ops.vkGetPhysicalDeviceProperties.unwrap_unchecked())(
                    (*adapter_vk).physical_device,
                    core::ptr::addr_of_mut!(properties2.properties),
                );
            }
            (*adapter_vk).device_limits = properties2.properties.limits;

            ((*vk_info).vk_ops.vkGetPhysicalDeviceMemoryProperties.unwrap_unchecked())(
                (*adapter_vk).physical_device,
                core::ptr::addr_of_mut!((*adapter_vk).memory_properties),
            );

            if id_properties.deviceLUIDValid != 0 {
                luid = core::ptr::addr_of_mut!(id_properties.deviceLUID) as *mut u8 as *mut LUID;
            } else if ordinal == 0
                && wined3d_get_primary_adapter_luid(core::ptr::addr_of_mut!(primary_luid)) != 0
            {
                luid = core::ptr::addr_of_mut!(primary_luid);
            }

            if wined3d_adapter_init(adapter, ordinal, luid, core::ptr::addr_of!(wined3d_adapter_vk_ops)) == 0 {
                free((*adapter_vk).device_extensions as *mut core::ffi::c_void);
                break 'fail_vulkan;
            }

            (*adapter).vertex_pipe = wined3d_spirv_vertex_pipe_init_vk();
            (*adapter).fragment_pipe = wined3d_spirv_fragment_pipe_init_vk();
            (*adapter).misc_state_template =
                core::ptr::addr_of!(misc_state_template_vk) as *const wined3d_state_entry_template;
            (*adapter).shader_backend = wined3d_spirv_shader_backend_init_vk();

            wined3d_adapter_vk_init_d3d_info(adapter_vk, wined3d_creation_flags);

            if !adapter_vk_init_driver_info(adapter_vk, core::ptr::addr_of_mut!(properties2.properties)) {
                break 'fail;
            }

            (*adapter).vram_bytes_used = 0;

            memcpy(
                core::ptr::addr_of_mut!((*adapter).driver_uuid) as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(id_properties.driverUUID) as *mut u8 as *const core::ffi::c_void,
                core::mem::size_of::<GUID>() as usize,
            );
            memcpy(
                core::ptr::addr_of_mut!((*adapter).device_uuid) as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(id_properties.deviceUUID) as *mut u8 as *const core::ffi::c_void,
                core::mem::size_of::<GUID>() as usize,
            );

            if !(wined3d_adapter_vk_init_format_info(adapter_vk, vk_info) == 0) {
                return (TRUE) as BOOL;
            }
        }

        // fail:
        wined3d_adapter_cleanup(adapter);
        free((*adapter_vk).device_extensions as *mut core::ffi::c_void);
    }

    // fail_vulkan:
    ((*vk_info).vk_ops.vkDestroyInstance.unwrap_unchecked())(
        (*vk_info).instance,
        core::ptr::null_mut(),
    );
    wined3d_unload_vulkan(vk_info);
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_adapter_vk_create(mut ordinal: ::core::ffi::c_uint, mut wined3d_creation_flags: ::core::ffi::c_uint) -> *mut wined3d_adapter {
    let mut adapter_vk: *mut wined3d_adapter_vk = core::ptr::null_mut::<wined3d_adapter_vk>();

    adapter_vk = calloc(
        1,
        (core::mem::size_of::<wined3d_adapter_vk>()) as _,
    ) as *mut wined3d_adapter_vk;
    if adapter_vk.is_null() {
        return core::ptr::null_mut::<wined3d_adapter>();
    }

    if wined3d_adapter_vk_init(adapter_vk, ordinal, wined3d_creation_flags) == 0 {
        free(adapter_vk as *mut ::core::ffi::c_void);
        return core::ptr::null_mut::<wined3d_adapter>();
    }

    return core::ptr::addr_of_mut!((*adapter_vk).a);
}
