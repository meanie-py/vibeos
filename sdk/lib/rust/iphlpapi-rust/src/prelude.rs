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

// The wine default debug channel, defined HERE in Rust rather than imported from a C shim.
// iphlpapi_main.rs carries 10 c2rust-expanded sites of the inline form
// `if (*__wine_dbch___default).flags & (1 << CLASS) { ros_dbg_log(...) }`; flags = 0 makes
// every one of them statically false, which is exactly the non-DBG C build. Defining it in
// Rust is what let iphlpapi_rust_shims.c be deleted (see the TRACE macros below).
static mut __WINE_DBCH_DEFAULT_STORAGE: __wine_debug_channel = __wine_debug_channel {
    flags: 0,
    name: [
        b'i' as c_char, b'p' as c_char, b'h' as c_char, b'l' as c_char, b'p' as c_char,
        b'a' as c_char, b'p' as c_char, b'i' as c_char, 0, 0, 0, 0, 0, 0, 0,
    ],
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __WINE_DBCH_DEFAULT_STORAGE };

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
    // `const char *` in the C shim -> *const c_char (i8), matching the `as *const i8` the
    // translation casts every format literal to.
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
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY; // winreg.h
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as i32 as isize as HKEY; // winreg.h
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
    // _snwprintf: a REAL msvcrt export and variadic. Declaring and CALLING a C-variadic is
    // stable Rust; only DEFINING one is not — which is the whole reason StringCbPrintfW gets
    // the fixed-arity treatment just below.
    pub fn _snwprintf(buffer: *mut WCHAR, count: usize, format: *const WCHAR, ...) -> c_int;
}

