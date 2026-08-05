//! umpnpmgr-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// reactos/debug.h DPRINT/DPRINT1 — no-op MACROS, never out-of-line fns
// (field manual §9.65: an out-of-line no-op is an optimization barrier).
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! DPRINT1 { ($($arg:tt)*) => {{}}; }
pub use crate::{DPRINT, DPRINT1};

// ================= CRT ========================================================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ================= NTSTATUS the island cannot carry ===========================================
// profile_derive flags these as BLOCKING (SWITCH_CONST_IN_BASE_HEADER): rpcserver.c switches on
// them, ntstatus.h OWNS them, and ntdll-sys does not carry them — so without a definition here
// each `match` arm would bind a fresh variable and swallow EVERY status (§9.112, the shape that
// shipped a BUGCHECK 0x1E). Allowlisting ntstatus.h is the wrong fix; it would re-emit the whole
// platform. Transcribed from sdk/include/psdk/ntstatus.h at the cited lines — read, not recalled.
pub const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC0000002u32 as NTSTATUS; // ntstatus.h:332
pub const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000000Du32 as NTSTATUS; // ntstatus.h:343
pub const STATUS_NO_SUCH_DEVICE: NTSTATUS = 0xC000000Eu32 as NTSTATUS; // ntstatus.h:344
pub const STATUS_ACCESS_DENIED: NTSTATUS = 0xC0000022u32 as NTSTATUS; // ntstatus.h:364
pub const STATUS_BUFFER_TOO_SMALL: NTSTATUS = 0xC0000023u32 as NTSTATUS; // ntstatus.h:365
pub const STATUS_OBJECT_NAME_NOT_FOUND: NTSTATUS = 0xC0000034u32 as NTSTATUS; // ntstatus.h:382

