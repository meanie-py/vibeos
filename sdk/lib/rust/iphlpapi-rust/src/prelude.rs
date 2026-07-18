//! iphlpapi-rust prelude — the single import surface the translated module uses.
//!
//! Bridges the gaps between the bindgen `types` island and the c2rust-expanded translation:
//!   1. the wine debug channel (wine/debug.h) — c2rust expands TRACE/ERR/FIXME to the raw
//!      `if dbch.flags & (1<<__WINE_DBCL_x) { ros_dbg_log(...) }` form; shimmed here (the
//!      lsasrv/netapi32 pattern) so it resolves to a release-inert channel + the real variadic
//!      ros_dbg_log at link.
//!   2. CRT (cdecl) + winsock (stdcall) + kernel32/advapi32 (stdcall) functions the translation
//!      calls but never declared. NEVER hand-write memcpy/memset (LLVM folds to self-call).
//!   3. cast-#define consts / typedefs bindgen skipped — transcribed with a citation.
//!   4. bridges added as the cargo-check grind reveals them (enum aliases, tail consts).
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_uchar, c_uint, c_ulong, c_ushort, c_void};

// ---- 1. wine debug internals (wine/debug.h) — the lsasrv/netapi32 pattern. ros_dbg_log is a
// real ReactOS export (variadic); declare it and let it resolve at link. NOT bindgen-emitted
// (wine/debug.h is parsed but not allowlisted) so there is no duplicate. ----
pub type __wine_debug_class = c_int;
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0;
pub const __WINE_DBCL_ERR: __wine_debug_class = 1;
pub const __WINE_DBCL_WARN: __wine_debug_class = 2;
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3;
#[repr(C)]
pub struct __wine_debug_channel {
    pub flags: c_uchar,
    pub name: [c_char; 15],
}
pub const WINE_DBG_DEFAULT_FLAGS: c_uchar = 3;
extern "C" {
    pub fn ros_dbg_log(
        cls: __wine_debug_class,
        ch: *mut __wine_debug_channel,
        file: *const c_char,
        func: *const c_char,
        line: c_int,
        msg: *const c_char,
        ...
    ) -> c_int;
}
#[inline]
pub fn debugstr_w(_s: *const u16) -> *const i8 {
    core::ptr::null()
}
#[inline]
pub fn debugstr_a(_s: *const i8) -> *const i8 {
    core::ptr::null()
}

// ---- 2a. CRT (cdecl, extern "C") — exported by msvcrt/ntdll, which iphlpapi.dll links. ----
extern "C" {
    pub fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
    pub fn strncmp(a: *const c_char, b: *const c_char, n: usize) -> c_int;
    pub fn strtoul(s: *const c_char, endptr: *mut *mut c_char, base: c_int) -> c_ulong;
    pub fn atoi(s: *const c_char) -> c_int;
    pub fn free(ptr: *mut c_void);
    pub fn malloc(size: usize) -> *mut c_void;
    pub fn sprintf(buf: *mut c_char, fmt: *const c_char, ...) -> c_int;
    pub fn wcslen(s: *const u16) -> usize;
    pub fn qsort(
        base: *mut c_void,
        num: usize,
        size: usize,
        cmp: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>,
    );
}

// ---- 2b. winsock byte-order (stdcall, extern "system" → `_Name@N` on i386) — from ws2_32. ----
extern "system" {
    pub fn htons(hostshort: u16) -> u16;
    pub fn htonl(hostlong: u32) -> u32;
    pub fn ntohs(netshort: u16) -> u16;
    pub fn ntohl(netlong: u32) -> u32;
}

// ---- 2c. kernel32 (Win32 = extern "system") — heap / handles / TLS. -------------------------
extern "system" {
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    pub fn HeapReAlloc(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID, dwBytes: SIZE_T) -> LPVOID;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    pub fn GetCurrentProcess() -> HANDLE;
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn GetLastError() -> DWORD;
    pub fn SetLastError(dwErrCode: DWORD);
    pub fn DisableThreadLibraryCalls(hLibModule: HMODULE) -> BOOL;
    pub fn LoadLibraryW(lpLibFileName: LPCWSTR) -> HMODULE;
    pub fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> *mut c_void;
    pub fn FreeLibrary(hLibModule: HMODULE) -> BOOL;
}

