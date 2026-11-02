//! dmilib-rust prelude — the single import surface the translated modules use.
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
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => { ::win32macros_rust::DBG_TYPECHECK!($($arg)*) }; }
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedCompareExchangePointer, InterlockedDecrement, InterlockedDecrement64, InterlockedExchange, InterlockedExchangePointer, InterlockedIncrement, InterlockedIncrement64, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _byteswap_uint64, _byteswap_ulong, _byteswap_ushort, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_variant, debugstr_vt, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, lrintf, strlenW, tolowerW, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_longlong, wine_dbgstr_variant, wine_dbgstr_vt, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const BIOS_DATE: _ID_STRINGS = _ID_STRINGS_BIOS_DATE; // island enum // gen:ee603da1f4a0
pub const BIOS_VENDOR: _ID_STRINGS = _ID_STRINGS_BIOS_VENDOR; // island enum // gen:6570fc568e6a
pub const BIOS_VERSION: _ID_STRINGS = _ID_STRINGS_BIOS_VERSION; // island enum // gen:9e7f8acb0202
pub const BOARD_ASSET_TAG: _ID_STRINGS = _ID_STRINGS_BOARD_ASSET_TAG; // island enum // gen:467495caf301
pub const BOARD_NAME: _ID_STRINGS = _ID_STRINGS_BOARD_NAME; // island enum // gen:a7524debda9b
pub const BOARD_SERIAL: _ID_STRINGS = _ID_STRINGS_BOARD_SERIAL; // island enum // gen:508822110314
pub const BOARD_VENDOR: _ID_STRINGS = _ID_STRINGS_BOARD_VENDOR; // island enum // gen:634319201fdc
pub const BOARD_VERSION: _ID_STRINGS = _ID_STRINGS_BOARD_VERSION; // island enum // gen:90e19fee031b
pub const DMI_BIOS_DATE: _DMI_FIELD_OFFSETS = _DMI_FIELD_OFFSETS_DMI_BIOS_DATE; // island enum // gen:1f46001bb45a
pub const DMI_BIOS_SIZE: _DMI_FIELD_OFFSETS = _DMI_FIELD_OFFSETS_DMI_BIOS_SIZE; // island enum // gen:6bd851821e68
pub const DMI_BIOS_VENDOR: _DMI_FIELD_OFFSETS = _DMI_FIELD_OFFSETS_DMI_BIOS_VENDOR; // island enum // gen:c77618ea5df8
pub const DMI_BIOS_VERSION: _DMI_FIELD_OFFSETS = _DMI_FIELD_OFFSETS_DMI_BIOS_VERSION; // island enum // gen:2be213ab6d56
pub const DMI_BOARD_ASSET_TAG: _DMI_FIELD_OFFSETS = _DMI_FIELD_OFFSETS_DMI_BOARD_ASSET_TAG; // island enum // gen:0b56e1dc4220
pub const DMI_BOARD_NAME: _DMI_FIELD_OFFSETS = _DMI_FIELD_OFFSETS_DMI_BOARD_NAME; // island enum // gen:27c69a0e47b4
pub const DMI_BOARD_SERIAL: _DMI_FIELD_OFFSETS = _DMI_FIELD_OFFSETS_DMI_BOARD_SERIAL; // island enum // gen:b003afa0b4da
pub const DMI_BOARD_SIZE: _DMI_FIELD_OFFSETS = _DMI_FIELD_OFFSETS_DMI_BOARD_SIZE; // island enum // gen:3d9dbd6fe76e
pub const DMI_BOARD_VENDOR: _DMI_FIELD_OFFSETS = _DMI_FIELD_OFFSETS_DMI_BOARD_VENDOR; // island enum // gen:8ef59c468ec3
pub const DMI_BOARD_VERSION: _DMI_FIELD_OFFSETS = _DMI_FIELD_OFFSETS_DMI_BOARD_VERSION; // island enum // gen:6befd1d29be2
pub const DMI_ENTRY_BASEBOARD: DMI_ENTRY_TYPE = DMI_ENTRY_TYPE_DMI_ENTRY_BASEBOARD; // island enum // gen:40fb6fa5cedb
pub const DMI_ENTRY_BIOS: DMI_ENTRY_TYPE = DMI_ENTRY_TYPE_DMI_ENTRY_BIOS; // island enum // gen:646a1b42bec9
pub const DMI_ENTRY_CHASSIS: DMI_ENTRY_TYPE = DMI_ENTRY_TYPE_DMI_ENTRY_CHASSIS; // island enum // gen:cbfbdbfffad7
pub const DMI_ENTRY_END_OF_TABLE: DMI_ENTRY_TYPE = DMI_ENTRY_TYPE_DMI_ENTRY_END_OF_TABLE; // island enum // gen:886911e9d9c2
pub const DMI_ENTRY_OEMSTRINGS: DMI_ENTRY_TYPE = DMI_ENTRY_TYPE_DMI_ENTRY_OEMSTRINGS; // island enum // gen:e4646ccd973d
pub const DMI_ENTRY_ONBOARD_DEVICE: DMI_ENTRY_TYPE = DMI_ENTRY_TYPE_DMI_ENTRY_ONBOARD_DEVICE; // island enum // gen:1e05f815d8a9
pub const DMI_ENTRY_SYSTEM: DMI_ENTRY_TYPE = DMI_ENTRY_TYPE_DMI_ENTRY_SYSTEM; // island enum // gen:cdd9871a9220
pub const DMI_SYS_FAMILY: _DMI_FIELD_OFFSETS = _DMI_FIELD_OFFSETS_DMI_SYS_FAMILY; // island enum // gen:3a90cfa363b7
pub const DMI_SYS_PRODUCT: _DMI_FIELD_OFFSETS = _DMI_FIELD_OFFSETS_DMI_SYS_PRODUCT; // island enum // gen:b10bf576823d
pub const DMI_SYS_SERIAL: _DMI_FIELD_OFFSETS = _DMI_FIELD_OFFSETS_DMI_SYS_SERIAL; // island enum // gen:70d2495fe11a
pub const DMI_SYS_SIZE: _DMI_FIELD_OFFSETS = _DMI_FIELD_OFFSETS_DMI_SYS_SIZE; // island enum // gen:1d44abc9ba42
pub const DMI_SYS_SKU: _DMI_FIELD_OFFSETS = _DMI_FIELD_OFFSETS_DMI_SYS_SKU; // island enum // gen:8690aff85177
pub const DMI_SYS_VENDOR: _DMI_FIELD_OFFSETS = _DMI_FIELD_OFFSETS_DMI_SYS_VENDOR; // island enum // gen:f5733c21406d
pub const DMI_SYS_VERSION: _DMI_FIELD_OFFSETS = _DMI_FIELD_OFFSETS_DMI_SYS_VERSION; // island enum // gen:8f1a8146a20e
pub const SYS_FAMILY: _ID_STRINGS = _ID_STRINGS_SYS_FAMILY; // island enum // gen:32aee00f86aa
pub const SYS_PRODUCT: _ID_STRINGS = _ID_STRINGS_SYS_PRODUCT; // island enum // gen:3d9267ded8ec
pub const SYS_SERIAL: _ID_STRINGS = _ID_STRINGS_SYS_SERIAL; // island enum // gen:c0c65b7354b3
pub const SYS_SKU: _ID_STRINGS = _ID_STRINGS_SYS_SKU; // island enum // gen:7bb346660eee
pub const SYS_VENDOR: _ID_STRINGS = _ID_STRINGS_SYS_VENDOR; // island enum // gen:b7ac26aa8d3f
pub const SYS_VERSION: _ID_STRINGS = _ID_STRINGS_SYS_VERSION; // island enum // gen:65d84d69114e
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbLengthW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
// ==== end port externs ====
