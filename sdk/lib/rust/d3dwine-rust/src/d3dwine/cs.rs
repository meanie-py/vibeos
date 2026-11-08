use crate::d3dwine::buffer::wined3d_buffer_copy;
use crate::d3dwine::buffer::wined3d_buffer_create;
use crate::d3dwine::buffer::wined3d_buffer_decref;
use crate::d3dwine::buffer::wined3d_buffer_incref;
use crate::d3dwine::buffer::wined3d_buffer_update_sub_resource;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_80073828D8A0: [u16; 10] = [0x006Eu16, 0x0074, 0x0064, 0x006C, 0x006C, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000];
pub static RUSTCONV_WSTR_EC4088632B18: [u16; 11] = [
    0x0077u16, 0x0069, 0x006E, 0x0065, 0x0064, 0x0033, 0x0064, 0x005F, 0x0063, 0x0073,
    0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut pNtAlertThreadByThreadId: Option<unsafe extern "C" fn(HANDLE) -> NTSTATUS> = None;

static mut pNtWaitForAlertByThreadId: Option<
    unsafe extern "C" fn(*mut ::core::ffi::c_void, *const LARGE_INTEGER) -> NTSTATUS,
> = None;

static mut wined3d_cs_push_constant_info: [push_constant_info; 8] = [
    push_constant_info {
        size: ::core::mem::size_of::<wined3d_vec4>() as size_t,
        max_count: WINED3D_MAX_VS_CONSTS_F as ::core::ffi::c_uint,
        shader_type: WINED3D_SHADER_TYPE_VERTEX,
    },
    push_constant_info {
        size: ::core::mem::size_of::<wined3d_vec4>() as size_t,
        max_count: WINED3D_MAX_PS_CONSTS_F as ::core::ffi::c_uint,
        shader_type: WINED3D_SHADER_TYPE_PIXEL,
    },
    push_constant_info {
        size: ::core::mem::size_of::<wined3d_ivec4>() as size_t,
        max_count: WINED3D_MAX_CONSTS_I as ::core::ffi::c_uint,
        shader_type: WINED3D_SHADER_TYPE_VERTEX,
    },
    push_constant_info {
        size: ::core::mem::size_of::<wined3d_ivec4>() as size_t,
        max_count: WINED3D_MAX_CONSTS_I as ::core::ffi::c_uint,
        shader_type: WINED3D_SHADER_TYPE_PIXEL,
    },
    push_constant_info {
        size: ::core::mem::size_of::<BOOL>() as size_t,
        max_count: WINED3D_MAX_CONSTS_B as ::core::ffi::c_uint,
        shader_type: WINED3D_SHADER_TYPE_VERTEX,
    },
    push_constant_info {
        size: ::core::mem::size_of::<BOOL>() as size_t,
        max_count: WINED3D_MAX_CONSTS_B as ::core::ffi::c_uint,
        shader_type: WINED3D_SHADER_TYPE_PIXEL,
    },
    push_constant_info {
        size: 1 as size_t,
        max_count: ::core::mem::size_of::<wined3d_ffp_vs_constants>() as ::core::ffi::c_uint,
        shader_type: WINED3D_SHADER_TYPE_VERTEX,
    },
    push_constant_info {
        size: 1 as size_t,
        max_count: ::core::mem::size_of::<wined3d_ffp_ps_constants>() as ::core::ffi::c_uint,
        shader_type: WINED3D_SHADER_TYPE_PIXEL,
    },
];

static mut wined3d_cs_op_handlers: [Option<
    unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
>; 50] = unsafe {
    [
        Some(
            wined3d_cs_exec_nop
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_present
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_clear
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_dispatch
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_draw
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_flush
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_predication
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_viewports
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_scissor_rects
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_rendertarget_views
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_depth_stencil_view
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_vertex_declaration
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_stream_sources
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_stream_outputs
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_index_buffer
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_constant_buffers
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_texture
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_shader_resource_views
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_unordered_access_views
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_samplers
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_shader
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_blend_state
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_depth_stencil_state
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_rasterizer_state
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_depth_bounds
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_render_state
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_texture_state
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_transform
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_clip_plane
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_color_key
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_light
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_light_enable
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_set_feature_level
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_push_constants
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_reset_state
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_callback
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_query_issue
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_preload_resource
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_unload_resource
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_map
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_unmap
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_map_bo_address
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_blt_sub_resource
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_update_sub_resource
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_add_dirty_texture_region
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_clear_sysmem_texture
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_clear_unordered_access_view
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_copy_uav_counter
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_generate_mipmaps
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
        Some(
            wined3d_cs_exec_execute_command_list
                as unsafe extern "C" fn(*mut wined3d_cs, *const ::core::ffi::c_void) -> (),
        ),
    ]
};

static mut wined3d_cs_st_ops: wined3d_device_context_ops = unsafe {
    wined3d_device_context_ops {
        require_space: Some(
            wined3d_cs_st_require_space
                as unsafe extern "C" fn(
                    *mut wined3d_device_context,
                    size_t,
                    wined3d_cs_queue_id,
                ) -> *mut ::core::ffi::c_void,
        ),
        submit: Some(
            wined3d_cs_st_submit
                as unsafe extern "C" fn(*mut wined3d_device_context, wined3d_cs_queue_id) -> (),
        ),
        finish: Some(
            wined3d_cs_st_finish
                as unsafe extern "C" fn(*mut wined3d_device_context, wined3d_cs_queue_id) -> (),
        ),
        map_upload_bo: Some(
            wined3d_cs_map_upload_bo
                as unsafe extern "C" fn(
                    *mut wined3d_device_context,
                    *mut wined3d_resource,
                    ::core::ffi::c_uint,
                    *mut wined3d_map_desc,
                    *const wined3d_box,
                    uint32_t,
                ) -> bool,
        ),
        unmap_upload_bo: Some(
            wined3d_cs_unmap_upload_bo
                as unsafe extern "C" fn(
                    *mut wined3d_device_context,
                    *mut wined3d_resource,
                    ::core::ffi::c_uint,
                    *mut wined3d_box,
                    *mut upload_bo,
                ) -> bool,
        ),
        issue_query: Some(
            wined3d_cs_issue_query
                as unsafe extern "C" fn(
                    *mut wined3d_device_context,
                    *mut wined3d_query,
                    ::core::ffi::c_uint,
                ) -> (),
        ),
        // bindgen typed this one field `extern "system"` on amd64/arm64 (`"C"` on i386) though the
        // header declares a plain pointer; on 64-bit the two are one ABI, so this is an identity.
        flush: Some(core::mem::transmute::<unsafe extern "C" fn(*mut wined3d_device_context), _>(wined3d_cs_flush)),
        reference_resource: Some(
            wined3d_cs_reference_resource
                as unsafe extern "C" fn(*mut wined3d_device_context, *mut wined3d_resource) -> (),
        ),
        reference_command_list: Some(
            wined3d_cs_reference_command_list
                as unsafe extern "C" fn(
                    *mut wined3d_device_context,
                    *mut wined3d_command_list,
                ) -> (),
        ),
    }
};

static mut wined3d_cs_mt_ops: wined3d_device_context_ops = unsafe {
    wined3d_device_context_ops {
        require_space: Some(
            wined3d_cs_mt_require_space
                as unsafe extern "C" fn(
                    *mut wined3d_device_context,
                    size_t,
                    wined3d_cs_queue_id,
                ) -> *mut ::core::ffi::c_void,
        ),
        submit: Some(
            wined3d_cs_mt_submit
                as unsafe extern "C" fn(*mut wined3d_device_context, wined3d_cs_queue_id) -> (),
        ),
        finish: Some(
            wined3d_cs_mt_finish
                as unsafe extern "C" fn(*mut wined3d_device_context, wined3d_cs_queue_id) -> (),
        ),
        map_upload_bo: Some(
            wined3d_cs_map_upload_bo
                as unsafe extern "C" fn(
                    *mut wined3d_device_context,
                    *mut wined3d_resource,
                    ::core::ffi::c_uint,
                    *mut wined3d_map_desc,
                    *const wined3d_box,
                    uint32_t,
                ) -> bool,
        ),
        unmap_upload_bo: Some(
            wined3d_cs_unmap_upload_bo
                as unsafe extern "C" fn(
                    *mut wined3d_device_context,
                    *mut wined3d_resource,
                    ::core::ffi::c_uint,
                    *mut wined3d_box,
                    *mut upload_bo,
                ) -> bool,
        ),
        issue_query: Some(
            wined3d_cs_issue_query
                as unsafe extern "C" fn(
                    *mut wined3d_device_context,
                    *mut wined3d_query,
                    ::core::ffi::c_uint,
                ) -> (),
        ),
        // bindgen typed this one field `extern "system"` on amd64/arm64 (`"C"` on i386) though the
        // header declares a plain pointer; on 64-bit the two are one ABI, so this is an identity.
        flush: Some(core::mem::transmute::<unsafe extern "C" fn(*mut wined3d_device_context), _>(wined3d_cs_flush)),
        reference_resource: Some(
            wined3d_cs_reference_resource
                as unsafe extern "C" fn(*mut wined3d_device_context, *mut wined3d_resource) -> (),
        ),
        reference_command_list: Some(
            wined3d_cs_reference_command_list
                as unsafe extern "C" fn(
                    *mut wined3d_device_context,
                    *mut wined3d_command_list,
                ) -> (),
        ),
    }
};

static mut wined3d_deferred_context_ops: wined3d_device_context_ops = unsafe {
    wined3d_device_context_ops {
        require_space: Some(
            wined3d_deferred_context_require_space
                as unsafe extern "C" fn(
                    *mut wined3d_device_context,
                    size_t,
                    wined3d_cs_queue_id,
                ) -> *mut ::core::ffi::c_void,
        ),
        submit: Some(
            wined3d_deferred_context_submit
                as unsafe extern "C" fn(*mut wined3d_device_context, wined3d_cs_queue_id) -> (),
        ),
        finish: Some(
            wined3d_deferred_context_finish
                as unsafe extern "C" fn(*mut wined3d_device_context, wined3d_cs_queue_id) -> (),
        ),
        map_upload_bo: Some(
            wined3d_deferred_context_map_upload_bo
                as unsafe extern "C" fn(
                    *mut wined3d_device_context,
                    *mut wined3d_resource,
                    ::core::ffi::c_uint,
                    *mut wined3d_map_desc,
                    *const wined3d_box,
                    uint32_t,
                ) -> bool,
        ),
        unmap_upload_bo: Some(
            wined3d_deferred_context_unmap_upload_bo
                as unsafe extern "C" fn(
                    *mut wined3d_device_context,
                    *mut wined3d_resource,
                    ::core::ffi::c_uint,
                    *mut wined3d_box,
                    *mut upload_bo,
                ) -> bool,
        ),
        issue_query: Some(
            wined3d_deferred_context_issue_query
                as unsafe extern "C" fn(
                    *mut wined3d_device_context,
                    *mut wined3d_query,
                    ::core::ffi::c_uint,
                ) -> (),
        ),
        // bindgen typed this one field `extern "system"` on amd64/arm64 (`"C"` on i386) though the
        // header declares a plain pointer; on 64-bit the two are one ABI, so this is an identity.
        flush: Some(core::mem::transmute::<unsafe extern "C" fn(*mut wined3d_device_context), _>(wined3d_deferred_context_flush)),
        reference_resource: Some(
            wined3d_deferred_context_reference_resource
                as unsafe extern "C" fn(*mut wined3d_device_context, *mut wined3d_resource) -> (),
        ),
        reference_command_list: Some(
            wined3d_deferred_context_reference_command_list
                as unsafe extern "C" fn(
                    *mut wined3d_device_context,
                    *mut wined3d_command_list,
                ) -> (),
        ),
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(d3d) / WINE_DECLARE_DEBUG_CHANNEL(d3d_perf|d3d_sync|fps|frametime): debug-channel macros, no Rust declaration emitted.

pub const WINED3D_INITIAL_CS_SIZE: usize = 4096;



pub type wined3d_cs_op = i32;
pub const WINED3D_CS_OP_NOP: wined3d_cs_op = 0;
pub const WINED3D_CS_OP_PRESENT: wined3d_cs_op = 1;
pub const WINED3D_CS_OP_CLEAR: wined3d_cs_op = 2;
pub const WINED3D_CS_OP_DISPATCH: wined3d_cs_op = 3;
pub const WINED3D_CS_OP_DRAW: wined3d_cs_op = 4;
pub const WINED3D_CS_OP_FLUSH: wined3d_cs_op = 5;
pub const WINED3D_CS_OP_SET_PREDICATION: wined3d_cs_op = 6;
pub const WINED3D_CS_OP_SET_VIEWPORTS: wined3d_cs_op = 7;
pub const WINED3D_CS_OP_SET_SCISSOR_RECTS: wined3d_cs_op = 8;
pub const WINED3D_CS_OP_SET_RENDERTARGET_VIEWS: wined3d_cs_op = 9;
pub const WINED3D_CS_OP_SET_DEPTH_STENCIL_VIEW: wined3d_cs_op = 10;
pub const WINED3D_CS_OP_SET_VERTEX_DECLARATION: wined3d_cs_op = 11;
pub const WINED3D_CS_OP_SET_STREAM_SOURCES: wined3d_cs_op = 12;
pub const WINED3D_CS_OP_SET_STREAM_OUTPUTS: wined3d_cs_op = 13;
pub const WINED3D_CS_OP_SET_INDEX_BUFFER: wined3d_cs_op = 14;
pub const WINED3D_CS_OP_SET_CONSTANT_BUFFERS: wined3d_cs_op = 15;
pub const WINED3D_CS_OP_SET_TEXTURE: wined3d_cs_op = 16;
pub const WINED3D_CS_OP_SET_SHADER_RESOURCE_VIEWS: wined3d_cs_op = 17;
pub const WINED3D_CS_OP_SET_UNORDERED_ACCESS_VIEWS: wined3d_cs_op = 18;
pub const WINED3D_CS_OP_SET_SAMPLERS: wined3d_cs_op = 19;
pub const WINED3D_CS_OP_SET_SHADER: wined3d_cs_op = 20;
pub const WINED3D_CS_OP_SET_BLEND_STATE: wined3d_cs_op = 21;
pub const WINED3D_CS_OP_SET_DEPTH_STENCIL_STATE: wined3d_cs_op = 22;
pub const WINED3D_CS_OP_SET_RASTERIZER_STATE: wined3d_cs_op = 23;
pub const WINED3D_CS_OP_SET_DEPTH_BOUNDS: wined3d_cs_op = 24;
pub const WINED3D_CS_OP_SET_RENDER_STATE: wined3d_cs_op = 25;
pub const WINED3D_CS_OP_SET_TEXTURE_STATE: wined3d_cs_op = 26;
pub const WINED3D_CS_OP_SET_TRANSFORM: wined3d_cs_op = 27;
pub const WINED3D_CS_OP_SET_CLIP_PLANE: wined3d_cs_op = 28;
pub const WINED3D_CS_OP_SET_COLOR_KEY: wined3d_cs_op = 29;
pub const WINED3D_CS_OP_SET_LIGHT: wined3d_cs_op = 30;
pub const WINED3D_CS_OP_SET_LIGHT_ENABLE: wined3d_cs_op = 31;
pub const WINED3D_CS_OP_SET_FEATURE_LEVEL: wined3d_cs_op = 32;
pub const WINED3D_CS_OP_PUSH_CONSTANTS: wined3d_cs_op = 33;
pub const WINED3D_CS_OP_RESET_STATE: wined3d_cs_op = 34;
pub const WINED3D_CS_OP_CALLBACK: wined3d_cs_op = 35;
pub const WINED3D_CS_OP_QUERY_ISSUE: wined3d_cs_op = 36;
pub const WINED3D_CS_OP_PRELOAD_RESOURCE: wined3d_cs_op = 37;
pub const WINED3D_CS_OP_UNLOAD_RESOURCE: wined3d_cs_op = 38;
pub const WINED3D_CS_OP_MAP: wined3d_cs_op = 39;
pub const WINED3D_CS_OP_UNMAP: wined3d_cs_op = 40;
pub const WINED3D_CS_OP_MAP_BO_ADDRESS: wined3d_cs_op = 41;
pub const WINED3D_CS_OP_BLT_SUB_RESOURCE: wined3d_cs_op = 42;
pub const WINED3D_CS_OP_UPDATE_SUB_RESOURCE: wined3d_cs_op = 43;
pub const WINED3D_CS_OP_ADD_DIRTY_TEXTURE_REGION: wined3d_cs_op = 44;
pub const WINED3D_CS_OP_CLEAR_SYSMEM_TEXTURE: wined3d_cs_op = 45;
pub const WINED3D_CS_OP_CLEAR_UNORDERED_ACCESS_VIEW: wined3d_cs_op = 46;
pub const WINED3D_CS_OP_COPY_UAV_COUNTER: wined3d_cs_op = 47;
pub const WINED3D_CS_OP_GENERATE_MIPMAPS: wined3d_cs_op = 48;
pub const WINED3D_CS_OP_EXECUTE_COMMAND_LIST: wined3d_cs_op = 49;
pub const WINED3D_CS_OP_STOP: wined3d_cs_op = 50;

#[repr(C)]
pub struct wined3d_cs_packet {
    pub size: usize,
    pub data: [u8; 1],
}

#[repr(C)]
pub struct wined3d_cs_nop {
    pub opcode: wined3d_cs_op,
}

#[repr(C)]
pub struct wined3d_cs_present {
    pub opcode: wined3d_cs_op,
    pub dst_window_override: HWND,
    pub swapchain: *mut wined3d_swapchain,
    pub src_rect: RECT,
    pub dst_rect: RECT,
    pub swap_interval: u32,
    pub flags: u32,
}

#[repr(C)]
pub struct wined3d_cs_clear {
    pub opcode: wined3d_cs_op,
    pub flags: u32,
    pub rt_count: u32,
    pub fb: wined3d_fb_state,
    pub draw_rect: RECT,
    pub color: wined3d_color,
    pub depth: f32,
    pub stencil: u32,
    pub rect_count: u32,
    pub rects: [RECT; 1],
}

#[repr(C)]
pub struct wined3d_cs_clear_sysmem_texture {
    pub opcode: wined3d_cs_op,
    pub texture: *mut wined3d_texture,
    pub sub_resource_idx: u32,
    pub color: wined3d_color,
    pub rect: RECT,
}

#[repr(C)]
pub struct wined3d_cs_dispatch {
    pub opcode: wined3d_cs_op,
    pub parameters: wined3d_dispatch_parameters,
}

#[repr(C)]
pub struct wined3d_cs_draw {
    pub opcode: wined3d_cs_op,
    pub primitive_type: wined3d_primitive_type,
    pub patch_vertex_count: u32,
    pub parameters: wined3d_draw_parameters,
}

#[repr(C)]
pub struct wined3d_cs_flush {
    pub opcode: wined3d_cs_op,
}

#[repr(C)]
pub struct wined3d_cs_set_predication {
    pub opcode: wined3d_cs_op,
    pub predicate: *mut wined3d_query,
    pub value: i32,
}

#[repr(C)]
pub struct wined3d_cs_set_viewports {
    pub opcode: wined3d_cs_op,
    pub viewport_count: u32,
    pub viewports: [wined3d_viewport; 1],
}

#[repr(C)]
pub struct wined3d_cs_set_scissor_rects {
    pub opcode: wined3d_cs_op,
    pub rect_count: u32,
    pub rects: [RECT; 1],
}

#[repr(C)]
pub struct wined3d_cs_set_rendertarget_views {
    pub opcode: wined3d_cs_op,
    pub start_idx: u32,
    pub count: u32,
    pub views: [*mut wined3d_rendertarget_view; 1],
}

#[repr(C)]
pub struct wined3d_cs_set_depth_stencil_view {
    pub opcode: wined3d_cs_op,
    pub view: *mut wined3d_rendertarget_view,
}

#[repr(C)]
pub struct wined3d_cs_set_vertex_declaration {
    pub opcode: wined3d_cs_op,
    pub declaration: *mut wined3d_vertex_declaration,
}

#[repr(C)]
pub struct wined3d_cs_set_stream_sources {
    pub opcode: wined3d_cs_op,
    pub start_idx: u32,
    pub count: u32,
    pub streams: [wined3d_stream_state; 1],
}

#[repr(C)]
pub struct wined3d_cs_set_stream_outputs {
    pub opcode: wined3d_cs_op,
    pub outputs: [wined3d_stream_output; (WINED3D_MAX_STREAM_OUTPUT_BUFFERS as _)],
}

#[repr(C)]
pub struct wined3d_cs_set_index_buffer {
    pub opcode: wined3d_cs_op,
    pub buffer: *mut wined3d_buffer,
    pub format_id: wined3d_format_id,
    pub offset: u32,
}

#[repr(C)]
pub struct wined3d_cs_set_constant_buffers {
    pub opcode: wined3d_cs_op,
    pub type_: wined3d_shader_type,
    pub start_idx: u32,
    pub count: u32,
    pub buffers: [wined3d_constant_buffer_state; 1],
}

#[repr(C)]
pub struct wined3d_cs_set_texture {
    pub opcode: wined3d_cs_op,
    pub shader_type: wined3d_shader_type,
    pub bind_index: u32,
    pub view: *mut wined3d_shader_resource_view,
}

#[repr(C)]
pub struct wined3d_cs_set_color_key {
    pub opcode: wined3d_cs_op,
    pub texture: *mut wined3d_texture,
    pub flags: u16,
    pub set: u16,
    pub color_key: wined3d_color_key,
}

#[repr(C)]
pub struct wined3d_cs_set_shader_resource_views {
    pub opcode: wined3d_cs_op,
    pub type_: wined3d_shader_type,
    pub start_idx: u32,
    pub count: u32,
    pub views: [*mut wined3d_shader_resource_view; 1],
}

#[repr(C)]
pub struct wined3d_cs_uav_entry {
    pub view: *mut wined3d_unordered_access_view,
    pub initial_count: u32,
}

#[repr(C)]
pub struct wined3d_cs_set_unordered_access_views {
    pub opcode: wined3d_cs_op,
    pub pipeline: wined3d_pipeline,
    pub start_idx: u32,
    pub count: u32,
    pub uavs: [wined3d_cs_uav_entry; 1],
}

#[repr(C)]
pub struct wined3d_cs_set_samplers {
    pub opcode: wined3d_cs_op,
    pub type_: wined3d_shader_type,
    pub start_idx: u32,
    pub count: u32,
    pub samplers: [*mut wined3d_sampler; 1],
}

#[repr(C)]
pub struct wined3d_cs_set_shader {
    pub opcode: wined3d_cs_op,
    pub type_: wined3d_shader_type,
    pub shader: *mut wined3d_shader,
}

#[repr(C)]
pub struct wined3d_cs_set_blend_state {
    pub opcode: wined3d_cs_op,
    pub state: *mut wined3d_blend_state,
    pub factor: wined3d_color,
    pub sample_mask: u32,
}

#[repr(C)]
pub struct wined3d_cs_set_depth_stencil_state {
    pub opcode: wined3d_cs_op,
    pub state: *mut wined3d_depth_stencil_state,
    pub stencil_ref: u32,
}

#[repr(C)]
pub struct wined3d_cs_set_rasterizer_state {
    pub opcode: wined3d_cs_op,
    pub state: *mut wined3d_rasterizer_state,
}

#[repr(C)]
pub struct wined3d_cs_set_depth_bounds {
    pub opcode: wined3d_cs_op,
    pub enable: bool,
    pub min_depth: f32,
    pub max_depth: f32,
}

#[repr(C)]
pub struct wined3d_cs_set_render_state {
    pub opcode: wined3d_cs_op,
    pub state: wined3d_render_state,
    pub value: u32,
}

#[repr(C)]
pub struct wined3d_cs_set_texture_state {
    pub opcode: wined3d_cs_op,
    pub stage: u32,
    pub state: wined3d_texture_stage_state,
    pub value: u32,
}

#[repr(C)]
pub struct wined3d_cs_set_transform {
    pub opcode: wined3d_cs_op,
    pub state: wined3d_transform_state,
    pub matrix: wined3d_matrix,
}

#[repr(C)]
pub struct wined3d_cs_set_clip_plane {
    pub opcode: wined3d_cs_op,
    pub plane_idx: u32,
    pub plane: wined3d_vec4,
}

#[repr(C)]
pub struct wined3d_cs_set_light {
    pub opcode: wined3d_cs_op,
    pub light: wined3d_light_info,
}

#[repr(C)]
pub struct wined3d_cs_set_light_enable {
    pub opcode: wined3d_cs_op,
    pub idx: u32,
    pub enable: i32,
}

#[repr(C)]
pub struct wined3d_cs_set_feature_level {
    pub opcode: wined3d_cs_op,
    pub level: wined3d_feature_level,
}

#[repr(C)]
pub struct wined3d_cs_push_constants {
    pub opcode: wined3d_cs_op,
    pub type_: wined3d_push_constants,
    pub start_idx: u32,
    pub count: u32,
    pub update_mask: u32,
}

#[repr(C)]
pub struct wined3d_cs_reset_state {
    pub opcode: wined3d_cs_op,
    pub invalidate: bool,
}

#[repr(C)]
pub struct wined3d_cs_callback {
    pub opcode: wined3d_cs_op,
    pub callback: Option<unsafe extern "C" fn(object: *mut core::ffi::c_void)>,
    pub object: *mut core::ffi::c_void,
}

#[repr(C)]
pub struct wined3d_cs_query_issue {
    pub opcode: wined3d_cs_op,
    pub query: *mut wined3d_query,
    pub flags: u32,
}

#[repr(C)]
pub struct wined3d_cs_preload_resource {
    pub opcode: wined3d_cs_op,
    pub resource: *mut wined3d_resource,
}

#[repr(C)]
pub struct wined3d_cs_unload_resource {
    pub opcode: wined3d_cs_op,
    pub resource: *mut wined3d_resource,
}

#[repr(C)]
pub struct wined3d_cs_map {
    pub opcode: wined3d_cs_op,
    pub resource: *mut wined3d_resource,
    pub sub_resource_idx: u32,
    pub map_ptr: *mut *mut core::ffi::c_void,
    pub box_: *const wined3d_box,
    pub flags: u32,
    pub hr: *mut HRESULT,
}

#[repr(C)]
pub struct wined3d_cs_unmap {
    pub opcode: wined3d_cs_op,
    pub resource: *mut wined3d_resource,
    pub sub_resource_idx: u32,
    pub hr: *mut HRESULT,
}

#[repr(C)]
pub struct wined3d_cs_map_bo_address {
    pub opcode: wined3d_cs_op,
    pub addr: wined3d_bo_address,
    pub size: usize,
    pub flags: u32,
}

#[repr(C)]
pub struct wined3d_cs_blt_sub_resource {
    pub opcode: wined3d_cs_op,
    pub dst_resource: *mut wined3d_resource,
    pub dst_sub_resource_idx: u32,
    pub dst_box: wined3d_box,
    pub src_resource: *mut wined3d_resource,
    pub src_sub_resource_idx: u32,
    pub src_box: wined3d_box,
    pub flags: u32,
    pub fx: wined3d_blt_fx,
    pub filter: wined3d_texture_filter_type,
}

#[repr(C)]
pub struct wined3d_cs_update_sub_resource {
    pub opcode: wined3d_cs_op,
    pub resource: *mut wined3d_resource,
    pub sub_resource_idx: u32,
    pub box_: wined3d_box,
    pub bo: upload_bo,
    pub row_pitch: u32,
    pub slice_pitch: u32,
}

#[repr(C)]
pub struct wined3d_cs_add_dirty_texture_region {
    pub opcode: wined3d_cs_op,
    pub texture: *mut wined3d_texture,
    pub layer: u32,
}

#[repr(C)]
pub struct wined3d_cs_clear_unordered_access_view {
    pub opcode: wined3d_cs_op,
    pub view: *mut wined3d_unordered_access_view,
    pub clear_value: wined3d_uvec4,
    pub fp: bool,
}

#[repr(C)]
pub struct wined3d_cs_copy_uav_counter {
    pub opcode: wined3d_cs_op,
    pub buffer: *mut wined3d_buffer,
    pub offset: u32,
    pub view: *mut wined3d_unordered_access_view,
}

#[repr(C)]
pub struct wined3d_cs_generate_mipmaps {
    pub opcode: wined3d_cs_op,
    pub view: *mut wined3d_shader_resource_view,
}

#[repr(C)]
pub struct wined3d_cs_execute_command_list {
    pub opcode: wined3d_cs_op,
    pub list: *mut wined3d_command_list,
}

#[repr(C)]
pub struct wined3d_cs_stop {
    pub opcode: wined3d_cs_op,
}

#[repr(C)]
pub struct push_constant_info {
    pub size: usize,
    pub max_count: u32,
    pub shader_type: wined3d_shader_type,
}

// Indexed by WINED3D_PUSH_CONSTANTS_* (VS_F, PS_F, VS_I, PS_I, VS_B, PS_B, VS_FFP, PS_FFP).

// static void wined3d_cs_exec_execute_command_list(struct wined3d_cs *cs, const void *data);  (forward declaration, no Rust item)

#[repr(C)]
pub struct wined3d_deferred_context {
    pub c: wined3d_device_context,
    pub data_size: usize,
    pub data_capacity: usize,
    pub data: *mut core::ffi::c_void,
    pub resource_count: usize,
    pub resources_capacity: usize,
    pub resources: *mut *mut wined3d_resource,
    pub upload_count: usize,
    pub uploads_capacity: usize,
    pub uploads: *mut wined3d_deferred_upload,
    pub upload_heap: HANDLE,
    pub upload_heap_refcount: *mut i32,
    /// List of command lists queued for execution on this context. A command
    /// list can be the only thing holding a pointer to another command list, so
    /// we need to hold a reference here and in wined3d_command_list as well.
    pub command_list_count: usize,
    pub command_lists_capacity: usize,
    pub command_lists: *mut *mut wined3d_command_list,
    pub query_count: usize,
    pub queries_capacity: usize,
    pub queries: *mut wined3d_deferred_query_issue,
}

unsafe extern "C" fn discard_client_address(resource: *mut wined3d_resource) {
    let client: *mut wined3d_client_resource = core::ptr::addr_of_mut!((*resource).client);
    (*client).addr.buffer_object = CLIENT_BO_DISCARDED;
    (*client).addr.addr = core::ptr::null_mut();
}

unsafe extern "C" fn invalidate_client_address(resource: *mut wined3d_resource) {
    let client: *mut wined3d_client_resource = core::ptr::addr_of_mut!((*resource).client);
    memset(
        core::ptr::addr_of_mut!((*client).addr) as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<wined3d_bo_address>() as size_t,
    );
}

unsafe extern "C" fn wined3d_device_context_require_space(
    context: *mut wined3d_device_context,
    size: usize,
    queue_id: wined3d_cs_queue_id,
) -> *mut core::ffi::c_void {
    return ((*(*context).ops).require_space.unwrap_unchecked())(context, size, queue_id);
}

unsafe extern "C" fn wined3d_device_context_submit(
    context: *mut wined3d_device_context,
    queue_id: wined3d_cs_queue_id,
) {
    (*(*context).ops).submit.unwrap_unchecked()(context, queue_id);
}

unsafe extern "C" fn wined3d_device_context_finish(
    context: *mut wined3d_device_context,
    queue_id: wined3d_cs_queue_id,
) {
    (*(*context).ops).finish.unwrap_unchecked()(context, queue_id);
}

unsafe extern "C" fn wined3d_device_context_reference_resource(
    context: *mut wined3d_device_context,
    resource: *mut wined3d_resource,
) {
    (*(*context).ops).reference_resource.unwrap_unchecked()(context, resource);
}

unsafe extern "C" fn wined3d_cs_from_context(
    context: *mut wined3d_device_context,
) -> *mut wined3d_cs {
    return (context as usize).wrapping_sub(core::mem::offset_of!(wined3d_cs, c)) as *mut wined3d_cs;
}

unsafe extern "C" fn debug_cs_op(op: wined3d_cs_op) -> *const ::core::ffi::c_char {
    match op {
        WINED3D_CS_OP_NOP => return b"WINED3D_CS_OP_NOP\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_PRESENT => return b"WINED3D_CS_OP_PRESENT\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_CLEAR => return b"WINED3D_CS_OP_CLEAR\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_DISPATCH => return b"WINED3D_CS_OP_DISPATCH\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_DRAW => return b"WINED3D_CS_OP_DRAW\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_FLUSH => return b"WINED3D_CS_OP_FLUSH\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_PREDICATION => return b"WINED3D_CS_OP_SET_PREDICATION\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_VIEWPORTS => return b"WINED3D_CS_OP_SET_VIEWPORTS\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_SCISSOR_RECTS => return b"WINED3D_CS_OP_SET_SCISSOR_RECTS\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_RENDERTARGET_VIEWS => return b"WINED3D_CS_OP_SET_RENDERTARGET_VIEWS\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_DEPTH_STENCIL_VIEW => return b"WINED3D_CS_OP_SET_DEPTH_STENCIL_VIEW\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_VERTEX_DECLARATION => return b"WINED3D_CS_OP_SET_VERTEX_DECLARATION\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_STREAM_SOURCES => return b"WINED3D_CS_OP_SET_STREAM_SOURCES\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_STREAM_OUTPUTS => return b"WINED3D_CS_OP_SET_STREAM_OUTPUTS\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_INDEX_BUFFER => return b"WINED3D_CS_OP_SET_INDEX_BUFFER\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_CONSTANT_BUFFERS => return b"WINED3D_CS_OP_SET_CONSTANT_BUFFERS\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_TEXTURE => return b"WINED3D_CS_OP_SET_TEXTURE\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_SHADER_RESOURCE_VIEWS => return b"WINED3D_CS_OP_SET_SHADER_RESOURCE_VIEWS\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_UNORDERED_ACCESS_VIEWS => return b"WINED3D_CS_OP_SET_UNORDERED_ACCESS_VIEWS\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_SAMPLERS => return b"WINED3D_CS_OP_SET_SAMPLERS\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_SHADER => return b"WINED3D_CS_OP_SET_SHADER\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_BLEND_STATE => return b"WINED3D_CS_OP_SET_BLEND_STATE\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_DEPTH_STENCIL_STATE => return b"WINED3D_CS_OP_SET_DEPTH_STENCIL_STATE\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_RASTERIZER_STATE => return b"WINED3D_CS_OP_SET_RASTERIZER_STATE\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_DEPTH_BOUNDS => return b"WINED3D_CS_OP_SET_DEPTH_BOUNDS\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_RENDER_STATE => return b"WINED3D_CS_OP_SET_RENDER_STATE\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_TEXTURE_STATE => return b"WINED3D_CS_OP_SET_TEXTURE_STATE\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_TRANSFORM => return b"WINED3D_CS_OP_SET_TRANSFORM\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_CLIP_PLANE => return b"WINED3D_CS_OP_SET_CLIP_PLANE\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_COLOR_KEY => return b"WINED3D_CS_OP_SET_COLOR_KEY\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_LIGHT => return b"WINED3D_CS_OP_SET_LIGHT\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_LIGHT_ENABLE => return b"WINED3D_CS_OP_SET_LIGHT_ENABLE\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_SET_FEATURE_LEVEL => return b"WINED3D_CS_OP_SET_FEATURE_LEVEL\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_PUSH_CONSTANTS => return b"WINED3D_CS_OP_PUSH_CONSTANTS\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_RESET_STATE => return b"WINED3D_CS_OP_RESET_STATE\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_CALLBACK => return b"WINED3D_CS_OP_CALLBACK\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_QUERY_ISSUE => return b"WINED3D_CS_OP_QUERY_ISSUE\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_PRELOAD_RESOURCE => return b"WINED3D_CS_OP_PRELOAD_RESOURCE\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_UNLOAD_RESOURCE => return b"WINED3D_CS_OP_UNLOAD_RESOURCE\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_MAP => return b"WINED3D_CS_OP_MAP\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_UNMAP => return b"WINED3D_CS_OP_UNMAP\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_MAP_BO_ADDRESS => return b"WINED3D_CS_OP_MAP_BO_ADDRESS\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_BLT_SUB_RESOURCE => return b"WINED3D_CS_OP_BLT_SUB_RESOURCE\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_UPDATE_SUB_RESOURCE => return b"WINED3D_CS_OP_UPDATE_SUB_RESOURCE\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_ADD_DIRTY_TEXTURE_REGION => return b"WINED3D_CS_OP_ADD_DIRTY_TEXTURE_REGION\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_CLEAR_SYSMEM_TEXTURE => return b"WINED3D_CS_OP_CLEAR_SYSMEM_TEXTURE\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_CLEAR_UNORDERED_ACCESS_VIEW => return b"WINED3D_CS_OP_CLEAR_UNORDERED_ACCESS_VIEW\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_COPY_UAV_COUNTER => return b"WINED3D_CS_OP_COPY_UAV_COUNTER\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_GENERATE_MIPMAPS => return b"WINED3D_CS_OP_GENERATE_MIPMAPS\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_EXECUTE_COMMAND_LIST => return b"WINED3D_CS_OP_EXECUTE_COMMAND_LIST\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_CS_OP_STOP => return b"WINED3D_CS_OP_STOP\0" as *const u8 as *const ::core::ffi::c_char,
        _ => {}
    }
    return wine_dbg_sprintf(
        b"UNKNOWN_OP(%#x)\0" as *const u8 as *const ::core::ffi::c_char,
        op as ::core::ffi::c_uint,
    );
}

unsafe extern "C" fn wined3d_next_cs_packet(data: *const uint8_t, offset: *mut SIZE_T, mask: SIZE_T) -> *mut wined3d_cs_packet {
    let packet: *mut wined3d_cs_packet = (data as *const u8).add((*offset & mask) as usize) as *mut wined3d_cs_packet;

    *offset = (*offset).wrapping_add((core::mem::offset_of!(wined3d_cs_packet, data) + (*packet).size as usize * core::mem::size_of::<u8>()) as _);

    return packet;
}

unsafe extern "C" fn wined3d_cs_exec_nop(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {}

unsafe extern "C" fn wined3d_cs_exec_present(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    static mut freq: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };

    let mut logo_texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut cursor_texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut back_buffer: *mut wined3d_texture = core::ptr::null_mut();
    let mut dsv: *mut wined3d_rendertarget_view = (*cs).state.fb.depth_stencil;
    let op: *const wined3d_cs_present = data as *const wined3d_cs_present;
    let mut desc: *const wined3d_swapchain_desc = core::ptr::null();
    let mut swapchain: *mut wined3d_swapchain = core::ptr::null_mut();
    let mut elapsed_time: LONGLONG;
    let mut time: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };

    if freq.QuadPart == 0 {
        QueryPerformanceFrequency(&raw mut freq);
    }

    swapchain = (*op).swapchain;
    desc = &raw const (*swapchain).state.desc;
    back_buffer = *(*swapchain).back_buffers.offset(0 as isize);
    wined3d_swapchain_set_window(swapchain, (*op).dst_window_override);

    logo_texture = (*(*swapchain).device).logo_texture;
    if !logo_texture.is_null() {
        let mut rect: RECT = RECT {
            left: 0 as LONG,
            top: 0 as LONG,
            right: (*logo_texture).resource.width as LONG,
            bottom: (*logo_texture).resource.height as LONG,
        };

        /* Blit the logo into the upper left corner of the back-buffer. */
        wined3d_device_context_blt(
            &raw mut (*cs).c,
            back_buffer,
            0 as core::ffi::c_uint,
            &raw mut rect,
            logo_texture,
            0 as core::ffi::c_uint,
            &raw mut rect,
            WINED3D_BLT_SRC_CKEY as core::ffi::c_uint,
            core::ptr::null(),
            WINED3D_TEXF_POINT,
        );
    }

    cursor_texture = (*(*swapchain).device).cursor_texture;
    if !cursor_texture.is_null()
        && (*(*swapchain).device).bCursorVisible() != 0
        && (*(*swapchain).device).hardwareCursor.is_null()
    {
        let mut dst_rect: RECT = RECT {
            left: (*(*swapchain).device)
                .xScreenSpace
                .wrapping_sub((*(*swapchain).device).xHotSpot) as LONG,
            top: (*(*swapchain).device)
                .yScreenSpace
                .wrapping_sub((*(*swapchain).device).yHotSpot) as LONG,
            right: (*(*swapchain).device)
                .xScreenSpace
                .wrapping_add((*(*swapchain).device).cursorWidth)
                .wrapping_sub((*(*swapchain).device).xHotSpot) as LONG,
            bottom: (*(*swapchain).device)
                .yScreenSpace
                .wrapping_add((*(*swapchain).device).cursorHeight)
                .wrapping_sub((*(*swapchain).device).yHotSpot) as LONG,
        };
        let mut src_rect: RECT = RECT {
            left: 0 as LONG,
            top: 0 as LONG,
            right: (*cursor_texture).resource.width as LONG,
            bottom: (*cursor_texture).resource.height as LONG,
        };
        let clip_rect: RECT = RECT {
            left: 0 as LONG,
            top: 0 as LONG,
            right: (*back_buffer).resource.width as LONG,
            bottom: (*back_buffer).resource.height as LONG,
        };

        if (*desc).windowed != 0 {
            MapWindowPoints(
                core::ptr::null_mut(),
                (*swapchain).win_handle,
                &raw mut dst_rect as LPPOINT,
                2 as UINT,
            );
        }
        if wined3d_clip_blit(&raw const clip_rect, &raw mut dst_rect, &raw mut src_rect) != 0 {
            wined3d_device_context_blt(
                &raw mut (*cs).c,
                back_buffer,
                0 as core::ffi::c_uint,
                &raw mut dst_rect,
                cursor_texture,
                0 as core::ffi::c_uint,
                &raw mut src_rect,
                WINED3D_BLT_ALPHA_TEST as core::ffi::c_uint,
                core::ptr::null(),
                WINED3D_TEXF_POINT,
            );
        }
    }

    (*(*swapchain).swapchain_ops)
        .swapchain_present
        .unwrap_unchecked()(
        swapchain,
        &raw const (*op).src_rect,
        &raw const (*op).dst_rect,
        (*op).swap_interval,
        (*op).flags,
    );

    /* Discard buffers if the swap effect allows it. */
    back_buffer = *(*swapchain).back_buffers.offset(
        (*desc)
            .backbuffer_count
            .wrapping_sub(1 as core::ffi::c_uint) as isize,
    );
    if (*desc).swap_effect as core::ffi::c_uint == WINED3D_SWAP_EFFECT_DISCARD as core::ffi::c_uint
        || (*desc).swap_effect as core::ffi::c_uint == WINED3D_SWAP_EFFECT_FLIP_DISCARD as core::ffi::c_uint
    {
        wined3d_texture_validate_location(
            back_buffer,
            0 as core::ffi::c_uint,
            WINED3D_LOCATION_DISCARDED as uint32_t,
        );
    }

    if !dsv.is_null()
        && (*(*dsv).resource).type_ as core::ffi::c_uint
            != WINED3D_RTYPE_BUFFER as core::ffi::c_uint
    {
        let ds: *mut wined3d_texture = texture_from_resource((*dsv).resource);

        if ((*desc).flags & (WINED3D_SWAPCHAIN_DISCARD_DEPTHSTENCIL as uint32_t)) != 0
            || ((*ds).flags & (WINED3D_TEXTURE_DISCARD as uint32_t)) != 0
        {
            wined3d_rendertarget_view_validate_location(dsv, WINED3D_LOCATION_DISCARDED as uint32_t);
        }
    }

    if ((TRACE_ON!(__wine_dbch_frametime)) != 0) {
        QueryPerformanceCounter(&raw mut time);
        if (*swapchain).last_present_time.QuadPart != 0 {
            elapsed_time = time.QuadPart - (*swapchain).last_present_time.QuadPart;
            WINE_TRACE_!(__wine_dbch_frametime,
                "Frame duration %u μs.\n",
                (elapsed_time * 1000000 as LONGLONG / freq.QuadPart) as core::ffi::c_uint
            );
        }
        (*swapchain).last_present_time = time;
    }

    if ((TRACE_ON!(__wine_dbch_fps)) != 0) {
        let time: DWORD = GetTickCount();
        (*swapchain).frames += 1;

        /* every 1.5 seconds */
        if time.wrapping_sub((*swapchain).prev_time as DWORD) > 1500 as DWORD {
            WINE_TRACE_!(__wine_dbch_fps,
                "%p @ approx %.2ffps\n",
                swapchain,
                1000.0f64 * (*swapchain).frames as f64
                    / time.wrapping_sub((*swapchain).prev_time as DWORD) as f64
            );
            (*swapchain).prev_time = time as LONG;
            (*swapchain).frames = 0 as LONG;
        }
    }

    InterlockedDecrement(&raw mut (*cs).pending_presents as *mut LONG);
    let __cmp_old: LONG = {
        let __p: *mut LONG = &raw mut (*cs).waiting_for_present as *mut LONG;
        match core::sync::atomic::AtomicI32::from_ptr(__p).compare_exchange(
            TRUE as LONG,
            FALSE as LONG,
            core::sync::atomic::Ordering::SeqCst,
            core::sync::atomic::Ordering::SeqCst,
        ) {
            Ok(v) => v,
            Err(v) => v,
        }
    };
    if __cmp_old != 0 {
        SetEvent((*cs).present_event);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_cs_emit_present(mut cs: *mut wined3d_cs, mut swapchain: *mut wined3d_swapchain, mut src_rect: *const RECT, mut dst_rect: *const RECT, mut dst_window_override: HWND, mut swap_interval: ::core::ffi::c_uint, mut flags: uint32_t) {
    let mut op: *mut wined3d_cs_present = ::core::ptr::null_mut::<wined3d_cs_present>();
    let mut i: ::core::ffi::c_uint = 0;
    let mut pending: LONG = 0;
    wined3d_not_from_cs(cs);
    op = wined3d_device_context_require_space(
        core::ptr::addr_of_mut!((*cs).c),
        core::mem::size_of::<wined3d_cs_present>() as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_present;
    (*op).opcode = WINED3D_CS_OP_PRESENT;
    (*op).dst_window_override = dst_window_override;
    (*op).swapchain = swapchain;
    (*op).src_rect = *src_rect;
    (*op).dst_rect = *dst_rect;
    (*op).swap_interval = swap_interval;
    (*op).flags = flags;
    pending = InterlockedIncrement(
        core::ptr::addr_of_mut!((*cs).pending_presents) as *mut ::core::ffi::c_long,
    ) as LONG;
    wined3d_resource_reference(core::ptr::addr_of_mut!((*(*swapchain).front_buffer).resource));
    i = 0 as ::core::ffi::c_uint;
    while i < (*swapchain).state.desc.backbuffer_count {
        wined3d_resource_reference(
            core::ptr::addr_of_mut!((**(*swapchain).back_buffers.offset(i as isize)).resource),
        );
        i = i.wrapping_add(1);
    }
    wined3d_device_context_submit(core::ptr::addr_of_mut!((*cs).c), WINED3D_CS_QUEUE_DEFAULT);
    /* Limit input latency by limiting the number of presents that we can get
     * ahead of the worker thread. */
    while (pending as ::core::ffi::c_uint) >= (*swapchain).max_frame_latency {
        InterlockedExchange(
            core::ptr::addr_of_mut!((*cs).waiting_for_present) as *mut ::core::ffi::c_long,
            TRUE as ::core::ffi::c_long,
        );
        pending = InterlockedCompareExchange(
            core::ptr::addr_of_mut!((*cs).pending_presents) as *mut ::core::ffi::c_long,
            0 as ::core::ffi::c_long,
            0 as ::core::ffi::c_long,
        ) as LONG;
        if (pending as ::core::ffi::c_uint) >= (*swapchain).max_frame_latency
            || InterlockedCompareExchange(
                core::ptr::addr_of_mut!((*cs).waiting_for_present) as *mut ::core::ffi::c_long,
                FALSE as ::core::ffi::c_long,
                TRUE as ::core::ffi::c_long,
            ) == 0
        {
            WINE_TRACE_!(__wine_dbch_d3d_perf,
                "Reached latency limit (%u frames), blocking to wait.\n",
                (*swapchain).max_frame_latency,
            );
            wined3d_mutex_unlock();
            WaitForSingleObject((*cs).present_event, INFINITE);
            wined3d_mutex_lock();
            WINE_TRACE_!(__wine_dbch_d3d_perf,"Woken up from the wait.\n");
        }
    }
}

unsafe extern "C" fn wined3d_cs_exec_clear(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let mut device: *mut wined3d_device = (*cs).c.device;
    let op: *const wined3d_cs_clear = data as *const wined3d_cs_clear;
    (*(*(*device).blitter).ops)
        .blitter_clear
        .unwrap_unchecked()(
        (*device).blitter,
        device,
        (*op).rt_count,
        core::ptr::addr_of!((*op).fb),
        (*op).rect_count,
        core::ptr::addr_of!((*op).rects) as *const RECT,
        core::ptr::addr_of!((*op).draw_rect),
        (*op).flags,
        core::ptr::addr_of!((*op).color),
        (*op).depth,
        (*op).stencil as core::ffi::c_uint,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_cs_emit_clear(mut cs: *mut wined3d_cs, mut rect_count: DWORD, mut rects: *const RECT, mut flags: uint32_t, mut color: *const wined3d_color, mut depth: ::core::ffi::c_float, mut stencil: DWORD) {
    let mut state: *const wined3d_state = (*cs).c.state;
    let mut vp: *const wined3d_viewport = (&raw const (*state).viewports as *const wined3d_viewport)
        .offset(0 as isize)
        as *const wined3d_viewport;
    let mut view: *mut wined3d_rendertarget_view =
        ::core::ptr::null_mut::<wined3d_rendertarget_view>();
    let mut op: *mut wined3d_cs_clear = ::core::ptr::null_mut::<wined3d_cs_clear>();
    let mut rt_count: ::core::ffi::c_uint = 0;
    let mut i: ::core::ffi::c_uint = 0;
    rt_count = if flags & WINED3DCLEAR_TARGET as uint32_t != 0 {
        (*(*(*cs).c.device).adapter).d3d_info.limits.max_rt_count
    } else {
        0 as ::core::ffi::c_uint
    };
    op = wined3d_device_context_require_space(
        &raw mut (*cs).c,
        (FIELD_OFFSET!(wined3d_cs_clear, rects[rect_count as usize]) as usize) as ::core::ffi::c_ulonglong as LONG
            as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_clear;
    (*op).opcode = WINED3D_CS_OP_CLEAR;
    (*op).flags =
        flags & (WINED3DCLEAR_TARGET | WINED3DCLEAR_ZBUFFER | WINED3DCLEAR_STENCIL) as uint32_t;
    (*op).rt_count = rt_count;
    (*op).fb = (*state).fb;
    SetRect(
        &raw mut (*op).draw_rect,
        (*vp).x as ::core::ffi::c_int,
        (*vp).y as ::core::ffi::c_int,
        ((*vp).x + (*vp).width) as ::core::ffi::c_int,
        ((*vp).y + (*vp).height) as ::core::ffi::c_int,
    );
    if !(*state).rasterizer_state.is_null() && (*(*state).rasterizer_state).desc.scissor != 0 {
        IntersectRect(
            &raw mut (*op).draw_rect,
            &raw mut (*op).draw_rect as LPCRECT,
            (&raw const (*state).scissor_rects as *const RECT)
                .offset(0 as isize) as LPCRECT,
        );
    }
    (*op).color = *color;
    (*op).depth = depth;
    (*op).stencil = stencil;
    (*op).rect_count = rect_count as ::core::ffi::c_uint;
    memcpy(
        &raw mut (*op).rects as *mut RECT as *mut ::core::ffi::c_void,
        rects as *const ::core::ffi::c_void,
        (::core::mem::size_of::<RECT>() as size_t).wrapping_mul(rect_count as size_t),
    );
    i = 0 as ::core::ffi::c_uint;
    while i < rt_count {
        view = (*state).fb.render_targets[i as usize];
        if !view.is_null() {
            wined3d_resource_reference((*view).resource);
        }
        i = i.wrapping_add(1);
    }
    if flags & (WINED3DCLEAR_ZBUFFER | WINED3DCLEAR_STENCIL) as uint32_t != 0 {
        view = (*state).fb.depth_stencil;
        wined3d_resource_reference((*view).resource);
    }

    wined3d_device_context_submit(&raw mut (*cs).c, WINED3D_CS_QUEUE_DEFAULT);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_clear_rendertarget_view(mut context: *mut wined3d_device_context, mut view: *mut wined3d_rendertarget_view, mut rect: *const RECT, mut flags: ::core::ffi::c_uint, mut color: *const wined3d_color, mut depth: ::core::ffi::c_float, mut stencil: ::core::ffi::c_uint) {
    let mut op: *mut wined3d_cs_clear = ::core::ptr::null_mut::<wined3d_cs_clear>();
    let mut size: size_t = 0;

    size = FIELD_OFFSET!(wined3d_cs_clear, rects[1]) as size_t;
    op = wined3d_device_context_require_space(context, size, WINED3D_CS_QUEUE_DEFAULT) as *mut wined3d_cs_clear;

    (*op).opcode = WINED3D_CS_OP_CLEAR;
    (*op).flags = (flags & (WINED3DCLEAR_TARGET | WINED3DCLEAR_ZBUFFER | WINED3DCLEAR_STENCIL)) as uint32_t;
    if flags & WINED3DCLEAR_TARGET != 0 {
        (*op).rt_count = 1;
        (*op).fb.render_targets[0 as usize] = view;
        (*op).fb.depth_stencil = ::core::ptr::null_mut::<wined3d_rendertarget_view>();
        (*op).color = *color;
    } else {
        (*op).rt_count = 0;
        (*op).fb.render_targets[0 as usize] =
            ::core::ptr::null_mut::<wined3d_rendertarget_view>();
        (*op).fb.depth_stencil = view;
        (*op).depth = depth;
        (*op).stencil = stencil as DWORD;
    }
    SetRect(
        core::ptr::addr_of_mut!((*op).draw_rect),
        0,
        0,
        (*view).width as ::core::ffi::c_int,
        (*view).height as ::core::ffi::c_int,
    );
    (*op).rect_count = 1;
    *(core::ptr::addr_of_mut!((*op).rects) as *mut RECT).offset(0 as isize) = *rect;

    wined3d_device_context_reference_resource(context, (*view).resource);

    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
    if flags & WINED3DCLEAR_SYNCHRONOUS != 0 {
        wined3d_device_context_finish(context, WINED3D_CS_QUEUE_DEFAULT);
    }
}

unsafe extern "C" fn wined3d_cs_exec_clear_sysmem_texture(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let op: *const wined3d_cs_clear_sysmem_texture = data as *const wined3d_cs_clear_sysmem_texture;
    let mut box_0: wined3d_box = wined3d_box {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        front: 0,
        back: 0,
    };
    box_0.left = (*op).rect.left as UINT;
    box_0.right = (*op).rect.right as UINT;
    box_0.top = (*op).rect.top as UINT;
    box_0.bottom = (*op).rect.bottom as UINT;
    box_0.front = 0;
    box_0.back = 1;
    cpu_blitter_clear_texture(
        (*op).texture,
        (*op).sub_resource_idx,
        core::ptr::addr_of_mut!(box_0),
        core::ptr::addr_of!((*op).color),
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_clear_sysmem_texture(
    mut context: *mut wined3d_device_context,
    mut texture: *mut wined3d_texture,
    mut sub_resource_idx: ::core::ffi::c_uint,
    mut rect: *const RECT,
    mut flags: ::core::ffi::c_uint,
    mut color: *const wined3d_color,
) -> HRESULT {
    let mut op: *mut wined3d_cs_clear_sysmem_texture =
        ::core::ptr::null_mut::<wined3d_cs_clear_sysmem_texture>();
    if !rect.is_null() {
        let mut b: wined3d_box = wined3d_box {
            left: (*rect).left as UINT,
            top: (*rect).top as UINT,
            right: (*rect).right as UINT,
            bottom: (*rect).bottom as UINT,
            front: 0 as UINT,
            back: 1 as UINT,
        };
        let mut hr: HRESULT = 0;
        hr = wined3d_resource_check_box_dimensions(
            &raw mut (*texture).resource,
            sub_resource_idx,
            &raw mut b,
        );
        if hr < 0 {
            return hr;
        }
    }
    wined3d_device_context_lock(context);
    op = wined3d_device_context_require_space(
        context,
        ::core::mem::size_of::<wined3d_cs_clear_sysmem_texture>() as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_clear_sysmem_texture;
    (*op).opcode = WINED3D_CS_OP_CLEAR_SYSMEM_TEXTURE;
    (*op).texture = texture;
    (*op).sub_resource_idx = sub_resource_idx;
    (*op).color = *color;
    if !rect.is_null() {
        (*op).rect = *rect;
    } else {
        let mut level_idx: ::core::ffi::c_uint =
            sub_resource_idx.wrapping_rem((*texture).level_count);
        (*op).rect.left = 0 as LONG;
        (*op).rect.top = 0 as LONG;
        (*op).rect.right = wined3d_texture_get_level_width(texture, level_idx) as LONG;
        (*op).rect.bottom = wined3d_texture_get_level_height(texture, level_idx) as LONG;
    }
    wined3d_device_context_reference_resource(context, &raw mut (*texture).resource);
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
    if (flags & WINED3DCLEAR_SYNCHRONOUS) != 0 {
        wined3d_device_context_finish(context, WINED3D_CS_QUEUE_DEFAULT);
    }
    wined3d_device_context_unlock(context);
    return S_OK;
}

unsafe extern "C" fn reference_shader_resources(
    mut context: *mut wined3d_device_context,
    mut shader_mask: ::core::ffi::c_uint,
) {
    let mut state: *const wined3d_state = (*context).state;
    let mut entry: *mut wined3d_shader_sampler_map_entry =
        ::core::ptr::null_mut::<wined3d_shader_sampler_map_entry>();
    let mut view: *mut wined3d_shader_resource_view =
        ::core::ptr::null_mut::<wined3d_shader_resource_view>();
    let mut shader: *mut wined3d_shader = ::core::ptr::null_mut::<wined3d_shader>();
    let mut i: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_uint = 0;
    i = 0 as ::core::ffi::c_int as ::core::ffi::c_uint;
    while i < WINED3D_SHADER_TYPE_COUNT as ::core::ffi::c_int as ::core::ffi::c_uint {
        'next: {
            if (shader_mask & ((1 as ::core::ffi::c_uint) << i)) == 0 {
                break 'next;
            }
            shader = (*state).shader[i as usize];
            if shader.is_null() {
                if i == WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int as ::core::ffi::c_uint {
                    j = 0 as ::core::ffi::c_int as ::core::ffi::c_uint;
                    while j < WINED3D_MAX_FFP_TEXTURES as ::core::ffi::c_int as ::core::ffi::c_uint {
                        view = (*state).shader_resource_view
                            [WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int as usize]
                            [j as usize];
                        if !view.is_null() {
                            wined3d_device_context_reference_resource(context, (*view).resource);
                        }
                        j = j.wrapping_add(1);
                    }
                }
                break 'next;
            }

            j = 0 as ::core::ffi::c_int as ::core::ffi::c_uint;
            while j < WINED3D_MAX_CBS as ::core::ffi::c_int as ::core::ffi::c_uint {
                if !(*state).cb[i as usize][j as usize].buffer.is_null() {
                    wined3d_device_context_reference_resource(
                        context,
                        core::ptr::addr_of_mut!(
                            (*(*state).cb[i as usize][j as usize].buffer).resource
                        ),
                    );
                }
                j = j.wrapping_add(1);
            }

            j = 0 as ::core::ffi::c_int as ::core::ffi::c_uint;
            while (j as usize) < (*shader).reg_maps.sampler_map.count {
                entry = (*shader).reg_maps.sampler_map.entries.offset(j as isize)
                    as *mut wined3d_shader_sampler_map_entry;
                view = (*state).shader_resource_view[i as usize][(*entry).resource_idx as usize];
                if view.is_null() {
                    j = j.wrapping_add(1);
                    continue;
                }

                wined3d_device_context_reference_resource(context, (*view).resource);
                j = j.wrapping_add(1);
            }
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn reference_unordered_access_resources(
    context: *mut wined3d_device_context,
    shader: *const wined3d_shader,
    views: *const *mut wined3d_unordered_access_view,
) {
    let mut i: ::core::ffi::c_uint;

    if shader.is_null() {
        return;
    }

    i = 0;
    while i < MAX_UNORDERED_ACCESS_VIEWS {
        if !((*shader).reg_maps.uav_resource_info[i as usize].type_ as u64 == 0) {
            if (*views.offset(i as isize)).is_null() {
                i = i.wrapping_add(1);
                continue;
            }
            wined3d_device_context_reference_resource(context, (**views.offset(i as isize)).resource);
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn wined3d_cs_exec_dispatch(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let op: *const wined3d_cs_dispatch = data as *const wined3d_cs_dispatch;
    let state: *mut wined3d_state = core::ptr::addr_of_mut!((*cs).state);
    if (*state).shader[WINED3D_SHADER_TYPE_COMPUTE as usize].is_null() {
        WARN!("No compute shader bound, skipping dispatch.\n");
    } else {
        (*(*(*(*cs).c.device).adapter).adapter_ops)
            .adapter_dispatch_compute
            .unwrap_unchecked()(
            (*cs).c.device,
            state,
            core::ptr::addr_of!((*op).parameters),
        );
    }
}

unsafe extern "C" fn reference_compute_pipeline_resources(context: *mut wined3d_device_context) {
    let state: *const wined3d_state = (*context).state;

    reference_shader_resources(
        context,
        (1 as ::core::ffi::c_uint) << (WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_uint),
    );
    reference_unordered_access_resources(
        context,
        (*state).shader[WINED3D_SHADER_TYPE_COMPUTE as usize],
        &raw const *(&raw const (*state).unordered_access_view
            as *const [*mut wined3d_unordered_access_view; 8])
            .offset(WINED3D_PIPELINE_COMPUTE as ::core::ffi::c_int as isize)
            as *const *mut wined3d_unordered_access_view,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_dispatch(
    mut context: *mut wined3d_device_context,
    mut group_count_x: ::core::ffi::c_uint,
    mut group_count_y: ::core::ffi::c_uint,
    mut group_count_z: ::core::ffi::c_uint,
) {
    let mut op: *mut wined3d_cs_dispatch = ::core::ptr::null_mut::<wined3d_cs_dispatch>();
    wined3d_device_context_lock(context);
    op = wined3d_device_context_require_space(
        context,
        core::mem::size_of::<wined3d_cs_dispatch>() as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_dispatch;
    (*op).opcode = WINED3D_CS_OP_DISPATCH;
    (*op).parameters.indirect = (FALSE as i32);
    (*op).parameters.u.direct.group_count_x = group_count_x;
    (*op).parameters.u.direct.group_count_y = group_count_y;
    (*op).parameters.u.direct.group_count_z = group_count_z;
    reference_compute_pipeline_resources(context);
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
    wined3d_device_context_unlock(context);
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_dispatch_indirect(
    context: *mut wined3d_device_context,
    buffer: *mut wined3d_buffer,
    offset: ::core::ffi::c_uint,
) {
    let mut op: *mut wined3d_cs_dispatch = core::ptr::null_mut::<wined3d_cs_dispatch>();

    wined3d_device_context_lock(context);
    op = wined3d_device_context_require_space(
        context,
        core::mem::size_of::<wined3d_cs_dispatch>() as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_dispatch;
    (*op).opcode = WINED3D_CS_OP_DISPATCH;
    (*op).parameters.indirect = TRUE as BOOL;
    (*op).parameters.u.indirect.buffer = buffer;
    (*op).parameters.u.indirect.offset = offset;

    reference_compute_pipeline_resources(context);
    wined3d_device_context_reference_resource(context, core::ptr::addr_of_mut!((*buffer).resource));

    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
    wined3d_device_context_unlock(context);
}

unsafe extern "C" fn wined3d_cs_exec_draw(cs: *mut wined3d_cs, data: *const ::core::ffi::c_void) {
    let d3d_info: *const wined3d_d3d_info = core::ptr::addr_of!((*(*(*cs).c.device).adapter).d3d_info);
    let mut geometry_shader: *const wined3d_shader = core::ptr::null::<wined3d_shader>();
    let device: *mut wined3d_device = (*cs).c.device;
    let mut base_vertex_idx: ::core::ffi::c_int = 0;
    let mut load_base_vertex_idx: ::core::ffi::c_int = 0;
    let state: *mut wined3d_state = core::ptr::addr_of_mut!((*cs).state);
    let op: *const wined3d_cs_draw = data as *const wined3d_cs_draw;
    let mut i: ::core::ffi::c_uint = 0;

    base_vertex_idx = 0;
    if (*op).parameters.indirect == 0 {
        let direct: *const wined3d_direct_draw_parameters = core::ptr::addr_of!((*op).parameters.u.direct);

        if (*op).parameters.indexed != 0 && (*d3d_info).draw_base_vertex_offset() as ::core::ffi::c_int != 0 {
            base_vertex_idx = (*direct).base_vertex_idx;
        } else if (*op).parameters.indexed == 0 {
            base_vertex_idx = (*direct).start_idx as ::core::ffi::c_int;
        }
    }

    /* ARB_draw_indirect always supports a base vertex offset. */
    if (*op).parameters.indirect == 0 && (*d3d_info).draw_base_vertex_offset() == 0 {
        load_base_vertex_idx = (*op).parameters.u.direct.base_vertex_idx;
    } else {
        load_base_vertex_idx = 0;
    }

    if (*state).base_vertex_index != base_vertex_idx {
        (*state).base_vertex_index = base_vertex_idx;
        i = 0;
        while i < (*device).context_count {
            (**(*device).contexts.offset(i as isize)).constant_update_mask |= WINED3D_SHADER_CONST_BASE_VERTEX_ID as DWORD;
            i = i.wrapping_add(1);
        }
    }

    if (*state).load_base_vertex_index != load_base_vertex_idx {
        (*state).load_base_vertex_index = load_base_vertex_idx;
        device_invalidate_state((*cs).c.device, (STATE_BASEVERTEXINDEX as u32));
    }

    if (*state).primitive_type as ::core::ffi::c_uint != (*op).primitive_type as ::core::ffi::c_uint {
        geometry_shader = (*state).shader[WINED3D_SHADER_TYPE_GEOMETRY as usize];
        if !geometry_shader.is_null() && (*geometry_shader).function.is_null() {
            device_invalidate_state((*cs).c.device, STATE_SHADER!((WINED3D_SHADER_TYPE_GEOMETRY) as i32));
        }
        if (*state).primitive_type as ::core::ffi::c_uint == WINED3D_PT_POINTLIST as ::core::ffi::c_uint
            || (*op).primitive_type as ::core::ffi::c_uint == WINED3D_PT_POINTLIST as ::core::ffi::c_uint
        {
            device_invalidate_state((*cs).c.device, STATE_SHADER!((WINED3D_SHADER_TYPE_VERTEX) as i32));
            device_invalidate_state((*cs).c.device, STATE_SHADER!((WINED3D_SHADER_TYPE_PIXEL) as i32));
        }
        (*state).primitive_type = (*op).primitive_type;
        i = 0;
        while i < (*device).context_count {
            (**(*device).contexts.offset(i as isize)).set_update_primitive_type((1) as _);
            i = i.wrapping_add(1);
        }
    }

    if (*state).patch_vertex_count != (*op).patch_vertex_count {
        i = 0;
        while i < (*device).context_count {
            (**(*device).contexts.offset(i as isize)).set_update_patch_vertex_count((1) as _);
            i = i.wrapping_add(1);
        }
    }
    (*state).patch_vertex_count = (*op).patch_vertex_count;

    (*(*(*(*cs).c.device).adapter).adapter_ops)
        .adapter_draw_primitive
        .unwrap_unchecked()((*cs).c.device, state, core::ptr::addr_of!((*op).parameters));
}

unsafe extern "C" fn reference_graphics_pipeline_resources(
    context: *mut wined3d_device_context,
    indexed: BOOL,
    d3d_info: *const wined3d_d3d_info,
) {
    let state: *const wined3d_state = (*context).state;
    let mut i: ::core::ffi::c_uint;

    if indexed != 0 {
        wined3d_device_context_reference_resource(context, &raw mut (*(*state).index_buffer).resource);
    }
    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!((*state).streams) as usize {
        if !(*state).streams[i as usize].buffer.is_null() {
            wined3d_device_context_reference_resource(
                context,
                &raw mut (*(*state).streams[i as usize].buffer).resource,
            );
        }
        i = i.wrapping_add(1);
    }
    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!((*state).stream_output) as usize {
        if !(*state).stream_output[i as usize].buffer.is_null() {
            wined3d_device_context_reference_resource(
                context,
                &raw mut (*(*state).stream_output[i as usize].buffer).resource,
            );
        }
        i = i.wrapping_add(1);
    }
    i = 0 as ::core::ffi::c_uint;
    while i < (*d3d_info).limits.max_rt_count {
        if !(*state).fb.render_targets[i as usize].is_null() {
            wined3d_device_context_reference_resource(
                context,
                (*(*state).fb.render_targets[i as usize]).resource,
            );
        }
        i = i.wrapping_add(1);
    }
    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < (WINED3D_PUSH_CONSTANTS_COUNT as usize) {
        if !(*(*context).device).push_constants[i as usize].is_null() {
            wined3d_device_context_reference_resource(
                context,
                &raw mut (*(*(*context).device).push_constants[i as usize]).resource,
            );
        }
        i = i.wrapping_add(1);
    }
    if !(*state).fb.depth_stencil.is_null() {
        wined3d_device_context_reference_resource(context, (*(*state).fb.depth_stencil).resource);
    }
    reference_shader_resources(
        context,
        !((1 as ::core::ffi::c_uint) << WINED3D_SHADER_TYPE_COMPUTE),
    );
    reference_unordered_access_resources(
        context,
        (*state).shader[WINED3D_SHADER_TYPE_PIXEL as usize],
        &raw const (*state).unordered_access_view[WINED3D_PIPELINE_GRAPHICS as usize]
            as *const *mut wined3d_unordered_access_view,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_draw(context: *mut wined3d_device_context, primitive_type: wined3d_primitive_type, patch_vertex_count: ::core::ffi::c_uint, base_vertex_idx: ::core::ffi::c_int, start_idx: ::core::ffi::c_uint, index_count: ::core::ffi::c_uint, start_instance: ::core::ffi::c_uint, instance_count: ::core::ffi::c_uint, indexed: bool) {
    let d3d_info: *const wined3d_d3d_info = core::ptr::addr_of!((*(*(*context).device).adapter).d3d_info);
    let mut op: *mut wined3d_cs_draw = core::ptr::null_mut::<wined3d_cs_draw>();

    op = wined3d_device_context_require_space(
        context,
        core::mem::size_of::<wined3d_cs_draw>() as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_draw;
    (*op).opcode = WINED3D_CS_OP_DRAW;
    (*op).primitive_type = primitive_type;
    (*op).patch_vertex_count = patch_vertex_count;
    (*op).parameters.indirect = (FALSE as i32);
    (*op).parameters.u.direct.base_vertex_idx = base_vertex_idx;
    (*op).parameters.u.direct.start_idx = start_idx;
    (*op).parameters.u.direct.index_count = index_count;
    (*op).parameters.u.direct.start_instance = start_instance;
    (*op).parameters.u.direct.instance_count = instance_count;
    (*op).parameters.indexed = indexed as BOOL;

    reference_graphics_pipeline_resources(context, indexed as BOOL, d3d_info);

    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_draw_indirect(
    mut context: *mut wined3d_device_context,
    mut buffer: *mut wined3d_buffer,
    mut offset: ::core::ffi::c_uint,
    mut indexed: bool,
) {
    let mut d3d_info: *const wined3d_d3d_info = &raw mut (*(*(*context).device).adapter).d3d_info;
    let mut state: *const wined3d_state = (*context).state;
    let mut op: *mut wined3d_cs_draw = core::ptr::null_mut::<wined3d_cs_draw>();
    wined3d_device_context_lock(context);
    op = wined3d_device_context_require_space(
        context,
        core::mem::size_of::<wined3d_cs_draw>() as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_draw;
    (*op).opcode = WINED3D_CS_OP_DRAW;
    (*op).primitive_type = (*state).primitive_type;
    (*op).patch_vertex_count = (*state).patch_vertex_count;
    (*op).parameters.indirect = TRUE as BOOL;
    (*op).parameters.u.indirect.buffer = buffer;
    (*op).parameters.u.indirect.offset = offset;
    (*op).parameters.indexed = indexed as BOOL;
    reference_graphics_pipeline_resources(context, indexed as BOOL, d3d_info);
    wined3d_device_context_reference_resource(context, &raw mut (*buffer).resource);
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
    wined3d_device_context_unlock(context);
}

unsafe extern "C" fn wined3d_cs_exec_flush(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let mut context: *mut wined3d_context = core::ptr::null_mut();
    WINE_TRACE_!(__wine_dbch_d3d_perf,"Flushing adapter.\n");
    context = context_acquire((*cs).c.device, core::ptr::null_mut(), 0);
    (((*(*(*(*cs).c.device).adapter).adapter_ops).adapter_flush_context).unwrap_unchecked())(context);
    context_release(context);
}

unsafe extern "C" fn wined3d_cs_flush(mut context: *mut wined3d_device_context) {
    let mut cs: *mut wined3d_cs = wined3d_cs_from_context(context);
    let mut op: *mut wined3d_cs_flush = core::ptr::null_mut::<wined3d_cs_flush>();
    op = wined3d_device_context_require_space(
        context,
        core::mem::size_of::<wined3d_cs_flush>() as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_flush;
    (*op).opcode = WINED3D_CS_OP_FLUSH;
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
    (*cs).queries_flushed = (TRUE as i32);
}

unsafe extern "C" fn wined3d_cs_exec_set_predication(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let op: *const wined3d_cs_set_predication = data as *const wined3d_cs_set_predication;

    (*cs).state.predicate = (*op).predicate;
    (*cs).state.predicate_value = (*op).value;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_predication(context: *mut wined3d_device_context, predicate: *mut wined3d_query, value: BOOL) {
    let mut op: *mut wined3d_cs_set_predication = core::ptr::null_mut::<wined3d_cs_set_predication>();

    op = wined3d_device_context_require_space(
        context,
        core::mem::size_of::<wined3d_cs_set_predication>(),
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_set_predication;
    (*op).opcode = WINED3D_CS_OP_SET_PREDICATION;
    (*op).predicate = predicate;
    (*op).value = value;

    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_viewports(cs: *mut wined3d_cs, data: *const ::core::ffi::c_void) {
    let op: *const wined3d_cs_set_viewports = data as *const wined3d_cs_set_viewports;
    if (*op).viewport_count != 0 {
        memcpy(
            core::ptr::addr_of_mut!((*cs).state.viewports) as *mut ::core::ffi::c_void,
            core::ptr::addr_of!((*op).viewports) as *const ::core::ffi::c_void,
            ((*op).viewport_count as usize).wrapping_mul(core::mem::size_of::<wined3d_viewport>()),
        );
    } else {
        memset(
            core::ptr::addr_of_mut!((*cs).state.viewports) as *mut ::core::ffi::c_void,
            (0) as ::core::ffi::c_int,
            core::mem::size_of::<wined3d_viewport>(),
        );
    }
    (*cs).state.viewport_count = (*op).viewport_count;
    device_invalidate_state(
        (*cs).c.device,
        (STATE_VIEWPORT) as ::core::ffi::c_uint,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_viewports(mut context: *mut wined3d_device_context, mut viewport_count: ::core::ffi::c_uint, mut viewports: *const wined3d_viewport) {
    let mut op: *mut wined3d_cs_set_viewports = ::core::ptr::null_mut::<wined3d_cs_set_viewports>();
    op = wined3d_device_context_require_space(
        context,
        (FIELD_OFFSET!(wined3d_cs_set_viewports, viewports[viewport_count as usize]) as usize) as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_set_viewports;
    (*op).opcode = WINED3D_CS_OP_SET_VIEWPORTS;
    memcpy(
        core::ptr::addr_of_mut!((*op).viewports) as *mut ::core::ffi::c_void,
        viewports as *const ::core::ffi::c_void,
        (viewport_count as size_t).wrapping_mul(::core::mem::size_of::<wined3d_viewport>() as size_t),
    );
    (*op).viewport_count = viewport_count;

    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_scissor_rects(mut cs: *mut wined3d_cs, mut data: *const ::core::ffi::c_void) {
    let mut op: *const wined3d_cs_set_scissor_rects = data as *const wined3d_cs_set_scissor_rects;
    if (*op).rect_count != 0 {
        memcpy(
            core::ptr::addr_of_mut!((*cs).state.scissor_rects) as *mut ::core::ffi::c_void,
            core::ptr::addr_of!((*op).rects) as *const ::core::ffi::c_void,
            ((*op).rect_count as usize).wrapping_mul(core::mem::size_of::<RECT>()),
        );
    } else {
        SetRectEmpty(core::ptr::addr_of_mut!((*cs).state.scissor_rects) as LPRECT);
    }
    (*cs).state.scissor_rect_count = (*op).rect_count;
    device_invalidate_state((*cs).c.device, (STATE_SCISSORRECT as u32));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_scissor_rects(mut context: *mut wined3d_device_context, mut rect_count: ::core::ffi::c_uint, mut rects: *const RECT) {
    let mut op: *mut wined3d_cs_set_scissor_rects = ::core::ptr::null_mut::<wined3d_cs_set_scissor_rects>();
    op = wined3d_device_context_require_space(
        context,
        core::mem::offset_of!(wined3d_cs_set_scissor_rects, rects) + (rect_count as usize) * core::mem::size_of::<RECT>(),
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_set_scissor_rects;
    (*op).opcode = WINED3D_CS_OP_SET_SCISSOR_RECTS;
    memcpy(
        core::ptr::addr_of_mut!((*op).rects) as *mut RECT as *mut ::core::ffi::c_void,
        rects as *const ::core::ffi::c_void,
        (rect_count as usize) * core::mem::size_of::<RECT>(),
    );
    (*op).rect_count = rect_count;

    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_rendertarget_views(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let op: *const wined3d_cs_set_rendertarget_views = data as *const wined3d_cs_set_rendertarget_views;
    let device: *mut wined3d_device = (*cs).c.device;
    let mut i: core::ffi::c_uint = 0;

    while i < (*op).count {
        let prev: *mut wined3d_rendertarget_view = (*cs).state.fb.render_targets[((*op).start_idx.wrapping_add(i)) as usize];
        let view: *mut wined3d_rendertarget_view = *(&raw const (*op).views as *const *mut wined3d_rendertarget_view).offset(i as isize);
        let prev_alpha_swizzle: bool;
        let curr_alpha_swizzle: bool;
        let prev_srgb_write: bool;
        let curr_srgb_write: bool;

        (*cs).state.fb.render_targets[((*op).start_idx.wrapping_add(i)) as usize] = view;

        prev_alpha_swizzle = !prev.is_null() && (*(*prev).format).id == WINED3DFMT_A8_UNORM;
        curr_alpha_swizzle = !view.is_null() && (*(*view).format).id == WINED3DFMT_A8_UNORM;
        if prev_alpha_swizzle != curr_alpha_swizzle {
            device_invalidate_state(device, STATE_SHADER!((WINED3D_SHADER_TYPE_PIXEL) as i32));
        }

        if ((((*(*device).adapter).d3d_info.wined3d_creation_flags) & WINED3D_SRGB_READ_WRITE_CONTROL) == 0)
            || (*cs).state.render_states[(WINED3D_RS_SRGBWRITEENABLE) as usize] != 0
        {
            prev_srgb_write = !prev.is_null() && ((*prev).format_caps & WINED3D_FORMAT_CAP_SRGB_WRITE) != 0;
            curr_srgb_write = !view.is_null() && ((*view).format_caps & WINED3D_FORMAT_CAP_SRGB_WRITE) != 0;
            if prev_srgb_write != curr_srgb_write {
                device_invalidate_state(device, STATE_RENDER!(WINED3D_RS_SRGBWRITEENABLE));
            }
        }

        i = i.wrapping_add(1);
    }

    device_invalidate_state(device, (STATE_FRAMEBUFFER as u32));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_rendertarget_views(mut context: *mut wined3d_device_context, mut start_idx: ::core::ffi::c_uint, mut count: ::core::ffi::c_uint, mut views: *const *mut wined3d_rendertarget_view) {
    let mut op: *mut wined3d_cs_set_rendertarget_views = core::ptr::null_mut();
    op = wined3d_device_context_require_space(
        context,
        core::mem::offset_of!(wined3d_cs_set_rendertarget_views, views)
            + (count as usize) * core::mem::size_of::<*mut wined3d_rendertarget_view>(),
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_set_rendertarget_views;
    (*op).opcode = WINED3D_CS_OP_SET_RENDERTARGET_VIEWS;
    (*op).start_idx = start_idx;
    (*op).count = count;
    memcpy(
        core::ptr::addr_of_mut!((*op).views) as *mut ::core::ffi::c_void,
        views as *const ::core::ffi::c_void,
        (count as usize) * core::mem::size_of::<*mut wined3d_rendertarget_view>(),
    );
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_depth_stencil_view(
    mut cs: *mut wined3d_cs,
    mut data: *const ::core::ffi::c_void,
) {
    let mut op: *const wined3d_cs_set_depth_stencil_view =
        data as *const wined3d_cs_set_depth_stencil_view;
    let mut device: *mut wined3d_device = (*cs).c.device;
    let mut prev: *mut wined3d_rendertarget_view =
        ::core::ptr::null_mut::<wined3d_rendertarget_view>();
    prev = (*cs).state.fb.depth_stencil;
    if !prev.is_null()
        && (*(*prev).resource).type_ as ::core::ffi::c_uint
            != WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint
    {
        let mut prev_texture: *mut wined3d_texture =
            texture_from_resource((*prev).resource);
        if (**(*device)
            .swapchains
            .offset(0 as isize))
        .state
        .desc
        .flags
            & WINED3D_SWAPCHAIN_DISCARD_DEPTHSTENCIL as uint32_t
            != 0
            || (*prev_texture).flags & WINED3D_TEXTURE_DISCARD as uint32_t != 0
        {
            wined3d_texture_validate_location(
                prev_texture,
                (*prev).sub_resource_idx,
                WINED3D_LOCATION_DISCARDED as uint32_t,
            );
        }
    }
    (*cs).state.fb.depth_stencil = (*op).view;
    if prev.is_null() as ::core::ffi::c_int != (*op).view.is_null() as ::core::ffi::c_int {
        device_invalidate_state(device, STATE_DEPTH_STENCIL as ::core::ffi::c_uint);
        device_invalidate_state(device, STATE_STENCIL_REF as ::core::ffi::c_uint);
        device_invalidate_state(device, STATE_RASTERIZER as ::core::ffi::c_uint);
    } else if !prev.is_null() {
        if (*(*prev).format).depth_bias_scale != (*(*(*op).view).format).depth_bias_scale {
            device_invalidate_state(device, STATE_RASTERIZER as ::core::ffi::c_uint);
        }
        if (*(*prev).format).stencil_size as ::core::ffi::c_int
            != (*(*(*op).view).format).stencil_size as ::core::ffi::c_int
        {
            device_invalidate_state(device, STATE_STENCIL_REF as ::core::ffi::c_uint);
        }
    }
    device_invalidate_state(device, STATE_FRAMEBUFFER as ::core::ffi::c_uint);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_depth_stencil_view(context: *mut wined3d_device_context, view: *mut wined3d_rendertarget_view) {
    let mut op: *mut wined3d_cs_set_depth_stencil_view = core::ptr::null_mut();

    op = wined3d_device_context_require_space(
        context,
        core::mem::size_of::<wined3d_cs_set_depth_stencil_view>(),
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_set_depth_stencil_view;
    (*op).opcode = WINED3D_CS_OP_SET_DEPTH_STENCIL_VIEW;
    (*op).view = view;

    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_vertex_declaration(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let op: *const wined3d_cs_set_vertex_declaration = data as *const wined3d_cs_set_vertex_declaration;

    (*cs).state.vertex_declaration = (*op).declaration;
    device_invalidate_state((*cs).c.device, (STATE_VDECL as u32));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_vertex_declaration(context: *mut wined3d_device_context, declaration: *mut wined3d_vertex_declaration) {
    let mut op: *mut wined3d_cs_set_vertex_declaration = core::ptr::null_mut();

    op = wined3d_device_context_require_space(context, core::mem::size_of::<wined3d_cs_set_vertex_declaration>(), WINED3D_CS_QUEUE_DEFAULT) as *mut wined3d_cs_set_vertex_declaration;
    (*op).opcode = WINED3D_CS_OP_SET_VERTEX_DECLARATION;
    (*op).declaration = declaration;

    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_stream_sources(
    mut cs: *mut wined3d_cs,
    mut data: *const ::core::ffi::c_void,
) {
    let mut op: *const wined3d_cs_set_stream_sources = data as *const wined3d_cs_set_stream_sources;
    let mut i: ::core::ffi::c_uint = 0;
    i = 0 as ::core::ffi::c_uint;
    while i < (*op).count {
        let mut prev: *mut wined3d_buffer = (*cs).state.streams
            [(*op).start_idx.wrapping_add(i) as usize]
            .buffer as *mut wined3d_buffer;
        let mut buffer: *mut wined3d_buffer =
            (*(&raw const (*op).streams as *const wined3d_stream_state).offset(i as isize)).buffer
                as *mut wined3d_buffer;
        if !buffer.is_null() {
            (*buffer).resource.bind_count += 1;
        }
        if !prev.is_null() {
            (*prev).resource.bind_count -= 1;
        }
        i = i.wrapping_add(1);
    }
    memcpy(
        (&raw mut (*cs).state.streams as *mut wined3d_stream_state).offset((*op).start_idx as isize)
            as *mut wined3d_stream_state as *mut ::core::ffi::c_void,
        &raw const (*op).streams as *const wined3d_stream_state as *const ::core::ffi::c_void,
        ((*op).count as size_t)
            .wrapping_mul(::core::mem::size_of::<wined3d_stream_state>() as size_t),
    );
    device_invalidate_state(
        (*cs).c.device,
        STATE_STREAMSRC as ::core::ffi::c_uint,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_stream_sources(context: *mut wined3d_device_context, start_idx: ::core::ffi::c_uint, count: ::core::ffi::c_uint, streams: *const wined3d_stream_state) {
    let mut op: *mut wined3d_cs_set_stream_sources =
        ::core::ptr::null_mut::<wined3d_cs_set_stream_sources>();
    op = wined3d_device_context_require_space(
        context,
        (FIELD_OFFSET!(wined3d_cs_set_stream_sources, streams[count as usize]) as usize) as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_set_stream_sources;
    (*op).opcode = WINED3D_CS_OP_SET_STREAM_SOURCES;
    (*op).start_idx = start_idx;
    (*op).count = count;
    memcpy(
        &raw mut (*op).streams as *mut wined3d_stream_state as *mut ::core::ffi::c_void,
        streams as *const ::core::ffi::c_void,
        (count as size_t).wrapping_mul(::core::mem::size_of::<wined3d_stream_state>() as size_t),
    );
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_stream_outputs(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let op: *const wined3d_cs_set_stream_outputs = data as *const wined3d_cs_set_stream_outputs;
    let mut i: core::ffi::c_uint = 0;
    i = 0 as core::ffi::c_uint;
    while i < WINED3D_MAX_STREAM_OUTPUT_BUFFERS as core::ffi::c_uint {
        let prev: *mut wined3d_buffer = (*cs).state.stream_output[i as usize].buffer;
        let buffer: *mut wined3d_buffer = (*op).outputs[i as usize].buffer;
        if !buffer.is_null() {
            (*buffer).resource.bind_count += 1;
        }
        if !prev.is_null() {
            (*prev).resource.bind_count -= 1;
        }
        i = i.wrapping_add(1);
    }
    memcpy(
        core::ptr::addr_of_mut!((*cs).state.stream_output) as *mut core::ffi::c_void,
        core::ptr::addr_of!((*op).outputs) as *const core::ffi::c_void,
        core::mem::size_of::<[wined3d_stream_output; 4]>() as usize,
    );
    device_invalidate_state((*cs).c.device, STATE_STREAM_OUTPUT as core::ffi::c_uint);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_stream_outputs(mut context: *mut wined3d_device_context, mut outputs: *const wined3d_stream_output) {
    let mut op: *mut wined3d_cs_set_stream_outputs = core::ptr::null_mut::<wined3d_cs_set_stream_outputs>();

    op = wined3d_device_context_require_space(
        context,
        core::mem::size_of::<wined3d_cs_set_stream_outputs>(),
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_set_stream_outputs;
    (*op).opcode = WINED3D_CS_OP_SET_STREAM_OUTPUTS;
    memcpy(
        core::ptr::addr_of_mut!((*op).outputs) as *mut ::core::ffi::c_void,
        outputs as *const ::core::ffi::c_void,
        core::mem::size_of::<[wined3d_stream_output; 4]>(),
    );

    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_index_buffer(cs: *mut wined3d_cs, data: *const ::core::ffi::c_void) {
    let op: *const wined3d_cs_set_index_buffer = data as *const wined3d_cs_set_index_buffer;
    let mut prev: *mut wined3d_buffer = ::core::ptr::null_mut::<wined3d_buffer>();

    prev = (*cs).state.index_buffer;
    (*cs).state.index_buffer = (*op).buffer;
    (*cs).state.index_format = (*op).format_id;
    (*cs).state.index_offset = (*op).offset;

    if !(*op).buffer.is_null() {
        (*(*op).buffer).resource.bind_count += 1;
    }
    if !prev.is_null() {
        (*prev).resource.bind_count -= 1;
    }

    device_invalidate_state((*cs).c.device, (STATE_INDEXBUFFER) as ::core::ffi::c_uint);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_index_buffer(context: *mut wined3d_device_context, buffer: *mut wined3d_buffer, format_id: wined3d_format_id, offset: ::core::ffi::c_uint) {
    let mut op: *mut wined3d_cs_set_index_buffer = core::ptr::null_mut();

    op = wined3d_device_context_require_space(context, core::mem::size_of::<wined3d_cs_set_index_buffer>(), WINED3D_CS_QUEUE_DEFAULT) as *mut wined3d_cs_set_index_buffer;
    (*op).opcode = WINED3D_CS_OP_SET_INDEX_BUFFER;
    (*op).buffer = buffer;
    (*op).format_id = format_id;
    (*op).offset = offset;

    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_constant_buffers(
    cs: *mut wined3d_cs,
    data: *const core::ffi::c_void,
) {
    let op: *const wined3d_cs_set_constant_buffers = data as *const wined3d_cs_set_constant_buffers;
    let mut i: core::ffi::c_uint = 0;
    while i < (*op).count {
        let prev: *mut wined3d_buffer =
            (*cs).state.cb[(*op).type_ as usize][(*op).start_idx.wrapping_add(i) as usize].buffer;
        let buffer: *mut wined3d_buffer = (*(core::ptr::addr_of!((*op).buffers)
            as *const wined3d_constant_buffer_state)
            .offset(i as isize))
        .buffer;

        (*cs).state.cb[(*op).type_ as usize][(*op).start_idx.wrapping_add(i) as usize] =
            *(core::ptr::addr_of!((*op).buffers) as *const wined3d_constant_buffer_state)
                .offset(i as isize);

        if !buffer.is_null() {
            (*buffer).resource.bind_count += 1;
        }
        if !prev.is_null() {
            (*prev).resource.bind_count -= 1;
        }
        i = i.wrapping_add(1);
    }
    device_invalidate_state(
        (*cs).c.device,
        STATE_CONSTANT_BUFFER!(((*op).type_) as i32),
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_constant_buffers(context: *mut wined3d_device_context, type_0: wined3d_shader_type, start_idx: ::core::ffi::c_uint, count: ::core::ffi::c_uint, buffers: *const wined3d_constant_buffer_state) {
    let mut op: *mut wined3d_cs_set_constant_buffers = core::ptr::null_mut::<wined3d_cs_set_constant_buffers>();
    op = wined3d_device_context_require_space(
        context,
        (core::mem::offset_of!(wined3d_cs_set_constant_buffers, buffers) as usize)
            .wrapping_add((count as usize).wrapping_mul(core::mem::size_of::<wined3d_constant_buffer_state>())),
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_set_constant_buffers;
    (*op).opcode = WINED3D_CS_OP_SET_CONSTANT_BUFFERS;
    (*op).type_ = type_0;
    (*op).start_idx = start_idx;
    (*op).count = count;
    memcpy(
        core::ptr::addr_of_mut!((*op).buffers) as *mut ::core::ffi::c_void,
        buffers as *const ::core::ffi::c_void,
        (count as usize).wrapping_mul(core::mem::size_of::<wined3d_constant_buffer_state>()),
    );
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn texture_binding_might_invalidate_ps(
    view: *mut wined3d_shader_resource_view,
    prev: *mut wined3d_shader_resource_view,
    d3d_info: *const wined3d_d3d_info,
) -> bool {
    let mut old_usage: core::ffi::c_uint;
    let mut new_usage: core::ffi::c_uint;
    let mut old_caps: core::ffi::c_uint;
    let mut new_caps: core::ffi::c_uint;
    let mut old_format: *const wined3d_format;
    let mut new_format: *const wined3d_format;

    if prev.is_null() {
        return true;
    }

    /* 1.x pixel shaders need to be recompiled based on the resource type. */
    old_usage = (*(*prev).resource).usage as core::ffi::c_uint;
    new_usage = (*(*view).resource).usage as core::ffi::c_uint;
    if (*(*view).resource).type_ as core::ffi::c_uint
        != (*(*prev).resource).type_ as core::ffi::c_uint
        || (old_usage & WINED3DUSAGE_LEGACY_CUBEMAP) != (new_usage & WINED3DUSAGE_LEGACY_CUBEMAP)
    {
        return true;
    }

    old_format = (*(*prev).resource).format;
    new_format = (*(*view).resource).format;
    old_caps = (*(*prev).resource).format_caps;
    new_caps = (*(*view).resource).format_caps;
    if (old_caps & WINED3D_FORMAT_CAP_SHADOW) != (new_caps & WINED3D_FORMAT_CAP_SHADOW) {
        return true;
    }

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

unsafe extern "C" fn wined3d_cs_exec_set_texture(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let d3d_info: *const wined3d_d3d_info = core::ptr::addr_of!((*(*(*cs).c.device).adapter).d3d_info);
    let op: *const wined3d_cs_set_texture = data as *const wined3d_cs_set_texture;
    let mut prev: *mut wined3d_shader_resource_view = core::ptr::null_mut();

    prev = (*cs).state.shader_resource_view[(*op).shader_type as usize][(*op).bind_index as usize];
    (*cs).state.shader_resource_view[(*op).shader_type as usize][(*op).bind_index as usize] =
        (*op).view;

    if !(*op).view.is_null() {
        (*(*(*op).view).resource).bind_count += 1;

        if (*op).shader_type as u32 == WINED3D_SHADER_TYPE_PIXEL {
            if texture_binding_might_invalidate_ps((*op).view, prev, d3d_info) {
                device_invalidate_state((*cs).c.device, STATE_SHADER!((WINED3D_SHADER_TYPE_PIXEL) as i32));
            }
        }
    }

    if !prev.is_null() {
        (*(*prev).resource).bind_count -= 1;
    }

    device_invalidate_state((*cs).c.device, (STATE_GRAPHICS_SHADER_RESOURCE_BINDING as u32));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_texture(context: *mut wined3d_device_context, shader_type: wined3d_shader_type, bind_index: ::core::ffi::c_uint, view: *mut wined3d_shader_resource_view) {
    let mut op: *mut wined3d_cs_set_texture = ::core::ptr::null_mut::<wined3d_cs_set_texture>();
    op = wined3d_device_context_require_space(
        context,
        core::mem::size_of::<wined3d_cs_set_texture>(),
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_set_texture;
    (*op).opcode = WINED3D_CS_OP_SET_TEXTURE;
    (*op).shader_type = shader_type;
    (*op).bind_index = bind_index;
    (*op).view = view;

    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_shader_resource_views(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let op: *const wined3d_cs_set_shader_resource_views = data as *const wined3d_cs_set_shader_resource_views;
    let mut i: ::core::ffi::c_uint = 0;
    while i < (*op).count {
        let prev: *mut wined3d_shader_resource_view = (*cs).state.shader_resource_view
            [(*op).type_ as usize][(*op).start_idx.wrapping_add(i) as usize];
        let view: *mut wined3d_shader_resource_view = *(&raw const (*op).views
            as *const *mut wined3d_shader_resource_view)
            .offset(i as isize);

        (*cs).state.shader_resource_view[(*op).type_ as usize]
            [(*op).start_idx.wrapping_add(i) as usize] = view;

        if !view.is_null() {
            (*(*view).resource).bind_count += 1;
        }
        if !prev.is_null() {
            (*(*prev).resource).bind_count -= 1;
        }
        i = i.wrapping_add(1);
    }

    if (*op).type_ as ::core::ffi::c_uint != WINED3D_SHADER_TYPE_COMPUTE {
        device_invalidate_state((*cs).c.device, (STATE_GRAPHICS_SHADER_RESOURCE_BINDING as u32));
    } else {
        device_invalidate_state((*cs).c.device, (STATE_COMPUTE_SHADER_RESOURCE_BINDING as u32));
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_shader_resource_views(context: *mut wined3d_device_context, type_0: wined3d_shader_type, start_idx: ::core::ffi::c_uint, count: ::core::ffi::c_uint, views: *const *mut wined3d_shader_resource_view) {
    let mut op: *mut wined3d_cs_set_shader_resource_views;

    op = wined3d_device_context_require_space(
        context,
        core::mem::offset_of!(wined3d_cs_set_shader_resource_views, views)
            + (count as usize) * core::mem::size_of::<*mut wined3d_shader_resource_view>(),
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_set_shader_resource_views;
    (*op).opcode = WINED3D_CS_OP_SET_SHADER_RESOURCE_VIEWS;
    (*op).type_ = type_0;
    (*op).start_idx = start_idx;
    (*op).count = count;
    memcpy(
        core::ptr::addr_of_mut!((*op).views) as *mut *mut wined3d_shader_resource_view
            as *mut core::ffi::c_void,
        views as *const core::ffi::c_void,
        (count as usize) * core::mem::size_of::<*mut wined3d_shader_resource_view>(),
    );

    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_unordered_access_views(mut cs: *mut wined3d_cs, mut data: *const ::core::ffi::c_void) {
    let mut op: *const wined3d_cs_set_unordered_access_views = data as *const wined3d_cs_set_unordered_access_views;
    let mut i: ::core::ffi::c_uint = 0;
    i = 0;
    while i < (*op).count {
        let mut prev: *mut wined3d_unordered_access_view = (*cs).state.unordered_access_view[(*op).pipeline as usize][(*op).start_idx.wrapping_add(i) as usize];
        let mut view: *mut wined3d_unordered_access_view = (*core::ptr::addr_of!((*op).uavs[0]).add(i as usize)).view;
        let mut initial_count: ::core::ffi::c_uint = (*core::ptr::addr_of!((*op).uavs[0]).add(i as usize)).initial_count;

        (*cs).state.unordered_access_view[(*op).pipeline as usize][(*op).start_idx.wrapping_add(i) as usize] = view;

        if !view.is_null() {
            (*(*view).resource).bind_count += 1;
        }
        if !prev.is_null() {
            (*(*prev).resource).bind_count -= 1;
        }

        if !view.is_null() && initial_count != !(0) {
            wined3d_unordered_access_view_set_counter(view, initial_count);
        }

        i = i.wrapping_add(1);
    }

    device_invalidate_state((*cs).c.device, STATE_UNORDERED_ACCESS_VIEW_BINDING!((*op).pipeline));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_unordered_access_views(context: *mut wined3d_device_context, pipeline: wined3d_pipeline, start_idx: ::core::ffi::c_uint, count: ::core::ffi::c_uint, views: *const *mut wined3d_unordered_access_view, initial_counts: *const ::core::ffi::c_uint) {
    let mut op: *mut wined3d_cs_set_unordered_access_views =
        ::core::ptr::null_mut::<wined3d_cs_set_unordered_access_views>();
    let mut i: ::core::ffi::c_uint = 0;
    op = wined3d_device_context_require_space(
        context,
        (core::mem::offset_of!(wined3d_cs_set_unordered_access_views, uavs)
            + (count as usize) * core::mem::size_of::<C2RustUnnamed_45>()) as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_set_unordered_access_views;
    (*op).opcode = WINED3D_CS_OP_SET_UNORDERED_ACCESS_VIEWS;
    (*op).pipeline = pipeline;
    (*op).start_idx = start_idx;
    (*op).count = count;
    i = 0 as ::core::ffi::c_uint;
    while i < count {
        let fresh28 = &raw mut (*(*op).uavs.as_mut_ptr().offset(i as isize)).view;
        *fresh28 = *views.offset(i as isize);
        (*(*op).uavs.as_mut_ptr().offset(i as isize)).initial_count =
            if !initial_counts.is_null() {
                *initial_counts.offset(i as isize)
            } else {
                !(0 as ::core::ffi::c_uint)
            };
        i = i.wrapping_add(1);
    }
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_samplers(mut cs: *mut wined3d_cs, mut data: *const ::core::ffi::c_void) {
    let op: *const wined3d_cs_set_samplers = data as *const wined3d_cs_set_samplers;
    let mut i: ::core::ffi::c_uint = 0;
    i = 0 as ::core::ffi::c_uint;
    while i < (*op).count {
        (*cs).state.sampler[(*op).type_ as usize][(*op).start_idx.wrapping_add(i) as usize] =
            *(core::ptr::addr_of!((*op).samplers) as *const *mut wined3d_sampler).add(i as usize);
        i = i.wrapping_add(1);
    }
    if (*op).type_ as ::core::ffi::c_uint != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_uint {
        device_invalidate_state((*cs).c.device, STATE_GRAPHICS_SHADER_RESOURCE_BINDING as ::core::ffi::c_uint);
    } else {
        device_invalidate_state((*cs).c.device, STATE_COMPUTE_SHADER_RESOURCE_BINDING as ::core::ffi::c_uint);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_samplers(mut context: *mut wined3d_device_context, mut type_0: wined3d_shader_type, mut start_idx: ::core::ffi::c_uint, mut count: ::core::ffi::c_uint, mut samplers: *const *mut wined3d_sampler) {
    let mut op: *mut wined3d_cs_set_samplers = ::core::ptr::null_mut::<wined3d_cs_set_samplers>();
    op = wined3d_device_context_require_space(
        context,
        (FIELD_OFFSET!(wined3d_cs_set_samplers, samplers[count as usize]) as usize) as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_set_samplers;
    (*op).opcode = WINED3D_CS_OP_SET_SAMPLERS;
    (*op).type_ = type_0;
    (*op).start_idx = start_idx;
    (*op).count = count;
    memcpy(
        &raw mut (*op).samplers as *mut *mut wined3d_sampler as *mut ::core::ffi::c_void,
        samplers as *const ::core::ffi::c_void,
        (count as size_t).wrapping_mul(::core::mem::size_of::<*mut wined3d_sampler>() as size_t),
    );
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_shader(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let op: *const wined3d_cs_set_shader = data as *const wined3d_cs_set_shader;

    /* CB binding may have been skipped earlier if the shader wasn't set, so make it happen. */
    if (*cs).state.shader[(*op).type_ as usize].is_null() && !(*op).shader.is_null() {
        device_invalidate_state((*cs).c.device, STATE_CONSTANT_BUFFER!(((*op).type_) as i32));
    }
    (*cs).state.shader[(*op).type_ as usize] = (*op).shader;
    device_invalidate_state((*cs).c.device, STATE_SHADER!(((*op).type_) as i32));
    if (*op).type_ as u32 != WINED3D_SHADER_TYPE_COMPUTE as u32 {
        device_invalidate_state((*cs).c.device, (STATE_GRAPHICS_SHADER_RESOURCE_BINDING as u32));
    } else {
        device_invalidate_state((*cs).c.device, (STATE_COMPUTE_SHADER_RESOURCE_BINDING as u32));
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_shader(mut context: *mut wined3d_device_context, mut type_0: wined3d_shader_type, mut shader: *mut wined3d_shader) {
    let mut op: *mut wined3d_cs_set_shader = ::core::ptr::null_mut::<wined3d_cs_set_shader>();
    op = wined3d_device_context_require_space(
        context,
        ::core::mem::size_of::<wined3d_cs_set_shader>() as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_set_shader;
    (*op).opcode = WINED3D_CS_OP_SET_SHADER;
    (*op).type_ = type_0;
    (*op).shader = shader;
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_blend_state(cs: *mut wined3d_cs, data: *const ::core::ffi::c_void) {
    let op: *const wined3d_cs_set_blend_state = data as *const wined3d_cs_set_blend_state;
    let device: *mut wined3d_device = (*cs).c.device;
    let state: *mut wined3d_state = core::ptr::addr_of_mut!((*cs).state);

    if (*state).blend_state != (*op).state {
        (*state).blend_state = (*op).state;
        device_invalidate_state(device, (STATE_BLEND as u32));
    }
    (*state).blend_factor = (*op).factor;
    device_invalidate_state(device, (STATE_BLEND_FACTOR as u32));
    (*state).sample_mask = (*op).sample_mask as uint32_t;
    device_invalidate_state(device, (STATE_SAMPLE_MASK as u32));
    let mut i: ::core::ffi::c_uint = 0;
    while i < (*device).context_count {
        (*(*(*device).contexts.offset(i as isize))).set_update_multisample_state((1) as _);
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_blend_state(mut context: *mut wined3d_device_context, mut state: *mut wined3d_blend_state, mut blend_factor: *const wined3d_color, mut sample_mask: ::core::ffi::c_uint) {
    let mut op: *mut wined3d_cs_set_blend_state = ::core::ptr::null_mut::<wined3d_cs_set_blend_state>();
    op = wined3d_device_context_require_space(
        context,
        ::core::mem::size_of::<wined3d_cs_set_blend_state>() as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_set_blend_state;
    (*op).opcode = WINED3D_CS_OP_SET_BLEND_STATE;
    (*op).state = state;
    (*op).factor = *blend_factor;
    (*op).sample_mask = sample_mask;
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_depth_stencil_state(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let op: *const wined3d_cs_set_depth_stencil_state = data as *const wined3d_cs_set_depth_stencil_state;
    let state: *mut wined3d_state = core::ptr::addr_of_mut!((*cs).state);

    if (*state).depth_stencil_state != (*op).state {
        (*state).depth_stencil_state = (*op).state;
        device_invalidate_state((*cs).c.device, (STATE_DEPTH_STENCIL as u32));
    }
    (*state).stencil_ref = (*op).stencil_ref;
    device_invalidate_state((*cs).c.device, (STATE_STENCIL_REF as u32));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_depth_stencil_state(context: *mut wined3d_device_context, state: *mut wined3d_depth_stencil_state, stencil_ref: ::core::ffi::c_uint) {
    let mut op: *mut wined3d_cs_set_depth_stencil_state = core::ptr::null_mut();

    op = wined3d_device_context_require_space(context, core::mem::size_of::<wined3d_cs_set_depth_stencil_state>(), WINED3D_CS_QUEUE_DEFAULT) as *mut wined3d_cs_set_depth_stencil_state;
    (*op).opcode = WINED3D_CS_OP_SET_DEPTH_STENCIL_STATE;
    (*op).state = state;
    (*op).stencil_ref = stencil_ref;

    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_rasterizer_state(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let op: *const wined3d_cs_set_rasterizer_state = data as *const wined3d_cs_set_rasterizer_state;
    (*cs).state.rasterizer_state = (*op).state;
    device_invalidate_state((*cs).c.device, (STATE_RASTERIZER as u32));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_rasterizer_state(context: *mut wined3d_device_context, rasterizer_state: *mut wined3d_rasterizer_state) {
    let mut op: *mut wined3d_cs_set_rasterizer_state;

    op = wined3d_device_context_require_space(
        context,
        core::mem::size_of::<wined3d_cs_set_rasterizer_state>(),
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_set_rasterizer_state;
    (*op).opcode = WINED3D_CS_OP_SET_RASTERIZER_STATE;
    (*op).state = rasterizer_state;

    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_depth_bounds(mut cs: *mut wined3d_cs, mut data: *const core::ffi::c_void) {
    let op: *const wined3d_cs_set_depth_bounds = data as *const wined3d_cs_set_depth_bounds;
    (*cs).state.depth_bounds_enable = (*op).enable;
    (*cs).state.depth_bounds_min = (*op).min_depth;
    (*cs).state.depth_bounds_max = (*op).max_depth;
    device_invalidate_state((*cs).c.device, (STATE_DEPTH_BOUNDS as u32));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_set_depth_bounds(context: *mut wined3d_device_context, enable: bool, min_depth: ::core::ffi::c_float, max_depth: ::core::ffi::c_float) {
    let mut op: *mut wined3d_cs_set_depth_bounds = core::ptr::null_mut();

    op = wined3d_device_context_require_space(context, core::mem::size_of::<wined3d_cs_set_depth_bounds>() as size_t, WINED3D_CS_QUEUE_DEFAULT)
        as *mut wined3d_cs_set_depth_bounds;
    (*op).opcode = WINED3D_CS_OP_SET_DEPTH_BOUNDS;
    (*op).enable = enable;
    (*op).min_depth = min_depth;
    (*op).max_depth = max_depth;

    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_render_state(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let op: *const wined3d_cs_set_render_state = data as *const wined3d_cs_set_render_state;
    (*cs).state.render_states[(*op).state as usize] = (*op).value;
    device_invalidate_state((*cs).c.device, STATE_RENDER!((*op).state));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_render_state(mut context: *mut wined3d_device_context, mut state: wined3d_render_state, mut value: ::core::ffi::c_uint) {
    let mut op: *mut wined3d_cs_set_render_state =
        ::core::ptr::null_mut::<wined3d_cs_set_render_state>();
    op = wined3d_device_context_require_space(
        context,
        ::core::mem::size_of::<wined3d_cs_set_render_state>() as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_set_render_state;
    (*op).opcode = WINED3D_CS_OP_SET_RENDER_STATE;
    (*op).state = state;
    (*op).value = value as DWORD;
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_texture_state(mut cs: *mut wined3d_cs, mut data: *const ::core::ffi::c_void) {
    let mut op: *const wined3d_cs_set_texture_state = data as *const wined3d_cs_set_texture_state;
    (*cs).state.texture_states[(*op).stage as usize][(*op).state as usize] = (*op).value as uint32_t;
    device_invalidate_state((*cs).c.device, (STATE_TEXTURESTAGE!((*op).stage as ::core::ffi::c_uint, ((*op).state as ::core::ffi::c_uint) as i32)) as u32);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_texture_state(mut context: *mut wined3d_device_context, mut stage: ::core::ffi::c_uint, mut state: wined3d_texture_stage_state, mut value: ::core::ffi::c_uint) {
    let mut op: *mut wined3d_cs_set_texture_state = ::core::ptr::null_mut::<wined3d_cs_set_texture_state>();
    op = wined3d_device_context_require_space(
        context,
        ::core::mem::size_of::<wined3d_cs_set_texture_state>() as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_set_texture_state;
    (*op).opcode = WINED3D_CS_OP_SET_TEXTURE_STATE;
    (*op).stage = stage as UINT;
    (*op).state = state;
    (*op).value = value as DWORD;
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_transform(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let op: *const wined3d_cs_set_transform = data as *const wined3d_cs_set_transform;
    (*cs).state.transforms[(*op).state as usize] = (*op).matrix;
    if (*op).state as u32 == WINED3D_TS_PROJECTION as u32
        && (*cs).state.render_states[WINED3D_RS_FOGENABLE as usize] != 0
        && (*cs).state.render_states[WINED3D_RS_FOGTABLEMODE as usize] != WINED3D_FOG_NONE as u32
    {
        device_invalidate_state((*cs).c.device, STATE_SHADER!((WINED3D_SHADER_TYPE_VERTEX) as i32));
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_transform(mut context: *mut wined3d_device_context, mut state: wined3d_transform_state, mut matrix: *const wined3d_matrix) {
    let mut op: *mut wined3d_cs_set_transform = ::core::ptr::null_mut::<wined3d_cs_set_transform>();
    op = wined3d_device_context_require_space(
        context,
        ::core::mem::size_of::<wined3d_cs_set_transform>() as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_set_transform;
    (*op).opcode = WINED3D_CS_OP_SET_TRANSFORM;
    (*op).state = state;
    (*op).matrix = *matrix;
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_clip_plane(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let op: *const wined3d_cs_set_clip_plane = data as *const wined3d_cs_set_clip_plane;

    (*cs).state.clip_planes[(*op).plane_idx as usize] = (*op).plane;
    device_invalidate_state((*cs).c.device, (STATE_CLIPPLANE!(((*op).plane_idx) as i32)) as u32);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_clip_plane(context: *mut wined3d_device_context, plane_idx: ::core::ffi::c_uint, plane: *const wined3d_vec4) {
    let mut op: *mut wined3d_cs_set_clip_plane = core::ptr::null_mut();

    op = wined3d_device_context_require_space(
        context,
        core::mem::size_of::<wined3d_cs_set_clip_plane>(),
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_set_clip_plane;
    (*op).opcode = WINED3D_CS_OP_SET_CLIP_PLANE;
    (*op).plane_idx = plane_idx as UINT;
    (*op).plane = *plane;

    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_color_key(
    mut cs: *mut wined3d_cs,
    mut data: *const ::core::ffi::c_void,
) {
    let mut op: *const wined3d_cs_set_color_key = data as *const wined3d_cs_set_color_key;
    let mut texture: *mut wined3d_texture = (*op).texture;
    if (*op).set != 0 {
        match ((*op).flags as ::core::ffi::c_int) as u32 {
            WINED3D_CKEY_DST_BLT => {
                (*texture).async_.dst_blt_color_key = (*op).color_key;
                (*texture).async_.color_key_flags |= WINED3D_CKEY_DST_BLT as DWORD;
            }
            WINED3D_CKEY_DST_OVERLAY => {
                (*texture).async_.dst_overlay_color_key = (*op).color_key;
                (*texture).async_.color_key_flags |= WINED3D_CKEY_DST_OVERLAY as DWORD;
            }
            WINED3D_CKEY_SRC_BLT => {
                if texture
                    == wined3d_state_get_ffp_texture(&raw mut (*cs).state, 0 as ::core::ffi::c_uint)
                {
                    if ((*texture).async_.color_key_flags & WINED3D_CKEY_SRC_BLT as DWORD) == 0 {
                        device_invalidate_state(
                            (*cs).c.device,
                            STATE_SHADER!((WINED3D_SHADER_TYPE_PIXEL) as i32) as ::core::ffi::c_uint,
                        );
                    }
                }
                (*texture).async_.src_blt_color_key = (*op).color_key;
                (*texture).async_.color_key_flags |= WINED3D_CKEY_SRC_BLT as DWORD;
            }
            WINED3D_CKEY_SRC_OVERLAY => {
                (*texture).async_.src_overlay_color_key = (*op).color_key;
                (*texture).async_.color_key_flags |= WINED3D_CKEY_SRC_OVERLAY as DWORD;
            }
            _ => {}
        }
    } else {
        match ((*op).flags as ::core::ffi::c_int) as u32 {
            WINED3D_CKEY_DST_BLT => {
                (*texture).async_.color_key_flags &= !(WINED3D_CKEY_DST_BLT as DWORD);
            }
            WINED3D_CKEY_DST_OVERLAY => {
                (*texture).async_.color_key_flags &= !(WINED3D_CKEY_DST_OVERLAY as DWORD);
            }
            WINED3D_CKEY_SRC_BLT => {
                if texture
                    == wined3d_state_get_ffp_texture(&raw mut (*cs).state, 0 as ::core::ffi::c_uint)
                    && ((*texture).async_.color_key_flags & WINED3D_CKEY_SRC_BLT as DWORD) != 0
                {
                    device_invalidate_state(
                        (*cs).c.device,
                        STATE_SHADER!((WINED3D_SHADER_TYPE_PIXEL) as i32) as ::core::ffi::c_uint,
                    );
                }
                (*texture).async_.color_key_flags &= !(WINED3D_CKEY_SRC_BLT as DWORD);
            }
            WINED3D_CKEY_SRC_OVERLAY => {
                (*texture).async_.color_key_flags &= !(WINED3D_CKEY_SRC_OVERLAY as DWORD);
            }
            _ => {}
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_cs_emit_set_color_key(cs: *mut wined3d_cs, texture: *mut wined3d_texture, flags: WORD, color_key: *const wined3d_color_key) {
    let mut op: *mut wined3d_cs_set_color_key = core::ptr::null_mut::<wined3d_cs_set_color_key>();

    op = wined3d_device_context_require_space(
        core::ptr::addr_of_mut!((*cs).c),
        core::mem::size_of::<wined3d_cs_set_color_key>(),
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_set_color_key;
    (*op).opcode = WINED3D_CS_OP_SET_COLOR_KEY;
    (*op).texture = texture;
    (*op).flags = flags;
    if !color_key.is_null() {
        (*op).color_key = *color_key;
        (*op).set = 1 as WORD;
    } else {
        (*op).set = 0 as WORD;
    }

    wined3d_device_context_submit(core::ptr::addr_of_mut!((*cs).c), WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_light(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let op: *const wined3d_cs_set_light = data as *const wined3d_cs_set_light;
    let mut light_info: *mut wined3d_light_info = core::ptr::null_mut::<wined3d_light_info>();
    let mut light_idx: core::ffi::c_uint = 0;

    light_idx = (*op).light.OriginalIndex;

    light_info = wined3d_light_state_get_light(core::ptr::addr_of_mut!((*cs).state.light_state), light_idx);
    if light_info.is_null() {
        TRACE!("Adding new light.\n");
        light_info = calloc(
            (1 as size_t) as _,
            (core::mem::size_of::<wined3d_light_info>() as size_t) as _,
        ) as *mut wined3d_light_info;
        if light_info.is_null() {
            ERR!("Failed to allocate light info.\n");
            return;
        }
        (*light_info).glIndex = -(1 as LONG);
        (*light_info).OriginalIndex = light_idx;
        rb_put(
            core::ptr::addr_of_mut!((*cs).state.light_state.lights_tree),
            light_idx as ULONG_PTR as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*light_info).entry),
        );
    }

    (*light_info).OriginalParms = (*op).light.OriginalParms;
    (*light_info).constants = (*op).light.constants;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_light(context: *mut wined3d_device_context, light: *const wined3d_light_info) {
    let mut op: *mut wined3d_cs_set_light = core::ptr::null_mut();
    op = wined3d_device_context_require_space(context, core::mem::size_of::<wined3d_cs_set_light>(), WINED3D_CS_QUEUE_DEFAULT) as *mut wined3d_cs_set_light;
    (*op).opcode = WINED3D_CS_OP_SET_LIGHT;
    (*op).light = *light;
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_light_enable(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let op: *const wined3d_cs_set_light_enable = data as *const wined3d_cs_set_light_enable;
    let device: *mut wined3d_device = (*cs).c.device;
    let mut light_info: *mut wined3d_light_info;

    light_info = wined3d_light_state_get_light(core::ptr::addr_of_mut!((*cs).state.light_state), (*op).idx);
    if light_info.is_null() {
        ERR!("Light doesn't exist.\n");
        return;
    }

    wined3d_light_state_enable_light(
        core::ptr::addr_of_mut!((*cs).state.light_state),
        core::ptr::addr_of_mut!((*(*device).adapter).d3d_info),
        light_info,
        (*op).enable,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_light_enable(context: *mut wined3d_device_context, idx: ::core::ffi::c_uint, enable: BOOL) {
    let mut op: *mut wined3d_cs_set_light_enable = core::ptr::null_mut();
    op = wined3d_device_context_require_space(context, core::mem::size_of::<wined3d_cs_set_light_enable>(), WINED3D_CS_QUEUE_DEFAULT) as *mut wined3d_cs_set_light_enable;
    (*op).opcode = WINED3D_CS_OP_SET_LIGHT_ENABLE;
    (*op).idx = idx;
    (*op).enable = enable;
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_set_feature_level(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let op: *const wined3d_cs_set_feature_level = data as *const wined3d_cs_set_feature_level;

    (*cs).state.feature_level = (*op).level;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_set_feature_level(mut context: *mut wined3d_device_context, mut level: wined3d_feature_level) {
    let mut op: *mut wined3d_cs_set_feature_level = core::ptr::null_mut::<wined3d_cs_set_feature_level>();
    op = wined3d_device_context_require_space(
        context,
        core::mem::size_of::<wined3d_cs_set_feature_level>() as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_set_feature_level;
    (*op).opcode = WINED3D_CS_OP_SET_FEATURE_LEVEL;
    (*op).level = level;
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn prepare_push_constant_buffer(
    mut context: *mut wined3d_device_context,
    mut type_0: wined3d_push_constants,
) -> bool {
    let info: *const push_constant_info =
        core::ptr::addr_of!(wined3d_cs_push_constant_info[type_0 as usize]);
    let device: *mut wined3d_device = (*context).device;
    let gpu: bool = (*(*device).adapter).d3d_info.gpu_push_constants() != 0;
    let mut hr: HRESULT = 0;
    let mut desc: wined3d_buffer_desc = wined3d_buffer_desc {
        byte_width: ((*info).max_count as usize).wrapping_mul((*info).size) as u32,
        usage: 0,
        bind_flags: 0,
        access: WINED3D_RESOURCE_ACCESS_CPU | WINED3D_RESOURCE_ACCESS_MAP_R | WINED3D_RESOURCE_ACCESS_MAP_W,
        misc_flags: 0,
        structure_byte_stride: 0,
    };

    if gpu {
        desc.bind_flags = WINED3D_BIND_CONSTANT_BUFFER;
        desc.access = WINED3D_RESOURCE_ACCESS_GPU;
    }

    if (*device).push_constants[type_0 as usize].is_null() && {
        hr = wined3d_buffer_create(
            device,
            core::ptr::addr_of_mut!(desc),
            core::ptr::null::<wined3d_sub_resource_data>(),
            core::ptr::null_mut::<core::ffi::c_void>(),
            core::ptr::addr_of!(wined3d_null_parent_ops),
            core::ptr::addr_of_mut!((*device).push_constants[type_0 as usize]),
        );
        hr < 0
    } {
        ERR!("Failed to create push constant buffer, hr %#lx.\n", hr);
        return false;
    }

    return true;
}

unsafe extern "C" fn wined3d_cs_exec_push_constants(
    cs: *mut wined3d_cs,
    data: *const ::core::ffi::c_void,
) {
    let op: *const wined3d_cs_push_constants = data as *const wined3d_cs_push_constants;
    let device: *mut wined3d_device = (*cs).c.device;
    let mut ffp_start_idx: ::core::ffi::c_uint = 0;
    let mut ffp_end_idx: ::core::ffi::c_uint = 0;
    let mut context_count: ::core::ffi::c_uint = 0;
    let mut i: ::core::ffi::c_uint = 0;

    ffp_start_idx = ((*op).start_idx as usize)
        .wrapping_div(::core::mem::size_of::<wined3d_vec4>() as usize)
        as ::core::ffi::c_uint;
    ffp_end_idx = ((*op).start_idx.wrapping_add((*op).count) as usize)
        .wrapping_add(::core::mem::size_of::<wined3d_vec4>() as usize)
        .wrapping_div(::core::mem::size_of::<wined3d_vec4>() as usize)
        as ::core::ffi::c_uint;

    if (*op).type_ as ::core::ffi::c_uint == WINED3D_PUSH_CONSTANTS_VS_F as ::core::ffi::c_uint {
        (*(*device).shader_backend)
            .shader_update_float_vertex_constants
            .unwrap_unchecked()(
            device,
            (*op).start_idx as UINT,
            (*op).count as UINT,
        );
    } else if (*op).type_ as ::core::ffi::c_uint == WINED3D_PUSH_CONSTANTS_PS_F as ::core::ffi::c_uint {
        (*(*device).shader_backend)
            .shader_update_float_pixel_constants
            .unwrap_unchecked()(
            device,
            (*op).start_idx as UINT,
            (*op).count as UINT,
        );
    } else if (*op).type_ as ::core::ffi::c_uint == WINED3D_PUSH_CONSTANTS_VS_FFP as ::core::ffi::c_uint {
        (*(*device).shader_backend)
            .shader_update_float_vertex_constants
            .unwrap_unchecked()(
            device,
            ffp_start_idx as UINT,
            (ffp_end_idx as UINT).wrapping_sub(ffp_start_idx as UINT),
        );
    } else if (*op).type_ as ::core::ffi::c_uint == WINED3D_PUSH_CONSTANTS_PS_FFP as ::core::ffi::c_uint {
        (*(*device).shader_backend)
            .shader_update_float_pixel_constants
            .unwrap_unchecked()(
            device,
            ffp_start_idx as UINT,
            (ffp_end_idx as UINT).wrapping_sub(ffp_start_idx as UINT),
        );
    }

    i = 0 as ::core::ffi::c_uint;
    context_count = (*device).context_count as ::core::ffi::c_uint;
    while i < context_count {
        (**(*device).contexts.offset(i as isize)).constant_update_mask |=
            (*op).update_mask as ::core::ffi::c_uint;
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn wined3d_device_context_emit_push_constants(
    context: *mut wined3d_device_context,
    type_: wined3d_push_constants,
    update_mask: uint32_t,
    start_idx: ::core::ffi::c_uint,
    count: ::core::ffi::c_uint,
) {
    let cs: *mut wined3d_cs = wined3d_cs_from_context(context);
    let mut op: *mut wined3d_cs_push_constants = core::ptr::null_mut::<wined3d_cs_push_constants>();

    op = wined3d_device_context_require_space(
        core::ptr::addr_of_mut!((*cs).c),
        core::mem::size_of::<wined3d_cs_push_constants>() as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_push_constants;
    (*op).opcode = WINED3D_CS_OP_PUSH_CONSTANTS;
    (*op).type_ = type_;
    (*op).update_mask = update_mask;
    (*op).start_idx = start_idx;
    (*op).count = count;

    wined3d_device_context_submit(core::ptr::addr_of_mut!((*cs).c), WINED3D_CS_QUEUE_DEFAULT);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_push_constants(context: *mut wined3d_device_context, type_: wined3d_push_constants, update_mask: uint32_t, start_idx: ::core::ffi::c_uint, count: ::core::ffi::c_uint, constants: *const ::core::ffi::c_void) {
    let info: *const push_constant_info =
        (&raw const wined3d_cs_push_constant_info as *const push_constant_info)
            .offset(type_ as isize) as *const push_constant_info;
    let byte_offset: ::core::ffi::c_uint =
        (start_idx as size_t).wrapping_mul((*info).size) as ::core::ffi::c_uint;
    let byte_size: ::core::ffi::c_uint =
        (count as size_t).wrapping_mul((*info).size) as ::core::ffi::c_uint;
    let mut box_: wined3d_box = wined3d_box {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        front: 0,
        back: 0,
    };

    if !prepare_push_constant_buffer(context, type_) {
        return;
    }

    wined3d_box_set(
        &raw mut box_,
        byte_offset,
        0,
        byte_offset.wrapping_add(byte_size),
        1,
        0,
        1,
    );
    wined3d_device_context_emit_update_sub_resource(
        context,
        &raw mut (**(&raw mut (*(*context).device).push_constants as *mut *mut wined3d_buffer)
            .offset(type_ as isize))
        .resource,
        0,
        &raw mut box_,
        constants,
        byte_size,
        byte_size,
    );
    wined3d_device_context_emit_push_constants(context, type_, update_mask, start_idx, count);
}

unsafe extern "C" fn wined3d_cs_exec_reset_state(cs: *mut wined3d_cs, data: *const ::core::ffi::c_void) {
    let device: *const wined3d_device = (*cs).c.device;
    let op: *const wined3d_cs_reset_state = data as *const wined3d_cs_reset_state;
    let mut state_table: *const wined3d_state_entry = core::ptr::null();
    let mut state: ::core::ffi::c_uint;

    state_cleanup(core::ptr::addr_of_mut!((*cs).state));
    wined3d_state_reset(
        core::ptr::addr_of_mut!((*cs).state),
        core::ptr::addr_of!((*(*device).adapter).d3d_info),
    );
    if (*op).invalidate != false {
        state_table = core::ptr::addr_of!((*device).state_table) as *const wined3d_state_entry;
        state = 0 as ::core::ffi::c_uint;
        while state <= STATE_HIGHEST as ::core::ffi::c_uint {
            if (*state_table.offset(state as isize)).representative != 0 {
                device_invalidate_state(device, state);
            }
            state = state.wrapping_add(1);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_reset_state(context: *mut wined3d_device_context, invalidate: bool) {
    let mut op: *mut wined3d_cs_reset_state;

    op = wined3d_device_context_require_space(context, core::mem::size_of::<wined3d_cs_reset_state>(), WINED3D_CS_QUEUE_DEFAULT) as *mut wined3d_cs_reset_state;
    (*op).opcode = WINED3D_CS_OP_RESET_STATE;
    (*op).invalidate = invalidate;

    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_callback(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let _ = cs;
    let op: *const wined3d_cs_callback = data as *const wined3d_cs_callback;
    ((*op).callback).unwrap_unchecked()((*op).object);
}

unsafe extern "C" fn wined3d_cs_emit_callback(cs: *mut wined3d_cs, callback: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>, object: *mut core::ffi::c_void) {
    let mut op: *mut wined3d_cs_callback = core::ptr::null_mut();

    op = wined3d_device_context_require_space(core::ptr::addr_of_mut!((*cs).c), core::mem::size_of::<wined3d_cs_callback>(), WINED3D_CS_QUEUE_DEFAULT) as *mut wined3d_cs_callback;
    (*op).opcode = WINED3D_CS_OP_CALLBACK;
    (*op).callback = callback;
    (*op).object = object;

    wined3d_device_context_submit(core::ptr::addr_of_mut!((*cs).c), WINED3D_CS_QUEUE_DEFAULT);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_cs_destroy_object(cs: *mut wined3d_cs, callback: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>, object: *mut ::core::ffi::c_void) {
    wined3d_cs_emit_callback(cs, callback, object);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_cs_init_object(cs: *mut wined3d_cs, callback: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>, object: *mut ::core::ffi::c_void) {
    wined3d_cs_emit_callback(cs, callback, object);
}

unsafe extern "C" fn wined3d_cs_exec_query_issue(mut cs: *mut wined3d_cs, mut data: *const ::core::ffi::c_void) {
    let mut op: *const wined3d_cs_query_issue = data as *const wined3d_cs_query_issue;
    let mut query: *mut wined3d_query = (*op).query;
    let mut poll: BOOL = 0;
    poll = (*(*query).query_ops)
        .query_issue
        .unwrap_unchecked()(query, (*op).flags);
    if !(*query).poll_in_cs {
        if ((*op).flags & WINED3DISSUE_END) != 0 {
            InterlockedIncrement(
                &raw mut (*query).counter_retrieved as *mut ::core::ffi::c_long
                    as *mut ::core::ffi::c_long,
            );
        }
        return;
    }
    if poll != 0 && list_empty(&raw mut (*query).poll_list_entry) != 0 {
        if (*query).buffer_object != 0 {
            InterlockedIncrement(
                &raw mut (*query).counter_retrieved as *mut ::core::ffi::c_long
                    as *mut ::core::ffi::c_long,
            );
        } else {
            list_add_tail(
                &raw mut (*cs).query_poll_list,
                &raw mut (*query).poll_list_entry,
            );
        }
        return;
    }
    if ((*op).flags & WINED3DISSUE_BEGIN) != 0
        && poll == 0
        && list_empty(&raw mut (*query).poll_list_entry) == 0
    {
        list_remove(&raw mut (*query).poll_list_entry);
        list_init(&raw mut (*query).poll_list_entry);
        InterlockedIncrement(
            &raw mut (*query).counter_retrieved as *mut ::core::ffi::c_long
                as *mut ::core::ffi::c_long,
        );
        return;
    }
    if ((*op).flags & WINED3DISSUE_END) != 0 {
        InterlockedIncrement(
            &raw mut (*query).counter_retrieved as *mut ::core::ffi::c_long
                as *mut ::core::ffi::c_long,
        );
    }
}

unsafe extern "C" fn wined3d_cs_issue_query(
    mut context: *mut wined3d_device_context,
    mut query: *mut wined3d_query,
    mut flags: ::core::ffi::c_uint,
) {
    let mut cs: *mut wined3d_cs = wined3d_cs_from_context(context);
    let mut op: *mut wined3d_cs_query_issue = ::core::ptr::null_mut::<wined3d_cs_query_issue>();
    if flags & WINED3DISSUE_END != 0 {
        (*query).counter_main += 1;
    }
    op = wined3d_device_context_require_space(
        context,
        ::core::mem::size_of::<wined3d_cs_query_issue>() as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_query_issue;
    (*op).opcode = WINED3D_CS_OP_QUERY_ISSUE;
    (*op).query = query;
    (*op).flags = flags as uint32_t;
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
    (*cs).queries_flushed = FALSE as BOOL;
    if flags & WINED3DISSUE_BEGIN != 0 {
        (*query).state = QUERY_BUILDING;
    } else {
        (*query).state = QUERY_SIGNALLED;
    };
}

unsafe extern "C" fn wined3d_cs_reference_command_list(context: *mut wined3d_device_context, list: *mut wined3d_command_list) {
    let mut cs: *mut wined3d_cs = wined3d_cs_from_context(context);
    let mut i: SIZE_T = 0;
    if (*list).query_count != 0 {
        (*cs).queries_flushed = (FALSE as i32);
        i = 0;
        while i < (*list).query_count {
            if (*(*list).queries.offset(i as isize)).flags & WINED3DISSUE_END != 0 {
                (*(*(*list).queries.offset(i as isize)).query).counter_main += 1;
                (*(*(*list).queries.offset(i as isize)).query).state = QUERY_SIGNALLED;
            } else {
                (*(*(*list).queries.offset(i as isize)).query).state = QUERY_BUILDING;
            }
            i = i.wrapping_add(1);
        }
    }
    i = 0;
    while i < (*list).resource_count {
        wined3d_resource_reference(*(*list).resources.offset(i as isize));
        i = i.wrapping_add(1);
    }
    i = 0;
    while i < (*list).command_list_count {
        wined3d_cs_reference_command_list(context, *(*list).command_lists.offset(i as isize));
        i = i.wrapping_add(1);
    }
    i = 0;
    while i < (*list).upload_count {
        invalidate_client_address((*(*list).uploads.offset(i as isize)).resource);
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn wined3d_cs_exec_preload_resource(
    mut cs: *mut wined3d_cs,
    mut data: *const ::core::ffi::c_void,
) {
    let mut op: *const wined3d_cs_preload_resource = data as *const wined3d_cs_preload_resource;
    let mut resource: *mut wined3d_resource = (*op).resource;
    (*(*resource).resource_ops)
        .resource_preload
        .unwrap_unchecked()(resource);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_cs_emit_preload_resource(cs: *mut wined3d_cs, resource: *mut wined3d_resource) {
    let mut op: *mut wined3d_cs_preload_resource = core::ptr::null_mut::<wined3d_cs_preload_resource>();
    op = wined3d_device_context_require_space(
        core::ptr::addr_of_mut!((*cs).c),
        core::mem::size_of::<wined3d_cs_preload_resource>(),
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_preload_resource;
    (*op).opcode = WINED3D_CS_OP_PRELOAD_RESOURCE;
    (*op).resource = resource;

    wined3d_resource_reference(resource);

    wined3d_device_context_submit(core::ptr::addr_of_mut!((*cs).c), WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_unload_resource(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let _ = cs;
    let op: *const wined3d_cs_unload_resource = data as *const wined3d_cs_unload_resource;
    let resource: *mut wined3d_resource = (*op).resource;

    (*(*resource).resource_ops).resource_unload.unwrap_unchecked()(resource);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_cs_emit_unload_resource(cs: *mut wined3d_cs, resource: *mut wined3d_resource) {
    let mut op: *mut wined3d_cs_unload_resource = core::ptr::null_mut::<wined3d_cs_unload_resource>();

    discard_client_address(resource);

    op = wined3d_device_context_require_space(
        core::ptr::addr_of_mut!((*cs).c),
        core::mem::size_of::<wined3d_cs_unload_resource>() as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_unload_resource;
    (*op).opcode = WINED3D_CS_OP_UNLOAD_RESOURCE;
    (*op).resource = resource;

    wined3d_resource_reference(resource);

    wined3d_device_context_submit(core::ptr::addr_of_mut!((*cs).c), WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_device_context_upload_bo(
    mut context: *mut wined3d_device_context,
    mut resource: *mut wined3d_resource,
    mut sub_resource_idx: ::core::ffi::c_uint,
    mut box_0: *const wined3d_box,
    mut bo: *const upload_bo,
    mut row_pitch: ::core::ffi::c_uint,
    mut slice_pitch: ::core::ffi::c_uint,
) {
    let mut op: *mut wined3d_cs_update_sub_resource =
        ::core::ptr::null_mut::<wined3d_cs_update_sub_resource>();
    WINE_TRACE!("context %p, resource %p, sub_resource_idx %u, box %s, bo %s, flags %#x, row_pitch %u, slice_pitch %u.\n", context, resource, sub_resource_idx, debug_box(box_0), debug_const_bo_address(&raw const (*bo).addr), (*bo).flags, row_pitch, slice_pitch);
    op = wined3d_device_context_require_space(
        context,
        core::mem::size_of::<wined3d_cs_update_sub_resource>() as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_update_sub_resource;
    (*op).opcode = WINED3D_CS_OP_UPDATE_SUB_RESOURCE;
    (*op).resource = resource;
    (*op).sub_resource_idx = sub_resource_idx;
    (*op).box_ = *box_0;
    (*op).bo = *bo;
    (*op).row_pitch = row_pitch;
    (*op).slice_pitch = slice_pitch;
    wined3d_device_context_reference_resource(context, resource);
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_map(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let _ = cs;
    let op: *const wined3d_cs_map = data as *const wined3d_cs_map;
    let resource: *mut wined3d_resource = (*op).resource;

    *(*op).hr = (*(*resource).resource_ops).resource_sub_resource_map.unwrap_unchecked()(
        resource,
        (*op).sub_resource_idx,
        (*op).map_ptr,
        (*op).box_,
        (*op).flags,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_map(mut context: *mut wined3d_device_context, mut resource: *mut wined3d_resource, mut sub_resource_idx: ::core::ffi::c_uint, mut map_desc: *mut wined3d_map_desc, mut box_0: *const wined3d_box, mut flags: ::core::ffi::c_uint) -> HRESULT {
    let mut op: *mut wined3d_cs_map = ::core::ptr::null_mut::<wined3d_cs_map>();
    let mut hr: HRESULT = 0;

    /* Mapping resources from the worker thread isn't an issue by itself, but
     * increasing the map count would be visible to applications. */
    wined3d_not_from_cs((*(*context).device).cs);

    if (flags & (WINED3D_MAP_DISCARD | WINED3D_MAP_NOOVERWRITE)) != 0
        && ((*(*context).ops)
            .map_upload_bo
            .unwrap_unchecked())(
            context,
            resource,
            sub_resource_idx,
            map_desc,
            box_0,
            flags as uint32_t,
        ) as ::core::ffi::c_int
            != 0
    {
        return WINED3D_OK;
    }

    WINE_TRACE_!(__wine_dbch_d3d_perf, "Mapping resource %p (type %u), flags %#x through the CS.\n", resource, (*resource).type_ as ::core::ffi::c_uint, flags);

    wined3d_resource_wait_idle(resource);

    /* We might end up invalidating the resource on the CS thread. */
    invalidate_client_address(resource);

    op = wined3d_device_context_require_space(
        context,
        core::mem::size_of::<wined3d_cs_map>() as size_t,
        WINED3D_CS_QUEUE_MAP,
    ) as *mut wined3d_cs_map;
    if op.is_null() {
        return E_OUTOFMEMORY;
    }
    (*op).opcode = WINED3D_CS_OP_MAP;
    (*op).resource = resource;
    (*op).sub_resource_idx = sub_resource_idx;
    (*op).map_ptr = core::ptr::addr_of_mut!((*map_desc).data);
    (*op).box_ = box_0;
    (*op).flags = flags as uint32_t;
    (*op).hr = core::ptr::addr_of_mut!(hr);

    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_MAP);
    wined3d_device_context_finish(context, WINED3D_CS_QUEUE_MAP);

    if hr >= 0 {
        wined3d_resource_get_sub_resource_map_pitch(
            resource,
            sub_resource_idx,
            core::ptr::addr_of_mut!((*map_desc).row_pitch),
            core::ptr::addr_of_mut!((*map_desc).slice_pitch),
        );
    }
    return hr;
}

unsafe extern "C" fn wined3d_cs_exec_unmap(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let _ = cs;
    let op: *const wined3d_cs_unmap = data as *const wined3d_cs_unmap;
    let resource: *mut wined3d_resource = (*op).resource;
    *(*op).hr = ((*(*resource).resource_ops).resource_sub_resource_unmap)
        .unwrap_unchecked()(resource, (*op).sub_resource_idx);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_unmap(mut context: *mut wined3d_device_context, mut resource: *mut wined3d_resource, mut sub_resource_idx: ::core::ffi::c_uint) -> HRESULT {
    let mut op: *mut wined3d_cs_unmap = core::ptr::null_mut::<wined3d_cs_unmap>();
    let mut box_0: wined3d_box = wined3d_box {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        front: 0,
        back: 0,
    };
    let mut bo: upload_bo = upload_bo {
        addr: wined3d_const_bo_address {
            buffer_object: core::ptr::null_mut::<wined3d_bo>(),
            addr: core::ptr::null::<BYTE>(),
        },
        flags: 0,
    };
    let mut hr: HRESULT = 0;
    if (*(*context).ops)
        .unmap_upload_bo
        .unwrap_unchecked()(
        context,
        resource,
        sub_resource_idx,
        core::ptr::addr_of_mut!(box_0),
        core::ptr::addr_of_mut!(bo),
    ) != false
    {
        let mut row_pitch: ::core::ffi::c_uint = 0;
        let mut slice_pitch: ::core::ffi::c_uint = 0;
        wined3d_resource_get_sub_resource_map_pitch(
            resource,
            sub_resource_idx,
            core::ptr::addr_of_mut!(row_pitch),
            core::ptr::addr_of_mut!(slice_pitch),
        );
        if (bo.flags & (UPLOAD_BO_UPLOAD_ON_UNMAP as uint32_t)) != 0 {
            wined3d_device_context_upload_bo(
                context,
                resource,
                sub_resource_idx,
                core::ptr::addr_of_mut!(box_0),
                core::ptr::addr_of_mut!(bo),
                row_pitch,
                slice_pitch,
            );
        }
        return WINED3D_OK;
    }
    wined3d_not_from_cs((*(*context).device).cs);
    if __wine_dbch_d3d_perf.flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = core::ptr::addr_of_mut!(__wine_dbch_d3d_perf);
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/directx/wine/wined3d/cs.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"wined3d_device_context_emit_unmap\0" as *const u8 as *const ::core::ffi::c_char,
            2561 as ::core::ffi::c_int,
            b"Unmapping resource %p (type %u) through the CS.\n\0" as *const u8
                as *const ::core::ffi::c_char,
            resource,
            (*resource).type_ as ::core::ffi::c_uint,
        );
    }
    op = wined3d_device_context_require_space(
        context,
        core::mem::size_of::<wined3d_cs_unmap>() as size_t,
        WINED3D_CS_QUEUE_MAP,
    ) as *mut wined3d_cs_unmap;
    if op.is_null() {
        return E_OUTOFMEMORY;
    }
    (*op).opcode = WINED3D_CS_OP_UNMAP;
    (*op).resource = resource;
    (*op).sub_resource_idx = sub_resource_idx;
    (*op).hr = core::ptr::addr_of_mut!(hr);
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_MAP);
    wined3d_device_context_finish(context, WINED3D_CS_QUEUE_MAP);
    return hr;
}

unsafe extern "C" fn wined3d_cs_exec_map_bo_address(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let op: *const wined3d_cs_map_bo_address = data as *const wined3d_cs_map_bo_address;
    let mut context: *mut wined3d_context = core::ptr::null_mut();

    context = context_acquire((*cs).c.device, core::ptr::null_mut(), 0);
    wined3d_context_map_bo_address(context, core::ptr::addr_of!((*op).addr), (*op).size, (*op).flags);
    context_release(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_cs_map_bo_address(cs: *mut wined3d_cs, addr: *mut wined3d_bo_address, size: size_t, flags: ::core::ffi::c_uint) {
    let context: *mut wined3d_device_context = core::ptr::addr_of_mut!((*cs).c);
    let mut op: *mut wined3d_cs_map_bo_address = core::ptr::null_mut::<wined3d_cs_map_bo_address>();

    op = wined3d_device_context_require_space(
        context,
        core::mem::size_of::<wined3d_cs_map_bo_address>() as size_t,
        WINED3D_CS_QUEUE_MAP,
    ) as *mut wined3d_cs_map_bo_address;
    (*op).opcode = WINED3D_CS_OP_MAP_BO_ADDRESS;
    (*op).addr = *addr;
    (*op).size = size;
    (*op).flags = flags as uint32_t;
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_MAP);
    wined3d_device_context_finish(context, WINED3D_CS_QUEUE_MAP);
}

unsafe extern "C" fn wined3d_cs_exec_blt_sub_resource(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let op: *const wined3d_cs_blt_sub_resource = data as *const wined3d_cs_blt_sub_resource;

    if (*(*op).dst_resource).type_ as u32 == WINED3D_RTYPE_BUFFER {
        wined3d_buffer_copy(
            buffer_from_resource((*op).dst_resource),
            (*op).dst_box.left,
            buffer_from_resource((*op).src_resource),
            (*op).src_box.left,
            (*op).src_box.right.wrapping_sub((*op).src_box.left),
        );
    } else if (*(*op).dst_resource).type_ as u32 == WINED3D_RTYPE_TEXTURE_3D {
        let mut src_texture: *mut wined3d_texture = core::ptr::null_mut::<wined3d_texture>();
        let mut dst_texture: *mut wined3d_texture = core::ptr::null_mut::<wined3d_texture>();
        let mut level: u32 = 0;
        let mut update_w: u32 = 0;
        let mut update_h: u32 = 0;
        let mut update_d: u32 = 0;
        let mut row_pitch: u32 = 0;
        let mut slice_pitch: u32 = 0;
        let mut context: *mut wined3d_context = core::ptr::null_mut::<wined3d_context>();
        let mut addr: wined3d_bo_address = wined3d_bo_address {
            buffer_object: core::ptr::null_mut::<wined3d_bo>(),
            addr: core::ptr::null_mut::<BYTE>(),
        };
        let mut location: u32 = 0;

        if (*op).flags & !WINED3D_BLT_RAW != 0 {
            FIXME!(
                "Flags %#x not implemented for %s resources.\n",
                (*op).flags,
                debug_d3dresourcetype((*(*op).dst_resource).type_)
            );
            return;
        }

        if (*op).flags & WINED3D_BLT_RAW == 0
            && (*(*op).src_resource).format != (*(*op).dst_resource).format
        {
            FIXME!(
                "Format conversion not implemented for %s resources.\n",
                debug_d3dresourcetype((*(*op).dst_resource).type_)
            );
            return;
        }

        update_w = (*op).dst_box.right.wrapping_sub((*op).dst_box.left) as u32;
        update_h = (*op).dst_box.bottom.wrapping_sub((*op).dst_box.top) as u32;
        update_d = (*op).dst_box.back.wrapping_sub((*op).dst_box.front) as u32;
        if (*op).src_box.right.wrapping_sub((*op).src_box.left) != update_w
            || (*op).src_box.bottom.wrapping_sub((*op).src_box.top) != update_h
            || (*op).src_box.back.wrapping_sub((*op).src_box.front) != update_d
        {
            FIXME!(
                "Stretching not implemented for %s resources.\n",
                debug_d3dresourcetype((*(*op).dst_resource).type_)
            );
            return;
        }

        dst_texture = texture_from_resource((*op).dst_resource);
        src_texture = texture_from_resource((*op).src_resource);

        context = context_acquire(
            (*cs).c.device,
            core::ptr::null_mut::<wined3d_texture>(),
            0,
        );

        location = (*src_texture).resource.map_binding as u32;
        if location == WINED3D_LOCATION_SYSMEM
            && wined3d_texture_can_use_pbo(
                src_texture,
                core::ptr::addr_of_mut!((*(*(*context).device).adapter).d3d_info),
            ) != 0
        {
            location = WINED3D_LOCATION_BUFFER;
        }

        if wined3d_texture_load_location(
            src_texture,
            (*op).src_sub_resource_idx,
            context,
            location,
        ) == 0
        {
            ERR!(
                "Failed to load source sub-resource into %s.\n",
                wined3d_debug_location(location)
            );
            context_release(context);
            return;
        }

        level = (*op).dst_sub_resource_idx.wrapping_rem((*dst_texture).level_count);
        if update_w == wined3d_texture_get_level_width(dst_texture, level)
            && update_h == wined3d_texture_get_level_height(dst_texture, level)
            && update_d == wined3d_texture_get_level_depth(dst_texture, level)
        {
            wined3d_texture_prepare_location(
                dst_texture,
                (*op).dst_sub_resource_idx,
                context,
                WINED3D_LOCATION_TEXTURE_RGB,
            );
        } else if wined3d_texture_load_location(
            dst_texture,
            (*op).dst_sub_resource_idx,
            context,
            WINED3D_LOCATION_TEXTURE_RGB,
        ) == 0
        {
            ERR!("Failed to load destination sub-resource.\n");
            context_release(context);
            return;
        }

        wined3d_texture_get_bo_address(
            src_texture,
            (*op).src_sub_resource_idx,
            core::ptr::addr_of_mut!(addr),
            location,
        );
        wined3d_texture_get_pitch(
            src_texture,
            (*op).src_sub_resource_idx.wrapping_rem((*src_texture).level_count),
            core::ptr::addr_of_mut!(row_pitch),
            core::ptr::addr_of_mut!(slice_pitch),
        );

        (*(*dst_texture).texture_ops)
            .texture_upload_data
            .unwrap_unchecked()(
            context,
            wined3d_const_bo_address(core::ptr::addr_of_mut!(addr)),
            (*dst_texture).resource.format,
            core::ptr::addr_of!((*op).src_box),
            row_pitch,
            slice_pitch,
            dst_texture,
            (*op).dst_sub_resource_idx,
            WINED3D_LOCATION_TEXTURE_RGB,
            (*op).dst_box.left,
            (*op).dst_box.top,
            (*op).dst_box.front,
        );
        wined3d_texture_validate_location(
            dst_texture,
            (*op).dst_sub_resource_idx,
            WINED3D_LOCATION_TEXTURE_RGB,
        );
        wined3d_texture_invalidate_location(
            dst_texture,
            (*op).dst_sub_resource_idx,
            !WINED3D_LOCATION_TEXTURE_RGB,
        );

        context_release(context);
    } else if texture2d_blt(
        texture_from_resource((*op).dst_resource),
        (*op).dst_sub_resource_idx,
        core::ptr::addr_of!((*op).dst_box),
        texture_from_resource((*op).src_resource),
        (*op).src_sub_resource_idx,
        core::ptr::addr_of!((*op).src_box),
        (*op).flags,
        core::ptr::addr_of!((*op).fx),
        (*op).filter,
    ) < 0
    {
        FIXME!("Blit failed.\n");
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_blt_sub_resource(mut context: *mut wined3d_device_context, mut dst_resource: *mut wined3d_resource, mut dst_sub_resource_idx: ::core::ffi::c_uint, mut dst_box: *const wined3d_box, mut src_resource: *mut wined3d_resource, mut src_sub_resource_idx: ::core::ffi::c_uint, mut src_box: *const wined3d_box, mut flags: ::core::ffi::c_uint, mut fx: *const wined3d_blt_fx, mut filter: wined3d_texture_filter_type) {
    let mut op: *mut wined3d_cs_blt_sub_resource =
        ::core::ptr::null_mut::<wined3d_cs_blt_sub_resource>();
    if (*dst_resource).type_ as ::core::ffi::c_uint
        == WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint
        && (*dst_box).right.wrapping_sub((*dst_box).left) == (*dst_resource).size
    {
        invalidate_client_address(dst_resource);
    }
    op = wined3d_device_context_require_space(
        context,
        ::core::mem::size_of::<wined3d_cs_blt_sub_resource>() as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_blt_sub_resource;
    (*op).opcode = WINED3D_CS_OP_BLT_SUB_RESOURCE;
    (*op).dst_resource = dst_resource;
    (*op).dst_sub_resource_idx = dst_sub_resource_idx;
    (*op).dst_box = *dst_box;
    (*op).src_resource = src_resource;
    (*op).src_sub_resource_idx = src_sub_resource_idx;
    (*op).src_box = *src_box;
    (*op).flags = flags as uint32_t;
    if !fx.is_null() {
        (*op).fx = *fx;
    } else {
        memset(
            core::ptr::addr_of_mut!((*op).fx) as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<wined3d_blt_fx>() as size_t,
        );
    }
    (*op).filter = filter;
    wined3d_device_context_reference_resource(context, dst_resource);
    if !src_resource.is_null() {
        wined3d_device_context_reference_resource(context, src_resource);
    }
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
    if flags & WINED3D_BLT_SYNCHRONOUS as ::core::ffi::c_uint != 0 {
        wined3d_device_context_finish(context, WINED3D_CS_QUEUE_DEFAULT);
    }
}

unsafe extern "C" fn wined3d_cs_exec_update_sub_resource(mut cs: *mut wined3d_cs, mut data: *const ::core::ffi::c_void) {
    let mut op: *const wined3d_cs_update_sub_resource = data as *const wined3d_cs_update_sub_resource;
    let mut resource: *mut wined3d_resource = (*op).resource;
    let mut box_0: *const wined3d_box = &raw const (*op).box_;
    let mut context: *mut wined3d_context = ::core::ptr::null_mut::<wined3d_context>();

    context = context_acquire(
        (*cs).c.device,
        ::core::ptr::null_mut::<wined3d_texture>(),
        0,
    );

    if (*resource).type_ as ::core::ffi::c_uint == WINED3D_RTYPE_BUFFER {
        wined3d_buffer_update_sub_resource(
            buffer_from_resource(resource),
            context,
            &raw const (*op).bo,
            (*box_0).left as ::core::ffi::c_uint,
            ((*box_0).right as ::core::ffi::c_uint).wrapping_sub((*box_0).left as ::core::ffi::c_uint),
        );
    } else {
        wined3d_texture_update_sub_resource(
            texture_from_resource(resource),
            (*op).sub_resource_idx,
            context,
            &raw const (*op).bo,
            box_0,
            (*op).row_pitch,
            (*op).slice_pitch,
        );
    }

    context_release(context);

    if ((*op).bo.flags & UPLOAD_BO_FREE_ON_UNMAP) != 0 {
        if !(*op).bo.addr.buffer_object.is_null() {
            FIXME!(
                "Free BO address %s.\n",
                debug_const_bo_address(&raw const (*op).bo.addr)
            );
        } else {
            free((*op).bo.addr.addr as *mut ::core::ffi::c_void);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_update_sub_resource(mut context: *mut wined3d_device_context, mut resource: *mut wined3d_resource, mut sub_resource_idx: ::core::ffi::c_uint, mut box_0: *const wined3d_box, mut data: *const ::core::ffi::c_void, mut row_pitch: ::core::ffi::c_uint, mut slice_pitch: ::core::ffi::c_uint) {
    let mut op: *mut wined3d_cs_update_sub_resource =
        ::core::ptr::null_mut::<wined3d_cs_update_sub_resource>();
    let mut map_desc: wined3d_map_desc = wined3d_map_desc {
        row_pitch: 0,
        slice_pitch: 0,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut dummy_box: wined3d_box = wined3d_box {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        front: 0,
        back: 0,
    };
    let mut bo: upload_bo = upload_bo {
        addr: wined3d_const_bo_address {
            buffer_object: ::core::ptr::null_mut::<wined3d_bo>(),
            addr: ::core::ptr::null::<BYTE>(),
        },
        flags: 0,
    };
    if (*resource).type_ as ::core::ffi::c_uint
        == WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint
        && (*box_0).right.wrapping_sub((*box_0).left) == (*resource).size
    {
        invalidate_client_address(resource);
    }
    if (*(*context).ops)
        .map_upload_bo
        .unwrap_unchecked()(
        context,
        resource,
        sub_resource_idx,
        &raw mut map_desc,
        box_0,
        WINED3D_MAP_WRITE as uint32_t,
    ) {
        wined3d_format_copy_data(
            (*resource).format,
            data as *const uint8_t,
            row_pitch,
            slice_pitch,
            map_desc.data as *mut uint8_t,
            map_desc.row_pitch as ::core::ffi::c_uint,
            map_desc.slice_pitch as ::core::ffi::c_uint,
            ((*box_0).right as ::core::ffi::c_uint)
                .wrapping_sub((*box_0).left as ::core::ffi::c_uint),
            ((*box_0).bottom as ::core::ffi::c_uint)
                .wrapping_sub((*box_0).top as ::core::ffi::c_uint),
            ((*box_0).back as ::core::ffi::c_uint)
                .wrapping_sub((*box_0).front as ::core::ffi::c_uint),
        );
        (*(*context).ops)
            .unmap_upload_bo
            .unwrap_unchecked()(
            context,
            resource,
            sub_resource_idx,
            &raw mut dummy_box,
            &raw mut bo,
        );
        wined3d_device_context_upload_bo(
            context,
            resource,
            sub_resource_idx,
            box_0,
            &raw mut bo,
            map_desc.row_pitch as ::core::ffi::c_uint,
            map_desc.slice_pitch as ::core::ffi::c_uint,
        );
        return;
    }
    wined3d_resource_wait_idle(resource);
    op = wined3d_device_context_require_space(
        context,
        ::core::mem::size_of::<wined3d_cs_update_sub_resource>() as size_t,
        WINED3D_CS_QUEUE_MAP,
    ) as *mut wined3d_cs_update_sub_resource;
    (*op).opcode = WINED3D_CS_OP_UPDATE_SUB_RESOURCE;
    (*op).resource = resource;
    (*op).sub_resource_idx = sub_resource_idx;
    (*op).box_ = *box_0;
    (*op).bo.addr.buffer_object = core::ptr::null_mut();
    (*op).bo.addr.addr = data as *const BYTE;
    (*op).bo.flags = 0 as uint32_t;
    (*op).row_pitch = row_pitch;
    (*op).slice_pitch = slice_pitch;
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_MAP);
    wined3d_device_context_finish(context, WINED3D_CS_QUEUE_MAP);
}

unsafe extern "C" fn wined3d_cs_exec_add_dirty_texture_region(mut cs: *mut wined3d_cs, mut data: *const core::ffi::c_void) {
    let mut op: *const wined3d_cs_add_dirty_texture_region = data as *const wined3d_cs_add_dirty_texture_region;
    let mut texture: *mut wined3d_texture = (*op).texture;
    let mut sub_resource_idx: u32 = 0;
    let mut i: u32 = 0;
    let mut context: *mut wined3d_context = core::ptr::null_mut();

    context = context_acquire((*cs).c.device, core::ptr::null_mut(), 0);
    sub_resource_idx = (*op).layer.wrapping_mul((*texture).level_count);
    i = 0;
    while i < (*texture).level_count {
        if wined3d_texture_load_location(
            texture,
            sub_resource_idx,
            context,
            (*texture).resource.map_binding as uint32_t,
        ) != 0
        {
            wined3d_texture_invalidate_location(
                texture,
                sub_resource_idx,
                !((*texture).resource.map_binding as i32) as uint32_t,
            );
        } else {
            ERR!("Failed to load location %s.\n", wined3d_debug_location((*texture).resource.map_binding as uint32_t));
        }
        i = i.wrapping_add(1);
        sub_resource_idx = sub_resource_idx.wrapping_add(1);
    }
    context_release(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_cs_emit_add_dirty_texture_region(mut cs: *mut wined3d_cs, mut texture: *mut wined3d_texture, mut layer: ::core::ffi::c_uint) {
    let mut op: *mut wined3d_cs_add_dirty_texture_region =
        ::core::ptr::null_mut::<wined3d_cs_add_dirty_texture_region>();
    op = wined3d_device_context_require_space(
        &raw mut (*cs).c,
        ::core::mem::size_of::<wined3d_cs_add_dirty_texture_region>() as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_add_dirty_texture_region;
    (*op).opcode = WINED3D_CS_OP_ADD_DIRTY_TEXTURE_REGION;
    (*op).texture = texture;
    (*op).layer = layer;
    wined3d_resource_reference(&raw mut (*texture).resource);
    wined3d_device_context_submit(&raw mut (*cs).c, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_clear_unordered_access_view(
    mut cs: *mut wined3d_cs,
    mut data: *const ::core::ffi::c_void,
) {
    let mut op: *const wined3d_cs_clear_unordered_access_view =
        data as *const wined3d_cs_clear_unordered_access_view;
    let mut view: *mut wined3d_unordered_access_view = (*op).view;
    let mut context: *mut wined3d_context = ::core::ptr::null_mut::<wined3d_context>();
    context = context_acquire(
        (*cs).c.device,
        ::core::ptr::null_mut::<wined3d_texture>(),
        0,
    );
    (*(*(*(*cs).c.device).adapter).adapter_ops)
        .adapter_clear_uav
        .unwrap_unchecked()(
        context, view, &raw const (*op).clear_value, (*op).fp
    );
    context_release(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_clear_uav(context: *mut wined3d_device_context, view: *mut wined3d_unordered_access_view, clear_value: *const wined3d_uvec4, fp: bool) {
    let mut op: *mut wined3d_cs_clear_unordered_access_view = core::ptr::null_mut();

    op = wined3d_device_context_require_space(
        context,
        core::mem::size_of::<wined3d_cs_clear_unordered_access_view>(),
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_clear_unordered_access_view;
    (*op).opcode = WINED3D_CS_OP_CLEAR_UNORDERED_ACCESS_VIEW;
    (*op).view = view;
    (*op).clear_value = *clear_value;
    (*op).fp = fp;

    wined3d_device_context_reference_resource(context, (*view).resource);

    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_copy_uav_counter(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let op: *const wined3d_cs_copy_uav_counter = data as *const wined3d_cs_copy_uav_counter;
    let view: *mut wined3d_unordered_access_view = (*op).view;
    let mut context: *mut wined3d_context = core::ptr::null_mut();
    context = context_acquire((*cs).c.device, core::ptr::null_mut(), 0);
    wined3d_unordered_access_view_copy_counter(view, (*op).buffer, (*op).offset, context);
    context_release(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_copy_uav_counter(mut context: *mut wined3d_device_context, mut dst_buffer: *mut wined3d_buffer, mut offset: ::core::ffi::c_uint, mut uav: *mut wined3d_unordered_access_view) {
    let mut op: *mut wined3d_cs_copy_uav_counter =
        ::core::ptr::null_mut::<wined3d_cs_copy_uav_counter>();
    op = wined3d_device_context_require_space(
        context,
        ::core::mem::size_of::<wined3d_cs_copy_uav_counter>() as size_t,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_copy_uav_counter;
    (*op).opcode = WINED3D_CS_OP_COPY_UAV_COUNTER;
    (*op).buffer = dst_buffer;
    (*op).offset = offset;
    (*op).view = uav;
    wined3d_device_context_reference_resource(context, core::ptr::addr_of_mut!((*dst_buffer).resource));
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_exec_generate_mipmaps(mut cs: *mut wined3d_cs, mut data: *const core::ffi::c_void) {
    let mut op: *const wined3d_cs_generate_mipmaps = data as *const wined3d_cs_generate_mipmaps;
    let mut view: *mut wined3d_shader_resource_view = (*op).view;
    let mut context: *mut wined3d_context = core::ptr::null_mut::<wined3d_context>();
    context = context_acquire(
        (*cs).c.device,
        core::ptr::null_mut::<wined3d_texture>(),
        0,
    );
    (*(*(*(*cs).c.device).adapter).adapter_ops)
        .adapter_generate_mipmap
        .unwrap_unchecked()(context, view);
    context_release(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_generate_mipmaps(context: *mut wined3d_device_context, view: *mut wined3d_shader_resource_view) {
    let mut op: *mut wined3d_cs_generate_mipmaps =
        core::ptr::null_mut::<wined3d_cs_generate_mipmaps>();
    op = wined3d_device_context_require_space(
        context,
        core::mem::size_of::<wined3d_cs_generate_mipmaps>(),
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_generate_mipmaps;
    (*op).opcode = WINED3D_CS_OP_GENERATE_MIPMAPS;
    (*op).view = view;

    wined3d_device_context_reference_resource(context, (*view).resource);

    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_emit_stop(cs: *mut wined3d_cs) {
    let mut op: *mut wined3d_cs_stop = core::ptr::null_mut();
    op = wined3d_device_context_require_space(
        core::ptr::addr_of_mut!((*cs).c),
        core::mem::size_of::<wined3d_cs_stop>(),
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_stop;
    (*op).opcode = WINED3D_CS_OP_STOP;

    wined3d_device_context_submit(core::ptr::addr_of_mut!((*cs).c), WINED3D_CS_QUEUE_DEFAULT);
    wined3d_cs_finish(cs, WINED3D_CS_QUEUE_DEFAULT);
}

unsafe extern "C" fn wined3d_cs_reference_resource(
    mut context: *mut wined3d_device_context,
    mut resource: *mut wined3d_resource,
) {
    wined3d_resource_reference(resource);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_emit_execute_command_list(context: *mut wined3d_device_context, list: *mut wined3d_command_list, restore_state: bool) {
    let mut op: *mut wined3d_cs_execute_command_list = core::ptr::null_mut::<wined3d_cs_execute_command_list>();
    op = wined3d_device_context_require_space(context, core::mem::size_of::<wined3d_cs_execute_command_list>() as size_t, WINED3D_CS_QUEUE_DEFAULT) as *mut wined3d_cs_execute_command_list;
    (*op).opcode = WINED3D_CS_OP_EXECUTE_COMMAND_LIST;
    (*op).list = list;
    ((*(*context).ops).reference_command_list.unwrap_unchecked())(context, list);
    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);
    if restore_state {
        wined3d_device_context_set_state(context, (*context).state);
    } else {
        wined3d_device_context_reset_state(context);
    }
}

unsafe extern "C" fn wined3d_cs_st_require_space(
    mut context: *mut wined3d_device_context,
    mut size: size_t,
    mut queue_id: wined3d_cs_queue_id,
) -> *mut ::core::ffi::c_void {
    let mut cs: *mut wined3d_cs = wined3d_cs_from_context(context);

    if size > (*cs).data_size.wrapping_sub((*cs).end) {
        let mut new_size: size_t = 0;
        let mut new_data: *mut ::core::ffi::c_void = core::ptr::null_mut::<::core::ffi::c_void>();

        new_size = max!(size, (*cs).data_size.wrapping_mul(2));
        if (*cs).end == 0 {
            new_data = realloc((*cs).data, new_size);
        } else {
            new_data = malloc(new_size);
        }
        if new_data.is_null() {
            return core::ptr::null_mut();
        }

        (*cs).data_size = new_size;
        (*cs).end = 0;
        (*cs).start = (*cs).end;
        (*cs).data = new_data;
    }

    (*cs).end = (*cs).end.wrapping_add(size);

    return ((*cs).data as *mut BYTE).offset((*cs).start as isize) as *mut ::core::ffi::c_void;
}

unsafe extern "C" fn wined3d_cs_st_submit(context: *mut wined3d_device_context, queue_id: wined3d_cs_queue_id) {
    let cs: *mut wined3d_cs = wined3d_cs_from_context(context);
    let opcode: wined3d_cs_op;
    let start: usize;
    let data: *mut BYTE;

    data = (*cs).data as *mut BYTE;
    start = (*cs).start;
    (*cs).start = (*cs).end;

    opcode = *(data.add(start) as *const wined3d_cs_op);
    if opcode as u32 >= (WINED3D_CS_OP_STOP as u32) {
        ERR!("Invalid opcode %#x.\n", opcode);
    } else {
        (wined3d_cs_op_handlers[opcode as usize].unwrap_unchecked())(cs, data.add(start) as *const core::ffi::c_void);
    }

    if (*cs).data == data as *mut core::ffi::c_void {
        (*cs).start = start;
        (*cs).end = start;
    } else if start == 0 {
        free(data as *mut core::ffi::c_void);
    }
}

unsafe extern "C" fn wined3d_cs_st_finish(
    context: *mut wined3d_device_context,
    queue_id: wined3d_cs_queue_id,
) {}

unsafe extern "C" fn get_map_pitch(
    format: *const wined3d_format,
    box_0: *const wined3d_box,
    map_desc: *mut wined3d_map_desc,
    size: *mut size_t,
) {
    let height: ::core::ffi::c_uint =
        ((*box_0).bottom as ::core::ffi::c_uint).wrapping_sub((*box_0).top as ::core::ffi::c_uint);
    let width: ::core::ffi::c_uint =
        ((*box_0).right as ::core::ffi::c_uint).wrapping_sub((*box_0).left as ::core::ffi::c_uint);
    let depth: ::core::ffi::c_uint =
        ((*box_0).back as ::core::ffi::c_uint).wrapping_sub((*box_0).front as ::core::ffi::c_uint);
    wined3d_format_calculate_pitch(
        format,
        1 as ::core::ffi::c_uint,
        width,
        height,
        core::ptr::addr_of_mut!((*map_desc).row_pitch),
        core::ptr::addr_of_mut!((*map_desc).slice_pitch),
    );
    *size = (depth as UINT)
        .wrapping_sub(1 as UINT)
        .wrapping_mul((*map_desc).slice_pitch)
        .wrapping_add(
            (height as UINT)
                .wrapping_sub(1 as UINT)
                .wrapping_div((*format).block_height)
                .wrapping_mul((*map_desc).row_pitch),
        )
        .wrapping_add(
            (width as UINT)
                .wrapping_add((*format).block_width)
                .wrapping_sub(1 as UINT)
                .wrapping_div((*format).block_width)
                .wrapping_mul((*format).block_byte_count),
        ) as size_t;
}

unsafe extern "C" fn wined3d_cs_map_upload_bo(
    context: *mut wined3d_device_context,
    resource: *mut wined3d_resource,
    sub_resource_idx: ::core::ffi::c_uint,
    map_desc: *mut wined3d_map_desc,
    box_0: *const wined3d_box,
    mut flags: uint32_t,
) -> bool {
    let client: *mut wined3d_client_resource = core::ptr::addr_of_mut!((*resource).client);
    let format: *const wined3d_format = (*resource).format;
    let mut size: size_t = 0;

    if flags & (WINED3D_MAP_DISCARD | WINED3D_MAP_NOOVERWRITE) != 0 {
        let device: *mut wined3d_device = (*context).device;
        let mut addr: wined3d_bo_address = wined3d_bo_address {
            buffer_object: core::ptr::null_mut::<wined3d_bo>(),
            addr: core::ptr::null_mut::<BYTE>(),
        };
        let mut bo: *mut wined3d_bo = core::ptr::null_mut::<wined3d_bo>();
        let mut map_ptr: *mut uint8_t = core::ptr::null_mut::<uint8_t>();

        if (*resource).pin_sysmem() != 0 {
            return false;
        }

        if (flags & WINED3D_MAP_NOOVERWRITE != 0)
            && (*client).addr.buffer_object == CLIENT_BO_DISCARDED
        {
            flags = (flags & !WINED3D_MAP_NOOVERWRITE) | WINED3D_MAP_DISCARD;
        }

        if flags & WINED3D_MAP_DISCARD != 0 {
            if !(*(*(*device).adapter).adapter_ops)
                .adapter_alloc_bo
                .unwrap_unchecked()(
                device,
                resource,
                sub_resource_idx,
                core::ptr::addr_of_mut!(addr),
            ) {
                return false;
            }

            /* Limit NOOVERWRITE maps to buffers for now; there are too many
             * ways that a texture can be invalidated to even count. */
            if (*resource).type_ as ::core::ffi::c_uint
                == WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint
            {
                (*client).addr = addr;
            }
        } else {
            addr = (*client).addr;
        }

        map_ptr = core::ptr::null_mut::<uint8_t>();
        bo = addr.buffer_object;
        if !bo.is_null() {
            wined3d_device_bo_map_lock(device);
            map_ptr = (*bo).map_ptr as *mut uint8_t;
            if !map_ptr.is_null() {
                (*bo).client_map_count = (*bo).client_map_count.wrapping_add(1);
            }
            wined3d_device_bo_map_unlock(device);

            if map_ptr.is_null() {
                /* adapter_alloc_bo() should have given us a mapped BO if we are
                 * discarding. */
                assert!((((flags & WINED3D_MAP_NOOVERWRITE) as ::core::ffi::c_int) != 0));
                WINE_WARN_!(__wine_dbch_d3d_perf,
                    b"Not accelerating a NOOVERWRITE map because the BO is not mapped.\n\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
                return false;
            }
        }
        map_ptr = map_ptr.offset(addr.addr as uintptr_t as isize);

        if map_ptr.is_null() {
            assert!((((flags & WINED3D_MAP_NOOVERWRITE) as ::core::ffi::c_int) != 0));
            WINE_WARN_!(__wine_dbch_d3d_perf,
                b"Not accelerating a NOOVERWRITE map because the sub-resource has no valid address.\n\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
            return false;
        }

        wined3d_resource_get_sub_resource_map_pitch(
            resource,
            sub_resource_idx,
            core::ptr::addr_of_mut!((*map_desc).row_pitch),
            core::ptr::addr_of_mut!((*map_desc).slice_pitch),
        );

        (*client).mapped_upload.addr = *wined3d_const_bo_address(core::ptr::addr_of_mut!(addr));
        (*client).mapped_upload.flags = 0;
        if !bo.is_null() {
            map_ptr = map_ptr.offset((*bo).memory_offset as isize);
            /* If we are not mapping all buffers persistently, use
             * UPDATE_SUB_RESOURCE as a means of telling the CS thread to try
             * to unmap the resource, so that we can free VA space. */
            if !(*bo).coherent != false || !wined3d_map_persistent() {
                (*client).mapped_upload.flags |= UPLOAD_BO_UPLOAD_ON_UNMAP;
            }
        }
        (*map_desc).data =
            resource_offset_map_pointer(resource, sub_resource_idx, map_ptr, box_0);

        if flags & WINED3D_MAP_DISCARD != 0 {
            (*client).mapped_upload.flags |= UPLOAD_BO_UPLOAD_ON_UNMAP | UPLOAD_BO_RENAME_ON_UNMAP;
        }

        (*client).mapped_box = *box_0;
        return true;
    }

    get_map_pitch(
        format,
        box_0,
        map_desc,
        core::ptr::addr_of_mut!(size),
    );

    (*map_desc).data = malloc(size);
    if (*map_desc).data.is_null() {
        WINE_WARN_!(__wine_dbch_d3d_perf,
            b"Failed to allocate a heap memory buffer.\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return false;
    }
    (*client).mapped_upload.addr.buffer_object = core::ptr::null_mut();
    (*client).mapped_upload.addr.addr = (*map_desc).data as *const BYTE;
    (*client).mapped_upload.flags = UPLOAD_BO_UPLOAD_ON_UNMAP | UPLOAD_BO_FREE_ON_UNMAP;
    (*client).mapped_box = *box_0;
    return true;
}

unsafe extern "C" fn wined3d_bo_address_is_null(addr: *mut wined3d_const_bo_address) -> bool {
    return (*addr).buffer_object.is_null() && (*addr).addr.is_null();
}

unsafe extern "C" fn wined3d_cs_unmap_upload_bo(
    context: *mut wined3d_device_context,
    resource: *mut wined3d_resource,
    sub_resource_idx: ::core::ffi::c_uint,
    box_0: *mut wined3d_box,
    upload_bo: *mut upload_bo,
) -> bool {
    let client: *mut wined3d_client_resource = core::ptr::addr_of_mut!((*resource).client);
    let device: *mut wined3d_device = (*context).device;
    let mut bo: *mut wined3d_bo = core::ptr::null_mut::<wined3d_bo>();

    if wined3d_bo_address_is_null(core::ptr::addr_of_mut!((*client).mapped_upload.addr)) {
        return false;
    }

    bo = (*client).mapped_upload.addr.buffer_object;
    if !bo.is_null() {
        wined3d_device_bo_map_lock(device);
        (*bo).client_map_count = (*bo).client_map_count.wrapping_sub(1);
        wined3d_device_bo_map_unlock(device);
    }

    *upload_bo = (*client).mapped_upload;
    *box_0 = (*client).mapped_box;
    memset(
        core::ptr::addr_of_mut!((*client).mapped_upload) as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<upload_bo>() as usize,
    );
    memset(
        core::ptr::addr_of_mut!((*client).mapped_box) as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<wined3d_box>() as usize,
    );
    return true;
}

unsafe extern "C" fn wined3d_cs_queue_is_empty(
    cs: *const wined3d_cs,
    queue: *const wined3d_cs_queue,
) -> BOOL {
    wined3d_from_cs(cs);
    return (core::ptr::read_volatile(core::ptr::addr_of!((*queue).head) as *const ULONG) == (*queue).tail) as BOOL;
}

unsafe extern "C" fn wined3d_cs_queue_submit(mut queue: *mut wined3d_cs_queue, mut cs: *mut wined3d_cs) {
    let mut packet: *mut wined3d_cs_packet = core::ptr::null_mut();
    let mut packet_size: size_t = 0;
    packet = (core::ptr::addr_of_mut!((*queue).data) as *mut BYTE)
        .offset(((*queue).head as ::core::ffi::c_uint & WINED3D_CS_QUEUE_MASK) as isize)
        as *mut BYTE as *mut wined3d_cs_packet;
    TRACE!(
        "Queuing op %s at %p.\n",
        debug_cs_op(*(core::ptr::addr_of_mut!((*packet).data) as *mut BYTE as *const wined3d_cs_op)),
        packet
    );
    packet_size = (FIELD_OFFSET!(wined3d_cs_packet, data[(*packet).size as usize]) as usize)
        as size_t;
    InterlockedExchange(
        core::ptr::addr_of_mut!((*queue).head) as *mut LONG,
        ((*queue).head as size_t).wrapping_add(packet_size) as LONG,
    );
    if InterlockedCompareExchange(
        core::ptr::addr_of_mut!((*cs).waiting_for_event) as *mut LONG,
        (FALSE as i32),
        (TRUE as i32),
    ) != 0
    {
        if pNtAlertThreadByThreadId.is_some() {
            (pNtAlertThreadByThreadId.unwrap_unchecked())((*cs).thread_id as ULONG_PTR as HANDLE);
        } else {
            SetEvent((*cs).event);
        }
    }
}

unsafe extern "C" fn wined3d_cs_mt_submit(context: *mut wined3d_device_context, queue_id: wined3d_cs_queue_id) {
    let mut cs: *mut wined3d_cs = wined3d_cs_from_context(context);

    if (*cs).thread_id == GetCurrentThreadId() {
        return wined3d_cs_st_submit(context, queue_id);
    }

    wined3d_cs_queue_submit(
        (&raw mut (*cs).queue as *mut wined3d_cs_queue).offset(queue_id as isize) as *mut wined3d_cs_queue,
        cs,
    );
}

unsafe extern "C" fn wined3d_cs_queue_require_space(queue: *mut wined3d_cs_queue, mut size: size_t, cs: *mut wined3d_cs) -> *mut core::ffi::c_void {
    let mut queue_size: size_t = (ARRAY_SIZE!((*queue).data)) as usize;
    let mut header_size: size_t = 0;
    let mut packet_size: size_t = 0;
    let mut remaining: size_t = 0;
    let mut packet: *mut wined3d_cs_packet = core::ptr::null_mut();
    let mut head: ULONG = (*queue).head & WINED3D_CS_QUEUE_MASK;

    header_size = FIELD_OFFSET!(wined3d_cs_packet, data[0]) as size_t;
    packet_size = FIELD_OFFSET!(wined3d_cs_packet, data[size]) as size_t;
    packet_size = (packet_size.wrapping_add(header_size).wrapping_sub(1 as size_t))
        & !(header_size.wrapping_sub(1 as size_t));
    size = packet_size.wrapping_sub(header_size);
    if packet_size >= WINED3D_CS_QUEUE_SIZE as size_t {
        ERR!("Packet size %Iu >= queue size %u.\n", packet_size, WINED3D_CS_QUEUE_SIZE);
        return core::ptr::null_mut();
    }

    remaining = queue_size.wrapping_sub(head as size_t);
    if remaining < packet_size {
        let mut nop_size: size_t = remaining.wrapping_sub(header_size);
        let mut nop: *mut wined3d_cs_nop = core::ptr::null_mut();

        nop = wined3d_cs_queue_require_space(queue, nop_size, cs) as *mut wined3d_cs_nop;
        if nop_size != 0 {
            (*nop).opcode = WINED3D_CS_OP_NOP;
        }

        wined3d_cs_queue_submit(queue, cs);
        head = (*queue).head & WINED3D_CS_QUEUE_MASK;
        assert!(head == 0);
    }

    loop {
        let tail: ULONG = core::ptr::read_volatile(core::ptr::addr_of!((*queue).tail)) & WINED3D_CS_QUEUE_MASK;
        let mut new_pos: ULONG = 0;

        /* Empty. */
        if head == tail {
            break;
        }
        new_pos = (head as size_t).wrapping_add(packet_size) as ULONG & WINED3D_CS_QUEUE_MASK;
        /* Head ahead of tail. We checked the remaining size above, so we only
         * need to make sure we don't make head equal to tail. */
        if head > tail && (new_pos != tail) {
            break;
        }
        /* Tail ahead of head. Make sure the new head is before the tail as
         * well. Note that new_pos is 0 when it's at the end of the queue. */
        if new_pos < tail && new_pos != 0 {
            break;
        }

        WINE_TRACE_!(__wine_dbch_d3d_perf,"Waiting for free space. Head %lu, tail %lu, packet size %Iu.\n",
                head, tail, packet_size);
    }

    packet = (core::ptr::addr_of_mut!((*queue).data) as *mut BYTE).offset(head as isize) as *mut wined3d_cs_packet;
    (*packet).size = size;
    return core::ptr::addr_of_mut!((*packet).data) as *mut core::ffi::c_void;
}

unsafe extern "C" fn wined3d_cs_mt_require_space(
    mut context: *mut wined3d_device_context,
    mut size: size_t,
    mut queue_id: wined3d_cs_queue_id,
) -> *mut ::core::ffi::c_void {
    let mut cs: *mut wined3d_cs = wined3d_cs_from_context(context);

    if (*cs).thread_id == GetCurrentThreadId() {
        return wined3d_cs_st_require_space(context, size, queue_id);
    }

    return wined3d_cs_queue_require_space(
        (&raw mut (*cs).queue as *mut wined3d_cs_queue).offset(queue_id as isize) as *mut wined3d_cs_queue,
        size,
        cs,
    );
}

unsafe extern "C" fn wined3d_cs_mt_finish(context: *mut wined3d_device_context, queue_id: wined3d_cs_queue_id) {
    let cs: *mut wined3d_cs = wined3d_cs_from_context(context);
    let mut spin_count: ::core::ffi::c_uint = 0;

    if (*cs).thread_id == GetCurrentThreadId() {
        return wined3d_cs_st_finish(context, queue_id);
    }

    WINE_TRACE_!(__wine_dbch_d3d_perf,"Waiting for queue %u to be empty.\n", queue_id as ::core::ffi::c_uint);
    while (*cs).queue[queue_id as usize].head
        != core::ptr::read_volatile(core::ptr::addr_of!((*cs).queue[queue_id as usize].tail))
    {
        wined3d_pause(core::ptr::addr_of_mut!(spin_count));
    }
    WINE_TRACE_!(__wine_dbch_d3d_perf,"Queue is now empty.\n");
}

unsafe extern "C" fn poll_queries(cs: *mut wined3d_cs) {
    let mut query: *mut wined3d_query = ::core::ptr::null_mut::<wined3d_query>();
    let mut cursor: *mut wined3d_query = ::core::ptr::null_mut::<wined3d_query>();
    query = ((*cs).query_poll_list.next as *mut ::core::ffi::c_char).offset(
        -(core::mem::offset_of!(wined3d_query, poll_list_entry)
            as ::core::ffi::c_ulonglong as isize),
    ) as *mut wined3d_query;
    cursor = ((*query).poll_list_entry.next as *mut ::core::ffi::c_char).offset(
        -(core::mem::offset_of!(wined3d_query, poll_list_entry)
            as ::core::ffi::c_ulonglong as isize),
    ) as *mut wined3d_query;
    while &raw mut (*query).poll_list_entry != &raw mut (*cs).query_poll_list {
        if !((*(*query).query_ops)
            .query_poll
            .unwrap_unchecked()(query, 0)
            == 0)
        {
            list_remove(&raw mut (*query).poll_list_entry);
            list_init(&raw mut (*query).poll_list_entry);
            InterlockedIncrement(
                &raw mut (*query).counter_retrieved as *mut ::core::ffi::c_long
                    as *mut ::core::ffi::c_long,
            );
        }
        query = cursor;
        cursor = ((*query).poll_list_entry.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(wined3d_query, poll_list_entry)
                as ::core::ffi::c_ulonglong as isize),
        ) as *mut wined3d_query;
    }
}

unsafe extern "C" fn wined3d_cs_wait_event(cs: *mut wined3d_cs) {
    static mut query_timeout: LARGE_INTEGER = LARGE_INTEGER {
        QuadPart: (WINED3D_CS_COMMAND_WAIT_WITH_QUERIES_TIMEOUT as LONGLONG * -10) as LONGLONG,
    };
    let mut timeout: *const LARGE_INTEGER = core::ptr::null::<LARGE_INTEGER>();
    if list_empty(core::ptr::addr_of_mut!((*cs).query_poll_list)) == 0 {
        timeout = core::ptr::addr_of!(query_timeout);
    }
    InterlockedExchange(
        core::ptr::addr_of_mut!((*cs).waiting_for_event) as *mut core::ffi::c_long,
        TRUE as core::ffi::c_long,
    );
    /* The main thread might have enqueued a command and blocked on it after
     * the CS thread decided to enter wined3d_cs_wait_event(), but before
     * "waiting_for_event" was set.
     *
     * Likewise, we can race with the main thread when resetting
     * "waiting_for_event", in which case we would need to call
     * WaitForSingleObject() because the main thread called SetEvent().
     *
     * The `!` negates ONLY the emptiness test, as in the C. Spanning the CAS
     * too inverted it: an empty queue reset "waiting_for_event" and then slept,
     * so no submit ever signalled and the device hung on its first slow call. */
    if !(wined3d_cs_queue_is_empty(
            cs,
            core::ptr::addr_of_mut!((*cs).queue[WINED3D_CS_QUEUE_DEFAULT as usize]),
        ) != 0
        && wined3d_cs_queue_is_empty(
            cs,
            core::ptr::addr_of_mut!((*cs).queue[WINED3D_CS_QUEUE_MAP as usize]),
        ) != 0)
        && InterlockedCompareExchange(
            core::ptr::addr_of_mut!((*cs).waiting_for_event) as *mut core::ffi::c_long,
            FALSE as core::ffi::c_long,
            TRUE as core::ffi::c_long,
        ) != 0
    {
        return;
    }
    if pNtWaitForAlertByThreadId.is_some() {
        (pNtWaitForAlertByThreadId.unwrap_unchecked())(core::ptr::null_mut(), timeout);
    } else {
        NtWaitForSingleObject(
            (*cs).event,
            FALSE as BOOLEAN,
            timeout as *mut LARGE_INTEGER,
        );
    }
}

unsafe extern "C" fn wined3d_cs_command_lock(cs: *const wined3d_cs) {
    if (*cs).serialize_commands != 0 {
        EnterCriticalSection(core::ptr::addr_of_mut!(wined3d_command_cs));
    }
}

unsafe extern "C" fn wined3d_cs_command_unlock(cs: *const wined3d_cs) {
    if (*cs).serialize_commands != 0 {
        LeaveCriticalSection(core::ptr::addr_of_mut!(wined3d_command_cs));
    }
}

unsafe extern "C" fn wined3d_cs_execute_next(cs: *mut wined3d_cs, queue: *mut wined3d_cs_queue) -> bool {
    let mut packet: *mut wined3d_cs_packet = core::ptr::null_mut();
    let mut opcode: wined3d_cs_op = WINED3D_CS_OP_NOP;
    let mut tail: SIZE_T;

    tail = (*queue).tail as SIZE_T;
    packet = wined3d_next_cs_packet(
        core::ptr::addr_of_mut!((*queue).data) as *const u8,
        core::ptr::addr_of_mut!(tail),
        WINED3D_CS_QUEUE_MASK as SIZE_T,
    );

    if (*packet).size != 0 {
        opcode = *(core::ptr::addr_of_mut!((*packet).data) as *mut BYTE as *const wined3d_cs_op);

        TRACE!(
            "Executing %s at %p.\n",
            debug_cs_op(opcode),
            packet
        );

        if opcode as ::core::ffi::c_uint >= WINED3D_CS_OP_STOP as ::core::ffi::c_uint {
            if opcode as ::core::ffi::c_uint > WINED3D_CS_OP_STOP as ::core::ffi::c_uint {
                ERR!("Invalid opcode %#x.\n", opcode as ::core::ffi::c_uint);
            }
            return false;
        }

        wined3d_cs_command_lock(cs);
        (wined3d_cs_op_handlers[opcode as usize].unwrap_unchecked())(
            cs,
            core::ptr::addr_of_mut!((*packet).data) as *mut BYTE as *const ::core::ffi::c_void,
        );
        wined3d_cs_command_unlock(cs);

        TRACE!(
            "%s at %p executed.\n",
            debug_cs_op(opcode),
            packet
        );
    }

    InterlockedExchange(
        core::ptr::addr_of_mut!((*queue).tail) as *mut LONG,
        tail as LONG,
    );
    return true;
}

unsafe extern "C" fn wined3d_cs_exec_execute_command_list(cs: *mut wined3d_cs, data: *const core::ffi::c_void) {
    let op: *const wined3d_cs_execute_command_list = data as *const wined3d_cs_execute_command_list;
    let mut start: SIZE_T = 0 as SIZE_T;
    let end: SIZE_T = (*(*op).list).data_size;
    let cs_data: *const BYTE = (*(*op).list).data as *const BYTE;
    let queue: *mut wined3d_cs_queue;

    TRACE!("Executing command list %p.\n", (*op).list);

    queue = (core::ptr::addr_of_mut!((*cs).queue) as *mut wined3d_cs_queue)
        .offset(WINED3D_CS_QUEUE_MAP as isize) as *mut wined3d_cs_queue;
    while start < end {
        let packet: *const wined3d_cs_packet;
        let mut opcode: wined3d_cs_op = WINED3D_CS_OP_NOP;

        while wined3d_cs_queue_is_empty(cs, queue) == 0 {
            wined3d_cs_execute_next(cs, queue);
        }

        packet = wined3d_next_cs_packet(
            cs_data as *const uint8_t,
            core::ptr::addr_of_mut!(start),
            WINED3D_CS_QUEUE_MASK as SIZE_T,
        );
        opcode = *(core::ptr::addr_of!((*packet).data) as *const BYTE as *const wined3d_cs_op);

        if opcode as u32 >= WINED3D_CS_OP_STOP as u32 {
            ERR!("Invalid opcode %#x.\n", opcode as u32);
        } else {
            wined3d_cs_op_handlers[opcode as usize].unwrap_unchecked()(
                cs,
                core::ptr::addr_of!((*packet).data) as *const BYTE as *const core::ffi::c_void,
            );
        }

        TRACE!("%s executed.\n", debug_cs_op(opcode));
    }
}

unsafe extern "system" fn wined3d_cs_run(ctx: *mut core::ffi::c_void) -> DWORD {
    let mut queue: *mut wined3d_cs_queue = core::ptr::null_mut::<wined3d_cs_queue>();
    let mut spin_count: core::ffi::c_uint = 0 as core::ffi::c_uint;
    let mut cs: *mut wined3d_cs = ctx as *mut wined3d_cs;
    let mut wined3d_module: HMODULE = core::ptr::null_mut::<HINSTANCE__>();
    let mut poll: core::ffi::c_uint = 0 as core::ffi::c_uint;
    let mut run: bool = true;

    WINE_TRACE!("Started.\n");

    SetThreadDescription(GetCurrentThread(), RUSTCONV_WSTR_EC4088632B18.as_ptr());

    /* Copy the module handle to a local variable to avoid racing with the
     * thread freeing "cs" before the FreeLibraryAndExitThread() call. */
    wined3d_module = (*cs).wined3d_module;

    list_init(&raw mut (*cs).query_poll_list);
    (*cs).thread_id = GetCurrentThreadId();
    while run {
        poll = poll.wrapping_add(1);
        if poll == WINED3D_CS_QUERY_POLL_INTERVAL as core::ffi::c_uint {
            wined3d_cs_command_lock(cs);
            poll_queries(cs);
            wined3d_cs_command_unlock(cs);
            poll = 0 as core::ffi::c_uint;
        }

        queue = (&raw mut (*cs).queue as *mut wined3d_cs_queue)
            .offset(WINED3D_CS_QUEUE_MAP as isize)
            as *mut wined3d_cs_queue;
        if wined3d_cs_queue_is_empty(cs, queue) != 0 {
            queue = (&raw mut (*cs).queue as *mut wined3d_cs_queue)
                .offset(WINED3D_CS_QUEUE_DEFAULT as isize)
                as *mut wined3d_cs_queue;
            if wined3d_cs_queue_is_empty(cs, queue) != 0 {
                YieldProcessor();
                spin_count = spin_count.wrapping_add(1);
                if spin_count >= WINED3D_CS_SPIN_COUNT as core::ffi::c_uint {
                    if poll != 0 {
                        poll = WINED3D_CS_QUERY_POLL_INTERVAL as core::ffi::c_uint
                            - 1 as core::ffi::c_uint;
                    } else {
                        wined3d_cs_wait_event(cs);
                    }
                }
                continue;
            }
        }
        spin_count = 0 as core::ffi::c_uint;

        run = wined3d_cs_execute_next(cs, queue);
    }

    (*cs).queue[WINED3D_CS_QUEUE_MAP as usize].tail =
        (*cs).queue[WINED3D_CS_QUEUE_MAP as usize].head;
    (*cs).queue[WINED3D_CS_QUEUE_DEFAULT as usize].tail =
        (*cs).queue[WINED3D_CS_QUEUE_DEFAULT as usize].head;

    WINE_TRACE!("Stopped.\n");

    FreeLibraryAndExitThread(wined3d_module, 0 as DWORD)
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_flush_mapped_buffer(
    context: *mut wined3d_device_context,
    buffer: *mut wined3d_buffer,
) {
    let client: *mut wined3d_client_resource = &raw mut (*buffer).resource.client;

    /* d3d9 applications can draw from a mapped dynamic buffer.
     * Castlevania 2 depends on this behaviour.
     * This means that we need to upload a pending discard bo, without actually
     * unmapping it. */

    assert!(context == &raw mut (*(*(*context).device).cs).c);

    if wined3d_bo_address_is_null(&raw mut (*client).mapped_upload.addr) {
        return;
    }

    if ((*client).mapped_upload.flags & UPLOAD_BO_UPLOAD_ON_UNMAP as u32) != 0 {
        wined3d_device_context_upload_bo(
            context,
            &raw mut (*buffer).resource,
            0 as ::core::ffi::c_uint,
            &raw mut (*client).mapped_box,
            &raw mut (*client).mapped_upload,
            (*buffer).resource.size as ::core::ffi::c_uint,
            (*buffer).resource.size as ::core::ffi::c_uint,
        );
    }

    if ((*client).mapped_upload.flags & UPLOAD_BO_RENAME_ON_UNMAP as u32) != 0 {
        (*client).mapped_upload.flags &= !(UPLOAD_BO_RENAME_ON_UNMAP as u32);
        /* This logic matches wined3d_cs_map_upload_bo(). */
        if ((*(*client).mapped_upload.addr.buffer_object).coherent as ::core::ffi::c_int != 0)
            && (wined3d_map_persistent() as ::core::ffi::c_int != 0)
        {
            (*client).mapped_upload.flags &= !(UPLOAD_BO_UPLOAD_ON_UNMAP as u32);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_cs_create(device: *mut wined3d_device, levels: *const wined3d_feature_level, level_count: ::core::ffi::c_uint) -> *mut wined3d_cs {
    let d3d_info: *const wined3d_d3d_info = ::core::ptr::addr_of!((*(*device).adapter).d3d_info);
    let mut cs: *mut wined3d_cs = ::core::ptr::null_mut();

    cs = calloc(1, (::core::mem::size_of::<wined3d_cs>()) as _) as *mut wined3d_cs;
    if cs.is_null() {
        return ::core::ptr::null_mut();
    }

    if wined3d_state_create(device, levels, level_count, ::core::ptr::addr_of_mut!((*cs).c.state)) < 0 {
        free(cs as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut();
    }

    (*cs).c.ops = ::core::ptr::addr_of!(wined3d_cs_st_ops);
    (*cs).c.device = device;
    (*cs).serialize_commands = (((TRACE_ON!(__wine_dbch_d3d_sync)) != 0)
        || (wined3d_settings.cs_multithreaded & WINED3D_CSMT_SERIALIZE) != 0) as BOOL;

    if (*cs).serialize_commands != 0 {
        WINE_ERR_!(__wine_dbch_d3d_sync, "Forcing serialization of all command streams.\n");
    }

    state_init(
        ::core::ptr::addr_of_mut!((*cs).state),
        d3d_info,
        WINED3D_STATE_NO_REF | WINED3D_STATE_INIT_DEFAULT,
        (*(*cs).c.state).feature_level,
    );

    (*cs).data_size = WINED3D_INITIAL_CS_SIZE as usize;

    'fail: {
        (*cs).data = malloc((*cs).data_size);
        if (*cs).data.is_null() {
            break 'fail;
        }

        if wined3d_settings.cs_multithreaded & WINED3D_CSMT_ENABLE != 0 {
            if (*d3d_info).fences() == 0 {
                WARN!("Disabling CSMT, adapter doesn't support fences.\n");
                wined3d_settings.cs_multithreaded &= !WINED3D_CSMT_ENABLE;
            }
        }

        if wined3d_settings.cs_multithreaded & WINED3D_CSMT_ENABLE != 0
            && RtlIsCriticalSectionLockedByThread(
                (*(*NtCurrentTeb()).ProcessEnvironmentBlock).LoaderLock as *mut RTL_CRITICAL_SECTION,
            ) == 0
        {
            (*cs).c.ops = ::core::ptr::addr_of!(wined3d_cs_mt_ops);

            if pNtAlertThreadByThreadId.is_none() {
                let ntdll: HANDLE = GetModuleHandleW(RUSTCONV_WSTR_80073828D8A0.as_ptr()) as HANDLE;

                pNtAlertThreadByThreadId = ::core::mem::transmute(GetProcAddress(
                    ntdll as HINSTANCE,
                    b"NtAlertThreadByThreadId\0".as_ptr() as LPCSTR,
                ));
                pNtWaitForAlertByThreadId = ::core::mem::transmute(GetProcAddress(
                    ntdll as HINSTANCE,
                    b"NtWaitForAlertByThreadId\0".as_ptr() as LPCSTR,
                ));
            }

            if pNtAlertThreadByThreadId.is_none() && {
                (*cs).event = CreateEventW(
                    ::core::ptr::null_mut(),
                    (FALSE) as BOOL,
                    (FALSE) as BOOL,
                    ::core::ptr::null(),
                );
                (*cs).event.is_null()
            } {
                ERR!("Failed to create command stream event.\n");
                free((*cs).data);
                break 'fail;
            } else {
                (*cs).present_event = CreateEventW(
                    ::core::ptr::null_mut(),
                    (FALSE) as BOOL,
                    (FALSE) as BOOL,
                    ::core::ptr::null(),
                );
                if (*cs).present_event.is_null() {
                    ERR!("Failed to create command stream present event.\n");
                    free((*cs).data);
                    break 'fail;
                }

                if GetModuleHandleExW(
                    GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
                    (wined3d_cs_run as *const ::core::ffi::c_void) as *const u16,
                    ::core::ptr::addr_of_mut!((*cs).wined3d_module),
                ) == 0
                {
                    ERR!("Failed to get wined3d module handle.\n");
                    CloseHandle((*cs).present_event);
                    if !(*cs).event.is_null() {
                        CloseHandle((*cs).event);
                    }
                    free((*cs).data);
                    break 'fail;
                }

                (*cs).thread = CreateThread(
                    ::core::ptr::null_mut(),
                    0,
                    Some(wined3d_cs_run),
                    cs as *mut ::core::ffi::c_void,
                    0,
                    ::core::ptr::null_mut(),
                );
                if (*cs).thread.is_null() {
                    ERR!("Failed to create wined3d command stream thread.\n");
                    FreeLibrary((*cs).wined3d_module);
                    CloseHandle((*cs).present_event);
                    if !(*cs).event.is_null() {
                        CloseHandle((*cs).event);
                    }
                    free((*cs).data);
                    break 'fail;
                }
            }
        }

        TRACE!("Created command stream %p.\n", cs);
        return cs;
    }

    wined3d_state_destroy((*cs).c.state);
    state_cleanup(::core::ptr::addr_of_mut!((*cs).state));
    free(cs as *mut ::core::ffi::c_void);
    return ::core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_cs_destroy(cs: *mut wined3d_cs) {
    if !(*cs).thread.is_null() {
        wined3d_cs_emit_stop(cs);
        CloseHandle((*cs).thread);
        if CloseHandle((*cs).present_event) == 0 {
            ERR!("Closing present event failed.\n");
        }
        if !(*cs).event.is_null() && CloseHandle((*cs).event) == 0 {
            ERR!("Closing event failed.\n");
        }
    }

    wined3d_state_destroy((*cs).c.state);
    state_cleanup(core::ptr::addr_of_mut!((*cs).state));
    free((*cs).data);
    free((cs).cast());
}

unsafe extern "C" fn wined3d_cs_packet_decref_objects(packet: *const wined3d_cs_packet) {
    let mut opcode: wined3d_cs_op =
        *(&raw const (*packet).data as *const BYTE as *const wined3d_cs_op);
    let mut i: ::core::ffi::c_uint = 0;

    match (opcode as ::core::ffi::c_uint) as i32 {
        WINED3D_CS_OP_SET_SAMPLERS => {
            let mut op: *mut wined3d_cs_set_samplers =
                &raw const (*packet).data as *const BYTE as *mut wined3d_cs_set_samplers;
            i = 0 as ::core::ffi::c_uint;
            while i < (*op).count {
                if !(*(*op).samplers.as_ptr().add(i as usize)).is_null() {
                    wined3d_sampler_decref(*(*op).samplers.as_ptr().add(i as usize));
                }
                i = i.wrapping_add(1);
            }
        }

        WINED3D_CS_OP_SET_SHADER => {
            let mut op: *mut wined3d_cs_set_shader =
                &raw const (*packet).data as *const BYTE as *mut wined3d_cs_set_shader;
            if !(*op).shader.is_null() {
                wined3d_shader_decref((*op).shader);
            }
        }

        WINED3D_CS_OP_SET_DEPTH_STENCIL_STATE => {
            let mut op: *mut wined3d_cs_set_depth_stencil_state =
                &raw const (*packet).data as *const BYTE as *mut wined3d_cs_set_depth_stencil_state;
            if !(*op).state.is_null() {
                wined3d_depth_stencil_state_decref((*op).state);
            }
        }

        WINED3D_CS_OP_SET_RASTERIZER_STATE => {
            let mut op: *mut wined3d_cs_set_rasterizer_state =
                &raw const (*packet).data as *const BYTE as *mut wined3d_cs_set_rasterizer_state;
            if !(*op).state.is_null() {
                wined3d_rasterizer_state_decref((*op).state);
            }
        }

        WINED3D_CS_OP_SET_BLEND_STATE => {
            let mut op: *mut wined3d_cs_set_blend_state =
                &raw const (*packet).data as *const BYTE as *mut wined3d_cs_set_blend_state;
            if !(*op).state.is_null() {
                wined3d_blend_state_decref((*op).state);
            }
        }

        WINED3D_CS_OP_SET_RENDERTARGET_VIEWS => {
            let mut op: *mut wined3d_cs_set_rendertarget_views =
                &raw const (*packet).data as *const BYTE as *mut wined3d_cs_set_rendertarget_views;
            i = 0 as ::core::ffi::c_uint;
            while i < (*op).count {
                if !(*core::ptr::addr_of!((*op).views[0]).add(i as usize)).is_null() {
                    wined3d_rendertarget_view_decref((*core::ptr::addr_of!((*op).views[0]).add(i as usize)));
                }
                i = i.wrapping_add(1);
            }
        }

        WINED3D_CS_OP_SET_SHADER_RESOURCE_VIEWS => {
            let mut op: *mut wined3d_cs_set_shader_resource_views =
                &raw const (*packet).data as *const BYTE as *mut wined3d_cs_set_shader_resource_views;
            i = 0 as ::core::ffi::c_uint;
            while i < (*op).count {
                if !(*core::ptr::addr_of!((*op).views[0]).add(i as usize)).is_null() {
                    wined3d_shader_resource_view_decref((*core::ptr::addr_of!((*op).views[0]).add(i as usize)));
                }
                i = i.wrapping_add(1);
            }
        }

        WINED3D_CS_OP_SET_UNORDERED_ACCESS_VIEWS => {
            let mut op: *mut wined3d_cs_set_unordered_access_views =
                &raw const (*packet).data as *const BYTE as *mut wined3d_cs_set_unordered_access_views;
            i = 0 as ::core::ffi::c_uint;
            while i < (*op).count {
                if !(*core::ptr::addr_of!((*op).uavs[0]).add(i as usize)).view.is_null() {
                    wined3d_unordered_access_view_decref((*core::ptr::addr_of!((*op).uavs[0]).add(i as usize)).view);
                }
                i = i.wrapping_add(1);
            }
        }

        WINED3D_CS_OP_SET_DEPTH_STENCIL_VIEW => {
            let mut op: *mut wined3d_cs_set_depth_stencil_view =
                &raw const (*packet).data as *const BYTE as *mut wined3d_cs_set_depth_stencil_view;
            if !(*op).view.is_null() {
                wined3d_rendertarget_view_decref((*op).view);
            }
        }

        WINED3D_CS_OP_SET_CONSTANT_BUFFERS => {
            let mut op: *mut wined3d_cs_set_constant_buffers =
                &raw const (*packet).data as *const BYTE as *mut wined3d_cs_set_constant_buffers;
            i = 0 as ::core::ffi::c_uint;
            while i < (*op).count {
                if !(*core::ptr::addr_of!((*op).buffers[0]).add(i as usize)).buffer.is_null() {
                    wined3d_buffer_decref((*core::ptr::addr_of!((*op).buffers[0]).add(i as usize)).buffer);
                }
                i = i.wrapping_add(1);
            }
        }

        WINED3D_CS_OP_CLEAR_UNORDERED_ACCESS_VIEW => {
            let mut op: *mut wined3d_cs_clear_unordered_access_view =
                &raw const (*packet).data as *const BYTE as *mut wined3d_cs_clear_unordered_access_view;
            wined3d_unordered_access_view_decref((*op).view);
        }

        WINED3D_CS_OP_CLEAR => {
            let mut op: *mut wined3d_cs_clear =
                &raw const (*packet).data as *const BYTE as *mut wined3d_cs_clear;
            i = 0 as ::core::ffi::c_uint;
            while i < (*op).rt_count {
                if !(*op).fb.render_targets[i as usize].is_null() {
                    wined3d_rendertarget_view_decref((*op).fb.render_targets[i as usize]);
                }
                i = i.wrapping_add(1);
            }
            if !(*op).fb.depth_stencil.is_null() {
                wined3d_rendertarget_view_decref((*op).fb.depth_stencil);
            }
        }

        WINED3D_CS_OP_CLEAR_SYSMEM_TEXTURE => {
            let mut op: *mut wined3d_cs_clear_sysmem_texture =
                &raw const (*packet).data as *const BYTE as *mut wined3d_cs_clear_sysmem_texture;
            wined3d_texture_decref((*op).texture);
        }

        WINED3D_CS_OP_DISPATCH => {
            let mut op: *mut wined3d_cs_dispatch =
                &raw const (*packet).data as *const BYTE as *mut wined3d_cs_dispatch;
            if (*op).parameters.indirect != 0 {
                wined3d_buffer_decref((*op).parameters.u.indirect.buffer);
            }
        }

        WINED3D_CS_OP_DRAW => {
            let mut op: *mut wined3d_cs_draw =
                &raw const (*packet).data as *const BYTE as *mut wined3d_cs_draw;
            if (*op).parameters.indirect != 0 {
                wined3d_buffer_decref((*op).parameters.u.indirect.buffer);
            }
        }

        WINED3D_CS_OP_SET_INDEX_BUFFER => {
            let mut op: *mut wined3d_cs_set_index_buffer =
                &raw const (*packet).data as *const BYTE as *mut wined3d_cs_set_index_buffer;
            if !(*op).buffer.is_null() {
                wined3d_buffer_decref((*op).buffer);
            }
        }

        WINED3D_CS_OP_SET_STREAM_OUTPUTS => {
            let mut op: *mut wined3d_cs_set_stream_outputs =
                &raw const (*packet).data as *const BYTE as *mut wined3d_cs_set_stream_outputs;
            i = 0 as ::core::ffi::c_uint;
            while (i as usize) < ARRAY_SIZE!((*op).outputs) as usize {
                if !(*op).outputs[i as usize].buffer.is_null() {
                    wined3d_buffer_decref((*op).outputs[i as usize].buffer);
                }
                i = i.wrapping_add(1);
            }
        }

        WINED3D_CS_OP_SET_STREAM_SOURCES => {
            let mut op: *mut wined3d_cs_set_stream_sources =
                &raw const (*packet).data as *const BYTE as *mut wined3d_cs_set_stream_sources;
            i = 0 as ::core::ffi::c_uint;
            while i < (*op).count {
                if !(*(*op).streams.as_ptr().add(i as usize)).buffer.is_null() {
                    wined3d_buffer_decref((*(*op).streams.as_ptr().add(i as usize)).buffer);
                }
                i = i.wrapping_add(1);
            }
        }

        WINED3D_CS_OP_UPDATE_SUB_RESOURCE => {
            let mut op: *mut wined3d_cs_update_sub_resource =
                &raw const (*packet).data as *const BYTE as *mut wined3d_cs_update_sub_resource;
            wined3d_resource_decref((*op).resource);
        }

        WINED3D_CS_OP_BLT_SUB_RESOURCE => {
            let mut op: *mut wined3d_cs_blt_sub_resource =
                &raw const (*packet).data as *const BYTE as *mut wined3d_cs_blt_sub_resource;
            if !(*op).src_resource.is_null() {
                wined3d_resource_decref((*op).src_resource);
            }
            wined3d_resource_decref((*op).dst_resource);
        }

        WINED3D_CS_OP_COPY_UAV_COUNTER => {
            let mut op: *mut wined3d_cs_copy_uav_counter =
                &raw const (*packet).data as *const BYTE as *mut wined3d_cs_copy_uav_counter;
            wined3d_buffer_decref((*op).buffer);
            wined3d_unordered_access_view_decref((*op).view);
        }

        WINED3D_CS_OP_GENERATE_MIPMAPS => {
            let mut op: *mut wined3d_cs_generate_mipmaps =
                &raw const (*packet).data as *const BYTE as *mut wined3d_cs_generate_mipmaps;
            wined3d_shader_resource_view_decref((*op).view);
        }

        _ => {}
    };
}

unsafe extern "C" fn wined3d_cs_packet_incref_objects(packet: *mut wined3d_cs_packet) {
    let opcode: wined3d_cs_op = *(&raw mut (*packet).data as *mut BYTE as *const wined3d_cs_op);
    let mut i: ::core::ffi::c_uint;
    match (opcode as ::core::ffi::c_uint) as i32 {
        WINED3D_CS_OP_SET_SAMPLERS => {
            let op: *mut wined3d_cs_set_samplers =
                &raw mut (*packet).data as *mut BYTE as *mut wined3d_cs_set_samplers;
            i = 0;
            while i < (*op).count {
                if !(*(&raw mut (*op).samplers as *mut *mut wined3d_sampler).offset(i as isize))
                    .is_null()
                {
                    wined3d_sampler_incref(
                        *(&raw mut (*op).samplers as *mut *mut wined3d_sampler).offset(i as isize),
                    );
                }
                i = i.wrapping_add(1);
            }
        }
        WINED3D_CS_OP_SET_SHADER => {
            let op: *mut wined3d_cs_set_shader =
                &raw mut (*packet).data as *mut BYTE as *mut wined3d_cs_set_shader;
            if !(*op).shader.is_null() {
                wined3d_shader_incref((*op).shader);
            }
        }
        WINED3D_CS_OP_SET_DEPTH_STENCIL_STATE => {
            let op: *mut wined3d_cs_set_depth_stencil_state =
                &raw mut (*packet).data as *mut BYTE as *mut wined3d_cs_set_depth_stencil_state;
            if !(*op).state.is_null() {
                wined3d_depth_stencil_state_incref((*op).state);
            }
        }
        WINED3D_CS_OP_SET_RASTERIZER_STATE => {
            let op: *mut wined3d_cs_set_rasterizer_state =
                &raw mut (*packet).data as *mut BYTE as *mut wined3d_cs_set_rasterizer_state;
            if !(*op).state.is_null() {
                wined3d_rasterizer_state_incref((*op).state);
            }
        }
        WINED3D_CS_OP_SET_BLEND_STATE => {
            let op: *mut wined3d_cs_set_blend_state =
                &raw mut (*packet).data as *mut BYTE as *mut wined3d_cs_set_blend_state;
            if !(*op).state.is_null() {
                wined3d_blend_state_incref((*op).state);
            }
        }
        WINED3D_CS_OP_SET_RENDERTARGET_VIEWS => {
            let op: *mut wined3d_cs_set_rendertarget_views =
                &raw mut (*packet).data as *mut BYTE as *mut wined3d_cs_set_rendertarget_views;
            i = 0;
            while i < (*op).count {
                if !(*(&raw mut (*op).views as *mut *mut wined3d_rendertarget_view)
                    .offset(i as isize))
                .is_null()
                {
                    wined3d_rendertarget_view_incref(
                        *(&raw mut (*op).views as *mut *mut wined3d_rendertarget_view)
                            .offset(i as isize),
                    );
                }
                i = i.wrapping_add(1);
            }
        }
        WINED3D_CS_OP_SET_SHADER_RESOURCE_VIEWS => {
            let op: *mut wined3d_cs_set_shader_resource_views =
                &raw mut (*packet).data as *mut BYTE as *mut wined3d_cs_set_shader_resource_views;
            i = 0;
            while i < (*op).count {
                if !(*(&raw mut (*op).views as *mut *mut wined3d_shader_resource_view)
                    .offset(i as isize))
                .is_null()
                {
                    wined3d_shader_resource_view_incref(
                        *(&raw mut (*op).views as *mut *mut wined3d_shader_resource_view)
                            .offset(i as isize),
                    );
                }
                i = i.wrapping_add(1);
            }
        }
        WINED3D_CS_OP_SET_UNORDERED_ACCESS_VIEWS => {
            let op: *mut wined3d_cs_set_unordered_access_views =
                &raw mut (*packet).data as *mut BYTE as *mut wined3d_cs_set_unordered_access_views;
            i = 0;
            while i < (*op).count {
                if !(*(&raw mut (*op).uavs as *mut wined3d_cs_uav_entry).offset(i as isize))
                    .view
                    .is_null()
                {
                    wined3d_unordered_access_view_incref(
                        (*(&raw mut (*op).uavs as *mut wined3d_cs_uav_entry).offset(i as isize)).view,
                    );
                }
                i = i.wrapping_add(1);
            }
        }
        WINED3D_CS_OP_SET_DEPTH_STENCIL_VIEW => {
            let op: *mut wined3d_cs_set_depth_stencil_view =
                &raw mut (*packet).data as *mut BYTE as *mut wined3d_cs_set_depth_stencil_view;
            if !(*op).view.is_null() {
                wined3d_rendertarget_view_incref((*op).view);
            }
        }
        WINED3D_CS_OP_SET_CONSTANT_BUFFERS => {
            let op: *mut wined3d_cs_set_constant_buffers =
                &raw mut (*packet).data as *mut BYTE as *mut wined3d_cs_set_constant_buffers;
            i = 0;
            while i < (*op).count {
                if !(*(&raw mut (*op).buffers as *mut wined3d_constant_buffer_state)
                    .offset(i as isize))
                .buffer
                .is_null()
                {
                    wined3d_buffer_incref(
                        (*(&raw mut (*op).buffers as *mut wined3d_constant_buffer_state)
                            .offset(i as isize))
                        .buffer,
                    );
                }
                i = i.wrapping_add(1);
            }
        }
        WINED3D_CS_OP_CLEAR_UNORDERED_ACCESS_VIEW => {
            let op: *mut wined3d_cs_clear_unordered_access_view =
                &raw mut (*packet).data as *mut BYTE as *mut wined3d_cs_clear_unordered_access_view;
            wined3d_unordered_access_view_incref((*op).view);
        }
        WINED3D_CS_OP_CLEAR => {
            let op: *mut wined3d_cs_clear =
                &raw mut (*packet).data as *mut BYTE as *mut wined3d_cs_clear;
            i = 0;
            while i < (*op).rt_count {
                if !(*op).fb.render_targets[i as usize].is_null() {
                    wined3d_rendertarget_view_incref((*op).fb.render_targets[i as usize]);
                }
                i = i.wrapping_add(1);
            }
            if !(*op).fb.depth_stencil.is_null() {
                wined3d_rendertarget_view_incref((*op).fb.depth_stencil);
            }
        }
        WINED3D_CS_OP_CLEAR_SYSMEM_TEXTURE => {
            let op: *mut wined3d_cs_clear_sysmem_texture =
                &raw mut (*packet).data as *mut BYTE as *mut wined3d_cs_clear_sysmem_texture;
            wined3d_texture_incref((*op).texture);
        }
        WINED3D_CS_OP_DISPATCH => {
            let op: *mut wined3d_cs_dispatch =
                &raw mut (*packet).data as *mut BYTE as *mut wined3d_cs_dispatch;
            if (*op).parameters.indirect != 0 {
                wined3d_buffer_incref((*op).parameters.u.indirect.buffer);
            }
        }
        WINED3D_CS_OP_DRAW => {
            let op: *mut wined3d_cs_draw =
                &raw mut (*packet).data as *mut BYTE as *mut wined3d_cs_draw;
            if (*op).parameters.indirect != 0 {
                wined3d_buffer_incref((*op).parameters.u.indirect.buffer);
            }
        }
        WINED3D_CS_OP_SET_INDEX_BUFFER => {
            let op: *mut wined3d_cs_set_index_buffer =
                &raw mut (*packet).data as *mut BYTE as *mut wined3d_cs_set_index_buffer;
            if !(*op).buffer.is_null() {
                wined3d_buffer_incref((*op).buffer);
            }
        }
        WINED3D_CS_OP_SET_STREAM_OUTPUTS => {
            let op: *mut wined3d_cs_set_stream_outputs =
                &raw mut (*packet).data as *mut BYTE as *mut wined3d_cs_set_stream_outputs;
            i = 0;
            while (i as usize) < ARRAY_SIZE!((*op).outputs) as usize {
                if !(*op).outputs[i as usize].buffer.is_null() {
                    wined3d_buffer_incref((*op).outputs[i as usize].buffer);
                }
                i = i.wrapping_add(1);
            }
        }
        WINED3D_CS_OP_SET_STREAM_SOURCES => {
            let op: *mut wined3d_cs_set_stream_sources =
                &raw mut (*packet).data as *mut BYTE as *mut wined3d_cs_set_stream_sources;
            i = 0;
            while i < (*op).count {
                if !(*(&raw mut (*op).streams as *mut wined3d_stream_state).offset(i as isize))
                    .buffer
                    .is_null()
                {
                    wined3d_buffer_incref(
                        (*(&raw mut (*op).streams as *mut wined3d_stream_state)
                            .offset(i as isize))
                        .buffer as *mut wined3d_buffer,
                    );
                }
                i = i.wrapping_add(1);
            }
        }
        WINED3D_CS_OP_UPDATE_SUB_RESOURCE => {
            let op: *mut wined3d_cs_update_sub_resource =
                &raw mut (*packet).data as *mut BYTE as *mut wined3d_cs_update_sub_resource;
            wined3d_resource_incref((*op).resource);
        }
        WINED3D_CS_OP_BLT_SUB_RESOURCE => {
            let op: *mut wined3d_cs_blt_sub_resource =
                &raw mut (*packet).data as *mut BYTE as *mut wined3d_cs_blt_sub_resource;
            if !(*op).src_resource.is_null() {
                wined3d_resource_incref((*op).src_resource);
            }
            wined3d_resource_incref((*op).dst_resource);
        }
        WINED3D_CS_OP_COPY_UAV_COUNTER => {
            let op: *mut wined3d_cs_copy_uav_counter =
                &raw mut (*packet).data as *mut BYTE as *mut wined3d_cs_copy_uav_counter;
            wined3d_buffer_incref((*op).buffer);
            wined3d_unordered_access_view_incref((*op).view);
        }
        WINED3D_CS_OP_GENERATE_MIPMAPS => {
            let op: *mut wined3d_cs_generate_mipmaps =
                &raw mut (*packet).data as *mut BYTE as *mut wined3d_cs_generate_mipmaps;
            wined3d_shader_resource_view_incref((*op).view);
        }
        _ => {}
    };
}

unsafe extern "C" fn wined3d_deferred_context_from_context(
    context: *mut wined3d_device_context,
) -> *mut wined3d_deferred_context {
    return CONTAINING_RECORD!(context, wined3d_deferred_context, c);
}

unsafe extern "C" fn wined3d_deferred_context_require_space(
    mut context: *mut wined3d_device_context,
    mut size: size_t,
    mut queue_id: wined3d_cs_queue_id,
) -> *mut ::core::ffi::c_void {
    let mut deferred: *mut wined3d_deferred_context =
        wined3d_deferred_context_from_context(context);
    let mut packet: *mut wined3d_cs_packet = core::ptr::null_mut::<wined3d_cs_packet>();
    let mut header_size: size_t = 0;
    let mut packet_size: size_t = 0;
    if queue_id as ::core::ffi::c_uint != WINED3D_CS_QUEUE_DEFAULT as ::core::ffi::c_uint {
        return core::ptr::null_mut();
    }
    header_size = (core::mem::offset_of!(wined3d_cs_packet, data) as size_t)
        .wrapping_add(0 as size_t);
    packet_size = (core::mem::offset_of!(wined3d_cs_packet, data) as size_t).wrapping_add(size);
    packet_size = packet_size
        .wrapping_add(header_size)
        .wrapping_sub(1 as size_t)
        & !header_size.wrapping_sub(1 as size_t);
    if wined3d_array_reserve(
        core::ptr::addr_of_mut!((*deferred).data),
        core::ptr::addr_of_mut!((*deferred).data_capacity) as *mut SIZE_T,
        ((*deferred).data_size.wrapping_add((packet_size as SIZE_T) as _)) as SIZE_T,
        1 as SIZE_T,
    ) == 0
    {
        return core::ptr::null_mut();
    }
    packet = ((*deferred).data as *mut BYTE).add((*deferred).data_size as usize)
        as *mut wined3d_cs_packet;
    WINE_TRACE!("size was %Iu, adding %Iu\n", (*deferred).data_size, packet_size);
    (*packet).size = packet_size.wrapping_sub(header_size);
    return core::ptr::addr_of_mut!((*packet).data) as *mut ::core::ffi::c_void;
}

unsafe extern "C" fn wined3d_deferred_context_submit(
    mut context: *mut wined3d_device_context,
    mut queue_id: wined3d_cs_queue_id,
) {
    let mut deferred: *mut wined3d_deferred_context = wined3d_deferred_context_from_context(context);
    let mut packet: *mut wined3d_cs_packet = core::ptr::null_mut();

    assert!(queue_id == WINED3D_CS_QUEUE_DEFAULT);
    packet = wined3d_next_cs_packet(
        (*deferred).data as *const uint8_t,
        core::ptr::addr_of_mut!((*deferred).data_size) as *mut SIZE_T,
        !(0 as SIZE_T),
    );
    wined3d_cs_packet_incref_objects(packet);
}

unsafe extern "C" fn wined3d_deferred_context_finish(
    context: *mut wined3d_device_context,
    queue_id: wined3d_cs_queue_id,
) {
    ERR!("Ignoring finish() on a deferred context.\n");
}

unsafe extern "C" fn deferred_context_get_upload(
    deferred: *mut wined3d_deferred_context,
    resource: *mut wined3d_resource,
    sub_resource_idx: ::core::ffi::c_uint,
) -> *mut wined3d_deferred_upload {
    let mut i: SIZE_T = ((*deferred).upload_count) as _;

    while i != 0 {
        i = i.wrapping_sub(1);
        let upload: *mut wined3d_deferred_upload = (*deferred).uploads.add(i as usize);

        if (*upload).resource == resource && (*upload).sub_resource_idx == sub_resource_idx {
            return upload;
        }
    }

    core::ptr::null_mut()
}

unsafe extern "C" fn wined3d_deferred_context_map_upload_bo(
    context: *mut wined3d_device_context,
    resource: *mut wined3d_resource,
    sub_resource_idx: u32,
    map_desc: *mut wined3d_map_desc,
    box_0: *const wined3d_box,
    flags: uint32_t,
) -> bool {
    let deferred: *mut wined3d_deferred_context = wined3d_deferred_context_from_context(context);
    let format: *const wined3d_format = (*resource).format;
    let device: *mut wined3d_device = (*context).device;
    let mut upload: *mut wined3d_deferred_upload = core::ptr::null_mut();
    let mut addr: wined3d_bo_address = wined3d_bo_address {
        buffer_object: core::ptr::null_mut::<wined3d_bo>(),
        addr: core::ptr::null_mut::<BYTE>(),
    };
    let mut sysmem: *mut uint8_t = core::ptr::null_mut();
    let mut size: size_t = 0;

    get_map_pitch(format, box_0, map_desc, core::ptr::addr_of_mut!(size));

    if (flags & (WINED3D_MAP_WRITE as uint32_t)) == 0 {
        WARN!("Flags %#x are not valid on a deferred context.\n", flags);
        return false;
    }

    if (flags
        & !((WINED3D_MAP_WRITE as uint32_t)
            | (WINED3D_MAP_DISCARD as uint32_t)
            | (WINED3D_MAP_NOOVERWRITE as uint32_t)))
        != 0
    {
        return false;
    }

    if (flags & (WINED3D_MAP_NOOVERWRITE as uint32_t)) != 0 {
        upload = deferred_context_get_upload(deferred, resource, sub_resource_idx);
        if upload.is_null() {
            return false;
        }

        (*upload).upload_flags = 0 as uint32_t;
        (*map_desc).data = (*upload).map_ptr as *mut core::ffi::c_void;
        return true;
    }

    if wined3d_array_reserve(
        core::ptr::addr_of_mut!((*deferred).uploads) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*deferred).uploads_capacity) as *mut SIZE_T,
        ((*deferred).upload_count.wrapping_add((1 as SIZE_T) as _)) as SIZE_T,
        core::mem::size_of::<wined3d_deferred_upload>() as SIZE_T,
    ) == 0
    {
        return false;
    }

    let fresh22 = (*deferred).upload_count;
    (*deferred).upload_count = (*deferred).upload_count.wrapping_add(1);
    upload = (*deferred).uploads.add(fresh22 as usize) as *mut wined3d_deferred_upload;

    if (flags & (WINED3D_MAP_DISCARD as uint32_t)) != 0
        && (*(*(*device).adapter).adapter_ops)
            .adapter_alloc_bo
            .unwrap_unchecked()(
            device,
            resource,
            sub_resource_idx,
            core::ptr::addr_of_mut!(addr),
        ) != false
    {
        (*upload).bo = addr.buffer_object;
        (*upload).sysmem = core::ptr::null_mut::<uint8_t>();

        TRACE!(
            "Allocated BO %s.\n",
            debug_bo_address(core::ptr::addr_of_mut!(addr))
        );

        wined3d_device_bo_map_lock(device);
        (*upload).map_ptr = (*addr.buffer_object).map_ptr as *mut uint8_t;
        wined3d_device_bo_map_unlock(device);
        (*upload).map_ptr = (*upload)
            .map_ptr
            .offset((*addr.buffer_object).memory_offset as isize);
        assert!(!(*upload).map_ptr.is_null());
    } else {
        if (*deferred).upload_heap.is_null() {
            (*deferred).upload_heap = HeapCreate(
                0 as DWORD,
                0 as SIZE_T,
                0 as SIZE_T,
            );
            if (*deferred).upload_heap.is_null() {
                ERR!("Failed to create upload heap.\n");
                return false;
            }

            (*deferred).upload_heap_refcount =
                malloc(core::mem::size_of::<LONG>() as size_t) as *mut LONG;
            if (*deferred).upload_heap_refcount.is_null() {
                HeapDestroy((*deferred).upload_heap);
                (*deferred).upload_heap = core::ptr::null_mut();
                return false;
            }

            *(*deferred).upload_heap_refcount = 1 as LONG;
        }

        sysmem = HeapAlloc(
            (*deferred).upload_heap,
            0 as DWORD,
            (size as SIZE_T)
                .wrapping_add(RESOURCE_ALIGNMENT as SIZE_T)
                .wrapping_sub(1 as SIZE_T),
        ) as *mut uint8_t;
        if sysmem.is_null() {
            return false;
        }

        (*upload).bo = core::ptr::null_mut::<wined3d_bo>();
        (*upload).sysmem = sysmem;
        (*upload).map_ptr = align(
            (*upload).sysmem as size_t,
            RESOURCE_ALIGNMENT as size_t,
        ) as *mut uint8_t;
    }

    (*upload).upload_flags = UPLOAD_BO_UPLOAD_ON_UNMAP as uint32_t;
    (*upload).resource = resource;
    wined3d_resource_incref(resource);
    (*upload).sub_resource_idx = sub_resource_idx;
    (*upload).box_ = *box_0;

    (*map_desc).data = (*upload).map_ptr as *mut core::ffi::c_void;
    return true;
}

unsafe extern "C" fn wined3d_deferred_context_unmap_upload_bo(
    context: *mut wined3d_device_context,
    resource: *mut wined3d_resource,
    sub_resource_idx: core::ffi::c_uint,
    box_0: *mut wined3d_box,
    bo: *mut upload_bo,
) -> bool {
    let deferred: *mut wined3d_deferred_context = wined3d_deferred_context_from_context(context);
    let mut upload: *mut wined3d_deferred_upload = core::ptr::null_mut::<wined3d_deferred_upload>();

    upload = deferred_context_get_upload(deferred, resource, sub_resource_idx);
    if !upload.is_null() {
        *box_0 = (*upload).box_;
        (*bo).addr.buffer_object = (*upload).bo;
        if !(*bo).addr.buffer_object.is_null() {
            (*bo).addr.addr = core::ptr::null_mut::<u8>();
        } else {
            (*bo).addr.addr = (*upload).map_ptr;
        }
        (*bo).flags = (*upload).upload_flags;
        (*upload).upload_flags = 0;
        return true;
    }

    false
}

unsafe extern "C" fn wined3d_deferred_context_issue_query(
    mut context: *mut wined3d_device_context,
    mut query: *mut wined3d_query,
    mut flags: ::core::ffi::c_uint,
) {
    let mut deferred: *mut wined3d_deferred_context = wined3d_deferred_context_from_context(context);
    let mut op: *mut wined3d_cs_query_issue = ::core::ptr::null_mut::<wined3d_cs_query_issue>();

    op = wined3d_device_context_require_space(
        context,
        (::core::mem::size_of::<wined3d_cs_query_issue>() as SIZE_T) as _,
        WINED3D_CS_QUEUE_DEFAULT,
    ) as *mut wined3d_cs_query_issue;
    (*op).opcode = WINED3D_CS_OP_QUERY_ISSUE;
    (*op).query = query;
    (*op).flags = flags as uint32_t;

    wined3d_device_context_submit(context, WINED3D_CS_QUEUE_DEFAULT);

    if wined3d_array_reserve(
        core::ptr::addr_of_mut!((*deferred).queries) as *mut *mut ::core::ffi::c_void,
        core::ptr::addr_of_mut!((*deferred).queries_capacity) as *mut SIZE_T,
        ((*deferred).query_count.wrapping_add(1)) as SIZE_T,
        ::core::mem::size_of::<wined3d_deferred_query_issue>() as SIZE_T,
    ) == 0
    {
        ERR!("Failed to reserve memory.\n");
        return;
    }

    (*(*deferred).queries.offset((*deferred).query_count as isize)).flags = flags;
    wined3d_query_incref({
        let fresh19 = (*deferred).query_count;
        (*deferred).query_count = (*deferred).query_count.wrapping_add(1);
        (*(*deferred).queries.offset(fresh19 as isize)).query = query;
        query
    });
}

unsafe extern "C" fn wined3d_deferred_context_flush(context: *mut wined3d_device_context) {
    // FIXME("context %p, stub!\n", context);
}

unsafe extern "C" fn wined3d_deferred_context_reference_resource(context: *mut wined3d_device_context, resource: *mut wined3d_resource) {
    let deferred: *mut wined3d_deferred_context = wined3d_deferred_context_from_context(context);

    if wined3d_array_reserve(
        core::ptr::addr_of_mut!((*deferred).resources) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*deferred).resources_capacity) as *mut SIZE_T,
        ((*deferred).resource_count + 1) as SIZE_T,
        core::mem::size_of::<*mut wined3d_resource>() as SIZE_T,
    ) == 0
    {
        return;
    }

    let fresh17 = (*deferred).resource_count;
    (*deferred).resource_count = fresh17 + 1;
    *(*deferred).resources.add(fresh17 as usize) = resource;
    wined3d_resource_incref(resource);
}

unsafe extern "C" fn wined3d_deferred_context_reference_command_list(context: *mut wined3d_device_context, list: *mut wined3d_command_list) {
    let deferred: *mut wined3d_deferred_context = wined3d_deferred_context_from_context(context);

    /* Grab a reference to the command list. Note that this implicitly prevents
     * any dependent command lists or resources from being freed as well. */
    if wined3d_array_reserve(
        core::ptr::addr_of_mut!((*deferred).command_lists) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*deferred).command_lists_capacity) as *mut SIZE_T,
        ((*deferred).command_list_count + 1) as SIZE_T,
        core::mem::size_of::<*mut wined3d_command_list>() as SIZE_T,
    ) == 0
    {
        return;
    }

    let idx = (*deferred).command_list_count;
    (*deferred).command_list_count = (*deferred).command_list_count.wrapping_add(1);
    let slot = (*deferred).command_lists.offset(idx as isize);
    *slot = list;
    wined3d_command_list_incref(*slot);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_deferred_context_create(
    mut device: *mut wined3d_device,
    mut context: *mut *mut wined3d_device_context,
) -> HRESULT {
    let mut object: *mut wined3d_deferred_context =
        ::core::ptr::null_mut::<wined3d_deferred_context>();
    let mut hr: HRESULT = 0;
    WINE_TRACE!("device %p, context %p.\n", device, context);
    object = calloc(
        1,
        (core::mem::size_of::<wined3d_deferred_context>() as size_t) as _,
    ) as *mut wined3d_deferred_context;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }
    hr = wined3d_state_create(
        device,
        &raw mut (*(*(*device).cs).c.state).feature_level,
        1,
        &raw mut (*object).c.state,
    );
    if hr < 0 {
        free(object as *mut ::core::ffi::c_void);
        return hr;
    }
    (*object).c.ops = &raw const wined3d_deferred_context_ops;
    (*object).c.device = device;
    wined3d_device_context_emit_reset_state(&raw mut (*object).c, true);
    WINE_TRACE!("Created deferred context %p.\n", object);
    *context = &raw mut (*object).c;
    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_deferred_context_destroy(context: *mut wined3d_device_context) {
    let deferred: *mut wined3d_deferred_context = wined3d_deferred_context_from_context(context);
    let mut packet: *const wined3d_cs_packet;
    let mut i: SIZE_T;
    let mut offset: SIZE_T = 0 as SIZE_T;

    i = 0 as SIZE_T;
    while i < ((*deferred).resource_count) as _ {
        wined3d_resource_decref(*(*deferred).resources.add(i as usize));
        i += 1;
    }
    free((*deferred).resources as *mut core::ffi::c_void);

    i = 0 as SIZE_T;
    while i < ((*deferred).upload_count) as _ {
        wined3d_resource_decref((*(*deferred).uploads.add(i as usize)).resource);
        HeapFree(
            (*deferred).upload_heap,
            0 as DWORD,
            (*(*deferred).uploads.add(i as usize)).resource as PVOID,
        );
        i += 1;
    }

    if !(*deferred).upload_heap.is_null() {
        if InterlockedDecrement((*deferred).upload_heap_refcount) == 0 {
            HeapDestroy((*deferred).upload_heap);
            free((*deferred).upload_heap_refcount as *mut core::ffi::c_void);
        }
    }

    free((*deferred).uploads as *mut core::ffi::c_void);

    i = 0 as SIZE_T;
    while i < ((*deferred).command_list_count) as _ {
        wined3d_command_list_decref(*(*deferred).command_lists.add(i as usize));
        i += 1;
    }
    free((*deferred).command_lists as *mut core::ffi::c_void);

    i = 0 as SIZE_T;
    while i < ((*deferred).query_count) as _ {
        wined3d_query_decref((*(*deferred).queries.add(i as usize)).query);
        i += 1;
    }
    free((*deferred).queries as *mut core::ffi::c_void);

    while offset < ((*deferred).data_size) as _ {
        packet = wined3d_next_cs_packet(
            (*deferred).data as *const uint8_t,
            core::ptr::addr_of_mut!(offset),
            !(0 as SIZE_T),
        );
        wined3d_cs_packet_decref_objects(packet);
    }

    wined3d_state_destroy((*deferred).c.state);
    free((*deferred).data as *mut core::ffi::c_void);
    free(deferred as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_deferred_context_record_command_list(
    context: *mut wined3d_device_context,
    restore: bool,
    list: *mut *mut wined3d_command_list,
) -> HRESULT {
    let deferred: *mut wined3d_deferred_context = wined3d_deferred_context_from_context(context);
    let mut object: *mut wined3d_command_list = core::ptr::null_mut();
    let mut memory: *mut core::ffi::c_void = core::ptr::null_mut();

    wined3d_device_context_lock(context);
    memory = malloc(
        (core::mem::size_of::<wined3d_command_list>() as usize)
            .wrapping_add(
                ((*deferred).resource_count as usize)
                    .wrapping_mul(core::mem::size_of::<*mut wined3d_resource>() as usize),
            )
            .wrapping_add(
                ((*deferred).upload_count as usize)
                    .wrapping_mul(core::mem::size_of::<wined3d_deferred_upload>() as usize),
            )
            .wrapping_add(
                ((*deferred).command_list_count as usize)
                    .wrapping_mul(core::mem::size_of::<*mut wined3d_command_list>() as usize),
            )
            .wrapping_add(
                ((*deferred).query_count as usize)
                    .wrapping_mul(core::mem::size_of::<wined3d_deferred_query_issue>() as usize),
            )
            .wrapping_add((*deferred).data_size as usize),
    );

    if memory.is_null() {
        wined3d_device_context_unlock(context);
        return E_OUTOFMEMORY;
    }

    object = memory as *mut wined3d_command_list;
    memory = object.offset(1 as isize) as *mut wined3d_command_list
        as *mut core::ffi::c_void;
    memset(
        object as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<wined3d_command_list>() as usize,
    );
    (*object).refcount = 1 as LONG;
    (*object).device = (*deferred).c.device;

    (*object).resources = memory as *mut *mut wined3d_resource;
    memory = (*object)
        .resources
        .offset((*deferred).resource_count as isize) as *mut *mut wined3d_resource
        as *mut core::ffi::c_void;
    (*object).resource_count = ((*deferred).resource_count) as _;
    memcpy(
        (*object).resources as *mut core::ffi::c_void,
        (*deferred).resources as *const core::ffi::c_void,
        ((*deferred).resource_count as usize)
            .wrapping_mul(core::mem::size_of::<*mut wined3d_resource>() as usize),
    );
    /* Transfer our references to the resources to the command list. */

    (*object).uploads = memory as *mut wined3d_deferred_upload;
    memory = (*object).uploads.offset((*deferred).upload_count as isize)
        as *mut wined3d_deferred_upload as *mut core::ffi::c_void;
    (*object).upload_count = ((*deferred).upload_count) as _;
    memcpy(
        (*object).uploads as *mut core::ffi::c_void,
        (*deferred).uploads as *const core::ffi::c_void,
        ((*deferred).upload_count as usize)
            .wrapping_mul(core::mem::size_of::<wined3d_deferred_upload>() as usize),
    );
    /* Transfer our references to the resources to the command list. */

    (*object).command_lists = memory as *mut *mut wined3d_command_list;
    memory = (*object)
        .command_lists
        .offset((*deferred).command_list_count as isize)
        as *mut *mut wined3d_command_list as *mut core::ffi::c_void;
    (*object).command_list_count = ((*deferred).command_list_count) as _;
    memcpy(
        (*object).command_lists as *mut core::ffi::c_void,
        (*deferred).command_lists as *const core::ffi::c_void,
        ((*deferred).command_list_count as usize)
            .wrapping_mul(core::mem::size_of::<*mut wined3d_command_list>() as usize),
    );
    /* Transfer our references to the command lists to the command list. */

    (*object).queries = memory as *mut wined3d_deferred_query_issue;
    memory = (*object).queries.offset((*deferred).query_count as isize)
        as *mut wined3d_deferred_query_issue as *mut core::ffi::c_void;
    (*object).query_count = ((*deferred).query_count) as _;
    memcpy(
        (*object).queries as *mut core::ffi::c_void,
        (*deferred).queries as *const core::ffi::c_void,
        ((*deferred).query_count as usize)
            .wrapping_mul(core::mem::size_of::<wined3d_deferred_query_issue>() as usize),
    );
    /* Transfer our references to the queries to the command list. */

    (*object).data = memory;
    (*object).data_size = ((*deferred).data_size) as _;
    memcpy(
        (*object).data,
        (*deferred).data,
        (*deferred).data_size as usize,
    );

    (*deferred).data_size = (0 as SIZE_T) as _;
    (*deferred).resource_count = (0 as SIZE_T) as _;
    (*deferred).upload_count = (0 as SIZE_T) as _;
    (*deferred).command_list_count = (0 as SIZE_T) as _;
    (*deferred).query_count = (0 as SIZE_T) as _;

    (*object).upload_heap = (*deferred).upload_heap;
    (*object).upload_heap_refcount = (*deferred).upload_heap_refcount;
    if !(*object).upload_heap_refcount.is_null() {
        InterlockedIncrement((*object).upload_heap_refcount);
    }

    /* This is in fact recorded into a subsequent command list. */
    if restore {
        wined3d_device_context_set_state(&raw mut (*deferred).c, (*deferred).c.state);
    } else {
        wined3d_device_context_reset_state(&raw mut (*deferred).c);
    }

    *list = object;
    wined3d_device_context_unlock(context);

    return S_OK;
}

unsafe extern "C" fn wined3d_command_list_destroy_object(object: *mut c_void) {
    let list: *mut wined3d_command_list = object as *mut wined3d_command_list;
    let mut context: *mut wined3d_context;
    let mut i: c_uint;

    context = context_acquire(
        (*list).device,
        core::ptr::null_mut::<wined3d_texture>(),
        0,
    );

    i = 0;
    while (i as usize) < ((*list).upload_count) as _ {
        let mut bo: *mut wined3d_bo = core::ptr::null_mut::<wined3d_bo>();
        bo = (*(*list).uploads.add(i as usize)).bo;
        if !bo.is_null() {
            (*bo).refcount = (*bo).refcount.wrapping_sub(1);
            if (*bo).refcount == 0 {
                wined3d_context_destroy_bo(context, bo);
                free(bo as *mut c_void);
            }
        } else {
            HeapFree(
                (*list).upload_heap,
                0,
                (*(*list).uploads.add(i as usize)).sysmem as PVOID,
            );
        }
        i = i.wrapping_add(1);
    }

    context_release(context);

    if !(*list).upload_heap.is_null() {
        if InterlockedDecrement((*list).upload_heap_refcount) == 0 {
            HeapDestroy((*list).upload_heap);
            free((*list).upload_heap_refcount as *mut c_void);
        }
    }

    free(list as *mut c_void);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_command_list_incref(mut list: *mut wined3d_command_list) -> ULONG {
    let mut refcount: ::core::ffi::c_uint = InterlockedIncrement(
        core::ptr::addr_of_mut!((*list).refcount) as *mut ::core::ffi::c_long,
    ) as ::core::ffi::c_uint;
    WINE_TRACE!("%p increasing refcount to %u.\n", list, refcount);
    return refcount as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_command_list_decref(mut list: *mut wined3d_command_list) -> ULONG {
    let mut refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*list).refcount)) as ULONG;
    let mut device: *mut wined3d_device = (*list).device;
    let mut packet: *const wined3d_cs_packet = core::ptr::null();
    let mut i: SIZE_T;
    let mut offset: SIZE_T;

    if refcount == 0 {
        i = 0;
        while i < (*list).command_list_count {
            wined3d_command_list_decref(*(*list).command_lists.offset(i as isize));
            i = i.wrapping_add(1);
        }
        i = 0;
        while i < (*list).resource_count {
            wined3d_resource_decref(*(*list).resources.offset(i as isize));
            i = i.wrapping_add(1);
        }
        i = 0;
        while i < (*list).upload_count {
            wined3d_resource_decref((*(*list).uploads.offset(i as isize)).resource);
            i = i.wrapping_add(1);
        }
        i = 0;
        while i < (*list).query_count {
            wined3d_query_decref((*(*list).queries.offset(i as isize)).query);
            i = i.wrapping_add(1);
        }

        offset = 0;
        while offset < (*list).data_size {
            packet = wined3d_next_cs_packet(
                (*list).data as *const uint8_t,
                core::ptr::addr_of_mut!(offset),
                !(0 as SIZE_T),
            );
            wined3d_cs_packet_decref_objects(packet);
        }

        wined3d_mutex_lock();
        wined3d_cs_destroy_object(
            (*device).cs as *mut wined3d_cs,
            Some(
                wined3d_command_list_destroy_object
                    as unsafe extern "C" fn(*mut core::ffi::c_void),
            ),
            list as *mut core::ffi::c_void,
        );
        wined3d_mutex_unlock();
    }

    return refcount;
}
