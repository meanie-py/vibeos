//! apphelp-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedDecrement, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, ZeroMemory, _countof, debugstr_a, debugstr_an, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, max, min, wide_bytes};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const DOS_PATH: _PATH_TYPE = _PATH_TYPE_DOS_PATH; // island enum
pub const E_FAIL: HRESULT = 0x80004005u32 as HRESULT; // aclui-rust/src/prelude.rs
pub const E_NOTIMPL: HRESULT = 0x80004001u32 as HRESULT; // appwiz-rust/src/prelude.rs
pub const FILE_GENERIC_READ: u32 = 1179785; // atapi-rust/src/types_amd64.rs
pub const FILE_GENERIC_WRITE: u32 = 1179926; // atapi-rust/src/types_amd64.rs
pub const INVALID_FILE_ATTRIBUTES: DWORD = 0xFFFFFFFF; // kernel32-rust/src/file/fileinfo.rs
pub const SEI_FAIL: _SEI_LOG_LEVEL = _SEI_LOG_LEVEL_SEI_FAIL; // island enum
pub const SEI_INFO: _SEI_LOG_LEVEL = _SEI_LOG_LEVEL_SEI_INFO; // island enum
pub const SEI_MSG: _SEI_LOG_LEVEL = _SEI_LOG_LEVEL_SEI_MSG; // island enum
pub const SEI_WARN: _SEI_LOG_LEVEL = _SEI_LOG_LEVEL_SEI_WARN; // island enum
pub const SHIM_ERR: _SHIM_LOG_LEVEL = _SHIM_LOG_LEVEL_SHIM_ERR; // island enum
pub const SHIM_INFO: _SHIM_LOG_LEVEL = _SHIM_LOG_LEVEL_SHIM_INFO; // island enum
pub const SHIM_WARN: _SHIM_LOG_LEVEL = _SHIM_LOG_LEVEL_SHIM_WARN; // island enum
pub const VS_FFI_SIGNATURE: DWORD = 0xFEEF04BD; // version-rust/src/lib.rs
extern "C" {
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // inflib-rust/src/infcore.rs
    pub fn _wcsnicmp(s1: *const u16, s2: *const u16, n: usize) -> i32; // kernel32-rust/src/client/vdm.rs
    pub fn strcmp(a: *const c_char, b: *const c_char) -> c_int; // browser-rust/src/prelude.rs
    pub fn strlen(s: *const i8) -> usize; // advapi32-rust/src/prelude.rs
    pub fn toupper(c: i32) -> i32; // cnt-ctype/src/lib.rs
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int; // browser-rust/src/prelude.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcsncmp(a: *const u16, b: *const u16, n: usize) -> c_int; // desk-rust/src/prelude.rs
    pub fn wcspbrk(s: *const u16, set: *const u16) -> *const u16; // ucrt-rust/src/argv/wildcards.rs
    pub fn wcsrchr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/prelude.rs
}
extern "system" {
    pub fn BaseDumpAppcompatCache() -> BOOL; // kernel32-rust/src/client/appcache.rs
    pub fn BaseFlushAppcompatCache() -> BOOL; // kernel32-rust/src/client/appcache.rs
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn FindClose(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn FindFirstFileW(arg1: LPCWSTR, arg2: LPWIN32_FIND_DATAW) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn FindNextFileW(arg1: HANDLE, arg2: LPWIN32_FIND_DATAW) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetDriveTypeW(arg1: LPCWSTR) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetFileVersionInfoSizeW(arg1: LPCWSTR, arg2: PDWORD) -> DWORD; // recyclebin-rust/src/types_amd64.rs
    pub fn GetFileVersionInfoW(arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: PVOID) -> BOOL; // recyclebin-rust/src/types_amd64.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetLongPathNameW(arg1: LPCWSTR, arg2: LPWSTR, arg3: DWORD) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetSystemTimeAsFileTime(arg1: LPFILETIME); // aclui-rust/src/types_amd64.rs
    pub fn GetSystemWindowsDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn LdrInitShimEngineDynamic(BaseAddress: PVOID) -> BOOLEAN; // ntdll-rust/src/ldr/ldrapi.rs
    pub fn VerQueryValueW(arg1: LPCVOID, arg2: LPCWSTR, arg3: *mut LPVOID, arg4: PUINT) -> BOOL; // recyclebin-rust/src/types_amd64.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthW, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// LANGANDCODEPAGE: c2rust, sdbfileattr.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LANGANDCODEPAGE {
    pub language: WORD,
    pub code_page: WORD,
}
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

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::apphelp::sdbapi::SdbCloseDatabase;
pub use crate::apphelp::sdbapi::SdbFindFirstNamedTag;
pub use crate::apphelp::sdbread::SdbFindFirstTag;
pub use crate::apphelp::sdbread::SdbFindNextTag;
pub use crate::apphelp::sdbfileattr::SdbFreeFileAttributes;
pub use crate::apphelp::sdbread::SdbGetDatabaseID;
pub use crate::apphelp::sdbfileattr::SdbGetFileAttributes;
pub use crate::apphelp::sdbread::SdbGetFirstChild;
pub use crate::apphelp::sdbread::SdbGetNextChild;
pub use crate::apphelp::layer::SdbGetPermLayerKeys;
pub use crate::apphelp::sdbread::SdbGetStringTagPtr;
pub use crate::apphelp::sdbread::SdbGetTagDataSize;
pub use crate::apphelp::sdbread::SdbGetTagFromTagID;
pub use crate::apphelp::sdbapi::SdbOpenDatabase;
pub use crate::apphelp::sdbapi::SdbQueryDataExTagID;
pub use crate::apphelp::sdbread::SdbReadDWORDTag;
pub use crate::apphelp::sdbread::SdbReadQWORDTag;
pub use crate::apphelp::sdbapi::SdbTagToString;
pub use crate::apphelp::sdbapi::SdbpAlloc;
pub use crate::apphelp::sdbapi::SdbpFree;
pub use crate::apphelp::sdbread::SdbpGetString;
pub use crate::apphelp::sdbapi::SdbpReAlloc;
pub use crate::apphelp::sdbread::SdbpReadData;
pub use crate::apphelp::sdbapi::SdbpStrlen;
pub use crate::apphelp::sdbapi::SdbpStrsize;
pub use crate::apphelp::sdbstringtable::SdbpTableDestroy;
// ==== end cross-TU definitions ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! IMAGE_ORDINAL { ($p0:expr $(,)?) => { ($p0 & 0xffff) }; }
#[macro_export] macro_rules! SdbAlloc { ($p0:expr $(,)?) => { SdbpAlloc((($p0) as _)) }; }
#[macro_export] macro_rules! SdbFree { ($p0:expr $(,)?) => { SdbpFree((($p0) as _)) }; }
#[macro_export] macro_rules! SdbReAlloc { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { SdbpReAlloc((($p0) as _), (($p1) as _), (($p2) as _)) }; }
#[macro_export] macro_rules! SeiAlloc { ($p0:expr $(,)?) => { SdbpAlloc((($p0) as _)) }; }
#[macro_export] macro_rules! SeiFree { ($p0:expr $(,)?) => { SdbpFree((($p0) as _)) }; }
#[macro_export] macro_rules! _ReturnAddress { () => { __builtin_return_address(((0) as _)) }; }
#[macro_export] macro_rules! va_end { ($p0:expr $(,)?) => { __builtin_va_end((($p0) as _)) }; }
#[macro_export] macro_rules! va_start { ($p0:expr, $p1:expr $(,)?) => { __builtin_va_start((($p0) as _), (($p1) as _)) }; }
pub const SharedUserData: *mut KUSER_SHARED_DATA = 0x7ffe0000usize as *mut KUSER_SHARED_DATA;
pub use crate::{IMAGE_ORDINAL, SdbAlloc, SdbFree, SdbReAlloc, SeiAlloc, SeiFree, _ReturnAddress, va_end, va_start};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/appcompat/apphelp/shimeng.c:123: #define ARRAY_Append(Array, TypeOfArray) (TypeOfArray*)ARRAY_AppendWorker((Array), sizeof(TypeOfArray), 5)
#[allow(unused_macros)]
macro_rules! ARRAY_Append {
    ($array:expr, $t:ty $(,)?) => {
        (ARRAY_AppendWorker($array, core::mem::size_of::<$t>(), 5) as *mut $t)
    };
}
#[allow(unused_imports)]
pub(crate) use ARRAY_Append;
// dll/appcompat/apphelp/shimeng.c:124: #define ARRAY_At(Array, TypeOfArray, at) (TypeOfArray*)ARRAY_AtWorker((Array), sizeof(TypeOfArray), at)
#[allow(unused_macros)]
macro_rules! ARRAY_At {
    ($array:expr, $t:ty, $at:expr $(,)?) => {
        (ARRAY_AtWorker($array, core::mem::size_of::<$t>(), $at) as *mut $t)
    };
}
#[allow(unused_imports)]
pub(crate) use ARRAY_At;
// dll/appcompat/apphelp/shimeng.c:122: #define ARRAY_Init(Array, TypeOfArray) ARRAY_InitWorker((Array), sizeof(TypeOfArray))
#[allow(unused_macros)]
macro_rules! ARRAY_Init {
    ($array:expr, $t:ty $(,)?) => {
        (ARRAY_InitWorker($array, core::mem::size_of::<$t>()))
    };
}
#[allow(unused_imports)]
pub(crate) use ARRAY_Init;
// dll/appcompat/apphelp/shimeng.c:125: #define ARRAY_Size(Array) (Array)->Size__
#[allow(unused_macros)]
macro_rules! ARRAY_Size {
    ($array:expr $(,)?) => {
        ((*$array).Size__)
    };
}
#[allow(unused_imports)]
pub(crate) use ARRAY_Size;
// ==== end port macros ====

// ==== hand-written (apphelp): module-local C macros and declarations ====
// Append-only, outside the generated blocks above.

// sdbpapi.h:76-78 SHIM_ERR/SHIM_WARN/SHIM_INFO and shimlib.h:69-72 SHIMENG_MSG/FAIL/WARN/INFO are
// MACROS that share their names with the log-level enumerators, so a call is `SHIM_ERR!(fmt, ..)`;
// the bare name stays the enum constant. Like the C they gate on the module's debug level and
// attribute the message to `__FUNCTION__` (recovered from the type name of a local fn item).
#[doc(hidden)]
pub struct FnName([u8; 64]);
impl FnName {
    /// `full` is `core::any::type_name_of_val` of a fn item named `__f` declared in the
    /// enclosing function: `crate::mod::Func::__f`. The function name is the segment before it.
    pub fn of(full: &str) -> FnName {
        let mut out = FnName([0u8; 64]);
        let b = full.as_bytes();
        let mut end = b.len();
        if full.ends_with("::__f") {
            end -= 5;
        }
        let mut start = end;
        while start > 0 && !(b[start - 1] == b':') {
            start -= 1;
        }
        let mut i = 0;
        while start + i < end && i < 63 {
            out.0[i] = b[start + i];
            i += 1;
        }
        out
    }
    pub fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr() as *const c_char
    }
}
#[macro_export] macro_rules! SHIM_ERR {
    ($($arg:tt)*) => { $crate::__shim_log!(SHIM_ERR, $($arg)*) };
}
#[macro_export] macro_rules! SHIM_WARN {
    ($($arg:tt)*) => { $crate::__shim_log!(SHIM_WARN, $($arg)*) };
}
#[macro_export] macro_rules! SHIM_INFO {
    ($($arg:tt)*) => { $crate::__shim_log!(SHIM_INFO, $($arg)*) };
}
#[doc(hidden)]
#[macro_export] macro_rules! __shim_log {
    ($lvl:ident, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        if unsafe { $crate::apphelp::apphelp::g_ShimDebugLevel } != 0 {
            fn __f() {}
            let __n = $crate::prelude::FnName::of(core::any::type_name_of_val(&__f));
            unsafe {
                $crate::apphelp::apphelp::ShimDbgPrint($crate::prelude::$lvl, __n.as_ptr(), $fmt $(, $arg)*);
            }
        }
    }};
}
#[macro_export] macro_rules! SHIMENG_MSG {
    ($($arg:tt)*) => { $crate::__shimeng_log!(SEI_MSG, $($arg)*) };
}
#[macro_export] macro_rules! SHIMENG_FAIL {
    ($($arg:tt)*) => { $crate::__shimeng_log!(SEI_FAIL, $($arg)*) };
}
#[macro_export] macro_rules! SHIMENG_WARN {
    ($($arg:tt)*) => { $crate::__shimeng_log!(SEI_WARN, $($arg)*) };
}
#[macro_export] macro_rules! SHIMENG_INFO {
    ($($arg:tt)*) => { $crate::__shimeng_log!(SEI_INFO, $($arg)*) };
}
#[doc(hidden)]
#[macro_export] macro_rules! __shimeng_log {
    ($lvl:ident, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        if unsafe { $crate::apphelp::shimeng::g_ShimEngDebugLevel } != 0 {
            fn __f() {}
            let __n = $crate::prelude::FnName::of(core::any::type_name_of_val(&__f));
            unsafe {
                $crate::apphelp::shimeng::SeiDbgPrint($crate::prelude::$lvl, __n.as_ptr(), $fmt $(, $arg)*);
            }
        }
    }};
}
pub use crate::{SHIM_ERR, SHIM_WARN, SHIM_INFO, SHIMENG_MSG, SHIMENG_FAIL, SHIMENG_WARN, SHIMENG_INFO};

