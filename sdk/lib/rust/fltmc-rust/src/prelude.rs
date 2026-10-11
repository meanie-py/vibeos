//! fltmc-rust prelude — the single import surface the translated modules use.
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

// ================= ATL ========================================================================
// ATL's CStringW/CComPtr/containers, ported once in atl-rust.
pub use atl_rust::{atl_format, CComPtr, CHeapPtr, CSimpleArray, CSimpleMap, CStringW};

// ================= CRT ========================================================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    /// msvcrt `wprintf` (cdecl, variadic): fltmc prints every message through it.
    pub fn wprintf(format: *const u16, ...) -> c_int;
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedDecrement, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, ZeroMemory, _countof, debugstr_a, debugstr_an, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, max, min, wide_bytes};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// fltmc.cpp's `LoadAndPrintString(ULONG MessageId, ...)`: a C-variadic DEFINITION, which stable
// Rust cannot write, so it is the macro it behaves as. Same steps as the C++: load the string
// into a CAtlStringW (no length cap, the process's own resources) and vwprintf the arguments
// through it -- here the variadic `wprintf` with the arguments spelled at the call site.
#[macro_export]
macro_rules! LoadAndPrintString {
    ($msg_id:expr $(, $arg:expr)* $(,)?) => {{
        let mut Message = CStringW::new();
        if Message.LoadStringW(($msg_id) as UINT) != 0 {
            wprintf(Message.GetString() $(, $arg)*);
        }
    }};
}
pub use crate::LoadAndPrintString;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const FLT_FSTYPE_EXFAT: _FLT_FILESYSTEM_TYPE = _FLT_FILESYSTEM_TYPE_FLT_FSTYPE_EXFAT; // island enum
pub const FLT_FSTYPE_FAT: _FLT_FILESYSTEM_TYPE = _FLT_FILESYSTEM_TYPE_FLT_FSTYPE_FAT; // island enum
pub const FLT_FSTYPE_MSFS: _FLT_FILESYSTEM_TYPE = _FLT_FILESYSTEM_TYPE_FLT_FSTYPE_MSFS; // island enum
pub const FLT_FSTYPE_MUP: _FLT_FILESYSTEM_TYPE = _FLT_FILESYSTEM_TYPE_FLT_FSTYPE_MUP; // island enum
pub const FLT_FSTYPE_NPFS: _FLT_FILESYSTEM_TYPE = _FLT_FILESYSTEM_TYPE_FLT_FSTYPE_NPFS; // island enum
pub const FLT_FSTYPE_NTFS: _FLT_FILESYSTEM_TYPE = _FLT_FILESYSTEM_TYPE_FLT_FSTYPE_NTFS; // island enum
pub const FLT_FSTYPE_UNKNOWN: _FLT_FILESYSTEM_TYPE = _FLT_FILESYSTEM_TYPE_FLT_FSTYPE_UNKNOWN; // island enum
pub const FilterAggregateStandardInformation: _FILTER_INFORMATION_CLASS = _FILTER_INFORMATION_CLASS_FilterAggregateStandardInformation; // island enum
pub const FilterFullInformation: _FILTER_INFORMATION_CLASS = _FILTER_INFORMATION_CLASS_FilterFullInformation; // island enum
pub const FilterVolumeStandardInformation: _FILTER_VOLUME_INFORMATION_CLASS = _FILTER_VOLUME_INFORMATION_CLASS_FilterVolumeStandardInformation; // island enum
pub const S_OK: HRESULT = 0; // aclui-rust/src/prelude.rs
extern "C" {
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // inflib-rust/src/infcore.rs
    pub fn vwprintf(format: *const wchar_t, valist: va_list,) -> i32; // msvcrt-crt-rust/src/msvcrt/file.rs
}
extern "system" {
    pub fn AdjustTokenPrivileges( arg1: HANDLE, arg2: BOOL, arg3: PTOKEN_PRIVILEGES, arg4: DWORD, arg5: PTOKEN_PRIVILEGES, arg6: PDWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FormatMessageW( dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD, lpBuffer: LPWSTR, nSize: DWORD, Arguments: *mut va_list, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetCurrentProcess() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn LoadStringW( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn LookupPrivilegeValueW(arg1: LPCWSTR, arg2: LPCWSTR, arg3: PLUID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn OpenProcessToken(arg1: HANDLE, arg2: DWORD, arg3: PHANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
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
// ==== end UNICODE TCHAR family ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! HRESULT_FROM_WIN32 { ($p0:expr $(,)?) => { (if ((((($p0) as HRESULT) > 0) as c_int)) as isize != 0 { ((((($p0 & 0x0000FFFF) | (7 << 16)) | (0x80000000u32 as HRESULT))) as HRESULT) } else { (($p0) as HRESULT) }) }; }
pub const SE_LOAD_DRIVER_NAME: *const u16 = (&[0x0053, 0x0065, 0x004c, 0x006f, 0x0061, 0x0064, 0x0044, 0x0072, 0x0069, 0x0076, 0x0065, 0x0072, 0x0050, 0x0072, 0x0069, 0x0076, 0x0069, 0x006c, 0x0065, 0x0067, 0x0065, 0x0000u16]).as_ptr(); // L"SeLoadDriverPrivilege"
pub use crate::{HRESULT_FROM_WIN32};
// ==== end SDK header macros ====
