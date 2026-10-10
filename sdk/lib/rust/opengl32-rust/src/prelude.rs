//! opengl32-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// reactos/debug.h DPRINT/UNIMPLEMENTED — no-op MACROS, never out-of-line fns
// (field manual §9.65: an out-of-line no-op is an optimization barrier). debug_macros.py
// rewrites C's bare `UNIMPLEMENTED;` to `UNIMPLEMENTED!()`, so the prelude must have it.
// DPRINT1 PRINTS in this DBG=1 tree, kernel and user mode alike: it comes from
// win32macros-rust with the other macros.
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED_ONCE { ($($arg:tt)*) => {{}}; }
pub use crate::{DPRINT, UNIMPLEMENTED, UNIMPLEMENTED_ONCE};

// ================= CRT ========================================================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}
// Translations reach for the C MACRO forms too (`NT_SUCCESS!(st)`, `CONTAINING_RECORD!(p, T, f)`)
// and a fresh crate otherwise starts with `cannot find macro` in every module that walks a list.
// Address arithmetic, so neither needs an `unsafe` context of its own.
#[macro_export] macro_rules! NT_SUCCESS { ($s:expr) => { (($s) as i32) >= 0 }; }
#[macro_export] macro_rules! CONTAINING_RECORD {
    // An indexed member (`Tail.Overlay.DriverContext[0]`): offsetof the array plus the
    // element stride, as clang folds it -- offset_of! refuses an index.
    ($addr:expr, $ty:ty, $($field:ident).+ [ $idx:expr ] $(,)?) => {{
        const fn __elem<T, const N: usize>(_: *const [T; N]) -> usize {
            core::mem::size_of::<T>()
        }
        let slot = core::mem::MaybeUninit::<$ty>::uninit();
        let stride = __elem(unsafe { core::ptr::addr_of!((*slot.as_ptr()).$($field).+) });
        (($addr) as usize - (core::mem::offset_of!($ty, $($field).+) + ($idx) as usize * stride))
            as *mut $ty
    }};
    // C's field argument is a member PATH (`Tail.Overlay.ListEntry`), and the model
    // often leaves a trailing comma after a multi-line call.
    ($addr:expr, $ty:ty, $($field:ident).+ $(,)?) => {
        (($addr) as usize - core::mem::offset_of!($ty, $($field).+)) as *mut $ty
    };
}
pub use crate::{CONTAINING_RECORD, NT_SUCCESS};
// The Win32 header macros, ported once in win32macros-rust, not per module.
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, ZeroMemory, _countof, debugstr_a, debugstr_an, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, max, min, wide_bytes};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001usize as HKEY; // access-rust/src/prelude.rs
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002usize as HKEY; // console-rust/src/prelude.rs
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0; // comctl32-rust/src/prelude.rs
pub const __WINE_DBCL_ERR: __wine_debug_class = 1; // wine/debug.h enum __wine_debug_class
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3; // appwiz-rust/src/prelude.rs
pub const __WINE_DBCL_WARN: __wine_debug_class = 2; // appwiz-rust/src/prelude.rs
extern "C" {
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // msvcrt-crt-rust/src/lib.rs
    pub fn abs(j: i32) -> i32; // cnt-rust/src/cabi/math.rs
    pub fn ros_dbg_log( cls: __wine_debug_class, ch: *mut __wine_debug_channel, file: *const c_char, func: *const c_char, line: c_int, msg: *const c_char, ... ) -> c_int; // iphlpapi-rust/src/prelude.rs
    pub fn sqrt(x: f64) -> f64; // cnt-rust/src/cabi/math.rs
    pub fn strcmp(a: *const c_char, b: *const c_char) -> c_int; // browser-rust/src/prelude.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
}
extern "system" {
    pub fn CallNextHookEx( arg1: HHOOK, arg2: ::core::ffi::c_int, arg3: WPARAM, arg4: LPARAM, ) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn CreateFontIndirectW(arg1: *const LOGFONTW) -> HFONT; // aclui-rust/src/types_amd64.rs
    pub fn CreateSolidBrush(arg1: COLORREF) -> HBRUSH; // aclui-rust/src/types_amd64.rs
    pub fn DPtoLP(hdc: HDC, lppt: LPPOINT, c: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DeleteObject(arg1: HGDIOBJ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_amd64.rs
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetCurrentObject(arg1: HDC, arg2: UINT) -> HGDIOBJ; // aclui-rust/src/types_amd64.rs
    pub fn GetCurrentThreadId() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetDC(arg1: HWND) -> HDC; // aclui-rust/src/types_amd64.rs
    pub fn GetGlyphOutlineA( hdc: HDC, uChar: UINT, fuFormat: UINT, lpgm: LPGLYPHMETRICS, cjBuffer: DWORD, pvBuffer: LPVOID, lpmat2: *const MAT2, ) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetGlyphOutlineW( hdc: HDC, uChar: UINT, fuFormat: UINT, lpgm: LPGLYPHMETRICS, cjBuffer: DWORD, pvBuffer: LPVOID, lpmat2: *const MAT2, ) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetObjectType(arg1: HGDIOBJ) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetObjectW(h: HANDLE, c: ::core::ffi::c_int, pv: LPVOID) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_amd64.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // advapi32-rust/src/wine/security.rs
    pub fn HeapReAlloc(arg1: HANDLE, arg2: DWORD, arg3: PVOID, arg4: SIZE_T) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_amd64.rs
    pub fn LoadLibraryW(arg1: LPCWSTR) -> HINSTANCE; // aclui-rust/src/types_amd64.rs
    pub fn PatBlt( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: DWORD, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ReleaseDC(arg1: HWND, arg2: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn SelectObject(arg1: HDC, arg2: HGDIOBJ) -> HGDIOBJ; // aclui-rust/src/types_amd64.rs
    pub fn SetDIBitsToDevice( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: DWORD, arg5: DWORD, arg6: ::core::ffi::c_int, arg7: ::core::ffi::c_int, arg8: UINT, arg9: UINT, arg10: *const ::core::ffi::c_void, arg11: *const BITMAPINFO, arg12: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn SetLastError(dwErrCode: DWORD); // advapi32-rust/src/sec/misc.rs
    pub fn SetPixel( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: COLORREF, ) -> COLORREF; // aclui-rust/src/types_amd64.rs
    pub fn UnhookWindowsHookEx(arg1: HHOOK) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn WindowFromDC(hDC: HDC) -> HWND; // aclui-rust/src/types_amd64.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::opengl32::wgl::IntDeleteAllContexts;
pub use crate::opengl32::icdload::IntDeleteAllICDs;
pub use crate::opengl32::icdload::IntGetIcdData;
pub use crate::opengl32::swimpl::sw_CopyContext;
pub use crate::opengl32::swimpl::sw_CreateContext;
pub use crate::opengl32::swimpl::sw_DeleteContext;
pub use crate::opengl32::swimpl::sw_DescribePixelFormat;
pub use crate::opengl32::swimpl::sw_GetProcAddress;
pub use crate::opengl32::swimpl::sw_ReleaseContext;
pub use crate::opengl32::swimpl::sw_SetContext;
pub use crate::opengl32::swimpl::sw_SetPixelFormat;
pub use crate::opengl32::swimpl::sw_ShareLists;
pub use crate::opengl32::swimpl::sw_SwapBuffers;
pub use crate::opengl32::wgl::wglMakeCurrent;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const STENCIL_BITS: i32 = 8; // clang, swimpl.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn gl_ResizeBuffersMESA(ctx: *mut GLcontext); // swimpl.c
    pub fn gl_Viewport(ctx: *mut GLcontext, x: GLint, y: GLint, width: GLsizei, height: GLsizei); // swimpl.c
    pub fn gl_create_context(
        visual: *mut GLvisual,
        share_list: *mut GLcontext,
        driver_ctx: *mut ::core::ffi::c_void,
    ) -> *mut GLcontext; // swimpl.c
    pub fn gl_create_framebuffer(visual: *mut GLvisual) -> *mut GLframebuffer; // swimpl.c
    pub fn gl_create_visual(
        rgb_flag: GLboolean,
        alpha_flag: GLboolean,
        db_flag: GLboolean,
        depth_bits: GLint,
        stencil_bits: GLint,
        accum_bits: GLint,
        index_bits: GLint,
        red_scale: GLfloat,
        green_scale: GLfloat,
        blue_scale: GLfloat,
        alpha_scale: GLfloat,
        red_bits: GLint,
        green_bits: GLint,
        blue_bits: GLint,
        alpha_bits: GLint,
    ) -> *mut GLvisual; // swimpl.c
    pub fn gl_destroy_context(ctx: *mut GLcontext); // swimpl.c
    pub fn gl_destroy_visual(vis: *mut GLvisual); // swimpl.c
    pub fn gl_make_current(ctx: *mut GLcontext, buffer: *mut GLframebuffer); // swimpl.c
}
extern "system" {
    pub fn GetObjectA(h: HANDLE, c: ::core::ffi::c_int, pv: LPVOID) -> ::core::ffi::c_int; // swimpl.c
    pub fn SetWindowsHookExA(_: ::core::ffi::c_int, _: HOOKPROC, _: HINSTANCE, _: DWORD) -> HHOOK; // swimpl.c
    pub fn _mesa_Accum(op: GLenum, value: GLfloat); // swimpl.c
    pub fn _mesa_AlphaFunc(func: GLenum, ref_0: GLclampf); // swimpl.c
    pub fn _mesa_AreTexturesResident(
        n: GLsizei,
        textures: *const GLuint,
        residences: *mut GLboolean,
    ) -> GLboolean; // swimpl.c
    pub fn _mesa_ArrayElement(i: GLint); // swimpl.c
    pub fn _mesa_Begin(mode: GLenum); // swimpl.c
    pub fn _mesa_BindTexture(target: GLenum, texture: GLuint); // swimpl.c
    pub fn _mesa_Bitmap(
        width: GLsizei,
        height: GLsizei,
        xorig: GLfloat,
        yorig: GLfloat,
        xmove: GLfloat,
        ymove: GLfloat,
        bitmap: *const GLubyte,
    ); // swimpl.c
    pub fn _mesa_BlendFunc(sfactor: GLenum, dfactor: GLenum); // swimpl.c
    pub fn _mesa_CallList(list: GLuint); // swimpl.c
    pub fn _mesa_CallLists(n: GLsizei, type_0: GLenum, lists: *const ::core::ffi::c_void); // swimpl.c
    pub fn _mesa_Clear(mask: GLbitfield); // swimpl.c
    pub fn _mesa_ClearAccum(red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat); // swimpl.c
    pub fn _mesa_ClearColor(red: GLclampf, green: GLclampf, blue: GLclampf, alpha: GLclampf); // swimpl.c
    pub fn _mesa_ClearDepth(depth: GLclampd); // swimpl.c
    pub fn _mesa_ClearIndex(c: GLfloat); // swimpl.c
    pub fn _mesa_ClearStencil(s: GLint); // swimpl.c
    pub fn _mesa_ClipPlane(plane: GLenum, equation: *const GLdouble); // swimpl.c
    pub fn _mesa_Color3b(red: GLbyte, green: GLbyte, blue: GLbyte); // swimpl.c
    pub fn _mesa_Color3bv(v: *const GLbyte); // swimpl.c
    pub fn _mesa_Color3d(red: GLdouble, green: GLdouble, blue: GLdouble); // swimpl.c
    pub fn _mesa_Color3dv(v: *const GLdouble); // swimpl.c
    pub fn _mesa_Color3f(red: GLfloat, green: GLfloat, blue: GLfloat); // swimpl.c
    pub fn _mesa_Color3fv(v: *const GLfloat); // swimpl.c
    pub fn _mesa_Color3i(red: GLint, green: GLint, blue: GLint); // swimpl.c
    pub fn _mesa_Color3iv(v: *const GLint); // swimpl.c
    pub fn _mesa_Color3s(red: GLshort, green: GLshort, blue: GLshort); // swimpl.c
    pub fn _mesa_Color3sv(v: *const GLshort); // swimpl.c
    pub fn _mesa_Color3ub(red: GLubyte, green: GLubyte, blue: GLubyte); // swimpl.c
    pub fn _mesa_Color3ubv(v: *const GLubyte); // swimpl.c
    pub fn _mesa_Color3ui(red: GLuint, green: GLuint, blue: GLuint); // swimpl.c
    pub fn _mesa_Color3uiv(v: *const GLuint); // swimpl.c
    pub fn _mesa_Color3us(red: GLushort, green: GLushort, blue: GLushort); // swimpl.c
    pub fn _mesa_Color3usv(v: *const GLushort); // swimpl.c
    pub fn _mesa_Color4b(red: GLbyte, green: GLbyte, blue: GLbyte, alpha: GLbyte); // swimpl.c
    pub fn _mesa_Color4bv(v: *const GLbyte); // swimpl.c
    pub fn _mesa_Color4d(red: GLdouble, green: GLdouble, blue: GLdouble, alpha: GLdouble); // swimpl.c
    pub fn _mesa_Color4dv(v: *const GLdouble); // swimpl.c
    pub fn _mesa_Color4f(red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat); // swimpl.c
    pub fn _mesa_Color4fv(v: *const GLfloat); // swimpl.c
    pub fn _mesa_Color4i(red: GLint, green: GLint, blue: GLint, alpha: GLint); // swimpl.c
    pub fn _mesa_Color4iv(v: *const GLint); // swimpl.c
    pub fn _mesa_Color4s(red: GLshort, green: GLshort, blue: GLshort, alpha: GLshort); // swimpl.c
    pub fn _mesa_Color4sv(v: *const GLshort); // swimpl.c
    pub fn _mesa_Color4ub(red: GLubyte, green: GLubyte, blue: GLubyte, alpha: GLubyte); // swimpl.c
    pub fn _mesa_Color4ubv(v: *const GLubyte); // swimpl.c
    pub fn _mesa_Color4ui(red: GLuint, green: GLuint, blue: GLuint, alpha: GLuint); // swimpl.c
    pub fn _mesa_Color4uiv(v: *const GLuint); // swimpl.c
    pub fn _mesa_Color4us(red: GLushort, green: GLushort, blue: GLushort, alpha: GLushort); // swimpl.c
    pub fn _mesa_Color4usv(v: *const GLushort); // swimpl.c
    pub fn _mesa_ColorMask(red: GLboolean, green: GLboolean, blue: GLboolean, alpha: GLboolean); // swimpl.c
    pub fn _mesa_ColorMaterial(face: GLenum, mode: GLenum); // swimpl.c
    pub fn _mesa_ColorPointer(
        size: GLint,
        type_0: GLenum,
        stride: GLsizei,
        pointer: *const ::core::ffi::c_void,
    ); // swimpl.c
    pub fn _mesa_ColorSubTableEXT(
        _: GLenum,
        _: GLsizei,
        _: GLsizei,
        _: GLenum,
        _: GLenum,
        _: *const ::core::ffi::c_void,
    ); // swimpl.c
    pub fn _mesa_ColorTableEXT(
        _: GLenum,
        _: GLenum,
        _: GLsizei,
        _: GLenum,
        _: GLenum,
        _: *const ::core::ffi::c_void,
    ); // swimpl.c
    pub fn _mesa_CopyPixels(x: GLint, y: GLint, width: GLsizei, height: GLsizei, type_0: GLenum); // swimpl.c
    pub fn _mesa_CopyTexImage1D(
        target: GLenum,
        level: GLint,
        internalformat: GLenum,
        x: GLint,
        y: GLint,
        width: GLsizei,
        border: GLint,
    ); // swimpl.c
    pub fn _mesa_CopyTexImage2D(
        target: GLenum,
        level: GLint,
        internalformat: GLenum,
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
    ); // swimpl.c
    pub fn _mesa_CopyTexSubImage1D(
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        x: GLint,
        y: GLint,
        width: GLsizei,
    ); // swimpl.c
    pub fn _mesa_CopyTexSubImage2D(
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
    ); // swimpl.c
    pub fn _mesa_CullFace(mode: GLenum); // swimpl.c
    pub fn _mesa_DeleteLists(list: GLuint, range: GLsizei); // swimpl.c
    pub fn _mesa_DeleteTextures(n: GLsizei, textures: *const GLuint); // swimpl.c
    pub fn _mesa_DepthFunc(func: GLenum); // swimpl.c
    pub fn _mesa_DepthMask(flag: GLboolean); // swimpl.c
    pub fn _mesa_DepthRange(zNear: GLclampd, zFar: GLclampd); // swimpl.c
    pub fn _mesa_Disable(cap: GLenum); // swimpl.c
    pub fn _mesa_DisableClientState(array: GLenum); // swimpl.c
    pub fn _mesa_DrawArrays(mode: GLenum, first: GLint, count: GLsizei); // swimpl.c
    pub fn _mesa_DrawBuffer(mode: GLenum); // swimpl.c
    pub fn _mesa_DrawElements(
        mode: GLenum,
        count: GLsizei,
        type_0: GLenum,
        indices: *const ::core::ffi::c_void,
    ); // swimpl.c
    pub fn _mesa_DrawPixels(
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_0: GLenum,
        pixels: *const ::core::ffi::c_void,
    ); // swimpl.c
    pub fn _mesa_EdgeFlag(flag: GLboolean); // swimpl.c
    pub fn _mesa_EdgeFlagPointer(stride: GLsizei, pointer: *const ::core::ffi::c_void); // swimpl.c
    pub fn _mesa_EdgeFlagv(flag: *const GLboolean); // swimpl.c
    pub fn _mesa_Enable(cap: GLenum); // swimpl.c
    pub fn _mesa_EnableClientState(array: GLenum); // swimpl.c
    pub fn _mesa_End(); // swimpl.c
    pub fn _mesa_EndList(); // swimpl.c
    pub fn _mesa_EvalCoord1d(u: GLdouble); // swimpl.c
    pub fn _mesa_EvalCoord1dv(u: *const GLdouble); // swimpl.c
    pub fn _mesa_EvalCoord1f(u: GLfloat); // swimpl.c
    pub fn _mesa_EvalCoord1fv(u: *const GLfloat); // swimpl.c
    pub fn _mesa_EvalCoord2d(u: GLdouble, v: GLdouble); // swimpl.c
    pub fn _mesa_EvalCoord2dv(u: *const GLdouble); // swimpl.c
    pub fn _mesa_EvalCoord2f(u: GLfloat, v: GLfloat); // swimpl.c
    pub fn _mesa_EvalCoord2fv(u: *const GLfloat); // swimpl.c
    pub fn _mesa_EvalMesh1(mode: GLenum, i1: GLint, i2: GLint); // swimpl.c
    pub fn _mesa_EvalMesh2(mode: GLenum, i1: GLint, i2: GLint, j1: GLint, j2: GLint); // swimpl.c
    pub fn _mesa_EvalPoint1(i: GLint); // swimpl.c
    pub fn _mesa_EvalPoint2(i: GLint, j: GLint); // swimpl.c
    pub fn _mesa_FeedbackBuffer(size: GLsizei, type_0: GLenum, buffer: *mut GLfloat); // swimpl.c
    pub fn _mesa_Finish(); // swimpl.c
    pub fn _mesa_Flush(); // swimpl.c
    pub fn _mesa_Fogf(pname: GLenum, param: GLfloat); // swimpl.c
    pub fn _mesa_Fogfv(pname: GLenum, params: *const GLfloat); // swimpl.c
    pub fn _mesa_Fogi(pname: GLenum, param: GLint); // swimpl.c
    pub fn _mesa_Fogiv(pname: GLenum, params: *const GLint); // swimpl.c
    pub fn _mesa_FrontFace(mode: GLenum); // swimpl.c
    pub fn _mesa_Frustum(
        left: GLdouble,
        right: GLdouble,
        bottom: GLdouble,
        top: GLdouble,
        zNear: GLdouble,
        zFar: GLdouble,
    ); // swimpl.c
    pub fn _mesa_GenLists(range: GLsizei) -> GLuint; // swimpl.c
    pub fn _mesa_GenTextures(n: GLsizei, textures: *mut GLuint); // swimpl.c
    pub fn _mesa_GetBooleanv(pname: GLenum, params: *mut GLboolean); // swimpl.c
    pub fn _mesa_GetClipPlane(plane: GLenum, equation: *mut GLdouble); // swimpl.c
    pub fn _mesa_GetColorTableEXT(_: GLenum, _: GLenum, _: GLenum, _: *mut ::core::ffi::c_void); // swimpl.c
    pub fn _mesa_GetColorTableParameterfvEXT(_: GLenum, _: GLenum, _: *mut GLint); // swimpl.c
    pub fn _mesa_GetColorTableParameterivEXT(_: GLenum, _: GLenum, _: *mut GLfloat); // swimpl.c
    pub fn _mesa_GetDoublev(pname: GLenum, params: *mut GLdouble); // swimpl.c
    pub fn _mesa_GetError() -> GLenum; // swimpl.c
    pub fn _mesa_GetFloatv(pname: GLenum, params: *mut GLfloat); // swimpl.c
    pub fn _mesa_GetIntegerv(pname: GLenum, params: *mut GLint); // swimpl.c
    pub fn _mesa_GetLightfv(light: GLenum, pname: GLenum, params: *mut GLfloat); // swimpl.c
    pub fn _mesa_GetLightiv(light: GLenum, pname: GLenum, params: *mut GLint); // swimpl.c
    pub fn _mesa_GetMapdv(target: GLenum, query: GLenum, v: *mut GLdouble); // swimpl.c
    pub fn _mesa_GetMapfv(target: GLenum, query: GLenum, v: *mut GLfloat); // swimpl.c
    pub fn _mesa_GetMapiv(target: GLenum, query: GLenum, v: *mut GLint); // swimpl.c
    pub fn _mesa_GetMaterialfv(face: GLenum, pname: GLenum, params: *mut GLfloat); // swimpl.c
    pub fn _mesa_GetMaterialiv(face: GLenum, pname: GLenum, params: *mut GLint); // swimpl.c
    pub fn _mesa_GetPixelMapfv(map: GLenum, values: *mut GLfloat); // swimpl.c
    pub fn _mesa_GetPixelMapuiv(map: GLenum, values: *mut GLuint); // swimpl.c
    pub fn _mesa_GetPixelMapusv(map: GLenum, values: *mut GLushort); // swimpl.c
    pub fn _mesa_GetPointerv(pname: GLenum, params: *mut *mut ::core::ffi::c_void); // swimpl.c
    pub fn _mesa_GetPolygonStipple(mask: *mut GLubyte); // swimpl.c
    pub fn _mesa_GetString(name: GLenum) -> *const GLubyte; // swimpl.c
    pub fn _mesa_GetTexEnvfv(target: GLenum, pname: GLenum, params: *mut GLfloat); // swimpl.c
    pub fn _mesa_GetTexEnviv(target: GLenum, pname: GLenum, params: *mut GLint); // swimpl.c
    pub fn _mesa_GetTexGendv(coord: GLenum, pname: GLenum, params: *mut GLdouble); // swimpl.c
    pub fn _mesa_GetTexGenfv(coord: GLenum, pname: GLenum, params: *mut GLfloat); // swimpl.c
    pub fn _mesa_GetTexGeniv(coord: GLenum, pname: GLenum, params: *mut GLint); // swimpl.c
    pub fn _mesa_GetTexImage(
        target: GLenum,
        level: GLint,
        format: GLenum,
        type_0: GLenum,
        pixels: *mut ::core::ffi::c_void,
    ); // swimpl.c
    pub fn _mesa_GetTexLevelParameterfv(
        target: GLenum,
        level: GLint,
        pname: GLenum,
        params: *mut GLfloat,
    ); // swimpl.c
    pub fn _mesa_GetTexLevelParameteriv(
        target: GLenum,
        level: GLint,
        pname: GLenum,
        params: *mut GLint,
    ); // swimpl.c
    pub fn _mesa_GetTexParameterfv(target: GLenum, pname: GLenum, params: *mut GLfloat); // swimpl.c
    pub fn _mesa_GetTexParameteriv(target: GLenum, pname: GLenum, params: *mut GLint); // swimpl.c
    pub fn _mesa_Hint(target: GLenum, mode: GLenum); // swimpl.c
    pub fn _mesa_IndexMask(mask: GLuint); // swimpl.c
    pub fn _mesa_IndexPointer(type_0: GLenum, stride: GLsizei, pointer: *const ::core::ffi::c_void); // swimpl.c
    pub fn _mesa_Indexd(c: GLdouble); // swimpl.c
    pub fn _mesa_Indexdv(c: *const GLdouble); // swimpl.c
    pub fn _mesa_Indexf(c: GLfloat); // swimpl.c
    pub fn _mesa_Indexfv(c: *const GLfloat); // swimpl.c
    pub fn _mesa_Indexi(c: GLint); // swimpl.c
    pub fn _mesa_Indexiv(c: *const GLint); // swimpl.c
    pub fn _mesa_Indexs(c: GLshort); // swimpl.c
    pub fn _mesa_Indexsv(c: *const GLshort); // swimpl.c
    pub fn _mesa_Indexub(c: GLubyte); // swimpl.c
    pub fn _mesa_Indexubv(c: *const GLubyte); // swimpl.c
    pub fn _mesa_InitNames(); // swimpl.c
    pub fn _mesa_InterleavedArrays(
        format: GLenum,
        stride: GLsizei,
        pointer: *const ::core::ffi::c_void,
    ); // swimpl.c
    pub fn _mesa_IsEnabled(cap: GLenum) -> GLboolean; // swimpl.c
    pub fn _mesa_IsList(list: GLuint) -> GLboolean; // swimpl.c
    pub fn _mesa_IsTexture(texture: GLuint) -> GLboolean; // swimpl.c
    pub fn _mesa_LightModelf(pname: GLenum, param: GLfloat); // swimpl.c
    pub fn _mesa_LightModelfv(pname: GLenum, params: *const GLfloat); // swimpl.c
    pub fn _mesa_LightModeli(pname: GLenum, param: GLint); // swimpl.c
    pub fn _mesa_LightModeliv(pname: GLenum, params: *const GLint); // swimpl.c
    pub fn _mesa_Lightf(light: GLenum, pname: GLenum, param: GLfloat); // swimpl.c
    pub fn _mesa_Lightfv(light: GLenum, pname: GLenum, params: *const GLfloat); // swimpl.c
    pub fn _mesa_Lighti(light: GLenum, pname: GLenum, param: GLint); // swimpl.c
    pub fn _mesa_Lightiv(light: GLenum, pname: GLenum, params: *const GLint); // swimpl.c
    pub fn _mesa_LineStipple(factor: GLint, pattern: GLushort); // swimpl.c
    pub fn _mesa_LineWidth(width: GLfloat); // swimpl.c
    pub fn _mesa_ListBase(base: GLuint); // swimpl.c
    pub fn _mesa_LoadIdentity(); // swimpl.c
    pub fn _mesa_LoadMatrixd(m: *const GLdouble); // swimpl.c
    pub fn _mesa_LoadMatrixf(m: *const GLfloat); // swimpl.c
    pub fn _mesa_LoadName(name: GLuint); // swimpl.c
    pub fn _mesa_LogicOp(opcode: GLenum); // swimpl.c
    pub fn _mesa_Map1d(
        target: GLenum,
        u1: GLdouble,
        u2: GLdouble,
        stride: GLint,
        order: GLint,
        points: *const GLdouble,
    ); // swimpl.c
    pub fn _mesa_Map1f(
        target: GLenum,
        u1: GLfloat,
        u2: GLfloat,
        stride: GLint,
        order: GLint,
        points: *const GLfloat,
    ); // swimpl.c
    pub fn _mesa_Map2d(
        target: GLenum,
        u1: GLdouble,
        u2: GLdouble,
        ustride: GLint,
        uorder: GLint,
        v1: GLdouble,
        v2: GLdouble,
        vstride: GLint,
        vorder: GLint,
        points: *const GLdouble,
    ); // swimpl.c
    pub fn _mesa_Map2f(
        target: GLenum,
        u1: GLfloat,
        u2: GLfloat,
        ustride: GLint,
        uorder: GLint,
        v1: GLfloat,
        v2: GLfloat,
        vstride: GLint,
        vorder: GLint,
        points: *const GLfloat,
    ); // swimpl.c
    pub fn _mesa_MapGrid1d(un: GLint, u1: GLdouble, u2: GLdouble); // swimpl.c
    pub fn _mesa_MapGrid1f(un: GLint, u1: GLfloat, u2: GLfloat); // swimpl.c
    pub fn _mesa_MapGrid2d(
        un: GLint,
        u1: GLdouble,
        u2: GLdouble,
        vn: GLint,
        v1: GLdouble,
        v2: GLdouble,
    ); // swimpl.c
    pub fn _mesa_MapGrid2f(un: GLint, u1: GLfloat, u2: GLfloat, vn: GLint, v1: GLfloat, v2: GLfloat); // swimpl.c
    pub fn _mesa_Materialf(face: GLenum, pname: GLenum, param: GLfloat); // swimpl.c
    pub fn _mesa_Materialfv(face: GLenum, pname: GLenum, params: *const GLfloat); // swimpl.c
    pub fn _mesa_Materiali(face: GLenum, pname: GLenum, param: GLint); // swimpl.c
    pub fn _mesa_Materialiv(face: GLenum, pname: GLenum, params: *const GLint); // swimpl.c
    pub fn _mesa_MatrixMode(mode: GLenum); // swimpl.c
    pub fn _mesa_MultMatrixd(m: *const GLdouble); // swimpl.c
    pub fn _mesa_MultMatrixf(m: *const GLfloat); // swimpl.c
    pub fn _mesa_NewList(list: GLuint, mode: GLenum); // swimpl.c
    pub fn _mesa_Normal3b(nx: GLbyte, ny: GLbyte, nz: GLbyte); // swimpl.c
    pub fn _mesa_Normal3bv(v: *const GLbyte); // swimpl.c
    pub fn _mesa_Normal3d(nx: GLdouble, ny: GLdouble, nz: GLdouble); // swimpl.c
    pub fn _mesa_Normal3dv(v: *const GLdouble); // swimpl.c
    pub fn _mesa_Normal3f(nx: GLfloat, ny: GLfloat, nz: GLfloat); // swimpl.c
    pub fn _mesa_Normal3fv(v: *const GLfloat); // swimpl.c
    pub fn _mesa_Normal3i(nx: GLint, ny: GLint, nz: GLint); // swimpl.c
    pub fn _mesa_Normal3iv(v: *const GLint); // swimpl.c
    pub fn _mesa_Normal3s(nx: GLshort, ny: GLshort, nz: GLshort); // swimpl.c
    pub fn _mesa_Normal3sv(v: *const GLshort); // swimpl.c
    pub fn _mesa_NormalPointer(type_0: GLenum, stride: GLsizei, pointer: *const ::core::ffi::c_void); // swimpl.c
    pub fn _mesa_Ortho(
        left: GLdouble,
        right: GLdouble,
        bottom: GLdouble,
        top: GLdouble,
        zNear: GLdouble,
        zFar: GLdouble,
    ); // swimpl.c
    pub fn _mesa_PassThrough(token: GLfloat); // swimpl.c
    pub fn _mesa_PixelMapfv(map: GLenum, mapsize: GLint, values: *const GLfloat); // swimpl.c
    pub fn _mesa_PixelMapuiv(map: GLenum, mapsize: GLint, values: *const GLuint); // swimpl.c
    pub fn _mesa_PixelMapusv(map: GLenum, mapsize: GLint, values: *const GLushort); // swimpl.c
    pub fn _mesa_PixelStoref(pname: GLenum, param: GLfloat); // swimpl.c
    pub fn _mesa_PixelStorei(pname: GLenum, param: GLint); // swimpl.c
    pub fn _mesa_PixelTransferf(pname: GLenum, param: GLfloat); // swimpl.c
    pub fn _mesa_PixelTransferi(pname: GLenum, param: GLint); // swimpl.c
    pub fn _mesa_PixelZoom(xfactor: GLfloat, yfactor: GLfloat); // swimpl.c
    pub fn _mesa_PointSize(size: GLfloat); // swimpl.c
    pub fn _mesa_PolygonMode(face: GLenum, mode: GLenum); // swimpl.c
    pub fn _mesa_PolygonOffset(factor: GLfloat, units: GLfloat); // swimpl.c
    pub fn _mesa_PolygonStipple(mask: *const GLubyte); // swimpl.c
    pub fn _mesa_PopAttrib(); // swimpl.c
    pub fn _mesa_PopClientAttrib(); // swimpl.c
    pub fn _mesa_PopMatrix(); // swimpl.c
    pub fn _mesa_PopName(); // swimpl.c
    pub fn _mesa_PrioritizeTextures(n: GLsizei, textures: *const GLuint, priorities: *const GLclampf); // swimpl.c
    pub fn _mesa_PushAttrib(mask: GLbitfield); // swimpl.c
    pub fn _mesa_PushClientAttrib(mask: GLbitfield); // swimpl.c
    pub fn _mesa_PushMatrix(); // swimpl.c
    pub fn _mesa_PushName(name: GLuint); // swimpl.c
    pub fn _mesa_RasterPos2d(x: GLdouble, y: GLdouble); // swimpl.c
    pub fn _mesa_RasterPos2dv(v: *const GLdouble); // swimpl.c
    pub fn _mesa_RasterPos2f(x: GLfloat, y: GLfloat); // swimpl.c
    pub fn _mesa_RasterPos2fv(v: *const GLfloat); // swimpl.c
    pub fn _mesa_RasterPos2i(x: GLint, y: GLint); // swimpl.c
    pub fn _mesa_RasterPos2iv(v: *const GLint); // swimpl.c
    pub fn _mesa_RasterPos2s(x: GLshort, y: GLshort); // swimpl.c
    pub fn _mesa_RasterPos2sv(v: *const GLshort); // swimpl.c
    pub fn _mesa_RasterPos3d(x: GLdouble, y: GLdouble, z: GLdouble); // swimpl.c
    pub fn _mesa_RasterPos3dv(v: *const GLdouble); // swimpl.c
    pub fn _mesa_RasterPos3f(x: GLfloat, y: GLfloat, z: GLfloat); // swimpl.c
    pub fn _mesa_RasterPos3fv(v: *const GLfloat); // swimpl.c
    pub fn _mesa_RasterPos3i(x: GLint, y: GLint, z: GLint); // swimpl.c
    pub fn _mesa_RasterPos3iv(v: *const GLint); // swimpl.c
    pub fn _mesa_RasterPos3s(x: GLshort, y: GLshort, z: GLshort); // swimpl.c
    pub fn _mesa_RasterPos3sv(v: *const GLshort); // swimpl.c
    pub fn _mesa_RasterPos4d(x: GLdouble, y: GLdouble, z: GLdouble, w: GLdouble); // swimpl.c
    pub fn _mesa_RasterPos4dv(v: *const GLdouble); // swimpl.c
    pub fn _mesa_RasterPos4f(x: GLfloat, y: GLfloat, z: GLfloat, w: GLfloat); // swimpl.c
    pub fn _mesa_RasterPos4fv(v: *const GLfloat); // swimpl.c
    pub fn _mesa_RasterPos4i(x: GLint, y: GLint, z: GLint, w: GLint); // swimpl.c
    pub fn _mesa_RasterPos4iv(v: *const GLint); // swimpl.c
    pub fn _mesa_RasterPos4s(x: GLshort, y: GLshort, z: GLshort, w: GLshort); // swimpl.c
    pub fn _mesa_RasterPos4sv(v: *const GLshort); // swimpl.c
    pub fn _mesa_ReadBuffer(mode: GLenum); // swimpl.c
    pub fn _mesa_ReadPixels(
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_0: GLenum,
        pixels: *mut ::core::ffi::c_void,
    ); // swimpl.c
    pub fn _mesa_Rectd(x1: GLdouble, y1: GLdouble, x2: GLdouble, y2: GLdouble); // swimpl.c
    pub fn _mesa_Rectdv(v1: *const GLdouble, v2: *const GLdouble); // swimpl.c
    pub fn _mesa_Rectf(x1: GLfloat, y1: GLfloat, x2: GLfloat, y2: GLfloat); // swimpl.c
    pub fn _mesa_Rectfv(v1: *const GLfloat, v2: *const GLfloat); // swimpl.c
    pub fn _mesa_Recti(x1: GLint, y1: GLint, x2: GLint, y2: GLint); // swimpl.c
    pub fn _mesa_Rectiv(v1: *const GLint, v2: *const GLint); // swimpl.c
    pub fn _mesa_Rects(x1: GLshort, y1: GLshort, x2: GLshort, y2: GLshort); // swimpl.c
    pub fn _mesa_Rectsv(v1: *const GLshort, v2: *const GLshort); // swimpl.c
    pub fn _mesa_RenderMode(mode: GLenum) -> GLint; // swimpl.c
    pub fn _mesa_Rotated(angle: GLdouble, x: GLdouble, y: GLdouble, z: GLdouble); // swimpl.c
    pub fn _mesa_Rotatef(angle: GLfloat, x: GLfloat, y: GLfloat, z: GLfloat); // swimpl.c
    pub fn _mesa_Scaled(x: GLdouble, y: GLdouble, z: GLdouble); // swimpl.c
    pub fn _mesa_Scalef(x: GLfloat, y: GLfloat, z: GLfloat); // swimpl.c
    pub fn _mesa_Scissor(x: GLint, y: GLint, width: GLsizei, height: GLsizei); // swimpl.c
    pub fn _mesa_SelectBuffer(size: GLsizei, buffer: *mut GLuint); // swimpl.c
    pub fn _mesa_ShadeModel(mode: GLenum); // swimpl.c
    pub fn _mesa_StencilFunc(func: GLenum, ref_0: GLint, mask: GLuint); // swimpl.c
    pub fn _mesa_StencilMask(mask: GLuint); // swimpl.c
    pub fn _mesa_StencilOp(fail: GLenum, zfail: GLenum, zpass: GLenum); // swimpl.c
    pub fn _mesa_TexCoord1d(s: GLdouble); // swimpl.c
    pub fn _mesa_TexCoord1dv(v: *const GLdouble); // swimpl.c
    pub fn _mesa_TexCoord1f(s: GLfloat); // swimpl.c
    pub fn _mesa_TexCoord1fv(v: *const GLfloat); // swimpl.c
    pub fn _mesa_TexCoord1i(s: GLint); // swimpl.c
    pub fn _mesa_TexCoord1iv(v: *const GLint); // swimpl.c
    pub fn _mesa_TexCoord1s(s: GLshort); // swimpl.c
    pub fn _mesa_TexCoord1sv(v: *const GLshort); // swimpl.c
    pub fn _mesa_TexCoord2d(s: GLdouble, t: GLdouble); // swimpl.c
    pub fn _mesa_TexCoord2dv(v: *const GLdouble); // swimpl.c
    pub fn _mesa_TexCoord2f(s: GLfloat, t: GLfloat); // swimpl.c
    pub fn _mesa_TexCoord2fv(v: *const GLfloat); // swimpl.c
    pub fn _mesa_TexCoord2i(s: GLint, t: GLint); // swimpl.c
    pub fn _mesa_TexCoord2iv(v: *const GLint); // swimpl.c
    pub fn _mesa_TexCoord2s(s: GLshort, t: GLshort); // swimpl.c
    pub fn _mesa_TexCoord2sv(v: *const GLshort); // swimpl.c
    pub fn _mesa_TexCoord3d(s: GLdouble, t: GLdouble, r: GLdouble); // swimpl.c
    pub fn _mesa_TexCoord3dv(v: *const GLdouble); // swimpl.c
    pub fn _mesa_TexCoord3f(s: GLfloat, t: GLfloat, r: GLfloat); // swimpl.c
    pub fn _mesa_TexCoord3fv(v: *const GLfloat); // swimpl.c
    pub fn _mesa_TexCoord3i(s: GLint, t: GLint, r: GLint); // swimpl.c
    pub fn _mesa_TexCoord3iv(v: *const GLint); // swimpl.c
    pub fn _mesa_TexCoord3s(s: GLshort, t: GLshort, r: GLshort); // swimpl.c
    pub fn _mesa_TexCoord3sv(v: *const GLshort); // swimpl.c
    pub fn _mesa_TexCoord4d(s: GLdouble, t: GLdouble, r: GLdouble, q: GLdouble); // swimpl.c
    pub fn _mesa_TexCoord4dv(v: *const GLdouble); // swimpl.c
    pub fn _mesa_TexCoord4f(s: GLfloat, t: GLfloat, r: GLfloat, q: GLfloat); // swimpl.c
    pub fn _mesa_TexCoord4fv(v: *const GLfloat); // swimpl.c
    pub fn _mesa_TexCoord4i(s: GLint, t: GLint, r: GLint, q: GLint); // swimpl.c
    pub fn _mesa_TexCoord4iv(v: *const GLint); // swimpl.c
    pub fn _mesa_TexCoord4s(s: GLshort, t: GLshort, r: GLshort, q: GLshort); // swimpl.c
    pub fn _mesa_TexCoord4sv(v: *const GLshort); // swimpl.c
    pub fn _mesa_TexCoordPointer(
        size: GLint,
        type_0: GLenum,
        stride: GLsizei,
        pointer: *const ::core::ffi::c_void,
    ); // swimpl.c
    pub fn _mesa_TexEnvf(target: GLenum, pname: GLenum, param: GLfloat); // swimpl.c
    pub fn _mesa_TexEnvfv(target: GLenum, pname: GLenum, params: *const GLfloat); // swimpl.c
    pub fn _mesa_TexEnvi(target: GLenum, pname: GLenum, param: GLint); // swimpl.c
    pub fn _mesa_TexEnviv(target: GLenum, pname: GLenum, params: *const GLint); // swimpl.c
    pub fn _mesa_TexGend(coord: GLenum, pname: GLenum, param: GLdouble); // swimpl.c
    pub fn _mesa_TexGendv(coord: GLenum, pname: GLenum, params: *const GLdouble); // swimpl.c
    pub fn _mesa_TexGenf(coord: GLenum, pname: GLenum, param: GLfloat); // swimpl.c
    pub fn _mesa_TexGenfv(coord: GLenum, pname: GLenum, params: *const GLfloat); // swimpl.c
    pub fn _mesa_TexGeni(coord: GLenum, pname: GLenum, param: GLint); // swimpl.c
    pub fn _mesa_TexGeniv(coord: GLenum, pname: GLenum, params: *const GLint); // swimpl.c
    pub fn _mesa_TexImage1D(
        target: GLenum,
        level: GLint,
        internalformat: GLint,
        width: GLsizei,
        border: GLint,
        format: GLenum,
        type_0: GLenum,
        pixels: *const ::core::ffi::c_void,
    ); // swimpl.c
    pub fn _mesa_TexImage2D(
        target: GLenum,
        level: GLint,
        internalformat: GLint,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
        format: GLenum,
        type_0: GLenum,
        pixels: *const ::core::ffi::c_void,
    ); // swimpl.c
    pub fn _mesa_TexParameterf(target: GLenum, pname: GLenum, param: GLfloat); // swimpl.c
    pub fn _mesa_TexParameterfv(target: GLenum, pname: GLenum, params: *const GLfloat); // swimpl.c
    pub fn _mesa_TexParameteri(target: GLenum, pname: GLenum, param: GLint); // swimpl.c
    pub fn _mesa_TexParameteriv(target: GLenum, pname: GLenum, params: *const GLint); // swimpl.c
    pub fn _mesa_TexSubImage1D(
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        width: GLsizei,
        format: GLenum,
        type_0: GLenum,
        pixels: *const ::core::ffi::c_void,
    ); // swimpl.c
    pub fn _mesa_TexSubImage2D(
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_0: GLenum,
        pixels: *const ::core::ffi::c_void,
    ); // swimpl.c
    pub fn _mesa_Translated(x: GLdouble, y: GLdouble, z: GLdouble); // swimpl.c
    pub fn _mesa_Translatef(x: GLfloat, y: GLfloat, z: GLfloat); // swimpl.c
    pub fn _mesa_Vertex2d(x: GLdouble, y: GLdouble); // swimpl.c
    pub fn _mesa_Vertex2dv(v: *const GLdouble); // swimpl.c
    pub fn _mesa_Vertex2f(x: GLfloat, y: GLfloat); // swimpl.c
    pub fn _mesa_Vertex2fv(v: *const GLfloat); // swimpl.c
    pub fn _mesa_Vertex2i(x: GLint, y: GLint); // swimpl.c
    pub fn _mesa_Vertex2iv(v: *const GLint); // swimpl.c
    pub fn _mesa_Vertex2s(x: GLshort, y: GLshort); // swimpl.c
    pub fn _mesa_Vertex2sv(v: *const GLshort); // swimpl.c
    pub fn _mesa_Vertex3d(x: GLdouble, y: GLdouble, z: GLdouble); // swimpl.c
    pub fn _mesa_Vertex3dv(v: *const GLdouble); // swimpl.c
    pub fn _mesa_Vertex3f(x: GLfloat, y: GLfloat, z: GLfloat); // swimpl.c
    pub fn _mesa_Vertex3fv(v: *const GLfloat); // swimpl.c
    pub fn _mesa_Vertex3i(x: GLint, y: GLint, z: GLint); // swimpl.c
    pub fn _mesa_Vertex3iv(v: *const GLint); // swimpl.c
    pub fn _mesa_Vertex3s(x: GLshort, y: GLshort, z: GLshort); // swimpl.c
    pub fn _mesa_Vertex3sv(v: *const GLshort); // swimpl.c
    pub fn _mesa_Vertex4d(x: GLdouble, y: GLdouble, z: GLdouble, w: GLdouble); // swimpl.c
    pub fn _mesa_Vertex4dv(v: *const GLdouble); // swimpl.c
    pub fn _mesa_Vertex4f(x: GLfloat, y: GLfloat, z: GLfloat, w: GLfloat); // swimpl.c
    pub fn _mesa_Vertex4fv(v: *const GLfloat); // swimpl.c
    pub fn _mesa_Vertex4i(x: GLint, y: GLint, z: GLint, w: GLint); // swimpl.c
    pub fn _mesa_Vertex4iv(v: *const GLint); // swimpl.c
    pub fn _mesa_Vertex4s(x: GLshort, y: GLshort, z: GLshort, w: GLshort); // swimpl.c
    pub fn _mesa_Vertex4sv(v: *const GLshort); // swimpl.c
    pub fn _mesa_VertexPointer(
        size: GLint,
        type_0: GLenum,
        stride: GLsizei,
        pointer: *const ::core::ffi::c_void,
    ); // swimpl.c
    pub fn _mesa_Viewport(x: GLint, y: GLint, width: GLsizei, height: GLsizei); // swimpl.c
}
pub use crate::prelude::GetObjectA as GetObject; // sdk alias
pub use crate::prelude::SetWindowsHookExA as SetWindowsHookEx; // sdk alias
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_opengl32: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x6f\x70\x65\x6e\x67\x6c\x33\x32\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_opengl32 };
// ==== end wine debug channels ====

// wgl.c and wgl_font.c say WINE_DEFAULT_DEBUG_CHANNEL(wgl), swimpl.c and icdload.c say
// (opengl32). Those two TUs shadow `__wine_dbch___default` with a file-local pointing here.
pub static mut __wine_dbch_wgl: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"wgl\0\0\0\0\0\0\0\0\0\0\0\0") },
};

// ================= wine/debug.h ================================================================
// TRACE/WARN/FIXME/ERR: test the class bit on the channel, then log -- the same shape
// __WINE_DPRINTF expands to. Macros, never out-of-line fns (field manual §9.65). The channel
// is resolved at the CALL SITE, so a TU that shadows `__wine_dbch___default` logs to its own.
// The format is either a str literal (NUL appended here) or an already-NUL-terminated pointer.
#[macro_export]
macro_rules! __WINE_DPRINTF {
    ($cls:expr, $ch:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__WINE_DPRINTF!($cls, $ch, concat!($fmt, "\0").as_ptr() $(, $arg)*)
    };
    ($cls:expr, $ch:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        let __dbch: *mut $crate::prelude::__wine_debug_channel = $ch;
        let __dbcl: $crate::prelude::__wine_debug_class = $cls;
        if ((*__dbch).flags as $crate::prelude::c_int & (1 << (__dbcl as $crate::prelude::c_int))) != 0 {
            $crate::prelude::ros_dbg_log(
                __dbcl,
                __dbch,
                concat!(file!(), "\0").as_ptr() as *const $crate::prelude::c_char,
                b"\0".as_ptr() as *const $crate::prelude::c_char,
                line!() as $crate::prelude::c_int,
                ($fmt) as *const $crate::prelude::c_char
                $(, $arg)*
            );
        }
    }};
}
#[macro_export]
macro_rules! TRACE { ($($t:tt)*) => { $crate::__WINE_DPRINTF!($crate::prelude::__WINE_DBCL_TRACE, __wine_dbch___default, $($t)*) }; }
#[macro_export]
macro_rules! WARN { ($($t:tt)*) => { $crate::__WINE_DPRINTF!($crate::prelude::__WINE_DBCL_WARN, __wine_dbch___default, $($t)*) }; }
#[macro_export]
macro_rules! FIXME { ($($t:tt)*) => { $crate::__WINE_DPRINTF!($crate::prelude::__WINE_DBCL_FIXME, __wine_dbch___default, $($t)*) }; }
#[macro_export]
macro_rules! ERR { ($($t:tt)*) => { $crate::__WINE_DPRINTF!($crate::prelude::__WINE_DBCL_ERR, __wine_dbch___default, $($t)*) }; }
/// `TRACE_ON(ch)`: is the TRACE class enabled on channel `ch`.
#[macro_export]
macro_rules! TRACE_ON {
    (wgl) => { ((*core::ptr::addr_of!($crate::prelude::__wine_dbch_wgl)).flags as $crate::prelude::c_int
        & (1 << ($crate::prelude::__WINE_DBCL_TRACE as $crate::prelude::c_int))) != 0 };
    (opengl32) => { ((*core::ptr::addr_of!($crate::prelude::__wine_dbch_opengl32)).flags as $crate::prelude::c_int
        & (1 << ($crate::prelude::__WINE_DBCL_TRACE as $crate::prelude::c_int))) != 0 };
}
pub use crate::{ERR, FIXME, TRACE, TRACE_ON, WARN, __WINE_DPRINTF};

