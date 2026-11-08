use crate::d3dwine::context::context_invalidate_compute_state;
use crate::d3dwine::buffer::wined3d_buffer_decref;
use crate::d3dwine::buffer::wined3d_buffer_incref;
use crate::d3dwine::context_gl::wined3d_context_gl_active_texture;
use crate::d3dwine::context_gl::wined3d_context_gl_allocate_vram_chunk_buffer;
use crate::d3dwine::context_gl::wined3d_context_gl_bind_bo;
use crate::d3dwine::context_gl::wined3d_context_gl_bind_dummy_textures;
use crate::d3dwine::context_gl::wined3d_context_gl_destroy;
use crate::d3dwine::context_gl::wined3d_context_gl_submit_command_fence;
use crate::d3dwine::context_gl::wined3d_context_gl_wait_command_fence;
use crate::d3dwine::context_vk::wined3d_context_vk_create_bo;
use crate::d3dwine::context_vk::wined3d_context_vk_create_image;
use crate::d3dwine::context_vk::wined3d_context_vk_destroy_bo;
use crate::d3dwine::context_vk::wined3d_context_vk_destroy_image;
use crate::d3dwine::context_vk::wined3d_context_vk_destroy_vk_buffer_view;
use crate::d3dwine::context_vk::wined3d_context_vk_destroy_vk_image_view;
use crate::d3dwine::context_vk::wined3d_context_vk_get_command_buffer;
use crate::d3dwine::context_vk::wined3d_context_vk_image_barrier;
use crate::d3dwine::cs::wined3d_cs_create;
use crate::d3dwine::cs::wined3d_cs_destroy;
use crate::d3dwine::cs::wined3d_cs_destroy_object;
use crate::d3dwine::cs::wined3d_cs_emit_clear;
use crate::d3dwine::cs::wined3d_cs_emit_unload_resource;
use crate::d3dwine::cs::wined3d_device_context_emit_blt_sub_resource;
use crate::d3dwine::cs::wined3d_device_context_emit_clear_rendertarget_view;
use crate::d3dwine::cs::wined3d_device_context_emit_clear_uav;
use crate::d3dwine::cs::wined3d_device_context_emit_copy_uav_counter;
use crate::d3dwine::cs::wined3d_device_context_emit_draw;
use crate::d3dwine::cs::wined3d_device_context_emit_execute_command_list;
use crate::d3dwine::cs::wined3d_device_context_emit_generate_mipmaps;
use crate::d3dwine::cs::wined3d_device_context_emit_map;
use crate::d3dwine::cs::wined3d_device_context_emit_reset_state;
use crate::d3dwine::cs::wined3d_device_context_emit_set_blend_state;
use crate::d3dwine::cs::wined3d_device_context_emit_set_constant_buffers;
use crate::d3dwine::cs::wined3d_device_context_emit_set_depth_stencil_state;
use crate::d3dwine::cs::wined3d_device_context_emit_set_depth_stencil_view;
use crate::d3dwine::cs::wined3d_device_context_emit_set_feature_level;
use crate::d3dwine::cs::wined3d_device_context_emit_set_index_buffer;
use crate::d3dwine::cs::wined3d_device_context_emit_set_predication;
use crate::d3dwine::cs::wined3d_device_context_emit_set_rasterizer_state;
use crate::d3dwine::cs::wined3d_device_context_emit_set_rendertarget_views;
use crate::d3dwine::cs::wined3d_device_context_emit_set_samplers;
use crate::d3dwine::cs::wined3d_device_context_emit_set_scissor_rects;
use crate::d3dwine::cs::wined3d_device_context_emit_set_shader;
use crate::d3dwine::cs::wined3d_device_context_emit_set_shader_resource_views;
use crate::d3dwine::cs::wined3d_device_context_emit_set_stream_outputs;
use crate::d3dwine::cs::wined3d_device_context_emit_set_stream_sources;
use crate::d3dwine::cs::wined3d_device_context_emit_set_unordered_access_views;
use crate::d3dwine::cs::wined3d_device_context_emit_set_vertex_declaration;
use crate::d3dwine::cs::wined3d_device_context_emit_set_viewports;
use crate::d3dwine::cs::wined3d_device_context_emit_unmap;
use crate::d3dwine::cs::wined3d_device_context_emit_update_sub_resource;
use crate::d3dwine::context::wined3d_stream_info_from_declaration;
use crate::prelude::*;
use core::ffi::c_void;


// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_52 {
    pub flags: GLbitfield,
}

#[no_mangle]
pub static mut WINED3D_default_light: wined3d_light = wined3d_light {
    type_: WINED3D_LIGHT_DIRECTIONAL,
    diffuse: wined3d_color {
        r: 1.0f32,
        g: 1.0f32,
        b: 1.0f32,
        a: 0.0f32,
    },
    specular: wined3d_color {
        r: 0.0f32,
        g: 0.0f32,
        b: 0.0f32,
        a: 0.0f32,
    },
    ambient: wined3d_color {
        r: 0.0f32,
        g: 0.0f32,
        b: 0.0f32,
        a: 0.0f32,
    },
    position: wined3d_vec3 {
        x: 0.0f32,
        y: 0.0f32,
        z: 0.0f32,
    },
    direction: wined3d_vec3 {
        x: 0.0f32,
        y: 0.0f32,
        z: 1.0f32,
    },
    range: 0.0f32,
    falloff: 0.0f32,
    attenuation0: 0.0f32,
    attenuation1: 0.0f32,
    attenuation2: 0.0f32,
    theta: 0.0f32,
    phi: 0.0f32,
};

static mut wined3d_allocator_gl_ops: wined3d_allocator_ops = unsafe {
    wined3d_allocator_ops {
        allocator_create_chunk: Some(
            wined3d_allocator_gl_create_chunk
                as unsafe extern "C" fn(
                    *mut wined3d_allocator,
                    *mut wined3d_context,
                    ::core::ffi::c_uint,
                    size_t,
                ) -> *mut wined3d_allocator_chunk,
        ),
        allocator_destroy_chunk: Some(
            wined3d_allocator_gl_destroy_chunk
                as unsafe extern "C" fn(*mut wined3d_allocator_chunk) -> (),
        ),
    }
};

static mut gl_memory_types: [C2RustUnnamed_52; 8] = [
    C2RustUnnamed_52 {
        flags: 0 as GLbitfield,
    },
    C2RustUnnamed_52 {
        flags: GL_MAP_READ_BIT as GLbitfield,
    },
    C2RustUnnamed_52 {
        flags: GL_MAP_WRITE_BIT as GLbitfield,
    },
    C2RustUnnamed_52 {
        flags: (GL_MAP_READ_BIT | GL_MAP_WRITE_BIT) as GLbitfield,
    },
    C2RustUnnamed_52 {
        flags: GL_CLIENT_STORAGE_BIT as GLbitfield,
    },
    C2RustUnnamed_52 {
        flags: (GL_CLIENT_STORAGE_BIT | GL_MAP_READ_BIT) as GLbitfield,
    },
    C2RustUnnamed_52 {
        flags: (GL_CLIENT_STORAGE_BIT | GL_MAP_WRITE_BIT) as GLbitfield,
    },
    C2RustUnnamed_52 {
        flags: (GL_CLIENT_STORAGE_BIT | GL_MAP_READ_BIT | GL_MAP_WRITE_BIT) as GLbitfield,
    },
];

// WINE_DEFAULT_DEBUG_CHANNEL(d3d), WINE_DECLARE_DEBUG_CHANNEL(d3d_perf) and
// WINE_DECLARE_DEBUG_CHANNEL(winediag): debug-channel macro invocations, handled by the crate's DPRINT/WARN/ERR macros.

#[repr(C)]
pub struct light_transformed {
    pub diffuse: wined3d_color,
    pub specular: wined3d_color,
    pub ambient: wined3d_color,
    pub position: wined3d_vec4,
    pub direction: wined3d_vec3,
    pub range: f32,
    pub falloff: f32,
    pub c_att: f32,
    pub l_att: f32,
    pub q_att: f32,
    pub cos_htheta: f32,
    pub cos_hphi: f32,
}
pub type Plight_transformed = *mut light_transformed;

// WINED3D_MAX_SOFTWARE_ACTIVE_LIGHTS (wined3d_private.h)

/// C bitfields are not expressible in Rust; laid out as two uint32_t words (LSB-first):
/// `_bitfield_1`: point_light_count[0:8], spot_light_count[8:16],
///                directional_light_count[16:24], parallel_point_light_count[24:32]
/// `_bitfield_2`: lighting[0], legacy_lighting[1], normalise[2], localviewer[3],
///                fog_coord_mode[4:6], fog_mode[6:8], padding[8:32]
#[repr(C)]
pub struct lights_settings {
    pub lights: [light_transformed; (WINED3D_MAX_SOFTWARE_ACTIVE_LIGHTS as _)],
    pub ambient_light: wined3d_color,
    pub modelview_matrix: wined3d_matrix,
    pub normal_matrix: wined3d_matrix,
    pub position_transformed: wined3d_vec4,
    pub fog_start: f32,
    pub fog_end: f32,
    pub fog_density: f32,
    pub _bitfield_1: u32,
    pub _bitfield_2: u32,
}
pub type Plights_settings = *mut lights_settings;
// C bitfield accessors (bindgen's names: `x()` / `set_x(v)`), unsigned, LSB-first.
impl lights_settings {
    #[inline] pub fn point_light_count(&self) -> u32 { (self._bitfield_1 as u32 >> 0) & 0xff }
    #[inline] pub fn set_point_light_count(&mut self, v: u32) { self._bitfield_1 = ((self._bitfield_1 as u32 & !(0xff << 0)) | ((v & 0xff) << 0)) as _; }
    #[inline] pub fn spot_light_count(&self) -> u32 { (self._bitfield_1 as u32 >> 8) & 0xff }
    #[inline] pub fn set_spot_light_count(&mut self, v: u32) { self._bitfield_1 = ((self._bitfield_1 as u32 & !(0xff << 8)) | ((v & 0xff) << 8)) as _; }
    #[inline] pub fn directional_light_count(&self) -> u32 { (self._bitfield_1 as u32 >> 16) & 0xff }
    #[inline] pub fn set_directional_light_count(&mut self, v: u32) { self._bitfield_1 = ((self._bitfield_1 as u32 & !(0xff << 16)) | ((v & 0xff) << 16)) as _; }
    #[inline] pub fn parallel_point_light_count(&self) -> u32 { (self._bitfield_1 as u32 >> 24) & 0xff }
    #[inline] pub fn set_parallel_point_light_count(&mut self, v: u32) { self._bitfield_1 = ((self._bitfield_1 as u32 & !(0xff << 24)) | ((v & 0xff) << 24)) as _; }
    #[inline] pub fn lighting(&self) -> u32 { (self._bitfield_2 as u32 >> 0) & 0x1 }
    #[inline] pub fn set_lighting(&mut self, v: u32) { self._bitfield_2 = ((self._bitfield_2 as u32 & !(0x1 << 0)) | ((v & 0x1) << 0)) as _; }
    #[inline] pub fn legacy_lighting(&self) -> u32 { (self._bitfield_2 as u32 >> 1) & 0x1 }
    #[inline] pub fn set_legacy_lighting(&mut self, v: u32) { self._bitfield_2 = ((self._bitfield_2 as u32 & !(0x1 << 1)) | ((v & 0x1) << 1)) as _; }
    #[inline] pub fn normalise(&self) -> u32 { (self._bitfield_2 as u32 >> 2) & 0x1 }
    #[inline] pub fn set_normalise(&mut self, v: u32) { self._bitfield_2 = ((self._bitfield_2 as u32 & !(0x1 << 2)) | ((v & 0x1) << 2)) as _; }
    #[inline] pub fn localviewer(&self) -> u32 { (self._bitfield_2 as u32 >> 3) & 0x1 }
    #[inline] pub fn set_localviewer(&mut self, v: u32) { self._bitfield_2 = ((self._bitfield_2 as u32 & !(0x1 << 3)) | ((v & 0x1) << 3)) as _; }
    #[inline] pub fn fog_coord_mode(&self) -> u32 { (self._bitfield_2 as u32 >> 4) & 0x3 }
    #[inline] pub fn set_fog_coord_mode(&mut self, v: u32) { self._bitfield_2 = ((self._bitfield_2 as u32 & !(0x3 << 4)) | ((v & 0x3) << 4)) as _; }
    #[inline] pub fn fog_mode(&self) -> u32 { (self._bitfield_2 as u32 >> 6) & 0x3 }
    #[inline] pub fn set_fog_mode(&mut self, v: u32) { self._bitfield_2 = ((self._bitfield_2 as u32 & !(0x3 << 6)) | ((v & 0x3) << 6)) as _; }
}

// enum wined3d_light_type (C int values)

#[repr(C)]
pub struct gl_memory_types_entry {
    pub flags: GLbitfield,
}

// GL_MAP_READ_BIT = 0x0001, GL_MAP_WRITE_BIT = 0x0002, GL_CLIENT_STORAGE_BIT = 0x0200

