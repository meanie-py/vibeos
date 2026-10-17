//! dhcpcsvc-rust prelude — the single import surface the translated modules use.
//!
//! Bridges the bindgen `types` island (rosdhcp/dhcpd/dhcpcsvc layouts) and the translated C:
//!   1. DbgPrint (ntdll variadic) — DeepSeek lowered the ROS DPRINT macro to DbgPrint calls.
//!   2. the CRT + advapi32 registry + ws2_32 socket/inet + ntdll Rtl + the dhcpcsvc MIDL
//!      Client_/Server_ stubs — declared once here, per the exporting module's convention.
//!   3. consts/inlines bindgen skips (NULL, NT_SUCCESS).
//!   4. bridges the cargo-check grind revealed: enum/struct aliases, winsock/CRT externs, the
//!      dhclient char predicate macros, and the intra-crate helpers each module exports.
//! bindgen (profile: dhcpcsvc) already supplies the DHO_*/DHCP_* option consts, ERROR_*/REG_*/
//! KEY_* and the advapi32/kernel32/rpcrt4/iphlpapi *function* decls — only the surfaces bindgen
//! left out (winsock2.h, the CRT, the ISC dhclient internals) are declared below.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};

// ================= 1. debug =====================================================================
// DbgPrint is a real ntdll cdecl-variadic export (declaring a C-variadic is stable Rust). The ROS
// DPRINT macro that DeepSeek left as `DPRINT(...)` fn-calls is rewritten to `DPRINT!(...)` in the
// grind; this no-op macro absorbs them (NDEBUG is #defined in the C, so DPRINT is compiled out).
extern "C" {
    pub fn DbgPrint(Format: *const c_char, ...) -> u32;
}
#[macro_export]
macro_rules! DPRINT {
    ($($arg:tt)*) => {{}};
}
// DPRINT1 / DPRINT2 / DH_DbgPrint are the other ROS debug channels. All are MACROS, never
// out-of-line fns (field-manual §9.65: an out-of-line no-op is an ABI barrier). Callers use
// fn-call syntax `DPRINT1(...)`, rewritten to `DPRINT1!(...)` in the grind.
//
// DPRINT1 is the one that is NOT a no-op. debug.h puts it under `#if DBG` beneath the comment
// "These are always printed"; only DPRINT is further gated on `#ifndef NDEBUG`. Our builds set
// DBG=TRUE, so every DPRINT1 in the C this replaced prints, and a no-op deletes a diagnostic
// from exactly the failure path that needed one.
#[macro_export]
macro_rules! DPRINT1 {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {{
        unsafe { $crate::prelude::DbgPrint(concat!($fmt, "\0").as_ptr() as *const _ $(, $arg)*) };
    }};
    // Some grind output already lowered the format to a NUL-terminated `*const c_char`
    // (dhcpcsvc's util.rs writes `"ERROR: %s\n\0".as_ptr() as *const i8`). Tried second, so a
    // plain literal still takes the arm above and gets its NUL appended exactly once.
    ($fmt:expr $(, $arg:expr)* $(,)?) => {{
        unsafe { $crate::prelude::DbgPrint($fmt $(, $arg)*) };
    }};
}
#[macro_export]
macro_rules! DPRINT2 {
    ($($arg:tt)*) => {{}};
}
#[macro_export]
macro_rules! DH_DbgPrint {
    ($($arg:tt)*) => {{}};
}
pub use crate::{DH_DbgPrint, DPRINT, DPRINT1, DPRINT2};
pub use core::ptr::{addr_of, addr_of_mut};

