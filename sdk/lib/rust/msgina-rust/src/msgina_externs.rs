//! Cross-DLL out-call declarations for msgina-rust.
//!
//! msgina calls the PUBLIC Win32/LSA APIs in user32/gdi32/advapi32/kernel32/secur32.
//! Those symbols resolve at LINK time via the cmake importlibs (`add_importlibs(msgina
//! advapi32 user32 gdi32 userenv msvcrt kernel32 ntdll)` + delay secur32), so here we
//! only declare them. WINAPI both lowers to `extern "system"` (stdcall on i386, cdecl
//! on amd64/arm64). CRT helpers are `extern "C"` (cdecl). Grown per-file as each
//! translation surfaces a new out-call.
#![allow(non_snake_case, non_upper_case_globals, dead_code)]

use crate::prelude::*;
use core::ffi::c_int;

extern "system" {
    // --- advapi32: tokens / impersonation / registry ---
    pub fn ImpersonateLoggedOnUser(hToken: HANDLE) -> BOOL;
    pub fn RevertToSelf() -> BOOL;
    pub fn OpenThreadToken(
        ThreadHandle: HANDLE,
        DesiredAccess: DWORD,
        OpenAsSelf: BOOL,
        TokenHandle: PHANDLE,
    ) -> BOOL;
    pub fn OpenProcessToken(
        ProcessHandle: HANDLE,
        DesiredAccess: DWORD,
        TokenHandle: PHANDLE,
    ) -> BOOL;
    pub fn GetTokenInformation(
        TokenHandle: HANDLE,
        TokenInformationClass: TOKEN_INFORMATION_CLASS,
        TokenInformation: PVOID,
        TokenInformationLength: DWORD,
        ReturnLength: *mut DWORD,
    ) -> BOOL;
    // Return LSTATUS — declared as DWORD so the Win32 ERROR_*/REG_* (u32) constants
    // and these status returns share one type (LSTATUS/LONG and DWORD are the same
    // 32-bit value; ABI-identical for the cohab).
    pub fn RegOpenCurrentUser(samDesired: REGSAM, phkResult: PHKEY) -> DWORD;
    pub fn RegQueryValueExW(
        hKey: HKEY,
        lpValueName: PCWSTR,
        lpReserved: *mut DWORD,
        lpType: *mut DWORD,
        lpData: PBYTE,
        lpcbData: *mut DWORD,
    ) -> DWORD;

    // --- kernel32: handles / heap / process+thread ---
    pub fn GetLastError() -> DWORD;
    pub fn GetCurrentThread() -> HANDLE;
    pub fn GetCurrentProcess() -> HANDLE;
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> PVOID;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: PVOID) -> BOOL;
    pub fn LocalAlloc(uFlags: UINT, uBytes: SIZE_T) -> PVOID;
    pub fn LocalFree(hMem: PVOID) -> PVOID;
}

extern "C" {
    // --- msvcrt CRT ---
    pub fn wcstoul(nptr: *const u16, endptr: *mut *mut u16, base: c_int) -> c_ulong;
}
