use crate::d3dwine::context_gl::wined3d_context_gl_get_current;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut old_multitex_glGetIntegerv: Option<unsafe extern "C" fn(GLenum, *mut GLint) -> ()> =
    None;

static mut old_multitex_glGetFloatv: Option<unsafe extern "C" fn(GLenum, *mut GLfloat) -> ()> =
    None;

static mut old_multitex_glGetDoublev: Option<unsafe extern "C" fn(GLenum, *mut GLdouble) -> ()> =
    None;

static mut old_fogcoord_glEnable: Option<unsafe extern "C" fn(GLenum) -> ()> = None;

static mut old_fogcoord_glDisable: Option<unsafe extern "C" fn(GLenum) -> ()> = None;

static mut old_fogcoord_glFogi: Option<unsafe extern "C" fn(GLenum, GLint) -> ()> = None;

static mut old_fogcoord_glFogiv: Option<unsafe extern "C" fn(GLenum, *const GLint) -> ()> = None;

static mut old_fogcoord_glFogf: Option<unsafe extern "C" fn(GLenum, GLfloat) -> ()> = None;

static mut old_fogcoord_glFogfv: Option<unsafe extern "C" fn(GLenum, *const GLfloat) -> ()> = None;

static mut old_fogcoord_glVertex4f: Option<
    unsafe extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat) -> (),
> = None;

static mut old_fogcoord_glColor4f: Option<
    unsafe extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat) -> (),
> = None;

// WINE_DEFAULT_DEBUG_CHANNEL(gl_compat) and WINE_DECLARE_DEBUG_CHANNEL(d3d_perf)
// are handled by the crate's DPRINT/ERR/WARN macros; no declaration emitted here.

unsafe extern "system" fn wine_glMultiTexCoord1fARB(mut target: GLenum, mut s: GLfloat) {
    if target != GL_TEXTURE0 as GLenum {
        ERR!("Texture unit > 0 used, but GL_ARB_multitexture is not supported.\n");
        return;
    }
    (*(*wined3d_context_gl_get_current()).gl_info)
        .gl_ops
        .gl
        .p_glTexCoord1f
        .unwrap_unchecked()(s);
}

unsafe extern "system" fn wine_glMultiTexCoord1fvARB(mut target: GLenum, mut v: *const GLfloat) {
    if target != GL_TEXTURE0 {
        ERR!("Texture unit > 0 used, but GL_ARB_multitexture is not supported.\n");
        return;
    }
    (*(*wined3d_context_gl_get_current()).gl_info).gl_ops.gl.p_glTexCoord1fv.unwrap_unchecked()(v);
}

unsafe extern "system" fn wine_glMultiTexCoord2fARB(mut target: GLenum, mut s: GLfloat, mut t: GLfloat) {
    if target != GL_TEXTURE0 {
        ERR!("Texture unit > 0 used, but GL_ARB_multitexture is not supported.\n");
        return;
    }
    (*(*wined3d_context_gl_get_current()).gl_info)
        .gl_ops
        .gl
        .p_glTexCoord2f
        .unwrap_unchecked()(s, t);
}

unsafe extern "system" fn wine_glMultiTexCoord2fvARB(target: GLenum, v: *const GLfloat) {
    if target != GL_TEXTURE0 {
        ERR!("Texture unit > 0 used, but GL_ARB_multitexture is not supported.\n");
        return;
    }
    ((*(*wined3d_context_gl_get_current()).gl_info).gl_ops.gl.p_glTexCoord2fv.unwrap_unchecked())(v);
}

unsafe extern "system" fn wine_glMultiTexCoord3fARB(
    mut target: GLenum,
    mut s: GLfloat,
    mut t: GLfloat,
    mut r: GLfloat,
) {
    if target != GL_TEXTURE0 {
        ERR!("Texture unit > 0 used, but GL_ARB_multitexture is not supported.\n");
        return;
    }
    (*(*wined3d_context_gl_get_current()).gl_info)
        .gl_ops
        .gl
        .p_glTexCoord3f
        .unwrap_unchecked()(s, t, r);
}

unsafe extern "system" fn wine_glMultiTexCoord3fvARB(mut target: GLenum, mut v: *const GLfloat) {
    if target != GL_TEXTURE0 {
        ERR!("Texture unit > 0 used, but GL_ARB_multitexture is not supported.\n");
        return;
    }
    (*(*wined3d_context_gl_get_current()).gl_info).gl_ops.gl.p_glTexCoord3fv.unwrap_unchecked()(v);
}

