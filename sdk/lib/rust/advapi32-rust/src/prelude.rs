//! Crate-wide prelude: the universal symbols the per-file translations expect in
//! scope. Body modules add `use crate::prelude::*;`.
//!
//! Base = the full user-mode Native API + security surface from ntdll-sys (Nt*/
//! Rtl*/Ldr* + SID/ACL/TOKEN/_SECURITY_DESCRIPTOR types). On top we add the Win32
//! aliases/constants the C uses that bindgen doesn't emit, the no-op CRT/Wine
//! helpers, and `extern` decls for advapi32's own not-yet-translated exports
//! (resolved against the C cohab at link). Subsystem-specific types/consts
//! (SCM/RPC marshalling) live in their owning module, not here.
#![allow(non_snake_case, non_upper_case_globals, dead_code)]

pub use ntdll_sys::nt::*;

// Win32 handle types exist in BOTH ntdll-sys and advapi32-sys as distinct nominal
// types (same layout, different HINSTANCE__/opaque tag). advapi32-sys structs
// (e.g. CRYPTPROV.hModule) embed the advapi32-sys versions, so standardize the
// whole crate on those (explicit `use` wins over the nt::* glob above — no
// ambiguity). This makes GetProcAddress/LoadLibraryW etc. interoperate with the
// advapi32-sys struct fields without per-call casts.
pub use advapi32_sys::{HMODULE, HINSTANCE, HWND, HINSTANCE__};

// TRUE/FALSE: bindgen types the bare `#define TRUE 1` macro as u32, but BOOL is
// c_int (i32) → `return FALSE` in a `-> BOOL` fn mismatches. Explicit defs here
// (correct BOOL type) shadow the nt glob's u32 versions. Files import ONLY this
// prelude (not ntdll_sys::nt directly) so there's a single, correctly-typed pair.
pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;

