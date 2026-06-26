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
};

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
pub const HKEY_CLASSES_ROOT: HKEY = 0x8000_0000u32 as usize as HKEY;
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as usize as HKEY;
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as usize as HKEY;
pub const HKEY_USERS: HKEY = 0x8000_0003u32 as usize as HKEY;

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

// ---- advapi32's own exported APIs called cross-file but not yet translated ----
// These resolve against the still-C cohab TUs at link. Remove an entry when its
// owning .c flips to Rust (the #[no_mangle] def then provides the symbol, and the
// extern decl would shadow-conflict). See externs module.
pub use crate::externs::*;