unsafe extern "system" fn wine_glMultiTexCoord4fARB(target: GLenum, s: GLfloat, t: GLfloat, r: GLfloat, q: GLfloat) {
    if target != GL_TEXTURE0 {
        ERR!("Texture unit > 0 used, but GL_ARB_multitexture is not supported.\n");
        return;
    }
    (*(*wined3d_context_gl_get_current()).gl_info).gl_ops.gl.p_glTexCoord4f.unwrap_unchecked()(s, t, r, q);
}

unsafe extern "system" fn wine_glMultiTexCoord4fvARB(mut target: GLenum, mut v: *const GLfloat) {
    if target != GL_TEXTURE0 as GLenum {
        ERR!("Texture unit > 0 used, but GL_ARB_multitexture is not supported.\n");
        return;
    }
    (*(*wined3d_context_gl_get_current()).gl_info)
        .gl_ops
        .gl
        .p_glTexCoord4fv
        .unwrap_unchecked()(v);
}

unsafe extern "system" fn wine_glMultiTexCoord2svARB(mut target: GLenum, mut v: *const GLshort) {
    if target != GL_TEXTURE0 {
        ERR!("Texture unit > 0 used, but GL_ARB_multitexture is not supported.\n");
        return;
    }
    (*(*wined3d_context_gl_get_current()).gl_info).gl_ops.gl.p_glTexCoord2sv.unwrap_unchecked()(v);
}

unsafe extern "system" fn wine_glMultiTexCoord4svARB(mut target: GLenum, mut v: *const GLshort) {
    if target != GL_TEXTURE0 {
        ERR!("Texture unit > 0 used, but GL_ARB_multitexture is not supported.\n");
        return;
    }
    (*(*wined3d_context_gl_get_current()).gl_info).gl_ops.gl.p_glTexCoord4sv.unwrap_unchecked()(v);
}

unsafe extern "system" fn wine_glActiveTexture(texture: GLenum) {
    if texture != GL_TEXTURE0 as GLenum {
        ERR!("Texture unit > 0 used, but GL_ARB_multitexture is not supported\n");
        return;
    }
}

unsafe extern "system" fn wine_glClientActiveTextureARB(mut texture: GLenum) {
    if texture != GL_TEXTURE0 as GLenum {
        ERR!("Texture unit > 0 used, but GL_ARB_multitexture is not supported\n");
        return;
    }
}

unsafe extern "system" fn wine_glGetIntegerv(mut pname: GLenum, mut params: *mut GLint) {
    match pname {
        GL_ACTIVE_TEXTURE => {
            *params = 0 as GLint;
        }
        GL_MAX_TEXTURE_UNITS_ARB => {
            *params = 1 as GLint;
        }
        _ => {
            (old_multitex_glGetIntegerv.unwrap_unchecked())(pname, params);
        }
    }
}

unsafe extern "system" fn wine_glGetFloatv(pname: GLenum, params: *mut GLfloat) {
    if pname == GL_ACTIVE_TEXTURE {
        *params = 0.0f32 as GLfloat;
    } else {
        (old_multitex_glGetFloatv.unwrap_unchecked())(pname, params);
    }
}

unsafe extern "system" fn wine_glGetDoublev(mut pname: GLenum, mut params: *mut GLdouble) {
    if pname == GL_ACTIVE_TEXTURE as GLenum {
        *params = 0.0f64 as GLdouble;
    } else {
        old_multitex_glGetDoublev.unwrap_unchecked()(pname, params);
    };
}

unsafe extern "system" fn wine_glEnable(cap: GLenum) {
    if cap == GL_FOG {
        let ctx: *mut wined3d_context_gl = wined3d_context_gl_get_current();

        (*ctx).set_fog_enabled((1) as _);
        if (*ctx).gl_fog_source != (GL_FRAGMENT_DEPTH_EXT as i32) {
            return;
        }
    }
    (old_fogcoord_glEnable.unwrap_unchecked())(cap);
}

unsafe extern "system" fn wine_glDisable(mut cap: GLenum) {
    if cap == GL_FOG {
        let mut ctx: *mut wined3d_context_gl = wined3d_context_gl_get_current();

        (*ctx).set_fog_enabled((0) as _);
        if (*ctx).gl_fog_source != (GL_FRAGMENT_DEPTH_EXT as i32) {
            return;
        }
    }
    (old_fogcoord_glDisable.unwrap_unchecked())(cap);
}

