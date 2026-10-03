//! dnsapi-rust prelude — the single import surface the translated modules use.
//!
//! Bridges the bindgen `types` island (windns/adns/dnsrslvr layouts) and the translated C:
//!   1. the wine debug channel (wine/debug.h) — DeepSeek expands TRACE/ERR/FIXME to the raw
//!      `if dbch.flags & (1<<__WINE_DBCL_x) { ros_dbg_log(...) }` form; shimmed here (the
//!      lsasrv/netapi32 pattern — release-inert channel + the real variadic ros_dbg_log at link).
//!   2. the 3rdparty `adns` resolver entry points (cdecl C staticlib) + the CRT + the rpcrt4
//!      binding helpers (rpc.c) — declared once here, per the exporting module's convention.
//!   3. consts/inlines bindgen skips (NULL, NT_SUCCESS).
//! Section 4 grows as the cargo-check grind reveals bridges (adns fns, enum aliases, tail consts).
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_uchar, c_void};

// ---- 1. wine debug internals (wine/debug.h) — the lsasrv/netapi32 pattern. ros_dbg_log is a
// real variadic wine export (sdk/lib/… ); the channel struct + class consts are hand-declared
// (wine/debug.h is parsed but NOT allowlisted, so bindgen emits no duplicate). ----
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
        format: *const c_char,
        ...
    ) -> c_int;
}
// wine debug string formatters — release-inert (return NULL like lsasrv/kernel32).
#[inline]
pub fn debugstr_w(_s: *const u16) -> *const i8 {
    core::ptr::null()
}
#[inline]
pub fn debugstr_a(_s: *const i8) -> *const i8 {
    core::ptr::null()
}

// ---- 2a. CRT (cdecl, extern "C") — exported by msvcrt/ntdll, which dnsapi.dll links. ----
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn wcslen(s: *const u16) -> usize;
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16;
}

// ---- 2b. Win32 (stdcall, extern "system" → `_Name@N` on i386) — kernel32/user32. ----
extern "system" {
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: usize) -> *mut c_void;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: *mut c_void) -> BOOL;
    pub fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: *const c_char,
        cbMultiByte: c_int,
        lpWideCharStr: *mut u16,
        cchWideChar: c_int,
    ) -> c_int;
    pub fn WideCharToMultiByte(
        CodePage: UINT,
        dwFlags: DWORD,
        lpWideCharStr: *const u16,
        cchWideChar: c_int,
        lpMultiByteStr: *mut c_char,
        cbMultiByte: c_int,
        lpDefaultChar: *const c_char,
        lpUsedDefaultChar: *mut BOOL,
    ) -> c_int;
}

// ---- 2c. rpcrt4 binding helpers (stdcall) — rpc.c's DNSRSLVR_HANDLE_bind/unbind. ----
extern "system" {
    pub fn RpcStringBindingComposeW(
        ObjUuid: *mut u16,
        ProtSeq: *mut u16,
        NetworkAddr: *mut u16,
        Endpoint: *mut u16,
        Options: *mut u16,
        StringBinding: *mut *mut u16,
    ) -> c_long;
    pub fn RpcBindingFromStringBindingW(StringBinding: *mut u16, Binding: *mut *mut c_void) -> c_long;
    pub fn RpcBindingFree(Binding: *mut *mut c_void) -> c_long;
    pub fn RpcStringFreeW(String: *mut *mut u16) -> c_long;
}
use core::ffi::c_long;

// ---- 3. consts / inlines bindgen skips ----
pub const NULL: *mut c_void = core::ptr::null_mut();
pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;

#[inline(always)]
pub fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

// ================= 4. bridges (cargo-check grind: adns fns, enum aliases, consts, heap) =========