// ================= 2. CRT (cdecl, extern "C") — msvcrt/ntdll exports; dhcpcsvc.dll links msvcrt ==
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
    pub fn malloc(size: usize) -> *mut c_void;
    pub fn calloc(nmemb: usize, size: usize) -> *mut c_void;
    pub fn free(ptr: *mut c_void);
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strcat(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strncat(dst: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    pub fn strncpy(dst: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    pub fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
    pub fn mbstowcs(dst: *mut u16, src: *const c_char, n: usize) -> usize;
    pub fn atol(s: *const c_char) -> c_long;
    pub fn strtol(s: *const c_char, endptr: *mut *mut c_char, base: c_int) -> c_long;
    pub fn rand() -> c_int;
    pub fn srand(seed: c_uint);
    pub fn isascii(c: c_int) -> c_int;
    pub fn isprint(c: c_int) -> c_int;
    pub fn tzset();
    pub fn strtok(s: *mut c_char, delim: *const c_char) -> *mut c_char;
    pub fn wcsicmp(a: *const u16, b: *const u16) -> c_int;
    pub fn time(t: *mut time_t) -> time_t;
    pub fn gmtime(t: *const time_t) -> *mut tm;
    // stdio (leaseFile in dhclient write_client_lease). FILE is opaque.
    pub fn fopen(path: *const c_char, mode: *const c_char) -> *mut FILE;
    pub fn fflush(stream: *mut FILE) -> c_int;
    pub fn rewind(stream: *mut FILE);
    // C-variadic CRT — declaring & calling a variadic is stable Rust (only *defining* one is not).
    pub fn _snprintf(buf: *mut c_char, count: usize, format: *const c_char, ...) -> c_int;
    pub fn sprintf(buf: *mut c_char, format: *const c_char, ...) -> c_int;
    pub fn fprintf(stream: *mut FILE, format: *const c_char, ...) -> c_int;
    // _vsnprintf takes a va_list (char* on all three windows-gnu targets), NOT a `...` list.
    pub fn _vsnprintf(buf: *mut c_char, count: usize, format: *const c_char, args: *mut c_char) -> c_int;
}

// ================= 3. dhclient internal log helpers (util.c, C-variadic, cdecl) =================
// These are REAL translated functions: the formatting logic lives in util.rs as fixed-arity `*_v`
// twins (a va_list cannot be `va_start`ed from stable Rust — c_variadic, rust-lang#44930). The
// variadic entry points that callers invoke are supplied by `rust_varargs_shim.c` (the sanctioned
// varargs-rust trampoline; see AGENTS.md "c_variadic"). Declaring & calling the variadic entry is
// stable Rust, so callers resolve here; the entry symbol is provided by the shim at DLL-link time.
extern "C" {
    pub fn note(format: *const c_char, ...) -> c_int;
    pub fn warn(format: *const c_char, ...) -> c_int;
    pub fn warning(format: *const c_char, ...) -> c_int;
    pub fn debug(format: *const c_char, ...) -> c_int;
    pub fn error(format: *const c_char, ...);
}

// ================= 4. Win32 (stdcall, extern "system" → `_Name@N` on i386) =======================
extern "system" {
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: usize) -> *mut c_void;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: *mut c_void) -> BOOL;
}

// 4a. ws2_32 (winsock, __stdcall). winsock2.h is NOT in the bindgen profile, so the socket layer
//     is declared here. `in_addr`/`sockaddr` are bindgen base types (used pervasively in the
//     rosdhcp layouts). inet_ntoa takes `struct in_addr` BY VALUE.
extern "system" {
    pub fn socket(af: c_int, kind: c_int, protocol: c_int) -> SOCKET;
    pub fn bind(s: SOCKET, addr: *const sockaddr, namelen: c_int) -> c_int;
    pub fn sendto(s: SOCKET, buf: *const c_char, len: c_int, flags: c_int, to: *const sockaddr, tolen: c_int) -> c_int;
    pub fn recvfrom(s: SOCKET, buf: *mut c_char, len: c_int, flags: c_int, from: *mut sockaddr, fromlen: *mut c_int) -> c_int;
    pub fn setsockopt(s: SOCKET, level: c_int, optname: c_int, optval: *const c_char, optlen: c_int) -> c_int;
    pub fn closesocket(s: SOCKET) -> c_int;
    pub fn htons(hostshort: u16) -> u16;
    pub fn htonl(hostlong: u32) -> u32;
    pub fn ntohs(netshort: u16) -> u16;
    pub fn ntohl(netlong: u32) -> u32;
    pub fn inet_ntoa(addr: in_addr) -> *mut c_char;
    pub fn WSAStartup(wVersionRequested: WORD, lpWSAData: *mut WSADATA) -> c_int;
    pub fn WSACleanup() -> c_int;
    pub fn WSAGetLastError() -> c_int;
    // iphlpapi — IP_ADAPTER_INFO / MIB_IFTABLE are declared below (not in the bindgen profile).
    pub fn GetAdaptersInfo(pAdapterInfo: PIP_ADAPTER_INFO, pOutBufLen: *mut ULONG) -> DWORD;
    pub fn GetIfTable(pIfTable: *mut MIB_IFTABLE, pdwSize: *mut ULONG, bOrder: BOOL) -> DWORD;
}

