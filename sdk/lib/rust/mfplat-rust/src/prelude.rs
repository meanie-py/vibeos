//! mfplat-rust prelude — the single import surface the translated modules use.
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
// wine/debug.h's TRACE class. Hand-kept: two modules carry PRIVATE copies, so `port resolve`
// sees the name as defined and never copies a crate-wide one (180 E0425 across the rest).
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3;
/// wine/debug.h's inline.
pub unsafe fn wine_dbgstr_longlong(ll: ULONGLONG) -> *const ::core::ffi::c_char {
    if ll >> 32 != 0 {
        wine_dbg_sprintf(b"%lx%08lx\0".as_ptr() as *const ::core::ffi::c_char, (ll >> 32) as u32, ll as u32)
    } else {
        wine_dbg_sprintf(b"%lx\0".as_ptr() as *const ::core::ffi::c_char, ll as u32)
    }
}

// ================= hand: CRT / winnt.h inlines the island cannot carry ========================
extern "C" {
    // msvcrt exports the underscored name; oldnames supplies `wcsdup` to the C build.
    #[link_name = "_wcsdup"]
    pub fn wcsdup(s: *const u16) -> *mut u16;
}
/// stdlib.h. msvcrt's `llabs(LLONG_MIN)` returns it unchanged, which is what wrapping does.
#[inline(always)]
pub fn llabs(v: i64) -> i64 {
    v.wrapping_abs()
}
/// math.h `round`: half away from zero. msvcrt has no export, and `f64::round` is std-only.
/// `a - t` is exact below 2^52, so the comparison cannot misround a value like 0.49999999999999994.
pub fn round(x: f64) -> f64 {
    let a = if x < 0.0 { -x } else { x };
    if !(a < 4503599627370496.0) {
        return x; // NaN, infinity, or already integral
    }
    let t = (a as u64) as f64;
    let r = if a - t >= 0.5 { t + 1.0 } else { t };
    if x.is_sign_negative() { -r } else { r }
}
/// winnt.h intrinsic: index of the highest set bit; FALSE (and `*index` untouched) for 0.
#[inline(always)]
pub unsafe fn BitScanReverse(index: *mut ::core::ffi::c_ulong, mask: ::core::ffi::c_ulong) -> BOOLEAN {
    if mask == 0 {
        return 0;
    }
    *index = 31 - (mask as u32).leading_zeros() as ::core::ffi::c_ulong;
    1
}
#[inline(always)]
pub unsafe fn InterlockedCompareExchangePointer(target: *mut *mut c_void, exchange: *mut c_void, comparand: *mut c_void) -> *mut c_void {
    use core::sync::atomic::{AtomicPtr, Ordering::SeqCst};
    match (*(target as *const AtomicPtr<c_void>)).compare_exchange(comparand, exchange, SeqCst, SeqCst) {
        Ok(v) | Err(v) => v,
    }
}
#[inline(always)]
pub unsafe fn InterlockedExchangePointer(target: *mut *mut c_void, value: *mut c_void) -> *mut c_void {
    (*(target as *const core::sync::atomic::AtomicPtr<c_void>)).swap(value, core::sync::atomic::Ordering::SeqCst)
}
/// winbase.h: RTL_RUN_ONCE_INIT, all zeroes.
pub const INIT_ONCE_STATIC_INIT: INIT_ONCE = unsafe { core::mem::zeroed() };
/// winerror.h: HRESULT_FROM_WIN32(ERROR_INSUFFICIENT_BUFFER).
pub const E_NOT_SUFFICIENT_BUFFER: HRESULT = 0x8007_007Au32 as HRESULT;

// HAND (moved out of the macros block so the fixes stick): mfplat_private.h:88 header-inline `mf_array_reserve`
#[allow(dead_code)]
#[inline]
pub unsafe fn mf_array_reserve(
    elements: *mut *mut core::ffi::c_void,
    capacity: *mut size_t,
    count: size_t,
    size: size_t,
) -> BOOL {
    if count <= *capacity {
        return (TRUE) as BOOL;
    }

    let max_capacity: size_t = !(0 as size_t) / size;
    if count > max_capacity {
        return (FALSE) as BOOL;
    }

    let mut new_capacity: size_t = max!(4, *capacity);
    while new_capacity < count && new_capacity <= max_capacity / 2 {
        new_capacity *= 2;
    }
    if new_capacity < count {
        new_capacity = max_capacity;
    }

    let new_elements = realloc(*elements, new_capacity * size);
    if new_elements.is_null() {
        return (FALSE) as BOOL;
    }

    *elements = new_elements;
    *capacity = new_capacity;

    (TRUE as i32)
}
// HAND: mfplat_private.h:151 header-inline `mf_debugstr_fourcc`
#[allow(dead_code)]
#[inline]
pub unsafe fn mf_debugstr_fourcc(format: DWORD) -> *const core::ffi::c_char {
    let formats: [(u32, &core::ffi::CStr); 11] = [
        (D3DFMT_R8G8B8, c"R8G8B8"),
        (D3DFMT_A8R8G8B8, c"A8R8G8B8"),
        (D3DFMT_X8R8G8B8, c"X8R8G8B8"),
        (D3DFMT_R5G6B5, c"R5G6B5"),
        // Wine spells this one "X1R5G6B5"; the format is X1R5G5B5.
        (D3DFMT_X1R5G5B5, c"X1R5G5B5"),
        (D3DFMT_A2B10G10R10, c"A2B10G10R10"),
        (D3DFMT_P8, c"P8"),
        (D3DFMT_L8, c"L8"),
        (D3DFMT_D16, c"D16"),
        (D3DFMT_L16, c"L16"),
        (D3DFMT_A16B16G16R16F, c"A16B16G16R16F"),
    ];

    if (format & 0xff) == format {
        let mut i: usize = 0;
        while i < formats.len() {
            if formats[i].0 == format {
                return wine_dbg_sprintf(
                    c"%s".as_ptr(),
                    wine_dbgstr_an(formats[i].1.as_ptr(), -1),
                );
            }
            i += 1;
        }

        return wine_dbg_sprintf(c"%#lx".as_ptr(), format);
    }

    wine_dbgstr_an(&format as *const u32 as *const core::ffi::c_char, 4)
}

extern "C" {
    // uuid.lib's; main.c defined its own selectany copy under <initguid.h>.
    pub static IID_IUnknown: GUID;
}