// #define copy_and_next(dest, src, size) ... : function-like macro, inlined by the functions (skipped).
// #undef copy_and_next : skipped.
#[no_mangle]
pub unsafe extern "C" fn device_context_add(device: *mut wined3d_device, context: *mut wined3d_context) -> BOOL {
    let mut new_array: *mut *mut wined3d_context = core::ptr::null_mut();

    TRACE!("Adding context %p.\n", context);

    if (*(*device).shader_backend)
        .shader_allocate_context_data
        .unwrap_unchecked()(context)
        == 0
    {
        ERR!("Failed to allocate shader backend context data.\n");
        return (FALSE) as BOOL;
    }
    (*(*device).shader_backend)
        .shader_init_context_state
        .unwrap_unchecked()(context);

    if (*(*(*device).adapter).fragment_pipe)
        .allocate_context_data
        .unwrap_unchecked()(context)
        == 0
    {
        ERR!("Failed to allocate fragment pipeline context data.\n");
        (*(*device).shader_backend)
            .shader_free_context_data
            .unwrap_unchecked()(context);
        return (FALSE) as BOOL;
    }

    new_array = realloc(
        (*device).contexts as *mut core::ffi::c_void,
        (core::mem::size_of::<*mut wined3d_context>() as size_t).wrapping_mul(
            ((*device).context_count.wrapping_add(1)) as size_t,
        ),
    ) as *mut *mut wined3d_context;
    if new_array.is_null() {
        ERR!("Failed to grow the context array.\n");
        (*(*(*device).adapter).fragment_pipe)
            .free_context_data
            .unwrap_unchecked()(context);
        (*(*device).shader_backend)
            .shader_free_context_data
            .unwrap_unchecked()(context);
        return (FALSE) as BOOL;
    }

    let idx = (*device).context_count;
    (*device).context_count = (*device).context_count.wrapping_add(1);
    *new_array.offset(idx as isize) = context;
    (*device).contexts = new_array;

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn device_context_remove(device: *mut wined3d_device, context: *mut wined3d_context) {
    let mut new_array: *mut *mut wined3d_context = core::ptr::null_mut();
    let mut found: BOOL = (FALSE as i32);
    let mut i: UINT = 0;

    (*(*(*device).adapter).fragment_pipe)
        .free_context_data
        .unwrap_unchecked()(context);
    (*(*device).shader_backend)
        .shader_free_context_data
        .unwrap_unchecked()(context);

    i = 0;
    while i < (*device).context_count {
        if *(*device).contexts.offset(i as isize) == context {
            found = (TRUE as i32);
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }

    if found == 0 {
        ERR!("Context %p doesn't exist in context array.\n", context);
        return;
    }

    (*device).context_count = (*device).context_count.wrapping_sub(1);
    if (*device).context_count == 0 {
        free((*device).contexts as *mut core::ffi::c_void);
        (*device).contexts = core::ptr::null_mut();
        return;
    }

    memmove(
        (*device).contexts.offset(i as isize) as *mut core::ffi::c_void,
        (*device).contexts.offset((i + 1) as isize) as *const core::ffi::c_void,
        ((*device).context_count.wrapping_sub(i) as usize)
            .wrapping_mul(core::mem::size_of::<*mut wined3d_context>()),
    );
    new_array = realloc(
        (*device).contexts as *mut core::ffi::c_void,
        ((*device).context_count as usize)
            .wrapping_mul(core::mem::size_of::<*mut wined3d_context>()),
    ) as *mut *mut wined3d_context;
    if new_array.is_null() {
        ERR!("Failed to shrink context array. Oh well.\n");
        return;
    }

    (*device).contexts = new_array;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_incref(device: *mut wined3d_device) -> ULONG {
    let mut refcount: u32 = InterlockedIncrement(core::ptr::addr_of_mut!((*device).ref_) as *mut i32) as u32;
    WINE_TRACE!("%p increasing refcount to %u.\n", device, refcount);
    return refcount as ULONG;
}

unsafe extern "C" fn device_free_so_desc(
    entry: *mut wine_rb_entry,
    context: *mut core::ffi::c_void,
) {
    let s: *mut wined3d_so_desc_entry = (entry as *mut u8)
        .offset(-(core::mem::offset_of!(wined3d_so_desc_entry, entry) as isize))
        as *mut wined3d_so_desc_entry;
    free(s as *mut core::ffi::c_void);
}

unsafe extern "C" fn device_leftover_sampler(
    entry: *mut rb_entry,
    context: *mut core::ffi::c_void,
) {
    let sampler: *mut wined3d_sampler = (entry as *mut u8).offset(-(0 as isize)) as *mut wined3d_sampler;
    let _ = context;
    ERR!("Leftover sampler %p.\n", sampler);
}

unsafe extern "C" fn device_leftover_rasterizer_state(entry: *mut rb_entry, context: *mut ::core::ffi::c_void) {
    let _ = context;
    let mut state: *mut wined3d_rasterizer_state = (entry as *mut u8).offset(-(core::mem::offset_of!(wined3d_rasterizer_state, entry) as isize)) as *mut wined3d_rasterizer_state;
    ERR!("Leftover rasterizer state %p.\n", state);
}

unsafe extern "C" fn device_leftover_blend_state(entry: *mut wine_rb_entry, context: *mut core::ffi::c_void) {
    let _ = context;
    let blend_state: *mut wined3d_blend_state = (entry as *mut u8).offset(-(core::mem::offset_of!(wined3d_blend_state, entry) as isize)) as *mut wined3d_blend_state;

    ERR!("Leftover blend state %p.\n", blend_state);
}

unsafe extern "C" fn device_leftover_depth_stencil_state(entry: *mut wine_rb_entry, context: *mut core::ffi::c_void) {
    let _ = context;
    let state: *mut wined3d_depth_stencil_state = (entry as *mut u8).offset(-(core::mem::offset_of!(wined3d_depth_stencil_state, entry) as isize)) as *mut wined3d_depth_stencil_state;

    ERR!("Leftover depth/stencil state %p.\n", state);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_cleanup(device: *mut wined3d_device) {
    let mut i: ::core::ffi::c_uint = 0;

    if (*device).swapchain_count != 0 {
        wined3d_device_uninit_3d(device);
    }

    wined3d_cs_destroy((*device).cs as *mut wined3d_cs);

    i = 0;
    while (i as usize) < ARRAY_SIZE!((*device).multistate_funcs) as usize {
        free((*device).multistate_funcs[i as usize] as *mut ::core::ffi::c_void);
        (*device).multistate_funcs[i as usize] = ::core::ptr::null_mut::<APPLYSTATEFUNC>();
        i += 1;
    }

    if list_empty(&raw mut (*device).resources) == 0 {
        let mut resource: *mut wined3d_resource = ::core::ptr::null_mut::<wined3d_resource>();

        ERR!("Device released with resources still bound.\n");

        resource = ((*device).resources.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(wined3d_resource, resource_list_entry) as isize),
        ) as *mut wined3d_resource;
        while &raw mut (*resource).resource_list_entry != &raw mut (*device).resources {
            ERR!(
                "Leftover resource %p with type %s (%#x).\n",
                resource,
                debug_d3dresourcetype((*resource).type_),
                (*resource).type_ as ::core::ffi::c_uint
            );

            resource = ((*resource).resource_list_entry.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(wined3d_resource, resource_list_entry) as isize),
            ) as *mut wined3d_resource;
        }
    }

    if !(*device).contexts.is_null() {
        ERR!("Context array not freed!\n");
    }
    if !(*device).hardwareCursor.is_null() {
        DestroyCursor((*device).hardwareCursor);
    }
    (*device).hardwareCursor = core::ptr::null_mut();

    wine_rb_destroy(
        &raw mut (*device).samplers,
        Some(device_leftover_sampler as unsafe extern "C" fn(*mut rb_entry, *mut ::core::ffi::c_void) -> ()),
        core::ptr::null_mut(),
    );
    wine_rb_destroy(
        &raw mut (*device).rasterizer_states,
        Some(device_leftover_rasterizer_state as unsafe extern "C" fn(*mut rb_entry, *mut ::core::ffi::c_void) -> ()),
        core::ptr::null_mut(),
    );
    wine_rb_destroy(
        &raw mut (*device).blend_states,
        Some(device_leftover_blend_state as unsafe extern "C" fn(*mut rb_entry, *mut ::core::ffi::c_void) -> ()),
        core::ptr::null_mut(),
    );
    wine_rb_destroy(
        &raw mut (*device).depth_stencil_states,
        Some(device_leftover_depth_stencil_state as unsafe extern "C" fn(*mut rb_entry, *mut ::core::ffi::c_void) -> ()),
        core::ptr::null_mut(),
    );
    wine_rb_destroy(
        &raw mut (*device).so_descs,
        Some(device_free_so_desc as unsafe extern "C" fn(*mut rb_entry, *mut ::core::ffi::c_void) -> ()),
        core::ptr::null_mut(),
    );

    wined3d_lock_cleanup(&raw mut (*device).bo_map_lock);

    wined3d_decref((*device).wined3d);
    (*device).wined3d = core::ptr::null_mut::<wined3d>();
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_decref(mut device: *mut wined3d_device) -> ULONG {
    let mut refcount: ::core::ffi::c_uint = InterlockedDecrement(
        core::ptr::addr_of_mut!((*device).ref_) as *mut ::core::ffi::c_long,
    ) as ::core::ffi::c_uint;
    WINE_TRACE!("%p decreasing refcount to %u.\n", device, refcount);
    if refcount == 0 {
        wined3d_mutex_lock();
        (*(*(*device).adapter).adapter_ops)
            .adapter_destroy_device
            .unwrap_unchecked()(device);
        WINE_TRACE!("Destroyed device %p.\n", device);
        wined3d_mutex_unlock();
    }
    return refcount as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_blend_state_incref(state: *mut wined3d_blend_state) -> ULONG {
    let mut refcount: u32 = InterlockedIncrement(core::ptr::addr_of_mut!((*state).refcount)) as u32;
    WINE_TRACE!("%p increasing refcount to %u.\n", state, refcount);
    return refcount as ULONG;
}

unsafe extern "C" fn wined3d_blend_state_destroy_object(object: *mut core::ffi::c_void) {
    // TRACE("object %p.\n", object);
    free(object);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_blend_state_decref(state: *mut wined3d_blend_state) -> ULONG {
    let mut refcount: ::core::ffi::c_uint =
        wined3d_atomic_decrement_mutex_lock(&raw mut (*state).refcount as *mut LONG) as ::core::ffi::c_uint;
    let mut device: *mut wined3d_device = (*state).device;
    WINE_TRACE!("%p decreasing refcount to %u.\n", state, refcount);
    if refcount == 0 {
        (*(*state).parent_ops)
            .wined3d_object_destroyed
            .unwrap_unchecked()((*state).parent);
        wined3d_cs_destroy_object(
            (*device).cs as *mut wined3d_cs,
            Some(
                wined3d_blend_state_destroy_object
                    as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
            ),
            state as *mut ::core::ffi::c_void,
        );
        wined3d_mutex_unlock();
    }
    return refcount as ULONG;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_blend_state_get_parent(
    state: *const wined3d_blend_state,
) -> *mut core::ffi::c_void {
    // TRACE("state %p.\n", state);
    (*state).parent
}

unsafe extern "C" fn is_dual_source(mut state: wined3d_blend) -> bool {
    return state as ::core::ffi::c_uint >= WINED3D_BLEND_SRC1COLOR as ::core::ffi::c_int as ::core::ffi::c_uint
        && state as ::core::ffi::c_uint <= WINED3D_BLEND_INVSRC1ALPHA as ::core::ffi::c_int as ::core::ffi::c_uint;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_blend_state_create(
    mut device: *mut wined3d_device,
    mut desc: *const wined3d_blend_state_desc,
    mut parent: *mut core::ffi::c_void,
    mut parent_ops: *const wined3d_parent_ops,
    mut state: *mut *mut wined3d_blend_state,
) -> HRESULT {
    let mut object: *mut wined3d_blend_state = core::ptr::null_mut::<wined3d_blend_state>();

    TRACE!(
        "device %p, desc %p, parent %p, parent_ops %p, state %p.\n",
        device,
        desc,
        parent,
        parent_ops,
        state
    );

    object = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<wined3d_blend_state>() as size_t) as _,
    ) as *mut wined3d_blend_state;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }
    (*object).refcount = 1 as LONG;
    (*object).desc = *desc;
    (*object).parent = parent;
    (*object).parent_ops = parent_ops;
    (*object).device = device;
    (*object).dual_source = ((*desc).rt[0 as usize].enable != 0
        && (is_dual_source((*desc).rt[0 as usize].src) as i32 != 0
            || is_dual_source((*desc).rt[0 as usize].dst) as i32 != 0
            || is_dual_source((*desc).rt[0 as usize].src_alpha) as i32 != 0
            || is_dual_source((*desc).rt[0 as usize].dst_alpha) as i32 != 0))
        as i32 as BOOL;

    TRACE!("Created blend state %p.\n", object);

    *state = object;

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_depth_stencil_state_incref(
    state: *mut wined3d_depth_stencil_state,
) -> ULONG {
    let mut refcount: u32 =
        InterlockedIncrement(core::ptr::addr_of_mut!((*state).refcount) as *mut i32) as u32;
    WINE_TRACE!("%p increasing refcount to %u.\n", state, refcount);
    return refcount as ULONG;
}

unsafe extern "C" fn wined3d_depth_stencil_state_destroy_object(
    object: *mut core::ffi::c_void,
) {
    // TRACE("object %p.\n", object);
    free(object);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_depth_stencil_state_decref(state: *mut wined3d_depth_stencil_state) -> ULONG {
    let refcount: ::core::ffi::c_uint =
        wined3d_atomic_decrement_mutex_lock(core::ptr::addr_of_mut!((*state).refcount) as *mut LONG)
            as ::core::ffi::c_uint;
    let device: *mut wined3d_device = (*state).device;

    TRACE!("%p decreasing refcount to %u.\n", state, refcount);

    if refcount == 0 {
        (*(*state).parent_ops)
            .wined3d_object_destroyed
            .unwrap_unchecked()((*state).parent);
        wined3d_cs_destroy_object(
            (*device).cs as *mut wined3d_cs,
            Some(
                wined3d_depth_stencil_state_destroy_object
                    as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
            ),
            state as *mut ::core::ffi::c_void,
        );
        wined3d_mutex_unlock();
    }

    return refcount as ULONG;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_depth_stencil_state_get_parent(
    state: *const wined3d_depth_stencil_state,
) -> *mut core::ffi::c_void {
    // TRACE("state %p.\n", state);
    (*state).parent
}

unsafe extern "C" fn stencil_op_writes_ds(desc: *const wined3d_stencil_op_desc) -> bool {
    return (*desc).fail_op as u32 != WINED3D_STENCIL_OP_KEEP as u32
        || (*desc).depth_fail_op as u32 != WINED3D_STENCIL_OP_KEEP as u32
        || (*desc).pass_op as u32 != WINED3D_STENCIL_OP_KEEP as u32;
}

unsafe extern "C" fn depth_stencil_state_desc_writes_ds(
    desc: *const wined3d_depth_stencil_state_desc,
) -> bool {
    if (*desc).depth != 0 && (*desc).depth_write != 0 {
        return true;
    }

    if (*desc).stencil != 0 && (*desc).stencil_write_mask != 0 {
        if stencil_op_writes_ds(core::ptr::addr_of!((*desc).front)) || stencil_op_writes_ds(core::ptr::addr_of!((*desc).back)) {
            return true;
        }
    }

    return false;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_depth_stencil_state_create(
    mut device: *mut wined3d_device,
    mut desc: *const wined3d_depth_stencil_state_desc,
    mut parent: *mut ::core::ffi::c_void,
    mut parent_ops: *const wined3d_parent_ops,
    mut state: *mut *mut wined3d_depth_stencil_state,
) -> HRESULT {
    let mut object: *mut wined3d_depth_stencil_state =
        ::core::ptr::null_mut::<wined3d_depth_stencil_state>();
    WINE_TRACE!("device %p, desc %p, parent %p, parent_ops %p, state %p.\n", device, desc, parent, parent_ops, state);
    object = calloc(
        1,
        (::core::mem::size_of::<wined3d_depth_stencil_state>() as size_t) as _,
    ) as *mut wined3d_depth_stencil_state;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }
    (*object).refcount = 1 as LONG;
    (*object).desc = *desc;
    (*object).parent = parent;
    (*object).parent_ops = parent_ops;
    (*object).device = device;
    (*object).writes_ds = depth_stencil_state_desc_writes_ds(desc);
    WINE_TRACE!("Created depth/stencil state %p.\n", object);
    *state = object;
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_rasterizer_state_incref(
    mut state: *mut wined3d_rasterizer_state,
) -> ULONG {
    let mut refcount: ::core::ffi::c_uint = InterlockedIncrement(
        core::ptr::addr_of_mut!((*state).refcount) as *mut ::core::ffi::c_long,
    ) as ::core::ffi::c_uint;
    WINE_TRACE!("%p increasing refcount to %u.\n", state, refcount);
    return refcount as ULONG;
}

unsafe extern "C" fn wined3d_rasterizer_state_destroy_object(object: *mut core::ffi::c_void) {
    // TRACE("object %p.\n", object);
    free(object);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_rasterizer_state_decref(state: *mut wined3d_rasterizer_state) -> ULONG {
    let mut refcount: core::ffi::c_uint =
        wined3d_atomic_decrement_mutex_lock(core::ptr::addr_of_mut!((*state).refcount) as *mut LONG) as core::ffi::c_uint;
    let mut device: *mut wined3d_device = (*state).device;

    TRACE!("%p decreasing refcount to %u.\n", state, refcount);

    if refcount == 0 {
        (((*(*state).parent_ops).wined3d_object_destroyed).unwrap_unchecked())((*state).parent);
        wined3d_cs_destroy_object(
            (*device).cs as *mut wined3d_cs,
            Some(wined3d_rasterizer_state_destroy_object as unsafe extern "C" fn(*mut core::ffi::c_void)),
            state as *mut core::ffi::c_void,
        );
        wined3d_mutex_unlock();
    }

    return refcount as ULONG;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_rasterizer_state_get_parent(
    state: *const wined3d_rasterizer_state,
) -> *mut core::ffi::c_void {
    // TRACE("rasterizer_state %p.\n", state);
    (*state).parent
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_rasterizer_state_create(
    device: *mut wined3d_device,
    desc: *const wined3d_rasterizer_state_desc,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    state: *mut *mut wined3d_rasterizer_state,
) -> HRESULT {
    let mut object: *mut wined3d_rasterizer_state = core::ptr::null_mut();

    object = calloc(1, (core::mem::size_of::<wined3d_rasterizer_state>()) as _) as *mut wined3d_rasterizer_state;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).refcount = 1;
    (*object).desc = *desc;
    (*object).parent = parent;
    (*object).parent_ops = parent_ops;
    (*object).device = device;

    *state = object;

    return WINED3D_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_device_get_swapchain_count(
    device: *const wined3d_device,
) -> UINT {
    // TRACE("device %p.\n", device);
    (*device).swapchain_count
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_get_swapchain(device: *const wined3d_device, swapchain_idx: UINT) -> *mut wined3d_swapchain {
    if swapchain_idx >= (*device).swapchain_count {
        WARN!("swapchain_idx %u >= swapchain_count %u.\n", swapchain_idx, (*device).swapchain_count);
        return core::ptr::null_mut();
    }

    return *(*device).swapchains.offset(swapchain_idx as isize);
}

unsafe extern "C" fn device_load_logo(device: *mut wined3d_device, filename: *const core::ffi::c_char) {
    let mut color_key: wined3d_color_key = core::mem::zeroed();
    let mut desc: wined3d_resource_desc = core::mem::zeroed();
    let mut hbm: HBITMAP = core::ptr::null_mut();
    let mut bm: BITMAP = core::mem::zeroed();
    let mut hr: HRESULT = 0;
    let mut dcb: HDC = core::ptr::null_mut();
    let mut dcs: HDC = core::ptr::null_mut();

    hbm = LoadImageA(
        core::ptr::null_mut(),
        filename as LPCSTR,
        IMAGE_BITMAP as UINT,
        0 as core::ffi::c_int,
        0 as core::ffi::c_int,
        (LR_LOADFROMFILE | LR_CREATEDIBSECTION) as UINT,
    ) as HBITMAP;
    if hbm.is_null() {
        WINE_ERR_!(__wine_dbch_winediag, "Failed to load logo %s.\n", wine_dbgstr_a(filename));
        return;
    }
    GetObjectA(
        hbm as HANDLE,
        core::mem::size_of::<BITMAP>() as core::ffi::c_int,
        core::ptr::addr_of_mut!(bm) as LPVOID,
    );

    'out: {
        dcb = CreateCompatibleDC(core::ptr::null_mut());
        if dcb.is_null() {
            break 'out;
        }
        SelectObject(dcb, hbm as HGDIOBJ);

        desc.resource_type = WINED3D_RTYPE_TEXTURE_2D;
        desc.format = WINED3DFMT_B5G6R5_UNORM;
        desc.multisample_type = WINED3D_MULTISAMPLE_NONE;
        desc.multisample_quality = 0;
        desc.usage = WINED3DUSAGE_DYNAMIC;
        desc.bind_flags = 0;
        desc.access = WINED3D_RESOURCE_ACCESS_GPU;
        desc.width = bm.bmWidth as core::ffi::c_uint;
        desc.height = bm.bmHeight as core::ffi::c_uint;
        desc.depth = 1;
        desc.size = 0;
        hr = wined3d_texture_create(
            device,
            core::ptr::addr_of_mut!(desc),
            1,
            1,
            WINED3D_TEXTURE_CREATE_GET_DC,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of!(wined3d_null_parent_ops),
            core::ptr::addr_of_mut!((*device).logo_texture),
        );
        if hr < 0 {
            ERR!("Wine logo requested, but failed to create texture, hr %#lx.\n", hr);
            break 'out;
        }

        hr = wined3d_texture_get_dc(
            (*device).logo_texture,
            0,
            core::ptr::addr_of_mut!(dcs),
        );
        if hr < 0 {
            wined3d_texture_decref((*device).logo_texture);
            (*device).logo_texture = core::ptr::null_mut();
            break 'out;
        }
        BitBlt(
            dcs,
            0,
            0,
            bm.bmWidth as INT,
            bm.bmHeight as INT,
            dcb,
            0,
            0,
            SRCCOPY,
        );
        wined3d_texture_release_dc((*device).logo_texture, 0, dcs);

        color_key.color_space_low_value = 0;
        color_key.color_space_high_value = 0;
        wined3d_texture_set_color_key(
            (*device).logo_texture,
            WINED3D_CKEY_SRC_BLT,
            core::ptr::addr_of_mut!(color_key),
        );
    }

    if !dcb.is_null() {
        DeleteDC(dcb);
    }
    if !hbm.is_null() {
        DeleteObject(hbm as HGDIOBJ);
    }
}

unsafe extern "C" fn wined3d_device_gl_create_dummy_textures(
    mut device_gl: *mut wined3d_device_gl,
    mut context_gl: *mut wined3d_context_gl,
) {
    let textures: *mut wined3d_dummy_textures = &raw mut (*device_gl).dummy_textures;
    let d3d_info: *const wined3d_d3d_info = (*context_gl).c.d3d_info;
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut i: ::core::ffi::c_uint = 0;
    let mut color: DWORD = 0;

    if ((*d3d_info).wined3d_creation_flags & WINED3D_LEGACY_UNBOUND_RESOURCE_COLOR) != 0 {
        color = 0x000000ff;
    } else {
        color = 0x00000000;
    }

    /* Under DirectX you can sample even if no texture is bound, whereas
     * OpenGL will only allow that when a valid texture is bound.
     * We emulate this by creating dummy textures and binding them
     * to each texture stage when the currently set D3D texture is NULL. */
    wined3d_context_gl_active_texture(context_gl, gl_info, 0);

    (*gl_info).gl_ops.gl.p_glGenTextures.unwrap_unchecked()(
        1,
        &raw mut (*textures).tex_1d,
    );

    (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
        GL_TEXTURE_1D,
        (*textures).tex_1d,
    );
    (*gl_info).gl_ops.gl.p_glTexImage1D.unwrap_unchecked()(
        GL_TEXTURE_1D,
        0,
        (GL_RGBA8 as i32),
        1,
        0,
        GL_RGBA,
        GL_UNSIGNED_INT_8_8_8_8,
        &raw mut color as *const ::core::ffi::c_void,
    );

    (*gl_info).gl_ops.gl.p_glGenTextures.unwrap_unchecked()(
        1,
        &raw mut (*textures).tex_2d,
    );

    (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
        GL_TEXTURE_2D,
        (*textures).tex_2d,
    );
    (*gl_info).gl_ops.gl.p_glTexImage2D.unwrap_unchecked()(
        GL_TEXTURE_2D,
        0,
        (GL_RGBA8 as i32),
        1,
        1,
        0,
        GL_RGBA,
        GL_UNSIGNED_INT_8_8_8_8,
        &raw mut color as *const ::core::ffi::c_void,
    );

    if (*gl_info).supported[(EXT_TEXTURE3D) as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glGenTextures.unwrap_unchecked()(
            1,
            &raw mut (*textures).tex_3d,
        );

        (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
            GL_TEXTURE_3D,
            (*textures).tex_3d,
        );
        ((*gl_info).gl_ops.ext.p_glTexImage3D.unwrap_unchecked())(
            GL_TEXTURE_3D,
            0,
            (GL_RGBA8 as i32),
            1,
            1,
            1,
            0,
            GL_RGBA,
            GL_UNSIGNED_INT_8_8_8_8,
            &raw mut color as *const ::core::ffi::c_void,
        );
    }

    if (*gl_info).supported[(ARB_TEXTURE_CUBE_MAP) as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glGenTextures.unwrap_unchecked()(
            1,
            &raw mut (*textures).tex_cube,
        );

        (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
            GL_TEXTURE_CUBE_MAP,
            (*textures).tex_cube,
        );
        i = GL_TEXTURE_CUBE_MAP_POSITIVE_X;
        while i <= GL_TEXTURE_CUBE_MAP_NEGATIVE_Z {
            (*gl_info).gl_ops.gl.p_glTexImage2D.unwrap_unchecked()(
                i as GLenum,
                0,
                (GL_RGBA8 as i32),
                1,
                1,
                0,
                GL_RGBA,
                GL_UNSIGNED_INT_8_8_8_8,
                &raw mut color as *const ::core::ffi::c_void,
            );
            i = i.wrapping_add(1);
        }
    }

    if (*gl_info).supported[(ARB_TEXTURE_CUBE_MAP_ARRAY) as usize] != 0 {
        let mut cube_array_data: [DWORD; 6] = [0; 6];

        (*gl_info).gl_ops.gl.p_glGenTextures.unwrap_unchecked()(
            1,
            &raw mut (*textures).tex_cube_array,
        );

        (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
            GL_TEXTURE_CUBE_MAP_ARRAY,
            (*textures).tex_cube_array,
        );
        i = 0;
        while (i as usize) < ARRAY_SIZE!(cube_array_data) as usize {
            cube_array_data[i as usize] = color;
            i = i.wrapping_add(1);
        }
        ((*gl_info).gl_ops.ext.p_glTexImage3D.unwrap_unchecked())(
            GL_TEXTURE_CUBE_MAP_ARRAY,
            0,
            (GL_RGBA8 as i32),
            1,
            1,
            6,
            0,
            GL_RGBA,
            GL_UNSIGNED_INT_8_8_8_8,
            &raw mut cube_array_data as *mut DWORD as *const ::core::ffi::c_void,
        );
    }

    if (*gl_info).supported[(EXT_TEXTURE_ARRAY) as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glGenTextures.unwrap_unchecked()(
            1,
            &raw mut (*textures).tex_1d_array,
        );

        (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
            GL_TEXTURE_1D_ARRAY,
            (*textures).tex_1d_array,
        );
        (*gl_info).gl_ops.gl.p_glTexImage2D.unwrap_unchecked()(
            GL_TEXTURE_1D_ARRAY,
            0,
            (GL_RGBA8 as i32),
            1,
            1,
            0,
            GL_RGBA,
            GL_UNSIGNED_INT_8_8_8_8,
            &raw mut color as *const ::core::ffi::c_void,
        );

        (*gl_info).gl_ops.gl.p_glGenTextures.unwrap_unchecked()(
            1,
            &raw mut (*textures).tex_2d_array,
        );

        (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
            GL_TEXTURE_2D_ARRAY,
            (*textures).tex_2d_array,
        );
        ((*gl_info).gl_ops.ext.p_glTexImage3D.unwrap_unchecked())(
            GL_TEXTURE_2D_ARRAY,
            0,
            (GL_RGBA8 as i32),
            1,
            1,
            1,
            0,
            GL_RGBA,
            GL_UNSIGNED_INT_8_8_8_8,
            &raw mut color as *const ::core::ffi::c_void,
        );
    }

    if (*gl_info).supported[(ARB_TEXTURE_BUFFER_OBJECT) as usize] != 0 {
        let mut buffer: GLuint = 0;

        ((*gl_info).gl_ops.ext.p_glGenBuffers.unwrap_unchecked())(
            1,
            &raw mut buffer,
        );
        ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(
            GL_TEXTURE_BUFFER,
            buffer,
        );
        ((*gl_info).gl_ops.ext.p_glBufferData.unwrap_unchecked())(
            GL_TEXTURE_BUFFER,
            core::mem::size_of::<DWORD>() as GLsizeiptr,
            &raw mut color as *const ::core::ffi::c_void,
            GL_STATIC_DRAW,
        );
        ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(
            GL_TEXTURE_BUFFER,
            0,
        );

        (*gl_info).gl_ops.gl.p_glGenTextures.unwrap_unchecked()(
            1,
            &raw mut (*textures).tex_buffer,
        );

        (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
            GL_TEXTURE_BUFFER,
            (*textures).tex_buffer,
        );
        ((*gl_info).gl_ops.ext.p_glTexBuffer.unwrap_unchecked())(
            GL_TEXTURE_BUFFER,
            GL_RGBA8,
            buffer,
        );
        ((*gl_info).gl_ops.ext.p_glDeleteBuffers.unwrap_unchecked())(
            1,
            &raw mut buffer,
        );
    }

    if (*gl_info).supported[(ARB_TEXTURE_MULTISAMPLE) as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glGenTextures.unwrap_unchecked()(
            1,
            &raw mut (*textures).tex_2d_ms,
        );

        (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
            GL_TEXTURE_2D_MULTISAMPLE,
            (*textures).tex_2d_ms,
        );
        ((*gl_info).gl_ops.ext.p_glTexImage2DMultisample.unwrap_unchecked())(
            GL_TEXTURE_2D_MULTISAMPLE,
            1,
            GL_RGBA8,
            1,
            1,
            (GL_TRUE as u8),
        );

        (*gl_info).gl_ops.gl.p_glGenTextures.unwrap_unchecked()(
            1,
            &raw mut (*textures).tex_2d_ms_array,
        );

        (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
            GL_TEXTURE_2D_MULTISAMPLE_ARRAY,
            (*textures).tex_2d_ms_array,
        );
        ((*gl_info).gl_ops.ext.p_glTexImage3DMultisample.unwrap_unchecked())(
            GL_TEXTURE_2D_MULTISAMPLE_ARRAY,
            1,
            GL_RGBA8,
            1,
            1,
            1,
            (GL_TRUE as u8),
        );

        if (*gl_info).supported[(ARB_CLEAR_TEXTURE) as usize] != 0 {
            ((*gl_info).gl_ops.ext.p_glClearTexImage.unwrap_unchecked())(
                (*textures).tex_2d_ms,
                0,
                GL_RGBA,
                GL_UNSIGNED_INT_8_8_8_8,
                &raw mut color as *const ::core::ffi::c_void,
            );
            ((*gl_info).gl_ops.ext.p_glClearTexImage.unwrap_unchecked())(
                (*textures).tex_2d_ms_array,
                0,
                GL_RGBA,
                GL_UNSIGNED_INT_8_8_8_8,
                &raw mut color as *const ::core::ffi::c_void,
            );
        } else {
            WARN!("ARB_clear_texture is currently required to clear dummy multisample textures.\n");
        }
    }

    checkGLcall!(gl_info, "create dummy textures");

    wined3d_context_gl_bind_dummy_textures(context_gl);
}

unsafe extern "C" fn wined3d_device_gl_destroy_dummy_textures(
    device_gl: *mut wined3d_device_gl,
    context_gl: *mut wined3d_context_gl,
) {
    let dummy_textures: *mut wined3d_dummy_textures = core::ptr::addr_of_mut!((*device_gl).dummy_textures);
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;

    if (*gl_info).supported[ARB_TEXTURE_MULTISAMPLE as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glDeleteTextures.unwrap_unchecked()(
            1 as GLsizei,
            core::ptr::addr_of_mut!((*dummy_textures).tex_2d_ms),
        );
        (*gl_info).gl_ops.gl.p_glDeleteTextures.unwrap_unchecked()(
            1 as GLsizei,
            core::ptr::addr_of_mut!((*dummy_textures).tex_2d_ms_array),
        );
    }

    if (*gl_info).supported[ARB_TEXTURE_BUFFER_OBJECT as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glDeleteTextures.unwrap_unchecked()(
            1 as GLsizei,
            core::ptr::addr_of_mut!((*dummy_textures).tex_buffer),
        );
    }

    if (*gl_info).supported[EXT_TEXTURE_ARRAY as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glDeleteTextures.unwrap_unchecked()(
            1 as GLsizei,
            core::ptr::addr_of_mut!((*dummy_textures).tex_2d_array),
        );
        (*gl_info).gl_ops.gl.p_glDeleteTextures.unwrap_unchecked()(
            1 as GLsizei,
            core::ptr::addr_of_mut!((*dummy_textures).tex_1d_array),
        );
    }

    if (*gl_info).supported[ARB_TEXTURE_CUBE_MAP_ARRAY as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glDeleteTextures.unwrap_unchecked()(
            1 as GLsizei,
            core::ptr::addr_of_mut!((*dummy_textures).tex_cube_array),
        );
    }

    if (*gl_info).supported[ARB_TEXTURE_CUBE_MAP as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glDeleteTextures.unwrap_unchecked()(
            1 as GLsizei,
            core::ptr::addr_of_mut!((*dummy_textures).tex_cube),
        );
    }

    if (*gl_info).supported[EXT_TEXTURE3D as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glDeleteTextures.unwrap_unchecked()(
            1 as GLsizei,
            core::ptr::addr_of_mut!((*dummy_textures).tex_3d),
        );
    }

    (*gl_info).gl_ops.gl.p_glDeleteTextures.unwrap_unchecked()(
        1 as GLsizei,
        core::ptr::addr_of_mut!((*dummy_textures).tex_2d),
    );
    (*gl_info).gl_ops.gl.p_glDeleteTextures.unwrap_unchecked()(
        1 as GLsizei,
        core::ptr::addr_of_mut!((*dummy_textures).tex_1d),
    );

    checkGLcall!(gl_info, b"delete dummy textures\0".as_ptr() as *const i8);

    memset(
        dummy_textures as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<wined3d_dummy_textures>(),
    );
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_device_create_default_samplers(device: *mut wined3d_device, context: *mut wined3d_context) {
    let mut desc: wined3d_sampler_desc = core::mem::zeroed();
    let mut hr: HRESULT;

    desc.address_u = WINED3D_TADDRESS_WRAP;
    desc.address_v = WINED3D_TADDRESS_WRAP;
    desc.address_w = WINED3D_TADDRESS_WRAP;
    memset(
        core::ptr::addr_of_mut!(desc.border_color) as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<[core::ffi::c_float; 4]>() as usize,
    );
    desc.mag_filter = WINED3D_TEXF_POINT;
    desc.min_filter = WINED3D_TEXF_POINT;
    desc.mip_filter = WINED3D_TEXF_NONE;
    desc.lod_bias = 0.0f32;
    desc.min_lod = -1000.0f32;
    desc.max_lod = 1000.0f32;
    desc.mip_base_level = 0;
    desc.max_anisotropy = 1;
    desc.compare = (FALSE as i32);
    desc.comparison_func = WINED3D_CMP_NEVER;
    desc.srgb_decode = (TRUE as i32);

    /* In SM4+ shaders there is a separation between resources and samplers. Some shader
     * instructions allow access to resources without using samplers.
     * In GLSL, resources are always accessed through sampler or image variables. The default
     * sampler object is used to emulate the direct resource access when there is no sampler state
     * to use.
     */
    hr = wined3d_sampler_create(
        device,
        core::ptr::addr_of_mut!(desc),
        core::ptr::null_mut(),
        core::ptr::addr_of!(wined3d_null_parent_ops),
        core::ptr::addr_of_mut!((*device).default_sampler),
    );
    if hr < 0 {
        ERR!("Failed to create default sampler, hr %#lx.\n", hr);
        (*device).default_sampler = core::ptr::null_mut();
    }

    /* In D3D10+, a NULL sampler maps to the default sampler state. */
    desc.address_u = WINED3D_TADDRESS_CLAMP;
    desc.address_v = WINED3D_TADDRESS_CLAMP;
    desc.address_w = WINED3D_TADDRESS_CLAMP;
    desc.mag_filter = WINED3D_TEXF_LINEAR;
    desc.min_filter = WINED3D_TEXF_LINEAR;
    desc.mip_filter = WINED3D_TEXF_LINEAR;
    hr = wined3d_sampler_create(
        device,
        core::ptr::addr_of_mut!(desc),
        core::ptr::null_mut(),
        core::ptr::addr_of!(wined3d_null_parent_ops),
        core::ptr::addr_of_mut!((*device).null_sampler),
    );
    if hr < 0 {
        ERR!("Failed to create null sampler, hr %#lx.\n", hr);
        (*device).null_sampler = core::ptr::null_mut();
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_destroy_default_samplers(device: *mut wined3d_device) {
    wined3d_sampler_decref((*device).default_sampler);
    (*device).default_sampler = core::ptr::null_mut();
    wined3d_sampler_decref((*device).null_sampler);
    (*device).null_sampler = core::ptr::null_mut();
}

unsafe extern "C" fn wined3d_null_image_vk_init(
    image: *mut wined3d_image_vk,
    context_vk: *mut wined3d_context_vk,
    vk_command_buffer: VkCommandBuffer,
    type_0: VkImageType,
    layer_count: ::core::ffi::c_uint,
    sample_count: ::core::ffi::c_uint,
) -> bool {
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut range: VkImageSubresourceRange = VkImageSubresourceRange {
        aspectMask: 0,
        baseMipLevel: 0,
        levelCount: 0,
        baseArrayLayer: 0,
        layerCount: 0,
    };
    let mut flags: uint32_t = 0;
    static mut colour: VkClearColorValue = VkClearColorValue {
        float32: [0.0, 0.0, 0.0, 0.0],
    };

    TRACE!(
        "image %p, context_vk %p, vk_command_buffer %p, type %#x, layer_count %u, sample_count %u.\n",
        image,
        context_vk,
        vk_command_buffer,
        type_0 as ::core::ffi::c_uint,
        layer_count,
        sample_count
    );

    if type_0 as ::core::ffi::c_uint == VK_IMAGE_TYPE_2D as ::core::ffi::c_uint
        && layer_count >= 6
    {
        flags |= VK_IMAGE_CREATE_CUBE_COMPATIBLE_BIT;
    }

    if ((wined3d_context_vk_create_image(
        context_vk,
        type_0,
        VK_IMAGE_USAGE_TRANSFER_DST_BIT | VK_IMAGE_USAGE_SAMPLED_BIT,
        VK_FORMAT_R8G8B8A8_UNORM,
        1,
        1,
        1,
        sample_count,
        1,
        layer_count,
        flags,
        image,
    )) == 0) {
        return false;
    }

    wined3d_context_vk_reference_image(context_vk, image);

    range.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
    range.baseMipLevel = 0;
    range.levelCount = 1;
    range.baseArrayLayer = 0;
    range.layerCount = layer_count;

    wined3d_context_vk_image_barrier(
        context_vk,
        vk_command_buffer,
        VK_PIPELINE_STAGE_ALL_COMMANDS_BIT,
        VK_PIPELINE_STAGE_TRANSFER_BIT,
        0,
        VK_ACCESS_TRANSFER_WRITE_BIT,
        VK_IMAGE_LAYOUT_UNDEFINED,
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
        (*image).vk_image,
        &raw mut range,
    );

    ((*vk_info).vk_ops.vkCmdClearColorImage.unwrap_unchecked())(
        vk_command_buffer,
        (*image).vk_image,
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
        &raw const colour,
        1,
        &raw mut range,
    );

    wined3d_context_vk_image_barrier(
        context_vk,
        vk_command_buffer,
        VK_PIPELINE_STAGE_TRANSFER_BIT,
        VK_PIPELINE_STAGE_ALL_COMMANDS_BIT,
        VK_ACCESS_TRANSFER_WRITE_BIT,
        0,
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
        VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
        (*image).vk_image,
        &raw mut range,
    );

    TRACE!(
        "Created NULL image 0x%s, memory 0x%s.\n",
        wine_dbgstr_longlong((*image).vk_image as ULONGLONG),
        wine_dbgstr_longlong((*image).vk_memory as ULONGLONG)
    );

    return true;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_vk_create_null_resources(device_vk: *mut wined3d_device_vk, context_vk: *mut wined3d_context_vk) -> bool {
    let r: *mut wined3d_null_resources_vk = core::ptr::addr_of_mut!((*device_vk).null_resources_vk);
    let mut vk_info: *const wined3d_vk_info;
    let mut format: *const wined3d_format;
    let mut memory_type: VkMemoryPropertyFlags;
    let mut vk_command_buffer: VkCommandBuffer;
    let mut sample_count: core::ffi::c_uint = 2;
    let mut usage: VkBufferUsageFlags;

    format = wined3d_get_format((*device_vk).d.adapter, WINED3DFMT_R8G8B8A8_UNORM, WINED3D_BIND_SHADER_RESOURCE);
    while sample_count != 0 && (sample_count & (*format).multisample_types) == 0 {
        sample_count <<= 1;
    }

    vk_command_buffer = wined3d_context_vk_get_command_buffer(context_vk);
    if vk_command_buffer.is_null() {
        ERR!("Failed to get command buffer.\n");
        return false;
    }

    vk_info = (*context_vk).vk_info;

    usage = VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_UNIFORM_TEXEL_BUFFER_BIT
        | VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT | VK_BUFFER_USAGE_VERTEX_BUFFER_BIT;
    memory_type = VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT;
    if wined3d_context_vk_create_bo(
        context_vk,
        16,
        usage,
        memory_type,
        core::ptr::addr_of_mut!((*r).bo),
    ) == 0
    {
        return false;
    }
    ((*vk_info).vk_ops.vkCmdFillBuffer.unwrap_unchecked())(
        vk_command_buffer,
        (*r).bo.vk_buffer,
        ((*r).bo.b.buffer_offset) as _,
        (*r).bo.size,
        0x00000000,
    );
    (*r).buffer_info.buffer = (*r).bo.vk_buffer;
    (*r).buffer_info.offset = ((*r).bo.b.buffer_offset) as _;
    (*r).buffer_info.range = (*r).bo.size;

    'fail: {
        if wined3d_null_image_vk_init(
            core::ptr::addr_of_mut!((*r).image_1d),
            context_vk,
            vk_command_buffer,
            VK_IMAGE_TYPE_1D,
            1,
            1,
        ) == false
        {
            ERR!("Failed to create 1D image.\n");
            break 'fail;
        }

        if wined3d_null_image_vk_init(
            core::ptr::addr_of_mut!((*r).image_2d),
            context_vk,
            vk_command_buffer,
            VK_IMAGE_TYPE_2D,
            6,
            1,
        ) == false
        {
            ERR!("Failed to create 2D image.\n");
            break 'fail;
        }

        if wined3d_null_image_vk_init(
            core::ptr::addr_of_mut!((*r).image_2dms),
            context_vk,
            vk_command_buffer,
            VK_IMAGE_TYPE_2D,
            1,
            sample_count,
        ) == false
        {
            ERR!("Failed to create 2D MSAA image.\n");
            break 'fail;
        }

        if wined3d_null_image_vk_init(
            core::ptr::addr_of_mut!((*r).image_3d),
            context_vk,
            vk_command_buffer,
            VK_IMAGE_TYPE_3D,
            1,
            1,
        ) == false
        {
            ERR!("Failed to create 3D image.\n");
            break 'fail;
        }

        return true;
    }

    if !((*r).image_2dms.vk_image == 0) {
        wined3d_context_vk_destroy_image(context_vk, core::ptr::addr_of_mut!((*r).image_2dms));
    }
    if !((*r).image_2d.vk_image == 0) {
        wined3d_context_vk_destroy_image(context_vk, core::ptr::addr_of_mut!((*r).image_2d));
    }
    if !((*r).image_1d.vk_image == 0) {
        wined3d_context_vk_destroy_image(context_vk, core::ptr::addr_of_mut!((*r).image_1d));
    }
    wined3d_context_vk_reference_bo(context_vk, core::ptr::addr_of_mut!((*r).bo));
    wined3d_context_vk_destroy_bo(context_vk, core::ptr::addr_of_mut!((*r).bo));
    return false;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_vk_destroy_null_resources(device_vk: *mut wined3d_device_vk, context_vk: *mut wined3d_context_vk) {
    let r: *mut wined3d_null_resources_vk = core::ptr::addr_of_mut!((*device_vk).null_resources_vk);

    /* We don't track command buffer references to NULL resources. We easily
     * could, but it doesn't seem worth it. */
    wined3d_context_vk_reference_image(context_vk, core::ptr::addr_of_mut!((*r).image_3d));
    wined3d_context_vk_destroy_image(context_vk, core::ptr::addr_of_mut!((*r).image_3d));
    wined3d_context_vk_reference_image(context_vk, core::ptr::addr_of_mut!((*r).image_2dms));
    wined3d_context_vk_destroy_image(context_vk, core::ptr::addr_of_mut!((*r).image_2dms));
    wined3d_context_vk_reference_image(context_vk, core::ptr::addr_of_mut!((*r).image_2d));
    wined3d_context_vk_destroy_image(context_vk, core::ptr::addr_of_mut!((*r).image_2d));
    wined3d_context_vk_reference_image(context_vk, core::ptr::addr_of_mut!((*r).image_1d));
    wined3d_context_vk_destroy_image(context_vk, core::ptr::addr_of_mut!((*r).image_1d));
    wined3d_context_vk_reference_bo(context_vk, core::ptr::addr_of_mut!((*r).bo));
    wined3d_context_vk_destroy_bo(context_vk, core::ptr::addr_of_mut!((*r).bo));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_vk_create_null_views(device_vk: *mut wined3d_device_vk, context_vk: *mut wined3d_context_vk) -> bool {
    let r: *mut wined3d_null_resources_vk = core::ptr::addr_of_mut!((*device_vk).null_resources_vk);
    let v: *mut wined3d_null_views_vk = core::ptr::addr_of_mut!((*device_vk).null_views_vk);
    let mut buffer_create_info: VkBufferViewCreateInfo = core::mem::zeroed();
    let mut usage_desc: VkImageViewUsageCreateInfoKHR = core::mem::zeroed();
    let mut vk_info: *const wined3d_vk_info = core::ptr::null();
    let mut view_desc: VkImageViewCreateInfo = core::mem::zeroed();
    let mut vr: VkResult = core::mem::zeroed();

    vk_info = (*context_vk).vk_info;

    buffer_create_info.sType = VK_STRUCTURE_TYPE_BUFFER_VIEW_CREATE_INFO;
    buffer_create_info.pNext = core::ptr::null();
    buffer_create_info.flags = 0;
    buffer_create_info.buffer = (*r).bo.vk_buffer;
    buffer_create_info.format = VK_FORMAT_R32_UINT;
    buffer_create_info.offset = ((*r).bo.b.buffer_offset) as _;
    buffer_create_info.range = (*r).bo.size;

    vr = ((*vk_info).vk_ops.vkCreateBufferView.unwrap_unchecked())(
        (*device_vk).vk_device,
        core::ptr::addr_of_mut!(buffer_create_info),
        core::ptr::null(),
        core::ptr::addr_of_mut!((*v).vk_view_buffer_uint),
    );
    if vr < 0 {
        ERR!("Failed to create buffer view, vr %s.\n", wined3d_debug_vkresult(vr));
        return false;
    }
    TRACE!(
        "Created buffer view 0x%s.\n",
        wine_dbgstr_longlong((*v).vk_view_buffer_uint as ULONGLONG)
    );

    buffer_create_info.format = VK_FORMAT_R32G32B32A32_SFLOAT;
    vr = ((*vk_info).vk_ops.vkCreateBufferView.unwrap_unchecked())(
        (*device_vk).vk_device,
        core::ptr::addr_of_mut!(buffer_create_info),
        core::ptr::null(),
        core::ptr::addr_of_mut!((*v).vk_view_buffer_float),
    );

    'fail: {
        if vr < 0 {
            ERR!("Failed to create buffer view, vr %s.\n", wined3d_debug_vkresult(vr));
            break 'fail;
        }
        TRACE!(
            "Created buffer view 0x%s.\n",
            wine_dbgstr_longlong((*v).vk_view_buffer_float as ULONGLONG)
        );

        view_desc.sType = VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO;
        view_desc.pNext = core::ptr::null();
        view_desc.flags = 0;
        view_desc.image = (*r).image_1d.vk_image;
        view_desc.viewType = VK_IMAGE_VIEW_TYPE_1D;
        view_desc.format = VK_FORMAT_R8G8B8A8_UNORM;
        view_desc.components.r = VK_COMPONENT_SWIZZLE_ZERO;
        view_desc.components.g = VK_COMPONENT_SWIZZLE_ZERO;
        view_desc.components.b = VK_COMPONENT_SWIZZLE_ZERO;
        view_desc.components.a = VK_COMPONENT_SWIZZLE_ZERO;
        view_desc.subresourceRange.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
        view_desc.subresourceRange.baseMipLevel = 0;
        view_desc.subresourceRange.levelCount = 1;
        view_desc.subresourceRange.baseArrayLayer = 0;
        view_desc.subresourceRange.layerCount = 1;

        if (*vk_info).supported[(WINED3D_VK_KHR_MAINTENANCE2) as usize] != 0
            || (*vk_info).api_version >= VK_API_VERSION_1_1
        {
            usage_desc.sType = VK_STRUCTURE_TYPE_IMAGE_VIEW_USAGE_CREATE_INFO_KHR;
            usage_desc.pNext = core::ptr::null();
            usage_desc.usage = VK_IMAGE_USAGE_SAMPLED_BIT;

            view_desc.pNext = core::ptr::addr_of_mut!(usage_desc) as *const core::ffi::c_void;
        }

        vr = ((*vk_info).vk_ops.vkCreateImageView.unwrap_unchecked())(
            (*device_vk).vk_device,
            core::ptr::addr_of_mut!(view_desc),
            core::ptr::null(),
            core::ptr::addr_of_mut!((*v).vk_info_1d.imageView),
        );
        if vr < 0 {
            ERR!("Failed to create 1D image view, vr %s.\n", wined3d_debug_vkresult(vr));
            break 'fail;
        }
        TRACE!(
            "Created 1D image view 0x%s.\n",
            wine_dbgstr_longlong((*v).vk_info_1d.imageView as ULONGLONG)
        );
        (*v).vk_info_1d.sampler = (VK_NULL_HANDLE as _);
        (*v).vk_info_1d.imageLayout = VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL;

        view_desc.viewType = VK_IMAGE_VIEW_TYPE_1D_ARRAY;
        vr = ((*vk_info).vk_ops.vkCreateImageView.unwrap_unchecked())(
            (*device_vk).vk_device,
            core::ptr::addr_of_mut!(view_desc),
            core::ptr::null(),
            core::ptr::addr_of_mut!((*v).vk_info_1d_array.imageView),
        );
        if vr < 0 {
            ERR!("Failed to create 1D image view, vr %s.\n", wined3d_debug_vkresult(vr));
            break 'fail;
        }
        TRACE!(
            "Created 1D array image view 0x%s.\n",
            wine_dbgstr_longlong((*v).vk_info_1d_array.imageView as ULONGLONG)
        );
        (*v).vk_info_1d_array.sampler = (VK_NULL_HANDLE as _);
        (*v).vk_info_1d_array.imageLayout = VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL;

        view_desc.image = (*r).image_2d.vk_image;
        view_desc.viewType = VK_IMAGE_VIEW_TYPE_2D;
        vr = ((*vk_info).vk_ops.vkCreateImageView.unwrap_unchecked())(
            (*device_vk).vk_device,
            core::ptr::addr_of_mut!(view_desc),
            core::ptr::null(),
            core::ptr::addr_of_mut!((*v).vk_info_2d.imageView),
        );
        if vr < 0 {
            ERR!("Failed to create 2D image view, vr %s.\n", wined3d_debug_vkresult(vr));
            break 'fail;
        }
        TRACE!(
            "Created 2D image view 0x%s.\n",
            wine_dbgstr_longlong((*v).vk_info_2d.imageView as ULONGLONG)
        );
        (*v).vk_info_2d.sampler = (VK_NULL_HANDLE as _);
        (*v).vk_info_2d.imageLayout = VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL;

        view_desc.image = (*r).image_2dms.vk_image;
        view_desc.viewType = VK_IMAGE_VIEW_TYPE_2D;
        vr = ((*vk_info).vk_ops.vkCreateImageView.unwrap_unchecked())(
            (*device_vk).vk_device,
            core::ptr::addr_of_mut!(view_desc),
            core::ptr::null(),
            core::ptr::addr_of_mut!((*v).vk_info_2dms.imageView),
        );
        if vr < 0 {
            ERR!("Failed to create 2D MSAA image view, vr %s.\n", wined3d_debug_vkresult(vr));
            break 'fail;
        }
        TRACE!(
            "Created 2D MSAA image view 0x%s.\n",
            wine_dbgstr_longlong((*v).vk_info_2dms.imageView as ULONGLONG)
        );
        (*v).vk_info_2dms.sampler = (VK_NULL_HANDLE as _);
        (*v).vk_info_2dms.imageLayout = VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL;

        view_desc.image = (*r).image_3d.vk_image;
        view_desc.viewType = VK_IMAGE_VIEW_TYPE_3D;
        vr = ((*vk_info).vk_ops.vkCreateImageView.unwrap_unchecked())(
            (*device_vk).vk_device,
            core::ptr::addr_of_mut!(view_desc),
            core::ptr::null(),
            core::ptr::addr_of_mut!((*v).vk_info_3d.imageView),
        );
        if vr < 0 {
            ERR!("Failed to create 3D image view, vr %s.\n", wined3d_debug_vkresult(vr));
            break 'fail;
        }
        TRACE!(
            "Created 3D image view 0x%s.\n",
            wine_dbgstr_longlong((*v).vk_info_3d.imageView as ULONGLONG)
        );
        (*v).vk_info_3d.sampler = (VK_NULL_HANDLE as _);
        (*v).vk_info_3d.imageLayout = VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL;

        view_desc.image = (*r).image_2d.vk_image;
        view_desc.subresourceRange.layerCount = 6;
        view_desc.viewType = VK_IMAGE_VIEW_TYPE_CUBE;
        vr = ((*vk_info).vk_ops.vkCreateImageView.unwrap_unchecked())(
            (*device_vk).vk_device,
            core::ptr::addr_of_mut!(view_desc),
            core::ptr::null(),
            core::ptr::addr_of_mut!((*v).vk_info_cube.imageView),
        );
        if vr < 0 {
            ERR!("Failed to create cube image view, vr %s.\n", wined3d_debug_vkresult(vr));
            break 'fail;
        }
        TRACE!(
            "Created cube image view 0x%s.\n",
            wine_dbgstr_longlong((*v).vk_info_cube.imageView as ULONGLONG)
        );
        (*v).vk_info_cube.sampler = (VK_NULL_HANDLE as _);
        (*v).vk_info_cube.imageLayout = VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL;

        view_desc.subresourceRange.layerCount = 1;
        view_desc.viewType = VK_IMAGE_VIEW_TYPE_2D_ARRAY;
        vr = ((*vk_info).vk_ops.vkCreateImageView.unwrap_unchecked())(
            (*device_vk).vk_device,
            core::ptr::addr_of_mut!(view_desc),
            core::ptr::null(),
            core::ptr::addr_of_mut!((*v).vk_info_2d_array.imageView),
        );
        if vr < 0 {
            ERR!("Failed to create 2D array image view, vr %s.\n", wined3d_debug_vkresult(vr));
            break 'fail;
        }
        TRACE!(
            "Created 2D array image view 0x%s.\n",
            wine_dbgstr_longlong((*v).vk_info_2d_array.imageView as ULONGLONG)
        );
        (*v).vk_info_2d_array.sampler = (VK_NULL_HANDLE as _);
        (*v).vk_info_2d_array.imageLayout = VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL;

        view_desc.image = (*r).image_2dms.vk_image;
        view_desc.viewType = VK_IMAGE_VIEW_TYPE_2D_ARRAY;
        vr = ((*vk_info).vk_ops.vkCreateImageView.unwrap_unchecked())(
            (*device_vk).vk_device,
            core::ptr::addr_of_mut!(view_desc),
            core::ptr::null(),
            core::ptr::addr_of_mut!((*v).vk_info_2dms_array.imageView),
        );
        if vr < 0 {
            ERR!("Failed to create 2D MSAA array image view, vr %s.\n", wined3d_debug_vkresult(vr));
            break 'fail;
        }
        TRACE!(
            "Created 2D MSAA array image view 0x%s.\n",
            wine_dbgstr_longlong((*v).vk_info_2dms_array.imageView as ULONGLONG)
        );
        (*v).vk_info_2dms_array.sampler = (VK_NULL_HANDLE as _);
        (*v).vk_info_2dms_array.imageLayout = VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL;

        view_desc.image = (*r).image_2d.vk_image;
        view_desc.subresourceRange.layerCount = 6;
        view_desc.viewType = VK_IMAGE_VIEW_TYPE_CUBE_ARRAY;
        vr = ((*vk_info).vk_ops.vkCreateImageView.unwrap_unchecked())(
            (*device_vk).vk_device,
            core::ptr::addr_of_mut!(view_desc),
            core::ptr::null(),
            core::ptr::addr_of_mut!((*v).vk_info_cube_array.imageView),
        );
        if vr < 0 {
            ERR!("Failed to create cube array image view, vr %s.\n", wined3d_debug_vkresult(vr));
            break 'fail;
        }
        TRACE!(
            "Created cube array image view 0x%s.\n",
            wine_dbgstr_longlong((*v).vk_info_cube_array.imageView as ULONGLONG)
        );
        (*v).vk_info_cube_array.sampler = (VK_NULL_HANDLE as _);
        (*v).vk_info_cube_array.imageLayout = VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL;

        return true;
    }

    // fail:
    if (*v).vk_info_cube_array.imageView != 0 {
        ((*vk_info).vk_ops.vkDestroyImageView.unwrap_unchecked())(
            (*device_vk).vk_device,
            (*v).vk_info_cube_array.imageView,
            core::ptr::null(),
        );
    }
    if (*v).vk_info_2d_array.imageView != 0 {
        ((*vk_info).vk_ops.vkDestroyImageView.unwrap_unchecked())(
            (*device_vk).vk_device,
            (*v).vk_info_2d_array.imageView,
            core::ptr::null(),
        );
    }
    if (*v).vk_info_cube.imageView != 0 {
        ((*vk_info).vk_ops.vkDestroyImageView.unwrap_unchecked())(
            (*device_vk).vk_device,
            (*v).vk_info_cube.imageView,
            core::ptr::null(),
        );
    }
    if (*v).vk_info_3d.imageView != 0 {
        ((*vk_info).vk_ops.vkDestroyImageView.unwrap_unchecked())(
            (*device_vk).vk_device,
            (*v).vk_info_3d.imageView,
            core::ptr::null(),
        );
    }
    if (*v).vk_info_2dms.imageView != 0 {
        ((*vk_info).vk_ops.vkDestroyImageView.unwrap_unchecked())(
            (*device_vk).vk_device,
            (*v).vk_info_2dms.imageView,
            core::ptr::null(),
        );
    }
    if (*v).vk_info_2d.imageView != 0 {
        ((*vk_info).vk_ops.vkDestroyImageView.unwrap_unchecked())(
            (*device_vk).vk_device,
            (*v).vk_info_2d.imageView,
            core::ptr::null(),
        );
    }
    if (*v).vk_info_1d_array.imageView != 0 {
        ((*vk_info).vk_ops.vkDestroyImageView.unwrap_unchecked())(
            (*device_vk).vk_device,
            (*v).vk_info_1d_array.imageView,
            core::ptr::null(),
        );
    }
    if (*v).vk_info_1d.imageView != 0 {
        ((*vk_info).vk_ops.vkDestroyImageView.unwrap_unchecked())(
            (*device_vk).vk_device,
            (*v).vk_info_1d.imageView,
            core::ptr::null(),
        );
    }
    if (*v).vk_view_buffer_float != 0 {
        ((*vk_info).vk_ops.vkDestroyBufferView.unwrap_unchecked())(
            (*device_vk).vk_device,
            (*v).vk_view_buffer_float,
            core::ptr::null(),
        );
    }
    ((*vk_info).vk_ops.vkDestroyBufferView.unwrap_unchecked())(
        (*device_vk).vk_device,
        (*v).vk_view_buffer_uint,
        core::ptr::null(),
    );
    return false;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_vk_destroy_null_views(device_vk: *mut wined3d_device_vk, context_vk: *mut wined3d_context_vk) {
    let v: *mut wined3d_null_views_vk = core::ptr::addr_of_mut!((*device_vk).null_views_vk);
    let id: uint64_t = (*context_vk).current_command_buffer.id;

    wined3d_context_vk_destroy_vk_image_view(context_vk, (*v).vk_info_cube_array.imageView, id);
    wined3d_context_vk_destroy_vk_image_view(context_vk, (*v).vk_info_2dms_array.imageView, id);
    wined3d_context_vk_destroy_vk_image_view(context_vk, (*v).vk_info_2d_array.imageView, id);
    wined3d_context_vk_destroy_vk_image_view(context_vk, (*v).vk_info_cube.imageView, id);
    wined3d_context_vk_destroy_vk_image_view(context_vk, (*v).vk_info_3d.imageView, id);
    wined3d_context_vk_destroy_vk_image_view(context_vk, (*v).vk_info_2dms.imageView, id);
    wined3d_context_vk_destroy_vk_image_view(context_vk, (*v).vk_info_2d.imageView, id);
    wined3d_context_vk_destroy_vk_image_view(context_vk, (*v).vk_info_1d_array.imageView, id);
    wined3d_context_vk_destroy_vk_image_view(context_vk, (*v).vk_info_1d.imageView, id);

    wined3d_context_vk_destroy_vk_buffer_view(context_vk, (*v).vk_view_buffer_float, id);
    wined3d_context_vk_destroy_vk_buffer_view(context_vk, (*v).vk_view_buffer_uint, id);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_acquire_focus_window(
    device: *mut wined3d_device,
    window: HWND,
) -> HRESULT {
    let mut screensaver_active: core::ffi::c_uint = 0;

    if wined3d_register_window(
        core::ptr::null_mut(),
        window,
        device,
        0,
    ) == 0
    {
        ERR!("Failed to register window %p.\n", window);
        return E_FAIL;
    }

    InterlockedExchangePointer(
        core::ptr::addr_of_mut!((*device).focus_window) as *mut *mut core::ffi::c_void,
        window as *mut core::ffi::c_void,
    );
    SetWindowPos(
        window,
        core::ptr::null_mut(),
        0,
        0,
        0,
        0,
        SWP_NOSIZE | SWP_NOMOVE,
    );
    SystemParametersInfoW(
        SPI_GETSCREENSAVEACTIVE,
        0,
        core::ptr::addr_of_mut!(screensaver_active) as *mut core::ffi::c_void,
        0,
    );
    (*device).set_restore_screensaver((screensaver_active != 0) as _);
    if (*device).restore_screensaver() != 0 {
        SystemParametersInfoW(
            SPI_SETSCREENSAVEACTIVE,
            FALSE,
            core::ptr::null_mut(),
            0,
        );
    }

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_release_focus_window(device: *mut wined3d_device) {
    // TRACE("device %p.\n", device);

    if !(*device).focus_window.is_null() {
        wined3d_unregister_window((*device).focus_window);
    }
    InterlockedExchangePointer(
        core::ptr::addr_of_mut!((*device).focus_window) as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );
    if (*device).restore_screensaver() != 0 {
        SystemParametersInfoW(
            SPI_SETSCREENSAVEACTIVE as u32,
            TRUE as u32,
            core::ptr::null_mut(),
            0 as u32,
        );
        (*device).set_restore_screensaver(FALSE as _);
    }
}

unsafe extern "C" fn device_init_swapchain_state(mut device: *mut wined3d_device, mut swapchain: *mut wined3d_swapchain) {
    let mut views: [*mut wined3d_rendertarget_view; (WINED3D_MAX_RENDER_TARGETS as _)] = [
        ::core::ptr::null_mut::<wined3d_rendertarget_view>(),
        ::core::ptr::null_mut::<wined3d_rendertarget_view>(),
        ::core::ptr::null_mut::<wined3d_rendertarget_view>(),
        ::core::ptr::null_mut::<wined3d_rendertarget_view>(),
        ::core::ptr::null_mut::<wined3d_rendertarget_view>(),
        ::core::ptr::null_mut::<wined3d_rendertarget_view>(),
        ::core::ptr::null_mut::<wined3d_rendertarget_view>(),
        ::core::ptr::null_mut::<wined3d_rendertarget_view>(),
    ];
    let mut ds_enable: BOOL = (*swapchain).state.desc.enable_auto_depth_stencil;
    let mut context: *mut wined3d_device_context = &raw mut (*(*device).cs).c;
    if !(*device).back_buffer_view.is_null() {
        views[0 as usize] = (*device).back_buffer_view;
    }
    wined3d_device_context_set_rendertarget_views(
        context,
        0 as ::core::ffi::c_uint,
        (*(*device).adapter).d3d_info.limits.max_rt_count,
        &raw mut views as *mut *mut wined3d_rendertarget_view,
        !(*device).back_buffer_view.is_null() as BOOL,
    );
    wined3d_device_context_set_depth_stencil_view(
        context,
        if ds_enable != 0 {
            (*device).auto_depth_stencil_view
        } else {
            ::core::ptr::null_mut::<wined3d_rendertarget_view>()
        },
    );
}

unsafe extern "C" fn wined3d_allocator_gl_create_chunk(
    allocator: *mut wined3d_allocator,
    context: *mut wined3d_context,
    memory_type: core::ffi::c_uint,
    chunk_size: size_t,
) -> *mut wined3d_allocator_chunk {
    let mut chunk_gl: *mut wined3d_allocator_chunk_gl = core::ptr::null_mut::<wined3d_allocator_chunk_gl>();
    let mut context_gl: *mut wined3d_context_gl = core::ptr::null_mut::<wined3d_context_gl>();

    if context.is_null() {
        return core::ptr::null_mut::<wined3d_allocator_chunk>();
    }
    context_gl = wined3d_context_gl(context);

    chunk_gl = malloc(core::mem::size_of::<wined3d_allocator_chunk_gl>() as size_t)
        as *mut wined3d_allocator_chunk_gl;
    if chunk_gl.is_null() {
        return core::ptr::null_mut::<wined3d_allocator_chunk>();
    }

    if wined3d_allocator_chunk_init(core::ptr::addr_of_mut!((*chunk_gl).c), allocator) == false {
        free(chunk_gl as *mut core::ffi::c_void);
        return core::ptr::null_mut::<wined3d_allocator_chunk>();
    }

    (*chunk_gl).memory_type = memory_type;
    (*chunk_gl).gl_buffer = wined3d_context_gl_allocate_vram_chunk_buffer(context_gl, memory_type, chunk_size);
    if (*chunk_gl).gl_buffer == 0 {
        wined3d_allocator_chunk_cleanup(core::ptr::addr_of_mut!((*chunk_gl).c));
        free(chunk_gl as *mut core::ffi::c_void);
        return core::ptr::null_mut::<wined3d_allocator_chunk>();
    }
    list_add_head(
        core::ptr::addr_of_mut!((*(*allocator).pools.offset(memory_type as isize)).chunks),
        core::ptr::addr_of_mut!((*chunk_gl).c.entry),
    );

    return core::ptr::addr_of_mut!((*chunk_gl).c);
}

unsafe extern "C" fn wined3d_allocator_gl_destroy_chunk(chunk: *mut wined3d_allocator_chunk) {
    let device_gl: *mut wined3d_device_gl =
        wined3d_device_gl_from_allocator((*chunk).allocator as *mut wined3d_allocator);
    let chunk_gl: *mut wined3d_allocator_chunk_gl = wined3d_allocator_chunk_gl(chunk);
    let mut gl_info: *const wined3d_gl_info = ::core::ptr::null::<wined3d_gl_info>();
    let mut context_gl: *mut wined3d_context_gl = ::core::ptr::null_mut::<wined3d_context_gl>();
    WINE_TRACE!("chunk %p.\n", chunk);
    context_gl = wined3d_context_gl(context_acquire(
        &raw mut (*device_gl).d,
        ::core::ptr::null_mut::<wined3d_texture>(),
        0,
    ));
    gl_info = (*context_gl).gl_info;
    wined3d_context_gl_bind_bo(
        context_gl,
        GL_PIXEL_UNPACK_BUFFER,
        (*chunk_gl).gl_buffer,
    );
    if !(*chunk_gl).c.map_ptr.is_null() {
        ((*gl_info).gl_ops.ext.p_glUnmapBuffer.unwrap_unchecked())(GL_PIXEL_UNPACK_BUFFER);
    }
    ((*gl_info).gl_ops.ext.p_glDeleteBuffers.unwrap_unchecked())(
        1,
        &raw mut (*chunk_gl).gl_buffer,
    );
    WINE_TRACE!("Freed buffer %u.\n", (*chunk_gl).gl_buffer);
    wined3d_allocator_chunk_cleanup(&raw mut (*chunk_gl).c);
    free(chunk_gl as *mut ::core::ffi::c_void);
    context_release(&raw mut (*context_gl).c);
}

unsafe extern "C" fn wined3d_device_gl_find_memory_type(mut flags: GLbitfield) -> ::core::ffi::c_uint {
    let mut i: ::core::ffi::c_uint;
    i = 0;
    while (i as usize) < ARRAY_SIZE!(gl_memory_types) as usize {
        if gl_memory_types[i as usize].flags == flags {
            return i;
        }
        i = i.wrapping_add(1);
    }
    assert!(0 != 0);
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_gl_get_memory_type_flags(
    mut memory_type_idx: ::core::ffi::c_uint,
) -> GLbitfield {
    return gl_memory_types[memory_type_idx as usize].flags;
}

unsafe extern "C" fn wined3d_device_gl_allocate_memory(
    mut device_gl: *mut wined3d_device_gl,
    mut context_gl: *mut wined3d_context_gl,
    mut memory_type: ::core::ffi::c_uint,
    mut size: GLsizeiptr,
    mut id: *mut GLuint,
) -> *mut wined3d_allocator_block {
    let mut allocator: *mut wined3d_allocator = &raw mut (*device_gl).allocator;
    let mut block: *mut wined3d_allocator_block = ::core::ptr::null_mut::<wined3d_allocator_block>();
    wined3d_device_gl_allocator_lock(device_gl);
    if size > (WINED3D_ALLOCATOR_CHUNK_SIZE / 2) as GLsizeiptr {
        if !context_gl.is_null() {
            *id = wined3d_context_gl_allocate_vram_chunk_buffer(context_gl, memory_type, size as size_t);
        }
        wined3d_device_gl_allocator_unlock(device_gl);
        return ::core::ptr::null_mut::<wined3d_allocator_block>();
    }
    block = wined3d_allocator_allocate(
        allocator,
        if !context_gl.is_null() {
            &raw mut (*context_gl).c
        } else {
            ::core::ptr::null_mut::<wined3d_context>()
        },
        memory_type,
        size as size_t,
    );
    if block.is_null() {
        wined3d_device_gl_allocator_unlock(device_gl);
        *id = 0 as GLuint;
        return ::core::ptr::null_mut::<wined3d_allocator_block>();
    }
    *id = (*wined3d_allocator_chunk_gl((*block).chunk)).gl_buffer;
    wined3d_device_gl_allocator_unlock(device_gl);
    WINE_TRACE!("Allocated offset %#Ix from chunk %p.\n", (*block).offset, (*block).chunk);
    return block;
}

unsafe extern "C" fn use_buffer_chunk_suballocation(
    device_gl: *mut wined3d_device_gl,
    gl_info: *const wined3d_gl_info,
    binding: GLenum,
) -> bool {
    match binding {
        GL_ARRAY_BUFFER | GL_ATOMIC_COUNTER_BUFFER | GL_DRAW_INDIRECT_BUFFER | GL_PIXEL_UNPACK_BUFFER | GL_UNIFORM_BUFFER => {
            return true;
        }
        GL_ELEMENT_ARRAY_BUFFER => {
            /* There is no way to specify an element array buffer offset for
             * indirect draws in OpenGL. */
            return ((*(*device_gl).d.wined3d).flags & WINED3D_NO_DRAW_INDIRECT as u32) != 0
                || (*gl_info).supported[ARB_DRAW_INDIRECT as usize] == 0;
        }
        GL_TEXTURE_BUFFER => {
            return (*gl_info).supported[ARB_TEXTURE_BUFFER_RANGE as usize] != 0;
        }
        _ => {
            return false;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_gl_create_bo(device_gl: *mut wined3d_device_gl, context_gl: *mut wined3d_context_gl, size: GLsizeiptr, binding: GLenum, usage: GLenum, coherent: bool, flags: GLbitfield, bo: *mut wined3d_bo_gl) -> bool {
    let gl_info: *const wined3d_gl_info = core::ptr::addr_of!(
        (*wined3d_adapter_gl((*device_gl).d.adapter)).gl_info
    );
    let memory_type_idx: ::core::ffi::c_uint = wined3d_device_gl_find_memory_type(flags);
    let mut memory: *mut wined3d_allocator_block = core::ptr::null_mut();
    let mut buffer_offset: GLsizeiptr = 0;
    let mut id: GLuint = 0;

    TRACE!(
        "device_gl %p, context_gl %p, size %Iu, binding %#x, usage %#x, coherent %#x, flags %#x, bo %p.\n",
        device_gl,
        context_gl,
        size,
        binding,
        usage,
        coherent as ::core::ffi::c_int,
        flags,
        bo
    );

    if (*gl_info).supported[(ARB_BUFFER_STORAGE) as usize] != 0 {
        /* Only suballocate dynamic buffers.
         *
         * We only need suballocation so that we can allocate GL buffers from
         * the client thread and thereby accelerate DISCARD maps.
         *
         * For other buffer types, suballocating means that a whole-buffer
         * upload won't be replacing the whole buffer anymore. If the driver
         * isn't smart enough to track individual buffer ranges then it'll
         * force synchronizing that BO with the GPU. Even using ARB_sync
         * ourselves won't help here, because glBufferSubData() is still
         * implicitly synchronized. */
        if flags & GL_CLIENT_STORAGE_BIT != 0 {
            if use_buffer_chunk_suballocation(device_gl, gl_info, binding) {
                memory = wined3d_device_gl_allocate_memory(
                    device_gl,
                    context_gl,
                    memory_type_idx,
                    size,
                    core::ptr::addr_of_mut!(id),
                );
                if !memory.is_null() {
                    buffer_offset = (*memory).offset as GLsizeiptr;
                } else if context_gl.is_null() {
                    WINE_WARN_!(__wine_dbch_d3d_perf, "Failed to suballocate buffer from the client thread.\n");
                }
            } else if !context_gl.is_null() {
                WINE_WARN_!(__wine_dbch_d3d_perf, "Not allocating chunk memory for binding type %#x.\n", binding);
                id = wined3d_context_gl_allocate_vram_chunk_buffer(context_gl, memory_type_idx, size as size_t);
            }
        } else {
            id = wined3d_context_gl_allocate_vram_chunk_buffer(context_gl, memory_type_idx, size as size_t);
        }

        if id == 0 {
            if !context_gl.is_null() {
                WARN!("Failed to allocate buffer.\n");
            }
            return false;
        }
    } else {
        if context_gl.is_null() {
            return false;
        }

        ((*gl_info).gl_ops.ext.p_glGenBuffers.unwrap_unchecked())(1, core::ptr::addr_of_mut!(id));
        if id == 0 {
            checkGLcall!(gl_info, "buffer object creation");
            return false;
        }

        TRACE!("Created buffer object %u.\n", id);

        wined3d_context_gl_bind_bo(context_gl, binding, id);

        if !coherent && (*gl_info).supported[(APPLE_FLUSH_BUFFER_RANGE) as usize] != 0 {
            ((*gl_info).gl_ops.ext.p_glBufferParameteriAPPLE.unwrap_unchecked())(
                binding,
                GL_BUFFER_FLUSHING_UNMAP_APPLE,
                (GL_FALSE as i32)
            );
            ((*gl_info).gl_ops.ext.p_glBufferParameteriAPPLE.unwrap_unchecked())(
                binding,
                GL_BUFFER_SERIALIZED_MODIFY_APPLE,
                (GL_FALSE as i32)
            );
        }

        ((*gl_info).gl_ops.ext.p_glBufferData.unwrap_unchecked())(binding, size, core::ptr::null(), usage);

        wined3d_context_gl_bind_bo(context_gl, binding, 0);
        checkGLcall!(gl_info, "buffer object creation");
    }

    (*bo).id = id;
    (*bo).memory = memory;
    (*bo).size = size;
    (*bo).binding = binding;
    (*bo).usage = usage;
    (*bo).flags = flags;
    (*bo).b.coherent = coherent;
    list_init(core::ptr::addr_of_mut!((*bo).b.users));
    (*bo).command_fence_id = 0;
    (*bo).b.buffer_offset = buffer_offset as size_t;
    (*bo).b.memory_offset = (*bo).b.buffer_offset;
    (*bo).b.map_ptr = core::ptr::null_mut();
    (*bo).b.client_map_count = 0;
    (*bo).b.refcount = 1;

    true
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_gl_delete_opengl_contexts_cs(mut object: *mut ::core::ffi::c_void) {
    let mut device_gl: *mut wined3d_device_gl = object as *mut wined3d_device_gl;
    let mut context_gl: *mut wined3d_context_gl = ::core::ptr::null_mut::<wined3d_context_gl>();
    let mut context: *mut wined3d_context = ::core::ptr::null_mut::<wined3d_context>();
    let mut device: *mut wined3d_device = ::core::ptr::null_mut::<wined3d_device>();
    let mut shader: *mut wined3d_shader = ::core::ptr::null_mut::<wined3d_shader>();
    WINE_TRACE!("device %p.\n", device_gl);
    device = &raw mut (*device_gl).d;
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
    context = context_acquire(
        device,
        ::core::ptr::null_mut::<wined3d_texture>(),
        0 as ::core::ffi::c_uint,
    );
    context_gl = wined3d_context_gl(context);
    (*(*(*device).blitter).ops)
        .blitter_destroy
        .unwrap_unchecked()((*device).blitter, context);
    (*(*device).shader_backend)
        .shader_free_private
        .unwrap_unchecked()(device, context);
    wined3d_device_gl_destroy_dummy_textures(device_gl, context_gl);
    if (*(*context_gl).c.d3d_info).fences() != 0 {
        wined3d_context_gl_submit_command_fence(context_gl);
        wined3d_context_gl_wait_command_fence(
            context_gl,
            (*wined3d_device_gl((*context_gl).c.device)).current_fence_id
                - 1,
        );
    }
    wined3d_allocator_cleanup(&raw mut (*device_gl).allocator);
    context_release(context);
    while (*device).context_count != 0 {
        wined3d_context_gl_destroy(wined3d_context_gl(
            *(*device).contexts.offset(0 as ::core::ffi::c_int as isize),
        ));
    }
    if !(*device_gl).backup_dc.is_null() {
        WINE_TRACE!("Destroying backup wined3d window %p, dc %p.\n", (*device_gl).backup_wnd, (*device_gl).backup_dc);
        wined3d_release_dc((*device_gl).backup_wnd, (*device_gl).backup_dc);
        DestroyWindow((*device_gl).backup_wnd);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_gl_create_primary_opengl_context_cs(object: *mut ::core::ffi::c_void) {
    let mut device_gl: *mut wined3d_device_gl = object as *mut wined3d_device_gl;
    let mut context_gl: *mut wined3d_context_gl = core::ptr::null_mut();
    let mut swapchain: *mut wined3d_swapchain = core::ptr::null_mut();
    let mut context: *mut wined3d_context = core::ptr::null_mut();
    let mut target: *mut wined3d_texture = core::ptr::null_mut();
    let mut device: *mut wined3d_device = core::ptr::null_mut();
    let mut hr: HRESULT;

    device = core::ptr::addr_of_mut!((*device_gl).d);
    swapchain = *(*device).swapchains.offset(0 as isize);
    target = if !(*swapchain).back_buffers.is_null() {
        *(*swapchain).back_buffers.offset(0 as isize)
    } else {
        (*swapchain).front_buffer
    };
    context = context_acquire(device, target, 0);
    if context.is_null() {
        WARN!("Failed to acquire context.\n");
        return;
    }

    context_gl = wined3d_context_gl(context);

    if !wined3d_allocator_init(
        core::ptr::addr_of_mut!((*device_gl).allocator),
        (ARRAY_SIZE!(gl_memory_types)) as size_t,
        core::ptr::addr_of!(wined3d_allocator_gl_ops),
    ) {
        WARN!("Failed to initialise allocator.\n");
        context_release(context);
        wined3d_context_gl_destroy(wined3d_context_gl(
            *(*device).contexts.offset(0 as isize),
        ));
        return;
    }

    hr = (*(*device).shader_backend)
        .shader_alloc_private
        .unwrap_unchecked()(
        device,
        (*(*device).adapter).vertex_pipe,
        (*(*device).adapter).fragment_pipe,
    );
    if hr < 0 {
        ERR!("Failed to allocate shader private data, hr %#lx.\n", hr);
        wined3d_allocator_cleanup(core::ptr::addr_of_mut!((*device_gl).allocator));
        context_release(context);
        wined3d_context_gl_destroy(wined3d_context_gl(
            *(*device).contexts.offset(0 as isize),
        ));
        return;
    }

    (*device).blitter = wined3d_cpu_blitter_create();
    if (*device).blitter.is_null() {
        ERR!("Failed to create CPU blitter.\n");
        (*(*device).shader_backend)
            .shader_free_private
            .unwrap_unchecked()(
            device,
            core::ptr::null_mut(),
        );
        wined3d_allocator_cleanup(core::ptr::addr_of_mut!((*device_gl).allocator));
        context_release(context);
        wined3d_context_gl_destroy(wined3d_context_gl(
            *(*device).contexts.offset(0 as isize),
        ));
        return;
    }

    wined3d_ffp_blitter_create(core::ptr::addr_of_mut!((*device).blitter), (*context_gl).gl_info);
    wined3d_glsl_blitter_create(core::ptr::addr_of_mut!((*device).blitter), device);
    wined3d_fbo_blitter_create(core::ptr::addr_of_mut!((*device).blitter), (*context_gl).gl_info);
    wined3d_raw_blitter_create(core::ptr::addr_of_mut!((*device).blitter), (*context_gl).gl_info);

    wined3d_device_gl_create_dummy_textures(device_gl, context_gl);
    wined3d_device_create_default_samplers(device, context);
    context_release(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_set_implicit_swapchain(mut device: *mut wined3d_device, mut swapchain: *mut wined3d_swapchain) -> HRESULT {
    let black: wined3d_color = wined3d_color { r: 0.0f32, g: 0.0f32, b: 0.0f32, a: 0.0f32 };
    let mut swapchain_desc: *const wined3d_swapchain_desc = core::ptr::null();
    let mut fb: *mut wined3d_fb_state = core::ptr::addr_of_mut!((*(*(*device).cs).c.state).fb);
    let mut clear_flags: DWORD = 0;
    let mut i: core::ffi::c_uint;
    let mut hr: HRESULT = 0;

    if (*device).d3d_initialized() != 0 {
        return WINED3DERR_INVALIDCALL;
    }

    (*device).swapchain_count = 1;
    (*device).swapchains = calloc(
        ((*device).swapchain_count as size_t) as _,
        (core::mem::size_of::<*mut wined3d_swapchain>() as size_t) as _,
    ) as *mut *mut wined3d_swapchain;
    'err_out: {
        if (*device).swapchains.is_null() {
            ERR!("Failed to allocate swapchain array.\n");
            hr = E_OUTOFMEMORY;
            break 'err_out;
        }
        *(*device).swapchains.offset(0 as isize) = swapchain;

        i = 0;
        while (i as usize) < ARRAY_SIZE!((*fb).render_targets) as usize {
            if !(*fb).render_targets[i as usize].is_null() {
                wined3d_rtv_bind_count_dec((*fb).render_targets[i as usize]);
            }
            i = i.wrapping_add(1);
        }
        memset(
            core::ptr::addr_of_mut!((*fb).render_targets) as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<[*mut wined3d_rendertarget_view; 8]>() as size_t,
        );

        hr = (*(*(*device).adapter).adapter_ops)
            .adapter_init_3d
            .unwrap_unchecked()(device);
        if hr < 0 {
            break 'err_out;
        }
        (*device).set_d3d_initialized((TRUE) as _);

        swapchain_desc = core::ptr::addr_of!((*swapchain).state.desc);
        if (*swapchain_desc).backbuffer_count != 0
            && (*swapchain_desc).backbuffer_bind_flags & WINED3D_BIND_RENDER_TARGET != 0
        {
            let back_buffer: *mut wined3d_resource =
                core::ptr::addr_of_mut!((*(*(*swapchain).back_buffers.add(0 as usize))).resource);
            let mut view_desc: wined3d_view_desc = core::mem::zeroed();

            view_desc.format_id = (*(*back_buffer).format).id;
            view_desc.flags = 0;
            view_desc.u.texture.level_idx = 0;
            view_desc.u.texture.level_count = 1;
            view_desc.u.texture.layer_idx = 0;
            view_desc.u.texture.layer_count = 1;
            hr = wined3d_rendertarget_view_create(
                core::ptr::addr_of_mut!(view_desc),
                back_buffer,
                core::ptr::null_mut(),
                core::ptr::addr_of!(wined3d_null_parent_ops),
                core::ptr::addr_of_mut!((*device).back_buffer_view),
            );
            if hr < 0 {
                ERR!("Failed to create rendertarget view, hr %#lx.\n", hr);
                (*(*(*device).adapter).adapter_ops)
                    .adapter_uninit_3d
                    .unwrap_unchecked()(device);
                (*device).set_d3d_initialized((FALSE) as _);
                break 'err_out;
            }
        }

        device_init_swapchain_state(device, swapchain);

        /* Clear the screen. */
        if !(*device).back_buffer_view.is_null() {
            clear_flags |= WINED3DCLEAR_TARGET;
        }
        if (*swapchain_desc).enable_auto_depth_stencil != 0 {
            clear_flags |= WINED3DCLEAR_ZBUFFER | WINED3DCLEAR_STENCIL;
        }
        if clear_flags != 0 {
            wined3d_device_clear(
                device,
                0,
                core::ptr::null::<RECT>(),
                clear_flags,
                core::ptr::addr_of!(black),
                1.0f32,
                0,
            );
        }

        if !wined3d_settings.logo.is_null() {
            device_load_logo(device, wined3d_settings.logo);
        }

        return WINED3D_OK;
    }

    free((*device).swapchains as *mut core::ffi::c_void);
    (*device).swapchains = core::ptr::null_mut();
    (*device).swapchain_count = 0;

    return hr;
}

unsafe extern "C" fn device_free_sampler(
    entry: *mut wine_rb_entry,
    context: *mut core::ffi::c_void,
) {
    let sampler: *mut wined3d_sampler = (entry as *mut u8)
        .offset(-(core::mem::offset_of!(wined3d_sampler, entry) as isize))
        as *mut wined3d_sampler;
    wined3d_sampler_decref(sampler);
}

unsafe extern "C" fn device_free_rasterizer_state(
    entry: *mut rb_entry,
    context: *mut core::ffi::c_void,
) {
    let state: *mut wined3d_rasterizer_state = (entry as *mut i8)
        .offset(-(core::mem::offset_of!(wined3d_rasterizer_state, entry) as isize))
        as *mut wined3d_rasterizer_state;
    wined3d_rasterizer_state_decref(state);
}

unsafe extern "C" fn device_free_blend_state(
    mut entry: *mut wine_rb_entry,
    mut context: *mut core::ffi::c_void,
) {
    let mut blend_state: *mut wined3d_blend_state = (entry as *mut core::ffi::c_char)
        .offset(-(core::mem::offset_of!(wined3d_blend_state, entry) as isize))
        as *mut wined3d_blend_state;
    wined3d_blend_state_decref(blend_state);
}

unsafe extern "C" fn device_free_depth_stencil_state(entry: *mut rb_entry, context: *mut core::ffi::c_void) {
    let state: *mut wined3d_depth_stencil_state = (entry as *mut u8)
        .sub(core::mem::offset_of!(wined3d_depth_stencil_state, entry))
        as *mut wined3d_depth_stencil_state;

    wined3d_depth_stencil_state_decref(state);
}

unsafe extern "C" fn device_free_ffp_vertex_shader(entry: *mut rb_entry, context: *mut ::core::ffi::c_void) {
    let mut vs: *mut wined3d_ffp_vs = (entry as *mut u8)
        .sub(core::mem::offset_of!(wined3d_ffp_vs, entry)) as *mut wined3d_ffp_vs;
    wined3d_shader_decref((*vs).shader);
    free(vs as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn device_free_ffp_pixel_shader(entry: *mut wine_rb_entry, context: *mut core::ffi::c_void) {
    let mut ps: *mut wined3d_ffp_ps = WINE_RB_ENTRY_VALUE!(entry, wined3d_ffp_ps, entry);

    wined3d_shader_decref((*ps).shader);
    free(ps as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_uninit_3d(device: *mut wined3d_device) {
    let mut state: *mut wined3d_state = (*(*device).cs).c.state;
    let mut resource: *mut wined3d_resource = core::ptr::null_mut();
    let mut cursor: *mut wined3d_resource = core::ptr::null_mut();
    let mut view: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut buffer: *mut wined3d_buffer = core::ptr::null_mut();
    let mut i: u32 = 0;

    if (*device).d3d_initialized() == 0 {
        ERR!("Called while 3D support was not initialised.\n");
        return;
    }

    wined3d_cs_finish((*device).cs, WINED3D_CS_QUEUE_DEFAULT);

    (*device).swapchain_count = 0;

    texture = (*device).logo_texture;
    if !texture.is_null() {
        (*device).logo_texture = core::ptr::null_mut();
        wined3d_texture_decref(texture);
    }

    texture = (*device).cursor_texture;
    if !texture.is_null() {
        (*device).cursor_texture = core::ptr::null_mut();
        wined3d_texture_decref(texture);
    }

    i = 0;
    while (i as usize) < ARRAY_SIZE!((*device).push_constants) as usize {
        buffer = (*device).push_constants[i as usize];
        if !buffer.is_null() {
            wined3d_buffer_decref(buffer);
        }
        i = i.wrapping_add(1);
    }
    memset(
        core::ptr::addr_of_mut!((*device).push_constants) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<[*mut wined3d_buffer; 8]>() as usize,
    );

    wined3d_device_context_emit_reset_state(core::ptr::addr_of_mut!((*(*device).cs).c), true);
    state_cleanup(state);

    wine_rb_destroy(core::ptr::addr_of_mut!((*device).samplers), Some(device_free_sampler), core::ptr::null_mut());
    wine_rb_destroy(core::ptr::addr_of_mut!((*device).rasterizer_states), Some(device_free_rasterizer_state), core::ptr::null_mut());
    wine_rb_destroy(core::ptr::addr_of_mut!((*device).blend_states), Some(device_free_blend_state), core::ptr::null_mut());
    wine_rb_destroy(core::ptr::addr_of_mut!((*device).depth_stencil_states), Some(device_free_depth_stencil_state), core::ptr::null_mut());
    wine_rb_destroy(core::ptr::addr_of_mut!((*device).ffp_vertex_shaders), Some(device_free_ffp_vertex_shader), core::ptr::null_mut());
    wine_rb_destroy(core::ptr::addr_of_mut!((*device).ffp_pixel_shaders), Some(device_free_ffp_pixel_shader), core::ptr::null_mut());

    resource = ((*device).resources.next as *mut core::ffi::c_char).offset(
        -(core::mem::offset_of!(wined3d_resource, resource_list_entry) as isize),
    ) as *mut wined3d_resource;
    cursor = ((*resource).resource_list_entry.next as *mut core::ffi::c_char).offset(
        -(core::mem::offset_of!(wined3d_resource, resource_list_entry) as isize),
    ) as *mut wined3d_resource;
    while core::ptr::addr_of_mut!((*resource).resource_list_entry)
        != core::ptr::addr_of_mut!((*device).resources)
    {
        wined3d_cs_emit_unload_resource((*device).cs, resource);
        resource = cursor;
        cursor = ((*resource).resource_list_entry.next as *mut core::ffi::c_char).offset(
            -(core::mem::offset_of!(wined3d_resource, resource_list_entry) as isize),
        ) as *mut wined3d_resource;
    }

    (*(*(*device).adapter).adapter_ops)
        .adapter_uninit_3d
        .unwrap_unchecked()(device);
    (*device).set_d3d_initialized((FALSE) as _);

    view = (*device).auto_depth_stencil_view;
    if !view.is_null() {
        (*device).auto_depth_stencil_view = core::ptr::null_mut();
        if wined3d_rendertarget_view_decref(view) != 0 {
            ERR!("Something's still holding the auto depth/stencil view (%p).\n", view);
        }
    }

    view = (*device).back_buffer_view;
    if !view.is_null() {
        (*device).back_buffer_view = core::ptr::null_mut();
        wined3d_rendertarget_view_decref(view);
    }

    free((*device).swapchains as *mut core::ffi::c_void);
    (*device).swapchains = core::ptr::null_mut();

    wined3d_state_reset(state, core::ptr::addr_of_mut!((*(*device).adapter).d3d_info));
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_device_set_multithreaded(device: *mut wined3d_device) {
    (*device).create_parms.flags |= WINED3DCREATE_MULTITHREADED as u32;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_get_available_texture_mem(
    device: *const wined3d_device,
) -> UINT {
    let mut driver_info: *const wined3d_driver_info = core::ptr::null::<wined3d_driver_info>();

    driver_info = core::ptr::addr_of!((*(*device).adapter).driver_info);

    WINE_TRACE!(
        "Emulating 0x%s bytes. 0x%s used, returning 0x%s left.\n",
        wine_dbgstr_longlong((*driver_info).vram_bytes as _),
        wine_dbgstr_longlong((*(*device).adapter).vram_bytes_used as _),
        wine_dbgstr_longlong(
            ((*driver_info).vram_bytes as u64)
                .wrapping_sub((*(*device).adapter).vram_bytes_used as u64) as _
        )
    );

    // Clamp and subtract in 64 bits, as the C's UINT64 arithmetic does; only the result
    // narrows. Narrowing first wrapped a >=4 GiB card's VRAM to its low 32 bits.
    (min!(UINT_MAX as u64, (*driver_info).vram_bytes as u64)
        .wrapping_sub((*(*device).adapter).vram_bytes_used as u64)) as UINT
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_get_stream_output(
    context: *mut wined3d_device_context,
    idx: ::core::ffi::c_uint,
    offset: *mut ::core::ffi::c_uint,
) -> *mut wined3d_buffer {
    if idx >= WINED3D_MAX_STREAM_OUTPUT_BUFFERS as ::core::ffi::c_uint {
        WARN!("Invalid stream output %u.\n", idx);
        return ::core::ptr::null_mut::<wined3d_buffer>();
    }

    if !offset.is_null() {
        *offset = (*(*context).state).stream_output[idx as usize].offset;
    }
    return (*(*context).state).stream_output[idx as usize].buffer;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_get_stream_source(
    mut context: *const wined3d_device_context,
    mut stream_idx: ::core::ffi::c_uint,
    mut buffer: *mut *mut wined3d_buffer,
    mut offset: *mut ::core::ffi::c_uint,
    mut stride: *mut ::core::ffi::c_uint,
) -> HRESULT {
    let mut stream: *const wined3d_stream_state = core::ptr::null::<wined3d_stream_state>();

    if stream_idx >= WINED3D_MAX_STREAMS {
        WARN!("Stream index %u out of range.\n", stream_idx);
        return WINED3DERR_INVALIDCALL;
    }

    stream = (core::ptr::addr_of!((*(*context).state).streams) as *const wined3d_stream_state)
        .offset(stream_idx as isize) as *const wined3d_stream_state;
    *buffer = (*stream).buffer as *mut wined3d_buffer;
    if !offset.is_null() {
        *offset = (*stream).offset as ::core::ffi::c_uint;
    }
    *stride = (*stream).stride as ::core::ffi::c_uint;

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_set_clip_status(
    device: *mut wined3d_device,
    clip_status: *const wined3d_clip_status,
) -> HRESULT {
    if clip_status.is_null() {
        return WINED3DERR_INVALIDCALL;
    }

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_get_clip_status(
    device: *const wined3d_device,
    clip_status: *mut wined3d_clip_status,
) -> HRESULT {
    DPRINT1!(
        "device %p, clip_status %p stub!\n",
        device,
        clip_status
    );

    if clip_status.is_null() {
        return WINED3DERR_INVALIDCALL;
    }

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_get_index_buffer(
    context: *const wined3d_device_context,
    format: *mut wined3d_format_id,
    offset: *mut u32,
) -> *mut wined3d_buffer {
    let state: *const wined3d_state = (*context).state;

    *format = (*state).index_format;
    if !offset.is_null() {
        *offset = (*state).index_offset;
    }
    return (*state).index_buffer;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_get_viewports(
    context: *const wined3d_device_context,
    viewport_count: *mut ::core::ffi::c_uint,
    viewports: *mut wined3d_viewport,
) {
    let state: *const wined3d_state = (*context).state;
    let mut count: ::core::ffi::c_uint;

    count = if !viewport_count.is_null() {
        min!(*viewport_count, (*state).viewport_count)
    } else {
        1
    };
    if count != 0 && !viewports.is_null() {
        memcpy(
            viewports as *mut ::core::ffi::c_void,
            &raw const (*state).viewports as *const wined3d_viewport as *const ::core::ffi::c_void,
            (count as usize).wrapping_mul(::core::mem::size_of::<wined3d_viewport>()),
        );
    }
    if !viewport_count.is_null() {
        *viewport_count = (*state).viewport_count;
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_get_blend_state(
    mut context: *const wined3d_device_context,
    mut blend_factor: *mut wined3d_color,
    mut sample_mask: *mut ::core::ffi::c_uint,
) -> *mut wined3d_blend_state {
    let mut state: *const wined3d_state = (*context).state;
    WINE_TRACE!("context %p, blend_factor %p, sample_mask %p.\n", context, blend_factor, sample_mask);
    *blend_factor = (*state).blend_factor;
    *sample_mask = (*state).sample_mask as ::core::ffi::c_uint;
    return (*state).blend_state;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_get_depth_stencil_state(
    context: *const wined3d_device_context,
    stencil_ref: *mut ::core::ffi::c_uint,
) -> *mut wined3d_depth_stencil_state {
    let state: *const wined3d_state = (*context).state;
    WINE_TRACE!("context %p, stencil_ref %p.\n", context, stencil_ref);
    *stencil_ref = (*state).stencil_ref;
    return (*state).depth_stencil_state;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_get_rasterizer_state(
    context: *mut wined3d_device_context,
) -> *mut wined3d_rasterizer_state {
    return (*(*context).state).rasterizer_state;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_get_scissor_rects(context: *const wined3d_device_context, rect_count: *mut u32, rects: *mut RECT) {
    let state: *const wined3d_state = (*context).state;
    let mut count: u32 = 0;

    if !rects.is_null() && {
        count = if !rect_count.is_null() {
            min!(*rect_count, (*state).scissor_rect_count)
        } else {
            1
        };
        count != 0
    } {
        memcpy(
            rects as *mut core::ffi::c_void,
            core::ptr::addr_of!((*state).scissor_rects) as *const core::ffi::c_void,
            (count as usize).wrapping_mul(core::mem::size_of::<RECT>()),
        );
    }
    if !rect_count.is_null() {
        *rect_count = (*state).scissor_rect_count;
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_reset_state(context: *mut wined3d_device_context) {
    TRACE!("context %p.\n", context);

    wined3d_device_context_lock(context);
    state_cleanup((*context).state);
    wined3d_state_reset(
        (*context).state,
        core::ptr::addr_of_mut!((*(*(*context).device).adapter).d3d_info),
    );
    wined3d_device_context_emit_reset_state(context, true);
    wined3d_device_context_unlock(context);
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_set_state(context: *mut wined3d_device_context, state: *mut wined3d_state) {
    let mut i: u32;

    TRACE!("context %p, state %p.\n", context, state);

    wined3d_device_context_lock(context);
    (*context).state = state;
    wined3d_device_context_emit_set_feature_level(context, (*state).feature_level);

    wined3d_device_context_emit_set_rendertarget_views(context, 0 as u32,
            ARRAY_SIZE!((*state).fb.render_targets) as u32,
            &raw mut (*state).fb.render_targets as *mut *mut wined3d_rendertarget_view);

    wined3d_device_context_emit_set_depth_stencil_view(context, (*state).fb.depth_stencil);
    wined3d_device_context_emit_set_vertex_declaration(context, (*state).vertex_declaration);

    wined3d_device_context_emit_set_stream_outputs(context, &raw mut (*state).stream_output as *mut wined3d_stream_output
            as *const wined3d_stream_output);

    wined3d_device_context_emit_set_stream_sources(context, 0 as u32, WINED3D_MAX_STREAMS as u32,
            &raw mut (*state).streams as *mut wined3d_stream_state);

    wined3d_device_context_emit_set_index_buffer(context, (*state).index_buffer,
            (*state).index_format, (*state).index_offset);

    wined3d_device_context_emit_set_predication(context, (*state).predicate, (*state).predicate_value);

    i = 0 as u32;
    while i < WINED3D_SHADER_TYPE_COUNT as u32 {
        wined3d_device_context_emit_set_shader(context, i as wined3d_shader_type, (*state).shader[i as usize]);
        wined3d_device_context_emit_set_constant_buffers(context, i as wined3d_shader_type, 0 as u32, MAX_CONSTANT_BUFFERS as u32,
                core::ptr::addr_of_mut!((*state).cb[i as usize]) as *mut wined3d_constant_buffer_state);
        wined3d_device_context_emit_set_samplers(context, i as wined3d_shader_type, 0 as u32, MAX_SAMPLER_OBJECTS as u32,
                core::ptr::addr_of_mut!((*state).sampler[i as usize]) as *mut *mut wined3d_sampler);
        wined3d_device_context_emit_set_shader_resource_views(context, i as wined3d_shader_type, 0 as u32,
                MAX_SHADER_RESOURCE_VIEWS as u32,
                core::ptr::addr_of_mut!((*state).shader_resource_view[i as usize]) as *mut *mut wined3d_shader_resource_view);
        i = i.wrapping_add(1);
    }

    i = 0 as u32;
    while i < WINED3D_PIPELINE_COUNT as u32 {
        wined3d_device_context_emit_set_unordered_access_views(context, i as wined3d_pipeline, 0 as u32, MAX_UNORDERED_ACCESS_VIEWS as u32,
                core::ptr::addr_of_mut!((*state).unordered_access_view[i as usize]) as *mut *mut wined3d_unordered_access_view,
                core::ptr::null::<u32>());
        i = i.wrapping_add(1);
    }

    wined3d_device_context_emit_set_viewports(context, (*state).viewport_count, &raw mut (*state).viewports as *mut wined3d_viewport);
    wined3d_device_context_emit_set_scissor_rects(context, (*state).scissor_rect_count, &raw mut (*state).scissor_rects as *mut RECT);

    wined3d_device_context_emit_set_blend_state(context, (*state).blend_state, &raw mut (*state).blend_factor, (*state).sample_mask as u32);
    wined3d_device_context_emit_set_depth_stencil_state(context, (*state).depth_stencil_state, (*state).stencil_ref);
    wined3d_device_context_emit_set_rasterizer_state(context, (*state).rasterizer_state);
    wined3d_device_context_unlock(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_get_state(
    mut device: *mut wined3d_device,
) -> *mut wined3d_state {
    WINE_TRACE!("device %p.\n", device);
    return (*(*device).cs).c.state;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_device_get_immediate_context(
    mut device: *mut wined3d_device,
) -> *mut wined3d_device_context {
    TRACE!("device %p.\n", device);
    return core::ptr::addr_of_mut!((*(*device).cs).c);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_get_vertex_declaration(
    context: *const wined3d_device_context,
) -> *mut wined3d_vertex_declaration {
    return (*(*context).state).vertex_declaration;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_set_shader(
    context: *mut wined3d_device_context,
    type_0: wined3d_shader_type,
    shader: *mut wined3d_shader,
) {
    let state: *mut wined3d_state = (*context).state;
    let mut prev: *mut wined3d_shader = core::ptr::null_mut();

    wined3d_device_context_lock(context);
    'out: {
        prev = (*state).shader[type_0 as usize];
        if shader == prev {
            break 'out;
        }

        if !shader.is_null() {
            wined3d_shader_incref(shader);
        }
        (*state).shader[type_0 as usize] = shader;
        wined3d_device_context_emit_set_shader(context, type_0, shader);
        if !prev.is_null() {
            wined3d_shader_decref(prev);
        }
    }
    wined3d_device_context_unlock(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_get_shader(
    context: *const wined3d_device_context,
    type_0: wined3d_shader_type,
) -> *mut wined3d_shader {
    return (*(*context).state).shader[type_0 as usize];
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_set_constant_buffers(
    context: *mut wined3d_device_context,
    type_: wined3d_shader_type,
    start_idx: u32,
    count: u32,
    buffers: *const wined3d_constant_buffer_state,
) {
    let state: *mut wined3d_state = (*context).state;
    let mut i: u32;

    if !wined3d_bound_range(start_idx, count, MAX_CONSTANT_BUFFERS) {
        WARN!("Invalid constant buffer index %u, count %u.\n", start_idx, count);
        return;
    }

    wined3d_device_context_lock(context);
    'out: {
        if memcmp(
            buffers as *const core::ffi::c_void,
            core::ptr::addr_of_mut!((*state).cb[type_ as usize][start_idx as usize]) as *const core::ffi::c_void,
            (count as usize).wrapping_mul(core::mem::size_of::<wined3d_constant_buffer_state>()),
        ) == 0
        {
            break 'out;
        }

        wined3d_device_context_emit_set_constant_buffers(context, type_, start_idx, count, buffers);
        i = 0;
        while i < count {
            let prev: *mut wined3d_buffer =
                (*state).cb[type_ as usize][start_idx.wrapping_add(i) as usize].buffer;
            let buffer: *mut wined3d_buffer = (*buffers.offset(i as isize)).buffer;

            if !buffer.is_null() {
                wined3d_buffer_incref(buffer);
            }
            (*state).cb[type_ as usize][start_idx.wrapping_add(i) as usize] =
                *buffers.offset(i as isize);
            if !prev.is_null() {
                wined3d_buffer_decref(prev);
            }
            i = i.wrapping_add(1);
        }
    }
    wined3d_device_context_unlock(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_set_blend_state(
    context: *mut wined3d_device_context,
    blend_state: *mut wined3d_blend_state,
    blend_factor: *const wined3d_color,
    sample_mask: u32,
) {
    let state: *mut wined3d_state = (*context).state;
    let mut prev: *mut wined3d_blend_state = core::ptr::null_mut();

    TRACE!(
        "context %p, blend_state %p, blend_factor %p, sample_mask %#x.\n",
        context,
        blend_state,
        blend_factor,
        sample_mask
    );

    wined3d_device_context_lock(context);
    prev = (*state).blend_state;
    'out: {
        if prev == blend_state
            && memcmp(
                blend_factor as *const core::ffi::c_void,
                core::ptr::addr_of!((*state).blend_factor) as *const core::ffi::c_void,
                core::mem::size_of::<wined3d_color>() as usize,
            ) == 0
            && sample_mask == (*state).sample_mask
        {
            break 'out;
        }

        if !blend_state.is_null() {
            wined3d_blend_state_incref(blend_state);
        }
        (*state).blend_state = blend_state;
        (*state).blend_factor = *blend_factor;
        (*state).sample_mask = sample_mask;
        wined3d_device_context_emit_set_blend_state(context, blend_state, blend_factor, sample_mask);
        if !prev.is_null() {
            wined3d_blend_state_decref(prev);
        }
    }
    wined3d_device_context_unlock(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_set_depth_stencil_state(
    mut context: *mut wined3d_device_context,
    mut depth_stencil_state: *mut wined3d_depth_stencil_state,
    mut stencil_ref: ::core::ffi::c_uint,
) {
    let mut state: *mut wined3d_state = (*context).state;
    let mut prev: *mut wined3d_depth_stencil_state =
        ::core::ptr::null_mut::<wined3d_depth_stencil_state>();
    WINE_TRACE!("context %p, depth_stencil_state %p, stencil_ref %u.\n", context, depth_stencil_state, stencil_ref);
    wined3d_device_context_lock(context);
    'out: {
        prev = (*state).depth_stencil_state;
        if prev == depth_stencil_state && (*state).stencil_ref == stencil_ref {
            break 'out;
        }

        if !depth_stencil_state.is_null() {
            wined3d_depth_stencil_state_incref(depth_stencil_state);
        }
        (*state).depth_stencil_state = depth_stencil_state;
        (*state).stencil_ref = stencil_ref;
        wined3d_device_context_emit_set_depth_stencil_state(context, depth_stencil_state, stencil_ref);
        if !prev.is_null() {
            wined3d_depth_stencil_state_decref(prev);
        }
    }
    wined3d_device_context_unlock(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_set_rasterizer_state(
    mut context: *mut wined3d_device_context,
    mut rasterizer_state: *mut wined3d_rasterizer_state,
) {
    let mut state: *mut wined3d_state = (*context).state;
    let mut prev: *mut wined3d_rasterizer_state = core::ptr::null_mut::<wined3d_rasterizer_state>();

    wined3d_device_context_lock(context);
    prev = (*state).rasterizer_state;
    'out: {
        if prev == rasterizer_state {
            break 'out;
        }

        if !rasterizer_state.is_null() {
            wined3d_rasterizer_state_incref(rasterizer_state);
        }
        (*state).rasterizer_state = rasterizer_state;
        wined3d_device_context_emit_set_rasterizer_state(context, rasterizer_state);
        if !prev.is_null() {
            wined3d_rasterizer_state_decref(prev);
        }
    }
    wined3d_device_context_unlock(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_set_viewports(
    mut context: *mut wined3d_device_context,
    mut viewport_count: ::core::ffi::c_uint,
    mut viewports: *const wined3d_viewport,
) {
    let mut state: *mut wined3d_state = (*context).state;
    let mut i: ::core::ffi::c_uint = 0;
    WINE_TRACE!("context %p, viewport_count %u, viewports %p.\n", context, viewport_count, viewports);
    i = 0;
    while i < viewport_count {
        WINE_TRACE!("%u: x %.8e, y %.8e, w %.8e, h %.8e, min_z %.8e, max_z %.8e.\n", i, (*viewports.offset(i as isize)).x as ::core::ffi::c_double, (*viewports.offset(i as isize)).y as ::core::ffi::c_double, (*viewports.offset(i as isize)).width as ::core::ffi::c_double, (*viewports.offset(i as isize)).height as ::core::ffi::c_double, (*viewports.offset(i as isize)).min_z as ::core::ffi::c_double, (*viewports.offset(i as isize)).max_z as ::core::ffi::c_double);
        i = i.wrapping_add(1);
    }
    wined3d_device_context_lock(context);
    if viewport_count != 0 {
        memcpy(
            &raw mut (*state).viewports as *mut wined3d_viewport as *mut ::core::ffi::c_void,
            viewports as *const ::core::ffi::c_void,
            (viewport_count as usize)
                .wrapping_mul(::core::mem::size_of::<wined3d_viewport>() as usize),
        );
    } else {
        memset(
            &raw mut (*state).viewports as *mut wined3d_viewport as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<[wined3d_viewport; 16]>() as usize,
        );
    }
    (*state).viewport_count = viewport_count;
    wined3d_device_context_emit_set_viewports(context, viewport_count, viewports);
    wined3d_device_context_unlock(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_set_scissor_rects(
    mut context: *mut wined3d_device_context,
    mut rect_count: ::core::ffi::c_uint,
    mut rects: *const RECT,
) {
    let mut state: *mut wined3d_state = (*context).state;
    let mut i: ::core::ffi::c_uint = 0;
    WINE_TRACE!("context %p, rect_count %u, rects %p.\n", context, rect_count, rects);
    i = 0 as ::core::ffi::c_uint;
    while i < rect_count {
        WINE_TRACE!("%u: %s\n", i, wine_dbgstr_rect(rects.offset(i as isize) as *const RECT));
        i = i.wrapping_add(1);
    }
    wined3d_device_context_lock(context);
    'out: {
        if (*state).scissor_rect_count == rect_count
            && memcmp(
                core::ptr::addr_of_mut!((*state).scissor_rects) as *mut RECT
                    as *const ::core::ffi::c_void,
                rects as *const ::core::ffi::c_void,
                (rect_count as size_t).wrapping_mul(::core::mem::size_of::<RECT>() as size_t),
            ) == 0
        {
            WINE_TRACE!("App is setting the old scissor rectangles over, nothing to do.\n");
            break 'out;
        }

        if rect_count != 0 {
            memcpy(
                core::ptr::addr_of_mut!((*state).scissor_rects) as *mut RECT
                    as *mut ::core::ffi::c_void,
                rects as *const ::core::ffi::c_void,
                (rect_count as size_t).wrapping_mul(::core::mem::size_of::<RECT>() as size_t),
            );
        } else {
            memset(
                core::ptr::addr_of_mut!((*state).scissor_rects) as *mut RECT
                    as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<[RECT; 16]>() as size_t,
            );
        }
        (*state).scissor_rect_count = rect_count;

        wined3d_device_context_emit_set_scissor_rects(context, rect_count, rects);
    }
    wined3d_device_context_unlock(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_set_shader_resource_views(context: *mut wined3d_device_context, type_: wined3d_shader_type, start_idx: u32, count: u32, views: *const *mut wined3d_shader_resource_view) {
    let mut real_views: [*mut wined3d_shader_resource_view; (MAX_SHADER_RESOURCE_VIEWS as _)] = [core::ptr::null_mut(); (MAX_SHADER_RESOURCE_VIEWS as _)];
    let state: *mut wined3d_state = (*context).state;
    let dsv: *const wined3d_rendertarget_view = (*state).fb.depth_stencil;
    let mut i: u32;

    if !wined3d_bound_range(start_idx, count, MAX_SHADER_RESOURCE_VIEWS) {
        WARN!("Invalid view index %u, count %u.\n", start_idx, count);
        return;
    }

    wined3d_device_context_lock(context);
    'out: {
        if memcmp(
            views as *const core::ffi::c_void,
            core::ptr::addr_of_mut!((*state).shader_resource_view[type_ as usize][start_idx as usize]) as *const core::ffi::c_void,
            (count as usize).wrapping_mul(core::mem::size_of::<*mut wined3d_shader_resource_view>()),
        ) == 0
        {
            break 'out;
        }

        memcpy(
            core::ptr::addr_of_mut!(real_views) as *mut core::ffi::c_void,
            views as *const core::ffi::c_void,
            (count as usize).wrapping_mul(core::mem::size_of::<*mut wined3d_shader_resource_view>()),
        );

        i = 0;
        while i < count {
            let view: *mut wined3d_shader_resource_view = real_views[i as usize];

            if !view.is_null()
                && (wined3d_is_srv_rtv_bound(state, view)
                    || (!dsv.is_null()
                        && (*dsv).resource == (*view).resource
                        && wined3d_dsv_srv_conflict(dsv, (*view).format) != 0))
            {
                WARN!("Application is trying to bind resource which is attached as render target.\n");
                real_views[i as usize] = core::ptr::null_mut();
            }
            i = i.wrapping_add(1);
        }

        wined3d_device_context_emit_set_shader_resource_views(
            context,
            type_,
            start_idx,
            count,
            core::ptr::addr_of_mut!(real_views) as *mut *mut wined3d_shader_resource_view,
        );
        i = 0;
        while i < count {
            let prev: *mut wined3d_shader_resource_view =
                (*state).shader_resource_view[type_ as usize][start_idx.wrapping_add(i) as usize];
            let view: *mut wined3d_shader_resource_view = real_views[i as usize];

            if !view.is_null() {
                wined3d_shader_resource_view_incref(view);
                wined3d_srv_bind_count_inc(view);
            }

            (*state).shader_resource_view[type_ as usize][start_idx.wrapping_add(i) as usize] = view;
            if !prev.is_null() {
                wined3d_srv_bind_count_dec(prev);
                wined3d_shader_resource_view_decref(prev);
            }
            i = i.wrapping_add(1);
        }
    }
    wined3d_device_context_unlock(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_set_samplers(context: *mut wined3d_device_context, type_: wined3d_shader_type, start_idx: core::ffi::c_uint, count: core::ffi::c_uint, samplers: *const *mut wined3d_sampler) {
    let state: *mut wined3d_state = (*context).state;
    let mut i: core::ffi::c_uint = 0;

    if !(wined3d_bound_range(start_idx, count, MAX_SAMPLER_OBJECTS) != false) {
        WARN!("Invalid sampler index %u, count %u.\n", start_idx, count);
        return;
    }

    wined3d_device_context_lock(context);
    'out: {
        if memcmp(
            samplers as *const core::ffi::c_void,
            addr_of_mut!((*state).sampler[type_ as usize][start_idx as usize]) as *const core::ffi::c_void,
            (count as usize).wrapping_mul(core::mem::size_of::<*mut wined3d_sampler>()),
        ) == 0
        {
            break 'out;
        }

        wined3d_device_context_emit_set_samplers(context, type_, start_idx, count, samplers);
        i = 0;
        while i < count {
            let mut prev: *mut wined3d_sampler = (*state).sampler[type_ as usize][start_idx.wrapping_add(i) as usize];
            let mut sampler: *mut wined3d_sampler = *samplers.offset(i as isize);

            if !sampler.is_null() {
                wined3d_sampler_incref(sampler);
            }
            (*state).sampler[type_ as usize][start_idx.wrapping_add(i) as usize] = sampler;
            if !prev.is_null() {
                wined3d_sampler_decref(prev);
            }
            i = i.wrapping_add(1);
        }
    }
    wined3d_device_context_unlock(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_set_unordered_access_views(
    context: *mut wined3d_device_context,
    pipeline: wined3d_pipeline,
    start_idx: ::core::ffi::c_uint,
    count: ::core::ffi::c_uint,
    uavs: *const *mut wined3d_unordered_access_view,
    initial_counts: *const ::core::ffi::c_uint,
) {
    let state: *mut wined3d_state = (*context).state;
    let mut i: ::core::ffi::c_uint;

    TRACE!(
        "context %p, pipeline %#x, start_idx %u, count %u, uavs %p, initial_counts %p.\n",
        context,
        pipeline as ::core::ffi::c_uint,
        start_idx,
        count,
        uavs,
        initial_counts
    );

    if !wined3d_bound_range(start_idx, count, MAX_UNORDERED_ACCESS_VIEWS) {
        WARN!("Invalid UAV index %u, count %u.\n", start_idx, count);
        return;
    }

    wined3d_device_context_lock(context);
    'out: {
        if memcmp(
            uavs as *const ::core::ffi::c_void,
            core::ptr::addr_of_mut!((*state).unordered_access_view[pipeline as usize][start_idx as usize])
                as *const ::core::ffi::c_void,
            (count as usize) * core::mem::size_of::<*mut wined3d_unordered_access_view>(),
        ) == 0
            && initial_counts.is_null()
        {
            break 'out;
        }

        wined3d_device_context_emit_set_unordered_access_views(context, pipeline, start_idx, count, uavs, initial_counts);
        i = 0 as ::core::ffi::c_uint;
        while i < count {
            let prev: *mut wined3d_unordered_access_view =
                (*state).unordered_access_view[pipeline as usize]
                    [start_idx.wrapping_add(i) as usize];
            let uav: *mut wined3d_unordered_access_view = *uavs.offset(i as isize);

            if !uav.is_null() {
                wined3d_unordered_access_view_incref(uav);
            }
            (*state).unordered_access_view[pipeline as usize][start_idx.wrapping_add(i) as usize] =
                uav;
            if !prev.is_null() {
                wined3d_unordered_access_view_decref(prev);
            }
            i = i.wrapping_add(1);
        }
    }
    wined3d_device_context_unlock(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_set_render_targets_and_unordered_access_views(
    context: *mut wined3d_device_context,
    rtv_count: ::core::ffi::c_uint,
    render_target_views: *const *mut wined3d_rendertarget_view,
    depth_stencil_view: *mut wined3d_rendertarget_view,
    uav_count: UINT,
    unordered_access_views: *const *mut wined3d_unordered_access_view,
    initial_counts: *const ::core::ffi::c_uint,
) {
    'out: {
        wined3d_device_context_lock(context);
        if rtv_count != !(0 as ::core::ffi::c_uint) {
            if !depth_stencil_view.is_null()
                && ((*(*depth_stencil_view).resource).bind_flags
                    & (WINED3D_BIND_DEPTH_STENCIL as ::core::ffi::c_uint))
                    == 0
            {
                WARN!(
                    "View resource %p has incompatible %s bind flags.\n",
                    (*depth_stencil_view).resource,
                    wined3d_debug_bind_flags((*(*depth_stencil_view).resource).bind_flags as uint32_t)
                );
                break 'out;
            }

            if (wined3d_device_context_set_rendertarget_views(
                context,
                0,
                rtv_count,
                render_target_views,
                (FALSE) as BOOL,
            )) < 0
            {
                break 'out;
            }

            wined3d_device_context_set_depth_stencil_view(context, depth_stencil_view);
        }

        if uav_count != !(0 as ::core::ffi::c_uint) {
            wined3d_device_context_set_unordered_access_views(
                context,
                WINED3D_PIPELINE_GRAPHICS,
                0 as ::core::ffi::c_uint,
                uav_count as ::core::ffi::c_uint,
                unordered_access_views,
                initial_counts,
            );
        }
    }
    wined3d_device_context_unlock(context);
}

unsafe extern "C" fn wined3d_device_context_unbind_srv_for_rtv(
    mut context: *mut wined3d_device_context,
    mut view: *const wined3d_rendertarget_view,
    mut dsv: BOOL,
) {
    let state: *const wined3d_state = (*context).state;
    let mut resource: *const wined3d_resource = core::ptr::null::<wined3d_resource>();

    if view.is_null() {
        return;
    }
    resource = (*view).resource;

    if (*resource).srv_bind_count_device != 0 {
        let mut srv: *const wined3d_shader_resource_view = core::ptr::null::<wined3d_shader_resource_view>();
        let mut i: core::ffi::c_uint;
        let mut j: core::ffi::c_uint;

        i = 0;
        while i < WINED3D_SHADER_TYPE_COUNT {
            j = 0;
            while j < MAX_SHADER_RESOURCE_VIEWS {
                srv = (*state).shader_resource_view[i as usize][j as usize];
                if !srv.is_null()
                    && (*srv).resource == resource as *mut wined3d_resource
                    && ((dsv == 0 && wined3d_is_srv_rtv_bound(state, srv) != false)
                        || (dsv != 0 && wined3d_dsv_srv_conflict(view, (*srv).format) != 0))
                {
                    static mut null_srv: *mut wined3d_shader_resource_view =
                        core::ptr::null_mut();

                    WARN!("Application sets bound resource as render target.\n");
                    wined3d_device_context_set_shader_resource_views(
                        context,
                        i as wined3d_shader_type,
                        j,
                        1,
                        core::ptr::addr_of!(null_srv),
                    );
                }
                j = j.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_set_rendertarget_views(context: *mut wined3d_device_context, start_idx: ::core::ffi::c_uint, mut count: ::core::ffi::c_uint, views: *const *mut wined3d_rendertarget_view, set_viewport: BOOL) -> HRESULT {
    let state: *mut wined3d_state = (*context).state;
    let mut i: ::core::ffi::c_uint;
    let max_rt_count: ::core::ffi::c_uint;

    max_rt_count = (*(*(*context).device).adapter).d3d_info.limits.max_rt_count;
    if start_idx >= max_rt_count {
        WARN!("Only %u render targets are supported.\n", max_rt_count);
        return WINED3DERR_INVALIDCALL;
    }
    count = min!(count, max_rt_count - start_idx);

    i = 0;
    while i < count {
        if !(*views.offset(i as isize)).is_null()
            && ((*(**views.offset(i as isize)).resource).bind_flags & (WINED3D_BIND_RENDER_TARGET as ::core::ffi::c_uint)) == 0
        {
            WARN!("View resource %p doesn't have render target bind flags.\n", (**views.offset(i as isize)).resource);
            return WINED3DERR_INVALIDCALL;
        }
        i = i.wrapping_add(1);
    }

    wined3d_device_context_lock(context);
    /* Set the viewport and scissor rectangles, if requested. Tests show that
     * stateblock recording is ignored, the change goes directly into the
     * primary stateblock. */
    if start_idx == 0 && set_viewport != 0 {
        (*state).viewports[0 as usize].x = 0.0;
        (*state).viewports[0 as usize].y = 0.0;
        (*state).viewports[0 as usize].width = (**views.offset(0 as isize)).width as f32;
        (*state).viewports[0 as usize].height = (**views.offset(0 as isize)).height as f32;
        (*state).viewports[0 as usize].min_z = 0.0f32;
        (*state).viewports[0 as usize].max_z = 1.0f32;
        (*state).viewport_count = 1;
        wined3d_device_context_emit_set_viewports(context, 1, core::ptr::addr_of_mut!((*state).viewports) as *mut wined3d_viewport);

        SetRect(
            core::ptr::addr_of_mut!((*state).scissor_rects[0 as usize]) as LPRECT,
            0,
            0,
            (**views.offset(0 as isize)).width as ::core::ffi::c_int,
            (**views.offset(0 as isize)).height as ::core::ffi::c_int,
        );
        (*state).scissor_rect_count = 1;
        wined3d_device_context_emit_set_scissor_rects(context, 1, core::ptr::addr_of_mut!((*state).scissor_rects) as *mut RECT);
    }

    'out: {
        if memcmp(
            views as *const ::core::ffi::c_void,
            (core::ptr::addr_of_mut!((*state).fb.render_targets) as *mut *mut wined3d_rendertarget_view).offset(start_idx as isize)
                as *const ::core::ffi::c_void,
            (count as usize).wrapping_mul(::core::mem::size_of::<*mut wined3d_rendertarget_view>()),
        ) == 0
        {
            break 'out;
        }

        wined3d_device_context_emit_set_rendertarget_views(context, start_idx, count, views);
        i = 0;
        while i < count {
            let mut prev: *mut wined3d_rendertarget_view = (*state).fb.render_targets[start_idx.wrapping_add(i) as usize];
            let view: *mut wined3d_rendertarget_view = *views.offset(i as isize);

            if !view.is_null() {
                wined3d_rendertarget_view_incref(view);
                wined3d_rtv_bind_count_inc(view);
            }
            (*state).fb.render_targets[start_idx.wrapping_add(i) as usize] = view;
            /* Release after the assignment, to prevent device_resource_released()
             * from seeing the resource as still in use. */
            if !prev.is_null() {
                wined3d_rtv_bind_count_dec(prev);
                wined3d_rendertarget_view_decref(prev);
            }

            wined3d_device_context_unbind_srv_for_rtv(context, view, (FALSE) as BOOL);
            i = i.wrapping_add(1);
        }
    }
    wined3d_device_context_unlock(context);
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_set_depth_stencil_view(
    mut context: *mut wined3d_device_context,
    mut view: *mut wined3d_rendertarget_view,
) -> HRESULT {
    let mut fb: *mut wined3d_fb_state = core::ptr::addr_of_mut!((*(*context).state).fb);
    let mut prev: *mut wined3d_rendertarget_view = core::ptr::null_mut();

    if !view.is_null() && ((*(*view).resource).bind_flags & WINED3D_BIND_DEPTH_STENCIL) == 0 {
        WARN!(
            "View resource %p has incompatible %s bind flags.\n",
            (*view).resource,
            wined3d_debug_bind_flags((*(*view).resource).bind_flags as uint32_t)
        );
        return WINED3DERR_INVALIDCALL;
    }

    wined3d_device_context_lock(context);
    prev = (*fb).depth_stencil;
    'out: {
        if prev == view {
            TRACE!("Trying to do a NOP SetRenderTarget operation.\n");
            break 'out;
        }

        (*fb).depth_stencil = view;
        if !(*fb).depth_stencil.is_null() {
            wined3d_rendertarget_view_incref(view);
        }
        wined3d_device_context_emit_set_depth_stencil_view(context, view);
        if !prev.is_null() {
            wined3d_rendertarget_view_decref(prev);
        }
        wined3d_device_context_unbind_srv_for_rtv(context, view, (TRUE) as BOOL);
    }
    wined3d_device_context_unlock(context);
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_set_predication(
    mut context: *mut wined3d_device_context,
    mut predicate: *mut wined3d_query,
    mut value: BOOL,
) {
    let mut state: *mut wined3d_state = (*context).state;
    let mut prev: *mut wined3d_query = core::ptr::null_mut();

    TRACE!("context %p, predicate %p, value %#x.\n", context, predicate, value);
    wined3d_device_context_lock(context);
    prev = (*state).predicate;
    if !predicate.is_null() {
        FIXME!("Predicated rendering not implemented.\n");
        wined3d_query_incref(predicate);
    }
    (*state).predicate = predicate;
    (*state).predicate_value = value;
    wined3d_device_context_emit_set_predication(context, predicate, value);
    if !prev.is_null() {
        wined3d_query_decref(prev);
    }
    wined3d_device_context_unlock(context);
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_set_stream_sources(
    context: *mut wined3d_device_context,
    start_idx: ::core::ffi::c_uint,
    mut count: ::core::ffi::c_uint,
    streams: *const wined3d_stream_state,
) -> HRESULT {
    let state: *mut wined3d_state = (*context).state;
    let mut i: ::core::ffi::c_uint = 0;

    if start_idx >= WINED3D_MAX_STREAMS {
        WARN!("Start index %u is out of range.\n", start_idx);
        return WINED3DERR_INVALIDCALL;
    }

    count = min!(count, WINED3D_MAX_STREAMS - start_idx);

    i = 0;
    while i < count {
        if (*streams.add(i as usize)).offset & 0x3 != 0 {
            WARN!(
                "Offset %u is not 4 byte aligned.\n",
                (*streams.add(i as usize)).offset
            );
            return WINED3DERR_INVALIDCALL;
        }
        i = i.wrapping_add(1);
    }

    'out: {
        wined3d_device_context_lock(context);
        if memcmp(
            streams as *const ::core::ffi::c_void,
            (core::ptr::addr_of_mut!((*state).streams) as *mut wined3d_stream_state)
                .add(start_idx as usize) as *const ::core::ffi::c_void,
            (count as usize).wrapping_mul(core::mem::size_of::<wined3d_stream_state>()),
        ) == 0
        {
            break 'out;
        }

        wined3d_device_context_emit_set_stream_sources(context, start_idx, count, streams);
        i = 0;
        while i < count {
            let prev: *mut wined3d_buffer =
                (*state).streams[start_idx.wrapping_add(i) as usize].buffer as *mut wined3d_buffer;
            let buffer: *mut wined3d_buffer =
                (*streams.add(i as usize)).buffer as *mut wined3d_buffer;

            (*state).streams[start_idx.wrapping_add(i) as usize] = *streams.add(i as usize);

            if !buffer.is_null() {
                wined3d_buffer_incref(buffer);
            }
            if !prev.is_null() {
                wined3d_buffer_decref(prev);
            }
            i = i.wrapping_add(1);
        }
    }
    wined3d_device_context_unlock(context);
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_set_index_buffer(
    context: *mut wined3d_device_context,
    buffer: *mut wined3d_buffer,
    format_id: wined3d_format_id,
    offset: u32,
) {
    let state: *mut wined3d_state = (*context).state;
    let mut prev_format: wined3d_format_id;
    let mut prev_buffer: *mut wined3d_buffer;
    let mut prev_offset: u32;

    wined3d_device_context_lock(context);
    prev_buffer = (*state).index_buffer;
    prev_format = (*state).index_format;
    prev_offset = (*state).index_offset;

    'out: {
        if prev_buffer == buffer && prev_format == format_id && prev_offset == offset {
            break 'out;
        }

        if !buffer.is_null() {
            wined3d_buffer_incref(buffer);
        }
        (*state).index_buffer = buffer;
        (*state).index_format = format_id;
        (*state).index_offset = offset;
        wined3d_device_context_emit_set_index_buffer(context, buffer, format_id, offset);
        if !prev_buffer.is_null() {
            wined3d_buffer_decref(prev_buffer);
        }
    }
    wined3d_device_context_unlock(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_set_vertex_declaration(
    context: *mut wined3d_device_context,
    declaration: *mut wined3d_vertex_declaration,
) {
    let state: *mut wined3d_state = (*context).state;
    let prev: *mut wined3d_vertex_declaration;

    wined3d_device_context_lock(context);
    prev = (*state).vertex_declaration;
    'out: {
        if declaration == prev {
            break 'out;
        }

        if !declaration.is_null() {
            wined3d_vertex_declaration_incref(declaration);
        }
        (*state).vertex_declaration = declaration;
        wined3d_device_context_emit_set_vertex_declaration(context, declaration);
        if !prev.is_null() {
            wined3d_vertex_declaration_decref(prev);
        }
    }
    wined3d_device_context_unlock(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_set_stream_outputs(
    mut context: *mut wined3d_device_context,
    mut outputs: *const wined3d_stream_output,
) {
    let mut state: *mut wined3d_state = (*context).state;
    let mut i: ::core::ffi::c_uint = 0;
    WINE_TRACE!("context %p, outputs %p.\n", context, outputs);
    wined3d_device_context_lock(context);
    wined3d_device_context_emit_set_stream_outputs(context, outputs);
    i = 0;
    while i < WINED3D_MAX_STREAM_OUTPUT_BUFFERS {
        let mut prev_buffer: *mut wined3d_buffer = (*state).stream_output[i as usize].buffer;
        let mut buffer: *mut wined3d_buffer = (*outputs.offset(i as isize)).buffer;
        if !buffer.is_null() {
            wined3d_buffer_incref(buffer);
        }
        (*state).stream_output[i as usize] = *outputs.offset(i as isize);
        if !prev_buffer.is_null() {
            wined3d_buffer_decref(prev_buffer);
        }
        i = i.wrapping_add(1);
    }
    wined3d_device_context_unlock(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_draw(
    context: *mut wined3d_device_context,
    start_vertex: ::core::ffi::c_uint,
    vertex_count: ::core::ffi::c_uint,
    start_instance: ::core::ffi::c_uint,
    instance_count: ::core::ffi::c_uint,
) {
    let state: *mut wined3d_state = (*context).state;

    wined3d_device_context_lock(context);
    wined3d_device_context_emit_draw(
        context,
        (*state).primitive_type,
        (*state).patch_vertex_count,
        0,
        start_vertex,
        vertex_count,
        start_instance,
        instance_count,
        false,
    );
    wined3d_device_context_unlock(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_draw_indexed(context: *mut wined3d_device_context, base_vertex_index: i32, start_index: u32, index_count: u32, start_instance: u32, instance_count: u32) {
    let mut state: *mut wined3d_state = (*context).state;

    wined3d_device_context_lock(context);
    wined3d_device_context_emit_draw(context, (*state).primitive_type, (*state).patch_vertex_count,
            base_vertex_index, start_index, index_count, start_instance, instance_count, true);
    wined3d_device_context_unlock(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_get_constant_buffer(
    context: *const wined3d_device_context,
    shader_type: wined3d_shader_type,
    idx: ::core::ffi::c_uint,
    state: *mut wined3d_constant_buffer_state,
) {
    if idx >= MAX_CONSTANT_BUFFERS as ::core::ffi::c_uint {
        WARN!("Invalid constant buffer index %u.\n", idx);
        return;
    }

    *state = (*(*context).state).cb[shader_type as usize][idx as usize];
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_get_shader_resource_view(
    context: *const wined3d_device_context,
    shader_type: wined3d_shader_type,
    idx: ::core::ffi::c_uint,
) -> *mut wined3d_shader_resource_view {
    if idx >= (MAX_SHADER_RESOURCE_VIEWS) as ::core::ffi::c_uint {
        WARN!("Invalid view index %u.\n", idx);
        return core::ptr::null_mut();
    }

    return (*(*context).state).shader_resource_view[shader_type as usize][idx as usize];
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_get_sampler(context: *const wined3d_device_context, shader_type: wined3d_shader_type, idx: ::core::ffi::c_uint) -> *mut wined3d_sampler {
    if idx >= MAX_SAMPLER_OBJECTS as ::core::ffi::c_uint {
        WARN!("Invalid sampler index %u.\n", idx);
        return core::ptr::null_mut();
    }

    return (*(*context).state).sampler[shader_type as usize][idx as usize];
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_get_unordered_access_view(
    context: *const wined3d_device_context,
    pipeline: wined3d_pipeline,
    idx: u32,
) -> *mut wined3d_unordered_access_view {
    if idx >= MAX_UNORDERED_ACCESS_VIEWS {
        WARN!("Invalid UAV index %u.\n", idx);
        return core::ptr::null_mut();
    }

    return (*(*context).state).unordered_access_view[pipeline as usize][idx as usize];
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_set_max_frame_latency(
    device: *mut wined3d_device,
    mut latency: ::core::ffi::c_uint,
) {
    let mut i: ::core::ffi::c_uint;

    if latency == 0 {
        latency = 3;
    }

    (*device).max_frame_latency = latency;
    i = 0;
    while i < (*device).swapchain_count {
        swapchain_set_max_frame_latency(*(*device).swapchains.offset(i as isize), device);
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_device_get_max_frame_latency(
    device: *const wined3d_device,
) -> u32 {
    (*device).max_frame_latency
}

unsafe extern "C" fn wined3d_get_flexible_vertex_size(mut fvf: uint32_t) -> ::core::ffi::c_uint {
    let mut texcoord_count: ::core::ffi::c_uint =
        ((fvf & WINED3DFVF_TEXCOUNT_MASK) >> WINED3DFVF_TEXCOUNT_SHIFT) as ::core::ffi::c_uint;
    let mut i: ::core::ffi::c_uint = 0;
    let mut size: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    if fvf & WINED3DFVF_NORMAL as uint32_t != 0 {
        size = size.wrapping_add(
            (3 as ::core::ffi::c_uint)
                .wrapping_mul(core::mem::size_of::<f32>() as ::core::ffi::c_uint),
        );
    }
    if fvf & WINED3DFVF_DIFFUSE as uint32_t != 0 {
        size = size.wrapping_add(core::mem::size_of::<DWORD>() as ::core::ffi::c_uint);
    }
    if fvf & WINED3DFVF_SPECULAR as uint32_t != 0 {
        size = size.wrapping_add(core::mem::size_of::<DWORD>() as ::core::ffi::c_uint);
    }
    if fvf & WINED3DFVF_PSIZE as uint32_t != 0 {
        size = size.wrapping_add(core::mem::size_of::<DWORD>() as ::core::ffi::c_uint);
    }
    match fvf & WINED3DFVF_POSITION_MASK as uint32_t {
        WINED3DFVF_XYZ => {
            size = size.wrapping_add(
                (3 as ::core::ffi::c_uint)
                    .wrapping_mul(core::mem::size_of::<f32>() as ::core::ffi::c_uint),
            );
        }
        WINED3DFVF_XYZRHW => {
            size = size.wrapping_add(
                (4 as ::core::ffi::c_uint)
                    .wrapping_mul(core::mem::size_of::<f32>() as ::core::ffi::c_uint),
            );
        }
        WINED3DFVF_XYZB1 => {
            size = size.wrapping_add(
                (4 as ::core::ffi::c_uint)
                    .wrapping_mul(core::mem::size_of::<f32>() as ::core::ffi::c_uint),
            );
        }
        WINED3DFVF_XYZB2 => {
            size = size.wrapping_add(
                (5 as ::core::ffi::c_uint)
                    .wrapping_mul(core::mem::size_of::<f32>() as ::core::ffi::c_uint),
            );
        }
        WINED3DFVF_XYZB3 => {
            size = size.wrapping_add(
                (6 as ::core::ffi::c_uint)
                    .wrapping_mul(core::mem::size_of::<f32>() as ::core::ffi::c_uint),
            );
        }
        WINED3DFVF_XYZB4 => {
            size = size.wrapping_add(
                (7 as ::core::ffi::c_uint)
                    .wrapping_mul(core::mem::size_of::<f32>() as ::core::ffi::c_uint),
            );
        }
        WINED3DFVF_XYZB5 => {
            size = size.wrapping_add(
                (8 as ::core::ffi::c_uint)
                    .wrapping_mul(core::mem::size_of::<f32>() as ::core::ffi::c_uint),
            );
        }
        WINED3DFVF_XYZW => {
            size = size.wrapping_add(
                (4 as ::core::ffi::c_uint)
                    .wrapping_mul(core::mem::size_of::<f32>() as ::core::ffi::c_uint),
            );
        }
        _ => {
            FIXME!(
                "Unexpected position mask %#x.\n",
                fvf & WINED3DFVF_POSITION_MASK as uint32_t
            );
        }
    }
    i = 0 as ::core::ffi::c_uint;
    while i < texcoord_count {
        size = size.wrapping_add(
            (GET_TEXCOORD_SIZE_FROM_FVF!(fvf, i) as ::core::ffi::c_uint)
                .wrapping_mul(core::mem::size_of::<f32>() as ::core::ffi::c_uint),
        );
        i += 1;
    }

    return size;
}

unsafe extern "C" fn wined3d_format_get_colour(
    format: *const wined3d_format,
    data: *const ::core::ffi::c_void,
    colour: *mut wined3d_color,
) {
    let mut output: *mut ::core::ffi::c_float = core::ptr::addr_of_mut!((*colour).r);
    let mut u32_data: *const uint32_t = core::ptr::null::<uint32_t>();
    let mut u16_data: *const uint16_t = core::ptr::null::<uint16_t>();
    let mut f32_data: *const ::core::ffi::c_float = core::ptr::null::<::core::ffi::c_float>();
    let mut i: ::core::ffi::c_uint = 0;
    static mut default_colour: wined3d_color = wined3d_color {
        r: 0.0f32,
        g: 0.0f32,
        b: 0.0f32,
        a: 1.0f32,
    };
    static mut warned: ::core::ffi::c_uint = 0;
    match (*format).id as ::core::ffi::c_uint {
        WINED3DFMT_B8G8R8A8_UNORM => {
            u32_data = data as *const uint32_t;
            wined3d_color_from_d3dcolor(colour, *u32_data);
        }
        WINED3DFMT_R8G8B8A8_UNORM => {
            u32_data = data as *const uint32_t;
            (*colour).r = (*u32_data & 0xff) as ::core::ffi::c_float / 255.0f32;
            (*colour).g = ((*u32_data >> 8) & 0xff)
                as ::core::ffi::c_float
                / 255.0f32;
            (*colour).b = ((*u32_data >> 16) & 0xff)
                as ::core::ffi::c_float
                / 255.0f32;
            (*colour).a = ((*u32_data >> 24) & 0xff)
                as ::core::ffi::c_float
                / 255.0f32;
        }
        WINED3DFMT_R16G16_UNORM | WINED3DFMT_R16G16B16A16_UNORM => {
            u16_data = data as *const uint16_t;
            *colour = default_colour;
            i = 0 as ::core::ffi::c_uint;
            while i < (*format).component_count {
                *output.offset(i as isize) = *u16_data.offset(i as isize) as ::core::ffi::c_int
                    as ::core::ffi::c_float
                    / 65535.0f32;
                i = i.wrapping_add(1);
            }
        }
        WINED3DFMT_R32_FLOAT | WINED3DFMT_R32G32_FLOAT | WINED3DFMT_R32G32B32_FLOAT | WINED3DFMT_R32G32B32A32_FLOAT => {
            f32_data = data as *const ::core::ffi::c_float;
            *colour = default_colour;
            i = 0 as ::core::ffi::c_uint;
            while i < (*format).component_count {
                *output.offset(i as isize) = *f32_data.offset(i as isize);
                i = i.wrapping_add(1);
            }
        }
        _ => {
            *colour = default_colour;
            let fresh = warned;
            warned = warned.wrapping_add(1);
            if fresh == 0 {
                FIXME!(
                    "Unhandled colour format conversion, format %s.\n",
                    debug_d3dformat((*format).id)
                );
            }
        }
    }
}

unsafe extern "C" fn wined3d_colour_from_mcs(colour: *mut wined3d_color, mcs: wined3d_material_color_source, material_colour: *const wined3d_color, index: ::core::ffi::c_uint, stream_info: *const wined3d_stream_info) {
    let mut element: *const wined3d_stream_info_element = core::ptr::null();
    match mcs as ::core::ffi::c_uint {
        WINED3D_MCS_MATERIAL => {
            *colour = *material_colour;
            return;
        }
        WINED3D_MCS_COLOR1 => {
            if !((*stream_info).use_map & (1 << WINED3D_FFP_DIFFUSE) != 0) {
                (*colour).r = 1.0f32;
                (*colour).g = 1.0f32;
                (*colour).b = 1.0f32;
                (*colour).a = 1.0f32;
                return;
            }
            element = (&raw const (*stream_info).elements as *const wined3d_stream_info_element).offset(WINED3D_FFP_DIFFUSE as isize);
        }
        WINED3D_MCS_COLOR2 => {
            if !((*stream_info).use_map & (1 << WINED3D_FFP_SPECULAR) != 0) {
                (*colour).r = 0.0f32;
                (*colour).g = 0.0f32;
                (*colour).b = 0.0f32;
                (*colour).a = 0.0f32;
                return;
            }
            element = (&raw const (*stream_info).elements as *const wined3d_stream_info_element).offset(WINED3D_FFP_SPECULAR as isize);
        }
        _ => {
            (*colour).r = 0.0f32;
            (*colour).g = 0.0f32;
            (*colour).b = 0.0f32;
            (*colour).a = 0.0f32;
            ERR!("Invalid material colour source %#x.\n", mcs as ::core::ffi::c_uint);
            return;
        }
    }
    wined3d_format_get_colour(
        (*element).format as *const wined3d_format,
        (*element).data.addr.offset(index.wrapping_mul((*element).stride) as isize) as *mut BYTE as *const ::core::ffi::c_void,
        colour,
    );
}

unsafe extern "C" fn wined3d_clamp(mut value: f32, mut min_value: f32, mut max_value: f32) -> f32 {
    return if value < min_value {
        min_value
    } else if value > max_value {
        max_value
    } else {
        value
    };
}

unsafe extern "C" fn wined3d_vec3_dot(v0: *const wined3d_vec3, v1: *const wined3d_vec3) -> f32 {
    return (*v0).x * (*v1).x + (*v0).y * (*v1).y + (*v0).z * (*v1).z;
}

unsafe extern "C" fn wined3d_vec3_subtract(v0: *mut wined3d_vec3, v1: *const wined3d_vec3) {
    (*v0).x -= (*v1).x;
    (*v0).y -= (*v1).y;
    (*v0).z -= (*v1).z;
}

unsafe extern "C" fn wined3d_vec3_scale(v: *mut wined3d_vec3, s: f32) {
    (*v).x *= s;
    (*v).y *= s;
    (*v).z *= s;
}

unsafe extern "C" fn wined3d_vec3_normalise(v: *mut wined3d_vec3) {
    let mut rnorm: f32 = 1.0f32 / sqrtf(wined3d_vec3_dot(v, v));

    if isfinite!(rnorm) != 0 {
        wined3d_vec3_scale(v, rnorm);
    }
}

unsafe extern "C" fn wined3d_vec3_transform(
    dst: *mut wined3d_vec3,
    v: *const wined3d_vec3,
    m: *const wined3d_matrix,
) {
    let mut tmp: wined3d_vec3 = wined3d_vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    tmp.x = (*v).x * (*m)._11 + (*v).y * (*m)._21 + (*v).z * (*m)._31;
    tmp.y = (*v).x * (*m)._12 + (*v).y * (*m)._22 + (*v).z * (*m)._32;
    tmp.z = (*v).x * (*m)._13 + (*v).y * (*m)._23 + (*v).z * (*m)._33;

    *dst = tmp;
}

unsafe extern "C" fn wined3d_color_clamp(dst: *mut wined3d_color, src: *const wined3d_color, min_value: f32, max_value: f32) {
    (*dst).r = wined3d_clamp((*src).r, min_value, max_value);
    (*dst).g = wined3d_clamp((*src).g, min_value, max_value);
    (*dst).b = wined3d_clamp((*src).b, min_value, max_value);
    (*dst).a = wined3d_clamp((*src).a, min_value, max_value);
}

unsafe extern "C" fn wined3d_color_rgb_mul_add(dst: *mut wined3d_color, src: *const wined3d_color, c: f32) {
    (*dst).r += (*src).r * c;
    (*dst).g += (*src).g * c;
    (*dst).b += (*src).b * c;
}

unsafe extern "C" fn init_transformed_lights(
    mut ls: *mut lights_settings,
    mut state: *const wined3d_stateblock_state,
    mut legacy_lighting: BOOL,
    mut compute_lighting: BOOL,
) {
    let mut lights: [*const wined3d_light_info; (WINED3D_MAX_SOFTWARE_ACTIVE_LIGHTS as _)] =
        [::core::ptr::null::<wined3d_light_info>(); (WINED3D_MAX_SOFTWARE_ACTIVE_LIGHTS as _)];
    let mut light_info: *const wined3d_light_info = ::core::ptr::null::<wined3d_light_info>();
    let mut light_iter: *mut wined3d_light_info = ::core::ptr::null_mut::<wined3d_light_info>();
    let mut light: *mut light_transformed = ::core::ptr::null_mut::<light_transformed>();
    let mut vec4: wined3d_vec4 = wined3d_vec4 {
        x: 0.,
        y: 0.,
        z: 0.,
        w: 0.,
    };
    let mut light_count: ::core::ffi::c_uint = 0;
    let mut i: ::core::ffi::c_uint = 0;
    let mut index: ::core::ffi::c_uint = 0;
    memset(
        ls as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<lights_settings>() as size_t,
    );
    (*ls).set_lighting((compute_lighting != 0) as ::core::ffi::c_int as uint32_t as uint32_t);
    (*ls).set_fog_mode(
        (*state).rs[WINED3D_RS_FOGVERTEXMODE as usize] as uint32_t as uint32_t,
    );
    (*ls).set_fog_coord_mode(
        (if (*state).rs[WINED3D_RS_RANGEFOGENABLE as usize] != 0 {
            WINED3D_FFP_VS_FOG_RANGE as ::core::ffi::c_int
        } else {
            WINED3D_FFP_VS_FOG_DEPTH as ::core::ffi::c_int
        }) as uint32_t as uint32_t,
    );
    (*ls).fog_start = int_to_float((*state).rs[WINED3D_RS_FOGSTART as usize]);
    (*ls).fog_end = int_to_float((*state).rs[WINED3D_RS_FOGEND as usize]);
    (*ls).fog_density = int_to_float((*state).rs[WINED3D_RS_FOGDENSITY as usize]);
    if (*ls).fog_mode() as ::core::ffi::c_int == WINED3D_FOG_NONE as ::core::ffi::c_int
        && compute_lighting == 0
    {
        return;
    }
    multiply_matrix(
        &raw mut (*ls).modelview_matrix,
        (&raw const (*state).transforms as *const wined3d_matrix)
            .offset(WINED3D_TS_VIEW as ::core::ffi::c_int as isize)
            as *const wined3d_matrix,
        (&raw const (*state).transforms as *const wined3d_matrix).offset(
            (0 as ::core::ffi::c_int + 256 as ::core::ffi::c_int)
                as wined3d_transform_state as isize,
        ) as *const wined3d_matrix,
    );
    if compute_lighting == 0 {
        return;
    }
    compute_normal_matrix(
        &raw mut (*ls).normal_matrix,
        legacy_lighting,
        &raw mut (*ls).modelview_matrix,
    );
    wined3d_color_from_d3dcolor(
        &raw mut (*ls).ambient_light,
        (*state).rs[WINED3D_RS_AMBIENT as usize],
    );
    (*ls).set_legacy_lighting((legacy_lighting != 0) as ::core::ffi::c_int as uint32_t as uint32_t);
    (*ls).set_normalise(
        ((*state).rs[WINED3D_RS_NORMALIZENORMALS as usize] != 0) as ::core::ffi::c_int as uint32_t
            as uint32_t,
    );
    (*ls).set_localviewer(
        ((*state).rs[WINED3D_RS_LOCALVIEWER as usize] != 0) as ::core::ffi::c_int as uint32_t
            as uint32_t,
    );
    index = 0 as ::core::ffi::c_uint;
    let mut current_block_28: u64;
    light_iter = (rb_head((*(*state).light_state).lights_tree.root) as *mut ::core::ffi::c_char)
        .offset(-(232 as ::core::ffi::c_ulonglong as isize))
        as *mut wined3d_light_info;
    while light_iter
        != ::core::ptr::null_mut::<::core::ffi::c_char>()
            .offset(-(232 as ::core::ffi::c_ulonglong as isize))
            as *mut wined3d_light_info
    {
        if !((*light_iter).enabled == 0) {
            match (*light_iter).OriginalParms.type_ as ::core::ffi::c_uint {
                WINED3D_LIGHT_DIRECTIONAL => {
                    (*ls).set_directional_light_count(
                        (*ls).directional_light_count() + 1 as uint32_t,
                    );
                    current_block_28 = 4775909272756257391;
                }
                WINED3D_LIGHT_POINT => {
                    (*ls).set_point_light_count((*ls).point_light_count() + 1 as uint32_t);
                    current_block_28 = 4775909272756257391;
                }
                WINED3D_LIGHT_SPOT => {
                    (*ls).set_spot_light_count((*ls).spot_light_count() + 1 as uint32_t);
                    current_block_28 = 4775909272756257391;
                }
                WINED3D_LIGHT_PARALLELPOINT => {
                    (*ls).set_parallel_point_light_count(
                        (*ls).parallel_point_light_count() + 1 as uint32_t,
                    );
                    current_block_28 = 4775909272756257391;
                }
                _ => {
                    FIXME!(
                        "Unhandled light type %#x.\n",
                        (*light_iter).OriginalParms.type_ as ::core::ffi::c_uint
                    );
                    current_block_28 = 10048703153582371463;
                }
            }
            match current_block_28 {
                10048703153582371463 => {}
                _ => {
                    let fresh16 = index;
                    index = index.wrapping_add(1);
                    lights[fresh16 as usize] = light_iter;
                    if index == WINED3D_MAX_SOFTWARE_ACTIVE_LIGHTS as ::core::ffi::c_uint {
                        break;
                    }
                }
            }
        }
        light_iter = (rb_next(&raw mut (*light_iter).entry) as *mut ::core::ffi::c_char)
            .offset(-(232 as ::core::ffi::c_ulonglong as isize))
            as *mut wined3d_light_info;
    }
    light_count = index;
    i = 0 as ::core::ffi::c_uint;
    index = 0 as ::core::ffi::c_uint;
    while i < light_count {
        light_info = lights[i as usize];
        if !((*light_info).OriginalParms.type_ as ::core::ffi::c_uint
            != WINED3D_LIGHT_DIRECTIONAL as ::core::ffi::c_uint)
        {
            light = (&raw mut (*ls).lights as *mut light_transformed).offset(index as isize)
                as *mut light_transformed;
            wined3d_vec4_transform(
                &raw mut vec4,
                &raw const (*light_info).constants.direction,
                (&raw const (*state).transforms as *const wined3d_matrix)
                    .offset(WINED3D_TS_VIEW as ::core::ffi::c_int as isize)
                    as *const wined3d_matrix,
            );
            (*light).direction = *(&raw mut vec4 as *mut wined3d_vec3);
            wined3d_vec3_normalise(&raw mut (*light).direction);
            (*light).diffuse = (*light_info).OriginalParms.diffuse;
            (*light).ambient = (*light_info).OriginalParms.ambient;
            (*light).specular = (*light_info).OriginalParms.specular;
            index = index.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    i = 0 as ::core::ffi::c_uint;
    while i < light_count {
        light_info = lights[i as usize];
        if !((*light_info).OriginalParms.type_ as ::core::ffi::c_uint
            != WINED3D_LIGHT_POINT as ::core::ffi::c_uint)
        {
            light = (&raw mut (*ls).lights as *mut light_transformed).offset(index as isize)
                as *mut light_transformed;
            wined3d_vec4_transform(
                &raw mut (*light).position,
                &raw const (*light_info).constants.position,
                (&raw const (*state).transforms as *const wined3d_matrix)
                    .offset(WINED3D_TS_VIEW as ::core::ffi::c_int as isize)
                    as *const wined3d_matrix,
            );
            (*light).range = (*light_info).OriginalParms.range;
            (*light).c_att = (*light_info).OriginalParms.attenuation0;
            (*light).l_att = (*light_info).OriginalParms.attenuation1;
            (*light).q_att = (*light_info).OriginalParms.attenuation2;
            (*light).diffuse = (*light_info).OriginalParms.diffuse;
            (*light).ambient = (*light_info).OriginalParms.ambient;
            (*light).specular = (*light_info).OriginalParms.specular;
            index = index.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    i = 0 as ::core::ffi::c_uint;
    while i < light_count {
        light_info = lights[i as usize];
        if !((*light_info).OriginalParms.type_ as ::core::ffi::c_uint
            != WINED3D_LIGHT_SPOT as ::core::ffi::c_uint)
        {
            light = (&raw mut (*ls).lights as *mut light_transformed).offset(index as isize)
                as *mut light_transformed;
            wined3d_vec4_transform(
                &raw mut (*light).position,
                &raw const (*light_info).constants.position,
                (&raw const (*state).transforms as *const wined3d_matrix)
                    .offset(WINED3D_TS_VIEW as ::core::ffi::c_int as isize)
                    as *const wined3d_matrix,
            );
            wined3d_vec4_transform(
                &raw mut vec4,
                &raw const (*light_info).constants.direction,
                (&raw const (*state).transforms as *const wined3d_matrix)
                    .offset(WINED3D_TS_VIEW as ::core::ffi::c_int as isize)
                    as *const wined3d_matrix,
            );
            (*light).direction = *(&raw mut vec4 as *mut wined3d_vec3);
            wined3d_vec3_normalise(&raw mut (*light).direction);
            (*light).range = (*light_info).OriginalParms.range;
            (*light).falloff = (*light_info).OriginalParms.falloff;
            (*light).c_att = (*light_info).OriginalParms.attenuation0;
            (*light).l_att = (*light_info).OriginalParms.attenuation1;
            (*light).q_att = (*light_info).OriginalParms.attenuation2;
            (*light).cos_htheta = cosf((*light_info).OriginalParms.theta / 2.0f32);
            (*light).cos_hphi = cosf((*light_info).OriginalParms.phi / 2.0f32);
            (*light).diffuse = (*light_info).OriginalParms.diffuse;
            (*light).ambient = (*light_info).OriginalParms.ambient;
            (*light).specular = (*light_info).OriginalParms.specular;
            index = index.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    i = 0 as ::core::ffi::c_uint;
    while i < light_count {
        light_info = lights[i as usize];
        if !((*light_info).OriginalParms.type_ as ::core::ffi::c_uint
            != WINED3D_LIGHT_PARALLELPOINT as ::core::ffi::c_uint)
        {
            light = (&raw mut (*ls).lights as *mut light_transformed).offset(index as isize)
                as *mut light_transformed;
            wined3d_vec4_transform(
                &raw mut vec4,
                &raw const (*light_info).constants.position,
                (&raw const (*state).transforms as *const wined3d_matrix)
                    .offset(WINED3D_TS_VIEW as ::core::ffi::c_int as isize)
                    as *const wined3d_matrix,
            );
            *(&raw mut (*light).position as *mut wined3d_vec3) =
                *(&raw mut vec4 as *mut wined3d_vec3);
            wined3d_vec3_normalise(&raw mut (*light).position as *mut wined3d_vec3);
            (*light).diffuse = (*light_info).OriginalParms.diffuse;
            (*light).ambient = (*light_info).OriginalParms.ambient;
            (*light).specular = (*light_info).OriginalParms.specular;
            index = index.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn update_light_diffuse_specular(
    diffuse: *mut wined3d_color,
    specular: *mut wined3d_color,
    dir: *const wined3d_vec3,
    att: f32,
    material_shininess: f32,
    normal_transformed: *const wined3d_vec3,
    position_transformed_normalised: *const wined3d_vec3,
    light: *const light_transformed,
    ls: *const lights_settings,
) {
    let mut vec3: wined3d_vec3 = wined3d_vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let mut t: f32 = 0.0;
    let mut c: f32 = 0.0;

    c = wined3d_clamp(wined3d_vec3_dot(dir, normal_transformed), 0.0f32, 1.0f32);
    wined3d_color_rgb_mul_add(diffuse, core::ptr::addr_of!((*light).diffuse), c * att);

    vec3 = *dir;
    if (*ls).localviewer() != 0 {
        wined3d_vec3_subtract(core::ptr::addr_of_mut!(vec3), position_transformed_normalised);
    } else {
        vec3.z -= 1.0f32;
    }
    wined3d_vec3_normalise(core::ptr::addr_of_mut!(vec3));
    t = wined3d_vec3_dot(normal_transformed, core::ptr::addr_of_mut!(vec3));
    if t > 0.0f32
        && ((*ls).legacy_lighting() == 0 || material_shininess > 0.0f32)
        && wined3d_vec3_dot(dir, normal_transformed) > 0.0f32
    {
        wined3d_color_rgb_mul_add(
            specular,
            core::ptr::addr_of!((*light).specular),
            att * powf(t, material_shininess),
        );
    }
}

unsafe extern "C" fn light_set_vertex_data(
    ls: *mut lights_settings,
    position: *const wined3d_vec4,
) {
    if ((*ls).fog_mode() as i32) as u32 == WINED3D_FOG_NONE && (*ls).lighting() == 0 {
        return;
    }

    wined3d_vec4_transform(
        core::ptr::addr_of_mut!((*ls).position_transformed),
        position,
        core::ptr::addr_of_mut!((*ls).modelview_matrix),
    );
    wined3d_vec3_scale(
        core::ptr::addr_of_mut!((*ls).position_transformed) as *mut wined3d_vec3,
        1.0f32 / (*ls).position_transformed.w,
    );
}

unsafe extern "C" fn compute_light(
    ambient: *mut wined3d_color,
    diffuse: *mut wined3d_color,
    specular: *mut wined3d_color,
    ls: *mut lights_settings,
    normal: *const wined3d_vec3,
    material_shininess: f32,
) {
    let mut position_transformed_normalised: wined3d_vec3 = wined3d_vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut normal_transformed: wined3d_vec3 = wined3d_vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut light: *const light_transformed = core::ptr::null::<light_transformed>();
    let mut dir: wined3d_vec3 = wined3d_vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut dst: wined3d_vec3 = wined3d_vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut i: u32 = 0;
    let mut index: u32 = 0;
    let mut att: f32 = 0.0;

    position_transformed_normalised = *(&raw mut (*ls).position_transformed as *const wined3d_vec3);
    wined3d_vec3_normalise(&raw mut position_transformed_normalised);

    if !normal.is_null() {
        wined3d_vec3_transform(&raw mut normal_transformed, normal, &raw mut (*ls).normal_matrix);
        if (*ls).normalise() != 0 {
            wined3d_vec3_normalise(&raw mut normal_transformed);
        }
    }

    (*diffuse).a = 0.0f32;
    (*diffuse).b = (*diffuse).a;
    (*diffuse).g = (*diffuse).b;
    (*diffuse).r = (*diffuse).g;
    *specular = *diffuse;
    *ambient = (*ls).ambient_light;

    index = 0;
    i = 0;
    while i < (*ls).directional_light_count() as u32 {
        light = (&raw mut (*ls).lights as *mut light_transformed).offset(index as isize) as *mut light_transformed;

        wined3d_color_rgb_mul_add(ambient, &raw const (*light).ambient, 1.0f32);
        if !normal.is_null() {
            update_light_diffuse_specular(
                diffuse,
                specular,
                &raw const (*light).direction,
                1.0f32,
                material_shininess,
                &raw mut normal_transformed,
                &raw mut position_transformed_normalised,
                light,
                ls,
            );
        }
        i = i.wrapping_add(1);
        index = index.wrapping_add(1);
    }

    i = 0;
    while i < (*ls).point_light_count() as u32 {
        'body: {
            light = (&raw mut (*ls).lights as *mut light_transformed).offset(index as isize) as *mut light_transformed;
            dir.x = (*light).position.x - (*ls).position_transformed.x;
            dir.y = (*light).position.y - (*ls).position_transformed.y;
            dir.z = (*light).position.z - (*ls).position_transformed.z;

            dst.z = wined3d_vec3_dot(&raw mut dir, &raw mut dir);
            dst.y = sqrtf(dst.z);
            dst.x = 1.0f32;
            if (*ls).legacy_lighting() != 0 {
                dst.y = ((*light).range - dst.y) / (*light).range;
                if !(dst.y > 0.0f32) {
                    break 'body;
                }
                dst.z = dst.y * dst.y;
            } else {
                if !(dst.y <= (*light).range) {
                    break 'body;
                }
            }
            att = dst.x * (*light).c_att + dst.y * (*light).l_att + dst.z * (*light).q_att;
            if (*ls).legacy_lighting() == 0 {
                att = 1.0f32 / att;
            }

            wined3d_color_rgb_mul_add(ambient, &raw const (*light).ambient, att);
            if !normal.is_null() {
                wined3d_vec3_normalise(&raw mut dir);
                update_light_diffuse_specular(
                    diffuse,
                    specular,
                    &raw mut dir,
                    att,
                    material_shininess,
                    &raw mut normal_transformed,
                    &raw mut position_transformed_normalised,
                    light,
                    ls,
                );
            }
        }
        i = i.wrapping_add(1);
        index = index.wrapping_add(1);
    }

    i = 0;
    while i < (*ls).spot_light_count() as u32 {
        'body: {
            let mut t: f32 = 0.0;

            light = (&raw mut (*ls).lights as *mut light_transformed).offset(index as isize) as *mut light_transformed;

            dir.x = (*light).position.x - (*ls).position_transformed.x;
            dir.y = (*light).position.y - (*ls).position_transformed.y;
            dir.z = (*light).position.z - (*ls).position_transformed.z;

            dst.z = wined3d_vec3_dot(&raw mut dir, &raw mut dir);
            dst.y = sqrtf(dst.z);
            dst.x = 1.0f32;

            if (*ls).legacy_lighting() != 0 {
                dst.y = ((*light).range - dst.y) / (*light).range;
                if !(dst.y > 0.0f32) {
                    break 'body;
                }
                dst.z = dst.y * dst.y;
            } else {
                if !(dst.y <= (*light).range) {
                    break 'body;
                }
            }
            wined3d_vec3_normalise(&raw mut dir);
            t = -wined3d_vec3_dot(&raw mut dir, &raw const (*light).direction);
            if t > (*light).cos_htheta {
                att = 1.0f32;
            } else if t <= (*light).cos_hphi {
                att = 0.0f32;
            } else {
                att = powf(
                    (t - (*light).cos_hphi) / ((*light).cos_htheta - (*light).cos_hphi),
                    (*light).falloff,
                );
            }

            t = dst.x * (*light).c_att + dst.y * (*light).l_att + dst.z * (*light).q_att;
            if (*ls).legacy_lighting() != 0 {
                att *= t;
            } else {
                att /= t;
            }

            wined3d_color_rgb_mul_add(ambient, &raw const (*light).ambient, att);

            if !normal.is_null() {
                update_light_diffuse_specular(
                    diffuse,
                    specular,
                    &raw mut dir,
                    att,
                    material_shininess,
                    &raw mut normal_transformed,
                    &raw mut position_transformed_normalised,
                    light,
                    ls,
                );
            }
        }
        i = i.wrapping_add(1);
        index = index.wrapping_add(1);
    }

    i = 0;
    while i < (*ls).parallel_point_light_count() as u32 {
        light = (&raw mut (*ls).lights as *mut light_transformed).offset(index as isize) as *mut light_transformed;

        wined3d_color_rgb_mul_add(ambient, &raw const (*light).ambient, 1.0f32);
        if !normal.is_null() {
            update_light_diffuse_specular(
                diffuse,
                specular,
                &raw const (*light).position as *const wined3d_vec3,
                1.0f32,
                material_shininess,
                &raw mut normal_transformed,
                &raw mut position_transformed_normalised,
                light,
                ls,
            );
        }
        i = i.wrapping_add(1);
        index = index.wrapping_add(1);
    }
}

unsafe extern "C" fn wined3d_calculate_fog_factor(fog_coord: f32, ls: *const lights_settings) -> f32 {
    match ((*ls).fog_mode() as i32) as u32 {
        WINED3D_FOG_NONE => {
            return fog_coord;
        }
        WINED3D_FOG_LINEAR => {
            return ((*ls).fog_end - fog_coord) / ((*ls).fog_end - (*ls).fog_start);
        }
        WINED3D_FOG_EXP => {
            return expf(-fog_coord * (*ls).fog_density);
        }
        WINED3D_FOG_EXP2 => {
            return expf(-fog_coord * fog_coord * (*ls).fog_density * (*ls).fog_density);
        }
        _ => {
            ERR!("Unhandled fog mode %#x.\n", (*ls).fog_mode() as i32);
            return 0.0f32;
        }
    }
}

unsafe extern "C" fn update_fog_factor(fog_factor: *mut ::core::ffi::c_float, ls: *mut lights_settings) {
    let mut fog_coord: ::core::ffi::c_float;
    if (*ls).fog_mode() as ::core::ffi::c_int == WINED3D_FOG_NONE as ::core::ffi::c_int {
        return;
    }
    if (*ls).fog_coord_mode() as ::core::ffi::c_int == WINED3D_FFP_VS_FOG_RANGE as ::core::ffi::c_int {
        fog_coord = sqrtf(wined3d_vec3_dot(
            core::ptr::addr_of_mut!((*ls).position_transformed) as *const wined3d_vec3,
            core::ptr::addr_of_mut!((*ls).position_transformed) as *const wined3d_vec3,
        ));
    } else if (*ls).fog_coord_mode() as ::core::ffi::c_int == WINED3D_FFP_VS_FOG_DEPTH as ::core::ffi::c_int {
        fog_coord = fabsf((*ls).position_transformed.z);
    } else {
        ERR!(
            "Unhandled fog coordinate mode %#x.\n",
            (*ls).fog_coord_mode() as ::core::ffi::c_int
        );
        return;
    }
    *fog_factor = wined3d_calculate_fog_factor(fog_coord, ls);
}

unsafe extern "C" fn process_vertices_strided(
    device: *const wined3d_device,
    state: *const wined3d_stateblock_state,
    dwDestIndex: DWORD,
    dwCount: DWORD,
    stream_info: *const wined3d_stream_info,
    dest: *mut wined3d_buffer,
    flags: uint32_t,
    dst_fvf: uint32_t,
) -> HRESULT {
    let mut diffuse_source: wined3d_material_color_source = core::mem::zeroed();
    let mut specular_source: wined3d_material_color_source = core::mem::zeroed();
    let mut ambient_source: wined3d_material_color_source = core::mem::zeroed();
    let mut emissive_source: wined3d_material_color_source = core::mem::zeroed();
    let device_state: *const wined3d_state = (*(*device).cs).c.state;
    let mut proj_mat: *const wined3d_matrix;
    let mut view_mat: *const wined3d_matrix;
    let mut world_mat: *const wined3d_matrix;
    let mut material_specular_state_colour: *const wined3d_color;
    let mut output_colour_format: *const wined3d_format;
    static mut black: wined3d_color = wined3d_color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    let mut map_desc: wined3d_map_desc = core::mem::zeroed();
    let mut r#box: wined3d_box = core::mem::zeroed();
    let mut vp: wined3d_viewport = core::mem::zeroed();
    let mut texture_count: ::core::ffi::c_uint;
    let mut ls: lights_settings = core::mem::zeroed();
    let mut mat: wined3d_matrix = core::mem::zeroed();
    let mut vertex_size: ::core::ffi::c_uint;
    let mut do_clip: BOOL;
    let mut lighting: BOOL;
    let mut min_z: f32 = core::mem::zeroed();
    let mut max_z: f32 = core::mem::zeroed();
    let mut i: ::core::ffi::c_uint;
    let mut dest_ptr: *mut BYTE;
    let mut hr: HRESULT;

    if (*stream_info).use_map & (1 << WINED3D_FFP_POSITION) == 0 {
        ERR!("Source has no position mask.\n");
        return WINED3DERR_INVALIDCALL;
    }

    if (*state).rs[WINED3D_RS_CLIPPING as usize] != 0 {
        static mut warned: BOOL = (FALSE as i32);
        /*
         * The clipping code is not quite correct. Some things need
         * to be checked against IDirect3DDevice3 (!), d3d8 and d3d9,
         * so disable clipping for now.
         * (The graphics in Half-Life are broken, and my processvertices
         *  test crashes with IDirect3DDevice3)
        do_clip = TRUE;
         */
        do_clip = (FALSE as i32);
        if warned == 0 {
            warned = (TRUE as i32);
        }
    } else {
        do_clip = (FALSE as i32);
    }

    vertex_size = wined3d_get_flexible_vertex_size(dst_fvf);
    r#box.left = (dwDestIndex as ::core::ffi::c_uint).wrapping_mul(vertex_size);
    r#box.right = r#box.left.wrapping_add((dwCount as ::core::ffi::c_uint).wrapping_mul(vertex_size));
    hr = wined3d_resource_map(
        core::ptr::addr_of_mut!((*dest).resource),
        0,
        core::ptr::addr_of_mut!(map_desc),
        core::ptr::addr_of_mut!(r#box),
        WINED3D_MAP_WRITE,
    );
    if hr < 0 {
        WARN!("Failed to map buffer, hr %#lx.\n", hr);
        return hr;
    }
    dest_ptr = map_desc.data as *mut BYTE;

    view_mat = (core::ptr::addr_of!((*state).transforms) as *const wined3d_matrix)
        .offset(WINED3D_TS_VIEW as isize);
    proj_mat = (core::ptr::addr_of!((*state).transforms) as *const wined3d_matrix)
        .offset(WINED3D_TS_PROJECTION as isize);
    world_mat = (core::ptr::addr_of!((*state).transforms) as *const wined3d_matrix)
        .offset(WINED3D_TS_WORLD as isize);

    /* Get the viewport */
    wined3d_device_context_get_viewports(core::ptr::addr_of!((*(*device).cs).c), core::ptr::null_mut(), core::ptr::addr_of_mut!(vp));

    multiply_matrix(core::ptr::addr_of_mut!(mat), view_mat, world_mat);
    multiply_matrix(core::ptr::addr_of_mut!(mat), proj_mat, core::ptr::addr_of!(mat));

    texture_count = ((dst_fvf & WINED3DFVF_TEXCOUNT_MASK) >> WINED3DFVF_TEXCOUNT_SHIFT) as ::core::ffi::c_uint;

    lighting = (((*state).rs[WINED3D_RS_LIGHTING as usize] != 0)
        && ((dst_fvf & (WINED3DFVF_DIFFUSE | WINED3DFVF_SPECULAR)) != 0)) as ::core::ffi::c_int as BOOL;
    wined3d_get_material_colour_source(
        core::ptr::addr_of_mut!(diffuse_source),
        core::ptr::addr_of_mut!(emissive_source),
        core::ptr::addr_of_mut!(ambient_source),
        core::ptr::addr_of_mut!(specular_source),
        device_state,
    );
    output_colour_format = wined3d_get_format((*device).adapter, WINED3DFMT_B8G8R8A8_UNORM, 0);
    material_specular_state_colour = if (*state).rs[WINED3D_RS_SPECULARENABLE as usize] != 0 {
        core::ptr::addr_of!((*state).material.specular)
    } else {
        core::ptr::addr_of!(black)
    };
    init_transformed_lights(
        core::ptr::addr_of_mut!(ls),
        state,
        (((*(*device).adapter).d3d_info.wined3d_creation_flags) & WINED3D_LEGACY_FFP_LIGHTING) as BOOL,
        lighting,
    );

    wined3d_viewport_get_z_range(core::ptr::addr_of_mut!(vp), core::ptr::addr_of_mut!(min_z), core::ptr::addr_of_mut!(max_z));

    i = 0;
    while i < dwCount {
        let position_element: *const wined3d_stream_info_element =
            (core::ptr::addr_of!((*stream_info).elements) as *const wined3d_stream_info_element)
                .offset(WINED3D_FFP_POSITION as isize);
        let p: *const f32 = (*position_element)
            .data
            .addr
            .offset(i.wrapping_mul((*position_element).stride) as isize) as *const f32;
        let mut ambient: wined3d_color = core::mem::zeroed();
        let mut diffuse: wined3d_color = core::mem::zeroed();
        let mut specular: wined3d_color = core::mem::zeroed();
        let mut position: wined3d_vec4 = core::mem::zeroed();
        let mut tex_index: ::core::ffi::c_uint;

        position.x = *p.offset(0 as isize);
        position.y = *p.offset(1 as isize);
        position.z = *p.offset(2 as isize);
        position.w = 1.0f32;

        light_set_vertex_data(core::ptr::addr_of_mut!(ls), core::ptr::addr_of_mut!(position));

        if ((dst_fvf & WINED3DFVF_POSITION_MASK) == WINED3DFVF_XYZ)
            || ((dst_fvf & WINED3DFVF_POSITION_MASK) == WINED3DFVF_XYZRHW)
        {
            /* The position first */
            let mut x: f32;
            let mut y: f32;
            let mut z: f32;
            let mut rhw: f32;

            /* Multiplication with world, view and projection matrix. */
            x = (*p.offset(0 as isize) * mat._11)
                + (*p.offset(1 as isize) * mat._21)
                + (*p.offset(2 as isize) * mat._31)
                + mat._41;
            y = (*p.offset(0 as isize) * mat._12)
                + (*p.offset(1 as isize) * mat._22)
                + (*p.offset(2 as isize) * mat._32)
                + mat._42;
            z = (*p.offset(0 as isize) * mat._13)
                + (*p.offset(1 as isize) * mat._23)
                + (*p.offset(2 as isize) * mat._33)
                + mat._43;
            rhw = (*p.offset(0 as isize) * mat._14)
                + (*p.offset(1 as isize) * mat._24)
                + (*p.offset(2 as isize) * mat._34)
                + mat._44;

            /* WARNING: The following things are taken from d3d7 and were not yet checked
             * against d3d8 or d3d9!
             */

            /* Clipping conditions: From msdn
             *
             * A vertex is clipped if it does not match the following requirements
             * -rhw < x <= rhw
             * -rhw < y <= rhw
             *    0 < z <= rhw
             *    0 < rhw ( Not in d3d7, but tested in d3d7)
             *
             * If clipping is on is determined by the D3DVOP_CLIP flag in D3D7, and
             * by the D3DRS_CLIPPING in D3D9(according to the msdn, not checked)
             *
             */

            if do_clip == 0
                || (-rhw - (eps as f32) < x
                    && -rhw - (eps as f32) < y
                    && -(eps as f32) < z
                    && x <= rhw + (eps as f32)
                    && y <= rhw + (eps as f32)
                    && z <= rhw + (eps as f32)
                    && rhw > (eps as f32))
            {
                /* "Normal" viewport transformation (not clipped)
                 * 1) The values are divided by rhw
                 * 2) The y axis is negative, so multiply it with -1
                 * 3) Screen coordinates go from -(Width/2) to +(Width/2) and
                 *    -(Height/2) to +(Height/2). The z range is MinZ to MaxZ
                 * 4) Multiply x with Width/2 and add Width/2
                 * 5) The same for the height
                 * 6) Add the viewpoint X and Y to the 2D coordinates and
                 *    The minimum Z value to z
                 * 7) rhw = 1 / rhw Reciprocal of Homogeneous W....
                 *
                 * Well, basically it's simply a linear transformation into viewport
                 * coordinates
                 */

                x /= rhw;
                y /= rhw;
                z /= rhw;

                y *= -(1 as f32);

                x *= vp.width / (2 as f32);
                y *= vp.height / (2 as f32);
                z *= max_z - min_z;

                x += vp.width / (2 as f32) + vp.x;
                y += vp.height / (2 as f32) + vp.y;
                z += min_z;

                rhw = (1 as f32) / rhw;
            } else {
                /* That vertex got clipped
                 * Contrary to OpenGL it is not dropped completely, it just
                 * undergoes a different calculation.
                 */
                x += rhw;
                y += rhw;

                x /= 2 as f32;
                y /= 2 as f32;

                /* Msdn mentions that Direct3D9 keeps a list of clipped vertices
                 * outside of the main vertex buffer memory. That needs some more
                 * investigation...
                 */
            }

            *(dest_ptr as *mut f32).offset(0 as isize) = x;
            *(dest_ptr as *mut f32).offset(1 as isize) = y;
            *(dest_ptr as *mut f32).offset(2 as isize) = z;
            *(dest_ptr as *mut f32).offset(3 as isize) = rhw; /* SIC, see ddraw test! */

            dest_ptr = dest_ptr.offset(
                ((3 as usize).wrapping_mul(core::mem::size_of::<f32>())) as isize,
            );

            if (dst_fvf & WINED3DFVF_POSITION_MASK) == WINED3DFVF_XYZRHW {
                dest_ptr = dest_ptr.offset(core::mem::size_of::<f32>() as isize);
            }
        }

        if (dst_fvf & WINED3DFVF_PSIZE) != 0 {
            dest_ptr = dest_ptr.offset(core::mem::size_of::<DWORD>() as isize);
        }

        if (dst_fvf & WINED3DFVF_NORMAL) != 0 {
            let element: *const wined3d_stream_info_element =
                (core::ptr::addr_of!((*stream_info).elements) as *const wined3d_stream_info_element)
                    .offset(WINED3D_FFP_NORMAL as isize);
            let normal: *const f32 = (*element)
                .data
                .addr
                .offset(i.wrapping_mul((*element).stride) as isize) as *const f32;
            /* AFAIK this should go into the lighting information */
            // copy_and_next(dest_ptr, normal, COMMENT_TOKEN_ECHO____ * sizeof(float));
            let copy_n: usize = (3 as usize).wrapping_mul(core::mem::size_of::<f32>());
            memcpy(dest_ptr as *mut ::core::ffi::c_void, normal as *const ::core::ffi::c_void, copy_n);
            dest_ptr = dest_ptr.offset(copy_n as isize);
        }

        if lighting != 0 {
            let element: *const wined3d_stream_info_element;
            let normal: *mut wined3d_vec3;

            if (*stream_info).use_map & (1 << WINED3D_FFP_NORMAL) != 0 {
                element = (core::ptr::addr_of!((*stream_info).elements) as *const wined3d_stream_info_element)
                    .offset(WINED3D_FFP_NORMAL as isize);
                normal = (*element)
                    .data
                    .addr
                    .offset(i.wrapping_mul((*element).stride) as isize) as *mut wined3d_vec3;
            } else {
                normal = core::ptr::null_mut();
            }
            compute_light(
                core::ptr::addr_of_mut!(ambient),
                core::ptr::addr_of_mut!(diffuse),
                core::ptr::addr_of_mut!(specular),
                core::ptr::addr_of_mut!(ls),
                normal,
                if (*state).rs[WINED3D_RS_SPECULARENABLE as usize] != 0 {
                    (*state).material.power
                } else {
                    0.0f32
                },
            );
        }

        if (dst_fvf & WINED3DFVF_DIFFUSE) != 0 {
            let mut material_diffuse: wined3d_color = core::mem::zeroed();
            let mut material_ambient: wined3d_color = core::mem::zeroed();
            let mut material_emissive: wined3d_color = core::mem::zeroed();
            let mut diffuse_colour: wined3d_color = core::mem::zeroed();

            wined3d_colour_from_mcs(
                core::ptr::addr_of_mut!(material_diffuse),
                diffuse_source,
                core::ptr::addr_of!((*state).material.diffuse),
                i,
                stream_info,
            );

            if lighting != 0 {
                wined3d_colour_from_mcs(
                    core::ptr::addr_of_mut!(material_ambient),
                    ambient_source,
                    core::ptr::addr_of!((*state).material.ambient),
                    i,
                    stream_info,
                );
                wined3d_colour_from_mcs(
                    core::ptr::addr_of_mut!(material_emissive),
                    emissive_source,
                    core::ptr::addr_of!((*state).material.emissive),
                    i,
                    stream_info,
                );

                diffuse_colour.r = ambient.r * material_ambient.r
                    + diffuse.r * material_diffuse.r
                    + material_emissive.r;
                diffuse_colour.g = ambient.g * material_ambient.g
                    + diffuse.g * material_diffuse.g
                    + material_emissive.g;
                diffuse_colour.b = ambient.b * material_ambient.b
                    + diffuse.b * material_diffuse.b
                    + material_emissive.b;
                diffuse_colour.a = material_diffuse.a;
            } else {
                diffuse_colour = material_diffuse;
            }
            wined3d_color_clamp(
                core::ptr::addr_of_mut!(diffuse_colour),
                core::ptr::addr_of_mut!(diffuse_colour),
                0.0f32,
                1.0f32,
            );
            wined3d_format_convert_from_float(
                output_colour_format,
                core::ptr::addr_of_mut!(diffuse_colour),
                dest_ptr as *mut ::core::ffi::c_void,
            );
            dest_ptr = dest_ptr.offset(core::mem::size_of::<DWORD>() as isize);
        }

        if (dst_fvf & WINED3DFVF_SPECULAR) != 0 {
            let mut material_specular: wined3d_color = core::mem::zeroed();
            let mut specular_colour: wined3d_color = core::mem::zeroed();

            wined3d_colour_from_mcs(
                core::ptr::addr_of_mut!(material_specular),
                specular_source,
                material_specular_state_colour,
                i,
                stream_info,
            );

            if lighting != 0 {
                specular_colour.r = specular.r * material_specular.r;
                specular_colour.g = specular.g * material_specular.g;
                specular_colour.b = specular.b * material_specular.b;
                specular_colour.a = if ls.legacy_lighting() as ::core::ffi::c_int != 0 {
                    0.0f32
                } else {
                    material_specular.a
                };
            } else {
                specular_colour = material_specular;
            }
            update_fog_factor(core::ptr::addr_of_mut!(specular_colour.a), core::ptr::addr_of_mut!(ls));
            wined3d_color_clamp(
                core::ptr::addr_of_mut!(specular_colour),
                core::ptr::addr_of_mut!(specular_colour),
                0.0f32,
                1.0f32,
            );
            wined3d_format_convert_from_float(
                output_colour_format,
                core::ptr::addr_of_mut!(specular_colour),
                dest_ptr as *mut ::core::ffi::c_void,
            );
            dest_ptr = dest_ptr.offset(core::mem::size_of::<DWORD>() as isize);
        }

        tex_index = 0;
        while tex_index < texture_count {
            let element: *const wined3d_stream_info_element =
                (core::ptr::addr_of!((*stream_info).elements) as *const wined3d_stream_info_element)
                    .offset((WINED3D_FFP_TEXCOORD0 + tex_index) as isize);
            let tex_coord: *const f32 = (*element)
                .data
                .addr
                .offset(i.wrapping_mul((*element).stride) as isize) as *const f32;
            if (*stream_info).use_map & (1 << (WINED3D_FFP_TEXCOORD0 + tex_index)) == 0 {
                ERR!("No source texture, but destination requests one\n");
                dest_ptr = dest_ptr.offset(
                    ((GET_TEXCOORD_SIZE_FROM_FVF!(dst_fvf, tex_index) as usize)
                        .wrapping_mul(core::mem::size_of::<f32>())) as isize,
                );
            } else {
                // copy_and_next(dest_ptr, tex_coord, GET_TEXCOORD_SIZE_FROM_FVF(dst_fvf, tex_index) * sizeof(float));
                let copy_n: usize = (GET_TEXCOORD_SIZE_FROM_FVF!(dst_fvf, tex_index) as usize)
                    .wrapping_mul(core::mem::size_of::<f32>());
                memcpy(dest_ptr as *mut ::core::ffi::c_void, tex_coord as *const ::core::ffi::c_void, copy_n);
                dest_ptr = dest_ptr.offset(copy_n as isize);
            }
            tex_index = tex_index.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }

    wined3d_resource_unmap(core::ptr::addr_of_mut!((*dest).resource), 0);

    return WINED3D_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_device_process_vertices(
    device: *mut wined3d_device,
    stateblock: *mut wined3d_stateblock,
    src_start_idx: UINT,
    dst_idx: UINT,
    vertex_count: UINT,
    dst_buffer: *mut wined3d_buffer,
    declaration: *const wined3d_vertex_declaration,
    flags: uint32_t,
    dst_fvf: uint32_t,
) -> HRESULT {
    let state: *const wined3d_stateblock_state = wined3d_stateblock_get_state(stateblock);
    let device_state: *mut wined3d_state = (*(*device).cs).c.state;
    let mut stream_info: wined3d_stream_info = core::mem::zeroed();
    let mut resource: *mut wined3d_resource = core::ptr::null_mut();
    let mut box_: wined3d_box = core::mem::zeroed();
    let mut vs: *mut wined3d_shader = core::ptr::null_mut();
    let mut i: core::ffi::c_uint = 0;
    let mut j: core::ffi::c_uint = 0;
    let mut map: uint32_t = 0;
    let mut hr: HRESULT = 0;

    TRACE!(
        "device %p, src_start_idx %u, dst_idx %u, vertex_count %u, dst_buffer %p, declaration %p, flags %#x, dst_fvf %#x.\n",
        device,
        src_start_idx,
        dst_idx,
        vertex_count,
        dst_buffer,
        declaration,
        flags,
        dst_fvf
    );

    if !declaration.is_null() {
        FIXME!("Output vertex declaration not implemented yet.\n");
    }

    wined3d_device_apply_stateblock(device, stateblock);

    vs = (*device_state).shader[WINED3D_SHADER_TYPE_VERTEX as usize];
    (*device_state).shader[WINED3D_SHADER_TYPE_VERTEX as usize] = core::ptr::null_mut();
    wined3d_stream_info_from_declaration(
        core::ptr::addr_of_mut!(stream_info),
        device_state,
        core::ptr::addr_of_mut!((*(*device).adapter).d3d_info),
    );
    (*device_state).shader[WINED3D_SHADER_TYPE_VERTEX as usize] = vs;

    /* We can't convert FROM a VBO, and vertex buffers used to source into
     * process_vertices() are unlikely to ever be used for drawing. Release
     * VBOs in those buffers and fix up the stream_info structure.
     *
     * Also apply the start index. */
    map = stream_info.use_map as uint32_t;
    while map != 0 {
        let mut e: *mut wined3d_stream_info_element = core::ptr::null_mut();
        let mut map_desc: wined3d_map_desc = core::mem::zeroed();

        i = wined3d_bit_scan(core::ptr::addr_of_mut!(map)) as core::ffi::c_uint;
        e = (core::ptr::addr_of_mut!(stream_info.elements) as *mut wined3d_stream_info_element)
            .offset(i as isize);
        resource = core::ptr::addr_of_mut!(
            (*(*(*state).streams.as_ptr().offset((*e).stream_idx as isize)).buffer).resource
        );
        box_.left = (src_start_idx as core::ffi::c_uint).wrapping_mul((*e).stride) as UINT;
        box_.right = (box_.left as core::ffi::c_uint)
            .wrapping_add((vertex_count as core::ffi::c_uint).wrapping_mul((*e).stride))
            as UINT;
        if (wined3d_resource_map(
            resource,
            0,
            core::ptr::addr_of_mut!(map_desc),
            core::ptr::addr_of_mut!(box_),
            WINED3D_MAP_READ,
        )) < 0
        {
            ERR!("Failed to map resource.\n");
            map = stream_info.use_map as uint32_t;
            while map != 0 {
                j = wined3d_bit_scan(core::ptr::addr_of_mut!(map)) as core::ffi::c_uint;
                if j >= i {
                    break;
                }

                e = (core::ptr::addr_of_mut!(stream_info.elements)
                    as *mut wined3d_stream_info_element)
                    .offset(j as isize);
                resource = core::ptr::addr_of_mut!(
                    (*(*(*state).streams.as_ptr().offset((*e).stream_idx as isize)).buffer)
                        .resource
                );
                if (wined3d_resource_unmap(resource, 0))
                    < 0
                {
                    ERR!("Failed to unmap resource.\n");
                }
            }
            return WINED3DERR_INVALIDCALL;
        }
        (*e).data.buffer_object = core::ptr::null_mut();
        (*e).data.addr = (*e).data.addr.offset(map_desc.data as ULONG_PTR as isize);
    }

    hr = process_vertices_strided(
        device,
        state,
        dst_idx,
        vertex_count,
        core::ptr::addr_of_mut!(stream_info),
        dst_buffer,
        flags,
        dst_fvf,
    );

    map = stream_info.use_map as uint32_t;
    while map != 0 {
        i = wined3d_bit_scan(core::ptr::addr_of_mut!(map)) as core::ffi::c_uint;
        resource = core::ptr::addr_of_mut!(
            (*(*(*state).streams.as_ptr().offset(
                (*(core::ptr::addr_of_mut!(stream_info.elements) as *mut wined3d_stream_info_element)
                    .offset(i as isize))
                .stream_idx as isize,
            ))
            .buffer)
            .resource
        );
        if (wined3d_resource_unmap(resource, 0)) < 0 {
            ERR!("Failed to unmap resource.\n");
        }
    }

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_get_device_caps(
    device: *const wined3d_device,
    caps: *mut wined3d_caps,
) -> HRESULT {
    return wined3d_get_device_caps((*device).adapter, (*device).create_parms.device_type, caps);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_get_display_mode(
    device: *const wined3d_device,
    swapchain_idx: UINT,
    mode: *mut wined3d_display_mode,
    rotation: *mut wined3d_display_rotation,
) -> HRESULT {
    let mut swapchain: *mut wined3d_swapchain = core::ptr::null_mut();
    WINE_TRACE!("device %p, swapchain_idx %u, mode %p, rotation %p.\n", device, swapchain_idx, mode, rotation);
    swapchain = wined3d_device_get_swapchain(device, swapchain_idx);
    if swapchain.is_null() {
        return WINED3DERR_INVALIDCALL;
    }
    return wined3d_swapchain_get_display_mode(swapchain, mode, rotation);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_begin_scene(device: *mut wined3d_device) -> HRESULT {
    /* At the moment we have no need for any functionality at the beginning
     * of a scene. */

    if (*device).inScene() != 0 {
        WARN!("Already in scene, returning WINED3DERR_INVALIDCALL.\n");
        return WINED3DERR_INVALIDCALL;
    }
    (*device).set_inScene((TRUE) as _);
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_end_scene(device: *mut wined3d_device) -> HRESULT {
    if (*device).inScene() == 0 {
        WARN!("Not in scene, returning WINED3DERR_INVALIDCALL.\n");
        return WINED3DERR_INVALIDCALL;
    }

    (*device).set_inScene((FALSE) as _);
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_clear(device: *mut wined3d_device, rect_count: core::ffi::c_uint, rects: *const RECT, flags: uint32_t, color: *const wined3d_color, depth: core::ffi::c_float, stencil: core::ffi::c_uint) -> HRESULT {
    let mut fb: *mut wined3d_fb_state = core::ptr::addr_of_mut!((*(*(*device).cs).c.state).fb);
    TRACE!("device %p, rect_count %u, rects %p, flags %#x, color %s, depth %.8e, stencil %u.\n", device, rect_count, rects, flags, wined3d_cs_emit_clear as usize, depth as core::ffi::c_double, stencil);
    if rect_count == 0 && !rects.is_null() {
        WARN!("Rects is %p, but rect_count is 0, ignoring clear\n", rects);
        return WINED3D_OK;
    }
    if flags & ((WINED3DCLEAR_ZBUFFER | WINED3DCLEAR_STENCIL) as uint32_t) != 0 {
        let mut ds: *mut wined3d_rendertarget_view = (*fb).depth_stencil;
        if ds.is_null() {
            WARN!("Clearing depth and/or stencil without a depth stencil buffer attached, returning WINED3DERR_INVALIDCALL\n");
            return WINED3DERR_INVALIDCALL;
        } else if flags & (WINED3DCLEAR_TARGET as uint32_t) != 0 {
            if (*ds).width < (*(*fb).render_targets[0 as usize]).width
                || (*ds).height < (*(*fb).render_targets[0 as usize]).height
            {
                WARN!("Silently ignoring depth and target clear with mismatching sizes\n");
                return WINED3D_OK;
            }
        }
    }
    wined3d_cs_emit_clear((*device).cs as *mut wined3d_cs, rect_count as DWORD, rects, flags, color, depth, stencil as DWORD);
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_get_predication(
    mut context: *mut wined3d_device_context,
    mut value: *mut BOOL,
) -> *mut wined3d_query {
    let mut state: *mut wined3d_state = (*context).state;
    WINE_TRACE!("context %p, value %p.\n", context, value);
    if !value.is_null() {
        *value = (*state).predicate_value;
    }
    return (*state).predicate;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_set_primitive_type(
    mut context: *mut wined3d_device_context,
    mut primitive_type: wined3d_primitive_type,
    mut patch_vertex_count: ::core::ffi::c_uint,
) {
    let mut state: *mut wined3d_state = (*context).state;
    WINE_TRACE!("context %p, primitive_type %s, patch_vertex_count %u.\n", context, debug_d3dprimitivetype(primitive_type), patch_vertex_count);
    wined3d_device_context_lock(context);
    (*state).primitive_type = primitive_type;
    (*state).patch_vertex_count = patch_vertex_count;
    wined3d_device_context_unlock(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_get_primitive_type(context: *const wined3d_device_context, primitive_type: *mut wined3d_primitive_type, patch_vertex_count: *mut ::core::ffi::c_uint) {
    let state: *const wined3d_state = (*context).state;
    TRACE!("context %p, primitive_type %p, patch_vertex_count %p.\n", context, primitive_type, patch_vertex_count);
    *primitive_type = (*state).primitive_type;
    if !patch_vertex_count.is_null() {
        *patch_vertex_count = (*state).patch_vertex_count;
    }
    TRACE!("Returning %s.\n", debug_d3dprimitivetype(*primitive_type));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_update_texture(
    mut device: *mut wined3d_device,
    mut src_texture: *mut wined3d_texture,
    mut dst_texture: *mut wined3d_texture,
) -> HRESULT {
    let mut src_size: ::core::ffi::c_uint = 0;
    let mut dst_size: ::core::ffi::c_uint = 0;
    let mut src_skip_levels: ::core::ffi::c_uint = 0;
    let mut src_level_count: ::core::ffi::c_uint = 0;
    let mut dst_level_count: ::core::ffi::c_uint = 0;
    let mut regions: *const wined3d_dirty_regions = ::core::ptr::null::<wined3d_dirty_regions>();
    let mut layer_count: ::core::ffi::c_uint = 0;
    let mut level_count: ::core::ffi::c_uint = 0;
    let mut i: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_uint = 0;
    let mut type_0: wined3d_resource_type = WINED3D_RTYPE_NONE;
    let mut entire_texture: BOOL = (TRUE as i32);
    let mut box_0: wined3d_box = wined3d_box {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        front: 0,
        back: 0,
    };

    if src_texture.is_null() || dst_texture.is_null() {
        WARN!("Source and destination textures must be non-NULL, returning WINED3DERR_INVALIDCALL.\n");
        return WINED3DERR_INVALIDCALL;
    }

    if (*src_texture).resource.access & WINED3D_RESOURCE_ACCESS_GPU != 0
        || (*src_texture).resource.usage & WINED3DUSAGE_SCRATCH as uint32_t != 0
    {
        WARN!("Source resource is GPU accessible or a scratch resource.\n");
        return WINED3DERR_INVALIDCALL;
    }
    if (*dst_texture).resource.access & WINED3D_RESOURCE_ACCESS_CPU != 0 {
        WARN!("Destination resource is CPU accessible.\n");
        return WINED3DERR_INVALIDCALL;
    }

    type_0 = (*src_texture).resource.type_;
    if (*dst_texture).resource.type_ as ::core::ffi::c_uint != type_0 as ::core::ffi::c_uint {
        WARN!("Source and destination have different types, returning WINED3DERR_INVALIDCALL.\n");
        return WINED3DERR_INVALIDCALL;
    }

    layer_count = (*src_texture).layer_count as ::core::ffi::c_uint;
    if layer_count != (*dst_texture).layer_count {
        WARN!("Source and destination have different layer counts.\n");
        return WINED3DERR_INVALIDCALL;
    }

    if (*src_texture).resource.format != (*dst_texture).resource.format {
        WARN!("Source and destination formats do not match.\n");
        return WINED3DERR_INVALIDCALL;
    }

    src_level_count = (*src_texture).level_count;
    dst_level_count = (*dst_texture).level_count;
    level_count = min!(src_level_count, dst_level_count);

    src_size = max!((*src_texture).resource.width, (*src_texture).resource.height);
    src_size = max!(src_size, (*src_texture).resource.depth);
    dst_size = max!((*dst_texture).resource.width, (*dst_texture).resource.height);
    dst_size = max!(dst_size, (*dst_texture).resource.depth);
    while src_size > dst_size {
        src_size >>= 1;
        src_skip_levels = src_skip_levels.wrapping_add(1);
    }

    if wined3d_texture_get_level_width(src_texture, src_skip_levels) != (*dst_texture).resource.width
        || wined3d_texture_get_level_height(src_texture, src_skip_levels) != (*dst_texture).resource.height
        || wined3d_texture_get_level_depth(src_texture, src_skip_levels) != (*dst_texture).resource.depth
    {
        WARN!("Source and destination dimensions do not match.\n");
        return WINED3DERR_INVALIDCALL;
    }

    regions = (*src_texture).dirty_regions as *const wined3d_dirty_regions;
    if !regions.is_null() {
        i = 0;
        while i < layer_count && entire_texture != 0 {
            if (*regions.offset(i as isize)).box_count >= WINED3D_MAX_DIRTY_REGION_COUNT as ::core::ffi::c_uint {
                i = i.wrapping_add(1);
                continue;
            }

            entire_texture = (FALSE as i32);
            break;
        }
    }

    if entire_texture != 0 {
        i = 0;
        while i < level_count {
            wined3d_texture_get_level_box(dst_texture, i, &raw mut box_0);
            j = 0;
            while j < layer_count {
                wined3d_device_context_emit_blt_sub_resource(
                    &raw mut (*(*device).cs).c,
                    &raw mut (*dst_texture).resource,
                    j.wrapping_mul(dst_level_count).wrapping_add(i),
                    &raw mut box_0,
                    &raw mut (*src_texture).resource,
                    j.wrapping_mul(src_level_count)
                        .wrapping_add(i)
                        .wrapping_add(src_skip_levels),
                    &raw mut box_0,
                    0,
                    ::core::ptr::null::<wined3d_blt_fx>(),
                    WINED3D_TEXF_POINT,
                );
                j = j.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
    } else {
        let mut src_level: ::core::ffi::c_uint = 0;
        let mut box_count: ::core::ffi::c_uint = 0;
        let mut k: ::core::ffi::c_uint = 0;
        let mut boxes: *const wined3d_box = ::core::ptr::null::<wined3d_box>();
        let mut b: wined3d_box = wined3d_box {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
            front: 0,
            back: 0,
        };

        i = 0;
        while i < layer_count {
            boxes = (*regions.offset(i as isize)).boxes;
            box_count = (*regions.offset(i as isize)).box_count;
            if (*regions.offset(i as isize)).box_count >= WINED3D_MAX_DIRTY_REGION_COUNT as ::core::ffi::c_uint {
                boxes = &raw mut b;
                box_count = 1;
                wined3d_texture_get_level_box(dst_texture, i, &raw mut b);
            }

            j = 0;
            while j < level_count {
                src_level = j.wrapping_add(src_skip_levels);

                k = 0;
                while k < box_count {
                    box_0 = *boxes.offset(k as isize);
                    if src_level != 0 {
                        box_0.left >>= src_level;
                        box_0.top >>= src_level;
                        box_0.right = min!(
                            ((box_0.right as ::core::ffi::c_uint)
                                .wrapping_add(1 << src_level)
                                .wrapping_sub(1))
                                >> src_level,
                            wined3d_texture_get_level_width(src_texture, src_level) as UINT as ::core::ffi::c_uint,
                        ) as UINT;
                        box_0.bottom = min!(
                            ((box_0.bottom as ::core::ffi::c_uint)
                                .wrapping_add(1 << src_level)
                                .wrapping_sub(1))
                                >> src_level,
                            wined3d_texture_get_level_height(src_texture, src_level) as UINT as ::core::ffi::c_uint,
                        ) as UINT;
                        box_0.front >>= src_level;
                        box_0.back = min!(
                            ((box_0.back as ::core::ffi::c_uint)
                                .wrapping_add(1 << src_level)
                                .wrapping_sub(1))
                                >> src_level,
                            wined3d_texture_get_level_depth(src_texture, src_level) as UINT as ::core::ffi::c_uint,
                        ) as UINT;
                    }

                    wined3d_device_context_emit_blt_sub_resource(
                        &raw mut (*(*device).cs).c,
                        &raw mut (*dst_texture).resource,
                        i.wrapping_mul(dst_level_count).wrapping_add(j),
                        &raw mut box_0,
                        &raw mut (*src_texture).resource,
                        i.wrapping_mul(src_level_count).wrapping_add(src_level),
                        &raw mut box_0,
                        0,
                        ::core::ptr::null::<wined3d_blt_fx>(),
                        WINED3D_TEXF_POINT,
                    );
                    k = k.wrapping_add(1);
                }
                j = j.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
    }

    wined3d_texture_clear_dirty_regions(src_texture);

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_validate_device(
    device: *const wined3d_device,
    state: *const wined3d_stateblock_state,
    num_passes: *mut DWORD,
) -> HRESULT {
    let device_state: *const wined3d_state = (*(*device).cs).c.state;
    let mut texture: *mut wined3d_texture = core::ptr::null_mut::<wined3d_texture>();
    let mut i: ::core::ffi::c_uint = 0;

    TRACE!("device %p, num_passes %p.\n", device, num_passes);

    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_MAX_COMBINED_SAMPLERS as ::core::ffi::c_uint {
        if (*state).sampler_states[i as usize][WINED3D_SAMP_MIN_FILTER as usize]
            == WINED3D_TEXF_NONE as uint32_t
        {
            WARN!("Sampler state %u has minfilter D3DTEXF_NONE, returning D3DERR_UNSUPPORTEDTEXTUREFILTER\n", i);
            return WINED3DERR_UNSUPPORTEDTEXTUREFILTER;
        }
        if (*state).sampler_states[i as usize][WINED3D_SAMP_MAG_FILTER as usize]
            == WINED3D_TEXF_NONE as uint32_t
        {
            WARN!("Sampler state %u has magfilter D3DTEXF_NONE, returning D3DERR_UNSUPPORTEDTEXTUREFILTER\n", i);
            return WINED3DERR_UNSUPPORTEDTEXTUREFILTER;
        }

        texture = (*state).textures[i as usize] as *mut wined3d_texture;
        if texture.is_null()
            || (*texture).resource.format_caps & WINED3D_FORMAT_CAP_FILTERING as ::core::ffi::c_uint != 0
        {
            i = i.wrapping_add(1);
            continue;
        }

        if (*state).sampler_states[i as usize][WINED3D_SAMP_MAG_FILTER as usize]
            != WINED3D_TEXF_POINT as uint32_t
        {
            WARN!("Non-filterable texture and mag filter enabled on sampler %u, returning E_FAIL\n", i);
            return E_FAIL;
        }
        if (*state).sampler_states[i as usize][WINED3D_SAMP_MIN_FILTER as usize]
            != WINED3D_TEXF_POINT as uint32_t
        {
            WARN!("Non-filterable texture and min filter enabled on sampler %u, returning E_FAIL\n", i);
            return E_FAIL;
        }
        if (*state).sampler_states[i as usize][WINED3D_SAMP_MIP_FILTER as usize]
            != WINED3D_TEXF_NONE as uint32_t
            && (*state).sampler_states[i as usize][WINED3D_SAMP_MIP_FILTER as usize]
                != WINED3D_TEXF_POINT as uint32_t
        {
            WARN!("Non-filterable texture and mip filter enabled on sampler %u, returning E_FAIL\n", i);
            return E_FAIL;
        }

        i = i.wrapping_add(1);
    }

    if (*state).rs[WINED3D_RS_ZENABLE as usize] != 0
        || (*state).rs[WINED3D_RS_ZWRITEENABLE as usize] != 0
        || (*state).rs[WINED3D_RS_STENCILENABLE as usize] != 0
    {
        let rt: *mut wined3d_rendertarget_view =
            (*device_state).fb.render_targets[0 as usize];
        let ds: *mut wined3d_rendertarget_view = (*device_state).fb.depth_stencil;

        if !ds.is_null()
            && !rt.is_null()
            && ((*ds).width < (*rt).width || (*ds).height < (*rt).height)
        {
            WARN!("Depth stencil is smaller than the color buffer, returning D3DERR_CONFLICTINGRENDERSTATE\n");
            return WINED3DERR_CONFLICTINGRENDERSTATE;
        }
    }

    /* return a sensible default */
    *num_passes = 1 as DWORD;

    TRACE!("returning D3D_OK\n");
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_set_software_vertex_processing(device: *mut wined3d_device, software: BOOL) {
    static mut warned: BOOL = 0;

    TRACE!("device %p, software %#x.\n", device, software);
    if warned == 0 {
        FIXME!("device %p, software %#x stub!\n", device, software);
        warned = (TRUE as i32);
    }

    (*device).set_softwareVertexProcessing((software) as _);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_get_software_vertex_processing(device: *const wined3d_device) -> BOOL {
    static mut warned: BOOL = 0;

    TRACE!("device %p.\n", device);

    if warned == 0 {
        FIXME!("device %p stub!\n", device);
        warned = (TRUE as i32);
    }

    return (*device).softwareVertexProcessing() as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_get_raster_status(
    device: *const wined3d_device,
    swapchain_idx: UINT,
    raster_status: *mut wined3d_raster_status,
) -> HRESULT {
    let mut swapchain: *mut wined3d_swapchain = core::ptr::null_mut();

    swapchain = wined3d_device_get_swapchain(device, swapchain_idx);
    if swapchain.is_null() {
        return WINED3DERR_INVALIDCALL;
    }

    return wined3d_swapchain_get_raster_status(swapchain, raster_status);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_set_npatch_mode(
    mut device: *mut wined3d_device,
    mut segments: ::core::ffi::c_float,
) -> HRESULT {
    static mut warned: BOOL = 0;
    WINE_TRACE!("device %p, segments %.8e.\n", device, segments as ::core::ffi::c_double);
    if segments != 0.0f32 {
        if warned == 0 {
            WINE_FIXME!("device %p, segments %.8e stub!\n", device, segments as ::core::ffi::c_double);
            warned = (TRUE as i32);
        }
    }

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_get_npatch_mode(device: *const wined3d_device) -> f32 {
    static mut warned: BOOL = 0;

    TRACE!("device %p.\n", device);

    if warned == 0 {
        FIXME!("device %p stub!\n", device);
        warned = TRUE as BOOL;
    }

    return 0.0f32;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_copy_uav_counter(
    context: *mut wined3d_device_context,
    dst_buffer: *mut wined3d_buffer,
    offset: u32,
    uav: *mut wined3d_unordered_access_view,
) {
    WINE_TRACE!("context %p, dst_buffer %p, offset %u, uav %p.\n", context, dst_buffer, offset, uav);
    wined3d_device_context_lock(context);
    wined3d_device_context_emit_copy_uav_counter(context, dst_buffer, offset, uav);
    wined3d_device_context_unlock(context);
}

unsafe extern "C" fn resources_format_compatible(
    src_resource: *const wined3d_resource,
    dst_resource: *const wined3d_resource,
) -> bool {
    if (*(*src_resource).format).id == (*(*dst_resource).format).id {
        return true;
    }
    if (*(*src_resource).format).typeless_id != 0
        && (*(*src_resource).format).typeless_id == (*(*dst_resource).format).typeless_id
    {
        return true;
    }
    if (*(*(*(*src_resource).device).cs).c.state).feature_level < WINED3D_FEATURE_LEVEL_10_1 {
        return false;
    }
    if ((*src_resource).format_attrs & WINED3D_FORMAT_ATTR_BLOCKS) != 0
        && ((*dst_resource).format_attrs & WINED3D_FORMAT_ATTR_CAST_TO_BLOCK) != 0
    {
        return (*(*src_resource).format).block_byte_count == (*(*dst_resource).format).byte_count;
    }
    if ((*src_resource).format_attrs & WINED3D_FORMAT_ATTR_CAST_TO_BLOCK) != 0
        && ((*dst_resource).format_attrs & WINED3D_FORMAT_ATTR_BLOCKS) != 0
    {
        return (*(*src_resource).format).byte_count == (*(*dst_resource).format).block_byte_count;
    }
    return false;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_copy_resource(
    mut context: *mut wined3d_device_context,
    mut dst_resource: *mut wined3d_resource,
    mut src_resource: *mut wined3d_resource,
) {
    let mut src_row_block_count: ::core::ffi::c_uint = 0;
    let mut dst_row_block_count: ::core::ffi::c_uint = 0;
    let mut dst_texture: *mut wined3d_texture = ::core::ptr::null_mut::<wined3d_texture>();
    let mut src_texture: *mut wined3d_texture = ::core::ptr::null_mut::<wined3d_texture>();
    let mut src_row_count: ::core::ffi::c_uint = 0;
    let mut dst_row_count: ::core::ffi::c_uint = 0;
    let mut src_box: wined3d_box = wined3d_box {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        front: 0,
        back: 0,
    };
    let mut dst_box: wined3d_box = wined3d_box {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        front: 0,
        back: 0,
    };
    let mut i: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_uint = 0;

    if src_resource == dst_resource {
        WARN!("Source and destination are the same resource.\n");
        return;
    }

    if (*src_resource).type_ as ::core::ffi::c_uint
        != (*dst_resource).type_ as ::core::ffi::c_uint
    {
        WARN!(
            "Resource types (%s / %s) don't match.\n",
            debug_d3dresourcetype((*dst_resource).type_),
            debug_d3dresourcetype((*src_resource).type_)
        );
        return;
    }

    if !resources_format_compatible(src_resource, dst_resource) {
        WARN!(
            "Resource formats %s and %s are incompatible.\n",
            debug_d3dformat((*(*dst_resource).format).id),
            debug_d3dformat((*(*src_resource).format).id)
        );
        return;
    }

    src_row_block_count = (*src_resource)
        .width
        .wrapping_add(
            (*(*src_resource).format)
                .block_width
                .wrapping_sub(1),
        )
        .wrapping_div((*(*src_resource).format).block_width)
        as ::core::ffi::c_uint;
    dst_row_block_count = (*dst_resource)
        .width
        .wrapping_add(
            (*(*dst_resource).format)
                .block_width
                .wrapping_sub(1),
        )
        .wrapping_div((*(*dst_resource).format).block_width)
        as ::core::ffi::c_uint;
    src_row_count = (*src_resource)
        .height
        .wrapping_add(
            (*(*src_resource).format)
                .block_height
                .wrapping_sub(1),
        )
        .wrapping_div((*(*src_resource).format).block_height)
        as ::core::ffi::c_uint;
    dst_row_count = (*dst_resource)
        .height
        .wrapping_add(
            (*(*dst_resource).format)
                .block_height
                .wrapping_sub(1),
        )
        .wrapping_div((*(*dst_resource).format).block_height)
        as ::core::ffi::c_uint;

    if src_row_block_count != dst_row_block_count
        || src_row_count != dst_row_count
        || (*src_resource).depth != (*dst_resource).depth
    {
        WARN!(
            "Resource block dimensions (%ux%ux%u / %ux%ux%u) don't match.\n",
            dst_row_block_count,
            dst_row_count,
            (*dst_resource).depth,
            src_row_block_count,
            src_row_count,
            (*src_resource).depth
        );
        return;
    }

    if (*dst_resource).type_ as ::core::ffi::c_uint
        == WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint
    {
        wined3d_box_set(
            core::ptr::addr_of_mut!(src_box),
            0,
            0,
            (*src_resource).size as ::core::ffi::c_uint,
            1,
            0,
            1,
        );
        wined3d_device_context_lock(context);
        wined3d_device_context_emit_blt_sub_resource(
            context,
            dst_resource,
            0,
            core::ptr::addr_of_mut!(src_box),
            src_resource,
            0,
            core::ptr::addr_of_mut!(src_box),
            WINED3D_BLT_RAW as ::core::ffi::c_uint,
            core::ptr::null::<wined3d_blt_fx>(),
            WINED3D_TEXF_POINT,
        );
        wined3d_device_context_unlock(context);
        return;
    }

    dst_texture = texture_from_resource(dst_resource);
    src_texture = texture_from_resource(src_resource);

    if (*src_texture).layer_count != (*dst_texture).layer_count
        || (*src_texture).level_count != (*dst_texture).level_count
    {
        WARN!(
            "Subresource layouts (%ux%u / %ux%u) don't match.\n",
            (*dst_texture).layer_count,
            (*dst_texture).level_count,
            (*src_texture).layer_count,
            (*src_texture).level_count
        );
        return;
    }

    wined3d_device_context_lock(context);
    i = 0;
    while i < (*dst_texture).level_count {
        wined3d_texture_get_level_box(src_texture, i, core::ptr::addr_of_mut!(src_box));
        wined3d_texture_get_level_box(dst_texture, i, core::ptr::addr_of_mut!(dst_box));
        j = 0;
        while j < (*dst_texture).layer_count {
            let mut idx: ::core::ffi::c_uint =
                j.wrapping_mul((*dst_texture).level_count).wrapping_add(i);

            wined3d_device_context_emit_blt_sub_resource(
                context,
                dst_resource,
                idx,
                core::ptr::addr_of_mut!(dst_box),
                src_resource,
                idx,
                core::ptr::addr_of_mut!(src_box),
                WINED3D_BLT_RAW as ::core::ffi::c_uint,
                core::ptr::null::<wined3d_blt_fx>(),
                WINED3D_TEXF_POINT,
            );
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    wined3d_device_context_unlock(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_copy_sub_resource_region(
    mut context: *mut wined3d_device_context,
    mut dst_resource: *mut wined3d_resource,
    mut dst_sub_resource_idx: u32,
    mut dst_x: u32,
    mut dst_y: u32,
    mut dst_z: u32,
    mut src_resource: *mut wined3d_resource,
    mut src_sub_resource_idx: u32,
    mut src_box: *const wined3d_box,
    mut flags: u32,
) -> HRESULT {
    let mut dst_box: wined3d_box = wined3d_box {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        front: 0,
        back: 0,
    };
    let mut b: wined3d_box = wined3d_box {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        front: 0,
        back: 0,
    };

    if flags != 0 {
        FIXME!("Ignoring flags %#x.\n", flags);
    }

    if src_resource == dst_resource && src_sub_resource_idx == dst_sub_resource_idx {
        WARN!("Source and destination are the same sub-resource.\n");
        return WINED3DERR_INVALIDCALL;
    }

    if !resources_format_compatible(src_resource, dst_resource) {
        WARN!(
            "Resource formats %s and %s are incompatible.\n",
            debug_d3dformat((*(*dst_resource).format).id),
            debug_d3dformat((*(*src_resource).format).id)
        );
        return WINED3DERR_INVALIDCALL;
    }

    if (*dst_resource).type_ as u32 == WINED3D_RTYPE_BUFFER as u32 {
        if (*src_resource).type_ as u32 != WINED3D_RTYPE_BUFFER as u32 {
            WARN!(
                "Resource types (%s / %s) don't match.\n",
                debug_d3dresourcetype((*dst_resource).type_),
                debug_d3dresourcetype((*src_resource).type_)
            );
            return WINED3DERR_INVALIDCALL;
        }

        if dst_sub_resource_idx != 0 {
            WARN!("Invalid dst_sub_resource_idx %u.\n", dst_sub_resource_idx);
            return WINED3DERR_INVALIDCALL;
        }

        if src_sub_resource_idx != 0 {
            WARN!("Invalid src_sub_resource_idx %u.\n", src_sub_resource_idx);
            return WINED3DERR_INVALIDCALL;
        }

        if src_box.is_null() {
            let mut dst_w: u32 = 0;

            dst_w = ((*dst_resource).size as u32).wrapping_sub(dst_x);
            wined3d_box_set(
                core::ptr::addr_of_mut!(b),
                0,
                0,
                min!((*src_resource).size as u32, dst_w),
                1,
                0,
                1
            );
            src_box = core::ptr::addr_of_mut!(b);
        } else if (*src_box).left >= (*src_box).right
            || (*src_box).top >= (*src_box).bottom
            || (*src_box).front >= (*src_box).back
        {
            WARN!(
                "Invalid box %s specified.\n",
                debug_box(src_box)
            );
            return WINED3DERR_INVALIDCALL;
        }

        if (*src_box).right > (*src_resource).size as u32
            || dst_x >= (*dst_resource).size as u32
            || (*src_box).right.wrapping_sub((*src_box).left)
                > ((*dst_resource).size as u32).wrapping_sub(dst_x)
        {
            WARN!(
                "Invalid range specified, dst_offset %u, src_offset %u, size %u.\n",
                dst_x,
                (*src_box).left,
                (*src_box).right.wrapping_sub((*src_box).left)
            );
            return WINED3DERR_INVALIDCALL;
        }

        wined3d_box_set(
            core::ptr::addr_of_mut!(dst_box),
            dst_x,
            0,
            dst_x.wrapping_add((*src_box).right.wrapping_sub((*src_box).left)),
            1,
            0,
            1
        );
    } else {
        let mut dst_texture: *mut wined3d_texture = texture_from_resource(dst_resource);
        let mut src_texture: *mut wined3d_texture = texture_from_resource(src_resource);
        let mut src_level: u32 = src_sub_resource_idx.wrapping_rem((*src_texture).level_count);
        let mut src_row_block_count: u32 = 0;
        let mut src_row_count: u32 = 0;

        if !wined3d_texture_validate_sub_resource_idx(dst_texture, dst_sub_resource_idx) {
            return WINED3DERR_INVALIDCALL;
        }

        if !wined3d_texture_validate_sub_resource_idx(src_texture, src_sub_resource_idx) {
            return WINED3DERR_INVALIDCALL;
        }

        if (*(*dst_texture)
            .sub_resources
            .offset(dst_sub_resource_idx as isize))
        .map_count
            != 0
        {
            WARN!(
                "Destination sub-resource %u is mapped.\n",
                dst_sub_resource_idx
            );
            return WINED3DERR_INVALIDCALL;
        }

        if (*(*src_texture)
            .sub_resources
            .offset(src_sub_resource_idx as isize))
        .map_count
            != 0
        {
            WARN!("Source sub-resource %u is mapped.\n", src_sub_resource_idx);
            return WINED3DERR_INVALIDCALL;
        }

        if src_box.is_null() {
            let mut src_w: u32 = 0;
            let mut src_h: u32 = 0;
            let mut src_d: u32 = 0;
            let mut dst_w: u32 = 0;
            let mut dst_h: u32 = 0;
            let mut dst_d: u32 = 0;
            let mut dst_level: u32 = 0;

            src_w = wined3d_texture_get_level_width(src_texture, src_level);
            src_h = wined3d_texture_get_level_height(src_texture, src_level);
            src_d = wined3d_texture_get_level_depth(src_texture, src_level);

            dst_level = dst_sub_resource_idx.wrapping_rem((*dst_texture).level_count);
            dst_w = wined3d_texture_get_level_width(dst_texture, dst_level).wrapping_sub(dst_x);
            dst_h = wined3d_texture_get_level_height(dst_texture, dst_level).wrapping_sub(dst_y);
            dst_d = wined3d_texture_get_level_depth(dst_texture, dst_level).wrapping_sub(dst_z);

            wined3d_box_set(
                core::ptr::addr_of_mut!(b),
                0,
                0,
                min!(src_w, dst_w),
                min!(src_h, dst_h),
                0,
                min!(src_d, dst_d)
            );
            src_box = core::ptr::addr_of_mut!(b);
        } else if wined3d_resource_check_box_dimensions(src_resource, src_sub_resource_idx, src_box)
            < 0
        {
            WARN!(
                "Invalid source box %s.\n",
                debug_box(src_box)
            );
            return WINED3DERR_INVALIDCALL;
        }

        if (*(*src_resource).format).block_width == (*(*dst_resource).format).block_width
            && (*(*src_resource).format).block_height == (*(*dst_resource).format).block_height
        {
            wined3d_box_set(
                core::ptr::addr_of_mut!(dst_box),
                dst_x,
                dst_y,
                dst_x.wrapping_add((*src_box).right.wrapping_sub((*src_box).left)),
                dst_y.wrapping_add((*src_box).bottom.wrapping_sub((*src_box).top)),
                dst_z,
                dst_z.wrapping_add((*src_box).back.wrapping_sub((*src_box).front))
            );
        } else {
            src_row_block_count = (*src_box)
                .right
                .wrapping_sub((*src_box).left)
                .wrapping_add((*(*src_resource).format).block_width)
                .wrapping_sub(1)
                .wrapping_div((*(*src_resource).format).block_width);
            src_row_count = (*src_box)
                .bottom
                .wrapping_sub((*src_box).top)
                .wrapping_add((*(*src_resource).format).block_height)
                .wrapping_sub(1)
                .wrapping_div((*(*src_resource).format).block_height);
            wined3d_box_set(
                core::ptr::addr_of_mut!(dst_box),
                dst_x,
                dst_y,
                dst_x.wrapping_add(
                    src_row_block_count.wrapping_mul((*(*dst_resource).format).block_width),
                ),
                dst_y.wrapping_add(
                    src_row_count.wrapping_mul((*(*dst_resource).format).block_height),
                ),
                dst_z,
                dst_z.wrapping_add((*src_box).back.wrapping_sub((*src_box).front))
            );
        }

        if wined3d_resource_check_box_dimensions(
            dst_resource,
            dst_sub_resource_idx,
            core::ptr::addr_of_mut!(dst_box),
        ) < 0
        {
            WARN!(
                "Invalid destination box %s.\n",
                debug_box(core::ptr::addr_of_mut!(dst_box))
            );
            return WINED3DERR_INVALIDCALL;
        }
    }

    wined3d_device_context_lock(context);
    wined3d_device_context_emit_blt_sub_resource(
        context,
        dst_resource,
        dst_sub_resource_idx,
        core::ptr::addr_of_mut!(dst_box),
        src_resource,
        src_sub_resource_idx,
        src_box,
        WINED3D_BLT_RAW,
        core::ptr::null(),
        WINED3D_TEXF_POINT
    );
    wined3d_device_context_unlock(context);

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_update_sub_resource(
    context: *mut wined3d_device_context,
    resource: *mut wined3d_resource,
    sub_resource_idx: core::ffi::c_uint,
    box_0: *const wined3d_box,
    data: *const core::ffi::c_void,
    row_pitch: core::ffi::c_uint,
    depth_pitch: core::ffi::c_uint,
    flags: core::ffi::c_uint,
) {
    let mut desc: wined3d_sub_resource_desc = core::mem::zeroed();
    let mut b: wined3d_box = core::mem::zeroed();
    let mut box_0 = box_0;

    if flags != 0 {
        FIXME!("Ignoring flags %#x.\n", flags);
    }

    if (*resource).access & (WINED3D_RESOURCE_ACCESS_GPU) == 0 {
        WARN!("Resource %p is not GPU accessible.\n", resource);
        return;
    }

    if (wined3d_resource_get_sub_resource_desc(resource, sub_resource_idx, core::ptr::addr_of_mut!(desc)))
        < (0)
    {
        return;
    }

    if box_0.is_null() {
        wined3d_box_set(
            core::ptr::addr_of_mut!(b),
            0,
            0,
            desc.width,
            desc.height,
            0,
            desc.depth,
        );
        box_0 = core::ptr::addr_of!(b);
    } else if (*box_0).left >= (*box_0).right
        || (*box_0).right > desc.width
        || (*box_0).top >= (*box_0).bottom
        || (*box_0).bottom > desc.height
        || (*box_0).front >= (*box_0).back
        || (*box_0).back > desc.depth
    {
        WARN!("Invalid box %s specified.\n", debug_box(box_0));
        return;
    }

    wined3d_device_context_lock(context);
    wined3d_device_context_emit_update_sub_resource(
        context,
        resource,
        sub_resource_idx,
        box_0,
        data,
        row_pitch,
        depth_pitch,
    );
    wined3d_device_context_unlock(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_resolve_sub_resource(
    context: *mut wined3d_device_context,
    dst_resource: *mut wined3d_resource,
    dst_sub_resource_idx: core::ffi::c_uint,
    src_resource: *mut wined3d_resource,
    src_sub_resource_idx: core::ffi::c_uint,
    format_id: wined3d_format_id,
) {
    let mut dst_texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut src_texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut dst_level: core::ffi::c_uint = 0;
    let mut src_level: core::ffi::c_uint = 0;
    let mut fx: wined3d_blt_fx = core::mem::zeroed();
    let mut dst_rect: RECT = core::mem::zeroed();
    let mut src_rect: RECT = core::mem::zeroed();

    if wined3d_format_is_typeless((*dst_resource).format) != 0
        || wined3d_format_is_typeless((*src_resource).format) != 0
    {
        // FIXME: multisample resolve of typeless formats is not fully supported
        // (diagnostic body is blank in the C source).
    }
    if (*dst_resource).type_ != WINED3D_RTYPE_TEXTURE_2D {
        WARN!("Invalid destination resource type %s.\n", debug_d3dresourcetype((*dst_resource).type_));
        return;
    }
    if (*src_resource).type_ != WINED3D_RTYPE_TEXTURE_2D {
        WARN!("Invalid source resource type %s.\n", debug_d3dresourcetype((*src_resource).type_));
        return;
    }

    wined3d_device_context_lock(context);
    fx.resolve_format_id = format_id;

    dst_texture = texture_from_resource(dst_resource);
    src_texture = texture_from_resource(src_resource);

    dst_level = dst_sub_resource_idx % (*dst_texture).level_count;
    SetRect(
        core::ptr::addr_of_mut!(dst_rect),
        0,
        0,
        (wined3d_texture_get_level_width(dst_texture, dst_level)) as i32,
        (wined3d_texture_get_level_height(dst_texture, dst_level)) as i32,
    );
    src_level = src_sub_resource_idx % (*src_texture).level_count;
    SetRect(
        core::ptr::addr_of_mut!(src_rect),
        0,
        0,
        (wined3d_texture_get_level_width(src_texture, src_level)) as i32,
        (wined3d_texture_get_level_height(src_texture, src_level)) as i32,
    );
    wined3d_device_context_blt(
        context,
        dst_texture,
        dst_sub_resource_idx,
        core::ptr::addr_of_mut!(dst_rect),
        src_texture,
        src_sub_resource_idx,
        core::ptr::addr_of_mut!(src_rect),
        0,
        core::ptr::addr_of_mut!(fx),
        WINED3D_TEXF_POINT,
    );
    wined3d_device_context_unlock(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_clear_rendertarget_view(
    context: *mut wined3d_device_context,
    view: *mut wined3d_rendertarget_view,
    mut rect: *const RECT,
    flags: u32,
    color: *const wined3d_color,
    depth: f32,
    stencil: u32,
) -> HRESULT {
    let mut resource: *mut wined3d_resource = core::ptr::null_mut();
    let mut r: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    TRACE!(
        "context %p, view %p, rect %s, flags %#x, color %s, depth %.8e, stencil %u.\n",
        context,
        view,
        wine_dbgstr_rect(rect),
        flags,
        debug_color(color),
        depth as f64,
        stencil
    );

    if flags == 0 {
        return WINED3D_OK;
    }

    resource = (*view).resource;
    if (*resource).type_ as u32 == WINED3D_RTYPE_BUFFER as u32 {
        FIXME!(
            "Not implemented for %s resources.\n",
            debug_d3dresourcetype((*resource).type_)
        );
        return WINED3DERR_INVALIDCALL;
    }

    if rect.is_null() {
        SetRect(
            &raw mut r,
            0,
            0,
            (*view).width as i32,
            (*view).height as i32,
        );
        rect = &raw mut r;
    } else {
        let mut b: wined3d_box = wined3d_box {
            left: (*rect).left as UINT,
            top: (*rect).top as UINT,
            right: (*rect).right as UINT,
            bottom: (*rect).bottom as UINT,
            front: 0 as UINT,
            back: 1 as UINT,
        };
        let mut hr: HRESULT = 0;

        hr = wined3d_resource_check_box_dimensions(resource, (*view).sub_resource_idx, &raw mut b);
        if hr < 0 {
            return hr;
        }
    }

    wined3d_device_context_lock(context);
    wined3d_device_context_emit_clear_rendertarget_view(context, view, rect, flags, color, depth, stencil);
    wined3d_device_context_unlock(context);

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_clear_uav_float(
    context: *mut wined3d_device_context,
    view: *mut wined3d_unordered_access_view,
    clear_value: *const wined3d_vec4,
) {
    if ((*(*view).format).attrs & (WINED3D_FORMAT_ATTR_FLOAT | WINED3D_FORMAT_ATTR_NORMALISED)) == 0 {
        WARN!("Not supported for view format %s.\n", debug_d3dformat((*(*view).format).id));
        return;
    }

    wined3d_device_context_lock(context);
    wined3d_device_context_emit_clear_uav(context, view, clear_value as *const wined3d_uvec4, true);
    wined3d_device_context_unlock(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_clear_uav_uint(
    mut context: *mut wined3d_device_context,
    mut view: *mut wined3d_unordered_access_view,
    mut clear_value: *const wined3d_uvec4,
) {
    WINE_TRACE!("context %p, view %p, clear_value %s.\n", context, view, debug_uvec4(clear_value));
    wined3d_device_context_lock(context);
    wined3d_device_context_emit_clear_uav(context, view, clear_value, false);
    wined3d_device_context_unlock(context);
}

unsafe extern "C" fn sanitise_map_flags(
    resource: *const wined3d_resource,
    mut flags: ::core::ffi::c_uint,
) -> ::core::ffi::c_uint {
    /* Not all flags make sense together, but Windows never returns an error.
     * Catch the cases that could cause issues. */
    if (flags & (WINED3D_MAP_READ as ::core::ffi::c_uint)) != 0 {
        if (flags & (WINED3D_MAP_DISCARD as ::core::ffi::c_uint)) != 0 {
            WARN!("WINED3D_MAP_READ combined with WINED3D_MAP_DISCARD, ignoring flags.\n");
            return flags & ((WINED3D_MAP_READ as ::core::ffi::c_uint) | (WINED3D_MAP_WRITE as ::core::ffi::c_uint));
        }
        if (flags & (WINED3D_MAP_NOOVERWRITE as ::core::ffi::c_uint)) != 0 {
            WARN!("WINED3D_MAP_READ combined with WINED3D_MAP_NOOVERWRITE, ignoring flags.\n");
            return flags & ((WINED3D_MAP_READ as ::core::ffi::c_uint) | (WINED3D_MAP_WRITE as ::core::ffi::c_uint));
        }
    } else if (flags & ((WINED3D_MAP_DISCARD as ::core::ffi::c_uint) | (WINED3D_MAP_NOOVERWRITE as ::core::ffi::c_uint))) != 0 {
        if ((*resource).access & (WINED3D_RESOURCE_ACCESS_GPU as uint32_t)) == 0
            || ((*resource).usage & (WINED3DUSAGE_DYNAMIC as uint32_t)) == 0
        {
            WARN!("DISCARD or NOOVERWRITE map not on dynamic GPU-accessible buffer, ignoring.\n");
            return flags & ((WINED3D_MAP_READ as ::core::ffi::c_uint) | (WINED3D_MAP_WRITE as ::core::ffi::c_uint));
        }
        if (flags & ((WINED3D_MAP_DISCARD as ::core::ffi::c_uint) | (WINED3D_MAP_NOOVERWRITE as ::core::ffi::c_uint)))
            == ((WINED3D_MAP_DISCARD as ::core::ffi::c_uint) | (WINED3D_MAP_NOOVERWRITE as ::core::ffi::c_uint))
        {
            WARN!("WINED3D_MAP_NOOVERWRITE used with WINED3D_MAP_DISCARD, ignoring WINED3D_MAP_DISCARD.\n");
            flags &= !(WINED3D_MAP_DISCARD as ::core::ffi::c_uint);
        }
    }

    return flags;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_map(
    context: *mut wined3d_device_context,
    resource: *mut wined3d_resource,
    sub_resource_idx: ::core::ffi::c_uint,
    map_desc: *mut wined3d_map_desc,
    mut box_0: *const wined3d_box,
    mut flags: ::core::ffi::c_uint,
) -> HRESULT {
    let mut desc: wined3d_sub_resource_desc = core::mem::zeroed();
    let mut b: wined3d_box = core::mem::zeroed();
    let mut hr: HRESULT = 0;

    TRACE!(
        "context %p, resource %p, sub_resource_idx %u, map_desc %p, box %s, flags %#x.\n",
        context,
        resource,
        sub_resource_idx,
        map_desc,
        debug_box(box_0),
        flags
    );

    if (flags & (WINED3D_MAP_READ | WINED3D_MAP_WRITE)) == 0 {
        WARN!("No read/write flags specified.\n");
        return E_INVALIDARG;
    }

    if (flags & WINED3D_MAP_READ) != 0 && ((*resource).access & WINED3D_RESOURCE_ACCESS_MAP_R) == 0 {
        WARN!("Resource does not have MAP_R access.\n");
        return E_INVALIDARG;
    }

    if (flags & WINED3D_MAP_WRITE) != 0 && ((*resource).access & WINED3D_RESOURCE_ACCESS_MAP_W) == 0 {
        WARN!("Resource does not have MAP_W access.\n");
        return E_INVALIDARG;
    }

    flags = sanitise_map_flags(resource, flags);

    if (wined3d_resource_get_sub_resource_desc(resource, sub_resource_idx, &raw mut desc)) < 0 {
        return E_INVALIDARG;
    }

    if box_0.is_null() {
        wined3d_box_set(
            &raw mut b,
            0,
            0,
            desc.width,
            desc.height,
            0,
            desc.depth,
        );
        box_0 = &raw mut b;
    } else if (wined3d_resource_check_box_dimensions(resource, sub_resource_idx, box_0)) < 0 {
        WARN!("Map box is invalid.\n");

        if (*resource).type_ != WINED3D_RTYPE_BUFFER && (*resource).type_ != WINED3D_RTYPE_TEXTURE_2D {
            return WINED3DERR_INVALIDCALL;
        }

        if ((*resource).format_attrs & WINED3D_FORMAT_ATTR_BLOCKS) != 0
            && ((*resource).access & WINED3D_RESOURCE_ACCESS_CPU) == 0
        {
            return WINED3DERR_INVALIDCALL;
        }
    }

    wined3d_device_context_lock(context);
    hr = wined3d_device_context_emit_map(context, resource, sub_resource_idx, map_desc, box_0, flags);
    wined3d_device_context_unlock(context);
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_unmap(
    context: *mut wined3d_device_context,
    resource: *mut wined3d_resource,
    sub_resource_idx: ::core::ffi::c_uint,
) -> HRESULT {
    let mut hr: HRESULT;

    wined3d_device_context_lock(context);
    hr = wined3d_device_context_emit_unmap(context, resource, sub_resource_idx);
    wined3d_device_context_unlock(context);
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_issue_query(
    mut context: *mut wined3d_device_context,
    mut query: *mut wined3d_query,
    mut flags: ::core::ffi::c_uint,
) {
    WINE_TRACE!("context %p, query %p, flags %#x.\n", context, query, flags);
    wined3d_device_context_lock(context);
    (*(*context).ops)
        .issue_query
        .unwrap_unchecked()(context, query, flags);
    wined3d_device_context_unlock(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_execute_command_list(
    mut context: *mut wined3d_device_context,
    mut list: *mut wined3d_command_list,
    mut restore_state: bool,
) {
    WINE_TRACE!("context %p, list %p, restore_state %d.\n", context, list, restore_state as ::core::ffi::c_int);
    wined3d_device_context_lock(context);
    wined3d_device_context_emit_execute_command_list(context, list, restore_state);
    wined3d_device_context_unlock(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_get_rendertarget_view(
    context: *const wined3d_device_context,
    view_idx: ::core::ffi::c_uint,
) -> *mut wined3d_rendertarget_view {
    let mut max_rt_count: ::core::ffi::c_uint;

    max_rt_count = (*(*(*context).device).adapter).d3d_info.limits.max_rt_count;
    if view_idx >= max_rt_count {
        WARN!("Only %u render targets are supported.\n", max_rt_count);
        return core::ptr::null_mut();
    }

    return (*(*context).state).fb.render_targets[view_idx as usize];
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_get_depth_stencil_view(
    context: *const wined3d_device_context,
) -> *mut wined3d_rendertarget_view {
    TRACE!("context %p.\n", context);

    return (*(*context).state).fb.depth_stencil;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_generate_mipmaps(
    mut context: *mut wined3d_device_context,
    mut view: *mut wined3d_shader_resource_view,
) {
    let mut texture: *mut wined3d_texture = ::core::ptr::null_mut::<wined3d_texture>();

    if (*(*view).resource).type_ == WINED3D_RTYPE_BUFFER {
        WARN!("Called on buffer resource %p.\n", (*view).resource);
        return;
    }

    texture = texture_from_resource((*view).resource);
    if ((*texture).flags & WINED3D_TEXTURE_GENERATE_MIPMAPS) == 0 {
        WARN!("Texture without the WINED3D_TEXTURE_GENERATE_MIPMAPS flag, ignoring.\n");
        return;
    }

    wined3d_device_context_lock(context);
    wined3d_device_context_emit_generate_mipmaps(context, view);
    wined3d_device_context_unlock(context);
}

unsafe extern "C" fn wined3d_device_create_cursor_texture(mut device: *mut wined3d_device, mut cursor_image: *mut wined3d_texture, mut sub_resource_idx: u32) -> *mut wined3d_texture {
    let mut texture_level: u32 = sub_resource_idx % (*cursor_image).level_count;
    let mut data: wined3d_sub_resource_data = core::mem::zeroed();
    let mut desc: wined3d_resource_desc = core::mem::zeroed();
    let mut map_desc: wined3d_map_desc = core::mem::zeroed();
    let mut texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    if (wined3d_resource_map(core::ptr::addr_of_mut!((*cursor_image).resource), sub_resource_idx, core::ptr::addr_of_mut!(map_desc), core::ptr::null(), WINED3D_MAP_READ)) < (0)
    {
        ERR!("Failed to map source texture.\n");
        return core::ptr::null_mut();
    }

    data.data = map_desc.data;
    data.row_pitch = map_desc.row_pitch;
    data.slice_pitch = map_desc.slice_pitch;

    desc.resource_type = WINED3D_RTYPE_TEXTURE_2D;
    desc.format = WINED3DFMT_B8G8R8A8_UNORM;
    desc.multisample_type = WINED3D_MULTISAMPLE_NONE;
    desc.multisample_quality = 0;
    desc.usage = WINED3DUSAGE_DYNAMIC;
    desc.bind_flags = 0;
    desc.access = WINED3D_RESOURCE_ACCESS_GPU;
    desc.width = wined3d_texture_get_level_width(cursor_image, texture_level);
    desc.height = wined3d_texture_get_level_height(cursor_image, texture_level);
    desc.depth = 1;
    desc.size = 0;

    hr = wined3d_texture_create(device, core::ptr::addr_of_mut!(desc), 1, 1, 0, core::ptr::addr_of_mut!(data), core::ptr::null_mut(), core::ptr::addr_of!(wined3d_null_parent_ops), core::ptr::addr_of_mut!(texture));
    wined3d_resource_unmap(core::ptr::addr_of_mut!((*cursor_image).resource), sub_resource_idx);
    if (hr) < (0)
    {
        ERR!("Failed to create cursor texture.\n");
        return core::ptr::null_mut();
    }

    return texture;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_set_cursor_properties(
    mut device: *mut wined3d_device,
    mut x_hotspot: UINT,
    mut y_hotspot: UINT,
    mut texture: *mut wined3d_texture,
    mut sub_resource_idx: ::core::ffi::c_uint,
) -> HRESULT {
    let mut texture_level: ::core::ffi::c_uint =
        sub_resource_idx.wrapping_rem((*texture).level_count);
    let mut cursor_width: ::core::ffi::c_uint = 0;
    let mut cursor_height: ::core::ffi::c_uint = 0;
    let mut map_desc: wined3d_map_desc = core::mem::zeroed();

    if !wined3d_texture_validate_sub_resource_idx(texture, sub_resource_idx)
        || (*texture).resource.type_ as ::core::ffi::c_uint != WINED3D_RTYPE_TEXTURE_2D
    {
        return WINED3DERR_INVALIDCALL;
    }

    if !(*device).cursor_texture.is_null() {
        wined3d_texture_decref((*device).cursor_texture);
        (*device).cursor_texture = core::ptr::null_mut::<wined3d_texture>();
    }

    if (*(*texture).resource.format).id as ::core::ffi::c_uint != WINED3DFMT_B8G8R8A8_UNORM {
        WARN!(
            "Texture %p has invalid format %s.\n",
            texture,
            debug_d3dformat((*(*texture).resource.format).id)
        );
        return WINED3DERR_INVALIDCALL;
    }

    /* Cursor width and height must all be powers of two */
    cursor_width = wined3d_texture_get_level_width(texture, texture_level);
    cursor_height = wined3d_texture_get_level_height(texture, texture_level);
    if (cursor_width & cursor_width.wrapping_sub(1)) != 0
        || (cursor_height & cursor_height.wrapping_sub(1)) != 0
    {
        WARN!(
            "Cursor size %ux%u are not all powers of two.\n",
            cursor_width,
            cursor_height
        );
        return WINED3DERR_INVALIDCALL;
    }

    /* Do not store the surface's pointer because the application may
     * release it after setting the cursor image. Windows doesn't
     * addref the set surface, so we can't do this either without
     * creating circular refcount dependencies. */
    (*device).cursor_texture = wined3d_device_create_cursor_texture(device, texture, sub_resource_idx);
    if (*device).cursor_texture.is_null() {
        ERR!("Failed to create cursor texture.\n");
        return WINED3DERR_INVALIDCALL;
    }

    if cursor_width == 32 && cursor_height == 32 {
        let mut mask_size: UINT = (cursor_width as UINT)
            .wrapping_mul(cursor_height as UINT)
            .wrapping_div(8);
        let mut cursor_info: ICONINFO = core::mem::zeroed();
        let mut mask_bits: *mut DWORD = core::ptr::null_mut::<DWORD>();
        let mut cursor: HCURSOR = core::ptr::null_mut();

        /* 32-bit user32 cursors ignore the alpha channel if it's all
         * zeroes, and use the mask instead. Fill the mask with all ones
         * to ensure we still get a fully transparent cursor. */
        mask_bits = malloc(mask_size as size_t) as *mut DWORD;
        if mask_bits.is_null() {
            return E_OUTOFMEMORY;
        }
        memset(
            mask_bits as *mut ::core::ffi::c_void,
            0xff,
            mask_size as size_t,
        );

        wined3d_resource_map(
            &raw mut (*texture).resource,
            sub_resource_idx,
            &raw mut map_desc,
            core::ptr::null::<wined3d_box>(),
            WINED3D_MAP_NO_DIRTY_UPDATE as uint32_t | WINED3D_MAP_READ as uint32_t,
        );
        cursor_info.fIcon = FALSE as BOOL;
        cursor_info.xHotspot = x_hotspot as DWORD;
        cursor_info.yHotspot = y_hotspot as DWORD;
        cursor_info.hbmMask = CreateBitmap(
            cursor_width as INT,
            cursor_height as INT,
            1 as UINT,
            1 as UINT,
            mask_bits as *const ::core::ffi::c_void,
        );
        cursor_info.hbmColor = CreateBitmap(
            cursor_width as INT,
            cursor_height as INT,
            1 as UINT,
            32 as UINT,
            map_desc.data,
        );
        wined3d_resource_unmap(&raw mut (*texture).resource, sub_resource_idx);

        /* Create our cursor and clean up. */
        cursor = CreateIconIndirect(&raw mut cursor_info) as HCURSOR;
        if !cursor_info.hbmMask.is_null() {
            DeleteObject(cursor_info.hbmMask as HGDIOBJ);
        }
        if !cursor_info.hbmColor.is_null() {
            DeleteObject(cursor_info.hbmColor as HGDIOBJ);
        }
        if !(*device).hardwareCursor.is_null() {
            DestroyCursor((*device).hardwareCursor);
        }
        (*device).hardwareCursor = cursor;
        if (*device).bCursorVisible() != 0 {
            SetCursor(cursor);
        }

        free(mask_bits as *mut ::core::ffi::c_void);
    }

    (*device).cursorWidth = cursor_width as UINT;
    (*device).cursorHeight = cursor_height as UINT;
    (*device).xHotSpot = x_hotspot;
    (*device).yHotSpot = y_hotspot;

    return WINED3D_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_device_set_cursor_position(
    device: *mut wined3d_device,
    x_screen_space: i32,
    y_screen_space: i32,
    flags: u32,
) {
    TRACE!(
        "device %p, x %d, y %d, flags %#x.\n",
        device,
        x_screen_space,
        y_screen_space,
        flags
    );
    (*device).xScreenSpace = x_screen_space as UINT;
    (*device).yScreenSpace = y_screen_space as UINT;

    if !(*device).hardwareCursor.is_null() {
        let mut pt: POINT = POINT { x: 0, y: 0 };

        GetCursorPos(core::ptr::addr_of_mut!(pt));
        if x_screen_space == pt.x && y_screen_space == pt.y {
            return;
        }
        SetCursorPos(x_screen_space, y_screen_space);

        /* Switch to the software cursor if position diverges from the hardware one. */
        GetCursorPos(core::ptr::addr_of_mut!(pt));
        if x_screen_space != pt.x || y_screen_space != pt.y {
            if (*device).bCursorVisible() != 0 {
                SetCursor(core::ptr::null_mut());
            }
            DestroyCursor((*device).hardwareCursor);
            (*device).hardwareCursor = core::ptr::null_mut();
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_show_cursor(device: *mut wined3d_device, show: BOOL) -> BOOL {
    let mut oldVisible: BOOL = (*device).bCursorVisible() as BOOL;

    /*
     * When ShowCursor is first called it should make the cursor appear at the OS's last
     * known cursor position.
     */
    if show != 0 && oldVisible == 0 {
        let mut pt: POINT = POINT { x: 0, y: 0 };
        GetCursorPos(core::ptr::addr_of_mut!(pt));
        (*device).xScreenSpace = pt.x as UINT;
        (*device).yScreenSpace = pt.y as UINT;
    }

    if !(*device).hardwareCursor.is_null() {
        (*device).set_bCursorVisible((show as BYTE) as _);
        if show != 0 {
            SetCursor((*device).hardwareCursor);
        } else {
            SetCursor(core::ptr::null_mut());
        }
    } else if !(*device).cursor_texture.is_null() {
        (*device).set_bCursorVisible((show as BYTE) as _);
    }

    return oldVisible;
}

unsafe extern "C" fn mark_managed_resource_dirty(mut resource: *mut wined3d_resource) {
    if (*resource).type_ as ::core::ffi::c_uint
        != WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint
    {
        let mut texture: *mut wined3d_texture = texture_from_resource(resource);
        let mut i: ::core::ffi::c_uint = 0;
        if !(*texture).dirty_regions.is_null() {
            i = 0 as ::core::ffi::c_uint;
            while i < (*texture).layer_count {
                wined3d_texture_add_dirty_region(
                    texture,
                    i as UINT,
                    ::core::ptr::null_mut::<wined3d_box>(),
                );
                i = i.wrapping_add(1);
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_evict_managed_resources(device: *mut wined3d_device) {
    let mut resource: *mut wined3d_resource = core::ptr::null_mut();
    let mut cursor: *mut wined3d_resource = core::ptr::null_mut();
    TRACE!("device %p.\n", device);
    resource = ((*device).resources.next as *mut u8)
        .sub(core::mem::offset_of!(wined3d_resource, resource_list_entry)) as *mut wined3d_resource;
    cursor = ((*resource).resource_list_entry.next as *mut u8)
        .sub(core::mem::offset_of!(wined3d_resource, resource_list_entry)) as *mut wined3d_resource;
    while core::ptr::addr_of_mut!((*resource).resource_list_entry)
        != core::ptr::addr_of_mut!((*device).resources)
    {
        TRACE!("Checking resource %p for eviction.\n", resource);
        if ((*resource).usage & WINED3DUSAGE_MANAGED) != 0 && (*resource).map_count == 0 {
            if ((*resource).access & WINED3D_RESOURCE_ACCESS_GPU) != 0 {
                TRACE!("Evicting %p.\n", resource);
                wined3d_cs_emit_unload_resource((*device).cs as *mut wined3d_cs, resource);
            }

            mark_managed_resource_dirty(resource);
        }
        resource = cursor;
        cursor = ((*resource).resource_list_entry.next as *mut u8)
            .sub(core::mem::offset_of!(wined3d_resource, resource_list_entry)) as *mut wined3d_resource;
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_context_flush(context: *mut wined3d_device_context) {
    wined3d_device_context_lock(context);
    ((*(*context).ops).flush.unwrap_unchecked())(context);
    wined3d_device_context_unlock(context);
}

unsafe extern "C" fn update_swapchain_flags(texture: *mut wined3d_texture) {
    let mut flags: ::core::ffi::c_uint = (*(*texture).swapchain).state.desc.flags as ::core::ffi::c_uint;
    if flags & WINED3D_SWAPCHAIN_LOCKABLE_BACKBUFFER != 0 {
        (*texture).resource.access |= WINED3D_RESOURCE_ACCESS_MAP_R | WINED3D_RESOURCE_ACCESS_MAP_W;
    } else {
        (*texture).resource.access &= !(WINED3D_RESOURCE_ACCESS_MAP_R | WINED3D_RESOURCE_ACCESS_MAP_W);
    }
    if flags & WINED3D_SWAPCHAIN_GDI_COMPATIBLE != 0 {
        (*texture).flags |= WINED3D_TEXTURE_GET_DC as uint32_t;
    } else {
        (*texture).flags &= !WINED3D_TEXTURE_GET_DC as uint32_t;
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_reset(
    device: *mut wined3d_device,
    swapchain_desc: *const wined3d_swapchain_desc,
    mode: *const wined3d_display_mode,
    callback: wined3d_device_reset_cb,
    reset_state: BOOL,
) -> HRESULT {
    static mut views: [*mut wined3d_rendertarget_view; (WINED3D_MAX_RENDER_TARGETS as _)] =
        [core::ptr::null_mut::<wined3d_rendertarget_view>(); (WINED3D_MAX_RENDER_TARGETS as _)];
    let d3d_info: *const wined3d_d3d_info = core::ptr::addr_of!((*(*device).adapter).d3d_info);
    let context: *mut wined3d_device_context = core::ptr::addr_of_mut!((*(*device).cs).c);
    let mut swapchain_state: *mut wined3d_swapchain_state = core::ptr::null_mut();
    let state: *mut wined3d_state = (*context).state;
    let mut current_desc: *mut wined3d_swapchain_desc = core::ptr::null_mut();
    let mut resource: *mut wined3d_resource = core::ptr::null_mut();
    let mut cursor: *mut wined3d_resource = core::ptr::null_mut();
    let mut view: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut swapchain: *mut wined3d_swapchain = core::ptr::null_mut();
    let mut view_desc: wined3d_view_desc = core::mem::zeroed();
    let mut backbuffer_resized: BOOL = 0;
    let mut windowed: BOOL = 0;
    let mut hr: HRESULT = WINED3D_OK;
    let mut device_window: HWND = core::ptr::null_mut();
    let mut i: ::core::ffi::c_uint = 0;

    TRACE!(
        "device %p, swapchain_desc %p, mode %p, callback %p, reset_state %#x.\n",
        device,
        swapchain_desc,
        mode,
        callback,
        reset_state
    );

    wined3d_cs_finish((*device).cs as *mut wined3d_cs, WINED3D_CS_QUEUE_DEFAULT);

    swapchain = wined3d_device_get_swapchain(device, 0);
    if swapchain.is_null() {
        ERR!("Failed to get the first implicit swapchain.\n");
        return WINED3DERR_INVALIDCALL;
    }
    swapchain_state = core::ptr::addr_of_mut!((*swapchain).state);
    current_desc = core::ptr::addr_of_mut!((*swapchain_state).desc);

    if reset_state != 0 {
        if !(*device).logo_texture.is_null() {
            wined3d_texture_decref((*device).logo_texture);
            (*device).logo_texture = core::ptr::null_mut();
        }
        if !(*device).cursor_texture.is_null() {
            wined3d_texture_decref((*device).cursor_texture);
            (*device).cursor_texture = core::ptr::null_mut();
        }
        let mut i: ::core::ffi::c_uint = 0;
        while (i as usize) < ARRAY_SIZE!((*device).push_constants) as usize {
            if !(*device).push_constants[i as usize].is_null() {
                wined3d_buffer_decref((*device).push_constants[i as usize]);
            }
            (*device).push_constants[i as usize] = core::ptr::null_mut();
            i += 1;
        }
        state_unbind_resources(state);
    }

    wined3d_device_context_set_rendertarget_views(
        context,
        0,
        (*d3d_info).limits.max_rt_count,
        core::ptr::addr_of!(views) as *const *mut wined3d_rendertarget_view,
        (FALSE) as BOOL,
    );
    wined3d_device_context_set_depth_stencil_view(context, core::ptr::null_mut());

    if reset_state != 0 {
        resource = ((*device).resources.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(wined3d_resource, resource_list_entry) as isize),
        ) as *mut wined3d_resource;
        cursor = ((*resource).resource_list_entry.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(wined3d_resource, resource_list_entry) as isize),
        ) as *mut wined3d_resource;
        while core::ptr::addr_of_mut!((*resource).resource_list_entry)
            != core::ptr::addr_of_mut!((*device).resources)
        {
            TRACE!("Enumerating resource %p.\n", resource);
            hr = (callback.unwrap_unchecked())(resource);
            if hr < 0 {
                return hr;
            }
            resource = cursor;
            cursor = ((*resource).resource_list_entry.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(wined3d_resource, resource_list_entry) as isize),
            ) as *mut wined3d_resource;
        }
    }

    TRACE!("New params:\n");
    TRACE!("output %p\n", (*swapchain_desc).output);
    TRACE!("backbuffer_width %u\n", (*swapchain_desc).backbuffer_width);
    TRACE!("backbuffer_height %u\n", (*swapchain_desc).backbuffer_height);
    TRACE!("backbuffer_count %u\n", (*swapchain_desc).backbuffer_count);
    TRACE!("multisample_type %#x\n", (*swapchain_desc).multisample_type as ::core::ffi::c_uint);
    TRACE!("multisample_quality %u\n", (*swapchain_desc).multisample_quality);
    TRACE!("swap_effect %#x\n", (*swapchain_desc).swap_effect as ::core::ffi::c_uint);
    TRACE!("device_window %p\n", (*swapchain_desc).device_window);
    TRACE!("windowed %#x\n", (*swapchain_desc).windowed);
    TRACE!("enable_auto_depth_stencil %#x\n", (*swapchain_desc).enable_auto_depth_stencil);
    if (*swapchain_desc).enable_auto_depth_stencil != 0 {
        TRACE!(
            "auto_depth_stencil_format %s\n",
            debug_d3dformat((*swapchain_desc).auto_depth_stencil_format)
        );
    }
    TRACE!("flags %#x\n", (*swapchain_desc).flags);
    TRACE!("refresh_rate %u\n", (*swapchain_desc).refresh_rate);
    TRACE!("auto_restore_display_mode %#x\n", (*swapchain_desc).auto_restore_display_mode);

    if (*swapchain_desc).backbuffer_bind_flags != 0
        && (*swapchain_desc).backbuffer_bind_flags != WINED3D_BIND_RENDER_TARGET
    {
        FIXME!(
            "Got unexpected backbuffer bind flags %#x.\n",
            (*swapchain_desc).backbuffer_bind_flags
        );
    }

    if (*swapchain_desc).swap_effect != WINED3D_SWAP_EFFECT_DISCARD
        && (*swapchain_desc).swap_effect != WINED3D_SWAP_EFFECT_SEQUENTIAL
        && (*swapchain_desc).swap_effect != WINED3D_SWAP_EFFECT_COPY
    {
        FIXME!("Unimplemented swap effect %#x.\n", (*swapchain_desc).swap_effect);
    }

    /* No special treatment of these parameters. Just store them */
    (*current_desc).swap_effect = (*swapchain_desc).swap_effect;
    (*current_desc).enable_auto_depth_stencil = (*swapchain_desc).enable_auto_depth_stencil;
    (*current_desc).auto_depth_stencil_format = (*swapchain_desc).auto_depth_stencil_format;
    (*current_desc).refresh_rate = (*swapchain_desc).refresh_rate;
    (*current_desc).auto_restore_display_mode = (*swapchain_desc).auto_restore_display_mode;

    device_window = if !(*swapchain_desc).device_window.is_null() {
        (*swapchain_desc).device_window
    } else {
        (*device).create_parms.focus_window
    };
    if !device_window.is_null() && device_window != (*current_desc).device_window {
        TRACE!(
            "Changing the device window from %p to %p.\n",
            (*current_desc).device_window,
            device_window
        );
        (*current_desc).device_window = device_window;
        (*swapchain_state).device_window = device_window;
        wined3d_swapchain_set_window(swapchain, core::ptr::null_mut());
    }

    backbuffer_resized = ((*swapchain_desc).backbuffer_width != (*current_desc).backbuffer_width
        || (*swapchain_desc).backbuffer_height != (*current_desc).backbuffer_height)
        as ::core::ffi::c_int as BOOL;
    windowed = (*current_desc).windowed;

    if ((*swapchain_desc).windowed == 0) != (windowed == 0)
        || (*swapchain).reapply_mode != false
        || !mode.is_null()
        || ((*swapchain_desc).windowed == 0 && backbuffer_resized != 0)
    {
        /* Switch from windowed to fullscreen. */
        if windowed != 0 && (*swapchain_desc).windowed == 0 {
            let mut focus_window: HWND = (*device).create_parms.focus_window;

            if focus_window.is_null() {
                focus_window = (*swapchain).state.device_window;
            }
            hr = wined3d_device_acquire_focus_window(device, focus_window);
            if hr < 0 {
                ERR!("Failed to acquire focus window, hr %#lx.\n", hr);
                return hr;
            }
        }

        hr = wined3d_swapchain_state_set_fullscreen(core::ptr::addr_of_mut!((*swapchain).state), swapchain_desc, mode);
        if hr < 0 {
            return hr;
        }

        /* Switch from fullscreen to windowed. */
        if windowed == 0 && (*swapchain_desc).windowed != 0 {
            wined3d_device_release_focus_window(device);
        }
    } else if (*swapchain_desc).windowed == 0 {
        let style: DWORD = (*swapchain_state).style as DWORD;
        let exstyle: DWORD = (*swapchain_state).exstyle as DWORD;
        let mut output_desc: wined3d_output_desc = core::mem::zeroed();

        /* If we're in fullscreen, and the mode wasn't changed, we have to get
         * the window back into the right position. Some applications
         * (Battlefield 2, Guild Wars) move it and then call Reset() to clean
         * up their mess. Guild Wars also loses the device during that. */
        hr = wined3d_output_get_desc((*swapchain_desc).output, core::ptr::addr_of_mut!(output_desc));
        if hr < 0 {
            ERR!("Failed to get output description, hr %#lx.\n", hr);
            return hr;
        }

        (*swapchain_state).style = 0;
        (*swapchain_state).exstyle = 0;
        wined3d_swapchain_state_setup_fullscreen(
            swapchain_state,
            (*swapchain_state).device_window,
            output_desc.desktop_rect.left,
            output_desc.desktop_rect.top,
            ((*swapchain_desc).backbuffer_width) as i32,
            ((*swapchain_desc).backbuffer_height) as i32,
        );
        (*swapchain_state).style = (style) as i32;
        (*swapchain_state).exstyle = (exstyle) as i32;
    }

    hr = wined3d_swapchain_resize_buffers(
        swapchain,
        (*swapchain_desc).backbuffer_count,
        (*swapchain_desc).backbuffer_width,
        (*swapchain_desc).backbuffer_height,
        (*swapchain_desc).backbuffer_format,
        (*swapchain_desc).multisample_type,
        (*swapchain_desc).multisample_quality,
    );
    if hr < 0 {
        return hr;
    }

    if (*swapchain_desc).flags != (*current_desc).flags {
        (*current_desc).flags = (*swapchain_desc).flags;

        update_swapchain_flags((*swapchain).front_buffer);
        i = 0;
        while i < (*current_desc).backbuffer_count {
            update_swapchain_flags(*(*swapchain).back_buffers.offset(i as isize));
            i = i.wrapping_add(1);
        }
    }

    view = (*device).auto_depth_stencil_view;
    if !view.is_null() {
        (*device).auto_depth_stencil_view = core::ptr::null_mut();
        wined3d_rendertarget_view_decref(view);
    }
    if (*current_desc).enable_auto_depth_stencil != 0 {
        let mut texture_desc: wined3d_resource_desc = core::mem::zeroed();
        let mut texture: *mut wined3d_texture = core::ptr::null_mut();

        TRACE!("Creating the depth stencil buffer.\n");

        texture_desc.resource_type = WINED3D_RTYPE_TEXTURE_2D;
        texture_desc.format = (*current_desc).auto_depth_stencil_format;
        texture_desc.multisample_type = (*current_desc).multisample_type;
        texture_desc.multisample_quality = (*current_desc).multisample_quality;
        texture_desc.usage = 0;
        texture_desc.bind_flags = WINED3D_BIND_DEPTH_STENCIL;
        texture_desc.access = WINED3D_RESOURCE_ACCESS_GPU;
        texture_desc.width = (*current_desc).backbuffer_width;
        texture_desc.height = (*current_desc).backbuffer_height;
        texture_desc.depth = 1;
        texture_desc.size = 0;

        hr = wined3d_texture_create(
            device,
            core::ptr::addr_of_mut!(texture_desc),
            1,
            1,
            0,
            core::ptr::null::<wined3d_sub_resource_data>(),
            core::ptr::null_mut::<core::ffi::c_void>(),
            core::ptr::addr_of!(wined3d_null_parent_ops),
            core::ptr::addr_of_mut!(texture),
        );
        if hr < 0 {
            ERR!("Failed to create the auto depth/stencil surface, hr %#lx.\n", hr);
            return WINED3DERR_INVALIDCALL;
        }

        view_desc.format_id = (*(*texture).resource.format).id;
        view_desc.flags = 0;
        view_desc.u.texture.level_idx = 0;
        view_desc.u.texture.level_count = 1;
        view_desc.u.texture.layer_idx = 0;
        view_desc.u.texture.layer_count = 1;
        hr = wined3d_rendertarget_view_create(
            core::ptr::addr_of_mut!(view_desc),
            core::ptr::addr_of_mut!((*texture).resource),
            core::ptr::null_mut::<core::ffi::c_void>(),
            core::ptr::addr_of!(wined3d_null_parent_ops),
            core::ptr::addr_of_mut!((*device).auto_depth_stencil_view),
        );
        wined3d_texture_decref(texture);
        if hr < 0 {
            ERR!("Failed to create rendertarget view, hr %#lx.\n", hr);
            return hr;
        }
    }

    view = (*device).back_buffer_view;
    if !view.is_null() {
        (*device).back_buffer_view = core::ptr::null_mut();
        wined3d_rendertarget_view_decref(view);
    }
    if (*current_desc).backbuffer_count != 0
        && ((*current_desc).backbuffer_bind_flags & WINED3D_BIND_RENDER_TARGET) != 0
    {
        let back_buffer: *mut wined3d_resource = core::ptr::addr_of_mut!(
            (**(*swapchain).back_buffers.offset(0 as isize)).resource
        );

        view_desc.format_id = (*(*back_buffer).format).id;
        view_desc.flags = 0;
        view_desc.u.texture.level_idx = 0;
        view_desc.u.texture.level_count = 1;
        view_desc.u.texture.layer_idx = 0;
        view_desc.u.texture.layer_count = 1;
        hr = wined3d_rendertarget_view_create(
            core::ptr::addr_of_mut!(view_desc),
            back_buffer,
            core::ptr::null_mut::<core::ffi::c_void>(),
            core::ptr::addr_of!(wined3d_null_parent_ops),
            core::ptr::addr_of_mut!((*device).back_buffer_view),
        );
        if hr < 0 {
            ERR!("Failed to create rendertarget view, hr %#lx.\n", hr);
            return hr;
        }
    }

    wine_rb_destroy(
        core::ptr::addr_of_mut!((*device).samplers),
        Some(
            device_free_sampler
                as unsafe extern "C" fn(*mut rb_entry, *mut ::core::ffi::c_void) -> (),
        ),
        core::ptr::null_mut(),
    );
    wine_rb_destroy(
        core::ptr::addr_of_mut!((*device).rasterizer_states),
        Some(
            device_free_rasterizer_state
                as unsafe extern "C" fn(*mut rb_entry, *mut ::core::ffi::c_void) -> (),
        ),
        core::ptr::null_mut(),
    );
    wine_rb_destroy(
        core::ptr::addr_of_mut!((*device).blend_states),
        Some(
            device_free_blend_state
                as unsafe extern "C" fn(*mut rb_entry, *mut ::core::ffi::c_void) -> (),
        ),
        core::ptr::null_mut(),
    );
    wine_rb_destroy(
        core::ptr::addr_of_mut!((*device).depth_stencil_states),
        Some(
            device_free_depth_stencil_state
                as unsafe extern "C" fn(*mut rb_entry, *mut ::core::ffi::c_void) -> (),
        ),
        core::ptr::null_mut(),
    );

    if reset_state != 0 {
        TRACE!("Resetting state.\n");
        if (*device).inScene() != 0 {
            wined3d_device_end_scene(device);
        }
        wined3d_device_context_emit_reset_state(core::ptr::addr_of_mut!((*(*device).cs).c), false);
        state_cleanup(state);

        resource = ((*device).resources.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(wined3d_resource, resource_list_entry) as isize),
        ) as *mut wined3d_resource;
        cursor = ((*resource).resource_list_entry.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(wined3d_resource, resource_list_entry) as isize),
        ) as *mut wined3d_resource;
        while core::ptr::addr_of_mut!((*resource).resource_list_entry)
            != core::ptr::addr_of_mut!((*device).resources)
        {
            TRACE!("Unloading resource %p.\n", resource);
            wined3d_cs_emit_unload_resource((*device).cs as *mut wined3d_cs, resource);

            if ((*resource).usage & WINED3DUSAGE_MANAGED) != 0 {
                mark_managed_resource_dirty(resource);
            }
            resource = cursor;
            cursor = ((*resource).resource_list_entry.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(wined3d_resource, resource_list_entry) as isize),
            ) as *mut wined3d_resource;
        }

        (*(*(*device).adapter).adapter_ops)
            .adapter_uninit_3d
            .unwrap_unchecked()(device);

        wined3d_state_reset(state, core::ptr::addr_of_mut!((*(*device).adapter).d3d_info));

        device_init_swapchain_state(device, swapchain);
        if !wined3d_settings.logo.is_null() {
            device_load_logo(device, wined3d_settings.logo);
        }

        hr = (*(*(*device).adapter).adapter_ops)
            .adapter_init_3d
            .unwrap_unchecked()(device);
    } else {
        view = (*device).back_buffer_view;
        if !view.is_null() {
            wined3d_device_context_set_rendertarget_views(
                context,
                0,
                1,
                core::ptr::addr_of_mut!(view),
                (FALSE) as BOOL,
            );
        }
        view = (*device).auto_depth_stencil_view;
        if !view.is_null() {
            wined3d_device_context_set_depth_stencil_view(context, view);
        }
    }

    /* All done. There is no need to reload resources or shaders, this will
     * happen automatically on the first use. */
    return hr;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_device_set_dialog_box_mode(device: *mut wined3d_device, enable_dialogs: BOOL) -> HRESULT {
    if enable_dialogs == 0 {
        FIXME!("Dialogs cannot be disabled yet.\n");
    }

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_get_creation_parameters(
    device: *const wined3d_device,
    parameters: *mut wined3d_device_creation_parameters,
) {
    *parameters = (*device).create_parms;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_device_get_wined3d(
    device: *const wined3d_device,
) -> *mut wined3d {
    // TRACE("device %p.\n", device);
    (*device).wined3d
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_set_gamma_ramp(
    device: *const wined3d_device,
    swapchain_idx: UINT,
    flags: uint32_t,
    ramp: *const wined3d_gamma_ramp,
) {
    let mut swapchain: *mut wined3d_swapchain;

    swapchain = wined3d_device_get_swapchain(device, swapchain_idx);
    if !swapchain.is_null() {
        wined3d_swapchain_set_gamma_ramp(swapchain, flags, ramp);
    }
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_device_get_gamma_ramp(
    device: *const wined3d_device,
    swapchain_idx: UINT,
    ramp: *mut wined3d_gamma_ramp,
) {
    let mut swapchain: *mut wined3d_swapchain = core::ptr::null_mut();

    TRACE!(
        "device %p, swapchain_idx %u, ramp %p.\n",
        device,
        swapchain_idx,
        ramp
    );

    swapchain = wined3d_device_get_swapchain(device, swapchain_idx);
    if !swapchain.is_null() {
        wined3d_swapchain_get_gamma_ramp(swapchain, ramp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_gl_get_backup_dc(device_gl: *mut wined3d_device_gl) -> HDC {
    if (*device_gl).backup_dc.is_null() {
        (*device_gl).backup_wnd = CreateWindowA!(WINED3D_OPENGL_WINDOW_CLASS_NAME.as_ptr(), b"WineD3D fake window\0".as_ptr() as *const i8,
                WS_OVERLAPPEDWINDOW, 10, 10, 10, 10, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
        if (*device_gl).backup_wnd.is_null() {
            ERR!("Failed to create a window.\n");
            return core::ptr::null_mut();
        }

        (*device_gl).backup_dc = GetDC((*device_gl).backup_wnd);
        if (*device_gl).backup_dc.is_null() {
            ERR!("Failed to get a DC.\n");
            DestroyWindow((*device_gl).backup_wnd);
            (*device_gl).backup_wnd = core::ptr::null_mut();
            return core::ptr::null_mut();
        }
    }

    return (*device_gl).backup_dc;
}
#[no_mangle]
pub unsafe extern "C" fn device_resource_add(device: *mut wined3d_device, resource: *mut wined3d_resource) {
    wined3d_not_from_cs((*device).cs);

    list_add_head(
        core::ptr::addr_of_mut!((*device).resources),
        core::ptr::addr_of_mut!((*resource).resource_list_entry),
    );
}

unsafe extern "C" fn device_resource_remove(device: *mut wined3d_device, resource: *mut wined3d_resource) {
    wined3d_not_from_cs((*device).cs);

    list_remove(core::ptr::addr_of_mut!((*resource).resource_list_entry));
}
#[no_mangle]
pub unsafe extern "C" fn device_resource_released(device: *mut wined3d_device, resource: *mut wined3d_resource) {
    let mut type_: wined3d_resource_type = (*resource).type_;
    let mut state: *mut wined3d_state = (*(*device).cs).c.state;
    let mut rtv: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut i: u32;

    i = 0;
    while (i as usize) < ARRAY_SIZE!((*state).fb.render_targets) as usize {
        rtv = (*state).fb.render_targets[i as usize];
        if !rtv.is_null() && (*rtv).resource == resource {
            ERR!("Resource %p is still in use as render target %u.\n", resource, i);
        }
        i = i.wrapping_add(1);
    }

    rtv = (*state).fb.depth_stencil;
    if !rtv.is_null() && (*rtv).resource == resource {
        ERR!("Resource %p is still in use as depth/stencil buffer.\n", resource);
    }

    match type_ as u32 {
        WINED3D_RTYPE_BUFFER => {
            i = 0;
            while i < WINED3D_MAX_STREAMS {
                if core::ptr::addr_of_mut!((*(*state).streams[i as usize].buffer).resource) == resource {
                    ERR!("Buffer resource %p is still in use, stream %u.\n", resource, i);
                    (*state).streams[i as usize].buffer = core::ptr::null_mut();
                }
                i = i.wrapping_add(1);
            }

            if core::ptr::addr_of_mut!((*(*state).index_buffer).resource) == resource {
                ERR!("Buffer resource %p is still in use as index buffer.\n", resource);
                (*state).index_buffer = core::ptr::null_mut();
            }
        }
        _ => {}
    }

    /* Remove the resource from the resourceStore */
    device_resource_remove(device, resource);
}

unsafe extern "C" fn wined3d_so_desc_compare(key: *const ::core::ffi::c_void, entry: *const rb_entry) -> ::core::ffi::c_int {
    let desc: *const wined3d_stream_output_desc = core::ptr::addr_of_mut!(
        (*((entry as *mut ::core::ffi::c_char).offset(0) as *mut wined3d_so_desc_entry)).desc
    );
    let k: *const wined3d_stream_output_desc = key as *const wined3d_stream_output_desc;
    let mut i: ::core::ffi::c_uint;
    let mut ret: ::core::ffi::c_int;

    ret = wined3d_uint32_compare((*k).element_count as uint32_t, (*desc).element_count as uint32_t);
    if ret != 0 {
        return ret;
    }
    ret = wined3d_uint32_compare((*k).buffer_stride_count as uint32_t, (*desc).buffer_stride_count as uint32_t);
    if ret != 0 {
        return ret;
    }
    ret = wined3d_uint32_compare((*k).rasterizer_stream_idx as uint32_t, (*desc).rasterizer_stream_idx as uint32_t);
    if ret != 0 {
        return ret;
    }

    i = 0;
    while i < (*k).element_count {
        let b: *const wined3d_stream_output_element =
            (*desc).elements.offset(i as isize) as *const wined3d_stream_output_element;
        let a: *const wined3d_stream_output_element =
            (*k).elements.offset(i as isize) as *const wined3d_stream_output_element;

        ret = wined3d_uint32_compare((*a).stream_idx as uint32_t, (*b).stream_idx as uint32_t);
        if ret != 0 {
            return ret;
        }
        ret = (*a).semantic_name.is_null() as ::core::ffi::c_int
            - (*b).semantic_name.is_null() as ::core::ffi::c_int;
        if ret != 0 {
            return ret;
        }
        if !(*a).semantic_name.is_null() {
            ret = strcmp((*a).semantic_name, (*b).semantic_name);
            if ret != 0 {
                return ret;
            }
        }
        ret = wined3d_uint32_compare((*a).semantic_idx as uint32_t, (*b).semantic_idx as uint32_t);
        if ret != 0 {
            return ret;
        }
        ret = wined3d_uint32_compare((*a).component_idx as uint32_t, (*b).component_idx as uint32_t);
        if ret != 0 {
            return ret;
        }
        ret = wined3d_uint32_compare((*a).component_count as uint32_t, (*b).component_count as uint32_t);
        if ret != 0 {
            return ret;
        }
        ret = wined3d_uint32_compare((*a).output_slot as uint32_t, (*b).output_slot as uint32_t);
        if ret != 0 {
            return ret;
        }
        i = i.wrapping_add(1);
    }

    i = 0;
    while i < (*k).buffer_stride_count {
        ret = wined3d_uint32_compare((*k).buffer_strides[i as usize], (*desc).buffer_strides[i as usize]);
        if ret != 0 {
            return ret;
        }
        i = i.wrapping_add(1);
    }

    return 0;
}

unsafe extern "C" fn wined3d_sampler_compare(key: *const core::ffi::c_void, entry: *const wine_rb_entry) -> core::ffi::c_int {
    let sampler: *const wined3d_sampler = (entry as *const u8).offset(-(core::mem::offset_of!(wined3d_sampler, entry) as isize)) as *const wined3d_sampler;
    return memcmp(core::ptr::addr_of!((*sampler).desc) as *const core::ffi::c_void, key, core::mem::size_of::<wined3d_sampler_desc>());
}

unsafe extern "C" fn wined3d_rasterizer_state_compare(key: *const c_void, entry: *const wine_rb_entry) -> c_int {
    let state: *const wined3d_rasterizer_state = WINE_RB_ENTRY_VALUE!(entry, wined3d_rasterizer_state, entry);

    return memcmp(&raw const (*state).desc as *const c_void, key, core::mem::size_of::<wined3d_rasterizer_state_desc>());
}

unsafe extern "C" fn wined3d_blend_state_compare(key: *const core::ffi::c_void, entry: *const wine_rb_entry) -> core::ffi::c_int {
    let state: *const wined3d_blend_state = (entry as *const u8).sub(core::mem::offset_of!(wined3d_blend_state, entry)) as *const wined3d_blend_state;
    return memcmp(core::ptr::addr_of!((*state).desc) as *const core::ffi::c_void, key, core::mem::size_of::<wined3d_blend_state_desc>()) as core::ffi::c_int;
}

unsafe extern "C" fn wined3d_depth_stencil_state_compare(key: *const c_void, entry: *const wine_rb_entry) -> c_int {
    let state: *const wined3d_depth_stencil_state = (entry as *mut u8).offset(-(88 as isize)) as *mut wined3d_depth_stencil_state;
    return memcmp(
        core::ptr::addr_of!((*state).desc) as *const c_void,
        key,
        core::mem::size_of::<wined3d_depth_stencil_state_desc>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_init(device: *mut wined3d_device, wined3d: *mut wined3d, adapter_idx: ::core::ffi::c_uint, device_type: wined3d_device_type, focus_window: HWND, flags: ::core::ffi::c_uint, surface_alignment: BYTE, levels: *const wined3d_feature_level, level_count: ::core::ffi::c_uint, supported_extensions: *const BOOL, device_parent: *mut wined3d_device_parent) -> HRESULT {
    let adapter: *mut wined3d_adapter = *(&raw mut (*wined3d).adapters as *mut *mut wined3d_adapter).offset(adapter_idx as isize);
    let mut fragment_pipeline: *const wined3d_fragment_pipe_ops = core::ptr::null();
    let mut vertex_pipeline: *const wined3d_vertex_pipe_ops = core::ptr::null();
    let mut i: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;

    (*device).ref_ = 1;
    (*device).wined3d = wined3d;
    wined3d_incref((*device).wined3d);
    (*device).adapter = adapter;
    (*device).device_parent = device_parent;
    list_init(&raw mut (*device).resources);
    list_init(&raw mut (*device).shaders);
    (*device).surface_alignment = surface_alignment;

    /* Save the creation parameters. */
    (*device).create_parms.adapter_idx = adapter_idx;
    (*device).create_parms.device_type = device_type;
    (*device).create_parms.focus_window = focus_window;
    (*device).create_parms.flags = flags;

    (*device).shader_backend = (*adapter).shader_backend;

    vertex_pipeline = (*adapter).vertex_pipe;

    fragment_pipeline = (*adapter).fragment_pipe;

    wine_rb_init(&raw mut (*device).so_descs, Some(wined3d_so_desc_compare));
    wine_rb_init(&raw mut (*device).samplers, Some(wined3d_sampler_compare));
    wine_rb_init(&raw mut (*device).rasterizer_states, Some(wined3d_rasterizer_state_compare));
    wine_rb_init(&raw mut (*device).blend_states, Some(wined3d_blend_state_compare));
    wine_rb_init(&raw mut (*device).depth_stencil_states, Some(wined3d_depth_stencil_state_compare));
    wine_rb_init(&raw mut (*device).ffp_vertex_shaders, Some(wined3d_ffp_vertex_program_key_compare));
    wine_rb_init(&raw mut (*device).ffp_pixel_shaders, Some(wined3d_ffp_frag_program_key_compare));

    if !(*vertex_pipeline).vp_states.is_null() && !(*fragment_pipeline).states.is_null() && {
        hr = compile_state_table(
            ((*device).state_table).as_mut_ptr(),
            ((*device).multistate_funcs).as_mut_ptr(),
            &raw mut (*adapter).d3d_info,
            supported_extensions,
            vertex_pipeline,
            fragment_pipeline,
            (*adapter).misc_state_template,
        );
        hr < 0
    } {
        ERR!("Failed to compile state table, hr %#lx.\n", hr);
        wine_rb_destroy(&raw mut (*device).samplers, None, core::ptr::null_mut());
        wine_rb_destroy(&raw mut (*device).rasterizer_states, None, core::ptr::null_mut());
        wine_rb_destroy(&raw mut (*device).blend_states, None, core::ptr::null_mut());
        wine_rb_destroy(&raw mut (*device).depth_stencil_states, None, core::ptr::null_mut());
        wine_rb_destroy(&raw mut (*device).so_descs, None, core::ptr::null_mut());
        wine_rb_destroy(&raw mut (*device).ffp_vertex_shaders, None, core::ptr::null_mut());
        wine_rb_destroy(&raw mut (*device).ffp_pixel_shaders, None, core::ptr::null_mut());
        wined3d_decref((*device).wined3d);
        return hr;
    }

    (*device).max_frame_latency = 3;

    'err: {
        (*device).cs = wined3d_cs_create(device, levels, level_count) as *mut wined3d_cs;
        if (*device).cs.is_null() {
            WARN!("Failed to create command stream.\n");
            hr = E_FAIL;
            break 'err;
        }

        wined3d_lock_init!(
            &raw mut (*device).bo_map_lock,
            b"wined3d_device.bo_map_lock\0".as_ptr() as *const ::core::ffi::c_char,
        );

        return WINED3D_OK;
    }

    // err:
    i = 0;
    while (i as usize) < ARRAY_SIZE!((*device).multistate_funcs) as usize {
        free(((*device).multistate_funcs[i as usize]).cast());
        i = i.wrapping_add(1);
    }
    wine_rb_destroy(&raw mut (*device).samplers, None, core::ptr::null_mut());
    wine_rb_destroy(&raw mut (*device).rasterizer_states, None, core::ptr::null_mut());
    wine_rb_destroy(&raw mut (*device).blend_states, None, core::ptr::null_mut());
    wine_rb_destroy(&raw mut (*device).depth_stencil_states, None, core::ptr::null_mut());
    wine_rb_destroy(&raw mut (*device).so_descs, None, core::ptr::null_mut());
    wine_rb_destroy(&raw mut (*device).ffp_vertex_shaders, None, core::ptr::null_mut());
    wine_rb_destroy(&raw mut (*device).ffp_pixel_shaders, None, core::ptr::null_mut());
    wined3d_decref((*device).wined3d);
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn device_invalidate_state(device: *const wined3d_device, state_id: ::core::ffi::c_uint) {
    let mut representative: ::core::ffi::c_uint = 0;
    let mut i: ::core::ffi::c_uint = 0;
    let mut idx: ::core::ffi::c_uint = 0;
    let mut shift: ::core::ffi::c_uint = 0;

    wined3d_from_cs((*device).cs);

    if STATE_IS_COMPUTE!(state_id) != false {
        i = 0 as ::core::ffi::c_uint;
        while i < (*device).context_count {
            context_invalidate_compute_state(*(*device).contexts.offset(i as isize), state_id);
            i = i.wrapping_add(1);
        }
        return;
    }

    representative = (*device).state_table[state_id as usize].representative;
    idx = representative
        / ((::core::mem::size_of::<uint32_t>() as ::core::ffi::c_uint)
            * (CHAR_BIT as ::core::ffi::c_uint));
    shift = representative
        & (((::core::mem::size_of::<uint32_t>() as ::core::ffi::c_uint)
            * (CHAR_BIT as ::core::ffi::c_uint))
            - (1 as ::core::ffi::c_uint));
    i = 0 as ::core::ffi::c_uint;
    while i < (*device).context_count {
        (**(*device).contexts.offset(i as isize)).dirty_graphics_states[idx as usize] |=
            (1 as uint32_t) << shift;
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn device_process_message(device: *mut wined3d_device, window: HWND, unicode: BOOL, message: UINT, wparam: WPARAM, lparam: LPARAM, proc_0: WNDPROC) -> LRESULT {
    if message == WM_DESTROY as UINT {
        TRACE!("unregister window %p.\n", window);
        wined3d_unregister_window(window);

        if InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*device).focus_window) as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
            window as *mut core::ffi::c_void,
        ) != window as *mut core::ffi::c_void
        {
            ERR!("Window %p is not the focus window for device %p.\n", window, device);
        }
    } else if message == WM_DISPLAYCHANGE as UINT {
        (*(*(*device).device_parent).ops)
            .mode_changed
            .unwrap_unchecked()((*device).device_parent);
    } else if message == WM_ACTIVATEAPP as UINT {
        let mut i: core::ffi::c_uint = (*device).swapchain_count as core::ffi::c_uint;
        loop {
            let fresh19 = i;
            i = i.wrapping_sub(1);
            if !(fresh19 != 0) {
                break;
            }
            wined3d_swapchain_activate(*(*device).swapchains.offset(i as isize), wparam as BOOL);
        }
    } else if message == WM_SYSCOMMAND as UINT {
        if wparam == SC_RESTORE as WPARAM
            && (*(*device).wined3d).flags & WINED3D_HANDLE_RESTORE as core::ffi::c_uint != 0
        {
            if unicode != 0 {
                DefWindowProcW(window, message, wparam, lparam);
            } else {
                DefWindowProcA(window, message, wparam, lparam);
            }
        }
    }

    if unicode != 0 {
        return CallWindowProcW(proc_0, window, message, wparam, lparam);
    } else {
        return CallWindowProcA(proc_0, window, message, wparam, lparam);
    }
}
