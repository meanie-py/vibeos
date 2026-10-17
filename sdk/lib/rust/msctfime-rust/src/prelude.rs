//! msctfime-rust prelude — the single import surface the translated modules use.
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
pub use crate::msctfime::cpp_classes::*;
pub use crate::msctfime::cpp_classes::{CAP_2700, CAP_ROT90, CAP_ROTANY, CAP_SOFTKBD, CActiveLanguageProfileNotifySink, CActiveLanguageProfileNotifySink__VTBL_own, CActiveLanguageProfileNotifySink__delete, CActiveLanguageProfileNotifySink__make_0, CActiveLanguageProfileNotifySink__new_0, CActiveLanguageProfileNotifySink__new_in_0, CActiveLanguageProfileNotifySink__vtbl, CActiveLanguageProfileNotifySink__zeroed, CCompButtonFrameWindow, CCompButtonFrameWindow__VTBL_b0, CCompButtonFrameWindow__delete, CCompButtonFrameWindow__make_0, CCompButtonFrameWindow__new_0, CCompButtonFrameWindow__new_in_0, CCompButtonFrameWindow__vtbl, CCompButtonFrameWindow__zeroed, CCompFinalizeButton, CCompFinalizeButton__VTBL_own, CCompFinalizeButton__delete, CCompFinalizeButton__make_0, CCompFinalizeButton__new_0, CCompFinalizeButton__new_in_0, CCompFinalizeButton__vtbl, CCompFinalizeButton__zeroed, CCompFrameWindow, CCompFrameWindow__VTBL_own, CCompFrameWindow__delete, CCompFrameWindow__make_0, CCompFrameWindow__new_0, CCompFrameWindow__new_in_0, CCompFrameWindow__vtbl, CCompFrameWindow__zeroed, CCompartmentEventSink, CCompartmentEventSink__VTBL_own, CCompartmentEventSink__delete, CCompartmentEventSink__make_0, CCompartmentEventSink__new_0, CCompartmentEventSink__new_in_0, CCompartmentEventSink__vtbl, CCompartmentEventSink__zeroed, CDefCompFrameGripper, CDefCompFrameGripper__VTBL_own, CDefCompFrameGripper__delete, CDefCompFrameGripper__make_0, CDefCompFrameGripper__new_0, CDefCompFrameGripper__new_in_0, CDefCompFrameGripper__vtbl, CDefCompFrameGripper__zeroed, CDefCompFrameWindow, CDefCompFrameWindow__VTBL_b0, CDefCompFrameWindow__delete, CDefCompFrameWindow__make_0, CDefCompFrameWindow__new_0, CDefCompFrameWindow__new_in_0, CDefCompFrameWindow__vtbl, CDefCompFrameWindow__zeroed, CESMAP__delete, CESMAP__make_0, CESMAP__new_0, CESMAP__new_in_0, CESMAP__zeroed, CFnDocFeed, CFnDocFeed__VTBL_own, CFnDocFeed__delete, CFnDocFeed__make_0, CFnDocFeed__new_0, CFnDocFeed__new_in_0, CFnDocFeed__vtbl, CFnDocFeed__zeroed, CFunctionProvider, CFunctionProviderBase, CFunctionProviderBase__VTBL_own, CFunctionProviderBase__delete, CFunctionProviderBase__make_0, CFunctionProviderBase__new_0, CFunctionProviderBase__new_in_0, CFunctionProviderBase__vtbl, CFunctionProviderBase__zeroed, CFunctionProvider__VTBL_b0, CFunctionProvider__delete, CFunctionProvider__make_0, CFunctionProvider__new_0, CFunctionProvider__new_in_0, CFunctionProvider__zeroed, CIMEUIWindowHandler, CIMEUIWindowHandler__delete, CIMEUIWindowHandler__make_0, CIMEUIWindowHandler__new_0, CIMEUIWindowHandler__new_in_0, CIMEUIWindowHandler__zeroed, CInputContextOwner, CInputContextOwner__ITfMouseTrackerACP_VTBL, CInputContextOwner__VTBL_own, CInputContextOwner__delete, CInputContextOwner__make_0, CInputContextOwner__new_0, CInputContextOwner__new_in_0, CInputContextOwner__vtbl, CInputContextOwner__zeroed, CModeBias, CModeBias__delete, CModeBias__make_0, CModeBias__new_0, CModeBias__new_in_0, CModeBias__zeroed, COMPWND, COMPWND__delete, COMPWND__make_0, COMPWND__new_0, COMPWND__new_in_0, COMPWND__zeroed, CPolyText, CPolyText__delete, CPolyText__make_0, CPolyText__new_0, CPolyText__new_in_0, CPolyText__zeroed, CTextEventSink, CTextEventSink__ITfTextLayoutSink_VTBL, CTextEventSink__VTBL_own, CTextEventSink__anon0__delete, CTextEventSink__anon0__make_0, CTextEventSink__anon0__new_0, CTextEventSink__anon0__new_in_0, CTextEventSink__anon0__zeroed, CTextEventSink__delete, CTextEventSink__make_0, CTextEventSink__new_0, CTextEventSink__new_in_0, CTextEventSink__vtbl, CTextEventSink__zeroed, CThreadMgrEventSink, CThreadMgrEventSink__VTBL_own, CThreadMgrEventSink__delete, CThreadMgrEventSink__make_0, CThreadMgrEventSink__new_0, CThreadMgrEventSink__new_in_0, CThreadMgrEventSink__vtbl, CThreadMgrEventSink__zeroed, CicBridge, CicBridge__VTBL_own, CicBridge__delete, CicBridge__make_0, CicBridge__new_0, CicBridge__new_in_0, CicBridge__vtbl, CicBridge__zeroed, CicInputContext, CicInputContext__ITfCompositionSink_VTBL, CicInputContext__ITfContextOwnerCompositionSink_VTBL, CicInputContext__VTBL_own, CicInputContext__delete, CicInputContext__make_0, CicInputContext__new_0, CicInputContext__new_in_0, CicInputContext__vtbl, CicInputContext__zeroed, CicProfile, CicProfile__VTBL_own, CicProfile__delete, CicProfile__make_0, CicProfile__new_0, CicProfile__new_in_0, CicProfile__vtbl, CicProfile__zeroed, ENUM_CREATE_DESTROY_IC, ENUM_CREATE_DESTROY_IC__delete, ENUM_CREATE_DESTROY_IC__make_0, ENUM_CREATE_DESTROY_IC__new_0, ENUM_CREATE_DESTROY_IC__new_in_0, ENUM_CREATE_DESTROY_IC__zeroed, FN_COMPARE, LANG_PROF_ENUM_ARG, MODEBIAS, MODEBIAS__delete, MODEBIAS__make_0, MODEBIAS__new_0, MODEBIAS__new_in_0, MODEBIAS__zeroed, MOUSE_SINK_ARGS, MOUSE_SINK_ARGS__delete, MOUSE_SINK_ARGS__make_0, MOUSE_SINK_ARGS__new_0, MOUSE_SINK_ARGS__new_in_0, MOUSE_SINK_ARGS__zeroed, PLANG_PROF_ENUM_ARG, TEXT_EVENT_SINK_END_EDIT, TEXT_EVENT_SINK_END_EDIT__delete, TEXT_EVENT_SINK_END_EDIT__make_0, TEXT_EVENT_SINK_END_EDIT__new_0, TEXT_EVENT_SINK_END_EDIT__new_in_0, TEXT_EVENT_SINK_END_EDIT__zeroed, TLS, TLS__delete, TLS__make_0, TLS__new_0, TLS__new_in_0, TLS__zeroed, UI, UIComposition, UIComposition__VTBL_own, UIComposition__delete, UIComposition__make_0, UIComposition__new_0, UIComposition__new_in_0, UIComposition__vtbl, UIComposition__zeroed, UI__VTBL_own, UI__delete, UI__make_0, UI__new_0, UI__new_in_0, UI__vtbl, UI__zeroed, _bindgen_ty_1, cpp_alloc, cpp_alloc_CicNoThrow, cpp_delete, cpp_delete_array, cpp_free, cpp_new, cpp_new_array, cpp_new_array_CicNoThrow, cpp_new_in, cpp_pure_virtual_call, g_ModeBiasMap, tagLANG_PROF_ENUM_ARG, tagLANG_PROF_ENUM_ARG__delete, tagLANG_PROF_ENUM_ARG__make_0, tagLANG_PROF_ENUM_ARG__new_0, tagLANG_PROF_ENUM_ARG__new_in_0, tagLANG_PROF_ENUM_ARG__zeroed};
// ==== end port cppapp ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1; // explorer-sys/src/lib.rs
pub const E_FAIL: HRESULT = 0x80004005u32 as HRESULT; // aclui-rust/src/prelude.rs
pub const E_INVALIDARG: HRESULT = 0x80070057u32 as HRESULT; // appwiz-rust/src/prelude.rs
pub const E_NOINTERFACE: HRESULT = 0x80004002u32 as HRESULT; // comctl32-rust/src/prelude.rs
pub const E_NOTIMPL: HRESULT = 0x80004001u32 as HRESULT; // appwiz-rust/src/prelude.rs
pub const E_OUTOFMEMORY: HRESULT = 0x8007000Eu32 as HRESULT; // comctl32-rust/src/prelude.rs
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as i32 as isize as HKEY; // access-rust/src/prelude.rs
pub const IDC_ARROW: LPCWSTR = 32512usize as LPCWSTR; // comctl32-rust/src/prelude.rs
pub const IDC_IBEAM: ULONG_PTR = 0x7F01; // comctl32-rust/src/prelude.rs
pub const LANGID_CHINESE_SIMPLIFIED: i32 = 0x804; // imm32-rust/src/prelude.rs
pub const LANGID_CHINESE_TRADITIONAL: i32 = 0x404; // imm32-rust/src/prelude.rs
pub const MK_ALT: DWORD = 0x20; // ole32-rust/src/win_consts.rs
pub const SECURITY_NT_AUTHORITY: i32 = 5; // imm32-rust/src/prelude.rs
pub const S_FALSE: HRESULT = 1; // comctl32-rust/src/prelude.rs
pub const S_OK: HRESULT = 0; // aclui-rust/src/prelude.rs
pub const TS_TRUE: THEMESIZE = THEMESIZE_TS_TRUE; // island enum
pub const VT_EMPTY: u16 = 0; // oleaut32-rust/src/consts.rs
pub const VT_I4: DWORD = 3; // shell32-rust/src/shell32/CDefView.rs
pub const VT_UNKNOWN: u16 = 13; // oleaut32-rust/src/consts.rs
extern "C" {
    pub fn wcstoul(s: *const u16, end: *mut *mut u16, base: c_int) -> ULONG; // kernel32-rust/src/prelude.rs
}
extern "system" {
    pub fn AllocateAndInitializeSid( arg1: PSID_IDENTIFIER_AUTHORITY, arg2: BYTE, arg3: DWORD, arg4: DWORD, arg5: DWORD, arg6: DWORD, arg7: DWORD, arg8: DWORD, arg9: DWORD, arg10: DWORD, arg11: *mut PSID, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CheckTokenMembership(arg1: HANDLE, arg2: PSID, arg3: PBOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DeleteCriticalSection(arg1: PCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn FreeSid(arg1: PSID) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_i386.rs
    pub fn InitializeCriticalSectionAndSpinCount(arg1: LPCRITICAL_SECTION, arg2: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn SetWindowTheme(arg1: HWND, arg2: LPCWSTR, arg3: LPCWSTR) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn SysAllocString(psz: *const u16) -> BSTR; // ole32-rust/src/ole32_exports.rs
    pub fn SysFreeString(bstr: BSTR); // oleaut32-rust/src/typelib.rs
    pub fn TlsAlloc() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn TlsFree(arg1: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TlsGetValue(arg1: DWORD) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn TlsSetValue(arg1: DWORD, arg2: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::msctfime::misc_cpp_h::ClearCompartment;
pub use crate::msctfime::misc_cpp_h::DllShutdownInProgress;
pub use crate::msctfime::misc_cpp_h::GetCompartment;
pub use crate::msctfime::misc_cpp_h::GetCompartmentDWORD;
pub use crate::msctfime::misc_cpp_h::InitDisplayAttrbuteLib;
pub use crate::msctfime::msctfime_cpp_h::Inquire;
pub use crate::msctfime::misc_cpp_h::IsEALang;
pub use crate::msctfime::misc_cpp_h::SetCompartmentDWORD;
pub use crate::msctfime::misc_cpp_h::UninitDisplayAttrbuteLib;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const LANGID_JAPANESE: i32 = 0x411; // clang, msctfime.cpp
pub const LANGID_KOREAN: i32 = 0x412; // clang, msctfime.cpp
pub const TF_LC_CHANGE: __WIDL_msctf_generated_name_0000001B = 1; // clang, sinks.cpp
pub const TF_LC_CREATE: __WIDL_msctf_generated_name_0000001B = 0; // clang, sinks.cpp
pub const TF_LC_DESTROY: __WIDL_msctf_generated_name_0000001B = 2; // clang, sinks.cpp
pub const UICOMP_GWLP_INDEX: i32 = 0; // clang, ui.cpp
pub const UICOMP_GWLP_SIZE: u32 = 4; // clang, ui.cpp
pub const UI_GWLP_HIMC: i32 = 0; // clang, ui.cpp
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn DefWindowProcW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
}
pub use crate::prelude::DefWindowProcW as DefWindowProc; // sdk alias
pub use crate::prelude::PostMessageW as PostMessage; // sdk alias
pub use crate::prelude::SendMessageW as SendMessage; // sdk alias
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
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

// ==== port resolve: INITGUID storage (generated) ====
#[no_mangle]
pub static CLSID_CAImmLayer: GUID = GUID { Data1: 0xB676DB87, Data2: 0x64DC, Data3: 0x4651, Data4: [0x99, 0xEC, 0x91, 0x07, 0x0E, 0xA4, 0x87, 0x90] };
#[no_mangle]
pub static GUID_COMPARTMENT_CTFIME_CICINPUTCONTEXT: GUID = GUID { Data1: 0x85A688F7, Data2: 0x6DC8, Data3: 0x4F17, Data4: [0xA8, 0x3A, 0xB1, 0x1C, 0x09, 0xCD, 0xD7, 0xBF] };
#[no_mangle]
pub static GUID_COMPARTMENT_CTFIME_DIMFLAGS: GUID = GUID { Data1: 0xA94C5FD2, Data2: 0xC471, Data3: 0x4031, Data4: [0x95, 0x46, 0x70, 0x9C, 0x17, 0x30, 0x0C, 0xB9] };
#[no_mangle]
pub static GUID_MODEBIAS_DEFAULT: GUID = GUID { Data1: 0xF3DA8BD4, Data2: 0x0786, Data3: 0x49C2, Data4: [0x8C, 0x09, 0x68, 0x39, 0xD8, 0xB8, 0x4F, 0x58] };
#[no_mangle]
pub static GUID_MODEBIAS_FILENAME: GUID = GUID { Data1: 0xD7F707FE, Data2: 0x44C6, Data3: 0x4FCA, Data4: [0x8E, 0x76, 0x86, 0xAB, 0x50, 0xC7, 0x93, 0x1B] };
#[no_mangle]
pub static GUID_MODEBIAS_NUMERIC: GUID = GUID { Data1: 0x4021766C, Data2: 0xE872, Data3: 0x48FD, Data4: [0x9C, 0xEE, 0x4E, 0xC5, 0xC7, 0x5E, 0x16, 0xC3] };
#[no_mangle]
pub static GUID_MODEBIAS_URLHISTORY: GUID = GUID { Data1: 0x8B0E54D9, Data2: 0x63F2, Data3: 0x4C68, Data4: [0x84, 0xD4, 0x79, 0xAE, 0xE7, 0xA5, 0x9F, 0x09] };
#[no_mangle]
pub static GUID_PROP_MODEBIAS: GUID = GUID { Data1: 0x372E0716, Data2: 0x974F, Data3: 0x40AC, Data4: [0xA0, 0x88, 0x08, 0xCD, 0xC9, 0x2E, 0xBF, 0xBC] };
#[no_mangle]
pub static GUID_TFCAT_DISPLAYATTRIBUTEPROPERTY: GUID = GUID { Data1: 0xB95F181B, Data2: 0xEA4C, Data3: 0x4AF1, Data4: [0x80, 0x56, 0x7C, 0x32, 0x1A, 0xBB, 0xB0, 0x91] };
#[no_mangle]
pub static IID_IAImmFnDocFeed: GUID = GUID { Data1: 0x6E098993, Data2: 0x9577, Data3: 0x499A, Data4: [0xA8, 0x30, 0x52, 0x34, 0x4F, 0x3E, 0x20, 0x0D] };
#[no_mangle]
pub static IID_ITfKeystrokeMgr_P: GUID = GUID { Data1: 0x53FA1BEC, Data2: 0x5BE1, Data3: 0x458E, Data4: [0xAE, 0x70, 0xA9, 0xF1, 0xDC, 0x84, 0x3E, 0x81] };
#[no_mangle]
pub static IID_ITfSysHookSink: GUID = GUID { Data1: 0x495388DA, Data2: 0x21A5, Data3: 0x4852, Data4: [0x8B, 0xB1, 0xED, 0x2F, 0x29, 0xDA, 0x8D, 0x60] };
#[no_mangle]
pub static IID_ITfThreadMgr_P: GUID = GUID { Data1: 0x7C6247A1, Data2: 0x2884, Data3: 0x4B7C, Data4: [0xAF, 0x24, 0xF1, 0x98, 0x04, 0x7A, 0xA7, 0x28] };
// ==== end INITGUID storage ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! IS_IME_HKL { ($p0:expr $(,)?) => { ((((($p0) as ULONG_PTR) & 0xF0000000) == 0xE0000000) as c_int) }; }
#[macro_export] macro_rules! V_I4 { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.lVal }; }
#[macro_export] macro_rules! V_UNKNOWN { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.punkVal }; }
#[macro_export] macro_rules! V_VT { ($p0:expr $(,)?) => { (*$p0).n1.n2.vt }; }
pub use crate::{IS_IME_HKL, V_I4, V_UNKNOWN, V_VT};
// ==== end SDK header macros ====

// REFIID/REFGUID: C++'s `const IID &`, a pointer at the ABI (as the island's vtables take it).
// resolve leaves them to a human: a dozen SDK headers each define them.
pub type REFGUID = *const GUID;
pub type REFIID = *const GUID;
// ui.cpp: `#define UI_GWLP_UI sizeof(HIMC)`, `UI_GWLP_SIZE (UI_GWLP_UI + sizeof(UI*))` --
// arch-dependent (resolve: i386 4/8, 64-bit 8/16), so from size_of, const on stable. `c_int`:
// they index Get/SetWindowLongPtr and size cbWndExtra, both int.
pub const UI_GWLP_UI: c_int = core::mem::size_of::<HIMC>() as c_int;
pub const UI_GWLP_SIZE: c_int = UI_GWLP_UI + core::mem::size_of::<*mut UI>() as c_int;
// msctfime.h: `#define GUID_MODEBIAS_NONE GUID_NULL` -- an alias, not an integer, so resolve
// leaves it. The island declares GUID_NULL (uuid.lib's storage).
pub use crate::types::GUID_NULL as GUID_MODEBIAS_NONE;
// guiddef.h: `#define IsEqualIID(riid1, riid2) IsEqualGUID(riid1, riid2)`. Hand-owned: the
// lowered form cast both arguments `as _`, which no call site can infer.
#[macro_export] macro_rules! IsEqualIID { ($a:expr, $b:expr $(,)?) => { IsEqualGUID!($a, $b) }; }
pub use crate::IsEqualIID;
// cicero/ciciface.h `CicInterface_RefCnt<T>` as profile.cpp uses it: the destructor releases
// whatever pointer the out-parameter call left in it, on every return path.
pub struct ComRelease(pub *mut IUnknown);
impl Drop for ComRelease {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { (*self.0).Release() };
        }
    }
}
// profile.cpp's local `CicEnumValue<IEnumTfLanguageProfiles, TF_LANGUAGEPROFILE>::DoEnum`, its
// only instantiation: Next() one profile at a time until the callback says S_OK.
pub unsafe fn CicEnumValue_DoEnum(
    iface: *mut IEnumTfLanguageProfiles,
    callback: crate::msctfime::profile_cpp_h::FN_LanguageProfilesCallback,
    lParam: LPARAM,
) -> DWORD {
    let mut data: TF_LANGUAGEPROFILE = core::mem::zeroed();
    loop {
        let hr: HRESULT = (*iface).Next(1, core::ptr::addr_of_mut!(data), core::ptr::null_mut());
        if hr != S_OK {
            break;
        }
        if callback(data, lParam) == S_OK {
            return ERROR_SUCCESS;
        }
    }
    ERROR_FILE_NOT_FOUND
}
// cicero/cictf.h: `struct ITfThreadMgr_P : ITfThreadMgr` adds six pure-virtual STDMETHODs. It is
// C++-only (no WIDL), so the island has just `{ _base: ITfThreadMgr }`; the C view of its
// table is ITfThreadMgr's slots followed by these, in declaration order.
#[repr(C)]
pub struct ITfThreadMgr_PVtbl {
    pub __base: ITfThreadMgrVtbl,
    pub GetAssociated: unsafe extern "system" fn(*mut ITfThreadMgr_P, HWND, *mut *mut ITfDocumentMgr) -> HRESULT,
    pub SetSysHookSink: unsafe extern "system" fn(*mut ITfThreadMgr_P, *mut ITfSysHookSink) -> HRESULT,
    pub RequestPostponedLock: unsafe extern "system" fn(*mut ITfThreadMgr_P, *mut ITfContext) -> HRESULT,
    pub IsKeystrokeFeedEnabled: unsafe extern "system" fn(*mut ITfThreadMgr_P, *mut c_int) -> HRESULT,
    pub CallImm32HotkeyHandler: unsafe extern "system" fn(*mut ITfThreadMgr_P, UINT, LPARAM, *mut HRESULT) -> HRESULT,
    pub ActivateEx: unsafe extern "system" fn(*mut ITfThreadMgr_P, *mut TfClientId, DWORD) -> HRESULT,
}
impl ITfThreadMgr_P {
    unsafe fn __p_vtbl(&self) -> *const ITfThreadMgr_PVtbl {
        self._base.lpVtbl as *const ITfThreadMgr_PVtbl
    }
    pub unsafe fn GetAssociated(&mut self, hwnd: HWND, ppDocMgr: *mut *mut ITfDocumentMgr) -> HRESULT {
        ((*self.__p_vtbl()).GetAssociated)(self, hwnd, ppDocMgr)
    }
    pub unsafe fn SetSysHookSink(&mut self, pSysHookSink: *mut ITfSysHookSink) -> HRESULT {
        ((*self.__p_vtbl()).SetSysHookSink)(self, pSysHookSink)
    }
    pub unsafe fn RequestPostponedLock(&mut self, pContext: *mut ITfContext) -> HRESULT {
        ((*self.__p_vtbl()).RequestPostponedLock)(self, pContext)
    }
    pub unsafe fn IsKeystrokeFeedEnabled(&mut self, p: *mut c_int) -> HRESULT {
        ((*self.__p_vtbl()).IsKeystrokeFeedEnabled)(self, p)
    }
    pub unsafe fn CallImm32HotkeyHandler(&mut self, vKey: UINT, lParam: LPARAM, phrResult: *mut HRESULT) -> HRESULT {
        ((*self.__p_vtbl()).CallImm32HotkeyHandler)(self, vKey, lParam, phrResult)
    }
    pub unsafe fn ActivateEx(&mut self, ptid: *mut TfClientId, dwFlags: DWORD) -> HRESULT {
        ((*self.__p_vtbl()).ActivateEx)(self, ptid, dwFlags)
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

// ==== port resolve: path-dependency definitions (generated) ====
pub use cicero_rust::cicero::cicbase_h::{cicGetFN};
pub use cicero_rust::cicero::cicbase_h_h::{cicMemAllocClear, cicMemFree, cicMemReAlloc};
pub use cicero_rust::cicero::cicimc_t::{CicIMCCLock__make_0, CicIMCCLock__zeroed};
pub use cicero_rust::cicero::cicuif_cpp_h::{cicDoneUIFLib, cicInitUIFLib};
pub use cicero_rust::cicero::cpp_classes::{CUIFTheme__zeroed, CicIMCLock__make_0, CicIMCLock__zeroed, CicRegKey__zeroed};
// ==== end path-dependency definitions ====

// winuser.h on i386: `#define GetWindowLongPtrW GetWindowLongW` (and Set..), so user32 exports
// no *LongPtrW there and the i386 island has no declaration; the 64-bit islands declare the
// real functions. LONG_PTR is LONG on i386, so the signatures are the C's own.
#[cfg(target_arch = "x86")]
extern "system" {
    #[link_name = "GetWindowLongW"]
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: c_int) -> LONG_PTR;
    #[link_name = "SetWindowLongW"]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: c_int, dwNewLong: LONG_PTR) -> LONG_PTR;
}
