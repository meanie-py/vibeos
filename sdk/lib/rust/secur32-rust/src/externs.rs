//! `externs.rs` — declarations for functions NOT defined in this crate:
//! sibling-DLL imports (kernel32 Heap*/LoadLibrary*, advapi32, netapi32, ntdll
//! Rtl*/Nt* beyond ntdll-sys) and secur32's own exports in not-yet-flipped `.c`.
//!
//! **ABI rule (i386-critical):** `WINAPI`/`__stdcall`/`NTAPI`/`SEC_ENTRY` →
//! `extern "system"` (emits `_Name@N`); plain-C + CRT → `extern "C"` (`_Name`).
//! Wrong ABI links clean on amd64/arm64 but yields a wrong `_Name@N` decoration →
//! undefined symbol on i386. Verify each against the C header/`.spec`. A later
//! in-crate `#[no_mangle]` definition silently overrides the decl (glob-shadow),
//! so churn is minimal as files flip. Grows per wave.
#![allow(non_snake_case, dead_code)]

use crate::prelude::*;
use core::ffi::c_void;

// ---- kernel32 Win32 imports (WINAPI = stdcall → extern "system", `_Name@N`) ----
extern "system" {
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    pub fn HeapReAlloc(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID, dwBytes: SIZE_T) -> LPVOID;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    pub fn MultiByteToWideChar(
        CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: LPCCH, cbMultiByte: i32,
        lpWideCharStr: LPWSTR, cchWideChar: i32,
    ) -> i32;
    pub fn WideCharToMultiByte(
        CodePage: UINT, dwFlags: DWORD, lpWideCharStr: LPCWCH, cchWideChar: i32,
        lpMultiByteStr: LPSTR, cbMultiByte: i32, lpDefaultChar: LPCCH,
        lpUsedDefaultChar: LPBOOL,
    ) -> i32;
    pub fn lstrlenA(lpString: LPCSTR) -> i32;
    pub fn lstrlenW(lpString: LPCWSTR) -> i32;
    pub fn LoadLibraryW(lpLibFileName: LPCWSTR) -> HMODULE;
    pub fn LoadLibraryA(lpLibFileName: LPCSTR) -> HMODULE;
    pub fn FreeLibrary(hLibModule: HMODULE) -> BOOL;
    pub fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> FARPROC;
    pub fn GetLastError() -> DWORD;
    pub fn SetLastError(dwErrCode: DWORD);
    // kernel32 lstr* string helpers (WINAPI = stdcall → extern "system").
    pub fn lstrcpyA(lpString1: LPSTR, lpString2: LPCSTR) -> LPSTR;
    pub fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR;
    pub fn lstrcpynA(lpString1: LPSTR, lpString2: LPCSTR, iMaxLength: i32) -> LPSTR;
    pub fn lstrcpynW(lpString1: LPWSTR, lpString2: LPCWSTR, iMaxLength: i32) -> LPWSTR;
    pub fn lstrcatA(lpString1: LPSTR, lpString2: LPCSTR) -> LPSTR;
    pub fn lstrcatW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR;
    pub fn lstrcmpiW(lpString1: LPCWSTR, lpString2: LPCWSTR) -> i32;
    // netapi32 (WINAPI) — NTLM credential lookup path.
    pub fn NetWkstaUserGetInfo(reserved: LPWSTR, level: DWORD, bufptr: *mut LPBYTE) -> NET_API_STATUS;
    pub fn NetApiBufferFree(Buffer: LPVOID) -> NET_API_STATUS;
    // wincred (advapi32, WINAPI) — cached-credential path.
    pub fn CredReadW(TargetName: LPCWSTR, Type: DWORD, Flags: DWORD, Credential: *mut PCREDENTIALW) -> BOOL;
    pub fn CredFree(Buffer: PVOID);
}

// ---- CRT string fns the wine paths reference (cdecl → extern "C") ----------
extern "C" {
    pub fn strstr(haystack: *const i8, needle: *const i8) -> *mut i8;
}

// ---- advapi32/secext computer+user name imports (WINAPI → extern "system") --
extern "system" {
    pub fn GetComputerNameW(lpBuffer: LPWSTR, nSize: LPDWORD) -> BOOL;
    pub fn GetUserNameW(lpBuffer: LPWSTR, pcbBuffer: LPDWORD) -> BOOL;
}

// ---- LSA policy API (advapi32, NTAPI → extern "system") --------------------
extern "system" {
    pub fn LsaOpenPolicy(
        SystemName: secur32_sys::PLSA_UNICODE_STRING,
        ObjectAttributes: PLSA_OBJECT_ATTRIBUTES,
        DesiredAccess: secur32_sys::ACCESS_MASK,
        PolicyHandle: PLSA_HANDLE,
    ) -> NTSTATUS;
    pub fn LsaQueryInformationPolicy(
        PolicyHandle: LSA_HANDLE,
        InformationClass: secur32_sys::_POLICY_INFORMATION_CLASS,
        Buffer: *mut PVOID,
    ) -> NTSTATUS;
    pub fn LsaClose(ObjectHandle: LSA_HANDLE) -> NTSTATUS;
    pub fn LsaFreeMemory(Buffer: PVOID) -> NTSTATUS;
    pub fn LsaNtStatusToWinError(Status: NTSTATUS) -> ULONG;
}

// ---- CRT (cdecl → extern "C", `_Name`) -----------------------------------
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(s: *mut c_void, c: i32, n: usize) -> *mut c_void;
    pub fn memchr(s: *const c_void, c: i32, n: usize) -> *mut c_void;
    pub fn memcmp(s1: *const c_void, s2: *const c_void, n: usize) -> i32;
    pub fn strlen(s: *const i8) -> usize;
    pub fn strcpy(dest: *mut i8, src: *const i8) -> *mut i8;
    pub fn strcat(dest: *mut i8, src: *const i8) -> *mut i8;
    pub fn strcmp(s1: *const i8, s2: *const i8) -> i32;
    pub fn strncmp(s1: *const i8, s2: *const i8, n: usize) -> i32;
    pub fn strchr(s: *const i8, c: i32) -> *mut i8;
}

// ---- cryptlib MD4/MD5 (declared `void WINAPI` → extern "system") ----------
use crate::types::{MD4_CTX, MD5_CTX};
extern "system" {
    pub fn MD5Init(ctx: *mut MD5_CTX);
    pub fn MD5Update(ctx: *mut MD5_CTX, buf: *const u8, len: UINT);
    pub fn MD5Final(ctx: *mut MD5_CTX);
    pub fn MD4Init(ctx: *mut MD4_CTX);
    pub fn MD4Update(ctx: *mut MD4_CTX, buf: *const u8, len: UINT);
    pub fn MD4Final(ctx: *mut MD4_CTX);
}

// ---- POSIX-ish CRT the wine dispatcher path references (cdecl → extern "C") -
extern "C" {
    pub fn close(fd: i32) -> i32;
    pub fn read(fd: i32, buf: *mut c_void, count: usize) -> isize;
    pub fn write(fd: i32, buf: *const c_void, count: usize) -> isize;
    pub fn sscanf(s: *const i8, fmt: *const i8, ...) -> i32;
}

// (sibling-DLL/secur32-own declarations added per wave below)
