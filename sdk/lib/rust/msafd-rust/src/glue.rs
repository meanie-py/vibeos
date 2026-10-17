//! msafd #define consts / type aliases / macro shims the substrate (`types`) and
//! ntdll-sys don't provide. Winsock/FD/MSG constant values match sdk/include/psdk/
//! winsock2.h; the wine debug macros (TRACE/ERR/WARN/FIXME) are release-noop, matching
//! the C build (DPRINT-style, compiled out). FIELD_OFFSET/NT_SUCCESS are the SDK macros
//! the C source uses as expressions.

// --- wine debug channel macros (release no-op) ------------------------------
// The C uses TRACE/ERR/WARN/FIXME(fmt, ...) as statement calls; under the ReactOS
// release build they compile to nothing. Make them no-op macros (faithful + no link
// symbol). #[macro_use] on `mod glue` puts them in scope crate-wide. The translation
// also emitted `extern "C" { fn TRACE(...) }` decls in some files — those are deleted
// per-file during integration (the macro supersedes them).

// FIELD_OFFSET(type, field) — byte offset of a struct field (SDK macro used as a
// const expression in IOCTL buffer-size math). offset_of! is the Rust equivalent.
#[macro_export]
macro_rules! FIELD_OFFSET {
    ($ty:ty, $($field:tt)+) => { core::mem::offset_of!($ty, $($field)+) };
}

// NT_SUCCESS(Status) — Status >= 0 (top bit clear). The C macro used as a bool expr.
#[inline(always)]
pub fn NT_SUCCESS(status: i32) -> bool { status >= 0 }

use crate::types::*;
// One declaration per external symbol: the variant matching the C prototype.
pub use crate::misc::helpers::HeapFree;
use core::ffi::c_void;

// --- winsock cast-macros bindgen can't emit (they use casts, not plain #define) --------
// INVALID_SOCKET = (SOCKET)(~0); SOCKET_ERROR = -1; INVALID_HANDLE_VALUE = (HANDLE)-1.
pub const INVALID_SOCKET: SOCKET = !0 as SOCKET;
pub const INVALID_HANDLE_VALUE: HANDLE = !0isize as HANDLE;
// (SOCKET_ERROR = -1 is a plain #define bindgen already emits in `types`; STATUS_SUCCESS
// comes from ntdll_sys::nt — both resolve via the substrate glob, so NOT redefined here.)
// IO_COMPLETION_ALL_ACCESS = STANDARD_RIGHTS_REQUIRED|SYNCHRONIZE|0x3 = 0x001F0003.
pub const IO_COMPLETION_ALL_ACCESS: u32 = 0x001F0003;
// HKEY_LOCAL_MACHINE = (HKEY)0x80000002 — a winreg.h macro constant, not a DLL import.
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;

// --- SOCKET_STATE / IPPROTO bare-name aliases -------------------------------------------
// bindgen constifies the enum with prefixed names (_SOCKET_STATE_SocketOpen /
// IPPROTO_IPPROTO_TCP); the C source uses the bare names. Alias to the substrate's typed
// consts so the field-comparison types (State: SOCKET_STATE, Protocol: IPPROTO) still match.
pub const SocketOpen: SOCKET_STATE = _SOCKET_STATE_SocketOpen;
pub const SocketBound: SOCKET_STATE = _SOCKET_STATE_SocketBound;
pub const SocketBoundUdp: SOCKET_STATE = _SOCKET_STATE_SocketBoundUdp;
pub const SocketConnected: SOCKET_STATE = _SOCKET_STATE_SocketConnected;
pub const SocketClosed: SOCKET_STATE = _SOCKET_STATE_SocketClosed;
pub const SockaddrAddressInfoBroadcast: SOCKADDR_ADDRESS_INFO = _SOCKADDR_ADDRESS_INFO_SockaddrAddressInfoBroadcast;
pub const SockaddrEndpointInfoWildcard: SOCKADDR_ENDPOINT_INFO = _SOCKADDR_ENDPOINT_INFO_SockaddrEndpointInfoWildcard;

// WSAEVENT = HANDLE (winsock2.h typedef bindgen doesn't surface as a bare alias).
pub type WSAEVENT = HANDLE;

// WSAID_* extension-function GUIDs (DEFINE_GUID in mswsock.h; bindgen can't emit them).
pub const WSAID_ACCEPTEX: GUID = GUID { Data1: 0xb5367df1, Data2: 0xcbac, Data3: 0x11cf,
    Data4: [0x95, 0xca, 0x00, 0x80, 0x5f, 0x48, 0xa1, 0x92] };
