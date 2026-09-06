//! tcpsvcs-rust prelude — the single import surface the translated modules use.
//! NOTE tcpsvcs has no DPRINT/wine channel: it logs through its OWN LogEvent (log.c, in-crate),
//! so that is a real fn, not a macro. Only the single stray FIXME is a no-op macro.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
pub use core::ptr::{addr_of, addr_of_mut};

// ================= CRT (cdecl) — msvcrt/ntdll exports ==========================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
    pub fn malloc(size: usize) -> *mut c_void;
    pub fn calloc(nmemb: usize, size: usize) -> *mut c_void;
    pub fn free(ptr: *mut c_void);
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
    pub fn wcslen(s: *const u16) -> usize;
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int;
    pub fn wcsicmp(a: *const u16, b: *const u16) -> c_int;
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16;
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16;
    // C-variadic: DECLARING and CALLING one is stable Rust; only *defining* one needs nightly.
    pub fn _snprintf(buf: *mut c_char, count: usize, format: *const c_char, ...) -> c_int;
    pub fn _snwprintf(buf: *mut u16, count: usize, format: *const u16, ...) -> c_int;
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// ================= CRT, continued ==============================================================
// <time.h>. `tm` is not in the island (winsock-scoped) nor in ntdll-sys; layout is the C standard
// one and is identical on all three arches (9 x int).
#[repr(C)]
#[derive(Copy, Clone)]
pub struct tm {
    pub tm_sec: c_int,
    pub tm_min: c_int,
    pub tm_hour: c_int,
    pub tm_mday: c_int,
    pub tm_mon: c_int,
    pub tm_year: c_int,
    pub tm_wday: c_int,
    pub tm_yday: c_int,
    pub tm_isdst: c_int,
}

extern "C" {
    // NOTE the *legacy* 2-arg swprintf (buffer, format, ...) — that is the form the C in this
    // module calls (`swprintf(buf, L"...", RetVal)`), matching msvcrt's non-conforming export.
    pub fn swprintf(buffer: *mut u16, format: *const u16, ...) -> c_int;
    pub fn time(destTime: *mut time_t) -> time_t;
    pub fn localtime(sourceTime: *const time_t) -> *mut tm;
    pub fn asctime(timeptr: *const tm) -> *mut c_char;
    pub fn rand() -> c_int;
    pub fn srand(seed: c_uint);
}

// ================= wide-string literals ========================================================
// The translation reached for three different spellings of an L"..." literal — `w"..."`, `u"..."`
// (neither is a real Rust prefix) and `wide!("...")`. This macro is the one that exists; the
// `w"…"`/`u"…"` sites were rewritten onto it. Every literal in this crate is ASCII, so the
// byte->u16 widening is exact; the const assert makes any future non-ASCII literal a COMPILE
// error rather than a silently mangled string.
#[macro_export]
macro_rules! wide {
    ($s:expr) => {{
        const __SRC: &[u8] = $s.as_bytes();
        const __N: usize = __SRC.len() + 1;
        static __W: [u16; __N] = {
            let mut w = [0u16; __N];
            let mut i = 0;
            while i < __SRC.len() {
                assert!(__SRC[i] < 0x80, "wide!() is ASCII-only");
                w[i] = __SRC[i] as u16;
                i += 1;
            }
            w // trailing element stays 0 => NUL-terminated
        };
        &__W
    }};
}
pub use crate::wide;

// ARRAYSIZE(x) — winnt.h. Takes the ARRAY (not a pointer to it), so it must be applied to the
// array binding itself; see the note on InitLogging in log.rs.
#[macro_export]
macro_rules! ARRAYSIZE {
    ($a:expr) => {
        ($a).len()
    };
}
pub use crate::ARRAYSIZE;

// minwindef.h / winnt.h cast macros bindgen cannot emit.
#[inline(always)]
pub fn MAKEWORD(a: u8, b: u8) -> WORD {
    ((a as WORD) & 0xff) | (((b as WORD) & 0xff) << 8)
}
#[inline(always)]
pub fn MAKELANGID(p: USHORT, s: USHORT) -> DWORD {
    (((s as DWORD) & 0xffff) << 10) | ((p as DWORD) & 0xffff)
}

