use crate::d3dwine::context_gl::context_gl_apply_texture_draw_state;
use crate::d3dwine::context_gl::context_gl_resource_released;
use crate::d3dwine::context::context_invalidate_compute_state;
use crate::d3dwine::context::context_invalidate_state;
use crate::d3dwine::context::context_restore;
use crate::d3dwine::device::device_invalidate_state;
use crate::d3dwine::resource::resource_cleanup;
use crate::d3dwine::resource::resource_init;
use crate::d3dwine::resource::resource_offset_map_pointer;
use crate::d3dwine::resource::resource_unload;
use crate::d3dwine::resource::vk_access_mask_from_bind_flags;
use crate::d3dwine::resource::vk_pipeline_stage_mask_from_bind_flags;
use crate::d3dwine::context_gl::wined3d_context_gl_apply_clear_state;
use crate::d3dwine::context_gl::wined3d_context_gl_apply_fbo_state_explicit;
use crate::d3dwine::context_gl::wined3d_context_gl_bind_texture;
use crate::d3dwine::context_gl::wined3d_context_gl_check_fbo_status;
use crate::d3dwine::context_gl::wined3d_context_gl_destroy_bo;
use crate::d3dwine::context_gl::wined3d_context_gl_map_bo_address;
use crate::d3dwine::context_gl::wined3d_context_gl_texture_update;
use crate::d3dwine::context_gl::wined3d_context_gl_unmap_bo_address;
use crate::d3dwine::context_vk::wined3d_context_vk_create_bo;
use crate::d3dwine::context_vk::wined3d_context_vk_create_image;
use crate::d3dwine::context_vk::wined3d_context_vk_destroy_bo;
use crate::d3dwine::context_vk::wined3d_context_vk_destroy_image;
use crate::d3dwine::context_vk::wined3d_context_vk_destroy_vk_framebuffer;
use crate::d3dwine::context_vk::wined3d_context_vk_destroy_vk_image_view;
use crate::d3dwine::context_vk::wined3d_context_vk_end_current_render_pass;
use crate::d3dwine::context_vk::wined3d_context_vk_get_command_buffer;
use crate::d3dwine::context_vk::wined3d_context_vk_get_render_pass;
use crate::d3dwine::context_vk::wined3d_context_vk_image_barrier;
use crate::d3dwine::context_vk::wined3d_context_vk_submit_command_buffer;
use crate::d3dwine::context_vk::wined3d_context_vk_wait_command_buffer;
use crate::d3dwine::cs::wined3d_cs_destroy_object;
use crate::d3dwine::cs::wined3d_cs_emit_add_dirty_texture_region;
use crate::d3dwine::cs::wined3d_cs_emit_set_color_key;
use crate::d3dwine::cs::wined3d_cs_emit_unload_resource;
use crate::d3dwine::cs::wined3d_cs_init_object;
use crate::d3dwine::cs::wined3d_device_context_emit_blt_sub_resource;
use crate::d3dwine::cs::wined3d_device_context_emit_update_sub_resource;
use crate::d3dwine::device::wined3d_device_gl_create_bo;
use crate::d3dwine::resource::wined3d_resource_check_box_dimensions;
use crate::d3dwine::resource::wined3d_resource_free_sysmem;
use crate::d3dwine::resource::wined3d_resource_get_decompress_format;
use crate::d3dwine::resource::wined3d_resource_get_sample_count;
use crate::d3dwine::resource::wined3d_resource_is_offscreen;
use crate::d3dwine::resource::wined3d_resource_memory_colour_fill;
use crate::d3dwine::resource::wined3d_resource_prepare_sysmem;
use crate::d3dwine::resource::wined3d_resource_update_draw_binding;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut WINED3D_READ_ONLY_BIND_MASK: uint32_t = (WINED3D_BIND_VERTEX_BUFFER
    | WINED3D_BIND_INDEX_BUFFER
    | WINED3D_BIND_CONSTANT_BUFFER
    | WINED3D_BIND_SHADER_RESOURCE
    | WINED3D_BIND_INDIRECT_BUFFER) as uint32_t;

static mut WINED3D_READ_ONLY_ACCESS_FLAGS: VkAccessFlags =
    (VK_ACCESS_INDIRECT_COMMAND_READ_BIT as ::core::ffi::c_int
        | VK_ACCESS_INDEX_READ_BIT as ::core::ffi::c_int
        | VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT as ::core::ffi::c_int
        | VK_ACCESS_UNIFORM_READ_BIT as ::core::ffi::c_int
        | VK_ACCESS_INPUT_ATTACHMENT_READ_BIT as ::core::ffi::c_int
        | VK_ACCESS_SHADER_READ_BIT as ::core::ffi::c_int
        | VK_ACCESS_COLOR_ATTACHMENT_READ_BIT as ::core::ffi::c_int
        | VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_READ_BIT as ::core::ffi::c_int
        | VK_ACCESS_TRANSFER_READ_BIT as ::core::ffi::c_int
        | VK_ACCESS_HOST_READ_BIT as ::core::ffi::c_int
        | VK_ACCESS_MEMORY_READ_BIT as ::core::ffi::c_int) as VkAccessFlags;

static mut wined3d_texture_sysmem_locations: uint32_t =
    (WINED3D_LOCATION_SYSMEM | WINED3D_LOCATION_BUFFER) as uint32_t;

static mut formats_src_alpha_fixup: [d3dfmt_alpha_fixup; 3] = [
    d3dfmt_alpha_fixup {
        format_id: WINED3DFMT_B8G8R8X8_UNORM,
        conv_format_id: WINED3DFMT_B8G8R8A8_UNORM,
    },
    d3dfmt_alpha_fixup {
        format_id: WINED3DFMT_B5G5R5X1_UNORM,
        conv_format_id: WINED3DFMT_B5G5R5A1_UNORM,
    },
    d3dfmt_alpha_fixup {
        format_id: WINED3DFMT_B4G4R4X4_UNORM,
        conv_format_id: WINED3DFMT_B4G4R4A4_UNORM,
    },
];

static mut texture_gl_ops: wined3d_texture_ops = unsafe {
    wined3d_texture_ops {
        texture_prepare_location: Some(
            wined3d_texture_gl_prepare_location
                as unsafe extern "C" fn(
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    *mut wined3d_context,
                    ::core::ffi::c_uint,
                ) -> BOOL,
        ),
        texture_load_location: Some(
            wined3d_texture_gl_load_location
                as unsafe extern "C" fn(
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    *mut wined3d_context,
                    uint32_t,
                ) -> BOOL,
        ),
        texture_unload_location: Some(
            wined3d_texture_gl_unload_location
                as unsafe extern "C" fn(
                    *mut wined3d_texture,
                    *mut wined3d_context,
                    ::core::ffi::c_uint,
                ) -> (),
        ),
        texture_upload_data: Some(
            wined3d_texture_gl_upload_data
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *const wined3d_const_bo_address,
                    *const wined3d_format,
                    *const wined3d_box,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                ) -> (),
        ),
        texture_download_data: Some(
            wined3d_texture_gl_download_data
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    *const wined3d_box,
                    *const wined3d_bo_address,
                    *const wined3d_format,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                ) -> (),
        ),
    }
};

static mut texture_resource_ops: wined3d_resource_ops = unsafe {
    wined3d_resource_ops {
        resource_incref: Some(
            texture_resource_incref as unsafe extern "C" fn(*mut wined3d_resource) -> ULONG,
        ),
        resource_decref: Some(
            texture_resource_decref as unsafe extern "C" fn(*mut wined3d_resource) -> ULONG,
        ),
        resource_preload: Some(
            texture_resource_preload as unsafe extern "C" fn(*mut wined3d_resource) -> (),
        ),
        resource_unload: Some(
            texture_resource_unload as unsafe extern "C" fn(*mut wined3d_resource) -> (),
        ),
        resource_sub_resource_get_desc: Some(
            texture_resource_sub_resource_get_desc
                as unsafe extern "C" fn(
                    *mut wined3d_resource,
                    ::core::ffi::c_uint,
                    *mut wined3d_sub_resource_desc,
                ) -> HRESULT,
        ),
        resource_sub_resource_get_map_pitch: Some(
            texture_resource_sub_resource_get_map_pitch
                as unsafe extern "C" fn(
                    *mut wined3d_resource,
                    ::core::ffi::c_uint,
                    *mut ::core::ffi::c_uint,
                    *mut ::core::ffi::c_uint,
                ) -> (),
        ),
        resource_sub_resource_map: Some(
            texture_resource_sub_resource_map
                as unsafe extern "C" fn(
                    *mut wined3d_resource,
                    ::core::ffi::c_uint,
                    *mut *mut ::core::ffi::c_void,
                    *const wined3d_box,
                    uint32_t,
                ) -> HRESULT,
        ),
        resource_sub_resource_unmap: Some(
            texture_resource_sub_resource_unmap
                as unsafe extern "C" fn(*mut wined3d_resource, ::core::ffi::c_uint) -> HRESULT,
        ),
    }
};

static mut wined3d_texture_no3d_ops: wined3d_texture_ops = unsafe {
    wined3d_texture_ops {
        texture_prepare_location: Some(
            wined3d_texture_no3d_prepare_location
                as unsafe extern "C" fn(
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    *mut wined3d_context,
                    ::core::ffi::c_uint,
                ) -> BOOL,
        ),
        texture_load_location: Some(
            wined3d_texture_no3d_load_location
                as unsafe extern "C" fn(
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    *mut wined3d_context,
                    uint32_t,
                ) -> BOOL,
        ),
        texture_unload_location: Some(
            wined3d_texture_no3d_unload_location
                as unsafe extern "C" fn(
                    *mut wined3d_texture,
                    *mut wined3d_context,
                    ::core::ffi::c_uint,
                ) -> (),
        ),
        texture_upload_data: Some(
            wined3d_texture_no3d_upload_data
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *const wined3d_const_bo_address,
                    *const wined3d_format,
                    *const wined3d_box,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                ) -> (),
        ),
        texture_download_data: Some(
            wined3d_texture_no3d_download_data
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    *const wined3d_box,
                    *const wined3d_bo_address,
                    *const wined3d_format,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                ) -> (),
        ),
    }
};

static mut wined3d_texture_vk_ops: wined3d_texture_ops = unsafe {
    wined3d_texture_ops {
        texture_prepare_location: Some(
            wined3d_texture_vk_prepare_location
                as unsafe extern "C" fn(
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    *mut wined3d_context,
                    ::core::ffi::c_uint,
                ) -> BOOL,
        ),
        texture_load_location: Some(
            wined3d_texture_vk_load_location
                as unsafe extern "C" fn(
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    *mut wined3d_context,
                    uint32_t,
                ) -> BOOL,
        ),
        texture_unload_location: Some(
            wined3d_texture_vk_unload_location
                as unsafe extern "C" fn(
                    *mut wined3d_texture,
                    *mut wined3d_context,
                    ::core::ffi::c_uint,
                ) -> (),
        ),
        texture_upload_data: Some(
            wined3d_texture_vk_upload_data
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *const wined3d_const_bo_address,
                    *const wined3d_format,
                    *const wined3d_box,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                ) -> (),
        ),
        texture_download_data: Some(
            wined3d_texture_vk_download_data
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    *const wined3d_box,
                    *const wined3d_bo_address,
                    *const wined3d_format,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                ) -> (),
        ),
    }
};

static mut ffp_blitter_ops: wined3d_blitter_ops = unsafe {
    wined3d_blitter_ops {
        blitter_destroy: Some(
            ffp_blitter_destroy
                as unsafe extern "C" fn(*mut wined3d_blitter, *mut wined3d_context) -> (),
        ),
        blitter_clear: Some(
            ffp_blitter_clear
                as unsafe extern "C" fn(
                    *mut wined3d_blitter,
                    *mut wined3d_device,
                    ::core::ffi::c_uint,
                    *const wined3d_fb_state,
                    ::core::ffi::c_uint,
                    *const RECT,
                    *const RECT,
                    uint32_t,
                    *const wined3d_color,
                    ::core::ffi::c_float,
                    ::core::ffi::c_uint,
                ) -> (),
        ),
        blitter_blit: Some(
            ffp_blitter_blit
                as unsafe extern "C" fn(
                    *mut wined3d_blitter,
                    wined3d_blit_op,
                    *mut wined3d_context,
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    DWORD,
                    *const RECT,
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    DWORD,
                    *const RECT,
                    *const wined3d_color_key,
                    wined3d_texture_filter_type,
                    *const wined3d_format,
                ) -> DWORD,
        ),
    }
};

static mut fbo_blitter_ops: wined3d_blitter_ops = unsafe {
    wined3d_blitter_ops {
        blitter_destroy: Some(
            fbo_blitter_destroy
                as unsafe extern "C" fn(*mut wined3d_blitter, *mut wined3d_context) -> (),
        ),
        blitter_clear: Some(
            fbo_blitter_clear
                as unsafe extern "C" fn(
                    *mut wined3d_blitter,
                    *mut wined3d_device,
                    ::core::ffi::c_uint,
                    *const wined3d_fb_state,
                    ::core::ffi::c_uint,
                    *const RECT,
                    *const RECT,
                    uint32_t,
                    *const wined3d_color,
                    ::core::ffi::c_float,
                    ::core::ffi::c_uint,
                ) -> (),
        ),
        blitter_blit: Some(
            fbo_blitter_blit
                as unsafe extern "C" fn(
                    *mut wined3d_blitter,
                    wined3d_blit_op,
                    *mut wined3d_context,
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    DWORD,
                    *const RECT,
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    DWORD,
                    *const RECT,
                    *const wined3d_color_key,
                    wined3d_texture_filter_type,
                    *const wined3d_format,
                ) -> DWORD,
        ),
    }
};

static mut raw_blitter_ops: wined3d_blitter_ops = unsafe {
    wined3d_blitter_ops {
        blitter_destroy: Some(
            raw_blitter_destroy
                as unsafe extern "C" fn(*mut wined3d_blitter, *mut wined3d_context) -> (),
        ),
        blitter_clear: Some(
            raw_blitter_clear
                as unsafe extern "C" fn(
                    *mut wined3d_blitter,
                    *mut wined3d_device,
                    ::core::ffi::c_uint,
                    *const wined3d_fb_state,
                    ::core::ffi::c_uint,
                    *const RECT,
                    *const RECT,
                    uint32_t,
                    *const wined3d_color,
                    ::core::ffi::c_float,
                    ::core::ffi::c_uint,
                ) -> (),
        ),
        blitter_blit: Some(
            raw_blitter_blit
                as unsafe extern "C" fn(
                    *mut wined3d_blitter,
                    wined3d_blit_op,
                    *mut wined3d_context,
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    DWORD,
                    *const RECT,
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    DWORD,
                    *const RECT,
                    *const wined3d_color_key,
                    wined3d_texture_filter_type,
                    *const wined3d_format,
                ) -> DWORD,
        ),
    }
};

static mut vk_blitter_ops: wined3d_blitter_ops = unsafe {
    wined3d_blitter_ops {
        blitter_destroy: Some(
            vk_blitter_destroy
                as unsafe extern "C" fn(*mut wined3d_blitter, *mut wined3d_context) -> (),
        ),
        blitter_clear: Some(
            vk_blitter_clear
                as unsafe extern "C" fn(
                    *mut wined3d_blitter,
                    *mut wined3d_device,
                    ::core::ffi::c_uint,
                    *const wined3d_fb_state,
                    ::core::ffi::c_uint,
                    *const RECT,
                    *const RECT,
                    uint32_t,
                    *const wined3d_color,
                    ::core::ffi::c_float,
                    ::core::ffi::c_uint,
                ) -> (),
        ),
        blitter_blit: Some(
            vk_blitter_blit
                as unsafe extern "C" fn(
                    *mut wined3d_blitter,
                    wined3d_blit_op,
                    *mut wined3d_context,
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    DWORD,
                    *const RECT,
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    DWORD,
                    *const RECT,
                    *const wined3d_color_key,
                    wined3d_texture_filter_type,
                    *const wined3d_format,
                ) -> DWORD,
        ),
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(d3d);
// WINE_DECLARE_DEBUG_CHANNEL(d3d_perf);
// WINE_DECLARE_DEBUG_CHANNEL(winediag);

pub const WINED3D_TEXTURE_DYNAMIC_MAP_THRESHOLD: u32 = 50;

#[repr(C)]
pub struct wined3d_texture_idx {
    pub texture: *mut wined3d_texture,
    pub sub_resource_idx: u32,
}
pub type Pwined3d_texture_idx = *mut wined3d_texture_idx;

#[repr(C)]
pub struct wined3d_rect_f {
    pub l: f32,
    pub t: f32,
    pub r: f32,
    pub b: f32,
}
pub type Pwined3d_rect_f = *mut wined3d_rect_f;

#[repr(C)]
pub struct d3dfmt_alpha_fixup {
    pub format_id: wined3d_format_id,
    pub conv_format_id: wined3d_format_id,
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_validate_sub_resource_idx(texture: *const wined3d_texture, sub_resource_idx: ::core::ffi::c_uint) -> bool {
    if sub_resource_idx < ((*texture).level_count as UINT).wrapping_mul((*texture).layer_count) {
        return true;
    }

    WARN!("Invalid sub-resource index %u.\n", sub_resource_idx);
    return false;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_can_use_pbo(
    texture: *const wined3d_texture,
    d3d_info: *const wined3d_d3d_info,
) -> BOOL {
    if (*d3d_info).pbo() == 0
        || (*(*texture).resource.format).conv_byte_count != 0
        || (*texture).resource.pin_sysmem() as ::core::ffi::c_int != 0
    {
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn wined3d_texture_use_pbo(texture: *const wined3d_texture, d3d_info: *const wined3d_d3d_info) -> BOOL {
    if wined3d_texture_can_use_pbo(texture, d3d_info) == 0 {
        return (FALSE) as BOOL;
    }

    /* Use a PBO for dynamic textures and read-only staging textures. */
    return ((((*texture).resource.access & WINED3D_RESOURCE_ACCESS_CPU) == 0)
        && ((*texture).resource.usage & WINED3DUSAGE_DYNAMIC) != 0
        || (*texture).resource.access == (WINED3D_RESOURCE_ACCESS_CPU | WINED3D_RESOURCE_ACCESS_MAP_R)) as BOOL;
}

unsafe extern "C" fn wined3d_texture_use_immutable_storage(
    texture: *const wined3d_texture,
    gl_info: *const wined3d_gl_info,
) -> BOOL {
    return ((*gl_info).supported[ARB_TEXTURE_STORAGE as usize] != 0
        && ((*texture).resource.format_attrs & WINED3D_FORMAT_ATTR_HEIGHT_SCALE) == 0) as i32;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_translate_drawable_coords(texture: *const wined3d_texture, window: HWND, rect: *mut RECT) {
    let drawable_height: u32;
    let mut offset: POINT = POINT { x: 0, y: 0 };
    let mut windowsize: RECT = core::mem::zeroed();

    if (*texture).swapchain.is_null() {
        return;
    }

    if texture == (*(*texture).swapchain).front_buffer as *const wined3d_texture {
        ScreenToClient(window, core::ptr::addr_of_mut!(offset));
        OffsetRect(rect, offset.x, offset.y);
    }

    GetClientRect(window, core::ptr::addr_of_mut!(windowsize));
    drawable_height = (windowsize.bottom - windowsize.top) as u32;

    (*rect).top = drawable_height.wrapping_sub((*rect).top as u32) as LONG;
    (*rect).bottom = drawable_height.wrapping_sub((*rect).bottom as u32) as LONG;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_get_gl_buffer(texture: *const wined3d_texture) -> GLenum {
    let swapchain: *const wined3d_swapchain = (*texture).swapchain;

    if swapchain.is_null() {
        ERR!("Texture %p is not part of a swapchain.\n", texture);
        return GL_NONE;
    }

    if texture == (*swapchain).front_buffer as *const wined3d_texture {
        TRACE!("Returning GL_FRONT.\n");
        return GL_FRONT;
    }

    if texture == *(*swapchain).back_buffers.offset(0 as isize) as *const wined3d_texture {
        TRACE!("Returning GL_BACK.\n");
        return GL_BACK;
    }

    FIXME!("Higher back buffer, returning GL_BACK.\n");
    return GL_BACK;
}

unsafe extern "C" fn wined3d_resource_access_from_location(mut location: uint32_t) -> uint32_t {
    match location {
        WINED3D_LOCATION_DISCARDED => return 0,
        WINED3D_LOCATION_SYSMEM => return WINED3D_RESOURCE_ACCESS_CPU,
        WINED3D_LOCATION_BUFFER
        | WINED3D_LOCATION_DRAWABLE
        | WINED3D_LOCATION_TEXTURE_RGB
        | WINED3D_LOCATION_TEXTURE_SRGB
        | WINED3D_LOCATION_RB_MULTISAMPLE
        | WINED3D_LOCATION_RB_RESOLVED => return WINED3D_RESOURCE_ACCESS_GPU,
        _ => {
            FIXME!("Unhandled location %#x.\n", location);
            return 0;
        }
    }
}

unsafe extern "C" fn cube_coords_float(r: *const RECT, w: UINT, h: UINT, f: *mut wined3d_rect_f) {
    (*f).l = (((*r).left as f32 * 2.0f32) / w as f32) - 1.0f32;
    (*f).t = (((*r).top as f32 * 2.0f32) / h as f32) - 1.0f32;
    (*f).r = (((*r).right as f32 * 2.0f32) / w as f32) - 1.0f32;
    (*f).b = (((*r).bottom as f32 * 2.0f32) / h as f32) - 1.0f32;
}
#[no_mangle]
pub unsafe extern "C" fn texture2d_get_blt_info(
    texture_gl: *const wined3d_texture_gl,
    sub_resource_idx: ::core::ffi::c_uint,
    rect: *const RECT,
    info: *mut wined3d_blt_info,
) {
    let coords: *mut wined3d_vec3 = core::ptr::addr_of_mut!((*info).texcoords) as *mut wined3d_vec3;
    let mut f: wined3d_rect_f = wined3d_rect_f {
        l: 0.0,
        t: 0.0,
        r: 0.0,
        b: 0.0,
    };
    let mut level: ::core::ffi::c_uint;
    let mut target: GLenum;
    let mut w: GLsizei;
    let mut h: GLsizei;

    level = sub_resource_idx % (*texture_gl).t.level_count;
    w = wined3d_texture_get_level_width(core::ptr::addr_of!((*texture_gl).t), level) as GLsizei;
    h = wined3d_texture_get_level_height(core::ptr::addr_of!((*texture_gl).t), level) as GLsizei;
    target = wined3d_texture_gl_get_sub_resource_target(texture_gl, sub_resource_idx);

    let go_2d: bool = match target {
        GL_TEXTURE_CUBE_MAP_POSITIVE_X => {
            (*info).bind_target = GL_TEXTURE_CUBE_MAP_ARB as GLenum;
            cube_coords_float(rect, w as UINT, h as UINT, core::ptr::addr_of_mut!(f));

            (*coords.offset(0 as isize)).x = 1.0f32;
            (*coords.offset(0 as isize)).y = -f.t;
            (*coords.offset(0 as isize)).z = -f.l;
            (*coords.offset(1 as isize)).x = 1.0f32;
            (*coords.offset(1 as isize)).y = -f.t;
            (*coords.offset(1 as isize)).z = -f.r;
            (*coords.offset(2 as isize)).x = 1.0f32;
            (*coords.offset(2 as isize)).y = -f.b;
            (*coords.offset(2 as isize)).z = -f.l;
            (*coords.offset(3 as isize)).x = 1.0f32;
            (*coords.offset(3 as isize)).y = -f.b;
            (*coords.offset(3 as isize)).z = -f.r;
            false
        }
        GL_TEXTURE_CUBE_MAP_NEGATIVE_X => {
            (*info).bind_target = GL_TEXTURE_CUBE_MAP_ARB as GLenum;
            cube_coords_float(rect, w as UINT, h as UINT, core::ptr::addr_of_mut!(f));

            (*coords.offset(0 as isize)).x = -1.0f32;
            (*coords.offset(0 as isize)).y = -f.t;
            (*coords.offset(0 as isize)).z = f.l;
            (*coords.offset(1 as isize)).x = -1.0f32;
            (*coords.offset(1 as isize)).y = -f.t;
            (*coords.offset(1 as isize)).z = f.r;
            (*coords.offset(2 as isize)).x = -1.0f32;
            (*coords.offset(2 as isize)).y = -f.b;
            (*coords.offset(2 as isize)).z = f.l;
            (*coords.offset(3 as isize)).x = -1.0f32;
            (*coords.offset(3 as isize)).y = -f.b;
            (*coords.offset(3 as isize)).z = f.r;
            false
        }
        GL_TEXTURE_CUBE_MAP_POSITIVE_Y => {
            (*info).bind_target = GL_TEXTURE_CUBE_MAP_ARB as GLenum;
            cube_coords_float(rect, w as UINT, h as UINT, core::ptr::addr_of_mut!(f));

            (*coords.offset(0 as isize)).x = f.l;
            (*coords.offset(0 as isize)).y = 1.0f32;
            (*coords.offset(0 as isize)).z = f.t;
            (*coords.offset(1 as isize)).x = f.r;
            (*coords.offset(1 as isize)).y = 1.0f32;
            (*coords.offset(1 as isize)).z = f.t;
            (*coords.offset(2 as isize)).x = f.l;
            (*coords.offset(2 as isize)).y = 1.0f32;
            (*coords.offset(2 as isize)).z = f.b;
            (*coords.offset(3 as isize)).x = f.r;
            (*coords.offset(3 as isize)).y = 1.0f32;
            (*coords.offset(3 as isize)).z = f.b;
            false
        }
        GL_TEXTURE_CUBE_MAP_NEGATIVE_Y => {
            (*info).bind_target = GL_TEXTURE_CUBE_MAP_ARB as GLenum;
            cube_coords_float(rect, w as UINT, h as UINT, core::ptr::addr_of_mut!(f));

            (*coords.offset(0 as isize)).x = f.l;
            (*coords.offset(0 as isize)).y = -1.0f32;
            (*coords.offset(0 as isize)).z = -f.t;
            (*coords.offset(1 as isize)).x = f.r;
            (*coords.offset(1 as isize)).y = -1.0f32;
            (*coords.offset(1 as isize)).z = -f.t;
            (*coords.offset(2 as isize)).x = f.l;
            (*coords.offset(2 as isize)).y = -1.0f32;
            (*coords.offset(2 as isize)).z = -f.b;
            (*coords.offset(3 as isize)).x = f.r;
            (*coords.offset(3 as isize)).y = -1.0f32;
            (*coords.offset(3 as isize)).z = -f.b;
            false
        }
        GL_TEXTURE_CUBE_MAP_POSITIVE_Z => {
            (*info).bind_target = GL_TEXTURE_CUBE_MAP_ARB as GLenum;
            cube_coords_float(rect, w as UINT, h as UINT, core::ptr::addr_of_mut!(f));

            (*coords.offset(0 as isize)).x = f.l;
            (*coords.offset(0 as isize)).y = -f.t;
            (*coords.offset(0 as isize)).z = 1.0f32;
            (*coords.offset(1 as isize)).x = f.r;
            (*coords.offset(1 as isize)).y = -f.t;
            (*coords.offset(1 as isize)).z = 1.0f32;
            (*coords.offset(2 as isize)).x = f.l;
            (*coords.offset(2 as isize)).y = -f.b;
            (*coords.offset(2 as isize)).z = 1.0f32;
            (*coords.offset(3 as isize)).x = f.r;
            (*coords.offset(3 as isize)).y = -f.b;
            (*coords.offset(3 as isize)).z = 1.0f32;
            false
        }
        GL_TEXTURE_CUBE_MAP_NEGATIVE_Z => {
            (*info).bind_target = GL_TEXTURE_CUBE_MAP_ARB as GLenum;
            cube_coords_float(rect, w as UINT, h as UINT, core::ptr::addr_of_mut!(f));

            (*coords.offset(0 as isize)).x = -f.l;
            (*coords.offset(0 as isize)).y = -f.t;
            (*coords.offset(0 as isize)).z = -1.0f32;
            (*coords.offset(1 as isize)).x = -f.r;
            (*coords.offset(1 as isize)).y = -f.t;
            (*coords.offset(1 as isize)).z = -1.0f32;
            (*coords.offset(2 as isize)).x = -f.l;
            (*coords.offset(2 as isize)).y = -f.b;
            (*coords.offset(2 as isize)).z = -1.0f32;
            (*coords.offset(3 as isize)).x = -f.r;
            (*coords.offset(3 as isize)).y = -f.b;
            (*coords.offset(3 as isize)).z = -1.0f32;
            false
        }
        GL_TEXTURE_2D => true,
        _ => {
            FIXME!("Unsupported texture target %#x.\n", target);
            true
        }
    };

    if go_2d {
        (*info).bind_target = GL_TEXTURE_2D as GLenum;
        (*coords.offset(0 as isize)).x =
            (*rect).left as ::core::ffi::c_float / w as ::core::ffi::c_float;
        (*coords.offset(0 as isize)).y =
            (*rect).top as ::core::ffi::c_float / h as ::core::ffi::c_float;
        (*coords.offset(0 as isize)).z = 0.0f32;

        (*coords.offset(1 as isize)).x =
            (*rect).right as ::core::ffi::c_float / w as ::core::ffi::c_float;
        (*coords.offset(1 as isize)).y =
            (*rect).top as ::core::ffi::c_float / h as ::core::ffi::c_float;
        (*coords.offset(1 as isize)).z = 0.0f32;

        (*coords.offset(2 as isize)).x =
            (*rect).left as ::core::ffi::c_float / w as ::core::ffi::c_float;
        (*coords.offset(2 as isize)).y =
            (*rect).bottom as ::core::ffi::c_float / h as ::core::ffi::c_float;
        (*coords.offset(2 as isize)).z = 0.0f32;

        (*coords.offset(3 as isize)).x =
            (*rect).right as ::core::ffi::c_float / w as ::core::ffi::c_float;
        (*coords.offset(3 as isize)).y =
            (*rect).bottom as ::core::ffi::c_float / h as ::core::ffi::c_float;
        (*coords.offset(3 as isize)).z = 0.0f32;
    }
}

unsafe extern "C" fn fbo_blitter_supported(
    mut blit_op: wined3d_blit_op,
    mut gl_info: *const wined3d_gl_info,
    mut src_resource: *const wined3d_resource,
    mut src_location: DWORD,
    mut dst_resource: *const wined3d_resource,
    mut dst_location: DWORD,
) -> bool {
    let mut src_format: *const wined3d_format = (*src_resource).format;
    let mut dst_format: *const wined3d_format = (*dst_resource).format;
    let mut src_ds: bool = false;
    let mut dst_ds: bool = false;
    if ((*src_resource).format_attrs | (*dst_resource).format_attrs) & WINED3D_FORMAT_ATTR_HEIGHT_SCALE != 0 {
        return false;
    }
    if (*src_resource).access & (*dst_resource).access & WINED3D_RESOURCE_ACCESS_GPU == 0 {
        return false;
    }
    if (*src_resource).type_ as ::core::ffi::c_uint != WINED3D_RTYPE_TEXTURE_2D as ::core::ffi::c_uint {
        return false;
    }
    src_ds = (*src_format).depth_size as ::core::ffi::c_int != 0
        || (*src_format).stencil_size as ::core::ffi::c_int != 0;
    dst_ds = (*dst_format).depth_size as ::core::ffi::c_int != 0
        || (*dst_format).stencil_size as ::core::ffi::c_int != 0;
    if src_ds as ::core::ffi::c_int != dst_ds as ::core::ffi::c_int {
        return false;
    }
    match blit_op as ::core::ffi::c_uint {
        WINED3D_BLIT_OP_COLOR_BLIT => {
            if !((*src_format).caps[WINED3D_GL_RES_TYPE_TEX_2D as usize] & WINED3D_FORMAT_CAP_FBO_ATTACHABLE as ::core::ffi::c_uint != 0
                || (*src_resource).bind_flags & WINED3D_BIND_RENDER_TARGET as ::core::ffi::c_uint != 0)
            {
                return false;
            }
            if !((*dst_format).caps[WINED3D_GL_RES_TYPE_TEX_2D as usize] & WINED3D_FORMAT_CAP_FBO_ATTACHABLE as ::core::ffi::c_uint != 0
                || (*dst_resource).bind_flags & WINED3D_BIND_RENDER_TARGET as ::core::ffi::c_uint != 0)
            {
                return false;
            }
            if ((*src_format).id as ::core::ffi::c_uint != (*dst_format).id as ::core::ffi::c_uint
                || dst_location == WINED3D_LOCATION_DRAWABLE as DWORD)
                && (is_identity_fixup((*src_format).color_fixup) == 0
                    || is_identity_fixup((*dst_format).color_fixup) == 0)
            {
                return false;
            }
        }
        WINED3D_BLIT_OP_DEPTH_BLIT => {
            if (*src_format).caps[WINED3D_GL_RES_TYPE_TEX_2D as usize] & WINED3D_FORMAT_CAP_DEPTH_STENCIL as ::core::ffi::c_uint == 0 {
                return false;
            }
            if (*dst_format).caps[WINED3D_GL_RES_TYPE_TEX_2D as usize] & WINED3D_FORMAT_CAP_DEPTH_STENCIL as ::core::ffi::c_uint == 0 {
                return false;
            }
            if is_complex_fixup((*src_format).color_fixup) != 0
                || is_complex_fixup((*dst_format).color_fixup) != 0
                || is_scaling_fixup((*src_format).color_fixup) != 0
                || is_scaling_fixup((*dst_format).color_fixup) != 0
            {
                return false;
            }
        }
        _ => return false,
    }
    return true;
}

unsafe extern "C" fn texture2d_blt_fbo(
    mut device: *mut wined3d_device,
    mut context: *mut wined3d_context,
    mut filter: wined3d_texture_filter_type,
    mut src_texture: *mut wined3d_texture,
    mut src_sub_resource_idx: ::core::ffi::c_uint,
    mut src_location: DWORD,
    mut src_rect: *const RECT,
    mut dst_texture: *mut wined3d_texture,
    mut dst_sub_resource_idx: ::core::ffi::c_uint,
    mut dst_location: DWORD,
    mut dst_rect: *const RECT,
    mut resolve_format: *const wined3d_format,
) {
    let mut required_texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut restore_texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut dst_save_texture: *mut wined3d_texture = dst_texture;
    let mut restore_idx: ::core::ffi::c_uint = 0;
    let mut dst_save_sub_resource_idx: ::core::ffi::c_uint = dst_sub_resource_idx;
    let mut resolve: bool = false;
    let mut scaled_resolve: bool = false;
    let mut restore_context: bool = false;
    let mut src_staging_texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut gl_info: *const wined3d_gl_info = core::ptr::null();
    let mut context_gl: *mut wined3d_context_gl = core::ptr::null_mut();
    let mut gl_filter: GLenum = 0;
    let mut buffer: GLenum = 0;
    let mut s: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut d: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    TRACE!(
        "device %p, context %p, filter %s, src_texture %p, src_sub_resource_idx %u, src_location %s, src_rect %s, dst_texture %p, dst_sub_resource_idx %u, dst_location %s, dst_rect %s, resolve format %p.\n",
        device,
        context,
        debug_d3dtexturefiltertype(filter),
        src_texture,
        src_sub_resource_idx,
        wined3d_debug_location(src_location as uint32_t),
        wine_dbgstr_rect(src_rect),
        dst_texture,
        dst_sub_resource_idx,
        wined3d_debug_location(dst_location as uint32_t),
        wine_dbgstr_rect(dst_rect),
        resolve_format
    );

    resolve = wined3d_texture_gl_is_multisample_location(wined3d_texture_gl(src_texture), src_location as uint32_t);
    scaled_resolve = resolve
        && (abs((*src_rect).bottom - (*src_rect).top)
            != abs((*dst_rect).bottom - (*dst_rect).top)
            || abs((*src_rect).right - (*src_rect).left)
                != abs((*dst_rect).right - (*dst_rect).left));

    if filter == WINED3D_TEXF_LINEAR {
        gl_filter = if scaled_resolve {
            GL_SCALED_RESOLVE_NICEST_EXT
        } else {
            GL_LINEAR
        } as GLenum;
    } else {
        gl_filter = if scaled_resolve {
            GL_SCALED_RESOLVE_FASTEST_EXT
        } else {
            GL_NEAREST
        } as GLenum;
    }

    'done: {
        if resolve {
            let mut resolve_internal: GLint = 0;
            let mut src_internal: GLint = 0;
            let mut dst_internal: GLint = 0;
            let mut resolve_format_id: wined3d_format_id = WINED3DFMT_UNKNOWN;

            src_internal = wined3d_gl_get_internal_format(
                core::ptr::addr_of_mut!((*src_texture).resource),
                wined3d_format_gl((*src_texture).resource.format),
                src_location == WINED3D_LOCATION_TEXTURE_SRGB,
            ) as GLint;
            dst_internal = wined3d_gl_get_internal_format(
                core::ptr::addr_of_mut!((*dst_texture).resource),
                wined3d_format_gl((*dst_texture).resource.format),
                dst_location == WINED3D_LOCATION_TEXTURE_SRGB,
            ) as GLint;

            if !resolve_format.is_null() {
                resolve_internal = (*wined3d_format_gl(resolve_format)).internal;
                resolve_format_id = (*resolve_format).id;
            } else if wined3d_format_is_typeless((*src_texture).resource.format) == 0 {
                resolve_internal = src_internal;
                resolve_format_id = (*(*src_texture).resource.format).id;
            } else {
                resolve_internal = dst_internal;
                resolve_format_id = (*(*dst_texture).resource.format).id;
            }

            /* In case of typeless resolve the texture type may not match the resolve type.
             * To handle that, allocate intermediate texture(s) to resolve from/to.
             * A possible performance improvement would be to resolve using a shader instead. */
            if src_internal != resolve_internal {
                let mut desc: wined3d_resource_desc = wined3d_resource_desc {
                    resource_type: WINED3D_RTYPE_NONE,
                    format: WINED3DFMT_UNKNOWN,
                    multisample_type: WINED3D_MULTISAMPLE_NONE,
                    multisample_quality: 0,
                    usage: 0,
                    bind_flags: 0,
                    access: 0,
                    width: 0,
                    height: 0,
                    depth: 0,
                    size: 0,
                };
                let mut src_level: ::core::ffi::c_uint = 0;
                let mut hr: HRESULT = 0;

                src_level = src_sub_resource_idx % (*src_texture).level_count;
                desc.resource_type = WINED3D_RTYPE_TEXTURE_2D;
                desc.format = resolve_format_id;
                desc.multisample_type = (*src_texture).resource.multisample_type;
                desc.multisample_quality = (*src_texture).resource.multisample_quality;
                desc.usage = WINED3DUSAGE_CS;
                desc.bind_flags = 0;
                desc.access = WINED3D_RESOURCE_ACCESS_GPU;
                desc.width = wined3d_texture_get_level_width(src_texture, src_level);
                desc.height = wined3d_texture_get_level_height(src_texture, src_level);
                desc.depth = 1;
                desc.size = 0;

                hr = wined3d_texture_create(
                    device,
                    core::ptr::addr_of_mut!(desc),
                    1,
                    1,
                    0,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                    core::ptr::addr_of!(wined3d_null_parent_ops),
                    core::ptr::addr_of_mut!(src_staging_texture),
                );
                if hr < 0 {
                    ERR!("Failed to create staging texture, hr %#lx.\n", hr);
                    break 'done;
                }

                if src_location == WINED3D_LOCATION_DRAWABLE {
                    FIXME!("WINED3D_LOCATION_DRAWABLE not supported for the source of a typeless resolve.\n");
                }

                (*(*(*device).blitter).ops).blitter_blit.unwrap_unchecked()(
                    (*device).blitter,
                    WINED3D_BLIT_OP_RAW_BLIT,
                    context,
                    src_texture,
                    src_sub_resource_idx,
                    src_location,
                    src_rect,
                    src_staging_texture,
                    0,
                    src_location,
                    src_rect,
                    core::ptr::null(),
                    WINED3D_TEXF_NONE,
                    core::ptr::null(),
                );

                src_texture = src_staging_texture;
                src_sub_resource_idx = 0;
            }

            if dst_internal != resolve_internal {
                let mut desc: wined3d_resource_desc = wined3d_resource_desc {
                    resource_type: WINED3D_RTYPE_NONE,
                    format: WINED3DFMT_UNKNOWN,
                    multisample_type: WINED3D_MULTISAMPLE_NONE,
                    multisample_quality: 0,
                    usage: 0,
                    bind_flags: 0,
                    access: 0,
                    width: 0,
                    height: 0,
                    depth: 0,
                    size: 0,
                };
                let mut dst_level: ::core::ffi::c_uint = 0;
                let mut hr: HRESULT = 0;

                dst_level = dst_sub_resource_idx % (*dst_texture).level_count;
                desc.resource_type = WINED3D_RTYPE_TEXTURE_2D;
                desc.format = resolve_format_id;
                desc.multisample_type = (*dst_texture).resource.multisample_type;
                desc.multisample_quality = (*dst_texture).resource.multisample_quality;
                desc.usage = WINED3DUSAGE_CS;
                desc.bind_flags = 0;
                desc.access = WINED3D_RESOURCE_ACCESS_GPU;
                desc.width = wined3d_texture_get_level_width(dst_texture, dst_level);
                desc.height = wined3d_texture_get_level_height(dst_texture, dst_level);
                desc.depth = 1;
                desc.size = 0;

                hr = wined3d_texture_create(
                    device,
                    core::ptr::addr_of_mut!(desc),
                    1,
                    1,
                    0,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                    core::ptr::addr_of!(wined3d_null_parent_ops),
                    core::ptr::addr_of_mut!(dst_texture),
                );
                if hr < 0 {
                    ERR!("Failed to create staging texture, hr %#lx.\n", hr);
                    break 'done;
                }

                wined3d_texture_load_location(dst_texture, 0, context, dst_location);
                dst_sub_resource_idx = 0;
            }
        }

        /* Make sure the locations are up-to-date. Loading the destination
         * surface isn't required if the entire surface is overwritten. (And is
         * in fact harmful if we're being called by surface_load_location() with
         * the purpose of loading the destination surface.) */
        wined3d_texture_load_location(src_texture, src_sub_resource_idx, context, src_location);
        if !wined3d_texture_is_full_rect(
            dst_texture,
            dst_sub_resource_idx % (*dst_texture).level_count,
            dst_rect,
        ) {
            wined3d_texture_load_location(dst_texture, dst_sub_resource_idx, context, dst_location);
        } else {
            wined3d_texture_prepare_location(dst_texture, dst_sub_resource_idx, context, dst_location);
        }

        /* Acquire a context for the front-buffer, even though we may be blitting
         * to/from a back-buffer. Since context_acquire() doesn't take the
         * resource location into account, it may consider the back-buffer to be
         * offscreen. */
        if src_location == WINED3D_LOCATION_DRAWABLE {
            required_texture = (*(*src_texture).swapchain).front_buffer;
        } else if dst_location == WINED3D_LOCATION_DRAWABLE {
            required_texture = (*(*dst_texture).swapchain).front_buffer;
        } else {
            required_texture = core::ptr::null_mut();
        }

        restore_texture = (*context).current_rt.texture;
        restore_idx = (*context).current_rt.sub_resource_idx;
        if restore_texture != required_texture {
            context = context_acquire(device, required_texture, 0);
            restore_context = true;
        }

        context_gl = wined3d_context_gl(context);
        if (*context_gl).valid() == 0 {
            context_release(context);
            WARN!("Invalid context, skipping blit.\n");
            restore_context = false;
            break 'done;
        }

        gl_info = (*context_gl).gl_info;

        if src_location == WINED3D_LOCATION_DRAWABLE {
            TRACE!("Source texture %p is onscreen.\n", src_texture);
            buffer = wined3d_texture_get_gl_buffer(src_texture);
            s = *src_rect;
            wined3d_texture_translate_drawable_coords(src_texture, (*context_gl).window, core::ptr::addr_of_mut!(s));
            src_rect = core::ptr::addr_of_mut!(s);
        } else {
            TRACE!("Source texture %p is offscreen.\n", src_texture);
            buffer = GL_COLOR_ATTACHMENT0;
        }

        wined3d_context_gl_apply_fbo_state_explicit(
            context_gl,
            GL_READ_FRAMEBUFFER,
            core::ptr::addr_of_mut!((*src_texture).resource),
            src_sub_resource_idx,
            core::ptr::null_mut(),
            0,
            src_location,
        );
        (*gl_info).gl_ops.gl.p_glReadBuffer.unwrap_unchecked()(buffer);
        checkGLcall!(gl_info, "glReadBuffer()");
        wined3d_context_gl_check_fbo_status(context_gl, GL_READ_FRAMEBUFFER);

        context_gl_apply_texture_draw_state(context_gl, dst_texture, dst_sub_resource_idx, dst_location);

        if dst_location == WINED3D_LOCATION_DRAWABLE {
            d = *dst_rect;
            wined3d_texture_translate_drawable_coords(dst_texture, (*context_gl).window, core::ptr::addr_of_mut!(d));
            dst_rect = core::ptr::addr_of_mut!(d);
        }

        (*gl_info).gl_ops.gl.p_glColorMask.unwrap_unchecked()(
            (GL_TRUE as u8),
            (GL_TRUE as u8),
            (GL_TRUE as u8),
            (GL_TRUE as u8),
        );
        context_invalidate_state(context, (STATE_BLEND as u32));

        (*gl_info).gl_ops.gl.p_glDisable.unwrap_unchecked()(GL_SCISSOR_TEST);
        context_invalidate_state(context, (STATE_RASTERIZER as u32));

        (*gl_info).fbo_ops.glBlitFramebuffer.unwrap_unchecked()(
            (*src_rect).left as GLint,
            (*src_rect).top as GLint,
            (*src_rect).right as GLint,
            (*src_rect).bottom as GLint,
            (*dst_rect).left as GLint,
            (*dst_rect).top as GLint,
            (*dst_rect).right as GLint,
            (*dst_rect).bottom as GLint,
            GL_COLOR_BUFFER_BIT,
            gl_filter,
        );
        checkGLcall!(gl_info, "glBlitFramebuffer()");

        if dst_location == WINED3D_LOCATION_DRAWABLE
            && (*(*dst_texture).swapchain).front_buffer == dst_texture
        {
            (*gl_info).gl_ops.gl.p_glFlush.unwrap_unchecked()();
        }

        if dst_texture != dst_save_texture {
            if dst_location == WINED3D_LOCATION_DRAWABLE {
                FIXME!("WINED3D_LOCATION_DRAWABLE not supported for the destination of a typeless resolve.\n");
            }

            (*(*(*device).blitter).ops).blitter_blit.unwrap_unchecked()(
                (*device).blitter,
                WINED3D_BLIT_OP_RAW_BLIT,
                context,
                dst_texture,
                0,
                dst_location,
                dst_rect,
                dst_save_texture,
                dst_save_sub_resource_idx,
                dst_location,
                dst_rect,
                core::ptr::null(),
                WINED3D_TEXF_NONE,
                core::ptr::null(),
            );
        }
    }

    /* done: */
    if dst_texture != dst_save_texture {
        wined3d_texture_decref(dst_texture);
    }

    if !src_staging_texture.is_null() {
        wined3d_texture_decref(src_staging_texture);
    }

    if restore_context {
        context_restore(context, restore_texture, restore_idx);
    }
}

unsafe extern "C" fn texture2d_depth_blt_fbo(device: *const wined3d_device, context: *mut wined3d_context, src_texture: *mut wined3d_texture, src_sub_resource_idx: core::ffi::c_uint, src_location: DWORD, src_rect: *const RECT, dst_texture: *mut wined3d_texture, dst_sub_resource_idx: core::ffi::c_uint, dst_location: DWORD, dst_rect: *const RECT) {
    let context_gl: *mut wined3d_context_gl = wined3d_context_gl(context);
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut src_mask: GLbitfield;
    let mut dst_mask: GLbitfield;
    let mut gl_mask: GLbitfield;

    TRACE!("device %p, src_texture %p, src_sub_resource_idx %u, src_location %s, src_rect %s, dst_texture %p, dst_sub_resource_idx %u, dst_location %s, dst_rect %s.\n",
            device,
            src_texture,
            src_sub_resource_idx,
            wined3d_debug_location(src_location as uint32_t),
            wine_dbgstr_rect(src_rect),
            dst_texture,
            dst_sub_resource_idx,
            wined3d_debug_location(dst_location as uint32_t),
            wine_dbgstr_rect(dst_rect));

    src_mask = 0 as GLbitfield;
    if (*(*src_texture).resource.format).depth_size != 0 {
        src_mask |= GL_DEPTH_BUFFER_BIT as GLbitfield;
    }
    if (*(*src_texture).resource.format).stencil_size != 0 {
        src_mask |= GL_STENCIL_BUFFER_BIT as GLbitfield;
    }

    dst_mask = 0 as GLbitfield;
    if (*(*dst_texture).resource.format).depth_size != 0 {
        dst_mask |= GL_DEPTH_BUFFER_BIT as GLbitfield;
    }
    if (*(*dst_texture).resource.format).stencil_size != 0 {
        dst_mask |= GL_STENCIL_BUFFER_BIT as GLbitfield;
    }

    if src_mask != dst_mask {
        ERR!("Incompatible formats %s and %s.\n",
                debug_d3dformat((*(*src_texture).resource.format).id),
                debug_d3dformat((*(*dst_texture).resource.format).id));
        return;
    }

    if src_mask == 0 {
        ERR!("Not a depth / stencil format: %s.\n",
                debug_d3dformat((*(*src_texture).resource.format).id));
        return;
    }
    gl_mask = src_mask;

    /* Make sure the locations are up-to-date. Loading the destination
     * surface isn't required if the entire surface is overwritten. */
    wined3d_texture_load_location(src_texture, src_sub_resource_idx, context, src_location as uint32_t);
    if !wined3d_texture_is_full_rect(dst_texture, dst_sub_resource_idx % (*dst_texture).level_count, dst_rect) {
        wined3d_texture_load_location(dst_texture, dst_sub_resource_idx, context, dst_location as uint32_t);
    } else {
        wined3d_texture_prepare_location(dst_texture, dst_sub_resource_idx, context, dst_location as core::ffi::c_uint);
    }

    wined3d_context_gl_apply_fbo_state_explicit(context_gl, GL_READ_FRAMEBUFFER as GLenum, core::ptr::null_mut::<wined3d_resource>(), 0 as core::ffi::c_uint,
            core::ptr::addr_of_mut!((*src_texture).resource), src_sub_resource_idx, src_location as uint32_t);
    wined3d_context_gl_check_fbo_status(context_gl, GL_READ_FRAMEBUFFER as GLenum);

    context_gl_apply_texture_draw_state(context_gl, dst_texture, dst_sub_resource_idx, dst_location as core::ffi::c_uint);

    if (gl_mask & (GL_DEPTH_BUFFER_BIT as GLbitfield)) != 0 {
        ((*gl_info).gl_ops.gl.p_glDepthMask.unwrap_unchecked())(GL_TRUE as GLboolean);
        context_invalidate_state(context, STATE_DEPTH_STENCIL as core::ffi::c_uint);
    }
    if (gl_mask & (GL_STENCIL_BUFFER_BIT as GLbitfield)) != 0 {
        if (*gl_info).supported[EXT_STENCIL_TWO_SIDE as usize] != 0 {
            ((*gl_info).gl_ops.gl.p_glDisable.unwrap_unchecked())(GL_STENCIL_TEST_TWO_SIDE_EXT as GLenum);
        }
        ((*gl_info).gl_ops.gl.p_glStencilMask.unwrap_unchecked())(!(0 as GLuint));
        context_invalidate_state(context, STATE_DEPTH_STENCIL as core::ffi::c_uint);
    }

    ((*gl_info).gl_ops.gl.p_glDisable.unwrap_unchecked())(GL_SCISSOR_TEST as GLenum);
    context_invalidate_state(context, STATE_RASTERIZER as core::ffi::c_uint);

    ((*gl_info).fbo_ops.glBlitFramebuffer.unwrap_unchecked())(
            (*src_rect).left as GLint,
            (*src_rect).top as GLint,
            (*src_rect).right as GLint,
            (*src_rect).bottom as GLint,
            (*dst_rect).left as GLint,
            (*dst_rect).top as GLint,
            (*dst_rect).right as GLint,
            (*dst_rect).bottom as GLint,
            gl_mask,
            GL_NEAREST as GLenum);
    checkGLcall!(gl_info, "glBlitFramebuffer()");
}

unsafe extern "C" fn wined3d_texture_evict_sysmem(texture: *mut wined3d_texture) {
    let mut sub_resource: *mut wined3d_texture_sub_resource = core::ptr::null_mut();
    let mut i: core::ffi::c_uint = 0;
    let mut sub_count: core::ffi::c_uint = 0;

    if ((*texture).flags & (WINED3D_TEXTURE_CONVERTED as uint32_t)) != 0
        || (*texture).resource.pin_sysmem() as core::ffi::c_int != 0
        || (*texture).download_count > (WINED3D_TEXTURE_DYNAMIC_MAP_THRESHOLD as core::ffi::c_uint)
    {
        return;
    }

    sub_count = ((*texture).level_count as UINT).wrapping_mul((*texture).layer_count)
        as core::ffi::c_uint;
    i = 0 as core::ffi::c_uint;
    while i < sub_count {
        sub_resource = (*texture).sub_resources.offset(i as isize) as *mut wined3d_texture_sub_resource;
        if (*sub_resource).locations == (WINED3D_LOCATION_SYSMEM as DWORD) {
            ERR!(
                "WINED3D_LOCATION_SYSMEM is the only location for sub-resource %u of texture %p.\n",
                i,
                texture
            );
        }
        (*sub_resource).locations &= !(WINED3D_LOCATION_SYSMEM as DWORD);
        i = i.wrapping_add(1);
    }
    wined3d_resource_free_sysmem(core::ptr::addr_of_mut!((*texture).resource));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_validate_location(mut texture: *mut wined3d_texture, mut sub_resource_idx: ::core::ffi::c_uint, mut location: uint32_t) {
    let mut sub_resource: *mut wined3d_texture_sub_resource =
        ::core::ptr::null_mut::<wined3d_texture_sub_resource>();
    let mut previous_locations: DWORD = 0;
    TRACE!(
        "texture %p, sub_resource_idx %u, location %s.\n",
        texture,
        sub_resource_idx,
        wined3d_debug_location(location)
    );
    sub_resource = (*texture).sub_resources.offset(sub_resource_idx as isize)
        as *mut wined3d_texture_sub_resource;
    previous_locations = (*sub_resource).locations;
    (*sub_resource).locations |= location as ::core::ffi::c_uint;
    if previous_locations == WINED3D_LOCATION_SYSMEM as DWORD
        && location != WINED3D_LOCATION_SYSMEM as uint32_t
        && {
            (*texture).sysmem_count = (*texture).sysmem_count.wrapping_sub(1);
            (*texture).sysmem_count == 0
        }
    {
        wined3d_texture_evict_sysmem(texture);
    }
    TRACE!(
        "New locations flags are %s.\n",
        wined3d_debug_location((*sub_resource).locations as uint32_t)
    );
}

unsafe extern "C" fn wined3d_texture_set_dirty(mut texture: *mut wined3d_texture) {
    (*texture).flags &= !(WINED3D_TEXTURE_RGB_VALID | WINED3D_TEXTURE_SRGB_VALID) as u32;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_invalidate_location(texture: *mut wined3d_texture, sub_resource_idx: ::core::ffi::c_uint, location: uint32_t) {
    let mut sub_resource: *mut wined3d_texture_sub_resource = core::ptr::null_mut();
    let mut previous_locations: DWORD = 0;

    if location & ((WINED3D_LOCATION_TEXTURE_RGB as u32) | (WINED3D_LOCATION_TEXTURE_SRGB as u32)) != 0 {
        wined3d_texture_set_dirty(texture);
    }

    sub_resource = (*texture).sub_resources.offset(sub_resource_idx as isize) as *mut wined3d_texture_sub_resource;
    previous_locations = (*sub_resource).locations;
    (*sub_resource).locations &= !location;
    if previous_locations != (WINED3D_LOCATION_SYSMEM as DWORD)
        && (*sub_resource).locations == (WINED3D_LOCATION_SYSMEM as DWORD)
    {
        (*texture).sysmem_count = (*texture).sysmem_count.wrapping_add(1);
    }

    if (*sub_resource).locations == 0 {
        ERR!(
            "Sub-resource %u of texture %p does not have any up to date location.\n",
            sub_resource_idx,
            texture
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_get_bo_address(texture: *const wined3d_texture, sub_resource_idx: ::core::ffi::c_uint, data: *mut wined3d_bo_address, location: uint32_t) {
    let sub_resource: *mut wined3d_texture_sub_resource =
        (*texture).sub_resources.offset(sub_resource_idx as isize) as *mut wined3d_texture_sub_resource;

    if location == WINED3D_LOCATION_BUFFER {
        (*data).addr = core::ptr::null_mut();
        (*data).buffer_object = (*sub_resource).bo;
    } else {
        assert!(location == WINED3D_LOCATION_SYSMEM);
        if !(*sub_resource).user_memory.is_null() {
            (*data).addr = ((*sub_resource).user_memory as *mut core::ffi::c_void).cast();
        } else {
            (*data).addr = ((*texture).resource.heap_memory as *mut core::ffi::c_void).cast();
            (*data).addr = (*data).addr.offset((*sub_resource).offset as isize);
        }
        (*data).buffer_object = core::ptr::null_mut();
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_clear_dirty_regions(mut texture: *mut wined3d_texture) {
    let mut i: u32 = 0;
    TRACE!("texture %p\n", texture);
    if (*texture).dirty_regions.is_null() {
        return;
    }
    i = 0;
    while i < (*texture).layer_count {
        (*(*texture).dirty_regions.offset(i as isize)).box_count = 0;
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_load_location(texture: *mut wined3d_texture, sub_resource_idx: ::core::ffi::c_uint, context: *mut wined3d_context, location: uint32_t) -> BOOL {
    let mut current: DWORD = (*(*texture).sub_resources.offset(sub_resource_idx as isize)).locations;
    let mut ret: BOOL = 0;

    TRACE!(
        "texture %p, sub_resource_idx %u, context %p, location %s.\n",
        texture,
        sub_resource_idx,
        context,
        wined3d_debug_location(location)
    );
    TRACE!(
        "Current resource location %s.\n",
        wined3d_debug_location(current as uint32_t)
    );

    if current & location != 0 {
        TRACE!(
            "Location %s is already up to date.\n",
            wined3d_debug_location(location)
        );
        return (TRUE) as BOOL;
    }

    if WARN_ON!(__wine_dbch_d3d) != 0 {
        let mut required_access: uint32_t = wined3d_resource_access_from_location(location);
        if (*texture).resource.access as uint32_t & required_access != required_access {
            WARN!(
                "Operation requires %#x access, but texture only has %#x.\n",
                required_access,
                (*texture).resource.access
            );
        }
    }

    if current & WINED3D_LOCATION_DISCARDED != 0 {
        if wined3d_texture_prepare_location(texture, sub_resource_idx, context, location) == 0 {
            return (FALSE) as BOOL;
        }
        wined3d_texture_validate_location(texture, sub_resource_idx, location);
        wined3d_texture_invalidate_location(texture, sub_resource_idx, WINED3D_LOCATION_DISCARDED);
        return (TRUE) as BOOL;
    }

    if current == 0 {
        ERR!(
            "Sub-resource %u of texture %p does not have any up to date location.\n",
            sub_resource_idx,
            texture
        );
        wined3d_texture_validate_location(texture, sub_resource_idx, WINED3D_LOCATION_DISCARDED);
        return wined3d_texture_load_location(texture, sub_resource_idx, context, location);
    }

    if location & wined3d_texture_sysmem_locations != 0
        && current & (wined3d_texture_sysmem_locations | WINED3D_LOCATION_CLEARED) != 0
    {
        let mut source: wined3d_bo_address = wined3d_bo_address {
            buffer_object: core::ptr::null_mut::<wined3d_bo>(),
            addr: core::ptr::null_mut::<BYTE>(),
        };
        let mut destination: wined3d_bo_address = wined3d_bo_address {
            buffer_object: core::ptr::null_mut::<wined3d_bo>(),
            addr: core::ptr::null_mut::<BYTE>(),
        };
        let mut range: wined3d_range = wined3d_range { offset: 0, size: 0 };

        if wined3d_texture_prepare_location(texture, sub_resource_idx, context, location) == 0 {
            return (FALSE) as BOOL;
        }
        wined3d_texture_get_bo_address(
            texture,
            sub_resource_idx,
            core::ptr::addr_of_mut!(destination),
            location,
        );
        range.offset = 0;
        range.size = (*(*texture).sub_resources.offset(sub_resource_idx as isize)).size;

        if current & WINED3D_LOCATION_CLEARED != 0 {
            let mut level_idx: ::core::ffi::c_uint =
                sub_resource_idx.wrapping_rem((*texture).level_count);
            let mut map: wined3d_map_desc = wined3d_map_desc {
                row_pitch: 0,
                slice_pitch: 0,
                data: core::ptr::null_mut::<::core::ffi::c_void>(),
            };
            let mut box_0: wined3d_box = wined3d_box {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
                front: 0,
                back: 0,
            };
            let mut c: wined3d_color = wined3d_color {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 0.,
            };

            if (*(*texture).resource.format).caps[WINED3D_GL_RES_TYPE_TEX_2D as usize]
                & WINED3D_FORMAT_CAP_DEPTH_STENCIL
                != 0
            {
                c.r = (*(*texture).sub_resources.offset(sub_resource_idx as isize))
                    .clear_value
                    .__bindgen_anon_1.depth;
                c.g = (*(*texture).sub_resources.offset(sub_resource_idx as isize))
                    .clear_value
                    .__bindgen_anon_1.stencil as ::core::ffi::c_float;
                c.a = 0.0f32;
                c.b = c.a;
            } else {
                c = (*(*texture).sub_resources.offset(sub_resource_idx as isize))
                    .clear_value
                    .colour;
            }

            wined3d_texture_get_pitch(
                texture,
                level_idx,
                core::ptr::addr_of_mut!(map.row_pitch),
                core::ptr::addr_of_mut!(map.slice_pitch),
            );
            if !destination.buffer_object.is_null() {
                map.data = wined3d_context_map_bo_address(
                    context,
                    core::ptr::addr_of_mut!(destination),
                    range.size as size_t,
                    WINED3D_MAP_WRITE | WINED3D_MAP_DISCARD,
                );
            } else {
                map.data = destination.addr as *mut ::core::ffi::c_void;
            }

            wined3d_texture_get_level_box(texture, level_idx, core::ptr::addr_of_mut!(box_0));
            wined3d_resource_memory_colour_fill(
                core::ptr::addr_of_mut!((*texture).resource),
                core::ptr::addr_of_mut!(map),
                core::ptr::addr_of_mut!(c),
                core::ptr::addr_of_mut!(box_0),
                true,
            );

            if !destination.buffer_object.is_null() {
                wined3d_context_unmap_bo_address(
                    context,
                    core::ptr::addr_of_mut!(destination),
                    1,
                    core::ptr::addr_of_mut!(range),
                );
            }
        } else {
            wined3d_texture_get_bo_address(
                texture,
                sub_resource_idx,
                core::ptr::addr_of_mut!(source),
                current & wined3d_texture_sysmem_locations,
            );
            wined3d_context_copy_bo_address(
                context,
                core::ptr::addr_of_mut!(destination),
                core::ptr::addr_of_mut!(source),
                1,
                core::ptr::addr_of_mut!(range),
                WINED3D_MAP_WRITE | WINED3D_MAP_DISCARD,
            );
        }
        ret = (TRUE as i32);
    } else {
        ret = (*(*texture).texture_ops)
            .texture_load_location
            .unwrap_unchecked()(
            texture,
            sub_resource_idx,
            context,
            location,
        );
    }

    if ret != 0 {
        wined3d_texture_validate_location(texture, sub_resource_idx, location);
    }

    return ret;
}

unsafe extern "C" fn wined3d_texture_get_memory(
    texture: *mut wined3d_texture,
    sub_resource_idx: ::core::ffi::c_uint,
    context: *mut wined3d_context,
    data: *mut wined3d_bo_address,
) {
    let sub_resource: *mut wined3d_texture_sub_resource =
        (*texture).sub_resources.offset(sub_resource_idx as isize) as *mut wined3d_texture_sub_resource;
    let mut locations: DWORD = (*sub_resource).locations;

    if (locations & ((WINED3D_LOCATION_DISCARDED as DWORD) | (WINED3D_LOCATION_CLEARED as DWORD))) != 0 {
        locations = (if wined3d_texture_use_pbo(texture, (*context).d3d_info) != 0 {
            WINED3D_LOCATION_BUFFER as ::core::ffi::c_int
        } else {
            WINED3D_LOCATION_SYSMEM as ::core::ffi::c_int
        }) as DWORD;
        if wined3d_texture_load_location(texture, sub_resource_idx, context, locations as uint32_t) == 0 {
            (*data).buffer_object = core::ptr::null_mut();
            (*data).addr = ::core::ptr::null_mut();
            return;
        }
    }
    if (locations & (WINED3D_LOCATION_BUFFER as DWORD)) != 0 {
        wined3d_texture_get_bo_address(
            texture,
            sub_resource_idx,
            data,
            WINED3D_LOCATION_BUFFER as uint32_t,
        );
        return;
    }

    if (locations & (WINED3D_LOCATION_SYSMEM as DWORD)) != 0 {
        wined3d_texture_get_bo_address(
            texture,
            sub_resource_idx,
            data,
            WINED3D_LOCATION_SYSMEM as uint32_t,
        );
        return;
    }

    ERR!("Unexpected locations %s.\n", wined3d_debug_location(locations as uint32_t));
    (*data).addr = ::core::ptr::null_mut();
    (*data).buffer_object = core::ptr::null_mut();
}

unsafe extern "C" fn wined3d_texture_remove_buffer_object(
    texture: *mut wined3d_texture,
    sub_resource_idx: ::core::ffi::c_uint,
    context_gl: *mut wined3d_context_gl,
) {
    let sub_resource: *mut wined3d_texture_sub_resource =
        (*texture).sub_resources.add(sub_resource_idx as usize);
    let bo_gl: *mut wined3d_bo_gl = wined3d_bo_gl((*sub_resource).bo);

    wined3d_context_gl_destroy_bo(context_gl, bo_gl);
    wined3d_texture_invalidate_location(texture, sub_resource_idx, WINED3D_LOCATION_BUFFER);
    (*sub_resource).bo = core::ptr::null_mut();
    free(bo_gl as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn wined3d_texture_unload_location(texture: *mut wined3d_texture, context: *mut wined3d_context, location: u32) {
    ((*(*texture).texture_ops).texture_unload_location.unwrap_unchecked())(texture, context, location);
}

unsafe extern "C" fn wined3d_texture_update_map_binding(texture: *mut wined3d_texture) {
    let sub_count: ::core::ffi::c_uint = (*texture)
        .level_count
        .wrapping_mul((*texture).layer_count as ::core::ffi::c_uint);
    let device: *mut wined3d_device = (*texture).resource.device;
    let map_binding: DWORD = (*texture).update_map_binding;
    let mut context: *mut wined3d_context;
    let mut i: ::core::ffi::c_uint;

    context = context_acquire(device, core::ptr::null_mut::<wined3d_texture>(), 0);

    i = 0;
    while i < sub_count {
        if (*(*texture).sub_resources.add(i as usize)).locations == (*texture).resource.map_binding as DWORD
            && wined3d_texture_load_location(texture, i, context, map_binding as uint32_t) == 0
        {
            ERR!("Failed to load location %s.\n", wined3d_debug_location(map_binding as uint32_t));
        }
        i = i.wrapping_add(1);
    }

    if (*texture).resource.map_binding as ::core::ffi::c_int == (WINED3D_LOCATION_BUFFER as i32) {
        wined3d_texture_unload_location(texture, context, WINED3D_LOCATION_BUFFER as ::core::ffi::c_uint);
    }

    context_release(context);

    (*texture).resource.map_binding = map_binding as WORD;
    (*texture).update_map_binding = 0;
}

unsafe extern "C" fn wined3d_texture_set_map_binding(texture: *mut wined3d_texture, map_binding: DWORD) {
    (*texture).update_map_binding = map_binding;
    if (*texture).resource.map_count == 0 {
        wined3d_texture_update_map_binding(texture);
    }
}

unsafe extern "C" fn gltexture_delete(
    device: *mut wined3d_device,
    gl_info: *const wined3d_gl_info,
    tex: *mut gl_texture,
) {
    context_gl_resource_released(device, (*tex).name, (FALSE) as BOOL);
    (*gl_info)
        .gl_ops
        .gl
        .p_glDeleteTextures
        .unwrap_unchecked()(1, core::ptr::addr_of_mut!((*tex).name));
    (*tex).name = 0;
}

unsafe extern "C" fn wined3d_texture_gl_allocate_mutable_storage(
    texture_gl: *mut wined3d_texture_gl,
    gl_internal_format: GLenum,
    format: *const wined3d_format_gl,
    gl_info: *const wined3d_gl_info,
) {
    let mut level: ::core::ffi::c_uint = 0;
    let mut level_count: ::core::ffi::c_uint = 0;
    let mut layer: ::core::ffi::c_uint = 0;
    let mut layer_count: ::core::ffi::c_uint = 0;
    let mut width: GLsizei = 0;
    let mut height: GLsizei = 0;
    let mut depth: GLsizei = 0;
    let mut target: GLenum = 0;

    level_count = (*texture_gl).t.level_count;
    if (*texture_gl).target == GL_TEXTURE_1D_ARRAY || (*texture_gl).target == GL_TEXTURE_2D_ARRAY {
        layer_count = 1 as ::core::ffi::c_uint;
    } else {
        layer_count = (*texture_gl).t.layer_count as ::core::ffi::c_uint;
    }

    ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(GL_PIXEL_UNPACK_BUFFER, 0);
    checkGLcall!(gl_info, b"glBindBuffer\0".as_ptr() as *const ::core::ffi::c_char);

    layer = 0 as ::core::ffi::c_uint;
    while layer < layer_count {
        target = wined3d_texture_gl_get_sub_resource_target(texture_gl, layer.wrapping_mul(level_count));

        level = 0 as ::core::ffi::c_uint;
        while level < level_count {
            width = wined3d_texture_get_level_width(core::ptr::addr_of_mut!((*texture_gl).t), level) as GLsizei;
            height = wined3d_texture_get_level_height(core::ptr::addr_of_mut!((*texture_gl).t), level) as GLsizei;
            if (*texture_gl).t.resource.format_attrs & (WINED3D_FORMAT_ATTR_HEIGHT_SCALE as ::core::ffi::c_uint) != 0 {
                height = (height as UINT).wrapping_mul((*format).f.height_scale.numerator) as GLsizei;
                height = (height as UINT).wrapping_div((*format).f.height_scale.denominator) as GLsizei;
            }

            WINE_TRACE!("texture_gl %p, layer %u, level %u, target %#x, width %u, height %u.\n", texture_gl, layer, level, target, width, height);

            if target == GL_TEXTURE_3D || target == GL_TEXTURE_2D_ARRAY {
                depth = wined3d_texture_get_level_depth(core::ptr::addr_of_mut!((*texture_gl).t), level) as GLsizei;
                ((*gl_info).gl_ops.ext.p_glTexImage3D.unwrap_unchecked())(
                    target,
                    level as GLint,
                    gl_internal_format as GLint,
                    width,
                    height,
                    (if target == GL_TEXTURE_2D_ARRAY {
                        (*texture_gl).t.layer_count
                    } else {
                        depth as UINT
                    }) as GLsizei,
                    0 as GLint,
                    (*format).format as GLenum,
                    (*format).type_ as GLenum,
                    core::ptr::null::<::core::ffi::c_void>(),
                );
                checkGLcall!(gl_info, b"glTexImage3D\0".as_ptr() as *const ::core::ffi::c_char);
            } else if target == GL_TEXTURE_1D {
                (*gl_info)
                    .gl_ops
                    .gl
                    .p_glTexImage1D
                    .unwrap_unchecked()(
                    target,
                    level as GLint,
                    gl_internal_format as GLint,
                    width,
                    0 as GLint,
                    (*format).format as GLenum,
                    (*format).type_ as GLenum,
                    core::ptr::null::<::core::ffi::c_void>(),
                );
            } else {
                (*gl_info)
                    .gl_ops
                    .gl
                    .p_glTexImage2D
                    .unwrap_unchecked()(
                    target,
                    level as GLint,
                    gl_internal_format as GLint,
                    width,
                    (if target == GL_TEXTURE_1D_ARRAY {
                        (*texture_gl).t.layer_count
                    } else {
                        height as UINT
                    }) as GLsizei,
                    0 as GLint,
                    (*format).format as GLenum,
                    (*format).type_ as GLenum,
                    core::ptr::null::<::core::ffi::c_void>(),
                );
                checkGLcall!(gl_info, b"glTexImage2D\0".as_ptr() as *const ::core::ffi::c_char);
            }
            level = level.wrapping_add(1);
        }
        layer = layer.wrapping_add(1);
    }
}

unsafe extern "C" fn wined3d_texture_gl_allocate_immutable_storage(
    mut texture_gl: *mut wined3d_texture_gl,
    mut gl_internal_format: GLenum,
    mut gl_info: *const wined3d_gl_info,
) {
    let mut samples: ::core::ffi::c_uint =
        wined3d_resource_get_sample_count(core::ptr::addr_of_mut!((*texture_gl).t.resource));
    let mut standard_pattern: GLboolean = (((*texture_gl).t.resource.multisample_type
        as ::core::ffi::c_uint
        != WINED3D_MULTISAMPLE_NON_MASKABLE as ::core::ffi::c_uint
        && (*texture_gl).t.resource.multisample_quality
            == WINED3D_STANDARD_MULTISAMPLE_PATTERN as ::core::ffi::c_uint)
        as ::core::ffi::c_int) as GLboolean;
    let mut height: GLsizei = wined3d_texture_get_level_height(
        core::ptr::addr_of_mut!((*texture_gl).t),
        0 as ::core::ffi::c_uint,
    ) as GLsizei;
    let mut width: GLsizei = wined3d_texture_get_level_width(
        core::ptr::addr_of_mut!((*texture_gl).t),
        0 as ::core::ffi::c_uint,
    ) as GLsizei;

    match (*texture_gl).target {
        GL_TEXTURE_3D => {
            ((*gl_info).gl_ops.ext.p_glTexStorage3D.unwrap_unchecked())(
                (*texture_gl).target,
                (*texture_gl).t.level_count as GLsizei,
                gl_internal_format,
                width,
                height,
                wined3d_texture_get_level_depth(
                    core::ptr::addr_of_mut!((*texture_gl).t),
                    0 as ::core::ffi::c_uint,
                ) as GLsizei,
            );
        }
        GL_TEXTURE_2D_ARRAY => {
            ((*gl_info).gl_ops.ext.p_glTexStorage3D.unwrap_unchecked())(
                (*texture_gl).target,
                (*texture_gl).t.level_count as GLsizei,
                gl_internal_format,
                width,
                height,
                (*texture_gl).t.layer_count as GLsizei,
            );
        }
        GL_TEXTURE_2D_MULTISAMPLE => {
            ((*gl_info).gl_ops.ext.p_glTexStorage2DMultisample.unwrap_unchecked())(
                (*texture_gl).target,
                samples as GLsizei,
                gl_internal_format,
                width,
                height,
                standard_pattern,
            );
        }
        GL_TEXTURE_2D_MULTISAMPLE_ARRAY => {
            ((*gl_info).gl_ops.ext.p_glTexStorage3DMultisample.unwrap_unchecked())(
                (*texture_gl).target,
                samples as GLsizei,
                gl_internal_format,
                width,
                height,
                (*texture_gl).t.layer_count as GLsizei,
                standard_pattern,
            );
        }
        GL_TEXTURE_1D_ARRAY => {
            ((*gl_info).gl_ops.ext.p_glTexStorage2D.unwrap_unchecked())(
                (*texture_gl).target,
                (*texture_gl).t.level_count as GLsizei,
                gl_internal_format,
                width,
                (*texture_gl).t.layer_count as GLsizei,
            );
        }
        GL_TEXTURE_1D => {
            ((*gl_info).gl_ops.ext.p_glTexStorage1D.unwrap_unchecked())(
                (*texture_gl).target,
                (*texture_gl).t.level_count as GLsizei,
                gl_internal_format,
                width,
            );
        }
        _ => {
            ((*gl_info).gl_ops.ext.p_glTexStorage2D.unwrap_unchecked())(
                (*texture_gl).target,
                (*texture_gl).t.level_count as GLsizei,
                gl_internal_format,
                width,
                height,
            );
        }
    }

    checkGLcall!(gl_info, "allocate immutable storage");
}

unsafe extern "C" fn wined3d_texture_dirty_region_add(
    mut texture: *mut wined3d_texture,
    mut layer: ::core::ffi::c_uint,
    mut box_0: *const wined3d_box,
) {
    let mut regions: *mut wined3d_dirty_regions = ::core::ptr::null_mut::<wined3d_dirty_regions>();
    let mut count: ::core::ffi::c_uint = 0;
    if (*texture).dirty_regions.is_null() {
        return;
    }
    regions = (*texture).dirty_regions.offset(layer as isize) as *mut wined3d_dirty_regions
        as *mut wined3d_dirty_regions;
    count = (*regions).box_count.wrapping_add(1 as ::core::ffi::c_uint);
    if count >= WINED3D_MAX_DIRTY_REGION_COUNT as ::core::ffi::c_uint
        || box_0.is_null()
        || (*box_0).left == 0
            && (*box_0).top == 0
            && (*box_0).front == 0
            && (*box_0).right == (*texture).resource.width
            && (*box_0).bottom == (*texture).resource.height
            && (*box_0).back == (*texture).resource.depth
    {
        (*regions).box_count = WINED3D_MAX_DIRTY_REGION_COUNT as ::core::ffi::c_uint;
        return;
    }
    if wined3d_array_reserve(
        &raw mut (*regions).boxes as *mut *mut ::core::ffi::c_void,
        &raw mut (*regions).boxes_size,
        count as SIZE_T,
        ::core::mem::size_of::<wined3d_box>() as SIZE_T,
    ) == 0
    {
        ERR!("Failed to grow boxes array, marking entire texture dirty.\n");
        (*regions).box_count = WINED3D_MAX_DIRTY_REGION_COUNT as ::core::ffi::c_uint;
        return;
    }
    let fresh15 = (*regions).box_count;
    (*regions).box_count = (*regions).box_count.wrapping_add(1);
    *(*regions).boxes.offset(fresh15 as isize) = *box_0;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_sub_resources_destroyed(texture: *mut wined3d_texture) {
    let mut sub_count: ::core::ffi::c_uint = (*texture)
        .level_count
        .wrapping_mul((*texture).layer_count as ::core::ffi::c_uint);
    let mut sub_resource: *mut wined3d_texture_sub_resource =
        core::ptr::null_mut::<wined3d_texture_sub_resource>();
    let mut i: ::core::ffi::c_uint = 0;
    i = 0 as ::core::ffi::c_uint;
    while i < sub_count {
        sub_resource = (*texture).sub_resources.offset(i as isize)
            as *mut wined3d_texture_sub_resource
            as *mut wined3d_texture_sub_resource;
        if !(*sub_resource).parent.is_null() {
            WINE_TRACE!("sub-resource %u.\n", i);
            (*(*sub_resource).parent_ops)
                .wined3d_object_destroyed
                .unwrap_unchecked()((*sub_resource).parent);
            (*sub_resource).parent = core::ptr::null_mut();
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn wined3d_texture_create_dc(mut object: *mut ::core::ffi::c_void) {
    let idx: *const wined3d_texture_idx = object as *const wined3d_texture_idx;
    let mut context: *mut wined3d_context = ::core::ptr::null_mut::<wined3d_context>();
    let mut sub_resource_idx: ::core::ffi::c_uint = 0;
    let mut level: ::core::ffi::c_uint = 0;
    let mut format: *const wined3d_format = ::core::ptr::null::<wined3d_format>();
    let mut row_pitch: ::core::ffi::c_uint = 0;
    let mut slice_pitch: ::core::ffi::c_uint = 0;
    let mut texture: *mut wined3d_texture = ::core::ptr::null_mut::<wined3d_texture>();
    let mut dc_info: *mut wined3d_dc_info = ::core::ptr::null_mut::<wined3d_dc_info>();
    let mut data: wined3d_bo_address = wined3d_bo_address {
        buffer_object: ::core::ptr::null_mut::<wined3d_bo>(),
        addr: ::core::ptr::null_mut::<BYTE>(),
    };
    let mut desc: D3DKMT_CREATEDCFROMMEMORY = core::mem::zeroed();
    let mut device: *mut wined3d_device = ::core::ptr::null_mut::<wined3d_device>();
    let mut status: NTSTATUS = 0;

    texture = (*idx).texture;
    sub_resource_idx = (*idx).sub_resource_idx;
    level = sub_resource_idx.wrapping_rem((*texture).level_count);
    device = (*texture).resource.device;

    format = (*texture).resource.format;
    if (*format).ddi_format as u64 == 0 {
        WARN!("Cannot create a DC for format %s.\n", debug_d3dformat((*format).id));
        return;
    }

    if (*texture).dc_info.is_null() {
        let sub_count: ::core::ffi::c_uint = (*texture)
            .level_count
            .wrapping_mul((*texture).layer_count as ::core::ffi::c_uint);

        (*texture).dc_info = calloc(
            (sub_count as size_t) as _,
            (core::mem::size_of::<wined3d_dc_info>() as size_t) as _,
        ) as *mut wined3d_texture_wined3d_dc_info;
        if (*texture).dc_info.is_null() {
            ERR!("Failed to allocate DC info.\n");
            return;
        }
    }

    if ((*(*texture).sub_resources.add(sub_resource_idx as usize)).locations
        & (*texture).resource.map_binding as DWORD)
        == 0
    {
        context = context_acquire(
            device,
            ::core::ptr::null_mut::<wined3d_texture>(),
            0,
        );
        wined3d_texture_load_location(
            texture,
            sub_resource_idx,
            context,
            (*texture).resource.map_binding as uint32_t,
        );
    }

    if !(*texture).dirty_regions.is_null() {
        wined3d_texture_dirty_region_add(
            texture,
            sub_resource_idx.wrapping_div((*texture).level_count),
            ::core::ptr::null::<wined3d_box>(),
        );
    }

    wined3d_texture_invalidate_location(
        texture,
        sub_resource_idx,
        !((*texture).resource.map_binding as ::core::ffi::c_int) as uint32_t,
    );
    wined3d_texture_get_pitch(
        texture,
        level,
        core::ptr::addr_of_mut!(row_pitch),
        core::ptr::addr_of_mut!(slice_pitch),
    );
    wined3d_texture_get_bo_address(
        texture,
        sub_resource_idx,
        core::ptr::addr_of_mut!(data),
        (*texture).resource.map_binding as uint32_t,
    );
    if !data.buffer_object.is_null() {
        if context.is_null() {
            context = context_acquire(
                device,
                ::core::ptr::null_mut::<wined3d_texture>(),
                0,
            );
        }
        desc.pMemory = wined3d_context_map_bo_address(
            context,
            core::ptr::addr_of_mut!(data),
            (*(*texture).sub_resources.add(sub_resource_idx as usize)).size as size_t,
            WINED3D_MAP_READ | WINED3D_MAP_WRITE,
        );
    } else {
        desc.pMemory = data.addr as *mut ::core::ffi::c_void;
    }

    if !context.is_null() {
        context_release(context);
    }

    desc.Format = (*format).ddi_format;
    desc.Width = wined3d_texture_get_level_width(texture, level) as UINT;
    desc.Height = wined3d_texture_get_level_height(texture, level) as UINT;
    desc.Pitch = row_pitch as UINT;
    desc.hDeviceDc = CreateCompatibleDC(::core::ptr::null_mut::<HDC__>());
    desc.pColorTable = ::core::ptr::null_mut::<PALETTEENTRY>();

    status = D3DKMTCreateDCFromMemory(core::ptr::addr_of_mut!(desc)) as NTSTATUS;
    DeleteDC(desc.hDeviceDc);
    if status != 0 {
        WARN!("Failed to create DC, status %#lx.\n", status);
        return;
    }

    dc_info = (*texture).dc_info.add(sub_resource_idx as usize) as *mut wined3d_dc_info;
    (*dc_info).dc = desc.hDc;
    (*dc_info).bitmap = desc.hBitmap as HBITMAP;

    TRACE!(
        "Created DC %p, bitmap %p for texture %p, %u.\n",
        (*dc_info).dc,
        (*dc_info).bitmap,
        texture,
        sub_resource_idx
    );
}

unsafe extern "C" fn wined3d_texture_destroy_dc(object: *mut core::ffi::c_void) {
    let idx: *const wined3d_texture_idx = object as *const wined3d_texture_idx;
    let mut destroy_desc: D3DKMT_DESTROYDCFROMMEMORY = D3DKMT_DESTROYDCFROMMEMORY {
        hDc: core::ptr::null_mut(),
        hBitmap: core::ptr::null_mut(),
    };
    let mut context: *mut wined3d_context = core::ptr::null_mut();
    let mut texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut dc_info: *mut wined3d_dc_info = core::ptr::null_mut();
    let mut data: wined3d_bo_address = wined3d_bo_address {
        buffer_object: core::ptr::null_mut(),
        addr: core::ptr::null_mut(),
    };
    let mut sub_resource_idx: core::ffi::c_uint = 0;
    let mut device: *mut wined3d_device = core::ptr::null_mut();
    let mut range: wined3d_range = wined3d_range { offset: 0, size: 0 };
    let mut status: NTSTATUS;

    texture = (*idx).texture;
    sub_resource_idx = (*idx).sub_resource_idx;
    device = (*texture).resource.device;
    dc_info = (*texture).dc_info.offset(sub_resource_idx as isize) as *mut wined3d_dc_info;

    if (*dc_info).dc.is_null() {
        ERR!("Sub-resource {%p, %u} has no DC.\n", texture, sub_resource_idx);
        return;
    }

    destroy_desc.hDc = (*dc_info).dc;
    destroy_desc.hBitmap = (*dc_info).bitmap as HANDLE;
    status = D3DKMTDestroyDCFromMemory(&raw mut destroy_desc) as NTSTATUS;
    if status != 0 {
        ERR!("Failed to destroy dc, status %#lx.\n", status);
    }
    (*dc_info).dc = core::ptr::null_mut();
    (*dc_info).bitmap = core::ptr::null_mut();

    wined3d_texture_get_bo_address(
        texture,
        sub_resource_idx,
        &raw mut data,
        (*texture).resource.map_binding as uint32_t,
    );
    if !data.buffer_object.is_null() {
        context = context_acquire(device, core::ptr::null_mut(), 0);
        range.offset = 0;
        range.size = (*(*texture).sub_resources.offset(sub_resource_idx as isize)).size;
        wined3d_context_unmap_bo_address(context, &raw mut data, 1, &raw mut range);
        context_release(context);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_set_swapchain(
    texture: *mut wined3d_texture,
    swapchain: *mut wined3d_swapchain,
) {
    (*texture).swapchain = swapchain;
    wined3d_resource_update_draw_binding(core::ptr::addr_of_mut!((*texture).resource));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_gl_texture_swizzle_from_color_fixup(swizzle: *mut GLint, fixup: color_fixup_desc) {
    static swizzle_source: [GLenum; 6] = [
        GL_ZERO as GLenum, /* CHANNEL_SOURCE_ZERO */
        GL_ONE as GLenum, /* CHANNEL_SOURCE_ONE */
        GL_RED as GLenum, /* CHANNEL_SOURCE_X */
        GL_GREEN as GLenum, /* CHANNEL_SOURCE_Y */
        GL_BLUE as GLenum, /* CHANNEL_SOURCE_Z */
        GL_ALPHA as GLenum, /* CHANNEL_SOURCE_W */
    ];

    *swizzle.offset(0 as isize) = swizzle_source[fixup.x_source() as usize] as GLint;
    *swizzle.offset(1 as isize) = swizzle_source[fixup.y_source() as usize] as GLint;
    *swizzle.offset(2 as isize) = swizzle_source[fixup.z_source() as usize] as GLint;
    *swizzle.offset(3 as isize) = swizzle_source[fixup.w_source() as usize] as GLint;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_gl_bind(texture_gl: *mut wined3d_texture_gl, context_gl: *mut wined3d_context_gl, mut srgb: BOOL) {
    let format: *const wined3d_format = (*texture_gl).t.resource.format;
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let fixup: color_fixup_desc = (*format).color_fixup;
    let mut gl_tex: *mut gl_texture = core::ptr::null_mut();
    let mut target: GLenum;

    if needs_separate_srgb_gl_texture(core::ptr::addr_of_mut!((*context_gl).c), core::ptr::addr_of_mut!((*texture_gl).t)) == 0 {
        srgb = (FALSE as i32);
    }

    /* sRGB mode cache for preload() calls outside drawprim. */
    if srgb != 0 {
        (*texture_gl).t.flags |= WINED3D_TEXTURE_IS_SRGB;
    } else {
        (*texture_gl).t.flags &= !WINED3D_TEXTURE_IS_SRGB;
    }

    gl_tex = wined3d_texture_gl_get_gl_texture(texture_gl, srgb != 0);
    target = (*texture_gl).target;

    if (*gl_tex).name != 0 {
        wined3d_context_gl_bind_texture(context_gl, target, (*gl_tex).name);
        return;
    }

    (*gl_info).gl_ops.gl.p_glGenTextures.unwrap_unchecked()(
        1,
        core::ptr::addr_of_mut!((*gl_tex).name),
    );
    checkGLcall!(gl_info, "glGenTextures");

    if (*gl_tex).name == 0 {
        ERR!("Failed to generate a texture name.\n");
        return;
    }

    /* Initialise the state of the texture object to the OpenGL defaults, not
     * the wined3d defaults. */
    (*gl_tex).sampler_desc.address_u = WINED3D_TADDRESS_WRAP;
    (*gl_tex).sampler_desc.address_v = WINED3D_TADDRESS_WRAP;
    (*gl_tex).sampler_desc.address_w = WINED3D_TADDRESS_WRAP;
    memset(
        core::ptr::addr_of_mut!((*gl_tex).sampler_desc.border_color) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<[core::ffi::c_float; 4]>(),
    );
    (*gl_tex).sampler_desc.mag_filter = WINED3D_TEXF_LINEAR;
    (*gl_tex).sampler_desc.min_filter = WINED3D_TEXF_POINT; /* GL_NEAREST_MIPMAP_LINEAR */
    (*gl_tex).sampler_desc.mip_filter = WINED3D_TEXF_LINEAR; /* GL_NEAREST_MIPMAP_LINEAR */
    (*gl_tex).sampler_desc.lod_bias = 0.0f32;
    (*gl_tex).sampler_desc.min_lod = -1000.0f32;
    (*gl_tex).sampler_desc.max_lod = 1000.0f32;
    (*gl_tex).sampler_desc.max_anisotropy = 1;
    (*gl_tex).sampler_desc.compare = (FALSE as i32);
    (*gl_tex).sampler_desc.comparison_func = WINED3D_CMP_LESSEQUAL;
    if (*gl_info).supported[EXT_TEXTURE_SRGB_DECODE as usize] != 0 {
        (*gl_tex).sampler_desc.srgb_decode = (TRUE as i32);
    } else {
        (*gl_tex).sampler_desc.srgb_decode = srgb;
    }
    (*gl_tex).sampler_desc.mip_base_level = 0;
    wined3d_texture_set_dirty(core::ptr::addr_of_mut!((*texture_gl).t));

    wined3d_context_gl_bind_texture(context_gl, target, (*gl_tex).name);

    /* For a new texture we have to set the texture levels after binding the
     * texture. */
    (*gl_info).gl_ops.gl.p_glTexParameteri.unwrap_unchecked()(
        target,
        GL_TEXTURE_MAX_LEVEL,
        (*texture_gl).t.level_count.wrapping_sub(1) as GLint,
    );
    checkGLcall!(gl_info, "glTexParameteri(target, GL_TEXTURE_MAX_LEVEL, texture->level_count)");

    if target == GL_TEXTURE_CUBE_MAP_ARB {
        /* Cubemaps are always set to clamp, regardless of the sampler state. */
        (*gl_info).gl_ops.gl.p_glTexParameteri.unwrap_unchecked()(
            target,
            GL_TEXTURE_WRAP_S,
            (GL_CLAMP_TO_EDGE as i32),
        );
        (*gl_info).gl_ops.gl.p_glTexParameteri.unwrap_unchecked()(
            target,
            GL_TEXTURE_WRAP_T,
            (GL_CLAMP_TO_EDGE as i32),
        );
        (*gl_info).gl_ops.gl.p_glTexParameteri.unwrap_unchecked()(
            target,
            GL_TEXTURE_WRAP_R,
            (GL_CLAMP_TO_EDGE as i32),
        );
    }

    if (*texture_gl).t.flags & WINED3D_TEXTURE_COND_NP2 != 0
        && target != GL_TEXTURE_2D_MULTISAMPLE
        && target != GL_TEXTURE_2D_MULTISAMPLE_ARRAY
    {
        /* Conditional non power of two textures use a different clamping
         * default. If we're using the GL_WINE_normalized_texrect partial
         * driver emulation, we're dealing with a GL_TEXTURE_2D texture which
         * has the address mode set to repeat - something that prevents us
         * from hitting the accelerated codepath. Thus manually set the GL
         * state. The same applies to filtering. Even if the texture has only
         * one mip level, the default LINEAR_MIPMAP_LINEAR filter causes a SW
         * fallback on macos. */
        (*gl_info).gl_ops.gl.p_glTexParameteri.unwrap_unchecked()(
            target,
            GL_TEXTURE_WRAP_S,
            (GL_CLAMP_TO_EDGE as i32),
        );
        (*gl_info).gl_ops.gl.p_glTexParameteri.unwrap_unchecked()(
            target,
            GL_TEXTURE_WRAP_T,
            (GL_CLAMP_TO_EDGE as i32),
        );
        (*gl_info).gl_ops.gl.p_glTexParameteri.unwrap_unchecked()(
            target,
            GL_TEXTURE_MIN_FILTER,
            (GL_NEAREST as i32),
        );
        (*gl_info).gl_ops.gl.p_glTexParameteri.unwrap_unchecked()(
            target,
            GL_TEXTURE_MAG_FILTER,
            (GL_NEAREST as i32),
        );
        checkGLcall!(gl_info, "glTexParameteri");
        (*gl_tex).sampler_desc.address_u = WINED3D_TADDRESS_CLAMP;
        (*gl_tex).sampler_desc.address_v = WINED3D_TADDRESS_CLAMP;
        (*gl_tex).sampler_desc.mag_filter = WINED3D_TEXF_POINT;
        (*gl_tex).sampler_desc.min_filter = WINED3D_TEXF_POINT;
        (*gl_tex).sampler_desc.mip_filter = WINED3D_TEXF_NONE;
    }

    if (*gl_info).supported[WINED3D_GL_LEGACY_CONTEXT as usize] != 0
        && (*gl_info).supported[ARB_DEPTH_TEXTURE as usize] != 0
    {
        (*gl_info).gl_ops.gl.p_glTexParameteri.unwrap_unchecked()(
            target,
            GL_DEPTH_TEXTURE_MODE_ARB,
            (GL_INTENSITY as i32),
        );
        checkGLcall!(gl_info, "glTexParameteri(GL_DEPTH_TEXTURE_MODE_ARB, GL_INTENSITY)");
    }

    if is_identity_fixup(fixup) == 0
        && can_use_texture_swizzle((*context_gl).c.d3d_info, format) != 0
    {
        let mut swizzle: [GLint; 4] = [0; 4];

        wined3d_gl_texture_swizzle_from_color_fixup(core::ptr::addr_of_mut!(swizzle) as *mut GLint, fixup);
        (*gl_info).gl_ops.gl.p_glTexParameteriv.unwrap_unchecked()(
            target,
            GL_TEXTURE_SWIZZLE_RGBA,
            core::ptr::addr_of_mut!(swizzle) as *mut GLint,
        );
        checkGLcall!(gl_info, "set format swizzle");
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_gl_bind_and_dirtify(texture_gl: *mut wined3d_texture_gl, context_gl: *mut wined3d_context_gl, srgb: BOOL) {
    /* FIXME: Ideally we'd only do this when touching a binding that's used by
     * a shader. */
    context_invalidate_compute_state(core::ptr::addr_of_mut!((*context_gl).c), (STATE_COMPUTE_SHADER_RESOURCE_BINDING) as DWORD);
    context_invalidate_state(core::ptr::addr_of_mut!((*context_gl).c), (STATE_GRAPHICS_SHADER_RESOURCE_BINDING as u32));

    wined3d_texture_gl_bind(texture_gl, context_gl, srgb);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_gl_apply_sampler_desc(
    mut texture_gl: *mut wined3d_texture_gl,
    mut sampler_desc: *const wined3d_sampler_desc,
    mut context_gl: *const wined3d_context_gl,
) {
    let mut gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut target: GLenum = (*texture_gl).target;
    let mut gl_tex: *mut gl_texture = core::ptr::null_mut::<gl_texture>();
    let mut state: DWORD = 0;

    gl_tex = wined3d_texture_gl_get_gl_texture(
        texture_gl,
        ((*texture_gl).t.flags & WINED3D_TEXTURE_IS_SRGB) != 0,
    );

    state = (*sampler_desc).address_u as DWORD;
    if state != (*gl_tex).sampler_desc.address_u as u32 {
        (*gl_info)
            .gl_ops
            .gl
            .p_glTexParameteri
            .unwrap_unchecked()(
            target,
            GL_TEXTURE_WRAP_S as GLenum,
            (*gl_info).wrap_lookup
                [state.wrapping_sub(WINED3D_TADDRESS_WRAP as DWORD) as usize],
        );
        (*gl_tex).sampler_desc.address_u = state as wined3d_texture_address;
    }

    state = (*sampler_desc).address_v as DWORD;
    if state != (*gl_tex).sampler_desc.address_v as u32 {
        (*gl_info)
            .gl_ops
            .gl
            .p_glTexParameteri
            .unwrap_unchecked()(
            target,
            GL_TEXTURE_WRAP_T as GLenum,
            (*gl_info).wrap_lookup
                [state.wrapping_sub(WINED3D_TADDRESS_WRAP as DWORD) as usize],
        );
        (*gl_tex).sampler_desc.address_v = state as wined3d_texture_address;
    }

    state = (*sampler_desc).address_w as DWORD;
    if state != (*gl_tex).sampler_desc.address_w as u32 {
        (*gl_info)
            .gl_ops
            .gl
            .p_glTexParameteri
            .unwrap_unchecked()(
            target,
            GL_TEXTURE_WRAP_R as GLenum,
            (*gl_info).wrap_lookup
                [state.wrapping_sub(WINED3D_TADDRESS_WRAP as DWORD) as usize],
        );
        (*gl_tex).sampler_desc.address_w = state as wined3d_texture_address;
    }

    if memcmp(
        core::ptr::addr_of_mut!((*gl_tex).sampler_desc.border_color) as *const core::ffi::c_void,
        core::ptr::addr_of!((*sampler_desc).border_color) as *const core::ffi::c_void,
        core::mem::size_of::<[core::ffi::c_float; 4]>() as size_t,
    ) != 0
    {
        (*gl_info)
            .gl_ops
            .gl
            .p_glTexParameterfv
            .unwrap_unchecked()(
            target,
            GL_TEXTURE_BORDER_COLOR as GLenum,
            (core::ptr::addr_of!((*sampler_desc).border_color) as *const core::ffi::c_float)
                .offset(0 as isize) as *const GLfloat,
        );
        memcpy(
            core::ptr::addr_of_mut!((*gl_tex).sampler_desc.border_color) as *mut core::ffi::c_void,
            core::ptr::addr_of!((*sampler_desc).border_color) as *const core::ffi::c_void,
            core::mem::size_of::<[core::ffi::c_float; 4]>() as size_t,
        );
    }

    state = (*sampler_desc).mag_filter as DWORD;
    if state != (*gl_tex).sampler_desc.mag_filter as u32 {
        (*gl_info)
            .gl_ops
            .gl
            .p_glTexParameteri
            .unwrap_unchecked()(
            target,
            GL_TEXTURE_MAG_FILTER as GLenum,
            wined3d_gl_mag_filter(state as wined3d_texture_filter_type) as GLint,
        );
        (*gl_tex).sampler_desc.mag_filter = state as wined3d_texture_filter_type;
    }

    if (*sampler_desc).min_filter as u32 != (*gl_tex).sampler_desc.min_filter as u32
        || (*sampler_desc).mip_filter as u32 != (*gl_tex).sampler_desc.mip_filter as u32
    {
        (*gl_info)
            .gl_ops
            .gl
            .p_glTexParameteri
            .unwrap_unchecked()(
            target,
            GL_TEXTURE_MIN_FILTER as GLenum,
            wined3d_gl_min_mip_filter((*sampler_desc).min_filter, (*sampler_desc).mip_filter) as GLint,
        );
        (*gl_tex).sampler_desc.min_filter = (*sampler_desc).min_filter;
        (*gl_tex).sampler_desc.mip_filter = (*sampler_desc).mip_filter;
    }

    state = (*sampler_desc).max_anisotropy as DWORD;
    if state != (*gl_tex).sampler_desc.max_anisotropy {
        if (*gl_info).supported[ARB_TEXTURE_FILTER_ANISOTROPIC as usize] != 0 {
            (*gl_info)
                .gl_ops
                .gl
                .p_glTexParameteri
                .unwrap_unchecked()(
                target,
                GL_TEXTURE_MAX_ANISOTROPY as GLenum,
                state as GLint,
            );
        } else {
            WARN!("Anisotropic filtering not supported.\n");
        }
        (*gl_tex).sampler_desc.max_anisotropy = state as u32;
    }

    if ((*sampler_desc).srgb_decode == 0) as i32 != ((*gl_tex).sampler_desc.srgb_decode == 0) as i32
        && ((*(*context_gl).c.d3d_info).wined3d_creation_flags & WINED3D_SRGB_READ_WRITE_CONTROL) != 0
        && (*gl_info).supported[EXT_TEXTURE_SRGB_DECODE as usize] != 0
    {
        (*gl_info)
            .gl_ops
            .gl
            .p_glTexParameteri
            .unwrap_unchecked()(
            target,
            GL_TEXTURE_SRGB_DECODE_EXT as GLenum,
            if (*sampler_desc).srgb_decode != 0 {
                (GL_DECODE_EXT as i32)
            } else {
                (GL_SKIP_DECODE_EXT as i32)
            },
        );
        (*gl_tex).sampler_desc.srgb_decode = (*sampler_desc).srgb_decode;
    }

    if ((*sampler_desc).compare == 0) as i32 != ((*gl_tex).sampler_desc.compare == 0) as i32 {
        if (*sampler_desc).compare != 0 {
            (*gl_info)
                .gl_ops
                .gl
                .p_glTexParameteri
                .unwrap_unchecked()(
                target,
                GL_TEXTURE_COMPARE_MODE_ARB as GLenum,
                (GL_COMPARE_R_TO_TEXTURE_ARB as i32),
            );
        } else {
            (*gl_info)
                .gl_ops
                .gl
                .p_glTexParameteri
                .unwrap_unchecked()(
                target,
                GL_TEXTURE_COMPARE_MODE_ARB as GLenum,
                (GL_NONE as i32),
            );
        }
        (*gl_tex).sampler_desc.compare = (*sampler_desc).compare;
    }

    checkGLcall!(gl_info, "Texture parameter application");

    if (*gl_info).supported[EXT_TEXTURE_LOD_BIAS as usize] != 0 {
        (*gl_info)
            .gl_ops
            .gl
            .p_glTexEnvf
            .unwrap_unchecked()(
            GL_TEXTURE_FILTER_CONTROL_EXT as GLenum,
            GL_TEXTURE_LOD_BIAS_EXT as GLenum,
            (*sampler_desc).lod_bias as GLfloat,
        );
        checkGLcall!(gl_info, "glTexEnvf(GL_TEXTURE_LOD_BIAS_EXT, ...)");
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_incref(mut texture: *mut wined3d_texture) -> ULONG {
    let mut refcount: ::core::ffi::c_uint = 0;
    // TRACE(...) debug-channel calls from the anchor are elided: the debug channel
    // globals and ros_dbg_log are not part of this crate's import set.
    refcount = InterlockedIncrement(
        &raw mut (*texture).resource.ref_ as *mut ::core::ffi::c_long as *mut ::core::ffi::c_long,
    ) as ::core::ffi::c_uint;
    // TRACE(...) debug-channel call elided (see above).
    return refcount as ULONG;
}

unsafe extern "C" fn wined3d_texture_destroy_object(mut object: *mut ::core::ffi::c_void) {
    let mut texture: *mut wined3d_texture = object as *mut wined3d_texture;
    let mut resource: *mut wined3d_resource = ::core::ptr::null_mut::<wined3d_resource>();
    let mut dc_info: *mut wined3d_dc_info = ::core::ptr::null_mut::<wined3d_dc_info>();
    let mut sub_count: ::core::ffi::c_uint = 0;
    let mut i: ::core::ffi::c_uint = 0;
    WINE_TRACE!("texture %p.\n", texture);
    resource = &raw mut (*texture).resource;
    sub_count = ((*texture).level_count as UINT).wrapping_mul((*texture).layer_count)
        as ::core::ffi::c_uint;
    dc_info = (*texture).dc_info as *mut wined3d_dc_info;
    if !dc_info.is_null() {
        i = 0 as ::core::ffi::c_uint;
        while i < sub_count {
            if !(*dc_info.offset(i as isize)).dc.is_null() {
                let mut texture_idx: wined3d_texture_idx = wined3d_texture_idx {
                    texture: texture,
                    sub_resource_idx: i,
                };
                wined3d_texture_destroy_dc(&raw mut texture_idx as *mut ::core::ffi::c_void);
            }
            i = i.wrapping_add(1);
        }
        free(dc_info as *mut ::core::ffi::c_void);
    }
    if !(*texture).overlay_info.is_null() {
        i = 0 as ::core::ffi::c_uint;
        while i < sub_count {
            let mut info: *mut wined3d_overlay_info =
                (*texture).overlay_info.offset(i as isize) as *mut wined3d_overlay_info;
            let mut overlay: *mut wined3d_overlay_info =
                ::core::ptr::null_mut::<wined3d_overlay_info>();
            let mut cur: *mut wined3d_overlay_info =
                ::core::ptr::null_mut::<wined3d_overlay_info>();
            list_remove(&raw mut (*info).entry);
            overlay = ((*info).overlays.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(wined3d_overlay_info, entry)
                    as ::core::ffi::c_ulonglong as isize),
            ) as *mut wined3d_overlay_info;
            cur = ((*overlay).entry.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(wined3d_overlay_info, entry)
                    as ::core::ffi::c_ulonglong as isize),
            ) as *mut wined3d_overlay_info;
            while &raw mut (*overlay).entry != &raw mut (*info).overlays {
                list_remove(&raw mut (*overlay).entry);
                overlay = cur;
                cur = ((*overlay).entry.next as *mut ::core::ffi::c_char).offset(
                    -(core::mem::offset_of!(wined3d_overlay_info, entry)
                        as ::core::ffi::c_ulonglong as isize),
                ) as *mut wined3d_overlay_info;
            }
            i = i.wrapping_add(1);
        }
        free((*texture).overlay_info as *mut ::core::ffi::c_void);
    }
    if !(*texture).dirty_regions.is_null() {
        i = 0 as ::core::ffi::c_uint;
        while i < (*texture).layer_count {
            free(
                (*(*texture).dirty_regions.offset(i as isize)).boxes as *mut ::core::ffi::c_void,
            );
            i = i.wrapping_add(1);
        }
        free((*texture).dirty_regions as *mut ::core::ffi::c_void);
    }
    if (*resource).access & WINED3D_RESOURCE_ACCESS_CPU != 0 {
        i = 0 as ::core::ffi::c_uint;
        while i < sub_count {
            wined3d_texture_validate_location(texture, i, WINED3D_LOCATION_DISCARDED as uint32_t);
            wined3d_texture_invalidate_location(texture, i, !(WINED3D_LOCATION_DISCARDED) as uint32_t);
            i = i.wrapping_add(1);
        }
    }
    (*(*resource).resource_ops)
        .resource_unload
        .unwrap_unchecked()(resource);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_cleanup(texture: *mut wined3d_texture) {
    wined3d_cs_destroy_object(
        (*(*texture).resource.device).cs as *mut wined3d_cs,
        Some(wined3d_texture_destroy_object as unsafe extern "C" fn(*mut core::ffi::c_void)),
        texture as *mut core::ffi::c_void,
    );
    resource_cleanup(core::ptr::addr_of_mut!((*texture).resource));
}

unsafe extern "C" fn wined3d_texture_cleanup_sync(texture: *mut wined3d_texture) {
    wined3d_texture_sub_resources_destroyed(texture);
    wined3d_texture_cleanup(texture);
    wined3d_resource_wait_idle(core::ptr::addr_of_mut!((*texture).resource));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_decref(texture: *mut wined3d_texture) -> ULONG {
    let mut i: u32;
    let mut sub_resource_count: u32;
    let mut refcount: u32;

    refcount = InterlockedDecrement(core::ptr::addr_of_mut!((*texture).resource.ref_)) as u32;

    if refcount == 0 {
        let in_cs_thread: bool =
            GetCurrentThreadId() == (*(*(*texture).resource.device).cs).thread_id;

        if !(*texture).identity_srv.is_null() {
            assert!(!in_cs_thread);
            wined3d_shader_resource_view_destroy((*texture).identity_srv);
        }

        /* This is called from the CS thread to destroy temporary textures. */
        if !in_cs_thread {
            wined3d_mutex_lock();
        }
        /* Wait for the texture to become idle if it's using user memory,
         * since the application is allowed to free that memory once the
         * texture is destroyed. Note that this implies that
         * the destroy handler can't access that memory either. */
        sub_resource_count = ((*texture).layer_count as u32).wrapping_mul((*texture).level_count);
        i = 0;
        while i < sub_resource_count {
            if !(*(*texture).sub_resources.offset(i as isize))
                .user_memory
                .is_null()
            {
                wined3d_resource_wait_idle(core::ptr::addr_of_mut!((*texture).resource));
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
        ((*(*(*(*texture).resource.device).adapter).adapter_ops).adapter_destroy_texture)
            .unwrap_unchecked()(texture);
        if !in_cs_thread {
            wined3d_mutex_unlock();
        }
    }

    return refcount as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_get_resource(texture: *mut wined3d_texture) -> *mut wined3d_resource {
    // TRACE in the anchor is a debug-channel log; the C source here has no such statement.
    core::ptr::addr_of_mut!((*texture).resource)
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_load(texture: *mut wined3d_texture, context: *mut wined3d_context, mut srgb: BOOL) {
    let mut sub_count: UINT = ((*texture).level_count as UINT).wrapping_mul((*texture).layer_count);
    let mut flag: DWORD = 0;
    let mut i: UINT = 0;

    if needs_separate_srgb_gl_texture(context, texture) == 0 {
        srgb = (FALSE as i32);
    }

    if srgb != 0 {
        flag = WINED3D_TEXTURE_SRGB_VALID;
    } else {
        flag = WINED3D_TEXTURE_RGB_VALID;
    }

    if (*texture).flags & flag as uint32_t != 0 {
        return;
    }

    /* Reload the surfaces if the texture is marked dirty. */
    i = 0;
    while i < sub_count {
        if wined3d_texture_load_location(
            texture,
            i,
            context,
            if srgb != 0 {
                WINED3D_LOCATION_TEXTURE_SRGB
            } else {
                WINED3D_LOCATION_TEXTURE_RGB
            },
        ) == 0
        {
            ERR!("Failed to load location (srgb %#x).\n", srgb);
        }
        i = i.wrapping_add(1);
    }
    (*texture).flags |= flag as uint32_t;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_get_parent(
    texture: *const wined3d_texture,
) -> *mut core::ffi::c_void {
    TRACE!("texture %p.\n", texture);
    return (*texture).resource.parent;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_get_pitch(
    texture: *const wined3d_texture,
    level: ::core::ffi::c_uint,
    row_pitch: *mut ::core::ffi::c_uint,
    slice_pitch: *mut ::core::ffi::c_uint,
) {
    let resource: *const wined3d_resource = core::ptr::addr_of!((*texture).resource);
    let width: ::core::ffi::c_uint = wined3d_texture_get_level_width(texture, level);
    let height: ::core::ffi::c_uint = wined3d_texture_get_level_height(texture, level);

    if (*texture).row_pitch != 0 {
        *row_pitch = (*texture).row_pitch;
        *slice_pitch = (*texture).slice_pitch;
        return;
    }

    wined3d_format_calculate_pitch(
        (*resource).format,
        (*(*resource).device).surface_alignment as ::core::ffi::c_uint,
        width,
        height,
        row_pitch,
        slice_pitch,
    );
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_get_lod(texture: *const wined3d_texture) -> u32 {
    // TRACE("texture %p, returning %u.\n", texture, texture->lod);
    (*texture).lod
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_set_lod(texture: *mut wined3d_texture, lod: u32) -> u32 {
    let mut lod: u32 = lod;
    let old: u32 = (*texture).lod;
    let resource: *mut wined3d_resource = core::ptr::addr_of_mut!((*texture).resource);
    if ((*resource).usage & WINED3DUSAGE_MANAGED) == 0 {
        return 0;
    }
    if lod >= (*texture).level_count {
        lod = (*texture).level_count.wrapping_sub(1);
    }
    (*texture).lod = lod;
    return old;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_get_level_count(
    texture: *const wined3d_texture,
) -> UINT {
    // TRACE("texture %p, returning %u.\n", texture, texture->level_count);
    (*texture).level_count
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_set_color_key(
    texture: *mut wined3d_texture,
    flags: uint32_t,
    color_key: *const wined3d_color_key,
) -> HRESULT {
    let device: *mut wined3d_device = (*texture).resource.device;
    let all_flags: DWORD = WINED3D_CKEY_DST_BLT | WINED3D_CKEY_DST_OVERLAY
        | WINED3D_CKEY_SRC_BLT | WINED3D_CKEY_SRC_OVERLAY;

    if flags & !all_flags != 0 {
        WARN!("Invalid flags passed, returning WINED3DERR_INVALIDCALL.\n");
        return WINED3DERR_INVALIDCALL;
    }

    if !color_key.is_null() {
        if (flags & WINED3D_CKEY_SRC_BLT) != 0 {
            (*texture).src_blt_color_key = *color_key;
        }
        (*texture).color_key_flags |= flags;
    } else {
        (*texture).color_key_flags &= !flags;
    }

    wined3d_cs_emit_set_color_key(
        (*device).cs as *mut wined3d_cs,
        texture,
        flags as WORD,
        color_key,
    );

    return WINED3D_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_gl_set_compatible_renderbuffer(
    texture_gl: *mut wined3d_texture_gl,
    context_gl: *mut wined3d_context_gl,
    level: core::ffi::c_uint,
    rt: *const wined3d_rendertarget_info,
) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut entry: *mut wined3d_renderbuffer_entry = core::ptr::null_mut();
    let mut src_width: core::ffi::c_uint = 0;
    let mut src_height: core::ffi::c_uint = 0;
    let mut width: core::ffi::c_uint = 0;
    let mut height: core::ffi::c_uint = 0;
    let mut renderbuffer: GLuint = 0;

    if (*gl_info).supported[ARB_FRAMEBUFFER_OBJECT as usize] != 0 {
        return;
    }

    if !rt.is_null()
        && (*(*(*rt).resource).format).id as core::ffi::c_uint
            != WINED3DFMT_NULL as core::ffi::c_uint
    {
        let mut rt_texture: *mut wined3d_texture = core::ptr::null_mut();
        let mut rt_level: core::ffi::c_uint = 0;

        if (*(*rt).resource).type_ as core::ffi::c_uint
            == WINED3D_RTYPE_BUFFER as core::ffi::c_uint
        {
            FIXME!(
                "Unsupported resource type %s.\n",
                debug_d3dresourcetype((*(*rt).resource).type_)
            );
            return;
        }
        rt_texture = wined3d_texture_from_resource((*rt).resource);
        rt_level = (*rt).sub_resource_idx % (*rt_texture).level_count;

        width = wined3d_texture_get_level_width(rt_texture, rt_level);
        height = wined3d_texture_get_level_height(rt_texture, rt_level);
    } else {
        width = wined3d_texture_get_level_width(core::ptr::addr_of_mut!((*texture_gl).t), level);
        height = wined3d_texture_get_level_height(core::ptr::addr_of_mut!((*texture_gl).t), level);
    }

    src_width = wined3d_texture_get_level_width(core::ptr::addr_of_mut!((*texture_gl).t), level);
    src_height = wined3d_texture_get_level_height(core::ptr::addr_of_mut!((*texture_gl).t), level);

    /* A depth stencil smaller than the render target is not valid */
    if width > src_width || height > src_height {
        return;
    }

    /* Remove any renderbuffer set if the sizes match */
    if width == src_width && height == src_height {
        (*texture_gl).current_renderbuffer = core::ptr::null_mut();
        return;
    }

    /* Look if we've already got a renderbuffer of the correct dimensions */
    entry = ((*texture_gl).renderbuffers.next as *mut u8).offset(
        -(core::mem::offset_of!(wined3d_renderbuffer_entry, entry) as isize),
    ) as *mut wined3d_renderbuffer_entry;
    while core::ptr::addr_of_mut!((*entry).entry) != core::ptr::addr_of_mut!((*texture_gl).renderbuffers) {
        if (*entry).width == width && (*entry).height == height {
            renderbuffer = (*entry).id;
            (*texture_gl).current_renderbuffer = entry;
            break;
        } else {
            entry = ((*entry).entry.next as *mut u8).offset(
                -(core::mem::offset_of!(wined3d_renderbuffer_entry, entry) as isize),
            ) as *mut wined3d_renderbuffer_entry;
        }
    }

    if renderbuffer == 0 {
        let mut format_gl: *const wined3d_format_gl = core::ptr::null();

        format_gl = wined3d_format_gl((*texture_gl).t.resource.format);
        (*gl_info).fbo_ops.glGenRenderbuffers.unwrap_unchecked()(
            1 as GLsizei,
            core::ptr::addr_of_mut!(renderbuffer),
        );
        (*gl_info).fbo_ops.glBindRenderbuffer.unwrap_unchecked()(
            GL_RENDERBUFFER as GLenum,
            renderbuffer,
        );
        (*gl_info).fbo_ops.glRenderbufferStorage.unwrap_unchecked()(
            GL_RENDERBUFFER as GLenum,
            (*format_gl).internal as GLenum,
            width as GLsizei,
            height as GLsizei,
        );

        entry = malloc(core::mem::size_of::<wined3d_renderbuffer_entry>() as size_t)
            as *mut wined3d_renderbuffer_entry;
        (*entry).width = width;
        (*entry).height = height;
        (*entry).id = renderbuffer;
        list_add_head(
            core::ptr::addr_of_mut!((*texture_gl).renderbuffers),
            core::ptr::addr_of_mut!((*entry).entry),
        );

        (*texture_gl).current_renderbuffer = entry;
    }

    checkGLcall!(gl_info, b"set compatible renderbuffer\0".as_ptr() as *const core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_update_desc(
    texture: *mut wined3d_texture,
    sub_resource_idx: ::core::ffi::c_uint,
    mem: *mut ::core::ffi::c_void,
    mut pitch: ::core::ffi::c_uint,
) -> HRESULT {
    let mut current_row_pitch: ::core::ffi::c_uint = 0;
    let mut current_slice_pitch: ::core::ffi::c_uint = 0;
    let mut width: ::core::ffi::c_uint = 0;
    let mut height: ::core::ffi::c_uint = 0;
    let mut sub_resource: *mut wined3d_texture_sub_resource =
        ::core::ptr::null_mut::<wined3d_texture_sub_resource>();
    let mut i: ::core::ffi::c_uint = 0;
    let mut level: ::core::ffi::c_uint = 0;
    let mut sub_resource_count: ::core::ffi::c_uint = 0;
    let mut format: *const wined3d_format = ::core::ptr::null::<wined3d_format>();
    let mut device: *mut wined3d_device = ::core::ptr::null_mut::<wined3d_device>();
    let mut slice_pitch: ::core::ffi::c_uint = 0;
    let mut update_memory_only: bool = false;
    let mut create_dib: bool = false;

    device = (*texture).resource.device;
    format = (*texture).resource.format;
    level = sub_resource_idx % (*texture).level_count;
    sub_resource_count = ((*texture).level_count as UINT).wrapping_mul((*texture).layer_count)
        as ::core::ffi::c_uint;

    width = wined3d_texture_get_level_width(texture, level);
    height = wined3d_texture_get_level_height(texture, level);
    if pitch != 0 {
        slice_pitch = height.wrapping_mul(pitch);
    } else {
        wined3d_format_calculate_pitch(
            format,
            1,
            width,
            height,
            core::ptr::addr_of_mut!(pitch),
            core::ptr::addr_of_mut!(slice_pitch),
        );
    }

    wined3d_texture_get_pitch(
        texture,
        level,
        core::ptr::addr_of_mut!(current_row_pitch),
        core::ptr::addr_of_mut!(current_slice_pitch),
    );
    update_memory_only = pitch == current_row_pitch && slice_pitch == current_slice_pitch;

    if sub_resource_count > 1 && !update_memory_only {
        return WINED3DERR_INVALIDCALL;
    }

    if (*texture).resource.type_ as ::core::ffi::c_uint != WINED3D_RTYPE_TEXTURE_2D {
        WARN!(
            "Not supported on %s.\n",
            debug_d3dresourcetype((*texture).resource.type_)
        );
        return WINED3DERR_INVALIDCALL;
    }

    if (*texture).resource.map_count != 0 {
        WARN!("Texture is mapped.\n");
        return WINED3DERR_INVALIDCALL;
    }

    /* We have no way of supporting a pitch that is not a multiple of the pixel
     * byte width short of uploading the texture row-by-row.
     * Fortunately that's not an issue since D3D9Ex doesn't allow a custom pitch
     * for user-memory textures (it always expects packed data) while DirectDraw
     * requires a 4-byte aligned pitch and doesn't support texture formats
     * larger than 4 bytes per pixel nor any format using 3 bytes per pixel.
     * This check is here to verify that the assumption holds. */
    if (pitch as UINT).wrapping_rem((*format).byte_count) != 0 {
        WARN!("Pitch unsupported, not a multiple of the texture format byte width.\n");
        return WINED3DERR_INVALIDCALL;
    }

    if (*device).d3d_initialized() != 0 {
        wined3d_cs_emit_unload_resource(
            (*device).cs as *mut wined3d_cs,
            core::ptr::addr_of_mut!((*texture).resource),
        );
    }
    wined3d_resource_wait_idle(core::ptr::addr_of_mut!((*texture).resource));

    if !(*texture).dc_info.is_null()
        && !(*(*texture).dc_info.offset(0 as isize))
            .dc
            .is_null()
    {
        let mut texture_idx: wined3d_texture_idx = wined3d_texture_idx {
            texture: texture,
            sub_resource_idx: sub_resource_idx,
        };

        wined3d_cs_destroy_object(
            (*device).cs as *mut wined3d_cs,
            Some(
                wined3d_texture_destroy_dc as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
            ),
            core::ptr::addr_of_mut!(texture_idx) as *mut ::core::ffi::c_void,
        );
        wined3d_cs_finish((*device).cs as *mut wined3d_cs, WINED3D_CS_QUEUE_DEFAULT);
        create_dib = true;
    }

    (*(*texture).sub_resources.offset(sub_resource_idx as isize)).user_memory = mem;

    if update_memory_only {
        i = 0;
        while i < sub_resource_count {
            if (*(*texture).sub_resources.offset(i as isize))
                .user_memory
                .is_null()
            {
                break;
            }
            i = i.wrapping_add(1);
        }

        if i == sub_resource_count {
            wined3d_resource_free_sysmem(core::ptr::addr_of_mut!((*texture).resource));
        }
    } else {
        wined3d_resource_free_sysmem(core::ptr::addr_of_mut!((*texture).resource));

        sub_resource = (*texture).sub_resources.offset(sub_resource_idx as isize)
            as *mut wined3d_texture_sub_resource;

        (*texture).row_pitch = pitch;
        (*texture).slice_pitch = slice_pitch;

        if ((*texture).resource.access & WINED3D_RESOURCE_ACCESS_CPU) == 0
            && ((*texture).resource.usage & (WINED3DUSAGE_VIDMEM_ACCOUNTING as uint32_t)) != 0
        {
            adapter_adjust_memory(
                (*device).adapter,
                (*texture).slice_pitch as INT64 - (*texture).resource.size as INT64,
            );
        }
        (*texture).resource.size = (*texture).slice_pitch as UINT;
        (*sub_resource).size = (*texture).slice_pitch;
        (*sub_resource).locations = WINED3D_LOCATION_DISCARDED as DWORD;
    }

    if mem.is_null()
        && wined3d_resource_prepare_sysmem(core::ptr::addr_of_mut!((*texture).resource)) == 0
    {
        ERR!("Failed to allocate resource memory.\n");
    }

    wined3d_texture_validate_location(texture, sub_resource_idx, WINED3D_LOCATION_SYSMEM as uint32_t);
    wined3d_texture_invalidate_location(
        texture,
        sub_resource_idx,
        !(WINED3D_LOCATION_SYSMEM as uint32_t),
    );

    if create_dib {
        let mut texture_idx_0: wined3d_texture_idx = wined3d_texture_idx {
            texture: texture,
            sub_resource_idx: sub_resource_idx,
        };

        wined3d_cs_init_object(
            (*device).cs as *mut wined3d_cs,
            Some(wined3d_texture_create_dc as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
            core::ptr::addr_of_mut!(texture_idx_0) as *mut ::core::ffi::c_void,
        );
        wined3d_cs_finish((*device).cs as *mut wined3d_cs, WINED3D_CS_QUEUE_DEFAULT);
    }

    return WINED3D_OK;
}

unsafe extern "C" fn wined3d_texture_gl_prepare_buffer_object(
    mut texture_gl: *mut wined3d_texture_gl,
    mut sub_resource_idx: ::core::ffi::c_uint,
    mut context_gl: *mut wined3d_context_gl,
) {
    let mut sub_resource: *mut wined3d_texture_sub_resource =
        ::core::ptr::null_mut::<wined3d_texture_sub_resource>();
    let mut bo: *mut wined3d_bo_gl = ::core::ptr::null_mut::<wined3d_bo_gl>();
    sub_resource = (*texture_gl)
        .t
        .sub_resources
        .offset(sub_resource_idx as isize) as *mut wined3d_texture_sub_resource
        as *mut wined3d_texture_sub_resource;
    if !(*sub_resource).bo.is_null() {
        return;
    }
    bo = malloc(core::mem::size_of::<wined3d_bo_gl>() as size_t) as *mut wined3d_bo_gl;
    if bo.is_null() {
        return;
    }
    if !wined3d_device_gl_create_bo(
        wined3d_device_gl((*texture_gl).t.resource.device),
        context_gl,
        (*sub_resource).size as GLsizeiptr,
        GL_PIXEL_UNPACK_BUFFER as GLenum,
        GL_STREAM_DRAW as GLenum,
        true,
        (GL_MAP_READ_BIT | GL_MAP_WRITE_BIT | GL_CLIENT_STORAGE_BIT) as GLbitfield,
        bo,
    ) {
        free(bo as *mut ::core::ffi::c_void);
        return;
    }
    WINE_TRACE!("Created buffer object %u for texture %p, sub-resource %u.\n", (*bo).id, texture_gl, sub_resource_idx);
    (*sub_resource).bo = core::ptr::addr_of_mut!((*bo).b);
}

unsafe extern "C" fn wined3d_texture_force_reload(texture: *mut wined3d_texture) {
    let sub_count: ::core::ffi::c_uint = (*texture)
        .level_count
        .wrapping_mul((*texture).layer_count as ::core::ffi::c_uint);
    let mut i: ::core::ffi::c_uint = 0;
    (*texture).flags &= !(WINED3D_TEXTURE_RGB_ALLOCATED
        | WINED3D_TEXTURE_SRGB_ALLOCATED
        | WINED3D_TEXTURE_CONVERTED) as uint32_t;
    (*texture).async_.flags &= !WINED3D_TEXTURE_ASYNC_COLOR_KEY as uint32_t;
    i = 0 as ::core::ffi::c_uint;
    while i < sub_count {
        wined3d_texture_invalidate_location(
            texture,
            i,
            (WINED3D_LOCATION_TEXTURE_RGB | WINED3D_LOCATION_TEXTURE_SRGB) as uint32_t,
        );
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_gl_prepare_texture(texture_gl: *mut wined3d_texture_gl, context_gl: *mut wined3d_context_gl, srgb: BOOL) {
    let alloc_flag: DWORD = (if srgb != 0 { WINED3D_TEXTURE_SRGB_ALLOCATED } else { WINED3D_TEXTURE_RGB_ALLOCATED }) as DWORD;
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let resource: *mut wined3d_resource = core::ptr::addr_of_mut!((*texture_gl).t.resource);
    let device: *const wined3d_device = (*resource).device;
    let mut format: *const wined3d_format = (*resource).format;
    let mut conversion: *const wined3d_color_key_conversion = core::ptr::null();
    let mut format_gl: *const wined3d_format_gl = core::ptr::null();
    let mut internal: GLenum = 0;

    TRACE!(
        "texture_gl %p, context_gl %p, srgb %d, format %s.\n",
        texture_gl,
        context_gl,
        srgb,
        (*format).id,
    );

    if ((*texture_gl).t.flags & (alloc_flag as uint32_t)) != 0 {
        return;
    }

    if ((*resource).format_caps & (WINED3D_FORMAT_CAP_DECOMPRESS as u32)) != 0 {
        TRACE!("WINED3D_FORMAT_CAP_DECOMPRESS set.\n");
        (*texture_gl).t.flags |= WINED3D_TEXTURE_CONVERTED as uint32_t;
        format = wined3d_resource_get_decompress_format(resource);
    } else if (*format).conv_byte_count != 0 {
        (*texture_gl).t.flags |= WINED3D_TEXTURE_CONVERTED as uint32_t;
    } else {
        conversion = wined3d_format_get_color_key_conversion(core::ptr::addr_of_mut!((*texture_gl).t), (TRUE) as BOOL);
        if !conversion.is_null() {
            (*texture_gl).t.flags |= WINED3D_TEXTURE_CONVERTED as uint32_t;
            format = wined3d_get_format(
                (*device).adapter,
                (*conversion).dst_format,
                (*resource).bind_flags,
            );
            TRACE!(
                "Using format %s for color key conversion.\n",
                (*format).id,
            );
        }
    }

    format_gl = wined3d_format_gl(format);

    wined3d_texture_gl_bind_and_dirtify(texture_gl, context_gl, srgb);

    internal = wined3d_gl_get_internal_format(resource, format_gl, srgb != 0) as GLenum;
    if internal == 0 {
        FIXME!(
            "No GL internal format for format %s.\n",
            debug_d3dformat((*format).id),
        );
    }

    TRACE!(
        "internal %#x, format %#x, type %#x.\n",
        internal,
        (*format_gl).format,
        (*format_gl).type_,
    );

    if wined3d_texture_use_immutable_storage(core::ptr::addr_of_mut!((*texture_gl).t), gl_info) != 0 {
        wined3d_texture_gl_allocate_immutable_storage(texture_gl, internal, gl_info);
    } else {
        wined3d_texture_gl_allocate_mutable_storage(texture_gl, internal, format_gl, gl_info);
    }

    (*texture_gl).t.flags = ((*texture_gl).t.flags | alloc_flag as uint32_t) as uint32_t;
}

unsafe extern "C" fn wined3d_texture_gl_prepare_rb(
    texture_gl: *mut wined3d_texture_gl,
    gl_info: *const wined3d_gl_info,
    multisample: BOOL,
) {
    let mut format_gl: *const wined3d_format_gl = ::core::ptr::null::<wined3d_format_gl>();

    format_gl = wined3d_format_gl((*texture_gl).t.resource.format);
    if multisample != 0 {
        let mut samples: DWORD = 0;

        if (*texture_gl).rb_multisample != 0 {
            return;
        }

        samples = wined3d_resource_get_sample_count(core::ptr::addr_of_mut!((*texture_gl).t.resource)) as DWORD;

        ((*gl_info).fbo_ops.glGenRenderbuffers.unwrap_unchecked())(
            1 as GLsizei,
            core::ptr::addr_of_mut!((*texture_gl).rb_multisample),
        );
        ((*gl_info).fbo_ops.glBindRenderbuffer.unwrap_unchecked())(
            GL_RENDERBUFFER as GLenum,
            (*texture_gl).rb_multisample,
        );
        ((*gl_info).fbo_ops.glRenderbufferStorageMultisample.unwrap_unchecked())(
            GL_RENDERBUFFER as GLenum,
            samples as GLsizei,
            (*format_gl).internal as GLenum,
            (*texture_gl).t.resource.width as GLsizei,
            (*texture_gl).t.resource.height as GLsizei,
        );
        checkGLcall!(gl_info, b"glRenderbufferStorageMultisample()\0" as *const u8 as *const ::core::ffi::c_char);
    } else {
        if (*texture_gl).rb_resolved != 0 {
            return;
        }

        ((*gl_info).fbo_ops.glGenRenderbuffers.unwrap_unchecked())(
            1 as GLsizei,
            core::ptr::addr_of_mut!((*texture_gl).rb_resolved),
        );
        ((*gl_info).fbo_ops.glBindRenderbuffer.unwrap_unchecked())(
            GL_RENDERBUFFER as GLenum,
            (*texture_gl).rb_resolved,
        );
        ((*gl_info).fbo_ops.glRenderbufferStorage.unwrap_unchecked())(
            GL_RENDERBUFFER as GLenum,
            (*format_gl).internal as GLenum,
            (*texture_gl).t.resource.width as GLsizei,
            (*texture_gl).t.resource.height as GLsizei,
        );
        checkGLcall!(gl_info, b"glRenderbufferStorage()\0" as *const u8 as *const ::core::ffi::c_char);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_prepare_location(texture: *mut wined3d_texture, sub_resource_idx: ::core::ffi::c_uint, context: *mut wined3d_context, location: uint32_t) -> BOOL {
    return (*(*texture).texture_ops)
        .texture_prepare_location
        .unwrap_unchecked()(
        texture, sub_resource_idx, context, location
    );
}

unsafe extern "C" fn wined3d_texture_get_sub_resource(
    texture: *mut wined3d_texture,
    sub_resource_idx: u32,
) -> *mut wined3d_texture_sub_resource {
    if !wined3d_texture_validate_sub_resource_idx(texture, sub_resource_idx) {
        return core::ptr::null_mut();
    }
    return (*texture).sub_resources.offset(sub_resource_idx as isize) as *mut wined3d_texture_sub_resource;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_add_dirty_region(
    texture: *mut wined3d_texture,
    layer: UINT,
    dirty_region: *const wined3d_box,
) -> HRESULT {
    if layer >= (*texture).layer_count {
        WARN!("Invalid layer %u specified.\n", layer);
        return WINED3DERR_INVALIDCALL;
    }

    if !dirty_region.is_null()
        && (wined3d_resource_check_box_dimensions(core::ptr::addr_of_mut!((*texture).resource), 0, dirty_region)) < 0
    {
        WARN!("Invalid dirty_region %s specified.\n", debug_box(dirty_region));
        return WINED3DERR_INVALIDCALL;
    }

    wined3d_texture_dirty_region_add(texture, layer, dirty_region);
    wined3d_cs_emit_add_dirty_texture_region((*(*texture).resource.device).cs, texture, layer);

    return WINED3D_OK;
}

unsafe extern "C" fn wined3d_texture_gl_upload_bo(
    src_format: *const wined3d_format,
    target: GLenum,
    level: ::core::ffi::c_uint,
    src_row_pitch: ::core::ffi::c_uint,
    src_slice_pitch: ::core::ffi::c_uint,
    dst_x: ::core::ffi::c_uint,
    dst_y: ::core::ffi::c_uint,
    dst_z: ::core::ffi::c_uint,
    update_w: ::core::ffi::c_uint,
    mut update_h: ::core::ffi::c_uint,
    mut update_d: ::core::ffi::c_uint,
    addr: *const BYTE,
    srgb: BOOL,
    dst_texture: *mut wined3d_texture,
    gl_info: *const wined3d_gl_info,
) {
    let format_gl: *const wined3d_format_gl = wined3d_format_gl(src_format);

    if ((*src_format).attrs & WINED3D_FORMAT_ATTR_COMPRESSED) != 0 {
        let internal: GLenum = wined3d_gl_get_internal_format(
            core::ptr::addr_of_mut!((*dst_texture).resource),
            format_gl,
            srgb != 0,
        );
        let mut dst_row_pitch: ::core::ffi::c_uint = 0;
        let mut dst_slice_pitch: ::core::ffi::c_uint = 0;

        wined3d_format_calculate_pitch(
            src_format,
            1,
            update_w,
            update_h,
            core::ptr::addr_of_mut!(dst_row_pitch),
            core::ptr::addr_of_mut!(dst_slice_pitch),
        );

        if target == GL_TEXTURE_1D {
            ((*gl_info).gl_ops.ext.p_glCompressedTexSubImage1D.unwrap_unchecked())(
                target,
                level as GLint,
                dst_x as GLint,
                update_w as GLsizei,
                internal,
                dst_row_pitch as GLsizei,
                addr as *const ::core::ffi::c_void,
            );
        } else {
            let mut row: ::core::ffi::c_uint = 0;
            let mut y: ::core::ffi::c_uint = 0;
            let mut slice: ::core::ffi::c_uint = 0;
            let mut slice_count: ::core::ffi::c_uint = 1;
            let mut row_count: ::core::ffi::c_uint = 1;

            /* glCompressedTexSubImage2D() ignores pixel store state, so we
             * can't use the unpack row length like for glTexSubImage2D. */
            if dst_row_pitch != src_row_pitch {
                row_count = update_h
                    .wrapping_add((*src_format).block_height)
                    .wrapping_sub(1)
                    .wrapping_div((*src_format).block_height);
                update_h = (*src_format).block_height;
                wined3d_format_calculate_pitch(
                    src_format,
                    1,
                    update_w,
                    update_h,
                    core::ptr::addr_of_mut!(dst_row_pitch),
                    core::ptr::addr_of_mut!(dst_slice_pitch),
                );
            }

            if dst_slice_pitch != src_slice_pitch {
                slice_count = update_d;
                update_d = 1;
            }

            slice = 0;
            while slice < slice_count {
                row = 0;
                y = dst_y;
                while row < row_count {
                    let upload_addr: *const BYTE = addr.offset(
                        slice
                            .wrapping_mul(src_slice_pitch)
                            .wrapping_add(row.wrapping_mul(src_row_pitch))
                            as isize,
                    ) as *const BYTE;

                    if target == GL_TEXTURE_2D_ARRAY || target == GL_TEXTURE_3D {
                        ((*gl_info).gl_ops.ext.p_glCompressedTexSubImage3D.unwrap_unchecked())(
                            target,
                            level as GLint,
                            dst_x as GLint,
                            y as GLint,
                            dst_z.wrapping_add(slice) as GLint,
                            update_w as GLsizei,
                            update_h as GLsizei,
                            update_d as GLsizei,
                            internal,
                            update_d.wrapping_mul(dst_slice_pitch) as GLsizei,
                            upload_addr as *const ::core::ffi::c_void,
                        );
                    } else {
                        ((*gl_info).gl_ops.ext.p_glCompressedTexSubImage2D.unwrap_unchecked())(
                            target,
                            level as GLint,
                            dst_x as GLint,
                            y as GLint,
                            update_w as GLsizei,
                            update_h as GLsizei,
                            internal,
                            dst_slice_pitch as GLsizei,
                            upload_addr as *const ::core::ffi::c_void,
                        );
                    }

                    y = y.wrapping_add((*src_format).block_height);
                    row = row.wrapping_add(1);
                }
                slice = slice.wrapping_add(1);
            }
        }
        checkGLcall!(gl_info, "Upload compressed texture data");
    } else {
        let mut y: ::core::ffi::c_uint = 0;
        let mut y_count: ::core::ffi::c_uint = 0;
        let mut z: ::core::ffi::c_uint = 0;
        let mut z_count: ::core::ffi::c_uint = 0;
        let mut unpacking_rows: bool = false;

        if src_row_pitch != 0 && src_row_pitch.wrapping_rem((*src_format).byte_count) == 0 {
            (*gl_info)
                .gl_ops
                .gl
                .p_glPixelStorei
                .unwrap_unchecked()(
                GL_UNPACK_ROW_LENGTH as GLenum,
                src_row_pitch.wrapping_div((*src_format).byte_count) as GLint,
            );
            y_count = 1;
            unpacking_rows = true;
        } else {
            y_count = update_h;
            update_h = 1;
        }

        if src_slice_pitch != 0
            && unpacking_rows
            && src_slice_pitch.wrapping_rem(src_row_pitch) == 0
        {
            (*gl_info)
                .gl_ops
                .gl
                .p_glPixelStorei
                .unwrap_unchecked()(
                GL_UNPACK_IMAGE_HEIGHT as GLenum,
                src_slice_pitch.wrapping_div(src_row_pitch) as GLint,
            );
            z_count = 1;
        } else if src_slice_pitch != 0
            && !unpacking_rows
            && src_slice_pitch
                .wrapping_rem(update_w.wrapping_mul((*src_format).byte_count))
                == 0
        {
            (*gl_info)
                .gl_ops
                .gl
                .p_glPixelStorei
                .unwrap_unchecked()(
                GL_UNPACK_IMAGE_HEIGHT as GLenum,
                src_slice_pitch
                    .wrapping_div(update_w.wrapping_mul((*src_format).byte_count))
                    as GLint,
            );
            z_count = 1;
        } else {
            z_count = update_d;
            update_d = 1;
        }

        z = 0;
        while z < z_count {
            y = 0;
            while y < y_count {
                let upload_addr: *const BYTE = addr.offset(
                    z.wrapping_mul(src_slice_pitch)
                        .wrapping_add(y.wrapping_mul(src_row_pitch))
                        as isize,
                ) as *const BYTE;

                if target == GL_TEXTURE_2D_ARRAY || target == GL_TEXTURE_3D {
                    ((*gl_info).gl_ops.ext.p_glTexSubImage3D.unwrap_unchecked())(
                        target,
                        level as GLint,
                        dst_x as GLint,
                        dst_y.wrapping_add(y) as GLint,
                        dst_z.wrapping_add(z) as GLint,
                        update_w as GLsizei,
                        update_h as GLsizei,
                        update_d as GLsizei,
                        ((*format_gl).format) as u32,
                        ((*format_gl).type_) as u32,
                        upload_addr as *const ::core::ffi::c_void,
                    );
                } else if target == GL_TEXTURE_1D {
                    (*gl_info)
                        .gl_ops
                        .gl
                        .p_glTexSubImage1D
                        .unwrap_unchecked()(
                        target,
                        level as GLint,
                        dst_x as GLint,
                        update_w as GLsizei,
                        (*format_gl).format as GLenum,
                        (*format_gl).type_ as GLenum,
                        upload_addr as *const ::core::ffi::c_void,
                    );
                } else {
                    (*gl_info)
                        .gl_ops
                        .gl
                        .p_glTexSubImage2D
                        .unwrap_unchecked()(
                        target,
                        level as GLint,
                        dst_x as GLint,
                        dst_y.wrapping_add(y) as GLint,
                        update_w as GLsizei,
                        update_h as GLsizei,
                        (*format_gl).format as GLenum,
                        (*format_gl).type_ as GLenum,
                        upload_addr as *const ::core::ffi::c_void,
                    );
                }
                y = y.wrapping_add(1);
            }
            z = z.wrapping_add(1);
        }

        (*gl_info)
            .gl_ops
            .gl
            .p_glPixelStorei
            .unwrap_unchecked()(
            GL_UNPACK_ROW_LENGTH as GLenum,
            0 as GLint,
        );
        (*gl_info)
            .gl_ops
            .gl
            .p_glPixelStorei
            .unwrap_unchecked()(
            GL_UNPACK_IMAGE_HEIGHT as GLenum,
            0 as GLint,
        );
        checkGLcall!(gl_info, "Upload texture data");
    }
}

unsafe extern "C" fn wined3d_get_alpha_fixup_format(format_id: wined3d_format_id, dst_format: *const wined3d_format) -> wined3d_format_id {
    let mut i: u32;

    if ((*dst_format).attrs & WINED3D_FORMAT_ATTR_COMPRESSED) == 0 && (*dst_format).alpha_size == 0 {
        return WINED3DFMT_UNKNOWN;
    }

    i = 0;
    while (i as usize) < (ARRAY_SIZE!(formats_src_alpha_fixup) as usize) {
        if formats_src_alpha_fixup[i as usize].format_id == format_id {
            return formats_src_alpha_fixup[i as usize].conv_format_id;
        }
        i = i.wrapping_add(1);
    }

    return WINED3DFMT_UNKNOWN;
}

unsafe extern "C" fn wined3d_fixup_alpha(
    mut format: *const wined3d_format,
    mut src: *const uint8_t,
    mut src_row_pitch: ::core::ffi::c_uint,
    mut dst: *mut uint8_t,
    mut dst_row_pitch: ::core::ffi::c_uint,
    mut width: ::core::ffi::c_uint,
    mut height: ::core::ffi::c_uint,
) {
    let mut byte_count: ::core::ffi::c_uint = 0;
    let mut alpha_mask: ::core::ffi::c_uint = 0;
    let mut x: ::core::ffi::c_uint = 0;
    let mut y: ::core::ffi::c_uint = 0;
    byte_count = (*format).byte_count as ::core::ffi::c_uint;
    alpha_mask = (wined3d_mask_from_size((*format).alpha_size as ::core::ffi::c_uint)
        << (*format).alpha_offset) as ::core::ffi::c_uint;
    match byte_count {
        2 => {
            y = 0 as ::core::ffi::c_uint;
            while y < height {
                let mut src_row: *const uint16_t =
                    src.offset(y.wrapping_mul(src_row_pitch) as isize) as *const uint8_t
                        as *const uint16_t;
                let mut dst_row: *mut uint16_t = dst.offset(y.wrapping_mul(dst_row_pitch) as isize)
                    as *mut uint8_t
                    as *mut uint16_t;
                x = 0 as ::core::ffi::c_uint;
                while x < width {
                    *dst_row.offset(x as isize) = (*src_row.offset(x as isize)
                        as ::core::ffi::c_uint
                        | alpha_mask) as uint16_t;
                    x = x.wrapping_add(1);
                }
                y = y.wrapping_add(1);
            }
        }
        4 => {
            y = 0 as ::core::ffi::c_uint;
            while y < height {
                let mut src_row_0: *const uint32_t =
                    src.offset(y.wrapping_mul(src_row_pitch) as isize) as *const uint8_t
                        as *const uint32_t;
                let mut dst_row_0: *mut uint32_t =
                    dst.offset(y.wrapping_mul(dst_row_pitch) as isize) as *mut uint8_t
                        as *mut uint32_t;
                x = 0 as ::core::ffi::c_uint;
                while x < width {
                    *dst_row_0.offset(x as isize) =
                        *src_row_0.offset(x as isize) | alpha_mask as uint32_t;
                    x = x.wrapping_add(1);
                }
                y = y.wrapping_add(1);
            }
        }
        _ => {
            ERR!("Unsupported byte count %u.\n", byte_count);
        }
    };
}

unsafe extern "C" fn wined3d_texture_gl_upload_data(
    mut context: *mut wined3d_context,
    mut src_bo_addr: *const wined3d_const_bo_address,
    mut src_format: *const wined3d_format,
    mut src_box: *const wined3d_box,
    mut src_row_pitch: ::core::ffi::c_uint,
    mut src_slice_pitch: ::core::ffi::c_uint,
    mut dst_texture: *mut wined3d_texture,
    mut dst_sub_resource_idx: ::core::ffi::c_uint,
    mut dst_location: ::core::ffi::c_uint,
    mut dst_x: ::core::ffi::c_uint,
    mut dst_y: ::core::ffi::c_uint,
    mut dst_z: ::core::ffi::c_uint,
) {
    let mut context_gl: *mut wined3d_context_gl = wined3d_context_gl(context);
    let mut alpha_fixup_format_id: wined3d_format_id = WINED3DFMT_UNKNOWN;
    let mut gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut update_w: ::core::ffi::c_uint = ((*src_box).right as ::core::ffi::c_uint)
        .wrapping_sub((*src_box).left as ::core::ffi::c_uint);
    let mut update_h: ::core::ffi::c_uint = ((*src_box).bottom as ::core::ffi::c_uint)
        .wrapping_sub((*src_box).top as ::core::ffi::c_uint);
    let mut update_d: ::core::ffi::c_uint = ((*src_box).back as ::core::ffi::c_uint)
        .wrapping_sub((*src_box).front as ::core::ffi::c_uint);
    let mut bo: wined3d_bo_address = wined3d_bo_address {
        buffer_object: ::core::ptr::null_mut::<wined3d_bo>(),
        addr: ::core::ptr::null_mut::<BYTE>(),
    };
    let mut level: ::core::ffi::c_uint = 0;
    let mut srgb: BOOL = (FALSE as i32);
    let mut decompress: BOOL = 0;
    let mut target: GLenum = 0;

    TRACE!(
        "context %p, src_bo_addr %s, src_format %s, src_box %s, src_row_pitch %u, src_slice_pitch %u, dst_texture %p, dst_sub_resource_idx %u, dst_location %s, dst_x %u, dst_y %u, dst_z %u.\n",
        context,
        debug_const_bo_address(src_bo_addr),
        debug_d3dformat((*src_format).id),
        debug_box(src_box),
        src_row_pitch,
        src_slice_pitch,
        dst_texture,
        dst_sub_resource_idx,
        wined3d_debug_location(dst_location as uint32_t),
        dst_x,
        dst_y,
        dst_z
    );

    if dst_location == WINED3D_LOCATION_TEXTURE_SRGB as ::core::ffi::c_uint {
        srgb = (TRUE as i32);
    } else if dst_location != WINED3D_LOCATION_TEXTURE_RGB as ::core::ffi::c_uint {
        FIXME!(
            "Unhandled location %s.\n",
            wined3d_debug_location(dst_location as uint32_t)
        );
        return;
    }

    wined3d_texture_gl_bind_and_dirtify(
        wined3d_texture_gl(dst_texture),
        wined3d_context_gl(context),
        srgb,
    );

    if (*(*dst_texture)
        .sub_resources
        .offset(dst_sub_resource_idx as isize))
    .map_count
        != 0
    {
        WARN!("Uploading a texture that is currently mapped, pinning sysmem.\n");
        (*dst_texture).resource.set_pin_sysmem((1) as _);
    }

    if ((*src_format).attrs & (WINED3D_FORMAT_ATTR_HEIGHT_SCALE as ::core::ffi::c_uint)) != 0 {
        update_h = update_h.wrapping_mul((*src_format).height_scale.numerator as ::core::ffi::c_uint);
        update_h = update_h.wrapping_div((*src_format).height_scale.denominator as ::core::ffi::c_uint);
    }

    target = wined3d_texture_gl_get_sub_resource_target(
        wined3d_texture_gl(dst_texture),
        dst_sub_resource_idx,
    );
    level = dst_sub_resource_idx.wrapping_rem((*dst_texture).level_count);

    match target {
        GL_TEXTURE_1D_ARRAY => {
            dst_y = dst_sub_resource_idx.wrapping_div((*dst_texture).level_count);
            update_h = 1;
        }
        GL_TEXTURE_2D_ARRAY => {
            dst_z = dst_sub_resource_idx.wrapping_div((*dst_texture).level_count);
            update_d = 1;
        }
        GL_TEXTURE_2D_MULTISAMPLE | GL_TEXTURE_2D_MULTISAMPLE_ARRAY => {
            FIXME!("Not supported for multisample textures.\n");
            return;
        }
        _ => {}
    }

    bo.buffer_object = (*src_bo_addr).buffer_object;
    bo.addr = ((*src_bo_addr).addr as *mut BYTE)
        .offset(((*src_box).front as ::core::ffi::c_uint).wrapping_mul(src_slice_pitch) as isize);
    if ((*dst_texture).resource.format_attrs & (WINED3D_FORMAT_ATTR_BLOCKS as ::core::ffi::c_uint)) != 0 {
        bo.addr = bo.addr.offset(
            ((*src_box).top as ::core::ffi::c_uint)
                .wrapping_div((*src_format).block_height as ::core::ffi::c_uint)
                .wrapping_mul(src_row_pitch) as isize,
        );
        bo.addr = bo.addr.offset(
            (*src_box)
                .left
                .wrapping_div((*src_format).block_width)
                .wrapping_mul((*src_format).block_byte_count) as isize,
        );
    } else {
        bo.addr = bo
            .addr
            .offset(((*src_box).top as ::core::ffi::c_uint).wrapping_mul(src_row_pitch) as isize);
        bo.addr = bo
            .addr
            .offset((*src_box).left.wrapping_mul((*src_format).byte_count) as isize);
    }

    decompress = (((*dst_texture).resource.format_caps & (WINED3D_FORMAT_CAP_DECOMPRESS as ::core::ffi::c_uint)) != 0
        || ((*src_format).decompress.is_some()
            && (*src_format).id != (*(*dst_texture).resource.format).id)) as ::core::ffi::c_int as BOOL;

    if (*src_format).upload.is_some() || decompress != 0 || {
        alpha_fixup_format_id =
            wined3d_get_alpha_fixup_format((*src_format).id, (*dst_texture).resource.format);
        alpha_fixup_format_id != WINED3DFMT_UNKNOWN
    } {
        let mut compressed_format: *const wined3d_format = src_format;
        let mut dst_row_pitch: ::core::ffi::c_uint = 0;
        let mut dst_slice_pitch: ::core::ffi::c_uint = 0;
        let mut f: wined3d_format_gl = ::core::mem::zeroed();
        let mut converted_mem: *mut ::core::ffi::c_void = ::core::ptr::null_mut();
        let mut z: ::core::ffi::c_uint = 0;
        let mut src_mem: *mut BYTE = ::core::ptr::null_mut();

        if decompress != 0 {
            src_format = wined3d_resource_get_decompress_format(core::ptr::addr_of_mut!((*dst_texture).resource));
        } else if alpha_fixup_format_id != WINED3DFMT_UNKNOWN {
            src_format = wined3d_get_format(
                (*(*context).device).adapter,
                alpha_fixup_format_id,
                0,
            );
            assert!(((!src_format.is_null() as ::core::ffi::c_int) != 0));
        } else {
            if ((*dst_texture).resource.format_attrs & (WINED3D_FORMAT_ATTR_BLOCKS as ::core::ffi::c_uint)) != 0 {
                ERR!("Converting a block-based format.\n");
            }

            f = *wined3d_format_gl(src_format);
            f.f.byte_count = (*src_format).conv_byte_count;
            src_format = core::ptr::addr_of_mut!(f.f);
        }

        wined3d_format_calculate_pitch(
            src_format,
            1,
            update_w,
            update_h,
            core::ptr::addr_of_mut!(dst_row_pitch),
            core::ptr::addr_of_mut!(dst_slice_pitch),
        );

        converted_mem = malloc(dst_slice_pitch as usize);
        if converted_mem.is_null() {
            ERR!("Failed to allocate upload buffer.\n");
            return;
        }

        src_mem = wined3d_context_gl_map_bo_address(
            context_gl,
            core::ptr::addr_of_mut!(bo),
            src_slice_pitch.wrapping_mul(update_d) as usize,
            WINED3D_MAP_READ,
        ) as *mut BYTE;

        ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(GL_PIXEL_UNPACK_BUFFER, 0);
        checkGLcall!(gl_info, b"glBindBuffer\0" as *const u8 as *const ::core::ffi::c_char);

        z = 0;
        while z < update_d {
            if decompress != 0 {
                (*compressed_format)
                    .decompress
                    .unwrap_unchecked()(
                    src_mem,
                    converted_mem as *mut BYTE,
                    src_row_pitch,
                    src_slice_pitch,
                    dst_row_pitch,
                    dst_slice_pitch,
                    update_w,
                    update_h,
                    1,
                );
            } else if alpha_fixup_format_id != WINED3DFMT_UNKNOWN {
                wined3d_fixup_alpha(
                    src_format,
                    src_mem,
                    src_row_pitch,
                    converted_mem as *mut uint8_t,
                    dst_row_pitch,
                    update_w,
                    update_h,
                );
            } else {
                (*src_format).upload.unwrap_unchecked()(
                    src_mem,
                    converted_mem as *mut BYTE,
                    src_row_pitch,
                    src_slice_pitch,
                    dst_row_pitch,
                    dst_slice_pitch,
                    update_w,
                    update_h,
                    1,
                );
            }

            wined3d_texture_gl_upload_bo(
                src_format,
                target,
                level,
                dst_row_pitch,
                dst_slice_pitch,
                dst_x,
                dst_y,
                dst_z.wrapping_add(z),
                update_w,
                update_h,
                1,
                converted_mem as *const BYTE,
                srgb,
                dst_texture,
                gl_info,
            );
            z = z.wrapping_add(1);
            src_mem = src_mem.offset(src_slice_pitch as isize);
        }

        wined3d_context_gl_unmap_bo_address(
            context_gl,
            core::ptr::addr_of_mut!(bo),
            0,
            core::ptr::null(),
        );
        free(converted_mem);
    } else {
        let mut offset: *const uint8_t = bo.addr;

        if !bo.buffer_object.is_null() {
            ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(
                GL_PIXEL_UNPACK_BUFFER,
                (*wined3d_bo_gl(bo.buffer_object)).id,
            );
            checkGLcall!(gl_info, b"glBindBuffer\0" as *const u8 as *const ::core::ffi::c_char);
            offset = offset.offset((*bo.buffer_object).buffer_offset as isize);
        } else {
            ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(GL_PIXEL_UNPACK_BUFFER, 0);
            checkGLcall!(gl_info, b"glBindBuffer\0" as *const u8 as *const ::core::ffi::c_char);
        }

        wined3d_texture_gl_upload_bo(
            src_format,
            target,
            level,
            src_row_pitch,
            src_slice_pitch,
            dst_x,
            dst_y,
            dst_z,
            update_w,
            update_h,
            update_d,
            offset as *const BYTE,
            srgb,
            dst_texture,
            gl_info,
        );

        if !bo.buffer_object.is_null() {
            ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(GL_PIXEL_UNPACK_BUFFER, 0);
            wined3d_context_gl_reference_bo(
                context_gl,
                wined3d_bo_gl(bo.buffer_object),
            );
            checkGLcall!(gl_info, b"glBindBuffer\0" as *const u8 as *const ::core::ffi::c_char);
        }
    }

    if ((*gl_info).quirks & (WINED3D_QUIRK_FBO_TEX_UPDATE as uint32_t)) != 0 {
        let mut device: *mut wined3d_device = (*dst_texture).resource.device;
        let mut i: ::core::ffi::c_uint = 0;

        i = 0;
        while i < (*device).context_count {
            wined3d_context_gl_texture_update(
                wined3d_context_gl(*(*device).contexts.offset(i as isize)),
                wined3d_texture_gl(dst_texture),
            );
            i = i.wrapping_add(1);
        }
    }
}

unsafe extern "C" fn wined3d_texture_gl_download_data_slow_path(
    texture_gl: *mut wined3d_texture_gl,
    sub_resource_idx: ::core::ffi::c_uint,
    context_gl: *mut wined3d_context_gl,
    data: *const wined3d_bo_address,
) {
    let bo: *mut wined3d_bo_gl = wined3d_bo_gl((*data).buffer_object);
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut sub_resource: *mut wined3d_texture_sub_resource =
        ::core::ptr::null_mut::<wined3d_texture_sub_resource>();
    let mut dst_row_pitch: ::core::ffi::c_uint = 0;
    let mut dst_slice_pitch: ::core::ffi::c_uint = 0;
    let mut src_row_pitch: ::core::ffi::c_uint = 0;
    let mut src_slice_pitch: ::core::ffi::c_uint = 0;
    let mut format_gl: *const wined3d_format_gl = ::core::ptr::null::<wined3d_format_gl>();
    let mut temporary_mem: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
    let mut level: ::core::ffi::c_uint = 0;
    let mut target: GLenum = 0;
    let mut mem: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();

    format_gl = wined3d_format_gl((*texture_gl).t.resource.format);

    /* Only support read back of converted P8 textures. */
    if ((*texture_gl).t.flags & WINED3D_TEXTURE_CONVERTED) != 0
        && (*format_gl).f.id as ::core::ffi::c_uint != WINED3DFMT_P8_UINT
        && (*format_gl).f.download.is_none()
    {
        ERR!(
            "Trying to read back converted texture %p, %u with format %s.\n",
            texture_gl,
            sub_resource_idx,
            debug_d3dformat((*format_gl).f.id)
        );
        return;
    }

    sub_resource = (*texture_gl)
        .t
        .sub_resources
        .offset(sub_resource_idx as isize) as *mut wined3d_texture_sub_resource;
    target = wined3d_texture_gl_get_sub_resource_target(texture_gl, sub_resource_idx);
    level = sub_resource_idx % (*texture_gl).t.level_count;

    if target == GL_TEXTURE_1D_ARRAY || target == GL_TEXTURE_2D_ARRAY {
        if (*format_gl).f.download.is_some() {
            return;
        }

        WINE_WARN_!(__wine_dbch_d3d_perf,
            "Downloading all miplevel layers to get the data for a single sub-resource.\n"
        );

        temporary_mem = calloc(
            ((*texture_gl).t.layer_count as usize) as _,
            ((*sub_resource).size as usize) as _,
        ) as *mut BYTE;
        if temporary_mem.is_null() {
            ERR!("Out of memory.\n");
            return;
        }
    }

    if (*format_gl).f.download.is_some() {
        let mut f: wined3d_format = core::mem::zeroed();

        if !bo.is_null() {
            ERR!("Converted texture %p uses PBO unexpectedly.\n", texture_gl);
        }

        WINE_WARN_!(__wine_dbch_d3d_perf,
            "Downloading converted texture %p, %u with format %s.\n",
            texture_gl,
            sub_resource_idx,
            debug_d3dformat((*format_gl).f.id)
        );

        f = (*format_gl).f;
        f.byte_count = (*format_gl).f.conv_byte_count;
        wined3d_texture_get_pitch(
            core::ptr::addr_of_mut!((*texture_gl).t),
            level,
            core::ptr::addr_of_mut!(dst_row_pitch),
            core::ptr::addr_of_mut!(dst_slice_pitch),
        );
        wined3d_format_calculate_pitch(
            core::ptr::addr_of_mut!(f),
            (*(*texture_gl).t.resource.device).surface_alignment as ::core::ffi::c_uint,
            wined3d_texture_get_level_width(core::ptr::addr_of_mut!((*texture_gl).t), level),
            wined3d_texture_get_level_height(core::ptr::addr_of_mut!((*texture_gl).t), level),
            core::ptr::addr_of_mut!(src_row_pitch),
            core::ptr::addr_of_mut!(src_slice_pitch),
        );

        temporary_mem = malloc(src_slice_pitch as usize) as *mut BYTE;
        if temporary_mem.is_null() {
            ERR!("Failed to allocate memory.\n");
            return;
        }
    }

    if !temporary_mem.is_null() {
        ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(
            GL_PIXEL_PACK_BUFFER,
            0,
        );
        checkGLcall!(gl_info, b"glBindBuffer\0".as_ptr() as *const ::core::ffi::c_char);
        mem = temporary_mem as *mut ::core::ffi::c_void;
    } else if !bo.is_null() {
        ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(
            GL_PIXEL_PACK_BUFFER,
            (*bo).id,
        );
        checkGLcall!(gl_info, b"glBindBuffer\0".as_ptr() as *const ::core::ffi::c_char);
        mem = ((*data).addr as *mut u8).add((*bo).b.buffer_offset) as *mut ::core::ffi::c_void;
    } else {
        ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(
            GL_PIXEL_PACK_BUFFER,
            0,
        );
        checkGLcall!(gl_info, b"glBindBuffer\0".as_ptr() as *const ::core::ffi::c_char);
        mem = (*data).addr as *mut ::core::ffi::c_void;
    }

    if ((*texture_gl).t.resource.format_attrs & WINED3D_FORMAT_ATTR_COMPRESSED) != 0 {
        ((*gl_info).gl_ops.ext.p_glGetCompressedTexImage.unwrap_unchecked())(target, (level) as i32, mem);
        checkGLcall!(gl_info, b"glGetCompressedTexImage\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        (*gl_info)
            .gl_ops
            .gl
            .p_glGetTexImage
            .unwrap_unchecked()(
            target,
            level as GLint,
            (*format_gl).format as GLenum,
            (*format_gl).type_ as GLenum,
            mem,
        );
        checkGLcall!(gl_info, b"glGetTexImage\0".as_ptr() as *const ::core::ffi::c_char);
    }

    if (*format_gl).f.download.is_some() {
        (*format_gl).f.download.unwrap_unchecked()(
            mem as *const BYTE,
            (*data).addr,
            src_row_pitch,
            src_slice_pitch,
            dst_row_pitch,
            dst_slice_pitch,
            wined3d_texture_get_level_width(core::ptr::addr_of_mut!((*texture_gl).t), level),
            wined3d_texture_get_level_height(core::ptr::addr_of_mut!((*texture_gl).t), level),
            1,
        );
    } else if !temporary_mem.is_null() {
        let layer: ::core::ffi::c_uint = sub_resource_idx / (*texture_gl).t.level_count;
        let src_data: *mut ::core::ffi::c_void = temporary_mem
            .add(layer as usize * (*sub_resource).size as usize)
            as *mut ::core::ffi::c_void;
        if !bo.is_null() {
            ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(
                GL_PIXEL_PACK_BUFFER,
                (*bo).id,
            );
            checkGLcall!(gl_info, b"glBindBuffer\0".as_ptr() as *const ::core::ffi::c_char);
            ((*gl_info).gl_ops.ext.p_glBufferSubData.unwrap_unchecked())(
                GL_PIXEL_PACK_BUFFER,
                ((*data).addr as usize).wrapping_add((*bo).b.buffer_offset) as _,
                (*sub_resource).size as usize as _,
                src_data,
            );
            checkGLcall!(gl_info, b"glBufferSubData\0".as_ptr() as *const ::core::ffi::c_char);
        } else {
            memcpy(
                (*data).addr as *mut ::core::ffi::c_void,
                src_data,
                (*sub_resource).size as usize,
            );
        }
    }

    if !bo.is_null() {
        ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(
            GL_PIXEL_PACK_BUFFER,
            0,
        );
        wined3d_context_gl_reference_bo(context_gl, bo);
        checkGLcall!(gl_info, b"glBindBuffer\0".as_ptr() as *const ::core::ffi::c_char);
    }

    free(temporary_mem as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn wined3d_texture_gl_download_data(
    context: *mut wined3d_context,
    src_texture: *mut wined3d_texture,
    src_sub_resource_idx: ::core::ffi::c_uint,
    src_location: ::core::ffi::c_uint,
    src_box: *const wined3d_box,
    dst_bo_addr: *const wined3d_bo_address,
    dst_format: *const wined3d_format,
    dst_x: ::core::ffi::c_uint,
    dst_y: ::core::ffi::c_uint,
    dst_z: ::core::ffi::c_uint,
    dst_row_pitch: ::core::ffi::c_uint,
    dst_slice_pitch: ::core::ffi::c_uint,
) {
    let src_texture_gl: *mut wined3d_texture_gl = wined3d_texture_gl(src_texture);
    let context_gl: *mut wined3d_context_gl = wined3d_context_gl(context);
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut src_level: ::core::ffi::c_uint = 0;
    let mut src_width: ::core::ffi::c_uint = 0;
    let mut src_height: ::core::ffi::c_uint = 0;
    let mut src_depth: ::core::ffi::c_uint = 0;
    let mut src_row_pitch: ::core::ffi::c_uint = 0;
    let mut src_slice_pitch: ::core::ffi::c_uint = 0;
    let mut format_gl: *const wined3d_format_gl = core::ptr::null();
    let mut offset: *mut uint8_t = (*dst_bo_addr).addr as *mut uint8_t;
    let mut dst_bo: *mut wined3d_bo = core::ptr::null_mut();
    let mut srgb: BOOL = (FALSE as i32);
    let mut target: GLenum = 0;

    TRACE!(
        "context %p, src_texture %p, src_sub_resource_idx %u, src_location %s, src_box %s, dst_bo_addr %s, dst_format %s, dst_x %u, dst_y %u, dst_z %u, dst_row_pitch %u, dst_slice_pitch %u.\n",
        context,
        src_texture,
        src_sub_resource_idx,
        wined3d_debug_location(src_location as uint32_t),
        debug_box(src_box),
        debug_bo_address(dst_bo_addr),
        debug_d3dformat((*dst_format).id),
        dst_x,
        dst_y,
        dst_z,
        dst_row_pitch,
        dst_slice_pitch
    );

    if src_location == WINED3D_LOCATION_TEXTURE_SRGB {
        srgb = (TRUE as i32);
    } else if src_location != WINED3D_LOCATION_TEXTURE_RGB {
        FIXME!("Unhandled location %s.\n", wined3d_debug_location(src_location as uint32_t));
        return;
    }

    src_level = src_sub_resource_idx.wrapping_rem((*src_texture).level_count);
    src_width = wined3d_texture_get_level_width(src_texture, src_level);
    src_height = wined3d_texture_get_level_height(src_texture, src_level);
    src_depth = wined3d_texture_get_level_depth(src_texture, src_level);
    if (*src_box).left != 0
        || (*src_box).top != 0
        || (*src_box).right != src_width
        || (*src_box).bottom != src_height
        || (*src_box).front != 0
        || (*src_box).back != src_depth
    {
        FIXME!("Unhandled source box %s.\n", debug_box(src_box));
        return;
    }

    if dst_x != 0 || dst_y != 0 || dst_z != 0 {
        FIXME!("Unhandled destination (%u, %u, %u).\n", dst_x, dst_y, dst_z);
        return;
    }

    if (*dst_format).id as ::core::ffi::c_uint
        != (*(*src_texture).resource.format).id as ::core::ffi::c_uint
    {
        FIXME!(
            "Unhandled format conversion (%s -> %s).\n",
            debug_d3dformat((*(*src_texture).resource.format).id),
            debug_d3dformat((*dst_format).id)
        );
        return;
    }

    wined3d_texture_get_pitch(src_texture, src_level, &raw mut src_row_pitch, &raw mut src_slice_pitch);
    if src_row_pitch != dst_row_pitch || src_slice_pitch != dst_slice_pitch {
        FIXME!(
            "Unhandled destination pitches %u/%u (source pitches %u/%u).\n",
            dst_row_pitch,
            dst_slice_pitch,
            src_row_pitch,
            src_slice_pitch
        );
        return;
    }

    wined3d_texture_gl_bind_and_dirtify(src_texture_gl, context_gl, srgb);

    format_gl = wined3d_format_gl((*src_texture).resource.format);
    target = wined3d_texture_gl_get_sub_resource_target(src_texture_gl, src_sub_resource_idx);

    if ((*src_texture).resource.type_ as ::core::ffi::c_uint
        == WINED3D_RTYPE_TEXTURE_2D as ::core::ffi::c_uint
        && (target == GL_TEXTURE_2D_ARRAY as GLenum
            || (*format_gl).f.conv_byte_count != 0
            || (*src_texture).flags & WINED3D_TEXTURE_CONVERTED as uint32_t != 0))
        || target == GL_TEXTURE_1D_ARRAY as GLenum
    {
        wined3d_texture_gl_download_data_slow_path(src_texture_gl, src_sub_resource_idx, context_gl, dst_bo_addr);
        return;
    }

    if (*format_gl).f.conv_byte_count != 0 {
        FIXME!(
            "Attempting to download a converted texture, type %s format %s.\n",
            debug_d3dresourcetype((*src_texture).resource.type_),
            debug_d3dformat((*format_gl).f.id)
        );
        return;
    }

    dst_bo = (*dst_bo_addr).buffer_object;
    if !dst_bo.is_null() {
        ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(
            GL_PIXEL_PACK_BUFFER as GLenum,
            (*wined3d_bo_gl(dst_bo)).id
        );
        checkGLcall!(gl_info, "glBindBuffer");
        offset = offset.offset((*dst_bo).buffer_offset as isize);
    } else {
        ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(
            GL_PIXEL_PACK_BUFFER as GLenum,
            0 as GLuint
        );
        checkGLcall!(gl_info, "glBindBuffer");
    }

    if (*src_texture).resource.format_attrs & WINED3D_FORMAT_ATTR_COMPRESSED as ::core::ffi::c_uint != 0 {
        TRACE!(
            "Downloading compressed texture %p, %u, level %u, format %#x, type %#x, data %p.\n",
            src_texture,
            src_sub_resource_idx,
            src_level,
            (*format_gl).format,
            (*format_gl).type_,
            offset
        );
        ((*gl_info).gl_ops.ext.p_glGetCompressedTexImage.unwrap_unchecked())(
            target,
            src_level as GLint,
            offset as *mut ::core::ffi::c_void
        );
        checkGLcall!(gl_info, "glGetCompressedTexImage");
    } else {
        TRACE!(
            "Downloading texture %p, %u, level %u, format %#x, type %#x, data %p.\n",
            src_texture,
            src_sub_resource_idx,
            src_level,
            (*format_gl).format,
            (*format_gl).type_,
            offset
        );
        (*gl_info)
            .gl_ops
            .gl
            .p_glGetTexImage
            .unwrap_unchecked()(
            target,
            src_level as GLint,
            (*format_gl).format as GLenum,
            (*format_gl).type_ as GLenum,
            offset as *mut ::core::ffi::c_void,
        );
        checkGLcall!(gl_info, "glGetTexImage");
    }

    if !dst_bo.is_null() {
        ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(
            GL_PIXEL_PACK_BUFFER as GLenum,
            0 as GLuint
        );
        wined3d_context_gl_reference_bo(context_gl, wined3d_bo_gl(dst_bo));
        checkGLcall!(gl_info, "glBindBuffer");
    }
}

unsafe extern "C" fn wined3d_texture_gl_load_sysmem(
    mut texture_gl: *mut wined3d_texture_gl,
    mut sub_resource_idx: ::core::ffi::c_uint,
    mut context_gl: *mut wined3d_context_gl,
    mut dst_location: DWORD,
) -> BOOL {
    let mut sub_resource: *mut wined3d_texture_sub_resource =
        ::core::ptr::null_mut::<wined3d_texture_sub_resource>();

    sub_resource = (*texture_gl)
        .t
        .sub_resources
        .add(sub_resource_idx as usize) as *mut wined3d_texture_sub_resource;

    /* We cannot download data from multisample textures directly. */
    if wined3d_texture_gl_is_multisample_location(texture_gl, WINED3D_LOCATION_TEXTURE_RGB) as ::core::ffi::c_int != 0
        || (*sub_resource).locations & WINED3D_LOCATION_RB_MULTISAMPLE != 0
    {
        wined3d_texture_load_location(
            core::ptr::addr_of_mut!((*texture_gl).t),
            sub_resource_idx,
            core::ptr::addr_of_mut!((*context_gl).c),
            WINED3D_LOCATION_RB_RESOLVED,
        );
    }

    if (*sub_resource).locations & WINED3D_LOCATION_RB_RESOLVED != 0 {
        texture2d_read_from_framebuffer(
            core::ptr::addr_of_mut!((*texture_gl).t),
            sub_resource_idx,
            core::ptr::addr_of_mut!((*context_gl).c),
            WINED3D_LOCATION_RB_RESOLVED,
            dst_location,
        );
        return (TRUE) as BOOL;
    }

    if (*sub_resource).locations & (WINED3D_LOCATION_TEXTURE_RGB | WINED3D_LOCATION_TEXTURE_SRGB) != 0 {
        let mut row_pitch: ::core::ffi::c_uint = 0;
        let mut slice_pitch: ::core::ffi::c_uint = 0;
        let mut level: ::core::ffi::c_uint = 0;
        let mut data: wined3d_bo_address = wined3d_bo_address {
            buffer_object: ::core::ptr::null_mut::<wined3d_bo>(),
            addr: ::core::ptr::null_mut::<BYTE>(),
        };
        let mut src_box: wined3d_box = wined3d_box {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
            front: 0,
            back: 0,
        };
        let mut src_location: ::core::ffi::c_uint = 0;

        level = sub_resource_idx % (*texture_gl).t.level_count;
        wined3d_texture_get_bo_address(
            core::ptr::addr_of_mut!((*texture_gl).t),
            sub_resource_idx,
            core::ptr::addr_of_mut!(data),
            dst_location,
        );
        src_location = if (*sub_resource).locations & WINED3D_LOCATION_TEXTURE_RGB != 0 {
            WINED3D_LOCATION_TEXTURE_RGB
        } else {
            WINED3D_LOCATION_TEXTURE_SRGB
        };
        wined3d_texture_get_level_box(
            core::ptr::addr_of_mut!((*texture_gl).t),
            level,
            core::ptr::addr_of_mut!(src_box),
        );
        wined3d_texture_get_pitch(
            core::ptr::addr_of_mut!((*texture_gl).t),
            level,
            core::ptr::addr_of_mut!(row_pitch),
            core::ptr::addr_of_mut!(slice_pitch),
        );
        wined3d_texture_gl_download_data(
            core::ptr::addr_of_mut!((*context_gl).c),
            core::ptr::addr_of_mut!((*texture_gl).t),
            sub_resource_idx,
            src_location,
            core::ptr::addr_of_mut!(src_box),
            core::ptr::addr_of_mut!(data),
            (*texture_gl).t.resource.format,
            0,
            0,
            0,
            row_pitch,
            slice_pitch,
        );

        (*texture_gl).t.download_count = (*texture_gl).t.download_count.wrapping_add(1);
        return (TRUE) as BOOL;
    }

    if (*sub_resource).locations & WINED3D_LOCATION_DRAWABLE != 0 {
        texture2d_read_from_framebuffer(
            core::ptr::addr_of_mut!((*texture_gl).t),
            sub_resource_idx,
            core::ptr::addr_of_mut!((*context_gl).c),
            ((*texture_gl).t.resource.draw_binding) as DWORD,
            dst_location,
        );
        return (TRUE) as BOOL;
    }

    WINE_FIXME!("Can't load texture %p, %u with location flags %s into sysmem.\n", texture_gl, sub_resource_idx, wined3d_debug_location((*sub_resource).locations as uint32_t));

    return (FALSE) as BOOL;
}

unsafe extern "C" fn wined3d_texture_load_drawable(
    texture: *mut wined3d_texture,
    sub_resource_idx: core::ffi::c_uint,
    context: *mut wined3d_context,
) -> BOOL {
    let mut device: *mut wined3d_device = core::ptr::null_mut();
    let mut level: core::ffi::c_uint = 0;
    let mut r: RECT = core::mem::zeroed();

    if (*texture).resource.bind_flags & WINED3D_BIND_DEPTH_STENCIL != 0 {
        let current: DWORD = (*(*texture).sub_resources.add(sub_resource_idx as usize)).locations;

        return (FALSE) as BOOL;
    }

    if wined3d_resource_is_offscreen(core::ptr::addr_of_mut!((*texture).resource)) != 0 {
        ERR!("Trying to load offscreen texture into WINED3D_LOCATION_DRAWABLE.\n");
        return (FALSE) as BOOL;
    }

    device = (*texture).resource.device;
    level = sub_resource_idx % (*texture).level_count;
    SetRect(
        core::ptr::addr_of_mut!(r),
        0,
        0,
        wined3d_texture_get_level_width(texture, level) as i32,
        wined3d_texture_get_level_height(texture, level) as i32,
    );
    wined3d_texture_load_location(texture, sub_resource_idx, context, WINED3D_LOCATION_TEXTURE_RGB);
    ((*(*(*device).blitter).ops).blitter_blit.unwrap_unchecked())(
        (*device).blitter,
        WINED3D_BLIT_OP_COLOR_BLIT,
        context,
        texture,
        sub_resource_idx,
        WINED3D_LOCATION_TEXTURE_RGB,
        core::ptr::addr_of_mut!(r),
        texture,
        sub_resource_idx,
        WINED3D_LOCATION_DRAWABLE,
        core::ptr::addr_of_mut!(r),
        core::ptr::null_mut(),
        WINED3D_TEXF_POINT,
        core::ptr::null_mut(),
    );

    return (TRUE) as BOOL;
}

unsafe extern "C" fn wined3d_texture_load_renderbuffer(
    mut texture: *mut wined3d_texture,
    mut sub_resource_idx: ::core::ffi::c_uint,
    mut context: *mut wined3d_context,
    mut dst_location: DWORD,
) -> BOOL {
    let mut level: ::core::ffi::c_uint = sub_resource_idx.wrapping_rem((*texture).level_count);
    let rect: RECT = RECT {
        left: 0 as LONG,
        top: 0 as LONG,
        right: wined3d_texture_get_level_width(texture, level) as LONG,
        bottom: wined3d_texture_get_level_height(texture, level) as LONG,
    };
    let mut sub_resource: *mut wined3d_texture_sub_resource =
        ::core::ptr::null_mut::<wined3d_texture_sub_resource>();
    let mut src_location: DWORD = 0;
    let mut locations: DWORD = 0;
    sub_resource = (*texture).sub_resources.add(sub_resource_idx as usize)
        as *mut wined3d_texture_sub_resource;
    locations = (*sub_resource).locations;
    if ((*texture).resource.bind_flags & (WINED3D_BIND_DEPTH_STENCIL as DWORD)) != 0 {
        return FALSE as BOOL;
    }
    if (locations & (WINED3D_LOCATION_RB_MULTISAMPLE as DWORD)) != 0 {
        src_location = WINED3D_LOCATION_RB_MULTISAMPLE as DWORD;
    } else if (locations & (WINED3D_LOCATION_RB_RESOLVED as DWORD)) != 0 {
        src_location = WINED3D_LOCATION_RB_RESOLVED as DWORD;
    } else if (locations & (WINED3D_LOCATION_TEXTURE_SRGB as DWORD)) != 0 {
        src_location = WINED3D_LOCATION_TEXTURE_SRGB as DWORD;
    } else if (locations & (WINED3D_LOCATION_TEXTURE_RGB as DWORD)) != 0 {
        src_location = WINED3D_LOCATION_TEXTURE_RGB as DWORD;
    } else if (locations & (WINED3D_LOCATION_DRAWABLE as DWORD)) != 0 {
        src_location = WINED3D_LOCATION_DRAWABLE as DWORD;
    } else {
        src_location = WINED3D_LOCATION_TEXTURE_RGB as DWORD;
    }
    texture2d_blt_fbo(
        (*texture).resource.device,
        context,
        WINED3D_TEXF_POINT,
        texture,
        sub_resource_idx,
        src_location,
        &raw const rect,
        texture,
        sub_resource_idx,
        dst_location,
        &raw const rect,
        ::core::ptr::null::<wined3d_format>(),
    );
    return TRUE as BOOL;
}

unsafe extern "C" fn wined3d_texture_gl_load_texture(
    texture_gl: *mut wined3d_texture_gl,
    sub_resource_idx: ::core::ffi::c_uint,
    context_gl: *mut wined3d_context_gl,
    srgb: BOOL,
) -> BOOL {
    let mut width: ::core::ffi::c_uint = 0;
    let mut height: ::core::ffi::c_uint = 0;
    let mut level: ::core::ffi::c_uint = 0;
    let mut src_row_pitch: ::core::ffi::c_uint = 0;
    let mut src_slice_pitch: ::core::ffi::c_uint = 0;
    let mut dst_row_pitch: ::core::ffi::c_uint = 0;
    let mut dst_slice_pitch: ::core::ffi::c_uint = 0;
    let mut device: *mut wined3d_device = (*texture_gl).t.resource.device;
    let mut gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut conversion: *const wined3d_color_key_conversion =
        ::core::ptr::null::<wined3d_color_key_conversion>();
    let mut sub_resource: *mut wined3d_texture_sub_resource =
        ::core::ptr::null_mut::<wined3d_texture_sub_resource>();
    let mut format: *const wined3d_format = ::core::ptr::null::<wined3d_format>();
    let mut data: wined3d_bo_address = wined3d_bo_address {
        buffer_object: ::core::ptr::null_mut::<wined3d_bo>(),
        addr: ::core::ptr::null_mut::<BYTE>(),
    };
    let mut src_mem: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
    let mut dst_mem: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
    let mut src_box: wined3d_box = wined3d_box {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        front: 0,
        back: 0,
    };
    let mut dst_location: DWORD = 0;
    let mut depth: BOOL = 0;

    depth = ((*texture_gl).t.resource.bind_flags & WINED3D_BIND_DEPTH_STENCIL as ::core::ffi::c_uint) as BOOL;
    sub_resource = (*texture_gl)
        .t
        .sub_resources
        .offset(sub_resource_idx as isize) as *mut wined3d_texture_sub_resource
        as *mut wined3d_texture_sub_resource;
    level = sub_resource_idx.wrapping_rem((*texture_gl).t.level_count);
    wined3d_texture_get_level_box(&raw mut (*texture_gl).t, level, &raw mut src_box);

    if depth == 0
        && ((*sub_resource).locations & ((WINED3D_LOCATION_TEXTURE_SRGB | WINED3D_LOCATION_TEXTURE_RGB) as DWORD)) != 0
        && ((*texture_gl).t.resource.format_caps & WINED3D_FORMAT_CAP_FBO_ATTACHABLE_SRGB as ::core::ffi::c_uint) != 0
        && fbo_blitter_supported(
            WINED3D_BLIT_OP_COLOR_BLIT,
            gl_info,
            &raw mut (*texture_gl).t.resource,
            WINED3D_LOCATION_TEXTURE_RGB as DWORD,
            &raw mut (*texture_gl).t.resource,
            WINED3D_LOCATION_TEXTURE_SRGB as DWORD,
        ) as ::core::ffi::c_int != 0
    {
        let mut src_rect: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };

        SetRect(
            &raw mut src_rect,
            src_box.left as ::core::ffi::c_int,
            src_box.top as ::core::ffi::c_int,
            src_box.right as ::core::ffi::c_int,
            src_box.bottom as ::core::ffi::c_int,
        );
        if srgb != 0 {
            texture2d_blt_fbo(
                device,
                &raw mut (*context_gl).c,
                WINED3D_TEXF_POINT,
                &raw mut (*texture_gl).t,
                sub_resource_idx,
                WINED3D_LOCATION_TEXTURE_RGB as DWORD,
                &raw mut src_rect,
                &raw mut (*texture_gl).t,
                sub_resource_idx,
                WINED3D_LOCATION_TEXTURE_SRGB as DWORD,
                &raw mut src_rect,
                ::core::ptr::null::<wined3d_format>(),
            );
        } else {
            texture2d_blt_fbo(
                device,
                &raw mut (*context_gl).c,
                WINED3D_TEXF_POINT,
                &raw mut (*texture_gl).t,
                sub_resource_idx,
                WINED3D_LOCATION_TEXTURE_SRGB as DWORD,
                &raw mut src_rect,
                &raw mut (*texture_gl).t,
                sub_resource_idx,
                WINED3D_LOCATION_TEXTURE_RGB as DWORD,
                &raw mut src_rect,
                ::core::ptr::null::<wined3d_format>(),
            );
        }

        return (TRUE) as BOOL;
    }

    if depth == 0
        && ((*sub_resource).locations & ((WINED3D_LOCATION_RB_MULTISAMPLE | WINED3D_LOCATION_RB_RESOLVED) as DWORD)) != 0
        && (srgb == 0
            || ((*texture_gl).t.resource.format_caps & WINED3D_FORMAT_CAP_FBO_ATTACHABLE_SRGB as ::core::ffi::c_uint) != 0)
    {
        let mut src_location: DWORD = (if (*sub_resource).locations
            & (WINED3D_LOCATION_RB_RESOLVED as DWORD)
            != 0
        {
            WINED3D_LOCATION_RB_RESOLVED
        } else {
            WINED3D_LOCATION_RB_MULTISAMPLE
        }) as DWORD;
        let mut src_rect_0: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };

        SetRect(
            &raw mut src_rect_0,
            src_box.left as ::core::ffi::c_int,
            src_box.top as ::core::ffi::c_int,
            src_box.right as ::core::ffi::c_int,
            src_box.bottom as ::core::ffi::c_int,
        );
        dst_location = (if srgb != 0 {
            WINED3D_LOCATION_TEXTURE_SRGB
        } else {
            WINED3D_LOCATION_TEXTURE_RGB
        }) as DWORD;
        if fbo_blitter_supported(
            WINED3D_BLIT_OP_COLOR_BLIT,
            gl_info,
            &raw mut (*texture_gl).t.resource,
            src_location,
            &raw mut (*texture_gl).t.resource,
            dst_location,
        ) {
            texture2d_blt_fbo(
                device,
                &raw mut (*context_gl).c,
                WINED3D_TEXF_POINT,
                &raw mut (*texture_gl).t,
                sub_resource_idx,
                src_location,
                &raw mut src_rect_0,
                &raw mut (*texture_gl).t,
                sub_resource_idx,
                dst_location,
                &raw mut src_rect_0,
                ::core::ptr::null::<wined3d_format>(),
            );
        }

        return (TRUE) as BOOL;
    }

    /* Upload from system memory */

    if srgb != 0 {
        dst_location = WINED3D_LOCATION_TEXTURE_SRGB as DWORD;
        if ((*sub_resource).locations
            & ((WINED3D_LOCATION_TEXTURE_RGB as DWORD) | ((*texture_gl).t.resource.map_binding as DWORD)))
            == WINED3D_LOCATION_TEXTURE_RGB as DWORD
        {
            WINE_FIXME_!(__wine_dbch_d3d_perf, "Downloading RGB texture %p, %u to reload it as sRGB.\n", texture_gl, sub_resource_idx);
            wined3d_texture_load_location(
                &raw mut (*texture_gl).t,
                sub_resource_idx,
                &raw mut (*context_gl).c,
                (*texture_gl).t.resource.map_binding as uint32_t,
            );
        }
    } else {
        dst_location = WINED3D_LOCATION_TEXTURE_RGB as DWORD;
        if ((*sub_resource).locations
            & ((WINED3D_LOCATION_TEXTURE_SRGB as DWORD) | ((*texture_gl).t.resource.map_binding as DWORD)))
            == WINED3D_LOCATION_TEXTURE_SRGB as DWORD
        {
            WINE_FIXME_!(__wine_dbch_d3d_perf, "Downloading sRGB texture %p, %u to reload it as RGB.\n", texture_gl, sub_resource_idx);
            wined3d_texture_load_location(
                &raw mut (*texture_gl).t,
                sub_resource_idx,
                &raw mut (*context_gl).c,
                (*texture_gl).t.resource.map_binding as uint32_t,
            );
        }
    }

    if ((*sub_resource).locations as uint32_t & wined3d_texture_sysmem_locations) == 0 {
        WARN!("Trying to load a texture from sysmem, but no simple location is valid.\n");
        /* Lets hope we get it from somewhere... */
        wined3d_texture_load_location(
            &raw mut (*texture_gl).t,
            sub_resource_idx,
            &raw mut (*context_gl).c,
            WINED3D_LOCATION_SYSMEM as uint32_t,
        );
    }

    wined3d_texture_get_pitch(
        &raw mut (*texture_gl).t,
        level,
        &raw mut src_row_pitch,
        &raw mut src_slice_pitch,
    );

    format = (*texture_gl).t.resource.format;
    conversion = wined3d_format_get_color_key_conversion(&raw mut (*texture_gl).t, TRUE as BOOL);
    if !conversion.is_null() {
        format = wined3d_get_format(
            (*device).adapter,
            (*conversion).dst_format,
            (*texture_gl).t.resource.bind_flags,
        );
    }

    /* Don't use PBOs for converted surfaces. During PBO conversion we look at
     * WINED3D_TEXTURE_CONVERTED but it isn't set (yet) in all cases it is
     * getting called. */
    if !conversion.is_null() && !(*sub_resource).bo.is_null() {
        TRACE!("Removing the pbo attached to texture %p, %u.\n", texture_gl, sub_resource_idx);
        wined3d_texture_load_location(
            &raw mut (*texture_gl).t,
            sub_resource_idx,
            &raw mut (*context_gl).c,
            WINED3D_LOCATION_SYSMEM as uint32_t,
        );
        wined3d_texture_set_map_binding(&raw mut (*texture_gl).t, WINED3D_LOCATION_SYSMEM as DWORD);
    }

    wined3d_texture_get_memory(
        &raw mut (*texture_gl).t,
        sub_resource_idx,
        &raw mut (*context_gl).c,
        &raw mut data,
    );
    if !conversion.is_null() {
        width = src_box.right.wrapping_sub(src_box.left) as ::core::ffi::c_uint;
        height = src_box.bottom.wrapping_sub(src_box.top) as ::core::ffi::c_uint;
        wined3d_format_calculate_pitch(
            format,
            (*device).surface_alignment as ::core::ffi::c_uint,
            width,
            height,
            &raw mut dst_row_pitch,
            &raw mut dst_slice_pitch,
        );

        src_mem = wined3d_context_gl_map_bo_address(
            context_gl,
            &raw mut data,
            src_slice_pitch as size_t,
            WINED3D_MAP_READ as uint32_t,
        ) as *mut BYTE;
        dst_mem = malloc(dst_slice_pitch as size_t) as *mut BYTE;
        if dst_mem.is_null() {
            ERR!("Out of memory (%u).\n", dst_slice_pitch);
            return (FALSE) as BOOL;
        }
        (*conversion).convert.unwrap_unchecked()(
            src_mem,
            src_row_pitch,
            dst_mem,
            dst_row_pitch,
            width,
            height,
            &raw mut (*texture_gl).t.async_.gl_color_key,
        );
        src_row_pitch = dst_row_pitch;
        src_slice_pitch = dst_slice_pitch;
        wined3d_context_gl_unmap_bo_address(
            context_gl,
            &raw mut data,
            0 as ::core::ffi::c_uint,
            ::core::ptr::null::<wined3d_range>(),
        );

        data.buffer_object = core::ptr::null_mut();
        data.addr = dst_mem;
    }

    wined3d_texture_gl_upload_data(
        &raw mut (*context_gl).c,
        wined3d_const_bo_address(&raw mut data),
        format,
        &raw mut src_box,
        src_row_pitch,
        src_slice_pitch,
        &raw mut (*texture_gl).t,
        sub_resource_idx,
        dst_location as ::core::ffi::c_uint,
        0 as ::core::ffi::c_uint,
        0 as ::core::ffi::c_uint,
        0 as ::core::ffi::c_uint,
    );

    free(dst_mem as *mut ::core::ffi::c_void);

    return (TRUE) as BOOL;
}

unsafe extern "C" fn wined3d_texture_gl_prepare_location(texture: *mut wined3d_texture, sub_resource_idx: u32, context: *mut wined3d_context, location: u32) -> BOOL {
    let texture_gl: *mut wined3d_texture_gl = wined3d_texture_gl(texture);
    let context_gl: *mut wined3d_context_gl = wined3d_context_gl(context);

    match location {
        WINED3D_LOCATION_SYSMEM => {
            return if !(*(*texture).sub_resources.offset(sub_resource_idx as isize)).user_memory.is_null() {
                (TRUE as i32)
            } else {
                wined3d_resource_prepare_sysmem(&raw mut (*texture).resource)
            };
        }

        WINED3D_LOCATION_BUFFER => {
            wined3d_texture_gl_prepare_buffer_object(texture_gl, sub_resource_idx, context_gl);
            return (TRUE) as BOOL;
        }

        WINED3D_LOCATION_TEXTURE_RGB => {
            wined3d_texture_gl_prepare_texture(texture_gl, context_gl, (FALSE) as BOOL);
            return (TRUE) as BOOL;
        }

        WINED3D_LOCATION_TEXTURE_SRGB => {
            wined3d_texture_gl_prepare_texture(texture_gl, context_gl, (TRUE) as BOOL);
            return (TRUE) as BOOL;
        }

        WINED3D_LOCATION_DRAWABLE => {
            if (*texture).swapchain.is_null() {
                ERR!("Texture %p does not have a drawable.\n", texture);
            }
            return (TRUE) as BOOL;
        }

        WINED3D_LOCATION_RB_MULTISAMPLE => {
            wined3d_texture_gl_prepare_rb(texture_gl, (*context_gl).gl_info, (TRUE) as BOOL);
            return (TRUE) as BOOL;
        }

        WINED3D_LOCATION_RB_RESOLVED => {
            wined3d_texture_gl_prepare_rb(texture_gl, (*context_gl).gl_info, (FALSE) as BOOL);
            return (TRUE) as BOOL;
        }

        _ => {
            ERR!("Invalid location %s.\n", wined3d_debug_location(location));
            return (FALSE) as BOOL;
        }
    }
}

unsafe extern "C" fn use_ffp_clear(
    texture: *const wined3d_texture,
    location: ::core::ffi::c_uint,
) -> bool {
    if location == WINED3D_LOCATION_DRAWABLE as ::core::ffi::c_uint {
        return true;
    }

    if location == WINED3D_LOCATION_TEXTURE_RGB as ::core::ffi::c_uint
        && ((*texture).resource.format_caps
            & (WINED3D_FORMAT_CAP_FBO_ATTACHABLE as ::core::ffi::c_uint))
            == 0
    {
        return false;
    }
    if location == WINED3D_LOCATION_TEXTURE_SRGB as ::core::ffi::c_uint
        && ((*texture).resource.format_caps
            & (WINED3D_FORMAT_CAP_FBO_ATTACHABLE_SRGB as ::core::ffi::c_uint))
            == 0
    {
        return false;
    }

    return (location
        & ((WINED3D_LOCATION_RB_MULTISAMPLE
            | WINED3D_LOCATION_RB_RESOLVED
            | WINED3D_LOCATION_TEXTURE_RGB
            | WINED3D_LOCATION_TEXTURE_SRGB) as ::core::ffi::c_uint))
        != 0;
}

unsafe extern "C" fn wined3d_texture_gl_clear(
    texture: *mut wined3d_texture,
    sub_resource_idx: ::core::ffi::c_uint,
    context_gl: *mut wined3d_context_gl,
    location: ::core::ffi::c_uint,
) -> bool {
    let sub_resource: *mut wined3d_texture_sub_resource =
        (*texture).sub_resources.offset(sub_resource_idx as isize) as *mut wined3d_texture_sub_resource;
    let format: *const wined3d_format = (*texture).resource.format;
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut addr: wined3d_bo_address = wined3d_bo_address {
        buffer_object: core::ptr::null_mut(),
        addr: core::ptr::null_mut(),
    };

    if (*format).depth_size == 0 && (*format).stencil_size == 0 {
        if (*sub_resource).clear_value.colour.r != 0.
            || (*sub_resource).clear_value.colour.g != 0.
            || (*sub_resource).clear_value.colour.b != 0.
            || (*sub_resource).clear_value.colour.a != 0.
        {
            ERR!(
                "Unexpected color clear value r=%08e, g=%08e, b=%08e, a=%08e.\n",
                (*sub_resource).clear_value.colour.r as core::ffi::c_double,
                (*sub_resource).clear_value.colour.g as core::ffi::c_double,
                (*sub_resource).clear_value.colour.b as core::ffi::c_double,
                (*sub_resource).clear_value.colour.a as core::ffi::c_double
            );
        }
    } else {
        if (*format).depth_size != 0 && (*sub_resource).clear_value.__bindgen_anon_1.depth != 0. {
            ERR!(
                "Unexpected depth clear value %08e.\n",
                (*sub_resource).clear_value.__bindgen_anon_1.depth as core::ffi::c_double
            );
        }
        if (*format).stencil_size != 0 && (*sub_resource).clear_value.__bindgen_anon_1.stencil != 0 {
            ERR!(
                "Unexpected stencil clear value %x.\n",
                (*sub_resource).clear_value.__bindgen_anon_1.stencil
            );
        }
    }

    if use_ffp_clear(texture, location) {
        let mut clear_mask: GLbitfield = 0 as GLbitfield;

        context_gl_apply_texture_draw_state(context_gl, texture, sub_resource_idx, location);

        (*gl_info).gl_ops.gl.p_glDisable.unwrap_unchecked()(
            GL_SCISSOR_TEST as GLenum,
        );
        context_invalidate_state(
            &raw mut (*context_gl).c,
            STATE_RASTERIZER as ::core::ffi::c_uint,
        );

        if (*format).depth_size != 0 {
            (*gl_info).gl_ops.gl.p_glDepthMask.unwrap_unchecked()(
                GL_TRUE as GLboolean,
            );
            context_invalidate_state(
                &raw mut (*context_gl).c,
                STATE_DEPTH_STENCIL as ::core::ffi::c_uint,
            );

            if (*gl_info).supported[ARB_ES2_COMPATIBILITY as usize] != 0 {
                ((*gl_info).gl_ops.ext.p_glClearDepthf.unwrap_unchecked())(0.0f32);
            } else {
                (*gl_info).gl_ops.gl.p_glClearDepth.unwrap_unchecked()(0.0f64);
            }
            clear_mask |= GL_DEPTH_BUFFER_BIT as GLbitfield;
        }

        if (*format).stencil_size != 0 {
            if (*gl_info).supported[EXT_STENCIL_TWO_SIDE as usize] != 0 {
                (*gl_info).gl_ops.gl.p_glDisable.unwrap_unchecked()(
                    GL_STENCIL_TEST_TWO_SIDE_EXT as GLenum,
                );
            }
            (*gl_info).gl_ops.gl.p_glStencilMask.unwrap_unchecked()(
                !(0 as GLuint),
            );
            context_invalidate_state(
                &raw mut (*context_gl).c,
                STATE_DEPTH_STENCIL as ::core::ffi::c_uint,
            );
            (*gl_info).gl_ops.gl.p_glClearStencil.unwrap_unchecked()(
                0 as GLint,
            );
            clear_mask |= GL_STENCIL_BUFFER_BIT as GLbitfield;
        }

        if (*format).depth_size == 0 && (*format).stencil_size == 0 {
            (*gl_info).gl_ops.gl.p_glColorMask.unwrap_unchecked()(
                GL_TRUE as GLboolean,
                GL_TRUE as GLboolean,
                GL_TRUE as GLboolean,
                GL_TRUE as GLboolean,
            );
            context_invalidate_state(
                &raw mut (*context_gl).c,
                STATE_BLEND as ::core::ffi::c_uint,
            );
            (*gl_info).gl_ops.gl.p_glClearColor.unwrap_unchecked()(
                0.0f32, 0.0f32, 0.0f32, 0.0f32,
            );
            clear_mask |= GL_COLOR_BUFFER_BIT as GLbitfield;
        }

        (*gl_info).gl_ops.gl.p_glClear.unwrap_unchecked()(clear_mask);
        checkGLcall!(gl_info, "clear texture");

        wined3d_texture_validate_location(texture, sub_resource_idx, location);
        return true;
    }

    if wined3d_texture_prepare_location(
        texture,
        sub_resource_idx,
        &raw mut (*context_gl).c,
        WINED3D_LOCATION_SYSMEM as ::core::ffi::c_uint,
    ) == 0
    {
        return false;
    }
    wined3d_texture_get_bo_address(
        texture,
        sub_resource_idx,
        &raw mut addr,
        WINED3D_LOCATION_SYSMEM as uint32_t,
    );
    memset(
        addr.addr as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        (*sub_resource).size as size_t,
    );
    wined3d_texture_validate_location(
        texture,
        sub_resource_idx,
        WINED3D_LOCATION_SYSMEM as uint32_t,
    );
    return true;
}

unsafe extern "C" fn wined3d_texture_gl_load_location(
    texture: *mut wined3d_texture,
    sub_resource_idx: u32,
    context: *mut wined3d_context,
    location: u32,
) -> BOOL {
    let sub_resource: *mut wined3d_texture_sub_resource =
        (*texture).sub_resources.offset(sub_resource_idx as isize);
    let texture_gl: *mut wined3d_texture_gl = wined3d_texture_gl(texture);
    let context_gl: *mut wined3d_context_gl = wined3d_context_gl(context);

    if wined3d_texture_gl_prepare_location(texture, sub_resource_idx, context, location) == 0 {
        return (FALSE) as BOOL;
    }

    if (*sub_resource).locations & WINED3D_LOCATION_CLEARED != 0 {
        if !wined3d_texture_gl_clear(texture, sub_resource_idx, context_gl, location) {
            return (FALSE) as BOOL;
        }

        if (*sub_resource).locations & location != 0 {
            return (TRUE) as BOOL;
        }
    }

    match location {
        WINED3D_LOCATION_SYSMEM | WINED3D_LOCATION_BUFFER => {
            return wined3d_texture_gl_load_sysmem(texture_gl, sub_resource_idx, context_gl, location);
        }
        WINED3D_LOCATION_DRAWABLE => {
            return wined3d_texture_load_drawable(texture, sub_resource_idx, context);
        }
        WINED3D_LOCATION_RB_RESOLVED | WINED3D_LOCATION_RB_MULTISAMPLE => {
            return wined3d_texture_load_renderbuffer(texture, sub_resource_idx, context, location);
        }
        WINED3D_LOCATION_TEXTURE_RGB | WINED3D_LOCATION_TEXTURE_SRGB => {
            return wined3d_texture_gl_load_texture(
                texture_gl,
                sub_resource_idx,
                context_gl,
                (location == WINED3D_LOCATION_TEXTURE_SRGB) as BOOL,
            );
        }
        _ => {
            FIXME!(
                "Unhandled %s load from %s.\n",
                wined3d_debug_location(location),
                wined3d_debug_location((*sub_resource).locations)
            );
            return (FALSE) as BOOL;
        }
    }
}

unsafe extern "C" fn wined3d_texture_gl_unload_location(
    mut texture: *mut wined3d_texture,
    mut context: *mut wined3d_context,
    mut location: ::core::ffi::c_uint,
) {
    let mut texture_gl: *mut wined3d_texture_gl = wined3d_texture_gl(texture);
    let mut context_gl: *mut wined3d_context_gl = wined3d_context_gl(context);
    let mut entry: *mut wined3d_renderbuffer_entry =
        ::core::ptr::null_mut::<wined3d_renderbuffer_entry>();
    let mut entry2: *mut wined3d_renderbuffer_entry =
        ::core::ptr::null_mut::<wined3d_renderbuffer_entry>();
    let mut i: ::core::ffi::c_uint = 0;
    let mut sub_count: ::core::ffi::c_uint = 0;
    TRACE!(
        "texture %p, context %p, location %s.\n",
        texture,
        context,
        wined3d_debug_location(location as uint32_t)
    );
    match location {
        WINED3D_LOCATION_BUFFER => {
            sub_count = ((*texture).level_count as UINT).wrapping_mul((*texture).layer_count)
                as ::core::ffi::c_uint;
            i = 0 as ::core::ffi::c_uint;
            while i < sub_count {
                if !(*(*texture_gl).t.sub_resources.offset(i as isize))
                    .bo
                    .is_null()
                {
                    wined3d_texture_remove_buffer_object(&raw mut (*texture_gl).t, i, context_gl);
                }
                i = i.wrapping_add(1);
            }
        }
        WINED3D_LOCATION_TEXTURE_RGB => {
            if (*texture_gl).texture_rgb.name != 0 {
                gltexture_delete(
                    (*texture_gl).t.resource.device,
                    (*context_gl).gl_info,
                    &raw mut (*texture_gl).texture_rgb,
                );
            }
        }
        WINED3D_LOCATION_TEXTURE_SRGB => {
            if (*texture_gl).texture_srgb.name != 0 {
                gltexture_delete(
                    (*texture_gl).t.resource.device,
                    (*context_gl).gl_info,
                    &raw mut (*texture_gl).texture_srgb,
                );
            }
        }
        WINED3D_LOCATION_RB_MULTISAMPLE => {
            if (*texture_gl).rb_multisample != 0 {
                TRACE!(
                    "Deleting multisample renderbuffer %u.\n",
                    (*texture_gl).rb_multisample
                );
                context_gl_resource_released(
                    (*texture_gl).t.resource.device,
                    (*texture_gl).rb_multisample,
                    (TRUE) as BOOL,
                );
                (*(*context_gl).gl_info)
                    .fbo_ops
                    .glDeleteRenderbuffers
                    .unwrap_unchecked()(
                    1 as GLsizei,
                    &raw mut (*texture_gl).rb_multisample,
                );
                (*texture_gl).rb_multisample = 0 as GLuint;
            }
        }
        WINED3D_LOCATION_RB_RESOLVED => {
            entry = ((*texture_gl).renderbuffers.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(wined3d_renderbuffer_entry, entry)
                    as ::core::ffi::c_ulonglong as isize),
            ) as *mut wined3d_renderbuffer_entry;
            entry2 = ((*entry).entry.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(wined3d_renderbuffer_entry, entry)
                    as ::core::ffi::c_ulonglong as isize),
            ) as *mut wined3d_renderbuffer_entry;
            while &raw mut (*entry).entry != &raw mut (*texture_gl).renderbuffers {
                context_gl_resource_released(
                    (*texture_gl).t.resource.device,
                    (*entry).id,
                    (TRUE) as BOOL,
                );
                (*(*context_gl).gl_info)
                    .fbo_ops
                    .glDeleteRenderbuffers
                    .unwrap_unchecked()(
                    1 as GLsizei,
                    &raw mut (*entry).id,
                );
                list_remove(&raw mut (*entry).entry);
                free(entry as *mut ::core::ffi::c_void);
                entry = entry2;
                entry2 = ((*entry).entry.next as *mut ::core::ffi::c_char).offset(
                    -(core::mem::offset_of!(wined3d_renderbuffer_entry, entry)
                        as ::core::ffi::c_ulonglong as isize),
                ) as *mut wined3d_renderbuffer_entry;
            }
            list_init(&raw mut (*texture_gl).renderbuffers);
            (*texture_gl).current_renderbuffer = ::core::ptr::null::<wined3d_renderbuffer_entry>();

            if (*texture_gl).rb_resolved != 0 {
                TRACE!(
                    "Deleting resolved renderbuffer %u.\n",
                    (*texture_gl).rb_resolved
                );
                context_gl_resource_released(
                    (*texture_gl).t.resource.device,
                    (*texture_gl).rb_resolved,
                    (TRUE) as BOOL,
                );
                (*(*context_gl).gl_info)
                    .fbo_ops
                    .glDeleteRenderbuffers
                    .unwrap_unchecked()(
                    1 as GLsizei,
                    &raw mut (*texture_gl).rb_resolved,
                );
                (*texture_gl).rb_resolved = 0 as GLuint;
            }
        }
        _ => {
            ERR!(
                "Unhandled location %s.\n",
                wined3d_debug_location(location as uint32_t)
            );
        }
    };
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_from_resource(resource: *mut wined3d_resource) -> *mut wined3d_texture {
    texture_from_resource(resource)
}

unsafe extern "C" fn texture_resource_incref(resource: *mut wined3d_resource) -> ULONG {
    return wined3d_texture_incref(texture_from_resource(resource));
}

unsafe extern "C" fn texture_resource_decref(resource: *mut wined3d_resource) -> ULONG {
    return wined3d_texture_decref(texture_from_resource(resource));
}

unsafe extern "C" fn texture_resource_preload(resource: *mut wined3d_resource) {
    let texture: *mut wined3d_texture = texture_from_resource(resource);
    let mut context: *mut wined3d_context;

    context = context_acquire((*resource).device, core::ptr::null_mut(), 0);
    wined3d_texture_load(texture, context, ((*texture).flags & WINED3D_TEXTURE_IS_SRGB) as BOOL);
    context_release(context);
}

unsafe extern "C" fn texture_resource_unload(resource: *mut wined3d_resource) {
    let mut texture: *mut wined3d_texture = texture_from_resource(resource);
    let mut device: *mut wined3d_device = (*resource).device;
    let mut location: ::core::ffi::c_uint = (*resource).map_binding as ::core::ffi::c_uint;
    let mut context: *mut wined3d_context = core::ptr::null_mut();
    let mut sub_count: ::core::ffi::c_uint;
    let mut i: ::core::ffi::c_uint;

    /* D3D is not initialised, so no GPU locations should currently exist.
     * Moreover, we may not be able to acquire a valid context. */
    if (*device).d3d_initialized() == 0 {
        return;
    }

    context = context_acquire(device, core::ptr::null_mut(), 0);

    if location == WINED3D_LOCATION_BUFFER {
        location = WINED3D_LOCATION_SYSMEM;
    }

    sub_count = ((*texture).level_count as ::core::ffi::c_uint)
        .wrapping_mul((*texture).layer_count as ::core::ffi::c_uint);
    i = 0;
    while i < sub_count {
        if ((*resource).access & WINED3D_RESOURCE_ACCESS_CPU) != 0
            && wined3d_texture_load_location(texture, i, context, location) != 0
        {
            wined3d_texture_invalidate_location(texture, i, !location);
        } else {
            if ((*resource).access & WINED3D_RESOURCE_ACCESS_CPU) != 0 {
                ERR!(
                    "Discarding %s %p sub-resource %u with resource access %s.\n",
                    debug_d3dresourcetype((*resource).type_),
                    resource,
                    i,
                    wined3d_debug_resource_access((*resource).access as u32)
                );
            }
            wined3d_texture_validate_location(texture, i, WINED3D_LOCATION_DISCARDED);
            wined3d_texture_invalidate_location(texture, i, !WINED3D_LOCATION_DISCARDED);
        }
        i = i.wrapping_add(1);
    }

    wined3d_texture_unload_location(texture, context, WINED3D_LOCATION_BUFFER);
    wined3d_texture_unload_location(texture, context, WINED3D_LOCATION_TEXTURE_RGB);
    wined3d_texture_unload_location(texture, context, WINED3D_LOCATION_TEXTURE_SRGB);
    wined3d_texture_unload_location(texture, context, WINED3D_LOCATION_RB_MULTISAMPLE);
    wined3d_texture_unload_location(texture, context, WINED3D_LOCATION_RB_RESOLVED);

    context_release(context);

    wined3d_texture_force_reload(texture);

    if (*texture).resource.bind_count != 0
        && ((*texture).resource.bind_flags & WINED3D_BIND_SHADER_RESOURCE) != 0
    {
        device_invalidate_state(device, (STATE_GRAPHICS_SHADER_RESOURCE_BINDING as u32));
    }

    wined3d_texture_set_dirty(texture);

    resource_unload(core::ptr::addr_of_mut!((*texture).resource));
}

unsafe extern "C" fn texture_resource_sub_resource_get_desc(
    resource: *mut wined3d_resource,
    sub_resource_idx: ::core::ffi::c_uint,
    desc: *mut wined3d_sub_resource_desc,
) -> HRESULT {
    let texture: *const wined3d_texture = texture_from_resource(resource);

    return wined3d_texture_get_sub_resource_desc(texture, sub_resource_idx, desc);
}

unsafe extern "C" fn texture_resource_sub_resource_get_map_pitch(
    resource: *mut wined3d_resource,
    sub_resource_idx: u32,
    row_pitch: *mut u32,
    slice_pitch: *mut u32,
) {
    let texture: *const wined3d_texture = texture_from_resource(resource);
    let level: u32 = sub_resource_idx % (*texture).level_count;

    if ((*resource).format_attrs & WINED3D_FORMAT_ATTR_BROKEN_PITCH) != 0 {
        *row_pitch = wined3d_texture_get_level_width(texture, level).wrapping_mul((*(*resource).format).byte_count);
        *slice_pitch = wined3d_texture_get_level_height(texture, level).wrapping_mul(*row_pitch);
    } else {
        wined3d_texture_get_pitch(texture, level, row_pitch, slice_pitch);
    }
}

unsafe extern "C" fn texture_resource_sub_resource_map(
    mut resource: *mut wined3d_resource,
    mut sub_resource_idx: ::core::ffi::c_uint,
    mut map_ptr: *mut *mut ::core::ffi::c_void,
    mut box_0: *const wined3d_box,
    mut flags: uint32_t,
) -> HRESULT {
    let mut sub_resource: *mut wined3d_texture_sub_resource = ::core::ptr::null_mut::<wined3d_texture_sub_resource>();
    let mut device: *mut wined3d_device = (*resource).device;
    let mut context: *mut wined3d_context = ::core::ptr::null_mut::<wined3d_context>();
    let mut texture: *mut wined3d_texture = ::core::ptr::null_mut::<wined3d_texture>();
    let mut data: wined3d_bo_address = wined3d_bo_address {
        buffer_object: ::core::ptr::null_mut::<wined3d_bo>(),
        addr: ::core::ptr::null_mut::<BYTE>(),
    };
    let mut texture_level: ::core::ffi::c_uint = 0;
    let mut base_memory: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
    let mut ret: BOOL = (TRUE as i32);

    TRACE!(
        b"resource %p, sub_resource_idx %u, map_ptr %p, box %s, flags %#x.\n\0" as *const u8 as *const ::core::ffi::c_char,
        resource,
        sub_resource_idx,
        map_ptr,
        debug_box(box_0),
        flags,
    );

    texture = texture_from_resource(resource);
    sub_resource = wined3d_texture_get_sub_resource(texture, sub_resource_idx);

    texture_level = sub_resource_idx % (*texture).level_count;

    if ((*texture).flags & WINED3D_TEXTURE_DC_IN_USE) != 0 {
        WARN!("DC is in use.\n");
        return WINED3DERR_INVALIDCALL;
    }

    if (*sub_resource).map_count != 0 {
        WARN!("Sub-resource is already mapped.\n");
        return WINED3DERR_INVALIDCALL;
    }

    context = context_acquire(device, ::core::ptr::null_mut::<wined3d_texture>(), 0);

    if (flags & WINED3D_MAP_DISCARD) != 0 {
        TRACE!(
            b"WINED3D_MAP_DISCARD flag passed, marking %s as up to date.\n\0" as *const u8 as *const ::core::ffi::c_char,
            wined3d_debug_location((*resource).map_binding as uint32_t),
        );
        ret = wined3d_texture_prepare_location(texture, sub_resource_idx, context, (*resource).map_binding as ::core::ffi::c_uint);
        if ret != 0 {
            wined3d_texture_validate_location(texture, sub_resource_idx, (*resource).map_binding as uint32_t);
        }
    } else {
        if ((*resource).usage & WINED3DUSAGE_DYNAMIC) != 0 {
            WINE_WARN_!(__wine_dbch_d3d_perf,"Mapping a dynamic texture without WINED3D_MAP_DISCARD.\n");
        }
        if texture_level == 0 {
            let mut i: ::core::ffi::c_uint = 0;

            i = 0;
            while i < (*texture).level_count {
                ret = wined3d_texture_load_location(
                    texture,
                    sub_resource_idx.wrapping_add(i),
                    context,
                    (*resource).map_binding as uint32_t,
                );
                if ret == 0 {
                    break;
                }
                i = i.wrapping_add(1);
            }
        } else {
            ret = wined3d_texture_load_location(texture, sub_resource_idx, context, (*resource).map_binding as uint32_t);
        }
    }

    if ret == 0 {
        ERR!("Failed to prepare location.\n");
        context_release(context);
        return E_OUTOFMEMORY;
    }

    /* We only record dirty regions for the top-most level. */
    if !(*texture).dirty_regions.is_null()
        && (flags & WINED3D_MAP_WRITE) != 0
        && (flags & WINED3D_MAP_NO_DIRTY_UPDATE) == 0
        && texture_level == 0
    {
        wined3d_texture_dirty_region_add(texture, sub_resource_idx.wrapping_div((*texture).level_count), box_0);
    }

    if (flags & WINED3D_MAP_WRITE) != 0 {
        if texture_level == 0 {
            let mut i_0: ::core::ffi::c_uint = 0;

            i_0 = 0;
            while i_0 < (*texture).level_count {
                wined3d_texture_invalidate_location(
                    texture,
                    sub_resource_idx.wrapping_add(i_0),
                    !((*resource).map_binding as ::core::ffi::c_int) as uint32_t,
                );
                i_0 = i_0.wrapping_add(1);
            }
        } else {
            wined3d_texture_invalidate_location(
                texture,
                sub_resource_idx,
                !((*resource).map_binding as ::core::ffi::c_int) as uint32_t,
            );
        }
    }

    wined3d_texture_get_bo_address(
        texture,
        sub_resource_idx,
        core::ptr::addr_of_mut!(data),
        (*resource).map_binding as uint32_t,
    );
    base_memory = wined3d_context_map_bo_address(
        context,
        core::ptr::addr_of_mut!(data),
        (*sub_resource).size as usize,
        flags,
    ) as *mut BYTE;
    (*sub_resource).map_flags = flags;

    TRACE!(
        b"Base memory pointer %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
        base_memory,
    );

    context_release(context);

    *map_ptr = resource_offset_map_pointer(resource, sub_resource_idx, base_memory as *mut uint8_t, box_0);

    if !(*texture).swapchain.is_null() && (*(*texture).swapchain).front_buffer == texture {
        let mut r: *mut RECT = core::ptr::addr_of_mut!((*(*texture).swapchain).front_buffer_update);

        SetRect(
            r as LPRECT,
            (*box_0).left as ::core::ffi::c_int,
            (*box_0).top as ::core::ffi::c_int,
            (*box_0).right as ::core::ffi::c_int,
            (*box_0).bottom as ::core::ffi::c_int,
        );

        TRACE!(
            b"Mapped front buffer %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
            wine_dbgstr_rect(r),
        );
    }

    (*resource).map_count += 1;
    (*sub_resource).map_count = (*sub_resource).map_count.wrapping_add(1);

    TRACE!(
        b"Returning memory %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
        *map_ptr,
    );

    return WINED3D_OK;
}

unsafe extern "C" fn texture_resource_sub_resource_unmap(
    mut resource: *mut wined3d_resource,
    mut sub_resource_idx: ::core::ffi::c_uint,
) -> HRESULT {
    let mut sub_resource: *mut wined3d_texture_sub_resource = core::ptr::null_mut();
    let mut device: *mut wined3d_device = (*resource).device;
    let mut context: *mut wined3d_context = core::ptr::null_mut();
    let mut texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut data: wined3d_bo_address = core::mem::zeroed();
    let mut range: wined3d_range = core::mem::zeroed();

    texture = texture_from_resource(resource);
    sub_resource = wined3d_texture_get_sub_resource(texture, sub_resource_idx);
    if sub_resource.is_null() {
        return E_INVALIDARG;
    }

    if (*sub_resource).map_count == 0 {
        WARN!("Trying to unmap unmapped sub-resource.\n");
        if ((*texture).flags & (WINED3D_TEXTURE_DC_IN_USE as uint32_t)) != 0 {
            return WINED3D_OK;
        }
        return WINEDDERR_NOTLOCKED;
    }

    context = context_acquire(
        device,
        core::ptr::null_mut::<wined3d_texture>(),
        0 as ::core::ffi::c_uint,
    );

    wined3d_texture_get_bo_address(
        texture,
        sub_resource_idx,
        &raw mut data,
        (*texture).resource.map_binding as uint32_t,
    );
    range.offset = 0 as ::core::ffi::c_uint;
    range.size = (*sub_resource).size;
    wined3d_context_unmap_bo_address(
        context,
        &raw mut data,
        (((*sub_resource).map_flags & (WINED3D_MAP_WRITE as uint32_t)) != 0)
            as ::core::ffi::c_int as ::core::ffi::c_uint,
        &raw mut range,
    );

    context_release(context);

    if !(*texture).swapchain.is_null() && (*(*texture).swapchain).front_buffer == texture {
        if ((*sub_resource).locations
            & ((WINED3D_LOCATION_DRAWABLE as DWORD) | (WINED3D_LOCATION_TEXTURE_RGB as DWORD)))
            == 0
        {
            (*(*(*texture).swapchain).swapchain_ops)
                .swapchain_frontbuffer_updated
                .unwrap_unchecked()((*texture).swapchain);
        }
    }

    (*sub_resource).map_count = (*sub_resource).map_count.wrapping_sub(1);
    (*resource).map_count -= 1;
    if (*resource).map_count == 0 && (*texture).update_map_binding != 0 {
        wined3d_texture_update_map_binding(texture);
    }

    return WINED3D_OK;
}

unsafe extern "C" fn wined3d_texture_init(
    texture: *mut wined3d_texture,
    desc: *const wined3d_resource_desc,
    layer_count: ::core::ffi::c_uint,
    level_count: ::core::ffi::c_uint,
    flags: uint32_t,
    device: *mut wined3d_device,
    parent: *mut ::core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    sub_resources: *mut ::core::ffi::c_void,
    texture_ops: *const wined3d_texture_ops,
) -> HRESULT {
    let d3d_info: *const wined3d_d3d_info = core::ptr::addr_of!((*(*device).adapter).d3d_info);
    let device_parent: *mut wined3d_device_parent = (*device).device_parent;
    let mut sub_count: ::core::ffi::c_uint = 0;
    let mut i: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_uint = 0;
    let mut size: ::core::ffi::c_uint = 0;
    let mut offset: ::core::ffi::c_uint = 0;
    let mut format: *const wined3d_format = core::ptr::null::<wined3d_format>();
    let mut hr: HRESULT = 0;

    if (*desc).width == 0 || (*desc).height == 0 || (*desc).depth == 0 {
        return WINED3DERR_INVALIDCALL;
    }

    if (*desc).resource_type as ::core::ffi::c_uint == WINED3D_RTYPE_TEXTURE_3D as ::core::ffi::c_uint
        && layer_count != 1
    {
        ERR!("Invalid layer count for volume texture.\n");
        return E_INVALIDARG;
    }

    (*texture).sub_resources = sub_resources as *mut wined3d_texture_sub_resource;

    /* TODO: It should only be possible to create textures for formats
     * that are reported as supported. */
    if WINED3DFMT_UNKNOWN as ::core::ffi::c_uint >= (*desc).format as ::core::ffi::c_uint {
        WARN!("Texture cannot be created with a format of WINED3DFMT_UNKNOWN.\n");
        return WINED3DERR_INVALIDCALL;
    }
    format = wined3d_get_format((*device).adapter, (*desc).format, (*desc).bind_flags);

    if ((*desc).usage & WINED3DUSAGE_DYNAMIC as ::core::ffi::c_uint) != 0
        && ((*desc).usage & (WINED3DUSAGE_MANAGED | WINED3DUSAGE_SCRATCH) as ::core::ffi::c_uint) != 0
    {
        WARN!("Attempted to create a dynamic texture with usage %s.\n", debug_d3dusage((*desc).usage));
        return WINED3DERR_INVALIDCALL;
    }

    if (((*desc).width & (*desc).width.wrapping_sub(1 as ::core::ffi::c_uint)) != 0
        || ((*desc).height & (*desc).height.wrapping_sub(1 as ::core::ffi::c_uint)) != 0
        || ((*desc).depth & (*desc).depth.wrapping_sub(1 as ::core::ffi::c_uint)) != 0)
        && (*d3d_info).unconditional_npot() == 0
    {
        /* level_count == 0 returns an error as well. */
        if level_count != 1 as ::core::ffi::c_uint
            || layer_count != 1 as ::core::ffi::c_uint
            || (*desc).resource_type as ::core::ffi::c_uint == WINED3D_RTYPE_TEXTURE_3D as ::core::ffi::c_uint
        {
            if (*desc).usage & WINED3DUSAGE_SCRATCH as ::core::ffi::c_uint == 0 {
                WARN!(
                    "Attempted to create a mipmapped/cube/array/volume NPOT texture without unconditional NPOT support.\n"
                );
                return WINED3DERR_INVALIDCALL;
            }

            WARN!("Creating a scratch mipmapped/cube/array NPOT texture despite lack of HW support.\n");
        }
        (*texture).flags |= WINED3D_TEXTURE_COND_NP2 as uint32_t;
    }

    if ((*desc).width > (*d3d_info).limits.texture_size
        || (*desc).height > (*d3d_info).limits.texture_size)
        && (*desc).bind_flags & WINED3D_BIND_SHADER_RESOURCE as ::core::ffi::c_uint != 0
    {
        /* One of four options:
         * 1: Scale the texture. (Any texture ops would require the texture to
         *    be scaled which is potentially slow.)
         * 2: Set the texture to the maximum size (bad idea).
         * 3: WARN and return WINED3DERR_NOTAVAILABLE.
         * 4: Create the surface, but allow it to be used only for DirectDraw
         *    Blts. Some apps (e.g. Swat 3) create textures with a height of
         *    16 and a width > 3000 and blt 16x16 letter areas from them to
         *    the render target. */
        if (*desc).access & WINED3D_RESOURCE_ACCESS_GPU as ::core::ffi::c_uint != 0 {
            WARN!("Dimensions (%ux%u) exceed the maximum texture size.\n", (*desc).width, (*desc).height);
            return WINED3DERR_NOTAVAILABLE;
        }

        /* We should never use this surface in combination with OpenGL. */
    }

    i = 0;
    while i < layer_count {
        j = 0;
        while j < level_count {
            let idx: ::core::ffi::c_uint = i.wrapping_mul(level_count).wrapping_add(j);

            size = wined3d_format_calculate_size(
                format,
                (*device).surface_alignment as UINT,
                max!(1, (*desc).width >> j) as UINT,
                max!(1, (*desc).height >> j) as UINT,
                max!(1, (*desc).depth >> j) as UINT,
            ) as ::core::ffi::c_uint;
            (*(*texture).sub_resources.offset(idx as isize)).offset = offset;
            (*(*texture).sub_resources.offset(idx as isize)).size = size;
            offset = offset.wrapping_add(size);
            j = j.wrapping_add(1);
        }
        offset = offset
            .wrapping_add((RESOURCE_ALIGNMENT - 1) as ::core::ffi::c_uint)
            & !(RESOURCE_ALIGNMENT - 1) as ::core::ffi::c_uint;
        i = i.wrapping_add(1);
    }

    if offset == WINED3DERR_INVALIDCALL as ::core::ffi::c_uint {
        return WINED3DERR_INVALIDCALL;
    }

    /* Ensure the last mip-level is at least large enough to hold a single
     * compressed block. It is questionable how useful these mip-levels are to
     * the application with "broken pitch" formats, but we want to avoid
     * memory corruption when loading textures into WINED3D_LOCATION_SYSMEM. */
    if (*format).attrs & WINED3D_FORMAT_ATTR_BROKEN_PITCH as ::core::ffi::c_uint != 0 {
        let mut min_size: ::core::ffi::c_uint = 0;

        min_size = (*(*texture).sub_resources.offset(
            level_count
                .wrapping_mul(layer_count)
                .wrapping_sub(1 as ::core::ffi::c_uint) as isize,
        ))
        .offset
        .wrapping_add((*format).block_byte_count as ::core::ffi::c_uint);
        min_size = min_size
            .wrapping_add((RESOURCE_ALIGNMENT - 1) as ::core::ffi::c_uint)
            & !(RESOURCE_ALIGNMENT - 1) as ::core::ffi::c_uint;
        if min_size > offset {
            offset = min_size;
        }
    }

    hr = resource_init(
        core::ptr::addr_of_mut!((*texture).resource),
        device,
        (*desc).resource_type,
        format,
        (*desc).multisample_type,
        (*desc).multisample_quality,
        (*desc).usage,
        (*desc).bind_flags,
        (*desc).access,
        (*desc).width,
        (*desc).height,
        (*desc).depth,
        offset,
        parent,
        parent_ops,
        core::ptr::addr_of!(texture_resource_ops),
    );
    if hr < 0 {
        static mut once: ::core::ffi::c_uint = 0;

        /* DXTn 3D textures are not supported. Do not write the ERR for them. */
        if ((*desc).format as ::core::ffi::c_uint == WINED3DFMT_DXT1 as ::core::ffi::c_uint
            || (*desc).format as ::core::ffi::c_uint == WINED3DFMT_DXT2 as ::core::ffi::c_uint
            || (*desc).format as ::core::ffi::c_uint == WINED3DFMT_DXT3 as ::core::ffi::c_uint
            || (*desc).format as ::core::ffi::c_uint == WINED3DFMT_DXT4 as ::core::ffi::c_uint
            || (*desc).format as ::core::ffi::c_uint == WINED3DFMT_DXT5 as ::core::ffi::c_uint)
            && (*format).caps[WINED3D_GL_RES_TYPE_TEX_2D as usize] & WINED3D_FORMAT_CAP_TEXTURE as ::core::ffi::c_uint == 0
            && (*desc).resource_type as ::core::ffi::c_uint != WINED3D_RTYPE_TEXTURE_3D as ::core::ffi::c_uint
            && {
                let fresh20 = once;
                once = once.wrapping_add(1);
                fresh20 == 0
            }
        {
            WINE_ERR_!(__wine_dbch_winediag, "The application tried to create a DXTn texture, but the driver does not support them.\n");
        }

        WARN!("Failed to initialize resource, returning %#lx\n", hr);
        return hr;
    }
    wined3d_resource_update_draw_binding(core::ptr::addr_of_mut!((*texture).resource));

    (*texture).texture_ops = texture_ops;

    (*texture).layer_count = layer_count as UINT;
    (*texture).level_count = level_count;
    (*texture).lod = 0 as ::core::ffi::c_uint;
    (*texture).flags |= WINED3D_TEXTURE_DOWNLOADABLE as uint32_t;
    if flags & WINED3D_TEXTURE_CREATE_GET_DC_LENIENT as uint32_t != 0 {
        (*texture).flags |= WINED3D_TEXTURE_GET_DC_LENIENT as uint32_t;
        (*texture).resource.set_pin_sysmem((1 as uint32_t) as _);
    }
    if flags & (WINED3D_TEXTURE_CREATE_GET_DC | WINED3D_TEXTURE_CREATE_GET_DC_LENIENT) as uint32_t != 0 {
        (*texture).flags |= WINED3D_TEXTURE_GET_DC as uint32_t;
    }
    if flags & WINED3D_TEXTURE_CREATE_DISCARD as uint32_t != 0 {
        (*texture).flags |= WINED3D_TEXTURE_DISCARD as uint32_t;
    }
    if flags & WINED3D_TEXTURE_CREATE_GENERATE_MIPMAPS as uint32_t != 0 {
        if (*texture).resource.format_caps & WINED3D_FORMAT_CAP_GEN_MIPMAP as ::core::ffi::c_uint == 0 {
            WARN!(
                "Format doesn't support mipmaps generation, ignoring WINED3D_TEXTURE_CREATE_GENERATE_MIPMAPS flag.\n"
            );
        } else {
            (*texture).flags |= WINED3D_TEXTURE_GENERATE_MIPMAPS as uint32_t;
        }
    }

    if flags & WINED3D_TEXTURE_CREATE_RECORD_DIRTY_REGIONS as uint32_t != 0 {
        (*texture).dirty_regions = calloc(
            ((*texture).layer_count as size_t) as _,
            (core::mem::size_of::<wined3d_dirty_regions>() as size_t) as _,
        ) as *mut wined3d_texture_wined3d_dirty_regions;
        if (*texture).dirty_regions.is_null() {
            wined3d_texture_cleanup_sync(texture);
            return E_OUTOFMEMORY;
        }
        i = 0;
        while i < (*texture).layer_count {
            wined3d_texture_dirty_region_add(texture, i, core::ptr::null::<wined3d_box>());
            i = i.wrapping_add(1);
        }
    }

    if wined3d_texture_use_pbo(texture, d3d_info) != 0 {
        (*texture).resource.map_binding = WINED3D_LOCATION_BUFFER as WORD;
    }

    sub_count = level_count.wrapping_mul(layer_count);
    if sub_count.wrapping_div(layer_count) != level_count {
        wined3d_texture_cleanup_sync(texture);
        return E_OUTOFMEMORY;
    }

    if (*desc).usage & WINED3DUSAGE_OVERLAY as ::core::ffi::c_uint != 0 {
        (*texture).overlay_info = calloc(
            (sub_count as size_t) as _,
            (core::mem::size_of::<wined3d_overlay_info>() as size_t) as _,
        ) as *mut wined3d_texture_wined3d_overlay_info;
        if (*texture).overlay_info.is_null() {
            wined3d_texture_cleanup_sync(texture);
            return E_OUTOFMEMORY;
        }

        i = 0;
        while i < sub_count {
            list_init(core::ptr::addr_of_mut!((*(*texture).overlay_info.offset(i as isize)).entry));
            list_init(core::ptr::addr_of_mut!((*(*texture).overlay_info.offset(i as isize)).overlays));
            i = i.wrapping_add(1);
        }
    }

    /* Generate all sub-resources. */
    i = 0;
    while i < sub_count {
        let mut sub_resource: *mut wined3d_texture_sub_resource =
            core::ptr::null_mut::<wined3d_texture_sub_resource>();

        sub_resource = (*texture).sub_resources.offset(i as isize) as *mut wined3d_texture_sub_resource;
        (*sub_resource).locations = WINED3D_LOCATION_CLEARED as DWORD;

        hr = (*(*device_parent).ops)
            .texture_sub_resource_created
            .unwrap_unchecked()(
            device_parent,
            (*desc).resource_type,
            texture,
            i,
            core::ptr::addr_of_mut!((*sub_resource).parent),
            core::ptr::addr_of_mut!((*sub_resource).parent_ops),
        );
        if hr < 0 {
            WARN!("Failed to create sub-resource parent, hr %#lx.\n", hr);
            (*sub_resource).parent = core::ptr::null_mut();
            wined3d_texture_cleanup_sync(texture);
            return hr;
        }

        if (*desc).usage & WINED3DUSAGE_OWNDC as ::core::ffi::c_uint != 0 {
            let mut texture_idx: wined3d_texture_idx = wined3d_texture_idx {
                texture: texture,
                sub_resource_idx: i,
            };

            wined3d_cs_init_object(
                (*device).cs as *mut wined3d_cs,
                Some(wined3d_texture_create_dc as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
                core::ptr::addr_of_mut!(texture_idx) as *mut ::core::ffi::c_void,
            );
            wined3d_cs_finish((*device).cs as *mut wined3d_cs, WINED3D_CS_QUEUE_DEFAULT);
            if (*texture).dc_info.is_null() || (*(*texture).dc_info.offset(i as isize)).dc.is_null() {
                wined3d_texture_cleanup_sync(texture);
                return WINED3DERR_INVALIDCALL;
            }
        }
        i = i.wrapping_add(1);
    }

    return WINED3D_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_blt(
    context: *mut wined3d_device_context,
    dst_texture: *mut wined3d_texture,
    dst_sub_resource_idx: ::core::ffi::c_uint,
    dst_rect: *const RECT,
    src_texture: *mut wined3d_texture,
    src_sub_resource_idx: ::core::ffi::c_uint,
    src_rect: *const RECT,
    flags: ::core::ffi::c_uint,
    fx: *const wined3d_blt_fx,
    filter: wined3d_texture_filter_type,
) -> HRESULT {
    let mut src_box: wined3d_box = wined3d_box {
        left: (*src_rect).left as UINT,
        top: (*src_rect).top as UINT,
        right: (*src_rect).right as UINT,
        bottom: (*src_rect).bottom as UINT,
        front: 0,
        back: 1,
    };
    let mut dst_box: wined3d_box = wined3d_box {
        left: (*dst_rect).left as UINT,
        top: (*dst_rect).top as UINT,
        right: (*dst_rect).right as UINT,
        bottom: (*dst_rect).bottom as UINT,
        front: 0,
        back: 1,
    };
    let mut hr: HRESULT;

    TRACE!(
        "context %p, dst_texture %p, dst_sub_resource_idx %u, dst_rect %s, src_texture %p, src_sub_resource_idx %u, src_rect %s, flags %#x, fx %p, filter %s.\n",
        context,
        dst_texture,
        dst_sub_resource_idx,
        wine_dbgstr_rect(dst_rect),
        src_texture,
        src_sub_resource_idx,
        wine_dbgstr_rect(src_rect),
        flags,
        fx,
        debug_d3dtexturefiltertype(filter)
    );

    if !wined3d_texture_validate_sub_resource_idx(dst_texture, dst_sub_resource_idx)
        || (*dst_texture).resource.type_ as ::core::ffi::c_uint
            != WINED3D_RTYPE_TEXTURE_2D as ::core::ffi::c_uint
    {
        return WINED3DERR_INVALIDCALL;
    }

    if !wined3d_texture_validate_sub_resource_idx(src_texture, src_sub_resource_idx)
        || (*src_texture).resource.type_ as ::core::ffi::c_uint
            != WINED3D_RTYPE_TEXTURE_2D as ::core::ffi::c_uint
    {
        return WINED3DERR_INVALIDCALL;
    }

    if filter as ::core::ffi::c_uint != WINED3D_TEXF_NONE as ::core::ffi::c_uint
        && filter as ::core::ffi::c_uint != WINED3D_TEXF_POINT as ::core::ffi::c_uint
        && filter as ::core::ffi::c_uint != WINED3D_TEXF_LINEAR as ::core::ffi::c_uint
    {
        return WINED3DERR_INVALIDCALL;
    }

    hr = wined3d_resource_check_box_dimensions(
        core::ptr::addr_of_mut!((*dst_texture).resource),
        dst_sub_resource_idx,
        core::ptr::addr_of_mut!(dst_box),
    );
    if hr < 0 {
        return hr;
    }

    hr = wined3d_resource_check_box_dimensions(
        core::ptr::addr_of_mut!((*src_texture).resource),
        src_sub_resource_idx,
        core::ptr::addr_of_mut!(src_box),
    );
    if hr < 0 {
        return hr;
    }

    if (*(*dst_texture).sub_resources.offset(dst_sub_resource_idx as isize)).map_count != 0
        || (*(*src_texture).sub_resources.offset(src_sub_resource_idx as isize)).map_count != 0
    {
        WARN!("Sub-resource is busy, returning WINEDDERR_SURFACEBUSY.\n");
        return WINEDDERR_SURFACEBUSY;
    }

    if ((*(*src_texture).resource.format).depth_size == 0) as ::core::ffi::c_int
        != ((*(*dst_texture).resource.format).depth_size == 0) as ::core::ffi::c_int
        || ((*(*src_texture).resource.format).stencil_size == 0) as ::core::ffi::c_int
            != ((*(*dst_texture).resource.format).stencil_size == 0) as ::core::ffi::c_int
    {
        WARN!("Rejecting depth/stencil blit between incompatible formats.\n");
        return WINED3DERR_INVALIDCALL;
    }

    if (*dst_texture).resource.device != (*src_texture).resource.device {
        FIXME!("Rejecting cross-device blit.\n");
        return E_NOTIMPL;
    }

    wined3d_device_context_emit_blt_sub_resource(
        context,
        core::ptr::addr_of_mut!((*dst_texture).resource),
        dst_sub_resource_idx,
        core::ptr::addr_of_mut!(dst_box),
        core::ptr::addr_of_mut!((*src_texture).resource),
        src_sub_resource_idx,
        core::ptr::addr_of_mut!(src_box),
        flags,
        fx,
        filter,
    );

    if !(*dst_texture).dirty_regions.is_null() {
        wined3d_texture_add_dirty_region(
            dst_texture,
            dst_sub_resource_idx as UINT,
            core::ptr::addr_of_mut!(dst_box),
        );
    }

    return WINED3D_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_get_overlay_position(
    texture: *const wined3d_texture,
    sub_resource_idx: ::core::ffi::c_uint,
    x: *mut LONG,
    y: *mut LONG,
) -> HRESULT {
    let mut overlay: *mut wined3d_overlay_info = core::ptr::null_mut();

    TRACE!(
        "texture %p, sub_resource_idx %u, x %p, y %p.\n",
        texture,
        sub_resource_idx,
        x,
        y
    );

    if (*texture).resource.usage & WINED3DUSAGE_OVERLAY as uint32_t == 0
        || !wined3d_texture_validate_sub_resource_idx(texture, sub_resource_idx)
    {
        return WINEDDERR_NOTAOVERLAYSURFACE;
    }

    overlay = (*texture).overlay_info.offset(sub_resource_idx as isize) as *mut wined3d_overlay_info
        as *mut wined3d_overlay_info;
    if (*overlay).dst_texture.is_null() {
        TRACE!("Overlay not visible.\n");
        *x = 0 as LONG;
        *y = 0 as LONG;
        return WINEDDERR_OVERLAYNOTVISIBLE;
    }

    *x = (*overlay).dst_rect.left;
    *y = (*overlay).dst_rect.top;

    TRACE!("Returning position %ld, %ld.\n", *x, *y);

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_set_overlay_position(
    texture: *mut wined3d_texture,
    sub_resource_idx: core::ffi::c_uint,
    x: LONG,
    y: LONG,
) -> HRESULT {
    let mut overlay: *mut wined3d_texture_wined3d_overlay_info = core::ptr::null_mut();
    let mut w: LONG;
    let mut h: LONG;

    if ((*texture).resource.usage & WINED3DUSAGE_OVERLAY) == 0
        || !wined3d_texture_validate_sub_resource_idx(texture, sub_resource_idx)
    {
        return WINEDDERR_NOTAOVERLAYSURFACE;
    }

    overlay = (*texture).overlay_info.offset(sub_resource_idx as isize);
    w = (*overlay).dst_rect.right - (*overlay).dst_rect.left;
    h = (*overlay).dst_rect.bottom - (*overlay).dst_rect.top;
    SetRect(
        core::ptr::addr_of_mut!((*overlay).dst_rect),
        x,
        y,
        x + w,
        y + h,
    );

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_update_overlay(
    mut texture: *mut wined3d_texture,
    mut sub_resource_idx: ::core::ffi::c_uint,
    mut src_rect: *const RECT,
    mut dst_texture: *mut wined3d_texture,
    mut dst_sub_resource_idx: ::core::ffi::c_uint,
    mut dst_rect: *const RECT,
    mut flags: uint32_t,
) -> HRESULT {
    let mut overlay: *mut wined3d_overlay_info = core::ptr::null_mut::<wined3d_overlay_info>();
    let mut level: ::core::ffi::c_uint = 0;
    let mut dst_level: ::core::ffi::c_uint = 0;

    if ((*texture).resource.usage & WINED3DUSAGE_OVERLAY as uint32_t) == 0
        || (*texture).resource.type_ as ::core::ffi::c_uint != WINED3D_RTYPE_TEXTURE_2D as ::core::ffi::c_uint
        || !wined3d_texture_validate_sub_resource_idx(texture, sub_resource_idx)
    {
        return WINEDDERR_NOTAOVERLAYSURFACE;
    }

    if dst_texture.is_null()
        || (*dst_texture).resource.type_ as ::core::ffi::c_uint != WINED3D_RTYPE_TEXTURE_2D as ::core::ffi::c_uint
        || !wined3d_texture_validate_sub_resource_idx(dst_texture, dst_sub_resource_idx)
    {
        return WINED3DERR_INVALIDCALL;
    }

    overlay = (*texture).overlay_info.offset(sub_resource_idx as isize) as *mut wined3d_overlay_info;

    level = sub_resource_idx.wrapping_rem((*texture).level_count);
    if !src_rect.is_null() {
        (*overlay).src_rect = *src_rect;
    } else {
        SetRect(
            core::ptr::addr_of_mut!((*overlay).src_rect),
            0,
            0,
            wined3d_texture_get_level_width(texture, level) as ::core::ffi::c_int,
            wined3d_texture_get_level_height(texture, level) as ::core::ffi::c_int,
        );
    }

    dst_level = dst_sub_resource_idx.wrapping_rem((*dst_texture).level_count);
    if !dst_rect.is_null() {
        (*overlay).dst_rect = *dst_rect;
    } else {
        SetRect(
            core::ptr::addr_of_mut!((*overlay).dst_rect),
            0,
            0,
            wined3d_texture_get_level_width(dst_texture, dst_level) as ::core::ffi::c_int,
            wined3d_texture_get_level_height(dst_texture, dst_level) as ::core::ffi::c_int,
        );
    }

    if !(*overlay).dst_texture.is_null()
        && ((*overlay).dst_texture != dst_texture
            || (*overlay).dst_sub_resource_idx != dst_sub_resource_idx
            || (flags & WINEDDOVER_HIDE as uint32_t) != 0)
    {
        (*overlay).dst_texture = core::ptr::null_mut::<wined3d_texture>();
        list_remove(core::ptr::addr_of_mut!((*overlay).entry));
    }

    if (flags & WINEDDOVER_SHOW as uint32_t) != 0 {
        if (*overlay).dst_texture != dst_texture
            || (*overlay).dst_sub_resource_idx != dst_sub_resource_idx
        {
            (*overlay).dst_texture = dst_texture;
            (*overlay).dst_sub_resource_idx = dst_sub_resource_idx;
            list_add_tail(
                core::ptr::addr_of_mut!(
                    (*(*texture).overlay_info.offset(dst_sub_resource_idx as isize)).overlays
                ),
                core::ptr::addr_of_mut!((*overlay).entry),
            );
        }
    } else if (flags & WINEDDOVER_HIDE as uint32_t) != 0 {
        /* Tests show that the rectangles are erased on hide. */
        SetRectEmpty(core::ptr::addr_of_mut!((*overlay).src_rect));
        SetRectEmpty(core::ptr::addr_of_mut!((*overlay).dst_rect));
        (*overlay).dst_texture = core::ptr::null_mut::<wined3d_texture>();
    }

    return WINED3D_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_get_swapchain(
    texture: *mut wined3d_texture,
) -> *mut wined3d_swapchain {
    (*texture).swapchain
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_get_sub_resource_parent(
    mut texture: *mut wined3d_texture,
    mut sub_resource_idx: ::core::ffi::c_uint,
) -> *mut ::core::ffi::c_void {
    WINE_TRACE!("texture %p, sub_resource_idx %u.\n", texture, sub_resource_idx);
    if !wined3d_texture_validate_sub_resource_idx(texture, sub_resource_idx) {
        return core::ptr::null_mut();
    }
    return (*(*texture).sub_resources.offset(sub_resource_idx as isize)).parent;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_set_sub_resource_parent(
    texture: *mut wined3d_texture,
    sub_resource_idx: core::ffi::c_uint,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
) {
    if !wined3d_texture_validate_sub_resource_idx(texture, sub_resource_idx) {
        return;
    }

    (*(*texture).sub_resources.offset(sub_resource_idx as isize)).parent = parent;
    (*(*texture).sub_resources.offset(sub_resource_idx as isize)).parent_ops = parent_ops;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_get_sub_resource_desc(
    texture: *const wined3d_texture,
    sub_resource_idx: ::core::ffi::c_uint,
    desc: *mut wined3d_sub_resource_desc,
) -> HRESULT {
    let mut resource: *const wined3d_resource = ::core::ptr::null::<wined3d_resource>();
    let mut level_idx: ::core::ffi::c_uint = 0;
    WINE_TRACE!("texture %p, sub_resource_idx %u, desc %p.\n", texture, sub_resource_idx, desc);
    if !wined3d_texture_validate_sub_resource_idx(texture, sub_resource_idx) {
        return WINED3DERR_INVALIDCALL;
    }
    resource = &raw const (*texture).resource;
    (*desc).format = (*(*resource).format).id;
    (*desc).multisample_type = (*resource).multisample_type;
    (*desc).multisample_quality = (*resource).multisample_quality as ::core::ffi::c_uint;
    (*desc).usage = (*resource).usage as ::core::ffi::c_uint;
    (*desc).bind_flags = (*resource).bind_flags;
    (*desc).access = (*resource).access;
    level_idx = sub_resource_idx.wrapping_rem((*texture).level_count);
    (*desc).width = wined3d_texture_get_level_width(texture, level_idx);
    (*desc).height = wined3d_texture_get_level_height(texture, level_idx);
    (*desc).depth = wined3d_texture_get_level_depth(texture, level_idx);
    (*desc).size = (*(*texture).sub_resources.offset(sub_resource_idx as isize)).size;
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_gl_init(mut texture_gl: *mut wined3d_texture_gl, mut device: *mut wined3d_device, mut desc: *const wined3d_resource_desc, mut layer_count: ::core::ffi::c_uint, mut level_count: ::core::ffi::c_uint, mut flags: uint32_t, mut parent: *mut ::core::ffi::c_void, mut parent_ops: *const wined3d_parent_ops) -> HRESULT {
    let gl_info: *const wined3d_gl_info = core::ptr::addr_of!(
        (*wined3d_adapter_gl((*device).adapter)).gl_info
    );
    let mut hr: HRESULT;

    if ((*desc).usage & WINED3DUSAGE_LEGACY_CUBEMAP) == 0
        && layer_count > 1
        && (*gl_info).supported[EXT_TEXTURE_ARRAY as usize] == 0
    {
        WARN!("OpenGL implementation does not support array textures.\n");
        return WINED3DERR_INVALIDCALL;
    }

    match (*desc).resource_type as ::core::ffi::c_uint {
        WINED3D_RTYPE_TEXTURE_1D => {
            if layer_count > 1 {
                (*texture_gl).target = GL_TEXTURE_1D_ARRAY as GLenum;
            } else {
                (*texture_gl).target = GL_TEXTURE_1D as GLenum;
            }
        }
        WINED3D_RTYPE_TEXTURE_2D => {
            if (*desc).usage & WINED3DUSAGE_LEGACY_CUBEMAP != 0 {
                (*texture_gl).target = GL_TEXTURE_CUBE_MAP_ARB as GLenum;
            } else if (*desc).multisample_type as ::core::ffi::c_uint != 0
                && (*gl_info).supported[ARB_TEXTURE_MULTISAMPLE as usize] != 0
            {
                if layer_count > 1 {
                    (*texture_gl).target = GL_TEXTURE_2D_MULTISAMPLE_ARRAY as GLenum;
                } else {
                    (*texture_gl).target = GL_TEXTURE_2D_MULTISAMPLE as GLenum;
                }
            } else {
                if layer_count > 1 {
                    (*texture_gl).target = GL_TEXTURE_2D_ARRAY as GLenum;
                } else {
                    (*texture_gl).target = GL_TEXTURE_2D as GLenum;
                }
            }
        }
        WINED3D_RTYPE_TEXTURE_3D => {
            if (*gl_info).supported[EXT_TEXTURE3D as usize] == 0 {
                WARN!("OpenGL implementation does not support 3D textures.\n");
                return WINED3DERR_INVALIDCALL;
            }
            (*texture_gl).target = GL_TEXTURE_3D as GLenum;
        }
        _ => {
            ERR!(
                "Invalid resource type %s.\n",
                debug_d3dresourcetype((*desc).resource_type)
            );
            return WINED3DERR_INVALIDCALL;
        }
    }

    list_init(core::ptr::addr_of_mut!((*texture_gl).renderbuffers));

    hr = wined3d_texture_init(
        core::ptr::addr_of_mut!((*texture_gl).t),
        desc,
        layer_count,
        level_count,
        flags,
        device,
        parent,
        parent_ops,
        texture_gl.offset(1 as isize) as *mut ::core::ffi::c_void,
        core::ptr::addr_of!(texture_gl_ops),
    );
    if hr < 0 {
        return hr;
    }

    if (*texture_gl).target == GL_TEXTURE_2D_MULTISAMPLE_ARRAY as GLenum
        || (*texture_gl).target == GL_TEXTURE_2D_MULTISAMPLE as GLenum
    {
        (*texture_gl).t.flags &= !(WINED3D_TEXTURE_DOWNLOADABLE) as uint32_t;
    }

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_create(
    mut device: *mut wined3d_device,
    mut desc: *const wined3d_resource_desc,
    mut layer_count: UINT,
    mut level_count: UINT,
    mut flags: uint32_t,
    mut data: *const wined3d_sub_resource_data,
    mut parent: *mut core::ffi::c_void,
    mut parent_ops: *const wined3d_parent_ops,
    mut texture: *mut *mut wined3d_texture,
) -> HRESULT {
    let mut sub_count: UINT = level_count.wrapping_mul(layer_count);
    let mut i: UINT;
    let mut hr: HRESULT;

    if layer_count == 0 {
        WARN!("Invalid layer count.\n");
        return E_INVALIDARG;
    }
    if ((*desc).usage & WINED3DUSAGE_LEGACY_CUBEMAP) != 0 && layer_count != 6 {
        ERR!("Invalid layer count %u for legacy cubemap.\n", layer_count);
        layer_count = 6;
    }

    if level_count == 0 {
        WARN!("Invalid level count.\n");
        return WINED3DERR_INVALIDCALL;
    }

    if (*desc).multisample_type as UINT != WINED3D_MULTISAMPLE_NONE {
        let format: *const wined3d_format =
            wined3d_get_format((*device).adapter, (*desc).format, (*desc).bind_flags);

        if (*desc).multisample_type as UINT == WINED3D_MULTISAMPLE_NON_MASKABLE
            && (*desc).multisample_quality
                >= wined3d_popcount((*format).multisample_types as UINT)
        {
            WARN!(
                "Unsupported quality level %u requested for WINED3D_MULTISAMPLE_NON_MASKABLE.\n",
                (*desc).multisample_quality
            );
            return WINED3DERR_NOTAVAILABLE;
        }
        if (*desc).multisample_type as UINT != WINED3D_MULTISAMPLE_NON_MASKABLE
            && (((*format).multisample_types as UINT
                & (1
                    << ((*desc).multisample_type as UINT).wrapping_sub(1)))
                == 0
                || ((*desc).multisample_quality != 0
                    && (*desc).multisample_quality != WINED3D_STANDARD_MULTISAMPLE_PATTERN))
        {
            WARN!(
                "Unsupported multisample type %u quality %u requested.\n",
                (*desc).multisample_type as UINT,
                (*desc).multisample_quality
            );
            return WINED3DERR_NOTAVAILABLE;
        }
    }

    if !data.is_null() {
        i = 0;
        while i < sub_count {
            if !(*data.offset(i as isize)).data.is_null() {
                i = i.wrapping_add(1);
                continue;
            }

            WARN!(
                "Invalid sub-resource data specified for sub-resource %u.\n",
                i
            );
            return E_INVALIDARG;
        }
    }

    hr = (*(*(*device).adapter).adapter_ops)
        .adapter_create_texture
        .unwrap_unchecked()(
        device,
        desc,
        layer_count,
        level_count,
        flags,
        parent,
        parent_ops,
        texture,
    );
    if hr < 0 {
        return hr;
    }

    /* FIXME: We'd like to avoid ever allocating system memory for the texture
     * in this case. */
    if !data.is_null() {
        let mut box_: wined3d_box = core::mem::zeroed();

        i = 0;
        while i < sub_count {
            wined3d_texture_get_level_box(
                *texture,
                i.wrapping_rem((**texture).level_count),
                core::ptr::addr_of_mut!(box_),
            );
            wined3d_device_context_emit_update_sub_resource(
                core::ptr::addr_of_mut!((*(*device).cs).c),
                core::ptr::addr_of_mut!((**texture).resource),
                i,
                core::ptr::addr_of_mut!(box_),
                (*data.offset(i as isize)).data,
                (*data.offset(i as isize)).row_pitch,
                (*data.offset(i as isize)).slice_pitch,
            );
            i = i.wrapping_add(1);
        }
    }

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_get_dc(
    mut texture: *mut wined3d_texture,
    mut sub_resource_idx: ::core::ffi::c_uint,
    mut dc: *mut HDC,
) -> HRESULT {
    let mut device: *mut wined3d_device = (*texture).resource.device;
    let mut sub_resource: *mut wined3d_texture_sub_resource =
        ::core::ptr::null_mut::<wined3d_texture_sub_resource>();
    let mut dc_info: *mut wined3d_dc_info = ::core::ptr::null_mut::<wined3d_dc_info>();

    if (*texture).flags & (WINED3D_TEXTURE_GET_DC as uint32_t) == 0 {
        WARN!("Texture does not support GetDC\n");
        /* Don't touch the DC */
        return WINED3DERR_INVALIDCALL;
    }

    sub_resource = wined3d_texture_get_sub_resource(texture, sub_resource_idx);
    if sub_resource.is_null() {
        return WINED3DERR_INVALIDCALL;
    }

    if (*texture).resource.type_ as ::core::ffi::c_uint
        != WINED3D_RTYPE_TEXTURE_2D as ::core::ffi::c_uint
    {
        WARN!(
            "Not supported on %s resources.\n",
            debug_d3dresourcetype((*texture).resource.type_)
        );
        return WINED3DERR_INVALIDCALL;
    }

    if (*texture).resource.map_count != 0
        && (*texture).flags & (WINED3D_TEXTURE_GET_DC_LENIENT as uint32_t) == 0
    {
        return WINED3DERR_INVALIDCALL;
    }

    dc_info = (*texture).dc_info as *mut wined3d_dc_info;
    if dc_info.is_null() || (*dc_info.offset(sub_resource_idx as isize)).dc.is_null() {
        let mut texture_idx: wined3d_texture_idx = wined3d_texture_idx {
            texture: texture,
            sub_resource_idx: sub_resource_idx,
        };
        wined3d_cs_init_object(
            (*device).cs as *mut wined3d_cs,
            Some(wined3d_texture_create_dc as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
            core::ptr::addr_of_mut!(texture_idx) as *mut ::core::ffi::c_void,
        );
        wined3d_cs_finish((*device).cs as *mut wined3d_cs, WINED3D_CS_QUEUE_DEFAULT);
        dc_info = (*texture).dc_info as *mut wined3d_dc_info;
        if dc_info.is_null() || (*dc_info.offset(sub_resource_idx as isize)).dc.is_null() {
            return WINED3DERR_INVALIDCALL;
        }
    }

    if (*texture).flags & (WINED3D_TEXTURE_GET_DC_LENIENT as uint32_t) == 0 {
        (*texture).flags |= WINED3D_TEXTURE_DC_IN_USE as uint32_t;
    }
    (*texture).resource.map_count += 1;
    (*sub_resource).map_count = (*sub_resource).map_count.wrapping_add(1);

    *dc = (*dc_info.offset(sub_resource_idx as isize)).dc;

    return WINED3D_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_release_dc(
    texture: *mut wined3d_texture,
    sub_resource_idx: ::core::ffi::c_uint,
    dc: HDC,
) -> HRESULT {
    let device: *mut wined3d_device = (*texture).resource.device;
    let mut sub_resource: *mut wined3d_texture_sub_resource = core::ptr::null_mut();
    let mut dc_info: *mut wined3d_dc_info = core::ptr::null_mut();

    TRACE!("texture %p, sub_resource_idx %u, dc %p.\n", texture, sub_resource_idx, dc);

    sub_resource = wined3d_texture_get_sub_resource(texture, sub_resource_idx);
    if sub_resource.is_null() {
        return WINED3DERR_INVALIDCALL;
    }

    if (*texture).resource.type_ as ::core::ffi::c_uint
        != WINED3D_RTYPE_TEXTURE_2D as ::core::ffi::c_uint
    {
        WARN!(
            "Not supported on %s resources.\n",
            debug_d3dresourcetype((*texture).resource.type_)
        );
        return WINED3DERR_INVALIDCALL;
    }

    if (*texture).flags & (WINED3D_TEXTURE_GET_DC_LENIENT | WINED3D_TEXTURE_DC_IN_USE) as uint32_t == 0 {
        return WINED3DERR_INVALIDCALL;
    }

    dc_info = (*texture).dc_info as *mut wined3d_dc_info;
    if dc_info.is_null() || (*dc_info.add(sub_resource_idx as usize)).dc != dc {
        WARN!(
            "Application tries to release invalid DC %p, sub-resource DC is %p.\n",
            dc,
            if !dc_info.is_null() {
                (*dc_info.add(sub_resource_idx as usize)).dc
            } else {
                core::ptr::null_mut::<HDC__>()
            }
        );
        return WINED3DERR_INVALIDCALL;
    }

    if (*texture).resource.usage & WINED3DUSAGE_OWNDC as uint32_t == 0 {
        let mut texture_idx: wined3d_texture_idx = wined3d_texture_idx {
            texture: texture,
            sub_resource_idx: sub_resource_idx,
        };
        wined3d_cs_destroy_object(
            (*device).cs as *mut wined3d_cs,
            Some(
                wined3d_texture_destroy_dc as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
            ),
            &raw mut texture_idx as *mut ::core::ffi::c_void,
        );
        wined3d_cs_finish(
            (*device).cs as *mut wined3d_cs,
            WINED3D_CS_QUEUE_DEFAULT,
        );
    }

    (*sub_resource).map_count = (*sub_resource).map_count.wrapping_sub(1);
    (*texture).resource.map_count -= 1;
    if (*texture).resource.map_count == 0 && (*texture).update_map_binding != 0 {
        wined3d_texture_update_map_binding(texture);
    }
    if (*texture).flags & WINED3D_TEXTURE_GET_DC_LENIENT as uint32_t == 0 {
        (*texture).flags &= !(WINED3D_TEXTURE_DC_IN_USE as uint32_t);
    }

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_upload_from_texture(mut dst_texture: *mut wined3d_texture, mut dst_sub_resource_idx: ::core::ffi::c_uint, mut dst_x: ::core::ffi::c_uint, mut dst_y: ::core::ffi::c_uint, mut dst_z: ::core::ffi::c_uint, mut src_texture: *mut wined3d_texture, mut src_sub_resource_idx: ::core::ffi::c_uint, mut src_box: *const wined3d_box) {
    let mut src_row_pitch: ::core::ffi::c_uint = 0;
    let mut src_slice_pitch: ::core::ffi::c_uint = 0;
    let mut update_w: ::core::ffi::c_uint = 0;
    let mut update_h: ::core::ffi::c_uint = 0;
    let mut update_d: ::core::ffi::c_uint = 0;
    let mut src_level: ::core::ffi::c_uint = 0;
    let mut dst_level: ::core::ffi::c_uint = 0;
    let mut context: *mut wined3d_context = core::ptr::null_mut::<wined3d_context>();
    let mut data: wined3d_bo_address = wined3d_bo_address {
        buffer_object: core::ptr::null_mut::<wined3d_bo>(),
        addr: core::ptr::null_mut::<BYTE>(),
    };
    WINE_TRACE!("dst_texture %p, dst_sub_resource_idx %u, dst_x %u, dst_y %u, dst_z %u, src_texture %p, src_sub_resource_idx %u, src_box %s.\n", dst_texture, dst_sub_resource_idx, dst_x, dst_y, dst_z, src_texture, src_sub_resource_idx, debug_box(src_box));

    context = context_acquire(
        (*dst_texture).resource.device,
        core::ptr::null_mut::<wined3d_texture>(),
        0,
    );

    /* Only load the sub-resource for partial updates. For newly allocated
     * textures the texture wouldn't be the current location, and we'd upload
     * zeroes just to overwrite them again. */
    update_w = (*src_box).right.wrapping_sub((*src_box).left) as ::core::ffi::c_uint;
    update_h = (*src_box).bottom.wrapping_sub((*src_box).top) as ::core::ffi::c_uint;
    update_d = (*src_box).back.wrapping_sub((*src_box).front) as ::core::ffi::c_uint;
    dst_level = dst_sub_resource_idx % (*dst_texture).level_count;
    if update_w == wined3d_texture_get_level_width(dst_texture, dst_level)
        && update_h == wined3d_texture_get_level_height(dst_texture, dst_level)
        && update_d == wined3d_texture_get_level_depth(dst_texture, dst_level)
    {
        wined3d_texture_prepare_location(
            dst_texture,
            dst_sub_resource_idx,
            context,
            WINED3D_LOCATION_TEXTURE_RGB,
        );
    } else {
        wined3d_texture_load_location(
            dst_texture,
            dst_sub_resource_idx,
            context,
            WINED3D_LOCATION_TEXTURE_RGB,
        );
    }

    src_level = src_sub_resource_idx % (*src_texture).level_count;
    wined3d_texture_get_memory(
        src_texture,
        src_sub_resource_idx,
        context,
        core::ptr::addr_of_mut!(data),
    );
    wined3d_texture_get_pitch(
        src_texture,
        src_level,
        core::ptr::addr_of_mut!(src_row_pitch),
        core::ptr::addr_of_mut!(src_slice_pitch),
    );

    (*(*dst_texture).texture_ops)
        .texture_upload_data
        .unwrap_unchecked()(
        context,
        wined3d_const_bo_address(core::ptr::addr_of_mut!(data)),
        (*src_texture).resource.format,
        src_box,
        src_row_pitch,
        src_slice_pitch,
        dst_texture,
        dst_sub_resource_idx,
        WINED3D_LOCATION_TEXTURE_RGB,
        dst_x,
        dst_y,
        dst_z,
    );

    context_release(context);

    wined3d_texture_validate_location(dst_texture, dst_sub_resource_idx, WINED3D_LOCATION_TEXTURE_RGB);
    wined3d_texture_invalidate_location(
        dst_texture,
        dst_sub_resource_idx,
        !(WINED3D_LOCATION_TEXTURE_RGB as uint32_t),
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_download_from_texture(dst_texture: *mut wined3d_texture, dst_sub_resource_idx: ::core::ffi::c_uint, src_texture: *mut wined3d_texture, src_sub_resource_idx: ::core::ffi::c_uint) {
    let mut src_level: ::core::ffi::c_uint = 0;
    let mut dst_level: ::core::ffi::c_uint = 0;
    let mut dst_row_pitch: ::core::ffi::c_uint = 0;
    let mut dst_slice_pitch: ::core::ffi::c_uint = 0;
    let mut dst_location: ::core::ffi::c_uint = (*dst_texture).resource.map_binding as ::core::ffi::c_uint;
    let mut context: *mut wined3d_context = core::ptr::null_mut::<wined3d_context>();
    let mut data: wined3d_bo_address = wined3d_bo_address {
        buffer_object: core::ptr::null_mut::<wined3d_bo>(),
        addr: core::ptr::null_mut::<BYTE>(),
    };
    let mut src_box: wined3d_box = wined3d_box {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        front: 0,
        back: 0,
    };
    let mut src_location: ::core::ffi::c_uint = 0;

    context = context_acquire(
        (*src_texture).resource.device,
        core::ptr::null_mut::<wined3d_texture>(),
        0 as ::core::ffi::c_uint,
    );

    wined3d_texture_prepare_location(dst_texture, dst_sub_resource_idx, context, dst_location);
    wined3d_texture_get_bo_address(
        dst_texture,
        dst_sub_resource_idx,
        core::ptr::addr_of_mut!(data),
        dst_location as uint32_t,
    );

    if ((*(*src_texture).sub_resources.offset(src_sub_resource_idx as isize)).locations
        & (WINED3D_LOCATION_TEXTURE_RGB as DWORD))
        != 0
    {
        src_location = WINED3D_LOCATION_TEXTURE_RGB as ::core::ffi::c_uint;
    } else {
        src_location = WINED3D_LOCATION_TEXTURE_SRGB as ::core::ffi::c_uint;
    }
    src_level = src_sub_resource_idx.wrapping_rem((*src_texture).level_count);
    wined3d_texture_get_level_box(src_texture, src_level, core::ptr::addr_of_mut!(src_box));

    dst_level = dst_sub_resource_idx.wrapping_rem((*dst_texture).level_count);
    wined3d_texture_get_pitch(
        dst_texture,
        dst_level,
        core::ptr::addr_of_mut!(dst_row_pitch),
        core::ptr::addr_of_mut!(dst_slice_pitch),
    );

    (*(*src_texture).texture_ops)
        .texture_download_data
        .unwrap_unchecked()(
        context,
        src_texture,
        src_sub_resource_idx,
        src_location,
        core::ptr::addr_of_mut!(src_box),
        core::ptr::addr_of_mut!(data),
        (*dst_texture).resource.format,
        0 as ::core::ffi::c_uint,
        0 as ::core::ffi::c_uint,
        0 as ::core::ffi::c_uint,
        dst_row_pitch,
        dst_slice_pitch,
    );

    context_release(context);

    wined3d_texture_validate_location(dst_texture, dst_sub_resource_idx, dst_location as uint32_t);
    wined3d_texture_invalidate_location(
        dst_texture,
        dst_sub_resource_idx,
        !(dst_location as uint32_t),
    );
}

unsafe extern "C" fn wined3d_texture_set_bo(
    texture: *mut wined3d_texture,
    sub_resource_idx: u32,
    context: *mut wined3d_context,
    bo: *mut wined3d_bo,
) {
    let sub_resource: *mut wined3d_texture_sub_resource =
        (*texture).sub_resources.add(sub_resource_idx as usize);
    let prev_bo: *mut wined3d_bo = (*sub_resource).bo;

    if !prev_bo.is_null() {
        let mut bo_user: *mut wined3d_bo_user = core::ptr::null_mut();

        bo_user = ((*prev_bo).users.next as *mut u8)
            .offset(-(core::mem::offset_of!(wined3d_bo_user, entry) as isize))
            as *mut wined3d_bo_user;
        while core::ptr::addr_of_mut!((*bo_user).entry)
            != core::ptr::addr_of_mut!((*prev_bo).users)
        {
            (*bo_user).valid = false;
            bo_user = ((*bo_user).entry.next as *mut u8)
                .offset(-(core::mem::offset_of!(wined3d_bo_user, entry) as isize))
                as *mut wined3d_bo_user;
        }
        list_init(core::ptr::addr_of_mut!((*prev_bo).users));

        assert!(
            list_empty(core::ptr::addr_of_mut!((*bo).users)) != 0,
        );

        wined3d_context_destroy_bo(context, prev_bo);
        free((prev_bo).cast());
    }

    (*sub_resource).bo = bo;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_update_sub_resource(
    mut texture: *mut wined3d_texture,
    mut sub_resource_idx: ::core::ffi::c_uint,
    mut context: *mut wined3d_context,
    mut upload_bo: *const upload_bo,
    mut box_0: *const wined3d_box,
    mut row_pitch: ::core::ffi::c_uint,
    mut slice_pitch: ::core::ffi::c_uint,
) {
    let mut level: ::core::ffi::c_uint = sub_resource_idx.wrapping_rem((*texture).level_count);
    let mut width: ::core::ffi::c_uint = wined3d_texture_get_level_width(texture, level);
    let mut height: ::core::ffi::c_uint = wined3d_texture_get_level_height(texture, level);
    let mut depth: ::core::ffi::c_uint = wined3d_texture_get_level_depth(texture, level);
    let mut src_box: wined3d_box = wined3d_box {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        front: 0,
        back: 0,
    };
    if ((*upload_bo).flags & UPLOAD_BO_RENAME_ON_UNMAP as uint32_t) != 0 {
        wined3d_texture_set_bo(
            texture,
            sub_resource_idx,
            context,
            (*upload_bo).addr.buffer_object,
        );
        wined3d_texture_validate_location(
            texture,
            sub_resource_idx,
            WINED3D_LOCATION_BUFFER as uint32_t,
        );
        wined3d_texture_invalidate_location(
            texture,
            sub_resource_idx,
            !WINED3D_LOCATION_BUFFER as uint32_t,
        );
        /* Try to free address space if we are not mapping persistently. */
        if !(*(*upload_bo).addr.buffer_object).map_ptr.is_null() {
            wined3d_context_unmap_bo_address(
                context,
                &raw const (*upload_bo).addr as *const wined3d_bo_address,
                0 as ::core::ffi::c_uint,
                ::core::ptr::null::<wined3d_range>(),
            );
        }
    }
    /* Only load the sub-resource for partial updates. */
    if (*box_0).left == 0
        && (*box_0).top == 0
        && (*box_0).front == 0
        && (*box_0).right == width
        && (*box_0).bottom == height
        && (*box_0).back == depth
    {
        wined3d_texture_prepare_location(
            texture,
            sub_resource_idx,
            context,
            WINED3D_LOCATION_TEXTURE_RGB as ::core::ffi::c_uint,
        );
    } else {
        wined3d_texture_load_location(
            texture,
            sub_resource_idx,
            context,
            WINED3D_LOCATION_TEXTURE_RGB as uint32_t,
        );
    }
    wined3d_box_set(
        &raw mut src_box,
        0 as ::core::ffi::c_uint,
        0 as ::core::ffi::c_uint,
        ((*box_0).right as ::core::ffi::c_uint).wrapping_sub((*box_0).left as ::core::ffi::c_uint),
        ((*box_0).bottom as ::core::ffi::c_uint).wrapping_sub((*box_0).top as ::core::ffi::c_uint),
        0 as ::core::ffi::c_uint,
        ((*box_0).back as ::core::ffi::c_uint).wrapping_sub((*box_0).front as ::core::ffi::c_uint),
    );
    (*(*texture).texture_ops)
        .texture_upload_data
        .unwrap_unchecked()(
        context,
        &raw const (*upload_bo).addr,
        (*texture).resource.format,
        &raw mut src_box,
        row_pitch,
        slice_pitch,
        texture,
        sub_resource_idx,
        WINED3D_LOCATION_TEXTURE_RGB as ::core::ffi::c_uint,
        (*box_0).left as ::core::ffi::c_uint,
        (*box_0).top as ::core::ffi::c_uint,
        (*box_0).front as ::core::ffi::c_uint,
    );
    wined3d_texture_validate_location(
        texture,
        sub_resource_idx,
        WINED3D_LOCATION_TEXTURE_RGB as uint32_t,
    );
    wined3d_texture_invalidate_location(
        texture,
        sub_resource_idx,
        !WINED3D_LOCATION_TEXTURE_RGB as uint32_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_acquire_identity_srv(
    texture: *mut wined3d_texture,
) -> *mut wined3d_shader_resource_view {
    let mut desc: wined3d_view_desc = core::mem::zeroed();
    let mut hr: HRESULT;

    if !(*texture).identity_srv.is_null() {
        return (*texture).identity_srv;
    }

    desc.format_id = (*(*texture).resource.format).id;
    desc.flags = WINED3D_VIEW_FORWARD_REFERENCE;
    desc.u.texture.level_idx = 0;
    desc.u.texture.level_count = (*texture).level_count;
    desc.u.texture.layer_idx = 0;
    desc.u.texture.layer_count = (*texture).layer_count;
    hr = wined3d_shader_resource_view_create(
        core::ptr::addr_of_mut!(desc),
        core::ptr::addr_of_mut!((*texture).resource),
        core::ptr::null_mut(),
        core::ptr::addr_of!(wined3d_null_parent_ops),
        core::ptr::addr_of_mut!((*texture).identity_srv),
    );
    if hr < 0 {
        ERR!("Failed to create shader resource view, hr %#lx.\n", hr);
        return core::ptr::null_mut();
    }
    wined3d_shader_resource_view_decref((*texture).identity_srv);

    return (*texture).identity_srv;
}

unsafe extern "C" fn wined3d_texture_no3d_upload_data(
    context: *mut wined3d_context,
    src_bo_addr: *const wined3d_const_bo_address,
    src_format: *const wined3d_format,
    src_box: *const wined3d_box,
    src_row_pitch: u32,
    src_slice_pitch: u32,
    dst_texture: *mut wined3d_texture,
    dst_sub_resource_idx: u32,
    dst_location: u32,
    dst_x: u32,
    dst_y: u32,
    dst_z: u32,
) {
    // FIXME("Not implemented.\n");
}

unsafe extern "C" fn wined3d_texture_no3d_download_data(
    context: *mut wined3d_context,
    src_texture: *mut wined3d_texture,
    src_sub_resource_idx: u32,
    src_location: u32,
    src_box: *const wined3d_box,
    dst_bo_addr: *const wined3d_bo_address,
    dst_format: *const wined3d_format,
    dst_x: u32,
    dst_y: u32,
    dst_z: u32,
    dst_row_pitch: u32,
    dst_slice_pitch: u32,
) {
    // FIXME("Not implemented.\n");
}

unsafe extern "C" fn wined3d_texture_no3d_prepare_location(
    texture: *mut wined3d_texture,
    sub_resource_idx: u32,
    context: *mut wined3d_context,
    location: u32,
) -> BOOL {
    if location == WINED3D_LOCATION_SYSMEM {
        return if !(*(*texture).sub_resources.add(sub_resource_idx as usize)).user_memory.is_null() {
            (TRUE as i32)
        } else {
            wined3d_resource_prepare_sysmem(core::ptr::addr_of_mut!((*texture).resource))
        };
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn wined3d_texture_no3d_load_location(texture: *mut wined3d_texture, sub_resource_idx: u32, context: *mut wined3d_context, location: u32) -> BOOL {
    if location == WINED3D_LOCATION_SYSMEM {
        return (TRUE) as BOOL;
    }

    ERR!("Unhandled location %s.\n", wined3d_debug_location(location));

    return (FALSE) as BOOL;
}

unsafe extern "C" fn wined3d_texture_no3d_unload_location(
    texture: *mut wined3d_texture,
    context: *mut wined3d_context,
    location: u32,
) {
    WINE_TRACE!("texture %p, context %p, location %s.\n", texture, context, wined3d_debug_location(location as uint32_t));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_no3d_init(texture_no3d: *mut wined3d_texture, device: *mut wined3d_device, desc: *const wined3d_resource_desc, layer_count: ::core::ffi::c_uint, level_count: ::core::ffi::c_uint, flags: uint32_t, parent: *mut ::core::ffi::c_void, parent_ops: *const wined3d_parent_ops) -> HRESULT {
    return wined3d_texture_init(texture_no3d, desc, layer_count, level_count,
            flags, device, parent, parent_ops, texture_no3d.add(1) as *mut ::core::ffi::c_void, core::ptr::addr_of!(wined3d_texture_no3d_ops));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_vk_swizzle_from_color_fixup(mut mapping: *mut VkComponentMapping, mut fixup: color_fixup_desc) {
    static swizzle_source: [VkComponentSwizzle; 6] = [
        VK_COMPONENT_SWIZZLE_ZERO, // CHANNEL_SOURCE_ZERO
        VK_COMPONENT_SWIZZLE_ONE, // CHANNEL_SOURCE_ONE
        VK_COMPONENT_SWIZZLE_R, // CHANNEL_SOURCE_X
        VK_COMPONENT_SWIZZLE_G, // CHANNEL_SOURCE_Y
        VK_COMPONENT_SWIZZLE_B, // CHANNEL_SOURCE_Z
        VK_COMPONENT_SWIZZLE_A, // CHANNEL_SOURCE_W
    ];
    (*mapping).r = swizzle_source[fixup.x_source() as usize];
    (*mapping).g = swizzle_source[fixup.y_source() as usize];
    (*mapping).b = swizzle_source[fixup.z_source() as usize];
    (*mapping).a = swizzle_source[fixup.w_source() as usize];
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_vk_get_default_image_info(texture_vk: *mut wined3d_texture_vk, context_vk: *mut wined3d_context_vk) -> *const VkDescriptorImageInfo {
    let mut format_vk: *const wined3d_format_vk = core::ptr::null();
    let mut vk_info: *const wined3d_vk_info = core::ptr::null();
    let mut device_vk: *mut wined3d_device_vk = core::ptr::null_mut();
    let mut create_info: VkImageViewCreateInfo = core::mem::zeroed();
    let mut fixup: color_fixup_desc = core::mem::zeroed();
    let mut flags: uint32_t = 0;
    let mut vr: VkResult;

    if (*texture_vk).default_image_info.imageView != 0 {
        return core::ptr::addr_of!((*texture_vk).default_image_info);
    }

    format_vk = wined3d_format_vk((*texture_vk).t.resource.format);
    device_vk = wined3d_device_vk((*texture_vk).t.resource.device);
    vk_info = (*context_vk).vk_info;

    if (*texture_vk).t.layer_count > 1 {
        flags |= WINED3D_VIEW_TEXTURE_ARRAY;
    }

    wined3d_texture_vk_prepare_texture(texture_vk, context_vk);
    create_info.sType = VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO;
    create_info.pNext = core::ptr::null();
    create_info.flags = 0;
    create_info.image = (*texture_vk).image.vk_image;
    create_info.viewType = vk_image_view_type_from_wined3d((*texture_vk).t.resource.type_, flags);
    create_info.format = (*format_vk).vk_format;
    fixup = (*format_vk).f.color_fixup;
    if is_identity_fixup(fixup) != 0
        || can_use_texture_swizzle((*context_vk).c.d3d_info, core::ptr::addr_of!((*format_vk).f)) == 0
    {
        create_info.components.r = VK_COMPONENT_SWIZZLE_IDENTITY;
        create_info.components.g = VK_COMPONENT_SWIZZLE_IDENTITY;
        create_info.components.b = VK_COMPONENT_SWIZZLE_IDENTITY;
        create_info.components.a = VK_COMPONENT_SWIZZLE_IDENTITY;
    } else {
        wined3d_vk_swizzle_from_color_fixup(core::ptr::addr_of_mut!(create_info.components), fixup);
    }
    create_info.subresourceRange.aspectMask =
        vk_aspect_mask_from_format(core::ptr::addr_of!((*format_vk).f));
    create_info.subresourceRange.baseMipLevel = 0;
    create_info.subresourceRange.levelCount = (*texture_vk).t.level_count;
    create_info.subresourceRange.baseArrayLayer = 0;
    create_info.subresourceRange.layerCount = (*texture_vk).t.layer_count;

    vr = ((*vk_info).vk_ops.vkCreateImageView.unwrap_unchecked())(
        (*device_vk).vk_device,
        core::ptr::addr_of_mut!(create_info),
        core::ptr::null(),
        core::ptr::addr_of_mut!((*texture_vk).default_image_info.imageView),
    );
    if (vr as i32) < 0 {
        ERR!(
            "Failed to create Vulkan image view, vr %s.\n",
            wined3d_debug_vkresult(vr)
        );
        return core::ptr::null();
    }

    TRACE!(
        "Created image view 0x%s.\n",
        wine_dbgstr_longlong((*texture_vk).default_image_info.imageView as ULONGLONG)
    );

    (*texture_vk).default_image_info.sampler = (VK_NULL_HANDLE as _);

    if (*texture_vk).layout == VK_IMAGE_LAYOUT_GENERAL {
        (*texture_vk).default_image_info.imageLayout = VK_IMAGE_LAYOUT_GENERAL;
    } else {
        (*texture_vk).default_image_info.imageLayout = VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL;
    }

    return core::ptr::addr_of!((*texture_vk).default_image_info);
}

unsafe extern "C" fn wined3d_texture_vk_upload_data(
    context: *mut wined3d_context,
    src_bo_addr: *const wined3d_const_bo_address,
    src_format: *const wined3d_format,
    src_box: *const wined3d_box,
    mut src_row_pitch: u32,
    mut src_slice_pitch: u32,
    dst_texture: *mut wined3d_texture,
    dst_sub_resource_idx: u32,
    mut dst_location: u32,
    dst_x: u32,
    dst_y: u32,
    dst_z: u32,
) {
    let mut dst_texture_vk: *mut wined3d_texture_vk = wined3d_texture_vk(dst_texture);
    let mut context_vk: *mut wined3d_context_vk = wined3d_context_vk(context);
    let mut dst_level: u32 = 0;
    let mut dst_row_pitch: u32 = 0;
    let mut dst_slice_pitch: u32 = 0;
    let mut sub_resource: *mut wined3d_texture_sub_resource = core::ptr::null_mut();
    let mut src_width: u32 = 0;
    let mut src_height: u32 = 0;
    let mut src_depth: u32 = 0;
    let mut staging_bo_addr: wined3d_bo_address = core::mem::zeroed();
    let mut bo_stage_flags: VkPipelineStageFlags = 0;
    let mut vk_info: *const wined3d_vk_info = core::ptr::null();
    let mut vk_command_buffer: VkCommandBuffer = core::ptr::null_mut();
    let mut vk_barrier: VkBufferMemoryBarrier = core::mem::zeroed();
    let mut vk_range: VkImageSubresourceRange = core::mem::zeroed();
    let mut staging_bo: wined3d_bo_vk = core::mem::zeroed();
    let mut aspect_mask: VkImageAspectFlags = 0;
    let mut src_bo: *mut wined3d_bo_vk = core::ptr::null_mut();
    let mut range: wined3d_range = core::mem::zeroed();
    let mut region: VkBufferImageCopy = core::mem::zeroed();
    let mut src_offset: usize = 0;
    let mut map_ptr: *mut core::ffi::c_void = core::ptr::null_mut();

    if (*src_format).id != (*(*dst_texture).resource.format).id {
        return;
    }

    dst_level = dst_sub_resource_idx % (*dst_texture).level_count;
    wined3d_texture_get_pitch(
        dst_texture,
        dst_level,
        core::ptr::addr_of_mut!(dst_row_pitch),
        core::ptr::addr_of_mut!(dst_slice_pitch),
    );
    if (*dst_texture).resource.type_ as u32 == WINED3D_RTYPE_TEXTURE_1D {
        dst_row_pitch = 0;
        src_row_pitch = dst_row_pitch;
    }
    if (*dst_texture).resource.type_ as u32 != WINED3D_RTYPE_TEXTURE_3D {
        dst_slice_pitch = 0;
        src_slice_pitch = dst_slice_pitch;
    }

    if dst_location != WINED3D_LOCATION_TEXTURE_RGB {
        return;
    }

    if wined3d_resource_get_sample_count(core::ptr::addr_of_mut!((*dst_texture_vk).t.resource))
        > 1
    {
        return;
    }

    aspect_mask = vk_aspect_mask_from_format((*dst_texture).resource.format);
    if wined3d_popcount(aspect_mask) > 1 {
        return;
    }

    sub_resource = (*dst_texture_vk)
        .t
        .sub_resources
        .offset(dst_sub_resource_idx as isize);
    vk_info = (*context_vk).vk_info;

    src_width = (*src_box).right.wrapping_sub((*src_box).left) as u32;
    src_height = (*src_box).bottom.wrapping_sub((*src_box).top) as u32;
    src_depth = (*src_box).back.wrapping_sub((*src_box).front) as u32;

    src_offset = (*src_box)
        .front
        .wrapping_mul(src_slice_pitch as u32)
        .wrapping_add(
            (*src_box)
                .top
                .wrapping_div((*src_format).block_height)
                .wrapping_mul(src_row_pitch as u32),
        )
        .wrapping_add(
            (*src_box)
                .left
                .wrapping_div((*src_format).block_width)
                .wrapping_mul((*src_format).block_byte_count),
        ) as usize;

    vk_command_buffer = wined3d_context_vk_get_command_buffer(context_vk);
    if vk_command_buffer.is_null() {
        ERR!("Failed to get command buffer.\n");
        return;
    }

    /* We need to be outside of a render pass for vkCmdPipelineBarrier() and vkCmdCopyBufferToImage() calls below. */
    wined3d_context_vk_end_current_render_pass(context_vk);
    if (*src_bo_addr).buffer_object.is_null() {
        let mut staging_row_pitch: u32 = 0;
        let mut staging_slice_pitch: u32 = 0;
        let mut staging_size: u32 = 0;

        wined3d_format_calculate_pitch(
            src_format,
            (*(*context).device).surface_alignment as u32,
            src_width,
            src_height,
            core::ptr::addr_of_mut!(staging_row_pitch),
            core::ptr::addr_of_mut!(staging_slice_pitch),
        );
        staging_size = staging_slice_pitch.wrapping_mul(src_depth);

        if wined3d_context_vk_create_bo(
            context_vk,
            staging_size as VkDeviceSize,
            VK_BUFFER_USAGE_TRANSFER_SRC_BIT,
            VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT,
            core::ptr::addr_of_mut!(staging_bo),
        ) == 0
        {
            ERR!("Failed to create staging bo.\n");
            return;
        }

        staging_bo_addr.buffer_object = core::ptr::addr_of_mut!(staging_bo.b);
        staging_bo_addr.addr = core::ptr::null_mut();
        map_ptr = wined3d_context_map_bo_address(
            context,
            core::ptr::addr_of_mut!(staging_bo_addr),
            staging_size as usize,
            (WINED3D_MAP_DISCARD | WINED3D_MAP_WRITE) as u32,
        );
        if map_ptr.is_null() {
            ERR!("Failed to map staging bo.\n");
            wined3d_context_vk_destroy_bo(context_vk, core::ptr::addr_of_mut!(staging_bo));
            return;
        }

        wined3d_format_copy_data(
            src_format,
            (*src_bo_addr).addr.offset(src_offset as isize),
            src_row_pitch,
            src_slice_pitch,
            map_ptr as *mut u8,
            staging_row_pitch,
            staging_slice_pitch,
            src_width,
            src_height,
            src_depth,
        );

        range.offset = 0;
        range.size = staging_size;
        wined3d_context_unmap_bo_address(
            context,
            core::ptr::addr_of_mut!(staging_bo_addr),
            1,
            core::ptr::addr_of_mut!(range),
        );

        src_bo = core::ptr::addr_of_mut!(staging_bo);

        src_offset = 0;
        src_row_pitch = staging_row_pitch;
        src_slice_pitch = staging_slice_pitch;
    } else {
        src_bo = wined3d_bo_vk((*src_bo_addr).buffer_object);

        vk_barrier.sType = VK_STRUCTURE_TYPE_BUFFER_MEMORY_BARRIER;
        vk_barrier.pNext = core::ptr::null();
        vk_barrier.srcAccessMask =
            vk_access_mask_from_buffer_usage((*src_bo).usage) & !WINED3D_READ_ONLY_ACCESS_FLAGS;
        vk_barrier.dstAccessMask = VK_ACCESS_TRANSFER_READ_BIT;
        vk_barrier.srcQueueFamilyIndex = (VK_QUEUE_FAMILY_IGNORED) as u32;
        vk_barrier.dstQueueFamilyIndex = (VK_QUEUE_FAMILY_IGNORED) as u32;
        vk_barrier.buffer = (*src_bo).vk_buffer;
        vk_barrier.offset = (*src_bo)
            .b
            .buffer_offset
            .wrapping_add((*src_bo_addr).addr as usize) as VkDeviceSize;
        vk_barrier.size = (*sub_resource).size as VkDeviceSize;

        src_offset = src_offset.wrapping_add((*src_bo_addr).addr as usize);

        bo_stage_flags = vk_pipeline_stage_mask_from_buffer_usage((*src_bo).usage);
        if vk_barrier.srcAccessMask != 0 {
            ((*vk_info).vk_ops.vkCmdPipelineBarrier.unwrap_unchecked())(
                vk_command_buffer,
                bo_stage_flags,
                VK_PIPELINE_STAGE_TRANSFER_BIT,
                0,
                0,
                core::ptr::null(),
                1,
                core::ptr::addr_of_mut!(vk_barrier),
                0,
                core::ptr::null(),
            );
        }
    }

    vk_range.aspectMask = aspect_mask;
    vk_range.baseMipLevel = dst_level;
    vk_range.levelCount = 1;
    vk_range.baseArrayLayer = dst_sub_resource_idx / (*dst_texture_vk).t.level_count;
    vk_range.layerCount = 1;

    wined3d_context_vk_image_barrier(
        context_vk,
        vk_command_buffer,
        VK_PIPELINE_STAGE_ALL_COMMANDS_BIT,
        VK_PIPELINE_STAGE_TRANSFER_BIT,
        vk_access_mask_from_bind_flags((*dst_texture_vk).t.resource.bind_flags),
        VK_ACCESS_TRANSFER_WRITE_BIT,
        (*dst_texture_vk).layout,
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
        (*dst_texture_vk).image.vk_image,
        core::ptr::addr_of_mut!(vk_range),
    );

    region.bufferOffset = (*src_bo).b.buffer_offset.wrapping_add(src_offset) as VkDeviceSize;
    region.bufferRowLength = (src_row_pitch / (*src_format).block_byte_count)
        .wrapping_mul((*src_format).block_width) as u32;
    if src_row_pitch != 0 {
        region.bufferImageHeight = (src_slice_pitch / src_row_pitch)
            .wrapping_mul((*src_format).block_height) as u32;
    } else {
        region.bufferImageHeight = 1;
    }
    region.imageSubresource.aspectMask = vk_range.aspectMask;
    region.imageSubresource.mipLevel = vk_range.baseMipLevel;
    region.imageSubresource.baseArrayLayer = vk_range.baseArrayLayer;
    region.imageSubresource.layerCount = vk_range.layerCount;
    region.imageOffset.x = dst_x as i32;
    region.imageOffset.y = dst_y as i32;
    region.imageOffset.z = dst_z as i32;
    region.imageExtent.width = src_width;
    region.imageExtent.height = src_height;
    region.imageExtent.depth = src_depth;

    ((*vk_info).vk_ops.vkCmdCopyBufferToImage.unwrap_unchecked())(
        vk_command_buffer,
        (*src_bo).vk_buffer,
        (*dst_texture_vk).image.vk_image,
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
        1,
        core::ptr::addr_of_mut!(region),
    );

    wined3d_context_vk_image_barrier(
        context_vk,
        vk_command_buffer,
        VK_PIPELINE_STAGE_TRANSFER_BIT,
        VK_PIPELINE_STAGE_ALL_COMMANDS_BIT,
        VK_ACCESS_TRANSFER_WRITE_BIT,
        vk_access_mask_from_bind_flags((*dst_texture_vk).t.resource.bind_flags),
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
        (*dst_texture_vk).layout,
        (*dst_texture_vk).image.vk_image,
        core::ptr::addr_of_mut!(vk_range),
    );
    wined3d_context_vk_reference_texture(context_vk, dst_texture_vk);
    wined3d_context_vk_reference_bo(context_vk, src_bo);

    if src_bo == core::ptr::addr_of_mut!(staging_bo) {
        wined3d_context_vk_destroy_bo(context_vk, core::ptr::addr_of_mut!(staging_bo));
    } else if vk_barrier.srcAccessMask != 0 {
        ((*vk_info).vk_ops.vkCmdPipelineBarrier.unwrap_unchecked())(
            vk_command_buffer,
            VK_PIPELINE_STAGE_TRANSFER_BIT,
            bo_stage_flags,
            0,
            0,
            core::ptr::null(),
            0,
            core::ptr::null(),
            0,
            core::ptr::null(),
        );
    }
}

unsafe extern "C" fn wined3d_texture_vk_download_data(
    context: *mut wined3d_context,
    src_texture: *mut wined3d_texture,
    src_sub_resource_idx: ::core::ffi::c_uint,
    src_location: ::core::ffi::c_uint,
    src_box: *const wined3d_box,
    dst_bo_addr: *const wined3d_bo_address,
    dst_format: *const wined3d_format,
    dst_x: ::core::ffi::c_uint,
    dst_y: ::core::ffi::c_uint,
    dst_z: ::core::ffi::c_uint,
    mut dst_row_pitch: ::core::ffi::c_uint,
    mut dst_slice_pitch: ::core::ffi::c_uint,
) {
    let src_texture_vk: *mut wined3d_texture_vk = wined3d_texture_vk(src_texture);
    let context_vk: *mut wined3d_context_vk = wined3d_context_vk(context);
    let mut src_level: ::core::ffi::c_uint = 0;
    let mut src_width: ::core::ffi::c_uint = 0;
    let mut src_height: ::core::ffi::c_uint = 0;
    let mut src_depth: ::core::ffi::c_uint = 0;
    let mut sub_resource: *mut wined3d_texture_sub_resource = core::ptr::null_mut();
    let mut src_row_pitch: ::core::ffi::c_uint = 0;
    let mut src_slice_pitch: ::core::ffi::c_uint = 0;
    let mut staging_bo_addr: wined3d_bo_address = core::mem::zeroed();
    let mut bo_stage_flags: VkPipelineStageFlags =
        0 as VkPipelineStageFlags;
    let mut vk_info: *const wined3d_vk_info = core::ptr::null();
    let mut vk_command_buffer: VkCommandBuffer = core::ptr::null_mut();
    let mut vk_range: VkImageSubresourceRange = core::mem::zeroed();
    let mut vk_barrier: VkBufferMemoryBarrier = core::mem::zeroed();
    let mut staging_bo: wined3d_bo_vk = core::mem::zeroed();
    let mut aspect_mask: VkImageAspectFlags = 0;
    let mut dst_bo: *mut wined3d_bo_vk = core::ptr::null_mut();
    let mut region: VkBufferImageCopy = core::mem::zeroed();
    let mut dst_offset: size_t = 0 as size_t;
    let mut map_ptr: *mut ::core::ffi::c_void = core::ptr::null_mut();

    TRACE!(
        "context %p, src_texture %p, src_sub_resource_idx %u, src_location %s, src_box %s, dst_bo_addr %s, dst_format %s, dst_x %u, dst_y %u, dst_z %u, dst_row_pitch %u, dst_slice_pitch %u.\n",
        context,
        src_texture,
        src_sub_resource_idx,
        wined3d_debug_location(src_location as uint32_t),
        debug_box(src_box),
        debug_bo_address(dst_bo_addr),
        debug_d3dformat((*dst_format).id),
        dst_x,
        dst_y,
        dst_z,
        dst_row_pitch,
        dst_slice_pitch
    );

    if src_location != WINED3D_LOCATION_TEXTURE_RGB as ::core::ffi::c_uint {
        FIXME!(
            "Unhandled location %s.\n",
            wined3d_debug_location(src_location as uint32_t)
        );
        return;
    }

    src_level = src_sub_resource_idx.wrapping_rem((*src_texture).level_count);
    src_width = wined3d_texture_get_level_width(src_texture, src_level);
    src_height = wined3d_texture_get_level_height(src_texture, src_level);
    src_depth = wined3d_texture_get_level_depth(src_texture, src_level);
    if (*src_box).left != 0
        || (*src_box).top != 0
        || (*src_box).right != src_width
        || (*src_box).bottom != src_height
        || (*src_box).front != 0
        || (*src_box).back != src_depth
    {
        FIXME!("Unhandled source box %s.\n", debug_box(src_box));
        return;
    }

    if (*dst_format).id as ::core::ffi::c_uint
        != (*(*src_texture).resource.format).id as ::core::ffi::c_uint
    {
        FIXME!(
            "Unhandled format conversion (%s -> %s).\n",
            debug_d3dformat((*(*src_texture).resource.format).id),
            debug_d3dformat((*dst_format).id)
        );
        return;
    }

    if dst_x != 0 || dst_y != 0 || dst_z != 0 {
        FIXME!(
            "Unhandled destination (%u, %u, %u).\n",
            dst_x,
            dst_y,
            dst_z
        );
        return;
    }

    if wined3d_resource_get_sample_count(&raw mut (*src_texture_vk).t.resource)
        > 1 as ::core::ffi::c_uint
    {
        FIXME!("Not supported for multisample textures.\n");
        return;
    }

    aspect_mask = vk_aspect_mask_from_format((*src_texture).resource.format);
    if wined3d_popcount(aspect_mask as ::core::ffi::c_uint)
        > 1 as ::core::ffi::c_uint
    {
        FIXME!(
            "Unhandled multi-aspect format %s.\n",
            debug_d3dformat((*(*src_texture).resource.format).id)
        );
        return;
    }

    wined3d_texture_get_pitch(src_texture, src_level, &raw mut src_row_pitch, &raw mut src_slice_pitch);
    if (*src_texture).resource.type_ as ::core::ffi::c_uint
        == WINED3D_RTYPE_TEXTURE_1D as ::core::ffi::c_uint
    {
        dst_row_pitch = 0 as ::core::ffi::c_uint;
        src_row_pitch = dst_row_pitch;
    }
    if (*src_texture).resource.type_ as ::core::ffi::c_uint
        != WINED3D_RTYPE_TEXTURE_3D as ::core::ffi::c_uint
    {
        dst_slice_pitch = 0 as ::core::ffi::c_uint;
        src_slice_pitch = dst_slice_pitch;
    }

    sub_resource = (*src_texture_vk)
        .t
        .sub_resources
        .offset(src_sub_resource_idx as isize)
        as *mut wined3d_texture_sub_resource;
    vk_info = (*context_vk).vk_info;
    vk_command_buffer = wined3d_context_vk_get_command_buffer(context_vk);
    if vk_command_buffer.is_null() {
        ERR!("Failed to get command buffer.\n");
        return;
    }

    /* We need to be outside of a render pass for vkCmdPipelineBarrier() and vkCmdCopyImageToBuffer() calls below. */
    wined3d_context_vk_end_current_render_pass(context_vk);

    if (*dst_bo_addr).buffer_object.is_null() {
        if wined3d_context_vk_create_bo(
            context_vk,
            (*sub_resource).size as VkDeviceSize,
            VK_BUFFER_USAGE_TRANSFER_DST_BIT as VkBufferUsageFlags,
            VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT as VkMemoryPropertyFlags,
            &raw mut staging_bo,
        ) == 0
        {
            ERR!("Failed to create staging bo.\n");
            return;
        }

        dst_bo = &raw mut staging_bo;
        region.bufferRowLength = (src_row_pitch as UINT)
            .wrapping_div((*dst_format).block_byte_count)
            .wrapping_mul((*dst_format).block_width) as uint32_t;
        if src_row_pitch != 0 {
            region.bufferImageHeight = (src_slice_pitch as UINT)
                .wrapping_div(src_row_pitch as UINT)
                .wrapping_mul((*dst_format).block_height)
                as uint32_t;
        } else {
            region.bufferImageHeight = 1 as uint32_t;
        }

        if (src_row_pitch as UINT).wrapping_rem((*dst_format).byte_count) != 0 {
            FIXME!(
                "Row pitch %u is not a multiple of byte count %u.\n",
                src_row_pitch,
                (*dst_format).byte_count
            );
            return;
        }
        if src_row_pitch != 0 && src_slice_pitch.wrapping_rem(src_row_pitch) != 0 {
            FIXME!(
                "Slice pitch %u is not a multiple of row pitch %u.\n",
                src_slice_pitch,
                src_row_pitch
            );
            return;
        }
    } else {
        dst_bo = wined3d_bo_vk((*dst_bo_addr).buffer_object);

        region.bufferRowLength = (dst_row_pitch as UINT)
            .wrapping_div((*dst_format).block_byte_count)
            .wrapping_mul((*dst_format).block_width) as uint32_t;
        if dst_row_pitch != 0 {
            region.bufferImageHeight = (dst_slice_pitch as UINT)
                .wrapping_div(dst_row_pitch as UINT)
                .wrapping_mul((*dst_format).block_height)
                as uint32_t;
        } else {
            region.bufferImageHeight = 1 as uint32_t;
        }

        if (dst_row_pitch as UINT).wrapping_rem((*dst_format).byte_count) != 0 {
            FIXME!(
                "Row pitch %u is not a multiple of byte count %u.\n",
                dst_row_pitch,
                (*dst_format).byte_count
            );
            return;
        }
        if dst_row_pitch != 0 && dst_slice_pitch.wrapping_rem(dst_row_pitch) != 0 {
            FIXME!(
                "Slice pitch %u is not a multiple of row pitch %u.\n",
                dst_slice_pitch,
                dst_row_pitch
            );
            return;
        }

        vk_barrier.sType = VK_STRUCTURE_TYPE_BUFFER_MEMORY_BARRIER as VkStructureType;
        vk_barrier.pNext = core::ptr::null();
        vk_barrier.srcAccessMask = vk_access_mask_from_buffer_usage((*dst_bo).usage);
        vk_barrier.dstAccessMask = VK_ACCESS_TRANSFER_WRITE_BIT as VkAccessFlags;
        vk_barrier.srcQueueFamilyIndex = VK_QUEUE_FAMILY_IGNORED as uint32_t;
        vk_barrier.dstQueueFamilyIndex = VK_QUEUE_FAMILY_IGNORED as uint32_t;
        vk_barrier.buffer = (*dst_bo).vk_buffer;
        vk_barrier.offset = (*dst_bo)
            .b
            .buffer_offset
            .wrapping_add((*dst_bo_addr).addr as size_t)
            as VkDeviceSize;
        vk_barrier.size = (*sub_resource).size as VkDeviceSize;

        bo_stage_flags = vk_pipeline_stage_mask_from_buffer_usage((*dst_bo).usage);
        dst_offset = (*dst_bo_addr).addr as size_t;

        ((*vk_info).vk_ops.vkCmdPipelineBarrier.unwrap_unchecked())(
            vk_command_buffer,
            bo_stage_flags,
            VK_PIPELINE_STAGE_TRANSFER_BIT as VkPipelineStageFlags,
            0 as VkDependencyFlags,
            0 as uint32_t,
            core::ptr::null(),
            1 as uint32_t,
            &raw mut vk_barrier,
            0 as uint32_t,
            core::ptr::null()
        );
    }

    vk_range.aspectMask = aspect_mask;
    vk_range.baseMipLevel = src_level as uint32_t;
    vk_range.levelCount = 1 as uint32_t;
    vk_range.baseArrayLayer =
        src_sub_resource_idx.wrapping_div((*src_texture_vk).t.level_count) as uint32_t;
    vk_range.layerCount = 1 as uint32_t;

    wined3d_context_vk_image_barrier(
        context_vk,
        vk_command_buffer,
        VK_PIPELINE_STAGE_ALL_COMMANDS_BIT as VkPipelineStageFlags,
        VK_PIPELINE_STAGE_TRANSFER_BIT as VkPipelineStageFlags,
        vk_access_mask_from_bind_flags((*src_texture_vk).t.resource.bind_flags as uint32_t),
        VK_ACCESS_TRANSFER_READ_BIT as VkAccessFlags,
        (*src_texture_vk).layout as VkImageLayout,
        VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL as VkImageLayout,
        (*src_texture_vk).image.vk_image,
        &raw mut vk_range,
    );

    region.bufferOffset = (*dst_bo).b.buffer_offset.wrapping_add(dst_offset) as VkDeviceSize;
    region.imageSubresource.aspectMask = vk_range.aspectMask;
    region.imageSubresource.mipLevel = vk_range.baseMipLevel;
    region.imageSubresource.baseArrayLayer = vk_range.baseArrayLayer;
    region.imageSubresource.layerCount = vk_range.layerCount;
    region.imageOffset.x = 0 as int32_t;
    region.imageOffset.y = 0 as int32_t;
    region.imageOffset.z = 0 as int32_t;
    region.imageExtent.width = src_width as uint32_t;
    region.imageExtent.height = src_height as uint32_t;
    region.imageExtent.depth = src_depth as uint32_t;

    ((*vk_info).vk_ops.vkCmdCopyImageToBuffer.unwrap_unchecked())(
        vk_command_buffer,
        (*src_texture_vk).image.vk_image,
        VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL as VkImageLayout,
        (*dst_bo).vk_buffer,
        1 as uint32_t,
        &raw mut region
    );

    wined3d_context_vk_image_barrier(
        context_vk,
        vk_command_buffer,
        VK_PIPELINE_STAGE_TRANSFER_BIT as VkPipelineStageFlags,
        VK_PIPELINE_STAGE_ALL_COMMANDS_BIT as VkPipelineStageFlags,
        VK_ACCESS_TRANSFER_READ_BIT as VkAccessFlags,
        vk_access_mask_from_bind_flags((*src_texture_vk).t.resource.bind_flags as uint32_t),
        VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL as VkImageLayout,
        (*src_texture_vk).layout as VkImageLayout,
        (*src_texture_vk).image.vk_image,
        &raw mut vk_range,
    );

    wined3d_context_vk_reference_texture(context_vk, src_texture_vk);
    wined3d_context_vk_reference_bo(context_vk, dst_bo);

    if dst_bo == &raw mut staging_bo {
        wined3d_context_vk_submit_command_buffer(
            context_vk,
            0 as ::core::ffi::c_uint,
            core::ptr::null(),
            core::ptr::null(),
            0 as ::core::ffi::c_uint,
            core::ptr::null(),
        );
        wined3d_context_vk_wait_command_buffer(context_vk, (*src_texture_vk).image.command_buffer_id);

        staging_bo_addr.buffer_object = &raw mut staging_bo.b;
        staging_bo_addr.addr = core::ptr::null_mut();
        map_ptr = wined3d_context_map_bo_address(
            context,
            &raw mut staging_bo_addr,
            (*sub_resource).size as size_t,
            WINED3D_MAP_READ as uint32_t,
        );
        if map_ptr.is_null() {
            ERR!("Failed to map staging bo.\n");
            wined3d_context_vk_destroy_bo(context_vk, &raw mut staging_bo);
            return;
        }

        wined3d_format_copy_data(
            dst_format,
            map_ptr as *const uint8_t,
            src_row_pitch,
            src_slice_pitch,
            (*dst_bo_addr).addr as *mut uint8_t,
            dst_row_pitch,
            dst_slice_pitch,
            ((*src_box).right as ::core::ffi::c_uint)
                .wrapping_sub((*src_box).left as ::core::ffi::c_uint),
            ((*src_box).bottom as ::core::ffi::c_uint)
                .wrapping_sub((*src_box).top as ::core::ffi::c_uint),
            ((*src_box).back as ::core::ffi::c_uint)
                .wrapping_sub((*src_box).front as ::core::ffi::c_uint),
        );

        wined3d_context_unmap_bo_address(
            context,
            &raw mut staging_bo_addr,
            0 as ::core::ffi::c_uint,
            core::ptr::null(),
        );
        wined3d_context_vk_destroy_bo(context_vk, &raw mut staging_bo);
    } else {
        vk_barrier.dstAccessMask = vk_barrier.srcAccessMask;
        vk_barrier.srcAccessMask = VK_ACCESS_TRANSFER_WRITE_BIT as VkAccessFlags;

        if (*dst_bo).host_synced {
            vk_barrier.dstAccessMask |= VK_ACCESS_HOST_READ_BIT as VkAccessFlags;
            bo_stage_flags |= VK_PIPELINE_STAGE_HOST_BIT as VkPipelineStageFlags;
        }

        ((*vk_info).vk_ops.vkCmdPipelineBarrier.unwrap_unchecked())(
            vk_command_buffer,
            VK_PIPELINE_STAGE_TRANSFER_BIT as VkPipelineStageFlags,
            bo_stage_flags,
            0 as VkDependencyFlags,
            0 as uint32_t,
            core::ptr::null(),
            1 as uint32_t,
            &raw mut vk_barrier,
            0 as uint32_t,
            core::ptr::null()
        );
        /* Start the download so we don't stall waiting for the result. */
        wined3d_context_vk_submit_command_buffer(
            context_vk,
            0 as ::core::ffi::c_uint,
            core::ptr::null(),
            core::ptr::null(),
            0 as ::core::ffi::c_uint,
            core::ptr::null(),
        );
    }
}

unsafe extern "C" fn wined3d_texture_vk_clear(
    texture_vk: *mut wined3d_texture_vk,
    sub_resource_idx: ::core::ffi::c_uint,
    context: *mut wined3d_context,
) -> bool {
    let sub_resource: *mut wined3d_texture_sub_resource =
        (*texture_vk).t.sub_resources.add(sub_resource_idx as usize);
    let context_vk: *mut wined3d_context_vk = wined3d_context_vk(context);
    let format: *const wined3d_format = (*texture_vk).t.resource.format;
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut depth_value: VkClearDepthStencilValue = core::mem::zeroed();
    let mut vk_command_buffer: VkCommandBuffer = core::ptr::null_mut();
    let mut vk_range: VkImageSubresourceRange = core::mem::zeroed();
    let mut colour_value: VkClearColorValue = core::mem::zeroed();
    let mut aspect_mask: VkImageAspectFlags = 0;
    let mut vk_image: VkImage = core::mem::zeroed();

    if (*texture_vk).t.resource.format_attrs & WINED3D_FORMAT_ATTR_COMPRESSED != 0 {
        let mut addr: wined3d_bo_address = core::mem::zeroed();
        let c: *mut wined3d_color = &raw mut (*sub_resource).clear_value.colour;

        if (*c).r != 0.0 || (*c).g != 0.0 || (*c).b != 0.0 || (*c).a != 0.0 {
            FIXME!(
                "Compressed resource %p is cleared to a non-zero color.\n",
                &raw mut (*texture_vk).t
            );
        }

        if wined3d_texture_prepare_location(
            &raw mut (*texture_vk).t,
            sub_resource_idx,
            context,
            WINED3D_LOCATION_SYSMEM,
        ) == 0
        {
            return false;
        }
        wined3d_texture_get_bo_address(
            &raw mut (*texture_vk).t,
            sub_resource_idx,
            &raw mut addr,
            WINED3D_LOCATION_SYSMEM,
        );
        memset(
            addr.addr as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            (*sub_resource).size as usize,
        );
        wined3d_texture_validate_location(
            &raw mut (*texture_vk).t,
            sub_resource_idx,
            WINED3D_LOCATION_SYSMEM,
        );
        return true;
    }

    vk_image = (*texture_vk).image.vk_image;

    vk_command_buffer = wined3d_context_vk_get_command_buffer(context_vk);
    if vk_command_buffer.is_null() {
        ERR!("Failed to get command buffer.\n");
        return false;
    }

    aspect_mask = vk_aspect_mask_from_format(format);

    vk_range.aspectMask = aspect_mask;
    vk_range.baseMipLevel = sub_resource_idx % (*texture_vk).t.level_count;
    vk_range.levelCount = 1;
    vk_range.baseArrayLayer = sub_resource_idx / (*texture_vk).t.level_count;
    vk_range.layerCount = 1;

    wined3d_context_vk_end_current_render_pass(context_vk);

    wined3d_context_vk_image_barrier(
        context_vk,
        vk_command_buffer,
        VK_PIPELINE_STAGE_ALL_COMMANDS_BIT,
        VK_PIPELINE_STAGE_TRANSFER_BIT,
        vk_access_mask_from_bind_flags((*texture_vk).t.resource.bind_flags),
        VK_ACCESS_TRANSFER_WRITE_BIT,
        (*texture_vk).layout,
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
        vk_image,
        &raw mut vk_range,
    );

    if (*format).depth_size != 0 || (*format).stencil_size != 0 {
        depth_value.depth = (*sub_resource).clear_value.__bindgen_anon_1.depth;
        depth_value.stencil = (*sub_resource).clear_value.__bindgen_anon_1.stencil;
        ((*vk_info).vk_ops.vkCmdClearDepthStencilImage.unwrap_unchecked())(
            vk_command_buffer,
            vk_image,
            VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
            &raw mut depth_value,
            1,
            &raw mut vk_range,
        );
    } else {
        wined3d_format_colour_to_vk(
            format,
            &raw mut (*sub_resource).clear_value.colour,
            &raw mut colour_value,
        );
        ((*vk_info).vk_ops.vkCmdClearColorImage.unwrap_unchecked())(
            vk_command_buffer,
            vk_image,
            VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
            &raw mut colour_value,
            1,
            &raw mut vk_range,
        );
    }

    wined3d_context_vk_image_barrier(
        context_vk,
        vk_command_buffer,
        VK_PIPELINE_STAGE_TRANSFER_BIT,
        VK_PIPELINE_STAGE_ALL_COMMANDS_BIT,
        VK_ACCESS_TRANSFER_WRITE_BIT,
        vk_access_mask_from_bind_flags((*texture_vk).t.resource.bind_flags),
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
        (*texture_vk).layout,
        vk_image,
        &raw mut vk_range,
    );
    wined3d_context_vk_reference_texture(context_vk, texture_vk);

    wined3d_texture_validate_location(
        &raw mut (*texture_vk).t,
        sub_resource_idx,
        WINED3D_LOCATION_TEXTURE_RGB,
    );
    return true;
}

unsafe extern "C" fn wined3d_texture_vk_load_texture(
    texture_vk: *mut wined3d_texture_vk,
    sub_resource_idx: ::core::ffi::c_uint,
    context: *mut wined3d_context,
) -> BOOL {
    let mut sub_resource: *mut wined3d_texture_sub_resource;
    let mut level: ::core::ffi::c_uint;
    let mut row_pitch: ::core::ffi::c_uint = core::mem::zeroed();
    let mut slice_pitch: ::core::ffi::c_uint = core::mem::zeroed();
    let mut data: wined3d_bo_address = core::mem::zeroed();
    let mut src_box: wined3d_box = core::mem::zeroed();

    sub_resource = (*texture_vk).t.sub_resources.add(sub_resource_idx as usize);

    if (*sub_resource).locations & WINED3D_LOCATION_CLEARED != 0 {
        if wined3d_texture_vk_clear(texture_vk, sub_resource_idx, context) == false {
            return (FALSE) as BOOL;
        }

        if (*sub_resource).locations & WINED3D_LOCATION_TEXTURE_RGB != 0 {
            return (TRUE) as BOOL;
        }
    }

    if !((*sub_resource).locations & wined3d_texture_sysmem_locations != 0) {
        ERR!(
            "Unimplemented load from %s.\n",
            wined3d_debug_location((*sub_resource).locations)
        );
        return (FALSE) as BOOL;
    }

    level = sub_resource_idx % (*texture_vk).t.level_count;
    wined3d_texture_get_memory(
        core::ptr::addr_of_mut!((*texture_vk).t),
        sub_resource_idx,
        context,
        core::ptr::addr_of_mut!(data),
    );
    wined3d_texture_get_level_box(
        core::ptr::addr_of_mut!((*texture_vk).t),
        level,
        core::ptr::addr_of_mut!(src_box),
    );
    wined3d_texture_get_pitch(
        core::ptr::addr_of_mut!((*texture_vk).t),
        level,
        core::ptr::addr_of_mut!(row_pitch),
        core::ptr::addr_of_mut!(slice_pitch),
    );
    wined3d_texture_vk_upload_data(
        context,
        wined3d_const_bo_address(core::ptr::addr_of_mut!(data)),
        (*texture_vk).t.resource.format,
        core::ptr::addr_of_mut!(src_box),
        row_pitch,
        slice_pitch,
        core::ptr::addr_of_mut!((*texture_vk).t),
        sub_resource_idx,
        WINED3D_LOCATION_TEXTURE_RGB,
        0,
        0,
        0,
    );

    return (TRUE) as BOOL;
}

unsafe extern "C" fn wined3d_texture_vk_load_sysmem(texture_vk: *mut wined3d_texture_vk, sub_resource_idx: u32, context: *mut wined3d_context, location: u32) -> BOOL {
    let mut sub_resource: *mut wined3d_texture_sub_resource = core::ptr::null_mut();
    let mut level: u32;
    let mut row_pitch: u32 = core::mem::zeroed();
    let mut slice_pitch: u32 = core::mem::zeroed();
    let mut data: wined3d_bo_address = core::mem::zeroed();
    let mut src_box: wined3d_box = core::mem::zeroed();

    sub_resource = (*texture_vk).t.sub_resources.add(sub_resource_idx as usize);
    if ((*sub_resource).locations & WINED3D_LOCATION_TEXTURE_RGB) == 0 {
        ERR!("Unimplemented load from %s.\n", wined3d_debug_location((*sub_resource).locations));
        return (FALSE) as BOOL;
    }

    level = sub_resource_idx % (*texture_vk).t.level_count;
    wined3d_texture_get_bo_address(core::ptr::addr_of_mut!((*texture_vk).t), sub_resource_idx, core::ptr::addr_of_mut!(data), location);
    wined3d_texture_get_level_box(core::ptr::addr_of_mut!((*texture_vk).t), level, core::ptr::addr_of_mut!(src_box));
    wined3d_texture_get_pitch(core::ptr::addr_of_mut!((*texture_vk).t), level, core::ptr::addr_of_mut!(row_pitch), core::ptr::addr_of_mut!(slice_pitch));
    wined3d_texture_vk_download_data(context, core::ptr::addr_of_mut!((*texture_vk).t), sub_resource_idx, WINED3D_LOCATION_TEXTURE_RGB,
            core::ptr::addr_of_mut!(src_box), core::ptr::addr_of_mut!(data), (*texture_vk).t.resource.format, 0, 0, 0, row_pitch, slice_pitch);

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_vk_prepare_texture(mut texture_vk: *mut wined3d_texture_vk, mut context_vk: *mut wined3d_context_vk) -> BOOL {
    let mut format_vk: *const wined3d_format_vk = core::ptr::null::<wined3d_format_vk>();
    let mut resource: *mut wined3d_resource = core::ptr::null_mut::<wined3d_resource>();
    let mut vk_command_buffer: VkCommandBuffer = core::ptr::null_mut::<VkCommandBuffer_T>();
    let mut vk_range: VkImageSubresourceRange = VkImageSubresourceRange {
        aspectMask: 0,
        baseMipLevel: 0,
        levelCount: 0,
        baseArrayLayer: 0,
        layerCount: 0,
    };
    let mut vk_usage: VkImageUsageFlags = 0;
    let mut vk_image_type: VkImageType = VK_IMAGE_TYPE_2D;
    let mut flags: core::ffi::c_uint = 0 as core::ffi::c_uint;

    if ((*texture_vk).t.flags & WINED3D_TEXTURE_RGB_ALLOCATED as uint32_t) != 0 {
        return (TRUE) as BOOL;
    }

    vk_command_buffer = wined3d_context_vk_get_command_buffer(context_vk);
    if vk_command_buffer.is_null() {
        ERR!("Failed to get command buffer.\n");
        return (FALSE) as BOOL;
    }

    resource = &raw mut (*texture_vk).t.resource;
    format_vk = wined3d_format_vk((*resource).format);

    if wined3d_format_is_typeless(&raw const (*format_vk).f) != 0
        || !(*texture_vk).t.swapchain.is_null()
        || ((*texture_vk).t.resource.bind_flags & WINED3D_BIND_UNORDERED_ACCESS as core::ffi::c_uint) != 0
    {
        /* For UAVs, we need this in case a clear necessitates creation of a new view
         * with a different format. */
        flags |= VK_IMAGE_CREATE_MUTABLE_FORMAT_BIT as core::ffi::c_uint;
    }

    match (*resource).type_ as core::ffi::c_uint {
        WINED3D_RTYPE_TEXTURE_1D => {
            vk_image_type = VK_IMAGE_TYPE_1D;
        }
        WINED3D_RTYPE_TEXTURE_2D => {
            vk_image_type = VK_IMAGE_TYPE_2D;
            if (*texture_vk).t.layer_count >= 6 as UINT
                && (*resource).width == (*resource).height
            {
                flags |= VK_IMAGE_CREATE_CUBE_COMPATIBLE_BIT as core::ffi::c_uint;
            }
        }
        WINED3D_RTYPE_TEXTURE_3D => {
            vk_image_type = VK_IMAGE_TYPE_3D;
            if ((*resource).bind_flags & (WINED3D_BIND_RENDER_TARGET | WINED3D_BIND_UNORDERED_ACCESS) as core::ffi::c_uint) != 0 {
                flags |= VK_IMAGE_CREATE_2D_ARRAY_COMPATIBLE_BIT as core::ffi::c_uint;
            }
        }
        _ => {
            ERR!("Invalid resource type %s.\n", debug_d3dresourcetype((*resource).type_));
            vk_image_type = VK_IMAGE_TYPE_2D;
        }
    }

    vk_usage = (VK_IMAGE_USAGE_TRANSFER_SRC_BIT | VK_IMAGE_USAGE_TRANSFER_DST_BIT) as VkImageUsageFlags;
    if ((*resource).bind_flags & WINED3D_BIND_SHADER_RESOURCE as core::ffi::c_uint) != 0 {
        vk_usage |= VK_IMAGE_USAGE_SAMPLED_BIT as VkImageUsageFlags;
    }
    if ((*resource).bind_flags & WINED3D_BIND_RENDER_TARGET as core::ffi::c_uint) != 0 {
        vk_usage |= VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT as VkImageUsageFlags;
    }
    if ((*resource).bind_flags & WINED3D_BIND_DEPTH_STENCIL as core::ffi::c_uint) != 0 {
        vk_usage |= VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT as VkImageUsageFlags;
    }
    if ((*resource).bind_flags & WINED3D_BIND_UNORDERED_ACCESS as core::ffi::c_uint) != 0 {
        vk_usage |= VK_IMAGE_USAGE_STORAGE_BIT as VkImageUsageFlags;
    }

    if ((*resource).bind_flags & WINED3D_BIND_UNORDERED_ACCESS as core::ffi::c_uint) != 0 {
        (*texture_vk).layout = VK_IMAGE_LAYOUT_GENERAL;
    } else if ((*resource).bind_flags & WINED3D_BIND_RENDER_TARGET as core::ffi::c_uint) != 0 {
        (*texture_vk).layout = VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL;
    } else if ((*resource).bind_flags & WINED3D_BIND_DEPTH_STENCIL as core::ffi::c_uint) != 0 {
        (*texture_vk).layout = VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
    } else if ((*resource).bind_flags & WINED3D_BIND_SHADER_RESOURCE as core::ffi::c_uint) != 0 {
        (*texture_vk).layout = VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL;
    } else {
        (*texture_vk).layout = VK_IMAGE_LAYOUT_GENERAL;
    }

    if wined3d_context_vk_create_image(
        context_vk,
        vk_image_type,
        vk_usage,
        (*format_vk).vk_format,
        (*resource).width as core::ffi::c_uint,
        (*resource).height as core::ffi::c_uint,
        (*resource).depth as core::ffi::c_uint,
        max!(1 as core::ffi::c_uint, wined3d_resource_get_sample_count(resource)),
        (*texture_vk).t.level_count,
        (*texture_vk).t.layer_count as core::ffi::c_uint,
        flags,
        &raw mut (*texture_vk).image,
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    /* We can't use a zero src access mask without synchronization2. Set the last-used bind mask to something
     * non-zero to avoid this. */
    (*texture_vk).bind_mask = (*resource).bind_flags as uint32_t;

    vk_range.aspectMask = vk_aspect_mask_from_format(&raw const (*format_vk).f);
    vk_range.baseMipLevel = 0 as uint32_t;
    vk_range.levelCount = VK_REMAINING_MIP_LEVELS as uint32_t;
    vk_range.baseArrayLayer = 0 as uint32_t;
    vk_range.layerCount = VK_REMAINING_ARRAY_LAYERS as uint32_t;

    wined3d_context_vk_reference_texture(context_vk, texture_vk);
    wined3d_context_vk_image_barrier(
        context_vk,
        vk_command_buffer,
        VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT as VkPipelineStageFlags,
        VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT as VkPipelineStageFlags,
        0 as VkAccessFlags,
        0 as VkAccessFlags,
        VK_IMAGE_LAYOUT_UNDEFINED,
        (*texture_vk).layout as VkImageLayout,
        (*texture_vk).image.vk_image,
        &raw mut vk_range,
    );

    (*texture_vk).t.flags |= WINED3D_TEXTURE_RGB_ALLOCATED as uint32_t;

    return (TRUE) as BOOL;
}

unsafe extern "C" fn wined3d_texture_vk_prepare_buffer_object(
    mut texture_vk: *mut wined3d_texture_vk,
    mut sub_resource_idx: ::core::ffi::c_uint,
    mut context_vk: *mut wined3d_context_vk,
) -> BOOL {
    let mut sub_resource: *mut wined3d_texture_sub_resource = core::ptr::null_mut();
    let mut bo: *mut wined3d_bo_vk = core::ptr::null_mut();

    sub_resource = core::ptr::addr_of_mut!((*texture_vk).t.sub_resources)
        .cast::<wined3d_texture_sub_resource>()
        .add(sub_resource_idx as usize);
    if !(*sub_resource).bo.is_null() {
        return (TRUE) as BOOL;
    }

    bo = malloc(core::mem::size_of::<wined3d_bo_vk>() as size_t) as *mut wined3d_bo_vk;
    if bo.is_null() {
        return (FALSE) as BOOL;
    }

    if wined3d_context_vk_create_bo(
        context_vk,
        (*sub_resource).size as VkDeviceSize,
        (VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_TRANSFER_SRC_BIT) as VkBufferUsageFlags,
        VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT as VkMemoryPropertyFlags,
        bo,
    ) == 0
    {
        free(bo as *mut ::core::ffi::c_void);
        return (FALSE) as BOOL;
    }

    /* Texture buffer objects receive a barrier to HOST_READ in wined3d_texture_vk_download_data(),
     * so they don't need it when they are mapped for reading. */
    (*bo).host_synced = true;
    (*sub_resource).bo = core::ptr::addr_of_mut!((*bo).b);

    TRACE!(
        "Created buffer object %p for texture %p, sub-resource %u.\n",
        bo,
        texture_vk,
        sub_resource_idx
    );

    return (TRUE) as BOOL;
}

unsafe extern "C" fn wined3d_texture_vk_prepare_location(
    texture: *mut wined3d_texture,
    sub_resource_idx: ::core::ffi::c_uint,
    context: *mut wined3d_context,
    location: ::core::ffi::c_uint,
) -> BOOL {
    match location {
        WINED3D_LOCATION_SYSMEM => {
            return if !(*(*texture).sub_resources.offset(sub_resource_idx as isize))
                .user_memory
                .is_null()
            {
                (TRUE as i32)
            } else {
                wined3d_resource_prepare_sysmem(core::ptr::addr_of_mut!((*texture).resource))
            };
        }
        WINED3D_LOCATION_TEXTURE_RGB => {
            return wined3d_texture_vk_prepare_texture(
                wined3d_texture_vk(texture),
                wined3d_context_vk(context),
            );
        }
        WINED3D_LOCATION_BUFFER => {
            return wined3d_texture_vk_prepare_buffer_object(
                wined3d_texture_vk(texture),
                sub_resource_idx,
                wined3d_context_vk(context),
            );
        }
        _ => {
            FIXME!(
                "Unhandled location %s.\n",
                wined3d_debug_location(location)
            );
            return (FALSE) as BOOL;
        }
    }
}

unsafe extern "C" fn wined3d_texture_vk_load_location(
    texture: *mut wined3d_texture,
    sub_resource_idx: ::core::ffi::c_uint,
    context: *mut wined3d_context,
    location: uint32_t,
) -> BOOL {
    if wined3d_texture_vk_prepare_location(texture, sub_resource_idx, context, location) == 0 {
        return (FALSE) as BOOL;
    }

    match location {
        WINED3D_LOCATION_TEXTURE_RGB => {
            return wined3d_texture_vk_load_texture(
                wined3d_texture_vk(texture),
                sub_resource_idx,
                context,
            );
        }
        WINED3D_LOCATION_SYSMEM | WINED3D_LOCATION_BUFFER => {
            return wined3d_texture_vk_load_sysmem(
                wined3d_texture_vk(texture),
                sub_resource_idx,
                context,
                location,
            );
        }
        _ => {
            FIXME!(
                "Unimplemented location %s.\n",
                wined3d_debug_location(location)
            );
            return (FALSE) as BOOL;
        }
    }
}

unsafe extern "C" fn wined3d_texture_vk_unload_location(
    mut texture: *mut wined3d_texture,
    mut context: *mut wined3d_context,
    mut location: ::core::ffi::c_uint,
) {
    let mut texture_vk: *mut wined3d_texture_vk = wined3d_texture_vk(texture);
    let mut context_vk: *mut wined3d_context_vk = wined3d_context_vk(context);
    let mut i: ::core::ffi::c_uint = 0;
    let mut sub_count: ::core::ffi::c_uint = 0;
    match location {
        WINED3D_LOCATION_TEXTURE_RGB => {
            if (*texture_vk).default_image_info.imageView != 0 {
                wined3d_context_vk_destroy_vk_image_view(
                    context_vk,
                    (*texture_vk).default_image_info.imageView,
                    (*texture_vk).image.command_buffer_id,
                );
                (*texture_vk).default_image_info.imageView = VK_NULL_HANDLE as VkImageView;
            }
            if (*texture_vk).image.vk_image != 0 {
                wined3d_context_vk_destroy_image(context_vk, &raw mut (*texture_vk).image);
            }
        }
        WINED3D_LOCATION_BUFFER => {
            sub_count = ((*texture).level_count as UINT).wrapping_mul((*texture).layer_count)
                as ::core::ffi::c_uint;
            i = 0 as ::core::ffi::c_uint;
            while i < sub_count {
                let mut sub_resource: *mut wined3d_texture_sub_resource =
                    (*texture).sub_resources.offset(i as isize)
                        as *mut wined3d_texture_sub_resource;
                if !(*sub_resource).bo.is_null() {
                    let mut bo_vk: *mut wined3d_bo_vk = wined3d_bo_vk((*sub_resource).bo);
                    wined3d_context_vk_destroy_bo(context_vk, bo_vk);
                    free(bo_vk as *mut ::core::ffi::c_void);
                    (*sub_resource).bo = ::core::ptr::null_mut::<wined3d_bo>();
                }
                i = i.wrapping_add(1);
            }
        }
        WINED3D_LOCATION_TEXTURE_SRGB | WINED3D_LOCATION_RB_MULTISAMPLE | WINED3D_LOCATION_RB_RESOLVED => {}
        _ => {
            ERR!(
                "Unhandled location %s.\n",
                wined3d_debug_location(location as uint32_t)
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_vk_init(texture_vk: *mut wined3d_texture_vk, device: *mut wined3d_device, desc: *const wined3d_resource_desc, layer_count: ::core::ffi::c_uint, level_count: ::core::ffi::c_uint, flags: uint32_t, parent: *mut ::core::ffi::c_void, parent_ops: *const wined3d_parent_ops) -> HRESULT {
    return wined3d_texture_init(
        core::ptr::addr_of_mut!((*texture_vk).t),
        desc,
        layer_count,
        level_count,
        flags,
        device,
        parent,
        parent_ops,
        texture_vk.add(1 as usize) as *mut core::ffi::c_void,
        core::ptr::addr_of!(wined3d_texture_vk_ops),
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_layout_from_bind_mask(texture_vk: *const wined3d_texture_vk, bind_mask: uint32_t) -> VkImageLayout {
    assert!(wined3d_popcount(bind_mask) == 1);

    /* We want to avoid switching between LAYOUT_GENERAL and other layouts. In Radeon GPUs (and presumably
     * others), this will trigger decompressing and recompressing the texture. We also hardcode the layout
     * into views when they are created. */
    if (*texture_vk).layout == VK_IMAGE_LAYOUT_GENERAL {
        return VK_IMAGE_LAYOUT_GENERAL;
    }

    match bind_mask {
        WINED3D_BIND_RENDER_TARGET => {
            return VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL;
        }
        WINED3D_BIND_DEPTH_STENCIL => {
            return VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
        }
        WINED3D_BIND_SHADER_RESOURCE => {
            return VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL;
        }
        _ => {
            ERR!("Unexpected bind mask %s.\n", wined3d_debug_bind_flags(bind_mask));
            return VK_IMAGE_LAYOUT_GENERAL;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_vk_barrier(
    texture_vk: *mut wined3d_texture_vk,
    context_vk: *mut wined3d_context_vk,
    bind_mask: u32,
) {
    let mut new_layout: VkImageLayout = VK_IMAGE_LAYOUT_UNDEFINED;
    let mut src_bind_mask: u32 = 0;

    new_layout = wined3d_layout_from_bind_mask(texture_vk, bind_mask);

    /* A layout transition is potentially a read-write operation, so even if we
     * prepare the texture to e.g. read only shader resource mode, we have to wait
     * for past operations to finish. */
    if bind_mask & !WINED3D_READ_ONLY_BIND_MASK != 0 || new_layout as u32 != (*texture_vk).layout as u32
    {
        src_bind_mask = (*texture_vk).bind_mask & WINED3D_READ_ONLY_BIND_MASK;
        if src_bind_mask == 0 {
            src_bind_mask = (*texture_vk).bind_mask;
        }

        (*texture_vk).bind_mask = bind_mask;
    } else if (*texture_vk).bind_mask & bind_mask != bind_mask {
        src_bind_mask = (*texture_vk).bind_mask & !WINED3D_READ_ONLY_BIND_MASK;
        (*texture_vk).bind_mask |= bind_mask;
    }

    if src_bind_mask != 0 {
        let mut vk_range: VkImageSubresourceRange = VkImageSubresourceRange {
            aspectMask: 0,
            baseMipLevel: 0,
            levelCount: 0,
            baseArrayLayer: 0,
            layerCount: 0,
        };

        vk_range.aspectMask = vk_aspect_mask_from_format((*texture_vk).t.resource.format);
        vk_range.baseMipLevel = 0;
        vk_range.levelCount = (VK_REMAINING_MIP_LEVELS) as u32;
        vk_range.baseArrayLayer = 0;
        vk_range.layerCount = (VK_REMAINING_ARRAY_LAYERS) as u32;

        wined3d_context_vk_image_barrier(
            context_vk,
            wined3d_context_vk_get_command_buffer(context_vk),
            vk_pipeline_stage_mask_from_bind_flags(src_bind_mask),
            vk_pipeline_stage_mask_from_bind_flags(bind_mask),
            vk_access_mask_from_bind_flags(src_bind_mask),
            vk_access_mask_from_bind_flags(bind_mask),
            (*texture_vk).layout as VkImageLayout,
            new_layout as VkImageLayout,
            (*texture_vk).image.vk_image,
            core::ptr::addr_of_mut!(vk_range),
        );

        (*texture_vk).layout = new_layout;
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_texture_vk_make_generic(
    mut texture_vk: *mut wined3d_texture_vk,
    mut context_vk: *mut wined3d_context_vk,
) {
    let mut vk_range: VkImageSubresourceRange = VkImageSubresourceRange {
        aspectMask: 0,
        baseMipLevel: 0,
        levelCount: 0,
        baseArrayLayer: 0,
        layerCount: 0,
    };
    if (*texture_vk).layout as ::core::ffi::c_uint == VK_IMAGE_LAYOUT_GENERAL as ::core::ffi::c_uint {
        return;
    }
    vk_range.aspectMask = vk_aspect_mask_from_format((*texture_vk).t.resource.format);
    vk_range.baseMipLevel = 0 as uint32_t;
    vk_range.levelCount = VK_REMAINING_MIP_LEVELS as uint32_t;
    vk_range.baseArrayLayer = 0 as uint32_t;
    vk_range.layerCount = VK_REMAINING_ARRAY_LAYERS as uint32_t;
    wined3d_context_vk_image_barrier(
        context_vk,
        wined3d_context_vk_get_command_buffer(context_vk),
        VK_PIPELINE_STAGE_ALL_COMMANDS_BIT as VkPipelineStageFlags,
        VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT as VkPipelineStageFlags,
        0 as VkAccessFlags,
        0 as VkAccessFlags,
        (*texture_vk).layout as VkImageLayout,
        VK_IMAGE_LAYOUT_GENERAL as VkImageLayout,
        (*texture_vk).image.vk_image,
        &raw mut vk_range,
    );
    (*texture_vk).layout = VK_IMAGE_LAYOUT_GENERAL;
    (*texture_vk).default_image_info.imageLayout = VK_IMAGE_LAYOUT_GENERAL;
}

unsafe extern "C" fn ffp_blitter_destroy(
    blitter: *mut wined3d_blitter,
    context: *mut wined3d_context,
) {
    let mut next: *mut wined3d_blitter = core::ptr::null_mut();
    next = (*blitter).next;
    if !next.is_null() {
        (((*(*next).ops).blitter_destroy).unwrap_unchecked())(next, context);
    }
    free(blitter as *mut core::ffi::c_void);
}

unsafe extern "C" fn is_full_clear(rtv: *const wined3d_rendertarget_view, draw_rect: *const RECT, clear_rect: *const RECT) -> bool {
    let mut height: ::core::ffi::c_uint = (*rtv).height;
    let mut width: ::core::ffi::c_uint = (*rtv).width;
    if (*draw_rect).left != 0
        || (*draw_rect).top != 0
        || ((*draw_rect).right as ::core::ffi::c_uint) < width
        || ((*draw_rect).bottom as ::core::ffi::c_uint) < height
    {
        return false;
    }
    if !clear_rect.is_null()
        && ((*clear_rect).left > 0
            || (*clear_rect).top > 0
            || ((*clear_rect).right as ::core::ffi::c_uint) < width
            || ((*clear_rect).bottom as ::core::ffi::c_uint) < height)
    {
        return false;
    }
    return true;
}

unsafe extern "C" fn ffp_blitter_clear_rendertargets(
    device: *mut wined3d_device,
    rt_count: u32,
    fb: *const wined3d_fb_state,
    rect_count: u32,
    clear_rect: *const RECT,
    draw_rect: *const RECT,
    flags: uint32_t,
    mut colour: *const wined3d_color,
    depth: f32,
    stencil: u32,
) {
    let mut rtv: *mut wined3d_rendertarget_view = if rt_count != 0 {
        (*fb).render_targets[0 as usize]
    } else {
        core::ptr::null_mut::<wined3d_rendertarget_view>()
    };
    let mut dsv: *mut wined3d_rendertarget_view = (*fb).depth_stencil;
    let mut state: *const wined3d_state = core::ptr::addr_of_mut!((*(*device).cs).state);
    let mut depth_stencil: *mut wined3d_texture = core::ptr::null_mut::<wined3d_texture>();
    let mut drawable_width: u32 = 0;
    let mut drawable_height: u32 = 0;
    let mut gl_info: *const wined3d_gl_info = core::ptr::null::<wined3d_gl_info>();
    let mut context_gl: *mut wined3d_context_gl = core::ptr::null_mut::<wined3d_context_gl>();
    let mut target: *mut wined3d_texture = core::ptr::null_mut::<wined3d_texture>();
    let mut colour_srgb: wined3d_color = wined3d_color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    let mut context: *mut wined3d_context = core::ptr::null_mut::<wined3d_context>();
    let mut clear_mask: GLbitfield = 0 as GLbitfield;
    let mut render_offscreen: bool = false;
    let mut i: u32 = 0;

    if !rtv.is_null()
        && (*(*rtv).resource).type_ as u32 != WINED3D_RTYPE_BUFFER as u32
    {
        target = texture_from_resource((*rtv).resource);
        context = context_acquire(device, target, (*rtv).sub_resource_idx);
    } else {
        context = context_acquire(
            device,
            core::ptr::null_mut::<wined3d_texture>(),
            0 as u32,
        );
    }
    context_gl = wined3d_context_gl(context);

    if !dsv.is_null()
        && (*(*dsv).resource).type_ as u32 != WINED3D_RTYPE_BUFFER as u32
    {
        depth_stencil = texture_from_resource((*dsv).resource);
    }

    if (*context_gl).valid() == 0 {
        context_release(context);
        WARN!("Invalid context, skipping clear.\n");
        return;
    }
    gl_info = (*context_gl).gl_info;

    /* When we're clearing parts of the drawable, make sure that the target
     * surface is well up to date in the drawable. After the clear we'll mark
     * the drawable up to date, so we have to make sure that this is true for
     * the cleared parts, and the untouched parts.
     *
     * If we're clearing the whole target there is no need to copy it into the
     * drawable, it will be overwritten anyway. If we're not clearing the
     * colour buffer we don't have to copy either since we're not going to set
     * the drawable up to date. We have to check all settings that limit the
     * clear area though. Do not bother checking all this if the destination
     * surface is in the drawable anyway. */
    i = 0 as u32;
    while i < rt_count {
        let rtv: *mut wined3d_rendertarget_view = (*fb).render_targets[i as usize];

        'body: {
            if !rtv.is_null() && (*(*rtv).format).id as u32 != WINED3DFMT_NULL as u32 {
                if (flags & WINED3DCLEAR_TARGET as uint32_t) != 0
                    && !(is_full_clear(
                        rtv,
                        draw_rect,
                        if rect_count != 0 {
                            clear_rect
                        } else {
                            core::ptr::null::<RECT>()
                        },
                    ))
                {
                    wined3d_rendertarget_view_load_location(
                        rtv,
                        context,
                        (*(*rtv).resource).draw_binding as uint32_t,
                    );
                } else {
                    wined3d_rendertarget_view_prepare_location(
                        rtv,
                        context,
                        (*(*rtv).resource).draw_binding as uint32_t,
                    );
                }
            }
        }
        i = i.wrapping_add(1);
    }

    if !target.is_null() {
        render_offscreen = wined3d_resource_is_offscreen(core::ptr::addr_of_mut!((*target).resource)) != 0;
        wined3d_rendertarget_view_get_drawable_size(
            rtv,
            context,
            core::ptr::addr_of_mut!(drawable_width),
            core::ptr::addr_of_mut!(drawable_height),
        );
    } else {
        let ds_level: u32 = (*dsv).sub_resource_idx % (*depth_stencil).level_count;

        render_offscreen = true;
        drawable_width = wined3d_texture_get_level_width(depth_stencil, ds_level);
        drawable_height = wined3d_texture_get_level_height(depth_stencil, ds_level);
    }

    if !depth_stencil.is_null() {
        let ds_location: DWORD = if render_offscreen {
            (*(*dsv).resource).draw_binding as DWORD
        } else {
            WINED3D_LOCATION_DRAWABLE as DWORD
        };

        if (flags & (WINED3DCLEAR_ZBUFFER as uint32_t | WINED3DCLEAR_STENCIL as uint32_t)) != 0
            && !(is_full_clear(
                dsv,
                draw_rect,
                if rect_count != 0 {
                    clear_rect
                } else {
                    core::ptr::null::<RECT>()
                },
            ))
        {
            wined3d_rendertarget_view_load_location(dsv, context, ds_location as uint32_t);
        } else {
            wined3d_rendertarget_view_prepare_location(dsv, context, ds_location as uint32_t);
        }

        if (flags & (WINED3DCLEAR_ZBUFFER as uint32_t | WINED3DCLEAR_STENCIL as uint32_t)) != 0 {
            wined3d_rendertarget_view_validate_location(dsv, ds_location as uint32_t);
            wined3d_rendertarget_view_invalidate_location(dsv, !(ds_location as uint32_t));
        }
    }

    if wined3d_context_gl_apply_clear_state(context_gl, state, rt_count, fb) == 0 {
        context_release(context);
        WARN!("Failed to apply clear state, skipping clear.\n");
        return;
    }

    /* Only set the values up once, as they are not changing. */
    if (flags & WINED3DCLEAR_STENCIL as uint32_t) != 0 {
        if (*gl_info).supported[EXT_STENCIL_TWO_SIDE as usize] != 0 {
            (*gl_info)
                .gl_ops
                .gl
                .p_glDisable
                .unwrap_unchecked()(GL_STENCIL_TEST_TWO_SIDE_EXT as GLenum);
        }
        (*gl_info)
            .gl_ops
            .gl
            .p_glStencilMask
            .unwrap_unchecked()(!(0 as GLuint));
        context_invalidate_state(context, STATE_DEPTH_STENCIL as u32);
        (*gl_info)
            .gl_ops
            .gl
            .p_glClearStencil
            .unwrap_unchecked()(stencil as GLint);
        checkGLcall!(gl_info, "glClearStencil");
        clear_mask = clear_mask | GL_STENCIL_BUFFER_BIT as GLbitfield;
    }

    if (flags & WINED3DCLEAR_ZBUFFER as uint32_t) != 0 {
        (*gl_info)
            .gl_ops
            .gl
            .p_glDepthMask
            .unwrap_unchecked()(GL_TRUE as GLboolean);
        context_invalidate_state(context, STATE_DEPTH_STENCIL as u32);
        if (*gl_info).supported[ARB_ES2_COMPATIBILITY as usize] != 0 {
            ((*gl_info).gl_ops.ext.p_glClearDepthf.unwrap_unchecked())(depth as GLfloat);
        } else {
            (*gl_info)
                .gl_ops
                .gl
                .p_glClearDepth
                .unwrap_unchecked()(depth as GLdouble);
        }
        checkGLcall!(gl_info, "glClearDepth");
        clear_mask = clear_mask | GL_DEPTH_BUFFER_BIT as GLbitfield;
    }

    if (flags & WINED3DCLEAR_TARGET as uint32_t) != 0 {
        i = 0 as u32;
        while i < rt_count {
            let rtv: *mut wined3d_rendertarget_view = (*fb).render_targets[i as usize];

            'body: {
                if rtv.is_null() {
                    break 'body;
                }

                if (*(*rtv).resource).type_ as u32 == WINED3D_RTYPE_BUFFER as u32 {
                    break 'body;
                }

                wined3d_rendertarget_view_validate_location(rtv, (*(*rtv).resource).draw_binding as uint32_t);
                wined3d_rendertarget_view_invalidate_location(rtv, !((*(*rtv).resource).draw_binding as uint32_t));
            }
            i = i.wrapping_add(1);
        }

        if (*gl_info).supported[ARB_FRAMEBUFFER_SRGB as usize] == 0
            && needs_srgb_write((*context).d3d_info, state, fb) != 0
        {
            if rt_count > 1 as u32 {
                WARN!("Clearing multiple sRGB render targets without GL_ARB_framebuffer_sRGB support, this might cause graphical issues.\n");
            }

            wined3d_colour_srgb_from_linear(core::ptr::addr_of_mut!(colour_srgb), colour);
            colour = core::ptr::addr_of_mut!(colour_srgb);
        }

        (*gl_info)
            .gl_ops
            .gl
            .p_glColorMask
            .unwrap_unchecked()(
            GL_TRUE as GLboolean,
            GL_TRUE as GLboolean,
            GL_TRUE as GLboolean,
            GL_TRUE as GLboolean,
        );
        context_invalidate_state(context, STATE_BLEND as u32);
        (*gl_info)
            .gl_ops
            .gl
            .p_glClearColor
            .unwrap_unchecked()(
            (*colour).r as GLfloat,
            (*colour).g as GLfloat,
            (*colour).b as GLfloat,
            (*colour).a as GLfloat,
        );
        checkGLcall!(gl_info, "glClearColor");
        clear_mask = clear_mask | GL_COLOR_BUFFER_BIT as GLbitfield;
    }

    if rect_count == 0 {
        if render_offscreen {
            (*gl_info)
                .gl_ops
                .gl
                .p_glScissor
                .unwrap_unchecked()(
                (*draw_rect).left as GLint,
                (*draw_rect).top as GLint,
                (*draw_rect).right as GLsizei - (*draw_rect).left as GLsizei,
                (*draw_rect).bottom as GLsizei - (*draw_rect).top as GLsizei,
            );
        } else {
            (*gl_info)
                .gl_ops
                .gl
                .p_glScissor
                .unwrap_unchecked()(
                (*draw_rect).left as GLint,
                drawable_height.wrapping_sub((*draw_rect).bottom as u32) as GLint,
                (*draw_rect).right as GLsizei - (*draw_rect).left as GLsizei,
                (*draw_rect).bottom as GLsizei - (*draw_rect).top as GLsizei,
            );
        }
        (*gl_info)
            .gl_ops
            .gl
            .p_glClear
            .unwrap_unchecked()(clear_mask);
    } else {
        let mut current_rect: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };

        /* Now process each rect in turn. */
        i = 0 as u32;
        while i < rect_count {
            /* Note that GL uses lower left, width/height. */
            IntersectRect(
                core::ptr::addr_of_mut!(current_rect),
                draw_rect as LPCRECT,
                clear_rect.offset(i as isize) as LPCRECT,
            );

            'body: {
                /* Tests show that rectangles where x1 > x2 or y1 > y2 are ignored
                 * silently. The rectangle is not cleared, no error is returned,
                 * but further rectangles are still cleared if they are valid. */
                if current_rect.left > current_rect.right || current_rect.top > current_rect.bottom {
                    TRACE!("Rectangle with negative dimensions, ignoring.\n");
                    break 'body;
                }

                if render_offscreen {
                    (*gl_info)
                        .gl_ops
                        .gl
                        .p_glScissor
                        .unwrap_unchecked()(
                        current_rect.left as GLint,
                        current_rect.top as GLint,
                        current_rect.right as GLsizei - current_rect.left as GLsizei,
                        current_rect.bottom as GLsizei - current_rect.top as GLsizei,
                    );
                } else {
                    (*gl_info)
                        .gl_ops
                        .gl
                        .p_glScissor
                        .unwrap_unchecked()(
                        current_rect.left as GLint,
                        drawable_height.wrapping_sub(current_rect.bottom as u32) as GLint,
                        current_rect.right as GLsizei - current_rect.left as GLsizei,
                        current_rect.bottom as GLsizei - current_rect.top as GLsizei,
                    );
                }
                (*gl_info)
                    .gl_ops
                    .gl
                    .p_glClear
                    .unwrap_unchecked()(clear_mask);
            }
            i = i.wrapping_add(1);
        }
    }

    (*context).scissor_rect_count = WINED3D_MAX_VIEWPORTS as u32;
    checkGLcall!(gl_info, "clear");

    if (flags & WINED3DCLEAR_TARGET as uint32_t) != 0
        && !(*target).swapchain.is_null()
        && (*(*target).swapchain).front_buffer == target
    {
        (*gl_info)
            .gl_ops
            .gl
            .p_glFlush
            .unwrap_unchecked()();
    }

    context_release(context);
}

unsafe extern "C" fn blitter_use_cpu_clear(mut view: *mut wined3d_rendertarget_view) -> bool {
    let mut resource: *mut wined3d_resource = ::core::ptr::null_mut::<wined3d_resource>();
    let mut texture: *mut wined3d_texture = ::core::ptr::null_mut::<wined3d_texture>();
    let mut locations: DWORD = 0;
    resource = (*view).resource;
    if (*resource).type_ as ::core::ffi::c_uint
        == WINED3D_RTYPE_BUFFER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ((*resource).access & WINED3D_RESOURCE_ACCESS_GPU) == 0;
    }
    texture = texture_from_resource(resource);
    locations = (*(*texture)
        .sub_resources
        .offset((*view).sub_resource_idx as isize))
    .locations;
    if (locations
        & (((*resource).map_binding as ::core::ffi::c_int | (WINED3D_LOCATION_DISCARDED as i32)) as DWORD))
        != 0
    {
        return ((*resource).access & WINED3D_RESOURCE_ACCESS_GPU) == 0
            || (*texture).resource.pin_sysmem() as ::core::ffi::c_int != 0;
    }
    return ((*resource).access & WINED3D_RESOURCE_ACCESS_GPU) == 0
        && ((*texture).flags & WINED3D_TEXTURE_CONVERTED as uint32_t) == 0;
}

unsafe extern "C" fn ffp_blitter_clear(blitter: *mut wined3d_blitter, device: *mut wined3d_device, rt_count: ::core::ffi::c_uint, fb: *const wined3d_fb_state, rect_count: ::core::ffi::c_uint, clear_rects: *const RECT, draw_rect: *const RECT, flags: uint32_t, colour: *const wined3d_color, depth: ::core::ffi::c_float, stencil: ::core::ffi::c_uint) {
    let mut rt_count = rt_count;
    let mut flags = flags;
    let mut view: *mut wined3d_rendertarget_view = core::ptr::null_mut::<wined3d_rendertarget_view>();
    let mut previous: *mut wined3d_rendertarget_view = core::ptr::null_mut::<wined3d_rendertarget_view>();
    let mut have_identical_size: bool = ((TRUE) != 0);
    let mut tmp_fb: wined3d_fb_state = wined3d_fb_state {
        render_targets: [core::ptr::null_mut::<wined3d_rendertarget_view>(); 8],
        depth_stencil: core::ptr::null_mut::<wined3d_rendertarget_view>(),
    };
    let mut next_rt_count: ::core::ffi::c_uint = 0;
    let mut next: *mut wined3d_blitter = core::ptr::null_mut::<wined3d_blitter>();
    let mut next_flags: DWORD = 0;
    let mut i: ::core::ffi::c_uint = 0;

    if flags & WINED3DCLEAR_TARGET != 0 {
        i = 0;
        while i < rt_count {
            view = (*fb).render_targets[i as usize];
            if view.is_null() {
                i = i.wrapping_add(1);
                continue;
            }
            if blitter_use_cpu_clear(view) as ::core::ffi::c_int != 0
                || ((*(*view).resource).bind_flags & WINED3D_BIND_RENDER_TARGET == 0
                    && (*view).format_caps & WINED3D_FORMAT_CAP_FBO_ATTACHABLE == 0)
            {
                next_flags |= WINED3DCLEAR_TARGET;
                flags &= !WINED3DCLEAR_TARGET;
                next_rt_count = rt_count;
                rt_count = 0;
                break;
            }
            i = i.wrapping_add(1);
        }
    }

    if flags & (WINED3DCLEAR_ZBUFFER | WINED3DCLEAR_STENCIL) != 0
        && {
            view = (*fb).depth_stencil;
            !view.is_null()
        }
        && ((*(*view).format).depth_size == 0 || flags & WINED3DCLEAR_ZBUFFER != 0)
        && ((*(*view).format).stencil_size == 0 || flags & WINED3DCLEAR_STENCIL != 0)
        && blitter_use_cpu_clear(view) as ::core::ffi::c_int != 0
    {
        next_flags |= flags & (WINED3DCLEAR_ZBUFFER | WINED3DCLEAR_STENCIL);
        flags &= !(WINED3DCLEAR_ZBUFFER | WINED3DCLEAR_STENCIL);
    }

    if flags != 0 {
        i = 0;
        while i < rt_count {
            view = (*fb).render_targets[i as usize];
            if view.is_null() {
                i = i.wrapping_add(1);
                continue;
            }
            if !previous.is_null()
                && ((*previous).width != (*view).width || (*previous).height != (*view).height)
            {
                have_identical_size = false;
            }
            previous = view;
            i = i.wrapping_add(1);
        }
        if flags & (WINED3DCLEAR_ZBUFFER | WINED3DCLEAR_STENCIL) != 0 {
            view = (*fb).depth_stencil;

            if !previous.is_null()
                && ((*previous).width != (*view).width || (*previous).height != (*view).height)
            {
                have_identical_size = false;
            }
        }

        if have_identical_size {
            ffp_blitter_clear_rendertargets(device, rt_count, fb, rect_count,
                clear_rects, draw_rect, flags, colour, depth, stencil);
        } else {
            i = 0;
            while i < rt_count {
                view = (*fb).render_targets[i as usize];
                if view.is_null() {
                    i = i.wrapping_add(1);
                    continue;
                }
                tmp_fb.render_targets[0 as usize] = view;
                tmp_fb.depth_stencil = core::ptr::null_mut::<wined3d_rendertarget_view>();
                ffp_blitter_clear_rendertargets(device, 1, core::ptr::addr_of_mut!(tmp_fb), rect_count,
                    clear_rects, draw_rect, WINED3DCLEAR_TARGET, colour, depth, stencil);
                i = i.wrapping_add(1);
            }
            if flags & (WINED3DCLEAR_ZBUFFER | WINED3DCLEAR_STENCIL) != 0 {
                tmp_fb.render_targets[0 as usize] = core::ptr::null_mut::<wined3d_rendertarget_view>();
                tmp_fb.depth_stencil = (*fb).depth_stencil;
                ffp_blitter_clear_rendertargets(device, 0, core::ptr::addr_of_mut!(tmp_fb), rect_count,
                    clear_rects, draw_rect, flags & !WINED3DCLEAR_TARGET, colour, depth, stencil);
            }
        }
    }

    if next_flags != 0 && {
        next = (*blitter).next;
        !next.is_null()
    } {
        ((*(*next).ops).blitter_clear.unwrap_unchecked())(
            next,
            device,
            next_rt_count,
            fb,
            rect_count,
            clear_rects,
            draw_rect,
            next_flags,
            colour,
            depth,
            stencil,
        );
    }
}

unsafe extern "C" fn ffp_blitter_blit(blitter: *mut wined3d_blitter, op: wined3d_blit_op, context: *mut wined3d_context, src_texture: *mut wined3d_texture, src_sub_resource_idx: u32, src_location: DWORD, src_rect: *const RECT, dst_texture: *mut wined3d_texture, dst_sub_resource_idx: u32, dst_location: DWORD, dst_rect: *const RECT, colour_key: *const wined3d_color_key, filter: wined3d_texture_filter_type, resolve_format: *const wined3d_format) -> DWORD {
    let mut next: *mut wined3d_blitter;
    next = (*blitter).next;
    if next.is_null() {
        ERR!("No blitter to handle blit op %#x.\n", op);
        return dst_location;
    }
    return (((*(*next).ops).blitter_blit).unwrap_unchecked())(next, op, context, src_texture, src_sub_resource_idx, src_location, src_rect, dst_texture, dst_sub_resource_idx, dst_location, dst_rect, colour_key, filter, resolve_format);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_ffp_blitter_create(next: *mut *mut wined3d_blitter, gl_info: *const wined3d_gl_info) {
    let mut blitter: *mut wined3d_blitter = core::ptr::null_mut();

    blitter = malloc(core::mem::size_of::<wined3d_blitter>()) as *mut wined3d_blitter;
    if blitter.is_null() {
        return;
    }

    (*blitter).ops = core::ptr::addr_of!(ffp_blitter_ops);
    (*blitter).next = *next;
    *next = blitter;
}

unsafe extern "C" fn fbo_blitter_destroy(
    blitter: *mut wined3d_blitter,
    context: *mut wined3d_context,
) {
    let mut next: *mut wined3d_blitter = core::ptr::null_mut();
    next = (*blitter).next;
    if !next.is_null() {
        (((*(*next).ops).blitter_destroy).unwrap_unchecked())(next, context);
    }
    free(blitter as *mut core::ffi::c_void);
}

unsafe extern "C" fn fbo_blitter_clear(
    blitter: *mut wined3d_blitter,
    device: *mut wined3d_device,
    rt_count: core::ffi::c_uint,
    fb: *const wined3d_fb_state,
    rect_count: core::ffi::c_uint,
    clear_rects: *const RECT,
    draw_rect: *const RECT,
    flags: u32,
    colour: *const wined3d_color,
    depth: f32,
    stencil: core::ffi::c_uint,
) {
    let mut next: *mut wined3d_blitter = core::ptr::null_mut();
    next = (*blitter).next;
    if !next.is_null() {
        (*(*next).ops).blitter_clear.unwrap_unchecked()(
            next,
            device,
            rt_count,
            fb,
            rect_count,
            clear_rects,
            draw_rect,
            flags,
            colour,
            depth,
            stencil,
        );
    }
}

unsafe extern "C" fn fbo_blitter_blit(
    mut blitter: *mut wined3d_blitter,
    mut op: wined3d_blit_op,
    mut context: *mut wined3d_context,
    mut src_texture: *mut wined3d_texture,
    mut src_sub_resource_idx: u32,
    mut src_location: DWORD,
    mut src_rect: *const RECT,
    mut dst_texture: *mut wined3d_texture,
    mut dst_sub_resource_idx: u32,
    mut dst_location: DWORD,
    mut dst_rect: *const RECT,
    mut colour_key: *const wined3d_color_key,
    mut filter: wined3d_texture_filter_type,
    mut resolve_format: *const wined3d_format,
) -> DWORD {
    let mut context_gl: *mut wined3d_context_gl = wined3d_context_gl(context);
    let mut src_resource: *mut wined3d_resource = core::ptr::null_mut();
    let mut dst_resource: *mut wined3d_resource = core::ptr::null_mut();
    let mut blit_op: wined3d_blit_op = op;
    let mut device: *mut wined3d_device = core::ptr::null_mut();
    let mut next: *mut wined3d_blitter = core::ptr::null_mut();

    TRACE!(
        "blitter %p, op %#x, context %p, src_texture %p, src_sub_resource_idx %u, src_location %s, src_rect %s, dst_texture %p, dst_sub_resource_idx %u, dst_location %s, dst_rect %s, colour_key %p, filter %s, resolve_format %p.\n",
        blitter,
        op as u32,
        context,
        src_texture,
        src_sub_resource_idx,
        wined3d_debug_location(src_location as uint32_t),
        wine_dbgstr_rect(src_rect),
        dst_texture,
        dst_sub_resource_idx,
        wined3d_debug_location(dst_location as uint32_t),
        wine_dbgstr_rect(dst_rect),
        colour_key,
        debug_d3dtexturefiltertype(filter),
        resolve_format
    );

    src_resource = core::ptr::addr_of_mut!((*src_texture).resource);
    dst_resource = core::ptr::addr_of_mut!((*dst_texture).resource);

    device = (*dst_resource).device;

    if blit_op == WINED3D_BLIT_OP_RAW_BLIT
        && (*(*dst_resource).format).id == (*(*src_resource).format).id
    {
        if (*(*dst_resource).format).depth_size != 0 || (*(*dst_resource).format).stencil_size != 0 {
            blit_op = WINED3D_BLIT_OP_DEPTH_BLIT;
        } else {
            blit_op = WINED3D_BLIT_OP_COLOR_BLIT;
        }
    }

    if !fbo_blitter_supported(
        blit_op,
        (*context_gl).gl_info,
        src_resource,
        src_location,
        dst_resource,
        dst_location,
    ) {
        next = (*blitter).next;
        if next.is_null() {
            ERR!("No blitter to handle blit op %#x.\n", op as u32);
            return dst_location;
        }

        TRACE!("Forwarding to blitter %p.\n", next);
        return (*(*next).ops)
            .blitter_blit
            .unwrap_unchecked()(
            next,
            op,
            context,
            src_texture,
            src_sub_resource_idx,
            src_location,
            src_rect,
            dst_texture,
            dst_sub_resource_idx,
            dst_location,
            dst_rect,
            colour_key,
            filter,
            resolve_format,
        );
    }

    if blit_op == WINED3D_BLIT_OP_COLOR_BLIT {
        TRACE!("Colour blit.\n");
        texture2d_blt_fbo(
            device,
            context,
            filter,
            src_texture,
            src_sub_resource_idx,
            src_location,
            src_rect,
            dst_texture,
            dst_sub_resource_idx,
            dst_location,
            dst_rect,
            resolve_format,
        );
        return dst_location;
    }

    if blit_op == WINED3D_BLIT_OP_DEPTH_BLIT {
        TRACE!("Depth/stencil blit.\n");
        texture2d_depth_blt_fbo(
            device,
            context,
            src_texture,
            src_sub_resource_idx,
            src_location,
            src_rect,
            dst_texture,
            dst_sub_resource_idx,
            dst_location,
            dst_rect,
        );
        return dst_location;
    }

    ERR!("This blitter does not implement blit op %#x.\n", blit_op as u32);
    return dst_location;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_fbo_blitter_create(
    next: *mut *mut wined3d_blitter,
    gl_info: *const wined3d_gl_info,
) {
    let mut blitter: *mut wined3d_blitter = core::ptr::null_mut();

    blitter = malloc(core::mem::size_of::<wined3d_blitter>()) as *mut wined3d_blitter;
    if blitter.is_null() {
        return;
    }

    (*blitter).ops = core::ptr::addr_of!(fbo_blitter_ops);
    (*blitter).next = *next;
    *next = blitter;
}

unsafe extern "C" fn raw_blitter_destroy(
    blitter: *mut wined3d_blitter,
    context: *mut wined3d_context,
) {
    let mut next: *mut wined3d_blitter;

    next = (*blitter).next;
    if !next.is_null() {
        (((*(*next).ops).blitter_destroy).unwrap_unchecked())(next, context);
    }

    free(blitter as *mut core::ffi::c_void);
}

unsafe extern "C" fn raw_blitter_clear(blitter: *mut wined3d_blitter, device: *mut wined3d_device, rt_count: u32, fb: *const wined3d_fb_state, rect_count: u32, clear_rects: *const RECT, draw_rect: *const RECT, flags: u32, colour: *const wined3d_color, depth: f32, stencil: u32) {
    let mut next: *mut wined3d_blitter = core::ptr::null_mut();

    next = (*blitter).next;
    if next.is_null() {
        ERR!("No blitter to handle clear.\n");
        return;
    }

    (((*(*next).ops).blitter_clear).unwrap_unchecked())(next, device, rt_count, fb, rect_count, clear_rects, draw_rect, flags, colour, depth, stencil);
}

unsafe extern "C" fn gl_formats_compatible(
    src_texture: *mut wined3d_texture,
    src_location: DWORD,
    dst_texture: *mut wined3d_texture,
    dst_location: DWORD,
) -> bool {
    let mut src_internal: GLuint = 0;
    let mut dst_internal: GLuint = 0;
    let mut src_ds: bool = false;
    let mut dst_ds: bool = false;
    src_ds = (*(*src_texture).resource.format).depth_size as ::core::ffi::c_int != 0
        || (*(*src_texture).resource.format).stencil_size as ::core::ffi::c_int != 0;
    dst_ds = (*(*dst_texture).resource.format).depth_size as ::core::ffi::c_int != 0
        || (*(*dst_texture).resource.format).stencil_size as ::core::ffi::c_int != 0;
    if src_ds as ::core::ffi::c_int == dst_ds as ::core::ffi::c_int {
        return true;
    }
    src_internal = wined3d_gl_get_internal_format(
        core::ptr::addr_of_mut!((*src_texture).resource),
        wined3d_format_gl((*src_texture).resource.format),
        src_location == WINED3D_LOCATION_TEXTURE_SRGB,
    );
    dst_internal = wined3d_gl_get_internal_format(
        core::ptr::addr_of_mut!((*dst_texture).resource),
        wined3d_format_gl((*dst_texture).resource.format),
        dst_location == WINED3D_LOCATION_TEXTURE_SRGB,
    );
    return src_internal == dst_internal;
}

unsafe extern "C" fn raw_blitter_blit(
    blitter: *mut wined3d_blitter,
    op: wined3d_blit_op,
    context: *mut wined3d_context,
    src_texture: *mut wined3d_texture,
    src_sub_resource_idx: ::core::ffi::c_uint,
    src_location: DWORD,
    src_rect: *const RECT,
    dst_texture: *mut wined3d_texture,
    dst_sub_resource_idx: ::core::ffi::c_uint,
    dst_location: DWORD,
    dst_rect: *const RECT,
    colour_key: *const wined3d_color_key,
    filter: wined3d_texture_filter_type,
    resolve_format: *const wined3d_format,
) -> DWORD {
    let src_texture_gl: *mut wined3d_texture_gl = wined3d_texture_gl(src_texture);
    let dst_texture_gl: *mut wined3d_texture_gl = wined3d_texture_gl(dst_texture);
    let context_gl: *mut wined3d_context_gl = wined3d_context_gl(context);
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut src_level: ::core::ffi::c_uint = 0;
    let mut src_layer: ::core::ffi::c_uint = 0;
    let mut dst_level: ::core::ffi::c_uint = 0;
    let mut dst_layer: ::core::ffi::c_uint = 0;
    let mut next: *mut wined3d_blitter = ::core::ptr::null_mut::<wined3d_blitter>();
    let mut src_name: GLuint = 0;
    let mut dst_name: GLuint = 0;
    let mut location: DWORD = 0;

    if (op as u32) != (WINED3D_BLIT_OP_RAW_BLIT as u32)
        || !gl_formats_compatible(src_texture, src_location, dst_texture, dst_location)
        || (((*src_texture).resource.format_attrs | (*dst_texture).resource.format_attrs)
            & (WINED3D_FORMAT_ATTR_HEIGHT_SCALE as u32))
            != 0
        || ((*(*src_texture).resource.format).id == (*(*dst_texture).resource.format).id
            && ((src_location & ((WINED3D_LOCATION_TEXTURE_RGB | WINED3D_LOCATION_TEXTURE_SRGB) as DWORD)) == 0
                || (dst_location & ((WINED3D_LOCATION_TEXTURE_RGB | WINED3D_LOCATION_TEXTURE_SRGB) as DWORD))
                    == 0))
    {
        next = (*blitter).next;
        if next.is_null() {
            ERR!("No blitter to handle blit op %#x.\n", op as u32);
            return dst_location;
        }
        TRACE!("Forwarding to blitter %p.\n", next);
        return (*(*next).ops)
            .blitter_blit
            .unwrap_unchecked()(
            next,
            op,
            context,
            src_texture,
            src_sub_resource_idx,
            src_location,
            src_rect,
            dst_texture,
            dst_sub_resource_idx,
            dst_location,
            dst_rect,
            colour_key,
            filter,
            resolve_format,
        );
    }
    TRACE!("Blit using ARB_copy_image.\n");

    src_level = src_sub_resource_idx % (*src_texture).level_count;
    src_layer = src_sub_resource_idx / (*src_texture).level_count;

    dst_level = dst_sub_resource_idx % (*dst_texture).level_count;
    dst_layer = dst_sub_resource_idx / (*dst_texture).level_count;

    location = src_location & ((WINED3D_LOCATION_TEXTURE_RGB | WINED3D_LOCATION_TEXTURE_SRGB) as DWORD);
    if location == 0 {
        location = (if ((*src_texture).flags & (WINED3D_TEXTURE_IS_SRGB as uint32_t)) != 0 {
            WINED3D_LOCATION_TEXTURE_SRGB
        } else {
            WINED3D_LOCATION_TEXTURE_RGB
        }) as DWORD;
    }
    if wined3d_texture_load_location(src_texture, src_sub_resource_idx, context, location as uint32_t) == 0 {
        ERR!(
            "Failed to load the source sub-resource into %s.\n",
            wined3d_debug_location(location as uint32_t)
        );
    }
    src_name = wined3d_texture_gl_get_texture_name(
        src_texture_gl,
        context,
        location == (WINED3D_LOCATION_TEXTURE_SRGB as DWORD),
    );

    location = dst_location & ((WINED3D_LOCATION_TEXTURE_RGB | WINED3D_LOCATION_TEXTURE_SRGB) as DWORD);
    if location == 0 {
        location = (if ((*dst_texture).flags & (WINED3D_TEXTURE_IS_SRGB as uint32_t)) != 0 {
            WINED3D_LOCATION_TEXTURE_SRGB
        } else {
            WINED3D_LOCATION_TEXTURE_RGB
        }) as DWORD;
    }
    if wined3d_texture_is_full_rect(dst_texture, dst_level, dst_rect) {
        if wined3d_texture_prepare_location(dst_texture, dst_sub_resource_idx, context, location as ::core::ffi::c_uint) == 0 {
            ERR!(
                "Failed to prepare the destination sub-resource into %s.\n",
                wined3d_debug_location(location as uint32_t)
            );
        }
    } else {
        if wined3d_texture_load_location(dst_texture, dst_sub_resource_idx, context, location as uint32_t) == 0 {
            ERR!(
                "Failed to load the destination sub-resource into %s.\n",
                wined3d_debug_location(location as uint32_t)
            );
        }
    }
    dst_name = wined3d_texture_gl_get_texture_name(
        dst_texture_gl,
        context,
        location == (WINED3D_LOCATION_TEXTURE_SRGB as DWORD),
    );

    ((*gl_info).gl_ops.ext.p_glCopyImageSubData.unwrap_unchecked())(
        src_name,
        (*src_texture_gl).target,
        src_level as GLint,
        (*src_rect).left as GLint,
        (*src_rect).top as GLint,
        src_layer as GLint,
        dst_name,
        (*dst_texture_gl).target,
        dst_level as GLint,
        (*dst_rect).left as GLint,
        (*dst_rect).top as GLint,
        dst_layer as GLint,
        (*src_rect).right as GLsizei - (*src_rect).left as GLsizei,
        (*src_rect).bottom as GLsizei - (*src_rect).top as GLsizei,
        1 as GLsizei,
    );
    checkGLcall!(gl_info, "copy image data");

    wined3d_texture_validate_location(dst_texture, dst_sub_resource_idx, location as uint32_t);
    wined3d_texture_invalidate_location(dst_texture, dst_sub_resource_idx, !(location as uint32_t));
    if wined3d_texture_load_location(dst_texture, dst_sub_resource_idx, context, dst_location as uint32_t) == 0 {
        ERR!(
            "Failed to load the destination sub-resource into %s.\n",
            wined3d_debug_location(dst_location as uint32_t)
        );
    }

    return dst_location | location;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_raw_blitter_create(next: *mut *mut wined3d_blitter, gl_info: *const wined3d_gl_info) {
    let mut blitter: *mut wined3d_blitter = core::ptr::null_mut();

    if (*gl_info).supported[ARB_COPY_IMAGE as usize] == 0 {
        return;
    }

    blitter = malloc(core::mem::size_of::<wined3d_blitter>()) as *mut wined3d_blitter;
    if blitter.is_null() {
        return;
    }

    (*blitter).ops = core::ptr::addr_of!(raw_blitter_ops);
    (*blitter).next = *next;
    *next = blitter;
}

unsafe extern "C" fn vk_blitter_destroy(blitter: *mut wined3d_blitter, context: *mut wined3d_context) {
    let mut next: *mut wined3d_blitter;

    next = (*blitter).next;
    if !next.is_null() {
        ((*(*next).ops).blitter_destroy.unwrap_unchecked())(next, context);
    }

    free(blitter as *mut core::ffi::c_void);
}

unsafe extern "C" fn vk_blitter_clear_rendertargets(context_vk: *mut wined3d_context_vk, mut rt_count: ::core::ffi::c_uint, fb: *const wined3d_fb_state, rect_count: ::core::ffi::c_uint, clear_rects: *const RECT, draw_rect: *const RECT, mut flags: uint32_t, colour: *const wined3d_color, depth: ::core::ffi::c_float, stencil: ::core::ffi::c_uint) {
    let mut i: ::core::ffi::c_uint = 0;
    let mut attachment_count: ::core::ffi::c_uint = 0;
    let mut immediate_rt_count: ::core::ffi::c_uint = 0;
    let mut delay_count: ::core::ffi::c_uint = 0;
    let mut clear_values: [VkClearValue; (WINED3D_MAX_RENDER_TARGETS + 1) as _] = core::mem::zeroed();
    let mut views: [VkImageView; (WINED3D_MAX_RENDER_TARGETS + 1) as _] = core::mem::zeroed();
    let mut rtv_vk: *mut wined3d_rendertarget_view_vk = core::ptr::null_mut();
    let mut view: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut vk_info: *const wined3d_vk_info = core::ptr::null();
    let mut immediate_fb: wined3d_fb_state = core::mem::zeroed();
    let mut device_vk: *mut wined3d_device_vk = core::ptr::null_mut();
    let mut vk_command_buffer: VkCommandBuffer = core::ptr::null_mut();
    let mut begin_desc: VkRenderPassBeginInfo = core::mem::zeroed();
    let mut fb_desc: VkFramebufferCreateInfo = core::mem::zeroed();
    let mut vk_framebuffer: VkFramebuffer = core::mem::zeroed();
    let mut vk_render_pass: VkRenderPass = core::mem::zeroed();
    let mut depth_stencil: bool = false;
    let mut layer_count: ::core::ffi::c_uint = 0;
    let mut c: *mut VkClearColorValue = core::ptr::null_mut();
    let mut vr: VkResult = core::mem::zeroed();
    let mut r: RECT = core::mem::zeroed();

    device_vk = wined3d_device_vk((*context_vk).c.device);
    vk_info = (*context_vk).vk_info;

    if !(flags & WINED3DCLEAR_TARGET as uint32_t != 0) {
        rt_count = 0;
    }

    i = 0;
    attachment_count = 0;
    layer_count = 1;
    while i < rt_count {
        view = (*fb).render_targets[i as usize];
        if !view.is_null() {
            if is_full_clear(view, draw_rect, clear_rects) as ::core::ffi::c_int != 0
                && (wined3d_format_is_typeless((*(*view).resource).format) == 0
                    || ((*colour).r == 0.
                        && (*colour).g == 0.
                        && (*colour).b == 0.
                        && (*colour).a == 0.))
            {
                let texture: *mut wined3d_texture = texture_from_resource((*view).resource);
                wined3d_rendertarget_view_validate_location(view, WINED3D_LOCATION_CLEARED as uint32_t);
                wined3d_rendertarget_view_invalidate_location(view, !WINED3D_LOCATION_CLEARED as uint32_t);
                (*(*texture)
                    .sub_resources
                    .offset((*view).sub_resource_idx as isize))
                .clear_value
                .colour = *colour;
                delay_count = delay_count.wrapping_add(1);
            } else {
                wined3d_rendertarget_view_load_location(
                    view,
                    core::ptr::addr_of_mut!((*context_vk).c),
                    (*(*view).resource).draw_binding as uint32_t,
                );
                wined3d_rendertarget_view_validate_location(view, (*(*view).resource).draw_binding as uint32_t);
                wined3d_rendertarget_view_invalidate_location(
                    view,
                    !((*(*view).resource).draw_binding as ::core::ffi::c_int) as uint32_t,
                );

                let fresh19 = immediate_rt_count;
                immediate_rt_count = immediate_rt_count.wrapping_add(1);
                immediate_fb.render_targets[fresh19 as usize] = view;
                rtv_vk = wined3d_rendertarget_view_vk(view);
                views[attachment_count as usize] = wined3d_rendertarget_view_vk_get_image_view(rtv_vk, context_vk);
                wined3d_rendertarget_view_vk_barrier(rtv_vk, context_vk, WINED3D_BIND_RENDER_TARGET as uint32_t);

                c = core::ptr::addr_of_mut!(clear_values[attachment_count as usize].color);
                wined3d_format_colour_to_vk((*view).format, colour, c);

                if (*view).layer_count > layer_count {
                    layer_count = (*view).layer_count;
                }

                attachment_count = attachment_count.wrapping_add(1);
            }
        }
        i = i.wrapping_add(1);
    }

    if attachment_count == 0 {
        flags &= !(WINED3DCLEAR_TARGET as uint32_t);
    }

    if (flags & (WINED3DCLEAR_ZBUFFER | WINED3DCLEAR_STENCIL) as uint32_t) != 0 && {
        view = (*fb).depth_stencil;
        !view.is_null()
    } {
        let mut full_flags: DWORD = 0 as DWORD;

        if (*(*view).format).depth_size != 0 {
            full_flags = WINED3DCLEAR_ZBUFFER as DWORD;
        }
        if (*(*view).format).stencil_size != 0 {
            full_flags |= WINED3DCLEAR_STENCIL as DWORD;
        }

        if !is_full_clear(view, draw_rect, clear_rects)
            || (flags & full_flags as uint32_t) != full_flags as uint32_t
            || (wined3d_format_is_typeless((*(*view).resource).format) != 0
                && (depth != 0. || stencil != 0))
        {
            wined3d_rendertarget_view_load_location(
                view,
                core::ptr::addr_of_mut!((*context_vk).c),
                (*(*view).resource).draw_binding as uint32_t,
            );
            wined3d_rendertarget_view_validate_location(view, (*(*view).resource).draw_binding as uint32_t);
            wined3d_rendertarget_view_invalidate_location(
                view,
                !((*(*view).resource).draw_binding as ::core::ffi::c_int) as uint32_t,
            );

            immediate_fb.depth_stencil = view;
            rtv_vk = wined3d_rendertarget_view_vk(view);
            views[attachment_count as usize] = wined3d_rendertarget_view_vk_get_image_view(rtv_vk, context_vk);
            wined3d_rendertarget_view_vk_barrier(rtv_vk, context_vk, WINED3D_BIND_DEPTH_STENCIL as uint32_t);

            clear_values[attachment_count as usize].depthStencil.depth = depth;
            clear_values[attachment_count as usize].depthStencil.stencil = stencil;

            if (*view).layer_count > layer_count {
                layer_count = (*view).layer_count;
            }

            depth_stencil = true;
            attachment_count = attachment_count.wrapping_add(1);
        } else {
            let texture: *mut wined3d_texture = texture_from_resource((*view).resource);
            (*(*texture)
                .sub_resources
                .offset((*view).sub_resource_idx as isize))
            .clear_value
            .__bindgen_anon_1.depth = depth;
            (*(*texture)
                .sub_resources
                .offset((*view).sub_resource_idx as isize))
            .clear_value
            .__bindgen_anon_1.stencil = stencil;
            wined3d_rendertarget_view_validate_location(view, WINED3D_LOCATION_CLEARED as uint32_t);
            wined3d_rendertarget_view_invalidate_location(view, !WINED3D_LOCATION_CLEARED as uint32_t);
            flags &= !((WINED3DCLEAR_ZBUFFER | WINED3DCLEAR_STENCIL) as uint32_t);
            delay_count = delay_count.wrapping_add(1);
        }
    }

    if attachment_count == 0 {
        return;
    }

    if delay_count != 0 {
        WINE_TRACE_!(__wine_dbch_d3d_perf, "Partial clear: %u immediate, %u delayed.\n", attachment_count, delay_count);
    }

    vk_render_pass = wined3d_context_vk_get_render_pass(
        context_vk,
        core::ptr::addr_of_mut!(immediate_fb),
        immediate_rt_count,
        (flags & (WINED3DCLEAR_ZBUFFER | WINED3DCLEAR_STENCIL) as uint32_t) != 0,
        flags,
    );
    if vk_render_pass == 0 {
        ERR!("Failed to get render pass.\n");
        return;
    }

    vk_command_buffer = wined3d_context_vk_get_command_buffer(context_vk);
    if vk_command_buffer.is_null() {
        ERR!("Failed to get command buffer.\n");
        return;
    }

    fb_desc.sType = VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO as _;
    fb_desc.pNext = core::ptr::null();
    fb_desc.flags = 0 as _;
    fb_desc.renderPass = vk_render_pass;
    fb_desc.attachmentCount = attachment_count as uint32_t;
    fb_desc.pAttachments = core::ptr::addr_of_mut!(views) as *mut VkImageView;
    fb_desc.width = ((*draw_rect).right - (*draw_rect).left) as uint32_t;
    fb_desc.height = ((*draw_rect).bottom - (*draw_rect).top) as uint32_t;
    fb_desc.layers = layer_count as uint32_t;
    vr = ((*vk_info).vk_ops.vkCreateFramebuffer.unwrap_unchecked())(
        (*device_vk).vk_device,
        core::ptr::addr_of_mut!(fb_desc),
        core::ptr::null(),
        core::ptr::addr_of_mut!(vk_framebuffer),
    );
    if (vr as ::core::ffi::c_int) < 0 {
        ERR!("Failed to create Vulkan framebuffer, vr %s.\n", wined3d_debug_vkresult(vr));
        return;
    }

    begin_desc.sType = VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO as _;
    begin_desc.pNext = core::ptr::null();
    begin_desc.renderPass = vk_render_pass;
    begin_desc.framebuffer = vk_framebuffer;
    begin_desc.clearValueCount = attachment_count as uint32_t;
    begin_desc.pClearValues = core::ptr::addr_of_mut!(clear_values) as *mut VkClearValue;

    wined3d_context_vk_end_current_render_pass(context_vk);

    i = 0;
    while i < rect_count {
        r.left = max!((*clear_rects.offset(i as isize)).left, (*draw_rect).left);
        r.top = max!((*clear_rects.offset(i as isize)).top, (*draw_rect).top);
        r.right = min!((*clear_rects.offset(i as isize)).right, (*draw_rect).right);
        r.bottom = min!((*clear_rects.offset(i as isize)).bottom, (*draw_rect).bottom);

        if !(r.left >= r.right || r.top >= r.bottom) {
            begin_desc.renderArea.offset.x = r.left as int32_t;
            begin_desc.renderArea.offset.y = r.top as int32_t;
            begin_desc.renderArea.extent.width = (r.right - r.left) as uint32_t;
            begin_desc.renderArea.extent.height = (r.bottom - r.top) as uint32_t;
            ((*vk_info).vk_ops.vkCmdBeginRenderPass.unwrap_unchecked())(
                vk_command_buffer,
                core::ptr::addr_of_mut!(begin_desc),
                VK_SUBPASS_CONTENTS_INLINE as _,
            );
            ((*vk_info).vk_ops.vkCmdEndRenderPass.unwrap_unchecked())(vk_command_buffer);
        }
        i = i.wrapping_add(1);
    }

    wined3d_context_vk_destroy_vk_framebuffer(
        context_vk,
        vk_framebuffer,
        (*context_vk).current_command_buffer.id,
    );

    i = 0;
    while i < immediate_rt_count {
        wined3d_context_vk_reference_rendertarget_view(
            context_vk,
            wined3d_rendertarget_view_vk(immediate_fb.render_targets[i as usize]),
        );
        i = i.wrapping_add(1);
    }

    if depth_stencil {
        wined3d_context_vk_reference_rendertarget_view(
            context_vk,
            wined3d_rendertarget_view_vk(immediate_fb.depth_stencil),
        );
    }
}

unsafe extern "C" fn vk_blitter_clear(
    blitter: *mut wined3d_blitter,
    device: *mut wined3d_device,
    mut rt_count: u32,
    fb: *const wined3d_fb_state,
    mut rect_count: u32,
    mut clear_rects: *const RECT,
    draw_rect: *const RECT,
    mut flags: u32,
    colour: *const wined3d_color,
    depth: f32,
    stencil: u32,
) {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk(device);
    let mut view: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut previous: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut context_vk: *mut wined3d_context_vk = core::ptr::null_mut();
    let mut have_identical_size: bool = true;
    let mut tmp_fb: wined3d_fb_state = core::mem::zeroed();
    let mut next_rt_count: u32 = 0;
    let mut next: *mut wined3d_blitter = core::ptr::null_mut();
    let mut next_flags: u32 = 0;
    let mut i: u32;

    if rect_count == 0 {
        rect_count = 1;
        clear_rects = draw_rect;
    }

    if flags & WINED3DCLEAR_TARGET != 0 {
        i = 0;
        while i < rt_count {
            view = (*fb).render_targets[i as usize];
            if view.is_null() {
                i += 1;
                continue;
            }

            if blitter_use_cpu_clear(view) {
                next_flags |= WINED3DCLEAR_TARGET;
                flags &= !WINED3DCLEAR_TARGET;
                next_rt_count = rt_count;
                rt_count = 0;
                break;
            }
            i += 1;
        }
    }

    if flags & (WINED3DCLEAR_ZBUFFER | WINED3DCLEAR_STENCIL) != 0
        && {
            view = (*fb).depth_stencil;
            !view.is_null()
        }
        && ((*(*view).format).depth_size == 0 || flags & WINED3DCLEAR_ZBUFFER != 0)
        && ((*(*view).format).stencil_size == 0 || flags & WINED3DCLEAR_STENCIL != 0)
        && blitter_use_cpu_clear(view)
    {
        next_flags |= flags & (WINED3DCLEAR_ZBUFFER | WINED3DCLEAR_STENCIL);
        flags &= !(WINED3DCLEAR_ZBUFFER | WINED3DCLEAR_STENCIL);
    }

    if flags != 0 {
        context_vk = wined3d_context_vk(context_acquire(
            core::ptr::addr_of_mut!((*device_vk).d),
            core::ptr::null_mut(),
            0,
        ));

        i = 0;
        while i < rt_count {
            view = (*fb).render_targets[i as usize];
            if view.is_null() {
                i += 1;
                continue;
            }
            if !previous.is_null()
                && ((*previous).width != (*view).width || (*previous).height != (*view).height)
            {
                have_identical_size = false;
            }
            previous = view;
            i += 1;
        }
        if flags & (WINED3DCLEAR_ZBUFFER | WINED3DCLEAR_STENCIL) != 0 {
            view = (*fb).depth_stencil;

            if !previous.is_null()
                && ((*previous).width != (*view).width || (*previous).height != (*view).height)
            {
                have_identical_size = false;
            }
        }

        if have_identical_size {
            vk_blitter_clear_rendertargets(
                context_vk,
                rt_count,
                fb,
                rect_count,
                clear_rects,
                draw_rect,
                flags,
                colour,
                depth,
                stencil,
            );
        } else {
            i = 0;
            while i < rt_count {
                view = (*fb).render_targets[i as usize];
                if view.is_null() {
                    i += 1;
                    continue;
                }

                tmp_fb.render_targets[0 as usize] = view;
                tmp_fb.depth_stencil = core::ptr::null_mut();
                vk_blitter_clear_rendertargets(
                    context_vk,
                    1,
                    core::ptr::addr_of_mut!(tmp_fb),
                    rect_count,
                    clear_rects,
                    draw_rect,
                    WINED3DCLEAR_TARGET,
                    colour,
                    depth,
                    stencil,
                );
                i += 1;
            }
            if flags & (WINED3DCLEAR_ZBUFFER | WINED3DCLEAR_STENCIL) != 0 {
                tmp_fb.render_targets[0 as usize] = core::ptr::null_mut();
                tmp_fb.depth_stencil = (*fb).depth_stencil;
                vk_blitter_clear_rendertargets(
                    context_vk,
                    0,
                    core::ptr::addr_of_mut!(tmp_fb),
                    rect_count,
                    clear_rects,
                    draw_rect,
                    flags & !WINED3DCLEAR_TARGET,
                    colour,
                    depth,
                    stencil,
                );
            }
        }

        context_release(core::ptr::addr_of_mut!((*context_vk).c));
    }

    if next_flags == 0 {
        return;
    }

    next = (*blitter).next;
    if next.is_null() {
        ERR!("No blitter to handle clear.\n");
        return;
    }

    (*(*next).ops)
        .blitter_clear
        .unwrap_unchecked()(
        next,
        device,
        next_rt_count,
        fb,
        rect_count,
        clear_rects,
        draw_rect,
        next_flags,
        colour,
        depth,
        stencil,
    );
}

unsafe extern "C" fn vk_blitter_blit_supported(op: wined3d_blit_op, context: *const wined3d_context, src_resource: *const wined3d_resource, src_rect: *const RECT, dst_resource: *const wined3d_resource, dst_rect: *const RECT, resolve_format: *const wined3d_format) -> bool {
    let _ = context;
    let mut src_format: *const wined3d_format = (*src_resource).format;
    let mut dst_format: *const wined3d_format = (*dst_resource).format;

    if !((*dst_resource).access & WINED3D_RESOURCE_ACCESS_GPU != 0) {
        return false;
    }

    if !((*src_resource).access & WINED3D_RESOURCE_ACCESS_GPU != 0) {
        return false;
    }

    if (*dst_format).id != (*src_format).id {
        if ((is_identity_fixup((*dst_format).color_fixup)) == 0) {
            return false;
        }

        if ((is_identity_fixup((*src_format).color_fixup)) == 0) {
            return false;
        }

        if op != WINED3D_BLIT_OP_RAW_BLIT
            && (*wined3d_format_vk(src_format)).vk_format != (*wined3d_format_vk(dst_format)).vk_format
            && ((((wined3d_format_is_typeless(src_format)) == 0) && ((wined3d_format_is_typeless(dst_format)) == 0))
                || resolve_format.is_null())
        {
            return false;
        }
    }

    if wined3d_resource_get_sample_count(dst_resource) > 1 {
        return false;
    }

    if op == WINED3D_BLIT_OP_RAW_BLIT {
        return true;
    }

    if op != WINED3D_BLIT_OP_COLOR_BLIT {
        return false;
    }

    if ((*src_rect).right - (*src_rect).left != (*dst_rect).right - (*dst_rect).left)
        || ((*src_rect).bottom - (*src_rect).top != (*dst_rect).bottom - (*dst_rect).top)
    {
        return false;
    }

    return true;
}

unsafe extern "C" fn vk_blitter_blit(
    mut blitter: *mut wined3d_blitter,
    mut op: wined3d_blit_op,
    mut context: *mut wined3d_context,
    mut src_texture: *mut wined3d_texture,
    mut src_sub_resource_idx: ::core::ffi::c_uint,
    mut src_location: DWORD,
    mut src_rect: *const RECT,
    mut dst_texture: *mut wined3d_texture,
    mut dst_sub_resource_idx: ::core::ffi::c_uint,
    mut dst_location: DWORD,
    mut dst_rect: *const RECT,
    mut colour_key: *const wined3d_color_key,
    mut filter: wined3d_texture_filter_type,
    mut resolve_format: *const wined3d_format,
) -> DWORD {
    let mut src_texture_vk: *mut wined3d_texture_vk = wined3d_texture_vk(src_texture);
    let mut dst_texture_vk: *mut wined3d_texture_vk = wined3d_texture_vk(dst_texture);
    let mut context_vk: *mut wined3d_context_vk = wined3d_context_vk(context);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut vk_src_range: VkImageSubresourceRange = core::mem::zeroed();
    let mut vk_dst_range: VkImageSubresourceRange = core::mem::zeroed();
    let mut src_layout: VkImageLayout = core::mem::zeroed();
    let mut dst_layout: VkImageLayout = core::mem::zeroed();
    let mut vk_command_buffer: VkCommandBuffer = core::ptr::null_mut();
    let mut next: *mut wined3d_blitter = core::ptr::null_mut();
    let mut src_sample_count: ::core::ffi::c_uint = 0;
    let mut resolve: bool = false;

    TRACE!(
        "blitter %p, op %#x, context %p, src_texture %p, src_sub_resource_idx %u, src_location %s, src_rect %s, dst_texture %p, dst_sub_resource_idx %u, dst_location %s, dst_rect %s, colour_key %p, filter %s, resolve format %p.\n",
        blitter,
        op as ::core::ffi::c_uint,
        context,
        src_texture,
        src_sub_resource_idx,
        wined3d_debug_location(src_location as uint32_t),
        wine_dbgstr_rect(src_rect),
        dst_texture,
        dst_sub_resource_idx,
        wined3d_debug_location(dst_location as uint32_t),
        wine_dbgstr_rect(dst_rect),
        colour_key,
        debug_d3dtexturefiltertype(filter),
        resolve_format,
    );

    'next_block: {
        'barrier_next: {
            if !vk_blitter_blit_supported(
                op,
                context,
                core::ptr::addr_of_mut!((*src_texture).resource),
                src_rect,
                core::ptr::addr_of_mut!((*dst_texture).resource),
                dst_rect,
                resolve_format,
            ) {
                break 'next_block;
            }

            src_sample_count = wined3d_resource_get_sample_count(core::ptr::addr_of_mut!((*src_texture_vk).t.resource));
            if src_sample_count > 1 {
                resolve = true;
            }

            vk_src_range.aspectMask = vk_aspect_mask_from_format((*src_texture_vk).t.resource.format);
            vk_src_range.baseMipLevel = src_sub_resource_idx.wrapping_rem((*src_texture).level_count) as uint32_t;
            vk_src_range.levelCount = 1;
            vk_src_range.baseArrayLayer = src_sub_resource_idx.wrapping_div((*src_texture).level_count) as uint32_t;
            vk_src_range.layerCount = 1;

            vk_dst_range.aspectMask = vk_aspect_mask_from_format((*dst_texture_vk).t.resource.format);
            vk_dst_range.baseMipLevel = dst_sub_resource_idx.wrapping_rem((*dst_texture).level_count) as uint32_t;
            vk_dst_range.levelCount = 1;
            vk_dst_range.baseArrayLayer = dst_sub_resource_idx.wrapping_div((*dst_texture).level_count) as uint32_t;
            vk_dst_range.layerCount = 1;

            if wined3d_texture_load_location(src_texture, src_sub_resource_idx, context, WINED3D_LOCATION_TEXTURE_RGB) == 0 {
                ERR!("Failed to load the source sub-resource.\n");
            }

            if wined3d_texture_is_full_rect(dst_texture, vk_dst_range.baseMipLevel, dst_rect) {
                if wined3d_texture_prepare_location(dst_texture, dst_sub_resource_idx, context, WINED3D_LOCATION_TEXTURE_RGB) == 0 {
                    ERR!("Failed to prepare the destination sub-resource.\n");
                    break 'next_block;
                }
            } else {
                if wined3d_texture_load_location(dst_texture, dst_sub_resource_idx, context, WINED3D_LOCATION_TEXTURE_RGB) == 0 {
                    ERR!("Failed to load the destination sub-resource.\n");
                    break 'next_block;
                }
            }

            vk_command_buffer = wined3d_context_vk_get_command_buffer(context_vk);
            if vk_command_buffer.is_null() {
                ERR!("Failed to get command buffer.\n");
                break 'next_block;
            }

            if (*src_texture_vk).layout == VK_IMAGE_LAYOUT_GENERAL {
                src_layout = VK_IMAGE_LAYOUT_GENERAL;
            } else {
                src_layout = VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL;
            }

            if (*dst_texture_vk).layout == VK_IMAGE_LAYOUT_GENERAL {
                dst_layout = VK_IMAGE_LAYOUT_GENERAL;
            } else {
                dst_layout = VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL;
            }

            wined3d_context_vk_image_barrier(
                context_vk,
                vk_command_buffer,
                VK_PIPELINE_STAGE_ALL_COMMANDS_BIT,
                VK_PIPELINE_STAGE_TRANSFER_BIT,
                vk_access_mask_from_bind_flags((*src_texture_vk).t.resource.bind_flags),
                VK_ACCESS_TRANSFER_READ_BIT,
                (*src_texture_vk).layout,
                src_layout,
                (*src_texture_vk).image.vk_image,
                core::ptr::addr_of_mut!(vk_src_range),
            );
            wined3d_context_vk_image_barrier(
                context_vk,
                vk_command_buffer,
                VK_PIPELINE_STAGE_ALL_COMMANDS_BIT,
                VK_PIPELINE_STAGE_TRANSFER_BIT,
                vk_access_mask_from_bind_flags((*dst_texture_vk).t.resource.bind_flags),
                VK_ACCESS_TRANSFER_WRITE_BIT,
                (*dst_texture_vk).layout,
                dst_layout,
                (*dst_texture_vk).image.vk_image,
                core::ptr::addr_of_mut!(vk_dst_range),
            );

            if resolve {
                let src_format_vk: *const wined3d_format_vk = wined3d_format_vk((*src_texture).resource.format);
                let dst_format_vk: *const wined3d_format_vk = wined3d_format_vk((*dst_texture).resource.format);
                let usage: ::core::ffi::c_uint = VK_IMAGE_USAGE_TRANSFER_SRC_BIT
                    | VK_IMAGE_USAGE_TRANSFER_DST_BIT
                    | VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT;
                let mut resolve_src_layout: VkImageLayout = core::mem::zeroed();
                let mut resolve_dst_layout: VkImageLayout = core::mem::zeroed();
                let mut src_vk_image: VkImage = core::mem::zeroed();
                let mut dst_vk_image: VkImage = core::mem::zeroed();
                let mut vk_range: VkImageSubresourceRange = core::mem::zeroed();
                let mut resolve_region: VkImageResolve = core::mem::zeroed();
                let mut vk_image_type: VkImageType = core::mem::zeroed();
                let mut copy_region: VkImageCopy = core::mem::zeroed();
                let mut vk_format: VkFormat = core::mem::zeroed();

                if !resolve_format.is_null() {
                    vk_format = (*wined3d_format_vk(resolve_format)).vk_format;
                } else if wined3d_format_is_typeless((*src_texture).resource.format) == 0 {
                    vk_format = (*src_format_vk).vk_format;
                } else {
                    vk_format = (*dst_format_vk).vk_format;
                }

                match (*src_texture).resource.type_ {
                    WINED3D_RTYPE_TEXTURE_1D => {
                        vk_image_type = VK_IMAGE_TYPE_1D;
                    }
                    WINED3D_RTYPE_TEXTURE_2D => {
                        vk_image_type = VK_IMAGE_TYPE_2D;
                    }
                    WINED3D_RTYPE_TEXTURE_3D => {
                        vk_image_type = VK_IMAGE_TYPE_3D;
                    }
                    _ => {
                        ERR!("Unexpected resource type: %s\n", debug_d3dresourcetype((*src_texture).resource.type_));
                        break 'barrier_next;
                    }
                }

                vk_range.baseMipLevel = 0;
                vk_range.levelCount = 1;
                vk_range.baseArrayLayer = 0;
                vk_range.layerCount = 1;

                resolve_region.srcSubresource.aspectMask = vk_src_range.aspectMask;
                resolve_region.dstSubresource.aspectMask = vk_dst_range.aspectMask;
                resolve_region.extent.width = ((*src_rect).right - (*src_rect).left) as uint32_t;
                resolve_region.extent.height = ((*src_rect).bottom - (*src_rect).top) as uint32_t;
                resolve_region.extent.depth = 1;

                if (*src_format_vk).vk_format != vk_format {
                    let mut src_image: wined3d_image_vk = core::mem::zeroed();

                    if wined3d_context_vk_create_image(
                        context_vk,
                        vk_image_type,
                        usage,
                        vk_format,
                        resolve_region.extent.width,
                        resolve_region.extent.height,
                        1,
                        src_sample_count,
                        1,
                        1,
                        0,
                        core::ptr::addr_of_mut!(src_image),
                    ) == 0
                    {
                        break 'barrier_next;
                    }

                    wined3d_context_vk_reference_image(context_vk, core::ptr::addr_of_mut!(src_image));
                    src_vk_image = src_image.vk_image;
                    wined3d_context_vk_destroy_image(context_vk, core::ptr::addr_of_mut!(src_image));

                    vk_range.aspectMask = vk_src_range.aspectMask;

                    wined3d_context_vk_image_barrier(
                        context_vk,
                        vk_command_buffer,
                        VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT,
                        VK_PIPELINE_STAGE_TRANSFER_BIT,
                        0,
                        VK_ACCESS_TRANSFER_WRITE_BIT,
                        VK_IMAGE_LAYOUT_UNDEFINED,
                        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
                        src_vk_image,
                        core::ptr::addr_of_mut!(vk_range),
                    );

                    copy_region.srcSubresource.aspectMask = vk_src_range.aspectMask;
                    copy_region.srcSubresource.mipLevel = vk_src_range.baseMipLevel;
                    copy_region.srcSubresource.baseArrayLayer = vk_src_range.baseArrayLayer;
                    copy_region.srcSubresource.layerCount = 1;
                    copy_region.srcOffset.x = (*src_rect).left as int32_t;
                    copy_region.srcOffset.y = (*src_rect).top as int32_t;
                    copy_region.srcOffset.z = 0;
                    copy_region.dstSubresource.aspectMask = vk_src_range.aspectMask;
                    copy_region.dstSubresource.mipLevel = 0;
                    copy_region.dstSubresource.baseArrayLayer = 0;
                    copy_region.dstSubresource.layerCount = 1;
                    copy_region.dstOffset.x = 0;
                    copy_region.dstOffset.y = 0;
                    copy_region.dstOffset.z = 0;
                    copy_region.extent.width = resolve_region.extent.width;
                    copy_region.extent.height = resolve_region.extent.height;
                    copy_region.extent.depth = 1;

                    ((*vk_info).vk_ops.vkCmdCopyImage.unwrap_unchecked())(
                        vk_command_buffer,
                        (*src_texture_vk).image.vk_image,
                        src_layout,
                        src_vk_image,
                        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
                        1,
                        core::ptr::addr_of_mut!(copy_region),
                    );

                    wined3d_context_vk_image_barrier(
                        context_vk,
                        vk_command_buffer,
                        VK_PIPELINE_STAGE_TRANSFER_BIT,
                        VK_PIPELINE_STAGE_TRANSFER_BIT,
                        VK_ACCESS_TRANSFER_WRITE_BIT,
                        VK_ACCESS_TRANSFER_READ_BIT,
                        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
                        VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL,
                        src_vk_image,
                        core::ptr::addr_of_mut!(vk_range),
                    );
                    resolve_src_layout = VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL;

                    resolve_region.srcSubresource.mipLevel = 0;
                    resolve_region.srcSubresource.baseArrayLayer = 0;
                    resolve_region.srcSubresource.layerCount = 1;
                    resolve_region.srcOffset.x = 0;
                    resolve_region.srcOffset.y = 0;
                    resolve_region.srcOffset.z = 0;
                } else {
                    src_vk_image = (*src_texture_vk).image.vk_image;
                    resolve_src_layout = src_layout;

                    resolve_region.srcSubresource.mipLevel = vk_src_range.baseMipLevel;
                    resolve_region.srcSubresource.baseArrayLayer = vk_src_range.baseArrayLayer;
                    resolve_region.srcSubresource.layerCount = 1;
                    resolve_region.srcOffset.x = (*src_rect).left as int32_t;
                    resolve_region.srcOffset.y = (*src_rect).top as int32_t;
                    resolve_region.srcOffset.z = 0;
                }

                if (*dst_format_vk).vk_format != vk_format {
                    let mut dst_image: wined3d_image_vk = core::mem::zeroed();

                    if wined3d_context_vk_create_image(
                        context_vk,
                        vk_image_type,
                        usage,
                        vk_format,
                        resolve_region.extent.width,
                        resolve_region.extent.height,
                        1,
                        VK_SAMPLE_COUNT_1_BIT,
                        1,
                        1,
                        0,
                        core::ptr::addr_of_mut!(dst_image),
                    ) == 0
                    {
                        break 'barrier_next;
                    }

                    wined3d_context_vk_reference_image(context_vk, core::ptr::addr_of_mut!(dst_image));
                    dst_vk_image = dst_image.vk_image;
                    wined3d_context_vk_destroy_image(context_vk, core::ptr::addr_of_mut!(dst_image));

                    vk_range.aspectMask = vk_dst_range.aspectMask;
                    wined3d_context_vk_image_barrier(
                        context_vk,
                        vk_command_buffer,
                        VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT,
                        VK_PIPELINE_STAGE_TRANSFER_BIT,
                        0,
                        VK_ACCESS_TRANSFER_WRITE_BIT,
                        VK_IMAGE_LAYOUT_UNDEFINED,
                        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
                        dst_vk_image,
                        core::ptr::addr_of_mut!(vk_range),
                    );
                    resolve_dst_layout = VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL;

                    resolve_region.dstSubresource.mipLevel = 0;
                    resolve_region.dstSubresource.baseArrayLayer = 0;
                    resolve_region.dstSubresource.layerCount = 1;
                    resolve_region.dstOffset.x = 0;
                    resolve_region.dstOffset.y = 0;
                    resolve_region.dstOffset.z = 0;
                } else {
                    dst_vk_image = (*dst_texture_vk).image.vk_image;
                    resolve_dst_layout = dst_layout;

                    resolve_region.dstSubresource.mipLevel = vk_dst_range.baseMipLevel;
                    resolve_region.dstSubresource.baseArrayLayer = vk_dst_range.baseArrayLayer;
                    resolve_region.dstSubresource.layerCount = 1;
                    resolve_region.dstOffset.x = (*dst_rect).left as int32_t;
                    resolve_region.dstOffset.y = (*dst_rect).top as int32_t;
                    resolve_region.dstOffset.z = 0;
                }

                ((*vk_info).vk_ops.vkCmdResolveImage.unwrap_unchecked())(
                    vk_command_buffer,
                    src_vk_image,
                    resolve_src_layout,
                    dst_vk_image,
                    resolve_dst_layout,
                    1,
                    core::ptr::addr_of_mut!(resolve_region),
                );

                if dst_vk_image != (*dst_texture_vk).image.vk_image {
                    wined3d_context_vk_image_barrier(
                        context_vk,
                        vk_command_buffer,
                        VK_PIPELINE_STAGE_TRANSFER_BIT,
                        VK_PIPELINE_STAGE_TRANSFER_BIT,
                        VK_ACCESS_TRANSFER_WRITE_BIT,
                        VK_ACCESS_TRANSFER_READ_BIT,
                        resolve_dst_layout,
                        VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL,
                        dst_vk_image,
                        core::ptr::addr_of_mut!(vk_range),
                    );

                    copy_region.srcSubresource.aspectMask = vk_dst_range.aspectMask;
                    copy_region.srcSubresource.mipLevel = 0;
                    copy_region.srcSubresource.baseArrayLayer = 0;
                    copy_region.srcSubresource.layerCount = 1;
                    copy_region.srcOffset.x = 0;
                    copy_region.srcOffset.y = 0;
                    copy_region.srcOffset.z = 0;
                    copy_region.dstSubresource.aspectMask = vk_dst_range.aspectMask;
                    copy_region.dstSubresource.mipLevel = vk_dst_range.baseMipLevel;
                    copy_region.dstSubresource.baseArrayLayer = vk_dst_range.baseArrayLayer;
                    copy_region.dstSubresource.layerCount = 1;
                    copy_region.dstOffset.x = (*dst_rect).left as int32_t;
                    copy_region.dstOffset.y = (*dst_rect).top as int32_t;
                    copy_region.dstOffset.z = 0;
                    copy_region.extent.width = resolve_region.extent.width;
                    copy_region.extent.height = resolve_region.extent.height;
                    copy_region.extent.depth = 1;

                    ((*vk_info).vk_ops.vkCmdCopyImage.unwrap_unchecked())(
                        vk_command_buffer,
                        dst_vk_image,
                        VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL,
                        (*dst_texture_vk).image.vk_image,
                        dst_layout,
                        1,
                        core::ptr::addr_of_mut!(copy_region),
                    );
                }
            } else {
                let mut region: VkImageCopy = core::mem::zeroed();

                region.srcSubresource.aspectMask = vk_src_range.aspectMask;
                region.srcSubresource.mipLevel = vk_src_range.baseMipLevel;
                region.srcSubresource.baseArrayLayer = vk_src_range.baseArrayLayer;
                region.srcSubresource.layerCount = vk_src_range.layerCount;
                region.srcOffset.x = (*src_rect).left as int32_t;
                region.srcOffset.y = (*src_rect).top as int32_t;
                region.srcOffset.z = 0;
                region.dstSubresource.aspectMask = vk_dst_range.aspectMask;
                region.dstSubresource.mipLevel = vk_dst_range.baseMipLevel;
                region.dstSubresource.baseArrayLayer = vk_dst_range.baseArrayLayer;
                region.dstSubresource.layerCount = vk_dst_range.layerCount;
                region.dstOffset.x = (*dst_rect).left as int32_t;
                region.dstOffset.y = (*dst_rect).top as int32_t;
                region.dstOffset.z = 0;
                region.extent.width = ((*src_rect).right - (*src_rect).left) as uint32_t;
                region.extent.height = ((*src_rect).bottom - (*src_rect).top) as uint32_t;
                region.extent.depth = 1;

                ((*vk_info).vk_ops.vkCmdCopyImage.unwrap_unchecked())(
                    vk_command_buffer,
                    (*src_texture_vk).image.vk_image,
                    src_layout,
                    (*dst_texture_vk).image.vk_image,
                    dst_layout,
                    1,
                    core::ptr::addr_of_mut!(region),
                );
            }

            wined3d_context_vk_image_barrier(
                context_vk,
                vk_command_buffer,
                VK_PIPELINE_STAGE_TRANSFER_BIT,
                VK_PIPELINE_STAGE_ALL_COMMANDS_BIT,
                VK_ACCESS_TRANSFER_WRITE_BIT,
                vk_access_mask_from_bind_flags((*dst_texture_vk).t.resource.bind_flags),
                dst_layout,
                (*dst_texture_vk).layout,
                (*dst_texture_vk).image.vk_image,
                core::ptr::addr_of_mut!(vk_dst_range),
            );
            wined3d_context_vk_image_barrier(
                context_vk,
                vk_command_buffer,
                VK_PIPELINE_STAGE_TRANSFER_BIT,
                VK_PIPELINE_STAGE_ALL_COMMANDS_BIT,
                VK_ACCESS_TRANSFER_READ_BIT,
                vk_access_mask_from_bind_flags((*src_texture_vk).t.resource.bind_flags),
                src_layout,
                (*src_texture_vk).layout,
                (*src_texture_vk).image.vk_image,
                core::ptr::addr_of_mut!(vk_src_range),
            );

            wined3d_texture_validate_location(dst_texture, dst_sub_resource_idx, WINED3D_LOCATION_TEXTURE_RGB);
            wined3d_texture_invalidate_location(dst_texture, dst_sub_resource_idx, !WINED3D_LOCATION_TEXTURE_RGB);
            if wined3d_texture_load_location(dst_texture, dst_sub_resource_idx, context, dst_location) == 0 {
                ERR!("Failed to load the destination sub-resource into %s.\n", wined3d_debug_location(dst_location));
            }

            wined3d_context_vk_reference_texture(context_vk, src_texture_vk);
            wined3d_context_vk_reference_texture(context_vk, dst_texture_vk);

            return dst_location | WINED3D_LOCATION_TEXTURE_RGB;
        }

        // barrier_next:
        wined3d_context_vk_image_barrier(
            context_vk,
            vk_command_buffer,
            VK_PIPELINE_STAGE_TRANSFER_BIT,
            VK_PIPELINE_STAGE_ALL_COMMANDS_BIT,
            VK_ACCESS_TRANSFER_WRITE_BIT,
            vk_access_mask_from_bind_flags((*dst_texture_vk).t.resource.bind_flags),
            dst_layout,
            (*dst_texture_vk).layout,
            (*dst_texture_vk).image.vk_image,
            core::ptr::addr_of_mut!(vk_dst_range),
        );
        wined3d_context_vk_image_barrier(
            context_vk,
            vk_command_buffer,
            VK_PIPELINE_STAGE_TRANSFER_BIT,
            VK_PIPELINE_STAGE_ALL_COMMANDS_BIT,
            VK_ACCESS_TRANSFER_READ_BIT,
            vk_access_mask_from_bind_flags((*src_texture_vk).t.resource.bind_flags),
            src_layout,
            (*src_texture_vk).layout,
            (*src_texture_vk).image.vk_image,
            core::ptr::addr_of_mut!(vk_src_range),
        );
    }

    // next:
    next = (*blitter).next;
    if next.is_null() {
        ERR!("No blitter to handle blit op %#x.\n", op as ::core::ffi::c_uint);
        return dst_location;
    }

    TRACE!("Forwarding to blitter %p.\n", next);

    return (*(*next).ops).blitter_blit.unwrap_unchecked()(
        next,
        op,
        context,
        src_texture,
        src_sub_resource_idx,
        src_location,
        src_rect,
        dst_texture,
        dst_sub_resource_idx,
        dst_location,
        dst_rect,
        colour_key,
        filter,
        resolve_format,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_vk_blitter_create(next: *mut *mut wined3d_blitter) {
    let mut blitter: *mut wined3d_blitter = core::ptr::null_mut();

    blitter = malloc(core::mem::size_of::<wined3d_blitter>()) as *mut wined3d_blitter;
    if blitter.is_null() {
        return;
    }

    WINE_TRACE!("Created blitter %p.\n", blitter);

    (*blitter).ops = core::ptr::addr_of!(vk_blitter_ops) as *const wined3d_blitter_ops;
    (*blitter).next = *next;
    *next = blitter;
}