pub const WSAID_GETACCEPTEXSOCKADDRS: GUID = GUID { Data1: 0xb5367df2, Data2: 0xcbac, Data3: 0x11cf,
    Data4: [0x95, 0xca, 0x00, 0x80, 0x5f, 0x48, 0xa1, 0x92] };
pub const WSAID_CONNECTEX: GUID = GUID { Data1: 0x25a207b9, Data2: 0xddf3, Data3: 0x4660,
    Data4: [0x8e, 0xe9, 0x76, 0xe5, 0x8c, 0x74, 0x06, 0x3e] };
pub const WSAID_DISCONNECTEX: GUID = GUID { Data1: 0x7fda2e11, Data2: 0x8630, Data3: 0x436f,
    Data4: [0xa0, 0x31, 0xf5, 0x36, 0xa6, 0xee, 0xc1, 0x57] };
// SO_DONTLINGER = ~SO_LINGER, SO_EXCLUSIVEADDRUSE = ~SO_REUSEADDR — bindgen skips these
// (their #define is a `~`-expression, not a literal). Values per winsock2.h.
pub const SO_DONTLINGER: u32 = !SO_LINGER;
pub const SO_EXCLUSIVEADDRUSE: u32 = !SO_REUSEADDR;
pub const IPPROTO_TCP: i32 = 6;
pub const IPPROTO_UDP: i32 = 17;
pub const IPPROTO_RAW: i32 = 255;

// --- NTSTATUS codes the ntdll-sys bindgen doesn't surface (AFD/TDI + misc) --------------
pub const STATUS_CANT_WAIT: NTSTATUS = 0xC00000D8u32 as NTSTATUS;
pub const STATUS_PROTOCOL_NOT_SUPPORTED: NTSTATUS = 0xC000A013u32 as NTSTATUS;
// NOTE: STATUS_RECEIVE_PARTIAL was MISSING — without this const, `match Status {
// STATUS_RECEIVE_PARTIAL => ... }` in sndrcv.rs was an irrefutable binding pattern
// (matches ANY value), so every successful recv() set *ReceiveFlags=MSG_PARTIAL and
// ws2_32 rejected it as WSAEMSGSIZE. (ntstatus.h: 0x4000000F.)
pub const STATUS_RECEIVE_PARTIAL: NTSTATUS = 0x4000000F;
pub const STATUS_RECEIVE_EXPEDITED: NTSTATUS = 0x40000010;
pub const STATUS_RECEIVE_PARTIAL_EXPEDITED: NTSTATUS = 0x40000011;
// More NTSTATUS values that were MISSING → their `match` arms in TranslateNtStatusError
// were irrefutable binding patterns (catch-all), so every unmatched status returned
// WSAECONNRESET and the intended per-status WSA mapping + default were dead code.
// (ntstatus.h values.)
pub const STATUS_FILE_CLOSED: NTSTATUS = 0xC0000128u32 as NTSTATUS;
pub const STATUS_INVALID_CONNECTION: NTSTATUS = 0xC0000140u32 as NTSTATUS;
pub const STATUS_ADDRESS_ALREADY_EXISTS: NTSTATUS = 0xC000020Au32 as NTSTATUS;
pub const STATUS_NETWORK_UNREACHABLE: NTSTATUS = 0xC000023Cu32 as NTSTATUS;
pub const STATUS_HOST_UNREACHABLE: NTSTATUS = 0xC000023Du32 as NTSTATUS;
// Winsock ioctlsocket codes (winsock2.h _IOW/_IOR expansions) — same missing-const
// binding-pattern bug in WSPIoctl's `match dwIoControlCode`.
pub const FIONBIO: DWORD = 0x8004667E;
pub const FIONREAD: DWORD = 0x4004667F;
pub const SIOCATMARK: DWORD = 0x40047307;