// ================= winsock2.h — cast macros + the FD_* macro family ============================
pub const INVALID_SOCKET: SOCKET = !0; // winsock2.h — (SOCKET)(~0)
pub const INADDR_ANY: u_long = 0; // winsock2.h — (u_long)0x00000000

extern "system" {
    // ws2_32.spec:50 — `151 stdcall __WSAFDIsSet(long ptr)`. FD_ISSET is a macro over it.
    pub fn __WSAFDIsSet(fd: SOCKET, set: *const fd_set) -> c_int;
}

#[inline(always)]
pub unsafe fn FD_ZERO(set: *mut fd_set) {
    (*set).fd_count = 0;
}
/// winsock2.h FD_SET: linear-scan for `fd`, append if absent and there is room.
/// (Value namespace only — it does not collide with the `FD_SET` *type* alias.)
#[inline(always)]
pub unsafe fn FD_SET(fd: SOCKET, set: *mut fd_set) {
    let mut i: u_int = 0;
    while i < (*set).fd_count {
        if *(*set).fd_array.as_ptr().add(i as usize) == fd {
            return;
        }
        i += 1;
    }
    if (*set).fd_count < FD_SETSIZE {
        *(*set).fd_array.as_mut_ptr().add(i as usize) = fd;
        (*set).fd_count += 1;
    }
}
#[inline(always)]
pub unsafe fn FD_ISSET(fd: SOCKET, set: *const fd_set) -> c_int {
    __WSAFDIsSet(fd, set)
}

// ================= winnt.h / handleapi.h / winbase.h ===========================================
pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE; // handleapi.h — (HANDLE)(-1)
pub const EXCEPTION_INT_OVERFLOW: DWORD = 0xC000_0095; // ntstatus.h STATUS_INTEGER_OVERFLOW

/// InterlockedExchange — an intrinsic in the C, an atomic swap here (same full-barrier
/// semantics). Mirrors ws2_32-rust/src/prelude.rs.
#[inline(always)]
pub unsafe fn InterlockedExchange(target: *mut LONG, value: LONG) -> LONG {
    (*(target as *const core::sync::atomic::AtomicI32))
        .swap(value, core::sync::atomic::Ordering::SeqCst)
}

// ================= strsafe.h ===================================================================
// These are `__inline` in strsafe.h — no DLL exports them, so the faithful translation is a Rust
// body. Both truncate-and-NUL-terminate and report STRSAFE_E_INSUFFICIENT_BUFFER, as the header
// does. Mirrors services-rust/src/database.rs and user32-rust/src/misc/dllmain.rs.
pub const S_OK: HRESULT = 0;
pub const STRSAFE_E_INSUFFICIENT_BUFFER: HRESULT = -2147024774; // 0x8007007A
#[inline]
pub unsafe fn StringCchCopyW(pszDest: *mut WCHAR, cchDest: usize, pszSrc: *const WCHAR) -> HRESULT {
    if cchDest == 0 {
        return STRSAFE_E_INSUFFICIENT_BUFFER;
    }
    let mut i = 0usize;
    while i + 1 < cchDest && *pszSrc.add(i) != 0 {
        *pszDest.add(i) = *pszSrc.add(i);
        i += 1;
    }
    *pszDest.add(i) = 0;
    if *pszSrc.add(i) != 0 {
        STRSAFE_E_INSUFFICIENT_BUFFER
    } else {
        S_OK
    }
}
#[inline]
pub unsafe fn StringCchCatW(pszDest: *mut WCHAR, cchDest: usize, pszSrc: *const WCHAR) -> HRESULT {
    let mut d = 0usize;
    while d < cchDest && *pszDest.add(d) != 0 {
        d += 1;
    }
    if d >= cchDest {
        return STRSAFE_E_INSUFFICIENT_BUFFER;
    }
    let mut s = 0usize;
    while d + 1 < cchDest && *pszSrc.add(s) != 0 {
        *pszDest.add(d) = *pszSrc.add(s);
        d += 1;
        s += 1;
    }
    *pszDest.add(d) = 0;
    if *pszSrc.add(s) != 0 {
        STRSAFE_E_INSUFFICIENT_BUFFER
    } else {
        S_OK
    }
}

