//! telnetd-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Grown during the cargo-check
//! grind: anything bindgen left out (CRT, inlines, sibling-C externs) is declared here once.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ========================================================================
// The wine/debug.h channel (TRACE/ERR/WARN/FIXME). Kept as no-op MACROS, never out-of-line fns:
// an out-of-line no-op is an optimization barrier that uncloaks latent ABI mismatches into ud2
// (field manual §9.65). The WINE_-prefixed spellings are the same macros under their long names.
#[macro_export] macro_rules! TRACE { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! ERR   { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! WARN  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! FIXME { ($($arg:tt)*) => {{}}; }
pub use crate::{ERR, FIXME, TRACE, WARN};
#[macro_export] macro_rules! WINE_TRACE { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! WINE_ERR   { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! WINE_WARN  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! WINE_FIXME { ($($arg:tt)*) => {{}}; }
pub use crate::{WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN};

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
    pub fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
    // C-variadic: DECLARING and CALLING one is stable Rust; only *defining* one needs nightly.
    pub fn _snprintf(buf: *mut c_char, count: usize, format: *const c_char, ...) -> c_int;
    pub fn _snwprintf(buf: *mut u16, count: usize, format: *const u16, ...) -> c_int;
    pub fn printf(format: *const c_char, ...) -> c_int;
    pub fn sprintf(buf: *mut c_char, format: *const c_char, ...) -> c_int;
    pub fn fprintf(stream: *mut FILE, format: *const c_char, ...) -> c_int;
    pub fn fflush(stream: *mut FILE) -> c_int;
    // telnetd.c: `#define telnetd_printf printf` — a plain alias, so bind the same symbol under
    // the second name rather than an out-of-line forwarding shim (which would add a real frame).
    #[link_name = "printf"]
    pub fn telnetd_printf(format: *const c_char, ...) -> c_int;
}

// crt/stdio.h: `struct _iobuf` / `typedef struct _iobuf FILE`.
#[repr(C)]
pub struct _iobuf {
    pub _ptr: *mut c_char,
    pub _cnt: c_int,
    pub _base: *mut c_char,
    pub _flag: c_int,
    pub _file: c_int,
    pub _charbuf: c_int,
    pub _bufsiz: c_int,
    pub _tmpfname: *mut c_char,
}
pub type FILE = _iobuf;

extern "C" {
    /// crt/stdio.h: `#define _iob __iob_func()` (msvcrt.spec: `@ cdecl __iob_func()`).
    pub fn __iob_func() -> *mut FILE;
}
// crt/stdio.h: `#define stdout (&_iob[1])`, `#define stderr (&_iob[2])`. Macros in C, so they are
// call-shaped here; the translated call sites spell them `stdout()` / `stderr()`.
#[inline(always)]
pub unsafe fn stdout() -> *mut FILE {
    __iob_func().add(1)
}
#[inline(always)]
pub unsafe fn stderr() -> *mut FILE {
    __iob_func().add(2)
}

// ================= kernel32 (WINAPI → extern "system") =========================================
pub type LPTHREAD_START_ROUTINE = Option<unsafe extern "system" fn(lpParameter: LPVOID) -> DWORD>;
pub type PHANDLER_ROUTINE = Option<unsafe extern "system" fn(dwCtrlType: DWORD) -> BOOL>;