// --- AFD IOCTL codes (must match the Rust afd.sys receiver — validated round-trip) ------
// _AFD_CONTROL_CODE(Op,Method) = (FILE_DEVICE_NETWORK=0x12 << 12) | (Op<<2) | Method.
// METHOD_NEITHER=3, METHOD_BUFFERED=0. Numbers per sdk/include/reactos/drivers/afd/shared.h.
const fn afd_ctl(op: u32, method: u32) -> u32 { (0x12u32 << 12) | (op << 2) | method }
pub const IOCTL_AFD_BIND: u32 = afd_ctl(0, 3);
pub const IOCTL_AFD_CONNECT: u32 = afd_ctl(1, 3);
pub const IOCTL_AFD_START_LISTEN: u32 = afd_ctl(2, 3);
pub const IOCTL_AFD_WAIT_FOR_LISTEN: u32 = afd_ctl(3, 0);
pub const IOCTL_AFD_ACCEPT: u32 = afd_ctl(4, 0);
pub const IOCTL_AFD_RECV: u32 = afd_ctl(5, 3);
pub const IOCTL_AFD_RECV_DATAGRAM: u32 = afd_ctl(6, 3);
pub const IOCTL_AFD_SEND: u32 = afd_ctl(7, 3);
pub const IOCTL_AFD_SEND_DATAGRAM: u32 = afd_ctl(8, 3);
pub const IOCTL_AFD_SELECT: u32 = afd_ctl(9, 0);
pub const IOCTL_AFD_DISCONNECT: u32 = afd_ctl(10, 3);
pub const IOCTL_AFD_GET_SOCK_NAME: u32 = afd_ctl(11, 3);
pub const IOCTL_AFD_GET_PEER_NAME: u32 = afd_ctl(12, 3);
pub const IOCTL_AFD_GET_TDI_HANDLES: u32 = afd_ctl(13, 3);
pub const IOCTL_AFD_SET_INFO: u32 = afd_ctl(14, 3);
pub const IOCTL_AFD_GET_CONTEXT_SIZE: u32 = afd_ctl(15, 3);
pub const IOCTL_AFD_GET_CONTEXT: u32 = afd_ctl(16, 3);
pub const IOCTL_AFD_SET_CONTEXT: u32 = afd_ctl(17, 3);
pub const IOCTL_AFD_SET_CONNECT_DATA: u32 = afd_ctl(18, 3);
pub const IOCTL_AFD_SET_CONNECT_OPTIONS: u32 = afd_ctl(19, 3);
pub const IOCTL_AFD_SET_DISCONNECT_DATA: u32 = afd_ctl(20, 3);
pub const IOCTL_AFD_SET_DISCONNECT_OPTIONS: u32 = afd_ctl(21, 3);
pub const IOCTL_AFD_GET_CONNECT_DATA: u32 = afd_ctl(22, 3);
pub const IOCTL_AFD_GET_CONNECT_OPTIONS: u32 = afd_ctl(23, 3);
pub const IOCTL_AFD_GET_DISCONNECT_DATA: u32 = afd_ctl(24, 3);
pub const IOCTL_AFD_GET_DISCONNECT_OPTIONS: u32 = afd_ctl(25, 3);
pub const IOCTL_AFD_SET_CONNECT_DATA_SIZE: u32 = afd_ctl(26, 3);
pub const IOCTL_AFD_SET_CONNECT_OPTIONS_SIZE: u32 = afd_ctl(27, 3);
pub const IOCTL_AFD_SET_DISCONNECT_DATA_SIZE: u32 = afd_ctl(28, 3);
pub const IOCTL_AFD_SET_DISCONNECT_OPTIONS_SIZE: u32 = afd_ctl(29, 3);
pub const IOCTL_AFD_GET_INFO: u32 = afd_ctl(30, 3);
pub const IOCTL_AFD_EVENT_SELECT: u32 = afd_ctl(33, 3);
pub const IOCTL_AFD_ENUM_NETWORK_EVENTS: u32 = afd_ctl(34, 3);
pub const IOCTL_AFD_DEFER_ACCEPT: u32 = afd_ctl(35, 3);
pub const IOCTL_AFD_GET_PENDING_CONNECT_DATA: u32 = afd_ctl(41, 3);
pub const IOCTL_AFD_VALIDATE_GROUP: u32 = afd_ctl(42, 3);
pub const IOCTL_AFD_SUPER_CONNECT: u32 = afd_ctl(49, 3);
pub const IOCTL_AFD_SUPER_ACCEPT: u32 = afd_ctl(32, 3);

/// shared.h `AFD_SUPER_ACCEPT_INFO` (upstream a68bfa2fb41, newer than the bindgen islands).
#[repr(C)]
#[derive(Copy, Clone)]
pub struct AFD_SUPER_ACCEPT_INFO {
    pub SanActive: BOOLEAN,
    pub FixAddressAlignment: BOOLEAN,
    pub AcceptHandle: HANDLE,
    pub ReceiveDataLength: ULONG,
    pub LocalAddressLength: ULONG,
    pub RemoteAddressLength: ULONG,
}