unsafe extern "system" fn wine_glFogi(pname: GLenum, param: GLint) {
    let ctx: *mut wined3d_context_gl = wined3d_context_gl_get_current();

    if pname == GL_FOG_COORDINATE_SOURCE_EXT {
        (*ctx).gl_fog_source = param;
        if param == (GL_FRAGMENT_DEPTH_EXT as GLint) {
            if (*ctx).fog_enabled() != 0 {
                (old_fogcoord_glEnable.unwrap_unchecked())(GL_FOG);
            }
        } else {
            WINE_WARN_!(__wine_dbch_d3d_perf,b"Fog coordinates activated, but not supported. Using slow emulation.\n\0".as_ptr() as *const i8);
            (old_fogcoord_glDisable.unwrap_unchecked())(GL_FOG);
        }
    } else {
        if pname == GL_FOG_START {
            (*ctx).fog_start = param as f32;
        } else if pname == GL_FOG_END {
            (*ctx).fog_end = param as f32;
        }
        (old_fogcoord_glFogi.unwrap_unchecked())(pname, param);
    }
}

unsafe extern "system" fn wine_glFogiv(pname: GLenum, param: *const GLint) {
    let mut ctx: *mut wined3d_context_gl = wined3d_context_gl_get_current();

    if pname == GL_FOG_COORDINATE_SOURCE_EXT as GLenum {
        (*ctx).gl_fog_source = *param;
        if *param == GL_FRAGMENT_DEPTH_EXT as GLint {
            if (*ctx).fog_enabled() != 0 {
                (old_fogcoord_glEnable.unwrap_unchecked())(GL_FOG as GLenum);
            }
        } else {
            WINE_WARN_!(__wine_dbch_d3d_perf, "Fog coordinates activated, but not supported. Using slow emulation.\n");
            (old_fogcoord_glDisable.unwrap_unchecked())(GL_FOG as GLenum);
        }
    } else {
        if pname == GL_FOG_START as GLenum {
            (*ctx).fog_start = *param as GLfloat;
        } else if pname == GL_FOG_END as GLenum {
            (*ctx).fog_end = *param as GLfloat;
        }
        (old_fogcoord_glFogiv.unwrap_unchecked())(pname, param);
    }
}

unsafe extern "system" fn wine_glFogf(mut pname: GLenum, mut param: GLfloat) {
    let mut ctx: *mut wined3d_context_gl = wined3d_context_gl_get_current();

    if pname == GL_FOG_COORDINATE_SOURCE_EXT as GLenum {
        (*ctx).gl_fog_source = param as GLint;
        if param == GL_FRAGMENT_DEPTH_EXT as GLfloat {
            if (*ctx).fog_enabled() != 0 {
                (old_fogcoord_glEnable.unwrap_unchecked())(GL_FOG as GLenum);
            }
        } else {
            WINE_WARN_!(__wine_dbch_d3d_perf,"Fog coordinates activated, but not supported. Using slow emulation.\n");
            (old_fogcoord_glDisable.unwrap_unchecked())(GL_FOG as GLenum);
        }
    } else {
        if pname == GL_FOG_START as GLenum {
            (*ctx).fog_start = param;
        } else if pname == GL_FOG_END as GLenum {
            (*ctx).fog_end = param;
        }
        (old_fogcoord_glFogf.unwrap_unchecked())(pname, param);
    }
}

unsafe extern "system" fn wine_glFogfv(pname: GLenum, param: *const GLfloat) {
    let ctx: *mut wined3d_context_gl = wined3d_context_gl_get_current();

    if pname == GL_FOG_COORDINATE_SOURCE_EXT as GLenum {
        (*ctx).gl_fog_source = *param as GLint;
        if *param == GL_FRAGMENT_DEPTH_EXT as GLfloat {
            if (*ctx).fog_enabled() != 0 {
                (old_fogcoord_glEnable.unwrap_unchecked())(GL_FOG as GLenum);
            }
        } else {
            WINE_WARN_!(__wine_dbch_d3d_perf, "Fog coordinates activated, but not supported. Using slow emulation.\n");
            (old_fogcoord_glDisable.unwrap_unchecked())(GL_FOG as GLenum);
        }
    } else {
        if pname == GL_FOG_COLOR as GLenum {
            (*ctx).fog_colour[0 as usize] = *param.offset(0 as isize);
            (*ctx).fog_colour[1 as usize] = *param.offset(1 as isize);
            (*ctx).fog_colour[2 as usize] = *param.offset(2 as isize);
            (*ctx).fog_colour[3 as usize] = *param.offset(3 as isize);
        } else if pname == GL_FOG_START as GLenum {
            (*ctx).fog_start = *param;
        } else if pname == GL_FOG_END as GLenum {
            (*ctx).fog_end = *param;
        }
        (old_fogcoord_glFogfv.unwrap_unchecked())(pname, param);
    }
}