// ================= winsvc.h ====================================================================
// The bindgen island is generated from tcpsvcs.h + winsock2.h and does not reach <winsvc.h>.
// Declarations copied from wkssvc-rust/src/types_{i386,amd64,arm64}.rs (byte-identical across all
// three arches for these types: SERVICE_STATUS is 7 x DWORD, the rest are pointer-sized) — the
// same approach wlansvc-rust/src/prelude.rs takes for the same reason.
#[repr(C)]
pub struct _SERVICE_STATUS {
    pub dwServiceType: DWORD,
    pub dwCurrentState: DWORD,
    pub dwControlsAccepted: DWORD,
    pub dwWin32ExitCode: DWORD,
    pub dwServiceSpecificExitCode: DWORD,
    pub dwCheckPoint: DWORD,
    pub dwWaitHint: DWORD,
}
pub type SERVICE_STATUS = _SERVICE_STATUS;
pub type LPSERVICE_STATUS = *mut _SERVICE_STATUS;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SERVICE_STATUS_HANDLE__ {
    pub unused: c_int,
}
pub type SERVICE_STATUS_HANDLE = *mut SERVICE_STATUS_HANDLE__;

// WINAPI == stdcall on i386, cdecl elsewhere; `extern "system"` resolves to the right one per
// arch, so the whole crate spells service/thread callbacks that one way (no cfg split needed).
pub type LPSERVICE_MAIN_FUNCTIONW =
    Option<unsafe extern "system" fn(dwNumServicesArgs: DWORD, lpServiceArgVectors: *mut LPWSTR)>;

#[repr(C)]
pub struct _SERVICE_TABLE_ENTRYW {
    pub lpServiceName: LPWSTR,
    pub lpServiceProc: LPSERVICE_MAIN_FUNCTIONW,
}
pub type SERVICE_TABLE_ENTRYW = _SERVICE_TABLE_ENTRYW;
pub type LPSERVICE_TABLE_ENTRYW = *mut _SERVICE_TABLE_ENTRYW;

// winsvc.h dwCurrentState values
pub const SERVICE_STOPPED: DWORD = 1;
pub const SERVICE_START_PENDING: DWORD = 2;
pub const SERVICE_STOP_PENDING: DWORD = 3;
pub const SERVICE_RUNNING: DWORD = 4;
pub const SERVICE_PAUSED: DWORD = 7;
// winsvc.h dwControlsAccepted bits
pub const SERVICE_ACCEPT_STOP: DWORD = 1;
pub const SERVICE_ACCEPT_PAUSE_CONTINUE: DWORD = 2;
pub const SERVICE_ACCEPT_SHUTDOWN: DWORD = 4;
// winsvc.h control codes
pub const SERVICE_CONTROL_STOP: DWORD = 1;
pub const SERVICE_CONTROL_PAUSE: DWORD = 2;
pub const SERVICE_CONTROL_CONTINUE: DWORD = 3;
pub const SERVICE_CONTROL_INTERROGATE: DWORD = 4;
pub const SERVICE_CONTROL_SHUTDOWN: DWORD = 5;

// ---- SERVICES (tcpsvcs.h) — ABI-spelling shadow of the island's `_Services` -------------------
// The island renders tcpsvcs.h's `LPTHREAD_START_ROUTINE lpService` with the *concrete* per-arch
// ABI bindgen picks ("stdcall" on i386, "C" on amd64/arm64). Our five handlers are defined
// `extern "system"`, which is the SAME ABI on every arch but a DISTINCT type to rustc — so
// `Some(EchoHandler)` will not typecheck against the island's alias on ANY of the three targets
// (measured, not assumed). Re-declaring the struct here with the "system" spelling keeps one
// definition of each handler and needs no transmute; the layout is unchanged (USHORT + pointer +
// function pointer), and an explicit item lawfully shadows the `crate::types::*` glob above.
pub type LPTHREAD_START_ROUTINE_SYS = Option<unsafe extern "system" fn(lpParameter: LPVOID) -> DWORD>;
#[repr(C)]
pub struct _Services_sys {
    pub Port: USHORT,
    pub lpName: LPWSTR,
    pub lpService: LPTHREAD_START_ROUTINE_SYS,
}
pub type SERVICES = _Services_sys;
pub type PSERVICES = *mut _Services_sys;

