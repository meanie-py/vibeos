//! netsh-rust prelude — the single import surface the translated modules use.
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
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1; // cleanmgr-rust/src/prelude.rs
pub const CMD_FLAG_HIDDEN: NS_CMD_FLAGS = NS_CMD_FLAGS_CMD_FLAG_HIDDEN; // island enum
pub const CMD_FLAG_LOCAL: NS_CMD_FLAGS = NS_CMD_FLAGS_CMD_FLAG_LOCAL; // island enum
pub const CMD_FLAG_ONLINE: NS_CMD_FLAGS = NS_CMD_FLAGS_CMD_FLAG_ONLINE; // island enum
pub const CMD_FLAG_PRIORITY: NS_CMD_FLAGS = NS_CMD_FLAGS_CMD_FLAG_PRIORITY; // island enum
pub const EOAC_NONE: DWORD = 0; // ole32-rust/src/marshal/types.rs
pub const EXIT_FAILURE: u32 = 1; // arp-rust/src/prelude.rs
pub const EXIT_SUCCESS: u32 = 0; // arp-rust/src/prelude.rs
pub const LANG_USER_DEFAULT: u32 = 1024; // arp-rust/src/prelude.rs
pub const NETSH_COMMIT: NS_MODE_CHANGE = NS_MODE_CHANGE_NETSH_COMMIT; // island enum
pub const NETSH_FLUSH: NS_MODE_CHANGE = NS_MODE_CHANGE_NETSH_FLUSH; // island enum
pub const NETSH_SAVE: NS_MODE_CHANGE = NS_MODE_CHANGE_NETSH_SAVE; // island enum
pub const NETSH_UNCOMMIT: NS_MODE_CHANGE = NS_MODE_CHANGE_NETSH_UNCOMMIT; // island enum
pub const NS_REQ_PRESENT: _NS_REQS = _NS_REQS_NS_REQ_PRESENT; // island enum
pub const RPC_C_AUTHN_LEVEL_CALL: u32 = 3; // eventlog-rust/src/types_amd64.rs
pub const RPC_C_AUTHN_LEVEL_DEFAULT: u32 = 0; // eventlog-rust/src/types_amd64.rs
pub const RPC_C_AUTHN_WINNT: u32 = 10; // eventlog-rust/src/types_amd64.rs
pub const RPC_C_AUTHZ_NONE: u32 = 0; // eventlog-rust/src/types_amd64.rs
pub const RPC_C_IMP_LEVEL_IMPERSONATE: u32 = 3; // eventlog-rust/src/types_amd64.rs
extern "C" {
    pub fn __iob_func() -> *mut FILE; // crt-startup-rust/src/misc.rs
    pub fn _swprintf(buf: *mut WCHAR, fmt: *const WCHAR, ...) -> c_int; // at-rust/src/prelude.rs
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // apphelp-rust/src/prelude.rs
    pub fn _wcsnicmp(s1: *const u16, s2: *const u16, n: usize) -> i32; // apphelp-rust/src/prelude.rs
    pub fn _wfopen(file: *const u16, mode: *const u16) -> *mut c_void; // comp-rust/src/prelude.rs
    pub fn fclose(stream: *mut FILE) -> c_int; // cmd-rust/src/prelude.rs
    pub fn fgetws(s: *mut wchar_t, size: i32, file: *mut FILE) -> *mut wchar_t; // doskey-rust/src/prelude.rs
    pub fn qsort(base: *mut c_void, num: usize, width: usize, comp: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>); // cmd-rust/src/prelude.rs
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int; // apphelp-rust/src/prelude.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcsrchr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/prelude.rs
}
extern "system" {
    pub fn CoCreateInstance( rclsid: *const CLSID, pUnkOuter: LPUNKNOWN, dwClsContext: DWORD, iid: *const IID, ppv: *mut LPVOID, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoInitialize(lpReserved: LPVOID) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoInitializeSecurity( pSecDesc: PSECURITY_DESCRIPTOR, cAuthSvc: LONG, asAuthSvc: *mut SOLE_AUTHENTICATION_SERVICE, pReserved1: *mut ::core::ffi::c_void, dwAuthnLevel: DWORD, dwImpLevel: DWORD, pReserved2: *mut ::core::ffi::c_void, dwCapabilities: DWORD, pReserved3: *mut ::core::ffi::c_void, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoSetProxyBlanket( pProxy: *mut IUnknown, dwAuthnSvc: DWORD, dwAuthzSvc: DWORD, pServerPrincName: *mut OLECHAR, dwAuthnLevel: DWORD, dwImpLevel: DWORD, pAuthInfo: *mut ::core::ffi::c_void, dwCapabilities: DWORD, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoUninitialize(); // aclui-rust/src/types_i386.rs
    pub fn FormatMessageW( dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD, lpBuffer: LPWSTR, nSize: DWORD, Arguments: *mut va_list, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LoadLibraryW(arg1: LPCWSTR) -> HINSTANCE; // aclui-rust/src/types_i386.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn NhGetGuidFromInterfaceName(pInterfaceName: PWSTR, pInterfaceGUID: *mut GUID, _dwUnknown3: u32, _dwUnknown4: u32,) -> u32; // iphlpapi-rust/src/iphlpapi/iphlpapi_main.rs
    pub fn NhGetInterfaceNameFromDeviceGuid(pInterfaceGUID: *const GUID, pInterfaceName: PWSTR, pOutBufLen: *mut u32, dwUnknown4: u32, dwUnknown5: u32,) -> u32; // iphlpapi-rust/src/iphlpapi/iphlpapi_main.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegCreateKeyExW( hKey: HKEY, lpSubKey: LPCWSTR, Reserved: DWORD, lpClass: LPWSTR, dwOptions: DWORD, samDesired: REGSAM, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, phkResult: PHKEY, lpdwDisposition: PDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegEnumValueW( hKey: HKEY, dwIndex: DWORD, lpValueName: LPWSTR, lpcchValueName: LPDWORD, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegQueryInfoKeyW( hKey: HKEY, lpClass: LPWSTR, lpcchClass: LPDWORD, lpReserved: LPDWORD, lpcSubKeys: LPDWORD, lpcbMaxSubKeyLen: LPDWORD, lpcbMaxClassLen: LPDWORD, lpcValues: LPDWORD, lpcbMaxValueNameLen: LPDWORD, lpcbMaxValueLen: LPDWORD, lpcbSecurityDescriptor: LPDWORD, lpftLastWriteTime: PFILETIME, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegSetValueExW( hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn SysAllocString(arg1: *const OLECHAR) -> BSTR; // msctf-rust/src/types_i386.rs
    pub fn SysFreeString(arg1: BSTR); // msctf-rust/src/types_i386.rs
    pub fn VariantClear(arg1: *mut VARIANT) -> HRESULT; // msctf-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// CONTEXT_STACK_ENTRY: c2rust, context.c
pub type CONTEXT_STACK_ENTRY = _CONTEXT_STACK_ENTRY;
// _CONTEXT_STACK_ENTRY: c2rust, context.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _CONTEXT_STACK_ENTRY {
    pub pPrev: *mut _CONTEXT_STACK_ENTRY,
    pub pNext: *mut _CONTEXT_STACK_ENTRY,
    pub pContext: PCONTEXT_ENTRY,
}
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn GetModuleHandleW(_: LPCWSTR) -> HMODULE; // netsh.c
    pub fn RegDeleteValueW(_: HKEY, _: LPCWSTR) -> LSTATUS; // helper.c
}
pub use crate::prelude::GetModuleHandleW as GetModuleHandle; // sdk alias
pub use conutils_h::K32LoadStringW as LoadStringW; // sdk alias
pub use crate::prelude::RegDeleteValueW as RegDeleteValue; // sdk alias
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

// ================= netsh hand port (above the generated blocks) ==============================
// C: #define NETSH_ROOT_GUID {0, 0, 0, {0, 0, 0, 0, 0, 0, 0, 0}} (sdk/include/psdk/netsh.h)
pub const NETSH_ROOT_GUID: GUID = GUID { Data1: 0, Data2: 0, Data3: 0, Data4: [0; 8] };
// conutils (sdk/lib/conutils/outstream.h) V-variants; the C side is linked, not in this crate.
extern "C" {
    // C: DWORD CDECL PrintError(HANDLE hModule, DWORD dwErrId, ...) -- variadic, defined by the shim.
    pub fn PrintError(hModule: HANDLE, dwErrId: DWORD, ...) -> DWORD;
    pub fn ConMsgPrintfV(Stream: *mut CON_STREAM, dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD, Arguments: *mut va_list) -> INT;
    pub fn ConResMsgPrintfExV(Stream: *mut CON_STREAM, hInstance: HINSTANCE, dwFlags: DWORD, uID: UINT, LanguageId: LANGID, Arguments: *mut va_list) -> INT;
}
// ================= end netsh hand port ========================================================

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! V_BSTR { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.bstrVal }; }
#[macro_export] macro_rules! V_UINT { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.uintVal }; }
#[macro_export] macro_rules! min { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 < $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
#[macro_export] macro_rules! va_end { ($p0:expr $(,)?) => { __builtin_va_end((($p0) as _)) }; }
#[macro_export] macro_rules! va_start { ($p0:expr, $p1:expr $(,)?) => { __builtin_va_start((($p0) as _), (($p1) as _)) }; }
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT);
pub const HKEY_LOCAL_MACHINE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY);
#[inline]
pub unsafe fn IEnumWbemClassObject_Next(This: *mut IEnumWbemClassObject, lTimeout: LONG, uCount: ULONG, apObjects: *mut *mut IWbemClassObject, puReturned: *mut ULONG) -> HRESULT { ((*(*This).lpVtbl).Next.unwrap_unchecked())(This, lTimeout, uCount, apObjects, puReturned) } // widl C wrapper
#[inline]
pub unsafe fn IEnumWbemClassObject_Release(This: *mut IEnumWbemClassObject) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IWbemClassObject_BeginEnumeration(This: *mut IWbemClassObject, lEnumFlags: LONG) -> HRESULT { ((*(*This).lpVtbl).BeginEnumeration.unwrap_unchecked())(This, lEnumFlags) } // widl C wrapper
#[inline]
pub unsafe fn IWbemClassObject_EndEnumeration(This: *mut IWbemClassObject) -> HRESULT { ((*(*This).lpVtbl).EndEnumeration.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IWbemClassObject_Next(This: *mut IWbemClassObject, lFlags: LONG, strName: *mut BSTR, pVal: *mut VARIANT, pType: *mut CIMTYPE, plFlavor: *mut LONG) -> HRESULT { ((*(*This).lpVtbl).Next.unwrap_unchecked())(This, lFlags, strName, pVal, pType, plFlavor) } // widl C wrapper
#[inline]
pub unsafe fn IWbemClassObject_Release(This: *mut IWbemClassObject) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IWbemLocator_ConnectServer(This: *mut IWbemLocator, strNetworkResource: BSTR, strUser: BSTR, strPassword: BSTR, strLocale: BSTR, lSecurityFlags: LONG, strAuthority: BSTR, pCtx: *mut IWbemContext, ppNamespace: *mut *mut IWbemServices) -> HRESULT { ((*(*This).lpVtbl).ConnectServer.unwrap_unchecked())(This, strNetworkResource, strUser, strPassword, strLocale, lSecurityFlags, strAuthority, pCtx, ppNamespace) } // widl C wrapper
#[inline]
pub unsafe fn IWbemLocator_Release(This: *mut IWbemLocator) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IWbemServices_ExecQuery(This: *mut IWbemServices, strQueryLanguage: BSTR, strQuery: BSTR, lFlags: LONG, pCtx: *mut IWbemContext, ppEnum: *mut *mut IEnumWbemClassObject) -> HRESULT { ((*(*This).lpVtbl).ExecQuery.unwrap_unchecked())(This, strQueryLanguage, strQuery, lFlags, pCtx, ppEnum) } // widl C wrapper
#[inline]
pub unsafe fn IWbemServices_Release(This: *mut IWbemServices) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
pub const S_OK: HRESULT = ((0) as HRESULT);
pub use crate::{V_BSTR, V_UINT, min, va_end, va_start};
// ==== end SDK header macros ====

// ==== port variadic: C-variadic entry points, trampolines in the module's rust_varargs_shim.c (generated -- `rustconv port integrate`) ====
extern "C" {
    pub fn MakeString(hModule: HANDLE, dwMsgId: DWORD, ...) -> LPWSTR;
}
// ==== end port variadic ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::netsh::context::g_bOnline;
pub use crate::netsh::netsh::g_hModule;
pub use crate::netsh::context::g_pszMachine;
pub use crate::netsh::context::pCurrentContext;
pub use crate::netsh::helper::pHelperListHead;
pub use crate::netsh::context::pRootContext;
// ==== end cross-TU definitions ====
