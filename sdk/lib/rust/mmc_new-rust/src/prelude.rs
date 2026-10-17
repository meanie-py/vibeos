//! mmc_new-rust prelude — the single import surface the translated modules use.
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
pub use atl_rust::{atl_format, atl_format_id, CAtlList, CComPtr, CHeapPtr, CSimpleArray, CLocalAllocator, CSimpleMap, CStringA, CStringW, POSITION, _AtlBaseModule};
pub use atl_rust::CStringW as CSimpleString;
pub use atl_rust::CComHeapPtr;

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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _byteswap_uint64, _byteswap_ulong, _byteswap_ushort, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_variant, debugstr_vt, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, lrintf, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_longlong, wine_dbgstr_variant, wine_dbgstr_vt, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port cppapp: the module's C++ classes (generated -- `rustconv port cppapp`) ====
pub use crate::mmc_new::cpp_classes::*;
pub use crate::mmc_new::cpp_classes::{CAboutSnapinDialog, CAboutSnapinDialog__VTBL_own, CAboutSnapinDialog__delete, CAboutSnapinDialog__make_0, CAboutSnapinDialog__new_0, CAboutSnapinDialog__new_in_0, CAboutSnapinDialog__vtbl, CAboutSnapinDialog__zeroed, CActionWnd, CActionWnd__VTBL_own, CActionWnd__delete, CActionWnd__make_0, CActionWnd__new_0, CActionWnd__new_in_0, CActionWnd__vtbl, CActionWnd__zeroed, CAddAdvancedDialog, CAddAdvancedDialog__VTBL_own, CAddAdvancedDialog__delete, CAddAdvancedDialog__make_0, CAddAdvancedDialog__new_0, CAddAdvancedDialog__new_in_0, CAddAdvancedDialog__vtbl, CAddAdvancedDialog__zeroed, CAddDialog, CAddDialog__VTBL_own, CAddDialog__delete, CAddDialog__make_0, CAddDialog__new_0, CAddDialog__new_in_0, CAddDialog__vtbl, CAddDialog__zeroed, CConsoleWnd, CConsoleWnd__IConsole2_VTBL, CConsoleWnd__IConsoleNameSpace2_VTBL, CConsoleWnd__delete, CConsoleWnd__make_0, CConsoleWnd__new_0, CConsoleWnd__new_in_0, CConsoleWnd__zeroed, CCustomizeDialog, CCustomizeDialog__VTBL_own, CCustomizeDialog__delete, CCustomizeDialog__make_0, CCustomizeDialog__new_0, CCustomizeDialog__new_in_0, CCustomizeDialog__vtbl, CCustomizeDialog__zeroed, CListView, CListView__delete, CListView__make_0, CListView__new_0, CListView__new_in_0, CListView__zeroed, CMMCModule, CMMCModule__VTBL_own, CMMCModule__delete, CMMCModule__make_0, CMMCModule__new_0, CMMCModule__new_in_0, CMMCModule__vtbl, CMMCModule__zeroed, CMainWnd, CMainWnd__VTBL_own, CMainWnd__delete, CMainWnd__make_0, CMainWnd__new_0, CMainWnd__new_in_0, CMainWnd__vtbl, CMainWnd__zeroed, COptionsDialog, COptionsDialog__VTBL_own, COptionsDialog__delete, COptionsDialog__make_0, COptionsDialog__new_0, COptionsDialog__new_in_0, COptionsDialog__vtbl, COptionsDialog__zeroed, CRecentFileEntry, CRecentFileEntry__delete, CRecentFileEntry__make_0, CRecentFileEntry__new_0, CRecentFileEntry__new_in_0, CRecentFileEntry__zeroed, CRegKey, CRegKey__delete, CRegKey__make_0, CRegKey__make_1, CRegKey__make_2, CRegKey__new_0, CRegKey__new_1, CRegKey__new_2, CRegKey__new_in_0, CRegKey__new_in_1, CRegKey__new_in_2, CRegKey__zeroed, CSnapin, CSnapinAlias, CSnapinAlias__delete, CSnapinAlias__make_0, CSnapinAlias__new_0, CSnapinAlias__new_in_0, CSnapinAlias__zeroed, CSnapinCacheEntry, CSnapinCacheEntry__delete, CSnapinCacheEntry__make_0, CSnapinCacheEntry__new_0, CSnapinCacheEntry__new_in_0, CSnapinCacheEntry__zeroed, CSnapin__delete, CSnapin__make_0, CSnapin__new_0, CSnapin__new_in_0, CSnapin__zeroed, CStatusBar, CStatusBar__delete, CStatusBar__make_0, CStatusBar__new_0, CStatusBar__new_in_0, CStatusBar__zeroed, CToolbar, CToolbar__delete, CToolbar__make_0, CToolbar__new_0, CToolbar__new_in_0, CToolbar__zeroed, CTooltips, CTooltips__delete, CTooltips__make_0, CTooltips__new_0, CTooltips__new_in_0, CTooltips__zeroed, CTreeView, CTreeView__delete, CTreeView__make_0, CTreeView__new_0, CTreeView__new_in_0, CTreeView__zeroed, CWindow, CWindow__delete, CWindow__make_0, CWindow__new_0, CWindow__new_in_0, CWindow__rcDefault, CWindow__zeroed, MscFile, MscFile__delete, MscFile__make_0, MscFile__new_0, MscFile__new_in_0, MscFile__zeroed, _IDHWNDPAIR, _IDHWNDPAIR__delete, _IDHWNDPAIR__make_0, _IDHWNDPAIR__new_0, _IDHWNDPAIR__new_in_0, _IDHWNDPAIR__zeroed, _U_MENUorID, _U_RECT, cpp_alloc, cpp_delete, cpp_delete_array, cpp_free, cpp_new, cpp_new_array, cpp_new_in, cpp_pure_virtual_call, g_AtlModule};
// ==== end port cppapp ====