// --- Win32 value-macros (LOBYTE/HIBYTE/MAKEWORD/MAKELONG/WSAMAKESELECTREPLY/IsEqualGUID) -
// The C source uses these as call expressions, so they're inline fns (not macro_rules).
#[inline(always)] pub fn LOBYTE(w: usize) -> BYTE { (w & 0xff) as BYTE }
#[inline(always)] pub fn HIBYTE(w: usize) -> BYTE { ((w >> 8) & 0xff) as BYTE }
#[inline(always)] pub fn MAKEWORD(low: usize, high: usize) -> WORD {
    ((low & 0xff) | ((high & 0xff) << 8)) as WORD
}
#[inline(always)] pub fn MAKELONG(low: u32, high: u32) -> LONG {
    ((low & 0xffff) | ((high & 0xffff) << 16)) as LONG
}
#[inline(always)] pub fn WSAMAKESELECTREPLY(event: u32, error: u32) -> LONG { MAKELONG(event, error) }
// IsEqualGUID(a,b) = !memcmp(a, b, sizeof(GUID)) — GUID is 16 bytes. msafd callers pass
// *const GUID (the extension-fn GUID table), so take that directly.
#[inline(always)] pub unsafe fn IsEqualGUID(a: *const GUID, b: *const GUID) -> bool {
    core::slice::from_raw_parts(a as *const u8, 16) == core::slice::from_raw_parts(b as *const u8, 16)
}

// --- pointer/handle cast macros (HandleToUlong/UlongToPtr/UlongToHandle/IS_INTRESOURCE) --
// C inline macros in basetsd.h/winuser.h — NOT DLL exports, so they must be inline here
// (declaring them extern would produce undefined symbols at link).
#[inline(always)] pub fn HandleToUlong(h: HANDLE) -> ULONG { (h as usize) as ULONG }
#[inline(always)] pub fn UlongToPtr(ul: ULONG) -> PVOID { ul as usize as PVOID }
#[inline(always)] pub fn UlongToHandle(ul: ULONG) -> HANDLE { ul as usize as HANDLE }
#[inline(always)] pub fn IS_INTRESOURCE(ptr: LPVOID) -> BOOL { ((ptr as usize) >> 16 == 0) as BOOL }

// --- fd_set macros (FD_ZERO / FD_SET) — operate on the winsock fd_set struct ------------
#[inline(always)] pub unsafe fn FD_ZERO(set: *mut fd_set) { (*set).fd_count = 0; }
#[inline(always)] pub unsafe fn FD_SET(fd: SOCKET, set: *mut fd_set) {
    let s = &mut *set;
    let n = s.fd_count as usize;
    if n < s.fd_array.len() { s.fd_array[n] = fd; s.fd_count += 1; }
}

// --- Win32/CRT/Rtl APIs referenced across modules (linked from kernel32/ntdll at DLL link) -
extern "system" {
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: usize) -> PVOID;
    pub fn GetProcessHeap() -> HANDLE;
    pub fn SetLastError(dwErrCode: DWORD);
    pub fn SleepEx(dwMilliseconds: DWORD, bAlertable: BOOL) -> DWORD;
    pub fn GetTickCount() -> DWORD;
    pub fn GetOverlappedResult(hFile: HANDLE, lpOverlapped: LPOVERLAPPED,
        lpNumberOfBytesTransferred: LPDWORD, bWait: BOOL) -> BOOL;
}
// RtlConvertLongToLargeInteger / RtlEnlargedIntegerMultiply are ntdll exports ONLY on i386
// (64-bit math helpers); on amd64/arm64 they're inlined (native 64-bit). Implement inline for
// all arches (identical result). LARGE_INTEGER is a repr(C) 8-byte union == i64 layout.
#[inline(always)] pub fn RtlConvertLongToLargeInteger(l: LONG) -> LARGE_INTEGER {
    unsafe { core::mem::transmute::<i64, LARGE_INTEGER>(l as i64) }
}
#[inline(always)] pub fn RtlEnlargedIntegerMultiply(a: LONG, b: LONG) -> LARGE_INTEGER {
    unsafe { core::mem::transmute::<i64, LARGE_INTEGER>((a as i64) * (b as i64)) }
}
// RtlCopyMemory/RtlZeroMemory/memset are NT header macros (memmove/memset); model as inline.
#[inline(always)] pub unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, len: usize) {
    core::ptr::copy(src as *const u8, dst as *mut u8, len);
}
#[inline(always)] pub unsafe fn RtlZeroMemory(dst: *mut c_void, len: usize) {
    core::ptr::write_bytes(dst as *mut u8, 0, len);
}
#[inline(always)] pub unsafe fn memset(s: *mut c_void, c: i32, n: usize) -> *mut c_void {
    core::ptr::write_bytes(s as *mut u8, c as u8, n); s
}
// kernel32 OutputDebugStringA — used by the #[panic_handler] in lib.rs to emit a
// "[MSAFD] RUST PANIC" breadcrumb to the debug port before the loop{} spin.
extern "system" { pub fn OutputDebugStringA(lpOutputString: *const i8); }

