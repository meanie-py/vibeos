//! d3d9-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
// wine/wined3d.h's static inline helpers (sdk/lib/rust/wined3d-h): the integer ones by
// name, the struct-bound ones expanded here against this crate's island types.
pub use wined3d_h::{wined3d_bit_scan, wined3d_bound_range, wined3d_log2i};
wined3d_h::wined3d_inlines!();
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

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const CP_ACP: u32 = 0; // advapi32-sys/src/bindgen_amd64.rs // gen:b2344a3cbdb6
pub const D3D9_DEVICE_STATE_LOST: d3d9_device_state = d3d9_device_state_D3D9_DEVICE_STATE_LOST; // island enum // gen:5d6b237e1181
pub const D3D9_DEVICE_STATE_NOT_RESET: d3d9_device_state = d3d9_device_state_D3D9_DEVICE_STATE_NOT_RESET; // island enum // gen:495e893af1de
pub const D3D9_DEVICE_STATE_OK: d3d9_device_state = d3d9_device_state_D3D9_DEVICE_STATE_OK; // island enum // gen:8a6e912d1c66
pub const D3DCREATE_MULTITHREADED: i32 = 4; // d3d8-rust/src/prelude.rs // gen:70e88f0b7c3c
pub const D3DDECLMETHOD_DEFAULT: _D3DDECLMETHOD = _D3DDECLMETHOD_D3DDECLMETHOD_DEFAULT; // island enum // gen:af3aec2bf4fe
pub const D3DDECLTYPE_D3DCOLOR: _D3DDECLTYPE = _D3DDECLTYPE_D3DDECLTYPE_D3DCOLOR; // island enum // gen:39f120da9c49
pub const D3DDECLTYPE_FLOAT1: _D3DDECLTYPE = _D3DDECLTYPE_D3DDECLTYPE_FLOAT1; // island enum // gen:918b05b0005d
pub const D3DDECLTYPE_FLOAT2: _D3DDECLTYPE = _D3DDECLTYPE_D3DDECLTYPE_FLOAT2; // island enum // gen:49f9174e00f1
pub const D3DDECLTYPE_FLOAT3: _D3DDECLTYPE = _D3DDECLTYPE_D3DDECLTYPE_FLOAT3; // island enum // gen:637c30fe389b
pub const D3DDECLTYPE_FLOAT4: _D3DDECLTYPE = _D3DDECLTYPE_D3DDECLTYPE_FLOAT4; // island enum // gen:c52b3477b4d3
pub const D3DDECLTYPE_UBYTE4: _D3DDECLTYPE = _D3DDECLTYPE_D3DDECLTYPE_UBYTE4; // island enum // gen:a2132e92396a
pub const D3DDECLTYPE_UNUSED: _D3DDECLTYPE = _D3DDECLTYPE_D3DDECLTYPE_UNUSED; // island enum // gen:a08c44a7e864
pub const D3DDECLUSAGE_BLENDINDICES: _D3DDECLUSAGE = _D3DDECLUSAGE_D3DDECLUSAGE_BLENDINDICES; // island enum // gen:d0464d8a9338
pub const D3DDECLUSAGE_BLENDWEIGHT: _D3DDECLUSAGE = _D3DDECLUSAGE_D3DDECLUSAGE_BLENDWEIGHT; // island enum // gen:d0097720afe5
pub const D3DDECLUSAGE_COLOR: _D3DDECLUSAGE = _D3DDECLUSAGE_D3DDECLUSAGE_COLOR; // island enum // gen:39a1cc68e9a6
pub const D3DDECLUSAGE_NORMAL: _D3DDECLUSAGE = _D3DDECLUSAGE_D3DDECLUSAGE_NORMAL; // island enum // gen:f7aeae1199ca
pub const D3DDECLUSAGE_POSITION: _D3DDECLUSAGE = _D3DDECLUSAGE_D3DDECLUSAGE_POSITION; // island enum // gen:4430b1c0b223
pub const D3DDECLUSAGE_POSITIONT: _D3DDECLUSAGE = _D3DDECLUSAGE_D3DDECLUSAGE_POSITIONT; // island enum // gen:f307ce10079d
pub const D3DDECLUSAGE_PSIZE: _D3DDECLUSAGE = _D3DDECLUSAGE_D3DDECLUSAGE_PSIZE; // island enum // gen:081d48b55db8
pub const D3DDECLUSAGE_TEXCOORD: _D3DDECLUSAGE = _D3DDECLUSAGE_D3DDECLUSAGE_TEXCOORD; // island enum // gen:3c9949ea1758
pub const D3DFMT_A16B16G16R16: _D3DFORMAT = _D3DFORMAT_D3DFMT_A16B16G16R16; // island enum // gen:dc7a84ae76c8
pub const D3DFMT_A16B16G16R16F: _D3DFORMAT = _D3DFORMAT_D3DFMT_A16B16G16R16F; // island enum // gen:fe9ee5d5fa99
pub const D3DFMT_A1R5G5B5: _D3DFORMAT = _D3DFORMAT_D3DFMT_A1R5G5B5; // island enum // gen:6f9a88590099
pub const D3DFMT_A2B10G10R10: _D3DFORMAT = _D3DFORMAT_D3DFMT_A2B10G10R10; // island enum // gen:bb0e3657ab41
pub const D3DFMT_A2R10G10B10: _D3DFORMAT = _D3DFORMAT_D3DFMT_A2R10G10B10; // island enum // gen:e5b898ed7004
pub const D3DFMT_A2W10V10U10: _D3DFORMAT = _D3DFORMAT_D3DFMT_A2W10V10U10; // island enum // gen:79e10536f3e8
pub const D3DFMT_A32B32G32R32F: _D3DFORMAT = _D3DFORMAT_D3DFMT_A32B32G32R32F; // island enum // gen:211f99ada322
pub const D3DFMT_A4L4: _D3DFORMAT = _D3DFORMAT_D3DFMT_A4L4; // island enum // gen:e185a882a5ac
pub const D3DFMT_A4R4G4B4: _D3DFORMAT = _D3DFORMAT_D3DFMT_A4R4G4B4; // island enum // gen:02f39479f797
pub const D3DFMT_A8: _D3DFORMAT = _D3DFORMAT_D3DFMT_A8; // island enum // gen:f52159c6641c
pub const D3DFMT_A8B8G8R8: _D3DFORMAT = _D3DFORMAT_D3DFMT_A8B8G8R8; // island enum // gen:02846a4a1f74
pub const D3DFMT_A8L8: _D3DFORMAT = _D3DFORMAT_D3DFMT_A8L8; // island enum // gen:9d234b7c31cc
pub const D3DFMT_A8P8: _D3DFORMAT = _D3DFORMAT_D3DFMT_A8P8; // island enum // gen:7dd2abeccce8
pub const D3DFMT_A8R3G3B2: _D3DFORMAT = _D3DFORMAT_D3DFMT_A8R3G3B2; // island enum // gen:af1f63301ce2
pub const D3DFMT_A8R8G8B8: _D3DFORMAT = _D3DFORMAT_D3DFMT_A8R8G8B8; // island enum // gen:9f6f0fc4c1b9
pub const D3DFMT_CxV8U8: _D3DFORMAT = _D3DFORMAT_D3DFMT_CxV8U8; // island enum // gen:9223adfb0e68
pub const D3DFMT_D15S1: _D3DFORMAT = _D3DFORMAT_D3DFMT_D15S1; // island enum // gen:8b80cd5d4b1b
pub const D3DFMT_D16: _D3DFORMAT = _D3DFORMAT_D3DFMT_D16; // island enum // gen:4752cb83efb7
pub const D3DFMT_D16_LOCKABLE: _D3DFORMAT = _D3DFORMAT_D3DFMT_D16_LOCKABLE; // island enum // gen:34db1e794b16
pub const D3DFMT_D24FS8: _D3DFORMAT = _D3DFORMAT_D3DFMT_D24FS8; // island enum // gen:f4fda5f3b1bd
pub const D3DFMT_D24S8: _D3DFORMAT = _D3DFORMAT_D3DFMT_D24S8; // island enum // gen:e1c4526080cb
pub const D3DFMT_D24X4S4: _D3DFORMAT = _D3DFORMAT_D3DFMT_D24X4S4; // island enum // gen:7563c3f06a76
pub const D3DFMT_D24X8: _D3DFORMAT = _D3DFORMAT_D3DFMT_D24X8; // island enum // gen:ad515ddad853
pub const D3DFMT_D32: _D3DFORMAT = _D3DFORMAT_D3DFMT_D32; // island enum // gen:520f2b18c70b
pub const D3DFMT_D32F_LOCKABLE: _D3DFORMAT = _D3DFORMAT_D3DFMT_D32F_LOCKABLE; // island enum // gen:e2cc72704f0c
pub const D3DFMT_G16R16: _D3DFORMAT = _D3DFORMAT_D3DFMT_G16R16; // island enum // gen:2f46745cc45d
pub const D3DFMT_G16R16F: _D3DFORMAT = _D3DFORMAT_D3DFMT_G16R16F; // island enum // gen:06c5270548f8
pub const D3DFMT_G32R32F: _D3DFORMAT = _D3DFORMAT_D3DFMT_G32R32F; // island enum // gen:e0976d93fca1
pub const D3DFMT_INDEX16: _D3DFORMAT = _D3DFORMAT_D3DFMT_INDEX16; // island enum // gen:20a989fae87c
pub const D3DFMT_INDEX32: _D3DFORMAT = _D3DFORMAT_D3DFMT_INDEX32; // island enum // gen:2a6926658c3d
pub const D3DFMT_L16: _D3DFORMAT = _D3DFORMAT_D3DFMT_L16; // island enum // gen:e139512b468a
pub const D3DFMT_L6V5U5: _D3DFORMAT = _D3DFORMAT_D3DFMT_L6V5U5; // island enum // gen:6962003f9e23
pub const D3DFMT_L8: _D3DFORMAT = _D3DFORMAT_D3DFMT_L8; // island enum // gen:d43c484f6894
pub const D3DFMT_P8: _D3DFORMAT = _D3DFORMAT_D3DFMT_P8; // island enum // gen:4885307876d4
pub const D3DFMT_Q16W16V16U16: _D3DFORMAT = _D3DFORMAT_D3DFMT_Q16W16V16U16; // island enum // gen:81dab6d1a598
pub const D3DFMT_Q8W8V8U8: _D3DFORMAT = _D3DFORMAT_D3DFMT_Q8W8V8U8; // island enum // gen:6b18c3cbbf7a
pub const D3DFMT_R16F: _D3DFORMAT = _D3DFORMAT_D3DFMT_R16F; // island enum // gen:6b6254241b8d
pub const D3DFMT_R32F: _D3DFORMAT = _D3DFORMAT_D3DFMT_R32F; // island enum // gen:6e13bb8107c8
pub const D3DFMT_R3G3B2: _D3DFORMAT = _D3DFORMAT_D3DFMT_R3G3B2; // island enum // gen:59b5c58dbdd1
pub const D3DFMT_R5G6B5: _D3DFORMAT = _D3DFORMAT_D3DFMT_R5G6B5; // island enum // gen:435d5f64d38c
pub const D3DFMT_R8G8B8: _D3DFORMAT = _D3DFORMAT_D3DFMT_R8G8B8; // island enum // gen:f88f76c0d7f8
pub const D3DFMT_UNKNOWN: _D3DFORMAT = _D3DFORMAT_D3DFMT_UNKNOWN; // island enum // gen:febb94381c1a
pub const D3DFMT_V16U16: _D3DFORMAT = _D3DFORMAT_D3DFMT_V16U16; // island enum // gen:eb7099a85cfc
pub const D3DFMT_V8U8: _D3DFORMAT = _D3DFORMAT_D3DFMT_V8U8; // island enum // gen:51a24d4af040
pub const D3DFMT_VERTEXDATA: _D3DFORMAT = _D3DFORMAT_D3DFMT_VERTEXDATA; // island enum // gen:6cb2619ce763
pub const D3DFMT_X1R5G5B5: _D3DFORMAT = _D3DFORMAT_D3DFMT_X1R5G5B5; // island enum // gen:b1217078b2eb
pub const D3DFMT_X4R4G4B4: _D3DFORMAT = _D3DFORMAT_D3DFMT_X4R4G4B4; // island enum // gen:095b89672f24
pub const D3DFMT_X8B8G8R8: _D3DFORMAT = _D3DFORMAT_D3DFMT_X8B8G8R8; // island enum // gen:f0bf51bc9d92
pub const D3DFMT_X8L8V8U8: _D3DFORMAT = _D3DFORMAT_D3DFMT_X8L8V8U8; // island enum // gen:62d884fcd754
pub const D3DFMT_X8R8G8B8: _D3DFORMAT = _D3DFORMAT_D3DFMT_X8R8G8B8; // island enum // gen:c6e7a7970fd6
pub const D3DMULTISAMPLE_16_SAMPLES: _D3DMULTISAMPLE_TYPE = _D3DMULTISAMPLE_TYPE_D3DMULTISAMPLE_16_SAMPLES; // island enum // gen:e30d633e02ae
pub const D3DPOOL_DEFAULT: _D3DPOOL = _D3DPOOL_D3DPOOL_DEFAULT; // island enum // gen:da3b87a3608d
pub const D3DPOOL_MANAGED: _D3DPOOL = _D3DPOOL_D3DPOOL_MANAGED; // island enum // gen:9c0f686269de
pub const D3DPOOL_SCRATCH: _D3DPOOL = _D3DPOOL_D3DPOOL_SCRATCH; // island enum // gen:67c884565452
pub const D3DPOOL_SYSTEMMEM: _D3DPOOL = _D3DPOOL_D3DPOOL_SYSTEMMEM; // island enum // gen:1fd728eeb6dc
pub const D3DPRASTERCAPS_ANISOTROPY: u32 = 131072; // d3d8-rust/src/types_amd64.rs // gen:cd1656f17100
pub const D3DPRASTERCAPS_DITHER: u32 = 1; // d3d8-rust/src/types_amd64.rs // gen:97b9a37ad621
pub const D3DPRASTERCAPS_FOGRANGE: u32 = 65536; // d3d8-rust/src/types_amd64.rs // gen:d058327abe96
pub const D3DPRASTERCAPS_FOGTABLE: u32 = 256; // d3d8-rust/src/types_amd64.rs // gen:966a1e8a91fd
pub const D3DPRASTERCAPS_FOGVERTEX: u32 = 128; // d3d8-rust/src/types_amd64.rs // gen:0cc33ab0c5ec
pub const D3DPRASTERCAPS_MIPMAPLODBIAS: u32 = 8192; // d3d8-rust/src/types_amd64.rs // gen:a459a89545e1
pub const D3DPRASTERCAPS_WBUFFER: u32 = 262144; // d3d8-rust/src/types_amd64.rs // gen:648442ec5f88
pub const D3DPRASTERCAPS_WFOG: u32 = 1048576; // d3d8-rust/src/types_amd64.rs // gen:0e0365d68446
pub const D3DPRASTERCAPS_ZBUFFERLESSHSR: u32 = 32768; // d3d8-rust/src/types_amd64.rs // gen:b9ac77f06f3f
pub const D3DPRASTERCAPS_ZFOG: u32 = 2097152; // d3d8-rust/src/types_amd64.rs // gen:fa6d76a47cc3
pub const D3DPRASTERCAPS_ZTEST: u32 = 16; // d3d8-rust/src/types_amd64.rs // gen:4279bbc5a994
pub const D3DPTEXTURECAPS_ALPHA: u32 = 4; // d3d8-rust/src/types_amd64.rs // gen:f00e03cfca05
pub const D3DPTEXTURECAPS_ALPHAPALETTE: u32 = 128; // d3d8-rust/src/types_amd64.rs // gen:95eb0089882f
pub const D3DPTEXTURECAPS_CUBEMAP: u32 = 2048; // d3d8-rust/src/types_amd64.rs // gen:fbe47e653c7b
pub const D3DPTEXTURECAPS_NONPOW2CONDITIONAL: u32 = 256; // d3d8-rust/src/types_amd64.rs // gen:87ad0e7e0ec1
pub const D3DPTEXTURECAPS_PERSPECTIVE: u32 = 1; // d3d8-rust/src/types_amd64.rs // gen:2a86da1abea0
pub const D3DPTEXTURECAPS_POW2: u32 = 2; // d3d8-rust/src/types_amd64.rs // gen:79d2e4cf7fc7
pub const D3DPTEXTURECAPS_PROJECTED: u32 = 1024; // d3d8-rust/src/types_amd64.rs // gen:bf0a9fe023f7
pub const D3DPTEXTURECAPS_SQUAREONLY: u32 = 32; // d3d8-rust/src/types_amd64.rs // gen:14bdf4788cfd
pub const D3DPTEXTURECAPS_TEXREPEATNOTSCALEDBYSIZE: u32 = 64; // d3d8-rust/src/types_amd64.rs // gen:39678f93ec6f
pub const D3DPTFILTERCAPS_MAGFANISOTROPIC: u32 = 67108864; // d3d8-rust/src/types_amd64.rs // gen:c76e52556a01
pub const D3DPTFILTERCAPS_MAGFLINEAR: u32 = 33554432; // d3d8-rust/src/types_amd64.rs // gen:f6b94f51a736
pub const D3DPTFILTERCAPS_MAGFPOINT: u32 = 16777216; // d3d8-rust/src/types_amd64.rs // gen:c8c7085540a6
pub const D3DPTFILTERCAPS_MINFANISOTROPIC: u32 = 1024; // d3d8-rust/src/types_amd64.rs // gen:6404aac36284
pub const D3DPTFILTERCAPS_MINFLINEAR: u32 = 512; // d3d8-rust/src/types_amd64.rs // gen:c52af99e8725
pub const D3DPTFILTERCAPS_MINFPOINT: u32 = 256; // d3d8-rust/src/types_amd64.rs // gen:aef74661271d
pub const D3DPTFILTERCAPS_MIPFLINEAR: u32 = 131072; // d3d8-rust/src/types_amd64.rs // gen:f4a39c0ec1d4
pub const D3DPTFILTERCAPS_MIPFPOINT: u32 = 65536; // d3d8-rust/src/types_amd64.rs // gen:0bf2138bc556
pub const D3DPT_LINELIST: _D3DPRIMITIVETYPE = _D3DPRIMITIVETYPE_D3DPT_LINELIST; // island enum // gen:079724b4a92e
pub const D3DPT_LINESTRIP: _D3DPRIMITIVETYPE = _D3DPRIMITIVETYPE_D3DPT_LINESTRIP; // island enum // gen:d52694065390
pub const D3DPT_POINTLIST: _D3DPRIMITIVETYPE = _D3DPRIMITIVETYPE_D3DPT_POINTLIST; // island enum // gen:05e9bada0d8c
pub const D3DPT_TRIANGLEFAN: _D3DPRIMITIVETYPE = _D3DPRIMITIVETYPE_D3DPT_TRIANGLEFAN; // island enum // gen:364b3fc70c95
pub const D3DPT_TRIANGLELIST: _D3DPRIMITIVETYPE = _D3DPRIMITIVETYPE_D3DPT_TRIANGLELIST; // island enum // gen:902c2df46fb7
pub const D3DPT_TRIANGLESTRIP: _D3DPRIMITIVETYPE = _D3DPRIMITIVETYPE_D3DPT_TRIANGLESTRIP; // island enum // gen:7a68725ac6b1
pub const D3DQUERYTYPE_MEMORYPRESSURE: _D3DQUERYTYPE = _D3DQUERYTYPE_D3DQUERYTYPE_MEMORYPRESSURE; // island enum // gen:79ed6c4d4a4b
pub const D3DQUERYTYPE_OCCLUSION: _D3DQUERYTYPE = _D3DQUERYTYPE_D3DQUERYTYPE_OCCLUSION; // island enum // gen:fbeb43f1b9af
pub const D3DQUERYTYPE_TIMESTAMPDISJOINT: _D3DQUERYTYPE = _D3DQUERYTYPE_D3DQUERYTYPE_TIMESTAMPDISJOINT; // island enum // gen:f50765b65c03
pub const D3DRS_POINTSIZE: _D3DRENDERSTATETYPE = _D3DRENDERSTATETYPE_D3DRS_POINTSIZE; // island enum // gen:9f2b832b8495
pub const D3DRTYPE_CUBETEXTURE: _D3DRESOURCETYPE = _D3DRESOURCETYPE_D3DRTYPE_CUBETEXTURE; // island enum // gen:41d45e50749e
pub const D3DRTYPE_INDEXBUFFER: _D3DRESOURCETYPE = _D3DRESOURCETYPE_D3DRTYPE_INDEXBUFFER; // island enum // gen:bd778b246623
pub const D3DRTYPE_SURFACE: _D3DRESOURCETYPE = _D3DRESOURCETYPE_D3DRTYPE_SURFACE; // island enum // gen:af78883c072b
pub const D3DRTYPE_TEXTURE: _D3DRESOURCETYPE = _D3DRESOURCETYPE_D3DRTYPE_TEXTURE; // island enum // gen:5bea2bb31114
pub const D3DRTYPE_VERTEXBUFFER: _D3DRESOURCETYPE = _D3DRESOURCETYPE_D3DRTYPE_VERTEXBUFFER; // island enum // gen:d30f023e8b2e
pub const D3DRTYPE_VOLUME: _D3DRESOURCETYPE = _D3DRESOURCETYPE_D3DRTYPE_VOLUME; // island enum // gen:567a56e9307d
pub const D3DRTYPE_VOLUMETEXTURE: _D3DRESOURCETYPE = _D3DRESOURCETYPE_D3DRTYPE_VOLUMETEXTURE; // island enum // gen:c581cfc04d87
pub const D3DSBT_ALL: _D3DSTATEBLOCKTYPE = _D3DSTATEBLOCKTYPE_D3DSBT_ALL; // island enum // gen:277d830bb7b7
pub const D3DSBT_PIXELSTATE: _D3DSTATEBLOCKTYPE = _D3DSTATEBLOCKTYPE_D3DSBT_PIXELSTATE; // island enum // gen:0c55033475dd
pub const D3DSBT_VERTEXSTATE: _D3DSTATEBLOCKTYPE = _D3DSTATEBLOCKTYPE_D3DSBT_VERTEXSTATE; // island enum // gen:94123895a665
pub const D3DSWAPEFFECT_COPY: _D3DSWAPEFFECT = _D3DSWAPEFFECT_D3DSWAPEFFECT_COPY; // island enum // gen:8612e75ea134
pub const D3DSWAPEFFECT_DISCARD: _D3DSWAPEFFECT = _D3DSWAPEFFECT_D3DSWAPEFFECT_DISCARD; // island enum // gen:b44268d22891
pub const D3DSWAPEFFECT_FLIP: _D3DSWAPEFFECT = _D3DSWAPEFFECT_D3DSWAPEFFECT_FLIP; // island enum // gen:987ef8830033
pub const D3DSWAPEFFECT_FLIPEX: _D3DSWAPEFFECT = _D3DSWAPEFFECT_D3DSWAPEFFECT_FLIPEX; // island enum // gen:6dee57c1427e
pub const D3DSWAPEFFECT_OVERLAY: _D3DSWAPEFFECT = _D3DSWAPEFFECT_D3DSWAPEFFECT_OVERLAY; // island enum // gen:c0afe673e79c
pub const D3DTEXF_LINEAR: _D3DTEXTUREFILTERTYPE = _D3DTEXTUREFILTERTYPE_D3DTEXF_LINEAR; // island enum // gen:168c310ef972
pub const D3DTEXF_NONE: _D3DTEXTUREFILTERTYPE = _D3DTEXTUREFILTERTYPE_D3DTEXF_NONE; // island enum // gen:1c8194662eeb
pub const D3DVTXPCAPS_DIRECTIONALLIGHTS: u32 = 8; // d3d8-rust/src/prelude.rs // gen:9ddec4c6330e
pub const D3DVTXPCAPS_LOCALVIEWER: u32 = 32; // d3d8-rust/src/prelude.rs // gen:8e531f5f4f1d
pub const D3DVTXPCAPS_MATERIALSOURCE7: u32 = 2; // d3d8-rust/src/prelude.rs // gen:3b735e0d3a96
pub const D3DVTXPCAPS_POSITIONALLIGHTS: u32 = 16; // d3d8-rust/src/prelude.rs // gen:dbafbfdfd55d
pub const D3DVTXPCAPS_TEXGEN: u32 = 1; // d3d8-rust/src/prelude.rs // gen:058239aa9420
pub const WINED3DFMT_A8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_A8_UNORM; // island enum // gen:b852344850b7
pub const WINED3DFMT_B10G10R10A2_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_B10G10R10A2_UNORM; // island enum // gen:fd65f7c0562e
pub const WINED3DFMT_B2G3R3A8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_B2G3R3A8_UNORM; // island enum // gen:2a968c2ee686
pub const WINED3DFMT_B2G3R3_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_B2G3R3_UNORM; // island enum // gen:4fef2a637c7d
pub const WINED3DFMT_B4G4R4A4_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_B4G4R4A4_UNORM; // island enum // gen:c77c181e2f2d
pub const WINED3DFMT_B4G4R4X4_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_B4G4R4X4_UNORM; // island enum // gen:fb254e5113cc
pub const WINED3DFMT_B5G5R5A1_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_B5G5R5A1_UNORM; // island enum // gen:ce8c178ac58e
pub const WINED3DFMT_B5G5R5X1_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_B5G5R5X1_UNORM; // island enum // gen:f97d07713986
pub const WINED3DFMT_B5G6R5_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_B5G6R5_UNORM; // island enum // gen:4b51b8db5a0e
pub const WINED3DFMT_B8G8R8A8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_B8G8R8A8_UNORM; // island enum // gen:2293a1e3fc0b
pub const WINED3DFMT_B8G8R8X8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_B8G8R8X8_UNORM; // island enum // gen:5c3514c0dc29
pub const WINED3DFMT_B8G8R8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_B8G8R8_UNORM; // island enum // gen:26e4d489e6fb
pub const WINED3DFMT_D16_LOCKABLE: wined3d_format_id = wined3d_format_id_WINED3DFMT_D16_LOCKABLE; // island enum // gen:cdf7ada8d3f0
pub const WINED3DFMT_D16_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_D16_UNORM; // island enum // gen:9c5a906a0784
pub const WINED3DFMT_D24_UNORM_S8_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_D24_UNORM_S8_UINT; // island enum // gen:9527094adfe6
pub const WINED3DFMT_D32_FLOAT: wined3d_format_id = wined3d_format_id_WINED3DFMT_D32_FLOAT; // island enum // gen:ed547dfdd88b
pub const WINED3DFMT_D32_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_D32_UNORM; // island enum // gen:fac0db7c6301
pub const WINED3DFMT_DF16: wined3d_format_id = wined3d_format_id_WINED3DFMT_DF16; // island enum // gen:a4bfb44af8fa
pub const WINED3DFMT_DF24: wined3d_format_id = wined3d_format_id_WINED3DFMT_DF24; // island enum // gen:a05a4d31466c
pub const WINED3DFMT_INTZ: wined3d_format_id = wined3d_format_id_WINED3DFMT_INTZ; // island enum // gen:315ea9570ef9
pub const WINED3DFMT_L16_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_L16_UNORM; // island enum // gen:b7207b17ac30
pub const WINED3DFMT_L4A4_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_L4A4_UNORM; // island enum // gen:7c9feb6fdd9c
pub const WINED3DFMT_L8A8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_L8A8_UNORM; // island enum // gen:621b76d8945e
pub const WINED3DFMT_L8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_L8_UNORM; // island enum // gen:385347c438e3
pub const WINED3DFMT_P8_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_P8_UINT; // island enum // gen:08be2f398d7b
pub const WINED3DFMT_P8_UINT_A8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_P8_UINT_A8_UNORM; // island enum // gen:ee16b1321b40
pub const WINED3DFMT_R10G10B10A2_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R10G10B10A2_UNORM; // island enum // gen:1c4034896072
pub const WINED3DFMT_R10G10B10X2_SNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R10G10B10X2_SNORM; // island enum // gen:fa5916576b7d
pub const WINED3DFMT_R10G10B10X2_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R10G10B10X2_UINT; // island enum // gen:e9b2e89b1d31
pub const WINED3DFMT_R10G10B10_SNORM_A2_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R10G10B10_SNORM_A2_UNORM; // island enum // gen:408936d9d5be
pub const WINED3DFMT_R16G16B16A16_FLOAT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16B16A16_FLOAT; // island enum // gen:7df2955d6781
pub const WINED3DFMT_R16G16B16A16_SINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16B16A16_SINT; // island enum // gen:2178584fcdfb
pub const WINED3DFMT_R16G16B16A16_SNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16B16A16_SNORM; // island enum // gen:7ad3ef834bb6
pub const WINED3DFMT_R16G16B16A16_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16B16A16_UNORM; // island enum // gen:a32ff2d7dffc
pub const WINED3DFMT_R16G16_FLOAT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16_FLOAT; // island enum // gen:df79272fcb2e
pub const WINED3DFMT_R16G16_SINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16_SINT; // island enum // gen:5aee36a3ea1d
pub const WINED3DFMT_R16G16_SNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16_SNORM; // island enum // gen:c6640d1032ad
pub const WINED3DFMT_R16G16_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16_UNORM; // island enum // gen:db785a6d61b2
pub const WINED3DFMT_R16_FLOAT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16_FLOAT; // island enum // gen:5700c3bf32d4
pub const WINED3DFMT_R16_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16_UINT; // island enum // gen:4ce044980323
pub const WINED3DFMT_R32G32B32A32_FLOAT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R32G32B32A32_FLOAT; // island enum // gen:df2bebda145f
pub const WINED3DFMT_R32G32B32_FLOAT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R32G32B32_FLOAT; // island enum // gen:67dc0b8b4a65
pub const WINED3DFMT_R32G32_FLOAT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R32G32_FLOAT; // island enum // gen:1183f2e5a833
pub const WINED3DFMT_R32_FLOAT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R32_FLOAT; // island enum // gen:59bc4b0e8aa7
pub const WINED3DFMT_R32_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R32_UINT; // island enum // gen:3e1ccdac6f8d
pub const WINED3DFMT_R5G5_SNORM_L6_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R5G5_SNORM_L6_UNORM; // island enum // gen:86cc0bb8f6b0
pub const WINED3DFMT_R8G8B8A8_SNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8B8A8_SNORM; // island enum // gen:69412f5ca2db
pub const WINED3DFMT_R8G8B8A8_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8B8A8_UINT; // island enum // gen:1d31fd9bb937
pub const WINED3DFMT_R8G8B8A8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8B8A8_UNORM; // island enum // gen:721264dd8e5b
pub const WINED3DFMT_R8G8B8X8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8B8X8_UNORM; // island enum // gen:b63b4e348a0e
pub const WINED3DFMT_R8G8_SNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8_SNORM; // island enum // gen:2315347f9119
pub const WINED3DFMT_R8G8_SNORM_Cx: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8_SNORM_Cx; // island enum // gen:cb32dcaeb972
pub const WINED3DFMT_R8G8_SNORM_L8X8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8_SNORM_L8X8_UNORM; // island enum // gen:d3e02c96ba02
pub const WINED3DFMT_S1_UINT_D15_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_S1_UINT_D15_UNORM; // island enum // gen:a396b379f935
pub const WINED3DFMT_S4X4_UINT_D24_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_S4X4_UINT_D24_UNORM; // island enum // gen:6fe6c428ea21
pub const WINED3DFMT_S8_UINT_D24_FLOAT: wined3d_format_id = wined3d_format_id_WINED3DFMT_S8_UINT_D24_FLOAT; // island enum // gen:ff71d2323417
pub const WINED3DFMT_UNKNOWN: wined3d_format_id = wined3d_format_id_WINED3DFMT_UNKNOWN; // island enum // gen:93d9074cc710
pub const WINED3DFMT_X8D24_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_X8D24_UNORM; // island enum // gen:b81fd5c118b7
pub const WINED3D_DEVICE_TYPE_HAL: wined3d_device_type = wined3d_device_type_WINED3D_DEVICE_TYPE_HAL; // island enum // gen:d73e66f36b9d
pub const WINED3D_FEATURE_LEVEL_5: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_5; // island enum // gen:f9c53f3d677f
pub const WINED3D_FEATURE_LEVEL_6: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_6; // island enum // gen:e970439b7cb3
pub const WINED3D_FEATURE_LEVEL_7: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_7; // island enum // gen:03cd701a1439
pub const WINED3D_FEATURE_LEVEL_8: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_8; // island enum // gen:396b2980b189
pub const WINED3D_FEATURE_LEVEL_9_1: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_9_1; // island enum // gen:5d3d08e56b20
pub const WINED3D_FEATURE_LEVEL_9_2: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_9_2; // island enum // gen:a1c2cdf52b31
pub const WINED3D_FEATURE_LEVEL_9_3: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_9_3; // island enum // gen:1666a64f5f90
pub const WINED3D_INPUT_PER_VERTEX_DATA: wined3d_input_classification = wined3d_input_classification_WINED3D_INPUT_PER_VERTEX_DATA; // island enum // gen:e87a81739d3a
pub const WINED3D_MULTISAMPLE_NONE: wined3d_multisample_type = wined3d_multisample_type_WINED3D_MULTISAMPLE_NONE; // island enum // gen:bfa66121fcaa
pub const WINED3D_MULTISAMPLE_NON_MASKABLE: wined3d_multisample_type = wined3d_multisample_type_WINED3D_MULTISAMPLE_NON_MASKABLE; // island enum // gen:d5b0152c15a3
pub const WINED3D_QUERY_TYPE_TIMESTAMP_DISJOINT: wined3d_query_type = wined3d_query_type_WINED3D_QUERY_TYPE_TIMESTAMP_DISJOINT; // island enum // gen:4591a4c62356
pub const WINED3D_RS_ZENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_ZENABLE; // island enum // gen:f8bcd5a86ca6
pub const WINED3D_RTYPE_BUFFER: wined3d_resource_type = wined3d_resource_type_WINED3D_RTYPE_BUFFER; // island enum // gen:8443940bf9c4
pub const WINED3D_RTYPE_NONE: wined3d_resource_type = wined3d_resource_type_WINED3D_RTYPE_NONE; // island enum // gen:20c459743c49
pub const WINED3D_RTYPE_TEXTURE_2D: wined3d_resource_type = wined3d_resource_type_WINED3D_RTYPE_TEXTURE_2D; // island enum // gen:de40c3024d68
pub const WINED3D_RTYPE_TEXTURE_3D: wined3d_resource_type = wined3d_resource_type_WINED3D_RTYPE_TEXTURE_3D; // island enum // gen:7752a0d240c9
pub const WINED3D_SBT_PRIMARY: wined3d_stateblock_type = wined3d_stateblock_type_WINED3D_SBT_PRIMARY; // island enum // gen:a4ee8d8e4d8b
pub const WINED3D_SBT_RECORDED: wined3d_stateblock_type = wined3d_stateblock_type_WINED3D_SBT_RECORDED; // island enum // gen:a3a30ecb7bc4
pub const WINED3D_SCANLINE_ORDERING_UNKNOWN: wined3d_scanline_ordering = wined3d_scanline_ordering_WINED3D_SCANLINE_ORDERING_UNKNOWN; // island enum // gen:7774b8a98afe
pub const WINED3D_SWAP_EFFECT_COPY: wined3d_swap_effect = wined3d_swap_effect_WINED3D_SWAP_EFFECT_COPY; // island enum // gen:c4b8f5e74118
pub const WINED3D_SWAP_EFFECT_DISCARD: wined3d_swap_effect = wined3d_swap_effect_WINED3D_SWAP_EFFECT_DISCARD; // island enum // gen:7f0e42311775
pub const WINED3D_SWAP_EFFECT_FLIP_SEQUENTIAL: wined3d_swap_effect = wined3d_swap_effect_WINED3D_SWAP_EFFECT_FLIP_SEQUENTIAL; // island enum // gen:f93ab0e8ad9c
pub const WINED3D_SWAP_EFFECT_OVERLAY: wined3d_swap_effect = wined3d_swap_effect_WINED3D_SWAP_EFFECT_OVERLAY; // island enum // gen:3ac9c3887d40
pub const WINED3D_SWAP_EFFECT_SEQUENTIAL: wined3d_swap_effect = wined3d_swap_effect_WINED3D_SWAP_EFFECT_SEQUENTIAL; // island enum // gen:5af9d459627d
pub const WINED3D_SWAP_INTERVAL_DEFAULT: wined3d_swap_interval = wined3d_swap_interval_WINED3D_SWAP_INTERVAL_DEFAULT; // island enum // gen:a656df69eabe
pub const WINED3D_SWAP_INTERVAL_FOUR: wined3d_swap_interval = wined3d_swap_interval_WINED3D_SWAP_INTERVAL_FOUR; // island enum // gen:c69812d048da
pub const WINED3D_SWAP_INTERVAL_IMMEDIATE: wined3d_swap_interval = wined3d_swap_interval_WINED3D_SWAP_INTERVAL_IMMEDIATE; // island enum // gen:ad9b6e2af6af
pub const WINED3D_SWAP_INTERVAL_ONE: wined3d_swap_interval = wined3d_swap_interval_WINED3D_SWAP_INTERVAL_ONE; // island enum // gen:e2ef53ab34e2
pub const WINED3D_SWAP_INTERVAL_THREE: wined3d_swap_interval = wined3d_swap_interval_WINED3D_SWAP_INTERVAL_THREE; // island enum // gen:2210973852c0
pub const WINED3D_SWAP_INTERVAL_TWO: wined3d_swap_interval = wined3d_swap_interval_WINED3D_SWAP_INTERVAL_TWO; // island enum // gen:5f33c15d0b73
pub const WINED3D_TEXF_POINT: wined3d_texture_filter_type = wined3d_texture_filter_type_WINED3D_TEXF_POINT; // island enum // gen:db8018ba15af
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
pub const WINED3D_TSS_INVALID: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_INVALID; // island enum // gen:917b0a20b2b3
pub const WINED3D_TSS_RESULT_ARG: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_RESULT_ARG; // island enum // gen:c25fee23b0cb
pub const WINED3D_TSS_TEXCOORD_INDEX: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_TEXCOORD_INDEX; // island enum // gen:8e80cbf7e4eb
pub const WINED3D_TSS_TEXTURE_TRANSFORM_FLAGS: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_TEXTURE_TRANSFORM_FLAGS; // island enum // gen:f9828e809b53
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3; // activeds-rust/src/prelude.rs // gen:845fcd2ce5ae
extern "C" {
    pub fn calloc(arg1: ::core::ffi::c_uint, arg2: ::core::ffi::c_uint) -> *mut ::core::ffi::c_void; // msvcrt-sys/src/bindgen_i386.rs // gen:d3193c100ce5
    pub fn free(p: *mut c_void); // activeds-rust/src/prelude.rs // gen:0ad441c3c230
    pub fn isprint(c: c_int) -> c_int; // d3d8-rust/src/prelude.rs // gen:90820586bb35
    pub fn malloc(size: usize) -> *mut c_void; // activeds-rust/src/prelude.rs // gen:86a212c6fd14
    pub fn realloc(p: *mut c_void, size: usize) -> *mut c_void; // activeds-rust/src/prelude.rs // gen:ce18447a16eb
}
extern "system" {
    pub fn SetRect( arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:9d720f054480
    pub fn WideCharToMultiByte( arg1: UINT, arg2: DWORD, arg3: LPCWSTR, arg4: ::core::ffi::c_int, arg5: LPSTR, arg6: ::core::ffi::c_int, arg7: LPCSTR, arg8: LPBOOL, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs // gen:0e129f7cfff2
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbLengthW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};
pub use winelist_rust::{list_add_after, list_add_before, list_add_head, list_add_tail, list_count, list_empty, list_head, list_init, list_move_head, list_move_tail, list_next, list_prev, list_remove, list_tail};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const D3DCAPS2_CANAUTOGENMIPMAP: i32 = 0x40000000; // clang, device.c
pub const D3DCAPS2_CANCALIBRATEGAMMA: i32 = 0x100000; // clang, device.c
pub const D3DCAPS2_CANMANAGERESOURCE: i32 = 0x10000000; // clang, device.c
pub const D3DCAPS2_DYNAMICTEXTURES: i32 = 0x20000000; // clang, device.c
pub const D3DCAPS2_FULLSCREENGAMMA: i32 = 0x20000; // clang, device.c
pub const D3DCAPS2_RESERVED: i32 = 0x2000000; // clang, device.c
pub const D3DCREATE_ADAPTERGROUP_DEVICE: i32 = 0x200; // clang, device.c
pub const D3DCREATE_FPU_PRESERVE: i32 = 2; // clang, device.c
pub const D3DCREATE_NOWINDOWCHANGES: i32 = 0x800; // clang, device.c
pub const D3DDEVCAPS2_ADAPTIVETESSNPATCH: i32 = 8; // clang, device.c
pub const D3DDEVCAPS2_ADAPTIVETESSRTPATCH: i32 = 4; // clang, device.c
pub const D3DDEVCAPS2_CAN_STRETCHRECT_FROM_TEXTURES: i32 = 0x10; // clang, device.c
pub const D3DDEVCAPS2_DMAPNPATCH: i32 = 2; // clang, device.c
pub const D3DDEVCAPS2_PRESAMPLEDDMAPNPATCH: i32 = 0x20; // clang, device.c
pub const D3DDEVCAPS2_STREAMOFFSET: i32 = 1; // clang, device.c
pub const D3DDEVCAPS2_VERTEXELEMENTSCANSHARESTREAMOFFSET: i32 = 0x40; // clang, device.c
pub const D3DFMT_RESZ: DWORD = 0x5A534552; // clang, directx.c
pub const D3DPRASTERCAPS_COLORPERSPECTIVE: i32 = 0x400000; // clang, device.c
pub const D3DPRASTERCAPS_DEPTHBIAS: i32 = 0x4000000; // clang, device.c
pub const D3DPRASTERCAPS_MULTISAMPLE_TOGGLE: i32 = 0x8000000; // clang, device.c
pub const D3DPRASTERCAPS_SCISSORTEST: i32 = 0x1000000; // clang, device.c
pub const D3DPRASTERCAPS_SLOPESCALEDEPTHBIAS: i32 = 0x2000000; // clang, device.c
pub const D3DPTEXTURECAPS_CUBEMAP_POW2: i32 = 0x20000; // clang, device.c
pub const D3DPTEXTURECAPS_MIPCUBEMAP: i32 = 0x10000; // clang, device.c
pub const D3DPTEXTURECAPS_MIPMAP: i32 = 0x4000; // clang, device.c
pub const D3DPTEXTURECAPS_MIPVOLUMEMAP: i32 = 0x8000; // clang, device.c
pub const D3DPTEXTURECAPS_NOPROJECTEDBUMPENV: i32 = 0x200000; // clang, device.c
pub const D3DPTEXTURECAPS_VOLUMEMAP: i32 = 0x2000; // clang, device.c
pub const D3DPTEXTURECAPS_VOLUMEMAP_POW2: i32 = 0x40000; // clang, device.c
pub const D3DPTFILTERCAPS_MAGFGAUSSIANQUAD: i32 = 0x10000000; // clang, device.c
pub const D3DPTFILTERCAPS_MAGFPYRAMIDALQUAD: i32 = 0x8000000; // clang, device.c
pub const D3DPTFILTERCAPS_MINFGAUSSIANQUAD: i32 = 0x1000; // clang, device.c
pub const D3DPTFILTERCAPS_MINFPYRAMIDALQUAD: i32 = 0x800; // clang, device.c
pub const D3DUSAGE_AUTOGENMIPMAP: i32 = 0x400; // clang, device.c
pub const D3DUSAGE_DEPTHSTENCIL: i32 = 2; // clang, buffer.c
pub const D3DUSAGE_DYNAMIC: i32 = 0x200; // clang, buffer.c
pub const D3DUSAGE_RENDERTARGET: i32 = 1; // clang, buffer.c
pub const D3DUSAGE_WRITEONLY: i32 = 8; // clang, device.c
pub const D3DVTXPCAPS_NO_TEXGEN_NONLOCALVIEWER: i32 = 0x200; // clang, device.c
pub const D3DVTXPCAPS_TEXGEN_SPHEREMAP: i32 = 0x100; // clang, device.c
pub const D3DVTXPCAPS_TWEENING: i32 = 0x40; // clang, device.c
pub const WINED3D_VERTEX_SAMPLER_OFFSET: i32 = 0x10; // clang, device.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn ros_dbg_log(
        cls: __wine_debug_class,
        ch: *mut __wine_debug_channel,
        file: *const ::core::ffi::c_char,
        func: *const ::core::ffi::c_char,
        line: ::core::ffi::c_int,
        format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // buffer.c
    pub fn wined3d_adapter_get_identifier(
        adapter: *const wined3d_adapter,
        flags: uint32_t,
        identifier: *mut wined3d_adapter_identifier,
    ) -> HRESULT; // directx.c
    pub fn wined3d_adapter_get_output(
        adapter: *const wined3d_adapter,
        idx: ::core::ffi::c_uint,
    ) -> *mut wined3d_output; // directx.c
    pub fn wined3d_adapter_get_output_count(adapter: *const wined3d_adapter) -> ::core::ffi::c_uint; // device.c
    pub fn wined3d_buffer_create(
        device: *mut wined3d_device,
        desc: *const wined3d_buffer_desc,
        data: *const wined3d_sub_resource_data,
        parent: *mut ::core::ffi::c_void,
        parent_ops: *const wined3d_parent_ops,
        buffer: *mut *mut wined3d_buffer,
    ) -> HRESULT; // buffer.c
    pub fn wined3d_buffer_get_resource(buffer: *mut wined3d_buffer) -> *mut wined3d_resource; // buffer.c
    pub fn wined3d_check_depth_stencil_match(
        adapter: *const wined3d_adapter,
        device_type: wined3d_device_type,
        adapter_format_id: wined3d_format_id,
        render_target_format_id: wined3d_format_id,
        depth_stencil_format_id: wined3d_format_id,
    ) -> HRESULT; // directx.c
    pub fn wined3d_check_device_format(
        wined3d: *const wined3d,
        adapter: *const wined3d_adapter,
        device_type: wined3d_device_type,
        adapter_format_id: wined3d_format_id,
        usage: uint32_t,
        bind_flags: ::core::ffi::c_uint,
        resource_type: wined3d_resource_type,
        check_format_id: wined3d_format_id,
    ) -> HRESULT; // directx.c
    pub fn wined3d_check_device_multisample_type(
        adapter: *const wined3d_adapter,
        device_type: wined3d_device_type,
        surface_format_id: wined3d_format_id,
        windowed: BOOL,
        multisample_type: wined3d_multisample_type,
        quality_levels: *mut ::core::ffi::c_uint,
    ) -> HRESULT; // directx.c
    pub fn wined3d_device_context_blt(
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
    ) -> HRESULT; // device.c
    pub fn wined3d_device_context_copy_sub_resource_region(
        context: *mut wined3d_device_context,
        dst_resource: *mut wined3d_resource,
        dst_sub_resource_idx: ::core::ffi::c_uint,
        dst_x: ::core::ffi::c_uint,
        dst_y: ::core::ffi::c_uint,
        dst_z: ::core::ffi::c_uint,
        src_resource: *mut wined3d_resource,
        src_sub_resource_idx: ::core::ffi::c_uint,
        src_box: *const wined3d_box,
        flags: ::core::ffi::c_uint,
    ) -> HRESULT; // device.c
    pub fn wined3d_device_context_generate_mipmaps(
        context: *mut wined3d_device_context,
        view: *mut wined3d_shader_resource_view,
    ); // texture.c
    pub fn wined3d_device_context_resolve_sub_resource(
        context: *mut wined3d_device_context,
        dst_resource: *mut wined3d_resource,
        dst_sub_resource_idx: ::core::ffi::c_uint,
        src_resource: *mut wined3d_resource,
        src_sub_resource_idx: ::core::ffi::c_uint,
        format_id: wined3d_format_id,
    ); // device.c
    pub fn wined3d_device_create(
        wined3d: *mut wined3d,
        adapter: *mut wined3d_adapter,
        device_type: wined3d_device_type,
        focus_window: HWND,
        behaviour_flags: ::core::ffi::c_uint,
        surface_alignment: BYTE,
        feature_levels: *const wined3d_feature_level,
        feature_level_count: ::core::ffi::c_uint,
        device_parent: *mut wined3d_device_parent,
        device: *mut *mut wined3d_device,
    ) -> HRESULT; // device.c
    pub fn wined3d_device_reset(
        device: *mut wined3d_device,
        swapchain_desc: *const wined3d_swapchain_desc,
        mode: *const wined3d_display_mode,
        callback: wined3d_device_reset_cb,
        reset_state: BOOL,
    ) -> HRESULT; // device.c
    pub fn wined3d_get_adapter(
        wined3d: *const wined3d,
        idx: ::core::ffi::c_uint,
    ) -> *mut wined3d_adapter; // directx.c
    pub fn wined3d_get_device_caps(
        adapter: *const wined3d_adapter,
        device_type: wined3d_device_type,
        caps: *mut wined3d_caps,
    ) -> HRESULT; // device.c
    pub fn wined3d_output_get_adapter(output: *const wined3d_output) -> *mut wined3d_adapter; // device.c
    pub fn wined3d_rendertarget_view_get_resource(
        view: *const wined3d_rendertarget_view,
    ) -> *mut wined3d_resource; // device.c
    pub fn wined3d_resource_get_desc(
        resource: *const wined3d_resource,
        desc: *mut wined3d_resource_desc,
    ); // buffer.c
    pub fn wined3d_resource_get_parent(resource: *const wined3d_resource) -> *mut ::core::ffi::c_void; // device.c
    pub fn wined3d_resource_get_priority(resource: *const wined3d_resource) -> ::core::ffi::c_uint; // buffer.c
    pub fn wined3d_resource_map(
        resource: *mut wined3d_resource,
        sub_resource_idx: ::core::ffi::c_uint,
        map_desc: *mut wined3d_map_desc,
        box_0: *const wined3d_box,
        flags: uint32_t,
    ) -> HRESULT; // buffer.c
    pub fn wined3d_resource_preload(resource: *mut wined3d_resource); // buffer.c
    pub fn wined3d_resource_set_priority(
        resource: *mut wined3d_resource,
        priority: ::core::ffi::c_uint,
    ) -> ::core::ffi::c_uint; // buffer.c
    pub fn wined3d_resource_unmap(
        resource: *mut wined3d_resource,
        sub_resource_idx: ::core::ffi::c_uint,
    ) -> HRESULT; // buffer.c
    pub fn wined3d_texture_acquire_identity_srv(
        texture: *mut wined3d_texture,
    ) -> *mut wined3d_shader_resource_view; // texture.c
    pub fn wined3d_texture_create(
        device: *mut wined3d_device,
        desc: *const wined3d_resource_desc,
        layer_count: UINT,
        level_count: ::core::ffi::c_uint,
        flags: uint32_t,
        data: *const wined3d_sub_resource_data,
        parent: *mut ::core::ffi::c_void,
        parent_ops: *const wined3d_parent_ops,
        texture: *mut *mut wined3d_texture,
    ) -> HRESULT; // device.c
    pub fn wined3d_texture_from_resource(resource: *mut wined3d_resource) -> *mut wined3d_texture; // device.c
    pub fn wined3d_texture_get_resource(texture: *mut wined3d_texture) -> *mut wined3d_resource; // device.c
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
pub static mut __wine_dbch_d3d9: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x64\x33\x64\x39\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_d3d9 };
// ==== end wine debug channels ====

// ==== port resolve: UNICODE TCHAR family (generated) ====
pub type TCHAR = WCHAR;
pub type PTCHAR = *mut WCHAR;
pub type TBYTE = WCHAR;
pub type PTBYTE = *mut WCHAR;
pub type LPTSTR = LPWSTR;
pub type LPCTSTR = LPCWSTR;
pub type PTSTR = LPWSTR;
pub type PCTSTR = LPCWSTR;
pub type LPTCH = LPWSTR;
pub type PTCH = LPWSTR;
// ==== end UNICODE TCHAR family ====

// ==== port resolve: INITGUID storage (generated) ====
#[no_mangle]
pub static IID_IDirect3D9Ex: GUID = GUID { Data1: 0x02177241, Data2: 0x69fc, Data3: 0x400c, Data4: [0x8f, 0xf1, 0x93, 0xa4, 0x4d, 0xf6, 0x86, 0x1d] };
#[no_mangle]
pub static IID_IDirect3DDevice9Ex: GUID = GUID { Data1: 0xb18b10ce, Data2: 0x2649, Data3: 0x405a, Data4: [0x87, 0xf, 0x95, 0xf7, 0x77, 0xd4, 0x31, 0x3a] };
#[no_mangle]
pub static IID_IDirect3DDevice9On12: GUID = GUID { Data1: 0xe7fda234, Data2: 0xb589, Data3: 0x4049, Data4: [0x94, 0x0d, 0x88, 0x78, 0x97, 0x75, 0x31, 0xc8] };
#[no_mangle]
pub static IID_IDirect3DSwapChain9Ex: GUID = GUID { Data1: 0x91886caf, Data2: 0x1c3d, Data3: 0x4d2e, Data4: [0xa0, 0xab, 0x3e, 0x4c, 0x7d, 0x8d, 0x33, 0x3] };
// ==== end INITGUID storage ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::d3d9::device::d3d9_caps_from_wined3dcaps;
pub use crate::d3d9::directx::d3d9_init;
pub use crate::d3d9::d3d9_main::d3d9_resource_cleanup;
pub use crate::d3d9::d3d9_main::d3d9_resource_free_private_data;
pub use crate::d3d9::d3d9_main::d3d9_resource_get_private_data;
pub use crate::d3d9::d3d9_main::d3d9_resource_init;
pub use crate::d3d9::d3d9_main::d3d9_resource_set_private_data;
pub use crate::d3d9::surface::d3d9_surface_acquire_rendertarget_view;
pub use crate::d3d9::surface::d3d9_surface_create;
pub use crate::d3d9::surface::d3d9_surface_get_device;
pub use crate::d3d9::surface::d3d9_surface_release_rendertarget_view;
pub use crate::d3d9::swapchain::d3d9_swapchain_create;
pub use crate::d3d9::texture::d3d9_texture_2d_init;
pub use crate::d3d9::texture::d3d9_texture_3d_init;
pub use crate::d3d9::texture::d3d9_texture_cube_init;
pub use crate::d3d9::texture::d3d9_texture_flag_auto_gen_mipmap;
pub use crate::d3d9::texture::d3d9_texture_gen_auto_mipmap;
pub use crate::d3d9::vertexdeclaration::d3d9_vertex_declaration_create;
pub use crate::d3d9::device::d3dformat_from_wined3dformat;
pub use crate::d3d9::device::device_init;
pub use crate::d3d9::buffer::indexbuffer_init;
pub use crate::d3d9::device::is_gdi_compat_wined3dformat;
pub use crate::d3d9::shader::pixelshader_init;
pub use crate::d3d9::device::present_parameters_from_wined3d_swapchain_desc;
pub use crate::d3d9::query::query_init;
pub use crate::d3d9::stateblock::stateblock_init;
pub use crate::d3d9::texture::unsafe_impl_from_IDirect3DBaseTexture9;
pub use crate::d3d9::buffer::unsafe_impl_from_IDirect3DIndexBuffer9;
pub use crate::d3d9::shader::unsafe_impl_from_IDirect3DPixelShader9;
pub use crate::d3d9::surface::unsafe_impl_from_IDirect3DSurface9;
pub use crate::d3d9::buffer::unsafe_impl_from_IDirect3DVertexBuffer9;
pub use crate::d3d9::vertexdeclaration::unsafe_impl_from_IDirect3DVertexDeclaration9;
pub use crate::d3d9::shader::unsafe_impl_from_IDirect3DVertexShader9;
pub use crate::d3d9::vertexdeclaration::vdecl_convert_fvf;
pub use crate::d3d9::buffer::vertexbuffer_init;
pub use crate::d3d9::shader::vertexshader_init;
pub use crate::d3d9::volume::volume_init;
pub use crate::d3d9::device::wined3dformat_from_d3dformat;
pub use crate::d3d9::device::wined3dmapflags_from_d3dmapflags;
// ==== end cross-TU definitions ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! D3DPS_VERSION { ($p0:expr, $p1:expr $(,)?) => { ((0xFFFF0000u32 | ($p0 << 8)) | $p1) }; } // gen:4b6922084b76
#[macro_export] macro_rules! D3DVS_VERSION { ($p0:expr, $p1:expr $(,)?) => { ((0xFFFE0000u32 | ($p0 << 8)) | $p1) }; } // gen:143c24ebf58c
#[macro_export] macro_rules! max { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 > $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; } // gen:29bb71d79dfd
#[macro_export] macro_rules! min { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 < $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; } // gen:fb1d8a90940a
pub const D3DERR_DEVICELOST: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2152) as c_uint))) as HRESULT); // gen:186ac49170f0
pub const D3DERR_DEVICENOTRESET: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2153) as c_uint))) as HRESULT); // gen:22a2962c5cdd
pub const D3DERR_DRIVERINTERNALERROR: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2087) as c_uint))) as HRESULT); // gen:13ac26757dd1
pub const D3DERR_INVALIDCALL: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2156) as c_uint))) as HRESULT); // gen:fa040802d037
pub const D3DERR_MOREDATA: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2151) as c_uint))) as HRESULT); // gen:23e0a265c5d9
pub const D3DERR_NOTAVAILABLE: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2154) as c_uint))) as HRESULT); // gen:2085985dde39
pub const D3DERR_NOTFOUND: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2150) as c_uint))) as HRESULT); // gen:fcbd44af2cdd
pub const D3DERR_OUTOFVIDEOMEMORY: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((380) as c_uint))) as HRESULT); // gen:6725e1216541
pub const D3D_OK: HRESULT = ((0) as HRESULT); // gen:f736e8837e47
pub const E_FAIL: HRESULT = ((0x80004005u32) as HRESULT); // gen:1275c1cb9e8d
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT); // gen:cd8a52171603
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT); // gen:80daa5b2ffa6
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT); // gen:cbf314acc21c
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT); // gen:98bee37aa995
#[inline] // gen:f97f25f83615
pub unsafe fn IDirect3D9Ex_AddRef(This: *mut IDirect3D9Ex) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:d01f265a4565
pub unsafe fn IDirect3D9Ex_QueryInterface(This: *mut IDirect3D9Ex, iid: *const IID, out: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, iid, out) } // widl C wrapper
#[inline] // gen:e4ae303eddc8
pub unsafe fn IDirect3D9Ex_Release(This: *mut IDirect3D9Ex) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:174a478bd79e
pub unsafe fn IDirect3D9_GetAdapterDisplayMode(This: *mut IDirect3D9, Adapter: UINT, pMode: *mut D3DDISPLAYMODE) -> HRESULT { ((*(*This).lpVtbl).GetAdapterDisplayMode.unwrap_unchecked())(This, Adapter, pMode) } // widl C wrapper
#[inline] // gen:a31a6d041304
pub unsafe fn IDirect3DBaseTexture9_AddRef(This: *mut IDirect3DBaseTexture9) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:2503be5ae48a
pub unsafe fn IDirect3DBaseTexture9_GetDevice(This: *mut IDirect3DBaseTexture9, ppDevice: *mut *mut IDirect3DDevice9) -> HRESULT { ((*(*This).lpVtbl).GetDevice.unwrap_unchecked())(This, ppDevice) } // widl C wrapper
#[inline] // gen:3bdb70c8a3f1
pub unsafe fn IDirect3DBaseTexture9_GetType(This: *mut IDirect3DBaseTexture9) -> D3DRESOURCETYPE { ((*(*This).lpVtbl).GetType.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:5531d1d6033d
pub unsafe fn IDirect3DBaseTexture9_QueryInterface(This: *mut IDirect3DBaseTexture9, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:35a6019cfd38
pub unsafe fn IDirect3DBaseTexture9_Release(This: *mut IDirect3DBaseTexture9) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:d0df5c8b6349
pub unsafe fn IDirect3DCubeTexture9_AddRef(This: *mut IDirect3DCubeTexture9) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:38d16a9a9bcf
pub unsafe fn IDirect3DDevice9Ex_AddRef(This: *mut IDirect3DDevice9Ex) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:df2f70e0a15d
pub unsafe fn IDirect3DDevice9Ex_QueryInterface(This: *mut IDirect3DDevice9Ex, iid: *const IID, out: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, iid, out) } // widl C wrapper
#[inline] // gen:3c852cd458de
pub unsafe fn IDirect3DDevice9Ex_Release(This: *mut IDirect3DDevice9Ex) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:75e23bb6e898
pub unsafe fn IDirect3DDevice9_AddRef(This: *mut IDirect3DDevice9) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:ea52d30d2568
pub unsafe fn IDirect3DIndexBuffer9_AddRef(This: *mut IDirect3DIndexBuffer9) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:97cb5fc727ec
pub unsafe fn IDirect3DPixelShader9_AddRef(This: *mut IDirect3DPixelShader9) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:5f27cc5f4787
pub unsafe fn IDirect3DQuery9_AddRef(This: *mut IDirect3DQuery9) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:2cb2608aa6f6
pub unsafe fn IDirect3DQuery9_Release(This: *mut IDirect3DQuery9) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:8b096a023489
pub unsafe fn IDirect3DStateBlock9_AddRef(This: *mut IDirect3DStateBlock9) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:93de82911b02
pub unsafe fn IDirect3DSurface9_AddRef(This: *mut IDirect3DSurface9) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:f3a016d815f5
pub unsafe fn IDirect3DSurface9_GetDesc(This: *mut IDirect3DSurface9, pDesc: *mut D3DSURFACE_DESC) -> HRESULT { ((*(*This).lpVtbl).GetDesc.unwrap_unchecked())(This, pDesc) } // widl C wrapper
#[inline] // gen:dd4c7bb38dcb
pub unsafe fn IDirect3DSurface9_LockRect(This: *mut IDirect3DSurface9, locked_rect: *mut D3DLOCKED_RECT, rect: *const RECT, flags: DWORD) -> HRESULT { ((*(*This).lpVtbl).LockRect.unwrap_unchecked())(This, locked_rect, rect, flags) } // widl C wrapper
#[inline] // gen:2e10b40e1110
pub unsafe fn IDirect3DSurface9_Release(This: *mut IDirect3DSurface9) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:ad7a8dbaebcc
pub unsafe fn IDirect3DSurface9_UnlockRect(This: *mut IDirect3DSurface9) -> HRESULT { ((*(*This).lpVtbl).UnlockRect.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:5dc49c8d4d41
pub unsafe fn IDirect3DSwapChain9Ex_AddRef(This: *mut IDirect3DSwapChain9Ex) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:b9a15e1723dc
pub unsafe fn IDirect3DSwapChain9Ex_GetBackBuffer(This: *mut IDirect3DSwapChain9Ex, backbuffer_idx: UINT, backbuffer_type: D3DBACKBUFFER_TYPE, backbuffer: *mut *mut IDirect3DSurface9) -> HRESULT { ((*(*This).lpVtbl).GetBackBuffer.unwrap_unchecked())(This, backbuffer_idx, backbuffer_type, backbuffer) } // widl C wrapper
#[inline] // gen:af27bcd88193
pub unsafe fn IDirect3DSwapChain9Ex_Release(This: *mut IDirect3DSwapChain9Ex) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:d2cea6072a84
pub unsafe fn IDirect3DTexture9_AddRef(This: *mut IDirect3DTexture9) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:824867331161
pub unsafe fn IDirect3DTexture9_Release(This: *mut IDirect3DTexture9) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:97cfe530ced7
pub unsafe fn IDirect3DVertexBuffer9_AddRef(This: *mut IDirect3DVertexBuffer9) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:7ebae29ceeaf
pub unsafe fn IDirect3DVertexDeclaration9_AddRef(This: *mut IDirect3DVertexDeclaration9) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:64bedb84f1f7
pub unsafe fn IDirect3DVertexDeclaration9_Release(This: *mut IDirect3DVertexDeclaration9) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:54bc37b7ea4d
pub unsafe fn IDirect3DVertexShader9_AddRef(This: *mut IDirect3DVertexShader9) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:fd41889660f4
pub unsafe fn IDirect3DVolume9_AddRef(This: *mut IDirect3DVolume9) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:9d9090372c6c
pub unsafe fn IDirect3DVolume9_LockBox(This: *mut IDirect3DVolume9, locked_box: *mut D3DLOCKED_BOX, box_: *const D3DBOX, flags: DWORD) -> HRESULT { ((*(*This).lpVtbl).LockBox.unwrap_unchecked())(This, locked_box, box_, flags) } // widl C wrapper
#[inline] // gen:458b3b304cc6
pub unsafe fn IDirect3DVolume9_UnlockBox(This: *mut IDirect3DVolume9) -> HRESULT { ((*(*This).lpVtbl).UnlockBox.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:8e6d2cda2af9
pub unsafe fn IDirect3DVolumeTexture9_AddRef(This: *mut IDirect3DVolumeTexture9) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:6ad0b58c268c
pub unsafe fn IUnknown_AddRef(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:38acc17a31e7
pub unsafe fn IUnknown_QueryInterface(This: *mut IUnknown, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
pub type REFGUID = *const GUID; // gen:4c78e76f64d6
pub type REFIID = *const IID; // gen:c79168ffaea6
pub const S_OK: HRESULT = ((0) as HRESULT); // gen:9969a2c48b97
pub const S_PRESENT_OCCLUDED: HRESULT = (((((((0) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2168) as c_uint))) as HRESULT); // gen:2faaf816d1f8
pub const WINED3DERR_INVALIDCALL: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2156) as c_uint))) as HRESULT); // gen:71d0f13a48b5
pub const WINED3DERR_NOTAVAILABLE: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2154) as c_uint))) as HRESULT); // gen:1ecf2da0a957
pub const WINED3D_OK: HRESULT = ((0) as HRESULT); // gen:7d6ca58d383b
pub const WINEDDERR_INVALIDRECT: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((150) as c_uint))) as HRESULT); // gen:e6270e3e06a4
pub const WINEDDERR_NOTLOCKED: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((584) as c_uint))) as HRESULT); // gen:3fc3eebb09ca
pub use crate::{D3DPS_VERSION, D3DVS_VERSION, max, min};
// ==== end SDK header macros ====