// strsafe.h A-forms. `StringCchPrintfExA` and `StringCchVPrintfExA` are `static __inline` there
// (no symbol), so they are macros over strsafe-rust's printf-ex worker with msvcrt's
// `_snprintf` / `_vsnprintf` handed in -- same arguments, bounds and truncation contract.
extern "C" {
    pub fn _vsnprintf(buf: *mut c_char, n: usize, fmt: *const c_char, ap: *mut c_char) -> c_int;
}
#[macro_export] macro_rules! StringCchPrintfExA {
    ($($arg:tt)*) => { strsafe_rust::RtlStringCchPrintfExA!($($arg)*) };
}
/// `va` is the C `va_list` VALUE (a `char*` on all three Windows targets).
#[macro_export] macro_rules! StringCchVPrintfExA {
    ($pszDest:expr, $cchDest:expr, $ppszDestEnd:expr, $pcchRemaining:expr, $dwFlags:expr,
     $pszFormat:expr, $va:expr $(,)?) => {{
        let pszDest: *mut core::ffi::c_char = $pszDest as _;
        let cchDest: usize = $cchDest as usize;
        let va: *mut core::ffi::c_char = $va as _;
        strsafe_rust::ntstrsafe::ntstrsafe_printf_ex_worker_a(
            pszDest, cchDest, cchDest,
            $ppszDestEnd as _, $pcchRemaining as _, $dwFlags as _, $pszFormat as _,
            |d, n, f| $crate::prelude::_vsnprintf(d, n, f, va),
        )
    }};
}
#[macro_export] macro_rules! StringCchPrintfA {
    ($d:expr, $cch:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        strsafe_rust::RtlStringCchPrintfExA!(
            $d, $cch,
            core::ptr::null_mut::<*mut core::ffi::c_char>(), core::ptr::null_mut::<usize>(),
            0u32, $fmt $(, $arg)*)
    };
}
pub use crate::{StringCchPrintfExA, StringCchVPrintfExA, StringCchPrintfA};

