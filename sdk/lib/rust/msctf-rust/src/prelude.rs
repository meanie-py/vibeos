//! msctf-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// The wine/debug.h channel (TRACE/ERR/WARN/FIXME). Kept as no-op MACROS, never
// out-of-line fns: an out-of-line no-op is an optimization barrier that uncloaks latent
// ABI mismatches into ud2 (field manual §9.65).
#[macro_export] macro_rules! TRACE { ($($arg:tt)*) => {{}}; }
// ERR/FIXME print in this DBG=1 tree, as every other wine crate's do (legide, ole32).
#[macro_export] macro_rules! ERR   { ($($arg:tt)*) => { ::win32macros_rust::DPRINT1!($($arg)*) }; }
#[macro_export] macro_rules! WARN  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! FIXME { ($($arg:tt)*) => { ::win32macros_rust::DPRINT1!($($arg)*) }; }
pub use crate::{ERR, FIXME, TRACE, WARN};

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

// ================= msctf's own macros (hand-written) ======================================
/// `SINK_FOR_EACH(cursor, list, type, elem)` (precomp.h), as every translated site spells it:
/// `SINK_FOR_EACH!(cursor, list, Iface, elem, { body })`. The C pastes `interfaces.p##type`;
/// every member of that union is a pointer at offset 0, so `pIUnknown` cast to the interface
/// is the same value. The cursor advances at the TOP, so a `continue` in the body steps on, as
/// the C `for`'s increment does. (The C also loads `elem` from the list HEAD before the first
/// test, a read of memory that is not a Sink; this reads only real entries.)
#[macro_export]
macro_rules! SINK_FOR_EACH {
    ($cursor:ident, $list:expr, $ty:ident, $elem:ident, $body:block) => {{
        let __list: *mut list = $list;
        $cursor = __list;
        loop {
            $cursor = (*$cursor).next;
            if $cursor == __list {
                break;
            }
            #[allow(unused_assignments)]
            {
                $elem = (*$crate::prelude::CONTAINING_RECORD!($cursor, Sink, entry)).interfaces.pIUnknown as *mut $ty;
            }
            $body
        }
    }};
}
pub use crate::SINK_FOR_EACH;

// REFIID/REFCLSID/REFGUID: C++'s `const IID &`, a pointer at the ABI (as the island's vtables
// take it). resolve leaves them to a human: a dozen SDK headers each define them.
pub type REFIID = *const GUID;
pub type REFCLSID = *const GUID;
pub type REFGUID = *const GUID;