// ---- 2d. advapi32 registry (Win32 = extern "system") — from winreg.h. -----------------------
extern "system" {
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS;
    pub fn RegOpenKeyExW(
        hKey: HKEY,
        lpSubKey: LPCWSTR,
        ulOptions: DWORD,
        samDesired: REGSAM,
        phkResult: PHKEY,
    ) -> LSTATUS;
    pub fn RegQueryValueExW(
        hKey: HKEY,
        lpValueName: LPCWSTR,
        lpReserved: LPDWORD,
        lpType: LPDWORD,
        lpData: LPBYTE,
        lpcbData: LPDWORD,
    ) -> LSTATUS;
}

// ---- 3. common consts / typedefs bindgen skipped — each header-cited. -----------------------
pub const NULL: PVOID = core::ptr::null_mut();
pub const INADDR_NONE: u32 = 0xFFFF_FFFF; // winsock.h — `#define INADDR_NONE 0xffffffff`
pub const INADDR_ANY: u32 = 0; // winsock.h
pub const INADDR_LOOPBACK: u32 = 0x7f00_0001; // winsock.h — 127.0.0.1
pub const IPPROTO_TCP: c_int = 6; // winsock.h
pub const IPPROTO_UDP: c_int = 17; // winsock.h
pub const ETH_ALEN: usize = 6; // iphlpapi_private.h — `#define ETH_ALEN 6`

#[inline]
pub fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

// TRUE/FALSE are bindgen u32, but BOOL is c_int (i32). Shadow as BOOL so `let x: BOOL = FALSE`
// and `x == FALSE` sites type-check (explicit item beats the types::* glob).
pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;

// ---- 4. Rtl*/Copy*/Zero* value-macros the C used (never hand-write memcpy — delegate). ------
#[inline]
pub unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, len: usize) {
    memcpy(dst, src, len);
}
#[inline]
pub unsafe fn CopyMemory(dst: *mut c_void, src: *const c_void, len: usize) {
    memcpy(dst, src, len);
}
#[inline]
pub unsafe fn RtlZeroMemory(dst: *mut c_void, len: usize) {
    memset(dst, 0, len);
}
#[inline]
pub unsafe fn ZeroMemory(dst: *mut c_void, len: usize) {
    memset(dst, 0, len);
}

// ---- 1-bis. wine TRACE/FIXME/ERR/WARN — the DeepSeek output emits BOTH the inline-expanded
// `if (*__wine_dbch___default).flags & (1<<DBCL) { ros_dbg_log(...) }` form AND bare `TRACE(fmt,
// ...)` / `FIXME(fmt)` calls. Declare the bare forms as extern "C" variadic (declaring a
// C-variadic is stable; only DEFINING needs a feature) — a no-op C shim (iphlpapi_rust_shims.c,
// wired in the cohab-flip) provides the linkable symbols, keeping the debug path release-inert.
// `__wine_dbch___default` is the default channel pointer the inline form dereferences; the shim
// also provides it, pointing at a flags=0 channel so the `if` is always false. ----
extern "C" {
    pub fn TRACE(fmt: *const u8, ...);
    pub fn FIXME(fmt: *const u8, ...);
    pub fn ERR(fmt: *const u8, ...);
    pub fn WARN(fmt: *const u8, ...);
    pub static mut __wine_dbch___default: *mut __wine_debug_channel;
}

// ---- 5. bridges added as the cargo-check grind revealed them. ----

// 5a. PULONG (bindgen didn't emit it) + a couple of _-prefixed struct aliases the module names
// under the public C typedef.
pub type PULONG = *mut ULONG;
pub type MIB_IFTABLE = _MIB_IFTABLE; // bindgen emitted only the _-prefixed struct

// 5b. FIELD_OFFSET(type, field) → byte offset as usize (call sites do `.wrapping_add(usize)`).
// winnt.h `#define FIELD_OFFSET(t,f) ((LONG)(LONG_PTR)&(((t*)0)->f))`.
macro_rules! FIELD_OFFSET {
    ($t:ty, $f:ident) => {
        core::mem::offset_of!($t, $f)
    };
}
pub(crate) use FIELD_OFFSET;

