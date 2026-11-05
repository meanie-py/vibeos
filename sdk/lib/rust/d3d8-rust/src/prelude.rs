//! d3d8-rust prelude — the single import surface the translated modules use.
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
// The wine/debug.h channel (TRACE/ERR/WARN/FIXME): wine's own flag test on the default
// channel, then ros_dbg_log -- win32macros-rust's WINE_*! macros, which expand HERE (the
// channel storage and the ros_dbg_log extern are this crate's; `port resolve` writes them).
// MACROS, never out-of-line fns: an out-of-line wrapper is an optimization barrier that
// uncloaks latent ABI mismatches into ud2 (field manual §9.65).
#[macro_export] macro_rules! TRACE { ($($arg:tt)*) => { win32macros_rust::WINE_TRACE!($($arg)*) }; }
#[macro_export] macro_rules! ERR   { ($($arg:tt)*) => { win32macros_rust::WINE_ERR!($($arg)*) }; }
#[macro_export] macro_rules! WARN  { ($($arg:tt)*) => { win32macros_rust::WINE_WARN!($($arg)*) }; }
#[macro_export] macro_rules! FIXME { ($($arg:tt)*) => { win32macros_rust::WINE_FIXME!($($arg)*) }; }
pub use crate::{ERR, FIXME, TRACE, WARN};

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
pub const D3D8_DEVICE_STATE_LOST: d3d8_device_state = d3d8_device_state_D3D8_DEVICE_STATE_LOST; // island enum // gen:dc3817b4942d
pub const D3D8_DEVICE_STATE_NOT_RESET: d3d8_device_state = d3d8_device_state_D3D8_DEVICE_STATE_NOT_RESET; // island enum // gen:fb5d52828a47
pub const D3D8_DEVICE_STATE_OK: d3d8_device_state = d3d8_device_state_D3D8_DEVICE_STATE_OK; // island enum // gen:45a9597e008d
pub const D3D8_HANDLE_FREE: d3d8_handle_type = d3d8_handle_type_D3D8_HANDLE_FREE; // island enum // gen:2a92b7a94dcf
pub const D3D8_HANDLE_PS: d3d8_handle_type = d3d8_handle_type_D3D8_HANDLE_PS; // island enum // gen:75dbed2503ef
pub const D3D8_HANDLE_SB: d3d8_handle_type = d3d8_handle_type_D3D8_HANDLE_SB; // island enum // gen:9c2052215a7f
pub const D3D8_HANDLE_VS: d3d8_handle_type = d3d8_handle_type_D3D8_HANDLE_VS; // island enum // gen:98696988b670
pub const D3DCREATE_MULTITHREADED: i32 = 4; // d3d9-rust/src/prelude.rs // gen:7db74cd43d56
pub const D3DFMT_A1R5G5B5: _D3DFORMAT = _D3DFORMAT_D3DFMT_A1R5G5B5; // island enum // gen:6f9a88590099
pub const D3DFMT_A2B10G10R10: _D3DFORMAT = _D3DFORMAT_D3DFMT_A2B10G10R10; // island enum // gen:bb0e3657ab41
pub const D3DFMT_A2W10V10U10: _D3DFORMAT = _D3DFORMAT_D3DFMT_A2W10V10U10; // island enum // gen:79e10536f3e8
pub const D3DFMT_A4L4: _D3DFORMAT = _D3DFORMAT_D3DFMT_A4L4; // island enum // gen:e185a882a5ac
pub const D3DFMT_A4R4G4B4: _D3DFORMAT = _D3DFORMAT_D3DFMT_A4R4G4B4; // island enum // gen:02f39479f797
pub const D3DFMT_A8: _D3DFORMAT = _D3DFORMAT_D3DFMT_A8; // island enum // gen:f52159c6641c
pub const D3DFMT_A8L8: _D3DFORMAT = _D3DFORMAT_D3DFMT_A8L8; // island enum // gen:9d234b7c31cc
pub const D3DFMT_A8P8: _D3DFORMAT = _D3DFORMAT_D3DFMT_A8P8; // island enum // gen:7dd2abeccce8
pub const D3DFMT_A8R3G3B2: _D3DFORMAT = _D3DFORMAT_D3DFMT_A8R3G3B2; // island enum // gen:af1f63301ce2
pub const D3DFMT_A8R8G8B8: _D3DFORMAT = _D3DFORMAT_D3DFMT_A8R8G8B8; // island enum // gen:9f6f0fc4c1b9
pub const D3DFMT_D15S1: _D3DFORMAT = _D3DFORMAT_D3DFMT_D15S1; // island enum // gen:8b80cd5d4b1b
pub const D3DFMT_D16: _D3DFORMAT = _D3DFORMAT_D3DFMT_D16; // island enum // gen:4752cb83efb7
pub const D3DFMT_D16_LOCKABLE: _D3DFORMAT = _D3DFORMAT_D3DFMT_D16_LOCKABLE; // island enum // gen:34db1e794b16
pub const D3DFMT_D24S8: _D3DFORMAT = _D3DFORMAT_D3DFMT_D24S8; // island enum // gen:e1c4526080cb
pub const D3DFMT_D24X4S4: _D3DFORMAT = _D3DFORMAT_D3DFMT_D24X4S4; // island enum // gen:7563c3f06a76
pub const D3DFMT_D24X8: _D3DFORMAT = _D3DFORMAT_D3DFMT_D24X8; // island enum // gen:ad515ddad853
pub const D3DFMT_D32: _D3DFORMAT = _D3DFORMAT_D3DFMT_D32; // island enum // gen:520f2b18c70b
pub const D3DFMT_G16R16: _D3DFORMAT = _D3DFORMAT_D3DFMT_G16R16; // island enum // gen:2f46745cc45d
pub const D3DFMT_INDEX16: _D3DFORMAT = _D3DFORMAT_D3DFMT_INDEX16; // island enum // gen:20a989fae87c
pub const D3DFMT_INDEX32: _D3DFORMAT = _D3DFORMAT_D3DFMT_INDEX32; // island enum // gen:2a6926658c3d
pub const D3DFMT_L6V5U5: _D3DFORMAT = _D3DFORMAT_D3DFMT_L6V5U5; // island enum // gen:6962003f9e23
pub const D3DFMT_L8: _D3DFORMAT = _D3DFORMAT_D3DFMT_L8; // island enum // gen:d43c484f6894
pub const D3DFMT_P8: _D3DFORMAT = _D3DFORMAT_D3DFMT_P8; // island enum // gen:4885307876d4
pub const D3DFMT_Q8W8V8U8: _D3DFORMAT = _D3DFORMAT_D3DFMT_Q8W8V8U8; // island enum // gen:6b18c3cbbf7a
pub const D3DFMT_R3G3B2: _D3DFORMAT = _D3DFORMAT_D3DFMT_R3G3B2; // island enum // gen:59b5c58dbdd1
pub const D3DFMT_R5G6B5: _D3DFORMAT = _D3DFORMAT_D3DFMT_R5G6B5; // island enum // gen:435d5f64d38c
pub const D3DFMT_R8G8B8: _D3DFORMAT = _D3DFORMAT_D3DFMT_R8G8B8; // island enum // gen:f88f76c0d7f8
pub const D3DFMT_UNKNOWN: _D3DFORMAT = _D3DFORMAT_D3DFMT_UNKNOWN; // island enum // gen:febb94381c1a
pub const D3DFMT_V16U16: _D3DFORMAT = _D3DFORMAT_D3DFMT_V16U16; // island enum // gen:eb7099a85cfc
pub const D3DFMT_V8U8: _D3DFORMAT = _D3DFORMAT_D3DFMT_V8U8; // island enum // gen:51a24d4af040
pub const D3DFMT_VERTEXDATA: _D3DFORMAT = _D3DFORMAT_D3DFMT_VERTEXDATA; // island enum // gen:6cb2619ce763
pub const D3DFMT_W11V11U10: _D3DFORMAT = _D3DFORMAT_D3DFMT_W11V11U10; // island enum // gen:0f9ced44e809
pub const D3DFMT_X1R5G5B5: _D3DFORMAT = _D3DFORMAT_D3DFMT_X1R5G5B5; // island enum // gen:b1217078b2eb
pub const D3DFMT_X4R4G4B4: _D3DFORMAT = _D3DFORMAT_D3DFMT_X4R4G4B4; // island enum // gen:095b89672f24
pub const D3DFMT_X8L8V8U8: _D3DFORMAT = _D3DFORMAT_D3DFMT_X8L8V8U8; // island enum // gen:62d884fcd754
pub const D3DFMT_X8R8G8B8: _D3DFORMAT = _D3DFORMAT_D3DFMT_X8R8G8B8; // island enum // gen:c6e7a7970fd6
pub const D3DMULTISAMPLE_16_SAMPLES: _D3DMULTISAMPLE_TYPE = _D3DMULTISAMPLE_TYPE_D3DMULTISAMPLE_16_SAMPLES; // island enum // gen:e30d633e02ae
pub const D3DPMISCCAPS_CULLCCW: u32 = 64; // ddraw-rust/src/prelude.rs // gen:7545292703e8
pub const D3DPMISCCAPS_CULLCW: u32 = 32; // ddraw-rust/src/prelude.rs // gen:f02f40171388
pub const D3DPMISCCAPS_CULLNONE: u32 = 16; // ddraw-rust/src/prelude.rs // gen:984face6ee2b
pub const D3DPMISCCAPS_LINEPATTERNREP: u32 = 4; // ddraw-rust/src/prelude.rs // gen:9a7be4250a55
pub const D3DPMISCCAPS_MASKZ: u32 = 2; // ddraw-rust/src/prelude.rs // gen:eba8bbcffbc2
pub const D3DPOOL_DEFAULT: _D3DPOOL = _D3DPOOL_D3DPOOL_DEFAULT; // island enum // gen:da3b87a3608d
pub const D3DPOOL_MANAGED: _D3DPOOL = _D3DPOOL_D3DPOOL_MANAGED; // island enum // gen:9c0f686269de
pub const D3DPOOL_SCRATCH: _D3DPOOL = _D3DPOOL_D3DPOOL_SCRATCH; // island enum // gen:67c884565452
pub const D3DPOOL_SYSTEMMEM: _D3DPOOL = _D3DPOOL_D3DPOOL_SYSTEMMEM; // island enum // gen:1fd728eeb6dc
pub const D3DPT_LINELIST: _D3DPRIMITIVETYPE = _D3DPRIMITIVETYPE_D3DPT_LINELIST; // island enum // gen:079724b4a92e
pub const D3DPT_LINESTRIP: _D3DPRIMITIVETYPE = _D3DPRIMITIVETYPE_D3DPT_LINESTRIP; // island enum // gen:d52694065390
pub const D3DPT_POINTLIST: _D3DPRIMITIVETYPE = _D3DPRIMITIVETYPE_D3DPT_POINTLIST; // island enum // gen:05e9bada0d8c
pub const D3DPT_TRIANGLEFAN: _D3DPRIMITIVETYPE = _D3DPRIMITIVETYPE_D3DPT_TRIANGLEFAN; // island enum // gen:364b3fc70c95
pub const D3DPT_TRIANGLELIST: _D3DPRIMITIVETYPE = _D3DPRIMITIVETYPE_D3DPT_TRIANGLELIST; // island enum // gen:902c2df46fb7
pub const D3DPT_TRIANGLESTRIP: _D3DPRIMITIVETYPE = _D3DPRIMITIVETYPE_D3DPT_TRIANGLESTRIP; // island enum // gen:7a68725ac6b1
pub const D3DRS_EDGEANTIALIAS: _D3DRENDERSTATETYPE = _D3DRENDERSTATETYPE_D3DRS_EDGEANTIALIAS; // island enum // gen:49aa5866f8be
pub const D3DRS_POINTSIZE: _D3DRENDERSTATETYPE = _D3DRENDERSTATETYPE_D3DRS_POINTSIZE; // island enum // gen:9f2b832b8495
pub const D3DRS_ZBIAS: _D3DRENDERSTATETYPE = _D3DRENDERSTATETYPE_D3DRS_ZBIAS; // island enum // gen:a0fa38c62584
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
pub const D3DSWAPEFFECT_COPY_VSYNC: _D3DSWAPEFFECT = _D3DSWAPEFFECT_D3DSWAPEFFECT_COPY_VSYNC; // island enum // gen:bc6067102e58
pub const D3DSWAPEFFECT_DISCARD: _D3DSWAPEFFECT = _D3DSWAPEFFECT_D3DSWAPEFFECT_DISCARD; // island enum // gen:b44268d22891
pub const D3DSWAPEFFECT_FLIP: _D3DSWAPEFFECT = _D3DSWAPEFFECT_D3DSWAPEFFECT_FLIP; // island enum // gen:987ef8830033
pub const D3DVSDE_BLENDINDICES: _D3DVSDE_REGISTER = _D3DVSDE_REGISTER_D3DVSDE_BLENDINDICES; // island enum // gen:d97d923a953d
pub const D3DVSDE_BLENDWEIGHT: _D3DVSDE_REGISTER = _D3DVSDE_REGISTER_D3DVSDE_BLENDWEIGHT; // island enum // gen:f6c5855aca6a
pub const D3DVSDE_DIFFUSE: _D3DVSDE_REGISTER = _D3DVSDE_REGISTER_D3DVSDE_DIFFUSE; // island enum // gen:7967b4374734
pub const D3DVSDE_NORMAL2: _D3DVSDE_REGISTER = _D3DVSDE_REGISTER_D3DVSDE_NORMAL2; // island enum // gen:28c2c89a9392
pub const D3DVSDE_NORMAL: _D3DVSDE_REGISTER = _D3DVSDE_REGISTER_D3DVSDE_NORMAL; // island enum // gen:268e38a11ee0
pub const D3DVSDE_POSITION2: _D3DVSDE_REGISTER = _D3DVSDE_REGISTER_D3DVSDE_POSITION2; // island enum // gen:0e36416cf0ba
pub const D3DVSDE_POSITION: _D3DVSDE_REGISTER = _D3DVSDE_REGISTER_D3DVSDE_POSITION; // island enum // gen:3fecbe27d136
pub const D3DVSDE_PSIZE: _D3DVSDE_REGISTER = _D3DVSDE_REGISTER_D3DVSDE_PSIZE; // island enum // gen:94ed28a478d0
pub const D3DVSDE_SPECULAR: _D3DVSDE_REGISTER = _D3DVSDE_REGISTER_D3DVSDE_SPECULAR; // island enum // gen:5863dd3bdbf8
pub const D3DVSDE_TEXCOORD0: _D3DVSDE_REGISTER = _D3DVSDE_REGISTER_D3DVSDE_TEXCOORD0; // island enum // gen:c274f47b70ad
pub const D3DVSDE_TEXCOORD1: _D3DVSDE_REGISTER = _D3DVSDE_REGISTER_D3DVSDE_TEXCOORD1; // island enum // gen:0161ca42292d
pub const D3DVSDE_TEXCOORD2: _D3DVSDE_REGISTER = _D3DVSDE_REGISTER_D3DVSDE_TEXCOORD2; // island enum // gen:2a8fb8b02af1
pub const D3DVSDE_TEXCOORD3: _D3DVSDE_REGISTER = _D3DVSDE_REGISTER_D3DVSDE_TEXCOORD3; // island enum // gen:7d52a0ed9eae
pub const D3DVSDE_TEXCOORD4: _D3DVSDE_REGISTER = _D3DVSDE_REGISTER_D3DVSDE_TEXCOORD4; // island enum // gen:8b7a116c7614
pub const D3DVSDE_TEXCOORD5: _D3DVSDE_REGISTER = _D3DVSDE_REGISTER_D3DVSDE_TEXCOORD5; // island enum // gen:f8773ce9c371
pub const D3DVSDE_TEXCOORD6: _D3DVSDE_REGISTER = _D3DVSDE_REGISTER_D3DVSDE_TEXCOORD6; // island enum // gen:15837ac80441
pub const D3DVSDE_TEXCOORD7: _D3DVSDE_REGISTER = _D3DVSDE_REGISTER_D3DVSDE_TEXCOORD7; // island enum // gen:6cdb38a2723a
pub const D3DVSDT_D3DCOLOR: _D3DVSDT_TYPE = _D3DVSDT_TYPE_D3DVSDT_D3DCOLOR; // island enum // gen:9ad4c5992b27
pub const D3DVSDT_FLOAT1: _D3DVSDT_TYPE = _D3DVSDT_TYPE_D3DVSDT_FLOAT1; // island enum // gen:aa3cdcb140ab
pub const D3DVSDT_FLOAT2: _D3DVSDT_TYPE = _D3DVSDT_TYPE_D3DVSDT_FLOAT2; // island enum // gen:fe9622b790ae
pub const D3DVSDT_FLOAT3: _D3DVSDT_TYPE = _D3DVSDT_TYPE_D3DVSDT_FLOAT3; // island enum // gen:b2940b066c5b
pub const D3DVSDT_FLOAT4: _D3DVSDT_TYPE = _D3DVSDT_TYPE_D3DVSDT_FLOAT4; // island enum // gen:b405f06f8f3b
pub const D3DVSDT_SHORT2: _D3DVSDT_TYPE = _D3DVSDT_TYPE_D3DVSDT_SHORT2; // island enum // gen:76a495e37983
pub const D3DVSDT_SHORT4: _D3DVSDT_TYPE = _D3DVSDT_TYPE_D3DVSDT_SHORT4; // island enum // gen:936729ee13f6
pub const D3DVSDT_UBYTE4: _D3DVSDT_TYPE = _D3DVSDT_TYPE_D3DVSDT_UBYTE4; // island enum // gen:b911db90dcbd
pub const D3DVSD_TOKEN_CONSTMEM: _D3DVSD_TOKENTYPE = _D3DVSD_TOKENTYPE_D3DVSD_TOKEN_CONSTMEM; // island enum // gen:a53eb2aa2bb9
pub const D3DVSD_TOKEN_END: _D3DVSD_TOKENTYPE = _D3DVSD_TOKENTYPE_D3DVSD_TOKEN_END; // island enum // gen:f592f13236d1
pub const D3DVSD_TOKEN_EXT: _D3DVSD_TOKENTYPE = _D3DVSD_TOKENTYPE_D3DVSD_TOKEN_EXT; // island enum // gen:6d497477c86a
pub const D3DVSD_TOKEN_NOP: _D3DVSD_TOKENTYPE = _D3DVSD_TOKENTYPE_D3DVSD_TOKEN_NOP; // island enum // gen:e300456f52c3
pub const D3DVSD_TOKEN_STREAM: _D3DVSD_TOKENTYPE = _D3DVSD_TOKENTYPE_D3DVSD_TOKEN_STREAM; // island enum // gen:1706b0349502
pub const D3DVSD_TOKEN_STREAMDATA: _D3DVSD_TOKENTYPE = _D3DVSD_TOKENTYPE_D3DVSD_TOKEN_STREAMDATA; // island enum // gen:5e9010c401ca
pub const D3DVSD_TOKEN_TESSELLATOR: _D3DVSD_TOKENTYPE = _D3DVSD_TOKENTYPE_D3DVSD_TOKEN_TESSELLATOR; // island enum // gen:8e92d6f6d5a9
pub const D3DVTXPCAPS_DIRECTIONALLIGHTS: u32 = 8; // d3d9-rust/src/prelude.rs // gen:62bcd6464521
pub const D3DVTXPCAPS_LOCALVIEWER: u32 = 32; // d3d9-rust/src/prelude.rs // gen:dd0d4cd9311a
pub const D3DVTXPCAPS_MATERIALSOURCE7: u32 = 2; // d3d9-rust/src/prelude.rs // gen:5354de73c83c
pub const D3DVTXPCAPS_POSITIONALLIGHTS: u32 = 16; // d3d9-rust/src/prelude.rs // gen:f34c17fea36e
pub const D3DVTXPCAPS_TEXGEN: u32 = 1; // d3d9-rust/src/prelude.rs // gen:690240211c10
pub const FALSE: u32 = 0; // at-rust/src/types_amd64.rs // gen:58c6c88227ec
pub const TRUE: u32 = 1; // at-rust/src/types_amd64.rs // gen:798e790b3940
pub const WINED3DFMT_A8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_A8_UNORM; // island enum // gen:b852344850b7
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
pub const WINED3DFMT_D32_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_D32_UNORM; // island enum // gen:fac0db7c6301
pub const WINED3DFMT_DF16: wined3d_format_id = wined3d_format_id_WINED3DFMT_DF16; // island enum // gen:a4bfb44af8fa
pub const WINED3DFMT_DF24: wined3d_format_id = wined3d_format_id_WINED3DFMT_DF24; // island enum // gen:a05a4d31466c
pub const WINED3DFMT_INTZ: wined3d_format_id = wined3d_format_id_WINED3DFMT_INTZ; // island enum // gen:315ea9570ef9
pub const WINED3DFMT_L4A4_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_L4A4_UNORM; // island enum // gen:7c9feb6fdd9c
pub const WINED3DFMT_L8A8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_L8A8_UNORM; // island enum // gen:621b76d8945e
pub const WINED3DFMT_L8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_L8_UNORM; // island enum // gen:385347c438e3
pub const WINED3DFMT_P8_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_P8_UINT; // island enum // gen:08be2f398d7b
pub const WINED3DFMT_P8_UINT_A8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_P8_UINT_A8_UNORM; // island enum // gen:ee16b1321b40
pub const WINED3DFMT_R10G10B10A2_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R10G10B10A2_UNORM; // island enum // gen:1c4034896072
pub const WINED3DFMT_R10G10B10X2_SNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R10G10B10X2_SNORM; // island enum // gen:fa5916576b7d
pub const WINED3DFMT_R10G10B10X2_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R10G10B10X2_UINT; // island enum // gen:e9b2e89b1d31
pub const WINED3DFMT_R10G10B10_SNORM_A2_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R10G10B10_SNORM_A2_UNORM; // island enum // gen:408936d9d5be
pub const WINED3DFMT_R10G11B11_SNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R10G11B11_SNORM; // island enum // gen:2d160d053c41
pub const WINED3DFMT_R16G16B16A16_FLOAT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16B16A16_FLOAT; // island enum // gen:7df2955d6781
pub const WINED3DFMT_R16G16B16A16_SINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16B16A16_SINT; // island enum // gen:2178584fcdfb
pub const WINED3DFMT_R16G16B16A16_SNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16B16A16_SNORM; // island enum // gen:7ad3ef834bb6
pub const WINED3DFMT_R16G16B16A16_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16B16A16_UNORM; // island enum // gen:a32ff2d7dffc
pub const WINED3DFMT_R16G16_FLOAT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16_FLOAT; // island enum // gen:df79272fcb2e
pub const WINED3DFMT_R16G16_SINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16_SINT; // island enum // gen:5aee36a3ea1d
pub const WINED3DFMT_R16G16_SNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16_SNORM; // island enum // gen:c6640d1032ad
pub const WINED3DFMT_R16G16_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16G16_UNORM; // island enum // gen:db785a6d61b2
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
pub const WINED3DFMT_R8G8_SNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8_SNORM; // island enum // gen:2315347f9119
pub const WINED3DFMT_R8G8_SNORM_L8X8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8_SNORM_L8X8_UNORM; // island enum // gen:d3e02c96ba02
pub const WINED3DFMT_S1_UINT_D15_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_S1_UINT_D15_UNORM; // island enum // gen:a396b379f935
pub const WINED3DFMT_S4X4_UINT_D24_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_S4X4_UINT_D24_UNORM; // island enum // gen:6fe6c428ea21
pub const WINED3DFMT_UNKNOWN: wined3d_format_id = wined3d_format_id_WINED3DFMT_UNKNOWN; // island enum // gen:93d9074cc710
pub const WINED3DFMT_X8D24_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_X8D24_UNORM; // island enum // gen:b81fd5c118b7
pub const WINED3D_DECL_METHOD_DEFAULT: wined3d_decl_method = wined3d_decl_method_WINED3D_DECL_METHOD_DEFAULT; // island enum // gen:317b339f67b0
pub const WINED3D_DECL_USAGE_BLEND_INDICES: wined3d_decl_usage = wined3d_decl_usage_WINED3D_DECL_USAGE_BLEND_INDICES; // island enum // gen:819fcd50e5a1
pub const WINED3D_DECL_USAGE_BLEND_WEIGHT: wined3d_decl_usage = wined3d_decl_usage_WINED3D_DECL_USAGE_BLEND_WEIGHT; // island enum // gen:27239a351146
pub const WINED3D_DECL_USAGE_COLOR: wined3d_decl_usage = wined3d_decl_usage_WINED3D_DECL_USAGE_COLOR; // island enum // gen:af9841f56ae3
pub const WINED3D_DECL_USAGE_NORMAL: wined3d_decl_usage = wined3d_decl_usage_WINED3D_DECL_USAGE_NORMAL; // island enum // gen:b5b8ef4a975f
pub const WINED3D_DECL_USAGE_POSITION: wined3d_decl_usage = wined3d_decl_usage_WINED3D_DECL_USAGE_POSITION; // island enum // gen:232ccb2172ff
pub const WINED3D_DECL_USAGE_PSIZE: wined3d_decl_usage = wined3d_decl_usage_WINED3D_DECL_USAGE_PSIZE; // island enum // gen:d7a80e727f7c
pub const WINED3D_DECL_USAGE_TEXCOORD: wined3d_decl_usage = wined3d_decl_usage_WINED3D_DECL_USAGE_TEXCOORD; // island enum // gen:ada9b7718732
pub const WINED3D_FEATURE_LEVEL_5: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_5; // island enum // gen:f9c53f3d677f
pub const WINED3D_FEATURE_LEVEL_6: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_6; // island enum // gen:e970439b7cb3
pub const WINED3D_FEATURE_LEVEL_7: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_7; // island enum // gen:03cd701a1439
pub const WINED3D_FEATURE_LEVEL_8: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_8; // island enum // gen:396b2980b189
pub const WINED3D_INPUT_PER_VERTEX_DATA: wined3d_input_classification = wined3d_input_classification_WINED3D_INPUT_PER_VERTEX_DATA; // island enum // gen:e87a81739d3a
pub const WINED3D_MULTISAMPLE_NONE: wined3d_multisample_type = wined3d_multisample_type_WINED3D_MULTISAMPLE_NONE; // island enum // gen:bfa66121fcaa
pub const WINED3D_MULTISAMPLE_NON_MASKABLE: wined3d_multisample_type = wined3d_multisample_type_WINED3D_MULTISAMPLE_NON_MASKABLE; // island enum // gen:d5b0152c15a3
pub const WINED3D_RS_ANTIALIASEDLINEENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_ANTIALIASEDLINEENABLE; // island enum // gen:a1d64f8207d7
pub const WINED3D_RS_DEPTHBIAS: wined3d_render_state = wined3d_render_state_WINED3D_RS_DEPTHBIAS; // island enum // gen:b185bd45eb78
pub const WINED3D_RS_POINTSIZE_MIN: wined3d_render_state = wined3d_render_state_WINED3D_RS_POINTSIZE_MIN; // island enum // gen:28fc240f2476
pub const WINED3D_RS_ZENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_ZENABLE; // island enum // gen:f8bcd5a86ca6
pub const WINED3D_RTYPE_BUFFER: wined3d_resource_type = wined3d_resource_type_WINED3D_RTYPE_BUFFER; // island enum // gen:8443940bf9c4
pub const WINED3D_RTYPE_NONE: wined3d_resource_type = wined3d_resource_type_WINED3D_RTYPE_NONE; // island enum // gen:20c459743c49
pub const WINED3D_RTYPE_TEXTURE_2D: wined3d_resource_type = wined3d_resource_type_WINED3D_RTYPE_TEXTURE_2D; // island enum // gen:de40c3024d68
pub const WINED3D_RTYPE_TEXTURE_3D: wined3d_resource_type = wined3d_resource_type_WINED3D_RTYPE_TEXTURE_3D; // island enum // gen:7752a0d240c9
pub const WINED3D_SAMP_ADDRESS_U: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_ADDRESS_U; // island enum // gen:1edc0ba1eb48
pub const WINED3D_SAMP_ADDRESS_V: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_ADDRESS_V; // island enum // gen:863872073cda
pub const WINED3D_SAMP_ADDRESS_W: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_ADDRESS_W; // island enum // gen:24b06d72c7b8
pub const WINED3D_SAMP_BORDER_COLOR: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_BORDER_COLOR; // island enum // gen:6359d21babb4
pub const WINED3D_SAMP_MAG_FILTER: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_MAG_FILTER; // island enum // gen:1f1d5697ec8a
pub const WINED3D_SAMP_MAX_ANISOTROPY: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_MAX_ANISOTROPY; // island enum // gen:8f1c9604c9be
pub const WINED3D_SAMP_MAX_MIP_LEVEL: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_MAX_MIP_LEVEL; // island enum // gen:798c7969f2c4
pub const WINED3D_SAMP_MIN_FILTER: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_MIN_FILTER; // island enum // gen:e75251a959b9
pub const WINED3D_SAMP_MIPMAP_LOD_BIAS: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_MIPMAP_LOD_BIAS; // island enum // gen:099207089185
pub const WINED3D_SAMP_MIP_FILTER: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_MIP_FILTER; // island enum // gen:118fa724e031
pub const WINED3D_SBT_PRIMARY: wined3d_stateblock_type = wined3d_stateblock_type_WINED3D_SBT_PRIMARY; // island enum // gen:a4ee8d8e4d8b
pub const WINED3D_SBT_RECORDED: wined3d_stateblock_type = wined3d_stateblock_type_WINED3D_SBT_RECORDED; // island enum // gen:a3a30ecb7bc4
pub const WINED3D_SCANLINE_ORDERING_UNKNOWN: wined3d_scanline_ordering = wined3d_scanline_ordering_WINED3D_SCANLINE_ORDERING_UNKNOWN; // island enum // gen:7774b8a98afe
pub const WINED3D_SWAP_EFFECT_COPY: wined3d_swap_effect = wined3d_swap_effect_WINED3D_SWAP_EFFECT_COPY; // island enum // gen:c4b8f5e74118
pub const WINED3D_SWAP_EFFECT_COPY_VSYNC: wined3d_swap_effect = wined3d_swap_effect_WINED3D_SWAP_EFFECT_COPY_VSYNC; // island enum // gen:99877ae015ca
pub const WINED3D_SWAP_EFFECT_DISCARD: wined3d_swap_effect = wined3d_swap_effect_WINED3D_SWAP_EFFECT_DISCARD; // island enum // gen:7f0e42311775
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
pub const WINED3D_TSS_INVALID: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_INVALID; // island enum // gen:917b0a20b2b3
pub const WINED3D_TSS_RESULT_ARG: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_RESULT_ARG; // island enum // gen:c25fee23b0cb
pub const WINED3D_TSS_TEXCOORD_INDEX: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_TEXCOORD_INDEX; // island enum // gen:8e80cbf7e4eb
pub const WINED3D_TSS_TEXTURE_TRANSFORM_FLAGS: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_TEXTURE_TRANSFORM_FLAGS; // island enum // gen:f9828e809b53
extern "C" {
    pub fn calloc(arg1: ::core::ffi::c_uint, arg2: ::core::ffi::c_uint) -> *mut ::core::ffi::c_void; // msvcrt-sys/src/bindgen_i386.rs // gen:d3193c100ce5
    pub fn free(p: *mut c_void); // activeds-rust/src/prelude.rs // gen:0ad441c3c230
    pub fn isprint(c: c_int) -> c_int; // d3d9-rust/src/prelude.rs // gen:c088d5c257f2
    pub fn malloc(size: usize) -> *mut c_void; // activeds-rust/src/prelude.rs // gen:86a212c6fd14
    pub fn realloc(p: *mut c_void, size: usize) -> *mut c_void; // activeds-rust/src/prelude.rs // gen:ce18447a16eb
}
extern "system" {
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs // gen:8b50fa1e44f3
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs // gen:3aaa7317691b
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbLengthW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};
pub use winelist_rust::{list_add_after, list_add_before, list_add_head, list_add_tail, list_count, list_empty, list_head, list_init, list_move_head, list_move_tail, list_next, list_prev, list_remove, list_tail};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const D3DCREATE_FPU_PRESERVE: i32 = 2; // clang, device.c
pub const D3DFMT_RESZ: DWORD = 0x5A534552; // clang, directx.c
pub const D3DPMISCCAPS_BLENDOP: i32 = 0x800; // clang, device.c
pub const D3DPMISCCAPS_CLIPPLANESCALEDPOINTS: i32 = 0x100; // clang, device.c
pub const D3DPMISCCAPS_CLIPTLVERTS: i32 = 0x200; // clang, device.c
pub const D3DPMISCCAPS_COLORWRITEENABLE: i32 = 0x80; // clang, device.c
pub const D3DPMISCCAPS_NULLREFERENCE: i32 = 0x1000; // clang, device.c
pub const D3DPMISCCAPS_TSSARGTEMP: i32 = 0x400; // clang, device.c
pub const D3DVTXPCAPS_NO_VSDT_UBYTE4: i32 = 0x80; // clang, device.c
pub const D3DVTXPCAPS_TWEENING: i32 = 0x40; // clang, device.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// D3DBACKBUFFER_TYPE: c2rust, device.c
pub type D3DBACKBUFFER_TYPE = _D3DBACKBUFFER_TYPE;
// D3DCUBEMAP_FACES: c2rust, texture.c
pub type D3DCUBEMAP_FACES = _D3DCUBEMAP_FACES;
// D3DDEVTYPE: c2rust, device.c
pub type D3DDEVTYPE = _D3DDEVTYPE;
// D3DFORMAT: c2rust, buffer.c
pub type D3DFORMAT = _D3DFORMAT;
// D3DMULTISAMPLE_TYPE: c2rust, device.c
pub type D3DMULTISAMPLE_TYPE = _D3DMULTISAMPLE_TYPE;
// D3DPOOL: c2rust, buffer.c
pub type D3DPOOL = _D3DPOOL;
// D3DPRIMITIVETYPE: c2rust, device.c
pub type D3DPRIMITIVETYPE = _D3DPRIMITIVETYPE;
// D3DRENDERSTATETYPE: c2rust, device.c
pub type D3DRENDERSTATETYPE = _D3DRENDERSTATETYPE;
// D3DRESOURCETYPE: c2rust, buffer.c
pub type D3DRESOURCETYPE = _D3DRESOURCETYPE;
// D3DSTATEBLOCKTYPE: c2rust, device.c
pub type D3DSTATEBLOCKTYPE = _D3DSTATEBLOCKTYPE;
// D3DSWAPEFFECT: c2rust, device.c
pub type D3DSWAPEFFECT = _D3DSWAPEFFECT;
// D3DTEXTURESTAGESTATETYPE: c2rust, device.c
pub type D3DTEXTURESTAGESTATETYPE = _D3DTEXTURESTAGESTATETYPE;
// D3DTRANSFORMSTATETYPE: c2rust, device.c
pub type D3DTRANSFORMSTATETYPE = _D3DTRANSFORMSTATETYPE;
// D3DVSDE_REGISTER: c2rust, vertexdeclaration.c
pub type D3DVSDE_REGISTER = _D3DVSDE_REGISTER;
// D3DVSDT_TYPE: c2rust, vertexdeclaration.c
pub type D3DVSDT_TYPE = _D3DVSDT_TYPE;
// D3DVSD_TOKENTYPE: c2rust, shader.c
pub type D3DVSD_TOKENTYPE = _D3DVSD_TOKENTYPE;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn __wine_dbg_get_channel_flags(channel: *mut __wine_debug_channel) -> ::core::ffi::c_uchar; // vertexdeclaration.c
    pub fn ros_dbg_log(
        cls: __wine_debug_class,
        ch: *mut __wine_debug_channel,
        file: *const ::core::ffi::c_char,
        func: *const ::core::ffi::c_char,
        line: ::core::ffi::c_int,
        format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // buffer.c
    pub fn strlen(_Str: *const ::core::ffi::c_char) -> size_t; // d3d8_main.c
    pub fn wined3d_adapter_get_identifier(
        adapter: *const wined3d_adapter,
        flags: uint32_t,
        identifier: *mut wined3d_adapter_identifier,
    ) -> HRESULT; // directx.c
    pub fn wined3d_adapter_get_output(
        adapter: *const wined3d_adapter,
        idx: ::core::ffi::c_uint,
    ) -> *mut wined3d_output; // directx.c
    pub fn wined3d_adapter_get_output_count(adapter: *const wined3d_adapter) -> ::core::ffi::c_uint; // directx.c
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
    pub fn wined3d_device_get_display_mode(
        device: *const wined3d_device,
        swapchain_idx: UINT,
        mode: *mut wined3d_display_mode,
        rotation: *mut wined3d_display_rotation,
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
    pub fn wined3d_output_get_display_mode(
        output: *const wined3d_output,
        mode: *mut wined3d_display_mode,
        rotation: *mut wined3d_display_rotation,
    ) -> HRESULT; // directx.c
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
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_d3d8: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x64\x33\x64\x38\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_d3d8 };
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

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::d3d8::directx::d3d8_init;
pub use crate::d3d8::device::d3d8_null_wined3d_parent_ops;
pub use crate::d3d8::shader::d3d8_pixel_shader_destroy;
pub use crate::d3d8::shader::d3d8_pixel_shader_init;
pub use crate::d3d8::d3d8_main::d3d8_resource_cleanup;
pub use crate::d3d8::d3d8_main::d3d8_resource_free_private_data;
pub use crate::d3d8::d3d8_main::d3d8_resource_get_private_data;
pub use crate::d3d8::d3d8_main::d3d8_resource_init;
pub use crate::d3d8::d3d8_main::d3d8_resource_set_private_data;
pub use crate::d3d8::surface::d3d8_surface_acquire_rendertarget_view;
pub use crate::d3d8::surface::d3d8_surface_create;
pub use crate::d3d8::surface::d3d8_surface_get_device;
pub use crate::d3d8::surface::d3d8_surface_release_rendertarget_view;
pub use crate::d3d8::swapchain::d3d8_swapchain_create;
pub use crate::d3d8::texture::d3d8_texture_2d_init;
pub use crate::d3d8::texture::d3d8_texture_3d_init;
pub use crate::d3d8::texture::d3d8_texture_cube_init;
pub use crate::d3d8::vertexdeclaration::d3d8_vertex_declaration_destroy;
pub use crate::d3d8::vertexdeclaration::d3d8_vertex_declaration_init;
pub use crate::d3d8::vertexdeclaration::d3d8_vertex_declaration_init_fvf;
pub use crate::d3d8::shader::d3d8_vertex_shader_destroy;
pub use crate::d3d8::shader::d3d8_vertex_shader_init;
pub use crate::d3d8::device::d3dcaps_from_wined3dcaps;
pub use crate::d3d8::device::d3dformat_from_wined3dformat;
pub use crate::d3d8::device::device_init;
pub use crate::d3d8::buffer::indexbuffer_init;
pub use crate::d3d8::vertexdeclaration::load_local_constants;
pub use crate::d3d8::vertexdeclaration::parse_token;
pub use crate::d3d8::texture::unsafe_impl_from_IDirect3DBaseTexture8;
pub use crate::d3d8::buffer::unsafe_impl_from_IDirect3DIndexBuffer8;
pub use crate::d3d8::surface::unsafe_impl_from_IDirect3DSurface8;
pub use crate::d3d8::buffer::unsafe_impl_from_IDirect3DVertexBuffer8;
pub use crate::d3d8::buffer::vertexbuffer_init;
pub use crate::d3d8::volume::volume_init;
pub use crate::d3d8::device::wined3dformat_from_d3dformat;
pub use crate::d3d8::device::wined3dmapflags_from_d3dmapflags;
// ==== end cross-TU definitions ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! D3DPS_VERSION { ($p0:expr, $p1:expr $(,)?) => { ((0xFFFF0000u32 | ($p0 << 8)) | $p1) }; } // gen:4b6922084b76
#[macro_export] macro_rules! D3DVSD_END { () => { 0xFFFFFFFFu32 }; } // gen:0c04424f48bf
#[macro_export] macro_rules! D3DVS_VERSION { ($p0:expr, $p1:expr $(,)?) => { ((0xFFFE0000u32 | ($p0 << 8)) | $p1) }; } // gen:143c24ebf58c
#[macro_export] macro_rules! max { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 > $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; } // gen:29bb71d79dfd
#[macro_export] macro_rules! min { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 < $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; } // gen:fb1d8a90940a
pub const D3DERR_DEVICELOST: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2152) as c_uint))) as HRESULT); // gen:186ac49170f0
pub const D3DERR_DEVICENOTRESET: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2153) as c_uint))) as HRESULT); // gen:22a2962c5cdd
pub const D3DERR_INVALIDCALL: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2156) as c_uint))) as HRESULT); // gen:fa040802d037
pub const D3DERR_MOREDATA: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2151) as c_uint))) as HRESULT); // gen:23e0a265c5d9
pub const D3DERR_NOTAVAILABLE: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2154) as c_uint))) as HRESULT); // gen:2085985dde39
pub const D3DERR_NOTFOUND: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2150) as c_uint))) as HRESULT); // gen:fcbd44af2cdd
pub const D3DERR_OUTOFVIDEOMEMORY: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((380) as c_uint))) as HRESULT); // gen:6725e1216541
pub const D3D_OK: HRESULT = ((0) as HRESULT); // gen:f736e8837e47
pub const E_FAIL: HRESULT = ((0x80004005u32) as HRESULT); // gen:1275c1cb9e8d
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT); // gen:cd8a52171603
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT); // gen:80daa5b2ffa6
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT); // gen:98bee37aa995
#[inline] // gen:9d090cdec1c2
pub unsafe fn IDirect3D8_AddRef(This: *mut IDirect3D8) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:a300078fe5e1
pub unsafe fn IDirect3D8_GetAdapterDisplayMode(This: *mut IDirect3D8, Adapter: UINT, pMode: *mut D3DDISPLAYMODE) -> HRESULT { ((*(*This).lpVtbl).GetAdapterDisplayMode.unwrap_unchecked())(This, Adapter, pMode) } // widl C wrapper
#[inline] // gen:cdfc4b3d2df5
pub unsafe fn IDirect3D8_QueryInterface(This: *mut IDirect3D8, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:f593a79c8246
pub unsafe fn IDirect3D8_Release(This: *mut IDirect3D8) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:709edac398c8
pub unsafe fn IDirect3DBaseTexture8_AddRef(This: *mut IDirect3DBaseTexture8) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:53a78eac50b7
pub unsafe fn IDirect3DBaseTexture8_GetDevice(This: *mut IDirect3DBaseTexture8, ppDevice: *mut *mut IDirect3DDevice8) -> HRESULT { ((*(*This).lpVtbl).GetDevice.unwrap_unchecked())(This, ppDevice) } // widl C wrapper
#[inline] // gen:08500c7ee8a5
pub unsafe fn IDirect3DBaseTexture8_GetType(This: *mut IDirect3DBaseTexture8) -> D3DRESOURCETYPE { ((*(*This).lpVtbl).GetType.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:13c99a572321
pub unsafe fn IDirect3DBaseTexture8_QueryInterface(This: *mut IDirect3DBaseTexture8, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:6c0159a5a655
pub unsafe fn IDirect3DBaseTexture8_Release(This: *mut IDirect3DBaseTexture8) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:bcd8bf37de35
pub unsafe fn IDirect3DCubeTexture8_AddRef(This: *mut IDirect3DCubeTexture8) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:60789af6a2ee
pub unsafe fn IDirect3DDevice8_AddRef(This: *mut IDirect3DDevice8) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:e2306274237d
pub unsafe fn IDirect3DDevice8_Release(This: *mut IDirect3DDevice8) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:e1550f074395
pub unsafe fn IDirect3DDevice8_SetPixelShader(This: *mut IDirect3DDevice8, Handle: DWORD) -> HRESULT { ((*(*This).lpVtbl).SetPixelShader.unwrap_unchecked())(This, Handle) } // widl C wrapper
#[inline] // gen:bc19a9d746f3
pub unsafe fn IDirect3DDevice8_SetVertexShader(This: *mut IDirect3DDevice8, Handle: DWORD) -> HRESULT { ((*(*This).lpVtbl).SetVertexShader.unwrap_unchecked())(This, Handle) } // widl C wrapper
#[inline] // gen:831135c90473
pub unsafe fn IDirect3DIndexBuffer8_AddRef(This: *mut IDirect3DIndexBuffer8) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:0689fa09cafc
pub unsafe fn IDirect3DSurface8_AddRef(This: *mut IDirect3DSurface8) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:58578e9a7f09
pub unsafe fn IDirect3DSurface8_GetDesc(This: *mut IDirect3DSurface8, pDesc: *mut D3DSURFACE_DESC) -> HRESULT { ((*(*This).lpVtbl).GetDesc.unwrap_unchecked())(This, pDesc) } // widl C wrapper
#[inline] // gen:e451c69c8f1f
pub unsafe fn IDirect3DSurface8_LockRect(This: *mut IDirect3DSurface8, locked_rect: *mut D3DLOCKED_RECT, rect: *const RECT, flags: DWORD) -> HRESULT { ((*(*This).lpVtbl).LockRect.unwrap_unchecked())(This, locked_rect, rect, flags) } // widl C wrapper
#[inline] // gen:2711a21fccc4
pub unsafe fn IDirect3DSurface8_UnlockRect(This: *mut IDirect3DSurface8) -> HRESULT { ((*(*This).lpVtbl).UnlockRect.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:d37d7befef37
pub unsafe fn IDirect3DSwapChain8_AddRef(This: *mut IDirect3DSwapChain8) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:6ec95a288ef5
pub unsafe fn IDirect3DSwapChain8_Present(This: *mut IDirect3DSwapChain8, src_rect: *const RECT, dst_rect: *const RECT, dst_window_override: HWND, dirty_region: *const RGNDATA) -> HRESULT { ((*(*This).lpVtbl).Present.unwrap_unchecked())(This, src_rect, dst_rect, dst_window_override, dirty_region) } // widl C wrapper
#[inline] // gen:22d98126a37d
pub unsafe fn IDirect3DSwapChain8_Release(This: *mut IDirect3DSwapChain8) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:2bce1f99f76a
pub unsafe fn IDirect3DTexture8_AddRef(This: *mut IDirect3DTexture8) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:941e460fc2f3
pub unsafe fn IDirect3DTexture8_Release(This: *mut IDirect3DTexture8) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:1793a8437400
pub unsafe fn IDirect3DVertexBuffer8_AddRef(This: *mut IDirect3DVertexBuffer8) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:a73d2f8f925b
pub unsafe fn IDirect3DVolume8_AddRef(This: *mut IDirect3DVolume8) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:082673d5d43e
pub unsafe fn IDirect3DVolume8_LockBox(This: *mut IDirect3DVolume8, locked_box: *mut D3DLOCKED_BOX, box_: *const D3DBOX, flags: DWORD) -> HRESULT { ((*(*This).lpVtbl).LockBox.unwrap_unchecked())(This, locked_box, box_, flags) } // widl C wrapper
#[inline] // gen:fb54364dfb81
pub unsafe fn IDirect3DVolume8_UnlockBox(This: *mut IDirect3DVolume8) -> HRESULT { ((*(*This).lpVtbl).UnlockBox.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:f44229808f52
pub unsafe fn IDirect3DVolumeTexture8_AddRef(This: *mut IDirect3DVolumeTexture8) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:6ad0b58c268c
pub unsafe fn IUnknown_AddRef(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:38acc17a31e7
pub unsafe fn IUnknown_QueryInterface(This: *mut IUnknown, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
pub type REFGUID = *const GUID; // gen:4c78e76f64d6
pub type REFIID = *const IID; // gen:c79168ffaea6
pub const S_FALSE: HRESULT = ((1) as HRESULT); // gen:b1b8213e7dac
pub const S_OK: HRESULT = ((0) as HRESULT); // gen:9969a2c48b97
pub const WINED3DERR_INVALIDCALL: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2156) as c_uint))) as HRESULT); // gen:71d0f13a48b5
pub const WINED3DERR_NOTAVAILABLE: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2154) as c_uint))) as HRESULT); // gen:1ecf2da0a957
pub const WINED3DERR_UNSUPPORTEDTEXTUREFILTER: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2082) as c_uint))) as HRESULT); // gen:1384eaa43fe5
pub const WINED3D_OK: HRESULT = ((0) as HRESULT); // gen:7d6ca58d383b
pub const WINEDDERR_NOTLOCKED: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((584) as c_uint))) as HRESULT); // gen:3fc3eebb09ca
pub use crate::{D3DPS_VERSION, D3DVSD_END, D3DVS_VERSION, max, min};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/directx/wine/d3d8/d3d8_private.h:257: header-inline `d3d8_texture_get_draw_texture` // gen:7fc6231ab17a
#[allow(dead_code)]
#[inline]
pub unsafe fn d3d8_texture_get_draw_texture(texture: *mut d3d8_texture) -> *mut wined3d_texture {
    if !(*texture).draw_texture.is_null() {
        (*texture).draw_texture
    } else {
        (*texture).wined3d_texture
    }
}
// dll/directx/wine/d3d8/d3d8_private.h:387: header-inline `d3dmultisample_type_from_wined3d` // gen:ed48645823c1
#[allow(dead_code)]
#[inline]
pub unsafe fn d3dmultisample_type_from_wined3d(r#type: wined3d_multisample_type) -> D3DMULTISAMPLE_TYPE {
    r#type as D3DMULTISAMPLE_TYPE
}
// dll/directx/wine/d3d8/d3d8_private.h:315: header-inline `d3dpool_from_wined3daccess` // gen:bdc10c195b0a
#[allow(dead_code)]
#[inline]
pub unsafe fn d3dpool_from_wined3daccess(access: u32, usage: u32) -> D3DPOOL {
    if (usage & WINED3DUSAGE_MANAGED) != 0 {
        return D3DPOOL_MANAGED;
    }
    let a = access & (WINED3D_RESOURCE_ACCESS_GPU | WINED3D_RESOURCE_ACCESS_CPU);
    if a == WINED3D_RESOURCE_ACCESS_CPU {
        if (usage & WINED3DUSAGE_SCRATCH) != 0 {
            D3DPOOL_SCRATCH
        } else {
            D3DPOOL_SYSTEMMEM
        }
    } else if a == (WINED3D_RESOURCE_ACCESS_GPU | WINED3D_RESOURCE_ACCESS_CPU) {
        D3DPOOL_MANAGED
    } else {
        D3DPOOL_DEFAULT
    }
}
// dll/directx/wine/d3d8/d3d8_private.h:305: header-inline `d3dusage_from_wined3dusage` // gen:f1682dfec8f7
#[allow(dead_code)]
#[inline]
pub unsafe fn d3dusage_from_wined3dusage(wined3d_usage: u32, bind_flags: u32) -> DWORD {
    let mut usage: DWORD = (wined3d_usage & WINED3DUSAGE_MASK) as DWORD;
    if (bind_flags & WINED3D_BIND_RENDER_TARGET) != 0 {
        usage |= D3DUSAGE_RENDERTARGET;
    }
    if (bind_flags & WINED3D_BIND_DEPTH_STENCIL) != 0 {
        usage |= D3DUSAGE_DEPTHSTENCIL;
    }
    usage
}
// dll/directx/wine/d3d8/d3d8_private.h:144: header-inline `impl_from_IDirect3DDevice8` // gen:e62fd0ccc481
#[allow(dead_code)]
#[inline]
pub unsafe fn impl_from_IDirect3DDevice8(iface: *mut IDirect3DDevice8) -> *mut d3d8_device {
    CONTAINING_RECORD!(iface, d3d8_device, IDirect3DDevice8_iface)
}
// dll/directx/wine/d3d8/d3d8_private.h:334: header-inline `map_access_from_usage` // gen:0e6869f97378
#[allow(dead_code)]
#[inline]
pub unsafe fn map_access_from_usage(usage: u32) -> u32 {
    if (usage & D3DUSAGE_WRITEONLY) != 0 {
        return WINED3D_RESOURCE_ACCESS_MAP_W;
    }
    WINED3D_RESOURCE_ACCESS_MAP_R | WINED3D_RESOURCE_ACCESS_MAP_W
}
// dll/directx/wine/d3d8/d3d8_private.h:375: header-inline `wined3d_bind_flags_from_d3d8_usage` // gen:aada748cf978
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_bind_flags_from_d3d8_usage(usage: DWORD) -> u32 {
    let mut bind_flags: u32 = 0;
    if (usage & D3DUSAGE_RENDERTARGET) != 0 {
        bind_flags |= WINED3D_BIND_RENDER_TARGET;
    }
    if (usage & D3DUSAGE_DEPTHSTENCIL) != 0 {
        bind_flags |= WINED3D_BIND_DEPTH_STENCIL;
    }
    bind_flags
}
// dll/directx/wine/d3d8/d3d8_private.h:392: header-inline `wined3d_device_type_from_d3d` // gen:e23620d3b8e8
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_device_type_from_d3d(r#type: D3DDEVTYPE) -> wined3d_device_type {
    r#type as wined3d_device_type
}
// dll/directx/wine/d3d8/d3d8_private.h:365: header-inline `wined3d_usage_from_d3d` // gen:2cceaea63f1d
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3d_usage_from_d3d(pool: D3DPOOL, usage: DWORD) -> u32 {
    let mut usage = usage & WINED3DUSAGE_MASK;
    if pool == D3DPOOL_SCRATCH {
        usage |= WINED3DUSAGE_SCRATCH;
    } else if pool == D3DPOOL_MANAGED {
        usage |= WINED3DUSAGE_MANAGED;
    }
    usage | WINED3DUSAGE_VIDMEM_ACCOUNTING
}
// dll/directx/wine/d3d8/d3d8_private.h:341: header-inline `wined3daccess_from_d3dpool` // gen:ce9544dd82cb
#[allow(dead_code)]
#[inline]
pub unsafe fn wined3daccess_from_d3dpool(pool: D3DPOOL, usage: u32) -> u32 {
    let mut access: u32 = match pool {
        D3DPOOL_DEFAULT => WINED3D_RESOURCE_ACCESS_GPU,
        D3DPOOL_MANAGED => WINED3D_RESOURCE_ACCESS_GPU | WINED3D_RESOURCE_ACCESS_CPU,
        D3DPOOL_SYSTEMMEM | D3DPOOL_SCRATCH => WINED3D_RESOURCE_ACCESS_CPU,
        _ => 0,
    };
    if pool != D3DPOOL_DEFAULT || (usage & D3DUSAGE_DYNAMIC) != 0 {
        access |= map_access_from_usage(usage);
    }
    access
}
// ==== end port macros ====
