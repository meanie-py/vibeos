//! msrle32-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedDecrement64, InterlockedExchange, InterlockedIncrement, InterlockedIncrement64, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _byteswap_uint64, _byteswap_ulong, _byteswap_ushort, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_variant, debugstr_vt, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, lrintf, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_longlong, wine_dbgstr_variant, wine_dbgstr_vt, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ================= hand ports (above the generated blocks: they survive replays) ==============
// msrle32.c:49 GetRawPixel(lpbi, lp, x): the pixel at x of a DIB row, at 1, 4 or 8 bpp. `lp`
// is a byte POINTER, which the generated lowering indexed like an array (E0608 at every use).
// Exceeds the C: its 1 bpp case shifts by `8 - x%8`, so the first pixel of every byte
// (x%8 == 0) is `byte >> 8`, always 0, and the rest read their left neighbour's bit. In a
// 1 bpp DIB pixel 0 is bit 7: the shift is `7 - x%8`. The 4 bpp case is right as written.
// Evaluates to C's promoted int.
#[allow(unused_macros)]
macro_rules! GetRawPixel {
    ($lpbi:expr, $lp:expr, $x:expr $(,)?) => {{
        let lp = $lp as *const u8;
        let x = ($x) as usize;
        if (*$lpbi).biBitCount == 1 {
            ((*lp.add(x / 8) as i32) >> (7 - (x % 8) as i32)) & 1
        } else if (*$lpbi).biBitCount == 4 {
            ((*lp.add(x / 2) as i32) >> (4 * (1 - (x % 2) as i32))) & 15
        } else {
            *lp.add(x) as i32
        }
    }};
}
#[allow(unused_imports)]
pub(crate) use GetRawPixel;

// vfw.h: `#define ICTYPE_VIDEO mmioFOURCC('v','i','d','c')` and
// `#define cktypeDIBbits aviTWOCC('d','b')`, character codes packed low byte first.
pub const ICTYPE_VIDEO: DWORD = 0x6364_6976;
pub const cktypeDIBbits: WORD = 0x6264;

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! MAKEAVICKID { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((((((if (((($p1 & 0x0f) > 9) as c_int)) as isize != 0 { ((($p1 & 0x0f) - 10) + 65) } else { (($p1 & 0x0f) + 48) })) as BYTE)) as c_int) << 8) | (((((if ((((($p1 & 0xf0) >> 4) > 9) as c_int)) as isize != 0 { (((($p1 & 0xf0) >> 4) - 10) + 65) } else { ((($p1 & 0xf0) >> 4) + 48) })) as BYTE)) as c_int))) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG) }; } // gen:4e8a314df0f7
#[macro_export] macro_rules! StreamFromFOURCC { ($p0:expr $(,)?) => { (((((if ((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD_PTR) & 0xff)) as BYTE) >= 65) as c_int)) as isize != 0 { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD_PTR) & 0xff)) as BYTE)) as c_int) + 10) - 65) } else { ((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD_PTR) & 0xff)) as BYTE)) as c_int) - 48) }) << 4) + (if (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD_PTR) >> 8) & 0xff)) as BYTE) >= 65) as c_int)) as isize != 0 { ((((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD_PTR) >> 8) & 0xff)) as BYTE)) as c_int) + 10) - 65) } else { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD_PTR) >> 8) & 0xff)) as BYTE)) as c_int) - 48) }))) as WORD) }; } // gen:d73d27fd5210
#[macro_export] macro_rules! min { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 < $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; } // gen:fb1d8a90940a
#[macro_export] macro_rules! mmioFOURCC { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { ((((((($p0) as BYTE)) as DWORD) | ((((($p1) as BYTE)) as DWORD) << 8)) | ((((($p2) as BYTE)) as DWORD) << 16)) | ((((($p3) as BYTE)) as DWORD) << 24)) }; } // gen:1c4ece2a2146
pub use crate::{MAKEAVICKID, StreamFromFOURCC, min, mmioFOURCC};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/win32/msrle32/msrle_private.h:45: #define DIBWIDTHBYTES(bi) WIDTHBYTES((WORD)(bi).biWidth * (WORD)(bi).biBitCount) // gen:e5211c03aba6
#[allow(unused_macros)]
macro_rules! DIBWIDTHBYTES {
    ($bi:expr $(,)?) => {
        (WIDTHBYTES!((($bi).biWidth as u16 as u32) * (($bi).biBitCount as u16 as u32)))
    };
}
#[allow(unused_imports)]
pub(crate) use DIBWIDTHBYTES;
// dll/win32/msrle32/msrle_private.h:44: #define WIDTHBYTES(i) ((WORD)((i+31u)&(~31u))/8u) // gen:9e44bf2a43d5
#[allow(unused_macros)]
macro_rules! WIDTHBYTES {
    ($i:expr $(,)?) => {
        ((((($i) as u32).wrapping_add(31u32) & !31u32) as u16) as u32 / 8u32)
    };
}
#[allow(unused_imports)]
pub(crate) use WIDTHBYTES;
// dll/win32/msrle32/msrle32.c:37: #define compare_fourcc(fcc1, fcc2) (((fcc1)^(fcc2))&~0x20202020) // gen:a02e1d46266c
#[allow(unused_macros)]
macro_rules! compare_fourcc {
    ($fcc1:expr, $fcc2:expr $(,)?) => {
        ((($fcc1) ^ ($fcc2)) & !0x20202020)
    };
}
#[allow(unused_imports)]
pub(crate) use compare_fourcc;
// ==== end port macros ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
extern "system" {
    pub fn DefDriverProc( arg1: DWORD_PTR, arg2: HDRVR, arg3: UINT, arg4: LPARAM, arg5: LPARAM, ) -> LRESULT; // sndrec32-rust/src/types_i386.rs
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GlobalAlloc(arg1: UINT, arg2: SIZE_T) -> HGLOBAL; // aclui-rust/src/types_i386.rs
    pub fn GlobalFree(arg1: HGLOBAL) -> HGLOBAL; // aclui-rust/src/types_i386.rs
    pub fn GlobalHandle(arg1: LPCVOID) -> HGLOBAL; // aclui-rust/src/types_i386.rs
    pub fn GlobalLock(arg1: HGLOBAL) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn GlobalUnlock(arg1: HGLOBAL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LoadStringW( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn LocalAlloc(arg1: UINT, arg2: SIZE_T) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn MessageBoxW( hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const FOURCC_MRLE: DWORD = 0x454C524D; // clang, msrle32.c
pub const FOURCC_RLE: DWORD = 0x20454C52; // clang, msrle32.c
pub const FOURCC_RLE4: DWORD = 0x34454C52; // clang, msrle32.c
pub const FOURCC_RLE8: DWORD = 0x38454C52; // clang, msrle32.c
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
    ) -> ::core::ffi::c_int; // msrle32.c
}
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_msrle32: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x6d\x73\x72\x6c\x65\x33\x32\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_msrle32 };
// ==== end wine debug channels ====
