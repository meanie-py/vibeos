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
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;

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

// winsvc.h — service current-state codes (Status.dwCurrentState).
pub const SERVICE_STOPPED: u32 = 0x00000001;
pub const SERVICE_START_PENDING: u32 = 0x00000002;
pub const SERVICE_STOP_PENDING: u32 = 0x00000003;
pub const SERVICE_RUNNING: u32 = 0x00000004;
pub const SERVICE_CONTINUE_PENDING: u32 = 0x00000005;
pub const SERVICE_PAUSE_PENDING: u32 = 0x00000006;
pub const SERVICE_PAUSED: u32 = 0x00000007;

// winsvc.h — controls accepted (Status.dwControlsAccepted).
pub const SERVICE_ACCEPT_STOP: u32 = 0x00000001;

// winsvc.h — control codes passed to ScmControlDriver / handlers.
pub const SERVICE_CONTROL_STOP: u32 = 0x00000001;
pub const SERVICE_CONTROL_PAUSE: u32 = 0x00000002;
pub const SERVICE_CONTROL_CONTINUE: u32 = 0x00000003;
pub const SERVICE_CONTROL_INTERROGATE: u32 = 0x00000004;
pub const SERVICE_CONTROL_SHUTDOWN: u32 = 0x00000005;
pub const SERVICE_CONTROL_PARAMCHANGE: u32 = 0x00000006;
pub const SERVICE_CONTROL_NETBINDADD: u32 = 0x00000007;
pub const SERVICE_CONTROL_NETBINDREMOVE: u32 = 0x00000008;
pub const SERVICE_CONTROL_NETBINDENABLE: u32 = 0x00000009;
pub const SERVICE_CONTROL_NETBINDDISABLE: u32 = 0x0000000A;
pub const SERVICE_CONTROL_DEVICEEVENT: u32 = 0x0000000B;
pub const SERVICE_CONTROL_HARDWAREPROFILECHANGE: u32 = 0x0000000C;
pub const SERVICE_CONTROL_POWEREVENT: u32 = 0x0000000D;
pub const SERVICE_CONTROL_SESSIONCHANGE: u32 = 0x0000000E;

// services.h
pub const LOCK_TAG: u32 = 0x4C697041; // 'ApiL'

// services.h / winsvc_undoc.h — services-private types shared across SCM modules
// (database.c + rpcserver.c). SERVICE_STATUS_HANDLE is an opaque pointer.
pub type SERVICE_STATUS_HANDLE = *mut core::ffi::c_void;
/// bindgen-style alias the translated code uses in offset_of!(_SERVICE, ...).
pub type _SERVICE = SERVICE;

// services.h `_SCM_CONTROL_PACKET` / `_SCM_REPLY_PACKET` — named-pipe IPC packets.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SCM_CONTROL_PACKET {
    pub dwSize: DWORD,
    pub dwControl: DWORD,
    pub dwArgumentsCount: DWORD,
    pub dwServiceTag: DWORD,
    pub hServiceStatus: SERVICE_STATUS_HANDLE,
    pub dwServiceNameOffset: DWORD,
    pub dwArgumentsOffset: DWORD,
}
pub type PSCM_CONTROL_PACKET = *mut SCM_CONTROL_PACKET;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SCM_REPLY_PACKET {
    pub dwError: DWORD,
}
pub type PSCM_REPLY_PACKET = *mut SCM_REPLY_PACKET;

// services.h — internal control codes for the start command.
pub const SERVICE_CONTROL_START_SHARE: u32 = 80;
pub const SERVICE_CONTROL_START_OWN: u32 = 81;

// winsvc_undoc.h — I_ScGetCurrentGroupStateW / tag→name query.
pub const TagTypeService: u32 = 1;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TAG_INFO_NAME_FROM_TAG_IN_PARAMS {
    pub dwPid: DWORD,
    pub dwTag: DWORD,
}
pub type PTAG_INFO_NAME_FROM_TAG_IN_PARAMS = *mut TAG_INFO_NAME_FROM_TAG_IN_PARAMS;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TAG_INFO_NAME_FROM_TAG_OUT_PARAMS {
    pub TagType: u32,
    pub pszName: LPWSTR,
}
pub type PTAG_INFO_NAME_FROM_TAG_OUT_PARAMS = *mut TAG_INFO_NAME_FROM_TAG_OUT_PARAMS;

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

// winsvc.h SC_HANDLE + rpcdce.h RPC_IF_HANDLE/handle_t (opaque pointers).
// (SC_RPC_HANDLE / SC_NOTIFY_RPC_HANDLE live in crate::svcctl.)
pub type SC_HANDLE = PVOID;
pub type RPC_IF_HANDLE = PVOID;
pub type handle_t = PVOID;

