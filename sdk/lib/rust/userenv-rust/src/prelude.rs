//! userenv-rust prelude — the single import surface the translated modules use.
//!
//! Same role as the winsock-family preludes: bridges the gaps between the bindgen `types`
//! island and the translated C:
//!   1. bindgen RUSTIFIED the C enums — alias the bare names the C source actually uses.
//!   2. bindgen SKIPS cast-#defines (CSIDL_*, shell-folder ids, registry flags) — transcribe
//!      them with a header citation.
//!   3. C MACROS/inlines (NT_SUCCESS, RtlZeroMemory) have NO linkable symbol — implement them
//!      here (never as an out-of-line no-op `extern fn`, field-manual §9.65).
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
use core::ffi::c_void;
// One declaration per external symbol: the variant matching the C prototype.
pub use crate::userenv::profile::InitializeSecurityDescriptor;
pub use crate::userenv::profile::SetSecurityDescriptorDacl;

// bindgen emits `TRUE/FALSE: u32`, but Win32 `BOOL` is `c_int` (i32) and the translated
// #[no_mangle] fns are declared `-> BOOL`; the u32 constants collide at every BOOL slot
// (return/field/arg). Redefine them at the correct width — an explicit item shadows the
// `types::*` glob (no conflict). winnt.h: `#define TRUE 1` / `#define FALSE 0`.
pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;

// ---- bridges added as the cargo-check grind reveals them ----

// Registry root pseudo-handles — winreg.h: `((HKEY)(LONG_PTR)(LONG)0x8000000N)`
// (sdk/include/…/winreg.h). Stored as HKEY (a *mut HKEY__); the u32 magic values
// zero-extend to the correct 64-bit pattern on amd64/arm64.
pub const HKEY_CLASSES_ROOT: HKEY = 0x8000_0000u32 as i32 as isize as HKEY;
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as i32 as isize as HKEY;
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;
pub const HKEY_USERS: HKEY = 0x8000_0003u32 as i32 as isize as HKEY;

// winnls.h: `#define CP_ACP 0` (ANSI code page). Passed to MultiByte/WideChar as UINT.
pub const CP_ACP: u32 = 0;

// handleapi.h: `#define INVALID_HANDLE_VALUE ((HANDLE)-1)`.
pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;

// userenv resource IDs — dll/win32/userenv/resources.h (string-table + dialog control ids).
// Used only to index the shell-folder tables / address dialog controls; values verbatim.
pub const IDS_PROFILEPATH: u32 = 1;
pub const IDS_APPDATA: u32 = 2;
pub const IDS_DESKTOP: u32 = 3;
pub const IDS_FAVORITES: u32 = 4;
pub const IDS_MYDOCUMENTS: u32 = 5;
pub const IDS_NETHOOD: u32 = 6;
pub const IDS_PRINTHOOD: u32 = 7;
pub const IDS_RECENT: u32 = 8;
pub const IDS_SENDTO: u32 = 9;
pub const IDS_TEMPLATES: u32 = 10;
pub const IDS_STARTMENU: u32 = 11;
pub const IDS_PROGRAMS: u32 = 12;
pub const IDS_STARTUP: u32 = 13;
pub const IDS_LOCALSETTINGS: u32 = 14;
pub const IDS_LOCALAPPDATA: u32 = 15;
pub const IDS_TEMP: u32 = 16;
pub const IDS_CACHE: u32 = 17;
pub const IDS_HISTORY: u32 = 18;
pub const IDS_COOKIES: u32 = 19;
// Dialog control ids (resources.h) — passed to GetDlgItem/SetDlgItemTextW as `int`.
pub const IDC_TIMEOUT: i32 = 1004;
pub const IDC_TIMEOUTSTATIC: i32 = 1005;
pub const IDC_ERRORDESC: i32 = 3001;

// FIELD_OFFSET(ACCESS_ALLOWED_ACE, SidStart) — winnt.h ACE layout (bindgen didn't emit
// these file-external structs). ACE_HEADER = {BYTE,BYTE,WORD} = 4 bytes; SidStart follows
// Header + ACCESS_MASK(=DWORD), so offset_of!(…, SidStart) == 8. Defined here so the
// misc.rs DACL-sizing site uses `offset_of!` (exact), never a hand-written 8.
#[repr(C)]
pub struct ACE_HEADER {
    pub AceType: u8,
    pub AceFlags: u8,
    pub AceSize: u16,
}
#[repr(C)]
pub struct ACCESS_ALLOWED_ACE {
    pub Header: ACE_HEADER,
    pub Mask: u32,
    pub SidStart: u32,
}