// advapi32-specific Win32 SDK / RPC-client / winsafer / aclapi / EFS types from
// advapi32-sys. CURATED (not a glob) so the base types advapi32-sys re-bindgen'd
// (HANDLE/ULONG/SID/GUID/…) don't collide with ntdll_sys::nt's. Grows per wave.
pub use advapi32_sys::{
    // aclapi — TRUSTEE / EXPLICIT_ACCESS
    TRUSTEE_W, PTRUSTEE_W, TRUSTEEW, PTRUSTEEW, _TRUSTEE_W,
    TRUSTEE_A, PTRUSTEE_A, TRUSTEEA, PTRUSTEEA, _TRUSTEE_A,
    EXPLICIT_ACCESS_W, PEXPLICIT_ACCESS_W, EXPLICIT_ACCESSW, PEXPLICIT_ACCESSW,
    EXPLICIT_ACCESS_A, PEXPLICIT_ACCESS_A, EXPLICIT_ACCESSA, PEXPLICIT_ACCESSA,
    ACCESS_MODE, MULTIPLE_TRUSTEE_OPERATION, TRUSTEE_FORM, TRUSTEE_TYPE,
    // winsafer
    SAFER_LEVEL_HANDLE, SAFER_POLICY_INFO_CLASS,
    SAFER_CODE_PROPERTIES_V1, SAFER_CODE_PROPERTIES_V2, SAFER_CODE_PROPERTIES,
    PSAFER_CODE_PROPERTIES,
    // RPC client
    RPC_STATUS, handle_t, RPC_UNICODE_STRING, PRPC_UNICODE_STRING,
    // winreg RPC + registry (HKEY/PHKEY come from ntdll_sys::nt — base type)
    PREGISTRY_SERVER_NAME, REGSAM,
    // wine/crypt
    ustring,
    // consts
    SAFER_TOKEN_NULL_IF_EQUAL, SAFER_TOKEN_COMPARE_ONLY, SAFER_TOKEN_MAKE_INERT,
    // EFS (winefs)
    ENCRYPTION_CERTIFICATE, PENCRYPTION_CERTIFICATE,
    ENCRYPTION_CERTIFICATE_LIST, PENCRYPTION_CERTIFICATE_LIST,
    ENCRYPTION_CERTIFICATE_HASH, PENCRYPTION_CERTIFICATE_HASH,
    ENCRYPTION_CERTIFICATE_HASH_LIST, PENCRYPTION_CERTIFICATE_HASH_LIST,
    // aclapi — objects/inherited/SE_OBJECT_TYPE/progress (Wave 2)
    SE_OBJECT_TYPE, PROG_INVOKE_SETTING, FN_PROGRESS,
    OBJECTS_AND_NAME_W, POBJECTS_AND_NAME_W, OBJECTS_AND_NAME_A, POBJECTS_AND_NAME_A,
    OBJECTS_AND_SID, POBJECTS_AND_SID,
    INHERITED_FROMW, PINHERITED_FROMW, INHERITED_FROMA, PINHERITED_FROMA,
    // winsvc — services / SCM (Wave 2)
    SERVICE_STATUS_HANDLE, RPC_SERVICE_STATUS_HANDLE, LPSERVICE_STATUS,
    LPHANDLER_FUNCTION, LPHANDLER_FUNCTION_EX,
    LPSERVICE_MAIN_FUNCTIONW, LPSERVICE_MAIN_FUNCTIONA,
    SERVICE_TABLE_ENTRYW, LPSERVICE_TABLE_ENTRYW,
    SERVICE_TABLE_ENTRYA, LPSERVICE_TABLE_ENTRYA,
    SCM_CONTROL_PACKET, PSCM_CONTROL_PACKET, SCM_REPLY_PACKET, PSCM_REPLY_PACKET,
    PFN_OBJECT_MGR_FUNCTS,
    // crypto contexts (cryptlib md4/md5)
    MD4_CTX, MD5_CTX,
    // consts (winsvc / winnls / ntsecapi) — Wave 2
    CP_ACP, MAX_SERVICE_NAME_LENGTH,
    SERVICE_CONTROL_STOP, SERVICE_CONTROL_PAUSE, SERVICE_CONTROL_CONTINUE,
    SERVICE_CONTROL_INTERROGATE, SERVICE_CONTROL_SHUTDOWN,
    RTL_ENCRYPT_OPTION_SAME_PROCESS, RTL_ENCRYPT_OPTION_CROSS_PROCESS,
    RTL_ENCRYPT_OPTION_SAME_LOGON,
    // winsvc / services.h service-control codes (Wave 2)
    SERVICE_CONTROL_NETBINDDISABLE, SERVICE_CONTROL_DEVICEEVENT,
    SERVICE_CONTROL_START_SHARE, SERVICE_CONTROL_START_OWN,
    // ---- Wave 3 ----
    // wincrypt provider/key/hash handles + algorithm id (wine/crypt.rs)
    ALG_ID, CRYPTHASH, CRYPTKEY, CRYPTPROV, HCRYPTHASH, HCRYPTKEY, HCRYPTPROV,
    PCRYPTHASH, PCRYPTKEY, PCRYPTPROV,
    // eventlogrpc RPC client marshalling (service/eventlog.rs)
    EVENTLOG_HANDLE_A, EVENTLOG_HANDLE_W, IELF_HANDLE, RPC_STRING, PRPC_STRING,
    RPC_CLIENT_ID, RPC_CSTR, RPC_WSTR,
    // aclapi progress callbacks (sec/misc.rs) — A/W variants (FN_PROGRESS in Wave 1)
    FN_PROGRESSA, FN_PROGRESSW, PPROG_INVOKE_SETTING,
    // ntmarta dynamic-load table (sec/misc.rs)
    NTMARTA, PNTMARTA,
    // ntsecapi MSV1_0 logon (misc/logon.rs)
    MSV1_0_INTERACTIVE_LOGON, MSV1_0_INTERACTIVE_PROFILE, SECURITY_LOGON_TYPE,
    // ntsecapi LSA public types (sec/lsa.rs)
    LSA_HANDLE, PLSA_HANDLE, LSA_STRING, LSA_UNICODE_STRING, PLSA_UNICODE_STRING,
    LSA_OBJECT_ATTRIBUTES, LSA_OPERATIONAL_MODE, PLSA_ENUMERATION_HANDLE,
    PLSA_REFERENCED_DOMAIN_LIST, PLSA_TRANSLATED_NAME, PLSA_TRANSLATED_SID,
    PLSA_TRANSLATED_SID2, PLSA_TRUST_INFORMATION, PLSA_OBJECT_ATTRIBUTES,
    PLSA_FOREST_TRUST_COLLISION_INFORMATION, PLSA_FOREST_TRUST_INFORMATION,
    POLICY_INFORMATION_CLASS, POLICY_DOMAIN_INFORMATION_CLASS,
    TRUSTED_INFORMATION_CLASS, PTRUSTED_DOMAIN_AUTH_INFORMATION,
    PTRUSTED_DOMAIN_INFORMATION_EX, PRPC_SID,
    // lsa_c RPC-marshalled LSAPR_* types (sec/lsa.rs)
    LSAPR_HANDLE, PLSAPR_HANDLE, LSAPR_ACCOUNT_ENUM_BUFFER, LSAPR_CR_CIPHER_VALUE,
    PLSAPR_CR_CIPHER_VALUE, LSAPR_PRIVILEGE_ENUM_BUFFER, LSAPR_PRIVILEGE_SET,
    PLSAPR_PRIVILEGE_SET, LSAPR_SID_ENUM_BUFFER, LSAPR_SR_SECURITY_DESCRIPTOR,
    PLSAPR_SR_SECURITY_DESCRIPTOR, LSAPR_TRANSLATED_NAMES, LSAPR_TRANSLATED_SIDS,
    LSAPR_TRANSLATED_SIDS_EX2, LSAPR_TRUSTED_ENUM_BUFFER, LSAPR_TRUSTED_ENUM_BUFFER_EX,
    LSAPR_USER_RIGHT_SET, PLSAPR_OBJECT_ATTRIBUTES, PLSAPR_POLICY_DOMAIN_INFORMATION,
    PLSAPR_POLICY_INFORMATION, PLSAPR_REFERENCED_DOMAIN_LIST, PLSAPR_SERVER_NAME,
    PLSAPR_SID_INFORMATION, PLSAPR_TRUSTED_DOMAIN_AUTH_INFORMATION_INTERNAL,
    PLSAPR_TRUSTED_DOMAIN_INFO, PLSAPR_TRUSTED_DOMAIN_INFORMATION_EX,
    PLSAPR_TRUST_INFORMATION,
    // lsa_c lookup-level enum (sec/lsa.rs)
    LSAP_LOOKUP_LEVEL,
    // ---- Wave 3 consts (present in bindgen) ----
    // wincrypt provider magic numbers + key/keyset flags (wine/crypt.rs)
    MAGIC_CRYPTPROV, MAGIC_CRYPTKEY, MAGIC_CRYPTHASH,
    CRYPT_USER_DEFAULT, CRYPT_MACHINE_DEFAULT, CRYPT_DELETE_DEFAULT, CRYPT_DELETEKEYSET,
    // ntsecapi policy lookup flag (sec/lsa.rs)
    POLICY_LOOKUP_NAMES,
    // wincred.h credential types + consts (wine/cred.rs)
    CREDENTIALW, CREDENTIALA, PCREDENTIALW, PCREDENTIALA,
    CERT_CREDENTIAL_INFO, PCERT_CREDENTIAL_INFO, USERNAME_TARGET_CREDENTIAL_INFO,
    CREDENTIAL_TARGET_INFORMATIONW, CREDENTIAL_TARGET_INFORMATIONA,
    PCREDENTIAL_TARGET_INFORMATIONW, PCREDENTIAL_TARGET_INFORMATIONA,
    CRED_MARSHAL_TYPE, PCRED_MARSHAL_TYPE,
    CRED_TYPE_GENERIC, CRED_TYPE_DOMAIN_PASSWORD,
    CRED_PERSIST_NONE, CRED_PERSIST_SESSION, CRED_PERSIST_LOCAL_MACHINE,
    CRED_PERSIST_ENTERPRISE, CRED_PRESERVE_CREDENTIAL_BLOB,
    CERT_HASH_LENGTH, CP_UTF8,
};