// 4a. RTL heap (ntdll, NTAPI = extern "system"). RtlGetProcessHeap() is a macro →
//     NtCurrentPeb()->ProcessHeap ≡ GetProcessHeap() (same handle). RtlFreeHeap in C takes any
//     pointer (implicit → void*); a generic wrapper preserves that so the many typed-pointer call
//     sites don't each need an `as PVOID` cast. RtlAllocateHeap's return is explicitly cast at
//     every site, so it stays a plain extern.
pub const HEAP_ZERO_MEMORY: ULONG = 0x0000_0008; // minwinbase.h — #define HEAP_ZERO_MEMORY 0x00000008
extern "system" {
    pub fn RtlAllocateHeap(HeapHandle: PVOID, Flags: ULONG, Size: usize) -> PVOID;
}
#[inline(always)]
pub unsafe fn RtlGetProcessHeap() -> PVOID {
    GetProcessHeap() as PVOID
}
#[inline(always)]
pub unsafe fn RtlFreeHeap<T>(HeapHandle: PVOID, Flags: ULONG, BaseAddress: *mut T) -> u8 {
    extern "system" {
        fn RtlFreeHeap(HeapHandle: PVOID, Flags: ULONG, BaseAddress: PVOID) -> u8;
    }
    RtlFreeHeap(HeapHandle, Flags, BaseAddress as PVOID)
}

// 4b. extra CRT (cdecl, extern "C") — msvcrt/ntdll exports the translation calls. ----
extern "C" {
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
    pub fn _stricmp(a: *const c_char, b: *const c_char) -> c_int;
    pub fn towupper(c: c_int) -> c_int;
    pub fn wcsstr(haystack: *const u16, needle: *const u16) -> *mut u16;
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16;
    // DbgPrint is a real ntdll cdecl-variadic export; declaring a C-variadic is stable Rust.
    pub fn DbgPrint(Format: *const c_char, ...) -> u32;
}