/// `StringCbPrintfW` — strsafe.h ships it ONLY as a `static inline`, so no linkable symbol
/// exists anywhere, and it is variadic (`WINAPIV` = __cdecl), which stable Rust cannot DEFINE.
/// It used to be supplied by a tiny C shim (iphlpapi_rust_shims.c); that file is gone now — it
/// also carried out-of-line no-op TRACE/FIXME/ERR/WARN, which is the field-manual §9.65 ud2
/// hazard, so keeping it alive purely for this one function was the wrong trade.
///
/// iphlpapi has exactly ONE call site (iphlpapi_main.c:3279) and it passes exactly ONE
/// variadic argument (a `%s` WCHAR pointer), so a fixed-arity Rust fn reproduces it exactly
/// with no variadic definition and no C. Semantics transcribed from the shim it replaces,
/// which in turn follows strsafe: cbDest is a BYTE count, the result is ALWAYS
/// null-terminated, and truncation is an error rather than a silent short write.
#[inline]
pub unsafe fn StringCbPrintfW_1(
    pszDest: *mut WCHAR,
    cbDest: usize,
    pszFormat: *const WCHAR,
    arg1: *const WCHAR,
) -> HRESULT {
    let cch_dest = cbDest / core::mem::size_of::<WCHAR>();
    if cch_dest == 0 {
        return 0x8007_0057u32 as HRESULT; // STRSAFE_E_INVALID_PARAMETER
    }
    let r = _snwprintf(pszDest, cch_dest, pszFormat, arg1);
    if r < 0 || (r as usize) >= cch_dest {
        *pszDest.add(cch_dest - 1) = 0; // strsafe guarantees termination even on overflow
        return 0x8007_007Au32 as HRESULT; // STRSAFE_E_INSUFFICIENT_BUFFER
    }
    0 // S_OK
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

// 5j. tdilib (sdk/lib/tdilib) — the ONLY sibling-C left after the FULL flip. ReactOS-internal,
// no WINAPI in tdilib.h, so cdecl (extern "C"). Signatures verbatim from sdk/lib/tdilib/tdilib.h.
extern "C" {
    pub fn openTcpFile(tcpFile: PHANDLE, DesiredAccess: ACCESS_MASK) -> NTSTATUS;
    pub fn closeTcpFile(tcpFile: HANDLE);
    pub fn tdiGetEntityIDSet(
        tcpFile: HANDLE,
        entitySet: *mut *mut TDIEntityID,
        numEntities: PDWORD,
    ) -> NTSTATUS;
    pub fn tdiGetSetOfThings(
        tcpFile: HANDLE,
        toiClass: DWORD,
        toiType: DWORD,
        toiId: DWORD,
        teiEntity: DWORD,
        teiInstance: DWORD,
        fixedPart: DWORD,
        entrySize: DWORD,
        tdiEntitySet: *mut PVOID,
        numEntries: PDWORD,
    ) -> NTSTATUS;
    pub fn tdiFreeThingSet(things: PVOID);
}

// 5j-bis. INTRA-CRATE cross-module helpers. The FULL flip moved every sibling .c into this crate,
// so these are Rust definitions in sibling modules — NOT externs. Re-exporting the real items
// (instead of re-declaring them `extern "C"`) is deliberate: a re-declaration would link fine
// against the `#[no_mangle]` symbol while silently permitting signature drift between the
// declaration and the definition (the classic cross-rlib ABI trap). Re-exports make rustc
// type-check every cross-module call against the true definition.
pub use crate::iphlpapi::dhcp_reactos::getDhcpInfoForAdapter;
pub use crate::iphlpapi::ifenum_reactos::{
    addIPAddress, consumeInterfaceName, deleteIpAddress, getIPAddrEntryForIf,
    getInterfaceBCastAddrByIndex, getInterfaceEntryByIndex, getInterfaceIPAddrByIndex,
    getInterfaceIndexTable, getInterfaceMaskByIndex, getInterfaceNameByIndex,
    getInterfacePhysicalByIndex, getNonLoopbackInterfaceIndexTable, getNumInterfaces,
    getNumNonLoopbackInterfaces, hasArp, interfaceMapFree, interfaceMapInit, isInterface,
    toIPAddressString,
};
pub use crate::iphlpapi::iphlpapi_reactos::TCPSendIoctl;
pub use crate::iphlpapi::ipstats_reactos::{
    getArpTable, getICMPStats, getIPStats, getInterfaceStatsByName, getNthIpEntity,
    getNumArpEntries, getNumRoutes, getRouteTable, getTCPStats, getTcpTable, getUDPStats,
    getUdpTable, tdiGetIpAddrsForIpEntity,
};
pub use crate::iphlpapi::resinfo_reactos::{disposeResInfo, EnumNameServers, getResInfo};
pub use crate::iphlpapi::route_reactos::{
    createIpForwardEntry, deleteIpForwardEntry, setIpForwardEntry,
};

// ---- 6. roots surfaced by the FULL-flip cargo grind. ----

// 6a. CRT stdio. getICMPStats/getIPStats (ipstats_reactos.c:303,416) really do call
// fopen("/proc/net/snmp")/fgets/fclose/_strnicmp — a Wine leftover ReactOS kept UNGUARDED (no
// #ifdef). On Windows the fopen simply fails, the `if (fp)` block is skipped and the function
// returns zeroed stats, so the calls are live code that must link. HexDump (ipstats_reactos.c:521)
// likewise calls fprintf(stderr, ...). All are msvcrt exports.
#[repr(C)]
pub struct FILE {
    _opaque: [u8; 0],
}
extern "C" {
    // crt/stdio.h — `#define _iob __iob_func()`; stderr is `(&_iob[2])`.
    pub fn __iob_func() -> *mut FILE;
    pub fn fopen(filename: *const c_char, mode: *const c_char) -> *mut FILE;
    pub fn fgets(buf: *mut c_char, n: c_int, stream: *mut FILE) -> *mut c_char;
    pub fn fclose(stream: *mut FILE) -> c_int;
    pub fn fprintf(stream: *mut FILE, fmt: *const c_char, ...) -> c_int;
    pub fn snprintf(buf: *mut c_char, n: usize, fmt: *const c_char, ...) -> c_int;
    pub fn calloc(num: usize, size: usize) -> *mut c_void;
    pub fn mbstowcs(wcstr: *mut WCHAR, mbstr: *const c_char, count: usize) -> usize;
    pub fn _strnicmp(a: *const c_char, b: *const c_char, n: usize) -> c_int;
    pub fn wcschr(s: *const WCHAR, c: WCHAR) -> *mut WCHAR;
}
/// crt/stdio.h — `#define stderr (&_iob[2])`, with `_iob` = `__iob_func()`.
#[inline]
pub unsafe fn stderr() -> *mut FILE {
    __iob_func().add(2)
}

// 6b. winsock (stdcall). inet_ntoa takes `struct in_addr` BY VALUE (winsock.h).
extern "system" {
    pub fn inet_ntoa(in_: in_addr) -> *mut c_char;
}

// 6c. kernel32 / advapi32 (stdcall) — the remaining Win32 calls.
extern "system" {
    pub fn CreateEventW(
        lpEventAttributes: LPSECURITY_ATTRIBUTES,
        bManualReset: BOOL,
        bInitialState: BOOL,
        lpName: LPCWSTR,
    ) -> HANDLE;
    pub fn RegEnumKeyExW(
        hKey: HKEY,
        dwIndex: DWORD,
        lpName: LPWSTR,
        lpcchName: LPDWORD,
        lpReserved: LPDWORD,
        lpClass: LPWSTR,
        lpcchClass: LPDWORD,
        lpftLastWriteTime: PFILETIME,
    ) -> LSTATUS;
    pub fn RegQueryInfoKeyW(
        hKey: HKEY,
        lpClass: LPWSTR,
        lpcchClass: LPDWORD,
        lpReserved: LPDWORD,
        lpcSubKeys: LPDWORD,
        lpcbMaxSubKeyLen: LPDWORD,
        lpcbMaxClassLen: LPDWORD,
        lpcValues: LPDWORD,
        lpcbMaxValueNameLen: LPDWORD,
        lpcbMaxValueLen: LPDWORD,
        lpcbSecurityDescriptor: LPDWORD,
        lpftLastWriteTime: PFILETIME,
    ) -> LSTATUS;
}

// 6d. TDI IOCTLs — CTL_CODE cast-#defines, which bindgen skips. tcpioctl.h:
//   `#define _TCP_CTL_CODE(Fn, Method, Access) CTL_CODE(FSCTL_TCP_BASE, Fn, Method, Access)`
//   FSCTL_TCP_BASE = FILE_DEVICE_NETWORK = 0x12 (xdk/iotypes.h:68)
//   CTL_CODE(t,f,m,a) = ((t)<<16) | ((a)<<14) | ((f)<<2) | (m)
//   METHOD_BUFFERED = 0, FILE_ANY_ACCESS = 0, FILE_WRITE_ACCESS = 2 (psdk/winioctl.h:133-141)
// (The same formula reproduces IOCTL_QUERY_IP_HW_ADDRESS/IOCTL_TCP_* transcribed in §5f above.)
pub const IOCTL_ICMP_ECHO_REQUEST: DWORD = 0x0012_0000; // fn 0,  METHOD_BUFFERED, ANY
pub const IOCTL_SET_IP_ADDRESS: DWORD = 0x0012_8038; //    fn 14, METHOD_BUFFERED, WRITE
pub const IOCTL_DELETE_IP_ADDRESS: DWORD = 0x0012_8040; // fn 16, METHOD_BUFFERED, WRITE

// 6e. consts bindgen didn't reach (ifmib.h is not on the include path from our wrapper).
pub const MAX_INTERFACE_NAME_LEN: usize = 256; // psdk/ifmib.h:31 — `#define MAX_INTERFACE_NAME_LEN 256`

// 6f. ipstats_reactos.c:721 local macro —
//   `#define Add2Ptr(PTR, INC) (PVOID)((ULONG_PTR)(PTR) + (INC))`
// Byte-granular pointer displacement; generic over the source pointer type so the call sites
// keep the C's implicit `(PVOID)` result.
#[inline]
pub unsafe fn Add2Ptr<T>(ptr: *mut T, inc: usize) -> *mut c_void {
    (ptr as *mut u8).add(inc) as *mut c_void
}

// 6g. enum bare-name aliases. bindgen names members `<Tag>_<Member>`; the C uses the bare member.
// The translation additionally invented THREE more spellings for the one IPHLPAddrType enum
// (`IPHLPAddrType_X`, `_IPHLP_ADDR_TYPE_X`, bare `X`) — all are aliased onto the single bindgen
// constant so every spelling denotes the same value (iphlpapi_private.h:140-142).
pub use crate::types::{
    _IPHLPAddrType_IFMtu as IFMtu, _IPHLPAddrType_IFStatus as IFStatus,
    _IPHLPAddrType_IPAAddr as IPAAddr, _IPHLPAddrType_IPABcast as IPABcast,
    _IPHLPAddrType_IPAMask as IPAMask,
};
pub use crate::types::{
    _IPHLPAddrType_IFMtu as IPHLPAddrType_IFMtu, _IPHLPAddrType_IFMtu as _IPHLP_ADDR_TYPE_IFMtu,
    _IPHLPAddrType_IFStatus as IPHLPAddrType_IFStatus,
    _IPHLPAddrType_IFStatus as _IPHLP_ADDR_TYPE_IFStatus,
    _IPHLPAddrType_IPAAddr as IPHLPAddrType_IPAAddr,
    _IPHLPAddrType_IPAAddr as _IPHLP_ADDR_TYPE_IPAAddr,
    _IPHLPAddrType_IPABcast as IPHLPAddrType_IPABcast,
    _IPHLPAddrType_IPABcast as _IPHLP_ADDR_TYPE_IPABcast,
    _IPHLPAddrType_IPAMask as IPHLPAddrType_IPAMask,
    _IPHLPAddrType_IPAMask as _IPHLP_ADDR_TYPE_IPAMask,
};
// ws2ipdef.h NL_SUFFIX_ORIGIN: the IP_ADAPTER_UNICAST_ADDRESS SuffixOrigin members are named
// `NlsoOther`/`NlsoManual`/... but iptypes.h ALSO documents them as `IpSuffixOriginOther`/...
// (the IP_SUFFIX_ORIGIN alias family). Same underlying enumerator — alias the spelling the
// translation used onto the bindgen name.
pub use crate::types::{
    NL_SUFFIX_ORIGIN_NlsoDhcp as IpSuffixOriginDhcp,
    NL_SUFFIX_ORIGIN_NlsoLinkLayerAddress as IpSuffixOriginLinkLayerAddress,
    NL_SUFFIX_ORIGIN_NlsoManual as IpSuffixOriginManual,
    NL_SUFFIX_ORIGIN_NlsoOther as IpSuffixOriginOther,
    NL_SUFFIX_ORIGIN_NlsoWellKnown as IpSuffixOriginWellKnown,
};

// 6h. wine debug channels, MACRO form. The DeepSeek output calls these BOTH as functions
// (`TRACE(fmt, ...)`, 158 sites — resolved by the extern "C" variadic block in §1-bis, which
// links to the no-op shim in iphlpapi_rust_shims.c) AND as macros (`TRACE!(fmt, ...)`, 18
// sites). Rust keeps macros and functions in separate namespaces, so both spellings can
// coexist: these macro_rules simply forward to the already-declared variadic symbol.
//
// Note this does NOT introduce a new out-of-line no-op debug function (the field-manual §9.65
// hazard — an out-of-line no-op is an optimization barrier that can uncloak a latent ABI
// mismatch into a ud2). The single no-op lives in the C shim, exactly as for every other
// wine-derived crate in the tree; these macros are pure syntax forwarding to it.
// (no `use` here: a `use` would import the name in ALL namespaces and collide with the
// same-named extern "C" fn above. Textual `#[macro_use]` scoping from lib.rs instead.)
