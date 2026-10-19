//! propsys-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedDecrement64, InterlockedExchange, InterlockedExchangePointer, InterlockedIncrement, InterlockedIncrement64, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _byteswap_uint64, _byteswap_ulong, _byteswap_ushort, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_variant, debugstr_vt, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, lrintf, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_longlong, wine_dbgstr_variant, wine_dbgstr_vt, wine_dbgstr_w};

// winerror.h: E_NOT_SUFFICIENT_BUFFER is HRESULT_FROM_WIN32(ERROR_INSUFFICIENT_BUFFER = 122).
pub const E_NOT_SUFFICIENT_BUFFER: HRESULT = 0x8007_007Au32 as HRESULT;

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const VT_ARRAY: i32 = 0x2000; // clang, propvar.c
pub const VT_BOOL: i32 = 0xB; // clang, propvar.c
pub const VT_BSTR: i32 = 8; // clang, propvar.c
pub const VT_CLSID: i32 = 0x48; // clang, propvar.c
pub const VT_EMPTY: i32 = 0; // clang, propvar.c
pub const VT_I1: i32 = 0x10; // clang, propvar.c
pub const VT_I2: i32 = 2; // clang, propvar.c
pub const VT_I4: i32 = 3; // clang, propvar.c
pub const VT_I8: i32 = 0x14; // clang, propvar.c
pub const VT_LPSTR: i32 = 0x1E; // clang, propvar.c
pub const VT_LPWSTR: i32 = 0x1F; // clang, propvar.c
pub const VT_NULL: i32 = 1; // clang, propvar.c
pub const VT_R4: i32 = 4; // clang, propvar.c
pub const VT_R8: i32 = 5; // clang, propvar.c
pub const VT_UI1: i32 = 0x11; // clang, propvar.c
pub const VT_UI2: i32 = 0x12; // clang, propvar.c
pub const VT_UI4: i32 = 0x13; // clang, propvar.c
pub const VT_UI8: i32 = 0x15; // clang, propvar.c
pub const VT_VECTOR: i32 = 0x1000; // clang, propvar.c
// ==== end port facts ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const CP_ACP: u32 = 0; // advapi32-sys/src/bindgen_amd64.rs
pub const PSC_NORMAL: PSC_STATE = PSC_STATE_PSC_NORMAL; // island enum
pub const PVCF_DEFAULT: tagPROPVAR_COMPARE_FLAGS = tagPROPVAR_COMPARE_FLAGS_PVCF_DEFAULT; // island enum
pub const PVCF_TREATEMPTYASGREATERTHAN: tagPROPVAR_COMPARE_FLAGS = tagPROPVAR_COMPARE_FLAGS_PVCF_TREATEMPTYASGREATERTHAN; // island enum
pub const PVCF_USESTRCMPI: tagPROPVAR_COMPARE_FLAGS = tagPROPVAR_COMPARE_FLAGS_PVCF_USESTRCMPI; // island enum
pub const PVCF_USESTRCMPIC: tagPROPVAR_COMPARE_FLAGS = tagPROPVAR_COMPARE_FLAGS_PVCF_USESTRCMPIC; // island enum
pub const PVCU_DEFAULT: tagPROPVAR_COMPARE_UNIT = tagPROPVAR_COMPARE_UNIT_PVCU_DEFAULT; // island enum
pub const STRSAFE_E_INSUFFICIENT_BUFFER: HRESULT = 0x8007007Au32 as HRESULT; // mfplat-rust/src/prelude.rs
pub const VT_BYREF: u16 = 16384; // oleaut32-rust/src/consts.rs
pub const VT_DATE: DWORD = 7; // cabview-rust/src/prelude.rs
pub const VT_FILETIME: DWORD = 0x40; // ole32-rust/src/win_consts.rs
pub const VT_RECORD: u16 = 36; // oleaut32-rust/src/consts.rs
pub const VT_RESERVED: u16 = 32768; // oleaut32-rust/src/consts.rs
pub const VT_TYPEMASK: u16 = 4095; // oleaut32-rust/src/consts.rs
pub const VT_VARIANT: u16 = 12; // activeds-rust/src/prelude.rs
pub const VT_VOID: u16 = 24; // oleaut32-rust/src/consts.rs
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0; // comctl32-rust/src/prelude.rs
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3; // activeds-rust/src/prelude.rs
extern "C" {
    pub fn _strtoi64(nptr: *const u8, endptr: *mut *mut u8, base: i32) -> i64; // cnt-rust/src/cabi/generated.rs
    #[link_name = "_snwprintf"]
    pub fn swprintf(s: *mut u16, n: usize, fmt: *const u16, ...) -> c_int; // credui-rust/src/prelude.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // activeds-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcstol(s: *const u16, endptr: *mut *mut u16, base: i32) -> i32; // cleanmgr-rust/src/prelude.rs
}
extern "stdcall" {
    pub fn InitializeCriticalSectionEx( lpCriticalSection: LPCRITICAL_SECTION, dwSpinCount: u32, Flags: u32, ) -> BOOL; // msvcrt-sys/src/bindgen_i386.rs
}
extern "system" {
    pub fn CoTaskMemAlloc(size: SIZE_T) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemFree(ptr: LPVOID); // aclui-rust/src/types_i386.rs
    pub fn DeleteCriticalSection(arg1: PCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn FileTimeToSystemTime(arg1: *const FILETIME, arg2: LPSYSTEMTIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn InitializeCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn MultiByteToWideChar( arg1: UINT, arg2: DWORD, arg3: LPCSTR, arg4: ::core::ffi::c_int, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn PropVariantClear(pvar: *mut PROPVARIANT) -> HRESULT; // mf-rust/src/prelude.rs
    pub fn PropVariantCopy(pvarDest: *mut PROPVARIANT, pvarSrc: *const PROPVARIANT,) -> HRESULT; // mf-rust/src/prelude.rs
    pub fn SafeArrayAccessData( arg1: *mut SAFEARRAY, arg2: *mut *mut ::core::ffi::c_void, ) -> HRESULT; // msctf-rust/src/types_i386.rs
    pub fn SafeArrayCreateVector(arg1: ::core::ffi::c_ushort, arg2: LONG, arg3: ULONG) -> *mut SAFEARRAY; // msctf-rust/src/types_i386.rs
    pub fn SafeArrayDestroy(arg1: *mut SAFEARRAY) -> HRESULT; // msctf-rust/src/types_i386.rs
    pub fn SafeArrayUnaccessData(arg1: *mut SAFEARRAY) -> HRESULT; // msctf-rust/src/types_i386.rs
    pub fn StringFromGUID2(id: *const GUID, str_: LPOLESTR, cmax: INT) -> INT; // aclui-rust/src/types_i386.rs
    pub fn SysAllocString(arg1: *const OLECHAR) -> BSTR; // msctf-rust/src/types_i386.rs
    pub fn SysAllocStringLen(arg1: *const OLECHAR, ui: UINT) -> BSTR; // msctf-rust/src/types_i386.rs
    pub fn SysStringLen(pbstr: BSTR) -> UINT; // msctf-rust/src/types_i386.rs
    pub fn SystemTimeToVariantTime(arg1: LPSYSTEMTIME, arg2: *mut f64) -> INT; // msctf-rust/src/types_i386.rs
    pub fn VariantInit(arg1: *mut VARIANT); // msctf-rust/src/types_i386.rs
    pub fn WideCharToMultiByte( arg1: UINT, arg2: DWORD, arg3: LPCWSTR, arg4: ::core::ffi::c_int, arg5: LPSTR, arg6: ::core::ffi::c_int, arg7: LPCSTR, arg8: LPBOOL, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn lstrcmpA(arg1: LPCSTR, arg2: LPCSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn lstrcmpW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn lstrcmpiA(arg1: LPCSTR, arg2: LPCSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn lstrcmpiW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrcpynW(arg1: LPWSTR, arg2: LPCWSTR, arg3: ::core::ffi::c_int) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrlenW(arg1: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbLengthW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};
pub use winelist_rust::{list_add_after, list_add_before, list_add_head, list_add_tail, list_count, list_empty, list_head, list_init, list_move_head, list_move_tail, list_next, list_prev, list_remove, list_tail};

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
    ) -> ::core::ffi::c_int; // propstore.c
    pub fn sprintf(
        _Dest: *mut ::core::ffi::c_char,
        _Format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // propvar.c
}
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_propsys: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x70\x72\x6f\x70\x73\x79\x73\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_propsys };
// ==== end wine debug channels ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! IsEqualIID { ($p0:expr, $p1:expr $(,)?) => { (((memcmp((($p0) as _), (($p1) as _), ((core::mem::size_of::<GUID>()) as _))) as isize == 0) as c_int) }; } // gen:aa85287a8771
#[macro_export] macro_rules! PropVariantInit { ($p0:expr $(,)?) => { memset((($p0) as _), ((0) as _), ((core::mem::size_of::<PROPVARIANT>()) as _)) }; } // gen:f24837ee355e
#[macro_export] macro_rules! V_ARRAY { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.parray }; } // gen:3d65c5e7153c
#[macro_export] macro_rules! V_BOOL { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.boolVal }; } // gen:10deecc53a7c
#[macro_export] macro_rules! V_BSTR { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.bstrVal }; } // gen:4066e24ecd24
#[macro_export] macro_rules! V_BSTRREF { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.pbstrVal }; } // gen:01ec1384c039
#[macro_export] macro_rules! V_DATE { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.date }; } // gen:60f8d64781fb
#[macro_export] macro_rules! V_I1 { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.cVal }; } // gen:74fdc44e5de0
#[macro_export] macro_rules! V_I2 { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.iVal }; } // gen:de2ee070a5e0
#[macro_export] macro_rules! V_I4 { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.lVal }; } // gen:2acba8bc60fa
#[macro_export] macro_rules! V_I8 { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.llVal }; } // gen:3bad43130099
#[macro_export] macro_rules! V_ISBYREF { ($p0:expr $(,)?) => { ((*$p0).n1.n2.vt & VT_BYREF) }; } // gen:40fef418c834
#[macro_export] macro_rules! V_R4 { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.fltVal }; } // gen:8d89520730d8
#[macro_export] macro_rules! V_R8 { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.dblVal }; } // gen:ee78e50acb47
#[macro_export] macro_rules! V_UI1 { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.bVal }; } // gen:d29b710e65a0
#[macro_export] macro_rules! V_UI2 { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.uiVal }; } // gen:107b1969f865
#[macro_export] macro_rules! V_UI4 { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.ulVal }; } // gen:419fb51bc82e
#[macro_export] macro_rules! V_UI8 { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.ullVal }; } // gen:58d84a6f4116
#[macro_export] macro_rules! V_VARIANTREF { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.pvarVal }; } // gen:b322dad263ec
#[macro_export] macro_rules! V_VT { ($p0:expr $(,)?) => { (*$p0).n1.n2.vt }; } // gen:bb0eb6ad6416
pub const CLASS_E_CLASSNOTAVAILABLE: HRESULT = ((0x80040111u32) as HRESULT); // gen:1197fd83f980
pub const CLASS_E_NOAGGREGATION: HRESULT = ((0x80040110u32) as HRESULT); // gen:01676cfea511
pub const DISP_E_BADVARTYPE: HRESULT = ((0x80020008u32) as HRESULT); // gen:fe6e0ba7c0f1
pub const DISP_E_TYPEMISMATCH: HRESULT = ((0x80020005u32) as HRESULT); // gen:beebef7fa6c9
pub const E_FAIL: HRESULT = ((0x80004005u32) as HRESULT); // gen:1275c1cb9e8d
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT); // gen:cd8a52171603
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT); // gen:80daa5b2ffa6
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT); // gen:cbf314acc21c
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT); // gen:98bee37aa995
pub const E_POINTER: HRESULT = ((0x80004003u32) as HRESULT); // gen:f72eeacc5d42
#[inline] // gen:cadbabb17eeb
pub unsafe fn IClassFactory_QueryInterface(This: *mut IClassFactory, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:237289c7f569
pub unsafe fn IPropertyStoreCache_QueryInterface(This: *mut IPropertyStoreCache, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:e37b7d7f2726
pub unsafe fn IPropertyStoreCache_Release(This: *mut IPropertyStoreCache) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:a5a12fc7eb46
pub unsafe fn IPropertySystem_AddRef(This: *mut IPropertySystem) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:c539bbda09da
pub unsafe fn IPropertySystem_QueryInterface(This: *mut IPropertySystem, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:6ad0b58c268c
pub unsafe fn IUnknown_AddRef(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:38acc17a31e7
pub unsafe fn IUnknown_QueryInterface(This: *mut IUnknown, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
pub type REFCLSID = *const CLSID; // gen:4a91e3a9efb2
pub type REFIID = *const IID; // gen:c79168ffaea6
pub type REFPROPERTYKEY = *const PROPERTYKEY; // gen:66cf8ce5c3a9
pub type REFPROPVARIANT = *const PROPVARIANT; // gen:76fd0d58d084
pub const S_FALSE: HRESULT = ((1) as HRESULT); // gen:b1b8213e7dac
pub const S_OK: HRESULT = ((0) as HRESULT); // gen:9969a2c48b97
pub const TYPE_E_ELEMENTNOTFOUND: HRESULT = ((0x8002802Bu32) as HRESULT); // gen:2f00df1404e1
pub const VARIANT_TRUE: VARIANT_BOOL = 0xffffusize as VARIANT_BOOL; // gen:80f078b25726
pub use crate::{IsEqualIID, PropVariantInit, V_ARRAY, V_BOOL, V_BSTR, V_BSTRREF, V_DATE, V_I1, V_I2, V_I4, V_I8, V_ISBYREF, V_R4, V_R8, V_UI1, V_UI2, V_UI4, V_UI8, V_VARIANTREF, V_VT};
// ==== end SDK header macros ====
