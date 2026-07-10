//! userinit-rust — Rust translation of userinit.exe, the post-login shell
//! launcher (winlogon → userinit → explorer.exe). Successor to the smss/csrss
//! user-mode spine flips; the last hop before the desktop.
//!
//! Full flip: both userinit.c (StartShell / RunOnce / shell selection) and
//! livecd.c (LiveCD locale + start dialogs, auto-advanced under `/MININT`).
//! Substrate is winsrv-sys (user32/gdi32/advapi32/kernel32 superset over
//! ntdll-sys). Per-binary cohabitation: each `.c` is commented out of
//! base/system/userinit/CMakeLists.txt once Rust parity is verified; the C
//! stays on disk (translation philosophy: never delete the C).

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code)]

// Shared header types (userinit.h structs/enums + LIVECD_UNATTEND), dropped
// by the per-file translation. Declared first so both modules can `use` them.
pub mod types;

// Win32 macros, resource IDs, registry paths, and hoisted wide-string
// literals the translation dropped/mis-emitted. Shared by both modules.
pub mod preamble;

// userinit.c — wWinMain entry, StartShell, IsLiveCD, SetUserSettings.
pub mod userinit;

// livecd.c — RunLiveCD + LocaleDlgProc/StartDlgProc + ParseUnattend + InitLogo.
pub mod livecd;

// USER-MODE no_std staticlib → needs its own panic handler + eh_personality
// (same role as smss-csrss-rust / winsrv-rust). Must NOT use privileged
// instructions; abort via the Native API instead.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}