// ==== port cppapp: the module's C++ classes (generated -- `rustconv port cppapp`) ====
pub use crate::msctf::cpp_classes::*;
pub use crate::msctf::cpp_classes::{CCategoryMgr, CCategoryMgr__VTBL_own, CCategoryMgr__delete, CCategoryMgr__make_0, CCategoryMgr__new_0, CCategoryMgr__new_in_0, CCategoryMgr__vtbl, CCategoryMgr__zeroed, CClassFactory, CClassFactory__VTBL_own, CClassFactory__delete, CClassFactory__make_0, CClassFactory__new_0, CClassFactory__new_in_0, CClassFactory__vtbl, CClassFactory__zeroed, CCompartment, CCompartmentEnumGuid, CCompartmentEnumGuid__VTBL_own, CCompartmentEnumGuid__delete, CCompartmentEnumGuid__make_0, CCompartmentEnumGuid__new_0, CCompartmentEnumGuid__new_in_0, CCompartmentEnumGuid__vtbl, CCompartmentEnumGuid__zeroed, CCompartmentMgr, CCompartmentMgr__VTBL_own, CCompartmentMgr__delete, CCompartmentMgr__make_0, CCompartmentMgr__new_0, CCompartmentMgr__new_in_0, CCompartmentMgr__vtbl, CCompartmentMgr__zeroed, CCompartment__ITfSource_VTBL, CCompartment__VTBL_own, CCompartment__delete, CCompartment__make_0, CCompartment__new_0, CCompartment__new_in_0, CCompartment__vtbl, CCompartment__zeroed, CContext, CContext__ITextStoreACPServices_VTBL, CContext__ITextStoreACPSink_VTBL, CContext__ITfContextOwnerCompositionServices_VTBL, CContext__ITfInsertAtSelection_VTBL, CContext__ITfSourceSingle_VTBL, CContext__ITfSource_VTBL, CContext__VTBL_own, CContext__delete, CContext__make_0, CContext__new_0, CContext__new_in_0, CContext__vtbl, CContext__zeroed, CDisplayAttributeMgr, CDisplayAttributeMgr__VTBL_own, CDisplayAttributeMgr__delete, CDisplayAttributeMgr__make_0, CDisplayAttributeMgr__new_0, CDisplayAttributeMgr__new_in_0, CDisplayAttributeMgr__vtbl, CDisplayAttributeMgr__zeroed, CDocumentMgr, CDocumentMgr__ITfSource_VTBL, CDocumentMgr__VTBL_own, CDocumentMgr__delete, CDocumentMgr__make_0, CDocumentMgr__new_0, CDocumentMgr__new_in_0, CDocumentMgr__vtbl, CDocumentMgr__zeroed, CEnumTfContext, CEnumTfContext__VTBL_own, CEnumTfContext__delete, CEnumTfContext__make_0, CEnumTfContext__new_0, CEnumTfContext__new_in_0, CEnumTfContext__vtbl, CEnumTfContext__zeroed, CEnumTfDocumentMgr, CEnumTfDocumentMgr__VTBL_own, CEnumTfDocumentMgr__delete, CEnumTfDocumentMgr__make_0, CEnumTfDocumentMgr__new_0, CEnumTfDocumentMgr__new_in_0, CEnumTfDocumentMgr__vtbl, CEnumTfDocumentMgr__zeroed, CEnumTfInputProcessorProfiles, CEnumTfInputProcessorProfiles__VTBL_own, CEnumTfInputProcessorProfiles__delete, CEnumTfInputProcessorProfiles__make_0, CEnumTfInputProcessorProfiles__new_0, CEnumTfInputProcessorProfiles__new_in_0, CEnumTfInputProcessorProfiles__vtbl, CEnumTfInputProcessorProfiles__zeroed, CEnumTfLanguageProfiles, CEnumTfLanguageProfiles__VTBL_own, CEnumTfLanguageProfiles__delete, CEnumTfLanguageProfiles__make_0, CEnumTfLanguageProfiles__new_0, CEnumTfLanguageProfiles__new_in_0, CEnumTfLanguageProfiles__vtbl, CEnumTfLanguageProfiles__zeroed, CInputProcessorProfiles, CInputProcessorProfiles__ITfInputProcessorProfileMgr_VTBL, CInputProcessorProfiles__ITfSource_VTBL, CInputProcessorProfiles__VTBL_own, CInputProcessorProfiles__delete, CInputProcessorProfiles__make_0, CInputProcessorProfiles__new_0, CInputProcessorProfiles__new_in_0, CInputProcessorProfiles__vtbl, CInputProcessorProfiles__zeroed, CLangBarMgr, CLangBarMgr__VTBL_own, CLangBarMgr__delete, CLangBarMgr__make_0, CLangBarMgr__new_0, CLangBarMgr__new_in_0, CLangBarMgr__vtbl, CLangBarMgr__zeroed, CProfilesEnumGuid, CProfilesEnumGuid__VTBL_own, CProfilesEnumGuid__delete, CProfilesEnumGuid__make_0, CProfilesEnumGuid__new_0, CProfilesEnumGuid__new_in_0, CProfilesEnumGuid__vtbl, CProfilesEnumGuid__zeroed, CRange, CRange__ITfSource_VTBL, CRange__VTBL_own, CRange__delete, CRange__make_0, CRange__new_0, CRange__new_in_0, CRange__vtbl, CRange__zeroed, CStaticIconList, CStaticIconList__delete, CStaticIconList__make_0, CStaticIconList__new_0, CStaticIconList__new_in_0, CStaticIconList__zeroed, CThreadMgr, CThreadMgr__ITfClientId_VTBL, CThreadMgr__ITfKeystrokeMgr_VTBL, CThreadMgr__ITfMessagePump_VTBL, CThreadMgr__ITfSourceSingle_VTBL, CThreadMgr__ITfSource_VTBL, CThreadMgr__ITfThreadMgrEventSink_VTBL, CThreadMgr__ITfUIElementMgr_VTBL, CThreadMgr__VTBL_own, CThreadMgr__delete, CThreadMgr__make_0, CThreadMgr__new_0, CThreadMgr__new_in_0, CThreadMgr__vtbl, CThreadMgr__zeroed, MLNGINFO, MLNGINFO__delete, MLNGINFO__make_0, MLNGINFO__new_0, MLNGINFO__new_in_0, MLNGINFO__zeroed, cpp_alloc, cpp_alloc_CicNoThrow, cpp_delete, cpp_delete_array, cpp_free, cpp_new, cpp_new_array, cpp_new_array_CicNoThrow, cpp_new_in, cpp_pure_virtual_call, g_IconList, tagAssociatedWindow, tagAssociatedWindow__delete, tagAssociatedWindow__make_0, tagAssociatedWindow__new_0, tagAssociatedWindow__new_in_0, tagAssociatedWindow__zeroed, tagCompartmentValue, tagCompartmentValue__delete, tagCompartmentValue__make_0, tagCompartmentValue__new_0, tagCompartmentValue__new_in_0, tagCompartmentValue__zeroed, tagDocumentMgrs, tagDocumentMgrs__delete, tagDocumentMgrs__make_0, tagDocumentMgrs__new_0, tagDocumentMgrs__new_in_0, tagDocumentMgrs__zeroed, tagEditCookie, tagEditCookie__delete, tagEditCookie__make_0, tagEditCookie__new_0, tagEditCookie__new_in_0, tagEditCookie__zeroed, tagPreservedKey, tagPreservedKey__delete, tagPreservedKey__make_0, tagPreservedKey__new_0, tagPreservedKey__new_in_0, tagPreservedKey__zeroed};
// ==== end port cppapp ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const ANSI_NULL: CHAR = 0; // imm32-rust/src/prelude.rs
pub const CLASS_E_CLASSNOTAVAILABLE: HRESULT = 0x8004_0111u32 as HRESULT; // netcfgx-rust/src/prelude.rs
pub const CLASS_E_NOAGGREGATION: HRESULT = 0x80040110u32 as HRESULT; // comctl32-rust/src/prelude.rs
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1; // explorer-sys/src/lib.rs
pub const E_FAIL: HRESULT = 0x80004005u32 as HRESULT; // aclui-rust/src/prelude.rs
pub const E_INVALIDARG: HRESULT = 0x80070057u32 as HRESULT; // appwiz-rust/src/prelude.rs
pub const E_NOINTERFACE: HRESULT = 0x80004002u32 as HRESULT; // comctl32-rust/src/prelude.rs
pub const E_NOTIMPL: HRESULT = 0x80004001u32 as HRESULT; // appwiz-rust/src/prelude.rs
pub const E_OUTOFMEMORY: HRESULT = 0x8007000Eu32 as HRESULT; // comctl32-rust/src/prelude.rs
pub const E_POINTER: HRESULT = 0x80004003u32 as HRESULT; // ole32-rust/src/hresults.rs
pub const E_UNEXPECTED: HRESULT = 0x8000_FFFFu32 as HRESULT; // netcfgx-rust/src/prelude.rs
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as i32 as isize as HKEY; // access-rust/src/prelude.rs
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY; // console-rust/src/prelude.rs
pub const S_FALSE: HRESULT = 1; // comctl32-rust/src/prelude.rs
pub const S_OK: HRESULT = 0; // aclui-rust/src/prelude.rs
pub const VT_BSTR: DWORD = 8; // shell32-rust/src/shell32/CFolderItem.rs
pub const VT_EMPTY: u16 = 0; // oleaut32-rust/src/consts.rs
pub const VT_I4: DWORD = 3; // shell32-rust/src/shell32/CDefView.rs
pub const VT_UNKNOWN: u16 = 13; // oleaut32-rust/src/consts.rs
extern "C" {
    pub fn wcstoul(s: *const u16, end: *mut *mut u16, base: c_int) -> ULONG; // kernel32-rust/src/prelude.rs
}
extern "system" {
    pub fn CLSIDFromString(arg1: LPCOLESTR, arg2: LPCLSID) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoCreateGuid(pguid: *mut GUID) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoCreateInstance( rclsid: *const CLSID, pUnkOuter: LPUNKNOWN, dwClsContext: DWORD, iid: *const IID, ppv: *mut LPVOID, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemAlloc(size: SIZE_T) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn DebugBreak(); // aclui-rust/src/types_i386.rs
    pub fn DeleteCriticalSection(arg1: PCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn GetCurrentProcess() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetCurrentThreadId() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetTokenInformation( arg1: HANDLE, arg2: TOKEN_INFORMATION_CLASS, arg3: PVOID, arg4: DWORD, arg5: PDWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn InitializeCriticalSectionAndSpinCount(arg1: LPCRITICAL_SECTION, arg2: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn OpenProcessToken(arg1: HANDLE, arg2: DWORD, arg3: PHANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegCreateKeyExW( hKey: HKEY, lpSubKey: LPCWSTR, Reserved: DWORD, lpClass: LPWSTR, dwOptions: DWORD, samDesired: REGSAM, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, phkResult: PHKEY, lpdwDisposition: PDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegDeleteTreeW(arg1: HKEY, arg2: LPCWSTR) -> LSTATUS; // shell32-rust/src/island_i386.rs
    pub fn RegEnumKeyExW( hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, lpcchName: LPDWORD, lpReserved: LPDWORD, lpClass: LPWSTR, lpcchClass: LPDWORD, lpftLastWriteTime: PFILETIME, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegEnumKeyW(hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, cchName: DWORD) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegSetValueExW( hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn StringFromGUID2(id: *const GUID, str_: LPOLESTR, cmax: INT) -> INT; // aclui-rust/src/types_i386.rs
    pub fn TlsAlloc() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn TlsFree(arg1: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TlsGetValue(arg1: DWORD) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn TlsSetValue(arg1: DWORD, arg2: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn lstrcmpiA(arg1: LPCSTR, arg2: LPCSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn lstrcpynW(arg1: LPWSTR, arg2: LPCWSTR, arg3: ::core::ffi::c_int) -> LPWSTR; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const CONNECT_E_ADVISELIMIT: i32 = 0x80040201_u32 as i32; // clang, threadmgr.cpp
pub const CONNECT_E_CANNOTCONNECT: i32 = 0x80040202_u32 as i32; // clang, msctf.cpp
pub const CONNECT_E_NOCONNECTION: i32 = 0x80040200_u32 as i32; // clang, compartmentmgr.cpp
pub const IDC_SIZENESW: u32 = 0x7F83; // clang, categorymgr.cpp
pub const IDC_SIZENWSE: u32 = 0x7F82; // clang, categorymgr.cpp
pub const TF_ANCHOR_END: __WIDL_msctf_generated_name_00000014 = 1; // clang, range.cpp
pub const TF_E_ALREADY_EXISTS: i32 = 0x80040506_u32 as i32; // clang, threadmgr.cpp
pub const TF_E_DISCONNECTED: i32 = 0x80040504_u32 as i32; // clang, context.cpp
pub const TF_E_NOLOCK: i32 = 0x80040201_u32 as i32; // clang, context.cpp
pub const TF_E_STACKFULL: i32 = 0x80040501_u32 as i32; // clang, documentmgr.cpp
pub const TF_SFT_EXTRA_ICON_GROUP: i32 = 0x600; // clang, langbarmgr.cpp
pub const TF_SFT_LABEL_GROUP: i32 = 0x180; // clang, langbarmgr.cpp
pub const TF_SFT_TRANSPARENCY_GROUP: i32 = 0x70; // clang, langbarmgr.cpp
pub const TF_SFT_VISIBILITY_GROUP: i32 = 0x80F; // clang, langbarmgr.cpp
pub const TS_E_NOLOCK: i32 = 0x80040201_u32 as i32; // clang, context.cpp
pub const TS_E_READONLY: i32 = 0x80040209_u32 as i32; // clang, context.cpp
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn ConvertSidToStringSidW(arg1: PSID, arg2: *mut LPWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateProcessW( arg1: LPCWSTR, arg2: LPWSTR, arg3: LPSECURITY_ATTRIBUTES, arg4: LPSECURITY_ATTRIBUTES, arg5: BOOL, arg6: DWORD, arg7: PVOID, arg8: LPCWSTR, arg9: LPSTARTUPINFOW, arg10: LPPROCESS_INFORMATION, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetUserObjectInformationW( hObj: HANDLE, nIndex: ::core::ffi::c_int, pvInfo: PVOID, nLength: DWORD, lpnLengthNeeded: LPDWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn OpenMutexW(arg1: DWORD, arg2: BOOL, arg3: LPCWSTR) -> HANDLE; // aclui-rust/src/types_i386.rs
}
pub use crate::prelude::ConvertSidToStringSidW as ConvertSidToStringSid; // sdk alias
pub use crate::prelude::CreateProcessW as CreateProcess; // sdk alias
pub use crate::prelude::GetUserObjectInformationW as GetUserObjectInformation; // sdk alias
pub use crate::prelude::OpenMutexW as OpenMutex; // sdk alias
pub use crate::prelude::RegOpenKeyExW as RegOpenKeyEx; // sdk alias
pub use crate::prelude::RegQueryValueExW as RegQueryValueEx; // sdk alias
pub use strsafe_rust::StringCchCatW as StringCchCat; // sdk alias
pub use strsafe_rust::StringCchCopyW as StringCchCopy; // sdk alias
pub use strsafe_rust::StringCchPrintfW as StringCchPrintf; // sdk alias
pub use crate::prelude::StringFromGUID2 as StringFromGUID2T; // sdk alias
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
pub type STARTUPINFO = STARTUPINFOW;
// ==== end UNICODE TCHAR family ====

// ==== port resolve: INITGUID storage (generated) ====
#[no_mangle]
pub static IID_CDisplayAttributeMgr: GUID = GUID { Data1: 0xFF4619E8, Data2: 0xEA5E, Data3: 0x43E5, Data4: [0xB3, 0x08, 0x11, 0xCD, 0x26, 0xAB, 0x6B, 0x3A] };
#[no_mangle]
pub static IID_ITfDisplayAttributeCollectionMgr: GUID = GUID { Data1: 0x4e3d2d48, Data2: 0x3c17, Data3: 0x457d, Data4: [0x84, 0xa1, 0xf2, 0x09, 0x47, 0x6d, 0xe8, 0x97] };
#[no_mangle]
pub static IID_ITfLangBarMgr_P: GUID = GUID { Data1: 0xD72C0FA9, Data2: 0xADD5, Data3: 0x4AF0, Data4: [0x87, 0x06, 0x4F, 0xA9, 0xAE, 0x3E, 0x2E, 0xFF] };
#[no_mangle]
pub static IID_ITfRangeAnchor: GUID = GUID { Data1: 0x8b99712b, Data2: 0x5815, Data3: 0x4bcc, Data4: [0xb9, 0xa9, 0x53, 0xdb, 0x1c, 0x8d, 0x67, 0x55] };
#[no_mangle]
pub static IID_PRIV_CRANGE: GUID = GUID { Data1: 0xB68832F0, Data2: 0x34B9, Data3: 0x11D3, Data4: [0xA7, 0x45, 0x00, 0x50, 0x04, 0x0A, 0xB4, 0x07] };
// ==== end INITGUID storage ====
// ==== port resolve: path-dependency definitions (generated) ====
pub use cicero_rust::cicero::cicbase_cpp_h::{cicMemReCalloc};
pub use cicero_rust::cicero::cicbase_h::{cicNoThrow};
pub use cicero_rust::cicero::cicbase_h_h::{cicMemAlloc, cicMemAllocClear, cicMemFree};
// ==== end path-dependency definitions ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! IS_IME_HKL { ($p0:expr $(,)?) => { ((((((($p0) as ULONG_PTR) & 0xF0000000) == 0xE0000000) as c_int)) != 0) }; }
#[macro_export] macro_rules! IS_SPECIAL_HKL { ($p0:expr $(,)?) => { ((((((($p0) as ULONG_PTR) & 0xF0000000) == 0xF0000000) as c_int)) != 0) }; }
#[macro_export] macro_rules! IsEqualIID { ($p0:expr, $p1:expr $(,)?) => { IsEqualGUID!(($p0), ($p1)) }; }
#[macro_export] macro_rules! SPECIALIDFROMHKL { ($p0:expr $(,)?) => { ((((((((((($p0) as DWORD_PTR) >> 16) & 0xffff)) as WORD)) as c_int) & 0x0FFF)) as WORD) }; }
#[macro_export] macro_rules! V_BSTR { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.bstrVal }; }
#[macro_export] macro_rules! V_UNKNOWN { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.punkVal }; }
#[macro_export] macro_rules! V_VT { ($p0:expr $(,)?) => { (*$p0).n1.n2.vt }; }
pub use crate::{IS_IME_HKL, IS_SPECIAL_HKL, IsEqualIID, SPECIALIDFROMHKL, V_BSTR, V_UNKNOWN, V_VT};
// ==== end SDK header macros ====

// cicero/cicbase.h `static inline bool cicIsNullPtr(LPCVOID ptr) { return !ptr; }` -- cicero's own
// translation is private to it. Generic over the pointee: every caller hands a typed pointer.
#[inline]
pub fn cicIsNullPtr<T>(ptr: *const T) -> bool {
    ptr.is_null()
}

extern "C" {
    // msctf.idl's ITfLangBarMgr IID: uuid.lib's storage, which the island does not declare.
    pub static IID_ITfLangBarMgr: IID;
}

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::msctf::compartmentmgr_cpp_h::CompartmentValue;
pub use crate::msctf::msctf_cpp_h::LPFNCONSTRUCTOR;
// ==== end cross-TU definitions ====
