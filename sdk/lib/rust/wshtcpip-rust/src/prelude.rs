//! wshtcpip-rust prelude — the single import surface the translated modules use.
//!
//! Same role as ws2_32-rust's prelude: rustconv emits per-file `extern` blocks and bare
//! macro/enum names, so this bridges the three gaps between the bindgen `types` island and
//! the translated C:
//!   1. bindgen RUSTIFIED the C enums (`_SOCKET_STATE_SocketStateCreated`) — alias the bare
//!      names the C source (and translation) actually use.
//!   2. bindgen SKIPS cast-#defines (IOCTL_TCP_*, SIO_*, IPPROTO_*, INADDR_*) — transcribe
//!      them, values verified against the exact header macro chain (see each citation).
//!   3. C MACROS/inlines (NULL, RtlZeroMemory) and wine's intrusive `struct list` have NO
//!      linkable symbol — implement them here (never as an out-of-line no-op `extern fn`,
//!      the field-manual §9.65 trap).
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
use core::ffi::c_void;

// ---- 0. Imports declared ONCE (ws2_32's "one declaration, one truth" — the c2rust output
// re-declared each per file, and they DISAGREED: openTcpFile as both C and system, HeapAlloc
// as usize and SIZE_T, which the clashing_extern lint flags). Convention is per the exporting
// module, NOT guessable from the C: kernel32/ws2_32 Win32 API is stdcall (extern "system" →
// `_Name@N` on i386); the CRT and tdilib's plain-C helpers are cdecl (extern "C" → `_Name`).
// c2rust put tdilib's openTcpFile/closeTcpFile in "system" — a silent i386 stack imbalance,
// since tdilib compiles them cdecl (sdk/lib/tdilib/handle.c: `NTSTATUS openTcpFile(...)`, no
// WINAPI). ----
extern "system" {
    pub fn DisableThreadLibraryCalls(hLibModule: HMODULE) -> BOOL;
    pub fn DeviceIoControl(
        hDevice: HANDLE,
        dwIoControlCode: DWORD,
        lpInBuffer: LPVOID,
        nInBufferSize: DWORD,
        lpOutBuffer: LPVOID,
        nOutBufferSize: DWORD,
        lpBytesReturned: LPDWORD,
        lpOverlapped: LPVOID,
    ) -> BOOL;
    pub fn GetLastError() -> DWORD;
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: usize) -> LPVOID;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    pub fn ntohl(netlong: u_long) -> u_long; // ws2_32.spec:14 stdcall
    pub fn ntohs(netshort: u16) -> u16; // ws2_32.spec:15 stdcall
}
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn openTcpFile(TcpFile: *mut HANDLE, DesiredAccess: ACCESS_MASK) -> NTSTATUS;
    pub fn closeTcpFile(TcpFile: HANDLE);
    pub fn tdiGetEntityIDSet(
        tcpFile: HANDLE,
        entityIDs: *mut *mut TDIEntityID,
        entityCount: *mut DWORD,
    ) -> NTSTATUS;
    pub fn tdiFreeThingSet(things: PVOID);
}

// ---- 1. Enum constants: bindgen prefixes them with the enum type name; the C uses the
// bare member name. Alias each bare name to the generated one so both resolve identically. ----
pub use crate::types::{
    _SOCKET_STATE_SocketStateBound as SocketStateBound,
    _SOCKET_STATE_SocketStateConnected as SocketStateConnected,
    _SOCKET_STATE_SocketStateCreated as SocketStateCreated,
    _SOCKET_STATE_SocketStateListening as SocketStateListening,
};
pub use crate::types::{
    _SOCKADDR_ADDRESS_INFO_SockaddrAddressInfoBroadcast as SockaddrAddressInfoBroadcast,
    _SOCKADDR_ADDRESS_INFO_SockaddrAddressInfoLoopback as SockaddrAddressInfoLoopback,
    _SOCKADDR_ADDRESS_INFO_SockaddrAddressInfoNormal as SockaddrAddressInfoNormal,
    _SOCKADDR_ADDRESS_INFO_SockaddrAddressInfoWildcard as SockaddrAddressInfoWildcard,
};
pub use crate::types::{
    _SOCKADDR_ENDPOINT_INFO_SockaddrEndpointInfoNormal as SockaddrEndpointInfoNormal,
    _SOCKADDR_ENDPOINT_INFO_SockaddrEndpointInfoReserved as SockaddrEndpointInfoReserved,
    _SOCKADDR_ENDPOINT_INFO_SockaddrEndpointInfoWildcard as SockaddrEndpointInfoWildcard,
};

