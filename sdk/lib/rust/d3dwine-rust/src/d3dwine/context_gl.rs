use crate::d3dwine::context::context_invalidate_compute_state;
use crate::d3dwine::context::context_invalidate_state;
use crate::d3dwine::context::context_update_stream_info;
use crate::d3dwine::buffer::wined3d_buffer_acquire_bo_for_write;
use crate::d3dwine::buffer::wined3d_buffer_invalidate_location;
use crate::d3dwine::buffer::wined3d_buffer_load;
use crate::d3dwine::buffer::wined3d_buffer_load_location;
use crate::d3dwine::buffer::wined3d_buffer_load_sysmem;
use crate::d3dwine::context::wined3d_context_cleanup;
use crate::d3dwine::context::wined3d_context_init;
use crate::prelude::*;
use core::ffi::c_void;


// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_51 {
    pub target: GLenum,
    pub binding: GLenum,
    pub str_0: *const ::core::ffi::c_char,
    pub extension: wined3d_gl_extension,
}

static mut wined3d_context_tls_idx: DWORD = 0;

// WINE_DEFAULT_DEBUG_CHANNEL(d3d) / WINE_DECLARE_DEBUG_CHANNEL(d3d_perf) / WINE_DECLARE_DEBUG_CHANNEL(d3d_sync):
// debug-channel registrations; handled by the crate's DPRINT/TRACE macros, no Rust item emitted here.

pub const WINED3D_MAX_FBO_ENTRIES: u32 = 64;

pub const WINED3D_ALL_LAYERS: u32 = u32::MAX;

pub type context_fbo_entry_func_t =
    Option<unsafe extern "C" fn(context_gl: *mut wined3d_context_gl, entry: *mut fbo_entry)>;

unsafe extern "C" fn gl_primitive_type_from_d3d(
    mut primitive_type: wined3d_primitive_type,
) -> GLenum {
    match primitive_type as ::core::ffi::c_uint {
        WINED3D_PT_POINTLIST => return GL_POINTS as GLenum,
        WINED3D_PT_LINELIST => return GL_LINES as GLenum,
        WINED3D_PT_LINESTRIP => return GL_LINE_STRIP as GLenum,
        WINED3D_PT_TRIANGLELIST => return GL_TRIANGLES as GLenum,
        WINED3D_PT_TRIANGLESTRIP => return GL_TRIANGLE_STRIP as GLenum,
        WINED3D_PT_TRIANGLEFAN => return GL_TRIANGLE_FAN as GLenum,
        WINED3D_PT_LINELIST_ADJ => return GL_LINES_ADJACENCY_ARB as GLenum,
        WINED3D_PT_LINESTRIP_ADJ => return GL_LINE_STRIP_ADJACENCY_ARB as GLenum,
        WINED3D_PT_TRIANGLELIST_ADJ => return GL_TRIANGLES_ADJACENCY_ARB as GLenum,
        WINED3D_PT_TRIANGLESTRIP_ADJ => return GL_TRIANGLE_STRIP_ADJACENCY_ARB as GLenum,
        WINED3D_PT_PATCH => return GL_PATCHES as GLenum,
        WINED3D_PT_UNDEFINED => {}
        _ => {
            FIXME!(
                "Unhandled primitive type %s.\n",
                debug_d3dprimitivetype(primitive_type)
            );
            return !(0 as GLenum);
        }
    }
    return !(0 as GLenum);
}

unsafe extern "C" fn wined3d_context_gl_bind_fbo(context_gl: *mut wined3d_context_gl, target: GLenum, fbo: GLuint) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;

    match target {
        GL_READ_FRAMEBUFFER => {
            if (*context_gl).fbo_read_binding == fbo {
                return;
            }
            (*context_gl).fbo_read_binding = fbo;
        }

        GL_DRAW_FRAMEBUFFER => {
            if (*context_gl).fbo_draw_binding == fbo {
                return;
            }
            (*context_gl).fbo_draw_binding = fbo;
        }

        GL_FRAMEBUFFER => {
            if (*context_gl).fbo_read_binding == fbo && (*context_gl).fbo_draw_binding == fbo {
                return;
            }
            (*context_gl).fbo_read_binding = fbo;
            (*context_gl).fbo_draw_binding = fbo;
        }

        _ => {
            FIXME!("Unhandled target %#x.\n", target);
        }
    }

    (*gl_info)
        .fbo_ops
        .glBindFramebuffer
        .unwrap_unchecked()(target, fbo);
    checkGLcall!(gl_info, b"glBindFramebuffer()\0".as_ptr() as *const i8);
}

unsafe extern "C" fn context_clean_fbo_attachments(gl_info: *const wined3d_gl_info, target: GLenum) {
    let mut i: ::core::ffi::c_uint;
    i = 0 as ::core::ffi::c_uint;
    while i < (*gl_info).limits.buffers {
        (*gl_info)
            .fbo_ops
            .glFramebufferTexture2D
            .unwrap_unchecked()(
            target,
            (GL_COLOR_ATTACHMENT0 as GLenum).wrapping_add(i as GLenum),
            GL_TEXTURE_2D as GLenum,
            0 as GLuint,
            0 as GLint,
        );
        checkGLcall!(gl_info, b"glFramebufferTexture2D()\0".as_ptr() as *const i8);
        i = i.wrapping_add(1);
    }
    (*gl_info)
        .fbo_ops
        .glFramebufferTexture2D
        .unwrap_unchecked()(
        target,
        GL_DEPTH_ATTACHMENT as GLenum,
        GL_TEXTURE_2D as GLenum,
        0 as GLuint,
        0 as GLint,
    );
    checkGLcall!(gl_info, b"glFramebufferTexture2D()\0".as_ptr() as *const i8);

    (*gl_info)
        .fbo_ops
        .glFramebufferTexture2D
        .unwrap_unchecked()(
        target,
        GL_STENCIL_ATTACHMENT as GLenum,
        GL_TEXTURE_2D as GLenum,
        0 as GLuint,
        0 as GLint,
    );
    checkGLcall!(gl_info, b"glFramebufferTexture2D()\0".as_ptr() as *const i8);
}

unsafe extern "C" fn wined3d_context_gl_destroy_fbo(
    context_gl: *mut wined3d_context_gl,
    mut fbo: GLuint,
) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;

    wined3d_context_gl_bind_fbo(context_gl, GL_FRAMEBUFFER, fbo);
    context_clean_fbo_attachments(gl_info, GL_FRAMEBUFFER);
    wined3d_context_gl_bind_fbo(context_gl, GL_FRAMEBUFFER, 0);

    (*gl_info)
        .fbo_ops
        .glDeleteFramebuffers
        .unwrap_unchecked()(
        1,
        core::ptr::addr_of_mut!(fbo),
    );
    checkGLcall!(gl_info, "glDeleteFramebuffers()");
}

unsafe extern "C" fn context_attach_depth_stencil_rb(
    gl_info: *const wined3d_gl_info,
    fbo_target: GLenum,
    flags: uint32_t,
    rb: GLuint,
) {
    if flags & WINED3D_FBO_ENTRY_FLAG_DEPTH != 0 {
        ((*gl_info).fbo_ops.glFramebufferRenderbuffer.unwrap_unchecked())(
            fbo_target,
            GL_DEPTH_ATTACHMENT,
            GL_RENDERBUFFER,
            rb,
        );
        checkGLcall!(gl_info, b"glFramebufferRenderbuffer()\0".as_ptr() as *const ::core::ffi::c_char);
    }

    if flags & WINED3D_FBO_ENTRY_FLAG_STENCIL != 0 {
        ((*gl_info).fbo_ops.glFramebufferRenderbuffer.unwrap_unchecked())(
            fbo_target,
            GL_STENCIL_ATTACHMENT,
            GL_RENDERBUFFER,
            rb,
        );
        checkGLcall!(gl_info, b"glFramebufferRenderbuffer()\0".as_ptr() as *const ::core::ffi::c_char);
    }
}

unsafe extern "C" fn wined3d_context_gl_attach_gl_texture_fbo(
    mut context_gl: *mut wined3d_context_gl,
    mut fbo_target: GLenum,
    mut attachment: GLenum,
    mut resource: *const wined3d_fbo_resource,
) {
    let mut gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    if resource.is_null() {
        (*gl_info)
            .fbo_ops
            .glFramebufferTexture2D
            .unwrap_unchecked()(
            fbo_target,
            attachment,
            GL_TEXTURE_2D as GLenum,
            0 as GLuint,
            0 as GLint,
        );
    } else if (*resource).layer == WINED3D_ALL_LAYERS as u32 {
        if (*gl_info).fbo_ops.glFramebufferTexture.is_none() {
            FIXME!("OpenGL implementation doesn't support glFramebufferTexture().\n");
            return;
        }
        (*gl_info)
            .fbo_ops
            .glFramebufferTexture
            .unwrap_unchecked()(
            fbo_target,
            attachment,
            (*resource).object,
            (*resource).level as GLint,
        );
    } else if (*resource).target == GL_TEXTURE_1D_ARRAY as GLenum
        || (*resource).target == GL_TEXTURE_2D_ARRAY as GLenum
        || (*resource).target == GL_TEXTURE_3D as GLenum
    {
        if (*gl_info).fbo_ops.glFramebufferTextureLayer.is_none() {
            FIXME!("OpenGL implementation doesn't support glFramebufferTextureLayer().\n");
            return;
        }
        (*gl_info)
            .fbo_ops
            .glFramebufferTextureLayer
            .unwrap_unchecked()(
            fbo_target,
            attachment,
            (*resource).object,
            (*resource).level as GLint,
            (*resource).layer as GLint,
        );
    } else if (*resource).target == GL_TEXTURE_1D as GLenum {
        (*gl_info)
            .fbo_ops
            .glFramebufferTexture1D
            .unwrap_unchecked()(
            fbo_target,
            attachment,
            (*resource).target,
            (*resource).object,
            (*resource).level as GLint,
        );
    } else {
        (*gl_info)
            .fbo_ops
            .glFramebufferTexture2D
            .unwrap_unchecked()(
            fbo_target,
            attachment,
            (*resource).target,
            (*resource).object,
            (*resource).level as GLint,
        );
    }
    checkGLcall!(gl_info, b"attach texture to fbo\0".as_ptr() as *const ::core::ffi::c_char);
}

unsafe extern "C" fn wined3d_context_gl_attach_depth_stencil_fbo(
    mut context_gl: *mut wined3d_context_gl,
    mut fbo_target: GLenum,
    mut resource: *const wined3d_fbo_resource,
    mut rb_namespace: BOOL,
    mut flags: uint32_t,
) {
    let mut gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    if (*resource).object != 0 {
        WINE_TRACE!("Attach depth stencil %u.\n", (*resource).object);
        if rb_namespace != 0 {
            context_attach_depth_stencil_rb(gl_info, fbo_target, flags, (*resource).object);
        } else {
            if (flags & (WINED3D_FBO_ENTRY_FLAG_DEPTH as uint32_t)) != 0 {
                wined3d_context_gl_attach_gl_texture_fbo(
                    context_gl,
                    fbo_target,
                    (GL_DEPTH_ATTACHMENT as GLenum),
                    resource,
                );
            }
            if (flags & (WINED3D_FBO_ENTRY_FLAG_STENCIL as uint32_t)) != 0 {
                wined3d_context_gl_attach_gl_texture_fbo(
                    context_gl,
                    fbo_target,
                    (GL_STENCIL_ATTACHMENT as GLenum),
                    resource,
                );
            }
        }
        if (flags & (WINED3D_FBO_ENTRY_FLAG_DEPTH as uint32_t)) == 0 {
            wined3d_context_gl_attach_gl_texture_fbo(
                context_gl,
                fbo_target,
                (GL_DEPTH_ATTACHMENT as GLenum),
                ::core::ptr::null::<wined3d_fbo_resource>(),
            );
        }
        if (flags & (WINED3D_FBO_ENTRY_FLAG_STENCIL as uint32_t)) == 0 {
            wined3d_context_gl_attach_gl_texture_fbo(
                context_gl,
                fbo_target,
                (GL_STENCIL_ATTACHMENT as GLenum),
                ::core::ptr::null::<wined3d_fbo_resource>(),
            );
        }
    } else {
        WINE_TRACE!("Attach depth stencil 0.\n");
        wined3d_context_gl_attach_gl_texture_fbo(
            context_gl,
            fbo_target,
            (GL_DEPTH_ATTACHMENT as GLenum),
            ::core::ptr::null::<wined3d_fbo_resource>(),
        );
        wined3d_context_gl_attach_gl_texture_fbo(
            context_gl,
            fbo_target,
            (GL_STENCIL_ATTACHMENT as GLenum),
            ::core::ptr::null::<wined3d_fbo_resource>(),
        );
    }
}

unsafe extern "C" fn wined3d_context_gl_attach_surface_fbo(
    context_gl: *mut wined3d_context_gl,
    fbo_target: GLenum,
    idx: ::core::ffi::c_uint,
    resource: *const wined3d_fbo_resource,
    rb_namespace: BOOL,
) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;

    TRACE!("Attach GL object %u to %u.\n", (*resource).object, idx);

    if (*resource).object != 0 {
        if rb_namespace != 0 {
            (*gl_info)
                .fbo_ops
                .glFramebufferRenderbuffer
                .unwrap_unchecked()(
                fbo_target,
                (GL_COLOR_ATTACHMENT0 as GLenum).wrapping_add(idx as GLenum),
                GL_RENDERBUFFER as GLenum,
                (*resource).object,
            );
            checkGLcall!(gl_info, "glFramebufferRenderbuffer()");
        } else {
            wined3d_context_gl_attach_gl_texture_fbo(
                context_gl,
                fbo_target,
                (GL_COLOR_ATTACHMENT0 as GLenum).wrapping_add(idx as GLenum),
                resource,
            );
        }
    } else {
        wined3d_context_gl_attach_gl_texture_fbo(
            context_gl,
            fbo_target,
            (GL_COLOR_ATTACHMENT0 as GLenum).wrapping_add(idx as GLenum),
            core::ptr::null::<wined3d_fbo_resource>(),
        );
    }
}

