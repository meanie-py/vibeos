//! netshell-rust prelude — the single import surface the translated modules use.
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
// ATL's CStringW/CComPtr/containers, ported once in atlclasses-rust.
pub use atl_rust::{atl_format, atl_format_id, CAtlArray, CAtlList, CComPtr, CHeapPtr, CSimpleArray, CLocalAllocator, CSimpleMap, CStringA, CStringW, POSITION, _AtlBaseModule};
pub use atl_rust::CStringW as CSimpleString;
pub use atl_rust::CStringW as CString;
pub use atl_rust::shellutils::{SHELL_ErrorBox, ShellFolderImpl_CompareItemIDs};
pub use atl_rust::{SHSetStrRet, ShellObjectCreatorInit};

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

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port cppapp: the module's C++ classes (generated -- `rustconv port cppapp`) ====
pub use crate::netshell::cpp_classes::*;
pub use crate::netshell::cpp_classes::{CEnumIDList, CEnumIDList__IEnumIDList_VTBL, CEnumIDList__delete, CEnumIDList__make_0, CEnumIDList__new_0, CEnumIDList__new_in_0, CEnumIDList__zeroed, CLanStatus, CLanStatus__IOleCommandTarget_VTBL, CLanStatus__delete, CLanStatus__make_0, CLanStatus__new_0, CLanStatus__new_in_0, CLanStatus__zeroed, CNetConUiObject, CNetConUiObject__IContextMenu3_VTBL, CNetConUiObject__IExtractIconW_VTBL, CNetConUiObject__IObjectWithSite_VTBL, CNetConUiObject__IQueryInfo_VTBL, CNetConUiObject__delete, CNetConUiObject__make_0, CNetConUiObject__new_0, CNetConUiObject__new_in_0, CNetConUiObject__zeroed, CNetConnection, CNetConnectionManager, CNetConnectionManager__IEnumNetConnection_VTBL, CNetConnectionManager__INetConnectionManager_VTBL, CNetConnectionManager__delete, CNetConnectionManager__make_0, CNetConnectionManager__new_0, CNetConnectionManager__new_in_0, CNetConnectionManager__zeroed, CNetConnectionPropertyUi, CNetConnectionPropertyUi__INetConnectionConnectUi_VTBL, CNetConnectionPropertyUi__INetConnectionPropertyUi2_VTBL, CNetConnectionPropertyUi__INetLanConnectionUiInfo_VTBL, CNetConnectionPropertyUi__delete, CNetConnectionPropertyUi__make_0, CNetConnectionPropertyUi__new_0, CNetConnectionPropertyUi__new_in_0, CNetConnectionPropertyUi__zeroed, CNetConnection__INetConnection_VTBL, CNetConnection__delete, CNetConnection__make_0, CNetConnection__new_0, CNetConnection__new_in_0, CNetConnection__zeroed, CNetworkConnections, CNetworkConnections__IOleCommandTarget_VTBL, CNetworkConnections__IPersistFolder2_VTBL, CNetworkConnections__IShellExtInit_VTBL, CNetworkConnections__IShellFolder2_VTBL, CNetworkConnections__IShellFolderViewCB_VTBL, CNetworkConnections__delete, CNetworkConnections__make_0, CNetworkConnections__new_0, CNetworkConnections__new_in_0, CNetworkConnections__zeroed, NetConnectSFHeader, ObjectMap, _NETWORKCOMPONENT, _NETWORKCOMPONENT__delete, _NETWORKCOMPONENT__make_0, _NETWORKCOMPONENT__new_0, _NETWORKCOMPONENT__new_in_0, _NETWORKCOMPONENT__zeroed, _NETWORKSETUPDATA, _NETWORKSETUPDATA__delete, _NETWORKSETUPDATA__make_0, _NETWORKSETUPDATA__new_0, _NETWORKSETUPDATA__new_in_0, _NETWORKSETUPDATA__zeroed, cpp_alloc, cpp_delete, cpp_delete_array, cpp_free, cpp_new, cpp_new_array, cpp_new_in, cpp_pure_virtual_call, gModule, tagENUMLIST, tagENUMLIST__delete, tagENUMLIST__make_0, tagENUMLIST__new_0, tagENUMLIST__new_in_0, tagENUMLIST__zeroed, tagINetConnectionItem__delete, tagINetConnectionItem__make_0, tagINetConnectionItem__new_0, tagINetConnectionItem__new_in_0, tagINetConnectionItem__zeroed, tagNETCONIDSTRUCT__delete, tagNETCONIDSTRUCT__make_0, tagNETCONIDSTRUCT__new_0, tagNETCONIDSTRUCT__new_in_0, tagNETCONIDSTRUCT__zeroed, tagNotificationItem__delete, tagNotificationItem__make_0, tagNotificationItem__new_0, tagNotificationItem__new_in_0, tagNotificationItem__zeroed};
// ==== end port cppapp ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const CLSCTX_ALL: DWORD = 0x17; // appwiz-rust/src/prelude.rs
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1; // acppage-rust/src/prelude.rs
pub const MIB_IF_OPER_STATUS_CONNECTED: i32 = 4; // inetmib1-rust/src/prelude.rs
pub const MIB_IF_OPER_STATUS_OPERATIONAL: i32 = 5; // inetmib1-rust/src/prelude.rs
pub const NCCF_ALLOW_REMOVAL: NETCON_CHARACTERISTIC_FLAGS = NETCON_CHARACTERISTIC_FLAGS_NCCF_ALLOW_REMOVAL; // island enum
pub const NCCF_ALLOW_RENAME: NETCON_CHARACTERISTIC_FLAGS = NETCON_CHARACTERISTIC_FLAGS_NCCF_ALLOW_RENAME; // island enum
pub const NCCF_SHOW_ICON: NETCON_CHARACTERISTIC_FLAGS = NETCON_CHARACTERISTIC_FLAGS_NCCF_SHOW_ICON; // island enum
pub const NCME_DEFAULT: NETCONMGR_ENUM_FLAGS = NETCONMGR_ENUM_FLAGS_NCME_DEFAULT; // island enum
pub const NCM_LAN: NETCON_MEDIATYPE = NETCON_MEDIATYPE_NCM_LAN; // island enum
pub const NCM_SHAREDACCESSHOST_RAS: NETCON_MEDIATYPE = NETCON_MEDIATYPE_NCM_SHAREDACCESSHOST_RAS; // island enum
pub const NCRP_QUERY_PROPERTY_UI: NCRP_FLAGS = NCRP_FLAGS_NCRP_QUERY_PROPERTY_UI; // island enum
pub const NCRP_SHOW_PROPERTY_UI: NCRP_FLAGS = NCRP_FLAGS_NCRP_SHOW_PROPERTY_UI; // island enum
pub const NCS_AUTHENTICATING: NETCON_STATUS = NETCON_STATUS_NCS_AUTHENTICATING; // island enum
pub const NCS_AUTHENTICATION_SUCCEEDED: NETCON_STATUS = NETCON_STATUS_NCS_AUTHENTICATION_SUCCEEDED; // island enum
pub const NCS_CONNECTED: NETCON_STATUS = NETCON_STATUS_NCS_CONNECTED; // island enum
pub const NCS_CONNECTING: NETCON_STATUS = NETCON_STATUS_NCS_CONNECTING; // island enum
pub const NCS_DISCONNECTED: NETCON_STATUS = NETCON_STATUS_NCS_DISCONNECTED; // island enum
pub const NCS_HARDWARE_DISABLED: NETCON_STATUS = NETCON_STATUS_NCS_HARDWARE_DISABLED; // island enum
pub const NCS_MEDIA_DISCONNECTED: NETCON_STATUS = NETCON_STATUS_NCS_MEDIA_DISCONNECTED; // island enum
pub const NCUC_NO_UI: NETCONUI_CONNECT_FLAGS = NETCONUI_CONNECT_FLAGS_NCUC_NO_UI; // island enum
pub const NET_TYPE_CLIENT: NET_TYPE = NET_TYPE_NET_TYPE_CLIENT; // island enum
pub const NET_TYPE_PROTOCOL: NET_TYPE = NET_TYPE_NET_TYPE_PROTOCOL; // island enum
pub const NET_TYPE_SERVICE: NET_TYPE = NET_TYPE_NET_TYPE_SERVICE; // island enum
pub const NM_DBLCLK: u32 = 0xFFFFFFFD; // certmgr-rust/src/prelude.rs
pub const OBO_USER: tagOBO_TOKEN_TYPE = tagOBO_TOKEN_TYPE_OBO_USER; // island enum
pub const OLECMDEXECOPT_DODEFAULT: OLECMDEXECOPT = OLECMDEXECOPT_OLECMDEXECOPT_DODEFAULT; // island enum
pub const OLECMDID_NEW: OLECMDID = OLECMDID_OLECMDID_NEW; // island enum
pub const UINT_MAX: u32 = 4294967295; // comctl32-rust/src/prelude.rs
extern "C" {
    pub fn _swprintf(buf: *mut WCHAR, fmt: *const WCHAR, ...) -> c_int; // at-rust/src/prelude.rs
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // acppage-rust/src/prelude.rs
    pub fn ros_dbg_log( cls: ::core::ffi::c_uint, ch: *mut __wine_debug_channel, file: *const ::core::ffi::c_char, func: *const ::core::ffi::c_char, line: ::core::ffi::c_int, format: *const ::core::ffi::c_char, ... ) -> ::core::ffi::c_int; // winegdi-rust/src/types_i386.rs
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
}
extern "system" {
    pub fn AppendMenuW(arg1: HMENU, arg2: UINT, arg3: UINT_PTR, arg4: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn BringWindowToTop(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CLSIDFromString(arg1: LPCOLESTR, arg2: LPCLSID) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CoCreateInstance( rclsid: *const CLSID, pUnkOuter: LPUNKNOWN, dwClsContext: DWORD, iid: *const IID, ppv: *mut LPVOID, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoInitialize(lpReserved: LPVOID) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemAlloc(size: SIZE_T) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemFree(ptr: LPVOID); // aclui-rust/src/types_i386.rs
    pub fn CoUninitialize(); // aclui-rust/src/types_i386.rs
    pub fn CreateDialogParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn CreatePopupMenu() -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn CreateProcessW( arg1: LPCWSTR, arg2: LPWSTR, arg3: LPSECURITY_ATTRIBUTES, arg4: LPSECURITY_ATTRIBUTES, arg5: BOOL, arg6: DWORD, arg7: PVOID, arg8: LPCWSTR, arg9: LPSTARTUPINFOW, arg10: LPPROCESS_INFORMATION, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateThread( arg1: LPSECURITY_ATTRIBUTES, arg2: DWORD, arg3: LPTHREAD_START_ROUTINE, arg4: PVOID, arg5: DWORD, arg6: PDWORD, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn DestroyIcon(arg1: HICON) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyMenu(arg1: HMENU) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DialogBoxParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> INT_PTR; // aclui-rust/src/types_i386.rs
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FindWindowExW(arg1: HWND, arg2: HWND, arg3: LPCWSTR, arg4: LPCWSTR) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetCursorPos(arg1: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetDlgItemTextW( hDlg: HWND, nIDDlgItem: ::core::ffi::c_int, lpString: LPWSTR, cchMax: ::core::ffi::c_int, ) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetModuleFileNameW(hModule: HINSTANCE, lpFilename: LPWSTR, nSize: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetParent(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetTickCount() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn InsertMenuItemW(arg1: HMENU, arg2: UINT, arg3: BOOL, arg4: LPCMENUITEMINFOW) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsBadReadPtr(arg1: *const ::core::ffi::c_void, arg2: UINT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsDlgButtonChecked(arg1: HWND, arg2: ::core::ffi::c_int) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn KillTimer(arg1: HWND, arg2: UINT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // aclui-rust/src/types_i386.rs
    pub fn LoadStringW( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn MessageBoxW( hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn SendDlgItemMessageW( arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn SetDlgItemTextW(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetFocus(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn SetForegroundWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetTimer(arg1: HWND, arg2: UINT_PTR, arg3: UINT, arg4: TIMERPROC) -> UINT_PTR; // aclui-rust/src/types_i386.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn Sleep(arg1: DWORD); // aclui-rust/src/types_i386.rs
    pub fn StringFromCLSID(id: *const CLSID, arg1: *mut LPOLESTR) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn TrackPopupMenuEx( arg1: HMENU, arg2: UINT, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: HWND, arg6: LPTPMPARAMS, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn lstrcpynW(arg1: LPWSTR, arg2: LPCWSTR, arg3: ::core::ffi::c_int) -> LPWSTR; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::netshell::enumlist_cpp_h::CEnumIDList_CreateInstance;
pub use crate::netshell::enumlist_cpp_h::ILCreateNetConnectItem;
pub use crate::netshell::enumlist_cpp_h::ILGetConnData;
pub use crate::netshell::enumlist_cpp_h::ILGetConnName;
pub use crate::netshell::enumlist_cpp_h::ILGetConnection;
pub use crate::netshell::enumlist_cpp_h::ILGetDeviceName;
pub use crate::netshell::netshell_cpp_h::InitializePropertySheetPage;
pub use crate::netshell::enumlist_cpp_h::LPENUMLIST;
pub use crate::netshell::netshell_cpp_h::NcFreeNetconProperties;
pub use crate::netshell::lanstatusui_cpp_h::RefreshNetTaskbarNotificationIcon;
pub use crate::netshell::shfldr_netconnect_cpp_h::SHCOLSTATE_ONBYDEFAULT;
pub use crate::netshell::shfldr_netconnect_cpp_h::SHCOLSTATE_TYPE_DATE;
pub use crate::netshell::shfldr_netconnect_cpp_h::SHCOLSTATE_TYPE_STR;
pub use crate::netshell::shfldr_netconnect_cpp_h::ShowNetConnectionProperties;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbLengthW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const COLUMN_DEVNAME: i32 = 3; // clang, shfldr_netconnect.cpp
pub const COLUMN_NAME: i32 = 0; // clang, shfldr_netconnect.cpp
pub const COLUMN_OWNER: i32 = 5; // clang, shfldr_netconnect.cpp
pub const COLUMN_PHONE: i32 = 4; // clang, shfldr_netconnect.cpp
pub const COLUMN_STATUS: i32 = 2; // clang, shfldr_netconnect.cpp
pub const COLUMN_TYPE: i32 = 1; // clang, shfldr_netconnect.cpp
pub const MAX_PROPERTY_SHEET_PAGE: i32 = 0xA; // clang, shfldr_netconnect.cpp
pub const MIB_IF_OPER_STATUS_CONNECTING: _INTERNAL_IF_OPER_STATUS = 3; // clang, connectmanager.cpp
pub const MIB_IF_OPER_STATUS_DISCONNECTED: _INTERNAL_IF_OPER_STATUS = 2; // clang, connectmanager.cpp
pub const MIB_IF_OPER_STATUS_NON_OPERATIONAL: _INTERNAL_IF_OPER_STATUS = 0; // clang, connectmanager.cpp
pub const MIB_IF_OPER_STATUS_UNREACHABLE: _INTERNAL_IF_OPER_STATUS = 1; // clang, connectmanager.cpp
pub const NETCONNECTSHELLVIEWCOLUMNS: i32 = 6; // clang, shfldr_netconnect.cpp
pub const NETTIMERID: i32 = 0xFABC; // clang, lanstatusui.cpp
pub const NETTIMERINTERVAL: i32 = 0x3E8; // clang, lanstatusui.cpp
pub const NETWORK_SETUP_MAGIC: i32 = 0x50515253; // clang, setup.cpp
pub const WPARAM_MAGIC: i32 = 0x4954534C; // clang, lanstatusui.cpp
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn CreatePropertySheetPageW(arg1: LPCPROPSHEETPAGEW) -> HPROPSHEETPAGE; // aclui-rust/src/types_i386.rs
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR; // charmap-rust/src/prelude.rs
    pub fn LoadImageW( hInst: HINSTANCE, name: LPCWSTR, type_: UINT, cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, fuLoad: UINT, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR; // charmap-rust/src/prelude.rs
    pub fn SetupDiGetClassDevsW( arg1: *const GUID, arg2: PCWSTR, arg3: HWND, arg4: DWORD, ) -> HDEVINFO; // desk-rust/src/types_i386.rs
    pub fn SetupDiSetClassInstallParamsW( DeviceInfoSet: HDEVINFO, DeviceInfoData: PSP_DEVINFO_DATA, ClassInstallParams: PSP_CLASSINSTALL_HEADER, ClassInstallParamsSize: DWORD, ) -> BOOL; // desk-rust/src/types_i386.rs
}
pub use crate::prelude::CreatePropertySheetPageW as CreatePropertySheetPage; // sdk alias
pub use win32macros_rust::WINE_FIXME as FIXME; // sdk alias
pub use crate::prelude::GetWindowLongPtrW as GetWindowLongPtr; // sdk alias
pub use crate::prelude::LoadIconW as LoadIcon; // sdk alias
pub use crate::prelude::LoadImageW as LoadImage; // sdk alias
pub use crate::prelude::PostMessageW as PostMessage; // sdk alias
pub use crate::prelude::SendDlgItemMessageW as SendDlgItemMessage; // sdk alias
pub use crate::prelude::SendMessageW as SendMessage; // sdk alias
pub use crate::prelude::SetWindowLongPtrW as SetWindowLongPtr; // sdk alias
pub use crate::prelude::SetupDiGetClassDevsW as SetupDiGetClassDevs; // sdk alias
pub use crate::prelude::SetupDiSetClassInstallParamsW as SetupDiSetClassInstallParams; // sdk alias
pub use strsafe_rust::StringCbCatW as StringCbCat; // sdk alias
pub use strsafe_rust::StringCbCopyW as StringCbCopy; // sdk alias
pub use win32macros_rust::WINE_TRACE as TRACE; // sdk alias
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub static mut __wine_dbch_shell: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x73\x68\x65\x6c\x6c\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_shell };
// ==== end wine debug channels ====

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
#[macro_export] macro_rules! FIRST_IPADDRESS { ($p0:expr $(,)?) => { (($p0 >> 24) & 0xff) }; } // gen:4674083a9150
#[macro_export] macro_rules! FOURTH_IPADDRESS { ($p0:expr $(,)?) => { ($p0 & 0xff) }; } // gen:d0abe33a54ea
#[macro_export] macro_rules! IsEqualIID { ($p0:expr, $p1:expr $(,)?) => { IsEqualGUID!(($p0), ($p1)) }; } // gen:57475db97dfd
#[macro_export] macro_rules! ListView_GetItemCount { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 4)) as _), ((0) as _), ((0) as _))) as c_int) }; } // gen:d120d9ae80f8
#[macro_export] macro_rules! ListView_SetCheckState { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { { let mut _ms_lvi: LVITEMW = core::mem::zeroed(); { _ms_lvi.stateMask = ((0xF000) as _); }; { _ms_lvi.state = ((((if ($p2) as isize != 0 { 2 } else { 1 }) << 12)) as _); }; SendMessageW((($p0) as _), (((0x1000 + 43)) as _), (($p1) as WPARAM), ((core::ptr::addr_of_mut!(_ms_lvi)) as LPARAM)); } }; } // gen:784a1d02a16d
#[macro_export] macro_rules! ListView_SetExtendedListViewStyleEx { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 54)) as _), (($p1) as _), (($p2) as _))) as DWORD) }; } // gen:7ccd5ba7d36c
#[macro_export] macro_rules! ListView_SetImageList { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 3)) as _), (($p2) as WPARAM), (($p1) as LPARAM))) as HIMAGELIST) }; } // gen:5e0bd3b75154
#[macro_export] macro_rules! ListView_SetItemState { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { { let mut _ms_lvi: LVITEMW = core::mem::zeroed(); { _ms_lvi.stateMask = (($p3) as _); }; { _ms_lvi.state = (($p2) as _); }; SendMessageW((($p0) as _), (((0x1000 + 43)) as _), (($p1) as WPARAM), ((core::ptr::addr_of_mut!(_ms_lvi)) as LPARAM)); } }; } // gen:1b44f6ac3270
#[macro_export] macro_rules! MAKEINTRESOURCE { ($p0:expr $(,)?) => { ((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as ULONG_PTR)) as LPWSTR) }; } // gen:971027e46ae0
#[macro_export] macro_rules! MAKELPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM) }; } // gen:b08956c0c0dd
#[macro_export] macro_rules! MAKE_COMPARE_HRESULT { ($p0:expr $(,)?) => { (if ((($p0 > 0) as c_int)) as isize != 0 { ((1) as HRESULT) } else { (if ((($p0 < 0) as c_int)) as isize != 0 { 0xffff } else { ((0) as HRESULT) }) }) }; } // gen:15072ba69188
#[macro_export] macro_rules! MAKE_HRESULT { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((((((($p0) as c_uint) << 31) | ((($p1) as c_uint) << 16)) | (($p2) as c_uint))) as HRESULT) }; } // gen:829efe363f45
#[macro_export] macro_rules! PropSheet_PressButton { ($p0:expr, $p1:expr $(,)?) => { SendMessageW((($p0) as _), ((1137) as _), (($p1) as _), ((0) as _)) }; } // gen:e59ece10eeeb
#[macro_export] macro_rules! PropSheet_SetWizButtons { ($p0:expr, $p1:expr $(,)?) => { PostMessageW((($p0) as _), (((1024 + 112)) as _), ((0) as _), (($p1) as LPARAM)) }; } // gen:346c22101632
#[macro_export] macro_rules! SECOND_IPADDRESS { ($p0:expr $(,)?) => { (($p0 >> 16) & 0xff) }; } // gen:22e05db4cd9b
#[macro_export] macro_rules! THIRD_IPADDRESS { ($p0:expr $(,)?) => { (($p0 >> 8) & 0xff) }; } // gen:fbc56406214f
pub const E_FAIL: HRESULT = ((0x80004005u32) as HRESULT); // gen:1275c1cb9e8d
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT); // gen:cd8a52171603
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT); // gen:80daa5b2ffa6
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT); // gen:cbf314acc21c
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT); // gen:98bee37aa995
pub const E_POINTER: HRESULT = ((0x80004003u32) as HRESULT); // gen:f72eeacc5d42
pub const E_UNEXPECTED: HRESULT = ((0x8000FFFFu32) as HRESULT); // gen:c1fe647c5e73
pub const HKEY_LOCAL_MACHINE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY); // gen:6463b26ca00d
pub const LOCALE_USER_DEFAULT: DWORD = ((((((((0x0) as USHORT)) as DWORD) << 16) | ((((((((((0x01) as USHORT)) as c_int) << 10) | ((((0x00) as USHORT)) as c_int))) as USHORT)) as DWORD))) as DWORD); // gen:2d64a44f0c5a
pub type LPPROPSHEETPAGE = LPPROPSHEETPAGEW; // gen:24f57e299212
pub type LV_COLUMN = LVCOLUMNW; // gen:c1142bf2812a
pub const NOERROR: HRESULT = ((0) as HRESULT); // gen:e8f751a60ae6
pub type PROPSHEETPAGE = PROPSHEETPAGEW; // gen:3ef1d2dd4058
pub type REFIID = *const IID; // gen:c79168ffaea6
pub const S_FALSE: HRESULT = ((1) as HRESULT); // gen:b1b8213e7dac
pub const S_OK: HRESULT = ((0) as HRESULT); // gen:9969a2c48b97
pub const TRAYICONWINDOWCAPTIONW: *const u16 = (&[0x0043, 0x006f, 0x006e, 0x006e, 0x0065, 0x0063, 0x0074, 0x0069, 0x006f, 0x006e, 0x0073, 0x0020, 0x0054, 0x0072, 0x0061, 0x0079, 0x0000u16]).as_ptr(); // L"Connections Tray" // gen:b29178f00ea3
pub use crate::{FIRST_IPADDRESS, FOURTH_IPADDRESS, IsEqualIID, ListView_GetItemCount, ListView_SetCheckState, ListView_SetExtendedListViewStyleEx, ListView_SetImageList, ListView_SetItemState, MAKEINTRESOURCE, MAKELPARAM, MAKE_COMPARE_HRESULT, MAKE_HRESULT, PropSheet_PressButton, PropSheet_SetWizButtons, SECOND_IPADDRESS, THIRD_IPADDRESS};
// ==== end SDK header macros ====