// 4a2. kernel32 / advapi32 / iphlpapi / rpcrt4 (__stdcall). The bindgen island supplies the TYPES
//      (it globs `ntdll_sys::nt::*` — HKEY, CRITICAL_SECTION, DWORD, …) but only ntdll's functions,
//      so the Win32 API surface dhcpcsvc.dll imports is declared here. Signatures per the psdk
//      headers; `A`-suffixed entry points are bound to the undecorated C spelling with #[link_name]
//      (the C compiles without UNICODE, so `RegQueryValueEx` ≡ `RegQueryValueExA`).
extern "system" {
    // --- kernel32 ---
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn CreateEventW(
        lpEventAttributes: *mut c_void,
        bManualReset: BOOL,
        bInitialState: BOOL,
        lpName: *const u16,
    ) -> HANDLE;
    #[link_name = "CreateEventA"]
    pub fn CreateEvent(
        lpEventAttributes: *mut c_void,
        bManualReset: BOOL,
        bInitialState: BOOL,
        lpName: *const c_char,
    ) -> HANDLE;
    pub fn CreateThread(
        lpThreadAttributes: *mut c_void,
        dwStackSize: usize,
        lpStartAddress: Option<unsafe extern "system" fn(LPVOID) -> DWORD>,
        lpParameter: LPVOID,
        dwCreationFlags: DWORD,
        lpThreadId: *mut DWORD,
    ) -> HANDLE;
    pub fn InitializeCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn DeleteCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn EnterCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn LeaveCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn DisableThreadLibraryCalls(hLibModule: HANDLE) -> BOOL;
    pub fn GetLastError() -> DWORD;
    pub fn GetTickCount() -> DWORD;
    #[link_name = "GetComputerNameA"]
    pub fn GetComputerName(lpBuffer: *mut c_char, nSize: *mut DWORD) -> BOOL;
    pub fn SetEvent(hEvent: HANDLE) -> BOOL;
    pub fn Sleep(dwMilliseconds: DWORD);
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
    pub fn WaitForMultipleObjects(
        nCount: DWORD,
        lpHandles: *const HANDLE,
        bWaitAll: BOOL,
        dwMilliseconds: DWORD,
    ) -> DWORD;

    // --- advapi32: registry ---
    pub fn RegCloseKey(hKey: HKEY) -> DWORD;
    pub fn RegOpenKeyExA(
        hKey: HKEY,
        lpSubKey: *const c_char,
        ulOptions: DWORD,
        samDesired: DWORD,
        phkResult: *mut HKEY,
    ) -> DWORD;
    #[link_name = "RegOpenKeyA"]
    pub fn RegOpenKey(hKey: HKEY, lpSubKey: *const c_char, phkResult: *mut HKEY) -> DWORD;
    pub fn RegCreateKeyExA(
        hKey: HKEY,
        lpSubKey: *const c_char,
        Reserved: DWORD,
        lpClass: *mut c_char,
        dwOptions: DWORD,
        samDesired: DWORD,
        lpSecurityAttributes: *mut c_void,
        phkResult: *mut HKEY,
        lpdwDisposition: *mut DWORD,
    ) -> DWORD;
    pub fn RegSetValueExA(
        hKey: HKEY,
        lpValueName: *const c_char,
        Reserved: DWORD,
        dwType: DWORD,
        lpData: *const BYTE,
        cbData: DWORD,
    ) -> DWORD;
    #[link_name = "RegQueryValueExA"]
    pub fn RegQueryValueEx(
        hKey: HKEY,
        lpValueName: *const c_char,
        lpReserved: *mut DWORD,
        lpType: *mut DWORD,
        lpData: *mut BYTE,
        lpcbData: *mut DWORD,
    ) -> DWORD;
    pub fn RegDeleteValueA(hKey: HKEY, lpValueName: *const c_char) -> DWORD;
    pub fn RegDeleteValueW(hKey: HKEY, lpValueName: *const u16) -> DWORD;
    #[link_name = "RegEnumKeyA"]
    pub fn RegEnumKey(hKey: HKEY, dwIndex: DWORD, lpName: *mut c_char, cchName: DWORD) -> DWORD;
    #[link_name = "RegQueryInfoKeyA"]
    pub fn RegQueryInfoKey(
        hKey: HKEY,
        lpClass: *mut c_char,
        lpcchClass: *mut DWORD,
        lpReserved: *mut DWORD,
        lpcSubKeys: *mut DWORD,
        lpcbMaxSubKeyLen: *mut DWORD,
        lpcbMaxClassLen: *mut DWORD,
        lpcValues: *mut DWORD,
        lpcbMaxValueNameLen: *mut DWORD,
        lpcbMaxValueLen: *mut DWORD,
        lpcbSecurityDescriptor: *mut DWORD,
        lpftLastWriteTime: *mut FILETIME,
    ) -> DWORD;

    // --- advapi32: service control ---
    pub fn RegisterServiceCtrlHandlerExW(
        lpServiceName: *const u16,
        lpHandlerProc: Option<unsafe extern "system" fn(DWORD, DWORD, LPVOID, LPVOID) -> DWORD>,
        lpContext: LPVOID,
    ) -> SERVICE_STATUS_HANDLE;
    pub fn SetServiceStatus(
        hServiceStatus: SERVICE_STATUS_HANDLE,
        lpServiceStatus: *mut SERVICE_STATUS,
    ) -> BOOL;

    // --- iphlpapi ---
    pub fn AddIPAddress(
        Address: u32,
        IpMask: u32,
        IfIndex: DWORD,
        NTEContext: *mut ULONG,
        NTEInstance: *mut ULONG,
    ) -> DWORD;
    pub fn DeleteIPAddress(NTEContext: ULONG) -> DWORD;
    pub fn CreateIpForwardEntry(pRoute: *mut MIB_IPFORWARDROW) -> DWORD;
    pub fn DeleteIpForwardEntry(pRoute: *mut MIB_IPFORWARDROW) -> DWORD;
    pub fn SendARP(DestIP: u32, SrcIP: u32, pMacAddr: *mut c_void, PhyAddrLen: *mut ULONG) -> DWORD;

    // --- ws2_32 event objects ---
    pub fn WSACreateEvent() -> HANDLE;
    pub fn WSACloseEvent(hEvent: HANDLE) -> BOOL;
    pub fn WSAResetEvent(hEvent: HANDLE) -> BOOL;
    pub fn WSAEventSelect(s: SOCKET, hEventObject: HANDLE, lNetworkEvents: c_long) -> c_int;

    // --- rpcrt4 ---
    pub fn RpcServerUseProtseqEpW(
        Protseq: *mut u16,
        MaxCalls: c_uint,
        Endpoint: *mut u16,
        SecurityDescriptor: *mut c_void,
    ) -> DWORD;
    pub fn RpcServerRegisterIf(
        IfSpec: *mut c_void,
        MgrTypeUuid: *mut c_void,
        MgrEpv: *mut c_void,
    ) -> DWORD;
    pub fn RpcServerListen(MinimumCallThreads: c_uint, MaxCalls: c_uint, DontWait: c_uint) -> DWORD;
    pub fn RpcMgmtStopServerListening(Binding: *mut c_void) -> DWORD;
    pub fn RpcStringBindingComposeW(
        ObjUuid: *mut u16,
        ProtSeq: *mut u16,
        NetworkAddr: *mut u16,
        Endpoint: *mut u16,
        Options: *mut u16,
        StringBinding: *mut *mut u16,
    ) -> DWORD;
    pub fn RpcBindingFromStringBindingW(StringBinding: *mut u16, Binding: *mut *mut c_void) -> DWORD;
    pub fn RpcBindingFree(Binding: *mut *mut c_void) -> DWORD;
    pub fn RpcStringFreeW(String: *mut *mut u16) -> DWORD;
}

