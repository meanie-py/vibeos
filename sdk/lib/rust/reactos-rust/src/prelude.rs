//! reactos-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _byteswap_uint64, _byteswap_ulong, _byteswap_ushort, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_variant, debugstr_vt, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, lrintf, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_longlong, wine_dbgstr_variant, wine_dbgstr_vt, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const ERROR_SYSTEM_PARTITION_NOT_FOUND: i32 = 0x2D; // clang, reactos.c
pub const NM_RETURN: u32 = 0xFFFFFFFC; // clang, drivepage.c
pub const SPFILENOTIFY_COPYERROR: i32 = 0xD; // clang, reactos.c
pub const SPFILENOTIFY_ENDCOPY: i32 = 0xC; // clang, reactos.c
pub const SPFILENOTIFY_ENDDELETE: i32 = 6; // clang, reactos.c
pub const SPFILENOTIFY_ENDRENAME: i32 = 9; // clang, reactos.c
pub const SPFILENOTIFY_STARTCOPY: i32 = 0xB; // clang, reactos.c
pub const SPFILENOTIFY_STARTDELETE: i32 = 5; // clang, reactos.c
pub const SPFILENOTIFY_STARTRENAME: i32 = 8; // clang, reactos.c
pub const SPFILENOTIFY_STARTSUBQUEUE: i32 = 3; // clang, reactos.c
pub const STATUS_PARTITION_FAILURE: NTSTATUS = 0xC0000172_u32 as NTSTATUS; // clang, reactos.c
pub const TVN_DELETEITEM: u32 = 0xFFFFFE36; // clang, drivepage.c
// ==== end port facts ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const ARCH_NEC98x86: _ARCHITECTURE_TYPE = _ARCHITECTURE_TYPE_ARCH_NEC98x86; // island enum
pub const ARCH_PcAT: _ARCHITECTURE_TYPE = _ARCHITECTURE_TYPE_ARCH_PcAT; // island enum
pub const ChangeSystemPartition: _FSVOLNOTIFY = _FSVOLNOTIFY_ChangeSystemPartition; // island enum
pub const DONE: CALLBACKCOMMAND = CALLBACKCOMMAND_DONE; // island enum
pub const ERROR_INSTALL_BOOTCODE: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_INSTALL_BOOTCODE; // island enum
pub const ERROR_LOAD_TXTSETUPSIF: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_LOAD_TXTSETUPSIF; // island enum
pub const ERROR_NO_SOURCE_DRIVE: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_NO_SOURCE_DRIVE; // island enum
pub const ERROR_WRITE_BOOT: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_WRITE_BOOT; // island enum
pub const FILEOP_ABORT: u32 = 0; // audiosrv-rust/src/types_amd64.rs
pub const FILEOP_COPY: u32 = 0; // audiosrv-rust/src/types_amd64.rs
pub const FILEOP_DELETE: u32 = 2; // audiosrv-rust/src/types_amd64.rs
pub const FILEOP_DOIT: u32 = 1; // audiosrv-rust/src/types_amd64.rs
pub const FILEOP_RENAME: u32 = 1; // audiosrv-rust/src/types_amd64.rs
pub const FILEOP_SKIP: u32 = 2; // audiosrv-rust/src/types_amd64.rs
pub const FMIFS_HARDDISK: FMIFS_MEDIA_FLAG = FMIFS_MEDIA_FLAG_FMIFS_HARDDISK; // island enum
pub const FSVOLNOTIFY_CHECKERROR: _FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_CHECKERROR; // island enum
pub const FSVOLNOTIFY_ENDCHECK: _FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_ENDCHECK; // island enum
pub const FSVOLNOTIFY_ENDFORMAT: _FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_ENDFORMAT; // island enum
pub const FSVOLNOTIFY_ENDQUEUE: _FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_ENDQUEUE; // island enum
pub const FSVOLNOTIFY_ENDSUBQUEUE: _FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_ENDSUBQUEUE; // island enum
pub const FSVOLNOTIFY_FORMATERROR: _FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_FORMATERROR; // island enum
pub const FSVOLNOTIFY_PARTITIONERROR: _FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_PARTITIONERROR; // island enum
pub const FSVOLNOTIFY_STARTCHECK: _FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_STARTCHECK; // island enum
pub const FSVOLNOTIFY_STARTFORMAT: _FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_STARTFORMAT; // island enum
pub const FSVOLNOTIFY_STARTQUEUE: _FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_STARTQUEUE; // island enum
pub const FSVOLNOTIFY_STARTSUBQUEUE: _FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_STARTSUBQUEUE; // island enum
pub const FSVOL_ABORT: _FSVOL_OP = _FSVOL_OP_FSVOL_ABORT; // island enum
pub const FSVOL_CHECK: _FSVOL_OP = _FSVOL_OP_FSVOL_CHECK; // island enum
pub const FSVOL_DOIT: _FSVOL_OP = _FSVOL_OP_FSVOL_DOIT; // island enum
pub const FSVOL_FORMAT: _FSVOL_OP = _FSVOL_OP_FSVOL_FORMAT; // island enum
pub const FSVOL_RETRY: _FSVOL_OP = _FSVOL_OP_FSVOL_RETRY; // island enum
pub const FSVOL_SKIP: _FSVOL_OP = _FSVOL_OP_FSVOL_SKIP; // island enum
pub const Formatted: _FORMATSTATE = _FORMATSTATE_Formatted; // island enum
pub const GB: ULONGLONG = 0x40000000; // usetup-rust/src/prelude.rs
pub const HWND_DESKTOP: HWND = 0x0usize as HWND; // eventvwr-rust/src/prelude.rs
pub const IOCTL_STORAGE_QUERY_PROPERTY: u32 = 0x002D_1400; // diskpart-rust/src/prelude.rs
pub const MB: ULONGLONG = 0x100000; // usetup-rust/src/prelude.rs
pub const NM_DBLCLK: u32 = 0xFFFFFFFD; // certmgr-rust/src/prelude.rs
pub const NOT_AN_ERROR: _ERROR_NUMBER = _ERROR_NUMBER_NOT_AN_ERROR; // island enum
pub const PROGRESS: CALLBACKCOMMAND = CALLBACKCOMMAND_PROGRESS; // island enum
pub const PropertyStandardQuery: _STORAGE_QUERY_TYPE = _STORAGE_QUERY_TYPE_PropertyStandardQuery; // island enum
pub const StorageDeviceProperty: _STORAGE_PROPERTY_ID = _STORAGE_PROPERTY_ID_StorageDeviceProperty; // island enum
pub const TVN_SELCHANGED: u32 = 0xFFFFFE3D; // certmgr-rust/src/prelude.rs
extern "C" {
    pub fn CreateComputerTypeList(InfFile: HINF) -> PGENERIC_LIST; // setuplib-rust/src/setuplib/settings.rs
    pub fn CreateDisplayDriverList(InfFile: HINF) -> PGENERIC_LIST; // setuplib-rust/src/setuplib/settings.rs
    pub fn CreateKeyboardDriverList(InfFile: HINF) -> PGENERIC_LIST; // setuplib-rust/src/setuplib/settings.rs
    pub fn CreateKeyboardLayoutList(InfFile: HINF, LanguageId: PCWSTR, DefaultKBLayout: PWSTR,) -> PGENERIC_LIST; // setuplib-rust/src/setuplib/settings.rs
    pub fn CreateLanguageList(InfFile: HINF, DefaultLanguage: PWSTR,) -> PGENERIC_LIST; // setuplib-rust/src/setuplib/settings.rs
    pub fn RoundingDivide(Dividend: ULONGLONG, Divisor: ULONGLONG) -> ULONGLONG; // usetup-rust/src/prelude.rs
    pub fn SelectPartition(List: PPARTLIST, DiskNumber: ULONG, PartitionNumber: ULONG,) -> PPARTENTRY; // setuplib-rust/src/setuplib/partlist.rs
    pub fn _vscwprintf(format: *const u16, args: *mut u8) -> i32; // msvcrt-crt-rust/src/printf_shells.rs
    pub fn _wcsdup(arg1: *const wchar_t) -> *mut wchar_t; // msvcrt-sys/src/bindgen_i386.rs
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // apphelp-rust/src/prelude.rs
    pub fn _wcsnicmp(s1: *const u16, s2: *const u16, n: usize) -> i32; // apphelp-rust/src/prelude.rs
    pub fn free(p: *mut c_void); // atlclasses-rust/src/lib.rs
    pub fn swscanf(_Src: *const wchar_t, _Format: *const wchar_t, ...) -> ::core::ffi::c_int; // winsrv-sys/src/bindgen_i386.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcstol(s: *const u16, endptr: *mut *mut u16, base: i32) -> i32; // cleanmgr-rust/src/prelude.rs
}
extern "system" {
    pub fn AdjustTokenPrivileges( arg1: HANDLE, arg2: BOOL, arg3: PTOKEN_PRIVILEGES, arg4: DWORD, arg5: PTOKEN_PRIVILEGES, arg6: PDWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CallWindowProcW( arg1: WNDPROC, arg2: HWND, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn CheckDlgButton(arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CheckRadioButton( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CheckUnattendedSetup(pSetupData: PUSETUP_DATA,) -> BOOLEAN; // setuplib-rust/src/setuplib/setuplib.rs
    pub fn CloseClipboard() -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateEventW( arg1: LPSECURITY_ATTRIBUTES, bManualReset: BOOL, bInitialState: BOOL, lpName: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateMutexW(arg1: LPSECURITY_ATTRIBUTES, arg2: BOOL, arg3: LPCWSTR) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateNTOSInstallationsList(PartList: *mut PARTLIST,) -> *mut GENERIC_LIST; // setuplib-rust/src/setuplib/osdetect.rs
    pub fn CreatePartition(List: PPARTLIST, PartEntry: PPARTENTRY, SizeBytes: ULONGLONG, PartitionInfo: ULONG_PTR,) -> BOOLEAN; // setuplib-rust/src/setuplib/partlist.rs
    pub fn CreatePartitionList() -> *mut PARTLIST; // setuplib-rust/src/setuplib/partlist.rs
    pub fn CreateProcessW( arg1: LPCWSTR, arg2: LPWSTR, arg3: LPSECURITY_ATTRIBUTES, arg4: LPSECURITY_ATTRIBUTES, arg5: BOOL, arg6: DWORD, arg7: PVOID, arg8: LPCWSTR, arg9: LPSTARTUPINFOW, arg10: LPPROCESS_INFORMATION, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateThread( arg1: LPSECURITY_ATTRIBUTES, arg2: DWORD, arg3: LPTHREAD_START_ROUTINE, arg4: PVOID, arg5: DWORD, arg6: PDWORD, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn DeleteObject(arg1: HGDIOBJ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DeletePartition(List: PPARTLIST, PartEntry: PPARTENTRY, FreeRegion: *mut PPARTENTRY,) -> BOOLEAN; // setuplib-rust/src/setuplib/partlist.rs
    pub fn DeviceIoControl( hDevice: *mut c_void, dwIoControlCode: u32, lpInBuffer: *mut c_void, nInBufferSize: u32, lpOutBuffer: *mut c_void, nOutBufferSize: u32, lpBytesReturned: LPDWORD, lpOverlapped: LPOVERLAPPED, ) -> BOOL; // shell32-rust/src/island_i386.rs
    pub fn DialogBoxParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> INT_PTR; // aclui-rust/src/types_i386.rs
    pub fn DispatchMessageW(arg1: *const MSG) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn DoFileCopy(pSetupData: *mut USETUP_DATA, MsgHandler: PSP_FILE_CALLBACK_W, Context: *mut core::ffi::c_void,) -> i32; // setuplib-rust/src/setuplib/install.rs
    pub fn EnableMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnumWindows(lpEnumFunc: WNDENUMPROC, lParam: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ExitProcess(arg1: UINT) -> !; // aclui-rust/src/types_i386.rs
    pub fn ExitWindowsEx(arg1: UINT, arg2: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FindSubStrI(str: PCWSTR, strSearch: PCWSTR,) -> PCWSTR; // setuplib-rust/src/setuplib/osdetect.rs
    pub fn FindWindowW(arg1: LPCWSTR, arg2: LPCWSTR) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn FinishSetup(pSetupData: PUSETUP_DATA); // setuplib-rust/src/setuplib/setuplib.rs
    pub fn FsVolCommitOpsQueue(PartitionList: PPARTLIST, SystemVolume: PVOLENTRY, InstallVolume: PVOLENTRY, FsVolCallback: PFSVOL_CALLBACK, Context: PVOID,) -> BOOLEAN; // setuplib-rust/src/setuplib/fsutil.rs
    pub fn GetAdjUnpartitionedEntry(PartEntry: PPARTENTRY, Direction: BOOLEAN,) -> PPARTENTRY; // setuplib-rust/src/setuplib/partlist.rs
    pub fn GetClipboardData(arg1: UINT) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetCurrentListEntry(List: PGENERIC_LIST) -> PGENERIC_LIST_ENTRY; // setuplib-rust/src/setuplib/genlist.rs
    pub fn GetCurrentProcess() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetDC(arg1: HWND) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn GetDesktopWindow() -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetDeviceCaps(arg1: HDC, arg2: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetDlgItemTextW( hDlg: HWND, nIDDlgItem: ::core::ffi::c_int, lpString: LPWSTR, cchMax: ::core::ffi::c_int, ) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetExitCodeThread(arg1: HANDLE, arg2: PDWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetFirstListEntry(List: PGENERIC_LIST) -> PGENERIC_LIST_ENTRY; // setuplib-rust/src/setuplib/genlist.rs
    pub fn GetKeyboardLayoutNameW(arg1: LPWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetLastActivePopup(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetListEntryData(Entry: PGENERIC_LIST_ENTRY) -> PVOID; // setuplib-rust/src/setuplib/genlist.rs
    pub fn GetLogicalDriveStringsW(arg1: DWORD, arg2: LPWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetMenuItemInfoW(arg1: HMENU, arg2: UINT, arg3: BOOL, arg4: LPMENUITEMINFOW) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetMessageW(arg1: LPMSG, arg2: HWND, arg3: UINT, arg4: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetModuleFileNameW(hModule: HINSTANCE, lpFilename: LPWSTR, nSize: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetModuleHandleW(arg1: LPCWSTR) -> HMODULE; // aclui-rust/src/types_i386.rs
    pub fn GetNextListEntry(Entry: PGENERIC_LIST_ENTRY,) -> PGENERIC_LIST_ENTRY; // setuplib-rust/src/setuplib/genlist.rs
    pub fn GetNumberOfListEntries(List: PGENERIC_LIST) -> u32; // setuplib-rust/src/setuplib/genlist.rs
    pub fn GetObjectW(h: HANDLE, c: ::core::ffi::c_int, pv: LPVOID) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetParent(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetRegisteredFileSystems(Index: u32, FileSystemName: *mut PCWSTR,) -> i32; // setuplib-rust/src/setuplib/fsutil.rs
    pub fn GetSysColor(arg1: ::core::ffi::c_int) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetSystemMenu(arg1: HWND, arg2: BOOL) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetThreadLocale() -> LCID; // desk-rust/src/types_i386.rs
    pub fn GetUserDefaultLCID() -> LCID; // desk-rust/src/types_i386.rs
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR; // charmap-rust/src/prelude.rs
    pub fn GetWindowLongW(arg1: HWND, arg2: ::core::ffi::c_int) -> LONG; // aclui-rust/src/types_i386.rs
    pub fn GetWindowRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetWindowTextLengthW(arg1: HWND) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetWindowTextW( hWnd: HWND, lpString: LPWSTR, nMaxCount: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GlobalAddAtomW(arg1: LPCWSTR) -> ATOM; // aclui-rust/src/types_i386.rs
    pub fn GlobalAlloc(arg1: UINT, arg2: SIZE_T) -> HGLOBAL; // aclui-rust/src/types_i386.rs
    pub fn GlobalDeleteAtom(arg1: ATOM) -> ATOM; // aclui-rust/src/types_i386.rs
    pub fn GlobalLock(arg1: HGLOBAL) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn GlobalUnlock(arg1: HGLOBAL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ImageList_Create( cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, flags: UINT, cInitial: ::core::ffi::c_int, cGrow: ::core::ffi::c_int, ) -> HIMAGELIST; // aclui-rust/src/types_i386.rs
    pub fn ImageList_Destroy(himl: HIMAGELIST) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ImageList_ReplaceIcon( himl: HIMAGELIST, i: ::core::ffi::c_int, hicon: HICON, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn InitCommonControlsEx(arg1: *const INITCOMMONCONTROLSEX) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn InitDestinationPaths(pSetupData: *mut USETUP_DATA, InstallationDir: PCWSTR, Volume: *mut VOLENTRY,) -> NTSTATUS; // setuplib-rust/src/setuplib/setuplib.rs
    pub fn InitSystemPartition(PartitionList: PPARTLIST, InstallPartition: PPARTENTRY, pSystemPartition: *mut PPARTENTRY, FsVolCallback: PFSVOL_CALLBACK, Context: PVOID,) -> BOOLEAN; // setuplib-rust/src/setuplib/setuplib.rs
    pub fn InitializeSetup(pSetupData: PUSETUP_DATA, ErrorRoutine: PSETUP_ERROR_ROUTINE, pSpFileExports: PSPFILE_EXPORTS, pSpInfExports: PSPINF_EXPORTS,) -> ERROR_NUMBER; // setuplib-rust/src/setuplib/setuplib.rs
    pub fn InstallBootManagerAndBootEntries(ArchType: ARCHITECTURE_TYPE, SystemRootPath: PCUNICODE_STRING, SourceRootPath: PCUNICODE_STRING, DestinationArcPath: PCUNICODE_STRING, Options: usize,) -> i32; // setuplib-rust/src/setuplib/bootsup.rs
    pub fn InstallBootcodeToRemovable(ArchType: ARCHITECTURE_TYPE, RemovableRootPath: PCUNICODE_STRING, SourceRootPath: PCUNICODE_STRING, DestinationArcPath: PCUNICODE_STRING,) -> NTSTATUS; // setuplib-rust/src/setuplib/bootsup.rs
    pub fn InstallSetupInfFile(pSetupData: PUSETUP_DATA); // setuplib-rust/src/setuplib/setuplib.rs
    pub fn InvalidateRect(arg1: HWND, arg2: LPCRECT, arg3: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsDlgButtonChecked(arg1: HWND, arg2: ::core::ffi::c_int) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn IsValidInstallDirectory(InstallDir: PCWSTR) -> i32; // setuplib-rust/src/setuplib/setuplib.rs
    pub fn IsWindowEnabled(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsWindowUnicode(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn KillTimer(arg1: HWND, arg2: UINT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // aclui-rust/src/types_i386.rs
    pub fn LoadStringW( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn LocalAlloc(arg1: UINT, arg2: SIZE_T) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn LookupPartitionTypeString(PartitionStyle: PARTITION_STYLE, PartitionType: PVOID) -> PCSTR; // usetup-rust/src/prelude.rs
    pub fn LookupPrivilegeValueW(arg1: LPCWSTR, arg2: LPCWSTR, arg3: PLUID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn MapWindowPoints( hWndFrom: HWND, hWndTo: HWND, lpPoints: LPPOINT, cPoints: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn MessageBeep(uType: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn MessageBoxIndirectW(lpmbp: *const MSGBOXPARAMSW) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn MessageBoxW( hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn MsgWaitForMultipleObjects( nCount: DWORD, pHandles: *const HANDLE, fWaitAll: BOOL, dwMilliseconds: DWORD, dwWakeMask: DWORD, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn MulDiv( arg1: ::core::ffi::c_int, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn MultiByteToWideChar( arg1: UINT, arg2: DWORD, arg3: LPCSTR, arg4: ::core::ffi::c_int, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn OpenClipboard(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn OpenProcessToken(arg1: HANDLE, arg2: DWORD, arg3: PHANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PartitionCreateChecks(PartEntry: PPARTENTRY, SizeBytes: ULONGLONG, PartitionInfo: ULONG_PTR,) -> ERROR_NUMBER; // setuplib-rust/src/setuplib/partlist.rs
    pub fn PeekMessageW(arg1: LPMSG, arg2: HWND, arg3: UINT, arg4: UINT, arg5: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PostThreadMessageW(arg1: DWORD, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PrepareFileCopy(pSetupData: PUSETUP_DATA, StatusRoutine: PFILE_COPY_STATUS_ROUTINE,) -> BOOLEAN; // setuplib-rust/src/setuplib/install.rs
    pub fn PropertySheetW(arg1: LPCPROPSHEETHEADERW) -> INT_PTR; // aclui-rust/src/types_i386.rs
    pub fn QueryDosDeviceW(arg1: LPCWSTR, arg2: LPWSTR, arg3: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn RegisterHotKey(arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT, arg4: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ReleaseDC(arg1: HWND, arg2: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn ResetEvent(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SendDlgItemMessageW( arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn SetClipboardData(arg1: UINT, arg2: HANDLE) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn SetCurrentListEntry(List: PGENERIC_LIST, Entry: PGENERIC_LIST_ENTRY,); // setuplib-rust/src/setuplib/genlist.rs
    pub fn SetCursor(arg1: HCURSOR) -> HCURSOR; // aclui-rust/src/types_i386.rs
    pub fn SetDlgItemTextW(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetEvent(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetFocus(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn SetThreadLocale(arg1: LCID) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn SetTimer(arg1: HWND, arg2: UINT_PTR, arg3: UINT, arg4: TIMERPROC) -> UINT_PTR; // aclui-rust/src/types_i386.rs
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR; // charmap-rust/src/prelude.rs
    pub fn SetWindowLongW(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LONG) -> LONG; // aclui-rust/src/types_i386.rs
    pub fn SetWindowPos( arg1: HWND, arg2: HWND, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: UINT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetWindowTextW(arg1: HWND, arg2: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetupCloseInfFile(arg1: HINF); // desk-rust/src/types_i386.rs
    pub fn SetupFindFirstLineW(arg1: HINF, arg2: PCWSTR, arg3: PCWSTR, arg4: PINFCONTEXT) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn SetupFindNextLine(arg1: PINFCONTEXT, arg2: PINFCONTEXT) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn SetupGetBinaryField( Context: PINFCONTEXT, FieldIndex: DWORD, ReturnBuffer: PBYTE, ReturnBufferSize: DWORD, RequiredSize: LPDWORD, ) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn SetupGetFieldCount(arg1: PINFCONTEXT) -> DWORD; // desk-rust/src/types_i386.rs
    pub fn SetupGetIntField(arg1: PINFCONTEXT, arg2: DWORD, arg3: PINT) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn SetupGetMultiSzFieldW( Context: PINFCONTEXT, FieldIndex: DWORD, ReturnBuffer: PWSTR, ReturnBufferSize: DWORD, RequiredSize: LPDWORD, ) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn SetupGetStringFieldW( Context: PINFCONTEXT, FieldIndex: DWORD, ReturnBuffer: PWSTR, ReturnBufferSize: DWORD, RequiredSize: PDWORD, ) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn SetupOpenInfFileW( FileName: PCWSTR, InfClass: PCWSTR, InfStyle: DWORD, ErrorLine: PUINT, ) -> HINF; // desk-rust/src/types_i386.rs
    pub fn ShowCursor(arg1: BOOL) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn StrFormatByteSizeW(arg1: LONGLONG, arg2: LPWSTR, cchBuf: UINT) -> LPWSTR; // certmgr-rust/src/types_i386.rs
    pub fn SwitchToThisWindow(arg1: HWND, arg2: BOOL); // aclui-rust/src/types_i386.rs
    pub fn SystemParametersInfoW( uiAction: UINT, uiParam: UINT, pvParam: PVOID, fWinIni: UINT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TranslateMessage(arg1: *const MSG) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn UnregisterHotKey(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn UpdateRegistry(pSetupData: *mut USETUP_DATA, RepairUpdateFlag: BOOLEAN, PartitionList: *mut PARTLIST, DestinationDriveLetter: WCHAR, SelectedLanguageId: PCWSTR, StatusRoutine: PREGISTRY_STATUS_ROUTINE, SubstSettings: *mut FONTSUBSTSETTINGS,) -> ERROR_NUMBER; // setuplib-rust/src/setuplib/setuplib.rs
    pub fn UpdateWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// HIMAGELIST: c2rust, drivepage.c
pub type HIMAGELIST = *mut _IMAGELIST;
// PFILEPATHS_W: c2rust, reactos.c
pub type PFILEPATHS_W = *mut _FILEPATHS_W;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn TreeListRegister(hInstance: HINSTANCE) -> ::core::ffi::c_int; // reactos.c
    pub fn TreeListUnregister(hInstance: HINSTANCE) -> BOOL; // reactos.c
    pub fn _stricmp(
        _Str1: *const ::core::ffi::c_char,
        _Str2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int; // reactos.c
    pub fn strlen(_Str: *const ::core::ffi::c_char) -> size_t; // drivepage.c
    pub fn wcschr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut ::core::ffi::c_ushort; // reactos.c
    pub fn wcsrchr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut wchar_t; // reactos.c
}
extern "system" {
    pub fn CreateFontIndirectW(_: *const LOGFONTW) -> HFONT; // reactos.c
    pub fn CreatePropertySheetPageW(_: LPCPROPSHEETPAGEW) -> HPROPSHEETPAGE; // reactos.c
    pub fn LoadCursorW(_: HINSTANCE, _: LPCWSTR) -> HCURSOR; // reactos.c
    pub fn pSetupGetField(Context: PINFCONTEXT, FieldIndex: ULONG) -> PCWSTR; // infsupp.c
}
pub use crate::prelude::CallWindowProcW as CallWindowProc; // sdk alias
pub use crate::prelude::CreateFontIndirectW as CreateFontIndirect; // sdk alias
pub use crate::prelude::CreatePropertySheetPageW as CreatePropertySheetPage; // sdk alias
pub use crate::prelude::LoadCursorW as LoadCursor; // sdk alias
pub use crate::prelude::PostMessageW as PostMessage; // sdk alias
pub use crate::prelude::SendDlgItemMessageW as SendDlgItemMessage; // sdk alias
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

// ==== port variadic: C-variadic entry points, trampolines in the module's rust_varargs_shim.c (generated -- `rustconv port integrate`) ====
extern "C" {
    pub fn DisplayError(hWnd: HWND, uIDTitle: UINT, uIDMessage: UINT, ...) -> INT;
    pub fn DisplayMessage(hWnd: HWND, uType: UINT, pszTitle: PCWSTR, pszFormatMessage: PCWSTR, ...) -> INT;
    pub fn RegistryStatus(RegStatus: REGISTRY_STATUS, ...);
    pub fn SetWindowResPrintfW(hWnd: HWND, hInstance: HINSTANCE, uID: UINT, ...);
}
// ==== end port variadic ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! ComboBox_GetCurSel { ($p0:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((327) as _), ((0) as _), ((0) as _))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ComboBox_GetItemData { ($p0:expr, $p1:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((336) as _), (((($p1) as c_int)) as WPARAM), ((0) as _))) as UINT_PTR)) as LRESULT) }; }
#[macro_export] macro_rules! DBG_UNREFERENCED_PARAMETER { ($p0:expr $(,)?) => { { let _ = $p0; } }; }
#[macro_export] macro_rules! DeleteFont { ($p0:expr $(,)?) => { DeleteObject((((($p0) as HFONT)) as HGDIOBJ)) }; }
#[macro_export] macro_rules! Edit_ShowBalloonTip { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1500 + 3)) as _), ((0) as _), (($p1) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! GetDiskSizeInBytes { ($p0:expr $(,)?) => { ((*$p0).SectorCount.QuadPart * ((*$p0).BytesPerSector as ULONGLONG)) }; }
#[macro_export] macro_rules! GetPartEntrySizeInBytes { ($p0:expr $(,)?) => { ((*$p0).SectorCount.QuadPart * ((*(*$p0).DiskEntry).BytesPerSector as ULONGLONG)) }; }
#[macro_export] macro_rules! ImageList_AddIcon { ($p0:expr, $p1:expr $(,)?) => { ImageList_ReplaceIcon((($p0) as _), (((-(1 as c_int))) as _), (($p1) as _)) }; }
#[macro_export] macro_rules! ListView_EnsureVisible { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 19)) as _), (((($p1) as c_int)) as WPARAM), (((((((((((((($p2) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | ((((((((0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! ListView_GetImageList { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 2)) as _), (((($p1) as INT)) as WPARAM), ((0) as _))) as HIMAGELIST) }; }
#[macro_export] macro_rules! ListView_GetItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 75)) as _), ((0) as _), (($p1) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! ListView_GetSelectionMark { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 66)) as _), ((0) as _), ((0) as _))) as c_int) }; }
#[macro_export] macro_rules! ListView_InsertColumn { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 97)) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as c_int) }; }
#[macro_export] macro_rules! ListView_SetExtendedListViewStyleEx { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 54)) as _), (($p1) as _), (($p2) as _))) as DWORD) }; }
#[macro_export] macro_rules! ListView_SetImageList { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 3)) as _), (($p2) as WPARAM), (($p1) as LPARAM))) as HIMAGELIST) }; }
#[macro_export] macro_rules! ListView_SetItemState { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { { let mut _ms_lvi: LVITEMW = core::mem::zeroed(); { _ms_lvi.stateMask = (($p3) as _); }; { _ms_lvi.state = (($p2) as _); }; SendMessageW((($p0) as _), (((0x1000 + 43)) as _), (($p1) as WPARAM), ((core::ptr::addr_of_mut!(_ms_lvi)) as LPARAM)); } }; }
#[macro_export] macro_rules! MAKELPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM) }; }
#[macro_export] macro_rules! MAKEWPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as WPARAM) }; }
#[macro_export] macro_rules! PropSheet_PressButton { ($p0:expr, $p1:expr $(,)?) => { SendMessageW((($p0) as _), ((1137) as _), (($p1) as _), ((0) as _)) }; }
#[macro_export] macro_rules! PropSheet_SetCurSelByID { ($p0:expr, $p1:expr $(,)?) => { SendMessageW((($p0) as _), ((1138) as _), ((0) as _), (($p1) as _)) }; }
#[macro_export] macro_rules! PropSheet_SetWizButtons { ($p0:expr, $p1:expr $(,)?) => { PostMessageW((($p0) as _), (((1024 + 112)) as _), ((0) as _), (($p1) as LPARAM)) }; }
#[macro_export] macro_rules! TreeList_DeleteAllItems { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 1)) as _), ((0) as _), (((((-(0x10000 as c_int))) as ULONG_PTR)) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! TreeList_DeleteItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 1)) as _), ((0) as _), (($p1) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! TreeList_Expand { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 2)) as _), (($p2) as WPARAM), (($p1) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! TreeList_GetImageList { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 8)) as _), (($p1) as _), ((0) as _))) as HIMAGELIST) }; }
#[macro_export] macro_rules! TreeList_GetItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 62)) as _), ((0) as _), (($p1) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! TreeList_GetNextItem { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 10)) as _), (($p2) as WPARAM), (($p1) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeList_GetNextSibling { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 10)) as _), ((0x1) as WPARAM), (($p1) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeList_GetParent { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 10)) as _), ((0x3) as WPARAM), (($p1) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeList_GetPrevSibling { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 10)) as _), ((0x2) as WPARAM), (($p1) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeList_GetSelection { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 10)) as _), ((0x9) as WPARAM), ((core::ptr::null_mut::<c_void>()) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeList_GetStyle { ($p0:expr $(,)?) => { ((GetWindowLongW((($p0) as _), (((-(16 as c_int))) as _))) as DWORD) }; }
#[macro_export] macro_rules! TreeList_InsertColumn { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((((0x1100 + 128) - 4) - 1)) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as INT) }; }
#[macro_export] macro_rules! TreeList_InsertItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 50)) as _), ((0) as _), (($p1) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeList_SelectItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 11)) as _), ((0x9) as WPARAM), (($p1) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! TreeList_SetColor { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 29)) as _), (($p1) as WPARAM), (($p2) as _))) as COLORREF) }; }
#[macro_export] macro_rules! TreeList_SetExtendedStyle { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 44)) as _), ((0) as _), (($p1) as _))) as DWORD) }; }
#[macro_export] macro_rules! TreeList_SetImageList { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 9)) as _), (($p2) as _), (($p1) as LPARAM))) as HIMAGELIST) }; }
#[macro_export] macro_rules! TreeList_SetItemState { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { { let mut _ms_TVi: TVITEMW = core::mem::zeroed(); { _ms_TVi.mask = ((0x8) as _); }; { _ms_TVi.hItem = (($p1) as _); }; { _ms_TVi.stateMask = (($p3) as _); }; { _ms_TVi.state = (($p2) as _); }; SendMessageW((($p0) as _), (((0x1100 + 63)) as _), ((0) as _), ((core::ptr::addr_of_mut!(_ms_TVi)) as LPARAM)); } }; }
#[macro_export] macro_rules! TreeList_SetStyle { ($p0:expr, $p1:expr $(,)?) => { ((SetWindowLongW((($p0) as _), (((-(16 as c_int))) as _), (($p1) as _))) as DWORD) }; }
#[macro_export] macro_rules! max { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 > $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
#[macro_export] macro_rules! min { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 < $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
#[macro_export] macro_rules! va_end { ($p0:expr $(,)?) => { __builtin_va_end((($p0) as _)) }; }
#[macro_export] macro_rules! va_start { ($p0:expr, $p1:expr $(,)?) => { __builtin_va_start((($p0) as _), (($p1) as _)) }; }
pub type HTLITEM = HTREEITEM;
pub const HWND_TOP: HWND = 0x0usize as HWND;
pub const IDC_WAIT: LPWSTR = 0x7f02usize as LPWSTR;
pub const KB: ULONGLONG = ((1024) as ULONGLONG);
pub type LPNMTREEVIEW = LPNMTREEVIEWW;
pub type LVCOLUMN = LVCOLUMNW;
pub type LVITEM = LVITEMW;
pub const SE_SHUTDOWN_NAME: *const u16 = (&[0x0053, 0x0065, 0x0053, 0x0068, 0x0075, 0x0074, 0x0064, 0x006f, 0x0077, 0x006e, 0x0050, 0x0072, 0x0069, 0x0076, 0x0069, 0x006c, 0x0065, 0x0067, 0x0065, 0x0000u16]).as_ptr(); // L"SeShutdownPrivilege"
pub const STATUS_DEVICE_NOT_READY: NTSTATUS = 0xc00000a3usize as NTSTATUS;
pub const STATUS_DLL_NOT_FOUND: NTSTATUS = 0xc0000135usize as NTSTATUS;
pub const STATUS_UNRECOGNIZED_VOLUME: NTSTATUS = 0xc000014fusize as NTSTATUS;
pub type TLCOLUMN = LVCOLUMNW;
pub type TLINSERTSTRUCTW = TVINSERTSTRUCTW;
pub type TLITEMW = TVITEMW;
pub const TVI_FIRST: HTREEITEM = (((((-(0xffff as c_int))) as ULONG_PTR)) as HTREEITEM);
pub const TVI_LAST: HTREEITEM = (((((-(0xfffe as c_int))) as ULONG_PTR)) as HTREEITEM);
pub const TVI_ROOT: HTREEITEM = (((((-(0x10000 as c_int))) as ULONG_PTR)) as HTREEITEM);
pub use crate::{ComboBox_GetCurSel, ComboBox_GetItemData, DBG_UNREFERENCED_PARAMETER, DeleteFont, Edit_ShowBalloonTip, GetDiskSizeInBytes, GetPartEntrySizeInBytes, ImageList_AddIcon, ListView_EnsureVisible, ListView_GetImageList, ListView_GetItem, ListView_GetSelectionMark, ListView_InsertColumn, ListView_SetExtendedListViewStyleEx, ListView_SetImageList, ListView_SetItemState, MAKELPARAM, MAKEWPARAM, PropSheet_PressButton, PropSheet_SetCurSelByID, PropSheet_SetWizButtons, TreeList_DeleteAllItems, TreeList_DeleteItem, TreeList_Expand, TreeList_GetImageList, TreeList_GetItem, TreeList_GetNextItem, TreeList_GetNextSibling, TreeList_GetParent, TreeList_GetPrevSibling, TreeList_GetSelection, TreeList_GetStyle, TreeList_InsertColumn, TreeList_InsertItem, TreeList_SelectItem, TreeList_SetColor, TreeList_SetExtendedStyle, TreeList_SetImageList, TreeList_SetItemState, TreeList_SetStyle, max, min, va_end, va_start};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// base/setup/reactos/reactos.h:31: #define EnableDlgItem(hDlg, nID, bEnable) EnableWindow(GetDlgItem((hDlg), (nID)), (bEnable))
#[allow(unused_macros)]
macro_rules! EnableDlgItem {
    ($hDlg:expr, $nID:expr, $bEnable:expr $(,)?) => {
        (EnableWindow(GetDlgItem(($hDlg), ($nID)), ($bEnable)))
    };
}
#[allow(unused_imports)]
pub(crate) use EnableDlgItem;
// base/setup/reactos/reactos.h:118: #define InterlockedFlagsTestAndSet8(Target, Flags) !!(InterlockedOr8((PCHAR)(Target), (Flags)) & (Flags))
#[allow(unused_macros)]
macro_rules! InterlockedFlagsTestAndSet8 {
    ($Target:expr, $Flags:expr $(,)?) => {
        (((InterlockedOr8(($Target) as PCHAR, ($Flags) as c_char) as c_int) & (($Flags) as c_int)) != 0)
    };
}
#[allow(unused_imports)]
pub(crate) use InterlockedFlagsTestAndSet8;
// base/setup/reactos/reactos.h:37: #define SetDlgItemFont(hDlg, nID, hFont, bRedraw) SetWindowFont(GetDlgItem((hDlg), (nID)), (hFont), (bRedraw))
#[allow(unused_macros)]
macro_rules! SetDlgItemFont {
    ($hDlg:expr, $nID:expr, $hFont:expr, $bRedraw:expr $(,)?) => {
        (SetWindowFont!(GetDlgItem(($hDlg), ($nID)), ($hFont), ($bRedraw)))
    };
}
#[allow(unused_imports)]
pub(crate) use SetDlgItemFont;
// base/setup/reactos/reactos.h:34: #define ShowDlgItem(hDlg, nID, nCmdShow) ShowWindow(GetDlgItem((hDlg), (nID)), (nCmdShow))
#[allow(unused_macros)]
macro_rules! ShowDlgItem {
    ($hDlg:expr, $nID:expr, $nCmdShow:expr $(,)?) => {
        (ShowWindow(GetDlgItem(($hDlg), ($nID)), ($nCmdShow)))
    };
}
#[allow(unused_imports)]
pub(crate) use ShowDlgItem;
// ==== end port macros ====

// ==== hand-transcribed (not generated) ====
// base/setup/lib/spapisup/fileqsup.h:58 (and psdk/setupapi.h:755, same layout on every
// target: two pointers then two 32-bit fields, so setupapi.h's pack(1) on i386 adds nothing).
// The island lacks it, so the `port local types` alias above has nothing to point at.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _FILEPATHS_W {
    pub Target: PCWSTR,
    pub Source: PCWSTR,
    pub Win32Error: UINT,
    pub Flags: ULONG,
}
pub type FILEPATHS_W = _FILEPATHS_W;

// base/setup/lib/setuplib.h:206:
// #define IS_VALID_INSTALL_PATH_CHAR(c) (isalnum(c) || (c) == L'.' || (c) == L'\\' || (c) == L'-' || (c) == L'_')
// The C hands the WCHAR, promoted to int, straight to msvcrt's isalnum.
extern "C" {
    pub fn isalnum(c: c_int) -> c_int;
}
#[macro_export]
macro_rules! IS_VALID_INSTALL_PATH_CHAR {
    ($c:expr $(,)?) => {
        ((isalnum(($c) as c_int) != 0)
            || (($c) as WCHAR == b'.' as WCHAR)
            || (($c) as WCHAR == b'\\' as WCHAR)
            || (($c) as WCHAR == b'-' as WCHAR)
            || (($c) as WCHAR == b'_' as WCHAR))
    };
}
pub use crate::IS_VALID_INSTALL_PATH_CHAR;

// psdk/windowsx.h:533: #define SetWindowFont(hwnd,hfont,fRedraw) FORWARD_WM_SETFONT((hwnd),(hfont),(fRedraw),SNDMSG)
// windowsx.h:220: #define FORWARD_WM_SETFONT(hwnd,hfont,fRedraw,fn) (void)(fn)((hwnd),WM_SETFONT,(WPARAM)(HFONT)(hfont),(LPARAM)(BOOL)(fRedraw))
#[macro_export]
macro_rules! SetWindowFont {
    ($hwnd:expr, $hfont:expr, $fRedraw:expr $(,)?) => {{
        let _ = SendMessageW(
            ($hwnd) as HWND,
            WM_SETFONT as UINT,
            (($hfont) as HFONT) as WPARAM,
            (($fRedraw) as BOOL) as LPARAM,
        );
    }};
}
pub use crate::SetWindowFont;

// vendor/treelist/treelist.h:409:
// #define TreeList_SetItem(h,p) ((BOOL)SNDMSG(h,TVM_SETITEM,0,(LPARAM)(const TV_ITEM*)(p)))
#[macro_export]
macro_rules! TreeList_SetItem {
    ($h:expr, $p:expr $(,)?) => {
        (SendMessageW(($h) as HWND, TVM_SETITEMW as UINT, 0 as WPARAM, (($p) as *const TVITEMW) as LPARAM)
            as BOOL)
    };
}
pub use crate::TreeList_SetItem;

// vendor/treelist/treelist.h:496 (the __REACTOS__ branch -- reactos.h builds with it defined):
// { TV_ITEM _ms_tvi; LPWSTR _my_pszText = (pszText_);
//   _ms_tvi.mask = TVIF_SUBITEM | TVIF_TEXT; _ms_tvi.hItem = (hItem_); _ms_tvi.stateMask = 0;
//   _ms_tvi.pszText = _my_pszText; _ms_tvi.cchTextMax = _my_pszText ? (int)wcslen(_my_pszText) : 0;
//   _ms_tvi.cChildren = (iSubItem_); TreeList_SetItem((hwndLV), &_ms_tvi); }
// The C leaves the other TV_ITEM members uninitialised; zeroing them is the defined spelling.
#[macro_export]
macro_rules! TreeList_SetItemText {
    ($hwndLV:expr, $hItem:expr, $iSubItem:expr, $pszText:expr $(,)?) => {{
        let mut _ms_tvi: TVITEMW = core::mem::zeroed();
        let _my_pszText: LPWSTR = ($pszText) as LPWSTR;
        _ms_tvi.mask = (TVIF_SUBITEM | TVIF_TEXT) as UINT;
        _ms_tvi.hItem = ($hItem) as HTREEITEM;
        _ms_tvi.stateMask = 0;
        _ms_tvi.pszText = _my_pszText;
        _ms_tvi.cchTextMax = if !_my_pszText.is_null() { wcslen(_my_pszText) as c_int } else { 0 };
        _ms_tvi.cChildren = ($iSubItem) as c_int;
        let _ = TreeList_SetItem!(($hwndLV), core::ptr::addr_of_mut!(_ms_tvi));
    }};
}
pub use crate::TreeList_SetItemText;

// xdk/interlocked.h:244: #define InterlockedOr8 _InterlockedOr8 -- the compiler intrinsic
// `char _InterlockedOr8(char volatile *Destination, char Value)`: an atomic OR returning the
// ORIGINAL value. A real function in C (an intrinsic, not a macro), so a real fn here; the
// generated InterlockedFlagsTestAndSet8 above calls it in function form.
#[inline(always)]
pub unsafe fn InterlockedOr8(Destination: *mut c_char, Value: c_char) -> c_char {
    (*core::sync::atomic::AtomicI8::from_ptr(Destination as *mut i8))
        .fetch_or(Value as i8, core::sync::atomic::Ordering::SeqCst) as c_char
}
// ---- spapisup/fileqsup.c #includes usetup's spapisup/cabinet.c + spapisup/fileqsup.c ----
// base/setup/lib/spapisup/fileqsup.h:22-50: the notification codes and FILEOP_* results the
// included queue code uses that the island and the generated facts above do not carry.
// Typed like their generated siblings (SPFILENOTIFY_* i32, FILEOP_* u32).
pub const SPFILENOTIFY_STARTQUEUE: i32 = 0x1;
pub const SPFILENOTIFY_ENDQUEUE: i32 = 0x2;
pub const SPFILENOTIFY_ENDSUBQUEUE: i32 = 0x4;
pub const SPFILENOTIFY_DELETEERROR: i32 = 0x7;
pub const SPFILENOTIFY_RENAMEERROR: i32 = 0xA;
pub const FILEOP_RETRY: u32 = FILEOP_DOIT;
pub const FILEOP_NEWPATH: u32 = 4;

// base/setup/lib/utils/filesup.h -- setuplib exports (reactos delay-imports setuplib), no
// calling-convention annotation, so cdecl.
extern "C" {
    pub fn SetupDeleteFile(FileName: PCWSTR, ForceDelete: BOOLEAN) -> NTSTATUS;
    pub fn SetupCopyFile(SourceFileName: PCWSTR, DestinationFileName: PCWSTR, FailIfExists: BOOLEAN) -> NTSTATUS;
    pub fn SetupMoveFile(ExistingFileName: PCWSTR, NewFileName: PCWSTR, Flags: ULONG) -> NTSTATUS;
    pub fn ConcatPaths(PathBuffer: PWSTR, cchPathSize: SIZE_T, NumberOfPathComponents: ULONG, ...) -> NTSTATUS;
    pub fn CombinePaths(PathBuffer: PWSTR, cchPathSize: SIZE_T, NumberOfPathComponents: ULONG, ...) -> NTSTATUS;
}

// msvcrt string/stdlib routines the cabinet code calls.
extern "C" {
    pub fn abs(x: c_int) -> c_int;
    pub fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int;
    pub fn wcsncpy(d: *mut u16, s: *const u16, n: size_t) -> *mut u16;
}

// vendor/zlib/include/zlib.h (Z_SOLO, no Z_PREFIX, no ZLIB_WINAPI: cdecl) and zconf.h:287.
// The constants keep usetup's island typing, which the shared cabinet code is written to.
pub const ZLIB_VERSION: &[u8; 6] = b"1.3.2\0";
pub const MAX_WBITS: i32 = 15;
pub const Z_SYNC_FLUSH: u32 = 2;
pub const Z_OK: u32 = 0;
pub const Z_STREAM_END: u32 = 1;
pub const Z_MEM_ERROR: i32 = -4;
pub type z_streamp = *mut z_stream;
extern "C" {
    pub fn inflate(strm: z_streamp, flush: c_int) -> c_int;
    pub fn inflateEnd(strm: z_streamp) -> c_int;
    pub fn inflateInit2_(strm: z_streamp, windowBits: c_int, version: *const c_char, stream_size: c_int) -> c_int;
}
// zlib.h: #define inflateInit2(strm, windowBits) inflateInit2_((strm), (windowBits), ZLIB_VERSION, (int)sizeof(z_stream))
#[inline(always)]
pub unsafe fn inflateInit2(strm: z_streamp, windowBits: c_int) -> c_int {
    inflateInit2_(strm, windowBits, ZLIB_VERSION.as_ptr() as *const c_char, core::mem::size_of::<z_stream>() as c_int)
}
// ==== end hand-transcribed ====
