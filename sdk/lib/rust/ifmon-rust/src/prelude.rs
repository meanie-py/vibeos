//! ifmon-rust prelude — the single import surface the translated modules use.
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
pub const NS_REQ_ZERO: _NS_REQS = _NS_REQS_NS_REQ_ZERO; // island enum
pub const S_OK: HRESULT = 0; // aclui-rust/src/prelude.rs
extern "C" {
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcsstr(haystack: LPCWSTR, needle: LPCWSTR) -> LPWSTR; // appwiz-rust/src/prelude.rs
    pub fn wcstoul(s: *const u16, endptr: *mut *mut u16, base: i32) -> u32; // at-rust/src/prelude.rs
}
extern "system" {
    pub fn CoCreateInstance( rclsid: *const GUID, pUnkOuter: *mut c_void, dwClsContext: u32, riid: *const GUID, ppv: *mut LPVOID, ) -> HRESULT; // schtasks-rust/src/prelude.rs
    pub fn CoInitialize(lpReserved: LPVOID) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemFree(ptr: LPVOID); // aclui-rust/src/types_i386.rs
    pub fn CoUninitialize(); // aclui-rust/src/types_i386.rs
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FlushIpNetTable(arg1: DWORD) -> DWORD; // tracert-rust/src/types_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn NhGetGuidFromInterfaceName(pInterfaceName: PWSTR, pInterfaceGUID: *mut GUID, _dwUnknown3: u32, _dwUnknown4: u32,) -> u32; // iphlpapi-rust/src/iphlpapi/iphlpapi_main.rs
    pub fn NhGetInterfaceNameFromGuid(pInterfaceGUID: *const GUID, pInterfaceName: PWSTR, pOutBufLen: *mut u32, dwUnknown4: u32, dwUnknown5: u32,) -> u32; // iphlpapi-rust/src/iphlpapi/iphlpapi_main.rs
    pub fn NhpAllocateAndGetInterfaceInfoFromStack(ppTable: *mut *mut IP_INTERFACE_NAME_INFO, pdwCount: PDWORD, bOrder: i32, hHeap: HANDLE, dwFlags: DWORD,) -> DWORD; // iphlpapi-rust/src/iphlpapi/iphlpapi_main.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::ifmon::interface::CMD_ENTRY;
pub use crate::ifmon::interface::CMD_GROUP_ENTRY;
pub use crate::ifmon::guid::GUID_IFMON_INTERFACE;
pub use crate::ifmon::guid::GUID_IFMON_IP;
pub use crate::ifmon::guid::GUID_IFMON_WINSOCK;
pub use crate::ifmon::interface::RegisterInterfaceHelper;
pub use crate::ifmon::ip::RegisterIpHelper;
pub use crate::ifmon::winsock::RegisterWinsockHelper;
pub use crate::ifmon::ifmon::hDllInstance;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn PrintMessage(pwszMessage: LPCWSTR, ...) -> DWORD; // interface.c
    pub fn PrintMessageFromModule(hModule: HANDLE, dwMsgId: DWORD, ...) -> DWORD; // interface.c
}
extern "system" {
    pub fn MatchEnumTag(
        hModule: HANDLE,
        pwcArg: LPCWSTR,
        dwNumArg: DWORD,
        pEnumTable: *const TOKEN_VALUE,
        pdwValue: PDWORD,
    ) -> DWORD; // ip.c
    pub fn MatchTagsInCmdLine(
        hModule: HANDLE,
        ppwcArguments: *mut LPWSTR,
        dwCurrentIndex: DWORD,
        dwArgCount: DWORD,
        pttTags: *mut TAG_TYPE,
        dwTagCount: DWORD,
        pdwTagType: *mut DWORD,
    ) -> DWORD; // ip.c
    pub fn MatchToken(pwszUserToken: LPCWSTR, pwszCmdToken: LPCWSTR) -> BOOL; // ip.c
    pub fn RegisterContext(pChildContext: *const NS_CONTEXT_ATTRIBUTES) -> DWORD; // interface.c
    pub fn RegisterHelper(
        pguidParentContext: *const GUID,
        pfnRegisterSubContext: *const NS_HELPER_ATTRIBUTES,
    ) -> DWORD; // interface.c
}
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