unsafe extern "C" fn context_dump_fbo_attachment(
    gl_info: *const wined3d_gl_info,
    target: GLenum,
    attachment: GLenum,
) {
    static mut texture_type: [C2RustUnnamed_51; 7] = [
        C2RustUnnamed_51 {
            target: GL_TEXTURE_1D as GLenum,
            binding: GL_TEXTURE_BINDING_1D as GLenum,
            str_0: b"1d\0" as *const u8 as *const ::core::ffi::c_char,
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_51 {
            target: GL_TEXTURE_1D_ARRAY as GLenum,
            binding: GL_TEXTURE_BINDING_1D_ARRAY as GLenum,
            str_0: b"1d-array\0" as *const u8 as *const ::core::ffi::c_char,
            extension: EXT_TEXTURE_ARRAY,
        },
        C2RustUnnamed_51 {
            target: GL_TEXTURE_2D as GLenum,
            binding: GL_TEXTURE_BINDING_2D as GLenum,
            str_0: b"2d\0" as *const u8 as *const ::core::ffi::c_char,
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_51 {
            target: GL_TEXTURE_2D_ARRAY as GLenum,
            binding: GL_TEXTURE_BINDING_2D_ARRAY as GLenum,
            str_0: b"2d-array\0" as *const u8 as *const ::core::ffi::c_char,
            extension: EXT_TEXTURE_ARRAY,
        },
        C2RustUnnamed_51 {
            target: GL_TEXTURE_CUBE_MAP as GLenum,
            binding: GL_TEXTURE_BINDING_CUBE_MAP as GLenum,
            str_0: b"cube\0" as *const u8 as *const ::core::ffi::c_char,
            extension: ARB_TEXTURE_CUBE_MAP,
        },
        C2RustUnnamed_51 {
            target: GL_TEXTURE_2D_MULTISAMPLE as GLenum,
            binding: GL_TEXTURE_BINDING_2D_MULTISAMPLE as GLenum,
            str_0: b"2d-ms\0" as *const u8 as *const ::core::ffi::c_char,
            extension: ARB_TEXTURE_MULTISAMPLE,
        },
        C2RustUnnamed_51 {
            target: GL_TEXTURE_2D_MULTISAMPLE_ARRAY as GLenum,
            binding: GL_TEXTURE_BINDING_2D_MULTISAMPLE_ARRAY as GLenum,
            str_0: b"2d-array-ms\0" as *const u8 as *const ::core::ffi::c_char,
            extension: ARB_TEXTURE_MULTISAMPLE,
        },
    ];
    let mut type_0: GLint = 0;
    let mut name: GLint = 0;
    let mut samples: GLint = 0;
    let mut width: GLint = 0;
    let mut height: GLint = 0;
    let mut old_texture: GLint = 0;
    let mut level: GLint = 0;
    let mut face: GLint = 0;
    let mut fmt: GLint = 0;
    let mut tex_target: GLint = 0;
    let mut tex_type_str: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut i: ::core::ffi::c_uint = 0;

    (*gl_info)
        .fbo_ops
        .glGetFramebufferAttachmentParameteriv
        .unwrap_unchecked()(
        target,
        attachment,
        GL_FRAMEBUFFER_ATTACHMENT_OBJECT_NAME as GLenum,
        &raw mut name,
    );
    (*gl_info)
        .fbo_ops
        .glGetFramebufferAttachmentParameteriv
        .unwrap_unchecked()(
        target,
        attachment,
        GL_FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE as GLenum,
        &raw mut type_0,
    );

    if type_0 == GL_RENDERBUFFER as GLint {
        (*gl_info)
            .fbo_ops
            .glBindRenderbuffer
            .unwrap_unchecked()(
            GL_RENDERBUFFER as GLenum,
            name as GLuint,
        );
        (*gl_info)
            .fbo_ops
            .glGetRenderbufferParameteriv
            .unwrap_unchecked()(
            GL_RENDERBUFFER as GLenum,
            GL_RENDERBUFFER_WIDTH as GLenum,
            &raw mut width,
        );
        (*gl_info)
            .fbo_ops
            .glGetRenderbufferParameteriv
            .unwrap_unchecked()(
            GL_RENDERBUFFER as GLenum,
            GL_RENDERBUFFER_HEIGHT as GLenum,
            &raw mut height,
        );
        if (*gl_info).limits.samples > 1 as ::core::ffi::c_uint {
            (*gl_info)
                .fbo_ops
                .glGetRenderbufferParameteriv
                .unwrap_unchecked()(
                GL_RENDERBUFFER as GLenum,
                GL_RENDERBUFFER_SAMPLES as GLenum,
                &raw mut samples,
            );
        } else {
            samples = 1 as GLint;
        }
        (*gl_info)
            .fbo_ops
            .glGetRenderbufferParameteriv
            .unwrap_unchecked()(
            GL_RENDERBUFFER as GLenum,
            GL_RENDERBUFFER_INTERNAL_FORMAT as GLenum,
            &raw mut fmt,
        );
    } else if type_0 == GL_TEXTURE as GLint {
        (*gl_info)
            .fbo_ops
            .glGetFramebufferAttachmentParameteriv
            .unwrap_unchecked()(
            target,
            attachment,
            GL_FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL as GLenum,
            &raw mut level,
        );
        (*gl_info)
            .fbo_ops
            .glGetFramebufferAttachmentParameteriv
            .unwrap_unchecked()(
            target,
            attachment,
            GL_FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE as GLenum,
            &raw mut face,
        );

        if (*gl_info).gl_ops.ext.p_glGetTextureParameteriv.is_some() {
            ((*gl_info).gl_ops.ext.p_glGetTextureParameteriv.unwrap_unchecked())(
                name as GLuint,
                GL_TEXTURE_TARGET as GLenum,
                &raw mut tex_target,
            );

            i = 0 as ::core::ffi::c_uint;
            while (i as usize) < ARRAY_SIZE!(texture_type) as usize {
                if texture_type[i as usize].target == tex_target as GLenum {
                    tex_type_str = texture_type[i as usize].str_0;
                    break;
                } else {
                    i = i.wrapping_add(1);
                }
            }
            if i as usize == ARRAY_SIZE!(texture_type) as usize {
                tex_type_str = wine_dbg_sprintf(
                    b"%#x\0" as *const u8 as *const ::core::ffi::c_char,
                    tex_target,
                );
            }
        } else if face != 0 {
            (*gl_info)
                .gl_ops
                .gl
                .p_glGetIntegerv
                .unwrap_unchecked()(
                GL_TEXTURE_BINDING_CUBE_MAP as GLenum,
                &raw mut old_texture,
            );
            (*gl_info)
                .gl_ops
                .gl
                .p_glBindTexture
                .unwrap_unchecked()(
                GL_TEXTURE_CUBE_MAP as GLenum,
                name as GLuint,
            );

            tex_target = GL_TEXTURE_CUBE_MAP as GLint;
            tex_type_str = b"cube\0" as *const u8 as *const ::core::ffi::c_char;
        } else {
            i = 0 as ::core::ffi::c_uint;
            while (i as usize) < ARRAY_SIZE!(texture_type) as usize {
                if !((*gl_info).supported[texture_type[i as usize].extension as usize] == 0) {
                    (*gl_info)
                        .gl_ops
                        .gl
                        .p_glGetIntegerv
                        .unwrap_unchecked()(
                        texture_type[i as usize].binding,
                        &raw mut old_texture,
                    );
                    while (*gl_info)
                        .gl_ops
                        .gl
                        .p_glGetError
                        .unwrap_unchecked()()
                        != 0
                    {}

                    (*gl_info)
                        .gl_ops
                        .gl
                        .p_glBindTexture
                        .unwrap_unchecked()(
                        texture_type[i as usize].target,
                        name as GLuint,
                    );
                    if (*gl_info)
                        .gl_ops
                        .gl
                        .p_glGetError
                        .unwrap_unchecked()()
                        == 0
                    {
                        tex_target = texture_type[i as usize].target as GLint;
                        tex_type_str = texture_type[i as usize].str_0;
                        break;
                    } else {
                        (*gl_info)
                            .gl_ops
                            .gl
                            .p_glBindTexture
                            .unwrap_unchecked()(
                            texture_type[i as usize].target,
                            old_texture as GLuint,
                        );
                    }
                }
                i = i.wrapping_add(1);
            }

            if tex_type_str.is_null() {
                return;
            }
        }

        if (*gl_info).gl_ops.ext.p_glGetTextureParameteriv.is_some() {
            ((*gl_info).gl_ops.ext.p_glGetTextureLevelParameteriv.unwrap_unchecked())(
                name as GLuint,
                (level as GLuint) as i32,
                GL_TEXTURE_INTERNAL_FORMAT as GLenum,
                &raw mut fmt,
            );
            ((*gl_info).gl_ops.ext.p_glGetTextureLevelParameteriv.unwrap_unchecked())(
                name as GLuint,
                (level as GLuint) as i32,
                GL_TEXTURE_WIDTH as GLenum,
                &raw mut width,
            );
            ((*gl_info).gl_ops.ext.p_glGetTextureLevelParameteriv.unwrap_unchecked())(
                name as GLuint,
                (level as GLuint) as i32,
                GL_TEXTURE_HEIGHT as GLenum,
                &raw mut height,
            );
            ((*gl_info).gl_ops.ext.p_glGetTextureLevelParameteriv.unwrap_unchecked())(
                name as GLuint,
                (level as GLuint) as i32,
                GL_TEXTURE_SAMPLES as GLenum,
                &raw mut samples,
            );
        } else {
            (*gl_info)
                .gl_ops
                .gl
                .p_glGetTexLevelParameteriv
                .unwrap_unchecked()(
                tex_target as GLenum,
                level,
                GL_TEXTURE_INTERNAL_FORMAT as GLenum,
                &raw mut fmt,
            );
            (*gl_info)
                .gl_ops
                .gl
                .p_glGetTexLevelParameteriv
                .unwrap_unchecked()(
                tex_target as GLenum,
                level,
                GL_TEXTURE_WIDTH as GLenum,
                &raw mut width,
            );
            (*gl_info)
                .gl_ops
                .gl
                .p_glGetTexLevelParameteriv
                .unwrap_unchecked()(
                tex_target as GLenum,
                level,
                GL_TEXTURE_HEIGHT as GLenum,
                &raw mut height,
            );
            if (*gl_info).supported[ARB_TEXTURE_MULTISAMPLE as usize] != 0 {
                (*gl_info)
                    .gl_ops
                    .gl
                    .p_glGetTexLevelParameteriv
                    .unwrap_unchecked()(
                    tex_target as GLenum,
                    level,
                    GL_TEXTURE_SAMPLES as GLenum,
                    &raw mut samples,
                );
            } else {
                samples = 1 as GLint;
            }

            (*gl_info)
                .gl_ops
                .gl
                .p_glBindTexture
                .unwrap_unchecked()(
                tex_target as GLenum,
                old_texture as GLuint,
            );
        }
    } else if type_0 == GL_NONE as GLint {
    } else {
        ERR!(
            "    %s: Unknown attachment %#x.\n",
            debug_fboattachment(attachment),
            type_0
        );
    }

    checkGLcall!(gl_info, b"dump FBO attachment\0" as *const u8 as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_check_fbo_status(context_gl: *const wined3d_context_gl, target: GLenum) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut status: GLenum = 0;

    if FIXME_ON!(__wine_dbch_d3d) == 0 {
        return;
    }

    status = (*gl_info)
        .fbo_ops
        .glCheckFramebufferStatus
        .unwrap_unchecked()(target);
    if status == GL_FRAMEBUFFER_COMPLETE as GLenum {
        TRACE!("FBO complete.\n");
    } else {
        let mut i: core::ffi::c_uint = 0;

        FIXME!("FBO status %s (%#x).\n", debug_fbostatus(status), status);

        if (*context_gl).current_fbo.is_null() {
            ERR!("FBO 0 is incomplete, driver bug?\n");
            return;
        }

        context_dump_fbo_attachment(gl_info, target, GL_DEPTH_ATTACHMENT as GLenum);
        context_dump_fbo_attachment(gl_info, target, GL_STENCIL_ATTACHMENT as GLenum);

        i = 0 as core::ffi::c_uint;
        while i < (*gl_info).limits.buffers {
            context_dump_fbo_attachment(
                gl_info,
                target,
                (GL_COLOR_ATTACHMENT0 as GLenum).wrapping_add(i as GLenum),
            );
            i = i.wrapping_add(1);
        }
    }
}

unsafe extern "C" fn context_generate_rt_mask(mut buffer: GLenum) -> DWORD {
    return if buffer != 0 {
        ((1 as DWORD) << (31 as DWORD)) | buffer as DWORD
    } else {
        0 as DWORD
    };
}

unsafe extern "C" fn context_generate_rt_mask_from_resource(resource: *mut wined3d_resource) -> DWORD {
    if (*resource).type_ as DWORD != WINED3D_RTYPE_TEXTURE_2D {
        FIXME!(
            "Not implemented for %s resources.\n",
            debug_d3dresourcetype((*resource).type_)
        );
        return 0;
    }

    return ((1 as DWORD) << 31)
        | wined3d_texture_get_gl_buffer(texture_from_resource(resource)) as DWORD;
}

unsafe extern "C" fn wined3d_context_gl_set_draw_buffer(context_gl: *mut wined3d_context_gl, buffer: GLenum) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let current_fbo: *mut fbo_entry = (*context_gl).current_fbo;
    let new_mask: uint32_t = context_generate_rt_mask(buffer) as uint32_t;
    let mut current_mask: *mut uint32_t;

    current_mask = if !current_fbo.is_null() {
        core::ptr::addr_of_mut!((*current_fbo).rt_mask)
    } else {
        core::ptr::addr_of_mut!((*context_gl).draw_buffers_mask)
    };
    if new_mask == *current_mask {
        return;
    }
    (*gl_info).gl_ops.gl.p_glDrawBuffer.unwrap_unchecked()(buffer);
    checkGLcall!(gl_info, "glDrawBuffer()");

    *current_mask = new_mask;
}

unsafe extern "C" fn wined3d_context_gl_set_fbo_key_for_render_target(
    context_gl: *const wined3d_context_gl,
    key: *mut wined3d_fbo_entry_key,
    idx: ::core::ffi::c_uint,
    render_target: *const wined3d_rendertarget_info,
    location: DWORD,
) {
    let mut sub_resource_idx: ::core::ffi::c_uint = (*render_target).sub_resource_idx;
    let mut resource: *mut wined3d_resource = (*render_target).resource;
    let mut texture_gl: *mut wined3d_texture_gl = ::core::ptr::null_mut::<wined3d_texture_gl>();
    if resource.is_null()
        || (*(*resource).format).id as ::core::ffi::c_uint
            == WINED3DFMT_NULL as ::core::ffi::c_uint
        || (*resource).type_ as ::core::ffi::c_uint
            == WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint
    {
        if !resource.is_null()
            && (*resource).type_ as ::core::ffi::c_uint
                == WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint
        {
            FIXME!(
                "Not implemented for %s resources.\n",
                debug_d3dresourcetype((*resource).type_)
            );
        }
        (*key).objects[idx as usize].object = 0 as GLuint;
        (*key).objects[idx as usize].target = 0 as GLenum;
        (*key).objects[idx as usize].layer = 0 as GLuint;
        (*key).objects[idx as usize].level = (*key).objects[idx as usize].layer;
        return;
    }
    if (*render_target).gl_view.name != 0 {
        (*key).objects[idx as usize].object = (*render_target).gl_view.name;
        (*key).objects[idx as usize].target = (*render_target).gl_view.target;
        (*key).objects[idx as usize].level = 0 as GLuint;
        (*key).objects[idx as usize].layer =
            (if (*render_target).layer_count == 1 as ::core::ffi::c_uint {
                0 as ::core::ffi::c_uint
            } else {
                WINED3D_ALL_LAYERS as ::core::ffi::c_uint
            }) as GLuint;
        return;
    }
    texture_gl = wined3d_texture_gl(wined3d_texture_from_resource(resource));
    if !(*texture_gl).current_renderbuffer.is_null() {
        (*key).objects[idx as usize].object = (*(*texture_gl).current_renderbuffer).id;
        (*key).objects[idx as usize].target = 0 as GLenum;
        (*key).objects[idx as usize].layer = 0 as GLuint;
        (*key).objects[idx as usize].level = (*key).objects[idx as usize].layer;
        (*key).rb_namespace |=
            ((1 as ::core::ffi::c_int) << idx) as uint32_t;
        return;
    }
    (*key).objects[idx as usize].target =
        wined3d_texture_gl_get_sub_resource_target(texture_gl, sub_resource_idx);
    (*key).objects[idx as usize].level =
        sub_resource_idx.wrapping_rem((*texture_gl).t.level_count) as GLuint;
    (*key).objects[idx as usize].layer =
        sub_resource_idx.wrapping_div((*texture_gl).t.level_count) as GLuint;
    if (*render_target).layer_count != 1 as ::core::ffi::c_uint {
        (*key).objects[idx as usize].layer = WINED3D_ALL_LAYERS as GLuint;
    }
    match location {
        WINED3D_LOCATION_TEXTURE_RGB => {
            (*key).objects[idx as usize].object = wined3d_texture_gl_get_texture_name(
                texture_gl,
                core::ptr::addr_of!((*context_gl).c),
                FALSE != 0,
            );
        }
        WINED3D_LOCATION_TEXTURE_SRGB => {
            (*key).objects[idx as usize].object = wined3d_texture_gl_get_texture_name(
                texture_gl,
                core::ptr::addr_of!((*context_gl).c),
                TRUE != 0,
            );
        }
        WINED3D_LOCATION_RB_MULTISAMPLE => {
            (*key).objects[idx as usize].object = (*texture_gl).rb_multisample;
            (*key).objects[idx as usize].target = 0 as GLenum;
            (*key).objects[idx as usize].layer = 0 as GLuint;
            (*key).objects[idx as usize].level = (*key).objects[idx as usize].layer;
            (*key).rb_namespace |=
                ((1 as ::core::ffi::c_int) << idx) as uint32_t;
        }
        WINED3D_LOCATION_RB_RESOLVED => {
            (*key).objects[idx as usize].object = (*texture_gl).rb_resolved;
            (*key).objects[idx as usize].target = 0 as GLenum;
            (*key).objects[idx as usize].layer = 0 as GLuint;
            (*key).objects[idx as usize].level = (*key).objects[idx as usize].layer;
            (*key).rb_namespace |=
                ((1 as ::core::ffi::c_int) << idx) as uint32_t;
        }
        _ => {}
    }
}

unsafe extern "C" fn wined3d_context_gl_generate_fbo_key(
    context_gl: *const wined3d_context_gl,
    key: *mut wined3d_fbo_entry_key,
    render_targets: *const wined3d_rendertarget_info,
    depth_stencil: *const wined3d_rendertarget_info,
    color_location: DWORD,
    ds_location: DWORD,
) {
    let buffers: ::core::ffi::c_uint = (*(*context_gl).gl_info).limits.buffers;
    let mut i: ::core::ffi::c_uint = 0;

    (*key).rb_namespace = 0;
    wined3d_context_gl_set_fbo_key_for_render_target(context_gl, key, 0, depth_stencil, ds_location);

    i = 0;
    while i < buffers {
        wined3d_context_gl_set_fbo_key_for_render_target(
            context_gl,
            key,
            i.wrapping_add(1 as ::core::ffi::c_uint),
            render_targets.add(i as usize),
            color_location,
        );
        i = i.wrapping_add(1);
    }

    memset(
        (core::ptr::addr_of_mut!((*key).objects) as *mut wined3d_fbo_resource)
            .add(buffers.wrapping_add(1 as ::core::ffi::c_uint) as usize)
            as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        (ARRAY_SIZE!((*key).objects) as usize)
            .wrapping_sub(buffers as usize)
            .wrapping_sub(1 as usize)
            .wrapping_mul(core::mem::size_of::<wined3d_fbo_resource>()),
    );
}

unsafe extern "C" fn wined3d_context_gl_create_fbo_entry(
    context_gl: *const wined3d_context_gl,
    render_targets: *const wined3d_rendertarget_info,
    depth_stencil: *const wined3d_rendertarget_info,
    color_location: DWORD,
    ds_location: DWORD,
) -> *mut fbo_entry {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut entry: *mut fbo_entry = core::ptr::null_mut::<fbo_entry>();

    entry = malloc(core::mem::size_of::<fbo_entry>()) as *mut fbo_entry;
    wined3d_context_gl_generate_fbo_key(
        context_gl,
        core::ptr::addr_of_mut!((*entry).key) as *mut wined3d_fbo_entry_key,
        render_targets,
        depth_stencil,
        color_location,
        ds_location,
    );
    (*entry).flags = 0 as u32;
    if !(*depth_stencil).resource.is_null() {
        if (*(*(*depth_stencil).resource).format).depth_size != 0 {
            (*entry).flags |= WINED3D_FBO_ENTRY_FLAG_DEPTH as u32;
        }
        if (*(*(*depth_stencil).resource).format).stencil_size != 0 {
            (*entry).flags |= WINED3D_FBO_ENTRY_FLAG_STENCIL as u32;
        }
    }
    (*entry).rt_mask = context_generate_rt_mask(GL_COLOR_ATTACHMENT0) as u32;
    ((*gl_info).fbo_ops.glGenFramebuffers
        .unwrap_unchecked())(
        1,
        core::ptr::addr_of_mut!((*entry).id),
    );
    checkGLcall!(gl_info, "glGenFramebuffers()");

    return entry;
}

unsafe extern "C" fn wined3d_context_gl_reuse_fbo_entry(
    context_gl: *mut wined3d_context_gl,
    target: GLenum,
    render_targets: *const wined3d_rendertarget_info,
    depth_stencil: *const wined3d_rendertarget_info,
    color_location: DWORD,
    ds_location: DWORD,
    entry: *mut fbo_entry,
) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;

    wined3d_context_gl_bind_fbo(context_gl, target, (*entry).id);
    context_clean_fbo_attachments(gl_info, target);

    wined3d_context_gl_generate_fbo_key(
        context_gl,
        core::ptr::addr_of_mut!((*entry).key) as *mut wined3d_fbo_entry_key,
        render_targets,
        depth_stencil,
        color_location,
        ds_location,
    );
    (*entry).flags = 0;
    if !(*depth_stencil).resource.is_null() {
        if (*(*(*depth_stencil).resource).format).depth_size != 0 {
            (*entry).flags |= WINED3D_FBO_ENTRY_FLAG_DEPTH;
        }
        if (*(*(*depth_stencil).resource).format).stencil_size != 0 {
            (*entry).flags |= WINED3D_FBO_ENTRY_FLAG_STENCIL;
        }
    }
}

unsafe extern "C" fn wined3d_context_gl_destroy_fbo_entry(
    mut context_gl: *mut wined3d_context_gl,
    mut entry: *mut fbo_entry,
) {
    if (*entry).id != 0 {
        WINE_TRACE!("Destroy FBO %u.\n", (*entry).id);
        wined3d_context_gl_destroy_fbo(context_gl, (*entry).id);
    }
    (*context_gl).fbo_entry_count = (*context_gl).fbo_entry_count.wrapping_sub(1);
    list_remove(core::ptr::addr_of_mut!((*entry).entry));
    free(entry as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn wined3d_context_gl_find_fbo_entry(
    context_gl: *mut wined3d_context_gl,
    target: GLenum,
    render_targets: *const wined3d_rendertarget_info,
    mut depth_stencil: *const wined3d_rendertarget_info,
    color_location: DWORD,
    ds_location: DWORD,
) -> *mut fbo_entry {
    static mut ds_null: wined3d_rendertarget_info = wined3d_rendertarget_info {
        gl_view: wined3d_gl_view {
            target: 0 as GLenum,
            name: 0,
        },
        resource: ::core::ptr::null::<wined3d_resource>() as *mut wined3d_resource,
        sub_resource_idx: 0,
        layer_count: 0,
    };
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut rt_texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut ds_texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut fbo_key: wined3d_fbo_entry_key = core::mem::zeroed();
    let mut i: core::ffi::c_uint = 0;
    let mut ds_level: core::ffi::c_uint = 0;
    let mut rt_level: core::ffi::c_uint = 0;
    let mut entry: *mut fbo_entry = core::ptr::null_mut();

    if !(*depth_stencil).resource.is_null()
        && (*(*depth_stencil).resource).type_ as core::ffi::c_uint
            != WINED3D_RTYPE_BUFFER as core::ffi::c_uint
        && !(*render_targets.offset(0 as isize))
            .resource
            .is_null()
        && (*(*render_targets.offset(0 as isize)).resource).type_
            as core::ffi::c_uint
            != WINED3D_RTYPE_BUFFER as core::ffi::c_uint
        && (*(*(*render_targets.offset(0 as isize)).resource).format).id
            as core::ffi::c_uint
            != WINED3DFMT_NULL as core::ffi::c_uint
    {
        rt_texture = wined3d_texture_from_resource(
            (*render_targets.offset(0 as isize)).resource,
        );
        rt_level = (*render_targets.offset(0 as isize))
            .sub_resource_idx
            .wrapping_rem((*rt_texture).level_count);
        ds_texture = wined3d_texture_from_resource((*depth_stencil).resource);
        ds_level = (*depth_stencil)
            .sub_resource_idx
            .wrapping_rem((*ds_texture).level_count);

        if wined3d_texture_get_level_width(ds_texture, ds_level)
            < wined3d_texture_get_level_width(rt_texture, rt_level)
            || wined3d_texture_get_level_height(ds_texture, ds_level)
                < wined3d_texture_get_level_height(rt_texture, rt_level)
        {
            WARN!("Depth stencil is smaller than the primary color buffer, disabling.\n");
            depth_stencil = core::ptr::addr_of!(ds_null);
        } else if (*ds_texture).resource.multisample_type as core::ffi::c_uint
            != (*rt_texture).resource.multisample_type as core::ffi::c_uint
            || ((*ds_texture).resource.multisample_type as core::ffi::c_uint != 0
                && (*ds_texture).resource.multisample_quality
                    != (*rt_texture).resource.multisample_quality)
        {
            WARN!(
                "Color multisample type %u and quality %u, depth stencil has %u and %u, disabling ds buffer.\n",
                (*rt_texture).resource.multisample_type as core::ffi::c_uint,
                (*rt_texture).resource.multisample_quality,
                (*ds_texture).resource.multisample_type as core::ffi::c_uint,
                (*ds_texture).resource.multisample_quality,
            );
            depth_stencil = core::ptr::addr_of!(ds_null);
        } else if (*(*depth_stencil).resource).type_ as core::ffi::c_uint
            == WINED3D_RTYPE_TEXTURE_2D as core::ffi::c_uint
        {
            wined3d_texture_gl_set_compatible_renderbuffer(
                wined3d_texture_gl(ds_texture),
                context_gl,
                ds_level,
                render_targets.offset(0 as isize)
                    as *const wined3d_rendertarget_info,
            );
        }
    }

    wined3d_context_gl_generate_fbo_key(
        context_gl,
        core::ptr::addr_of_mut!(fbo_key),
        render_targets,
        depth_stencil,
        color_location,
        ds_location,
    );

    if TRACE_ON!(__wine_dbch_d3d) != 0 {
        let mut resource: *mut wined3d_resource = core::ptr::null_mut();
        let mut width: core::ffi::c_uint = 0;
        let mut height: core::ffi::c_uint = 0;
        let mut resource_type: *const core::ffi::c_char = core::ptr::null();

        i = 0 as core::ffi::c_uint;
        while i < (*gl_info).limits.buffers {
            resource = (*render_targets.offset(i as isize)).resource;
            if !resource.is_null() {
                if (*resource).type_ as core::ffi::c_uint
                    == WINED3D_RTYPE_BUFFER as core::ffi::c_uint
                {
                    width = (*resource).size as core::ffi::c_uint;
                    height = 1 as core::ffi::c_uint;
                    resource_type = b"buffer\0" as *const u8 as *const core::ffi::c_char;
                } else {
                    rt_texture = wined3d_texture_from_resource(resource);
                    rt_level = (*render_targets.offset(i as isize))
                        .sub_resource_idx
                        .wrapping_rem((*rt_texture).level_count);
                    width = wined3d_texture_get_level_width(rt_texture, rt_level);
                    height = wined3d_texture_get_level_height(rt_texture, rt_level);
                    resource_type = b"texture\0" as *const u8 as *const core::ffi::c_char;
                }
                TRACE!(
                    "    Color attachment %u: %p, %u format %s, %s %u, %ux%u, %u samples.\n",
                    i,
                    resource,
                    (*render_targets.offset(i as isize)).sub_resource_idx,
                    debug_d3dformat((*(*resource).format).id),
                    if fbo_key.rb_namespace
                        & ((1 as core::ffi::c_uint) << i.wrapping_add(1)) as uint32_t
                        != 0
                    {
                        b"renderbuffer\0" as *const u8 as *const core::ffi::c_char
                    } else {
                        resource_type
                    },
                    fbo_key.objects[i.wrapping_add(1) as usize].object,
                    width,
                    height,
                    (*resource).multisample_type as core::ffi::c_uint,
                );
            }
            i = i.wrapping_add(1);
        }
        resource = (*depth_stencil).resource;
        if !resource.is_null() {
            if (*resource).type_ as core::ffi::c_uint
                == WINED3D_RTYPE_BUFFER as core::ffi::c_uint
            {
                width = (*resource).size as core::ffi::c_uint;
                height = 1 as core::ffi::c_uint;
                resource_type = b"buffer\0" as *const u8 as *const core::ffi::c_char;
            } else {
                ds_texture = wined3d_texture_from_resource(resource);
                ds_level = (*depth_stencil)
                    .sub_resource_idx
                    .wrapping_rem((*ds_texture).level_count);
                width = wined3d_texture_get_level_width(ds_texture, ds_level);
                height = wined3d_texture_get_level_height(ds_texture, ds_level);
                resource_type = b"texture\0" as *const u8 as *const core::ffi::c_char;
            }
            TRACE!(
                "    Depth attachment: %p, %u format %s, %s %u, %ux%u, %u samples.\n",
                resource,
                (*depth_stencil).sub_resource_idx,
                debug_d3dformat((*(*resource).format).id),
                if fbo_key.rb_namespace
                    & ((1 as core::ffi::c_uint) << 0) as uint32_t
                    != 0
                {
                    b"renderbuffer\0" as *const u8 as *const core::ffi::c_char
                } else {
                    resource_type
                },
                fbo_key.objects[0].object,
                width,
                height,
                (*resource).multisample_type as core::ffi::c_uint,
            );
        }
    }

    // LIST_FOR_EACH_ENTRY(entry, &context_gl->fbo_list, struct fbo_entry, entry)
    entry = ((*context_gl).fbo_list.next as *mut core::ffi::c_char)
        .offset(-(core::mem::offset_of!(fbo_entry, entry) as isize))
        as *mut fbo_entry;
    while core::ptr::addr_of_mut!((*entry).entry)
        != core::ptr::addr_of_mut!((*context_gl).fbo_list)
    {
        if memcmp(
            core::ptr::addr_of_mut!(fbo_key) as *const core::ffi::c_void,
            core::ptr::addr_of_mut!((*entry).key) as *const core::ffi::c_void,
            core::mem::size_of::<wined3d_fbo_entry_key>() as size_t,
        ) != 0
        {
            entry = ((*entry).entry.next as *mut core::ffi::c_char)
                .offset(-(core::mem::offset_of!(fbo_entry, entry) as isize))
                as *mut fbo_entry;
        } else {
            list_remove(core::ptr::addr_of_mut!((*entry).entry));
            list_add_head(
                core::ptr::addr_of_mut!((*context_gl).fbo_list),
                core::ptr::addr_of_mut!((*entry).entry),
            );
            return entry;
        }
    }

    if (*context_gl).fbo_entry_count < WINED3D_MAX_FBO_ENTRIES as core::ffi::c_uint {
        entry = wined3d_context_gl_create_fbo_entry(
            context_gl,
            render_targets,
            depth_stencil,
            color_location,
            ds_location,
        );
        list_add_head(
            core::ptr::addr_of_mut!((*context_gl).fbo_list),
            core::ptr::addr_of_mut!((*entry).entry),
        );
        (*context_gl).fbo_entry_count = (*context_gl).fbo_entry_count.wrapping_add(1);
    } else {
        // entry = LIST_ENTRY!(list_tail(&context_gl->fbo_list), struct fbo_entry, entry)
        entry = (list_tail(core::ptr::addr_of_mut!((*context_gl).fbo_list))
            as *mut core::ffi::c_char)
            .offset(-(core::mem::offset_of!(fbo_entry, entry) as isize))
            as *mut fbo_entry;
        wined3d_context_gl_reuse_fbo_entry(
            context_gl,
            target,
            render_targets,
            depth_stencil,
            color_location,
            ds_location,
            entry,
        );
        list_remove(core::ptr::addr_of_mut!((*entry).entry));
        list_add_head(
            core::ptr::addr_of_mut!((*context_gl).fbo_list),
            core::ptr::addr_of_mut!((*entry).entry),
        );
    }

    return entry;
}

unsafe extern "C" fn wined3d_context_gl_apply_fbo_entry(
    context_gl: *mut wined3d_context_gl,
    target: GLenum,
    entry: *mut fbo_entry,
) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut read_binding: GLuint;
    let mut draw_binding: GLuint;
    let mut i: ::core::ffi::c_uint;

    if ((*entry).flags & WINED3D_FBO_ENTRY_FLAG_ATTACHED) != 0 {
        wined3d_context_gl_bind_fbo(context_gl, target, (*entry).id);
        return;
    }

    read_binding = (*context_gl).fbo_read_binding;
    draw_binding = (*context_gl).fbo_draw_binding;
    wined3d_context_gl_bind_fbo(context_gl, GL_FRAMEBUFFER, (*entry).id);

    if (*gl_info).supported[ARB_FRAMEBUFFER_NO_ATTACHMENTS as usize] != 0 {
        ((*gl_info).gl_ops.ext.p_glFramebufferParameteri.unwrap_unchecked())(GL_FRAMEBUFFER,
                GL_FRAMEBUFFER_DEFAULT_WIDTH, (*gl_info).limits.framebuffer_width as GLint);
        ((*gl_info).gl_ops.ext.p_glFramebufferParameteri.unwrap_unchecked())(GL_FRAMEBUFFER,
                GL_FRAMEBUFFER_DEFAULT_HEIGHT, (*gl_info).limits.framebuffer_height as GLint);
        ((*gl_info).gl_ops.ext.p_glFramebufferParameteri.unwrap_unchecked())(GL_FRAMEBUFFER, GL_FRAMEBUFFER_DEFAULT_LAYERS, 1);
        ((*gl_info).gl_ops.ext.p_glFramebufferParameteri.unwrap_unchecked())(GL_FRAMEBUFFER, GL_FRAMEBUFFER_DEFAULT_SAMPLES, 1);
    }

    /* Apply render targets */
    i = 0;
    while i < (*gl_info).limits.buffers {
        wined3d_context_gl_attach_surface_fbo(
            context_gl,
            target,
            i,
            (core::ptr::addr_of_mut!((*entry).key.objects) as *mut wined3d_fbo_resource)
                .offset(i.wrapping_add(1) as isize) as *mut wined3d_fbo_resource,
            ((*entry).key.rb_namespace
                & (1 << i.wrapping_add(1)))
                as BOOL,
        );
        i = i.wrapping_add(1);
    }

    wined3d_context_gl_attach_depth_stencil_fbo(
        context_gl,
        target,
        (core::ptr::addr_of_mut!((*entry).key.objects) as *mut wined3d_fbo_resource)
            .offset(0 as isize) as *mut wined3d_fbo_resource,
        ((*entry).key.rb_namespace & 0x1) as BOOL,
        (*entry).flags,
    );

    /* Set valid read and draw buffer bindings to satisfy pedantic pre-ES2_compatibility
     * GL contexts requirements. */
    (*gl_info).gl_ops.gl.p_glReadBuffer.unwrap_unchecked()(
        GL_NONE,
    );
    wined3d_context_gl_set_draw_buffer(context_gl, GL_NONE);
    if target != GL_FRAMEBUFFER {
        if target == GL_READ_FRAMEBUFFER {
            wined3d_context_gl_bind_fbo(context_gl, GL_DRAW_FRAMEBUFFER, draw_binding);
        } else {
            wined3d_context_gl_bind_fbo(context_gl, GL_READ_FRAMEBUFFER, read_binding);
        }
    }

    (*entry).flags |= WINED3D_FBO_ENTRY_FLAG_ATTACHED;
}

unsafe extern "C" fn wined3d_context_gl_apply_fbo_state(
    context_gl: *mut wined3d_context_gl,
    target: GLenum,
    render_targets: *const wined3d_rendertarget_info,
    depth_stencil: *const wined3d_rendertarget_info,
    color_location: DWORD,
    ds_location: DWORD,
) {
    let mut entry: *mut fbo_entry;
    let mut entry2: *mut fbo_entry;
    entry = ((*context_gl).fbo_destroy_list.next as *mut u8)
        .sub(core::mem::offset_of!(fbo_entry, entry)) as *mut fbo_entry;
    entry2 = ((*entry).entry.next as *mut u8)
        .sub(core::mem::offset_of!(fbo_entry, entry)) as *mut fbo_entry;
    while core::ptr::addr_of_mut!((*entry).entry)
        != core::ptr::addr_of_mut!((*context_gl).fbo_destroy_list)
    {
        wined3d_context_gl_destroy_fbo_entry(context_gl, entry);
        entry = entry2;
        entry2 = ((*entry).entry.next as *mut u8)
            .sub(core::mem::offset_of!(fbo_entry, entry)) as *mut fbo_entry;
    }

    if (*context_gl).rebind_fbo() != 0 {
        wined3d_context_gl_bind_fbo(context_gl, GL_FRAMEBUFFER, 0);
        (*context_gl).set_rebind_fbo((FALSE) as _);
    }

    if color_location == WINED3D_LOCATION_DRAWABLE {
        (*context_gl).current_fbo = core::ptr::null_mut();
        wined3d_context_gl_bind_fbo(context_gl, target, 0);
    } else {
        (*context_gl).current_fbo = wined3d_context_gl_find_fbo_entry(
            context_gl,
            target,
            render_targets,
            depth_stencil,
            color_location,
            ds_location,
        );
        wined3d_context_gl_apply_fbo_entry(context_gl, target, (*context_gl).current_fbo);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_apply_fbo_state_explicit(context_gl: *mut wined3d_context_gl, target: GLenum, rt: *mut wined3d_resource, rt_sub_resource_idx: ::core::ffi::c_uint, ds: *mut wined3d_resource, ds_sub_resource_idx: ::core::ffi::c_uint, location: uint32_t) {
    let mut ds_info: wined3d_rendertarget_info = wined3d_rendertarget_info {
        gl_view: wined3d_gl_view {
            target: 0 as GLenum,
            name: 0,
        },
        resource: ::core::ptr::null_mut::<wined3d_resource>(),
        sub_resource_idx: 0,
        layer_count: 0,
    };

    memset(
        &raw mut (*context_gl).blit_targets as *mut wined3d_rendertarget_info
            as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[wined3d_rendertarget_info; 8]>() as size_t,
    );
    if !rt.is_null() {
        (*context_gl).blit_targets[0 as usize].resource = rt;
        (*context_gl).blit_targets[0 as usize].sub_resource_idx =
            rt_sub_resource_idx;
        (*context_gl).blit_targets[0 as usize].layer_count =
            1;
    }

    if !ds.is_null() {
        ds_info.resource = ds;
        ds_info.sub_resource_idx = ds_sub_resource_idx;
        ds_info.layer_count = 1;
    }

    wined3d_context_gl_apply_fbo_state(
        context_gl,
        target,
        &raw mut (*context_gl).blit_targets as *mut wined3d_rendertarget_info,
        &raw mut ds_info,
        location as DWORD,
        location as DWORD,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_alloc_occlusion_query(context_gl: *mut wined3d_context_gl, query: *mut wined3d_occlusion_query) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;

    if (*context_gl).free_occlusion_query_count != 0 {
        (*context_gl).free_occlusion_query_count = (*context_gl).free_occlusion_query_count.wrapping_sub(1);
        (*query).id = *(*context_gl)
            .free_occlusion_queries
            .offset((*context_gl).free_occlusion_query_count as isize);
    } else {
        if (*gl_info).supported[(ARB_OCCLUSION_QUERY) as usize] != 0 {
            ((*gl_info).gl_ops.ext.p_glGenQueries.unwrap_unchecked())(
                1,
                core::ptr::addr_of_mut!((*query).id),
            );
            checkGLcall!(gl_info, "glGenQueries");
        } else {
            WARN!("Occlusion queries not supported, not allocating query id.\n");
            (*query).id = 0;
        }
    }

    (*query).context_gl = context_gl;
    list_add_head(
        core::ptr::addr_of_mut!((*context_gl).occlusion_queries),
        core::ptr::addr_of_mut!((*query).entry),
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_free_occlusion_query(query: *mut wined3d_occlusion_query) {
    let mut context_gl: *mut wined3d_context_gl = (*query).context_gl;

    list_remove(core::ptr::addr_of_mut!((*query).entry));
    (*query).context_gl = core::ptr::null_mut();

    if wined3d_array_reserve(
        core::ptr::addr_of_mut!((*context_gl).free_occlusion_queries) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*context_gl).free_occlusion_query_size),
        (*context_gl).free_occlusion_query_count.wrapping_add(1) as SIZE_T,
        core::mem::size_of::<GLuint>() as SIZE_T,
    ) == 0
    {
        ERR!(
            "Failed to grow free list, leaking query %u in context %p.\n",
            (*query).id,
            context_gl
        );
        return;
    }

    let fresh33 = (*context_gl).free_occlusion_query_count;
    (*context_gl).free_occlusion_query_count = (*context_gl).free_occlusion_query_count.wrapping_add(1);
    *(*context_gl).free_occlusion_queries.offset(fresh33 as isize) = (*query).id;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_alloc_fence(context_gl: *mut wined3d_context_gl, fence: *mut wined3d_fence) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;

    if (*context_gl).free_fence_count != 0 {
        (*context_gl).free_fence_count = (*context_gl).free_fence_count.wrapping_sub(1);
        (*fence).object = *(*context_gl).free_fences.offset((*context_gl).free_fence_count as isize);
    } else if (*gl_info).supported[ARB_SYNC as usize] != 0 {
        /* Using ARB_sync, not much to do here. */
        (*fence).object.sync = core::ptr::null_mut();
    } else if (*gl_info).supported[APPLE_FENCE as usize] != 0 {
        ((*gl_info).gl_ops.ext.p_glGenFencesAPPLE.unwrap_unchecked())(1, &raw mut (*fence).object.id);
        checkGLcall!(gl_info, b"glGenFencesAPPLE\0".as_ptr() as *const i8);
    } else if (*gl_info).supported[NV_FENCE as usize] != 0 {
        ((*gl_info).gl_ops.ext.p_glGenFencesNV.unwrap_unchecked())(1, &raw mut (*fence).object.id);
        checkGLcall!(gl_info, b"glGenFencesNV\0".as_ptr() as *const i8);
    } else {
        WARN!("Fences not supported, not allocating fence.\n");
        (*fence).object.id = 0;
    }

    (*fence).context_gl = context_gl;
    list_add_head(&raw mut (*context_gl).fences, &raw mut (*fence).entry);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_free_fence(fence: *mut wined3d_fence) {
    let mut context_gl: *mut wined3d_context_gl = (*fence).context_gl as *mut wined3d_context_gl;

    list_remove(core::ptr::addr_of_mut!((*fence).entry));
    (*fence).context_gl = core::ptr::null_mut();

    if wined3d_array_reserve(
        core::ptr::addr_of_mut!((*context_gl).free_fences) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*context_gl).free_fence_size),
        (*context_gl).free_fence_count.wrapping_add(1 as core::ffi::c_uint) as SIZE_T,
        core::mem::size_of::<wined3d_gl_fence_object>() as SIZE_T,
    ) == 0
    {
        ERR!(
            "Failed to grow free list, leaking fence %u in context %p.\n",
            (*fence).object.id,
            context_gl
        );
        return;
    }

    let fresh32 = (*context_gl).free_fence_count;
    (*context_gl).free_fence_count = (*context_gl).free_fence_count.wrapping_add(1);
    *(*context_gl).free_fences.offset(fresh32 as isize) = (*fence).object;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_alloc_timestamp_query(context_gl: *mut wined3d_context_gl, query: *mut wined3d_timestamp_query) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;

    if (*context_gl).free_timestamp_query_count != 0 {
        (*context_gl).free_timestamp_query_count -= 1;
        (*query).id = *(*context_gl)
            .free_timestamp_queries
            .offset((*context_gl).free_timestamp_query_count as isize);
    } else {
        ((*gl_info).gl_ops.ext.p_glGenQueries.unwrap_unchecked())(
            1,
            core::ptr::addr_of_mut!((*query).id),
        );
        checkGLcall!(gl_info, b"glGenQueries\0".as_ptr() as *const ::core::ffi::c_char);
    }

    (*query).context_gl = context_gl;
    list_add_head(
        core::ptr::addr_of_mut!((*context_gl).timestamp_queries),
        core::ptr::addr_of_mut!((*query).entry),
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_free_timestamp_query(mut query: *mut wined3d_timestamp_query) {
    let mut context_gl: *mut wined3d_context_gl = (*query).context_gl;

    list_remove(core::ptr::addr_of_mut!((*query).entry));
    (*query).context_gl = core::ptr::null_mut();

    if wined3d_array_reserve(
        core::ptr::addr_of_mut!((*context_gl).free_timestamp_queries) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*context_gl).free_timestamp_query_size),
        (*context_gl)
            .free_timestamp_query_count
            .wrapping_add(1) as SIZE_T,
        core::mem::size_of::<GLuint>() as SIZE_T,
    ) == 0
    {
        ERR!(
            "Failed to grow free list, leaking query %u in context %p.\n",
            (*query).id,
            context_gl
        );
        return;
    }

    let fresh36 = (*context_gl).free_timestamp_query_count;
    (*context_gl).free_timestamp_query_count =
        (*context_gl).free_timestamp_query_count.wrapping_add(1);
    *(*context_gl)
        .free_timestamp_queries
        .offset(fresh36 as isize) = (*query).id;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_alloc_so_statistics_query(context_gl: *mut wined3d_context_gl, query: *mut wined3d_so_statistics_query) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;

    if (*context_gl).free_so_statistics_query_count != 0 {
        (*context_gl).free_so_statistics_query_count =
            (*context_gl).free_so_statistics_query_count.wrapping_sub(1);
        (*query).u = *(*context_gl)
            .free_so_statistics_queries
            .offset((*context_gl).free_so_statistics_query_count as isize);
    } else {
        ((*gl_info).gl_ops.ext.p_glGenQueries.unwrap_unchecked())(
            (ARRAY_SIZE!((*query).u.id)) as i32,
            core::ptr::addr_of_mut!((*query).u.id) as *mut GLuint,
        );
        checkGLcall!(gl_info, b"glGenQueries\0" as *const u8 as *const i8);
    }

    (*query).context_gl = context_gl;
    list_add_head(
        core::ptr::addr_of_mut!((*context_gl).so_statistics_queries),
        core::ptr::addr_of_mut!((*query).entry),
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_free_so_statistics_query(query: *mut wined3d_so_statistics_query) {
    let mut context_gl: *mut wined3d_context_gl = (*query).context_gl;

    list_remove(core::ptr::addr_of_mut!((*query).entry));
    (*query).context_gl = core::ptr::null_mut();

    if wined3d_array_reserve(
        core::ptr::addr_of_mut!((*context_gl).free_so_statistics_queries) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*context_gl).free_so_statistics_query_size),
        (*context_gl).free_so_statistics_query_count.wrapping_add(1) as SIZE_T,
        core::mem::size_of::<wined3d_gl_so_statistics_query>() as SIZE_T,
    ) == 0
    {
        ERR!(
            "Failed to grow free list, leaking GL queries %u, %u in context %p.\n",
            (*query).u.id[0 as usize],
            (*query).u.id[1 as usize],
            context_gl
        );
        return;
    }

    let fresh = (*context_gl).free_so_statistics_query_count;
    (*context_gl).free_so_statistics_query_count = (*context_gl).free_so_statistics_query_count.wrapping_add(1);
    *(*context_gl).free_so_statistics_queries.add(fresh as usize) = (*query).u;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_alloc_pipeline_statistics_query(mut context_gl: *mut wined3d_context_gl, mut query: *mut wined3d_pipeline_statistics_query) {
    let mut gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    if (*context_gl).free_pipeline_statistics_query_count != 0 {
        (*context_gl).free_pipeline_statistics_query_count = (*context_gl)
            .free_pipeline_statistics_query_count
            .wrapping_sub(1);
        (*query).u = *(*context_gl)
            .free_pipeline_statistics_queries
            .offset((*context_gl).free_pipeline_statistics_query_count as isize);
    } else {
        ((*gl_info).gl_ops.ext.p_glGenQueries.unwrap_unchecked())(
            (ARRAY_SIZE!((*query).u.id)) as i32,
            &raw mut (*query).u.id as *mut GLuint,
        );
        checkGLcall!(gl_info, b"glGenQueries\0".as_ptr() as *const i8);
    }

    (*query).context_gl = context_gl;
    list_add_head(
        &raw mut (*context_gl).pipeline_statistics_queries,
        &raw mut (*query).entry,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_free_pipeline_statistics_query(query: *mut wined3d_pipeline_statistics_query) {
    let mut context_gl: *mut wined3d_context_gl = (*query).context_gl;

    list_remove(core::ptr::addr_of_mut!((*query).entry));
    (*query).context_gl = core::ptr::null_mut();

    if wined3d_array_reserve(
        core::ptr::addr_of_mut!((*context_gl).free_pipeline_statistics_queries) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*context_gl).free_pipeline_statistics_query_size),
        (*context_gl).free_pipeline_statistics_query_count.wrapping_add(1) as SIZE_T,
        core::mem::size_of::<wined3d_gl_pipeline_statistics_query>() as SIZE_T,
    ) == 0
    {
        ERR!("Failed to grow free list, leaking GL queries in context %p.\n", context_gl);
        return;
    }

    let fresh34 = (*context_gl).free_pipeline_statistics_query_count;
    (*context_gl).free_pipeline_statistics_query_count = (*context_gl)
        .free_pipeline_statistics_query_count
        .wrapping_add(1);
    *(*context_gl)
        .free_pipeline_statistics_queries
        .add(fresh34 as usize) = (*query).u;
}

unsafe extern "C" fn wined3d_context_gl_enum_fbo_entries(
    mut device: *const wined3d_device,
    mut name: GLuint,
    mut rb_namespace: BOOL,
    mut callback: context_fbo_entry_func_t,
) {
    let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut j: ::core::ffi::c_uint = 0;
    while i < (*device).context_count {
        let mut context_gl: *mut wined3d_context_gl =
            wined3d_context_gl(*(*device).contexts.offset(i as isize));
        let mut gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
        let mut entry: *mut fbo_entry = ::core::ptr::null_mut::<fbo_entry>();
        let mut entry2: *mut fbo_entry = ::core::ptr::null_mut::<fbo_entry>();
        // LIST_FOR_EACH_ENTRY_SAFE(entry, entry2, &context_gl->fbo_list, struct fbo_entry, entry)
        entry = ((*context_gl).fbo_list.next as *mut u8)
            .sub(core::mem::offset_of!(fbo_entry, entry)) as *mut fbo_entry;
        entry2 = ((*entry).entry.next as *mut u8)
            .sub(core::mem::offset_of!(fbo_entry, entry)) as *mut fbo_entry;
        while core::ptr::addr_of_mut!((*entry).entry)
            != core::ptr::addr_of_mut!((*context_gl).fbo_list)
        {
            j = 0 as ::core::ffi::c_uint;
            while j
                < (*gl_info)
                    .limits
                    .buffers
                    .wrapping_add(1 as ::core::ffi::c_uint)
            {
                if (*entry).key.objects[j as usize].object == name
                    && (((*entry).key.rb_namespace
                        & ((1 as ::core::ffi::c_int) << j) as uint32_t)
                        == 0) as ::core::ffi::c_int
                        == (rb_namespace == 0) as ::core::ffi::c_int
                {
                    let callback = callback.unwrap_unchecked();
                    callback(context_gl, entry);
                    break;
                } else {
                    j = j.wrapping_add(1);
                }
            }
            entry = entry2;
            entry2 = ((*entry).entry.next as *mut u8)
                .sub(core::mem::offset_of!(fbo_entry, entry)) as *mut fbo_entry;
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn wined3d_context_gl_queue_fbo_entry_destruction(
    context_gl: *mut wined3d_context_gl,
    entry: *mut fbo_entry,
) {
    list_remove(core::ptr::addr_of_mut!((*entry).entry));
    list_add_head(core::ptr::addr_of_mut!((*context_gl).fbo_destroy_list), core::ptr::addr_of_mut!((*entry).entry));
}
#[no_mangle]
pub unsafe extern "C" fn context_gl_resource_released(device: *mut wined3d_device, name: GLuint, rb_namespace: BOOL) {
    wined3d_context_gl_enum_fbo_entries(
        device,
        name,
        rb_namespace,
        Some(wined3d_context_gl_queue_fbo_entry_destruction
            as unsafe extern "C" fn(*mut wined3d_context_gl, *mut fbo_entry)),
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_texture_update(context_gl: *mut wined3d_context_gl, texture_gl: *const wined3d_texture_gl) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let entry: *mut fbo_entry = (*context_gl).current_fbo;
    let mut i: ::core::ffi::c_uint;

    if entry.is_null() || (*context_gl).rebind_fbo() != 0 {
        return;
    }

    i = 0;
    while i < (*gl_info).limits.buffers.wrapping_add(1) {
        if (*texture_gl).texture_rgb.name == (*entry).key.objects[i as usize].object
            || (*texture_gl).texture_srgb.name == (*entry).key.objects[i as usize].object
        {
            WINE_TRACE!("Updated texture %p is bound as attachment %u to the current FBO.\n", texture_gl, i);
            (*context_gl).set_rebind_fbo((TRUE) as _);
            return;
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn wined3d_context_gl_restore_pixel_format(
    context_gl: *mut wined3d_context_gl,
) -> BOOL {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut ret: BOOL = (FALSE as i32);

    if (*context_gl).restore_pf != 0 && IsWindow((*context_gl).restore_pf_win) != 0 {
        if (*gl_info).supported[WGL_WINE_PIXEL_FORMAT_PASSTHROUGH as usize] != 0 {
            let dc: HDC = GetDCEx(
                (*context_gl).restore_pf_win,
                core::ptr::null_mut(),
                (DCX_USESTYLE | DCX_CACHE) as DWORD,
            );
            if !dc.is_null() {
                ret = ((*gl_info).gl_ops.ext.p_wglSetPixelFormatWINE.unwrap_unchecked())(dc, (*context_gl).restore_pf);
                if ret == 0 {
                    ERR!(
                        "Failed to restore pixel format %d on window %p.\n",
                        (*context_gl).restore_pf,
                        (*context_gl).restore_pf_win
                    );
                }
                ReleaseDC((*context_gl).restore_pf_win, dc);
            }
        } else {
            ERR!(
                "Unable to restore pixel format %d on window %p.\n",
                (*context_gl).restore_pf,
                (*context_gl).restore_pf_win
            );
        }
    }

    (*context_gl).restore_pf = 0;
    (*context_gl).restore_pf_win = core::ptr::null_mut();
    return ret;
}

unsafe extern "C" fn wined3d_context_gl_set_pixel_format(
    context_gl: *mut wined3d_context_gl,
) -> BOOL {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let private: BOOL = ((*context_gl).dc_is_private()) as i32;
    let format: ::core::ffi::c_int = (*context_gl).pixel_format;
    let dc: HDC = (*context_gl).dc;
    let mut current: ::core::ffi::c_int;
    let mut win: HWND;

    if private != 0 && (*context_gl).dc_has_format() != 0 {
        return (TRUE) as BOOL;
    }

    if private == 0 && WindowFromDC(dc) != (*context_gl).window {
        return (FALSE) as BOOL;
    }

    current = (*gl_info).gl_ops.wgl.p_wglGetPixelFormat.unwrap_unchecked()(dc);
    if current == format || (current == 0 && (*context_gl).internal_format_set() != 0) {
        // goto success
    } else {
        /* By default WGL doesn't allow pixel format adjustments but we need it
         * here. For this reason there's a Wine specific wglSetPixelFormat()
         * which allows us to set the pixel format multiple times. Use it when we
         * can, because even though no pixel format may currently be set, the
         * application may try to set one later. */
        if (*gl_info).supported[WGL_WINE_PIXEL_FORMAT_PASSTHROUGH as usize] != 0 {
            if ((*gl_info).gl_ops.ext.p_wglSetPixelFormatWINE.unwrap_unchecked())(dc, format) == 0 {
                ERR!(
                    "wglSetPixelFormatWINE failed to set pixel format %d on device context %p.\n",
                    format,
                    dc
                );
                return (FALSE) as BOOL;
            }
            (*context_gl).set_internal_format_set((1) as _);
        } else if current != 0 {
            /* OpenGL doesn't allow pixel format adjustments. Print an error and
             * continue using the old format. There's a big chance that the old
             * format works although with a performance hit and perhaps rendering
             * errors. */
            ERR!(
                "Unable to set pixel format %d on device context %p. Already using format %d.\n",
                format,
                dc,
                current
            );
            return (TRUE) as BOOL;
        } else if SetPixelFormat(dc, format, core::ptr::null_mut()) == 0 {
            /* This may also happen if the dc belongs to a destroyed window. */
            WARN!(
                "Failed to set pixel format %d on device context %p, last error %#lx.\n",
                format,
                dc,
                GetLastError()
            );
            return (FALSE) as BOOL;
        }

        win = if private != 0 {
            core::ptr::null_mut()
        } else {
            WindowFromDC(dc)
        };
        if win != (*context_gl).restore_pf_win {
            wined3d_context_gl_restore_pixel_format(context_gl);
        }
        (*context_gl).restore_pf = if private != 0 {
            0
        } else {
            current
        };
        (*context_gl).restore_pf_win = win;
    }

    // success:
    if private != 0 {
        (*context_gl).set_dc_has_format((TRUE) as _);
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn wined3d_context_gl_set_gl_context(context_gl: *mut wined3d_context_gl) -> BOOL {
    let mut device_gl: *mut wined3d_device_gl = wined3d_device_gl((*context_gl).c.device);
    let mut backup: BOOL = (FALSE as i32);

    if wined3d_context_gl_set_pixel_format(context_gl) == 0 {
        WARN!("Failed to set pixel format %d on device context %p.\n",
                (*context_gl).pixel_format, (*context_gl).dc);
        backup = (TRUE as i32);
    }

    if backup != 0 || wglMakeCurrent((*context_gl).dc, (*context_gl).gl_ctx) == 0 {
        WARN!("Failed to make GL context %p current on device context %p, last error %#lx.\n",
                (*context_gl).gl_ctx, (*context_gl).dc, GetLastError());
        (*context_gl).set_valid((0) as _);
        WARN!("Trying fallback to the backup window.\n");

        if (*context_gl).c.destroyed() != 0 {
            wined3d_context_gl_set_current(core::ptr::null_mut());
            return (FALSE) as BOOL;
        }

        wined3d_release_dc((*context_gl).window, (*context_gl).dc);
        (*context_gl).dc = wined3d_device_gl_get_backup_dc(device_gl);
        if (*context_gl).dc.is_null() {
            wined3d_context_gl_set_current(core::ptr::null_mut());
            return (FALSE) as BOOL;
        }

        (*context_gl).set_dc_is_private((TRUE) as _);
        (*context_gl).set_dc_has_format((FALSE) as _);

        if wined3d_context_gl_set_pixel_format(context_gl) == 0 {
            ERR!("Failed to set pixel format %d on device context %p.\n",
                    (*context_gl).pixel_format, (*context_gl).dc);
            wined3d_context_gl_set_current(core::ptr::null_mut());
            return (FALSE) as BOOL;
        }

        if wglMakeCurrent((*context_gl).dc, (*context_gl).gl_ctx) == 0 {
            ERR!("Fallback to backup window (dc %p) failed too, last error %#lx.\n",
                    (*context_gl).dc, GetLastError());
            wined3d_context_gl_set_current(core::ptr::null_mut());
            return (FALSE) as BOOL;
        }

        (*context_gl).set_valid((1) as _);
    }
    (*context_gl).set_needs_set((0) as _);

    return (TRUE) as BOOL;
}

unsafe extern "C" fn context_restore_gl_context(gl_info: *const wined3d_gl_info, dc: HDC, gl_ctx: HGLRC) {
    if wglMakeCurrent(dc, gl_ctx) == 0 {
        ERR!("Failed to restore GL context %p on device context %p, last error %#lx.\n", gl_ctx, dc, GetLastError());
        wined3d_context_gl_set_current(core::ptr::null_mut());
    }
}

unsafe extern "C" fn wined3d_context_gl_update_window(mut context_gl: *mut wined3d_context_gl) {
    if (*context_gl).c.swapchain.is_null() {
        return;
    }
    if (*context_gl).window == (*(*context_gl).c.swapchain).win_handle {
        return;
    }
    TRACE!(
        "Updating context %p window from %p to %p.\n",
        context_gl,
        (*context_gl).window,
        (*(*context_gl).c.swapchain).win_handle
    );
    if !(*context_gl).dc.is_null() {
        wined3d_release_dc((*context_gl).window, (*context_gl).dc);
    }
    (*context_gl).window = (*(*context_gl).c.swapchain).win_handle;
    (*context_gl).set_dc_is_private(FALSE);
    (*context_gl).set_dc_has_format(FALSE);
    (*context_gl).set_needs_set(1);
    (*context_gl).set_valid(1);
    (*context_gl).set_internal_format_set(0);
    (*context_gl).dc = GetDCEx(
        (*context_gl).window,
        core::ptr::null_mut(),
        DCX_USESTYLE | DCX_CACHE,
    );
    if (*context_gl).dc.is_null() {
        ERR!(
            "Failed to get a device context for window %p.\n",
            (*context_gl).window
        );
        (*context_gl).set_valid(0);
    }
}

unsafe extern "C" fn wined3d_context_gl_cleanup(context_gl: *mut wined3d_context_gl) {
    let mut pipeline_statistics_query: *mut wined3d_pipeline_statistics_query = core::ptr::null_mut();
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut so_statistics_query: *mut wined3d_so_statistics_query = core::ptr::null_mut();
    let mut timestamp_query: *mut wined3d_timestamp_query = core::ptr::null_mut();
    let mut occlusion_query: *mut wined3d_occlusion_query = core::ptr::null_mut();
    let mut current: *mut wined3d_context_gl = core::ptr::null_mut();
    let mut entry: *mut fbo_entry = core::ptr::null_mut();
    let mut entry2: *mut fbo_entry = core::ptr::null_mut();
    let mut fence: *mut wined3d_fence = core::ptr::null_mut();
    let mut restore_ctx: HGLRC = core::ptr::null_mut();
    let mut restore_dc: HDC = core::ptr::null_mut();
    let mut i: ::core::ffi::c_uint = 0;

    TRACE!("context_gl %p.\n", context_gl);

    restore_ctx = wglGetCurrentContext();
    restore_dc = wglGetCurrentDC();

    if (*context_gl).valid() != 0 && (*context_gl).gl_ctx != restore_ctx {
        /* Attempting to restore a GL context corresponding to a wined3d
         * context is not particularly useful. Worse, when we're called from
         * wined3d_context_gl_destroy(), we subsequently clear the "current
         * D3D context" TLS value, which would cause
         * wined3d_context_gl_enter() to consider the GL context a non-D3D
         * context. */
        current = wined3d_context_gl_get_current();
        if !current.is_null() && (*current).gl_ctx == restore_ctx {
            restore_ctx = core::ptr::null_mut();
        }
        wined3d_context_gl_set_gl_context(context_gl);
    } else {
        restore_ctx = core::ptr::null_mut();
    }

    if (*context_gl).valid() != 0 {
        /* If we're here because we're switching away from a previously
         * destroyed context, acquiring a context in order to submit a fence
         * is problematic. In particular, we'd end up back here again in the
         * process of switching to the newly acquired context.
         *
         * If fences aren't supported there should be nothing to wait for
         * anyway, so just do nothing in that case. */
        if (*context_gl).c.destroyed() != 0 {
            (*gl_info).gl_ops.gl.p_glFinish.unwrap_unchecked()();
        } else if (*(*context_gl).c.d3d_info).fences() != 0 {
            wined3d_context_gl_submit_command_fence(context_gl);
            wined3d_context_gl_wait_command_fence(
                context_gl,
                wined3d_device_gl((*context_gl).c.device)
                    .read()
                    .current_fence_id
                    .wrapping_sub(1),
            );
        }

        if (*context_gl).dummy_arbfp_prog != 0 {
            ((*gl_info).gl_ops.ext.p_glDeleteProgramsARB.unwrap_unchecked())(
                1,
                &raw mut (*context_gl).dummy_arbfp_prog,
            );
        }

        if (*context_gl).blit_vbo != 0 {
            ((*gl_info).gl_ops.ext.p_glDeleteBuffers.unwrap_unchecked())(
                1,
                &raw mut (*context_gl).blit_vbo,
            );
        }

        i = 0;
        while i < (*context_gl).free_pipeline_statistics_query_count {
            let q: *mut wined3d_gl_pipeline_statistics_query =
                (*context_gl).free_pipeline_statistics_queries.offset(i as isize);
            ((*gl_info).gl_ops.ext.p_glDeleteQueries.unwrap_unchecked())((ARRAY_SIZE!((*q).id)) as i32, ((*q).id).as_ptr());
            i = i.wrapping_add(1);
        }

        i = 0;
        while i < (*context_gl).free_so_statistics_query_count {
            let q: *mut wined3d_gl_so_statistics_query =
                (*context_gl).free_so_statistics_queries.offset(i as isize);
            ((*gl_info).gl_ops.ext.p_glDeleteQueries.unwrap_unchecked())((ARRAY_SIZE!((*q).id)) as i32, ((*q).id).as_ptr());
            i = i.wrapping_add(1);
        }

        if (*context_gl).free_timestamp_query_count != 0 {
            ((*gl_info).gl_ops.ext.p_glDeleteQueries.unwrap_unchecked())(
                ((*context_gl).free_timestamp_query_count) as i32,
                (*context_gl).free_timestamp_queries,
            );
        }

        if (*gl_info).supported[ARB_SYNC as usize] != 0 {
            i = 0;
            while i < (*context_gl).free_fence_count {
                ((*gl_info).gl_ops.ext.p_glDeleteSync.unwrap_unchecked())(
                    (*(*context_gl).free_fences.offset(i as isize)).sync,
                );
                i = i.wrapping_add(1);
            }
        } else if (*gl_info).supported[APPLE_FENCE as usize] != 0 {
            i = 0;
            while i < (*context_gl).free_fence_count {
                ((*gl_info).gl_ops.ext.p_glDeleteFencesAPPLE.unwrap_unchecked())(
                    1,
                    &raw mut (*(*context_gl).free_fences.offset(i as isize)).id,
                );
                i = i.wrapping_add(1);
            }
        } else if (*gl_info).supported[NV_FENCE as usize] != 0 {
            i = 0;
            while i < (*context_gl).free_fence_count {
                ((*gl_info).gl_ops.ext.p_glDeleteFencesNV.unwrap_unchecked())(
                    1,
                    &raw mut (*(*context_gl).free_fences.offset(i as isize)).id,
                );
                i = i.wrapping_add(1);
            }
        }

        if (*context_gl).free_occlusion_query_count != 0 {
            ((*gl_info).gl_ops.ext.p_glDeleteQueries.unwrap_unchecked())(
                ((*context_gl).free_occlusion_query_count) as i32,
                (*context_gl).free_occlusion_queries,
            );
        }

        checkGLcall!(gl_info, "context cleanup");
    }
    free((*context_gl).submitted.fences as *mut ::core::ffi::c_void);
    free((*context_gl).free_pipeline_statistics_queries as *mut ::core::ffi::c_void);
    free((*context_gl).free_so_statistics_queries as *mut ::core::ffi::c_void);
    free((*context_gl).free_timestamp_queries as *mut ::core::ffi::c_void);
    free((*context_gl).free_fences as *mut ::core::ffi::c_void);
    free((*context_gl).free_occlusion_queries as *mut ::core::ffi::c_void);

    pipeline_statistics_query = ((*context_gl).pipeline_statistics_queries.next as *mut ::core::ffi::c_char)
        .offset(-(core::mem::offset_of!(wined3d_pipeline_statistics_query, entry) as isize))
        as *mut wined3d_pipeline_statistics_query;
    while &raw mut (*pipeline_statistics_query).entry != &raw mut (*context_gl).pipeline_statistics_queries {
        if (*context_gl).valid() != 0 {
            ((*gl_info).gl_ops.ext.p_glDeleteQueries.unwrap_unchecked())(
                (ARRAY_SIZE!((*pipeline_statistics_query).u.id)) as i32,
                ((*pipeline_statistics_query).u.id).as_ptr(),
            );
        }
        (*pipeline_statistics_query).context_gl = core::ptr::null_mut();
        pipeline_statistics_query = ((*pipeline_statistics_query).entry.next as *mut ::core::ffi::c_char)
            .offset(-(core::mem::offset_of!(wined3d_pipeline_statistics_query, entry) as isize))
            as *mut wined3d_pipeline_statistics_query;
    }

    so_statistics_query = ((*context_gl).so_statistics_queries.next as *mut ::core::ffi::c_char)
        .offset(-(core::mem::offset_of!(wined3d_so_statistics_query, entry) as isize))
        as *mut wined3d_so_statistics_query;
    while &raw mut (*so_statistics_query).entry != &raw mut (*context_gl).so_statistics_queries {
        if (*context_gl).valid() != 0 {
            ((*gl_info).gl_ops.ext.p_glDeleteQueries.unwrap_unchecked())(
                (ARRAY_SIZE!((*so_statistics_query).u.id)) as i32,
                ((*so_statistics_query).u.id).as_ptr(),
            );
        }
        (*so_statistics_query).context_gl = core::ptr::null_mut();
        so_statistics_query = ((*so_statistics_query).entry.next as *mut ::core::ffi::c_char)
            .offset(-(core::mem::offset_of!(wined3d_so_statistics_query, entry) as isize))
            as *mut wined3d_so_statistics_query;
    }

    timestamp_query = ((*context_gl).timestamp_queries.next as *mut ::core::ffi::c_char)
        .offset(-(core::mem::offset_of!(wined3d_timestamp_query, entry) as isize))
        as *mut wined3d_timestamp_query;
    while &raw mut (*timestamp_query).entry != &raw mut (*context_gl).timestamp_queries {
        if (*context_gl).valid() != 0 {
            ((*gl_info).gl_ops.ext.p_glDeleteQueries.unwrap_unchecked())(
                1,
                &raw mut (*timestamp_query).id,
            );
        }
        (*timestamp_query).context_gl = core::ptr::null_mut();
        timestamp_query = ((*timestamp_query).entry.next as *mut ::core::ffi::c_char)
            .offset(-(core::mem::offset_of!(wined3d_timestamp_query, entry) as isize))
            as *mut wined3d_timestamp_query;
    }

    fence = ((*context_gl).fences.next as *mut ::core::ffi::c_char)
        .offset(-(core::mem::offset_of!(wined3d_fence, entry) as isize)) as *mut wined3d_fence;
    while &raw mut (*fence).entry != &raw mut (*context_gl).fences {
        if (*context_gl).valid() != 0 {
            if (*gl_info).supported[ARB_SYNC as usize] != 0 {
                if !(*fence).object.sync.is_null() {
                    ((*gl_info).gl_ops.ext.p_glDeleteSync.unwrap_unchecked())((*fence).object.sync);
                }
            } else if (*gl_info).supported[APPLE_FENCE as usize] != 0 {
                ((*gl_info).gl_ops.ext.p_glDeleteFencesAPPLE.unwrap_unchecked())(
                    1,
                    &raw mut (*fence).object.id,
                );
            } else if (*gl_info).supported[NV_FENCE as usize] != 0 {
                ((*gl_info).gl_ops.ext.p_glDeleteFencesNV.unwrap_unchecked())(
                    1,
                    &raw mut (*fence).object.id,
                );
            }
        }
        (*fence).context_gl = core::ptr::null_mut();
        fence = ((*fence).entry.next as *mut ::core::ffi::c_char)
            .offset(-(core::mem::offset_of!(wined3d_fence, entry) as isize)) as *mut wined3d_fence;
    }

    occlusion_query = ((*context_gl).occlusion_queries.next as *mut ::core::ffi::c_char)
        .offset(-(core::mem::offset_of!(wined3d_occlusion_query, entry) as isize))
        as *mut wined3d_occlusion_query;
    while &raw mut (*occlusion_query).entry != &raw mut (*context_gl).occlusion_queries {
        if (*context_gl).valid() != 0 {
            ((*gl_info).gl_ops.ext.p_glDeleteQueries.unwrap_unchecked())(
                1,
                &raw mut (*occlusion_query).id,
            );
        }
        (*occlusion_query).context_gl = core::ptr::null_mut();
        occlusion_query = ((*occlusion_query).entry.next as *mut ::core::ffi::c_char)
            .offset(-(core::mem::offset_of!(wined3d_occlusion_query, entry) as isize))
            as *mut wined3d_occlusion_query;
    }

    entry = ((*context_gl).fbo_destroy_list.next as *mut ::core::ffi::c_char)
        .offset(-(core::mem::offset_of!(fbo_entry, entry) as isize)) as *mut fbo_entry;
    entry2 = ((*entry).entry.next as *mut ::core::ffi::c_char)
        .offset(-(core::mem::offset_of!(fbo_entry, entry) as isize)) as *mut fbo_entry;
    while &raw mut (*entry).entry != &raw mut (*context_gl).fbo_destroy_list {
        if (*context_gl).valid() == 0 {
            (*entry).id = 0;
        }
        wined3d_context_gl_destroy_fbo_entry(context_gl, entry);
        entry = entry2;
        entry2 = ((*entry).entry.next as *mut ::core::ffi::c_char)
            .offset(-(core::mem::offset_of!(fbo_entry, entry) as isize)) as *mut fbo_entry;
    }

    entry = ((*context_gl).fbo_list.next as *mut ::core::ffi::c_char)
        .offset(-(core::mem::offset_of!(fbo_entry, entry) as isize)) as *mut fbo_entry;
    entry2 = ((*entry).entry.next as *mut ::core::ffi::c_char)
        .offset(-(core::mem::offset_of!(fbo_entry, entry) as isize)) as *mut fbo_entry;
    while &raw mut (*entry).entry != &raw mut (*context_gl).fbo_list {
        if (*context_gl).valid() == 0 {
            (*entry).id = 0;
        }
        wined3d_context_gl_destroy_fbo_entry(context_gl, entry);
        entry = entry2;
        entry2 = ((*entry).entry.next as *mut ::core::ffi::c_char)
            .offset(-(core::mem::offset_of!(fbo_entry, entry) as isize)) as *mut fbo_entry;
    }

    free((*context_gl).texture_type as *mut ::core::ffi::c_void);

    wined3d_context_gl_restore_pixel_format(context_gl);
    if !restore_ctx.is_null() {
        context_restore_gl_context(gl_info, restore_dc, restore_ctx);
    } else if !wglGetCurrentContext().is_null()
        && wglMakeCurrent(core::ptr::null_mut(), core::ptr::null_mut()) == 0
    {
        ERR!("Failed to disable GL context.\n");
    }

    wined3d_release_dc((*context_gl).window, (*context_gl).dc);

    if wglDeleteContext((*context_gl).gl_ctx) == 0 {
        let err: ::core::ffi::c_uint = GetLastError() as ::core::ffi::c_uint;
        ERR!("Failed to delete GL context %p, last error %#x.\n", (*context_gl).gl_ctx, err);
    }

    wined3d_context_cleanup(&raw mut (*context_gl).c);
}
#[no_mangle]
pub unsafe extern "C" fn context_get_tls_idx() -> DWORD {
    return wined3d_context_tls_idx;
}
#[no_mangle]
pub unsafe extern "C" fn context_set_tls_idx(idx: DWORD) {
    wined3d_context_tls_idx = idx;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_get_current() -> *mut wined3d_context_gl {
    return TlsGetValue(wined3d_context_tls_idx) as *mut wined3d_context_gl;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_set_current(context_gl: *mut wined3d_context_gl) -> BOOL {
    let old: *mut wined3d_context_gl = wined3d_context_gl_get_current();

    if old == context_gl {
        return (TRUE) as BOOL;
    }

    if !old.is_null() {
        if (*old).c.destroyed() != 0 {
            wined3d_context_gl_cleanup(old);
            free((*old).gl_info as *mut core::ffi::c_void);
            free(old as *mut core::ffi::c_void);
        } else {
            if !wglGetCurrentContext().is_null() {
                let gl_info: *const wined3d_gl_info = (*old).gl_info;
                (*gl_info)
                    .gl_ops
                    .gl
                    .p_glFlush
                    .unwrap_unchecked()();
            }
            (*old).c.set_current((0) as _);
        }
    }

    if !context_gl.is_null() {
        if (*context_gl).valid() == 0 {
            ERR!("Trying to make invalid context %p current.\n", context_gl);
            return (FALSE) as BOOL;
        }

        if wined3d_context_gl_set_gl_context(context_gl) == 0 {
            return (FALSE) as BOOL;
        }
        (*context_gl).c.set_current((1) as _);
    } else if !wglGetCurrentContext().is_null() {
        if wglMakeCurrent(core::ptr::null_mut(), core::ptr::null_mut()) == 0 {
            let err: u32 = GetLastError();
            ERR!("Failed to clear current GL context, last error %#x.\n", err);
            TlsSetValue(wined3d_context_tls_idx, core::ptr::null_mut());
            return (FALSE) as BOOL;
        }
    }

    return TlsSetValue(wined3d_context_tls_idx, context_gl as PVOID);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_release(context_gl: *mut wined3d_context_gl) {
    TRACE!("Releasing context %p, level %u.\n", context_gl, (*context_gl).level);
    if WARN_ON!(__wine_dbch_d3d) != 0 {
        if (*context_gl).level == 0 {
            WARN!("Context %p is not active.\n", context_gl);
        } else if context_gl != wined3d_context_gl_get_current() {
            WARN!("Context %p is not the current context.\n", context_gl);
        }
    }

    (*context_gl).level = (*context_gl).level.wrapping_sub(1);
    if (*context_gl).level == 0 {
        if wined3d_context_gl_restore_pixel_format(context_gl) != 0 {
            (*context_gl).set_needs_set(1 as uint32_t);
        }
        if !(*context_gl).restore_ctx.is_null() {
            TRACE!(
                "Restoring GL context %p on device context %p.\n",
                (*context_gl).restore_ctx,
                (*context_gl).restore_dc
            );
            context_restore_gl_context(
                (*context_gl).gl_info,
                (*context_gl).restore_dc,
                (*context_gl).restore_ctx,
            );
            (*context_gl).restore_ctx = core::ptr::null_mut();
            (*context_gl).restore_dc = core::ptr::null_mut();
        }

        if (*context_gl).c.destroy_delayed() != 0 {
            TRACE!("Destroying context %p.\n", context_gl);
            wined3d_context_gl_destroy(context_gl);
        }
    }
}

unsafe extern "C" fn wined3d_context_gl_enter(context_gl: *mut wined3d_context_gl) {
    WINE_TRACE!("Entering context %p, level %u.\n", context_gl, (*context_gl).level.wrapping_add(1));
    let fresh30 = (*context_gl).level;
    (*context_gl).level = (*context_gl).level.wrapping_add(1);
    if fresh30 == 0 {
        let current_context: *const wined3d_context_gl = wined3d_context_gl_get_current();
        let current_gl: HGLRC = wglGetCurrentContext();

        if !current_gl.is_null()
            && (current_context.is_null() || (*current_context).gl_ctx != current_gl)
        {
            WINE_TRACE!("Another GL context (%p on device context %p) is already current.\n", current_gl, wglGetCurrentDC());
            (*context_gl).restore_ctx = current_gl;
            (*context_gl).restore_dc = wglGetCurrentDC();
            (*context_gl).set_needs_set((1) as _);
        } else if (*context_gl).needs_set() == 0
            && !((*context_gl).dc_is_private() as i32 != 0 && (*context_gl).dc_has_format() as i32 != 0)
        {
            let current: i32 = (*(*context_gl).gl_info)
                .gl_ops
                .wgl
                .p_wglGetPixelFormat
                .unwrap_unchecked()((*context_gl).dc);

            if (current != 0 && current != (*context_gl).pixel_format)
                || (current == 0 && (*context_gl).internal_format_set() == 0)
            {
                (*context_gl).set_needs_set((1) as _);
            }
        }
    }
}

unsafe extern "C" fn context_choose_pixel_format(
    device: *const wined3d_device,
    hdc: HDC,
    color_format: *const wined3d_format,
    ds_format: *const wined3d_format,
    aux_buffers: bool,
    swap_effect_copy: bool,
) -> ::core::ffi::c_int {
    let cfg_count: ::core::ffi::c_uint =
        (*wined3d_adapter_gl((*device).adapter)).pixel_format_count;
    let mut current_value: ::core::ffi::c_uint;
    let mut pfd: PIXELFORMATDESCRIPTOR = core::mem::zeroed();
    let mut iPixelFormat: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_uint;

    TRACE!(
        "device %p, dc %p, color_format %s, ds_format %s, aux_buffers %#x, swap_effect_copy %#x.\n",
        device,
        hdc,
        debug_d3dformat((*color_format).id),
        debug_d3dformat((*ds_format).id),
        aux_buffers as ::core::ffi::c_int,
        swap_effect_copy as ::core::ffi::c_int
    );

    current_value = 0;
    i = 0;
    while i < cfg_count {
        let cfg: *const wined3d_pixel_format =
            (*wined3d_adapter_gl((*device).adapter)).pixel_formats.offset(i as isize);
        let mut value: ::core::ffi::c_uint;

        'next: {
            /* For now only accept RGBA formats. Perhaps some day we will
             * allow floating point formats for pbuffers. */
            if (*cfg).iPixelType != (WGL_TYPE_RGBA_ARB as i32) {
                break 'next;
            }
            /* In window mode we need a window drawable format and double buffering. */
            if !((*cfg).windowDrawable != 0 && (*cfg).doubleBuffer != 0) {
                break 'next;
            }
            if ((*cfg).redSize as DWORD) < (*color_format).red_size {
                break 'next;
            }
            if ((*cfg).greenSize as DWORD) < (*color_format).green_size {
                break 'next;
            }
            if ((*cfg).blueSize as DWORD) < (*color_format).blue_size {
                break 'next;
            }
            if ((*cfg).alphaSize as DWORD) < (*color_format).alpha_size {
                break 'next;
            }
            if ((*cfg).depthSize) < ((*ds_format).depth_size as ::core::ffi::c_int) {
                break 'next;
            }
            if (*ds_format).stencil_size as ::core::ffi::c_int != 0
                && (*cfg).stencilSize != (*ds_format).stencil_size as ::core::ffi::c_int
            {
                break 'next;
            }
            /* Check multisampling support. */
            if (*cfg).numSamples != 0 {
                break 'next;
            }

            value = 1;
            if swap_effect_copy && (*cfg).swap_method == (WGL_SWAP_COPY_ARB as i32) {
                value = value.wrapping_add(1);
            }
            /* We try to locate a format which matches our requirements exactly. In case of
             * depth it is no problem to emulate 16-bit using e.g. 24-bit, so accept that. */
            if (*cfg).depthSize == (*ds_format).depth_size as ::core::ffi::c_int {
                value = value.wrapping_add(2);
            }
            if (*cfg).stencilSize == (*ds_format).stencil_size as ::core::ffi::c_int {
                value = value.wrapping_add(4);
            }
            if (*cfg).alphaSize as DWORD == (*color_format).alpha_size {
                value = value.wrapping_add(8);
            }
            /* We like to have aux buffers in backbuffer mode */
            if aux_buffers && (*cfg).auxBuffers != 0 {
                value = value.wrapping_add(16);
            }
            if (*cfg).redSize as DWORD == (*color_format).red_size
                && (*cfg).greenSize as DWORD == (*color_format).green_size
                && (*cfg).blueSize as DWORD == (*color_format).blue_size
            {
                value = value.wrapping_add(32);
            }

            if value > current_value {
                iPixelFormat = (*cfg).iPixelFormat;
                current_value = value;
            }
        }
        i = i.wrapping_add(1);
    }

    if iPixelFormat == 0 {
        ERR!("Trying to locate a compatible pixel format because an exact match failed.\n");

        memset(
            core::ptr::addr_of_mut!(pfd) as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            core::mem::size_of::<PIXELFORMATDESCRIPTOR>() as size_t,
        );
        pfd.nSize = core::mem::size_of::<PIXELFORMATDESCRIPTOR>() as WORD;
        pfd.nVersion = 1 as WORD;
        pfd.dwFlags = (PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER | PFD_DRAW_TO_WINDOW)
            as DWORD; /*PFD_GENERIC_ACCELERATED*/
        pfd.iPixelType = PFD_TYPE_RGBA as BYTE;
        pfd.cAlphaBits = (*color_format).alpha_size as BYTE;
        pfd.cColorBits = (*color_format)
            .red_size
            .wrapping_add((*color_format).green_size)
            .wrapping_add((*color_format).blue_size)
            .wrapping_add((*color_format).alpha_size) as BYTE;
        pfd.cDepthBits = (*ds_format).depth_size;
        pfd.cStencilBits = (*ds_format).stencil_size;
        pfd.iLayerType = PFD_MAIN_PLANE as BYTE;

        iPixelFormat = ChoosePixelFormat(hdc, core::ptr::addr_of_mut!(pfd));
        if iPixelFormat == 0 {
            /* Something is very wrong as ChoosePixelFormat() barely fails. */
            ERR!("Can't find a suitable pixel format.\n");
            return 0;
        }
    }

    TRACE!(
        "Found iPixelFormat=%d for ColorFormat=%s, DepthStencilFormat=%s.\n",
        iPixelFormat,
        debug_d3dformat((*color_format).id),
        debug_d3dformat((*ds_format).id)
    );
    return iPixelFormat;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_bind_dummy_textures(context_gl: *const wined3d_context_gl) {
    let textures: *const wined3d_dummy_textures = core::ptr::addr_of!(
        (*wined3d_device_gl((*context_gl).c.device)).dummy_textures
    );
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut i: ::core::ffi::c_uint = 0;
    i = 0 as ::core::ffi::c_uint;
    while i < (*gl_info).limits.combined_samplers {
        ((*gl_info).gl_ops.ext.p_glActiveTexture.unwrap_unchecked())(GL_TEXTURE0 + i);

        (*gl_info)
            .gl_ops
            .gl
            .p_glBindTexture
            .unwrap_unchecked()(
            GL_TEXTURE_1D as GLenum,
            (*textures).tex_1d,
        );
        (*gl_info)
            .gl_ops
            .gl
            .p_glBindTexture
            .unwrap_unchecked()(
            GL_TEXTURE_2D as GLenum,
            (*textures).tex_2d,
        );

        if (*gl_info).supported[EXT_TEXTURE3D as usize] != 0 {
            (*gl_info)
                .gl_ops
                .gl
                .p_glBindTexture
                .unwrap_unchecked()(
                GL_TEXTURE_3D as GLenum,
                (*textures).tex_3d,
            );
        }

        if (*gl_info).supported[ARB_TEXTURE_CUBE_MAP as usize] != 0 {
            (*gl_info)
                .gl_ops
                .gl
                .p_glBindTexture
                .unwrap_unchecked()(
                GL_TEXTURE_CUBE_MAP as GLenum,
                (*textures).tex_cube,
            );
        }

        if (*gl_info).supported[ARB_TEXTURE_CUBE_MAP_ARRAY as usize] != 0 {
            (*gl_info)
                .gl_ops
                .gl
                .p_glBindTexture
                .unwrap_unchecked()(
                GL_TEXTURE_CUBE_MAP_ARRAY as GLenum,
                (*textures).tex_cube_array,
            );
        }

        if (*gl_info).supported[EXT_TEXTURE_ARRAY as usize] != 0 {
            (*gl_info)
                .gl_ops
                .gl
                .p_glBindTexture
                .unwrap_unchecked()(
                GL_TEXTURE_1D_ARRAY as GLenum,
                (*textures).tex_1d_array,
            );
            (*gl_info)
                .gl_ops
                .gl
                .p_glBindTexture
                .unwrap_unchecked()(
                GL_TEXTURE_2D_ARRAY as GLenum,
                (*textures).tex_2d_array,
            );
        }

        if (*gl_info).supported[ARB_TEXTURE_BUFFER_OBJECT as usize] != 0 {
            (*gl_info)
                .gl_ops
                .gl
                .p_glBindTexture
                .unwrap_unchecked()(
                GL_TEXTURE_BUFFER as GLenum,
                (*textures).tex_buffer,
            );
        }

        if (*gl_info).supported[ARB_TEXTURE_MULTISAMPLE as usize] != 0 {
            (*gl_info)
                .gl_ops
                .gl
                .p_glBindTexture
                .unwrap_unchecked()(
                GL_TEXTURE_2D_MULTISAMPLE as GLenum,
                (*textures).tex_2d_ms,
            );
            (*gl_info)
                .gl_ops
                .gl
                .p_glBindTexture
                .unwrap_unchecked()(
                GL_TEXTURE_2D_MULTISAMPLE_ARRAY as GLenum,
                (*textures).tex_2d_ms_array,
            );
        }

        i = i.wrapping_add(1);
    }

    checkGLcall!(gl_info, b"bind dummy textures\0".as_ptr() as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_check_gl_call(
    gl_info: *const wined3d_gl_info,
    file: *const core::ffi::c_char,
    line: core::ffi::c_uint,
    name: *const core::ffi::c_char,
) {
    let mut err: GLint;

    if (*gl_info).supported[ARB_DEBUG_OUTPUT as usize] != 0 || {
        err = (*gl_info)
            .gl_ops
            .gl
            .p_glGetError
            .unwrap_unchecked()() as GLint;
        err == (GL_NO_ERROR as GLint)
    } {
        return;
    }

    loop {
        ERR!(
            ">>>>>>> %s (%#x) from %s @ %s / %u.\n",
            debug_glerror(err as GLenum),
            err,
            name,
            file,
            line
        );
        err = (*gl_info)
            .gl_ops
            .gl
            .p_glGetError
            .unwrap_unchecked()() as GLint;
        if !(err != (GL_NO_ERROR as GLint)) {
            break;
        }
    }
}

unsafe extern "C" fn context_debug_output_enabled(gl_info: *const wined3d_gl_info) -> BOOL {
    return (((*gl_info).supported[ARB_DEBUG_OUTPUT as usize] != 0)
        && (ERR_ON!(__wine_dbch_d3d) != 0 || FIXME_ON!(__wine_dbch_d3d) != 0 || WARN_ON!(__wine_dbch_d3d_perf) != 0)) as BOOL;
}

unsafe extern "system" fn wined3d_debug_callback(
    source: GLenum,
    type_: GLenum,
    id: GLuint,
    severity: GLenum,
    length: GLsizei,
    message: *const core::ffi::c_char,
    ctx: *mut core::ffi::c_void,
) {
    match type_ {
        GL_DEBUG_TYPE_ERROR_ARB => {
            ERR!("%p: %s.\n", ctx, debugstr_an(message, length));
        }
        GL_DEBUG_TYPE_DEPRECATED_BEHAVIOR_ARB | GL_DEBUG_TYPE_UNDEFINED_BEHAVIOR_ARB | GL_DEBUG_TYPE_PORTABILITY_ARB => {
            FIXME!("%p: %s.\n", ctx, debugstr_an(message, length));
        }
        GL_DEBUG_TYPE_PERFORMANCE_ARB => {
            WINE_WARN_!(__wine_dbch_d3d_perf, "%p: %s.\n", ctx, debugstr_an(message, length));
        }
        _ => {
            FIXME!("ctx %p, type %#x: %s.\n", ctx, type_, debugstr_an(message, length));
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn context_create_wgl_attribs(gl_info: *const wined3d_gl_info, hdc: HDC, share_ctx: HGLRC) -> HGLRC {
    let mut ctx: HGLRC = core::ptr::null_mut();
    let mut ctx_attrib_idx: core::ffi::c_uint = 0;
    let mut ctx_attribs: [GLint; 7] = [0 as GLint; 7];
    let mut ctx_flags: GLint = 0 as GLint;

    if context_debug_output_enabled(gl_info) != 0 {
        ctx_flags = WGL_CONTEXT_DEBUG_BIT_ARB as GLint;
    }
    ctx_attribs[ctx_attrib_idx as usize] = WGL_CONTEXT_MAJOR_VERSION_ARB as GLint;
    ctx_attrib_idx += 1;
    ctx_attribs[ctx_attrib_idx as usize] =
        ((*gl_info).selected_gl_version >> 16) as GLint;
    ctx_attrib_idx += 1;
    ctx_attribs[ctx_attrib_idx as usize] = WGL_CONTEXT_MINOR_VERSION_ARB as GLint;
    ctx_attrib_idx += 1;
    ctx_attribs[ctx_attrib_idx as usize] =
        ((*gl_info).selected_gl_version & 0xffff) as GLint;
    ctx_attrib_idx += 1;
    if ctx_flags != 0 {
        ctx_attribs[ctx_attrib_idx as usize] = WGL_CONTEXT_FLAGS_ARB as GLint;
        ctx_attrib_idx += 1;
        ctx_attribs[ctx_attrib_idx as usize] = ctx_flags;
        ctx_attrib_idx += 1;
    }
    ctx_attribs[ctx_attrib_idx as usize] = 0 as GLint;

    ctx = (*gl_info)
        .p_wglCreateContextAttribsARB
        .unwrap_unchecked()(hdc, share_ctx, ctx_attribs.as_mut_ptr());
    if ctx.is_null() {
        if (*gl_info).selected_gl_version
            >= MAKEDWORD_VERSION!(3, 2)
        {
            if ctx_flags != 0 {
                ctx_flags |= WGL_CONTEXT_FORWARD_COMPATIBLE_BIT_ARB as GLint;
                ctx_attribs[(ctx_attrib_idx - 1) as usize] = ctx_flags;
            } else {
                ctx_flags = WGL_CONTEXT_FORWARD_COMPATIBLE_BIT_ARB as GLint;
                ctx_attribs[ctx_attrib_idx as usize] = WGL_CONTEXT_FLAGS_ARB as GLint;
                ctx_attrib_idx += 1;
                ctx_attribs[ctx_attrib_idx as usize] = ctx_flags;
                ctx_attrib_idx += 1;
                ctx_attribs[ctx_attrib_idx as usize] = 0 as GLint;
            }
            ctx = (*gl_info)
                .p_wglCreateContextAttribsARB
                .unwrap_unchecked()(hdc, share_ctx, ctx_attribs.as_mut_ptr());
            if ctx.is_null() {
                WARN!(
                    "Failed to create a WGL context with wglCreateContextAttribsARB, last error %#lx.\n",
                    GetLastError()
                );
            }
        }
    }
    ctx
}

unsafe extern "C" fn wined3d_context_gl_create_wgl_ctx(
    context_gl: *mut wined3d_context_gl,
    swapchain_gl: *mut wined3d_swapchain_gl,
    new_drawable: *mut BOOL,
) -> BOOL {
    let mut swap_effect: wined3d_swap_effect = (*swapchain_gl).s.state.desc.swap_effect;
    let mut colour_format: *const wined3d_format = core::ptr::null();
    let mut ds_format: *const wined3d_format = core::ptr::null();
    let mut context: *mut wined3d_context = core::ptr::addr_of_mut!((*context_gl).c);
    let mut gl_info: *const wined3d_gl_info = core::ptr::null();
    let mut target: *mut wined3d_resource = core::ptr::null_mut();
    let mut adapter: *mut wined3d_adapter = core::ptr::null_mut();
    let mut target_bind_flags: u32 = 0;
    let mut device: *mut wined3d_device = core::ptr::null_mut();
    let mut swap_effect_copy: bool = false;
    let mut ctx: HGLRC = core::ptr::null_mut();
    let mut share_ctx: HGLRC = core::ptr::null_mut();

    device = (*context).device;
    adapter = (*device).adapter;
    gl_info = core::ptr::addr_of!((*wined3d_adapter_gl(adapter)).gl_info);

    target = core::ptr::addr_of_mut!((*(*context).current_rt.texture).resource);
    target_bind_flags = (*target).bind_flags;

    swap_effect_copy = swap_effect as u32 == WINED3D_SWAP_EFFECT_COPY
        || swap_effect as u32 == WINED3D_SWAP_EFFECT_COPY_VSYNC;

    *new_drawable = (GetPixelFormat((*context_gl).dc) == 0) as BOOL;

    colour_format = wined3d_get_format(adapter, WINED3DFMT_B8G8R8A8_UNORM, target_bind_flags);
    ds_format = wined3d_get_format(
        adapter,
        WINED3DFMT_UNKNOWN,
        WINED3D_BIND_DEPTH_STENCIL,
    );
    (*context_gl).pixel_format = context_choose_pixel_format(
        device,
        (*context_gl).dc,
        colour_format,
        ds_format,
        false,
        swap_effect_copy,
    );

    if (*context_gl).pixel_format == 0 {
        ERR!("Failed to choose pixel format.\n");
        return (FALSE) as BOOL;
    }

    wined3d_context_gl_enter(context_gl);

    if wined3d_context_gl_set_pixel_format(context_gl) == 0 {
        context_release(context);

        if (*context_gl).dc_is_private() != 0 {
            ERR!(
                "Failed to set pixel format %d on device context %p.\n",
                (*context_gl).pixel_format,
                (*context_gl).dc
            );

            return (FALSE) as BOOL;
        }

        WARN!(
            "Failed to set pixel format %d on device context %p, trying backup DC.\n",
            (*context_gl).pixel_format,
            (*context_gl).dc
        );

        wined3d_release_dc((*context_gl).window, (*context_gl).dc);
        (*context_gl).dc = wined3d_device_gl_get_backup_dc(wined3d_device_gl(device));
        if (*context_gl).dc.is_null() {
            ERR!("Failed to retrieve the backup device context.\n");
            return (FALSE) as BOOL;
        }
        (*context_gl).set_dc_is_private(TRUE as u32);

        return wined3d_context_gl_create_wgl_ctx(context_gl, swapchain_gl, new_drawable);
    }

    share_ctx = if (*device).context_count != 0 {
        (*wined3d_context_gl(
            *(*device).contexts.offset(0 as isize),
        ))
        .gl_ctx
    } else {
        core::ptr::null_mut()
    };

    if (*gl_info).p_wglCreateContextAttribsARB.is_some() {
        ctx = context_create_wgl_attribs(gl_info, (*context_gl).dc, share_ctx);
        if ctx.is_null() {
            ERR!("Failed to create a WGL context.\n");
            context_release(context);
            return (FALSE) as BOOL;
        }
    } else {
        ctx = wglCreateContext((*context_gl).dc);
        if ctx.is_null() {
            ERR!("Failed to create a WGL context.\n");
            context_release(context);
            return (FALSE) as BOOL;
        }

        if !share_ctx.is_null() && wglShareLists(share_ctx, ctx) == 0 {
            ERR!(
                "wglShareLists(%p, %p) failed, last error %#lx.\n",
                share_ctx,
                ctx,
                GetLastError()
            );
            context_release(context);
            if wglDeleteContext(ctx) == 0 {
                ERR!(
                    "wglDeleteContext(%p) failed, last error %#lx.\n",
                    ctx,
                    GetLastError()
                );
            }
            return (FALSE) as BOOL;
        }
    }

    (*context_gl).set_dc_has_format(TRUE as u32);
    (*context_gl).set_needs_set(1 as u32);
    (*context_gl).set_valid(1 as u32);
    (*context_gl).gl_ctx = ctx;

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_init(context_gl: *mut wined3d_context_gl, swapchain_gl: *mut wined3d_swapchain_gl) -> HRESULT {
    let context: *mut wined3d_context = &raw mut (*context_gl).c;
    let mut d3d_info: *const wined3d_d3d_info = core::ptr::null();
    let mut gl_info: *const wined3d_gl_info = core::ptr::null();
    let mut device: *mut wined3d_device = core::ptr::null_mut();
    let mut new_drawable: BOOL = 0;
    let mut i: u32 = 0;

    wined3d_context_init(&raw mut (*context_gl).c, &raw mut (*swapchain_gl).s);

    device = (*context).device;
    gl_info = &raw const (*wined3d_adapter_gl((*device).adapter)).gl_info;
    (*context_gl).gl_info = gl_info;
    d3d_info = (*context).d3d_info;

    (*context_gl).tid = GetCurrentThreadId();
    (*context_gl).window = (*(*context).swapchain).win_handle;
    if (*context_gl).window == GetDesktopWindow() {
        (*context_gl).dc = core::ptr::null_mut();
    } else {
        (*context_gl).dc = GetDCEx(
            (*context_gl).window,
            core::ptr::null_mut(),
            DCX_USESTYLE | DCX_CACHE,
        );
        if (*context_gl).dc.is_null() {
            WARN!("Failed to retrieve device context, trying swapchain backup.\n");
        }
    }

    if (*context_gl).dc.is_null() {
        (*context_gl).dc = wined3d_device_gl_get_backup_dc(wined3d_device_gl(device));
        if (*context_gl).dc.is_null() {
            ERR!("Failed to retrieve a device context.\n");
            return E_FAIL;
        }
        (*context_gl).set_dc_is_private((TRUE) as _);
    }

    list_init(&raw mut (*context_gl).fbo_list);
    list_init(&raw mut (*context_gl).fbo_destroy_list);

    list_init(&raw mut (*context_gl).occlusion_queries);
    list_init(&raw mut (*context_gl).fences);
    list_init(&raw mut (*context_gl).timestamp_queries);
    list_init(&raw mut (*context_gl).so_statistics_queries);
    list_init(&raw mut (*context_gl).pipeline_statistics_queries);

    i = 0;
    while (i as usize) < ARRAY_SIZE!((*context_gl).tex_unit_map) as usize {
        (*context_gl).tex_unit_map[i as usize] = (WINED3D_UNMAPPED_STAGE) as u32;
        i += 1;
    }
    i = 0;
    while (i as usize) < ARRAY_SIZE!((*context_gl).rev_tex_unit_map) as usize {
        (*context_gl).rev_tex_unit_map[i as usize] = (WINED3D_UNMAPPED_STAGE) as u32;
        i += 1;
    }

    'fail: {
        if (*gl_info).limits.graphics_samplers >= WINED3D_MAX_COMBINED_SAMPLERS {
            /* Initialize the texture unit mapping to a 1:1 mapping. */
            let mut base: u32 = 0;
            let mut count: u32 = 0;

            wined3d_gl_limits_get_texture_unit_range(
                &raw const (*gl_info).limits,
                WINED3D_SHADER_TYPE_PIXEL,
                &raw mut base,
                &raw mut count,
            );
            if (base as usize) + (WINED3D_MAX_FRAGMENT_SAMPLERS as usize)
                > ARRAY_SIZE!((*context_gl).rev_tex_unit_map) as usize
            {
                ERR!("Unexpected texture unit base index %u.\n", base);
                break 'fail;
            }
            i = 0;
            while i < min!(count, WINED3D_MAX_FRAGMENT_SAMPLERS) {
                (*context_gl).tex_unit_map[i as usize] = base + i;
                (*context_gl).rev_tex_unit_map[(base + i) as usize] = i;
                i += 1;
            }

            wined3d_gl_limits_get_texture_unit_range(
                &raw const (*gl_info).limits,
                WINED3D_SHADER_TYPE_VERTEX,
                &raw mut base,
                &raw mut count,
            );
            if (base as usize) + (WINED3D_MAX_VERTEX_SAMPLERS as usize)
                > ARRAY_SIZE!((*context_gl).rev_tex_unit_map) as usize
            {
                ERR!("Unexpected texture unit base index %u.\n", base);
                break 'fail;
            }
            i = 0;
            while i < min!(count, WINED3D_MAX_VERTEX_SAMPLERS) {
                (*context_gl).tex_unit_map[(WINED3D_MAX_FRAGMENT_SAMPLERS + i) as usize] = base + i;
                (*context_gl).rev_tex_unit_map[(base + i) as usize] = WINED3D_MAX_FRAGMENT_SAMPLERS + i;
                i += 1;
            }
        }

        (*context_gl).texture_type = calloc(
            ((*gl_info).limits.combined_samplers as usize) as _,
            (core::mem::size_of::<GLenum>()) as _,
        ) as *mut GLenum;
        if (*context_gl).texture_type.is_null() {
            break 'fail;
        }

        if wined3d_context_gl_create_wgl_ctx(context_gl, swapchain_gl, &raw mut new_drawable) == 0 {
            break 'fail;
        }

        /* Set up the context defaults. */

        (*context_gl).draw_buffers_mask =
            context_generate_rt_mask(GL_BACK) as u32;

        if wined3d_context_gl_set_current(context_gl) == 0 {
            ERR!("Cannot activate context to set up defaults.\n");
            context_release(context);
            if wglDeleteContext((*context_gl).gl_ctx) == 0 {
                ERR!(
                    "wglDeleteContext(%p) failed, last error %#lx.\n",
                    (*context_gl).gl_ctx,
                    GetLastError()
                );
            }
            break 'fail;
        }

        if context_debug_output_enabled(gl_info) != 0 {
            ((*gl_info).gl_ops.ext.p_glDebugMessageCallback.unwrap_unchecked())((wined3d_debug_callback as *mut core::ffi::c_void), context as *const core::ffi::c_void);
            if ((TRACE_ON!(__wine_dbch_d3d_sync)) != 0) {
                (*gl_info).gl_ops.gl.p_glEnable.unwrap_unchecked()(
                    GL_DEBUG_OUTPUT_SYNCHRONOUS,
                );
            }
            ((*gl_info).gl_ops.ext.p_glDebugMessageControl.unwrap_unchecked())(
                GL_DONT_CARE,
                GL_DONT_CARE,
                GL_DONT_CARE,
                0,
                core::ptr::null(),
                (GL_FALSE as u8),
            );
            if ((ERR_ON!(__wine_dbch_d3d)) != 0) {
                ((*gl_info).gl_ops.ext.p_glDebugMessageControl.unwrap_unchecked())(
                    GL_DONT_CARE,
                    GL_DEBUG_TYPE_ERROR,
                    GL_DONT_CARE,
                    0,
                    core::ptr::null(),
                    (GL_TRUE as u8),
                );
            }
            if ((FIXME_ON!(__wine_dbch_d3d)) != 0) {
                ((*gl_info).gl_ops.ext.p_glDebugMessageControl.unwrap_unchecked())(
                    GL_DONT_CARE,
                    GL_DEBUG_TYPE_DEPRECATED_BEHAVIOR,
                    GL_DONT_CARE,
                    0,
                    core::ptr::null(),
                    (GL_TRUE as u8),
                );
                ((*gl_info).gl_ops.ext.p_glDebugMessageControl.unwrap_unchecked())(
                    GL_DONT_CARE,
                    GL_DEBUG_TYPE_UNDEFINED_BEHAVIOR,
                    GL_DONT_CARE,
                    0,
                    core::ptr::null(),
                    (GL_TRUE as u8),
                );
                ((*gl_info).gl_ops.ext.p_glDebugMessageControl.unwrap_unchecked())(
                    GL_DONT_CARE,
                    GL_DEBUG_TYPE_PORTABILITY,
                    GL_DONT_CARE,
                    0,
                    core::ptr::null(),
                    (GL_TRUE as u8),
                );
            }
            if ((WARN_ON!(__wine_dbch_d3d_perf)) != 0) {
                ((*gl_info).gl_ops.ext.p_glDebugMessageControl.unwrap_unchecked())(
                    GL_DONT_CARE,
                    GL_DEBUG_TYPE_PERFORMANCE,
                    GL_DONT_CARE,
                    0,
                    core::ptr::null(),
                    (GL_TRUE as u8),
                );
            }
        }

        if (*gl_info).supported[WINED3D_GL_LEGACY_CONTEXT as usize] != 0 {
            (*gl_info).gl_ops.gl.p_glGetIntegerv.unwrap_unchecked()(
                GL_AUX_BUFFERS,
                &raw mut (*context_gl).aux_buffers,
            );
        }

        if (*gl_info).supported[WINED3D_GL_LEGACY_CONTEXT as usize] != 0 {
            (*gl_info).gl_ops.gl.p_glLightModeli.unwrap_unchecked()(
                GL_LIGHT_MODEL_LOCAL_VIEWER,
                (GL_TRUE as i32),
            );
            checkGLcall!(gl_info, "glLightModeli(GL_LIGHT_MODEL_LOCAL_VIEWER, GL_TRUE);");

            (*gl_info).gl_ops.gl.p_glTexEnvf.unwrap_unchecked()(
                GL_TEXTURE_ENV,
                GL_TEXTURE_ENV_MODE,
                GL_COMBINE_EXT as f32,
            );
            checkGLcall!(gl_info, "glTexEnvf(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, GL_COMBINE_EXT);");

            (*gl_info).gl_ops.gl.p_glLightModeli.unwrap_unchecked()(
                GL_LIGHT_MODEL_COLOR_CONTROL,
                (GL_SEPARATE_SPECULAR_COLOR as i32),
            );
            checkGLcall!(gl_info, "glLightModeli(GL_LIGHT_MODEL_COLOR_CONTROL, GL_SEPARATE_SPECULAR_COLOR);");
        } else {
            let mut vao: GLuint = 0;

            ((*gl_info).gl_ops.ext.p_glGenVertexArrays.unwrap_unchecked())(1, &raw mut vao);
            ((*gl_info).gl_ops.ext.p_glBindVertexArray.unwrap_unchecked())(vao);
            checkGLcall!(gl_info, "creating VAO");
        }

        (*gl_info).gl_ops.gl.p_glPixelStorei.unwrap_unchecked()(
            GL_PACK_ALIGNMENT,
            ((*device).surface_alignment) as _,
        );
        checkGLcall!(gl_info, "glPixelStorei(GL_PACK_ALIGNMENT, device->surface_alignment);");
        (*gl_info).gl_ops.gl.p_glPixelStorei.unwrap_unchecked()(
            GL_UNPACK_ALIGNMENT,
            1,
        );
        checkGLcall!(gl_info, "glPixelStorei(GL_UNPACK_ALIGNMENT, 1);");

        if (*gl_info).supported[NV_TEXTURE_SHADER2 as usize] != 0 {
            /* Set up the previous texture input for all shader units. This applies to bump mapping, and in d3d
             * the previous texture where to source the offset from is always unit - 1.
             */
            i = 1;
            while i < (*gl_info).limits.ffp_textures {
                wined3d_context_gl_active_texture(context_gl, gl_info, i);
                (*gl_info).gl_ops.gl.p_glTexEnvi.unwrap_unchecked()(
                    GL_TEXTURE_SHADER_NV,
                    GL_PREVIOUS_TEXTURE_INPUT_NV,
                    (GL_TEXTURE0_ARB + i - 1) as GLint,
                );
                checkGLcall!(gl_info, "glTexEnvi(GL_TEXTURE_SHADER_NV, GL_PREVIOUS_TEXTURE_INPUT_NV, ...");
                i += 1;
            }
        }
        if (*gl_info).supported[ARB_FRAGMENT_PROGRAM as usize] != 0 {
            /* MacOS(radeon X1600 at least, but most likely others too) refuses to draw if GLSL and ARBFP are
             * enabled, but the currently bound arbfp program is 0. Enabling ARBFP with prog 0 is invalid, but
             * GLSL should bypass this. This causes problems in programs that never use the fixed function pipeline,
             * because the ARBFP extension is enabled by the ARBFP pipeline at context creation, but no program
             * is ever assigned.
             *
             * So make sure a program is assigned to each context. The first real ARBFP use will set a different
             * program and the dummy program is destroyed when the context is destroyed.
             */
            static dummy_program: [::core::ffi::c_char; 58] = unsafe {
                ::core::mem::transmute::<[u8; 58], [::core::ffi::c_char; 58]>(
                    *b"!!ARBfp1.0\nMOV result.color, fragment.color.primary;\nEND\n\0",
                )
            };
            ((*gl_info).gl_ops.ext.p_glGenProgramsARB.unwrap_unchecked())(
                1,
                &raw mut (*context_gl).dummy_arbfp_prog,
            );
            ((*gl_info).gl_ops.ext.p_glBindProgramARB.unwrap_unchecked())(
                GL_FRAGMENT_PROGRAM_ARB,
                (*context_gl).dummy_arbfp_prog,
            );
            ((*gl_info).gl_ops.ext.p_glProgramStringARB.unwrap_unchecked())(
                GL_FRAGMENT_PROGRAM_ARB,
                GL_PROGRAM_FORMAT_ASCII_ARB,
                strlen(&raw const dummy_program as *const ::core::ffi::c_char)
                    as GLsizei,
                &raw const dummy_program as *const ::core::ffi::c_void,
            );
        }

        if (*gl_info).supported[ARB_POINT_SPRITE as usize] != 0 {
            /* The actual effect of D3DRS_POINTSPRITEENABLE is more accurately
             * congruent to GL_COORD_REPLACE_ARB. GL_POINT_SPRITE_ARB, when
             * disabled, affects point rasterization (which we do not want),
             * implicitly disables GL_COORD_REPLACE_ARB, and causes gl_PointCoord
             * to return an undefined value.
             *
             * We implement D3DRS_POINTSPRITEENABLE by altering the shader to read
             * from gl_PointCoord or from the actual texcoord, so we do not need
             * GL_COORD_REPLACE_ARB, nor do we need to dynamically toggle
             * GL_POINT_SPRITE_ARB. */

            (*gl_info).gl_ops.gl.p_glEnable.unwrap_unchecked()(
                GL_POINT_SPRITE_ARB,
            );

            i = 0;
            while i < (*gl_info).limits.ffp_textures {
                wined3d_context_gl_active_texture(context_gl, gl_info, i);
                (*gl_info).gl_ops.gl.p_glTexEnvi.unwrap_unchecked()(
                    GL_POINT_SPRITE_ARB,
                    GL_COORD_REPLACE_ARB,
                    (GL_TRUE as i32),
                );
                checkGLcall!(gl_info, "glTexEnvi(GL_POINT_SPRITE_ARB, GL_COORD_REPLACE_ARB, GL_TRUE)");
                i += 1;
            }
        }

        if (*gl_info).supported[ARB_CLIP_CONTROL as usize] == 0
            && (*gl_info).supported[WINED3D_GL_VERSION_2_0 as usize] != 0
        {
            ((*gl_info).gl_ops.ext.p_glPointParameteri.unwrap_unchecked())(
                GL_POINT_SPRITE_COORD_ORIGIN,
                (GL_LOWER_LEFT as i32),
            );
            checkGLcall!(gl_info, "glPointParameteri(GL_POINT_SPRITE_COORD_ORIGIN, GL_LOWER_LEFT)");
        }

        if (*gl_info).supported[ARB_PROVOKING_VERTEX as usize] != 0 {
            ((*gl_info).gl_ops.ext.p_glProvokingVertex.unwrap_unchecked())(GL_FIRST_VERTEX_CONVENTION);
        } else if (*gl_info).supported[EXT_PROVOKING_VERTEX as usize] != 0 {
            ((*gl_info).gl_ops.ext.p_glProvokingVertexEXT.unwrap_unchecked())(GL_FIRST_VERTEX_CONVENTION_EXT);
        }
        if ((*d3d_info).wined3d_creation_flags & WINED3D_NO_PRIMITIVE_RESTART) == 0 {
            if (*gl_info).supported[ARB_ES3_COMPATIBILITY as usize] != 0 {
                (*gl_info).gl_ops.gl.p_glEnable.unwrap_unchecked()(
                    GL_PRIMITIVE_RESTART_FIXED_INDEX,
                );
                checkGLcall!(gl_info, "enable GL_PRIMITIVE_RESTART_FIXED_INDEX");
            } else {
                /* empty */
            }
        }
        if ((*d3d_info).wined3d_creation_flags & WINED3D_LEGACY_CUBEMAP_FILTERING) == 0
            && (*gl_info).supported[ARB_SEAMLESS_CUBE_MAP as usize] != 0
        {
            (*gl_info).gl_ops.gl.p_glEnable.unwrap_unchecked()(
                GL_TEXTURE_CUBE_MAP_SEAMLESS,
            );
            checkGLcall!(gl_info, "enable seamless cube map filtering");
        }
        if (*gl_info).supported[ARB_CLIP_CONTROL as usize] != 0 {
            ((*gl_info).gl_ops.ext.p_glPointParameteri.unwrap_unchecked())(
                GL_POINT_SPRITE_COORD_ORIGIN,
                (GL_LOWER_LEFT as i32),
            );
        }

        /* If this happens to be the first context for the device, dummy textures
         * are not created yet. In that case, they will be created (and bound) by
         * create_dummy_textures right after this context is initialized. */
        if (*wined3d_device_gl(device)).dummy_textures.tex_2d != 0 {
            wined3d_context_gl_bind_dummy_textures(context_gl);
        }

        /* Initialise all rectangles to avoid resetting unused ones later. */
        (*gl_info).gl_ops.gl.p_glScissor.unwrap_unchecked()(
            0,
            0,
            0,
            0,
        );
        checkGLcall!(gl_info, "glScissor");

        if new_drawable != 0 {
            (*gl_info).gl_ops.gl.p_glClearColor.unwrap_unchecked()(
                0.0f32, 0.0f32, 0.0f32, 0.0f32,
            );
            (*gl_info).gl_ops.gl.p_glClearDepth.unwrap_unchecked()(1.0f64);
            (*gl_info).gl_ops.gl.p_glClearStencil.unwrap_unchecked()(
                0,
            );
            (*gl_info).gl_ops.gl.p_glClear.unwrap_unchecked()(
                GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT | GL_STENCIL_BUFFER_BIT,
            );
            checkGLcall!(gl_info, "glClear");
        }
        return WINED3D_OK;
    }

    free((*context_gl).texture_type as *mut ::core::ffi::c_void);
    wined3d_release_dc((*context_gl).window, (*context_gl).dc);
    return E_FAIL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_destroy(context_gl: *mut wined3d_context_gl) {
    let device: *mut wined3d_device = (*context_gl).c.device;

    wined3d_from_cs((*device).cs);

    /* We delay destroying a context when it is active. The context_release()
     * function invokes wined3d_context_gl_destroy() again while leaving the
     * last level. */
    if (*context_gl).level != 0 {
        (*context_gl).c.set_destroy_delayed((1) as _);
        /* FIXME: Get rid of a pointer to swapchain from wined3d_context. */
        (*context_gl).c.swapchain = core::ptr::null_mut();
        (*context_gl).c.device = core::ptr::null_mut();
        return;
    }

    device_context_remove(device, core::ptr::addr_of_mut!((*context_gl).c));

    if (*context_gl).c.current() != 0 && (*context_gl).tid != GetCurrentThreadId() {
        let mut gl_info: *mut wined3d_gl_info = core::ptr::null_mut();

        /* Make a copy of gl_info for wined3d_context_gl_cleanup() use, the
         * one in wined3d_adapter may go away in the meantime. */
        gl_info = malloc(core::mem::size_of::<wined3d_gl_info>() as usize) as *mut wined3d_gl_info;
        *gl_info = *(*context_gl).gl_info;
        (*context_gl).gl_info = gl_info;
        (*context_gl).c.set_destroyed((1) as _);

        return;
    }

    wined3d_context_gl_cleanup(context_gl);
    TlsSetValue(context_get_tls_idx(), core::ptr::null_mut());
    free(context_gl as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_get_tex_unit_mapping(context_gl: *const wined3d_context_gl, shader_version: *const wined3d_shader_version, base: *mut ::core::ffi::c_uint, count: *mut ::core::ffi::c_uint) -> *const ::core::ffi::c_uint {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;

    if shader_version.is_null() {
        *base = 0;
        *count = WINED3D_MAX_FFP_TEXTURES;
        return core::ptr::addr_of!((*context_gl).tex_unit_map) as *const u32;
    }

    if ((*shader_version).major as i32) >= 4 {
        wined3d_gl_limits_get_texture_unit_range(
            core::ptr::addr_of!((*gl_info).limits),
            (*shader_version).type_,
            base,
            count,
        );
        return core::ptr::null::<u32>();
    }

    let ty: u32 = (*shader_version).type_ as u32;
    if ty == WINED3D_SHADER_TYPE_PIXEL {
        *base = 0;
        *count = WINED3D_MAX_FRAGMENT_SAMPLERS;
    } else if ty == WINED3D_SHADER_TYPE_VERTEX {
        *base = WINED3D_MAX_FRAGMENT_SAMPLERS;
        *count = WINED3D_MAX_VERTEX_SAMPLERS;
    } else {
        ERR!("Unhandled shader type %#x.\n", (*shader_version).type_ as u32);
        *base = 0;
        *count = 0;
    }

    core::ptr::addr_of!((*context_gl).tex_unit_map) as *const u32
}

unsafe extern "C" fn wined3d_context_gl_get_rt_size(context_gl: *const wined3d_context_gl, size: *mut SIZE) {
    let rt: *const wined3d_texture = (*context_gl).c.current_rt.texture;
    let mut level: u32;

    if !(*rt).swapchain.is_null() {
        let mut window_size: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };

        GetClientRect((*context_gl).window, core::ptr::addr_of_mut!(window_size));
        (*size).cx = window_size.right - window_size.left;
        (*size).cy = window_size.bottom - window_size.top;

        return;
    }

    level = (*context_gl).c.current_rt.sub_resource_idx % (*rt).level_count;
    (*size).cx = wined3d_texture_get_level_width(rt, level) as LONG;
    (*size).cy = wined3d_texture_get_level_height(rt, level) as LONG;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_enable_clip_distances(context_gl: *mut wined3d_context_gl, mut enable_mask: uint32_t) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut clip_distance_count: ::core::ffi::c_uint;
    let mut i: ::core::ffi::c_uint;
    let mut disable_mask: uint32_t;
    let mut current_mask: uint32_t;

    clip_distance_count = (*gl_info).limits.user_clip_distances;
    disable_mask = !enable_mask;
    enable_mask &= wined3d_mask_from_size(clip_distance_count);
    disable_mask &= wined3d_mask_from_size(clip_distance_count);
    current_mask = (*context_gl).c.clip_distance_mask();
    (*context_gl).c.set_clip_distance_mask((enable_mask) as _);

    enable_mask &= !current_mask;
    while enable_mask != 0 {
        i = wined3d_bit_scan(core::ptr::addr_of_mut!(enable_mask)) as ::core::ffi::c_uint;
        (((*gl_info).gl_ops.gl.p_glEnable).unwrap_unchecked())((GL_CLIP_DISTANCE0 + i) as GLenum);
    }
    disable_mask &= current_mask;
    while disable_mask != 0 {
        i = wined3d_bit_scan(core::ptr::addr_of_mut!(disable_mask)) as ::core::ffi::c_uint;
        (((*gl_info).gl_ops.gl.p_glDisable).unwrap_unchecked())((GL_CLIP_DISTANCE0 + i) as GLenum);
    }
    checkGLcall!(gl_info, "toggle clip distances");
}

unsafe extern "C" fn is_rt_mask_onscreen(mut rt_mask: DWORD) -> BOOL {
    return (rt_mask & (1 << 31)) as BOOL;
}

unsafe extern "C" fn draw_buffer_from_rt_mask(mut rt_mask: DWORD) -> GLenum {
    return rt_mask as GLenum & !((1 as GLenum) << 31);
}

unsafe extern "C" fn wined3d_context_gl_apply_draw_buffers(context_gl: *mut wined3d_context_gl, mut rt_mask: uint32_t) {
    let mut gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut draw_buffers: [GLenum; (WINED3D_MAX_RENDER_TARGETS as _)] = [0; (WINED3D_MAX_RENDER_TARGETS as _)];

    if rt_mask == 0 {
        (*gl_info)
            .gl_ops
            .gl
            .p_glDrawBuffer
            .unwrap_unchecked()(GL_NONE as GLenum);
    } else if is_rt_mask_onscreen(rt_mask as DWORD) != 0 {
        (*gl_info)
            .gl_ops
            .gl
            .p_glDrawBuffer
            .unwrap_unchecked()(draw_buffer_from_rt_mask(rt_mask as DWORD));
    } else {
        let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;

        while rt_mask != 0 {
            if (rt_mask & 1 as uint32_t) != 0 {
                draw_buffers[i as usize] =
                    (GL_COLOR_ATTACHMENT0 as ::core::ffi::c_uint).wrapping_add(i) as GLenum;
            } else {
                draw_buffers[i as usize] = GL_NONE as GLenum;
            }
            rt_mask >>= 1 as ::core::ffi::c_int;
            i = i.wrapping_add(1);
        }

        if (*gl_info).supported[ARB_DRAW_BUFFERS as usize] != 0 {
            ((*gl_info).gl_ops.ext.p_glDrawBuffers.unwrap_unchecked())(
                i as GLsizei,
                &raw mut draw_buffers as *mut GLenum,
            );
        } else {
            (*gl_info)
                .gl_ops
                .gl
                .p_glDrawBuffer
                .unwrap_unchecked()(
                draw_buffers[0 as usize],
            );
        }
    }

    checkGLcall!(gl_info, "apply draw buffers");
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_active_texture(context_gl: *mut wined3d_context_gl, gl_info: *const wined3d_gl_info, unit: ::core::ffi::c_uint) {
    ((*gl_info).gl_ops.ext.p_glActiveTexture.unwrap_unchecked())((GL_TEXTURE0 as GLenum).wrapping_add(unit as GLenum));
    checkGLcall!(gl_info, b"glActiveTexture\0".as_ptr() as *const core::ffi::c_char);
    (*context_gl).active_texture = unit;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_bind_bo(context_gl: *mut wined3d_context_gl, binding: GLenum, name: GLuint) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;

    if binding == GL_ELEMENT_ARRAY_BUFFER {
        context_invalidate_state(core::ptr::addr_of_mut!((*context_gl).c), (STATE_INDEXBUFFER as u32));
    }

    ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(binding, name);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_bind_texture(context_gl: *mut wined3d_context_gl, mut target: GLenum, name: GLuint) {
    let textures: *const wined3d_dummy_textures = core::ptr::addr_of!(
        (*wined3d_device_gl((*context_gl).c.device)).dummy_textures
    );
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut old_texture_type: GLenum;
    let mut unit: u32;

    if name != 0 {
        (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(target, name);
    } else {
        target = GL_NONE;
    }

    unit = (*context_gl).active_texture;
    old_texture_type = *(*context_gl).texture_type.offset(unit as isize);
    if old_texture_type != target {
        match old_texture_type {
            GL_NONE => {
                // nothing to do
            }
            GL_TEXTURE_1D => {
                (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
                    GL_TEXTURE_1D as GLenum,
                    (*textures).tex_1d,
                );
            }
            GL_TEXTURE_1D_ARRAY => {
                (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
                    GL_TEXTURE_1D_ARRAY as GLenum,
                    (*textures).tex_1d_array,
                );
            }
            GL_TEXTURE_2D => {
                (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
                    GL_TEXTURE_2D as GLenum,
                    (*textures).tex_2d,
                );
            }
            GL_TEXTURE_2D_ARRAY => {
                (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
                    GL_TEXTURE_2D_ARRAY as GLenum,
                    (*textures).tex_2d_array,
                );
            }
            GL_TEXTURE_CUBE_MAP => {
                (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
                    GL_TEXTURE_CUBE_MAP as GLenum,
                    (*textures).tex_cube,
                );
            }
            GL_TEXTURE_CUBE_MAP_ARRAY => {
                (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
                    GL_TEXTURE_CUBE_MAP_ARRAY as GLenum,
                    (*textures).tex_cube_array,
                );
            }
            GL_TEXTURE_3D => {
                (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
                    GL_TEXTURE_3D as GLenum,
                    (*textures).tex_3d,
                );
            }
            GL_TEXTURE_BUFFER => {
                (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
                    GL_TEXTURE_BUFFER as GLenum,
                    (*textures).tex_buffer,
                );
            }
            GL_TEXTURE_2D_MULTISAMPLE => {
                (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
                    GL_TEXTURE_2D_MULTISAMPLE as GLenum,
                    (*textures).tex_2d_ms,
                );
            }
            GL_TEXTURE_2D_MULTISAMPLE_ARRAY => {
                (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
                    GL_TEXTURE_2D_MULTISAMPLE_ARRAY as GLenum,
                    (*textures).tex_2d_ms_array,
                );
            }
            _ => {
                ERR!("Unexpected texture target %#x.\n", old_texture_type);
            }
        }

        *(*context_gl).texture_type.offset(unit as isize) = target;
    }

    checkGLcall!(gl_info, b"bind texture\0".as_ptr() as *const i8);
}

unsafe extern "C" fn wined3d_context_gl_poll_fences(context_gl: *mut wined3d_context_gl) {
    let device_gl: *mut wined3d_device_gl = wined3d_device_gl((*context_gl).c.device);
    let mut f: *mut wined3d_command_fence_gl = core::ptr::null_mut();
    let mut i: SIZE_T = 0;
    while i < (*context_gl).submitted.fence_count {
        f = (*context_gl).submitted.fences.add(i as usize);
        'body: {
            if (*f).id > (*device_gl).completed_fence_id {
                if wined3d_fence_test(
                    (*f).fence,
                    core::ptr::addr_of_mut!((*device_gl).d),
                    0,
                ) != WINED3D_FENCE_OK
                {
                    break 'body;
                }
                (*device_gl).completed_fence_id = (*f).id;
            }

            wined3d_fence_destroy((*f).fence);
            if i != (*context_gl).submitted.fence_count - 1 {
                *f = *(*context_gl)
                    .submitted
                    .fences
                    .add(((*context_gl).submitted.fence_count - 1) as usize);
            }
            (*context_gl).submitted.fence_count -= 1;
        }
        i += 1;
    }
}

unsafe extern "C" fn wined3d_device_gl_free_memory(
    mut device_gl: *mut wined3d_device_gl,
    mut block: *mut wined3d_allocator_block,
) {
    ASSERT!((*(*block).chunk).allocator == &raw mut (*device_gl).allocator);
    wined3d_device_gl_allocator_lock(device_gl);
    wined3d_allocator_block_free(block);
    wined3d_device_gl_allocator_unlock(device_gl);
}

unsafe extern "C" fn wined3d_context_gl_cleanup_resources(context_gl: *mut wined3d_context_gl) {
    let device_gl: *mut wined3d_device_gl = wined3d_device_gl((*context_gl).c.device);
    let mut r: *mut wined3d_retired_block_gl;
    let mut blocks: *mut wined3d_retired_block_gl;
    let mut count: SIZE_T;
    let mut i: SIZE_T = 0;
    let mut id: uint64_t;

    wined3d_context_gl_poll_fences(context_gl);
    id = (*device_gl).completed_fence_id;

    blocks = (*device_gl).retired_blocks as *mut wined3d_retired_block_gl;
    count = (*device_gl).retired_block_count;
    while i < count {
        r = blocks.add(i as usize);

        if (*r).fence_id > id {
            i = i.wrapping_add(1);
            continue;
        }

        wined3d_device_gl_free_memory(device_gl, (*r).block);
        count = count.wrapping_sub(1);
        if i != count {
            *r = *blocks.add(count as usize);
        }
    }
    (*device_gl).retired_block_count = count;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_wait_command_fence(context_gl: *mut wined3d_context_gl, id: uint64_t) {
    let device_gl: *mut wined3d_device_gl = wined3d_device_gl((*context_gl).c.device);
    let mut ret: wined3d_fence_result;
    let mut i: SIZE_T;

    if id <= (*device_gl).completed_fence_id || id > (*device_gl).current_fence_id {
        return;
    }

    i = 0;
    while i < (*context_gl).submitted.fence_count {
        if (*(*context_gl).submitted.fences.offset(i as isize)).id != id {
            i = i.wrapping_add(1);
        } else {
            ret = wined3d_fence_wait(
                (*(*context_gl).submitted.fences.offset(i as isize)).fence,
                core::ptr::addr_of_mut!((*device_gl).d),
            );
            if ret != WINED3D_FENCE_OK {
                ERR!(
                    "Failed to wait for command fence with id 0x%s, ret %#x.\n",
                    wine_dbgstr_longlong(id),
                    ret as u32
                );
            }
            wined3d_context_gl_cleanup_resources(context_gl);
            return;
        }
    }

    ERR!(
        "Failed to find fence for command fence with id 0x%s.\n",
        wine_dbgstr_longlong(id)
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_submit_command_fence(context_gl: *mut wined3d_context_gl) {
    let device_gl: *mut wined3d_device_gl = wined3d_device_gl((*context_gl).c.device);
    let mut f: *mut wined3d_command_fence_gl;
    let mut hr: HRESULT;

    if wined3d_array_reserve(
        core::ptr::addr_of_mut!((*context_gl).submitted.fences) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*context_gl).submitted.fences_size),
        (*context_gl).submitted.fence_count + 1 as SIZE_T,
        core::mem::size_of::<wined3d_command_fence_gl>() as SIZE_T,
    ) == 0
    {
        ERR!("Failed to grow submitted command buffer array.\n");
    }

    let idx = (*context_gl).submitted.fence_count;
    (*context_gl).submitted.fence_count = (*context_gl).submitted.fence_count.wrapping_add(1);
    f = (*context_gl).submitted.fences.add(idx as usize);
    (*f).id = (*device_gl).current_fence_id;
    hr = wined3d_fence_create(
        core::ptr::addr_of_mut!((*device_gl).d),
        core::ptr::addr_of_mut!((*f).fence),
    );
    if hr < 0 as HRESULT {
        ERR!("Failed to create fence, hr %#lx.\n", hr);
    }
    wined3d_fence_issue((*f).fence, core::ptr::addr_of_mut!((*device_gl).d));

    /* We don't expect this to ever happen, but handle it anyway. */
    (*device_gl).current_fence_id = (*device_gl).current_fence_id.wrapping_add(1);
    if (*device_gl).current_fence_id == 0 {
        wined3d_context_gl_wait_command_fence(
            context_gl,
            (*device_gl).current_fence_id.wrapping_sub(1 as uint64_t),
        );
        (*device_gl).completed_fence_id = 0 as uint64_t;
        (*device_gl).current_fence_id = 1 as uint64_t;
    }
    (*device_gl).retired_bo_size = 0 as uint64_t;
    wined3d_context_gl_cleanup_resources(context_gl);
}

unsafe extern "C" fn wined3d_context_gl_destroy_allocator_block(context_gl: *mut wined3d_context_gl, block: *mut wined3d_allocator_block, fence_id: uint64_t) {
    let device_gl: *mut wined3d_device_gl = wined3d_device_gl((*context_gl).c.device);
    let mut r: *mut wined3d_retired_block_gl;

    if (*device_gl).completed_fence_id >= fence_id {
        wined3d_device_gl_free_memory(device_gl, block);

        return;
    }

    if wined3d_array_reserve(
        core::ptr::addr_of_mut!((*device_gl).retired_blocks) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*device_gl).retired_blocks_size),
        (*device_gl).retired_block_count.wrapping_add(1),
        core::mem::size_of::<wined3d_retired_block_gl>() as SIZE_T,
    ) == 0
    {
        ERR!("Leaking block %p.\n", block);
        return;
    }

    let fresh31 = (*device_gl).retired_block_count;
    (*device_gl).retired_block_count = fresh31.wrapping_add(1);
    r = (*device_gl).retired_blocks.add(fresh31 as usize) as *mut wined3d_retired_block_gl;
    (*r).block = block;
    (*r).fence_id = fence_id;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_allocate_vram_chunk_buffer(context_gl: *mut wined3d_context_gl, pool: ::core::ffi::c_uint, size: size_t) -> GLuint {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut flags: GLbitfield;
    let mut id: GLuint = 0;

    TRACE!(
        "context_gl %p, pool %u, size %Iu.\n",
        context_gl,
        pool,
        size
    );

    ((*gl_info).gl_ops.ext.p_glGenBuffers.unwrap_unchecked())(
        1,
        core::ptr::addr_of_mut!(id),
    );
    if id == 0 {
        checkGLcall!(gl_info, "buffer object creation");
        return 0;
    }
    wined3d_context_gl_bind_bo(context_gl, GL_PIXEL_UNPACK_BUFFER, id);

    flags = wined3d_device_gl_get_memory_type_flags(pool) | GL_DYNAMIC_STORAGE_BIT;
    if flags & (GL_MAP_READ_BIT | GL_MAP_WRITE_BIT) != 0 {
        flags |= GL_MAP_PERSISTENT_BIT | GL_MAP_COHERENT_BIT;
    }
    ((*gl_info).gl_ops.ext.p_glBufferStorage.unwrap_unchecked())(
        GL_PIXEL_UNPACK_BUFFER,
        size as GLsizeiptr,
        core::ptr::null(),
        flags,
    );

    checkGLcall!(gl_info, "buffer object creation");

    TRACE!(
        "Created buffer object %u.\n",
        id
    );

    return id;
}

unsafe extern "C" fn wined3d_allocator_chunk_gl_map(
    chunk_gl: *mut wined3d_allocator_chunk_gl,
    context_gl: *mut wined3d_context_gl,
) -> *mut core::ffi::c_void {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut map_ptr: *mut core::ffi::c_void;

    TRACE!(
        "chunk %p, gl_buffer %u, map_ptr %p.\n",
        chunk_gl,
        (*chunk_gl).gl_buffer,
        (*chunk_gl).c.map_ptr
    );

    wined3d_allocator_chunk_gl_lock(chunk_gl);

    if (*chunk_gl).c.map_ptr.is_null() {
        let mut flags: u32 = (wined3d_device_gl_get_memory_type_flags((*chunk_gl).memory_type) as u32)
            & !GL_CLIENT_STORAGE_BIT;

        flags |= GL_MAP_PERSISTENT_BIT | GL_MAP_COHERENT_BIT;
        if (flags & GL_MAP_READ_BIT) == 0 {
            flags |= GL_MAP_UNSYNCHRONIZED_BIT;
        }
        if (flags & GL_MAP_WRITE_BIT) != 0 {
            flags |= GL_MAP_FLUSH_EXPLICIT_BIT;
        }
        wined3d_context_gl_bind_bo(context_gl, GL_PIXEL_UNPACK_BUFFER, (*chunk_gl).gl_buffer);
        (*chunk_gl).c.map_ptr = ((*gl_info).gl_ops.ext.p_glMapBufferRange.unwrap_unchecked())(
            GL_PIXEL_UNPACK_BUFFER,
            0,
            WINED3D_ALLOCATOR_CHUNK_SIZE as GLsizeiptr,
            flags,
        );
        if (*chunk_gl).c.map_ptr.is_null() {
            wined3d_allocator_chunk_gl_unlock(chunk_gl);
            ERR!("Failed to map chunk memory.\n");
            return core::ptr::null_mut();
        }

        adapter_adjust_mapped_memory(
            (*(*context_gl).c.device).adapter,
            (WINED3D_ALLOCATOR_CHUNK_SIZE) as ssize_t,
        );
    }

    (*chunk_gl).c.map_count = (*chunk_gl).c.map_count.wrapping_add(1);
    map_ptr = (*chunk_gl).c.map_ptr;

    wined3d_allocator_chunk_gl_unlock(chunk_gl);

    return map_ptr;
}

unsafe extern "C" fn wined3d_allocator_chunk_gl_unmap(
    mut chunk_gl: *mut wined3d_allocator_chunk_gl,
    mut context_gl: *mut wined3d_context_gl,
) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;

    wined3d_allocator_chunk_gl_lock(chunk_gl);

    (*chunk_gl).c.map_count = (*chunk_gl).c.map_count.wrapping_sub(1);
    if (*chunk_gl).c.map_count == 0 {
        wined3d_context_gl_bind_bo(context_gl, GL_PIXEL_UNPACK_BUFFER, (*chunk_gl).gl_buffer);
        ((*gl_info).gl_ops.ext.p_glUnmapBuffer.unwrap_unchecked())(GL_PIXEL_UNPACK_BUFFER);
        (*chunk_gl).c.map_ptr = core::ptr::null_mut();

        adapter_adjust_mapped_memory((*(*context_gl).c.device).adapter, (-(WINED3D_ALLOCATOR_CHUNK_SIZE as i32)) as ssize_t);
    }

    wined3d_allocator_chunk_gl_unlock(chunk_gl);
}

unsafe extern "C" fn wined3d_bo_gl_map(
    bo: *mut wined3d_bo_gl,
    context_gl: *mut wined3d_context_gl,
    flags: uint32_t,
) -> *mut core::ffi::c_void {
    let device_gl: *mut wined3d_device_gl = wined3d_device_gl((*context_gl).c.device);
    let mut gl_info: *const wined3d_gl_info = core::ptr::null();
    let mut bo_user: *mut wined3d_bo_user = core::ptr::null_mut();
    let mut tmp: wined3d_bo_gl = core::mem::zeroed();

    'map: {
        if (flags & WINED3D_MAP_NOOVERWRITE) != 0 {
            break 'map;
        }

        if ((flags & WINED3D_MAP_DISCARD) != 0)
            && (*bo).command_fence_id > (*device_gl).completed_fence_id
        {
            if wined3d_device_gl_create_bo(
                device_gl,
                context_gl,
                (*bo).size,
                (*bo).binding,
                (*bo).usage,
                (*bo).b.coherent,
                (*bo).flags,
                core::ptr::addr_of_mut!(tmp),
            ) {
                bo_user = ((*bo).b.users.next as *mut u8)
                    .sub(core::mem::offset_of!(wined3d_bo_user, entry))
                    as *mut wined3d_bo_user;
                while core::ptr::addr_of_mut!((*bo_user).entry)
                    != core::ptr::addr_of_mut!((*bo).b.users)
                {
                    (*bo_user).valid = false;
                    bo_user = ((*bo_user).entry.next as *mut u8)
                        .sub(core::mem::offset_of!(wined3d_bo_user, entry))
                        as *mut wined3d_bo_user;
                }
                list_init(core::ptr::addr_of_mut!((*bo).b.users));

                wined3d_context_gl_destroy_bo(context_gl, bo);
                *bo = tmp;
                list_init(core::ptr::addr_of_mut!((*bo).b.users));

                break 'map;
            }

            ERR!("Failed to create new buffer object.\n");
        }

        if (*(*context_gl).c.d3d_info).fences() != 0 {
            if (*bo).command_fence_id == (*device_gl).current_fence_id {
                wined3d_context_gl_submit_command_fence(context_gl);
            }
            wined3d_context_gl_wait_command_fence(context_gl, (*bo).command_fence_id);
        }
    }

    if !(*bo).b.map_ptr.is_null() {
        return (*bo).b.map_ptr;
    }

    if !(*bo).memory.is_null() {
        let chunk_gl: *mut wined3d_allocator_chunk_gl =
            wined3d_allocator_chunk_gl((*(*bo).memory).chunk);

        (*bo).b.map_ptr = wined3d_allocator_chunk_gl_map(chunk_gl, context_gl);
        if (*bo).b.map_ptr.is_null() {
            ERR!("Failed to map chunk.\n");
        }
        return (*bo).b.map_ptr;
    }

    gl_info = (*context_gl).gl_info;
    wined3d_context_gl_bind_bo(context_gl, (*bo).binding, (*bo).id);

    if (*gl_info).supported[ARB_BUFFER_STORAGE as usize] != 0 {
        let mut gl_flags: GLbitfield = 0;

        gl_flags = (*bo).flags & !(GL_CLIENT_STORAGE_BIT as GLbitfield);
        if (gl_flags & (GL_MAP_READ_BIT as GLbitfield)) == 0 {
            gl_flags |= GL_MAP_UNSYNCHRONIZED_BIT as GLbitfield;
        }
        if (gl_flags & (GL_MAP_WRITE_BIT as GLbitfield)) != 0 {
            gl_flags |= GL_MAP_FLUSH_EXPLICIT_BIT as GLbitfield;
        }
        gl_flags |= (GL_MAP_PERSISTENT_BIT as GLbitfield) | (GL_MAP_COHERENT_BIT as GLbitfield);

        (*bo).b.map_ptr = ((*gl_info).gl_ops.ext.p_glMapBufferRange.unwrap_unchecked())(
            (*bo).binding,
            0,
            (*bo).size,
            gl_flags,
        );
    } else if (*gl_info).supported[ARB_MAP_BUFFER_RANGE as usize] != 0 {
        (*bo).b.map_ptr = ((*gl_info).gl_ops.ext.p_glMapBufferRange.unwrap_unchecked())(
            (*bo).binding,
            0,
            (*bo).size,
            wined3d_resource_gl_map_flags(bo, flags as DWORD),
        );
    } else {
        (*bo).b.map_ptr = ((*gl_info).gl_ops.ext.p_glMapBuffer.unwrap_unchecked())(
            (*bo).binding,
            wined3d_resource_gl_legacy_map_flags(flags as DWORD),
        );
    }

    if !(*bo).b.map_ptr.is_null() {
        adapter_adjust_mapped_memory((*device_gl).d.adapter, (*bo).size as ssize_t);
    }

    wined3d_context_gl_bind_bo(context_gl, (*bo).binding, 0);
    checkGLcall!(gl_info, b"Map buffer object\0".as_ptr() as *const core::ffi::c_char);

    return (*bo).b.map_ptr;
}

unsafe extern "C" fn wined3d_bo_gl_unmap(bo: *mut wined3d_bo_gl, context_gl: *mut wined3d_context_gl) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;

    if (*(*context_gl).c.d3d_info).persistent_map() != 0
        && (*(*(*context_gl).c.device).adapter).mapped_size <= MAX_PERSISTENT_MAPPED_BYTES as isize
    {
        return;
    }

    wined3d_device_bo_map_lock((*context_gl).c.device);
    /* The mapping is still in use by the client (viz. for an accelerated
     * NOOVERWRITE map). The client will trigger another unmap request when the
     * d3d application requests to unmap the BO. */
    if (*bo).b.client_map_count != 0 {
        wined3d_device_bo_map_unlock((*context_gl).c.device);
        assert!((*(*context_gl).c.d3d_info).persistent_map() != 0);
        return;
    }
    (*bo).b.map_ptr = core::ptr::null_mut();
    wined3d_device_bo_map_unlock((*context_gl).c.device);

    if !(*bo).memory.is_null() {
        let chunk_gl: *mut wined3d_allocator_chunk_gl =
            wined3d_allocator_chunk_gl((*(*bo).memory).chunk);

        wined3d_allocator_chunk_gl_unmap(chunk_gl, context_gl);
        return;
    }

    wined3d_context_gl_bind_bo(context_gl, (*bo).binding, (*bo).id);
    ((*gl_info).gl_ops.ext.p_glUnmapBuffer.unwrap_unchecked())((*bo).binding);
    wined3d_context_gl_bind_bo(context_gl, (*bo).binding, 0);
    checkGLcall!(gl_info, "Unmap buffer object");

    adapter_adjust_mapped_memory((*(*context_gl).c.device).adapter, -((*bo).size as isize));
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_map_bo_address(context_gl: *mut wined3d_context_gl, data: *const wined3d_bo_address, size: size_t, flags: uint32_t) -> *mut ::core::ffi::c_void {
    let mut bo: *mut wined3d_bo = core::ptr::null_mut::<wined3d_bo>();
    let mut map_ptr: *mut ::core::ffi::c_void = core::ptr::null_mut::<::core::ffi::c_void>();

    bo = (*data).buffer_object;
    if bo.is_null() {
        return (*data).addr as *mut ::core::ffi::c_void;
    }

    map_ptr = wined3d_bo_gl_map(wined3d_bo_gl(bo), context_gl, flags);
    if map_ptr.is_null() {
        ERR!("Failed to map bo.\n");
        return core::ptr::null_mut();
    }

    return (map_ptr as *mut uint8_t)
        .offset((*bo).buffer_offset as isize)
        .offset((*data).addr as uintptr_t as isize) as *mut ::core::ffi::c_void;
}

unsafe extern "C" fn flush_bo_ranges(
    context_gl: *mut wined3d_context_gl,
    data: *const wined3d_const_bo_address,
    range_count: ::core::ffi::c_uint,
    ranges: *const wined3d_range,
) {
    let mut gl_info: *const wined3d_gl_info;
    let mut bo: *mut wined3d_bo_gl;
    let mut i: ::core::ffi::c_uint;

    if (*data).buffer_object.is_null() || (*(*data).buffer_object).coherent != false {
        return;
    }
    bo = wined3d_bo_gl((*data).buffer_object);

    gl_info = (*context_gl).gl_info;
    wined3d_context_gl_bind_bo(context_gl, (*bo).binding, (*bo).id);

    if (*gl_info).supported[ARB_MAP_BUFFER_RANGE as usize] != 0 {
        /* The offset passed to glFlushMappedBufferRange() is relative to the
         * mapped range, but we map the whole buffer anyway. */
        i = 0 as ::core::ffi::c_uint;
        while i < range_count {
            ((*gl_info).gl_ops.ext.p_glFlushMappedBufferRange.unwrap_unchecked())(
                (*bo).binding,
                ((*bo).b.buffer_offset as usize)
                    .wrapping_add((*data).addr as usize)
                    .wrapping_add((*ranges.add(i as usize)).offset as usize)
                    as GLintptr,
                (*ranges.add(i as usize)).size as GLsizeiptr,
            );
            i = i.wrapping_add(1);
        }
    } else if (*gl_info).supported[APPLE_FLUSH_BUFFER_RANGE as usize] != 0 {
        i = 0 as ::core::ffi::c_uint;
        while i < range_count {
            ((*gl_info).gl_ops.ext.p_glFlushMappedBufferRangeAPPLE.unwrap_unchecked())(
                (*bo).binding,
                ((*bo).b.buffer_offset as usize)
                    .wrapping_add((*data).addr as usize)
                    .wrapping_add((*ranges.add(i as usize)).offset as usize)
                    as GLintptr,
                (*ranges.add(i as usize)).size as GLsizeiptr,
            );
            checkGLcall!(gl_info, "glFlushMappedBufferRangeAPPLE");
            i = i.wrapping_add(1);
        }
    }

    wined3d_context_gl_bind_bo(context_gl, (*bo).binding, 0 as GLuint);
    checkGLcall!(gl_info, "Flush buffer object");
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_unmap_bo_address(mut context_gl: *mut wined3d_context_gl, mut data: *const wined3d_bo_address, mut range_count: ::core::ffi::c_uint, mut ranges: *const wined3d_range) {
    let mut bo: *mut wined3d_bo_gl = core::ptr::null_mut();

    if (*data).buffer_object.is_null() {
        return;
    }
    bo = wined3d_bo_gl((*data).buffer_object);

    assert!((!(*bo).b.map_ptr.is_null()));

    flush_bo_ranges(
        context_gl,
        wined3d_const_bo_address(data),
        range_count,
        ranges,
    );
    wined3d_bo_gl_unmap(bo, context_gl);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_flush_bo_address(context_gl: *mut wined3d_context_gl, data: *const wined3d_const_bo_address, size: size_t) {
    let mut range: wined3d_range = wined3d_range { offset: 0, size: 0 };

    TRACE!(
        "context_gl %p, data %s, size %Iu.\n",
        context_gl,
        debug_const_bo_address(data),
        size
    );

    range.offset = (*data).addr as uintptr_t as u32;
    range.size = size as u32;

    flush_bo_ranges(
        context_gl,
        data,
        1,
        core::ptr::addr_of_mut!(range),
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_copy_bo_address(mut context_gl: *mut wined3d_context_gl, mut dst: *const wined3d_bo_address, mut src: *const wined3d_bo_address, mut range_count: ::core::ffi::c_uint, mut ranges: *const wined3d_range, mut map_flags: uint32_t) {
    let mut gl_info: *const wined3d_gl_info = ::core::ptr::null::<wined3d_gl_info>();
    let mut src_bo: *mut wined3d_bo_gl = ::core::ptr::null_mut::<wined3d_bo_gl>();
    let mut dst_bo: *mut wined3d_bo_gl = ::core::ptr::null_mut::<wined3d_bo_gl>();
    let mut dst_ptr: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
    let mut src_ptr: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
    let mut i: ::core::ffi::c_uint = 0;

    gl_info = (*context_gl).gl_info;
    src_bo = if !(*src).buffer_object.is_null() {
        wined3d_bo_gl((*src).buffer_object)
    } else {
        ::core::ptr::null_mut::<wined3d_bo_gl>()
    };
    dst_bo = if !(*dst).buffer_object.is_null() {
        wined3d_bo_gl((*dst).buffer_object)
    } else {
        ::core::ptr::null_mut::<wined3d_bo_gl>()
    };

    if !dst_bo.is_null() && !src_bo.is_null() {
        if (*gl_info).supported[ARB_COPY_BUFFER as usize] != 0 {
            ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(
                GL_COPY_READ_BUFFER as GLenum,
                (*src_bo).id,
            );
            ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(
                GL_COPY_WRITE_BUFFER as GLenum,
                (*dst_bo).id,
            );

            i = 0;
            while i < range_count {
                ((*gl_info).gl_ops.ext.p_glCopyBufferSubData.unwrap_unchecked())(
                    GL_COPY_READ_BUFFER as GLenum,
                    GL_COPY_WRITE_BUFFER as GLenum,
                    (*src_bo)
                        .b
                        .buffer_offset
                        .wrapping_add((*src).addr as GLintptr as size_t)
                        .wrapping_add((*ranges.offset(i as isize)).offset as size_t)
                        as GLintptr,
                    (*dst_bo)
                        .b
                        .buffer_offset
                        .wrapping_add((*dst).addr as GLintptr as size_t)
                        .wrapping_add((*ranges.offset(i as isize)).offset as size_t)
                        as GLintptr,
                    (*ranges.offset(i as isize)).size as GLsizeiptr,
                );
                i = i.wrapping_add(1);
            }
            checkGLcall!(gl_info, "direct buffer copy");

            wined3d_context_gl_reference_bo(context_gl, src_bo);
            wined3d_context_gl_reference_bo(context_gl, dst_bo);
        } else {
            src_ptr = wined3d_context_gl_map_bo_address(
                context_gl,
                src,
                ((*src_bo).size as size_t).wrapping_sub((*src).addr as size_t),
                WINED3D_MAP_READ as uint32_t,
            ) as *mut BYTE;
            dst_ptr = wined3d_context_gl_map_bo_address(
                context_gl,
                dst,
                ((*dst_bo).size as size_t).wrapping_sub((*dst).addr as size_t),
                map_flags,
            ) as *mut BYTE;

            i = 0;
            while i < range_count {
                memcpy(
                    dst_ptr.offset((*ranges.offset(i as isize)).offset as isize)
                        as *mut ::core::ffi::c_void,
                    src_ptr.offset((*ranges.offset(i as isize)).offset as isize)
                        as *const ::core::ffi::c_void,
                    (*ranges.offset(i as isize)).size as size_t,
                );
                i = i.wrapping_add(1);
            }

            wined3d_context_gl_unmap_bo_address(context_gl, dst, range_count, ranges);
            wined3d_context_gl_unmap_bo_address(
                context_gl,
                src,
                0 as ::core::ffi::c_uint,
                ::core::ptr::null::<wined3d_range>(),
            );
        }
    } else if dst_bo.is_null() && !src_bo.is_null() {
        wined3d_context_gl_bind_bo(context_gl, (*src_bo).binding, (*src_bo).id);
        i = 0;
        while i < range_count {
            ((*gl_info).gl_ops.ext.p_glGetBufferSubData.unwrap_unchecked())(
                (*src_bo).binding,
                (*src_bo)
                    .b
                    .buffer_offset
                    .wrapping_add((*src).addr as GLintptr as size_t)
                    .wrapping_add((*ranges.offset(i as isize)).offset as size_t)
                    as GLintptr,
                (*ranges.offset(i as isize)).size as GLsizeiptr,
                (*dst).addr.offset((*ranges.offset(i as isize)).offset as isize)
                    as *mut ::core::ffi::c_void,
            );
            i = i.wrapping_add(1);
        }
        checkGLcall!(gl_info, "buffer download");

        wined3d_context_gl_reference_bo(context_gl, src_bo);
    } else if !dst_bo.is_null() && src_bo.is_null() {
        if (map_flags & WINED3D_MAP_DISCARD as uint32_t) != 0
            && ((*dst_bo).flags & GL_MAP_WRITE_BIT as GLbitfield) != 0
        {
            dst_ptr = wined3d_context_gl_map_bo_address(
                context_gl,
                dst,
                (*dst_bo).size as size_t,
                map_flags,
            ) as *mut BYTE;
            memcpy(
                dst_ptr as *mut ::core::ffi::c_void,
                (*src).addr as *const ::core::ffi::c_void,
                (*dst_bo).size as size_t,
            );
            wined3d_context_gl_unmap_bo_address(context_gl, dst, range_count, ranges);
        } else {
            wined3d_context_gl_bind_bo(context_gl, (*dst_bo).binding, (*dst_bo).id);
            i = 0;
            while i < range_count {
                ((*gl_info).gl_ops.ext.p_glBufferSubData.unwrap_unchecked())(
                    (*dst_bo).binding,
                    (*dst_bo)
                        .b
                        .buffer_offset
                        .wrapping_add((*dst).addr as GLintptr as size_t)
                        .wrapping_add((*ranges.offset(i as isize)).offset as size_t)
                        as GLintptr,
                    (*ranges.offset(i as isize)).size as GLsizeiptr,
                    (*src).addr.offset((*ranges.offset(i as isize)).offset as isize)
                        as *const ::core::ffi::c_void,
                );
                i = i.wrapping_add(1);
            }
            checkGLcall!(gl_info, "buffer upload");

            wined3d_context_gl_reference_bo(context_gl, dst_bo);
        }
    } else {
        i = 0;
        while i < range_count {
            memcpy(
                (*dst).addr.offset((*ranges.offset(i as isize)).offset as isize)
                    as *mut ::core::ffi::c_void,
                (*src).addr.offset((*ranges.offset(i as isize)).offset as isize)
                    as *const ::core::ffi::c_void,
                (*ranges.offset(i as isize)).size as size_t,
            );
            i = i.wrapping_add(1);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_destroy_bo(context_gl: *mut wined3d_context_gl, bo: *mut wined3d_bo_gl) {
    let device_gl: *mut wined3d_device_gl = wined3d_device_gl((*context_gl).c.device);
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;

    assert!(list_empty(core::ptr::addr_of_mut!((*bo).b.users)) != 0);

    if !(*bo).memory.is_null() {
        let order: core::ffi::c_uint = (*(*bo).memory).order;

        if !(*bo).b.map_ptr.is_null() {
            wined3d_allocator_chunk_gl_unmap(
                wined3d_allocator_chunk_gl((*(*bo).memory).chunk),
                context_gl,
            );
        }
        wined3d_context_gl_destroy_allocator_block(context_gl, (*bo).memory, (*bo).command_fence_id);

        if (*bo).command_fence_id == (*device_gl).current_fence_id {
            (*device_gl).retired_bo_size += (WINED3D_ALLOCATOR_CHUNK_SIZE >> order) as u64;
            if (*device_gl).retired_bo_size > (WINED3D_RETIRED_BO_SIZE_THRESHOLD as _) {
                wined3d_context_gl_submit_command_fence(context_gl);
            }
        }

        (*bo).id = 0;
        return;
    }

    if !(*bo).b.map_ptr.is_null() {
        wined3d_context_gl_bind_bo(context_gl, (*bo).binding, (*bo).id);
        ((*gl_info).gl_ops.ext.p_glUnmapBuffer.unwrap_unchecked())((*bo).binding);
        adapter_adjust_mapped_memory((*(*context_gl).c.device).adapter, (-(*bo).size) as ssize_t);
    }

    ((*gl_info).gl_ops.ext.p_glDeleteBuffers.unwrap_unchecked())(
        1,
        core::ptr::addr_of_mut!((*bo).id),
    );
    checkGLcall!(gl_info, b"buffer object destruction\0".as_ptr() as *const core::ffi::c_char);
    (*bo).id = 0;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_get_offscreen_gl_buffer(context_gl: *const wined3d_context_gl) -> GLenum {
    GL_COLOR_ATTACHMENT0
}

unsafe extern "C" fn wined3d_context_gl_generate_rt_mask_no_fbo(
    context_gl: *const wined3d_context_gl,
    rt: *mut wined3d_resource,
) -> uint32_t {
    if rt.is_null()
        || (*(*rt).format).id as ::core::ffi::c_uint
            == WINED3DFMT_NULL as ::core::ffi::c_uint
    {
        return 0 as uint32_t;
    } else if (*rt).type_ as ::core::ffi::c_uint
        != WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint
        && !(*texture_from_resource(rt)).swapchain.is_null()
    {
        return context_generate_rt_mask_from_resource(rt) as uint32_t;
    } else {
        return context_generate_rt_mask(wined3d_context_gl_get_offscreen_gl_buffer(context_gl)) as uint32_t;
    };
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_apply_blit_state(context_gl: *mut wined3d_context_gl, device: *const wined3d_device) {
    let context: *mut wined3d_context = core::ptr::addr_of_mut!((*context_gl).c);
    let mut gl_info: *const wined3d_gl_info = core::ptr::null();
    let mut rt_size: SIZE = SIZE { cx: 0, cy: 0 };

    gl_info = (*context_gl).gl_info;

    wined3d_context_gl_get_rt_size(context_gl, core::ptr::addr_of_mut!(rt_size));

    if (*context).last_was_blit() != 0 {
        if (*context_gl).blit_size.cx != rt_size.cx || (*context_gl).blit_size.cy != rt_size.cy {
            (*gl_info).gl_ops.gl.p_glViewport.unwrap_unchecked()(
                0,
                0,
                rt_size.cx,
                rt_size.cy,
            );
            (*context).viewport_count = WINED3D_MAX_VIEWPORTS;
            (*context_gl).blit_size = rt_size;
            /* No need to dirtify here, the states are still dirtified because
             * they weren't applied since the last context_apply_blit_state()
             * call. */
        }
        checkGLcall!(gl_info, b"blit state application\0".as_ptr() as *const i8);
        return;
    }
    (*context).set_last_was_blit((TRUE) as _);

    if (*gl_info).supported[ARB_SAMPLER_OBJECTS as usize] != 0 {
        ((*gl_info).gl_ops.ext.p_glBindSampler.unwrap_unchecked())(0, 0);
    }
    wined3d_context_gl_active_texture(context_gl, gl_info, 0);

    context_invalidate_compute_state(context, (STATE_COMPUTE_SHADER_RESOURCE_BINDING) as DWORD);
    context_invalidate_state(context, (STATE_GRAPHICS_SHADER_RESOURCE_BINDING as u32));

    if (*gl_info).supported[WINED3D_GL_LEGACY_CONTEXT as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glDisable.unwrap_unchecked()(
            GL_ALPHA_TEST,
        );
        context_invalidate_state(context, STATE_RENDER!(WINED3D_RS_ALPHATESTENABLE));
    }
    (*gl_info).gl_ops.gl.p_glDisable.unwrap_unchecked()(GL_BLEND);
    (*gl_info).gl_ops.gl.p_glColorMask.unwrap_unchecked()(
        (GL_TRUE as u8),
        (GL_TRUE as u8),
        (GL_TRUE as u8),
        (GL_TRUE as u8),
    );
    context_invalidate_state(context, (STATE_BLEND as u32));
    (*gl_info).gl_ops.gl.p_glDisable.unwrap_unchecked()(GL_CULL_FACE);
    (*gl_info).gl_ops.gl.p_glDisable.unwrap_unchecked()(GL_SCISSOR_TEST);
    context_invalidate_state(context, (STATE_RASTERIZER as u32));
    (*gl_info).gl_ops.gl.p_glDisable.unwrap_unchecked()(GL_DEPTH_TEST);
    (*gl_info).gl_ops.gl.p_glDisable.unwrap_unchecked()(GL_STENCIL_TEST);
    context_invalidate_state(context, (STATE_DEPTH_STENCIL as u32));
    if (*gl_info).supported[ARB_FRAMEBUFFER_SRGB as usize] != 0 {
        (*gl_info).gl_ops.gl.p_glDisable.unwrap_unchecked()(
            GL_FRAMEBUFFER_SRGB,
        );
        context_invalidate_state(context, STATE_RENDER!(WINED3D_RS_SRGBWRITEENABLE));
    }

    (*context).set_last_was_rhw((TRUE) as _);
    context_invalidate_state(context, (STATE_VDECL as u32)); /* because of last_was_rhw = TRUE */

    wined3d_context_gl_enable_clip_distances(context_gl, 0);
    context_invalidate_state(context, STATE_RENDER!(WINED3D_RS_CLIPPING));

    /* FIXME: Make draw_textured_quad() able to work with a upper left origin. */
    if (*gl_info).supported[ARB_CLIP_CONTROL as usize] != 0 {
        ((*gl_info).gl_ops.ext.p_glClipControl.unwrap_unchecked())(GL_LOWER_LEFT, GL_NEGATIVE_ONE_TO_ONE);
    }
    (*gl_info).gl_ops.gl.p_glViewport.unwrap_unchecked()(
        0,
        0,
        rt_size.cx,
        rt_size.cy,
    );
    (*context).viewport_count = WINED3D_MAX_VIEWPORTS;
    context_invalidate_state(context, (STATE_VIEWPORT as u32));

    (*(*device).shader_backend).shader_disable.unwrap_unchecked()(
        (*device).shader_priv,
        context,
    );

    (*context_gl).blit_size = rt_size;

    checkGLcall!(gl_info, b"blit state application\0".as_ptr() as *const i8);
}

unsafe extern "C" fn have_framebuffer_attachment(
    mut rt_count: ::core::ffi::c_uint,
    mut rts: *const *mut wined3d_rendertarget_view,
    mut ds: *const wined3d_rendertarget_view,
) -> BOOL {
    let mut i: ::core::ffi::c_uint = 0;
    if !ds.is_null() {
        return (TRUE) as BOOL;
    }
    i = 0 as ::core::ffi::c_uint;
    while i < rt_count {
        if !(*rts.offset(i as isize)).is_null()
            && (*(**rts.offset(i as isize)).format).id as ::core::ffi::c_uint
                != WINED3DFMT_NULL as ::core::ffi::c_uint
        {
            return (TRUE) as BOOL;
        }
        i = i.wrapping_add(1);
    }
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_apply_clear_state(context_gl: *mut wined3d_context_gl, state: *const wined3d_state, rt_count: ::core::ffi::c_uint, fb: *const wined3d_fb_state) -> BOOL {
    let mut rts: *const *mut wined3d_rendertarget_view =
        core::ptr::addr_of!((*fb).render_targets) as *const *mut wined3d_rendertarget_view;
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut dsv: *mut wined3d_rendertarget_view = (*fb).depth_stencil;
    let mut rt_mask: uint32_t = 0;
    let mut cur_mask: *mut uint32_t = core::ptr::null_mut();
    let mut i: ::core::ffi::c_uint = 0;

    if isStateDirty(core::ptr::addr_of_mut!((*context_gl).c), (STATE_FRAMEBUFFER as u32)) != false
        || fb != core::ptr::addr_of!((*state).fb)
        || rt_count != (*gl_info).limits.buffers
    {
        let mut ds_info: wined3d_rendertarget_info = core::mem::zeroed();
        let _ = 0;

        if have_framebuffer_attachment(rt_count, rts, dsv) == 0 {
            WARN!("Invalid render target config, need at least one attachment.\n");
            return (FALSE) as BOOL;
        }

        if rt_count == 0
            || wined3d_resource_is_offscreen((**rts.offset(0 as isize)).resource) != 0
        {
            memset(
                core::ptr::addr_of_mut!((*context_gl).blit_targets) as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                core::mem::size_of::<[wined3d_rendertarget_info; 8]>() as size_t,
            );
            i = 0 as ::core::ffi::c_uint;
            while i < rt_count {
                if !(*rts.offset(i as isize)).is_null() {
                    let rtv_gl: *mut wined3d_rendertarget_view_gl =
                        wined3d_rendertarget_view_gl(*rts.offset(i as isize));
                    (*context_gl).blit_targets[i as usize].gl_view = (*rtv_gl).gl_view;
                    (*context_gl).blit_targets[i as usize].resource = (*rtv_gl).v.resource;
                    (*context_gl).blit_targets[i as usize].sub_resource_idx =
                        (*rtv_gl).v.sub_resource_idx;
                    (*context_gl).blit_targets[i as usize].layer_count = (*rtv_gl).v.layer_count;
                }
                if !(*rts.offset(i as isize)).is_null()
                    && (*(**rts.offset(i as isize)).format).id != WINED3DFMT_NULL
                {
                    rt_mask |= (1 as uint32_t) << i;
                }
                i = i.wrapping_add(1);
            }

            if !dsv.is_null() {
                let dsv_gl: *mut wined3d_rendertarget_view_gl = wined3d_rendertarget_view_gl(dsv);
                ds_info.gl_view = (*dsv_gl).gl_view;
                ds_info.resource = (*dsv_gl).v.resource;
                ds_info.sub_resource_idx = (*dsv_gl).v.sub_resource_idx;
                ds_info.layer_count = (*dsv_gl).v.layer_count;
            }

            wined3d_context_gl_apply_fbo_state(
                context_gl,
                GL_FRAMEBUFFER as GLenum,
                core::ptr::addr_of_mut!((*context_gl).blit_targets) as *const wined3d_rendertarget_info,
                core::ptr::addr_of_mut!(ds_info),
                (if rt_count != 0 {
                    (*(**rts.offset(0 as isize)).resource).draw_binding
                } else {
                    0
                }) as DWORD,
                (if !dsv.is_null() {
                    (*(*dsv).resource).draw_binding
                } else {
                    0
                }) as DWORD,
            );
        } else {
            wined3d_context_gl_apply_fbo_state(
                context_gl,
                GL_FRAMEBUFFER as GLenum,
                core::ptr::null(),
                core::ptr::addr_of_mut!(ds_info),
                WINED3D_LOCATION_DRAWABLE as DWORD,
                WINED3D_LOCATION_DRAWABLE as DWORD,
            );
            rt_mask = context_generate_rt_mask_from_resource((**rts.offset(0 as isize)).resource)
                as uint32_t;
        }

        /* If the framebuffer is not the device's fb the device's fb has to be reapplied
         * next draw. Otherwise we could mark the framebuffer state clean here, once the
         * state management allows this */
        context_invalidate_state(
            core::ptr::addr_of_mut!((*context_gl).c),
            (STATE_FRAMEBUFFER as u32),
        );
    } else if rt_count == 0
        || wined3d_resource_is_offscreen((**rts.offset(0 as isize)).resource) != 0
    {
        i = 0 as ::core::ffi::c_uint;
        while i < rt_count {
            if !(*rts.offset(i as isize)).is_null()
                && (*(**rts.offset(i as isize)).format).id != WINED3DFMT_NULL
            {
                rt_mask |= (1 as uint32_t) << i;
            }
            i = i.wrapping_add(1);
        }
    } else {
        rt_mask = wined3d_context_gl_generate_rt_mask_no_fbo(
            context_gl,
            if rt_count != 0 {
                (**rts.offset(0 as isize)).resource
            } else {
                core::ptr::null_mut()
            },
        );
    }

    cur_mask = if !(*context_gl).current_fbo.is_null() {
        core::ptr::addr_of_mut!((*(*context_gl).current_fbo).rt_mask)
    } else {
        core::ptr::addr_of_mut!((*context_gl).draw_buffers_mask)
    };

    if rt_mask != *cur_mask {
        wined3d_context_gl_apply_draw_buffers(context_gl, rt_mask);
        *cur_mask = rt_mask;
        context_invalidate_state(
            core::ptr::addr_of_mut!((*context_gl).c),
            (STATE_FRAMEBUFFER as u32),
        );
    }

    wined3d_context_gl_check_fbo_status(context_gl, GL_FRAMEBUFFER as GLenum);

    (*context_gl).c.set_last_was_blit((FALSE) as _);
    (*context_gl).c.set_last_was_ffp_blit((FALSE) as _);

    /* Blending and clearing should be orthogonal, but tests on the nvidia
     * driver show that disabling blending when clearing improves the clearing
     * performance incredibly. */
    (*gl_info)
        .gl_ops
        .gl
        .p_glDisable
        .unwrap_unchecked()(GL_BLEND as GLenum);
    (*gl_info)
        .gl_ops
        .gl
        .p_glEnable
        .unwrap_unchecked()(GL_SCISSOR_TEST as GLenum);
    if rt_count != 0 && (*gl_info).supported[ARB_FRAMEBUFFER_SRGB as usize] != 0 {
        if needs_srgb_write((*context_gl).c.d3d_info, state, fb) != 0 {
            (*gl_info)
                .gl_ops
                .gl
                .p_glEnable
                .unwrap_unchecked()(GL_FRAMEBUFFER_SRGB as GLenum);
        } else {
            (*gl_info)
                .gl_ops
                .gl
                .p_glDisable
                .unwrap_unchecked()(GL_FRAMEBUFFER_SRGB as GLenum);
        }
        context_invalidate_state(
            core::ptr::addr_of_mut!((*context_gl).c),
            STATE_RENDER!(WINED3D_RS_SRGBWRITEENABLE),
        );
    }
    checkGLcall!(gl_info, b"setting up state for clear\0".as_ptr() as *const ::core::ffi::c_char);

    context_invalidate_state(
        core::ptr::addr_of_mut!((*context_gl).c),
        (STATE_BLEND as u32),
    );
    context_invalidate_state(
        core::ptr::addr_of_mut!((*context_gl).c),
        (STATE_RASTERIZER as u32),
    );
    context_invalidate_state(
        core::ptr::addr_of_mut!((*context_gl).c),
        (STATE_SCISSORRECT as u32),
    );

    return (TRUE) as BOOL;
}

unsafe extern "C" fn find_draw_buffers_mask(
    context_gl: *const wined3d_context_gl,
    state: *const wined3d_state,
) -> uint32_t {
    let rts: *const *mut wined3d_rendertarget_view =
        core::ptr::addr_of!((*state).fb.render_targets) as *const *mut wined3d_rendertarget_view;
    let ps: *mut wined3d_shader = (*state).shader[WINED3D_SHADER_TYPE_PIXEL as usize];
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut rt_mask: ::core::ffi::c_uint = 0;
    let mut mask: ::core::ffi::c_uint = 0;
    let mut i: ::core::ffi::c_uint = 0;

    rt_mask = (if !ps.is_null() {
        (*ps).reg_maps.rt_mask
    } else {
        1 as DWORD
    }) as ::core::ffi::c_uint;
    rt_mask &= wined3d_mask_from_size((*gl_info).limits.buffers) as ::core::ffi::c_uint;
    if !(*state).blend_state.is_null() && (*(*state).blend_state).dual_source != 0 {
        rt_mask = 1 as ::core::ffi::c_uint;
    }

    mask = rt_mask;
    while mask != 0 {
        i = wined3d_bit_scan(core::ptr::addr_of_mut!(mask)) as ::core::ffi::c_uint;
        if (*rts.offset(i as isize)).is_null()
            || (*(**rts.offset(i as isize)).format).id as ::core::ffi::c_uint
                == WINED3DFMT_NULL as ::core::ffi::c_uint
        {
            rt_mask &= !((1 as ::core::ffi::c_uint) << i);
        }
    }

    return rt_mask as uint32_t;
}
#[no_mangle]
pub unsafe extern "C" fn context_gl_apply_texture_draw_state(context_gl: *mut wined3d_context_gl, texture: *mut wined3d_texture, sub_resource_idx: ::core::ffi::c_uint, location: ::core::ffi::c_uint) {
    let format: *const wined3d_format = (*texture).resource.format;
    let context: *mut wined3d_context = core::ptr::addr_of_mut!((*context_gl).c);
    let mut rt_mask: u32 = 0;
    let mut cur_mask: *mut u32 = core::ptr::null_mut::<u32>();

    WINE_TRACE!("context_gl %p, texture %p, sub_resource_idx %u, location %s.\n", context_gl, texture, sub_resource_idx, wined3d_debug_location(location as u32));

    if (*format).depth_size as core::ffi::c_int != 0 || (*format).stencil_size as core::ffi::c_int != 0 {
        wined3d_context_gl_apply_fbo_state_explicit(
            context_gl,
            GL_DRAW_FRAMEBUFFER,
            core::ptr::null_mut::<wined3d_resource>(),
            0,
            core::ptr::addr_of_mut!((*texture).resource),
            sub_resource_idx,
            location as u32,
        );
    } else {
        wined3d_context_gl_apply_fbo_state_explicit(
            context_gl,
            GL_DRAW_FRAMEBUFFER,
            core::ptr::addr_of_mut!((*texture).resource),
            sub_resource_idx,
            core::ptr::null_mut::<wined3d_resource>(),
            0,
            location as u32,
        );
    }

    if location != WINED3D_LOCATION_DRAWABLE {
        if (*(*texture).resource.format).id as core::ffi::c_uint == WINED3DFMT_NULL
            || (*format).depth_size as core::ffi::c_int != 0
            || (*format).stencil_size as core::ffi::c_int != 0
        {
            rt_mask = 0;
        } else {
            rt_mask = 1;
        }
    } else {
        rt_mask = context_generate_rt_mask_from_resource(core::ptr::addr_of_mut!((*texture).resource)) as u32;
    }

    cur_mask = if !(*context_gl).current_fbo.is_null() {
        core::ptr::addr_of_mut!((*(*context_gl).current_fbo).rt_mask)
    } else {
        core::ptr::addr_of_mut!((*context_gl).draw_buffers_mask)
    };
    if rt_mask != *cur_mask {
        wined3d_context_gl_apply_draw_buffers(context_gl, rt_mask);
        *cur_mask = rt_mask;
    }

    wined3d_context_gl_check_fbo_status(context_gl, GL_DRAW_FRAMEBUFFER);

    context_invalidate_state(context, (STATE_FRAMEBUFFER as u32));
}
#[no_mangle]
pub unsafe extern "C" fn context_state_fb(mut context: *mut wined3d_context, mut state: *const wined3d_state, mut state_id: DWORD) {
    let mut context_gl: *mut wined3d_context_gl = wined3d_context_gl(context);
    let mut rt_mask: uint32_t = find_draw_buffers_mask(context_gl, state);
    let mut view_gl: *const wined3d_rendertarget_view_gl =
        ::core::ptr::null::<wined3d_rendertarget_view_gl>();
    let mut ds_info: wined3d_rendertarget_info = wined3d_rendertarget_info {
        gl_view: wined3d_gl_view {
            target: 0 as GLenum,
            name: 0,
        },
        resource: ::core::ptr::null_mut::<wined3d_resource>(),
        sub_resource_idx: 0,
        layer_count: 0,
    };
    let mut fb: *const wined3d_fb_state = &raw const (*state).fb;
    let mut color_location: DWORD = 0 as DWORD;
    let mut cur_mask: *mut uint32_t = ::core::ptr::null_mut::<uint32_t>();

    memset(
        &raw mut (*context_gl).blit_targets as *mut wined3d_rendertarget_info
            as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[wined3d_rendertarget_info; 8]>() as size_t,
    );
    let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    while i < (*(*context_gl).gl_info).limits.buffers {
        if !(*fb).render_targets[i as usize].is_null() {
            view_gl = wined3d_rendertarget_view_gl((*fb).render_targets[i as usize]);
            (*context_gl).blit_targets[i as usize].gl_view = (*view_gl).gl_view;
            (*context_gl).blit_targets[i as usize].resource = (*view_gl).v.resource;
            (*context_gl).blit_targets[i as usize].sub_resource_idx = (*view_gl).v.sub_resource_idx;
            (*context_gl).blit_targets[i as usize].layer_count = (*view_gl).v.layer_count;
            if color_location == 0 {
                color_location = (*(*view_gl).v.resource).draw_binding as DWORD;
            }
        }
        i = i.wrapping_add(1);
    }
    if !(*fb).depth_stencil.is_null() {
        view_gl = wined3d_rendertarget_view_gl((*fb).depth_stencil);
        ds_info.gl_view = (*view_gl).gl_view;
        ds_info.resource = (*view_gl).v.resource;
        ds_info.sub_resource_idx = (*view_gl).v.sub_resource_idx;
        ds_info.layer_count = (*view_gl).v.layer_count;
    }
    wined3d_context_gl_apply_fbo_state(
        context_gl,
        GL_FRAMEBUFFER as GLenum,
        &raw mut (*context_gl).blit_targets as *mut wined3d_rendertarget_info,
        &raw mut ds_info,
        color_location,
        (if !(*fb).depth_stencil.is_null() {
            (*(*(*fb).depth_stencil).resource).draw_binding as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as DWORD,
    );
    cur_mask = if !(*context_gl).current_fbo.is_null() {
        &raw mut (*(*context_gl).current_fbo).rt_mask
    } else {
        &raw mut (*context_gl).draw_buffers_mask
    };
    if rt_mask != *cur_mask {
        wined3d_context_gl_apply_draw_buffers(context_gl, rt_mask);
        *cur_mask = rt_mask;
    }
}

unsafe extern "C" fn wined3d_context_gl_map_stage(
    mut context_gl: *mut wined3d_context_gl,
    mut stage: ::core::ffi::c_uint,
    mut unit: ::core::ffi::c_uint,
) {
    let mut i: ::core::ffi::c_uint = (*context_gl).rev_tex_unit_map[unit as usize];
    let mut j: ::core::ffi::c_uint = (*context_gl).tex_unit_map[stage as usize];
    WINE_TRACE!("Mapping stage %u to unit %u.\n", stage, unit);
    (*context_gl).tex_unit_map[stage as usize] = unit;
    if i != (WINED3D_UNMAPPED_STAGE) as u32 && i != stage {
        (*context_gl).tex_unit_map[i as usize] = (WINED3D_UNMAPPED_STAGE) as u32;
    }
    (*context_gl).rev_tex_unit_map[unit as usize] = stage;
    if j != (WINED3D_UNMAPPED_STAGE) as u32 && j != unit {
        (*context_gl).rev_tex_unit_map[j as usize] = (WINED3D_UNMAPPED_STAGE) as u32;
    }
}

unsafe extern "C" fn context_invalidate_texture_stage(context: *mut wined3d_context, stage: DWORD) {
    let mut i: DWORD;
    i = 0;
    while i <= (WINED3D_HIGHEST_TEXTURE_STATE as DWORD) {
        context_invalidate_state(context, (STATE_TEXTURESTAGE!(stage, (i) as i32)) as u32);
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn use_ffp_ps(state: *const wined3d_state) -> bool {
    let vs: *mut wined3d_shader = (*state).shader[WINED3D_SHADER_TYPE_VERTEX as usize];

    if use_vs(state) == 0 {
        return true;
    }
    if !vs.is_null() && (*vs).reg_maps.shader_version.major as i32 >= 4 {
        return false;
    }
    return true;
}

unsafe extern "C" fn context_update_fixed_function_usage_map(
    mut context: *mut wined3d_context,
    mut state: *const wined3d_state,
) {
    let mut i: UINT = 0;
    let mut start: UINT;
    let mut end: UINT;

    (*context).set_fixed_function_usage_map((0) as _);

    if use_ffp_ps(state) {
        i = 0;
        while i < WINED3D_MAX_FFP_TEXTURES {
            let color_op: wined3d_texture_op =
                (*state).texture_states[i as usize][WINED3D_TSS_COLOR_OP as usize] as wined3d_texture_op;
            let alpha_op: wined3d_texture_op =
                (*state).texture_states[i as usize][WINED3D_TSS_ALPHA_OP as usize] as wined3d_texture_op;
            let color_arg1: DWORD =
                (*state).texture_states[i as usize][WINED3D_TSS_COLOR_ARG1 as usize] & WINED3DTA_SELECTMASK;
            let color_arg2: DWORD =
                (*state).texture_states[i as usize][WINED3D_TSS_COLOR_ARG2 as usize] & WINED3DTA_SELECTMASK;
            let color_arg3: DWORD =
                (*state).texture_states[i as usize][WINED3D_TSS_COLOR_ARG0 as usize] & WINED3DTA_SELECTMASK;
            let alpha_arg1: DWORD =
                (*state).texture_states[i as usize][WINED3D_TSS_ALPHA_ARG1 as usize] & WINED3DTA_SELECTMASK;
            let alpha_arg2: DWORD =
                (*state).texture_states[i as usize][WINED3D_TSS_ALPHA_ARG2 as usize] & WINED3DTA_SELECTMASK;
            let alpha_arg3: DWORD =
                (*state).texture_states[i as usize][WINED3D_TSS_ALPHA_ARG0 as usize] & WINED3DTA_SELECTMASK;

            /* Not used, and disable higher stages. */
            if color_op as ::core::ffi::c_uint == WINED3D_TOP_DISABLE as ::core::ffi::c_uint {
                break;
            }

            if ((color_arg1 == WINED3DTA_TEXTURE as DWORD)
                && color_op as ::core::ffi::c_uint != WINED3D_TOP_SELECT_ARG2 as ::core::ffi::c_uint)
                || ((color_arg2 == WINED3DTA_TEXTURE as DWORD)
                    && color_op as ::core::ffi::c_uint != WINED3D_TOP_SELECT_ARG1 as ::core::ffi::c_uint)
                || ((color_arg3 == WINED3DTA_TEXTURE as DWORD)
                    && (color_op as ::core::ffi::c_uint == WINED3D_TOP_MULTIPLY_ADD as ::core::ffi::c_uint
                        || color_op as ::core::ffi::c_uint == WINED3D_TOP_LERP as ::core::ffi::c_uint))
                || ((alpha_arg1 == WINED3DTA_TEXTURE as DWORD)
                    && alpha_op as ::core::ffi::c_uint != WINED3D_TOP_SELECT_ARG2 as ::core::ffi::c_uint)
                || ((alpha_arg2 == WINED3DTA_TEXTURE as DWORD)
                    && alpha_op as ::core::ffi::c_uint != WINED3D_TOP_SELECT_ARG1 as ::core::ffi::c_uint)
                || ((alpha_arg3 == WINED3DTA_TEXTURE as DWORD)
                    && (alpha_op as ::core::ffi::c_uint == WINED3D_TOP_MULTIPLY_ADD as ::core::ffi::c_uint
                        || alpha_op as ::core::ffi::c_uint == WINED3D_TOP_LERP as ::core::ffi::c_uint))
            {
                (*context).set_fixed_function_usage_map(((*context).fixed_function_usage_map() | ((1 << i) as DWORD)) as _);
            }

            if (color_op as ::core::ffi::c_uint == WINED3D_TOP_BUMPENVMAP as ::core::ffi::c_uint
                || color_op as ::core::ffi::c_uint == WINED3D_TOP_BUMPENVMAP_LUMINANCE as ::core::ffi::c_uint)
                && i < WINED3D_MAX_FFP_TEXTURES - 1
            {
                (*context).set_fixed_function_usage_map(((*context).fixed_function_usage_map() | ((1 << (i + 1)) as DWORD)) as _);
            }

            i = i.wrapping_add(1);
        }
    }

    if i < (*context).lowest_disabled_stage() {
        start = i;
        end = (*context).lowest_disabled_stage();
    } else {
        start = (*context).lowest_disabled_stage();
        end = i;
    }

    (*context).set_lowest_disabled_stage((i) as _);
    i = start + 1;
    while i < end {
        context_invalidate_state(context, (STATE_TEXTURESTAGE!(i, (WINED3D_TSS_COLOR_OP) as i32)) as u32);
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn wined3d_context_gl_map_fixed_function_samplers(
    mut context_gl: *mut wined3d_context_gl,
    mut state: *const wined3d_state,
) {
    let mut d3d_info: *const wined3d_d3d_info = (*context_gl).c.d3d_info;
    let mut i: ::core::ffi::c_uint = 0;
    let mut tex: ::core::ffi::c_uint = 0;
    let mut ffu_map: uint32_t = 0;
    ffu_map = (*context_gl).c.fixed_function_usage_map() as uint32_t;
    if (*d3d_info).ffp_fragment_caps.max_textures == (*d3d_info).ffp_fragment_caps.max_blend_stages
        || (*context_gl).c.lowest_disabled_stage() <= (*d3d_info).ffp_fragment_caps.max_textures
    {
        while ffu_map != 0 {
            i = wined3d_bit_scan(&raw mut ffu_map) as ::core::ffi::c_uint;
            if (*context_gl).tex_unit_map[i as usize] != i {
                wined3d_context_gl_map_stage(context_gl, i, i);
                context_invalidate_state(&raw mut (*context_gl).c, (STATE_GRAPHICS_SHADER_RESOURCE_BINDING as u32));
                context_invalidate_texture_stage(&raw mut (*context_gl).c, i as DWORD);
            }
        }
        return;
    }
    tex = 0;
    while ffu_map != 0 {
        i = wined3d_bit_scan(&raw mut ffu_map) as ::core::ffi::c_uint;
        if (*context_gl).tex_unit_map[i as usize] != tex {
            wined3d_context_gl_map_stage(context_gl, i, tex);
            context_invalidate_state(&raw mut (*context_gl).c, (STATE_GRAPHICS_SHADER_RESOURCE_BINDING as u32));
            context_invalidate_texture_stage(&raw mut (*context_gl).c, i as DWORD);
        }
        tex = tex.wrapping_add(1);
    }
}

unsafe extern "C" fn wined3d_context_gl_map_psamplers(
    context_gl: *mut wined3d_context_gl,
    state: *const wined3d_state,
) {
    let d3d_info: *const wined3d_d3d_info = (*context_gl).c.d3d_info;
    let resource_info: *const wined3d_shader_resource_info =
        &raw mut (*(*state).shader[WINED3D_SHADER_TYPE_PIXEL as usize]).reg_maps.resource_info
            as *mut wined3d_shader_resource_info;
    let mut i: ::core::ffi::c_uint = 0;

    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_MAX_FRAGMENT_SAMPLERS as ::core::ffi::c_uint {
        if (*resource_info.offset(i as isize)).type_ as ::core::ffi::c_uint != 0
            && (*context_gl).tex_unit_map[i as usize] != i
        {
            wined3d_context_gl_map_stage(context_gl, i, i);
            context_invalidate_state(&raw mut (*context_gl).c, STATE_GRAPHICS_SHADER_RESOURCE_BINDING as ::core::ffi::c_uint);
            if i < (*d3d_info).ffp_fragment_caps.max_blend_stages {
                context_invalidate_texture_stage(&raw mut (*context_gl).c, i as DWORD);
            }
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn wined3d_context_gl_unit_free_for_vs(
    context_gl: *const wined3d_context_gl,
    ps_resource_info: *const wined3d_shader_resource_info,
    unit: ::core::ffi::c_uint,
) -> BOOL {
    let mut current_mapping: ::core::ffi::c_uint = (*context_gl).rev_tex_unit_map[unit as usize];

    /* Not currently used */
    if current_mapping == (WINED3D_UNMAPPED_STAGE) as u32 {
        return (TRUE) as BOOL;
    }

    if current_mapping < WINED3D_MAX_FRAGMENT_SAMPLERS as ::core::ffi::c_uint {
        /* Used by a fragment sampler */

        if ps_resource_info.is_null() {
            /* No pixel shader, check fixed function */
            return (current_mapping >= WINED3D_MAX_FFP_TEXTURES as ::core::ffi::c_uint
                || (*context_gl).c.fixed_function_usage_map()
                    & (1 as ::core::ffi::c_uint) << current_mapping
                    == 0) as ::core::ffi::c_int;
        }

        /* Pixel shader, check the shader's sampler map */
        return ((*ps_resource_info.offset(current_mapping as isize)).type_ as u64 == 0)
            as ::core::ffi::c_int;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn wined3d_context_gl_map_vsamplers(
    context_gl: *mut wined3d_context_gl,
    ps: BOOL,
    state: *const wined3d_state,
) {
    let vs_resource_info: *const wined3d_shader_resource_info =
        &raw mut (**(&raw const (*state).shader as *const *mut wined3d_shader)
            .offset(WINED3D_SHADER_TYPE_VERTEX as isize))
        .reg_maps
        .resource_info as *mut wined3d_shader_resource_info;
    let mut ps_resource_info: *const wined3d_shader_resource_info =
        core::ptr::null::<wined3d_shader_resource_info>();
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut start: core::ffi::c_int =
        min!(WINED3D_MAX_COMBINED_SAMPLERS, (*gl_info).limits.graphics_samplers) as core::ffi::c_int
            - 1 as core::ffi::c_int;
    let mut i: core::ffi::c_int;

    /* Note that we only care if a resource is used or not, not the
     * resource's specific type. Otherwise we'd need to call
     * shader_update_samplers() here for 1.x pixelshaders. */
    if ps != 0 {
        ps_resource_info = &raw mut (**(&raw const (*state).shader as *const *mut wined3d_shader)
            .offset(WINED3D_SHADER_TYPE_PIXEL as isize))
        .reg_maps
        .resource_info as *mut wined3d_shader_resource_info;
    }

    i = 0 as core::ffi::c_int;
    while i < WINED3D_MAX_VERTEX_SAMPLERS as core::ffi::c_int {
        let vsampler_idx: DWORD = (i + WINED3D_MAX_FRAGMENT_SAMPLERS as core::ffi::c_int) as DWORD;
        if (*vs_resource_info.offset(i as isize)).type_ as u64 != 0 {
            while start >= 0 as core::ffi::c_int {
                if wined3d_context_gl_unit_free_for_vs(
                    context_gl,
                    ps_resource_info,
                    start as core::ffi::c_uint,
                ) != 0
                {
                    if (*context_gl).tex_unit_map[vsampler_idx as usize]
                        != start as core::ffi::c_uint
                    {
                        wined3d_context_gl_map_stage(
                            context_gl,
                            vsampler_idx as core::ffi::c_uint,
                            start as core::ffi::c_uint,
                        );
                        context_invalidate_state(
                            &raw mut (*context_gl).c,
                            STATE_GRAPHICS_SHADER_RESOURCE_BINDING as core::ffi::c_uint,
                        );
                    }

                    start -= 1;
                    break;
                }

                start -= 1;
            }
            if (*context_gl).tex_unit_map[vsampler_idx as usize]
                == (WINED3D_UNMAPPED_STAGE) as u32
            {
                WARN!("Couldn't find a free texture unit for vertex sampler %u.\n", i);
            }
        }
        i += 1;
    }
}

unsafe extern "C" fn wined3d_context_gl_update_tex_unit_map(
    context_gl: *mut wined3d_context_gl,
    state: *const wined3d_state,
) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let vs: BOOL = use_vs(state);
    let ps: BOOL = use_ps(state);

    if ps == 0 {
        context_update_fixed_function_usage_map(core::ptr::addr_of_mut!((*context_gl).c), state);
    }

    /* Try to go for a 1:1 mapping of the samplers when possible. Pixel shaders
     * need a 1:1 map at the moment.
     * When the mapping of a stage is changed, sampler and ALL texture stage
     * states have to be reset. */

    if (*gl_info).limits.graphics_samplers >= WINED3D_MAX_COMBINED_SAMPLERS as ::core::ffi::c_uint {
        return;
    }

    if ps != 0 {
        wined3d_context_gl_map_psamplers(context_gl, state);
    } else {
        wined3d_context_gl_map_fixed_function_samplers(context_gl, state);
    }

    if vs != 0 {
        wined3d_context_gl_map_vsamplers(context_gl, ps, state);
    }
}
#[no_mangle]
pub unsafe extern "C" fn context_state_drawbuf(context: *mut wined3d_context, state: *const wined3d_state, state_id: DWORD) {
    let context_gl: *mut wined3d_context_gl = wined3d_context_gl(context);
    let mut rt_mask: uint32_t = 0;
    let mut cur_mask: *mut uint32_t = core::ptr::null_mut::<uint32_t>();

    if isStateDirty(context, (STATE_FRAMEBUFFER as u32)) != false {
        return;
    }

    cur_mask = if !(*context_gl).current_fbo.is_null() {
        core::ptr::addr_of_mut!((*(*context_gl).current_fbo).rt_mask)
    } else {
        core::ptr::addr_of_mut!((*context_gl).draw_buffers_mask)
    };
    rt_mask = find_draw_buffers_mask(context_gl, state);
    if rt_mask != *cur_mask {
        wined3d_context_gl_apply_draw_buffers(context_gl, rt_mask);
        *cur_mask = rt_mask;
    }
}

unsafe extern "C" fn wined3d_context_gl_bind_shader_resources(
    context_gl: *mut wined3d_context_gl,
    state: *const wined3d_state,
    shader_type: wined3d_shader_type,
) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut bind_idx: ::core::ffi::c_uint = 0;
    let mut shader_sampler_count: ::core::ffi::c_uint = 0;
    let mut base: ::core::ffi::c_uint = 0;
    let mut count: ::core::ffi::c_uint = 0;
    let mut i: ::core::ffi::c_uint = 0;
    let device: *const wined3d_device = (*context_gl).c.device;
    let mut entry: *mut wined3d_shader_sampler_map_entry =
        ::core::ptr::null_mut::<wined3d_shader_sampler_map_entry>();
    let mut view: *mut wined3d_shader_resource_view =
        ::core::ptr::null_mut::<wined3d_shader_resource_view>();
    let mut shader: *const wined3d_shader = ::core::ptr::null::<wined3d_shader>();
    let mut tex_unit_map: *const ::core::ffi::c_uint = ::core::ptr::null::<::core::ffi::c_uint>();
    let mut sampler: *mut wined3d_sampler = ::core::ptr::null_mut::<wined3d_sampler>();

    shader = (*state).shader[shader_type as usize];
    if shader.is_null() {
        if shader_type as ::core::ffi::c_uint == WINED3D_SHADER_TYPE_PIXEL {
            let mut ffu_map: uint32_t = (*context_gl).c.fixed_function_usage_map() as uint32_t;

            while ffu_map != 0 {
                i = wined3d_bit_scan(core::ptr::addr_of_mut!(ffu_map)) as ::core::ffi::c_uint;
                bind_idx = (*context_gl).tex_unit_map[i as usize];

                view = (*state).shader_resource_view[(WINED3D_SHADER_TYPE_PIXEL) as usize][i as usize];
                sampler = (*state).sampler[(WINED3D_SHADER_TYPE_PIXEL) as usize][i as usize];

                if !view.is_null() {
                    wined3d_shader_resource_view_gl_bind(
                        wined3d_shader_resource_view_gl(view),
                        bind_idx,
                        wined3d_sampler_gl(sampler),
                        context_gl,
                    );
                } else {
                    WARN!("No resource view bound at index %u.\n", i);
                    wined3d_context_gl_active_texture(context_gl, gl_info, bind_idx);
                    wined3d_context_gl_bind_texture(
                        context_gl,
                        GL_NONE as GLenum,
                        0 as GLuint,
                    );
                    if (*gl_info).supported[(ARB_SAMPLER_OBJECTS) as usize] != 0 {
                        ((*gl_info).gl_ops.ext.p_glBindSampler.unwrap_unchecked())(
                            bind_idx as GLuint,
                            0 as GLuint,
                        );
                    }
                }
            }
        }

        return;
    }

    tex_unit_map = wined3d_context_gl_get_tex_unit_mapping(
        context_gl,
        core::ptr::addr_of!((*shader).reg_maps.shader_version),
        core::ptr::addr_of_mut!(base),
        core::ptr::addr_of_mut!(count),
    );

    shader_sampler_count = (*shader).reg_maps.sampler_map.count as ::core::ffi::c_uint;
    if shader_sampler_count > count {
        FIXME!(
            "Shader %p needs %u samplers, but only %u are supported.\n",
            shader,
            shader_sampler_count,
            count
        );
    }
    count = min!(shader_sampler_count, count);

    i = 0 as ::core::ffi::c_uint;
    while i < count {
        entry = (*shader).reg_maps.sampler_map.entries.offset(i as isize)
            as *mut wined3d_shader_sampler_map_entry;
        bind_idx = base.wrapping_add((*entry).bind_idx);
        if !tex_unit_map.is_null() {
            bind_idx = *tex_unit_map.offset(bind_idx as isize);
        }

        view = (*state).shader_resource_view[shader_type as usize][(*entry).resource_idx as usize];
        if !view.is_null() {
            if (*entry).sampler_idx == (WINED3D_SAMPLER_DEFAULT) as u32 {
                sampler = (*device).default_sampler;
            } else {
                sampler = (*state).sampler[shader_type as usize][(*entry).sampler_idx as usize];
                if sampler.is_null() {
                    sampler = (*device).null_sampler;
                }
            }
            wined3d_shader_resource_view_gl_bind(
                wined3d_shader_resource_view_gl(view),
                bind_idx,
                wined3d_sampler_gl(sampler),
                context_gl,
            );
        } else {
            WARN!(
                "No resource view bound at index %u, %u.\n",
                shader_type as ::core::ffi::c_uint,
                (*entry).resource_idx
            );
            wined3d_context_gl_active_texture(context_gl, gl_info, bind_idx);
            wined3d_context_gl_bind_texture(
                context_gl,
                GL_NONE as GLenum,
                0 as GLuint,
            );
            if (*gl_info).supported[(ARB_SAMPLER_OBJECTS) as usize] != 0 {
                ((*gl_info).gl_ops.ext.p_glBindSampler.unwrap_unchecked())(
                    bind_idx as GLuint,
                    0 as GLuint,
                );
            }
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn wined3d_context_gl_bind_unordered_access_views(
    context_gl: *mut wined3d_context_gl,
    shader: *const wined3d_shader,
    views: *const *mut wined3d_unordered_access_view,
) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut view_gl: *mut wined3d_unordered_access_view_gl = core::ptr::null_mut();
    let mut format_gl: *const wined3d_format_gl = core::ptr::null();
    let mut texture_name: GLuint = 0;
    let mut i: core::ffi::c_uint;
    let mut level: GLint = 0;

    if shader.is_null() {
        return;
    }

    i = 0;
    while i < MAX_UNORDERED_ACCESS_VIEWS {
        'body: {
            if (*views.offset(i as isize)).is_null() {
                if (*shader).reg_maps.uav_resource_info[i as usize].type_ != 0 {
                    WARN!("No unordered access view bound at index %u.\n", i);
                }
                ((*gl_info).gl_ops.ext.p_glBindImageTexture.unwrap_unchecked())(
                    i as GLuint,
                    0,
                    0,
                    (GL_FALSE as u8),
                    0,
                    GL_READ_WRITE,
                    GL_R8
                );
                break 'body;
            }

            view_gl = wined3d_unordered_access_view_gl(*views.offset(i as isize));
            if (*view_gl).gl_view.name != 0 {
                texture_name = (*view_gl).gl_view.name;
                level = 0;
            } else if (*(*view_gl).v.resource).type_
                != WINED3D_RTYPE_BUFFER
            {
                let texture_gl: *mut wined3d_texture_gl =
                    wined3d_texture_gl(texture_from_resource((*view_gl).v.resource));
                texture_name = wined3d_texture_gl_get_texture_name(
                    texture_gl,
                    &raw mut (*context_gl).c,
                    FALSE != 0,
                );
                level = (*view_gl).v.desc.u.texture.level_idx as GLint;
            } else {
                ((*gl_info).gl_ops.ext.p_glBindImageTexture.unwrap_unchecked())(
                    i as GLuint,
                    0,
                    0,
                    (GL_FALSE as u8),
                    0,
                    GL_READ_WRITE,
                    GL_R8
                );
                break 'body;
            }

            format_gl = wined3d_format_gl((*view_gl).v.format);
            ((*gl_info).gl_ops.ext.p_glBindImageTexture.unwrap_unchecked())(
                i as GLuint,
                texture_name,
                level,
                (GL_TRUE as u8),
                0,
                GL_READ_WRITE,
                (*format_gl).internal as GLenum
            );

            if (*view_gl).counter_bo.id != 0 {
                ((*gl_info).gl_ops.ext.p_glBindBufferRange.unwrap_unchecked())(
                    GL_ATOMIC_COUNTER_BUFFER,
                    i as GLuint,
                    (*view_gl).counter_bo.id,
                    (*view_gl).counter_bo.b.buffer_offset as GLintptr,
                    (*view_gl).counter_bo.size
                );
            }
        }
        i = i.wrapping_add(1);
    }
    checkGLcall!(gl_info, "Bind unordered access views");
}

unsafe extern "C" fn is_resource_rtv_bound(state: *const wined3d_state, resource: *const wined3d_resource) -> bool {
    let mut i: ::core::ffi::c_uint = 0;

    if !(*state).fb.depth_stencil.is_null()
        && (*(*state).fb.depth_stencil).resource == resource as *mut wined3d_resource
    {
        return true;
    }

    if (*resource).rtv_bind_count_device == 0 {
        return false;
    }

    while (i as usize) < ARRAY_SIZE!((*state).fb.render_targets) as usize {
        if !(*state).fb.render_targets[i as usize].is_null()
            && (*(*state).fb.render_targets[i as usize]).resource
                == resource as *mut wined3d_resource
        {
            return true;
        }
        i = i.wrapping_add(1);
    }

    return false;
}

unsafe extern "C" fn context_gl_load_shader_resources(
    mut context_gl: *mut wined3d_context_gl,
    mut state: *const wined3d_state,
    mut shader_mask: ::core::ffi::c_uint,
) {
    let mut entry: *mut wined3d_shader_sampler_map_entry =
        ::core::ptr::null_mut::<wined3d_shader_sampler_map_entry>();
    let mut srv_gl: *mut wined3d_shader_resource_view_gl =
        ::core::ptr::null_mut::<wined3d_shader_resource_view_gl>();
    let mut view: *mut wined3d_shader_resource_view =
        ::core::ptr::null_mut::<wined3d_shader_resource_view>();
    let mut sampler: *mut wined3d_sampler = ::core::ptr::null_mut::<wined3d_sampler>();
    let mut shader: *mut wined3d_shader = ::core::ptr::null_mut::<wined3d_shader>();
    let mut buffer: *mut wined3d_buffer = ::core::ptr::null_mut::<wined3d_buffer>();
    let mut i: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_uint = 0;

    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_SHADER_TYPE_COUNT as ::core::ffi::c_uint {
        if (shader_mask & ((1 as ::core::ffi::c_uint) << i)) == 0 {
            i = i.wrapping_add(1);
            continue;
        }

        shader = (*state).shader[i as usize];
        if shader.is_null() {
            if i == WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_uint {
                let mut ffu_map: uint32_t = (*context_gl).c.fixed_function_usage_map() as uint32_t;

                while ffu_map != 0 {
                    i = wined3d_bit_scan(core::ptr::addr_of_mut!(ffu_map))
                        as ::core::ffi::c_uint;

                    view = (*state).shader_resource_view
                        [WINED3D_SHADER_TYPE_PIXEL as usize][i as usize];
                    sampler = (*state).sampler
                        [WINED3D_SHADER_TYPE_PIXEL as usize][i as usize];

                    if !view.is_null() {
                        wined3d_texture_load(
                            texture_from_resource((*view).resource),
                            core::ptr::addr_of_mut!((*context_gl).c),
                            (*sampler).desc.srgb_decode,
                        );
                    }
                }
            }

            i = i.wrapping_add(1);
            continue;
        }

        j = 0 as ::core::ffi::c_uint;
        while j < WINED3D_MAX_CBS as ::core::ffi::c_uint {
            if (*state).cb[i as usize][j as usize].buffer.is_null() {
                j = j.wrapping_add(1);
                continue;
            }

            buffer = (*state).cb[i as usize][j as usize].buffer;
            wined3d_buffer_load(buffer, core::ptr::addr_of_mut!((*context_gl).c), state);
            wined3d_context_gl_reference_buffer(context_gl, buffer);
            if !(*buffer).bo_user.valid {
                device_invalidate_state(
                    (*context_gl).c.device,
                    STATE_CONSTANT_BUFFER!((i) as i32),
                );
            }
            j = j.wrapping_add(1);
        }

        j = 0 as ::core::ffi::c_uint;
        while (j as size_t) < (*shader).reg_maps.sampler_map.count {
            entry = (*shader).reg_maps.sampler_map.entries.offset(j as isize)
                as *mut wined3d_shader_sampler_map_entry;

            view = (*state).shader_resource_view[i as usize][(*entry).resource_idx as usize];
            if view.is_null() {
                j = j.wrapping_add(1);
                continue;
            }

            if is_resource_rtv_bound(state, (*view).resource) {
                (*context_gl).c.set_uses_fbo_attached_resources((1 as DWORD) as _);
            }

            if (*(*view).resource).type_ as ::core::ffi::c_uint
                == WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint
            {
                buffer = buffer_from_resource((*view).resource);
                wined3d_buffer_load(buffer, core::ptr::addr_of_mut!((*context_gl).c), state);
                wined3d_context_gl_reference_buffer(context_gl, buffer);

                srv_gl = wined3d_shader_resource_view_gl(view);
                if !(*srv_gl).bo_user.valid {
                    wined3d_shader_resource_view_gl_update(srv_gl, context_gl);
                }
            } else {
                let mut srgb: BOOL = TRUE as BOOL;

                if (*entry).sampler_idx != (WINED3D_SAMPLER_DEFAULT) as u32 && {
                    sampler = (*state).sampler[i as usize][(*entry).sampler_idx as usize];
                    !sampler.is_null()
                } {
                    srgb = (*sampler).desc.srgb_decode;
                }

                wined3d_texture_load(
                    texture_from_resource((*view).resource),
                    core::ptr::addr_of_mut!((*context_gl).c),
                    srgb,
                );
            }

            j = j.wrapping_add(1);
        }

        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn context_gl_load_unordered_access_resources(
    context_gl: *mut wined3d_context_gl,
    shader: *const wined3d_shader,
    views: *const *mut wined3d_unordered_access_view,
) {
    let mut uav_gl: *mut wined3d_unordered_access_view_gl = core::ptr::null_mut();
    let mut view: *mut wined3d_unordered_access_view = core::ptr::null_mut();
    let mut texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut buffer: *mut wined3d_buffer = core::ptr::null_mut();
    let mut i: ::core::ffi::c_uint;

    (*context_gl).c.set_uses_uavs((0) as _);

    if shader.is_null() {
        return;
    }

    i = 0;
    while i < MAX_UNORDERED_ACCESS_VIEWS {
        'body: {
            view = *views.offset(i as isize);
            if view.is_null() {
                break 'body;
            }

            if (*(*view).resource).type_ as u32 == WINED3D_RTYPE_BUFFER {
                buffer = buffer_from_resource((*view).resource);
                wined3d_buffer_acquire_bo_for_write(buffer, core::ptr::addr_of_mut!((*context_gl).c));
                wined3d_buffer_load_location(
                    buffer,
                    core::ptr::addr_of_mut!((*context_gl).c),
                    WINED3D_LOCATION_BUFFER,
                );
                wined3d_unordered_access_view_invalidate_location(view, !WINED3D_LOCATION_BUFFER);
                wined3d_context_gl_reference_buffer(context_gl, buffer);

                uav_gl = wined3d_unordered_access_view_gl(view);
                if !(*uav_gl).bo_user.valid {
                    wined3d_unordered_access_view_gl_update(uav_gl, context_gl);
                }
            } else {
                texture = texture_from_resource((*view).resource);
                wined3d_texture_load(
                    texture,
                    core::ptr::addr_of_mut!((*context_gl).c),
                    (FALSE) as BOOL,
                );
                wined3d_unordered_access_view_invalidate_location(view, !WINED3D_LOCATION_TEXTURE_RGB);
            }

            (*context_gl).c.set_uses_uavs((1) as _);
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn context_gl_load_stream_output_buffers(
    context_gl: *mut wined3d_context_gl,
    state: *const wined3d_state,
) {
    let mut buffer: *mut wined3d_buffer = core::ptr::null_mut();
    let mut i: u32 = 0;
    while (i as usize) < ARRAY_SIZE!((*state).stream_output) as usize {
        buffer = (*state).stream_output[i as usize].buffer;
        if !buffer.is_null() {
            wined3d_buffer_acquire_bo_for_write(buffer, &raw mut (*context_gl).c);

            wined3d_buffer_load(buffer, &raw mut (*context_gl).c, state);
            wined3d_buffer_invalidate_location(buffer, (!WINED3D_LOCATION_BUFFER) as u32);
            wined3d_context_gl_reference_buffer(context_gl, buffer);
            if !(*buffer).bo_user.valid {
                device_invalidate_state((*context_gl).c.device, (STATE_STREAM_OUTPUT as u32));
            }
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn context_apply_draw_state(
    mut context: *mut wined3d_context,
    mut device: *const wined3d_device,
    mut state: *const wined3d_state,
    mut indexed: BOOL,
) -> BOOL {
    let mut state_table: *const wined3d_state_entry = (*context).state_table as *const wined3d_state_entry;
    let mut context_gl: *mut wined3d_context_gl = wined3d_context_gl(context);
    let mut gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut fb: *const wined3d_fb_state = core::ptr::addr_of!((*state).fb);
    let mut i: u32 = 0;
    let mut base: u32 = 0;
    let mut map: u32 = 0;

    (*context).set_uses_fbo_attached_resources(0 as DWORD);

    if have_framebuffer_attachment(
        (*gl_info).limits.buffers,
        core::ptr::addr_of!((*fb).render_targets) as *const *mut wined3d_rendertarget_view,
        (*fb).depth_stencil,
    ) == 0
    {
        if (*gl_info).supported[ARB_FRAMEBUFFER_NO_ATTACHMENTS as usize] == 0 {
            FIXME!("OpenGL implementation does not support framebuffers with no attachments.\n");
            return (FALSE) as BOOL;
        }
    }

    /* Preload resources before FBO setup. Texture preload in particular may
     * result in changes to the current FBO, due to using e.g. FBO blits for
     * updating a resource location. */
    wined3d_context_gl_update_tex_unit_map(context_gl, state);
    context_gl_load_shader_resources(
        context_gl,
        state,
        (!(1 << WINED3D_SHADER_TYPE_COMPUTE)) as u32,
    );
    context_gl_load_unordered_access_resources(
        context_gl,
        (*state).shader[WINED3D_SHADER_TYPE_PIXEL as usize],
        (core::ptr::addr_of!((*state).unordered_access_view) as *const [*mut wined3d_unordered_access_view; 8])
            .offset(WINED3D_PIPELINE_GRAPHICS as isize) as *const *mut wined3d_unordered_access_view,
    );
    context_gl_load_stream_output_buffers(context_gl, state);
    /* TODO: Right now the dependency on the vertex shader is necessary
     * since wined3d_stream_info_from_declaration() depends on the reg_maps of
     * the current VS but maybe it's possible to relax the coupling in some
     * situations at least. */
    if isStateDirty(context, (STATE_VDECL as u32)) != false
        || isStateDirty(context, (STATE_STREAMSRC as u32)) != false
        || isStateDirty(context, STATE_SHADER!((WINED3D_SHADER_TYPE_VERTEX) as i32)) != false
    {
        context_update_stream_info(context, state);
    }

    map = (*context).stream_info.use_map as u32;
    while map != 0 {
        let mut e: *const wined3d_stream_info_element = core::ptr::null::<wined3d_stream_info_element>();
        let mut buffer: *mut wined3d_buffer = core::ptr::null_mut::<wined3d_buffer>();

        e = (core::ptr::addr_of_mut!((*context).stream_info.elements) as *mut wined3d_stream_info_element)
            .offset(wined3d_bit_scan(core::ptr::addr_of_mut!(map)) as isize);
        buffer = (*state).streams[(*e).stream_idx as usize].buffer as *mut wined3d_buffer;

        if !(*buffer).bo_user.valid {
            device_invalidate_state(device, (STATE_STREAMSRC as u32));
        } else {
            wined3d_buffer_load(buffer, context, state);
        }
    }
    /* Loading the buffers above may have invalidated the stream info. */
    if wined3d_context_is_graphics_state_dirty(context, (STATE_STREAMSRC as u32)) != false {
        context_update_stream_info(context, state);
    }

    map = (*context).stream_info.use_map as u32;
    while map != 0 {
        let mut e_0: *const wined3d_stream_info_element = core::ptr::null::<wined3d_stream_info_element>();
        let mut buffer_0: *mut wined3d_buffer = core::ptr::null_mut::<wined3d_buffer>();

        e_0 = (core::ptr::addr_of_mut!((*context).stream_info.elements) as *mut wined3d_stream_info_element)
            .offset(wined3d_bit_scan(core::ptr::addr_of_mut!(map)) as isize);
        buffer_0 = (*state).streams[(*e_0).stream_idx as usize].buffer as *mut wined3d_buffer;

        wined3d_context_gl_reference_buffer(context_gl, buffer_0);
    }

    if indexed != 0 && !(*state).index_buffer.is_null() {
        let mut buffer_1: *mut wined3d_buffer = (*state).index_buffer;

        if (*context).stream_info.all_vbo() != 0 {
            wined3d_buffer_load(buffer_1, context, state);
            if !(*buffer_1).bo_user.valid {
                device_invalidate_state(device, (STATE_INDEXBUFFER as u32));
            }
            wined3d_context_gl_reference_buffer(context_gl, buffer_1);
        } else {
            wined3d_buffer_load_sysmem(buffer_1, context);
        }
    }

    i = 0;
    base = 0;
    while (i as usize) < ARRAY_SIZE!((*context).dirty_graphics_states) as usize {
        let mut dirty_mask: u32 = (*context).dirty_graphics_states[i as usize];

        while dirty_mask != 0 {
            let state_id: u32 = base.wrapping_add(wined3d_bit_scan(core::ptr::addr_of_mut!(dirty_mask)) as u32);

            (*state_table.offset(state_id as isize))
                .apply
                .unwrap_unchecked()(context, state, state_id as DWORD);
        }
        base = base.wrapping_add(
            (core::mem::size_of::<u32>() as u32).wrapping_mul(CHAR_BIT as u32),
        );
        i = i.wrapping_add(1);
    }

    memset(
        core::ptr::addr_of_mut!((*context).dirty_graphics_states) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<[u32; 13]>(),
    );

    if (*context).update_shader_resource_bindings() != 0 {
        i = 0;
        while i < WINED3D_SHADER_TYPE_GRAPHICS_COUNT {
            wined3d_context_gl_bind_shader_resources(context_gl, state, i as wined3d_shader_type);
            i = i.wrapping_add(1);
        }
        (*context).set_update_shader_resource_bindings(0 as DWORD);
        if (*gl_info).limits.combined_samplers == (*gl_info).limits.graphics_samplers {
            (*context).set_update_compute_shader_resource_bindings(1 as DWORD);
        }
    }

    if (*context).update_unordered_access_view_bindings() != 0 {
        wined3d_context_gl_bind_unordered_access_views(
            context_gl,
            (*state).shader[WINED3D_SHADER_TYPE_PIXEL as usize],
            (core::ptr::addr_of!((*state).unordered_access_view) as *const [*mut wined3d_unordered_access_view; 8])
                .offset(WINED3D_PIPELINE_GRAPHICS as isize) as *const *mut wined3d_unordered_access_view,
        );
        (*context).set_update_unordered_access_view_bindings(0 as DWORD);
        (*context).set_update_compute_unordered_access_view_bindings(1 as DWORD);
    }

    wined3d_context_gl_check_fbo_status(context_gl, GL_FRAMEBUFFER);

    (*(*device).shader_backend)
        .shader_apply_draw_state
        .unwrap_unchecked()((*device).shader_priv, context, state);
    (*context).set_shader_update_mask(
        (*context).shader_update_mask() & ((1 << WINED3D_SHADER_TYPE_COMPUTE) as DWORD),
    );
    (*context).constant_update_mask = 0 as DWORD;

    (*context).set_last_was_blit(FALSE as DWORD);
    (*context).set_last_was_ffp_blit(FALSE as DWORD);

    return (TRUE) as BOOL;
}

unsafe extern "C" fn wined3d_context_gl_apply_compute_state(
    context_gl: *mut wined3d_context_gl,
    device: *const wined3d_device,
    state: *const wined3d_state,
) {
    let state_table: *const wined3d_state_entry = (*context_gl).c.state_table as *const wined3d_state_entry;
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut state_id: u32;
    let mut i: u32;

    context_gl_load_shader_resources(context_gl, state, 1 << WINED3D_SHADER_TYPE_COMPUTE);
    context_gl_load_unordered_access_resources(
        context_gl,
        (*state).shader[(WINED3D_SHADER_TYPE_COMPUTE) as usize],
        (core::ptr::addr_of!((*state).unordered_access_view) as *const [*mut wined3d_unordered_access_view; 8])
            .offset(WINED3D_PIPELINE_COMPUTE as isize) as *const *mut wined3d_unordered_access_view,
    );

    i = 0;
    state_id = (STATE_COMPUTE_OFFSET as u32);
    while (i as usize) < ARRAY_SIZE!((*context_gl).c.dirty_compute_states) as usize {
        let mut dirty_mask: u32 = (*(*context_gl).c.dirty_compute_states.as_ptr().add(i as usize));

        while dirty_mask != 0 {
            let current_state_id: u32 = state_id
                .wrapping_add(wined3d_bit_scan(core::ptr::addr_of_mut!(dirty_mask)) as u32);
            ((*state_table.offset(current_state_id as isize)).apply.unwrap_unchecked())(
                core::ptr::addr_of_mut!((*context_gl).c),
                state,
                current_state_id as DWORD,
            );
        }
        state_id = state_id.wrapping_add(
            (core::mem::size_of::<u32>() as u32).wrapping_mul(CHAR_BIT as u32),
        );
        i = i.wrapping_add(1);
    }
    memset(
        core::ptr::addr_of_mut!((*context_gl).c.dirty_compute_states) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<u32>() as usize,
    );

    ((*(*device).shader_backend).shader_apply_compute_state.unwrap_unchecked())(
        (*device).shader_priv,
        core::ptr::addr_of_mut!((*context_gl).c),
        state,
    );

    if (*context_gl).c.update_compute_shader_resource_bindings() != 0 {
        wined3d_context_gl_bind_shader_resources(context_gl, state, WINED3D_SHADER_TYPE_COMPUTE);
        (*context_gl).c.set_update_compute_shader_resource_bindings((0) as _);
        if (*gl_info).limits.combined_samplers == (*gl_info).limits.graphics_samplers {
            (*context_gl).c.set_update_shader_resource_bindings((1) as _);
        }
    }

    if (*context_gl).c.update_compute_unordered_access_view_bindings() != 0 {
        wined3d_context_gl_bind_unordered_access_views(
            context_gl,
            (*state).shader[(WINED3D_SHADER_TYPE_COMPUTE) as usize],
            (core::ptr::addr_of!((*state).unordered_access_view) as *const [*mut wined3d_unordered_access_view; 8])
                .offset(WINED3D_PIPELINE_COMPUTE as isize) as *const *mut wined3d_unordered_access_view,
        );
        (*context_gl).c.set_update_compute_unordered_access_view_bindings((0) as _);
        (*context_gl).c.set_update_unordered_access_view_bindings((1) as _);
    }

    /* Updates to currently bound render targets aren't necessarily coherent
     * between the graphics and compute pipelines. Unbind any currently bound
     * FBO here to ensure preceding updates to its attachments by the graphics
     * pipeline are visible to the compute pipeline.
     *
     * Without this, the bloom effect in Nier:Automata is too bright on the
     * Mesa radeonsi driver, and presumably on other Mesa based drivers. */
    wined3d_context_gl_bind_fbo(context_gl, GL_FRAMEBUFFER, 0);
    context_invalidate_state(core::ptr::addr_of_mut!((*context_gl).c), (STATE_FRAMEBUFFER as u32));

    (*context_gl).c.set_last_was_blit((FALSE) as _);
    (*context_gl).c.set_last_was_ffp_blit((FALSE) as _);
    (*context_gl).c.set_shader_update_mask(((*context_gl).c.shader_update_mask() & (!(1 << WINED3D_SHADER_TYPE_COMPUTE))) as _);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_end_transform_feedback(context_gl: *mut wined3d_context_gl) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;

    if (*context_gl).c.transform_feedback_active() != 0 {
        ((*gl_info).gl_ops.ext.p_glEndTransformFeedback.unwrap_unchecked())();
        checkGLcall!(gl_info, "glEndTransformFeedback");
        (*context_gl).c.set_transform_feedback_active((0) as _);
        (*context_gl).c.set_transform_feedback_paused((0) as _);
    }
}

unsafe extern "C" fn wined3d_context_gl_pause_transform_feedback(context_gl: *mut wined3d_context_gl, force: BOOL) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;

    if (*context_gl).c.transform_feedback_active() == 0 || (*context_gl).c.transform_feedback_paused() != 0 {
        return;
    }

    if (*gl_info).supported[ARB_TRANSFORM_FEEDBACK2 as usize] != 0 {
        ((*gl_info).gl_ops.ext.p_glPauseTransformFeedback.unwrap_unchecked())();
        checkGLcall!(gl_info, "glPauseTransformFeedback");
        (*context_gl).c.set_transform_feedback_paused(1 as DWORD);
        return;
    }

    WARN!("Cannot pause transform feedback operations.\n");

    if force != 0 {
        wined3d_context_gl_end_transform_feedback(context_gl);
    }
}

unsafe extern "C" fn wined3d_context_gl_setup_target(
    mut context_gl: *mut wined3d_context_gl,
    mut texture: *mut wined3d_texture,
    mut sub_resource_idx: ::core::ffi::c_uint,
) {
    if (*context_gl).c.current_rt.texture == texture
        && (*context_gl).c.current_rt.sub_resource_idx == sub_resource_idx
    {
        return;
    }
    if (*context_gl).c.current_rt.texture.is_null() {
        context_invalidate_state(
            core::ptr::addr_of_mut!((*context_gl).c),
            (STATE_BLEND) as ::core::ffi::c_uint,
        );
    } else {
        let mut old: *const wined3d_format = (*(*context_gl).c.current_rt.texture).resource.format;
        let mut new: *const wined3d_format = (*texture).resource.format;
        if (*old).id as ::core::ffi::c_uint != (*new).id as ::core::ffi::c_uint {
            if ((*old).alpha_size != 0 && (*new).alpha_size == 0)
                || ((*old).alpha_size == 0 && (*new).alpha_size != 0)
                || (((*texture).resource.format_caps & (WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING as ::core::ffi::c_uint)) == 0)
            {
                context_invalidate_state(
                    core::ptr::addr_of_mut!((*context_gl).c),
                    (STATE_BLEND) as ::core::ffi::c_uint,
                );
            }
        }
    }
    (*context_gl).c.current_rt.texture = texture;
    (*context_gl).c.current_rt.sub_resource_idx = sub_resource_idx;
}

unsafe extern "C" fn wined3d_context_gl_activate(mut context_gl: *mut wined3d_context_gl, mut texture: *mut wined3d_texture, mut sub_resource_idx: ::core::ffi::c_uint) {
    wined3d_context_gl_enter(context_gl);
    if !texture.is_null() && !(*texture).swapchain.is_null() && (*texture).swapchain != (*context_gl).c.swapchain {
        (*context_gl).c.swapchain = (*texture).swapchain;
    }
    wined3d_context_gl_update_window(context_gl);
    wined3d_context_gl_setup_target(context_gl, texture, sub_resource_idx);
    if (*context_gl).valid() == 0 {
        return;
    }
    if context_gl != wined3d_context_gl_get_current() {
        if wined3d_context_gl_set_current(context_gl) == 0 {
            ERR!("Failed to activate the new context.\n");
        }
    } else if (*context_gl).needs_set() != 0 {
        wined3d_context_gl_set_gl_context(context_gl);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_acquire(mut device: *const wined3d_device, mut texture: *mut wined3d_texture, mut sub_resource_idx: ::core::ffi::c_uint) -> *mut wined3d_context {
    let mut current_context: *mut wined3d_context_gl = wined3d_context_gl_get_current();
    let mut context_gl: *mut wined3d_context_gl = ::core::ptr::null_mut::<wined3d_context_gl>();
    if !current_context.is_null() && (*current_context).c.destroyed() != 0 {
        current_context = ::core::ptr::null_mut::<wined3d_context_gl>();
    }
    if texture.is_null() {
        if !current_context.is_null()
            && (*current_context).c.device == device as *mut wined3d_device
        {
            texture = (*current_context).c.current_rt.texture;
            sub_resource_idx = (*current_context).c.current_rt.sub_resource_idx;
        } else {
            let mut swapchain: *mut wined3d_swapchain =
                *(*device).swapchains.offset(0 as isize);
            if !(*swapchain).back_buffers.is_null() {
                texture = *(*swapchain)
                    .back_buffers
                    .offset(0 as isize);
            } else {
                texture = (*swapchain).front_buffer;
            }
            sub_resource_idx = 0 as ::core::ffi::c_uint;
        }
    }
    if !current_context.is_null() && (*current_context).c.current_rt.texture == texture {
        context_gl = current_context;
    } else if wined3d_resource_is_offscreen(core::ptr::addr_of_mut!((*texture).resource)) == 0 {
        context_gl = wined3d_swapchain_gl_get_context(wined3d_swapchain_gl((*texture).swapchain));
        if context_gl.is_null() {
            return ::core::ptr::null_mut::<wined3d_context>();
        }
    } else {
        if !current_context.is_null()
            && (*current_context).c.device == device as *mut wined3d_device
        {
            context_gl = current_context;
        } else {
            context_gl = wined3d_swapchain_gl_get_context(wined3d_swapchain_gl(
                *(*device).swapchains.offset(0 as isize),
            ));
            if context_gl.is_null() {
                return ::core::ptr::null_mut::<wined3d_context>();
            }
        }
    }
    wined3d_context_gl_activate(context_gl, texture, sub_resource_idx);
    return core::ptr::addr_of_mut!((*context_gl).c);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_reacquire(mut context_gl: *mut wined3d_context_gl) -> *mut wined3d_context_gl {
    let mut acquired_context: *mut wined3d_context = core::ptr::null_mut::<wined3d_context>();
    let mut device: *mut wined3d_device = core::ptr::null_mut::<wined3d_device>();

    if context_gl.is_null() || (*context_gl).tid != GetCurrentThreadId() {
        return core::ptr::null_mut::<wined3d_context_gl>();
    }

    device = (*context_gl).c.device;
    wined3d_from_cs((*device).cs);

    if !(*context_gl).c.current_rt.texture.is_null() {
        wined3d_context_gl_activate(
            context_gl,
            (*context_gl).c.current_rt.texture,
            (*context_gl).c.current_rt.sub_resource_idx,
        );
        return context_gl;
    }

    acquired_context = context_acquire(
        device,
        core::ptr::null_mut::<wined3d_texture>(),
        0,
    );
    if acquired_context != core::ptr::addr_of_mut!((*context_gl).c) {
        ERR!(
            "Acquired context %p instead of %p.\n",
            acquired_context,
            core::ptr::addr_of_mut!((*context_gl).c)
        );
    }
    return wined3d_context_gl(acquired_context);
}
#[no_mangle]
pub unsafe extern "C" fn dispatch_compute(device: *mut wined3d_device, state: *const wined3d_state, parameters: *const wined3d_dispatch_parameters) {
    let mut gl_info: *const wined3d_gl_info;
    let mut context_gl: *mut wined3d_context_gl;

    context_gl = wined3d_context_gl(context_acquire(device, core::ptr::null_mut(), 0));
    if (*context_gl).valid() == 0 {
        context_release(core::ptr::addr_of_mut!((*context_gl).c));
        WARN!("Invalid context, skipping dispatch.\n");
        return;
    }
    gl_info = (*context_gl).gl_info;

    if (*gl_info).supported[ARB_COMPUTE_SHADER as usize] == 0 {
        context_release(core::ptr::addr_of_mut!((*context_gl).c));

        return;
    }

    if (*parameters).indirect != 0 {
        wined3d_buffer_load((*parameters).u.indirect.buffer, core::ptr::addr_of_mut!((*context_gl).c), state);
    }

    wined3d_context_gl_apply_compute_state(context_gl, device, state);

    if (*parameters).indirect != 0 {
        let indirect: *const wined3d_indirect_dispatch_parameters = core::ptr::addr_of!((*parameters).u.indirect);
        let bo_gl: *mut wined3d_bo_gl = wined3d_bo_gl((*(*indirect).buffer).buffer_object);

        ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(GL_DISPATCH_INDIRECT_BUFFER, (*bo_gl).id);
        ((*gl_info).gl_ops.ext.p_glDispatchComputeIndirect.unwrap_unchecked())((*bo_gl).b.buffer_offset.wrapping_add((*indirect).offset as GLintptr as size_t) as GLintptr);
        ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(GL_DISPATCH_INDIRECT_BUFFER, 0);
        wined3d_context_gl_reference_bo(context_gl, bo_gl);
    } else {
        let direct: *const wined3d_direct_dispatch_parameters = core::ptr::addr_of!((*parameters).u.direct);
        ((*gl_info).gl_ops.ext.p_glDispatchCompute.unwrap_unchecked())(
            (*direct).group_count_x as GLuint,
            (*direct).group_count_y as GLuint,
            (*direct).group_count_z as GLuint,
        );
    }
    checkGLcall!(gl_info, "dispatch compute");

    ((*gl_info).gl_ops.ext.p_glMemoryBarrier.unwrap_unchecked())(GL_ALL_BARRIER_BITS);
    checkGLcall!(gl_info, "glMemoryBarrier");

    context_release(core::ptr::addr_of_mut!((*context_gl).c));
}

unsafe extern "C" fn wined3d_context_gl_draw_primitive_arrays(
    mut context_gl: *mut wined3d_context_gl,
    mut state: *const wined3d_state,
    mut idx_data: *const ::core::ffi::c_void,
    mut idx_size: ::core::ffi::c_uint,
    mut base_vertex_idx: ::core::ffi::c_int,
    mut start_idx: ::core::ffi::c_uint,
    mut count: ::core::ffi::c_uint,
    mut start_instance: ::core::ffi::c_uint,
    mut instance_count: ::core::ffi::c_uint,
) {
    let mut idx_type: GLenum = if idx_size == 2 {
        GL_UNSIGNED_SHORT
    } else {
        GL_UNSIGNED_INT
    } as GLenum;
    let mut si: *const wined3d_stream_info = core::ptr::addr_of!((*context_gl).c.stream_info);
    let mut mode: GLenum = gl_primitive_type_from_d3d((*state).primitive_type);
    let mut gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut instanced_elements: [::core::ffi::c_uint; 32] = [0 as ::core::ffi::c_uint; 32];
    let mut ops: *const wined3d_ffp_attrib_ops = core::ptr::null::<wined3d_ffp_attrib_ops>();
    let mut instanced_element_count: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut indices: *const ::core::ffi::c_void = core::ptr::null::<::core::ffi::c_void>();
    let mut i: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_uint = 0;

    ops = core::ptr::addr_of!((*gl_info).ffp_attrib_ops);

    indices = (idx_data as *const ::core::ffi::c_char)
        .offset(idx_size.wrapping_mul(start_idx) as isize)
        as *const ::core::ffi::c_void;

    if instance_count == 0 {
        if idx_size == 0 {
            (*gl_info)
                .gl_ops
                .gl
                .p_glDrawArrays
                .unwrap_unchecked()(
                mode, start_idx as GLint, count as GLsizei
            );
            checkGLcall!(gl_info, "glDrawArrays");
            return;
        }

        if (*gl_info).supported[ARB_DRAW_ELEMENTS_BASE_VERTEX as usize] != 0 {
            ((*gl_info).gl_ops.ext.p_glDrawElementsBaseVertex.unwrap_unchecked())(
                mode,
                count as GLsizei,
                idx_type,
                indices,
                base_vertex_idx as GLint,
            );
            checkGLcall!(gl_info, "glDrawElementsBaseVertex");
            return;
        }

        (*gl_info)
            .gl_ops
            .gl
            .p_glDrawElements
            .unwrap_unchecked()(mode, count as GLsizei, idx_type, indices);
        checkGLcall!(gl_info, "glDrawElements");
        return;
    }

    if start_instance != 0
        && !((*gl_info).supported[ARB_BASE_INSTANCE as usize] != 0
            && (*gl_info).supported[ARB_INSTANCED_ARRAYS as usize] != 0)
    {
        FIXME!("Start instance (%u) not supported.\n", start_instance);
    }

    if (*gl_info).supported[ARB_INSTANCED_ARRAYS as usize] != 0 {
        if idx_size == 0 {
            if (*gl_info).supported[ARB_BASE_INSTANCE as usize] != 0 {
                ((*gl_info).gl_ops.ext.p_glDrawArraysInstancedBaseInstance.unwrap_unchecked())(
                    mode,
                    start_idx as GLint,
                    count as GLsizei,
                    instance_count as GLsizei,
                    start_instance as GLuint,
                );
                checkGLcall!(gl_info, "glDrawArraysInstancedBaseInstance");
                return;
            }

            ((*gl_info).gl_ops.ext.p_glDrawArraysInstanced.unwrap_unchecked())(
                mode,
                start_idx as GLint,
                count as GLsizei,
                instance_count as GLsizei,
            );
            checkGLcall!(gl_info, "glDrawArraysInstanced");
            return;
        }

        if (*gl_info).supported[ARB_BASE_INSTANCE as usize] != 0 {
            ((*gl_info).gl_ops.ext.p_glDrawElementsInstancedBaseVertexBaseInstance.unwrap_unchecked())(
                mode,
                count as GLsizei,
                idx_type,
                indices,
                instance_count as GLsizei,
                base_vertex_idx as GLint,
                start_instance as GLuint,
            );
            checkGLcall!(gl_info, "glDrawElementsInstancedBaseVertexBaseInstance");
            return;
        }
        if (*gl_info).supported[ARB_DRAW_ELEMENTS_BASE_VERTEX as usize] != 0 {
            ((*gl_info).gl_ops.ext.p_glDrawElementsInstancedBaseVertex.unwrap_unchecked())(
                mode,
                count as GLsizei,
                idx_type,
                indices,
                instance_count as GLsizei,
                base_vertex_idx as GLint,
            );
            checkGLcall!(gl_info, "glDrawElementsInstancedBaseVertex");
            return;
        }

        ((*gl_info).gl_ops.ext.p_glDrawElementsInstanced.unwrap_unchecked())(
            mode,
            count as GLsizei,
            idx_type,
            indices,
            instance_count as GLsizei,
        );
        checkGLcall!(gl_info, "glDrawElementsInstanced");
        return;
    }

    /* Instancing emulation by mixing immediate mode and arrays. */

    /* This is a nasty thing. MSDN says no hardware supports this and
     * applications have to use software vertex processing. We don't support
     * this for now.
     *
     * Shouldn't be too hard to support with OpenGL, in theory just call
     * glDrawArrays() instead of drawElements(). But the stream frequency value
     * has a different meaning in that situation. */
    if idx_size == 0 {
        WARN!("Non-indexed instanced drawing is not supported.\n");
        return;
    }

    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!((*si).elements) as usize {
        if !((*si).use_map as ::core::ffi::c_uint & ((1 as ::core::ffi::c_uint) << i) == 0) {
            if (*state).streams[(*si).elements[i as usize].stream_idx as usize].flags
                as ::core::ffi::c_uint
                & WINED3DSTREAMSOURCE_INSTANCEDATA as ::core::ffi::c_uint
                != 0
            {
                let fresh16 = instanced_element_count;
                instanced_element_count = instanced_element_count.wrapping_add(1);
                instanced_elements[fresh16 as usize] = i;
            }
        }
        i = i.wrapping_add(1);
    }

    i = 0 as ::core::ffi::c_uint;
    while i < instance_count {
        j = 0 as ::core::ffi::c_uint;
        while j < instanced_element_count {
            let mut element: *const wined3d_stream_info_element =
                core::ptr::null::<wined3d_stream_info_element>();
            let mut element_idx: ::core::ffi::c_uint = 0;
            let mut ptr: *const BYTE = core::ptr::null::<BYTE>();

            element_idx = instanced_elements[j as usize];
            element = ((si as *const wined3d_stream_info_element)
                .cast::<wined3d_stream_info_element>())
                .offset(0)
                .add(0)
                .wrapping_add(element_idx as usize);
            ptr = (*element)
                .data
                .addr
                .offset((*element).stride.wrapping_mul(i) as isize);
            if !(*element).data.buffer_object.is_null() {
                ptr = ptr.offset(wined3d_buffer_load_sysmem(
                    (*state).streams[(*element).stream_idx as usize].buffer as *mut wined3d_buffer,
                    core::ptr::addr_of_mut!((*context_gl).c),
                ) as ULONG_PTR as isize);
            }
            (*ops).generic[(*(*element).format).emit_idx as usize]
                .unwrap_unchecked()(
                element_idx as GLuint,
                ptr as *const ::core::ffi::c_void,
            );
            j = j.wrapping_add(1);
        }

        if (*gl_info).supported[ARB_DRAW_ELEMENTS_BASE_VERTEX as usize] != 0 {
            ((*gl_info).gl_ops.ext.p_glDrawElementsBaseVertex.unwrap_unchecked())(
                mode,
                count as GLsizei,
                idx_type,
                indices,
                base_vertex_idx as GLint,
            );
            checkGLcall!(gl_info, "glDrawElementsBaseVertex");
        } else {
            (*gl_info)
                .gl_ops
                .gl
                .p_glDrawElements
                .unwrap_unchecked()(
                mode, count as GLsizei, idx_type, indices
            );
            checkGLcall!(gl_info, "glDrawElements");
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn wined3d_context_gl_draw_indirect(
    context_gl: *mut wined3d_context_gl,
    state: *const wined3d_state,
    parameters: *const wined3d_indirect_draw_parameters,
    idx_size: u32,
) {
    let bo_gl: *mut wined3d_bo_gl = wined3d_bo_gl((*(*parameters).buffer).buffer_object);
    let gl_primitive_type: GLenum = gl_primitive_type_from_d3d((*state).primitive_type);
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut offset: *const core::ffi::c_void = core::ptr::null();

    if (*gl_info).supported[ARB_DRAW_INDIRECT as usize] == 0 {
        return;
    }

    ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(GL_DRAW_INDIRECT_BUFFER, (*bo_gl).id);

    offset = ((*bo_gl).b.buffer_offset as *const u8).offset((*parameters).offset as isize)
        as *const core::ffi::c_void;
    if idx_size != 0 {
        let idx_type: GLenum = if idx_size == 2 {
            GL_UNSIGNED_SHORT
        } else {
            GL_UNSIGNED_INT
        };
        if (*state).index_offset != 0 {
            FIXME!("Ignoring index offset %u.\n", (*state).index_offset);
        }
        ((*gl_info).gl_ops.ext.p_glDrawElementsIndirect.unwrap_unchecked())(gl_primitive_type, idx_type, offset);
    } else {
        ((*gl_info).gl_ops.ext.p_glDrawArraysIndirect.unwrap_unchecked())(gl_primitive_type, offset);
    }

    ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(GL_DRAW_INDIRECT_BUFFER, 0);
    wined3d_context_gl_reference_bo(context_gl, bo_gl);

    checkGLcall!(gl_info, b"draw indirect\0".as_ptr() as *const core::ffi::c_char);
}

unsafe extern "C" fn gl_tfb_primitive_type_from_d3d(primitive_type: wined3d_primitive_type) -> GLenum {
    let gl_primitive_type: GLenum = gl_primitive_type_from_d3d(primitive_type);
    match gl_primitive_type {
        GL_POINTS => {
            return GL_POINTS;
        }

        GL_LINE_STRIP | GL_LINE_STRIP_ADJACENCY | GL_LINES_ADJACENCY | GL_LINES => {
            return GL_LINES;
        }

        GL_TRIANGLE_FAN | GL_TRIANGLE_STRIP | GL_TRIANGLE_STRIP_ADJACENCY | GL_TRIANGLES_ADJACENCY | GL_TRIANGLES => {
            return GL_TRIANGLES;
        }

        _ => {
            return gl_primitive_type;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn draw_primitive(mut device: *mut wined3d_device, mut state: *const wined3d_state, mut parameters: *const wined3d_draw_parameters) {
    let mut fb: *const wined3d_fb_state = &raw const (*state).fb;
    let mut stream_info: *const wined3d_stream_info = ::core::ptr::null::<wined3d_stream_info>();
    let mut dsv: *mut wined3d_rendertarget_view =
        ::core::ptr::null_mut::<wined3d_rendertarget_view>();
    let mut rtv: *mut wined3d_rendertarget_view =
        ::core::ptr::null_mut::<wined3d_rendertarget_view>();
    let mut gl_info: *const wined3d_gl_info = ::core::ptr::null::<wined3d_gl_info>();
    let mut context_gl: *mut wined3d_context_gl = ::core::ptr::null_mut::<wined3d_context_gl>();
    let mut context: *mut wined3d_context = ::core::ptr::null_mut::<wined3d_context>();
    let mut rasterizer_discard: bool = false;
    let mut i: ::core::ffi::c_uint = 0;
    let mut idx_size: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut idx_data: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();

    if (*parameters).indirect == 0 && (*parameters).u.direct.index_count == 0 {
        return;
    }

    if (*parameters).indirect == 0 {
        TRACE!(
            "base_vertex_idx %d, start_idx %u, index_count %u, start_instance %u, instance_count %u.\n",
            (*parameters).u.direct.base_vertex_idx,
            (*parameters).u.direct.start_idx,
            (*parameters).u.direct.index_count,
            (*parameters).u.direct.start_instance,
            (*parameters).u.direct.instance_count,
        );
    }

    rtv = (*fb).render_targets[0 as usize];
    if rtv.is_null() {
        rtv = (*fb).depth_stencil;
    }

    if !rtv.is_null() && (*(*rtv).resource).type_ as ::core::ffi::c_uint == WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint {
        FIXME!("Buffer render targets not implemented.\n");
        return;
    }

    if !rtv.is_null() {
        context = context_acquire(
            device,
            wined3d_texture_from_resource((*rtv).resource),
            (*rtv).sub_resource_idx,
        );
    } else {
        context = context_acquire(
            device,
            ::core::ptr::null_mut::<wined3d_texture>(),
            0 as ::core::ffi::c_uint,
        );
    }
    context_gl = wined3d_context_gl(context);
    if (*context_gl).valid() == 0 {
        context_release(context);
        WARN!("Invalid context, skipping draw.\n");
        return;
    }
    gl_info = (*context_gl).gl_info;

    if use_transform_feedback(state) == 0 {
        wined3d_context_gl_pause_transform_feedback(context_gl, (TRUE) as BOOL);
    }

    i = 0 as ::core::ffi::c_uint;
    while i < (*gl_info).limits.buffers {
        rtv = (*fb).render_targets[i as usize];
        if !(rtv.is_null() || (*(*rtv).format).id as ::core::ffi::c_uint == WINED3DFMT_NULL as ::core::ffi::c_uint) {
            if wined3d_blend_state_get_writemask((*state).blend_state, i) != 0 {
                wined3d_rendertarget_view_load_location(rtv, context, (*(*rtv).resource).draw_binding as uint32_t);
                wined3d_rendertarget_view_invalidate_location(rtv, !((*(*rtv).resource).draw_binding as ::core::ffi::c_int) as uint32_t);
            } else {
                wined3d_rendertarget_view_prepare_location(rtv, context, (*(*rtv).resource).draw_binding as uint32_t);
            }
        }
        i = i.wrapping_add(1);
    }

    dsv = (*fb).depth_stencil;
    if !dsv.is_null() {
        let mut location: uint32_t = (*(*dsv).resource).draw_binding as uint32_t;

        if wined3d_state_uses_depth_buffer(state) != false {
            wined3d_rendertarget_view_load_location(dsv, context, location);
        } else {
            wined3d_rendertarget_view_prepare_location(dsv, context, location);
        }
    }

    if (*parameters).indirect != 0 {
        wined3d_buffer_load((*parameters).u.indirect.buffer, context, state);
    }

    if context_apply_draw_state(context, device, state, (*parameters).indexed) == 0 {
        context_release(context);
        WARN!("Unable to apply draw state, skipping draw.\n");
        return;
    }

    if !dsv.is_null() && ((*state).depth_stencil_state.is_null() || (*(*state).depth_stencil_state).writes_ds as ::core::ffi::c_int != 0) {
        let mut location_0: uint32_t = (*(*dsv).resource).draw_binding as uint32_t;

        wined3d_rendertarget_view_validate_location(dsv, location_0);
        wined3d_rendertarget_view_invalidate_location(dsv, !location_0);
    }

    stream_info = &raw mut (*context).stream_info;

    if (*parameters).indexed != 0 {
        let mut index_buffer: *mut wined3d_buffer = (*state).index_buffer;
        let mut bo: *mut wined3d_bo = (*index_buffer).buffer_object;

        if bo.is_null() || (*stream_info).all_vbo() == 0 {
            idx_data = (*index_buffer).resource.heap_memory;
        } else {
            idx_data = (*bo).buffer_offset as *mut ::core::ffi::c_void;
        }
        idx_data = (idx_data as *const BYTE).offset((*state).index_offset as isize) as *const ::core::ffi::c_void;

        if (*state).index_format as ::core::ffi::c_uint == WINED3DFMT_R16_UINT as ::core::ffi::c_uint {
            idx_size = 2 as ::core::ffi::c_uint;
        } else {
            idx_size = 4 as ::core::ffi::c_uint;
        }
    }

    if use_transform_feedback(state) != 0 {
        let mut shader: *const wined3d_shader = (*state).shader[WINED3D_SHADER_TYPE_GEOMETRY as usize];

        if is_rasterization_disabled(shader) != 0 {
            glEnable(GL_RASTERIZER_DISCARD as GLenum);
            checkGLcall!(gl_info, "enable rasterizer discard");
            rasterizer_discard = true;
        }

        if (*context).transform_feedback_paused() != 0 {
            ((*gl_info).gl_ops.ext.p_glResumeTransformFeedback.unwrap_unchecked())();
            checkGLcall!(gl_info, "glResumeTransformFeedback");
            (*context).set_transform_feedback_paused(0 as DWORD);
        } else if (*context).transform_feedback_active() == 0 {
            let mut primitive_type: wined3d_primitive_type = (if (*shader).u.gs.output_type as ::core::ffi::c_uint != 0 {
                (*shader).u.gs.output_type as ::core::ffi::c_uint
            } else {
                (*state).primitive_type as ::core::ffi::c_uint
            }) as wined3d_primitive_type;
            let mut mode: GLenum = gl_tfb_primitive_type_from_d3d(primitive_type);
            ((*gl_info).gl_ops.ext.p_glBeginTransformFeedback.unwrap_unchecked())(mode);
            checkGLcall!(gl_info, "glBeginTransformFeedback");
            (*context).set_transform_feedback_active(1 as DWORD);
        }
    }

    if (*state).primitive_type as ::core::ffi::c_uint == WINED3D_PT_PATCH as ::core::ffi::c_uint {
        ((*gl_info).gl_ops.ext.p_glPatchParameteri.unwrap_unchecked())(
            GL_PATCH_VERTICES as GLenum,
            (*state).patch_vertex_count as GLint,
        );
        checkGLcall!(gl_info, "glPatchParameteri");
    }

    if (*context).uses_fbo_attached_resources() != 0 {
        static mut fixme_once: ::core::ffi::c_uint = 0;

        if (*gl_info).supported[ARB_TEXTURE_BARRIER as usize] != 0 {
            ((*gl_info).gl_ops.ext.p_glTextureBarrier.unwrap_unchecked())();
        } else if (*gl_info).supported[NV_TEXTURE_BARRIER as usize] != 0 {
            ((*gl_info).gl_ops.ext.p_glTextureBarrierNV.unwrap_unchecked())();
        } else {
            let fresh15 = fixme_once;
            fixme_once = fixme_once.wrapping_add(1);
            if fresh15 == 0 {
                FIXME!("Sampling attached render targets is not supported.\n");
            }

            WARN!("Sampling attached render targets is not supported, skipping draw.\n");
            context_release(context);
            return;
        }
        checkGLcall!(gl_info, "glTextureBarrier");
    }

    if (*parameters).indirect != 0 {
        wined3d_context_gl_draw_indirect(
            context_gl,
            state,
            &raw const (*parameters).u.indirect,
            idx_size,
        );
    } else {
        wined3d_context_gl_draw_primitive_arrays(
            context_gl,
            state,
            idx_data,
            idx_size,
            (*parameters).u.direct.base_vertex_idx,
            (*parameters).u.direct.start_idx,
            (*parameters).u.direct.index_count,
            (*parameters).u.direct.start_instance,
            (*parameters).u.direct.instance_count,
        );
    }

    if (*context).uses_uavs() != 0 {
        ((*gl_info).gl_ops.ext.p_glMemoryBarrier.unwrap_unchecked())(GL_ALL_BARRIER_BITS as GLenum);
        checkGLcall!(gl_info, "glMemoryBarrier");
    }

    if rasterizer_discard {
        glDisable(GL_RASTERIZER_DISCARD as GLenum);
        checkGLcall!(gl_info, "disable rasterizer discard");
    }

    context_release(context);

    TRACE!("Draw completed.\n");
}

unsafe extern "C" fn get_vertex_attrib_pointer(
    element: *const wined3d_stream_info_element,
    state: *const wined3d_state,
) -> *const core::ffi::c_void {
    let mut offset: *const u8 = (*element).data.addr.offset(
        ((*state).load_base_vertex_index as core::ffi::c_uint).wrapping_mul((*element).stride)
            as isize,
    );
    if !(*element).data.buffer_object.is_null() {
        offset = offset.offset((*(*element).data.buffer_object).buffer_offset as isize);
    }
    return offset as *const core::ffi::c_void;
}

unsafe extern "C" fn wined3d_context_gl_unload_numbered_array(
    context_gl: *mut wined3d_context_gl,
    i: ::core::ffi::c_uint,
) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;

    ((*gl_info).gl_ops.ext.p_glDisableVertexAttribArray.unwrap_unchecked())(i);
    checkGLcall!(gl_info, b"glDisableVertexAttribArray\0".as_ptr() as *const ::core::ffi::c_char);
    if (*gl_info).supported[ARB_INSTANCED_ARRAYS as usize] != 0 {
        ((*gl_info).gl_ops.ext.p_glVertexAttribDivisor.unwrap_unchecked())(i, 0);
    }

    (*context_gl).c.numbered_array_mask &= !(1 << i);
}

unsafe extern "C" fn wined3d_context_gl_unload_numbered_arrays(context_gl: *mut wined3d_context_gl) {
    let mut mask: u32 = (*context_gl).c.numbered_array_mask as u32;
    let mut i: core::ffi::c_uint = 0;
    while mask != 0 {
        i = wined3d_bit_scan(core::ptr::addr_of_mut!(mask)) as core::ffi::c_uint;
        wined3d_context_gl_unload_numbered_array(context_gl, i);
    }
}

unsafe extern "C" fn wined3d_context_gl_load_numbered_arrays(
    context_gl: *mut wined3d_context_gl,
    stream_info: *const wined3d_stream_info,
    state: *const wined3d_state,
) {
    let context: *mut wined3d_context = &raw mut (*context_gl).c;
    let vs: *const wined3d_shader = (*state).shader[WINED3D_SHADER_TYPE_VERTEX as usize] as *const wined3d_shader;
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut current_bo: GLuint = 0;
    let mut bo: GLuint = 0;
    let mut i: ::core::ffi::c_uint = 0;

    /* Default to no instancing. */
    current_bo = if (*gl_info).supported[ARB_VERTEX_BUFFER_OBJECT as usize] != 0 {
        !0
    } else {
        0
    };

    if ((*stream_info).use_map & !wined3d_mask_from_size((*gl_info).limits.vertex_attribs)) != 0 {
        static mut once: ::core::ffi::c_uint = 0;
        let fresh = once;
        once = once.wrapping_add(1);
        if fresh == 0 {
            FIXME!("More than the supported %u vertex attributes are in use.\n", (*gl_info).limits.vertex_attribs);
        }
    }

    i = 0;
    while i < (*gl_info).limits.vertex_attribs {
        'iter: {
            let element: *const wined3d_stream_info_element =
                (&raw const (*stream_info).elements as *const wined3d_stream_info_element).offset(i as isize);
            let mut stream: *const wined3d_stream_state = core::ptr::null::<wined3d_stream_state>();
            let mut format_gl: *const wined3d_format_gl = core::ptr::null::<wined3d_format_gl>();

            if ((*stream_info).use_map & ((1) << i)) == 0 {
                if ((*context).numbered_array_mask & ((1) << i)) != 0 {
                    wined3d_context_gl_unload_numbered_array(context_gl, i);
                }
                if ((*context_gl).default_attrib_value_set & ((1) << i)) == 0 {
                    ((*gl_info).gl_ops.ext.p_glVertexAttrib4f.unwrap_unchecked())(i, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
                }
                (*context_gl).default_attrib_value_set |= (1) << i;
                break 'iter;
            }

            format_gl = wined3d_format_gl((*element).format as *const wined3d_format);
            stream = (&raw const (*state).streams as *const wined3d_stream_state)
                .offset((*element).stream_idx as isize);

            if (*gl_info).supported[ARB_INSTANCED_ARRAYS as usize] != 0 {
                let mut divisor: ::core::ffi::c_uint = 0;

                if (*element).instanced {
                    divisor = if (*element).divisor != 0 {
                        (*element).divisor
                    } else {
                        UINT_MAX
                    };
                }
                ((*gl_info).gl_ops.ext.p_glVertexAttribDivisor.unwrap_unchecked())(i, divisor);
            } else if (*element).divisor != 0 {
                /* Unload instanced arrays, they will be loaded using immediate
                 * mode instead. */
                if ((*context).numbered_array_mask & ((1) << i)) != 0 {
                    wined3d_context_gl_unload_numbered_array(context_gl, i);
                }
                (*context_gl).default_attrib_value_set &= !((1) << i);
                break 'iter;
            }

            if (*element).stride != 0 {
                let offset: *const ::core::ffi::c_void = get_vertex_attrib_pointer(element, state);
                let format_attrs: ::core::ffi::c_uint = (*format_gl).f.attrs;

                bo = wined3d_bo_gl_id((*element).data.buffer_object);
                if current_bo != bo {
                    ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(GL_ARRAY_BUFFER, bo);
                    checkGLcall!(gl_info, b"glBindBuffer\0".as_ptr() as *const ::core::ffi::c_char);
                    current_bo = bo;
                }
                if bo != 0 {
                    wined3d_buffer_validate_user((*stream).buffer as *mut wined3d_buffer);
                }

                /* Use the VBO to find out if a vertex buffer exists, not the vb
                 * pointer. vb can point to a user pointer data blob. In that case
                 * current_bo will be 0. If there is a vertex buffer but no vbo we
                 * won't be load converted attributes anyway. */
                if !vs.is_null()
                    && (((*vs).reg_maps.shader_version.major as ::core::ffi::c_int) >= 4)
                    && ((format_attrs & (WINED3D_FORMAT_ATTR_INTEGER)) != 0)
                {
                    ((*gl_info).gl_ops.ext.p_glVertexAttribIPointer.unwrap_unchecked())(
                        i,
                        (*format_gl).vtx_format,
                        (*format_gl).vtx_type,
                        (*element).stride as GLsizei,
                        offset,
                    );
                } else {
                    ((*gl_info).gl_ops.ext.p_glVertexAttribPointer.unwrap_unchecked())(
                        i,
                        (*format_gl).vtx_format,
                        (*format_gl).vtx_type,
                        ((format_attrs & (WINED3D_FORMAT_ATTR_NORMALISED)) != 0) as ::core::ffi::c_int as GLboolean,
                        (*element).stride as GLsizei,
                        offset,
                    );
                }

                if ((*context).numbered_array_mask & ((1) << i)) == 0 {
                    ((*gl_info).gl_ops.ext.p_glEnableVertexAttribArray.unwrap_unchecked())(i);
                    (*context).numbered_array_mask |= (1) << i;
                }
            } else {
                /* Stride = 0 means always the same values.
                 * glVertexAttribPointer() doesn't do that. Instead disable the
                 * pointer and set up the attribute statically. But we have to
                 * figure out the system memory address. */
                let mut ptr: *const BYTE = (*element).data.addr;
                if !(*element).data.buffer_object.is_null() {
                    ptr = ptr.offset(
                        wined3d_buffer_load_sysmem((*stream).buffer as *mut wined3d_buffer, context) as usize as isize,
                    );
                }

                if ((*context).numbered_array_mask & ((1) << i)) != 0 {
                    wined3d_context_gl_unload_numbered_array(context_gl, i);
                }

                match (*format_gl).f.id as ::core::ffi::c_uint {
                    WINED3DFMT_R32_FLOAT => {
                        ((*gl_info).gl_ops.ext.p_glVertexAttrib1fv.unwrap_unchecked())(i, ptr as *const GLfloat);
                    }
                    WINED3DFMT_R32G32_FLOAT => {
                        ((*gl_info).gl_ops.ext.p_glVertexAttrib2fv.unwrap_unchecked())(i, ptr as *const GLfloat);
                    }
                    WINED3DFMT_R32G32B32_FLOAT => {
                        ((*gl_info).gl_ops.ext.p_glVertexAttrib3fv.unwrap_unchecked())(i, ptr as *const GLfloat);
                    }
                    WINED3DFMT_R32G32B32A32_FLOAT => {
                        ((*gl_info).gl_ops.ext.p_glVertexAttrib4fv.unwrap_unchecked())(i, ptr as *const GLfloat);
                    }
                    WINED3DFMT_R8G8B8A8_UINT => {
                        ((*gl_info).gl_ops.ext.p_glVertexAttrib4ubv.unwrap_unchecked())(i, ptr);
                    }
                    WINED3DFMT_B8G8R8A8_UNORM => {
                        if (*gl_info).supported[ARB_VERTEX_ARRAY_BGRA as usize] != 0 {
                            let src: *const DWORD = ptr as *const DWORD;
                            let mut c: DWORD = *src & 0xff00ff00;
                            c |= (*src & 0xff0000) >> 16;
                            c |= (*src & 0xff) << 16;
                            ((*gl_info).gl_ops.ext.p_glVertexAttrib4Nubv.unwrap_unchecked())(i, &raw mut c as *mut GLubyte);
                        } else {
                            /* else fallthrough */
                            ((*gl_info).gl_ops.ext.p_glVertexAttrib4Nubv.unwrap_unchecked())(i, ptr);
                        }
                    }
                    WINED3DFMT_R8G8B8A8_UNORM => {
                        ((*gl_info).gl_ops.ext.p_glVertexAttrib4Nubv.unwrap_unchecked())(i, ptr);
                    }
                    WINED3DFMT_R16G16_SINT => {
                        ((*gl_info).gl_ops.ext.p_glVertexAttrib2sv.unwrap_unchecked())(i, ptr as *const GLshort);
                    }
                    WINED3DFMT_R16G16B16A16_SINT => {
                        ((*gl_info).gl_ops.ext.p_glVertexAttrib4sv.unwrap_unchecked())(i, ptr as *const GLshort);
                    }
                    WINED3DFMT_R16G16_SNORM => {
                        let s: [GLshort; 4 as usize] = [
                            *(ptr as *const GLshort).offset(0 as isize),
                            *(ptr as *const GLshort).offset(1 as isize),
                            0 as GLshort,
                            1 as GLshort,
                        ];
                        ((*gl_info).gl_ops.ext.p_glVertexAttrib4Nsv.unwrap_unchecked())(i, &raw const s as *const GLshort);
                    }
                    WINED3DFMT_R16G16_UNORM => {
                        let s: [GLushort; 4 as usize] = [
                            *(ptr as *const GLushort).offset(0 as isize),
                            *(ptr as *const GLushort).offset(1 as isize),
                            0 as GLushort,
                            1 as GLushort,
                        ];
                        ((*gl_info).gl_ops.ext.p_glVertexAttrib4Nusv.unwrap_unchecked())(i, &raw const s as *const GLushort);
                    }
                    WINED3DFMT_R16G16B16A16_SNORM => {
                        ((*gl_info).gl_ops.ext.p_glVertexAttrib4Nsv.unwrap_unchecked())(i, ptr as *const GLshort);
                    }
                    WINED3DFMT_R16G16B16A16_UNORM => {
                        ((*gl_info).gl_ops.ext.p_glVertexAttrib4Nusv.unwrap_unchecked())(i, ptr as *const GLushort);
                    }
                    WINED3DFMT_R10G10B10X2_UINT => {
                        FIXME!("Unsure about WINED3DDECLTYPE_UDEC3.\n");
                        /*glVertexAttrib3usvARB(i, (const GLushort *)ptr); Does not exist */
                    }
                    WINED3DFMT_R10G10B10X2_SNORM => {
                        FIXME!("Unsure about WINED3DDECLTYPE_DEC3N.\n");
                        /*glVertexAttrib3NusvARB(i, (const GLushort *)ptr); Does not exist */
                    }
                    WINED3DFMT_R16G16_FLOAT => {
                        if (*gl_info).supported[NV_HALF_FLOAT as usize] != 0
                            && (*gl_info).supported[NV_VERTEX_PROGRAM as usize] != 0
                        {
                            /* Not supported by GL_ARB_half_float_vertex. */
                            ((*gl_info).gl_ops.ext.p_glVertexAttrib2hvNV.unwrap_unchecked())(i, ptr as *const GLhalfNV);
                        } else {
                            let x: ::core::ffi::c_float =
                                float_16_to_32((ptr as *const ::core::ffi::c_ushort).offset(0 as isize));
                            let y: ::core::ffi::c_float =
                                float_16_to_32((ptr as *const ::core::ffi::c_ushort).offset(1 as isize));
                            ((*gl_info).gl_ops.ext.p_glVertexAttrib2f.unwrap_unchecked())(i, x as GLfloat, y as GLfloat);
                        }
                    }
                    WINED3DFMT_R16G16B16A16_FLOAT => {
                        if (*gl_info).supported[NV_HALF_FLOAT as usize] != 0
                            && (*gl_info).supported[NV_VERTEX_PROGRAM as usize] != 0
                        {
                            /* Not supported by GL_ARB_half_float_vertex. */
                            ((*gl_info).gl_ops.ext.p_glVertexAttrib4hvNV.unwrap_unchecked())(i, ptr as *const GLhalfNV);
                        } else {
                            let x: ::core::ffi::c_float =
                                float_16_to_32((ptr as *const ::core::ffi::c_ushort).offset(0 as isize));
                            let y: ::core::ffi::c_float =
                                float_16_to_32((ptr as *const ::core::ffi::c_ushort).offset(1 as isize));
                            let z: ::core::ffi::c_float =
                                float_16_to_32((ptr as *const ::core::ffi::c_ushort).offset(2 as isize));
                            let w: ::core::ffi::c_float =
                                float_16_to_32((ptr as *const ::core::ffi::c_ushort).offset(3 as isize));
                            ((*gl_info).gl_ops.ext.p_glVertexAttrib4f.unwrap_unchecked())(i, x as GLfloat, y as GLfloat, z as GLfloat, w as GLfloat);
                        }
                    }
                    _ => {
                        ERR!("Unexpected declaration in stride 0 attributes.\n");
                    }
                }
                (*context_gl).default_attrib_value_set &= !((1) << i);
            }
        }
        i = i.wrapping_add(1);
    }
    checkGLcall!(gl_info, b"Loading numbered arrays\0".as_ptr() as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_update_stream_sources(context_gl: *mut wined3d_context_gl, state: *const wined3d_state) {
    wined3d_context_gl_load_numbered_arrays(
        context_gl,
        core::ptr::addr_of_mut!((*context_gl).c.stream_info),
        state,
    );
}

unsafe extern "C" fn apply_texture_blit_state(
    mut gl_info: *const wined3d_gl_info,
    mut texture: *mut gl_texture,
    mut target: GLenum,
    mut level: ::core::ffi::c_uint,
    mut filter: wined3d_texture_filter_type,
) {
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        target,
        GL_TEXTURE_MAG_FILTER as GLenum,
        wined3d_gl_mag_filter(filter) as GLint,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        target,
        GL_TEXTURE_MIN_FILTER as GLenum,
        wined3d_gl_min_mip_filter(filter, WINED3D_TEXF_NONE) as GLint,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        target,
        GL_TEXTURE_WRAP_S as GLenum,
        (GL_CLAMP_TO_EDGE as i32),
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        target,
        GL_TEXTURE_WRAP_T as GLenum,
        (GL_CLAMP_TO_EDGE as i32),
    );
    if (*gl_info).supported[EXT_TEXTURE_SRGB_DECODE as ::core::ffi::c_int as usize] != 0 {
        (*gl_info)
            .gl_ops
            .gl
            .p_glTexParameteri
            .unwrap_unchecked()(
            target,
            GL_TEXTURE_SRGB_DECODE_EXT as GLenum,
            (GL_SKIP_DECODE_EXT as i32),
        );
    }
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        target,
        GL_TEXTURE_BASE_LEVEL as GLenum,
        level as GLint,
    );

    /* We changed the filtering settings on the texture. Make sure they get
     * reset on subsequent draws. */
    (*texture).sampler_desc.mag_filter = WINED3D_TEXF_POINT;
    (*texture).sampler_desc.min_filter = WINED3D_TEXF_POINT;
    (*texture).sampler_desc.mip_filter = WINED3D_TEXF_NONE;
    (*texture).sampler_desc.address_u = WINED3D_TADDRESS_CLAMP;
    (*texture).sampler_desc.address_v = WINED3D_TADDRESS_CLAMP;
    (*texture).sampler_desc.srgb_decode = (FALSE as i32);
    (*texture).sampler_desc.mip_base_level = level;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_gl_draw_shaded_quad(context_gl: *mut wined3d_context_gl, texture_gl: *mut wined3d_texture_gl, sub_resource_idx: ::core::ffi::c_uint, src_rect: *const RECT, dst_rect: *const RECT, filter: wined3d_texture_filter_type) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut info: wined3d_blt_info = core::mem::zeroed();
    let mut level: ::core::ffi::c_uint = 0;
    let mut w: ::core::ffi::c_uint = 0;
    let mut h: ::core::ffi::c_uint = 0;
    let mut i: ::core::ffi::c_uint = 0;
    let mut dst_size: SIZE = core::mem::zeroed();
    let mut quad: [blit_vertex; 4] = core::mem::zeroed();

    texture2d_get_blt_info(texture_gl, sub_resource_idx, src_rect, core::ptr::addr_of_mut!(info));

    level = sub_resource_idx % (*texture_gl).t.level_count;
    wined3d_context_gl_bind_texture(context_gl, info.bind_target, (*texture_gl).texture_rgb.name);
    apply_texture_blit_state(gl_info, core::ptr::addr_of_mut!((*texture_gl).texture_rgb), info.bind_target, level, filter);
    (*gl_info).gl_ops.gl.p_glTexParameteri.unwrap_unchecked()(
        info.bind_target,
        GL_TEXTURE_MAX_LEVEL as GLenum,
        level as GLint,
    );

    wined3d_context_gl_pause_transform_feedback(context_gl, (FALSE) as BOOL);

    wined3d_context_gl_get_rt_size(context_gl, core::ptr::addr_of_mut!(dst_size));
    w = dst_size.cx as ::core::ffi::c_uint;
    h = dst_size.cy as ::core::ffi::c_uint;

    quad[0].x = (*dst_rect).left as ::core::ffi::c_float * 2.0f32 / w as ::core::ffi::c_float - 1.0f32;
    quad[0].y = (*dst_rect).top as ::core::ffi::c_float * 2.0f32 / h as ::core::ffi::c_float - 1.0f32;
    quad[0].texcoord = info.texcoords[0];

    quad[1].x = (*dst_rect).right as ::core::ffi::c_float * 2.0f32 / w as ::core::ffi::c_float - 1.0f32;
    quad[1].y = (*dst_rect).top as ::core::ffi::c_float * 2.0f32 / h as ::core::ffi::c_float - 1.0f32;
    quad[1].texcoord = info.texcoords[1];

    quad[2].x = (*dst_rect).left as ::core::ffi::c_float * 2.0f32 / w as ::core::ffi::c_float - 1.0f32;
    quad[2].y = (*dst_rect).bottom as ::core::ffi::c_float * 2.0f32 / h as ::core::ffi::c_float - 1.0f32;
    quad[2].texcoord = info.texcoords[2];

    quad[3].x = (*dst_rect).right as ::core::ffi::c_float * 2.0f32 / w as ::core::ffi::c_float - 1.0f32;
    quad[3].y = (*dst_rect).bottom as ::core::ffi::c_float * 2.0f32 / h as ::core::ffi::c_float - 1.0f32;
    quad[3].texcoord = info.texcoords[3];

    /* Draw a quad. */
    if (*gl_info).supported[(ARB_VERTEX_BUFFER_OBJECT) as usize] != 0 {
        if (*context_gl).blit_vbo == 0 {
            ((*gl_info).gl_ops.ext.p_glGenBuffers.unwrap_unchecked())(1, core::ptr::addr_of_mut!((*context_gl).blit_vbo));
        }
        ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(GL_ARRAY_BUFFER, (*context_gl).blit_vbo);

        wined3d_context_gl_unload_numbered_arrays(context_gl);

        ((*gl_info).gl_ops.ext.p_glBufferData.unwrap_unchecked())(
            GL_ARRAY_BUFFER,
            core::mem::size_of::<[blit_vertex; 4]>() as GLsizeiptr,
            core::ptr::addr_of_mut!(quad) as *mut blit_vertex as *const ::core::ffi::c_void,
            GL_STREAM_DRAW,
        );
        ((*gl_info).gl_ops.ext.p_glVertexAttribPointer.unwrap_unchecked())(
            0,
            2,
            GL_FLOAT,
            (FALSE as u8),
            (core::mem::size_of::<blit_vertex>()) as i32,
            core::ptr::null::<::core::ffi::c_void>(),
        );
        ((*gl_info).gl_ops.ext.p_glVertexAttribPointer.unwrap_unchecked())(
            1,
            3,
            GL_FLOAT,
            (FALSE as u8),
            (core::mem::size_of::<blit_vertex>()) as i32,
            FIELD_OFFSET!(blit_vertex, texcoord) as *mut ::core::ffi::c_void,
        );

        ((*gl_info).gl_ops.ext.p_glEnableVertexAttribArray.unwrap_unchecked())(0);
        ((*gl_info).gl_ops.ext.p_glEnableVertexAttribArray.unwrap_unchecked())(1);

        (*gl_info).gl_ops.gl.p_glDrawArrays.unwrap_unchecked()(
            GL_TRIANGLE_STRIP,
            0,
            4,
        );

        ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(GL_ARRAY_BUFFER, 0);
        ((*gl_info).gl_ops.ext.p_glDisableVertexAttribArray.unwrap_unchecked())(1);
        ((*gl_info).gl_ops.ext.p_glDisableVertexAttribArray.unwrap_unchecked())(0);
    } else {
        (*gl_info).gl_ops.gl.p_glBegin.unwrap_unchecked()(GL_TRIANGLE_STRIP as GLenum);

        i = 0;
        while (i as usize) < ARRAY_SIZE!(quad) as usize {
            ((*gl_info).gl_ops.ext.p_glVertexAttrib3fv.unwrap_unchecked())(
                1,
                core::ptr::addr_of_mut!((*(core::ptr::addr_of_mut!(quad) as *mut blit_vertex).offset(i as isize)).texcoord.x),
            );
            ((*gl_info).gl_ops.ext.p_glVertexAttrib2fv.unwrap_unchecked())(
                0,
                core::ptr::addr_of_mut!((*(core::ptr::addr_of_mut!(quad) as *mut blit_vertex).offset(i as isize)).x),
            );
            i += 1;
        }

        (*gl_info).gl_ops.gl.p_glEnd.unwrap_unchecked()();
    }
    checkGLcall!(gl_info, "draw");

    (*gl_info).gl_ops.gl.p_glTexParameteri.unwrap_unchecked()(
        info.bind_target,
        GL_TEXTURE_MAX_LEVEL as GLenum,
        ((*texture_gl).t.level_count - 1) as GLint,
    );
    wined3d_context_gl_bind_texture(context_gl, info.bind_target, 0);
}