// ================= kernel32 / advapi32 / ws2_32 imports ========================================
// All `extern "system"` = stdcall-on-i386 / cdecl elsewhere, which is what the import libs export.
extern "system" {
    // ---- advapi32: service control ----
    pub fn RegisterServiceCtrlHandlerExW(
        lpServiceName: LPCWSTR,
        lpHandlerProc: Option<unsafe extern "system" fn(DWORD, DWORD, LPVOID, LPVOID) -> DWORD>,
        lpContext: LPVOID,
    ) -> SERVICE_STATUS_HANDLE;
    pub fn SetServiceStatus(
        hServiceStatus: SERVICE_STATUS_HANDLE,
        lpServiceStatus: LPSERVICE_STATUS,
    ) -> BOOL;
    pub fn StartServiceCtrlDispatcherW(lpServiceStartTable: *const SERVICE_TABLE_ENTRYW) -> BOOL;

    // ---- advapi32: event log ----
    pub fn RegisterEventSourceW(lpUNCServerName: LPCWSTR, lpSourceName: LPCWSTR) -> HANDLE;
    pub fn ReportEventW(
        hEventLog: HANDLE,
        wType: WORD,
        wCategory: WORD,
        dwEventID: DWORD,
        lpUserSid: PSID,
        wNumStrings: WORD,
        dwDataSize: DWORD,
        lpStrings: *const LPCWSTR,
        lpRawData: LPVOID,
    ) -> BOOL;
    pub fn CloseEventLog(hEventLog: HANDLE) -> BOOL;

    // ---- kernel32: threads / sync ----
    pub fn CreateThread(
        lpThreadAttributes: LPSECURITY_ATTRIBUTES,
        dwStackSize: SIZE_T,
        lpStartAddress: LPTHREAD_START_ROUTINE_SYS,
        lpParameter: LPVOID,
        dwCreationFlags: DWORD,
        lpThreadId: LPDWORD,
    ) -> HANDLE;
    // DECLSPEC_NORETURN in the header, but declared returning () on purpose: a Rust `!` return
    // teaches LLVM the call cannot return and it emits ud2 after it (field manual §9.65).
    // Same choice as wlansvc-rust/src/prelude.rs.
    pub fn ExitThread(dwExitCode: DWORD);
    pub fn ExitProcess(uExitCode: UINT);
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
    pub fn WaitForMultipleObjects(
        nCount: DWORD,
        lpHandles: *const HANDLE,
        bWaitAll: BOOL,
        dwMilliseconds: DWORD,
    ) -> DWORD;
    pub fn CreateEventW(
        lpEventAttributes: LPSECURITY_ATTRIBUTES,
        bManualReset: BOOL,
        bInitialState: BOOL,
        lpName: LPCWSTR,
    ) -> HANDLE;
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;

    // ---- kernel32: files ----
    pub fn CreateFileW(
        lpFileName: LPCWSTR,
        dwDesiredAccess: DWORD,
        dwShareMode: DWORD,
        lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
        dwCreationDisposition: DWORD,
        dwFlagsAndAttributes: DWORD,
        hTemplateFile: HANDLE,
    ) -> HANDLE;
    pub fn ReadFile(
        hFile: HANDLE,
        lpBuffer: LPVOID,
        nNumberOfBytesToRead: DWORD,
        lpNumberOfBytesRead: LPDWORD,
        lpOverlapped: LPOVERLAPPED,
    ) -> BOOL;
    pub fn WriteFile(
        hFile: HANDLE,
        lpBuffer: *const c_void,
        nNumberOfBytesToWrite: DWORD,
        lpNumberOfBytesWritten: LPDWORD,
        lpOverlapped: LPOVERLAPPED,
    ) -> BOOL;
    pub fn FlushFileBuffers(hFile: HANDLE) -> BOOL;
    pub fn GetFileSize(hFile: HANDLE, lpFileSizeHigh: LPDWORD) -> DWORD;
    pub fn GetOverlappedResult(
        hFile: HANDLE,
        lpOverlapped: LPOVERLAPPED,
        lpNumberOfBytesTransferred: LPDWORD,
        bWait: BOOL,
    ) -> BOOL;
    pub fn DeleteFileW(lpFileName: LPCWSTR) -> BOOL;

    // ---- kernel32: environment / misc ----
    pub fn GetEnvironmentVariableW(lpName: LPCWSTR, lpBuffer: LPWSTR, nSize: DWORD) -> DWORD;
    pub fn GetSystemDirectoryW(lpBuffer: LPWSTR, uSize: UINT) -> UINT;
    pub fn GetTickCount() -> DWORD;
    pub fn GetLastError() -> DWORD;
    pub fn RaiseException(
        dwExceptionCode: DWORD,
        dwExceptionFlags: DWORD,
        nNumberOfArguments: DWORD,
        lpArguments: *const ULONG_PTR,
    );
    pub fn FormatMessageW(
        dwFlags: DWORD,
        lpSource: *const c_void,
        dwMessageId: DWORD,
        dwLanguageId: DWORD,
        lpBuffer: LPWSTR,
        nSize: DWORD,
        Arguments: *mut c_void,
    ) -> DWORD;

    // ---- kernel32: heap ----
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    pub fn LocalFree(hMem: HLOCAL) -> HLOCAL;

    // ---- ws2_32 ----
    pub fn WSAStartup(wVersionRequested: WORD, lpWSAData: LPWSADATA) -> c_int;
    pub fn WSACleanup() -> c_int;
    pub fn WSAGetLastError() -> c_int;
    pub fn socket(af: c_int, type_: c_int, protocol: c_int) -> SOCKET;
    pub fn bind(s: SOCKET, name: *const SOCKADDR, namelen: c_int) -> c_int;
    pub fn listen(s: SOCKET, backlog: c_int) -> c_int;
    pub fn accept(s: SOCKET, addr: *mut SOCKADDR, addrlen: *mut c_int) -> SOCKET;
    pub fn send(s: SOCKET, buf: *const c_char, len: c_int, flags: c_int) -> c_int;
    pub fn recv(s: SOCKET, buf: *mut c_char, len: c_int, flags: c_int) -> c_int;
    pub fn shutdown(s: SOCKET, how: c_int) -> c_int;
    pub fn closesocket(s: SOCKET) -> c_int;
    pub fn select(
        nfds: c_int,
        readfds: *mut fd_set,
        writefds: *mut fd_set,
        exceptfds: *mut fd_set,
        timeout: *const timeval,
    ) -> c_int;
    pub fn htons(hostshort: u_short) -> u_short;
    pub fn ntohs(netshort: u_short) -> u_short;
    pub fn htonl(hostlong: u_long) -> u_long;
    pub fn inet_ntoa(in_: in_addr) -> *mut c_char;
}