// PROVFUNCS / VTableProvStruc: bindgen names the structs `tagPROVFUNCS` /
// `_VTableProvStruc` (no bare typedef alias emitted) — alias them (wine/crypt.rs).
pub use advapi32_sys::tagPROVFUNCS as PROVFUNCS;
pub use advapi32_sys::_VTableProvStruc as VTableProvStruc;

// NOTE: enum-variant bare-name consts (SECURITY_LOGON_TYPE Interactive/Network/
// Batch/Service, MsV1_0InteractiveLogon, LsapLookupWksta, ForestTrustDomainInfo,
// TrustedDomain*Internal) are defined module-LOCALLY in their owning files
// (misc/logon.rs, sec/lsa.rs) rather than here: common identifiers like `Service`
// in the global prelude get treated as const-patterns by `let Service = ...`
// bindings in sibling modules (sctrl.rs), breaking them.

// ---- Wave 3 hand-defined consts (cast-macros bindgen can't evaluate) ----
// NTE_* crypto error HRESULTs (winerror.h _HRESULT_TYPEDEF_) — passed to
// SetLastError(ULONG) in wine/crypt.rs, so typed as DWORD.
pub const NTE_BAD_UID: DWORD = 0x8009_0001;
pub const NTE_BAD_FLAGS: DWORD = 0x8009_0009;
pub const NTE_BAD_PROVIDER: DWORD = 0x8009_0013;
pub const NTE_BAD_PROV_TYPE: DWORD = 0x8009_0014;
pub const NTE_PROV_TYPE_NOT_DEF: DWORD = 0x8009_0017;
pub const NTE_PROV_TYPE_ENTRY_BAD: DWORD = 0x8009_0018;
pub const NTE_KEYSET_NOT_DEF: DWORD = 0x8009_0019;
pub const NTE_PROV_TYPE_NO_MATCH: DWORD = 0x8009_001B;
pub const NTE_PROVIDER_DLL_FAIL: DWORD = 0x8009_001D;
pub const NTE_PROV_DLL_NOT_FOUND: DWORD = 0x8009_001E;
pub const NTE_FAIL: DWORD = 0x8009_0020;
// ntstatus.h cast macros (sec/lsa.rs, sec/misc.rs) — NTSTATUS-typed.
pub const STATUS_SOME_NOT_MAPPED: NTSTATUS = 0x0000_0107;
pub const RPC_NT_SERVER_UNAVAILABLE: NTSTATUS = 0xC002_0017u32 as i32;
// wine/crypt.h: #define MAXPROVTYPES 999 (not in allowlist-var) — DWORD compare.
pub const MAXPROVTYPES: DWORD = 999;
// wincrypt.h provider-param ids (PP_*) not bindgen'd (allowlist) (wine/crypt.rs).
pub const PP_CLIENT_HWND: DWORD = 1;
pub const PP_USE_HARDWARE_RNG: DWORD = 38;
// crt/fcntl.h _O_RDONLY (wine/crypt.rs file open flag).
pub const O_RDONLY: c_int = 0;
// winnls.h string-compare consts not in allowlist-var (wine/cred.rs).
pub const NORM_IGNORECASE: DWORD = 1;
pub const CSTR_EQUAL: c_int = 2;

// advapi32.h heap helpers: static inline heap_alloc/heap_free →
// HeapAlloc/HeapFree(GetProcessHeap(),0,..). Used crate-wide (wine/cred.rs).
// Takes `usize` (not SIZE_T, which is u32 on i386) to match the translation's
// `... as usize` length expressions; converted to SIZE_T at the HeapAlloc call.
#[inline]
pub unsafe fn heap_alloc(len: usize) -> PVOID {
    HeapAlloc(GetProcessHeap(), 0, len as SIZE_T)
}
#[inline]
pub unsafe fn heap_free(mem: PVOID) -> BOOL {
    HeapFree(GetProcessHeap(), 0, mem)
}
// winnt.h SID_IDENTIFIER_AUTHORITY initializers {0,0,0,0,0,N}. The C uses them as
// full brace-initializers of a SID_IDENTIFIER_AUTHORITY struct (e.g. the Wave-4
// WellKnownSids table in wine/security.rs: `IdentifierAuthority: SECURITY_NT_
// AUTHORITY`), so they MUST be the struct value, not the bare trailing byte. The
// Wave-1/2/3 sites that wanted just the byte read `.Value[5]`.
pub const SECURITY_NULL_SID_AUTHORITY: SID_IDENTIFIER_AUTHORITY =
    SID_IDENTIFIER_AUTHORITY { Value: [0, 0, 0, 0, 0, 0] };