// ---------------------------------------------------------------------------
// The central SCM database structs (services.h). These are SHARED across every
// SCM .c file — database.c writes a SERVICE, groupdb.c reads ->lpGroup, etc. —
// so the layout MUST be byte-exact with the C. Built repr(C) field-for-field in
// C order over the same scalar/pointer types, with arch-gated size+offset
// assertions validated against clang ground truth (see tools probe). A wrong
// field would silently corrupt cross-file reads (cf. struct-mirror-layout-repair).
// ---------------------------------------------------------------------------

/// winsvc.h `_SERVICE_STATUS` (7 DWORDs) — embedded by value in SERVICE; not in
/// winsrv-sys bindgen, so mirrored here.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SERVICE_STATUS {
    pub dwServiceType: DWORD,
    pub dwCurrentState: DWORD,
    pub dwControlsAccepted: DWORD,
    pub dwWin32ExitCode: DWORD,
    pub dwServiceSpecificExitCode: DWORD,
    pub dwCheckPoint: DWORD,
    pub dwWaitHint: DWORD,
}
pub type LPSERVICE_STATUS = *mut SERVICE_STATUS;

/// services.h `_SERVICE_GROUP`.
#[repr(C)]
pub struct SERVICE_GROUP {
    pub GroupListEntry: LIST_ENTRY,
    pub lpGroupName: LPWSTR,
    pub dwRefCount: DWORD,
    pub ServicesRunning: BOOLEAN,
    pub TagCount: ULONG,
    pub TagArray: PULONG,
    pub szGroupName: [u16; 1], // WCHAR[1] flexible tail
}
pub type PSERVICE_GROUP = *mut SERVICE_GROUP;

/// services.h `_SERVICE_IMAGE`.
#[repr(C)]
pub struct SERVICE_IMAGE {
    pub ImageListEntry: LIST_ENTRY,
    pub pszImagePath: LPWSTR,
    pub pszAccountName: LPWSTR,
    pub dwImageRunCount: DWORD,
    pub hControlPipe: HANDLE,
    pub hProcess: HANDLE,
    pub dwProcessId: DWORD,
    pub hToken: HANDLE,
    pub hProfile: HANDLE,
}
pub type PSERVICE_IMAGE = *mut SERVICE_IMAGE;

/// services.h `_SERVICE` — the SCM database record.
#[repr(C)]
pub struct SERVICE {
    pub ServiceListEntry: LIST_ENTRY,
    pub lpServiceName: LPWSTR,
    pub lpDisplayName: LPWSTR,
    pub lpGroup: PSERVICE_GROUP,
    pub lpImage: PSERVICE_IMAGE,
    pub bDeleted: BOOL,
    pub dwResumeCount: DWORD,
    pub RefCount: LONG,
    pub Status: SERVICE_STATUS,
    pub dwStartType: DWORD,
    pub dwErrorControl: DWORD,
    pub dwTag: DWORD,
    pub dwServiceBits: DWORD,
    pub dwServiceTag: DWORD,
    pub Flags: ULONG,
    pub pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    pub ServiceVisited: BOOLEAN,
    pub szServiceName: [u16; 1], // WCHAR[1] flexible tail
}
pub type PSERVICE = *mut SERVICE;

// Arch-gated layout assertions vs clang ground truth (i386 vs amd64/arm64).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<SERVICE_STATUS>() == 28);
    assert!(core::mem::size_of::<SERVICE_GROUP>() == 32);
    assert!(core::mem::size_of::<SERVICE_IMAGE>() == 40);
    assert!(core::mem::size_of::<SERVICE>() == 96);
    assert!(core::mem::offset_of!(SERVICE, lpGroup) == 16);
    assert!(core::mem::offset_of!(SERVICE, Status) == 36);
    assert!(core::mem::offset_of!(SERVICE, pSecurityDescriptor) == 88);
    assert!(core::mem::offset_of!(SERVICE, szServiceName) == 94);
};
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<SERVICE_STATUS>() == 28);
    assert!(core::mem::size_of::<SERVICE_GROUP>() == 56);
    assert!(core::mem::size_of::<SERVICE_IMAGE>() == 80);
    assert!(core::mem::size_of::<SERVICE>() == 128);
    assert!(core::mem::offset_of!(SERVICE, lpGroup) == 32);
    assert!(core::mem::offset_of!(SERVICE, Status) == 60);
    assert!(core::mem::offset_of!(SERVICE, pSecurityDescriptor) == 112);
    assert!(core::mem::offset_of!(SERVICE, szServiceName) == 122);
};

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