// 4b. dhcpcsvc MIDL client stubs (dhcpcsvc_c.c, __stdcall). Stay C; declared for the Rust callers
//     in dhcpcsvc.rs. PDHCP_SERVER_NAME = wchar_t* (bindgen supplies it). Signatures verbatim from
//     build-*/base/services/dhcpcsvc/dhcpcsvc_c.h.
extern "system" {
    pub fn Client_AcquireParameters(ServerName: PDHCP_SERVER_NAME, AdapterName: LPWSTR) -> DWORD;
    pub fn Client_ReleaseParameters(ServerName: PDHCP_SERVER_NAME, AdapterName: LPWSTR) -> DWORD;
    pub fn Client_QueryHWInfo(
        ServerName: PDHCP_SERVER_NAME,
        AdapterIndex: DWORD,
        MediaType: PDWORD,
        Mtu: PDWORD,
        Speed: PDWORD,
    ) -> DWORD;
    pub fn Client_StaticRefreshParams(
        ServerName: PDHCP_SERVER_NAME,
        AdapterIndex: DWORD,
        Address: DWORD,
        Netmask: DWORD,
    ) -> DWORD;
    pub fn Client_RemoveDNSRegistrations(ServerName: PDHCP_SERVER_NAME) -> DWORD;
    // RPC interface handles (extern data from the generated stubs).
    pub static Client_dhcpcsvc_v0_0_c_ifspec: *mut c_void;
    pub static Server_dhcpcsvc_v0_0_s_ifspec: *mut c_void;
}