unsafe extern "system" fn wine_glVertex4f(mut x: GLfloat, mut y: GLfloat, mut z: GLfloat, mut w: GLfloat) {
    let ctx_gl: *const wined3d_context_gl = wined3d_context_gl_get_current();

    /* This can be called from draw_test_quad() and at that point there is no
     * wined3d_context current. */
    if ctx_gl.is_null() {
        (old_fogcoord_glVertex4f.unwrap_unchecked())(x, y, z, w);
        return;
    }

    if (*ctx_gl).gl_fog_source == (GL_FOG_COORDINATE_EXT as i32) && ((*ctx_gl).fog_enabled() as i32) != 0 {
        let mut c: [GLfloat; 4] = [
            (*ctx_gl).colour[0],
            (*ctx_gl).colour[1],
            (*ctx_gl).colour[2],
            (*ctx_gl).colour[3],
        ];
        let mut i: GLfloat = 0.0;

        i = ((*ctx_gl).fog_end - (*ctx_gl).fog_coord_value) / ((*ctx_gl).fog_end - (*ctx_gl).fog_start);
        c[0] = i * c[0] + (1.0f32 - i) * (*ctx_gl).fog_colour[0];
        c[1] = i * c[1] + (1.0f32 - i) * (*ctx_gl).fog_colour[1];
        c[2] = i * c[2] + (1.0f32 - i) * (*ctx_gl).fog_colour[2];

        (old_fogcoord_glColor4f.unwrap_unchecked())(
            c[0],
            c[1],
            c[2],
            c[3],
        );
        (old_fogcoord_glVertex4f.unwrap_unchecked())(x, y, z, w);
    } else {
        (old_fogcoord_glVertex4f.unwrap_unchecked())(x, y, z, w);
    }
}

unsafe extern "system" fn wine_glVertex4fv(pos: *const GLfloat) {
    wine_glVertex4f(
        *pos.offset(0 as isize),
        *pos.offset(1 as isize),
        *pos.offset(2 as isize),
        *pos.offset(3 as isize),
    );
}

unsafe extern "system" fn wine_glVertex3f(x: GLfloat, y: GLfloat, z: GLfloat) {
    wine_glVertex4f(x, y, z, 1.0f32);
}

unsafe extern "system" fn wine_glVertex3fv(pos: *const GLfloat) {
    wine_glVertex4f(*pos.offset(0 as isize), *pos.offset(1 as isize), *pos.offset(2 as isize), 1.0f32);
}

unsafe extern "system" fn wine_glColor4f(r: GLfloat, g: GLfloat, b: GLfloat, a: GLfloat) {
    let ctx_gl: *mut wined3d_context_gl = wined3d_context_gl_get_current();

    /* This can be called from draw_test_quad() and at that point there is no
     * wined3d_context current. */
    if ctx_gl.is_null() {
        (old_fogcoord_glColor4f.unwrap_unchecked())(r, g, b, a);
        return;
    }

    (*ctx_gl).colour[0 as usize] = r;
    (*ctx_gl).colour[1 as usize] = g;
    (*ctx_gl).colour[2 as usize] = b;
    (*ctx_gl).colour[3 as usize] = a;
    (old_fogcoord_glColor4f.unwrap_unchecked())(r, g, b, a);
}

unsafe extern "system" fn wine_glColor4fv(c: *const GLfloat) {
    wine_glColor4f(
        *c.offset(0 as isize),
        *c.offset(1 as isize),
        *c.offset(2 as isize),
        *c.offset(3 as isize),
    );
}

unsafe extern "system" fn wine_glColor3f(r: GLfloat, g: GLfloat, b: GLfloat) {
    wine_glColor4f(r, g, b, 1.0f32);
}

