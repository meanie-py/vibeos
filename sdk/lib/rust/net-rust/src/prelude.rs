//! net-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const ENABLE_ECHO_INPUT: u32 = 4; // cmd-rust/src/types_amd64.rs
pub const ENABLE_LINE_INPUT: u32 = 2; // cmd-rust/src/types_amd64.rs
pub const LANG_USER_DEFAULT: u32 = 1024; // cacls-rust/src/prelude.rs
pub const LM20_PWLEN: u32 = 14; // localspl-rust/src/types_amd64.rs
pub const LOCALE_USER_DEFAULT: DWORD = 0x0400; // cmd-rust/src/prelude.rs
pub const MAX_LANMAN_MESSAGE_ID: u32 = 5799; // localspl-rust/src/types_amd64.rs
pub const MAX_PREFERRED_LENGTH: DWORD = 0xFFFF_FFFF; // netapi32-rust/src/prelude_common.rs
pub const PWLEN: u32 = 256; // localspl-rust/src/types_amd64.rs
pub const STD_INPUT_HANDLE: u32 = 4294967286; // cmd-rust/src/prelude.rs
pub const TIMEQ_FOREVER: ULONG = 0xFFFF_FFFF; // netapi32-rust/src/prelude_common.rs
pub const ULONG_MAX: u32 = 4294967295; // msv1_0-rust/src/types_amd64.rs
extern "C" {
    pub fn ConMsgPrintfV(Stream: *mut CON_STREAM, dwFlags: u32, lpSource: *const c_void, dwMessageId: u32, dwLanguageId: u32, Arguments: *mut va_list,) -> i32; // conutils-rust/src/conutils/outstream.rs
    pub fn _swprintf( buffer: *mut u16, format: *const u16, ... ) -> ::core::ffi::c_int; // shell32-rust/src/prelude.rs
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // control-rust/src/prelude.rs
    pub fn _wcslwr(s: LPWSTR) -> LPWSTR; // shell32-rust/src/prelude.rs
    pub fn _wcsnicmp(s1: *const u16, s2: *const u16, n: usize) -> i32; // kernel32-rust/src/client/vdm.rs
    pub fn printf(format: *const i8, ...) -> i32; // oleaut32-rust/src/olepicture.rs
    pub fn qsort(base: *mut c_void, nmemb: usize, size: usize, compar: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>); // gdi32-rust/src/objects/font.rs
    pub fn rand() -> c_int; // dhcpcsvc-rust/src/prelude.rs
    pub fn srand(seed: c_uint); // dhcpcsvc-rust/src/prelude.rs
    pub fn towupper(c: c_int) -> c_int; // dnsapi-rust/src/prelude.rs
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int; // browser-rust/src/prelude.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcsncmp(a: *const u16, b: *const u16, n: usize) -> c_int; // desk-rust/src/prelude.rs
    pub fn wcsncpy(d: *mut u16, s: *const u16, n: usize) -> *mut u16; // kernel32-rust/src/prelude.rs
    pub fn wcspbrk(s: *const u16, set: *const u16) -> *const u16; // ucrt-rust/src/argv/wildcards.rs
    pub fn wcsstr(haystack: LPCWSTR, needle: LPCWSTR) -> LPWSTR; // recyclebin-rust/src/shellrecyclebin/RecycleBin5.rs
    pub fn wcstol(s: *const u16, endptr: *mut *mut u16, base: i32) -> i32; // mlang-rust/src/mlang/mlang.rs
    pub fn wcstoul(s: *const u16, end: *mut *mut u16, base: c_int) -> ULONG; // kernel32-rust/src/prelude.rs
}
extern "stdcall" {
    pub fn GetConsoleMode(hConsoleHandle: *mut ::core::ffi::c_void, lpMode: LPDWORD) -> BOOL; // msvcrt-sys/src/bindgen_i386.rs
    pub fn SetConsoleMode(hConsoleHandle: *mut ::core::ffi::c_void, dwMode: u32) -> BOOL; // msvcrt-sys/src/bindgen_i386.rs
}
extern "system" {
    pub fn CloseServiceHandle(arg1: SC_HANDLE) -> BOOL; // syssetup-rust/src/types_i386.rs
    pub fn ControlService(arg1: SC_HANDLE, arg2: DWORD, arg3: LPSERVICE_STATUS) -> BOOL; // syssetup-rust/src/types_i386.rs
    pub fn EnumServicesStatusW( hSCManager: SC_HANDLE, dwServiceType: DWORD, dwServiceState: DWORD, lpServices: LPENUM_SERVICE_STATUSW, cbBufSize: DWORD, pcbBytesNeeded: LPDWORD, lpServicesReturned: LPDWORD, lpResumeHandle: LPDWORD, ) -> BOOL; // syssetup-rust/src/types_i386.rs
    pub fn FileTimeToLocalFileTime(arg1: *const FILETIME, arg2: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FileTimeToSystemTime(arg1: *const FILETIME, arg2: LPSYSTEMTIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FormatMessageW( dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD, lpBuffer: LPWSTR, nSize: DWORD, Arguments: *mut va_list, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetDateFormatW( arg1: LCID, arg2: DWORD, arg3: *const SYSTEMTIME, arg4: LPCWSTR, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetLocaleInfoW( Locale: LCID, LCType: LCTYPE, lpLCData: LPWSTR, cchData: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn GetModuleHandleW(arg1: LPCWSTR) -> HMODULE; // aclui-rust/src/types_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetStdHandle(arg1: DWORD) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetSystemDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetSystemTimeAsFileTime(arg1: LPFILETIME); // aclui-rust/src/types_i386.rs
    pub fn GetTickCount() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetTimeFormatW( arg1: LCID, arg2: DWORD, arg3: *const SYSTEMTIME, arg4: LPCWSTR, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn GetTimeZoneInformation(arg1: LPTIME_ZONE_INFORMATION) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LocalFileTimeToFileTime(arg1: *const FILETIME, arg2: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn MultiByteToWideChar( arg1: UINT, arg2: DWORD, arg3: LPCSTR, arg4: ::core::ffi::c_int, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn NetApiBufferFree(arg1: *mut c_void) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetGroupAdd(arg1: LPCWSTR, arg2: u32, arg3: PBYTE, arg4: PDWORD) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetGroupAddUser(arg1: LPCWSTR, arg2: LPCWSTR, arg3: LPCWSTR) -> DWORD; // secur32-sys/src/bindgen_i386.rs
    pub fn NetGroupDel(arg1: LPCWSTR, arg2: LPCWSTR) -> DWORD; // secur32-sys/src/bindgen_i386.rs
    pub fn NetGroupDelUser(arg1: LPCWSTR, arg2: LPCWSTR, arg3: LPCWSTR) -> DWORD; // secur32-sys/src/bindgen_i386.rs
    pub fn NetGroupEnum( arg1: LPCWSTR, arg2: u32, arg3: *mut PBYTE, arg4: u32, arg5: PDWORD, arg6: PDWORD, arg7: PDWORD_PTR, ) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetGroupGetInfo(arg1: LPCWSTR, arg2: LPCWSTR, arg3: u32, arg4: *mut PBYTE) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetGroupGetUsers( arg1: LPCWSTR, arg2: LPCWSTR, arg3: u32, arg4: *mut PBYTE, arg5: u32, arg6: PDWORD, arg7: PDWORD, arg8: PDWORD_PTR, ) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetGroupSetInfo( arg1: LPCWSTR, arg2: LPCWSTR, arg3: u32, arg4: PBYTE, arg5: PDWORD, ) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetLocalGroupAdd(arg1: LPCWSTR, arg2: u32, arg3: PBYTE, arg4: PDWORD) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetLocalGroupAddMembers( arg1: LPCWSTR, arg2: LPCWSTR, arg3: u32, arg4: PBYTE, arg5: u32, ) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetLocalGroupDel(arg1: LPCWSTR, arg2: LPCWSTR) -> DWORD; // secur32-sys/src/bindgen_i386.rs
    pub fn NetLocalGroupDelMembers( arg1: LPCWSTR, arg2: LPCWSTR, arg3: u32, arg4: PBYTE, arg5: u32, ) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetLocalGroupEnum( arg1: LPCWSTR, arg2: u32, arg3: *mut PBYTE, arg4: u32, arg5: PDWORD, arg6: PDWORD, arg7: PDWORD_PTR, ) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetLocalGroupGetInfo( arg1: LPCWSTR, arg2: LPCWSTR, arg3: u32, arg4: *mut PBYTE, ) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetLocalGroupGetMembers( arg1: LPCWSTR, arg2: LPCWSTR, arg3: u32, arg4: *mut PBYTE, arg5: u32, arg6: PDWORD, arg7: PDWORD, arg8: PDWORD_PTR, ) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetLocalGroupSetInfo( arg1: LPCWSTR, arg2: LPCWSTR, arg3: u32, arg4: PBYTE, arg5: PDWORD, ) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetServerGetInfo(arg1: LPWSTR, arg2: u32, arg3: *mut PBYTE) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetServerSetInfo(arg1: LPWSTR, arg2: u32, arg3: PBYTE, arg4: PDWORD) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetServerTransportEnum( arg1: LPWSTR, arg2: u32, arg3: *mut PBYTE, arg4: u32, arg5: PDWORD, arg6: PDWORD, arg7: PDWORD, ) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetSessionDel(arg1: LPWSTR, arg2: LPWSTR, arg3: LPWSTR) -> DWORD; // secur32-sys/src/bindgen_i386.rs
    pub fn NetSessionEnum( arg1: LPWSTR, arg2: LPWSTR, arg3: LPWSTR, arg4: ::core::ffi::c_ulong, arg5: *mut LPBYTE, arg6: ::core::ffi::c_ulong, arg7: LPDWORD, arg8: LPDWORD, arg9: LPDWORD, ) -> DWORD; // secur32-sys/src/bindgen_i386.rs
    pub fn NetShareAdd(arg1: LPWSTR, arg2: ::core::ffi::c_ulong, arg3: LPBYTE, arg4: LPDWORD) -> DWORD; // secur32-sys/src/bindgen_i386.rs
    pub fn NetShareDel(arg1: LPWSTR, arg2: LPWSTR, arg3: ::core::ffi::c_ulong) -> DWORD; // secur32-sys/src/bindgen_i386.rs
    pub fn NetShareEnum( arg1: LPWSTR, arg2: ::core::ffi::c_ulong, arg3: *mut LPBYTE, arg4: ::core::ffi::c_ulong, arg5: LPDWORD, arg6: LPDWORD, arg7: LPDWORD, ) -> DWORD; // secur32-sys/src/bindgen_i386.rs
    pub fn NetShareGetInfo(arg1: LPWSTR, arg2: LPWSTR, arg3: ::core::ffi::c_ulong, arg4: *mut LPBYTE) -> DWORD; // secur32-sys/src/bindgen_i386.rs
    pub fn NetStatisticsGet( arg1: LPWSTR, arg2: LPWSTR, arg3: ::core::ffi::c_ulong, arg4: ::core::ffi::c_ulong, arg5: *mut PBYTE, ) -> DWORD; // secur32-sys/src/bindgen_i386.rs
    pub fn NetUserAdd(arg1: LPCWSTR, arg2: u32, arg3: PBYTE, arg4: PDWORD) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetUserDel(arg1: LPCWSTR, arg2: LPCWSTR) -> DWORD; // secur32-sys/src/bindgen_i386.rs
    pub fn NetUserEnum( arg1: LPCWSTR, arg2: u32, arg3: u32, arg4: *mut PBYTE, arg5: u32, arg6: PDWORD, arg7: PDWORD, arg8: PDWORD, ) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetUserGetGroups( arg1: LPCWSTR, arg2: LPCWSTR, arg3: u32, arg4: *mut PBYTE, arg5: u32, arg6: PDWORD, arg7: PDWORD, ) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetUserGetInfo(arg1: LPCWSTR, arg2: LPCWSTR, arg3: u32, arg4: *mut PBYTE) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetUserGetLocalGroups( arg1: LPCWSTR, arg2: LPCWSTR, arg3: u32, arg4: u32, arg5: *mut PBYTE, arg6: u32, arg7: PDWORD, arg8: PDWORD, ) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetUserModalsGet(arg1: LPCWSTR, arg2: u32, arg3: *mut PBYTE) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetUserModalsSet(arg1: LPCWSTR, arg2: u32, arg3: PBYTE, arg4: PDWORD) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetUserSetInfo( arg1: LPCWSTR, arg2: LPCWSTR, arg3: u32, arg4: PBYTE, arg5: PDWORD, ) -> DWORD; // shell32-rust/src/island_i386.rs
    pub fn NetWkstaGetInfo(arg1: LPWSTR, arg2: ::core::ffi::c_ulong, arg3: *mut PBYTE) -> DWORD; // secur32-sys/src/bindgen_i386.rs
    pub fn NetWkstaTransportEnum( arg1: LPWSTR, arg2: ::core::ffi::c_ulong, arg3: *mut PBYTE, arg4: ::core::ffi::c_ulong, arg5: PDWORD, arg6: PDWORD, arg7: PDWORD, ) -> DWORD; // secur32-sys/src/bindgen_i386.rs
    pub fn NetWkstaUserGetInfo(arg1: LPWSTR, arg2: ::core::ffi::c_ulong, arg3: *mut PBYTE) -> DWORD; // secur32-sys/src/bindgen_i386.rs
    pub fn OpenSCManagerW(arg1: LPCWSTR, arg2: LPCWSTR, arg3: DWORD) -> SC_HANDLE; // syssetup-rust/src/types_i386.rs
    pub fn OpenServiceW(arg1: SC_HANDLE, arg2: LPCWSTR, arg3: DWORD) -> SC_HANDLE; // syssetup-rust/src/types_i386.rs
    pub fn ReadFile( arg1: HANDLE, arg2: PVOID, arg3: DWORD, arg4: PDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn StartServiceW( hService: SC_HANDLE, dwNumServiceArgs: DWORD, lpServiceArgVectors: *mut LPCWSTR, ) -> BOOL; // syssetup-rust/src/types_i386.rs
    pub fn SystemTimeToFileTime(arg1: *const SYSTEMTIME, arg2: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn WNetCloseEnum(arg1: *mut c_void) -> DWORD; // shell32-rust/src/island_i386.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::net::main::hModuleNetMsg;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const MIN_LANMAN_MESSAGE_ID: i32 = 0x834; // clang, main.c
pub const MSG_ACCOUNTS_HELP: DWORD = 0x2711; // clang, cmdAccounts.c
pub const MSG_ACCOUNTS_SYNTAX: DWORD = 0x2710; // clang, cmdAccounts.c
pub const MSG_COMPUTER_HELP: DWORD = 0x2713; // clang, cmdComputer.c
pub const MSG_COMPUTER_SYNTAX: DWORD = 0x2712; // clang, cmdComputer.c
pub const MSG_CONFIG_HELP: DWORD = 0x2715; // clang, cmdConfig.c
pub const MSG_CONFIG_SERVER_HELP: DWORD = 0x2717; // clang, cmdConfig.c
pub const MSG_CONFIG_SERVER_SYNTAX: DWORD = 0x2716; // clang, cmdConfig.c
pub const MSG_CONFIG_SYNTAX: DWORD = 0x2714; // clang, cmdConfig.c
pub const MSG_CONTINUE_HELP: DWORD = 0x2719; // clang, cmdContinue.c
pub const MSG_CONTINUE_SYNTAX: DWORD = 0x2718; // clang, cmdContinue.c
pub const MSG_FILE_HELP: DWORD = 0x271B; // clang, cmdHelp.c
pub const MSG_FILE_SYNTAX: DWORD = 0x271A; // clang, cmdHelp.c
pub const MSG_GROUP_HELP: DWORD = 0x271D; // clang, cmdGroup.c
pub const MSG_GROUP_SYNTAX: DWORD = 0x271C; // clang, cmdGroup.c
pub const MSG_HELPMSG_HELP: DWORD = 0x2721; // clang, cmdHelp.c
pub const MSG_HELPMSG_SYNTAX: DWORD = 0x2720; // clang, cmdHelp.c
pub const MSG_HELP_HELP: DWORD = 0x271F; // clang, cmdHelp.c
pub const MSG_HELP_SYNTAX: DWORD = 0x271E; // clang, cmdHelp.c
pub const MSG_LOCALGROUP_HELP: DWORD = 0x2723; // clang, cmdHelp.c
pub const MSG_LOCALGROUP_SYNTAX: DWORD = 0x2722; // clang, cmdHelp.c
pub const MSG_NET_SYNTAX: DWORD = 0x2738; // clang, main.c
pub const MSG_PAUSE_HELP: DWORD = 0x2725; // clang, cmdHelp.c
pub const MSG_PAUSE_SYNTAX: DWORD = 0x2724; // clang, cmdHelp.c
pub const MSG_SESSION_HELP: DWORD = 0x2727; // clang, cmdHelp.c
pub const MSG_SESSION_SYNTAX: DWORD = 0x2726; // clang, cmdHelp.c
pub const MSG_SHARE_HELP: DWORD = 0x2729; // clang, cmdHelp.c
pub const MSG_SHARE_SYNTAX: DWORD = 0x2728; // clang, cmdHelp.c
pub const MSG_START_HELP: DWORD = 0x272B; // clang, cmdHelp.c
pub const MSG_START_SYNTAX: DWORD = 0x272A; // clang, cmdHelp.c
pub const MSG_STATISTICS_HELP: DWORD = 0x272D; // clang, cmdHelp.c
pub const MSG_STATISTICS_SYNTAX: DWORD = 0x272C; // clang, cmdHelp.c
pub const MSG_STOP_HELP: DWORD = 0x272F; // clang, cmdHelp.c
pub const MSG_STOP_SYNTAX: DWORD = 0x272E; // clang, cmdHelp.c
pub const MSG_SYNTAX_HELP: DWORD = 0x2739; // clang, cmdHelp.c
pub const MSG_TIME_HELP: DWORD = 0x2731; // clang, cmdHelp.c
pub const MSG_TIME_SYNTAX: DWORD = 0x2730; // clang, cmdHelp.c
pub const MSG_USER_HELP: DWORD = 0x2735; // clang, cmdHelp.c
pub const MSG_USER_SYNTAX: DWORD = 0x2734; // clang, cmdHelp.c
pub const MSG_USE_HELP: DWORD = 0x2733; // clang, cmdHelp.c
pub const MSG_USE_SYNTAX: DWORD = 0x2732; // clang, cmdHelp.c
pub const MSG_VIEW_HELP: DWORD = 0x2737; // clang, cmdHelp.c
pub const MSG_VIEW_SYNTAX: DWORD = 0x2736; // clang, cmdHelp.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn LoadLibraryW(_: LPCWSTR) -> HINSTANCE; // main.c
    pub fn WNetCancelConnection2W(_: LPCWSTR, _: DWORD, _: BOOL) -> DWORD; // cmdUse.c
    pub fn WNetEnumResourceW(
        hEnum: HANDLE,
        lpcCount: LPDWORD,
        lpBuffer: LPVOID,
        lpBufferSize: LPDWORD,
    ) -> DWORD; // cmdUse.c
    pub fn WNetOpenEnumW(_: DWORD, _: DWORD, _: DWORD, _: LPNETRESOURCEW, _: LPHANDLE) -> DWORD; // cmdUse.c
    pub fn WNetUseConnectionW(
        hwndOwner: HWND,
        lpNetResource: LPNETRESOURCEW,
        lpPassword: LPCWSTR,
        lpUserId: LPCWSTR,
        dwFlags: DWORD,
        lpAccessName: LPWSTR,
        lpBufferSize: LPDWORD,
        lpResult: LPDWORD,
    ) -> DWORD; // cmdUse.c
}
pub use crate::prelude::LoadLibraryW as LoadLibrary; // sdk alias
pub use crate::prelude::OpenSCManagerW as OpenSCManager; // sdk alias
pub use crate::prelude::OpenServiceW as OpenService; // sdk alias
pub use crate::prelude::WNetCancelConnection2W as WNetCancelConnection2; // sdk alias
pub use crate::prelude::WNetEnumResourceW as WNetEnumResource; // sdk alias
pub use crate::prelude::WNetOpenEnumW as WNetOpenEnum; // sdk alias
pub use crate::prelude::WNetUseConnectionW as WNetUseConnection; // sdk alias
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
pub type OSVERSIONINFOEX = OSVERSIONINFOEXW;
// ==== end UNICODE TCHAR family ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! FillMemory { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { memset((($p0) as _), (($p2) as _), (($p1) as _)) }; }
#[macro_export] macro_rules! va_end { ($p0:expr $(,)?) => { __builtin_va_end((($p0) as _)) }; }
#[macro_export] macro_rules! va_start { ($p0:expr, $p1:expr $(,)?) => { __builtin_va_start((($p0) as _), (($p1) as _)) }; }
pub type NET_API_STATUS = DWORD;
pub const SERVICES_ACTIVE_DATABASE: *const u16 = (&[0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0073, 0x0041, 0x0063, 0x0074, 0x0069, 0x0076, 0x0065, 0x0000u16]).as_ptr(); // L"ServicesActive"
pub const SERVICES_ACTIVE_DATABASEW: *const u16 = (&[0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0073, 0x0041, 0x0063, 0x0074, 0x0069, 0x0076, 0x0065, 0x0000u16]).as_ptr(); // L"ServicesActive"
pub const SERVICE_SERVER: *const u16 = (&[0x004c, 0x0061, 0x006e, 0x006d, 0x0061, 0x006e, 0x0053, 0x0065, 0x0072, 0x0076, 0x0065, 0x0072, 0x0000u16]).as_ptr(); // L"LanmanServer"
pub const SERVICE_WORKSTATION: *const u16 = (&[0x004c, 0x0061, 0x006e, 0x006d, 0x0061, 0x006e, 0x0057, 0x006f, 0x0072, 0x006b, 0x0073, 0x0074, 0x0061, 0x0074, 0x0069, 0x006f, 0x006e, 0x0000u16]).as_ptr(); // L"LanmanWorkstation"
pub use crate::{FillMemory, va_end, va_start};
// ==== end SDK header macros ====

// ==== port variadic: C-variadic entry points, trampolines in the module's rust_varargs_shim.c (generated -- `rustconv port integrate`) ====
extern "C" {
    pub fn PrintMessageStringV(dwMessage: DWORD, ...);
}
// ==== end port variadic ====