// ================= 5. consts / inlines bindgen skips ============================================
pub const NULL: *mut c_void = core::ptr::null_mut();
pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;

#[inline(always)]
pub fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

// 5a. winsock2.h / ws2def.h / winsock.h consts (profile omitted winsock2.h).
pub const AF_INET: c_int = 2; // winsock2.h — #define AF_INET 2
pub const SOCK_DGRAM: c_int = 2; // winsock2.h — #define SOCK_DGRAM 2
pub const SOCK_RAW: c_int = 3; // winsock2.h — #define SOCK_RAW 3
pub const IPPROTO_UDP: c_int = 17; // ws2def.h — #define IPPROTO_UDP 17
pub const SOL_SOCKET: c_int = 0xffff; // winsock2.h — #define SOL_SOCKET 0xffff
pub const SO_BROADCAST: c_int = 0x0020; // winsock2.h — #define SO_BROADCAST 0x0020
pub const SO_REUSEADDR: c_int = 0x0004; // winsock2.h — #define SO_REUSEADDR 0x0004
pub const INADDR_ANY: u32 = 0x0000_0000; // winsock.h — (u_long)0
pub const INADDR_BROADCAST: u32 = 0xffff_ffff; // winsock.h — (u_long)0xffffffff
pub const INADDR_NONE: u32 = 0xffff_ffff; // winsock.h — 0xffffffff
pub const INVALID_SOCKET: SOCKET = !0; // winsock2.h — (SOCKET)(~0)
pub const WSAEMSGSIZE: c_int = 10040; // winsock2.h — WSABASEERR+40

