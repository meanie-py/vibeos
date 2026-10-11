//! apistubs.c's `#include "glfuncs.h"` under USE_GL_FUNC / USE_GL_FUNC_RET, expanded.
//!
//! Every non-i386 GL export forwards through the current dispatch table; i386 gets them from
//! glapi_x86.s instead, exactly as the C build does. Generated from glfuncs.h (one function
//! per row, same order); the translation pipeline produced none of them, because the whole
//! file is an X-macro the model reads as "no declarations".
#![cfg(not(target_arch = "x86"))]
#![allow(non_snake_case)]
use crate::prelude::*;

#[no_mangle]
pub unsafe extern "system" fn glNewList(list: GLuint, mode: GLenum) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).NewList {
        f(list, mode);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glEndList() {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).EndList {
        f();
    }
}

#[no_mangle]
pub unsafe extern "system" fn glCallList(list: GLuint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).CallList {
        f(list);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glCallLists(n: GLsizei, r#type: GLenum, lists: *const GLvoid) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).CallLists {
        f(n, r#type, lists);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glDeleteLists(list: GLuint, range: GLsizei) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).DeleteLists {
        f(list, range);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGenLists(range: GLsizei) -> GLuint {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return 0 as GLuint;
    }
    match (*dispatch).GenLists {
        Some(f) => f(range),
        None => 0 as GLuint,
    }
}

#[no_mangle]
pub unsafe extern "system" fn glListBase(base: GLuint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).ListBase {
        f(base);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glBegin(mode: GLenum) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Begin {
        f(mode);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glBitmap(width: GLsizei, height: GLsizei, xorig: GLfloat, yorig: GLfloat, xmove: GLfloat, ymove: GLfloat, bitmap: *const GLubyte) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Bitmap {
        f(width, height, xorig, yorig, xmove, ymove, bitmap);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor3b(red: GLbyte, green: GLbyte, blue: GLbyte) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color3b {
        f(red, green, blue);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor3bv(v: *const GLbyte) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color3bv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor3d(red: GLdouble, green: GLdouble, blue: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color3d {
        f(red, green, blue);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor3dv(v: *const GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color3dv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor3f(red: GLfloat, green: GLfloat, blue: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color3f {
        f(red, green, blue);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor3fv(v: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color3fv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor3i(red: GLint, green: GLint, blue: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color3i {
        f(red, green, blue);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor3iv(v: *const GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color3iv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor3s(red: GLshort, green: GLshort, blue: GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color3s {
        f(red, green, blue);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor3sv(v: *const GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color3sv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor3ub(red: GLubyte, green: GLubyte, blue: GLubyte) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color3ub {
        f(red, green, blue);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor3ubv(v: *const GLubyte) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color3ubv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor3ui(red: GLuint, green: GLuint, blue: GLuint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color3ui {
        f(red, green, blue);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor3uiv(v: *const GLuint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color3uiv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor3us(red: GLushort, green: GLushort, blue: GLushort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color3us {
        f(red, green, blue);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor3usv(v: *const GLushort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color3usv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor4b(red: GLbyte, green: GLbyte, blue: GLbyte, alpha: GLbyte) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color4b {
        f(red, green, blue, alpha);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor4bv(v: *const GLbyte) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color4bv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor4d(red: GLdouble, green: GLdouble, blue: GLdouble, alpha: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color4d {
        f(red, green, blue, alpha);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor4dv(v: *const GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color4dv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor4f(red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color4f {
        f(red, green, blue, alpha);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor4fv(v: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color4fv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor4i(red: GLint, green: GLint, blue: GLint, alpha: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color4i {
        f(red, green, blue, alpha);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor4iv(v: *const GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color4iv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor4s(red: GLshort, green: GLshort, blue: GLshort, alpha: GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color4s {
        f(red, green, blue, alpha);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor4sv(v: *const GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color4sv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor4ub(red: GLubyte, green: GLubyte, blue: GLubyte, alpha: GLubyte) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color4ub {
        f(red, green, blue, alpha);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor4ubv(v: *const GLubyte) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color4ubv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor4ui(red: GLuint, green: GLuint, blue: GLuint, alpha: GLuint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color4ui {
        f(red, green, blue, alpha);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor4uiv(v: *const GLuint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color4uiv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor4us(red: GLushort, green: GLushort, blue: GLushort, alpha: GLushort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color4us {
        f(red, green, blue, alpha);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColor4usv(v: *const GLushort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Color4usv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glEdgeFlag(flag: GLboolean) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).EdgeFlag {
        f(flag);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glEdgeFlagv(flag: *const GLboolean) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).EdgeFlagv {
        f(flag);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glEnd() {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).End {
        f();
    }
}

#[no_mangle]
pub unsafe extern "system" fn glIndexd(c: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Indexd {
        f(c);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glIndexdv(c: *const GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Indexdv {
        f(c);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glIndexf(c: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Indexf {
        f(c);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glIndexfv(c: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Indexfv {
        f(c);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glIndexi(c: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Indexi {
        f(c);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glIndexiv(c: *const GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Indexiv {
        f(c);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glIndexs(c: GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Indexs {
        f(c);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glIndexsv(c: *const GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Indexsv {
        f(c);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glNormal3b(nx: GLbyte, ny: GLbyte, nz: GLbyte) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Normal3b {
        f(nx, ny, nz);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glNormal3bv(v: *const GLbyte) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Normal3bv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glNormal3d(nx: GLdouble, ny: GLdouble, nz: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Normal3d {
        f(nx, ny, nz);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glNormal3dv(v: *const GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Normal3dv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glNormal3f(nx: GLfloat, ny: GLfloat, nz: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Normal3f {
        f(nx, ny, nz);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glNormal3fv(v: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Normal3fv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glNormal3i(nx: GLint, ny: GLint, nz: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Normal3i {
        f(nx, ny, nz);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glNormal3iv(v: *const GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Normal3iv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glNormal3s(nx: GLshort, ny: GLshort, nz: GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Normal3s {
        f(nx, ny, nz);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glNormal3sv(v: *const GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Normal3sv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos2d(x: GLdouble, y: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos2d {
        f(x, y);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos2dv(v: *const GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos2dv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos2f(x: GLfloat, y: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos2f {
        f(x, y);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos2fv(v: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos2fv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos2i(x: GLint, y: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos2i {
        f(x, y);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos2iv(v: *const GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos2iv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos2s(x: GLshort, y: GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos2s {
        f(x, y);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos2sv(v: *const GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos2sv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos3d(x: GLdouble, y: GLdouble, z: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos3d {
        f(x, y, z);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos3dv(v: *const GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos3dv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos3f(x: GLfloat, y: GLfloat, z: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos3f {
        f(x, y, z);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos3fv(v: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos3fv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos3i(x: GLint, y: GLint, z: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos3i {
        f(x, y, z);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos3iv(v: *const GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos3iv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos3s(x: GLshort, y: GLshort, z: GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos3s {
        f(x, y, z);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos3sv(v: *const GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos3sv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos4d(x: GLdouble, y: GLdouble, z: GLdouble, w: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos4d {
        f(x, y, z, w);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos4dv(v: *const GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos4dv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos4f(x: GLfloat, y: GLfloat, z: GLfloat, w: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos4f {
        f(x, y, z, w);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos4fv(v: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos4fv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos4i(x: GLint, y: GLint, z: GLint, w: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos4i {
        f(x, y, z, w);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos4iv(v: *const GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos4iv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos4s(x: GLshort, y: GLshort, z: GLshort, w: GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos4s {
        f(x, y, z, w);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRasterPos4sv(v: *const GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).RasterPos4sv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRectd(x1: GLdouble, y1: GLdouble, x2: GLdouble, y2: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Rectd {
        f(x1, y1, x2, y2);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRectdv(v1: *const GLdouble, v2: *const GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Rectdv {
        f(v1, v2);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRectf(x1: GLfloat, y1: GLfloat, x2: GLfloat, y2: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Rectf {
        f(x1, y1, x2, y2);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRectfv(v1: *const GLfloat, v2: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Rectfv {
        f(v1, v2);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRecti(x1: GLint, y1: GLint, x2: GLint, y2: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Recti {
        f(x1, y1, x2, y2);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRectiv(v1: *const GLint, v2: *const GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Rectiv {
        f(v1, v2);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRects(x1: GLshort, y1: GLshort, x2: GLshort, y2: GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Rects {
        f(x1, y1, x2, y2);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRectsv(v1: *const GLshort, v2: *const GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Rectsv {
        f(v1, v2);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord1d(s: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord1d {
        f(s);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord1dv(v: *const GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord1dv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord1f(s: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord1f {
        f(s);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord1fv(v: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord1fv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord1i(s: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord1i {
        f(s);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord1iv(v: *const GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord1iv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord1s(s: GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord1s {
        f(s);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord1sv(v: *const GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord1sv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord2d(s: GLdouble, t: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord2d {
        f(s, t);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord2dv(v: *const GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord2dv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord2f(s: GLfloat, t: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord2f {
        f(s, t);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord2fv(v: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord2fv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord2i(s: GLint, t: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord2i {
        f(s, t);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord2iv(v: *const GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord2iv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord2s(s: GLshort, t: GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord2s {
        f(s, t);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord2sv(v: *const GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord2sv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord3d(s: GLdouble, t: GLdouble, r: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord3d {
        f(s, t, r);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord3dv(v: *const GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord3dv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord3f(s: GLfloat, t: GLfloat, r: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord3f {
        f(s, t, r);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord3fv(v: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord3fv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord3i(s: GLint, t: GLint, r: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord3i {
        f(s, t, r);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord3iv(v: *const GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord3iv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord3s(s: GLshort, t: GLshort, r: GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord3s {
        f(s, t, r);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord3sv(v: *const GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord3sv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord4d(s: GLdouble, t: GLdouble, r: GLdouble, q: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord4d {
        f(s, t, r, q);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord4dv(v: *const GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord4dv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord4f(s: GLfloat, t: GLfloat, r: GLfloat, q: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord4f {
        f(s, t, r, q);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord4fv(v: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord4fv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord4i(s: GLint, t: GLint, r: GLint, q: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord4i {
        f(s, t, r, q);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord4iv(v: *const GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord4iv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord4s(s: GLshort, t: GLshort, r: GLshort, q: GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord4s {
        f(s, t, r, q);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoord4sv(v: *const GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoord4sv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex2d(x: GLdouble, y: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex2d {
        f(x, y);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex2dv(v: *const GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex2dv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex2f(x: GLfloat, y: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex2f {
        f(x, y);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex2fv(v: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex2fv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex2i(x: GLint, y: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex2i {
        f(x, y);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex2iv(v: *const GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex2iv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex2s(x: GLshort, y: GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex2s {
        f(x, y);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex2sv(v: *const GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex2sv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex3d(x: GLdouble, y: GLdouble, z: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex3d {
        f(x, y, z);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex3dv(v: *const GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex3dv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex3f(x: GLfloat, y: GLfloat, z: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex3f {
        f(x, y, z);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex3fv(v: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex3fv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex3i(x: GLint, y: GLint, z: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex3i {
        f(x, y, z);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex3iv(v: *const GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex3iv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex3s(x: GLshort, y: GLshort, z: GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex3s {
        f(x, y, z);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex3sv(v: *const GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex3sv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex4d(x: GLdouble, y: GLdouble, z: GLdouble, w: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex4d {
        f(x, y, z, w);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex4dv(v: *const GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex4dv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex4f(x: GLfloat, y: GLfloat, z: GLfloat, w: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex4f {
        f(x, y, z, w);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex4fv(v: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex4fv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex4i(x: GLint, y: GLint, z: GLint, w: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex4i {
        f(x, y, z, w);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex4iv(v: *const GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex4iv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex4s(x: GLshort, y: GLshort, z: GLshort, w: GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex4s {
        f(x, y, z, w);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertex4sv(v: *const GLshort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Vertex4sv {
        f(v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glClipPlane(plane: GLenum, equation: *const GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).ClipPlane {
        f(plane, equation);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColorMaterial(face: GLenum, mode: GLenum) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).ColorMaterial {
        f(face, mode);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glCullFace(mode: GLenum) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).CullFace {
        f(mode);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glFogf(pname: GLenum, param: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Fogf {
        f(pname, param);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glFogfv(pname: GLenum, params: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Fogfv {
        f(pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glFogi(pname: GLenum, param: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Fogi {
        f(pname, param);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glFogiv(pname: GLenum, params: *const GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Fogiv {
        f(pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glFrontFace(mode: GLenum) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).FrontFace {
        f(mode);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glHint(target: GLenum, mode: GLenum) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Hint {
        f(target, mode);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glLightf(light: GLenum, pname: GLenum, param: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Lightf {
        f(light, pname, param);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glLightfv(light: GLenum, pname: GLenum, params: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Lightfv {
        f(light, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glLighti(light: GLenum, pname: GLenum, param: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Lighti {
        f(light, pname, param);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glLightiv(light: GLenum, pname: GLenum, params: *const GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Lightiv {
        f(light, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glLightModelf(pname: GLenum, param: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).LightModelf {
        f(pname, param);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glLightModelfv(pname: GLenum, params: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).LightModelfv {
        f(pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glLightModeli(pname: GLenum, param: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).LightModeli {
        f(pname, param);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glLightModeliv(pname: GLenum, params: *const GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).LightModeliv {
        f(pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glLineStipple(factor: GLint, pattern: GLushort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).LineStipple {
        f(factor, pattern);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glLineWidth(width: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).LineWidth {
        f(width);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glMaterialf(face: GLenum, pname: GLenum, param: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Materialf {
        f(face, pname, param);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glMaterialfv(face: GLenum, pname: GLenum, params: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Materialfv {
        f(face, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glMateriali(face: GLenum, pname: GLenum, param: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Materiali {
        f(face, pname, param);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glMaterialiv(face: GLenum, pname: GLenum, params: *const GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Materialiv {
        f(face, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glPointSize(size: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).PointSize {
        f(size);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glPolygonMode(face: GLenum, mode: GLenum) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).PolygonMode {
        f(face, mode);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glPolygonStipple(mask: *const GLubyte) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).PolygonStipple {
        f(mask);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glScissor(x: GLint, y: GLint, width: GLsizei, height: GLsizei) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Scissor {
        f(x, y, width, height);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glShadeModel(mode: GLenum) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).ShadeModel {
        f(mode);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexParameterf(target: GLenum, pname: GLenum, param: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexParameterf {
        f(target, pname, param);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexParameterfv(target: GLenum, pname: GLenum, params: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexParameterfv {
        f(target, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexParameteri(target: GLenum, pname: GLenum, param: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexParameteri {
        f(target, pname, param);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexParameteriv(target: GLenum, pname: GLenum, params: *const GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexParameteriv {
        f(target, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexImage1D(target: GLenum, level: GLint, internalformat: GLint, width: GLsizei, border: GLint, format: GLenum, r#type: GLenum, pixels: *const GLvoid) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexImage1D {
        f(target, level, internalformat, width, border, format, r#type, pixels);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexImage2D(target: GLenum, level: GLint, internalformat: GLint, width: GLsizei, height: GLsizei, border: GLint, format: GLenum, r#type: GLenum, pixels: *const GLvoid) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexImage2D {
        f(target, level, internalformat, width, height, border, format, r#type, pixels);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexEnvf(target: GLenum, pname: GLenum, param: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexEnvf {
        f(target, pname, param);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexEnvfv(target: GLenum, pname: GLenum, params: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexEnvfv {
        f(target, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexEnvi(target: GLenum, pname: GLenum, param: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexEnvi {
        f(target, pname, param);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexEnviv(target: GLenum, pname: GLenum, params: *const GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexEnviv {
        f(target, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexGend(coord: GLenum, pname: GLenum, param: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexGend {
        f(coord, pname, param);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexGendv(coord: GLenum, pname: GLenum, params: *const GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexGendv {
        f(coord, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexGenf(coord: GLenum, pname: GLenum, param: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexGenf {
        f(coord, pname, param);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexGenfv(coord: GLenum, pname: GLenum, params: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexGenfv {
        f(coord, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexGeni(coord: GLenum, pname: GLenum, param: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexGeni {
        f(coord, pname, param);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexGeniv(coord: GLenum, pname: GLenum, params: *const GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexGeniv {
        f(coord, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glFeedbackBuffer(size: GLsizei, r#type: GLenum, buffer: *mut GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).FeedbackBuffer {
        f(size, r#type, buffer);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glSelectBuffer(size: GLsizei, buffer: *mut GLuint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).SelectBuffer {
        f(size, buffer);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRenderMode(mode: GLenum) -> GLint {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return 0 as GLint;
    }
    match (*dispatch).RenderMode {
        Some(f) => f(mode),
        None => 0 as GLint,
    }
}

#[no_mangle]
pub unsafe extern "system" fn glInitNames() {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).InitNames {
        f();
    }
}

#[no_mangle]
pub unsafe extern "system" fn glLoadName(name: GLuint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).LoadName {
        f(name);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glPassThrough(token: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).PassThrough {
        f(token);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glPopName() {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).PopName {
        f();
    }
}

#[no_mangle]
pub unsafe extern "system" fn glPushName(name: GLuint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).PushName {
        f(name);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glDrawBuffer(mode: GLenum) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).DrawBuffer {
        f(mode);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glClear(mask: GLbitfield) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Clear {
        f(mask);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glClearAccum(red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).ClearAccum {
        f(red, green, blue, alpha);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glClearIndex(c: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).ClearIndex {
        f(c);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glClearColor(red: GLclampf, green: GLclampf, blue: GLclampf, alpha: GLclampf) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).ClearColor {
        f(red, green, blue, alpha);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glClearStencil(s: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).ClearStencil {
        f(s);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glClearDepth(depth: GLclampd) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).ClearDepth {
        f(depth);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glStencilMask(mask: GLuint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).StencilMask {
        f(mask);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColorMask(red: GLboolean, green: GLboolean, blue: GLboolean, alpha: GLboolean) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).ColorMask {
        f(red, green, blue, alpha);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glDepthMask(flag: GLboolean) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).DepthMask {
        f(flag);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glIndexMask(mask: GLuint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).IndexMask {
        f(mask);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glAccum(op: GLenum, value: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Accum {
        f(op, value);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glDisable(cap: GLenum) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Disable {
        f(cap);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glEnable(cap: GLenum) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Enable {
        f(cap);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glFinish() {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Finish {
        f();
    }
}

#[no_mangle]
pub unsafe extern "system" fn glFlush() {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Flush {
        f();
    }
}

#[no_mangle]
pub unsafe extern "system" fn glPopAttrib() {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).PopAttrib {
        f();
    }
}

#[no_mangle]
pub unsafe extern "system" fn glPushAttrib(mask: GLbitfield) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).PushAttrib {
        f(mask);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glMap1d(target: GLenum, u1: GLdouble, u2: GLdouble, stride: GLint, order: GLint, points: *const GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Map1d {
        f(target, u1, u2, stride, order, points);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glMap1f(target: GLenum, u1: GLfloat, u2: GLfloat, stride: GLint, order: GLint, points: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Map1f {
        f(target, u1, u2, stride, order, points);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glMap2d(target: GLenum, u1: GLdouble, u2: GLdouble, ustride: GLint, uorder: GLint, v1: GLdouble, v2: GLdouble, vstride: GLint, vorder: GLint, points: *const GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Map2d {
        f(target, u1, u2, ustride, uorder, v1, v2, vstride, vorder, points);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glMap2f(target: GLenum, u1: GLfloat, u2: GLfloat, ustride: GLint, uorder: GLint, v1: GLfloat, v2: GLfloat, vstride: GLint, vorder: GLint, points: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Map2f {
        f(target, u1, u2, ustride, uorder, v1, v2, vstride, vorder, points);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glMapGrid1d(un: GLint, u1: GLdouble, u2: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).MapGrid1d {
        f(un, u1, u2);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glMapGrid1f(un: GLint, u1: GLfloat, u2: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).MapGrid1f {
        f(un, u1, u2);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glMapGrid2d(un: GLint, u1: GLdouble, u2: GLdouble, vn: GLint, v1: GLdouble, v2: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).MapGrid2d {
        f(un, u1, u2, vn, v1, v2);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glMapGrid2f(un: GLint, u1: GLfloat, u2: GLfloat, vn: GLint, v1: GLfloat, v2: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).MapGrid2f {
        f(un, u1, u2, vn, v1, v2);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glEvalCoord1d(u: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).EvalCoord1d {
        f(u);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glEvalCoord1dv(u: *const GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).EvalCoord1dv {
        f(u);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glEvalCoord1f(u: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).EvalCoord1f {
        f(u);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glEvalCoord1fv(u: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).EvalCoord1fv {
        f(u);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glEvalCoord2d(u: GLdouble, v: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).EvalCoord2d {
        f(u, v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glEvalCoord2dv(u: *const GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).EvalCoord2dv {
        f(u);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glEvalCoord2f(u: GLfloat, v: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).EvalCoord2f {
        f(u, v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glEvalCoord2fv(u: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).EvalCoord2fv {
        f(u);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glEvalMesh1(mode: GLenum, i1: GLint, i2: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).EvalMesh1 {
        f(mode, i1, i2);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glEvalPoint1(i: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).EvalPoint1 {
        f(i);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glEvalMesh2(mode: GLenum, i1: GLint, i2: GLint, j1: GLint, j2: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).EvalMesh2 {
        f(mode, i1, i2, j1, j2);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glEvalPoint2(i: GLint, j: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).EvalPoint2 {
        f(i, j);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glAlphaFunc(func: GLenum, r#ref: GLclampf) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).AlphaFunc {
        f(func, r#ref);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glBlendFunc(sfactor: GLenum, dfactor: GLenum) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).BlendFunc {
        f(sfactor, dfactor);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glLogicOp(opcode: GLenum) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).LogicOp {
        f(opcode);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glStencilFunc(func: GLenum, r#ref: GLint, mask: GLuint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).StencilFunc {
        f(func, r#ref, mask);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glStencilOp(fail: GLenum, zfail: GLenum, zpass: GLenum) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).StencilOp {
        f(fail, zfail, zpass);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glDepthFunc(func: GLenum) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).DepthFunc {
        f(func);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glPixelZoom(xfactor: GLfloat, yfactor: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).PixelZoom {
        f(xfactor, yfactor);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glPixelTransferf(pname: GLenum, param: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).PixelTransferf {
        f(pname, param);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glPixelTransferi(pname: GLenum, param: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).PixelTransferi {
        f(pname, param);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glPixelStoref(pname: GLenum, param: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).PixelStoref {
        f(pname, param);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glPixelStorei(pname: GLenum, param: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).PixelStorei {
        f(pname, param);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glPixelMapfv(map: GLenum, mapsize: GLint, values: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).PixelMapfv {
        f(map, mapsize, values);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glPixelMapuiv(map: GLenum, mapsize: GLint, values: *const GLuint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).PixelMapuiv {
        f(map, mapsize, values);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glPixelMapusv(map: GLenum, mapsize: GLint, values: *const GLushort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).PixelMapusv {
        f(map, mapsize, values);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glReadBuffer(mode: GLenum) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).ReadBuffer {
        f(mode);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glCopyPixels(x: GLint, y: GLint, width: GLsizei, height: GLsizei, r#type: GLenum) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).CopyPixels {
        f(x, y, width, height, r#type);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glReadPixels(x: GLint, y: GLint, width: GLsizei, height: GLsizei, format: GLenum, r#type: GLenum, pixels: *mut GLvoid) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).ReadPixels {
        f(x, y, width, height, format, r#type, pixels);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glDrawPixels(width: GLsizei, height: GLsizei, format: GLenum, r#type: GLenum, pixels: *const GLvoid) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).DrawPixels {
        f(width, height, format, r#type, pixels);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetBooleanv(pname: GLenum, params: *mut GLboolean) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetBooleanv {
        f(pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetClipPlane(plane: GLenum, equation: *mut GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetClipPlane {
        f(plane, equation);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetDoublev(pname: GLenum, params: *mut GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetDoublev {
        f(pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetError() -> GLenum {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return 0 as GLenum;
    }
    match (*dispatch).GetError {
        Some(f) => f(),
        None => 0 as GLenum,
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetFloatv(pname: GLenum, params: *mut GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetFloatv {
        f(pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetIntegerv(pname: GLenum, params: *mut GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetIntegerv {
        f(pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetLightfv(light: GLenum, pname: GLenum, params: *mut GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetLightfv {
        f(light, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetLightiv(light: GLenum, pname: GLenum, params: *mut GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetLightiv {
        f(light, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetMapdv(target: GLenum, query: GLenum, v: *mut GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetMapdv {
        f(target, query, v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetMapfv(target: GLenum, query: GLenum, v: *mut GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetMapfv {
        f(target, query, v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetMapiv(target: GLenum, query: GLenum, v: *mut GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetMapiv {
        f(target, query, v);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetMaterialfv(face: GLenum, pname: GLenum, params: *mut GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetMaterialfv {
        f(face, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetMaterialiv(face: GLenum, pname: GLenum, params: *mut GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetMaterialiv {
        f(face, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetPixelMapfv(map: GLenum, values: *mut GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetPixelMapfv {
        f(map, values);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetPixelMapuiv(map: GLenum, values: *mut GLuint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetPixelMapuiv {
        f(map, values);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetPixelMapusv(map: GLenum, values: *mut GLushort) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetPixelMapusv {
        f(map, values);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetPolygonStipple(mask: *mut GLubyte) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetPolygonStipple {
        f(mask);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetString(name: GLenum) -> *const GLubyte {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return core::ptr::null();
    }
    match (*dispatch).GetString {
        Some(f) => f(name),
        None => core::ptr::null(),
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetTexEnvfv(target: GLenum, pname: GLenum, params: *mut GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetTexEnvfv {
        f(target, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetTexEnviv(target: GLenum, pname: GLenum, params: *mut GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetTexEnviv {
        f(target, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetTexGendv(coord: GLenum, pname: GLenum, params: *mut GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetTexGendv {
        f(coord, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetTexGenfv(coord: GLenum, pname: GLenum, params: *mut GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetTexGenfv {
        f(coord, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetTexGeniv(coord: GLenum, pname: GLenum, params: *mut GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetTexGeniv {
        f(coord, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetTexImage(target: GLenum, level: GLint, format: GLenum, r#type: GLenum, pixels: *mut GLvoid) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetTexImage {
        f(target, level, format, r#type, pixels);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetTexParameterfv(target: GLenum, pname: GLenum, params: *mut GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetTexParameterfv {
        f(target, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetTexParameteriv(target: GLenum, pname: GLenum, params: *mut GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetTexParameteriv {
        f(target, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetTexLevelParameterfv(target: GLenum, level: GLint, pname: GLenum, params: *mut GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetTexLevelParameterfv {
        f(target, level, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetTexLevelParameteriv(target: GLenum, level: GLint, pname: GLenum, params: *mut GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetTexLevelParameteriv {
        f(target, level, pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glIsEnabled(cap: GLenum) -> GLboolean {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return 0 as GLboolean;
    }
    match (*dispatch).IsEnabled {
        Some(f) => f(cap),
        None => 0 as GLboolean,
    }
}

#[no_mangle]
pub unsafe extern "system" fn glIsList(list: GLuint) -> GLboolean {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return 0 as GLboolean;
    }
    match (*dispatch).IsList {
        Some(f) => f(list),
        None => 0 as GLboolean,
    }
}

#[no_mangle]
pub unsafe extern "system" fn glDepthRange(zNear: GLclampd, zFar: GLclampd) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).DepthRange {
        f(zNear, zFar);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glFrustum(left: GLdouble, right: GLdouble, bottom: GLdouble, top: GLdouble, zNear: GLdouble, zFar: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Frustum {
        f(left, right, bottom, top, zNear, zFar);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glLoadIdentity() {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).LoadIdentity {
        f();
    }
}

#[no_mangle]
pub unsafe extern "system" fn glLoadMatrixf(m: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).LoadMatrixf {
        f(m);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glLoadMatrixd(m: *const GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).LoadMatrixd {
        f(m);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glMatrixMode(mode: GLenum) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).MatrixMode {
        f(mode);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glMultMatrixf(m: *const GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).MultMatrixf {
        f(m);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glMultMatrixd(m: *const GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).MultMatrixd {
        f(m);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glOrtho(left: GLdouble, right: GLdouble, bottom: GLdouble, top: GLdouble, zNear: GLdouble, zFar: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Ortho {
        f(left, right, bottom, top, zNear, zFar);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glPopMatrix() {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).PopMatrix {
        f();
    }
}

#[no_mangle]
pub unsafe extern "system" fn glPushMatrix() {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).PushMatrix {
        f();
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRotated(angle: GLdouble, x: GLdouble, y: GLdouble, z: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Rotated {
        f(angle, x, y, z);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glRotatef(angle: GLfloat, x: GLfloat, y: GLfloat, z: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Rotatef {
        f(angle, x, y, z);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glScaled(x: GLdouble, y: GLdouble, z: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Scaled {
        f(x, y, z);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glScalef(x: GLfloat, y: GLfloat, z: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Scalef {
        f(x, y, z);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTranslated(x: GLdouble, y: GLdouble, z: GLdouble) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Translated {
        f(x, y, z);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTranslatef(x: GLfloat, y: GLfloat, z: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Translatef {
        f(x, y, z);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glViewport(x: GLint, y: GLint, width: GLsizei, height: GLsizei) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Viewport {
        f(x, y, width, height);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glArrayElement(i: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).ArrayElement {
        f(i);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glBindTexture(target: GLenum, texture: GLuint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).BindTexture {
        f(target, texture);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glColorPointer(size: GLint, r#type: GLenum, stride: GLsizei, pointer: *const GLvoid) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).ColorPointer {
        f(size, r#type, stride, pointer);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glDisableClientState(array: GLenum) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).DisableClientState {
        f(array);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glDrawArrays(mode: GLenum, first: GLint, count: GLsizei) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).DrawArrays {
        f(mode, first, count);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glDrawElements(mode: GLenum, count: GLsizei, r#type: GLenum, indices: *const GLvoid) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).DrawElements {
        f(mode, count, r#type, indices);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glEdgeFlagPointer(stride: GLsizei, pointer: *const GLvoid) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).EdgeFlagPointer {
        f(stride, pointer);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glEnableClientState(array: GLenum) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).EnableClientState {
        f(array);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glIndexPointer(r#type: GLenum, stride: GLsizei, pointer: *const GLvoid) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).IndexPointer {
        f(r#type, stride, pointer);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glIndexub(c: GLubyte) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Indexub {
        f(c);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glIndexubv(c: *const GLubyte) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).Indexubv {
        f(c);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glInterleavedArrays(format: GLenum, stride: GLsizei, pointer: *const GLvoid) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).InterleavedArrays {
        f(format, stride, pointer);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glNormalPointer(r#type: GLenum, stride: GLsizei, pointer: *const GLvoid) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).NormalPointer {
        f(r#type, stride, pointer);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glPolygonOffset(factor: GLfloat, units: GLfloat) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).PolygonOffset {
        f(factor, units);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexCoordPointer(size: GLint, r#type: GLenum, stride: GLsizei, pointer: *const GLvoid) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexCoordPointer {
        f(size, r#type, stride, pointer);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glVertexPointer(size: GLint, r#type: GLenum, stride: GLsizei, pointer: *const GLvoid) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).VertexPointer {
        f(size, r#type, stride, pointer);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glAreTexturesResident(n: GLsizei, textures: *const GLuint, residences: *mut GLboolean) -> GLboolean {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return 0 as GLboolean;
    }
    match (*dispatch).AreTexturesResident {
        Some(f) => f(n, textures, residences),
        None => 0 as GLboolean,
    }
}

#[no_mangle]
pub unsafe extern "system" fn glCopyTexImage1D(target: GLenum, level: GLint, internalformat: GLenum, x: GLint, y: GLint, width: GLsizei, border: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).CopyTexImage1D {
        f(target, level, internalformat, x, y, width, border);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glCopyTexImage2D(target: GLenum, level: GLint, internalformat: GLenum, x: GLint, y: GLint, width: GLsizei, height: GLsizei, border: GLint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).CopyTexImage2D {
        f(target, level, internalformat, x, y, width, height, border);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glCopyTexSubImage1D(target: GLenum, level: GLint, xoffset: GLint, x: GLint, y: GLint, width: GLsizei) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).CopyTexSubImage1D {
        f(target, level, xoffset, x, y, width);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glCopyTexSubImage2D(target: GLenum, level: GLint, xoffset: GLint, yoffset: GLint, x: GLint, y: GLint, width: GLsizei, height: GLsizei) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).CopyTexSubImage2D {
        f(target, level, xoffset, yoffset, x, y, width, height);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glDeleteTextures(n: GLsizei, textures: *const GLuint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).DeleteTextures {
        f(n, textures);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGenTextures(n: GLsizei, textures: *mut GLuint) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GenTextures {
        f(n, textures);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glGetPointerv(pname: GLenum, params: *mut *mut GLvoid) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).GetPointerv {
        f(pname, params);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glIsTexture(texture: GLuint) -> GLboolean {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return 0 as GLboolean;
    }
    match (*dispatch).IsTexture {
        Some(f) => f(texture),
        None => 0 as GLboolean,
    }
}

#[no_mangle]
pub unsafe extern "system" fn glPrioritizeTextures(n: GLsizei, textures: *const GLuint, priorities: *const GLclampf) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).PrioritizeTextures {
        f(n, textures, priorities);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexSubImage1D(target: GLenum, level: GLint, xoffset: GLint, width: GLsizei, format: GLenum, r#type: GLenum, pixels: *const GLvoid) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexSubImage1D {
        f(target, level, xoffset, width, format, r#type, pixels);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glTexSubImage2D(target: GLenum, level: GLint, xoffset: GLint, yoffset: GLint, width: GLsizei, height: GLsizei, format: GLenum, r#type: GLenum, pixels: *const GLvoid) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).TexSubImage2D {
        f(target, level, xoffset, yoffset, width, height, format, r#type, pixels);
    }
}

#[no_mangle]
pub unsafe extern "system" fn glPopClientAttrib() {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).PopClientAttrib {
        f();
    }
}

#[no_mangle]
pub unsafe extern "system" fn glPushClientAttrib(mask: GLbitfield) {
    let dispatch = IntGetCurrentDispatchTable();
    if dispatch.is_null() {
        return;
    }
    if let Some(f) = (*dispatch).PushClientAttrib {
        f(mask);
    }
}