// Sibling-TU symbol (apphelp.c); the module uses it everywhere.
pub use crate::apphelp::apphelp::ShimDbgPrint;

// ReactOS ndk RTL_INIT_OBJECT_ATTRIBUTES(n, a): { sizeof(OBJECT_ATTRIBUTES), NULL, n, a, NULL, NULL }.
#[macro_export] macro_rules! RTL_INIT_OBJECT_ATTRIBUTES {
    ($n:expr, $a:expr $(,)?) => {
        _OBJECT_ATTRIBUTES {
            Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
            RootDirectory: core::ptr::null_mut(),
            ObjectName: ($n) as PUNICODE_STRING,
            Attributes: ($a) as ULONG,
            SecurityDescriptor: core::ptr::null_mut(),
            SecurityQualityOfService: core::ptr::null_mut(),
        }
    };
}
pub use crate::RTL_INIT_OBJECT_ATTRIBUTES;

// winnt.h IMAGE_SNAP_BY_ORDINAL: top bit of the pointer-width ordinal word (0x80000000 on
// i386, 0x8000000000000000 on amd64/arm64), the width of ULONG_PTR.
#[cfg(target_pointer_width = "64")]
pub const IMAGE_ORDINAL_FLAG: usize = 0x8000000000000000;
#[cfg(target_pointer_width = "32")]
pub const IMAGE_ORDINAL_FLAG: usize = 0x80000000;
#[macro_export] macro_rules! IMAGE_SNAP_BY_ORDINAL {
    ($o:expr $(,)?) => { ((($o) as usize) & $crate::prelude::IMAGE_ORDINAL_FLAG) != 0 };
}
pub use crate::IMAGE_SNAP_BY_ORDINAL;
// ==== end hand-written (apphelp) ====