// 4c. Win32 / ws2_32 / iphlpapi / ntdll-Rtl (stdcall → `_Name@N` on i386, extern "system"). ----
extern "system" {
    pub fn lstrlenW(lpString: *const u16) -> c_int;
    pub fn lstrlenA(lpString: *const c_char) -> c_int;
    pub fn lstrcmpiW(a: *const u16, b: *const u16) -> c_int;
    pub fn lstrcmpiA(a: *const c_char, b: *const c_char) -> c_int;
    pub fn lstrcpyW(dst: *mut u16, src: *const u16) -> *mut u16;
    pub fn lstrcpyA(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn ntohl(netlong: u32) -> u32; // ws2_32
    pub fn inet_addr(cp: *const c_char) -> u32; // ws2_32
    pub fn GetSystemTimeAsFileTime(lpSystemTimeAsFileTime: *mut FILETIME);
    pub fn SystemTimeToFileTime(lpSystemTime: *const SYSTEMTIME, lpFileTime: *mut FILETIME) -> BOOL;
    pub fn GetNetworkParams(pFixedInfo: *mut FIXED_INFO, pOutBufLen: *mut ULONG) -> DWORD; // iphlpapi
    pub fn GetAdaptersAddresses(
        Family: ULONG,
        Flags: ULONG,
        Reserved: PVOID,
        AdapterAddresses: PIP_ADAPTER_ADDRESSES,
        SizePointer: *mut ULONG,
    ) -> ULONG; // iphlpapi
    // ntdll RTL string→address (NTAPI). Strict is BOOLEAN (u8).
    pub fn RtlIpv4StringToAddressW(
        S: PCWSTR,
        Strict: u8,
        Terminator: *mut PCWSTR,
        Addr: PIN_ADDR,
    ) -> NTSTATUS;
    pub fn RtlIpv6StringToAddressW(
        S: PCWSTR,
        Terminator: *mut PCWSTR,
        Addr: PIN6_ADDR,
    ) -> NTSTATUS;
    // dnsrslvr MIDL client stubs (dnsrslvr_c.c, __stdcall). DNSRSLVR_HANDLE = LPWSTR.
    pub fn CRrReadCache(
        pwszServerName: DNSRSLVR_HANDLE,
        ppCacheEntries: *mut *mut DNS_CACHE_ENTRY,
    ) -> DWORD;
    pub fn R_ResolverFlushCacheEntry(
        pwszServerName: DNSRSLVR_HANDLE,
        pszName: LPCWSTR,
        wType: WORD,
    ) -> DWORD;
    pub fn R_ResolverQuery(
        pwszServerName: DNSRSLVR_HANDLE,
        pwsName: LPCWSTR,
        wType: WORD,
        Flags: DWORD,
        dwRecords: *mut DWORD,
        ppResultRecords: *mut *mut DNS_RECORDW,
    ) -> DWORD;
}

// 4d. adns 3rdparty async resolver (cdecl C staticlib, -DADNS_JGAA_WIN32). Signatures from
//     vendor/adns/src/adns.h. adns_state/adns_answer/adns_{rrtype,initflags,queryflags}
//     come from the bindgen `types` island; in_addr is a winsock base type.
extern "C" {
    pub fn adns_init(
        newstate_r: *mut adns_state,
        flags: adns_initflags,
        diagfile: *mut c_void, /* FILE* */
    ) -> c_int;
    pub fn adns_finish(ads: adns_state);
    pub fn adns_synchronous(
        ads: adns_state,
        owner: *const c_char,
        type_: adns_rrtype,
        flags: adns_queryflags,
        answer_r: *mut *mut adns_answer,
    ) -> c_int;
    pub fn adns_numservers(state: adns_state) -> c_int;
    pub fn adns_addserver(state: adns_state, server: in_addr);
    pub fn adns_ccf_search(state: adns_state, fname: *const c_char, lno: c_int, buf: *const c_char);
}

// 4e. adns enum members — bindgen names them <enumtype>_<member>; re-export the bare C name.
//     The *status* codes are compared against `int` adns results, so they are transcribed as i32
//     (the flag/rrtype/queryflags ones keep their natural c_uint type).
pub const adns_s_ok: c_int = crate::types::adns_status_adns_s_ok as c_int;
pub const adns_s_nomemory: c_int = crate::types::adns_status_adns_s_nomemory as c_int;
pub const adns_s_systemfail: c_int = crate::types::adns_status_adns_s_systemfail as c_int;
pub const adns_s_prohibitedcname: c_int = crate::types::adns_status_adns_s_prohibitedcname as c_int;
pub use crate::types::{
    adns_initflags_adns_if_noenv as adns_if_noenv,
    adns_initflags_adns_if_noerrprint as adns_if_noerrprint,
    adns_initflags_adns_if_noserverwarn as adns_if_noserverwarn,
    adns_queryflags_adns_qf_search as adns_qf_search,
    adns_rrtype_adns_r_addr as adns_r_addr,
};

// 4f. windns.h enum members — bindgen prefixes; the translation uses the bare C name. DNS_SECTION_*
//     and DNS_FREE_TYPE_* have no leading `_`; DNS_CHARSET_*/DNS_NAME_FORMAT_* do (bindgen read the
//     `_`-tagged struct name). memory.rs additionally spells DnsFree* with the full `_DNS_FREE_TYPE_`
//     prefix — alias that spelling too.
pub use crate::types::{
    DNS_FREE_TYPE_DnsFreeFlat as DnsFreeFlat,
    DNS_FREE_TYPE_DnsFreeFlat as _DNS_FREE_TYPE_DnsFreeFlat,
    DNS_FREE_TYPE_DnsFreeParsedMessageFields as DnsFreeParsedMessageFields,
    DNS_FREE_TYPE_DnsFreeParsedMessageFields as _DNS_FREE_TYPE_DnsFreeParsedMessageFields,
    DNS_FREE_TYPE_DnsFreeRecordList as DnsFreeRecordList,
    DNS_FREE_TYPE_DnsFreeRecordList as _DNS_FREE_TYPE_DnsFreeRecordList,
    DNS_SECTION_DnsSectionAdditional as DnsSectionAdditional,
    DNS_SECTION_DnsSectionAnswer as DnsSectionAnswer,
    DNS_SECTION_DnsSectionAuthority as DnsSectionAuthority,
    DNS_SECTION_DnsSectionQuestion as DnsSectionQuestion,
    _DNS_CHARSET_DnsCharSetAnsi as DnsCharSetAnsi,
    _DNS_CHARSET_DnsCharSetUnicode as DnsCharSetUnicode,
    _DNS_CHARSET_DnsCharSetUnknown as DnsCharSetUnknown,
    _DNS_CHARSET_DnsCharSetUtf8 as DnsCharSetUtf8,
    _DNS_NAME_FORMAT_DnsNameDomain as DnsNameDomain,
    _DNS_NAME_FORMAT_DnsNameDomainLabel as DnsNameDomainLabel,
    _DNS_NAME_FORMAT_DnsNameHostnameFull as DnsNameHostnameFull,
    _DNS_NAME_FORMAT_DnsNameHostnameLabel as DnsNameHostnameLabel,
    _DNS_NAME_FORMAT_DnsNameSrvRecord as DnsNameSrvRecord,
    _DNS_NAME_FORMAT_DnsNameWildcard as DnsNameWildcard,
};

// 4g. type aliases bindgen emitted only under a `W`/`_`-decorated name (windns UNICODE build),
//     plus the record pointer the translation uses pervasively (DNS_RECORD ≡ DNS_RECORDW here).
pub type DNS_RECORD = _DnsRecordW;
pub type PDNS_RECORD = *mut _DnsRecordW;
pub type DNS_TXT_DATA = DNS_TXT_DATAW; // windns.h — #define DNS_TXT_DATA DNS_TXT_DATAW (UNICODE)
pub type DNS_PTR_DATA = DNS_PTR_DATAW;
pub type DNS_MX_DATA = DNS_MX_DATAW;
pub type DNS_MINFO_DATA = DNS_MINFO_DATAW;
pub type SOCKADDR_IN = sockaddr_in; // ws2def.h — SOCKADDR_IN ≡ struct sockaddr_in

// dnsapi-internal handle (precomp.h: `typedef struct { adns_state State; } WINDNS_CONTEXT;`).
#[repr(C)]
pub struct WINDNS_CONTEXT {
    pub State: adns_state,
}

// 4h. iphlpapi structs the dnsapi bindgen profile did not pull in (iptypes.h). Transcribed exactly
//     (GetNetworkParams / GetAdaptersAddresses fill these). MAX_HOSTNAME_LEN+4=132, MAX_DOMAIN_NAME_LEN+4=132,
//     MAX_SCOPE_ID_LEN+4=260, IP_ADDRESS_STRING = char[16]. IP_ADAPTER_ADDRESSES is only walked by
//     .Next and read via .FirstUnicastAddress; the leading fields carry the accessed offsets.
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
pub struct FIXED_INFO {
    pub HostName: [c_char; 132],
    pub DomainName: [c_char; 132],
    pub CurrentDnsServer: PIP_ADDR_STRING,
    pub DnsServerList: IP_ADDR_STRING,
    pub NodeType: UINT,
    pub ScopeId: [c_char; 260],
    pub EnableRouting: UINT,
    pub EnableProxy: UINT,
    pub EnableDns: UINT,
}
pub type PFIXED_INFO = *mut FIXED_INFO;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SOCKET_ADDRESS {
    pub lpSockaddr: *mut c_void, // LPSOCKADDR
    pub iSockaddrLength: c_int,
}
#[repr(C)]
pub struct IP_ADAPTER_UNICAST_ADDRESS {
    pub Length: ULONG,
    pub Flags: DWORD,
    pub Next: *mut IP_ADAPTER_UNICAST_ADDRESS,
    pub Address: SOCKET_ADDRESS,
    // (LH tail — PrefixOrigin/…/OnLinkPrefixLength — unused by dnsapi, elided; the OS fills it in
    //  the heap buffer we hand it, and only the prefix above is dereferenced.)
}
#[repr(C)]
pub struct IP_ADAPTER_ADDRESSES {
    pub Alignment: u64, // union { ULONGLONG Alignment; struct { ULONG Length; DWORD IfIndex; } }
    pub Next: *mut IP_ADAPTER_ADDRESSES,
    pub AdapterName: *mut c_char,
    pub FirstUnicastAddress: *mut IP_ADAPTER_UNICAST_ADDRESS,
    // (remaining ~380 bytes of IP_ADAPTER_ADDRESSES_LH elided — never dereferenced by dnsapi.)
}
pub type PIP_ADAPTER_ADDRESSES = *mut IP_ADAPTER_ADDRESSES;

// GetAdaptersAddresses family/flags (iptypes.h — verified against sdk/include/psdk/iptypes.h).
pub const GAA_FLAG_SKIP_ANYCAST: ULONG = 0x0002;
pub const GAA_FLAG_SKIP_MULTICAST: ULONG = 0x0004;
pub const GAA_FLAG_SKIP_DNS_SERVER: ULONG = 0x0008;
pub const GAA_FLAG_SKIP_FRIENDLY_NAME: ULONG = 0x0020;

// 4i. consts bindgen skipped — each header-cited.
pub const CP_ACP: UINT = 0; // winnls.h — ANSI code page
pub const CP_UTF8: UINT = 65001; // winnls.h — UTF-8 code page
pub const AF_INET: ULONG = 2; // winsock2.h — internetwork: UDP, TCP, etc.
pub const INADDR_LOOPBACK: u32 = 0x7f00_0001; // winsock.h — 127.0.0.1 (u_long)
pub const INADDR_ANY: u32 = 0x0000_0000; // winsock.h — (u_long)0
pub const INADDR_NONE: u32 = 0xffff_ffff; // winsock.h — 0xffffffff
pub const MAX_PATH: usize = 260; // minwindef.h — #define MAX_PATH 260
// winerror.h
pub const ERROR_SUCCESS: DWORD = 0;
pub const ERROR_FILE_NOT_FOUND: DWORD = 2;
pub const ERROR_OUTOFMEMORY: DWORD = 14;
pub const ERROR_INVALID_PARAMETER: DWORD = 87;
pub const ERROR_INVALID_NAME: DWORD = 123;
pub const DNS_ERROR_NON_RFC_NAME: DWORD = 9556; // winerror.h
pub const DNS_ERROR_INVALID_NAME_CHAR: DWORD = 9560; // winerror.h
pub const DNS_ERROR_NUMERIC_NAME: DWORD = 9561; // winerror.h

// 4j. strsafe.h `static inline` string helpers — NO linkable symbol; implement the bounded copies
//     inline (the iphlpapi/winsock lane pattern). Cch* count CHARACTERS, Cb* count BYTES.
pub const S_OK: HRESULT = 0;
pub const STRSAFE_E_INVALID_PARAMETER: HRESULT = 0x8007_0057u32 as i32 as HRESULT;
pub const STRSAFE_E_INSUFFICIENT_BUFFER: HRESULT = 0x8007_007Au32 as i32 as HRESULT;
#[inline]
pub unsafe fn StringCchLengthA(psz: *const c_char, cchMax: usize, pcch: *mut usize) -> HRESULT {
    if psz.is_null() || cchMax == 0 {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    let mut n = 0usize;
    while n < cchMax && *psz.add(n) != 0 {
        n += 1;
    }
    if n >= cchMax {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    if !pcch.is_null() {
        *pcch = n;
    }
    S_OK
}
#[inline]
pub unsafe fn StringCbLengthA(psz: *const c_char, cbMax: usize, pcb: *mut usize) -> HRESULT {
    let mut n = 0usize;
    let hr = StringCchLengthA(psz, cbMax, &mut n);
    if hr == S_OK && !pcb.is_null() {
        *pcb = n; // 1 byte / char (ANSI)
    }
    hr
}
#[inline]
pub unsafe fn StringCbLengthW(psz: *const u16, cbMax: usize, pcb: *mut usize) -> HRESULT {
    if psz.is_null() || cbMax < 2 {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    let cchMax = cbMax / 2;
    let mut n = 0usize;
    while n < cchMax && *psz.add(n) != 0 {
        n += 1;
    }
    if n >= cchMax {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    if !pcb.is_null() {
        *pcb = n * 2; // bytes
    }
    S_OK
}
#[inline]
pub unsafe fn StringCchCopyA(pszDest: *mut c_char, cchDest: usize, pszSrc: *const c_char) -> HRESULT {
    if pszDest.is_null() || cchDest == 0 {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    let mut i = 0usize;
    while i + 1 < cchDest && *pszSrc.add(i) != 0 {
        *pszDest.add(i) = *pszSrc.add(i);
        i += 1;
    }
    let truncated = *pszSrc.add(i) != 0;
    *pszDest.add(i) = 0;
    if truncated {
        STRSAFE_E_INSUFFICIENT_BUFFER
    } else {
        S_OK
    }
}
#[inline]
pub unsafe fn StringCbCopyA(pszDest: *mut c_char, cbDest: usize, pszSrc: *const c_char) -> HRESULT {
    StringCchCopyA(pszDest, cbDest, pszSrc) // 1 byte / char (ANSI)
}
#[inline]
pub unsafe fn StringCbCopyW(pszDest: *mut u16, cbDest: usize, pszSrc: *const u16) -> HRESULT {
    if pszDest.is_null() || cbDest < 2 {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    let cchDest = cbDest / 2;
    let mut i = 0usize;
    while i + 1 < cchDest && *pszSrc.add(i) != 0 {
        *pszDest.add(i) = *pszSrc.add(i);
        i += 1;
    }
    let truncated = *pszSrc.add(i) != 0;
    *pszDest.add(i) = 0;
    if truncated {
        STRSAFE_E_INSUFFICIENT_BUFFER
    } else {
        S_OK
    }
}
#[inline]
pub unsafe fn StringCchCatA(pszDest: *mut c_char, cchDest: usize, pszSrc: *const c_char) -> HRESULT {
    if pszDest.is_null() || cchDest == 0 {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    let mut dlen = 0usize;
    while dlen < cchDest && *pszDest.add(dlen) != 0 {
        dlen += 1;
    }
    if dlen >= cchDest {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    let mut i = 0usize;
    while dlen + i + 1 < cchDest && *pszSrc.add(i) != 0 {
        *pszDest.add(dlen + i) = *pszSrc.add(i);
        i += 1;
    }
    let truncated = *pszSrc.add(i) != 0;
    *pszDest.add(dlen + i) = 0;
    if truncated {
        STRSAFE_E_INSUFFICIENT_BUFFER
    } else {
        S_OK
    }
}

// 4k. precomp.h `static inline` dns_strdup_* charset-converting duplicators — NO linkable symbol;
//     reimplemented verbatim (CP_UTF8/CP_ACP MultiByte↔Wide + Heap{Alloc,Free}).
#[inline]
pub unsafe fn dns_strdup_uw(str: *const c_char) -> LPWSTR {
    if str.is_null() {
        return core::ptr::null_mut();
    }
    let len = MultiByteToWideChar(CP_UTF8, 0, str, -1, core::ptr::null_mut(), 0);
    let ret = HeapAlloc(GetProcessHeap(), 0, len as usize * 2) as LPWSTR;
    if !ret.is_null() {
        MultiByteToWideChar(CP_UTF8, 0, str, -1, ret, len);
    }
    ret
}
#[inline]
pub unsafe fn dns_strdup_aw(str: LPCSTR) -> LPWSTR {
    if str.is_null() {
        return core::ptr::null_mut();
    }
    let len = MultiByteToWideChar(CP_ACP, 0, str, -1, core::ptr::null_mut(), 0);
    let ret = HeapAlloc(GetProcessHeap(), 0, len as usize * 2) as LPWSTR;
    if !ret.is_null() {
        MultiByteToWideChar(CP_ACP, 0, str, -1, ret, len);
    }
    ret
}
#[inline]
pub unsafe fn dns_strdup_a(src: LPCSTR) -> LPSTR {
    if src.is_null() {
        return core::ptr::null_mut();
    }
    let dst = HeapAlloc(GetProcessHeap(), 0, (lstrlenA(src) as usize + 1)) as LPSTR;
    if !dst.is_null() {
        lstrcpyA(dst, src);
    }
    dst
}
#[inline]
pub unsafe fn dns_strdup_u(src: *const c_char) -> *mut c_char {
    if src.is_null() {
        return core::ptr::null_mut();
    }
    let dst = HeapAlloc(GetProcessHeap(), 0, strlen(src) + 1) as *mut c_char;
    if !dst.is_null() {
        strcpy(dst, src);
    }
    dst
}
#[inline]
pub unsafe fn dns_strdup_w(src: LPCWSTR) -> LPWSTR {
    if src.is_null() {
        return core::ptr::null_mut();
    }
    let dst = HeapAlloc(GetProcessHeap(), 0, (lstrlenW(src) as usize + 1) * 2) as LPWSTR;
    if !dst.is_null() {
        lstrcpyW(dst, src);
    }
    dst
}
#[inline]
pub unsafe fn dns_strdup_wa(str: LPCWSTR) -> LPSTR {
    if str.is_null() {
        return core::ptr::null_mut();
    }
    let len = WideCharToMultiByte(CP_ACP, 0, str, -1, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut());
    let ret = HeapAlloc(GetProcessHeap(), 0, len as usize) as LPSTR;
    if !ret.is_null() {
        WideCharToMultiByte(CP_ACP, 0, str, -1, ret, len, core::ptr::null_mut(), core::ptr::null_mut());
    }
    ret
}
#[inline]
pub unsafe fn dns_strdup_wu(str: LPCWSTR) -> *mut c_char {
    if str.is_null() {
        return core::ptr::null_mut();
    }
    let len = WideCharToMultiByte(CP_UTF8, 0, str, -1, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut());
    let ret = HeapAlloc(GetProcessHeap(), 0, len as usize) as *mut c_char;
    if !ret.is_null() {
        WideCharToMultiByte(CP_UTF8, 0, str, -1, ret, len, core::ptr::null_mut(), core::ptr::null_mut());
    }
    ret
}
#[inline]
pub unsafe fn dns_strdup_au(src: LPCSTR) -> *mut c_char {
    let mut dst: *mut c_char = core::ptr::null_mut();
    let ret = dns_strdup_aw(src);
    if !ret.is_null() {
        dst = dns_strdup_wu(ret);
        HeapFree(GetProcessHeap(), 0, ret as LPVOID);
    }
    dst
}
#[inline]
pub unsafe fn dns_strdup_ua(src: *const c_char) -> LPSTR {
    let mut dst: LPSTR = core::ptr::null_mut();
    let ret = dns_strdup_uw(src);
    if !ret.is_null() {
        dst = dns_strdup_wa(ret);
        HeapFree(GetProcessHeap(), 0, ret as LPVOID);
    }
    dst
}

// 4l. DNS_RRSET builder macros (windns.h) — implemented as inline fns. INIT points pLastRR at the
//     address of pFirstRR (the classic head-insert trick) so the first ADD threads through pFirstRR.
#[inline]
pub unsafe fn DNS_RRSET_INIT(rrset: *mut DNS_RRSET) {
    (*rrset).pFirstRR = core::ptr::null_mut();
    (*rrset).pLastRR = core::ptr::addr_of_mut!((*rrset).pFirstRR) as PDNS_RECORDW;
}
#[inline]
pub unsafe fn DNS_RRSET_ADD(rrset: *mut DNS_RRSET, pnewRR: PDNS_RECORD) {
    (*(*rrset).pLastRR).pNext = pnewRR;
    (*rrset).pLastRR = pnewRR;
}
#[inline]
pub unsafe fn DNS_RRSET_TERMINATE(rrset: *mut DNS_RRSET) {
    (*(*rrset).pLastRR).pNext = core::ptr::null_mut();
}

// 4m. intra-crate helpers the C compiled in one TU. In Rust each module is its own scope, so the
//     shared helpers are made `pub` at their definition site and re-exported here (netapi32 pattern).
pub use crate::dnsapi::adns::DnsIntTranslateAdnsToDNS_STATUS;
pub use crate::dnsapi::query::DnsIntFreeRecordList;
