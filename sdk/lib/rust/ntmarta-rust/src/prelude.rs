//! ntmarta-rust prelude — the single import surface the translated modules use.
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

// ================= enum member aliases ========================================================
// bindgen prefixes a C enum's members with the enum's tag, so the bare spelling the C uses is
// absent from the island. Alias, never re-declare: a literal here could drift from the island's
// value silently.
//
// FIFTEEN of these are `case` labels in ntmarta.c, which makes island completeness a
// CORRECTNESS property, not a convenience (§9.112): a constant the translation cannot resolve
// becomes a catch-all `match` arm that swallows every case. `profile_derive` flagged only
// DENY_ACCESS, because it checks whether the constant's owning HEADER is allowlisted —
// accctrl.h is, and the bare name is still missing. Allowlisting is necessary, not sufficient.
pub const NOT_USED_ACCESS: _ACCESS_MODE = _ACCESS_MODE_NOT_USED_ACCESS;
pub const GRANT_ACCESS: _ACCESS_MODE = _ACCESS_MODE_GRANT_ACCESS;
pub const SET_ACCESS: _ACCESS_MODE = _ACCESS_MODE_SET_ACCESS;
pub const DENY_ACCESS: _ACCESS_MODE = _ACCESS_MODE_DENY_ACCESS;
pub const REVOKE_ACCESS: _ACCESS_MODE = _ACCESS_MODE_REVOKE_ACCESS;
pub const SET_AUDIT_SUCCESS: _ACCESS_MODE = _ACCESS_MODE_SET_AUDIT_SUCCESS;
pub const SET_AUDIT_FAILURE: _ACCESS_MODE = _ACCESS_MODE_SET_AUDIT_FAILURE;

pub const TRUSTEE_IS_SID: _TRUSTEE_FORM = _TRUSTEE_FORM_TRUSTEE_IS_SID;
pub const TRUSTEE_IS_NAME: _TRUSTEE_FORM = _TRUSTEE_FORM_TRUSTEE_IS_NAME;
pub const TRUSTEE_IS_OBJECTS_AND_SID: _TRUSTEE_FORM = _TRUSTEE_FORM_TRUSTEE_IS_OBJECTS_AND_SID;
pub const TRUSTEE_IS_OBJECTS_AND_NAME: _TRUSTEE_FORM = _TRUSTEE_FORM_TRUSTEE_IS_OBJECTS_AND_NAME;

pub const SE_KERNEL_OBJECT: _SE_OBJECT_TYPE = _SE_OBJECT_TYPE_SE_KERNEL_OBJECT;
pub const SE_FILE_OBJECT: _SE_OBJECT_TYPE = _SE_OBJECT_TYPE_SE_FILE_OBJECT;
pub const SE_SERVICE: _SE_OBJECT_TYPE = _SE_OBJECT_TYPE_SE_SERVICE;
pub const SE_REGISTRY_KEY: _SE_OBJECT_TYPE = _SE_OBJECT_TYPE_SE_REGISTRY_KEY;
pub const SE_WINDOW_OBJECT: _SE_OBJECT_TYPE = _SE_OBJECT_TYPE_SE_WINDOW_OBJECT;

pub const NO_MULTIPLE_TRUSTEE: _MULTIPLE_TRUSTEE_OPERATION =
    _MULTIPLE_TRUSTEE_OPERATION_NO_MULTIPLE_TRUSTEE;

