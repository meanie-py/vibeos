//! ws2_32-rust prelude — the single import surface every translated module uses.
//!
//! WHY A PRELUDE. rustconv emits per-file `extern` blocks, so before this existed the same
//! symbol was declared in up to 14 modules — and they DISAGREED. `WsSockHeap` was declared
//! five different ways (`*mut c_void`, `LPVOID`, `HANDLE`, under both `extern "C"` and
//! `extern "system"`), and `HeapAlloc` had nine variants across the crate. Every one of
//! those is a place for the ABI to drift silently. One declaration, one truth.
//!
//! Three kinds of thing live here, and the distinction is load-bearing:
//!
//!   1. RE-EXPORTS of symbols this crate DEFINES (the Ws*/WPU* object model). These are real
//!      Rust re-exports, not `extern` re-declarations — so there is no second copy of the
//!      signature that can drift from the definition.
//!   2. EXTERN DECLS of genuine imports. The ABI of each is taken from the .spec of the DLL
//!      that exports it (postprocess::enforce_import_abi) — kernel32 is stdcall, the CRT is
//!      cdecl, and the C source does not say which is which.
//!   3. MACROS AND INLINES that have NO symbol at all. These MUST NOT become `extern fn`
//!      declarations — that is a link error at best, and the field manual's §9.65 trap at
//!      worst (an out-of-line no-op where C had a macro is an optimization barrier that
//!      uncloaks latent ABI mismatches). `MAKEWORD`/`LOBYTE`/`HIBYTE` are macros in
//!      minwindef.h; `WsGetProcess`/`WsQuickProlog`/`WsQuickPrologTid` are FORCEINLINE in
//!      ws2_32p.h. They are IMPLEMENTED below, not declared.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use ntdll_sys::nt::*;

// ---- 1. In-crate object model: RE-EXPORTED from the module that defines it ----
// (No `extern` re-declaration: the definition is the only signature.)
pub use crate::src::r#async::{WsAsyncTerminateThread, WsAsyncThreadInitialized};
pub use crate::src::addrconv::{htonl, htons, ntohs};
pub use crate::src::getxbyxx::{gethostbyname, getservbyname};
pub use crate::src::dcatalog::{
    WsTcDelete, WsTcEnumerateCatalogItems, WsTcGetEntryFromCatalogEntryId, WsTcRefreshFromRegistry,
};
pub use crate::src::dcatitem::{WsTcEntryDereference, WsTcEntrySetProvider};
pub use crate::src::dllmain::{GlobalTlsIndex, WsSockHeap};
pub use crate::src::dprocess::{WsProcDelete, WsProcGetTCatalog};
pub use crate::src::dprovide::{WsTpAllocate, WsTpDereference, WsTpInitialize, WsTpWSPCleanup};
pub use crate::src::dsocket::{
    WsSockDeleteSockets, WsSockDereference, WsSockDisassociateHandle, WsSockGetSocket,
    WsSockHandleTable,
};
pub use crate::src::dthread::WsThreadGetCurrentThread;
pub use crate::src::enumprot::WPUGetProviderPath;
pub use crate::src::nscatalo::{
    WsNcDelete, WsNcEnumerateCatalogItems, WsNcLoadProvider, WsNcMatchProtocols,
};
pub use crate::src::nscatent::{
    WsNcEntryAllocate, WsNcEntryInitializeFromRegistry, WsNcEntrySetProvider,
};
pub use crate::src::nsprovid::{
    WsNpAllocate, WsNpDereference, WsNpInitialize, WsNqProvAllocate, WsNqProvInitialize,
};
pub use crate::src::nsquery::{WsNqAllocate, WsNqDelete, WsNqInitialize, WsNqLookupServiceBegin};
pub use crate::src::qshelpr::{AnsiDupFromUnicode, UnicodeDupFromAnsi};
pub use crate::src::rnr::{WSALookupServiceBeginA, WSALookupServiceEnd, WSALookupServiceNextA};
pub use crate::src::select::WPUFDIsSet;
pub use crate::src::socklife::{
    WPUCloseSocketHandle, WPUCreateSocketHandle, WPUModifyIFSHandle, WPUQuerySocketHandleContext,
};
pub use crate::src::sputil::{
    WPUCloseEvent, WPUCloseThread, WPUCreateEvent, WPUOpenCurrentThread, WPUPostMessage,
    WPUQueryBlockingCallback, WPUQueueApc, WPUResetEvent, WPUSetEvent,
};
pub use crate::src::startup::{WSAStartup, WsSockPostRoutine};
pub use crate::src::wsautil::{WsApiProlog, WsCheckCatalogState, WsSetupCatalogProtection};