// ---- C macros the translator emitted as calls — reimplemented inline (no linkable
// symbol; never an out-of-line no-op extern, field-manual §9.65). ----
// ntdef.h: NT_SUCCESS(Status) == ((NTSTATUS)(Status) >= 0)
#[inline(always)]
pub fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}
// winuser.h: LOWORD/HIWORD extract the low/high WORD of a DWORD_PTR-sized value.
#[inline(always)]
pub fn LOWORD(l: WPARAM) -> WORD {
    (l & 0xffff) as WORD
}
#[inline(always)]
pub fn HIWORD(l: WPARAM) -> WORD {
    ((l >> 16) & 0xffff) as WORD
}
// winuser.h: MAKEWPARAM(l,h) = (WORD)l | ((WORD)h << 16), widened to WPARAM.
#[inline(always)]
pub fn MAKEWPARAM(l: u32, h: u16) -> WPARAM {
    (((l & 0xffff) | ((h as u32) << 16)) as WPARAM)
}
// winuser.h: MAKEINTRESOURCEW(i) = (LPWSTR)(ULONG_PTR)(WORD)i
#[inline(always)]
pub fn MAKEINTRESOURCEW(i: INT) -> LPWSTR {
    (i as u16 as usize) as LPWSTR
}
// winbase.h: ZeroMemory(dst,len) = RtlZeroMemory = memset(dst,0,len).
#[inline(always)]
pub unsafe fn ZeroMemory(dst: PVOID, len: usize) {
    core::ptr::write_bytes(dst as *mut u8, 0, len);
}

// ---- Win32 / CRT / userenv-internal imports absent from a subset of the per-file
// enforce_import_abi blocks. Declared once here (the module `use crate::prelude::*`
// glob is shadowed by any file-local decl, so no conflict). ABI per the exporting DLL:
// Win32 = stdcall (extern "system"), CRT + strsafe (as the translation calls them) and
// the #[no_mangle] extern "C" userenv-internal helpers = cdecl. ----
extern "system" {
    pub fn SetLastError(dwErrCode: DWORD);
    pub fn RegOpenKeyExW(hKey: HKEY, lpSubKey: LPCWSTR, ulOptions: DWORD, samDesired: DWORD, phkResult: *mut HKEY) -> LONG;
    pub fn RegCloseKey(hKey: HKEY) -> LONG;
    pub fn RegQueryValueExW(hKey: HKEY, lpValueName: LPCWSTR, lpReserved: *mut DWORD, lpType: *mut DWORD, lpData: LPBYTE, lpcbData: *mut DWORD) -> LONG;
    pub fn RegFlushKey(hKey: HKEY) -> LONG;
    pub fn RegDeleteKeyW(hKey: HKEY, lpSubKey: LPCWSTR) -> LONG;
    pub fn RegCopyTreeW(hKeySrc: HKEY, lpSubKey: LPCWSTR, hKeyDest: HKEY) -> LONG;
    pub fn FreeSid(pSid: PSID) -> PVOID;
    pub fn GetLengthSid(pSid: PSID) -> DWORD;
    pub fn AllocateAndInitializeSid(pIdentifierAuthority: *mut SID_IDENTIFIER_AUTHORITY, nSubAuthorityCount: u8, dwSubAuthority0: DWORD, dwSubAuthority1: DWORD, dwSubAuthority2: DWORD, dwSubAuthority3: DWORD, dwSubAuthority4: DWORD, dwSubAuthority5: DWORD, dwSubAuthority6: DWORD, dwSubAuthority7: DWORD, pSid: *mut PSID) -> BOOL;
    pub fn InitializeAcl(pAcl: PACL, nAclLength: DWORD, dwAclRevision: DWORD) -> BOOL;
    pub fn AddAccessAllowedAce(pAcl: PACL, dwAceRevision: DWORD, AccessMask: DWORD, pSid: PSID) -> BOOL;
    pub fn LocalAlloc(uFlags: UINT, uBytes: usize) -> LPVOID;
    pub fn LocalFree(hMem: HLOCAL) -> HLOCAL;
    pub fn SetFileAttributesW(lpFileName: LPCWSTR, dwFileAttributes: DWORD) -> BOOL;
    pub fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD;
    pub fn SetEnvironmentVariableW(lpName: LPCWSTR, lpValue: LPCWSTR) -> BOOL;
    pub fn GetSystemTimeAsFileTime(lpSystemTimeAsFileTime: *mut FILETIME);
    pub fn LoadLibraryW(lpLibFileName: LPCWSTR) -> HMODULE;
    pub fn FreeLibrary(hLibModule: HMODULE) -> BOOL;
    pub fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> PVOID;
    pub fn CreateEventW(lpEventAttributes: *mut SECURITY_ATTRIBUTES, bManualReset: BOOL, bInitialState: BOOL, lpName: LPCWSTR) -> HANDLE;
    pub fn InitializeCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn DeleteCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn SetDlgItemTextW(hDlg: HWND, nIDDlgItem: INT, lpString: LPCWSTR) -> BOOL;
    pub fn GetDlgItem(hDlg: HWND, nIDDlgItem: INT) -> HWND;
    pub fn ShowWindow(hWnd: HWND, nCmdShow: INT) -> BOOL;
    pub fn SetForegroundWindow(hWnd: HWND) -> BOOL;
    pub fn EndDialog(hDlg: HWND, nResult: INT_PTR) -> BOOL;
    pub fn KillTimer(hWnd: HWND, uIDEvent: usize) -> BOOL;
    pub fn SetTimer(hWnd: HWND, nIDEvent: usize, uElapse: UINT, lpTimerFunc: PVOID) -> usize;
    pub fn PostMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    pub fn DialogBoxParamW(
        hInstance: HINSTANCE,
        lpTemplateName: LPCWSTR,
        hWndParent: HWND,
        lpDialogFunc: Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR>,
        dwInitParam: LPARAM,
    ) -> INT_PTR;
}
extern "C" {
    // StringCb*/StringCch* non-variadic are strsafe.h `static inline`s (no symbol) → inline shims
    // below. RemoveDirectoryPath etc. are userenv-internal #[no_mangle] cdecl fns defined in the
    // modules.
    pub fn RemoveDirectoryPath(lpPathName: LPCWSTR) -> BOOL;
    pub fn UpdateUsersShellFolderSettings(lpUserProfilePath: LPCWSTR, hUserKey: HKEY) -> BOOL;
    pub fn AppendSystemPostfix(lpName: LPWSTR, dwMaxLength: DWORD) -> BOOL;
}
// strsafe.h's printf entry points are VARIADIC, and stable Rust can CALL a C-variadic though
// it cannot define one; so these are strsafe-rust macros that call msvcrt's `_snwprintf` in
// place, with strsafe's bounds and HRESULTs. The C trampoline that used to define them
// (rust_varargs_shim.c) retired 2026-09-25. Call sites say `StringCchPrintfW!(...)`.
pub use strsafe_rust::StringCchPrintfW;