// as_apc — reinterpret an `extern "system"` APC fn as ntdll-sys's PIO_APC_ROUTINE. bindgen
// spells that type `extern "stdcall"`(i386)/`extern "C"`(x64/arm64); the msafd APC routines
// are defined `extern "system"` (portable). The two are ABI-IDENTICAL per arch (system ==
// stdcall on i386, == C on x64/arm64, exactly what PIO_APC_ROUTINE is), so transmuting the
// fn pointer is a provably-safe no-op bit reinterpret — it only bridges the Rust type-spelling
// mismatch Rust won't unify. Used at the NtDeviceIoControlFile ApcRoutine assignment sites.
#[inline(always)] pub unsafe fn as_apc(
    f: unsafe extern "system" fn(ApcContext: PVOID, IoStatusBlock: PIO_STATUS_BLOCK, Reserved: ULONG),
) -> PIO_APC_ROUTINE {
    core::mem::transmute(f)
}

// InterlockedIncrement/Decrement — atomic ±1 on a LONG (compiler intrinsic in C).
#[inline(always)] pub unsafe fn InterlockedIncrement(addend: *mut LONG) -> LONG {
    (*(addend as *const core::sync::atomic::AtomicI32)).fetch_add(1, core::sync::atomic::Ordering::SeqCst) + 1
}
#[inline(always)] pub unsafe fn InterlockedDecrement(addend: *mut LONG) -> LONG {
    (*(addend as *const core::sync::atomic::AtomicI32)).fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1
}

// --- shared msafd module globals (defined `pub static mut` in misc::dllmain) -------------
// Re-exported here so every module that does `use crate::glue::*` sees them by bare name
// (the C had them as translation-unit globals shared via extern decls in msafd.h).
pub use crate::misc::dllmain::{
    GlobalHeap, Upcalls, SockHelpersListHead, SockAsyncCompletionPort,
    SockAsyncSelectCalled, lpWPUCompleteOverlappedRequest,
};

// --- shared msafd internal helpers (defined `#[no_mangle] pub` in misc::dllmain) ---------
// Re-exported so callers in every module see them by bare name via `use crate::glue::*`
// (the C shared them through msafd.h prototypes; MsafdReturnWithErrno was a header inline).
pub use crate::misc::dllmain::{
    MsafdReturnWithErrno, TranslateNtStatusError, GetSocketStructure, MsafdWaitForAlert, MsafdWaitForIo,
    SetSocketInformation, SockCreateOrReferenceAsyncThread, SockGetAsyncSelectHelperAfdHandle,
    SockProcessQueuedAsyncSelect,
};

// --- WSP* dispatch entrypoints (defined `#[no_mangle] pub` across the misc modules) ------
// WSPStartup fills the WSPPROC_TABLE with pointers to these; the referencing module (dllmain)
// needs every WSP* in scope by bare name. Re-export them all here so `use crate::glue::*`
// resolves the cross-module fn-pointer references.
pub use crate::misc::dllmain::{
    WSPAccept, WSPAddressToString, WSPBind, WSPCleanup, WSPCloseSocket, WSPConnect, WSPConnectEx,
    WSPDuplicateSocket, WSPGetPeerName, WSPGetSockName, WSPGetSockOpt, WSPSetSockOpt, WSPIoctl,
    WSPListen, WSPSelect, WSPShutdown, WSPSocket, WSPStartup, WSPStringToAddress,
};
pub use crate::misc::event::{WSPEnumNetworkEvents, WSPEventSelect};
pub use crate::misc::sndrcv::{
    WSPAsyncSelect, WSPGetOverlappedResult, WSPRecv, WSPRecvDisconnect, WSPRecvFrom, WSPSend,
    WSPSendDisconnect, WSPSendTo,
};
pub use crate::misc::stubs::{
    WSPCancelBlockingCall, WSPDisconnectEx, WSPGetQOSByName, WSPJoinLeaf,
};
