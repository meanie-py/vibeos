//! services.h private types + svcctl RPC lock-status structs + the internal
//! Scm* database-lock helpers (defined in database.c — stay C) and the one CRT
//! function lock.c needs. Shared substrate for the services.exe (SCM) flip.

use winsrv_sys::winsrv::*;

/// `wstr!("Foo")` → a `&'static [u16]` NUL-terminated UTF-16 literal (ASCII
/// only — sufficient for the SCM's registry path/value names). Avoids the
/// align-1 `b"...".as_ptr() as *mut u16` misalignment hazard and the lack of a
/// wide-string literal in core. `.as_ptr()` on the result is a stable pointer
/// (the inner `const` is promoted to 'static).
#[macro_export]
macro_rules! wstr {
    ($s:literal) => {{
        const BYTES: &[u8] = $s.as_bytes();
        const N: usize = BYTES.len() + 1;
        const ARR: [u16; N] = {
            let mut a = [0u16; N];
            let mut i = 0;
            while i < BYTES.len() {
                a[i] = BYTES[i] as u16;
                i += 1;
            }
            a
        };
        &ARR as &[u16]
    }};
}

// Predefined registry root handles (winreg.h) — not in winsrv-sys bindgen.
pub const HKEY_LOCAL_MACHINE: HKEY = 0x80000002u32 as usize as HKEY;

// winsvc.h service-object access rights (not in winsrv-sys bindgen). The
// SECURITY_*_RID / DOMAIN_ALIAS_RID_ADMINS / READ_CONTROL / GENERIC_ALL /
// ACL_REVISION etc. DO come from winsrv-sys — only these SERVICE_* masks are
// missing.
pub const SERVICE_QUERY_CONFIG: u32 = 0x0001;
pub const SERVICE_CHANGE_CONFIG: u32 = 0x0002;
pub const SERVICE_QUERY_STATUS: u32 = 0x0004;
pub const SERVICE_ENUMERATE_DEPENDENTS: u32 = 0x0008;
pub const SERVICE_START: u32 = 0x0010;
pub const SERVICE_STOP: u32 = 0x0020;
pub const SERVICE_PAUSE_CONTINUE: u32 = 0x0040;
pub const SERVICE_INTERROGATE: u32 = 0x0080;
pub const SERVICE_USER_DEFINED_CONTROL: u32 = 0x0100;
pub const SERVICE_ALL_ACCESS: u32 = 0x000F01FF; // STANDARD_RIGHTS_REQUIRED | 0x1FF

// services.h
pub const LOCK_TAG: u32 = 0x4C697041; // 'ApiL'

/// services.h `_START_LOCK` — the SCM's unique user start-serialization lock.
#[repr(C)]
pub struct START_LOCK {
    pub Tag: u32,            // DWORD — must be LOCK_TAG
    pub TimeWhenLocked: u32, // DWORD — seconds since 1970
    pub LockOwnerSid: PSID,  // NULL if the SCM acquired the lock
}
pub type PSTART_LOCK = *mut START_LOCK;

// winsvc.h flat RPC structs (not in winsrv-sys bindgen).
#[repr(C)]
pub struct QUERY_SERVICE_LOCK_STATUSW {
    pub fIsLocked: u32,
    pub lpLockOwner: LPWSTR,
    pub dwLockDuration: u32,
}
pub type LPQUERY_SERVICE_LOCK_STATUSW = *mut QUERY_SERVICE_LOCK_STATUSW;

#[repr(C)]
pub struct QUERY_SERVICE_LOCK_STATUSA {
    pub fIsLocked: u32,
    pub lpLockOwner: LPSTR,
    pub dwLockDuration: u32,
}
pub type LPQUERY_SERVICE_LOCK_STATUSA = *mut QUERY_SERVICE_LOCK_STATUSA;

// svcctl.idl: `typedef [context_handle] PVOID SC_RPC_LOCK;` and
// `typedef SC_RPC_LOCK* LPSC_RPC_LOCK;`.
pub type SC_RPC_LOCK = PVOID;
pub type LPSC_RPC_LOCK = *mut SC_RPC_LOCK;

// Win32 error codes (ERROR_SUCCESS / ERROR_NOT_ENOUGH_MEMORY /
// ERROR_INVALID_SERVICE_LOCK / ERROR_SERVICE_DATABASE_LOCKED) come from
// winsrv-sys — don't redefine them locally (avoids shadowing the canonical
// bindgen values and the E0659 ambiguity in modules that glob both).

// database.c internal helpers (no WINAPI in services.h → cdecl → `extern "C"`,
// i386 symbol `_ScmLockDatabaseExclusive` etc.; they stay C for now).
extern "C" {
    pub fn ScmLockDatabaseExclusive() -> BOOL;
    pub fn ScmLockDatabaseShared() -> BOOL;
    pub fn ScmUnlockDatabase();
}

// msvcrt time(): time_t is __time64_t on ReactOS. lock.c only uses the low 32
// bits (`(DWORD)time(NULL)`), so reading it as i64 and truncating is ABI-safe
// regardless of the actual time_t width on either arch.
extern "C" {
    pub fn time(timer: *mut core::ffi::c_void) -> i64;
}