extern "system" {
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn CreateThread(
        lpThreadAttributes: *mut SECURITY_ATTRIBUTES,
        dwStackSize: SIZE_T,
        lpStartAddress: LPTHREAD_START_ROUTINE,
        lpParameter: LPVOID,
        dwCreationFlags: DWORD,
        lpThreadId: *mut DWORD,
    ) -> HANDLE;
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
    pub fn GetExitCodeProcess(hProcess: HANDLE, lpExitCode: *mut DWORD) -> BOOL;
    pub fn Sleep(dwMilliseconds: DWORD);
    pub fn GenerateConsoleCtrlEvent(dwCtrlEvent: DWORD, dwProcessGroupId: DWORD) -> BOOL;
    pub fn TerminateProcess(hProcess: HANDLE, uExitCode: UINT) -> BOOL;
    pub fn SetEvent(hEvent: HANDLE) -> BOOL;
    pub fn CreateEventA(
        lpEventAttributes: *mut SECURITY_ATTRIBUTES,
        bManualReset: BOOL,
        bInitialState: BOOL,
        lpName: *const c_char,
    ) -> HANDLE;
    pub fn CreateEventW(
        lpEventAttributes: *mut SECURITY_ATTRIBUTES,
        bManualReset: BOOL,
        bInitialState: BOOL,
        lpName: *const u16,
    ) -> HANDLE;
    pub fn SetConsoleCtrlHandler(HandlerRoutine: PHANDLER_ROUTINE, Add: BOOL) -> BOOL;
    pub fn CreatePipe(
        hReadPipe: *mut HANDLE,
        hWritePipe: *mut HANDLE,
        lpPipeAttributes: *mut SECURITY_ATTRIBUTES,
        nSize: DWORD,
    ) -> BOOL;
    pub fn WriteFile(
        hFile: HANDLE,
        lpBuffer: *const c_void,
        nNumberOfBytesToWrite: DWORD,
        lpNumberOfBytesWritten: *mut DWORD,
        lpOverlapped: *mut OVERLAPPED,
    ) -> BOOL;
    pub fn ReadFile(
        hFile: HANDLE,
        lpBuffer: *mut c_void,
        nNumberOfBytesToRead: DWORD,
        lpNumberOfBytesRead: *mut DWORD,
        lpOverlapped: *mut OVERLAPPED,
    ) -> BOOL;
    pub fn PeekNamedPipe(
        hNamedPipe: HANDLE,
        lpBuffer: *mut c_void,
        nBufferSize: DWORD,
        lpBytesRead: *mut DWORD,
        lpTotalBytesAvail: *mut DWORD,
        lpBytesLeftThisMessage: *mut DWORD,
    ) -> BOOL;
    pub fn GetEnvironmentVariableA(
        lpName: *const c_char,
        lpBuffer: *mut c_char,
        nSize: DWORD,
    ) -> DWORD;
    pub fn GetSystemDirectoryA(lpBuffer: *mut c_char, uSize: UINT) -> UINT;
    pub fn CreateProcessA(
        lpApplicationName: *const c_char,
        lpCommandLine: *mut c_char,
        lpProcessAttributes: *mut SECURITY_ATTRIBUTES,
        lpThreadAttributes: *mut SECURITY_ATTRIBUTES,
        bInheritHandles: BOOL,
        dwCreationFlags: DWORD,
        lpEnvironment: *mut c_void,
        lpCurrentDirectory: *const c_char,
        lpStartupInfo: *mut STARTUPINFOA,
        lpProcessInformation: *mut PROCESS_INFORMATION,
    ) -> BOOL;
    // DECLSPEC_NORETURN in winbase.h, but the *type* is VOID; declaring `()` (not `!`) keeps the
    // ABI honest and avoids minting an unreachable/ud2 tail the C side never had.
    pub fn ExitProcess(uExitCode: UINT);
}

// wincon.h — console control events (TerminateShell re-declares these as function-local consts,
// matching the C; these are for the call sites that use them bare).
pub const CTRL_C_EVENT: DWORD = 0;
pub const CTRL_BREAK_EVENT: DWORD = 1;

// ================= advapi32 / winsvc.h =========================================================
// winsvc.h is not in the bindgen island (telnetd.h does not include it; serviceentry.c does), so
// the service surface is declared here, transcribed from sdk/include/psdk/winsvc.h.
#[repr(C)]
pub struct SERVICE_STATUS {
    pub dwServiceType: DWORD,
    pub dwCurrentState: DWORD,
    pub dwControlsAccepted: DWORD,
    pub dwWin32ExitCode: DWORD,
    pub dwServiceSpecificExitCode: DWORD,
    pub dwCheckPoint: DWORD,
    pub dwWaitHint: DWORD,
}
pub type LPSERVICE_STATUS = *mut SERVICE_STATUS;
/// winsvc.h: `DECLARE_HANDLE(SERVICE_STATUS_HANDLE)`.
pub type SERVICE_STATUS_HANDLE = *mut c_void;
/// winsvc.h: `typedef void (WINAPI *LPSERVICE_MAIN_FUNCTIONW)(DWORD,LPWSTR*)`.
pub type LPSERVICE_MAIN_FUNCTIONW =
    Option<unsafe extern "system" fn(dwNumServicesArgs: DWORD, lpServiceArgVectors: *mut *mut u16)>;
