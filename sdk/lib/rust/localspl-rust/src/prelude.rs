//! localspl-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedDecrement, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, ZeroMemory, _countof, debugstr_a, debugstr_an, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, max, min, wide_bytes};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const DsRolePrimaryDomainInfoBasic: _DSROLE_PRIMARY_DOMAIN_INFO_LEVEL = _DSROLE_PRIMARY_DOMAIN_INFO_LEVEL_DsRolePrimaryDomainInfoBasic; // island enum
pub const DsRole_RoleMemberServer: _DSROLE_MACHINE_ROLE = _DSROLE_MACHINE_ROLE_DsRole_RoleMemberServer; // island enum
pub const DsRole_RoleMemberWorkstation: _DSROLE_MACHINE_ROLE = _DSROLE_MACHINE_ROLE_DsRole_RoleMemberWorkstation; // island enum
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY; // console-rust/src/prelude.rs
pub const HWND_BROADCAST: HWND = 0xffffusize as HWND; // desk-rust/src/prelude.rs
pub const INVALID_FILE_ATTRIBUTES: DWORD = 0xFFFFFFFF; // kernel32-rust/src/file/fileinfo.rs
pub const NameSamCompatible: EXTENDED_NAME_FORMAT = EXTENDED_NAME_FORMAT_NameSamCompatible; // island enum
pub const RPC_S_OK: RPC_STATUS = 0; // advapi32-rust/src/prelude.rs
pub const __WINE_DBCL_ERR: __wine_debug_class = 1; // appwiz-rust/src/prelude.rs
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0; // comctl32-rust/src/prelude.rs
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3; // appwiz-rust/src/prelude.rs
pub const __WINE_DBCL_WARN: __wine_debug_class = 2; // appwiz-rust/src/prelude.rs
extern "C" {
    pub fn DeleteElementSkiplist(Skiplist: *mut SKIPLIST, Element: PVOID,) -> PVOID; // skiplist-rust/src/skiplist.rs
    pub fn InitializeSkiplist(Skiplist: *mut SKIPLIST, AllocateRoutine: PSKIPLIST_ALLOCATE_ROUTINE, CompareRoutine: PSKIPLIST_COMPARE_ROUTINE, FreeRoutine: PSKIPLIST_FREE_ROUTINE,); // skiplist-rust/src/skiplist.rs
    pub fn InsertElementSkiplist(Skiplist: *mut SKIPLIST, Element: PVOID,) -> BOOL; // skiplist-rust/src/skiplist.rs
    pub fn InsertTailElementSkiplist(Skiplist: *mut SKIPLIST, Element: PVOID,) -> BOOL; // skiplist-rust/src/skiplist.rs
    pub fn LookupElementSkiplist(Skiplist: *mut SKIPLIST, Element: PVOID, ElementIndex: *mut DWORD,) -> PVOID; // skiplist-rust/src/skiplist.rs
    pub fn LookupNodeByIndexSkiplist(Skiplist: *mut SKIPLIST, ElementIndex: DWORD,) -> *mut SKIPLIST_NODE; // skiplist-rust/src/skiplist.rs
    pub fn _swprintf(buf: *mut WCHAR, fmt: *const WCHAR, ...) -> c_int; // rtl-actctx/src/shim.rs
    pub fn _ultow(value: u32, string: *mut u16, radix: i32) -> *mut u16; // cnt-rust/src/cabi/generated.rs
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // inflib-rust/src/infcore.rs
    pub fn _wcslwr(s: LPWSTR) -> LPWSTR; // shell32-rust/src/prelude.rs
    pub fn _wcsnicmp(s1: *const u16, s2: *const u16, n: usize) -> i32; // kernel32-rust/src/client/vdm.rs
    pub fn ros_dbg_log( cls: __wine_debug_class, ch: *mut __wine_debug_channel, file: *const c_char, func: *const c_char, line: c_int, msg: *const c_char, ... ) -> c_int; // iphlpapi-rust/src/prelude.rs
    pub fn towlower(c: u16) -> u16; // cnt-ctype/src/lib.rs
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int; // browser-rust/src/prelude.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcsncmp(a: *const u16, b: *const u16, n: usize) -> c_int; // desk-rust/src/prelude.rs
    pub fn wcsrchr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcstoul(s: *const u16, endptr: *mut *mut u16, base: i32) -> u32; // concfg-rust/src/font.rs
    pub fn wsprintfW(arg1: LPWSTR, arg2: LPCWSTR, ...) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
}
extern "system" {
    pub fn CloseHandle(hObject: HANDLE) -> BOOL; // advapi32-rust/src/externs.rs
    pub fn CompareFileTime(arg1: *const FILETIME, arg2: *const FILETIME) -> LONG; // aclui-rust/src/types_amd64.rs
    pub fn CopyFileW( lpExistingFileName: LPCWSTR, lpNewFileName: LPCWSTR, bFailIfExists: BOOL, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn CreateDirectoryW( lpPathName: LPCWSTR, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn CreateThread( arg1: LPSECURITY_ATTRIBUTES, arg2: DWORD, arg3: LPTHREAD_START_ROUTINE, arg4: PVOID, arg5: DWORD, arg6: PDWORD, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn DeleteFileW(arg1: LPCWSTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DsRoleFreeMemory(Buffer: *mut core::ffi::c_void); // netapi32-rust/src/netapi32/dssetup.rs
    pub fn DsRoleGetPrimaryDomainInformation(lpServer: LPCWSTR, InfoLevel: DSROLE_PRIMARY_DOMAIN_INFO_LEVEL, Buffer: *mut PBYTE,) -> DWORD; // netapi32-rust/src/netapi32/dssetup.rs
    pub fn FindClose(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn FindFirstFileW(arg1: LPCWSTR, arg2: LPWIN32_FIND_DATAW) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn FindNextFileW(arg1: HANDLE, arg2: LPWIN32_FIND_DATAW) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetComputerNameExW(arg1: COMPUTER_NAME_FORMAT, arg2: LPWSTR, arg3: LPDWORD) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetComputerNameW(lpBuffer: LPWSTR, nSize: LPDWORD) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetFileSize(arg1: HANDLE, arg2: PDWORD) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_amd64.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn GetSystemDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn GetSystemInfo(arg1: LPSYSTEM_INFO); // aclui-rust/src/types_amd64.rs
    pub fn GetSystemTime(lpSystemTime: LPSYSTEMTIME); // aclui-rust/src/types_amd64.rs
    pub fn GetSystemTimeAsFileTime(arg1: LPFILETIME); // aclui-rust/src/types_amd64.rs
    pub fn GetUserNameExW(arg1: EXTENDED_NAME_FORMAT, arg2: LPWSTR, arg3: PULONG) -> BOOLEAN; // shell32-rust/src/island_amd64.rs
    pub fn GetUserNameW(lpBuffer: LPWSTR, pcbBuffer: LPDWORD) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetVersion() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetVersionExW(arg1: LPOSVERSIONINFOW) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // advapi32-rust/src/wine/security.rs
    pub fn LoadLibraryW(arg1: LPCWSTR) -> HINSTANCE; // aclui-rust/src/types_amd64.rs
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ReadFile( arg1: HANDLE, arg2: PVOID, arg3: DWORD, arg4: PDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegCreateKeyExW( hKey: HKEY, lpSubKey: LPCWSTR, Reserved: DWORD, lpClass: LPWSTR, dwOptions: DWORD, samDesired: REGSAM, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, phkResult: PHKEY, lpdwDisposition: PDWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegCreateKeyW(hKey: HKEY, lpSubKey: LPCWSTR, phkResult: PHKEY) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegDeleteKeyW(hKey: HKEY, lpSubKey: LPCWSTR) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegDeleteTreeW(arg1: HKEY, arg2: LPCWSTR) -> LSTATUS; // shell32-rust/src/island_amd64.rs
    pub fn RegDeleteValueW(arg1: HKEY, arg2: LPCWSTR) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegEnumKeyExW( hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, lpcchName: LPDWORD, lpReserved: LPDWORD, lpClass: LPWSTR, lpcchClass: LPDWORD, lpftLastWriteTime: PFILETIME, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegOpenKeyW(arg1: HKEY, arg2: LPCWSTR, arg3: PHKEY) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegQueryInfoKeyW( hKey: HKEY, lpClass: LPWSTR, lpcchClass: LPDWORD, lpReserved: LPDWORD, lpcSubKeys: LPDWORD, lpcbMaxSubKeyLen: LPDWORD, lpcbMaxClassLen: LPDWORD, lpcValues: LPDWORD, lpcbMaxValueNameLen: LPDWORD, lpcbMaxValueLen: LPDWORD, lpcbSecurityDescriptor: LPDWORD, lpftLastWriteTime: PFILETIME, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegSetValueExW( hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RpcBindingFree(Binding: *mut RPC_BINDING_HANDLE) -> RPC_STATUS; // ole32-rust/src/marshal/types.rs
    pub fn RpcBindingServerFromClient( ClientBinding: RPC_BINDING_HANDLE, ServerBinding: *mut RPC_BINDING_HANDLE, ) -> RPC_STATUS; // syssetup-rust/src/types_amd64.rs
    pub fn RpcBindingToStringBindingW( Binding: RPC_BINDING_HANDLE, StringBinding: *mut RPC_WSTR, ) -> RPC_STATUS; // syssetup-rust/src/types_amd64.rs
    pub fn RpcStringBindingParseW( StringBinding: RPC_WSTR, ObjUuid: *mut RPC_WSTR, Protseq: *mut RPC_WSTR, NetworkAddr: *mut RPC_WSTR, Endpoint: *mut RPC_WSTR, NetworkOptions: *mut RPC_WSTR, ) -> RPC_STATUS; // syssetup-rust/src/types_amd64.rs
    pub fn RpcStringFreeW(String: *mut RPC_WSTR) -> RPC_STATUS; // netcfgx-rust/src/prelude.rs
    pub fn SetLastError(dwErrCode: DWORD); // advapi32-rust/src/sec/misc.rs
    pub fn SystemTimeToFileTime(arg1: *const SYSTEMTIME, arg2: LPFILETIME) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn WriteFile( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn lstrcatW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_amd64.rs
    pub fn lstrcmpiW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn lstrlenW(text: LPCWSTR) -> INT; // menuband-rust/src/enum_merged_folder.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthW, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// RPC_BINDING_HANDLE: c2rust, jobs.c
pub type RPC_BINDING_HANDLE = I_RPC_HANDLE;
// RPC_WSTR: c2rust, jobs.c
pub type RPC_WSTR = *mut ::core::ffi::c_ushort;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn AllocSplStr(pwszInput: PCWSTR) -> PWSTR; // jobs.c
    pub fn DllAllocSplMem(dwBytes: DWORD) -> PVOID; // forms.c
    pub fn DllFreeSplMem(pMem: PVOID) -> BOOL; // forms.c
    pub fn DllFreeSplStr(pwszString: PWSTR) -> BOOL; // jobs.c
    pub fn PackStrings(
        pSource: *mut PCWSTR,
        pDest: PBYTE,
        DestOffsets: *const DWORD,
        pEnd: PBYTE,
    ) -> PBYTE; // forms.c
    pub fn ReallocSplStr(ppwszString: *mut PWSTR, pwszInput: PCWSTR) -> BOOL; // jobs.c
}
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
pub static mut __wine_dbch_localspl: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x6c\x6f\x63\x61\x6c\x73\x70\x6c\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_localspl };
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

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::localspl::tools::AllocAndRegQueryWSZ;
pub use crate::localspl::printers::BroadcastChange;
pub use crate::localspl::tools::DuplicateDevMode;
pub use crate::localspl::printprocessors::FindDatatype;
pub use crate::localspl::printprocessors::FindPrintProcessor;
pub use crate::localspl::ports::InitializePortList;
pub use crate::localspl::monitors::InitializePrintMonitorList;
pub use crate::localspl::printprocessors::InitializePrintProcessorList;
pub use crate::localspl::printerdrivers::InitializePrinterDrivers;
pub use crate::localspl::printers::InitializePrinterList;
pub use crate::localspl::monitors::LocalAddMonitor;
pub use crate::localspl::ports::LocalAddPort;
pub use crate::localspl::ports::LocalAddPortEx;
pub use crate::localspl::printerdrivers::LocalAddPrinterDriver;
pub use crate::localspl::printerdrivers::LocalAddPrinterDriverEx;
pub use crate::localspl::printers::LocalClosePrinter;
pub use crate::localspl::spoolfile::LocalCloseSpoolFileHandle;
pub use crate::localspl::spoolfile::LocalCommitSpoolData;
pub use crate::localspl::ports::LocalConfigurePort;
pub use crate::localspl::monitors::LocalDeleteMonitor;
pub use crate::localspl::ports::LocalDeletePort;
pub use crate::localspl::printers::LocalEndDocPrinter;
pub use crate::localspl::printers::LocalEndPagePrinter;
pub use crate::localspl::monitors::LocalEnumMonitors;
pub use crate::localspl::ports::LocalEnumPorts;
pub use crate::localspl::printprocessors::LocalEnumPrintProcessorDatatypes;
pub use crate::localspl::printprocessors::LocalEnumPrintProcessors;
pub use crate::localspl::printerdrivers::LocalEnumPrinterDrivers;
pub use crate::localspl::printers::LocalEnumPrinters;
pub use crate::localspl::printprocessors::LocalGetPrintProcessorDirectory;
pub use crate::localspl::printers::LocalGetPrinter;
pub use crate::localspl::printerdata::LocalGetPrinterData;
pub use crate::localspl::printerdata::LocalGetPrinterDataEx;
pub use crate::localspl::printerdrivers::LocalGetPrinterDriver;
pub use crate::localspl::printerdrivers::LocalGetPrinterDriverDirectory;
pub use crate::localspl::printerdrivers::LocalGetPrinterDriverEx;
pub use crate::localspl::spoolfile::LocalGetSpoolFileInfo;
pub use crate::localspl::printers::LocalOpenPrinter;
pub use crate::localspl::printers::LocalPrinterMessageBox;
pub use crate::localspl::printers::LocalReadPrinter;
pub use crate::localspl::ports::LocalSetPort;
pub use crate::localspl::printerdata::LocalSetPrinterData;
pub use crate::localspl::printerdata::LocalSetPrinterDataEx;
pub use crate::localspl::printers::LocalStartDocPrinter;
pub use crate::localspl::printers::LocalStartPagePrinter;
pub use crate::localspl::printers::LocalWritePrinter;
pub use crate::localspl::xcv::LocalXcvData;
pub use crate::localspl::printingthread::PrintingThreadProc;
pub use crate::localspl::tools::copy_servername_from_name;
pub use crate::localspl::printerdrivers::validate_envW;
// ==== end cross-TU definitions ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
pub const SPLREG_ALLOW_USER_MANAGEFORMS: *const u16 = (&[0x0041, 0x006c, 0x006c, 0x006f, 0x0077, 0x0055, 0x0073, 0x0065, 0x0072, 0x004d, 0x0061, 0x006e, 0x0061, 0x0067, 0x0065, 0x0046, 0x006f, 0x0072, 0x006d, 0x0073, 0x0000u16]).as_ptr(); // L"AllowUserManageForms"
pub const SPLREG_ARCHITECTURE: *const u16 = (&[0x0041, 0x0072, 0x0063, 0x0068, 0x0069, 0x0074, 0x0065, 0x0063, 0x0074, 0x0075, 0x0072, 0x0065, 0x0000u16]).as_ptr(); // L"Architecture"
pub const SPLREG_BEEP_ENABLED: *const u16 = (&[0x0042, 0x0065, 0x0065, 0x0070, 0x0045, 0x006e, 0x0061, 0x0062, 0x006c, 0x0065, 0x0064, 0x0000u16]).as_ptr(); // L"BeepEnabled"
pub const SPLREG_DEFAULT_SPOOL_DIRECTORY: *const u16 = (&[0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006c, 0x0074, 0x0053, 0x0070, 0x006f, 0x006f, 0x006c, 0x0044, 0x0069, 0x0072, 0x0065, 0x0063, 0x0074, 0x006f, 0x0072, 0x0079, 0x0000u16]).as_ptr(); // L"DefaultSpoolDirectory"
pub const SPLREG_DNS_MACHINE_NAME: *const u16 = (&[0x0044, 0x004e, 0x0053, 0x004d, 0x0061, 0x0063, 0x0068, 0x0069, 0x006e, 0x0065, 0x004e, 0x0061, 0x006d, 0x0065, 0x0000u16]).as_ptr(); // L"DNSMachineName"
pub const SPLREG_DS_PRESENT: *const u16 = (&[0x0044, 0x0073, 0x0050, 0x0072, 0x0065, 0x0073, 0x0065, 0x006e, 0x0074, 0x0000u16]).as_ptr(); // L"DsPresent"
pub const SPLREG_DS_PRESENT_FOR_USER: *const u16 = (&[0x0044, 0x0073, 0x0050, 0x0072, 0x0065, 0x0073, 0x0065, 0x006e, 0x0074, 0x0046, 0x006f, 0x0072, 0x0055, 0x0073, 0x0065, 0x0072, 0x0000u16]).as_ptr(); // L"DsPresentForUser"
pub const SPLREG_EVENT_LOG: *const u16 = (&[0x0045, 0x0076, 0x0065, 0x006e, 0x0074, 0x004c, 0x006f, 0x0067, 0x0000u16]).as_ptr(); // L"EventLog"
pub const SPLREG_MAJOR_VERSION: *const u16 = (&[0x004d, 0x0061, 0x006a, 0x006f, 0x0072, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006f, 0x006e, 0x0000u16]).as_ptr(); // L"MajorVersion"
pub const SPLREG_MINOR_VERSION: *const u16 = (&[0x004d, 0x0069, 0x006e, 0x006f, 0x0072, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006f, 0x006e, 0x0000u16]).as_ptr(); // L"MinorVersion"
pub const SPLREG_NET_POPUP: *const u16 = (&[0x004e, 0x0065, 0x0074, 0x0050, 0x006f, 0x0070, 0x0075, 0x0070, 0x0000u16]).as_ptr(); // L"NetPopup"
pub const SPLREG_NET_POPUP_TO_COMPUTER: *const u16 = (&[0x004e, 0x0065, 0x0074, 0x0050, 0x006f, 0x0070, 0x0075, 0x0070, 0x0054, 0x006f, 0x0043, 0x006f, 0x006d, 0x0070, 0x0075, 0x0074, 0x0065, 0x0072, 0x0000u16]).as_ptr(); // L"NetPopupToComputer"
pub const SPLREG_OS_VERSION: *const u16 = (&[0x004f, 0x0053, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006f, 0x006e, 0x0000u16]).as_ptr(); // L"OSVersion"
pub const SPLREG_OS_VERSIONEX: *const u16 = (&[0x004f, 0x0053, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006f, 0x006e, 0x0045, 0x0078, 0x0000u16]).as_ptr(); // L"OSVersionEx"
pub const SPLREG_PORT_THREAD_PRIORITY: *const u16 = (&[0x0050, 0x006f, 0x0072, 0x0074, 0x0054, 0x0068, 0x0072, 0x0065, 0x0061, 0x0064, 0x0050, 0x0072, 0x0069, 0x006f, 0x0072, 0x0069, 0x0074, 0x0079, 0x0000u16]).as_ptr(); // L"PortThreadPriority"
pub const SPLREG_PORT_THREAD_PRIORITY_DEFAULT: *const u16 = (&[0x0050, 0x006f, 0x0072, 0x0074, 0x0054, 0x0068, 0x0072, 0x0065, 0x0061, 0x0064, 0x0050, 0x0072, 0x0069, 0x006f, 0x0072, 0x0069, 0x0074, 0x0079, 0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006c, 0x0074, 0x0000u16]).as_ptr(); // L"PortThreadPriorityDefault"
pub const SPLREG_REMOTE_FAX: *const u16 = (&[0x0052, 0x0065, 0x006d, 0x006f, 0x0074, 0x0065, 0x0046, 0x0061, 0x0078, 0x0000u16]).as_ptr(); // L"RemoteFax"
pub const SPLREG_RESTART_JOB_ON_POOL_ENABLED: *const u16 = (&[0x0052, 0x0065, 0x0073, 0x0074, 0x0061, 0x0072, 0x0074, 0x004a, 0x006f, 0x0062, 0x004f, 0x006e, 0x0050, 0x006f, 0x006f, 0x006c, 0x0045, 0x006e, 0x0061, 0x0062, 0x006c, 0x0065, 0x0064, 0x0000u16]).as_ptr(); // L"RestartJobOnPoolEnabled"
pub const SPLREG_RESTART_JOB_ON_POOL_ERROR: *const u16 = (&[0x0052, 0x0065, 0x0073, 0x0074, 0x0061, 0x0072, 0x0074, 0x004a, 0x006f, 0x0062, 0x004f, 0x006e, 0x0050, 0x006f, 0x006f, 0x006c, 0x0045, 0x0072, 0x0072, 0x006f, 0x0072, 0x0000u16]).as_ptr(); // L"RestartJobOnPoolError"
pub const SPLREG_RETRY_POPUP: *const u16 = (&[0x0052, 0x0065, 0x0074, 0x0072, 0x0079, 0x0050, 0x006f, 0x0070, 0x0075, 0x0070, 0x0000u16]).as_ptr(); // L"RetryPopup"
pub const SPLREG_SCHEDULER_THREAD_PRIORITY: *const u16 = (&[0x0053, 0x0063, 0x0068, 0x0065, 0x0064, 0x0075, 0x006c, 0x0065, 0x0072, 0x0054, 0x0068, 0x0072, 0x0065, 0x0061, 0x0064, 0x0050, 0x0072, 0x0069, 0x006f, 0x0072, 0x0069, 0x0074, 0x0079, 0x0000u16]).as_ptr(); // L"SchedulerThreadPriority"
pub const SPLREG_SCHEDULER_THREAD_PRIORITY_DEFAULT: *const u16 = (&[0x0053, 0x0063, 0x0068, 0x0065, 0x0064, 0x0075, 0x006c, 0x0065, 0x0072, 0x0054, 0x0068, 0x0072, 0x0065, 0x0061, 0x0064, 0x0050, 0x0072, 0x0069, 0x006f, 0x0072, 0x0069, 0x0074, 0x0079, 0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006c, 0x0074, 0x0000u16]).as_ptr(); // L"SchedulerThreadPriorityDefault"
pub const SPLREG_WEBSHAREMGMT: *const u16 = (&[0x0057, 0x0065, 0x0062, 0x0053, 0x0068, 0x0061, 0x0072, 0x0065, 0x004d, 0x0067, 0x006d, 0x0074, 0x0000u16]).as_ptr(); // L"WebShareMgmt"
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// win32ss/printing/providers/localspl/precomp.h:38: #define IS_VALID_JOB_ID(ID) (ID >= 1 && ID <= 99999)
#[allow(unused_macros)]
macro_rules! IS_VALID_JOB_ID {
    ($id:expr $(,)?) => {
        (($id) >= 1 && ($id) <= 99999)
    };
}
#[allow(unused_imports)]
pub(crate) use IS_VALID_JOB_ID;
// win32ss/printing/providers/localspl/precomp.h:39: #define IS_VALID_PRIORITY(P) (P >= MIN_PRIORITY && P <= MAX_PRIORITY)
#[allow(unused_macros)]
macro_rules! IS_VALID_PRIORITY {
    ($p:expr $(,)?) => {
        (($p) >= MIN_PRIORITY && ($p) <= MAX_PRIORITY)
    };
}
#[allow(unused_imports)]
pub(crate) use IS_VALID_PRIORITY;
// ==== end port macros ====

// ==== module-local (hand-written): nested anonymous enum of struct _LOCAL_HANDLE (precomp.h:200) ====
// bindgen prefixes the members with the struct tag; the C source uses the bare names.
pub use crate::types::_LOCAL_HANDLE_HandleType_Port as HandleType_Port;
pub use crate::types::_LOCAL_HANDLE_HandleType_Printer as HandleType_Printer;
pub use crate::types::_LOCAL_HANDLE_HandleType_PrintServer as HandleType_PrintServer;
pub use crate::types::_LOCAL_HANDLE_HandleType_Xcv as HandleType_Xcv;

pub use strsafe_rust::StringCbCopyExW;
