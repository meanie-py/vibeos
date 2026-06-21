//! Crate-wide prelude of the universal symbols the per-file DeepSeek
//! translations expect in scope: the NT base substrate (ntdll_sys::nt), the
//! LSAPR_*/RPC_* marshalled types (lsasrv-sys), CRT + Wine wide-string helpers,
//! the lsasrv consts (resources/STATUS/SECURITY_ACCESS), MIDL allocators, and
//! no-op Wine debug shims/macros. Each translated file adds
//! `use crate::prelude::*;`. Mirrors kernel32-rust/src/prelude.rs.
#![allow(non_snake_case, non_upper_case_globals, dead_code)]

pub use ntdll_sys::nt::*;
// lsasrv marshalled + policy info types absent from ntdll_sys::nt. Curated
// (not a glob) so the base types lsasrv-sys re-bindgen'd (UNICODE_STRING/GUID/
// LUID/…) don't collide with ntdll_sys::nt's.
pub use lsasrv_sys::{
    RPC_UNICODE_STRING, PRPC_UNICODE_STRING, _RPC_UNICODE_STRING,
    LSAPR_ACL, PLSAPR_ACL,
    LSAPR_HANDLE, LSA_HANDLE,
    LSAPR_OBJECT_ATTRIBUTES, PLSAPR_OBJECT_ATTRIBUTES,
    LSAPR_SECURITY_DESCRIPTOR, PLSAPR_SECURITY_DESCRIPTOR,
    LSAPR_SR_SECURITY_DESCRIPTOR, PLSAPR_SR_SECURITY_DESCRIPTOR,
    LSAPR_POLICY_INFORMATION, PLSAPR_POLICY_INFORMATION,
    LSAPR_POLICY_PRIVILEGE_DEF, PLSAPR_POLICY_PRIVILEGE_DEF,
    LSAPR_PRIVILEGE_ENUM_BUFFER, PLSAPR_PRIVILEGE_ENUM_BUFFER,
    LSAPR_POLICY_AUDIT_EVENTS_INFO, PLSAPR_POLICY_AUDIT_EVENTS_INFO,
    LSAPR_POLICY_PRIMARY_DOM_INFO, LSAPR_POLICY_ACCOUNT_DOM_INFO,
    LSAPR_SID_INFORMATION, LSAPR_SID_ENUM_BUFFER,
    LSAPR_REFERENCED_DOMAIN_LIST, PLSAPR_REFERENCED_DOMAIN_LIST,
    LSAPR_TRUST_INFORMATION,
    POLICY_INFORMATION_CLASS,
    DSROLE_PRIMARY_DOMAIN_INFO_LEVEL,
    PDSROLER_PRIMARY_DOMAIN_INFORMATION,
};

pub use crate::consts::*;

// ---- universal type aliases the c2rust output uses ----
pub type VOID = core::ffi::c_void;
pub type size_t = usize;
pub use core::ffi::{c_int, c_long};

// ---- common NT/Win32 constants the translations bare-reference ----
pub const UNICODE_NULL: u16 = 0;
pub const NULL: PVOID = core::ptr::null_mut();
pub const MAXULONG: ULONG = 0xFFFF_FFFF;

#[inline]
pub fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

// ---- Wine debug channel (wine/debug.h) — the c2rust file preambles construct a
// static __wine_debug_channel; flags are release-inert (TRACE/WARN are no-ops). ----
#[repr(C)]
pub struct __wine_debug_channel {
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub const WINE_DBG_DEFAULT_FLAGS: ::core::ffi::c_uchar = 3;

// ---- MIDL allocators (lsasrv.c; __RPC_USER = stdcall; uppercase real symbols) ----
extern "system" {
    pub fn MIDL_user_allocate(len: usize) -> *mut VOID;
    pub fn MIDL_user_free(ptr: *mut VOID);
    // kernel32 imports (WINAPI = stdcall).
    pub fn GetModuleHandleW(lpModuleName: *const u16) -> HMODULE;
    pub fn FindResourceW(hModule: HMODULE, lpName: *const u16, lpType: *const u16) -> HRSRC;
    pub fn LoadResource(hModule: HMODULE, hResInfo: HRSRC) -> HGLOBAL;
    pub fn LockResource(hResData: HGLOBAL) -> *mut VOID;
    pub fn SizeofResource(hModule: HMODULE, hResInfo: HRSRC) -> DWORD;
}

// Resource handle aliases (winbase) — kernel32 resource API handle types.
pub type HRSRC = HANDLE;
pub type HGLOBAL = HANDLE;

// ---- CRT functions (cdecl → extern "C"); resolve against ntdll/CRT at link ----
extern "C" {
    pub fn memcpy(d: *mut VOID, s: *const VOID, n: usize) -> *mut VOID;
    pub fn memmove(d: *mut VOID, s: *const VOID, n: usize) -> *mut VOID;
    pub fn memset(d: *mut VOID, c: c_int, n: usize) -> *mut VOID;
    pub fn memcmp(a: *const VOID, b: *const VOID, n: usize) -> c_int;
    pub fn wcslen(s: *const u16) -> usize;
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16;
    pub fn wcsrchr(s: *const u16, c: u16) -> *mut u16;
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int;
    pub fn wcscpy(d: *mut u16, s: *const u16) -> *mut u16;
    pub fn wcscat(d: *mut u16, s: *const u16) -> *mut u16;
    pub fn wcsncpy(d: *mut u16, s: *const u16, n: usize) -> *mut u16;
    pub fn _wcsicmp(a: *const u16, b: *const u16) -> c_int;
    pub fn _wcsnicmp(a: *const u16, b: *const u16, n: usize) -> c_int;
    pub fn wcstoul(s: *const u16, end: *mut *mut u16, base: c_int) -> ULONG;
}


// RtlGetProcessHeap() is a macro (NtCurrentPeb()->ProcessHeap) in rtlfuncs.h,
// not an export — implement in-crate so call sites bind directly (mirrors
// kernel32-rust/k32_externs).
#[inline]
pub unsafe fn RtlGetProcessHeap() -> PVOID {
    (*NtCurrentPeb()).ProcessHeap
}

// ---- STUB / UNIMPLEMENTED markers (ReactOS `STUB;` statements → no-op) ----
pub const STUB: () = ();
pub const UNIMPLEMENTED: () = ();

// DUMMY_PSEH: SEH filters/handlers are dead code.
#[inline]
pub fn _SEH2_GetExceptionCode() -> LONG {
    0
}

// ---- no-op Wine debug shims (NDEBUG: TRACE/WARN/ERR/FIXME compile out) ----
#[inline]
pub fn debugstr_w(_s: *const u16) -> *const i8 {
    core::ptr::null()
}
#[inline]
pub fn debugstr_a(_s: *const i8) -> *const i8 {
    core::ptr::null()
}
