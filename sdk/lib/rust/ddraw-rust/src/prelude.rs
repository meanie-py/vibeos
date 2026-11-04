//! ddraw-rust prelude — the single import surface the translated modules use.
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
pub const D3DDD_BCLIPPING: u32 = 16; // win32k-sys/src/bindgen_amd64.rs // gen:7e1162139982
pub const D3DDD_COLORMODEL: u32 = 1; // win32k-sys/src/bindgen_amd64.rs // gen:996651a406b8
pub const D3DDD_DEVCAPS: u32 = 2; // win32k-sys/src/bindgen_amd64.rs // gen:d832030f24b2
pub const D3DDD_DEVICERENDERBITDEPTH: u32 = 128; // win32k-sys/src/bindgen_amd64.rs // gen:87bdfc650d0e
pub const D3DDD_DEVICEZBUFFERBITDEPTH: u32 = 256; // win32k-sys/src/bindgen_amd64.rs // gen:d778372bac86
pub const D3DDD_LIGHTINGCAPS: u32 = 8; // win32k-sys/src/bindgen_amd64.rs // gen:3418254c9621
pub const D3DDD_LINECAPS: u32 = 32; // win32k-sys/src/bindgen_amd64.rs // gen:e75d6467e2ea
pub const D3DDD_MAXBUFFERSIZE: u32 = 512; // win32k-sys/src/bindgen_amd64.rs // gen:8091a120a24b
pub const D3DDD_MAXVERTEXCOUNT: u32 = 1024; // win32k-sys/src/bindgen_amd64.rs // gen:3ec5915c79fa
pub const D3DDD_TRANSFORMCAPS: u32 = 4; // win32k-sys/src/bindgen_amd64.rs // gen:e2b4bb64705c
pub const D3DDD_TRICAPS: u32 = 64; // win32k-sys/src/bindgen_amd64.rs // gen:3e6490fce95c
pub const D3DDEB_BUFSIZE: u32 = 1; // win32k-sys/src/bindgen_amd64.rs // gen:3c871fcd1ef5
pub const D3DDEB_LPDATA: u32 = 4; // win32k-sys/src/bindgen_amd64.rs // gen:51c5673737f8
pub const D3DDEVCAPS_CANBLTSYSTONONLOCAL: u32 = 131072; // win32k-sys/src/bindgen_amd64.rs // gen:d76ef358d4fe
pub const D3DDEVCAPS_CANRENDERAFTERFLIP: u32 = 2048; // win32k-sys/src/bindgen_amd64.rs // gen:b643d9cad7d2
pub const D3DDEVCAPS_DRAWPRIMITIVES2: u32 = 8192; // win32k-sys/src/bindgen_amd64.rs // gen:88eb72481c1e
pub const D3DDEVCAPS_DRAWPRIMITIVES2EX: u32 = 32768; // win32k-sys/src/bindgen_amd64.rs // gen:f3abd5482146
pub const D3DDEVCAPS_DRAWPRIMTLVERTEX: u32 = 1024; // win32k-sys/src/bindgen_amd64.rs // gen:8209653d26ab
pub const D3DDEVCAPS_EXECUTESYSTEMMEMORY: u32 = 16; // win32k-sys/src/bindgen_amd64.rs // gen:223c9fa0b56f
pub const D3DDEVCAPS_EXECUTEVIDEOMEMORY: u32 = 32; // win32k-sys/src/bindgen_amd64.rs // gen:668d5da64401
pub const D3DDEVCAPS_FLOATTLVERTEX: u32 = 1; // win32k-sys/src/bindgen_amd64.rs // gen:4d334474c5d3
pub const D3DDEVCAPS_HWRASTERIZATION: u32 = 524288; // win32k-sys/src/bindgen_amd64.rs // gen:f55652b74e8e
pub const D3DDEVCAPS_HWTRANSFORMANDLIGHT: u32 = 65536; // win32k-sys/src/bindgen_amd64.rs // gen:55045da4b216
pub const D3DDEVCAPS_SEPARATETEXTUREMEMORIES: u32 = 16384; // win32k-sys/src/bindgen_amd64.rs // gen:90dac2d7d959
pub const D3DDEVCAPS_SORTDECREASINGZ: u32 = 4; // win32k-sys/src/bindgen_amd64.rs // gen:0be336b53632
pub const D3DDEVCAPS_SORTEXACT: u32 = 8; // win32k-sys/src/bindgen_amd64.rs // gen:4ced27a76f81
pub const D3DDEVCAPS_SORTINCREASINGZ: u32 = 2; // win32k-sys/src/bindgen_amd64.rs // gen:aa2459302731
pub const D3DDEVCAPS_TEXTURENONLOCALVIDMEM: u32 = 4096; // win32k-sys/src/bindgen_amd64.rs // gen:77ed7e886773
pub const D3DDEVCAPS_TEXTURESYSTEMMEMORY: u32 = 256; // win32k-sys/src/bindgen_amd64.rs // gen:8a77a7a81bd7
pub const D3DDEVCAPS_TEXTUREVIDEOMEMORY: u32 = 512; // win32k-sys/src/bindgen_amd64.rs // gen:552f1921dcef
pub const D3DDEVCAPS_TLVERTEXSYSTEMMEMORY: u32 = 64; // win32k-sys/src/bindgen_amd64.rs // gen:a610bd24e677
pub const D3DDEVCAPS_TLVERTEXVIDEOMEMORY: u32 = 128; // win32k-sys/src/bindgen_amd64.rs // gen:1e0e68c2dec6
pub const D3DFDS_COLORMODEL: u32 = 1; // win32k-sys/src/bindgen_amd64.rs // gen:2024b1d03c82
pub const D3DFDS_GUID: u32 = 2; // win32k-sys/src/bindgen_amd64.rs // gen:b8168f3c089e
pub const D3DFILTER_LINEAR: D3DTEXTUREFILTER = D3DTEXTUREFILTER_D3DFILTER_LINEAR; // island enum // gen:9e6dc7162cbf
pub const D3DFILTER_LINEARMIPLINEAR: D3DTEXTUREFILTER = D3DTEXTUREFILTER_D3DFILTER_LINEARMIPLINEAR; // island enum // gen:c14b781479bc
pub const D3DFILTER_LINEARMIPNEAREST: D3DTEXTUREFILTER = D3DTEXTUREFILTER_D3DFILTER_LINEARMIPNEAREST; // island enum // gen:0fa857713aa1
pub const D3DFILTER_MIPLINEAR: D3DTEXTUREFILTER = D3DTEXTUREFILTER_D3DFILTER_MIPLINEAR; // island enum // gen:97374b4e5dcb
pub const D3DFILTER_MIPNEAREST: D3DTEXTUREFILTER = D3DTEXTUREFILTER_D3DFILTER_MIPNEAREST; // island enum // gen:a78193f6aabe
pub const D3DFILTER_NEAREST: D3DTEXTUREFILTER = D3DTEXTUREFILTER_D3DFILTER_NEAREST; // island enum // gen:1f8c0f85f9e7
pub const D3DLIGHTCAPS_DIRECTIONAL: u32 = 4; // win32k-sys/src/bindgen_amd64.rs // gen:3f24f5b8f6c7
pub const D3DLIGHTCAPS_PARALLELPOINT: u32 = 8; // win32k-sys/src/bindgen_amd64.rs // gen:69d484d1ef94
pub const D3DLIGHTCAPS_POINT: u32 = 1; // win32k-sys/src/bindgen_amd64.rs // gen:cf92cd0babd8
pub const D3DLIGHTCAPS_SPOT: u32 = 2; // win32k-sys/src/bindgen_amd64.rs // gen:7314e4a4cc9a
pub const D3DLIGHTINGMODEL_RGB: u32 = 1; // win32k-sys/src/bindgen_amd64.rs // gen:2e7bfb59e3d4
pub const D3DLIGHTSTATE_AMBIENT: D3DLIGHTSTATETYPE = D3DLIGHTSTATETYPE_D3DLIGHTSTATE_AMBIENT; // island enum // gen:2db9237355ad
pub const D3DLIGHTSTATE_COLORMODEL: D3DLIGHTSTATETYPE = D3DLIGHTSTATETYPE_D3DLIGHTSTATE_COLORMODEL; // island enum // gen:7abbda0e28c2
pub const D3DLIGHTSTATE_COLORVERTEX: D3DLIGHTSTATETYPE = D3DLIGHTSTATETYPE_D3DLIGHTSTATE_COLORVERTEX; // island enum // gen:b0064a4792f1
pub const D3DLIGHTSTATE_FOGDENSITY: D3DLIGHTSTATETYPE = D3DLIGHTSTATETYPE_D3DLIGHTSTATE_FOGDENSITY; // island enum // gen:bb37f3b81538
pub const D3DLIGHTSTATE_FOGEND: D3DLIGHTSTATETYPE = D3DLIGHTSTATETYPE_D3DLIGHTSTATE_FOGEND; // island enum // gen:53e95cdfc227
pub const D3DLIGHTSTATE_FOGMODE: D3DLIGHTSTATETYPE = D3DLIGHTSTATETYPE_D3DLIGHTSTATE_FOGMODE; // island enum // gen:a2c3fbeb5bd0
pub const D3DLIGHTSTATE_FOGSTART: D3DLIGHTSTATETYPE = D3DLIGHTSTATETYPE_D3DLIGHTSTATE_FOGSTART; // island enum // gen:86e089529a68
pub const D3DLIGHTSTATE_MATERIAL: D3DLIGHTSTATETYPE = D3DLIGHTSTATETYPE_D3DLIGHTSTATE_MATERIAL; // island enum // gen:eb9bbea6a9b9
pub const D3DLIGHT_PARALLELPOINT: D3DLIGHTTYPE = D3DLIGHTTYPE_D3DLIGHT_PARALLELPOINT; // island enum // gen:d82b5ce05ee3
pub const D3DOP_BRANCHFORWARD: _D3DOPCODE = _D3DOPCODE_D3DOP_BRANCHFORWARD; // island enum // gen:cba48530e0c9
pub const D3DOP_EXIT: _D3DOPCODE = _D3DOPCODE_D3DOP_EXIT; // island enum // gen:c1d3fef8aaf8
pub const D3DOP_LINE: _D3DOPCODE = _D3DOPCODE_D3DOP_LINE; // island enum // gen:27caa2600e0d
pub const D3DOP_MATRIXLOAD: _D3DOPCODE = _D3DOPCODE_D3DOP_MATRIXLOAD; // island enum // gen:058ed8c69381
pub const D3DOP_MATRIXMULTIPLY: _D3DOPCODE = _D3DOPCODE_D3DOP_MATRIXMULTIPLY; // island enum // gen:9efcfac899d2
pub const D3DOP_POINT: _D3DOPCODE = _D3DOPCODE_D3DOP_POINT; // island enum // gen:1c8b84fb7250
pub const D3DOP_PROCESSVERTICES: _D3DOPCODE = _D3DOPCODE_D3DOP_PROCESSVERTICES; // island enum // gen:c7eef40c9a90
pub const D3DOP_SETSTATUS: _D3DOPCODE = _D3DOPCODE_D3DOP_SETSTATUS; // island enum // gen:5db8c9946635
pub const D3DOP_SPAN: _D3DOPCODE = _D3DOPCODE_D3DOP_SPAN; // island enum // gen:29e742a3a17a
pub const D3DOP_STATELIGHT: _D3DOPCODE = _D3DOPCODE_D3DOP_STATELIGHT; // island enum // gen:3cc13865834c
pub const D3DOP_STATERENDER: _D3DOPCODE = _D3DOPCODE_D3DOP_STATERENDER; // island enum // gen:4fb362f07375
pub const D3DOP_STATETRANSFORM: _D3DOPCODE = _D3DOPCODE_D3DOP_STATETRANSFORM; // island enum // gen:193f91048d29
pub const D3DOP_TEXTURELOAD: _D3DOPCODE = _D3DOPCODE_D3DOP_TEXTURELOAD; // island enum // gen:0d8e84452072
pub const D3DOP_TRIANGLE: _D3DOPCODE = _D3DOPCODE_D3DOP_TRIANGLE; // island enum // gen:fbedd72b5172
pub const D3DPBLENDCAPS_BOTHINVSRCALPHA: u32 = 4096; // win32k-sys/src/bindgen_amd64.rs // gen:2d0284c54b1c
pub const D3DPBLENDCAPS_BOTHSRCALPHA: u32 = 2048; // win32k-sys/src/bindgen_amd64.rs // gen:cda3b95a8329
pub const D3DPBLENDCAPS_DESTALPHA: u32 = 64; // win32k-sys/src/bindgen_amd64.rs // gen:d83e1c9855ec
pub const D3DPBLENDCAPS_DESTCOLOR: u32 = 256; // win32k-sys/src/bindgen_amd64.rs // gen:b019f4a9902f
pub const D3DPBLENDCAPS_INVDESTALPHA: u32 = 128; // win32k-sys/src/bindgen_amd64.rs // gen:455e752112d0
pub const D3DPBLENDCAPS_INVDESTCOLOR: u32 = 512; // win32k-sys/src/bindgen_amd64.rs // gen:287a2621762b
pub const D3DPBLENDCAPS_INVSRCALPHA: u32 = 32; // win32k-sys/src/bindgen_amd64.rs // gen:eb96ddbc271d
pub const D3DPBLENDCAPS_INVSRCCOLOR: u32 = 8; // win32k-sys/src/bindgen_amd64.rs // gen:305abc8777c0
pub const D3DPBLENDCAPS_ONE: u32 = 2; // win32k-sys/src/bindgen_amd64.rs // gen:e5416aad539b
pub const D3DPBLENDCAPS_SRCALPHA: u32 = 16; // win32k-sys/src/bindgen_amd64.rs // gen:22e1ebc6d8b2
pub const D3DPBLENDCAPS_SRCALPHASAT: u32 = 1024; // win32k-sys/src/bindgen_amd64.rs // gen:c7a955b55d11
pub const D3DPBLENDCAPS_SRCCOLOR: u32 = 4; // win32k-sys/src/bindgen_amd64.rs // gen:3c05888cc83b
pub const D3DPBLENDCAPS_ZERO: u32 = 1; // win32k-sys/src/bindgen_amd64.rs // gen:4d4ee6207724
pub const D3DPCMPCAPS_ALWAYS: u32 = 128; // win32k-sys/src/bindgen_amd64.rs // gen:91c90a347c4b
pub const D3DPCMPCAPS_EQUAL: u32 = 4; // win32k-sys/src/bindgen_amd64.rs // gen:ab1a1409ae72
pub const D3DPCMPCAPS_GREATER: u32 = 16; // win32k-sys/src/bindgen_amd64.rs // gen:c37ddcc92cee
pub const D3DPCMPCAPS_GREATEREQUAL: u32 = 64; // win32k-sys/src/bindgen_amd64.rs // gen:f23ac2a24a27
pub const D3DPCMPCAPS_LESS: u32 = 2; // win32k-sys/src/bindgen_amd64.rs // gen:c4985f1093a0
pub const D3DPCMPCAPS_LESSEQUAL: u32 = 8; // win32k-sys/src/bindgen_amd64.rs // gen:d7132e5a1e95
pub const D3DPCMPCAPS_NEVER: u32 = 1; // win32k-sys/src/bindgen_amd64.rs // gen:0f3bbfbafa91
pub const D3DPCMPCAPS_NOTEQUAL: u32 = 32; // win32k-sys/src/bindgen_amd64.rs // gen:06cf8ab61d49
pub const D3DPMISCCAPS_CONFORMANT: u32 = 8; // win32k-sys/src/bindgen_amd64.rs // gen:53b530c9532a
pub const D3DPMISCCAPS_CULLCCW: u32 = 64; // win32k-sys/src/bindgen_amd64.rs // gen:8cf0ff7822e9
pub const D3DPMISCCAPS_CULLCW: u32 = 32; // win32k-sys/src/bindgen_amd64.rs // gen:c3095d9e6ad0
pub const D3DPMISCCAPS_CULLNONE: u32 = 16; // win32k-sys/src/bindgen_amd64.rs // gen:8eeb5b4343c4
pub const D3DPMISCCAPS_LINEPATTERNREP: u32 = 4; // win32k-sys/src/bindgen_amd64.rs // gen:78b9cf375a47
pub const D3DPMISCCAPS_MASKPLANES: u32 = 1; // win32k-sys/src/bindgen_amd64.rs // gen:568636ac00eb
pub const D3DPMISCCAPS_MASKZ: u32 = 2; // win32k-sys/src/bindgen_amd64.rs // gen:4b621288d769
pub const D3DPRASTERCAPS_ANISOTROPY: u32 = 131072; // win32k-sys/src/bindgen_amd64.rs // gen:bc777ef58bcc
pub const D3DPRASTERCAPS_ANTIALIASEDGES: u32 = 4096; // win32k-sys/src/bindgen_amd64.rs // gen:756b82a8752a
pub const D3DPRASTERCAPS_ANTIALIASSORTDEPENDENT: u32 = 1024; // win32k-sys/src/bindgen_amd64.rs // gen:0494f503aa24
pub const D3DPRASTERCAPS_ANTIALIASSORTINDEPENDENT: u32 = 2048; // win32k-sys/src/bindgen_amd64.rs // gen:330d07f906f6
pub const D3DPRASTERCAPS_DITHER: u32 = 1; // win32k-sys/src/bindgen_amd64.rs // gen:7a2f9961a412
pub const D3DPRASTERCAPS_FOGRANGE: u32 = 65536; // win32k-sys/src/bindgen_amd64.rs // gen:323eaf546335
pub const D3DPRASTERCAPS_FOGTABLE: u32 = 256; // win32k-sys/src/bindgen_amd64.rs // gen:1ecbedfc61ac
pub const D3DPRASTERCAPS_FOGVERTEX: u32 = 128; // win32k-sys/src/bindgen_amd64.rs // gen:fae71cadc293
pub const D3DPRASTERCAPS_MIPMAPLODBIAS: u32 = 8192; // win32k-sys/src/bindgen_amd64.rs // gen:3af433a77505
pub const D3DPRASTERCAPS_PAT: u32 = 8; // win32k-sys/src/bindgen_amd64.rs // gen:94a3906b2d99
pub const D3DPRASTERCAPS_ROP2: u32 = 2; // win32k-sys/src/bindgen_amd64.rs // gen:8a55d48666ba
pub const D3DPRASTERCAPS_STIPPLE: u32 = 512; // win32k-sys/src/bindgen_amd64.rs // gen:fe3bdb51928f
pub const D3DPRASTERCAPS_SUBPIXEL: u32 = 32; // win32k-sys/src/bindgen_amd64.rs // gen:198bd5155843
pub const D3DPRASTERCAPS_SUBPIXELX: u32 = 64; // win32k-sys/src/bindgen_amd64.rs // gen:e6be409aeedc
pub const D3DPRASTERCAPS_TRANSLUCENTSORTINDEPENDENT: u32 = 524288; // win32k-sys/src/bindgen_amd64.rs // gen:dba7a23f91d8
pub const D3DPRASTERCAPS_WBUFFER: u32 = 262144; // win32k-sys/src/bindgen_amd64.rs // gen:2f6d1a45e2e3
pub const D3DPRASTERCAPS_WFOG: u32 = 1048576; // win32k-sys/src/bindgen_amd64.rs // gen:15c046223ac1
pub const D3DPRASTERCAPS_XOR: u32 = 4; // win32k-sys/src/bindgen_amd64.rs // gen:d434e28294fa
pub const D3DPRASTERCAPS_ZBIAS: u32 = 16384; // win32k-sys/src/bindgen_amd64.rs // gen:06f05b982d5f
pub const D3DPRASTERCAPS_ZBUFFERLESSHSR: u32 = 32768; // win32k-sys/src/bindgen_amd64.rs // gen:2c0a39fce312
pub const D3DPRASTERCAPS_ZFOG: u32 = 2097152; // win32k-sys/src/bindgen_amd64.rs // gen:894ab550966a
pub const D3DPRASTERCAPS_ZTEST: u32 = 16; // win32k-sys/src/bindgen_amd64.rs // gen:a977483b7af2
pub const D3DPSHADECAPS_ALPHAFLATBLEND: u32 = 4096; // win32k-sys/src/bindgen_amd64.rs // gen:58abed07d47d
pub const D3DPSHADECAPS_ALPHAFLATSTIPPLED: u32 = 8192; // win32k-sys/src/bindgen_amd64.rs // gen:cf4f5a5a1cba
pub const D3DPSHADECAPS_ALPHAGOURAUDBLEND: u32 = 16384; // win32k-sys/src/bindgen_amd64.rs // gen:77e507802976
pub const D3DPSHADECAPS_ALPHAGOURAUDSTIPPLED: u32 = 32768; // win32k-sys/src/bindgen_amd64.rs // gen:3c6efafacf60
pub const D3DPSHADECAPS_ALPHAPHONGBLEND: u32 = 65536; // win32k-sys/src/bindgen_amd64.rs // gen:021ca735e08b
pub const D3DPSHADECAPS_ALPHAPHONGSTIPPLED: u32 = 131072; // win32k-sys/src/bindgen_amd64.rs // gen:4583e6acc101
pub const D3DPSHADECAPS_COLORFLATMONO: u32 = 1; // win32k-sys/src/bindgen_amd64.rs // gen:708d85e5de01
pub const D3DPSHADECAPS_COLORFLATRGB: u32 = 2; // win32k-sys/src/bindgen_amd64.rs // gen:266859c44f2e
pub const D3DPSHADECAPS_COLORGOURAUDMONO: u32 = 4; // win32k-sys/src/bindgen_amd64.rs // gen:63467908668d
pub const D3DPSHADECAPS_COLORGOURAUDRGB: u32 = 8; // win32k-sys/src/bindgen_amd64.rs // gen:c836a36f41d3
pub const D3DPSHADECAPS_COLORPHONGMONO: u32 = 16; // win32k-sys/src/bindgen_amd64.rs // gen:8af118f89191
pub const D3DPSHADECAPS_COLORPHONGRGB: u32 = 32; // win32k-sys/src/bindgen_amd64.rs // gen:526ec2bfc288
pub const D3DPSHADECAPS_FOGFLAT: u32 = 262144; // win32k-sys/src/bindgen_amd64.rs // gen:c331dd24bef1
pub const D3DPSHADECAPS_FOGGOURAUD: u32 = 524288; // win32k-sys/src/bindgen_amd64.rs // gen:dbf5a2683369
pub const D3DPSHADECAPS_FOGPHONG: u32 = 1048576; // win32k-sys/src/bindgen_amd64.rs // gen:cd5ed34e533e
pub const D3DPSHADECAPS_SPECULARFLATMONO: u32 = 64; // win32k-sys/src/bindgen_amd64.rs // gen:4e3c605977a4
pub const D3DPSHADECAPS_SPECULARFLATRGB: u32 = 128; // win32k-sys/src/bindgen_amd64.rs // gen:e83c6b2ebd36
pub const D3DPSHADECAPS_SPECULARGOURAUDMONO: u32 = 256; // win32k-sys/src/bindgen_amd64.rs // gen:6b08a586a210
pub const D3DPSHADECAPS_SPECULARGOURAUDRGB: u32 = 512; // win32k-sys/src/bindgen_amd64.rs // gen:a3e8a872ebcf
pub const D3DPSHADECAPS_SPECULARPHONGMONO: u32 = 1024; // win32k-sys/src/bindgen_amd64.rs // gen:52b5cb892be6
pub const D3DPSHADECAPS_SPECULARPHONGRGB: u32 = 2048; // win32k-sys/src/bindgen_amd64.rs // gen:e3ed742d6627
pub const D3DPTADDRESSCAPS_BORDER: u32 = 8; // win32k-sys/src/bindgen_amd64.rs // gen:70b2d78cec91
pub const D3DPTADDRESSCAPS_CLAMP: u32 = 4; // win32k-sys/src/bindgen_amd64.rs // gen:7fa6d87ef722
pub const D3DPTADDRESSCAPS_INDEPENDENTUV: u32 = 16; // win32k-sys/src/bindgen_amd64.rs // gen:f675a31cc96f
pub const D3DPTADDRESSCAPS_MIRROR: u32 = 2; // win32k-sys/src/bindgen_amd64.rs // gen:bd649ef11cc7
pub const D3DPTADDRESSCAPS_WRAP: u32 = 1; // win32k-sys/src/bindgen_amd64.rs // gen:503f51c6ff61
pub const D3DPTBLENDCAPS_ADD: u32 = 128; // win32k-sys/src/bindgen_amd64.rs // gen:26f40e11079c
pub const D3DPTBLENDCAPS_COPY: u32 = 64; // win32k-sys/src/bindgen_amd64.rs // gen:f585fb2d09c8
pub const D3DPTBLENDCAPS_DECAL: u32 = 1; // win32k-sys/src/bindgen_amd64.rs // gen:276b5b66cda8
pub const D3DPTBLENDCAPS_DECALALPHA: u32 = 4; // win32k-sys/src/bindgen_amd64.rs // gen:5a9376fd1743
pub const D3DPTBLENDCAPS_DECALMASK: u32 = 16; // win32k-sys/src/bindgen_amd64.rs // gen:bd76f86c2386
pub const D3DPTBLENDCAPS_MODULATE: u32 = 2; // win32k-sys/src/bindgen_amd64.rs // gen:4f07dea582e6
pub const D3DPTBLENDCAPS_MODULATEALPHA: u32 = 8; // win32k-sys/src/bindgen_amd64.rs // gen:d78eb8e90464
pub const D3DPTBLENDCAPS_MODULATEMASK: u32 = 32; // win32k-sys/src/bindgen_amd64.rs // gen:9a313afb9056
pub const D3DPTEXTURECAPS_ALPHA: u32 = 4; // win32k-sys/src/bindgen_amd64.rs // gen:41914ef14da8
pub const D3DPTEXTURECAPS_ALPHAPALETTE: u32 = 128; // win32k-sys/src/bindgen_amd64.rs // gen:b6bd12a0b9e0
pub const D3DPTEXTURECAPS_BORDER: u32 = 16; // win32k-sys/src/bindgen_amd64.rs // gen:de3c44d8680a
pub const D3DPTEXTURECAPS_COLORKEYBLEND: u32 = 4096; // win32k-sys/src/bindgen_amd64.rs // gen:07e71e21734d
pub const D3DPTEXTURECAPS_CUBEMAP: u32 = 2048; // win32k-sys/src/bindgen_amd64.rs // gen:62b9a002b9fe
pub const D3DPTEXTURECAPS_PERSPECTIVE: u32 = 1; // win32k-sys/src/bindgen_amd64.rs // gen:e7b7ba86374f
pub const D3DPTEXTURECAPS_POW2: u32 = 2; // win32k-sys/src/bindgen_amd64.rs // gen:bf9842a8811f
pub const D3DPTEXTURECAPS_PROJECTED: u32 = 1024; // win32k-sys/src/bindgen_amd64.rs // gen:079a1c534fc4
pub const D3DPTEXTURECAPS_SQUAREONLY: u32 = 32; // win32k-sys/src/bindgen_amd64.rs // gen:8c88b1a55102
pub const D3DPTEXTURECAPS_TEXREPEATNOTSCALEDBYSIZE: u32 = 64; // win32k-sys/src/bindgen_amd64.rs // gen:23ca534b05dd
pub const D3DPTEXTURECAPS_TRANSPARENCY: u32 = 8; // win32k-sys/src/bindgen_amd64.rs // gen:ddf101c58459
pub const D3DPTFILTERCAPS_LINEAR: u32 = 2; // win32k-sys/src/bindgen_amd64.rs // gen:1b0eb236a2fc
pub const D3DPTFILTERCAPS_LINEARMIPLINEAR: u32 = 32; // win32k-sys/src/bindgen_amd64.rs // gen:8247a79552bf
pub const D3DPTFILTERCAPS_LINEARMIPNEAREST: u32 = 16; // win32k-sys/src/bindgen_amd64.rs // gen:f5342d1c9fe8
pub const D3DPTFILTERCAPS_MAGFAFLATCUBIC: u32 = 134217728; // win32k-sys/src/bindgen_amd64.rs // gen:e89f77bd8790
pub const D3DPTFILTERCAPS_MAGFANISOTROPIC: u32 = 67108864; // win32k-sys/src/bindgen_amd64.rs // gen:5e133e27b315
pub const D3DPTFILTERCAPS_MAGFGAUSSIANCUBIC: u32 = 268435456; // win32k-sys/src/bindgen_amd64.rs // gen:5438731baffe
pub const D3DPTFILTERCAPS_MAGFLINEAR: u32 = 33554432; // win32k-sys/src/bindgen_amd64.rs // gen:2a5add588062
pub const D3DPTFILTERCAPS_MAGFPOINT: u32 = 16777216; // win32k-sys/src/bindgen_amd64.rs // gen:9e4889534c41
pub const D3DPTFILTERCAPS_MINFANISOTROPIC: u32 = 1024; // win32k-sys/src/bindgen_amd64.rs // gen:9aa6ca372442
pub const D3DPTFILTERCAPS_MINFLINEAR: u32 = 512; // win32k-sys/src/bindgen_amd64.rs // gen:81e45084586f
pub const D3DPTFILTERCAPS_MINFPOINT: u32 = 256; // win32k-sys/src/bindgen_amd64.rs // gen:51f5d73afaed
pub const D3DPTFILTERCAPS_MIPFLINEAR: u32 = 131072; // win32k-sys/src/bindgen_amd64.rs // gen:21bd08475fe2
pub const D3DPTFILTERCAPS_MIPFPOINT: u32 = 65536; // win32k-sys/src/bindgen_amd64.rs // gen:aee281c56aed
pub const D3DPTFILTERCAPS_MIPLINEAR: u32 = 8; // win32k-sys/src/bindgen_amd64.rs // gen:a8c070ad00c4
pub const D3DPTFILTERCAPS_MIPNEAREST: u32 = 4; // win32k-sys/src/bindgen_amd64.rs // gen:7076515e58ef
pub const D3DPTFILTERCAPS_NEAREST: u32 = 1; // win32k-sys/src/bindgen_amd64.rs // gen:92fe131a222c
pub const D3DRENDERSTATE_AMBIENT: D3DRENDERSTATETYPE = D3DRENDERSTATETYPE_D3DRENDERSTATE_AMBIENT; // island enum // gen:c455e9f9611e
pub const D3DRENDERSTATE_BORDERCOLOR: D3DRENDERSTATETYPE = D3DRENDERSTATETYPE_D3DRENDERSTATE_BORDERCOLOR; // island enum // gen:f72bb0edb4fb
pub const D3DRENDERSTATE_CLIPPLANEENABLE: D3DRENDERSTATETYPE = D3DRENDERSTATETYPE_D3DRENDERSTATE_CLIPPLANEENABLE; // island enum // gen:63f51745f0e1
pub const D3DRENDERSTATE_COLORVERTEX: D3DRENDERSTATETYPE = D3DRENDERSTATETYPE_D3DRENDERSTATE_COLORVERTEX; // island enum // gen:a4cad061904f
pub const D3DRENDERSTATE_EDGEANTIALIAS: D3DRENDERSTATETYPE = D3DRENDERSTATETYPE_D3DRENDERSTATE_EDGEANTIALIAS; // island enum // gen:b76e6b06e57b
pub const D3DRENDERSTATE_FOGDENSITY: D3DRENDERSTATETYPE = D3DRENDERSTATETYPE_D3DRENDERSTATE_FOGDENSITY; // island enum // gen:ee86aad19ad5
pub const D3DRENDERSTATE_FOGEND: D3DRENDERSTATETYPE = D3DRENDERSTATETYPE_D3DRENDERSTATE_FOGEND; // island enum // gen:6573b6253db0
pub const D3DRENDERSTATE_FOGSTART: D3DRENDERSTATETYPE = D3DRENDERSTATETYPE_D3DRENDERSTATE_FOGSTART; // island enum // gen:13cbc33f8eb9
pub const D3DRENDERSTATE_FOGVERTEXMODE: D3DRENDERSTATETYPE = D3DRENDERSTATETYPE_D3DRENDERSTATE_FOGVERTEXMODE; // island enum // gen:a4e358e3778a
pub const D3DRENDERSTATE_LIGHTING: D3DRENDERSTATETYPE = D3DRENDERSTATETYPE_D3DRENDERSTATE_LIGHTING; // island enum // gen:f94fd764709a
pub const D3DRENDERSTATE_LOCALVIEWER: D3DRENDERSTATETYPE = D3DRENDERSTATETYPE_D3DRENDERSTATE_LOCALVIEWER; // island enum // gen:7894735928d0
pub const D3DRENDERSTATE_NORMALIZENORMALS: D3DRENDERSTATETYPE = D3DRENDERSTATETYPE_D3DRENDERSTATE_NORMALIZENORMALS; // island enum // gen:340a7a3aed4a
pub const D3DRENDERSTATE_STIPPLEPATTERN00: D3DRENDERSTATETYPE = D3DRENDERSTATETYPE_D3DRENDERSTATE_STIPPLEPATTERN00; // island enum // gen:2dbee18e6b30
pub const D3DRENDERSTATE_STIPPLEPATTERN31: D3DRENDERSTATETYPE = D3DRENDERSTATETYPE_D3DRENDERSTATE_STIPPLEPATTERN31; // island enum // gen:791fad369433
pub const D3DRENDERSTATE_TEXTUREADDRESS: D3DRENDERSTATETYPE = D3DRENDERSTATETYPE_D3DRENDERSTATE_TEXTUREADDRESS; // island enum // gen:d008e1d44e26
pub const D3DRENDERSTATE_TEXTUREADDRESSU: D3DRENDERSTATETYPE = D3DRENDERSTATETYPE_D3DRENDERSTATE_TEXTUREADDRESSU; // island enum // gen:a9c795e4fbb0
pub const D3DRENDERSTATE_TEXTUREADDRESSV: D3DRENDERSTATETYPE = D3DRENDERSTATETYPE_D3DRENDERSTATE_TEXTUREADDRESSV; // island enum // gen:f39ff5bfd827
pub const D3DRENDERSTATE_TEXTUREHANDLE: D3DRENDERSTATETYPE = D3DRENDERSTATETYPE_D3DRENDERSTATE_TEXTUREHANDLE; // island enum // gen:097d6fe13414
pub const D3DRENDERSTATE_TEXTUREMAG: D3DRENDERSTATETYPE = D3DRENDERSTATETYPE_D3DRENDERSTATE_TEXTUREMAG; // island enum // gen:487a180131fa
pub const D3DRENDERSTATE_TEXTUREMAPBLEND: D3DRENDERSTATETYPE = D3DRENDERSTATETYPE_D3DRENDERSTATE_TEXTUREMAPBLEND; // island enum // gen:94a4534c4de8
pub const D3DRENDERSTATE_TEXTUREMIN: D3DRENDERSTATETYPE = D3DRENDERSTATETYPE_D3DRENDERSTATE_TEXTUREMIN; // island enum // gen:a792d98ebf0b
pub const D3DRENDERSTATE_ZBIAS: D3DRENDERSTATETYPE = D3DRENDERSTATETYPE_D3DRENDERSTATE_ZBIAS; // island enum // gen:ac8f4d18f885
pub const D3DSBT_ALL: _D3DSTATEBLOCKTYPE = _D3DSTATEBLOCKTYPE_D3DSBT_ALL; // island enum // gen:277d830bb7b7
pub const D3DSBT_PIXELSTATE: _D3DSTATEBLOCKTYPE = _D3DSTATEBLOCKTYPE_D3DSBT_PIXELSTATE; // island enum // gen:0c55033475dd
pub const D3DSBT_VERTEXSTATE: _D3DSTATEBLOCKTYPE = _D3DSTATEBLOCKTYPE_D3DSBT_VERTEXSTATE; // island enum // gen:94123895a665
pub const D3DSTENCILCAPS_DECR: u32 = 128; // win32k-sys/src/bindgen_amd64.rs // gen:7132fb696128
pub const D3DSTENCILCAPS_DECRSAT: u32 = 16; // win32k-sys/src/bindgen_amd64.rs // gen:acb878d41eae
pub const D3DSTENCILCAPS_INCR: u32 = 64; // win32k-sys/src/bindgen_amd64.rs // gen:ef6d26c7ba51
pub const D3DSTENCILCAPS_INCRSAT: u32 = 8; // win32k-sys/src/bindgen_amd64.rs // gen:446853a2523b
pub const D3DSTENCILCAPS_INVERT: u32 = 32; // win32k-sys/src/bindgen_amd64.rs // gen:f18e3228d122
pub const D3DSTENCILCAPS_KEEP: u32 = 1; // win32k-sys/src/bindgen_amd64.rs // gen:1dac66c91b6d
pub const D3DSTENCILCAPS_REPLACE: u32 = 4; // win32k-sys/src/bindgen_amd64.rs // gen:4924a5d657b6
pub const D3DSTENCILCAPS_ZERO: u32 = 2; // win32k-sys/src/bindgen_amd64.rs // gen:1ed5cbd6a9b2
pub const D3DTBLEND_ADD: D3DTEXTUREBLEND = D3DTEXTUREBLEND_D3DTBLEND_ADD; // island enum // gen:f8ddb76907ff
pub const D3DTBLEND_COPY: D3DTEXTUREBLEND = D3DTEXTUREBLEND_D3DTBLEND_COPY; // island enum // gen:a34b710608e5
pub const D3DTBLEND_DECAL: D3DTEXTUREBLEND = D3DTEXTUREBLEND_D3DTBLEND_DECAL; // island enum // gen:12f5af88cf6c
pub const D3DTBLEND_DECALALPHA: D3DTEXTUREBLEND = D3DTEXTUREBLEND_D3DTBLEND_DECALALPHA; // island enum // gen:c081587a3dc4
pub const D3DTBLEND_MODULATE: D3DTEXTUREBLEND = D3DTEXTUREBLEND_D3DTBLEND_MODULATE; // island enum // gen:7b4ce547312f
pub const D3DTBLEND_MODULATEALPHA: D3DTEXTUREBLEND = D3DTEXTUREBLEND_D3DTBLEND_MODULATEALPHA; // island enum // gen:0f93dde28798
pub const D3DTEXOPCAPS_ADD: u32 = 64; // win32k-sys/src/bindgen_amd64.rs // gen:85e2d13da8da
pub const D3DTEXOPCAPS_ADDSIGNED2X: u32 = 256; // win32k-sys/src/bindgen_amd64.rs // gen:3825cf6e5cd5
pub const D3DTEXOPCAPS_ADDSIGNED: u32 = 128; // win32k-sys/src/bindgen_amd64.rs // gen:2f1ccb0dcb57
pub const D3DTEXOPCAPS_ADDSMOOTH: u32 = 1024; // win32k-sys/src/bindgen_amd64.rs // gen:d32a63765cf8
pub const D3DTEXOPCAPS_BLENDCURRENTALPHA: u32 = 32768; // win32k-sys/src/bindgen_amd64.rs // gen:2d563c3bfdd0
pub const D3DTEXOPCAPS_BLENDFACTORALPHA: u32 = 8192; // win32k-sys/src/bindgen_amd64.rs // gen:e5bcd5dae622
pub const D3DTEXOPCAPS_BLENDTEXTUREALPHA: u32 = 4096; // win32k-sys/src/bindgen_amd64.rs // gen:6b6a48730c39
pub const D3DTEXOPCAPS_BLENDTEXTUREALPHAPM: u32 = 16384; // win32k-sys/src/bindgen_amd64.rs // gen:e3ff5625dbc3
pub const D3DTEXOPCAPS_BUMPENVMAP: u32 = 2097152; // win32k-sys/src/bindgen_amd64.rs // gen:f565fab90230
pub const D3DTEXOPCAPS_BUMPENVMAPLUMINANCE: u32 = 4194304; // win32k-sys/src/bindgen_amd64.rs // gen:83bc816db6a2
pub const D3DTEXOPCAPS_DISABLE: u32 = 1; // win32k-sys/src/bindgen_amd64.rs // gen:063f6df53bc0
pub const D3DTEXOPCAPS_DOTPRODUCT3: u32 = 8388608; // win32k-sys/src/bindgen_amd64.rs // gen:8f51ab84032d
pub const D3DTEXOPCAPS_MODULATE2X: u32 = 16; // win32k-sys/src/bindgen_amd64.rs // gen:19173a976268
pub const D3DTEXOPCAPS_MODULATE4X: u32 = 32; // win32k-sys/src/bindgen_amd64.rs // gen:09ee64e9ff81
pub const D3DTEXOPCAPS_MODULATE: u32 = 8; // win32k-sys/src/bindgen_amd64.rs // gen:94fecfdbad91
pub const D3DTEXOPCAPS_MODULATEALPHA_ADDCOLOR: u32 = 131072; // win32k-sys/src/bindgen_amd64.rs // gen:d39493c6e507
pub const D3DTEXOPCAPS_MODULATECOLOR_ADDALPHA: u32 = 262144; // win32k-sys/src/bindgen_amd64.rs // gen:1a40d1ad2068
pub const D3DTEXOPCAPS_MODULATEINVALPHA_ADDCOLOR: u32 = 524288; // win32k-sys/src/bindgen_amd64.rs // gen:badc1eac4b9a
pub const D3DTEXOPCAPS_MODULATEINVCOLOR_ADDALPHA: u32 = 1048576; // win32k-sys/src/bindgen_amd64.rs // gen:377c02475372
pub const D3DTEXOPCAPS_PREMODULATE: u32 = 65536; // win32k-sys/src/bindgen_amd64.rs // gen:1b646a37f67c
pub const D3DTEXOPCAPS_SELECTARG1: u32 = 2; // win32k-sys/src/bindgen_amd64.rs // gen:b08dc3472be9
pub const D3DTEXOPCAPS_SELECTARG2: u32 = 4; // win32k-sys/src/bindgen_amd64.rs // gen:053326021ed1
pub const D3DTEXOPCAPS_SUBTRACT: u32 = 512; // win32k-sys/src/bindgen_amd64.rs // gen:4713e2299376
pub const D3DTFG_ANISOTROPIC: _D3DTEXTUREMAGFILTER = _D3DTEXTUREMAGFILTER_D3DTFG_ANISOTROPIC; // island enum // gen:4fa816139185
pub const D3DTFG_FLATCUBIC: _D3DTEXTUREMAGFILTER = _D3DTEXTUREMAGFILTER_D3DTFG_FLATCUBIC; // island enum // gen:c8528df290c4
pub const D3DTFG_GAUSSIANCUBIC: _D3DTEXTUREMAGFILTER = _D3DTEXTUREMAGFILTER_D3DTFG_GAUSSIANCUBIC; // island enum // gen:2cf957de32ed
pub const D3DTFG_LINEAR: _D3DTEXTUREMAGFILTER = _D3DTEXTUREMAGFILTER_D3DTFG_LINEAR; // island enum // gen:2be428469639
pub const D3DTFG_POINT: _D3DTEXTUREMAGFILTER = _D3DTEXTUREMAGFILTER_D3DTFG_POINT; // island enum // gen:aa9208d26cba
pub const D3DTFP_LINEAR: _D3DTEXTUREMIPFILTER = _D3DTEXTUREMIPFILTER_D3DTFP_LINEAR; // island enum // gen:08c4e490e05b
pub const D3DTFP_NONE: _D3DTEXTUREMIPFILTER = _D3DTEXTUREMIPFILTER_D3DTFP_NONE; // island enum // gen:0e1690c2888e
pub const D3DTFP_POINT: _D3DTEXTUREMIPFILTER = _D3DTEXTUREMIPFILTER_D3DTFP_POINT; // island enum // gen:6676f98c33e1
pub const D3DTRANSFORMSTATE_PROJECTION: _D3DTRANSFORMSTATETYPE = _D3DTRANSFORMSTATETYPE_D3DTRANSFORMSTATE_PROJECTION; // island enum // gen:51ba7e40a17d
pub const D3DTRANSFORMSTATE_VIEW: _D3DTRANSFORMSTATETYPE = _D3DTRANSFORMSTATETYPE_D3DTRANSFORMSTATE_VIEW; // island enum // gen:596e517c35e8
pub const D3DTRANSFORMSTATE_WORLD1: _D3DTRANSFORMSTATETYPE = _D3DTRANSFORMSTATETYPE_D3DTRANSFORMSTATE_WORLD1; // island enum // gen:4cd2ff58435c
pub const D3DTRANSFORMSTATE_WORLD2: _D3DTRANSFORMSTATETYPE = _D3DTRANSFORMSTATETYPE_D3DTRANSFORMSTATE_WORLD2; // island enum // gen:b260508b5efb
pub const D3DTRANSFORMSTATE_WORLD3: _D3DTRANSFORMSTATETYPE = _D3DTRANSFORMSTATETYPE_D3DTRANSFORMSTATE_WORLD3; // island enum // gen:d22ace787baf
pub const D3DTRANSFORMSTATE_WORLD: _D3DTRANSFORMSTATETYPE = _D3DTRANSFORMSTATETYPE_D3DTRANSFORMSTATE_WORLD; // island enum // gen:bd8cdaa75659
pub const D3DTSS_ADDRESS: _D3DTEXTURESTAGESTATETYPE = _D3DTEXTURESTAGESTATETYPE_D3DTSS_ADDRESS; // island enum // gen:fcf1898fa257
pub const D3DTSS_MAGFILTER: _D3DTEXTURESTAGESTATETYPE = _D3DTEXTURESTAGESTATETYPE_D3DTSS_MAGFILTER; // island enum // gen:7c83f8d17d6d
pub const D3DTSS_MIPFILTER: _D3DTEXTURESTAGESTATETYPE = _D3DTEXTURESTAGESTATETYPE_D3DTSS_MIPFILTER; // island enum // gen:71365f495569
pub const D3DTSS_TEXTURETRANSFORMFLAGS: _D3DTEXTURESTAGESTATETYPE = _D3DTEXTURESTAGESTATETYPE_D3DTSS_TEXTURETRANSFORMFLAGS; // island enum // gen:d634de7a65b5
pub const D3DVTXPCAPS_DIRECTIONALLIGHTS: u32 = 8; // win32k-sys/src/bindgen_amd64.rs // gen:e40d3d3e3647
pub const D3DVTXPCAPS_LOCALVIEWER: u32 = 32; // win32k-sys/src/bindgen_amd64.rs // gen:8d27f61632e2
pub const D3DVTXPCAPS_MATERIALSOURCE7: u32 = 2; // win32k-sys/src/bindgen_amd64.rs // gen:dfcdfebf92ba
pub const D3DVTXPCAPS_POSITIONALLIGHTS: u32 = 16; // win32k-sys/src/bindgen_amd64.rs // gen:96731e74737c
pub const D3DVTXPCAPS_TEXGEN: u32 = 1; // win32k-sys/src/bindgen_amd64.rs // gen:342c71c6af16
pub const D3DVTXPCAPS_VERTEXFOG: u32 = 4; // win32k-sys/src/bindgen_amd64.rs // gen:ee778d40fc12
pub const D3DVT_LVERTEX: D3DVERTEXTYPE = D3DVERTEXTYPE_D3DVT_LVERTEX; // island enum // gen:bb55734edbc3
pub const D3DVT_TLVERTEX: D3DVERTEXTYPE = D3DVERTEXTYPE_D3DVT_TLVERTEX; // island enum // gen:42b21e911309
pub const D3DVT_VERTEX: D3DVERTEXTYPE = D3DVERTEXTYPE_D3DVT_VERTEX; // island enum // gen:c5e4b7e04762
pub const DDRAW_DEVICE_STATE_LOST: ddraw_device_state = ddraw_device_state_DDRAW_DEVICE_STATE_LOST; // island enum // gen:6d85378cf217
pub const DDRAW_DEVICE_STATE_NOT_RESTORED: ddraw_device_state = ddraw_device_state_DDRAW_DEVICE_STATE_NOT_RESTORED; // island enum // gen:878093e27d7c
pub const DDRAW_DEVICE_STATE_OK: ddraw_device_state = ddraw_device_state_DDRAW_DEVICE_STATE_OK; // island enum // gen:8d1fab865d1f
pub const DDRAW_HANDLE_FREE: ddraw_handle_type = ddraw_handle_type_DDRAW_HANDLE_FREE; // island enum // gen:ed113962fe2b
pub const DDRAW_HANDLE_MATERIAL: ddraw_handle_type = ddraw_handle_type_DDRAW_HANDLE_MATERIAL; // island enum // gen:c689b5d31ec4
pub const DDRAW_HANDLE_MATRIX: ddraw_handle_type = ddraw_handle_type_DDRAW_HANDLE_MATRIX; // island enum // gen:6c27401b1a44
pub const DDRAW_HANDLE_STATEBLOCK: ddraw_handle_type = ddraw_handle_type_DDRAW_HANDLE_STATEBLOCK; // island enum // gen:2e7f17d716df
pub const DDRAW_HANDLE_SURFACE: ddraw_handle_type = ddraw_handle_type_DDRAW_HANDLE_SURFACE; // island enum // gen:9c5dab4a8f27
pub const DDRAW_VIEWPORT_VERSION_1: ddraw_viewport_version = ddraw_viewport_version_DDRAW_VIEWPORT_VERSION_1; // island enum // gen:3a51d02c0e4c
pub const DDRAW_VIEWPORT_VERSION_2: ddraw_viewport_version = ddraw_viewport_version_DDRAW_VIEWPORT_VERSION_2; // island enum // gen:e319f8234569
pub const DDRAW_VIEWPORT_VERSION_NONE: ddraw_viewport_version = ddraw_viewport_version_DDRAW_VIEWPORT_VERSION_NONE; // island enum // gen:e115ed938226
pub const UINT_MAX: u32 = 4294967295; // comctl32-rust/src/prelude.rs // gen:1b06153cd4a1
pub const USHRT_MAX: u32 = 65535; // calc-rust/src/prelude.rs // gen:bd2e945a04d2
pub const WINED3DFMT_A8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_A8_UNORM; // island enum // gen:b852344850b7
pub const WINED3DFMT_ATI2N: wined3d_format_id = wined3d_format_id_WINED3DFMT_ATI2N; // island enum // gen:a09342789cf1
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
pub const WINED3DFMT_D16_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_D16_UNORM; // island enum // gen:9c5a906a0784
pub const WINED3DFMT_D24_UNORM_S8_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_D24_UNORM_S8_UINT; // island enum // gen:9527094adfe6
pub const WINED3DFMT_D32_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_D32_UNORM; // island enum // gen:fac0db7c6301
pub const WINED3DFMT_DXT1: wined3d_format_id = wined3d_format_id_WINED3DFMT_DXT1; // island enum // gen:553a6c265db3
pub const WINED3DFMT_DXT2: wined3d_format_id = wined3d_format_id_WINED3DFMT_DXT2; // island enum // gen:82bd4645ec25
pub const WINED3DFMT_DXT3: wined3d_format_id = wined3d_format_id_WINED3DFMT_DXT3; // island enum // gen:187a70eb9414
pub const WINED3DFMT_DXT4: wined3d_format_id = wined3d_format_id_WINED3DFMT_DXT4; // island enum // gen:4f2fb5f8a31e
pub const WINED3DFMT_DXT5: wined3d_format_id = wined3d_format_id_WINED3DFMT_DXT5; // island enum // gen:bf70eb5d2476
pub const WINED3DFMT_G8R8_G8B8: wined3d_format_id = wined3d_format_id_WINED3DFMT_G8R8_G8B8; // island enum // gen:cd1b2c0c8c7e
pub const WINED3DFMT_L4A4_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_L4A4_UNORM; // island enum // gen:7c9feb6fdd9c
pub const WINED3DFMT_L8A8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_L8A8_UNORM; // island enum // gen:621b76d8945e
pub const WINED3DFMT_L8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_L8_UNORM; // island enum // gen:385347c438e3
pub const WINED3DFMT_MULTI2_ARGB8: wined3d_format_id = wined3d_format_id_WINED3DFMT_MULTI2_ARGB8; // island enum // gen:d42d21bf71ce
pub const WINED3DFMT_NVHS: wined3d_format_id = wined3d_format_id_WINED3DFMT_NVHS; // island enum // gen:07bc4e14b527
pub const WINED3DFMT_NVHU: wined3d_format_id = wined3d_format_id_WINED3DFMT_NVHU; // island enum // gen:32b7173dc692
pub const WINED3DFMT_P8_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_P8_UINT; // island enum // gen:08be2f398d7b
pub const WINED3DFMT_R10G10B10_SNORM_A2_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R10G10B10_SNORM_A2_UNORM; // island enum // gen:408936d9d5be
pub const WINED3DFMT_R10G11B11_SNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R10G11B11_SNORM; // island enum // gen:2d160d053c41
pub const WINED3DFMT_R16_UINT: wined3d_format_id = wined3d_format_id_WINED3DFMT_R16_UINT; // island enum // gen:4ce044980323
pub const WINED3DFMT_R5G5_SNORM_L6_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R5G5_SNORM_L6_UNORM; // island enum // gen:86cc0bb8f6b0
pub const WINED3DFMT_R8G8B8X8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8B8X8_UNORM; // island enum // gen:b63b4e348a0e
pub const WINED3DFMT_R8G8_B8G8: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8_B8G8; // island enum // gen:0ef20fcc1e92
pub const WINED3DFMT_R8G8_SNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8_SNORM; // island enum // gen:2315347f9119
pub const WINED3DFMT_R8G8_SNORM_L8X8_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_R8G8_SNORM_L8X8_UNORM; // island enum // gen:d3e02c96ba02
pub const WINED3DFMT_S1_UINT_D15_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_S1_UINT_D15_UNORM; // island enum // gen:a396b379f935
pub const WINED3DFMT_S4X4_UINT_D24_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_S4X4_UINT_D24_UNORM; // island enum // gen:6fe6c428ea21
pub const WINED3DFMT_UNKNOWN: wined3d_format_id = wined3d_format_id_WINED3DFMT_UNKNOWN; // island enum // gen:93d9074cc710
pub const WINED3DFMT_UYVY: wined3d_format_id = wined3d_format_id_WINED3DFMT_UYVY; // island enum // gen:d3667707ec52
pub const WINED3DFMT_X8D24_UNORM: wined3d_format_id = wined3d_format_id_WINED3DFMT_X8D24_UNORM; // island enum // gen:b81fd5c118b7
pub const WINED3DFMT_YUY2: wined3d_format_id = wined3d_format_id_WINED3DFMT_YUY2; // island enum // gen:a2c4bf4f1ef3
pub const WINED3DFMT_YV12: wined3d_format_id = wined3d_format_id_WINED3DFMT_YV12; // island enum // gen:99688803f1b7
pub const WINED3D_CUBEMAP_FACE_NEGATIVE_X: wined3d_cubemap_face = wined3d_cubemap_face_WINED3D_CUBEMAP_FACE_NEGATIVE_X; // island enum // gen:2cf689846463
pub const WINED3D_CUBEMAP_FACE_NEGATIVE_Y: wined3d_cubemap_face = wined3d_cubemap_face_WINED3D_CUBEMAP_FACE_NEGATIVE_Y; // island enum // gen:e3ac4c5ecec8
pub const WINED3D_CUBEMAP_FACE_NEGATIVE_Z: wined3d_cubemap_face = wined3d_cubemap_face_WINED3D_CUBEMAP_FACE_NEGATIVE_Z; // island enum // gen:917d9163e997
pub const WINED3D_CUBEMAP_FACE_POSITIVE_X: wined3d_cubemap_face = wined3d_cubemap_face_WINED3D_CUBEMAP_FACE_POSITIVE_X; // island enum // gen:0281705c1c64
pub const WINED3D_CUBEMAP_FACE_POSITIVE_Y: wined3d_cubemap_face = wined3d_cubemap_face_WINED3D_CUBEMAP_FACE_POSITIVE_Y; // island enum // gen:90c17e6a9218
pub const WINED3D_CUBEMAP_FACE_POSITIVE_Z: wined3d_cubemap_face = wined3d_cubemap_face_WINED3D_CUBEMAP_FACE_POSITIVE_Z; // island enum // gen:f43d8d2a9b10
pub const WINED3D_DEVICE_TYPE_HAL: wined3d_device_type = wined3d_device_type_WINED3D_DEVICE_TYPE_HAL; // island enum // gen:d73e66f36b9d
pub const WINED3D_DEVICE_TYPE_NULLREF: wined3d_device_type = wined3d_device_type_WINED3D_DEVICE_TYPE_NULLREF; // island enum // gen:0e375cf3960a
pub const WINED3D_DEVICE_TYPE_REF: wined3d_device_type = wined3d_device_type_WINED3D_DEVICE_TYPE_REF; // island enum // gen:c435a2c3156e
pub const WINED3D_DEVICE_TYPE_SW: wined3d_device_type = wined3d_device_type_WINED3D_DEVICE_TYPE_SW; // island enum // gen:3ce94f2a146f
pub const WINED3D_FEATURE_LEVEL_5: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_5; // island enum // gen:f9c53f3d677f
pub const WINED3D_FEATURE_LEVEL_6: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_6; // island enum // gen:e970439b7cb3
pub const WINED3D_FEATURE_LEVEL_7: wined3d_feature_level = wined3d_feature_level_WINED3D_FEATURE_LEVEL_7; // island enum // gen:03cd701a1439
pub const WINED3D_MULTISAMPLE_NONE: wined3d_multisample_type = wined3d_multisample_type_WINED3D_MULTISAMPLE_NONE; // island enum // gen:bfa66121fcaa
pub const WINED3D_PT_LINELIST: wined3d_primitive_type = wined3d_primitive_type_WINED3D_PT_LINELIST; // island enum // gen:7ae764f5dcb1
pub const WINED3D_PT_POINTLIST: wined3d_primitive_type = wined3d_primitive_type_WINED3D_PT_POINTLIST; // island enum // gen:e83778ace339
pub const WINED3D_PT_TRIANGLELIST: wined3d_primitive_type = wined3d_primitive_type_WINED3D_PT_TRIANGLELIST; // island enum // gen:ca4ffe7bc6b4
pub const WINED3D_RS_ANTIALIASEDLINEENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_ANTIALIASEDLINEENABLE; // island enum // gen:a1d64f8207d7
pub const WINED3D_RS_CLIPPING: wined3d_render_state = wined3d_render_state_WINED3D_RS_CLIPPING; // island enum // gen:d55448b0a48b
pub const WINED3D_RS_COLORKEYENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_COLORKEYENABLE; // island enum // gen:c0fd7831337b
pub const WINED3D_RS_DEPTHBIAS: wined3d_render_state = wined3d_render_state_WINED3D_RS_DEPTHBIAS; // island enum // gen:b185bd45eb78
pub const WINED3D_RS_LIGHTING: wined3d_render_state = wined3d_render_state_WINED3D_RS_LIGHTING; // island enum // gen:16927db0b497
pub const WINED3D_RS_NORMALIZENORMALS: wined3d_render_state = wined3d_render_state_WINED3D_RS_NORMALIZENORMALS; // island enum // gen:a4be87cb153a
pub const WINED3D_RS_SPECULARENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_SPECULARENABLE; // island enum // gen:e1b5760c7e27
pub const WINED3D_RS_ZENABLE: wined3d_render_state = wined3d_render_state_WINED3D_RS_ZENABLE; // island enum // gen:f8bcd5a86ca6
pub const WINED3D_RTYPE_TEXTURE_2D: wined3d_resource_type = wined3d_resource_type_WINED3D_RTYPE_TEXTURE_2D; // island enum // gen:de40c3024d68
pub const WINED3D_SAMP_ADDRESS_U: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_ADDRESS_U; // island enum // gen:1edc0ba1eb48
pub const WINED3D_SAMP_ADDRESS_V: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_ADDRESS_V; // island enum // gen:863872073cda
pub const WINED3D_SAMP_BORDER_COLOR: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_BORDER_COLOR; // island enum // gen:6359d21babb4
pub const WINED3D_SAMP_MAG_FILTER: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_MAG_FILTER; // island enum // gen:1f1d5697ec8a
pub const WINED3D_SAMP_MAX_ANISOTROPY: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_MAX_ANISOTROPY; // island enum // gen:8f1c9604c9be
pub const WINED3D_SAMP_MAX_MIP_LEVEL: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_MAX_MIP_LEVEL; // island enum // gen:798c7969f2c4
pub const WINED3D_SAMP_MIN_FILTER: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_MIN_FILTER; // island enum // gen:e75251a959b9
pub const WINED3D_SAMP_MIPMAP_LOD_BIAS: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_MIPMAP_LOD_BIAS; // island enum // gen:099207089185
pub const WINED3D_SAMP_MIP_FILTER: wined3d_sampler_state = wined3d_sampler_state_WINED3D_SAMP_MIP_FILTER; // island enum // gen:118fa724e031
pub const WINED3D_SBT_ALL: wined3d_stateblock_type = wined3d_stateblock_type_WINED3D_SBT_ALL; // island enum // gen:8ff21d87a028
pub const WINED3D_SBT_PRIMARY: wined3d_stateblock_type = wined3d_stateblock_type_WINED3D_SBT_PRIMARY; // island enum // gen:a4ee8d8e4d8b
pub const WINED3D_SBT_RECORDED: wined3d_stateblock_type = wined3d_stateblock_type_WINED3D_SBT_RECORDED; // island enum // gen:a3a30ecb7bc4
pub const WINED3D_SCANLINE_ORDERING_UNKNOWN: wined3d_scanline_ordering = wined3d_scanline_ordering_WINED3D_SCANLINE_ORDERING_UNKNOWN; // island enum // gen:7774b8a98afe
pub const WINED3D_SWAP_EFFECT_DISCARD: wined3d_swap_effect = wined3d_swap_effect_WINED3D_SWAP_EFFECT_DISCARD; // island enum // gen:7f0e42311775
pub const WINED3D_TEXF_ANISOTROPIC: wined3d_texture_filter_type = wined3d_texture_filter_type_WINED3D_TEXF_ANISOTROPIC; // island enum // gen:ebd8fe7d182c
pub const WINED3D_TEXF_FLAT_CUBIC: wined3d_texture_filter_type = wined3d_texture_filter_type_WINED3D_TEXF_FLAT_CUBIC; // island enum // gen:a80008d91d10
pub const WINED3D_TEXF_GAUSSIAN_CUBIC: wined3d_texture_filter_type = wined3d_texture_filter_type_WINED3D_TEXF_GAUSSIAN_CUBIC; // island enum // gen:b239b428d4e2
pub const WINED3D_TEXF_LINEAR: wined3d_texture_filter_type = wined3d_texture_filter_type_WINED3D_TEXF_LINEAR; // island enum // gen:a0db557e9dc5
pub const WINED3D_TEXF_NONE: wined3d_texture_filter_type = wined3d_texture_filter_type_WINED3D_TEXF_NONE; // island enum // gen:68af396e0aaf
pub const WINED3D_TEXF_POINT: wined3d_texture_filter_type = wined3d_texture_filter_type_WINED3D_TEXF_POINT; // island enum // gen:db8018ba15af
pub const WINED3D_TOP_ADD: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_ADD; // island enum // gen:e2dc16a35b7f
pub const WINED3D_TOP_BLEND_TEXTURE_ALPHA: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_BLEND_TEXTURE_ALPHA; // island enum // gen:b6e095727311
pub const WINED3D_TOP_MODULATE: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_MODULATE; // island enum // gen:d4c339bb6934
pub const WINED3D_TOP_SELECT_ARG1: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_SELECT_ARG1; // island enum // gen:1db12af4f4e5
pub const WINED3D_TOP_SELECT_ARG2: wined3d_texture_op = wined3d_texture_op_WINED3D_TOP_SELECT_ARG2; // island enum // gen:a5ee1ebbb8f5
pub const WINED3D_TSS_ALPHA_ARG1: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_ALPHA_ARG1; // island enum // gen:ee76536251ec
pub const WINED3D_TSS_ALPHA_ARG2: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_ALPHA_ARG2; // island enum // gen:793c542ef090
pub const WINED3D_TSS_ALPHA_OP: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_ALPHA_OP; // island enum // gen:2439b29fc399
pub const WINED3D_TSS_BUMPENV_LOFFSET: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_BUMPENV_LOFFSET; // island enum // gen:42a822880cb8
pub const WINED3D_TSS_BUMPENV_LSCALE: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_BUMPENV_LSCALE; // island enum // gen:27feb64a5bfa
pub const WINED3D_TSS_BUMPENV_MAT00: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_BUMPENV_MAT00; // island enum // gen:cb219866bcc3
pub const WINED3D_TSS_BUMPENV_MAT01: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_BUMPENV_MAT01; // island enum // gen:5869728db43e
pub const WINED3D_TSS_BUMPENV_MAT10: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_BUMPENV_MAT10; // island enum // gen:028a69e3ab12
pub const WINED3D_TSS_BUMPENV_MAT11: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_BUMPENV_MAT11; // island enum // gen:37d61e1797ae
pub const WINED3D_TSS_COLOR_ARG1: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_COLOR_ARG1; // island enum // gen:d6cbebeefa27
pub const WINED3D_TSS_COLOR_ARG2: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_COLOR_ARG2; // island enum // gen:6952c588431b
pub const WINED3D_TSS_COLOR_OP: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_COLOR_OP; // island enum // gen:f829cae71631
pub const WINED3D_TSS_INVALID: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_INVALID; // island enum // gen:917b0a20b2b3
pub const WINED3D_TSS_TEXCOORD_INDEX: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_TEXCOORD_INDEX; // island enum // gen:8e80cbf7e4eb
pub const WINED3D_TSS_TEXTURE_TRANSFORM_FLAGS: wined3d_texture_stage_state = wined3d_texture_stage_state_WINED3D_TSS_TEXTURE_TRANSFORM_FLAGS; // island enum // gen:f9828e809b53
pub const WINED3D_TS_PROJECTION: wined3d_transform_state = wined3d_transform_state_WINED3D_TS_PROJECTION; // island enum // gen:886cceefd518
pub const WINED3D_TS_VIEW: wined3d_transform_state = wined3d_transform_state_WINED3D_TS_VIEW; // island enum // gen:3093d1a59334
pub const WINED3D_TS_WORLD: wined3d_transform_state = wined3d_transform_state_WINED3D_TS_WORLD; // island enum // gen:2ae8c4109807
pub const WINED3D_ZB_FALSE: wined3d_depth_buffer_type = wined3d_depth_buffer_type_WINED3D_ZB_FALSE; // island enum // gen:f6f39574a296
pub const WINED3D_ZB_TRUE: wined3d_depth_buffer_type = wined3d_depth_buffer_type_WINED3D_ZB_TRUE; // island enum // gen:93340c841fa2
extern "C" {
    pub fn calloc(arg1: ::core::ffi::c_uint, arg2: ::core::ffi::c_uint) -> *mut ::core::ffi::c_void; // msvcrt-sys/src/bindgen_i386.rs // gen:d3193c100ce5
    pub fn fabs(x: f64) -> f64; // calc-rust/src/prelude.rs // gen:e1ed7df7f860
    pub fn free(p: *mut c_void); // activeds-rust/src/prelude.rs // gen:0ad441c3c230
    pub fn malloc(size: usize) -> *mut c_void; // activeds-rust/src/prelude.rs // gen:86a212c6fd14
    pub fn realloc(p: *mut c_void, size: usize) -> *mut c_void; // activeds-rust/src/prelude.rs // gen:ce18447a16eb
    pub fn sqrtf(x: f32) -> f32; // cnt-rust/src/cabi/math.rs // gen:dc284b279d22
}
extern "system" {
    pub fn ClipCursor(arg1: LPCRECT) -> BOOL; // aclui-rust/src/types_i386.rs // gen:01f42bbf5e44
    pub fn CreateWindowExA( dwExStyle: DWORD, lpClassName: LPCSTR, lpWindowName: LPCSTR, dwStyle: DWORD, X: ::core::ffi::c_int, Y: ::core::ffi::c_int, nWidth: ::core::ffi::c_int, nHeight: ::core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID, ) -> HWND; // aclui-rust/src/types_i386.rs // gen:2a5bd341d50b
    pub fn DefWindowProcA(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs // gen:68bcc01160cf
    pub fn DestroyWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs // gen:b6137cacf702
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:111127023c7f
    pub fn GetDC(arg1: HWND) -> HDC; // aclui-rust/src/types_i386.rs // gen:39c5b894d19a
    pub fn GetDesktopWindow() -> HWND; // aclui-rust/src/types_i386.rs // gen:dec8495c75ca
    pub fn GetForegroundWindow() -> HWND; // aclui-rust/src/types_i386.rs // gen:e7b5ca97e8ff
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs // gen:50e246193424
    pub fn GetModuleHandleExW(arg1: DWORD, arg2: LPCWSTR, arg3: *mut HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:e24f313d56d9
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:33a0dab07dca
    pub fn GetTickCount() -> DWORD; // aclui-rust/src/types_i386.rs // gen:f624a448a1f6
    pub fn GetVersion() -> DWORD; // aclui-rust/src/types_i386.rs // gen:4652a0b3e9c8
    pub fn IsBadReadPtr(arg1: *const ::core::ffi::c_void, arg2: UINT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs // gen:8480f7ee53e5
    pub fn IsRectEmpty(arg1: LPCRECT) -> BOOL; // aclui-rust/src/types_i386.rs // gen:56044909b2d0
    pub fn RedrawWindow(arg1: HWND, arg2: LPCRECT, arg3: HRGN, arg4: UINT) -> BOOL; // aclui-rust/src/types_i386.rs // gen:64f80ad809d1
    pub fn RegisterClassA(arg1: *const WNDCLASSA) -> ATOM; // aclui-rust/src/types_i386.rs // gen:c110e5303288
    pub fn ReleaseDC(arg1: HWND, arg2: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:0952239cd2d2
    pub fn SetRect( arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:9d720f054480
    pub fn SetRectEmpty(arg1: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs // gen:24dd6a6e6d13
    pub fn SetWindowPos( arg1: HWND, arg2: HWND, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: UINT, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:bbf7b4058eae
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs // gen:a99a94e9f6ba
    pub fn UnregisterClassA(arg1: LPCSTR, arg2: HINSTANCE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:d14f916dd30c
    pub fn WideCharToMultiByte( arg1: UINT, arg2: DWORD, arg3: LPCWSTR, arg4: ::core::ffi::c_int, arg5: LPSTR, arg6: ::core::ffi::c_int, arg7: LPCSTR, arg8: LPBOOL, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs // gen:0e129f7cfff2
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::ddraw::utils::DDRAW_Convert_DDDEVICEIDENTIFIER_2_To_1;
pub use crate::ddraw::utils::DDRAW_Convert_DDSCAPS_1_To_2;
pub use crate::ddraw::utils::DDRAW_dump_DDCAPS;
pub use crate::ddraw::utils::DDRAW_dump_DDSCAPS2;
pub use crate::ddraw::utils::DDRAW_dump_cooperativelevel;
pub use crate::ddraw::utils::DDRAW_dump_surface_desc;
pub use crate::ddraw::utils::DDSD2_to_DDSD;
pub use crate::ddraw::utils::DDSD_to_DDSD2;
pub use crate::ddraw::ddraw::DirectDrawCreateClipper;
pub use crate::ddraw::device::d3d_device_apply_state;
pub use crate::ddraw::device::d3d_device_create;
pub use crate::ddraw::device::d3d_device_sync_surfaces;
pub use crate::ddraw::device::d3d_device_update_depth_stencil;
pub use crate::ddraw::executebuffer::d3d_execute_buffer_execute;
pub use crate::ddraw::executebuffer::d3d_execute_buffer_init;
pub use crate::ddraw::light::d3d_light_init;
pub use crate::ddraw::material::d3d_material_create;
pub use crate::ddraw::vertexbuffer::d3d_vertex_buffer_create;
pub use crate::ddraw::viewport::d3d_viewport_init;
pub use crate::ddraw::main::ddraw_allocate_handle;
pub use crate::ddraw::clipper::ddraw_clipper_init;
pub use crate::ddraw::clipper::ddraw_clipper_is_valid;
pub use crate::ddraw::ddraw::ddraw_d3dcaps1_from_7;
pub use crate::ddraw::ddraw::ddraw_find_decl;
pub use crate::ddraw::main::ddraw_free_handle;
pub use crate::ddraw::ddraw::ddraw_get_d3dcaps;
pub use crate::ddraw::main::ddraw_get_object;
pub use crate::ddraw::main::ddraw_handle_table_destroy;
pub use crate::ddraw::main::ddraw_handle_table_init;
pub use crate::ddraw::ddraw::ddraw_init;
pub use crate::ddraw::palette::ddraw_palette_init;
pub use crate::ddraw::surface::ddraw_surface_create;
pub use crate::ddraw::surface::ddraw_surface_get_rendertarget_view;
pub use crate::ddraw::surface::ddraw_surface_update_frontbuffer;
pub use crate::ddraw::ddraw::ddraw_update_lost_surfaces;
pub use crate::ddraw::utils::ddrawformat_from_wined3dformat;
pub use crate::ddraw::utils::dump_D3DMATRIX;
pub use crate::ddraw::utils::get_flexible_vertex_size;
pub use crate::ddraw::utils::hr_ddraw_from_wined3d;
pub use crate::ddraw::light::light_activate;
pub use crate::ddraw::light::light_deactivate;
pub use crate::ddraw::material::material_activate;
pub use crate::ddraw::utils::multiply_matrix;
pub use crate::ddraw::device::tss_lookup;
pub use crate::ddraw::device::unsafe_impl_from_IDirect3DDevice;
pub use crate::ddraw::device::unsafe_impl_from_IDirect3DDevice2;
pub use crate::ddraw::device::unsafe_impl_from_IDirect3DDevice3;
pub use crate::ddraw::device::unsafe_impl_from_IDirect3DDevice7;
pub use crate::ddraw::executebuffer::unsafe_impl_from_IDirect3DExecuteBuffer;
pub use crate::ddraw::light::unsafe_impl_from_IDirect3DLight;
pub use crate::ddraw::surface::unsafe_impl_from_IDirect3DTexture;
pub use crate::ddraw::surface::unsafe_impl_from_IDirect3DTexture2;
pub use crate::ddraw::vertexbuffer::unsafe_impl_from_IDirect3DVertexBuffer7;
pub use crate::ddraw::viewport::unsafe_impl_from_IDirect3DViewport;
pub use crate::ddraw::viewport::unsafe_impl_from_IDirect3DViewport2;
pub use crate::ddraw::viewport::unsafe_impl_from_IDirect3DViewport3;
pub use crate::ddraw::clipper::unsafe_impl_from_IDirectDrawClipper;
pub use crate::ddraw::palette::unsafe_impl_from_IDirectDrawPalette;
pub use crate::ddraw::surface::unsafe_impl_from_IDirectDrawSurface;
pub use crate::ddraw::surface::unsafe_impl_from_IDirectDrawSurface4;
pub use crate::ddraw::surface::unsafe_impl_from_IDirectDrawSurface7;
pub use crate::ddraw::viewport::viewport_activate;
pub use crate::ddraw::viewport::viewport_alloc_active_light_index;
pub use crate::ddraw::viewport::viewport_deactivate;
pub use crate::ddraw::viewport::viewport_free_active_light_index;
pub use crate::ddraw::utils::wined3d_colour_from_ddraw_colour;
pub use crate::ddraw::utils::wined3dformat_from_ddrawformat;
pub use crate::ddraw::utils::wined3dmapflags_from_ddrawmapflags;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbLengthW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};
pub use winelist_rust::{list_add_after, list_add_before, list_add_head, list_add_tail, list_count, list_empty, list_head, list_init, list_move_head, list_move_tail, list_next, list_prev, list_remove, list_tail};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const D3DPTEXTURECAPS_NONPOW2CONDITIONAL: i32 = 0x100; // clang, ddraw.c
pub const D3DTRANSFORMCAPS_CLIP: i32 = 1; // clang, ddraw.c
pub const WINED3DERR_INVALIDCALL: HRESULT = 0x8876086C_u32 as HRESULT; // clang, device.c
pub const WINED3DERR_NOTAVAILABLE: HRESULT = 0x8876086A_u32 as HRESULT; // clang, surface.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// D3DFINDDEVICERESULT: c2rust, ddraw.c
pub type D3DFINDDEVICERESULT = _D3DFINDDEVICERESULT;
// D3DFINDDEVICESEARCH: c2rust, ddraw.c
pub type D3DFINDDEVICESEARCH = _D3DFINDDEVICESEARCH;
// compare_info: c2rust, ddraw.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct compare_info {
    pub flag: DWORD,
    pub offset: ptrdiff_t,
    pub size: size_t,
}
// ddraw_class_factory: c2rust, main.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ddraw_class_factory {
    pub IClassFactory_iface: IClassFactory,
    pub ref_: LONG,
    pub pfnCreateInstance: Option<
        unsafe extern "C" fn(*mut IUnknown, *const IID, *mut *mut ::core::ffi::c_void) -> HRESULT,
    >,
}
// ddraw_find_device_result_v1: c2rust, ddraw.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ddraw_find_device_result_v1 {
    pub size: DWORD,
    pub guid: GUID,
    pub hw_desc: D3DDEVICEDESC_V1,
    pub sw_desc: D3DDEVICEDESC_V1,
}
// ddraw_find_device_result_v2: c2rust, ddraw.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ddraw_find_device_result_v2 {
    pub size: DWORD,
    pub guid: GUID,
    pub hw_desc: D3DDEVICEDESC_V2,
    pub sw_desc: D3DDEVICEDESC_V2,
}
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn __wine_dbg_get_channel_flags(channel: *mut __wine_debug_channel) -> ::core::ffi::c_uchar; // ddraw.c
    pub fn ros_dbg_log(
        cls: __wine_debug_class,
        ch: *mut __wine_debug_channel,
        file: *const ::core::ffi::c_char,
        func: *const ::core::ffi::c_char,
        line: ::core::ffi::c_int,
        format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // clipper.c
    pub fn strcpy(
        _Dest: *mut ::core::ffi::c_char,
        _Source: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char; // ddraw.c
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
pub static mut __wine_dbch_ddraw: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x64\x64\x72\x61\x77\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch_winediag: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x77\x69\x6e\x65\x64\x69\x61\x67\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch_fps: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x66\x70\x73\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_ddraw };
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

// hand-edited, so kept out of the generated block below (generated_items.py); delete to regenerate
#[macro_export] macro_rules! D3DRGBA { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { ((((((($p3 * 255.0)) as LONG) << 24) | (((($p0 * 255.0)) as LONG) << 16)) | (((($p1 * 255.0)) as LONG) << 8)) | ((($p2 * 255.0)) as LONG)) }; }
pub use crate::D3DRGBA;

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! WINED3D_TS_WORLD_MATRIX { ($p0:expr $(,)?) => { ((($p0 + 256)) as wined3d_transform_state) }; } // gen:874f862f4537
#[macro_export] macro_rules! max { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 > $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; } // gen:29bb71d79dfd
#[macro_export] macro_rules! min { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 < $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; } // gen:fb1d8a90940a
pub const CLASS_E_CLASSNOTAVAILABLE: HRESULT = ((0x80040111u32) as HRESULT); // gen:1197fd83f980
pub const CLASS_E_NOAGGREGATION: HRESULT = ((0x80040110u32) as HRESULT); // gen:01676cfea511
pub const D3DERR_INBEGINSTATEBLOCK: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2101) as c_uint))) as HRESULT); // gen:5d4d224e20bf
pub const D3DERR_INVALIDSTATEBLOCK: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2100) as c_uint))) as HRESULT); // gen:78b0765f2f46
pub const D3DERR_LIGHTHASVIEWPORT: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((751) as c_uint))) as HRESULT); // gen:8a792c0dfba6
pub const D3DERR_NOCURRENTVIEWPORT: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((775) as c_uint))) as HRESULT); // gen:95cb81bdda8a
pub const D3DERR_NOTINBEGINSTATEBLOCK: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2102) as c_uint))) as HRESULT); // gen:4775390894d2
pub const D3DERR_SCENE_IN_SCENE: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((760) as c_uint))) as HRESULT); // gen:09bc77bc88af
pub const D3DERR_SCENE_NOT_IN_SCENE: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((761) as c_uint))) as HRESULT); // gen:26c2ff316f2a
pub const D3DERR_SURFACENOTINVIDMEM: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((747) as c_uint))) as HRESULT); // gen:bc3288909af3
pub const D3DERR_TEXTURE_LOAD_FAILED: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((725) as c_uint))) as HRESULT); // gen:b74a3392e361
pub const D3DERR_VERTEXBUFFERLOCKED: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((2062) as c_uint))) as HRESULT); // gen:70e884530c31
pub const D3DERR_VIEWPORTDATANOTSET: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((773) as c_uint))) as HRESULT); // gen:a0b445a8cc81
pub const D3DERR_VIEWPORTHASNODEVICE: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((774) as c_uint))) as HRESULT); // gen:fae3d44d7ba0
pub const D3D_OK: HRESULT = ((0) as HRESULT); // gen:f736e8837e47
pub const DDERR_ALREADYINITIALIZED: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((5) as c_uint))) as HRESULT); // gen:110a7d6b6721
pub const DDERR_BLTFASTCANTCLIP: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((574) as c_uint))) as HRESULT); // gen:64c0c46e4252
pub const DDERR_CANNOTATTACHSURFACE: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((10) as c_uint))) as HRESULT); // gen:ed91b53edfdc
pub const DDERR_CANNOTDETACHSURFACE: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((20) as c_uint))) as HRESULT); // gen:de97e7312f56
pub const DDERR_CANTCREATEDC: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((585) as c_uint))) as HRESULT); // gen:309a58fa84f3
pub const DDERR_CLIPPERISUSINGHWND: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((567) as c_uint))) as HRESULT); // gen:5c47030a420c
pub const DDERR_DCALREADYCREATED: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((620) as c_uint))) as HRESULT); // gen:fe643df88a32
pub const DDERR_HWNDALREADYSET: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((571) as c_uint))) as HRESULT); // gen:7f69ea69d21c
pub const DDERR_INVALIDCAPS: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((100) as c_uint))) as HRESULT); // gen:870b18d16d41
pub const DDERR_INVALIDOBJECT: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((130) as c_uint))) as HRESULT); // gen:6649d66eb29c
pub const DDERR_INVALIDPARAMS: HRESULT = ((0x80070057u32) as HRESULT); // gen:bc28897fef7d
pub const DDERR_INVALIDPIXELFORMAT: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((145) as c_uint))) as HRESULT); // gen:5c0460af4d38
pub const DDERR_INVALIDRECT: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((150) as c_uint))) as HRESULT); // gen:0b7d217e4764
pub const DDERR_INVALIDSURFACETYPE: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((592) as c_uint))) as HRESULT); // gen:3f174a63b705
pub const DDERR_MOREDATA: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((690) as c_uint))) as HRESULT); // gen:38d6b8de8e2c
pub const DDERR_NOCLIPLIST: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((205) as c_uint))) as HRESULT); // gen:00a3300bf236
pub const DDERR_NOCLIPPERATTACHED: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((568) as c_uint))) as HRESULT); // gen:ecc63ebcbd20
pub const DDERR_NOCOLORKEY: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((215) as c_uint))) as HRESULT); // gen:7a7c4431be36
pub const DDERR_NOCOLORKEYHW: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((220) as c_uint))) as HRESULT); // gen:b33c284e771b
pub const DDERR_NOCOOPERATIVELEVELSET: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((212) as c_uint))) as HRESULT); // gen:6d5619d3d07e
pub const DDERR_NODC: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((586) as c_uint))) as HRESULT); // gen:2c6f9d8c9adb
pub const DDERR_NODIRECTDRAWHW: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((563) as c_uint))) as HRESULT); // gen:bea64cfe203e
pub const DDERR_NOEXCLUSIVEMODE: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((225) as c_uint))) as HRESULT); // gen:079a40f6bbc7
pub const DDERR_NOFOCUSWINDOW: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((602) as c_uint))) as HRESULT); // gen:41513f41508a
pub const DDERR_NOHWND: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((569) as c_uint))) as HRESULT); // gen:56fcce3eb32a
pub const DDERR_NOOVERLAYHW: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((260) as c_uint))) as HRESULT); // gen:4a103a4610b8
pub const DDERR_NOPALETTEATTACHED: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((572) as c_uint))) as HRESULT); // gen:bc972fd87348
pub const DDERR_NORASTEROPHW: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((280) as c_uint))) as HRESULT); // gen:d6ab27d0ea3c
pub const DDERR_NOTAOVERLAYSURFACE: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((580) as c_uint))) as HRESULT); // gen:cd5c733e5438
pub const DDERR_NOTFLIPPABLE: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((582) as c_uint))) as HRESULT); // gen:5d1636ad1733
pub const DDERR_NOTFOUND: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((255) as c_uint))) as HRESULT); // gen:60db141e60a2
pub const DDERR_NOTONMIPMAPSUBLEVEL: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((603) as c_uint))) as HRESULT); // gen:040ce46bf530
pub const DDERR_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT); // gen:a081ac9e5927
pub const DDERR_OUTOFVIDEOMEMORY: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((380) as c_uint))) as HRESULT); // gen:d13f0e5731e5
pub const DDERR_OVERLAYNOTVISIBLE: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((577) as c_uint))) as HRESULT); // gen:3a0c5b016d78
pub const DDERR_SURFACEALREADYATTACHED: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((410) as c_uint))) as HRESULT); // gen:17afc7a0cce2
pub const DDERR_SURFACEBUSY: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((430) as c_uint))) as HRESULT); // gen:f54e150a609b
pub const DDERR_SURFACELOST: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((450) as c_uint))) as HRESULT); // gen:775ed5416598
pub const DDERR_SURFACENOTATTACHED: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((460) as c_uint))) as HRESULT); // gen:1d0ba0f2084e
pub const DDERR_UNSUPPORTED: HRESULT = ((0x80004001u32) as HRESULT); // gen:c7310cbb50a8
pub const DDERR_VERTICALBLANKINPROGRESS: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((537) as c_uint))) as HRESULT); // gen:e9e56a121cec
pub const DDERR_WRONGMODE: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((587) as c_uint))) as HRESULT); // gen:5189780b59b7
pub const DD_OK: HRESULT = ((0) as HRESULT); // gen:dbfa6c658ffc
pub const E_FAIL: HRESULT = ((0x80004005u32) as HRESULT); // gen:1275c1cb9e8d
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT); // gen:cd8a52171603
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT); // gen:80daa5b2ffa6
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT); // gen:cbf314acc21c
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT); // gen:98bee37aa995
pub const E_POINTER: HRESULT = ((0x80004003u32) as HRESULT); // gen:f72eeacc5d42
pub const HKEY_LOCAL_MACHINE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY); // gen:6463b26ca00d
pub const HWND_NOTOPMOST: HWND = (((-(2 as c_int))) as HWND); // gen:25aa63012d49
pub const HWND_TOP: HWND = 0x0usize as HWND; // gen:1312707d72c6
#[inline] // gen:dc9a16403a7a
pub unsafe fn IClassFactory_AddRef(This: *mut IClassFactory) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:64af75ab786b
pub unsafe fn IDirect3D2_AddRef(This: *mut IDirect3D2) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:0fe1939db941
pub unsafe fn IDirect3D3_AddRef(This: *mut IDirect3D3) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:397d7303e32c
pub unsafe fn IDirect3D7_AddRef(This: *mut IDirect3D7) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:4fe98251fcac
pub unsafe fn IDirect3DDevice2_AddRef(This: *mut IDirect3DDevice2) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:dc3fac1d8555
pub unsafe fn IDirect3DDevice2_GetRenderState(This: *mut IDirect3DDevice2, dwRenderStateType: D3DRENDERSTATETYPE, lpdwRenderState: LPDWORD) -> HRESULT { ((*(*This).lpVtbl).GetRenderState.unwrap_unchecked())(This, dwRenderStateType, lpdwRenderState) } // widl C wrapper
#[inline] // gen:08680acb32f5
pub unsafe fn IDirect3DDevice2_SetRenderState(This: *mut IDirect3DDevice2, dwRenderStateType: D3DRENDERSTATETYPE, dwRenderState: DWORD) -> HRESULT { ((*(*This).lpVtbl).SetRenderState.unwrap_unchecked())(This, dwRenderStateType, dwRenderState) } // widl C wrapper
#[inline] // gen:c0f028799c1e
pub unsafe fn IDirect3DDevice3_AddRef(This: *mut IDirect3DDevice3) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:e11811c4d9dd
pub unsafe fn IDirect3DDevice3_DeleteViewport(This: *mut IDirect3DDevice3, lpDirect3DViewport3: LPDIRECT3DVIEWPORT3) -> HRESULT { ((*(*This).lpVtbl).DeleteViewport.unwrap_unchecked())(This, lpDirect3DViewport3) } // widl C wrapper
#[inline] // gen:a9fe46902d64
pub unsafe fn IDirect3DDevice3_GetCurrentViewport(This: *mut IDirect3DDevice3, lplpDirect3DViewport3: *mut LPDIRECT3DVIEWPORT3) -> HRESULT { ((*(*This).lpVtbl).GetCurrentViewport.unwrap_unchecked())(This, lplpDirect3DViewport3) } // widl C wrapper
#[inline] // gen:3cd1c6e67903
pub unsafe fn IDirect3DDevice3_GetRenderState(This: *mut IDirect3DDevice3, dwRenderStateType: D3DRENDERSTATETYPE, lpdwRenderState: LPDWORD) -> HRESULT { ((*(*This).lpVtbl).GetRenderState.unwrap_unchecked())(This, dwRenderStateType, lpdwRenderState) } // widl C wrapper
#[inline] // gen:f3140bc740ed
pub unsafe fn IDirect3DDevice3_GetTransform(This: *mut IDirect3DDevice3, dtstTransformStateType: D3DTRANSFORMSTATETYPE, lpD3DMatrix: LPD3DMATRIX) -> HRESULT { ((*(*This).lpVtbl).GetTransform.unwrap_unchecked())(This, dtstTransformStateType, lpD3DMatrix) } // widl C wrapper
#[inline] // gen:9c09d200a59f
pub unsafe fn IDirect3DDevice3_MultiplyTransform(This: *mut IDirect3DDevice3, dtstTransformStateType: D3DTRANSFORMSTATETYPE, lpD3DMatrix: LPD3DMATRIX) -> HRESULT { ((*(*This).lpVtbl).MultiplyTransform.unwrap_unchecked())(This, dtstTransformStateType, lpD3DMatrix) } // widl C wrapper
#[inline] // gen:d591c7893b13
pub unsafe fn IDirect3DDevice3_SetCurrentViewport(This: *mut IDirect3DDevice3, lpDirect3DViewport3: LPDIRECT3DVIEWPORT3) -> HRESULT { ((*(*This).lpVtbl).SetCurrentViewport.unwrap_unchecked())(This, lpDirect3DViewport3) } // widl C wrapper
#[inline] // gen:18bff55815c5
pub unsafe fn IDirect3DDevice3_SetLightState(This: *mut IDirect3DDevice3, dwLightStateType: D3DLIGHTSTATETYPE, dwLightState: DWORD) -> HRESULT { ((*(*This).lpVtbl).SetLightState.unwrap_unchecked())(This, dwLightStateType, dwLightState) } // widl C wrapper
#[inline] // gen:264c985417cd
pub unsafe fn IDirect3DDevice3_SetRenderState(This: *mut IDirect3DDevice3, dwRenderStateType: D3DRENDERSTATETYPE, dwRenderState: DWORD) -> HRESULT { ((*(*This).lpVtbl).SetRenderState.unwrap_unchecked())(This, dwRenderStateType, dwRenderState) } // widl C wrapper
#[inline] // gen:ca9a5b252dae
pub unsafe fn IDirect3DDevice3_SetTexture(This: *mut IDirect3DDevice3, dwStage: DWORD, lpTexture2: LPDIRECT3DTEXTURE2) -> HRESULT { ((*(*This).lpVtbl).SetTexture.unwrap_unchecked())(This, dwStage, lpTexture2) } // widl C wrapper
#[inline] // gen:5b46ccd86230
pub unsafe fn IDirect3DDevice3_SetTransform(This: *mut IDirect3DDevice3, dtstTransformStateType: D3DTRANSFORMSTATETYPE, lpD3DMatrix: LPD3DMATRIX) -> HRESULT { ((*(*This).lpVtbl).SetTransform.unwrap_unchecked())(This, dtstTransformStateType, lpD3DMatrix) } // widl C wrapper
#[inline] // gen:45917c4aea44
pub unsafe fn IDirect3DDevice7_AddRef(This: *mut IDirect3DDevice7) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:8226a2e8c3e2
pub unsafe fn IDirect3DDevice7_BeginScene(This: *mut IDirect3DDevice7) -> HRESULT { ((*(*This).lpVtbl).BeginScene.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:9fe6803f31d9
pub unsafe fn IDirect3DDevice7_Clear(This: *mut IDirect3DDevice7, dwCount: DWORD, lpRects: LPD3DRECT, dwFlags: DWORD, dwColor: D3DCOLOR, dvZ: D3DVALUE, dwStencil: DWORD) -> HRESULT { ((*(*This).lpVtbl).Clear.unwrap_unchecked())(This, dwCount, lpRects, dwFlags, dwColor, dvZ, dwStencil) } // widl C wrapper
#[inline] // gen:f3a24004607c
pub unsafe fn IDirect3DDevice7_DeleteStateBlock(This: *mut IDirect3DDevice7, dwBlockHandle: DWORD) -> HRESULT { ((*(*This).lpVtbl).DeleteStateBlock.unwrap_unchecked())(This, dwBlockHandle) } // widl C wrapper
#[inline] // gen:8d9e526ca585
pub unsafe fn IDirect3DDevice7_DrawIndexedPrimitive(This: *mut IDirect3DDevice7, d3dptPrimitiveType: D3DPRIMITIVETYPE, d3dvtVertexType: DWORD, lpvVertices: LPVOID, dwVertexCount: DWORD, dwIndices: LPWORD, dwIndexCount: DWORD, dwFlags: DWORD) -> HRESULT { ((*(*This).lpVtbl).DrawIndexedPrimitive.unwrap_unchecked())(This, d3dptPrimitiveType, d3dvtVertexType, lpvVertices, dwVertexCount, dwIndices, dwIndexCount, dwFlags) } // widl C wrapper
#[inline] // gen:3fb03840b430
pub unsafe fn IDirect3DDevice7_DrawIndexedPrimitiveStrided(This: *mut IDirect3DDevice7, d3dptPrimitiveType: D3DPRIMITIVETYPE, dwVertexType: DWORD, lpD3DDrawPrimStrideData: LPD3DDRAWPRIMITIVESTRIDEDDATA, dwVertexCount: DWORD, lpIndex: LPWORD, dwIndexCount: DWORD, dwFlags: DWORD) -> HRESULT { ((*(*This).lpVtbl).DrawIndexedPrimitiveStrided.unwrap_unchecked())(This, d3dptPrimitiveType, dwVertexType, lpD3DDrawPrimStrideData, dwVertexCount, lpIndex, dwIndexCount, dwFlags) } // widl C wrapper
#[inline] // gen:30459400a911
pub unsafe fn IDirect3DDevice7_DrawIndexedPrimitiveVB(This: *mut IDirect3DDevice7, d3dptPrimitiveType: D3DPRIMITIVETYPE, lpD3DVertexBuf: LPDIRECT3DVERTEXBUFFER7, dwStartVertex: DWORD, dwNumVertices: DWORD, lpwIndices: LPWORD, dwIndexCount: DWORD, dwFlags: DWORD) -> HRESULT { ((*(*This).lpVtbl).DrawIndexedPrimitiveVB.unwrap_unchecked())(This, d3dptPrimitiveType, lpD3DVertexBuf, dwStartVertex, dwNumVertices, lpwIndices, dwIndexCount, dwFlags) } // widl C wrapper
#[inline] // gen:caf969e5c80f
pub unsafe fn IDirect3DDevice7_DrawPrimitive(This: *mut IDirect3DDevice7, d3dptPrimitiveType: D3DPRIMITIVETYPE, d3dvtVertexType: DWORD, lpvVertices: LPVOID, dwVertexCount: DWORD, dwFlags: DWORD) -> HRESULT { ((*(*This).lpVtbl).DrawPrimitive.unwrap_unchecked())(This, d3dptPrimitiveType, d3dvtVertexType, lpvVertices, dwVertexCount, dwFlags) } // widl C wrapper
#[inline] // gen:d33d2e8e69d2
pub unsafe fn IDirect3DDevice7_DrawPrimitiveStrided(This: *mut IDirect3DDevice7, d3dptPrimitiveType: D3DPRIMITIVETYPE, dwVertexType: DWORD, lpD3DDrawPrimStrideData: LPD3DDRAWPRIMITIVESTRIDEDDATA, dwVertexCount: DWORD, dwFlags: DWORD) -> HRESULT { ((*(*This).lpVtbl).DrawPrimitiveStrided.unwrap_unchecked())(This, d3dptPrimitiveType, dwVertexType, lpD3DDrawPrimStrideData, dwVertexCount, dwFlags) } // widl C wrapper
#[inline] // gen:2d2ad89199a4
pub unsafe fn IDirect3DDevice7_DrawPrimitiveVB(This: *mut IDirect3DDevice7, d3dptPrimitiveType: D3DPRIMITIVETYPE, lpD3DVertexBuf: LPDIRECT3DVERTEXBUFFER7, dwStartVertex: DWORD, dwNumVertices: DWORD, dwFlags: DWORD) -> HRESULT { ((*(*This).lpVtbl).DrawPrimitiveVB.unwrap_unchecked())(This, d3dptPrimitiveType, lpD3DVertexBuf, dwStartVertex, dwNumVertices, dwFlags) } // widl C wrapper
#[inline] // gen:3177db54fe4c
pub unsafe fn IDirect3DDevice7_EndScene(This: *mut IDirect3DDevice7) -> HRESULT { ((*(*This).lpVtbl).EndScene.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:1e440b0214e8
pub unsafe fn IDirect3DDevice7_EnumTextureFormats(This: *mut IDirect3DDevice7, lpD3DEnumPixelProc: LPD3DENUMPIXELFORMATSCALLBACK, lpArg: LPVOID) -> HRESULT { ((*(*This).lpVtbl).EnumTextureFormats.unwrap_unchecked())(This, lpD3DEnumPixelProc, lpArg) } // widl C wrapper
#[inline] // gen:95a66035e98d
pub unsafe fn IDirect3DDevice7_GetClipPlane(This: *mut IDirect3DDevice7, dwIndex: DWORD, pPlaneEquation: *mut D3DVALUE) -> HRESULT { ((*(*This).lpVtbl).GetClipPlane.unwrap_unchecked())(This, dwIndex, pPlaneEquation) } // widl C wrapper
#[inline] // gen:38c73b10d539
pub unsafe fn IDirect3DDevice7_GetClipStatus(This: *mut IDirect3DDevice7, lpD3DClipStatus: LPD3DCLIPSTATUS) -> HRESULT { ((*(*This).lpVtbl).GetClipStatus.unwrap_unchecked())(This, lpD3DClipStatus) } // widl C wrapper
#[inline] // gen:978af00f2f22
pub unsafe fn IDirect3DDevice7_GetRenderState(This: *mut IDirect3DDevice7, dwRenderStateType: D3DRENDERSTATETYPE, lpdwRenderState: LPDWORD) -> HRESULT { ((*(*This).lpVtbl).GetRenderState.unwrap_unchecked())(This, dwRenderStateType, lpdwRenderState) } // widl C wrapper
#[inline] // gen:439747d9fe5e
pub unsafe fn IDirect3DDevice7_GetTexture(This: *mut IDirect3DDevice7, dwStage: DWORD, lpTexture: *mut LPDIRECTDRAWSURFACE7) -> HRESULT { ((*(*This).lpVtbl).GetTexture.unwrap_unchecked())(This, dwStage, lpTexture) } // widl C wrapper
#[inline] // gen:76f83b74b1b7
pub unsafe fn IDirect3DDevice7_GetTextureStageState(This: *mut IDirect3DDevice7, dwStage: DWORD, d3dTexStageStateType: D3DTEXTURESTAGESTATETYPE, lpdwState: LPDWORD) -> HRESULT { ((*(*This).lpVtbl).GetTextureStageState.unwrap_unchecked())(This, dwStage, d3dTexStageStateType, lpdwState) } // widl C wrapper
#[inline] // gen:6bdb2c8bd20d
pub unsafe fn IDirect3DDevice7_GetTransform(This: *mut IDirect3DDevice7, dtstTransformStateType: D3DTRANSFORMSTATETYPE, lpD3DMatrix: LPD3DMATRIX) -> HRESULT { ((*(*This).lpVtbl).GetTransform.unwrap_unchecked())(This, dtstTransformStateType, lpD3DMatrix) } // widl C wrapper
#[inline] // gen:d7c9650b59c5
pub unsafe fn IDirect3DDevice7_LightEnable(This: *mut IDirect3DDevice7, dwLightIndex: DWORD, bEnable: BOOL) -> HRESULT { ((*(*This).lpVtbl).LightEnable.unwrap_unchecked())(This, dwLightIndex, bEnable) } // widl C wrapper
#[inline] // gen:59014425dc78
pub unsafe fn IDirect3DDevice7_MultiplyTransform(This: *mut IDirect3DDevice7, dtstTransformStateType: D3DTRANSFORMSTATETYPE, lpD3DMatrix: LPD3DMATRIX) -> HRESULT { ((*(*This).lpVtbl).MultiplyTransform.unwrap_unchecked())(This, dtstTransformStateType, lpD3DMatrix) } // widl C wrapper
#[inline] // gen:ed6440d69752
pub unsafe fn IDirect3DDevice7_SetClipStatus(This: *mut IDirect3DDevice7, lpD3DClipStatus: LPD3DCLIPSTATUS) -> HRESULT { ((*(*This).lpVtbl).SetClipStatus.unwrap_unchecked())(This, lpD3DClipStatus) } // widl C wrapper
#[inline] // gen:7be29d5e92eb
pub unsafe fn IDirect3DDevice7_SetLight(This: *mut IDirect3DDevice7, dwLightIndex: DWORD, lpLight: LPD3DLIGHT7) -> HRESULT { ((*(*This).lpVtbl).SetLight.unwrap_unchecked())(This, dwLightIndex, lpLight) } // widl C wrapper
#[inline] // gen:96074fc028bc
pub unsafe fn IDirect3DDevice7_SetMaterial(This: *mut IDirect3DDevice7, lpMat: LPD3DMATERIAL7) -> HRESULT { ((*(*This).lpVtbl).SetMaterial.unwrap_unchecked())(This, lpMat) } // widl C wrapper
#[inline] // gen:b67bd8a5c04f
pub unsafe fn IDirect3DDevice7_SetRenderState(This: *mut IDirect3DDevice7, dwRenderStateType: D3DRENDERSTATETYPE, dwRenderState: DWORD) -> HRESULT { ((*(*This).lpVtbl).SetRenderState.unwrap_unchecked())(This, dwRenderStateType, dwRenderState) } // widl C wrapper
#[inline] // gen:8693bdbf0806
pub unsafe fn IDirect3DDevice7_SetTextureStageState(This: *mut IDirect3DDevice7, dwStage: DWORD, d3dTexStageStateType: D3DTEXTURESTAGESTATETYPE, dwState: DWORD) -> HRESULT { ((*(*This).lpVtbl).SetTextureStageState.unwrap_unchecked())(This, dwStage, d3dTexStageStateType, dwState) } // widl C wrapper
#[inline] // gen:853e94ae0da7
pub unsafe fn IDirect3DDevice7_SetTransform(This: *mut IDirect3DDevice7, dtstTransformStateType: D3DTRANSFORMSTATETYPE, lpD3DMatrix: LPD3DMATRIX) -> HRESULT { ((*(*This).lpVtbl).SetTransform.unwrap_unchecked())(This, dtstTransformStateType, lpD3DMatrix) } // widl C wrapper
#[inline] // gen:f8017f1c1e0e
pub unsafe fn IDirect3DDevice7_SetViewport(This: *mut IDirect3DDevice7, lpData: LPD3DVIEWPORT7) -> HRESULT { ((*(*This).lpVtbl).SetViewport.unwrap_unchecked())(This, lpData) } // widl C wrapper
#[inline] // gen:05fd4b7e0a14
pub unsafe fn IDirect3DDevice7_ValidateDevice(This: *mut IDirect3DDevice7, lpdwPasses: LPDWORD) -> HRESULT { ((*(*This).lpVtbl).ValidateDevice.unwrap_unchecked())(This, lpdwPasses) } // widl C wrapper
#[inline] // gen:00aef593cb78
pub unsafe fn IDirect3DDevice_AddRef(This: *mut IDirect3DDevice) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:cfbd91718172
pub unsafe fn IDirect3DExecuteBuffer_AddRef(This: *mut IDirect3DExecuteBuffer) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:0d0ebdc8f9ac
pub unsafe fn IDirect3DLight_AddRef(This: *mut IDirect3DLight) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:cd0e6f701a63
pub unsafe fn IDirect3DLight_Release(This: *mut IDirect3DLight) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:2a5599ca1ff7
pub unsafe fn IDirect3DMaterial2_AddRef(This: *mut IDirect3DMaterial2) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:dc313ad57d60
pub unsafe fn IDirect3DMaterial3_AddRef(This: *mut IDirect3DMaterial3) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:4a6a2cdfd054
pub unsafe fn IDirect3DMaterial_AddRef(This: *mut IDirect3DMaterial) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:1df83722290f
pub unsafe fn IDirect3DTexture2_AddRef(This: *mut IDirect3DTexture2) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:e64d7ba6fe63
pub unsafe fn IDirect3DTexture2_Load(This: *mut IDirect3DTexture2, lpD3DTexture2: LPDIRECT3DTEXTURE2) -> HRESULT { ((*(*This).lpVtbl).Load.unwrap_unchecked())(This, lpD3DTexture2) } // widl C wrapper
#[inline] // gen:0a2220570d4f
pub unsafe fn IDirect3DVertexBuffer7_AddRef(This: *mut IDirect3DVertexBuffer7) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:45e9ea68dfa2
pub unsafe fn IDirect3DViewport3_AddRef(This: *mut IDirect3DViewport3) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:4bb06b8996b8
pub unsafe fn IDirect3DViewport3_Release(This: *mut IDirect3DViewport3) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:2dc753a1264b
pub unsafe fn IDirect3D_AddRef(This: *mut IDirect3D) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:d711c5d64bd7
pub unsafe fn IDirectDraw4_AddRef(This: *mut IDirectDraw4) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:acb31f4792bf
pub unsafe fn IDirectDraw4_GetSurfaceFromDC(This: *mut IDirectDraw4, arg1: HDC, arg2: *mut LPDIRECTDRAWSURFACE4) -> HRESULT { ((*(*This).lpVtbl).GetSurfaceFromDC.unwrap_unchecked())(This, arg1, arg2) } // widl C wrapper
#[inline] // gen:3ef3a7de41f0
pub unsafe fn IDirectDraw7_AddRef(This: *mut IDirectDraw7) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:6f6bf231e259
pub unsafe fn IDirectDraw7_CreateSurface(This: *mut IDirectDraw7, arg1: LPDDSURFACEDESC2, arg2: *mut LPDIRECTDRAWSURFACE7, arg3: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).CreateSurface.unwrap_unchecked())(This, arg1, arg2, arg3) } // widl C wrapper
#[inline] // gen:b629f3f4ebb4
pub unsafe fn IDirectDraw7_EnumSurfaces(This: *mut IDirectDraw7, arg1: DWORD, arg2: LPDDSURFACEDESC2, arg3: LPVOID, arg4: LPDDENUMSURFACESCALLBACK7) -> HRESULT { ((*(*This).lpVtbl).EnumSurfaces.unwrap_unchecked())(This, arg1, arg2, arg3, arg4) } // widl C wrapper
#[inline] // gen:a9c35fd25687
pub unsafe fn IDirectDraw7_GetAvailableVidMem(This: *mut IDirectDraw7, arg1: LPDDSCAPS2, arg2: LPDWORD, arg3: LPDWORD) -> HRESULT { ((*(*This).lpVtbl).GetAvailableVidMem.unwrap_unchecked())(This, arg1, arg2, arg3) } // widl C wrapper
#[inline] // gen:d0ae8ddced79
pub unsafe fn IDirectDraw7_GetFourCCCodes(This: *mut IDirectDraw7, arg1: LPDWORD, arg2: LPDWORD) -> HRESULT { ((*(*This).lpVtbl).GetFourCCCodes.unwrap_unchecked())(This, arg1, arg2) } // widl C wrapper
#[inline] // gen:1ae07611e8ab
pub unsafe fn IDirectDraw7_GetGDISurface(This: *mut IDirectDraw7, arg1: *mut LPDIRECTDRAWSURFACE7) -> HRESULT { ((*(*This).lpVtbl).GetGDISurface.unwrap_unchecked())(This, arg1) } // widl C wrapper
#[inline] // gen:fe780af83255
pub unsafe fn IDirectDraw7_Initialize(This: *mut IDirectDraw7, arg1: *mut GUID) -> HRESULT { ((*(*This).lpVtbl).Initialize.unwrap_unchecked())(This, arg1) } // widl C wrapper
#[inline] // gen:d3d850d3f5fa
pub unsafe fn IDirectDraw7_QueryInterface(This: *mut IDirectDraw7, riid: *const IID, ppvObj: *mut LPVOID) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObj) } // widl C wrapper
#[inline] // gen:1b8e6bd03da4
pub unsafe fn IDirectDraw7_Release(This: *mut IDirectDraw7) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:7c6081c2dc7e
pub unsafe fn IDirectDraw7_RestoreDisplayMode(This: *mut IDirectDraw7) -> HRESULT { ((*(*This).lpVtbl).RestoreDisplayMode.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:7939958aa0d8
pub unsafe fn IDirectDraw7_SetCooperativeLevel(This: *mut IDirectDraw7, arg1: HWND, arg2: DWORD) -> HRESULT { ((*(*This).lpVtbl).SetCooperativeLevel.unwrap_unchecked())(This, arg1, arg2) } // widl C wrapper
#[inline] // gen:113b7919af32
pub unsafe fn IDirectDrawClipper_AddRef(This: *mut IDirectDrawClipper) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:6258cbf357af
pub unsafe fn IDirectDrawClipper_GetClipList(This: *mut IDirectDrawClipper, arg1: LPRECT, arg2: LPRGNDATA, arg3: LPDWORD) -> HRESULT { ((*(*This).lpVtbl).GetClipList.unwrap_unchecked())(This, arg1, arg2, arg3) } // widl C wrapper
#[inline] // gen:b2af219ac9b8
pub unsafe fn IDirectDrawClipper_GetHWnd(This: *mut IDirectDrawClipper, arg1: *mut HWND) -> HRESULT { ((*(*This).lpVtbl).GetHWnd.unwrap_unchecked())(This, arg1) } // widl C wrapper
#[inline] // gen:ebc430a7873c
pub unsafe fn IDirectDrawClipper_QueryInterface(This: *mut IDirectDrawClipper, riid: *const IID, ppvObj: *mut LPVOID) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObj) } // widl C wrapper
#[inline] // gen:964652e7259b
pub unsafe fn IDirectDrawClipper_Release(This: *mut IDirectDrawClipper) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:08972337e14c
pub unsafe fn IDirectDrawGammaControl_AddRef(This: *mut IDirectDrawGammaControl) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:2411dddb2959
pub unsafe fn IDirectDrawPalette_AddRef(This: *mut IDirectDrawPalette) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:98d6f7b8f3fd
pub unsafe fn IDirectDrawPalette_GetEntries(This: *mut IDirectDrawPalette, arg1: DWORD, arg2: DWORD, arg3: DWORD, arg4: LPPALETTEENTRY) -> HRESULT { ((*(*This).lpVtbl).GetEntries.unwrap_unchecked())(This, arg1, arg2, arg3, arg4) } // widl C wrapper
#[inline] // gen:23c0d91932dd
pub unsafe fn IDirectDrawPalette_Release(This: *mut IDirectDrawPalette) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:57bbe831194f
pub unsafe fn IDirectDrawPalette_SetEntries(This: *mut IDirectDrawPalette, arg1: DWORD, arg2: DWORD, arg3: DWORD, arg4: LPPALETTEENTRY) -> HRESULT { ((*(*This).lpVtbl).SetEntries.unwrap_unchecked())(This, arg1, arg2, arg3, arg4) } // widl C wrapper
#[inline] // gen:b3b9fd39329f
pub unsafe fn IDirectDrawSurface2_AddRef(This: *mut IDirectDrawSurface2) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:5e4bf6217261
pub unsafe fn IDirectDrawSurface2_QueryInterface(This: *mut IDirectDrawSurface2, riid: *const IID, ppvObj: *mut LPVOID) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObj) } // widl C wrapper
#[inline] // gen:8554c9b7e044
pub unsafe fn IDirectDrawSurface2_Release(This: *mut IDirectDrawSurface2) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:6b909bd2e4cc
pub unsafe fn IDirectDrawSurface3_AddRef(This: *mut IDirectDrawSurface3) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:b261be29a5a3
pub unsafe fn IDirectDrawSurface3_QueryInterface(This: *mut IDirectDrawSurface3, riid: *const IID, ppvObj: *mut LPVOID) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObj) } // widl C wrapper
#[inline] // gen:720c4b4fd069
pub unsafe fn IDirectDrawSurface3_Release(This: *mut IDirectDrawSurface3) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:8828451c1e46
pub unsafe fn IDirectDrawSurface4_AddRef(This: *mut IDirectDrawSurface4) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:d5d37d7d6ea5
pub unsafe fn IDirectDrawSurface4_QueryInterface(This: *mut IDirectDrawSurface4, riid: *const IID, ppvObj: *mut LPVOID) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObj) } // widl C wrapper
#[inline] // gen:ebde23976a29
pub unsafe fn IDirectDrawSurface4_Release(This: *mut IDirectDrawSurface4) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:fe68c30d0f11
pub unsafe fn IDirectDrawSurface7_AddRef(This: *mut IDirectDrawSurface7) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:11463c698d3f
pub unsafe fn IDirectDrawSurface7_Flip(This: *mut IDirectDrawSurface7, arg1: LPDIRECTDRAWSURFACE7, arg2: DWORD) -> HRESULT { ((*(*This).lpVtbl).Flip.unwrap_unchecked())(This, arg1, arg2) } // widl C wrapper
#[inline] // gen:02ad0e42362d
pub unsafe fn IDirectDrawSurface7_GetAttachedSurface(This: *mut IDirectDrawSurface7, arg1: LPDDSCAPS2, arg2: *mut LPDIRECTDRAWSURFACE7) -> HRESULT { ((*(*This).lpVtbl).GetAttachedSurface.unwrap_unchecked())(This, arg1, arg2) } // widl C wrapper
#[inline] // gen:4cc3f5733a2b
pub unsafe fn IDirectDrawSurface7_GetColorKey(This: *mut IDirectDrawSurface7, arg1: DWORD, arg2: LPDDCOLORKEY) -> HRESULT { ((*(*This).lpVtbl).GetColorKey.unwrap_unchecked())(This, arg1, arg2) } // widl C wrapper
#[inline] // gen:90c3d8e3d06d
pub unsafe fn IDirectDrawSurface7_GetPalette(This: *mut IDirectDrawSurface7, arg1: *mut LPDIRECTDRAWPALETTE) -> HRESULT { ((*(*This).lpVtbl).GetPalette.unwrap_unchecked())(This, arg1) } // widl C wrapper
#[inline] // gen:bad3971b934d
pub unsafe fn IDirectDrawSurface7_QueryInterface(This: *mut IDirectDrawSurface7, riid: *const IID, ppvObj: *mut LPVOID) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObj) } // widl C wrapper
#[inline] // gen:79c9ca43e46e
pub unsafe fn IDirectDrawSurface7_Release(This: *mut IDirectDrawSurface7) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:ceb96a049fce
pub unsafe fn IDirectDrawSurface7_SetColorKey(This: *mut IDirectDrawSurface7, arg1: DWORD, arg2: LPDDCOLORKEY) -> HRESULT { ((*(*This).lpVtbl).SetColorKey.unwrap_unchecked())(This, arg1, arg2) } // widl C wrapper
#[inline] // gen:b0b37b514295
pub unsafe fn IDirectDrawSurface_AddRef(This: *mut IDirectDrawSurface) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:d356fcd98ae4
pub unsafe fn IDirectDrawSurface_QueryInterface(This: *mut IDirectDrawSurface, riid: *const IID, ppvObj: *mut LPVOID) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObj) } // widl C wrapper
#[inline] // gen:27675f20eea2
pub unsafe fn IDirectDrawSurface_Release(This: *mut IDirectDrawSurface) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:a32aab4686d8
pub unsafe fn IDirectDrawSurface_Restore(This: *mut IDirectDrawSurface) -> HRESULT { ((*(*This).lpVtbl).Restore.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:8f0cace02229
pub unsafe fn IDirectDraw_Initialize(This: *mut IDirectDraw, arg1: *mut GUID) -> HRESULT { ((*(*This).lpVtbl).Initialize.unwrap_unchecked())(This, arg1) } // widl C wrapper
#[inline] // gen:e38f8805f4c0
pub unsafe fn IDirectDraw_Release(This: *mut IDirectDraw) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:6ad0b58c268c
pub unsafe fn IUnknown_AddRef(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:38acc17a31e7
pub unsafe fn IUnknown_QueryInterface(This: *mut IUnknown, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:aa234a85a98a
pub unsafe fn IUnknown_Release(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
pub type REFCLSID = *const CLSID; // gen:4a91e3a9efb2
pub type REFGUID = *const GUID; // gen:4c78e76f64d6
pub type REFIID = *const IID; // gen:c79168ffaea6
pub const S_FALSE: HRESULT = ((1) as HRESULT); // gen:b1b8213e7dac
pub const S_OK: HRESULT = ((0) as HRESULT); // gen:9969a2c48b97
pub const WINED3D_OK: HRESULT = ((0) as HRESULT); // gen:7d6ca58d383b
pub const WINEDDERR_NOTAOVERLAYSURFACE: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((580) as c_uint))) as HRESULT); // gen:5f576f9df060
pub const WINEDDERR_OVERLAYNOTVISIBLE: HRESULT = (((((((1) as c_uint) << 31) | (((0x876) as c_uint) << 16)) | ((577) as c_uint))) as HRESULT); // gen:aa041abecea4
pub use crate::{WINED3D_TS_WORLD_MATRIX, max, min};
// ==== end SDK header macros ====

// Hand-ported ddraw_private.h initializer macros (outside the generated block, so `port
// macros` keeps them): C's `{ x, #x }` is a brace initializer of the table's element type,
// which a macro_rules! can only build by naming the struct. `ME`'s `STRUCT` is whatever the C
// `#define`d it to before the table -- a `type STRUCT = ..;` where the table is -- and its
// member is a PATH, since bindgen puts some DDSURFACEDESC2 members in anonymous unions.
#[allow(unused_macros)]
macro_rules! FE {
    ($x:expr $(,)?) => {
        flag_info {
            val: ($x) as DWORD,
            name: concat!(stringify!($x), "\0").as_ptr() as *const core::ffi::c_char,
        }
    };
}
#[allow(unused_imports)]
pub(crate) use FE;
#[allow(unused_macros)]
macro_rules! ME {
    ($x:expr, $f:expr, $($e:tt)+) => {
        member_info {
            val: ($x) as DWORD,
            name: concat!(stringify!($x), "\0").as_ptr() as *const core::ffi::c_char,
            // `(void (*)(const void *))(f)`: each dumper takes its own pointee type.
            func: Some(unsafe {
                core::mem::transmute::<*const (), unsafe extern "C" fn(*const core::ffi::c_void)>(
                    $f as *const (),
                )
            }),
            offset: core::mem::offset_of!(STRUCT, $($e)+) as isize,
        }
    };
}
#[allow(unused_imports)]
pub(crate) use ME;

// hand-edited, so kept out of the generated block below (generated_items.py); delete to regenerate
// dll/directx/wine/ddraw/ddraw_private.h:650: header-inline `ddraw_surface_can_be_lost`
#[allow(dead_code)]
#[inline]
pub unsafe fn ddraw_surface_can_be_lost(surface: *const ddraw_surface) -> BOOL {
    let caps = (*surface).surface_desc.ddsCaps.dwCaps;

    if (caps & DDSCAPS_SYSTEMMEMORY) == 0
        || (caps & (DDSCAPS_PRIMARYSURFACE | DDSCAPS_ZBUFFER)) != 0
    {
        return (TRUE) as BOOL;
    }

    ((*surface).sysmem_fallback()) as i32
}
// hand-edited, so kept out of the generated block below (generated_items.py); delete to regenerate
// dll/directx/wine/ddraw/ddraw_private.h:636: header-inline `format_is_compressed`
#[allow(dead_code)]
#[inline]
pub unsafe fn format_is_compressed(format: *const DDPIXELFORMAT) -> BOOL {
    if ((*format).dwFlags & DDPF_FOURCC) != 0
        && ((*format).dwFourCC == WINED3DFMT_DXT1
            || (*format).dwFourCC == WINED3DFMT_DXT2
            || (*format).dwFourCC == WINED3DFMT_DXT3
            || (*format).dwFourCC == WINED3DFMT_DXT4
            || (*format).dwFourCC == WINED3DFMT_DXT5)
    {
        (TRUE as i32)
    } else {
        (FALSE as i32)
    }
}
// hand-edited, so kept out of the generated block below (generated_items.py); delete to regenerate
// dll/directx/wine/ddraw/ddraw_private.h:643: header-inline `format_is_paletteindexed`
#[allow(dead_code)]
#[inline]
pub unsafe fn format_is_paletteindexed(fmt: *const DDPIXELFORMAT) -> BOOL {
    let flags = DDPF_PALETTEINDEXED1
        | DDPF_PALETTEINDEXED2
        | DDPF_PALETTEINDEXED4
        | DDPF_PALETTEINDEXED8
        | DDPF_PALETTEINDEXEDTO8;
    if ((*fmt).dwFlags & flags) != 0 {
        (TRUE as i32)
    } else {
        (FALSE as i32)
    }
}

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/directx/wine/ddraw/ddraw_private.h:748: #define DD_STRUCT_COPY_BYSIZE(to, from) DD_STRUCT_COPY_BYSIZE_(to,from,(to)->dwSize,(from)->dwSize) // gen:8efd0f54942c
#[allow(unused_macros)]
macro_rules! DD_STRUCT_COPY_BYSIZE {
    ($to:expr, $from:expr $(,)?) => {
        DD_STRUCT_COPY_BYSIZE_!($to, $from, (*$to).dwSize, (*$from).dwSize)
    };
}
#[allow(unused_imports)]
pub(crate) use DD_STRUCT_COPY_BYSIZE;
// dll/directx/wine/ddraw/ddraw_private.h:737: #define DD_STRUCT_COPY_BYSIZE_(to, from, to_size, from_size) do { DWORD __size = (to)->dwSize; DWORD __resetsize = min(to_size, sizeof(*to)); DWORD __copysize = min(__resetsize, from_size); assert(to != from); memcpy(to, from, __copysize); memset((char*)(to) + __copysize, 0, __resetsize - __copysize); (to)->dwSize = __size; } while (0) // gen:8ca5e54b970d
#[allow(unused_macros)]
macro_rules! DD_STRUCT_COPY_BYSIZE_ {
    ($to:expr, $from:expr, $to_size:expr, $from_size:expr $(,)?) => {{
        let __size = (*$to).dwSize;
        let __resetsize = min!($to_size, core::mem::size_of_val(&*$to) as _);
        let __copysize = min!(__resetsize, $from_size);
        assert!(($to) != ($from));
        memcpy(($to).cast(), ($from).cast(), __copysize as usize);
        memset(
            ($to).cast::<u8>().add(__copysize as usize).cast(),
            0,
            (__resetsize - __copysize) as usize,
        );
        (*$to).dwSize = __size;
    }};
}
#[allow(unused_imports)]
pub(crate) use DD_STRUCT_COPY_BYSIZE_;
// dll/directx/wine/ddraw/ddraw_private.h:615: #define GET_TEXCOORD_SIZE_FROM_FVF(d3dvtVertexType, tex_num) (((((d3dvtVertexType) >> (16 + (2 * (tex_num)))) + 1) & 0x03) + 1) // gen:ddc15694dea5
#[allow(unused_macros)]
macro_rules! GET_TEXCOORD_SIZE_FROM_FVF {
    ($d3dvtVertexType:expr, $tex_num:expr $(,)?) => {
        ((((($d3dvtVertexType) >> (16 + (2 * ($tex_num)))) + 1) & 0x03) + 1)
    };
}
#[allow(unused_imports)]
pub(crate) use GET_TEXCOORD_SIZE_FROM_FVF;
// dll/directx/wine/ddraw/ddraw_private.h:612: #define GET_TEXCOUNT_FROM_FVF(d3dvtVertexType) (((d3dvtVertexType) & D3DFVF_TEXCOUNT_MASK) >> D3DFVF_TEXCOUNT_SHIFT) // gen:42649cdaae36
#[allow(unused_macros)]
macro_rules! GET_TEXCOUNT_FROM_FVF {
    ($d3dvtVertexType:expr $(,)?) => {
        ((($d3dvtVertexType) & D3DFVF_TEXCOUNT_MASK) >> D3DFVF_TEXCOUNT_SHIFT)
    };
}
#[allow(unused_imports)]
pub(crate) use GET_TEXCOUNT_FROM_FVF;
// dll/directx/wine/ddraw/ddraw_private.h:147: header-inline `ddraw_set_swapchain_window` // gen:e800a12d7b92
#[allow(dead_code)]
#[inline]
pub unsafe fn ddraw_set_swapchain_window(ddraw: *mut ddraw, mut window: HWND) {
    if window == GetDesktopWindow() {
        window = core::ptr::null_mut();
    }
    (*ddraw).swapchain_window = window;
}
// dll/directx/wine/ddraw/ddraw_private.h:704: header-inline `ddraw_surface_get_any_texture` // gen:42a2445cc76e
#[allow(dead_code)]
#[inline]
pub unsafe fn ddraw_surface_get_any_texture(
    surface: *mut ddraw_surface,
    flags: core::ffi::c_uint,
) -> *mut wined3d_texture {
    if ((*surface).texture_location & DDRAW_SURFACE_LOCATION_DEFAULT) != 0
        || ((*surface).surface_desc.ddsCaps.dwCaps & DDSCAPS_SYSTEMMEMORY) != 0
    {
        return ddraw_surface_get_default_texture(surface, flags);
    }

    assert!(((*surface).texture_location & DDRAW_SURFACE_LOCATION_DRAW) != 0);
    ddraw_surface_get_draw_texture(surface, flags)
}
// dll/directx/wine/ddraw/ddraw_private.h:667: header-inline `ddraw_surface_get_default_texture` // gen:61ae6b06c79c
#[allow(dead_code)]
#[inline]
pub unsafe fn ddraw_surface_get_default_texture(
    surface: *mut ddraw_surface,
    flags: core::ffi::c_uint,
) -> *mut wined3d_texture {
    if !(*surface).draw_texture.is_null() {
        if (flags & DDRAW_SURFACE_READ) != 0
            && ((*surface).texture_location & DDRAW_SURFACE_LOCATION_DEFAULT) == 0
        {
            wined3d_device_context_copy_sub_resource_region(
                (*(*surface).ddraw).immediate_context,
                wined3d_texture_get_resource((*surface).wined3d_texture),
                (*surface).sub_resource_idx,
                0,
                0,
                0,
                wined3d_texture_get_resource((*surface).draw_texture),
                (*surface).sub_resource_idx,
                core::ptr::null_mut(),
                0,
            );
            (*surface).texture_location |= DDRAW_SURFACE_LOCATION_DEFAULT;
        }

        if (flags & DDRAW_SURFACE_WRITE) != 0 {
            (*surface).texture_location = DDRAW_SURFACE_LOCATION_DEFAULT;
        }
    }
    (*surface).wined3d_texture
}
// dll/directx/wine/ddraw/ddraw_private.h:685: header-inline `ddraw_surface_get_draw_texture` // gen:cedd47f50a91
#[allow(dead_code)]
#[inline]
pub unsafe fn ddraw_surface_get_draw_texture(
    surface: *mut ddraw_surface,
    flags: core::ffi::c_uint,
) -> *mut wined3d_texture {
    if (*surface).draw_texture.is_null() {
        return (*surface).wined3d_texture;
    }

    if (flags & DDRAW_SURFACE_READ) != 0
        && ((*surface).texture_location & DDRAW_SURFACE_LOCATION_DRAW) == 0
    {
        wined3d_device_context_copy_sub_resource_region(
            (*(*surface).ddraw).immediate_context,
            wined3d_texture_get_resource((*surface).draw_texture),
            (*surface).sub_resource_idx,
            0,
            0,
            0,
            wined3d_texture_get_resource((*surface).wined3d_texture),
            (*surface).sub_resource_idx,
            core::ptr::null_mut(),
            0,
        );
        (*surface).texture_location |= DDRAW_SURFACE_LOCATION_DRAW;
    }

    if (flags & DDRAW_SURFACE_WRITE) != 0 {
        (*surface).texture_location = DDRAW_SURFACE_LOCATION_DRAW;
    }

    (*surface).draw_texture
}
// dll/directx/wine/ddraw/ddraw_private.h:380: header-inline `impl_from_IDirect3DDevice` // gen:9b27076456ea
#[allow(dead_code)]
#[inline]
pub unsafe fn impl_from_IDirect3DDevice(iface: *mut IDirect3DDevice) -> *mut d3d_device {
    CONTAINING_RECORD!(iface, d3d_device, IDirect3DDevice_iface)
}
// dll/directx/wine/ddraw/ddraw_private.h:385: header-inline `impl_from_IDirect3DDevice2` // gen:58bf5eefc2ac
#[allow(dead_code)]
#[inline]
pub unsafe fn impl_from_IDirect3DDevice2(iface: *mut IDirect3DDevice2) -> *mut d3d_device {
    CONTAINING_RECORD!(iface, d3d_device, IDirect3DDevice2_iface)
}
// dll/directx/wine/ddraw/ddraw_private.h:390: header-inline `impl_from_IDirect3DDevice3` // gen:e016f9d56376
#[allow(dead_code)]
#[inline]
pub unsafe fn impl_from_IDirect3DDevice3(iface: *mut IDirect3DDevice3) -> *mut d3d_device {
    CONTAINING_RECORD!(iface, d3d_device, IDirect3DDevice3_iface)
}
// dll/directx/wine/ddraw/ddraw_private.h:395: header-inline `impl_from_IDirect3DDevice7` // gen:f313b0873edc
#[allow(dead_code)]
#[inline]
pub unsafe fn impl_from_IDirect3DDevice7(iface: *mut IDirect3DDevice7) -> *mut d3d_device {
    CONTAINING_RECORD!(iface, d3d_device, IDirect3DDevice7_iface)
}
// dll/directx/wine/ddraw/ddraw_private.h:240: header-inline `impl_from_IDirect3DTexture` // gen:89e4f1aed437
#[allow(dead_code)]
#[inline]
pub unsafe fn impl_from_IDirect3DTexture(iface: *mut IDirect3DTexture) -> *mut ddraw_surface {
    CONTAINING_RECORD!(iface, ddraw_surface, IDirect3DTexture_iface)
}
// dll/directx/wine/ddraw/ddraw_private.h:245: header-inline `impl_from_IDirect3DTexture2` // gen:6b574b5516c1
#[allow(dead_code)]
#[inline]
pub unsafe fn impl_from_IDirect3DTexture2(iface: *mut IDirect3DTexture2) -> *mut ddraw_surface {
    CONTAINING_RECORD!(iface, ddraw_surface, IDirect3DTexture2_iface)
}
// dll/directx/wine/ddraw/ddraw_private.h:433: header-inline `impl_from_IDirectDrawPalette` // gen:b876f32bee9f
#[allow(dead_code)]
#[inline]
pub unsafe fn impl_from_IDirectDrawPalette(iface: *mut IDirectDrawPalette) -> *mut ddraw_palette {
    CONTAINING_RECORD!(iface, ddraw_palette, IDirectDrawPalette_iface)
}
// dll/directx/wine/ddraw/ddraw_private.h:250: header-inline `impl_from_IDirectDrawSurface` // gen:042766e9e914
#[allow(dead_code)]
#[inline]
pub unsafe fn impl_from_IDirectDrawSurface(iface: *mut IDirectDrawSurface) -> *mut ddraw_surface {
    CONTAINING_RECORD!(iface, ddraw_surface, IDirectDrawSurface_iface)
}
// dll/directx/wine/ddraw/ddraw_private.h:255: header-inline `impl_from_IDirectDrawSurface2` // gen:1b90dc7a49d2
#[allow(dead_code)]
#[inline]
pub unsafe fn impl_from_IDirectDrawSurface2(iface: *mut IDirectDrawSurface2) -> *mut ddraw_surface {
    CONTAINING_RECORD!(iface, ddraw_surface, IDirectDrawSurface2_iface)
}
// dll/directx/wine/ddraw/ddraw_private.h:260: header-inline `impl_from_IDirectDrawSurface3` // gen:312e71f61580
#[allow(dead_code)]
#[inline]
pub unsafe fn impl_from_IDirectDrawSurface3(iface: *mut IDirectDrawSurface3) -> *mut ddraw_surface {
    CONTAINING_RECORD!(iface, ddraw_surface, IDirectDrawSurface3_iface)
}
// dll/directx/wine/ddraw/ddraw_private.h:265: header-inline `impl_from_IDirectDrawSurface4` // gen:a6d55b78a425
#[allow(dead_code)]
#[inline]
pub unsafe fn impl_from_IDirectDrawSurface4(iface: *mut IDirectDrawSurface4) -> *mut ddraw_surface {
    CONTAINING_RECORD!(iface, ddraw_surface, IDirectDrawSurface4_iface)
}
// dll/directx/wine/ddraw/ddraw_private.h:270: header-inline `impl_from_IDirectDrawSurface7` // gen:e29d81058fd5
#[allow(dead_code)]
#[inline]
pub unsafe fn impl_from_IDirectDrawSurface7(iface: *mut IDirectDrawSurface7) -> *mut ddraw_surface {
    CONTAINING_RECORD!(iface, ddraw_surface, IDirectDrawSurface7_iface)
}
// ==== end port macros ====