/// `#define TlsIndex GlobalTlsIndex` (ws2_32p.h:277) — an alias, not a second variable.
pub use crate::src::dllmain::GlobalTlsIndex as TlsIndex;

// ---- 2. Genuine imports. ABI per the exporting DLL's .spec (enforce_import_abi). ----
extern "system" {
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn CreateFileA( lpFileName: LPCSTR, dwDesiredAccess: DWORD, dwShareMode: DWORD, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, dwCreationDisposition: DWORD, dwFlagsAndAttributes: DWORD, hTemplateFile: HANDLE, ) -> HANDLE;
    pub fn DeleteCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn DispatchMessageA(lpMsg: *const MSG) -> isize;
    pub fn EnterCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn ExpandEnvironmentStringsA(lpSrc: LPCSTR, lpDst: LPSTR, nSize: DWORD) -> DWORD;
    pub fn FreeLibrary(hLibModule: *mut core::ffi::c_void) -> i32;
    pub fn FreeLibraryAndExitThread(hLibModule: HMODULE, dwExitCode: DWORD);
    pub fn GetHandleInformation(hObject: *mut core::ffi::c_void, lpdwFlags: *mut u32) -> i32;
    pub fn GetLastError() -> DWORD;
    pub fn GetProcAddress(hModule: HINSTANCE, lpProcName: *const i8) -> *mut core::ffi::c_void;
    pub fn GetProcessHeap() -> HANDLE;
    pub fn GetSystemDirectoryA(lpBuffer: LPSTR, uSize: UINT) -> UINT;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    pub fn InitializeCriticalSection(lpCriticalSection: *mut core::ffi::c_void);
    pub fn IsBadCodePtr(lpfn: FARPROC) -> i32;
    pub fn IsBadReadPtr(lp: *const core::ffi::c_void, ucb: usize) -> i32;
    pub fn IsBadWritePtr(lp: LPVOID, ucb: usize) -> INT;
    pub fn LeaveCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn LoadLibraryA(lpLibFileName: *const i8) -> HINSTANCE;
    pub fn MultiByteToWideChar( CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: LPCSTR, cbMultiByte: INT, lpWideCharStr: LPWSTR, cchWideChar: INT, ) -> INT;
    pub fn PeekMessageA( lpMsg: *mut MSG, hWnd: *mut core::ffi::c_void, wMsgFilterMin: u32, wMsgFilterMax: u32, wRemoveMsg: u32, ) -> i32;
    pub fn ReadFile( hFile: HANDLE, lpBuffer: LPVOID, nNumberOfBytesToRead: DWORD, lpNumberOfBytesRead: *mut DWORD, lpOverlapped: *mut OVERLAPPED, ) -> BOOL;
    pub fn RegCloseKey(hKey: HKEY) -> LONG;
    pub fn RegCreateKeyExA( hKey: HKEY, lpSubKey: *const i8, Reserved: u32, lpClass: *mut i8, dwOptions: u32, samDesired: u32, lpSecurityAttributes: *mut core::ffi::c_void, phkResult: *mut HKEY, lpdwDisposition: *mut u32, ) -> i32;
    pub fn RegOpenKeyExA(hKey: HKEY, lpSubKey: LPCSTR, ulOptions: DWORD, samDesired: REGSAM, phkResult: *mut HKEY) -> LONG;
    pub fn RegQueryValueExA(hKey: HKEY, lpValueName: LPCSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD) -> LONG;
    pub fn RegQueryValueExW(hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD) -> LONG;
    pub fn RegSetValueExA( hKey: HKEY, lpValueName: *const i8, Reserved: u32, dwType: u32, lpData: *const u8, cbData: u32, ) -> i32;
    pub fn ResetEvent(hEvent: HANDLE) -> BOOL;
    pub fn SetFilePointer( hFile: HANDLE, lDistanceToMove: LONG, lpDistanceToMoveHigh: *mut LONG, dwMoveMethod: DWORD, ) -> DWORD;
    pub fn SetLastError(dwErrCode: DWORD);
    pub fn TlsAlloc() -> DWORD;
    pub fn TlsFree(dwTlsIndex: DWORD) -> BOOL;
    pub fn TlsGetValue(dwTlsIndex: DWORD) -> LPVOID;
    pub fn TlsSetValue(dwTlsIndex: DWORD, lpTlsValue: *mut core::ffi::c_void) -> i32;
    pub fn TranslateMessage(lpMsg: *const MSG) -> i32;
    pub fn WahDestroyHandleContextTable(HandleTable: PWAH_HANDLE_TABLE);
    pub fn WahOpenCurrentThread(AsyncHelper: *mut core::ffi::c_void, WahThreadId: LPWSATHREADID) -> DWORD;
    pub fn WahReferenceContextByHandle(HandleTable: PWAH_HANDLE_TABLE, Handle: LPVOID) -> LPVOID;
    pub fn WahRemoveHandleContext( HandleTable: PWAH_HANDLE_TABLE, Handle: PWAH_HANDLE, ) -> INT;
    pub fn WaitForMultipleObjectsEx( nCount: DWORD, lpHandles: *const HANDLE, bWaitAll: BOOL, dwMilliseconds: DWORD, bAlertable: BOOL, ) -> DWORD;
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
    pub fn WideCharToMultiByte( CodePage: u32, dwFlags: u32, lpWideCharStr: LPWSTR, cchWideChar: i32, lpMultiByteStr: LPSTR, cbMultiByte: i32, lpDefaultChar: *const i8, lpUsedDefaultChar: *mut i32, ) -> i32;
    pub fn Ws2HelpDllMain(hModule: HANDLE, dwReason: DWORD, lpReserved: LPVOID) -> BOOL;
    pub fn lstrcpy(lpString1: *mut i8, lpString2: *const i8) -> *mut i8;
}