// strsafe.h StringCchCopyNW is a `static inline` (no linkable symbol) — undefined at link when
// declared extern. Reimplement inline (§9.65-sanctioned for a header inline): copy at most
// cchSrcLength chars from src into dest (cchDest chars incl. the NUL), always NUL-terminates.
// Returns S_OK (0) / STRSAFE_E_INVALID_PARAMETER for a zero-length dest. The 2 environment.rs
// call sites ignore the return.
#[inline]
pub unsafe fn StringCchCopyNW(
    pszDest: LPWSTR,
    cchDest: usize,
    pszSrc: LPCWSTR,
    cchSrcLength: usize,
) -> HRESULT {
    if cchDest == 0 {
        return 0x8007_0057u32 as HRESULT;
    }
    let mut i: usize = 0;
    while i + 1 < cchDest && i < cchSrcLength && *pszSrc.add(i) != 0 {
        *pszDest.add(i) = *pszSrc.add(i);
        i += 1;
    }
    *pszDest.add(i) = 0;
    0 // S_OK
}

// strsafe.h StringCch*/StringCb* (non-variadic) — `static inline`s, no linkable symbol → shims.
// Cch = CHAR count, Cb = BYTE count (delegates to the Cch form). All NUL-terminate; return
// S_OK (0) / STRSAFE_E_INVALID_PARAMETER (0x80070057) for a zero-length dest.
#[inline]
pub unsafe fn StringCchCopyW(pszDest: LPWSTR, cchDest: usize, pszSrc: LPCWSTR) -> i32 {
    if cchDest == 0 {
        return 0x8007_0057u32 as i32;
    }
    let mut i: usize = 0;
    while i + 1 < cchDest && *pszSrc.add(i) != 0 {
        *pszDest.add(i) = *pszSrc.add(i);
        i += 1;
    }
    *pszDest.add(i) = 0;
    0
}
#[inline]
pub unsafe fn StringCchCatW(pszDest: LPWSTR, cchDest: usize, pszSrc: LPCWSTR) -> i32 {
    if cchDest == 0 {
        return 0x8007_0057u32 as i32;
    }
    let mut d: usize = 0;
    while d < cchDest && *pszDest.add(d) != 0 {
        d += 1;
    }
    let mut s: usize = 0;
    while d + 1 < cchDest && *pszSrc.add(s) != 0 {
        *pszDest.add(d) = *pszSrc.add(s);
        d += 1;
        s += 1;
    }
    if d < cchDest {
        *pszDest.add(d) = 0;
    }
    0
}
#[inline]
pub unsafe fn StringCbCopyW(pszDest: LPWSTR, cbDest: usize, pszSrc: LPCWSTR) -> i32 {
    StringCchCopyW(pszDest, cbDest / core::mem::size_of::<u16>(), pszSrc)
}
#[inline]
pub unsafe fn StringCbCatW(pszDest: LPWSTR, cbDest: usize, pszSrc: LPCWSTR) -> i32 {
    StringCchCatW(pszDest, cbDest / core::mem::size_of::<u16>(), pszSrc)
}

// winerror.h SUCCEEDED(hr) — macro `((HRESULT)(hr) >= 0)`, no linkable symbol → inline shim.
#[inline]
pub fn SUCCEEDED(hr: HRESULT) -> BOOL {
    (hr >= 0) as BOOL
}