// WINSOCK_MAPPING's inner triple struct is anonymous in C, so bindgen names it
// `_WINSOCK_MAPPING__bindgen_ty_1`; the translation calls it WINSOCK_MAPPING_TRIPLE.
pub use crate::types::_WINSOCK_MAPPING__bindgen_ty_1 as WINSOCK_MAPPING_TRIPLE;

// ---- 2. Constants bindgen skips (cast-#defines) — values verified against the header
// macro chain with a standalone C program (scratchpad/ioctl_vals.c), not hand-arithmetic. ----
// tcpioctl.h: _TCP_CTL_CODE(fn,method,access) = CTL_CODE(FSCTL_TCP_BASE=FILE_DEVICE_NETWORK=0x12,…)
pub const IOCTL_TCP_QUERY_INFORMATION_EX: DWORD = 0x0012_0003; // (0,METHOD_NEITHER,FILE_ANY_ACCESS)
pub const IOCTL_TCP_SET_INFORMATION_EX: DWORD = 0x0012_8004; // (1,METHOD_BUFFERED,FILE_WRITE_ACCESS)
// ws2ipdef.h:62 SIO_GET_INTERFACE_LIST = _IOR('t',127,ULONG); ULONG is 4 bytes on Windows
// (NOT the host's 8-byte long — that width difference changes the encoded size field).
pub const SIO_GET_INTERFACE_LIST: DWORD = 0x4004_747F;
// winsock.h:250-255 / ws2def.h — protocol numbers. u32 to match the bindgen `IPPROTO_IP`
// family (types uses them as `match Level as u32` arms alongside IPPROTO_IP); INT contexts
// (`… as DWORD` sockopt-level writes) cast at the site.
pub const IPPROTO_ICMP: u32 = 1;
pub const IPPROTO_TCP: u32 = 6;
pub const IPPROTO_UDP: u32 = 17;
// winsock.h:305/307 — (u_long)0 / (u_long)0xffffffff.
pub const INADDR_ANY: u_long = 0;
pub const INADDR_BROADCAST: u_long = 0xFFFF_FFFF;
// ipifcons.h:241 INTERNAL_IF_OPER_STATUS — the OLD MIB oper-status enum (0=NON_OPERATIONAL,
// 3=CONNECTING, 4=CONNECTED, 5=OPERATIONAL), which is what an IFEntry.if_operstatus (a TDI
// MIB row) carries. This is a DIFFERENT enum from the ifdef.h NET_IF_OPER_STATUS that
// bindgen emitted as `IF_OPER_STATUS_IfOperStatus*` — do NOT conflate them; the C's
// `if_operstatus < IF_OPER_STATUS_CONNECTING` down-test needs the MIB value 3.
pub const IF_OPER_STATUS_CONNECTING: IF_OPER_STATUS = 3;

// ---- 3a. NULL: the C uses it as a null pointer literal in pointer contexts; the type is
// inferred per site, so a typed const would E0308. `core::ptr::null_mut()` is the site-
// inferred equivalent, but a plain `pub const NULL` cannot carry a fresh type per use — so
// the modules sed `NULL` → `core::ptr::null_mut()` at integration; nothing to define here. ----

// ---- 3a'. NT_SUCCESS: a DDK macro (`(NTSTATUS)(x) >= 0`), no symbol; the c2rust output
// calls it as a fn. Mirrors ws2_32-rust's prelude. (A macro would not survive the double
// glob re-export types→prelude, and ntdll-sys exposes it only as a macro.) ----
#[inline(always)]
pub fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

// ---- 3b. RtlZeroMemory: a write_bytes macro in the DDK; no symbol to link. `len` is a
// byte count → usize (most c2rust call sites pass `size_of::<T>()`; the two that wrote
// `as ULONG` are cast to `as usize` at the site). ----
#[inline(always)]
pub unsafe fn RtlZeroMemory(dst: *mut c_void, len: usize) {
    core::ptr::write_bytes(dst as *mut u8, 0, len);
}

// ---- 3c. wine's intrusive doubly-linked `struct list` (wine/list.h): the shared port. ----
pub use winelist_rust::{list, list_add_head, list_init, list_remove};