extern "C" {
    pub fn _itoa(value: i32, str: PSTR, radix: i32) -> PSTR;
    pub fn _stricmp(string1: *const CHAR, string2: *const CHAR) -> INT;
    pub fn atoi(s: *const i8) -> i32;
    pub fn isdigit(c: i32) -> i32;
    pub fn islower(c: i32) -> i32;
    pub fn isspace(c: i32) -> i32;
    pub fn isxdigit(c: i32) -> i32;
    pub fn memcpy(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
    pub fn sprintf(s: *mut core::ffi::c_char, format: *const core::ffi::c_char, ...) -> core::ffi::c_int;
    pub fn strcat(dest: PSTR, src: PSTR) -> PSTR;
    pub fn strchr(s: *const i8, c: i32) -> *mut i8;
    pub fn strcpy(dest: PSTR, src: PCSTR) -> PSTR;
    pub fn strlen(s: *const core::ffi::c_char) -> size_t;
    pub fn strcmp(s1: *const core::ffi::c_char, s2: *const core::ffi::c_char) -> INT;
    pub fn strtoul(s: *const core::ffi::c_char, end: *mut *mut core::ffi::c_char, base: INT) -> ULONG;
    pub fn strncat(dest: *mut core::ffi::c_char, src: *const core::ffi::c_char, n: size_t) -> *mut core::ffi::c_char;
    pub fn strncpy(dest: *mut i8, src: *const i8, count: usize) -> *mut i8;
    pub fn wcscpy(dest: LPWSTR, src: *const u16) -> LPWSTR;
    pub fn wcslen(s: *const u16) -> usize;
    pub fn wcstombs(dest: *mut i8, src: *const u16, count: usize) -> usize;
    pub fn wsprintfW(buf: LPWSTR, fmt: PCWSTR, ...) -> i32;
}


// ---- 3a. Constants bindgen skips (they are #defines with a cast, not enum members) ----
// Values transcribed from the real headers, with citations. Do not "simplify" a cast away:
// INVALID_SOCKET is `(SOCKET)(~0)`, i.e. all-ones in the SOCKET width — which is 64-bit on
// amd64/arm64 and 32-bit on i386. Writing `-1i32` here would be right on one arch only.
pub const INVALID_SOCKET: SOCKET = !0 as SOCKET; // winsock2.h:236 — (SOCKET)(~0)
pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE; // handleapi.h:294 — (HANDLE)(-1)
pub const TLS_OUT_OF_INDEXES: DWORD = 0xFFFF_FFFF; // processthreadsapi.h:529
pub const CP_ACP: UINT = 0; // winnls.h:245
pub const INADDR_ANY: u_long = 0; // winsock2.h:305 — (u_long)0
pub const IPPROTO_TCP: INT = 6; // ws2def.h:438
pub const IPPROTO_UDP: INT = 17; // ws2def.h:445

// EAI_* are NOT redefined here: once ws2tcpip.h was added to the bindgen wrapper, the island
// generates them (as u32, aliased to the WSA codes exactly as ws2tcpip.h:32-40 does). Defining
// them here too made every use `ambiguous` — two globs, two truths. The island wins.

pub type size_t = usize;
pub type WSAEVENT = HANDLE; // winsock2.h:579 — #define WSAEVENT HANDLE
pub type HHEAP = HANDLE;

// ---- 3b. C MACROS. Implemented, never declared `extern` ----
// minwindef.h:187/191/192. These are macros: there is NO symbol to link against, and an
// out-of-line no-op fn standing in for a C macro is the field-manual §9.65 trap.
#[inline(always)]
pub fn MAKEWORD(bLow: u8, bHigh: u8) -> WORD {
    (bLow as WORD) | ((bHigh as WORD) << 8)
}
#[inline(always)]
pub fn LOBYTE(w: WORD) -> u8 {
    (w & 0xff) as u8
}
#[inline(always)]
pub fn HIBYTE(w: WORD) -> u8 {
    ((w >> 8) & 0xff) as u8
}
#[inline(always)]
pub fn UlongToPtr(u: u32) -> *mut core::ffi::c_void {
    u as usize as *mut core::ffi::c_void
}
#[inline(always)]
pub fn PtrToUlong(p: *const core::ffi::c_void) -> u32 {
    p as usize as u32
}
/// `#define CopyMemory RtlCopyMemory` -> memcpy. Never hand-roll a byte loop here: LLVM folds
/// one back into a call to memcpy itself (infinite self-recursion — AGENTS.md, field manual).
#[inline(always)]
pub unsafe fn CopyMemory(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, len: usize) {
    memcpy(dst, src, len);
}
#[inline(always)]
pub unsafe fn RtlCopyMemory(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, len: usize) {
    memcpy(dst, src, len);
}
#[inline(always)]
pub unsafe fn RtlZeroMemory(dst: *mut core::ffi::c_void, len: usize) {
    core::ptr::write_bytes(dst as *mut u8, 0, len);
}
#[inline(always)]
pub fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}
/// `#define IsEqualGUID(a,b) (!memcmp(&(a), &(b), sizeof(GUID)))` — bindgen's _GUID derives
/// no PartialEq, and adding one would be a second definition of equality that could drift
/// from the C's. Compare the bytes, exactly as the macro does.
#[inline(always)]
pub unsafe fn IsEqualGUID(a: *const GUID, b: *const GUID) -> bool {
    core::slice::from_raw_parts(a as *const u8, core::mem::size_of::<GUID>())
        == core::slice::from_raw_parts(b as *const u8, core::mem::size_of::<GUID>())
}
/// ws2ipdef.h — an IPv4-mapped IPv6 address is ::ffff:a.b.c.d.
/// Returns BOOL (i32), NOT Rust's bool: the C is a macro yielding an int, and the call sites
/// this translation produced compare it `!= 0`. Matching the C's shape here is not laziness —
/// changing it would silently flip the meaning of any site that treated 0/nonzero directly.
#[inline(always)]
pub unsafe fn IN6_IS_ADDR_V4MAPPED(a: *const in6_addr) -> BOOL {
    let b = (*a).u.Byte;
    (b[..10].iter().all(|&x| x == 0) && b[10] == 0xff && b[11] == 0xff) as BOOL
}