// ==== hand-written (apphelp): RTL_CONSTANT_STRING for statics ====
// win32macros-rust's RTL_CONSTANT_STRING measures through a trait method, which is not callable
// in a `static` initializer. These two are `const`-evaluable: the wide form takes the NAME of a
// `[u16; N]` static (C's `sizeof` counts the NUL: Length = sizeof - 2, MaximumLength = sizeof);
// the ANSI form takes a NUL-terminated byte string literal (ANSI_STRING, ntdef.h).
#[macro_export] macro_rules! RTL_CONSTANT_STRING_STATIC {
    ($arr:path $(,)?) => {
        _UNICODE_STRING {
            Length: (core::mem::size_of_val(&$arr) - core::mem::size_of::<u16>()) as u16,
            MaximumLength: core::mem::size_of_val(&$arr) as u16,
            Buffer: $arr.as_ptr() as *mut u16,
        }
    };
}
#[macro_export] macro_rules! RTL_CONSTANT_STRING_A {
    ($s:expr $(,)?) => {
        _STRING {
            Length: ($s.len() - 1) as u16,
            MaximumLength: $s.len() as u16,
            Buffer: $s.as_ptr() as *mut core::ffi::c_char,
        }
    };
}
pub use crate::{RTL_CONSTANT_STRING_STATIC, RTL_CONSTANT_STRING_A};
// ==== end RTL_CONSTANT_STRING for statics ====