// ==== mmc_new hand additions (not generated): ATL window machinery ====
// CAtlString is ATL's CAtlStringW (= CStringW); CWndClassInfo/CWndProcThunk/_AtlWinModule/
// StartWindowProc are the thunkless CWindowImpl pieces in windowimpl.rs. rosctrls.h's control
// wrappers are translated from the header (mmc_new/rosctrls_h_h.rs).
pub use crate::templates::CAtlListExt;
pub use crate::windowimpl::{AtlWindowImpl, CAtlString, CAtlWinModule, CWndClassInfo, CWndProcThunk, GetWindowLongPtr, SetWindowLongPtr, StartWindowProc, _AtlWinModule};
// ==== end mmc_new hand additions ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const CLSCTX_INPROC: u32 = 3; // cleanmgr-rust/src/prelude.rs
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1; // cleanmgr-rust/src/prelude.rs
pub const CLSCTX_LOCAL_SERVER: DWORD = 0x4; // ole32-rust/src/compobj/com.rs
pub const COINIT_APARTMENTTHREADED: DWORD = 0x2; // imm32-rust/src/prelude.rs
pub const DocumentMode_Author: _DOCUMENT_MODE = _DOCUMENT_MODE_DocumentMode_Author; // island enum
pub const DocumentMode_User: _DOCUMENT_MODE = _DOCUMENT_MODE_DocumentMode_User; // island enum
pub const DocumentMode_UserMDI: _DOCUMENT_MODE = _DOCUMENT_MODE_DocumentMode_UserMDI; // island enum
pub const DocumentMode_UserSDI: _DOCUMENT_MODE = _DOCUMENT_MODE_DocumentMode_UserSDI; // island enum
pub const HWND_DESKTOP: HWND = 0x0usize as HWND; // eventvwr-rust/src/prelude.rs
pub const INT_MAX: u32 = 2147483647; // comctl32-rust/src/prelude.rs
pub const LVM_GETITEM: i32 = 0x104B; // rapps-rust/src/prelude.rs
pub const LVM_INSERTCOLUMN: i32 = 0x1061; // rapps-rust/src/prelude.rs
pub const LVM_INSERTITEM: i32 = 0x104D; // dxdiag-rust/src/prelude.rs
pub const LVM_SETITEMTEXT: i32 = 0x1074; // msconfig-rust/src/prelude.rs
pub const ListView_Detail: _LISTVIEW_MODE = _LISTVIEW_MODE_ListView_Detail; // island enum
pub const ListView_LargeIcons: _LISTVIEW_MODE = _LISTVIEW_MODE_ListView_LargeIcons; // island enum
pub const ListView_List: _LISTVIEW_MODE = _LISTVIEW_MODE_ListView_List; // island enum
pub const ListView_SmallIcons: _LISTVIEW_MODE = _LISTVIEW_MODE_ListView_SmallIcons; // island enum
pub const NM_DBLCLK: u32 = 0xFFFFFFFD; // certmgr-rust/src/prelude.rs
pub const NODE_ELEMENT: tagDOMNodeType = tagDOMNodeType_NODE_ELEMENT; // island enum
pub const REGCLS_MULTIPLEUSE: DWORD = 1; // ole32-rust/src/compobj/com.rs
pub const SB_SETTEXT: i32 = 0x40B; // charmap-rust/src/prelude.rs
pub const TVHT_ONITEM: i32 = 0x46; // comctl32-rust/src/prelude.rs
pub const TVM_INSERTITEM: i32 = 0x1132; // rapps-rust/src/prelude.rs
pub const TVN_SELCHANGED: u32 = 0xFFFFFE3D; // certmgr-rust/src/prelude.rs
pub const ULONG_MAX: u32 = 4294967295; // calc-rust/src/prelude.rs
pub const VT_BSTR: DWORD = 8; // cleanmgr-rust/src/prelude.rs
pub const VT_NULL: u16 = 1; // oleaut32-rust/src/consts.rs
extern "C" {
    pub fn _swprintf(buf: *mut WCHAR, fmt: *const WCHAR, ...) -> c_int; // at-rust/src/prelude.rs
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // apphelp-rust/src/prelude.rs
    pub fn _wcsnicmp(s1: *const u16, s2: *const u16, n: usize) -> i32; // apphelp-rust/src/prelude.rs
    pub fn _wtol(str: *const u16) -> i32; // cnt-rust/src/cabi/generated.rs
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int; // apphelp-rust/src/prelude.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcsrchr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcstol(s: *const u16, endptr: *mut *mut u16, base: i32) -> i32; // cleanmgr-rust/src/prelude.rs
    pub fn wcstoul(s: *const u16, endptr: *mut *mut u16, base: i32) -> u32; // at-rust/src/prelude.rs
    pub static IID_IUnknown: GUID; // cicero-rust/src/types_amd64.rs
    pub static IID_IXMLDOMDocument: IID; // mmc-rust/src/prelude.rs
}
extern "system" {
    pub fn BeginPaint(arg1: HWND, arg2: LPPAINTSTRUCT) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn CLSIDFromString(arg1: LPCOLESTR, arg2: LPCLSID) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CheckDlgButton(arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CheckMenuRadioItem(arg1: HMENU, arg2: UINT, arg3: UINT, arg4: UINT, arg5: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ClientToScreen(arg1: HWND, arg2: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CoCreateInstance( rclsid: *const CLSID, pUnkOuter: LPUNKNOWN, dwClsContext: DWORD, iid: *const IID, ppv: *mut LPVOID, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoInitializeEx(lpReserved: LPVOID, dwCoInit: DWORD) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemFree(ptr: LPVOID); // aclui-rust/src/types_i386.rs
    pub fn CoUninitialize(); // aclui-rust/src/types_i386.rs
    pub fn DefWindowProcW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn DestroyMenu(arg1: HMENU) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DispatchMessageW(arg1: *const MSG) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn DrawTextW( hdc: HDC, lpchText: LPCWSTR, cchText: ::core::ffi::c_int, lprc: LPRECT, format: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndPaint(arg1: HWND, arg2: *const PAINTSTRUCT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FormatMessageW( dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD, lpBuffer: LPWSTR, nSize: DWORD, Arguments: *mut va_list, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetCapture() -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetMenu(arg1: HWND) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn GetMenuItemInfoW(arg1: HMENU, arg2: UINT, arg3: BOOL, arg4: LPMENUITEMINFOW) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetMessageW(arg1: LPMSG, arg2: HWND, arg3: UINT, arg4: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetParent(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetSubMenu(arg1: HMENU, arg2: ::core::ffi::c_int) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn GetSysColor(arg1: ::core::ffi::c_int) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetSysColorBrush(arg1: ::core::ffi::c_int) -> HBRUSH; // aclui-rust/src/types_i386.rs
    pub fn GetSystemDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetWindowPlacement(arg1: HWND, arg2: *mut WINDOWPLACEMENT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetWindowRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetWindowTextW( hWnd: HWND, lpString: LPWSTR, nMaxCount: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn InsertMenuItemW(arg1: HMENU, arg2: UINT, arg3: BOOL, arg4: LPCMENUITEMINFOW) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsDlgButtonChecked(arg1: HWND, arg2: ::core::ffi::c_int) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn LoadAcceleratorsW(arg1: HINSTANCE, arg2: LPCWSTR) -> HACCEL; // aclui-rust/src/types_i386.rs
    pub fn LoadBitmapW(arg1: HINSTANCE, arg2: LPCWSTR) -> HBITMAP; // aclui-rust/src/types_i386.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn MapWindowPoints( hWndFrom: HWND, hWndTo: HWND, lpPoints: LPPOINT, cPoints: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn MonitorFromWindow(arg1: HWND, arg2: DWORD) -> HMONITOR; // aclui-rust/src/types_i386.rs
    pub fn MoveWindow( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: BOOL, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PostQuitMessage(arg1: ::core::ffi::c_int); // aclui-rust/src/types_i386.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn ReleaseCapture() -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RemoveMenu(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ScreenToClient(arg1: HWND, arg2: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetCapture(hWnd: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn SetMenu(arg1: HWND, arg2: HMENU) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetWindowPlacement(hWnd: HWND, arg1: *const WINDOWPLACEMENT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetWindowPos( arg1: HWND, arg2: HWND, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: UINT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn StringFromCLSID(id: *const CLSID, arg1: *mut LPOLESTR) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn SysAllocString(arg1: *const OLECHAR) -> BSTR; // msctf-rust/src/types_i386.rs
    pub fn SysFreeString(arg1: BSTR); // msctf-rust/src/types_i386.rs
    pub fn TrackPopupMenu( arg1: HMENU, arg2: UINT, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: HWND, arg7: LPCRECT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TranslateAcceleratorW(arg1: HWND, arg2: HACCEL, arg3: LPMSG) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn TranslateMDISysAccel(arg1: HWND, arg2: LPMSG) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TranslateMessage(arg1: *const MSG) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn UpdateWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn VariantClear(arg1: *mut VARIANT) -> HRESULT; // msctf-rust/src/types_i386.rs
    pub fn VariantInit(arg1: *mut VARIANT); // msctf-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const CBEM_INSERTITEM: i32 = 0x40B; // clang, CAddDialog.cpp
pub const SPLITTER_LEFT: u32 = 1; // clang, CConsoleWnd.cpp
pub const SPLITTER_NONE: u32 = 0; // clang, CConsoleWnd.cpp
pub const SPLITTER_RIGHT: u32 = 2; // clang, CConsoleWnd.cpp
pub const TBN_GETINFOTIP: u32 = 0xFFFFFD31; // clang, CMainWnd.cpp
pub const TVM_EDITLABEL: i32 = 0x1141; // clang, CAboutSnapinDialog.cpp
pub const TVM_GETITEM: i32 = 0x113E; // clang, CAboutSnapinDialog.cpp
pub const TVN_BEGINLABELEDIT: u32 = 0xFFFFFE35; // clang, CConsoleWnd.cpp
pub const TVN_ENDLABELEDIT: u32 = 0xFFFFFE34; // clang, CConsoleWnd.cpp
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
}
extern "system" {
    pub fn CreateWindowExW( dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, X: ::core::ffi::c_int, Y: ::core::ffi::c_int, nWidth: ::core::ffi::c_int, nHeight: ::core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID, ) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn DefFrameProcW(arg1: HWND, arg2: HWND, arg3: UINT, arg4: WPARAM, arg5: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn DefMDIChildProcW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn GetMonitorInfoW(arg1: HMONITOR, arg2: LPMONITORINFO) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetWindowLongW(arg1: HWND, arg2: ::core::ffi::c_int) -> LONG; // aclui-rust/src/types_i386.rs
    pub fn GetWindowTextLengthW(arg1: HWND) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // aclui-rust/src/types_i386.rs
    pub fn LoadMenuW(arg1: HINSTANCE, arg2: LPCWSTR) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RegCreateKeyExW( hKey: HKEY, lpSubKey: LPCWSTR, Reserved: DWORD, lpClass: LPWSTR, dwOptions: DWORD, samDesired: REGSAM, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, phkResult: PHKEY, lpdwDisposition: PDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegEnumKeyExW( hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, lpcchName: LPDWORD, lpReserved: LPDWORD, lpClass: LPWSTR, lpcchClass: LPDWORD, lpftLastWriteTime: PFILETIME, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegSetValueExW( hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn SetWindowLongW(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LONG) -> LONG; // aclui-rust/src/types_i386.rs
    pub fn SetWindowTextW(arg1: HWND, arg2: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
}
pub use crate::prelude::CreateWindowExW as CreateWindowEx; // sdk alias
pub use crate::prelude::DefFrameProcW as DefFrameProc; // sdk alias
pub use crate::prelude::DefMDIChildProcW as DefMDIChildProc; // sdk alias
pub use crate::prelude::FormatMessageW as FormatMessage; // sdk alias
pub use crate::prelude::GetMonitorInfoW as GetMonitorInfo; // sdk alias
pub use crate::prelude::GetWindowLongW as GetWindowLong; // sdk alias
pub use crate::prelude::GetWindowTextW as GetWindowText; // sdk alias
pub use crate::prelude::GetWindowTextLengthW as GetWindowTextLength; // sdk alias
pub use crate::prelude::LoadIconW as LoadIcon; // sdk alias
pub use crate::prelude::LoadMenuW as LoadMenu; // sdk alias
pub use crate::prelude::PostMessageW as PostMessage; // sdk alias
pub use crate::prelude::RegCreateKeyExW as RegCreateKeyEx; // sdk alias
pub use crate::prelude::RegEnumKeyExW as RegEnumKeyEx; // sdk alias
pub use crate::prelude::RegOpenKeyExW as RegOpenKeyEx; // sdk alias
pub use crate::prelude::RegQueryValueExW as RegQueryValueEx; // sdk alias
pub use crate::prelude::RegSetValueExW as RegSetValueEx; // sdk alias
pub use crate::prelude::SendMessageW as SendMessage; // sdk alias
pub use crate::prelude::SetWindowLongW as SetWindowLong; // sdk alias
pub use crate::prelude::SetWindowTextW as SetWindowText; // sdk alias
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
pub type MDICREATESTRUCT = MDICREATESTRUCTW;
pub type WNDCLASSEX = WNDCLASSEXW;
// ==== end UNICODE TCHAR family ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::mmc_new::misc_cpp_h::CmdIdToViewMode;
pub use crate::mmc_new::misc_cpp_h::ShowCmdFromString;
pub use crate::mmc_new::misc_cpp_h::ShowCmdToString;
pub use crate::mmc_new::misc_cpp_h::ViewModeToCmdId;
pub use crate::mmc_new::misc_cpp_h::ViewModeToString;
pub use crate::mmc_new::misc_cpp_h::ViewModeToStyle;
// ==== end cross-TU definitions ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! ListView_GetSelectedCount { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 50)) as _), ((0) as _), ((0) as _))) as UINT) }; }
#[macro_export] macro_rules! MAKEINTRESOURCE { ($p0:expr $(,)?) => { ((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as ULONG_PTR)) as LPWSTR) }; }
#[macro_export] macro_rules! MAKELPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM) }; }
#[macro_export] macro_rules! MAKEWPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as WPARAM) }; }
#[macro_export] macro_rules! TreeView_GetItemRect { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { { { (*(($p2) as *mut HTREEITEM)) = (($p1) as _); }; ((SendMessageW((($p0) as _), (((0x1100 + 4)) as _), (($p3) as WPARAM), (($p2) as LPARAM))) as BOOL) } }; }
#[macro_export] macro_rules! TreeView_SetItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 63)) as _), ((0) as _), (($p1) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! V_BSTR { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.bstrVal }; }
#[macro_export] macro_rules! V_VT { ($p0:expr $(,)?) => { (*$p0).n1.n2.vt }; }
pub const CW_USEDEFAULT: c_int = ((0x80000000u32) as c_int);
pub const E_FAIL: HRESULT = ((0x80004005u32) as HRESULT);
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT);
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT);
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT);
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT);
pub const HKEY_CURRENT_USER: HKEY = ((((((0x80000001u32) as LONG)) as LONG_PTR)) as HKEY);
pub const HKEY_LOCAL_MACHINE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY);
pub const IDC_ARROW: LPWSTR = 0x7f00usize as LPWSTR;
pub const IDC_SIZEWE: LPWSTR = 0x7f84usize as LPWSTR;
pub type LPNMTBGETINFOTIP = LPNMTBGETINFOTIPW;
pub type LPNMTREEVIEW = LPNMTREEVIEWW;
pub type LVITEM = LVITEMW;
pub const S_FALSE: HRESULT = ((1) as HRESULT);
pub const S_OK: HRESULT = ((0) as HRESULT);
pub const TOOLBARCLASSNAME: *const u16 = (&[0x0054, 0x006f, 0x006f, 0x006c, 0x0062, 0x0061, 0x0072, 0x0057, 0x0069, 0x006e, 0x0064, 0x006f, 0x0077, 0x0033, 0x0032, 0x0000u16]).as_ptr(); // L"ToolbarWindow32"
pub const TVI_FIRST: HTREEITEM = (((((-(0xffff as c_int))) as ULONG_PTR)) as HTREEITEM);
pub const TVI_LAST: HTREEITEM = (((((-(0xfffe as c_int))) as ULONG_PTR)) as HTREEITEM);
pub const TVI_ROOT: HTREEITEM = (((((-(0x10000 as c_int))) as ULONG_PTR)) as HTREEITEM);
pub type TV_DISPINFO = NMTVDISPINFOW;
pub type TV_INSERTSTRUCT = TVINSERTSTRUCTW;
pub type TV_ITEMW = TVITEMW;
pub const VARIANT_FALSE: VARIANT_BOOL = 0x0usize as VARIANT_BOOL;
pub const VARIANT_TRUE: VARIANT_BOOL = 0xffffusize as VARIANT_BOOL;
pub const WC_LISTVIEW: *const u16 = (&[0x0053, 0x0079, 0x0073, 0x004c, 0x0069, 0x0073, 0x0074, 0x0056, 0x0069, 0x0065, 0x0077, 0x0033, 0x0032, 0x0000u16]).as_ptr(); // L"SysListView32"
pub use crate::{ListView_GetSelectedCount, MAKEINTRESOURCE, MAKELPARAM, MAKEWPARAM, TreeView_GetItemRect, TreeView_SetItem, V_BSTR, V_VT};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// base/applications/mmc_new/mscfile.h:12: #define CHK_ALLOC(p) do { if (!(p)) { hr = E_OUTOFMEMORY; goto CleanUp; } } while(0)
#[allow(unused_macros)]
macro_rules! CHK_ALLOC {
    ($p:expr $(,)?) => {{
        if !($p) {
            hr = E_OUTOFMEMORY;
            break 'CleanUp;
        }
    }};
}
#[allow(unused_imports)]
pub(crate) use CHK_ALLOC;
// base/applications/mmc_new/mscfile.h:9: #define CHK_HR(stmt) do { hr=(stmt); if (FAILED(hr)) goto CleanUp; } while(0)
#[allow(unused_macros)]
macro_rules! CHK_HR {
    ($stmt:expr $(,)?) => {{
        hr = $stmt;
        if FAILED(hr) {
            break 'CleanUp;
        }
    }};
}
#[allow(unused_imports)]
pub(crate) use CHK_HR;
// base/applications/mmc_new/mscfile.h:15: #define SAFE_RELEASE(p) do { if ((p)) { (p)->Release(); (p) = NULL; } } while(0)
#[allow(unused_macros)]
macro_rules! SAFE_RELEASE {
    ($p:expr $(,)?) => {{
        if !($p).is_null() {
            (*$p).Release();
            $p = core::ptr::null_mut();
        }
    }};
}
#[allow(unused_imports)]
pub(crate) use SAFE_RELEASE;
// ==== end port macros ====