/// winsvc.h: `typedef DWORD (WINAPI *LPHANDLER_FUNCTION_EX)(DWORD,DWORD,LPVOID,LPVOID)`.
pub type LPHANDLER_FUNCTION_EX = Option<
    unsafe extern "system" fn(
        dwControl: DWORD,
        dwEventType: DWORD,
        lpEventData: LPVOID,
        lpContext: LPVOID,
    ) -> DWORD,
>;
#[repr(C)]
pub struct SERVICE_TABLE_ENTRYW {
    pub lpServiceName: *mut u16,
    pub lpServiceProc: LPSERVICE_MAIN_FUNCTIONW,
}

// winsvc.h service states / accepted-control bits / control codes.
pub const SERVICE_STOPPED: DWORD = 1;
pub const SERVICE_START_PENDING: DWORD = 2;
pub const SERVICE_STOP_PENDING: DWORD = 3;
pub const SERVICE_RUNNING: DWORD = 4;
pub const SERVICE_ACCEPT_STOP: DWORD = 1;
pub const SERVICE_ACCEPT_SHUTDOWN: DWORD = 4;
pub const SERVICE_CONTROL_STOP: DWORD = 1;
pub const SERVICE_CONTROL_SHUTDOWN: DWORD = 5;

extern "system" {
    pub fn SetServiceStatus(
        hServiceStatus: SERVICE_STATUS_HANDLE,
        lpServiceStatus: LPSERVICE_STATUS,
    ) -> BOOL;
    pub fn RegisterServiceCtrlHandlerExW(
        lpServiceName: *const u16,
        lpHandlerProc: LPHANDLER_FUNCTION_EX,
        lpContext: LPVOID,
    ) -> SERVICE_STATUS_HANDLE;
    pub fn StartServiceCtrlDispatcherW(lpServiceStartTable: *const SERVICE_TABLE_ENTRYW) -> BOOL;
}

// ================= ws2_32 (WSAAPI → extern "system") ===========================================
// NOTE: SOCKET is UINT_PTR — 32-bit on i386 but 64-bit on amd64/arm64. The C keeps its sockets in
// `int` (see `static int sock` and `client_t.socket`), so every call site truncates on Win64
// exactly as the C compiler does. Kept faithful; see the report on this hazard.
extern "system" {
    pub fn socket(af: c_int, ty: c_int, protocol: c_int) -> SOCKET;
    pub fn bind(s: SOCKET, addr: *const sockaddr, namelen: c_int) -> c_int;
    pub fn listen(s: SOCKET, backlog: c_int) -> c_int;
    pub fn accept(s: SOCKET, addr: *mut sockaddr, addrlen: *mut c_int) -> SOCKET;
    pub fn send(s: SOCKET, buf: *const c_char, len: c_int, flags: c_int) -> c_int;
    pub fn recv(s: SOCKET, buf: *mut c_char, len: c_int, flags: c_int) -> c_int;
    pub fn select(
        nfds: c_int,
        readfds: *mut fd_set,
        writefds: *mut fd_set,
        exceptfds: *mut fd_set,
        timeout: *const timeval,
    ) -> c_int;
    pub fn closesocket(s: SOCKET) -> c_int;
    pub fn htons(hostshort: u_short) -> u_short;
    pub fn gethostname(name: *mut c_char, namelen: c_int) -> c_int;
    pub fn WSAStartup(wVersionRequired: WORD, lpWSAData: *mut WSADATA) -> c_int;
    pub fn WSACleanup() -> c_int;
    pub fn WSAGetLastError() -> c_int;
}

