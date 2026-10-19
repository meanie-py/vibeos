//! fontext-rust prelude — the single import surface the translated modules use.
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
pub use atl_rust::CComHeapPtr;
pub use atl_rust::shellutils::{DataObject_SetData, HIDA_GetPIDLFolder, HIDA_GetPIDLItem, hida_clipformat};
pub use atl_rust::{SHSetStrRet, ShellObjectCreatorInit};
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
pub use crate::fontext::cpp_classes::*;
pub use crate::fontext::cpp_classes::{CEnumFonts, CEnumFonts__IEnumIDList_VTBL, CEnumFonts__delete, CEnumFonts__make_0, CEnumFonts__new_0, CEnumFonts__new_in_0, CEnumFonts__zeroed, CFontBackgroundMenu, CFontBackgroundMenu__IContextMenu3_VTBL, CFontBackgroundMenu__delete, CFontBackgroundMenu__make_0, CFontBackgroundMenu__new_0, CFontBackgroundMenu__new_in_0, CFontBackgroundMenu__zeroed, CFontCache, CFontCache__delete, CFontCache__make_0, CFontCache__new_0, CFontCache__new_in_0, CFontCache__zeroed, CFontExt, CFontExt__IDropTarget_VTBL, CFontExt__IPersistFolder2_VTBL, CFontExt__IShellFolder2_VTBL, CFontExt__delete, CFontExt__make_0, CFontExt__new_0, CFontExt__new_in_0, CFontExt__zeroed, CFontFolderViewCB, CFontFolderViewCB__IShellFolderViewCB_VTBL, CFontFolderViewCB__delete, CFontFolderViewCB__make_0, CFontFolderViewCB__new_0, CFontFolderViewCB__new_in_0, CFontFolderViewCB__zeroed, CFontInfo, CFontInfo__delete, CFontInfo__make_0, CFontInfo__new_0, CFontInfo__new_in_0, CFontInfo__zeroed, CRegKey, CRegKeyHandleArray, CRegKeyHandleArray__delete, CRegKeyHandleArray__make_0, CRegKeyHandleArray__new_0, CRegKeyHandleArray__new_in_0, CRegKeyHandleArray__zeroed, CRegKey__delete, CRegKey__make_0, CRegKey__make_1, CRegKey__make_2, CRegKey__new_0, CRegKey__new_1, CRegKey__new_2, CRegKey__new_in_0, CRegKey__new_in_1, CRegKey__new_in_2, CRegKey__zeroed, FONTEXT_COL_ATTR, FONTEXT_COL_FILENAME, FONTEXT_COL_MODIFIED, FONTEXT_COL_NAME, FONTEXT_COL_SIZE, FolderViewColumns, FolderViewColumns__delete, FolderViewColumns__make_0, FolderViewColumns__new_0, FolderViewColumns__new_in_0, FolderViewColumns__zeroed, FontPidlEntry, FontPidlEntry__delete, FontPidlEntry__make_0, FontPidlEntry__new_0, FontPidlEntry__new_in_0, FontPidlEntry__zeroed, ObjectMap, cpp_alloc, cpp_delete, cpp_delete_array, cpp_free, cpp_new, cpp_new_array, cpp_new_in, cpp_pure_virtual_call, font_columns, gModule, g_ColumnDefs, tagINSTALL_FONT_DATA__delete, tagINSTALL_FONT_DATA__make_0, tagINSTALL_FONT_DATA__new_0, tagINSTALL_FONT_DATA__new_in_0, tagINSTALL_FONT_DATA__zeroed};
// ==== end port cppapp ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const DATE_SHORTDATE: u32 = 1; // appwiz-rust/src/types_amd64.rs
pub const DVASPECT_CONTENT: DWORD = 1; // acppage-rust/src/prelude.rs
pub const FR_PRIVATE: u32 = 16; // access-rust/src/types_amd64.rs
pub const LVCFMT_LEFT: u32 = 0; // access-rust/src/types_amd64.rs
pub const LVCFMT_RIGHT: u32 = 1; // access-rust/src/types_amd64.rs
pub const PBM_SETPOS: u32 = 1026; // access-rust/src/types_amd64.rs
pub const PBM_SETRANGE: u32 = 1025; // access-rust/src/types_amd64.rs
pub const SHCOLSTATE_ONBYDEFAULT: DWORD = 0x10; // cabview-rust/src/prelude.rs
pub const SHCOLSTATE_TYPE_DATE: DWORD = 3; // cabview-rust/src/prelude.rs
pub const SHCOLSTATE_TYPE_STR: DWORD = 1; // cabview-rust/src/prelude.rs
pub const SHGDN_FORPARSING: tagSHGDN = tagSHGDN_SHGDN_FORPARSING; // island enum
pub const SHGDN_INFOLDER: tagSHGDN = tagSHGDN_SHGDN_INFOLDER; // island enum
pub const STRRET_CSTR: UINT = 2; // cabview-rust/src/prelude.rs
pub const TIME_NOSECONDS: u32 = 2; // appwiz-rust/src/types_amd64.rs
pub const TYMED_HGLOBAL: tagTYMED = tagTYMED_TYMED_HGLOBAL; // island enum
pub const TYMED_NULL: tagTYMED = tagTYMED_TYMED_NULL; // island enum
extern "C" {
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // acppage-rust/src/prelude.rs
    pub fn ros_dbg_log( cls: ::core::ffi::c_uint, ch: *mut __wine_debug_channel, file: *const ::core::ffi::c_char, func: *const ::core::ffi::c_char, line: ::core::ffi::c_int, format: *const ::core::ffi::c_char, ... ) -> ::core::ffi::c_int; // winegdi-rust/src/types_i386.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
}
extern "system" {
    pub fn AddFontResourceW(pszFilename: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn AppendMenuW(arg1: HMENU, arg2: UINT, arg3: UINT_PTR, arg4: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemAlloc(size: SIZE_T) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemFree(ptr: LPVOID); // aclui-rust/src/types_i386.rs
    pub fn CompareFileTime(arg1: *const FILETIME, arg2: *const FILETIME) -> LONG; // aclui-rust/src/types_i386.rs
    pub fn CopyFileW( lpExistingFileName: LPCWSTR, lpNewFileName: LPCWSTR, bFailIfExists: BOOL, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn DeleteFileW(arg1: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DeleteMenu(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DialogBoxParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> INT_PTR; // aclui-rust/src/types_i386.rs
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnableMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FileTimeToLocalFileTime(arg1: *const FILETIME, arg2: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FileTimeToSystemTime(arg1: *const FILETIME, arg2: LPSYSTEMTIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FindClose(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FindFirstFileW(arg1: LPCWSTR, arg2: LPWIN32_FIND_DATAW) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetDateFormatA( arg1: LCID, arg2: DWORD, arg3: *const SYSTEMTIME, arg4: LPCSTR, arg5: LPSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetTimeFormatA( arg1: LCID, arg2: DWORD, arg3: *const SYSTEMTIME, arg4: LPCSTR, arg5: LPSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR; // charmap-rust/src/prelude.rs
    pub fn GlobalLock(arg1: HGLOBAL) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn GlobalUnlock(arg1: HGLOBAL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn InsertMenuW(arg1: HMENU, arg2: UINT, arg3: UINT, arg4: UINT_PTR, arg5: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn MessageBoxW( hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn OleGetClipboard(ppDataObj: *mut *mut IDataObject) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ProgIDFromCLSID(clsid: *const CLSID, lplpszProgID: *mut LPOLESTR) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn ReleaseStgMedium(arg1: LPSTGMEDIUM); // aclui-rust/src/types_i386.rs
    pub fn RemoveFontResourceExW(arg1: LPCWSTR, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RemoveFontResourceW(arg1: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SendDlgItemMessageW( arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn SetFileAttributesW(lpFileName: LPCWSTR, dwFileAttributes: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetMenuDefaultItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR; // charmap-rust/src/prelude.rs
    pub fn StringFromGUID2(id: *const GUID, str_: LPOLESTR, cmax: INT) -> INT; // aclui-rust/src/types_i386.rs
    pub fn WriteFile( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn lstrcmpiA(arg1: LPCSTR, arg2: LPCSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn lstrcpynA(arg1: LPSTR, arg2: LPCSTR, arg3: ::core::ffi::c_int) -> LPSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrcpynW(arg1: LPWSTR, arg2: LPCWSTR, arg3: ::core::ffi::c_int) -> LPWSTR; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::fontext::CFontBackgroundMenu_cpp_h::CFontBackgroundMenu_Create;
pub use crate::fontext::fontext_cpp_h::CheckDataObject;
pub use crate::fontext::fontext_cpp_h::CloseRegKeyArray;
pub use crate::fontext::fontext_cpp_h::DoDeleteFontFiles;
pub use crate::fontext::fontext_cpp_h::DoPreviewFontFiles;
pub use crate::fontext::fontext_cpp_h::InstallFontsFromDataObject;
pub use crate::fontext::CDataObject_cpp_h::_CDataObject_CreateInstance;
pub use crate::fontext::CEnumFonts_cpp_h::_CEnumFonts_CreateInstance;
pub use crate::fontext::fontpidl_cpp_h::_FontFromIL;
pub use crate::fontext::fontpidl_cpp_h::_ILCreate;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbLengthW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const FONTPIDL_MAGIC: i32 = 0x7066; // clang, fontpidl.cpp
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn PathFindFileNameW(pszPath: LPCWSTR) -> LPWSTR; // acppage-rust/src/types_i386.rs
    pub fn RegDeleteValueW(arg1: HKEY, arg2: LPCWSTR) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
}
pub use win32macros_rust::WINE_ERR as ERR; // sdk alias
pub use crate::prelude::PathFindFileNameW as PathFindFileName; // sdk alias
pub use crate::prelude::RegDeleteValueW as RegDeleteValue; // sdk alias
pub use crate::prelude::RegOpenKeyExW as RegOpenKeyEx; // sdk alias
pub use crate::prelude::RegQueryValueExW as RegQueryValueEx; // sdk alias
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_fontext: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x66\x6f\x6e\x74\x65\x78\x74\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_fontext };
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
#[macro_export] macro_rules! IsEqualIID { ($p0:expr, $p1:expr $(,)?) => { IsEqualGUID!(($p0), ($p1)) }; } // gen:57475db97dfd
#[macro_export] macro_rules! MAKELPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM) }; } // gen:b08956c0c0dd
#[macro_export] macro_rules! MAKE_COMPARE_HRESULT { ($p0:expr $(,)?) => { (if ((($p0 > 0) as c_int)) as isize != 0 { ((1) as HRESULT) } else { (if ((($p0 < 0) as c_int)) as isize != 0 { 0xffff } else { ((0) as HRESULT) }) }) }; } // gen:15072ba69188
#[macro_export] macro_rules! MAKE_HRESULT { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((((((($p0) as c_uint) << 31) | ((($p1) as c_uint) << 16)) | (($p2) as c_uint))) as HRESULT) }; } // gen:829efe363f45
#[macro_export] macro_rules! min { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 < $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; } // gen:fb1d8a90940a
pub const DFM_CMD_COPY: UINT = (((-(3 as c_int))) as UINT); // gen:ec3ea4102ddf
pub const DFM_CMD_DELETE: UINT = (((-(1 as c_int))) as UINT); // gen:f5dbe2b4f66f
pub const DFM_CMD_MOVE: UINT = (((-(2 as c_int))) as UINT); // gen:2f06002f8da0
pub const DFM_CMD_PASTE: UINT = (((-(7 as c_int))) as UINT); // gen:a508c435ae63
pub const DFM_CMD_PROPERTIES: UINT = (((-(5 as c_int))) as UINT); // gen:d7d076e8cd94
pub const E_ABORT: HRESULT = ((0x80004004u32) as HRESULT); // gen:aace7783eaab
pub const E_FAIL: HRESULT = ((0x80004005u32) as HRESULT); // gen:1275c1cb9e8d
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT); // gen:cd8a52171603
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT); // gen:80daa5b2ffa6
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT); // gen:cbf314acc21c
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT); // gen:98bee37aa995
pub const E_POINTER: HRESULT = ((0x80004003u32) as HRESULT); // gen:f72eeacc5d42
pub const E_UNEXPECTED: HRESULT = ((0x8000FFFFu32) as HRESULT); // gen:c1fe647c5e73
pub const FONT_HIVE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY); // gen:c1b99921feb5
pub const HKEY_CLASSES_ROOT: HKEY = ((((((0x80000000u32) as LONG)) as LONG_PTR)) as HKEY); // gen:76d941b315d7
pub const HWND_BROADCAST: HWND = 0xffffusize as HWND; // gen:7bbe1c304e55
pub const INVALID_FILE_ATTRIBUTES: DWORD = (((-(1 as c_int))) as DWORD); // gen:da773c86944d
pub const LOCALE_USER_DEFAULT: DWORD = ((((((((0x0) as USHORT)) as DWORD) << 16) | ((((((((((0x01) as USHORT)) as c_int) << 10) | ((((0x00) as USHORT)) as c_int))) as USHORT)) as DWORD))) as DWORD); // gen:2d64a44f0c5a
pub type REFIID = *const IID; // gen:c79168ffaea6
pub const S_FALSE: HRESULT = ((1) as HRESULT); // gen:b1b8213e7dac
pub const S_OK: HRESULT = ((0) as HRESULT); // gen:9969a2c48b97
pub use crate::{IsEqualIID, MAKELPARAM, MAKE_COMPARE_HRESULT, MAKE_HRESULT, min};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/shellext/fontext/fontpidl.cpp:14: #define ALIGN_DWORD(size) (((size) + 3) & ~3) // gen:97f2096ebbc9
#[allow(unused_macros)]
macro_rules! ALIGN_DWORD {
    ($size:expr $(,)?) => {
        ((($size) + 3) & !3)
    };
}
#[allow(unused_imports)]
pub(crate) use ALIGN_DWORD;
// ==== end port macros ====