// 5b. handle / service / rpc / iphlpapi consts.
pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE; // handleapi.h — (HANDLE)(LONG_PTR)-1
pub const SERVICE_STOPPED: DWORD = 1; // winsvc.h
pub const SERVICE_START_PENDING: DWORD = 2; // winsvc.h
pub const SERVICE_STOP_PENDING: DWORD = 3; // winsvc.h
pub const SERVICE_RUNNING: DWORD = 4; // winsvc.h
pub const SERVICE_ACCEPT_STOP: DWORD = 1; // winsvc.h — SERVICE_ACCEPT_STOP
pub const SERVICE_ACCEPT_SHUTDOWN: DWORD = 4; // winsvc.h — SERVICE_ACCEPT_SHUTDOWN
pub const SERVICE_CONTROL_STOP: DWORD = 1; // winsvc.h
pub const SERVICE_CONTROL_SHUTDOWN: DWORD = 5; // winsvc.h
pub const SERVICE_WIN32_OWN_PROCESS: DWORD = 0x00000010; // winsvc.h
pub const RPC_S_OK: c_long = 0; // rpcdce.h — RPC_S_OK == ERROR_SUCCESS
pub const RPC_C_LISTEN_MAX_CALLS_DEFAULT: c_uint = 1234; // rpcdce.h
pub const IF_OPER_STATUS_CONNECTED: DWORD = 4; // ipifcons.h — IF_OPER_STATUS_CONNECTED
pub const IF_OPER_STATUS_OPERATIONAL: DWORD = 5; // ipifcons.h — IF_OPER_STATUS_OPERATIONAL
pub const IF_TYPE_IEEE80211: DWORD = 71; // ipifcons.h — #define IF_TYPE_IEEE80211 71
pub const MIB_IF_TYPE_ETHERNET: DWORD = 6; // ipifcons.h — #define MIB_IF_TYPE_ETHERNET 6
pub const MAX_COMPUTERNAME_LENGTH: usize = 15; // winbase.h — #define MAX_COMPUTERNAME_LENGTH 15

// HKEY predefined keys (winreg.h). HKEY is bindgen-supplied (a raw pointer handle).
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY; // winreg.h
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as i32 as isize as HKEY; // winreg.h

// ================= 6. types bindgen skipped (declared, per the dnsapi/iphlpapi pattern) =========
// NOTE: `time_t` deliberately NOT redefined here — ntdll_sys::nt (globbed via `types`)
// supplies the ARCH-CORRECT CRT type (__time64_t on amd64/arm64, __time32_t on i386).
// Hardcoding it to i64 silently broke every `time(&mut cur_time)` on i386.
pub type SOCKET = usize; // winsock2.h — typedef UINT_PTR SOCKET
pub type SERVICE_STATUS_HANDLE = HANDLE; // winsvc.h — DECLARE_HANDLE(SERVICE_STATUS_HANDLE)

// opaque stdio FILE (zero-sized private field — a zero-variant enum can't carry #[repr(C)])
#[repr(C)]
pub struct FILE {
    _private: [u8; 0],
}

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

#[repr(C)]
#[derive(Copy, Clone)]
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

// WSADATA (winsock2.h) — the char-array vs counter field order differs by arch; the DHCP code only
// zeroes it and hands it to WSAStartup, but transcribe the real per-arch layout for correctness.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct WSADATA {
    pub wVersion: WORD,
    pub wHighVersion: WORD,
    #[cfg(target_pointer_width = "64")]
    pub iMaxSockets: c_ushort,
    #[cfg(target_pointer_width = "64")]
    pub iMaxUdpDg: c_ushort,
    #[cfg(target_pointer_width = "64")]
    pub lpVendorInfo: *mut c_char,
    pub szDescription: [c_char; 257],
    pub szSystemStatus: [c_char; 129],
    #[cfg(target_pointer_width = "32")]
    pub iMaxSockets: c_ushort,
    #[cfg(target_pointer_width = "32")]
    pub iMaxUdpDg: c_ushort,
    #[cfg(target_pointer_width = "32")]
    pub lpVendorInfo: *mut c_char,
}

