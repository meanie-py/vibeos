//! cabview-rust prelude — the single import surface the translated modules use.
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
pub use atl_rust::CStringW as CString;
pub use atl_rust::CComHeapPtr;
pub use atl_rust::shellutils::{HIDA_GetPIDLItem, HResultFromWin32, ILIsEmpty, ILIsSingle, SHELL_ErrorBox, SHILClone, hida_clipformat};
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
pub use crate::cabview::cpp_classes::*;
pub use crate::cabview::cpp_classes::{CABITEM, CABITEM__delete, CABITEM__make_0, CABITEM__new_0, CABITEM__new_in_0, CABITEM__zeroed, CCabFolder, CCabFolder__IPersistFolder2_VTBL, CCabFolder__IShellFolder2_VTBL, CCabFolder__IShellFolderViewCB_VTBL, CCabFolder__IShellIcon_VTBL, CCabFolder__delete, CCabFolder__make_0, CCabFolder__new_0, CCabFolder__new_in_0, CCabFolder__zeroed, CEnumIDList, CEnumIDList__IEnumIDList_VTBL, CEnumIDList__delete, CEnumIDList__make_0, CEnumIDList__new_0, CEnumIDList__new_in_0, CEnumIDList__zeroed, CLSID_CabFolder, COLCOUNT, COL_ATT, COL_MDATE, COL_NAME, COL_PATH, COL_SIZE, COL_TYPE, EXTRACTCABINETINTERNALDATA, EXTRACTCABINETINTERNALDATA__delete, EXTRACTCABINETINTERNALDATA__make_0, EXTRACTCABINETINTERNALDATA__new_0, EXTRACTCABINETINTERNALDATA__new_in_0, EXTRACTCABINETINTERNALDATA__zeroed, EXTRACTCALLBACKDATA__delete, EXTRACTCALLBACKDATA__make_0, EXTRACTCALLBACKDATA__new_0, EXTRACTCALLBACKDATA__new_in_0, EXTRACTCALLBACKDATA__zeroed, EXTRACTFILESDATA, EXTRACTFILESDATA__delete, EXTRACTFILESDATA__make_0, EXTRACTFILESDATA__new_0, EXTRACTFILESDATA__new_in_0, EXTRACTFILESDATA__zeroed, FILLCALLBACKDATA, FILLCALLBACKDATA__delete, FILLCALLBACKDATA__make_0, FILLCALLBACKDATA__new_0, FILLCALLBACKDATA__new_in_0, FILLCALLBACKDATA__zeroed, FOLDERCOLUMN, FOLDERCOLUMNS, FOLDERCOLUMN__delete, FOLDERCOLUMN__make_0, FOLDERCOLUMN__new_0, FOLDERCOLUMN__new_in_0, FOLDERCOLUMN__zeroed, ObjectMap, PDNCALLBACKDATA, PDNCALLBACKDATA__delete, PDNCALLBACKDATA__make_0, PDNCALLBACKDATA__new_0, PDNCALLBACKDATA__new_in_0, PDNCALLBACKDATA__zeroed, cpp_alloc, cpp_delete, cpp_delete_array, cpp_free, cpp_new, cpp_new_array, cpp_new_in, cpp_pure_virtual_call, g_Columns, g_Module};
// ==== end port cppapp ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const BFFM_SETSELECTION: DWORD = 0x467; // newdev-rust/src/prelude.rs
pub const CP_ACP: u32 = 0; // advapi32-sys/src/bindgen_amd64.rs
pub const CP_UTF8: u32 = 65001; // advapi32-sys/src/bindgen_amd64.rs
pub const DVASPECT_CONTENT: DWORD = 1; // acppage-rust/src/prelude.rs
pub const ECM_BEGIN: EXTRACTCALLBACKMSG = EXTRACTCALLBACKMSG_ECM_BEGIN; // island enum
pub const ECM_ERROR: EXTRACTCALLBACKMSG = EXTRACTCALLBACKMSG_ECM_ERROR; // island enum
pub const ECM_FILE: EXTRACTCALLBACKMSG = EXTRACTCALLBACKMSG_ECM_FILE; // island enum
pub const ECM_PREPAREPATH: EXTRACTCALLBACKMSG = EXTRACTCALLBACKMSG_ECM_PREPAREPATH; // island enum
pub const FDIERROR_ALLOC_FAIL: FDIERROR = FDIERROR_FDIERROR_ALLOC_FAIL; // island enum
pub const FDIERROR_CABINET_NOT_FOUND: FDIERROR = FDIERROR_FDIERROR_CABINET_NOT_FOUND; // island enum
pub const FDIERROR_NONE: FDIERROR = FDIERROR_FDIERROR_NONE; // island enum
pub const FDIERROR_USER_ABORT: FDIERROR = FDIERROR_FDIERROR_USER_ABORT; // island enum
pub const LVCFMT_LEFT: u32 = 0; // access-rust/src/types_amd64.rs
pub const LVCFMT_RIGHT: u32 = 1; // access-rust/src/types_amd64.rs
pub const PID_FINDDATA: u32 = 0; // control-rust/src/types_amd64.rs
pub const SHCOLSTATE_ONBYDEFAULT: DWORD = 0x10; // shell32-rust/src/cfs_folder.rs
pub const SHCOLSTATE_TYPE_DATE: DWORD = 3; // shell32-rust/src/cfs_folder.rs
pub const SHCOLSTATE_TYPE_INT: DWORD = 2; // shell32-rust/src/cfs_folder.rs
pub const SHCOLSTATE_TYPE_STR: DWORD = 1; // shell32-rust/src/cfs_folder.rs
pub const SHCONTF_FOLDERS: tagSHCONTF = tagSHCONTF_SHCONTF_FOLDERS; // island enum
pub const SHCONTF_INCLUDEHIDDEN: tagSHCONTF = tagSHCONTF_SHCONTF_INCLUDEHIDDEN; // island enum
pub const SHCONTF_INCLUDESUPERHIDDEN: tagSHCONTF = tagSHCONTF_SHCONTF_INCLUDESUPERHIDDEN; // island enum
pub const SHCONTF_NONFOLDERS: tagSHCONTF = tagSHCONTF_SHCONTF_NONFOLDERS; // island enum
pub const SHCONTF_STORAGE: tagSHCONTF = tagSHCONTF_SHCONTF_STORAGE; // island enum
pub const SHGDN_FORPARSING: tagSHGDN = tagSHGDN_SHGDN_FORPARSING; // island enum
pub const SHGDN_INFOLDER: tagSHGDN = tagSHGDN_SHGDN_INFOLDER; // island enum
pub const SHGDN_NORMAL: tagSHGDN = tagSHGDN_SHGDN_NORMAL; // island enum
pub const STRRET_CSTR: UINT = 2; // shell32-rust/src/prelude.rs
pub const STRRET_WSTR: UINT = 0; // shell32-rust/src/folders/CDrivesFolder.rs
pub const TYMED_HGLOBAL: DWORD = 1; // acppage-rust/src/prelude.rs
pub const TYMED_NULL: DWORD = 0; // mydocs-rust/src/prelude.rs
pub const VT_ARRAY: VARTYPE = 0x2000; // oleaut32-rust/src/olesys.rs
pub const VT_BSTR: DWORD = 8; // cleanmgr-rust/src/prelude.rs
pub const VT_DATE: DWORD = 7; // shell32-rust/src/cfs_folder.rs
pub const VT_EMPTY: u16 = 0; // mf-rust/src/prelude.rs
pub const VT_UI1: VARTYPE = 17; // mfplat-rust/src/prelude.rs
pub const VT_UI4: DWORD = 19; // mfplat-rust/src/prelude.rs
pub const _O_CREAT: u32 = 256; // msvcrt-sys/src/bindgen_amd64.rs
pub const _O_RDWR: u32 = 2; // msvcrt-sys/src/bindgen_amd64.rs
pub const _O_WRONLY: u32 = 1; // msvcrt-sys/src/bindgen_amd64.rs
pub const fdintCABINET_INFO: FDINOTIFICATIONTYPE = FDINOTIFICATIONTYPE_fdintCABINET_INFO; // island enum
pub const fdintCLOSE_FILE_INFO: FDINOTIFICATIONTYPE = FDINOTIFICATIONTYPE_fdintCLOSE_FILE_INFO; // island enum
pub const fdintCOPY_FILE: FDINOTIFICATIONTYPE = FDINOTIFICATIONTYPE_fdintCOPY_FILE; // island enum
pub const fdintENUMERATE: FDINOTIFICATIONTYPE = FDINOTIFICATIONTYPE_fdintENUMERATE; // island enum
pub const fdintNEXT_CABINET: FDINOTIFICATIONTYPE = FDINOTIFICATIONTYPE_fdintNEXT_CABINET; // island enum
pub const fdintPARTIAL_FILE: FDINOTIFICATIONTYPE = FDINOTIFICATIONTYPE_fdintPARTIAL_FILE; // island enum
extern "C" {
    pub static CLSID_ProgressDialog: GUID; // control-rust/src/types_amd64.rs
    pub static FMTID_ShellDetails: GUID; // control-rust/src/types_amd64.rs
    pub static FMTID_Storage: GUID; // control-rust/src/types_amd64.rs
    pub static IID_IDataObject: GUID; // aclui-rust/src/types_amd64.rs
    pub static IID_IPersist: GUID; // aclui-rust/src/types_amd64.rs
    pub static IID_IProgressDialog: GUID; // control-rust/src/types_amd64.rs
    pub static IID_IShellFolderViewCB: GUID; // control-rust/src/types_amd64.rs
}
extern "system" {
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CoGetInterfaceAndReleaseStream( pStm: LPSTREAM, iid: *const IID, ppv: *mut LPVOID, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoMarshalInterThreadInterfaceInStream( riid: *const IID, pUnk: LPUNKNOWN, ppStm: *mut LPSTREAM, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn DPA_Create(cItemGrow: ::core::ffi::c_int) -> HDPA; // aclui-rust/src/types_i386.rs
    pub fn DPA_InsertPtr( hdpa: HDPA, i: ::core::ffi::c_int, p: *mut ::core::ffi::c_void, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DosDateTimeToFileTime(arg1: WORD, arg2: WORD, arg3: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DosDateTimeToVariantTime(arg1: USHORT, arg2: USHORT, arg3: *mut f64) -> INT; // msctf-rust/src/types_i386.rs
    pub fn FileTimeToLocalFileTime(arg1: *const FILETIME, arg2: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetModuleHandleW(arg1: LPCWSTR) -> HMODULE; // aclui-rust/src/types_i386.rs
    pub fn GlobalLock(arg1: HGLOBAL) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn GlobalUnlock(arg1: HGLOBAL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn InsertMenuItemW(arg1: HMENU, arg2: UINT, arg3: BOOL, arg4: LPCMENUITEMINFOW) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsWindowVisible(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LoadStringW( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn MultiByteToWideChar( arg1: UINT, arg2: DWORD, arg3: LPCSTR, arg4: ::core::ffi::c_int, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ReadFile( arg1: HANDLE, arg2: PVOID, arg3: DWORD, arg4: PDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ReleaseStgMedium(pmedium: *mut STGMEDIUM); // desk-rust/src/prelude.rs
    pub fn SafeArrayCreateVector(arg1: ::core::ffi::c_ushort, arg2: LONG, arg3: ULONG) -> *mut SAFEARRAY; // msctf-rust/src/types_i386.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn SetFilePointer(arg1: HANDLE, arg2: LONG, arg3: PLONG, arg4: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn SetFileTime( arg1: HANDLE, arg2: *const FILETIME, arg3: *const FILETIME, arg4: *const FILETIME, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SysAllocStringLen(arg1: *const OLECHAR, ui: UINT) -> BSTR; // msctf-rust/src/types_i386.rs
    pub fn WideCharToMultiByte( arg1: UINT, arg2: DWORD, arg3: LPCWSTR, arg4: ::core::ffi::c_int, arg5: LPSTR, arg6: ::core::ffi::c_int, arg7: LPCSTR, arg8: LPBOOL, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn WriteFile( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn lstrcmpiW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn lstrcpyA(lpString1: LPSTR, lpString2: LPCSTR) -> LPSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrcpynW(arg1: LPWSTR, arg2: LPCWSTR, arg3: ::core::ffi::c_int) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrlenW(arg1: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::cabview::util_h_h::ErrorBox__T0;
pub use crate::cabview::extract_cpp_h::ExtractCabinet;
pub use crate::cabview::util_h_h::GetDetailsOf;
pub use crate::cabview::util_h_h::IncludeInEnumIDList;
pub use crate::cabview::util_h_h::InitVariantFromBuffer;
pub use crate::cabview::util_h_h::InsertMenuItem;
pub use crate::cabview::util_h_h::IsEqual;
pub use crate::cabview::util_h_h::IsPathSep__T0;
pub use crate::cabview::util_h_h::IsPathSep__T1;
pub use crate::cabview::util_h_h::MapFSToSFAttributes;
pub use crate::cabview::util_h_h::MapPIDLToSystemImageListIndex;
pub use crate::cabview::util_h_h::StrRetToVariantBSTR;
pub use crate::cabview::util_h_h::StrTo__0;
pub use crate::cabview::util_h_h::StrTo__1;
pub use crate::cabview::util_h_h::StrTo__2;
pub use crate::cabview::util_h_h::StrTo__3;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const DPA_ERR: i32 = 0xFFFFFFFF_u32 as i32; // clang, cabview.cpp
pub const FSATTS: u32 = 0x37; // clang, folder.cpp
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn SHGetFileInfoW( pszPath: LPCWSTR, dwFileAttributes: DWORD, psfi: *mut SHFILEINFOW, cbFileInfo: UINT, uFlags: UINT, ) -> DWORD_PTR; // acppage-rust/src/types_i386.rs
}
pub use crate::prelude::SHGetFileInfoW as SHGetFileInfo; // sdk alias
pub use crate::prelude::lstrcpynW as lstrcpyn; // sdk alias
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
pub type WIN32_FIND_DATA = WIN32_FIND_DATAW;
// ==== end UNICODE TCHAR family ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! DPA_AppendPtr { ($p0:expr, $p1:expr $(,)?) => { DPA_InsertPtr((($p0) as _), ((0x7fffffff) as _), (($p1) as _)) }; } // gen:95f48dbf6f2c
#[macro_export] macro_rules! DPA_FastGetPtr { ($p0:expr, $p1:expr $(,)?) => { (*((*((((($p0) as *mut BYTE)).wrapping_add((core::mem::size_of::<*mut c_void>()) as usize)) as *mut *mut *mut c_void))).add(($p1) as usize)) }; } // gen:5cacff9a5e0d
#[macro_export] macro_rules! DPA_GetPtrCount { ($p0:expr $(,)?) => { (*(($p0) as *mut c_int)) }; } // gen:959597abc43e
#[macro_export] macro_rules! HRESULT_CODE { ($p0:expr $(,)?) => { ($p0 & 0xFFFF) }; } // gen:cf237b6d05e9
#[macro_export] macro_rules! MAKE_COMPARE_HRESULT { ($p0:expr $(,)?) => { (if ((($p0 > 0) as c_int)) as isize != 0 { ((1) as HRESULT) } else { (if ((($p0 < 0) as c_int)) as isize != 0 { 0xffff } else { ((0) as HRESULT) }) }) }; } // gen:15072ba69188
#[macro_export] macro_rules! V_BSTR { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.bstrVal }; } // gen:4066e24ecd24
#[macro_export] macro_rules! V_DATE { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.date }; } // gen:60f8d64781fb
#[macro_export] macro_rules! V_UI4 { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.ulVal }; } // gen:419fb51bc82e
#[macro_export] macro_rules! V_UNION { ($p0:expr, $p1:ident $(,)?) => { (*$p0).n1.n2.n3.$p1 }; } // gen:aa8f5fd8eb6f
#[macro_export] macro_rules! V_VT { ($p0:expr $(,)?) => { (*$p0).n1.n2.vt }; } // gen:bb0eb6ad6416
#[macro_export] macro_rules! max { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 > $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; } // gen:29bb71d79dfd
pub const ANSI_NULL: CHAR = ((0) as CHAR); // gen:c71ec491a7a6
pub type BROWSEINFO = BROWSEINFOW; // gen:da831566238b
pub const E_FAIL: HRESULT = ((0x80004005u32) as HRESULT); // gen:1275c1cb9e8d
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT); // gen:cd8a52171603
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT); // gen:80daa5b2ffa6
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT); // gen:cbf314acc21c
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT); // gen:98bee37aa995
pub const E_UNEXPECTED: HRESULT = ((0x8000FFFFu32) as HRESULT); // gen:c1fe647c5e73
pub const INVALID_FILE_ATTRIBUTES: DWORD = (((-(1 as c_int))) as DWORD); // gen:da773c86944d
pub const PID_STG_ATTRIBUTES: PROPID = 0xdusize as PROPID; // gen:155ea740f41e
pub const PID_STG_NAME: PROPID = 0xausize as PROPID; // gen:8def0c3ab287
pub const PID_STG_SIZE: PROPID = 0xcusize as PROPID; // gen:e7c8594359f9
pub const PID_STG_STORAGETYPE: PROPID = 0x4usize as PROPID; // gen:25116f36b7c1
pub const PID_STG_WRITETIME: PROPID = 0xeusize as PROPID; // gen:92debd9a7410
pub type REFGUID = *const GUID; // gen:4c78e76f64d6
pub const S_FALSE: HRESULT = ((1) as HRESULT); // gen:b1b8213e7dac
pub const S_OK: HRESULT = ((0) as HRESULT); // gen:9969a2c48b97
pub use crate::{DPA_AppendPtr, DPA_FastGetPtr, DPA_GetPtrCount, HRESULT_CODE, MAKE_COMPARE_HRESULT, V_BSTR, V_DATE, V_UI4, V_UNION, V_VT, max};
// ==== end SDK header macros ====
