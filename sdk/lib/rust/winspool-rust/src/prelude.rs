//! winspool-rust prelude — the single import surface the translated modules use.
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

// The marshalling tables (static const in the C headers), flat under their C names.
// NAMED, not globbed: the bindgen island also declares these as `extern` statics (bindgen sees
// the header's `static const`), and an explicit import outranks a glob where two globs clash.
pub use crate::winspool::marshalling_tables::{
    FormInfo1Marshalling,
    FormInfo2Marshalling,
    pFormInfoMarshalling,
    AddJobInfo1Marshalling,
    JobInfo1Marshalling,
    JobInfo2Marshalling,
    pJobInfoMarshalling,
    MonitorInfo1Marshalling,
    MonitorInfo2Marshalling,
    pMonitorInfoMarshalling,
    PortInfo1Marshalling,
    PortInfo2Marshalling,
    pPortInfoMarshalling,
    PrinterDriver1Marshalling,
    PrinterDriver2Marshalling,
    PrinterDriver3Marshalling,
    PrinterDriver4Marshalling,
    PrinterDriver5Marshalling,
    PrinterDriver6Marshalling,
    PrinterDriver8Marshalling,
    pPrinterDriverMarshalling,
    PrinterInfoStressMarshalling,
    PrinterInfo1Marshalling,
    PrinterInfo2Marshalling,
    PrinterInfo3Marshalling,
    PrinterInfo4Marshalling,
    PrinterInfo5Marshalling,
    PrinterInfo6Marshalling,
    PrinterInfo7Marshalling,
    PrinterInfo8Marshalling,
    PrinterInfo9Marshalling,
    pPrinterInfoMarshalling,
    DatatypesInfo1Marshalling,
    PrintProcessorInfo1Marshalling,
};

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const CP_ACP: u32 = 0; // advapi32-sys/src/bindgen_amd64.rs
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001usize as HKEY; // access-rust/src/prelude.rs
pub const INVALID_FILE_ATTRIBUTES: DWORD = 0xFFFFFFFF; // kernel32-rust/src/file/fileinfo.rs
pub const RPC_S_OK: RPC_STATUS = 0; // advapi32-rust/src/prelude.rs
pub const __WINE_DBCL_ERR: __wine_debug_class = 1; // appwiz-rust/src/prelude.rs
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0; // comctl32-rust/src/prelude.rs
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3; // appwiz-rust/src/prelude.rs
pub const __WINE_DBCL_WARN: __wine_debug_class = 2; // appwiz-rust/src/prelude.rs
extern "C" {
    pub fn FIXME(format: *const i8, ...); // cabinet-rust/src/cabinet/fci.rs
    pub fn _SEH2_END(); // cdfs-rust/src/cdmacros.rs
    pub fn _SEH2_TRY(); // cdfs-rust/src/cdmacros.rs
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // inflib-rust/src/infcore.rs
    pub fn ros_dbg_log( cls: __wine_debug_class, ch: *mut __wine_debug_channel, file: *const c_char, func: *const c_char, line: c_int, msg: *const c_char, ... ) -> c_int; // iphlpapi-rust/src/prelude.rs
    pub fn strlen(s: *const i8) -> usize; // advapi32-rust/src/prelude.rs
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
}
extern "system" {
    pub fn CloseHandle(hObject: HANDLE) -> BOOL; // advapi32-rust/src/externs.rs
    pub fn CreateEventW( arg1: LPSECURITY_ATTRIBUTES, bManualReset: BOOL, bInitialState: BOOL, lpName: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn CreateThread( arg1: LPSECURITY_ATTRIBUTES, arg2: DWORD, arg3: LPTHREAD_START_ROUTINE, arg4: PVOID, arg5: DWORD, arg6: PDWORD, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn DeactivateActCtx(arg1: DWORD, arg2: ULONG_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DeleteCriticalSection(arg1: PCRITICAL_SECTION); // aclui-rust/src/types_amd64.rs
    pub fn DeleteFileW(arg1: LPCWSTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DialogBoxParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> INT_PTR; // aclui-rust/src/types_amd64.rs
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DispatchMessageW(arg1: *const MSG) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_amd64.rs
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GdiConvertToDevmodeW(arg1: *const DEVMODEA) -> *mut DEVMODEW; // aclui-rust/src/types_amd64.rs
    pub fn GetCurrentProcessId() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetDlgItemTextW( hDlg: HWND, nIDDlgItem: ::core::ffi::c_int, lpString: LPWSTR, cchMax: ::core::ffi::c_int, ) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetForegroundWindow() -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetFullPathNameW(arg1: LPCWSTR, arg2: DWORD, arg3: LPWSTR, arg4: *mut LPWSTR) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_amd64.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn GetSecurityDescriptorDacl( arg1: PSECURITY_DESCRIPTOR, arg2: LPBOOL, arg3: *mut PACL, arg4: LPBOOL, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetSecurityDescriptorGroup( arg1: PSECURITY_DESCRIPTOR, arg2: *mut PSID, arg3: LPBOOL, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetSecurityDescriptorLength(arg1: PSECURITY_DESCRIPTOR) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetSecurityDescriptorOwner( arg1: PSECURITY_DESCRIPTOR, arg2: *mut PSID, arg3: LPBOOL, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetSecurityDescriptorSacl( arg1: PSECURITY_DESCRIPTOR, arg2: LPBOOL, arg3: *mut PACL, arg4: LPBOOL, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetSystemDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_amd64.rs
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR; // shell32-rust/src/copy_to_menu.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // advapi32-rust/src/wine/security.rs
    pub fn HeapReAlloc(arg1: HANDLE, arg2: DWORD, arg3: PVOID, arg4: SIZE_T) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn InitializeCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_amd64.rs
    pub fn InitializeSecurityDescriptor(arg1: PSECURITY_DESCRIPTOR, arg2: DWORD) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsValidSecurityDescriptor(arg1: PSECURITY_DESCRIPTOR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_amd64.rs
    pub fn LoadLibraryA(arg1: LPCSTR) -> HINSTANCE; // aclui-rust/src/types_amd64.rs
    pub fn LoadLibraryW(arg1: LPCWSTR) -> HINSTANCE; // aclui-rust/src/types_amd64.rs
    pub fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT; // shell32-rust/src/cfs_folder.rs
    pub fn MakeSelfRelativeSD( arg1: PSECURITY_DESCRIPTOR, arg2: PSECURITY_DESCRIPTOR, arg3: PDWORD, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn MessageBoxW( hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn MsgWaitForMultipleObjects( nCount: DWORD, pHandles: *const HANDLE, fWaitAll: BOOL, dwMilliseconds: DWORD, dwWakeMask: DWORD, ) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn MultiByteToWideChar( arg1: UINT, arg2: DWORD, arg3: LPCSTR, arg4: ::core::ffi::c_int, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_amd64.rs
    pub fn PeekMessageW(arg1: LPMSG, arg2: HWND, arg3: UINT, arg4: UINT, arg5: UINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ProcessIdToSessionId(arg1: DWORD, arg2: *mut DWORD) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegEnumValueW( hKey: HKEY, dwIndex: DWORD, lpValueName: LPWSTR, lpcchValueName: LPDWORD, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegSetValueExW( hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn ReleaseActCtx(arg1: HANDLE); // aclui-rust/src/types_amd64.rs
    pub fn RpcBindingFree(Binding: *mut RPC_BINDING_HANDLE) -> RPC_STATUS; // ole32-rust/src/marshal/types.rs
    pub fn RpcBindingFromStringBindingW( StringBinding: RPC_WSTR, Binding: *mut RPC_BINDING_HANDLE, ) -> RPC_STATUS; // syssetup-rust/src/types_amd64.rs
    pub fn RpcStringBindingComposeW( ObjUuid: RPC_WSTR, Protseq: RPC_WSTR, NetworkAddr: RPC_WSTR, Endpoint: RPC_WSTR, Options: RPC_WSTR, StringBinding: *mut RPC_WSTR, ) -> RPC_STATUS; // syssetup-rust/src/types_amd64.rs
    pub fn RpcStringFreeW(String: *mut RPC_WSTR) -> RPC_STATUS; // netcfgx-rust/src/prelude.rs
    pub fn SendDlgItemMessageW( arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SetEvent(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetFocus(arg1: HWND) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn SetForegroundWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetLastError(dwErrCode: DWORD); // advapi32-rust/src/sec/misc.rs
    pub fn SetSecurityDescriptorDacl( arg1: PSECURITY_DESCRIPTOR, arg2: BOOL, arg3: PACL, arg4: BOOL, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetSecurityDescriptorGroup(arg1: PSECURITY_DESCRIPTOR, arg2: PSID, arg3: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetSecurityDescriptorOwner(arg1: PSECURITY_DESCRIPTOR, arg2: PSID, arg3: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetSecurityDescriptorSacl( arg1: PSECURITY_DESCRIPTOR, arg2: BOOL, arg3: PACL, arg4: BOOL, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR; // shell32-rust/src/copy_to_menu.rs
    pub fn TranslateMessage(arg1: *const MSG) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn WideCharToMultiByte( arg1: UINT, arg2: DWORD, arg3: LPCWSTR, arg4: ::core::ffi::c_int, arg5: LPSTR, arg6: ::core::ffi::c_int, arg7: LPCSTR, arg8: LPBOOL, ) -> ::core::ffi::c_int; // desk-rust/src/types_amd64.rs
    pub fn WriteFile( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn lstrlenA(arg1: LPCSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn lstrlenW(text: LPCWSTR) -> INT; // menuband-rust/src/enum_merged_folder.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::winspool::printers::ClosePrinter;
pub use crate::winspool::printers::GetPrinterW;
pub use crate::winspool::utils::IntProtectHandle;
pub use crate::winspool::utils::IntUnprotectHandle;
pub use crate::winspool::marshalling::MarshallUpStructuresArray;
pub use crate::winspool::printers::OpenPrinterA;
pub use crate::winspool::printers::OpenPrinterW;
pub use crate::winspool::main::hProcessHeap;
pub use crate::winspool::main::rtlCritSec;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthW, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const SPOOLER_HANDLE_SIG: i32 = 0x6767; // clang, printers.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// MINIMUM_SIZE_TABLE: c2rust, devmode.c
pub type MINIMUM_SIZE_TABLE = _MINIMUM_SIZE_TABLE;
// _MINIMUM_SIZE_TABLE: c2rust, devmode.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _MINIMUM_SIZE_TABLE {
    pub dwField: DWORD,
    pub wSize: WORD,
}
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _RpcAbortPrinter(hPrinter: WINSPOOL_PRINTER_HANDLE) -> DWORD; // printers.c
    pub fn _RpcAddForm(
        hPrinter: WINSPOOL_PRINTER_HANDLE,
        pFormInfoContainer: *mut WINSPOOL_FORM_CONTAINER,
    ) -> DWORD; // forms.c
    pub fn _RpcAddJob(
        hPrinter: WINSPOOL_PRINTER_HANDLE,
        Level: DWORD,
        pAddJob: *mut BYTE,
        cbBuf: DWORD,
        pcbNeeded: *mut DWORD,
    ) -> DWORD; // jobs.c
    pub fn _RpcAddMonitor(
        pName: WINSPOOL_HANDLE,
        pMonitorContainer: *mut WINSPOOL_MONITOR_CONTAINER,
    ) -> DWORD; // monitors.c
    pub fn _RpcAddPrintProcessor(
        pName: WINSPOOL_HANDLE,
        pEnvironment: *mut WCHAR,
        pPathName: *mut WCHAR,
        pPrintProcessorName: *mut WCHAR,
    ) -> DWORD; // printprocessors.c
    pub fn _RpcAddPrintProvidor(
        pName: WINSPOOL_HANDLE,
        pProvidorContainer: *mut WINSPOOL_PROVIDOR_CONTAINER,
    ) -> DWORD; // printproviders.c
    pub fn _RpcAddPrinter(
        pName: WINSPOOL_HANDLE,
        pPrinterContainer: *mut WINSPOOL_PRINTER_CONTAINER,
        pDevModeContainer: *mut WINSPOOL_DEVMODE_CONTAINER,
        pSecurityContainer: *mut WINSPOOL_SECURITY_CONTAINER,
        pHandle: *mut WINSPOOL_PRINTER_HANDLE,
    ) -> DWORD; // printers.c
    pub fn _RpcAddPrinterDriverEx(
        pName: WINSPOOL_HANDLE,
        pDriverContainer: *mut WINSPOOL_DRIVER_CONTAINER,
        dwFileCopyFlags: DWORD,
    ) -> DWORD; // printerdrivers.c
    pub fn _RpcCloseSpoolFileHandle(hPrinter: WINSPOOL_PRINTER_HANDLE) -> DWORD; // spoolfile.c
    pub fn _RpcCommitSpoolData2(
        hPrinter: WINSPOOL_PRINTER_HANDLE,
        dwProcessId: DWORD,
        cbCommit: DWORD,
        Level: DWORD,
        pFileInfoContainer: *mut WINSPOOL_FILE_INFO_CONTAINER,
    ) -> DWORD; // spoolfile.c
    pub fn _RpcDeleteForm(hPrinter: WINSPOOL_PRINTER_HANDLE, pFormName: *mut WCHAR) -> DWORD; // forms.c
    pub fn _RpcDeleteMonitor(
        pName: WINSPOOL_HANDLE,
        pEnvironment: *mut WCHAR,
        pMonitorName: *mut WCHAR,
    ) -> DWORD; // monitors.c
    pub fn _RpcDeletePrintProcessor(
        pName: WINSPOOL_HANDLE,
        pEnvironment: *mut WCHAR,
        pPrintProcessorName: *mut WCHAR,
    ) -> DWORD; // printprocessors.c
    pub fn _RpcDeletePrintProvidor(
        pName: WINSPOOL_HANDLE,
        pEnvironment: *mut WCHAR,
        pPrintProviderName: *mut WCHAR,
    ) -> DWORD; // printproviders.c
    pub fn _RpcDeletePrinterDriverEx(
        pName: WINSPOOL_HANDLE,
        pEnvironment: *mut WCHAR,
        pDriverName: *mut WCHAR,
        dwDeleteFlag: DWORD,
        dwVersionNum: DWORD,
    ) -> DWORD; // printerdrivers.c
    pub fn _RpcEndDocPrinter(hPrinter: WINSPOOL_PRINTER_HANDLE) -> DWORD; // printers.c
    pub fn _RpcEndPagePrinter(hPrinter: WINSPOOL_PRINTER_HANDLE) -> DWORD; // printers.c
    pub fn _RpcEnumForms(
        hPrinter: WINSPOOL_PRINTER_HANDLE,
        Level: DWORD,
        pForm: *mut BYTE,
        cbBuf: DWORD,
        pcbNeeded: *mut DWORD,
        pcReturned: *mut DWORD,
    ) -> DWORD; // forms.c
    pub fn _RpcEnumJobs(
        hPrinter: WINSPOOL_PRINTER_HANDLE,
        FirstJob: DWORD,
        NoJobs: DWORD,
        Level: DWORD,
        pJob: *mut BYTE,
        cbBuf: DWORD,
        pcbNeeded: *mut DWORD,
        pcReturned: *mut DWORD,
    ) -> DWORD; // jobs.c
    pub fn _RpcEnumMonitors(
        pName: WINSPOOL_HANDLE,
        Level: DWORD,
        pMonitor: *mut BYTE,
        cbBuf: DWORD,
        pcbNeeded: *mut DWORD,
        pcReturned: *mut DWORD,
    ) -> DWORD; // monitors.c
    pub fn _RpcEnumPrintProcessorDatatypes(
        pName: WINSPOOL_HANDLE,
        pPrintProcessorName: *mut WCHAR,
        Level: DWORD,
        pDatatypes: *mut BYTE,
        cbBuf: DWORD,
        pcbNeeded: *mut DWORD,
        pcReturned: *mut DWORD,
    ) -> DWORD; // printprocessors.c
    pub fn _RpcEnumPrintProcessors(
        pName: WINSPOOL_HANDLE,
        pEnvironment: *mut WCHAR,
        Level: DWORD,
        pPrintProcessorInfo: *mut BYTE,
        cbBuf: DWORD,
        pcbNeeded: *mut DWORD,
        pcReturned: *mut DWORD,
    ) -> DWORD; // printprocessors.c
    pub fn _RpcEnumPrinterDrivers(
        pName: WINSPOOL_HANDLE,
        pEnvironment: *mut WCHAR,
        Level: DWORD,
        pDrivers: *mut BYTE,
        cbBuf: DWORD,
        pcbNeeded: *mut DWORD,
        pcReturned: *mut DWORD,
    ) -> DWORD; // printerdrivers.c
    pub fn _RpcEnumPrinters(
        Flags: DWORD,
        Name: WINSPOOL_HANDLE,
        Level: DWORD,
        pPrinterEnum: *mut BYTE,
        cbBuf: DWORD,
        pcbNeeded: *mut DWORD,
        pcReturned: *mut DWORD,
    ) -> DWORD; // printers.c
    pub fn _RpcGetForm(
        hPrinter: WINSPOOL_PRINTER_HANDLE,
        pFormName: *mut WCHAR,
        Level: DWORD,
        pForm: *mut BYTE,
        cbBuf: DWORD,
        pcbNeeded: *mut DWORD,
    ) -> DWORD; // forms.c
    pub fn _RpcGetJob(
        hPrinter: WINSPOOL_PRINTER_HANDLE,
        JobId: DWORD,
        Level: DWORD,
        pJob: *mut BYTE,
        cbBuf: DWORD,
        pcbNeeded: *mut DWORD,
    ) -> DWORD; // jobs.c
    pub fn _RpcGetPrintProcessorDirectory(
        pName: WINSPOOL_HANDLE,
        pEnvironment: *mut WCHAR,
        Level: DWORD,
        pPrintProcessorDirectory: *mut BYTE,
        cbBuf: DWORD,
        pcbNeeded: *mut DWORD,
    ) -> DWORD; // printprocessors.c
    pub fn _RpcGetPrinter(
        hPrinter: WINSPOOL_PRINTER_HANDLE,
        Level: DWORD,
        pPrinter: *mut BYTE,
        cbBuf: DWORD,
        pcbNeeded: *mut DWORD,
    ) -> DWORD; // printers.c
    pub fn _RpcGetPrinterDataEx(
        hPrinter: WINSPOOL_PRINTER_HANDLE,
        pKeyName: *const WCHAR,
        pValueName: *const WCHAR,
        pType: *mut DWORD,
        pData: *mut BYTE,
        nSize: DWORD,
        pcbNeeded: *mut DWORD,
    ) -> DWORD; // printerdata.c
    pub fn _RpcGetPrinterDriver(
        hPrinter: WINSPOOL_PRINTER_HANDLE,
        pEnvironment: *mut WCHAR,
        Level: DWORD,
        pDriver: *mut BYTE,
        cbBuf: DWORD,
        pcbNeeded: *mut DWORD,
    ) -> DWORD; // printerdrivers.c
    pub fn _RpcGetPrinterDriverDirectory(
        pName: WINSPOOL_HANDLE,
        pEnvironment: *mut WCHAR,
        Level: DWORD,
        pDriverDirectory: *mut BYTE,
        cbBuf: DWORD,
        pcbNeeded: *mut DWORD,
    ) -> DWORD; // printerdrivers.c
    pub fn _RpcGetSpoolFileInfo2(
        hPrinter: WINSPOOL_PRINTER_HANDLE,
        dwProcessId: DWORD,
        Level: DWORD,
        pFileInfoContainer: *mut WINSPOOL_FILE_INFO_CONTAINER,
    ) -> DWORD; // spoolfile.c
    pub fn _RpcOpenPrinter(
        pPrinterName: WINSPOOL_HANDLE,
        phPrinter: *mut WINSPOOL_PRINTER_HANDLE,
        pDatatype: *mut WCHAR,
        pDevModeContainer: *mut WINSPOOL_DEVMODE_CONTAINER,
        AccessRequired: DWORD,
    ) -> DWORD; // printers.c
    pub fn _RpcReadPrinter(
        hPrinter: WINSPOOL_PRINTER_HANDLE,
        pBuf: *mut BYTE,
        cbBuf: DWORD,
        pcNoBytesRead: *mut DWORD,
    ) -> DWORD; // printers.c
    pub fn _RpcScheduleJob(hPrinter: WINSPOOL_PRINTER_HANDLE, JobId: DWORD) -> DWORD; // jobs.c
    pub fn _RpcSeekPrinter(
        hPrinter: WINSPOOL_PRINTER_HANDLE,
        liDistanceToMove: LARGE_INTEGER,
        pliNewPointer: PLARGE_INTEGER,
        dwMoveMethod: DWORD,
        bWrite: BOOL,
    ) -> DWORD; // printers.c
    pub fn _RpcSetForm(
        hPrinter: WINSPOOL_PRINTER_HANDLE,
        pFormName: *mut WCHAR,
        pFormInfoContainer: *mut WINSPOOL_FORM_CONTAINER,
    ) -> DWORD; // forms.c
    pub fn _RpcSetJob(
        hPrinter: WINSPOOL_PRINTER_HANDLE,
        JobId: DWORD,
        pJobContainer: *mut WINSPOOL_JOB_CONTAINER,
        Command: DWORD,
    ) -> DWORD; // jobs.c
    pub fn _RpcSetPrinter(
        hPrinter: WINSPOOL_PRINTER_HANDLE,
        pPrinterContainer: *mut WINSPOOL_PRINTER_CONTAINER,
        pDevModeContainer: *mut WINSPOOL_DEVMODE_CONTAINER,
        pSecurityContainer: *mut WINSPOOL_SECURITY_CONTAINER,
        Command: DWORD,
    ) -> DWORD; // printers.c
    pub fn _RpcSetPrinterDataEx(
        hPrinter: WINSPOOL_PRINTER_HANDLE,
        pKeyName: *const WCHAR,
        pValueName: *const WCHAR,
        Type: DWORD,
        pData: *mut BYTE,
        cbData: DWORD,
    ) -> DWORD; // printerdata.c
    pub fn _RpcSpoolerInit() -> DWORD; // main.c
    pub fn _RpcStartDocPrinter(
        hPrinter: WINSPOOL_PRINTER_HANDLE,
        pDocInfoContainer: *mut WINSPOOL_DOC_INFO_CONTAINER,
        pJobId: *mut DWORD,
    ) -> DWORD; // printers.c
    pub fn _RpcStartPagePrinter(hPrinter: WINSPOOL_PRINTER_HANDLE) -> DWORD; // printers.c
    pub fn _RpcWritePrinter(
        hPrinter: WINSPOOL_PRINTER_HANDLE,
        pBuf: *mut BYTE,
        cbBuf: DWORD,
        pcWritten: *mut DWORD,
    ) -> DWORD; // printers.c
    pub fn _RpcXcvData(
        hXcv: WINSPOOL_PRINTER_HANDLE,
        pszDataName: *const WCHAR,
        pInputData: *mut BYTE,
        cbInputData: DWORD,
        pOutputData: *mut BYTE,
        cbOutputData: DWORD,
        pcbOutputNeeded: *mut DWORD,
        pdwStatus: *mut DWORD,
    ) -> DWORD; // printers.c
}
pub use crate::prelude::LoadLibraryW as LoadLibrary; // sdk alias
pub use crate::prelude::_SEH2_TRY as RpcTryExcept; // sdk alias
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
pub static mut __wine_dbch_winspool: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x77\x69\x6e\x73\x70\x6f\x6f\x6c\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_winspool };
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
#[macro_export] macro_rules! HRESULT_CODE { ($p0:expr $(,)?) => { ($p0 & 0xFFFF) }; }
#[macro_export] macro_rules! RpcExceptionCode { () => { _SEH2_VolatileExceptionCode }; }
#[macro_export] macro_rules! va_arg { ($p0:expr, $p1:expr $(,)?) => { __builtin_va_arg((($p0) as _), (($p1) as _)) }; }
#[macro_export] macro_rules! va_end { ($p0:expr $(,)?) => { __builtin_va_end((($p0) as _)) }; }
#[macro_export] macro_rules! va_start { ($p0:expr, $p1:expr $(,)?) => { __builtin_va_start((($p0) as _), (($p1) as _)) }; }
pub const SPLREG_OS_VERSION: *const u16 = (&[0x004f, 0x0053, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006f, 0x006e, 0x0000u16]).as_ptr(); // L"OSVersion"
pub const SPLREG_OS_VERSIONEX: *const u16 = (&[0x004f, 0x0053, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006f, 0x006e, 0x0045, 0x0078, 0x0000u16]).as_ptr(); // L"OSVersionEx"
pub use crate::{HRESULT_CODE, RpcExceptionCode, va_arg, va_end, va_start};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// win32ss/printing/base/winspool/precomp.h:89: header-inline `AsciiToUnicode`
#[allow(dead_code)]
#[inline]
pub unsafe fn AsciiToUnicode(usBufferPtr: *mut UNICODE_STRING, src: LPCSTR) -> PWSTR {
    if !src.is_null() {
        RtlCreateUnicodeStringFromAsciiz(usBufferPtr, src);
        return (*usBufferPtr).Buffer;
    }
    (*usBufferPtr).Buffer = core::ptr::null_mut(); /* so that RtlFreeUnicodeString won't barf */
    core::ptr::null_mut()
}
// ==== end port macros ====

// ==== winspool local: prtprocenv.h (per-arch current environment) ====
// win32ss/printing/include/prtprocenv.h: the string differs per architecture, and
// cbCurrentEnvironment = sizeof() in bytes INCLUDING the NUL. Both are non-static
// `const` globals in C, so they are exported unmangled.
const fn env_wide<const N: usize>(s: &[u8; N]) -> [u16; N] {
    let mut out = [0u16; N];
    let mut i = 0;
    while i < N {
        out[i] = s[i] as u16;
        i += 1;
    }
    out
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub static wszCurrentEnvironment: [u16; 15] = env_wide(b"Windows NT x86\0");
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub static wszCurrentEnvironment: [u16; 12] = env_wide(b"Windows x64\0");
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub static wszCurrentEnvironment: [u16; 14] = env_wide(b"Windows ARM64\0");

#[no_mangle]
pub static cbCurrentEnvironment: DWORD = core::mem::size_of_val(&wszCurrentEnvironment) as DWORD;
// ==== end winspool local ====

// ==== winspool local: strsafe.h StringCchCopyA (absent from strsafe-rust) ====
// Ported from sdk/include/psdk/strsafe.h:145 + StringCopyWorkerA:893. Candidate for the
// shared strsafe-rust crate.
pub const STRSAFE_E_INSUFFICIENT_BUFFER: HRESULT = 0x8007_007Au32 as HRESULT;
pub const STRSAFE_E_INVALID_PARAMETER: HRESULT = 0x8007_0057u32 as HRESULT;

unsafe fn StringCopyWorkerA(mut pszDest: *mut c_char, mut cchDest: usize, mut pszSrc: *const c_char) -> HRESULT {
    let mut hr: HRESULT = 0; // S_OK
    if cchDest == 0 {
        hr = STRSAFE_E_INVALID_PARAMETER;
    } else {
        while cchDest != 0 && *pszSrc != 0 {
            *pszDest = *pszSrc;
            pszDest = pszDest.add(1);
            pszSrc = pszSrc.add(1);
            cchDest -= 1;
        }
        if cchDest == 0 {
            pszDest = pszDest.sub(1);
            hr = STRSAFE_E_INSUFFICIENT_BUFFER;
        }
        *pszDest = 0;
    }
    hr
}

pub unsafe fn StringCchCopyA(pszDest: *mut c_char, cchDest: usize, pszSrc: *const c_char) -> HRESULT {
    if cchDest > STRSAFE_MAX_CCH as usize {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    StringCopyWorkerA(pszDest, cchDest, pszSrc)
}
// ==== end winspool local strsafe ====

// ==== winspool local: wine debug channels ====
// reactos/wine/debug.h ERR/FIXME print through DbgPrint in this DBG=1 tree, same as DPRINT1.
#[macro_export] macro_rules! ERR { ($($arg:tt)*) => { ::win32macros_rust::DPRINT1!($($arg)*) }; }
#[macro_export] macro_rules! FIXME { ($($arg:tt)*) => { ::win32macros_rust::DPRINT1!($($arg)*) }; }
pub use crate::{ERR, FIXME};
// ==== end winspool local debug ====

// ==== winspool local: MIDL stubs the resolve pass missed ====
// Exact prototypes from build-<arch>/win32ss/printing/base/winspool/winspool_c.h (all __cdecl).
extern "C" {
    pub fn _RpcAddPort(pName: WINSPOOL_HANDLE, hWnd: ULONG_PTR, pMonitorName: *mut WCHAR) -> DWORD;
    pub fn _RpcConfigurePort(pName: WINSPOOL_HANDLE, hWnd: ULONG_PTR, pPortName: *mut WCHAR) -> DWORD;
    pub fn _RpcDeletePort(pName: WINSPOOL_HANDLE, hWnd: ULONG_PTR, pPortName: *mut WCHAR) -> DWORD;
    pub fn _RpcEnumPorts(pName: WINSPOOL_HANDLE, Level: DWORD, pPort: *mut BYTE, cbBuf: DWORD, pcbNeeded: *mut DWORD, pcReturned: *mut DWORD) -> DWORD;
    pub fn _RpcAddPortEx(pName: WINSPOOL_HANDLE, pPortContainer: *mut WINSPOOL_PORT_CONTAINER, pPortVarContainer: *mut WINSPOOL_PORT_VAR_CONTAINER, pMonitorName: *mut WCHAR) -> DWORD;
    pub fn _RpcSetPort(pName: WINSPOOL_HANDLE, pPortName: *mut WCHAR, pPortContainer: *mut WINSPOOL_PORT_CONTAINER) -> DWORD;
}
// ==== end winspool local MIDL stubs ====

// ==== winspool local: BOOL constants ====
// ntdll-sys's bindgen TRUE/FALSE are u32; C's are int. The C compares/assigns them as BOOL.
pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;
// ==== end winspool local BOOL ====