/// winsock2.h: `#define INADDR_ANY (ULONG)0x00000000`.
pub const INADDR_ANY: ULONG = 0x0000_0000;
/// The island emits ws2def.h's IPPROTO enum with bindgen's type-name prefix.
pub use crate::types::IPPROTO_IPPROTO_TCP as IPPROTO_TCP;

/// winsock2.h: `#define FD_ZERO(set) (((fd_set FAR *)(set))->fd_count=0)`.
#[inline(always)]
pub unsafe fn FD_ZERO(set: *mut fd_set) {
    (*set).fd_count = 0;
}
/// winsock2.h `FD_SET(fd, set)`: append `fd` unless already present and there is room.
#[inline(always)]
pub unsafe fn FD_SET(fd: SOCKET, set: *mut fd_set) {
    let mut i: u_int = 0;
    while i < (*set).fd_count {
        if *(*set).fd_array.as_ptr().add(i as usize) == fd {
            break;
        }
        i += 1;
    }
    if i == (*set).fd_count && (*set).fd_count < FD_SETSIZE {
        *(*set).fd_array.as_mut_ptr().add(i as usize) = fd;
        (*set).fd_count += 1;
    }
}

// ================= windef.h / strsafe.h inlines ================================================
/// windef.h: `#define MAKEWORD(a,b) ((WORD)(((BYTE)(a))|(((WORD)((BYTE)(b)))<<8)))`.
#[inline(always)]
pub const fn MAKEWORD(a: u8, b: u8) -> WORD {
    (a as WORD) | ((b as WORD) << 8)
}

/// winerror.h: `#define S_OK ((HRESULT)0L)`.
pub const S_OK: HRESULT = 0;
/// strsafe.h `STRSAFE_MAX_CCH`.
pub const STRSAFE_MAX_CCH: usize = 2147483647;
/// strsafe.h `STRSAFE_E_INVALID_PARAMETER` / `STRSAFE_E_INSUFFICIENT_BUFFER`.
pub const STRSAFE_E_INVALID_PARAMETER: HRESULT = 0x80070057u32 as HRESULT;
pub const STRSAFE_E_INSUFFICIENT_BUFFER: HRESULT = 0x8007007Au32 as HRESULT;

/// strsafe.h `StringLengthWorkerA` (a header `static __inline`, so it has no exported symbol).
#[inline]
unsafe fn StringLengthWorkerA(psz: *const c_char, cchMax: usize, pcchLength: *mut usize) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut psz = psz;
    let mut cchMax = cchMax;
    let cchMaxPrev = cchMax;
    while cchMax != 0 && *psz != 0 {
        psz = psz.add(1);
        cchMax -= 1;
    }
    if cchMax == 0 {
        hr = STRSAFE_E_INVALID_PARAMETER;
    }
    if !pcchLength.is_null() {
        if hr >= 0 {
            *pcchLength = cchMaxPrev - cchMax;
        } else {
            *pcchLength = 0;
        }
    }
    hr
}

/// strsafe.h `StringCopyWorkerA`.
#[inline]
unsafe fn StringCopyWorkerA(pszDest: *mut c_char, cchDest: usize, pszSrc: *const c_char) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    if cchDest == 0 {
        hr = STRSAFE_E_INVALID_PARAMETER;
    } else {
        let mut pszDest = pszDest;
        let mut pszSrc = pszSrc;
        let mut cchDest = cchDest;
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

/// strsafe.h `StringCchCatA` = bounds check + `StringCatWorkerA` (length then copy at the tail).
/// A header inline in C, so it is a Rust fn here, not a link-time import.
#[inline]
pub unsafe fn StringCchCatA(pszDest: *mut c_char, cchDest: usize, pszSrc: *const c_char) -> HRESULT {
    if cchDest > STRSAFE_MAX_CCH {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    let mut cchDestLength: usize = 0;
    let hr = StringLengthWorkerA(pszDest, cchDest, &mut cchDestLength);
    if hr >= 0 {
        return StringCopyWorkerA(
            pszDest.add(cchDestLength),
            cchDest - cchDestLength,
            pszSrc,
        );
    }
    hr
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}