// ================= CRT wide-string ============================================================
// msvcrt, not the island: these live in the CRT headers, which are not part of the module's
// Win32 surface. Declared (never defined) — the variadics are legal to DECLARE on stable Rust.
extern "C" {
    pub fn wcslen(s: *const WCHAR) -> usize;
    pub fn wcscpy(dst: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR;
    pub fn wcscat(dst: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR;
    pub fn wcschr(s: *const WCHAR, c: WCHAR) -> *mut WCHAR;
    pub fn wcscmp(a: *const WCHAR, b: *const WCHAR) -> c_int;
    pub fn _wcsicmp(a: *const WCHAR, b: *const WCHAR) -> c_int;
    pub fn swprintf(buf: *mut WCHAR, fmt: *const WCHAR, ...) -> c_int;
}

// ================= constants bindgen cannot fold ==============================================
// Every one below is a CAST or macro EXPRESSION in its header, so the island carries none of
// them. Values read from the cited header, not recalled.
// winreg.h — the predefined registry handles are `((HKEY)(LONG_PTR)(LONG)0x8000000N)`.
pub const HKEY_LOCAL_MACHINE: HKEY = 0x80000002u32 as LONG as LONG_PTR as HKEY;
pub const HKEY_CURRENT_CONFIG: HKEY = 0x80000005u32 as LONG as LONG_PTR as HKEY;
// winnt.h `#define UNICODE_NULL ((WCHAR)0)`
pub const UNICODE_NULL: WCHAR = 0;
// winuser.h `#define HWND_BROADCAST ((HWND)0xffff)`
pub const HWND_BROADCAST: HWND = 0xffff as HWND;
// winbase.h `#define INVALID_HANDLE_VALUE (HANDLE)-1`
pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
// winerror.h `#define RPC_S_OK ERROR_SUCCESS`
pub const RPC_S_OK: LONG = ERROR_SUCCESS as LONG;
// winnt.h `#define PROCESS_ALL_ACCESS (STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0xFFFF)`
pub const PROCESS_ALL_ACCESS: DWORD = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0xFFFF;
// winnt.h `#define SECURITY_NT_AUTHORITY {0,0,0,0,0,5}` — an initialiser, not a value.
pub const SECURITY_NT_AUTHORITY: [BYTE; 6] = [0, 0, 0, 0, 0, 5];
// ntstatus.h:299 `#define STATUS_PLUGPLAY_QUERY_VETOED ((NTSTATUS)0x80000028)`
pub const STATUS_PLUGPLAY_QUERY_VETOED: NTSTATUS = 0x80000028u32 as NTSTATUS;

// ================= wide literals ==============================================================
// This replaces the byte-string-as-wide-pointer form, which is wrong twice over: `b"..."` has
// ALIGNMENT 1 (an odd address is an amd64-only 0x1E STATUS_DATATYPE_MISALIGNMENT), and Rust
// does not append the SECOND zero byte a UTF-16 terminator needs — `b"\\\0"` is one code unit
// with no terminator at all, and an odd byte count cannot be whole u16 units.
#[macro_export]
macro_rules! w {
    ($s:expr $(,)?) => {{
        const INPUT: &[u8] = $s.as_bytes();
        static OUT: [u16; INPUT.len() + 1] = {
            let mut out = [0u16; INPUT.len() + 1];
            let mut i = 0;
            while i < INPUT.len() {
                assert!(INPUT[i] < 0x80, "w!() takes ASCII literals only");
                out[i] = INPUT[i] as u16;
                i += 1;
            }
            out // trailing element stays 0 => NUL-terminated
        };
        OUT.as_ptr()
    }};
}
pub use crate::w;

// ================= names the island spells differently ========================================
// bindgen PREFIXES an anonymous enum's members with the typedef name, so the C's bare
// `CLASS_NOTIFICATION` (precomp.h:48) is emitted as `NOTIFICATION_TYPE_CLASS_NOTIFICATION`.
// Aliased rather than re-declared: a second `const` with a hand-typed value is exactly how a
// match arm ends up comparing against the wrong number (§9.112 / task #38).
pub use crate::types::{
    NOTIFICATION_TYPE_CLASS_NOTIFICATION as CLASS_NOTIFICATION,
    NOTIFICATION_TYPE_TARGET_NOTIFICATION as TARGET_NOTIFICATION,
};
// rpcdce.h `typedef GUID UUID;` — rpcdce.h is not on the allowlist, but the underlying type is.
pub use crate::types::GUID as UUID;
// umpnpmgr compiles -DUNICODE, so wsprintf is wsprintfW before the island ever sees it.
pub use crate::types::wsprintfW as wsprintf;

// PNP_GetVersionInternal returns the WINVER this module was COMPILED for (-DWINVER=0x502 in
// the CMake VER_DEFS). Named for the module rather than `WINVER`: other crates compile at other
// levels, and a bare `WINVER` with a different value in each is exactly what
// scan_const_contradictions exists to catch.
pub const UMPNPMGR_WINVER: WORD = 0x502;
// regstr.h:42 `#define REGSTR_PATH_CLASS TEXT("System\\CurrentControlSet\\Services\\Class")`
#[macro_export]
macro_rules! REGSTR_PATH_CLASS {
    () => { $crate::w!("System\\CurrentControlSet\\Services\\Class") };
}
pub use crate::REGSTR_PATH_CLASS;
// winnt.h `#define ARRAYSIZE(A) RTL_NUMBER_OF_V2(A)` — a COUNT of elements, never a byte size.
#[macro_export]
macro_rules! ARRAYSIZE { ($a:expr $(,)?) => { ($a).len() as u32 }; }
pub use crate::ARRAYSIZE;
// winnt.h `#define FIELD_OFFSET(t, f) ((LONG)(LONG_PTR)&(((t*)0)->f))`
#[macro_export]
macro_rules! FIELD_OFFSET {
    ($t:ty, $f:ident $(,)?) => { core::mem::offset_of!($t, $f) as LONG };
}
pub use crate::FIELD_OFFSET;

// ================= UNICODE A/W aliases ========================================================
// umpnpmgr compiles -DUNICODE, so `#define RegOpenKeyEx RegOpenKeyExW` and its siblings are
// resolved by the PREPROCESSOR and never reach the island. Aliases, never re-declarations — a
// hand-written prototype for RegQueryValueExW is the exact shape that links and then corrupts.
pub use crate::types::{
    RegDeleteValueW as RegDeleteValue, RegOpenKeyExW as RegOpenKeyEx,
    RegQueryValueExW as RegQueryValueEx, RegSetValueExW as RegSetValueEx,
};

// ================= winnt.h block-op macros ====================================================
// Macros, never out-of-line fns (§9.65), and never hand-written byte loops: LLVM folds any
// byte-copy loop back into a call to the function itself → infinite self-recursion.
#[macro_export]
macro_rules! CopyMemory {
    ($dst:expr, $src:expr, $len:expr $(,)?) => {
        core::ptr::copy_nonoverlapping($src as *const u8, $dst as *mut u8, $len as usize)
    };
}
#[macro_export]
macro_rules! MoveMemory {
    ($dst:expr, $src:expr, $len:expr $(,)?) => {
        core::ptr::copy($src as *const u8, $dst as *mut u8, $len as usize)
    };
}
#[macro_export]
macro_rules! ZeroMemory {
    ($dst:expr, $len:expr $(,)?) => { core::ptr::write_bytes($dst as *mut u8, 0u8, $len as usize) };
}
#[macro_export]
macro_rules! RtlZeroMemory {
    ($dst:expr, $len:expr $(,)?) => { core::ptr::write_bytes($dst as *mut u8, 0u8, $len as usize) };
}
#[macro_export]
macro_rules! UNREFERENCED_PARAMETER { ($($p:expr),* $(,)?) => { $( let _ = &$p; )* }; }
#[macro_export]
macro_rules! RtlGetProcessHeap { () => { (*NtCurrentPeb()).ProcessHeap }; }
pub use crate::{
    CopyMemory, MoveMemory, RtlGetProcessHeap, RtlZeroMemory, UNREFERENCED_PARAMETER, ZeroMemory,
};