// ANSI/Unicode aliases. ws2_32's CMakeLists does NOT define UNICODE, so the unsuffixed
// spellings in the C resolve to the -A forms (winbase.h/winreg.h/winuser.h #define them).
pub use self::{
    RegCreateKeyExA as RegCreateKeyEx, RegOpenKeyExA as RegOpenKeyEx,
    RegQueryValueExA as RegQueryValueEx, RegSetValueExA as RegSetValueEx,
};

// ---- 3c. FORCEINLINE helpers from ws2_32p.h. No out-of-line symbol exists ----
extern "system" { // ws2_32p.h:339 // :343 // :657
}

/// ws2_32p.h:884-888 — `FORCEINLINE PWSPROCESS WsGetProcess(VOID) { return CurrentWsProcess; }`
#[inline(always)]
pub unsafe fn WsGetProcess() -> PWSPROCESS {
    CurrentWsProcess
}

/// ws2_32p.h:890-896 — the fast path of the API prolog: if the process object already exists
/// we are initialized, otherwise fall into the (out-of-line) slow prolog.
#[inline(always)]
pub unsafe fn WsQuickProlog() -> DWORD {
    if !WsGetProcess().is_null() {
        ERROR_SUCCESS
    } else {
        (WsSlowProlog() as u32)
    }
}