pub const SECURITY_WORLD_SID_AUTHORITY: SID_IDENTIFIER_AUTHORITY =
    SID_IDENTIFIER_AUTHORITY { Value: [0, 0, 0, 0, 0, 1] };
pub const SECURITY_LOCAL_SID_AUTHORITY: SID_IDENTIFIER_AUTHORITY =
    SID_IDENTIFIER_AUTHORITY { Value: [0, 0, 0, 0, 0, 2] };
pub const SECURITY_CREATOR_SID_AUTHORITY: SID_IDENTIFIER_AUTHORITY =
    SID_IDENTIFIER_AUTHORITY { Value: [0, 0, 0, 0, 0, 3] };
pub const SECURITY_NT_AUTHORITY: SID_IDENTIFIER_AUTHORITY =
    SID_IDENTIFIER_AUTHORITY { Value: [0, 0, 0, 0, 0, 5] };
pub const SECURITY_MANDATORY_LABEL_AUTHORITY: SID_IDENTIFIER_AUTHORITY =
    SID_IDENTIFIER_AUTHORITY { Value: [0, 0, 0, 0, 0, 16] };

// wine/unicode.h: #define sprintfW _swprintf — link to the CRT _swprintf symbol
// (variadic, cdecl). Used by wine/crypt.rs to format a UUID container name.
extern "C" {
    #[link_name = "_swprintf"] pub fn sprintfW(buffer: *mut u16, format: *const u16, ...) -> c_int;
}

// UlongToPtr (basetsd.h inline cast)
#[inline]
pub fn UlongToPtr(ul: ULONG) -> PVOID {
    ul as usize as PVOID
}
// RtlCopyMemory is a macro alias for memcpy.
#[inline]
pub unsafe fn RtlCopyMemory(dst: PVOID, src: *const core::ffi::c_void, len: usize) {
    memcpy(dst, src, len);
}

// reg/reg.h FORCEINLINE HKCR-key tag helpers (no exported symbol → Rust inlines).
#[inline]
pub fn IsHKCRKey(hKey: HKEY) -> BOOL {
    (((hKey as ULONG_PTR) & 0x2) != 0) as BOOL
}
#[inline]
pub unsafe fn MakeHKCRKey(hKey: *mut HKEY) {
    *hKey = ((*hKey as ULONG_PTR) | 0x2) as HKEY;
}

// rtl FORCEINLINEs (ntddk/rtlfuncs.h) — no exported symbol → Rust inlines.
#[inline]
pub fn RtlUlongByteSwap(source: ULONG) -> ULONG {
    source.swap_bytes()
}
#[inline]
pub unsafe fn RtlInitEmptyAnsiString(
    AnsiString: PANSI_STRING,
    Buffer: PCHAR,
    BufferSize: USHORT,
) {
    (*AnsiString).Length = 0;
    (*AnsiString).MaximumLength = BufferSize;
    (*AnsiString).Buffer = Buffer;
}
#[inline]
pub unsafe fn RtlInitEmptyUnicodeString(
    UnicodeString: PUNICODE_STRING,
    Buffer: PWSTR,
    BufferSize: USHORT,
) {
    (*UnicodeString).Length = 0;
    (*UnicodeString).MaximumLength = BufferSize;
    (*UnicodeString).Buffer = Buffer;
}

// UUID is `typedef GUID UUID;` (rpcdce.h) — bindgen folds it into GUID.
pub type UUID = GUID;

// RPC_S_OK: bindgen types it u32, but RPC_STATUS is c_long (i32) → comparisons
// `status != RPC_S_OK` mismatch. Define with the correct RPC_STATUS type.
pub const RPC_S_OK: RPC_STATUS = 0;

// ---- evntrace.h types (excluded from bindgen — WMI header ordering) ----
// evntrace.c only uses these opaquely (OpenTrace returns INVALID_PROCESSTRACE_
// HANDLE; ProcessTrace returns an error) → opaque is faithful.
pub type TRACEHANDLE = u64;
pub type PTRACEHANDLE = *mut u64;
pub type PEVENT_TRACE_LOGFILEW = *mut core::ffi::c_void;
pub type PEVENT_TRACE_LOGFILEA = *mut core::ffi::c_void;
pub const INVALID_PROCESSTRACE_HANDLE: TRACEHANDLE = !0u64;

// ---- enum-variant bare-name aliases (bindgen prefixes them) ----
pub const TRUSTEE_IS_SID: TRUSTEE_FORM = advapi32_sys::_TRUSTEE_FORM_TRUSTEE_IS_SID;
pub const TRUSTEE_IS_NAME: TRUSTEE_FORM = advapi32_sys::_TRUSTEE_FORM_TRUSTEE_IS_NAME;
pub const TRUSTEE_BAD_FORM: TRUSTEE_FORM = advapi32_sys::_TRUSTEE_FORM_TRUSTEE_BAD_FORM;
pub const TRUSTEE_IS_OBJECTS_AND_SID: TRUSTEE_FORM =
    advapi32_sys::_TRUSTEE_FORM_TRUSTEE_IS_OBJECTS_AND_SID;