/// wingdi.h `#define PALETTEINDEX(i) ((COLORREF)(0x01000000 | (DWORD)(WORD)(i)))`.
#[macro_export]
macro_rules! PALETTEINDEX {
    ($i:expr $(,)?) => { ((0x01000000 as $crate::prelude::DWORD) | (($i) as $crate::prelude::WORD as $crate::prelude::DWORD)) as $crate::prelude::COLORREF };
}
pub use crate::PALETTEINDEX;

// pseh2.h under DUMMY_PSEH (sdk/lib/pseh/dummy.c), which every arch builds C with here:
// `_SEH2_TRY { A } _SEH2_EXCEPT(f) { B } _SEH2_END` is `A;` then `if (_SEH2_Volatile0 || ..) { B }`.
extern "C" {
    pub static _SEH2_Volatile0: c_int;
    pub static _SEH2_VolatileExceptionCode: c_int;
}

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/opengl/opengl32/opengl32.h:109: header-inline `IntGetCurrentDC`
#[allow(dead_code)]
#[inline]
pub unsafe fn IntGetCurrentDC() -> HDC {
    ((*NtCurrentTeb()).glReserved2).cast()
}
// dll/opengl/opengl32/opengl32.h:137: header-inline `IntGetCurrentDHGLRC`
#[allow(dead_code)]
#[inline]
pub unsafe fn IntGetCurrentDHGLRC() -> DHGLRC {
    let ctx: *mut wgl_context = IntGetCurrentRC() as *mut wgl_context;
    if ctx.is_null() {
        return core::ptr::null_mut();
    }
    (*ctx).dhglrc
}
// dll/opengl/opengl32/opengl32.h:116: header-inline `IntGetCurrentDcData`
#[allow(dead_code)]
#[inline]
pub unsafe fn IntGetCurrentDcData() -> *mut wgl_dc_data {
    ((*NtCurrentTeb()).glSectionInfo).cast()
}
// dll/opengl/opengl32/opengl32.h:77: header-inline `IntGetCurrentDispatchTable`
#[allow(dead_code)]
#[inline]
pub unsafe fn IntGetCurrentDispatchTable() -> *const GLDISPATCHTABLE {
    (*NtCurrentTeb()).glTable as *const GLDISPATCHTABLE
}
// dll/opengl/opengl32/opengl32.h:130: header-inline `IntGetCurrentICDPrivate`
#[allow(dead_code)]
#[inline]
pub unsafe fn IntGetCurrentICDPrivate() -> *mut core::ffi::c_void {
    (*NtCurrentTeb()).glContext as *mut core::ffi::c_void
}
// dll/opengl/opengl32/opengl32.h:102: header-inline `IntGetCurrentRC`
#[allow(dead_code)]
#[inline]
pub unsafe fn IntGetCurrentRC() -> HGLRC {
    ((*NtCurrentTeb()).glCurrentRC).cast()
}
// dll/opengl/opengl32/opengl32.h:91: header-inline `IntMakeCurrent`
#[allow(dead_code)]
#[inline]
pub unsafe fn IntMakeCurrent(hglrc: HGLRC, hdc: HDC, dc_data: *mut wgl_dc_data) {
    let CurrentTeb = NtCurrentTeb();
    (*CurrentTeb).glCurrentRC = (hglrc).cast();
    (*CurrentTeb).glReserved2 = (hdc).cast();
    (*CurrentTeb).glSectionInfo = (dc_data).cast();
}
// dll/opengl/opengl32/opengl32.h:84: header-inline `IntSetCurrentDispatchTable`
#[allow(dead_code)]
#[inline]
pub unsafe fn IntSetCurrentDispatchTable(table: *const GLDISPATCHTABLE) {
    (*NtCurrentTeb()).glTable = table as *mut core::ffi::c_void;
}
// dll/opengl/opengl32/opengl32.h:123: header-inline `IntSetCurrentICDPrivate`
#[allow(dead_code)]
#[inline]
pub unsafe fn IntSetCurrentICDPrivate(value: *mut core::ffi::c_void) {
    (*NtCurrentTeb()).glContext = value;
}
// dll/opengl/opengl32/swimpl.c:16: #define WIDTH_BYTES_ALIGN32(cx, bpp) ((((cx) * (bpp) + 31) & ~31) >> 3)
#[allow(unused_macros)]
macro_rules! WIDTH_BYTES_ALIGN32 {
    ($cx:expr, $bpp:expr $(,)?) => {
        ((((($cx) * ($bpp) + 31) & !31) >> 3))
    };
}
#[allow(unused_imports)]
pub(crate) use WIDTH_BYTES_ALIGN32;
// ==== end port macros ====