unsafe extern "system" fn wine_glColor3fv(c: *const GLfloat) {
    wine_glColor4f(*c.offset(0 as isize), *c.offset(1 as isize), *c.offset(2 as isize), 1.0f32);
}

unsafe extern "system" fn wine_glColor4ub(mut r: GLubyte, mut g: GLubyte, mut b: GLubyte, mut a: GLubyte) {
    wine_glColor4f(
        r as GLfloat / 255.0f32,
        g as GLfloat / 255.0f32,
        b as GLfloat / 255.0f32,
        a as GLfloat / 255.0f32,
    );
}

unsafe extern "system" fn wine_glFogCoordfEXT(f: GLfloat) {
    let ctx: *mut wined3d_context_gl = wined3d_context_gl_get_current();

    (*ctx).fog_coord_value = f;
}

unsafe extern "system" fn wine_glFogCoorddEXT(f: GLdouble) {
    wine_glFogCoordfEXT(f as GLfloat);
}

unsafe extern "system" fn wine_glFogCoordfvEXT(f: *const GLfloat) {
    wine_glFogCoordfEXT(*f);
}

unsafe extern "system" fn wine_glFogCoorddvEXT(f: *const GLdouble) {
    wine_glFogCoordfEXT(*f as GLfloat);
}
#[no_mangle]
pub unsafe extern "C" fn install_gl_compat_wrapper(mut gl_info: *mut wined3d_gl_info, mut ext: wined3d_gl_extension) {
    match ext as ::core::ffi::c_uint {
        ARB_MULTITEXTURE => {
            if (*gl_info).supported[ARB_MULTITEXTURE as usize] != 0 {
                return;
            }
            if (*gl_info).gl_ops.ext.p_glActiveTexture
                == Some(wine_glActiveTexture)
            {
                WINE_FIXME!("ARB_multitexture emulation hooks already applied.\n");
                return;
            }
            WINE_TRACE!("Applying GL_ARB_multitexture emulation hooks.\n");
            (*gl_info).gl_ops.ext.p_glActiveTexture =
                Some(wine_glActiveTexture);
            (*gl_info).gl_ops.ext.p_glClientActiveTextureARB =
                Some(wine_glClientActiveTextureARB);
            (*gl_info).gl_ops.ext.p_glMultiTexCoord1fARB =
                Some(wine_glMultiTexCoord1fARB);
            (*gl_info).gl_ops.ext.p_glMultiTexCoord1fvARB = Some(wine_glMultiTexCoord1fvARB);
            (*gl_info).gl_ops.ext.p_glMultiTexCoord2fARB = Some(wine_glMultiTexCoord2fARB);
            (*gl_info).gl_ops.ext.p_glMultiTexCoord2fvARB = Some(wine_glMultiTexCoord2fvARB);
            (*gl_info).gl_ops.ext.p_glMultiTexCoord3fARB = Some(wine_glMultiTexCoord3fARB);
            (*gl_info).gl_ops.ext.p_glMultiTexCoord3fvARB = Some(wine_glMultiTexCoord3fvARB);
            (*gl_info).gl_ops.ext.p_glMultiTexCoord4fARB = Some(wine_glMultiTexCoord4fARB);
            (*gl_info).gl_ops.ext.p_glMultiTexCoord4fvARB = Some(wine_glMultiTexCoord4fvARB);
            (*gl_info).gl_ops.ext.p_glMultiTexCoord2svARB = Some(wine_glMultiTexCoord2svARB);
            (*gl_info).gl_ops.ext.p_glMultiTexCoord4svARB = Some(wine_glMultiTexCoord4svARB);
            old_multitex_glGetIntegerv = *(core::ptr::addr_of!((*gl_info).gl_ops.gl.p_glGetIntegerv) as *const _);
            (*gl_info).gl_ops.gl.p_glGetIntegerv =
                Some(wine_glGetIntegerv);
            old_multitex_glGetFloatv = *(core::ptr::addr_of!((*gl_info).gl_ops.gl.p_glGetFloatv) as *const _);
            (*gl_info).gl_ops.gl.p_glGetFloatv =
                Some(wine_glGetFloatv);
            old_multitex_glGetDoublev = *(core::ptr::addr_of!((*gl_info).gl_ops.gl.p_glGetDoublev) as *const _);
            (*gl_info).gl_ops.gl.p_glGetDoublev =
                Some(wine_glGetDoublev);
            (*gl_info).supported[ARB_MULTITEXTURE as usize] = TRUE as BOOL;
            return;
        }
        EXT_FOG_COORD => {
            if (*gl_info).supported[EXT_FOG_COORD as usize] != 0 {
                return;
            }
            if (*gl_info).gl_ops.gl.p_glFogi
                == Some(wine_glFogi)
            {
                WINE_FIXME!("EXT_fog_coord emulation hooks already applied.\n");
                return;
            }
            WINE_TRACE!("Applying GL_ARB_fog_coord emulation hooks\n");
            if (*gl_info).supported[ARB_FRAGMENT_PROGRAM as usize] != 0 {
                WINE_FIXME!("GL implementation supports GL_ARB_fragment_program but not GL_EXT_fog_coord\n");
                WINE_FIXME!("The fog coord emulation will most likely fail\n");
            } else if (*gl_info).supported[ARB_FRAGMENT_SHADER as usize] != 0 {
                WINE_FIXME!("GL implementation supports GL_ARB_fragment_shader but not GL_EXT_fog_coord\n");
                WINE_FIXME!("The fog coord emulation will most likely fail\n");
            }
            old_fogcoord_glFogi = *(core::ptr::addr_of!((*gl_info).gl_ops.gl.p_glFogi) as *const _);
            (*gl_info).gl_ops.gl.p_glFogi =
                Some(wine_glFogi);
            old_fogcoord_glFogiv = *(core::ptr::addr_of!((*gl_info).gl_ops.gl.p_glFogiv) as *const _);
            (*gl_info).gl_ops.gl.p_glFogiv =
                Some(wine_glFogiv);
            old_fogcoord_glFogf = *(core::ptr::addr_of!((*gl_info).gl_ops.gl.p_glFogf) as *const _);
            (*gl_info).gl_ops.gl.p_glFogf =
                Some(wine_glFogf);
            old_fogcoord_glFogfv = *(core::ptr::addr_of!((*gl_info).gl_ops.gl.p_glFogfv) as *const _);
            (*gl_info).gl_ops.gl.p_glFogfv =
                Some(wine_glFogfv);
            old_fogcoord_glEnable = *(core::ptr::addr_of!((*gl_info).p_glEnableWINE) as *const _);
            (*gl_info).p_glEnableWINE = Some(wine_glEnable);
            old_fogcoord_glDisable = *(core::ptr::addr_of!((*gl_info).p_glDisableWINE) as *const _);
            (*gl_info).p_glDisableWINE = Some(wine_glDisable);
            old_fogcoord_glVertex4f = *(core::ptr::addr_of!((*gl_info).gl_ops.gl.p_glVertex4f) as *const _);
            (*gl_info).gl_ops.gl.p_glVertex4f = Some(wine_glVertex4f);
            (*gl_info).gl_ops.gl.p_glVertex4fv =
                Some(wine_glVertex4fv);
            (*gl_info).gl_ops.gl.p_glVertex3f =
                Some(wine_glVertex3f);
            (*gl_info).gl_ops.gl.p_glVertex3fv =
                Some(wine_glVertex3fv);
            old_fogcoord_glColor4f = *(core::ptr::addr_of!((*gl_info).gl_ops.gl.p_glColor4f) as *const _);
            (*gl_info).gl_ops.gl.p_glColor4f = Some(wine_glColor4f);
            (*gl_info).gl_ops.gl.p_glColor4fv =
                Some(wine_glColor4fv);
            (*gl_info).gl_ops.gl.p_glColor3f =
                Some(wine_glColor3f);
            (*gl_info).gl_ops.gl.p_glColor3fv =
                Some(wine_glColor3fv);
            (*gl_info).gl_ops.gl.p_glColor4ub = Some(wine_glColor4ub);
            (*gl_info).gl_ops.ext.p_glFogCoordfEXT =
                Some(wine_glFogCoordfEXT);
            (*gl_info).gl_ops.ext.p_glFogCoordfvEXT =
                Some(wine_glFogCoordfvEXT);
            (*gl_info).gl_ops.ext.p_glFogCoorddEXT =
                Some(wine_glFogCoorddEXT);
            (*gl_info).gl_ops.ext.p_glFogCoorddvEXT =
                Some(wine_glFogCoorddvEXT);
            (*gl_info).supported[EXT_FOG_COORD as usize] = TRUE as BOOL;
            return;
        }
        _ => {
            WINE_FIXME!("Extension %u emulation not supported.\n", ext as ::core::ffi::c_uint);
        }
    };
}