// 5c. FAILED(hr) — winerror.h `#define FAILED(hr) (((HRESULT)(hr)) < 0)`.
#[inline]
pub fn FAILED(hr: HRESULT) -> bool {
    (hr as i32) < 0
}

// 5e. enum members — bindgen names them <EnumTag>_<Member>; re-export under the bare C member.
pub use crate::types::{
    IF_OPER_STATUS_IfOperStatusDown as IfOperStatusDown,
    IF_OPER_STATUS_IfOperStatusUp as IfOperStatusUp,
    MIB_IPFORWARD_TYPE_MIB_IPROUTE_TYPE_DIRECT as MIB_IPROUTE_TYPE_DIRECT,
    MIB_TCP_STATE_MIB_TCP_STATE_LISTEN as MIB_TCP_STATE_LISTEN,
    NL_DAD_STATE_IpDadStatePreferred as IpDadStatePreferred,
    NL_PREFIX_ORIGIN_IpPrefixOriginOther as IpPrefixOriginOther,
    NL_ROUTE_PROTOCOL_MIB_IPPROTO_LOCAL as MIB_IPPROTO_LOCAL,
    _CLASS_TABLE_ClassBasic as ClassBasic, _CLASS_TABLE_ClassModule as ClassModule,
    _CLASS_TABLE_ClassModulePid as ClassModulePid,
    _INTERNAL_IF_OPER_STATUS_IF_OPER_STATUS_CONNECTING as IF_OPER_STATUS_CONNECTING,
    _TCP_TABLE_CLASS_TCP_TABLE_BASIC_ALL as TCP_TABLE_BASIC_ALL,
    _TCP_TABLE_CLASS_TCP_TABLE_BASIC_CONNECTIONS as TCP_TABLE_BASIC_CONNECTIONS,
    _TCP_TABLE_CLASS_TCP_TABLE_BASIC_LISTENER as TCP_TABLE_BASIC_LISTENER,
    _TCP_TABLE_CLASS_TCP_TABLE_OWNER_MODULE_ALL as TCP_TABLE_OWNER_MODULE_ALL,
    _TCP_TABLE_CLASS_TCP_TABLE_OWNER_MODULE_CONNECTIONS as TCP_TABLE_OWNER_MODULE_CONNECTIONS,
    _TCP_TABLE_CLASS_TCP_TABLE_OWNER_MODULE_LISTENER as TCP_TABLE_OWNER_MODULE_LISTENER,
    _TCP_TABLE_CLASS_TCP_TABLE_OWNER_PID_ALL as TCP_TABLE_OWNER_PID_ALL,
    _TCP_TABLE_CLASS_TCP_TABLE_OWNER_PID_CONNECTIONS as TCP_TABLE_OWNER_PID_CONNECTIONS,
    _TCP_TABLE_CLASS_TCP_TABLE_OWNER_PID_LISTENER as TCP_TABLE_OWNER_PID_LISTENER,
    _UDP_TABLE_CLASS_UDP_TABLE_BASIC as UDP_TABLE_BASIC,
    _UDP_TABLE_CLASS_UDP_TABLE_OWNER_MODULE as UDP_TABLE_OWNER_MODULE,
    _UDP_TABLE_CLASS_UDP_TABLE_OWNER_PID as UDP_TABLE_OWNER_PID,
};

// 5f. consts bindgen skipped — each header-cited.
pub const UNICODE_NULL: WCHAR = 0; // ntdef.h — UNICODE_NULL = L'\0'
pub const CP_ACP: UINT = 0; // winnls.h — ANSI code page
pub const PF_INET: c_int = 2; // winsock2.h — PF_INET/AF_INET
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as HKEY; // winreg.h
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as HKEY; // winreg.h
pub const INVALID_HANDLE_VALUE: HANDLE = usize::MAX as HANDLE; // handleapi.h — (HANDLE)-1
// tcpioctl.h — CTL_CODE(FILE_DEVICE_NETWORK=0x12, Function, Method, Access):
//   ((0x12<<16) | (Access<<14) | (Function<<2) | Method)
pub const IOCTL_TCP_QUERY_INFORMATION_EX: DWORD = 0x0012_0003; // fn 0, METHOD_NEITHER,  ANY
pub const IOCTL_TCP_SET_INFORMATION_EX: DWORD = 0x0012_8004; //   fn 1, METHOD_BUFFERED, WRITE
pub const IOCTL_QUERY_IP_HW_ADDRESS: DWORD = 0x0012_003C; //      fn 15, METHOD_BUFFERED, ANY