// ================= function-like MACROS =======================================================
// Macros in C, macros here. `FIELD_OFFSET` must be one regardless: its second argument is a
// FIELD NAME, and as a fn call the compiler reads it as a value and reports 9 phantom
// "cannot find value `SidStart`" / "`ObjectType`" errors.
#[macro_export]
macro_rules! FIELD_OFFSET { ($t:ty, $f:ident $(,)?) => { (core::mem::offset_of!($t, $f) as LONG) }; }
// winbase.h ZeroMemory→RtlZeroMemory, CopyMemory→RtlCopyMemory. Routed to the CRT entry points
// rather than a Rust byte loop: LLVM folds any such loop back into a call to memset/memcpy.
#[macro_export]
macro_rules! ZeroMemory {
    ($d:expr, $l:expr $(,)?) => { $crate::prelude::memset($d as *mut c_void, 0, $l as usize) };
}
#[macro_export]
macro_rules! CopyMemory {
    ($d:expr, $s:expr, $l:expr $(,)?) => {
        $crate::prelude::memcpy($d as *mut c_void, $s as *const c_void, $l as usize)
    };
}
// reactos/debug.h. No-op MACROS, never out-of-line fns (field manual §9.65: an out-of-line
// no-op is an optimization barrier that can uncloak a latent ABI mismatch into `ud2`).
#[macro_export] macro_rules! UNIMPLEMENTED { () => {{}}; }
#[macro_export] macro_rules! UNREFERENCED_PARAMETER { ($($arg:tt)*) => {{}}; }
pub use crate::{CopyMemory, FIELD_OFFSET, UNIMPLEMENTED, UNREFERENCED_PARAMETER, ZeroMemory};

// ================= constants bindgen cannot emit ==============================================
// winreg.h spells each of these as a cast chain, `((HKEY)(LONG_PTR)(LONG)0x8000000N)`, not an
// integer constant. The sign-extend through LONG matters on 64-bit: 0x80000002 as i32 as isize
// gives 0xFFFFFFFF80000002, which is what the C produces and what the registry expects.
pub const HKEY_CLASSES_ROOT: HKEY = 0x8000_0000u32 as i32 as isize as HKEY;
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as i32 as isize as HKEY;
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;
pub const HKEY_USERS: HKEY = 0x8000_0003u32 as i32 as isize as HKEY;
pub const HKEY_CURRENT_CONFIG: HKEY = 0x8000_0005u32 as i32 as isize as HKEY;

// ================= CRT ========================================================================
// msvcrt entry points, cdecl.
extern "C" {
    pub fn _wcsicmp(_Str1: *const wchar_t, _Str2: *const wchar_t) -> c_int;
    pub fn wcschr(
        _Str: *const c_ushort,
        _Ch: c_ushort,
    ) -> *mut c_ushort;
    pub fn wcscmp(
        _Str1: *const c_ushort,
        _Str2: *const c_ushort,
    ) -> c_int;
    pub fn wcscpy(_Dest: *mut wchar_t, _Source: *const wchar_t) -> *mut wchar_t;
    pub fn wcslen(_Str: *const c_ushort) -> usize;
}