pub const TRUSTEE_IS_OBJECTS_AND_NAME: TRUSTEE_FORM =
    advapi32_sys::_TRUSTEE_FORM_TRUSTEE_IS_OBJECTS_AND_NAME;
pub const NO_MULTIPLE_TRUSTEE: MULTIPLE_TRUSTEE_OPERATION =
    advapi32_sys::_MULTIPLE_TRUSTEE_OPERATION_NO_MULTIPLE_TRUSTEE;
pub const TRUSTEE_IS_IMPERSONATE: MULTIPLE_TRUSTEE_OPERATION =
    advapi32_sys::_MULTIPLE_TRUSTEE_OPERATION_TRUSTEE_IS_IMPERSONATE;
pub const TRUSTEE_IS_UNKNOWN: TRUSTEE_TYPE = advapi32_sys::_TRUSTEE_TYPE_TRUSTEE_IS_UNKNOWN;

// ---- HKEY predefined-key consts (cast macros — not bindgen-evaluable) ----
// winreg.h: `#define HKEY_CLASSES_ROOT ((HKEY)(ULONG_PTR)((LONG)0x80000000))` — the LONG cast
// makes them SIGN-extended on 64-bit (0xFFFFFFFF80000000), which is what every C and mingw
// caller passes. These were `as usize` (zero-extended) until 2026-09: exact comparisons
// against them never matched a real caller on amd64/arm64, so RegOverridePredefKey refused
// every key with ERROR_INVALID_HANDLE. Compare with `SamePredefKey`, not `==` — several Rust
// crates still define their own zero-extended copies and pass those in.
pub const HKEY_CLASSES_ROOT: HKEY = 0x8000_0000u32 as i32 as isize as HKEY;
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as i32 as isize as HKEY;
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;
pub const HKEY_USERS: HKEY = 0x8000_0003u32 as i32 as isize as HKEY;

// UlongToHandle / UlongToPtr (basetsd.h inline casts)
#[inline]
pub fn UlongToHandle(h: ULONG) -> HANDLE {
    h as usize as HANDLE
}

// ---- universal type aliases the translations use ----
pub type VOID = core::ffi::c_void;
pub type size_t = usize;
pub use core::ffi::{c_int, c_long, c_uint, c_ulong, c_void};

// ---- universal NT / Win32 constants not in the NT bindgen ----
pub const NULL: PVOID = core::ptr::null_mut();
pub const UNICODE_NULL: u16 = 0;
pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;

// ---- min/max helpers (c2rust emits bare `min(a,b)` / `max(a,b)`) ----
#[inline]
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b { a } else { b }
}
#[inline]
pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

// ---- NT_SUCCESS macro (C macro, not bindgen'd) → const fn ----
#[inline]
pub fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

// ---- RtlGetProcessHeap: ReactOS macro `NtCurrentPeb()->ProcessHeap` ----
#[inline]
pub unsafe fn RtlGetProcessHeap() -> PVOID {
    (*NtCurrentPeb()).ProcessHeap
}

// ---- RtlZeroMemory: macro alias for memset(dst, 0, len) ----
#[inline]
pub unsafe fn RtlZeroMemory(dst: PVOID, len: usize) {
    memset(dst, 0, len);
}

// ---- InterlockedCompareExchangePointer: winnt.h is an inline intrinsic
// (_InlineInterlockedCompareExchangePointer, no exported symbol) → implement with
// a core atomic CAS. Returns the *initial* value (C semantics), regardless of
// whether the exchange happened. ----
#[inline]
pub unsafe fn InterlockedCompareExchangePointer(
    dest: *mut PVOID,
    exchange: PVOID,
    comparand: PVOID,
) -> PVOID {
    let a = &*(dest as *const core::sync::atomic::AtomicPtr<c_void>);
    match a.compare_exchange(
        comparand,
        exchange,
        core::sync::atomic::Ordering::SeqCst,
        core::sync::atomic::Ordering::SeqCst,
    ) {
        Ok(v) => v,
        Err(v) => v,
    }
}

// ---- Interlocked* intrinsics (winnt.h inline, no exported symbol) ----
#[inline]
pub unsafe fn InterlockedIncrement(addend: *mut LONG) -> LONG {
    (&*(addend as *const core::sync::atomic::AtomicI32))
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst)
        + 1
}
#[inline]
pub unsafe fn InterlockedDecrement(addend: *mut LONG) -> LONG {
    (&*(addend as *const core::sync::atomic::AtomicI32))
        .fetch_sub(1, core::sync::atomic::Ordering::SeqCst)
        - 1
}
#[inline]
pub unsafe fn InterlockedExchangePointer(target: *mut PVOID, value: PVOID) -> PVOID {
    (&*(target as *const core::sync::atomic::AtomicPtr<c_void>))
        .swap(value, core::sync::atomic::Ordering::SeqCst)
}

// ---- HandleToUlong (basetsd.h: ((ULONG)(ULONG_PTR)(h))) ----
#[inline]
pub fn HandleToUlong(h: HANDLE) -> ULONG {
    h as usize as ULONG
}

