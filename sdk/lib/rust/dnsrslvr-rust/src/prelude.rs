//! dnsrslvr-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Grown during the cargo-check
//! grind: anything bindgen left out (CRT, inlines, sibling-C externs) is declared here once.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ========================================================================
// The ROS DPRINT channel. NDEBUG is #defined in the C build, so these compile out. They MUST stay
// MACROS: an out-of-line no-op fn is an optimization barrier that uncloaks latent ABI mismatches
// into ud2 (field manual §9.65) — and the dhcpcsvc bug where one became `loop {}` hung the caller.
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export]
macro_rules! DPRINT1 {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        unsafe {
            $crate::prelude::DbgPrint(
                concat!("(", file!(), ":", line!(), ") ", $fmt, "\0").as_ptr() as *const _
                $(, $arg)*,
            )
        }
    };
}
pub use crate::{DPRINT, DPRINT1};

extern "C" {
    pub fn DbgPrint(Format: *const c_char, ...) -> u32;
}

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

// ================= enum bare-name aliases =======================================================
// bindgen prefixes enum members with the enum type name; the C uses the bare spelling.
pub use crate::types::{
    DNS_SECTION_DnsSectionAnswer as DnsSectionAnswer,      // windns.h DNS_SECTION
    _DNS_CHARSET_DnsCharSetUnicode as DnsCharSetUnicode,   // windns.h DNS_CHARSET
};
// windns.h names the record struct DNS_RECORDW/_DnsRecordW; some bodies spell it _DNS_RECORDW.
pub type _DNS_RECORDW = _DnsRecordW;
pub type _DNS_A_DATA = DNS_A_DATA;
pub type _DNS_PTR_DATA = DNS_PTR_DATAW;
pub type DNS_PTR_DATA = DNS_PTR_DATAW;
pub type DNS_RECORD = DNS_RECORDW;

// ================= Win32 memory helpers =========================================================
// RtlZeroMemory/RtlCopyMemory. Real work, so #[inline(always)] fns are fine (the §9.65
// out-of-line-barrier hazard applies to NO-OP debug shims). Both lower to the memset/memcpy
// INTRINSIC — never hand-roll a byte loop (LLVM folds it into a self-call → silent hang).
#[inline(always)]
pub unsafe fn ZeroMemory(dst: *mut c_void, len: usize) {
    core::ptr::write_bytes(dst as *mut u8, 0, len)
}
#[inline(always)]
pub unsafe fn CopyMemory(dst: *mut c_void, src: *const c_void, len: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len)
}

// ================= consts bindgen cannot emit ===================================================
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY; // winreg.h cast macro
pub const RPC_S_OK: RPC_STATUS = 0;
pub type RPC_STATUS = LONG; // rpcdce.h: typedef long RPC_STATUS
pub type RPC_MGR_EPV = *mut c_void;