// ==== port resolve: INITGUID storage (generated) ====
#[no_mangle]
pub static GUID_DEVCLASS_NETTRANS: GUID = GUID { Data1: 0x4D36E975, Data2: 0xE325, Data3: 0x11CE, Data4: [0xBF, 0xC1, 0x08, 0x00, 0x2B, 0xE1, 0x03, 0x18] };
// ==== end INITGUID storage ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[inline]
pub unsafe fn INetCfgClass_FindComponent(This: *mut INetCfgClass, pszwComponentId: LPCWSTR, ppnccItem: *mut *mut INetCfgComponent) -> HRESULT { ((*(*This).lpVtbl).FindComponent.unwrap_unchecked())(This, pszwComponentId, ppnccItem) } // widl C wrapper
#[inline]
pub unsafe fn INetCfgClass_Release(This: *mut INetCfgClass) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn INetCfgComponentPrivate_Release(This: *mut INetCfgComponentPrivate) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn INetCfgComponentPrivate_Unknown1(This: *mut INetCfgComponentPrivate, riid: *const IID, ppv: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).Unknown1.unwrap_unchecked())(This, riid, ppv) } // widl C wrapper
#[inline]
pub unsafe fn INetCfgComponent_QueryInterface(This: *mut INetCfgComponent, riid: *const IID, ppv: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppv) } // widl C wrapper
#[inline]
pub unsafe fn INetCfgComponent_Release(This: *mut INetCfgComponent) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn INetCfgLock_AcquireWriteLock(This: *mut INetCfgLock, cmsTimeout: DWORD, pszwClientDescription: LPCWSTR, ppszwClientDescription: *mut LPWSTR) -> HRESULT { ((*(*This).lpVtbl).AcquireWriteLock.unwrap_unchecked())(This, cmsTimeout, pszwClientDescription, ppszwClientDescription) } // widl C wrapper
#[inline]
pub unsafe fn INetCfgLock_Release(This: *mut INetCfgLock) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn INetCfgLock_ReleaseWriteLock(This: *mut INetCfgLock) -> HRESULT { ((*(*This).lpVtbl).ReleaseWriteLock.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn INetCfg_Apply(This: *mut INetCfg) -> HRESULT { ((*(*This).lpVtbl).Apply.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn INetCfg_Initialize(This: *mut INetCfg, pvReserved: PVOID) -> HRESULT { ((*(*This).lpVtbl).Initialize.unwrap_unchecked())(This, pvReserved) } // widl C wrapper
#[inline]
pub unsafe fn INetCfg_QueryInterface(This: *mut INetCfg, riid: *const IID, ppv: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppv) } // widl C wrapper
#[inline]
pub unsafe fn INetCfg_QueryNetCfgClass(This: *mut INetCfg, pguidClass: *const GUID, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryNetCfgClass.unwrap_unchecked())(This, pguidClass, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn INetCfg_Release(This: *mut INetCfg) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn INetCfg_Uninitialize(This: *mut INetCfg) -> HRESULT { ((*(*This).lpVtbl).Uninitialize.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn ITcpipProperties_Release(This: *mut ITcpipProperties) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn ITcpipProperties_Unknown1(This: *mut ITcpipProperties, pAdapterName: *mut GUID, ppProperties: *mut PTCPIP_PROPERTIES) -> HRESULT { ((*(*This).lpVtbl).Unknown1.unwrap_unchecked())(This, pAdapterName, ppProperties) } // widl C wrapper
#[inline]
pub unsafe fn ITcpipProperties_Unknown2(This: *mut ITcpipProperties, pAdapterName: *mut GUID, pProperties: PTCPIP_PROPERTIES) -> HRESULT { ((*(*This).lpVtbl).Unknown2.unwrap_unchecked())(This, pAdapterName, pProperties) } // widl C wrapper
// ==== end SDK header macros ====