/// ws2_32p.h:898-910 — same, plus the per-thread WSATHREADID the SPI callbacks need.
#[inline(always)]
pub unsafe fn WsQuickPrologTid(Tid: *mut LPWSATHREADID) -> DWORD {
    let proc_ = WsGetProcess();
    if !proc_.is_null() {
        let tid = WsThreadGetThreadId(proc_);
        if !tid.is_null() {
            *Tid = tid;
            return ERROR_SUCCESS;
        }
    }
    (WsSlowPrologTid(Tid) as u32)
}

// ---- 3d. Interlocked* — compiler intrinsics, not symbols ----
// The C headers make these intrinsics (`_InterlockedIncrement` etc.), so there is nothing to
// link against. Same implementation msafd-rust settled on. Note the C contract returns the
// value AFTER the operation, which is why the +1/-1 is there — `fetch_add` returns the value
// BEFORE. Getting that wrong silently off-by-ones every refcount in the DLL.
use core::sync::atomic::{AtomicI32, Ordering};
use crate::src::dthread::WsThreadGetThreadId;
use crate::src::wsautil::WsSlowProlog;
use crate::src::wsautil::WsSlowPrologTid;

#[inline(always)]
pub unsafe fn InterlockedIncrement(addend: *mut LONG) -> LONG {
    (*(addend as *const AtomicI32)).fetch_add(1, Ordering::SeqCst) + 1
}
#[inline(always)]
pub unsafe fn InterlockedDecrement(addend: *mut LONG) -> LONG {
    (*(addend as *const AtomicI32)).fetch_sub(1, Ordering::SeqCst) - 1
}
#[inline(always)]
pub unsafe fn InterlockedExchange(target: *mut LONG, value: LONG) -> LONG {
    (*(target as *const AtomicI32)).swap(value, Ordering::SeqCst)
}
#[inline(always)]
pub unsafe fn InterlockedCompareExchange(dest: *mut LONG, exchange: LONG, comparand: LONG) -> LONG {
    match (*(dest as *const AtomicI32)).compare_exchange(
        comparand,
        exchange,
        Ordering::SeqCst,
        Ordering::SeqCst,
    ) {
        Ok(prev) | Err(prev) => prev,
    }
}
#[inline(always)]
pub unsafe fn InterlockedExchangePointer(
    target: *mut *mut core::ffi::c_void,
    value: *mut core::ffi::c_void,
) -> *mut core::ffi::c_void {
    let a = &*(target as *const core::sync::atomic::AtomicPtr<core::ffi::c_void>);
    a.swap(value, Ordering::SeqCst)
}

// ---- 3e. WSASYNCOPS enum members ----
// bindgen prefixes C enum members with the enum's name (`_WSASYNCOPS_WsAsyncGetHostByName`);
// the C — and so the translation — uses the bare member name. Alias, do not re-define: a
// second `const WsAsyncGetHostByName = 1` would be a literal that can drift from the header.
pub use crate::types::{
    _WSASYNCOPS_WsAsyncGetHostByAddr as WsAsyncGetHostByAddr,
    _WSASYNCOPS_WsAsyncGetHostByName as WsAsyncGetHostByName,
    _WSASYNCOPS_WsAsyncGetProtoByName as WsAsyncGetProtoByName,
    _WSASYNCOPS_WsAsyncGetProtoByNumber as WsAsyncGetProtoByNumber,
    _WSASYNCOPS_WsAsyncGetServByName as WsAsyncGetServByName,
    _WSASYNCOPS_WsAsyncGetServByPort as WsAsyncGetServByPort,
    _WSASYNCOPS_WsAsyncTerminate as WsAsyncTerminate,
};