// 5g. extra CRT (cdecl) the translation calls but the base block above didn't list.
extern "C" {
    pub fn wcscpy(dest: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR;
    pub fn strncpy(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    // strsafe StringCbPrintfW is WINAPIV(__cdecl) varargs — DECLARING a C-variadic is stable;
    // the no-op-free real symbol comes from iphlpapi_rust_shims.c (via _vsnwprintf) at link.
    pub fn StringCbPrintfW(pszDest: *mut WCHAR, cbDest: usize, pszFormat: *const WCHAR, ...)
        -> HRESULT;
}

// _BitScanForward is an MSVC INTRINSIC — no linkable symbol exists (the C compiler expands it
// inline). Implement it as a Rust inline fn: sets *Index to the bit index of the lowest set bit,
// returns 0 if Mask==0. (winnt.h intrinsic contract.)
#[inline]
pub unsafe fn _BitScanForward(Index: *mut c_ulong, Mask: c_ulong) -> c_uchar {
    if Mask == 0 {
        0
    } else {
        *Index = Mask.trailing_zeros() as c_ulong;
        1
    }
}
// StringCchCatW is a strsafe.h `static inline` (NO linkable symbol). Implement the bounded wide
// strcat inline: append pszSrc to pszDest; total (incl NUL) must fit cchDest chars, else truncate
// + STRSAFE_E_INSUFFICIENT_BUFFER. (strsafe.h contract.)
pub const S_OK: HRESULT = 0;
pub const STRSAFE_E_INSUFFICIENT_BUFFER: HRESULT = 0x8007007Au32 as i32 as HRESULT;
#[inline]
pub unsafe fn StringCchCatW(pszDest: *mut WCHAR, cchDest: usize, pszSrc: *const WCHAR) -> HRESULT {
    let mut dlen = 0usize;
    while dlen < cchDest && *pszDest.add(dlen) != 0 {
        dlen += 1;
    }
    if dlen >= cchDest {
        return STRSAFE_E_INSUFFICIENT_BUFFER;
    }
    let mut i = 0usize;
    while *pszSrc.add(i) != 0 {
        if dlen + i + 1 >= cchDest {
            *pszDest.add(cchDest - 1) = 0;
            return STRSAFE_E_INSUFFICIENT_BUFFER;
        }
        *pszDest.add(dlen + i) = *pszSrc.add(i);
        i += 1;
    }
    *pszDest.add(dlen + i) = 0;
    S_OK
}

// 5h. extra winsock (stdcall on i386).
extern "system" {
    pub fn inet_addr(cp: *const c_char) -> u32;
}

// 5i. extra Win32 (stdcall on i386) the translation calls.
extern "system" {
    pub fn DeviceIoControl(
        hDevice: HANDLE,
        dwIoControlCode: DWORD,
        lpInBuffer: LPVOID,
        nInBufferSize: DWORD,
        lpOutBuffer: LPVOID,
        nOutBufferSize: DWORD,
        lpBytesReturned: LPDWORD,
        lpOverlapped: LPOVERLAPPED,
    ) -> BOOL;
    pub fn GetSystemDirectoryW(lpBuffer: LPWSTR, uSize: UINT) -> UINT;
    pub fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HMODULE;
    pub fn LocalFree(hMem: HLOCAL) -> HLOCAL;
    pub fn IsBadWritePtr(lp: LPVOID, ucb: usize) -> BOOL;
    pub fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: LPCSTR,
        cbMultiByte: c_int,
        lpWideCharStr: LPWSTR,
        cchWideChar: c_int,
    ) -> c_int;
    pub fn WideCharToMultiByte(
        CodePage: UINT,
        dwFlags: DWORD,
        lpWideCharStr: LPCWSTR,
        cchWideChar: c_int,
        lpMultiByteStr: LPSTR,
        cbMultiByte: c_int,
        lpDefaultChar: LPCSTR,
        lpUsedDefaultChar: LPBOOL,
    ) -> c_int;
    pub fn RegOpenKeyA(hKey: HKEY, lpSubKey: LPCSTR, phkResult: PHKEY) -> LSTATUS;
    pub fn RegOpenKeyExA(
        hKey: HKEY,
        lpSubKey: LPCSTR,
        ulOptions: DWORD,
        samDesired: REGSAM,
        phkResult: PHKEY,
    ) -> LSTATUS;
    pub fn RegQueryValueExA(
        hKey: HKEY,
        lpValueName: LPCSTR,
        lpReserved: LPDWORD,
        lpType: LPDWORD,
        lpData: LPBYTE,
        lpcbData: LPDWORD,
    ) -> LSTATUS;
    pub fn OpenProcess(dwDesiredAccess: DWORD, bInheritHandle: BOOL, dwProcessId: DWORD) -> HANDLE;
    pub fn GetModuleFileNameExW(
        hProcess: HANDLE,
        hModule: HMODULE,
        lpFilename: LPWSTR,
        nSize: DWORD,
    ) -> DWORD;
    pub fn GetModuleBaseNameW(
        hProcess: HANDLE,
        hModule: HMODULE,
        lpBaseName: LPWSTR,
        nSize: DWORD,
    ) -> DWORD;
    // dhcpcsvc (stdcall).
    pub fn DhcpCApiInitialize(Version: *mut DWORD) -> DWORD;
    pub fn DhcpCApiCleanup();
    pub fn DhcpAcquireParameters(Name: PWSTR) -> DWORD;
    pub fn DhcpReleaseParameters(AdapterName: PWSTR) -> DWORD;
}

// 5j. iphlpapi sibling-C helpers (dll/win32/iphlpapi/*.c + sdk/lib/tdilib). These are
// ReactOS-internal — no WINAPI in the headers, so cdecl (extern "C"). Signatures taken from
// ifenum.h / ipstats.h / route.h / resinfo.h / dhcp.h / iphlpapi_private.h / tdilib.h.
extern "C" {
    // ifenum.h
    pub fn interfaceMapInit();
    pub fn interfaceMapFree();
    pub fn getNumInterfaces() -> DWORD;
    pub fn getNumNonLoopbackInterfaces() -> DWORD;
    pub fn getInterfaceIndexTable() -> *mut InterfaceIndexTable;
    pub fn getNonLoopbackInterfaceIndexTable() -> *mut InterfaceIndexTable;
    pub fn getInterfaceNameByIndex(index: DWORD) -> *const c_char;
    pub fn consumeInterfaceName(ifname: *const c_char);
    pub fn getInterfaceIPAddrByIndex(index: DWORD) -> DWORD;
    pub fn getInterfaceMaskByIndex(index: DWORD) -> DWORD;
    pub fn getInterfaceBCastAddrByIndex(index: DWORD) -> DWORD;
    pub fn getInterfacePhysicalByIndex(
        index: DWORD,
        len: *mut DWORD,
        addr: *mut BYTE,
        ty: *mut DWORD,
    ) -> DWORD;
    pub fn getInterfaceEntryByIndex(index: DWORD, entry: PMIB_IFROW) -> DWORD;
    pub fn toIPAddressString(addr: c_uint, string: *mut c_char) -> *mut c_char;
    pub fn addIPAddress(
        Address: IPAddr,
        Mask: IPMask,
        IfIndex: DWORD,
        NteContext: PULONG,
        NteInstance: PULONG,
    ) -> NTSTATUS;
    pub fn deleteIpAddress(NteContext: ULONG) -> NTSTATUS;
    // ipstats.h
    pub fn getInterfaceStatsByName(name: *const c_char, entry: PMIB_IFROW) -> DWORD;
    pub fn getICMPStats(stats: PMIB_ICMP) -> DWORD;
    pub fn getIPStats(tcpFile: HANDLE, stats: PMIB_IPSTATS) -> DWORD;
    pub fn getTCPStats(tcpFile: HANDLE, stats: *mut MIB_TCPSTATS) -> DWORD;
    pub fn getUDPStats(tcpFile: HANDLE, stats: *mut MIB_UDPSTATS) -> DWORD;
    pub fn getNumRoutes() -> DWORD;
    pub fn getRouteTable() -> *mut RouteTable;
    pub fn getNumArpEntries() -> DWORD;
    pub fn getArpTable() -> PMIB_IPNETTABLE;
    pub fn getTcpTable(class: DWORD) -> LPVOID;
    pub fn getUdpTable(class: c_int) -> LPVOID;
    // route.h
    pub fn createIpForwardEntry(route: PMIB_IPFORWARDROW) -> DWORD;
    pub fn setIpForwardEntry(route: PMIB_IPFORWARDROW) -> DWORD;
    pub fn deleteIpForwardEntry(route: PMIB_IPFORWARDROW) -> DWORD;
    // resinfo.h
    pub fn getResInfo() -> PIPHLP_RES_INFO;
    pub fn disposeResInfo(info: PIPHLP_RES_INFO);
    // dhcp.h
    pub fn getDhcpInfoForAdapter(index: DWORD, ptr: PIP_ADAPTER_INFO) -> DWORD;
    // iphlpapi_private.h
    pub fn getNthIpEntity(tcpFile: HANDLE, index: DWORD, ent: *mut TDIEntityID) -> NTSTATUS;
    pub fn getIPAddrEntryForIf(
        tcpFile: HANDLE,
        name: *mut c_char,
        index: DWORD,
        ifInfo: *mut IFInfo,
    ) -> NTSTATUS;
    pub fn TCPSendIoctl(
        hDevice: HANDLE,
        dwIoControlCode: DWORD,
        lpInBuffer: LPVOID,
        pInBufferSize: PULONG,
        lpOutBuffer: LPVOID,
        pOutBufferSize: PULONG,
    ) -> DWORD;
    pub fn EnumNameServers(
        RegHandle: HKEY,
        Interface: LPWSTR,
        Data: PVOID,
        cb: Option<unsafe extern "C" fn(*mut WCHAR, *mut WCHAR, PVOID)>,
    ) -> LSTATUS;
    // sdk/lib/tdilib
    pub fn openTcpFile(tcpFile: PHANDLE, access: ACCESS_MASK) -> NTSTATUS;
    pub fn closeTcpFile(tcpFile: HANDLE);
}

// 5k. iphlpapi-private aggregate types bindgen didn't emit (dll/win32/iphlpapi/iphlpapi_private.h).
// The C originals are unions sized by a CHAR MaxSize[...] alternative; the bindgen struct members
// (IFEntry/TCP_REQUEST_SET_INFORMATION_EX) are not `Copy`, so a real Rust `union` is illegal here.
// Modelled instead as a struct: the used arm first, then trailing bytes so the whole object is at
// least as large as the C union — the sibling-C fills it in place and only the used arm is read.
pub const MAX_ADAPTER_DESCRIPTION_LENGTH: usize = 128; // wine/iptypes.h

#[repr(C)]
pub struct IFEntrySafelySized {
    pub ent: IFEntry,
    // C: CHAR MaxSize[sizeof(DWORD) + sizeof(IFEntry) + MAX_ADAPTER_DESCRIPTION_LENGTH + 1]
    _tail: [u8; core::mem::size_of::<DWORD>() + MAX_ADAPTER_DESCRIPTION_LENGTH + 1],
}

#[repr(C)]
pub struct IFInfo {
    pub entity_id: TDIEntityID,
    pub if_info: IFEntrySafelySized,
    pub ip_addr: IPAddrEntry,
}

#[repr(C)]
pub struct TCP_REQUEST_SET_INFORMATION_EX_ARP_ENTRY {
    pub Req: TCP_REQUEST_SET_INFORMATION_EX,
    // C: CHAR MaxSize[sizeof(TCP_REQUEST_SET_INFORMATION_EX) - 1 + sizeof(MIB_IPNETROW)]
    _tail: [u8; core::mem::size_of::<MIB_IPNETROW>()],
}
