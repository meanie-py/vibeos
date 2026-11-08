use crate::d3dwine::context::context_invalidate_compute_state;
use crate::d3dwine::context::context_invalidate_state;
use crate::d3dwine::resource::vk_access_mask_from_bind_flags;
use crate::d3dwine::resource::vk_pipeline_stage_mask_from_bind_flags;
use crate::d3dwine::buffer::wined3d_buffer_acquire_bo_for_write;
use crate::d3dwine::buffer::wined3d_buffer_copy_bo_address;
use crate::d3dwine::buffer::wined3d_buffer_get_parent;
use crate::d3dwine::buffer::wined3d_buffer_invalidate_location;
use crate::d3dwine::buffer::wined3d_buffer_load_location;
use crate::d3dwine::buffer::wined3d_buffer_prepare_location;
use crate::d3dwine::buffer::wined3d_buffer_validate_location;
use crate::d3dwine::context_gl::wined3d_context_gl_active_texture;
use crate::d3dwine::context_gl::wined3d_context_gl_bind_bo;
use crate::d3dwine::context_gl::wined3d_context_gl_bind_texture;
use crate::d3dwine::context_vk::wined3d_context_vk_create_bo;
use crate::d3dwine::context_vk::wined3d_context_vk_create_vk_descriptor_set;
use crate::d3dwine::context_vk::wined3d_context_vk_destroy_bo;
use crate::d3dwine::context_vk::wined3d_context_vk_destroy_vk_buffer_view;
use crate::d3dwine::context_vk::wined3d_context_vk_destroy_vk_image_view;
use crate::d3dwine::context_vk::wined3d_context_vk_end_current_render_pass;
use crate::d3dwine::context_vk::wined3d_context_vk_get_command_buffer;
use crate::d3dwine::context_vk::wined3d_context_vk_get_pipeline_layout;
use crate::d3dwine::context_vk::wined3d_context_vk_image_barrier;
use crate::d3dwine::cs::wined3d_cs_init_object;
use crate::d3dwine::device::wined3d_device_gl_create_bo;
use crate::d3dwine::texture::wined3d_gl_texture_swizzle_from_color_fixup;
use crate::d3dwine::resource::wined3d_resource_get_sample_count;
use crate::d3dwine::sampler::wined3d_sampler_gl_bind;
use crate::d3dwine::texture::wined3d_texture_from_resource;
use crate::d3dwine::texture::wined3d_texture_gl_bind;
use crate::d3dwine::texture::wined3d_texture_gl_bind_and_dirtify;
use crate::d3dwine::texture::wined3d_texture_gl_prepare_texture;
use crate::d3dwine::texture::wined3d_texture_invalidate_location;
use crate::d3dwine::texture::wined3d_texture_load_location;
use crate::d3dwine::texture::wined3d_texture_prepare_location;
use crate::d3dwine::texture::wined3d_texture_validate_location;
use crate::d3dwine::texture::wined3d_texture_vk_get_default_image_info;
use crate::d3dwine::texture::wined3d_texture_vk_prepare_texture;
use crate::d3dwine::texture::wined3d_vk_swizzle_from_color_fixup;
use crate::prelude::*;
use core::ffi::c_void;


// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_52 {
    pub texture_target: GLenum,
    pub view_flags: ::core::ffi::c_uint,
    pub view_target: GLenum,
    pub extension: wined3d_gl_extension,
}


// WINE_DEFAULT_DEBUG_CHANNEL(d3d): debug channel name "d3d" (NUL-terminated)

pub const WINED3D_VIEW_CUBE_ARRAY: u32 = WINED3D_VIEW_TEXTURE_CUBE | WINED3D_VIEW_TEXTURE_ARRAY;

#[repr(C)]
pub struct wined3d_uav_clear_constants_vk {
    pub color: VkClearColorValue,
    pub offset: VkOffset2D,
    pub extent: VkExtent2D,
}

unsafe extern "C" fn is_stencil_view_format(format: *const wined3d_format) -> BOOL {
    return ((*format).id as ::core::ffi::c_uint == WINED3DFMT_X24_TYPELESS_G8_UINT as ::core::ffi::c_uint
        || (*format).id as ::core::ffi::c_uint == WINED3DFMT_X32_TYPELESS_G8X24_UINT as ::core::ffi::c_uint)
        as ::core::ffi::c_int;
}