// ---- 3f. The rest of the import surface ----
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY; // winreg.h:12

extern "system" {
    // kernel32
    pub fn CreateEventA(
        lpEventAttributes: *mut core::ffi::c_void,
        bManualReset: BOOL,
        bInitialState: BOOL,
        lpName: LPCSTR,
    ) -> HANDLE;
    pub fn CreateEventW(
        lpEventAttributes: *mut core::ffi::c_void,
        bManualReset: BOOL,
        bInitialState: BOOL,
        lpName: LPCWSTR,
    ) -> HANDLE;
    pub fn CreateThread(
        lpThreadAttributes: *mut core::ffi::c_void,
        dwStackSize: SIZE_T,
        lpStartAddress: Option<unsafe extern "system" fn(LPVOID) -> DWORD>,
        lpParameter: LPVOID,
        dwCreationFlags: DWORD,
        lpThreadId: LPDWORD,
    ) -> HANDLE;
    pub fn SetEvent(hEvent: HANDLE) -> BOOL;
    // user32
    pub fn PostMessageA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    // ws2help.dll — the handle-context table + APC/thread helpers ws2_32 is built on.
    pub fn WahCreateHandleContextTable(HandleTable: *mut PWAH_HANDLE_TABLE) -> INT;
    pub fn WahInsertHandleContext(
        HandleTable: PWAH_HANDLE_TABLE,
        HandleContext: PWAH_HANDLE,
    ) -> PWAH_HANDLE;
    pub fn WahEnumerateHandleContexts(
        HandleTable: PWAH_HANDLE_TABLE,
        Callback: PWAH_HANDLE_ENUMERATE_PROC,
        Context: PVOID,
    ) -> INT;
    pub fn WahCloseSocketHandle(HandleTable: PWAH_HANDLE_TABLE, Handle: SOCKET) -> INT;
    pub fn WahCloseHandleHelper(HandleHelper: PVOID) -> INT;
    pub fn WahCloseApcHelper(ApcHelper: PVOID) -> INT;
    pub fn WahOpenApcHelper(ApcHelper: *mut PVOID) -> INT;
    pub fn WahCloseNotificationHandleHelper(NotificationHelper: PVOID) -> INT;
    pub fn WahCloseThread(AsyncHelper: PVOID, WahThreadId: LPWSATHREADID) -> INT;
}

// ws2_32 is built WITHOUT -DUNICODE, so the unsuffixed spellings the C uses resolve to -A.
pub use self::{CreateEventA as CreateEvent, PostMessageA as PostMessage};
pub use crate::src::dprocess::CurrentWsProcess;

// ---- 3g. Portable fn-pointer -> FARPROC / LPBLOCKINGCALLBACK ----
// FARPROC's inner fn type is `extern "C"` on amd64/arm64 but `extern "stdcall"` on i386
// (ntdll-sys bindgen), and rustc does NOT unify `extern "system"` with `extern "stdcall"` at
// the TYPE level even though they lower identically — so there is no single ABI spelling that
// compiles on all three arches. The translation hardcoded the amd64 spelling
// (`as unsafe extern "C" fn() -> DWORD`), which is the same "wrote the _WIN64 shape" bug that
// put an 8-byte V_INT_PTR through a 4-byte field in oleaut32. Go through a raw pointer, where
// there is no ABI to spell.
#[inline(always)]
pub unsafe fn as_farproc(f: *const ()) -> FARPROC {
    core::mem::transmute::<*const (), FARPROC>(f)
}
#[inline(always)]
pub unsafe fn as_blocking_callback(f: *const ()) -> LPBLOCKINGCALLBACK {
    core::mem::transmute::<*const (), LPBLOCKINGCALLBACK>(f)
}