// ================= Win32 imports ==============================================================
// Declared, never defined: these live in the DLLs ntmarta links against (add_importlibs).
// GENERATED by bindgen from the same header set and the same -D flags ntmarta.c compiles with,
// then checked IDENTICAL across all three arches (only wcslen's size_t differs, spelled `usize`
// above). A hand-written signature that is merely plausible links fine and corrupts silently.
extern "system" {
    pub fn AddAccessAllowedAce(arg1: PACL, arg2: DWORD, arg3: DWORD, arg4: PSID) -> BOOL;
    pub fn AddAccessDeniedAce(arg1: PACL, arg2: DWORD, arg3: DWORD, arg4: PSID) -> BOOL;
    pub fn BuildTrusteeWithObjectsAndSidW(
        arg1: PTRUSTEE_W,
        arg2: POBJECTS_AND_SID,
        arg3: *mut GUID,
        arg4: *mut GUID,
        arg5: PSID,
    );
    pub fn BuildTrusteeWithSidW(arg1: PTRUSTEE_W, arg2: PSID);
    pub fn CloseHandle(arg1: HANDLE) -> BOOL;
    pub fn CloseServiceHandle(arg1: SC_HANDLE) -> BOOL;
    pub fn CopySid(arg1: DWORD, arg2: PSID, arg3: PSID) -> BOOL;
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL;
    pub fn GetAce(arg1: PACL, arg2: DWORD, arg3: *mut LPVOID) -> BOOL;
    pub fn GetAclInformation(
        arg1: PACL,
        arg2: PVOID,
        arg3: DWORD,
        arg4: ACL_INFORMATION_CLASS,
    ) -> BOOL;
    pub fn GetCurrentProcess() -> HANDLE;
    pub fn GetCurrentThread() -> HANDLE;
    pub fn GetLastError() -> DWORD;
    pub fn GetLengthSid(arg1: PSID) -> DWORD;
    pub fn GetSecurityDescriptorDacl(
        arg1: PSECURITY_DESCRIPTOR,
        arg2: LPBOOL,
        arg3: *mut PACL,
        arg4: LPBOOL,
    ) -> BOOL;
    pub fn GetSecurityDescriptorGroup(
        arg1: PSECURITY_DESCRIPTOR,
        arg2: *mut PSID,
        arg3: LPBOOL,
    ) -> BOOL;
    pub fn GetSecurityDescriptorOwner(
        arg1: PSECURITY_DESCRIPTOR,
        arg2: *mut PSID,
        arg3: LPBOOL,
    ) -> BOOL;
    pub fn GetSecurityDescriptorSacl(
        arg1: PSECURITY_DESCRIPTOR,
        arg2: LPBOOL,
        arg3: *mut PACL,
        arg4: LPBOOL,
    ) -> BOOL;
    pub fn GetTokenInformation(
        arg1: HANDLE,
        arg2: TOKEN_INFORMATION_CLASS,
        arg3: PVOID,
        arg4: DWORD,
        arg5: PDWORD,
    ) -> BOOL;
    pub fn GetUserObjectSecurity(
        hObj: HANDLE,
        pSIRequested: PSECURITY_INFORMATION,
        pSID: PSECURITY_DESCRIPTOR,
        nLength: DWORD,
        lpnLengthNeeded: LPDWORD,
    ) -> BOOL;
    pub fn InitializeAcl(arg1: PACL, arg2: DWORD, arg3: DWORD) -> BOOL;
    pub fn LocalAlloc(arg1: UINT, arg2: SIZE_T) -> HLOCAL;
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL;
    pub fn LocalReAlloc(arg1: HLOCAL, arg2: SIZE_T, arg3: UINT) -> HLOCAL;
    pub fn LsaClose(arg1: LSA_HANDLE) -> NTSTATUS;
    pub fn LsaFreeMemory(arg1: PVOID) -> NTSTATUS;
    pub fn LsaLookupNames2(
        arg1: LSA_HANDLE,
        arg2: ULONG,
        arg3: ULONG,
        arg4: PLSA_UNICODE_STRING,
        arg5: *mut PLSA_REFERENCED_DOMAIN_LIST,
        arg6: *mut PLSA_TRANSLATED_SID2,
    ) -> NTSTATUS;
    pub fn LsaNtStatusToWinError(arg1: NTSTATUS) -> ULONG;
    pub fn LsaOpenPolicy(
        arg1: PLSA_UNICODE_STRING,
        arg2: PLSA_OBJECT_ATTRIBUTES,
        arg3: ACCESS_MASK,
        arg4: PLSA_HANDLE,
    ) -> NTSTATUS;
    pub fn OpenProcessToken(arg1: HANDLE, arg2: DWORD, arg3: PHANDLE) -> BOOL;
    pub fn OpenSCManagerW(arg1: LPCWSTR, arg2: LPCWSTR, arg3: DWORD) -> SC_HANDLE;
    pub fn OpenServiceW(arg1: SC_HANDLE, arg2: LPCWSTR, arg3: DWORD) -> SC_HANDLE;
    pub fn OpenThreadToken(arg1: HANDLE, arg2: DWORD, arg3: BOOL, arg4: PHANDLE) -> BOOL;
    pub fn QueryServiceObjectSecurity(
        hService: SC_HANDLE,
        dwSecurityInformation: SECURITY_INFORMATION,
        lpSecurityDescriptor: PSECURITY_DESCRIPTOR,
        cbBufSize: DWORD,
        pcbBytesNeeded: LPDWORD,
    ) -> BOOL;
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS;
    pub fn RegConnectRegistryW(arg1: LPCWSTR, arg2: HKEY, arg3: PHKEY) -> LSTATUS;
    pub fn RegGetKeySecurity(
        hKey: HKEY,
        SecurityInformation: SECURITY_INFORMATION,
        pSecurityDescriptor: PSECURITY_DESCRIPTOR,
        lpcbSecurityDescriptor: LPDWORD,
    ) -> LSTATUS;
    pub fn RegOpenKeyExW(
        arg1: HKEY,
        arg2: LPCWSTR,
        arg3: DWORD,
        arg4: REGSAM,
        arg5: PHKEY,
    ) -> LSTATUS;
    pub fn RegSetKeySecurity(
        arg1: HKEY,
        arg2: SECURITY_INFORMATION,
        arg3: PSECURITY_DESCRIPTOR,
    ) -> LSTATUS;
    pub fn SetLastError(arg1: DWORD);
    pub fn SetServiceObjectSecurity(
        arg1: SC_HANDLE,
        arg2: SECURITY_INFORMATION,
        arg3: PSECURITY_DESCRIPTOR,
    ) -> BOOL;
    pub fn SetUserObjectSecurity(
        arg1: HANDLE,
        arg2: PSECURITY_INFORMATION,
        arg3: PSECURITY_DESCRIPTOR,
    ) -> BOOL;
}

