//! spoolsv-rust prelude — the single import surface the translated modules use.
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
pub const RPC_S_OK: RPC_STATUS = 0; // advapi32-rust/src/prelude.rs
pub const __WINE_DBCL_ERR: __wine_debug_class = 1; // appwiz-rust/src/prelude.rs
extern "C" {
    pub fn ros_dbg_log( cls: __wine_debug_class, ch: *mut __wine_debug_channel, file: *const c_char, func: *const c_char, line: c_int, msg: *const c_char, ... ) -> c_int; // iphlpapi-rust/src/prelude.rs
}
extern "system" {
    pub fn CloseHandle(hObject: HANDLE) -> BOOL; // advapi32-rust/src/externs.rs
    pub fn CreateThread( arg1: LPSECURITY_ATTRIBUTES, arg2: DWORD, arg3: LPTHREAD_START_ROUTINE, arg4: PVOID, arg5: DWORD, arg6: PDWORD, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn EnumPrintersW( flags: DWORD, name: LPWSTR, level: DWORD, buffer: LPBYTE, buffer_size: DWORD, needed: *mut DWORD, returned: *mut DWORD, ) -> BOOL; // shell32-rust/src/printers_enum.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // advapi32-rust/src/wine/security.rs
    pub fn OpenProcess(arg1: DWORD, arg2: BOOL, arg3: DWORD) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn RegisterServiceCtrlHandlerExW( arg1: LPCWSTR, arg2: LPHANDLER_FUNCTION_EX, arg3: LPVOID, ) -> SERVICE_STATUS_HANDLE; // syssetup-rust/src/types_amd64.rs
    pub fn RpcImpersonateClient(Binding: RPC_BINDING_HANDLE) -> RPC_STATUS; // syssetup-rust/src/types_amd64.rs
    pub fn RpcMgmtStopServerListening(Binding: RPC_BINDING_HANDLE) -> RPC_STATUS; // browser-rust/src/prelude.rs
    pub fn RpcRevertToSelf() -> RPC_STATUS; // syssetup-rust/src/types_amd64.rs
    pub fn RpcServerListen( MinimumCallThreads: ::core::ffi::c_uint, MaxCalls: ::core::ffi::c_uint, DontWait: ::core::ffi::c_uint, ) -> RPC_STATUS; // syssetup-rust/src/types_amd64.rs
    pub fn RpcServerRegisterIf( IfSpec: RPC_IF_HANDLE, MgrTypeUuid: *mut UUID, MgrEpv: *mut ::core::ffi::c_void, ) -> RPC_STATUS; // syssetup-rust/src/types_amd64.rs
    pub fn RpcServerUseProtseqEpW( Protseq: RPC_WSTR, MaxCalls: ::core::ffi::c_uint, Endpoint: RPC_WSTR, SecurityDescriptor: *mut ::core::ffi::c_void, ) -> RPC_STATUS; // syssetup-rust/src/types_amd64.rs
    pub fn SetServiceStatus(arg1: SERVICE_STATUS_HANDLE, arg2: LPSERVICE_STATUS) -> BOOL; // syssetup-rust/src/types_amd64.rs
    pub fn StartServiceCtrlDispatcherW(arg1: *const SERVICE_TABLE_ENTRYW) -> BOOL; // syssetup-rust/src/types_amd64.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::spoolsv::rpcserver::LrpcThreadProc;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthW, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn AbortPrinter(hPrinter: HANDLE) -> WINBOOL; // printers.c
    pub fn AddFormW(hPrinter: HANDLE, Level: DWORD, pForm: LPBYTE) -> WINBOOL; // forms.c
    pub fn AddJobW(
        hPrinter: HANDLE,
        Level: DWORD,
        pData: LPBYTE,
        cbBuf: DWORD,
        pcbNeeded: LPDWORD,
    ) -> WINBOOL; // jobs.c
    pub fn AddMonitorW(pName: LPWSTR, Level: DWORD, pMonitorInfo: LPBYTE) -> WINBOOL; // monitors.c
    pub fn AddPortExW(_: LPWSTR, _: DWORD, _: LPBYTE, _: LPWSTR) -> BOOL; // ports.c
    pub fn AddPortW(pName: LPWSTR, hWnd: HWND, pMonitorName: LPWSTR) -> WINBOOL; // ports.c
    pub fn AddPrintProcessorW(
        pName: LPWSTR,
        pEnvironment: LPWSTR,
        pPathName: LPWSTR,
        pPrintProcessorName: LPWSTR,
    ) -> WINBOOL; // printprocessors.c
    pub fn AddPrintProvidorW(pName: LPWSTR, level: DWORD, pProvidorInfo: LPBYTE) -> WINBOOL; // printproviders.c
    pub fn AddPrinterDriverExW(
        pName: LPWSTR,
        Level: DWORD,
        pDriverInfo: LPBYTE,
        dwFileCopyFlags: DWORD,
    ) -> WINBOOL; // printerdrivers.c
    pub fn AddPrinterDriverW(pName: LPWSTR, Level: DWORD, pDriverInfo: LPBYTE) -> WINBOOL; // printerdrivers.c
    pub fn AlignRpcPtr(pBuffer: PVOID, pcbBuffer: PDWORD) -> PVOID; // forms.c
    pub fn ClosePrinter(hPrinter: HANDLE) -> WINBOOL; // printers.c
    pub fn ConfigurePortW(pName: LPWSTR, hWnd: HWND, pPortName: LPWSTR) -> WINBOOL; // ports.c
    pub fn DeleteFormW(hPrinter: HANDLE, pFormName: LPWSTR) -> WINBOOL; // forms.c
    pub fn DeleteMonitorW(pName: LPWSTR, pEnvironment: LPWSTR, pMonitorName: LPWSTR) -> WINBOOL; // monitors.c
    pub fn DeletePortW(pName: LPWSTR, hWnd: HWND, pPortName: LPWSTR) -> WINBOOL; // ports.c
    pub fn DeletePrintProcessorW(
        pName: LPWSTR,
        pEnvironment: LPWSTR,
        pPrintProcessorName: LPWSTR,
    ) -> WINBOOL; // printprocessors.c
    pub fn DeletePrintProvidorW(
        pName: LPWSTR,
        pEnvironment: LPWSTR,
        pPrintProvidorName: LPWSTR,
    ) -> WINBOOL; // printproviders.c
    pub fn DeletePrinter(hPrinter: HANDLE) -> WINBOOL; // printers.c
    pub fn DeletePrinterDriverExW(
        pName: LPWSTR,
        pEnvironment: LPWSTR,
        pDriverName: LPWSTR,
        dwDeleteFlag: DWORD,
        dwVersionFlag: DWORD,
    ) -> WINBOOL; // printerdrivers.c
    pub fn DeletePrinterDriverW(pName: LPWSTR, pEnvironment: LPWSTR, pDriverName: LPWSTR) -> WINBOOL; // printerdrivers.c
    pub fn DllAllocSplMem(dwBytes: DWORD) -> PVOID; // printerdrivers.c
    pub fn DllFreeSplMem(pMem: PVOID) -> BOOL; // printerdrivers.c
    pub fn EndDocPrinter(hPrinter: HANDLE) -> WINBOOL; // printers.c
    pub fn EndPagePrinter(hPrinter: HANDLE) -> WINBOOL; // printers.c
    pub fn EnumFormsW(
        hPrinter: HANDLE,
        Level: DWORD,
        pForm: LPBYTE,
        cbBuf: DWORD,
        pcbNeeded: LPDWORD,
        pcReturned: LPDWORD,
    ) -> WINBOOL; // forms.c
    pub fn EnumJobsW(
        hPrinter: HANDLE,
        FirstJob: DWORD,
        NoJobs: DWORD,
        Level: DWORD,
        pJob: LPBYTE,
        cbBuf: DWORD,
        pcbNeeded: LPDWORD,
        pcReturned: LPDWORD,
    ) -> WINBOOL; // jobs.c
    pub fn EnumMonitorsW(
        pName: LPWSTR,
        Level: DWORD,
        pMonitor: LPBYTE,
        cbBuf: DWORD,
        pcbNeeded: LPDWORD,
        pcReturned: LPDWORD,
    ) -> WINBOOL; // monitors.c
    pub fn EnumPortsW(
        pName: LPWSTR,
        Level: DWORD,
        pPorts: LPBYTE,
        cbBuf: DWORD,
        pcbNeeded: LPDWORD,
        pcReturned: LPDWORD,
    ) -> WINBOOL; // ports.c
    pub fn EnumPrintProcessorDatatypesW(
        pName: LPWSTR,
        pPrintProcessorName: LPWSTR,
        Level: DWORD,
        pDatatypes: LPBYTE,
        cbBuf: DWORD,
        pcbNeeded: LPDWORD,
        pcReturned: LPDWORD,
    ) -> WINBOOL; // printprocessors.c
    pub fn EnumPrintProcessorsW(
        pName: LPWSTR,
        pEnvironment: LPWSTR,
        Level: DWORD,
        pPrintProcessorInfo: LPBYTE,
        cbBuf: DWORD,
        pcbNeeded: LPDWORD,
        pcReturned: LPDWORD,
    ) -> WINBOOL; // printprocessors.c
    pub fn EnumPrinterDriversW(
        pName: LPWSTR,
        pEnvironment: LPWSTR,
        Level: DWORD,
        pDriverInfo: LPBYTE,
        cbBuf: DWORD,
        pcbNeeded: LPDWORD,
        pcReturned: LPDWORD,
    ) -> WINBOOL; // printerdrivers.c
    pub fn GetFormW(
        hPrinter: HANDLE,
        pFormName: LPWSTR,
        Level: DWORD,
        pForm: LPBYTE,
        cbBuf: DWORD,
        pcbNeeded: LPDWORD,
    ) -> WINBOOL; // forms.c
    pub fn GetJobW(
        hPrinter: HANDLE,
        JobId: DWORD,
        Level: DWORD,
        pJob: LPBYTE,
        cbBuf: DWORD,
        pcbNeeded: LPDWORD,
    ) -> WINBOOL; // jobs.c
    pub fn GetPrintProcessorDirectoryW(
        pName: LPWSTR,
        pEnvironment: LPWSTR,
        Level: DWORD,
        pPrintProcessorInfo: LPBYTE,
        cbBuf: DWORD,
        pcbNeeded: LPDWORD,
    ) -> WINBOOL; // printprocessors.c
    pub fn GetPrinterDataExW(
        hPrinter: HANDLE,
        pKeyName: LPCWSTR,
        pValueName: LPCWSTR,
        pType: LPDWORD,
        pData: LPBYTE,
        nSize: DWORD,
        pcbNeeded: LPDWORD,
    ) -> DWORD; // printerdata.c
    pub fn GetPrinterDriverDirectoryW(
        pName: LPWSTR,
        pEnvironment: LPWSTR,
        Level: DWORD,
        pDriverDirectory: LPBYTE,
        cbBuf: DWORD,
        pcbNeeded: LPDWORD,
    ) -> WINBOOL; // printerdrivers.c
    pub fn GetPrinterDriverExW(
        hPrinter: HANDLE,
        pEnvironment: LPWSTR,
        Level: DWORD,
        pDriverInfo: LPBYTE,
        cbBuf: DWORD,
        pcbNeeded: LPDWORD,
        dwClientMajorVersion: DWORD,
        dwClientMinorVersion: DWORD,
        pdwServerMajorVersion: PDWORD,
        pdwServerMinorVersion: PDWORD,
    ) -> BOOL; // printerdrivers.c
    pub fn GetPrinterDriverW(
        hPrinter: HANDLE,
        pEnvironment: LPWSTR,
        Level: DWORD,
        pDriverInfo: LPBYTE,
        cbBuf: DWORD,
        pcbNeeded: LPDWORD,
    ) -> WINBOOL; // printerdrivers.c
    pub fn GetPrinterW(
        hPrinter: HANDLE,
        Level: DWORD,
        pPrinter: LPBYTE,
        cbBuf: DWORD,
        pcbNeeded: LPDWORD,
    ) -> WINBOOL; // printers.c
    pub fn InitializeRouter(SpoolerStatusHandle: HANDLE) -> BOOL; // main.c
    pub fn MarshallDownStructure(
        pStructure: PVOID,
        pInfo: *const MARSHALLING_INFO,
        cbStructureSize: DWORD,
        bSomeBoolean: BOOL,
    ) -> BOOL; // forms.c
    pub fn MarshallDownStructuresArray(
        pStructuresArray: PVOID,
        cElements: DWORD,
        pInfo: *const MARSHALLING_INFO,
        cbStructureSize: DWORD,
        bSomeBoolean: BOOL,
    ) -> BOOL; // forms.c
    pub fn OpenPrinterW(
        pPrinterName: LPWSTR,
        phPrinter: LPHANDLE,
        pDefault: LPPRINTER_DEFAULTSW,
    ) -> WINBOOL; // printers.c
    pub fn PrinterMessageBoxW(
        hPrinter: HANDLE,
        Error: DWORD,
        hWnd: HWND,
        pText: LPWSTR,
        pCaption: LPWSTR,
        dwType: DWORD,
    ) -> DWORD; // rpcstubs.c
    pub fn ReadPrinter(hPrinter: HANDLE, pBuf: LPVOID, cbBuf: DWORD, pNoBytesRead: LPDWORD) -> WINBOOL; // printers.c
    pub fn ResetPrinterW(hPrinter: HANDLE, pDefault: LPPRINTER_DEFAULTSW) -> WINBOOL; // printers.c
    pub fn ScheduleJob(hPrinter: HANDLE, JobId: DWORD) -> WINBOOL; // jobs.c
    pub fn SeekPrinter(
        hPrinter: HANDLE,
        liDistanceToMove: LARGE_INTEGER,
        pliNewPointer: PLARGE_INTEGER,
        dwMoveMethod: DWORD,
        bWrite: BOOL,
    ) -> BOOL; // printers.c
    pub fn SetFormW(hPrinter: HANDLE, pFormName: LPWSTR, Level: DWORD, pForm: LPBYTE) -> WINBOOL; // forms.c
    pub fn SetJobW(
        hPrinter: HANDLE,
        JobId: DWORD,
        Level: DWORD,
        pJob: LPBYTE,
        Command: DWORD,
    ) -> WINBOOL; // jobs.c
    pub fn SetPortW(pName: LPWSTR, pPortName: LPWSTR, dwLevel: DWORD, pPortInfo: LPBYTE) -> WINBOOL; // ports.c
    pub fn SetPrinterDataExW(
        hPrinter: HANDLE,
        pKeyName: LPCWSTR,
        pValueName: LPCWSTR,
        Type: DWORD,
        pData: LPBYTE,
        cbData: DWORD,
    ) -> DWORD; // printerdata.c
    pub fn SplCloseSpoolFileHandle(hPrinter: HANDLE) -> BOOL; // spoolfile.c
    pub fn SplCommitSpoolData(
        hPrinter: HANDLE,
        hProcessHandle: HANDLE,
        cbCommit: DWORD,
        Level: DWORD,
        pFileInfo: *mut FILE_INFO_1,
        dwSize: DWORD,
        dwNeeded: *mut DWORD,
    ) -> BOOL; // spoolfile.c
    pub fn SplGetSpoolFileInfo(
        hPrinter: HANDLE,
        hProcessHandle: HANDLE,
        Level: DWORD,
        pFileInfo: *mut FILE_INFO_1,
        dwSize: DWORD,
        dwNeeded: *mut DWORD,
    ) -> BOOL; // spoolfile.c
    pub fn SpoolerInit() -> BOOL; // init.c
    pub fn StartDocPrinterW(hPrinter: HANDLE, Level: DWORD, pDocInfo: LPBYTE) -> DWORD; // printers.c
    pub fn StartPagePrinter(hPrinter: HANDLE) -> WINBOOL; // printers.c
    pub fn UndoAlignRpcPtr(
        pDestinationBuffer: PVOID,
        pSourceBuffer: PVOID,
        cbBuffer: DWORD,
        pcbNeeded: PDWORD,
    ) -> PDWORD; // forms.c
    pub fn WritePrinter(hPrinter: HANDLE, pBuf: LPVOID, cbBuf: DWORD, pcWritten: LPDWORD) -> WINBOOL; // printers.c
    pub fn XcvDataW(
        hXcv: HANDLE,
        pszDataName: PCWSTR,
        pInputData: PBYTE,
        cbInputData: DWORD,
        pOutputData: PBYTE,
        cbOutputData: DWORD,
        pcbOutputNeeded: PDWORD,
        pdwStatus: PDWORD,
    ) -> WINBOOL; // xcv.c
}
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
pub static mut __wine_dbch_spoolsv: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x73\x70\x6f\x6f\x6c\x73\x76\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_spoolsv };
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

// The marshalling tables (static const in the C headers), flat under their C names.
// NAMED, not globbed: the bindgen island also declares these as `extern` statics (bindgen sees
// the header's `static const`), and an explicit import outranks a glob where two globs clash.
pub use crate::spoolsv::marshalling_tables::{
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