// hand-edited, so kept out of the generated block below (generated_items.py); delete to regenerate
// dll/directx/wine/d3d9/d3d9_private.h:315: header-inline `d3dusage_from_wined3dusage`
#[allow(dead_code)]
#[inline]
pub unsafe fn d3dusage_from_wined3dusage(wined3d_usage: core::ffi::c_uint, bind_flags: core::ffi::c_uint) -> DWORD {
    let mut usage: DWORD = (wined3d_usage & WINED3DUSAGE_MASK) as DWORD;
    if (bind_flags & WINED3D_BIND_RENDER_TARGET) != 0 {
        usage |= (D3DUSAGE_RENDERTARGET as u32);
    }
    if (bind_flags & WINED3D_BIND_DEPTH_STENCIL) != 0 {
        usage |= (D3DUSAGE_DEPTHSTENCIL as u32);
    }
    usage
}
// hand-edited, so kept out of the generated block below (generated_items.py); delete to regenerate
// dll/directx/wine/d3d9/d3d9_private.h:354: header-inline `map_access_from_usage`
#[allow(dead_code)]
#[inline]
pub unsafe fn map_access_from_usage(usage: core::ffi::c_uint) -> core::ffi::c_uint {
    if (usage & (D3DUSAGE_WRITEONLY as core::ffi::c_uint)) != 0 {
        return WINED3D_RESOURCE_ACCESS_MAP_W;
    }
    WINED3D_RESOURCE_ACCESS_MAP_R | WINED3D_RESOURCE_ACCESS_MAP_W
}
// hand-edited, so kept out of the generated block below (generated_items.py); delete to regenerate
// dll/directx/wine/d3d9/d3d9_private.h:395: header-inline `wined3d_bind_flags_from_d3d9_usage`
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_bind_flags_from_d3d9_usage(usage: DWORD) -> core::ffi::c_uint {
    let mut bind_flags: core::ffi::c_uint = 0;

    if (usage & (D3DUSAGE_RENDERTARGET as DWORD)) != 0 {
        bind_flags |= WINED3D_BIND_RENDER_TARGET;
    }
    if (usage & (D3DUSAGE_DEPTHSTENCIL as DWORD)) != 0 {
        bind_flags |= WINED3D_BIND_DEPTH_STENCIL;
    }

    bind_flags
}
// hand-edited, so kept out of the generated block below (generated_items.py); delete to regenerate
// dll/directx/wine/d3d9/d3d9_private.h:361: header-inline `wined3daccess_from_d3dpool`
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3daccess_from_d3dpool(pool: D3DPOOL, usage: core::ffi::c_uint) -> core::ffi::c_uint {
    let mut access: core::ffi::c_uint = match pool {
        D3DPOOL_DEFAULT => WINED3D_RESOURCE_ACCESS_GPU,
        D3DPOOL_MANAGED => WINED3D_RESOURCE_ACCESS_GPU | WINED3D_RESOURCE_ACCESS_CPU,
        D3DPOOL_SYSTEMMEM | D3DPOOL_SCRATCH => WINED3D_RESOURCE_ACCESS_CPU,
        _ => 0,
    };
    if pool != D3DPOOL_DEFAULT || (usage & (D3DUSAGE_DYNAMIC as core::ffi::c_uint)) != 0 {
        access |= map_access_from_usage(usage);
    }
    access
}

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/directx/wine/d3d9/d3d9_private.h:238: header-inline `d3d9_texture_get_draw_texture` // gen:2f5e468da325
#[allow(dead_code)]
#[inline]
pub unsafe fn d3d9_texture_get_draw_texture(texture: *mut d3d9_texture) -> *mut wined3d_texture {
    if !(*texture).draw_texture.is_null() {
        (*texture).draw_texture
    } else {
        (*texture).wined3d_texture
    }
}
// dll/directx/wine/d3d9/d3d9_private.h:344: header-inline `d3dmultisample_type_from_wined3d` // gen:e3cc924d5fad
#[allow(dead_code)]
#[inline]
pub unsafe fn d3dmultisample_type_from_wined3d(ty: wined3d_multisample_type) -> D3DMULTISAMPLE_TYPE {
    ty as D3DMULTISAMPLE_TYPE
}
// dll/directx/wine/d3d9/d3d9_private.h:325: header-inline `d3dpool_from_wined3daccess` // gen:ab5356198d7a
#[allow(dead_code)]
#[inline]
pub unsafe fn d3dpool_from_wined3daccess(access: core::ffi::c_uint, usage: core::ffi::c_uint) -> D3DPOOL {
    if (usage & WINED3DUSAGE_MANAGED) != 0 {
        return D3DPOOL_MANAGED;
    }

    match access & (WINED3D_RESOURCE_ACCESS_GPU | WINED3D_RESOURCE_ACCESS_CPU) {
        WINED3D_RESOURCE_ACCESS_GPU => D3DPOOL_DEFAULT,
        WINED3D_RESOURCE_ACCESS_CPU => {
            if (usage & WINED3DUSAGE_SCRATCH) != 0 {
                D3DPOOL_SCRATCH
            } else {
                D3DPOOL_SYSTEMMEM
            }
        }
        x if x == (WINED3D_RESOURCE_ACCESS_GPU | WINED3D_RESOURCE_ACCESS_CPU) => D3DPOOL_MANAGED,
        _ => D3DPOOL_DEFAULT,
    }
}
// dll/directx/wine/d3d9/d3d9_private.h:349: header-inline `d3dscanlineordering_from_wined3d` // gen:e818776b0776
#[allow(dead_code)]
#[inline]
pub unsafe fn d3dscanlineordering_from_wined3d(ordering: wined3d_scanline_ordering) -> D3DSCANLINEORDERING {
    ordering as D3DSCANLINEORDERING
}
// dll/directx/wine/d3d9/d3d9_private.h:310: header-inline `impl_from_IDirect3DDevice9Ex` // gen:244d23c2429e
#[allow(dead_code)]
#[inline]
pub unsafe fn impl_from_IDirect3DDevice9Ex(iface: *mut IDirect3DDevice9Ex) -> *mut d3d9_device {
    CONTAINING_RECORD!(iface, d3d9_device, IDirect3DDevice9Ex_iface)
}
// dll/directx/wine/d3d9/d3d9_private.h:412: header-inline `wined3d_device_type_from_d3d` // gen:a38e41387ef8
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_device_type_from_d3d(ty: D3DDEVTYPE) -> wined3d_device_type {
    ty as wined3d_device_type
}
// dll/directx/wine/d3d9/d3d9_private.h:407: header-inline `wined3d_multisample_type_from_d3d` // gen:1fbc756bbee1
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_multisample_type_from_d3d(ty: D3DMULTISAMPLE_TYPE) -> wined3d_multisample_type {
    ty as wined3d_multisample_type
}
// dll/directx/wine/d3d9/d3d9_private.h:417: header-inline `wined3d_scanline_ordering_from_d3d` // gen:6ed0de1f8bb3
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_scanline_ordering_from_d3d(ordering: D3DSCANLINEORDERING) -> wined3d_scanline_ordering {
    ordering as wined3d_scanline_ordering
}
// dll/directx/wine/d3d9/d3d9_private.h:385: header-inline `wined3d_usage_from_d3d` // gen:37fe1563e9b4
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_usage_from_d3d(pool: D3DPOOL, usage: DWORD) -> core::ffi::c_uint {
    let mut usage: core::ffi::c_uint = (usage & WINED3DUSAGE_MASK) as core::ffi::c_uint;
    if pool == D3DPOOL_SCRATCH {
        usage |= WINED3DUSAGE_SCRATCH;
    } else if pool == D3DPOOL_MANAGED {
        usage |= WINED3DUSAGE_MANAGED;
    }
    usage
}
// ==== end port macros ====