// The UNICODE `#define FOO FOOW` renames, as aliases so there is exactly one symbol identity.
pub use self::{
    BuildTrusteeWithObjectsAndSidW as BuildTrusteeWithObjectsAndSid,
    BuildTrusteeWithSidW as BuildTrusteeWithSid, OpenSCManagerW as OpenSCManager,
    OpenServiceW as OpenService, RegConnectRegistryW as RegConnectRegistry,
    RegOpenKeyExW as RegOpenKeyEx,
};

// winnt.h `#define TRUE 1` / `FALSE 0`. ntdll-sys's bindgen types them u32; every use here is
// a BOOL (`*Allocated = FALSE`, an `OpenThreadToken` argument). An explicit item beats a glob
// import, so naming them here re-types them crate-wide without a cast at each site.
pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;
// ntstatus.h `#define STATUS_NONE_MAPPED ((NTSTATUS)0xC0000073L)` — a cast expression, so
// bindgen's macro evaluator skips it.
pub const STATUS_NONE_MAPPED: NTSTATUS = 0xC000_0073u32 as NTSTATUS;

// ndk/obtypes.h `InitializeObjectAttributes(p,n,a,r,s)` — a BLOCK macro that fills the struct
// field by field. Kept a macro: it takes the destination by address and writes six fields, so
// an out-of-line fn would need a &mut and change every call site.
#[macro_export]
macro_rules! InitializeObjectAttributes {
    ($p:expr, $n:expr, $a:expr, $r:expr, $s:expr $(,)?) => {{
        (*$p).Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
        (*$p).RootDirectory = $r;
        (*$p).Attributes = $a;
        (*$p).ObjectName = $n;
        (*$p).SecurityDescriptor = $s;
        (*$p).SecurityQualityOfService = core::ptr::null_mut();
    }};
}
pub use crate::InitializeObjectAttributes;

// ndk/rtlfuncs.h:965 — NTSYSAPI BOOLEAN NTAPI RtlFreeHeap(HANDLE, ULONG, PVOID).
extern "system" {
    pub fn RtlFreeHeap(HeapHandle: HANDLE, Flags: ULONG, P: PVOID) -> BOOLEAN;
}

// ndk/rtlfuncs.h:1127 `#define RtlGetProcessHeap() (NtCurrentPeb()->ProcessHeap)` — a macro
// over the PEB, so there is no symbol; kept a macro here for the same reason.
#[macro_export]
macro_rules! RtlGetProcessHeap { () => { (*NtCurrentPeb()).ProcessHeap }; }
pub use crate::RtlGetProcessHeap;