// ================= in-crate cross-module surface ===============================================
// The C compiled these as 8 separate TUs against tcpsvcs.h; here they are sibling modules, so the
// header's prototypes become re-exports. `bShutdown`/`bPause` are the two `extern volatile BOOL`
// globals — see the read_volatile note at their use sites in skelserver.rs.
pub use crate::tcpsvcs::chargen::ChargenHandler;
pub use crate::tcpsvcs::daytime::DaytimeHandler;
pub use crate::tcpsvcs::discard::DiscardHandler;
pub use crate::tcpsvcs::echo::EchoHandler;
pub use crate::tcpsvcs::log::{InitLogging, LogEvent, UninitLogging};
pub use crate::tcpsvcs::qotd::QotdHandler;
pub use crate::tcpsvcs::skelserver::{ShutdownConnection, StartServer};
pub use crate::tcpsvcs::tcpsvcs::{bPause, bShutdown};

/// Read of the C's `volatile BOOL bShutdown` (tcpsvcs.h).
///
/// The `volatile` is load-bearing: ServerCtrlHandler sets this from the SCM's thread while every
/// worker/accept thread SPINS on it (`while (!bShutdown)`). A plain Rust read of a `static mut`
/// carries no such guarantee, so the translation's bare `bShutdown == 0` was licensing LLVM to
/// hoist the load out of those loops — i.e. a service that never notices a stop request. Today
/// the address escapes (InterlockedExchange takes `&bShutdown`) and every loop body makes an
/// opaque call, so the reload happens anyway; this makes it guaranteed rather than incidental.
#[macro_export]
macro_rules! bShutdown {
    () => {
        core::ptr::read_volatile(core::ptr::addr_of!($crate::tcpsvcs::tcpsvcs::bShutdown))
    };
}
pub use crate::bShutdown;
