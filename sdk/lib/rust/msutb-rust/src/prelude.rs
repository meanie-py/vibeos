//! msutb-rust prelude — the single import surface the translated modules use.
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
pub use atl_rust::{atl_format, CAtlList, CComPtr, CHeapPtr, CSimpleArray, CSimpleMap, CStringW, POSITION};

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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, ZeroMemory, _countof, debugstr_a, debugstr_an, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port cppapp: the module's C++ classes (generated -- `rustconv port cppapp`) ====
pub use crate::msutb::cpp_classes::*;
pub use crate::msutb::cpp_classes::{CButtonIconItem, CButtonIconItem__VTBL_b0, CButtonIconItem__delete, CButtonIconItem__make_0, CButtonIconItem__new_0, CButtonIconItem__new_in_0, CButtonIconItem__zeroed, CCicLibMenu, CCicLibMenuItem, CCicLibMenuItem__VTBL_own, CCicLibMenuItem__delete, CCicLibMenuItem__make_0, CCicLibMenuItem__new_0, CCicLibMenuItem__new_in_0, CCicLibMenuItem__vtbl, CCicLibMenuItem__zeroed, CCicLibMenu__VTBL_own, CCicLibMenu__delete, CCicLibMenu__make_0, CCicLibMenu__new_0, CCicLibMenu__new_in_0, CCicLibMenu__vtbl, CCicLibMenu__zeroed, CLBarInatItem, CLBarInatItem__VTBL_b0b0, CLBarInatItem__b0b1_ITfLangBarItem_VTBL, CLBarInatItem__b0b2_ITfLangBarItemButton_VTBL, CLBarInatItem__b0b3_ITfSource_VTBL, CLBarInatItem__delete, CLBarInatItem__make_0, CLBarInatItem__new_0, CLBarInatItem__new_in_0, CLBarInatItem__zeroed, CLBarItemBase, CLBarItemBase__VTBL_own, CLBarItemBase__delete, CLBarItemBase__make_0, CLBarItemBase__new_0, CLBarItemBase__new_in_0, CLBarItemBase__vtbl, CLBarItemBase__zeroed, CLBarItemButtonBase, CLBarItemButtonBase__ITfLangBarItemButton_VTBL, CLBarItemButtonBase__ITfLangBarItem_VTBL, CLBarItemButtonBase__ITfSource_VTBL, CLBarItemButtonBase__VTBL_b0, CLBarItemButtonBase__delete, CLBarItemButtonBase__make_0, CLBarItemButtonBase__new_0, CLBarItemButtonBase__new_in_0, CLBarItemButtonBase__zeroed, CLangBarItemList, CLangBarItemList__delete, CLangBarItemList__make_0, CLangBarItemList__new_0, CLangBarItemList__new_in_0, CLangBarItemList__zeroed, CMainIconItem, CMainIconItem__VTBL_b0b0, CMainIconItem__delete, CMainIconItem__make_0, CMainIconItem__new_0, CMainIconItem__new_in_0, CMainIconItem__zeroed, CModalMenu, CModalMenu__VTBL_own, CModalMenu__delete, CModalMenu__make_0, CModalMenu__new_0, CModalMenu__new_in_0, CModalMenu__vtbl, CModalMenu__zeroed, CMsUtbModule, CMsUtbModule__VTBL_own, CMsUtbModule__delete, CMsUtbModule__make_0, CMsUtbModule__new_0, CMsUtbModule__new_in_0, CMsUtbModule__vtbl, CMsUtbModule__zeroed, CRegKey, CRegKey__delete, CRegKey__make_0, CRegKey__make_1, CRegKey__make_2, CRegKey__new_0, CRegKey__new_1, CRegKey__new_2, CRegKey__new_in_0, CRegKey__new_in_1, CRegKey__new_in_2, CRegKey__zeroed, CShellWndThread, CShellWndThread__delete, CShellWndThread__make_0, CShellWndThread__new_0, CShellWndThread__new_in_0, CShellWndThread__zeroed, CTipbarAccItem, CTipbarAccItem__VTBL_own, CTipbarAccItem__delete, CTipbarAccItem__make_0, CTipbarAccItem__new_0, CTipbarAccItem__new_in_0, CTipbarAccItem__vtbl, CTipbarAccItem__zeroed, CTipbarAccessible, CTipbarAccessible__VTBL_own, CTipbarAccessible__delete, CTipbarAccessible__make_0, CTipbarAccessible__new_0, CTipbarAccessible__new_in_0, CTipbarAccessible__vtbl, CTipbarAccessible__zeroed, CTipbarCoInitialize, CTipbarCoInitialize__delete, CTipbarCoInitialize__make_0, CTipbarCoInitialize__new_0, CTipbarCoInitialize__new_in_0, CTipbarCoInitialize__zeroed, CTipbarGripper, CTipbarGripper__VTBL_own, CTipbarGripper__delete, CTipbarGripper__make_0, CTipbarGripper__new_0, CTipbarGripper__new_in_0, CTipbarGripper__vtbl, CTipbarGripper__zeroed, CTipbarItem, CTipbarItem__VTBL_b0, CTipbarItem__delete, CTipbarItem__make_0, CTipbarItem__new_0, CTipbarItem__new_in_0, CTipbarItem__vtbl, CTipbarItem__zeroed, CTipbarThread, CTipbarThread__VTBL_own, CTipbarThread__delete, CTipbarThread__make_0, CTipbarThread__new_0, CTipbarThread__new_in_0, CTipbarThread__vtbl, CTipbarThread__zeroed, CTipbarWnd, CTipbarWnd__VTBL_b2, CTipbarWnd__VTBL_own, CTipbarWnd__delete, CTipbarWnd__make_0, CTipbarWnd__new_0, CTipbarWnd__new_in_0, CTipbarWnd__vtbl, CTipbarWnd__zeroed, CTrayIconItem, CTrayIconItem__VTBL_own, CTrayIconItem__delete, CTrayIconItem__make_0, CTrayIconItem__new_0, CTrayIconItem__new_in_0, CTrayIconItem__vtbl, CTrayIconItem__zeroed, CTrayIconWnd, CTrayIconWnd__delete, CTrayIconWnd__make_0, CTrayIconWnd__new_0, CTrayIconWnd__new_in_0, CTrayIconWnd__zeroed, CUTBCloseLangBarDlg, CUTBCloseLangBarDlg__VTBL_b0, CUTBCloseLangBarDlg__delete, CUTBCloseLangBarDlg__make_0, CUTBCloseLangBarDlg__new_0, CUTBCloseLangBarDlg__new_in_0, CUTBCloseLangBarDlg__zeroed, CUTBContextMenu, CUTBContextMenu__VTBL_b0, CUTBContextMenu__delete, CUTBContextMenu__make_0, CUTBContextMenu__new_0, CUTBContextMenu__new_in_0, CUTBContextMenu__zeroed, CUTBLBarMenu, CUTBLBarMenuItem, CUTBLBarMenuItem__VTBL_b0, CUTBLBarMenuItem__delete, CUTBLBarMenuItem__make_0, CUTBLBarMenuItem__new_0, CUTBLBarMenuItem__new_in_0, CUTBLBarMenuItem__zeroed, CUTBLBarMenu__VTBL_b0, CUTBLBarMenu__delete, CUTBLBarMenu__make_0, CUTBLBarMenu__new_0, CUTBLBarMenu__new_in_0, CUTBLBarMenu__zeroed, CUTBLangBarDlg, CUTBLangBarDlg__VTBL_own, CUTBLangBarDlg__delete, CUTBLangBarDlg__make_0, CUTBLangBarDlg__new_0, CUTBLangBarDlg__new_in_0, CUTBLangBarDlg__vtbl, CUTBLangBarDlg__zeroed, CUTBMenuItem, CUTBMenuItem__VTBL_b0, CUTBMenuItem__delete, CUTBMenuItem__make_0, CUTBMenuItem__new_0, CUTBMenuItem__new_in_0, CUTBMenuItem__zeroed, CUTBMenuWnd, CUTBMenuWnd__VTBL_b0, CUTBMenuWnd__delete, CUTBMenuWnd__make_0, CUTBMenuWnd__new_0, CUTBMenuWnd__new_in_0, CUTBMenuWnd__vtbl, CUTBMenuWnd__zeroed, CUTBMinimizeLangBarDlg, CUTBMinimizeLangBarDlg__VTBL_b0, CUTBMinimizeLangBarDlg__delete, CUTBMinimizeLangBarDlg__make_0, CUTBMinimizeLangBarDlg__new_0, CUTBMinimizeLangBarDlg__new_in_0, CUTBMinimizeLangBarDlg__zeroed, LANGBARITEMSTATE, LANGBARITEMSTATE__delete, LANGBARITEMSTATE__make_0, LANGBARITEMSTATE__new_0, LANGBARITEMSTATE__new_in_0, LANGBARITEMSTATE__zeroed, cpp_alloc, cpp_alloc_CicNoThrow, cpp_delete, cpp_delete_array, cpp_free, cpp_new, cpp_new_array, cpp_new_array_CicNoThrow, cpp_new_in, cpp_pure_virtual_call, gModule};
// ==== end port cppapp ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1; // explorer-sys/src/lib.rs
pub const DISP_E_MEMBERNOTFOUND: HRESULT = 0x80020003u32 as HRESULT; // ole32-rust/src/hresults.rs
pub const E_FAIL: HRESULT = 0x80004005u32 as HRESULT; // aclui-rust/src/prelude.rs
pub const E_INVALIDARG: HRESULT = 0x80070057u32 as HRESULT; // appwiz-rust/src/prelude.rs
pub const E_NOTIMPL: HRESULT = 0x80004001u32 as HRESULT; // appwiz-rust/src/prelude.rs
pub const E_OUTOFMEMORY: HRESULT = 0x8007000Eu32 as HRESULT; // comctl32-rust/src/prelude.rs
pub const E_UNEXPECTED: HRESULT = 0x8000_FFFFu32 as HRESULT; // netcfgx-rust/src/prelude.rs
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as i32 as isize as HKEY; // access-rust/src/prelude.rs
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY; // console-rust/src/prelude.rs
pub const IDC_ARROW: LPCWSTR = 32512usize as LPCWSTR; // comctl32-rust/src/prelude.rs
pub const LANGID_CHINESE_SIMPLIFIED: i32 = 0x804; // imm32-rust/src/prelude.rs
pub const LANGID_CHINESE_TRADITIONAL: i32 = 0x404; // imm32-rust/src/prelude.rs
pub const LANGID_JAPANESE: i32 = 0x411; // imm32-rust/src/prelude.rs
pub const OBJID_CLIENT: u32 = 4294967292; // comctl32-rust/src/prelude.rs
pub const S_FALSE: HRESULT = 1; // comctl32-rust/src/prelude.rs
pub const S_OK: HRESULT = 0; // aclui-rust/src/prelude.rs
pub const TS_TRUE: THEMESIZE = THEMESIZE_TS_TRUE; // island enum
pub const TYPE_E_ELEMENTNOTFOUND: HRESULT = 0x8002802bu32 as HRESULT; // ole32-rust/src/hresults.rs
pub const VT_EMPTY: u16 = 0; // oleaut32-rust/src/consts.rs
pub const VT_I4: DWORD = 3; // shell32-rust/src/shell32/CDefView.rs
extern "C" {
    pub static IID_IDispatch: GUID; // aclui-rust/src/types_amd64.rs
}
extern "system" {
    pub fn CLSIDFromString(arg1: LPCOLESTR, arg2: LPCLSID) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CoCreateInstance( rclsid: *const CLSID, pUnkOuter: LPUNKNOWN, dwClsContext: DWORD, iid: *const IID, ppv: *mut LPVOID, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoInitialize(lpReserved: LPVOID) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemFree(ptr: LPVOID); // aclui-rust/src/types_i386.rs
    pub fn CoUninitialize(); // aclui-rust/src/types_i386.rs
    pub fn CreateThread( arg1: LPSECURITY_ATTRIBUTES, arg2: DWORD, arg3: LPTHREAD_START_ROUTINE, arg4: PVOID, arg5: DWORD, arg6: PDWORD, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn DeleteCriticalSection(arg1: PCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetCurrentThreadId() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetThemeFont( arg1: HTHEME, arg2: HDC, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: *mut LOGFONTW, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn GetThemeMargins( arg1: HTHEME, arg2: HDC, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: *mut RECT, arg7: *mut MARGINS, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn GetThemePartSize( arg1: HTHEME, arg2: HDC, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: *mut RECT, arg6: THEMESIZE, arg7: *mut SIZE, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn GetThemeSysSize(arg1: HTHEME, arg2: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetThemeTextExtent( arg1: HTHEME, arg2: HDC, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: LPCWSTR, arg6: ::core::ffi::c_int, arg7: DWORD, arg8: *const RECT, arg9: *mut RECT, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: PDWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn InitializeCriticalSectionAndSpinCount(arg1: LPCRITICAL_SECTION, arg2: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LoadRegTypeLib(libid: REFGUID, major: WORD, minor: WORD, lcid: LCID, out: *mut *mut ITypeLib) -> HRESULT; // shell32-rust/src/shell_dispatch.rs
    pub fn LoadTypeLib(szFile: LPCOLESTR, pptLib: *mut *mut ITypeLib) -> HRESULT; // oleaut32-rust/src/typelib.rs
    pub fn MulDiv( arg1: ::core::ffi::c_int, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegDeleteKeyW(hKey: HKEY, lpSubKey: LPCWSTR) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegEnumKeyExW( hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, lpcchName: LPDWORD, lpReserved: LPDWORD, lpClass: LPWSTR, lpcchClass: LPDWORD, lpftLastWriteTime: PFILETIME, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegEnumKeyW(hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, cchName: DWORD) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn Sleep(arg1: DWORD); // aclui-rust/src/types_i386.rs
    pub fn StringFromGUID2(id: *const GUID, str_: LPOLESTR, cmax: INT) -> INT; // aclui-rust/src/types_i386.rs
    pub fn SysAllocString(psz: *const u16) -> BSTR; // ole32-rust/src/ole32_exports.rs
    pub fn SysAllocStringLen(str_: LPCOLESTR, len: UINT) -> BSTR; // oleaut32-rust/src/typelib/sys.rs
    pub fn SysFreeString(bstr: BSTR); // oleaut32-rust/src/typelib.rs
    pub fn SysStringLen(str: BSTR) -> UINT; // oleaut32-rust/src/oleaut.rs
    pub fn lstrlenW(arg1: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const CIC_OSINFO_XPPLUS: i32 = 0x80; // clang, msutb.cpp
pub const LANGID_KOREAN: i32 = 0x412; // clang, msutb.cpp
pub const TF_E_NOLOCK: i32 = 0x80040201_u32 as i32; // clang, msutb.cpp
pub const TF_E_NOOBJECT: i32 = 0x80040202_u32 as i32; // clang, msutb.cpp
pub const TF_LBI_CLK_LEFT: __WIDL_ctfutb_generated_name_0000000D = 2; // clang, msutb.cpp
pub const TF_LBI_CLK_RIGHT: __WIDL_ctfutb_generated_name_0000000D = 1; // clang, msutb.cpp
pub const TIMER_ID_DOACCDEFAULTACTION: i32 = 0xB; // clang, msutb.cpp
pub const TIPBAR_ATTACHED: u32 = 1; // clang, msutb.cpp
pub const TIPBAR_BOTTOMFIT: u32 = 0x80000; // clang, msutb.cpp
pub const TIPBAR_CHILD: u32 = 2; // clang, msutb.cpp
pub const TIPBAR_ENSURING: u32 = 0x2000; // clang, msutb.cpp
pub const TIPBAR_HIGHCONTRAST: u32 = 0x10; // clang, msutb.cpp
pub const TIPBAR_LEFTFIT: u32 = 0x200000; // clang, msutb.cpp
pub const TIPBAR_NODESKBAND: u32 = 0x4000; // clang, msutb.cpp
pub const TIPBAR_RIGHTFIT: u32 = 0x100000; // clang, msutb.cpp
pub const TIPBAR_TOOLBARENDED: u32 = 0x10000; // clang, msutb.cpp
pub const TIPBAR_TOPFIT: u32 = 0x40000; // clang, msutb.cpp
pub const TIPBAR_TRAYICON: u32 = 0x20; // clang, msutb.cpp
pub const TIPBAR_UPDATING: u32 = 0x400; // clang, msutb.cpp
pub const TIPBAR_VERTICAL: u32 = 4; // clang, msutb.cpp
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn wcstoul(s: *const u16, end: *mut *mut u16, base: c_int) -> ULONG; // kernel32-rust/src/prelude.rs
}
extern "system" {
    pub fn CreateDCW( pszDriver: LPCWSTR, pszDevice: LPCWSTR, psz: LPCWSTR, pdmInit: *const DEVMODEW, ) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn CreateWindowExW( dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, X: ::core::ffi::c_int, Y: ::core::ffi::c_int, nWidth: ::core::ffi::c_int, nHeight: ::core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID, ) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn DefWindowProcW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn DialogBoxParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> INT_PTR; // aclui-rust/src/types_i386.rs
    pub fn EnumFontsW( arg1: HDC, arg2: LPCWSTR, arg3: FONTENUMPROCW, arg4: LPARAM, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn FindWindowW(arg1: LPCWSTR, arg2: LPCWSTR) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetClassNameW( hWnd: HWND, lpClassName: LPWSTR, nMaxCount: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    // i386 only: there `GetWindowLongPtrW` is a winuser.h macro for `GetWindowLongW`, so the island has no
    // declaration of it. On 64-bit the island's header-exact one is used.
    #[cfg(target_arch = "x86")]
    #[link_name = "GetWindowLongW"]
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR; // shell32-rust/src/copy_to_menu.rs
    pub fn LoadCursorW(arg1: HINSTANCE, arg2: LPCWSTR) -> HCURSOR; // aclui-rust/src/types_i386.rs
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RegCreateKeyExW( hKey: HKEY, lpSubKey: LPCWSTR, Reserved: DWORD, lpClass: LPWSTR, dwOptions: DWORD, samDesired: REGSAM, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, phkResult: PHKEY, lpdwDisposition: PDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegDeleteValueW(arg1: HKEY, arg2: LPCWSTR) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegisterClassExW(arg1: *const WNDCLASSEXW) -> ATOM; // aclui-rust/src/types_i386.rs
    pub fn SendMessageTimeoutW( arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM, arg5: UINT, arg6: UINT, arg7: PDWORD_PTR, ) -> LRESULT; // aclui-rust/src/types_i386.rs
    // i386 only: there `SetWindowLongPtrW` is a winuser.h macro for `SetWindowLongW`, so the island has no
    // declaration of it. On 64-bit the island's header-exact one is used.
    #[cfg(target_arch = "x86")]
    #[link_name = "SetWindowLongW"]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR; // shell32-rust/src/copy_to_menu.rs
    pub fn SetWindowTextW(arg1: HWND, arg2: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SystemParametersInfoW( uiAction: UINT, uiParam: UINT, pvParam: PVOID, fWinIni: UINT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn lstrcmpW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
pub use crate::prelude::CreateDCW as CreateDC; // sdk alias
pub use crate::prelude::CreateWindowExW as CreateWindowEx; // sdk alias
pub use crate::prelude::DefWindowProcW as DefWindowProc; // sdk alias
pub use crate::prelude::DialogBoxParamW as DialogBoxParam; // sdk alias
pub use crate::prelude::EnumFontsW as EnumFonts; // sdk alias
pub use crate::prelude::FindWindowW as FindWindow; // sdk alias
pub use crate::prelude::GetClassNameW as GetClassName; // sdk alias
pub use crate::prelude::GetWindowLongPtrW as GetWindowLongPtr; // sdk alias
pub use crate::prelude::LoadCursorW as LoadCursor; // sdk alias
pub use crate::prelude::PostMessageW as PostMessage; // sdk alias
pub use crate::prelude::RegCreateKeyExW as RegCreateKeyEx; // sdk alias
pub use crate::prelude::RegDeleteValueW as RegDeleteValue; // sdk alias
pub use crate::prelude::RegOpenKeyExW as RegOpenKeyEx; // sdk alias
pub use crate::prelude::RegisterClassExW as RegisterClassEx; // sdk alias
pub use crate::prelude::SendMessageTimeoutW as SendMessageTimeout; // sdk alias
pub use crate::prelude::SetWindowLongPtrW as SetWindowLongPtr; // sdk alias
pub use crate::prelude::SetWindowTextW as SetWindowText; // sdk alias
pub use crate::prelude::SystemParametersInfoW as SystemParametersInfo; // sdk alias
pub use crate::prelude::wcstoul as _tcstoul; // sdk alias
pub use crate::prelude::lstrcmpW as lstrcmp; // sdk alias
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
pub type HIGHCONTRAST = HIGHCONTRASTW;
pub type WNDCLASSEX = WNDCLASSEXW;
// ==== end UNICODE TCHAR family ====


// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! IsEqualCLSID { ($p0:expr, $p1:expr $(,)?) => { IsEqualGUID!(($p0), ($p1)) }; }
#[macro_export] macro_rules! IsEqualIID { ($p0:expr, $p1:expr $(,)?) => { IsEqualGUID!(($p0), ($p1)) }; }
#[macro_export] macro_rules! MAKEINTRESOURCE { ($p0:expr $(,)?) => { ((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as ULONG_PTR)) as LPWSTR) }; }
#[macro_export] macro_rules! MAKELPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM) }; }
#[macro_export] macro_rules! V_I4 { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.lVal }; }
#[macro_export] macro_rules! V_VT { ($p0:expr $(,)?) => { (*$p0).n1.n2.vt }; }
pub use crate::{IsEqualCLSID, IsEqualIID, MAKEINTRESOURCE, MAKELPARAM, V_I4, V_VT};
// ==== end SDK header macros ====
// ==== port resolve: path-dependency definitions (generated) ====
pub use cicero_rust::cicero::cicbase_cpp_h::{TFInitLib, cicGetOSInfo, cicRealCoCreateInstance};
pub use cicero_rust::cicero::cicbase_h::{cicNoThrow};
pub use cicero_rust::cicero::cicuif_cpp_h::{cicDoneUIFLib, cicGetIconBitmaps, cicGetScreenRect, cicGetWorkAreaRect, cicInitUIFLib, cicIsFullScreenSize};
pub use cicero_rust::cicero::cpp_classes::{CUIFTheme__zeroed, CicRegKey__zeroed};
// ==== end path-dependency definitions ====

// REFIID/REFCLSID/REFGUID: C++'s `const IID &`, a pointer at the ABI (as the island's vtables
// take it). resolve leaves them to a human: a dozen SDK headers each define them.
pub type REFGUID = *const GUID;
pub type REFCLSID = *const GUID;
pub type REFIID = *const IID;

// Classes msutb.cpp only forward-declares (and CDeskBand, defined under ENABLE_DESKBAND, which
// this build leaves off): reached only through pointers, as an incomplete C++ type is.
#[repr(C)]
pub struct CDeskBand {
    _incomplete: [u8; 0],
}
#[repr(C)]
pub struct CTipbarCtrlButtonHolder {
    _incomplete: [u8; 0],
}
#[repr(C)]
pub struct CTipbarBalloonItem {
    _incomplete: [u8; 0],
}

// ==== port resolve: INITGUID storage (generated) ====
#[no_mangle]
pub static CATID_DeskBand: GUID = GUID { Data1: 0x00021492, Data2: 0x0000, Data3: 0x0000, Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46] };
#[no_mangle]
pub static CLSID_MSUTBDeskBand: GUID = GUID { Data1: 0x540D8A8B, Data2: 0x1C3F, Data3: 0x4E32, Data4: [0x81, 0x32, 0x53, 0x0F, 0x6A, 0x50, 0x20, 0x90] };
#[no_mangle]
pub static CLSID_SYSTEMLANGBARITEM: GUID = GUID { Data1: 0xBEBACC94, Data2: 0x5CD3, Data3: 0x4662, Data4: [0xA1, 0xE0, 0xF3, 0x31, 0x99, 0x49, 0x36, 0x69] };
#[no_mangle]
pub static GUID_LBI_CTRL: GUID = GUID { Data1: 0x58C99D96, Data2: 0x2F9B, Data3: 0x42CE, Data4: [0x91, 0xBE, 0x37, 0xEF, 0x18, 0x60, 0xF8, 0x82] };
#[no_mangle]
pub static GUID_LBI_INATITEM: GUID = GUID { Data1: 0xCDBC683A, Data2: 0x55CE, Data3: 0x4717, Data4: [0xBA, 0xC0, 0x50, 0xBF, 0x44, 0xA3, 0x27, 0x0C] };
#[no_mangle]
pub static GUID_LBI_TRAYMAIN: GUID = GUID { Data1: 0xE0B724E9, Data2: 0x6F76, Data3: 0x45F7, Data4: [0xB4, 0xC1, 0xB1, 0xC0, 0xFA, 0xBC, 0xE2, 0x3E] };
#[no_mangle]
pub static IID_ITfLangBarEventSink_P: GUID = GUID { Data1: 0x7A460360, Data2: 0xDA21, Data3: 0x4B09, Data4: [0xA8, 0xA0, 0x8A, 0x69, 0xE7, 0x28, 0xD8, 0x93] };
#[no_mangle]
pub static IID_ITfLangBarMgr_P: GUID = GUID { Data1: 0xD72C0FA9, Data2: 0xADD5, Data3: 0x4AF0, Data4: [0x87, 0x06, 0x4F, 0xA9, 0xAE, 0x3E, 0x2E, 0xFF] };
// ==== end INITGUID storage ====

// ==== port resolve: wine debug channels (generated) ====
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
// ==== end wine debug channels ====

// ATL's BEGIN_OBJECT_MAP entry, as atlbase.h spells it (the island has it under ATL::).
pub type _ATL_OBJMAP_ENTRY = ATL__ATL_OBJMAP_ENTRY;
// msutb.cpp: `CicArray<HKL> *g_prghklSkipRedrawing` -- the generic, under the name the
// translation gave the instantiation.
pub type CicArray_HKL = CicArray<HKL>;

/// ATL's `CComModule` (sdk/lib/atl/atlbase.h), the members msutb's `CMsUtbModule gModule`
/// uses, ported as written. The class's own object map is walked exactly as ATL walks it.
/// What ATL then defers to -- `_AtlComModule`'s AUTO object map (the `ATL$__a..z` section
/// that OBJECT_ENTRY_AUTO fills) -- holds only its two null markers in msutb, which declares
/// no OBJECT_ENTRY_AUTO; so those calls reduce to ATL's own results for an empty auto map
/// (S_OK to register/unregister without a type library, CLASS_E_CLASSNOTAVAILABLE for an
/// unknown class). Type-library registration is not ported: msutb only ever passes FALSE,
/// and TRUE fails loudly with E_NOTIMPL rather than pretending to have registered one.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CComModule {
    pub m_pObjMap: *mut _ATL_OBJMAP_ENTRY,
    pub m_nLockCnt: LONG,
}
impl CComModule {
    pub unsafe fn __ctor_0(&mut self) {
        self.m_pObjMap = core::ptr::null_mut();
        self.m_nLockCnt = 0;
    }
    pub fn GetLockCount(&self) -> LONG {
        self.m_nLockCnt
    }
    pub unsafe fn Init(&mut self, p: *mut _ATL_OBJMAP_ENTRY, _h: HINSTANCE, _plibid: *const GUID) -> HRESULT {
        if p as isize != -1 {
            self.m_pObjMap = p;
            if !p.is_null() {
                let mut e = p;
                while !(*e).pclsid.is_null() {
                    if let Some(main) = (*e).pfnObjectMain {
                        main(true);
                    }
                    e = e.add(1);
                }
            }
        }
        S_OK
    }
    pub unsafe fn Term(&mut self) {
        if !self.m_pObjMap.is_null() {
            let mut e = self.m_pObjMap;
            while !(*e).pclsid.is_null() {
                if !(*e).pCF.is_null() {
                    (*(*e).pCF).Release();
                }
                (*e).pCF = core::ptr::null_mut();
                if let Some(main) = (*e).pfnObjectMain {
                    main(false);
                }
                e = e.add(1);
            }
        }
    }
    pub unsafe fn GetClassObject(&mut self, rclsid: REFCLSID, riid: REFIID, ppv: *mut LPVOID) -> HRESULT {
        if ppv.is_null() {
            return E_POINTER;
        }
        *ppv = core::ptr::null_mut();
        let mut hr: HRESULT = S_OK;
        if !self.m_pObjMap.is_null() {
            let mut e = self.m_pObjMap;
            while !(*e).pclsid.is_null() {
                if let Some(get) = (*e).pfnGetClassObject {
                    if IsEqualGUID!(rclsid, (*e).pclsid) {
                        if (*e).pCF.is_null() {
                            let create = (*e).pfnCreateInstance.map_or(core::ptr::null_mut(), |f| f as *mut c_void);
                            hr = get(create, &IID_IUnknown, core::ptr::addr_of_mut!((*e).pCF) as *mut LPVOID);
                        }
                        if !(*e).pCF.is_null() {
                            hr = (*(*e).pCF).QueryInterface(riid, ppv);
                        }
                        break;
                    }
                }
                e = e.add(1);
            }
        }
        if hr == S_OK && (*ppv).is_null() {
            hr = CLASS_E_CLASSNOTAVAILABLE; // AtlComModuleGetClassObject over msutb's empty auto map
        }
        hr
    }
    unsafe fn UpdateRegistry(&mut self, bRegister: BOOL, bTypeLib: BOOL) -> HRESULT {
        let mut hr: HRESULT = S_OK;
        let mut e = self.m_pObjMap;
        if !e.is_null() {
            while !(*e).pclsid.is_null() {
                if let Some(update) = (*e).pfnUpdateRegistry {
                    hr = update(bRegister);
                    if hr < 0 {
                        break;
                    }
                }
                e = e.add(1);
            }
        }
        if hr >= 0 && bTypeLib != 0 {
            hr = E_NOTIMPL; // type-library (un)registration is not ported (see above)
        }
        hr
    }
    pub unsafe fn DllRegisterServer(&mut self, bRegTypeLib: BOOL) -> HRESULT {
        self.UpdateRegistry(TRUE as BOOL, bRegTypeLib)
    }
    pub unsafe fn DllUnregisterServer(&mut self, bUnRegTypeLib: BOOL) -> HRESULT {
        self.UpdateRegistry(FALSE as BOOL, bUnRegTypeLib)
    }
    pub fn DllCanUnloadNow(&self) -> HRESULT {
        if self.GetLockCount() == 0 { S_OK } else { S_FALSE }
    }
    pub unsafe fn DllGetClassObject(&mut self, rclsid: REFCLSID, riid: REFIID, ppv: *mut LPVOID) -> HRESULT {
        self.GetClassObject(rclsid, riid, ppv)
    }
}

// cicero/cicutb.h's inline TFUninitLib_Thread: header-only, so no crate translated it.
pub unsafe fn TFUninitLib_Thread(pLibThread: PCIC_LIBTHREAD) {
    if pLibThread.is_null() {
        return;
    }
    if !(*pLibThread).m_pCategoryMgr.is_null() {
        (*(*pLibThread).m_pCategoryMgr).Release();
        (*pLibThread).m_pCategoryMgr = core::ptr::null_mut();
    }
    if !(*pLibThread).m_pDisplayAttrMgr.is_null() {
        (*(*pLibThread).m_pDisplayAttrMgr).Release();
        (*pLibThread).m_pDisplayAttrMgr = core::ptr::null_mut();
    }
}

// cicero/cicutb.h: `struct ITfLangBarMgr_P : ITfLangBarMgr { STDMETHOD(GetPrevShowFloatingStatus)
// (DWORD*) = 0; }` -- C++-only, so its table is ITfLangBarMgr's slots and then this one.
#[repr(C)]
pub struct ITfLangBarMgr_PVtbl {
    pub __base: ITfLangBarMgrVtbl,
    pub GetPrevShowFloatingStatus: unsafe extern "system" fn(*mut ITfLangBarMgr_P, *mut DWORD) -> HRESULT,
}
impl ITfLangBarMgr_P {
    pub unsafe fn GetPrevShowFloatingStatus(&mut self, pdw: *mut DWORD) -> HRESULT {
        let vt = self._base.lpVtbl as *const ITfLangBarMgr_PVtbl;
        ((*vt).GetPrevShowFloatingStatus)(self, pdw)
    }
}

// winerror.h / uuid: what the CComModule port above names.
pub const E_POINTER: HRESULT = 0x80004003u32 as HRESULT;
pub const CLASS_E_CLASSNOTAVAILABLE: HRESULT = 0x80040111u32 as HRESULT;
extern "C" {
    pub static IID_IUnknown: IID; // uuid.lib
}

impl CRegKey {
    /// atlbase.h `CRegKey::_DoDeleteKeyTree`, the helper RecurseDeleteKey (msutb calls it
    /// twice) recurses through. The atlbase_h unit carried only the members msutb calls by
    /// name, so this one came along by hand, as written: `key.Open` with ATL's default
    /// `KEY_READ | KEY_WRITE`, and `key.EnumKey` inlined to the `RegEnumKeyEx` it is.
    pub unsafe fn _DoDeleteKeyTree(hParentKey: HKEY, lpszKey: LPCTSTR) -> LONG {
        let mut key: CRegKey = CRegKey__make_0();
        let mut ret: LONG = key.Open(hParentKey, lpszKey, (KEY_READ | KEY_WRITE) as REGSAM);
        if ret != ERROR_SUCCESS as LONG {
            return ret;
        }

        let mut NameMax: DWORD = 0;
        ret = RegQueryInfoKeyW(
            key.m_hKey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(NameMax),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as LONG;
        if ret != ERROR_SUCCESS as LONG {
            return ret;
        }
        NameMax += 1; // for NUL

        let mut szNameBuf: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let pszName: *mut WCHAR;
        if NameMax > MAX_PATH as DWORD {
            pszName = malloc(NameMax as usize * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
            if pszName.is_null() {
                return ERROR_OUTOFMEMORY as LONG;
            }
        } else {
            NameMax = MAX_PATH as DWORD;
            pszName = szNameBuf.as_mut_ptr();
        }

        loop {
            let mut Count: DWORD = NameMax;
            ret = RegEnumKeyExW(
                key.m_hKey,
                0,
                pszName,
                core::ptr::addr_of_mut!(Count),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) as LONG;
            if ret != ERROR_SUCCESS as LONG {
                if ret == ERROR_NO_MORE_ITEMS as LONG {
                    ret = ERROR_SUCCESS as LONG;
                }
                break;
            }

            ret = CRegKey::_DoDeleteKeyTree(key.m_hKey, pszName);
            if ret != ERROR_SUCCESS as LONG {
                break;
            }
        }

        key.Close();

        if ret == ERROR_SUCCESS as LONG {
            ret = RegDeleteKeyW(hParentKey, lpszKey) as LONG;
        }

        if pszName != szNameBuf.as_mut_ptr() {
            free(pszName as *mut c_void);
        }
        ret
    }
}

// The Win32/CRT entry points the _DoDeleteKeyTree port calls that the island leaves out.
extern "system" {
    pub fn RegQueryInfoKeyW(
        hKey: HKEY,
        lpClass: LPWSTR,
        lpcchClass: LPDWORD,
        lpReserved: LPDWORD,
        lpcSubKeys: LPDWORD,
        lpcbMaxSubKeyLen: LPDWORD,
        lpcbMaxClassLen: LPDWORD,
        lpcValues: LPDWORD,
        lpcbMaxValueNameLen: LPDWORD,
        lpcbMaxValueLen: LPDWORD,
        lpcbSecurityDescriptor: LPDWORD,
        lpftLastWriteTime: *mut FILETIME,
    ) -> LSTATUS;
}
extern "C" {
    pub fn malloc(n: usize) -> *mut c_void;
    pub fn free(p: *mut c_void);
}