unsafe extern "C" fn get_texture_view_target(
    mut gl_info: *const wined3d_gl_info,
    mut desc: *const wined3d_view_desc,
    mut texture_gl: *const wined3d_texture_gl,
) -> GLenum {
    static view_types: [C2RustUnnamed_52; 16] = [
        C2RustUnnamed_52 {
            texture_target: GL_TEXTURE_CUBE_MAP as GLenum,
            view_flags: 0 as ::core::ffi::c_uint,
            view_target: GL_TEXTURE_CUBE_MAP as GLenum,
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_52 {
            texture_target: GL_TEXTURE_3D as GLenum,
            view_flags: 0 as ::core::ffi::c_uint,
            view_target: GL_TEXTURE_3D as GLenum,
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_52 {
            texture_target: GL_TEXTURE_2D as GLenum,
            view_flags: 0 as ::core::ffi::c_uint,
            view_target: GL_TEXTURE_2D as GLenum,
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_52 {
            texture_target: GL_TEXTURE_2D as GLenum,
            view_flags: WINED3D_VIEW_TEXTURE_ARRAY as ::core::ffi::c_uint,
            view_target: GL_TEXTURE_2D_ARRAY as GLenum,
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_52 {
            texture_target: GL_TEXTURE_2D_ARRAY as GLenum,
            view_flags: 0 as ::core::ffi::c_uint,
            view_target: GL_TEXTURE_2D as GLenum,
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_52 {
            texture_target: GL_TEXTURE_2D_ARRAY as GLenum,
            view_flags: WINED3D_VIEW_TEXTURE_ARRAY as ::core::ffi::c_uint,
            view_target: GL_TEXTURE_2D_ARRAY as GLenum,
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_52 {
            texture_target: GL_TEXTURE_2D_ARRAY as GLenum,
            view_flags: WINED3D_VIEW_TEXTURE_CUBE as ::core::ffi::c_uint,
            view_target: GL_TEXTURE_CUBE_MAP as GLenum,
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_52 {
            texture_target: GL_TEXTURE_2D_ARRAY as GLenum,
            view_flags: WINED3D_VIEW_CUBE_ARRAY as ::core::ffi::c_uint,
            view_target: GL_TEXTURE_CUBE_MAP_ARRAY as GLenum,
            extension: ARB_TEXTURE_CUBE_MAP_ARRAY,
        },
        C2RustUnnamed_52 {
            texture_target: GL_TEXTURE_2D_MULTISAMPLE as GLenum,
            view_flags: 0 as ::core::ffi::c_uint,
            view_target: GL_TEXTURE_2D_MULTISAMPLE as GLenum,
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_52 {
            texture_target: GL_TEXTURE_2D_MULTISAMPLE as GLenum,
            view_flags: WINED3D_VIEW_TEXTURE_ARRAY as ::core::ffi::c_uint,
            view_target: GL_TEXTURE_2D_MULTISAMPLE_ARRAY as GLenum,
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_52 {
            texture_target: GL_TEXTURE_2D_MULTISAMPLE_ARRAY as GLenum,
            view_flags: 0 as ::core::ffi::c_uint,
            view_target: GL_TEXTURE_2D_MULTISAMPLE as GLenum,
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_52 {
            texture_target: GL_TEXTURE_2D_MULTISAMPLE_ARRAY as GLenum,
            view_flags: WINED3D_VIEW_TEXTURE_ARRAY as ::core::ffi::c_uint,
            view_target: GL_TEXTURE_2D_MULTISAMPLE_ARRAY as GLenum,
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_52 {
            texture_target: GL_TEXTURE_1D as GLenum,
            view_flags: 0 as ::core::ffi::c_uint,
            view_target: GL_TEXTURE_1D as GLenum,
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_52 {
            texture_target: GL_TEXTURE_1D as GLenum,
            view_flags: WINED3D_VIEW_TEXTURE_ARRAY as ::core::ffi::c_uint,
            view_target: GL_TEXTURE_1D_ARRAY as GLenum,
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_52 {
            texture_target: GL_TEXTURE_1D_ARRAY as GLenum,
            view_flags: 0 as ::core::ffi::c_uint,
            view_target: GL_TEXTURE_1D as GLenum,
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_52 {
            texture_target: GL_TEXTURE_1D_ARRAY as GLenum,
            view_flags: WINED3D_VIEW_TEXTURE_ARRAY as ::core::ffi::c_uint,
            view_target: GL_TEXTURE_1D_ARRAY as GLenum,
            extension: WINED3D_GL_EXT_NONE,
        },
    ];
    let mut flags: ::core::ffi::c_uint = (*desc).flags
        & (WINED3D_VIEW_BUFFER_RAW
            | WINED3D_VIEW_BUFFER_APPEND
            | WINED3D_VIEW_BUFFER_COUNTER
            | WINED3D_VIEW_TEXTURE_CUBE
            | WINED3D_VIEW_TEXTURE_ARRAY) as ::core::ffi::c_uint;
    let mut i: ::core::ffi::c_uint = 0;
    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(view_types) as usize {
        if !(view_types[i as usize].texture_target != (*texture_gl).target
            || view_types[i as usize].view_flags != flags)
        {
            if (*gl_info).supported[view_types[i as usize].extension as usize] != 0 {
                return view_types[i as usize].view_target;
            }
            WINE_FIXME!("Extension %#x not supported.\n", view_types[i as usize].extension as ::core::ffi::c_uint);
        }
        i = i.wrapping_add(1);
    }
    WINE_FIXME!("Unhandled view flags %#x for texture target %#x.\n", flags, (*texture_gl).target);
    return (*texture_gl).target;
}

unsafe extern "C" fn validate_resource_view(
    desc: *const wined3d_view_desc,
    resource: *mut wined3d_resource,
    mip_slice: BOOL,
    allow_srgb_toggle: BOOL,
) -> *const wined3d_format {
    let adapter: *const wined3d_adapter = (*(*resource).device).adapter;
    let mut format: *const wined3d_format;

    format = wined3d_get_format(adapter, (*desc).format_id, (*resource).bind_flags);
    if (*resource).type_ as ::core::ffi::c_uint == WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint
        && ((*desc).flags & (WINED3D_VIEW_BUFFER_RAW as ::core::ffi::c_uint)) != 0
    {
        if (*format).id as ::core::ffi::c_uint != WINED3DFMT_R32_TYPELESS as ::core::ffi::c_uint {
            WARN!("Invalid format %s for raw buffer view.\n", debug_d3dformat((*format).id));
            return core::ptr::null();
        }

        format = wined3d_get_format(adapter, WINED3DFMT_R32_UINT, (*resource).bind_flags);
    }

    if wined3d_format_is_typeless(format) != 0 {
        WARN!("Trying to create view for typeless format %s.\n", debug_d3dformat((*format).id));
        return core::ptr::null();
    }

    if (*resource).type_ as ::core::ffi::c_uint == WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint {
        let buffer: *mut wined3d_buffer = buffer_from_resource(resource);
        let mut buffer_size: ::core::ffi::c_uint;
        let mut element_size: ::core::ffi::c_uint;

        if (*buffer).structure_byte_stride != 0 {
            if (*desc).format_id as ::core::ffi::c_uint != WINED3DFMT_UNKNOWN as ::core::ffi::c_uint {
                WARN!("Invalid format %s for structured buffer view.\n", debug_d3dformat((*desc).format_id));
                return core::ptr::null();
            }

            format = wined3d_get_format(adapter, WINED3DFMT_R32_UINT, (*resource).bind_flags);
            element_size = (*buffer).structure_byte_stride;
        } else {
            element_size = (*format).byte_count as ::core::ffi::c_uint;
        }

        if element_size == 0 {
            return core::ptr::null();
        }

        buffer_size = ((*buffer).resource.size as ::core::ffi::c_uint).wrapping_div(element_size);
        if !wined3d_bound_range(
            (*desc).u.buffer.start_idx,
            (*desc).u.buffer.count,
            buffer_size,
        ) {
            return core::ptr::null();
        }
    } else {
        let texture: *mut wined3d_texture = texture_from_resource(resource);
        let mut depth_or_layer_count: ::core::ffi::c_uint;

        if (*(*resource).format).id as ::core::ffi::c_uint != (*format).id as ::core::ffi::c_uint
            && wined3d_format_is_typeless((*resource).format) == 0
            && (allow_srgb_toggle == 0
                || wined3d_formats_are_srgb_variants((*(*resource).format).id, (*format).id) == 0)
        {
            WARN!("Trying to create incompatible view for non typeless format %s.\n", debug_d3dformat((*format).id));
            return core::ptr::null();
        }

        if mip_slice != 0
            && (*resource).type_ as ::core::ffi::c_uint == WINED3D_RTYPE_TEXTURE_3D as ::core::ffi::c_uint
        {
            depth_or_layer_count = wined3d_texture_get_level_depth(texture, (*desc).u.texture.level_idx);
        } else {
            depth_or_layer_count = (*texture).layer_count as ::core::ffi::c_uint;
        }

        if (*desc).u.texture.level_count == 0
            || (mip_slice != 0 && (*desc).u.texture.level_count != 1 as ::core::ffi::c_uint)
            || !wined3d_bound_range(
                (*desc).u.texture.level_idx,
                (*desc).u.texture.level_count,
                (*texture).level_count,
            )
            || (*desc).u.texture.layer_count == 0
            || !wined3d_bound_range(
                (*desc).u.texture.layer_idx,
                (*desc).u.texture.layer_count,
                depth_or_layer_count,
            )
        {
            return core::ptr::null();
        }
    }

    return format;
}

unsafe extern "C" fn create_texture_view(
    view: *mut wined3d_gl_view,
    view_target: GLenum,
    desc: *const wined3d_view_desc,
    texture_gl: *mut wined3d_texture_gl,
    view_format: *const wined3d_format,
) {
    let mut view_format_gl: *const wined3d_format_gl;
    let mut level_idx: core::ffi::c_uint;
    let mut layer_idx: core::ffi::c_uint;
    let mut layer_count: core::ffi::c_uint;
    let mut gl_info: *const wined3d_gl_info;
    let mut context_gl: *mut wined3d_context_gl;
    let mut context: *mut wined3d_context;
    let mut texture_name: GLuint;

    view_format_gl = wined3d_format_gl(view_format);
    (*view).target = view_target;

    context = context_acquire((*texture_gl).t.resource.device, core::ptr::null_mut(), 0);
    context_gl = wined3d_context_gl(context);
    gl_info = (*context_gl).gl_info;

    if (*gl_info).supported[ARB_TEXTURE_VIEW as usize] == 0
    {
        context_release(context);

        return;
    }

    wined3d_texture_gl_prepare_texture(texture_gl, context_gl, (FALSE) as BOOL);
    texture_name = wined3d_texture_gl_get_texture_name(texture_gl, context, ((FALSE) != 0));

    level_idx = (*desc).u.texture.level_idx;
    layer_idx = (*desc).u.texture.layer_idx;
    layer_count = (*desc).u.texture.layer_count;
    if view_target == GL_TEXTURE_3D as GLenum
    {
        if layer_idx != 0 || layer_count != wined3d_texture_get_level_depth(core::ptr::addr_of_mut!((*texture_gl).t), level_idx)
        {
            FIXME!("Depth slice (%u-%u) not supported.\n", layer_idx, layer_count);
        }
        layer_idx = 0;
        layer_count = 1;
    }

    ((*gl_info).gl_ops.gl.p_glGenTextures.unwrap_unchecked())(
        1,
        core::ptr::addr_of_mut!((*view).name),
    );
    ((*gl_info).gl_ops.ext.p_glTextureView.unwrap_unchecked())(
        (*view).name,
        (*view).target,
        texture_name,
        (*view_format_gl).internal as GLenum,
        level_idx as GLuint,
        (*desc).u.texture.level_count as GLuint,
        layer_idx as GLuint,
        layer_count as GLuint,
    );
    checkGLcall!(gl_info, "create texture view");

    if is_stencil_view_format(view_format) != 0
    {
        static swizzle: [GLint; 4] = [(GL_ZERO as i32), (GL_RED as i32), (GL_ZERO as i32), (GL_ZERO as i32)];

        if (*gl_info).supported[ARB_STENCIL_TEXTURING as usize] == 0
        {
            context_release(context);

            return;
        }

        wined3d_context_gl_bind_texture(context_gl, (*view).target, (*view).name);
        ((*gl_info).gl_ops.gl.p_glTexParameteriv.unwrap_unchecked())(
            (*view).target,
            GL_TEXTURE_SWIZZLE_RGBA as GLenum,
            core::ptr::addr_of!(swizzle) as *const GLint,
        );
        ((*gl_info).gl_ops.gl.p_glTexParameteri.unwrap_unchecked())(
            (*view).target,
            GL_DEPTH_STENCIL_TEXTURE_MODE as GLenum,
            GL_STENCIL_INDEX as GLint,
        );
        checkGLcall!(gl_info, "initialize stencil view");

        context_invalidate_compute_state(context, STATE_COMPUTE_SHADER_RESOURCE_BINDING as DWORD);
        context_invalidate_state(context, STATE_GRAPHICS_SHADER_RESOURCE_BINDING as core::ffi::c_uint);
    }
    else if is_identity_fixup((*view_format).color_fixup) == 0 && can_use_texture_swizzle((*context).d3d_info, view_format) != 0
    {
        let mut swizzle_0: [GLint; 4] = [0; 4];

        wined3d_context_gl_bind_texture(context_gl, (*view).target, (*view).name);
        wined3d_gl_texture_swizzle_from_color_fixup(core::ptr::addr_of_mut!(swizzle_0) as *mut GLint, (*view_format).color_fixup);
        ((*gl_info).gl_ops.gl.p_glTexParameteriv.unwrap_unchecked())(
            (*view).target,
            GL_TEXTURE_SWIZZLE_RGBA as GLenum,
            core::ptr::addr_of_mut!(swizzle_0) as *mut GLint,
        );
        checkGLcall!(gl_info, "set format swizzle");

        context_invalidate_compute_state(context, STATE_COMPUTE_SHADER_RESOURCE_BINDING as DWORD);
        context_invalidate_state(context, STATE_GRAPHICS_SHADER_RESOURCE_BINDING as core::ffi::c_uint);
    }

    context_release(context);
}

unsafe extern "C" fn create_buffer_texture(
    view: *mut wined3d_gl_view,
    context_gl: *mut wined3d_context_gl,
    buffer: *mut wined3d_buffer,
    view_format_gl: *const wined3d_format_gl,
    mut offset: u32,
    size: u32,
) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let bo_gl: *const wined3d_bo_gl;

    if (*gl_info).supported[ARB_TEXTURE_BUFFER_OBJECT as usize] == 0 {
        FIXME!("OpenGL implementation does not support buffer textures.\n");
        return;
    }

    wined3d_buffer_load_location(buffer, core::ptr::addr_of_mut!((*context_gl).c), WINED3D_LOCATION_BUFFER);
    bo_gl = wined3d_bo_gl((*buffer).buffer_object);
    offset = (offset as usize).wrapping_add((*bo_gl).b.buffer_offset) as u32;

    if offset & (*gl_info).limits.texture_buffer_offset_alignment.wrapping_sub(1 as u32) != 0 {
        FIXME!("Buffer offset %u is not %u byte aligned.\n", offset, (*gl_info).limits.texture_buffer_offset_alignment);
        return;
    }

    (*view).target = GL_TEXTURE_BUFFER;
    if (*view).name == 0 {
        (*gl_info).gl_ops.gl.p_glGenTextures.unwrap_unchecked()(
            1,
            core::ptr::addr_of_mut!((*view).name),
        );
    }

    wined3d_context_gl_bind_texture(context_gl, GL_TEXTURE_BUFFER, (*view).name);
    if (*gl_info).supported[ARB_TEXTURE_BUFFER_RANGE as usize] != 0 {
        ((*gl_info).gl_ops.ext.p_glTexBufferRange.unwrap_unchecked())(
            GL_TEXTURE_BUFFER,
            (*view_format_gl).internal as u32,
            (*bo_gl).id,
            offset as isize as _,
            size as isize as _,
        );
    } else {
        if offset != 0 || size != (*buffer).resource.size {
            FIXME!("OpenGL implementation does not support ARB_texture_buffer_range.\n");
        }
        ((*gl_info).gl_ops.ext.p_glTexBuffer.unwrap_unchecked())(
            GL_TEXTURE_BUFFER,
            ((*view_format_gl).internal) as u32,
            (*bo_gl).id,
        );
    }
    checkGLcall!(gl_info, "Create buffer texture");

    context_invalidate_compute_state(core::ptr::addr_of_mut!((*context_gl).c), (STATE_COMPUTE_SHADER_RESOURCE_BINDING) as DWORD);
    context_invalidate_state(core::ptr::addr_of_mut!((*context_gl).c), (STATE_GRAPHICS_SHADER_RESOURCE_BINDING as u32));
}

unsafe extern "C" fn get_buffer_view_range(
    buffer: *const wined3d_buffer,
    desc: *const wined3d_view_desc,
    view_format: *const wined3d_format,
    offset: *mut ::core::ffi::c_uint,
    size: *mut ::core::ffi::c_uint,
) {
    if (*desc).format_id as ::core::ffi::c_uint == WINED3DFMT_UNKNOWN as ::core::ffi::c_uint
    {
        *offset = (*desc)
            .u
            .buffer
            .start_idx
            .wrapping_mul((*buffer).structure_byte_stride);
        *size = (*desc)
            .u
            .buffer
            .count
            .wrapping_mul((*buffer).structure_byte_stride);
    } else {
        *offset = ((*desc).u.buffer.start_idx as UINT).wrapping_mul((*view_format).byte_count)
            as ::core::ffi::c_uint;
        *size = ((*desc).u.buffer.count as UINT).wrapping_mul((*view_format).byte_count)
            as ::core::ffi::c_uint;
    }
}

unsafe extern "C" fn create_buffer_view(
    view: *mut wined3d_gl_view,
    context: *mut wined3d_context,
    desc: *const wined3d_view_desc,
    buffer: *mut wined3d_buffer,
    view_format: *const wined3d_format,
) {
    let mut offset: u32 = 0;
    let mut size: u32 = 0;

    get_buffer_view_range(
        buffer,
        desc,
        view_format,
        core::ptr::addr_of_mut!(offset),
        core::ptr::addr_of_mut!(size),
    );
    create_buffer_texture(
        view,
        wined3d_context_gl(context),
        buffer,
        wined3d_format_gl(view_format),
        offset,
        size,
    );
}

unsafe extern "C" fn wined3d_view_invalidate_location(
    mut resource: *mut wined3d_resource,
    mut desc: *const wined3d_view_desc,
    mut location: DWORD,
) {
    let mut i: ::core::ffi::c_uint = 0;
    let mut sub_resource_idx: ::core::ffi::c_uint = 0;
    let mut texture: *mut wined3d_texture = ::core::ptr::null_mut::<wined3d_texture>();
    if (*resource).type_ as ::core::ffi::c_uint == WINED3D_RTYPE_BUFFER {
        wined3d_buffer_invalidate_location(buffer_from_resource(resource), location as uint32_t);
        return;
    }
    texture = texture_from_resource(resource);
    if (*resource).type_ as ::core::ffi::c_uint == WINED3D_RTYPE_TEXTURE_3D {
        wined3d_texture_invalidate_location(texture, (*desc).u.texture.level_idx, location as uint32_t);
        return;
    }
    sub_resource_idx = (*desc)
        .u
        .texture
        .layer_idx
        .wrapping_mul((*texture).level_count)
        .wrapping_add((*desc).u.texture.level_idx);
    i = 0;
    while i < (*desc).u.texture.layer_count {
        wined3d_texture_invalidate_location(texture, sub_resource_idx, location as uint32_t);
        i = i.wrapping_add(1);
        sub_resource_idx = sub_resource_idx.wrapping_add((*texture).level_count);
    }
}

unsafe extern "C" fn wined3d_view_load_location(
    mut resource: *mut wined3d_resource,
    mut desc: *const wined3d_view_desc,
    mut context: *mut wined3d_context,
    mut location: DWORD,
) {
    let mut i: ::core::ffi::c_uint = 0;
    let mut sub_resource_idx: ::core::ffi::c_uint = 0;
    let mut texture: *mut wined3d_texture = ::core::ptr::null_mut::<wined3d_texture>();
    if (*resource).type_ as ::core::ffi::c_uint == WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint {
        wined3d_buffer_load_location(buffer_from_resource(resource), context, location);
        return;
    }
    texture = texture_from_resource(resource);
    if (*resource).type_ as ::core::ffi::c_uint == WINED3D_RTYPE_TEXTURE_3D as ::core::ffi::c_uint {
        wined3d_texture_load_location(texture, (*desc).u.texture.level_idx, context, location);
        return;
    }
    sub_resource_idx = (*desc)
        .u
        .texture
        .layer_idx
        .wrapping_mul((*texture).level_count)
        .wrapping_add((*desc).u.texture.level_idx);
    i = 0 as ::core::ffi::c_uint;
    while i < (*desc).u.texture.layer_count {
        wined3d_texture_load_location(texture, sub_resource_idx, context, location);
        i = i.wrapping_add(1);
        sub_resource_idx = sub_resource_idx.wrapping_add((*texture).level_count);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_rendertarget_view_incref(
    mut view: *mut wined3d_rendertarget_view,
) -> ULONG {
    let mut refcount: ::core::ffi::c_uint = InterlockedIncrement(
        &raw mut (*view).refcount as *mut ::core::ffi::c_long as *mut ::core::ffi::c_long,
    ) as ::core::ffi::c_uint;
    WINE_TRACE!("%p increasing refcount to %u.\n", view, refcount);
    return refcount as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_rendertarget_view_cleanup(view: *mut wined3d_rendertarget_view) {
    (((*(*view).parent_ops).wined3d_object_destroyed).unwrap_unchecked())((*view).parent);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_rendertarget_view_decref(
    mut view: *mut wined3d_rendertarget_view,
) -> ULONG {
    let mut refcount: c_uint = InterlockedDecrement(
        core::ptr::addr_of_mut!((*view).refcount) as *mut c_long,
    ) as c_uint;
    WINE_TRACE!("%p decreasing refcount to %u.\n", view, refcount);
    if refcount == 0 {
        let mut resource: *mut wined3d_resource = (*view).resource;
        wined3d_mutex_lock();
        (*(*(*(*resource).device).adapter).adapter_ops)
            .adapter_destroy_rendertarget_view
            .unwrap_unchecked()(view);
        wined3d_mutex_unlock();
        wined3d_resource_decref(resource);
    }
    return refcount as ULONG;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_rendertarget_view_get_parent(
    view: *const wined3d_rendertarget_view,
) -> *mut core::ffi::c_void {
    // TRACE("view %p.\n", view);
    (*view).parent
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_rendertarget_view_get_sub_resource_parent(
    view: *const wined3d_rendertarget_view,
) -> *mut core::ffi::c_void {
    let mut texture: *mut wined3d_texture = core::ptr::null_mut::<wined3d_texture>();
    WINE_TRACE!("view %p.\n", view);
    if (*(*view).resource).type_ == WINED3D_RTYPE_BUFFER {
        return wined3d_buffer_get_parent(buffer_from_resource((*view).resource));
    }
    texture = texture_from_resource((*view).resource);
    return (*(*texture).sub_resources.offset((*view).sub_resource_idx as isize)).parent;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_rendertarget_view_set_parent(
    mut view: *mut wined3d_rendertarget_view,
    mut parent: *mut ::core::ffi::c_void,
    mut parent_ops: *const wined3d_parent_ops,
) {
    WINE_TRACE!("view %p, parent %p, parent_ops %p.\n", view, parent, parent_ops);
    (*view).parent = parent;
    (*view).parent_ops = parent_ops;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_rendertarget_view_get_resource(
    view: *const wined3d_rendertarget_view,
) -> *mut wined3d_resource {
    // TRACE("view %p.\n", view);
    (*view).resource
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_rendertarget_view_get_drawable_size(
    view: *const wined3d_rendertarget_view,
    context: *const wined3d_context,
    width: *mut ::core::ffi::c_uint,
    height: *mut ::core::ffi::c_uint,
) {
    let mut texture: *const wined3d_texture = ::core::ptr::null::<wined3d_texture>();
    if (*(*view).resource).type_ as ::core::ffi::c_uint != WINED3D_RTYPE_TEXTURE_2D as ::core::ffi::c_uint {
        *width = (*view).width;
        *height = (*view).height;
        return;
    }
    texture = texture_from_resource((*view).resource);
    if !(*texture).swapchain.is_null() {
        *width = (*texture).resource.width as ::core::ffi::c_uint;
        *height = (*texture).resource.height as ::core::ffi::c_uint;
    } else {
        let level_idx: ::core::ffi::c_uint = (*view)
            .sub_resource_idx
            .wrapping_rem((*texture).level_count);
        *width = wined3d_texture_get_level_width(texture, level_idx);
        *height = wined3d_texture_get_level_height(texture, level_idx);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_rendertarget_view_prepare_location(
    view: *mut wined3d_rendertarget_view,
    context: *mut wined3d_context,
    location: u32,
) {
    let resource: *mut wined3d_resource = (*view).resource;
    let mut i: core::ffi::c_uint;
    let mut sub_resource_idx: core::ffi::c_uint;
    let mut layer_count: core::ffi::c_uint;
    let mut texture: *mut wined3d_texture;

    if (*resource).type_ as core::ffi::c_uint == WINED3D_RTYPE_BUFFER as core::ffi::c_uint {
        FIXME!(
            "Not implemented for resources %s.\n",
            debug_d3dresourcetype((*resource).type_)
        );
        return;
    }

    texture = texture_from_resource(resource);
    sub_resource_idx = (*view).sub_resource_idx;
    layer_count = if (*resource).type_ as core::ffi::c_uint != WINED3D_RTYPE_TEXTURE_3D as core::ffi::c_uint {
        (*view).layer_count
    } else {
        1 as core::ffi::c_uint
    };
    i = 0 as core::ffi::c_uint;
    while i < layer_count {
        wined3d_texture_prepare_location(texture, sub_resource_idx, context, location);
        i = i.wrapping_add(1);
        sub_resource_idx = sub_resource_idx.wrapping_add((*texture).level_count);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_rendertarget_view_load_location(
    view: *mut wined3d_rendertarget_view,
    context: *mut wined3d_context,
    location: uint32_t,
) {
    wined3d_view_load_location(
        (*view).resource,
        core::ptr::addr_of_mut!((*view).desc),
        context,
        location as DWORD,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_rendertarget_view_validate_location(
    mut view: *mut wined3d_rendertarget_view,
    mut location: uint32_t,
) {
    let mut resource: *mut wined3d_resource = (*view).resource;
    let mut i: ::core::ffi::c_uint = 0;
    let mut sub_resource_idx: ::core::ffi::c_uint = 0;
    let mut layer_count: ::core::ffi::c_uint = 0;
    let mut texture: *mut wined3d_texture = ::core::ptr::null_mut::<wined3d_texture>();
    if (*resource).type_ as ::core::ffi::c_uint == WINED3D_RTYPE_BUFFER {
        return;
    }
    texture = texture_from_resource(resource);
    sub_resource_idx = (*view).sub_resource_idx;
    layer_count = if (*resource).type_ as ::core::ffi::c_uint != WINED3D_RTYPE_TEXTURE_3D {
        (*view).layer_count
    } else {
        1
    };
    i = 0;
    while i < layer_count {
        wined3d_texture_validate_location(texture, sub_resource_idx, location);
        i = i.wrapping_add(1);
        sub_resource_idx = sub_resource_idx.wrapping_add((*texture).level_count);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_rendertarget_view_invalidate_location(view: *mut wined3d_rendertarget_view, location: u32) {
    wined3d_view_invalidate_location((*view).resource, core::ptr::addr_of_mut!((*view).desc), location);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_rendertarget_view_get_locations(
    view: *const wined3d_rendertarget_view,
) -> DWORD {
    let mut resource: *mut wined3d_resource = (*view).resource;
    let mut i: ::core::ffi::c_uint = 0;
    let mut sub_resource_idx: ::core::ffi::c_uint = 0;
    let mut layer_count: ::core::ffi::c_uint = 0;
    let mut texture: *const wined3d_texture = core::ptr::null();
    let mut ret: DWORD = !0;

    if (*resource).type_ as ::core::ffi::c_uint == WINED3D_RTYPE_BUFFER {
        return (*buffer_from_resource(resource)).locations;
    }

    texture = texture_from_resource(resource);
    sub_resource_idx = (*view).sub_resource_idx;
    layer_count = if (*resource).type_ as ::core::ffi::c_uint != WINED3D_RTYPE_TEXTURE_3D {
        (*view).layer_count
    } else {
        1
    };
    i = 0;
    while i < layer_count {
        ret &= (*(*texture).sub_resources.offset(sub_resource_idx as isize)).locations;
        i = i.wrapping_add(1);
        sub_resource_idx = sub_resource_idx.wrapping_add((*texture).level_count);
    }

    if ret == 0 {
        WARN!(
            "View %p (texture %p) layers do not have a location in common.\n",
            view,
            texture
        );
    }

    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_rendertarget_view_get_box(view: *mut wined3d_rendertarget_view, box_0: *mut wined3d_box) {
    if (*(*view).resource).type_ as ::core::ffi::c_uint != WINED3D_RTYPE_TEXTURE_3D {
        wined3d_box_set(
            box_0,
            0,
            0,
            (*view).width,
            (*view).height,
            0,
            1,
        );
    } else {
        let texture: *mut wined3d_texture = texture_from_resource((*view).resource);
        wined3d_texture_get_level_box(texture, (*view).sub_resource_idx, box_0);
    }
}

unsafe extern "C" fn wined3d_render_target_view_gl_cs_init(mut object: *mut ::core::ffi::c_void) {
    let mut view_gl: *mut wined3d_rendertarget_view_gl = object as *mut wined3d_rendertarget_view_gl;
    let mut resource: *mut wined3d_resource = (*view_gl).v.resource;
    let mut desc: *const wined3d_view_desc = &raw const (*view_gl).v.desc;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        TRACE!("view_gl %p.\n", view_gl);
    }
    if (*resource).type_ as ::core::ffi::c_uint == WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
            != 0
        {
            FIXME!(
                "Not implemented for resources %s.\n",
                debug_d3dresourcetype((*resource).type_),
            );
        }
    } else {
        let mut texture_gl: *mut wined3d_texture_gl =
            wined3d_texture_gl(texture_from_resource(resource));
        let mut depth_or_layer_count: ::core::ffi::c_uint = 0;
        if (*resource).type_ as ::core::ffi::c_uint == WINED3D_RTYPE_TEXTURE_3D as ::core::ffi::c_uint {
            depth_or_layer_count = wined3d_texture_get_level_depth(
                &raw mut (*texture_gl).t,
                (*desc).u.texture.level_idx,
            );
        } else {
            depth_or_layer_count = (*texture_gl).t.layer_count as ::core::ffi::c_uint;
        }
        if (*(*resource).format).id as ::core::ffi::c_uint
            != (*(*view_gl).v.format).id as ::core::ffi::c_uint
            || (*view_gl).v.layer_count != 1 as ::core::ffi::c_uint
                && (*view_gl).v.layer_count != depth_or_layer_count
        {
            let mut resource_class: GLenum = 0;
            let mut view_class: GLenum = 0;
            resource_class = (*wined3d_format_gl((*resource).format)).view_class;
            view_class = (*wined3d_format_gl((*view_gl).v.format)).view_class;
            if resource_class != view_class {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                    != 0
                {
                    FIXME!(
                        "Render target view not supported, resource format %s, view format %s.\n",
                        debug_d3dformat((*(*resource).format).id),
                        debug_d3dformat((*(*view_gl).v.format).id),
                    );
                }
                return;
            }
            if !(*texture_gl).t.swapchain.is_null()
                && (*(*texture_gl).t.swapchain).state.desc.backbuffer_count
                    > 1 as ::core::ffi::c_uint
            {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                    != 0
                {
                    FIXME!("Swapchain views not supported.\n");
                }
                return;
            }
            create_texture_view(
                &raw mut (*view_gl).gl_view,
                (*texture_gl).target,
                desc,
                texture_gl,
                (*view_gl).v.format,
            );
        }
    }
}

unsafe extern "C" fn wined3d_rendertarget_view_init(
    view: *mut wined3d_rendertarget_view,
    desc: *const wined3d_view_desc,
    resource: *mut wined3d_resource,
    parent: *mut ::core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
) -> HRESULT {
    let mut allow_srgb_toggle: BOOL = (FALSE as i32);

    (*view).refcount = 1;
    (*view).parent = parent;
    (*view).parent_ops = parent_ops;

    if (*resource).type_ as ::core::ffi::c_uint != WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint {
        let texture: *mut wined3d_texture = texture_from_resource(resource);

        if !(*texture).swapchain.is_null() {
            allow_srgb_toggle = (TRUE as i32);
        }
    }
    (*view).format = validate_resource_view(desc, resource, (TRUE) as BOOL, allow_srgb_toggle);
    if (*view).format.is_null() {
        return E_INVALIDARG;
    }
    (*view).format_attrs = (*(*view).format).attrs;
    (*view).format_caps = (*(*view).format).caps[(*resource).gl_type as usize];
    (*view).desc = *desc;

    if (*resource).type_ as ::core::ffi::c_uint == WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint {
        (*view).sub_resource_idx = 0;
        (*view).layer_count = 1;
        (*view).width = (*desc).u.buffer.count;
        (*view).height = 1;
    } else {
        let texture_0: *mut wined3d_texture = texture_from_resource(resource);

        (*view).sub_resource_idx = (*desc).u.texture.level_idx;
        if (*resource).type_ as ::core::ffi::c_uint != WINED3D_RTYPE_TEXTURE_3D as ::core::ffi::c_uint {
            (*view).sub_resource_idx = (*view).sub_resource_idx.wrapping_add(
                (*desc).u.texture.layer_idx.wrapping_mul((*texture_0).level_count),
            );
        }
        (*view).layer_count = (*desc).u.texture.layer_count;
        (*view).width = wined3d_texture_get_level_width(texture_0, (*desc).u.texture.level_idx);
        (*view).height = wined3d_texture_get_level_height(texture_0, (*desc).u.texture.level_idx);
    }

    (*view).resource = resource;
    wined3d_resource_incref((*view).resource);

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_rendertarget_view_no3d_init(mut view_no3d: *mut wined3d_rendertarget_view, mut desc: *const wined3d_view_desc, mut resource: *mut wined3d_resource, mut parent: *mut ::core::ffi::c_void, mut parent_ops: *const wined3d_parent_ops) -> HRESULT {
    WINE_TRACE!("view_no3d %p, desc %s, resource %p, parent %p, parent_ops %p.\n", view_no3d, wined3d_debug_view_desc(desc, resource), resource, parent, parent_ops);
    return wined3d_rendertarget_view_init(view_no3d, desc, resource, parent, parent_ops);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_rendertarget_view_gl_init(view_gl: *mut wined3d_rendertarget_view_gl, desc: *const wined3d_view_desc, resource: *mut wined3d_resource, parent: *mut ::core::ffi::c_void, parent_ops: *const wined3d_parent_ops) -> HRESULT {
    let mut hr: HRESULT;

    hr = wined3d_rendertarget_view_init(
        core::ptr::addr_of_mut!((*view_gl).v),
        desc,
        resource,
        parent,
        parent_ops,
    );
    if hr < 0 {
        return hr;
    }

    wined3d_cs_init_object(
        (*(*resource).device).cs as *mut wined3d_cs,
        Some(
            wined3d_render_target_view_gl_cs_init
                as unsafe extern "C" fn(*mut core::ffi::c_void),
        ),
        view_gl as *mut core::ffi::c_void,
    );

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn vk_image_view_type_from_wined3d(type_0: wined3d_resource_type, flags: uint32_t) -> VkImageViewType {
    match type_0 as u32 {
        WINED3D_RTYPE_TEXTURE_1D => {
            if (flags & WINED3D_VIEW_TEXTURE_ARRAY) != 0 {
                return VK_IMAGE_VIEW_TYPE_1D_ARRAY;
            } else {
                return VK_IMAGE_VIEW_TYPE_1D;
            }
        }
        WINED3D_RTYPE_TEXTURE_2D => {
            if (flags & WINED3D_VIEW_TEXTURE_CUBE) != 0 {
                if (flags & WINED3D_VIEW_TEXTURE_ARRAY) != 0 {
                    return VK_IMAGE_VIEW_TYPE_CUBE_ARRAY;
                } else {
                    return VK_IMAGE_VIEW_TYPE_CUBE;
                }
            }
            if (flags & WINED3D_VIEW_TEXTURE_ARRAY) != 0 {
                return VK_IMAGE_VIEW_TYPE_2D_ARRAY;
            } else {
                return VK_IMAGE_VIEW_TYPE_2D;
            }
        }
        WINED3D_RTYPE_TEXTURE_3D => {
            return VK_IMAGE_VIEW_TYPE_3D;
        }
        _ => {
            ERR!("Unhandled resource type %s.\n", debug_d3dresourcetype(type_0));
            return !(0 as ::core::ffi::c_uint) as VkImageViewType;
        }
    };
}

unsafe extern "C" fn wined3d_view_vk_create_vk_buffer_view(
    context_vk: *mut wined3d_context_vk,
    desc: *const wined3d_view_desc,
    buffer_vk: *mut wined3d_buffer_vk,
    view_format_vk: *const wined3d_format_vk,
) -> VkBufferView {
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut create_info: VkBufferViewCreateInfo = core::mem::zeroed();
    let mut device_vk: *mut wined3d_device_vk;
    let mut vk_buffer_view: VkBufferView = core::mem::zeroed();
    let mut offset: core::ffi::c_uint = core::mem::zeroed();
    let mut size: core::ffi::c_uint = core::mem::zeroed();
    let mut bo: *mut wined3d_bo_vk;
    let mut vr: VkResult;

    get_buffer_view_range(
        core::ptr::addr_of_mut!((*buffer_vk).b),
        desc,
        core::ptr::addr_of!((*view_format_vk).f),
        core::ptr::addr_of_mut!(offset),
        core::ptr::addr_of_mut!(size),
    );
    wined3d_buffer_prepare_location(
        core::ptr::addr_of_mut!((*buffer_vk).b),
        core::ptr::addr_of_mut!((*context_vk).c),
        WINED3D_LOCATION_BUFFER as core::ffi::c_uint,
    );
    bo = wined3d_bo_vk((*buffer_vk).b.buffer_object);

    create_info.sType = VK_STRUCTURE_TYPE_BUFFER_VIEW_CREATE_INFO;
    create_info.pNext = core::ptr::null::<core::ffi::c_void>();
    create_info.flags = 0 as VkBufferViewCreateFlags;
    create_info.buffer = (*bo).vk_buffer;
    create_info.format = (*view_format_vk).vk_format;
    create_info.offset = (*bo).b.buffer_offset.wrapping_add(offset as usize) as VkDeviceSize;
    create_info.range = size as VkDeviceSize;

    device_vk = wined3d_device_vk((*buffer_vk).b.resource.device);
    vr = ((*vk_info).vk_ops.vkCreateBufferView.unwrap_unchecked())(
        (*device_vk).vk_device,
        core::ptr::addr_of_mut!(create_info),
        core::ptr::null::<VkAllocationCallbacks>(),
        core::ptr::addr_of_mut!(vk_buffer_view),
    );
    if (vr as core::ffi::c_int) < 0 {
        ERR!(
            "Failed to create buffer view, vr %s.\n",
            wined3d_debug_vkresult(vr)
        );
        return VK_NULL_HANDLE as VkBufferView;
    }

    return vk_buffer_view;
}

unsafe extern "C" fn wined3d_view_vk_create_vk_image_view(
    context_vk: *mut wined3d_context_vk,
    desc: *const wined3d_view_desc,
    texture_vk: *mut wined3d_texture_vk,
    view_format_vk: *const wined3d_format_vk,
    fixup: color_fixup_desc,
    rtv: bool,
    usage: VkImageUsageFlags,
) -> VkImageView {
    let resource: *const wined3d_resource = &raw mut (*texture_vk).t.resource;
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut format_vk: *const wined3d_format_vk = core::ptr::null::<wined3d_format_vk>();
    let mut usage_info: VkImageViewUsageCreateInfoKHR = core::mem::zeroed();
    let mut device_vk: *mut wined3d_device_vk = core::ptr::null_mut::<wined3d_device_vk>();
    let mut create_info: VkImageViewCreateInfo = core::mem::zeroed();
    let mut vk_image_view: VkImageView = core::mem::zeroed();
    let mut vr: VkResult = core::mem::zeroed();

    device_vk = wined3d_device_vk((*resource).device);

    if wined3d_texture_vk_prepare_texture(texture_vk, context_vk) == 0 {
        ERR!("Failed to prepare texture.\n");
        return VK_NULL_HANDLE as VkImageView;
    }

    if ((*resource).bind_flags & WINED3D_BIND_DEPTH_STENCIL as u32) != 0 {
        format_vk = wined3d_format_vk((*resource).format);
    } else {
        format_vk = view_format_vk;
    }

    create_info.sType = VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO;
    create_info.pNext = core::ptr::null::<core::ffi::c_void>();
    create_info.flags = 0;
    create_info.image = (*texture_vk).image.vk_image;
    create_info.viewType = vk_image_view_type_from_wined3d((*resource).type_, (*desc).flags as u32);
    if rtv && create_info.viewType == VK_IMAGE_VIEW_TYPE_3D {
        if (*desc).u.texture.layer_count > 1 {
            create_info.viewType = VK_IMAGE_VIEW_TYPE_2D_ARRAY;
        } else {
            create_info.viewType = VK_IMAGE_VIEW_TYPE_2D;
        }
    }
    create_info.format = (*format_vk).vk_format;
    if is_stencil_view_format(&raw const (*view_format_vk).f) != 0 {
        create_info.components.r = VK_COMPONENT_SWIZZLE_ZERO;
        create_info.components.g = VK_COMPONENT_SWIZZLE_R;
        create_info.components.b = VK_COMPONENT_SWIZZLE_ZERO;
        create_info.components.a = VK_COMPONENT_SWIZZLE_ZERO;
    } else if is_identity_fixup(fixup) != 0
        || can_use_texture_swizzle((*context_vk).c.d3d_info, &raw const (*format_vk).f) == 0
    {
        create_info.components.r = VK_COMPONENT_SWIZZLE_IDENTITY;
        create_info.components.g = VK_COMPONENT_SWIZZLE_IDENTITY;
        create_info.components.b = VK_COMPONENT_SWIZZLE_IDENTITY;
        create_info.components.a = VK_COMPONENT_SWIZZLE_IDENTITY;
    } else {
        wined3d_vk_swizzle_from_color_fixup(&raw mut create_info.components, fixup);
    }
    if ((*resource).bind_flags & WINED3D_BIND_DEPTH_STENCIL as u32) != 0
        && ((*view_format_vk).f.red_size != 0 || (*view_format_vk).f.green_size != 0)
    {
        create_info.subresourceRange.aspectMask = 0;
        if (*view_format_vk).f.red_size != 0 {
            create_info.subresourceRange.aspectMask |= VK_IMAGE_ASPECT_DEPTH_BIT;
        }
        if (*view_format_vk).f.green_size != 0 {
            create_info.subresourceRange.aspectMask |= VK_IMAGE_ASPECT_STENCIL_BIT;
        }
    } else {
        create_info.subresourceRange.aspectMask =
            vk_aspect_mask_from_format(&raw const (*format_vk).f);
    }
    create_info.subresourceRange.baseMipLevel = (*desc).u.texture.level_idx as u32;
    create_info.subresourceRange.levelCount = (*desc).u.texture.level_count as u32;
    if create_info.viewType == VK_IMAGE_VIEW_TYPE_3D {
        if (*desc).u.texture.layer_idx != 0
            || ((*desc).u.texture.layer_count != (*texture_vk).t.resource.depth
                && (*desc).u.texture.layer_count != !(0 as u32))
        {
            WARN!("Partial 3D texture views are not supported.\n");
        }
        create_info.subresourceRange.baseArrayLayer = 0;
        create_info.subresourceRange.layerCount = 1;
    } else {
        create_info.subresourceRange.baseArrayLayer = (*desc).u.texture.layer_idx as u32;
        create_info.subresourceRange.layerCount = (*desc).u.texture.layer_count as u32;
    }
    if (*vk_info).supported[WINED3D_VK_KHR_MAINTENANCE2 as usize] != 0
        || (*vk_info).api_version as u32 >= VK_API_VERSION_1_1
    {
        usage_info.sType = VK_STRUCTURE_TYPE_IMAGE_VIEW_USAGE_CREATE_INFO_KHR;
        usage_info.pNext = core::ptr::null::<core::ffi::c_void>();
        usage_info.usage = usage;
        create_info.pNext = &raw mut usage_info as *const core::ffi::c_void;
    }
    vr = ((*vk_info).vk_ops.vkCreateImageView.unwrap_unchecked())(
        (*device_vk).vk_device,
        &raw mut create_info,
        core::ptr::null::<VkAllocationCallbacks>(),
        &raw mut vk_image_view,
    );
    if (vr as i32) < (0 as i32) {
        ERR!(
            "Failed to create Vulkan image view, vr %s.\n",
            wined3d_debug_vkresult(vr)
        );
        return VK_NULL_HANDLE as VkImageView;
    }

    return vk_image_view;
}

unsafe extern "C" fn wined3d_render_target_view_vk_cs_init(mut object: *mut core::ffi::c_void) {
    let mut view_vk: *mut wined3d_rendertarget_view_vk = object as *mut wined3d_rendertarget_view_vk;
    let mut desc: *mut wined3d_view_desc = &raw mut (*view_vk).v.desc;
    let mut format_vk: *const wined3d_format_vk = core::ptr::null::<wined3d_format_vk>();
    let mut texture_vk: *mut wined3d_texture_vk = core::ptr::null_mut::<wined3d_texture_vk>();
    let mut resource: *mut wined3d_resource = core::ptr::null_mut::<wined3d_resource>();
    let mut context: *mut wined3d_context = core::ptr::null_mut::<wined3d_context>();
    let mut vk_usage: VkImageUsageFlags = 0 as VkImageUsageFlags;
    let mut default_flags: uint32_t = 0 as uint32_t;

    TRACE!("view_vk %p.\n", view_vk);

    resource = (*view_vk).v.resource;
    if (*resource).type_ as core::ffi::c_uint == WINED3D_RTYPE_BUFFER as core::ffi::c_uint {
        FIXME!("Buffer views not implemented.\n");
        return;
    }

    texture_vk = wined3d_texture_vk(texture_from_resource(resource));
    format_vk = wined3d_format_vk((*view_vk).v.format);

    if (*texture_vk).t.layer_count > 1 as UINT {
        default_flags |= WINED3D_VIEW_TEXTURE_ARRAY as uint32_t;
    }

    if (*(*resource).format).id as core::ffi::c_uint == (*format_vk).f.id as core::ffi::c_uint
        && (*desc).flags as uint32_t == default_flags
        && (*desc).u.texture.level_idx == 0
        && (*desc).u.texture.level_count == (*texture_vk).t.level_count
        && (*desc).u.texture.layer_idx == 0
        && (*desc).u.texture.layer_count == (*texture_vk).t.layer_count
        && is_stencil_view_format(&raw const (*format_vk).f) == 0
        && (*resource).type_ as core::ffi::c_uint != WINED3D_RTYPE_TEXTURE_3D as core::ffi::c_uint
        && is_identity_fixup((*format_vk).f.color_fixup) != 0
    {
        TRACE!("Creating identity render target view.\n");
        return;
    }

    if !(*texture_vk).t.swapchain.is_null()
        && (*(*texture_vk).t.swapchain).state.desc.backbuffer_count > 1 as core::ffi::c_uint
    {
        FIXME!("Swapchain views not supported.\n");
        return;
    }

    if (*resource).bind_flags & WINED3D_BIND_RENDER_TARGET as core::ffi::c_uint != 0 {
        vk_usage |= VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT as VkImageUsageFlags;
    }
    if (*resource).bind_flags & WINED3D_BIND_DEPTH_STENCIL as core::ffi::c_uint != 0 {
        vk_usage |= VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT as VkImageUsageFlags;
    }

    context = context_acquire(
        (*resource).device,
        core::ptr::null_mut::<wined3d_texture>(),
        0 as core::ffi::c_uint,
    );
    (*view_vk).vk_image_view = wined3d_view_vk_create_vk_image_view(
        wined3d_context_vk(context),
        desc,
        texture_vk,
        format_vk,
        COLOR_FIXUP_IDENTITY,
        true,
        vk_usage,
    );
    context_release(context);

    if (*view_vk).vk_image_view == 0 {
        return;
    }

    TRACE!(
        "Created image view 0x%s.\n",
        wine_dbgstr_longlong((*view_vk).vk_image_view as ULONGLONG)
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_rendertarget_view_vk_init(view_vk: *mut wined3d_rendertarget_view_vk, desc: *const wined3d_view_desc, resource: *mut wined3d_resource, parent: *mut ::core::ffi::c_void, parent_ops: *const wined3d_parent_ops) -> HRESULT {
    let mut hr: HRESULT = 0;

    TRACE!(
        "view_vk %p, desc %s, resource %p, parent %p, parent_ops %p.\n",
        view_vk,
        wined3d_debug_view_desc(desc, resource),
        resource,
        parent,
        parent_ops
    );

    if ({
        hr = wined3d_rendertarget_view_init(
            core::ptr::addr_of_mut!((*view_vk).v),
            desc,
            resource,
            parent,
            parent_ops,
        );
        hr
    }) < 0
    {
        return hr;
    }

    wined3d_cs_init_object(
        (*(*resource).device).cs as *mut wined3d_cs,
        Some(wined3d_render_target_view_vk_cs_init as unsafe extern "C" fn(*mut core::ffi::c_void)),
        view_vk as *mut core::ffi::c_void,
    );

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_rendertarget_view_create(
    mut desc: *const wined3d_view_desc,
    mut resource: *mut wined3d_resource,
    mut parent: *mut ::core::ffi::c_void,
    mut parent_ops: *const wined3d_parent_ops,
    mut view: *mut *mut wined3d_rendertarget_view,
) -> HRESULT {
    let mut adapter_ops: *const wined3d_adapter_ops = ::core::ptr::null::<wined3d_adapter_ops>();
    WINE_TRACE!("desc %s, resource %p, parent %p, parent_ops %p, view %p.\n", wined3d_debug_view_desc(desc, resource), resource, parent, parent_ops, view);
    adapter_ops = (*(*(*resource).device).adapter).adapter_ops;
    return (*adapter_ops)
        .adapter_create_rendertarget_view
        .unwrap_unchecked()(desc, resource, parent, parent_ops, view);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_rendertarget_view_create_from_sub_resource(
    texture: *mut wined3d_texture,
    sub_resource_idx: ::core::ffi::c_uint,
    parent: *mut ::core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    view: *mut *mut wined3d_rendertarget_view,
) -> HRESULT {
    let mut desc: wined3d_view_desc = core::mem::zeroed();

    WINE_TRACE!("texture %p, sub_resource_idx %u, parent %p, parent_ops %p, view %p.\n", texture, sub_resource_idx, parent, parent_ops, view);

    desc.format_id = (*(*texture).resource.format).id;
    desc.flags = 0 as ::core::ffi::c_uint;
    desc.u.texture.level_idx = sub_resource_idx.wrapping_rem((*texture).level_count);
    desc.u.texture.level_count = 1 as ::core::ffi::c_uint;
    desc.u.texture.layer_idx = sub_resource_idx.wrapping_div((*texture).level_count);
    desc.u.texture.layer_count = 1 as ::core::ffi::c_uint;

    return wined3d_rendertarget_view_create(
        &raw mut desc,
        core::ptr::addr_of_mut!((*texture).resource),
        parent,
        parent_ops,
        view,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_resource_view_incref(
    mut view: *mut wined3d_shader_resource_view,
) -> ULONG {
    let mut refcount: ::core::ffi::c_uint = 0;
    if (*view).desc.flags & WINED3D_VIEW_FORWARD_REFERENCE as ::core::ffi::c_uint != 0 {
        return wined3d_resource_incref((*view).resource);
    }
    refcount = InterlockedIncrement(
        &raw mut (*view).refcount as *mut ::core::ffi::c_long as *mut ::core::ffi::c_long,
    ) as ::core::ffi::c_uint;
    WINE_TRACE!("%p increasing refcount to %u.\n", view, refcount);
    return refcount as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_resource_view_cleanup(view: *mut wined3d_shader_resource_view) {
    ((*(*view).parent_ops).wined3d_object_destroyed.unwrap_unchecked())((*view).parent);
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_resource_view_destroy(view: *mut wined3d_shader_resource_view) {
    wined3d_mutex_lock();
    (*(*(*(*(*view).resource).device).adapter).adapter_ops)
        .adapter_destroy_shader_resource_view
        .unwrap_unchecked()(view);
    wined3d_mutex_unlock();
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_resource_view_decref(
    view: *mut wined3d_shader_resource_view,
) -> ULONG {
    let mut refcount: ::core::ffi::c_uint = 0;
    if ((*view).desc.flags & (WINED3D_VIEW_FORWARD_REFERENCE as ::core::ffi::c_uint)) != 0 {
        return wined3d_resource_decref((*view).resource);
    }
    refcount = InterlockedDecrement(
        core::ptr::addr_of_mut!((*view).refcount) as *mut ::core::ffi::c_long,
    ) as ::core::ffi::c_uint;
    TRACE!("%p decreasing refcount to %u.\n", view, refcount);
    if refcount == 0 {
        let mut resource: *mut wined3d_resource = (*view).resource;
        wined3d_shader_resource_view_destroy(view);
        wined3d_resource_decref(resource);
    }
    return refcount as ULONG;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_resource_view_get_parent(
    view: *const wined3d_shader_resource_view,
) -> *mut core::ffi::c_void {
    // TRACE("view %p.\n", view);
    (*view).parent
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_resource_view_gl_update(
    mut srv_gl: *mut wined3d_shader_resource_view_gl,
    mut context_gl: *mut wined3d_context_gl,
) {
    let mut buffer: *mut wined3d_buffer = buffer_from_resource((*srv_gl).v.resource);

    assert!(!(*srv_gl).bo_user.valid);

    create_buffer_view(
        &raw mut (*srv_gl).gl_view,
        &raw mut (*context_gl).c,
        &raw mut (*srv_gl).v.desc,
        buffer,
        (*srv_gl).v.format,
    );
    (*srv_gl).bo_user.valid = true;
    list_add_head(
        &raw mut (*(*buffer).buffer_object).users,
        &raw mut (*srv_gl).bo_user.entry,
    );
}

unsafe extern "C" fn wined3d_shader_resource_view_gl_cs_init(mut object: *mut ::core::ffi::c_void) {
    let mut view_gl: *mut wined3d_shader_resource_view_gl = object as *mut wined3d_shader_resource_view_gl;
    let mut resource: *mut wined3d_resource = (*view_gl).v.resource;
    let mut view_format: *const wined3d_format = core::ptr::null::<wined3d_format>();
    let mut gl_info: *const wined3d_gl_info = core::ptr::null::<wined3d_gl_info>();
    let mut desc: *const wined3d_view_desc = core::ptr::null::<wined3d_view_desc>();
    let mut view_target: GLenum = 0;

    TRACE!("view_gl %p.\n", view_gl);

    view_format = (*view_gl).v.format;
    gl_info = core::ptr::addr_of_mut!(
        (*wined3d_adapter_gl((*(*resource).device).adapter)).gl_info
    );
    desc = core::ptr::addr_of_mut!((*view_gl).v.desc);

    if (*resource).type_ as ::core::ffi::c_uint == WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint {
        let mut buffer: *mut wined3d_buffer = buffer_from_resource(resource);
        let mut context: *mut wined3d_context = core::ptr::null_mut::<wined3d_context>();

        context = context_acquire(
            (*resource).device,
            core::ptr::null_mut::<wined3d_texture>(),
            0 as ::core::ffi::c_uint,
        );
        create_buffer_view(
            core::ptr::addr_of_mut!((*view_gl).gl_view),
            context,
            desc,
            buffer,
            view_format,
        );
        (*view_gl).bo_user.valid = 1 != 0;
        list_add_head(
            core::ptr::addr_of_mut!((*(*buffer).buffer_object).users),
            core::ptr::addr_of_mut!((*view_gl).bo_user.entry),
        );
        context_release(context);
    } else {
        let mut texture_gl: *mut wined3d_texture_gl =
            wined3d_texture_gl(texture_from_resource(resource));
        let mut resource_class: GLenum = 0;
        let mut view_class: GLenum = 0;

        resource_class = (*wined3d_format_gl((*resource).format)).view_class;
        view_class = (*wined3d_format_gl(view_format)).view_class;
        view_target = get_texture_view_target(gl_info, desc, texture_gl);

        if (*(*resource).format).id as ::core::ffi::c_uint == (*view_format).id as ::core::ffi::c_uint
            && (*texture_gl).target == view_target
            && (*desc).u.texture.level_idx == 0
            && (*desc).u.texture.level_count == (*texture_gl).t.level_count
            && (*desc).u.texture.layer_idx == 0
            && (*desc).u.texture.layer_count == (*texture_gl).t.layer_count
            && is_stencil_view_format(view_format) == 0
        {
            TRACE!("Creating identity shader resource view.\n");
        } else if !(*texture_gl).t.swapchain.is_null()
            && (*(*texture_gl).t.swapchain).state.desc.backbuffer_count > 1 as ::core::ffi::c_uint
        {
            FIXME!("Swapchain shader resource views not supported.\n");
        } else if (*(*resource).format).typeless_id as ::core::ffi::c_uint
            == (*view_format).typeless_id as ::core::ffi::c_uint
            && resource_class == view_class
        {
            create_texture_view(
                core::ptr::addr_of_mut!((*view_gl).gl_view),
                view_target,
                desc,
                texture_gl,
                view_format,
            );
        } else if wined3d_format_is_depth_view((*(*resource).format).id, (*view_format).id) != 0 {
            create_texture_view(
                core::ptr::addr_of_mut!((*view_gl).gl_view),
                view_target,
                desc,
                texture_gl,
                (*resource).format,
            );
        } else {
            FIXME!(
                "Shader resource view not supported, resource format %s, view format %s.\n",
                debug_d3dformat((*(*resource).format).id),
                debug_d3dformat((*view_format).id),
            );
        }
    }
}

unsafe extern "C" fn wined3d_shader_resource_view_init(
    view: *mut wined3d_shader_resource_view,
    desc: *const wined3d_view_desc,
    resource: *mut wined3d_resource,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
) -> HRESULT {
    (*view).refcount = 1;
    (*view).parent = parent;
    (*view).parent_ops = parent_ops;

    if ((*resource).bind_flags & WINED3D_BIND_SHADER_RESOURCE) == 0 {
        return E_INVALIDARG;
    }
    (*view).format = validate_resource_view(desc, resource, (FALSE) as BOOL, (FALSE) as BOOL);
    if (*view).format.is_null() {
        return E_INVALIDARG;
    }
    (*view).desc = *desc;

    /* If WINED3D_VIEW_FORWARD_REFERENCE, the view shouldn't take a reference
     * to the resource. However, the reference to the view returned by this
     * function should translate to a resource reference, so we increment the
     * resource's reference count anyway. */
    (*view).resource = resource;
    wined3d_resource_incref((*view).resource);

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_resource_view_gl_init(view_gl: *mut wined3d_shader_resource_view_gl, desc: *const wined3d_view_desc, resource: *mut wined3d_resource, parent: *mut ::core::ffi::c_void, parent_ops: *const wined3d_parent_ops) -> HRESULT {
    let mut hr: HRESULT;

    hr = wined3d_shader_resource_view_init(
        &raw mut (*view_gl).v,
        desc,
        resource,
        parent,
        parent_ops,
    );
    if hr < 0 {
        return hr;
    }

    list_init(&raw mut (*view_gl).bo_user.entry);
    wined3d_cs_init_object(
        (*(*resource).device).cs as *mut wined3d_cs,
        Some(
            wined3d_shader_resource_view_gl_cs_init
                as unsafe extern "C" fn(*mut core::ffi::c_void) -> (),
        ),
        view_gl as *mut core::ffi::c_void,
    );

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_resource_view_vk_update_buffer(
    srv_vk: *mut wined3d_shader_resource_view_vk,
    context_vk: *mut wined3d_context_vk,
) {
    let view_format_vk: *const wined3d_format_vk = wined3d_format_vk((*srv_vk).v.format);
    let desc: *const wined3d_view_desc = core::ptr::addr_of!((*srv_vk).v.desc);
    let resource: *mut wined3d_resource = (*srv_vk).v.resource;
    let view_vk: *mut wined3d_view_vk = core::ptr::addr_of_mut!((*srv_vk).view_vk);
    let mut buffer_vk: *mut wined3d_buffer_vk = core::ptr::null_mut();
    let mut vk_buffer_view: VkBufferView = 0;

    assert!(!(*view_vk).bo_user.valid);

    buffer_vk = wined3d_buffer_vk(buffer_from_resource(resource));
    wined3d_context_vk_destroy_vk_buffer_view(context_vk, (*view_vk).u.vk_buffer_view, (*view_vk).command_buffer_id);
    vk_buffer_view = wined3d_view_vk_create_vk_buffer_view(context_vk, desc, buffer_vk, view_format_vk);
    if vk_buffer_view != 0 {
        (*view_vk).u.vk_buffer_view = vk_buffer_view;
        (*view_vk).bo_user.valid = true;
        list_add_head(
            core::ptr::addr_of_mut!((*(*buffer_vk).b.buffer_object).users),
            core::ptr::addr_of_mut!((*view_vk).bo_user.entry),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_resource_view_vk_update_layout(
    srv_vk: *mut wined3d_shader_resource_view_vk,
    layout: VkImageLayout,
) {
    (*srv_vk).view_vk.u.vk_image_info.imageLayout = layout;
}

unsafe extern "C" fn wined3d_shader_resource_view_vk_cs_init(object: *mut ::core::ffi::c_void) {
    let srv_vk: *mut wined3d_shader_resource_view_vk = object as *mut wined3d_shader_resource_view_vk;
    let desc: *mut wined3d_view_desc = core::ptr::addr_of_mut!((*srv_vk).v.desc);
    let mut texture_vk: *mut wined3d_texture_vk = core::ptr::null_mut();
    let mut format: *const wined3d_format = core::ptr::null();
    let mut buffer_vk: *mut wined3d_buffer_vk = core::ptr::null_mut();
    let mut resource: *mut wined3d_resource = core::ptr::null_mut();
    let mut context: *mut wined3d_context = core::ptr::null_mut();
    let mut vk_buffer_view: VkBufferView = 0;
    let mut default_flags: uint32_t = 0;
    let mut vk_image_view: VkImageView = 0;
    let mut bo: *mut wined3d_bo_vk = core::ptr::null_mut();

    TRACE!("srv_vk %p.\n", srv_vk);

    resource = (*srv_vk).v.resource;
    format = (*srv_vk).v.format;

    if (*resource).type_ == WINED3D_RTYPE_BUFFER {
        buffer_vk = wined3d_buffer_vk(buffer_from_resource(resource));

        context = context_acquire((*resource).device, core::ptr::null_mut(), 0);
        vk_buffer_view = wined3d_view_vk_create_vk_buffer_view(
            wined3d_context_vk(context),
            desc,
            buffer_vk,
            wined3d_format_vk(format),
        );
        context_release(context);

        if vk_buffer_view == 0 {
            return;
        }
        bo = wined3d_bo_vk((*buffer_vk).b.buffer_object);

        TRACE!("Created buffer view 0x%s.\n", wine_dbgstr_longlong(vk_buffer_view as ULONGLONG));

        (*srv_vk).view_vk.u.vk_buffer_view = vk_buffer_view;
        (*srv_vk).view_vk.bo_user.valid = true;
        list_add_head(
            core::ptr::addr_of_mut!((*bo).b.users),
            core::ptr::addr_of_mut!((*srv_vk).view_vk.bo_user.entry),
        );

        return;
    }

    texture_vk = wined3d_texture_vk(texture_from_resource(resource));

    if (*texture_vk).t.layer_count > 1 {
        default_flags |= WINED3D_VIEW_TEXTURE_ARRAY as uint32_t;
    }

    if (*(*resource).format).id == (*format).id
        && (*desc).flags as uint32_t == default_flags
        && (*desc).u.texture.level_idx == 0
        && (*desc).u.texture.level_count == (*texture_vk).t.level_count
        && (*desc).u.texture.layer_idx == 0
        && (*desc).u.texture.layer_count == (*texture_vk).t.layer_count
        && ((*resource).bind_flags & WINED3D_BIND_DEPTH_STENCIL as ::core::ffi::c_uint) == 0
    {
        TRACE!("Creating identity shader resource view.\n");
        return;
    }

    if !(*texture_vk).t.swapchain.is_null()
        && (*(*texture_vk).t.swapchain).state.desc.backbuffer_count > 1
    {
        FIXME!("Swapchain shader resource views not supported.\n");
    }

    context = context_acquire((*resource).device, core::ptr::null_mut(), 0);
    vk_image_view = wined3d_view_vk_create_vk_image_view(
        wined3d_context_vk(context),
        desc,
        texture_vk,
        wined3d_format_vk(format),
        (*format).color_fixup,
        false,
        VK_IMAGE_USAGE_SAMPLED_BIT,
    );
    context_release(context);

    if vk_image_view == 0 {
        return;
    }

    TRACE!("Created image view 0x%s.\n", wine_dbgstr_longlong(vk_image_view as ULONGLONG));

    (*srv_vk).view_vk.u.vk_image_info.imageView = vk_image_view;
    (*srv_vk).view_vk.u.vk_image_info.sampler = (VK_NULL_HANDLE as _);
    if (*texture_vk).layout == VK_IMAGE_LAYOUT_GENERAL {
        (*srv_vk).view_vk.u.vk_image_info.imageLayout = VK_IMAGE_LAYOUT_GENERAL;
    } else {
        (*srv_vk).view_vk.u.vk_image_info.imageLayout = VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL;
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_resource_view_vk_init(view_vk: *mut wined3d_shader_resource_view_vk, desc: *const wined3d_view_desc, resource: *mut wined3d_resource, parent: *mut ::core::ffi::c_void, parent_ops: *const wined3d_parent_ops) -> HRESULT {
    let mut hr: HRESULT = 0;

    hr = wined3d_shader_resource_view_init(
        core::ptr::addr_of_mut!((*view_vk).v),
        desc,
        resource,
        parent,
        parent_ops,
    );
    if hr < 0 {
        return hr;
    }

    list_init(core::ptr::addr_of_mut!((*view_vk).view_vk.bo_user.entry));
    wined3d_cs_init_object(
        (*(*resource).device).cs as *mut wined3d_cs,
        Some(wined3d_shader_resource_view_vk_cs_init),
        view_vk as *mut core::ffi::c_void,
    );

    return hr;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_resource_view_create(
    mut desc: *const wined3d_view_desc,
    mut resource: *mut wined3d_resource,
    mut parent: *mut ::core::ffi::c_void,
    mut parent_ops: *const wined3d_parent_ops,
    mut view: *mut *mut wined3d_shader_resource_view,
) -> HRESULT {
    let mut adapter_ops: *const wined3d_adapter_ops = core::ptr::null::<wined3d_adapter_ops>();
    WINE_TRACE!("desc %s, resource %p, parent %p, parent_ops %p, view %p.\n", wined3d_debug_view_desc(desc, resource), resource, parent, parent_ops, view);
    adapter_ops = (*(*(*resource).device).adapter).adapter_ops;
    return (*adapter_ops)
        .adapter_create_shader_resource_view
        .unwrap_unchecked()(desc, resource, parent, parent_ops, view);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_resource_view_gl_bind(
    view_gl: *mut wined3d_shader_resource_view_gl,
    unit: ::core::ffi::c_uint,
    sampler_gl: *mut wined3d_sampler_gl,
    context_gl: *mut wined3d_context_gl,
) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut texture_gl: *mut wined3d_texture_gl = ::core::ptr::null_mut::<wined3d_texture_gl>();
    wined3d_context_gl_active_texture(context_gl, gl_info, unit);
    if (*view_gl).gl_view.name != 0 {
        wined3d_context_gl_bind_texture(
            context_gl,
            (*view_gl).gl_view.target,
            (*view_gl).gl_view.name,
        );
        wined3d_sampler_gl_bind(
            sampler_gl,
            unit,
            ::core::ptr::null_mut::<wined3d_texture_gl>(),
            context_gl,
        );
        return;
    }
    if (*(*view_gl).v.resource).type_ as ::core::ffi::c_uint
        == WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint
    {
        WINE_FIXME!("Buffer shader resources not supported.\n");
        return;
    }
    texture_gl = wined3d_texture_gl(wined3d_texture_from_resource((*view_gl).v.resource));
    wined3d_texture_gl_bind(texture_gl, context_gl, (*sampler_gl).s.desc.srgb_decode);
    wined3d_sampler_gl_bind(sampler_gl, unit, texture_gl, context_gl);
}

unsafe extern "C" fn shader_resource_view_gl_bind_and_dirtify(view_gl: *mut wined3d_shader_resource_view_gl, context_gl: *mut wined3d_context_gl) {
    /* FIXME: Ideally we'd only do this when touching a binding that's used by
     * a shader. */
    context_invalidate_compute_state(core::ptr::addr_of_mut!((*context_gl).c), (STATE_COMPUTE_SHADER_RESOURCE_BINDING) as DWORD);
    context_invalidate_state(core::ptr::addr_of_mut!((*context_gl).c), (STATE_GRAPHICS_SHADER_RESOURCE_BINDING as u32));

    wined3d_context_gl_bind_texture(context_gl, (*view_gl).gl_view.target, (*view_gl).gl_view.name);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_resource_view_gl_generate_mipmap(view_gl: *mut wined3d_shader_resource_view_gl, context_gl: *mut wined3d_context_gl) {
    let mut i: c_uint = 0;
    let mut j: c_uint = 0;
    let mut layer_count: c_uint = 0;
    let mut level_count: c_uint = 0;
    let mut base_level: c_uint = 0;
    let mut base_layer: c_uint = 0;
    let mut sub_resource_idx: c_uint = 0;
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut texture_gl: *mut wined3d_texture_gl = core::ptr::null_mut();
    let mut gl_tex: *mut gl_texture = core::ptr::null_mut();
    let mut location: DWORD = 0;
    let mut srgb: BOOL = 0;

    layer_count = (*view_gl).v.desc.u.texture.layer_count;
    level_count = (*view_gl).v.desc.u.texture.level_count;
    base_level = (*view_gl).v.desc.u.texture.level_idx;
    base_layer = (*view_gl).v.desc.u.texture.layer_idx;

    texture_gl = wined3d_texture_gl(texture_from_resource((*view_gl).v.resource));
    srgb = (((*texture_gl).t.flags & WINED3D_TEXTURE_IS_SRGB) != 0) as BOOL;
    location = (if srgb != 0 {
        WINED3D_LOCATION_TEXTURE_SRGB
    } else {
        WINED3D_LOCATION_TEXTURE_RGB
    }) as DWORD;
    i = 0 as c_uint;
    while i < layer_count {
        sub_resource_idx = base_layer
            .wrapping_add(i)
            .wrapping_mul((*texture_gl).t.level_count)
            .wrapping_add(base_level);
        if wined3d_texture_load_location(
            core::ptr::addr_of_mut!((*texture_gl).t),
            sub_resource_idx,
            core::ptr::addr_of_mut!((*context_gl).c),
            location as uint32_t,
        ) == 0
        {
            ERR!("Failed to load source layer %u.\n", base_layer.wrapping_add(i));
        }
        i = i.wrapping_add(1);
    }

    if (*view_gl).gl_view.name != 0 {
        shader_resource_view_gl_bind_and_dirtify(view_gl, context_gl);
    } else {
        wined3d_texture_gl_bind_and_dirtify(texture_gl, context_gl, srgb);
        (*gl_info)
            .gl_ops
            .gl
            .p_glTexParameteri
            .unwrap_unchecked()(
            (*texture_gl).target,
            GL_TEXTURE_BASE_LEVEL as GLenum,
            base_level as GLint,
        );
        (*gl_info)
            .gl_ops
            .gl
            .p_glTexParameteri
            .unwrap_unchecked()(
            (*texture_gl).target,
            GL_TEXTURE_MAX_LEVEL as GLenum,
            base_level
                .wrapping_add(level_count)
                .wrapping_sub(1 as c_uint) as GLint,
        );
    }

    if (*gl_info).supported[ARB_SAMPLER_OBJECTS as usize] != 0 {
        ((*gl_info).gl_ops.ext.p_glBindSampler.unwrap_unchecked())(
            (*context_gl).active_texture as GLuint,
            0 as GLuint,
        );
    }
    gl_tex = wined3d_texture_gl_get_gl_texture(texture_gl, srgb != 0);
    if ((*(*context_gl).c.d3d_info).wined3d_creation_flags & WINED3D_SRGB_READ_WRITE_CONTROL) != 0
        && (*gl_info).supported[EXT_TEXTURE_SRGB_DECODE as usize] != 0
    {
        (*gl_info)
            .gl_ops
            .gl
            .p_glTexParameteri
            .unwrap_unchecked()(
            (*texture_gl).target,
            GL_TEXTURE_SRGB_DECODE_EXT as GLenum,
            (GL_SKIP_DECODE_EXT as i32),
        );
        (*gl_tex).sampler_desc.srgb_decode = FALSE as BOOL;
    }

    (*gl_info)
        .fbo_ops
        .glGenerateMipmap
        .unwrap_unchecked()((*texture_gl).target);
    checkGLcall!(gl_info, "glGenerateMipMap()");

    i = 0 as c_uint;
    while i < layer_count {
        j = 1 as c_uint;
        while j < level_count {
            sub_resource_idx = base_layer
                .wrapping_add(i)
                .wrapping_mul((*texture_gl).t.level_count)
                .wrapping_add(base_level)
                .wrapping_add(j);
            wined3d_texture_validate_location(
                core::ptr::addr_of_mut!((*texture_gl).t),
                sub_resource_idx,
                location as uint32_t,
            );
            wined3d_texture_invalidate_location(
                core::ptr::addr_of_mut!((*texture_gl).t),
                sub_resource_idx,
                !(location as uint32_t),
            );
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }

    if (*view_gl).gl_view.name == 0 {
        (*gl_tex).sampler_desc.mip_base_level = base_level;
        (*gl_info)
            .gl_ops
            .gl
            .p_glTexParameteri
            .unwrap_unchecked()(
            (*texture_gl).target,
            GL_TEXTURE_MAX_LEVEL as GLenum,
            (*texture_gl)
                .t
                .level_count
                .wrapping_sub(1 as c_uint) as GLint,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_resource_view_vk_generate_mipmap(srv_vk: *mut wined3d_shader_resource_view_vk, context_vk: *mut wined3d_context_vk) {
    let mut i: u32 = 0;
    let mut j: u32 = 0;
    let mut layer_count: u32 = 0;
    let mut level_count: u32 = 0;
    let mut base_level: u32 = 0;
    let mut base_layer: u32 = 0;
    let mut sub_resource_idx: u32 = 0;
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut vk_src_range: VkImageSubresourceRange = VkImageSubresourceRange {
        aspectMask: 0,
        baseMipLevel: 0,
        levelCount: 0,
        baseArrayLayer: 0,
        layerCount: 0,
    };
    let mut vk_dst_range: VkImageSubresourceRange = VkImageSubresourceRange {
        aspectMask: 0,
        baseMipLevel: 0,
        levelCount: 0,
        baseArrayLayer: 0,
        layerCount: 0,
    };
    let mut texture_vk: *mut wined3d_texture_vk = core::ptr::null_mut();
    let mut vk_command_buffer: VkCommandBuffer = core::ptr::null_mut();
    let mut region: VkImageBlit = VkImageBlit {
        srcSubresource: VkImageSubresourceLayers {
            aspectMask: 0,
            mipLevel: 0,
            baseArrayLayer: 0,
            layerCount: 0,
        },
        srcOffsets: [VkOffset3D { x: 0, y: 0, z: 0 }; 2],
        dstSubresource: VkImageSubresourceLayers {
            aspectMask: 0,
            mipLevel: 0,
            baseArrayLayer: 0,
            layerCount: 0,
        },
        dstOffsets: [VkOffset3D { x: 0, y: 0, z: 0 }; 2],
    };

    layer_count = (*srv_vk).v.desc.u.texture.layer_count;
    level_count = (*srv_vk).v.desc.u.texture.level_count;
    base_level = (*srv_vk).v.desc.u.texture.level_idx;
    base_layer = (*srv_vk).v.desc.u.texture.layer_idx;

    texture_vk = wined3d_texture_vk(texture_from_resource((*srv_vk).v.resource));
    i = 0;
    while i < layer_count {
        sub_resource_idx = base_layer
            .wrapping_add(i)
            .wrapping_mul((*texture_vk).t.level_count)
            .wrapping_add(base_level);
        if wined3d_texture_load_location(
            core::ptr::addr_of_mut!((*texture_vk).t),
            sub_resource_idx,
            core::ptr::addr_of_mut!((*context_vk).c),
            WINED3D_LOCATION_TEXTURE_RGB,
        ) == 0
        {
            ERR!("Failed to load source layer %u.\n", base_layer.wrapping_add(i));
        }
        i = i.wrapping_add(1);
    }

    if ((*(*context_vk).c.d3d_info).wined3d_creation_flags & WINED3D_SRGB_READ_WRITE_CONTROL) != 0 {
        FIXME!("Unhandled sRGB read/write control.\n");
    }

    if (*wined3d_format_vk((*srv_vk).v.format)).vk_format
        != (*wined3d_format_vk((*texture_vk).t.resource.format)).vk_format
    {
        FIXME!(
            "Ignoring view format %s.\n",
            debug_d3dformat((*(*srv_vk).v.format).id)
        );
    }

    if wined3d_resource_get_sample_count(core::ptr::addr_of_mut!((*texture_vk).t.resource))
        > 1
    {
        FIXME!("Unhandled multi-sampled resource.\n");
    }

    vk_command_buffer = wined3d_context_vk_get_command_buffer(context_vk);
    if vk_command_buffer.is_null() {
        ERR!("Failed to get command buffer.\n");
        return;
    }

    vk_src_range.aspectMask = vk_aspect_mask_from_format((*texture_vk).t.resource.format);
    vk_src_range.baseMipLevel = base_level;
    vk_src_range.levelCount = 1;
    vk_src_range.baseArrayLayer = base_layer;
    vk_src_range.layerCount = layer_count;

    vk_dst_range = vk_src_range;
    vk_dst_range.baseMipLevel = vk_dst_range.baseMipLevel.wrapping_add(1);

    wined3d_context_vk_image_barrier(
        context_vk,
        vk_command_buffer,
        VK_PIPELINE_STAGE_ALL_COMMANDS_BIT,
        VK_PIPELINE_STAGE_TRANSFER_BIT,
        vk_access_mask_from_bind_flags((*texture_vk).t.resource.bind_flags),
        VK_ACCESS_TRANSFER_READ_BIT,
        (*texture_vk).layout,
        VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL,
        (*texture_vk).image.vk_image,
        core::ptr::addr_of_mut!(vk_src_range),
    );
    wined3d_context_vk_image_barrier(
        context_vk,
        vk_command_buffer,
        VK_PIPELINE_STAGE_ALL_COMMANDS_BIT,
        VK_PIPELINE_STAGE_TRANSFER_BIT,
        vk_access_mask_from_bind_flags((*texture_vk).t.resource.bind_flags),
        VK_ACCESS_TRANSFER_WRITE_BIT,
        (*texture_vk).layout,
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
        (*texture_vk).image.vk_image,
        core::ptr::addr_of_mut!(vk_dst_range),
    );

    region.srcSubresource.aspectMask = vk_src_range.aspectMask;
    region.srcSubresource.mipLevel = vk_src_range.baseMipLevel;
    region.srcSubresource.baseArrayLayer = vk_src_range.baseArrayLayer;
    region.srcSubresource.layerCount = vk_src_range.layerCount;
    region.srcOffsets[0 as usize].x = 0;
    region.srcOffsets[0 as usize].y = 0;
    region.srcOffsets[0 as usize].z = 0;

    region.dstSubresource.aspectMask = vk_dst_range.aspectMask;
    region.dstSubresource.mipLevel = vk_dst_range.baseMipLevel;
    region.dstSubresource.baseArrayLayer = vk_dst_range.baseArrayLayer;
    region.dstSubresource.layerCount = vk_dst_range.layerCount;
    region.dstOffsets[0 as usize].x = 0;
    region.dstOffsets[0 as usize].y = 0;
    region.dstOffsets[0 as usize].z = 0;

    i = 1;
    while i < level_count {
        region.srcOffsets[1 as usize].x =
            wined3d_texture_get_level_width(
                core::ptr::addr_of_mut!((*texture_vk).t),
                vk_src_range.baseMipLevel,
            ) as i32;
        region.srcOffsets[1 as usize].y =
            wined3d_texture_get_level_height(
                core::ptr::addr_of_mut!((*texture_vk).t),
                vk_src_range.baseMipLevel,
            ) as i32;
        region.srcOffsets[1 as usize].z =
            wined3d_texture_get_level_depth(
                core::ptr::addr_of_mut!((*texture_vk).t),
                vk_src_range.baseMipLevel,
            ) as i32;

        region.dstOffsets[1 as usize].x =
            wined3d_texture_get_level_width(
                core::ptr::addr_of_mut!((*texture_vk).t),
                vk_dst_range.baseMipLevel,
            ) as i32;
        region.dstOffsets[1 as usize].y =
            wined3d_texture_get_level_height(
                core::ptr::addr_of_mut!((*texture_vk).t),
                vk_dst_range.baseMipLevel,
            ) as i32;
        region.dstOffsets[1 as usize].z =
            wined3d_texture_get_level_depth(
                core::ptr::addr_of_mut!((*texture_vk).t),
                vk_dst_range.baseMipLevel,
            ) as i32;

        ((*vk_info).vk_ops.vkCmdBlitImage.unwrap_unchecked())(
            vk_command_buffer,
            (*texture_vk).image.vk_image,
            VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL,
            (*texture_vk).image.vk_image,
            VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
            1,
            core::ptr::addr_of_mut!(region),
            VK_FILTER_LINEAR,
        );

        wined3d_context_vk_image_barrier(
            context_vk,
            vk_command_buffer,
            VK_PIPELINE_STAGE_TRANSFER_BIT,
            VK_PIPELINE_STAGE_ALL_COMMANDS_BIT,
            VK_ACCESS_TRANSFER_READ_BIT,
            vk_access_mask_from_bind_flags((*texture_vk).t.resource.bind_flags),
            VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL,
            (*texture_vk).layout,
            (*texture_vk).image.vk_image,
            core::ptr::addr_of_mut!(vk_src_range),
        );

        if i == level_count.wrapping_sub(1) {
            wined3d_context_vk_image_barrier(
                context_vk,
                vk_command_buffer,
                VK_PIPELINE_STAGE_TRANSFER_BIT,
                VK_PIPELINE_STAGE_ALL_COMMANDS_BIT,
                VK_ACCESS_TRANSFER_WRITE_BIT,
                vk_access_mask_from_bind_flags((*texture_vk).t.resource.bind_flags),
                VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
                (*texture_vk).layout,
                (*texture_vk).image.vk_image,
                core::ptr::addr_of_mut!(vk_dst_range),
            );
        } else {
            vk_src_range.baseMipLevel = vk_src_range.baseMipLevel.wrapping_add(1);
            region.srcSubresource.mipLevel = vk_src_range.baseMipLevel;
            vk_dst_range.baseMipLevel = vk_dst_range.baseMipLevel.wrapping_add(1);
            region.dstSubresource.mipLevel = vk_dst_range.baseMipLevel;

            wined3d_context_vk_image_barrier(
                context_vk,
                vk_command_buffer,
                VK_PIPELINE_STAGE_TRANSFER_BIT,
                VK_PIPELINE_STAGE_TRANSFER_BIT,
                VK_ACCESS_TRANSFER_WRITE_BIT,
                VK_ACCESS_TRANSFER_READ_BIT,
                VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
                VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL,
                (*texture_vk).image.vk_image,
                core::ptr::addr_of_mut!(vk_src_range),
            );
            wined3d_context_vk_image_barrier(
                context_vk,
                vk_command_buffer,
                VK_PIPELINE_STAGE_ALL_COMMANDS_BIT,
                VK_PIPELINE_STAGE_TRANSFER_BIT,
                vk_access_mask_from_bind_flags((*texture_vk).t.resource.bind_flags),
                VK_ACCESS_TRANSFER_WRITE_BIT,
                (*texture_vk).layout,
                VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
                (*texture_vk).image.vk_image,
                core::ptr::addr_of_mut!(vk_dst_range),
            );
        }
        i = i.wrapping_add(1);
    }

    i = 0;
    while i < layer_count {
        j = 1;
        while j < level_count {
            sub_resource_idx = base_layer
                .wrapping_add(i)
                .wrapping_mul((*texture_vk).t.level_count)
                .wrapping_add(base_level)
                .wrapping_add(j);
            wined3d_texture_validate_location(
                core::ptr::addr_of_mut!((*texture_vk).t),
                sub_resource_idx,
                WINED3D_LOCATION_TEXTURE_RGB,
            );
            wined3d_texture_invalidate_location(
                core::ptr::addr_of_mut!((*texture_vk).t),
                sub_resource_idx,
                !(WINED3D_LOCATION_TEXTURE_RGB as u32),
            );
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }

    wined3d_context_vk_reference_texture(context_vk, texture_vk);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_unordered_access_view_incref(
    mut view: *mut wined3d_unordered_access_view,
) -> ULONG {
    let mut refcount: ::core::ffi::c_uint = InterlockedIncrement(
        &raw mut (*view).refcount as *mut ::core::ffi::c_long as *mut ::core::ffi::c_long,
    ) as ::core::ffi::c_uint;
    WINE_TRACE!("%p increasing refcount to %u.\n", view, refcount);
    return refcount as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_unordered_access_view_cleanup(view: *mut wined3d_unordered_access_view) {
    ((*(*view).parent_ops).wined3d_object_destroyed.unwrap_unchecked())((*view).parent);
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_unordered_access_view_decref(
    view: *mut wined3d_unordered_access_view,
) -> ULONG {
    let mut refcount: u32 = InterlockedDecrement(
        core::ptr::addr_of_mut!((*view).refcount) as *mut i32,
    ) as u32;

    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        TRACE!("%p decreasing refcount to %u.\n", view, refcount);
    }

    if refcount == 0 {
        let mut resource: *mut wined3d_resource = (*view).resource;
        wined3d_mutex_lock();
        (*(*(*(*resource).device).adapter).adapter_ops)
            .adapter_destroy_unordered_access_view
            .unwrap_unchecked()(view);
        wined3d_mutex_unlock();
        wined3d_resource_decref(resource);
    }

    return refcount as ULONG;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_unordered_access_view_get_parent(
    view: *const wined3d_unordered_access_view,
) -> *mut core::ffi::c_void {
    // TRACE("view %p.\n", view);
    (*view).parent
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_unordered_access_view_invalidate_location(view: *mut wined3d_unordered_access_view, location: u32) {
    wined3d_view_invalidate_location((*view).resource, core::ptr::addr_of_mut!((*view).desc), location as DWORD);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_unordered_access_view_gl_clear(view_gl: *mut wined3d_unordered_access_view_gl, clear_value: *const wined3d_uvec4, context_gl: *mut wined3d_context_gl, fp: bool) {
    let mut gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut format_gl: *const wined3d_format_gl = core::ptr::null::<wined3d_format_gl>();
    let mut resource: *mut wined3d_resource = core::ptr::null_mut::<wined3d_resource>();
    let mut buffer: *mut wined3d_buffer = core::ptr::null_mut::<wined3d_buffer>();
    let mut bo_gl: *mut wined3d_bo_gl = core::ptr::null_mut::<wined3d_bo_gl>();
    let mut offset: ::core::ffi::c_uint = 0;
    let mut size: ::core::ffi::c_uint = 0;

    resource = (*view_gl).v.resource;
    if (*resource).type_ as ::core::ffi::c_uint != WINED3D_RTYPE_BUFFER {
        let mut layer_count: ::core::ffi::c_uint = 0;
        let mut level_count: ::core::ffi::c_uint = 0;
        let mut base_level: ::core::ffi::c_uint = 0;
        let mut base_layer: ::core::ffi::c_uint = 0;
        let mut sub_resource_idx: ::core::ffi::c_uint = 0;
        let mut width: ::core::ffi::c_uint = 0;
        let mut height: ::core::ffi::c_uint = 0;
        let mut depth: ::core::ffi::c_uint = 0;
        let mut i: ::core::ffi::c_uint = 0;
        let mut j: ::core::ffi::c_uint = 0;
        let mut texture_gl: *mut wined3d_texture_gl = core::ptr::null_mut::<wined3d_texture_gl>();
        let mut data: *const ::core::ffi::c_void = clear_value as *const ::core::ffi::c_void;
        let mut gl_format: GLenum = 0;
        let mut gl_type: GLenum = 0;
        let mut packed: uint32_t = 0;

        if (*gl_info).supported[ARB_CLEAR_TEXTURE as usize] == 0 {
            FIXME!("OpenGL implementation does not support ARB_clear_texture.\n");
            return;
        }

        format_gl = wined3d_format_gl((*resource).format);
        texture_gl = wined3d_texture_gl(texture_from_resource(resource));
        level_count = (*view_gl).v.desc.u.texture.level_count;
        base_level = (*view_gl).v.desc.u.texture.level_idx;

        if (*resource).type_ as ::core::ffi::c_uint == WINED3D_RTYPE_TEXTURE_3D {
            layer_count = 1;
            base_layer = 0;
        } else {
            layer_count = (*view_gl).v.desc.u.texture.layer_count;
            base_layer = (*view_gl).v.desc.u.texture.layer_idx;
        }

        if (*format_gl).f.byte_count <= 4 && !fp {
            gl_format = ((*format_gl).format) as u32;
            gl_type = ((*format_gl).type_) as u32;
            packed = wined3d_format_pack(core::ptr::addr_of!((*format_gl).f), clear_value);
            data = core::ptr::addr_of_mut!(packed) as *const ::core::ffi::c_void;
        } else if ((*resource).format_attrs & WINED3D_FORMAT_ATTR_INTEGER) != 0 {
            gl_format = GL_RGBA_INTEGER;
            gl_type = GL_UNSIGNED_INT;
        } else {
            gl_format = GL_RGBA;
            gl_type = GL_FLOAT;
        }

        i = 0;
        while i < layer_count {
            j = 0;
            while j < level_count {
                sub_resource_idx = base_layer
                    .wrapping_add(i)
                    .wrapping_mul((*texture_gl).t.level_count)
                    .wrapping_add(base_level)
                    .wrapping_add(j);
                wined3d_texture_prepare_location(
                    core::ptr::addr_of_mut!((*texture_gl).t),
                    sub_resource_idx,
                    core::ptr::addr_of_mut!((*context_gl).c),
                    WINED3D_LOCATION_TEXTURE_RGB,
                );

                width = wined3d_texture_get_level_width(
                    core::ptr::addr_of_mut!((*texture_gl).t),
                    base_level.wrapping_add(j),
                );
                height = wined3d_texture_get_level_height(
                    core::ptr::addr_of_mut!((*texture_gl).t),
                    base_level.wrapping_add(j),
                );
                depth = wined3d_texture_get_level_depth(
                    core::ptr::addr_of_mut!((*texture_gl).t),
                    base_level.wrapping_add(j),
                );

                let target_: u32 = (*texture_gl).target;
                if target_ == GL_TEXTURE_1D_ARRAY {
                    ((*gl_info).gl_ops.ext.p_glClearTexSubImage.unwrap_unchecked())(
                        (*texture_gl).texture_rgb.name,
                        (base_level.wrapping_add(j)) as i32,
                        0,
                        (base_layer.wrapping_add(i)) as i32,
                        0,
                        (width) as i32,
                        1,
                        1,
                        gl_format,
                        gl_type,
                        data,
                    );
                } else if target_ == GL_TEXTURE_2D_ARRAY
                    || target_ == GL_TEXTURE_2D_MULTISAMPLE_ARRAY
                    || target_ == GL_TEXTURE_CUBE_MAP
                    || target_ == GL_TEXTURE_CUBE_MAP_ARRAY
                {
                    ((*gl_info).gl_ops.ext.p_glClearTexSubImage.unwrap_unchecked())(
                        (*texture_gl).texture_rgb.name,
                        (base_level.wrapping_add(j)) as i32,
                        0,
                        0,
                        (base_layer.wrapping_add(i)) as i32,
                        (width) as i32,
                        (height) as i32,
                        1,
                        gl_format,
                        gl_type,
                        data,
                    );
                } else {
                    ((*gl_info).gl_ops.ext.p_glClearTexSubImage.unwrap_unchecked())(
                        (*texture_gl).texture_rgb.name,
                        (base_level.wrapping_add(j)) as i32,
                        0,
                        0,
                        0,
                        (width) as i32,
                        (height) as i32,
                        (depth) as i32,
                        gl_format,
                        gl_type,
                        data,
                    );
                }

                wined3d_texture_validate_location(
                    core::ptr::addr_of_mut!((*texture_gl).t),
                    sub_resource_idx,
                    WINED3D_LOCATION_TEXTURE_RGB,
                );
                wined3d_texture_invalidate_location(
                    core::ptr::addr_of_mut!((*texture_gl).t),
                    sub_resource_idx,
                    !WINED3D_LOCATION_TEXTURE_RGB,
                );
                j = j.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }

        return;
    }

    if (*gl_info).supported[ARB_CLEAR_BUFFER_OBJECT as usize] == 0 {
        FIXME!("OpenGL implementation does not support ARB_clear_buffer_object.\n");
        return;
    }

    format_gl = wined3d_format_gl((*view_gl).v.format);
    if (*format_gl).f.id as ::core::ffi::c_uint != WINED3DFMT_R32_UINT
        && (*format_gl).f.id as ::core::ffi::c_uint != WINED3DFMT_R32_SINT
        && (*format_gl).f.id as ::core::ffi::c_uint != WINED3DFMT_R32G32B32A32_UINT
        && (*format_gl).f.id as ::core::ffi::c_uint != WINED3DFMT_R32G32B32A32_SINT
    {
        FIXME!("Not implemented for format %s.\n", debug_d3dformat((*format_gl).f.id));
        return;
    }

    if fp {
        FIXME!("Floating-point buffer clears not implemented.\n");
        return;
    }

    buffer = buffer_from_resource(resource);
    get_buffer_view_range(
        buffer,
        core::ptr::addr_of!((*view_gl).v.desc),
        core::ptr::addr_of!((*format_gl).f),
        core::ptr::addr_of_mut!(offset),
        core::ptr::addr_of_mut!(size),
    );

    if offset == 0 && size == (*buffer).resource.size {
        wined3d_buffer_prepare_location(
            buffer,
            core::ptr::addr_of_mut!((*context_gl).c),
            WINED3D_LOCATION_BUFFER,
        );
    } else {
        wined3d_buffer_acquire_bo_for_write(buffer, core::ptr::addr_of_mut!((*context_gl).c));
        wined3d_buffer_load_location(
            buffer,
            core::ptr::addr_of_mut!((*context_gl).c),
            WINED3D_LOCATION_BUFFER,
        );
    }
    wined3d_unordered_access_view_invalidate_location(
        core::ptr::addr_of_mut!((*view_gl).v),
        !WINED3D_LOCATION_BUFFER,
    );

    bo_gl = wined3d_bo_gl((*buffer).buffer_object);
    wined3d_context_gl_bind_bo(context_gl, (*bo_gl).binding, (*bo_gl).id);
    ((*gl_info).gl_ops.ext.p_glClearBufferSubData.unwrap_unchecked())(
        (*bo_gl).binding,
        (*format_gl).internal as _,
        ((*bo_gl).b.buffer_offset.wrapping_add(offset as size_t)) as _,
        size as _,
        ((*format_gl).format) as _,
        ((*format_gl).type_) as _,
        clear_value as *const ::core::ffi::c_void,
    );
    wined3d_context_gl_reference_bo(context_gl, bo_gl);
    checkGLcall!(gl_info, "clear unordered access view");
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_unordered_access_view_set_counter(
    view: *mut wined3d_unordered_access_view,
    mut value: ::core::ffi::c_uint,
) {
    let mut dst: wined3d_bo_address = wined3d_bo_address {
        buffer_object: ::core::ptr::null_mut::<wined3d_bo>(),
        addr: ::core::ptr::null_mut::<BYTE>(),
    };
    let mut src: wined3d_bo_address = wined3d_bo_address {
        buffer_object: ::core::ptr::null_mut::<wined3d_bo>(),
        addr: ::core::ptr::null_mut::<BYTE>(),
    };
    let mut context: *mut wined3d_context = ::core::ptr::null_mut::<wined3d_context>();
    let mut range: wined3d_range = wined3d_range { offset: 0, size: 0 };

    if (*view).counter_bo.is_null() {
        return;
    }

    context = context_acquire(
        (*(*view).resource).device,
        ::core::ptr::null_mut::<wined3d_texture>(),
        0,
    );

    src.buffer_object = core::ptr::null_mut();
    src.addr = core::ptr::addr_of_mut!(value) as *mut ::core::ffi::c_void as *mut BYTE;

    dst.buffer_object = (*view).counter_bo;
    dst.addr = ::core::ptr::null_mut::<BYTE>();

    range.offset = 0;
    range.size = ::core::mem::size_of::<::core::ffi::c_uint>() as ::core::ffi::c_uint;

    wined3d_context_copy_bo_address(
        context,
        core::ptr::addr_of_mut!(dst),
        core::ptr::addr_of_mut!(src),
        1,
        core::ptr::addr_of_mut!(range),
        (WINED3D_MAP_WRITE | WINED3D_MAP_DISCARD) as uint32_t,
    );

    context_release(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_unordered_access_view_copy_counter(
    view: *mut wined3d_unordered_access_view,
    buffer: *mut wined3d_buffer,
    offset: ::core::ffi::c_uint,
    context: *mut wined3d_context,
) {
    let mut src: wined3d_const_bo_address = wined3d_const_bo_address {
        buffer_object: ::core::ptr::null_mut::<wined3d_bo>(),
        addr: ::core::ptr::null::<BYTE>(),
    };

    if (*view).counter_bo.is_null() {
        return;
    }

    src.buffer_object = (*view).counter_bo;
    src.addr = ::core::ptr::null::<BYTE>();

    wined3d_buffer_copy_bo_address(
        buffer,
        context,
        offset,
        core::ptr::addr_of_mut!(src),
        core::mem::size_of::<uint32_t>() as ::core::ffi::c_uint,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_unordered_access_view_gl_update(uav_gl: *mut wined3d_unordered_access_view_gl, context_gl: *mut wined3d_context_gl) {
    let buffer: *mut wined3d_buffer = buffer_from_resource((*uav_gl).v.resource);

    assert!(!(*uav_gl).bo_user.valid);
    create_buffer_view(
        core::ptr::addr_of_mut!((*uav_gl).gl_view),
        core::ptr::addr_of_mut!((*context_gl).c),
        core::ptr::addr_of_mut!((*uav_gl).v.desc),
        buffer,
        (*uav_gl).v.format,
    );
    (*uav_gl).bo_user.valid = true;
    list_add_head(
        core::ptr::addr_of_mut!((*(*buffer).buffer_object).users),
        core::ptr::addr_of_mut!((*uav_gl).bo_user.entry),
    );
}

unsafe extern "C" fn wined3d_unordered_access_view_gl_cs_init(object: *mut ::core::ffi::c_void) {
    let view_gl: *mut wined3d_unordered_access_view_gl = object as *mut wined3d_unordered_access_view_gl;
    let resource: *mut wined3d_resource = (*view_gl).v.resource;
    let desc: *mut wined3d_view_desc = core::ptr::addr_of_mut!((*view_gl).v.desc);
    let gl_info: *const wined3d_gl_info;

    TRACE!("view_gl %p.\n", view_gl);

    gl_info = core::ptr::addr_of!((*wined3d_adapter_gl((*(*resource).device).adapter)).gl_info);

    if (*resource).type_ as ::core::ffi::c_uint == WINED3D_RTYPE_BUFFER {
        let device_gl: *mut wined3d_device_gl = wined3d_device_gl((*resource).device);
        let buffer: *mut wined3d_buffer = buffer_from_resource(resource);
        let context_gl: *mut wined3d_context_gl;

        context_gl = wined3d_context_gl(context_acquire(
            core::ptr::addr_of_mut!((*device_gl).d),
            core::ptr::null_mut(),
            0,
        ));
        create_buffer_view(
            core::ptr::addr_of_mut!((*view_gl).gl_view),
            core::ptr::addr_of_mut!((*context_gl).c),
            desc,
            buffer,
            (*view_gl).v.format,
        );
        (*view_gl).bo_user.valid = true;
        list_add_head(
            core::ptr::addr_of_mut!((*(*buffer).buffer_object).users),
            core::ptr::addr_of_mut!((*view_gl).bo_user.entry),
        );
        if ((*desc).flags & (WINED3D_VIEW_BUFFER_COUNTER | WINED3D_VIEW_BUFFER_APPEND)) != 0 {
            let bo: *mut wined3d_bo_gl = core::ptr::addr_of_mut!((*view_gl).counter_bo);

            (*view_gl).v.counter_bo = core::ptr::addr_of_mut!((*bo).b);
            wined3d_device_gl_create_bo(
                device_gl,
                context_gl,
                core::mem::size_of::<u32>() as GLsizeiptr,
                GL_ATOMIC_COUNTER_BUFFER,
                GL_STATIC_DRAW,
                true,
                GL_MAP_READ_BIT | GL_MAP_WRITE_BIT | GL_CLIENT_STORAGE_BIT,
                bo,
            );
            wined3d_unordered_access_view_set_counter(core::ptr::addr_of_mut!((*view_gl).v), 0);
        }
        context_release(core::ptr::addr_of_mut!((*context_gl).c));
    } else {
        let texture_gl: *mut wined3d_texture_gl =
            wined3d_texture_gl(texture_from_resource(resource));
        let depth_or_layer_count: ::core::ffi::c_uint;

        if (*resource).type_ as ::core::ffi::c_uint == WINED3D_RTYPE_TEXTURE_3D {
            depth_or_layer_count = wined3d_texture_get_level_depth(
                core::ptr::addr_of_mut!((*texture_gl).t),
                (*desc).u.texture.level_idx,
            );
        } else {
            depth_or_layer_count = (*texture_gl).t.layer_count as ::core::ffi::c_uint;
        }

        if (*desc).u.texture.layer_idx != 0 || (*desc).u.texture.layer_count != depth_or_layer_count {
            create_texture_view(
                core::ptr::addr_of_mut!((*view_gl).gl_view),
                get_texture_view_target(gl_info, desc, texture_gl),
                desc,
                texture_gl,
                (*view_gl).v.format,
            );
        }
    }
}

unsafe extern "C" fn wined3d_unordered_access_view_init(
    mut view: *mut wined3d_unordered_access_view,
    mut desc: *const wined3d_view_desc,
    mut resource: *mut wined3d_resource,
    mut parent: *mut ::core::ffi::c_void,
    mut parent_ops: *const wined3d_parent_ops,
) -> HRESULT {
    (*view).refcount = 1 as LONG;
    (*view).parent = parent;
    (*view).parent_ops = parent_ops;
    if ((*resource).bind_flags & WINED3D_BIND_UNORDERED_ACCESS as ::core::ffi::c_uint) == 0 {
        return E_INVALIDARG;
    }
    (*view).format = validate_resource_view(desc, resource, (TRUE) as BOOL, (FALSE) as BOOL);
    if (*view).format.is_null() {
        return E_INVALIDARG;
    }
    (*view).desc = *desc;
    (*view).resource = resource;
    wined3d_resource_incref((*view).resource);
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_unordered_access_view_gl_init(view_gl: *mut wined3d_unordered_access_view_gl, desc: *const wined3d_view_desc, resource: *mut wined3d_resource, parent: *mut ::core::ffi::c_void, parent_ops: *const wined3d_parent_ops) -> HRESULT {
    let mut hr: HRESULT;

    hr = wined3d_unordered_access_view_init(
        core::ptr::addr_of_mut!((*view_gl).v),
        desc,
        resource,
        parent,
        parent_ops,
    );
    if hr < 0 {
        return hr;
    }

    list_init(core::ptr::addr_of_mut!((*view_gl).bo_user.entry));
    wined3d_cs_init_object(
        (*(*resource).device).cs as *mut wined3d_cs,
        Some(wined3d_unordered_access_view_gl_cs_init as unsafe extern "C" fn(*mut core::ffi::c_void)),
        view_gl as *mut core::ffi::c_void,
    );

    return hr;
}

unsafe extern "C" fn compile_hlsl_cs(
    hlsl: *const vkd3d_shader_code,
    dxbc: *mut vkd3d_shader_code,
) -> ::core::ffi::c_int {
    let mut hlsl_info: vkd3d_shader_hlsl_source_info = vkd3d_shader_hlsl_source_info {
        type_: VKD3D_SHADER_STRUCTURE_TYPE_HLSL_SOURCE_INFO,
        next: ::core::ptr::null::<::core::ffi::c_void>(),
        entry_point: ::core::ptr::null::<::core::ffi::c_char>(),
        secondary_code: vkd3d_shader_code {
            code: ::core::ptr::null::<::core::ffi::c_void>(),
            size: 0,
        },
        profile: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    let mut info: vkd3d_shader_compile_info = vkd3d_shader_compile_info {
        type_: VKD3D_SHADER_STRUCTURE_TYPE_COMPILE_INFO,
        next: ::core::ptr::null::<::core::ffi::c_void>(),
        source: vkd3d_shader_code {
            code: ::core::ptr::null::<::core::ffi::c_void>(),
            size: 0,
        },
        source_type: VKD3D_SHADER_SOURCE_HLSL,
        target_type: VKD3D_SHADER_TARGET_DXBC_TPF,
        options: ::core::ptr::null::<vkd3d_shader_compile_option>(),
        option_count: 0,
        log_level: VKD3D_SHADER_LOG_NONE,
        source_name: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    static mut options: [vkd3d_shader_compile_option; 1] = [vkd3d_shader_compile_option {
        name: VKD3D_SHADER_COMPILE_OPTION_API_VERSION,
        value: VKD3D_SHADER_API_VERSION_1_12,
    }];
    info.type_ = VKD3D_SHADER_STRUCTURE_TYPE_COMPILE_INFO;
    info.next = &raw mut hlsl_info as *const ::core::ffi::c_void;
    info.source = *hlsl;
    info.source_type = VKD3D_SHADER_SOURCE_HLSL;
    info.target_type = VKD3D_SHADER_TARGET_DXBC_TPF;
    info.options = &raw const options as *const vkd3d_shader_compile_option;
    info.option_count = ARRAY_SIZE!(options) as ::core::ffi::c_uint;
    info.log_level = VKD3D_SHADER_LOG_NONE;
    info.source_name = ::core::ptr::null::<::core::ffi::c_char>();
    hlsl_info.type_ = VKD3D_SHADER_STRUCTURE_TYPE_HLSL_SOURCE_INFO;
    hlsl_info.next = ::core::ptr::null::<::core::ffi::c_void>();
    hlsl_info.entry_point = b"main\0".as_ptr() as *const ::core::ffi::c_char;
    hlsl_info.secondary_code.code = ::core::ptr::null::<::core::ffi::c_void>();
    hlsl_info.secondary_code.size = 0;
    hlsl_info.profile = b"cs_5_0\0".as_ptr() as *const ::core::ffi::c_char;
    return vkd3d_shader_compile(
        &raw mut info,
        dxbc,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    );
}

unsafe extern "C" fn create_uav_pipeline(
    context_vk: *mut wined3d_context_vk,
    layout: *mut wined3d_pipeline_layout_vk,
    resource_filename: *const ::core::ffi::c_char,
    resource_type: wined3d_shader_resource_type,
) -> VkPipeline {
    let mut pipeline_info: VkComputePipelineCreateInfo = core::mem::zeroed();
    let mut shader_desc: wined3d_shader_desc = core::mem::zeroed();
    let mut vk_info: *const wined3d_vk_info = core::ptr::null();
    let mut code: vkd3d_shader_code = core::mem::zeroed();
    let mut dxbc: vkd3d_shader_code = core::mem::zeroed();
    let mut context: *mut wined3d_context = core::ptr::null_mut();
    let mut shader_module: VkShaderModule = core::mem::zeroed();
    let mut vk_device: VkDevice = core::ptr::null_mut();
    let mut resource_ptr: *mut ::core::ffi::c_void = core::ptr::null_mut();
    let mut result: VkPipeline = core::mem::zeroed();
    let mut global: HGLOBAL = core::ptr::null_mut();
    let mut module: HMODULE = core::ptr::null_mut();
    let mut resource: HRSRC = core::ptr::null_mut();
    let mut vr: VkResult = core::mem::zeroed();
    let mut ret: ::core::ffi::c_int = 0;

    vk_info = (*context_vk).vk_info;
    context = core::ptr::addr_of_mut!((*context_vk).c);

    if GetModuleHandleExA(
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        create_uav_pipeline as *const ::core::ffi::c_char,
        core::ptr::addr_of_mut!(module),
    ) == 0
    {
        ERR!("Failed to get a reference to the current module, last error %ld.\n", GetLastError());
        return (VK_NULL_HANDLE) as VkPipeline;
    }

    resource = FindResourceA(module, resource_filename as LPCSTR, RT_RCDATA as LPCSTR);
    if resource.is_null() {
        ERR!("Failed to retrieve resource, last error %ld.\n", GetLastError());
        return (VK_NULL_HANDLE) as VkPipeline;
    }

    global = LoadResource(module as HINSTANCE, resource);
    if global.is_null() {
        ERR!("Failed to load resource, last error %ld.\n", GetLastError());
        return (VK_NULL_HANDLE) as VkPipeline;
    }

    resource_ptr = LockResource(global) as *mut ::core::ffi::c_void;
    if resource_ptr.is_null() {
        ERR!("Failed to lock resource.\n");
        FreeResource((resource).cast());
        return (VK_NULL_HANDLE) as VkPipeline;
    }

    code.code = resource_ptr;
    code.size = SizeofResource(module as HINSTANCE, resource) as size_t;

    ret = compile_hlsl_cs(core::ptr::addr_of_mut!(code), core::ptr::addr_of_mut!(dxbc));
    if ret < 0 {
        ERR!("Failed to compile shader, ret %d.\n", ret);
        FreeResource((resource).cast());
        return (VK_NULL_HANDLE) as VkPipeline;
    }

    if FreeResource(resource as HGLOBAL) != 0 {
        ERR!("Failed to free resource.\n");
    }

    shader_desc.byte_code = dxbc.code as *const DWORD;
    shader_desc.byte_code_size = dxbc.size;

    shader_module = (*(*(*(*context).device).adapter).shader_backend)
        .shader_compile
        .unwrap_unchecked()(
        context,
        core::ptr::addr_of_mut!(shader_desc),
        WINED3D_SHADER_TYPE_COMPUTE,
    );
    if shader_module == (VK_NULL_HANDLE as VkShaderModule) {
        ERR!("Failed to create shader.\n");
        vkd3d_shader_free_shader_code(core::ptr::addr_of_mut!(dxbc));
        return (VK_NULL_HANDLE) as VkPipeline;
    }

    vkd3d_shader_free_shader_code(core::ptr::addr_of_mut!(dxbc));

    pipeline_info.sType = VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO;
    pipeline_info.pNext = core::ptr::null();
    pipeline_info.flags = 0;
    pipeline_info.stage.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
    pipeline_info.stage.pNext = core::ptr::null();
    pipeline_info.stage.flags = 0;
    pipeline_info.stage.stage = VK_SHADER_STAGE_COMPUTE_BIT;
    pipeline_info.stage.pName = b"main\0".as_ptr() as *const ::core::ffi::c_char;
    pipeline_info.stage.pSpecializationInfo = core::ptr::null();
    pipeline_info.stage.module = shader_module;
    pipeline_info.layout = (*layout).vk_pipeline_layout;
    pipeline_info.basePipelineHandle = (VK_NULL_HANDLE as _);
    pipeline_info.basePipelineIndex = -1;

    vk_device = (*wined3d_device_vk((*context).device)).vk_device;

    vr = ((*vk_info).vk_ops.vkCreateComputePipelines.unwrap_unchecked())(
        vk_device,
        (VK_NULL_HANDLE as _),
        1,
        core::ptr::addr_of_mut!(pipeline_info),
        core::ptr::null(),
        core::ptr::addr_of_mut!(result),
    );
    if (vr as ::core::ffi::c_int) < 0 {
        ERR!("Failed to create Vulkan compute pipeline, vr %s.\n", wined3d_debug_vkresult(vr));
        return (VK_NULL_HANDLE) as VkPipeline;
    }

    ((*vk_info).vk_ops.vkDestroyShaderModule.unwrap_unchecked())(vk_device, shader_module, core::ptr::null());
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_vk_uav_clear_state_init(device_vk: *mut wined3d_device_vk) {
    let state: *mut wined3d_uav_clear_state_vk = core::ptr::addr_of_mut!((*device_vk).uav_clear_state);
    let context_vk: *mut wined3d_context_vk = core::ptr::addr_of_mut!((*device_vk).context_vk);
    let mut vk_set_bindings: [VkDescriptorSetLayoutBinding; 2] = core::mem::zeroed();

    vk_set_bindings[0 as usize].binding = 0;
    vk_set_bindings[0 as usize].descriptorCount = 1;
    vk_set_bindings[0 as usize].stageFlags = VK_SHADER_STAGE_COMPUTE_BIT;
    vk_set_bindings[0 as usize].pImmutableSamplers = core::ptr::null();
    vk_set_bindings[0 as usize].descriptorType = VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER;

    vk_set_bindings[1 as usize].binding = 1;
    vk_set_bindings[1 as usize].descriptorCount = 1;
    vk_set_bindings[1 as usize].stageFlags = VK_SHADER_STAGE_COMPUTE_BIT;
    vk_set_bindings[1 as usize].pImmutableSamplers = core::ptr::null();

    vk_set_bindings[1 as usize].descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_IMAGE;
    (*state).image_layout = wined3d_context_vk_get_pipeline_layout(
        context_vk,
        &raw mut vk_set_bindings as *mut VkDescriptorSetLayoutBinding,
        2,
    );

    vk_set_bindings[1 as usize].descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER;
    (*state).buffer_layout = wined3d_context_vk_get_pipeline_layout(
        context_vk,
        &raw mut vk_set_bindings as *mut VkDescriptorSetLayoutBinding,
        2,
    );

    (*state).float_pipelines.buffer = create_uav_pipeline(
        context_vk,
        (*state).buffer_layout,
        b"cs_uav_clear_buffer_float_code.hlsl\0".as_ptr() as *const i8,
        WINED3D_SHADER_RESOURCE_BUFFER,
    );
    (*state).uint_pipelines.buffer = create_uav_pipeline(
        context_vk,
        (*state).buffer_layout,
        b"cs_uav_clear_buffer_uint_code.hlsl\0".as_ptr() as *const i8,
        WINED3D_SHADER_RESOURCE_BUFFER,
    );
    (*state).float_pipelines.image_1d = create_uav_pipeline(
        context_vk,
        (*state).image_layout,
        b"cs_uav_clear_1d_float_code.hlsl\0".as_ptr() as *const i8,
        WINED3D_SHADER_RESOURCE_TEXTURE_1D,
    );
    (*state).uint_pipelines.image_1d = create_uav_pipeline(
        context_vk,
        (*state).image_layout,
        b"cs_uav_clear_1d_uint_code.hlsl\0".as_ptr() as *const i8,
        WINED3D_SHADER_RESOURCE_TEXTURE_1D,
    );
    (*state).float_pipelines.image_1d_array = create_uav_pipeline(
        context_vk,
        (*state).image_layout,
        b"cs_uav_clear_1d_array_float_code.hlsl\0".as_ptr() as *const i8,
        WINED3D_SHADER_RESOURCE_TEXTURE_1DARRAY,
    );
    (*state).uint_pipelines.image_1d_array = create_uav_pipeline(
        context_vk,
        (*state).image_layout,
        b"cs_uav_clear_1d_array_uint_code.hlsl\0".as_ptr() as *const i8,
        WINED3D_SHADER_RESOURCE_TEXTURE_1DARRAY,
    );
    (*state).float_pipelines.image_2d = create_uav_pipeline(
        context_vk,
        (*state).image_layout,
        b"cs_uav_clear_2d_float_code.hlsl\0".as_ptr() as *const i8,
        WINED3D_SHADER_RESOURCE_TEXTURE_2D,
    );
    (*state).uint_pipelines.image_2d = create_uav_pipeline(
        context_vk,
        (*state).image_layout,
        b"cs_uav_clear_2d_uint_code.hlsl\0".as_ptr() as *const i8,
        WINED3D_SHADER_RESOURCE_TEXTURE_2D,
    );
    (*state).float_pipelines.image_2d_array = create_uav_pipeline(
        context_vk,
        (*state).image_layout,
        b"cs_uav_clear_2d_array_float_code.hlsl\0".as_ptr() as *const i8,
        WINED3D_SHADER_RESOURCE_TEXTURE_2DARRAY,
    );
    (*state).uint_pipelines.image_2d_array = create_uav_pipeline(
        context_vk,
        (*state).image_layout,
        b"cs_uav_clear_2d_array_uint_code.hlsl\0".as_ptr() as *const i8,
        WINED3D_SHADER_RESOURCE_TEXTURE_2DARRAY,
    );
    (*state).float_pipelines.image_3d = create_uav_pipeline(
        context_vk,
        (*state).image_layout,
        b"cs_uav_clear_3d_float_code.hlsl\0".as_ptr() as *const i8,
        WINED3D_SHADER_RESOURCE_TEXTURE_3D,
    );
    (*state).uint_pipelines.image_3d = create_uav_pipeline(
        context_vk,
        (*state).image_layout,
        b"cs_uav_clear_3d_uint_code.hlsl\0".as_ptr() as *const i8,
        WINED3D_SHADER_RESOURCE_TEXTURE_3D,
    );

    (*state).buffer_group_size.x = 128;
    (*state).buffer_group_size.y = 1;
    (*state).buffer_group_size.z = 1;
    (*state).image_1d_group_size.x = 64;
    (*state).image_1d_group_size.y = 1;
    (*state).image_1d_group_size.z = 1;
    (*state).image_1d_array_group_size.x = 64;
    (*state).image_1d_array_group_size.y = 1;
    (*state).image_1d_array_group_size.z = 1;
    (*state).image_2d_group_size.x = 8;
    (*state).image_2d_group_size.y = 8;
    (*state).image_2d_group_size.z = 1;
    (*state).image_2d_array_group_size.x = 8;
    (*state).image_2d_array_group_size.y = 8;
    (*state).image_2d_array_group_size.z = 1;
    (*state).image_3d_group_size.x = 8;
    (*state).image_3d_group_size.y = 8;
    (*state).image_3d_group_size.z = 1;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_vk_uav_clear_state_cleanup(device_vk: *mut wined3d_device_vk) {
    let state: *mut wined3d_uav_clear_state_vk = core::ptr::addr_of_mut!((*device_vk).uav_clear_state);
    let vk_info: *const wined3d_vk_info = core::ptr::addr_of!((*device_vk).vk_info);

    ((*vk_info).vk_ops.vkDestroyPipeline.unwrap_unchecked())((*device_vk).vk_device, (*state).float_pipelines.buffer, core::ptr::null_mut());
    ((*vk_info).vk_ops.vkDestroyPipeline.unwrap_unchecked())((*device_vk).vk_device, (*state).float_pipelines.image_1d, core::ptr::null_mut());
    ((*vk_info).vk_ops.vkDestroyPipeline.unwrap_unchecked())((*device_vk).vk_device, (*state).float_pipelines.image_1d_array, core::ptr::null_mut());
    ((*vk_info).vk_ops.vkDestroyPipeline.unwrap_unchecked())((*device_vk).vk_device, (*state).float_pipelines.image_2d, core::ptr::null_mut());
    ((*vk_info).vk_ops.vkDestroyPipeline.unwrap_unchecked())((*device_vk).vk_device, (*state).float_pipelines.image_2d_array, core::ptr::null_mut());
    ((*vk_info).vk_ops.vkDestroyPipeline.unwrap_unchecked())((*device_vk).vk_device, (*state).float_pipelines.image_3d, core::ptr::null_mut());

    ((*vk_info).vk_ops.vkDestroyPipeline.unwrap_unchecked())((*device_vk).vk_device, (*state).uint_pipelines.buffer, core::ptr::null_mut());
    ((*vk_info).vk_ops.vkDestroyPipeline.unwrap_unchecked())((*device_vk).vk_device, (*state).uint_pipelines.image_1d, core::ptr::null_mut());
    ((*vk_info).vk_ops.vkDestroyPipeline.unwrap_unchecked())((*device_vk).vk_device, (*state).uint_pipelines.image_1d_array, core::ptr::null_mut());
    ((*vk_info).vk_ops.vkDestroyPipeline.unwrap_unchecked())((*device_vk).vk_device, (*state).uint_pipelines.image_2d, core::ptr::null_mut());
    ((*vk_info).vk_ops.vkDestroyPipeline.unwrap_unchecked())((*device_vk).vk_device, (*state).uint_pipelines.image_2d_array, core::ptr::null_mut());
    ((*vk_info).vk_ops.vkDestroyPipeline.unwrap_unchecked())((*device_vk).vk_device, (*state).uint_pipelines.image_3d, core::ptr::null_mut());
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_unordered_access_view_vk_clear(view_vk: *mut wined3d_unordered_access_view_vk, clear_value: *const wined3d_uvec4, context_vk: *mut wined3d_context_vk, fp: bool) {
    let mut cb_source_address: wined3d_bo_address = core::mem::zeroed();
    let mut cb_destination_address: wined3d_bo_address = core::mem::zeroed();
    let view_format: *const wined3d_format = (*view_vk).v.format;
    let view_desc: *mut wined3d_view_desc = core::ptr::addr_of_mut!((*view_vk).v.desc);
    let mut constants: wined3d_uav_clear_constants_vk = core::mem::zeroed();
    let device: *mut wined3d_device = (*context_vk).c.device;
    let mut group_count: wined3d_shader_thread_group_size = core::mem::zeroed();
    let mut format_id: wined3d_format_id = (*view_format).id;
    let mut pipelines: *mut wined3d_uav_clear_pipelines_vk = core::ptr::null_mut();
    let mut texture_vk: *mut wined3d_texture_vk = core::ptr::null_mut();
    let mut layout: *mut wined3d_pipeline_layout_vk = core::ptr::null_mut();
    let mut state: *mut wined3d_uav_clear_state_vk = core::ptr::null_mut();
    let mut vk_info: *const wined3d_vk_info = core::ptr::null();
    let mut vk_image_info: VkDescriptorImageInfo = core::mem::zeroed();
    let mut device_vk: *mut wined3d_device_vk = core::ptr::null_mut();
    let mut buffer_info: VkDescriptorBufferInfo = core::mem::zeroed();
    let mut resource: *mut wined3d_resource = core::ptr::null_mut();
    let mut vk_command_buffer: VkCommandBuffer = core::ptr::null_mut();
    let mut constants_bo: wined3d_bo_vk = core::mem::zeroed();
    let mut vk_writes: [VkWriteDescriptorSet; 2] = core::mem::zeroed();
    let mut vk_buffer_view: VkBufferView = core::mem::zeroed();
    let mut range: wined3d_range = core::mem::zeroed();
    let mut vk_barrier: VkMemoryBarrier = core::mem::zeroed();
    let mut vk_pipeline: VkPipeline = core::mem::zeroed();
    let mut is_array: bool = false;

    device_vk = wined3d_device_vk(device);
    state = core::ptr::addr_of_mut!((*device_vk).uav_clear_state);
    pipelines = if fp {
        core::ptr::addr_of_mut!((*state).float_pipelines)
    } else {
        core::ptr::addr_of_mut!((*state).uint_pipelines)
    };

    resource = (*view_vk).v.resource;
    is_array = ((*view_desc).flags & WINED3D_VIEW_TEXTURE_ARRAY) != 0;

    if (*resource).type_ == WINED3D_RTYPE_BUFFER {
        vk_pipeline = (*pipelines).buffer;
        group_count = (*state).buffer_group_size;
    } else if (*resource).type_ == WINED3D_RTYPE_TEXTURE_1D {
        if is_array {
            vk_pipeline = (*pipelines).image_1d_array;
            group_count = (*state).image_1d_array_group_size;
        } else {
            vk_pipeline = (*pipelines).image_1d;
            group_count = (*state).image_1d_group_size;
        }
    } else if (*resource).type_ == WINED3D_RTYPE_TEXTURE_2D {
        if is_array {
            vk_pipeline = (*pipelines).image_2d_array;
            group_count = (*state).image_2d_array_group_size;
        } else {
            vk_pipeline = (*pipelines).image_2d;
            group_count = (*state).image_2d_group_size;
        }
    } else if (*resource).type_ == WINED3D_RTYPE_TEXTURE_3D {
        vk_pipeline = (*pipelines).image_3d;
        group_count = (*state).image_3d_group_size;
    } else {
        ERR!("Unhandled resource type %s.\n", debug_d3dresourcetype((*resource).type_));
        return;
    }

    if vk_pipeline == (VK_NULL_HANDLE as VkPipeline) {
        ERR!("Pipeline was not correctly initialized.\n");
        return;
    }

    vk_writes[0].sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
    vk_writes[0].pNext = core::ptr::null();
    vk_writes[0].dstBinding = 1;
    vk_writes[0].dstArrayElement = 0;
    vk_writes[0].descriptorCount = 1;
    vk_writes[0].pImageInfo = core::ptr::null();
    vk_writes[0].pTexelBufferView = core::ptr::addr_of_mut!(vk_buffer_view);
    vk_writes[0].pImageInfo = core::ptr::addr_of_mut!(vk_image_info);

    if (*resource).type_ == WINED3D_RTYPE_BUFFER {
        let buffer: *mut wined3d_buffer = buffer_from_resource(resource);
        let mut offset: core::ffi::c_uint = 0;
        let mut size: core::ffi::c_uint = 0;

        layout = (*state).buffer_layout;
        vk_writes[0].descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER;

        if (*buffer).structure_byte_stride != 0 {
            constants.extent.width = (*view_desc)
                .u
                .buffer
                .count
                .wrapping_mul((*buffer).structure_byte_stride)
                .wrapping_div(4) as _;
        } else {
            constants.extent.width = (*view_desc).u.buffer.count as _;
        }
        constants.extent.height = 1;

        get_buffer_view_range(buffer, view_desc, view_format, &mut offset, &mut size);
        if offset == 0 && size == (*buffer).resource.size as _ {
            wined3d_buffer_prepare_location(buffer, core::ptr::addr_of_mut!((*context_vk).c), WINED3D_LOCATION_BUFFER);
        } else {
            wined3d_buffer_acquire_bo_for_write(buffer, core::ptr::addr_of_mut!((*context_vk).c));
            wined3d_buffer_load_location(buffer, core::ptr::addr_of_mut!((*context_vk).c), WINED3D_LOCATION_BUFFER);
        }
        wined3d_buffer_validate_location(buffer, WINED3D_LOCATION_BUFFER);
        wined3d_buffer_invalidate_location(buffer, !WINED3D_LOCATION_BUFFER);
    } else {
        let mut layer_count: core::ffi::c_uint = 0;
        let mut level_count: core::ffi::c_uint = 0;
        let mut base_level: core::ffi::c_uint = 0;
        let mut base_layer: core::ffi::c_uint = 0;
        let mut i: core::ffi::c_uint = 0;
        let mut j: core::ffi::c_uint = 0;

        texture_vk = wined3d_texture_vk(wined3d_texture_from_resource(resource));

        layout = (*state).image_layout;
        vk_writes[0].descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_IMAGE;

        level_count = (*view_desc).u.texture.level_count;
        base_level = (*view_desc).u.texture.level_idx;
        if (*resource).type_ == WINED3D_RTYPE_TEXTURE_3D {
            layer_count = 1;
            base_layer = 0;
        } else {
            layer_count = (*view_desc).u.texture.layer_count;
            base_layer = (*view_desc).u.texture.layer_idx;
        }

        constants.extent.width = wined3d_texture_get_level_width(core::ptr::addr_of_mut!((*texture_vk).t), base_level) as _;
        constants.extent.height = wined3d_texture_get_level_height(core::ptr::addr_of_mut!((*texture_vk).t), base_level) as _;
        group_count.z = (*view_desc)
            .u
            .texture
            .layer_count
            .wrapping_add(group_count.z)
            .wrapping_sub(1)
            .wrapping_div(group_count.z);

        i = 0;
        while i < layer_count {
            j = 0;
            while j < level_count {
                let sub_resource_idx: core::ffi::c_uint = base_layer
                    .wrapping_add(i)
                    .wrapping_mul((*texture_vk).t.level_count)
                    .wrapping_add(base_level)
                    .wrapping_add(j);

                wined3d_texture_prepare_location(
                    core::ptr::addr_of_mut!((*texture_vk).t),
                    sub_resource_idx,
                    core::ptr::addr_of_mut!((*context_vk).c),
                    WINED3D_LOCATION_TEXTURE_RGB,
                );
                wined3d_texture_validate_location(
                    core::ptr::addr_of_mut!((*texture_vk).t),
                    sub_resource_idx,
                    WINED3D_LOCATION_TEXTURE_RGB,
                );
                wined3d_texture_invalidate_location(
                    core::ptr::addr_of_mut!((*texture_vk).t),
                    sub_resource_idx,
                    !WINED3D_LOCATION_TEXTURE_RGB,
                );
                j = j.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
    }

    group_count.x = constants
        .extent
        .width
        .wrapping_add(group_count.x as _)
        .wrapping_sub(1)
        .wrapping_div(group_count.x as _) as _;
    group_count.y = constants
        .extent
        .height
        .wrapping_add(group_count.y as _)
        .wrapping_sub(1)
        .wrapping_div(group_count.y as _) as _;

    constants.color.uint32[0] = (*clear_value).x as _;
    constants.color.uint32[1] = (*clear_value).y as _;
    constants.color.uint32[2] = (*clear_value).z as _;
    constants.color.uint32[3] = (*clear_value).w as _;

    if !fp {
        /* Make sure values are truncated, not saturated to some maximum value. */
        constants.color.uint32[0] &= wined3d_mask_from_size((*view_format).red_size as _);
        constants.color.uint32[1] &= wined3d_mask_from_size((*view_format).green_size as _);
        constants.color.uint32[2] &= wined3d_mask_from_size((*view_format).blue_size as _);
        constants.color.uint32[3] &= wined3d_mask_from_size((*view_format).alpha_size as _);

        if (*view_format).id == WINED3DFMT_R11G11B10_FLOAT {
            constants.color.uint32[0] |=
                constants.color.uint32[1] << 11;
            constants.color.uint32[0] |=
                constants.color.uint32[2] << 22;
            format_id = WINED3DFMT_R32_UINT;
        } else {
            format_id = wined3d_get_typed_format_id(
                (*(*context_vk).c.device).adapter,
                view_format,
                WINED3D_CHANNEL_TYPE_UINT,
            );
        }
    }

    'out: {
        if format_id == WINED3DFMT_UNKNOWN {
            ERR!("Unsupported format %s.\n", debug_d3dformat((*view_format).id));
            return;
        }

        if (*resource).type_ == WINED3D_RTYPE_BUFFER {
            if format_id == (*view_format).id {
                vk_buffer_view = (*view_vk).view_vk.u.vk_buffer_view;
            } else {
                vk_buffer_view = wined3d_view_vk_create_vk_buffer_view(
                    context_vk,
                    view_desc,
                    wined3d_buffer_vk(buffer_from_resource(resource)),
                    wined3d_format_vk(wined3d_get_format(
                        (*(*context_vk).c.device).adapter,
                        format_id,
                        WINED3D_BIND_UNORDERED_ACCESS,
                    )),
                );
                if vk_buffer_view == (VK_NULL_HANDLE as VkBufferView) {
                    return;
                }
            }
        } else {
            if format_id == (*view_format).id {
                vk_image_info = (*view_vk).view_vk.u.vk_image_info;
                if vk_image_info.imageView == 0 {
                    let mut default_info: *const VkDescriptorImageInfo = core::ptr::null();
                    default_info = wined3d_texture_vk_get_default_image_info(texture_vk, context_vk);
                    if default_info.is_null() {
                        return;
                    }
                    vk_image_info = *default_info;
                }
            } else {
                vk_image_info.sampler = (VK_NULL_HANDLE as _);
                vk_image_info.imageLayout = (*texture_vk).layout;
                vk_image_info.imageView = wined3d_view_vk_create_vk_image_view(
                    context_vk,
                    view_desc,
                    texture_vk,
                    wined3d_format_vk(wined3d_get_format(
                        (*(*context_vk).c.device).adapter,
                        format_id,
                        WINED3D_BIND_UNORDERED_ACCESS,
                    )),
                    COLOR_FIXUP_IDENTITY,
                    false,
                    VK_IMAGE_USAGE_STORAGE_BIT,
                );

                if vk_image_info.imageView == (VK_NULL_HANDLE as _) {
                    return;
                }
            }
        }

        vk_writes[1].sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
        vk_writes[1].pNext = core::ptr::null();
        vk_writes[1].dstBinding = 0;
        vk_writes[1].dstArrayElement = 0;
        vk_writes[1].descriptorCount = 1;
        vk_writes[1].pImageInfo = core::ptr::null();
        vk_writes[1].pTexelBufferView = core::ptr::null();
        vk_writes[1].descriptorType = VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER;
        vk_writes[1].pBufferInfo = core::ptr::addr_of_mut!(buffer_info);

        if wined3d_context_vk_create_bo(
            context_vk,
            core::mem::size_of::<wined3d_uav_clear_constants_vk>() as _,
            VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT,
            VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT,
            core::ptr::addr_of_mut!(constants_bo),
        ) == 0
        {
            ERR!("Failed to create constants BO.\n");
            break 'out;
        }

        cb_source_address.buffer_object = core::ptr::null_mut();
        cb_source_address.addr = core::ptr::addr_of_mut!(constants) as *mut BYTE;
        cb_destination_address.buffer_object = core::ptr::addr_of_mut!(constants_bo.b);
        cb_destination_address.addr = core::ptr::null_mut();

        range.offset = 0;
        range.size = core::mem::size_of::<wined3d_uav_clear_constants_vk>() as _;
        adapter_vk_copy_bo_address(
            core::ptr::addr_of_mut!((*context_vk).c),
            core::ptr::addr_of_mut!(cb_destination_address),
            core::ptr::addr_of_mut!(cb_source_address),
            1,
            core::ptr::addr_of_mut!(range),
            WINED3D_MAP_WRITE | WINED3D_MAP_DISCARD,
        );

        buffer_info.buffer = constants_bo.vk_buffer;
        buffer_info.range = constants_bo.size;
        buffer_info.offset = constants_bo.b.buffer_offset as _;

        vk_info = (*context_vk).vk_info;

        vk_writes[0].dstSet =
            wined3d_context_vk_create_vk_descriptor_set(context_vk, (*layout).vk_set_layout);
        if vk_writes[0].dstSet == 0 {
            ERR!("Failed to create descriptor set.\n");
            wined3d_context_vk_destroy_bo(context_vk, core::ptr::addr_of_mut!(constants_bo));
            break 'out;
        }

        vk_writes[1].dstSet = vk_writes[0].dstSet;

        ((*vk_info).vk_ops.vkUpdateDescriptorSets.unwrap_unchecked())(
            (*device_vk).vk_device,
            2,
            vk_writes.as_mut_ptr(),
            0,
            core::ptr::null(),
        );

        vk_command_buffer = wined3d_context_vk_get_command_buffer(context_vk);
        wined3d_context_vk_end_current_render_pass(context_vk);

        wined3d_context_vk_reference_unordered_access_view(context_vk, view_vk);
        wined3d_context_vk_reference_bo(context_vk, core::ptr::addr_of_mut!(constants_bo));
        wined3d_context_vk_destroy_bo(context_vk, core::ptr::addr_of_mut!(constants_bo));

        vk_barrier.sType = VK_STRUCTURE_TYPE_MEMORY_BARRIER;
        vk_barrier.pNext = core::ptr::null();
        vk_barrier.srcAccessMask = vk_access_mask_from_bind_flags((*resource).bind_flags as _);
        vk_barrier.dstAccessMask = VK_ACCESS_SHADER_WRITE_BIT;

        ((*vk_info).vk_ops.vkCmdPipelineBarrier.unwrap_unchecked())(
            vk_command_buffer,
            vk_pipeline_stage_mask_from_bind_flags((*resource).bind_flags as _),
            VK_PIPELINE_STAGE_COMPUTE_SHADER_BIT,
            0,
            1,
            core::ptr::addr_of_mut!(vk_barrier),
            0,
            core::ptr::null(),
            0,
            core::ptr::null(),
        );
        ((*vk_info).vk_ops.vkCmdBindPipeline.unwrap_unchecked())(
            vk_command_buffer,
            VK_PIPELINE_BIND_POINT_COMPUTE,
            vk_pipeline,
        );
        ((*vk_info).vk_ops.vkCmdBindDescriptorSets.unwrap_unchecked())(
            vk_command_buffer,
            VK_PIPELINE_BIND_POINT_COMPUTE,
            (*layout).vk_pipeline_layout,
            0,
            1,
            core::ptr::addr_of_mut!(vk_writes[0].dstSet),
            0,
            core::ptr::null(),
        );
        ((*vk_info).vk_ops.vkCmdDispatch.unwrap_unchecked())(
            vk_command_buffer,
            group_count.x,
            group_count.y,
            group_count.z,
        );

        vk_barrier.srcAccessMask = VK_ACCESS_SHADER_WRITE_BIT;
        vk_barrier.dstAccessMask = vk_access_mask_from_bind_flags((*resource).bind_flags as _);

        ((*vk_info).vk_ops.vkCmdPipelineBarrier.unwrap_unchecked())(
            vk_command_buffer,
            VK_PIPELINE_STAGE_COMPUTE_SHADER_BIT,
            vk_pipeline_stage_mask_from_bind_flags((*resource).bind_flags as _),
            0,
            1,
            core::ptr::addr_of_mut!(vk_barrier),
            0,
            core::ptr::null(),
            0,
            core::ptr::null(),
        );

        context_invalidate_compute_state(core::ptr::addr_of_mut!((*context_vk).c), (STATE_COMPUTE_SHADER) as DWORD);
    }

    // out:
    if format_id != (*view_format).id {
        if (*resource).type_ == WINED3D_RTYPE_BUFFER {
            wined3d_context_vk_destroy_vk_buffer_view(
                context_vk,
                vk_buffer_view,
                (*view_vk).view_vk.command_buffer_id,
            );
        } else {
            wined3d_context_vk_destroy_vk_image_view(
                context_vk,
                vk_image_info.imageView,
                (*view_vk).view_vk.command_buffer_id,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_unordered_access_view_vk_update(
    uav_vk: *mut wined3d_unordered_access_view_vk,
    context_vk: *mut wined3d_context_vk,
) {
    let view_format_vk: *const wined3d_format_vk = wined3d_format_vk((*uav_vk).v.format);
    let desc: *const wined3d_view_desc = core::ptr::addr_of!((*uav_vk).v.desc);
    let resource: *mut wined3d_resource = (*uav_vk).v.resource;
    let view_vk: *mut wined3d_view_vk = core::ptr::addr_of_mut!((*uav_vk).view_vk);
    let mut buffer_vk: *mut wined3d_buffer_vk = core::ptr::null_mut();
    let mut vk_buffer_view: VkBufferView = 0;

    assert!(!(*view_vk).bo_user.valid);

    buffer_vk = wined3d_buffer_vk(buffer_from_resource(resource));
    wined3d_context_vk_destroy_vk_buffer_view(context_vk, (*view_vk).u.vk_buffer_view, (*view_vk).command_buffer_id);
    vk_buffer_view = wined3d_view_vk_create_vk_buffer_view(context_vk, desc, buffer_vk, view_format_vk);
    if vk_buffer_view != 0 {
        (*view_vk).u.vk_buffer_view = vk_buffer_view;
        (*view_vk).bo_user.valid = true;
        list_add_head(
            core::ptr::addr_of_mut!((*(*buffer_vk).b.buffer_object).users),
            core::ptr::addr_of_mut!((*view_vk).bo_user.entry),
        );
    }
}

unsafe extern "C" fn wined3d_unordered_access_view_vk_cs_init(object: *mut ::core::ffi::c_void) {
    let uav_vk: *mut wined3d_unordered_access_view_vk = object as *mut wined3d_unordered_access_view_vk;
    let view_vk: *mut wined3d_view_vk = core::ptr::addr_of_mut!((*uav_vk).view_vk);
    let desc: *mut wined3d_view_desc = core::ptr::addr_of_mut!((*uav_vk).v.desc);
    let mut format_vk: *const wined3d_format_vk = core::ptr::null::<wined3d_format_vk>();
    let mut vk_info: *const wined3d_vk_info = core::ptr::null::<wined3d_vk_info>();
    let mut texture_vk: *mut wined3d_texture_vk = core::ptr::null_mut::<wined3d_texture_vk>();
    let mut context_vk: *mut wined3d_context_vk = core::ptr::null_mut::<wined3d_context_vk>();
    let mut device_vk: *mut wined3d_device_vk = core::ptr::null_mut::<wined3d_device_vk>();
    let mut buffer_vk: *mut wined3d_buffer_vk = core::ptr::null_mut::<wined3d_buffer_vk>();
    let mut create_info: VkBufferViewCreateInfo = core::mem::zeroed::<VkBufferViewCreateInfo>();
    let mut resource: *mut wined3d_resource = core::ptr::null_mut::<wined3d_resource>();
    let mut vk_buffer_view: VkBufferView = 0;
    let mut default_flags: uint32_t = 0;
    let mut vk_image_view: VkImageView = 0;
    let mut vr: VkResult = 0;

    resource = (*uav_vk).v.resource;
    device_vk = wined3d_device_vk((*resource).device);
    format_vk = wined3d_format_vk((*uav_vk).v.format);

    if (*resource).type_ as ::core::ffi::c_uint == WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint {
        buffer_vk = wined3d_buffer_vk(buffer_from_resource(resource));

        context_vk = wined3d_context_vk(context_acquire(
            core::ptr::addr_of_mut!((*device_vk).d),
            core::ptr::null_mut::<wined3d_texture>(),
            0 as ::core::ffi::c_uint,
        ));
        vk_info = (*context_vk).vk_info;

        vk_buffer_view = wined3d_view_vk_create_vk_buffer_view(context_vk, desc, buffer_vk, format_vk);
        if vk_buffer_view != 0 {
            let bo: *mut wined3d_bo_vk = wined3d_bo_vk((*buffer_vk).b.buffer_object);

            (*uav_vk).view_vk.u.vk_buffer_view = vk_buffer_view;
            (*uav_vk).view_vk.bo_user.valid = true;
            list_add_head(
                core::ptr::addr_of_mut!((*bo).b.users),
                core::ptr::addr_of_mut!((*view_vk).bo_user.entry),
            );
        }

        if (*desc).flags & (WINED3D_VIEW_BUFFER_COUNTER | WINED3D_VIEW_BUFFER_APPEND) as ::core::ffi::c_uint != 0 {
            if wined3d_context_vk_create_bo(
                context_vk,
                core::mem::size_of::<uint32_t>() as VkDeviceSize,
                (VK_BUFFER_USAGE_TRANSFER_SRC_BIT
                    | VK_BUFFER_USAGE_TRANSFER_DST_BIT
                    | VK_BUFFER_USAGE_STORAGE_TEXEL_BUFFER_BIT) as VkBufferUsageFlags,
                VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT as VkMemoryPropertyFlags,
                core::ptr::addr_of_mut!((*uav_vk).counter_bo),
            ) == 0
            {
                ERR!("Failed to create counter bo.\n");
                context_release(core::ptr::addr_of_mut!((*context_vk).c));

                return;
            }

            wined3d_context_vk_end_current_render_pass(context_vk);
            ((*vk_info).vk_ops.vkCmdFillBuffer.unwrap_unchecked())(
                wined3d_context_vk_get_command_buffer(context_vk),
                (*uav_vk).counter_bo.vk_buffer,
                (*uav_vk).counter_bo.b.buffer_offset as VkDeviceSize,
                core::mem::size_of::<uint32_t>() as VkDeviceSize,
                0 as uint32_t,
            );
            wined3d_context_vk_reference_bo(context_vk, core::ptr::addr_of_mut!((*uav_vk).counter_bo));

            create_info.sType = VK_STRUCTURE_TYPE_BUFFER_VIEW_CREATE_INFO as VkStructureType;
            create_info.pNext = core::ptr::null::<::core::ffi::c_void>();
            create_info.flags = 0 as VkBufferViewCreateFlags;
            create_info.buffer = (*uav_vk).counter_bo.vk_buffer;
            create_info.format = VK_FORMAT_R32_UINT as VkFormat;
            create_info.offset = (*uav_vk).counter_bo.b.buffer_offset as VkDeviceSize;
            create_info.range = core::mem::size_of::<uint32_t>() as usize as VkDeviceSize;
            vr = ((*vk_info).vk_ops.vkCreateBufferView.unwrap_unchecked())(
                (*device_vk).vk_device,
                core::ptr::addr_of_mut!(create_info),
                core::ptr::null::<VkAllocationCallbacks>(),
                core::ptr::addr_of_mut!((*uav_vk).vk_counter_view),
            );
            if (vr as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                ERR!(
                    "Failed to create counter buffer view, vr %s.\n",
                    wined3d_debug_vkresult(vr)
                );
            } else {
                (*uav_vk).v.counter_bo = core::ptr::addr_of_mut!((*uav_vk).counter_bo.b);
            }
        }

        context_release(core::ptr::addr_of_mut!((*context_vk).c));

        return;
    }

    texture_vk = wined3d_texture_vk(texture_from_resource(resource));

    if (*texture_vk).t.layer_count > 1 as UINT {
        default_flags |= WINED3D_VIEW_TEXTURE_ARRAY as uint32_t;
    }

    if (*(*resource).format).id as ::core::ffi::c_uint == (*format_vk).f.id as ::core::ffi::c_uint
        && (*desc).flags as uint32_t == default_flags
        && (*desc).u.texture.level_idx == 0
        && (*desc).u.texture.level_count == (*texture_vk).t.level_count
        && (*desc).u.texture.layer_idx == 0
        && (*desc).u.texture.layer_count == (*texture_vk).t.layer_count
        && ((*resource).bind_flags & WINED3D_BIND_DEPTH_STENCIL as ::core::ffi::c_uint) == 0
        && ((*resource).type_ as ::core::ffi::c_uint) != WINED3D_RTYPE_TEXTURE_3D as ::core::ffi::c_uint
    {
        return;
    }

    if !(*texture_vk).t.swapchain.is_null()
        && (*(*texture_vk).t.swapchain).state.desc.backbuffer_count > 1 as ::core::ffi::c_uint
    {
        FIXME!("Swapchain unordered access views not supported.\n");
    }

    context_vk = wined3d_context_vk(context_acquire(
        core::ptr::addr_of_mut!((*device_vk).d),
        core::ptr::null_mut::<wined3d_texture>(),
        0 as ::core::ffi::c_uint,
    ));
    vk_image_view = wined3d_view_vk_create_vk_image_view(
        context_vk,
        desc,
        texture_vk,
        format_vk,
        (*format_vk).f.color_fixup,
        false,
        VK_IMAGE_USAGE_STORAGE_BIT as VkImageUsageFlags,
    );
    context_release(core::ptr::addr_of_mut!((*context_vk).c));

    if vk_image_view == 0 {
        return;
    }

    (*view_vk).u.vk_image_info.imageView = vk_image_view;
    (*view_vk).u.vk_image_info.sampler = VK_NULL_HANDLE as VkSampler;
    (*view_vk).u.vk_image_info.imageLayout = (*texture_vk).layout as VkImageLayout;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_unordered_access_view_vk_init(view_vk: *mut wined3d_unordered_access_view_vk, desc: *const wined3d_view_desc, resource: *mut wined3d_resource, parent: *mut ::core::ffi::c_void, parent_ops: *const wined3d_parent_ops) -> HRESULT {
    let mut hr: HRESULT;

    hr = wined3d_unordered_access_view_init(
        core::ptr::addr_of_mut!((*view_vk).v),
        desc,
        resource,
        parent,
        parent_ops,
    );
    if hr < 0 {
        return hr;
    }

    list_init(core::ptr::addr_of_mut!((*view_vk).view_vk.bo_user.entry));
    wined3d_cs_init_object(
        (*(*resource).device).cs as *mut wined3d_cs,
        Some(
            wined3d_unordered_access_view_vk_cs_init
                as unsafe extern "C" fn(*mut core::ffi::c_void),
        ),
        view_vk as *mut core::ffi::c_void,
    );

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_unordered_access_view_create(
    mut desc: *const wined3d_view_desc,
    mut resource: *mut wined3d_resource,
    mut parent: *mut core::ffi::c_void,
    mut parent_ops: *const wined3d_parent_ops,
    mut view: *mut *mut wined3d_unordered_access_view,
) -> HRESULT {
    let mut adapter_ops: *const wined3d_adapter_ops = core::ptr::null::<wined3d_adapter_ops>();
    WINE_TRACE!("desc %s, resource %p, parent %p, parent_ops %p, view %p.\n", wined3d_debug_view_desc(desc, resource), resource, parent, parent_ops, view);
    adapter_ops = (*(*(*resource).device).adapter).adapter_ops;
    return (*adapter_ops)
        .adapter_create_unordered_access_view
        .unwrap_unchecked()(desc, resource, parent, parent_ops, view);
}