// iptypes.h — GetAdaptersInfo family (profile omitted iptypes.h). IP_ADDR_STRING chain + the
// IP_ADAPTER_INFO node. The DHCP code walks .Next and reads .Index; the full layout is transcribed
// so size_of<IP_ADAPTER_INFO>() (used for the initial GetAdaptersInfo buffer) is correct.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct IP_ADDRESS_STRING {
    pub String: [c_char; 16],
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct IP_ADDR_STRING {
    pub Next: *mut IP_ADDR_STRING,
    pub IpAddress: IP_ADDRESS_STRING,
    pub IpMask: IP_ADDRESS_STRING,
    pub Context: DWORD,
}
pub type PIP_ADDR_STRING = *mut IP_ADDR_STRING;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct IP_ADAPTER_INFO {
    pub Next: *mut IP_ADAPTER_INFO,
    pub ComboIndex: DWORD,
    pub AdapterName: [c_char; 260],   // MAX_ADAPTER_NAME_LENGTH(256) + 4
    pub Description: [c_char; 132],    // MAX_ADAPTER_DESCRIPTION_LENGTH(128) + 4
    pub AddressLength: UINT,
    pub Address: [BYTE; 8],            // MAX_ADAPTER_ADDRESS_LENGTH
    pub Index: DWORD,
    pub Type: UINT,
    pub DhcpEnabled: UINT,
    pub CurrentIpAddress: PIP_ADDR_STRING,
    pub IpAddressList: IP_ADDR_STRING,
    pub GatewayList: IP_ADDR_STRING,
    pub DhcpServer: IP_ADDR_STRING,
    pub HaveWins: BOOL,
    pub PrimaryWinsServer: IP_ADDR_STRING,
    pub SecondaryWinsServer: IP_ADDR_STRING,
    pub LeaseObtained: time_t,
    pub LeaseExpires: time_t,
}
pub type PIP_ADAPTER_INFO = *mut IP_ADAPTER_INFO;

// iprtrmib.h — MIB_IFTABLE: count + trailing flexible array of MIB_IFROW (bindgen supplies MIB_IFROW).
// `table` is a `[MIB_IFROW; 1]` flex array — index it via `.as_ptr().add(i)` at the call sites, never
// `table[i]` (a bindgen `[T;1]` bounds-panics for i>=1).
#[repr(C)]
pub struct MIB_IFTABLE {
    pub dwNumEntries: DWORD,
    pub table: [MIB_IFROW; 1],
}
pub type PMIB_IFTABLE = *mut MIB_IFTABLE;

// ================= 7. BSD/ISC struct name aliases (code uses `_name`; bindgen emitted `name`) ====
pub type _tree_cache = tree_cache;
pub type _packet = packet;
pub type _interface_info = interface_info;
pub type _client_lease = client_lease;
pub type _timeout = timeout;
pub type _iaddrlist = iaddrlist;

// ================= 8. DHCP state enum members (bindgen prefixed them `dhcp_state_`) ==============
pub use crate::types::{
    dhcp_state_S_BOUND as S_BOUND,
    dhcp_state_S_INIT as S_INIT,
    dhcp_state_S_REBINDING as S_REBINDING,
    dhcp_state_S_REBOOTING as S_REBOOTING,
    dhcp_state_S_RELEASED as S_RELEASED,
    dhcp_state_S_RENEWING as S_RENEWING,
    dhcp_state_S_REQUESTING as S_REQUESTING,
    dhcp_state_S_SELECTING as S_SELECTING,
    dhcp_state_S_STATIC as S_STATIC,
    // two sites DeepSeek spelled with a stray enum-name prefix — alias those spellings too.
    dhcp_state_S_SELECTING as _dhcp_state_S_SELECTING,
    dhcp_state_S_STATIC as _DHCPState_S_STATIC,
};