// ==== hand-written (apphelp): SDK inlines the island does not carry ====
// winnt.h InterlockedCompareExchangePointer: returns the value `*Destination` held before.
#[inline(always)]
pub unsafe fn InterlockedCompareExchangePointer(Destination: *mut PVOID, Exchange: PVOID, Comperand: PVOID) -> PVOID {
    let a = &*(Destination as *const core::sync::atomic::AtomicPtr<c_void>);
    match a.compare_exchange(Comperand, Exchange, core::sync::atomic::Ordering::SeqCst, core::sync::atomic::Ordering::SeqCst) {
        Ok(v) | Err(v) => v,
    }
}
// strsafe.h StringCbCatNW(pszDest, cbDest, pszSrc, cbToAppend): the byte counts become
// WCHAR counts exactly as the header does (cchDest = cbDest / sizeof(wchar), same for
// cchToAppend), then it is StringCchCatNW.
#[inline(always)]
pub unsafe fn StringCbCatNW(pszDest: *mut WCHAR, cbDest: usize, pszSrc: *const WCHAR, cbToAppend: usize) -> HRESULT {
    strsafe_rust::StringCchCatNW(
        pszDest,
        cbDest / core::mem::size_of::<WCHAR>(),
        pszSrc,
        cbToAppend / core::mem::size_of::<WCHAR>(),
    )
}
// shimeng.c:1118-1119 #define SYSTEM32 L"\\system32" / WINSXS L"\\winsxs" (function-local there;
// the Rust uses them as `*const WCHAR`).
pub static SYSTEM32: [u16; 10] = [0x5C, 0x73, 0x79, 0x73, 0x74, 0x65, 0x6D, 0x33, 0x32, 0];
pub static WINSXS: [u16; 8] = [0x5C, 0x77, 0x69, 0x6E, 0x73, 0x78, 0x73, 0];
// ==== end SDK inlines ====