// ---- STATUS_LOCAL_USER_SESSION_KEY (ntstatus.h cast macro — not bindgen'd) ----
pub const STATUS_LOCAL_USER_SESSION_KEY: NTSTATUS = 0x4000_0006u32 as i32;

// ---- IOCTL_KSEC_* (ksecioctl.h CTL_CODE macros — not bindgen-evaluable) ----
// FILE_DEVICE_KSEC=0x39, METHOD_OUT_DIRECT=2 (values confirmed in header comments).
pub const IOCTL_KSEC_ENCRYPT_SAME_PROCESS: ULONG = 0x0039_000E;
pub const IOCTL_KSEC_DECRYPT_SAME_PROCESS: ULONG = 0x0039_0012;
pub const IOCTL_KSEC_ENCRYPT_CROSS_PROCESS: ULONG = 0x0039_0016;
pub const IOCTL_KSEC_DECRYPT_CROSS_PROCESS: ULONG = 0x0039_001A;
pub const IOCTL_KSEC_ENCRYPT_SAME_LOGON: ULONG = 0x0039_001E;
pub const IOCTL_KSEC_DECRYPT_SAME_LOGON: ULONG = 0x0039_0022;

// ---- RC4_CONTEXT (cryptlib sdk/lib/cryptlib/rc4.h — not in advapi32-sys) ----
#[repr(C)]
pub struct RC4_CONTEXT {
    pub state: [u8; 256],
    pub x: u8,
    pub y: u8,
}

