//! msiexec-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// Hand port, OUTSIDE the generated blocks: report_error's `vsnprintf(buffer, .., msg, va_args)`.
// On every Windows target va_list is a plain `char *`, so it is declarable on stable (resolve
// refuses only Rust's nightly `VaList`); the shim hands report_error_v a pointer to its list.
extern "C" {
    pub fn vsnprintf(
        buf: *mut ::core::ffi::c_char,
        count: usize,
        format: *const ::core::ffi::c_char,
        args: va_list,
    ) -> ::core::ffi::c_int;
}

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const ADVERTISEFLAGS_MACHINEASSIGN: tagADVERTISEFLAGS = tagADVERTISEFLAGS_ADVERTISEFLAGS_MACHINEASSIGN; // island enum
pub const ADVERTISEFLAGS_USERASSIGN: tagADVERTISEFLAGS = tagADVERTISEFLAGS_ADVERTISEFLAGS_USERASSIGN; // island enum
pub const COINIT_MULTITHREADED: tagCOINIT = tagCOINIT_COINIT_MULTITHREADED; // island enum
pub const CP_ACP: u32 = 0; // advapi32-sys/src/bindgen_amd64.rs
pub const CSTR_EQUAL: u32 = 2; // appwiz-rust/src/types_amd64.rs
pub const INSTALLLOGATTRIBUTES_APPEND: tagINSTALLLOGATTRIBUTES = tagINSTALLLOGATTRIBUTES_INSTALLLOGATTRIBUTES_APPEND; // island enum
pub const INSTALLLOGATTRIBUTES_FLUSHEACHLINE: tagINSTALLLOGATTRIBUTES = tagINSTALLLOGATTRIBUTES_INSTALLLOGATTRIBUTES_FLUSHEACHLINE; // island enum
pub const INSTALLLOGMODE_ACTIONDATA: tagINSTALLLOGMODE = tagINSTALLLOGMODE_INSTALLLOGMODE_ACTIONDATA; // island enum
pub const INSTALLLOGMODE_ACTIONSTART: tagINSTALLLOGMODE = tagINSTALLLOGMODE_INSTALLLOGMODE_ACTIONSTART; // island enum
pub const INSTALLLOGMODE_COMMONDATA: tagINSTALLLOGMODE = tagINSTALLLOGMODE_INSTALLLOGMODE_COMMONDATA; // island enum
pub const INSTALLLOGMODE_ERROR: tagINSTALLLOGMODE = tagINSTALLLOGMODE_INSTALLLOGMODE_ERROR; // island enum
pub const INSTALLLOGMODE_FATALEXIT: tagINSTALLLOGMODE = tagINSTALLLOGMODE_INSTALLLOGMODE_FATALEXIT; // island enum
pub const INSTALLLOGMODE_INFO: tagINSTALLLOGMODE = tagINSTALLLOGMODE_INSTALLLOGMODE_INFO; // island enum
pub const INSTALLLOGMODE_INITIALIZE: tagINSTALLLOGMODE = tagINSTALLLOGMODE_INSTALLLOGMODE_INITIALIZE; // island enum
pub const INSTALLLOGMODE_OUTOFDISKSPACE: tagINSTALLLOGMODE = tagINSTALLLOGMODE_INSTALLLOGMODE_OUTOFDISKSPACE; // island enum
pub const INSTALLLOGMODE_PROGRESS: tagINSTALLLOGMODE = tagINSTALLLOGMODE_INSTALLLOGMODE_PROGRESS; // island enum
pub const INSTALLLOGMODE_PROPERTYDUMP: tagINSTALLLOGMODE = tagINSTALLLOGMODE_INSTALLLOGMODE_PROPERTYDUMP; // island enum
pub const INSTALLLOGMODE_RESOLVESOURCE: tagINSTALLLOGMODE = tagINSTALLLOGMODE_INSTALLLOGMODE_RESOLVESOURCE; // island enum
pub const INSTALLLOGMODE_SHOWDIALOG: tagINSTALLLOGMODE = tagINSTALLLOGMODE_INSTALLLOGMODE_SHOWDIALOG; // island enum
pub const INSTALLLOGMODE_TERMINATE: tagINSTALLLOGMODE = tagINSTALLLOGMODE_INSTALLLOGMODE_TERMINATE; // island enum
pub const INSTALLLOGMODE_USER: tagINSTALLLOGMODE = tagINSTALLLOGMODE_INSTALLLOGMODE_USER; // island enum
pub const INSTALLLOGMODE_VERBOSE: tagINSTALLLOGMODE = tagINSTALLLOGMODE_INSTALLLOGMODE_VERBOSE; // island enum
pub const INSTALLLOGMODE_WARNING: tagINSTALLLOGMODE = tagINSTALLLOGMODE_INSTALLLOGMODE_WARNING; // island enum
pub const INSTALLSTATE_DEFAULT: tagINSTALLSTATE = tagINSTALLSTATE_INSTALLSTATE_DEFAULT; // island enum
pub const INSTALLTYPE_DEFAULT: tagINSTALLTYPE = tagINSTALLTYPE_INSTALLTYPE_DEFAULT; // island enum
pub const INSTALLTYPE_NETWORK_IMAGE: tagINSTALLTYPE = tagINSTALLTYPE_INSTALLTYPE_NETWORK_IMAGE; // island enum
pub const INSTALLUILEVEL_BASIC: tagINSTALLUILEVEL = tagINSTALLUILEVEL_INSTALLUILEVEL_BASIC; // island enum
pub const INSTALLUILEVEL_ENDDIALOG: tagINSTALLUILEVEL = tagINSTALLUILEVEL_INSTALLUILEVEL_ENDDIALOG; // island enum
pub const INSTALLUILEVEL_FULL: tagINSTALLUILEVEL = tagINSTALLUILEVEL_INSTALLUILEVEL_FULL; // island enum
pub const INSTALLUILEVEL_HIDECANCEL: tagINSTALLUILEVEL = tagINSTALLUILEVEL_INSTALLUILEVEL_HIDECANCEL; // island enum
pub const INSTALLUILEVEL_NONE: tagINSTALLUILEVEL = tagINSTALLUILEVEL_INSTALLUILEVEL_NONE; // island enum
pub const INSTALLUILEVEL_PROGRESSONLY: tagINSTALLUILEVEL = tagINSTALLUILEVEL_INSTALLUILEVEL_PROGRESSONLY; // island enum
pub const INSTALLUILEVEL_REDUCED: tagINSTALLUILEVEL = tagINSTALLUILEVEL_INSTALLUILEVEL_REDUCED; // island enum
pub const NORM_IGNORECASE: u32 = 1; // appwiz-rust/src/types_amd64.rs
pub const REINSTALLMODE_FILEEQUALVERSION: tagREINSTALLMODE = tagREINSTALLMODE_REINSTALLMODE_FILEEQUALVERSION; // island enum
pub const REINSTALLMODE_FILEEXACT: tagREINSTALLMODE = tagREINSTALLMODE_REINSTALLMODE_FILEEXACT; // island enum
pub const REINSTALLMODE_FILEMISSING: tagREINSTALLMODE = tagREINSTALLMODE_REINSTALLMODE_FILEMISSING; // island enum
pub const REINSTALLMODE_FILEOLDERVERSION: tagREINSTALLMODE = tagREINSTALLMODE_REINSTALLMODE_FILEOLDERVERSION; // island enum
pub const REINSTALLMODE_FILEREPLACE: tagREINSTALLMODE = tagREINSTALLMODE_REINSTALLMODE_FILEREPLACE; // island enum
pub const REINSTALLMODE_FILEVERIFY: tagREINSTALLMODE = tagREINSTALLMODE_REINSTALLMODE_FILEVERIFY; // island enum
pub const REINSTALLMODE_MACHINEDATA: tagREINSTALLMODE = tagREINSTALLMODE_REINSTALLMODE_MACHINEDATA; // island enum
pub const REINSTALLMODE_PACKAGE: tagREINSTALLMODE = tagREINSTALLMODE_REINSTALLMODE_PACKAGE; // island enum
pub const REINSTALLMODE_SHORTCUT: tagREINSTALLMODE = tagREINSTALLMODE_REINSTALLMODE_SHORTCUT; // island enum
pub const REINSTALLMODE_USERDATA: tagREINSTALLMODE = tagREINSTALLMODE_REINSTALLMODE_USERDATA; // island enum
extern "C" {
    pub fn free(p: *mut c_void); // atlclasses-rust/src/lib.rs
    pub fn malloc(size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
    pub fn ros_dbg_log( cls: ::core::ffi::c_uint, ch: *mut __wine_debug_channel, file: *const ::core::ffi::c_char, func: *const ::core::ffi::c_char, line: ::core::ffi::c_int, format: *const ::core::ffi::c_char, ... ) -> ::core::ffi::c_int; // winegdi-rust/src/types_i386.rs
    // The COUNTED swprintf is a header inline over _vsnwprintf: msvcrt.dll's own `swprintf`
    // export is the legacy (buffer, format, ...) form, so the count would be read as the
    // format. `scan_swprintf_count` gates it.
    #[link_name = "_snwprintf"]
    pub fn swprintf(s: *mut u16, n: usize, fmt: *const u16, ...) -> c_int; // ole32-rust/src/crt.rs
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcstol(s: *const u16, endptr: *mut *mut u16, base: i32) -> i32; // mlang-rust/src/mlang/mlang.rs
    pub fn wine_dbg_printf(format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int; // winegdi-rust/src/types_i386.rs
}
extern "system" {
    pub fn CLSIDFromString(arg1: LPCOLESTR, arg2: LPCLSID) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CloseServiceHandle(arg1: SC_HANDLE) -> BOOL; // syssetup-rust/src/types_i386.rs
    pub fn CoInitializeEx(lpReserved: LPVOID, dwCoInit: DWORD) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoUninitialize(); // aclui-rust/src/types_i386.rs
    pub fn CompareStringW( Locale: LCID, dwCmpFlags: DWORD, lpString1: LPCWSTR, cchCount1: ::core::ffi::c_int, lpString2: LPCWSTR, cchCount2: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn CreateEventW( arg1: LPSECURITY_ATTRIBUTES, bManualReset: BOOL, bInitialState: BOOL, lpName: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateServiceW( arg1: SC_HANDLE, arg2: LPCWSTR, arg3: LPCWSTR, arg4: DWORD, arg5: DWORD, arg6: DWORD, arg7: DWORD, arg8: LPCWSTR, arg9: LPCWSTR, arg10: PDWORD, arg11: LPCWSTR, arg12: LPCWSTR, arg13: LPCWSTR, ) -> SC_HANDLE; // syssetup-rust/src/types_i386.rs
    pub fn CreateThread( arg1: LPSECURITY_ATTRIBUTES, arg2: DWORD, arg3: LPTHREAD_START_ROUTINE, arg4: PVOID, arg5: DWORD, arg6: PDWORD, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn DeleteService(arg1: SC_HANDLE) -> BOOL; // syssetup-rust/src/types_i386.rs
    pub fn ExitProcess(arg1: UINT) -> !; // aclui-rust/src/types_i386.rs
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetCommandLineW() -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetModuleFileNameW(hModule: HINSTANCE, lpFilename: LPWSTR, nSize: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetModuleHandleA(arg1: LPCSTR) -> HMODULE; // aclui-rust/src/types_i386.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_i386.rs
    pub fn GetSystemDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetThreadLocale() -> LCID; // desk-rust/src/types_i386.rs
    pub fn GetUserDefaultLangID() -> LANGID; // desk-rust/src/types_i386.rs
    pub fn InitCommonControls(); // aclui-rust/src/types_i386.rs
    pub fn LoadLibraryExW(arg1: LPCWSTR, arg2: HANDLE, arg3: DWORD) -> HINSTANCE; // aclui-rust/src/types_i386.rs
    pub fn LoadStringW( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn MultiByteToWideChar( arg1: UINT, arg2: DWORD, arg3: LPCSTR, arg4: ::core::ffi::c_int, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn OpenSCManagerW(arg1: LPCWSTR, arg2: LPCWSTR, arg3: DWORD) -> SC_HANDLE; // syssetup-rust/src/types_i386.rs
    pub fn OpenServiceW(arg1: SC_HANDLE, arg2: LPCWSTR, arg3: DWORD) -> SC_HANDLE; // syssetup-rust/src/types_i386.rs
    pub fn ReadFile( arg1: HANDLE, arg2: PVOID, arg3: DWORD, arg4: PDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegOpenKeyW(arg1: HKEY, arg2: LPCWSTR, arg3: PHKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegisterServiceCtrlHandlerA( arg1: LPCSTR, arg2: LPHANDLER_FUNCTION, ) -> SERVICE_STATUS_HANDLE; // syssetup-rust/src/types_i386.rs
    pub fn SetEvent(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetServiceStatus(arg1: SERVICE_STATUS_HANDLE, arg2: LPSERVICE_STATUS) -> BOOL; // syssetup-rust/src/types_i386.rs
    pub fn StartServiceCtrlDispatcherA(arg1: *const SERVICE_TABLE_ENTRYA) -> BOOL; // syssetup-rust/src/types_i386.rs
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn WriteFile( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn lstrcatW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrcpynW(arg1: LPWSTR, arg2: LPCWSTR, arg3: ::core::ffi::c_int) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrlenW(arg1: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// INSTALLTYPE: c2rust, msiexec.c
pub type INSTALLTYPE = tagINSTALLTYPE;
// INSTALLUILEVEL: c2rust, msiexec.c
pub type INSTALLUILEVEL = tagINSTALLUILEVEL;
// chomp_state: c2rust, msiexec.c
pub type chomp_state = ::core::ffi::c_uint;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn __wine_msi_call_dll_function(client_pid_0: DWORD, guid: *const GUID) -> UINT; // msiexec.c
}
extern "system" {
    pub fn MsiAdvertiseProductW(_: LPCWSTR, _: LPCWSTR, _: LPCWSTR, _: LANGID) -> UINT; // msiexec.c
    pub fn MsiApplyPatchW(_: LPCWSTR, _: LPCWSTR, _: INSTALLTYPE, _: LPCWSTR) -> UINT; // msiexec.c
    pub fn MsiConfigureProductExW(
        _: LPCWSTR,
        _: ::core::ffi::c_int,
        _: INSTALLSTATE,
        _: LPCWSTR,
    ) -> UINT; // msiexec.c
    pub fn MsiEnableLogW(_: DWORD, _: LPCWSTR, _: DWORD) -> UINT; // msiexec.c
    pub fn MsiGetFileVersionW(
        _: LPCWSTR,
        _: LPWSTR,
        pcchVersionBuf: LPDWORD,
        _: LPWSTR,
        pcchLangBuf: LPDWORD,
    ) -> UINT; // msiexec.c
    pub fn MsiInstallProductW(_: LPCWSTR, _: LPCWSTR) -> UINT; // msiexec.c
    pub fn MsiMessageBoxA(_: HWND, _: LPCSTR, _: LPCSTR, _: UINT, _: WORD, _: DWORD) -> UINT; // msiexec.c
    pub fn MsiMessageBoxW(_: HWND, _: LPCWSTR, _: LPCWSTR, _: UINT, _: WORD, _: DWORD) -> UINT; // msiexec.c
    pub fn MsiReinstallProductW(_: LPCWSTR, _: DWORD) -> UINT; // msiexec.c
    pub fn MsiSetInternalUI(_: INSTALLUILEVEL, _: *mut HWND) -> INSTALLUILEVEL; // msiexec.c
}
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_msiexec };
// ==== end wine debug channels ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
pub const HKEY_LOCAL_MACHINE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY);
pub const NOERROR: HRESULT = ((0) as HRESULT);
// ==== end SDK header macros ====

// ==== port variadic: C-variadic entry points, trampolines in the module's rust_varargs_shim.c (generated -- `rustconv port integrate`) ====
extern "C" {
    pub fn report_error(msg: *const ::core::ffi::c_char, ...);
}
// ==== end port variadic ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::msiexec::msiexec::__wine_dbch_msiexec;
// ==== end cross-TU definitions ====
