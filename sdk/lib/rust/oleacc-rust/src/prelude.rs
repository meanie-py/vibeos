//! oleacc-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedDecrement64, InterlockedExchange, InterlockedExchangePointer, InterlockedIncrement, InterlockedIncrement64, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _byteswap_uint64, _byteswap_ulong, _byteswap_ushort, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_variant, debugstr_vt, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, lrintf, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_longlong, wine_dbgstr_variant, wine_dbgstr_vt, wine_dbgstr_w};

// The widl proxy's entry points: proxy.dlldata.c (ENTRY_PREFIX=OLEACC_) stays C and links
// into this DLL, so no import library exports these; main.c declares them itself.
extern "system" {
    pub fn OLEACC_DllGetClassObject(rclsid: REFCLSID, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    pub fn OLEACC_DllMain(hinst: HINSTANCE, reason: DWORD, reserved: LPVOID) -> BOOL;
    pub fn OLEACC_DllRegisterServer() -> HRESULT;
    pub fn OLEACC_DllUnregisterServer() -> HRESULT;
}

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const OBJID_CLIENT: LONG = 0xFFFFFFFC_u32 as LONG; // clang, main.c
pub const OBJID_WINDOW: LONG = 0; // clang, client.c
// ==== end port facts ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const MSHCTX_LOCAL: DWORD = 0; // ole32-rust/src/compobj/com.rs
pub const MSHLFLAGS_NORMAL: DWORD = 0; // ole32-rust/src/compobj/com.rs
pub const STATFLAG_NONAME: DWORD = 1; // mfplat-rust/src/prelude.rs
pub const STREAM_SEEK_SET: DWORD = 0; // comctl32-rust/src/prelude.rs
pub const VT_DISPATCH: u16 = 9; // mi-rust/src/sys.rs
pub const VT_EMPTY: u16 = 0; // cabview-rust/src/prelude.rs
pub const VT_I4: VARENUM = VARENUM_VT_I4; // island enum
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3; // activeds-rust/src/prelude.rs
extern "C" {
    #[link_name = "_snwprintf"]
    pub fn swprintf(s: *mut u16, n: usize, fmt: *const u16, ...) -> c_int; // credui-rust/src/prelude.rs
    pub fn wcstoul(s: *const u16, endptr: *mut *mut u16, base: i32) -> u32; // at-rust/src/prelude.rs
}
extern "system" {
    pub fn ChildWindowFromPointEx(arg1: HWND, arg2: POINT, arg3: UINT) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn ClientToScreen(arg1: HWND, arg2: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CoMarshalInterface( pStm: LPSTREAM, riid: *const IID, pUnk: LPUNKNOWN, dwDestContext: DWORD, pvDestContext: LPVOID, mshlflags: DWORD, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoReleaseMarshalData(pStm: LPSTREAM) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoUnmarshalInterface(pStm: LPSTREAM, riid: *const IID, ppv: *mut LPVOID) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CreateFileMappingW( arg1: HANDLE, arg2: LPSECURITY_ATTRIBUTES, arg3: DWORD, arg4: DWORD, arg5: DWORD, arg6: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateStreamOnHGlobal( hGlobal: HGLOBAL, fDeleteOnRelease: BOOL, ppstm: *mut LPSTREAM, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DuplicateHandle( arg1: HANDLE, arg2: HANDLE, arg3: HANDLE, arg4: PHANDLE, arg5: DWORD, arg6: BOOL, arg7: DWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetAncestor(arg1: HWND, arg2: UINT) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetCurrentProcess() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetCurrentProcessId() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetGUIThreadInfo(arg1: DWORD, arg2: LPGUITHREADINFO) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetWindow(arg1: HWND, arg2: UINT) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetWindowLongW(arg1: HWND, arg2: ::core::ffi::c_int) -> LONG; // aclui-rust/src/types_i386.rs
    pub fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: PDWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GlobalAddAtomW(arg1: LPCWSTR) -> ATOM; // aclui-rust/src/types_i386.rs
    pub fn GlobalAlloc(arg1: UINT, arg2: SIZE_T) -> HGLOBAL; // aclui-rust/src/types_i386.rs
    pub fn GlobalDeleteAtom(arg1: ATOM) -> ATOM; // aclui-rust/src/types_i386.rs
    pub fn GlobalFree(arg1: HGLOBAL) -> HGLOBAL; // aclui-rust/src/types_i386.rs
    pub fn GlobalGetAtomNameW(nAtom: ATOM, lpBuffer: LPWSTR, nSize: ::core::ffi::c_int) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsChild(arg1: HWND, arg2: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsWindowVisible(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LoadStringA( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn LoadStringW( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn MapViewOfFile( arg1: HANDLE, arg2: DWORD, arg3: DWORD, arg4: DWORD, arg5: SIZE_T, ) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn MapWindowPoints( hWndFrom: HWND, hWndTo: HWND, lpPoints: LPPOINT, cPoints: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn OpenProcess(arg1: DWORD, arg2: BOOL, arg3: DWORD) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn PtInRect(arg1: LPCRECT, arg2: POINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RealGetWindowClassW(hwnd: HWND, ptszClassName: LPWSTR, cchClassNameMax: UINT) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn ScreenToClient(arg1: HWND, arg2: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn SysAllocString(arg1: *const OLECHAR) -> BSTR; // msctf-rust/src/types_i386.rs
    pub fn SysAllocStringLen(arg1: *const OLECHAR, ui: UINT) -> BSTR; // msctf-rust/src/types_i386.rs
    pub fn UnmapViewOfFile(arg1: LPCVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn VariantClear(arg1: *mut VARIANT) -> HRESULT; // msctf-rust/src/types_i386.rs
    pub fn VariantInit(arg1: *mut VARIANT); // msctf-rust/src/types_i386.rs
    pub fn WideCharToMultiByte( arg1: UINT, arg2: DWORD, arg3: LPCWSTR, arg4: ::core::ffi::c_int, arg5: LPSTR, arg6: ::core::ffi::c_int, arg7: LPCSTR, arg8: LPBOOL, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn WindowFromPoint(arg1: POINT) -> HWND; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::oleacc::main::AccessibleObjectFromWindow;
pub use crate::oleacc::window::__WINE_DBCL_FIXME;
pub use crate::oleacc::main::convert_child_id;
pub use crate::oleacc::window::create_window_object;
pub use crate::oleacc::main::find_class_data;
pub use crate::oleacc::propservice::get_accpropservices_factory;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbLengthW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // acppage-rust/src/prelude.rs
    pub fn ros_dbg_log(
        cls: __wine_debug_class,
        ch: *mut __wine_debug_channel,
        file: *const ::core::ffi::c_char,
        func: *const ::core::ffi::c_char,
        line: ::core::ffi::c_int,
        format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // client.c
}
pub use crate::prelude::_wcsicmp as wcsicmp; // sdk alias
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_oleacc: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x6f\x6c\x65\x61\x63\x63\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_oleacc };
// ==== end wine debug channels ====

// ==== port resolve: INITGUID storage (generated) ====
#[no_mangle]
pub static IIS_IsOleaccProxy: GUID = GUID { Data1: 0x902697fa, Data2: 0x80e4, Data3: 0x4560, Data4: [0x80, 0x2a, 0xa1, 0x3f, 0x22, 0xa6, 0x47, 0x09] };
// ==== end INITGUID storage ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! IsEqualIID { ($p0:expr, $p1:expr $(,)?) => { (((memcmp((($p0) as _), (($p1) as _), ((core::mem::size_of::<GUID>()) as _))) as isize == 0) as c_int) }; } // gen:aa85287a8771
#[macro_export] macro_rules! V_DISPATCH { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.pdispVal }; } // gen:6140ee7924c5
#[macro_export] macro_rules! V_I4 { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.lVal }; } // gen:2acba8bc60fa
#[macro_export] macro_rules! V_VT { ($p0:expr $(,)?) => { (*$p0).n1.n2.vt }; } // gen:bb0eb6ad6416
pub const CLASS_E_NOAGGREGATION: HRESULT = ((0x80040110u32) as HRESULT); // gen:01676cfea511
pub const E_ACCESSDENIED: HRESULT = ((0x80070005u32) as HRESULT); // gen:97ace9e06777
pub const E_FAIL: HRESULT = ((0x80004005u32) as HRESULT); // gen:1275c1cb9e8d
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT); // gen:cd8a52171603
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT); // gen:80daa5b2ffa6
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT); // gen:cbf314acc21c
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT); // gen:98bee37aa995
#[inline] // gen:e7d52f194fa4
pub unsafe fn IAccPropServices_QueryInterface(This: *mut IAccPropServices, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:38e6cbd5f3ba
pub unsafe fn IAccessible_AddRef(This: *mut IAccessible) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:a8dc866b6cc9
pub unsafe fn IAccessible_QueryInterface(This: *mut IAccessible, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:90a76ea53351
pub unsafe fn IAccessible_Release(This: *mut IAccessible) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:2c222bb95295
pub unsafe fn IAccessible_accHitTest(This: *mut IAccessible, left: LONG, top: LONG, pvarID: *mut VARIANT) -> HRESULT { ((*(*This).lpVtbl).accHitTest.unwrap_unchecked())(This, left, top, pvarID) } // widl C wrapper
#[inline] // gen:7da756ddc96f
pub unsafe fn IAccessible_accNavigate(This: *mut IAccessible, dir: LONG, varStart: VARIANT, pvarEnd: *mut VARIANT) -> HRESULT { ((*(*This).lpVtbl).accNavigate.unwrap_unchecked())(This, dir, varStart, pvarEnd) } // widl C wrapper
#[inline] // gen:b86f6d00f0fb
pub unsafe fn IAccessible_get_accChild(This: *mut IAccessible, varChildID: VARIANT, ppdispChild: *mut *mut IDispatch) -> HRESULT { ((*(*This).lpVtbl).get_accChild.unwrap_unchecked())(This, varChildID, ppdispChild) } // widl C wrapper
#[inline] // gen:63ea9cd3ad33
pub unsafe fn IAccessible_get_accChildCount(This: *mut IAccessible, pcountChildren: *mut LONG) -> HRESULT { ((*(*This).lpVtbl).get_accChildCount.unwrap_unchecked())(This, pcountChildren) } // widl C wrapper
#[inline] // gen:df123f3311ef
pub unsafe fn IAccessible_get_accParent(This: *mut IAccessible, ppdispParent: *mut *mut IDispatch) -> HRESULT { ((*(*This).lpVtbl).get_accParent.unwrap_unchecked())(This, ppdispParent) } // widl C wrapper
#[inline] // gen:cadbabb17eeb
pub unsafe fn IClassFactory_QueryInterface(This: *mut IClassFactory, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:87dc4bda7a0f
pub unsafe fn IDispatch_QueryInterface(This: *mut IDispatch, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:2dd34cc31a72
pub unsafe fn IDispatch_Release(This: *mut IDispatch) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:1acdbf888797
pub unsafe fn IEnumVARIANT_Next(This: *mut IEnumVARIANT, celt: ULONG, rgVar: *mut VARIANT, pCeltFetched: *mut ULONG) -> HRESULT { ((*(*This).lpVtbl).Next.unwrap_unchecked())(This, celt, rgVar, pCeltFetched) } // widl C wrapper
#[inline] // gen:d2a9357e309b
pub unsafe fn IEnumVARIANT_Release(This: *mut IEnumVARIANT) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:82f5169c4c8a
pub unsafe fn IEnumVARIANT_Reset(This: *mut IEnumVARIANT) -> HRESULT { ((*(*This).lpVtbl).Reset.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:44b82376cef5
pub unsafe fn IEnumVARIANT_Skip(This: *mut IEnumVARIANT, celt: ULONG) -> HRESULT { ((*(*This).lpVtbl).Skip.unwrap_unchecked())(This, celt) } // widl C wrapper
#[inline] // gen:c93df0b8635a
pub unsafe fn IOleWindow_GetWindow(This: *mut IOleWindow, phwnd: *mut HWND) -> HRESULT { ((*(*This).lpVtbl).GetWindow.unwrap_unchecked())(This, phwnd) } // widl C wrapper
#[inline] // gen:45b6eded6dcb
pub unsafe fn IOleWindow_Release(This: *mut IOleWindow) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:95899110c9d2
pub unsafe fn IServiceProvider_QueryService(This: *mut IServiceProvider, guidService: *const GUID, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryService.unwrap_unchecked())(This, guidService, riid, ppvObject) } // widl C wrapper
#[inline] // gen:8748b95f2e30
pub unsafe fn IServiceProvider_Release(This: *mut IServiceProvider) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:57df1036880d
pub unsafe fn IStream_Read(This: *mut IStream, pv: *mut ::core::ffi::c_void, cb: ULONG, pcbRead: *mut ULONG) -> HRESULT { ((*(*This).lpVtbl).Read.unwrap_unchecked())(This, pv, cb, pcbRead) } // widl C wrapper
#[inline] // gen:41d6f1edc6ec
pub unsafe fn IStream_Release(This: *mut IStream) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:74bd0263bdaf
pub unsafe fn IStream_Seek(This: *mut IStream, dlibMove: LARGE_INTEGER, dwOrigin: DWORD, plibNewPosition: *mut ULARGE_INTEGER) -> HRESULT { ((*(*This).lpVtbl).Seek.unwrap_unchecked())(This, dlibMove, dwOrigin, plibNewPosition) } // widl C wrapper
#[inline] // gen:3226c366765c
pub unsafe fn IStream_Stat(This: *mut IStream, pstatstg: *mut STATSTG, grfStatFlag: DWORD) -> HRESULT { ((*(*This).lpVtbl).Stat.unwrap_unchecked())(This, pstatstg, grfStatFlag) } // widl C wrapper
#[inline] // gen:6ad0b58c268c
pub unsafe fn IUnknown_AddRef(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
pub const OBJID_QUERYCLASSNAMEIDX: LONG = ((0xFFFFFFF4u32) as LONG); // gen:010847c266f2
pub type REFCLSID = *const CLSID; // gen:4a91e3a9efb2
pub type REFIID = *const IID; // gen:c79168ffaea6
pub const S_FALSE: HRESULT = ((1) as HRESULT); // gen:b1b8213e7dac
pub const S_OK: HRESULT = ((0) as HRESULT); // gen:9969a2c48b97
pub use crate::{IsEqualIID, V_DISPATCH, V_I4, V_VT};
// ==== end SDK header macros ====