// ================= advapi32 / kernel32 / rpcrt4 =================================================
// BLOCKLIST_FUNCS=1 strips every prototype from the island by design, so the real imports are
// declared here once. `extern "system"` = stdcall on i386, cdecl elsewhere — what the import
// libraries export.
extern "system" {
    // advapi32 — registry + service control
    pub fn RegOpenKeyExW(hKey: HKEY, lpSubKey: LPCWSTR, ulOptions: DWORD, samDesired: REGSAM, phkResult: PHKEY) -> LSTATUS;
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS;
    pub fn RegQueryValueExW(hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD) -> LSTATUS;
    pub fn RegisterServiceCtrlHandlerExW(
        lpServiceName: LPCWSTR,
        lpHandlerProc: Option<unsafe extern "system" fn(DWORD, DWORD, LPVOID, LPVOID) -> DWORD>,
        lpContext: LPVOID,
    ) -> SERVICE_STATUS_HANDLE;
    pub fn SetServiceStatus(hServiceStatus: SERVICE_STATUS_HANDLE, lpServiceStatus: LPSERVICE_STATUS) -> BOOL;

    // kernel32
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn CreateThread(
        lpThreadAttributes: LPSECURITY_ATTRIBUTES,
        dwStackSize: SIZE_T,
        lpStartAddress: Option<unsafe extern "system" fn(*mut c_void) -> DWORD>,
        lpParameter: LPVOID,
        dwCreationFlags: DWORD,
        lpThreadId: LPDWORD,
    ) -> HANDLE;
    pub fn DisableThreadLibraryCalls(hLibModule: HMODULE) -> BOOL;
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    pub fn GetSystemDirectoryW(lpBuffer: LPWSTR, uSize: UINT) -> UINT;
    pub fn ExpandEnvironmentStringsW(lpSrc: LPCWSTR, lpDst: LPWSTR, nSize: DWORD) -> DWORD;
    pub fn MultiByteToWideChar(CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: LPCSTR, cbMultiByte: c_int, lpWideCharStr: LPWSTR, cchWideChar: c_int) -> c_int;
    // CRITICAL_SECTION — kernel32 re-exports the Rtl* implementations under these names.
    pub fn InitializeCriticalSection(lpCriticalSection: LPCRITICAL_SECTION);
    pub fn DeleteCriticalSection(lpCriticalSection: LPCRITICAL_SECTION);
    pub fn EnterCriticalSection(lpCriticalSection: LPCRITICAL_SECTION);
    pub fn LeaveCriticalSection(lpCriticalSection: LPCRITICAL_SECTION);

    // rpcrt4
    pub fn RpcServerUseProtseqEpW(Protseq: *mut c_ushort, MaxCalls: c_uint, Endpoint: *mut c_ushort, SecurityDescriptor: *mut c_void) -> RPC_STATUS;
    pub fn RpcServerRegisterIf(IfSpec: RPC_IF_HANDLE, MgrTypeUuid: *mut GUID, MgrEpv: RPC_MGR_EPV) -> RPC_STATUS;
    pub fn RpcServerListen(MinimumCallThreads: c_uint, MaxCalls: c_uint, DontWait: c_uint) -> RPC_STATUS;
    pub fn RpcMgmtStopServerListening(Binding: RPC_BINDING_HANDLE) -> RPC_STATUS;

    // dnsapi.dll — dnsrslvr links it (see this module's add_importlibs). Query_Main is the
    // ROS-private resolver entry point (dnsapi.spec:190 `@ stdcall Query_Main(wstr long long ptr)`,
    // prototyped in reactos/windns_undoc.h).
    pub fn Query_Main(Name: LPCWSTR, Type: WORD, Options: DWORD, pResult: *mut PDNS_RECORDW) -> DNS_STATUS;
    pub fn DnsRecordListFree(pRecordList: PDNS_RECORD, FreeType: DNS_FREE_TYPE);
    pub fn DnsRecordSetCopyEx(pRecordSet: PDNS_RECORD, CharSetIn: DNS_CHARSET, CharSetOut: DNS_CHARSET) -> PDNS_RECORD;
}
// windns.h:137-141 — DNS_FREE_TYPE enumerators. DnsFreeRecordList is the FreeType ARGUMENT
// passed to DnsRecordListFree(), not a function; bindgen prefixes it with the enum name.
pub use crate::types::{
    DNS_FREE_TYPE_DnsFreeFlat as DnsFreeFlat,
    DNS_FREE_TYPE_DnsFreeRecordList as DnsFreeRecordList,
};

// ================= strsafe.h inlines ============================================================
// strsafe ships these ONLY as `static inline`, so no linkable symbol exists — they must be
// reimplemented. Semantics per strsafe: the destination is ALWAYS null-terminated, and
// truncation is an ERROR (not a silent short write).
pub const STRSAFE_E_INVALID_PARAMETER: HRESULT = 0x8007_0057u32 as HRESULT;
pub const STRSAFE_E_INSUFFICIENT_BUFFER: HRESULT = 0x8007_007Au32 as HRESULT;

