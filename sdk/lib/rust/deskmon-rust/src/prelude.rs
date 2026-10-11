//! deskmon-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// Hand port, kept OUTSIDE the generated blocks. deskmon.h:
//   #define impl_to_interface(impl,iface) (struct iface *)(&(impl)->lp##iface##Vtbl)
//   #define interface_to_impl(instance,iface) ((PDESKMONITOR)((ULONG_PTR)instance - FIELD_OFFSET(DESKMONITOR,lp##iface##Vtbl)))
// macro_rules cannot paste `lp##iface##Vtbl`, so each interface the object implements is an arm.
#[allow(unused_macros)]
macro_rules! impl_to_interface {
    ($impl:expr, IShellPropSheetExt) => {
        core::ptr::addr_of_mut!((*$impl).lpIShellPropSheetExtVtbl) as *mut IShellPropSheetExt
    };
    ($impl:expr, IShellExtInit) => {
        core::ptr::addr_of_mut!((*$impl).lpIShellExtInitVtbl) as *mut IShellExtInit
    };
    ($impl:expr, IClassFactory) => {
        core::ptr::addr_of_mut!((*$impl).lpIClassFactoryVtbl) as *mut IClassFactory
    };
}
#[allow(unused_imports)]
pub(crate) use impl_to_interface;
#[allow(unused_macros)]
macro_rules! interface_to_impl {
    ($inst:expr, IShellPropSheetExt) => {
        (($inst as ULONG_PTR) - core::mem::offset_of!(DESKMONITOR, lpIShellPropSheetExtVtbl) as ULONG_PTR) as PDESKMONITOR
    };
    ($inst:expr, IShellExtInit) => {
        (($inst as ULONG_PTR) - core::mem::offset_of!(DESKMONITOR, lpIShellExtInitVtbl) as ULONG_PTR) as PDESKMONITOR
    };
    ($inst:expr, IClassFactory) => {
        (($inst as ULONG_PTR) - core::mem::offset_of!(DESKMONITOR, lpIClassFactoryVtbl) as ULONG_PTR) as PDESKMONITOR
    };
}
#[allow(unused_imports)]
pub(crate) use interface_to_impl;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const CP_ACP: u32 = 0; // advapi32-sys/src/bindgen_amd64.rs
pub const DVASPECT_CONTENT: tagDVASPECT = tagDVASPECT_DVASPECT_CONTENT; // island enum
pub const TYMED_HGLOBAL: DWORD = 1; // regedit-rust/src/prelude.rs
extern "C" {
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub static IID_IClassFactory: GUID; // desk-rust/src/types_amd64.rs
    pub static IID_IShellExtInit: GUID; // appwiz-rust/src/types_amd64.rs
    pub static IID_IShellPropSheetExt: GUID; // appwiz-rust/src/types_amd64.rs
    pub static IID_IUnknown: GUID; // desk-rust/src/types_amd64.rs
}
extern "system" {
    pub fn CM_Get_Child(pdnDevInst: PDEVINST, dnDevInst: DEVINST, ulFlags: ULONG) -> CONFIGRET; // desk-rust/src/types_i386.rs
    pub fn CM_Get_Device_ID_Size(pulLen: PULONG, dnDevInst: DEVINST, ulFlags: ULONG) -> CONFIGRET; // desk-rust/src/types_i386.rs
    pub fn CM_Get_Sibling(pdnDevInst: PDEVINST, DevInst: DEVINST, ulFlags: ULONG) -> CONFIGRET; // desk-rust/src/types_i386.rs
    pub fn CheckDlgButton(arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetParent(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GlobalLock(arg1: HGLOBAL) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn GlobalUnlock(arg1: HGLOBAL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsDlgButtonChecked(arg1: HWND, arg2: ::core::ffi::c_int) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn LocalAlloc(arg1: UINT, arg2: SIZE_T) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn ReleaseStgMedium(pmedium: *mut STGMEDIUM); // desk-rust/src/prelude.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn WideCharToMultiByte( arg1: UINT, arg2: DWORD, arg3: LPCWSTR, arg4: ::core::ffi::c_int, arg5: LPSTR, arg6: ::core::ffi::c_int, arg7: LPCSTR, arg8: LPBOOL, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::deskmon::deskcplx_h::DeskCplExtDisplaySaveSettings;
pub use crate::deskmon::deskcplx_h::QueryDeskCplExtInterface;
pub use crate::deskmon::deskcplx_h::QueryDeskCplString;
pub use crate::deskmon::shxiface::dll_refs;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! IsEqualCLSID { ($p0:expr, $p1:expr $(,)?) => { (((memcmp((($p0) as _), (($p1) as _), ((core::mem::size_of::<GUID>()) as _))) as isize == 0) as c_int) }; }
#[macro_export] macro_rules! IsEqualIID { ($p0:expr, $p1:expr $(,)?) => { (((memcmp((($p0) as _), (($p1) as _), ((core::mem::size_of::<GUID>()) as _))) as isize == 0) as c_int) }; }
#[macro_export] macro_rules! MAKEINTRESOURCE { ($p0:expr $(,)?) => { ((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as ULONG_PTR)) as LPWSTR) }; }
#[macro_export] macro_rules! PropSheet_Changed { ($p0:expr, $p1:expr $(,)?) => { SendMessageW((($p0) as _), (((1024 + 104)) as _), (($p1) as WPARAM), ((0) as _)) }; }
#[macro_export] macro_rules! PropSheet_RestartWindows { ($p0:expr $(,)?) => { SendMessageW((($p0) as _), (((1024 + 105)) as _), ((0) as _), ((0) as _)) }; }
pub const CLASS_E_CLASSNOTAVAILABLE: HRESULT = ((0x80040111u32) as HRESULT);
pub const CLASS_E_NOAGGREGATION: HRESULT = ((0x80040110u32) as HRESULT);
pub const DESK_EXT_DISPLAYDEVICE: *const u16 = (&[0x0044, 0x0069, 0x0073, 0x0070, 0x006c, 0x0061, 0x0079, 0x0020, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x0000u16]).as_ptr(); // L"Display Device"
pub const DESK_EXT_DISPLAYID: *const u16 = (&[0x0044, 0x0069, 0x0073, 0x0070, 0x006c, 0x0061, 0x0079, 0x0020, 0x0049, 0x0044, 0x0000u16]).as_ptr(); // L"Display ID"
pub const DESK_EXT_MONITORNAME: *const u16 = (&[0x004d, 0x006f, 0x006e, 0x0069, 0x0074, 0x006f, 0x0072, 0x0020, 0x004e, 0x0061, 0x006d, 0x0065, 0x0000u16]).as_ptr(); // L"Monitor Name"
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT);
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT);
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT);
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT);
#[inline]
pub unsafe fn IDataObject_AddRef(This: *mut IDataObject) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IDataObject_GetData(This: *mut IDataObject, pformatetcIn: *mut FORMATETC, pmedium: *mut STGMEDIUM) -> HRESULT { ((*(*This).lpVtbl).GetData.unwrap_unchecked())(This, pformatetcIn, pmedium) } // widl C wrapper
#[inline]
pub unsafe fn IDataObject_Release(This: *mut IDataObject) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
pub type LPCPROPSHEETPAGE = LPCPROPSHEETPAGEW;
pub type PROPSHEETPAGE = PROPSHEETPAGEW;
pub type REFCLSID = *const CLSID;
pub type REFIID = *const IID;
pub const S_FALSE: HRESULT = ((1) as HRESULT);
pub const S_OK: HRESULT = ((0) as HRESULT);
pub use crate::{IsEqualCLSID, IsEqualIID, MAKEINTRESOURCE, PropSheet_Changed, PropSheet_RestartWindows};
// ==== end SDK header macros ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _snwprintf(
        _Dest: *mut wchar_t,
        _Count: size_t,
        _Format: *const wchar_t,
        ...
    ) -> ::core::ffi::c_int; // deskmon.c
    pub fn wcscat(_Dest: *mut wchar_t, _Source: *const wchar_t) -> *mut wchar_t; // deskmon.c
    pub fn wcscmp(_Str1: *const wchar_t, _Str2: *const wchar_t) -> ::core::ffi::c_int; // deskmon.c
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
}
extern "system" {
    pub fn CM_Get_DevNode_Registry_PropertyW(
        dnDevInst: DEVINST,
        ulProperty: ULONG,
        pulRegDataType: PULONG,
        Buffer: PVOID,
        pulLength: PULONG,
        ulFlags: ULONG,
    ) -> CONFIGRET; // deskmon.c
    pub fn CM_Get_Device_IDW(
        dnDevInst: DEVINST,
        Buffer: PWCHAR,
        BufferLen: ULONG,
        ulFlags: ULONG,
    ) -> CONFIGRET; // deskmon.c
    pub fn CM_Locate_DevNodeW(
        pdnDevInst: PDEVINST,
        pDeviceID: DEVINSTID_W,
        ulFlags: ULONG,
    ) -> CONFIGRET; // deskmon.c
    pub fn CreatePropertySheetPageW(_: LPCPROPSHEETPAGEW) -> HPROPSHEETPAGE; // deskmon.c
    pub fn EnumDisplayDevicesW(_: LPCWSTR, _: DWORD, _: PDISPLAY_DEVICEW, _: DWORD) -> BOOL; // deskmon.c
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(_: HWND, _: ::core::ffi::c_int) -> LONG_PTR; // deskmon.c
    pub fn LoadLibraryW(_: LPCWSTR) -> HINSTANCE; // deskmon.c
    pub fn LoadStringW(
        hInstance_0: HINSTANCE,
        uID: UINT,
        lpBuffer: LPWSTR,
        cchBufferMax: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int; // deskmon.c
    pub fn RegisterClipboardFormatW(_: LPCWSTR) -> UINT; // deskmon.c
    pub fn SendDlgItemMessageW(
        _: HWND,
        _: ::core::ffi::c_int,
        _: UINT,
        _: WPARAM,
        _: LPARAM,
    ) -> LRESULT; // deskmon.c
    pub fn SetDlgItemTextW(_: HWND, _: ::core::ffi::c_int, _: LPCWSTR) -> BOOL; // deskmon.c
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(_: HWND, _: ::core::ffi::c_int, _: LONG_PTR) -> LONG_PTR; // deskmon.c
}
pub use crate::prelude::CM_Get_DevNode_Registry_PropertyW as CM_Get_DevNode_Registry_Property; // sdk alias
pub use crate::prelude::CM_Get_Device_IDW as CM_Get_Device_ID; // sdk alias
pub use crate::prelude::CM_Locate_DevNodeW as CM_Locate_DevNode; // sdk alias
pub use crate::prelude::CreatePropertySheetPageW as CreatePropertySheetPage; // sdk alias
pub use crate::prelude::EnumDisplayDevicesW as EnumDisplayDevices; // sdk alias
pub use crate::prelude::GetWindowLongPtrW as GetWindowLongPtr; // sdk alias
pub use crate::prelude::LoadLibraryW as LoadLibrary; // sdk alias
pub use crate::prelude::LoadStringW as LoadString; // sdk alias
pub use crate::prelude::RegisterClipboardFormatW as RegisterClipboardFormat; // sdk alias
pub use crate::prelude::SendDlgItemMessageW as SendDlgItemMessage; // sdk alias
pub use crate::prelude::SetDlgItemTextW as SetDlgItemText; // sdk alias
pub use crate::prelude::SetWindowLongPtrW as SetWindowLongPtr; // sdk alias
pub use crate::prelude::_snwprintf as _sntprintf; // sdk alias
pub use crate::prelude::wcscat as _tcscat; // sdk alias
pub use crate::prelude::wcscmp as _tcscmp; // sdk alias
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
// ==== end UNICODE TCHAR family ====
