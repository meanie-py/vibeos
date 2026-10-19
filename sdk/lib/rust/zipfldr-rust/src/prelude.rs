//! zipfldr-rust prelude — the single import surface the translated modules use.
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
pub use atl_rust::{atl_format, atl_format_id, CA2WEX, CW2AEX, CAtlArray, CAtlList, CComPtr, CHeapPtr, CSimpleArray, CLocalAllocator, CSimpleMap, CStringA, CStringW, POSITION, _AtlBaseModule};
pub use atl_rust::CStringW as CSimpleString;
pub use atl_rust::CStringW as CString;
pub use atl_rust::CComHeapPtr;
pub use atl_rust::bstr::CComBSTR;
pub use atl_rust::CPath;
pub use atl_rust::CPath as CPathW;
pub use atl_rust::shellutils::{HIDA_GetPIDLItem, hida_clipformat};
pub use atl_rust::{SHSetStrRet, ShellObjectCreatorIid, ShellObjectCreatorInit};
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedDecrement64, InterlockedExchange, InterlockedExchangePointer, InterlockedIncrement, InterlockedIncrement64, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _byteswap_uint64, _byteswap_ulong, _byteswap_ushort, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_variant, debugstr_vt, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, lrintf, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_longlong, wine_dbgstr_variant, wine_dbgstr_vt, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port cppapp: the module's C++ classes (generated -- `rustconv port cppapp`) ====
pub use crate::zipfldr::cpp_classes::*;
pub use crate::zipfldr::cpp_classes::{CCompleteSettingsPage, CCompleteSettingsPage__VTBL_own, CCompleteSettingsPage__delete, CCompleteSettingsPage__make_0, CCompleteSettingsPage__new_0, CCompleteSettingsPage__new_in_0, CCompleteSettingsPage__vtbl, CCompleteSettingsPage__zeroed, CConfirmReplace, CConfirmReplace__VTBL_own, CConfirmReplace__delete, CConfirmReplace__make_0, CConfirmReplace__new_0, CConfirmReplace__new_in_0, CConfirmReplace__vtbl, CConfirmReplace__zeroed, CEnumExplorerCommand, CEnumExplorerCommand__IEnumExplorerCommand_VTBL, CEnumExplorerCommand__delete, CEnumExplorerCommand__make_0, CEnumExplorerCommand__new_0, CEnumExplorerCommand__new_in_0, CEnumExplorerCommand__zeroed, CEnumZipContents, CEnumZipContents__IEnumIDList_VTBL, CEnumZipContents__delete, CEnumZipContents__make_0, CEnumZipContents__new_0, CEnumZipContents__new_in_0, CEnumZipContents__zeroed, CExplorerCommand, CExplorerCommandProvider, CExplorerCommandProvider__IExplorerCommandProvider_VTBL, CExplorerCommandProvider__delete, CExplorerCommandProvider__make_0, CExplorerCommandProvider__new_0, CExplorerCommandProvider__new_in_0, CExplorerCommandProvider__zeroed, CExplorerCommand__IExplorerCommand_VTBL, CExplorerCommand__delete, CExplorerCommand__make_0, CExplorerCommand__new_0, CExplorerCommand__new_in_0, CExplorerCommand__zeroed, CExtractSettingsPage, CExtractSettingsPage__VTBL_own, CExtractSettingsPage__delete, CExtractSettingsPage__make_0, CExtractSettingsPage__new_0, CExtractSettingsPage__new_in_0, CExtractSettingsPage__vtbl, CExtractSettingsPage__zeroed, CFolderViewCB, CFolderViewCB__IShellFolderViewCB_VTBL, CFolderViewCB__delete, CFolderViewCB__make_0, CFolderViewCB__new_0, CFolderViewCB__new_in_0, CFolderViewCB__zeroed, CLSID_ZipFolderContextMenu, CLSID_ZipFolderDropHandler, CLSID_ZipFolderExtractAllCommand, CLSID_ZipFolderRightDragHandler, CLSID_ZipFolderSendTo, CLSID_ZipFolderStorageHandler, COL_COMPRSIZE, COL_DATE_MOD, COL_NAME, COL_PASSWORD, COL_RATIO, COL_SIZE, COL_TYPE, CSendToZip, CSendToZip__IDropTarget_VTBL, CSendToZip__IPersistFile_VTBL, CSendToZip__delete, CSendToZip__make_0, CSendToZip__new_0, CSendToZip__new_in_0, CSendToZip__zeroed, CWindow, CWindow__delete, CWindow__make_0, CWindow__new_0, CWindow__new_in_0, CWindow__rcDefault, CWindow__zeroed, CZCERR_CREATE, CZCERR_NOFILES, CZCERR_READ, CZCERR_ZEROITEMS, CZC_ERROR, CZipCreator, CZipCreatorImpl, CZipCreatorImpl__delete, CZipCreatorImpl__make_0, CZipCreatorImpl__new_0, CZipCreatorImpl__new_in_0, CZipCreatorImpl__zeroed, CZipCreator__VTBL_own, CZipCreator__delete, CZipCreator__make_0, CZipCreator__new_0, CZipCreator__new_in_0, CZipCreator__vtbl, CZipCreator__zeroed, CZipEnumerator, CZipEnumerator__delete, CZipEnumerator__make_0, CZipEnumerator__new_0, CZipEnumerator__new_in_0, CZipEnumerator__zeroed, CZipExtract, CZipExtract__IZip_VTBL, CZipExtract__delete, CZipExtract__make_0, CZipExtract__new_0, CZipExtract__new_in_0, CZipExtract__zeroed, CZipFolder, CZipFolder__IContextMenu_VTBL, CZipFolder__IDropTarget_VTBL, CZipFolder__IPersistFile_VTBL, CZipFolder__IPersistFolder2_VTBL, CZipFolder__IShellExtInit_VTBL, CZipFolder__IShellFolder2_VTBL, CZipFolder__IZip_VTBL, CZipFolder__delete, CZipFolder__make_0, CZipFolder__new_0, CZipFolder__new_in_0, CZipFolder__zeroed, CZipPassword, CZipPassword__VTBL_own, CZipPassword__delete, CZipPassword__make_0, CZipPassword__new_0, CZipPassword__new_in_0, CZipPassword__vtbl, CZipPassword__zeroed, FOLDERCOLUMN, FolderViewColumn, FolderViewColumn__delete, FolderViewColumn__make_0, FolderViewColumn__new_0, FolderViewColumn__new_in_0, FolderViewColumn__zeroed, IZip, IZipVtbl, ObjectMap, ZIP_PIDL_DIRECTORY, ZIP_PIDL_FILE, ZipPidlEntry, ZipPidlEntry__delete, ZipPidlEntry__make_0, ZipPidlEntry__new_0, ZipPidlEntry__new_in_0, ZipPidlEntry__zeroed, ZipPidlType, _IDHWNDPAIR, _IDHWNDPAIR__delete, _IDHWNDPAIR__make_0, _IDHWNDPAIR__new_0, _IDHWNDPAIR__new_in_0, _IDHWNDPAIR__zeroed, browse_info, browse_info__delete, browse_info__make_0, browse_info__new_0, browse_info__new_in_0, browse_info__zeroed, cpp_alloc, cpp_delete, cpp_delete_array, cpp_free, cpp_new, cpp_new_array, cpp_new_in, cpp_pure_virtual_call, gModule, g_ColumnDefs};
// ==== end port cppapp ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const BFFM_SETSELECTION: DWORD = 0x467; // cabview-rust/src/prelude.rs
pub const CP_ACP: u32 = 0; // advapi32-sys/src/bindgen_amd64.rs
pub const CP_UTF8: u32 = 65001; // advapi32-sys/src/bindgen_amd64.rs
pub const DATE_SHORTDATE: u32 = 1; // appwiz-rust/src/types_amd64.rs
pub const DVASPECT_CONTENT: DWORD = 1; // acppage-rust/src/prelude.rs
pub const ECF_DEFAULT: _EXPCMDFLAGS = _EXPCMDFLAGS_ECF_DEFAULT; // island enum
pub const ECS_ENABLED: _EXPCMDSTATE = _EXPCMDSTATE_ECS_ENABLED; // island enum
pub const FVM_DETAILS: DWORD = 4; // ntobjshex-rust/src/prelude.rs
pub const LVCFMT_LEFT: u32 = 0; // access-rust/src/types_amd64.rs
pub const LVCFMT_RIGHT: u32 = 1; // access-rust/src/types_amd64.rs
pub const MAX_WBITS: i32 = 0xF; // spapisup-rust/src/prelude.rs
pub const PBM_SETPOS: u32 = 1026; // access-rust/src/types_amd64.rs
pub const PBM_SETRANGE32: u32 = 1030; // access-rust/src/types_amd64.rs
pub const PID_STG_NAME: UINT = 10; // shell32-rust/src/cfs_folder.rs
pub const PID_STG_SIZE: UINT = 12; // shell32-rust/src/cfs_folder.rs
pub const PID_STG_STORAGETYPE: UINT = 4; // shell32-rust/src/cfs_folder.rs
pub const PID_STG_WRITETIME: UINT = 14; // shell32-rust/src/cfs_folder.rs
pub const SHCOLSTATE_ONBYDEFAULT: DWORD = 0x10; // cabview-rust/src/prelude.rs
pub const SHCOLSTATE_TYPE_DATE: DWORD = 3; // cabview-rust/src/prelude.rs
pub const SHCOLSTATE_TYPE_INT: DWORD = 2; // cabview-rust/src/prelude.rs
pub const SHCOLSTATE_TYPE_STR: DWORD = 1; // cabview-rust/src/prelude.rs
pub const SHCONTF_FOLDERS: tagSHCONTF = tagSHCONTF_SHCONTF_FOLDERS; // island enum
pub const SHCONTF_INCLUDEHIDDEN: tagSHCONTF = tagSHCONTF_SHCONTF_INCLUDEHIDDEN; // island enum
pub const SHCONTF_INCLUDESUPERHIDDEN: tagSHCONTF = tagSHCONTF_SHCONTF_INCLUDESUPERHIDDEN; // island enum
pub const SHCONTF_NONFOLDERS: tagSHCONTF = tagSHCONTF_SHCONTF_NONFOLDERS; // island enum
pub const SHCONTF_STORAGE: tagSHCONTF = tagSHCONTF_SHCONTF_STORAGE; // island enum
pub const SHGDN_FORPARSING: tagSHGDN = tagSHGDN_SHGDN_FORPARSING; // island enum
pub const SHGDN_INFOLDER: tagSHGDN = tagSHGDN_SHGDN_INFOLDER; // island enum
pub const SHGFP_TYPE_DEFAULT: SHGFP_TYPE = SHGFP_TYPE_SHGFP_TYPE_DEFAULT; // island enum
pub const STRRET_WSTR: UINT = 0; // cabview-rust/src/prelude.rs
pub const TYMED_HGLOBAL: tagTYMED = tagTYMED_TYMED_HGLOBAL; // island enum
pub const TYMED_NULL: tagTYMED = tagTYMED_TYMED_NULL; // island enum
pub const VT_BSTR: DWORD = 8; // activeds-rust/src/prelude.rs
pub const VT_DATE: DWORD = 7; // cabview-rust/src/prelude.rs
pub const VT_EMPTY: u16 = 0; // cabview-rust/src/prelude.rs
pub const VT_UI8: DWORD = 0x15; // mfplat-rust/src/prelude.rs
pub const Z_DEFAULT_STRATEGY: u32 = 0; // spapisup-rust/src/types_amd64.rs
pub const Z_DEFLATED: u32 = 8; // spapisup-rust/src/types_amd64.rs
pub const Z_NULL: u32 = 0; // spapisup-rust/src/types_amd64.rs
pub const eAbort: eZipPasswordResponse = eZipPasswordResponse_eAbort; // island enum
pub const eAccept: eZipPasswordResponse = eZipPasswordResponse_eAccept; // island enum
pub const eCancel: eZipConfirmResponse = eZipConfirmResponse_eCancel; // island enum
pub const eDirectoryError: eZipExtractError = eZipExtractError_eDirectoryError; // island enum
pub const eExtractAbort: eZipExtractError = eZipExtractError_eExtractAbort; // island enum
pub const eFileError: eZipExtractError = eZipExtractError_eFileError; // island enum
pub const eNo: eZipConfirmResponse = eZipConfirmResponse_eNo; // island enum
pub const eNoError: eZipExtractError = eZipExtractError_eNoError; // island enum
pub const eOpenError: eZipExtractError = eZipExtractError_eOpenError; // island enum
pub const eSkip: eZipPasswordResponse = eZipPasswordResponse_eSkip; // island enum
pub const eUnpackError: eZipExtractError = eZipExtractError_eUnpackError; // island enum
pub const eYes: eZipConfirmResponse = eZipConfirmResponse_eYes; // island enum
pub const eYesToAll: eZipConfirmResponse = eZipConfirmResponse_eYesToAll; // island enum
extern "C" {
    pub fn _beginthreadex( security: *mut c_void, stack_size: u32, start_address: Option<unsafe extern "system" fn(*mut c_void) -> u32>, arglist: *mut c_void, initflag: u32, thrdaddr: *mut u32, ) -> usize; // explorer-rust/src/explorer/syspager.rs
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int; // apphelp-rust/src/prelude.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // activeds-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcstol(s: *const u16, endptr: *mut *mut u16, base: i32) -> i32; // cleanmgr-rust/src/prelude.rs
}
extern "system" {
    pub fn CheckDlgButton(arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemAlloc(size: SIZE_T) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemFree(ptr: LPVOID); // aclui-rust/src/types_i386.rs
    pub fn CreateFileMappingW( arg1: HANDLE, arg2: LPSECURITY_ATTRIBUTES, arg3: DWORD, arg4: DWORD, arg5: DWORD, arg6: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateThread( arg1: LPSECURITY_ATTRIBUTES, arg2: DWORD, arg3: LPTHREAD_START_ROUTINE, arg4: PVOID, arg5: DWORD, arg6: PDWORD, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn DeleteFileW(arg1: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DosDateTimeToFileTime(arg1: WORD, arg2: WORD, arg3: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DosDateTimeToVariantTime(arg1: USHORT, arg2: USHORT, arg3: *mut f64) -> INT; // msctf-rust/src/types_i386.rs
    pub fn EnableMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FileTimeToLocalFileTime(arg1: *const FILETIME, arg2: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FileTimeToSystemTime(arg1: *const FILETIME, arg2: LPSYSTEMTIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FindClose(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FindFirstFileW(arg1: LPCWSTR, arg2: LPWIN32_FIND_DATAW) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn FindNextFileW(arg1: HANDLE, arg2: LPWIN32_FIND_DATAW) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FormatMessageW( dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD, lpBuffer: LPWSTR, nSize: DWORD, Arguments: *mut va_list, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetDC(arg1: HWND) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn GetDateFormatW( arg1: LCID, arg2: DWORD, arg3: *const SYSTEMTIME, arg4: LPCWSTR, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn GetDesktopWindow() -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetDlgItemTextA( hDlg: HWND, nIDDlgItem: ::core::ffi::c_int, lpString: LPSTR, cchMax: ::core::ffi::c_int, ) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetFileSizeEx(arg1: HANDLE, arg2: PLARGE_INTEGER) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetFileTime(arg1: HANDLE, arg2: LPFILETIME, arg3: LPFILETIME, arg4: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetFullPathNameW(arg1: LPCWSTR, arg2: DWORD, arg3: LPWSTR, arg4: *mut LPWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetModuleHandleA(arg1: LPCSTR) -> HMODULE; // aclui-rust/src/types_i386.rs
    pub fn GetParent(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetSystemInfo(arg1: LPSYSTEM_INFO); // aclui-rust/src/types_i386.rs
    pub fn GetTempFileNameW(arg1: LPCWSTR, arg2: LPCWSTR, arg3: UINT, arg4: LPWSTR) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetTempPathW(arg1: DWORD, arg2: LPWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetTimeFormatW( arg1: LCID, arg2: DWORD, arg3: *const SYSTEMTIME, arg4: LPCWSTR, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn GetWindowRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetWindowTextLengthA(arg1: HWND) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetWindowTextW( hWnd: HWND, lpString: LPWSTR, nMaxCount: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetWindowsDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GlobalLock(arg1: HGLOBAL) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn GlobalUnlock(arg1: HGLOBAL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn InsertMenuW(arg1: HMENU, arg2: UINT, arg3: UINT, arg4: UINT_PTR, arg5: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsDlgButtonChecked(arg1: HWND, arg2: ::core::ffi::c_int) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // aclui-rust/src/types_i386.rs
    pub fn LoadStringW( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn LocalFileTimeToFileTime(arg1: *const FILETIME, arg2: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn MapViewOfFile( arg1: HANDLE, arg2: DWORD, arg3: DWORD, arg4: DWORD, arg5: SIZE_T, ) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn MessageBoxW( hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn MonitorFromWindow(arg1: HWND, arg2: DWORD) -> HMONITOR; // aclui-rust/src/types_i386.rs
    pub fn MoveWindow( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: BOOL, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ProgIDFromCLSID(clsid: *const CLSID, lplpszProgID: *mut LPOLESTR) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn ReleaseDC(arg1: HWND, arg2: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn ReleaseStgMedium(arg1: LPSTGMEDIUM); // aclui-rust/src/types_i386.rs
    pub fn ReplaceFileW( arg1: LPCWSTR, arg2: LPCWSTR, arg3: LPCWSTR, arg4: DWORD, arg5: LPVOID, arg6: LPVOID, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetDlgItemTextW(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetFileTime( arg1: HANDLE, arg2: *const FILETIME, arg3: *const FILETIME, arg4: *const FILETIME, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn StringFromGUID2(id: *const GUID, str_: LPOLESTR, cmax: INT) -> INT; // aclui-rust/src/types_i386.rs
    pub fn SysAllocString(arg1: *const OLECHAR) -> BSTR; // msctf-rust/src/types_i386.rs
    pub fn UnmapViewOfFile(arg1: LPCVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn WriteFile( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn lstrcmpiW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrlenW(arg1: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::zipfldr::CZipFolder_cpp_h::FmtIdZipFolder;
pub use crate::zipfldr::zipfldr_cpp_h::GetZipCodePage;
pub use crate::zipfldr::CEnumZipContents_cpp_h::_CEnumZipContents_CreateInstance;
pub use crate::zipfldr::CExplorerCommand_cpp_h::_CExplorerCommandProvider_CreateInstance;
pub use crate::zipfldr::CFolderViewCB_cpp_h::_CFolderViewCB_CreateInstance;
pub use crate::zipfldr::CZipPassword_cpp_h::_CZipAskPassword;
pub use crate::zipfldr::CZipExtract_cpp_h::_CZipExtract_runWizard;
pub use crate::zipfldr::zippidl_cpp_h::_ILCreateZipItem;
pub use crate::zipfldr::zippidl_cpp_h::_ZipFromIL;
pub use crate::zipfldr::Debug_cpp_h::guid2string;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbLengthW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const EF_HEADER_SIZE: i32 = 4; // clang, CZipEnumerator.cpp
pub const EF_UNIPATH: i32 = 0x7075; // clang, CZipCreator.cpp
pub const EF_UNIPATH_VERSION: i32 = 1; // clang, CZipEnumerator.cpp
pub const Z_DEFAULT_COMPRESSION: i32 = 0xFFFFFFFF_u32 as i32; // clang, CZipCreator.cpp
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn GetMonitorInfoW(arg1: HMONITOR, arg2: LPMONITORINFO) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetWindowLongW(arg1: HWND, arg2: ::core::ffi::c_int) -> LONG; // aclui-rust/src/types_i386.rs
    pub fn GetWindowTextLengthW(arg1: HWND) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn SHStrDupW(psz: LPCWSTR, ppwsz: *mut *mut WCHAR) -> HRESULT; // acppage-rust/src/types_i386.rs
    pub fn SendDlgItemMessageW( arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn SendMessageCallbackW( arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM, arg5: SENDASYNCPROC, arg6: ULONG_PTR, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
}
pub use crate::prelude::GetMonitorInfoW as GetMonitorInfo; // sdk alias
pub use crate::prelude::GetWindowLongW as GetWindowLong; // sdk alias
pub use crate::prelude::GetWindowTextW as GetWindowText; // sdk alias
pub use crate::prelude::GetWindowTextLengthW as GetWindowTextLength; // sdk alias
pub use crate::prelude::LoadIconW as LoadIcon; // sdk alias
pub use crate::prelude::SHStrDupW as SHStrDup; // sdk alias
pub use crate::prelude::SendDlgItemMessageW as SendDlgItemMessage; // sdk alias
pub use crate::prelude::SendMessageW as SendMessage; // sdk alias
pub use crate::prelude::SendMessageCallbackW as SendMessageCallback; // sdk alias
pub use crate::prelude::SetDlgItemTextW as SetDlgItemText; // sdk alias
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
#[macro_export] macro_rules! HRESULT_CODE { ($p0:expr $(,)?) => { ($p0 & 0xFFFF) }; } // gen:cf237b6d05e9
#[macro_export] macro_rules! HRESULT_FACILITY { ($p0:expr $(,)?) => { (($p0 >> 16) & 0x1FFF) }; } // gen:1588163fe8e1
#[macro_export] macro_rules! MAKEINTRESOURCE { ($p0:expr $(,)?) => { ((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as ULONG_PTR)) as LPWSTR) }; } // gen:971027e46ae0
#[macro_export] macro_rules! MAKE_COMPARE_HRESULT { ($p0:expr $(,)?) => { (if ((($p0 > 0) as c_int)) as isize != 0 { ((1) as HRESULT) } else { (if ((($p0 < 0) as c_int)) as isize != 0 { 0xffff } else { ((0) as HRESULT) }) }) }; } // gen:15072ba69188
#[macro_export] macro_rules! MAKE_HRESULT { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((((((($p0) as c_uint) << 31) | ((($p1) as c_uint) << 16)) | (($p2) as c_uint))) as HRESULT) }; } // gen:829efe363f45
#[macro_export] macro_rules! V_BSTR { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.bstrVal }; } // gen:4066e24ecd24
#[macro_export] macro_rules! V_DATE { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.date }; } // gen:60f8d64781fb
#[macro_export] macro_rules! V_UI8 { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.ullVal }; } // gen:58d84a6f4116
#[macro_export] macro_rules! V_VT { ($p0:expr $(,)?) => { (*$p0).n1.n2.vt }; } // gen:bb0eb6ad6416
#[macro_export] macro_rules! min { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 < $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; } // gen:fb1d8a90940a
pub const CW_USEDEFAULT: c_int = ((0x80000000u32) as c_int); // gen:5affe1df4577
pub const DFM_CMD_DELETE: UINT = (((-(1 as c_int))) as UINT); // gen:f5dbe2b4f66f
pub const E_FAIL: HRESULT = ((0x80004005u32) as HRESULT); // gen:1275c1cb9e8d
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT); // gen:cd8a52171603
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT); // gen:80daa5b2ffa6
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT); // gen:cbf314acc21c
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT); // gen:98bee37aa995
pub const E_POINTER: HRESULT = ((0x80004003u32) as HRESULT); // gen:f72eeacc5d42
pub const E_UNEXPECTED: HRESULT = ((0x8000FFFFu32) as HRESULT); // gen:c1fe647c5e73
pub const HKEY_CLASSES_ROOT: HKEY = ((((((0x80000000u32) as LONG)) as LONG_PTR)) as HKEY); // gen:76d941b315d7
pub const HKEY_CURRENT_USER: HKEY = ((((((0x80000001u32) as LONG)) as LONG_PTR)) as HKEY); // gen:f003d9cba1a1
pub const IDI_EXCLAMATION: LPWSTR = 0x7f03usize as LPWSTR; // gen:b693917ee9aa
pub const LOCALE_USER_DEFAULT: DWORD = ((((((((0x0) as USHORT)) as DWORD) << 16) | ((((((((((0x01) as USHORT)) as c_int) << 10) | ((((0x00) as USHORT)) as c_int))) as USHORT)) as DWORD))) as DWORD); // gen:2d64a44f0c5a
pub type REFCLSID = *const CLSID; // gen:4a91e3a9efb2
pub type REFGUID = *const GUID; // gen:4c78e76f64d6
pub type REFIID = *const IID; // gen:c79168ffaea6
pub const S_FALSE: HRESULT = ((1) as HRESULT); // gen:b1b8213e7dac
pub const S_OK: HRESULT = ((0) as HRESULT); // gen:9969a2c48b97
pub use crate::{HRESULT_CODE, HRESULT_FACILITY, MAKEINTRESOURCE, MAKE_COMPARE_HRESULT, MAKE_HRESULT, V_BSTR, V_DATE, V_UI8, V_VT, min};
// ==== end SDK header macros ====