#[inline]
pub unsafe fn StringCchLengthW(psz: *const WCHAR, cchMax: usize, pcchLength: *mut usize) -> HRESULT {
    if psz.is_null() || cchMax == 0 {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    let mut n = 0usize;
    while n < cchMax && *psz.add(n) != 0 {
        n += 1;
    }
    if n >= cchMax {
        // no NUL within cchMax => the string is not valid for this bound
        return STRSAFE_E_INVALID_PARAMETER;
    }
    if !pcchLength.is_null() {
        *pcchLength = n;
    }
    0 // S_OK
}

#[inline]
pub unsafe fn StringCchCatW(pszDest: *mut WCHAR, cchDest: usize, pszSrc: *const WCHAR) -> HRESULT {
    if pszDest.is_null() || pszSrc.is_null() || cchDest == 0 {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    let mut d = 0usize;
    while d < cchDest && *pszDest.add(d) != 0 {
        d += 1;
    }
    if d >= cchDest {
        return STRSAFE_E_INVALID_PARAMETER; // destination not terminated within its bound
    }
    let mut s = 0usize;
    while *pszSrc.add(s) != 0 {
        if d + s + 1 >= cchDest {
            *pszDest.add(cchDest - 1) = 0; // strsafe terminates even on overflow
            return STRSAFE_E_INSUFFICIENT_BUFFER;
        }
        *pszDest.add(d + s) = *pszSrc.add(s);
        s += 1;
    }
    *pszDest.add(d + s) = 0;
    0 // S_OK
}

// ================= intra-crate surface ==========================================================
// The C declared these across TUs in precomp.h; the crate is now one translation unit, so they
// are re-exported here to keep a single import surface.
pub use crate::dnsrslvr::cache::{
    DnsIntCacheAddEntry, DnsIntCacheFlush, DnsIntCacheFree, DnsIntCacheGetEntries,
    DnsIntCacheGetEntryByName, DnsIntCacheInitialize, DnsIntCacheRemoveEntryByName,
    DnsIntCacheRemoveEntryItem, DnsIntFlushCacheEntry,
};
pub use crate::dnsrslvr::hostsfile::ReadHostsFile;

// ---- CRT bits the translation calls (msvcrt; dnsrslvr links it) -------------------------------
extern "C" {
    pub fn isspace(c: c_int) -> c_int;
    pub fn _wcsicmp(a: *const WCHAR, b: *const WCHAR) -> c_int;
    pub fn swprintf(buf: *mut WCHAR, format: *const WCHAR, ...) -> c_int;
    pub fn fopen(path: *const c_char, mode: *const c_char) -> *mut FILE;
    pub fn fclose(stream: *mut FILE) -> c_int;
    pub fn fgets(buf: *mut c_char, n: c_int, stream: *mut FILE) -> *mut c_char;
}

// windns.h's PDNS_RECORD follows the UNICODE build (this module is UNICODE).
pub type PDNS_RECORD = PDNS_RECORDW;

extern "C" {
    // hosts file is opened by WIDE path (the module is UNICODE) — msvcrt's _wfopen.
    pub fn _wfopen(path: *const WCHAR, mode: *const WCHAR) -> *mut FILE;
}

// ================= wide string literals =========================================================
// `w!("x")` is the LPCWSTR literal the C wrote as L"x". The buffer is a `static`, NOT a `const`:
// a const is inlined per use site, so `CONST.as_ptr()` would hand the callee the address of a
// temporary. ASCII-only by construction — a non-ASCII literal fails at COMPILE time.
#[macro_export]
macro_rules! w {
    ($s:expr) => {{
        const INPUT: &[u8] = $s.as_bytes();
        static OUT: [u16; INPUT.len() + 1] = {
            let mut out = [0u16; INPUT.len() + 1];
            let mut i = 0;
            while i < INPUT.len() {
                assert!(INPUT[i] < 0x80, "w!() takes ASCII literals only");
                out[i] = INPUT[i] as u16;
                i += 1;
            }
            out
        };
        OUT.as_ptr()
    }};
}
pub use crate::w;

// The MIDL allocator hooks are APPLICATION-supplied: the C defines them in this module's own
// rpcserver.c and the generated _s.c marshaller calls them. They are NOT rpcrt4 imports —
// declaring them as such (under the capitalised `MIDL_user_allocate` link name) manufactured
// an undefined symbol that only surfaced at LINK, never at cargo check. Re-export the crate's
// own definitions so every module sees the one real implementation.
pub use crate::dnsrslvr::rpcserver::{midl_user_allocate, midl_user_free};