// ---- CRT functions (cdecl → extern "C"); resolve against ntdll/CRT at link ----
extern "C" {
    pub fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memmove(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dst: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn strlen(s: *const i8) -> usize;
    pub fn wcslen(s: *const u16) -> usize;
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16;
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> c_int;
    pub fn swprintf(buffer: *mut u16, format: *const u16, ...) -> c_int;
}

// ---- cryptlib rc4 (sdk/lib/cryptlib/rc4.c — plain C/cdecl → extern "C") ----
extern "C" {
    pub fn rc4_init(a4i: *mut RC4_CONTEXT, key: *const u8, keyLen: c_uint);
    pub fn rc4_crypt(a4i: *mut RC4_CONTEXT, inout: *mut u8, length: c_uint);
}

// ============================ Wave 4 ============================
// SCM / service / wine-security / reg types curated from advapi32-sys (all
// present in bindgen; none collide with ntdll_sys::nt). service/scm.rs +
// wine/security.rs + reg/reg.rs.
pub use advapi32_sys::{
    // winsvc / svcctl SCM handle + RPC types
    SC_HANDLE, SC_LOCK, SC_RPC_HANDLE, LPSC_RPC_HANDLE, SC_RPC_LOCK, LPSC_RPC_LOCK,
    SC_RPC_CONFIG_INFOW, SC_RPC_CONFIG_INFOA, SC_ENUM_TYPE, SC_STATUS_TYPE, LPSC_ACTION,
    SVCCTL_HANDLEW, SVCCTL_HANDLEA,
    // enum-service status (W/A + LP)
    ENUM_SERVICE_STATUSW, ENUM_SERVICE_STATUSA, LPENUM_SERVICE_STATUSW, LPENUM_SERVICE_STATUSA,
    ENUM_SERVICE_STATUS_PROCESSW, ENUM_SERVICE_STATUS_PROCESSA,
    LPENUM_SERVICE_STATUS_PROCESSW, LPENUM_SERVICE_STATUS_PROCESSA,
    // query-service config / lock-status (W/A + LP)
    QUERY_SERVICE_CONFIGW, QUERY_SERVICE_CONFIGA, LPQUERY_SERVICE_CONFIGW, LPQUERY_SERVICE_CONFIGA,
    QUERY_SERVICE_LOCK_STATUSW, QUERY_SERVICE_LOCK_STATUSA,
    LPQUERY_SERVICE_LOCK_STATUSW, LPQUERY_SERVICE_LOCK_STATUSA,
    // service description / failure-actions (W/A + LP)
    SERVICE_DESCRIPTIONW, SERVICE_DESCRIPTIONA, LPSERVICE_DESCRIPTIONW, LPSERVICE_DESCRIPTIONA,
    SERVICE_FAILURE_ACTIONSW, SERVICE_FAILURE_ACTIONSA,
    LPSERVICE_FAILURE_ACTIONSW, LPSERVICE_FAILURE_ACTIONSA,
    SERVICE_STATUS_PROCESS,
    // svcctl RPC marshalling (STRING_PTRS, TAG_INFO, seclogon SECL)
    LPSTRING_PTRSW, LPSTRING_PTRSA,
    TAG_INFO_LEVEL, PTAG_INFO_NAME_FROM_TAG,
    PTAG_INFO_NAME_FROM_TAG_IN_PARAMS, PTAG_INFO_NAME_FROM_TAG_OUT_PARAMS,
    SECL_REQUEST, SECL_RESPONSE,
    // aclapi ACTRL access/audit (wine/security.rs)
    PACTRL_ACCESSW, PACTRL_ACCESSA, PACTRL_AUDITW, PACTRL_AUDITA,
    // winsvc SC_MANAGER access masks (present in bindgen-var)
    SC_MANAGER_CONNECT, SC_MANAGER_ENUMERATE_SERVICE,
};

// Enum-variant bare-name consts (bindgen prefixes them) — scm.rs uses the bare
// names.
pub const SC_STATUS_PROCESS_INFO: SC_STATUS_TYPE =
    advapi32_sys::_SC_STATUS_TYPE_SC_STATUS_PROCESS_INFO;
pub const TagInfoLevelNameFromTag: TAG_INFO_LEVEL =
    advapi32_sys::_TAG_INFO_LEVEL_TagInfoLevelNameFromTag;

// HKEY predefined-key cast-macro consts (winreg.h) — Wave 4 adds the last two.
pub const HKEY_PERFORMANCE_DATA: HKEY = 0x8000_0004u32 as i32 as isize as HKEY;
pub const HKEY_CURRENT_CONFIG: HKEY = 0x8000_0005u32 as i32 as isize as HKEY;

// winsvc.h SCM event/db names — W-API wide strings (services.h SCM_START_EVENT is
// L"..."; bindgen folded it narrow, so hand-define wide to match OpenEventW).
pub const SCM_START_EVENT: PCWSTR = crate::wide!("SvcctrlStartEvent_A3752DX").as_ptr();
pub const SERVICES_ACTIVE_DATABASEW: PCWSTR = crate::wide!("ServicesActive").as_ptr();

// winnt.h access-mask consts (FILE_GENERIC_* / FILE_ALL_ACCESS) — DWORD.
pub const FILE_GENERIC_READ: DWORD = 0x0012_0089;
pub const FILE_GENERIC_WRITE: DWORD = 0x0012_0116;
pub const FILE_GENERIC_EXECUTE: DWORD = 0x0012_00A0;
pub const FILE_ALL_ACCESS: DWORD = 0x001F_01FF;

// winioctl.h: FSCTL_PIPE_IMPERSONATE = CTL_CODE(FILE_DEVICE_NAMED_PIPE=0x11, 7,
//   METHOD_BUFFERED=0, FILE_ANY_ACCESS=0) = (0x11<<16)|(0<<14)|(7<<2)|0 = 0x11001C.
pub const FSCTL_PIPE_IMPERSONATE: DWORD = 0x0011_001C;

// winuser.h: RT_STRING = MAKEINTRESOURCE(6) — used `RT_STRING as LPWSTR`.
pub const RT_STRING: ULONG_PTR = 6;

// sddl.h SDDL_REVISION (numeric) + the 2-letter SDDL right strings (TEXT("..")
// → wide). The ACEFLAG table in wine/security.rs uses `.as_ptr()` on these.
pub const SDDL_REVISION_1: DWORD = 1;
pub const SDDL_REVISION: DWORD = 1;
pub const SDDL_PROTECTED: &[u16] = crate::wide!("P");
pub const SDDL_AUTO_INHERIT_REQ: &[u16] = crate::wide!("AR");
pub const SDDL_AUTO_INHERITED: &[u16] = crate::wide!("AI");
pub const SDDL_READ_PROPERTY: &[u16] = crate::wide!("RP");
pub const SDDL_WRITE_PROPERTY: &[u16] = crate::wide!("WP");
pub const SDDL_CREATE_CHILD: &[u16] = crate::wide!("CC");
pub const SDDL_DELETE_CHILD: &[u16] = crate::wide!("DC");
pub const SDDL_LIST_CHILDREN: &[u16] = crate::wide!("LC");
pub const SDDL_SELF_WRITE: &[u16] = crate::wide!("SW");
pub const SDDL_LIST_OBJECT: &[u16] = crate::wide!("LO");
pub const SDDL_DELETE_TREE: &[u16] = crate::wide!("DT");
pub const SDDL_CONTROL_ACCESS: &[u16] = crate::wide!("CR");
pub const SDDL_READ_CONTROL: &[u16] = crate::wide!("RC");
pub const SDDL_WRITE_DAC: &[u16] = crate::wide!("WD");
pub const SDDL_WRITE_OWNER: &[u16] = crate::wide!("WO");
pub const SDDL_STANDARD_DELETE: &[u16] = crate::wide!("SD");
pub const SDDL_GENERIC_ALL: &[u16] = crate::wide!("GA");
pub const SDDL_GENERIC_READ: &[u16] = crate::wide!("GR");
pub const SDDL_GENERIC_WRITE: &[u16] = crate::wide!("GW");
pub const SDDL_GENERIC_EXECUTE: &[u16] = crate::wide!("GX");
pub const SDDL_FILE_ALL: &[u16] = crate::wide!("FA");
pub const SDDL_FILE_READ: &[u16] = crate::wide!("FR");
pub const SDDL_FILE_WRITE: &[u16] = crate::wide!("FW");
pub const SDDL_FILE_EXECUTE: &[u16] = crate::wide!("FX");
pub const SDDL_KEY_ALL: &[u16] = crate::wide!("KA");
pub const SDDL_KEY_READ: &[u16] = crate::wide!("KR");
pub const SDDL_KEY_WRITE: &[u16] = crate::wide!("KW");
pub const SDDL_KEY_EXECUTE: &[u16] = crate::wide!("KX");

// ---- Win32 macros (winbase.h / minwindef.h / winuser.h) → inline fns ----
#[inline]
pub unsafe fn CopyMemory(dst: PVOID, src: *const c_void, len: usize) {
    memcpy(dst, src, len);
}
#[inline]
pub unsafe fn MoveMemory(dst: PVOID, src: *const c_void, len: usize) {
    memmove(dst, src, len);
}
#[inline]
pub unsafe fn ZeroMemory(dst: PVOID, len: usize) {
    memset(dst, 0, len);
}
// SecureZeroMemory: zeroes and is not elidable. Byte-volatile loop = faithful.
#[inline]
pub unsafe fn SecureZeroMemory(ptr: PVOID, cnt: SIZE_T) -> PVOID {
    let mut p = ptr as *mut u8;
    let mut n = cnt as usize;
    while n != 0 {
        core::ptr::write_volatile(p, 0);
        p = p.add(1);
        n -= 1;
    }
    ptr
}
// InitializeObjectAttributes (ntdef.h field-init macro).
#[inline]
pub unsafe fn InitializeObjectAttributes(
    p: *mut OBJECT_ATTRIBUTES,
    n: *mut UNICODE_STRING,
    a: ULONG,
    r: HANDLE,
    s: PVOID,
) {
    (*p).Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
    (*p).RootDirectory = r as _;
    (*p).Attributes = a;
    (*p).ObjectName = n as _;
    (*p).SecurityDescriptor = s as _;
    (*p).SecurityQualityOfService = core::ptr::null_mut();
}
// MAKEWORD/MAKELONG/LOWORD/HIWORD are minwindef.h macros applied to mixed integer
// widths (BYTE/WORD/DWORD) in the C — generic over Into<u32> so each call site's
// native arg type (u8/u16/u32) is accepted without a cast.
#[inline]
pub fn MAKEWORD<A: Into<u32>, B: Into<u32>>(a: A, b: B) -> WORD {
    ((a.into() & 0xff) | ((b.into() & 0xff) << 8)) as WORD
}
#[inline]
pub fn MAKELONG<A: Into<u32>, B: Into<u32>>(a: A, b: B) -> LONG {
    ((a.into() & 0xffff) | ((b.into() & 0xffff) << 16)) as LONG
}
#[inline]
pub fn LOWORD<L: Into<u32>>(l: L) -> WORD {
    (l.into() & 0xffff) as WORD
}
#[inline]
pub fn HIWORD<L: Into<u32>>(l: L) -> WORD {
    ((l.into() >> 16) & 0xffff) as WORD
}
#[inline]
pub fn MAKEINTRESOURCEW(i: WORD) -> LPWSTR {
    i as ULONG_PTR as LPWSTR
}

// Marks a read-only `static` whose value holds raw pointers (not auto-Sync) as
// shareable. Sound because the data is immutable constant tables (C `static
// const`). Used by the LPCWSTR lookup tables in wine/security.rs.
#[repr(transparent)]
pub struct SyncConst<T>(pub T);
unsafe impl<T> Sync for SyncConst<T> {}

// ---- reg.c file-local predefined-key macros → inline fns (reg/reg.rs) ----
#[inline]
pub unsafe fn ClosePredefKey(handle: HANDLE) {
    if (handle as ULONG_PTR) & 0x1 != 0 {
        NtClose(handle);
    }
}
/// `a` is the predefined key `predef`, however the caller extended it to pointer width.
#[inline]
pub fn SamePredefKey(a: HKEY, predef: HKEY) -> bool {
    IsPredefKey(a) != 0 && (a as ULONG_PTR as u32) == (predef as ULONG_PTR as u32)
}
#[inline]
pub fn IsPredefKey(hkey: HKEY) -> BOOL {
    (((hkey as ULONG_PTR) & 0xF000_0000) == 0x8000_0000) as BOOL
}
#[inline]
pub fn GetPredefKeyIndex(hkey: HKEY) -> ULONG_PTR {
    (hkey as ULONG_PTR) & 0x0FFF_FFFF
}

// ---- wine/unicode.h string macros → inline fns / CRT externs ----
// wine/unicode.h: strlenW returns `unsigned int`, strncmpW takes `unsigned int n`
// (not size_t) — match those so the DWORD call sites need no cast.
#[inline]
pub unsafe fn strlenW(s: *const u16) -> c_uint {
    wcslen(s) as c_uint
}
#[inline]
pub unsafe fn strncmpW(s1: *const u16, s2: *const u16, n: c_uint) -> c_int {
    wcsncmp(s1, s2, n as usize)
}
#[inline]
pub unsafe fn strtoulW(s: *const u16, end: *mut *mut u16, base: c_int) -> c_ulong {
    wcstoul(s, end, base)
}
#[inline]
pub unsafe fn atoiW(s: *const u16) -> c_int {
    _wtoi(s)
}
extern "C" {
    pub fn wcscmp(s1: *const u16, s2: *const u16) -> c_int;
    pub fn wcsncmp(s1: *const u16, s2: *const u16, n: usize) -> c_int;
    pub fn wcsrchr(s: *const u16, c: u16) -> *mut u16;
    pub fn wcstoul(s: *const u16, end: *mut *mut u16, base: c_int) -> c_ulong;
    pub fn _wtoi(s: *const u16) -> c_int;
    // wine/debug.h variadic debug-string formatter (cdecl) — only reached from
    // TRACE no-ops, but the debugstr_* bodies still compile/call it.
    pub fn wine_dbg_sprintf(format: *const core::ffi::c_char, ...) -> *const core::ffi::c_char;
}

// ---- advapi32's own exported APIs called cross-file but not yet translated ----
// These resolve against the still-C cohab TUs at link. Remove an entry when its
// owning .c flips to Rust (the #[no_mangle] def then provides the symbol, and the
// extern decl would shadow-conflict). See externs module.
pub use crate::externs::*;
