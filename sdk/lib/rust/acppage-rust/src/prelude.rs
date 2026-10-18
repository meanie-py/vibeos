//! acppage-rust prelude — the single import surface the translated modules use.
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
// atlstr.h's TCHAR `CString` is CStringW in a UNICODE module.
pub use atl_rust::CStringW as CString;
pub use atl_rust::CComHeapPtr;
// shellutils.h's CCoInit (GetExeFromLnk).
pub use atl_rust::shellutils::CCoInit;
pub use atl_rust::bstr::CComBSTR;

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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedDecrement64, InterlockedExchange, InterlockedIncrement, InterlockedIncrement64, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _byteswap_uint64, _byteswap_ulong, _byteswap_ushort, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_variant, debugstr_vt, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, lrintf, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_longlong, wine_dbgstr_variant, wine_dbgstr_vt, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port cppapp: the module's C++ classes (generated -- `rustconv port cppapp`) ====
pub use crate::acppage::cpp_classes::*;
pub use crate::acppage::cpp_classes::{CEditCompatModes, CEditCompatModes__VTBL_own, CEditCompatModes__delete, CEditCompatModes__make_0, CEditCompatModes__new_0, CEditCompatModes__new_in_0, CEditCompatModes__vtbl, CEditCompatModes__zeroed, CLayerStringList, CLayerStringList__IEnumString_VTBL, CLayerStringList__delete, CLayerStringList__make_0, CLayerStringList__new_0, CLayerStringList__new_in_0, CLayerStringList__zeroed, CLayerUIPropPage, CLayerUIPropPage__IShellExtInit_VTBL, CLayerUIPropPage__IShellPropSheetExt_VTBL, CLayerUIPropPage__delete, CLayerUIPropPage__make_0, CLayerUIPropPage__new_0, CLayerUIPropPage__new_in_0, CLayerUIPropPage__zeroed, CWindow, CWindow__delete, CWindow__make_0, CWindow__new_0, CWindow__new_in_0, CWindow__rcDefault, CWindow__zeroed, IDD, ObjectMap, _CreatorClass, _IDHWNDPAIR, _IDHWNDPAIR__delete, _IDHWNDPAIR__make_0, _IDHWNDPAIR__new_0, _IDHWNDPAIR__new_in_0, _IDHWNDPAIR__zeroed, _bindgen_ty_1, cpp_alloc, cpp_delete, cpp_delete_array, cpp_free, cpp_new, cpp_new_array, cpp_new_in, cpp_pure_virtual_call, gModule};
// ==== end port cppapp ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1; // cleanmgr-rust/src/prelude.rs
pub const DOS_PATH: _PATH_TYPE = _PATH_TYPE_DOS_PATH; // island enum
pub const DVASPECT_CONTENT: DWORD = 1; // deskadp-rust/src/prelude.rs
pub const GPLK_MACHINE: i32 = 2; // apphelp-rust/src/apphelp/hsdb.rs
pub const GPLK_USER: i32 = 1; // apphelp-rust/src/apphelp/hsdb.rs
pub const MAX_LAYER_LENGTH: u32 = 256; // apphelp-rust/src/apphelp/layer.rs
pub const NM_CLICK: u32 = 0xFFFFFFFE; // comctl32-rust/src/prelude.rs
pub const NM_RETURN: u32 = 0xFFFFFFFC; // comctl32-rust/src/prelude.rs
pub const PSNRET_NOERROR: u32 = 0; // access-rust/src/types_amd64.rs
pub const PSP_USETITLE: u32 = 8; // access-rust/src/types_amd64.rs
pub const SLGP_RAWPATH: DWORD = 0x0004; // shell32-rust/src/shell_link.rs
pub const SLR_NOSEARCH: DWORD = 0x0010; // shell32-rust/src/shell_link.rs
pub const SLR_NOUPDATE: DWORD = 0x0008; // shell32-rust/src/shell_link.rs
pub const SLR_NO_UI: DWORD = 0x0001; // shell32-rust/src/shell_link.rs
pub const STGM_READ: u32 = 0; // appwiz-rust/src/prelude.rs
pub const TAGID_NULL: u32 = 0; // apphelp-rust/src/types_amd64.rs
pub const TAGID_ROOT: u32 = 0; // apphelp-rust/src/types_amd64.rs
pub const TAG_DATABASE: u32 = 28673; // apphelp-rust/src/types_amd64.rs
pub const TAG_LAYER: u32 = 28683; // apphelp-rust/src/types_amd64.rs
pub const TAG_NAME: u32 = 24577; // apphelp-rust/src/types_amd64.rs
pub const TYMED_HGLOBAL: DWORD = 1; // deskadp-rust/src/prelude.rs
extern "C" {
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // apphelp-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcstok(s: *mut u16, delim: *const u16) -> *mut u16; // desk-rust/src/prelude.rs
    pub static IID_IEnumString: GUID; // aclui-rust/src/types_amd64.rs
}
extern "system" {
    pub fn CheckDlgButton(arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CoCreateInstance( rclsid: *const CLSID, pUnkOuter: LPUNKNOWN, dwClsContext: DWORD, iid: *const IID, ppv: *mut LPVOID, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemAlloc(size: SIZE_T) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn DestroyPropertySheetPage(arg1: HPROPSHEETPAGE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ExpandEnvironmentStringsW(arg1: LPCWSTR, arg2: LPWSTR, arg3: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetParent(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetWindowRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GlobalLock(arg1: HGLOBAL) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn GlobalUnlock(arg1: HGLOBAL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsDlgButtonChecked(arg1: HWND, arg2: ::core::ffi::c_int) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn MonitorFromWindow(arg1: HWND, arg2: DWORD) -> HMONITOR; // aclui-rust/src/types_i386.rs
    pub fn MoveWindow( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: BOOL, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn OutputDebugStringW(arg1: LPCWSTR); // aclui-rust/src/types_i386.rs
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn ReleaseStgMedium(arg1: LPSTGMEDIUM); // aclui-rust/src/types_i386.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn SetFocus(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn GetMonitorInfoW(arg1: HMONITOR, arg2: LPMONITORINFO) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetWindowLongW(arg1: HWND, arg2: ::core::ffi::c_int) -> LONG; // aclui-rust/src/types_i386.rs
    pub fn SetDlgItemTextW(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ShellExecuteW( hwnd: HWND, lpOperation: LPCWSTR, lpFile: LPCWSTR, lpParameters: LPCWSTR, lpDirectory: LPCWSTR, nShowCmd: INT, ) -> HINSTANCE; // ctfmon-rust/src/types_i386.rs
}
pub use crate::prelude::GetMonitorInfoW as GetMonitorInfo; // sdk alias
pub use crate::prelude::GetWindowLongW as GetWindowLong; // sdk alias
pub use crate::prelude::SetDlgItemTextW as SetDlgItemText; // sdk alias
pub use crate::prelude::ShellExecuteW as ShellExecute; // sdk alias
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
pub use crate::acppage::ACPPage_cpp_h::GetExeFromLnk;
// ==== end cross-TU definitions ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! ComboBox_AddString { ($p0:expr, $p1:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((323) as _), ((0) as _), (($p1) as LPARAM))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ComboBox_GetCurSel { ($p0:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((327) as _), ((0) as _), ((0) as _))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ComboBox_SetCurSel { ($p0:expr, $p1:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((334) as _), (((($p1) as c_int)) as WPARAM), ((0) as _))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ListBox_AddString { ($p0:expr, $p1:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((384) as _), ((0) as _), (($p1) as LPARAM))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ListBox_DeleteString { ($p0:expr, $p1:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((386) as _), (((($p1) as c_int)) as WPARAM), ((0) as _))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ListBox_FindStringExact { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((418) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ListBox_GetCount { ($p0:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((395) as _), ((0) as _), ((0) as _))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ListBox_GetCurSel { ($p0:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((392) as _), ((0) as _), ((0) as _))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ListBox_GetText { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((393) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ListBox_GetTextLen { ($p0:expr, $p1:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((394) as _), (((($p1) as c_int)) as WPARAM), ((0) as _))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ListBox_ResetContent { ($p0:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((388) as _), ((0) as _), ((0) as _))) as DWORD)) as BOOL) }; }
#[macro_export] macro_rules! ListBox_SetCurSel { ($p0:expr, $p1:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((390) as _), (((($p1) as c_int)) as WPARAM), ((0) as _))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! va_end { ($p0:expr $(,)?) => { __builtin_va_end((($p0) as _)) }; }
#[macro_export] macro_rules! va_start { ($p0:expr, $p1:expr $(,)?) => { __builtin_va_start((($p0) as _), (($p1) as _)) }; }
pub const CW_USEDEFAULT: c_int = ((0x80000000u32) as c_int);
pub const E_ACCESSDENIED: HRESULT = ((0x80070005u32) as HRESULT);
pub const E_FAIL: HRESULT = ((0x80004005u32) as HRESULT);
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT);
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT);
pub const HKEY_LOCAL_MACHINE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY);
pub type REFCLSID = *const CLSID;
pub type REFIID = *const IID;
pub const S_FALSE: HRESULT = ((1) as HRESULT);
pub const S_OK: HRESULT = ((0) as HRESULT);
pub use crate::{ComboBox_AddString, ComboBox_GetCurSel, ComboBox_SetCurSel, ListBox_AddString, ListBox_DeleteString, ListBox_FindStringExact, ListBox_GetCount, ListBox_GetCurSel, ListBox_GetText, ListBox_GetTextLen, ListBox_ResetContent, ListBox_SetCurSel, va_end, va_start};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/shellext/acppage/CLayerUIPropPage.cpp:106: #define ACDBG(fmt, ...) ACDBG_FN(__FUNCTION__, fmt, ##__VA_ARGS__ )
#[allow(unused_macros)]
macro_rules! ACDBG {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {
        (ACDBG_FN(concat!(module_path!(), "\0").as_ptr() as *const core::ffi::c_char, $fmt $(, $arg)*))
    };
}
#[allow(unused_imports)]
pub(crate) use ACDBG;
// ==== end port macros ====

// ==== port variadic: C-variadic entry points, trampolines in the module's rust_varargs_shim.c (generated -- `rustconv port integrate`) ====
extern "C" {
    pub fn ACDBG_FN(FunctionName: PCSTR, Format: PCWSTR, ...);
}
// ==== end port variadic ====
