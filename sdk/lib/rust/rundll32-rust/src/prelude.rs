//! rundll32-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
extern "C" {
    pub fn free(p: *mut c_void); // atlclasses-rust/src/lib.rs
    pub fn malloc(size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
}
extern "system" {
    pub fn ActivateActCtx(arg1: HANDLE, arg2: *mut ULONG_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DeactivateActCtx(arg1: DWORD, arg2: ULONG_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnumWindows(lpEnumFunc: WNDENUMPROC, lParam: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_i386.rs
    pub fn GetWindow(arg1: HWND, arg2: UINT) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SearchPathW( lpPath: LPCWSTR, lpFileName: LPCWSTR, lpExtension: LPCWSTR, nBufferLength: DWORD, lpBuffer: LPWSTR, lpFilePart: *mut LPWSTR, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn SetActiveWindow(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn WideCharToMultiByte( arg1: UINT, arg2: DWORD, arg3: LPCWSTR, arg4: ::core::ffi::c_int, arg5: LPSTR, arg6: ::core::ffi::c_int, arg7: LPCSTR, arg8: LPBOOL, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _swprintf(_Dest: *mut wchar_t, _Format: *const wchar_t, ...) -> ::core::ffi::c_int; // rundll32.c
    pub fn _wcsicmp(_Str1: *const wchar_t, _Str2: *const wchar_t) -> ::core::ffi::c_int; // rundll32.c
    pub fn wcscpy(_Dest: *mut wchar_t, _Source: *const wchar_t) -> *mut wchar_t; // rundll32.c
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
}
extern "system" {
    pub fn CreateActCtxW(_: PCACTCTXW) -> HANDLE; // rundll32.c
    pub fn CreateWindowExW(
        dwExStyle: DWORD,
        lpClassName: LPCWSTR,
        lpWindowName: LPCWSTR,
        dwStyle: DWORD,
        X: ::core::ffi::c_int,
        Y: ::core::ffi::c_int,
        nWidth: ::core::ffi::c_int,
        nHeight: ::core::ffi::c_int,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HINSTANCE,
        lpParam: LPVOID,
    ) -> HWND; // rundll32.c
    pub fn DefWindowProcW(_: HWND, _: UINT, _: WPARAM, _: LPARAM) -> LRESULT; // rundll32.c
    pub fn GetCommandLineW() -> LPWSTR; // rundll32.c
    pub fn GetModuleFileNameW(hModule: HINSTANCE, lpFilename: LPWSTR, nSize: DWORD) -> DWORD; // rundll32.c
    pub fn GetModuleHandleW(_: LPCWSTR) -> HMODULE; // rundll32.c
    pub fn LoadLibraryW(_: LPCWSTR) -> HINSTANCE; // rundll32.c
    pub fn LoadStringW(
        hInstance: HINSTANCE,
        uID: UINT,
        lpBuffer: LPWSTR,
        cchBufferMax: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int; // rundll32.c
    pub fn MessageBoxW(
        hWnd: HWND,
        lpText: LPCWSTR,
        lpCaption: LPCWSTR,
        uType: UINT,
    ) -> ::core::ffi::c_int; // rundll32.c
    pub fn RegisterClassExW(_: *const WNDCLASSEXW) -> ATOM; // rundll32.c
    pub fn UnregisterClassW(_: LPCWSTR, _: HINSTANCE) -> BOOL; // rundll32.c
}
pub use crate::prelude::CreateActCtxW as CreateActCtx; // sdk alias
pub use crate::prelude::CreateWindowExW as CreateWindowEx; // sdk alias
pub use crate::prelude::DefWindowProcW as DefWindowProc; // sdk alias
pub use crate::prelude::GetCommandLineW as GetCommandLine; // sdk alias
pub use crate::prelude::GetModuleFileNameW as GetModuleFileName; // sdk alias
pub use crate::prelude::GetModuleHandleW as GetModuleHandle; // sdk alias
pub use crate::prelude::LoadLibraryW as LoadLibrary; // sdk alias
pub use crate::prelude::LoadStringW as LoadString; // sdk alias
pub use crate::prelude::MessageBoxW as MessageBox; // sdk alias
pub use crate::prelude::RegisterClassExW as RegisterClassEx; // sdk alias
pub use crate::prelude::UnregisterClassW as UnregisterClass; // sdk alias
pub use crate::prelude::_swprintf as _stprintf; // sdk alias
pub use crate::prelude::wcscpy as _tcscpy; // sdk alias
pub use crate::prelude::_wcsicmp as _tcsicmp; // sdk alias
pub use crate::prelude::wcslen as _tcslen; // sdk alias
// ==== end port externs ====

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
pub type ACTCTX = ACTCTXW;
pub type WNDCLASSEX = WNDCLASSEXW;
// ==== end UNICODE TCHAR family ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
pub const CW_USEDEFAULT: c_int = ((0x80000000u32) as c_int);
pub const INVALID_FILE_ATTRIBUTES: DWORD = (((-(1 as c_int))) as DWORD);
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// base/system/rundll32/rundll32.c:272: #define ConvertToMultiByte(lptString) DuplicateToMultiByte(lptString,0)
#[allow(unused_macros)]
macro_rules! ConvertToMultiByte {
    ($lptString:expr $(,)?) => {
        (DuplicateToMultiByte($lptString, 0))
    };
}
#[allow(unused_imports)]
pub(crate) use ConvertToMultiByte;
// base/system/rundll32/rundll32.c:245: #define ConvertToWideChar(lptString) (lptString)
#[allow(unused_macros)]
macro_rules! ConvertToWideChar {
    ($lptString:expr $(,)?) => {
        ($lptString)
    };
}
#[allow(unused_imports)]
pub(crate) use ConvertToWideChar;
// base/system/rundll32/rundll32.c:273: #define FreeConvertedMultiByte(lpaString) free(lpaString)
#[allow(unused_macros)]
macro_rules! FreeConvertedMultiByte {
    ($lpaString:expr $(,)?) => {
        (free($lpaString))
    };
}
#[allow(unused_imports)]
pub(crate) use FreeConvertedMultiByte;
// base/system/rundll32/rundll32.c:246: #define FreeConvertedWideChar(lpwString)
#[allow(unused_macros)]
macro_rules! FreeConvertedWideChar {
    ($lpwString:expr $(,)?) => {
        ()
    };
}
#[allow(unused_imports)]
pub(crate) use FreeConvertedWideChar;
// ==== end port macros ====