// COBJMACROS' IStream_Read/IStream_Write (4 arguments), NOT shlwapi's 3-argument helpers of
// the same names: shlwapi.h declares those for C++ only, so the C always meant the vtable.
#[inline]
pub unsafe fn IStream_Read(This: *mut IStream, pv: *mut c_void, cb: ULONG, pcbRead: *mut ULONG) -> HRESULT {
    ((*(*This).lpVtbl).Read.unwrap_unchecked())(This, pv, cb, pcbRead)
}
#[inline]
pub unsafe fn IStream_Write(This: *mut IStream, pv: *const c_void, cb: ULONG, pcbWritten: *mut ULONG) -> HRESULT {
    ((*(*This).lpVtbl).Write.unwrap_unchecked())(This, pv, cb, pcbWritten)
}

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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const BI_BITFIELDS: u32 = 3; // access-rust/src/types_amd64.rs
pub const BI_RGB: u32 = 0; // access-rust/src/types_amd64.rs
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1; // cleanmgr-rust/src/prelude.rs
pub const D3D11_BIND_RENDER_TARGET: D3D11_BIND_FLAG = D3D11_BIND_FLAG_D3D11_BIND_RENDER_TARGET; // island enum
pub const D3D11_BIND_SHADER_RESOURCE: D3D11_BIND_FLAG = D3D11_BIND_FLAG_D3D11_BIND_SHADER_RESOURCE; // island enum
pub const D3D11_CPU_ACCESS_READ: D3D11_CPU_ACCESS_FLAG = D3D11_CPU_ACCESS_FLAG_D3D11_CPU_ACCESS_READ; // island enum
pub const D3D11_CPU_ACCESS_WRITE: D3D11_CPU_ACCESS_FLAG = D3D11_CPU_ACCESS_FLAG_D3D11_CPU_ACCESS_WRITE; // island enum
pub const D3D11_MAP_READ_WRITE: D3D11_MAP = D3D11_MAP_D3D11_MAP_READ_WRITE; // island enum
pub const D3D11_RESOURCE_MISC_SHARED: D3D11_RESOURCE_MISC_FLAG = D3D11_RESOURCE_MISC_FLAG_D3D11_RESOURCE_MISC_SHARED; // island enum
pub const D3D11_RESOURCE_MISC_SHARED_KEYEDMUTEX: D3D11_RESOURCE_MISC_FLAG = D3D11_RESOURCE_MISC_FLAG_D3D11_RESOURCE_MISC_SHARED_KEYEDMUTEX; // island enum
pub const D3D11_USAGE_DEFAULT: D3D11_USAGE = D3D11_USAGE_D3D11_USAGE_DEFAULT; // island enum
pub const D3D11_USAGE_DYNAMIC: D3D11_USAGE = D3D11_USAGE_D3D11_USAGE_DYNAMIC; // island enum
pub const D3D11_USAGE_IMMUTABLE: D3D11_USAGE = D3D11_USAGE_D3D11_USAGE_IMMUTABLE; // island enum
pub const D3D11_USAGE_STAGING: D3D11_USAGE = D3D11_USAGE_D3D11_USAGE_STAGING; // island enum
pub const D3D12_COMMAND_LIST_TYPE_COPY: D3D12_COMMAND_LIST_TYPE = D3D12_COMMAND_LIST_TYPE_D3D12_COMMAND_LIST_TYPE_COPY; // island enum
pub const D3D12_COMMAND_QUEUE_FLAG_NONE: D3D12_COMMAND_QUEUE_FLAGS = D3D12_COMMAND_QUEUE_FLAGS_D3D12_COMMAND_QUEUE_FLAG_NONE; // island enum
pub const D3D12_FENCE_FLAG_SHARED: D3D12_FENCE_FLAGS = D3D12_FENCE_FLAGS_D3D12_FENCE_FLAG_SHARED; // island enum
pub const D3D12_HEAP_FLAG_NONE: D3D12_HEAP_FLAGS = D3D12_HEAP_FLAGS_D3D12_HEAP_FLAG_NONE; // island enum
pub const D3D12_HEAP_TYPE_READBACK: D3D12_HEAP_TYPE = D3D12_HEAP_TYPE_D3D12_HEAP_TYPE_READBACK; // island enum
pub const D3D12_HEAP_TYPE_UPLOAD: D3D12_HEAP_TYPE = D3D12_HEAP_TYPE_D3D12_HEAP_TYPE_UPLOAD; // island enum
pub const D3D12_RESOURCE_BARRIER_FLAG_NONE: D3D12_RESOURCE_BARRIER_FLAGS = D3D12_RESOURCE_BARRIER_FLAGS_D3D12_RESOURCE_BARRIER_FLAG_NONE; // island enum
pub const D3D12_RESOURCE_BARRIER_TYPE_TRANSITION: D3D12_RESOURCE_BARRIER_TYPE = D3D12_RESOURCE_BARRIER_TYPE_D3D12_RESOURCE_BARRIER_TYPE_TRANSITION; // island enum
pub const D3D12_RESOURCE_DIMENSION_BUFFER: D3D12_RESOURCE_DIMENSION = D3D12_RESOURCE_DIMENSION_D3D12_RESOURCE_DIMENSION_BUFFER; // island enum
pub const D3D12_RESOURCE_DIMENSION_TEXTURE2D: D3D12_RESOURCE_DIMENSION = D3D12_RESOURCE_DIMENSION_D3D12_RESOURCE_DIMENSION_TEXTURE2D; // island enum
pub const D3D12_RESOURCE_FLAG_NONE: D3D12_RESOURCE_FLAGS = D3D12_RESOURCE_FLAGS_D3D12_RESOURCE_FLAG_NONE; // island enum
pub const D3D12_RESOURCE_STATE_COMMON: D3D12_RESOURCE_STATES = D3D12_RESOURCE_STATES_D3D12_RESOURCE_STATE_COMMON; // island enum
pub const D3D12_RESOURCE_STATE_COPY_DEST: D3D12_RESOURCE_STATES = D3D12_RESOURCE_STATES_D3D12_RESOURCE_STATE_COPY_DEST; // island enum
pub const D3D12_RESOURCE_STATE_COPY_SOURCE: D3D12_RESOURCE_STATES = D3D12_RESOURCE_STATES_D3D12_RESOURCE_STATE_COPY_SOURCE; // island enum
pub const D3D12_RESOURCE_STATE_GENERIC_READ: D3D12_RESOURCE_STATES = D3D12_RESOURCE_STATES_D3D12_RESOURCE_STATE_GENERIC_READ; // island enum
pub const D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT: D3D12_TEXTURE_COPY_TYPE = D3D12_TEXTURE_COPY_TYPE_D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT; // island enum
pub const D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX: D3D12_TEXTURE_COPY_TYPE = D3D12_TEXTURE_COPY_TYPE_D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX; // island enum
pub const D3D12_TEXTURE_LAYOUT_ROW_MAJOR: D3D12_TEXTURE_LAYOUT = D3D12_TEXTURE_LAYOUT_D3D12_TEXTURE_LAYOUT_ROW_MAJOR; // island enum
pub const D3DFMT_A16B16G16R16: _D3DFORMAT = _D3DFORMAT_D3DFMT_A16B16G16R16; // island enum
pub const D3DFMT_A16B16G16R16F: _D3DFORMAT = _D3DFORMAT_D3DFMT_A16B16G16R16F; // island enum
pub const D3DFMT_A1: _D3DFORMAT = _D3DFORMAT_D3DFMT_A1; // island enum
pub const D3DFMT_A1R5G5B5: _D3DFORMAT = _D3DFORMAT_D3DFMT_A1R5G5B5; // island enum
pub const D3DFMT_A2B10G10R10: _D3DFORMAT = _D3DFORMAT_D3DFMT_A2B10G10R10; // island enum
pub const D3DFMT_A2R10G10B10: _D3DFORMAT = _D3DFORMAT_D3DFMT_A2R10G10B10; // island enum
pub const D3DFMT_A32B32G32R32F: _D3DFORMAT = _D3DFORMAT_D3DFMT_A32B32G32R32F; // island enum
pub const D3DFMT_A4R4G4B4: _D3DFORMAT = _D3DFORMAT_D3DFMT_A4R4G4B4; // island enum
pub const D3DFMT_A8: _D3DFORMAT = _D3DFORMAT_D3DFMT_A8; // island enum
pub const D3DFMT_A8B8G8R8: _D3DFORMAT = _D3DFORMAT_D3DFMT_A8B8G8R8; // island enum
pub const D3DFMT_A8P8: _D3DFORMAT = _D3DFORMAT_D3DFMT_A8P8; // island enum
pub const D3DFMT_A8R8G8B8: _D3DFORMAT = _D3DFORMAT_D3DFMT_A8R8G8B8; // island enum
pub const D3DFMT_D16: _D3DFORMAT = _D3DFORMAT_D3DFMT_D16; // island enum
pub const D3DFMT_D16_LOCKABLE: _D3DFORMAT = _D3DFORMAT_D3DFMT_D16_LOCKABLE; // island enum
pub const D3DFMT_D24S8: _D3DFORMAT = _D3DFORMAT_D3DFMT_D24S8; // island enum
pub const D3DFMT_D32F_LOCKABLE: _D3DFORMAT = _D3DFORMAT_D3DFMT_D32F_LOCKABLE; // island enum
pub const D3DFMT_DXT1: _D3DFORMAT = _D3DFORMAT_D3DFMT_DXT1; // island enum
pub const D3DFMT_DXT2: _D3DFORMAT = _D3DFORMAT_D3DFMT_DXT2; // island enum
pub const D3DFMT_DXT4: _D3DFORMAT = _D3DFORMAT_D3DFMT_DXT4; // island enum
pub const D3DFMT_G16R16: _D3DFORMAT = _D3DFORMAT_D3DFMT_G16R16; // island enum
pub const D3DFMT_G16R16F: _D3DFORMAT = _D3DFORMAT_D3DFMT_G16R16F; // island enum
pub const D3DFMT_G32R32F: _D3DFORMAT = _D3DFORMAT_D3DFMT_G32R32F; // island enum
pub const D3DFMT_L16: _D3DFORMAT = _D3DFORMAT_D3DFMT_L16; // island enum
pub const D3DFMT_L8: _D3DFORMAT = _D3DFORMAT_D3DFMT_L8; // island enum
pub const D3DFMT_P8: _D3DFORMAT = _D3DFORMAT_D3DFMT_P8; // island enum
pub const D3DFMT_Q16W16V16U16: _D3DFORMAT = _D3DFORMAT_D3DFMT_Q16W16V16U16; // island enum
pub const D3DFMT_Q8W8V8U8: _D3DFORMAT = _D3DFORMAT_D3DFMT_Q8W8V8U8; // island enum
pub const D3DFMT_R16F: _D3DFORMAT = _D3DFORMAT_D3DFMT_R16F; // island enum
pub const D3DFMT_R32F: _D3DFORMAT = _D3DFORMAT_D3DFMT_R32F; // island enum
pub const D3DFMT_R5G6B5: _D3DFORMAT = _D3DFORMAT_D3DFMT_R5G6B5; // island enum
pub const D3DFMT_R8G8B8: _D3DFORMAT = _D3DFORMAT_D3DFMT_R8G8B8; // island enum
pub const D3DFMT_V16U16: _D3DFORMAT = _D3DFORMAT_D3DFMT_V16U16; // island enum
pub const D3DFMT_V8U8: _D3DFORMAT = _D3DFORMAT_D3DFMT_V8U8; // island enum
pub const D3DFMT_X1R5G5B5: _D3DFORMAT = _D3DFORMAT_D3DFMT_X1R5G5B5; // island enum
pub const D3DFMT_X8R8G8B8: _D3DFORMAT = _D3DFORMAT_D3DFMT_X8R8G8B8; // island enum
pub const D3DFMT_YUY2: _D3DFORMAT = _D3DFORMAT_D3DFMT_YUY2; // island enum
pub const D3DPOOL_DEFAULT: _D3DPOOL = _D3DPOOL_D3DPOOL_DEFAULT; // island enum
pub const DXGI_FORMAT_420_OPAQUE: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_420_OPAQUE; // island enum
pub const DXGI_FORMAT_A8P8: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_A8P8; // island enum
pub const DXGI_FORMAT_A8_UNORM: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_A8_UNORM; // island enum
pub const DXGI_FORMAT_AI44: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_AI44; // island enum
pub const DXGI_FORMAT_AYUV: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_AYUV; // island enum
pub const DXGI_FORMAT_B8G8R8A8_UNORM: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_B8G8R8A8_UNORM; // island enum
pub const DXGI_FORMAT_B8G8R8A8_UNORM_SRGB: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_B8G8R8A8_UNORM_SRGB; // island enum
pub const DXGI_FORMAT_B8G8R8X8_UNORM: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_B8G8R8X8_UNORM; // island enum
pub const DXGI_FORMAT_B8G8R8X8_UNORM_SRGB: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_B8G8R8X8_UNORM_SRGB; // island enum
pub const DXGI_FORMAT_BC1_UNORM: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_BC1_UNORM; // island enum
pub const DXGI_FORMAT_BC1_UNORM_SRGB: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_BC1_UNORM_SRGB; // island enum
pub const DXGI_FORMAT_BC2_UNORM: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_BC2_UNORM; // island enum
pub const DXGI_FORMAT_BC2_UNORM_SRGB: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_BC2_UNORM_SRGB; // island enum
pub const DXGI_FORMAT_BC3_UNORM: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_BC3_UNORM; // island enum
pub const DXGI_FORMAT_BC3_UNORM_SRGB: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_BC3_UNORM_SRGB; // island enum
pub const DXGI_FORMAT_D16_UNORM: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_D16_UNORM; // island enum
pub const DXGI_FORMAT_D24_UNORM_S8_UINT: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_D24_UNORM_S8_UINT; // island enum
pub const DXGI_FORMAT_D32_FLOAT: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_D32_FLOAT; // island enum
pub const DXGI_FORMAT_IA44: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_IA44; // island enum
pub const DXGI_FORMAT_NV11: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_NV11; // island enum
pub const DXGI_FORMAT_NV12: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_NV12; // island enum
pub const DXGI_FORMAT_P010: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_P010; // island enum
pub const DXGI_FORMAT_P016: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_P016; // island enum
pub const DXGI_FORMAT_P8: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_P8; // island enum
pub const DXGI_FORMAT_R10G10B10A2_UNORM: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_R10G10B10A2_UNORM; // island enum
pub const DXGI_FORMAT_R16G16B16A16_FLOAT: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_R16G16B16A16_FLOAT; // island enum
pub const DXGI_FORMAT_R16G16B16A16_SNORM: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_R16G16B16A16_SNORM; // island enum
pub const DXGI_FORMAT_R16G16B16A16_UNORM: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_R16G16B16A16_UNORM; // island enum
pub const DXGI_FORMAT_R16G16_FLOAT: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_R16G16_FLOAT; // island enum
pub const DXGI_FORMAT_R16G16_SNORM: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_R16G16_SNORM; // island enum
pub const DXGI_FORMAT_R16G16_UNORM: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_R16G16_UNORM; // island enum
pub const DXGI_FORMAT_R16_FLOAT: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_R16_FLOAT; // island enum
pub const DXGI_FORMAT_R16_UNORM: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_R16_UNORM; // island enum
pub const DXGI_FORMAT_R32G32B32A32_FLOAT: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_R32G32B32A32_FLOAT; // island enum
pub const DXGI_FORMAT_R32G32_FLOAT: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_R32G32_FLOAT; // island enum
pub const DXGI_FORMAT_R32_FLOAT: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_R32_FLOAT; // island enum
pub const DXGI_FORMAT_R8G8B8A8_SNORM: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_R8G8B8A8_SNORM; // island enum
pub const DXGI_FORMAT_R8G8B8A8_UNORM: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_R8G8B8A8_UNORM; // island enum
pub const DXGI_FORMAT_R8G8B8A8_UNORM_SRGB: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_R8G8B8A8_UNORM_SRGB; // island enum
pub const DXGI_FORMAT_R8G8_SNORM: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_R8G8_SNORM; // island enum
pub const DXGI_FORMAT_R8_UINT: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_R8_UINT; // island enum
pub const DXGI_FORMAT_R8_UNORM: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_R8_UNORM; // island enum
pub const DXGI_FORMAT_UNKNOWN: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_UNKNOWN; // island enum
pub const DXGI_FORMAT_Y210: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_Y210; // island enum
pub const DXGI_FORMAT_Y216: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_Y216; // island enum
pub const DXGI_FORMAT_Y410: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_Y410; // island enum
pub const DXGI_FORMAT_Y416: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_Y416; // island enum
pub const DXGI_FORMAT_YUY2: DXGI_FORMAT = DXGI_FORMAT_DXGI_FORMAT_YUY2; // island enum
pub const DXVA2_SampleFieldInterleavedEvenFirst: _DXVA2_SampleFormat = _DXVA2_SampleFormat_DXVA2_SampleFieldInterleavedEvenFirst; // island enum
pub const MF2DBuffer_LockFlags_Read: _MF2DBuffer_LockFlags = _MF2DBuffer_LockFlags_MF2DBuffer_LockFlags_Read; // island enum
pub const MF2DBuffer_LockFlags_ReadWrite: _MF2DBuffer_LockFlags = _MF2DBuffer_LockFlags_MF2DBuffer_LockFlags_ReadWrite; // island enum
pub const MF2DBuffer_LockFlags_Write: _MF2DBuffer_LockFlags = _MF2DBuffer_LockFlags_MF2DBuffer_LockFlags_Write; // island enum
pub const MFCLOCK_CHARACTERISTICS_FLAG_ALWAYS_RUNNING: _MFCLOCK_CHARACTERISTICS_FLAGS = _MFCLOCK_CHARACTERISTICS_FLAGS_MFCLOCK_CHARACTERISTICS_FLAG_ALWAYS_RUNNING; // island enum
pub const MFCLOCK_CHARACTERISTICS_FLAG_FREQUENCY_10MHZ: _MFCLOCK_CHARACTERISTICS_FLAGS = _MFCLOCK_CHARACTERISTICS_FLAGS_MFCLOCK_CHARACTERISTICS_FLAG_FREQUENCY_10MHZ; // island enum
pub const MFCLOCK_CHARACTERISTICS_FLAG_IS_SYSTEM_CLOCK: _MFCLOCK_CHARACTERISTICS_FLAGS = _MFCLOCK_CHARACTERISTICS_FLAGS_MFCLOCK_CHARACTERISTICS_FLAG_IS_SYSTEM_CLOCK; // island enum
pub const MFCLOCK_STATE_INVALID: _MFCLOCK_STATE = _MFCLOCK_STATE_MFCLOCK_STATE_INVALID; // island enum
pub const MFCLOCK_STATE_PAUSED: _MFCLOCK_STATE = _MFCLOCK_STATE_MFCLOCK_STATE_PAUSED; // island enum
pub const MFCLOCK_STATE_RUNNING: _MFCLOCK_STATE = _MFCLOCK_STATE_MFCLOCK_STATE_RUNNING; // island enum
pub const MFCLOCK_STATE_STOPPED: _MFCLOCK_STATE = _MFCLOCK_STATE_MFCLOCK_STATE_STOPPED; // island enum
pub const MFNominalRange_Normal: _MFNominalRange = _MFNominalRange_MFNominalRange_Normal; // island enum
pub const MFT_ENUM_FLAG_ASYNCMFT: _MFT_ENUM_FLAG = _MFT_ENUM_FLAG_MFT_ENUM_FLAG_ASYNCMFT; // island enum
pub const MFT_ENUM_FLAG_FIELDOFUSE: _MFT_ENUM_FLAG = _MFT_ENUM_FLAG_MFT_ENUM_FLAG_FIELDOFUSE; // island enum
pub const MFT_ENUM_FLAG_HARDWARE: _MFT_ENUM_FLAG = _MFT_ENUM_FLAG_MFT_ENUM_FLAG_HARDWARE; // island enum
pub const MFT_ENUM_FLAG_LOCALMFT: _MFT_ENUM_FLAG = _MFT_ENUM_FLAG_MFT_ENUM_FLAG_LOCALMFT; // island enum
pub const MFT_ENUM_FLAG_SORTANDFILTER: _MFT_ENUM_FLAG = _MFT_ENUM_FLAG_MFT_ENUM_FLAG_SORTANDFILTER; // island enum
pub const MFT_ENUM_FLAG_SYNCMFT: _MFT_ENUM_FLAG = _MFT_ENUM_FLAG_MFT_ENUM_FLAG_SYNCMFT; // island enum
pub const MFT_ENUM_FLAG_TRANSCODE_ONLY: _MFT_ENUM_FLAG = _MFT_ENUM_FLAG_MFT_ENUM_FLAG_TRANSCODE_ONLY; // island enum
pub const MFVideoFlag_AnalogProtected: _MFVideoFlags = _MFVideoFlags_MFVideoFlag_AnalogProtected; // island enum
pub const MFVideoFlag_BottomUpLinearRep: _MFVideoFlags = _MFVideoFlags_MFVideoFlag_BottomUpLinearRep; // island enum
pub const MFVideoFlag_DigitallyProtected: _MFVideoFlags = _MFVideoFlags_MFVideoFlag_DigitallyProtected; // island enum
pub const MFVideoFlag_PAD_TO_Mask: _MFVideoFlags = _MFVideoFlags_MFVideoFlag_PAD_TO_Mask; // island enum
pub const MFVideoFlag_PanScanEnabled: _MFVideoFlags = _MFVideoFlags_MFVideoFlag_PanScanEnabled; // island enum
pub const MFVideoFlag_SrcContentHintMask: _MFVideoFlags = _MFVideoFlags_MFVideoFlag_SrcContentHintMask; // island enum
pub const MFVideoInterlace_MixedInterlaceOrProgressive: _MFVideoInterlaceMode = _MFVideoInterlaceMode_MFVideoInterlace_MixedInterlaceOrProgressive; // island enum
pub const MFVideoInterlace_Progressive: _MFVideoInterlaceMode = _MFVideoInterlaceMode_MFVideoInterlace_Progressive; // island enum
pub const MFVideoInterlace_Unknown: _MFVideoInterlaceMode = _MFVideoInterlaceMode_MFVideoInterlace_Unknown; // island enum
pub const MFVideoLighting_office: _MFVideoLighting = _MFVideoLighting_MFVideoLighting_office; // island enum
pub const MFVideoPrimaries_BT709: _MFVideoPrimaries = _MFVideoPrimaries_MFVideoPrimaries_BT709; // island enum
pub const MFVideoTransFunc_10: _MFVideoTransferFunction = _MFVideoTransferFunction_MFVideoTransFunc_10; // island enum
pub const MFVideoTransFunc_sRGB: _MFVideoTransferFunction = _MFVideoTransferFunction_MFVideoTransFunc_sRGB; // island enum
pub const MFWaveFormatExConvertFlag_ForceExtensible: _MFWaveFormatExConvertFlags = _MFWaveFormatExConvertFlags_MFWaveFormatExConvertFlag_ForceExtensible; // island enum
pub const MFWaveFormatExConvertFlag_Normal: _MFWaveFormatExConvertFlags = _MFWaveFormatExConvertFlags_MFWaveFormatExConvertFlag_Normal; // island enum
pub const MF_ACCESSMODE_READ: MF_FILE_ACCESSMODE = MF_FILE_ACCESSMODE_MF_ACCESSMODE_READ; // island enum
pub const MF_ACCESSMODE_READWRITE: MF_FILE_ACCESSMODE = MF_FILE_ACCESSMODE_MF_ACCESSMODE_READWRITE; // island enum
pub const MF_ACCESSMODE_WRITE: MF_FILE_ACCESSMODE = MF_FILE_ACCESSMODE_MF_ACCESSMODE_WRITE; // island enum
pub const MF_ATTRIBUTES_MATCH_ALL_ITEMS: _MF_ATTRIBUTES_MATCH_TYPE = _MF_ATTRIBUTES_MATCH_TYPE_MF_ATTRIBUTES_MATCH_ALL_ITEMS; // island enum
pub const MF_ATTRIBUTES_MATCH_INTERSECTION: _MF_ATTRIBUTES_MATCH_TYPE = _MF_ATTRIBUTES_MATCH_TYPE_MF_ATTRIBUTES_MATCH_INTERSECTION; // island enum
pub const MF_ATTRIBUTES_MATCH_OUR_ITEMS: _MF_ATTRIBUTES_MATCH_TYPE = _MF_ATTRIBUTES_MATCH_TYPE_MF_ATTRIBUTES_MATCH_OUR_ITEMS; // island enum
pub const MF_ATTRIBUTES_MATCH_SMALLER: _MF_ATTRIBUTES_MATCH_TYPE = _MF_ATTRIBUTES_MATCH_TYPE_MF_ATTRIBUTES_MATCH_SMALLER; // island enum
pub const MF_ATTRIBUTES_MATCH_THEIR_ITEMS: _MF_ATTRIBUTES_MATCH_TYPE = _MF_ATTRIBUTES_MATCH_TYPE_MF_ATTRIBUTES_MATCH_THEIR_ITEMS; // island enum
pub const MF_ATTRIBUTE_BLOB: _MF_ATTRIBUTE_TYPE = _MF_ATTRIBUTE_TYPE_MF_ATTRIBUTE_BLOB; // island enum
pub const MF_ATTRIBUTE_DOUBLE: _MF_ATTRIBUTE_TYPE = _MF_ATTRIBUTE_TYPE_MF_ATTRIBUTE_DOUBLE; // island enum
pub const MF_ATTRIBUTE_GUID: _MF_ATTRIBUTE_TYPE = _MF_ATTRIBUTE_TYPE_MF_ATTRIBUTE_GUID; // island enum
pub const MF_ATTRIBUTE_IUNKNOWN: _MF_ATTRIBUTE_TYPE = _MF_ATTRIBUTE_TYPE_MF_ATTRIBUTE_IUNKNOWN; // island enum
pub const MF_ATTRIBUTE_STRING: _MF_ATTRIBUTE_TYPE = _MF_ATTRIBUTE_TYPE_MF_ATTRIBUTE_STRING; // island enum
pub const MF_ATTRIBUTE_UINT32: _MF_ATTRIBUTE_TYPE = _MF_ATTRIBUTE_TYPE_MF_ATTRIBUTE_UINT32; // island enum
pub const MF_ATTRIBUTE_UINT64: _MF_ATTRIBUTE_TYPE = _MF_ATTRIBUTE_TYPE_MF_ATTRIBUTE_UINT64; // island enum
pub const MF_Plugin_Type_MFT: _MF_Plugin_Type = _MF_Plugin_Type_MF_Plugin_Type_MFT; // island enum
pub const PVCF_DEFAULT: tagPROPVAR_COMPARE_FLAGS = tagPROPVAR_COMPARE_FLAGS_PVCF_DEFAULT; // island enum
pub const PVCU_DEFAULT: tagPROPVAR_COMPARE_UNIT = tagPROPVAR_COMPARE_UNIT_PVCU_DEFAULT; // island enum
pub const STATFLAG_NONAME: DWORD = 1; // ole32-rust/src/compobj/com.rs
pub const STGC_DEFAULT: DWORD = 0x0; // ole32-rust/src/win_consts.rs
pub const STGM_READWRITE: u32 = 2; // comctl32-rust/src/types_amd64.rs
pub const STGM_WRITE: u32 = 1; // comctl32-rust/src/types_amd64.rs
pub const STREAM_SEEK_SET: DWORD = 0; // comctl32-rust/src/prelude.rs
pub const STRSAFE_E_INSUFFICIENT_BUFFER: HRESULT = 0x8007007Au32 as HRESULT; // shell32-rust/src/cfs_folder.rs
pub const VT_CLSID: u16 = 72; // oleaut32-rust/src/consts.rs
pub const VT_EMPTY: VARENUM = VARENUM_VT_EMPTY; // island enum
pub const VT_I8: VARENUM = VARENUM_VT_I8; // island enum
pub const VT_LPWSTR: u16 = 31; // oleaut32-rust/src/consts.rs
pub const VT_NULL: VARENUM = VARENUM_VT_NULL; // island enum
pub const VT_R4: VARENUM = VARENUM_VT_R4; // island enum
pub const VT_R8: i32 = 5; // msvcrt-crt-rust/src/printf.rs
pub const VT_UI1: VARTYPE = 17; // oleaut32-rust/src/olesys.rs
pub const VT_UI4: DWORD = 19; // mfreadwrite-rust/src/prelude.rs
pub const VT_UI8: DWORD = 0x15; // ole32-rust/src/win_consts.rs
pub const VT_UNKNOWN: u16 = 13; // mfreadwrite-rust/src/prelude.rs
pub const VT_VECTOR: u16 = 4096; // oleaut32-rust/src/consts.rs
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0; // comctl32-rust/src/prelude.rs
pub const msoBegin: _MFBYTESTREAM_SEEK_ORIGIN = _MFBYTESTREAM_SEEK_ORIGIN_msoBegin; // island enum
pub const msoCurrent: _MFBYTESTREAM_SEEK_ORIGIN = _MFBYTESTREAM_SEEK_ORIGIN_msoCurrent; // island enum
extern "C" {
    pub fn abs(x: i32) -> i32; // charmap-rust/src/prelude.rs
    pub fn bsearch( key: *const c_void, base: *const c_void, nmemb: usize, size: usize, compar: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> i32>, ) -> *mut c_void; // evr-rust/src/prelude.rs
    pub fn calloc(arg1: ::core::ffi::c_uint, arg2: ::core::ffi::c_uint) -> *mut ::core::ffi::c_void; // msvcrt-sys/src/bindgen_i386.rs
    pub fn free(p: *mut c_void); // atlclasses-rust/src/lib.rs
    pub fn isalpha(c: i32) -> i32; // cmd-rust/src/prelude.rs
    pub fn malloc(size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
    pub fn modf(x: f64, iptr: *mut f64) -> f64; // calc-rust/src/prelude.rs
    pub fn qsort(base: *mut c_void, num: usize, width: usize, comp: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>); // cmd-rust/src/prelude.rs
    pub fn realloc(p: *mut c_void, size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
    pub fn ros_dbg_log( cls: ::core::ffi::c_uint, ch: *mut __wine_debug_channel, file: *const ::core::ffi::c_char, func: *const ::core::ffi::c_char, line: ::core::ffi::c_int, format: *const ::core::ffi::c_char, ... ) -> ::core::ffi::c_int; // winegdi-rust/src/types_i386.rs
    // The COUNTED swprintf is a header inline over _vsnwprintf: msvcrt.dll's own `swprintf`
    // export is the legacy (buffer, format, ...) form, so the count would be read as the
    // format. `scan_swprintf_count` gates it.
    #[link_name = "_snwprintf"]
    pub fn swprintf(s: *mut u16, n: usize, fmt: *const u16, ...) -> c_int; // msiexec-rust/src/prelude.rs
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wine_dbg_sprintf(format: *const ::core::ffi::c_char, ...) -> *const ::core::ffi::c_char; // winegdi-rust/src/types_i386.rs
    pub fn wine_dbgstr_an( s: *const ::core::ffi::c_char, n: ::core::ffi::c_int, ) -> *const ::core::ffi::c_char; // winegdi-rust/src/types_i386.rs
}
extern "system" {
    pub fn CLSIDFromString(arg1: LPCOLESTR, arg2: LPCLSID) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CoCreateInstance( rclsid: *const CLSID, pUnkOuter: LPUNKNOWN, dwClsContext: DWORD, iid: *const IID, ppv: *mut LPVOID, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemAlloc(size: SIZE_T) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemFree(ptr: LPVOID); // aclui-rust/src/types_i386.rs
    pub fn CreateEventA( lpEventAttributes: LPSECURITY_ATTRIBUTES, bManualReset: BOOL, bInitialState: BOOL, lpName: LPCSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateEventW( arg1: LPSECURITY_ATTRIBUTES, bManualReset: BOOL, bInitialState: BOOL, lpName: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn DeleteCriticalSection(arg1: PCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn GetCurrentThreadId() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetFileSizeEx(arg1: HANDLE, arg2: PLARGE_INTEGER) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetFileTime(arg1: HANDLE, arg2: LPFILETIME, arg3: LPFILETIME, arg4: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetTempPathW(arg1: DWORD, arg2: LPWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetTickCount() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn InitOnceExecuteOnce( InitOnce: PINIT_ONCE, InitFn: PINIT_ONCE_FN, Parameter: PVOID, Context: *mut LPVOID, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn InitializeConditionVariable(ConditionVariable: PRTL_CONDITION_VARIABLE); // kernel32-vista-rust/src/sync.rs
    pub fn InitializeCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn PathCchAppend(path1: *mut u16, size: usize, path2: PCWSTR) -> HRESULT; // kernel32-rust/src/client/pathcch.rs
    pub fn PathCreateFromUrlW( pszUrl: LPCWSTR, pszPath: LPWSTR, pcchPath: LPDWORD, dwFlags: DWORD, ) -> HRESULT; // certmgr-rust/src/types_i386.rs
    pub fn PropVariantClear(pvar: *mut PROPVARIANT) -> HRESULT; // mfreadwrite-rust/src/prelude.rs
    pub fn PropVariantCopy(pvarDest: *mut PROPVARIANT, pvarSrc: *const PROPVARIANT,) -> HRESULT; // mfreadwrite-rust/src/prelude.rs
    pub fn QueryPerformanceCounter(arg1: PLARGE_INTEGER) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn QueryPerformanceFrequency(arg1: PLARGE_INTEGER) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ReadFile( arg1: HANDLE, arg2: PVOID, arg3: DWORD, arg4: PDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegCreateKeyW(hKey: HKEY, lpSubKey: LPCWSTR, phkResult: PHKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegDeleteKeyW(hKey: HKEY, lpSubKey: LPCWSTR) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegEnumKeyExW( hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, lpcchName: LPDWORD, lpReserved: LPDWORD, lpClass: LPWSTR, lpcchClass: LPDWORD, lpftLastWriteTime: PFILETIME, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegEnumValueW( hKey: HKEY, dwIndex: DWORD, lpValueName: LPWSTR, lpcchValueName: LPDWORD, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegOpenKeyW(arg1: HKEY, arg2: LPCWSTR, arg3: PHKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegSetValueExW( hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn SetEvent(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetFilePointerEx( arg1: HANDLE, arg2: LARGE_INTEGER, arg3: PLARGE_INTEGER, arg4: DWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SleepConditionVariableCS(ConditionVariable: PRTL_CONDITION_VARIABLE, CriticalSection: PRTL_CRITICAL_SECTION, Timeout: DWORD,) -> BOOL; // kernel32-vista-rust/src/sync.rs
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn WakeAllConditionVariable(ConditionVariable: PRTL_CONDITION_VARIABLE); // kernel32-vista-rust/src/sync.rs
    pub fn WriteFile( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn lstrcmpiW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn lstrlenW(arg1: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};
pub use winelist_rust::{list_add_after, list_add_before, list_add_head, list_add_tail, list_count, list_empty, list_head, list_init, list_move_head, list_move_tail, list_next, list_prev, list_remove, list_tail};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const I64_MAX: i64 = 0x7FFFFFFFFFFFFFFF; // clang, main.c
pub const MF_FILEFLAGS_NOBUFFERING: i32 = 1; // clang, main.c
pub const MF_OPENMODE_APPEND_IF_EXIST: i32 = 3; // clang, main.c
pub const MF_OPENMODE_DELETE_IF_EXIST: i32 = 4; // clang, main.c
pub const MF_OPENMODE_FAIL_IF_EXIST: i32 = 1; // clang, main.c
pub const MF_OPENMODE_FAIL_IF_NOT_EXIST: i32 = 0; // clang, main.c
pub const MF_OPENMODE_RESET_IF_EXIST: i32 = 2; // clang, main.c
pub const MF_VERSION_WIN7: LONG = 0x20070; // clang, main.c
pub const MF_VERSION_XP: LONG = 0x10070; // clang, main.c
pub const RTWQ_STANDARD_WORKQUEUE: i32 = 0; // clang, queue.c
pub const UI64_MAX: u64 = 0xFFFFFFFFFFFFFFFF; // clang, main.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// CONDITION_VARIABLE: c2rust, main.c
pub type CONDITION_VARIABLE = RTL_CONDITION_VARIABLE;
// async_stream_op_type: c2rust, main.c
pub type async_stream_op_type = ::core::ffi::c_uint;
// clock_command: c2rust, main.c
pub type clock_command = ::core::ffi::c_uint;
// comparand: c2rust, mediatype.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct comparand {
    pub type_: *mut IMFAttributes,
    pub value: PROPVARIANT,
    pub count: UINT32,
    pub guid: GUID,
    pub hr: HRESULT,
}
// resolved_object_origin: c2rust, main.c
pub type resolved_object_origin = ::core::ffi::c_uint;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _aligned_free(_Memory: *mut ::core::ffi::c_void); // buffer.c
    pub fn _aligned_malloc(_Size: size_t, _Alignment: size_t) -> *mut ::core::ffi::c_void; // buffer.c
    pub fn towlower(_C: wint_t) -> wint_t; // main.c
    pub fn wcsrchr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut wchar_t; // main.c
}
extern "system" {
    pub fn BCryptGenRandom(_: BCRYPT_ALG_HANDLE, _: PUCHAR, _: ULONG, _: ULONG) -> NTSTATUS; // main.c
    pub fn PropVariantCompareEx(
        propvar1: *const PROPVARIANT,
        propvar2: *const PROPVARIANT,
        uint: PROPVAR_COMPARE_UNIT,
        flags: PROPVAR_COMPARE_FLAGS,
    ) -> INT; // main.c
    pub fn PropVariantToBuffer(
        propvarIn: *const PROPVARIANT,
        ret: *mut ::core::ffi::c_void,
        cb: UINT,
    ) -> HRESULT; // main.c
    pub fn RtwqAllocateWorkQueue(queue_type: RTWQ_WORKQUEUE_TYPE, queue: *mut DWORD) -> HRESULT; // queue.c
    pub fn RtwqBeginRegisterWorkQueueWithMMCSS(
        queue: DWORD,
        mmcss_class: *const WCHAR,
        taskid: DWORD,
        priority: LONG,
        callback: *mut IRtwqAsyncCallback,
        state: *mut IUnknown,
    ) -> HRESULT; // queue.c
    pub fn RtwqCancelWorkItem(key: RTWQWORKITEM_KEY) -> HRESULT; // main.c
    pub fn RtwqCreateAsyncResult(
        object: *mut IUnknown,
        callback: *mut IRtwqAsyncCallback,
        state: *mut IUnknown,
        result: *mut *mut IRtwqAsyncResult,
    ) -> HRESULT; // main.c
    pub fn RtwqInvokeCallback(result: *mut IRtwqAsyncResult) -> HRESULT; // main.c
    pub fn RtwqPutWaitingWorkItem(
        event: HANDLE,
        priority: LONG,
        result: *mut IRtwqAsyncResult,
        key: *mut RTWQWORKITEM_KEY,
    ) -> HRESULT; // main.c
    pub fn RtwqPutWorkItem(queue: DWORD, priority: LONG, result: *mut IRtwqAsyncResult) -> HRESULT; // main.c
    pub fn RtwqScheduleWorkItem(
        result: *mut IRtwqAsyncResult,
        timeout: INT64,
        key: *mut RTWQWORKITEM_KEY,
    ) -> HRESULT; // queue.c
    pub fn RtwqShutdown() -> HRESULT; // main.c
    pub fn RtwqStartup() -> HRESULT; // main.c
}
pub use win32macros_rust::WINE_ERR as ERR; // sdk alias
pub use win32macros_rust::WINE_FIXME as FIXME; // sdk alias
pub use win32macros_rust::WINE_TRACE as TRACE; // sdk alias
pub use win32macros_rust::WINE_WARN as WARN; // sdk alias
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_mfplat: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x6d\x66\x70\x6c\x61\x74\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_mfplat };
// ==== end wine debug channels ====

// ==== port resolve: INITGUID storage (generated) ====
#[no_mangle]
pub static IID_ID3D11Texture2D: GUID = GUID { Data1: 0x6f15aaf2, Data2: 0xd208, Data3: 0x4e89, Data4: [0x9a, 0xb4, 0x48, 0x95, 0x35, 0xd3, 0x4f, 0x9c] };
#[no_mangle]
pub static IID_ID3D12CommandAllocator: GUID = GUID { Data1: 0x6102dee4, Data2: 0xaf59, Data3: 0x4b09, Data4: [0xb9, 0x99, 0xb4, 0x4d, 0x73, 0xf0, 0x9b, 0x24] };
#[no_mangle]
pub static IID_ID3D12CommandQueue: GUID = GUID { Data1: 0x0ec870a6, Data2: 0x5d7e, Data3: 0x4c22, Data4: [0x8c, 0xfc, 0x5b, 0xaa, 0xe0, 0x76, 0x16, 0xed] };
#[no_mangle]
pub static IID_ID3D12GraphicsCommandList: GUID = GUID { Data1: 0x5b160d0f, Data2: 0xac1b, Data3: 0x4185, Data4: [0x8b, 0xa8, 0xb3, 0xae, 0x42, 0xa5, 0xa4, 0x55] };
#[no_mangle]
pub static IID_ID3D12Resource: GUID = GUID { Data1: 0x696442be, Data2: 0xa72e, Data3: 0x4059, Data4: [0xbc, 0x79, 0x5b, 0x5c, 0x98, 0x04, 0x0f, 0xad] };
// ==== end INITGUID storage ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::mfplat::main::IID_ID3D12Device;
pub use crate::mfplat::main::IID_ID3D12Fence;
pub use crate::mfplat::main::MFCopyImage;
pub use crate::mfplat::main::MFCreateD3D12SynchronizationObject;
pub use crate::mfplat::main::MFGetAttributesAsBlob;
pub use crate::mfplat::main::MFGetAttributesAsBlobSize;
pub use crate::mfplat::mediatype::MFGetPlaneSize;
pub use crate::mfplat::main::MFInitAttributesFromBlob;
pub use crate::mfplat::mediatype::MFMapDXGIFormatToDX9Format;
pub use crate::mfplat::main::attributes_Compare;
pub use crate::mfplat::main::attributes_CompareItem;
pub use crate::mfplat::main::attributes_CopyAllItems;
pub use crate::mfplat::main::attributes_DeleteAllItems;
pub use crate::mfplat::main::attributes_DeleteItem;
pub use crate::mfplat::main::attributes_GetAllocatedBlob;
pub use crate::mfplat::main::attributes_GetAllocatedString;
pub use crate::mfplat::main::attributes_GetBlob;
pub use crate::mfplat::main::attributes_GetBlobSize;
pub use crate::mfplat::main::attributes_GetCount;
pub use crate::mfplat::main::attributes_GetDouble;
pub use crate::mfplat::main::attributes_GetGUID;
pub use crate::mfplat::main::attributes_GetItem;
pub use crate::mfplat::main::attributes_GetItemByIndex;
pub use crate::mfplat::main::attributes_GetItemType;
pub use crate::mfplat::main::attributes_GetString;
pub use crate::mfplat::main::attributes_GetStringLength;
pub use crate::mfplat::main::attributes_GetUINT32;
pub use crate::mfplat::main::attributes_GetUINT64;
pub use crate::mfplat::main::attributes_GetUnknown;
pub use crate::mfplat::main::attributes_LockStore;
pub use crate::mfplat::main::attributes_SetBlob;
pub use crate::mfplat::main::attributes_SetDouble;
pub use crate::mfplat::main::attributes_SetGUID;
pub use crate::mfplat::main::attributes_SetItem;
pub use crate::mfplat::main::attributes_SetString;
pub use crate::mfplat::main::attributes_SetUINT32;
pub use crate::mfplat::main::attributes_SetUINT64;
pub use crate::mfplat::main::attributes_SetUnknown;
pub use crate::mfplat::main::attributes_UnlockStore;
pub use crate::mfplat::main::clear_attributes_object;
pub use crate::mfplat::main::debugstr_attr;
pub use crate::mfplat::main::debugstr_mf_guid;
pub use crate::mfplat::main::init_attributes_object;
pub use crate::mfplat::mediatype::mf_format_get_stride;
// ==== end cross-TU definitions ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! IsEqualCLSID { ($p0:expr, $p1:expr $(,)?) => { (((memcmp((($p0) as _), (($p1) as _), ((core::mem::size_of::<GUID>()) as _))) as isize == 0) as c_int) }; }
#[macro_export] macro_rules! IsEqualIID { ($p0:expr, $p1:expr $(,)?) => { (((memcmp((($p0) as _), (($p1) as _), ((core::mem::size_of::<GUID>()) as _))) as isize == 0) as c_int) }; }
#[macro_export] macro_rules! MAKEFOURCC { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { ((((((($p0) as BYTE)) as DWORD) | ((((($p1) as BYTE)) as DWORD) << 8)) | ((((($p2) as BYTE)) as DWORD) << 16)) | ((((($p3) as BYTE)) as DWORD) << 24)) }; }
#[macro_export] macro_rules! PropVariantInit { ($p0:expr $(,)?) => { memset((($p0) as _), ((0) as _), ((core::mem::size_of::<PROPVARIANT>()) as _)) }; }
#[macro_export] macro_rules! max { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 > $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
pub const E_FAIL: HRESULT = ((0x80004005u32) as HRESULT);
pub const E_HANDLE: HRESULT = ((0x80070006u32) as HRESULT);
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT);
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT);
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT);
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT);
pub const E_POINTER: HRESULT = ((0x80004003u32) as HRESULT);
pub const E_UNEXPECTED: HRESULT = ((0x8000FFFFu32) as HRESULT);
pub const HKEY_CLASSES_ROOT: HKEY = ((((((0x80000000u32) as LONG)) as LONG_PTR)) as HKEY);
pub const HKEY_CURRENT_USER: HKEY = ((((((0x80000001u32) as LONG)) as LONG_PTR)) as HKEY);
pub const HKEY_LOCAL_MACHINE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY);
#[inline]
pub unsafe fn IClassFactory_AddRef(This: *mut IClassFactory) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IClassFactory_CreateInstance(This: *mut IClassFactory, pUnkOuter: *mut IUnknown, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).CreateInstance.unwrap_unchecked())(This, pUnkOuter, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IClassFactory_Release(This: *mut IClassFactory) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn ID3D11DeviceContext_CopySubresourceRegion(This: *mut ID3D11DeviceContext, pDstResource: *mut ID3D11Resource, DstSubresource: UINT, DstX: UINT, DstY: UINT, DstZ: UINT, pSrcResource: *mut ID3D11Resource, SrcSubresource: UINT, pSrcBox: *const D3D11_BOX) { ((*(*This).lpVtbl).CopySubresourceRegion.unwrap_unchecked())(This, pDstResource, DstSubresource, DstX, DstY, DstZ, pSrcResource, SrcSubresource, pSrcBox) } // widl C wrapper
#[inline]
pub unsafe fn ID3D11DeviceContext_Map(This: *mut ID3D11DeviceContext, pResource: *mut ID3D11Resource, Subresource: UINT, MapType: D3D11_MAP, MapFlags: UINT, pMappedResource: *mut D3D11_MAPPED_SUBRESOURCE) -> HRESULT { ((*(*This).lpVtbl).Map.unwrap_unchecked())(This, pResource, Subresource, MapType, MapFlags, pMappedResource) } // widl C wrapper
#[inline]
pub unsafe fn ID3D11DeviceContext_Release(This: *mut ID3D11DeviceContext) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn ID3D11DeviceContext_Unmap(This: *mut ID3D11DeviceContext, pResource: *mut ID3D11Resource, Subresource: UINT) { ((*(*This).lpVtbl).Unmap.unwrap_unchecked())(This, pResource, Subresource) } // widl C wrapper
#[inline]
pub unsafe fn ID3D11Device_CreateTexture2D(This: *mut ID3D11Device, pDesc: *const D3D11_TEXTURE2D_DESC, pInitialData: *const D3D11_SUBRESOURCE_DATA, ppTexture2D: *mut *mut ID3D11Texture2D) -> HRESULT { ((*(*This).lpVtbl).CreateTexture2D.unwrap_unchecked())(This, pDesc, pInitialData, ppTexture2D) } // widl C wrapper
#[inline]
pub unsafe fn ID3D11Device_GetImmediateContext(This: *mut ID3D11Device, ppImmediateContext: *mut *mut ID3D11DeviceContext) { ((*(*This).lpVtbl).GetImmediateContext.unwrap_unchecked())(This, ppImmediateContext) } // widl C wrapper
#[inline]
pub unsafe fn ID3D11Device_Release(This: *mut ID3D11Device) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn ID3D11Texture2D_GetDesc(This: *mut ID3D11Texture2D, pDesc: *mut D3D11_TEXTURE2D_DESC) { ((*(*This).lpVtbl).GetDesc.unwrap_unchecked())(This, pDesc) } // widl C wrapper
#[inline]
pub unsafe fn ID3D11Texture2D_GetDevice(This: *mut ID3D11Texture2D, ppDevice: *mut *mut ID3D11Device) { ((*(*This).lpVtbl).GetDevice.unwrap_unchecked())(This, ppDevice) } // widl C wrapper
#[inline]
pub unsafe fn ID3D11Texture2D_QueryInterface(This: *mut ID3D11Texture2D, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn ID3D11Texture2D_Release(This: *mut ID3D11Texture2D) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12CommandAllocator_Release(This: *mut ID3D12CommandAllocator) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12CommandAllocator_Reset(This: *mut ID3D12CommandAllocator) -> HRESULT { ((*(*This).lpVtbl).Reset.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12CommandQueue_ExecuteCommandLists(This: *mut ID3D12CommandQueue, command_list_count: UINT, command_lists: *const *mut ID3D12CommandList) { ((*(*This).lpVtbl).ExecuteCommandLists.unwrap_unchecked())(This, command_list_count, command_lists) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12CommandQueue_Release(This: *mut ID3D12CommandQueue) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12CommandQueue_Signal(This: *mut ID3D12CommandQueue, fence: *mut ID3D12Fence, value: UINT64) -> HRESULT { ((*(*This).lpVtbl).Signal.unwrap_unchecked())(This, fence, value) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12CommandQueue_Wait(This: *mut ID3D12CommandQueue, fence: *mut ID3D12Fence, value: UINT64) -> HRESULT { ((*(*This).lpVtbl).Wait.unwrap_unchecked())(This, fence, value) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12Device_AddRef(This: *mut ID3D12Device) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12Device_CreateCommandAllocator(This: *mut ID3D12Device, type_: D3D12_COMMAND_LIST_TYPE, riid: *const IID, command_allocator: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).CreateCommandAllocator.unwrap_unchecked())(This, type_, riid, command_allocator) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12Device_CreateCommandList(This: *mut ID3D12Device, node_mask: UINT, type_: D3D12_COMMAND_LIST_TYPE, command_allocator: *mut ID3D12CommandAllocator, initial_pipeline_state: *mut ID3D12PipelineState, riid: *const IID, command_list: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).CreateCommandList.unwrap_unchecked())(This, node_mask, type_, command_allocator, initial_pipeline_state, riid, command_list) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12Device_CreateCommandQueue(This: *mut ID3D12Device, desc: *const D3D12_COMMAND_QUEUE_DESC, riid: *const IID, command_queue: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).CreateCommandQueue.unwrap_unchecked())(This, desc, riid, command_queue) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12Device_CreateCommittedResource(This: *mut ID3D12Device, heap_properties: *const D3D12_HEAP_PROPERTIES, heap_flags: D3D12_HEAP_FLAGS, desc: *const D3D12_RESOURCE_DESC, initial_state: D3D12_RESOURCE_STATES, optimized_clear_value: *const D3D12_CLEAR_VALUE, riid: *const IID, resource: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).CreateCommittedResource.unwrap_unchecked())(This, heap_properties, heap_flags, desc, initial_state, optimized_clear_value, riid, resource) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12Device_CreateFence(This: *mut ID3D12Device, initial_value: UINT64, flags: D3D12_FENCE_FLAGS, riid: *const IID, fence: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).CreateFence.unwrap_unchecked())(This, initial_value, flags, riid, fence) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12Device_GetCopyableFootprints(This: *mut ID3D12Device, desc: *const D3D12_RESOURCE_DESC, first_sub_resource: UINT, sub_resource_count: UINT, base_offset: UINT64, layouts: *mut D3D12_PLACED_SUBRESOURCE_FOOTPRINT, row_count: *mut UINT, row_size: *mut UINT64, total_bytes: *mut UINT64) { ((*(*This).lpVtbl).GetCopyableFootprints.unwrap_unchecked())(This, desc, first_sub_resource, sub_resource_count, base_offset, layouts, row_count, row_size, total_bytes) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12Device_Release(This: *mut ID3D12Device) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12Fence_GetCompletedValue(This: *mut ID3D12Fence) -> UINT64 { ((*(*This).lpVtbl).GetCompletedValue.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12Fence_Release(This: *mut ID3D12Fence) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12Fence_SetEventOnCompletion(This: *mut ID3D12Fence, value: UINT64, event: HANDLE) -> HRESULT { ((*(*This).lpVtbl).SetEventOnCompletion.unwrap_unchecked())(This, value, event) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12GraphicsCommandList_Close(This: *mut ID3D12GraphicsCommandList) -> HRESULT { ((*(*This).lpVtbl).Close.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12GraphicsCommandList_CopyTextureRegion(This: *mut ID3D12GraphicsCommandList, dst: *const D3D12_TEXTURE_COPY_LOCATION, dst_x: UINT, dst_y: UINT, dst_z: UINT, src: *const D3D12_TEXTURE_COPY_LOCATION, src_box: *const D3D12_BOX) { ((*(*This).lpVtbl).CopyTextureRegion.unwrap_unchecked())(This, dst, dst_x, dst_y, dst_z, src, src_box) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12GraphicsCommandList_Release(This: *mut ID3D12GraphicsCommandList) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12GraphicsCommandList_Reset(This: *mut ID3D12GraphicsCommandList, allocator: *mut ID3D12CommandAllocator, initial_state: *mut ID3D12PipelineState) -> HRESULT { ((*(*This).lpVtbl).Reset.unwrap_unchecked())(This, allocator, initial_state) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12GraphicsCommandList_ResourceBarrier(This: *mut ID3D12GraphicsCommandList, barrier_count: UINT, barriers: *const D3D12_RESOURCE_BARRIER) { ((*(*This).lpVtbl).ResourceBarrier.unwrap_unchecked())(This, barrier_count, barriers) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12Resource_GetDesc(This: *mut ID3D12Resource, __ret: *mut D3D12_RESOURCE_DESC) -> *mut D3D12_RESOURCE_DESC { ((*(*This).lpVtbl).GetDesc.unwrap_unchecked())(This, __ret) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12Resource_GetDevice(This: *mut ID3D12Resource, riid: *const IID, device: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).GetDevice.unwrap_unchecked())(This, riid, device) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12Resource_Map(This: *mut ID3D12Resource, sub_resource: UINT, read_range: *const D3D12_RANGE, data: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).Map.unwrap_unchecked())(This, sub_resource, read_range, data) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12Resource_QueryInterface(This: *mut ID3D12Resource, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12Resource_Release(This: *mut ID3D12Resource) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn ID3D12Resource_Unmap(This: *mut ID3D12Resource, sub_resource: UINT, written_range: *const D3D12_RANGE) { ((*(*This).lpVtbl).Unmap.unwrap_unchecked())(This, sub_resource, written_range) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DDeviceManager9_CloseDeviceHandle(This: *mut IDirect3DDeviceManager9, hDevice: HANDLE) -> HRESULT { ((*(*This).lpVtbl).CloseDeviceHandle.unwrap_unchecked())(This, hDevice) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DDeviceManager9_GetVideoService(This: *mut IDirect3DDeviceManager9, hDevice: HANDLE, riid: *const IID, ppService: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).GetVideoService.unwrap_unchecked())(This, hDevice, riid, ppService) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DDeviceManager9_OpenDeviceHandle(This: *mut IDirect3DDeviceManager9, phDevice: *mut HANDLE) -> HRESULT { ((*(*This).lpVtbl).OpenDeviceHandle.unwrap_unchecked())(This, phDevice) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DDeviceManager9_Release(This: *mut IDirect3DDeviceManager9) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DSurface9_GetDesc(This: *mut IDirect3DSurface9, pDesc: *mut D3DSURFACE_DESC) -> HRESULT { ((*(*This).lpVtbl).GetDesc.unwrap_unchecked())(This, pDesc) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DSurface9_LockRect(This: *mut IDirect3DSurface9, locked_rect: *mut D3DLOCKED_RECT, rect: *const RECT, flags: DWORD) -> HRESULT { ((*(*This).lpVtbl).LockRect.unwrap_unchecked())(This, locked_rect, rect, flags) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DSurface9_QueryInterface(This: *mut IDirect3DSurface9, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DSurface9_Release(This: *mut IDirect3DSurface9) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DSurface9_UnlockRect(This: *mut IDirect3DSurface9) -> HRESULT { ((*(*This).lpVtbl).UnlockRect.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IDirectXVideoProcessorService_CreateSurface(This: *mut IDirectXVideoProcessorService, width: UINT, height: UINT, backBuffers: UINT, format: D3DFORMAT, pool: D3DPOOL, usage: DWORD, dxvaType: DWORD, ppSurface: *mut *mut IDirect3DSurface9, pSharedHandle: *mut HANDLE) -> HRESULT { ((*(*This).lpVtbl).CreateSurface.unwrap_unchecked())(This, width, height, backBuffers, format, pool, usage, dxvaType, ppSurface, pSharedHandle) } // widl C wrapper
#[inline]
pub unsafe fn IDirectXVideoProcessorService_Release(This: *mut IDirectXVideoProcessorService) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMF2DBuffer2_AddRef(This: *mut IMF2DBuffer2) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMF2DBuffer2_ContiguousCopyFrom(This: *mut IMF2DBuffer2, pbSrcBuffer: *const BYTE, cbSrcBuffer: DWORD) -> HRESULT { ((*(*This).lpVtbl).ContiguousCopyFrom.unwrap_unchecked())(This, pbSrcBuffer, cbSrcBuffer) } // widl C wrapper
#[inline]
pub unsafe fn IMF2DBuffer2_ContiguousCopyTo(This: *mut IMF2DBuffer2, pbDestBuffer: *mut BYTE, cbDestBuffer: DWORD) -> HRESULT { ((*(*This).lpVtbl).ContiguousCopyTo.unwrap_unchecked())(This, pbDestBuffer, cbDestBuffer) } // widl C wrapper
#[inline]
pub unsafe fn IMF2DBuffer2_Copy2DTo(This: *mut IMF2DBuffer2, dest_buffer: *mut IMF2DBuffer2) -> HRESULT { ((*(*This).lpVtbl).Copy2DTo.unwrap_unchecked())(This, dest_buffer) } // widl C wrapper
#[inline]
pub unsafe fn IMF2DBuffer2_GetContiguousLength(This: *mut IMF2DBuffer2, pcbLength: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetContiguousLength.unwrap_unchecked())(This, pcbLength) } // widl C wrapper
#[inline]
pub unsafe fn IMF2DBuffer2_GetScanline0AndPitch(This: *mut IMF2DBuffer2, pbScanline0: *mut *mut BYTE, plPitch: *mut LONG) -> HRESULT { ((*(*This).lpVtbl).GetScanline0AndPitch.unwrap_unchecked())(This, pbScanline0, plPitch) } // widl C wrapper
#[inline]
pub unsafe fn IMF2DBuffer2_IsContiguousFormat(This: *mut IMF2DBuffer2, pfIsContiguous: *mut BOOL) -> HRESULT { ((*(*This).lpVtbl).IsContiguousFormat.unwrap_unchecked())(This, pfIsContiguous) } // widl C wrapper
#[inline]
pub unsafe fn IMF2DBuffer2_Lock2D(This: *mut IMF2DBuffer2, pbScanline0: *mut *mut BYTE, plPitch: *mut LONG) -> HRESULT { ((*(*This).lpVtbl).Lock2D.unwrap_unchecked())(This, pbScanline0, plPitch) } // widl C wrapper
#[inline]
pub unsafe fn IMF2DBuffer2_Lock2DSize(This: *mut IMF2DBuffer2, flags: MF2DBuffer_LockFlags, scanline0: *mut *mut BYTE, pitch: *mut LONG, buffer_start: *mut *mut BYTE, buffer_length: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).Lock2DSize.unwrap_unchecked())(This, flags, scanline0, pitch, buffer_start, buffer_length) } // widl C wrapper
#[inline]
pub unsafe fn IMF2DBuffer2_Release(This: *mut IMF2DBuffer2) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMF2DBuffer2_Unlock2D(This: *mut IMF2DBuffer2) -> HRESULT { ((*(*This).lpVtbl).Unlock2D.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMF2DBuffer_ContiguousCopyFrom(This: *mut IMF2DBuffer, pbSrcBuffer: *const BYTE, cbSrcBuffer: DWORD) -> HRESULT { ((*(*This).lpVtbl).ContiguousCopyFrom.unwrap_unchecked())(This, pbSrcBuffer, cbSrcBuffer) } // widl C wrapper
#[inline]
pub unsafe fn IMF2DBuffer_GetContiguousLength(This: *mut IMF2DBuffer, pcbLength: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetContiguousLength.unwrap_unchecked())(This, pcbLength) } // widl C wrapper
#[inline]
pub unsafe fn IMF2DBuffer_Release(This: *mut IMF2DBuffer) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFActivate_ActivateObject(This: *mut IMFActivate, riid: *const IID, ppv: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).ActivateObject.unwrap_unchecked())(This, riid, ppv) } // widl C wrapper
#[inline]
pub unsafe fn IMFActivate_AddRef(This: *mut IMFActivate) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFActivate_SetBlob(This: *mut IMFActivate, guidKey: *const GUID, pBuf: *const UINT8, cbBufSize: UINT32) -> HRESULT { ((*(*This).lpVtbl).SetBlob.unwrap_unchecked())(This, guidKey, pBuf, cbBufSize) } // widl C wrapper
#[inline]
pub unsafe fn IMFActivate_SetGUID(This: *mut IMFActivate, guidKey: *const GUID, guidValue: *const GUID) -> HRESULT { ((*(*This).lpVtbl).SetGUID.unwrap_unchecked())(This, guidKey, guidValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFActivate_SetString(This: *mut IMFActivate, guidKey: *const GUID, wszValue: LPCWSTR) -> HRESULT { ((*(*This).lpVtbl).SetString.unwrap_unchecked())(This, guidKey, wszValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFActivate_SetUINT32(This: *mut IMFActivate, guidKey: *const GUID, unValue: UINT32) -> HRESULT { ((*(*This).lpVtbl).SetUINT32.unwrap_unchecked())(This, guidKey, unValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAsyncCallback_AddRef(This: *mut IMFAsyncCallback) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAsyncResult_GetObject(This: *mut IMFAsyncResult, ppObject: *mut *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).GetObject.unwrap_unchecked())(This, ppObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFAsyncResult_GetStatus(This: *mut IMFAsyncResult) -> HRESULT { ((*(*This).lpVtbl).GetStatus.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAsyncResult_Release(This: *mut IMFAsyncResult) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAsyncResult_SetStatus(This: *mut IMFAsyncResult, hrStatus: HRESULT) -> HRESULT { ((*(*This).lpVtbl).SetStatus.unwrap_unchecked())(This, hrStatus) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_AddRef(This: *mut IMFAttributes) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_Compare(This: *mut IMFAttributes, pTheirs: *mut IMFAttributes, MatchType: MF_ATTRIBUTES_MATCH_TYPE, pbResult: *mut BOOL) -> HRESULT { ((*(*This).lpVtbl).Compare.unwrap_unchecked())(This, pTheirs, MatchType, pbResult) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_CompareItem(This: *mut IMFAttributes, guidKey: *const GUID, Value: *const PROPVARIANT, pbResult: *mut BOOL) -> HRESULT { ((*(*This).lpVtbl).CompareItem.unwrap_unchecked())(This, guidKey, Value, pbResult) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_CopyAllItems(This: *mut IMFAttributes, pDest: *mut IMFAttributes) -> HRESULT { ((*(*This).lpVtbl).CopyAllItems.unwrap_unchecked())(This, pDest) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_DeleteAllItems(This: *mut IMFAttributes) -> HRESULT { ((*(*This).lpVtbl).DeleteAllItems.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_DeleteItem(This: *mut IMFAttributes, guidKey: *const GUID) -> HRESULT { ((*(*This).lpVtbl).DeleteItem.unwrap_unchecked())(This, guidKey) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetAllocatedBlob(This: *mut IMFAttributes, guidKey: *const GUID, ppBuf: *mut *mut UINT8, pcbSize: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetAllocatedBlob.unwrap_unchecked())(This, guidKey, ppBuf, pcbSize) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetAllocatedString(This: *mut IMFAttributes, guidKey: *const GUID, ppwszValue: *mut LPWSTR, pcchLength: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetAllocatedString.unwrap_unchecked())(This, guidKey, ppwszValue, pcchLength) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetBlob(This: *mut IMFAttributes, guidKey: *const GUID, pBuf: *mut UINT8, cbBufSize: UINT32, pcbBlobSize: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetBlob.unwrap_unchecked())(This, guidKey, pBuf, cbBufSize, pcbBlobSize) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetBlobSize(This: *mut IMFAttributes, guidKey: *const GUID, pcbBlobSize: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetBlobSize.unwrap_unchecked())(This, guidKey, pcbBlobSize) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetCount(This: *mut IMFAttributes, pcItems: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetCount.unwrap_unchecked())(This, pcItems) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetDouble(This: *mut IMFAttributes, guidKey: *const GUID, pfValue: *mut f64) -> HRESULT { ((*(*This).lpVtbl).GetDouble.unwrap_unchecked())(This, guidKey, pfValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetGUID(This: *mut IMFAttributes, guidKey: *const GUID, pguidValue: *mut GUID) -> HRESULT { ((*(*This).lpVtbl).GetGUID.unwrap_unchecked())(This, guidKey, pguidValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetItem(This: *mut IMFAttributes, guidKey: *const GUID, pValue: *mut PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).GetItem.unwrap_unchecked())(This, guidKey, pValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetItemByIndex(This: *mut IMFAttributes, unIndex: UINT32, pguidKey: *mut GUID, pValue: *mut PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).GetItemByIndex.unwrap_unchecked())(This, unIndex, pguidKey, pValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetItemType(This: *mut IMFAttributes, guidKey: *const GUID, pType: *mut MF_ATTRIBUTE_TYPE) -> HRESULT { ((*(*This).lpVtbl).GetItemType.unwrap_unchecked())(This, guidKey, pType) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetString(This: *mut IMFAttributes, guidKey: *const GUID, pwszValue: LPWSTR, cchBufSize: UINT32, pcchLength: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetString.unwrap_unchecked())(This, guidKey, pwszValue, cchBufSize, pcchLength) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetStringLength(This: *mut IMFAttributes, guidKey: *const GUID, pcchLength: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetStringLength.unwrap_unchecked())(This, guidKey, pcchLength) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetUINT32(This: *mut IMFAttributes, guidKey: *const GUID, punValue: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetUINT32.unwrap_unchecked())(This, guidKey, punValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetUINT64(This: *mut IMFAttributes, guidKey: *const GUID, punValue: *mut UINT64) -> HRESULT { ((*(*This).lpVtbl).GetUINT64.unwrap_unchecked())(This, guidKey, punValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetUnknown(This: *mut IMFAttributes, guidKey: *const GUID, riid: *const IID, ppv: *mut LPVOID) -> HRESULT { ((*(*This).lpVtbl).GetUnknown.unwrap_unchecked())(This, guidKey, riid, ppv) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_LockStore(This: *mut IMFAttributes) -> HRESULT { ((*(*This).lpVtbl).LockStore.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_Release(This: *mut IMFAttributes) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_SetBlob(This: *mut IMFAttributes, guidKey: *const GUID, pBuf: *const UINT8, cbBufSize: UINT32) -> HRESULT { ((*(*This).lpVtbl).SetBlob.unwrap_unchecked())(This, guidKey, pBuf, cbBufSize) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_SetDouble(This: *mut IMFAttributes, guidKey: *const GUID, fValue: f64) -> HRESULT { ((*(*This).lpVtbl).SetDouble.unwrap_unchecked())(This, guidKey, fValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_SetGUID(This: *mut IMFAttributes, guidKey: *const GUID, guidValue: *const GUID) -> HRESULT { ((*(*This).lpVtbl).SetGUID.unwrap_unchecked())(This, guidKey, guidValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_SetItem(This: *mut IMFAttributes, guidKey: *const GUID, Value: *const PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).SetItem.unwrap_unchecked())(This, guidKey, Value) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_SetString(This: *mut IMFAttributes, guidKey: *const GUID, wszValue: LPCWSTR) -> HRESULT { ((*(*This).lpVtbl).SetString.unwrap_unchecked())(This, guidKey, wszValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_SetUINT32(This: *mut IMFAttributes, guidKey: *const GUID, unValue: UINT32) -> HRESULT { ((*(*This).lpVtbl).SetUINT32.unwrap_unchecked())(This, guidKey, unValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_SetUINT64(This: *mut IMFAttributes, guidKey: *const GUID, unValue: UINT64) -> HRESULT { ((*(*This).lpVtbl).SetUINT64.unwrap_unchecked())(This, guidKey, unValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_SetUnknown(This: *mut IMFAttributes, guidKey: *const GUID, pUnknown: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).SetUnknown.unwrap_unchecked())(This, guidKey, pUnknown) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_UnlockStore(This: *mut IMFAttributes) -> HRESULT { ((*(*This).lpVtbl).UnlockStore.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStreamBuffering_EnableBuffering(This: *mut IMFByteStreamBuffering, enable: BOOL) -> HRESULT { ((*(*This).lpVtbl).EnableBuffering.unwrap_unchecked())(This, enable) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStreamBuffering_Release(This: *mut IMFByteStreamBuffering) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStreamBuffering_SetBufferingParams(This: *mut IMFByteStreamBuffering, params: *mut MFBYTESTREAM_BUFFERING_PARAMS) -> HRESULT { ((*(*This).lpVtbl).SetBufferingParams.unwrap_unchecked())(This, params) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStreamBuffering_StopBuffering(This: *mut IMFByteStreamBuffering) -> HRESULT { ((*(*This).lpVtbl).StopBuffering.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStreamCacheControl_Release(This: *mut IMFByteStreamCacheControl) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStreamCacheControl_StopBackgroundTransfer(This: *mut IMFByteStreamCacheControl) -> HRESULT { ((*(*This).lpVtbl).StopBackgroundTransfer.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStreamHandler_BeginCreateObject(This: *mut IMFByteStreamHandler, stream: *mut IMFByteStream, url: *const WCHAR, flags: DWORD, props: *mut IPropertyStore, cancel_cookie: *mut *mut IUnknown, callback: *mut IMFAsyncCallback, state: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).BeginCreateObject.unwrap_unchecked())(This, stream, url, flags, props, cancel_cookie, callback, state) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStreamHandler_CancelObjectCreation(This: *mut IMFByteStreamHandler, cancel_cookie: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).CancelObjectCreation.unwrap_unchecked())(This, cancel_cookie) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStreamHandler_EndCreateObject(This: *mut IMFByteStreamHandler, result: *mut IMFAsyncResult, obj_type: *mut MF_OBJECT_TYPE, object: *mut *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).EndCreateObject.unwrap_unchecked())(This, result, obj_type, object) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStreamHandler_Release(This: *mut IMFByteStreamHandler) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStreamTimeSeek_GetTimeSeekResult(This: *mut IMFByteStreamTimeSeek, start_time: *mut QWORD, stop_time: *mut QWORD, duration: *mut QWORD) -> HRESULT { ((*(*This).lpVtbl).GetTimeSeekResult.unwrap_unchecked())(This, start_time, stop_time, duration) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStreamTimeSeek_IsTimeSeekSupported(This: *mut IMFByteStreamTimeSeek, is_supported: *mut BOOL) -> HRESULT { ((*(*This).lpVtbl).IsTimeSeekSupported.unwrap_unchecked())(This, is_supported) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStreamTimeSeek_Release(This: *mut IMFByteStreamTimeSeek) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStreamTimeSeek_TimeSeek(This: *mut IMFByteStreamTimeSeek, position: QWORD) -> HRESULT { ((*(*This).lpVtbl).TimeSeek.unwrap_unchecked())(This, position) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStream_AddRef(This: *mut IMFByteStream) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStream_BeginRead(This: *mut IMFByteStream, pb: *mut BYTE, cb: ULONG, pCallback: *mut IMFAsyncCallback, punkState: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).BeginRead.unwrap_unchecked())(This, pb, cb, pCallback, punkState) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStream_BeginWrite(This: *mut IMFByteStream, pb: *const BYTE, cb: ULONG, pCallback: *mut IMFAsyncCallback, punkState: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).BeginWrite.unwrap_unchecked())(This, pb, cb, pCallback, punkState) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStream_EndRead(This: *mut IMFByteStream, pResult: *mut IMFAsyncResult, pcbRead: *mut ULONG) -> HRESULT { ((*(*This).lpVtbl).EndRead.unwrap_unchecked())(This, pResult, pcbRead) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStream_EndWrite(This: *mut IMFByteStream, pResult: *mut IMFAsyncResult, pcbWritten: *mut ULONG) -> HRESULT { ((*(*This).lpVtbl).EndWrite.unwrap_unchecked())(This, pResult, pcbWritten) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStream_Flush(This: *mut IMFByteStream) -> HRESULT { ((*(*This).lpVtbl).Flush.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStream_GetCapabilities(This: *mut IMFByteStream, pdwCapabilities: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetCapabilities.unwrap_unchecked())(This, pdwCapabilities) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStream_GetCurrentPosition(This: *mut IMFByteStream, pqwPosition: *mut QWORD) -> HRESULT { ((*(*This).lpVtbl).GetCurrentPosition.unwrap_unchecked())(This, pqwPosition) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStream_GetLength(This: *mut IMFByteStream, pqwLength: *mut QWORD) -> HRESULT { ((*(*This).lpVtbl).GetLength.unwrap_unchecked())(This, pqwLength) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStream_IsEndOfStream(This: *mut IMFByteStream, pfEndOfStream: *mut BOOL) -> HRESULT { ((*(*This).lpVtbl).IsEndOfStream.unwrap_unchecked())(This, pfEndOfStream) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStream_QueryInterface(This: *mut IMFByteStream, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStream_Read(This: *mut IMFByteStream, pb: *mut BYTE, cb: ULONG, pcbRead: *mut ULONG) -> HRESULT { ((*(*This).lpVtbl).Read.unwrap_unchecked())(This, pb, cb, pcbRead) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStream_Release(This: *mut IMFByteStream) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStream_Seek(This: *mut IMFByteStream, SeekOrigin: MFBYTESTREAM_SEEK_ORIGIN, llSeekOffset: LONGLONG, dwSeekFlags: DWORD, pqwCurrentPosition: *mut QWORD) -> HRESULT { ((*(*This).lpVtbl).Seek.unwrap_unchecked())(This, SeekOrigin, llSeekOffset, dwSeekFlags, pqwCurrentPosition) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStream_SetCurrentPosition(This: *mut IMFByteStream, qwPosition: QWORD) -> HRESULT { ((*(*This).lpVtbl).SetCurrentPosition.unwrap_unchecked())(This, qwPosition) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStream_SetLength(This: *mut IMFByteStream, qwLength: QWORD) -> HRESULT { ((*(*This).lpVtbl).SetLength.unwrap_unchecked())(This, qwLength) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStream_Write(This: *mut IMFByteStream, pb: *const BYTE, cb: ULONG, pcbWritten: *mut ULONG) -> HRESULT { ((*(*This).lpVtbl).Write.unwrap_unchecked())(This, pb, cb, pcbWritten) } // widl C wrapper
#[inline]
pub unsafe fn IMFClock_AddRef(This: *mut IMFClock) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFClock_GetCorrelatedTime(This: *mut IMFClock, reserved: DWORD, clock_time: *mut LONGLONG, system_time: *mut MFTIME) -> HRESULT { ((*(*This).lpVtbl).GetCorrelatedTime.unwrap_unchecked())(This, reserved, clock_time, system_time) } // widl C wrapper
#[inline]
pub unsafe fn IMFClock_Release(This: *mut IMFClock) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFCollection_AddRef(This: *mut IMFCollection) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFD3D12SynchronizationObjectCommands_EnqueueResourceReady(This: *mut IMFD3D12SynchronizationObjectCommands, producer_queue: *mut ID3D12CommandQueue) -> HRESULT { ((*(*This).lpVtbl).EnqueueResourceReady.unwrap_unchecked())(This, producer_queue) } // widl C wrapper
#[inline]
pub unsafe fn IMFD3D12SynchronizationObjectCommands_EnqueueResourceReadyWait(This: *mut IMFD3D12SynchronizationObjectCommands, consumer_queue: *mut ID3D12CommandQueue) -> HRESULT { ((*(*This).lpVtbl).EnqueueResourceReadyWait.unwrap_unchecked())(This, consumer_queue) } // widl C wrapper
#[inline]
pub unsafe fn IMFD3D12SynchronizationObjectCommands_EnqueueResourceRelease(This: *mut IMFD3D12SynchronizationObjectCommands, consumer_queue: *mut ID3D12CommandQueue) -> HRESULT { ((*(*This).lpVtbl).EnqueueResourceRelease.unwrap_unchecked())(This, consumer_queue) } // widl C wrapper
#[inline]
pub unsafe fn IMFD3D12SynchronizationObjectCommands_Release(This: *mut IMFD3D12SynchronizationObjectCommands) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFD3D12SynchronizationObject_AddRef(This: *mut IMFD3D12SynchronizationObject) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFD3D12SynchronizationObject_QueryInterface(This: *mut IMFD3D12SynchronizationObject, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFD3D12SynchronizationObject_Release(This: *mut IMFD3D12SynchronizationObject) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFDXGIBuffer_SetUnknown(This: *mut IMFDXGIBuffer, guid: *const IID, data: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).SetUnknown.unwrap_unchecked())(This, guid, data) } // widl C wrapper
#[inline]
pub unsafe fn IMFDXGIDeviceManager_AddRef(This: *mut IMFDXGIDeviceManager) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFDXGIDeviceManager_CloseDeviceHandle(This: *mut IMFDXGIDeviceManager, device: HANDLE) -> HRESULT { ((*(*This).lpVtbl).CloseDeviceHandle.unwrap_unchecked())(This, device) } // widl C wrapper
#[inline]
pub unsafe fn IMFDXGIDeviceManager_GetVideoService(This: *mut IMFDXGIDeviceManager, device: HANDLE, riid: *const IID, service: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).GetVideoService.unwrap_unchecked())(This, device, riid, service) } // widl C wrapper
#[inline]
pub unsafe fn IMFDXGIDeviceManager_OpenDeviceHandle(This: *mut IMFDXGIDeviceManager, device: *mut HANDLE) -> HRESULT { ((*(*This).lpVtbl).OpenDeviceHandle.unwrap_unchecked())(This, device) } // widl C wrapper
#[inline]
pub unsafe fn IMFDXGIDeviceManager_Release(This: *mut IMFDXGIDeviceManager) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaBuffer_AddRef(This: *mut IMFMediaBuffer) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaBuffer_GetCurrentLength(This: *mut IMFMediaBuffer, pcbCurrentLength: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetCurrentLength.unwrap_unchecked())(This, pcbCurrentLength) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaBuffer_GetMaxLength(This: *mut IMFMediaBuffer, pcbMaxLength: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetMaxLength.unwrap_unchecked())(This, pcbMaxLength) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaBuffer_Lock(This: *mut IMFMediaBuffer, ppbBuffer: *mut *mut BYTE, pcbMaxLength: *mut DWORD, pcbCurrentLength: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).Lock.unwrap_unchecked())(This, ppbBuffer, pcbMaxLength, pcbCurrentLength) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaBuffer_QueryInterface(This: *mut IMFMediaBuffer, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaBuffer_Release(This: *mut IMFMediaBuffer) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaBuffer_SetCurrentLength(This: *mut IMFMediaBuffer, cbCurrentLength: DWORD) -> HRESULT { ((*(*This).lpVtbl).SetCurrentLength.unwrap_unchecked())(This, cbCurrentLength) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaBuffer_Unlock(This: *mut IMFMediaBuffer) -> HRESULT { ((*(*This).lpVtbl).Unlock.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEventGenerator_BeginGetEvent(This: *mut IMFMediaEventGenerator, pCallback: *mut IMFAsyncCallback, punkState: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).BeginGetEvent.unwrap_unchecked())(This, pCallback, punkState) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEventGenerator_EndGetEvent(This: *mut IMFMediaEventGenerator, pResult: *mut IMFAsyncResult, ppEvent: *mut *mut IMFMediaEvent) -> HRESULT { ((*(*This).lpVtbl).EndGetEvent.unwrap_unchecked())(This, pResult, ppEvent) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEventGenerator_GetEvent(This: *mut IMFMediaEventGenerator, dwFlags: DWORD, ppEvent: *mut *mut IMFMediaEvent) -> HRESULT { ((*(*This).lpVtbl).GetEvent.unwrap_unchecked())(This, dwFlags, ppEvent) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEventGenerator_QueueEvent(This: *mut IMFMediaEventGenerator, met: MediaEventType, guidExtendedType: *const GUID, hrStatus: HRESULT, pvValue: *const PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).QueueEvent.unwrap_unchecked())(This, met, guidExtendedType, hrStatus, pvValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEventGenerator_Release(This: *mut IMFMediaEventGenerator) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEventQueue_AddRef(This: *mut IMFMediaEventQueue) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEvent_AddRef(This: *mut IMFMediaEvent) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEvent_Release(This: *mut IMFMediaEvent) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaTypeHandler_AddRef(This: *mut IMFMediaTypeHandler) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_AddRef(This: *mut IMFMediaType) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_Compare(This: *mut IMFMediaType, pTheirs: *mut IMFAttributes, MatchType: MF_ATTRIBUTES_MATCH_TYPE, pbResult: *mut BOOL) -> HRESULT { ((*(*This).lpVtbl).Compare.unwrap_unchecked())(This, pTheirs, MatchType, pbResult) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_DeleteAllItems(This: *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).DeleteAllItems.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_GetAllocatedBlob(This: *mut IMFMediaType, guidKey: *const GUID, ppBuf: *mut *mut UINT8, pcbSize: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetAllocatedBlob.unwrap_unchecked())(This, guidKey, ppBuf, pcbSize) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_GetBlob(This: *mut IMFMediaType, guidKey: *const GUID, pBuf: *mut UINT8, cbBufSize: UINT32, pcbBlobSize: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetBlob.unwrap_unchecked())(This, guidKey, pBuf, cbBufSize, pcbBlobSize) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_GetBlobSize(This: *mut IMFMediaType, guidKey: *const GUID, pcbBlobSize: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetBlobSize.unwrap_unchecked())(This, guidKey, pcbBlobSize) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_GetGUID(This: *mut IMFMediaType, guidKey: *const GUID, pguidValue: *mut GUID) -> HRESULT { ((*(*This).lpVtbl).GetGUID.unwrap_unchecked())(This, guidKey, pguidValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_GetItem(This: *mut IMFMediaType, guidKey: *const GUID, pValue: *mut PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).GetItem.unwrap_unchecked())(This, guidKey, pValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_GetMajorType(This: *mut IMFMediaType, pguidMajorType: *mut GUID) -> HRESULT { ((*(*This).lpVtbl).GetMajorType.unwrap_unchecked())(This, pguidMajorType) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_GetUINT32(This: *mut IMFMediaType, guidKey: *const GUID, punValue: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetUINT32.unwrap_unchecked())(This, guidKey, punValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_GetUINT64(This: *mut IMFMediaType, guidKey: *const GUID, punValue: *mut UINT64) -> HRESULT { ((*(*This).lpVtbl).GetUINT64.unwrap_unchecked())(This, guidKey, punValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_IsEqual(This: *mut IMFMediaType, pIMediaType: *mut IMFMediaType, pdwFlags: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).IsEqual.unwrap_unchecked())(This, pIMediaType, pdwFlags) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_QueryInterface(This: *mut IMFMediaType, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_Release(This: *mut IMFMediaType) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_SetBlob(This: *mut IMFMediaType, guidKey: *const GUID, pBuf: *const UINT8, cbBufSize: UINT32) -> HRESULT { ((*(*This).lpVtbl).SetBlob.unwrap_unchecked())(This, guidKey, pBuf, cbBufSize) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_SetGUID(This: *mut IMFMediaType, guidKey: *const GUID, guidValue: *const GUID) -> HRESULT { ((*(*This).lpVtbl).SetGUID.unwrap_unchecked())(This, guidKey, guidValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_SetUINT32(This: *mut IMFMediaType, guidKey: *const GUID, unValue: UINT32) -> HRESULT { ((*(*This).lpVtbl).SetUINT32.unwrap_unchecked())(This, guidKey, unValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_SetUINT64(This: *mut IMFMediaType, guidKey: *const GUID, unValue: UINT64) -> HRESULT { ((*(*This).lpVtbl).SetUINT64.unwrap_unchecked())(This, guidKey, unValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFPluginControl_GetPreferredClsidByIndex(This: *mut IMFPluginControl, pluginType: DWORD, index: DWORD, selector: *mut LPWSTR, clsid: *mut CLSID) -> HRESULT { ((*(*This).lpVtbl).GetPreferredClsidByIndex.unwrap_unchecked())(This, pluginType, index, selector, clsid) } // widl C wrapper
#[inline]
pub unsafe fn IMFPluginControl_IsDisabled(This: *mut IMFPluginControl, pluginType: DWORD, clsid: *const CLSID) -> HRESULT { ((*(*This).lpVtbl).IsDisabled.unwrap_unchecked())(This, pluginType, clsid) } // widl C wrapper
#[inline]
pub unsafe fn IMFPluginControl_Release(This: *mut IMFPluginControl) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationDescriptor_AddRef(This: *mut IMFPresentationDescriptor) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationDescriptor_Release(This: *mut IMFPresentationDescriptor) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationTimeSource_AddRef(This: *mut IMFPresentationTimeSource) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationTimeSource_QueryInterface(This: *mut IMFPresentationTimeSource, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationTimeSource_Release(This: *mut IMFPresentationTimeSource) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSampleOutputStream_BeginWriteSample(This: *mut IMFSampleOutputStream, sample: *mut IMFSample, callback: *mut IMFAsyncCallback, state: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).BeginWriteSample.unwrap_unchecked())(This, sample, callback, state) } // widl C wrapper
#[inline]
pub unsafe fn IMFSampleOutputStream_Close(This: *mut IMFSampleOutputStream) -> HRESULT { ((*(*This).lpVtbl).Close.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSampleOutputStream_EndWriteSample(This: *mut IMFSampleOutputStream, result: *mut IMFAsyncResult) -> HRESULT { ((*(*This).lpVtbl).EndWriteSample.unwrap_unchecked())(This, result) } // widl C wrapper
#[inline]
pub unsafe fn IMFSampleOutputStream_Release(This: *mut IMFSampleOutputStream) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_AddBuffer(This: *mut IMFSample, pBuffer: *mut IMFMediaBuffer) -> HRESULT { ((*(*This).lpVtbl).AddBuffer.unwrap_unchecked())(This, pBuffer) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_AddRef(This: *mut IMFSample) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_Compare(This: *mut IMFSample, pTheirs: *mut IMFAttributes, MatchType: MF_ATTRIBUTES_MATCH_TYPE, pbResult: *mut BOOL) -> HRESULT { ((*(*This).lpVtbl).Compare.unwrap_unchecked())(This, pTheirs, MatchType, pbResult) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_CompareItem(This: *mut IMFSample, guidKey: *const GUID, Value: *const PROPVARIANT, pbResult: *mut BOOL) -> HRESULT { ((*(*This).lpVtbl).CompareItem.unwrap_unchecked())(This, guidKey, Value, pbResult) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_ConvertToContiguousBuffer(This: *mut IMFSample, ppBuffer: *mut *mut IMFMediaBuffer) -> HRESULT { ((*(*This).lpVtbl).ConvertToContiguousBuffer.unwrap_unchecked())(This, ppBuffer) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_CopyAllItems(This: *mut IMFSample, pDest: *mut IMFAttributes) -> HRESULT { ((*(*This).lpVtbl).CopyAllItems.unwrap_unchecked())(This, pDest) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_CopyToBuffer(This: *mut IMFSample, pBuffer: *mut IMFMediaBuffer) -> HRESULT { ((*(*This).lpVtbl).CopyToBuffer.unwrap_unchecked())(This, pBuffer) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_DeleteAllItems(This: *mut IMFSample) -> HRESULT { ((*(*This).lpVtbl).DeleteAllItems.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_DeleteItem(This: *mut IMFSample, guidKey: *const GUID) -> HRESULT { ((*(*This).lpVtbl).DeleteItem.unwrap_unchecked())(This, guidKey) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetAllocatedBlob(This: *mut IMFSample, guidKey: *const GUID, ppBuf: *mut *mut UINT8, pcbSize: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetAllocatedBlob.unwrap_unchecked())(This, guidKey, ppBuf, pcbSize) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetAllocatedString(This: *mut IMFSample, guidKey: *const GUID, ppwszValue: *mut LPWSTR, pcchLength: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetAllocatedString.unwrap_unchecked())(This, guidKey, ppwszValue, pcchLength) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetBlob(This: *mut IMFSample, guidKey: *const GUID, pBuf: *mut UINT8, cbBufSize: UINT32, pcbBlobSize: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetBlob.unwrap_unchecked())(This, guidKey, pBuf, cbBufSize, pcbBlobSize) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetBlobSize(This: *mut IMFSample, guidKey: *const GUID, pcbBlobSize: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetBlobSize.unwrap_unchecked())(This, guidKey, pcbBlobSize) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetBufferByIndex(This: *mut IMFSample, dwIndex: DWORD, ppBuffer: *mut *mut IMFMediaBuffer) -> HRESULT { ((*(*This).lpVtbl).GetBufferByIndex.unwrap_unchecked())(This, dwIndex, ppBuffer) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetBufferCount(This: *mut IMFSample, pdwBufferCount: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetBufferCount.unwrap_unchecked())(This, pdwBufferCount) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetCount(This: *mut IMFSample, pcItems: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetCount.unwrap_unchecked())(This, pcItems) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetDouble(This: *mut IMFSample, guidKey: *const GUID, pfValue: *mut f64) -> HRESULT { ((*(*This).lpVtbl).GetDouble.unwrap_unchecked())(This, guidKey, pfValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetGUID(This: *mut IMFSample, guidKey: *const GUID, pguidValue: *mut GUID) -> HRESULT { ((*(*This).lpVtbl).GetGUID.unwrap_unchecked())(This, guidKey, pguidValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetItem(This: *mut IMFSample, guidKey: *const GUID, pValue: *mut PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).GetItem.unwrap_unchecked())(This, guidKey, pValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetItemByIndex(This: *mut IMFSample, unIndex: UINT32, pguidKey: *mut GUID, pValue: *mut PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).GetItemByIndex.unwrap_unchecked())(This, unIndex, pguidKey, pValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetItemType(This: *mut IMFSample, guidKey: *const GUID, pType: *mut MF_ATTRIBUTE_TYPE) -> HRESULT { ((*(*This).lpVtbl).GetItemType.unwrap_unchecked())(This, guidKey, pType) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetSampleDuration(This: *mut IMFSample, phnsSampleDuration: *mut LONGLONG) -> HRESULT { ((*(*This).lpVtbl).GetSampleDuration.unwrap_unchecked())(This, phnsSampleDuration) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetSampleFlags(This: *mut IMFSample, pdwSampleFlags: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetSampleFlags.unwrap_unchecked())(This, pdwSampleFlags) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetSampleTime(This: *mut IMFSample, phnsSampleTime: *mut LONGLONG) -> HRESULT { ((*(*This).lpVtbl).GetSampleTime.unwrap_unchecked())(This, phnsSampleTime) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetString(This: *mut IMFSample, guidKey: *const GUID, pwszValue: LPWSTR, cchBufSize: UINT32, pcchLength: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetString.unwrap_unchecked())(This, guidKey, pwszValue, cchBufSize, pcchLength) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetStringLength(This: *mut IMFSample, guidKey: *const GUID, pcchLength: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetStringLength.unwrap_unchecked())(This, guidKey, pcchLength) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetTotalLength(This: *mut IMFSample, pcbTotalLength: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetTotalLength.unwrap_unchecked())(This, pcbTotalLength) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetUINT32(This: *mut IMFSample, guidKey: *const GUID, punValue: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetUINT32.unwrap_unchecked())(This, guidKey, punValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetUINT64(This: *mut IMFSample, guidKey: *const GUID, punValue: *mut UINT64) -> HRESULT { ((*(*This).lpVtbl).GetUINT64.unwrap_unchecked())(This, guidKey, punValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetUnknown(This: *mut IMFSample, guidKey: *const GUID, riid: *const IID, ppv: *mut LPVOID) -> HRESULT { ((*(*This).lpVtbl).GetUnknown.unwrap_unchecked())(This, guidKey, riid, ppv) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_LockStore(This: *mut IMFSample) -> HRESULT { ((*(*This).lpVtbl).LockStore.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_QueryInterface(This: *mut IMFSample, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_Release(This: *mut IMFSample) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_RemoveAllBuffers(This: *mut IMFSample) -> HRESULT { ((*(*This).lpVtbl).RemoveAllBuffers.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_RemoveBufferByIndex(This: *mut IMFSample, dwIndex: DWORD) -> HRESULT { ((*(*This).lpVtbl).RemoveBufferByIndex.unwrap_unchecked())(This, dwIndex) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetBlob(This: *mut IMFSample, guidKey: *const GUID, pBuf: *const UINT8, cbBufSize: UINT32) -> HRESULT { ((*(*This).lpVtbl).SetBlob.unwrap_unchecked())(This, guidKey, pBuf, cbBufSize) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetDouble(This: *mut IMFSample, guidKey: *const GUID, fValue: f64) -> HRESULT { ((*(*This).lpVtbl).SetDouble.unwrap_unchecked())(This, guidKey, fValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetGUID(This: *mut IMFSample, guidKey: *const GUID, guidValue: *const GUID) -> HRESULT { ((*(*This).lpVtbl).SetGUID.unwrap_unchecked())(This, guidKey, guidValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetItem(This: *mut IMFSample, guidKey: *const GUID, Value: *const PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).SetItem.unwrap_unchecked())(This, guidKey, Value) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetSampleDuration(This: *mut IMFSample, hnsSampleDuration: LONGLONG) -> HRESULT { ((*(*This).lpVtbl).SetSampleDuration.unwrap_unchecked())(This, hnsSampleDuration) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetSampleFlags(This: *mut IMFSample, dwSampleFlags: DWORD) -> HRESULT { ((*(*This).lpVtbl).SetSampleFlags.unwrap_unchecked())(This, dwSampleFlags) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetSampleTime(This: *mut IMFSample, hnsSampleTime: LONGLONG) -> HRESULT { ((*(*This).lpVtbl).SetSampleTime.unwrap_unchecked())(This, hnsSampleTime) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetString(This: *mut IMFSample, guidKey: *const GUID, wszValue: LPCWSTR) -> HRESULT { ((*(*This).lpVtbl).SetString.unwrap_unchecked())(This, guidKey, wszValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetUINT32(This: *mut IMFSample, guidKey: *const GUID, unValue: UINT32) -> HRESULT { ((*(*This).lpVtbl).SetUINT32.unwrap_unchecked())(This, guidKey, unValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetUINT64(This: *mut IMFSample, guidKey: *const GUID, unValue: UINT64) -> HRESULT { ((*(*This).lpVtbl).SetUINT64.unwrap_unchecked())(This, guidKey, unValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetUnknown(This: *mut IMFSample, guidKey: *const GUID, pUnknown: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).SetUnknown.unwrap_unchecked())(This, guidKey, pUnknown) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_UnlockStore(This: *mut IMFSample) -> HRESULT { ((*(*This).lpVtbl).UnlockStore.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSchemeHandler_BeginCreateObject(This: *mut IMFSchemeHandler, url: *const WCHAR, flags: DWORD, props: *mut IPropertyStore, cancel_cookie: *mut *mut IUnknown, callback: *mut IMFAsyncCallback, state: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).BeginCreateObject.unwrap_unchecked())(This, url, flags, props, cancel_cookie, callback, state) } // widl C wrapper
#[inline]
pub unsafe fn IMFSchemeHandler_CancelObjectCreation(This: *mut IMFSchemeHandler, cancel_cookie: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).CancelObjectCreation.unwrap_unchecked())(This, cancel_cookie) } // widl C wrapper
#[inline]
pub unsafe fn IMFSchemeHandler_EndCreateObject(This: *mut IMFSchemeHandler, result: *mut IMFAsyncResult, obj_type: *mut MF_OBJECT_TYPE, object: *mut *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).EndCreateObject.unwrap_unchecked())(This, result, obj_type, object) } // widl C wrapper
#[inline]
pub unsafe fn IMFSchemeHandler_Release(This: *mut IMFSchemeHandler) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSourceResolver_AddRef(This: *mut IMFSourceResolver) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSourceResolver_Release(This: *mut IMFSourceResolver) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamDescriptor_AddRef(This: *mut IMFStreamDescriptor) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamDescriptor_Release(This: *mut IMFStreamDescriptor) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTrackedSample_QueryInterface(This: *mut IMFTrackedSample, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFTrackedSample_Release(This: *mut IMFTrackedSample) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTrackedSample_SetAllocator(This: *mut IMFTrackedSample, sample_allocator: *mut IMFAsyncCallback, state: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).SetAllocator.unwrap_unchecked())(This, sample_allocator, state) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_QueryInterface(This: *mut IMFTransform, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_Release(This: *mut IMFTransform) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocatorEx_AddRef(This: *mut IMFVideoSampleAllocatorEx) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocatorEx_QueryInterface(This: *mut IMFVideoSampleAllocatorEx, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocatorEx_Release(This: *mut IMFVideoSampleAllocatorEx) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocatorNotify_AddRef(This: *mut IMFVideoSampleAllocatorNotify) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocatorNotify_NotifyRelease(This: *mut IMFVideoSampleAllocatorNotify) -> HRESULT { ((*(*This).lpVtbl).NotifyRelease.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocatorNotify_Release(This: *mut IMFVideoSampleAllocatorNotify) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMediaBuffer_AddRef(This: *mut IMediaBuffer) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMediaBuffer_QueryInterface(This: *mut IMediaBuffer, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMediaBuffer_Release(This: *mut IMediaBuffer) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IPropertyStore_AddRef(This: *mut IPropertyStore) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IPropertyStore_Commit(This: *mut IPropertyStore) -> HRESULT { ((*(*This).lpVtbl).Commit.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IPropertyStore_GetAt(This: *mut IPropertyStore, iProp: DWORD, pkey: *mut PROPERTYKEY) -> HRESULT { ((*(*This).lpVtbl).GetAt.unwrap_unchecked())(This, iProp, pkey) } // widl C wrapper
#[inline]
pub unsafe fn IPropertyStore_GetCount(This: *mut IPropertyStore, cProps: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetCount.unwrap_unchecked())(This, cProps) } // widl C wrapper
#[inline]
pub unsafe fn IPropertyStore_GetValue(This: *mut IPropertyStore, key: *const PROPERTYKEY, pv: *mut PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).GetValue.unwrap_unchecked())(This, key, pv) } // widl C wrapper
#[inline]
pub unsafe fn IPropertyStore_Release(This: *mut IPropertyStore) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IPropertyStore_SetValue(This: *mut IPropertyStore, key: *const PROPERTYKEY, propvar: *const PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).SetValue.unwrap_unchecked())(This, key, propvar) } // widl C wrapper
#[inline]
pub unsafe fn IRtwqAsyncCallback_AddRef(This: *mut IRtwqAsyncCallback) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IRtwqAsyncCallback_Release(This: *mut IRtwqAsyncCallback) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IRtwqAsyncResult_AddRef(This: *mut IRtwqAsyncResult) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IRtwqAsyncResult_GetObject(This: *mut IRtwqAsyncResult, object: *mut *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).GetObject.unwrap_unchecked())(This, object) } // widl C wrapper
#[inline]
pub unsafe fn IRtwqAsyncResult_GetStateNoAddRef(This: *mut IRtwqAsyncResult) -> *mut IUnknown { ((*(*This).lpVtbl).GetStateNoAddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IRtwqAsyncResult_GetStatus(This: *mut IRtwqAsyncResult) -> HRESULT { ((*(*This).lpVtbl).GetStatus.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IRtwqAsyncResult_Release(This: *mut IRtwqAsyncResult) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IRtwqAsyncResult_SetStatus(This: *mut IRtwqAsyncResult, status: HRESULT) -> HRESULT { ((*(*This).lpVtbl).SetStatus.unwrap_unchecked())(This, status) } // widl C wrapper
#[inline]
pub unsafe fn IStream_AddRef(This: *mut IStream) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IStream_Commit(This: *mut IStream, grfCommitFlags: DWORD) -> HRESULT { ((*(*This).lpVtbl).Commit.unwrap_unchecked())(This, grfCommitFlags) } // widl C wrapper
#[inline]
pub unsafe fn IStream_Release(This: *mut IStream) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IStream_Seek(This: *mut IStream, dlibMove: LARGE_INTEGER, dwOrigin: DWORD, plibNewPosition: *mut ULARGE_INTEGER) -> HRESULT { ((*(*This).lpVtbl).Seek.unwrap_unchecked())(This, dlibMove, dwOrigin, plibNewPosition) } // widl C wrapper
#[inline]
pub unsafe fn IStream_SetSize(This: *mut IStream, libNewSize: ULARGE_INTEGER) -> HRESULT { ((*(*This).lpVtbl).SetSize.unwrap_unchecked())(This, libNewSize) } // widl C wrapper
#[inline]
pub unsafe fn IStream_Stat(This: *mut IStream, pstatstg: *mut STATSTG, grfStatFlag: DWORD) -> HRESULT { ((*(*This).lpVtbl).Stat.unwrap_unchecked())(This, pstatstg, grfStatFlag) } // widl C wrapper
#[inline]
pub unsafe fn IUnknown_AddRef(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IUnknown_QueryInterface(This: *mut IUnknown, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IUnknown_Release(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
pub const MF_E_ATTRIBUTENOTFOUND: HRESULT = ((0xc00d36e6u32) as HRESULT);
pub const MF_E_BAD_STARTUP_VERSION: HRESULT = ((0xc00d36e3u32) as HRESULT);
pub const MF_E_BUFFERTOOSMALL: HRESULT = ((0xc00d36b1u32) as HRESULT);
pub const MF_E_DXGI_DEVICE_NOT_INITIALIZED: HRESULT = ((0x80041000u32) as HRESULT);
pub const MF_E_DXGI_NEW_VIDEO_DEVICE: HRESULT = ((0x80041001u32) as HRESULT);
pub const MF_E_DXGI_VIDEO_DEVICE_LOCKED: HRESULT = ((0x80041002u32) as HRESULT);
pub const MF_E_INVALIDMEDIATYPE: HRESULT = ((0xc00d36b4u32) as HRESULT);
pub const MF_E_INVALIDREQUEST: HRESULT = ((0xc00d36b2u32) as HRESULT);
pub const MF_E_INVALIDTYPE: HRESULT = ((0xc00d36bdu32) as HRESULT);
pub const MF_E_MULTIPLE_BEGIN: HRESULT = ((0xc00d36d9u32) as HRESULT);
pub const MF_E_MULTIPLE_SUBSCRIBERS: HRESULT = ((0xc00d36dau32) as HRESULT);
pub const MF_E_NOTACCEPTING: HRESULT = ((0xc00d36b5u32) as HRESULT);
pub const MF_E_NOT_FOUND: HRESULT = ((0xc00d36d5u32) as HRESULT);
pub const MF_E_NOT_INITIALIZED: HRESULT = ((0xc00d36b6u32) as HRESULT);
pub const MF_E_NO_EVENTS_AVAILABLE: HRESULT = ((0xc00d3e80u32) as HRESULT);
pub const MF_E_NO_MORE_TYPES: HRESULT = ((0xc00d36b9u32) as HRESULT);
pub const MF_E_NO_SAMPLE_DURATION: HRESULT = ((0xc00d36c9u32) as HRESULT);
pub const MF_E_NO_SAMPLE_TIMESTAMP: HRESULT = ((0xc00d36c8u32) as HRESULT);
pub const MF_E_SAMPLEALLOCATOR_EMPTY: HRESULT = ((0xc00d4a3eu32) as HRESULT);
pub const MF_E_SHUTDOWN: HRESULT = ((0xc00d3e85u32) as HRESULT);
pub const MF_E_UNEXPECTED: HRESULT = ((0xc00d36bbu32) as HRESULT);
pub const MF_E_UNSUPPORTED_BYTESTREAM_TYPE: HRESULT = ((0xc00d36c4u32) as HRESULT);
pub const MF_E_UNSUPPORTED_RATE: HRESULT = ((0xc00d36d0u32) as HRESULT);
pub const MF_E_UNSUPPORTED_REPRESENTATION: HRESULT = ((0xc00d36b7u32) as HRESULT);
pub const MF_E_UNSUPPORTED_SCHEME: HRESULT = ((0xc00d36c3u32) as HRESULT);
pub const MF_S_MULTIPLE_BEGIN: HRESULT = ((0x000d36d8) as HRESULT);
pub type REFCLSID = *const CLSID;
pub type REFGUID = *const GUID;
pub type REFIID = *const IID;
pub type REFPROPERTYKEY = *const PROPERTYKEY;
pub type REFPROPVARIANT = *const PROPVARIANT;
pub const S_FALSE: HRESULT = ((1) as HRESULT);
pub const S_OK: HRESULT = ((0) as HRESULT);
pub use crate::{IsEqualCLSID, IsEqualIID, MAKEFOURCC, PropVariantInit, max};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/win32/mfplat/buffer.c:35: #define ALIGN_SIZE(size, alignment) (((size) + (alignment)) & ~((alignment)))
#[allow(unused_macros)]
macro_rules! ALIGN_SIZE {
    ($size:expr, $alignment:expr $(,)?) => {
        ((($size) + ($alignment)) & !(($alignment)))
    };
}
#[allow(unused_imports)]
pub(crate) use ALIGN_SIZE;
// dll/win32/mfplat/mfplat_private.h:117: header-inline `debugstr_propvar`
#[allow(dead_code)]
#[inline]
pub unsafe fn debugstr_propvar(v: *const PROPVARIANT) -> *const core::ffi::c_char {
    if v.is_null() {
        return c"(null)".as_ptr();
    }

    let vt = (*v).vt;
    if (vt) as u32 == VT_EMPTY {
        wine_dbg_sprintf(c"%p {VT_EMPTY}".as_ptr(), v)
    } else if (vt) as u32 == VT_NULL {
        wine_dbg_sprintf(c"%p {VT_NULL}".as_ptr(), v)
    } else if vt == (VT_UI4 as u16) {
        wine_dbg_sprintf(c"%p {VT_UI4: %ld}".as_ptr(), v, (*v).__bindgen_anon_1.ulVal)
    } else if vt == (VT_UI8 as u16) {
        wine_dbg_sprintf(
            c"%p {VT_UI8: %s}".as_ptr(),
            v,
            wine_dbgstr_longlong((*v).__bindgen_anon_1.uhVal.QuadPart),
        )
    } else if (vt) as u32 == VT_I8 {
        wine_dbg_sprintf(
            c"%p {VT_I8: %s}".as_ptr(),
            v,
            wine_dbgstr_longlong(((*v).__bindgen_anon_1.hVal.QuadPart) as ULONGLONG),
        )
    } else if (vt) as u32 == VT_R4 {
        wine_dbg_sprintf(c"%p {VT_R4: %.8e}".as_ptr(), v, (*v).__bindgen_anon_1.fltVal as f64)
    } else if vt == (VT_R8 as u16) {
        wine_dbg_sprintf(c"%p {VT_R8: %lf}".as_ptr(), v, (*v).__bindgen_anon_1.dblVal)
    } else if vt == VT_CLSID {
        wine_dbg_sprintf(
            c"%p {VT_CLSID: %s}".as_ptr(),
            v,
            debugstr_mf_guid((*v).__bindgen_anon_1.puuid),
        )
    } else if vt == VT_LPWSTR {
        wine_dbg_sprintf(
            c"%p {VT_LPWSTR: %s}".as_ptr(),
            v,
            wine_dbgstr_w((*v).__bindgen_anon_1.pwszVal),
        )
    } else if vt == (VT_VECTOR | VT_UI1) {
        wine_dbg_sprintf(
            c"%p {VT_VECTOR|VT_UI1: %p}".as_ptr(),
            v,
            (*v).__bindgen_anon_1.caub.pElems,
        )
    } else if vt == VT_UNKNOWN {
        wine_dbg_sprintf(c"%p {VT_UNKNOWN: %p}".as_ptr(), v, (*v).__bindgen_anon_1.punkVal)
    } else {
        wine_dbg_sprintf(c"%p {vt %#x}".as_ptr(), v, (*v).vt as u32)
    }
}
// dll/win32/mfplat/mfplat_private.h:187: header-inline `debugstr_time`
#[allow(dead_code)]
#[inline]
pub unsafe fn debugstr_time(time: LONGLONG) -> *const core::ffi::c_char {
    let mut abstime: ULONGLONG = if time >= 0 {
        time as ULONGLONG
    } else {
        (time.wrapping_neg()) as ULONGLONG
    };
    let mut i: usize = 0;
    let mut j: usize = 0;
    let mut buffer = [0u8; 23];
    let mut rev = [0u8; 23];

    while abstime != 0 || i <= 8 {
        buffer[i] = b'0' + (abstime % 10) as u8;
        i += 1;
        abstime /= 10;
        if i == 7 {
            buffer[i] = b'.';
            i += 1;
        }
    }
    if time < 0 {
        buffer[i] = b'-';
        i += 1;
    }

    while i > 0 {
        i -= 1;
        rev[j] = buffer[i];
        j += 1;
    }
    while rev[j - 1] == b'0' && rev[j - 2] != b'.' {
        j -= 1;
    }
    rev[j] = 0;

    wine_dbg_sprintf(c"%s".as_ptr(), rev.as_ptr() as *const core::ffi::c_char)
}
// ==== end port macros ====