// ================= 9. dhclient char-predicate macros (dhclient.c #defines) ======================
// Translated call sites use fn-call syntax `periodchar(ch)` and test the result C-style
// (`periodchar(ch) != 0`), so these return `c_int` like the C macros do — not `bool`. Generic over
// the integer argument (Into<i64>) so no per-site cast is needed. Values verbatim from dhclient.c.
#[inline(always)]
pub fn hyphenchar<T: Into<i64>>(c: T) -> c_int { (c.into() == 0x2d) as c_int }
#[inline(always)]
pub fn bslashchar<T: Into<i64>>(c: T) -> c_int { (c.into() == 0x5c) as c_int }
#[inline(always)]
pub fn periodchar<T: Into<i64>>(c: T) -> c_int { (c.into() == 0x2e) as c_int }
#[inline(always)]
pub fn asterchar<T: Into<i64>>(c: T) -> c_int { (c.into() == 0x2a) as c_int }
#[inline(always)]
pub fn alphachar<T: Into<i64>>(c: T) -> c_int {
    let c = c.into();
    ((0x41..=0x5a).contains(&c) || (0x61..=0x7a).contains(&c)) as c_int
}
#[inline(always)]
pub fn digitchar<T: Into<i64>>(c: T) -> c_int { (0x30..=0x39).contains(&c.into()) as c_int }
#[inline(always)]
pub fn borderchar<T: Into<i64>>(c: T) -> c_int {
    let c = c.into();
    (alphachar(c) != 0 || digitchar(c) != 0) as c_int
}
#[inline(always)]
pub fn middlechar<T: Into<i64>>(c: T) -> c_int {
    let c = c.into();
    (borderchar(c) != 0 || hyphenchar(c) != 0) as c_int
}
#[inline(always)]
pub fn domainchar<T: Into<i64>>(c: T) -> c_int {
    let c = c.into();
    (c > 0x20 && c < 0x7f) as c_int
}

// ================= 10. intra-crate helpers (each module's exported symbols) =====================
// The C compiled in one TU; in Rust each module is its own scope, so the shared helpers are made
// `pub` at their definition site and re-exported here (the dnsapi/netapi32 pattern). Callers invoke
// them bare, resolving through the prelude glob.
pub use crate::dhcp::util::{addr_eq, dmalloc, getLong, getShort, getULong, getUShort, piaddr, read_client_conf};
pub use crate::dhcp::compat::{arc4random, inet_aton, strlcpy};
pub use crate::dhcp::hash::{add_hash, new_hash};
pub use crate::dhcp::alloc::{dfree, free_hash_bucket, new_hash_bucket, new_hash_table};
pub use crate::dhcp::dispatch::{
    add_protocol, add_timeout, cancel_timeout, dispatch, find_protocol_by_adapter, got_one,
    remove_protocol,
};
pub use crate::dhcp::options::{cons_options, do_packet, pretty_print_option};
pub use crate::dhcp::socket::{receive_packet, send_packet};
pub use crate::dhcp::dhclient::{bootp, dhcp, init_client, state_init, state_reboot, state_release, stop_client};
pub use crate::dhcp::adapter::{
    AdapterFindByHardwareAddress, AdapterFindIndex, AdapterFindInfo, AdapterFindName, AdapterInit,
    DhcpSocket, StartAdapterDiscovery,
};
pub use crate::dhcp::api::{ApiInit, ApiLock, ApiUnlock, InitRpc, ShutdownRpc};
pub use crate::dhcpcsvc::dhcpcsvc::hAdapterStateChangedEvent;
