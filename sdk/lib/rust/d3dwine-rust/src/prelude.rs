//! d3dwine-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
// wine/wined3d.h's static inline helpers (sdk/lib/rust/wined3d-h), shared with ddraw/d3d8/d3d9:
// the integer ones by name, the struct-bound ones expanded here against this crate's island.
pub use wined3d_h::{wined3d_bit_scan, wined3d_bound_range, wined3d_log2i};
wined3d_h::wined3d_inlines!();
// wine/rbtree.h (sdk/lib/rust/winerbtree-rust), expanded against this island's rb_entry/rb_tree.
winerbtree_rust::wine_rb_inlines!();
pub use winerbtree_rust::{RB_ENTRY_VALUE, WINE_RB_ENTRY_VALUE};
// <stdint.h>/<stddef.h> names: no island allowlists those headers (ddraw used 26 of them).
// Fixed-width, so identical on every target; an island that does carry one shadows nothing
// (a glob import never conflicts with an item the prelude defines).
pub type int8_t = i8;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type intptr_t = isize;
pub type uintptr_t = usize;
pub type ptrdiff_t = isize;
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// reactos/debug.h DPRINT/UNIMPLEMENTED — no-op MACROS, never out-of-line fns
// (field manual §9.65: an out-of-line no-op is an optimization barrier). debug_macros.py
// rewrites C's bare `UNIMPLEMENTED;` to `UNIMPLEMENTED!()`, so the prelude must have it.
// DPRINT1 PRINTS in this DBG=1 tree, kernel and user mode alike: it comes from
// win32macros-rust with the other macros. DPRINT is off, but its arguments still
// type-check (DBG_TYPECHECK: a never-called closure), so a stale one is a compile error
// rather than silently swallowed tokens.
#[macro_export] macro_rules! DPRINT {
    ($($arg:tt)*) => { ::win32macros_rust::DBG_TYPECHECK!($($arg)*) };
}
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

// <math.h>: the float forms are static inlines over the double CRT on i386 (msvcrt exports no
// `powf` there), so they are written that way on every target -- same values, one definition.
// <stdlib.h>/<string.h>/<ctype.h>: msvcrt imports; `putenv`/`stricmp` resolve through oldnames.
pub type double = f64;
pub const INFINITY: f32 = f32::INFINITY;
pub const NAN: f32 = f32::NAN;
pub const FLT_MAX: f32 = f32::MAX;
extern "C" {
    pub fn pow(x: f64, y: f64) -> f64;
    pub fn sqrt(x: f64) -> f64;
    pub fn cos(x: f64) -> f64;
    pub fn sin(x: f64) -> f64;
    pub fn exp(x: f64) -> f64;
    pub fn putenv(s: *const c_char) -> c_int;
    pub fn stricmp(a: *const c_char, b: *const c_char) -> c_int;
    pub fn isprint(c: c_int) -> c_int;
}
#[inline(always)]
pub unsafe fn powf(x: f32, y: f32) -> f32 { pow(x as f64, y as f64) as f32 }
#[inline(always)]
pub unsafe fn sqrtf(x: f32) -> f32 { sqrt(x as f64) as f32 }
#[inline(always)]
pub unsafe fn cosf(x: f32) -> f32 { cos(x as f64) as f32 }
#[inline(always)]
pub unsafe fn sinf(x: f32) -> f32 { sin(x as f64) as f32 }
#[inline(always)]
pub unsafe fn expf(x: f32) -> f32 { exp(x as f64) as f32 }

// winnt.h / wine's headers: inlines no island carries.
#[inline(always)]
pub fn YieldProcessor() {
    core::hint::spin_loop() // pause (x86) / yield (arm64), as the intrinsic
}
/// wined3d_private.h's `InterlockedExchangeAddSizeT`: InterlockedExchangeAdd64 on 64-bit,
/// InterlockedExchangeAdd on i386 -- one pointer-width atomic add, returning the old value.
#[inline(always)]
pub unsafe fn InterlockedExchangeAddSizeT(dest: *mut isize, value: isize) -> isize {
    (*(dest as *const core::sync::atomic::AtomicIsize))
        .fetch_add(value, core::sync::atomic::Ordering::SeqCst)
}
#[inline(always)]
pub fn copysignf(x: f32, y: f32) -> f32 {
    f32::from_bits((x.to_bits() & 0x7fff_ffff) | (y.to_bits() & 0x8000_0000))
}
/// wine/debug.h's `wine_dbgstr_point`.
pub unsafe fn wine_dbgstr_point(pt: *const POINT) -> *const c_char {
    if pt.is_null() {
        return c"(null)".as_ptr();
    }
    wine_dbg_sprintf(c"(%d,%d)".as_ptr(), (*pt).x as c_int, (*pt).y as c_int)
}
extern "C" {
    // msvcrt has no `_recalloc` export here: <malloc.h> makes it dllimport, satisfied by
    // msvcrtex's recalloc.c `__imp__recalloc` pointer to its own `recalloc`. A plain call
    // links that function directly.
    #[link_name = "recalloc"]
    pub fn _recalloc(p: *mut c_void, n: usize, size: usize) -> *mut c_void;
    // Windows' va_list is a `char*` on every target, so this declares on stable.
    #[link_name = "_vsnprintf"]
    pub fn vsnprintf(buf: *mut c_char, n: usize, fmt: *const c_char, ap: va_list) -> c_int;
}

// wined3d_private.h: `static const struct color_fixup_desc COLOR_FIXUP_IDENTITY` -- internal
// linkage, a private copy per TU in C, so never exported. One crate-wide definition (it is
// constant); the island's extern declaration of it names no symbol anything defines.
pub static COLOR_FIXUP_IDENTITY: color_fixup_desc =
    unsafe { ::core::mem::transmute::<[u8; 2], color_fixup_desc>([0x64, 0xa8]) };

// wined3d_main.c includes <initguid.h> before unknwn.h's IID_IUnknown, which vkd3d (linked
// in) references; d3dwine links no uuid library. The generated GUID block refuses it only
// because the tree spells the value more than once -- COM fixes it.
#[no_mangle]
pub static IID_IUnknown: GUID = GUID {
    Data1: 0x00000000,
    Data2: 0x0000,
    Data3: 0x0000,
    Data4: [0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};

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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_TYPECHECK, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedCompareExchangePointer, InterlockedDecrement, InterlockedDecrement64, InterlockedExchange, InterlockedExchangePointer, InterlockedIncrement, InterlockedIncrement64, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlSecureZeroMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, SecureZeroMemory, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_ERR_, WINE_FIXME, WINE_FIXME_, WINE_TRACE, WINE_TRACE_, WINE_WARN, WINE_WARN_, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _byteswap_uint64, _byteswap_ulong, _byteswap_ushort, _countof, debugstr_a, debugstr_an, debugstr_fourcc, debugstr_guid, debugstr_variant, debugstr_vt, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, lrintf, strlenW, tolowerW, wide_bytes, wine_dbgstr_a, wine_dbgstr_fourcc, wine_dbgstr_guid, wine_dbgstr_longlong, wine_dbgstr_rect, wine_dbgstr_variant, wine_dbgstr_vt, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const ARG0: i32 = 4; // clang, utils.c
pub const ARG1: i32 = 1; // clang, utils.c
pub const ARG2: i32 = 2; // clang, utils.c
pub const STATE_BASEVERTEXINDEX: i32 = 0x17D; // clang, adapter_vk.c
pub const STATE_BLEND: i32 = 0x180; // clang, adapter_vk.c
pub const STATE_BLEND_FACTOR: i32 = 0x181; // clang, adapter_vk.c
pub const STATE_COMPUTE_CONSTANT_BUFFER: i32 = 0x186; // clang, context_vk.c
pub const STATE_COMPUTE_HIGHEST: i32 = 0x188; // clang, adapter_gl.c
pub const STATE_COMPUTE_OFFSET: i32 = 0x185; // clang, context.c
pub const STATE_COMPUTE_SHADER: i32 = 0x185; // clang, context_vk.c
pub const STATE_COMPUTE_SHADER_RESOURCE_BINDING: i32 = 0x187; // clang, adapter_vk.c
pub const STATE_COMPUTE_UNORDERED_ACCESS_VIEW_BINDING: i32 = 0x188; // clang, adapter_vk.c
pub const STATE_DEPTH_BOUNDS: i32 = 0x17C; // clang, adapter_vk.c
pub const STATE_DEPTH_STENCIL: i32 = 0x183; // clang, adapter_vk.c
pub const STATE_FRAMEBUFFER: i32 = 0x17E; // clang, adapter_vk.c
pub const STATE_GRAPHICS_SHADER_RESOURCE_BINDING: i32 = 0x16C; // clang, adapter_vk.c
pub const STATE_GRAPHICS_UNORDERED_ACCESS_VIEW_BINDING: i32 = 0x16D; // clang, adapter_vk.c
pub const STATE_HIGHEST: i32 = 0x188; // clang, context.c
pub const STATE_INDEXBUFFER: i32 = 0x16F; // clang, adapter_vk.c
pub const STATE_RASTERIZER: i32 = 0x17B; // clang, adapter_vk.c
pub const STATE_SAMPLE_MASK: i32 = 0x182; // clang, adapter_vk.c
pub const STATE_SCISSORRECT: i32 = 0x172; // clang, adapter_vk.c
pub const STATE_STENCIL_REF: i32 = 0x184; // clang, adapter_vk.c
pub const STATE_STREAMSRC: i32 = 0x16E; // clang, adapter_vk.c
pub const STATE_STREAM_OUTPUT: i32 = 0x17F; // clang, adapter_vk.c
pub const STATE_VDECL: i32 = 0x170; // clang, adapter_vk.c
pub const STATE_VIEWPORT: i32 = 0x171; // clang, adapter_vk.c
pub const STATUS_GRAPHICS_VIDPN_SOURCE_IN_USE: NTSTATUS = 0xC01E0342_u32 as NTSTATUS; // clang, directx.c
pub const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000000D_u32 as NTSTATUS; // clang, directx.c
pub const STATUS_PROCEDURE_NOT_FOUND: NTSTATUS = 0xC000007A_u32 as NTSTATUS; // clang, directx.c
pub const STATUS_SUCCESS: NTSTATUS = 0; // clang, directx.c
pub const UINT64_MAX: u64 = 0xFFFFFFFFFFFFFFFF; // clang, context_vk.c
pub const VKD3D_SHADER_NO_SWIZZLE: i32 = 0x3020100; // clang, shader_spirv.c
pub const VK_API_VERSION_1_0: uint32_t = 0x400000; // clang, adapter_vk.c
pub const VK_API_VERSION_1_1: uint32_t = 0x401000; // clang, adapter_vk.c
pub const VK_API_VERSION_1_2: uint32_t = 0x402000; // clang, adapter_vk.c
pub const VK_API_VERSION_1_3: uint32_t = 0x403000; // clang, adapter_vk.c
pub const WINED3D_ALPHA_TO_COVERAGE_DISABLE: DWORD = 0x304D3241; // clang, stateblock.c
pub const WINED3D_ALPHA_TO_COVERAGE_ENABLE: DWORD = 0x314D3241; // clang, stateblock.c
pub const WINED3D_HIGHEST_TEXTURE_STATE: i32 = 0x11; // clang, context_gl.c
pub const WINED3D_HIGHEST_TRANSFORM_STATE: u32 = 0x1FF; // clang, stateblock.c
pub const WINEHIGHEST_RENDER_STATE: i32 = 0xD1; // clang, ffp_gl.c
// ==== end port facts ====

// ==== port variadic: C-variadic entry points, trampolines in the module's rust_varargs_shim.c (generated -- `rustconv port integrate`) ====
extern "C" {
    pub fn declare_in_varying(gl_info: *const wined3d_gl_info, buffer: *mut wined3d_string_buffer, flat: BOOL, format: *const ::core::ffi::c_char, ...);
    pub fn declare_out_varying(gl_info: *const wined3d_gl_info, buffer: *mut wined3d_string_buffer, flat: BOOL, format: *const i8, ...);
    pub fn shader_addline(buffer: *mut wined3d_string_buffer, format: *const i8, ...) -> i32;
    pub fn shader_glsl_gen_sample_code(ins: *const wined3d_shader_instruction, sampler_bind_idx: ::core::ffi::c_uint, sample_function: *const glsl_sample_function, swizzle: DWORD, dx: *const ::core::ffi::c_char, dy: *const ::core::ffi::c_char, bias: *const ::core::ffi::c_char, offset: *const wined3d_shader_texel_offset, coord_reg_fmt: *const ::core::ffi::c_char, ...);
    pub fn string_buffer_sprintf(buffer: *mut wined3d_string_buffer, format: *const i8, ...);
}
// ==== end port variadic ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const APPLE_FENCE: wined3d_gl_extension = wined3d_gl_extension_APPLE_FENCE; // island enum // gen:05ea2772f414
pub const APPLE_FLOAT_PIXELS: wined3d_gl_extension = wined3d_gl_extension_APPLE_FLOAT_PIXELS; // island enum // gen:3724d9786ef2
pub const APPLE_FLUSH_BUFFER_RANGE: wined3d_gl_extension = wined3d_gl_extension_APPLE_FLUSH_BUFFER_RANGE; // island enum // gen:abf2ef630979
pub const APPLE_FLUSH_RENDER: wined3d_gl_extension = wined3d_gl_extension_APPLE_FLUSH_RENDER; // island enum // gen:cb8336c0b12e
pub const APPLE_RGB_422: wined3d_gl_extension = wined3d_gl_extension_APPLE_RGB_422; // island enum // gen:e481fb9a410c
pub const APPLE_YCBCR_422: wined3d_gl_extension = wined3d_gl_extension_APPLE_YCBCR_422; // island enum // gen:ad19a53ab61f
pub const ARB_BASE_INSTANCE: wined3d_gl_extension = wined3d_gl_extension_ARB_BASE_INSTANCE; // island enum // gen:5c1ac8150632
pub const ARB_BLEND_FUNC_EXTENDED: wined3d_gl_extension = wined3d_gl_extension_ARB_BLEND_FUNC_EXTENDED; // island enum // gen:82153d678450
pub const ARB_BUFFER_STORAGE: wined3d_gl_extension = wined3d_gl_extension_ARB_BUFFER_STORAGE; // island enum // gen:c41f7872d144
pub const ARB_CLEAR_BUFFER_OBJECT: wined3d_gl_extension = wined3d_gl_extension_ARB_CLEAR_BUFFER_OBJECT; // island enum // gen:9bbbd5b06f89
pub const ARB_CLEAR_TEXTURE: wined3d_gl_extension = wined3d_gl_extension_ARB_CLEAR_TEXTURE; // island enum // gen:0f2bacc3ee95
pub const ARB_CLIP_CONTROL: wined3d_gl_extension = wined3d_gl_extension_ARB_CLIP_CONTROL; // island enum // gen:9da0aba77824
pub const ARB_COLOR_BUFFER_FLOAT: wined3d_gl_extension = wined3d_gl_extension_ARB_COLOR_BUFFER_FLOAT; // island enum // gen:def1ad1396da
pub const ARB_COMPUTE_SHADER: wined3d_gl_extension = wined3d_gl_extension_ARB_COMPUTE_SHADER; // island enum // gen:8e3907cea967
pub const ARB_CONSERVATIVE_DEPTH: wined3d_gl_extension = wined3d_gl_extension_ARB_CONSERVATIVE_DEPTH; // island enum // gen:42fcce91a36c
pub const ARB_COPY_BUFFER: wined3d_gl_extension = wined3d_gl_extension_ARB_COPY_BUFFER; // island enum // gen:108c111c946e
pub const ARB_COPY_IMAGE: wined3d_gl_extension = wined3d_gl_extension_ARB_COPY_IMAGE; // island enum // gen:b2a1a4973cd3
pub const ARB_CULL_DISTANCE: wined3d_gl_extension = wined3d_gl_extension_ARB_CULL_DISTANCE; // island enum // gen:e4a73d1e10c2
pub const ARB_DEBUG_OUTPUT: wined3d_gl_extension = wined3d_gl_extension_ARB_DEBUG_OUTPUT; // island enum // gen:feee49977342
pub const ARB_DEPTH_BUFFER_FLOAT: wined3d_gl_extension = wined3d_gl_extension_ARB_DEPTH_BUFFER_FLOAT; // island enum // gen:21de515a2920
pub const ARB_DEPTH_CLAMP: wined3d_gl_extension = wined3d_gl_extension_ARB_DEPTH_CLAMP; // island enum // gen:ad89ed508610
pub const ARB_DEPTH_TEXTURE: wined3d_gl_extension = wined3d_gl_extension_ARB_DEPTH_TEXTURE; // island enum // gen:79686cbc4e1f
pub const ARB_DERIVATIVE_CONTROL: wined3d_gl_extension = wined3d_gl_extension_ARB_DERIVATIVE_CONTROL; // island enum // gen:4245815b48a2
pub const ARB_DRAW_BUFFERS: wined3d_gl_extension = wined3d_gl_extension_ARB_DRAW_BUFFERS; // island enum // gen:44f428083a94
pub const ARB_DRAW_BUFFERS_BLEND: wined3d_gl_extension = wined3d_gl_extension_ARB_DRAW_BUFFERS_BLEND; // island enum // gen:b666fb87da32
pub const ARB_DRAW_ELEMENTS_BASE_VERTEX: wined3d_gl_extension = wined3d_gl_extension_ARB_DRAW_ELEMENTS_BASE_VERTEX; // island enum // gen:73d92ff6adb3
pub const ARB_DRAW_INDIRECT: wined3d_gl_extension = wined3d_gl_extension_ARB_DRAW_INDIRECT; // island enum // gen:06f61739f6c2
pub const ARB_DRAW_INSTANCED: wined3d_gl_extension = wined3d_gl_extension_ARB_DRAW_INSTANCED; // island enum // gen:9209959afe31
pub const ARB_ES2_COMPATIBILITY: wined3d_gl_extension = wined3d_gl_extension_ARB_ES2_COMPATIBILITY; // island enum // gen:22c2117cfb59
pub const ARB_ES3_COMPATIBILITY: wined3d_gl_extension = wined3d_gl_extension_ARB_ES3_COMPATIBILITY; // island enum // gen:2992649f9362
pub const ARB_EXPLICIT_ATTRIB_LOCATION: wined3d_gl_extension = wined3d_gl_extension_ARB_EXPLICIT_ATTRIB_LOCATION; // island enum // gen:ac1ce00ddfaa
pub const ARB_FRAGMENT_COORD_CONVENTIONS: wined3d_gl_extension = wined3d_gl_extension_ARB_FRAGMENT_COORD_CONVENTIONS; // island enum // gen:11d94f7e52bc
pub const ARB_FRAGMENT_LAYER_VIEWPORT: wined3d_gl_extension = wined3d_gl_extension_ARB_FRAGMENT_LAYER_VIEWPORT; // island enum // gen:0c80f4cb882a
pub const ARB_FRAGMENT_PROGRAM: wined3d_gl_extension = wined3d_gl_extension_ARB_FRAGMENT_PROGRAM; // island enum // gen:029ae28a9ea8
pub const ARB_FRAGMENT_PROGRAM_SHADOW: wined3d_gl_extension = wined3d_gl_extension_ARB_FRAGMENT_PROGRAM_SHADOW; // island enum // gen:258d879b8d37
pub const ARB_FRAGMENT_SHADER: wined3d_gl_extension = wined3d_gl_extension_ARB_FRAGMENT_SHADER; // island enum // gen:02ca05d745e3
pub const ARB_FRAMEBUFFER_NO_ATTACHMENTS: wined3d_gl_extension = wined3d_gl_extension_ARB_FRAMEBUFFER_NO_ATTACHMENTS; // island enum // gen:847bbde7fa8a
pub const ARB_FRAMEBUFFER_OBJECT: wined3d_gl_extension = wined3d_gl_extension_ARB_FRAMEBUFFER_OBJECT; // island enum // gen:6ec8630b466f
pub const ARB_FRAMEBUFFER_SRGB: wined3d_gl_extension = wined3d_gl_extension_ARB_FRAMEBUFFER_SRGB; // island enum // gen:f4ac4c78c832
pub const ARB_GEOMETRY_SHADER4: wined3d_gl_extension = wined3d_gl_extension_ARB_GEOMETRY_SHADER4; // island enum // gen:054311031b8c
pub const ARB_GPU_SHADER5: wined3d_gl_extension = wined3d_gl_extension_ARB_GPU_SHADER5; // island enum // gen:890420116ec3
pub const ARB_HALF_FLOAT_PIXEL: wined3d_gl_extension = wined3d_gl_extension_ARB_HALF_FLOAT_PIXEL; // island enum // gen:fa0f86c8526b
pub const ARB_HALF_FLOAT_VERTEX: wined3d_gl_extension = wined3d_gl_extension_ARB_HALF_FLOAT_VERTEX; // island enum // gen:5a20ae172805
pub const ARB_INSTANCED_ARRAYS: wined3d_gl_extension = wined3d_gl_extension_ARB_INSTANCED_ARRAYS; // island enum // gen:d03c0b398625
pub const ARB_INTERNALFORMAT_QUERY2: wined3d_gl_extension = wined3d_gl_extension_ARB_INTERNALFORMAT_QUERY2; // island enum // gen:c72c7e7f3105
pub const ARB_INTERNALFORMAT_QUERY: wined3d_gl_extension = wined3d_gl_extension_ARB_INTERNALFORMAT_QUERY; // island enum // gen:ad2f49712bae
pub const ARB_MAP_BUFFER_ALIGNMENT: wined3d_gl_extension = wined3d_gl_extension_ARB_MAP_BUFFER_ALIGNMENT; // island enum // gen:7db3ef2b57a9
pub const ARB_MAP_BUFFER_RANGE: wined3d_gl_extension = wined3d_gl_extension_ARB_MAP_BUFFER_RANGE; // island enum // gen:3559f308e883
pub const ARB_MULTISAMPLE: wined3d_gl_extension = wined3d_gl_extension_ARB_MULTISAMPLE; // island enum // gen:bf777c6da8c2
pub const ARB_MULTITEXTURE: wined3d_gl_extension = wined3d_gl_extension_ARB_MULTITEXTURE; // island enum // gen:6f12ae8d4784
pub const ARB_OCCLUSION_QUERY: wined3d_gl_extension = wined3d_gl_extension_ARB_OCCLUSION_QUERY; // island enum // gen:5313cb845f06
pub const ARB_PIPELINE_STATISTICS_QUERY: wined3d_gl_extension = wined3d_gl_extension_ARB_PIPELINE_STATISTICS_QUERY; // island enum // gen:a6a2a87cccef
pub const ARB_PIXEL_BUFFER_OBJECT: wined3d_gl_extension = wined3d_gl_extension_ARB_PIXEL_BUFFER_OBJECT; // island enum // gen:b1b0f982e128
pub const ARB_POINT_PARAMETERS: wined3d_gl_extension = wined3d_gl_extension_ARB_POINT_PARAMETERS; // island enum // gen:e2214856df63
pub const ARB_POINT_SPRITE: wined3d_gl_extension = wined3d_gl_extension_ARB_POINT_SPRITE; // island enum // gen:27b4174b9183
pub const ARB_POLYGON_OFFSET_CLAMP: wined3d_gl_extension = wined3d_gl_extension_ARB_POLYGON_OFFSET_CLAMP; // island enum // gen:b925bc0646ff
pub const ARB_PROVOKING_VERTEX: wined3d_gl_extension = wined3d_gl_extension_ARB_PROVOKING_VERTEX; // island enum // gen:a6099383875f
pub const ARB_QUERY_BUFFER_OBJECT: wined3d_gl_extension = wined3d_gl_extension_ARB_QUERY_BUFFER_OBJECT; // island enum // gen:720eac7cd425
pub const ARB_SAMPLER_OBJECTS: wined3d_gl_extension = wined3d_gl_extension_ARB_SAMPLER_OBJECTS; // island enum // gen:b6366763f3c9
pub const ARB_SAMPLE_SHADING: wined3d_gl_extension = wined3d_gl_extension_ARB_SAMPLE_SHADING; // island enum // gen:2f11a50b2f48
pub const ARB_SEAMLESS_CUBE_MAP: wined3d_gl_extension = wined3d_gl_extension_ARB_SEAMLESS_CUBE_MAP; // island enum // gen:68b3bc025367
pub const ARB_SHADER_ATOMIC_COUNTERS: wined3d_gl_extension = wined3d_gl_extension_ARB_SHADER_ATOMIC_COUNTERS; // island enum // gen:302b1edc1aac
pub const ARB_SHADER_BIT_ENCODING: wined3d_gl_extension = wined3d_gl_extension_ARB_SHADER_BIT_ENCODING; // island enum // gen:3cea358a2d83
pub const ARB_SHADER_IMAGE_LOAD_STORE: wined3d_gl_extension = wined3d_gl_extension_ARB_SHADER_IMAGE_LOAD_STORE; // island enum // gen:de3fcd2a613e
pub const ARB_SHADER_IMAGE_SIZE: wined3d_gl_extension = wined3d_gl_extension_ARB_SHADER_IMAGE_SIZE; // island enum // gen:6eea6fe7dc89
pub const ARB_SHADER_STENCIL_EXPORT: wined3d_gl_extension = wined3d_gl_extension_ARB_SHADER_STENCIL_EXPORT; // island enum // gen:c1475999b30e
pub const ARB_SHADER_STORAGE_BUFFER_OBJECT: wined3d_gl_extension = wined3d_gl_extension_ARB_SHADER_STORAGE_BUFFER_OBJECT; // island enum // gen:1dbc71427489
pub const ARB_SHADER_TEXTURE_IMAGE_SAMPLES: wined3d_gl_extension = wined3d_gl_extension_ARB_SHADER_TEXTURE_IMAGE_SAMPLES; // island enum // gen:397d1cd6134d
pub const ARB_SHADER_TEXTURE_LOD: wined3d_gl_extension = wined3d_gl_extension_ARB_SHADER_TEXTURE_LOD; // island enum // gen:447af3832b62
pub const ARB_SHADER_VIEWPORT_LAYER_ARRAY: wined3d_gl_extension = wined3d_gl_extension_ARB_SHADER_VIEWPORT_LAYER_ARRAY; // island enum // gen:4654ed0d9154
pub const ARB_SHADING_LANGUAGE_100: wined3d_gl_extension = wined3d_gl_extension_ARB_SHADING_LANGUAGE_100; // island enum // gen:0d37240332f7
pub const ARB_SHADING_LANGUAGE_420PACK: wined3d_gl_extension = wined3d_gl_extension_ARB_SHADING_LANGUAGE_420PACK; // island enum // gen:3d09dcd708ba
pub const ARB_SHADING_LANGUAGE_PACKING: wined3d_gl_extension = wined3d_gl_extension_ARB_SHADING_LANGUAGE_PACKING; // island enum // gen:58b8daf0f32c
pub const ARB_SHADOW: wined3d_gl_extension = wined3d_gl_extension_ARB_SHADOW; // island enum // gen:68ab354f521a
pub const ARB_STENCIL_TEXTURING: wined3d_gl_extension = wined3d_gl_extension_ARB_STENCIL_TEXTURING; // island enum // gen:fbe8e1a762ec
pub const ARB_SYNC: wined3d_gl_extension = wined3d_gl_extension_ARB_SYNC; // island enum // gen:c0655aa88c38
pub const ARB_TESSELLATION_SHADER: wined3d_gl_extension = wined3d_gl_extension_ARB_TESSELLATION_SHADER; // island enum // gen:5ce7c3033782
pub const ARB_TEXTURE_BARRIER: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_BARRIER; // island enum // gen:615802290a87
pub const ARB_TEXTURE_BORDER_CLAMP: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_BORDER_CLAMP; // island enum // gen:976b810f8fe6
pub const ARB_TEXTURE_BUFFER_OBJECT: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_BUFFER_OBJECT; // island enum // gen:89cdfe5b9159
pub const ARB_TEXTURE_BUFFER_RANGE: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_BUFFER_RANGE; // island enum // gen:74019dbc93c7
pub const ARB_TEXTURE_COMPRESSION: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_COMPRESSION; // island enum // gen:5876b54e5602
pub const ARB_TEXTURE_COMPRESSION_BPTC: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_COMPRESSION_BPTC; // island enum // gen:ca20637cfba9
pub const ARB_TEXTURE_COMPRESSION_RGTC: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_COMPRESSION_RGTC; // island enum // gen:f5f98273bedd
pub const ARB_TEXTURE_CUBE_MAP: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_CUBE_MAP; // island enum // gen:bbea38fd8a53
pub const ARB_TEXTURE_CUBE_MAP_ARRAY: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_CUBE_MAP_ARRAY; // island enum // gen:3a252fc500d4
pub const ARB_TEXTURE_ENV_COMBINE: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_ENV_COMBINE; // island enum // gen:76f9dea8b63e
pub const ARB_TEXTURE_ENV_DOT3: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_ENV_DOT3; // island enum // gen:82db77fdf3ff
pub const ARB_TEXTURE_FILTER_ANISOTROPIC: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_FILTER_ANISOTROPIC; // island enum // gen:d177cec0dbdb
pub const ARB_TEXTURE_FLOAT: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_FLOAT; // island enum // gen:27b98c0de024
pub const ARB_TEXTURE_GATHER: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_GATHER; // island enum // gen:d7349c84d92e
pub const ARB_TEXTURE_MIRRORED_REPEAT: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_MIRRORED_REPEAT; // island enum // gen:996d651152e9
pub const ARB_TEXTURE_MIRROR_CLAMP_TO_EDGE: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_MIRROR_CLAMP_TO_EDGE; // island enum // gen:6a5f497f5ef3
pub const ARB_TEXTURE_MULTISAMPLE: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_MULTISAMPLE; // island enum // gen:33c414f29a41
pub const ARB_TEXTURE_NON_POWER_OF_TWO: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_NON_POWER_OF_TWO; // island enum // gen:fdb55a085099
pub const ARB_TEXTURE_QUERY_LEVELS: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_QUERY_LEVELS; // island enum // gen:ca7a77323446
pub const ARB_TEXTURE_RG: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_RG; // island enum // gen:d29576e6b379
pub const ARB_TEXTURE_RGB10_A2UI: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_RGB10_A2UI; // island enum // gen:17c68141f6b5
pub const ARB_TEXTURE_STORAGE: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_STORAGE; // island enum // gen:3ba24d73e7fa
pub const ARB_TEXTURE_STORAGE_MULTISAMPLE: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_STORAGE_MULTISAMPLE; // island enum // gen:53e859052dee
pub const ARB_TEXTURE_SWIZZLE: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_SWIZZLE; // island enum // gen:78676c24d19b
pub const ARB_TEXTURE_VIEW: wined3d_gl_extension = wined3d_gl_extension_ARB_TEXTURE_VIEW; // island enum // gen:584397f5f373
pub const ARB_TIMER_QUERY: wined3d_gl_extension = wined3d_gl_extension_ARB_TIMER_QUERY; // island enum // gen:13f9a53afe50
pub const ARB_TRANSFORM_FEEDBACK2: wined3d_gl_extension = wined3d_gl_extension_ARB_TRANSFORM_FEEDBACK2; // island enum // gen:aa4db06f894f
pub const ARB_TRANSFORM_FEEDBACK3: wined3d_gl_extension = wined3d_gl_extension_ARB_TRANSFORM_FEEDBACK3; // island enum // gen:7f29c472cd1d
pub const ARB_UNIFORM_BUFFER_OBJECT: wined3d_gl_extension = wined3d_gl_extension_ARB_UNIFORM_BUFFER_OBJECT; // island enum // gen:c6e220f22669
pub const ARB_VERTEX_ARRAY_BGRA: wined3d_gl_extension = wined3d_gl_extension_ARB_VERTEX_ARRAY_BGRA; // island enum // gen:45e0c513aed2
pub const ARB_VERTEX_BUFFER_OBJECT: wined3d_gl_extension = wined3d_gl_extension_ARB_VERTEX_BUFFER_OBJECT; // island enum // gen:55b28ee2650f
pub const ARB_VERTEX_PROGRAM: wined3d_gl_extension = wined3d_gl_extension_ARB_VERTEX_PROGRAM; // island enum // gen:5e1746f117d0
pub const ARB_VERTEX_SHADER: wined3d_gl_extension = wined3d_gl_extension_ARB_VERTEX_SHADER; // island enum // gen:d8ad35c3f81c
pub const ARB_VERTEX_TYPE_10F_11F_11F_REV: wined3d_gl_extension = wined3d_gl_extension_ARB_VERTEX_TYPE_10F_11F_11F_REV; // island enum // gen:9e7cfa9e341a
pub const ARB_VERTEX_TYPE_2_10_10_10_REV: wined3d_gl_extension = wined3d_gl_extension_ARB_VERTEX_TYPE_2_10_10_10_REV; // island enum // gen:bf85732133df
pub const ARB_VIEWPORT_ARRAY: wined3d_gl_extension = wined3d_gl_extension_ARB_VIEWPORT_ARRAY; // island enum // gen:7497f2be3e6d
pub const ATI_FRAGMENT_SHADER: wined3d_gl_extension = wined3d_gl_extension_ATI_FRAGMENT_SHADER; // island enum // gen:412ddbaadbf5
pub const ATI_SEPARATE_STENCIL: wined3d_gl_extension = wined3d_gl_extension_ATI_SEPARATE_STENCIL; // island enum // gen:6573ba560570
pub const ATI_TEXTURE_COMPRESSION_3DC: wined3d_gl_extension = wined3d_gl_extension_ATI_TEXTURE_COMPRESSION_3DC; // island enum // gen:984414c9897f
pub const ATI_TEXTURE_ENV_COMBINE3: wined3d_gl_extension = wined3d_gl_extension_ATI_TEXTURE_ENV_COMBINE3; // island enum // gen:4eb401962f2d
pub const ATI_TEXTURE_MIRROR_ONCE: wined3d_gl_extension = wined3d_gl_extension_ATI_TEXTURE_MIRROR_ONCE; // island enum // gen:a5b8e930a8e3
pub const CARD_AMD_RADEON_7200: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_7200; // island enum // gen:ba09aace54dd
pub const CARD_AMD_RADEON_8500: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_8500; // island enum // gen:5a0738d5b6e0
pub const CARD_AMD_RADEON_9500: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_9500; // island enum // gen:76a2ce1cf620
pub const CARD_AMD_RADEON_HD2350: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD2350; // island enum // gen:21f557fb3724
pub const CARD_AMD_RADEON_HD2600: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD2600; // island enum // gen:928ed43157a4
pub const CARD_AMD_RADEON_HD2900: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD2900; // island enum // gen:7b6cb0663f42
pub const CARD_AMD_RADEON_HD3200: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD3200; // island enum // gen:08f17c58ee26
pub const CARD_AMD_RADEON_HD3850: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD3850; // island enum // gen:39e17a8756e9
pub const CARD_AMD_RADEON_HD4200M: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD4200M; // island enum // gen:2e333eaf3552
pub const CARD_AMD_RADEON_HD4350: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD4350; // island enum // gen:602d4a16edf8
pub const CARD_AMD_RADEON_HD4600: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD4600; // island enum // gen:0fe2e70c9f79
pub const CARD_AMD_RADEON_HD4700: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD4700; // island enum // gen:3c0b5ece1c0a
pub const CARD_AMD_RADEON_HD4800: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD4800; // island enum // gen:49a96cb51bb5
pub const CARD_AMD_RADEON_HD5400: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD5400; // island enum // gen:d9684b4d2444
pub const CARD_AMD_RADEON_HD5600: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD5600; // island enum // gen:a29fe574a4b7
pub const CARD_AMD_RADEON_HD5700: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD5700; // island enum // gen:3b718211e437
pub const CARD_AMD_RADEON_HD5800: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD5800; // island enum // gen:51ffd738502f
pub const CARD_AMD_RADEON_HD5900: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD5900; // island enum // gen:09b232d8a44b
pub const CARD_AMD_RADEON_HD6300: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD6300; // island enum // gen:2ca115305fc6
pub const CARD_AMD_RADEON_HD6400: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD6400; // island enum // gen:3ee6b94230c8
pub const CARD_AMD_RADEON_HD6410D: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD6410D; // island enum // gen:29f5a1307140
pub const CARD_AMD_RADEON_HD6480G: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD6480G; // island enum // gen:6cce3fe85027
pub const CARD_AMD_RADEON_HD6490M: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD6490M; // island enum // gen:a35fb628ee5f
pub const CARD_AMD_RADEON_HD6550D: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD6550D; // island enum // gen:21bf98c5be3e
pub const CARD_AMD_RADEON_HD6600: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD6600; // island enum // gen:e208f5f63492
pub const CARD_AMD_RADEON_HD6600M: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD6600M; // island enum // gen:543c59eceded
pub const CARD_AMD_RADEON_HD6700: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD6700; // island enum // gen:5b5418d9dc3b
pub const CARD_AMD_RADEON_HD6800: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD6800; // island enum // gen:d33e7f6cbb7d
pub const CARD_AMD_RADEON_HD6900: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD6900; // island enum // gen:7dfefccbb473
pub const CARD_AMD_RADEON_HD7660D: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD7660D; // island enum // gen:0f4944e634f5
pub const CARD_AMD_RADEON_HD7700: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD7700; // island enum // gen:8abf51d06127
pub const CARD_AMD_RADEON_HD7800: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD7800; // island enum // gen:d0fc1a20fe09
pub const CARD_AMD_RADEON_HD7870: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD7870; // island enum // gen:bc102e9cb11e
pub const CARD_AMD_RADEON_HD7900: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD7900; // island enum // gen:fa50c2a1c8ee
pub const CARD_AMD_RADEON_HD8600M: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD8600M; // island enum // gen:4dddc705ad7f
pub const CARD_AMD_RADEON_HD8670: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD8670; // island enum // gen:889e6a41b7ba
pub const CARD_AMD_RADEON_HD8770: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_HD8770; // island enum // gen:98337584db7c
pub const CARD_AMD_RADEON_PRO_V620: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_PRO_V620; // island enum // gen:c476f4a0bf05
pub const CARD_AMD_RADEON_PRO_V620_VF: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_PRO_V620_VF; // island enum // gen:1e3522f6b8dc
pub const CARD_AMD_RADEON_R3: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_R3; // island enum // gen:8fdd170ba2e6
pub const CARD_AMD_RADEON_R7: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_R7; // island enum // gen:0c86c2d4b13a
pub const CARD_AMD_RADEON_R9_285: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_R9_285; // island enum // gen:9f1c65c997d4
pub const CARD_AMD_RADEON_R9_290: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_R9_290; // island enum // gen:717684ac751e
pub const CARD_AMD_RADEON_R9_290X: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_R9_290X; // island enum // gen:8cc84156abca
pub const CARD_AMD_RADEON_R9_FURY: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_R9_FURY; // island enum // gen:3d9a77965b6a
pub const CARD_AMD_RADEON_R9_M370X: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_R9_M370X; // island enum // gen:6b4a2002a6c8
pub const CARD_AMD_RADEON_R9_M380: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_R9_M380; // island enum // gen:3edac8b30b41
pub const CARD_AMD_RADEON_R9_M395X: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_R9_M395X; // island enum // gen:12e4feebc650
pub const CARD_AMD_RADEON_RAVEN: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_RAVEN; // island enum // gen:3fc04c2d6cb6
pub const CARD_AMD_RADEON_RX_460: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_RX_460; // island enum // gen:54655dca1a44
pub const CARD_AMD_RADEON_RX_480: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_RX_480; // island enum // gen:b610a72219b6
pub const CARD_AMD_RADEON_RX_NAVI_10: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_RX_NAVI_10; // island enum // gen:210df270096e
pub const CARD_AMD_RADEON_RX_NAVI_14: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_RX_NAVI_14; // island enum // gen:f7ff67254541
pub const CARD_AMD_RADEON_RX_NAVI_21: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_RX_NAVI_21; // island enum // gen:0928fbed52f1
pub const CARD_AMD_RADEON_RX_VEGA_10: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_RX_VEGA_10; // island enum // gen:1dc5bafd1b0e
pub const CARD_AMD_RADEON_RX_VEGA_12: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_RX_VEGA_12; // island enum // gen:359ae4e2da40
pub const CARD_AMD_RADEON_RX_VEGA_20: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_RX_VEGA_20; // island enum // gen:4cd8b055ebfb
pub const CARD_AMD_RADEON_X1600: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_X1600; // island enum // gen:cbd8fd2519fd
pub const CARD_AMD_RADEON_X700: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_X700; // island enum // gen:d6a76d3053a6
pub const CARD_AMD_RADEON_XPRESS_200M: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RADEON_XPRESS_200M; // island enum // gen:ee7ab1a944c6
pub const CARD_AMD_RAGE_128PRO: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RAGE_128PRO; // island enum // gen:c87f9c0cb861
pub const CARD_AMD_RAPHAEL: wined3d_pci_device = wined3d_pci_device_CARD_AMD_RAPHAEL; // island enum // gen:3ae52d3ced11
pub const CARD_AMD_VANGOGH: wined3d_pci_device = wined3d_pci_device_CARD_AMD_VANGOGH; // island enum // gen:cb14ff40d07b
pub const CARD_INTEL_830M: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_830M; // island enum // gen:cd072e96e5d3
pub const CARD_INTEL_845G: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_845G; // island enum // gen:d3d210575e66
pub const CARD_INTEL_855GM: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_855GM; // island enum // gen:d6162924fe2a
pub const CARD_INTEL_865G: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_865G; // island enum // gen:ef9d0cb7e71c
pub const CARD_INTEL_915G: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_915G; // island enum // gen:2e7f628889d7
pub const CARD_INTEL_915GM: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_915GM; // island enum // gen:e89b673c230a
pub const CARD_INTEL_945G: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_945G; // island enum // gen:0b1f40779328
pub const CARD_INTEL_945GM: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_945GM; // island enum // gen:a1f701a2df23
pub const CARD_INTEL_945GME: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_945GME; // island enum // gen:eee12e179fb8
pub const CARD_INTEL_946GZ: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_946GZ; // island enum // gen:9e4cd0eabd94
pub const CARD_INTEL_965G: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_965G; // island enum // gen:a73aa4f74c90
pub const CARD_INTEL_965GM: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_965GM; // island enum // gen:d9b65fef2e9c
pub const CARD_INTEL_965GME: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_965GME; // island enum // gen:a34954db65b6
pub const CARD_INTEL_965Q: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_965Q; // island enum // gen:8e9c56f92278
pub const CARD_INTEL_B43: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_B43; // island enum // gen:d45d5a883e32
pub const CARD_INTEL_E7221G: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_E7221G; // island enum // gen:5b36cfd72f93
pub const CARD_INTEL_G33: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_G33; // island enum // gen:6f67f90e0e0c
pub const CARD_INTEL_G41: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_G41; // island enum // gen:d29d3eb1eacd
pub const CARD_INTEL_G45: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_G45; // island enum // gen:3842031ce85a
pub const CARD_INTEL_GM45: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_GM45; // island enum // gen:58898c04b485
pub const CARD_INTEL_HD5000_1: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_HD5000_1; // island enum // gen:dd56ae62d77d
pub const CARD_INTEL_HD5000_2: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_HD5000_2; // island enum // gen:e65e96645d0e
pub const CARD_INTEL_HD510_1: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_HD510_1; // island enum // gen:577420ff876c
pub const CARD_INTEL_HD510_2: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_HD510_2; // island enum // gen:5c37a461cea0
pub const CARD_INTEL_HD510_3: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_HD510_3; // island enum // gen:6d8ce2ae5cec
pub const CARD_INTEL_HD515: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_HD515; // island enum // gen:e895057db1ca
pub const CARD_INTEL_HD520_1: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_HD520_1; // island enum // gen:189cf521a2de
pub const CARD_INTEL_HD520_2: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_HD520_2; // island enum // gen:f222b8686eab
pub const CARD_INTEL_HD5300: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_HD5300; // island enum // gen:aa42fe6f3b80
pub const CARD_INTEL_HD530_1: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_HD530_1; // island enum // gen:0ede9d1d00a8
pub const CARD_INTEL_HD530_2: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_HD530_2; // island enum // gen:9d41c693a207
pub const CARD_INTEL_HD5500: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_HD5500; // island enum // gen:26daea5714f7
pub const CARD_INTEL_HD5600: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_HD5600; // island enum // gen:b83aa4b31936
pub const CARD_INTEL_HD6000: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_HD6000; // island enum // gen:08eda779c47f
pub const CARD_INTEL_HD615: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_HD615; // island enum // gen:0b3218871324
pub const CARD_INTEL_HD620: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_HD620; // island enum // gen:7eb92749c0b7
pub const CARD_INTEL_HD630_1: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_HD630_1; // island enum // gen:513f6bf5819a
pub const CARD_INTEL_HD630_2: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_HD630_2; // island enum // gen:768733c2a6b8
pub const CARD_INTEL_HDP530: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_HDP530; // island enum // gen:c6fc11c988a2
pub const CARD_INTEL_HWD: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_HWD; // island enum // gen:08561a2f5495
pub const CARD_INTEL_HWM: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_HWM; // island enum // gen:2c551afc427e
pub const CARD_INTEL_I5100_1: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_I5100_1; // island enum // gen:f76d46dbf4bc
pub const CARD_INTEL_I5100_2: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_I5100_2; // island enum // gen:228162b2a017
pub const CARD_INTEL_I5100_3: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_I5100_3; // island enum // gen:45e0bd649ef9
pub const CARD_INTEL_I5100_4: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_I5100_4; // island enum // gen:d2a0297ff1d8
pub const CARD_INTEL_I540: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_I540; // island enum // gen:4aa6215491cc
pub const CARD_INTEL_I550: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_I550; // island enum // gen:daa3da42f052
pub const CARD_INTEL_I555: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_I555; // island enum // gen:c00a4e123914
pub const CARD_INTEL_I6100: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_I6100; // island enum // gen:a84f0dadd2a8
pub const CARD_INTEL_IGD: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_IGD; // island enum // gen:0e6ab22bc9aa
pub const CARD_INTEL_ILKD: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_ILKD; // island enum // gen:20b2b6aeb239
pub const CARD_INTEL_ILKM: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_ILKM; // island enum // gen:365ba1a74808
pub const CARD_INTEL_IP5200_1: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_IP5200_1; // island enum // gen:0b7e8f38dceb
pub const CARD_INTEL_IP5200_2: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_IP5200_2; // island enum // gen:0ff8f35734e2
pub const CARD_INTEL_IP5200_3: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_IP5200_3; // island enum // gen:6ef82734ebd8
pub const CARD_INTEL_IP5200_4: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_IP5200_4; // island enum // gen:deae18daf4c7
pub const CARD_INTEL_IP5200_5: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_IP5200_5; // island enum // gen:339d591d7c94
pub const CARD_INTEL_IP5200_6: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_IP5200_6; // island enum // gen:419414cbf370
pub const CARD_INTEL_IP555: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_IP555; // island enum // gen:8a5e4e85e40a
pub const CARD_INTEL_IP580_1: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_IP580_1; // island enum // gen:cfd9225f36e3
pub const CARD_INTEL_IP580_2: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_IP580_2; // island enum // gen:9051b9454d1b
pub const CARD_INTEL_IP6200: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_IP6200; // island enum // gen:58de6720524c
pub const CARD_INTEL_IPP580_1: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_IPP580_1; // island enum // gen:2980d47ec54b
pub const CARD_INTEL_IPP580_2: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_IPP580_2; // island enum // gen:e0af0fb04a38
pub const CARD_INTEL_IPP6300: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_IPP6300; // island enum // gen:12bd8be10d55
pub const CARD_INTEL_IVBD: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_IVBD; // island enum // gen:044c102d3c16
pub const CARD_INTEL_IVBM: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_IVBM; // island enum // gen:c3bcad739823
pub const CARD_INTEL_IVBS: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_IVBS; // island enum // gen:7ebc966a5fa6
pub const CARD_INTEL_PNVG: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_PNVG; // island enum // gen:72c6712f3f40
pub const CARD_INTEL_PNVM: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_PNVM; // island enum // gen:d26b8bd63c13
pub const CARD_INTEL_Q33: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_Q33; // island enum // gen:4b5d2b61d0fd
pub const CARD_INTEL_Q35: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_Q35; // island enum // gen:3f674d525e3d
pub const CARD_INTEL_Q45: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_Q45; // island enum // gen:7427aa5d5cce
pub const CARD_INTEL_SNBD: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_SNBD; // island enum // gen:fe3aa40d755f
pub const CARD_INTEL_SNBM: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_SNBM; // island enum // gen:55512cdf6d35
pub const CARD_INTEL_SNBS: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_SNBS; // island enum // gen:046884d9e62f
pub const CARD_INTEL_UHD617: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_UHD617; // island enum // gen:720ce0490022
pub const CARD_INTEL_UHD620: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_UHD620; // island enum // gen:335a82be8ed5
pub const CARD_INTEL_UHD630_1: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_UHD630_1; // island enum // gen:d12d99b36790
pub const CARD_INTEL_UHD630_2: wined3d_pci_device = wined3d_pci_device_CARD_INTEL_UHD630_2; // island enum // gen:f9d90ff314f2
pub const CARD_NVIDIA_AMPERE_A10: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_AMPERE_A10; // island enum // gen:195ad5de38ce
pub const CARD_NVIDIA_GEFORCE2: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE2; // island enum // gen:f9cd0a1956d9
pub const CARD_NVIDIA_GEFORCE2_MX: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE2_MX; // island enum // gen:28d2fee12cac
pub const CARD_NVIDIA_GEFORCE3: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE3; // island enum // gen:394cdcfaa0ba
pub const CARD_NVIDIA_GEFORCE4_MX: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE4_MX; // island enum // gen:bd6c9f501e02
pub const CARD_NVIDIA_GEFORCE4_TI4200: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE4_TI4200; // island enum // gen:738ac7761a0c
pub const CARD_NVIDIA_GEFORCE: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE; // island enum // gen:b4a148a89a48
pub const CARD_NVIDIA_GEFORCEFX_5200: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCEFX_5200; // island enum // gen:059018bc2412
pub const CARD_NVIDIA_GEFORCEFX_5600: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCEFX_5600; // island enum // gen:4588127bb05c
pub const CARD_NVIDIA_GEFORCEFX_5800: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCEFX_5800; // island enum // gen:9b8d20ae54c7
pub const CARD_NVIDIA_GEFORCE_210: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_210; // island enum // gen:94f5808f9ff1
pub const CARD_NVIDIA_GEFORCE_315M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_315M; // island enum // gen:aa12feea982b
pub const CARD_NVIDIA_GEFORCE_320M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_320M; // island enum // gen:bbe1d673ec28
pub const CARD_NVIDIA_GEFORCE_410M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_410M; // island enum // gen:c8615c3ee6c8
pub const CARD_NVIDIA_GEFORCE_6200: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_6200; // island enum // gen:9cc31fdd3dab
pub const CARD_NVIDIA_GEFORCE_6600GT: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_6600GT; // island enum // gen:a35623f0dfa5
pub const CARD_NVIDIA_GEFORCE_6800: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_6800; // island enum // gen:b2ea36011224
pub const CARD_NVIDIA_GEFORCE_7300: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_7300; // island enum // gen:8a0d17e78e54
pub const CARD_NVIDIA_GEFORCE_7400: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_7400; // island enum // gen:d095d70745a6
pub const CARD_NVIDIA_GEFORCE_7600: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_7600; // island enum // gen:759da1bba3e0
pub const CARD_NVIDIA_GEFORCE_7800GT: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_7800GT; // island enum // gen:715038e35b32
pub const CARD_NVIDIA_GEFORCE_8200: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_8200; // island enum // gen:fbe070fa95a4
pub const CARD_NVIDIA_GEFORCE_820M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_820M; // island enum // gen:34c29bcc0600
pub const CARD_NVIDIA_GEFORCE_8300GS: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_8300GS; // island enum // gen:00ba5dba2893
pub const CARD_NVIDIA_GEFORCE_830M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_830M; // island enum // gen:d63ca33ea4da
pub const CARD_NVIDIA_GEFORCE_8400GS: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_8400GS; // island enum // gen:7c4e0931f480
pub const CARD_NVIDIA_GEFORCE_840M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_840M; // island enum // gen:bc39802e89a7
pub const CARD_NVIDIA_GEFORCE_845M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_845M; // island enum // gen:2c6df951e6fb
pub const CARD_NVIDIA_GEFORCE_8500GT: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_8500GT; // island enum // gen:aa8a517bf1cb
pub const CARD_NVIDIA_GEFORCE_8600GT: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_8600GT; // island enum // gen:23624ccad8e7
pub const CARD_NVIDIA_GEFORCE_8600MGT: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_8600MGT; // island enum // gen:761a1dee3d97
pub const CARD_NVIDIA_GEFORCE_8800GTS: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_8800GTS; // island enum // gen:14e02bbf85be
pub const CARD_NVIDIA_GEFORCE_8800GTX: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_8800GTX; // island enum // gen:8942a82671fd
pub const CARD_NVIDIA_GEFORCE_9200: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_9200; // island enum // gen:3d19a4d46a43
pub const CARD_NVIDIA_GEFORCE_9300: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_9300; // island enum // gen:666ae52bb617
pub const CARD_NVIDIA_GEFORCE_9400GT: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_9400GT; // island enum // gen:9aa2dc9f09e0
pub const CARD_NVIDIA_GEFORCE_9400M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_9400M; // island enum // gen:1533241c5d7c
pub const CARD_NVIDIA_GEFORCE_940M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_940M; // island enum // gen:893069e42daf
pub const CARD_NVIDIA_GEFORCE_9500GT: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_9500GT; // island enum // gen:d2c9aa332772
pub const CARD_NVIDIA_GEFORCE_9600GT: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_9600GT; // island enum // gen:1745e46beaae
pub const CARD_NVIDIA_GEFORCE_9700MGT: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_9700MGT; // island enum // gen:0dda511d6b90
pub const CARD_NVIDIA_GEFORCE_9800GT: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_9800GT; // island enum // gen:c081343cd733
pub const CARD_NVIDIA_GEFORCE_GT1030: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT1030; // island enum // gen:b7656b93489a
pub const CARD_NVIDIA_GEFORCE_GT220: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT220; // island enum // gen:f36414a143f5
pub const CARD_NVIDIA_GEFORCE_GT240: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT240; // island enum // gen:bfa983791207
pub const CARD_NVIDIA_GEFORCE_GT320M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT320M; // island enum // gen:86a6f2c15581
pub const CARD_NVIDIA_GEFORCE_GT325M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT325M; // island enum // gen:2caf6414b8f5
pub const CARD_NVIDIA_GEFORCE_GT330: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT330; // island enum // gen:4a6f4111ed08
pub const CARD_NVIDIA_GEFORCE_GT420: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT420; // island enum // gen:c68449caa66e
pub const CARD_NVIDIA_GEFORCE_GT425M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT425M; // island enum // gen:c0a5ceb82eea
pub const CARD_NVIDIA_GEFORCE_GT430: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT430; // island enum // gen:9c22c50062d1
pub const CARD_NVIDIA_GEFORCE_GT440: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT440; // island enum // gen:8ffb6bd08430
pub const CARD_NVIDIA_GEFORCE_GT520: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT520; // island enum // gen:059722e11f85
pub const CARD_NVIDIA_GEFORCE_GT525M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT525M; // island enum // gen:0f4b52701075
pub const CARD_NVIDIA_GEFORCE_GT540M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT540M; // island enum // gen:6ff515c80dab
pub const CARD_NVIDIA_GEFORCE_GT555M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT555M; // island enum // gen:03f48d579446
pub const CARD_NVIDIA_GEFORCE_GT610: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT610; // island enum // gen:abe8e370d5ce
pub const CARD_NVIDIA_GEFORCE_GT630: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT630; // island enum // gen:249feae5ba7c
pub const CARD_NVIDIA_GEFORCE_GT630M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT630M; // island enum // gen:00bf3fadc297
pub const CARD_NVIDIA_GEFORCE_GT640: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT640; // island enum // gen:1df33054ad09
pub const CARD_NVIDIA_GEFORCE_GT640M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT640M; // island enum // gen:13239a005dfc
pub const CARD_NVIDIA_GEFORCE_GT650M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT650M; // island enum // gen:1f2aba5ad69d
pub const CARD_NVIDIA_GEFORCE_GT720: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT720; // island enum // gen:572eeeacb341
pub const CARD_NVIDIA_GEFORCE_GT730: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT730; // island enum // gen:881ab96c9e52
pub const CARD_NVIDIA_GEFORCE_GT730M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT730M; // island enum // gen:c384b84f5d43
pub const CARD_NVIDIA_GEFORCE_GT740M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT740M; // island enum // gen:6473fb47d512
pub const CARD_NVIDIA_GEFORCE_GT750M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT750M; // island enum // gen:27fc1bebab1d
pub const CARD_NVIDIA_GEFORCE_GT755M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GT755M; // island enum // gen:530c71f7e30d
pub const CARD_NVIDIA_GEFORCE_GTS250: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTS250; // island enum // gen:03e4a5aff4de
pub const CARD_NVIDIA_GEFORCE_GTS350M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTS350M; // island enum // gen:834194d882ca
pub const CARD_NVIDIA_GEFORCE_GTS450: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTS450; // island enum // gen:a4054fc5ec74
pub const CARD_NVIDIA_GEFORCE_GTX1050: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX1050; // island enum // gen:52d98ff325de
pub const CARD_NVIDIA_GEFORCE_GTX1050TI: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX1050TI; // island enum // gen:2abc74bde323
pub const CARD_NVIDIA_GEFORCE_GTX1060: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX1060; // island enum // gen:74a5517475a4
pub const CARD_NVIDIA_GEFORCE_GTX1060M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX1060M; // island enum // gen:b81a87d7a2b5
pub const CARD_NVIDIA_GEFORCE_GTX1060_3GB: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX1060_3GB; // island enum // gen:aded3efaf8a1
pub const CARD_NVIDIA_GEFORCE_GTX1070: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX1070; // island enum // gen:ddc0eb734595
pub const CARD_NVIDIA_GEFORCE_GTX1070M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX1070M; // island enum // gen:e99fb00fb505
pub const CARD_NVIDIA_GEFORCE_GTX1080: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX1080; // island enum // gen:6f307ca9eb07
pub const CARD_NVIDIA_GEFORCE_GTX1080M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX1080M; // island enum // gen:f1d872f7b15f
pub const CARD_NVIDIA_GEFORCE_GTX1080TI: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX1080TI; // island enum // gen:d7b906f2619e
pub const CARD_NVIDIA_GEFORCE_GTX1650: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX1650; // island enum // gen:963f116a3f6f
pub const CARD_NVIDIA_GEFORCE_GTX1650SUPER: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX1650SUPER; // island enum // gen:b1eefa88af64
pub const CARD_NVIDIA_GEFORCE_GTX1660SUPER: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX1660SUPER; // island enum // gen:19ee1fd95ace
pub const CARD_NVIDIA_GEFORCE_GTX1660TI: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX1660TI; // island enum // gen:f92fad5965c8
pub const CARD_NVIDIA_GEFORCE_GTX260: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX260; // island enum // gen:9527f5f8332b
pub const CARD_NVIDIA_GEFORCE_GTX275: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX275; // island enum // gen:49da42931fe0
pub const CARD_NVIDIA_GEFORCE_GTX280: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX280; // island enum // gen:c13d45ffa5ac
pub const CARD_NVIDIA_GEFORCE_GTX460: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX460; // island enum // gen:e0f5cccb30f8
pub const CARD_NVIDIA_GEFORCE_GTX460M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX460M; // island enum // gen:a3114c676dea
pub const CARD_NVIDIA_GEFORCE_GTX465: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX465; // island enum // gen:624c0442d517
pub const CARD_NVIDIA_GEFORCE_GTX470: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX470; // island enum // gen:c7fb57a39495
pub const CARD_NVIDIA_GEFORCE_GTX480: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX480; // island enum // gen:c1bd93e78622
pub const CARD_NVIDIA_GEFORCE_GTX550: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX550; // island enum // gen:d24d91adef02
pub const CARD_NVIDIA_GEFORCE_GTX560: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX560; // island enum // gen:ac5e3295dfcb
pub const CARD_NVIDIA_GEFORCE_GTX560M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX560M; // island enum // gen:6f1f8bf0cbfc
pub const CARD_NVIDIA_GEFORCE_GTX560TI: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX560TI; // island enum // gen:10053ea2cff9
pub const CARD_NVIDIA_GEFORCE_GTX570: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX570; // island enum // gen:ee8de1b37cac
pub const CARD_NVIDIA_GEFORCE_GTX580: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX580; // island enum // gen:3b828a3ea16f
pub const CARD_NVIDIA_GEFORCE_GTX650: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX650; // island enum // gen:6cd1e6048cad
pub const CARD_NVIDIA_GEFORCE_GTX650TI: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX650TI; // island enum // gen:00d37a032371
pub const CARD_NVIDIA_GEFORCE_GTX660: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX660; // island enum // gen:2e588f5ff41b
pub const CARD_NVIDIA_GEFORCE_GTX660M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX660M; // island enum // gen:c5c43d50ad0f
pub const CARD_NVIDIA_GEFORCE_GTX660TI: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX660TI; // island enum // gen:342d8a22eadc
pub const CARD_NVIDIA_GEFORCE_GTX670: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX670; // island enum // gen:f6924b71565d
pub const CARD_NVIDIA_GEFORCE_GTX670MX: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX670MX; // island enum // gen:02f21bf9ee3c
pub const CARD_NVIDIA_GEFORCE_GTX675MX_1: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX675MX_1; // island enum // gen:5e94c1fa7f67
pub const CARD_NVIDIA_GEFORCE_GTX675MX_2: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX675MX_2; // island enum // gen:f7dbee5bc35b
pub const CARD_NVIDIA_GEFORCE_GTX680: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX680; // island enum // gen:25baab45f169
pub const CARD_NVIDIA_GEFORCE_GTX690: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX690; // island enum // gen:9049869be922
pub const CARD_NVIDIA_GEFORCE_GTX750: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX750; // island enum // gen:96acfb1050a1
pub const CARD_NVIDIA_GEFORCE_GTX750TI: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX750TI; // island enum // gen:965504f711fd
pub const CARD_NVIDIA_GEFORCE_GTX760: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX760; // island enum // gen:80a3eee8e1b9
pub const CARD_NVIDIA_GEFORCE_GTX760TI: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX760TI; // island enum // gen:606871439c19
pub const CARD_NVIDIA_GEFORCE_GTX765M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX765M; // island enum // gen:fdb8eb681244
pub const CARD_NVIDIA_GEFORCE_GTX770: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX770; // island enum // gen:45ccae335a27
pub const CARD_NVIDIA_GEFORCE_GTX770M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX770M; // island enum // gen:17de31690ec5
pub const CARD_NVIDIA_GEFORCE_GTX775M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX775M; // island enum // gen:ecff12c9e86a
pub const CARD_NVIDIA_GEFORCE_GTX780: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX780; // island enum // gen:32b82a3dd587
pub const CARD_NVIDIA_GEFORCE_GTX780M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX780M; // island enum // gen:dd7516bf32e7
pub const CARD_NVIDIA_GEFORCE_GTX780TI: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX780TI; // island enum // gen:9bd63a39e01f
pub const CARD_NVIDIA_GEFORCE_GTX850M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX850M; // island enum // gen:5afa28289f96
pub const CARD_NVIDIA_GEFORCE_GTX860M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX860M; // island enum // gen:f70daf61eda9
pub const CARD_NVIDIA_GEFORCE_GTX870M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX870M; // island enum // gen:fdeec716b419
pub const CARD_NVIDIA_GEFORCE_GTX880M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX880M; // island enum // gen:457ebdfe3f7c
pub const CARD_NVIDIA_GEFORCE_GTX950: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX950; // island enum // gen:6a707daf7ac7
pub const CARD_NVIDIA_GEFORCE_GTX950M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX950M; // island enum // gen:e46c58016b4c
pub const CARD_NVIDIA_GEFORCE_GTX960: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX960; // island enum // gen:2ebb9621a386
pub const CARD_NVIDIA_GEFORCE_GTX960M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX960M; // island enum // gen:ddc2fd94bc75
pub const CARD_NVIDIA_GEFORCE_GTX970: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX970; // island enum // gen:6d0674c33593
pub const CARD_NVIDIA_GEFORCE_GTX970M: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX970M; // island enum // gen:d246ddd6a0fb
pub const CARD_NVIDIA_GEFORCE_GTX980: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX980; // island enum // gen:d0fc5c08041c
pub const CARD_NVIDIA_GEFORCE_GTX980TI: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTX980TI; // island enum // gen:5406fd39328a
pub const CARD_NVIDIA_GEFORCE_GTXTITAN: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTXTITAN; // island enum // gen:4a546402f4b3
pub const CARD_NVIDIA_GEFORCE_GTXTITANB: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTXTITANB; // island enum // gen:92a3642f1787
pub const CARD_NVIDIA_GEFORCE_GTXTITANX: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTXTITANX; // island enum // gen:b2c9831107a2
pub const CARD_NVIDIA_GEFORCE_GTXTITANZ: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_GTXTITANZ; // island enum // gen:2a25e30a4fc3
pub const CARD_NVIDIA_GEFORCE_RTX2060: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX2060; // island enum // gen:bbf44e9ed894
pub const CARD_NVIDIA_GEFORCE_RTX2070: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX2070; // island enum // gen:fa876db099a9
pub const CARD_NVIDIA_GEFORCE_RTX2080: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX2080; // island enum // gen:7f5adbf9365a
pub const CARD_NVIDIA_GEFORCE_RTX2080TI: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX2080TI; // island enum // gen:4b87fa1e58d3
pub const CARD_NVIDIA_GEFORCE_RTX3050: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX3050; // island enum // gen:7a404057ad2b
pub const CARD_NVIDIA_GEFORCE_RTX3060: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX3060; // island enum // gen:9d84eb76c0cf
pub const CARD_NVIDIA_GEFORCE_RTX3060TI_GA103: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX3060TI_GA103; // island enum // gen:7fc18bf6e01a
pub const CARD_NVIDIA_GEFORCE_RTX3060TI_GA104: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX3060TI_GA104; // island enum // gen:3512b7859666
pub const CARD_NVIDIA_GEFORCE_RTX3060TI_GA104_LHR: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX3060TI_GA104_LHR; // island enum // gen:79ef85ad411a
pub const CARD_NVIDIA_GEFORCE_RTX3060_LHR: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX3060_LHR; // island enum // gen:f5c6841a4548
pub const CARD_NVIDIA_GEFORCE_RTX3070: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX3070; // island enum // gen:7744a3c5e1f8
pub const CARD_NVIDIA_GEFORCE_RTX3070TI: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX3070TI; // island enum // gen:2265363b52bf
pub const CARD_NVIDIA_GEFORCE_RTX3070_LHR: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX3070_LHR; // island enum // gen:c533d16bded5
pub const CARD_NVIDIA_GEFORCE_RTX3070_MOBILE: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX3070_MOBILE; // island enum // gen:4c23c006d32b
pub const CARD_NVIDIA_GEFORCE_RTX3080TI: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX3080TI; // island enum // gen:d118f9544866
pub const CARD_NVIDIA_GEFORCE_RTX3080_10GB: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX3080_10GB; // island enum // gen:7e0be40c87c0
pub const CARD_NVIDIA_GEFORCE_RTX3080_10GB_LHR: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX3080_10GB_LHR; // island enum // gen:998d82351e23
pub const CARD_NVIDIA_GEFORCE_RTX3080_12GB: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX3080_12GB; // island enum // gen:1cd580e156af
pub const CARD_NVIDIA_GEFORCE_RTX3090: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX3090; // island enum // gen:515ba2621cc5
pub const CARD_NVIDIA_GEFORCE_RTX3090TI: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX3090TI; // island enum // gen:6d67c1554865
pub const CARD_NVIDIA_GEFORCE_RTX4060: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX4060; // island enum // gen:2ee66b929b2e
pub const CARD_NVIDIA_GEFORCE_RTX4060TI16G: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX4060TI16G; // island enum // gen:6aa40abada82
pub const CARD_NVIDIA_GEFORCE_RTX4060TI8G: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX4060TI8G; // island enum // gen:716c302fda8e
pub const CARD_NVIDIA_GEFORCE_RTX4070: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX4070; // island enum // gen:9ca2452bbedc
pub const CARD_NVIDIA_GEFORCE_RTX4070SUPER: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX4070SUPER; // island enum // gen:f7d0903d95cf
pub const CARD_NVIDIA_GEFORCE_RTX4070TI: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX4070TI; // island enum // gen:ad358c948571
pub const CARD_NVIDIA_GEFORCE_RTX4070TISUPER: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX4070TISUPER; // island enum // gen:e9b208756e06
pub const CARD_NVIDIA_GEFORCE_RTX4080: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX4080; // island enum // gen:3255d3b83c89
pub const CARD_NVIDIA_GEFORCE_RTX4080SUPER: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX4080SUPER; // island enum // gen:3628ae13af11
pub const CARD_NVIDIA_GEFORCE_RTX4090: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_GEFORCE_RTX4090; // island enum // gen:bdab5a4792fa
pub const CARD_NVIDIA_RIVA_128: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_RIVA_128; // island enum // gen:ef49ed30f592
pub const CARD_NVIDIA_RIVA_TNT2: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_RIVA_TNT2; // island enum // gen:3481b0830a0a
pub const CARD_NVIDIA_RIVA_TNT: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_RIVA_TNT; // island enum // gen:7b1644256fc3
pub const CARD_NVIDIA_TESLA_T4: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_TESLA_T4; // island enum // gen:0cad3a6f3d8a
pub const CARD_NVIDIA_TITANV: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_TITANV; // island enum // gen:22e6b158860e
pub const CARD_NVIDIA_TITANX_PASCAL: wined3d_pci_device = wined3d_pci_device_CARD_NVIDIA_TITANX_PASCAL; // island enum // gen:36c35650677b
pub const CARD_REDHAT_VIRGL: wined3d_pci_device = wined3d_pci_device_CARD_REDHAT_VIRGL; // island enum // gen:55f7af3f15f8
pub const CARD_VMWARE_SVGA3D: wined3d_pci_device = wined3d_pci_device_CARD_VMWARE_SVGA3D; // island enum // gen:f92a910dd375
pub const CARD_WINE: wined3d_pci_device = wined3d_pci_device_CARD_WINE; // island enum // gen:8b5d7546aa8b
pub const CHANNEL_SOURCE_COMPLEX0: fixup_channel_source = fixup_channel_source_CHANNEL_SOURCE_COMPLEX0; // island enum // gen:ebdc4fefdcb5
pub const CHANNEL_SOURCE_COMPLEX1: fixup_channel_source = fixup_channel_source_CHANNEL_SOURCE_COMPLEX1; // island enum // gen:75d777b3fb9e
pub const CHANNEL_SOURCE_ONE: fixup_channel_source = fixup_channel_source_CHANNEL_SOURCE_ONE; // island enum // gen:f34607a5a06a
pub const CHANNEL_SOURCE_W: fixup_channel_source = fixup_channel_source_CHANNEL_SOURCE_W; // island enum // gen:da1d42f07093
pub const CHANNEL_SOURCE_X: fixup_channel_source = fixup_channel_source_CHANNEL_SOURCE_X; // island enum // gen:c0b0d6e8e161
pub const CHANNEL_SOURCE_Y: fixup_channel_source = fixup_channel_source_CHANNEL_SOURCE_Y; // island enum // gen:64ebf07db884
pub const CHANNEL_SOURCE_Z: fixup_channel_source = fixup_channel_source_CHANNEL_SOURCE_Z; // island enum // gen:4a8415a88698
pub const CHANNEL_SOURCE_ZERO: fixup_channel_source = fixup_channel_source_CHANNEL_SOURCE_ZERO; // island enum // gen:c7f2011ce833
pub const CHAR_BIT: u32 = 8; // mspaint-rust/src/prelude.rs // gen:949c91f18890
pub const COMPLEX_FIXUP_NONE: complex_fixup = complex_fixup_COMPLEX_FIXUP_NONE; // island enum // gen:82312365207a
pub const COMPLEX_FIXUP_NV12: complex_fixup = complex_fixup_COMPLEX_FIXUP_NV12; // island enum // gen:01ac1a73faef
pub const COMPLEX_FIXUP_P8: complex_fixup = complex_fixup_COMPLEX_FIXUP_P8; // island enum // gen:6106f27dedf9
pub const COMPLEX_FIXUP_UYVY: complex_fixup = complex_fixup_COMPLEX_FIXUP_UYVY; // island enum // gen:88bab3080c21
pub const COMPLEX_FIXUP_YUV: complex_fixup = complex_fixup_COMPLEX_FIXUP_YUV; // island enum // gen:52c04ea17c02
pub const COMPLEX_FIXUP_YUY2: complex_fixup = complex_fixup_COMPLEX_FIXUP_YUY2; // island enum // gen:5cbe88c43eee
pub const COMPLEX_FIXUP_YV12: complex_fixup = complex_fixup_COMPLEX_FIXUP_YV12; // island enum // gen:269855363218
pub const D3DDDIFMT_A1R5G5B5: _D3DDDIFORMAT = _D3DDDIFORMAT_D3DDDIFMT_A1R5G5B5; // island enum // gen:79b307915ef9
pub const D3DDDIFMT_A4R4G4B4: _D3DDDIFORMAT = _D3DDDIFORMAT_D3DDDIFMT_A4R4G4B4; // island enum // gen:f70471630fe4
pub const D3DDDIFMT_A8R8G8B8: _D3DDDIFORMAT = _D3DDDIFORMAT_D3DDDIFMT_A8R8G8B8; // island enum // gen:783b4b9388f3
pub const D3DDDIFMT_P8: _D3DDDIFORMAT = _D3DDDIFORMAT_D3DDDIFMT_P8; // island enum // gen:cad1a4753363
pub const D3DDDIFMT_R5G6B5: _D3DDDIFORMAT = _D3DDDIFORMAT_D3DDDIFMT_R5G6B5; // island enum // gen:8715656f58c8
pub const D3DDDIFMT_R8G8B8: _D3DDDIFORMAT = _D3DDDIFORMAT_D3DDDIFMT_R8G8B8; // island enum // gen:1b6cf89823f6
pub const D3DDDIFMT_X1R5G5B5: _D3DDDIFORMAT = _D3DDDIFORMAT_D3DDDIFMT_X1R5G5B5; // island enum // gen:941735a3833b
pub const D3DDDIFMT_X4R4G4B4: _D3DDDIFORMAT = _D3DDDIFORMAT_D3DDDIFMT_X4R4G4B4; // island enum // gen:58adbeddf8f6
pub const D3DDDIFMT_X8R8G8B8: _D3DDDIFORMAT = _D3DDDIFORMAT_D3DDDIFMT_X8R8G8B8; // island enum // gen:f81202601d7a
pub const D3DKMT_VIDPNSOURCEOWNER_EXCLUSIVE: _D3DKMT_VIDPNSOURCEOWNER_TYPE = _D3DKMT_VIDPNSOURCEOWNER_TYPE_D3DKMT_VIDPNSOURCEOWNER_EXCLUSIVE; // island enum // gen:ddf1d6b4d5d2
pub const D3DKMT_VIDPNSOURCEOWNER_SHARED: _D3DKMT_VIDPNSOURCEOWNER_TYPE = _D3DKMT_VIDPNSOURCEOWNER_TYPE_D3DKMT_VIDPNSOURCEOWNER_SHARED; // island enum // gen:d31c5d7062fe
pub const DCX_USESTYLE: u32 = 65536; // comctl32-rust/src/types_amd64.rs // gen:3a026749848d
pub const DRIVER_AMD_R100: wined3d_display_driver = wined3d_display_driver_DRIVER_AMD_R100; // island enum // gen:eaf63381d678
pub const DRIVER_AMD_R300: wined3d_display_driver = wined3d_display_driver_DRIVER_AMD_R300; // island enum // gen:023b18b879c0
pub const DRIVER_AMD_R600: wined3d_display_driver = wined3d_display_driver_DRIVER_AMD_R600; // island enum // gen:43d4e02671fa
pub const DRIVER_AMD_RAGE_128PRO: wined3d_display_driver = wined3d_display_driver_DRIVER_AMD_RAGE_128PRO; // island enum // gen:00440d878985
pub const DRIVER_AMD_RX: wined3d_display_driver = wined3d_display_driver_DRIVER_AMD_RX; // island enum // gen:c25217648588
pub const DRIVER_INTEL_GMA3000: wined3d_display_driver = wined3d_display_driver_DRIVER_INTEL_GMA3000; // island enum // gen:198982bf8f59
pub const DRIVER_INTEL_GMA800: wined3d_display_driver = wined3d_display_driver_DRIVER_INTEL_GMA800; // island enum // gen:96728f87b2f6
pub const DRIVER_INTEL_GMA900: wined3d_display_driver = wined3d_display_driver_DRIVER_INTEL_GMA900; // island enum // gen:5f25b57d3efb
pub const DRIVER_INTEL_GMA950: wined3d_display_driver = wined3d_display_driver_DRIVER_INTEL_GMA950; // island enum // gen:cf64b59760f3
pub const DRIVER_INTEL_HD4000: wined3d_display_driver = wined3d_display_driver_DRIVER_INTEL_HD4000; // island enum // gen:23cb2d8137ce
pub const DRIVER_NVIDIA_FERMI: wined3d_display_driver = wined3d_display_driver_DRIVER_NVIDIA_FERMI; // island enum // gen:c2897969ade4
pub const DRIVER_NVIDIA_GEFORCE2MX: wined3d_display_driver = wined3d_display_driver_DRIVER_NVIDIA_GEFORCE2MX; // island enum // gen:12a8be659756
pub const DRIVER_NVIDIA_GEFORCE6: wined3d_display_driver = wined3d_display_driver_DRIVER_NVIDIA_GEFORCE6; // island enum // gen:fcc0205982ab
pub const DRIVER_NVIDIA_GEFORCE8: wined3d_display_driver = wined3d_display_driver_DRIVER_NVIDIA_GEFORCE8; // island enum // gen:f06518914d3d
pub const DRIVER_NVIDIA_GEFORCEFX: wined3d_display_driver = wined3d_display_driver_DRIVER_NVIDIA_GEFORCEFX; // island enum // gen:fd59cec7c8c2
pub const DRIVER_NVIDIA_KEPLER: wined3d_display_driver = wined3d_display_driver_DRIVER_NVIDIA_KEPLER; // island enum // gen:31e5a6246a26
pub const DRIVER_NVIDIA_TNT: wined3d_display_driver = wined3d_display_driver_DRIVER_NVIDIA_TNT; // island enum // gen:81db9bd60924
pub const DRIVER_REDHAT_VIRGL: wined3d_display_driver = wined3d_display_driver_DRIVER_REDHAT_VIRGL; // island enum // gen:ab08109df3d5
pub const DRIVER_VMWARE: wined3d_display_driver = wined3d_display_driver_DRIVER_VMWARE; // island enum // gen:b0e8d62a1036
pub const DRIVER_WINE: wined3d_display_driver = wined3d_display_driver_DRIVER_WINE; // island enum // gen:23b486b371f7
pub const EXT_BLEND_COLOR: wined3d_gl_extension = wined3d_gl_extension_EXT_BLEND_COLOR; // island enum // gen:8ac8939838ba
pub const EXT_BLEND_EQUATION_SEPARATE: wined3d_gl_extension = wined3d_gl_extension_EXT_BLEND_EQUATION_SEPARATE; // island enum // gen:e6d5dc654d1e
pub const EXT_BLEND_FUNC_SEPARATE: wined3d_gl_extension = wined3d_gl_extension_EXT_BLEND_FUNC_SEPARATE; // island enum // gen:2cb37626205d
pub const EXT_BLEND_MINMAX: wined3d_gl_extension = wined3d_gl_extension_EXT_BLEND_MINMAX; // island enum // gen:45260984de1b
pub const EXT_BLEND_SUBTRACT: wined3d_gl_extension = wined3d_gl_extension_EXT_BLEND_SUBTRACT; // island enum // gen:d4158ce59781
pub const EXT_DEPTH_BOUNDS_TEST: wined3d_gl_extension = wined3d_gl_extension_EXT_DEPTH_BOUNDS_TEST; // island enum // gen:99cd03cee83e
pub const EXT_DRAW_BUFFERS2: wined3d_gl_extension = wined3d_gl_extension_EXT_DRAW_BUFFERS2; // island enum // gen:5689ea1a1f00
pub const EXT_FOG_COORD: wined3d_gl_extension = wined3d_gl_extension_EXT_FOG_COORD; // island enum // gen:680ab0c126be
pub const EXT_FRAMEBUFFER_BLIT: wined3d_gl_extension = wined3d_gl_extension_EXT_FRAMEBUFFER_BLIT; // island enum // gen:6f2c012dfc7b
pub const EXT_FRAMEBUFFER_MULTISAMPLE: wined3d_gl_extension = wined3d_gl_extension_EXT_FRAMEBUFFER_MULTISAMPLE; // island enum // gen:a9771b57b68e
pub const EXT_FRAMEBUFFER_MULTISAMPLE_BLIT_SCALED: wined3d_gl_extension = wined3d_gl_extension_EXT_FRAMEBUFFER_MULTISAMPLE_BLIT_SCALED; // island enum // gen:f11e913a5efe
pub const EXT_FRAMEBUFFER_OBJECT: wined3d_gl_extension = wined3d_gl_extension_EXT_FRAMEBUFFER_OBJECT; // island enum // gen:56b3f8d196dd
pub const EXT_GPU_PROGRAM_PARAMETERS: wined3d_gl_extension = wined3d_gl_extension_EXT_GPU_PROGRAM_PARAMETERS; // island enum // gen:0b4e1a38114b
pub const EXT_GPU_SHADER4: wined3d_gl_extension = wined3d_gl_extension_EXT_GPU_SHADER4; // island enum // gen:72813dd1d994
pub const EXT_MEMORY_OBJECT: wined3d_gl_extension = wined3d_gl_extension_EXT_MEMORY_OBJECT; // island enum // gen:2a74cffbe5d6
pub const EXT_PACKED_DEPTH_STENCIL: wined3d_gl_extension = wined3d_gl_extension_EXT_PACKED_DEPTH_STENCIL; // island enum // gen:d195aaeb477e
pub const EXT_PACKED_FLOAT: wined3d_gl_extension = wined3d_gl_extension_EXT_PACKED_FLOAT; // island enum // gen:18f404787f22
pub const EXT_POINT_PARAMETERS: wined3d_gl_extension = wined3d_gl_extension_EXT_POINT_PARAMETERS; // island enum // gen:2d8e1c0a7cb5
pub const EXT_PROVOKING_VERTEX: wined3d_gl_extension = wined3d_gl_extension_EXT_PROVOKING_VERTEX; // island enum // gen:0266269f808a
pub const EXT_SECONDARY_COLOR: wined3d_gl_extension = wined3d_gl_extension_EXT_SECONDARY_COLOR; // island enum // gen:b20b8a58bf10
pub const EXT_SHADER_INTEGER_MIX: wined3d_gl_extension = wined3d_gl_extension_EXT_SHADER_INTEGER_MIX; // island enum // gen:3bd091864d9b
pub const EXT_STENCIL_TWO_SIDE: wined3d_gl_extension = wined3d_gl_extension_EXT_STENCIL_TWO_SIDE; // island enum // gen:5a854df5d596
pub const EXT_STENCIL_WRAP: wined3d_gl_extension = wined3d_gl_extension_EXT_STENCIL_WRAP; // island enum // gen:b34528288bac
pub const EXT_TEXTURE3D: wined3d_gl_extension = wined3d_gl_extension_EXT_TEXTURE3D; // island enum // gen:1e2691ccd343
pub const EXT_TEXTURE_ARRAY: wined3d_gl_extension = wined3d_gl_extension_EXT_TEXTURE_ARRAY; // island enum // gen:1b3ca9e61285
pub const EXT_TEXTURE_COMPRESSION_RGTC: wined3d_gl_extension = wined3d_gl_extension_EXT_TEXTURE_COMPRESSION_RGTC; // island enum // gen:77d75c6d7e0a
pub const EXT_TEXTURE_COMPRESSION_S3TC: wined3d_gl_extension = wined3d_gl_extension_EXT_TEXTURE_COMPRESSION_S3TC; // island enum // gen:2ae6b228800c
pub const EXT_TEXTURE_ENV_COMBINE: wined3d_gl_extension = wined3d_gl_extension_EXT_TEXTURE_ENV_COMBINE; // island enum // gen:c9070c51b4fa
pub const EXT_TEXTURE_ENV_DOT3: wined3d_gl_extension = wined3d_gl_extension_EXT_TEXTURE_ENV_DOT3; // island enum // gen:2f5429afb3c8
pub const EXT_TEXTURE_INTEGER: wined3d_gl_extension = wined3d_gl_extension_EXT_TEXTURE_INTEGER; // island enum // gen:95c475da8505
pub const EXT_TEXTURE_LOD_BIAS: wined3d_gl_extension = wined3d_gl_extension_EXT_TEXTURE_LOD_BIAS; // island enum // gen:a4e87e98994a
pub const EXT_TEXTURE_MIRROR_CLAMP: wined3d_gl_extension = wined3d_gl_extension_EXT_TEXTURE_MIRROR_CLAMP; // island enum // gen:edd7d4be8b88
pub const EXT_TEXTURE_SHADOW_LOD: wined3d_gl_extension = wined3d_gl_extension_EXT_TEXTURE_SHADOW_LOD; // island enum // gen:7f9adf769784
pub const EXT_TEXTURE_SHARED_EXPONENT: wined3d_gl_extension = wined3d_gl_extension_EXT_TEXTURE_SHARED_EXPONENT; // island enum // gen:3c0dd853e252
pub const EXT_TEXTURE_SNORM: wined3d_gl_extension = wined3d_gl_extension_EXT_TEXTURE_SNORM; // island enum // gen:ed4bede6d9ca
pub const EXT_TEXTURE_SRGB: wined3d_gl_extension = wined3d_gl_extension_EXT_TEXTURE_SRGB; // island enum // gen:2a7159a23562
pub const EXT_TEXTURE_SRGB_DECODE: wined3d_gl_extension = wined3d_gl_extension_EXT_TEXTURE_SRGB_DECODE; // island enum // gen:0e95d9fde646
pub const FOGSOURCE_FFP: fogsource = fogsource_FOGSOURCE_FFP; // island enum // gen:324bd73234e5
pub const FOGSOURCE_VS: fogsource = fogsource_FOGSOURCE_VS; // island enum // gen:2dce0f7eaea8
pub const HW_VENDOR_AMD: wined3d_pci_vendor = wined3d_pci_vendor_HW_VENDOR_AMD; // island enum // gen:1322ad31e730
pub const HW_VENDOR_INTEL: wined3d_pci_vendor = wined3d_pci_vendor_HW_VENDOR_INTEL; // island enum // gen:3f96109dc590
pub const HW_VENDOR_NVIDIA: wined3d_pci_vendor = wined3d_pci_vendor_HW_VENDOR_NVIDIA; // island enum // gen:be554ec3faef
pub const HW_VENDOR_REDHAT: wined3d_pci_vendor = wined3d_pci_vendor_HW_VENDOR_REDHAT; // island enum // gen:4d4476eba6f0
pub const HW_VENDOR_SOFTWARE: wined3d_pci_vendor = wined3d_pci_vendor_HW_VENDOR_SOFTWARE; // island enum // gen:db8f7da334ec
pub const HW_VENDOR_VMWARE: wined3d_pci_vendor = wined3d_pci_vendor_HW_VENDOR_VMWARE; // island enum // gen:16b6186a06ca
pub const LONG_MAX: u32 = 2147483647; // msv1_0-rust/src/types_amd64.rs // gen:446779415045
pub const NV_FENCE: wined3d_gl_extension = wined3d_gl_extension_NV_FENCE; // island enum // gen:febb60609d47
pub const NV_FOG_DISTANCE: wined3d_gl_extension = wined3d_gl_extension_NV_FOG_DISTANCE; // island enum // gen:bda089d965db
pub const NV_FRAGMENT_PROGRAM2: wined3d_gl_extension = wined3d_gl_extension_NV_FRAGMENT_PROGRAM2; // island enum // gen:c99481695ea7
pub const NV_FRAGMENT_PROGRAM: wined3d_gl_extension = wined3d_gl_extension_NV_FRAGMENT_PROGRAM; // island enum // gen:2ed31685419b
pub const NV_FRAGMENT_PROGRAM_OPTION: wined3d_gl_extension = wined3d_gl_extension_NV_FRAGMENT_PROGRAM_OPTION; // island enum // gen:62e50a5e72d1
pub const NV_HALF_FLOAT: wined3d_gl_extension = wined3d_gl_extension_NV_HALF_FLOAT; // island enum // gen:e9d55dd75fbd
pub const NV_LIGHT_MAX_EXPONENT: wined3d_gl_extension = wined3d_gl_extension_NV_LIGHT_MAX_EXPONENT; // island enum // gen:c20a62c148aa
pub const NV_POINT_SPRITE: wined3d_gl_extension = wined3d_gl_extension_NV_POINT_SPRITE; // island enum // gen:efc354d743b6
pub const NV_REGISTER_COMBINERS2: wined3d_gl_extension = wined3d_gl_extension_NV_REGISTER_COMBINERS2; // island enum // gen:45c98625486a
pub const NV_REGISTER_COMBINERS: wined3d_gl_extension = wined3d_gl_extension_NV_REGISTER_COMBINERS; // island enum // gen:4ad6c25e387c
pub const NV_TEXGEN_REFLECTION: wined3d_gl_extension = wined3d_gl_extension_NV_TEXGEN_REFLECTION; // island enum // gen:3916298c88d7
pub const NV_TEXTURE_BARRIER: wined3d_gl_extension = wined3d_gl_extension_NV_TEXTURE_BARRIER; // island enum // gen:27834b41c6c2
pub const NV_TEXTURE_ENV_COMBINE4: wined3d_gl_extension = wined3d_gl_extension_NV_TEXTURE_ENV_COMBINE4; // island enum // gen:e69c6512ad47
pub const NV_TEXTURE_SHADER2: wined3d_gl_extension = wined3d_gl_extension_NV_TEXTURE_SHADER2; // island enum // gen:a6516f9c99a0
pub const NV_TEXTURE_SHADER: wined3d_gl_extension = wined3d_gl_extension_NV_TEXTURE_SHADER; // island enum // gen:7176cbc0f81c
pub const NV_VERTEX_PROGRAM1_1: wined3d_gl_extension = wined3d_gl_extension_NV_VERTEX_PROGRAM1_1; // island enum // gen:1ed7443788c4
pub const NV_VERTEX_PROGRAM2: wined3d_gl_extension = wined3d_gl_extension_NV_VERTEX_PROGRAM2; // island enum // gen:b062eaa17250
pub const NV_VERTEX_PROGRAM2_OPTION: wined3d_gl_extension = wined3d_gl_extension_NV_VERTEX_PROGRAM2_OPTION; // island enum // gen:11738bdb0173
pub const NV_VERTEX_PROGRAM3: wined3d_gl_extension = wined3d_gl_extension_NV_VERTEX_PROGRAM3; // island enum // gen:4a580c030885
pub const NV_VERTEX_PROGRAM: wined3d_gl_extension = wined3d_gl_extension_NV_VERTEX_PROGRAM; // island enum // gen:1256895dfd60
pub const QUERY_BUILDING: wined3d_query_state = wined3d_query_state_QUERY_BUILDING; // island enum // gen:dd592429bb8b
pub const QUERY_CREATED: wined3d_query_state = wined3d_query_state_QUERY_CREATED; // island enum // gen:e336818fdae7
pub const QUERY_SIGNALLED: wined3d_query_state = wined3d_query_state_QUERY_SIGNALLED; // island enum // gen:9bcca5abc1d6
pub const TLS_OUT_OF_INDEXES: DWORD = 0xFFFF_FFFF; // menuband-rust/src/prelude.rs // gen:08ba168bc7a9
pub const UINT8_MAX: u32 = 255; // mf-rust/src/types_amd64.rs // gen:366f8ddf8244
pub const UINT_MAX: u32 = 4294967295; // comctl32-rust/src/prelude.rs // gen:1b06153cd4a1
pub const VKD3D_SHADER_API_VERSION_1_12: vkd3d_shader_api_version = vkd3d_shader_api_version_VKD3D_SHADER_API_VERSION_1_12; // island enum // gen:383b92ba3449
pub const VKD3D_SHADER_API_VERSION_1_6: vkd3d_shader_api_version = vkd3d_shader_api_version_VKD3D_SHADER_API_VERSION_1_6; // island enum // gen:6b1f14db4d8c
pub const VKD3D_SHADER_API_VERSION_1_7: vkd3d_shader_api_version = vkd3d_shader_api_version_VKD3D_SHADER_API_VERSION_1_7; // island enum // gen:d97e2bb39f80
pub const VKD3D_SHADER_BINDING_FLAG_BUFFER: vkd3d_shader_binding_flag = vkd3d_shader_binding_flag_VKD3D_SHADER_BINDING_FLAG_BUFFER; // island enum // gen:11a3aecd770f
pub const VKD3D_SHADER_BINDING_FLAG_IMAGE: vkd3d_shader_binding_flag = vkd3d_shader_binding_flag_VKD3D_SHADER_BINDING_FLAG_IMAGE; // island enum // gen:a7fd8fc999dc
pub const VKD3D_SHADER_COMPILE_OPTION_API_VERSION: vkd3d_shader_compile_option_name = vkd3d_shader_compile_option_name_VKD3D_SHADER_COMPILE_OPTION_API_VERSION; // island enum // gen:3791ef73f627
pub const VKD3D_SHADER_COMPILE_OPTION_WRITE_TESS_GEOM_POINT_SIZE: vkd3d_shader_compile_option_name = vkd3d_shader_compile_option_name_VKD3D_SHADER_COMPILE_OPTION_WRITE_TESS_GEOM_POINT_SIZE; // island enum // gen:0249b4b4f95a
pub const VKD3D_SHADER_D3DBC_BOOL_CONSTANT_REGISTER: vkd3d_shader_d3dbc_constant_register = vkd3d_shader_d3dbc_constant_register_VKD3D_SHADER_D3DBC_BOOL_CONSTANT_REGISTER; // island enum // gen:5057c84d1d4a
pub const VKD3D_SHADER_D3DBC_FLOAT_CONSTANT_REGISTER: vkd3d_shader_d3dbc_constant_register = vkd3d_shader_d3dbc_constant_register_VKD3D_SHADER_D3DBC_FLOAT_CONSTANT_REGISTER; // island enum // gen:cf207c6f9b15
pub const VKD3D_SHADER_D3DBC_INT_CONSTANT_REGISTER: vkd3d_shader_d3dbc_constant_register = vkd3d_shader_d3dbc_constant_register_VKD3D_SHADER_D3DBC_INT_CONSTANT_REGISTER; // island enum // gen:3597aabb5bc1
pub const VKD3D_SHADER_DESCRIPTOR_INFO_FLAG_UAV_COUNTER: vkd3d_shader_descriptor_info_flag = vkd3d_shader_descriptor_info_flag_VKD3D_SHADER_DESCRIPTOR_INFO_FLAG_UAV_COUNTER; // island enum // gen:6cbbcee5a81e
pub const VKD3D_SHADER_DESCRIPTOR_TYPE_CBV: vkd3d_shader_descriptor_type = vkd3d_shader_descriptor_type_VKD3D_SHADER_DESCRIPTOR_TYPE_CBV; // island enum // gen:ff9442221968
pub const VKD3D_SHADER_DESCRIPTOR_TYPE_SAMPLER: vkd3d_shader_descriptor_type = vkd3d_shader_descriptor_type_VKD3D_SHADER_DESCRIPTOR_TYPE_SAMPLER; // island enum // gen:dc91a9c82dff
pub const VKD3D_SHADER_DESCRIPTOR_TYPE_SRV: vkd3d_shader_descriptor_type = vkd3d_shader_descriptor_type_VKD3D_SHADER_DESCRIPTOR_TYPE_SRV; // island enum // gen:b21601ed09b4
pub const VKD3D_SHADER_DESCRIPTOR_TYPE_UAV: vkd3d_shader_descriptor_type = vkd3d_shader_descriptor_type_VKD3D_SHADER_DESCRIPTOR_TYPE_UAV; // island enum // gen:05005e5b1be2
pub const VKD3D_SHADER_LOG_NONE: vkd3d_shader_log_level = vkd3d_shader_log_level_VKD3D_SHADER_LOG_NONE; // island enum // gen:80a28770486b
pub const VKD3D_SHADER_LOG_WARNING: vkd3d_shader_log_level = vkd3d_shader_log_level_VKD3D_SHADER_LOG_WARNING; // island enum // gen:093bcb4b5eec
pub const VKD3D_SHADER_PARAMETER_DATA_TYPE_FLOAT32: vkd3d_shader_parameter_data_type = vkd3d_shader_parameter_data_type_VKD3D_SHADER_PARAMETER_DATA_TYPE_FLOAT32; // island enum // gen:983d845ac234
pub const VKD3D_SHADER_PARAMETER_DATA_TYPE_UINT32: vkd3d_shader_parameter_data_type = vkd3d_shader_parameter_data_type_VKD3D_SHADER_PARAMETER_DATA_TYPE_UINT32; // island enum // gen:f6a645f12b16
pub const VKD3D_SHADER_PARAMETER_NAME_ALPHA_TEST_FUNC: vkd3d_shader_parameter_name = vkd3d_shader_parameter_name_VKD3D_SHADER_PARAMETER_NAME_ALPHA_TEST_FUNC; // island enum // gen:70b4de9663b7
pub const VKD3D_SHADER_PARAMETER_NAME_ALPHA_TEST_REF: vkd3d_shader_parameter_name = vkd3d_shader_parameter_name_VKD3D_SHADER_PARAMETER_NAME_ALPHA_TEST_REF; // island enum // gen:5f641a84bba8
pub const VKD3D_SHADER_PARAMETER_NAME_FLAT_INTERPOLATION: vkd3d_shader_parameter_name = vkd3d_shader_parameter_name_VKD3D_SHADER_PARAMETER_NAME_FLAT_INTERPOLATION; // island enum // gen:0348540d5c25
pub const VKD3D_SHADER_PARAMETER_NAME_RASTERIZER_SAMPLE_COUNT: vkd3d_shader_parameter_name = vkd3d_shader_parameter_name_VKD3D_SHADER_PARAMETER_NAME_RASTERIZER_SAMPLE_COUNT; // island enum // gen:8e5b837b69bf
pub const VKD3D_SHADER_PARAMETER_TYPE_BUFFER: vkd3d_shader_parameter_type = vkd3d_shader_parameter_type_VKD3D_SHADER_PARAMETER_TYPE_BUFFER; // island enum // gen:2c0a77d2289e
pub const VKD3D_SHADER_PARAMETER_TYPE_IMMEDIATE_CONSTANT: vkd3d_shader_parameter_type = vkd3d_shader_parameter_type_VKD3D_SHADER_PARAMETER_TYPE_IMMEDIATE_CONSTANT; // island enum // gen:9d0ebb5f19bc
pub const VKD3D_SHADER_RESOURCE_BUFFER: vkd3d_shader_resource_type = vkd3d_shader_resource_type_VKD3D_SHADER_RESOURCE_BUFFER; // island enum // gen:b6f73db3290a
pub const VKD3D_SHADER_RESOURCE_DATA_FLOAT: vkd3d_shader_resource_data_type = vkd3d_shader_resource_data_type_VKD3D_SHADER_RESOURCE_DATA_FLOAT; // island enum // gen:1835078c95d4
pub const VKD3D_SHADER_RESOURCE_DATA_INT: vkd3d_shader_resource_data_type = vkd3d_shader_resource_data_type_VKD3D_SHADER_RESOURCE_DATA_INT; // island enum // gen:9a1ec81f8ae6
pub const VKD3D_SHADER_RESOURCE_DATA_MIXED: vkd3d_shader_resource_data_type = vkd3d_shader_resource_data_type_VKD3D_SHADER_RESOURCE_DATA_MIXED; // island enum // gen:dbdd3625796f
pub const VKD3D_SHADER_RESOURCE_DATA_SNORM: vkd3d_shader_resource_data_type = vkd3d_shader_resource_data_type_VKD3D_SHADER_RESOURCE_DATA_SNORM; // island enum // gen:57c7d54107a8
pub const VKD3D_SHADER_RESOURCE_DATA_UINT: vkd3d_shader_resource_data_type = vkd3d_shader_resource_data_type_VKD3D_SHADER_RESOURCE_DATA_UINT; // island enum // gen:9d2bf2cd0b8d
pub const VKD3D_SHADER_RESOURCE_DATA_UNORM: vkd3d_shader_resource_data_type = vkd3d_shader_resource_data_type_VKD3D_SHADER_RESOURCE_DATA_UNORM; // island enum // gen:dee62281cd5a
pub const VKD3D_SHADER_SOURCE_D3D_BYTECODE: vkd3d_shader_source_type = vkd3d_shader_source_type_VKD3D_SHADER_SOURCE_D3D_BYTECODE; // island enum // gen:6ef851cb85ad
pub const VKD3D_SHADER_SOURCE_DXBC_TPF: vkd3d_shader_source_type = vkd3d_shader_source_type_VKD3D_SHADER_SOURCE_DXBC_TPF; // island enum // gen:b4ebef77e9d6
pub const VKD3D_SHADER_SOURCE_HLSL: vkd3d_shader_source_type = vkd3d_shader_source_type_VKD3D_SHADER_SOURCE_HLSL; // island enum // gen:ae9e3089226a
pub const VKD3D_SHADER_SOURCE_NONE: vkd3d_shader_source_type = vkd3d_shader_source_type_VKD3D_SHADER_SOURCE_NONE; // island enum // gen:f5c4f8b36ba3
pub const VKD3D_SHADER_SPIRV_ENVIRONMENT_VULKAN_1_0: vkd3d_shader_spirv_environment = vkd3d_shader_spirv_environment_VKD3D_SHADER_SPIRV_ENVIRONMENT_VULKAN_1_0; // island enum // gen:ea0898ee6ae0
pub const VKD3D_SHADER_SPIRV_EXTENSION_EXT_STENCIL_EXPORT: vkd3d_shader_spirv_extension = vkd3d_shader_spirv_extension_VKD3D_SHADER_SPIRV_EXTENSION_EXT_STENCIL_EXPORT; // island enum // gen:b61d4bbce0e7
pub const VKD3D_SHADER_STRUCTURE_TYPE_COMPILE_INFO: vkd3d_shader_structure_type = vkd3d_shader_structure_type_VKD3D_SHADER_STRUCTURE_TYPE_COMPILE_INFO; // island enum // gen:1c9428a25d85
pub const VKD3D_SHADER_STRUCTURE_TYPE_HLSL_SOURCE_INFO: vkd3d_shader_structure_type = vkd3d_shader_structure_type_VKD3D_SHADER_STRUCTURE_TYPE_HLSL_SOURCE_INFO; // island enum // gen:43b8af7b977b
pub const VKD3D_SHADER_STRUCTURE_TYPE_INTERFACE_INFO: vkd3d_shader_structure_type = vkd3d_shader_structure_type_VKD3D_SHADER_STRUCTURE_TYPE_INTERFACE_INFO; // island enum // gen:688b2f24a411
pub const VKD3D_SHADER_STRUCTURE_TYPE_PARAMETER_INFO: vkd3d_shader_structure_type = vkd3d_shader_structure_type_VKD3D_SHADER_STRUCTURE_TYPE_PARAMETER_INFO; // island enum // gen:db258723b515
pub const VKD3D_SHADER_STRUCTURE_TYPE_SCAN_DESCRIPTOR_INFO: vkd3d_shader_structure_type = vkd3d_shader_structure_type_VKD3D_SHADER_STRUCTURE_TYPE_SCAN_DESCRIPTOR_INFO; // island enum // gen:c91482954afd
pub const VKD3D_SHADER_STRUCTURE_TYPE_SCAN_SIGNATURE_INFO: vkd3d_shader_structure_type = vkd3d_shader_structure_type_VKD3D_SHADER_STRUCTURE_TYPE_SCAN_SIGNATURE_INFO; // island enum // gen:27619c196080
pub const VKD3D_SHADER_STRUCTURE_TYPE_SPIRV_TARGET_INFO: vkd3d_shader_structure_type = vkd3d_shader_structure_type_VKD3D_SHADER_STRUCTURE_TYPE_SPIRV_TARGET_INFO; // island enum // gen:f3e10bbc7fca
pub const VKD3D_SHADER_STRUCTURE_TYPE_TRANSFORM_FEEDBACK_INFO: vkd3d_shader_structure_type = vkd3d_shader_structure_type_VKD3D_SHADER_STRUCTURE_TYPE_TRANSFORM_FEEDBACK_INFO; // island enum // gen:2ec5df775385
pub const VKD3D_SHADER_STRUCTURE_TYPE_VARYING_MAP_INFO: vkd3d_shader_structure_type = vkd3d_shader_structure_type_VKD3D_SHADER_STRUCTURE_TYPE_VARYING_MAP_INFO; // island enum // gen:ab2d7c8770c0
pub const VKD3D_SHADER_TARGET_D3D_ASM: vkd3d_shader_target_type = vkd3d_shader_target_type_VKD3D_SHADER_TARGET_D3D_ASM; // island enum // gen:ee642f7dc85f
pub const VKD3D_SHADER_TARGET_D3D_BYTECODE: vkd3d_shader_target_type = vkd3d_shader_target_type_VKD3D_SHADER_TARGET_D3D_BYTECODE; // island enum // gen:fd79672c2ca7
pub const VKD3D_SHADER_TARGET_DXBC_TPF: vkd3d_shader_target_type = vkd3d_shader_target_type_VKD3D_SHADER_TARGET_DXBC_TPF; // island enum // gen:4c3e08665929
pub const VKD3D_SHADER_TARGET_GLSL: vkd3d_shader_target_type = vkd3d_shader_target_type_VKD3D_SHADER_TARGET_GLSL; // island enum // gen:8c5046ea0900
pub const VKD3D_SHADER_TARGET_NONE: vkd3d_shader_target_type = vkd3d_shader_target_type_VKD3D_SHADER_TARGET_NONE; // island enum // gen:c6dc91e8e6fa
pub const VKD3D_SHADER_TARGET_SPIRV_BINARY: vkd3d_shader_target_type = vkd3d_shader_target_type_VKD3D_SHADER_TARGET_SPIRV_BINARY; // island enum // gen:28ce52c53ade
pub const VKD3D_SHADER_VISIBILITY_ALL: vkd3d_shader_visibility = vkd3d_shader_visibility_VKD3D_SHADER_VISIBILITY_ALL; // island enum // gen:5655b327caa6
pub const VKD3D_SHADER_VISIBILITY_COMPUTE: vkd3d_shader_visibility = vkd3d_shader_visibility_VKD3D_SHADER_VISIBILITY_COMPUTE; // island enum // gen:137896a1458e
pub const VKD3D_SHADER_VISIBILITY_DOMAIN: vkd3d_shader_visibility = vkd3d_shader_visibility_VKD3D_SHADER_VISIBILITY_DOMAIN; // island enum // gen:14d66f646cf7
pub const VKD3D_SHADER_VISIBILITY_GEOMETRY: vkd3d_shader_visibility = vkd3d_shader_visibility_VKD3D_SHADER_VISIBILITY_GEOMETRY; // island enum // gen:a2ef92532f05
pub const VKD3D_SHADER_VISIBILITY_HULL: vkd3d_shader_visibility = vkd3d_shader_visibility_VKD3D_SHADER_VISIBILITY_HULL; // island enum // gen:4e35caa0cb0b
pub const VKD3D_SHADER_VISIBILITY_PIXEL: vkd3d_shader_visibility = vkd3d_shader_visibility_VKD3D_SHADER_VISIBILITY_PIXEL; // island enum // gen:58569903e528
pub const VKD3D_SHADER_VISIBILITY_VERTEX: vkd3d_shader_visibility = vkd3d_shader_visibility_VKD3D_SHADER_VISIBILITY_VERTEX; // island enum // gen:630caf990bbf
pub const VK_ACCESS_COLOR_ATTACHMENT_READ_BIT: VkAccessFlagBits = VkAccessFlagBits_VK_ACCESS_COLOR_ATTACHMENT_READ_BIT; // island enum // gen:03df9b291b15
pub const VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT: VkAccessFlagBits = VkAccessFlagBits_VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT; // island enum // gen:4193925af487
pub const VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_READ_BIT: VkAccessFlagBits = VkAccessFlagBits_VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_READ_BIT; // island enum // gen:bcc58237ae7c
pub const VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT: VkAccessFlagBits = VkAccessFlagBits_VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT; // island enum // gen:0a0757a6a19d
pub const VK_ACCESS_HOST_READ_BIT: VkAccessFlagBits = VkAccessFlagBits_VK_ACCESS_HOST_READ_BIT; // island enum // gen:8ab84becb7ad
pub const VK_ACCESS_INDEX_READ_BIT: VkAccessFlagBits = VkAccessFlagBits_VK_ACCESS_INDEX_READ_BIT; // island enum // gen:0da6e3302987
pub const VK_ACCESS_INDIRECT_COMMAND_READ_BIT: VkAccessFlagBits = VkAccessFlagBits_VK_ACCESS_INDIRECT_COMMAND_READ_BIT; // island enum // gen:d6ea91121b6a
pub const VK_ACCESS_INPUT_ATTACHMENT_READ_BIT: VkAccessFlagBits = VkAccessFlagBits_VK_ACCESS_INPUT_ATTACHMENT_READ_BIT; // island enum // gen:7ef5a60eb4a8
pub const VK_ACCESS_MEMORY_READ_BIT: VkAccessFlagBits = VkAccessFlagBits_VK_ACCESS_MEMORY_READ_BIT; // island enum // gen:54b3f25f7389
pub const VK_ACCESS_SHADER_READ_BIT: VkAccessFlagBits = VkAccessFlagBits_VK_ACCESS_SHADER_READ_BIT; // island enum // gen:76d9c9bdd77a
pub const VK_ACCESS_SHADER_WRITE_BIT: VkAccessFlagBits = VkAccessFlagBits_VK_ACCESS_SHADER_WRITE_BIT; // island enum // gen:2c1bfb8fe1a9
pub const VK_ACCESS_TRANSFER_READ_BIT: VkAccessFlagBits = VkAccessFlagBits_VK_ACCESS_TRANSFER_READ_BIT; // island enum // gen:5ba55f091a98
pub const VK_ACCESS_TRANSFER_WRITE_BIT: VkAccessFlagBits = VkAccessFlagBits_VK_ACCESS_TRANSFER_WRITE_BIT; // island enum // gen:5bad8884f2b0
pub const VK_ACCESS_TRANSFORM_FEEDBACK_COUNTER_READ_BIT_EXT: VkAccessFlagBits = VkAccessFlagBits_VK_ACCESS_TRANSFORM_FEEDBACK_COUNTER_READ_BIT_EXT; // island enum // gen:fec8e87d2b63
pub const VK_ACCESS_TRANSFORM_FEEDBACK_COUNTER_WRITE_BIT_EXT: VkAccessFlagBits = VkAccessFlagBits_VK_ACCESS_TRANSFORM_FEEDBACK_COUNTER_WRITE_BIT_EXT; // island enum // gen:e05d501898b5
pub const VK_ACCESS_TRANSFORM_FEEDBACK_WRITE_BIT_EXT: VkAccessFlagBits = VkAccessFlagBits_VK_ACCESS_TRANSFORM_FEEDBACK_WRITE_BIT_EXT; // island enum // gen:f9c2e2d42716
pub const VK_ACCESS_UNIFORM_READ_BIT: VkAccessFlagBits = VkAccessFlagBits_VK_ACCESS_UNIFORM_READ_BIT; // island enum // gen:31e18f927a8e
pub const VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT: VkAccessFlagBits = VkAccessFlagBits_VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT; // island enum // gen:790172b432ce
pub const VK_ATTACHMENT_LOAD_OP_CLEAR: VkAttachmentLoadOp = VkAttachmentLoadOp_VK_ATTACHMENT_LOAD_OP_CLEAR; // island enum // gen:bab99f175c60
pub const VK_ATTACHMENT_LOAD_OP_DONT_CARE: VkAttachmentLoadOp = VkAttachmentLoadOp_VK_ATTACHMENT_LOAD_OP_DONT_CARE; // island enum // gen:3c0fd366274b
pub const VK_ATTACHMENT_LOAD_OP_LOAD: VkAttachmentLoadOp = VkAttachmentLoadOp_VK_ATTACHMENT_LOAD_OP_LOAD; // island enum // gen:5186dde27cfc
pub const VK_ATTACHMENT_STORE_OP_DONT_CARE: VkAttachmentStoreOp = VkAttachmentStoreOp_VK_ATTACHMENT_STORE_OP_DONT_CARE; // island enum // gen:75106b1da6ae
pub const VK_ATTACHMENT_STORE_OP_STORE: VkAttachmentStoreOp = VkAttachmentStoreOp_VK_ATTACHMENT_STORE_OP_STORE; // island enum // gen:eeef086370c8
pub const VK_BLEND_FACTOR_CONSTANT_ALPHA: VkBlendFactor = VkBlendFactor_VK_BLEND_FACTOR_CONSTANT_ALPHA; // island enum // gen:cff9fe515927
pub const VK_BLEND_FACTOR_CONSTANT_COLOR: VkBlendFactor = VkBlendFactor_VK_BLEND_FACTOR_CONSTANT_COLOR; // island enum // gen:3580625b9b63
pub const VK_BLEND_FACTOR_DST_ALPHA: VkBlendFactor = VkBlendFactor_VK_BLEND_FACTOR_DST_ALPHA; // island enum // gen:26b7b03268a9
pub const VK_BLEND_FACTOR_DST_COLOR: VkBlendFactor = VkBlendFactor_VK_BLEND_FACTOR_DST_COLOR; // island enum // gen:f00bf0479801
pub const VK_BLEND_FACTOR_ONE: VkBlendFactor = VkBlendFactor_VK_BLEND_FACTOR_ONE; // island enum // gen:3cad056e69ca
pub const VK_BLEND_FACTOR_ONE_MINUS_CONSTANT_ALPHA: VkBlendFactor = VkBlendFactor_VK_BLEND_FACTOR_ONE_MINUS_CONSTANT_ALPHA; // island enum // gen:05df81eb521a
pub const VK_BLEND_FACTOR_ONE_MINUS_CONSTANT_COLOR: VkBlendFactor = VkBlendFactor_VK_BLEND_FACTOR_ONE_MINUS_CONSTANT_COLOR; // island enum // gen:813a56bb8d7f
pub const VK_BLEND_FACTOR_ONE_MINUS_DST_ALPHA: VkBlendFactor = VkBlendFactor_VK_BLEND_FACTOR_ONE_MINUS_DST_ALPHA; // island enum // gen:7678848ec155
pub const VK_BLEND_FACTOR_ONE_MINUS_DST_COLOR: VkBlendFactor = VkBlendFactor_VK_BLEND_FACTOR_ONE_MINUS_DST_COLOR; // island enum // gen:e2333e925d96
pub const VK_BLEND_FACTOR_ONE_MINUS_SRC1_ALPHA: VkBlendFactor = VkBlendFactor_VK_BLEND_FACTOR_ONE_MINUS_SRC1_ALPHA; // island enum // gen:d93547be394d
pub const VK_BLEND_FACTOR_ONE_MINUS_SRC1_COLOR: VkBlendFactor = VkBlendFactor_VK_BLEND_FACTOR_ONE_MINUS_SRC1_COLOR; // island enum // gen:25e7fa8eb5f7
pub const VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA: VkBlendFactor = VkBlendFactor_VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA; // island enum // gen:3d05501c366c
pub const VK_BLEND_FACTOR_ONE_MINUS_SRC_COLOR: VkBlendFactor = VkBlendFactor_VK_BLEND_FACTOR_ONE_MINUS_SRC_COLOR; // island enum // gen:fa90321aed50
pub const VK_BLEND_FACTOR_SRC1_ALPHA: VkBlendFactor = VkBlendFactor_VK_BLEND_FACTOR_SRC1_ALPHA; // island enum // gen:a305725b83bf
pub const VK_BLEND_FACTOR_SRC1_COLOR: VkBlendFactor = VkBlendFactor_VK_BLEND_FACTOR_SRC1_COLOR; // island enum // gen:313af662ec50
pub const VK_BLEND_FACTOR_SRC_ALPHA: VkBlendFactor = VkBlendFactor_VK_BLEND_FACTOR_SRC_ALPHA; // island enum // gen:ee6ae9729273
pub const VK_BLEND_FACTOR_SRC_ALPHA_SATURATE: VkBlendFactor = VkBlendFactor_VK_BLEND_FACTOR_SRC_ALPHA_SATURATE; // island enum // gen:879bd323416c
pub const VK_BLEND_FACTOR_SRC_COLOR: VkBlendFactor = VkBlendFactor_VK_BLEND_FACTOR_SRC_COLOR; // island enum // gen:1ef722253f5d
pub const VK_BLEND_FACTOR_ZERO: VkBlendFactor = VkBlendFactor_VK_BLEND_FACTOR_ZERO; // island enum // gen:5d8339de8900
pub const VK_BLEND_OP_ADD: VkBlendOp = VkBlendOp_VK_BLEND_OP_ADD; // island enum // gen:8746c813518f
pub const VK_BLEND_OP_MAX: VkBlendOp = VkBlendOp_VK_BLEND_OP_MAX; // island enum // gen:a86b0bcbb6be
pub const VK_BLEND_OP_MIN: VkBlendOp = VkBlendOp_VK_BLEND_OP_MIN; // island enum // gen:e0fe5c91dc68
pub const VK_BLEND_OP_REVERSE_SUBTRACT: VkBlendOp = VkBlendOp_VK_BLEND_OP_REVERSE_SUBTRACT; // island enum // gen:ca87a983aa8e
pub const VK_BLEND_OP_SUBTRACT: VkBlendOp = VkBlendOp_VK_BLEND_OP_SUBTRACT; // island enum // gen:2161a92e43e6
pub const VK_BORDER_COLOR_FLOAT_OPAQUE_BLACK: VkBorderColor = VkBorderColor_VK_BORDER_COLOR_FLOAT_OPAQUE_BLACK; // island enum // gen:a6ed908c8a90
pub const VK_BORDER_COLOR_FLOAT_OPAQUE_WHITE: VkBorderColor = VkBorderColor_VK_BORDER_COLOR_FLOAT_OPAQUE_WHITE; // island enum // gen:22387d63eece
pub const VK_BORDER_COLOR_FLOAT_TRANSPARENT_BLACK: VkBorderColor = VkBorderColor_VK_BORDER_COLOR_FLOAT_TRANSPARENT_BLACK; // island enum // gen:88081e74f833
pub const VK_BUFFER_USAGE_INDEX_BUFFER_BIT: VkBufferUsageFlagBits = VkBufferUsageFlagBits_VK_BUFFER_USAGE_INDEX_BUFFER_BIT; // island enum // gen:595e8d2cb7ed
pub const VK_BUFFER_USAGE_INDIRECT_BUFFER_BIT: VkBufferUsageFlagBits = VkBufferUsageFlagBits_VK_BUFFER_USAGE_INDIRECT_BUFFER_BIT; // island enum // gen:e161ff76f809
pub const VK_BUFFER_USAGE_STORAGE_BUFFER_BIT: VkBufferUsageFlagBits = VkBufferUsageFlagBits_VK_BUFFER_USAGE_STORAGE_BUFFER_BIT; // island enum // gen:183d371e83ca
pub const VK_BUFFER_USAGE_STORAGE_TEXEL_BUFFER_BIT: VkBufferUsageFlagBits = VkBufferUsageFlagBits_VK_BUFFER_USAGE_STORAGE_TEXEL_BUFFER_BIT; // island enum // gen:417258b60ed5
pub const VK_BUFFER_USAGE_TRANSFER_DST_BIT: VkBufferUsageFlagBits = VkBufferUsageFlagBits_VK_BUFFER_USAGE_TRANSFER_DST_BIT; // island enum // gen:ca43b914b647
pub const VK_BUFFER_USAGE_TRANSFER_SRC_BIT: VkBufferUsageFlagBits = VkBufferUsageFlagBits_VK_BUFFER_USAGE_TRANSFER_SRC_BIT; // island enum // gen:6540d586070e
pub const VK_BUFFER_USAGE_TRANSFORM_FEEDBACK_BUFFER_BIT_EXT: VkBufferUsageFlagBits = VkBufferUsageFlagBits_VK_BUFFER_USAGE_TRANSFORM_FEEDBACK_BUFFER_BIT_EXT; // island enum // gen:8ff99dc65d43
pub const VK_BUFFER_USAGE_TRANSFORM_FEEDBACK_COUNTER_BUFFER_BIT_EXT: VkBufferUsageFlagBits = VkBufferUsageFlagBits_VK_BUFFER_USAGE_TRANSFORM_FEEDBACK_COUNTER_BUFFER_BIT_EXT; // island enum // gen:c95ab235a0b7
pub const VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT: VkBufferUsageFlagBits = VkBufferUsageFlagBits_VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT; // island enum // gen:ad130cf957c1
pub const VK_BUFFER_USAGE_UNIFORM_TEXEL_BUFFER_BIT: VkBufferUsageFlagBits = VkBufferUsageFlagBits_VK_BUFFER_USAGE_UNIFORM_TEXEL_BUFFER_BIT; // island enum // gen:b7fb53e55a10
pub const VK_BUFFER_USAGE_VERTEX_BUFFER_BIT: VkBufferUsageFlagBits = VkBufferUsageFlagBits_VK_BUFFER_USAGE_VERTEX_BUFFER_BIT; // island enum // gen:78a74c654845
pub const VK_COLOR_COMPONENT_A_BIT: VkColorComponentFlagBits = VkColorComponentFlagBits_VK_COLOR_COMPONENT_A_BIT; // island enum // gen:1c5e50225a99
pub const VK_COLOR_COMPONENT_B_BIT: VkColorComponentFlagBits = VkColorComponentFlagBits_VK_COLOR_COMPONENT_B_BIT; // island enum // gen:f1e395fe4daf
pub const VK_COLOR_COMPONENT_G_BIT: VkColorComponentFlagBits = VkColorComponentFlagBits_VK_COLOR_COMPONENT_G_BIT; // island enum // gen:d889900518d1
pub const VK_COLOR_COMPONENT_R_BIT: VkColorComponentFlagBits = VkColorComponentFlagBits_VK_COLOR_COMPONENT_R_BIT; // island enum // gen:290cbe7b0b69
pub const VK_COLOR_SPACE_SRGB_NONLINEAR_KHR: VkColorSpaceKHR = VkColorSpaceKHR_VK_COLOR_SPACE_SRGB_NONLINEAR_KHR; // island enum // gen:310b92705522
pub const VK_COMMAND_BUFFER_LEVEL_PRIMARY: VkCommandBufferLevel = VkCommandBufferLevel_VK_COMMAND_BUFFER_LEVEL_PRIMARY; // island enum // gen:ebc9288eaccf
pub const VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT: VkCommandBufferUsageFlagBits = VkCommandBufferUsageFlagBits_VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT; // island enum // gen:6924dfdd11c7
pub const VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT: VkCommandPoolCreateFlagBits = VkCommandPoolCreateFlagBits_VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT; // island enum // gen:924bd73a81dd
pub const VK_COMMAND_POOL_CREATE_TRANSIENT_BIT: VkCommandPoolCreateFlagBits = VkCommandPoolCreateFlagBits_VK_COMMAND_POOL_CREATE_TRANSIENT_BIT; // island enum // gen:91a5eb5a4509
pub const VK_COMPARE_OP_ALWAYS: VkCompareOp = VkCompareOp_VK_COMPARE_OP_ALWAYS; // island enum // gen:0b587d519c70
pub const VK_COMPARE_OP_EQUAL: VkCompareOp = VkCompareOp_VK_COMPARE_OP_EQUAL; // island enum // gen:76c03b52e677
pub const VK_COMPARE_OP_GREATER: VkCompareOp = VkCompareOp_VK_COMPARE_OP_GREATER; // island enum // gen:b5f7c96b64f5
pub const VK_COMPARE_OP_GREATER_OR_EQUAL: VkCompareOp = VkCompareOp_VK_COMPARE_OP_GREATER_OR_EQUAL; // island enum // gen:b52832ee5d28
pub const VK_COMPARE_OP_LESS: VkCompareOp = VkCompareOp_VK_COMPARE_OP_LESS; // island enum // gen:4598a29e9afd
pub const VK_COMPARE_OP_LESS_OR_EQUAL: VkCompareOp = VkCompareOp_VK_COMPARE_OP_LESS_OR_EQUAL; // island enum // gen:6867bc903a80
pub const VK_COMPARE_OP_NEVER: VkCompareOp = VkCompareOp_VK_COMPARE_OP_NEVER; // island enum // gen:8a6e10481547
pub const VK_COMPARE_OP_NOT_EQUAL: VkCompareOp = VkCompareOp_VK_COMPARE_OP_NOT_EQUAL; // island enum // gen:baf80f81bd8b
pub const VK_COMPONENT_SWIZZLE_A: VkComponentSwizzle = VkComponentSwizzle_VK_COMPONENT_SWIZZLE_A; // island enum // gen:d48ac52bd24d
pub const VK_COMPONENT_SWIZZLE_B: VkComponentSwizzle = VkComponentSwizzle_VK_COMPONENT_SWIZZLE_B; // island enum // gen:8a02975e27ce
pub const VK_COMPONENT_SWIZZLE_G: VkComponentSwizzle = VkComponentSwizzle_VK_COMPONENT_SWIZZLE_G; // island enum // gen:f97e3b52fe19
pub const VK_COMPONENT_SWIZZLE_IDENTITY: VkComponentSwizzle = VkComponentSwizzle_VK_COMPONENT_SWIZZLE_IDENTITY; // island enum // gen:bc9f274927ab
pub const VK_COMPONENT_SWIZZLE_ONE: VkComponentSwizzle = VkComponentSwizzle_VK_COMPONENT_SWIZZLE_ONE; // island enum // gen:d33ce27097a7
pub const VK_COMPONENT_SWIZZLE_R: VkComponentSwizzle = VkComponentSwizzle_VK_COMPONENT_SWIZZLE_R; // island enum // gen:b944432f85f1
pub const VK_COMPONENT_SWIZZLE_ZERO: VkComponentSwizzle = VkComponentSwizzle_VK_COMPONENT_SWIZZLE_ZERO; // island enum // gen:ed940974dc89
pub const VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR: VkCompositeAlphaFlagBitsKHR = VkCompositeAlphaFlagBitsKHR_VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR; // island enum // gen:4ffb5e80307f
pub const VK_CULL_MODE_BACK_BIT: VkCullModeFlagBits = VkCullModeFlagBits_VK_CULL_MODE_BACK_BIT; // island enum // gen:320d23d36071
pub const VK_CULL_MODE_FRONT_BIT: VkCullModeFlagBits = VkCullModeFlagBits_VK_CULL_MODE_FRONT_BIT; // island enum // gen:7cc0e79b7fea
pub const VK_CULL_MODE_NONE: VkCullModeFlagBits = VkCullModeFlagBits_VK_CULL_MODE_NONE; // island enum // gen:c017ddbf4c92
pub const VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE: VkDescriptorType = VkDescriptorType_VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE; // island enum // gen:9cc262c039ab
pub const VK_DESCRIPTOR_TYPE_SAMPLER: VkDescriptorType = VkDescriptorType_VK_DESCRIPTOR_TYPE_SAMPLER; // island enum // gen:0d112ddbc79a
pub const VK_DESCRIPTOR_TYPE_STORAGE_IMAGE: VkDescriptorType = VkDescriptorType_VK_DESCRIPTOR_TYPE_STORAGE_IMAGE; // island enum // gen:77d31b688e72
pub const VK_DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER: VkDescriptorType = VkDescriptorType_VK_DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER; // island enum // gen:09c8e9bd00cb
pub const VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER: VkDescriptorType = VkDescriptorType_VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER; // island enum // gen:8e7de413e4ec
pub const VK_DESCRIPTOR_TYPE_UNIFORM_TEXEL_BUFFER: VkDescriptorType = VkDescriptorType_VK_DESCRIPTOR_TYPE_UNIFORM_TEXEL_BUFFER; // island enum // gen:760fe0a38bb4
pub const VK_DYNAMIC_STATE_ALPHA_TO_COVERAGE_ENABLE_EXT: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_ALPHA_TO_COVERAGE_ENABLE_EXT; // island enum // gen:555b8851b027
pub const VK_DYNAMIC_STATE_BLEND_CONSTANTS: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_BLEND_CONSTANTS; // island enum // gen:f15c632c1c07
pub const VK_DYNAMIC_STATE_COLOR_BLEND_ENABLE_EXT: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_COLOR_BLEND_ENABLE_EXT; // island enum // gen:da40ec98030d
pub const VK_DYNAMIC_STATE_COLOR_BLEND_EQUATION_EXT: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_COLOR_BLEND_EQUATION_EXT; // island enum // gen:07cfd0618025
pub const VK_DYNAMIC_STATE_COLOR_WRITE_MASK_EXT: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_COLOR_WRITE_MASK_EXT; // island enum // gen:b43faf0d72c9
pub const VK_DYNAMIC_STATE_CULL_MODE_EXT: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_CULL_MODE_EXT; // island enum // gen:d820480a20da
pub const VK_DYNAMIC_STATE_DEPTH_BIAS: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_DEPTH_BIAS; // island enum // gen:d421ea63adb1
pub const VK_DYNAMIC_STATE_DEPTH_BIAS_ENABLE_EXT: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_DEPTH_BIAS_ENABLE_EXT; // island enum // gen:5b90667809e8
pub const VK_DYNAMIC_STATE_DEPTH_CLAMP_ENABLE_EXT: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_DEPTH_CLAMP_ENABLE_EXT; // island enum // gen:9aaaf018bcec
pub const VK_DYNAMIC_STATE_DEPTH_COMPARE_OP_EXT: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_DEPTH_COMPARE_OP_EXT; // island enum // gen:e27db5f4e18c
pub const VK_DYNAMIC_STATE_DEPTH_TEST_ENABLE_EXT: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_DEPTH_TEST_ENABLE_EXT; // island enum // gen:bdc8f35c138c
pub const VK_DYNAMIC_STATE_DEPTH_WRITE_ENABLE_EXT: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_DEPTH_WRITE_ENABLE_EXT; // island enum // gen:06786a989704
pub const VK_DYNAMIC_STATE_FRONT_FACE_EXT: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_FRONT_FACE_EXT; // island enum // gen:738628e50220
pub const VK_DYNAMIC_STATE_PATCH_CONTROL_POINTS_EXT: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_PATCH_CONTROL_POINTS_EXT; // island enum // gen:40eb8cdf3c30
pub const VK_DYNAMIC_STATE_PRIMITIVE_RESTART_ENABLE_EXT: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_PRIMITIVE_RESTART_ENABLE_EXT; // island enum // gen:7e5f3e434836
pub const VK_DYNAMIC_STATE_PRIMITIVE_TOPOLOGY_EXT: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_PRIMITIVE_TOPOLOGY_EXT; // island enum // gen:e667198e06e5
pub const VK_DYNAMIC_STATE_RASTERIZATION_SAMPLES_EXT: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_RASTERIZATION_SAMPLES_EXT; // island enum // gen:172781e1bab6
pub const VK_DYNAMIC_STATE_RASTERIZER_DISCARD_ENABLE_EXT: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_RASTERIZER_DISCARD_ENABLE_EXT; // island enum // gen:023e61e97c6e
pub const VK_DYNAMIC_STATE_SAMPLE_MASK_EXT: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_SAMPLE_MASK_EXT; // island enum // gen:54b22658cd09
pub const VK_DYNAMIC_STATE_SCISSOR: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_SCISSOR; // island enum // gen:d8fa87512623
pub const VK_DYNAMIC_STATE_STENCIL_COMPARE_MASK: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_STENCIL_COMPARE_MASK; // island enum // gen:2e4f78307d9a
pub const VK_DYNAMIC_STATE_STENCIL_OP_EXT: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_STENCIL_OP_EXT; // island enum // gen:7588a0eaaae6
pub const VK_DYNAMIC_STATE_STENCIL_REFERENCE: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_STENCIL_REFERENCE; // island enum // gen:fe901292e00e
pub const VK_DYNAMIC_STATE_STENCIL_TEST_ENABLE_EXT: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_STENCIL_TEST_ENABLE_EXT; // island enum // gen:7ac0621d2330
pub const VK_DYNAMIC_STATE_STENCIL_WRITE_MASK: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_STENCIL_WRITE_MASK; // island enum // gen:2bcd467bb253
pub const VK_DYNAMIC_STATE_VIEWPORT: VkDynamicState = VkDynamicState_VK_DYNAMIC_STATE_VIEWPORT; // island enum // gen:346f3e6c4109
pub const VK_ERROR_DEVICE_LOST: VkResult = VkResult_VK_ERROR_DEVICE_LOST; // island enum // gen:c106d6fddb5e
pub const VK_ERROR_EXTENSION_NOT_PRESENT: VkResult = VkResult_VK_ERROR_EXTENSION_NOT_PRESENT; // island enum // gen:b55324cc38f4
pub const VK_ERROR_FEATURE_NOT_PRESENT: VkResult = VkResult_VK_ERROR_FEATURE_NOT_PRESENT; // island enum // gen:655d55cddfa2
pub const VK_ERROR_FORMAT_NOT_SUPPORTED: VkResult = VkResult_VK_ERROR_FORMAT_NOT_SUPPORTED; // island enum // gen:c4549c66c456
pub const VK_ERROR_FRAGMENTATION_EXT: VkResult = VkResult_VK_ERROR_FRAGMENTATION_EXT; // island enum // gen:71698431d93f
pub const VK_ERROR_FRAGMENTED_POOL: VkResult = VkResult_VK_ERROR_FRAGMENTED_POOL; // island enum // gen:f48a7e55a290
pub const VK_ERROR_INCOMPATIBLE_DRIVER: VkResult = VkResult_VK_ERROR_INCOMPATIBLE_DRIVER; // island enum // gen:418a77a454d8
pub const VK_ERROR_INITIALIZATION_FAILED: VkResult = VkResult_VK_ERROR_INITIALIZATION_FAILED; // island enum // gen:d4dc319facbe
pub const VK_ERROR_INVALID_EXTERNAL_HANDLE: VkResult = VkResult_VK_ERROR_INVALID_EXTERNAL_HANDLE; // island enum // gen:9fa1681e1d4f
pub const VK_ERROR_INVALID_OPAQUE_CAPTURE_ADDRESS_KHR: VkResult = VkResult_VK_ERROR_INVALID_OPAQUE_CAPTURE_ADDRESS_KHR; // island enum // gen:993d6782fec5
pub const VK_ERROR_INVALID_SHADER_NV: VkResult = VkResult_VK_ERROR_INVALID_SHADER_NV; // island enum // gen:1989dc7a3d78
pub const VK_ERROR_LAYER_NOT_PRESENT: VkResult = VkResult_VK_ERROR_LAYER_NOT_PRESENT; // island enum // gen:baea27d30cc5
pub const VK_ERROR_MEMORY_MAP_FAILED: VkResult = VkResult_VK_ERROR_MEMORY_MAP_FAILED; // island enum // gen:9ff591375af6
pub const VK_ERROR_NATIVE_WINDOW_IN_USE_KHR: VkResult = VkResult_VK_ERROR_NATIVE_WINDOW_IN_USE_KHR; // island enum // gen:e06afad2b769
pub const VK_ERROR_NOT_PERMITTED_EXT: VkResult = VkResult_VK_ERROR_NOT_PERMITTED_EXT; // island enum // gen:05d4aaf87939
pub const VK_ERROR_OUT_OF_DATE_KHR: VkResult = VkResult_VK_ERROR_OUT_OF_DATE_KHR; // island enum // gen:707242e105dc
pub const VK_ERROR_OUT_OF_DEVICE_MEMORY: VkResult = VkResult_VK_ERROR_OUT_OF_DEVICE_MEMORY; // island enum // gen:56d60b1ee2ea
pub const VK_ERROR_OUT_OF_HOST_MEMORY: VkResult = VkResult_VK_ERROR_OUT_OF_HOST_MEMORY; // island enum // gen:e2bbf64bd668
pub const VK_ERROR_OUT_OF_POOL_MEMORY: VkResult = VkResult_VK_ERROR_OUT_OF_POOL_MEMORY; // island enum // gen:d424d8891932
pub const VK_ERROR_SURFACE_LOST_KHR: VkResult = VkResult_VK_ERROR_SURFACE_LOST_KHR; // island enum // gen:d447c009426f
pub const VK_ERROR_TOO_MANY_OBJECTS: VkResult = VkResult_VK_ERROR_TOO_MANY_OBJECTS; // island enum // gen:c195cae2a454
pub const VK_EVENT_RESET: VkResult = VkResult_VK_EVENT_RESET; // island enum // gen:fa59319107ae
pub const VK_EVENT_SET: VkResult = VkResult_VK_EVENT_SET; // island enum // gen:561dd5f99d9f
pub const VK_FILTER_LINEAR: VkFilter = VkFilter_VK_FILTER_LINEAR; // island enum // gen:6a30d6522caf
pub const VK_FILTER_NEAREST: VkFilter = VkFilter_VK_FILTER_NEAREST; // island enum // gen:e10bdf22f7b4
pub const VK_FORMAT_A1R5G5B5_UNORM_PACK16: VkFormat = VkFormat_VK_FORMAT_A1R5G5B5_UNORM_PACK16; // island enum // gen:f938fe90ef17
pub const VK_FORMAT_A2B10G10R10_UNORM_PACK32: VkFormat = VkFormat_VK_FORMAT_A2B10G10R10_UNORM_PACK32; // island enum // gen:c3c147e844c7
pub const VK_FORMAT_B10G11R11_UFLOAT_PACK32: VkFormat = VkFormat_VK_FORMAT_B10G11R11_UFLOAT_PACK32; // island enum // gen:54ab5b5cd33c
pub const VK_FORMAT_B8G8R8A8_SRGB: VkFormat = VkFormat_VK_FORMAT_B8G8R8A8_SRGB; // island enum // gen:f6eac6d5ca37
pub const VK_FORMAT_B8G8R8A8_UNORM: VkFormat = VkFormat_VK_FORMAT_B8G8R8A8_UNORM; // island enum // gen:4d2b179115cb
pub const VK_FORMAT_BC1_RGBA_SRGB_BLOCK: VkFormat = VkFormat_VK_FORMAT_BC1_RGBA_SRGB_BLOCK; // island enum // gen:a530093919ea
pub const VK_FORMAT_BC1_RGBA_UNORM_BLOCK: VkFormat = VkFormat_VK_FORMAT_BC1_RGBA_UNORM_BLOCK; // island enum // gen:9eb686b842b0
pub const VK_FORMAT_BC2_SRGB_BLOCK: VkFormat = VkFormat_VK_FORMAT_BC2_SRGB_BLOCK; // island enum // gen:01349911021f
pub const VK_FORMAT_BC2_UNORM_BLOCK: VkFormat = VkFormat_VK_FORMAT_BC2_UNORM_BLOCK; // island enum // gen:e59d184ffc71
pub const VK_FORMAT_BC3_SRGB_BLOCK: VkFormat = VkFormat_VK_FORMAT_BC3_SRGB_BLOCK; // island enum // gen:268f091a47e9
pub const VK_FORMAT_BC3_UNORM_BLOCK: VkFormat = VkFormat_VK_FORMAT_BC3_UNORM_BLOCK; // island enum // gen:696d82cbd56c
pub const VK_FORMAT_BC4_SNORM_BLOCK: VkFormat = VkFormat_VK_FORMAT_BC4_SNORM_BLOCK; // island enum // gen:9c9923820e16
pub const VK_FORMAT_BC4_UNORM_BLOCK: VkFormat = VkFormat_VK_FORMAT_BC4_UNORM_BLOCK; // island enum // gen:fd8c6f51a801
pub const VK_FORMAT_BC5_SNORM_BLOCK: VkFormat = VkFormat_VK_FORMAT_BC5_SNORM_BLOCK; // island enum // gen:634548daadff
pub const VK_FORMAT_BC5_UNORM_BLOCK: VkFormat = VkFormat_VK_FORMAT_BC5_UNORM_BLOCK; // island enum // gen:da48a94e6b46
pub const VK_FORMAT_BC6H_SFLOAT_BLOCK: VkFormat = VkFormat_VK_FORMAT_BC6H_SFLOAT_BLOCK; // island enum // gen:68c134a1e304
pub const VK_FORMAT_BC6H_UFLOAT_BLOCK: VkFormat = VkFormat_VK_FORMAT_BC6H_UFLOAT_BLOCK; // island enum // gen:417e24f74a6b
pub const VK_FORMAT_BC7_SRGB_BLOCK: VkFormat = VkFormat_VK_FORMAT_BC7_SRGB_BLOCK; // island enum // gen:5dece7dec52c
pub const VK_FORMAT_BC7_UNORM_BLOCK: VkFormat = VkFormat_VK_FORMAT_BC7_UNORM_BLOCK; // island enum // gen:29daba469aad
pub const VK_FORMAT_D16_UNORM: VkFormat = VkFormat_VK_FORMAT_D16_UNORM; // island enum // gen:5180190e5eb5
pub const VK_FORMAT_D24_UNORM_S8_UINT: VkFormat = VkFormat_VK_FORMAT_D24_UNORM_S8_UINT; // island enum // gen:d236af7b37e7
pub const VK_FORMAT_D32_SFLOAT: VkFormat = VkFormat_VK_FORMAT_D32_SFLOAT; // island enum // gen:f9d995dda61e
pub const VK_FORMAT_D32_SFLOAT_S8_UINT: VkFormat = VkFormat_VK_FORMAT_D32_SFLOAT_S8_UINT; // island enum // gen:0e7a356d25e2
pub const VK_FORMAT_E5B9G9R9_UFLOAT_PACK32: VkFormat = VkFormat_VK_FORMAT_E5B9G9R9_UFLOAT_PACK32; // island enum // gen:99e5c5714339
pub const VK_FORMAT_FEATURE_COLOR_ATTACHMENT_BIT: VkFormatFeatureFlagBits = VkFormatFeatureFlagBits_VK_FORMAT_FEATURE_COLOR_ATTACHMENT_BIT; // island enum // gen:9b0d8be40f3a
pub const VK_FORMAT_FEATURE_COLOR_ATTACHMENT_BLEND_BIT: VkFormatFeatureFlagBits = VkFormatFeatureFlagBits_VK_FORMAT_FEATURE_COLOR_ATTACHMENT_BLEND_BIT; // island enum // gen:09d3161c9bde
pub const VK_FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT: VkFormatFeatureFlagBits = VkFormatFeatureFlagBits_VK_FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT; // island enum // gen:be56f4d84094
pub const VK_FORMAT_FEATURE_SAMPLED_IMAGE_BIT: VkFormatFeatureFlagBits = VkFormatFeatureFlagBits_VK_FORMAT_FEATURE_SAMPLED_IMAGE_BIT; // island enum // gen:62ffb3f6d5b2
pub const VK_FORMAT_FEATURE_SAMPLED_IMAGE_FILTER_LINEAR_BIT: VkFormatFeatureFlagBits = VkFormatFeatureFlagBits_VK_FORMAT_FEATURE_SAMPLED_IMAGE_FILTER_LINEAR_BIT; // island enum // gen:db1528f7f198
pub const VK_FORMAT_FEATURE_STORAGE_IMAGE_BIT: VkFormatFeatureFlagBits = VkFormatFeatureFlagBits_VK_FORMAT_FEATURE_STORAGE_IMAGE_BIT; // island enum // gen:4ad5d3167b60
pub const VK_FORMAT_FEATURE_TRANSFER_DST_BIT: VkFormatFeatureFlagBits = VkFormatFeatureFlagBits_VK_FORMAT_FEATURE_TRANSFER_DST_BIT; // island enum // gen:98aef1a2787e
pub const VK_FORMAT_FEATURE_TRANSFER_SRC_BIT: VkFormatFeatureFlagBits = VkFormatFeatureFlagBits_VK_FORMAT_FEATURE_TRANSFER_SRC_BIT; // island enum // gen:4f674b92e735
pub const VK_FORMAT_FEATURE_UNIFORM_TEXEL_BUFFER_BIT: VkFormatFeatureFlagBits = VkFormatFeatureFlagBits_VK_FORMAT_FEATURE_UNIFORM_TEXEL_BUFFER_BIT; // island enum // gen:e6aa435c36b2
pub const VK_FORMAT_FEATURE_VERTEX_BUFFER_BIT: VkFormatFeatureFlagBits = VkFormatFeatureFlagBits_VK_FORMAT_FEATURE_VERTEX_BUFFER_BIT; // island enum // gen:194afd36b935
pub const VK_FORMAT_R16G16B16A16_SFLOAT: VkFormat = VkFormat_VK_FORMAT_R16G16B16A16_SFLOAT; // island enum // gen:41708fb7e0e7
pub const VK_FORMAT_R16G16B16A16_SINT: VkFormat = VkFormat_VK_FORMAT_R16G16B16A16_SINT; // island enum // gen:f1dc83ac85c9
pub const VK_FORMAT_R16G16B16A16_SNORM: VkFormat = VkFormat_VK_FORMAT_R16G16B16A16_SNORM; // island enum // gen:3bc7723efbda
pub const VK_FORMAT_R16G16B16A16_UINT: VkFormat = VkFormat_VK_FORMAT_R16G16B16A16_UINT; // island enum // gen:1068a0c752be
pub const VK_FORMAT_R16G16B16A16_UNORM: VkFormat = VkFormat_VK_FORMAT_R16G16B16A16_UNORM; // island enum // gen:a3598403addc
pub const VK_FORMAT_R16G16_SFLOAT: VkFormat = VkFormat_VK_FORMAT_R16G16_SFLOAT; // island enum // gen:5ab71d8d3668
pub const VK_FORMAT_R16G16_SINT: VkFormat = VkFormat_VK_FORMAT_R16G16_SINT; // island enum // gen:f051b8831932
pub const VK_FORMAT_R16G16_SNORM: VkFormat = VkFormat_VK_FORMAT_R16G16_SNORM; // island enum // gen:31ce863b3577
pub const VK_FORMAT_R16G16_UINT: VkFormat = VkFormat_VK_FORMAT_R16G16_UINT; // island enum // gen:a6d778ded9cd
pub const VK_FORMAT_R16G16_UNORM: VkFormat = VkFormat_VK_FORMAT_R16G16_UNORM; // island enum // gen:f1d52751e6e2
pub const VK_FORMAT_R16_SFLOAT: VkFormat = VkFormat_VK_FORMAT_R16_SFLOAT; // island enum // gen:d6e66cdcdb88
pub const VK_FORMAT_R16_SINT: VkFormat = VkFormat_VK_FORMAT_R16_SINT; // island enum // gen:61013cf69216
pub const VK_FORMAT_R16_SNORM: VkFormat = VkFormat_VK_FORMAT_R16_SNORM; // island enum // gen:e862814a66c4
pub const VK_FORMAT_R16_UINT: VkFormat = VkFormat_VK_FORMAT_R16_UINT; // island enum // gen:ecdb8812b833
pub const VK_FORMAT_R16_UNORM: VkFormat = VkFormat_VK_FORMAT_R16_UNORM; // island enum // gen:a3a1a6a1537c
pub const VK_FORMAT_R32G32B32A32_SFLOAT: VkFormat = VkFormat_VK_FORMAT_R32G32B32A32_SFLOAT; // island enum // gen:6fd58f865f05
pub const VK_FORMAT_R32G32B32A32_SINT: VkFormat = VkFormat_VK_FORMAT_R32G32B32A32_SINT; // island enum // gen:0c8ffc129bf8
pub const VK_FORMAT_R32G32B32A32_UINT: VkFormat = VkFormat_VK_FORMAT_R32G32B32A32_UINT; // island enum // gen:2ee01ba2046c
pub const VK_FORMAT_R32G32B32_SFLOAT: VkFormat = VkFormat_VK_FORMAT_R32G32B32_SFLOAT; // island enum // gen:ce8de33f5afa
pub const VK_FORMAT_R32G32B32_SINT: VkFormat = VkFormat_VK_FORMAT_R32G32B32_SINT; // island enum // gen:a72adc37a79d
pub const VK_FORMAT_R32G32B32_UINT: VkFormat = VkFormat_VK_FORMAT_R32G32B32_UINT; // island enum // gen:b572707360e3
pub const VK_FORMAT_R32G32_SFLOAT: VkFormat = VkFormat_VK_FORMAT_R32G32_SFLOAT; // island enum // gen:28db9da65f85
pub const VK_FORMAT_R32G32_SINT: VkFormat = VkFormat_VK_FORMAT_R32G32_SINT; // island enum // gen:79ea15608a25
pub const VK_FORMAT_R32G32_UINT: VkFormat = VkFormat_VK_FORMAT_R32G32_UINT; // island enum // gen:eaab6899839a
pub const VK_FORMAT_R32_SFLOAT: VkFormat = VkFormat_VK_FORMAT_R32_SFLOAT; // island enum // gen:307db9285bfd
pub const VK_FORMAT_R32_SINT: VkFormat = VkFormat_VK_FORMAT_R32_SINT; // island enum // gen:8d4b5c343ec5
pub const VK_FORMAT_R32_UINT: VkFormat = VkFormat_VK_FORMAT_R32_UINT; // island enum // gen:b1623bdb0b0b
pub const VK_FORMAT_R4G4B4A4_UNORM_PACK16: VkFormat = VkFormat_VK_FORMAT_R4G4B4A4_UNORM_PACK16; // island enum // gen:44802b14cf6d
pub const VK_FORMAT_R5G6B5_UNORM_PACK16: VkFormat = VkFormat_VK_FORMAT_R5G6B5_UNORM_PACK16; // island enum // gen:12b5d9bae942
pub const VK_FORMAT_R8G8B8A8_SINT: VkFormat = VkFormat_VK_FORMAT_R8G8B8A8_SINT; // island enum // gen:f1f49ef2633f
pub const VK_FORMAT_R8G8B8A8_SNORM: VkFormat = VkFormat_VK_FORMAT_R8G8B8A8_SNORM; // island enum // gen:0ce8f8ccfe31
pub const VK_FORMAT_R8G8B8A8_SRGB: VkFormat = VkFormat_VK_FORMAT_R8G8B8A8_SRGB; // island enum // gen:5007811997f9
pub const VK_FORMAT_R8G8B8A8_UINT: VkFormat = VkFormat_VK_FORMAT_R8G8B8A8_UINT; // island enum // gen:9adf99c2c439
pub const VK_FORMAT_R8G8B8A8_UNORM: VkFormat = VkFormat_VK_FORMAT_R8G8B8A8_UNORM; // island enum // gen:a5422d07014c
pub const VK_FORMAT_R8G8_SINT: VkFormat = VkFormat_VK_FORMAT_R8G8_SINT; // island enum // gen:05cd1f6762b7
pub const VK_FORMAT_R8G8_SNORM: VkFormat = VkFormat_VK_FORMAT_R8G8_SNORM; // island enum // gen:7d38c1260492
pub const VK_FORMAT_R8G8_UINT: VkFormat = VkFormat_VK_FORMAT_R8G8_UINT; // island enum // gen:bbfe92cc7c4c
pub const VK_FORMAT_R8G8_UNORM: VkFormat = VkFormat_VK_FORMAT_R8G8_UNORM; // island enum // gen:040deadbee48
pub const VK_FORMAT_R8_SINT: VkFormat = VkFormat_VK_FORMAT_R8_SINT; // island enum // gen:82abcdfd5717
pub const VK_FORMAT_R8_SNORM: VkFormat = VkFormat_VK_FORMAT_R8_SNORM; // island enum // gen:2fe989683067
pub const VK_FORMAT_R8_UINT: VkFormat = VkFormat_VK_FORMAT_R8_UINT; // island enum // gen:a61c3bccb2f9
pub const VK_FORMAT_R8_UNORM: VkFormat = VkFormat_VK_FORMAT_R8_UNORM; // island enum // gen:9d94e12d1b41
pub const VK_FORMAT_UNDEFINED: VkFormat = VkFormat_VK_FORMAT_UNDEFINED; // island enum // gen:edcad2394d18
pub const VK_FRONT_FACE_CLOCKWISE: VkFrontFace = VkFrontFace_VK_FRONT_FACE_CLOCKWISE; // island enum // gen:ffa9ca4c92f6
pub const VK_FRONT_FACE_COUNTER_CLOCKWISE: VkFrontFace = VkFrontFace_VK_FRONT_FACE_COUNTER_CLOCKWISE; // island enum // gen:f919ba706fd2
pub const VK_IMAGE_ASPECT_COLOR_BIT: VkImageAspectFlagBits = VkImageAspectFlagBits_VK_IMAGE_ASPECT_COLOR_BIT; // island enum // gen:61c2fdc4866a
pub const VK_IMAGE_ASPECT_DEPTH_BIT: VkImageAspectFlagBits = VkImageAspectFlagBits_VK_IMAGE_ASPECT_DEPTH_BIT; // island enum // gen:7545f7332338
pub const VK_IMAGE_ASPECT_STENCIL_BIT: VkImageAspectFlagBits = VkImageAspectFlagBits_VK_IMAGE_ASPECT_STENCIL_BIT; // island enum // gen:8cdf0fcc36be
pub const VK_IMAGE_CREATE_2D_ARRAY_COMPATIBLE_BIT: VkImageCreateFlagBits = VkImageCreateFlagBits_VK_IMAGE_CREATE_2D_ARRAY_COMPATIBLE_BIT; // island enum // gen:8f3871fc1c31
pub const VK_IMAGE_CREATE_CUBE_COMPATIBLE_BIT: VkImageCreateFlagBits = VkImageCreateFlagBits_VK_IMAGE_CREATE_CUBE_COMPATIBLE_BIT; // island enum // gen:15de65961fc2
pub const VK_IMAGE_CREATE_MUTABLE_FORMAT_BIT: VkImageCreateFlagBits = VkImageCreateFlagBits_VK_IMAGE_CREATE_MUTABLE_FORMAT_BIT; // island enum // gen:7a92a9c2d1e8
pub const VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL: VkImageLayout = VkImageLayout_VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL; // island enum // gen:8076b2e55621
pub const VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL: VkImageLayout = VkImageLayout_VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL; // island enum // gen:712ba4def7a4
pub const VK_IMAGE_LAYOUT_GENERAL: VkImageLayout = VkImageLayout_VK_IMAGE_LAYOUT_GENERAL; // island enum // gen:45c04336e146
pub const VK_IMAGE_LAYOUT_PRESENT_SRC_KHR: VkImageLayout = VkImageLayout_VK_IMAGE_LAYOUT_PRESENT_SRC_KHR; // island enum // gen:6c1cf2283ebf
pub const VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL: VkImageLayout = VkImageLayout_VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL; // island enum // gen:18032b351efe
pub const VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL: VkImageLayout = VkImageLayout_VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL; // island enum // gen:70031a6036a4
pub const VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL: VkImageLayout = VkImageLayout_VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL; // island enum // gen:c8fbfaea64f0
pub const VK_IMAGE_LAYOUT_UNDEFINED: VkImageLayout = VkImageLayout_VK_IMAGE_LAYOUT_UNDEFINED; // island enum // gen:f689cbb2ee35
pub const VK_IMAGE_TILING_OPTIMAL: VkImageTiling = VkImageTiling_VK_IMAGE_TILING_OPTIMAL; // island enum // gen:78b6646baf66
pub const VK_IMAGE_TYPE_1D: VkImageType = VkImageType_VK_IMAGE_TYPE_1D; // island enum // gen:390f81b297cb
pub const VK_IMAGE_TYPE_2D: VkImageType = VkImageType_VK_IMAGE_TYPE_2D; // island enum // gen:07a1451ec887
pub const VK_IMAGE_TYPE_3D: VkImageType = VkImageType_VK_IMAGE_TYPE_3D; // island enum // gen:b5ad1ec28f83
pub const VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT: VkImageUsageFlagBits = VkImageUsageFlagBits_VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT; // island enum // gen:4aa632a19249
pub const VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT: VkImageUsageFlagBits = VkImageUsageFlagBits_VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT; // island enum // gen:052e4d7dfecd
pub const VK_IMAGE_USAGE_SAMPLED_BIT: VkImageUsageFlagBits = VkImageUsageFlagBits_VK_IMAGE_USAGE_SAMPLED_BIT; // island enum // gen:93fe79a5df8f
pub const VK_IMAGE_USAGE_STORAGE_BIT: VkImageUsageFlagBits = VkImageUsageFlagBits_VK_IMAGE_USAGE_STORAGE_BIT; // island enum // gen:455cfe3c6fd6
pub const VK_IMAGE_USAGE_TRANSFER_DST_BIT: VkImageUsageFlagBits = VkImageUsageFlagBits_VK_IMAGE_USAGE_TRANSFER_DST_BIT; // island enum // gen:e1cff8d87990
pub const VK_IMAGE_USAGE_TRANSFER_SRC_BIT: VkImageUsageFlagBits = VkImageUsageFlagBits_VK_IMAGE_USAGE_TRANSFER_SRC_BIT; // island enum // gen:617fe38cbe4a
pub const VK_IMAGE_VIEW_TYPE_1D: VkImageViewType = VkImageViewType_VK_IMAGE_VIEW_TYPE_1D; // island enum // gen:366c53fde05e
pub const VK_IMAGE_VIEW_TYPE_1D_ARRAY: VkImageViewType = VkImageViewType_VK_IMAGE_VIEW_TYPE_1D_ARRAY; // island enum // gen:02438bddd6f0
pub const VK_IMAGE_VIEW_TYPE_2D: VkImageViewType = VkImageViewType_VK_IMAGE_VIEW_TYPE_2D; // island enum // gen:529cc673dd7b
pub const VK_IMAGE_VIEW_TYPE_2D_ARRAY: VkImageViewType = VkImageViewType_VK_IMAGE_VIEW_TYPE_2D_ARRAY; // island enum // gen:684e8cf48fd1
pub const VK_IMAGE_VIEW_TYPE_3D: VkImageViewType = VkImageViewType_VK_IMAGE_VIEW_TYPE_3D; // island enum // gen:1139f7bfbbde
pub const VK_IMAGE_VIEW_TYPE_CUBE: VkImageViewType = VkImageViewType_VK_IMAGE_VIEW_TYPE_CUBE; // island enum // gen:854bb447c3b8
pub const VK_IMAGE_VIEW_TYPE_CUBE_ARRAY: VkImageViewType = VkImageViewType_VK_IMAGE_VIEW_TYPE_CUBE_ARRAY; // island enum // gen:992594142b61
pub const VK_INCOMPLETE: VkResult = VkResult_VK_INCOMPLETE; // island enum // gen:971cb8e98400
pub const VK_INDEX_TYPE_UINT16: VkIndexType = VkIndexType_VK_INDEX_TYPE_UINT16; // island enum // gen:7fd5f5869d1d
pub const VK_INDEX_TYPE_UINT32: VkIndexType = VkIndexType_VK_INDEX_TYPE_UINT32; // island enum // gen:9a2bd1d1af64
pub const VK_LOGIC_OP_COPY: VkLogicOp = VkLogicOp_VK_LOGIC_OP_COPY; // island enum // gen:dbfb3a672f17
pub const VK_MEMORY_HEAP_DEVICE_LOCAL_BIT: VkMemoryHeapFlagBits = VkMemoryHeapFlagBits_VK_MEMORY_HEAP_DEVICE_LOCAL_BIT; // island enum // gen:bfd22166c22d
pub const VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT: VkMemoryPropertyFlagBits = VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT; // island enum // gen:7d62499daa08
pub const VK_MEMORY_PROPERTY_HOST_CACHED_BIT: VkMemoryPropertyFlagBits = VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_CACHED_BIT; // island enum // gen:99da6112a639
pub const VK_MEMORY_PROPERTY_HOST_COHERENT_BIT: VkMemoryPropertyFlagBits = VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_COHERENT_BIT; // island enum // gen:2e8688b16e8c
pub const VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT: VkMemoryPropertyFlagBits = VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT; // island enum // gen:1f1eb169b401
pub const VK_NOT_READY: VkResult = VkResult_VK_NOT_READY; // island enum // gen:57cd6ee8214e
pub const VK_PIPELINE_BIND_POINT_COMPUTE: VkPipelineBindPoint = VkPipelineBindPoint_VK_PIPELINE_BIND_POINT_COMPUTE; // island enum // gen:f7ef685dddc5
pub const VK_PIPELINE_BIND_POINT_GRAPHICS: VkPipelineBindPoint = VkPipelineBindPoint_VK_PIPELINE_BIND_POINT_GRAPHICS; // island enum // gen:02e720f743e4
pub const VK_PIPELINE_STAGE_ALL_COMMANDS_BIT: VkPipelineStageFlagBits = VkPipelineStageFlagBits_VK_PIPELINE_STAGE_ALL_COMMANDS_BIT; // island enum // gen:abf032ada26a
pub const VK_PIPELINE_STAGE_ALL_GRAPHICS_BIT: VkPipelineStageFlagBits = VkPipelineStageFlagBits_VK_PIPELINE_STAGE_ALL_GRAPHICS_BIT; // island enum // gen:35eca8a143a7
pub const VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT: VkPipelineStageFlagBits = VkPipelineStageFlagBits_VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT; // island enum // gen:9edc1ef64950
pub const VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT: VkPipelineStageFlagBits = VkPipelineStageFlagBits_VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT; // island enum // gen:cfea478100d7
pub const VK_PIPELINE_STAGE_COMPUTE_SHADER_BIT: VkPipelineStageFlagBits = VkPipelineStageFlagBits_VK_PIPELINE_STAGE_COMPUTE_SHADER_BIT; // island enum // gen:eb1580a32184
pub const VK_PIPELINE_STAGE_DRAW_INDIRECT_BIT: VkPipelineStageFlagBits = VkPipelineStageFlagBits_VK_PIPELINE_STAGE_DRAW_INDIRECT_BIT; // island enum // gen:6b0f189e52e6
pub const VK_PIPELINE_STAGE_EARLY_FRAGMENT_TESTS_BIT: VkPipelineStageFlagBits = VkPipelineStageFlagBits_VK_PIPELINE_STAGE_EARLY_FRAGMENT_TESTS_BIT; // island enum // gen:a182dbaf83ab
pub const VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT: VkPipelineStageFlagBits = VkPipelineStageFlagBits_VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT; // island enum // gen:5ec708d88d4f
pub const VK_PIPELINE_STAGE_GEOMETRY_SHADER_BIT: VkPipelineStageFlagBits = VkPipelineStageFlagBits_VK_PIPELINE_STAGE_GEOMETRY_SHADER_BIT; // island enum // gen:46d712b68f12
pub const VK_PIPELINE_STAGE_HOST_BIT: VkPipelineStageFlagBits = VkPipelineStageFlagBits_VK_PIPELINE_STAGE_HOST_BIT; // island enum // gen:2bf62690efca
pub const VK_PIPELINE_STAGE_LATE_FRAGMENT_TESTS_BIT: VkPipelineStageFlagBits = VkPipelineStageFlagBits_VK_PIPELINE_STAGE_LATE_FRAGMENT_TESTS_BIT; // island enum // gen:b914de30aa99
pub const VK_PIPELINE_STAGE_TESSELLATION_CONTROL_SHADER_BIT: VkPipelineStageFlagBits = VkPipelineStageFlagBits_VK_PIPELINE_STAGE_TESSELLATION_CONTROL_SHADER_BIT; // island enum // gen:fad916d9c2da
pub const VK_PIPELINE_STAGE_TESSELLATION_EVALUATION_SHADER_BIT: VkPipelineStageFlagBits = VkPipelineStageFlagBits_VK_PIPELINE_STAGE_TESSELLATION_EVALUATION_SHADER_BIT; // island enum // gen:ee69bac7c27b
pub const VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT: VkPipelineStageFlagBits = VkPipelineStageFlagBits_VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT; // island enum // gen:7cab5ffbd7e6
pub const VK_PIPELINE_STAGE_TRANSFER_BIT: VkPipelineStageFlagBits = VkPipelineStageFlagBits_VK_PIPELINE_STAGE_TRANSFER_BIT; // island enum // gen:9c3b6dc3edf8
pub const VK_PIPELINE_STAGE_TRANSFORM_FEEDBACK_BIT_EXT: VkPipelineStageFlagBits = VkPipelineStageFlagBits_VK_PIPELINE_STAGE_TRANSFORM_FEEDBACK_BIT_EXT; // island enum // gen:e5659a605d56
pub const VK_PIPELINE_STAGE_VERTEX_INPUT_BIT: VkPipelineStageFlagBits = VkPipelineStageFlagBits_VK_PIPELINE_STAGE_VERTEX_INPUT_BIT; // island enum // gen:7266ca7efffd
pub const VK_PIPELINE_STAGE_VERTEX_SHADER_BIT: VkPipelineStageFlagBits = VkPipelineStageFlagBits_VK_PIPELINE_STAGE_VERTEX_SHADER_BIT; // island enum // gen:8a827b7e0dec
pub const VK_PRESENT_MODE_FIFO_KHR: VkPresentModeKHR = VkPresentModeKHR_VK_PRESENT_MODE_FIFO_KHR; // island enum // gen:85386392452f
pub const VK_PRESENT_MODE_IMMEDIATE_KHR: VkPresentModeKHR = VkPresentModeKHR_VK_PRESENT_MODE_IMMEDIATE_KHR; // island enum // gen:25c064d31aa0
pub const VK_PRIMITIVE_TOPOLOGY_LINE_LIST: VkPrimitiveTopology = VkPrimitiveTopology_VK_PRIMITIVE_TOPOLOGY_LINE_LIST; // island enum // gen:344ce0af1ab8
pub const VK_PRIMITIVE_TOPOLOGY_LINE_LIST_WITH_ADJACENCY: VkPrimitiveTopology = VkPrimitiveTopology_VK_PRIMITIVE_TOPOLOGY_LINE_LIST_WITH_ADJACENCY; // island enum // gen:80f789388db9
pub const VK_PRIMITIVE_TOPOLOGY_LINE_STRIP: VkPrimitiveTopology = VkPrimitiveTopology_VK_PRIMITIVE_TOPOLOGY_LINE_STRIP; // island enum // gen:c961b509d020
pub const VK_PRIMITIVE_TOPOLOGY_LINE_STRIP_WITH_ADJACENCY: VkPrimitiveTopology = VkPrimitiveTopology_VK_PRIMITIVE_TOPOLOGY_LINE_STRIP_WITH_ADJACENCY; // island enum // gen:c55b76010221
pub const VK_PRIMITIVE_TOPOLOGY_PATCH_LIST: VkPrimitiveTopology = VkPrimitiveTopology_VK_PRIMITIVE_TOPOLOGY_PATCH_LIST; // island enum // gen:620efdb6e815
pub const VK_PRIMITIVE_TOPOLOGY_POINT_LIST: VkPrimitiveTopology = VkPrimitiveTopology_VK_PRIMITIVE_TOPOLOGY_POINT_LIST; // island enum // gen:422a2f906744
pub const VK_PRIMITIVE_TOPOLOGY_TRIANGLE_FAN: VkPrimitiveTopology = VkPrimitiveTopology_VK_PRIMITIVE_TOPOLOGY_TRIANGLE_FAN; // island enum // gen:b94f7bd8b466
pub const VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST: VkPrimitiveTopology = VkPrimitiveTopology_VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST; // island enum // gen:e074c1119aa8
pub const VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST_WITH_ADJACENCY: VkPrimitiveTopology = VkPrimitiveTopology_VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST_WITH_ADJACENCY; // island enum // gen:3f05d8fad4b7
pub const VK_PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP: VkPrimitiveTopology = VkPrimitiveTopology_VK_PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP; // island enum // gen:01d2fe008514
pub const VK_PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP_WITH_ADJACENCY: VkPrimitiveTopology = VkPrimitiveTopology_VK_PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP_WITH_ADJACENCY; // island enum // gen:7bfc4a6cc4a2
pub const VK_QUERY_CONTROL_PRECISE_BIT: VkQueryControlFlagBits = VkQueryControlFlagBits_VK_QUERY_CONTROL_PRECISE_BIT; // island enum // gen:ce7f270b8d84
pub const VK_QUERY_PIPELINE_STATISTIC_CLIPPING_INVOCATIONS_BIT: VkQueryPipelineStatisticFlagBits = VkQueryPipelineStatisticFlagBits_VK_QUERY_PIPELINE_STATISTIC_CLIPPING_INVOCATIONS_BIT; // island enum // gen:e9df0ad00122
pub const VK_QUERY_PIPELINE_STATISTIC_CLIPPING_PRIMITIVES_BIT: VkQueryPipelineStatisticFlagBits = VkQueryPipelineStatisticFlagBits_VK_QUERY_PIPELINE_STATISTIC_CLIPPING_PRIMITIVES_BIT; // island enum // gen:a683fcfc1936
pub const VK_QUERY_PIPELINE_STATISTIC_COMPUTE_SHADER_INVOCATIONS_BIT: VkQueryPipelineStatisticFlagBits = VkQueryPipelineStatisticFlagBits_VK_QUERY_PIPELINE_STATISTIC_COMPUTE_SHADER_INVOCATIONS_BIT; // island enum // gen:3a0f7b8fb894
pub const VK_QUERY_PIPELINE_STATISTIC_FRAGMENT_SHADER_INVOCATIONS_BIT: VkQueryPipelineStatisticFlagBits = VkQueryPipelineStatisticFlagBits_VK_QUERY_PIPELINE_STATISTIC_FRAGMENT_SHADER_INVOCATIONS_BIT; // island enum // gen:b20efdaa9669
pub const VK_QUERY_PIPELINE_STATISTIC_GEOMETRY_SHADER_INVOCATIONS_BIT: VkQueryPipelineStatisticFlagBits = VkQueryPipelineStatisticFlagBits_VK_QUERY_PIPELINE_STATISTIC_GEOMETRY_SHADER_INVOCATIONS_BIT; // island enum // gen:ad15c82f63c5
pub const VK_QUERY_PIPELINE_STATISTIC_GEOMETRY_SHADER_PRIMITIVES_BIT: VkQueryPipelineStatisticFlagBits = VkQueryPipelineStatisticFlagBits_VK_QUERY_PIPELINE_STATISTIC_GEOMETRY_SHADER_PRIMITIVES_BIT; // island enum // gen:4c462a88eb81
pub const VK_QUERY_PIPELINE_STATISTIC_INPUT_ASSEMBLY_PRIMITIVES_BIT: VkQueryPipelineStatisticFlagBits = VkQueryPipelineStatisticFlagBits_VK_QUERY_PIPELINE_STATISTIC_INPUT_ASSEMBLY_PRIMITIVES_BIT; // island enum // gen:491c88186a90
pub const VK_QUERY_PIPELINE_STATISTIC_INPUT_ASSEMBLY_VERTICES_BIT: VkQueryPipelineStatisticFlagBits = VkQueryPipelineStatisticFlagBits_VK_QUERY_PIPELINE_STATISTIC_INPUT_ASSEMBLY_VERTICES_BIT; // island enum // gen:c8d2c39165e5
pub const VK_QUERY_PIPELINE_STATISTIC_TESSELLATION_CONTROL_SHADER_PATCHES_BIT: VkQueryPipelineStatisticFlagBits = VkQueryPipelineStatisticFlagBits_VK_QUERY_PIPELINE_STATISTIC_TESSELLATION_CONTROL_SHADER_PATCHES_BIT; // island enum // gen:fb949f0fc4b6
pub const VK_QUERY_PIPELINE_STATISTIC_TESSELLATION_EVALUATION_SHADER_INVOCATIONS_BIT: VkQueryPipelineStatisticFlagBits = VkQueryPipelineStatisticFlagBits_VK_QUERY_PIPELINE_STATISTIC_TESSELLATION_EVALUATION_SHADER_INVOCATIONS_BIT; // island enum // gen:e9c184fddcba
pub const VK_QUERY_PIPELINE_STATISTIC_VERTEX_SHADER_INVOCATIONS_BIT: VkQueryPipelineStatisticFlagBits = VkQueryPipelineStatisticFlagBits_VK_QUERY_PIPELINE_STATISTIC_VERTEX_SHADER_INVOCATIONS_BIT; // island enum // gen:9369974eb28e
pub const VK_QUERY_RESULT_64_BIT: VkQueryResultFlagBits = VkQueryResultFlagBits_VK_QUERY_RESULT_64_BIT; // island enum // gen:98ec8a3e8d0a
pub const VK_QUERY_TYPE_OCCLUSION: VkQueryType = VkQueryType_VK_QUERY_TYPE_OCCLUSION; // island enum // gen:9a7acac8c0d8
pub const VK_QUERY_TYPE_PIPELINE_STATISTICS: VkQueryType = VkQueryType_VK_QUERY_TYPE_PIPELINE_STATISTICS; // island enum // gen:ae3da5dae150
pub const VK_QUERY_TYPE_TIMESTAMP: VkQueryType = VkQueryType_VK_QUERY_TYPE_TIMESTAMP; // island enum // gen:a2cf2b7ac28f
pub const VK_QUERY_TYPE_TRANSFORM_FEEDBACK_STREAM_EXT: VkQueryType = VkQueryType_VK_QUERY_TYPE_TRANSFORM_FEEDBACK_STREAM_EXT; // island enum // gen:40f98419f46e
pub const VK_QUEUE_GRAPHICS_BIT: VkQueueFlagBits = VkQueueFlagBits_VK_QUEUE_GRAPHICS_BIT; // island enum // gen:7cdde0e825e9
pub const VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_BORDER: VkSamplerAddressMode = VkSamplerAddressMode_VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_BORDER; // island enum // gen:2a9049a73ad8
pub const VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE: VkSamplerAddressMode = VkSamplerAddressMode_VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE; // island enum // gen:ce7ab5d3dd6a
pub const VK_SAMPLER_ADDRESS_MODE_MIRRORED_REPEAT: VkSamplerAddressMode = VkSamplerAddressMode_VK_SAMPLER_ADDRESS_MODE_MIRRORED_REPEAT; // island enum // gen:e70c7d840875
pub const VK_SAMPLER_ADDRESS_MODE_MIRROR_CLAMP_TO_EDGE: VkSamplerAddressMode = VkSamplerAddressMode_VK_SAMPLER_ADDRESS_MODE_MIRROR_CLAMP_TO_EDGE; // island enum // gen:2d227ad7d180
pub const VK_SAMPLER_ADDRESS_MODE_REPEAT: VkSamplerAddressMode = VkSamplerAddressMode_VK_SAMPLER_ADDRESS_MODE_REPEAT; // island enum // gen:3f15bcaaf864
pub const VK_SAMPLER_MIPMAP_MODE_LINEAR: VkSamplerMipmapMode = VkSamplerMipmapMode_VK_SAMPLER_MIPMAP_MODE_LINEAR; // island enum // gen:316c3a1f1bf9
pub const VK_SAMPLER_MIPMAP_MODE_NEAREST: VkSamplerMipmapMode = VkSamplerMipmapMode_VK_SAMPLER_MIPMAP_MODE_NEAREST; // island enum // gen:68f3f05fd332
pub const VK_SAMPLE_COUNT_1_BIT: VkSampleCountFlagBits = VkSampleCountFlagBits_VK_SAMPLE_COUNT_1_BIT; // island enum // gen:286103c44790
pub const VK_SHADER_STAGE_COMPUTE_BIT: VkShaderStageFlagBits = VkShaderStageFlagBits_VK_SHADER_STAGE_COMPUTE_BIT; // island enum // gen:541d4ebc0ae6
pub const VK_SHADER_STAGE_FRAGMENT_BIT: VkShaderStageFlagBits = VkShaderStageFlagBits_VK_SHADER_STAGE_FRAGMENT_BIT; // island enum // gen:e32b8957a820
pub const VK_SHADER_STAGE_GEOMETRY_BIT: VkShaderStageFlagBits = VkShaderStageFlagBits_VK_SHADER_STAGE_GEOMETRY_BIT; // island enum // gen:55fd4823e75b
pub const VK_SHADER_STAGE_TESSELLATION_CONTROL_BIT: VkShaderStageFlagBits = VkShaderStageFlagBits_VK_SHADER_STAGE_TESSELLATION_CONTROL_BIT; // island enum // gen:a68ee7cf6d66
pub const VK_SHADER_STAGE_TESSELLATION_EVALUATION_BIT: VkShaderStageFlagBits = VkShaderStageFlagBits_VK_SHADER_STAGE_TESSELLATION_EVALUATION_BIT; // island enum // gen:ff3163a10dfe
pub const VK_SHADER_STAGE_VERTEX_BIT: VkShaderStageFlagBits = VkShaderStageFlagBits_VK_SHADER_STAGE_VERTEX_BIT; // island enum // gen:3e8e13826bee
pub const VK_SHARING_MODE_EXCLUSIVE: VkSharingMode = VkSharingMode_VK_SHARING_MODE_EXCLUSIVE; // island enum // gen:bb5cff71316f
pub const VK_STENCIL_FACE_BACK_BIT: VkStencilFaceFlagBits = VkStencilFaceFlagBits_VK_STENCIL_FACE_BACK_BIT; // island enum // gen:4b9756d8eb85
pub const VK_STENCIL_FACE_FRONT_AND_BACK: VkStencilFaceFlagBits = VkStencilFaceFlagBits_VK_STENCIL_FACE_FRONT_AND_BACK; // island enum // gen:f2359cd4b9c1
pub const VK_STENCIL_FACE_FRONT_BIT: VkStencilFaceFlagBits = VkStencilFaceFlagBits_VK_STENCIL_FACE_FRONT_BIT; // island enum // gen:4a0f84356539
pub const VK_STENCIL_OP_DECREMENT_AND_CLAMP: VkStencilOp = VkStencilOp_VK_STENCIL_OP_DECREMENT_AND_CLAMP; // island enum // gen:b86e45c301f4
pub const VK_STENCIL_OP_DECREMENT_AND_WRAP: VkStencilOp = VkStencilOp_VK_STENCIL_OP_DECREMENT_AND_WRAP; // island enum // gen:d6909ecef810
pub const VK_STENCIL_OP_INCREMENT_AND_CLAMP: VkStencilOp = VkStencilOp_VK_STENCIL_OP_INCREMENT_AND_CLAMP; // island enum // gen:7672814ea3cb
pub const VK_STENCIL_OP_INCREMENT_AND_WRAP: VkStencilOp = VkStencilOp_VK_STENCIL_OP_INCREMENT_AND_WRAP; // island enum // gen:919791d20490
pub const VK_STENCIL_OP_INVERT: VkStencilOp = VkStencilOp_VK_STENCIL_OP_INVERT; // island enum // gen:f39050aafd38
pub const VK_STENCIL_OP_KEEP: VkStencilOp = VkStencilOp_VK_STENCIL_OP_KEEP; // island enum // gen:97235d38e5b5
pub const VK_STENCIL_OP_REPLACE: VkStencilOp = VkStencilOp_VK_STENCIL_OP_REPLACE; // island enum // gen:b82972fd4c69
pub const VK_STENCIL_OP_ZERO: VkStencilOp = VkStencilOp_VK_STENCIL_OP_ZERO; // island enum // gen:496f15588e84
pub const VK_STRUCTURE_TYPE_APPLICATION_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_APPLICATION_INFO; // island enum // gen:865a56d81aa8
pub const VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO; // island enum // gen:52decf536bbc
pub const VK_STRUCTURE_TYPE_BUFFER_MEMORY_BARRIER: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_BUFFER_MEMORY_BARRIER; // island enum // gen:1bb475d6809a
pub const VK_STRUCTURE_TYPE_BUFFER_VIEW_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_BUFFER_VIEW_CREATE_INFO; // island enum // gen:43804f66bd22
pub const VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO; // island enum // gen:83f2596b744f
pub const VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO; // island enum // gen:97b12fcd1696
pub const VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO; // island enum // gen:41c035e48736
pub const VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO; // island enum // gen:6a0549e621fd
pub const VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO; // island enum // gen:5aecba0d4c0e
pub const VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO; // island enum // gen:49b226f00c18
pub const VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO; // island enum // gen:c0c5cbe535cb
pub const VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO; // island enum // gen:a867cec144ea
pub const VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO; // island enum // gen:ccd1de06ea89
pub const VK_STRUCTURE_TYPE_EVENT_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_EVENT_CREATE_INFO; // island enum // gen:d59b2cc52353
pub const VK_STRUCTURE_TYPE_FENCE_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_FENCE_CREATE_INFO; // island enum // gen:f3c90f5460c3
pub const VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO; // island enum // gen:c8c569fa449c
pub const VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO; // island enum // gen:2b39c285c3cd
pub const VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO; // island enum // gen:570704721786
pub const VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER; // island enum // gen:cc2fad146920
pub const VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO; // island enum // gen:a113e428ba0c
pub const VK_STRUCTURE_TYPE_IMAGE_VIEW_USAGE_CREATE_INFO_KHR: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_IMAGE_VIEW_USAGE_CREATE_INFO_KHR; // island enum // gen:cdac3e96c5c5
pub const VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO; // island enum // gen:09a1cb61d4f9
pub const VK_STRUCTURE_TYPE_MAPPED_MEMORY_RANGE: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_MAPPED_MEMORY_RANGE; // island enum // gen:8817c011d64c
pub const VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO; // island enum // gen:df54cb926caf
pub const VK_STRUCTURE_TYPE_MEMORY_BARRIER: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_MEMORY_BARRIER; // island enum // gen:86ba4d25067b
pub const VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTENDED_DYNAMIC_STATE_2_FEATURES_EXT: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTENDED_DYNAMIC_STATE_2_FEATURES_EXT; // island enum // gen:c7b8e8678dc4
pub const VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTENDED_DYNAMIC_STATE_3_FEATURES_EXT: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTENDED_DYNAMIC_STATE_3_FEATURES_EXT; // island enum // gen:a24510b62098
pub const VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTENDED_DYNAMIC_STATE_FEATURES_EXT: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTENDED_DYNAMIC_STATE_FEATURES_EXT; // island enum // gen:eb63c086f75d
pub const VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_FEATURES_2: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_FEATURES_2; // island enum // gen:0ef9e283c1f2
pub const VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_HOST_QUERY_RESET_FEATURES: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_HOST_QUERY_RESET_FEATURES; // island enum // gen:0b5beaa72ea9
pub const VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_ID_PROPERTIES: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_ID_PROPERTIES; // island enum // gen:df69f2e6b7a0
pub const VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROPERTIES_2: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROPERTIES_2; // island enum // gen:c1d5325b36b3
pub const VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SHADER_DRAW_PARAMETERS_FEATURES: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SHADER_DRAW_PARAMETERS_FEATURES; // island enum // gen:3fdeb6af2cd2
pub const VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_TRANSFORM_FEEDBACK_FEATURES_EXT: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_TRANSFORM_FEEDBACK_FEATURES_EXT; // island enum // gen:3e7599188b1e
pub const VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_FEATURES_EXT: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_FEATURES_EXT; // island enum // gen:9fb2664584c4
pub const VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO; // island enum // gen:affd7f41ab72
pub const VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO; // island enum // gen:95b9f28789fe
pub const VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO; // island enum // gen:6ecf85d854e7
pub const VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO; // island enum // gen:ac31c339362a
pub const VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO; // island enum // gen:80c43766838a
pub const VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO; // island enum // gen:375d57e3611b
pub const VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO; // island enum // gen:683474e5c5b8
pub const VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO; // island enum // gen:5ca6246789b1
pub const VK_STRUCTURE_TYPE_PIPELINE_TESSELLATION_STATE_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_TESSELLATION_STATE_CREATE_INFO; // island enum // gen:39cd7b4cfe5a
pub const VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_DIVISOR_STATE_CREATE_INFO_EXT: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_DIVISOR_STATE_CREATE_INFO_EXT; // island enum // gen:634b26ab8da8
pub const VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO; // island enum // gen:5802320d25b8
pub const VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO; // island enum // gen:5df5a0a54ead
pub const VK_STRUCTURE_TYPE_PRESENT_INFO_KHR: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_PRESENT_INFO_KHR; // island enum // gen:8193bdeb7113
pub const VK_STRUCTURE_TYPE_QUERY_POOL_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_QUERY_POOL_CREATE_INFO; // island enum // gen:bc8635d91926
pub const VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO; // island enum // gen:642f34f6c06f
pub const VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO; // island enum // gen:43db1e71b43e
pub const VK_STRUCTURE_TYPE_SAMPLER_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_SAMPLER_CREATE_INFO; // island enum // gen:312ec841451e
pub const VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO; // island enum // gen:7b8466e332a8
pub const VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO; // island enum // gen:7a3fe947e6ef
pub const VK_STRUCTURE_TYPE_SUBMIT_INFO: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_SUBMIT_INFO; // island enum // gen:c26305860067
pub const VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR; // island enum // gen:f25f4046a879
pub const VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR; // island enum // gen:b2a1040ee3cf
pub const VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET: VkStructureType = VkStructureType_VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET; // island enum // gen:d63a04f9d83e
pub const VK_SUBOPTIMAL_KHR: VkResult = VkResult_VK_SUBOPTIMAL_KHR; // island enum // gen:3d3d9bd17d60
pub const VK_SUBPASS_CONTENTS_INLINE: VkSubpassContents = VkSubpassContents_VK_SUBPASS_CONTENTS_INLINE; // island enum // gen:d5780cda06b2
pub const VK_SUCCESS: VkResult = VkResult_VK_SUCCESS; // island enum // gen:e4500b3f2aac
pub const VK_TIMEOUT: VkResult = VkResult_VK_TIMEOUT; // island enum // gen:1b28a1843199
pub const VK_VERTEX_INPUT_RATE_INSTANCE: VkVertexInputRate = VkVertexInputRate_VK_VERTEX_INPUT_RATE_INSTANCE; // island enum // gen:bb7c6b8e324f
pub const VK_VERTEX_INPUT_RATE_VERTEX: VkVertexInputRate = VkVertexInputRate_VK_VERTEX_INPUT_RATE_VERTEX; // island enum // gen:3441cf985ae0
pub const VS_FOG_COORD: fog_src_type = fog_src_type_VS_FOG_COORD; // island enum // gen:2fd8b7ba330a
pub const VS_FOG_W: fog_src_type = fog_src_type_VS_FOG_W; // island enum // gen:d1af9966b0a7
pub const VS_FOG_Z: fog_src_type = fog_src_type_VS_FOG_Z; // island enum // gen:645ee20b9a1c
pub const WGL_ARB_PIXEL_FORMAT: wined3d_gl_extension = wined3d_gl_extension_WGL_ARB_PIXEL_FORMAT; // island enum // gen:fee51cdc7a18
pub const WGL_EXT_SWAP_CONTROL: wined3d_gl_extension = wined3d_gl_extension_WGL_EXT_SWAP_CONTROL; // island enum // gen:a1810ea7c23a
pub const WGL_WINE_PIXEL_FORMAT_PASSTHROUGH: wined3d_gl_extension = wined3d_gl_extension_WGL_WINE_PIXEL_FORMAT_PASSTHROUGH; // island enum // gen:9fbb419b9963
pub const WGL_WINE_QUERY_RENDERER: wined3d_gl_extension = wined3d_gl_extension_WGL_WINE_QUERY_RENDERER; // island enum // gen:4efd3293aad2
pub const WINED3DFMT_A8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_A8_UNORM; // island enum // gen:b852344850b7
pub const WINED3DFMT_AL16: wined3d_format_id = wined3d_format_id_WINED3DFMT_AL16; // island enum // gen:ae6e29fc1392
pub const WINED3DFMT_ATI1N: wined3d_format_id = wined3d_format_id_WINED3DFMT_ATI1N; // island enum // gen:fc494765db3f
pub const WINED3DFMT_ATI2N: wined3d_format_id = wined3d_format_id_WINED3DFMT_ATI2N; // island enum // gen:a09342789cf1
pub const WINED3DFMT_ATOC: wined3d_format_id = wined3d_format_id_WINED3DFMT_ATOC; // island enum // gen:b9f47f440552
pub const WINED3DFMT_B10G10R10A2_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_B10G10R10A2_UNORM; // island enum // gen:fd65f7c0562e
pub const WINED3DFMT_B2G3R3A8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_B2G3R3A8_UNORM; // island enum // gen:2a968c2ee686
pub const WINED3DFMT_B2G3R3_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_B2G3R3_UNORM; // island enum // gen:4fef2a637c7d
pub const WINED3DFMT_B4G4R4A4_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_B4G4R4A4_UNORM; // island enum // gen:c77c181e2f2d
pub const WINED3DFMT_B4G4R4X4_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_B4G4R4X4_UNORM; // island enum // gen:fb254e5113cc
pub const WINED3DFMT_B5G5R5A1_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_B5G5R5A1_UNORM; // island enum // gen:ce8c178ac58e
pub const WINED3DFMT_B5G5R5X1_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_B5G5R5X1_UNORM; // island enum // gen:f97d07713986
pub const WINED3DFMT_B5G6R5_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_B5G6R5_UNORM; // island enum // gen:4b51b8db5a0e
pub const WINED3DFMT_B8G8R8A8_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_B8G8R8A8_TYPELESS; // island enum // gen:50fba0ad0e62
pub const WINED3DFMT_B8G8R8A8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_B8G8R8A8_UNORM; // island enum // gen:2293a1e3fc0b
pub const WINED3DFMT_B8G8R8A8_UNORM_SRGB: wined3d_format_id = wined3d_format_id_WINED3DFMT_B8G8R8A8_UNORM_SRGB; // island enum // gen:82daa091d60a
pub const WINED3DFMT_B8G8R8X8_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_B8G8R8X8_TYPELESS; // island enum // gen:e950e1af308b
pub const WINED3DFMT_B8G8R8X8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_B8G8R8X8_UNORM; // island enum // gen:5c3514c0dc29
pub const WINED3DFMT_B8G8R8X8_UNORM_SRGB: wined3d_format_id = wined3d_format_id_WINED3DFMT_B8G8R8X8_UNORM_SRGB; // island enum // gen:f328a84114d6
pub const WINED3DFMT_B8G8R8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_B8G8R8_UNORM; // island enum // gen:26e4d489e6fb
pub const WINED3DFMT_BC1_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_BC1_TYPELESS; // island enum // gen:fa5ebd32a8f2
pub const WINED3DFMT_BC1_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_BC1_UNORM; // island enum // gen:d2e2c9ce4c3e
pub const WINED3DFMT_BC1_UNORM_SRGB: wined3d_format_id = wined3d_format_id_WINED3DFMT_BC1_UNORM_SRGB; // island enum // gen:421fd73b9afb
pub const WINED3DFMT_BC2_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_BC2_TYPELESS; // island enum // gen:f51ec49d2596
pub const WINED3DFMT_BC2_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_BC2_UNORM; // island enum // gen:2e50fef718b2
pub const WINED3DFMT_BC2_UNORM_SRGB: wined3d_format_id = wined3d_format_id_WINED3DFMT_BC2_UNORM_SRGB; // island enum // gen:0306925f5dc8
pub const WINED3DFMT_BC3_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_BC3_TYPELESS; // island enum // gen:57786d9ecf4a
pub const WINED3DFMT_BC3_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_BC3_UNORM; // island enum // gen:79168c2d84fb
pub const WINED3DFMT_BC3_UNORM_SRGB: wined3d_format_id = wined3d_format_id_WINED3DFMT_BC3_UNORM_SRGB; // island enum // gen:c29fce48a240
pub const WINED3DFMT_BC4_SNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_BC4_SNORM; // island enum // gen:c5cd6ab57784
pub const WINED3DFMT_BC4_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_BC4_TYPELESS; // island enum // gen:c2e555626d75
pub const WINED3DFMT_BC4_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_BC4_UNORM; // island enum // gen:3331e479e1cb
pub const WINED3DFMT_BC5_SNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_BC5_SNORM; // island enum // gen:2a6a95ba9810
pub const WINED3DFMT_BC5_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_BC5_TYPELESS; // island enum // gen:bc911c38a350
pub const WINED3DFMT_BC5_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_BC5_UNORM; // island enum // gen:b75fc85af908
pub const WINED3DFMT_BC6H_SF16: wined3d_format_id = wined3d_format_id_WINED3DFMT_BC6H_SF16; // island enum // gen:abd15ee7d696
pub const WINED3DFMT_BC6H_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_BC6H_TYPELESS; // island enum // gen:7fdf9ca42f18
pub const WINED3DFMT_BC6H_UF16: wined3d_format_id = wined3d_format_id_WINED3DFMT_BC6H_UF16; // island enum // gen:ba400c3ef38d
pub const WINED3DFMT_BC7_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_BC7_TYPELESS; // island enum // gen:c893075aedb2
pub const WINED3DFMT_BC7_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_BC7_UNORM; // island enum // gen:5fa432c94ac2
pub const WINED3DFMT_BC7_UNORM_SRGB: wined3d_format_id = wined3d_format_id_WINED3DFMT_BC7_UNORM_SRGB; // island enum // gen:58e64246ddc3
pub const WINED3DFMT_D16_LOCKABLE: wined3d_format_id = wined3d_format_id_WINED3DFMT_D16_LOCKABLE; // island enum // gen:cdf7ada8d3f0
pub const WINED3DFMT_D16_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_D16_UNORM; // island enum // gen:9c5a906a0784
pub const WINED3DFMT_D24_UNORM_S8_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_D24_UNORM_S8_UINT; // island enum // gen:9527094adfe6
pub const WINED3DFMT_D32_FLOAT: wined3d_format_id = wined3d_format_id_WINED3DFMT_D32_FLOAT; // island enum // gen:ed547dfdd88b
pub const WINED3DFMT_D32_FLOAT_S8X24_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_D32_FLOAT_S8X24_UINT; // island enum // gen:8e1b483ba3ab
pub const WINED3DFMT_D32_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_D32_UNORM; // island enum // gen:fac0db7c6301
pub const WINED3DFMT_DXT1: wined3d_format_id = wined3d_format_id_WINED3DFMT_DXT1; // island enum // gen:553a6c265db3
pub const WINED3DFMT_DXT2: wined3d_format_id = wined3d_format_id_WINED3DFMT_DXT2; // island enum // gen:82bd4645ec25
pub const WINED3DFMT_DXT3: wined3d_format_id = wined3d_format_id_WINED3DFMT_DXT3; // island enum // gen:187a70eb9414
pub const WINED3DFMT_DXT4: wined3d_format_id = wined3d_format_id_WINED3DFMT_DXT4; // island enum // gen:4f2fb5f8a31e
pub const WINED3DFMT_DXT5: wined3d_format_id = wined3d_format_id_WINED3DFMT_DXT5; // island enum // gen:bf70eb5d2476
pub const WINED3DFMT_G8R8_G8B8: wined3d_format_id = wined3d_format_id_WINED3DFMT_G8R8_G8B8; // island enum // gen:cd1b2c0c8c7e
pub const WINED3DFMT_G8R8_G8B8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_G8R8_G8B8_UNORM; // island enum // gen:e7643f36a3b2
pub const WINED3DFMT_INST: wined3d_format_id = wined3d_format_id_WINED3DFMT_INST; // island enum // gen:7bf4c0c75106
pub const WINED3DFMT_INTZ: wined3d_format_id = wined3d_format_id_WINED3DFMT_INTZ; // island enum // gen:315ea9570ef9
pub const WINED3DFMT_L16_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_L16_UNORM; // island enum // gen:b7207b17ac30
pub const WINED3DFMT_L4A4_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_L4A4_UNORM; // island enum // gen:7c9feb6fdd9c
pub const WINED3DFMT_L8A8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_L8A8_UNORM; // island enum // gen:621b76d8945e
pub const WINED3DFMT_L8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_L8_UNORM; // island enum // gen:385347c438e3
pub const WINED3DFMT_MULTI2_ARGB8: wined3d_format_id = wined3d_format_id_WINED3DFMT_MULTI2_ARGB8; // island enum // gen:d42d21bf71ce
pub const WINED3DFMT_NULL: wined3d_format_id = wined3d_format_id_WINED3DFMT_NULL; // island enum // gen:7a1f27be49b0
pub const WINED3DFMT_NV12: wined3d_format_id = wined3d_format_id_WINED3DFMT_NV12; // island enum // gen:740135c26db6
pub const WINED3DFMT_NVDB: wined3d_format_id = wined3d_format_id_WINED3DFMT_NVDB; // island enum // gen:c0957f1cecf2
pub const WINED3DFMT_NVHS: wined3d_format_id = wined3d_format_id_WINED3DFMT_NVHS; // island enum // gen:07bc4e14b527
pub const WINED3DFMT_NVHU: wined3d_format_id = wined3d_format_id_WINED3DFMT_NVHU; // island enum // gen:32b7173dc692
pub const WINED3DFMT_P8_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_P8_UINT; // island enum // gen:08be2f398d7b
pub const WINED3DFMT_P8_UINT_A8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_P8_UINT_A8_UNORM; // island enum // gen:ee16b1321b40
pub const WINED3DFMT_R10G10B10A2_SNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R10G10B10A2_SNORM; // island enum // gen:6f126d1cc9e5
pub const WINED3DFMT_R10G10B10A2_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_R10G10B10A2_TYPELESS; // island enum // gen:8c03a133429c
pub const WINED3DFMT_R10G10B10A2_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R10G10B10A2_UINT; // island enum // gen:efd52b485d52
pub const WINED3DFMT_R10G10B10A2_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R10G10B10A2_UNORM; // island enum // gen:1c4034896072
pub const WINED3DFMT_R10G10B10X2_SNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R10G10B10X2_SNORM; // island enum // gen:fa5916576b7d
pub const WINED3DFMT_R10G10B10X2_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_R10G10B10X2_TYPELESS; // island enum // gen:43736df119d7
pub const WINED3DFMT_R10G10B10X2_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R10G10B10X2_UINT; // island enum // gen:e9b2e89b1d31
pub const WINED3DFMT_R10G10B10_SNORM_A2_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R10G10B10_SNORM_A2_UNORM; // island enum // gen:408936d9d5be
pub const WINED3DFMT_R10G10B10_XR_BIAS_A2_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R10G10B10_XR_BIAS_A2_UNORM; // island enum // gen:b2d1b99ccbe5
pub const WINED3DFMT_R10G11B11_SNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R10G11B11_SNORM; // island enum // gen:2d160d053c41
pub const WINED3DFMT_R11G11B10_FLOAT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R11G11B10_FLOAT; // island enum // gen:c9665e182cb0
pub const WINED3DFMT_R16: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16; // island enum // gen:7fcd94172148
pub const WINED3DFMT_R16G16B16A16_FLOAT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16B16A16_FLOAT; // island enum // gen:7df2955d6781
pub const WINED3DFMT_R16G16B16A16_SINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16B16A16_SINT; // island enum // gen:2178584fcdfb
pub const WINED3DFMT_R16G16B16A16_SNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16B16A16_SNORM; // island enum // gen:7ad3ef834bb6
pub const WINED3DFMT_R16G16B16A16_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16B16A16_TYPELESS; // island enum // gen:b3edb1a815e7
pub const WINED3DFMT_R16G16B16A16_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16B16A16_UINT; // island enum // gen:91d9a5cfc198
pub const WINED3DFMT_R16G16B16A16_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16B16A16_UNORM; // island enum // gen:a32ff2d7dffc
pub const WINED3DFMT_R16G16_FLOAT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16_FLOAT; // island enum // gen:df79272fcb2e
pub const WINED3DFMT_R16G16_SINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16_SINT; // island enum // gen:5aee36a3ea1d
pub const WINED3DFMT_R16G16_SNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16_SNORM; // island enum // gen:c6640d1032ad
pub const WINED3DFMT_R16G16_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16_TYPELESS; // island enum // gen:85483f64827c
pub const WINED3DFMT_R16G16_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16_UINT; // island enum // gen:020c1e6ebb5e
pub const WINED3DFMT_R16G16_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16_UNORM; // island enum // gen:db785a6d61b2
pub const WINED3DFMT_R16_FLOAT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16_FLOAT; // island enum // gen:5700c3bf32d4
pub const WINED3DFMT_R16_SINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16_SINT; // island enum // gen:3892cb46b41a
pub const WINED3DFMT_R16_SNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16_SNORM; // island enum // gen:dafc0aaaa21c
pub const WINED3DFMT_R16_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16_TYPELESS; // island enum // gen:5e7b9d5401a8
pub const WINED3DFMT_R16_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16_UINT; // island enum // gen:4ce044980323
pub const WINED3DFMT_R16_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16_UNORM; // island enum // gen:7afcfa714e2d
pub const WINED3DFMT_R1_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R1_UNORM; // island enum // gen:57bdf1342230
pub const WINED3DFMT_R24G8_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_R24G8_TYPELESS; // island enum // gen:d740cfab7681
pub const WINED3DFMT_R24_UNORM_X8_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_R24_UNORM_X8_TYPELESS; // island enum // gen:11638f45d69f
pub const WINED3DFMT_R32G32B32A32_FLOAT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R32G32B32A32_FLOAT; // island enum // gen:df2bebda145f
pub const WINED3DFMT_R32G32B32A32_SINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R32G32B32A32_SINT; // island enum // gen:1a74f74561d0
pub const WINED3DFMT_R32G32B32A32_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_R32G32B32A32_TYPELESS; // island enum // gen:56e95b52269a
pub const WINED3DFMT_R32G32B32A32_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R32G32B32A32_UINT; // island enum // gen:d99512f7dca7
pub const WINED3DFMT_R32G32B32_FLOAT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R32G32B32_FLOAT; // island enum // gen:67dc0b8b4a65
pub const WINED3DFMT_R32G32B32_SINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R32G32B32_SINT; // island enum // gen:6f13aeb9522b
pub const WINED3DFMT_R32G32B32_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_R32G32B32_TYPELESS; // island enum // gen:1ec01d12e091
pub const WINED3DFMT_R32G32B32_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R32G32B32_UINT; // island enum // gen:4bb29d77a3b1
pub const WINED3DFMT_R32G32_FLOAT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R32G32_FLOAT; // island enum // gen:1183f2e5a833
pub const WINED3DFMT_R32G32_SINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R32G32_SINT; // island enum // gen:4c4c2c44c2c8
pub const WINED3DFMT_R32G32_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_R32G32_TYPELESS; // island enum // gen:a342b4c0566a
pub const WINED3DFMT_R32G32_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R32G32_UINT; // island enum // gen:7381389e0df7
pub const WINED3DFMT_R32G8X24_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_R32G8X24_TYPELESS; // island enum // gen:81c12cb2e495
pub const WINED3DFMT_R32_FLOAT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R32_FLOAT; // island enum // gen:59bc4b0e8aa7
pub const WINED3DFMT_R32_FLOAT_X8X24_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_R32_FLOAT_X8X24_TYPELESS; // island enum // gen:745de1cd3d35
pub const WINED3DFMT_R32_SINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R32_SINT; // island enum // gen:cef92021bf48
pub const WINED3DFMT_R32_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_R32_TYPELESS; // island enum // gen:2d57de105052
pub const WINED3DFMT_R32_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R32_UINT; // island enum // gen:3e1ccdac6f8d
pub const WINED3DFMT_R5G5_SNORM_L6_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R5G5_SNORM_L6_UNORM; // island enum // gen:86cc0bb8f6b0
pub const WINED3DFMT_R8G8B8A8_SINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8B8A8_SINT; // island enum // gen:8640dc41181e
pub const WINED3DFMT_R8G8B8A8_SNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8B8A8_SNORM; // island enum // gen:69412f5ca2db
pub const WINED3DFMT_R8G8B8A8_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8B8A8_TYPELESS; // island enum // gen:a86296beb8aa
pub const WINED3DFMT_R8G8B8A8_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8B8A8_UINT; // island enum // gen:1d31fd9bb937
pub const WINED3DFMT_R8G8B8A8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8B8A8_UNORM; // island enum // gen:721264dd8e5b
pub const WINED3DFMT_R8G8B8A8_UNORM_SRGB: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8B8A8_UNORM_SRGB; // island enum // gen:7960a7ebd97c
pub const WINED3DFMT_R8G8B8X8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8B8X8_UNORM; // island enum // gen:b63b4e348a0e
pub const WINED3DFMT_R8G8_B8G8: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8_B8G8; // island enum // gen:0ef20fcc1e92
pub const WINED3DFMT_R8G8_B8G8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8_B8G8_UNORM; // island enum // gen:d1abe0350b20
pub const WINED3DFMT_R8G8_SINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8_SINT; // island enum // gen:40ebb829e95f
pub const WINED3DFMT_R8G8_SNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8_SNORM; // island enum // gen:2315347f9119
pub const WINED3DFMT_R8G8_SNORM_Cx: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8_SNORM_Cx; // island enum // gen:cb32dcaeb972
pub const WINED3DFMT_R8G8_SNORM_L8X8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8_SNORM_L8X8_UNORM; // island enum // gen:d3e02c96ba02
pub const WINED3DFMT_R8G8_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8_TYPELESS; // island enum // gen:023f9924b236
pub const WINED3DFMT_R8G8_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8_UINT; // island enum // gen:fb88fbba7807
pub const WINED3DFMT_R8G8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8_UNORM; // island enum // gen:d410b4752017
pub const WINED3DFMT_R8_SINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8_SINT; // island enum // gen:6ba889e75944
pub const WINED3DFMT_R8_SNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8_SNORM; // island enum // gen:db6aa70333d8
pub const WINED3DFMT_R8_TYPELESS: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8_TYPELESS; // island enum // gen:9c8ad559d646
pub const WINED3DFMT_R8_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8_UINT; // island enum // gen:e169edac93c6
pub const WINED3DFMT_R8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8_UNORM; // island enum // gen:a2c35308bdcb
pub const WINED3DFMT_R9G9B9E5_SHAREDEXP: wined3d_format_id = wined3d_format_id_WINED3DFMT_R9G9B9E5_SHAREDEXP; // island enum // gen:fef765c38eb7
pub const WINED3DFMT_RESZ: wined3d_format_id = wined3d_format_id_WINED3DFMT_RESZ; // island enum // gen:0c07797deb91
pub const WINED3DFMT_S1_UINT_D15_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_S1_UINT_D15_UNORM; // island enum // gen:a396b379f935
pub const WINED3DFMT_S4X4_UINT_D24_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_S4X4_UINT_D24_UNORM; // island enum // gen:6fe6c428ea21
pub const WINED3DFMT_S8_UINT_D24_FLOAT: wined3d_format_id = wined3d_format_id_WINED3DFMT_S8_UINT_D24_FLOAT; // island enum // gen:ff71d2323417
pub const WINED3DFMT_UNKNOWN: wined3d_format_id = wined3d_format_id_WINED3DFMT_UNKNOWN; // island enum // gen:93d9074cc710
pub const WINED3DFMT_UYVY: wined3d_format_id = wined3d_format_id_WINED3DFMT_UYVY; // island enum // gen:d3667707ec52
pub const WINED3DFMT_X24_TYPELESS_G8_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_X24_TYPELESS_G8_UINT; // island enum // gen:9f30458b8508
pub const WINED3DFMT_X32_TYPELESS_G8X24_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_X32_TYPELESS_G8X24_UINT; // island enum // gen:33625ef16576
pub const WINED3DFMT_X8D24_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_X8D24_UNORM; // island enum // gen:b81fd5c118b7
pub const WINED3DFMT_YUY2: wined3d_format_id = wined3d_format_id_WINED3DFMT_YUY2; // island enum // gen:a2c4bf4f1ef3
pub const WINED3DFMT_YV12: wined3d_format_id = wined3d_format_id_WINED3DFMT_YV12; // island enum // gen:99688803f1b7
pub const WINED3DSGF_FORCE_EARLY_DEPTH_STENCIL: wined3d_shader_global_flags = wined3d_shader_global_flags_WINED3DSGF_FORCE_EARLY_DEPTH_STENCIL; // island enum // gen:a153e9463f0c
pub const WINED3DSIH_ABS: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ABS; // island enum // gen:90572a1e2122
pub const WINED3DSIH_ADD: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ADD; // island enum // gen:a04172d26d78
pub const WINED3DSIH_AND: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_AND; // island enum // gen:8f111f4b6adb
pub const WINED3DSIH_ATOMIC_AND: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ATOMIC_AND; // island enum // gen:16f5db968def
pub const WINED3DSIH_ATOMIC_CMP_STORE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ATOMIC_CMP_STORE; // island enum // gen:b3e9436151a5
pub const WINED3DSIH_ATOMIC_IADD: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ATOMIC_IADD; // island enum // gen:3dc4358e611c
pub const WINED3DSIH_ATOMIC_IMAX: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ATOMIC_IMAX; // island enum // gen:daa628690315
pub const WINED3DSIH_ATOMIC_IMIN: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ATOMIC_IMIN; // island enum // gen:c140ae089f65
pub const WINED3DSIH_ATOMIC_OR: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ATOMIC_OR; // island enum // gen:202b4c6438d0
pub const WINED3DSIH_ATOMIC_UMAX: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ATOMIC_UMAX; // island enum // gen:46c053cfecb8
pub const WINED3DSIH_ATOMIC_UMIN: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ATOMIC_UMIN; // island enum // gen:bad02e9f7b75
pub const WINED3DSIH_ATOMIC_XOR: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ATOMIC_XOR; // island enum // gen:b72fdcc3ba04
pub const WINED3DSIH_BEM: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_BEM; // island enum // gen:63f184f77a5f
pub const WINED3DSIH_BFI: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_BFI; // island enum // gen:898964c2981f
pub const WINED3DSIH_BFREV: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_BFREV; // island enum // gen:a01ffe82606c
pub const WINED3DSIH_BREAK: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_BREAK; // island enum // gen:b0b24e8bc832
pub const WINED3DSIH_BREAKC: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_BREAKC; // island enum // gen:09b2a4243999
pub const WINED3DSIH_BREAKP: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_BREAKP; // island enum // gen:897e7a49bab2
pub const WINED3DSIH_BUFINFO: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_BUFINFO; // island enum // gen:a8a0e97985e9
pub const WINED3DSIH_CALL: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_CALL; // island enum // gen:5dd591cb8fd3
pub const WINED3DSIH_CALLNZ: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_CALLNZ; // island enum // gen:574cae5b037c
pub const WINED3DSIH_CASE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_CASE; // island enum // gen:dc1a25b8cf85
pub const WINED3DSIH_CMP: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_CMP; // island enum // gen:9df89ccde8cd
pub const WINED3DSIH_CND: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_CND; // island enum // gen:f41b3c1d0b1e
pub const WINED3DSIH_CONTINUE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_CONTINUE; // island enum // gen:de8b50c6f507
pub const WINED3DSIH_CONTINUEP: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_CONTINUEP; // island enum // gen:e36bed8bf7bd
pub const WINED3DSIH_COUNTBITS: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_COUNTBITS; // island enum // gen:467452bf8374
pub const WINED3DSIH_CRS: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_CRS; // island enum // gen:8160913c0d10
pub const WINED3DSIH_CUT: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_CUT; // island enum // gen:a9c3290c6ab9
pub const WINED3DSIH_CUT_STREAM: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_CUT_STREAM; // island enum // gen:332deb8b006f
pub const WINED3DSIH_DCL: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL; // island enum // gen:6b986fa0787a
pub const WINED3DSIH_DCL_CONSTANT_BUFFER: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_CONSTANT_BUFFER; // island enum // gen:04238ac54686
pub const WINED3DSIH_DCL_FUNCTION_BODY: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_FUNCTION_BODY; // island enum // gen:f1719d4c9a03
pub const WINED3DSIH_DCL_FUNCTION_TABLE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_FUNCTION_TABLE; // island enum // gen:9da21e0e9127
pub const WINED3DSIH_DCL_GLOBAL_FLAGS: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_GLOBAL_FLAGS; // island enum // gen:ecec27fe0434
pub const WINED3DSIH_DCL_GS_INSTANCES: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_GS_INSTANCES; // island enum // gen:a8dabc8fa5e6
pub const WINED3DSIH_DCL_HS_FORK_PHASE_INSTANCE_COUNT: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_HS_FORK_PHASE_INSTANCE_COUNT; // island enum // gen:5f2add0ad473
pub const WINED3DSIH_DCL_HS_JOIN_PHASE_INSTANCE_COUNT: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_HS_JOIN_PHASE_INSTANCE_COUNT; // island enum // gen:bec4d43ca6a2
pub const WINED3DSIH_DCL_HS_MAX_TESSFACTOR: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_HS_MAX_TESSFACTOR; // island enum // gen:318ffebf43e9
pub const WINED3DSIH_DCL_IMMEDIATE_CONSTANT_BUFFER: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_IMMEDIATE_CONSTANT_BUFFER; // island enum // gen:8fe8a8e8f39a
pub const WINED3DSIH_DCL_INDEXABLE_TEMP: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_INDEXABLE_TEMP; // island enum // gen:d5d8557040ae
pub const WINED3DSIH_DCL_INDEX_RANGE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_INDEX_RANGE; // island enum // gen:4adce89f9266
pub const WINED3DSIH_DCL_INPUT: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_INPUT; // island enum // gen:1bc32dcf221d
pub const WINED3DSIH_DCL_INPUT_CONTROL_POINT_COUNT: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_INPUT_CONTROL_POINT_COUNT; // island enum // gen:e11bca9085c7
pub const WINED3DSIH_DCL_INPUT_PRIMITIVE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_INPUT_PRIMITIVE; // island enum // gen:105da4c159b1
pub const WINED3DSIH_DCL_INPUT_PS: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_INPUT_PS; // island enum // gen:733e1794a811
pub const WINED3DSIH_DCL_INPUT_PS_SGV: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_INPUT_PS_SGV; // island enum // gen:c9293ea3d2a1
pub const WINED3DSIH_DCL_INPUT_PS_SIV: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_INPUT_PS_SIV; // island enum // gen:a000bb3c3fd2
pub const WINED3DSIH_DCL_INPUT_SGV: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_INPUT_SGV; // island enum // gen:01c9d1105d6c
pub const WINED3DSIH_DCL_INPUT_SIV: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_INPUT_SIV; // island enum // gen:a8676b3daf22
pub const WINED3DSIH_DCL_INTERFACE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_INTERFACE; // island enum // gen:50b5015aeb7e
pub const WINED3DSIH_DCL_OUTPUT: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_OUTPUT; // island enum // gen:6b3bb56554a2
pub const WINED3DSIH_DCL_OUTPUT_CONTROL_POINT_COUNT: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_OUTPUT_CONTROL_POINT_COUNT; // island enum // gen:6bebc98af12a
pub const WINED3DSIH_DCL_OUTPUT_SIV: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_OUTPUT_SIV; // island enum // gen:cabbf168b693
pub const WINED3DSIH_DCL_OUTPUT_TOPOLOGY: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_OUTPUT_TOPOLOGY; // island enum // gen:76a4e9d4b23c
pub const WINED3DSIH_DCL_RESOURCE_RAW: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_RESOURCE_RAW; // island enum // gen:7b26f1e12c35
pub const WINED3DSIH_DCL_RESOURCE_STRUCTURED: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_RESOURCE_STRUCTURED; // island enum // gen:818b6f8958de
pub const WINED3DSIH_DCL_SAMPLER: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_SAMPLER; // island enum // gen:78e4d19b55c9
pub const WINED3DSIH_DCL_STREAM: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_STREAM; // island enum // gen:43aa2c4017de
pub const WINED3DSIH_DCL_TEMPS: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_TEMPS; // island enum // gen:84e404e77910
pub const WINED3DSIH_DCL_TESSELLATOR_DOMAIN: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_TESSELLATOR_DOMAIN; // island enum // gen:da69ab479c06
pub const WINED3DSIH_DCL_TESSELLATOR_OUTPUT_PRIMITIVE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_TESSELLATOR_OUTPUT_PRIMITIVE; // island enum // gen:fe6e9f6ad171
pub const WINED3DSIH_DCL_TESSELLATOR_PARTITIONING: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_TESSELLATOR_PARTITIONING; // island enum // gen:5cfa0a342590
pub const WINED3DSIH_DCL_TGSM_RAW: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_TGSM_RAW; // island enum // gen:eb3f3e0f9249
pub const WINED3DSIH_DCL_TGSM_STRUCTURED: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_TGSM_STRUCTURED; // island enum // gen:56869d0e3ace
pub const WINED3DSIH_DCL_THREAD_GROUP: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_THREAD_GROUP; // island enum // gen:32b8c0c06bef
pub const WINED3DSIH_DCL_UAV_RAW: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_UAV_RAW; // island enum // gen:912a6122a9b7
pub const WINED3DSIH_DCL_UAV_STRUCTURED: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_UAV_STRUCTURED; // island enum // gen:24341ce5fd2c
pub const WINED3DSIH_DCL_UAV_TYPED: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_UAV_TYPED; // island enum // gen:0623caa15768
pub const WINED3DSIH_DCL_VERTICES_OUT: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DCL_VERTICES_OUT; // island enum // gen:38e0677351b2
pub const WINED3DSIH_DEF: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DEF; // island enum // gen:c8b2625f03d1
pub const WINED3DSIH_DEFAULT: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DEFAULT; // island enum // gen:88425ab967ca
pub const WINED3DSIH_DEFB: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DEFB; // island enum // gen:3a9eeb75e82b
pub const WINED3DSIH_DEFI: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DEFI; // island enum // gen:53400fe66e8b
pub const WINED3DSIH_DIV: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DIV; // island enum // gen:eb34ab2f8047
pub const WINED3DSIH_DP2: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DP2; // island enum // gen:018940c3fd27
pub const WINED3DSIH_DP2ADD: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DP2ADD; // island enum // gen:1a398a3dba62
pub const WINED3DSIH_DP3: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DP3; // island enum // gen:500e6e607e20
pub const WINED3DSIH_DP4: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DP4; // island enum // gen:6f687a8dfa59
pub const WINED3DSIH_DST: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DST; // island enum // gen:48433327970e
pub const WINED3DSIH_DSX: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DSX; // island enum // gen:63dfa4b2e036
pub const WINED3DSIH_DSX_COARSE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DSX_COARSE; // island enum // gen:e0de8ba5776f
pub const WINED3DSIH_DSX_FINE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DSX_FINE; // island enum // gen:5ff09f5b7194
pub const WINED3DSIH_DSY: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DSY; // island enum // gen:1898afbd63ce
pub const WINED3DSIH_DSY_COARSE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DSY_COARSE; // island enum // gen:2e728e8bbfe8
pub const WINED3DSIH_DSY_FINE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_DSY_FINE; // island enum // gen:4cfeda877e74
pub const WINED3DSIH_ELSE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ELSE; // island enum // gen:90aff0405642
pub const WINED3DSIH_EMIT: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_EMIT; // island enum // gen:3c3639e36a47
pub const WINED3DSIH_EMIT_STREAM: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_EMIT_STREAM; // island enum // gen:d7cce072c3ac
pub const WINED3DSIH_ENDIF: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ENDIF; // island enum // gen:9a4fd1f89090
pub const WINED3DSIH_ENDLOOP: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ENDLOOP; // island enum // gen:62b5affe1cab
pub const WINED3DSIH_ENDREP: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ENDREP; // island enum // gen:2e893c175876
pub const WINED3DSIH_ENDSWITCH: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ENDSWITCH; // island enum // gen:ada1ea6be706
pub const WINED3DSIH_EQ: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_EQ; // island enum // gen:452ee8780a68
pub const WINED3DSIH_EVAL_CENTROID: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_EVAL_CENTROID; // island enum // gen:04a9b53c598b
pub const WINED3DSIH_EVAL_SAMPLE_INDEX: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_EVAL_SAMPLE_INDEX; // island enum // gen:3fb36d10436a
pub const WINED3DSIH_EXP: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_EXP; // island enum // gen:b1ebc5e789de
pub const WINED3DSIH_EXPP: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_EXPP; // island enum // gen:33e13d851e09
pub const WINED3DSIH_F16TOF32: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_F16TOF32; // island enum // gen:03a0e8ea0f4b
pub const WINED3DSIH_F32TOF16: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_F32TOF16; // island enum // gen:b83b95d2b691
pub const WINED3DSIH_FCALL: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_FCALL; // island enum // gen:2b30a65e564a
pub const WINED3DSIH_FIRSTBIT_HI: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_FIRSTBIT_HI; // island enum // gen:d7a0aaaf09a3
pub const WINED3DSIH_FIRSTBIT_LO: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_FIRSTBIT_LO; // island enum // gen:4d468a8ad309
pub const WINED3DSIH_FIRSTBIT_SHI: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_FIRSTBIT_SHI; // island enum // gen:40fe81319f78
pub const WINED3DSIH_FRC: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_FRC; // island enum // gen:f701c977041f
pub const WINED3DSIH_FTOI: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_FTOI; // island enum // gen:08ec04e9c3a9
pub const WINED3DSIH_FTOU: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_FTOU; // island enum // gen:b9d226bc6a22
pub const WINED3DSIH_GATHER4: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_GATHER4; // island enum // gen:c520b6b36539
pub const WINED3DSIH_GATHER4_C: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_GATHER4_C; // island enum // gen:20572dd47fee
pub const WINED3DSIH_GATHER4_PO: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_GATHER4_PO; // island enum // gen:9347df2d564a
pub const WINED3DSIH_GATHER4_PO_C: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_GATHER4_PO_C; // island enum // gen:4a4afca27626
pub const WINED3DSIH_GE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_GE; // island enum // gen:37dbb7f174be
pub const WINED3DSIH_HS_CONTROL_POINT_PHASE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_HS_CONTROL_POINT_PHASE; // island enum // gen:cafcec4dac98
pub const WINED3DSIH_HS_DECLS: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_HS_DECLS; // island enum // gen:a6ae96630f33
pub const WINED3DSIH_HS_FORK_PHASE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_HS_FORK_PHASE; // island enum // gen:8f4b025a9b05
pub const WINED3DSIH_HS_JOIN_PHASE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_HS_JOIN_PHASE; // island enum // gen:39d5597aced2
pub const WINED3DSIH_IADD: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_IADD; // island enum // gen:e6b1b243d91f
pub const WINED3DSIH_IBFE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_IBFE; // island enum // gen:f96e9e97c67d
pub const WINED3DSIH_IEQ: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_IEQ; // island enum // gen:c5b3e7126f28
pub const WINED3DSIH_IF: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_IF; // island enum // gen:e3d1f90ef23d
pub const WINED3DSIH_IFC: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_IFC; // island enum // gen:8082adb232bd
pub const WINED3DSIH_IGE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_IGE; // island enum // gen:c17ce3ab2877
pub const WINED3DSIH_ILT: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ILT; // island enum // gen:4785ca2072b3
pub const WINED3DSIH_IMAD: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_IMAD; // island enum // gen:c21d4bf8d0ca
pub const WINED3DSIH_IMAX: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_IMAX; // island enum // gen:558f23157e90
pub const WINED3DSIH_IMIN: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_IMIN; // island enum // gen:015a8d5a73a2
pub const WINED3DSIH_IMM_ATOMIC_ALLOC: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_IMM_ATOMIC_ALLOC; // island enum // gen:7a29af9e05bd
pub const WINED3DSIH_IMM_ATOMIC_AND: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_IMM_ATOMIC_AND; // island enum // gen:005b9783d063
pub const WINED3DSIH_IMM_ATOMIC_CMP_EXCH: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_IMM_ATOMIC_CMP_EXCH; // island enum // gen:d0563c9410f5
pub const WINED3DSIH_IMM_ATOMIC_CONSUME: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_IMM_ATOMIC_CONSUME; // island enum // gen:20e110dfe8dd
pub const WINED3DSIH_IMM_ATOMIC_EXCH: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_IMM_ATOMIC_EXCH; // island enum // gen:ef94893b7872
pub const WINED3DSIH_IMM_ATOMIC_IADD: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_IMM_ATOMIC_IADD; // island enum // gen:37c535b97ec3
pub const WINED3DSIH_IMM_ATOMIC_IMAX: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_IMM_ATOMIC_IMAX; // island enum // gen:994784cd88e8
pub const WINED3DSIH_IMM_ATOMIC_IMIN: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_IMM_ATOMIC_IMIN; // island enum // gen:94f259557dcf
pub const WINED3DSIH_IMM_ATOMIC_OR: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_IMM_ATOMIC_OR; // island enum // gen:971a7505b546
pub const WINED3DSIH_IMM_ATOMIC_UMAX: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_IMM_ATOMIC_UMAX; // island enum // gen:ff0ab123aba6
pub const WINED3DSIH_IMM_ATOMIC_UMIN: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_IMM_ATOMIC_UMIN; // island enum // gen:90f80ef47840
pub const WINED3DSIH_IMM_ATOMIC_XOR: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_IMM_ATOMIC_XOR; // island enum // gen:79dbb8732780
pub const WINED3DSIH_IMUL: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_IMUL; // island enum // gen:fc501bb9eb03
pub const WINED3DSIH_INE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_INE; // island enum // gen:1e1e6cc2846c
pub const WINED3DSIH_INEG: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_INEG; // island enum // gen:707eb7e1063f
pub const WINED3DSIH_ISHL: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ISHL; // island enum // gen:924f14e1ec4e
pub const WINED3DSIH_ISHR: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ISHR; // island enum // gen:e255546a8224
pub const WINED3DSIH_ITOF: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ITOF; // island enum // gen:821f5926229a
pub const WINED3DSIH_LABEL: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_LABEL; // island enum // gen:49ed4a14c9cf
pub const WINED3DSIH_LD2DMS: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_LD2DMS; // island enum // gen:99414984f01f
pub const WINED3DSIH_LD: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_LD; // island enum // gen:3dc22c0393a9
pub const WINED3DSIH_LD_RAW: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_LD_RAW; // island enum // gen:9a74f390c112
pub const WINED3DSIH_LD_STRUCTURED: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_LD_STRUCTURED; // island enum // gen:12474527e5b5
pub const WINED3DSIH_LD_UAV_TYPED: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_LD_UAV_TYPED; // island enum // gen:5c0f6199b635
pub const WINED3DSIH_LIT: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_LIT; // island enum // gen:bc2d485c57f6
pub const WINED3DSIH_LOD: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_LOD; // island enum // gen:86eafe5142a1
pub const WINED3DSIH_LOG: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_LOG; // island enum // gen:3693536872b1
pub const WINED3DSIH_LOGP: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_LOGP; // island enum // gen:4807b085677b
pub const WINED3DSIH_LOOP: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_LOOP; // island enum // gen:30e7bdcdff02
pub const WINED3DSIH_LRP: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_LRP; // island enum // gen:716a79ed19fa
pub const WINED3DSIH_LT: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_LT; // island enum // gen:7f64ff27ad29
pub const WINED3DSIH_M3x2: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_M3x2; // island enum // gen:52fe4fc1d40e
pub const WINED3DSIH_M3x3: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_M3x3; // island enum // gen:82f744efc7f3
pub const WINED3DSIH_M3x4: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_M3x4; // island enum // gen:7c719c2d0c1a
pub const WINED3DSIH_M4x3: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_M4x3; // island enum // gen:9eecb4898133
pub const WINED3DSIH_M4x4: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_M4x4; // island enum // gen:47207c1ba5d3
pub const WINED3DSIH_MAD: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_MAD; // island enum // gen:9cae5a7c5206
pub const WINED3DSIH_MAX: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_MAX; // island enum // gen:f64327176e5b
pub const WINED3DSIH_MIN: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_MIN; // island enum // gen:f2e0115641ab
pub const WINED3DSIH_MOV: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_MOV; // island enum // gen:f6e205bb6060
pub const WINED3DSIH_MOVA: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_MOVA; // island enum // gen:b69262d88467
pub const WINED3DSIH_MOVC: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_MOVC; // island enum // gen:e30d0f988ce3
pub const WINED3DSIH_MUL: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_MUL; // island enum // gen:0ff2c8f78d6e
pub const WINED3DSIH_NE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_NE; // island enum // gen:8a109b5f52fb
pub const WINED3DSIH_NOP: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_NOP; // island enum // gen:0b0b9e6de35f
pub const WINED3DSIH_NOT: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_NOT; // island enum // gen:f788e1629757
pub const WINED3DSIH_NRM: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_NRM; // island enum // gen:9d9207755b2d
pub const WINED3DSIH_OR: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_OR; // island enum // gen:ec56ff7a200c
pub const WINED3DSIH_PHASE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_PHASE; // island enum // gen:e926cf44de5b
pub const WINED3DSIH_POW: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_POW; // island enum // gen:bc901253991c
pub const WINED3DSIH_RCP: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_RCP; // island enum // gen:632d32912160
pub const WINED3DSIH_REP: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_REP; // island enum // gen:1676d70f7e5f
pub const WINED3DSIH_RESINFO: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_RESINFO; // island enum // gen:124808e66e21
pub const WINED3DSIH_RET: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_RET; // island enum // gen:dd15c400525f
pub const WINED3DSIH_RETP: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_RETP; // island enum // gen:d9f2aab1bb2e
pub const WINED3DSIH_ROUND_NE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ROUND_NE; // island enum // gen:66d969785466
pub const WINED3DSIH_ROUND_NI: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ROUND_NI; // island enum // gen:dde994402646
pub const WINED3DSIH_ROUND_PI: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ROUND_PI; // island enum // gen:28c8624278e1
pub const WINED3DSIH_ROUND_Z: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ROUND_Z; // island enum // gen:665ed9564856
pub const WINED3DSIH_RSQ: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_RSQ; // island enum // gen:96d1e1008b5b
pub const WINED3DSIH_SAMPLE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_SAMPLE; // island enum // gen:e577fbe60370
pub const WINED3DSIH_SAMPLE_B: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_SAMPLE_B; // island enum // gen:1bfb841c4e77
pub const WINED3DSIH_SAMPLE_C: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_SAMPLE_C; // island enum // gen:d5990a2fd941
pub const WINED3DSIH_SAMPLE_C_LZ: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_SAMPLE_C_LZ; // island enum // gen:d4922ed80da4
pub const WINED3DSIH_SAMPLE_GRAD: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_SAMPLE_GRAD; // island enum // gen:955a2a947bbb
pub const WINED3DSIH_SAMPLE_INFO: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_SAMPLE_INFO; // island enum // gen:a164818d63bf
pub const WINED3DSIH_SAMPLE_LOD: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_SAMPLE_LOD; // island enum // gen:b274cdfed947
pub const WINED3DSIH_SAMPLE_POS: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_SAMPLE_POS; // island enum // gen:7a131d20111a
pub const WINED3DSIH_SETP: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_SETP; // island enum // gen:2967e7230e51
pub const WINED3DSIH_SGE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_SGE; // island enum // gen:b37beb66be11
pub const WINED3DSIH_SGN: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_SGN; // island enum // gen:31fc09d4040e
pub const WINED3DSIH_SINCOS: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_SINCOS; // island enum // gen:63aa08e07860
pub const WINED3DSIH_SLT: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_SLT; // island enum // gen:ef932e289842
pub const WINED3DSIH_SQRT: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_SQRT; // island enum // gen:c3daf47372b1
pub const WINED3DSIH_STORE_RAW: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_STORE_RAW; // island enum // gen:6c5c541d475c
pub const WINED3DSIH_STORE_STRUCTURED: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_STORE_STRUCTURED; // island enum // gen:a69fb518c8b6
pub const WINED3DSIH_STORE_UAV_TYPED: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_STORE_UAV_TYPED; // island enum // gen:7b1ad2bc8540
pub const WINED3DSIH_SUB: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_SUB; // island enum // gen:4d9f975c01f1
pub const WINED3DSIH_SWAPC: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_SWAPC; // island enum // gen:0220083fafc7
pub const WINED3DSIH_SWITCH: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_SWITCH; // island enum // gen:3011ff5223e9
pub const WINED3DSIH_SYNC: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_SYNC; // island enum // gen:9e92bbd5533f
pub const WINED3DSIH_TABLE_SIZE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_TABLE_SIZE; // island enum // gen:b227e15ca7a9
pub const WINED3DSIH_TEX: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_TEX; // island enum // gen:e80cea391c8f
pub const WINED3DSIH_TEXBEM: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_TEXBEM; // island enum // gen:6f5b3a5f4ec6
pub const WINED3DSIH_TEXBEML: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_TEXBEML; // island enum // gen:1281fcde6dbb
pub const WINED3DSIH_TEXCOORD: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_TEXCOORD; // island enum // gen:f8b54198bc7a
pub const WINED3DSIH_TEXDEPTH: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_TEXDEPTH; // island enum // gen:49e9653da27a
pub const WINED3DSIH_TEXDP3: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_TEXDP3; // island enum // gen:c7706a3f34ff
pub const WINED3DSIH_TEXDP3TEX: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_TEXDP3TEX; // island enum // gen:a4de4cedba55
pub const WINED3DSIH_TEXKILL: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_TEXKILL; // island enum // gen:486733aead87
pub const WINED3DSIH_TEXLDD: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_TEXLDD; // island enum // gen:9ae43d377f55
pub const WINED3DSIH_TEXLDL: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_TEXLDL; // island enum // gen:2f82eb5f3b2e
pub const WINED3DSIH_TEXM3x2DEPTH: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_TEXM3x2DEPTH; // island enum // gen:7d79d1d70909
pub const WINED3DSIH_TEXM3x2PAD: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_TEXM3x2PAD; // island enum // gen:65d54dce0f2f
pub const WINED3DSIH_TEXM3x2TEX: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_TEXM3x2TEX; // island enum // gen:be465c859d1c
pub const WINED3DSIH_TEXM3x3: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_TEXM3x3; // island enum // gen:39bffa677515
pub const WINED3DSIH_TEXM3x3DIFF: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_TEXM3x3DIFF; // island enum // gen:231b528d912e
pub const WINED3DSIH_TEXM3x3PAD: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_TEXM3x3PAD; // island enum // gen:d995efd04f3e
pub const WINED3DSIH_TEXM3x3SPEC: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_TEXM3x3SPEC; // island enum // gen:030f5eac9f4a
pub const WINED3DSIH_TEXM3x3TEX: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_TEXM3x3TEX; // island enum // gen:1007513407a0
pub const WINED3DSIH_TEXM3x3VSPEC: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_TEXM3x3VSPEC; // island enum // gen:ce175814d766
pub const WINED3DSIH_TEXREG2AR: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_TEXREG2AR; // island enum // gen:b58051f5fe58
pub const WINED3DSIH_TEXREG2GB: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_TEXREG2GB; // island enum // gen:4700af7cf196
pub const WINED3DSIH_TEXREG2RGB: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_TEXREG2RGB; // island enum // gen:acba67ec2e9d
pub const WINED3DSIH_UBFE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_UBFE; // island enum // gen:0de5142c7cdc
pub const WINED3DSIH_UDIV: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_UDIV; // island enum // gen:9501b76f455f
pub const WINED3DSIH_UGE: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_UGE; // island enum // gen:df8f4e4aa8ce
pub const WINED3DSIH_ULT: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_ULT; // island enum // gen:d7f0d3773d45
pub const WINED3DSIH_UMAX: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_UMAX; // island enum // gen:2bc78beb3830
pub const WINED3DSIH_UMIN: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_UMIN; // island enum // gen:20dd962434c4
pub const WINED3DSIH_UMUL: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_UMUL; // island enum // gen:558864fd387c
pub const WINED3DSIH_USHR: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_USHR; // island enum // gen:51318aaf17e4
pub const WINED3DSIH_UTOF: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_UTOF; // island enum // gen:d01c538649bb
pub const WINED3DSIH_XOR: WINED3D_SHADER_INSTRUCTION_HANDLER = WINED3D_SHADER_INSTRUCTION_HANDLER_WINED3DSIH_XOR; // island enum // gen:297a10d21721
pub const WINED3DSIM_CONSTANT: wined3d_shader_interpolation_mode = wined3d_shader_interpolation_mode_WINED3DSIM_CONSTANT; // island enum // gen:1728ccef63e8
pub const WINED3DSIM_LINEAR: wined3d_shader_interpolation_mode = wined3d_shader_interpolation_mode_WINED3DSIM_LINEAR; // island enum // gen:45d187bbf70d
pub const WINED3DSIM_LINEAR_NOPERSPECTIVE: wined3d_shader_interpolation_mode = wined3d_shader_interpolation_mode_WINED3DSIM_LINEAR_NOPERSPECTIVE; // island enum // gen:716c12561603
pub const WINED3DSIM_LINEAR_NOPERSPECTIVE_SAMPLE: wined3d_shader_interpolation_mode = wined3d_shader_interpolation_mode_WINED3DSIM_LINEAR_NOPERSPECTIVE_SAMPLE; // island enum // gen:6fa67b50df36
pub const WINED3DSIM_NONE: wined3d_shader_interpolation_mode = wined3d_shader_interpolation_mode_WINED3DSIM_NONE; // island enum // gen:d1f19ae6fed0
pub const WINED3DSPDM_MSAMPCENTROID: wined3d_shader_dst_modifier = wined3d_shader_dst_modifier_WINED3DSPDM_MSAMPCENTROID; // island enum // gen:2fe62493ccd7
pub const WINED3DSPDM_NONE: wined3d_shader_dst_modifier = wined3d_shader_dst_modifier_WINED3DSPDM_NONE; // island enum // gen:f3167b017be2
pub const WINED3DSPDM_PARTIALPRECISION: wined3d_shader_dst_modifier = wined3d_shader_dst_modifier_WINED3DSPDM_PARTIALPRECISION; // island enum // gen:ab9e436f385d
pub const WINED3DSPDM_SATURATE: wined3d_shader_dst_modifier = wined3d_shader_dst_modifier_WINED3DSPDM_SATURATE; // island enum // gen:bf9c2f5e1e85
pub const WINED3DSPR_ADDR: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_ADDR; // island enum // gen:d9e0b3f9339d
pub const WINED3DSPR_ATTROUT: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_ATTROUT; // island enum // gen:9145c3e0b45b
pub const WINED3DSPR_COLOROUT: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_COLOROUT; // island enum // gen:2a7d467ef633
pub const WINED3DSPR_CONST: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_CONST; // island enum // gen:85171af58de4
pub const WINED3DSPR_CONSTBOOL: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_CONSTBOOL; // island enum // gen:1a656125e18f
pub const WINED3DSPR_CONSTBUFFER: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_CONSTBUFFER; // island enum // gen:6e96a174c463
pub const WINED3DSPR_CONSTINT: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_CONSTINT; // island enum // gen:43d68673de5e
pub const WINED3DSPR_COVERAGE: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_COVERAGE; // island enum // gen:1829f524a877
pub const WINED3DSPR_DEPTHOUT: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_DEPTHOUT; // island enum // gen:cbd8253737dc
pub const WINED3DSPR_DEPTHOUTGE: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_DEPTHOUTGE; // island enum // gen:065bf2d90bba
pub const WINED3DSPR_DEPTHOUTLE: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_DEPTHOUTLE; // island enum // gen:dceab5186dee
pub const WINED3DSPR_FORKINSTID: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_FORKINSTID; // island enum // gen:0281fbcf1905
pub const WINED3DSPR_FUNCTIONBODY: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_FUNCTIONBODY; // island enum // gen:246af57685da
pub const WINED3DSPR_FUNCTIONPOINTER: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_FUNCTIONPOINTER; // island enum // gen:5b2cc4f0eba0
pub const WINED3DSPR_GROUPSHAREDMEM: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_GROUPSHAREDMEM; // island enum // gen:10500041659b
pub const WINED3DSPR_GSINSTID: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_GSINSTID; // island enum // gen:cddb98a6ba74
pub const WINED3DSPR_IDXTEMP: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_IDXTEMP; // island enum // gen:f0aafa0cae7a
pub const WINED3DSPR_IMMCONST: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_IMMCONST; // island enum // gen:6e96557197fb
pub const WINED3DSPR_IMMCONSTBUFFER: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_IMMCONSTBUFFER; // island enum // gen:f07bd21f2c84
pub const WINED3DSPR_INCONTROLPOINT: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_INCONTROLPOINT; // island enum // gen:780b8ec3909d
pub const WINED3DSPR_INPUT: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_INPUT; // island enum // gen:2473185d2c64
pub const WINED3DSPR_JOININSTID: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_JOININSTID; // island enum // gen:97d34d6ba3b8
pub const WINED3DSPR_LOCALTHREADID: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_LOCALTHREADID; // island enum // gen:b3d36ba2041c
pub const WINED3DSPR_LOCALTHREADINDEX: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_LOCALTHREADINDEX; // island enum // gen:bc367a257a3b
pub const WINED3DSPR_LOOP: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_LOOP; // island enum // gen:16c3651f33a6
pub const WINED3DSPR_MISCTYPE: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_MISCTYPE; // island enum // gen:4e888d0cf7db
pub const WINED3DSPR_NULL: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_NULL; // island enum // gen:23f4e2e816a0
pub const WINED3DSPR_OUTCONTROLPOINT: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_OUTCONTROLPOINT; // island enum // gen:1aa0b2b7f2d5
pub const WINED3DSPR_OUTPOINTID: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_OUTPOINTID; // island enum // gen:d46ab73142f0
pub const WINED3DSPR_OUTPUT: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_OUTPUT; // island enum // gen:61737dc44dbb
pub const WINED3DSPR_PATCHCONST: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_PATCHCONST; // island enum // gen:d2c1a3414051
pub const WINED3DSPR_PREDICATE: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_PREDICATE; // island enum // gen:a7f4e6ee934f
pub const WINED3DSPR_PRIMID: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_PRIMID; // island enum // gen:f311ddf4eecc
pub const WINED3DSPR_RASTERIZER: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_RASTERIZER; // island enum // gen:60f67651b6bd
pub const WINED3DSPR_RASTOUT: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_RASTOUT; // island enum // gen:d9c271f383c0
pub const WINED3DSPR_RESOURCE: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_RESOURCE; // island enum // gen:6840d8ed4cbd
pub const WINED3DSPR_SAMPLEMASK: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_SAMPLEMASK; // island enum // gen:58941fbfb7e0
pub const WINED3DSPR_SAMPLER: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_SAMPLER; // island enum // gen:e1e489bb0d32
pub const WINED3DSPR_STENCILREF: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_STENCILREF; // island enum // gen:da0234591b7f
pub const WINED3DSPR_STREAM: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_STREAM; // island enum // gen:1e1dc5988f33
pub const WINED3DSPR_TEMP: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_TEMP; // island enum // gen:19829679c4a0
pub const WINED3DSPR_TESSCOORD: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_TESSCOORD; // island enum // gen:94be4a452b85
pub const WINED3DSPR_TEXCRDOUT: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_TEXCRDOUT; // island enum // gen:cdea3b1111e2
pub const WINED3DSPR_TEXTURE: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_TEXTURE; // island enum // gen:57230991fe14
pub const WINED3DSPR_THREADGROUPID: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_THREADGROUPID; // island enum // gen:355596a82b8b
pub const WINED3DSPR_THREADID: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_THREADID; // island enum // gen:c77a8e30d916
pub const WINED3DSPR_UAV: wined3d_shader_register_type = wined3d_shader_register_type_WINED3DSPR_UAV; // island enum // gen:34c1ecfe6843
pub const WINED3DSPSM_ABS: wined3d_shader_src_modifier = wined3d_shader_src_modifier_WINED3DSPSM_ABS; // island enum // gen:afea770b164d
pub const WINED3DSPSM_ABSNEG: wined3d_shader_src_modifier = wined3d_shader_src_modifier_WINED3DSPSM_ABSNEG; // island enum // gen:b08a07b0c70a
pub const WINED3DSPSM_BIAS: wined3d_shader_src_modifier = wined3d_shader_src_modifier_WINED3DSPSM_BIAS; // island enum // gen:e16d4bd0308c
pub const WINED3DSPSM_BIASNEG: wined3d_shader_src_modifier = wined3d_shader_src_modifier_WINED3DSPSM_BIASNEG; // island enum // gen:45baaaa63ff7
pub const WINED3DSPSM_COMP: wined3d_shader_src_modifier = wined3d_shader_src_modifier_WINED3DSPSM_COMP; // island enum // gen:8355fda73924
pub const WINED3DSPSM_DW: wined3d_shader_src_modifier = wined3d_shader_src_modifier_WINED3DSPSM_DW; // island enum // gen:41ed69cf437b
pub const WINED3DSPSM_DZ: wined3d_shader_src_modifier = wined3d_shader_src_modifier_WINED3DSPSM_DZ; // island enum // gen:2ba280165909
pub const WINED3DSPSM_NEG: wined3d_shader_src_modifier = wined3d_shader_src_modifier_WINED3DSPSM_NEG; // island enum // gen:c149e4578d4e
pub const WINED3DSPSM_NONE: wined3d_shader_src_modifier = wined3d_shader_src_modifier_WINED3DSPSM_NONE; // island enum // gen:79055e3664d4
pub const WINED3DSPSM_NOT: wined3d_shader_src_modifier = wined3d_shader_src_modifier_WINED3DSPSM_NOT; // island enum // gen:2aeb34e4184c
pub const WINED3DSPSM_SIGN: wined3d_shader_src_modifier = wined3d_shader_src_modifier_WINED3DSPSM_SIGN; // island enum // gen:22323c58ff22
pub const WINED3DSPSM_SIGNNEG: wined3d_shader_src_modifier = wined3d_shader_src_modifier_WINED3DSPSM_SIGNNEG; // island enum // gen:ca0eaf521f21
pub const WINED3DSPSM_X2: wined3d_shader_src_modifier = wined3d_shader_src_modifier_WINED3DSPSM_X2; // island enum // gen:3f5e87240067
pub const WINED3DSPSM_X2NEG: wined3d_shader_src_modifier = wined3d_shader_src_modifier_WINED3DSPSM_X2NEG; // island enum // gen:4855c06fb3d6
pub const WINED3DSSF_GLOBAL_UAV: wined3d_shader_sync_flags = wined3d_shader_sync_flags_WINED3DSSF_GLOBAL_UAV; // island enum // gen:0e3c6d7eae12
pub const WINED3DSSF_GROUP_SHARED_MEMORY: wined3d_shader_sync_flags = wined3d_shader_sync_flags_WINED3DSSF_GROUP_SHARED_MEMORY; // island enum // gen:526cc1b2c445
pub const WINED3DSSF_THREAD_GROUP: wined3d_shader_sync_flags = wined3d_shader_sync_flags_WINED3DSSF_THREAD_GROUP; // island enum // gen:3d0b140c7e67
pub const WINED3D_BLEND_BLENDFACTOR: wined3d_blend = wined3d_blend_WINED3D_BLEND_BLENDFACTOR; // island enum // gen:8f59ea86d6c2
pub const WINED3D_BLEND_BOTHINVSRCALPHA: wined3d_blend = wined3d_blend_WINED3D_BLEND_BOTHINVSRCALPHA; // island enum // gen:ae4699ac249b
pub const WINED3D_BLEND_BOTHSRCALPHA: wined3d_blend = wined3d_blend_WINED3D_BLEND_BOTHSRCALPHA; // island enum // gen:61b0549503ef
pub const WINED3D_BLEND_DESTALPHA: wined3d_blend = wined3d_blend_WINED3D_BLEND_DESTALPHA; // island enum // gen:e1e8e1959f32
pub const WINED3D_BLEND_DESTCOLOR: wined3d_blend = wined3d_blend_WINED3D_BLEND_DESTCOLOR; // island enum // gen:7025c3eeb44b
pub const WINED3D_BLEND_INVBLENDFACTOR: wined3d_blend = wined3d_blend_WINED3D_BLEND_INVBLENDFACTOR; // island enum // gen:57d334214ff7
pub const WINED3D_BLEND_INVDESTALPHA: wined3d_blend = wined3d_blend_WINED3D_BLEND_INVDESTALPHA; // island enum // gen:61b3ae0afc69
pub const WINED3D_BLEND_INVDESTCOLOR: wined3d_blend = wined3d_blend_WINED3D_BLEND_INVDESTCOLOR; // island enum // gen:9f380ed9fc2a
pub const WINED3D_BLEND_INVSRC1ALPHA: wined3d_blend = wined3d_blend_WINED3D_BLEND_INVSRC1ALPHA; // island enum // gen:c262e5635c0f
pub const WINED3D_BLEND_INVSRC1COLOR: wined3d_blend = wined3d_blend_WINED3D_BLEND_INVSRC1COLOR; // island enum // gen:a327350349cb
pub const WINED3D_BLEND_INVSRCALPHA: wined3d_blend = wined3d_blend_WINED3D_BLEND_INVSRCALPHA; // island enum // gen:7c910dbe7c8d
pub const WINED3D_BLEND_INVSRCCOLOR: wined3d_blend = wined3d_blend_WINED3D_BLEND_INVSRCCOLOR; // island enum // gen:04a3e554b4f4
pub const WINED3D_BLEND_ONE: wined3d_blend = wined3d_blend_WINED3D_BLEND_ONE; // island enum // gen:5384bbd7e04c
pub const WINED3D_BLEND_OP_ADD: wined3d_blend_op = wined3d_blend_op_WINED3D_BLEND_OP_ADD; // island enum // gen:46e0e33e1652
pub const WINED3D_BLEND_OP_MAX: wined3d_blend_op = wined3d_blend_op_WINED3D_BLEND_OP_MAX; // island enum // gen:ac2534bcbe72
pub const WINED3D_BLEND_OP_MIN: wined3d_blend_op = wined3d_blend_op_WINED3D_BLEND_OP_MIN; // island enum // gen:12534a0f7a08
pub const WINED3D_BLEND_OP_REVSUBTRACT: wined3d_blend_op = wined3d_blend_op_WINED3D_BLEND_OP_REVSUBTRACT; // island enum // gen:a6091ce43304
pub const WINED3D_BLEND_OP_SUBTRACT: wined3d_blend_op = wined3d_blend_op_WINED3D_BLEND_OP_SUBTRACT; // island enum // gen:2d6fac4a8997
pub const WINED3D_BLEND_SRC1ALPHA: wined3d_blend = wined3d_blend_WINED3D_BLEND_SRC1ALPHA; // island enum // gen:ab221117148d
pub const WINED3D_BLEND_SRC1COLOR: wined3d_blend = wined3d_blend_WINED3D_BLEND_SRC1COLOR; // island enum // gen:b7341d3a2f84
pub const WINED3D_BLEND_SRCALPHA: wined3d_blend = wined3d_blend_WINED3D_BLEND_SRCALPHA; // island enum // gen:7df89c7435a7
pub const WINED3D_BLEND_SRCALPHASAT: wined3d_blend = wined3d_blend_WINED3D_BLEND_SRCALPHASAT; // island enum // gen:a99eecf10f7f
pub const WINED3D_BLEND_SRCCOLOR: wined3d_blend = wined3d_blend_WINED3D_BLEND_SRCCOLOR; // island enum // gen:7b9c661f3637
pub const WINED3D_BLEND_ZERO: wined3d_blend = wined3d_blend_WINED3D_BLEND_ZERO; // island enum // gen:2a4e9e59d089
pub const WINED3D_BLIT_OP_COLOR_BLIT: wined3d_blit_op = wined3d_blit_op_WINED3D_BLIT_OP_COLOR_BLIT; // island enum // gen:68cb775bf1d1
pub const WINED3D_BLIT_OP_COLOR_BLIT_ALPHATEST: wined3d_blit_op = wined3d_blit_op_WINED3D_BLIT_OP_COLOR_BLIT_ALPHATEST; // island enum // gen:5df4f81ebab9
pub const WINED3D_BLIT_OP_COLOR_BLIT_CKEY: wined3d_blit_op = wined3d_blit_op_WINED3D_BLIT_OP_COLOR_BLIT_CKEY; // island enum // gen:7999c109c911
pub const WINED3D_BLIT_OP_DEPTH_BLIT: wined3d_blit_op = wined3d_blit_op_WINED3D_BLIT_OP_DEPTH_BLIT; // island enum // gen:63621ba7d0bc
pub const WINED3D_BLIT_OP_RAW_BLIT: wined3d_blit_op = wined3d_blit_op_WINED3D_BLIT_OP_RAW_BLIT; // island enum // gen:4146a599db4d
pub const WINED3D_CHANNEL_TYPE_DEPTH: wined3d_channel_type = wined3d_channel_type_WINED3D_CHANNEL_TYPE_DEPTH; // island enum // gen:829ca10a376c
pub const WINED3D_CHANNEL_TYPE_FLOAT: wined3d_channel_type = wined3d_channel_type_WINED3D_CHANNEL_TYPE_FLOAT; // island enum // gen:abbc4fa36d49
pub const WINED3D_CHANNEL_TYPE_NONE: wined3d_channel_type = wined3d_channel_type_WINED3D_CHANNEL_TYPE_NONE; // island enum // gen:02761995d950
pub const WINED3D_CHANNEL_TYPE_SINT: wined3d_channel_type = wined3d_channel_type_WINED3D_CHANNEL_TYPE_SINT; // island enum // gen:2eb589054b02
pub const WINED3D_CHANNEL_TYPE_SNORM: wined3d_channel_type = wined3d_channel_type_WINED3D_CHANNEL_TYPE_SNORM; // island enum // gen:118c681248c4
pub const WINED3D_CHANNEL_TYPE_STENCIL: wined3d_channel_type = wined3d_channel_type_WINED3D_CHANNEL_TYPE_STENCIL; // island enum // gen:909557fb2163
pub const WINED3D_CHANNEL_TYPE_UINT: wined3d_channel_type = wined3d_channel_type_WINED3D_CHANNEL_TYPE_UINT; // island enum // gen:b0d7d8c2ab89
pub const WINED3D_CHANNEL_TYPE_UNORM: wined3d_channel_type = wined3d_channel_type_WINED3D_CHANNEL_TYPE_UNORM; // island enum // gen:e179574a3061
pub const WINED3D_CHANNEL_TYPE_UNUSED: wined3d_channel_type = wined3d_channel_type_WINED3D_CHANNEL_TYPE_UNUSED; // island enum // gen:cd22a4958c18
pub const WINED3D_CMP_ALWAYS: wined3d_cmp_func = wined3d_cmp_func_WINED3D_CMP_ALWAYS; // island enum // gen:7fe00df4b75f
pub const WINED3D_CMP_EQUAL: wined3d_cmp_func = wined3d_cmp_func_WINED3D_CMP_EQUAL; // island enum // gen:dba93966e3b2
pub const WINED3D_CMP_GREATER: wined3d_cmp_func = wined3d_cmp_func_WINED3D_CMP_GREATER; // island enum // gen:1972743e6bb1
pub const WINED3D_CMP_GREATEREQUAL: wined3d_cmp_func = wined3d_cmp_func_WINED3D_CMP_GREATEREQUAL; // island enum // gen:a1083993512f
pub const WINED3D_CMP_LESS: wined3d_cmp_func = wined3d_cmp_func_WINED3D_CMP_LESS; // island enum // gen:a6bba25529fc
pub const WINED3D_CMP_LESSEQUAL: wined3d_cmp_func = wined3d_cmp_func_WINED3D_CMP_LESSEQUAL; // island enum // gen:151c7de1889f
pub const WINED3D_CMP_NEVER: wined3d_cmp_func = wined3d_cmp_func_WINED3D_CMP_NEVER; // island enum // gen:8072cf93856e
pub const WINED3D_CMP_NOTEQUAL: wined3d_cmp_func = wined3d_cmp_func_WINED3D_CMP_NOTEQUAL; // island enum // gen:fd7dcd8e2b35
pub const WINED3D_CS_QUEUE_DEFAULT: wined3d_cs_queue_id = wined3d_cs_queue_id_WINED3D_CS_QUEUE_DEFAULT; // island enum // gen:0a42ad0feaac
pub const WINED3D_CS_QUEUE_MAP: wined3d_cs_queue_id = wined3d_cs_queue_id_WINED3D_CS_QUEUE_MAP; // island enum // gen:c68f59717886
pub const WINED3D_CULL_BACK: wined3d_cull = wined3d_cull_WINED3D_CULL_BACK; // island enum // gen:3e5c69626cbb
pub const WINED3D_CULL_FRONT: wined3d_cull = wined3d_cull_WINED3D_CULL_FRONT; // island enum // gen:5c841631877d
pub const WINED3D_CULL_NONE: wined3d_cull = wined3d_cull_WINED3D_CULL_NONE; // island enum // gen:ee175d2e9b5d
pub const WINED3D_DATA_FLOAT: wined3d_data_type = wined3d_data_type_WINED3D_DATA_FLOAT; // island enum // gen:bfd5ad76a7a2
pub const WINED3D_DATA_INT: wined3d_data_type = wined3d_data_type_WINED3D_DATA_INT; // island enum // gen:068b6b7e75a4
pub const WINED3D_DATA_OPAQUE: wined3d_data_type = wined3d_data_type_WINED3D_DATA_OPAQUE; // island enum // gen:54d40850243f
pub const WINED3D_DATA_RESOURCE: wined3d_data_type = wined3d_data_type_WINED3D_DATA_RESOURCE; // island enum // gen:a8e9ba7e3f04
pub const WINED3D_DATA_SAMPLER: wined3d_data_type = wined3d_data_type_WINED3D_DATA_SAMPLER; // island enum // gen:de8715dd543d
pub const WINED3D_DATA_SNORM: wined3d_data_type = wined3d_data_type_WINED3D_DATA_SNORM; // island enum // gen:2abae0bbef2d
pub const WINED3D_DATA_UAV: wined3d_data_type = wined3d_data_type_WINED3D_DATA_UAV; // island enum // gen:e93aae008bc5
pub const WINED3D_DATA_UINT: wined3d_data_type = wined3d_data_type_WINED3D_DATA_UINT; // island enum // gen:1d101498512e
pub const WINED3D_DATA_UNORM: wined3d_data_type = wined3d_data_type_WINED3D_DATA_UNORM; // island enum // gen:3c63180cb166
pub const WINED3D_DECL_METHOD_CROSS_UV: wined3d_decl_method = wined3d_decl_method_WINED3D_DECL_METHOD_CROSS_UV; // island enum // gen:9b7daa9254c1
pub const WINED3D_DECL_METHOD_DEFAULT: wined3d_decl_method = wined3d_decl_method_WINED3D_DECL_METHOD_DEFAULT; // island enum // gen:317b339f67b0
pub const WINED3D_DECL_METHOD_LOOKUP: wined3d_decl_method = wined3d_decl_method_WINED3D_DECL_METHOD_LOOKUP; // island enum // gen:0cd0a1983d5d
pub const WINED3D_DECL_METHOD_LOOKUP_PRESAMPLED: wined3d_decl_method = wined3d_decl_method_WINED3D_DECL_METHOD_LOOKUP_PRESAMPLED; // island enum // gen:3207b9d56e54
pub const WINED3D_DECL_METHOD_PARTIAL_U: wined3d_decl_method = wined3d_decl_method_WINED3D_DECL_METHOD_PARTIAL_U; // island enum // gen:83ee51f09282
pub const WINED3D_DECL_METHOD_PARTIAL_V: wined3d_decl_method = wined3d_decl_method_WINED3D_DECL_METHOD_PARTIAL_V; // island enum // gen:7f63f650ce13
pub const WINED3D_DECL_METHOD_UV: wined3d_decl_method = wined3d_decl_method_WINED3D_DECL_METHOD_UV; // island enum // gen:9239dcc9dbe2
pub const WINED3D_DECL_USAGE_BINORMAL: wined3d_decl_usage = wined3d_decl_usage_WINED3D_DECL_USAGE_BINORMAL; // island enum // gen:f8af66f33109
pub const WINED3D_DECL_USAGE_BLEND_INDICES: wined3d_decl_usage = wined3d_decl_usage_WINED3D_DECL_USAGE_BLEND_INDICES; // island enum // gen:819fcd50e5a1
pub const WINED3D_DECL_USAGE_BLEND_WEIGHT: wined3d_decl_usage = wined3d_decl_usage_WINED3D_DECL_USAGE_BLEND_WEIGHT; // island enum // gen:27239a351146
pub const WINED3D_DECL_USAGE_COLOR: wined3d_decl_usage = wined3d_decl_usage_WINED3D_DECL_USAGE_COLOR; // island enum // gen:af9841f56ae3
pub const WINED3D_DECL_USAGE_DEPTH: wined3d_decl_usage = wined3d_decl_usage_WINED3D_DECL_USAGE_DEPTH; // island enum // gen:3b53ab08228d
pub const WINED3D_DECL_USAGE_FOG: wined3d_decl_usage = wined3d_decl_usage_WINED3D_DECL_USAGE_FOG; // island enum // gen:a599cca24859
pub const WINED3D_DECL_USAGE_NORMAL: wined3d_decl_usage = wined3d_decl_usage_WINED3D_DECL_USAGE_NORMAL; // island enum // gen:b5b8ef4a975f
pub const WINED3D_DECL_USAGE_POSITION: wined3d_decl_usage = wined3d_decl_usage_WINED3D_DECL_USAGE_POSITION; // island enum // gen:232ccb2172ff
pub const WINED3D_DECL_USAGE_POSITIONT: wined3d_decl_usage = wined3d_decl_usage_WINED3D_DECL_USAGE_POSITIONT; // island enum // gen:39f97ec22ae9
pub const WINED3D_DECL_USAGE_PSIZE: wined3d_decl_usage = wined3d_decl_usage_WINED3D_DECL_USAGE_PSIZE; // island enum // gen:d7a80e727f7c
pub const WINED3D_DECL_USAGE_SAMPLE: wined3d_decl_usage = wined3d_decl_usage_WINED3D_DECL_USAGE_SAMPLE; // island enum // gen:99485ac80fd0
pub const WINED3D_DECL_USAGE_TANGENT: wined3d_decl_usage = wined3d_decl_usage_WINED3D_DECL_USAGE_TANGENT; // island enum // gen:c145c76f8a51
pub const WINED3D_DECL_USAGE_TESS_FACTOR: wined3d_decl_usage = wined3d_decl_usage_WINED3D_DECL_USAGE_TESS_FACTOR; // island enum // gen:191a3c89cb42
pub const WINED3D_DECL_USAGE_TEXCOORD: wined3d_decl_usage = wined3d_decl_usage_WINED3D_DECL_USAGE_TEXCOORD; // island enum // gen:ada9b7718732
pub const WINED3D_DEGREE_CUBIC: wined3d_degree_type = wined3d_degree_type_WINED3D_DEGREE_CUBIC; // island enum // gen:7b0c4689ab32
pub const WINED3D_DEGREE_LINEAR: wined3d_degree_type = wined3d_degree_type_WINED3D_DEGREE_LINEAR; // island enum // gen:043948d5efd8
pub const WINED3D_DEVICE_TYPE_HAL: wined3d_device_type = wined3d_device_type_WINED3D_DEVICE_TYPE_HAL; // island enum // gen:d73e66f36b9d
pub const WINED3D_DEVICE_TYPE_REF: wined3d_device_type = wined3d_device_type_WINED3D_DEVICE_TYPE_REF; // island enum // gen:c435a2c3156e
pub const WINED3D_DEVICE_TYPE_SW: wined3d_device_type = wined3d_device_type_WINED3D_DEVICE_TYPE_SW; // island enum // gen:3ce94f2a146f
pub const WINED3D_DISPLAY_ROTATION_0: wined3d_display_rotation = wined3d_display_rotation_WINED3D_DISPLAY_ROTATION_0; // island enum // gen:d59f40058026
pub const WINED3D_DISPLAY_ROTATION_180: wined3d_display_rotation = wined3d_display_rotation_WINED3D_DISPLAY_ROTATION_180; // island enum // gen:ceea514b42ae
pub const WINED3D_DISPLAY_ROTATION_270: wined3d_display_rotation = wined3d_display_rotation_WINED3D_DISPLAY_ROTATION_270; // island enum // gen:c0cf1a1daed1
pub const WINED3D_DISPLAY_ROTATION_90: wined3d_display_rotation = wined3d_display_rotation_WINED3D_DISPLAY_ROTATION_90; // island enum // gen:a9f517e12197
pub const WINED3D_DISPLAY_ROTATION_UNSPECIFIED: wined3d_display_rotation = wined3d_display_rotation_WINED3D_DISPLAY_ROTATION_UNSPECIFIED; // island enum // gen:7003685f35cf
pub const WINED3D_FEATURE_LEVEL_10: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_10; // island enum // gen:112f22b6763c
pub const WINED3D_FEATURE_LEVEL_10_1: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_10_1; // island enum // gen:c19f96297a4a
pub const WINED3D_FEATURE_LEVEL_11: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_11; // island enum // gen:ca926f110819
pub const WINED3D_FEATURE_LEVEL_11_1: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_11_1; // island enum // gen:ff994bf420fa
pub const WINED3D_FEATURE_LEVEL_5: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_5; // island enum // gen:f9c53f3d677f
pub const WINED3D_FEATURE_LEVEL_6: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_6; // island enum // gen:e970439b7cb3
pub const WINED3D_FEATURE_LEVEL_7: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_7; // island enum // gen:03cd701a1439
pub const WINED3D_FEATURE_LEVEL_8: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_8; // island enum // gen:396b2980b189
pub const WINED3D_FEATURE_LEVEL_9_1: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_9_1; // island enum // gen:5d3d08e56b20
pub const WINED3D_FEATURE_LEVEL_9_2: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_9_2; // island enum // gen:a1c2cdf52b31
pub const WINED3D_FEATURE_LEVEL_9_3: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_9_3; // island enum // gen:1666a64f5f90
pub const WINED3D_FEATURE_LEVEL_NONE: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_NONE; // island enum // gen:c53bcb83b45b
pub const WINED3D_FENCE_ERROR: wined3d_fence_result = wined3d_fence_result_WINED3D_FENCE_ERROR; // island enum // gen:dff049b44355
pub const WINED3D_FENCE_NOT_STARTED: wined3d_fence_result = wined3d_fence_result_WINED3D_FENCE_NOT_STARTED; // island enum // gen:0f3f2b23144c
pub const WINED3D_FENCE_OK: wined3d_fence_result = wined3d_fence_result_WINED3D_FENCE_OK; // island enum // gen:686f218bc377
pub const WINED3D_FENCE_WAITING: wined3d_fence_result = wined3d_fence_result_WINED3D_FENCE_WAITING; // island enum // gen:59953332d045
pub const WINED3D_FENCE_WRONG_THREAD: wined3d_fence_result = wined3d_fence_result_WINED3D_FENCE_WRONG_THREAD; // island enum // gen:7ec957325b56
pub const WINED3D_FFP_ATTRIBS_COUNT: wined3d_ffp_idx = wined3d_ffp_idx_WINED3D_FFP_ATTRIBS_COUNT; // island enum // gen:7d943682d222
pub const WINED3D_FFP_BLENDINDICES: wined3d_ffp_idx = wined3d_ffp_idx_WINED3D_FFP_BLENDINDICES; // island enum // gen:8995c9e4e89d
pub const WINED3D_FFP_BLENDWEIGHT: wined3d_ffp_idx = wined3d_ffp_idx_WINED3D_FFP_BLENDWEIGHT; // island enum // gen:59f639aa7d9c
pub const WINED3D_FFP_DIFFUSE: wined3d_ffp_idx = wined3d_ffp_idx_WINED3D_FFP_DIFFUSE; // island enum // gen:5ed302270090
pub const WINED3D_FFP_EMIT_COUNT: wined3d_ffp_emit_idx = wined3d_ffp_emit_idx_WINED3D_FFP_EMIT_COUNT; // island enum // gen:be6d3148d970
pub const WINED3D_FFP_EMIT_D3DCOLOR: wined3d_ffp_emit_idx = wined3d_ffp_emit_idx_WINED3D_FFP_EMIT_D3DCOLOR; // island enum // gen:44910bba4b0f
pub const WINED3D_FFP_EMIT_DEC3N: wined3d_ffp_emit_idx = wined3d_ffp_emit_idx_WINED3D_FFP_EMIT_DEC3N; // island enum // gen:25a3871f6fc6
pub const WINED3D_FFP_EMIT_FLOAT16_2: wined3d_ffp_emit_idx = wined3d_ffp_emit_idx_WINED3D_FFP_EMIT_FLOAT16_2; // island enum // gen:7046edcd8e02
pub const WINED3D_FFP_EMIT_FLOAT16_4: wined3d_ffp_emit_idx = wined3d_ffp_emit_idx_WINED3D_FFP_EMIT_FLOAT16_4; // island enum // gen:0ee679c0010a
pub const WINED3D_FFP_EMIT_FLOAT1: wined3d_ffp_emit_idx = wined3d_ffp_emit_idx_WINED3D_FFP_EMIT_FLOAT1; // island enum // gen:b62cba58e1f1
pub const WINED3D_FFP_EMIT_FLOAT2: wined3d_ffp_emit_idx = wined3d_ffp_emit_idx_WINED3D_FFP_EMIT_FLOAT2; // island enum // gen:f8afc39aeb4f
pub const WINED3D_FFP_EMIT_FLOAT3: wined3d_ffp_emit_idx = wined3d_ffp_emit_idx_WINED3D_FFP_EMIT_FLOAT3; // island enum // gen:0e171e8c0aa5
pub const WINED3D_FFP_EMIT_FLOAT4: wined3d_ffp_emit_idx = wined3d_ffp_emit_idx_WINED3D_FFP_EMIT_FLOAT4; // island enum // gen:a126595ccde2
pub const WINED3D_FFP_EMIT_INVALID: wined3d_ffp_emit_idx = wined3d_ffp_emit_idx_WINED3D_FFP_EMIT_INVALID; // island enum // gen:fda7f059956c
pub const WINED3D_FFP_EMIT_SHORT2: wined3d_ffp_emit_idx = wined3d_ffp_emit_idx_WINED3D_FFP_EMIT_SHORT2; // island enum // gen:5a88fde01b23
pub const WINED3D_FFP_EMIT_SHORT2N: wined3d_ffp_emit_idx = wined3d_ffp_emit_idx_WINED3D_FFP_EMIT_SHORT2N; // island enum // gen:c12c46395c7b
pub const WINED3D_FFP_EMIT_SHORT4: wined3d_ffp_emit_idx = wined3d_ffp_emit_idx_WINED3D_FFP_EMIT_SHORT4; // island enum // gen:14e1df56181c
pub const WINED3D_FFP_EMIT_SHORT4N: wined3d_ffp_emit_idx = wined3d_ffp_emit_idx_WINED3D_FFP_EMIT_SHORT4N; // island enum // gen:effe60042208
pub const WINED3D_FFP_EMIT_UBYTE4: wined3d_ffp_emit_idx = wined3d_ffp_emit_idx_WINED3D_FFP_EMIT_UBYTE4; // island enum // gen:f56f2f1d0aa0
pub const WINED3D_FFP_EMIT_UBYTE4N: wined3d_ffp_emit_idx = wined3d_ffp_emit_idx_WINED3D_FFP_EMIT_UBYTE4N; // island enum // gen:1c6d77e52365
pub const WINED3D_FFP_EMIT_UDEC3: wined3d_ffp_emit_idx = wined3d_ffp_emit_idx_WINED3D_FFP_EMIT_UDEC3; // island enum // gen:6e6b46e410c0
pub const WINED3D_FFP_EMIT_USHORT2N: wined3d_ffp_emit_idx = wined3d_ffp_emit_idx_WINED3D_FFP_EMIT_USHORT2N; // island enum // gen:d8eb4da16c85
pub const WINED3D_FFP_EMIT_USHORT4N: wined3d_ffp_emit_idx = wined3d_ffp_emit_idx_WINED3D_FFP_EMIT_USHORT4N; // island enum // gen:52ba465ebcea
pub const WINED3D_FFP_NORMAL: wined3d_ffp_idx = wined3d_ffp_idx_WINED3D_FFP_NORMAL; // island enum // gen:e56da4fc3133
pub const WINED3D_FFP_POSITION: wined3d_ffp_idx = wined3d_ffp_idx_WINED3D_FFP_POSITION; // island enum // gen:ce05f71375a2
pub const WINED3D_FFP_PSIZE: wined3d_ffp_idx = wined3d_ffp_idx_WINED3D_FFP_PSIZE; // island enum // gen:fd534483e319
pub const WINED3D_FFP_PS_FOG_EXP2: wined3d_ffp_ps_fog_mode = wined3d_ffp_ps_fog_mode_WINED3D_FFP_PS_FOG_EXP2; // island enum // gen:9ace5fb85045
pub const WINED3D_FFP_PS_FOG_EXP: wined3d_ffp_ps_fog_mode = wined3d_ffp_ps_fog_mode_WINED3D_FFP_PS_FOG_EXP; // island enum // gen:8a8734066ea0
pub const WINED3D_FFP_PS_FOG_LINEAR: wined3d_ffp_ps_fog_mode = wined3d_ffp_ps_fog_mode_WINED3D_FFP_PS_FOG_LINEAR; // island enum // gen:c2c29c53fbb4
pub const WINED3D_FFP_PS_FOG_OFF: wined3d_ffp_ps_fog_mode = wined3d_ffp_ps_fog_mode_WINED3D_FFP_PS_FOG_OFF; // island enum // gen:a2b9e41c35e4
pub const WINED3D_FFP_SPECULAR: wined3d_ffp_idx = wined3d_ffp_idx_WINED3D_FFP_SPECULAR; // island enum // gen:980be9618797
pub const WINED3D_FFP_TEXCOORD0: wined3d_ffp_idx = wined3d_ffp_idx_WINED3D_FFP_TEXCOORD0; // island enum // gen:e25bbdb59f70
pub const WINED3D_FFP_VS_FOG_DEPTH: wined3d_ffp_vs_fog_mode = wined3d_ffp_vs_fog_mode_WINED3D_FFP_VS_FOG_DEPTH; // island enum // gen:ab84bfa276c0
pub const WINED3D_FFP_VS_FOG_FOGCOORD: wined3d_ffp_vs_fog_mode = wined3d_ffp_vs_fog_mode_WINED3D_FFP_VS_FOG_FOGCOORD; // island enum // gen:04b8bd2d72e3
pub const WINED3D_FFP_VS_FOG_OFF: wined3d_ffp_vs_fog_mode = wined3d_ffp_vs_fog_mode_WINED3D_FFP_VS_FOG_OFF; // island enum // gen:498b9113f564
pub const WINED3D_FFP_VS_FOG_RANGE: wined3d_ffp_vs_fog_mode = wined3d_ffp_vs_fog_mode_WINED3D_FFP_VS_FOG_RANGE; // island enum // gen:5faa0db42770
pub const WINED3D_FILL_POINT: wined3d_fill_mode = wined3d_fill_mode_WINED3D_FILL_POINT; // island enum // gen:7512f9464993
pub const WINED3D_FILL_SOLID: wined3d_fill_mode = wined3d_fill_mode_WINED3D_FILL_SOLID; // island enum // gen:0f11b1034c59
pub const WINED3D_FILL_WIREFRAME: wined3d_fill_mode = wined3d_fill_mode_WINED3D_FILL_WIREFRAME; // island enum // gen:023ca7f552d2
pub const WINED3D_FOG_EXP2: wined3d_fog_mode = wined3d_fog_mode_WINED3D_FOG_EXP2; // island enum // gen:2aa830651d26
pub const WINED3D_FOG_EXP: wined3d_fog_mode = wined3d_fog_mode_WINED3D_FOG_EXP; // island enum // gen:0e3fa5dc6335
pub const WINED3D_FOG_LINEAR: wined3d_fog_mode = wined3d_fog_mode_WINED3D_FOG_LINEAR; // island enum // gen:190fe00e4353
pub const WINED3D_FOG_NONE: wined3d_fog_mode = wined3d_fog_mode_WINED3D_FOG_NONE; // island enum // gen:9e4d582dafff
pub const WINED3D_GLSL_130: wined3d_gl_extension = wined3d_gl_extension_WINED3D_GLSL_130; // island enum // gen:92432430cb23
pub const WINED3D_GL_BLEND_EQUATION: wined3d_gl_extension = wined3d_gl_extension_WINED3D_GL_BLEND_EQUATION; // island enum // gen:bff1ec9b391e
pub const WINED3D_GL_EXT_NONE: wined3d_gl_extension = wined3d_gl_extension_WINED3D_GL_EXT_NONE; // island enum // gen:883822af9969
pub const WINED3D_GL_LEGACY_CONTEXT: wined3d_gl_extension = wined3d_gl_extension_WINED3D_GL_LEGACY_CONTEXT; // island enum // gen:6e64356fd11a
pub const WINED3D_GL_NORMALIZED_TEXRECT: wined3d_gl_extension = wined3d_gl_extension_WINED3D_GL_NORMALIZED_TEXRECT; // island enum // gen:ad6ec35200fc
pub const WINED3D_GL_PRIMITIVE_QUERY: wined3d_gl_extension = wined3d_gl_extension_WINED3D_GL_PRIMITIVE_QUERY; // island enum // gen:f1b008bb269a
pub const WINED3D_GL_RES_TYPE_BUFFER: wined3d_gl_resource_type = wined3d_gl_resource_type_WINED3D_GL_RES_TYPE_BUFFER; // island enum // gen:72506ce4b572
pub const WINED3D_GL_RES_TYPE_COUNT: wined3d_gl_resource_type = wined3d_gl_resource_type_WINED3D_GL_RES_TYPE_COUNT; // island enum // gen:90b87e3fbd15
pub const WINED3D_GL_RES_TYPE_RB: wined3d_gl_resource_type = wined3d_gl_resource_type_WINED3D_GL_RES_TYPE_RB; // island enum // gen:7f401e7f36d1
pub const WINED3D_GL_RES_TYPE_TEX_1D: wined3d_gl_resource_type = wined3d_gl_resource_type_WINED3D_GL_RES_TYPE_TEX_1D; // island enum // gen:f5f9806d56d5
pub const WINED3D_GL_RES_TYPE_TEX_2D: wined3d_gl_resource_type = wined3d_gl_resource_type_WINED3D_GL_RES_TYPE_TEX_2D; // island enum // gen:211abd5d6c29
pub const WINED3D_GL_RES_TYPE_TEX_3D: wined3d_gl_resource_type = wined3d_gl_resource_type_WINED3D_GL_RES_TYPE_TEX_3D; // island enum // gen:8402bc781166
pub const WINED3D_GL_RES_TYPE_TEX_CUBE: wined3d_gl_resource_type = wined3d_gl_resource_type_WINED3D_GL_RES_TYPE_TEX_CUBE; // island enum // gen:d2b70d4b95ca
pub const WINED3D_GL_VERSION_2_0: wined3d_gl_extension = wined3d_gl_extension_WINED3D_GL_VERSION_2_0; // island enum // gen:519c8a002e67
pub const WINED3D_GL_VERSION_3_2: wined3d_gl_extension = wined3d_gl_extension_WINED3D_GL_VERSION_3_2; // island enum // gen:8f1de1bda545
pub const WINED3D_IMMCONST_SCALAR: wined3d_immconst_type = wined3d_immconst_type_WINED3D_IMMCONST_SCALAR; // island enum // gen:34c9534b37e4
pub const WINED3D_IMMCONST_VEC4: wined3d_immconst_type = wined3d_immconst_type_WINED3D_IMMCONST_VEC4; // island enum // gen:4696f47597f8
pub const WINED3D_INPUT_PER_INSTANCE_DATA: wined3d_input_classification = wined3d_input_classification_WINED3D_INPUT_PER_INSTANCE_DATA; // island enum // gen:1920f090b45d
pub const WINED3D_INPUT_PER_VERTEX_DATA: wined3d_input_classification = wined3d_input_classification_WINED3D_INPUT_PER_VERTEX_DATA; // island enum // gen:e87a81739d3a
pub const WINED3D_LIGHT_DIRECTIONAL: wined3d_light_type = wined3d_light_type_WINED3D_LIGHT_DIRECTIONAL; // island enum // gen:056bb5fb55a6
pub const WINED3D_LIGHT_GLSPOT: wined3d_light_type = wined3d_light_type_WINED3D_LIGHT_GLSPOT; // island enum // gen:cbbac52de0d0
pub const WINED3D_LIGHT_PARALLELPOINT: wined3d_light_type = wined3d_light_type_WINED3D_LIGHT_PARALLELPOINT; // island enum // gen:572ecadfefe9
pub const WINED3D_LIGHT_POINT: wined3d_light_type = wined3d_light_type_WINED3D_LIGHT_POINT; // island enum // gen:bc484de09386
pub const WINED3D_LIGHT_SPOT: wined3d_light_type = wined3d_light_type_WINED3D_LIGHT_SPOT; // island enum // gen:10301fffe718
pub const WINED3D_MCS_COLOR1: wined3d_material_color_source = wined3d_material_color_source_WINED3D_MCS_COLOR1; // island enum // gen:503272aa60a8
pub const WINED3D_MCS_COLOR2: wined3d_material_color_source = wined3d_material_color_source_WINED3D_MCS_COLOR2; // island enum // gen:3a698c023731
pub const WINED3D_MCS_MATERIAL: wined3d_material_color_source = wined3d_material_color_source_WINED3D_MCS_MATERIAL; // island enum // gen:d5bc773cf160
pub const WINED3D_MEMORY_SEGMENT_GROUP_LOCAL: wined3d_memory_segment_group = wined3d_memory_segment_group_WINED3D_MEMORY_SEGMENT_GROUP_LOCAL; // island enum // gen:64b89e767f34
pub const WINED3D_MEMORY_SEGMENT_GROUP_NON_LOCAL: wined3d_memory_segment_group = wined3d_memory_segment_group_WINED3D_MEMORY_SEGMENT_GROUP_NON_LOCAL; // island enum // gen:56587b8ae8c6
pub const WINED3D_MULTISAMPLE_16_SAMPLES: wined3d_multisample_type = wined3d_multisample_type_WINED3D_MULTISAMPLE_16_SAMPLES; // island enum // gen:d883788d04d9
pub const WINED3D_MULTISAMPLE_NONE: wined3d_multisample_type = wined3d_multisample_type_WINED3D_MULTISAMPLE_NONE; // island enum // gen:bfa66121fcaa
pub const WINED3D_MULTISAMPLE_NON_MASKABLE: wined3d_multisample_type = wined3d_multisample_type_WINED3D_MULTISAMPLE_NON_MASKABLE; // island enum // gen:d5b0152c15a3
pub const WINED3D_PATCH_EDGE_DISCRETE: wined3d_patch_edge_style = wined3d_patch_edge_style_WINED3D_PATCH_EDGE_DISCRETE; // island enum // gen:6faedae3e4e4
pub const WINED3D_PIPELINE_COMPUTE: wined3d_pipeline = wined3d_pipeline_WINED3D_PIPELINE_COMPUTE; // island enum // gen:571544baaccf
pub const WINED3D_PIPELINE_COUNT: wined3d_pipeline = wined3d_pipeline_WINED3D_PIPELINE_COUNT; // island enum // gen:b4311982b30c
pub const WINED3D_PIPELINE_GRAPHICS: wined3d_pipeline = wined3d_pipeline_WINED3D_PIPELINE_GRAPHICS; // island enum // gen:72e2eefe6f7c
pub const WINED3D_PROJECTION_COUNT3: wined3d_projection_type = wined3d_projection_type_WINED3D_PROJECTION_COUNT3; // island enum // gen:cd7671a74065
pub const WINED3D_PROJECTION_COUNT4: wined3d_projection_type = wined3d_projection_type_WINED3D_PROJECTION_COUNT4; // island enum // gen:88865c2bb52b
pub const WINED3D_PROJECTION_NONE: wined3d_projection_type = wined3d_projection_type_WINED3D_PROJECTION_NONE; // island enum // gen:79bece63975f
pub const WINED3D_PT_LINELIST: wined3d_primitive_type = wined3d_primitive_type_WINED3D_PT_LINELIST; // island enum // gen:7ae764f5dcb1
pub const WINED3D_PT_LINELIST_ADJ: wined3d_primitive_type = wined3d_primitive_type_WINED3D_PT_LINELIST_ADJ; // island enum // gen:cc690c225e89
pub const WINED3D_PT_LINESTRIP: wined3d_primitive_type = wined3d_primitive_type_WINED3D_PT_LINESTRIP; // island enum // gen:66e7ee144dd2
pub const WINED3D_PT_LINESTRIP_ADJ: wined3d_primitive_type = wined3d_primitive_type_WINED3D_PT_LINESTRIP_ADJ; // island enum // gen:335c6cecfed9
pub const WINED3D_PT_PATCH: wined3d_primitive_type = wined3d_primitive_type_WINED3D_PT_PATCH; // island enum // gen:8b7ce5fba4cc
pub const WINED3D_PT_POINTLIST: wined3d_primitive_type = wined3d_primitive_type_WINED3D_PT_POINTLIST; // island enum // gen:e83778ace339
pub const WINED3D_PT_TRIANGLEFAN: wined3d_primitive_type = wined3d_primitive_type_WINED3D_PT_TRIANGLEFAN; // island enum // gen:2fc49512114a
pub const WINED3D_PT_TRIANGLELIST: wined3d_primitive_type = wined3d_primitive_type_WINED3D_PT_TRIANGLELIST; // island enum // gen:ca4ffe7bc6b4
pub const WINED3D_PT_TRIANGLELIST_ADJ: wined3d_primitive_type = wined3d_primitive_type_WINED3D_PT_TRIANGLELIST_ADJ; // island enum // gen:03143a1fec9e
pub const WINED3D_PT_TRIANGLESTRIP: wined3d_primitive_type = wined3d_primitive_type_WINED3D_PT_TRIANGLESTRIP; // island enum // gen:93c16c717508
pub const WINED3D_PT_TRIANGLESTRIP_ADJ: wined3d_primitive_type = wined3d_primitive_type_WINED3D_PT_TRIANGLESTRIP_ADJ; // island enum // gen:74ef75efd20b
pub const WINED3D_PT_UNDEFINED: wined3d_primitive_type = wined3d_primitive_type_WINED3D_PT_UNDEFINED; // island enum // gen:dbbcaabc5dd5
pub const WINED3D_PUSH_CONSTANTS_COUNT: wined3d_push_constants = wined3d_push_constants_WINED3D_PUSH_CONSTANTS_COUNT; // island enum // gen:e25de6ff4b1b
pub const WINED3D_PUSH_CONSTANTS_PS_B: wined3d_push_constants = wined3d_push_constants_WINED3D_PUSH_CONSTANTS_PS_B; // island enum // gen:fc55b44460c4
pub const WINED3D_PUSH_CONSTANTS_PS_F: wined3d_push_constants = wined3d_push_constants_WINED3D_PUSH_CONSTANTS_PS_F; // island enum // gen:205a540a9b9a
pub const WINED3D_PUSH_CONSTANTS_PS_FFP: wined3d_push_constants = wined3d_push_constants_WINED3D_PUSH_CONSTANTS_PS_FFP; // island enum // gen:68a3c801ff71
pub const WINED3D_PUSH_CONSTANTS_PS_I: wined3d_push_constants = wined3d_push_constants_WINED3D_PUSH_CONSTANTS_PS_I; // island enum // gen:33b4514ce0bd
pub const WINED3D_PUSH_CONSTANTS_VS_B: wined3d_push_constants = wined3d_push_constants_WINED3D_PUSH_CONSTANTS_VS_B; // island enum // gen:8570c5f09e1b
pub const WINED3D_PUSH_CONSTANTS_VS_F: wined3d_push_constants = wined3d_push_constants_WINED3D_PUSH_CONSTANTS_VS_F; // island enum // gen:e620976d26c2
pub const WINED3D_PUSH_CONSTANTS_VS_FFP: wined3d_push_constants = wined3d_push_constants_WINED3D_PUSH_CONSTANTS_VS_FFP; // island enum // gen:53d7b2219268
pub const WINED3D_PUSH_CONSTANTS_VS_I: wined3d_push_constants = wined3d_push_constants_WINED3D_PUSH_CONSTANTS_VS_I; // island enum // gen:d541d1b73f30
pub const WINED3D_QUERY_TYPE_EVENT: wined3d_query_type = wined3d_query_type_WINED3D_QUERY_TYPE_EVENT; // island enum // gen:3b792b535823
pub const WINED3D_QUERY_TYPE_OCCLUSION: wined3d_query_type = wined3d_query_type_WINED3D_QUERY_TYPE_OCCLUSION; // island enum // gen:1bbf3b56073a
pub const WINED3D_QUERY_TYPE_PIPELINE_STATISTICS: wined3d_query_type = wined3d_query_type_WINED3D_QUERY_TYPE_PIPELINE_STATISTICS; // island enum // gen:7b1d867fb2db
pub const WINED3D_QUERY_TYPE_SO_STATISTICS: wined3d_query_type = wined3d_query_type_WINED3D_QUERY_TYPE_SO_STATISTICS; // island enum // gen:2096f44fd3c0
pub const WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM0: wined3d_query_type = wined3d_query_type_WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM0; // island enum // gen:b55dc6b3f5b1
pub const WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM1: wined3d_query_type = wined3d_query_type_WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM1; // island enum // gen:ae18517b197f
pub const WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM2: wined3d_query_type = wined3d_query_type_WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM2; // island enum // gen:ab3e87ab6a02
pub const WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM3: wined3d_query_type = wined3d_query_type_WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM3; // island enum // gen:ad8e318ced78
pub const WINED3D_QUERY_TYPE_TIMESTAMP: wined3d_query_type = wined3d_query_type_WINED3D_QUERY_TYPE_TIMESTAMP; // island enum // gen:d7facb982406
pub const WINED3D_QUERY_TYPE_TIMESTAMP_DISJOINT: wined3d_query_type = wined3d_query_type_WINED3D_QUERY_TYPE_TIMESTAMP_DISJOINT; // island enum // gen:4591a4c62356
pub const WINED3D_QUERY_TYPE_TIMESTAMP_FREQ: wined3d_query_type = wined3d_query_type_WINED3D_QUERY_TYPE_TIMESTAMP_FREQ; // island enum // gen:ca2f72eda971
pub const WINED3D_RENDERER_AUTO: wined3d_renderer = wined3d_renderer_WINED3D_RENDERER_AUTO; // island enum // gen:bf9b2f90a1d5
pub const WINED3D_RENDERER_NO3D: wined3d_renderer = wined3d_renderer_WINED3D_RENDERER_NO3D; // island enum // gen:fa9ad950e794
pub const WINED3D_RENDERER_OPENGL: wined3d_renderer = wined3d_renderer_WINED3D_RENDERER_OPENGL; // island enum // gen:a2c4cdeb119b
pub const WINED3D_RENDERER_VULKAN: wined3d_renderer = wined3d_renderer_WINED3D_RENDERER_VULKAN; // island enum // gen:d41d268d7b53
pub const WINED3D_RETIRED_ALLOCATOR_BLOCK_VK: wined3d_retired_object_type_vk = wined3d_retired_object_type_vk_WINED3D_RETIRED_ALLOCATOR_BLOCK_VK; // island enum // gen:873fb68c95bf
pub const WINED3D_RETIRED_BO_SLAB_SLICE_VK: wined3d_retired_object_type_vk = wined3d_retired_object_type_vk_WINED3D_RETIRED_BO_SLAB_SLICE_VK; // island enum // gen:935c213087a2
pub const WINED3D_RETIRED_BUFFER_VIEW_VK: wined3d_retired_object_type_vk = wined3d_retired_object_type_vk_WINED3D_RETIRED_BUFFER_VIEW_VK; // island enum // gen:875a23deec00
pub const WINED3D_RETIRED_BUFFER_VK: wined3d_retired_object_type_vk = wined3d_retired_object_type_vk_WINED3D_RETIRED_BUFFER_VK; // island enum // gen:d3e404007c87
pub const WINED3D_RETIRED_DESCRIPTOR_POOL_VK: wined3d_retired_object_type_vk = wined3d_retired_object_type_vk_WINED3D_RETIRED_DESCRIPTOR_POOL_VK; // island enum // gen:669d64f9b2d5
pub const WINED3D_RETIRED_EVENT_VK: wined3d_retired_object_type_vk = wined3d_retired_object_type_vk_WINED3D_RETIRED_EVENT_VK; // island enum // gen:4f049459f1e3
pub const WINED3D_RETIRED_FRAMEBUFFER_VK: wined3d_retired_object_type_vk = wined3d_retired_object_type_vk_WINED3D_RETIRED_FRAMEBUFFER_VK; // island enum // gen:20c7544afcda
pub const WINED3D_RETIRED_FREE_VK: wined3d_retired_object_type_vk = wined3d_retired_object_type_vk_WINED3D_RETIRED_FREE_VK; // island enum // gen:e2010146c3b1
pub const WINED3D_RETIRED_IMAGE_VIEW_VK: wined3d_retired_object_type_vk = wined3d_retired_object_type_vk_WINED3D_RETIRED_IMAGE_VIEW_VK; // island enum // gen:a858e97578e6
pub const WINED3D_RETIRED_IMAGE_VK: wined3d_retired_object_type_vk = wined3d_retired_object_type_vk_WINED3D_RETIRED_IMAGE_VK; // island enum // gen:fa1b827aa4fa
pub const WINED3D_RETIRED_MEMORY_VK: wined3d_retired_object_type_vk = wined3d_retired_object_type_vk_WINED3D_RETIRED_MEMORY_VK; // island enum // gen:f86b3d299580
pub const WINED3D_RETIRED_PIPELINE_VK: wined3d_retired_object_type_vk = wined3d_retired_object_type_vk_WINED3D_RETIRED_PIPELINE_VK; // island enum // gen:dc33466ff67d
pub const WINED3D_RETIRED_QUERY_POOL_VK: wined3d_retired_object_type_vk = wined3d_retired_object_type_vk_WINED3D_RETIRED_QUERY_POOL_VK; // island enum // gen:df43bedcbd9f
pub const WINED3D_RETIRED_SAMPLER_VK: wined3d_retired_object_type_vk = wined3d_retired_object_type_vk_WINED3D_RETIRED_SAMPLER_VK; // island enum // gen:68d000c5b2b6
pub const WINED3D_RS_ADAPTIVETESS_W: wined3d_render_state = wined3d_render_state_WINED3D_RS_ADAPTIVETESS_W; // island enum // gen:4f28b426ee2f
pub const WINED3D_RS_ADAPTIVETESS_X: wined3d_render_state = wined3d_render_state_WINED3D_RS_ADAPTIVETESS_X; // island enum // gen:47f323f31899
pub const WINED3D_RS_ADAPTIVETESS_Y: wined3d_render_state = wined3d_render_state_WINED3D_RS_ADAPTIVETESS_Y; // island enum // gen:6e290927776a
pub const WINED3D_RS_ADAPTIVETESS_Z: wined3d_render_state = wined3d_render_state_WINED3D_RS_ADAPTIVETESS_Z; // island enum // gen:3f5e759cde5c
pub const WINED3D_RS_ALPHABLENDENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_ALPHABLENDENABLE; // island enum // gen:d264ef4d7156
pub const WINED3D_RS_ALPHAFUNC: wined3d_render_state = wined3d_render_state_WINED3D_RS_ALPHAFUNC; // island enum // gen:638a0eb30fdc
pub const WINED3D_RS_ALPHAREF: wined3d_render_state = wined3d_render_state_WINED3D_RS_ALPHAREF; // island enum // gen:620f42cf74da
pub const WINED3D_RS_ALPHATESTENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_ALPHATESTENABLE; // island enum // gen:db84c936388b
pub const WINED3D_RS_AMBIENT: wined3d_render_state = wined3d_render_state_WINED3D_RS_AMBIENT; // island enum // gen:5b3cad508e22
pub const WINED3D_RS_AMBIENTMATERIALSOURCE: wined3d_render_state = wined3d_render_state_WINED3D_RS_AMBIENTMATERIALSOURCE; // island enum // gen:ae347db723c9
pub const WINED3D_RS_ANISOTROPY: wined3d_render_state = wined3d_render_state_WINED3D_RS_ANISOTROPY; // island enum // gen:feb2d77c3ce7
pub const WINED3D_RS_ANTIALIAS: wined3d_render_state = wined3d_render_state_WINED3D_RS_ANTIALIAS; // island enum // gen:1bfd9a22ec2c
pub const WINED3D_RS_ANTIALIASEDLINEENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_ANTIALIASEDLINEENABLE; // island enum // gen:a1d64f8207d7
pub const WINED3D_RS_BACK_STENCILFAIL: wined3d_render_state = wined3d_render_state_WINED3D_RS_BACK_STENCILFAIL; // island enum // gen:b62e618171ec
pub const WINED3D_RS_BACK_STENCILFUNC: wined3d_render_state = wined3d_render_state_WINED3D_RS_BACK_STENCILFUNC; // island enum // gen:41418d169052
pub const WINED3D_RS_BACK_STENCILPASS: wined3d_render_state = wined3d_render_state_WINED3D_RS_BACK_STENCILPASS; // island enum // gen:a707e0b68597
pub const WINED3D_RS_BACK_STENCILZFAIL: wined3d_render_state = wined3d_render_state_WINED3D_RS_BACK_STENCILZFAIL; // island enum // gen:16fd34427dfd
pub const WINED3D_RS_BLENDFACTOR: wined3d_render_state = wined3d_render_state_WINED3D_RS_BLENDFACTOR; // island enum // gen:f571077194d6
pub const WINED3D_RS_BLENDOP: wined3d_render_state = wined3d_render_state_WINED3D_RS_BLENDOP; // island enum // gen:782e1fa77c1d
pub const WINED3D_RS_BLENDOPALPHA: wined3d_render_state = wined3d_render_state_WINED3D_RS_BLENDOPALPHA; // island enum // gen:1d4f2098af03
pub const WINED3D_RS_CLIPPING: wined3d_render_state = wined3d_render_state_WINED3D_RS_CLIPPING; // island enum // gen:d55448b0a48b
pub const WINED3D_RS_CLIPPLANEENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_CLIPPLANEENABLE; // island enum // gen:c77f6bf809fe
pub const WINED3D_RS_COLORKEYBLENDENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_COLORKEYBLENDENABLE; // island enum // gen:d8d4abf90553
pub const WINED3D_RS_COLORKEYENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_COLORKEYENABLE; // island enum // gen:c0fd7831337b
pub const WINED3D_RS_COLORVERTEX: wined3d_render_state = wined3d_render_state_WINED3D_RS_COLORVERTEX; // island enum // gen:f4520ac0178b
pub const WINED3D_RS_COLORWRITEENABLE1: wined3d_render_state = wined3d_render_state_WINED3D_RS_COLORWRITEENABLE1; // island enum // gen:b9dd5ab3d633
pub const WINED3D_RS_COLORWRITEENABLE2: wined3d_render_state = wined3d_render_state_WINED3D_RS_COLORWRITEENABLE2; // island enum // gen:ccb1b8e28968
pub const WINED3D_RS_COLORWRITEENABLE3: wined3d_render_state = wined3d_render_state_WINED3D_RS_COLORWRITEENABLE3; // island enum // gen:bb57fbc41fcc
pub const WINED3D_RS_COLORWRITEENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_COLORWRITEENABLE; // island enum // gen:d241871b790f
pub const WINED3D_RS_CULLMODE: wined3d_render_state = wined3d_render_state_WINED3D_RS_CULLMODE; // island enum // gen:0772814bb087
pub const WINED3D_RS_DEBUGMONITORTOKEN: wined3d_render_state = wined3d_render_state_WINED3D_RS_DEBUGMONITORTOKEN; // island enum // gen:5cac78a1f7a3
pub const WINED3D_RS_DEPTHBIAS: wined3d_render_state = wined3d_render_state_WINED3D_RS_DEPTHBIAS; // island enum // gen:b185bd45eb78
pub const WINED3D_RS_DESTBLEND: wined3d_render_state = wined3d_render_state_WINED3D_RS_DESTBLEND; // island enum // gen:c57d7706c70c
pub const WINED3D_RS_DESTBLENDALPHA: wined3d_render_state = wined3d_render_state_WINED3D_RS_DESTBLENDALPHA; // island enum // gen:66a3ecdce6d2
pub const WINED3D_RS_DIFFUSEMATERIALSOURCE: wined3d_render_state = wined3d_render_state_WINED3D_RS_DIFFUSEMATERIALSOURCE; // island enum // gen:c108fe38d780
pub const WINED3D_RS_DITHERENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_DITHERENABLE; // island enum // gen:54187ee0c7ef
pub const WINED3D_RS_EMISSIVEMATERIALSOURCE: wined3d_render_state = wined3d_render_state_WINED3D_RS_EMISSIVEMATERIALSOURCE; // island enum // gen:927572b1c62f
pub const WINED3D_RS_ENABLEADAPTIVETESSELLATION: wined3d_render_state = wined3d_render_state_WINED3D_RS_ENABLEADAPTIVETESSELLATION; // island enum // gen:94357a972d60
pub const WINED3D_RS_EXTENTS: wined3d_render_state = wined3d_render_state_WINED3D_RS_EXTENTS; // island enum // gen:ff56d16770c5
pub const WINED3D_RS_FILLMODE: wined3d_render_state = wined3d_render_state_WINED3D_RS_FILLMODE; // island enum // gen:bbf9e25e30c4
pub const WINED3D_RS_FLUSHBATCH: wined3d_render_state = wined3d_render_state_WINED3D_RS_FLUSHBATCH; // island enum // gen:e67abae3d7c1
pub const WINED3D_RS_FOGCOLOR: wined3d_render_state = wined3d_render_state_WINED3D_RS_FOGCOLOR; // island enum // gen:e9778a3e7772
pub const WINED3D_RS_FOGDENSITY: wined3d_render_state = wined3d_render_state_WINED3D_RS_FOGDENSITY; // island enum // gen:c97c0009dce7
pub const WINED3D_RS_FOGENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_FOGENABLE; // island enum // gen:9db13e8548f2
pub const WINED3D_RS_FOGEND: wined3d_render_state = wined3d_render_state_WINED3D_RS_FOGEND; // island enum // gen:a3c81e702049
pub const WINED3D_RS_FOGSTART: wined3d_render_state = wined3d_render_state_WINED3D_RS_FOGSTART; // island enum // gen:95e3e76386b1
pub const WINED3D_RS_FOGTABLEMODE: wined3d_render_state = wined3d_render_state_WINED3D_RS_FOGTABLEMODE; // island enum // gen:45f7aee8cbb4
pub const WINED3D_RS_FOGVERTEXMODE: wined3d_render_state = wined3d_render_state_WINED3D_RS_FOGVERTEXMODE; // island enum // gen:0c3e37c56dc5
pub const WINED3D_RS_INDEXEDVERTEXBLENDENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_INDEXEDVERTEXBLENDENABLE; // island enum // gen:1e59903fc96a
pub const WINED3D_RS_LASTPIXEL: wined3d_render_state = wined3d_render_state_WINED3D_RS_LASTPIXEL; // island enum // gen:e485bc86ea1f
pub const WINED3D_RS_LIGHTING: wined3d_render_state = wined3d_render_state_WINED3D_RS_LIGHTING; // island enum // gen:16927db0b497
pub const WINED3D_RS_LINEPATTERN: wined3d_render_state = wined3d_render_state_WINED3D_RS_LINEPATTERN; // island enum // gen:b78428d277f5
pub const WINED3D_RS_LOCALVIEWER: wined3d_render_state = wined3d_render_state_WINED3D_RS_LOCALVIEWER; // island enum // gen:e1e9933d88ea
pub const WINED3D_RS_MAXTESSELLATIONLEVEL: wined3d_render_state = wined3d_render_state_WINED3D_RS_MAXTESSELLATIONLEVEL; // island enum // gen:bb7c38c321d6
pub const WINED3D_RS_MINTESSELLATIONLEVEL: wined3d_render_state = wined3d_render_state_WINED3D_RS_MINTESSELLATIONLEVEL; // island enum // gen:513c9d6de852
pub const WINED3D_RS_MIPMAPLODBIAS: wined3d_render_state = wined3d_render_state_WINED3D_RS_MIPMAPLODBIAS; // island enum // gen:700ac1af836b
pub const WINED3D_RS_MONOENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_MONOENABLE; // island enum // gen:0ae16da889a0
pub const WINED3D_RS_MULTISAMPLEANTIALIAS: wined3d_render_state = wined3d_render_state_WINED3D_RS_MULTISAMPLEANTIALIAS; // island enum // gen:f83bfecd8836
pub const WINED3D_RS_MULTISAMPLEMASK: wined3d_render_state = wined3d_render_state_WINED3D_RS_MULTISAMPLEMASK; // island enum // gen:08acebdd87f5
pub const WINED3D_RS_NORMALDEGREE: wined3d_render_state = wined3d_render_state_WINED3D_RS_NORMALDEGREE; // island enum // gen:14fca8ff27c8
pub const WINED3D_RS_NORMALIZENORMALS: wined3d_render_state = wined3d_render_state_WINED3D_RS_NORMALIZENORMALS; // island enum // gen:a4be87cb153a
pub const WINED3D_RS_PATCHEDGESTYLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_PATCHEDGESTYLE; // island enum // gen:976577d6da82
pub const WINED3D_RS_PATCHSEGMENTS: wined3d_render_state = wined3d_render_state_WINED3D_RS_PATCHSEGMENTS; // island enum // gen:aade16252964
pub const WINED3D_RS_PLANEMASK: wined3d_render_state = wined3d_render_state_WINED3D_RS_PLANEMASK; // island enum // gen:a0bc9e4ce49e
pub const WINED3D_RS_POINTSCALEENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_POINTSCALEENABLE; // island enum // gen:45c232812dba
pub const WINED3D_RS_POINTSCALE_A: wined3d_render_state = wined3d_render_state_WINED3D_RS_POINTSCALE_A; // island enum // gen:a9c88dda4d95
pub const WINED3D_RS_POINTSCALE_B: wined3d_render_state = wined3d_render_state_WINED3D_RS_POINTSCALE_B; // island enum // gen:50f3427c493a
pub const WINED3D_RS_POINTSCALE_C: wined3d_render_state = wined3d_render_state_WINED3D_RS_POINTSCALE_C; // island enum // gen:2173f244b8fd
pub const WINED3D_RS_POINTSIZE: wined3d_render_state = wined3d_render_state_WINED3D_RS_POINTSIZE; // island enum // gen:d1c5f6771885
pub const WINED3D_RS_POINTSIZE_MAX: wined3d_render_state = wined3d_render_state_WINED3D_RS_POINTSIZE_MAX; // island enum // gen:b683eba77aa1
pub const WINED3D_RS_POINTSIZE_MIN: wined3d_render_state = wined3d_render_state_WINED3D_RS_POINTSIZE_MIN; // island enum // gen:28fc240f2476
pub const WINED3D_RS_POINTSPRITEENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_POINTSPRITEENABLE; // island enum // gen:9309c1141862
pub const WINED3D_RS_POSITIONDEGREE: wined3d_render_state = wined3d_render_state_WINED3D_RS_POSITIONDEGREE; // island enum // gen:11f6c6a853c2
pub const WINED3D_RS_RANGEFOGENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_RANGEFOGENABLE; // island enum // gen:1dd7db583e53
pub const WINED3D_RS_ROP2: wined3d_render_state = wined3d_render_state_WINED3D_RS_ROP2; // island enum // gen:5cd3f2f38a3b
pub const WINED3D_RS_SCISSORTESTENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_SCISSORTESTENABLE; // island enum // gen:ecf983ec9569
pub const WINED3D_RS_SEPARATEALPHABLENDENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_SEPARATEALPHABLENDENABLE; // island enum // gen:3c877f1a1c27
pub const WINED3D_RS_SHADEMODE: wined3d_render_state = wined3d_render_state_WINED3D_RS_SHADEMODE; // island enum // gen:a6bc81aa1be5
pub const WINED3D_RS_SLOPESCALEDEPTHBIAS: wined3d_render_state = wined3d_render_state_WINED3D_RS_SLOPESCALEDEPTHBIAS; // island enum // gen:b1df0ee66f8e
pub const WINED3D_RS_SOFTWAREVERTEXPROCESSING: wined3d_render_state = wined3d_render_state_WINED3D_RS_SOFTWAREVERTEXPROCESSING; // island enum // gen:46f8bec26e87
pub const WINED3D_RS_SPECULARENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_SPECULARENABLE; // island enum // gen:e1b5760c7e27
pub const WINED3D_RS_SPECULARMATERIALSOURCE: wined3d_render_state = wined3d_render_state_WINED3D_RS_SPECULARMATERIALSOURCE; // island enum // gen:695fb5289732
pub const WINED3D_RS_SRCBLEND: wined3d_render_state = wined3d_render_state_WINED3D_RS_SRCBLEND; // island enum // gen:634cb5a48f44
pub const WINED3D_RS_SRCBLENDALPHA: wined3d_render_state = wined3d_render_state_WINED3D_RS_SRCBLENDALPHA; // island enum // gen:ba150eb8637d
pub const WINED3D_RS_SRGBWRITEENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_SRGBWRITEENABLE; // island enum // gen:58036660d46d
pub const WINED3D_RS_STENCILENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_STENCILENABLE; // island enum // gen:ed28afdf2cdb
pub const WINED3D_RS_STENCILFAIL: wined3d_render_state = wined3d_render_state_WINED3D_RS_STENCILFAIL; // island enum // gen:0b2351c4210b
pub const WINED3D_RS_STENCILFUNC: wined3d_render_state = wined3d_render_state_WINED3D_RS_STENCILFUNC; // island enum // gen:89112b82feeb
pub const WINED3D_RS_STENCILMASK: wined3d_render_state = wined3d_render_state_WINED3D_RS_STENCILMASK; // island enum // gen:8d6bfe189791
pub const WINED3D_RS_STENCILPASS: wined3d_render_state = wined3d_render_state_WINED3D_RS_STENCILPASS; // island enum // gen:92b0b111f3ca
pub const WINED3D_RS_STENCILREF: wined3d_render_state = wined3d_render_state_WINED3D_RS_STENCILREF; // island enum // gen:822dd330553a
pub const WINED3D_RS_STENCILWRITEMASK: wined3d_render_state = wined3d_render_state_WINED3D_RS_STENCILWRITEMASK; // island enum // gen:295b498e1eb6
pub const WINED3D_RS_STENCILZFAIL: wined3d_render_state = wined3d_render_state_WINED3D_RS_STENCILZFAIL; // island enum // gen:0ae3059ebe3f
pub const WINED3D_RS_STIPPLEDALPHA: wined3d_render_state = wined3d_render_state_WINED3D_RS_STIPPLEDALPHA; // island enum // gen:c9b17bc0634b
pub const WINED3D_RS_STIPPLEENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_STIPPLEENABLE; // island enum // gen:ff05eaa8bd72
pub const WINED3D_RS_SUBPIXEL: wined3d_render_state = wined3d_render_state_WINED3D_RS_SUBPIXEL; // island enum // gen:b3d5abb72b32
pub const WINED3D_RS_SUBPIXELX: wined3d_render_state = wined3d_render_state_WINED3D_RS_SUBPIXELX; // island enum // gen:18c4c4bb2c42
pub const WINED3D_RS_TEXTUREFACTOR: wined3d_render_state = wined3d_render_state_WINED3D_RS_TEXTUREFACTOR; // island enum // gen:3c70b340c1e0
pub const WINED3D_RS_TEXTUREPERSPECTIVE: wined3d_render_state = wined3d_render_state_WINED3D_RS_TEXTUREPERSPECTIVE; // island enum // gen:83344b8f0ac9
pub const WINED3D_RS_TRANSLUCENTSORTINDEPENDENT: wined3d_render_state = wined3d_render_state_WINED3D_RS_TRANSLUCENTSORTINDEPENDENT; // island enum // gen:b2c999734ac1
pub const WINED3D_RS_TWEENFACTOR: wined3d_render_state = wined3d_render_state_WINED3D_RS_TWEENFACTOR; // island enum // gen:e7322c9e69b1
pub const WINED3D_RS_TWOSIDEDSTENCILMODE: wined3d_render_state = wined3d_render_state_WINED3D_RS_TWOSIDEDSTENCILMODE; // island enum // gen:6b0530fd6ff3
pub const WINED3D_RS_VERTEXBLEND: wined3d_render_state = wined3d_render_state_WINED3D_RS_VERTEXBLEND; // island enum // gen:4e84a4f35a92
pub const WINED3D_RS_WRAP0: wined3d_render_state = wined3d_render_state_WINED3D_RS_WRAP0; // island enum // gen:7d3531deecfa
pub const WINED3D_RS_WRAP10: wined3d_render_state = wined3d_render_state_WINED3D_RS_WRAP10; // island enum // gen:a1d71c04f408
pub const WINED3D_RS_WRAP11: wined3d_render_state = wined3d_render_state_WINED3D_RS_WRAP11; // island enum // gen:d3891fb449dd
pub const WINED3D_RS_WRAP12: wined3d_render_state = wined3d_render_state_WINED3D_RS_WRAP12; // island enum // gen:a30c3877a4cd
pub const WINED3D_RS_WRAP13: wined3d_render_state = wined3d_render_state_WINED3D_RS_WRAP13; // island enum // gen:2084299bd469
pub const WINED3D_RS_WRAP14: wined3d_render_state = wined3d_render_state_WINED3D_RS_WRAP14; // island enum // gen:f68d7f51139a
pub const WINED3D_RS_WRAP15: wined3d_render_state = wined3d_render_state_WINED3D_RS_WRAP15; // island enum // gen:55f2cb0a6165
pub const WINED3D_RS_WRAP1: wined3d_render_state = wined3d_render_state_WINED3D_RS_WRAP1; // island enum // gen:3007251f3b33
pub const WINED3D_RS_WRAP2: wined3d_render_state = wined3d_render_state_WINED3D_RS_WRAP2; // island enum // gen:65a0f9083c68
pub const WINED3D_RS_WRAP3: wined3d_render_state = wined3d_render_state_WINED3D_RS_WRAP3; // island enum // gen:d0252ea7a353
pub const WINED3D_RS_WRAP4: wined3d_render_state = wined3d_render_state_WINED3D_RS_WRAP4; // island enum // gen:09eb8e15d6d8
pub const WINED3D_RS_WRAP5: wined3d_render_state = wined3d_render_state_WINED3D_RS_WRAP5; // island enum // gen:d40c6b1fd506
pub const WINED3D_RS_WRAP6: wined3d_render_state = wined3d_render_state_WINED3D_RS_WRAP6; // island enum // gen:d5c0dece3bd8
pub const WINED3D_RS_WRAP7: wined3d_render_state = wined3d_render_state_WINED3D_RS_WRAP7; // island enum // gen:256903f0dc7a
pub const WINED3D_RS_WRAP8: wined3d_render_state = wined3d_render_state_WINED3D_RS_WRAP8; // island enum // gen:fbd5c35d0540
pub const WINED3D_RS_WRAP9: wined3d_render_state = wined3d_render_state_WINED3D_RS_WRAP9; // island enum // gen:9dd2a07919c3
pub const WINED3D_RS_WRAPU: wined3d_render_state = wined3d_render_state_WINED3D_RS_WRAPU; // island enum // gen:a743a24e0662
pub const WINED3D_RS_WRAPV: wined3d_render_state = wined3d_render_state_WINED3D_RS_WRAPV; // island enum // gen:2cd06448cb98
pub const WINED3D_RS_ZENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_ZENABLE; // island enum // gen:f8bcd5a86ca6
pub const WINED3D_RS_ZFUNC: wined3d_render_state = wined3d_render_state_WINED3D_RS_ZFUNC; // island enum // gen:bb82e0176910
pub const WINED3D_RS_ZVISIBLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_ZVISIBLE; // island enum // gen:dd4f3c338105
pub const WINED3D_RS_ZWRITEENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_ZWRITEENABLE; // island enum // gen:2a0862d11120
pub const WINED3D_RTYPE_BUFFER: wined3d_resource_type = wined3d_resource_type_WINED3D_RTYPE_BUFFER; // island enum // gen:8443940bf9c4
pub const WINED3D_RTYPE_NONE: wined3d_resource_type = wined3d_resource_type_WINED3D_RTYPE_NONE; // island enum // gen:20c459743c49
pub const WINED3D_RTYPE_TEXTURE_1D: wined3d_resource_type = wined3d_resource_type_WINED3D_RTYPE_TEXTURE_1D; // island enum // gen:cfcea9995bc2
pub const WINED3D_RTYPE_TEXTURE_2D: wined3d_resource_type = wined3d_resource_type_WINED3D_RTYPE_TEXTURE_2D; // island enum // gen:de40c3024d68
pub const WINED3D_RTYPE_TEXTURE_3D: wined3d_resource_type = wined3d_resource_type_WINED3D_RTYPE_TEXTURE_3D; // island enum // gen:7752a0d240c9
pub const WINED3D_SAMP_ADDRESS_U: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_ADDRESS_U; // island enum // gen:1edc0ba1eb48
pub const WINED3D_SAMP_ADDRESS_V: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_ADDRESS_V; // island enum // gen:863872073cda
pub const WINED3D_SAMP_ADDRESS_W: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_ADDRESS_W; // island enum // gen:24b06d72c7b8
pub const WINED3D_SAMP_BORDER_COLOR: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_BORDER_COLOR; // island enum // gen:6359d21babb4
pub const WINED3D_SAMP_DMAP_OFFSET: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_DMAP_OFFSET; // island enum // gen:f5ca5c22c64f
pub const WINED3D_SAMP_ELEMENT_INDEX: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_ELEMENT_INDEX; // island enum // gen:52ddebcafb4f
pub const WINED3D_SAMP_MAG_FILTER: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_MAG_FILTER; // island enum // gen:1f1d5697ec8a
pub const WINED3D_SAMP_MAX_ANISOTROPY: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_MAX_ANISOTROPY; // island enum // gen:8f1c9604c9be
pub const WINED3D_SAMP_MAX_MIP_LEVEL: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_MAX_MIP_LEVEL; // island enum // gen:798c7969f2c4
pub const WINED3D_SAMP_MIN_FILTER: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_MIN_FILTER; // island enum // gen:e75251a959b9
pub const WINED3D_SAMP_MIPMAP_LOD_BIAS: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_MIPMAP_LOD_BIAS; // island enum // gen:099207089185
pub const WINED3D_SAMP_MIP_FILTER: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_MIP_FILTER; // island enum // gen:118fa724e031
pub const WINED3D_SAMP_SRGB_TEXTURE: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_SRGB_TEXTURE; // island enum // gen:01fba5b39da0
pub const WINED3D_SBT_ALL: wined3d_stateblock_type = wined3d_stateblock_type_WINED3D_SBT_ALL; // island enum // gen:8ff21d87a028
pub const WINED3D_SBT_PIXEL_STATE: wined3d_stateblock_type = wined3d_stateblock_type_WINED3D_SBT_PIXEL_STATE; // island enum // gen:8f142cd456ca
pub const WINED3D_SBT_PRIMARY: wined3d_stateblock_type = wined3d_stateblock_type_WINED3D_SBT_PRIMARY; // island enum // gen:a4ee8d8e4d8b
pub const WINED3D_SBT_RECORDED: wined3d_stateblock_type = wined3d_stateblock_type_WINED3D_SBT_RECORDED; // island enum // gen:a3a30ecb7bc4
pub const WINED3D_SBT_VERTEX_STATE: wined3d_stateblock_type = wined3d_stateblock_type_WINED3D_SBT_VERTEX_STATE; // island enum // gen:ffd3d162ba1f
pub const WINED3D_SCANLINE_ORDERING_INTERLACED: wined3d_scanline_ordering = wined3d_scanline_ordering_WINED3D_SCANLINE_ORDERING_INTERLACED; // island enum // gen:fc06a0d655de
pub const WINED3D_SCANLINE_ORDERING_PROGRESSIVE: wined3d_scanline_ordering = wined3d_scanline_ordering_WINED3D_SCANLINE_ORDERING_PROGRESSIVE; // island enum // gen:c1eaa2460a37
pub const WINED3D_SCANLINE_ORDERING_UNKNOWN: wined3d_scanline_ordering = wined3d_scanline_ordering_WINED3D_SCANLINE_ORDERING_UNKNOWN; // island enum // gen:7774b8a98afe
pub const WINED3D_SHADER_BACKEND_AUTO: wined3d_shader_backend = wined3d_shader_backend_WINED3D_SHADER_BACKEND_AUTO; // island enum // gen:792aaa76b91f
pub const WINED3D_SHADER_BACKEND_GLSL: wined3d_shader_backend = wined3d_shader_backend_WINED3D_SHADER_BACKEND_GLSL; // island enum // gen:b995c0c7a681
pub const WINED3D_SHADER_BACKEND_GLSL_VKD3D: wined3d_shader_backend = wined3d_shader_backend_WINED3D_SHADER_BACKEND_GLSL_VKD3D; // island enum // gen:342858412e92
pub const WINED3D_SHADER_CONDITIONAL_OP_NZ: wined3d_shader_conditional_op = wined3d_shader_conditional_op_WINED3D_SHADER_CONDITIONAL_OP_NZ; // island enum // gen:7d0895a105f4
pub const WINED3D_SHADER_CONDITIONAL_OP_Z: wined3d_shader_conditional_op = wined3d_shader_conditional_op_WINED3D_SHADER_CONDITIONAL_OP_Z; // island enum // gen:f5432a43753f
pub const WINED3D_SHADER_DESCRIPTOR_TYPE_CBV: wined3d_shader_descriptor_type = wined3d_shader_descriptor_type_WINED3D_SHADER_DESCRIPTOR_TYPE_CBV; // island enum // gen:8f44482e6a44
pub const WINED3D_SHADER_DESCRIPTOR_TYPE_SAMPLER: wined3d_shader_descriptor_type = wined3d_shader_descriptor_type_WINED3D_SHADER_DESCRIPTOR_TYPE_SAMPLER; // island enum // gen:366f9f64e5d5
pub const WINED3D_SHADER_DESCRIPTOR_TYPE_SRV: wined3d_shader_descriptor_type = wined3d_shader_descriptor_type_WINED3D_SHADER_DESCRIPTOR_TYPE_SRV; // island enum // gen:2594f0e810cf
pub const WINED3D_SHADER_DESCRIPTOR_TYPE_UAV: wined3d_shader_descriptor_type = wined3d_shader_descriptor_type_WINED3D_SHADER_DESCRIPTOR_TYPE_UAV; // island enum // gen:07b676b30888
pub const WINED3D_SHADER_DESCRIPTOR_TYPE_UAV_COUNTER: wined3d_shader_descriptor_type = wined3d_shader_descriptor_type_WINED3D_SHADER_DESCRIPTOR_TYPE_UAV_COUNTER; // island enum // gen:3e0a3f983c3d
pub const WINED3D_SHADER_REL_OP_EQ: wined3d_shader_rel_op = wined3d_shader_rel_op_WINED3D_SHADER_REL_OP_EQ; // island enum // gen:408a040b2fd5
pub const WINED3D_SHADER_REL_OP_GE: wined3d_shader_rel_op = wined3d_shader_rel_op_WINED3D_SHADER_REL_OP_GE; // island enum // gen:363c19a5e632
pub const WINED3D_SHADER_REL_OP_GT: wined3d_shader_rel_op = wined3d_shader_rel_op_WINED3D_SHADER_REL_OP_GT; // island enum // gen:1750c3f122eb
pub const WINED3D_SHADER_REL_OP_LE: wined3d_shader_rel_op = wined3d_shader_rel_op_WINED3D_SHADER_REL_OP_LE; // island enum // gen:fe887187fe1e
pub const WINED3D_SHADER_REL_OP_LT: wined3d_shader_rel_op = wined3d_shader_rel_op_WINED3D_SHADER_REL_OP_LT; // island enum // gen:619bd5fba911
pub const WINED3D_SHADER_REL_OP_NE: wined3d_shader_rel_op = wined3d_shader_rel_op_WINED3D_SHADER_REL_OP_NE; // island enum // gen:90b1abec6ac6
pub const WINED3D_SHADER_RESOURCE_BUFFER: wined3d_shader_resource_type = wined3d_shader_resource_type_WINED3D_SHADER_RESOURCE_BUFFER; // island enum // gen:329314fd4d11
pub const WINED3D_SHADER_RESOURCE_NONE: wined3d_shader_resource_type = wined3d_shader_resource_type_WINED3D_SHADER_RESOURCE_NONE; // island enum // gen:c2affffa217f
pub const WINED3D_SHADER_RESOURCE_TEXTURE_1D: wined3d_shader_resource_type = wined3d_shader_resource_type_WINED3D_SHADER_RESOURCE_TEXTURE_1D; // island enum // gen:f0e37f8ebfd3
pub const WINED3D_SHADER_RESOURCE_TEXTURE_1DARRAY: wined3d_shader_resource_type = wined3d_shader_resource_type_WINED3D_SHADER_RESOURCE_TEXTURE_1DARRAY; // island enum // gen:baf005c44907
pub const WINED3D_SHADER_RESOURCE_TEXTURE_2D: wined3d_shader_resource_type = wined3d_shader_resource_type_WINED3D_SHADER_RESOURCE_TEXTURE_2D; // island enum // gen:fc8230a96986
pub const WINED3D_SHADER_RESOURCE_TEXTURE_2DARRAY: wined3d_shader_resource_type = wined3d_shader_resource_type_WINED3D_SHADER_RESOURCE_TEXTURE_2DARRAY; // island enum // gen:7af827ef2ec0
pub const WINED3D_SHADER_RESOURCE_TEXTURE_2DMS: wined3d_shader_resource_type = wined3d_shader_resource_type_WINED3D_SHADER_RESOURCE_TEXTURE_2DMS; // island enum // gen:23f03b6799b7
pub const WINED3D_SHADER_RESOURCE_TEXTURE_2DMSARRAY: wined3d_shader_resource_type = wined3d_shader_resource_type_WINED3D_SHADER_RESOURCE_TEXTURE_2DMSARRAY; // island enum // gen:a6c373637798
pub const WINED3D_SHADER_RESOURCE_TEXTURE_3D: wined3d_shader_resource_type = wined3d_shader_resource_type_WINED3D_SHADER_RESOURCE_TEXTURE_3D; // island enum // gen:88e3b1708852
pub const WINED3D_SHADER_RESOURCE_TEXTURE_CUBE: wined3d_shader_resource_type = wined3d_shader_resource_type_WINED3D_SHADER_RESOURCE_TEXTURE_CUBE; // island enum // gen:de15d88e25cc
pub const WINED3D_SHADER_RESOURCE_TEXTURE_CUBEARRAY: wined3d_shader_resource_type = wined3d_shader_resource_type_WINED3D_SHADER_RESOURCE_TEXTURE_CUBEARRAY; // island enum // gen:65a61aac9f21
pub const WINED3D_SHADER_TEX_2D: wined3d_shader_tex_types = wined3d_shader_tex_types_WINED3D_SHADER_TEX_2D; // island enum // gen:13d8d7320759
pub const WINED3D_SHADER_TEX_3D: wined3d_shader_tex_types = wined3d_shader_tex_types_WINED3D_SHADER_TEX_3D; // island enum // gen:331b8cfe9aff
pub const WINED3D_SHADER_TEX_CUBE: wined3d_shader_tex_types = wined3d_shader_tex_types_WINED3D_SHADER_TEX_CUBE; // island enum // gen:9bd019cb3214
pub const WINED3D_SHADER_TYPE_COMPUTE: wined3d_shader_type = wined3d_shader_type_WINED3D_SHADER_TYPE_COMPUTE; // island enum // gen:e692d880f583
pub const WINED3D_SHADER_TYPE_COUNT: wined3d_shader_type = wined3d_shader_type_WINED3D_SHADER_TYPE_COUNT; // island enum // gen:fc1614ff81e7
pub const WINED3D_SHADER_TYPE_DOMAIN: wined3d_shader_type = wined3d_shader_type_WINED3D_SHADER_TYPE_DOMAIN; // island enum // gen:997d2259df90
pub const WINED3D_SHADER_TYPE_GEOMETRY: wined3d_shader_type = wined3d_shader_type_WINED3D_SHADER_TYPE_GEOMETRY; // island enum // gen:abba3d8540f6
pub const WINED3D_SHADER_TYPE_GRAPHICS_COUNT: wined3d_shader_type = wined3d_shader_type_WINED3D_SHADER_TYPE_GRAPHICS_COUNT; // island enum // gen:073d96838b8c
pub const WINED3D_SHADER_TYPE_HULL: wined3d_shader_type = wined3d_shader_type_WINED3D_SHADER_TYPE_HULL; // island enum // gen:0cfdb4c27235
pub const WINED3D_SHADER_TYPE_INVALID: wined3d_shader_type = wined3d_shader_type_WINED3D_SHADER_TYPE_INVALID; // island enum // gen:ac3a6c0ccae6
pub const WINED3D_SHADER_TYPE_PIXEL: wined3d_shader_type = wined3d_shader_type_WINED3D_SHADER_TYPE_PIXEL; // island enum // gen:262c46a9d6e6
pub const WINED3D_SHADER_TYPE_VERTEX: wined3d_shader_type = wined3d_shader_type_WINED3D_SHADER_TYPE_VERTEX; // island enum // gen:00e043d762ba
pub const WINED3D_SHADE_FLAT: wined3d_shade_mode = wined3d_shade_mode_WINED3D_SHADE_FLAT; // island enum // gen:8d9138c6ccbe
pub const WINED3D_SHADE_GOURAUD: wined3d_shade_mode = wined3d_shade_mode_WINED3D_SHADE_GOURAUD; // island enum // gen:e25d8dd70322
pub const WINED3D_SHADE_PHONG: wined3d_shade_mode = wined3d_shade_mode_WINED3D_SHADE_PHONG; // island enum // gen:0aaa726b96b1
pub const WINED3D_STENCIL_OP_DECR: wined3d_stencil_op = wined3d_stencil_op_WINED3D_STENCIL_OP_DECR; // island enum // gen:6b7043e1629b
pub const WINED3D_STENCIL_OP_DECR_SAT: wined3d_stencil_op = wined3d_stencil_op_WINED3D_STENCIL_OP_DECR_SAT; // island enum // gen:e9397726aa6c
pub const WINED3D_STENCIL_OP_INCR: wined3d_stencil_op = wined3d_stencil_op_WINED3D_STENCIL_OP_INCR; // island enum // gen:9163f9ec2807
pub const WINED3D_STENCIL_OP_INCR_SAT: wined3d_stencil_op = wined3d_stencil_op_WINED3D_STENCIL_OP_INCR_SAT; // island enum // gen:3fe494639ad1
pub const WINED3D_STENCIL_OP_INVERT: wined3d_stencil_op = wined3d_stencil_op_WINED3D_STENCIL_OP_INVERT; // island enum // gen:efda8bd01023
pub const WINED3D_STENCIL_OP_KEEP: wined3d_stencil_op = wined3d_stencil_op_WINED3D_STENCIL_OP_KEEP; // island enum // gen:365147f9acf8
pub const WINED3D_STENCIL_OP_REPLACE: wined3d_stencil_op = wined3d_stencil_op_WINED3D_STENCIL_OP_REPLACE; // island enum // gen:66b736fd5f1d
pub const WINED3D_STENCIL_OP_ZERO: wined3d_stencil_op = wined3d_stencil_op_WINED3D_STENCIL_OP_ZERO; // island enum // gen:195cc5e38c64
pub const WINED3D_SV_CLIP_DISTANCE: wined3d_sysval_semantic = wined3d_sysval_semantic_WINED3D_SV_CLIP_DISTANCE; // island enum // gen:f6b20fe38cd9
pub const WINED3D_SV_CULL_DISTANCE: wined3d_sysval_semantic = wined3d_sysval_semantic_WINED3D_SV_CULL_DISTANCE; // island enum // gen:980fbafde218
pub const WINED3D_SV_INSTANCE_ID: wined3d_sysval_semantic = wined3d_sysval_semantic_WINED3D_SV_INSTANCE_ID; // island enum // gen:ace4c818e545
pub const WINED3D_SV_IS_FRONT_FACE: wined3d_sysval_semantic = wined3d_sysval_semantic_WINED3D_SV_IS_FRONT_FACE; // island enum // gen:f86ce1ad3d48
pub const WINED3D_SV_POSITION: wined3d_sysval_semantic = wined3d_sysval_semantic_WINED3D_SV_POSITION; // island enum // gen:c7303cc19124
pub const WINED3D_SV_RENDER_TARGET_ARRAY_INDEX: wined3d_sysval_semantic = wined3d_sysval_semantic_WINED3D_SV_RENDER_TARGET_ARRAY_INDEX; // island enum // gen:3cdaba6a123a
pub const WINED3D_SV_SAMPLE_INDEX: wined3d_sysval_semantic = wined3d_sysval_semantic_WINED3D_SV_SAMPLE_INDEX; // island enum // gen:626ea6b35679
pub const WINED3D_SV_TESS_FACTOR_LINEDEN: wined3d_sysval_semantic = wined3d_sysval_semantic_WINED3D_SV_TESS_FACTOR_LINEDEN; // island enum // gen:479ae058e676
pub const WINED3D_SV_TESS_FACTOR_LINEDET: wined3d_sysval_semantic = wined3d_sysval_semantic_WINED3D_SV_TESS_FACTOR_LINEDET; // island enum // gen:27ce4fcfdcd9
pub const WINED3D_SV_TESS_FACTOR_QUADEDGE: wined3d_sysval_semantic = wined3d_sysval_semantic_WINED3D_SV_TESS_FACTOR_QUADEDGE; // island enum // gen:a050fd662daa
pub const WINED3D_SV_TESS_FACTOR_QUADINT: wined3d_sysval_semantic = wined3d_sysval_semantic_WINED3D_SV_TESS_FACTOR_QUADINT; // island enum // gen:90376b80757a
pub const WINED3D_SV_TESS_FACTOR_TRIEDGE: wined3d_sysval_semantic = wined3d_sysval_semantic_WINED3D_SV_TESS_FACTOR_TRIEDGE; // island enum // gen:e6ca84f15fec
pub const WINED3D_SV_TESS_FACTOR_TRIINT: wined3d_sysval_semantic = wined3d_sysval_semantic_WINED3D_SV_TESS_FACTOR_TRIINT; // island enum // gen:50088800abe9
pub const WINED3D_SV_VERTEX_ID: wined3d_sysval_semantic = wined3d_sysval_semantic_WINED3D_SV_VERTEX_ID; // island enum // gen:a816ce1413ab
pub const WINED3D_SV_VIEWPORT_ARRAY_INDEX: wined3d_sysval_semantic = wined3d_sysval_semantic_WINED3D_SV_VIEWPORT_ARRAY_INDEX; // island enum // gen:ce646645e80c
pub const WINED3D_SWAP_EFFECT_COPY: wined3d_swap_effect = wined3d_swap_effect_WINED3D_SWAP_EFFECT_COPY; // island enum // gen:c4b8f5e74118
pub const WINED3D_SWAP_EFFECT_COPY_VSYNC: wined3d_swap_effect = wined3d_swap_effect_WINED3D_SWAP_EFFECT_COPY_VSYNC; // island enum // gen:99877ae015ca
pub const WINED3D_SWAP_EFFECT_DISCARD: wined3d_swap_effect = wined3d_swap_effect_WINED3D_SWAP_EFFECT_DISCARD; // island enum // gen:7f0e42311775
pub const WINED3D_SWAP_EFFECT_FLIP_DISCARD: wined3d_swap_effect = wined3d_swap_effect_WINED3D_SWAP_EFFECT_FLIP_DISCARD; // island enum // gen:2a7d0f43a7c4
pub const WINED3D_SWAP_EFFECT_SEQUENTIAL: wined3d_swap_effect = wined3d_swap_effect_WINED3D_SWAP_EFFECT_SEQUENTIAL; // island enum // gen:5af9d459627d
pub const WINED3D_SWAP_INTERVAL_DEFAULT: wined3d_swap_interval = wined3d_swap_interval_WINED3D_SWAP_INTERVAL_DEFAULT; // island enum // gen:a656df69eabe
pub const WINED3D_TADDRESS_BORDER: wined3d_texture_address = wined3d_texture_address_WINED3D_TADDRESS_BORDER; // island enum // gen:81977d2c1e11
pub const WINED3D_TADDRESS_CLAMP: wined3d_texture_address = wined3d_texture_address_WINED3D_TADDRESS_CLAMP; // island enum // gen:fe1f85fcb3c8
pub const WINED3D_TADDRESS_MIRROR: wined3d_texture_address = wined3d_texture_address_WINED3D_TADDRESS_MIRROR; // island enum // gen:47ce235e11a8
pub const WINED3D_TADDRESS_MIRROR_ONCE: wined3d_texture_address = wined3d_texture_address_WINED3D_TADDRESS_MIRROR_ONCE; // island enum // gen:8771e01e936f
pub const WINED3D_TADDRESS_WRAP: wined3d_texture_address = wined3d_texture_address_WINED3D_TADDRESS_WRAP; // island enum // gen:f6dfc7940465
pub const WINED3D_TESSELLATOR_DOMAIN_LINE: wined3d_tessellator_domain = wined3d_tessellator_domain_WINED3D_TESSELLATOR_DOMAIN_LINE; // island enum // gen:1d7cfe2c75a4
pub const WINED3D_TESSELLATOR_DOMAIN_QUAD: wined3d_tessellator_domain = wined3d_tessellator_domain_WINED3D_TESSELLATOR_DOMAIN_QUAD; // island enum // gen:a13d649561d6
pub const WINED3D_TESSELLATOR_DOMAIN_TRIANGLE: wined3d_tessellator_domain = wined3d_tessellator_domain_WINED3D_TESSELLATOR_DOMAIN_TRIANGLE; // island enum // gen:bafbbb997b6f
pub const WINED3D_TESSELLATOR_OUTPUT_LINE: wined3d_tessellator_output_primitive = wined3d_tessellator_output_primitive_WINED3D_TESSELLATOR_OUTPUT_LINE; // island enum // gen:f260ea27570c
pub const WINED3D_TESSELLATOR_OUTPUT_POINT: wined3d_tessellator_output_primitive = wined3d_tessellator_output_primitive_WINED3D_TESSELLATOR_OUTPUT_POINT; // island enum // gen:489e6e096168
pub const WINED3D_TESSELLATOR_OUTPUT_TRIANGLE_CCW: wined3d_tessellator_output_primitive = wined3d_tessellator_output_primitive_WINED3D_TESSELLATOR_OUTPUT_TRIANGLE_CCW; // island enum // gen:d81a91b16360
pub const WINED3D_TESSELLATOR_OUTPUT_TRIANGLE_CW: wined3d_tessellator_output_primitive = wined3d_tessellator_output_primitive_WINED3D_TESSELLATOR_OUTPUT_TRIANGLE_CW; // island enum // gen:0d9f5ae94886
pub const WINED3D_TESSELLATOR_PARTITIONING_FRACTIONAL_EVEN: wined3d_tessellator_partitioning = wined3d_tessellator_partitioning_WINED3D_TESSELLATOR_PARTITIONING_FRACTIONAL_EVEN; // island enum // gen:b4279faaa617
pub const WINED3D_TESSELLATOR_PARTITIONING_FRACTIONAL_ODD: wined3d_tessellator_partitioning = wined3d_tessellator_partitioning_WINED3D_TESSELLATOR_PARTITIONING_FRACTIONAL_ODD; // island enum // gen:cd332ea58c2d
pub const WINED3D_TESSELLATOR_PARTITIONING_INTEGER: wined3d_tessellator_partitioning = wined3d_tessellator_partitioning_WINED3D_TESSELLATOR_PARTITIONING_INTEGER; // island enum // gen:10b11584b26e
pub const WINED3D_TESSELLATOR_PARTITIONING_POW2: wined3d_tessellator_partitioning = wined3d_tessellator_partitioning_WINED3D_TESSELLATOR_PARTITIONING_POW2; // island enum // gen:9dde73fbecde
pub const WINED3D_TEXF_ANISOTROPIC: wined3d_texture_filter_type = wined3d_texture_filter_type_WINED3D_TEXF_ANISOTROPIC; // island enum // gen:ebd8fe7d182c
pub const WINED3D_TEXF_FLAT_CUBIC: wined3d_texture_filter_type = wined3d_texture_filter_type_WINED3D_TEXF_FLAT_CUBIC; // island enum // gen:a80008d91d10
pub const WINED3D_TEXF_GAUSSIAN_CUBIC: wined3d_texture_filter_type = wined3d_texture_filter_type_WINED3D_TEXF_GAUSSIAN_CUBIC; // island enum // gen:b239b428d4e2
pub const WINED3D_TEXF_GAUSSIAN_QUAD: wined3d_texture_filter_type = wined3d_texture_filter_type_WINED3D_TEXF_GAUSSIAN_QUAD; // island enum // gen:1c7b2475627d
pub const WINED3D_TEXF_LINEAR: wined3d_texture_filter_type = wined3d_texture_filter_type_WINED3D_TEXF_LINEAR; // island enum // gen:a0db557e9dc5
pub const WINED3D_TEXF_NONE: wined3d_texture_filter_type = wined3d_texture_filter_type_WINED3D_TEXF_NONE; // island enum // gen:68af396e0aaf
pub const WINED3D_TEXF_POINT: wined3d_texture_filter_type = wined3d_texture_filter_type_WINED3D_TEXF_POINT; // island enum // gen:db8018ba15af
pub const WINED3D_TEXF_PYRAMIDAL_QUAD: wined3d_texture_filter_type = wined3d_texture_filter_type_WINED3D_TEXF_PYRAMIDAL_QUAD; // island enum // gen:25d49d14011f
pub const WINED3D_TOP_ADD: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_ADD; // island enum // gen:e2dc16a35b7f
pub const WINED3D_TOP_ADD_SIGNED: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_ADD_SIGNED; // island enum // gen:6e50af906207
pub const WINED3D_TOP_ADD_SIGNED_2X: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_ADD_SIGNED_2X; // island enum // gen:5c008b863266
pub const WINED3D_TOP_ADD_SMOOTH: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_ADD_SMOOTH; // island enum // gen:1b5d6b0be295
pub const WINED3D_TOP_BLEND_CURRENT_ALPHA: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_BLEND_CURRENT_ALPHA; // island enum // gen:e6aabd06b7f4
pub const WINED3D_TOP_BLEND_DIFFUSE_ALPHA: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_BLEND_DIFFUSE_ALPHA; // island enum // gen:9f56b710dd4a
pub const WINED3D_TOP_BLEND_FACTOR_ALPHA: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_BLEND_FACTOR_ALPHA; // island enum // gen:462b8601dd77
pub const WINED3D_TOP_BLEND_TEXTURE_ALPHA: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_BLEND_TEXTURE_ALPHA; // island enum // gen:b6e095727311
pub const WINED3D_TOP_BLEND_TEXTURE_ALPHA_PM: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_BLEND_TEXTURE_ALPHA_PM; // island enum // gen:bceb7177a1a7
pub const WINED3D_TOP_BUMPENVMAP: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_BUMPENVMAP; // island enum // gen:2d56b3c28588
pub const WINED3D_TOP_BUMPENVMAP_LUMINANCE: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_BUMPENVMAP_LUMINANCE; // island enum // gen:7830fc974613
pub const WINED3D_TOP_DISABLE: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_DISABLE; // island enum // gen:3013ac282edb
pub const WINED3D_TOP_DOTPRODUCT3: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_DOTPRODUCT3; // island enum // gen:06683d83880b
pub const WINED3D_TOP_LERP: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_LERP; // island enum // gen:ba777fd28f0e
pub const WINED3D_TOP_MODULATE: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_MODULATE; // island enum // gen:d4c339bb6934
pub const WINED3D_TOP_MODULATE_2X: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_MODULATE_2X; // island enum // gen:5c1c802650b6
pub const WINED3D_TOP_MODULATE_4X: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_MODULATE_4X; // island enum // gen:5a5c2a1a06bb
pub const WINED3D_TOP_MODULATE_ALPHA_ADD_COLOR: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_MODULATE_ALPHA_ADD_COLOR; // island enum // gen:d3585f1014bf
pub const WINED3D_TOP_MODULATE_COLOR_ADD_ALPHA: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_MODULATE_COLOR_ADD_ALPHA; // island enum // gen:e149ac2d9803
pub const WINED3D_TOP_MODULATE_INVALPHA_ADD_COLOR: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_MODULATE_INVALPHA_ADD_COLOR; // island enum // gen:b2f192bd2dc4
pub const WINED3D_TOP_MODULATE_INVCOLOR_ADD_ALPHA: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_MODULATE_INVCOLOR_ADD_ALPHA; // island enum // gen:f81de5f8e323
pub const WINED3D_TOP_MULTIPLY_ADD: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_MULTIPLY_ADD; // island enum // gen:0aba69b1085a
pub const WINED3D_TOP_PREMODULATE: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_PREMODULATE; // island enum // gen:82ea1a1b3a47
pub const WINED3D_TOP_SELECT_ARG1: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_SELECT_ARG1; // island enum // gen:1db12af4f4e5
pub const WINED3D_TOP_SELECT_ARG2: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_SELECT_ARG2; // island enum // gen:a5ee1ebbb8f5
pub const WINED3D_TOP_SUBTRACT: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_SUBTRACT; // island enum // gen:fbff50ddd5b3
pub const WINED3D_TSS_ALPHA_ARG0: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_ALPHA_ARG0; // island enum // gen:a8dfb94596fb
pub const WINED3D_TSS_ALPHA_ARG1: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_ALPHA_ARG1; // island enum // gen:ee76536251ec
pub const WINED3D_TSS_ALPHA_ARG2: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_ALPHA_ARG2; // island enum // gen:793c542ef090
pub const WINED3D_TSS_ALPHA_OP: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_ALPHA_OP; // island enum // gen:2439b29fc399
pub const WINED3D_TSS_BUMPENV_LOFFSET: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_BUMPENV_LOFFSET; // island enum // gen:42a822880cb8
pub const WINED3D_TSS_BUMPENV_LSCALE: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_BUMPENV_LSCALE; // island enum // gen:27feb64a5bfa
pub const WINED3D_TSS_BUMPENV_MAT00: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_BUMPENV_MAT00; // island enum // gen:cb219866bcc3
pub const WINED3D_TSS_BUMPENV_MAT01: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_BUMPENV_MAT01; // island enum // gen:5869728db43e
pub const WINED3D_TSS_BUMPENV_MAT10: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_BUMPENV_MAT10; // island enum // gen:028a69e3ab12
pub const WINED3D_TSS_BUMPENV_MAT11: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_BUMPENV_MAT11; // island enum // gen:37d61e1797ae
pub const WINED3D_TSS_COLOR_ARG0: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_COLOR_ARG0; // island enum // gen:d10f481bbde6
pub const WINED3D_TSS_COLOR_ARG1: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_COLOR_ARG1; // island enum // gen:d6cbebeefa27
pub const WINED3D_TSS_COLOR_ARG2: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_COLOR_ARG2; // island enum // gen:6952c588431b
pub const WINED3D_TSS_COLOR_OP: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_COLOR_OP; // island enum // gen:f829cae71631
pub const WINED3D_TSS_CONSTANT: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_CONSTANT; // island enum // gen:0f0ea854c5ce
pub const WINED3D_TSS_RESULT_ARG: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_RESULT_ARG; // island enum // gen:c25fee23b0cb
pub const WINED3D_TSS_TEXCOORD_INDEX: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_TEXCOORD_INDEX; // island enum // gen:8e80cbf7e4eb
pub const WINED3D_TSS_TEXTURE_TRANSFORM_FLAGS: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_TEXTURE_TRANSFORM_FLAGS; // island enum // gen:f9828e809b53
pub const WINED3D_TS_PROJECTION: wined3d_transform_state = wined3d_transform_state_WINED3D_TS_PROJECTION; // island enum // gen:886cceefd518
pub const WINED3D_TS_TEXTURE0: wined3d_transform_state = wined3d_transform_state_WINED3D_TS_TEXTURE0; // island enum // gen:d9a0b20c50a9
pub const WINED3D_TS_TEXTURE1: wined3d_transform_state = wined3d_transform_state_WINED3D_TS_TEXTURE1; // island enum // gen:8fc8e16e0317
pub const WINED3D_TS_TEXTURE2: wined3d_transform_state = wined3d_transform_state_WINED3D_TS_TEXTURE2; // island enum // gen:327be17302a3
pub const WINED3D_TS_TEXTURE3: wined3d_transform_state = wined3d_transform_state_WINED3D_TS_TEXTURE3; // island enum // gen:1d69e8cc06d5
pub const WINED3D_TS_TEXTURE4: wined3d_transform_state = wined3d_transform_state_WINED3D_TS_TEXTURE4; // island enum // gen:2314e1d8f220
pub const WINED3D_TS_TEXTURE5: wined3d_transform_state = wined3d_transform_state_WINED3D_TS_TEXTURE5; // island enum // gen:ba9e675cf6ae
pub const WINED3D_TS_TEXTURE6: wined3d_transform_state = wined3d_transform_state_WINED3D_TS_TEXTURE6; // island enum // gen:e0f7d8db5372
pub const WINED3D_TS_TEXTURE7: wined3d_transform_state = wined3d_transform_state_WINED3D_TS_TEXTURE7; // island enum // gen:3cadebb6ac4a
pub const WINED3D_TS_VIEW: wined3d_transform_state = wined3d_transform_state_WINED3D_TS_VIEW; // island enum // gen:3093d1a59334
pub const WINED3D_TS_WORLD: wined3d_transform_state = wined3d_transform_state_WINED3D_TS_WORLD; // island enum // gen:2ae8c4109807
pub const WINED3D_TTFF_COUNT1: wined3d_texture_transform_flags = wined3d_texture_transform_flags_WINED3D_TTFF_COUNT1; // island enum // gen:6fc93b26158f
pub const WINED3D_TTFF_COUNT2: wined3d_texture_transform_flags = wined3d_texture_transform_flags_WINED3D_TTFF_COUNT2; // island enum // gen:38051d90f775
pub const WINED3D_TTFF_COUNT3: wined3d_texture_transform_flags = wined3d_texture_transform_flags_WINED3D_TTFF_COUNT3; // island enum // gen:7b865810e9c9
pub const WINED3D_TTFF_COUNT4: wined3d_texture_transform_flags = wined3d_texture_transform_flags_WINED3D_TTFF_COUNT4; // island enum // gen:ded41c351b1d
pub const WINED3D_TTFF_DISABLE: wined3d_texture_transform_flags = wined3d_texture_transform_flags_WINED3D_TTFF_DISABLE; // island enum // gen:902b779dc05e
pub const WINED3D_TTFF_PROJECTED: wined3d_texture_transform_flags = wined3d_texture_transform_flags_WINED3D_TTFF_PROJECTED; // island enum // gen:8ba28038d5e9
pub const WINED3D_TYPE_FLOAT: wined3d_component_type = wined3d_component_type_WINED3D_TYPE_FLOAT; // island enum // gen:88bfde01dabd
pub const WINED3D_TYPE_INT: wined3d_component_type = wined3d_component_type_WINED3D_TYPE_INT; // island enum // gen:346cf8c8fbad
pub const WINED3D_TYPE_UINT: wined3d_component_type = wined3d_component_type_WINED3D_TYPE_UINT; // island enum // gen:cdb00d85455d
pub const WINED3D_TYPE_UNKNOWN: wined3d_component_type = wined3d_component_type_WINED3D_TYPE_UNKNOWN; // island enum // gen:b1844fa5fc62
pub const WINED3D_VBF_1WEIGHTS: wined3d_vertex_blend_flags = wined3d_vertex_blend_flags_WINED3D_VBF_1WEIGHTS; // island enum // gen:14c009b554af
pub const WINED3D_VBF_2WEIGHTS: wined3d_vertex_blend_flags = wined3d_vertex_blend_flags_WINED3D_VBF_2WEIGHTS; // island enum // gen:612fd9e3827e
pub const WINED3D_VBF_3WEIGHTS: wined3d_vertex_blend_flags = wined3d_vertex_blend_flags_WINED3D_VBF_3WEIGHTS; // island enum // gen:4acd77506e1d
pub const WINED3D_VBF_DISABLE: wined3d_vertex_blend_flags = wined3d_vertex_blend_flags_WINED3D_VBF_DISABLE; // island enum // gen:6789edc00255
pub const WINED3D_VERTEX_SAMPLER_OFFSET: i32 = 0x10; // d3d9-rust/src/prelude.rs // gen:caa9c357b4b7
pub const WINED3D_VK_EXT_EXTENDED_DYNAMIC_STATE2: wined3d_vk_extension = wined3d_vk_extension_WINED3D_VK_EXT_EXTENDED_DYNAMIC_STATE2; // island enum // gen:f147c4d28288
pub const WINED3D_VK_EXT_EXTENDED_DYNAMIC_STATE3: wined3d_vk_extension = wined3d_vk_extension_WINED3D_VK_EXT_EXTENDED_DYNAMIC_STATE3; // island enum // gen:871e117d4410
pub const WINED3D_VK_EXT_EXTENDED_DYNAMIC_STATE: wined3d_vk_extension = wined3d_vk_extension_WINED3D_VK_EXT_EXTENDED_DYNAMIC_STATE; // island enum // gen:3d212f1f95cd
pub const WINED3D_VK_EXT_HOST_QUERY_RESET: wined3d_vk_extension = wined3d_vk_extension_WINED3D_VK_EXT_HOST_QUERY_RESET; // island enum // gen:1d1fca1c3b19
pub const WINED3D_VK_EXT_NONE: wined3d_vk_extension = wined3d_vk_extension_WINED3D_VK_EXT_NONE; // island enum // gen:718b8584d7fa
pub const WINED3D_VK_EXT_SHADER_STENCIL_EXPORT: wined3d_vk_extension = wined3d_vk_extension_WINED3D_VK_EXT_SHADER_STENCIL_EXPORT; // island enum // gen:f16109b8186d
pub const WINED3D_VK_EXT_TRANSFORM_FEEDBACK: wined3d_vk_extension = wined3d_vk_extension_WINED3D_VK_EXT_TRANSFORM_FEEDBACK; // island enum // gen:6bb2c8a1da02
pub const WINED3D_VK_EXT_VERTEX_ATTRIBUTE_DIVISOR: wined3d_vk_extension = wined3d_vk_extension_WINED3D_VK_EXT_VERTEX_ATTRIBUTE_DIVISOR; // island enum // gen:b0b255c7140d
pub const WINED3D_VK_KHR_MAINTENANCE2: wined3d_vk_extension = wined3d_vk_extension_WINED3D_VK_KHR_MAINTENANCE2; // island enum // gen:40089ae8686d
pub const WINED3D_VK_KHR_SAMPLER_MIRROR_CLAMP_TO_EDGE: wined3d_vk_extension = wined3d_vk_extension_WINED3D_VK_KHR_SAMPLER_MIRROR_CLAMP_TO_EDGE; // island enum // gen:4dd6aa3005ed
pub const WINED3D_VK_KHR_SHADER_DRAW_PARAMETERS: wined3d_vk_extension = wined3d_vk_extension_WINED3D_VK_KHR_SHADER_DRAW_PARAMETERS; // island enum // gen:9703bcadc0ff
pub const WINED3D_VP_MODE_FF: wined3d_vertex_processing_mode = wined3d_vertex_processing_mode_WINED3D_VP_MODE_FF; // island enum // gen:4ddcc8c48cec
pub const WINED3D_VP_MODE_NONE: wined3d_vertex_processing_mode = wined3d_vertex_processing_mode_WINED3D_VP_MODE_NONE; // island enum // gen:91272ff077b4
pub const WINED3D_VP_MODE_SHADER: wined3d_vertex_processing_mode = wined3d_vertex_processing_mode_WINED3D_VP_MODE_SHADER; // island enum // gen:1e480757c4ff
pub const WINED3D_ZB_FALSE: wined3d_depth_buffer_type = wined3d_depth_buffer_type_WINED3D_ZB_FALSE; // island enum // gen:f6f39574a296
pub const WINED3D_ZB_TRUE: wined3d_depth_buffer_type = wined3d_depth_buffer_type_WINED3D_ZB_TRUE; // island enum // gen:93340c841fa2
pub const WINED3D_ZB_USEW: wined3d_depth_buffer_type = wined3d_depth_buffer_type_WINED3D_ZB_USEW; // island enum // gen:66ca33f9f66f
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0; // comctl32-rust/src/prelude.rs // gen:2780c6213b17
pub type wined3d_ffp_bumpenv_constants = wined3d_ffp_ps_constants_wined3d_ffp_bumpenv_constants; // island nested // gen:e70b34c240f4
pub type wined3d_ffp_vs_modelview_matrices = wined3d_ffp_vs_constants_wined3d_ffp_vs_modelview_matrices; // island nested // gen:079bb1d84478
pub type wined3d_texture_sub_resource = wined3d_texture_wined3d_texture_sub_resource; // island nested // gen:6c939e0f00d0
extern "C" {
    pub fn __wine_dbg_output(str_: *const ::core::ffi::c_char) -> ::core::ffi::c_int; // winegdi-rust/src/types_i386.rs // gen:57c09b98cb41
    pub fn _finite(x: f64) -> i32; // calc-rust/src/prelude.rs // gen:fbba3be37245
    pub fn _isnan(x: f64) -> i32; // calc-rust/src/prelude.rs // gen:0c9c3fe9a2cb
    pub fn abs(x: i32) -> i32; // charmap-rust/src/prelude.rs // gen:0d23679e353a
    pub fn calloc(arg1: ::core::ffi::c_uint, arg2: ::core::ffi::c_uint) -> *mut ::core::ffi::c_void; // msvcrt-sys/src/bindgen_i386.rs // gen:d3193c100ce5
    pub fn fabsf(x: f32) -> f32; // cnt-rust/src/cabi/math.rs // gen:8d2f26542941
    pub fn floor(x: f64) -> f64; // cnt-rust/src/cabi/math.rs // gen:76be2167a6a3
    pub fn free(p: *mut c_void); // activeds-rust/src/prelude.rs // gen:0ad441c3c230
    pub fn isdigit(c: i32) -> i32; // arp-rust/src/prelude.rs // gen:98c05ad9bc93
    pub fn isspace(c: i32) -> i32; // cnt-ctype/src/lib.rs // gen:bd320cdfba63
    pub fn malloc(size: usize) -> *mut c_void; // activeds-rust/src/prelude.rs // gen:86a212c6fd14
    pub fn memchr(s: *const c_void, c: i32, n: usize) -> *mut c_void; // cmd-rust/src/prelude.rs // gen:40376f9ab53d
    pub fn modf(x: f64, iptr: *mut f64) -> f64; // calc-rust/src/prelude.rs // gen:40cc62d4020d
    pub fn realloc(p: *mut c_void, size: usize) -> *mut c_void; // activeds-rust/src/prelude.rs // gen:ce18447a16eb
    pub fn sscanf(s: *const i8, fmt: *const i8, ...) -> i32; // arp-rust/src/prelude.rs // gen:449a48721d25
    pub fn wine_dbg_sprintf(format: *const ::core::ffi::c_char, ...) -> *const ::core::ffi::c_char; // winegdi-rust/src/types_i386.rs // gen:64f2d13938b7
}
extern "stdcall" {
    pub fn InitializeCriticalSectionEx( lpCriticalSection: LPCRITICAL_SECTION, dwSpinCount: u32, Flags: u32, ) -> BOOL; // msvcrt-sys/src/bindgen_i386.rs // gen:90fdae6c8e87
}
extern "system" {
    pub fn AdjustWindowRectEx(arg1: LPRECT, arg2: DWORD, arg3: BOOL, arg4: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs // gen:a8f999718935
    pub fn AllocateLocallyUniqueId(arg1: PLUID) -> BOOL; // aclui-rust/src/types_i386.rs // gen:e064b529a1cd
    pub fn BitBlt( hdcDest: HDC, xDest: INT, yDest: INT, cxDest: INT, cyDest: INT, hdcSrc: HDC, xSrc: INT, ySrc: INT, dwRop: DWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:6e151f759486
    pub fn CallNextHookEx( arg1: HHOOK, arg2: ::core::ffi::c_int, arg3: WPARAM, arg4: LPARAM, ) -> LRESULT; // aclui-rust/src/types_i386.rs // gen:e2583d3437b5
    pub fn CallWindowProcA( arg1: WNDPROC, arg2: HWND, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_i386.rs // gen:91db831e35ef
    pub fn CallWindowProcW( arg1: WNDPROC, arg2: HWND, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_i386.rs // gen:e20f8c580dce
    pub fn ChangeDisplaySettingsExW( arg1: LPCWSTR, arg2: LPDEVMODEW, arg3: HWND, arg4: DWORD, arg5: LPVOID, ) -> LONG; // aclui-rust/src/types_i386.rs // gen:1e132dc14e81
    pub fn ChoosePixelFormat(hdc: HDC, ppfd: *const PIXELFORMATDESCRIPTOR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:76849455e906
    pub fn ClientToScreen(arg1: HWND, arg2: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs // gen:c97a37b41925
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:4a43278b42fd
    pub fn CreateBitmap( cx: INT, cy: INT, cPlanes: UINT, cBitsPerPel: UINT, pvBits: *const ::core::ffi::c_void, ) -> HBITMAP; // aclui-rust/src/types_i386.rs // gen:814d2f0e94bc
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC; // aclui-rust/src/types_i386.rs // gen:791328af02e4
    pub fn CreateDCW( pszDriver: LPCWSTR, pszDevice: LPCWSTR, psz: LPCWSTR, pdmInit: *const DEVMODEW, ) -> HDC; // aclui-rust/src/types_i386.rs // gen:cc321e6e3d17
    pub fn CreateEventW( arg1: LPSECURITY_ATTRIBUTES, bManualReset: BOOL, bInitialState: BOOL, lpName: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs // gen:f162982b6326
    pub fn CreateIconIndirect(arg1: PICONINFO) -> HICON; // aclui-rust/src/types_i386.rs // gen:b807d77fded0
    pub fn CreateThread( arg1: LPSECURITY_ATTRIBUTES, arg2: DWORD, arg3: LPTHREAD_START_ROUTINE, arg4: PVOID, arg5: DWORD, arg6: PDWORD, ) -> HANDLE; // aclui-rust/src/types_i386.rs // gen:29214eec4ab3
    pub fn CreateWindowExA( dwExStyle: DWORD, lpClassName: LPCSTR, lpWindowName: LPCSTR, dwStyle: DWORD, X: ::core::ffi::c_int, Y: ::core::ffi::c_int, nWidth: ::core::ffi::c_int, nHeight: ::core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID, ) -> HWND; // aclui-rust/src/types_i386.rs // gen:2a5bd341d50b
    pub fn D3DKMTCloseAdapter(unnamedParam1: *const D3DKMT_CLOSEADAPTER) -> NTSTATUS; // gdi32_vista-rust/src/gdi32_vista/d3dkmt.rs // gen:645df47e6ee5
    pub fn D3DKMTCreateDCFromMemory(desc: *mut D3DKMT_CREATEDCFROMMEMORY,) -> NTSTATUS; // gdi32_vista-rust/src/gdi32_vista/dc.rs // gen:af93409be586
    pub fn D3DKMTCreateDevice(unnamedParam1: *mut D3DKMT_CREATEDEVICE,) -> NTSTATUS; // gdi32_vista-rust/src/gdi32_vista/d3dkmt.rs // gen:bfe56901bcba
    pub fn D3DKMTDestroyDCFromMemory(desc: *const D3DKMT_DESTROYDCFROMMEMORY,) -> NTSTATUS; // gdi32_vista-rust/src/gdi32_vista/dc.rs // gen:a3e3718181ba
    pub fn D3DKMTDestroyDevice(unnamedParam1: *const D3DKMT_DESTROYDEVICE,) -> NTSTATUS; // gdi32_vista-rust/src/gdi32_vista/d3dkmt.rs // gen:8004c39e1227
    pub fn D3DKMTOpenAdapterFromGdiDisplayName(unnamedParam1: *mut D3DKMT_OPENADAPTERFROMGDIDISPLAYNAME,) -> NTSTATUS; // gdi32_vista-rust/src/gdi32_vista/d3dkmt.rs // gen:17f0b370972a
    pub fn D3DKMTOpenAdapterFromLuid(unnamedParam1: *const D3DKMT_OPENADAPTERFROMLUID,) -> NTSTATUS; // gdi32_vista-rust/src/gdi32_vista/d3dkmt.rs // gen:6e2bd10d0b5e
    pub fn D3DKMTQueryVideoMemoryInfo(unnamedParam1: *mut D3DKMT_QUERYVIDEOMEMORYINFO,) -> NTSTATUS; // gdi32_vista-rust/src/gdi32_vista/d3dkmt.rs // gen:bd49bf267fe4
    pub fn D3DKMTSetVidPnSourceOwner(unnamedParam1: *const D3DKMT_SETVIDPNSOURCEOWNER,) -> NTSTATUS; // gdi32_vista-rust/src/gdi32_vista/d3dkmt.rs // gen:70cdf6e0f48b
    pub fn DebugBreak(); // aclui-rust/src/types_i386.rs // gen:f423611c3e66
    pub fn DefWindowProcA(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs // gen:68bcc01160cf
    pub fn DefWindowProcW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs // gen:aca94375b75d
    pub fn DeleteCriticalSection(arg1: PCRITICAL_SECTION); // aclui-rust/src/types_i386.rs // gen:7893eb6ab4e4
    pub fn DeleteDC(arg1: HDC) -> BOOL; // aclui-rust/src/types_i386.rs // gen:879520c6bc24
    pub fn DeleteObject(arg1: HGDIOBJ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:000f9d795415
    pub fn DescribePixelFormat( hdc: HDC, iPixelFormat: ::core::ffi::c_int, nBytes: UINT, ppfd: LPPIXELFORMATDESCRIPTOR, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:254b5d472899
    pub fn DestroyCursor(arg1: HCURSOR) -> BOOL; // aclui-rust/src/types_i386.rs // gen:8f5520ba8f5a
    pub fn DestroyWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs // gen:b6137cacf702
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:111127023c7f
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs // gen:b4d73724c7aa
    pub fn EnumDisplayDevicesW( arg1: LPCWSTR, arg2: DWORD, arg3: PDISPLAY_DEVICEW, arg4: DWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:9990299104a6
    pub fn EnumDisplayMonitors( arg1: HDC, arg2: LPCRECT, arg3: MONITORENUMPROC, arg4: LPARAM, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:e07603f3b1d4
    pub fn EnumDisplaySettingsExW( arg1: LPCWSTR, arg2: DWORD, arg3: LPDEVMODEW, arg4: DWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:b7deafd9f823
    pub fn EnumDisplaySettingsW(arg1: LPCWSTR, arg2: DWORD, arg3: PDEVMODEW) -> BOOL; // aclui-rust/src/types_i386.rs // gen:384d43ed40d7
    pub fn FindResourceA(arg1: HMODULE, arg2: LPCSTR, arg3: LPCSTR) -> HRSRC; // aclui-rust/src/types_i386.rs // gen:651a45575f6c
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:fcc80d36382e
    pub fn FreeLibraryAndExitThread(arg1: HMODULE, arg2: DWORD) -> !; // aclui-rust/src/types_i386.rs // gen:ee650b60881b
    pub fn FreeResource(arg1: HGLOBAL) -> BOOL; // aclui-rust/src/types_i386.rs // gen:304c3df78c3d
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs // gen:ef10fc0cc6f5
    pub fn GetCurrentThread() -> HANDLE; // aclui-rust/src/types_i386.rs // gen:dde70d52bf8b
    pub fn GetCurrentThreadId() -> DWORD; // aclui-rust/src/types_i386.rs // gen:b82f9257e472
    pub fn GetCursorPos(arg1: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs // gen:4ebf9f96bef7
    pub fn GetDC(arg1: HWND) -> HDC; // aclui-rust/src/types_i386.rs // gen:39c5b894d19a
    pub fn GetDCEx(arg1: HWND, arg2: HRGN, arg3: DWORD) -> HDC; // aclui-rust/src/types_i386.rs // gen:c98bd9210ebb
    pub fn GetDesktopWindow() -> HWND; // aclui-rust/src/types_i386.rs // gen:dec8495c75ca
    pub fn GetDeviceGammaRamp(hdc: HDC, lpRamp: LPVOID) -> BOOL; // aclui-rust/src/types_i386.rs // gen:4474dbd1ba6e
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs // gen:50e246193424
    pub fn GetModuleFileNameA(hModule: HINSTANCE, lpFilename: LPSTR, nSize: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs // gen:51bffb082f29
    pub fn GetModuleHandleA(arg1: LPCSTR) -> HMODULE; // aclui-rust/src/types_i386.rs // gen:f6b080237bcc
    pub fn GetModuleHandleExA(arg1: DWORD, arg2: LPCSTR, arg3: *mut HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:43d97743191d
    pub fn GetModuleHandleExW(arg1: DWORD, arg2: LPCWSTR, arg3: *mut HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:e24f313d56d9
    pub fn GetModuleHandleW(arg1: LPCWSTR) -> HMODULE; // aclui-rust/src/types_i386.rs // gen:be8cca7cb13f
    pub fn GetMonitorInfoW(arg1: HMONITOR, arg2: LPMONITORINFO) -> BOOL; // aclui-rust/src/types_i386.rs // gen:961fd7b1cff4
    pub fn GetObjectA(h: HANDLE, c: ::core::ffi::c_int, pv: LPVOID) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:9d28f862cb24
    pub fn GetPixelFormat(arg1: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:ca91e1e0aff7
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_i386.rs // gen:2f213353c565
    pub fn GetTickCount() -> DWORD; // aclui-rust/src/types_i386.rs // gen:f624a448a1f6
    pub fn GetVersionExW(arg1: LPOSVERSIONINFOW) -> BOOL; // aclui-rust/src/types_i386.rs // gen:a5a1b9e0a808
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongA")]
    pub fn GetWindowLongPtrA(hWnd: HWND, nIndex: i32) -> LONG_PTR; // user32-rust/src/windows/class.rs // gen:5cd14c184c70
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR; // charmap-rust/src/prelude.rs // gen:79545aae4953
    pub fn GetWindowLongW(arg1: HWND, arg2: ::core::ffi::c_int) -> LONG; // aclui-rust/src/types_i386.rs // gen:f8e45f883779
    pub fn GetWindowRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs // gen:c643e52829aa
    pub fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: PDWORD) -> DWORD; // aclui-rust/src/types_i386.rs // gen:6e6ec69d00ea
    pub fn GlobalMemoryStatusEx(arg1: LPMEMORYSTATUSEX) -> BOOL; // aclui-rust/src/types_i386.rs // gen:8cb4f652cb9e
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs // gen:3aaa7317691b
    pub fn HeapCreate(arg1: DWORD, arg2: SIZE_T, arg3: SIZE_T) -> HANDLE; // aclui-rust/src/types_i386.rs // gen:28b3eb8611b5
    pub fn HeapDestroy(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:62f88ac42fb0
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs // gen:cd40b76d9b6b
    pub fn IntersectRect(arg1: LPRECT, arg2: LPCRECT, arg3: LPCRECT) -> BOOL; // aclui-rust/src/types_i386.rs // gen:63e50b34f63a
    pub fn IsRectEmpty(arg1: LPCRECT) -> BOOL; // aclui-rust/src/types_i386.rs // gen:56044909b2d0
    pub fn IsWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs // gen:40af86961eab
    pub fn IsWindowUnicode(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs // gen:53b47819bd0e
    pub fn IsWindowVisible(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs // gen:284c2890a15a
    pub fn KillTimer(arg1: HWND, arg2: UINT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs // gen:ba76aa7174f9
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs // gen:d32c25758174
    pub fn LoadCursorA(arg1: HINSTANCE, arg2: LPCSTR) -> HCURSOR; // aclui-rust/src/types_i386.rs // gen:ca3ae8b0eca5
    pub fn LoadIconA(hInstance: HINSTANCE, lpIconName: LPCSTR) -> HICON; // aclui-rust/src/types_i386.rs // gen:6f66dee36d36
    pub fn LoadImageA( hInst: HINSTANCE, name: LPCSTR, type_: UINT, cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, fuLoad: UINT, ) -> HANDLE; // aclui-rust/src/types_i386.rs // gen:d3d91867ac0a
    pub fn LoadLibraryA(arg1: LPCSTR) -> HINSTANCE; // aclui-rust/src/types_i386.rs // gen:fd1cf5811182
    pub fn LoadResource(arg1: HINSTANCE, arg2: HRSRC) -> HGLOBAL; // aclui-rust/src/types_i386.rs // gen:d8e6509045a9
    pub fn LockResource(arg1: HGLOBAL) -> PVOID; // aclui-rust/src/types_i386.rs // gen:052ec03930d2
    pub fn MapWindowPoints( hWndFrom: HWND, hWndTo: HWND, lpPoints: LPPOINT, cPoints: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:9c84986486a6
    pub fn MonitorFromWindow(arg1: HWND, arg2: DWORD) -> HMONITOR; // aclui-rust/src/types_i386.rs // gen:a7d129f06074
    pub fn MoveWindow( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: BOOL, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:dd7ecd6b3241
    pub fn OffsetRect(arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs // gen:268f21e4edae
    pub fn QueryPerformanceCounter(arg1: PLARGE_INTEGER) -> BOOL; // aclui-rust/src/types_i386.rs // gen:e1a525d90389
    pub fn QueryPerformanceFrequency(arg1: PLARGE_INTEGER) -> BOOL; // aclui-rust/src/types_i386.rs // gen:9475484b8364
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:d83c13224bef
    pub fn RegOpenKeyA(arg1: HKEY, arg2: LPCSTR, arg3: PHKEY) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:86e841a4c587
    pub fn RegQueryValueExA( hKey: HKEY, lpValueName: LPCSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:6bcb7f4421b5
    pub fn RegisterClassA(arg1: *const WNDCLASSA) -> ATOM; // aclui-rust/src/types_i386.rs // gen:c110e5303288
    pub fn ReleaseDC(arg1: HWND, arg2: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:0952239cd2d2
    pub fn ScreenToClient(arg1: HWND, arg2: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs // gen:e0b384d622d2
    pub fn SelectObject(arg1: HDC, arg2: HGDIOBJ) -> HGDIOBJ; // aclui-rust/src/types_i386.rs // gen:6f8d653c63e8
    pub fn SetCursor(arg1: HCURSOR) -> HCURSOR; // aclui-rust/src/types_i386.rs // gen:2ccc224e8201
    pub fn SetCursorPos(arg1: ::core::ffi::c_int, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs // gen:2421e291c6bb
    pub fn SetDIBColorTable(hdc: HDC, iStart: UINT, cEntries: UINT, prgbq: *const RGBQUAD) -> UINT; // aclui-rust/src/types_i386.rs // gen:26f06c7476db
    pub fn SetDeviceGammaRamp(hdc: HDC, lpRamp: LPVOID) -> BOOL; // aclui-rust/src/types_i386.rs // gen:6d89325143ca
    pub fn SetEvent(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:198f85e2baee
    pub fn SetPixelFormat( arg1: HDC, arg2: ::core::ffi::c_int, arg3: *const PIXELFORMATDESCRIPTOR, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:96fbef90fdfa
    pub fn SetRect( arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:9d720f054480
    pub fn SetRectEmpty(arg1: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs // gen:24dd6a6e6d13
    pub fn SetThreadDescription(hThread: *mut ::core::ffi::c_void, lpThreadDescription: PCWSTR) -> HRESULT; // winlogon-sys/src/bindgen_i386.rs // gen:071c8128c0c9
    pub fn SetTimer(arg1: HWND, arg2: UINT_PTR, arg3: UINT, arg4: TIMERPROC) -> UINT_PTR; // aclui-rust/src/types_i386.rs // gen:4ec7a1ac58ac
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongA")]
    pub fn SetWindowLongPtrA(hWnd: HWND, nIndex: i32, dwNewLong: isize) -> LONG_PTR; // winhlp32-rust/src/prelude.rs // gen:466eb41b0730
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR; // charmap-rust/src/prelude.rs // gen:07cfdf4cb040
    pub fn SetWindowLongW(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LONG) -> LONG; // aclui-rust/src/types_i386.rs // gen:d2848a99266a
    pub fn SetWindowPos( arg1: HWND, arg2: HWND, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: UINT, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:bbf7b4058eae
    pub fn SetWindowsHookExW( arg1: ::core::ffi::c_int, arg2: HOOKPROC, arg3: HINSTANCE, arg4: DWORD, ) -> HHOOK; // aclui-rust/src/types_i386.rs // gen:ec97e57f1b44
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs // gen:a99a94e9f6ba
    pub fn SizeofResource(arg1: HINSTANCE, arg2: HRSRC) -> DWORD; // aclui-rust/src/types_i386.rs // gen:c7c25cdaea0f
    pub fn StretchBlt( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: HDC, arg7: ::core::ffi::c_int, arg8: ::core::ffi::c_int, arg9: ::core::ffi::c_int, arg10: ::core::ffi::c_int, arg11: DWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:5c0e097bf9d2
    pub fn SystemParametersInfoW( uiAction: UINT, uiParam: UINT, pvParam: PVOID, fWinIni: UINT, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:4749b0b0b1c0
    pub fn TlsAlloc() -> DWORD; // aclui-rust/src/types_i386.rs // gen:67961a36e037
    pub fn TlsFree(arg1: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs // gen:c939accad583
    pub fn TlsGetValue(arg1: DWORD) -> PVOID; // aclui-rust/src/types_i386.rs // gen:b6c80b186b22
    pub fn TlsSetValue(arg1: DWORD, arg2: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs // gen:3adc17cda88c
    pub fn UnhookWindowsHookEx(arg1: HHOOK) -> BOOL; // aclui-rust/src/types_i386.rs // gen:d6dccb9b9b7f
    pub fn UnregisterClassA(arg1: LPCSTR, arg2: HINSTANCE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:d14f916dd30c
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs // gen:0d4f240a125e
    pub fn WindowFromDC(hDC: HDC) -> HWND; // aclui-rust/src/types_i386.rs // gen:9452d0b1c745
    pub fn glDisable(cap: ::core::ffi::c_uint); // opengl32-rust/src/types_i386.rs // gen:6e5f7f32d17a
    pub fn glEnable(cap: ::core::ffi::c_uint); // opengl32-rust/src/types_i386.rs // gen:7349ed7ff3d4
    pub fn lstrcmpiW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:f3454dea79d1
    pub fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs // gen:eea6eb419b40
    pub fn wglCreateContext(arg1: HDC) -> HGLRC; // aclui-rust/src/types_i386.rs // gen:14aa1370340b
    pub fn wglDeleteContext(arg1: HGLRC) -> BOOL; // aclui-rust/src/types_i386.rs // gen:320b16a24694
    pub fn wglGetCurrentContext() -> HGLRC; // aclui-rust/src/types_i386.rs // gen:683129feecac
    pub fn wglGetCurrentDC() -> HDC; // aclui-rust/src/types_i386.rs // gen:0eed3d5b0c29
    pub fn wglMakeCurrent(arg1: HDC, arg2: HGLRC) -> BOOL; // aclui-rust/src/types_i386.rs // gen:17f7f0f730a1
    pub fn wglShareLists(arg1: HGLRC, arg2: HGLRC) -> BOOL; // aclui-rust/src/types_i386.rs // gen:01864271773e
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::d3dwine::utils::C2RustUnnamed_45;
pub use crate::d3dwine::sampler::C2RustUnnamed_58;
pub use crate::d3dwine::utils::C2RustUnnamed_59;
pub use crate::d3dwine::buffer::__WINE_DBCL_TRACE;
pub use crate::d3dwine::directx::adapter_adjust_mapped_memory;
pub use crate::d3dwine::directx::adapter_adjust_memory;
pub use crate::d3dwine::adapter_vk::adapter_vk_copy_bo_address;
pub use crate::d3dwine::ffp_gl::compile_state_table;
pub use crate::d3dwine::utils::compute_normal_matrix;
pub use crate::d3dwine::surface::cpu_blitter_clear_texture;
pub use crate::d3dwine::utils::debug_bo_address;
pub use crate::d3dwine::utils::debug_box;
pub use crate::d3dwine::utils::debug_color;
pub use crate::d3dwine::utils::debug_const_bo_address;
pub use crate::d3dwine::utils::debug_d3ddeclmethod;
pub use crate::d3dwine::utils::debug_d3ddeclusage;
pub use crate::d3dwine::utils::debug_d3dformat;
pub use crate::d3dwine::utils::debug_d3dinput_classification;
pub use crate::d3dwine::utils::debug_d3dprimitivetype;
pub use crate::d3dwine::utils::debug_d3dresourcetype;
pub use crate::d3dwine::shader::debug_d3dshaderinstructionhandler;
pub use crate::d3dwine::utils::debug_d3dstate;
pub use crate::d3dwine::utils::debug_d3dtexturefiltertype;
pub use crate::d3dwine::utils::debug_d3dusage;
pub use crate::d3dwine::utils::debug_fboattachment;
pub use crate::d3dwine::utils::debug_fbostatus;
pub use crate::d3dwine::utils::debug_glerror;
pub use crate::d3dwine::utils::debug_shader_type;
pub use crate::d3dwine::utils::debug_uvec4;
pub use crate::d3dwine::device::device_context_remove;
pub use crate::d3dwine::device::device_invalidate_state;
pub use crate::d3dwine::shader::find_ds_compile_args;
pub use crate::d3dwine::shader::find_gs_compile_args;
pub use crate::d3dwine::shader::find_ps_compile_args;
pub use crate::d3dwine::shader::find_vs_compile_args;
pub use crate::d3dwine::glsl_shader::glsl_sample_function;
pub use crate::d3dwine::ffp_gl::misc_state_template_gl;
pub use crate::d3dwine::utils::multiply_matrix;
pub use crate::d3dwine::utils::pixelformat_for_depth;
pub use crate::d3dwine::shader::pixelshader_get_resource_type;
pub use crate::d3dwine::resource::resource_cleanup;
pub use crate::d3dwine::resource::resource_init;
pub use crate::d3dwine::resource::resource_offset_map_pointer;
pub use crate::d3dwine::resource::resource_unload;
pub use crate::d3dwine::shader::shader_generate_code;
pub use crate::d3dwine::shader::shader_get_stream_output_register_info;
pub use crate::d3dwine::shader::shader_match_semantic;
pub use crate::d3dwine::shader::shader_vaddline;
pub use crate::d3dwine::stateblock::state_cleanup;
pub use crate::d3dwine::stateblock::state_init;
pub use crate::d3dwine::stateblock::state_unbind_resources;
pub use crate::d3dwine::shader::string_buffer_clear;
pub use crate::d3dwine::shader::string_buffer_free;
pub use crate::d3dwine::shader::string_buffer_get;
pub use crate::d3dwine::shader::string_buffer_init;
pub use crate::d3dwine::shader::string_buffer_list_cleanup;
pub use crate::d3dwine::shader::string_buffer_list_init;
pub use crate::d3dwine::shader::string_buffer_release;
pub use crate::d3dwine::shader::string_buffer_resize;
pub use crate::d3dwine::swapchain::swapchain_set_max_frame_latency;
pub use crate::d3dwine::surface::texture2d_blt;
pub use crate::d3dwine::texture::texture2d_get_blt_info;
pub use crate::d3dwine::surface::texture2d_read_from_framebuffer;
pub use crate::d3dwine::resource::vk_access_mask_from_bind_flags;
pub use crate::d3dwine::adapter_vk::vk_access_mask_from_buffer_usage;
pub use crate::d3dwine::view::vk_image_view_type_from_wined3d;
pub use crate::d3dwine::resource::vk_pipeline_stage_mask_from_bind_flags;
pub use crate::d3dwine::adapter_vk::vk_pipeline_stage_mask_from_buffer_usage;
pub use crate::d3dwine::shader::vkd3d_shader_visibility_from_wined3d;
pub use crate::d3dwine::shader::vshader_get_input;
pub use crate::d3dwine::adapter_vk::wined3d_adapter_vk_get_memory_type_index;
pub use crate::d3dwine::utils::wined3d_allocator_allocate;
pub use crate::d3dwine::utils::wined3d_allocator_block_free;
pub use crate::d3dwine::utils::wined3d_allocator_chunk_cleanup;
pub use crate::d3dwine::utils::wined3d_allocator_chunk_init;
pub use crate::d3dwine::utils::wined3d_allocator_cleanup;
pub use crate::d3dwine::utils::wined3d_allocator_init;
pub use crate::d3dwine::utils::wined3d_array_reserve;
pub use crate::d3dwine::device::wined3d_blend_state_decref;
pub use crate::d3dwine::device::wined3d_blend_state_incref;
pub use crate::d3dwine::adapter_vk::wined3d_bo_slab_vk_unmap;
pub use crate::d3dwine::buffer::wined3d_buffer_decref;
pub use crate::d3dwine::buffer::wined3d_buffer_vk_barrier;
pub use crate::d3dwine::context_gl::wined3d_check_gl_call;
pub use crate::d3dwine::utils::wined3d_clip_blit;
pub use crate::d3dwine::context_gl::wined3d_context_gl_destroy_bo;
pub use crate::d3dwine::context_gl::wined3d_context_gl_end_transform_feedback;
pub use crate::d3dwine::context_vk::wined3d_context_vk_create_bo;
pub use crate::d3dwine::context_vk::wined3d_context_vk_destroy_bo;
pub use crate::d3dwine::context_vk::wined3d_context_vk_end_current_render_pass;
pub use crate::d3dwine::context_vk::wined3d_context_vk_get_command_buffer;
pub use crate::d3dwine::surface::wined3d_cpu_blitter_create;
pub use crate::d3dwine::cs::wined3d_cs_destroy_object;
pub use crate::d3dwine::utils::wined3d_debug_bind_flags;
pub use crate::d3dwine::utils::wined3d_debug_location;
pub use crate::d3dwine::utils::wined3d_debug_resource_access;
pub use crate::d3dwine::utils::wined3d_debug_view_desc;
pub use crate::d3dwine::utils::wined3d_debug_vkresult;
pub use crate::d3dwine::directx::wined3d_decref;
pub use crate::d3dwine::device::wined3d_depth_stencil_state_decref;
pub use crate::d3dwine::device::wined3d_depth_stencil_state_incref;
pub use crate::d3dwine::stateblock::wined3d_device_apply_stateblock;
pub use crate::d3dwine::texture::wined3d_device_context_blt;
pub use crate::d3dwine::cs::wined3d_device_context_emit_update_sub_resource;
pub use crate::d3dwine::device::wined3d_device_context_reset_state;
pub use crate::d3dwine::device::wined3d_device_context_set_state;
pub use crate::d3dwine::device::wined3d_device_gl_create_bo;
pub use crate::d3dwine::device::wined3d_device_gl_get_backup_dc;
pub use crate::d3dwine::device::wined3d_device_gl_get_memory_type_flags;
pub use crate::d3dwine::texture::wined3d_fbo_blitter_create;
pub use crate::d3dwine::query::wined3d_fence_create;
pub use crate::d3dwine::query::wined3d_fence_destroy;
pub use crate::d3dwine::query::wined3d_fence_issue;
pub use crate::d3dwine::query::wined3d_fence_test;
pub use crate::d3dwine::query::wined3d_fence_wait;
pub use crate::d3dwine::texture::wined3d_ffp_blitter_create;
pub use crate::d3dwine::utils::wined3d_ffp_frag_program_key_compare;
pub use crate::d3dwine::utils::wined3d_ffp_vertex_program_key_compare;
pub use crate::d3dwine::wined3d_main::wined3d_filter_messages;
pub use crate::d3dwine::utils::wined3d_format_calculate_pitch;
pub use crate::d3dwine::utils::wined3d_format_calculate_size;
pub use crate::d3dwine::utils::wined3d_format_colour_to_vk;
pub use crate::d3dwine::utils::wined3d_format_convert_from_float;
pub use crate::d3dwine::utils::wined3d_format_copy_data;
pub use crate::d3dwine::utils::wined3d_format_get_color_key_conversion;
pub use crate::d3dwine::utils::wined3d_format_is_depth_view;
pub use crate::d3dwine::utils::wined3d_format_pack;
pub use crate::d3dwine::utils::wined3d_formats_are_srgb_variants;
pub use crate::d3dwine::directx::wined3d_get_device_caps;
pub use crate::d3dwine::utils::wined3d_get_format;
pub use crate::d3dwine::utils::wined3d_get_typed_format_id;
pub use crate::d3dwine::utils::wined3d_gl_limits_get_texture_unit_range;
pub use crate::d3dwine::utils::wined3d_gl_limits_get_uniform_block_range;
pub use crate::d3dwine::glsl_shader::wined3d_glsl_blitter_create;
pub use crate::d3dwine::directx::wined3d_incref;
pub use crate::d3dwine::directx::wined3d_init;
pub use crate::d3dwine::stateblock::wined3d_light_state_enable_light;
pub use crate::d3dwine::stateblock::wined3d_light_state_get_light;
pub use crate::d3dwine::wined3d_main::wined3d_mutex_lock;
pub use crate::d3dwine::wined3d_main::wined3d_mutex_unlock;
pub use crate::d3dwine::directx::wined3d_output_find_closest_matching_mode;
pub use crate::d3dwine::directx::wined3d_output_get_desc;
pub use crate::d3dwine::directx::wined3d_output_get_display_mode;
pub use crate::d3dwine::directx::wined3d_output_get_gamma_ramp;
pub use crate::d3dwine::directx::wined3d_output_get_raster_status;
pub use crate::d3dwine::directx::wined3d_output_set_display_mode;
pub use crate::d3dwine::directx::wined3d_output_set_gamma_ramp;
pub use crate::d3dwine::query::wined3d_query_decref;
pub use crate::d3dwine::query::wined3d_query_incref;
pub use crate::d3dwine::query::wined3d_query_pool_vk_allocate_query;
pub use crate::d3dwine::query::wined3d_query_pool_vk_cleanup;
pub use crate::d3dwine::query::wined3d_query_pool_vk_init;
pub use crate::d3dwine::query::wined3d_query_pool_vk_mark_free;
pub use crate::d3dwine::query::wined3d_query_vk_resume;
pub use crate::d3dwine::query::wined3d_query_vk_suspend;
pub use crate::d3dwine::device::wined3d_rasterizer_state_decref;
pub use crate::d3dwine::device::wined3d_rasterizer_state_incref;
pub use crate::d3dwine::texture::wined3d_raw_blitter_create;
pub use crate::d3dwine::wined3d_main::wined3d_register_window;
pub use crate::d3dwine::utils::wined3d_release_dc;
pub use crate::d3dwine::view::wined3d_rendertarget_view_create;
pub use crate::d3dwine::view::wined3d_rendertarget_view_decref;
pub use crate::d3dwine::view::wined3d_rendertarget_view_get_drawable_size;
pub use crate::d3dwine::view::wined3d_rendertarget_view_get_locations;
pub use crate::d3dwine::view::wined3d_rendertarget_view_incref;
pub use crate::d3dwine::view::wined3d_rendertarget_view_invalidate_location;
pub use crate::d3dwine::view::wined3d_rendertarget_view_load_location;
pub use crate::d3dwine::view::wined3d_rendertarget_view_prepare_location;
pub use crate::d3dwine::view::wined3d_rendertarget_view_validate_location;
pub use crate::d3dwine::resource::wined3d_resource_check_box_dimensions;
pub use crate::d3dwine::resource::wined3d_resource_free_sysmem;
pub use crate::d3dwine::resource::wined3d_resource_get_sample_count;
pub use crate::d3dwine::resource::wined3d_resource_gl_legacy_map_flags;
pub use crate::d3dwine::resource::wined3d_resource_gl_map_flags;
pub use crate::d3dwine::resource::wined3d_resource_gl_storage_flags;
pub use crate::d3dwine::resource::wined3d_resource_is_offscreen;
pub use crate::d3dwine::resource::wined3d_resource_map;
pub use crate::d3dwine::resource::wined3d_resource_prepare_sysmem;
pub use crate::d3dwine::resource::wined3d_resource_unmap;
pub use crate::d3dwine::directx::wined3d_restore_display_modes;
pub use crate::d3dwine::sampler::wined3d_sampler_create;
pub use crate::d3dwine::sampler::wined3d_sampler_decref;
pub use crate::d3dwine::sampler::wined3d_sampler_incref;
pub use crate::d3dwine::shader::wined3d_shader_create_ffp_ps;
pub use crate::d3dwine::shader::wined3d_shader_create_ffp_vs;
pub use crate::d3dwine::shader::wined3d_shader_decref;
pub use crate::d3dwine::shader::wined3d_shader_incref;
pub use crate::d3dwine::view::wined3d_shader_resource_view_create;
pub use crate::d3dwine::view::wined3d_shader_resource_view_decref;
pub use crate::d3dwine::view::wined3d_shader_resource_view_destroy;
pub use crate::d3dwine::view::wined3d_shader_resource_view_gl_bind;
pub use crate::d3dwine::view::wined3d_shader_resource_view_gl_update;
pub use crate::d3dwine::view::wined3d_shader_resource_view_incref;
pub use crate::d3dwine::view::wined3d_shader_resource_view_vk_update_buffer;
pub use crate::d3dwine::view::wined3d_shader_resource_view_vk_update_layout;
pub use crate::d3dwine::shader::wined3d_srgb_const;
pub use crate::d3dwine::stateblock::wined3d_state_create;
pub use crate::d3dwine::stateblock::wined3d_state_destroy;
pub use crate::d3dwine::stateblock::wined3d_stateblock_get_state;
pub use crate::d3dwine::swapchain::wined3d_swapchain_activate;
pub use crate::d3dwine::swapchain::wined3d_swapchain_get_display_mode;
pub use crate::d3dwine::swapchain::wined3d_swapchain_get_gamma_ramp;
pub use crate::d3dwine::swapchain::wined3d_swapchain_get_raster_status;
pub use crate::d3dwine::swapchain::wined3d_swapchain_gl_get_context;
pub use crate::d3dwine::swapchain::wined3d_swapchain_resize_buffers;
pub use crate::d3dwine::swapchain::wined3d_swapchain_set_gamma_ramp;
pub use crate::d3dwine::swapchain::wined3d_swapchain_set_window;
pub use crate::d3dwine::wined3d_main::wined3d_swapchain_state_cleanup;
pub use crate::d3dwine::wined3d_main::wined3d_swapchain_state_register;
pub use crate::d3dwine::swapchain::wined3d_swapchain_state_set_fullscreen;
pub use crate::d3dwine::swapchain::wined3d_swapchain_state_setup_fullscreen;
pub use crate::d3dwine::texture::wined3d_texture_add_dirty_region;
pub use crate::d3dwine::texture::wined3d_texture_can_use_pbo;
pub use crate::d3dwine::texture::wined3d_texture_clear_dirty_regions;
pub use crate::d3dwine::texture::wined3d_texture_create;
pub use crate::d3dwine::texture::wined3d_texture_decref;
pub use crate::d3dwine::texture::wined3d_texture_from_resource;
pub use crate::d3dwine::texture::wined3d_texture_get_bo_address;
pub use crate::d3dwine::texture::wined3d_texture_get_dc;
pub use crate::d3dwine::texture::wined3d_texture_get_gl_buffer;
pub use crate::d3dwine::texture::wined3d_texture_get_pitch;
pub use crate::d3dwine::texture::wined3d_texture_gl_apply_sampler_desc;
pub use crate::d3dwine::texture::wined3d_texture_gl_set_compatible_renderbuffer;
pub use crate::d3dwine::texture::wined3d_texture_incref;
pub use crate::d3dwine::texture::wined3d_texture_invalidate_location;
pub use crate::d3dwine::texture::wined3d_texture_load;
pub use crate::d3dwine::texture::wined3d_texture_load_location;
pub use crate::d3dwine::texture::wined3d_texture_prepare_location;
pub use crate::d3dwine::texture::wined3d_texture_release_dc;
pub use crate::d3dwine::texture::wined3d_texture_set_color_key;
pub use crate::d3dwine::texture::wined3d_texture_set_swapchain;
pub use crate::d3dwine::texture::wined3d_texture_update_sub_resource;
pub use crate::d3dwine::texture::wined3d_texture_validate_location;
pub use crate::d3dwine::texture::wined3d_texture_validate_sub_resource_idx;
pub use crate::d3dwine::texture::wined3d_texture_vk_barrier;
pub use crate::d3dwine::texture::wined3d_texture_vk_get_default_image_info;
pub use crate::d3dwine::texture::wined3d_texture_vk_make_generic;
pub use crate::d3dwine::view::wined3d_unordered_access_view_copy_counter;
pub use crate::d3dwine::view::wined3d_unordered_access_view_decref;
pub use crate::d3dwine::view::wined3d_unordered_access_view_gl_update;
pub use crate::d3dwine::view::wined3d_unordered_access_view_incref;
pub use crate::d3dwine::view::wined3d_unordered_access_view_invalidate_location;
pub use crate::d3dwine::view::wined3d_unordered_access_view_set_counter;
pub use crate::d3dwine::view::wined3d_unordered_access_view_vk_update;
pub use crate::d3dwine::wined3d_main::wined3d_unregister_window;
pub use crate::d3dwine::vertexdeclaration::wined3d_vertex_declaration_decref;
pub use crate::d3dwine::vertexdeclaration::wined3d_vertex_declaration_incref;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbLengthW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};
pub use winelist_rust::{list_add_after, list_add_before, list_add_head, list_add_tail, list_count, list_empty, list_head, list_init, list_move_head, list_move_tail, list_next, list_prev, list_remove, list_tail};

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// GLsync: c2rust, query.c
pub type GLsync = *mut __GLsync;
// VkCommandBuffer: c2rust, adapter_vk.c
pub type VkCommandBuffer = *mut VkCommandBuffer_T;
// VkDevice: c2rust, adapter_vk.c
pub type VkDevice = *mut VkDevice_T;
// VkInstance: c2rust, adapter_vk.c
pub type VkInstance = *mut VkInstance_T;
// VkPhysicalDevice: c2rust, adapter_vk.c
pub type VkPhysicalDevice = *mut VkPhysicalDevice_T;
// blit_vertex: c2rust, context_gl.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct blit_vertex {
    pub x: ::core::ffi::c_float,
    pub y: ::core::ffi::c_float,
    pub texcoord: wined3d_vec3,
}
// driver_quirk: c2rust, adapter_gl.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct driver_quirk {
    pub match_: Option<
        unsafe extern "C" fn(
            *const wined3d_gl_info,
            *mut wined3d_caps_gl_ctx,
            *const ::core::ffi::c_char,
            wined3d_gl_vendor,
            wined3d_pci_vendor,
            wined3d_pci_device,
        ) -> BOOL,
    >,
    pub apply: Option<unsafe extern "C" fn(*mut wined3d_gl_info) -> ()>,
    pub description: *const ::core::ffi::c_char,
}
// limits_entry: c2rust, shader.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct limits_entry {
    pub min_version: ::core::ffi::c_uint,
    pub max_version: ::core::ffi::c_uint,
    pub limits: wined3d_shader_limits,
}
// ssize_t: c2rust, directx.c
pub type ssize_t = isize;
// stage_state: c2rust, stateblock.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stage_state {
    pub stage: ::core::ffi::c_uint,
    pub state: ::core::ffi::c_uint,
}
// wined3d_command_list: c2rust, cs.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wined3d_command_list {
    pub refcount: LONG,
    pub device: *mut wined3d_device,
    pub data_size: SIZE_T,
    pub data: *mut ::core::ffi::c_void,
    pub resource_count: SIZE_T,
    pub resources: *mut *mut wined3d_resource,
    pub upload_count: SIZE_T,
    pub uploads: *mut wined3d_deferred_upload,
    pub upload_heap: HANDLE,
    pub upload_heap_refcount: *mut LONG,
    pub command_list_count: SIZE_T,
    pub command_lists: *mut *mut wined3d_command_list,
    pub query_count: SIZE_T,
    pub queries: *mut wined3d_deferred_query_issue,
}
// wined3d_dc_info: c2rust, swapchain.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wined3d_dc_info {
    pub bitmap: HBITMAP,
    pub dc: HDC,
}
// wined3d_deferred_query_issue: c2rust, cs.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wined3d_deferred_query_issue {
    pub query: *mut wined3d_query,
    pub flags: ::core::ffi::c_uint,
}
// wined3d_deferred_upload: c2rust, cs.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wined3d_deferred_upload {
    pub resource: *mut wined3d_resource,
    pub sub_resource_idx: ::core::ffi::c_uint,
    pub bo: *mut wined3d_bo,
    pub sysmem: *mut uint8_t,
    pub map_ptr: *mut uint8_t,
    pub box_: wined3d_box,
    pub upload_flags: uint32_t,
}
// wined3d_dirty_regions: c2rust, device.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wined3d_dirty_regions {
    pub boxes: *mut wined3d_box,
    pub boxes_size: SIZE_T,
    pub box_count: ::core::ffi::c_uint,
}
// wined3d_fallback_card: c2rust, directx.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wined3d_fallback_card {
    pub feature_level: wined3d_feature_level,
    pub device_id: wined3d_pci_device,
}
// wined3d_fbo_entry_key: c2rust, context_gl.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wined3d_fbo_entry_key {
    pub rb_namespace: uint32_t,
    pub objects: [wined3d_fbo_resource; 9],
}
// wined3d_ffp_light_constants: c2rust, stateblock.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wined3d_ffp_light_constants {
    pub ambient: wined3d_color,
    pub lights: [wined3d_light_constants; 8],
}
// wined3d_ffp_point_constants: c2rust, stateblock.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wined3d_ffp_point_constants {
    pub scale_const: ::core::ffi::c_float,
    pub scale_linear: ::core::ffi::c_float,
    pub scale_quad: ::core::ffi::c_float,
    pub size: ::core::ffi::c_float,
}
// wined3d_gl_vendor: c2rust, adapter_gl.c
pub type wined3d_gl_vendor = ::core::ffi::c_uint;
// wined3d_overlay_info: c2rust, texture.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wined3d_overlay_info {
    pub entry: list,
    pub overlays: list,
    pub dst_texture: *mut wined3d_texture,
    pub dst_sub_resource_idx: ::core::ffi::c_uint,
    pub src_rect: RECT,
    pub dst_rect: RECT,
}
// wined3d_rendertarget_blend_state_desc: c2rust, context_vk.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wined3d_rendertarget_blend_state_desc {
    pub enable: BOOL,
    pub src: wined3d_blend,
    pub dst: wined3d_blend,
    pub op: wined3d_blend_op,
    pub src_alpha: wined3d_blend,
    pub dst_alpha: wined3d_blend,
    pub op_alpha: wined3d_blend_op,
    pub writemask: ::core::ffi::c_uint,
}
// wined3d_retired_block_gl: c2rust, context_gl.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wined3d_retired_block_gl {
    pub block: *mut wined3d_allocator_block,
    pub fence_id: uint64_t,
}
// wined3d_saved_states: c2rust, stateblock.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wined3d_saved_states {
    pub changed_lights: list,
    pub vs_consts_f: [uint32_t; 8],
    pub vertexShaderConstantsI: uint16_t,
    pub vertexShaderConstantsB: uint16_t,
    pub ps_consts_f: [uint32_t; 7],
    pub pixelShaderConstantsI: uint16_t,
    pub pixelShaderConstantsB: uint16_t,
    pub transform: [uint32_t; 16],
    pub streamSource: uint16_t,
    pub streamFreq: uint16_t,
    pub renderState: [uint32_t; 7],
    pub textureState: [uint32_t; 8],
    pub samplerState: [uint16_t; 20],
    pub clipplane: uint32_t,
    pub textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants:
        [u8; 5],
    pub c2rust_padding: [u8; 3],
}
impl wined3d_saved_states {
    #[inline]
    pub fn textures(&self) -> uint32_t {
        let s = &self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut v: u64 = 0;
        let mut i = 0usize;
        while i < 20 {
            let b = 0 + i;
            v |= (((s[b / 8] >> (b % 8)) & 1) as u64) << i;
            i += 1;
        }
        v as uint32_t
    }
    #[inline]
    pub fn set_textures(&mut self, value: uint32_t) {
        let v = value as u64;
        let s = &mut self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut i = 0usize;
        while i < 20 {
            let b = 0 + i;
            let bit = (((v >> i) & 1) as u8) << (b % 8);
            s[b / 8] = (s[b / 8] & !(1u8 << (b % 8))) | bit;
            i += 1;
        }
    }
    #[inline]
    pub fn indices(&self) -> uint32_t {
        let s = &self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut v: u64 = 0;
        let mut i = 0usize;
        while i < 1 {
            let b = 20 + i;
            v |= (((s[b / 8] >> (b % 8)) & 1) as u64) << i;
            i += 1;
        }
        v as uint32_t
    }
    #[inline]
    pub fn set_indices(&mut self, value: uint32_t) {
        let v = value as u64;
        let s = &mut self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut i = 0usize;
        while i < 1 {
            let b = 20 + i;
            let bit = (((v >> i) & 1) as u8) << (b % 8);
            s[b / 8] = (s[b / 8] & !(1u8 << (b % 8))) | bit;
            i += 1;
        }
    }
    #[inline]
    pub fn material(&self) -> uint32_t {
        let s = &self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut v: u64 = 0;
        let mut i = 0usize;
        while i < 1 {
            let b = 21 + i;
            v |= (((s[b / 8] >> (b % 8)) & 1) as u64) << i;
            i += 1;
        }
        v as uint32_t
    }
    #[inline]
    pub fn set_material(&mut self, value: uint32_t) {
        let v = value as u64;
        let s = &mut self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut i = 0usize;
        while i < 1 {
            let b = 21 + i;
            let bit = (((v >> i) & 1) as u8) << (b % 8);
            s[b / 8] = (s[b / 8] & !(1u8 << (b % 8))) | bit;
            i += 1;
        }
    }
    #[inline]
    pub fn viewport(&self) -> uint32_t {
        let s = &self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut v: u64 = 0;
        let mut i = 0usize;
        while i < 1 {
            let b = 22 + i;
            v |= (((s[b / 8] >> (b % 8)) & 1) as u64) << i;
            i += 1;
        }
        v as uint32_t
    }
    #[inline]
    pub fn set_viewport(&mut self, value: uint32_t) {
        let v = value as u64;
        let s = &mut self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut i = 0usize;
        while i < 1 {
            let b = 22 + i;
            let bit = (((v >> i) & 1) as u8) << (b % 8);
            s[b / 8] = (s[b / 8] & !(1u8 << (b % 8))) | bit;
            i += 1;
        }
    }
    #[inline]
    pub fn vertexDecl(&self) -> uint32_t {
        let s = &self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut v: u64 = 0;
        let mut i = 0usize;
        while i < 1 {
            let b = 23 + i;
            v |= (((s[b / 8] >> (b % 8)) & 1) as u64) << i;
            i += 1;
        }
        v as uint32_t
    }
    #[inline]
    pub fn set_vertexDecl(&mut self, value: uint32_t) {
        let v = value as u64;
        let s = &mut self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut i = 0usize;
        while i < 1 {
            let b = 23 + i;
            let bit = (((v >> i) & 1) as u8) << (b % 8);
            s[b / 8] = (s[b / 8] & !(1u8 << (b % 8))) | bit;
            i += 1;
        }
    }
    #[inline]
    pub fn pixelShader(&self) -> uint32_t {
        let s = &self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut v: u64 = 0;
        let mut i = 0usize;
        while i < 1 {
            let b = 24 + i;
            v |= (((s[b / 8] >> (b % 8)) & 1) as u64) << i;
            i += 1;
        }
        v as uint32_t
    }
    #[inline]
    pub fn set_pixelShader(&mut self, value: uint32_t) {
        let v = value as u64;
        let s = &mut self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut i = 0usize;
        while i < 1 {
            let b = 24 + i;
            let bit = (((v >> i) & 1) as u8) << (b % 8);
            s[b / 8] = (s[b / 8] & !(1u8 << (b % 8))) | bit;
            i += 1;
        }
    }
    #[inline]
    pub fn vertexShader(&self) -> uint32_t {
        let s = &self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut v: u64 = 0;
        let mut i = 0usize;
        while i < 1 {
            let b = 25 + i;
            v |= (((s[b / 8] >> (b % 8)) & 1) as u64) << i;
            i += 1;
        }
        v as uint32_t
    }
    #[inline]
    pub fn set_vertexShader(&mut self, value: uint32_t) {
        let v = value as u64;
        let s = &mut self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut i = 0usize;
        while i < 1 {
            let b = 25 + i;
            let bit = (((v >> i) & 1) as u8) << (b % 8);
            s[b / 8] = (s[b / 8] & !(1u8 << (b % 8))) | bit;
            i += 1;
        }
    }
    #[inline]
    pub fn scissorRect(&self) -> uint32_t {
        let s = &self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut v: u64 = 0;
        let mut i = 0usize;
        while i < 1 {
            let b = 26 + i;
            v |= (((s[b / 8] >> (b % 8)) & 1) as u64) << i;
            i += 1;
        }
        v as uint32_t
    }
    #[inline]
    pub fn set_scissorRect(&mut self, value: uint32_t) {
        let v = value as u64;
        let s = &mut self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut i = 0usize;
        while i < 1 {
            let b = 26 + i;
            let bit = (((v >> i) & 1) as u8) << (b % 8);
            s[b / 8] = (s[b / 8] & !(1u8 << (b % 8))) | bit;
            i += 1;
        }
    }
    #[inline]
    pub fn store_stream_offset(&self) -> uint32_t {
        let s = &self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut v: u64 = 0;
        let mut i = 0usize;
        while i < 1 {
            let b = 27 + i;
            v |= (((s[b / 8] >> (b % 8)) & 1) as u64) << i;
            i += 1;
        }
        v as uint32_t
    }
    #[inline]
    pub fn set_store_stream_offset(&mut self, value: uint32_t) {
        let v = value as u64;
        let s = &mut self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut i = 0usize;
        while i < 1 {
            let b = 27 + i;
            let bit = (((v >> i) & 1) as u8) << (b % 8);
            s[b / 8] = (s[b / 8] & !(1u8 << (b % 8))) | bit;
            i += 1;
        }
    }
    #[inline]
    pub fn alpha_to_coverage(&self) -> uint32_t {
        let s = &self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut v: u64 = 0;
        let mut i = 0usize;
        while i < 1 {
            let b = 28 + i;
            v |= (((s[b / 8] >> (b % 8)) & 1) as u64) << i;
            i += 1;
        }
        v as uint32_t
    }
    #[inline]
    pub fn set_alpha_to_coverage(&mut self, value: uint32_t) {
        let v = value as u64;
        let s = &mut self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut i = 0usize;
        while i < 1 {
            let b = 28 + i;
            let bit = (((v >> i) & 1) as u8) << (b % 8);
            s[b / 8] = (s[b / 8] & !(1u8 << (b % 8))) | bit;
            i += 1;
        }
    }
    #[inline]
    pub fn lights(&self) -> uint32_t {
        let s = &self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut v: u64 = 0;
        let mut i = 0usize;
        while i < 1 {
            let b = 29 + i;
            v |= (((s[b / 8] >> (b % 8)) & 1) as u64) << i;
            i += 1;
        }
        v as uint32_t
    }
    #[inline]
    pub fn set_lights(&mut self, value: uint32_t) {
        let v = value as u64;
        let s = &mut self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut i = 0usize;
        while i < 1 {
            let b = 29 + i;
            let bit = (((v >> i) & 1) as u8) << (b % 8);
            s[b / 8] = (s[b / 8] & !(1u8 << (b % 8))) | bit;
            i += 1;
        }
    }
    #[inline]
    pub fn transforms(&self) -> uint32_t {
        let s = &self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut v: u64 = 0;
        let mut i = 0usize;
        while i < 1 {
            let b = 30 + i;
            v |= (((s[b / 8] >> (b % 8)) & 1) as u64) << i;
            i += 1;
        }
        v as uint32_t
    }
    #[inline]
    pub fn set_transforms(&mut self, value: uint32_t) {
        let v = value as u64;
        let s = &mut self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut i = 0usize;
        while i < 1 {
            let b = 30 + i;
            let bit = (((v >> i) & 1) as u8) << (b % 8);
            s[b / 8] = (s[b / 8] & !(1u8 << (b % 8))) | bit;
            i += 1;
        }
    }
    #[inline]
    pub fn ffp_ps_constants(&self) -> uint32_t {
        let s = &self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut v: u64 = 0;
        let mut i = 0usize;
        while i < 1 {
            let b = 31 + i;
            v |= (((s[b / 8] >> (b % 8)) & 1) as u64) << i;
            i += 1;
        }
        v as uint32_t
    }
    #[inline]
    pub fn set_ffp_ps_constants(&mut self, value: uint32_t) {
        let v = value as u64;
        let s = &mut self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut i = 0usize;
        while i < 1 {
            let b = 31 + i;
            let bit = (((v >> i) & 1) as u8) << (b % 8);
            s[b / 8] = (s[b / 8] & !(1u8 << (b % 8))) | bit;
            i += 1;
        }
    }
    #[inline]
    pub fn texture_matrices(&self) -> uint32_t {
        let s = &self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut v: u64 = 0;
        let mut i = 0usize;
        while i < 1 {
            let b = 32 + i;
            v |= (((s[b / 8] >> (b % 8)) & 1) as u64) << i;
            i += 1;
        }
        v as uint32_t
    }
    #[inline]
    pub fn set_texture_matrices(&mut self, value: uint32_t) {
        let v = value as u64;
        let s = &mut self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut i = 0usize;
        while i < 1 {
            let b = 32 + i;
            let bit = (((v >> i) & 1) as u8) << (b % 8);
            s[b / 8] = (s[b / 8] & !(1u8 << (b % 8))) | bit;
            i += 1;
        }
    }
    #[inline]
    pub fn modelview_matrices(&self) -> uint32_t {
        let s = &self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut v: u64 = 0;
        let mut i = 0usize;
        while i < 1 {
            let b = 33 + i;
            v |= (((s[b / 8] >> (b % 8)) & 1) as u64) << i;
            i += 1;
        }
        v as uint32_t
    }
    #[inline]
    pub fn set_modelview_matrices(&mut self, value: uint32_t) {
        let v = value as u64;
        let s = &mut self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut i = 0usize;
        while i < 1 {
            let b = 33 + i;
            let bit = (((v >> i) & 1) as u8) << (b % 8);
            s[b / 8] = (s[b / 8] & !(1u8 << (b % 8))) | bit;
            i += 1;
        }
    }
    #[inline]
    pub fn point_scale(&self) -> uint32_t {
        let s = &self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut v: u64 = 0;
        let mut i = 0usize;
        while i < 1 {
            let b = 34 + i;
            v |= (((s[b / 8] >> (b % 8)) & 1) as u64) << i;
            i += 1;
        }
        v as uint32_t
    }
    #[inline]
    pub fn set_point_scale(&mut self, value: uint32_t) {
        let v = value as u64;
        let s = &mut self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut i = 0usize;
        while i < 1 {
            let b = 34 + i;
            let bit = (((v >> i) & 1) as u8) << (b % 8);
            s[b / 8] = (s[b / 8] & !(1u8 << (b % 8))) | bit;
            i += 1;
        }
    }
    #[inline]
    pub fn ffp_vs_settings(&self) -> uint32_t {
        let s = &self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut v: u64 = 0;
        let mut i = 0usize;
        while i < 1 {
            let b = 35 + i;
            v |= (((s[b / 8] >> (b % 8)) & 1) as u64) << i;
            i += 1;
        }
        v as uint32_t
    }
    #[inline]
    pub fn set_ffp_vs_settings(&mut self, value: uint32_t) {
        let v = value as u64;
        let s = &mut self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut i = 0usize;
        while i < 1 {
            let b = 35 + i;
            let bit = (((v >> i) & 1) as u8) << (b % 8);
            s[b / 8] = (s[b / 8] & !(1u8 << (b % 8))) | bit;
            i += 1;
        }
    }
    #[inline]
    pub fn ffp_ps_settings(&self) -> uint32_t {
        let s = &self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut v: u64 = 0;
        let mut i = 0usize;
        while i < 1 {
            let b = 36 + i;
            v |= (((s[b / 8] >> (b % 8)) & 1) as u64) << i;
            i += 1;
        }
        v as uint32_t
    }
    #[inline]
    pub fn set_ffp_ps_settings(&mut self, value: uint32_t) {
        let v = value as u64;
        let s = &mut self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut i = 0usize;
        while i < 1 {
            let b = 36 + i;
            let bit = (((v >> i) & 1) as u8) << (b % 8);
            s[b / 8] = (s[b / 8] & !(1u8 << (b % 8))) | bit;
            i += 1;
        }
    }
    #[inline]
    pub fn rasterizer_state(&self) -> uint32_t {
        let s = &self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut v: u64 = 0;
        let mut i = 0usize;
        while i < 1 {
            let b = 37 + i;
            v |= (((s[b / 8] >> (b % 8)) & 1) as u64) << i;
            i += 1;
        }
        v as uint32_t
    }
    #[inline]
    pub fn set_rasterizer_state(&mut self, value: uint32_t) {
        let v = value as u64;
        let s = &mut self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut i = 0usize;
        while i < 1 {
            let b = 37 + i;
            let bit = (((v >> i) & 1) as u8) << (b % 8);
            s[b / 8] = (s[b / 8] & !(1u8 << (b % 8))) | bit;
            i += 1;
        }
    }
    #[inline]
    pub fn position_transformed(&self) -> uint32_t {
        let s = &self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut v: u64 = 0;
        let mut i = 0usize;
        while i < 1 {
            let b = 38 + i;
            v |= (((s[b / 8] >> (b % 8)) & 1) as u64) << i;
            i += 1;
        }
        v as uint32_t
    }
    #[inline]
    pub fn set_position_transformed(&mut self, value: uint32_t) {
        let v = value as u64;
        let s = &mut self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut i = 0usize;
        while i < 1 {
            let b = 38 + i;
            let bit = (((v >> i) & 1) as u8) << (b % 8);
            s[b / 8] = (s[b / 8] & !(1u8 << (b % 8))) | bit;
            i += 1;
        }
    }
    #[inline]
    pub fn bumpenv_constants(&self) -> uint32_t {
        let s = &self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut v: u64 = 0;
        let mut i = 0usize;
        while i < 1 {
            let b = 39 + i;
            v |= (((s[b / 8] >> (b % 8)) & 1) as u64) << i;
            i += 1;
        }
        v as uint32_t
    }
    #[inline]
    pub fn set_bumpenv_constants(&mut self, value: uint32_t) {
        let v = value as u64;
        let s = &mut self.textures_indices_material_viewport_vertexDecl_pixelShader_vertexShader_scissorRect_store_stream_offset_alpha_to_coverage_lights_transforms_ffp_ps_constants_texture_matrices_modelview_matrices_point_scale_ffp_vs_settings_ffp_ps_settings_rasterizer_state_position_transformed_bumpenv_constants;
        let mut i = 0usize;
        while i < 1 {
            let b = 39 + i;
            let bit = (((v >> i) & 1) as u8) << (b % 8);
            s[b / 8] = (s[b / 8] & !(1u8 << (b % 8))) | bit;
            i += 1;
        }
    }
}
// wined3d_sm4_data_type: c2rust, shader_sm4.c
pub type wined3d_sm4_data_type = ::core::ffi::c_uint;
// wined3d_sm4_dimension: c2rust, shader_sm4.c
pub type wined3d_sm4_dimension = ::core::ffi::c_uint;
// wined3d_sm4_instruction_modifier: c2rust, shader_sm4.c
pub type wined3d_sm4_instruction_modifier = ::core::ffi::c_uint;
// wined3d_sm4_opcode: c2rust, shader_sm4.c
pub type wined3d_sm4_opcode = ::core::ffi::c_uint;
// wined3d_sm4_register_type: c2rust, shader_sm4.c
pub type wined3d_sm4_register_type = ::core::ffi::c_uint;
// wined3d_sm4_resource_type: c2rust, shader_sm4.c
pub type wined3d_sm4_resource_type = ::core::ffi::c_uint;
// wined3d_sm4_shader_data_type: c2rust, shader_sm4.c
pub type wined3d_sm4_shader_data_type = ::core::ffi::c_uint;
// wined3d_sm4_swizzle_type: c2rust, shader_sm4.c
pub type wined3d_sm4_swizzle_type = ::core::ffi::c_uint;
// wined3d_stateblock: c2rust, stateblock.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wined3d_stateblock {
    pub ref_: LONG,
    pub device: *mut wined3d_device,
    pub changed: wined3d_saved_states,
    pub stateblock_state: wined3d_stateblock_state,
    pub light_state: wined3d_light_state,
    pub contained_render_states: [::core::ffi::c_uint; 210],
    pub num_contained_render_states: ::core::ffi::c_uint,
    pub contained_transform_states: [::core::ffi::c_uint; 512],
    pub num_contained_transform_states: ::core::ffi::c_uint,
    pub contained_tss_states: [stage_state; 144],
    pub num_contained_tss_states: ::core::ffi::c_uint,
    pub contained_sampler_states: [stage_state; 260],
    pub num_contained_sampler_states: ::core::ffi::c_uint,
}
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn __wine_dbg_get_channel_flags(channel: *mut __wine_debug_channel) -> ::core::ffi::c_uchar; // adapter_gl.c
    pub fn _snprintf(
        _Dest: *mut ::core::ffi::c_char,
        _Count: size_t,
        _Format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // utils.c
    pub fn atoi(_Str: *const ::core::ffi::c_char) -> ::core::ffi::c_int; // adapter_gl.c
    pub fn getenv(_VarName: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char; // wined3d_main.c
    pub fn ros_dbg_log(
        cls: __wine_debug_class,
        ch: *mut __wine_debug_channel,
        file: *const ::core::ffi::c_char,
        func: *const ::core::ffi::c_char,
        line: ::core::ffi::c_int,
        format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // adapter_gl.c
    pub fn sprintf(
        _Dest: *mut ::core::ffi::c_char,
        _Format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // glsl_shader.c
    pub fn strcat(
        _Dest: *mut ::core::ffi::c_char,
        _Source: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char; // glsl_shader.c
    pub fn strcmp(
        _Str1: *const ::core::ffi::c_char,
        _Str2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int; // adapter_gl.c
    pub fn strcpy(
        _Dest: *mut ::core::ffi::c_char,
        _Source: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char; // utils.c
    pub fn strlen(_Str: *const ::core::ffi::c_char) -> size_t; // adapter_gl.c
    pub fn strpbrk(
        _Str: *const ::core::ffi::c_char,
        _Control: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char; // wined3d_main.c
    pub fn strrchr(
        _Str: *const ::core::ffi::c_char,
        _Ch: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char; // wined3d_main.c
    pub fn strstr(
        _Str: *const ::core::ffi::c_char,
        _SubStr: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char; // adapter_gl.c
    pub fn strtoul(
        _Str: *const ::core::ffi::c_char,
        _EndPtr: *mut *mut ::core::ffi::c_char,
        _Radix: ::core::ffi::c_int,
    ) -> ::core::ffi::c_ulong; // wined3d_main.c
    pub fn vkd3d_set_log_callback(callback: PFN_vkd3d_log); // wined3d_main.c
    pub fn vkd3d_shader_build_varying_map(
        output_signature: *const vkd3d_shader_signature,
        input_signature: *const vkd3d_shader_signature,
        count: *mut ::core::ffi::c_uint,
        varyings: *mut vkd3d_shader_varying_map,
    ); // shader_spirv.c
    pub fn vkd3d_shader_compile(
        compile_info: *const vkd3d_shader_compile_info,
        out: *mut vkd3d_shader_code,
        messages: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int; // ffp_hlsl.c
    pub fn vkd3d_shader_free_dxbc(dxbc: *mut vkd3d_shader_dxbc_desc); // shader_sm4.c
    pub fn vkd3d_shader_free_messages(messages: *mut ::core::ffi::c_char); // ffp_hlsl.c
    pub fn vkd3d_shader_free_scan_descriptor_info(
        scan_descriptor_info: *mut vkd3d_shader_scan_descriptor_info,
    ); // shader_spirv.c
    pub fn vkd3d_shader_free_scan_signature_info(info: *mut vkd3d_shader_scan_signature_info); // shader_spirv.c
    pub fn vkd3d_shader_free_shader_code(code: *mut vkd3d_shader_code); // glsl_shader.c
    pub fn vkd3d_shader_parse_dxbc(
        dxbc: *const vkd3d_shader_code,
        flags: uint32_t,
        desc: *mut vkd3d_shader_dxbc_desc,
        messages: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int; // shader_sm4.c
    pub fn vkd3d_shader_scan(
        compile_info: *const vkd3d_shader_compile_info,
        messages: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int; // shader_spirv.c
}
pub use win32macros_rust::WINE_ERR as ERR; // sdk alias
pub use win32macros_rust::WINE_FIXME as FIXME; // sdk alias
pub use win32macros_rust::WINE_TRACE as TRACE; // sdk alias
pub use win32macros_rust::WINE_WARN as WARN; // sdk alias
pub use crate::prelude::_snprintf as snprintf; // sdk alias
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_d3d: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x64\x33\x64\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch_d3d_perf: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x64\x33\x64\x5f\x70\x65\x72\x66\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch_winediag: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x77\x69\x6e\x65\x64\x69\x61\x67\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch_d3d_sync: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x64\x33\x64\x5f\x73\x79\x6e\x63\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch_fps: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x66\x70\x73\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch_frametime: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x66\x72\x61\x6d\x65\x74\x69\x6d\x65\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch_d3d_shader: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x64\x33\x64\x5f\x73\x68\x61\x64\x65\x72\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch_d3d_bytecode: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x64\x33\x64\x5f\x62\x79\x74\x65\x63\x6f\x64\x65\x00\x00\x00") },
};
pub static mut __wine_dbch_d3d_decl: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x64\x33\x64\x5f\x64\x65\x63\x6c\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch_vkd3d: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x76\x6b\x64\x33\x64\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_d3d };
// ==== end wine debug channels ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! CreateWindowA { ($p0:expr, $p1:expr, $p2:expr, $p3:expr, $p4:expr, $p5:expr, $p6:expr, $p7:expr, $p8:expr, $p9:expr, $p10:expr $(,)?) => { CreateWindowExA(((0) as _), (($p0) as _), (($p1) as _), (($p2) as _), (($p3) as _), (($p4) as _), (($p5) as _), (($p6) as _), (($p7) as _), (($p8) as _), (($p9) as _), (($p10) as _)) }; } // gen:b82d9b081d79
#[macro_export] macro_rules! VK_MAKE_VERSION { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((((($p0) as u32) << 22) | ((($p1) as u32) << 12)) | (($p2) as u32)) }; } // gen:40146e72bff6
#[macro_export] macro_rules! VK_VERSION_MAJOR { ($p0:expr $(,)?) => { ((($p0) as u32) >> 22) }; } // gen:b5a99912ce06
#[macro_export] macro_rules! VK_VERSION_MINOR { ($p0:expr $(,)?) => { (((($p0) as u32) >> 12) & 0x3FF) }; } // gen:1f8d945a708f
#[macro_export] macro_rules! VK_VERSION_PATCH { ($p0:expr $(,)?) => { ((($p0) as u32) & 0xFFF) }; } // gen:678e6932448b
#[macro_export] macro_rules! WINED3D_TS_WORLD_MATRIX { ($p0:expr $(,)?) => { ((($p0 + 256)) as wined3d_transform_state) }; } // gen:874f862f4537
#[macro_export] macro_rules! WINEMAKEFOURCC { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { ((((((($p0) as BYTE)) as DWORD) | ((((($p1) as BYTE)) as DWORD) << 8)) | ((((($p2) as BYTE)) as DWORD) << 16)) | ((((($p3) as BYTE)) as DWORD) << 24)) }; } // gen:bc2c6edbd35b
#[macro_export] macro_rules! isfinite { ($p0:expr $(,)?) => { ((_finite((($p0) as double)) != 0) as c_int) }; } // gen:5c06c60b2e35
#[macro_export] macro_rules! isinf { ($p0:expr $(,)?) => { ((((((((_finite((($p0) as double))) as isize == 0) as c_int)) as isize) != 0) && ((((((_isnan((($p0) as double))) as isize == 0) as c_int)) as isize) != 0)) as c_int) }; } // gen:1a9fd1ff1fc3
#[macro_export] macro_rules! isnan { ($p0:expr $(,)?) => { ((_isnan((($p0) as double)) != 0) as c_int) }; } // gen:193e7c5b14af
#[macro_export] macro_rules! max { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 > $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; } // gen:29bb71d79dfd
#[macro_export] macro_rules! min { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 < $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; } // gen:fb1d8a90940a
#[macro_export] macro_rules! va_end { ($p0:expr $(,)?) => { __builtin_va_end((($p0) as _)) }; } // gen:70e72d575819
#[macro_export] macro_rules! va_start { ($p0:expr, $p1:expr $(,)?) => { __builtin_va_start((($p0) as _), (($p1) as _)) }; } // gen:ce7a1aa01504
pub const CLIENT_BO_DISCARDED: *mut wined3d_bo = (((!((0) as UINT_PTR))) as *mut wined3d_bo); // gen:9c136f76ae0e
pub const DXGI_ERROR_NOT_CURRENTLY_AVAILABLE: HRESULT = ((0x887A0022u32) as HRESULT); // gen:f6f332d1beb8
pub const ENUM_CURRENT_SETTINGS: DWORD = (((-(1 as c_int))) as DWORD); // gen:7f7d3891df7d
pub const ENUM_REGISTRY_SETTINGS: DWORD = (((-(2 as c_int))) as DWORD); // gen:92d828294c82
pub const E_FAIL: HRESULT = ((0x80004005u32) as HRESULT); // gen:1275c1cb9e8d
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT); // gen:cd8a52171603
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT); // gen:80daa5b2ffa6
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT); // gen:cbf314acc21c
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT); // gen:98bee37aa995
pub const HKEY_CURRENT_USER: HKEY = ((((((0x80000001u32) as LONG)) as LONG_PTR)) as HKEY); // gen:f003d9cba1a1
pub const HWND_NOTOPMOST: HWND = (((-(2 as c_int))) as HWND); // gen:25aa63012d49
pub const HWND_TOPMOST: HWND = (((-(1 as c_int))) as HWND); // gen:cd45f4c88f71
pub const IDC_ARROW: ULONG_PTR = ((((32512) as WORD)) as ULONG_PTR); // gen:1fd690149ef3
pub const IDI_WINLOGO: ULONG_PTR = ((((32517) as WORD)) as ULONG_PTR); // gen:8b3f1b9ba5ec
pub const RT_RCDATA: ULONG_PTR = ((((10) as WORD)) as ULONG_PTR); // gen:dcc6ad98f6ac
pub const S_FALSE: HRESULT = ((1) as HRESULT); // gen:b1b8213e7dac
pub const S_OK: HRESULT = ((0) as HRESULT); // gen:9969a2c48b97
pub const WINED3DERR_CONFLICTINGRENDERSTATE: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2081) as c_uint))) as HRESULT); // gen:4832971bc084
pub const WINED3DERR_INVALIDCALL: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2156) as c_uint))) as HRESULT); // gen:71d0f13a48b5
pub const WINED3DERR_NOTAVAILABLE: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2154) as c_uint))) as HRESULT); // gen:1ecf2da0a957
pub const WINED3DERR_OUTOFVIDEOMEMORY: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((380) as c_uint))) as HRESULT); // gen:71de699a8e5a
pub const WINED3DERR_UNSUPPORTEDTEXTUREFILTER: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2082) as c_uint))) as HRESULT); // gen:1384eaa43fe5
pub const WINED3DOK_NOMIPGEN: HRESULT = (((((((0) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2159) as c_uint))) as HRESULT); // gen:6960a27191f9
pub const WINED3D_OK: HRESULT = ((0) as HRESULT); // gen:7d6ca58d383b
pub const WINEDDERR_INVALIDRECT: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((150) as c_uint))) as HRESULT); // gen:e6270e3e06a4
pub const WINEDDERR_NOTAOVERLAYSURFACE: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((580) as c_uint))) as HRESULT); // gen:5f576f9df060
pub const WINEDDERR_NOTLOCKED: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((584) as c_uint))) as HRESULT); // gen:3fc3eebb09ca
pub const WINEDDERR_OVERLAYNOTVISIBLE: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((577) as c_uint))) as HRESULT); // gen:aa041abecea4
pub const WINEDDERR_SURFACEBUSY: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((430) as c_uint))) as HRESULT); // gen:9d1104cc5022
pub type wine_rb_entry = rb_entry; // gen:50fa556f323c
pub type wine_rb_tree = rb_tree; // gen:edcab44af885
pub use crate::{CreateWindowA, VK_MAKE_VERSION, VK_VERSION_MAJOR, VK_VERSION_MINOR, VK_VERSION_PATCH, WINED3D_TS_WORLD_MATRIX, WINEMAKEFOURCC, isfinite, isinf, isnan, max, min, va_end, va_start};
// ==== end SDK header macros ====

// HAND PORT (above the generated block, so `port macros` never shadows it):
// wined3d_private.h `#define wined3d_lock_init(lock, name) wined3d_lock_init_(lock, __FILE__ ": " name)`.
// The name is only the critical section's debug label. A string literal gets C's exact
// compile-time `__FILE__ ": " name`, NUL-terminated; the bodies pass `b"..\0".as_ptr()`, which
// cannot be concatenated, so that form keeps the bare name (the label loses its file prefix).
#[allow(unused_macros)]
macro_rules! wined3d_lock_init {
    ($lock:expr, $name:literal $(,)?) => {
        wined3d_lock_init_($lock, concat!(file!(), ": ", $name, "\0").as_ptr() as *const ::core::ffi::c_char)
    };
    ($lock:expr, $name:expr $(,)?) => {
        wined3d_lock_init_($lock, $name)
    };
}
#[allow(unused_imports)]
pub(crate) use wined3d_lock_init;

// HAND PORT: wined3d_gl.h:402 `checkGLcall(A)` -- ERR channel on and no ARB_debug_output:
// wined3d_check_gl_call(gl_info, __FILE__, __LINE__, A). It reads the CALLER's `gl_info`, which
// macro hygiene hides from a prelude macro, so integrate passes it in (the line below).
// port: caller-locals checkGLcall(gl_info)
#[allow(unused_macros)]
macro_rules! checkGLcall {
    ($gl_info:expr, $a:literal $(,)?) => {
        checkGLcall!($gl_info, concat!($a, "\0").as_ptr() as *const ::core::ffi::c_char)
    };
    ($gl_info:expr, $a:expr $(,)?) => {{
        let gl_info: *const wined3d_gl_info = $gl_info as *const wined3d_gl_info;
        if ERR_ON!(__wine_dbch_d3d) != 0 && (*gl_info).supported[ARB_DEBUG_OUTPUT as usize] == 0 {
            wined3d_check_gl_call(
                gl_info,
                concat!(file!(), "\0").as_ptr() as *const ::core::ffi::c_char,
                line!() as ::core::ffi::c_uint,
                $a as *const ::core::ffi::c_char,
            );
        }
    }};
}
#[allow(unused_imports)]
pub(crate) use checkGLcall;

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/directx/wine/wined3d/wined3d_private.h:1561: #define D3DCOLOR_B_A(dw) (((dw) >> 24) & 0xff) // gen:b7bd839730e7
#[allow(unused_macros)]
macro_rules! D3DCOLOR_B_A {
    ($dw:expr $(,)?) => {
        ((($dw) >> 24) & 0xff)
    };
}
#[allow(unused_imports)]
pub(crate) use D3DCOLOR_B_A;
// dll/directx/wine/wined3d/wined3d_private.h:1560: #define D3DCOLOR_B_B(dw) (((dw) >> 0) & 0xff) // gen:b99846f791e0
#[allow(unused_macros)]
macro_rules! D3DCOLOR_B_B {
    ($dw:expr $(,)?) => {
        ((($dw) >> 0) & 0xff)
    };
}
#[allow(unused_imports)]
pub(crate) use D3DCOLOR_B_B;
// dll/directx/wine/wined3d/wined3d_private.h:1559: #define D3DCOLOR_B_G(dw) (((dw) >> 8) & 0xff) // gen:89efdd33a847
#[allow(unused_macros)]
macro_rules! D3DCOLOR_B_G {
    ($dw:expr $(,)?) => {
        ((($dw) >> 8) & 0xff)
    };
}
#[allow(unused_imports)]
pub(crate) use D3DCOLOR_B_G;
// dll/directx/wine/wined3d/wined3d_private.h:1558: #define D3DCOLOR_B_R(dw) (((dw) >> 16) & 0xff) // gen:f46d8d02fc1d
#[allow(unused_macros)]
macro_rules! D3DCOLOR_B_R {
    ($dw:expr $(,)?) => {
        ((($dw) >> 16) & 0xff)
    };
}
#[allow(unused_imports)]
pub(crate) use D3DCOLOR_B_R;
// dll/directx/wine/wined3d/wined3d_private.h:1711: #define GET_TEXCOORD_SIZE_FROM_FVF(d3dvtVertexType, tex_num) (((((d3dvtVertexType) >> (16 + (2 * (tex_num)))) + 1) & 0x03) + 1) // gen:717713e0fa56
#[allow(unused_macros)]
macro_rules! GET_TEXCOORD_SIZE_FROM_FVF {
    ($v:expr, $t:expr $(,)?) => {
        (((($v >> (16 + (2 * $t))) + 1) & 0x03) + 1)
    };
}
#[allow(unused_imports)]
pub(crate) use GET_TEXCOORD_SIZE_FROM_FVF;
// dll/directx/wine/wined3d/wined3d_private.h:97: #define MAKEDWORD_VERSION(maj, min) (((maj & 0xffffu) << 16) | (min & 0xffffu)) // gen:74f2bf794775
#[allow(unused_macros)]
macro_rules! MAKEDWORD_VERSION {
    ($maj:expr, $min:expr $(,)?) => {
        ((((($maj) as u32) & 0xffffu32) << 16) | ((($min) as u32) & 0xffffu32))
    };
}
#[allow(unused_imports)]
pub(crate) use MAKEDWORD_VERSION;
// dll/directx/wine/wined3d/wined3d_private.h:1753: #define STATE_CLIPPLANE(a) (STATE_SCISSORRECT + 1 + (a)) // gen:eac60d992af5
#[allow(unused_macros)]
macro_rules! STATE_CLIPPLANE {
    ($a:expr $(,)?) => {
        (((STATE_SCISSORRECT) as u32) + 1u32 + ((($a) as u32)))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_CLIPPLANE;
// dll/directx/wine/wined3d/wined3d_private.h:1807: #define STATE_CONSTANT_BUFFER(a) ((a) != WINED3D_SHADER_TYPE_COMPUTE ? STATE_GRAPHICS_CONSTANT_BUFFER(a) : STATE_COMPUTE_CONSTANT_BUFFER) // gen:ed5f10e8cc46
#[allow(unused_macros)]
macro_rules! STATE_CONSTANT_BUFFER {
    ($a:expr $(,)?) => {
        (if (($a) as u32) != ((WINED3D_SHADER_TYPE_COMPUTE) as u32) {
            STATE_GRAPHICS_CONSTANT_BUFFER!((($a) as u32))
        } else {
            ((STATE_COMPUTE_CONSTANT_BUFFER) as u32)
        })
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_CONSTANT_BUFFER;
// dll/directx/wine/wined3d/wined3d_private.h:1728: #define STATE_GRAPHICS_CONSTANT_BUFFER(a) (STATE_GRAPHICS_SHADER(WINED3D_SHADER_TYPE_GRAPHICS_COUNT) + (a)) // gen:9b496a6f72a0
#[allow(unused_macros)]
macro_rules! STATE_GRAPHICS_CONSTANT_BUFFER {
    ($a:expr $(,)?) => {
        (STATE_GRAPHICS_SHADER!(((WINED3D_SHADER_TYPE_GRAPHICS_COUNT) as u32)) + (($a) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_GRAPHICS_CONSTANT_BUFFER;
// dll/directx/wine/wined3d/wined3d_private.h:1724: #define STATE_GRAPHICS_SHADER(a) (STATE_TEXTURESTAGE(WINED3D_MAX_FFP_TEXTURES - 1, WINED3D_HIGHEST_TEXTURE_STATE) + 1 + (a)) // gen:cb054692e142
#[allow(unused_macros)]
macro_rules! STATE_GRAPHICS_SHADER {
    ($a:expr $(,)?) => {
        (STATE_TEXTURESTAGE!(((WINED3D_MAX_FFP_TEXTURES) as u32) - 1, ((WINED3D_HIGHEST_TEXTURE_STATE) as u32)) + 1u32 + (($a) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_GRAPHICS_SHADER;
// dll/directx/wine/wined3d/wined3d_private.h:1763: #define STATE_IS_BASEVERTEXINDEX(a) ((a) == STATE_BASEVERTEXINDEX) // gen:6b81a0fff4e3
#[allow(unused_macros)]
macro_rules! STATE_IS_BASEVERTEXINDEX {
    ($a:expr $(,)?) => {
        ((($a) as u32) == ((STATE_BASEVERTEXINDEX) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_BASEVERTEXINDEX;
// dll/directx/wine/wined3d/wined3d_private.h:1772: #define STATE_IS_BLEND(a) ((a) == STATE_BLEND) // gen:5076286f37e0
#[allow(unused_macros)]
macro_rules! STATE_IS_BLEND {
    ($a:expr $(,)?) => {
        ((($a) as u32) == ((STATE_BLEND) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_BLEND;
// dll/directx/wine/wined3d/wined3d_private.h:1775: #define STATE_IS_BLEND_FACTOR(a) ((a) == STATE_BLEND_FACTOR) // gen:7ff63d411d3d
#[allow(unused_macros)]
macro_rules! STATE_IS_BLEND_FACTOR {
    ($a:expr $(,)?) => {
        ((($a) as u32) == ((STATE_BLEND_FACTOR) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_BLEND_FACTOR;
// dll/directx/wine/wined3d/wined3d_private.h:1754: #define STATE_IS_CLIPPLANE(a) ((a) >= STATE_CLIPPLANE(0) && (a) <= STATE_CLIPPLANE(WINED3D_MAX_CLIP_DISTANCES - 1)) // gen:f34f4c9cb20d
#[allow(unused_macros)]
macro_rules! STATE_IS_CLIPPLANE {
    ($a:expr $(,)?) => {
        ((($a) as u32) >= STATE_CLIPPLANE!(0) && (($a) as u32) <= STATE_CLIPPLANE!(((WINED3D_MAX_CLIP_DISTANCES) as u32) - 1u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_CLIPPLANE;
// dll/directx/wine/wined3d/wined3d_private.h:1803: #define STATE_IS_COMPUTE(a) ((a) >= STATE_COMPUTE_OFFSET && (a) <= STATE_COMPUTE_HIGHEST) // gen:ea98ac91a631
#[allow(unused_macros)]
macro_rules! STATE_IS_COMPUTE {
    ($a:expr $(,)?) => {
        ((($a) as u32) >= ((STATE_COMPUTE_OFFSET) as u32) && (($a) as u32) <= ((STATE_COMPUTE_HIGHEST) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_COMPUTE;
// dll/directx/wine/wined3d/wined3d_private.h:1792: #define STATE_IS_COMPUTE_CONSTANT_BUFFER(a) ((a) == STATE_COMPUTE_CONSTANT_BUFFER) // gen:64f020333be7
#[allow(unused_macros)]
macro_rules! STATE_IS_COMPUTE_CONSTANT_BUFFER {
    ($a:expr $(,)?) => {
        ((($a) as u32) == ((STATE_COMPUTE_CONSTANT_BUFFER) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_COMPUTE_CONSTANT_BUFFER;
// dll/directx/wine/wined3d/wined3d_private.h:1789: #define STATE_IS_COMPUTE_SHADER(a) ((a) == STATE_COMPUTE_SHADER) // gen:e85f903db087
#[allow(unused_macros)]
macro_rules! STATE_IS_COMPUTE_SHADER {
    ($a:expr $(,)?) => {
        ((($a) as u32) == ((STATE_COMPUTE_SHADER) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_COMPUTE_SHADER;
// dll/directx/wine/wined3d/wined3d_private.h:1795: #define STATE_IS_COMPUTE_SHADER_RESOURCE_BINDING(a) ((a) == STATE_COMPUTE_SHADER_RESOURCE_BINDING) // gen:cdd53d873f1b
#[allow(unused_macros)]
macro_rules! STATE_IS_COMPUTE_SHADER_RESOURCE_BINDING {
    ($a:expr $(,)?) => {
        ((($a) as u32) == ((STATE_COMPUTE_SHADER_RESOURCE_BINDING) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_COMPUTE_SHADER_RESOURCE_BINDING;
// dll/directx/wine/wined3d/wined3d_private.h:1798: #define STATE_IS_COMPUTE_UNORDERED_ACCESS_VIEW_BINDING(a) ((a) == STATE_COMPUTE_UNORDERED_ACCESS_VIEW_BINDING) // gen:1b50a9569609
#[allow(unused_macros)]
macro_rules! STATE_IS_COMPUTE_UNORDERED_ACCESS_VIEW_BINDING {
    ($a:expr $(,)?) => {
        ((($a) as u32) == ((STATE_COMPUTE_UNORDERED_ACCESS_VIEW_BINDING) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_COMPUTE_UNORDERED_ACCESS_VIEW_BINDING;
// dll/directx/wine/wined3d/wined3d_private.h:1760: #define STATE_IS_DEPTH_BOUNDS(a) ((a) == STATE_DEPTH_BOUNDS) // gen:6011efde7ca4
#[allow(unused_macros)]
macro_rules! STATE_IS_DEPTH_BOUNDS {
    ($a:expr $(,)?) => {
        ((($a) as u32) == ((STATE_DEPTH_BOUNDS) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_DEPTH_BOUNDS;
// dll/directx/wine/wined3d/wined3d_private.h:1781: #define STATE_IS_DEPTH_STENCIL(a) ((a) == STATE_DEPTH_STENCIL) // gen:f542625d89a9
#[allow(unused_macros)]
macro_rules! STATE_IS_DEPTH_STENCIL {
    ($a:expr $(,)?) => {
        ((($a) as u32) == ((STATE_DEPTH_STENCIL) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_DEPTH_STENCIL;
// dll/directx/wine/wined3d/wined3d_private.h:1766: #define STATE_IS_FRAMEBUFFER(a) ((a) == STATE_FRAMEBUFFER) // gen:2327f97bbd67
#[allow(unused_macros)]
macro_rules! STATE_IS_FRAMEBUFFER {
    ($a:expr $(,)?) => {
        ((($a) as u32) == ((STATE_FRAMEBUFFER) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_FRAMEBUFFER;
// dll/directx/wine/wined3d/wined3d_private.h:1729: #define STATE_IS_GRAPHICS_CONSTANT_BUFFER(a) ((a) >= STATE_GRAPHICS_CONSTANT_BUFFER(0) && (a) < STATE_GRAPHICS_CONSTANT_BUFFER(WINED3D_SHADER_TYPE_GRAPHICS_COUNT)) // gen:7fde08c2a5cc
#[allow(unused_macros)]
macro_rules! STATE_IS_GRAPHICS_CONSTANT_BUFFER {
    ($a:expr $(,)?) => {
        ((($a) as u32) >= STATE_GRAPHICS_CONSTANT_BUFFER!(0) && (($a) as u32) < STATE_GRAPHICS_CONSTANT_BUFFER!(((WINED3D_SHADER_TYPE_GRAPHICS_COUNT) as u32)))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_GRAPHICS_CONSTANT_BUFFER;
// dll/directx/wine/wined3d/wined3d_private.h:1725: #define STATE_IS_GRAPHICS_SHADER(a) ((a) >= STATE_GRAPHICS_SHADER(0) && (a) < STATE_GRAPHICS_SHADER(WINED3D_SHADER_TYPE_GRAPHICS_COUNT)) // gen:046ad6484a64
#[allow(unused_macros)]
macro_rules! STATE_IS_GRAPHICS_SHADER {
    ($a:expr $(,)?) => {
        ((($a) as u32) >= STATE_GRAPHICS_SHADER!(0) && (($a) as u32) < STATE_GRAPHICS_SHADER!(((WINED3D_SHADER_TYPE_GRAPHICS_COUNT) as u32)))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_GRAPHICS_SHADER;
// dll/directx/wine/wined3d/wined3d_private.h:1734: #define STATE_IS_GRAPHICS_SHADER_RESOURCE_BINDING(a) ((a) == STATE_GRAPHICS_SHADER_RESOURCE_BINDING) // gen:9e0457047e7f
#[allow(unused_macros)]
macro_rules! STATE_IS_GRAPHICS_SHADER_RESOURCE_BINDING {
    ($a:expr $(,)?) => {
        ((($a) as u32) == ((STATE_GRAPHICS_SHADER_RESOURCE_BINDING) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_GRAPHICS_SHADER_RESOURCE_BINDING;
// dll/directx/wine/wined3d/wined3d_private.h:1737: #define STATE_IS_GRAPHICS_UNORDERED_ACCESS_VIEW_BINDING(a) ((a) == STATE_GRAPHICS_UNORDERED_ACCESS_VIEW_BINDING) // gen:558b79831c70
#[allow(unused_macros)]
macro_rules! STATE_IS_GRAPHICS_UNORDERED_ACCESS_VIEW_BINDING {
    ($a:expr $(,)?) => {
        ((($a) as u32) == ((STATE_GRAPHICS_UNORDERED_ACCESS_VIEW_BINDING) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_GRAPHICS_UNORDERED_ACCESS_VIEW_BINDING;
// dll/directx/wine/wined3d/wined3d_private.h:1742: #define STATE_IS_INDEXBUFFER(a) ((a) == STATE_INDEXBUFFER) // gen:775fdbc98cd5
#[allow(unused_macros)]
macro_rules! STATE_IS_INDEXBUFFER {
    ($a:expr $(,)?) => {
        ((($a) as u32) == ((STATE_INDEXBUFFER) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_INDEXBUFFER;
// dll/directx/wine/wined3d/wined3d_private.h:1757: #define STATE_IS_RASTERIZER(a) ((a) == STATE_RASTERIZER) // gen:4c636942cb0c
#[allow(unused_macros)]
macro_rules! STATE_IS_RASTERIZER {
    ($a:expr $(,)?) => {
        ((($a) as u32) == ((STATE_RASTERIZER) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_RASTERIZER;
// dll/directx/wine/wined3d/wined3d_private.h:1717: #define STATE_IS_RENDER(a) ((a) >= STATE_RENDER(1) && (a) <= STATE_RENDER(WINEHIGHEST_RENDER_STATE)) // gen:8677c82ec6bf
#[allow(unused_macros)]
macro_rules! STATE_IS_RENDER {
    ($a:expr $(,)?) => {
        ((($a) as u32) >= STATE_RENDER!(1u32) && (($a) as u32) <= STATE_RENDER!(((WINEHIGHEST_RENDER_STATE) as u32)))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_RENDER;
// dll/directx/wine/wined3d/wined3d_private.h:1778: #define STATE_IS_SAMPLE_MASK(a) ((a) == STATE_SAMPLE_MASK) // gen:bba0ca0b0035
#[allow(unused_macros)]
macro_rules! STATE_IS_SAMPLE_MASK {
    ($a:expr $(,)?) => {
        ((($a) as u32) == ((STATE_SAMPLE_MASK) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_SAMPLE_MASK;
// dll/directx/wine/wined3d/wined3d_private.h:1751: #define STATE_IS_SCISSORRECT(a) ((a) == STATE_SCISSORRECT) // gen:98a37e099546
#[allow(unused_macros)]
macro_rules! STATE_IS_SCISSORRECT {
    ($a:expr $(,)?) => {
        ((($a) as u32) == ((STATE_SCISSORRECT) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_SCISSORRECT;
// dll/directx/wine/wined3d/wined3d_private.h:1784: #define STATE_IS_STENCIL_REF(a) ((a) == STATE_STENCIL_REF) // gen:5da51153581e
#[allow(unused_macros)]
macro_rules! STATE_IS_STENCIL_REF {
    ($a:expr $(,)?) => {
        ((($a) as u32) == ((STATE_STENCIL_REF) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_STENCIL_REF;
// dll/directx/wine/wined3d/wined3d_private.h:1740: #define STATE_IS_STREAMSRC(a) ((a) == STATE_STREAMSRC) // gen:e9dfa77d5d81
#[allow(unused_macros)]
macro_rules! STATE_IS_STREAMSRC {
    ($a:expr $(,)?) => {
        ((($a) as u32) == ((STATE_STREAMSRC) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_STREAMSRC;
// dll/directx/wine/wined3d/wined3d_private.h:1769: #define STATE_IS_STREAM_OUTPUT(a) ((a) == STATE_STREAM_OUTPUT) // gen:ecee9e4ee99d
#[allow(unused_macros)]
macro_rules! STATE_IS_STREAM_OUTPUT {
    ($a:expr $(,)?) => {
        ((($a) as u32) == ((STATE_STREAM_OUTPUT) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_STREAM_OUTPUT;
// dll/directx/wine/wined3d/wined3d_private.h:1721: #define STATE_IS_TEXTURESTAGE(a) ((a) >= STATE_TEXTURESTAGE(0, 1) && (a) <= STATE_TEXTURESTAGE(WINED3D_MAX_FFP_TEXTURES - 1, WINED3D_HIGHEST_TEXTURE_STATE)) // gen:cb990dfd4d42
#[allow(unused_macros)]
macro_rules! STATE_IS_TEXTURESTAGE {
    ($a:expr $(,)?) => {
        ((($a) as u32) >= STATE_TEXTURESTAGE!(0, 1u32) && (($a) as u32) <= STATE_TEXTURESTAGE!(((WINED3D_MAX_FFP_TEXTURES) as u32) - 1, ((WINED3D_HIGHEST_TEXTURE_STATE) as u32)))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_TEXTURESTAGE;
// dll/directx/wine/wined3d/wined3d_private.h:1745: #define STATE_IS_VDECL(a) ((a) == STATE_VDECL) // gen:878f638256f9
#[allow(unused_macros)]
macro_rules! STATE_IS_VDECL {
    ($a:expr $(,)?) => {
        ((($a) as u32) == ((STATE_VDECL) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_VDECL;
// dll/directx/wine/wined3d/wined3d_private.h:1748: #define STATE_IS_VIEWPORT(a) ((a) == STATE_VIEWPORT) // gen:021354798cd7
#[allow(unused_macros)]
macro_rules! STATE_IS_VIEWPORT {
    ($a:expr $(,)?) => {
        ((($a) as u32) == ((STATE_VIEWPORT) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_IS_VIEWPORT;
// dll/directx/wine/wined3d/wined3d_private.h:1716: #define STATE_RENDER(a) (a) // gen:8101b2f614de
#[allow(unused_macros)]
macro_rules! STATE_RENDER {
    ($a:expr $(,)?) => {
        ((($a) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_RENDER;
// dll/directx/wine/wined3d/wined3d_private.h:1806: #define STATE_SHADER(a) ((a) != WINED3D_SHADER_TYPE_COMPUTE ? STATE_GRAPHICS_SHADER(a) : STATE_COMPUTE_SHADER) // gen:83fe75be4c39
#[allow(unused_macros)]
macro_rules! STATE_SHADER {
    ($a:expr $(,)?) => {
        (if (($a) as u32) != ((WINED3D_SHADER_TYPE_COMPUTE) as u32) {
            STATE_GRAPHICS_SHADER!((($a) as u32))
        } else {
            ((STATE_COMPUTE_SHADER) as u32)
        })
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_SHADER;
// dll/directx/wine/wined3d/wined3d_private.h:1719: #define STATE_TEXTURESTAGE(stage, num) (STATE_RENDER(WINEHIGHEST_RENDER_STATE) + 1 + (stage) * (WINED3D_HIGHEST_TEXTURE_STATE + 1) + (num)) // gen:d3bd30884622
#[allow(unused_macros)]
macro_rules! STATE_TEXTURESTAGE {
    ($stage:expr, $num:expr $(,)?) => {
        (STATE_RENDER!(((WINEHIGHEST_RENDER_STATE) as u32)) + 1u32 + (($stage) as u32) * (((WINED3D_HIGHEST_TEXTURE_STATE) as u32) + 1u32) + (($num) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_TEXTURESTAGE;
// dll/directx/wine/wined3d/wined3d_private.h:1809: #define STATE_UNORDERED_ACCESS_VIEW_BINDING(a) ((a) == WINED3D_PIPELINE_GRAPHICS ? STATE_GRAPHICS_UNORDERED_ACCESS_VIEW_BINDING : STATE_COMPUTE_UNORDERED_ACCESS_VIEW_BINDING) // gen:9303c1eabfbf
#[allow(unused_macros)]
macro_rules! STATE_UNORDERED_ACCESS_VIEW_BINDING {
    ($a:expr $(,)?) => {
        (if (($a) as u32) == ((WINED3D_PIPELINE_GRAPHICS) as u32) {
            ((STATE_GRAPHICS_UNORDERED_ACCESS_VIEW_BINDING) as u32)
        } else {
            ((STATE_COMPUTE_UNORDERED_ACCESS_VIEW_BINDING) as u32)
        })
    };
}
#[allow(unused_imports)]
pub(crate) use STATE_UNORDERED_ACCESS_VIEW_BINDING;
// dll/directx/wine/wined3d/wined3d_vk.h:272: #define VK_CALL(f) (vk_info->vk_ops.f) // gen:f99a2aac0628
#[allow(unused_macros)]
macro_rules! VK_CALL {
    ($f:ident $(,)?) => {
        ((*vk_info).vk_ops.$f)
    };
}
#[allow(unused_imports)]
pub(crate) use VK_CALL;
// dll/directx/wine/wined3d/wined3d_vk.h:61: #define VK_DEVICE_FUNCS() VK_DEVICE_PFN(vkAllocateCommandBuffers) VK_DEVICE_PFN(vkAllocateDescriptorSets) VK_DEVICE_PFN(vkAllocateMemory) VK_DEVICE_PFN(vkBeginCommandBuffer) VK_DEVICE_PFN(vkBindBufferMemory) VK_DEVICE_PFN(vkBindImageMemory) VK_DEVICE_PFN(vkCmdBeginQuery) VK_DEVICE_PFN(vkCmdBeginRenderPass) VK_DEVICE_PFN(vkCmdBindDescriptorSets) VK_DEVICE_PFN(vkCmdBindIndexBuffer) VK_DEVICE_PFN(vkCmdBindPipeline) VK_DEVICE_PFN(vkCmdBindVertexBuffers) VK_DEVICE_PFN(vkCmdBlitImage) VK_DEVICE_PFN(vkCmdClearAttachments) VK_DEVICE_PFN(vkCmdClearColorImage) VK_DEVICE_PFN(vkCmdClearDepthStencilImage) VK_DEVICE_PFN(vkCmdCopyBuffer) VK_DEVICE_PFN(vkCmdCopyBufferToImage) VK_DEVICE_PFN(vkCmdCopyImage) VK_DEVICE_PFN(vkCmdCopyImageToBuffer) VK_DEVICE_PFN(vkCmdCopyQueryPoolResults) VK_DEVICE_PFN(vkCmdDispatch) VK_DEVICE_PFN(vkCmdDispatchIndirect) VK_DEVICE_PFN(vkCmdDraw) VK_DEVICE_PFN(vkCmdDrawIndexed) VK_DEVICE_PFN(vkCmdDrawIndexedIndirect) VK_DEVICE_PFN(vkCmdDrawIndirect) VK_DEVICE_PFN(vkCmdEndQuery) VK_DEVICE_PFN(vkCmdEndRenderPass) VK_DEVICE_PFN(vkCmdExecuteCommands) VK_DEVICE_PFN(vkCmdFillBuffer) VK_DEVICE_PFN(vkCmdNextSubpass) VK_DEVICE_PFN(vkCmdPipelineBarrier) VK_DEVICE_PFN(vkCmdPushConstants) VK_DEVICE_PFN(vkCmdResetEvent) VK_DEVICE_PFN(vkCmdResetQueryPool) VK_DEVICE_PFN(vkCmdResolveImage) VK_DEVICE_PFN(vkCmdSetBlendConstants) VK_DEVICE_PFN(vkCmdSetDepthBias) VK_DEVICE_PFN(vkCmdSetDepthBounds) VK_DEVICE_PFN(vkCmdSetEvent) VK_DEVICE_PFN(vkCmdSetLineWidth) VK_DEVICE_PFN(vkCmdSetScissor) VK_DEVICE_PFN(vkCmdSetStencilCompareMask) VK_DEVICE_PFN(vkCmdSetStencilReference) VK_DEVICE_PFN(vkCmdSetStencilWriteMask) VK_DEVICE_PFN(vkCmdSetViewport) VK_DEVICE_PFN(vkCmdUpdateBuffer) VK_DEVICE_PFN(vkCmdWaitEvents) VK_DEVICE_PFN(vkCmdWriteTimestamp) VK_DEVICE_PFN(vkCreateBuffer) VK_DEVICE_PFN(vkCreateBufferView) VK_DEVICE_PFN(vkCreateCommandPool) VK_DEVICE_PFN(vkCreateComputePipelines) VK_DEVICE_PFN(vkCreateDescriptorPool) VK_DEVICE_PFN(vkCreateDescriptorSetLayout) VK_DEVICE_PFN(vkCreateEvent) VK_DEVICE_PFN(vkCreateFence) VK_DEVICE_PFN(vkCreateFramebuffer) VK_DEVICE_PFN(vkCreateGraphicsPipelines) VK_DEVICE_PFN(vkCreateImage) VK_DEVICE_PFN(vkCreateImageView) VK_DEVICE_PFN(vkCreatePipelineCache) VK_DEVICE_PFN(vkCreatePipelineLayout) VK_DEVICE_PFN(vkCreateQueryPool) VK_DEVICE_PFN(vkCreateRenderPass) VK_DEVICE_PFN(vkCreateSampler) VK_DEVICE_PFN(vkCreateSemaphore) VK_DEVICE_PFN(vkCreateShaderModule) VK_DEVICE_PFN(vkDestroyBuffer) VK_DEVICE_PFN(vkDestroyBufferView) VK_DEVICE_PFN(vkDestroyCommandPool) VK_DEVICE_PFN(vkDestroyDescriptorPool) VK_DEVICE_PFN(vkDestroyDescriptorSetLayout) VK_DEVICE_PFN(vkDestroyDevice) VK_DEVICE_PFN(vkDestroyEvent) VK_DEVICE_PFN(vkDestroyFence) VK_DEVICE_PFN(vkDestroyFramebuffer) VK_DEVICE_PFN(vkDestroyImage) VK_DEVICE_PFN(vkDestroyImageView) VK_DEVICE_PFN(vkDestroyPipeline) VK_DEVICE_PFN(vkDestroyPipelineCache) VK_DEVICE_PFN(vkDestroyPipelineLayout) VK_DEVICE_PFN(vkDestroyQueryPool) VK_DEVICE_PFN(vkDestroyRenderPass) VK_DEVICE_PFN(vkDestroySampler) VK_DEVICE_PFN(vkDestroySemaphore) VK_DEVICE_PFN(vkDestroyShaderModule) VK_DEVICE_PFN(vkDeviceWaitIdle) VK_DEVICE_PFN(vkEndCommandBuffer) VK_DEVICE_PFN(vkFlushMappedMemoryRanges) VK_DEVICE_PFN(vkFreeCommandBuffers) VK_DEVICE_PFN(vkFreeDescriptorSets) VK_DEVICE_PFN(vkFreeMemory) VK_DEVICE_PFN(vkGetBufferMemoryRequirements) VK_DEVICE_PFN(vkGetDeviceMemoryCommitment) VK_DEVICE_PFN(vkGetDeviceQueue) VK_DEVICE_PFN(vkGetEventStatus) VK_DEVICE_PFN(vkGetFenceStatus) VK_DEVICE_PFN(vkGetImageMemoryRequirements) VK_DEVICE_PFN(vkGetImageSparseMemoryRequirements) VK_DEVICE_PFN(vkGetImageSubresourceLayout) VK_DEVICE_PFN(vkGetPipelineCacheData) VK_DEVICE_PFN(vkGetQueryPoolResults) VK_DEVICE_PFN(vkGetRenderAreaGranularity) VK_DEVICE_PFN(vkInvalidateMappedMemoryRanges) VK_DEVICE_PFN(vkMapMemory) VK_DEVICE_PFN(vkMergePipelineCaches) VK_DEVICE_PFN(vkQueueBindSparse) VK_DEVICE_PFN(vkQueueSubmit) VK_DEVICE_PFN(vkQueueWaitIdle) VK_DEVICE_PFN(vkResetCommandBuffer) VK_DEVICE_PFN(vkResetCommandPool) VK_DEVICE_PFN(vkResetDescriptorPool) VK_DEVICE_PFN(vkResetEvent) VK_DEVICE_PFN(vkResetFences) VK_DEVICE_PFN(vkSetEvent) VK_DEVICE_PFN(vkUnmapMemory) VK_DEVICE_PFN(vkUpdateDescriptorSets) VK_DEVICE_PFN(vkWaitForFences) VK_DEVICE_EXT_PFN(vkCmdSetDepthCompareOpEXT) VK_DEVICE_EXT_PFN(vkCmdSetDepthTestEnableEXT) VK_DEVICE_EXT_PFN(vkCmdSetDepthWriteEnableEXT) VK_DEVICE_EXT_PFN(vkCmdSetPrimitiveTopologyEXT) VK_DEVICE_EXT_PFN(vkCmdSetStencilOpEXT) VK_DEVICE_EXT_PFN(vkCmdSetStencilTestEnableEXT) VK_DEVICE_EXT_PFN(vkCmdSetPatchControlPointsEXT) VK_DEVICE_EXT_PFN(vkCmdSetPrimitiveRestartEnableEXT) VK_DEVICE_EXT_PFN(vkCmdSetAlphaToCoverageEnableEXT) VK_DEVICE_EXT_PFN(vkCmdSetColorBlendEnableEXT) VK_DEVICE_EXT_PFN(vkCmdSetColorBlendEquationEXT) VK_DEVICE_EXT_PFN(vkCmdSetColorWriteMaskEXT) VK_DEVICE_EXT_PFN(vkCmdSetCullModeEXT) VK_DEVICE_EXT_PFN(vkCmdSetDepthBiasEnableEXT) VK_DEVICE_EXT_PFN(vkCmdSetDepthClampEnableEXT) VK_DEVICE_EXT_PFN(vkCmdSetFrontFaceEXT) VK_DEVICE_EXT_PFN(vkCmdSetRasterizationSamplesEXT) VK_DEVICE_EXT_PFN(vkCmdSetRasterizerDiscardEnableEXT) VK_DEVICE_EXT_PFN(vkCmdSetSampleMaskEXT) VK_DEVICE_EXT_PFN(vkCmdBeginQueryIndexedEXT) VK_DEVICE_EXT_PFN(vkCmdBeginTransformFeedbackEXT) VK_DEVICE_EXT_PFN(vkCmdBindTransformFeedbackBuffersEXT) VK_DEVICE_EXT_PFN(vkCmdEndQueryIndexedEXT) VK_DEVICE_EXT_PFN(vkCmdEndTransformFeedbackEXT) VK_DEVICE_PFN(vkAcquireNextImageKHR) VK_DEVICE_PFN(vkCreateSwapchainKHR) VK_DEVICE_PFN(vkDestroySwapchainKHR) VK_DEVICE_PFN(vkGetSwapchainImagesKHR) VK_DEVICE_PFN(vkQueuePresentKHR) // gen:e98ce8f6672a
#[allow(unused_macros)]
macro_rules! VK_DEVICE_FUNCS {
    () => {
        VK_DEVICE_PFN!(vkAllocateCommandBuffers)
        VK_DEVICE_PFN!(vkAllocateDescriptorSets)
        VK_DEVICE_PFN!(vkAllocateMemory)
        VK_DEVICE_PFN!(vkBeginCommandBuffer)
        VK_DEVICE_PFN!(vkBindBufferMemory)
        VK_DEVICE_PFN!(vkBindImageMemory)
        VK_DEVICE_PFN!(vkCmdBeginQuery)
        VK_DEVICE_PFN!(vkCmdBeginRenderPass)
        VK_DEVICE_PFN!(vkCmdBindDescriptorSets)
        VK_DEVICE_PFN!(vkCmdBindIndexBuffer)
        VK_DEVICE_PFN!(vkCmdBindPipeline)
        VK_DEVICE_PFN!(vkCmdBindVertexBuffers)
        VK_DEVICE_PFN!(vkCmdBlitImage)
        VK_DEVICE_PFN!(vkCmdClearAttachments)
        VK_DEVICE_PFN!(vkCmdClearColorImage)
        VK_DEVICE_PFN!(vkCmdClearDepthStencilImage)
        VK_DEVICE_PFN!(vkCmdCopyBuffer)
        VK_DEVICE_PFN!(vkCmdCopyBufferToImage)
        VK_DEVICE_PFN!(vkCmdCopyImage)
        VK_DEVICE_PFN!(vkCmdCopyImageToBuffer)
        VK_DEVICE_PFN!(vkCmdCopyQueryPoolResults)
        VK_DEVICE_PFN!(vkCmdDispatch)
        VK_DEVICE_PFN!(vkCmdDispatchIndirect)
        VK_DEVICE_PFN!(vkCmdDraw)
        VK_DEVICE_PFN!(vkCmdDrawIndexed)
        VK_DEVICE_PFN!(vkCmdDrawIndexedIndirect)
        VK_DEVICE_PFN!(vkCmdDrawIndirect)
        VK_DEVICE_PFN!(vkCmdEndQuery)
        VK_DEVICE_PFN!(vkCmdEndRenderPass)
        VK_DEVICE_PFN!(vkCmdExecuteCommands)
        VK_DEVICE_PFN!(vkCmdFillBuffer)
        VK_DEVICE_PFN!(vkCmdNextSubpass)
        VK_DEVICE_PFN!(vkCmdPipelineBarrier)
        VK_DEVICE_PFN!(vkCmdPushConstants)
        VK_DEVICE_PFN!(vkCmdResetEvent)
        VK_DEVICE_PFN!(vkCmdResetQueryPool)
        VK_DEVICE_PFN!(vkCmdResolveImage)
        VK_DEVICE_PFN!(vkCmdSetBlendConstants)
        VK_DEVICE_PFN!(vkCmdSetDepthBias)
        VK_DEVICE_PFN!(vkCmdSetDepthBounds)
        VK_DEVICE_PFN!(vkCmdSetEvent)
        VK_DEVICE_PFN!(vkCmdSetLineWidth)
        VK_DEVICE_PFN!(vkCmdSetScissor)
        VK_DEVICE_PFN!(vkCmdSetStencilCompareMask)
        VK_DEVICE_PFN!(vkCmdSetStencilReference)
        VK_DEVICE_PFN!(vkCmdSetStencilWriteMask)
        VK_DEVICE_PFN!(vkCmdSetViewport)
        VK_DEVICE_PFN!(vkCmdUpdateBuffer)
        VK_DEVICE_PFN!(vkCmdWaitEvents)
        VK_DEVICE_PFN!(vkCmdWriteTimestamp)
        VK_DEVICE_PFN!(vkCreateBuffer)
        VK_DEVICE_PFN!(vkCreateBufferView)
        VK_DEVICE_PFN!(vkCreateCommandPool)
        VK_DEVICE_PFN!(vkCreateComputePipelines)
        VK_DEVICE_PFN!(vkCreateDescriptorPool)
        VK_DEVICE_PFN!(vkCreateDescriptorSetLayout)
        VK_DEVICE_PFN!(vkCreateEvent)
        VK_DEVICE_PFN!(vkCreateFence)
        VK_DEVICE_PFN!(vkCreateFramebuffer)
        VK_DEVICE_PFN!(vkCreateGraphicsPipelines)
        VK_DEVICE_PFN!(vkCreateImage)
        VK_DEVICE_PFN!(vkCreateImageView)
        VK_DEVICE_PFN!(vkCreatePipelineCache)
        VK_DEVICE_PFN!(vkCreatePipelineLayout)
        VK_DEVICE_PFN!(vkCreateQueryPool)
        VK_DEVICE_PFN!(vkCreateRenderPass)
        VK_DEVICE_PFN!(vkCreateSampler)
        VK_DEVICE_PFN!(vkCreateSemaphore)
        VK_DEVICE_PFN!(vkCreateShaderModule)
        VK_DEVICE_PFN!(vkDestroyBuffer)
        VK_DEVICE_PFN!(vkDestroyBufferView)
        VK_DEVICE_PFN!(vkDestroyCommandPool)
        VK_DEVICE_PFN!(vkDestroyDescriptorPool)
        VK_DEVICE_PFN!(vkDestroyDescriptorSetLayout)
        VK_DEVICE_PFN!(vkDestroyDevice)
        VK_DEVICE_PFN!(vkDestroyEvent)
        VK_DEVICE_PFN!(vkDestroyFence)
        VK_DEVICE_PFN!(vkDestroyFramebuffer)
        VK_DEVICE_PFN!(vkDestroyImage)
        VK_DEVICE_PFN!(vkDestroyImageView)
        VK_DEVICE_PFN!(vkDestroyPipeline)
        VK_DEVICE_PFN!(vkDestroyPipelineCache)
        VK_DEVICE_PFN!(vkDestroyPipelineLayout)
        VK_DEVICE_PFN!(vkDestroyQueryPool)
        VK_DEVICE_PFN!(vkDestroyRenderPass)
        VK_DEVICE_PFN!(vkDestroySampler)
        VK_DEVICE_PFN!(vkDestroySemaphore)
        VK_DEVICE_PFN!(vkDestroyShaderModule)
        VK_DEVICE_PFN!(vkDeviceWaitIdle)
        VK_DEVICE_PFN!(vkEndCommandBuffer)
        VK_DEVICE_PFN!(vkFlushMappedMemoryRanges)
        VK_DEVICE_PFN!(vkFreeCommandBuffers)
        VK_DEVICE_PFN!(vkFreeDescriptorSets)
        VK_DEVICE_PFN!(vkFreeMemory)
        VK_DEVICE_PFN!(vkGetBufferMemoryRequirements)
        VK_DEVICE_PFN!(vkGetDeviceMemoryCommitment)
        VK_DEVICE_PFN!(vkGetDeviceQueue)
        VK_DEVICE_PFN!(vkGetEventStatus)
        VK_DEVICE_PFN!(vkGetFenceStatus)
        VK_DEVICE_PFN!(vkGetImageMemoryRequirements)
        VK_DEVICE_PFN!(vkGetImageSparseMemoryRequirements)
        VK_DEVICE_PFN!(vkGetImageSubresourceLayout)
        VK_DEVICE_PFN!(vkGetPipelineCacheData)
        VK_DEVICE_PFN!(vkGetQueryPoolResults)
        VK_DEVICE_PFN!(vkGetRenderAreaGranularity)
        VK_DEVICE_PFN!(vkInvalidateMappedMemoryRanges)
        VK_DEVICE_PFN!(vkMapMemory)
        VK_DEVICE_PFN!(vkMergePipelineCaches)
        VK_DEVICE_PFN!(vkQueueBindSparse)
        VK_DEVICE_PFN!(vkQueueSubmit)
        VK_DEVICE_PFN!(vkQueueWaitIdle)
        VK_DEVICE_PFN!(vkResetCommandBuffer)
        VK_DEVICE_PFN!(vkResetCommandPool)
        VK_DEVICE_PFN!(vkResetDescriptorPool)
        VK_DEVICE_PFN!(vkResetEvent)
        VK_DEVICE_PFN!(vkResetFences)
        VK_DEVICE_PFN!(vkSetEvent)
        VK_DEVICE_PFN!(vkUnmapMemory)
        VK_DEVICE_PFN!(vkUpdateDescriptorSets)
        VK_DEVICE_PFN!(vkWaitForFences)
        VK_DEVICE_EXT_PFN!(vkCmdSetDepthCompareOpEXT)
        VK_DEVICE_EXT_PFN!(vkCmdSetDepthTestEnableEXT)
        VK_DEVICE_EXT_PFN!(vkCmdSetDepthWriteEnableEXT)
        VK_DEVICE_EXT_PFN!(vkCmdSetPrimitiveTopologyEXT)
        VK_DEVICE_EXT_PFN!(vkCmdSetStencilOpEXT)
        VK_DEVICE_EXT_PFN!(vkCmdSetStencilTestEnableEXT)
        VK_DEVICE_EXT_PFN!(vkCmdSetPatchControlPointsEXT)
        VK_DEVICE_EXT_PFN!(vkCmdSetPrimitiveRestartEnableEXT)
        VK_DEVICE_EXT_PFN!(vkCmdSetAlphaToCoverageEnableEXT)
        VK_DEVICE_EXT_PFN!(vkCmdSetColorBlendEnableEXT)
        VK_DEVICE_EXT_PFN!(vkCmdSetColorBlendEquationEXT)
        VK_DEVICE_EXT_PFN!(vkCmdSetColorWriteMaskEXT)
        VK_DEVICE_EXT_PFN!(vkCmdSetCullModeEXT)
        VK_DEVICE_EXT_PFN!(vkCmdSetDepthBiasEnableEXT)
        VK_DEVICE_EXT_PFN!(vkCmdSetDepthClampEnableEXT)
        VK_DEVICE_EXT_PFN!(vkCmdSetFrontFaceEXT)
        VK_DEVICE_EXT_PFN!(vkCmdSetRasterizationSamplesEXT)
        VK_DEVICE_EXT_PFN!(vkCmdSetRasterizerDiscardEnableEXT)
        VK_DEVICE_EXT_PFN!(vkCmdSetSampleMaskEXT)
        VK_DEVICE_EXT_PFN!(vkCmdBeginQueryIndexedEXT)
        VK_DEVICE_EXT_PFN!(vkCmdBeginTransformFeedbackEXT)
        VK_DEVICE_EXT_PFN!(vkCmdBindTransformFeedbackBuffersEXT)
        VK_DEVICE_EXT_PFN!(vkCmdEndQueryIndexedEXT)
        VK_DEVICE_EXT_PFN!(vkCmdEndTransformFeedbackEXT)
        VK_DEVICE_PFN!(vkAcquireNextImageKHR)
        VK_DEVICE_PFN!(vkCreateSwapchainKHR)
        VK_DEVICE_PFN!(vkDestroySwapchainKHR)
        VK_DEVICE_PFN!(vkGetSwapchainImagesKHR)
        VK_DEVICE_PFN!(vkQueuePresentKHR)
    };
}
#[allow(unused_imports)]
pub(crate) use VK_DEVICE_FUNCS;
// dll/directx/wine/wined3d/wined3d_vk.h:33: #define VK_INSTANCE_FUNCS() VK_INSTANCE_PFN(vkCreateDevice) VK_INSTANCE_PFN(vkDestroyInstance) VK_INSTANCE_PFN(vkEnumerateDeviceExtensionProperties) VK_INSTANCE_PFN(vkEnumerateDeviceLayerProperties) VK_INSTANCE_PFN(vkEnumeratePhysicalDevices) VK_INSTANCE_PFN(vkGetDeviceProcAddr) VK_INSTANCE_PFN(vkGetPhysicalDeviceFeatures) VK_INSTANCE_PFN(vkGetPhysicalDeviceFormatProperties) VK_INSTANCE_PFN(vkGetPhysicalDeviceImageFormatProperties) VK_INSTANCE_PFN(vkGetPhysicalDeviceMemoryProperties) VK_INSTANCE_PFN(vkGetPhysicalDeviceProperties) VK_INSTANCE_PFN(vkGetPhysicalDeviceQueueFamilyProperties) VK_INSTANCE_PFN(vkGetPhysicalDeviceSparseImageFormatProperties) VK_INSTANCE_EXT_PFN(vkGetPhysicalDeviceFeatures2) VK_INSTANCE_EXT_PFN(vkGetPhysicalDeviceProperties2) VK_INSTANCE_PFN(vkDestroySurfaceKHR) VK_INSTANCE_PFN(vkGetPhysicalDeviceSurfaceCapabilitiesKHR) VK_INSTANCE_PFN(vkGetPhysicalDeviceSurfaceFormatsKHR) VK_INSTANCE_PFN(vkGetPhysicalDeviceSurfacePresentModesKHR) VK_INSTANCE_PFN(vkGetPhysicalDeviceSurfaceSupportKHR) VK_INSTANCE_PFN(vkCreateWin32SurfaceKHR) VK_INSTANCE_EXT_PFN(vkResetQueryPoolEXT) // gen:ff1865a70c08
#[allow(unused_macros)]
macro_rules! VK_INSTANCE_FUNCS {
    () => {
        VK_INSTANCE_PFN!(vkCreateDevice)
        VK_INSTANCE_PFN!(vkDestroyInstance)
        VK_INSTANCE_PFN!(vkEnumerateDeviceExtensionProperties)
        VK_INSTANCE_PFN!(vkEnumerateDeviceLayerProperties)
        VK_INSTANCE_PFN!(vkEnumeratePhysicalDevices)
        VK_INSTANCE_PFN!(vkGetDeviceProcAddr)
        VK_INSTANCE_PFN!(vkGetPhysicalDeviceFeatures)
        VK_INSTANCE_PFN!(vkGetPhysicalDeviceFormatProperties)
        VK_INSTANCE_PFN!(vkGetPhysicalDeviceImageFormatProperties)
        VK_INSTANCE_PFN!(vkGetPhysicalDeviceMemoryProperties)
        VK_INSTANCE_PFN!(vkGetPhysicalDeviceProperties)
        VK_INSTANCE_PFN!(vkGetPhysicalDeviceQueueFamilyProperties)
        VK_INSTANCE_PFN!(vkGetPhysicalDeviceSparseImageFormatProperties)
        VK_INSTANCE_EXT_PFN!(vkGetPhysicalDeviceFeatures2)
        VK_INSTANCE_EXT_PFN!(vkGetPhysicalDeviceProperties2)
        VK_INSTANCE_PFN!(vkDestroySurfaceKHR)
        VK_INSTANCE_PFN!(vkGetPhysicalDeviceSurfaceCapabilitiesKHR)
        VK_INSTANCE_PFN!(vkGetPhysicalDeviceSurfaceFormatsKHR)
        VK_INSTANCE_PFN!(vkGetPhysicalDeviceSurfacePresentModesKHR)
        VK_INSTANCE_PFN!(vkGetPhysicalDeviceSurfaceSupportKHR)
        VK_INSTANCE_PFN!(vkCreateWin32SurfaceKHR)
        VK_INSTANCE_EXT_PFN!(vkResetQueryPoolEXT)
    };
}
#[allow(unused_imports)]
pub(crate) use VK_INSTANCE_FUNCS;
// dll/directx/wine/wined3d/wined3d_private.h:1103: #define WINED3D_SHADER_VERSION(major, minor) (((major) << 8) | (minor)) // gen:3d51054ea192
#[allow(unused_macros)]
macro_rules! WINED3D_SHADER_VERSION {
    ($major:expr, $minor:expr $(,)?) => {
        // C promotes BYTE operands to int before the shift; u8 << 8 overflows.
        (((($major) as u32) << 8) | (($minor) as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use WINED3D_SHADER_VERSION;
// dll/directx/wine/wined3d/shader_sm1.c:90: #define WINED3D_SM1_VERSION_MAJOR(version) (((version) >> 8) & 0xff) // gen:e0df96f3017a
#[allow(unused_macros)]
macro_rules! WINED3D_SM1_VERSION_MAJOR {
    ($version:expr $(,)?) => {
        (($version >> 8) & 0xff)
    };
}
#[allow(unused_imports)]
pub(crate) use WINED3D_SM1_VERSION_MAJOR;
// dll/directx/wine/wined3d/shader_sm1.c:91: #define WINED3D_SM1_VERSION_MINOR(version) (((version) >> 0) & 0xff) // gen:6d9ca8c5e1c6
#[allow(unused_macros)]
macro_rules! WINED3D_SM1_VERSION_MINOR {
    ($version:expr $(,)?) => {
        (($version >> 0) & 0xff)
    };
}
#[allow(unused_imports)]
pub(crate) use WINED3D_SM1_VERSION_MINOR;
// dll/directx/wine/wined3d/shader_sm4.c:119: #define WINED3D_SM4_VERSION_MAJOR(version) (((version) >> 4) & 0xf) // gen:c8747116dde7
#[allow(unused_macros)]
macro_rules! WINED3D_SM4_VERSION_MAJOR {
    ($version:expr $(,)?) => {
        (($version >> 4) & 0xf)
    };
}
#[allow(unused_imports)]
pub(crate) use WINED3D_SM4_VERSION_MAJOR;
// dll/directx/wine/wined3d/shader_sm4.c:120: #define WINED3D_SM4_VERSION_MINOR(version) (((version) >> 0) & 0xf) // gen:54e4ef9151e8
#[allow(unused_macros)]
macro_rules! WINED3D_SM4_VERSION_MINOR {
    ($version:expr $(,)?) => {
        (($version >> 0) & 0xf)
    };
}
#[allow(unused_imports)]
pub(crate) use WINED3D_SM4_VERSION_MINOR;
// dll/directx/wine/wined3d/context_vk.c:2439: #define X (VK_COLOR_COMPONENT_R_BIT | VK_COLOR_COMPONENT_G_BIT | VK_COLOR_COMPONENT_B_BIT | VK_COLOR_COMPONENT_A_BIT) // gen:aa00c5ce914f
#[allow(unused_macros)]
macro_rules! X {
    ($($arg:expr),* $(,)?) => {
        (VK_COLOR_COMPONENT_R_BIT | VK_COLOR_COMPONENT_G_BIT | VK_COLOR_COMPONENT_B_BIT | VK_COLOR_COMPONENT_A_BIT)
    };
}
#[allow(unused_imports)]
pub(crate) use X;
// dll/directx/wine/wined3d/wined3d_private.h:54: header-inline `align` // gen:78ca8d879e14
#[allow(dead_code)]
#[inline]
pub unsafe fn align(addr: size_t, alignment: size_t) -> size_t {
    (addr + (alignment - 1)) & !(alignment - 1)
}
// dll/directx/wine/wined3d/wined3d_private.h:3893: header-inline `buffer_from_resource` // gen:9f9307a48839
#[allow(dead_code)]
#[inline]
pub unsafe fn buffer_from_resource(resource: *mut wined3d_resource) -> *mut wined3d_buffer {
    CONTAINING_RECORD!(resource, wined3d_buffer, resource)
}
// dll/directx/wine/wined3d/wined3d_private.h:4661: header-inline `can_use_texture_swizzle` // gen:437f4ca08616
#[allow(dead_code)]
#[inline]
pub unsafe fn can_use_texture_swizzle(d3d_info: *const wined3d_d3d_info, format: *const wined3d_format) -> BOOL {
    ((((*d3d_info).texture_swizzle()) != 0) && ((is_complex_fixup((*format).color_fixup)) == 0) && ((is_scaling_fixup((*format).color_fixup)) == 0)) as i32
}
// dll/directx/wine/wined3d/wined3d_private.h:4782: header-inline `context_acquire` // gen:2e22b6229863
#[allow(dead_code)]
#[inline]
pub unsafe fn context_acquire(device: *mut wined3d_device, texture: *mut wined3d_texture, sub_resource_idx: u32) -> *mut wined3d_context {
    wined3d_from_cs((*device).cs);
    ((*(*(*device).adapter).adapter_ops).adapter_acquire_context.unwrap_unchecked())(device, texture, sub_resource_idx)
}
// dll/directx/wine/wined3d/wined3d_private.h:4626: header-inline `context_apply_state` // gen:016aeb8e6bd8
#[allow(dead_code)]
#[inline]
pub unsafe fn context_apply_state(context: *mut wined3d_context, state: *const wined3d_state, state_id: DWORD) {
    let state_table: *const wined3d_state_entry = (*context).state_table;
    let rep: u32 = (*state_table.add(state_id as usize)).representative;
    ((*state_table.add(rep as usize)).apply.unwrap_unchecked())(context, state, rep);
}
// dll/directx/wine/wined3d/wined3d_private.h:4790: header-inline `context_release` // gen:70e443d9aa24
#[allow(dead_code)]
#[inline]
pub unsafe fn context_release(context: *mut wined3d_context) {
    ((*(*(*(*context).device).adapter).adapter_ops).adapter_release_context.unwrap_unchecked())(context);
}
// dll/directx/wine/wined3d/wined3d_private.h:256: header-inline `create_complex_fixup_desc` // gen:2eb8816dbcdb
#[allow(dead_code)]
#[inline]
pub unsafe fn create_complex_fixup_desc(complex_fixup: complex_fixup) -> color_fixup_desc {
    // A bitfield struct: bindgen gives it setters, not literal fields. Sign fixups stay 0.
    let mut d: color_fixup_desc = core::mem::zeroed();
    let src = |bit: u32| {
        if (complex_fixup as u32 & (1 << bit)) != 0 { CHANNEL_SOURCE_COMPLEX1 } else { CHANNEL_SOURCE_COMPLEX0 }
    };
    d.set_x_source(src(0) as _);
    d.set_y_source(src(1) as _);
    d.set_z_source(src(2) as _);
    d.set_w_source(src(3) as _);
    d
}
// dll/directx/wine/wined3d/wined3d_private.h:330: header-inline `float_16_to_32` // gen:f5881668703b
#[allow(dead_code)]
#[inline]
pub unsafe fn float_16_to_32(in_: *const u16) -> f32 {
    let s: u16 = *in_ & 0x8000;
    let e: u16 = (*in_ & 0x7c00) >> 10;
    let m: u16 = *in_ & 0x3ff;
    let sgn: f32 = if s != 0 { -1.0 } else { 1.0 };

    if e == 0 {
        if m == 0 {
            return sgn * 0.0;
        } else {
            return sgn * powf(2.0, -14.0) * (m as f32 / 1024.0);
        }
    } else if e < 31 {
        return sgn * powf(2.0, e as f32 - 15.0) * (1.0 + (m as f32 / 1024.0));
    } else {
        if m == 0 {
            return sgn * INFINITY;
        } else {
            return NAN;
        }
    }
}
// dll/directx/wine/wined3d/wined3d_private.h:348: header-inline `float_24_to_32` // gen:db9b01dcac9b
#[allow(dead_code)]
#[inline]
pub unsafe fn float_24_to_32(in_: DWORD) -> f32 {
    let sgn: f32 = if (in_ & 0x800000) != 0 { -1.0 } else { 1.0 };
    let e: u16 = ((in_ & 0x780000) >> 19) as u16;
    let m: u32 = in_ & 0x7ffff;

    if e == 0 {
        if m == 0 {
            return sgn * 0.0;
        } else {
            return sgn * powf(2.0, -6.0) * (m as f32 / 524288.0);
        }
    } else if e < 15 {
        return sgn * powf(2.0, e as f32 - 7.0) * (1.0 + (m as f32 / 524288.0));
    } else {
        if m == 0 {
            return sgn * INFINITY;
        } else {
            return NAN;
        }
    }
}
// dll/directx/wine/wined3d/wined3d_private.h:370: header-inline `float_32_to_16` // gen:6d7b65c46e98
#[allow(dead_code)]
#[inline]
pub unsafe fn float_32_to_16(in_: *const f32) -> u16 {
    let mut exp: i32 = 0;
    let mut tmp: f32 = fabsf(*in_);
    let mut mantissa: u32;
    let mut ret: u16;

    if *in_ == 0.0 {
        return 0x0000;
    }
    if ((isnan!(*in_)) != 0) {
        return 0x7c01;
    }
    if ((isinf!(*in_)) != 0) {
        return if *in_ < 0.0 { 0xfc00 } else { 0x7c00 };
    }

    if tmp < ((1u32 << 10) as f32) {
        loop {
            tmp = tmp * 2.0;
            exp -= 1;
            if !(tmp < ((1u32 << 10) as f32)) {
                break;
            }
        }
    } else if tmp >= ((1u32 << 11) as f32) {
        loop {
            tmp /= 2.0;
            exp += 1;
            if !(tmp >= ((1u32 << 11) as f32)) {
                break;
            }
        }
    }

    mantissa = tmp as u32;
    if (tmp - (mantissa as f32)) >= 0.5 {
        mantissa += 1;
    }

    exp += 10;
    exp += 15;

    if exp > 30 {
        ret = 0x7c00;
    } else if exp <= 0 {
        while exp <= 0 {
            mantissa = mantissa >> 1;
            exp += 1;
        }
        ret = (mantissa & 0x3ff) as u16;
    } else {
        ret = (((exp << 10) as u32) | (mantissa & 0x3ff)) as u16;
    }

    ret |= (if *in_ < 0.0 { 1u16 } else { 0u16 }) << 15;
    ret
}
// dll/directx/wine/wined3d/wined3d_private.h:291: header-inline `get_complex_fixup` // gen:dfb9d2dde438
#[allow(dead_code)]
#[inline]
pub unsafe fn get_complex_fixup(fixup: color_fixup_desc) -> complex_fixup {
    let mut complex_fixup: complex_fixup = 0;
    if (fixup.x_source()) as u32 == CHANNEL_SOURCE_COMPLEX1 {
        complex_fixup |= 1 << 0;
    }
    if (fixup.y_source()) as u32 == CHANNEL_SOURCE_COMPLEX1 {
        complex_fixup |= 1 << 1;
    }
    if (fixup.z_source()) as u32 == CHANNEL_SOURCE_COMPLEX1 {
        complex_fixup |= 1 << 2;
    }
    if (fixup.w_source()) as u32 == CHANNEL_SOURCE_COMPLEX1 {
        complex_fixup |= 1 << 3;
    }
    complex_fixup
}
// dll/directx/wine/wined3d/wined3d_private.h:71: header-inline `int_to_float` // gen:1efcfda7a6da
#[allow(dead_code)]
#[inline]
pub unsafe fn int_to_float(i: uint32_t) -> f32 {
    f32::from_bits(i)
}
// dll/directx/wine/wined3d/wined3d_private.h:5019: header-inline `isStateDirty` // gen:600bbcc092c9
#[allow(dead_code)]
#[inline]
pub unsafe fn isStateDirty(context: *const wined3d_context, state_id: u32) -> bool {
    wined3d_context_is_graphics_state_dirty(context, state_id)
}
// dll/directx/wine/wined3d/wined3d_private.h:273: header-inline `is_complex_fixup` // gen:4e6efd94f22d
#[allow(dead_code)]
#[inline]
pub unsafe fn is_complex_fixup(fixup: color_fixup_desc) -> BOOL {
    ((fixup.x_source()) as u32 == CHANNEL_SOURCE_COMPLEX0 || (fixup.x_source()) as u32 == CHANNEL_SOURCE_COMPLEX1) as i32
}
// dll/directx/wine/wined3d/wined3d_private.h:268: header-inline `is_identity_fixup` // gen:a9117a388df7
#[allow(dead_code)]
#[inline]
pub unsafe fn is_identity_fixup(fixup: color_fixup_desc) -> BOOL {
    (memcmp(
        &fixup as *const color_fixup_desc as *const core::ffi::c_void,
        &COLOR_FIXUP_IDENTITY as *const color_fixup_desc as *const core::ffi::c_void,
        core::mem::size_of::<color_fixup_desc>(),
    ) == 0) as i32
}
// dll/directx/wine/wined3d/wined3d_private.h:4666: header-inline `is_rasterization_disabled` // gen:7496b35bc8b7
#[allow(dead_code)]
#[inline]
pub unsafe fn is_rasterization_disabled(geometry_shader: *const wined3d_shader) -> BOOL {
    (!geometry_shader.is_null()
        && !(*geometry_shader).u.gs.so_desc.is_null()
        && (*(*geometry_shader).u.gs.so_desc).rasterizer_stream_idx == WINED3D_NO_RASTERIZER_STREAM) as i32
}
// dll/directx/wine/wined3d/wined3d_private.h:283: header-inline `is_same_fixup` // gen:b8147a7e6476
#[allow(dead_code)]
#[inline]
pub unsafe fn is_same_fixup(f1: color_fixup_desc, f2: color_fixup_desc) -> BOOL {
    (f1.x_sign_fixup() == f2.x_sign_fixup() && f1.x_source() == f2.x_source()
        && f1.y_sign_fixup() == f2.y_sign_fixup() && f1.y_source() == f2.y_source()
        && f1.z_sign_fixup() == f2.z_sign_fixup() && f1.z_source() == f2.z_source()
        && f1.w_sign_fixup() == f2.w_sign_fixup() && f1.w_source() == f2.w_source()) as i32
}
// dll/directx/wine/wined3d/wined3d_private.h:278: header-inline `is_scaling_fixup` // gen:82c857a427e2
#[allow(dead_code)]
#[inline]
pub unsafe fn is_scaling_fixup(fixup: color_fixup_desc) -> BOOL {
    (fixup.x_sign_fixup() != 0 || fixup.y_sign_fixup() != 0 || fixup.z_sign_fixup() != 0 || fixup.w_sign_fixup() != 0) as i32
}
// dll/directx/wine/wined3d/wined3d_private.h:4634: header-inline `needs_separate_srgb_gl_texture` // gen:ba22f88eb075
#[allow(dead_code)]
#[inline]
pub unsafe fn needs_separate_srgb_gl_texture(context: *const wined3d_context, texture: *const wined3d_texture) -> BOOL {
    let d3d_info = (*context).d3d_info;

    if ((*d3d_info).wined3d_creation_flags & WINED3D_SRGB_READ_WRITE_CONTROL) == 0 {
        return (FALSE) as BOOL;
    }

    if (((*d3d_info).srgb_read_control()) == 0)
        && ((*texture).resource.bind_flags & WINED3D_BIND_SHADER_RESOURCE) != 0
        && ((*texture).resource.format_caps & WINED3D_FORMAT_CAP_SRGB_READ) != 0
    {
        return (TRUE) as BOOL;
    }

    if (((*d3d_info).srgb_write_control()) == 0)
        && ((*texture).resource.bind_flags & WINED3D_BIND_RENDER_TARGET) != 0
        && ((*texture).resource.format_caps & WINED3D_FORMAT_CAP_SRGB_WRITE) != 0
    {
        return (TRUE) as BOOL;
    }

    (FALSE as i32)
}
// dll/directx/wine/wined3d/wined3d_private.h:4653: header-inline `needs_srgb_write` // gen:3f78b4cd841f
#[allow(dead_code)]
#[inline]
pub unsafe fn needs_srgb_write(d3d_info: *const wined3d_d3d_info, state: *const wined3d_state, fb: *const wined3d_fb_state) -> BOOL {
    ((((*d3d_info).wined3d_creation_flags & WINED3D_SRGB_READ_WRITE_CONTROL) == 0
        || (*state).render_states[WINED3D_RS_SRGBWRITEENABLE as usize] != 0)
        && !(*fb).render_targets[0].is_null()
        && ((*(*fb).render_targets[0]).format_caps & WINED3D_FORMAT_CAP_SRGB_WRITE) != 0) as i32
}
// dll/directx/wine/wined3d/wined3d_private.h:4415: header-inline `shader_constant_is_local` // gen:e48d07f4cd60
#[allow(dead_code)]
#[inline]
pub unsafe fn shader_constant_is_local(shader: *const wined3d_shader, reg: DWORD) -> BOOL {
    if (*shader).load_local_constsF {
        return (FALSE) as BOOL;
    }

    let head: *const list = &(*shader).constantsF;
    let mut cur: *const list = (*head).next;
    while cur != head {
        let lconst: *mut wined3d_shader_lconst = CONTAINING_RECORD!(cur, wined3d_shader_lconst, entry);
        if (*lconst).idx == reg {
            return (TRUE) as BOOL;
        }
        cur = (*cur).next;
    }

    (FALSE as i32)
}
// dll/directx/wine/wined3d/wined3d_private.h:4384: header-inline `shader_get_position_fixup` // gen:afc943e3f61a
#[allow(dead_code)]
#[inline]
pub unsafe fn shader_get_position_fixup(
    context: *const wined3d_context,
    state: *const wined3d_state,
    fixup_count: u32,
    position_fixup: *mut f32,
) {
    let mut center_offset: f32;
    let mut x: f32 = 0.0;
    let mut y: f32 = 0.0;
    let mut i: u32;

    if ((*(*context).d3d_info).wined3d_creation_flags & WINED3D_PIXEL_CENTER_INTEGER) != 0 {
        center_offset = 1.0;
    } else {
        center_offset = 0.0;
    }

    center_offset += (*(*context).d3d_info).filling_convention_offset;

    i = 0;
    while i < fixup_count {
        *position_fixup.add((4 * i) as usize) = 1.0;
        *position_fixup.add((4 * i + 1) as usize) = -1.0;
        if (((*(*context).d3d_info).subpixel_viewport()) == 0) {
            let mut dummy: f64 = 0.0;
            x = (modf((*state).viewports[i as usize].x as f64, &mut dummy) * 2.0) as f32;
            y = (modf((*state).viewports[i as usize].y as f64, &mut dummy) * 2.0) as f32;
        }
        *position_fixup.add((4 * i + 2) as usize) = (center_offset + x) / (*state).viewports[i as usize].width;
        *position_fixup.add((4 * i + 3) as usize) = (center_offset + y) / (*state).viewports[i as usize].height;
        i += 1;
    }
}
// dll/directx/wine/wined3d/wined3d_private.h:4341: header-inline `shader_is_scalar` // gen:5d1f066d9990
#[allow(dead_code)]
#[inline]
pub unsafe fn shader_is_scalar(reg: *const wined3d_shader_register) -> BOOL {
    match (*reg).type_ {
        WINED3DSPR_RASTOUT => {
            if (*reg).idx[0].offset != 0 {
                return (TRUE) as BOOL;
            }
            (FALSE as i32)
        }
        WINED3DSPR_CONSTBOOL
        | WINED3DSPR_DEPTHOUT
        | WINED3DSPR_DEPTHOUTGE
        | WINED3DSPR_DEPTHOUTLE
        | WINED3DSPR_LOOP
        | WINED3DSPR_OUTPOINTID
        | WINED3DSPR_PREDICATE
        | WINED3DSPR_PRIMID
        | WINED3DSPR_COVERAGE
        | WINED3DSPR_SAMPLEMASK
        | WINED3DSPR_STENCILREF => (TRUE as i32),
        WINED3DSPR_MISCTYPE => match (*reg).idx[0].offset {
            0 => (FALSE as i32),
            1 => (TRUE as i32),
            _ => (FALSE as i32),
        },
        WINED3DSPR_IMMCONST => ((*reg).immconst_type == WINED3D_IMMCONST_SCALAR) as i32,
        _ => (FALSE as i32),
    }
}
// dll/directx/wine/wined3d/wined3d_private.h:4431: header-inline `shader_sampler_is_shadow` // gen:4ca86dcb3691
#[allow(dead_code)]
#[inline]
pub unsafe fn shader_sampler_is_shadow(
    shader: *const wined3d_shader,
    ps_args: *const ps_compile_args,
    resource_idx: u32,
    sampler_idx: u32,
) -> BOOL {
    let version: *const wined3d_shader_version = &(*shader).reg_maps.shader_version;

    if (*version).major >= 4 {
        return (((*shader).reg_maps.sampler_comparison_mode & (1u32 << sampler_idx)) != 0) as i32;
    }

    ((*version).type_ == WINED3D_SHADER_TYPE_PIXEL && ((*ps_args).shadow as u32 & (1u32 << resource_idx)) != 0) as i32
}
// dll/directx/wine/wined3d/wined3d_private.h:3469: header-inline `texture_from_resource` // gen:6cacd6c3fc06
#[allow(dead_code)]
#[inline]
pub unsafe fn texture_from_resource(resource: *mut wined3d_resource) -> *mut wined3d_texture {
    CONTAINING_RECORD!(resource, wined3d_texture, resource)
}
// dll/directx/wine/wined3d/wined3d_private.h:4612: header-inline `use_ps` // gen:60b8e65825fc
#[allow(dead_code)]
#[inline]
pub unsafe fn use_ps(state: *const wined3d_state) -> BOOL {
    (!(*state).shader[WINED3D_SHADER_TYPE_PIXEL as usize].is_null()) as i32
}
// dll/directx/wine/wined3d/wined3d_private.h:4617: header-inline `use_transform_feedback` // gen:f5418868d760
#[allow(dead_code)]
#[inline]
pub unsafe fn use_transform_feedback(state: *const wined3d_state) -> BOOL {
    let shader: *const wined3d_shader = (*state).shader[WINED3D_SHADER_TYPE_GEOMETRY as usize];
    if shader.is_null() {
        return (FALSE) as BOOL;
    }
    (!(*shader).u.gs.so_desc.is_null()) as i32
}
// dll/directx/wine/wined3d/wined3d_private.h:4604: header-inline `use_vs` // gen:91ee2fec6d89
#[allow(dead_code)]
#[inline]
pub unsafe fn use_vs(state: *const wined3d_state) -> BOOL {
    (!(*state).shader[WINED3D_SHADER_TYPE_VERTEX as usize].is_null()
        && ((*state).vertex_declaration.is_null() || !(*(*state).vertex_declaration).position_transformed)) as i32
}
// dll/directx/wine/wined3d/wined3d_private.h:4726: header-inline `validate_material_colour_source` // gen:63d6cc69ff88
#[allow(dead_code)]
#[inline]
pub unsafe fn validate_material_colour_source(
    vdecl: *const wined3d_vertex_declaration,
    source: wined3d_material_color_source,
) -> wined3d_material_color_source {
    if source == WINED3D_MCS_COLOR1 && (*vdecl).diffuse != false {
        return source;
    }
    if source == WINED3D_MCS_COLOR2 && (*vdecl).specular != false {
        return source;
    }
    WINED3D_MCS_MATERIAL
}
// dll/directx/wine/wined3d/wined3d_vk.h:294: header-inline `vk_aspect_mask_from_format` // gen:7ef96a4ddd6c
#[allow(dead_code)]
#[inline]
pub unsafe fn vk_aspect_mask_from_format(format: *const wined3d_format) -> VkImageAspectFlags {
    let mut mask: VkImageAspectFlags = 0;

    if (*format).depth_size != 0 {
        mask |= VK_IMAGE_ASPECT_DEPTH_BIT;
    }
    if (*format).stencil_size != 0 {
        mask |= VK_IMAGE_ASPECT_STENCIL_BIT;
    }
    if mask == 0
        || (*format).red_size != 0
        || (*format).green_size != 0
        || (*format).blue_size != 0
        || (*format).alpha_size != 0
    {
        mask |= VK_IMAGE_ASPECT_COLOR_BIT;
    }

    mask
}
// dll/directx/wine/wined3d/wined3d_gl.h:806: header-inline `wined3d_adapter_gl` // gen:f02a7590a0a5
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_adapter_gl(adapter: *mut wined3d_adapter) -> *mut wined3d_adapter_gl {
    CONTAINING_RECORD!(adapter, wined3d_adapter_gl, a)
}
// dll/directx/wine/wined3d/wined3d_gl.h:811: header-inline `wined3d_adapter_gl_const` // gen:15cefad59847
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_adapter_gl_const(adapter: *const wined3d_adapter) -> *const wined3d_adapter_gl {
    CONTAINING_RECORD!(adapter, wined3d_adapter_gl, a)
}
// dll/directx/wine/wined3d/wined3d_vk.h:737: header-inline `wined3d_adapter_vk` // gen:364126290a12
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_adapter_vk(adapter: *mut wined3d_adapter) -> *mut wined3d_adapter_vk {
    CONTAINING_RECORD!(adapter, wined3d_adapter_vk, a)
}
// dll/directx/wine/wined3d/wined3d_gl.h:825: header-inline `wined3d_allocator_chunk_gl` // gen:bd8c8cc5aad9
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_allocator_chunk_gl(chunk: *mut wined3d_allocator_chunk) -> *mut wined3d_allocator_chunk_gl {
    CONTAINING_RECORD!(chunk, wined3d_allocator_chunk_gl, c)
}
// dll/directx/wine/wined3d/wined3d_gl.h:890: header-inline `wined3d_allocator_chunk_gl_lock` // gen:400617591443
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_allocator_chunk_gl_lock(chunk_gl: *mut wined3d_allocator_chunk_gl) {
    wined3d_device_gl_allocator_lock(wined3d_device_gl_from_allocator((*chunk_gl).c.allocator));
}
// dll/directx/wine/wined3d/wined3d_gl.h:895: header-inline `wined3d_allocator_chunk_gl_unlock` // gen:0119ac920625
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_allocator_chunk_gl_unlock(chunk_gl: *mut wined3d_allocator_chunk_gl) {
    wined3d_device_gl_allocator_unlock(wined3d_device_gl_from_allocator((*chunk_gl).c.allocator));
}
// dll/directx/wine/wined3d/wined3d_vk.h:783: header-inline `wined3d_allocator_chunk_vk` // gen:90c38f662e58
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_allocator_chunk_vk(chunk: *mut wined3d_allocator_chunk) -> *mut wined3d_allocator_chunk_vk {
    CONTAINING_RECORD!(chunk, wined3d_allocator_chunk_vk, c)
}
// dll/directx/wine/wined3d/wined3d_private.h:3184: header-inline `wined3d_atomic_decrement_mutex_lock` // gen:5219a57fa3e4
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_atomic_decrement_mutex_lock(refcount: *mut LONG) -> ULONG {
    let mut count: ULONG;
    let mut old_count: ULONG = *refcount as ULONG;
    loop {
        count = old_count;
        if count == 1 {
            wined3d_mutex_lock();
            count = InterlockedDecrement(refcount) as ULONG;
            if count != 0 {
                wined3d_mutex_unlock();
            }
            return count;
        }

        old_count = InterlockedCompareExchange(
            refcount,
            count.wrapping_sub(1) as LONG,
            count as LONG,
        ) as ULONG;
        if old_count == count {
            break;
        }
    }

    count.wrapping_sub(1)
}
// dll/directx/wine/wined3d/wined3d_private.h:4940: header-inline `wined3d_bitmap_clear` // gen:b64d3b7309b0
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_bitmap_clear(map: *mut u32, idx: u32) -> BOOL {
    let p: *mut u32 = map.add((idx >> 5) as usize);
    *p &= !(1u32 << (idx & 0x1f));
    (*p != 0) as i32
}
// dll/directx/wine/wined3d/wined3d_private.h:4981: header-inline `wined3d_bitmap_ffs` // gen:d5d4ecb6f978
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_bitmap_ffs(
    bitmap: *const uint32_t,
    bit_count: core::ffi::c_uint,
    start: core::ffi::c_uint,
) -> core::ffi::c_uint {
    wined3d_bitmap_ffs_xor(bitmap, bit_count, start, 0)
}
// dll/directx/wine/wined3d/wined3d_private.h:4955: header-inline `wined3d_bitmap_ffs_xor` // gen:332752afbd1f
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_bitmap_ffs_xor(
    bitmap: *const uint32_t,
    bit_count: core::ffi::c_uint,
    start: core::ffi::c_uint,
    xor_mask: uint32_t,
) -> core::ffi::c_uint {
    let word_bit_count: core::ffi::c_uint =
        (core::mem::size_of::<uint32_t>() as core::ffi::c_uint) * CHAR_BIT;
    assert!(bit_count < word_bit_count || bit_count % word_bit_count == 0);

    let mut ptr: *const uint32_t = bitmap.add((start / word_bit_count) as usize);
    let end_ptr: *const uint32_t =
        bitmap.add(((bit_count + word_bit_count - 1) / word_bit_count) as usize);

    if ptr >= end_ptr {
        return !0;
    }

    let mask: uint32_t = !0 << (start % word_bit_count);
    let mut map: uint32_t = (*ptr ^ xor_mask) & mask;
    while map == 0 {
        ptr = ptr.add(1);
        if ptr == end_ptr {
            return !0;
        }
        map = *ptr ^ xor_mask;
    }
    (ptr.offset_from(bitmap) as core::ffi::c_uint) * word_bit_count
        + (wined3d_bit_scan(&mut map)) as u32
}
// dll/directx/wine/wined3d/wined3d_private.h:4986: header-inline `wined3d_bitmap_ffz` // gen:7c63bec93dc1
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_bitmap_ffz(bitmap: *const u32, bit_count: u32, start: u32) -> u32 {
    wined3d_bitmap_ffs_xor(bitmap, bit_count, start, !0u32)
}
// dll/directx/wine/wined3d/wined3d_private.h:4991: header-inline `wined3d_bitmap_get_range` // gen:e7b38c4c1f7f
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_bitmap_get_range(
    bitmap: *const u32,
    bit_count: u32,
    start: u32,
    range: *mut wined3d_range,
) -> BOOL {
    let range_start: u32 = wined3d_bitmap_ffs(bitmap, bit_count, start);
    if range_start == !0u32 {
        return (FALSE) as BOOL;
    }

    let mut range_end: u32 = wined3d_bitmap_ffz(bitmap, bit_count, range_start + 1);
    if range_end == !0u32 {
        range_end = bit_count;
    }

    (*range).offset = range_start;
    (*range).size = range_end - range_start;
    (TRUE as i32)
}
// dll/directx/wine/wined3d/wined3d_private.h:4950: header-inline `wined3d_bitmap_is_set` // gen:66f2e96dcf24
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_bitmap_is_set(map: *const u32, idx: u32) -> BOOL {
    ((*map.add((idx >> 5) as usize) & (1u32 << (idx & 0x1f))) != 0) as i32
}
// dll/directx/wine/wined3d/wined3d_private.h:4945: header-inline `wined3d_bitmap_set` // gen:c6f5941fce7b
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_bitmap_set(map: *mut u32, idx: u32) -> BOOL {
    let p: *mut u32 = map.add((idx >> 5) as usize);
    *p |= 1u32 << (idx & 0x1f);
    (*p != 0) as i32
}
// dll/directx/wine/wined3d/wined3d_private.h:2879: header-inline `wined3d_blend_state_get_writemask` // gen:4b9fd061e3b4
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_blend_state_get_writemask(state: *const wined3d_blend_state, mut index: u32) -> u32 {
    if state.is_null() {
        return 0xf;
    }
    if (((*state).desc.independent) == 0) {
        index = 0;
    }
    (*state).desc.rt[index as usize].writemask
}
// dll/directx/wine/wined3d/wined3d_gl.h:429: header-inline `wined3d_bo_gl` // gen:e7c07f04d5e6
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_bo_gl(bo: *mut wined3d_bo) -> *mut wined3d_bo_gl {
    CONTAINING_RECORD!(bo, wined3d_bo_gl, b)
}
// dll/directx/wine/wined3d/wined3d_gl.h:434: header-inline `wined3d_bo_gl_id` // gen:87188a68fde8
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_bo_gl_id(bo: *mut wined3d_bo) -> GLuint {
    if !bo.is_null() {
        (*wined3d_bo_gl(bo)).id
    } else {
        0
    }
}
// dll/directx/wine/wined3d/wined3d_vk.h:326: header-inline `wined3d_bo_vk` // gen:51c5a44662e4
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_bo_vk(bo: *mut wined3d_bo) -> *mut wined3d_bo_vk {
    CONTAINING_RECORD!(bo, wined3d_bo_vk, b)
}
// dll/directx/wine/wined3d/wined3d_gl.h:1049: header-inline `wined3d_buffer_gl` // gen:9e7be2c6d22e
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_buffer_gl(buffer: *mut wined3d_buffer) -> *mut wined3d_buffer_gl {
    CONTAINING_RECORD!(buffer, wined3d_buffer_gl, b)
}
// dll/directx/wine/wined3d/wined3d_private.h:3898: header-inline `wined3d_buffer_validate_user` // gen:c478e3174639
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_buffer_validate_user(buffer: *mut wined3d_buffer) {
    if (*buffer).bo_user.valid {
        return;
    }
    (*buffer).bo_user.valid = true;
    list_add_head(&mut (*(*buffer).buffer_object).users, &mut (*buffer).bo_user.entry);
}
// dll/directx/wine/wined3d/wined3d_vk.h:926: header-inline `wined3d_buffer_vk` // gen:6f502ac88522
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_buffer_vk(buffer: *mut wined3d_buffer) -> *mut wined3d_buffer_vk {
    CONTAINING_RECORD!(buffer, wined3d_buffer_vk, b)
}
// dll/directx/wine/wined3d/wined3d_private.h:1563: header-inline `wined3d_color_from_d3dcolor` // gen:8b47ee2fd401
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_color_from_d3dcolor(wined3d_color: *mut wined3d_color, d3d_color: DWORD) {
    (*wined3d_color).r = D3DCOLOR_B_R!(d3d_color) as f32 / 255.0;
    (*wined3d_color).g = D3DCOLOR_B_G!(d3d_color) as f32 / 255.0;
    (*wined3d_color).b = D3DCOLOR_B_B!(d3d_color) as f32 / 255.0;
    (*wined3d_color).a = D3DCOLOR_B_A!(d3d_color) as f32 / 255.0;
}
// dll/directx/wine/wined3d/wined3d_private.h:1584: header-inline `wined3d_colour_srgb_from_linear` // gen:f1f6c83b5016
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_colour_srgb_from_linear(colour_srgb: *mut wined3d_color, colour: *const wined3d_color) {
    (*colour_srgb).r = wined3d_srgb_from_linear((*colour).r);
    (*colour_srgb).g = wined3d_srgb_from_linear((*colour).g);
    (*colour_srgb).b = wined3d_srgb_from_linear((*colour).b);
    (*colour_srgb).a = (*colour).a;
}
// dll/directx/wine/wined3d/wined3d_private.h:1628: header-inline `wined3d_const_bo_address` // gen:b67df8b3c698
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_const_bo_address(data: *const wined3d_bo_address) -> *mut wined3d_const_bo_address {
    data as *mut wined3d_const_bo_address
}
// dll/directx/wine/wined3d/wined3d_private.h:4820: header-inline `wined3d_context_copy_bo_address` // gen:80ec74b3b78b
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_context_copy_bo_address(
    context: *mut wined3d_context,
    dst: *const wined3d_bo_address,
    src: *const wined3d_bo_address,
    range_count: u32,
    ranges: *const wined3d_range,
    map_flags: uint32_t,
) {
    ((*(*(*(*context).device).adapter).adapter_ops).adapter_copy_bo_address.unwrap_unchecked())(
        context, dst, src, range_count, ranges, map_flags,
    );
}
// dll/directx/wine/wined3d/wined3d_private.h:4833: header-inline `wined3d_context_destroy_bo` // gen:4e7dc3fd3f97
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_context_destroy_bo(context: *mut wined3d_context, bo: *mut wined3d_bo) {
    ((*(*(*(*context).device).adapter).adapter_ops).adapter_destroy_bo.unwrap_unchecked())(context, bo);
}
// dll/directx/wine/wined3d/wined3d_gl.h:694: header-inline `wined3d_context_gl` // gen:455dbb6fd486
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_context_gl(context: *mut wined3d_context) -> *mut wined3d_context_gl {
    CONTAINING_RECORD!(context, wined3d_context_gl, c)
}
// dll/directx/wine/wined3d/wined3d_gl.h:699: header-inline `wined3d_context_gl_const` // gen:e8e568c2a269
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_context_gl_const(context: *const wined3d_context) -> *const wined3d_context_gl {
    CONTAINING_RECORD!(context, wined3d_context_gl, c)
}
// dll/directx/wine/wined3d/wined3d_gl.h:912: header-inline `wined3d_context_gl_reference_bo` // gen:8b348de8e1aa
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_context_gl_reference_bo(context_gl: *mut wined3d_context_gl, bo_gl: *mut wined3d_bo_gl) {
    let device_gl: *mut wined3d_device_gl = wined3d_device_gl((*context_gl).c.device);
    (*bo_gl).command_fence_id = (*device_gl).current_fence_id;
}
// dll/directx/wine/wined3d/wined3d_gl.h:919: header-inline `wined3d_context_gl_reference_buffer` // gen:8f3eb73a6e3a
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_context_gl_reference_buffer(context_gl: *mut wined3d_context_gl, buffer: *mut wined3d_buffer) {
    if !(*buffer).buffer_object.is_null() {
        wined3d_context_gl_reference_bo(context_gl, wined3d_bo_gl((*buffer).buffer_object));
    }
}
// dll/directx/wine/wined3d/wined3d_private.h:5014: header-inline `wined3d_context_is_compute_state_dirty` // gen:90af95cde8c9
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_context_is_compute_state_dirty(context: *const wined3d_context, state_id: u32) -> bool {
    ((wined3d_bitmap_is_set(((*context).dirty_compute_states).as_ptr(), state_id - (STATE_COMPUTE_OFFSET as u32))) != 0)
}
// dll/directx/wine/wined3d/wined3d_private.h:5009: header-inline `wined3d_context_is_graphics_state_dirty` // gen:ca16e4700d22
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_context_is_graphics_state_dirty(context: *const wined3d_context, state_id: u32) -> bool {
    ((wined3d_bitmap_is_set(((*context).dirty_graphics_states).as_ptr(), state_id)) != 0)
}
// dll/directx/wine/wined3d/wined3d_private.h:4808: header-inline `wined3d_context_map_bo_address` // gen:56cff71d6ac9
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_context_map_bo_address(
    context: *mut wined3d_context,
    data: *const wined3d_bo_address,
    size: size_t,
    map_flags: uint32_t,
) -> *mut core::ffi::c_void {
    ((*(*(*(*context).device).adapter).adapter_ops).adapter_map_bo_address.unwrap_unchecked())(context, data, size, map_flags)
}
// dll/directx/wine/wined3d/wined3d_private.h:4814: header-inline `wined3d_context_unmap_bo_address` // gen:fe4932ac95e0
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_context_unmap_bo_address(
    context: *mut wined3d_context,
    data: *const wined3d_bo_address,
    range_count: u32,
    ranges: *const wined3d_range,
) {
    ((*(*(*(*context).device).adapter).adapter_ops).adapter_unmap_bo_address.unwrap_unchecked())(
        context, data, range_count, ranges,
    );
}
// dll/directx/wine/wined3d/wined3d_vk.h:662: header-inline `wined3d_context_vk` // gen:79f81c8ea6a8
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_context_vk(context: *mut wined3d_context) -> *mut wined3d_context_vk {
    CONTAINING_RECORD!(context, wined3d_context_vk, c)
}
// dll/directx/wine/wined3d/wined3d_vk.h:1091: header-inline `wined3d_context_vk_reference_bo` // gen:c3d22c1ffdd8
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_context_vk_reference_bo(context_vk: *const wined3d_context_vk, bo: *mut wined3d_bo_vk) {
    (*bo).command_buffer_id = (*context_vk).current_command_buffer.id;
}
// dll/directx/wine/wined3d/wined3d_vk.h:1097: header-inline `wined3d_context_vk_reference_image` // gen:07fd832b625d
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_context_vk_reference_image(context_vk: *const wined3d_context_vk, image: *mut wined3d_image_vk) {
    (*image).command_buffer_id = (*context_vk).current_command_buffer.id;
}
// dll/directx/wine/wined3d/wined3d_vk.h:1118: header-inline `wined3d_context_vk_reference_query` // gen:34eef6f173e1
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_context_vk_reference_query(context_vk: *const wined3d_context_vk, query_vk: *mut wined3d_query_vk) {
    (*query_vk).command_buffer_id = (*context_vk).current_command_buffer.id;
}
// dll/directx/wine/wined3d/wined3d_vk.h:1130: header-inline `wined3d_context_vk_reference_rendertarget_view` // gen:9002f34fc990
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_context_vk_reference_rendertarget_view(
    context_vk: *const wined3d_context_vk,
    rtv_vk: *mut wined3d_rendertarget_view_vk,
) {
    wined3d_context_vk_reference_resource(context_vk, (*rtv_vk).v.resource);
    (*rtv_vk).command_buffer_id = (*context_vk).current_command_buffer.id;
}
// dll/directx/wine/wined3d/wined3d_vk.h:1109: header-inline `wined3d_context_vk_reference_resource` // gen:0a8efdc89917
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_context_vk_reference_resource(
    context_vk: *const wined3d_context_vk,
    resource: *mut wined3d_resource,
) {
    if (*resource).type_ == WINED3D_RTYPE_BUFFER {
        wined3d_context_vk_reference_bo(
            context_vk,
            wined3d_bo_vk((*buffer_from_resource(resource)).buffer_object)
        );
    } else {
        wined3d_context_vk_reference_texture(
            context_vk,
            wined3d_texture_vk(texture_from_resource(resource))
        );
    }
}
// dll/directx/wine/wined3d/wined3d_vk.h:1124: header-inline `wined3d_context_vk_reference_sampler` // gen:ed22644e4897
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_context_vk_reference_sampler(context_vk: *const wined3d_context_vk, sampler_vk: *mut wined3d_sampler_vk) {
    (*sampler_vk).command_buffer_id = (*context_vk).current_command_buffer.id;
}
// dll/directx/wine/wined3d/wined3d_vk.h:1137: header-inline `wined3d_context_vk_reference_shader_resource_view` // gen:be71fb522199
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_context_vk_reference_shader_resource_view(
    context_vk: *const wined3d_context_vk,
    srv_vk: *mut wined3d_shader_resource_view_vk,
) {
    wined3d_context_vk_reference_resource(context_vk, (*srv_vk).v.resource);
    (*srv_vk).view_vk.command_buffer_id = (*context_vk).current_command_buffer.id;
}
// dll/directx/wine/wined3d/wined3d_vk.h:1103: header-inline `wined3d_context_vk_reference_texture` // gen:b2405b3595e8
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_context_vk_reference_texture(context_vk: *const wined3d_context_vk, texture_vk: *mut wined3d_texture_vk) {
    wined3d_context_vk_reference_image(context_vk, &mut (*texture_vk).image);
}
// dll/directx/wine/wined3d/wined3d_vk.h:1144: header-inline `wined3d_context_vk_reference_unordered_access_view` // gen:6615fd72bc07
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_context_vk_reference_unordered_access_view(
    context_vk: *const wined3d_context_vk,
    uav_vk: *mut wined3d_unordered_access_view_vk,
) {
    wined3d_context_vk_reference_resource(context_vk, (*uav_vk).v.resource);
    (*uav_vk).view_vk.command_buffer_id = (*context_vk).current_command_buffer.id;
}
// dll/directx/wine/wined3d/wined3d_private.h:3714: header-inline `wined3d_cs_finish` // gen:2922af62a1ee
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_cs_finish(cs: *mut wined3d_cs, queue_id: wined3d_cs_queue_id) {
    ((*(*cs).c.ops).finish.unwrap_unchecked())(&mut (*cs).c, queue_id);
}
// dll/directx/wine/wined3d/wined3d_private.h:3100: header-inline `wined3d_device_bo_map_lock` // gen:7be5e32bbee1
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_device_bo_map_lock(device: *mut wined3d_device) {
    EnterCriticalSection(&mut (*device).bo_map_lock);
}
// dll/directx/wine/wined3d/wined3d_private.h:3105: header-inline `wined3d_device_bo_map_unlock` // gen:2ddd5ea62ab3
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_device_bo_map_unlock(device: *mut wined3d_device) {
    LeaveCriticalSection(&mut (*device).bo_map_lock);
}
// dll/directx/wine/wined3d/wined3d_private.h:3676: header-inline `wined3d_device_context_lock` // gen:524b405dd617
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_device_context_lock(context: *mut wined3d_device_context) {
    if core::ptr::eq(context as *const _, &(*(*(*context).device).cs).c as *const _) {
        wined3d_mutex_lock();
    }
}
// dll/directx/wine/wined3d/wined3d_private.h:3682: header-inline `wined3d_device_context_unlock` // gen:3477ba3230aa
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_device_context_unlock(context: *mut wined3d_device_context) {
    if core::ptr::eq(context as *const _, &(*(*(*context).device).cs).c as *const _) {
        wined3d_mutex_unlock();
    }
}
// dll/directx/wine/wined3d/wined3d_gl.h:870: header-inline `wined3d_device_gl` // gen:24984160eabd
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_device_gl(device: *mut wined3d_device) -> *mut wined3d_device_gl {
    CONTAINING_RECORD!(device, wined3d_device_gl, d)
}
// dll/directx/wine/wined3d/wined3d_gl.h:880: header-inline `wined3d_device_gl_allocator_lock` // gen:1996030ab3f3
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_device_gl_allocator_lock(device_gl: *mut wined3d_device_gl) {
    EnterCriticalSection(&mut (*device_gl).allocator_cs);
}
// dll/directx/wine/wined3d/wined3d_gl.h:885: header-inline `wined3d_device_gl_allocator_unlock` // gen:904a15e168c4
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_device_gl_allocator_unlock(device_gl: *mut wined3d_device_gl) {
    LeaveCriticalSection(&mut (*device_gl).allocator_cs);
}
// dll/directx/wine/wined3d/wined3d_gl.h:875: header-inline `wined3d_device_gl_from_allocator` // gen:d3b2eebb318d
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_device_gl_from_allocator(allocator: *mut wined3d_allocator) -> *mut wined3d_device_gl {
    CONTAINING_RECORD!(allocator, wined3d_device_gl, allocator)
}
// dll/directx/wine/wined3d/wined3d_private.h:3117: header-inline `wined3d_device_no3d` // gen:76b96674ae97
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_device_no3d(device: *mut wined3d_device) -> *mut wined3d_device_no3d {
    CONTAINING_RECORD!(device, wined3d_device_no3d, d)
}
// dll/directx/wine/wined3d/wined3d_vk.h:841: header-inline `wined3d_device_vk` // gen:bc439b5ede4e
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_device_vk(device: *mut wined3d_device) -> *mut wined3d_device_vk {
    CONTAINING_RECORD!(device, wined3d_device_vk, d)
}
// dll/directx/wine/wined3d/wined3d_vk.h:851: header-inline `wined3d_device_vk_allocator_lock` // gen:00168c45e996
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_device_vk_allocator_lock(device_vk: *mut wined3d_device_vk) {
    EnterCriticalSection(&mut (*device_vk).allocator_cs);
}
// dll/directx/wine/wined3d/wined3d_vk.h:856: header-inline `wined3d_device_vk_allocator_unlock` // gen:4f516c9ddaa6
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_device_vk_allocator_unlock(device_vk: *mut wined3d_device_vk) {
    LeaveCriticalSection(&mut (*device_vk).allocator_cs);
}
// dll/directx/wine/wined3d/wined3d_vk.h:846: header-inline `wined3d_device_vk_from_allocator` // gen:d33b95b7a1e8
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_device_vk_from_allocator(allocator: *mut wined3d_allocator) -> *mut wined3d_device_vk {
    CONTAINING_RECORD!(allocator, wined3d_device_vk, allocator)
}
// dll/directx/wine/wined3d/wined3d_private.h:4838: header-inline `wined3d_dsv_srv_conflict` // gen:33636c4cd2cf
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_dsv_srv_conflict(dsv: *const wined3d_rendertarget_view, srv_format: *const wined3d_format) -> BOOL {
    (srv_format.is_null()
        || ((*srv_format).red_size != 0 && ((*dsv).desc.flags & WINED3D_VIEW_READ_ONLY_DEPTH) == 0)
        || ((*srv_format).green_size != 0 && ((*dsv).desc.flags & WINED3D_VIEW_READ_ONLY_STENCIL) == 0)) as i32
}
// dll/directx/wine/wined3d/wined3d_private.h:4677: header-inline `wined3d_extract_bits` // gen:d7f47f07530f
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_extract_bits(bitstream: *const u32, offset: u32, count: u32) -> DWORD {
    let word_bit_count: u32 = (core::mem::size_of::<u32>() as u32) * CHAR_BIT;
    let idx: u32 = offset / word_bit_count;
    let shift: u32 = offset % word_bit_count;
    let mut mask: DWORD = (1u32 << count) - 1;
    let mut ret: DWORD;

    ret = (*bitstream.add(idx as usize) >> shift) & mask;
    if shift + count > word_bit_count {
        let extracted_bit_count: u32 = word_bit_count - shift;
        let remaining_bit_count: u32 = count - extracted_bit_count;
        mask = (1u32 << remaining_bit_count) - 1;
        ret |= (*bitstream.add((idx + 1) as usize) & mask) << extracted_bit_count;
    }
    ret
}
// dll/directx/wine/wined3d/wined3d_gl.h:394: header-inline `wined3d_fence_supported` // gen:3854bce3e0ee
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_fence_supported(gl_info: *const wined3d_gl_info) -> BOOL {
    ((((*gl_info).supported[ARB_SYNC as usize]) != 0)
        || (((*gl_info).supported[NV_FENCE as usize]) != 0)
        || (((*gl_info).supported[APPLE_FENCE as usize]) != 0)) as i32
}
// dll/directx/wine/wined3d/wined3d_gl.h:1156: header-inline `wined3d_format_gl` // gen:57f2fd93c52c
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_format_gl(format: *const wined3d_format) -> *const wined3d_format_gl {
    CONTAINING_RECORD!(format, wined3d_format_gl, f)
}
// dll/directx/wine/wined3d/wined3d_private.h:4599: header-inline `wined3d_format_is_typeless` // gen:71392b7d1558
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_format_is_typeless(format: *const wined3d_format) -> BOOL {
    ((*format).id == (*format).typeless_id && (*format).id != WINED3DFMT_UNKNOWN) as i32
}
// dll/directx/wine/wined3d/wined3d_vk.h:1086: header-inline `wined3d_format_vk` // gen:ea3ee60f9580
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_format_vk(format: *const wined3d_format) -> *const wined3d_format_vk {
    CONTAINING_RECORD!(format, wined3d_format_vk, f)
}
// dll/directx/wine/wined3d/wined3d_private.h:4715: header-inline `wined3d_from_cs` // gen:8499ec941f95
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_from_cs(cs: *const wined3d_cs) {
    if !(*cs).thread.is_null() {
        assert!((*cs).thread_id == GetCurrentThreadId());
    }
}
// dll/directx/wine/wined3d/wined3d_private.h:3821: header-inline `wined3d_ge_wrap` // gen:08b03ae4ff3b
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_ge_wrap(x: ULONG, y: ULONG) -> BOOL {
    (x.wrapping_sub(y) < UINT_MAX / 2) as BOOL
}
// dll/directx/wine/wined3d/wined3d_private.h:83: header-inline `wined3d_get_line` // gen:434937cbbc73
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_get_line(ptr: *mut *const core::ffi::c_char, end: *const core::ffi::c_char) -> *const core::ffi::c_char {
    let p: *const core::ffi::c_char;
    let q: *const core::ffi::c_char;

    p = *ptr;
    if p >= end {
        return core::ptr::null();
    }

    q = memchr(
        p as *const core::ffi::c_void,
        b'\n' as core::ffi::c_int,
        end.offset_from(p) as usize,
    ) as *const core::ffi::c_char;
    if q.is_null() {
        *ptr = end;
    } else {
        *ptr = q.add(1);
    }
    p
}
// dll/directx/wine/wined3d/wined3d_private.h:4736: header-inline `wined3d_get_material_colour_source` // gen:7c31a744786c
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_get_material_colour_source(
    diffuse: *mut wined3d_material_color_source,
    emissive: *mut wined3d_material_color_source,
    ambient: *mut wined3d_material_color_source,
    specular: *mut wined3d_material_color_source,
    state: *const wined3d_state,
) {
    let vdecl: *const wined3d_vertex_declaration = (*state).vertex_declaration;

    if (*state).render_states[WINED3D_RS_LIGHTING as usize] == 0 {
        *diffuse = WINED3D_MCS_COLOR1;
        *specular = WINED3D_MCS_COLOR2;
        *emissive = WINED3D_MCS_MATERIAL;
        *ambient = WINED3D_MCS_MATERIAL;
        return;
    }

    if (*state).render_states[WINED3D_RS_COLORVERTEX as usize] == 0 {
        *diffuse = WINED3D_MCS_MATERIAL;
        *emissive = WINED3D_MCS_MATERIAL;
        *ambient = WINED3D_MCS_MATERIAL;
        *specular = WINED3D_MCS_MATERIAL;
        return;
    }

    *diffuse = validate_material_colour_source(vdecl, (*state).render_states[WINED3D_RS_DIFFUSEMATERIALSOURCE as usize]);
    *emissive = validate_material_colour_source(vdecl, (*state).render_states[WINED3D_RS_EMISSIVEMATERIALSOURCE as usize]);
    *ambient = validate_material_colour_source(vdecl, (*state).render_states[WINED3D_RS_AMBIENTMATERIALSOURCE as usize]);
    *specular = validate_material_colour_source(vdecl, (*state).render_states[WINED3D_RS_SPECULARMATERIALSOURCE as usize]);
}
// dll/directx/wine/wined3d/wined3d_gl.h:1161: header-inline `wined3d_gl_get_internal_format` // gen:b5b99e20b6c1
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_gl_get_internal_format(
    resource: *mut wined3d_resource,
    format_gl: *const wined3d_format_gl,
    srgb: bool,
) -> GLuint {
    if srgb {
        ((*format_gl).srgb_internal) as u32
    } else if ((*resource).bind_flags & WINED3D_BIND_RENDER_TARGET) != 0 && ((wined3d_resource_is_offscreen(resource)) != 0) {
        ((*format_gl).rt_internal) as u32
    } else {
        ((*format_gl).internal) as u32
    }
}
// dll/directx/wine/wined3d/wined3d_gl.h:377: header-inline `wined3d_gl_mag_filter` // gen:ed19ef97e449
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_gl_mag_filter(mag_filter: wined3d_texture_filter_type) -> GLenum {
    magLookup[mag_filter as usize]
}
// dll/directx/wine/wined3d/wined3d_gl.h:382: header-inline `wined3d_gl_min_mip_filter` // gen:d48b9e9fdfcc
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_gl_min_mip_filter(
    min_filter: wined3d_texture_filter_type,
    mip_filter: wined3d_texture_filter_type,
) -> GLenum {
    minMipLookup[min_filter as usize].mip[mip_filter as usize]
}
// dll/directx/wine/wined3d/wined3d_private.h:4697: header-inline `wined3d_insert_bits` // gen:54d51628a778
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_insert_bits(bitstream: *mut u32, offset: u32, count: u32, bits: DWORD) {
    let word_bit_count: u32 = (core::mem::size_of::<u32>() as u32) * CHAR_BIT;
    let idx: u32 = offset / word_bit_count;
    let shift: u32 = offset % word_bit_count;
    let mut mask: DWORD = (1u32 << count) - 1;

    *bitstream.add(idx as usize) |= (bits & mask) << shift;
    if shift + count > word_bit_count {
        let inserted_bit_count: u32 = word_bit_count - shift;
        let remaining_bit_count: u32 = count - inserted_bit_count;
        mask = (1u32 << remaining_bit_count) - 1;
        *bitstream.add((idx + 1) as usize) |= (bits >> inserted_bit_count) & mask;
    }
}
// dll/directx/wine/wined3d/wined3d_private.h:4913: header-inline `wined3d_is_srv_rtv_bound` // gen:64777d2860c2
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_is_srv_rtv_bound(state: *const wined3d_state, srv: *const wined3d_shader_resource_view) -> bool {
    let mut i: u32 = 0;

    if (*(*srv).resource).rtv_bind_count_device == 0 {
        return false;
    }

    while i < (ARRAY_SIZE!((*state).fb.render_targets) as u32) {
        let rtv = (*state).fb.render_targets[i as usize];
        if !rtv.is_null() && wined3d_rtv_overlaps_srv(rtv, srv) {
            return true;
        }
        i += 1;
    }

    false
}
// dll/directx/wine/wined3d/wined3d_private.h:4161: header-inline `wined3d_lock_cleanup` // gen:ab0ec35b5109
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_lock_cleanup(lock: *mut CRITICAL_SECTION) {
    if (*lock).DebugInfo != (usize::MAX as *mut RTL_CRITICAL_SECTION_DEBUG) {
        (*(*lock).DebugInfo).__bindgen_anon_1.Spare[0] = 0;
    }
    DeleteCriticalSection(lock);
}
// dll/directx/wine/wined3d/wined3d_private.h:4154: header-inline `wined3d_lock_init_` // gen:97493a77a3b7
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_lock_init_(lock: *mut CRITICAL_SECTION, name: *const core::ffi::c_char) {
    InitializeCriticalSectionEx(lock, 0, RTL_CRITICAL_SECTION_FLAG_FORCE_DEBUG_INFO);
    if (*lock).DebugInfo != (-1isize) as *mut RTL_CRITICAL_SECTION_DEBUG {
        (*(*lock).DebugInfo).__bindgen_anon_1.Spare[0] = name as DWORD_PTR;
    }
}
// dll/directx/wine/wined3d/wined3d_private.h:4845: header-inline `wined3d_bind_layer_count`
// (macros did not buy it: no translated body called it; two header inlines here do).
#[inline]
pub unsafe fn wined3d_bind_layer_count(texture: *const wined3d_texture) -> u32 {
    if (*texture).resource.type_ == WINED3D_RTYPE_TEXTURE_3D {
        (*texture).resource.depth
    } else {
        (*texture).layer_count
    }
}
// dll/directx/wine/wined3d/wined3d_private.h:5034: header-inline `wined3d_map_persistent` // gen:e81b62885fe6
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_map_persistent() -> bool {
    core::mem::size_of::<*const core::ffi::c_void>() >= core::mem::size_of::<uint64_t>()
}
// dll/directx/wine/wined3d/wined3d_private.h:4672: header-inline `wined3d_mask_from_size` // gen:fe84a29544b9
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_mask_from_size(size: u32) -> uint32_t {
    if size < 32 {
        (1u32 << size) - 1
    } else {
        !0u32
    }
}
// dll/directx/wine/wined3d/wined3d_private.h:4721: header-inline `wined3d_not_from_cs` // gen:f88cde775eaf
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_not_from_cs(cs: *const wined3d_cs) {
    assert!((*cs).thread_id != GetCurrentThreadId());
}
// dll/directx/wine/wined3d/wined3d_private.h:3813: header-inline `wined3d_pause` // gen:59d34ea04f77
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_pause(spin_count: *mut u32) {
    static mut timeout: LARGE_INTEGER = LARGE_INTEGER {
        QuadPart: (WINED3D_CS_CLIENT_WAIT_TIMEOUT as i64) * -10,
    };

    *spin_count += 1;
    if *spin_count >= WINED3D_PAUSE_SPIN_COUNT {
        NtDelayExecution((FALSE) as BOOLEAN, &raw mut timeout);
    }
}
// dll/directx/wine/wined3d/wined3d_private.h:432: header-inline `wined3d_popcount` // gen:2520b7697582
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_popcount(mut x: u32) -> u32 {
    x = x.wrapping_sub((x >> 1) & 0x55555555);
    x = (x & 0x33333333).wrapping_add((x >> 2) & 0x33333333);
    ((x.wrapping_add(x >> 4) & 0x0f0f0f0f).wrapping_mul(0x01010101)) >> 24
}
// dll/directx/wine/wined3d/wined3d_private.h:5024: header-inline `wined3d_primitive_type_is_list` // gen:ad69b914164f
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_primitive_type_is_list(t: wined3d_primitive_type) -> bool {
    t == WINED3D_PT_POINTLIST
        || t == WINED3D_PT_LINELIST
        || t == WINED3D_PT_TRIANGLELIST
        || t == WINED3D_PT_LINELIST_ADJ
        || t == WINED3D_PT_TRIANGLELIST_ADJ
        || t == WINED3D_PT_PATCH
}
// dll/directx/wine/wined3d/wined3d_vk.h:407: header-inline `wined3d_query_vk` // gen:621f35323e74
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_query_vk(query: *mut wined3d_query) -> *mut wined3d_query_vk {
    CONTAINING_RECORD!(query, wined3d_query_vk, q)
}
// dll/directx/wine/wined3d/wined3d_gl.h:1069: header-inline `wined3d_rendertarget_view_gl` // gen:6de320b15513
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_rendertarget_view_gl(view: *mut wined3d_rendertarget_view) -> *mut wined3d_rendertarget_view_gl {
    CONTAINING_RECORD!(view, wined3d_rendertarget_view_gl, v)
}
// dll/directx/wine/wined3d/wined3d_vk.h:955: header-inline `wined3d_rendertarget_view_vk` // gen:632ec423258f
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_rendertarget_view_vk(view: *mut wined3d_rendertarget_view) -> *mut wined3d_rendertarget_view_vk {
    CONTAINING_RECORD!(view, wined3d_rendertarget_view_vk, v)
}
// dll/directx/wine/wined3d/wined3d_vk.h:961: header-inline `wined3d_rendertarget_view_vk_barrier` // gen:69fd7918457b
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_rendertarget_view_vk_barrier(
    rtv_vk: *mut wined3d_rendertarget_view_vk,
    context_vk: *mut wined3d_context_vk,
    bind_mask: uint32_t,
) {
    wined3d_resource_vk_barrier((*rtv_vk).v.resource, context_vk, bind_mask);
}
// dll/directx/wine/wined3d/wined3d_vk.h:967: header-inline `wined3d_rendertarget_view_vk_get_image_view` // gen:8f9e9b9de73b
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_rendertarget_view_vk_get_image_view(
    rtv_vk: *mut wined3d_rendertarget_view_vk,
    context_vk: *mut wined3d_context_vk,
) -> VkImageView {
    let texture_vk: *mut wined3d_texture_vk;

    if (*rtv_vk).vk_image_view != 0 {
        return (*rtv_vk).vk_image_view;
    }

    texture_vk = wined3d_texture_vk(wined3d_texture_from_resource((*rtv_vk).v.resource));
    (*wined3d_texture_vk_get_default_image_info(texture_vk, context_vk)).imageView
}
// dll/directx/wine/wined3d/wined3d_private.h:3283: header-inline `wined3d_resource_decref` // gen:f42e0ae9f4e1
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_resource_decref(resource: *mut wined3d_resource) -> ULONG {
    ((*(*resource).resource_ops).resource_decref.unwrap_unchecked())(resource)
}
// dll/directx/wine/wined3d/wined3d_private.h:3288: header-inline `wined3d_resource_get_sub_resource_desc` // gen:8f7c7e20783f
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_resource_get_sub_resource_desc(
    resource: *mut wined3d_resource,
    sub_resource_idx: u32,
    desc: *mut wined3d_sub_resource_desc,
) -> HRESULT {
    ((*(*resource).resource_ops).resource_sub_resource_get_desc.unwrap_unchecked())(resource, sub_resource_idx, desc)
}
// dll/directx/wine/wined3d/wined3d_private.h:3294: header-inline `wined3d_resource_get_sub_resource_map_pitch` // gen:940e8ac1aeab
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_resource_get_sub_resource_map_pitch(
    resource: *mut wined3d_resource,
    sub_resource_idx: u32,
    row_pitch: *mut u32,
    slice_pitch: *mut u32,
) {
    ((*(*resource).resource_ops).resource_sub_resource_get_map_pitch.unwrap_unchecked())(
        resource, sub_resource_idx, row_pitch, slice_pitch,
    );
}
// dll/directx/wine/wined3d/wined3d_private.h:3278: header-inline `wined3d_resource_incref` // gen:567d6790005c
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_resource_incref(resource: *mut wined3d_resource) -> ULONG {
    ((*(*resource).resource_ops).resource_incref.unwrap_unchecked())(resource)
}
// dll/directx/wine/wined3d/wined3d_private.h:3805: header-inline `wined3d_resource_reference` // gen:af432e22a5f8
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_resource_reference(resource: *mut wined3d_resource) {
    let cs: *const wined3d_cs = (*(*resource).device).cs;
    (*resource).access_time = (*cs).queue[WINED3D_CS_QUEUE_DEFAULT as usize].head;
}
// dll/directx/wine/wined3d/wined3d_vk.h:938: header-inline `wined3d_resource_vk_barrier` // gen:b04eb63fc2d8
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_resource_vk_barrier(
    resource: *mut wined3d_resource,
    context_vk: *mut wined3d_context_vk,
    bind_mask: uint32_t,
) {
    if (*resource).type_ == WINED3D_RTYPE_BUFFER {
        wined3d_buffer_vk_barrier(
            wined3d_buffer_vk(buffer_from_resource(resource)),
            context_vk,
            bind_mask,
        );
    } else {
        wined3d_texture_vk_barrier(
            wined3d_texture_vk(texture_from_resource(resource)),
            context_vk,
            bind_mask,
        );
    }
}
// dll/directx/wine/wined3d/wined3d_private.h:3827: header-inline `wined3d_resource_wait_idle` // gen:2bb8c5648cff
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_resource_wait_idle(resource: *const wined3d_resource) {
    let cs: *const wined3d_cs = (*(*resource).device).cs;
    let mut tail: ULONG;
    let mut spin_count: u32 = 0;

    if (*cs).thread.is_null() || (*cs).thread_id == GetCurrentThreadId() {
        return;
    }

    let access_time: ULONG = (*resource).access_time;
    let head: ULONG = (*cs).queue[WINED3D_CS_QUEUE_DEFAULT as usize].head;

    if ((wined3d_ge_wrap(head, access_time)) == 0) {
        return;
    }

    loop {
        tail = core::ptr::read_volatile(&(*cs).queue[WINED3D_CS_QUEUE_DEFAULT as usize].tail);
        if head == tail {
            break;
        }

        if ((wined3d_ge_wrap(access_time, tail)) == 0) && access_time != tail {
            break;
        }

        wined3d_pause(&mut spin_count);
    }
}
// dll/directx/wine/wined3d/wined3d_private.h:4865: header-inline `wined3d_rtv_all_subresources` // gen:910cdac69714
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_rtv_all_subresources(rtv: *const wined3d_rendertarget_view) -> bool {
    if (*(*rtv).resource).type_ == WINED3D_RTYPE_BUFFER {
        return ((TRUE) != 0);
    }

    if (*rtv).sub_resource_idx != 0 {
        return ((FALSE) != 0);
    }

    let texture = texture_from_resource((*rtv).resource);
    (*texture).level_count == 1 && (*rtv).layer_count == wined3d_bind_layer_count(texture)
}
// dll/directx/wine/wined3d/wined3d_private.h:4894: header-inline `wined3d_rtv_bind_count_dec` // gen:a1b0d728b64f
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_rtv_bind_count_dec(rtv: *mut wined3d_rendertarget_view) {
    InterlockedDecrement(&mut (*(*rtv).resource).rtv_bind_count_device);
}
// dll/directx/wine/wined3d/wined3d_private.h:4889: header-inline `wined3d_rtv_bind_count_inc` // gen:c93ae1ef81ee
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_rtv_bind_count_inc(rtv: *mut wined3d_rendertarget_view) {
    InterlockedIncrement(&mut (*(*rtv).resource).rtv_bind_count_device);
}
// dll/directx/wine/wined3d/wined3d_private.h:4899: header-inline `wined3d_rtv_overlaps_srv` // gen:82f88fecebe8
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_rtv_overlaps_srv(
    rtv: *const wined3d_rendertarget_view,
    srv: *const wined3d_shader_resource_view,
) -> bool {
    if (*rtv).resource != (*srv).resource {
        return false;
    }

    if wined3d_srv_all_subresources(srv) || wined3d_rtv_all_subresources(rtv) {
        return true;
    }

    (*rtv).sub_resource_idx >= (*srv).desc.u.texture.level_idx
        && (*rtv).sub_resource_idx
            < (*srv).desc.u.texture.level_idx + (*srv).desc.u.texture.level_count
        && (*rtv).layer_count >= (*srv).desc.u.texture.layer_idx
}
// dll/directx/wine/wined3d/wined3d_gl.h:1034: header-inline `wined3d_sampler_gl` // gen:03dbb31e90c4
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_sampler_gl(sampler: *mut wined3d_sampler) -> *mut wined3d_sampler_gl {
    CONTAINING_RECORD!(sampler, wined3d_sampler_gl, s)
}
// dll/directx/wine/wined3d/wined3d_vk.h:909: header-inline `wined3d_sampler_vk` // gen:552f7bef5959
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_sampler_vk(sampler: *mut wined3d_sampler) -> *mut wined3d_sampler_vk {
    CONTAINING_RECORD!(sampler, wined3d_sampler_vk, s)
}
// dll/directx/wine/wined3d/wined3d_private.h:314: header-inline `wined3d_sanitize_cmp_func` // gen:8be737260386
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_sanitize_cmp_func(func: wined3d_cmp_func) -> wined3d_cmp_func {
    if func < WINED3D_CMP_NEVER || func > WINED3D_CMP_ALWAYS {
        return WINED3D_CMP_ALWAYS;
    }
    func
}
// dll/directx/wine/wined3d/wined3d_private.h:1367: header-inline `wined3d_shader_instruction_has_texel_offset` // gen:afcf5298a1f6
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_shader_instruction_has_texel_offset(ins: *const wined3d_shader_instruction) -> BOOL {
    ((*ins).texel_offset.u != 0 || (*ins).texel_offset.v != 0 || (*ins).texel_offset.w != 0) as i32
}
// dll/directx/wine/wined3d/wined3d_gl.h:1086: header-inline `wined3d_shader_resource_view_gl` // gen:4c63939f404f
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_shader_resource_view_gl(view: *mut wined3d_shader_resource_view) -> *mut wined3d_shader_resource_view_gl {
    CONTAINING_RECORD!(view, wined3d_shader_resource_view_gl, v)
}
// dll/directx/wine/wined3d/wined3d_vk.h:1000: header-inline `wined3d_shader_resource_view_vk` // gen:b32f31fdbb5b
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_shader_resource_view_vk(view: *mut wined3d_shader_resource_view) -> *mut wined3d_shader_resource_view_vk {
    CONTAINING_RECORD!(view, wined3d_shader_resource_view_vk, v)
}
// dll/directx/wine/wined3d/wined3d_vk.h:1006: header-inline `wined3d_shader_resource_view_vk_barrier` // gen:8e9cba344fd2
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_shader_resource_view_vk_barrier(
    srv_vk: *mut wined3d_shader_resource_view_vk,
    context_vk: *mut wined3d_context_vk,
    bind_mask: uint32_t,
) {
    wined3d_resource_vk_barrier((*srv_vk).v.resource, context_vk, bind_mask);
}
// dll/directx/wine/wined3d/wined3d_private.h:1573: header-inline `wined3d_srgb_from_linear` // gen:fc9d01112f80
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_srgb_from_linear(colour: f32) -> f32 {
    if colour < 0.0 {
        return 0.0;
    }
    if colour < wined3d_srgb_const[1].x {
        return colour * wined3d_srgb_const[0].w;
    }
    if colour < 1.0 {
        return wined3d_srgb_const[0].y * powf(colour, wined3d_srgb_const[0].x)
            - wined3d_srgb_const[0].z;
    }
    1.0
}
// dll/directx/wine/wined3d/wined3d_private.h:4850: header-inline `wined3d_srv_all_subresources` // gen:808f79d9be38
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_srv_all_subresources(srv: *const wined3d_shader_resource_view) -> bool {
    if (*(*srv).resource).type_ == WINED3D_RTYPE_BUFFER {
        return ((TRUE) != 0);
    }

    if (*srv).desc.u.texture.layer_idx != 0 || (*srv).desc.u.texture.level_idx != 0 {
        return ((FALSE) != 0);
    }

    let texture = texture_from_resource((*srv).resource);
    (*srv).desc.u.texture.level_count == (*texture).level_count
        && (*srv).desc.u.texture.layer_count == wined3d_bind_layer_count(texture)
}
// dll/directx/wine/wined3d/wined3d_private.h:4884: header-inline `wined3d_srv_bind_count_dec` // gen:f379c9c07865
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_srv_bind_count_dec(srv: *mut wined3d_shader_resource_view) {
    InterlockedDecrement(&mut (*(*srv).resource).srv_bind_count_device);
}
// dll/directx/wine/wined3d/wined3d_private.h:4879: header-inline `wined3d_srv_bind_count_inc` // gen:6da5db807b03
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_srv_bind_count_inc(srv: *mut wined3d_shader_resource_view) {
    InterlockedIncrement(&mut (*(*srv).resource).srv_bind_count_device);
}
// dll/directx/wine/wined3d/wined3d_private.h:3987: header-inline `wined3d_state_get_ffp_texture` // gen:fa44c161d75f
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_state_get_ffp_texture(state: *const wined3d_state, idx: u32) -> *mut wined3d_texture {
    let view = (*state).shader_resource_view[WINED3D_SHADER_TYPE_PIXEL as usize][idx as usize];

    assert!(idx <= WINED3D_MAX_FFP_TEXTURES);
    if !view.is_null() {
        texture_from_resource((*view).resource)
    } else {
        core::ptr::null_mut()
    }
}
// dll/directx/wine/wined3d/wined3d_private.h:2976: header-inline `wined3d_state_reset` // gen:9ce1c1fd3c2a
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_state_reset(state: *mut wined3d_state, d3d_info: *const wined3d_d3d_info) {
    let feature_level: wined3d_feature_level = (*state).feature_level;
    let flags: uint32_t = (*state).flags;

    memset(state as *mut core::ffi::c_void, 0, core::mem::size_of::<wined3d_state>());
    state_init(state, d3d_info, flags, feature_level);
}
// dll/directx/wine/wined3d/wined3d_private.h:2985: header-inline `wined3d_state_uses_depth_buffer` // gen:0a26e7f03737
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_state_uses_depth_buffer(state: *const wined3d_state) -> bool {
    if (*state).depth_stencil_state.is_null() {
        return true;
    }
    (((*(*state).depth_stencil_state).desc.depth) != 0)
        || (((*(*state).depth_stencil_state).desc.depth_write) != 0)
        || (((*(*state).depth_stencil_state).desc.stencil) != 0)
}
// dll/directx/wine/wined3d/wined3d_gl.h:1129: header-inline `wined3d_swapchain_gl` // gen:e181e4b1557b
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_swapchain_gl(swapchain: *mut wined3d_swapchain) -> *mut wined3d_swapchain_gl {
    CONTAINING_RECORD!(swapchain, wined3d_swapchain_gl, s)
}
// dll/directx/wine/wined3d/wined3d_vk.h:1068: header-inline `wined3d_swapchain_vk` // gen:f7139e13d421
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_swapchain_vk(swapchain: *mut wined3d_swapchain) -> *mut wined3d_swapchain_vk {
    CONTAINING_RECORD!(swapchain, wined3d_swapchain_vk, s)
}
// dll/directx/wine/wined3d/wined3d_private.h:3459: header-inline `wined3d_texture_allocate_object_memory` // gen:fab6f6193e6e
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_texture_allocate_object_memory(s: SIZE_T, level_count: SIZE_T, layer_count: SIZE_T) -> *mut core::ffi::c_void {
    let elem: SIZE_T = (core::mem::size_of::<wined3d_texture_sub_resource>()) as _;

    if level_count > ((!(0 as SIZE_T) - s) / elem) / layer_count {
        return core::ptr::null_mut();
    }

    calloc(1, (s + level_count * layer_count * elem) as u32)
}
// dll/directx/wine/wined3d/wined3d_private.h:3492: header-inline `wined3d_texture_get_level_box` // gen:1d8c2ff39d55
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_texture_get_level_box(texture: *const wined3d_texture, level: u32, box_: *mut wined3d_box) {
    wined3d_box_set(
        box_,
        0,
        0,
        wined3d_texture_get_level_width(texture, level),
        wined3d_texture_get_level_height(texture, level),
        0,
        wined3d_texture_get_level_depth(texture, level),
    );
}
// dll/directx/wine/wined3d/wined3d_private.h:3486: header-inline `wined3d_texture_get_level_depth` // gen:22e55d9b8d35
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_texture_get_level_depth(texture: *const wined3d_texture, level: u32) -> u32 {
    max!(1, (*texture).resource.depth >> level)
}
// dll/directx/wine/wined3d/wined3d_private.h:3480: header-inline `wined3d_texture_get_level_height` // gen:f0b98f098c70
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_texture_get_level_height(texture: *const wined3d_texture, level: u32) -> u32 {
    max!(1, (*texture).resource.height >> level)
}
// dll/directx/wine/wined3d/wined3d_private.h:3474: header-inline `wined3d_texture_get_level_width` // gen:e365c536c36b
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_texture_get_level_width(texture: *const wined3d_texture, level: u32) -> u32 {
    max!(1, (*texture).resource.width >> level)
}
// dll/directx/wine/wined3d/wined3d_gl.h:954: header-inline `wined3d_texture_gl` // gen:de5da20266b2
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_texture_gl(texture: *mut wined3d_texture) -> *mut wined3d_texture_gl {
    CONTAINING_RECORD!(texture, wined3d_texture_gl, t)
}
// dll/directx/wine/wined3d/wined3d_gl.h:959: header-inline `wined3d_texture_gl_get_gl_texture` // gen:04a7146f6c02
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_texture_gl_get_gl_texture(texture_gl: *mut wined3d_texture_gl, srgb: bool) -> *mut gl_texture {
    if srgb {
        core::ptr::addr_of_mut!((*texture_gl).texture_srgb)
    } else {
        core::ptr::addr_of_mut!((*texture_gl).texture_rgb)
    }
}
// dll/directx/wine/wined3d/wined3d_gl.h:964: header-inline `wined3d_texture_gl_get_sub_resource_target` // gen:3f5a0cb22906
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_texture_gl_get_sub_resource_target(texture_gl: *const wined3d_texture_gl, sub_resource_idx: u32) -> GLenum {
    static cube_targets: [GLenum; 6] = [
        GL_TEXTURE_CUBE_MAP_POSITIVE_X_ARB,
        GL_TEXTURE_CUBE_MAP_NEGATIVE_X_ARB,
        GL_TEXTURE_CUBE_MAP_POSITIVE_Y_ARB,
        GL_TEXTURE_CUBE_MAP_NEGATIVE_Y_ARB,
        GL_TEXTURE_CUBE_MAP_POSITIVE_Z_ARB,
        GL_TEXTURE_CUBE_MAP_NEGATIVE_Z_ARB,
    ];

    if ((*texture_gl).t.resource.usage & WINED3DUSAGE_LEGACY_CUBEMAP) != 0 {
        return cube_targets[(sub_resource_idx / (*texture_gl).t.level_count) as usize];
    }
    (*texture_gl).target
}
// dll/directx/wine/wined3d/wined3d_gl.h:982: header-inline `wined3d_texture_gl_get_texture_name` // gen:30fc0186a183
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_texture_gl_get_texture_name(
    texture_gl: *const wined3d_texture_gl,
    context: *const wined3d_context,
    srgb: bool,
) -> GLuint {
    if srgb && ((needs_separate_srgb_gl_texture(context, &(*texture_gl).t)) != 0) {
        return (*texture_gl).texture_srgb.name;
    }
    (*texture_gl).texture_rgb.name
}
// dll/directx/wine/wined3d/wined3d_gl.h:990: header-inline `wined3d_texture_gl_is_multisample_location` // gen:bb5d8f2eda57
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_texture_gl_is_multisample_location(texture_gl: *const wined3d_texture_gl, location: uint32_t) -> bool {
    if location == WINED3D_LOCATION_RB_MULTISAMPLE {
        return true;
    }
    if location != WINED3D_LOCATION_TEXTURE_RGB && location != WINED3D_LOCATION_TEXTURE_SRGB {
        return false;
    }
    (*texture_gl).target == GL_TEXTURE_2D_MULTISAMPLE || (*texture_gl).target == GL_TEXTURE_2D_MULTISAMPLE_ARRAY
}
// dll/directx/wine/wined3d/wined3d_private.h:3501: header-inline `wined3d_texture_is_full_rect` // gen:e5617f78e080
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_texture_is_full_rect(texture: *const wined3d_texture, level: u32, r: *const RECT) -> bool {
    let mut t: u32;

    t = wined3d_texture_get_level_width(texture, level);
    if ((*r).left != 0 && (*r).right != 0) || (abs((*r).right - (*r).left) as u32) != t {
        return false;
    }

    t = wined3d_texture_get_level_height(texture, level);
    if ((*r).top != 0 && (*r).bottom != 0) || (abs((*r).bottom - (*r).top) as u32) != t {
        return false;
    }

    true
}
// dll/directx/wine/wined3d/wined3d_vk.h:884: header-inline `wined3d_texture_vk` // gen:42b0aef8487d
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_texture_vk(texture: *mut wined3d_texture) -> *mut wined3d_texture_vk {
    CONTAINING_RECORD!(texture, wined3d_texture_vk, t)
}
// dll/directx/wine/wined3d/wined3d_private.h:443: header-inline `wined3d_uint32_compare` // gen:dc79a97ae2c5
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_uint32_compare(x: uint32_t, y: uint32_t) -> i32 {
    ((x > y) as i32) - ((x < y) as i32)
}
// dll/directx/wine/wined3d/wined3d_private.h:448: header-inline `wined3d_uint64_compare` // gen:8c4c5f576afa
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_uint64_compare(x: uint64_t, y: uint64_t) -> i32 {
    ((x > y) as i32) - ((x < y) as i32)
}
// dll/directx/wine/wined3d/wined3d_gl.h:1110: header-inline `wined3d_unordered_access_view_gl` // gen:f08ef60f47a5
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_unordered_access_view_gl(view: *mut wined3d_unordered_access_view) -> *mut wined3d_unordered_access_view_gl {
    CONTAINING_RECORD!(view, wined3d_unordered_access_view_gl, v)
}
// dll/directx/wine/wined3d/wined3d_vk.h:1031: header-inline `wined3d_unordered_access_view_vk` // gen:32cd2a00a9b5
#[allow(dead_code)]
#[inline] pub unsafe fn wined3d_unordered_access_view_vk(view: *mut wined3d_unordered_access_view) -> *mut wined3d_unordered_access_view_vk {
    CONTAINING_RECORD!(view, wined3d_unordered_access_view_vk, v)
}
// dll/directx/wine/wined3d/wined3d_vk.h:1037: header-inline `wined3d_unordered_access_view_vk_barrier` // gen:d5e7f8234149
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_unordered_access_view_vk_barrier(
    uav_vk: *mut wined3d_unordered_access_view_vk,
    context_vk: *mut wined3d_context_vk,
    bind_mask: uint32_t,
) {
    wined3d_resource_vk_barrier((*uav_vk).v.resource, context_vk, bind_mask);
}
// dll/directx/wine/wined3d/wined3d_private.h:4764: header-inline `wined3d_vec4_transform` // gen:46cf5e48cc4b
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_vec4_transform(dst: *mut wined3d_vec4, v: *const wined3d_vec4, m: *const wined3d_matrix) {
    let tmp = wined3d_vec4 {
        x: (*v).x * (*m)._11 + (*v).y * (*m)._21 + (*v).z * (*m)._31 + (*v).w * (*m)._41,
        y: (*v).x * (*m)._12 + (*v).y * (*m)._22 + (*v).z * (*m)._32 + (*v).w * (*m)._42,
        z: (*v).x * (*m)._13 + (*v).y * (*m)._23 + (*v).z * (*m)._33 + (*v).w * (*m)._43,
        w: (*v).x * (*m)._14 + (*v).y * (*m)._24 + (*v).z * (*m)._34 + (*v).w * (*m)._44,
    };

    *dst = tmp;
}
// dll/directx/wine/wined3d/wined3d_private.h:4932: header-inline `wined3d_viewport_get_z_range` // gen:68d1995bf346
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_viewport_get_z_range(vp: *const wined3d_viewport, min_z: *mut f32, max_z: *mut f32) {
    *min_z = (*vp).min_z;
    *max_z = max!((*vp).max_z, (*vp).min_z + 0.001);
}
// ==== end port macros ====

// ==== port resolve: INITGUID storage (generated) ====
#[no_mangle]
pub static IID_ID3D10Blob: GUID = GUID { Data1: 0x8ba5fb08, Data2: 0x5195, Data3: 0x40e2, Data4: [0xac, 0x58, 0x0d, 0x98, 0x9c, 0x3a, 0x01, 0x02] };
#[no_mangle]
pub static IID_ID3D12CommandAllocator: GUID = GUID { Data1: 0x6102dee4, Data2: 0xaf59, Data3: 0x4b09, Data4: [0xb9, 0x99, 0xb4, 0x4d, 0x73, 0xf0, 0x9b, 0x24] };
#[no_mangle]
pub static IID_ID3D12CommandList: GUID = GUID { Data1: 0x7116d91c, Data2: 0xe7e4, Data3: 0x47ce, Data4: [0xb8, 0xc6, 0xec, 0x81, 0x68, 0xf4, 0x37, 0xe5] };
#[no_mangle]
pub static IID_ID3D12CommandQueue: GUID = GUID { Data1: 0x0ec870a6, Data2: 0x5d7e, Data3: 0x4c22, Data4: [0x8c, 0xfc, 0x5b, 0xaa, 0xe0, 0x76, 0x16, 0xed] };
#[no_mangle]
pub static IID_ID3D12CommandSignature: GUID = GUID { Data1: 0xc36a797c, Data2: 0xec80, Data3: 0x4f0a, Data4: [0x89, 0x85, 0xa7, 0xb2, 0x47, 0x50, 0x82, 0xd1] };
#[no_mangle]
pub static IID_ID3D12DescriptorHeap: GUID = GUID { Data1: 0x8efb471d, Data2: 0x616c, Data3: 0x4f49, Data4: [0x90, 0xf7, 0x12, 0x7b, 0xb7, 0x63, 0xfa, 0x51] };
#[no_mangle]
pub static IID_ID3D12Device: GUID = GUID { Data1: 0x189819f1, Data2: 0x1db6, Data3: 0x4b57, Data4: [0xbe, 0x54, 0x18, 0x21, 0x33, 0x9b, 0x85, 0xf7] };
#[no_mangle]
pub static IID_ID3D12Device1: GUID = GUID { Data1: 0x77acce80, Data2: 0x638e, Data3: 0x4e65, Data4: [0x88, 0x95, 0xc1, 0xf2, 0x33, 0x86, 0x86, 0x3e] };
#[no_mangle]
pub static IID_ID3D12Device2: GUID = GUID { Data1: 0x30baa41e, Data2: 0xb15b, Data3: 0x475c, Data4: [0xa0, 0xbb, 0x1a, 0xf5, 0xc5, 0xb6, 0x43, 0x28] };
#[no_mangle]
pub static IID_ID3D12Device3: GUID = GUID { Data1: 0x81dadc15, Data2: 0x2bad, Data3: 0x4392, Data4: [0x93, 0xc5, 0x10, 0x13, 0x45, 0xc4, 0xaa, 0x98] };
#[no_mangle]
pub static IID_ID3D12Device4: GUID = GUID { Data1: 0xe865df17, Data2: 0xa9ee, Data3: 0x46f9, Data4: [0xa4, 0x63, 0x30, 0x98, 0x31, 0x5a, 0xa2, 0xe5] };
#[no_mangle]
pub static IID_ID3D12Device5: GUID = GUID { Data1: 0x8b4f173b, Data2: 0x2fea, Data3: 0x4b80, Data4: [0x8f, 0x58, 0x43, 0x07, 0x19, 0x1a, 0xb9, 0x5d] };
#[no_mangle]
pub static IID_ID3D12Device6: GUID = GUID { Data1: 0xc70b221b, Data2: 0x40e4, Data3: 0x4a17, Data4: [0x89, 0xaf, 0x02, 0x5a, 0x07, 0x27, 0xa6, 0xdc] };
#[no_mangle]
pub static IID_ID3D12Device7: GUID = GUID { Data1: 0x5c014b53, Data2: 0x68a1, Data3: 0x4b9b, Data4: [0x8b, 0xd1, 0xdd, 0x60, 0x46, 0xb9, 0x35, 0x8b] };
#[no_mangle]
pub static IID_ID3D12Device8: GUID = GUID { Data1: 0x9218e6bb, Data2: 0xf944, Data3: 0x4f7e, Data4: [0xa7, 0x5c, 0xb1, 0xb2, 0xc7, 0xb7, 0x01, 0xf3] };
#[no_mangle]
pub static IID_ID3D12Device9: GUID = GUID { Data1: 0x4c80e962, Data2: 0xf032, Data3: 0x4f60, Data4: [0xbc, 0x9e, 0xeb, 0xc2, 0xcf, 0xa1, 0xd8, 0x3c] };
#[no_mangle]
pub static IID_ID3D12DeviceChild: GUID = GUID { Data1: 0x905db94b, Data2: 0xa00c, Data3: 0x4140, Data4: [0x9d, 0xf5, 0x2b, 0x64, 0xca, 0x9e, 0xa3, 0x57] };
#[no_mangle]
pub static IID_ID3D12Fence: GUID = GUID { Data1: 0x0a753dcf, Data2: 0xc4d8, Data3: 0x4b91, Data4: [0xad, 0xf6, 0xbe, 0x5a, 0x60, 0xd9, 0x5a, 0x76] };
#[no_mangle]
pub static IID_ID3D12Fence1: GUID = GUID { Data1: 0x433685fe, Data2: 0xe22b, Data3: 0x4ca0, Data4: [0xa8, 0xdb, 0xb5, 0xb4, 0xf4, 0xdd, 0x0e, 0x4a] };
#[no_mangle]
pub static IID_ID3D12GraphicsCommandList: GUID = GUID { Data1: 0x5b160d0f, Data2: 0xac1b, Data3: 0x4185, Data4: [0x8b, 0xa8, 0xb3, 0xae, 0x42, 0xa5, 0xa4, 0x55] };
#[no_mangle]
pub static IID_ID3D12GraphicsCommandList1: GUID = GUID { Data1: 0x553103fb, Data2: 0x1fe7, Data3: 0x4557, Data4: [0xbb, 0x38, 0x94, 0x6d, 0x7d, 0x0e, 0x7c, 0xa7] };
#[no_mangle]
pub static IID_ID3D12GraphicsCommandList2: GUID = GUID { Data1: 0x38c3e585, Data2: 0xff17, Data3: 0x412c, Data4: [0x91, 0x50, 0x4f, 0xc6, 0xf9, 0xd7, 0x2a, 0x28] };
#[no_mangle]
pub static IID_ID3D12GraphicsCommandList3: GUID = GUID { Data1: 0x6fda83a7, Data2: 0xb84c, Data3: 0x4e38, Data4: [0x9a, 0xc8, 0xc7, 0xbd, 0x22, 0x01, 0x6b, 0x3d] };
#[no_mangle]
pub static IID_ID3D12GraphicsCommandList4: GUID = GUID { Data1: 0x8754318e, Data2: 0xd3a9, Data3: 0x4541, Data4: [0x98, 0xcf, 0x64, 0x5b, 0x50, 0xdc, 0x48, 0x74] };
#[no_mangle]
pub static IID_ID3D12GraphicsCommandList5: GUID = GUID { Data1: 0x55050859, Data2: 0x4024, Data3: 0x474c, Data4: [0x87, 0xf5, 0x64, 0x72, 0xea, 0xee, 0x44, 0xea] };
#[no_mangle]
pub static IID_ID3D12GraphicsCommandList6: GUID = GUID { Data1: 0xc3827890, Data2: 0xe548, Data3: 0x4cfa, Data4: [0x96, 0xcf, 0x56, 0x89, 0xa9, 0x37, 0x0f, 0x80] };
#[no_mangle]
pub static IID_ID3D12Heap: GUID = GUID { Data1: 0x6b3b2502, Data2: 0x6e51, Data3: 0x45b3, Data4: [0x90, 0xee, 0x98, 0x84, 0x26, 0x5e, 0x8d, 0xf3] };
#[no_mangle]
pub static IID_ID3D12Object: GUID = GUID { Data1: 0xc4fec28f, Data2: 0x7966, Data3: 0x4e95, Data4: [0x9f, 0x94, 0xf4, 0x31, 0xcb, 0x56, 0xc3, 0xb8] };
#[no_mangle]
pub static IID_ID3D12Pageable: GUID = GUID { Data1: 0x63ee58fb, Data2: 0x1268, Data3: 0x4835, Data4: [0x86, 0xda, 0xf0, 0x08, 0xce, 0x62, 0xf0, 0xd6] };
#[no_mangle]
pub static IID_ID3D12PipelineState: GUID = GUID { Data1: 0x765a30f3, Data2: 0xf624, Data3: 0x4c6f, Data4: [0xa8, 0x28, 0xac, 0xe9, 0x48, 0x62, 0x24, 0x45] };
#[no_mangle]
pub static IID_ID3D12QueryHeap: GUID = GUID { Data1: 0x0d9658ae, Data2: 0xed45, Data3: 0x469e, Data4: [0xa6, 0x1d, 0x97, 0x0e, 0xc5, 0x83, 0xca, 0xb4] };
#[no_mangle]
pub static IID_ID3D12Resource: GUID = GUID { Data1: 0x696442be, Data2: 0xa72e, Data3: 0x4059, Data4: [0xbc, 0x79, 0x5b, 0x5c, 0x98, 0x04, 0x0f, 0xad] };
#[no_mangle]
pub static IID_ID3D12Resource1: GUID = GUID { Data1: 0x9d5e227a, Data2: 0x4430, Data3: 0x4161, Data4: [0x88, 0xb3, 0x3e, 0xca, 0x6b, 0xb1, 0x6e, 0x19] };
#[no_mangle]
pub static IID_ID3D12Resource2: GUID = GUID { Data1: 0xbe36ec3b, Data2: 0xea85, Data3: 0x4aeb, Data4: [0xa4, 0x5a, 0xe9, 0xd7, 0x64, 0x04, 0xa4, 0x95] };
#[no_mangle]
pub static IID_ID3D12RootSignature: GUID = GUID { Data1: 0xc54a6b66, Data2: 0x72df, Data3: 0x4ee8, Data4: [0x8b, 0xe5, 0xa9, 0x46, 0xa1, 0x42, 0x92, 0x14] };
#[no_mangle]
pub static IID_ID3D12RootSignatureDeserializer: GUID = GUID { Data1: 0x34ab647b, Data2: 0x3cc8, Data3: 0x46ac, Data4: [0x84, 0x1b, 0xc0, 0x96, 0x56, 0x45, 0xc0, 0x46] };
#[no_mangle]
pub static IID_ID3D12ShaderCacheSession: GUID = GUID { Data1: 0x28e2495d, Data2: 0x0f64, Data3: 0x4ae4, Data4: [0xa6, 0xec, 0x12, 0x92, 0x55, 0xdc, 0x49, 0xa8] };
#[no_mangle]
pub static IID_ID3D12VersionedRootSignatureDeserializer: GUID = GUID { Data1: 0x7f91ce67, Data2: 0x090c, Data3: 0x4bb7, Data4: [0xb7, 0x8e, 0xed, 0x8f, 0xf2, 0xe3, 0x1d, 0xa0] };
// ==== end INITGUID storage ====