// ndk/rtlfuncs.h's RtlInitEmptyUnicodeString is FORCEINLINE: no DLL exports it, so the island's
// extern declaration of it is unresolvable at link time. The C's three stores.
pub unsafe fn RtlInitEmptyUnicodeString(UnicodeString: PUNICODE_STRING, Buffer: PWCHAR, BufferSize: USHORT) {
    (*UnicodeString).Length = 0;
    (*UnicodeString).MaximumLength = BufferSize;
    (*UnicodeString).Buffer = Buffer;
}

// ndk/rtlfuncs.h's RTL_BUFFER helpers are FORCEINLINE (NTOS_MODE_USER): no DLL exports them, so
// the island's extern declarations of them are unresolvable at link time. Transcribed from the
// header; only RtlpEnsureBufferSize (the slow path) is an ntdll export.
extern "system" {
    pub fn RtlpEnsureBufferSize(Flags: ULONG, Buffer: PRTL_BUFFER, RequiredSize: SIZE_T) -> NTSTATUS;
}
pub unsafe fn RtlInitBuffer(Buffer: PRTL_BUFFER, Data: PUCHAR, DataSize: ULONG) {
    (*Buffer).StaticBuffer = Data;
    (*Buffer).Buffer = Data;
    (*Buffer).StaticSize = DataSize as _;
    (*Buffer).Size = DataSize as _;
    (*Buffer).ReservedForAllocatedSize = 0;
    (*Buffer).ReservedForIMalloc = core::ptr::null_mut();
}
pub unsafe fn RtlEnsureBufferSize(Flags: ULONG, Buffer: PRTL_BUFFER, RequiredSize: ULONG) -> NTSTATUS {
    if !Buffer.is_null() && RequiredSize as SIZE_T <= (*Buffer).Size as SIZE_T {
        return STATUS_SUCCESS as NTSTATUS;
    }
    RtlpEnsureBufferSize(Flags, Buffer, RequiredSize as SIZE_T)
}
pub unsafe fn RtlFreeBuffer(Buffer: PRTL_BUFFER) {
    // RtlGetProcessHeap() is `NtCurrentPeb()->ProcessHeap`.
    if (*Buffer).Buffer != (*Buffer).StaticBuffer && !(*Buffer).Buffer.is_null() {
        RtlFreeHeap((*NtCurrentPeb()).ProcessHeap, 0, (*Buffer).Buffer as PVOID);
    }
    (*Buffer).Buffer = (*Buffer).StaticBuffer;
    (*Buffer).Size = (*Buffer).StaticSize;
}
