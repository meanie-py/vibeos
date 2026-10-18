//! Preamble: Win32 predefined constants and the extern `WLSession`
//! global that the per-file translation can't resolve. winlogon-sys
//! covers the API surface + WLSESSION layout, but the predefined HKEY
//! roots are winreg.h macros (not bindgen consts) and `WLSession` is a
//! global owned by the still-C winlogon core. Shared by the leaf modules.

#![allow(non_snake_case, non_upper_case_globals)]

use winlogon_sys::nt::*;

/// `wstr!("Foo")` → a `&'static [u16]` NUL-terminated UTF-16 literal (ASCII
/// source). Mirrors services-rust's macro; used by the flipped GINA/SAS/RPC
/// cores for the many registry-path / pipe-name / window-name wide literals.
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
        &ARR
    }};
}

/// `wstr_arr!("Foo")` → an owned `[u16; N]` NUL-terminated UTF-16 array. For the
/// mutable command-line buffers CreateProcessW needs (the C uses
/// `WCHAR line[] = L"...";` writable stack arrays).
#[macro_export]
macro_rules! wstr_arr {
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
        ARR
    }};
}

// Predefined registry roots (winreg.h #defines — not bindgen-visible).
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as i32 as isize as HKEY;
pub const HKEY_CLASSES_ROOT: HKEY = 0x8000_0000u32 as i32 as isize as HKEY;
pub const HKEY_USERS: HKEY = 0x8000_0003u32 as i32 as isize as HKEY;

// The session state + module instance + heap-strdup, now owned by the (Rust)
// winlogon core (winlogon.rs, flipped from winlogon.c). Re-exported here so the
// leaves keep resolving them through the preamble; an `extern` decl would
// collide with the in-crate definition (dup-symbol → ud2), so re-export — same
// pattern as notify's CallNotificationDlls. WLSession layout comes from
// winlogon-sys (_WLSESSION, size-asserted ×3).
pub use crate::winlogon::{hAppInstance, WLSession, WlStrDup};

// CallNotificationDlls now lives in the Rust notify module (notify.c was
// flipped). Re-export it so the other leaves keep resolving it through the
// preamble; declaring it `extern` here too would collide with the in-crate
// definition (dup-symbol → ud2).
pub use crate::notify::CallNotificationDlls;

// Undocumented user32 export (WINAPI) — assigns a window station to a
// logged-in user. No public prototype, so it never appears in bindgen.
extern "system" {
    pub fn SetWindowStationUser(
        hWindowStation: HWINSTA,
        pluid: PLUID,
        psid: PSID,
        size: DWORD,
    ) -> BOOL;
}

// LIST_ENTRY ops are FORCEINLINE in the C headers — bindgen emits extern decls
// but there's NO exported symbol (link fails as `undefined _InitializeListHead`).
// Provide them inline (they shadow the glob's extern decls — local wins). The
// GINA/SAS cores use them for the dialog tracking list etc.
#[inline]
pub unsafe fn InitializeListHead(head: PLIST_ENTRY) {
    (*head).Flink = head;
    (*head).Blink = head;
}
#[inline]
pub unsafe fn InsertHeadList(head: PLIST_ENTRY, entry: PLIST_ENTRY) {
    let first = (*head).Flink;
    (*entry).Flink = first;
    (*entry).Blink = head;
    (*first).Blink = entry;
    (*head).Flink = entry;
}
#[inline]
pub unsafe fn InsertTailList(head: PLIST_ENTRY, entry: PLIST_ENTRY) {
    let last = (*head).Blink;
    (*entry).Flink = head;
    (*entry).Blink = last;
    (*last).Flink = entry;
    (*head).Blink = entry;
}
#[inline]
pub unsafe fn RemoveEntryList(entry: PLIST_ENTRY) {
    let flink = (*entry).Flink;
    let blink = (*entry).Blink;
    (*blink).Flink = flink;
    (*flink).Blink = blink;
}

// RtlGetProcessHeap() is a macro (`NtCurrentPeb()->ProcessHeap`) — not a
// bindgen symbol. GetProcessHeap() returns the same PEB->ProcessHeap
// handle, without the arch-specific NtCurrentPeb() segment read.
#[allow(non_snake_case)]
#[inline]
pub unsafe fn RtlGetProcessHeap() -> PVOID {
    GetProcessHeap() as PVOID
}

// CRT (cdecl) wide-string helpers the leaves call — not Win32 APIs, so
// not in winlogon-sys's bindgen. winlogon.exe links msvcrt.
extern "C" {
    pub fn wcscpy(dest: *mut u16, src: *const u16) -> *mut u16;
    pub fn wcscat(dest: *mut u16, src: *const u16) -> *mut u16;
    pub fn wcslen(s: *const u16) -> usize;
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16;
    pub fn wcsncpy(dest: *mut u16, src: *const u16, n: usize) -> *mut u16;
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32;
    pub fn wcscmp(s1: *const u16, s2: *const u16) -> i32;
    // `int iswspace(wint_t)`. Declared with i32 arg (not u16) so the i386
    // cdecl call pushes a clean zero-extended 32-bit slot — the CRT reads
    // its wint_t param from a 4-byte stack slot.
    pub fn iswspace(c: i32) -> i32;
    // Variadic CRT formatter — Rust can't *define* a C variadic but may
    // *call* one. `int swprintf(wchar_t*, const wchar_t* fmt, ...)`.
    pub fn swprintf(buffer: *mut u16, format: *const u16, ...) -> i32;
    // user32 `int WINAPIV wsprintfW(LPWSTR, LPCWSTR fmt, ...)` — WINAPIV is
    // __cdecl for varargs, so extern "C" is the correct ABI on all arches.
    pub fn wsprintfW(out: *mut u16, fmt: *const u16, ...) -> i32;
}
