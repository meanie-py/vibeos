//! ntobjshex-rust prelude — the single import surface the translated modules use.
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
pub use atl_rust::{ShellObjectCreatorIid, ShellObjectCreatorInit};
pub use atl_rust::shellutils::DumpIdList;
pub use atl_rust::com::ShellObjectCreator;

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
pub use crate::ntobjshex::cpp_classes::*;
pub use crate::ntobjshex::cpp_classes::{CCommonFolder_CNtObjectFolder_NtPidlEntry_CNtObjectFolderExtractIcon, CCommonFolder_CNtObjectFolder_NtPidlEntry_CNtObjectFolderExtractIcon__IPersistFolder2_VTBL, CCommonFolder_CNtObjectFolder_NtPidlEntry_CNtObjectFolderExtractIcon__IShellFolder2_VTBL, CCommonFolder_CNtObjectFolder_NtPidlEntry_CNtObjectFolderExtractIcon__VTBL_own, CCommonFolder_CNtObjectFolder_NtPidlEntry_CNtObjectFolderExtractIcon__delete, CCommonFolder_CNtObjectFolder_NtPidlEntry_CNtObjectFolderExtractIcon__make_0, CCommonFolder_CNtObjectFolder_NtPidlEntry_CNtObjectFolderExtractIcon__new_0, CCommonFolder_CNtObjectFolder_NtPidlEntry_CNtObjectFolderExtractIcon__new_in_0, CCommonFolder_CNtObjectFolder_NtPidlEntry_CNtObjectFolderExtractIcon__vtbl, CCommonFolder_CNtObjectFolder_NtPidlEntry_CNtObjectFolderExtractIcon__zeroed, CCommonFolder_CRegistryFolder_RegPidlEntry_CRegistryFolderExtractIcon, CCommonFolder_CRegistryFolder_RegPidlEntry_CRegistryFolderExtractIcon__IPersistFolder2_VTBL, CCommonFolder_CRegistryFolder_RegPidlEntry_CRegistryFolderExtractIcon__IShellFolder2_VTBL, CCommonFolder_CRegistryFolder_RegPidlEntry_CRegistryFolderExtractIcon__VTBL_own, CCommonFolder_CRegistryFolder_RegPidlEntry_CRegistryFolderExtractIcon__delete, CCommonFolder_CRegistryFolder_RegPidlEntry_CRegistryFolderExtractIcon__make_0, CCommonFolder_CRegistryFolder_RegPidlEntry_CRegistryFolderExtractIcon__new_0, CCommonFolder_CRegistryFolder_RegPidlEntry_CRegistryFolderExtractIcon__new_in_0, CCommonFolder_CRegistryFolder_RegPidlEntry_CRegistryFolderExtractIcon__vtbl, CCommonFolder_CRegistryFolder_RegPidlEntry_CRegistryFolderExtractIcon__zeroed, CEnumNTDirectory, CEnumNTDirectory__IEnumIDList_VTBL, CEnumNTDirectory__delete, CEnumNTDirectory__make_0, CEnumNTDirectory__new_0, CEnumNTDirectory__new_in_0, CEnumNTDirectory__zeroed, CEnumRegKey, CEnumRegKey__IEnumIDList_VTBL, CEnumRegKey__delete, CEnumRegKey__make_0, CEnumRegKey__new_0, CEnumRegKey__new_in_0, CEnumRegKey__zeroed, CEnumRegRoot, CEnumRegRoot__IEnumIDList_VTBL, CEnumRegRoot__delete, CEnumRegRoot__make_0, CEnumRegRoot__new_0, CEnumRegRoot__new_in_0, CEnumRegRoot__zeroed, CFolderViewCB, CFolderViewCB__IShellFolderViewCB_VTBL, CFolderViewCB__VTBL_own, CFolderViewCB__delete, CFolderViewCB__make_0, CFolderViewCB__new_0, CFolderViewCB__new_in_0, CFolderViewCB__vtbl, CFolderViewCB__zeroed, CNtObjectFolder, CNtObjectFolderExtractIcon, CNtObjectFolderExtractIcon__IExtractIconW_VTBL, CNtObjectFolderExtractIcon__delete, CNtObjectFolderExtractIcon__make_0, CNtObjectFolderExtractIcon__new_0, CNtObjectFolderExtractIcon__new_in_0, CNtObjectFolderExtractIcon__zeroed, CNtObjectFolder__VTBL_b1, CNtObjectFolder__VTBL_own, CNtObjectFolder____base1___base1_IShellFolder2_VTBL, CNtObjectFolder____base1___base2_IPersistFolder2_VTBL, CNtObjectFolder__delete, CNtObjectFolder__make_0, CNtObjectFolder__new_0, CNtObjectFolder__new_in_0, CNtObjectFolder__vtbl, CNtObjectFolder__zeroed, CRegistryFolder, CRegistryFolderExtractIcon, CRegistryFolderExtractIcon__IExtractIconW_VTBL, CRegistryFolderExtractIcon__delete, CRegistryFolderExtractIcon__make_0, CRegistryFolderExtractIcon__new_0, CRegistryFolderExtractIcon__new_in_0, CRegistryFolderExtractIcon__zeroed, CRegistryFolder__VTBL_b1, CRegistryFolder__VTBL_own, CRegistryFolder____base1___base1_IShellFolder2_VTBL, CRegistryFolder____base1___base2_IPersistFolder2_VTBL, CRegistryFolder__delete, CRegistryFolder__make_0, CRegistryFolder__new_0, CRegistryFolder__new_in_0, CRegistryFolder__vtbl, CRegistryFolder__zeroed, NTOBJECT_COLUMN_END, NTOBJECT_COLUMN_LINKTARGET, NTOBJECT_COLUMN_NAME, NTOBJECT_COLUMN_TYPE, NtObjectColumns, NtPidlEntry__delete, NtPidlEntry__make_0, NtPidlEntry__new_0, NtPidlEntry__new_in_0, NtPidlEntry__zeroed, NtPidlTypeData__delete, NtPidlTypeData__make_0, NtPidlTypeData__new_0, NtPidlTypeData__new_in_0, NtPidlTypeData__zeroed, ObjectMap, ObjectTypeNames, REGISTRY_COLUMN_END, REGISTRY_COLUMN_NAME, REGISTRY_COLUMN_TYPE, REGISTRY_COLUMN_VALUE, RegPidlEntry__anon0__anon0, RegPidlEntry__anon0__anon0__delete, RegPidlEntry__anon0__anon0__make_0, RegPidlEntry__anon0__anon0__new_0, RegPidlEntry__anon0__anon0__new_in_0, RegPidlEntry__anon0__anon0__zeroed, RegPidlEntry__anon0__delete, RegPidlEntry__anon0__make_0, RegPidlEntry__anon0__new_0, RegPidlEntry__anon0__new_in_0, RegPidlEntry__anon0__zeroed, RegPidlEntry__delete, RegPidlEntry__make_0, RegPidlEntry__new_0, RegPidlEntry__new_in_0, RegPidlEntry__zeroed, RegistryColumns, RegistryTypeNames, RootKeyEntry, RootKeyEntry__delete, RootKeyEntry__make_0, RootKeyEntry__new_0, RootKeyEntry__new_in_0, RootKeyEntry__zeroed, RootKeys, _IDHWNDPAIR, _IDHWNDPAIR__delete, _IDHWNDPAIR__make_0, _IDHWNDPAIR__new_0, _IDHWNDPAIR__new_in_0, _IDHWNDPAIR__zeroed, _bindgen_ty_1, _bindgen_ty_1__bindgen_ty_1, cpp_alloc, cpp_delete, cpp_delete_array, cpp_free, cpp_new, cpp_new_array, cpp_new_in, cpp_pure_virtual_call, g_Module};
// ==== end port cppapp ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const DEVICE_OBJECT: OBJECT_TYPE = OBJECT_TYPE_DEVICE_OBJECT; // island enum
pub const DIRECTORY_OBJECT: OBJECT_TYPE = OBJECT_TYPE_DIRECTORY_OBJECT; // island enum
pub const EVENTPAIR_OBJECT: OBJECT_TYPE = OBJECT_TYPE_EVENTPAIR_OBJECT; // island enum
pub const EVENT_OBJECT: OBJECT_TYPE = OBJECT_TYPE_EVENT_OBJECT; // island enum
pub const FILE_OBJECT: OBJECT_TYPE = OBJECT_TYPE_FILE_OBJECT; // island enum
pub const FVM_DETAILS: DWORD = 4; // shell32-rust/src/def_view_util.rs
pub const IOCOMPLETION_OBJECT: OBJECT_TYPE = OBJECT_TYPE_IOCOMPLETION_OBJECT; // island enum
pub const KEY_OBJECT: OBJECT_TYPE = OBJECT_TYPE_KEY_OBJECT; // island enum
pub const LVCFMT_LEFT: u32 = 0; // access-rust/src/types_amd64.rs
pub const MUTANT_OBJECT: OBJECT_TYPE = OBJECT_TYPE_MUTANT_OBJECT; // island enum
pub const PORT_OBJECT: OBJECT_TYPE = OBJECT_TYPE_PORT_OBJECT; // island enum
pub const REG_ENTRY_KEY: REG_ENTRY_TYPE = REG_ENTRY_TYPE_REG_ENTRY_KEY; // island enum
pub const REG_ENTRY_ROOT: REG_ENTRY_TYPE = REG_ENTRY_TYPE_REG_ENTRY_ROOT; // island enum
pub const REG_ENTRY_VALUE: REG_ENTRY_TYPE = REG_ENTRY_TYPE_REG_ENTRY_VALUE; // island enum
pub const REG_ENTRY_VALUE_WITH_CONTENT: REG_ENTRY_TYPE = REG_ENTRY_TYPE_REG_ENTRY_VALUE_WITH_CONTENT; // island enum
pub const SECTION_OBJECT: OBJECT_TYPE = OBJECT_TYPE_SECTION_OBJECT; // island enum
pub const SEMAPHORE_OBJECT: OBJECT_TYPE = OBJECT_TYPE_SEMAPHORE_OBJECT; // island enum
pub const SHCOLSTATE_ONBYDEFAULT: DWORD = 0x10; // cabview-rust/src/prelude.rs
pub const SHCOLSTATE_SLOW: DWORD = 0x20; // shell32-rust/src/cfs_folder.rs
pub const SHCOLSTATE_TYPE_STR: DWORD = 1; // cabview-rust/src/prelude.rs
pub const SHCONTF_FOLDERS: tagSHCONTF = tagSHCONTF_SHCONTF_FOLDERS; // island enum
pub const SHCONTF_NONFOLDERS: tagSHCONTF = tagSHCONTF_SHCONTF_NONFOLDERS; // island enum
pub const SHGDN_FOREDITING: tagSHGDN = tagSHGDN_SHGDN_FOREDITING; // island enum
pub const SHGDN_FORPARSING: tagSHGDN = tagSHGDN_SHGDN_FORPARSING; // island enum
pub const SHGDN_INFOLDER: tagSHGDN = tagSHGDN_SHGDN_INFOLDER; // island enum
pub const STRRET_WSTR: UINT = 0; // cabview-rust/src/prelude.rs
pub const SYMBOLICLINK_OBJECT: OBJECT_TYPE = OBJECT_TYPE_SYMBOLICLINK_OBJECT; // island enum
pub const TIMER_OBJECT: OBJECT_TYPE = OBJECT_TYPE_TIMER_OBJECT; // island enum
pub const UNKNOWN_OBJECT_TYPE: OBJECT_TYPE = OBJECT_TYPE_UNKNOWN_OBJECT_TYPE; // island enum
pub const VT_BSTR: DWORD = 8; // cabview-rust/src/prelude.rs
pub const VT_EMPTY: u16 = 0; // cabview-rust/src/prelude.rs
extern "C" {
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
}
extern "system" {
    pub fn CoTaskMemAlloc(size: SIZE_T) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemFree(ptr: LPVOID); // aclui-rust/src/types_i386.rs
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetModuleFileNameW(hModule: HINSTANCE, lpFilename: LPWSTR, nSize: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn SysAllocString(arg1: *const OLECHAR) -> BSTR; // msctf-rust/src/types_i386.rs
    pub fn lstrcmpW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::ntobjshex::ntobjenum_cpp_h::GetEnumNTDirectory;
pub use crate::ntobjshex::ntobjenum_cpp_h::GetEnumRegistryKey;
pub use crate::ntobjshex::ntobjenum_cpp_h::GetEnumRegistryRoot;
pub use crate::ntobjshex::util_h_h::GetFullName;
pub use crate::ntobjshex::ntobjenum_cpp_h::GetNTObjectSymbolicLinkTarget;
pub use crate::ntobjshex::util_h_h::MakeStrRetFromString__0;
pub use crate::ntobjshex::util_h_h::MakeStrRetFromString__1;
pub use crate::ntobjshex::util_h_h::MakeVariantString;
pub use crate::ntobjshex::ntobjenum_cpp_h::ReadRegistryValue;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbLengthW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const MAX_EMBEDDED_DATA: i32 = 0x20; // clang, ntobjenum.cpp
pub const NT_OBJECT_PIDL_MAGIC: u16 = 0x9A03; // clang, ntobjenum.cpp
pub const REGISTRY_PIDL_MAGIC: u16 = 0x5364; // clang, ntobjenum.cpp
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn PathAppendW(arg1: LPWSTR, arg2: LPCWSTR) -> BOOL; // acppage-rust/src/types_i386.rs
}
pub use crate::prelude::PathAppendW as PathAppend; // sdk alias
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

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! HRESULT_FROM_NT { ($p0:expr $(,)?) => { ((($p0 | 0x10000000)) as HRESULT) }; } // gen:dd4ea9bfea29
#[macro_export] macro_rules! IsEqualIID { ($p0:expr, $p1:expr $(,)?) => { IsEqualGUID!(($p0), ($p1)) }; } // gen:57475db97dfd
#[macro_export] macro_rules! MAKE_COMPARE_HRESULT { ($p0:expr $(,)?) => { (if ((($p0 > 0) as c_int)) as isize != 0 { ((1) as HRESULT) } else { (if ((($p0 < 0) as c_int)) as isize != 0 { 0xffff } else { ((0) as HRESULT) }) }) }; } // gen:15072ba69188
#[macro_export] macro_rules! V_BSTR { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.bstrVal }; } // gen:4066e24ecd24
#[macro_export] macro_rules! V_VT { ($p0:expr $(,)?) => { (*$p0).n1.n2.vt }; } // gen:bb0eb6ad6416
#[macro_export] macro_rules! isalphaW { ($p0:expr $(,)?) => { iswalpha((($p0) as _)) }; } // gen:610d8fe123f3
#[macro_export] macro_rules! min { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 < $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; } // gen:fb1d8a90940a
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT); // gen:cd8a52171603
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT); // gen:80daa5b2ffa6
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT); // gen:cbf314acc21c
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT); // gen:98bee37aa995
pub const E_POINTER: HRESULT = ((0x80004003u32) as HRESULT); // gen:f72eeacc5d42
pub const E_UNEXPECTED: HRESULT = ((0x8000FFFFu32) as HRESULT); // gen:c1fe647c5e73
pub const HKEY_CLASSES_ROOT: HKEY = ((((((0x80000000u32) as LONG)) as LONG_PTR)) as HKEY); // gen:76d941b315d7
pub const HKEY_CURRENT_CONFIG: HKEY = ((((((0x80000005u32) as LONG)) as LONG_PTR)) as HKEY); // gen:9d32956405e7
pub const HKEY_CURRENT_USER: HKEY = ((((((0x80000001u32) as LONG)) as LONG_PTR)) as HKEY); // gen:f003d9cba1a1
pub const HKEY_LOCAL_MACHINE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY); // gen:6463b26ca00d
pub const HKEY_USERS: HKEY = ((((((0x80000003u32) as LONG)) as LONG_PTR)) as HKEY); // gen:2df773c10cde
pub const PID_STG_CONTENTS: PROPID = 0x13usize as PROPID; // gen:1a1555a45328
pub const PID_STG_NAME: PROPID = 0xausize as PROPID; // gen:8def0c3ab287
pub const PID_STG_STORAGETYPE: PROPID = 0x4usize as PROPID; // gen:25116f36b7c1
pub const PSGUID_STORAGE: GUID = GUID { Data1: 0xb725f130, Data2: 0x47ef, Data3: 0x101a, Data4: [0xa5, 0xf1, 0x02, 0x60, 0x8c, 0x9e, 0xeb, 0xac] }; // gen:83a7af8e1c06
pub const S_EQUAL: HRESULT = ((0) as HRESULT); // gen:54fd235266bd
pub const S_FALSE: HRESULT = ((1) as HRESULT); // gen:b1b8213e7dac
pub const S_GREATERTHAN: HRESULT = ((1) as HRESULT); // gen:dc85c6c50d6f
pub const S_OK: HRESULT = ((0) as HRESULT); // gen:9969a2c48b97
pub use crate::{HRESULT_FROM_NT, IsEqualIID, MAKE_COMPARE_HRESULT, V_BSTR, V_VT, isalphaW, min};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/shellext/ntobjshex/precomp.h:52: #define GET_SHGDN_FOR(dwFlags) ((DWORD)dwFlags & (DWORD)0x0000FF00) // gen:72ab4c435fe1
#[allow(unused_macros)]
macro_rules! GET_SHGDN_FOR {
    ($dwFlags:expr $(,)?) => {
        ((($dwFlags) as u32) & (0x0000FF00u32))
    };
}
#[allow(unused_imports)]
pub(crate) use GET_SHGDN_FOR;
// dll/shellext/ntobjshex/precomp.h:53: #define GET_SHGDN_RELATION(dwFlags) ((DWORD)dwFlags & (DWORD)0x000000FF) // gen:e001e45b4253
#[allow(unused_macros)]
macro_rules! GET_SHGDN_RELATION {
    ($dwFlags:expr $(,)?) => {
        ((($dwFlags) as u32) & (0x000000FFu32))
    };
}
#[allow(unused_imports)]
pub(crate) use GET_SHGDN_RELATION;
// ==== end port macros ====
