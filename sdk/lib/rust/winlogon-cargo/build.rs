//! Cargo-driven `winlogon.exe` link (i386 + amd64 + arm64). Manifest for the
//! shared reactos-link engine. Third spine binary moved cmake→cargo (after
//! lsass + userinit); the first whose body is a PARTIAL flip, so the still-C
//! residue is compiled IN-cargo via `c_from_compile_db`.
//!
//! Mirrors the cmake winlogon.exe link (build-<arch>/build.ninja):
//!   subsystem windows:5.01 ; entry _WinMainCRTStartup (i386) / undecorated
//!   (amd64/arm64) — ANSI WinMain, win32gui WITHOUT UNICODE ; --gc-sections ;
//!   auto image base.
//!   inputs: libwinlogon_rust.a (the Rust leaves) + the still-C residue
//!   (rpcserver/sas/winlogon/wlx/winlogon_rust_stub + the GENERATED winreg_s
//!   RPC stub) + winlogon.rc.res + libwine/libpseh/libchkstk/libmsvcrtex/
//!   oldnames + the 10 import libs (gdi32/user32/advapi32/userenv/secur32/
//!   rpcrt4/mpr/msvcrt/kernel32/ntdll).
//!
//! The C residue is replayed flag-exact from compile_commands.json (the cmake
//! `winlogon` target) — including the generated `winreg_s.c` (build-dir
//! source; the engine resolves it under build-<arch>/). Import libs are
//! generated in-cargo from each .spec. libwine/libpseh/CRT statics + the
//! load-bearing .res are reused from cmake.

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    // winlogon is set_module_type win32gui WITHOUT UNICODE, so its body uses
    // ANSI `WinMain` (`_WinMain@16`) and the CRT entry is `WinMainCRTStartup`
    // (NOT the wide `wWinMainCRTStartup` that lsass/userinit use).
    let (target_json, entry) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "_WinMainCRTStartup"),
        "x86_64" => ("x86_64-kernel.json", "WinMainCRTStartup"),
        "aarch64" => ("aarch64-kernel.json", "WinMainCRTStartup"),
        o => panic!("winlogon-cargo: unsupported arch {o}"),
    };

    Linker::new(target_json)
        .rust_staticlib("winlogon-rust")
        // Still-C residue of the partial flip, compiled in-cargo by replaying
        // cmake's clang (the `winlogon_c` OBJECT lib).
        // winlogon_rust_stub.c dropped — cmake-era empty cohabitation stub
        // (archive-extraction hack), vestigial in the cargo link. rpcserver.c +
        // winlogon.c + wlx.c flipped → winlogon-rust/src/{rpcserver,winlogon,
        // wlx}.rs. wlx.rs owns GinaInit/CreateWindowStationAndDesktops/
        // InitDialogListHead/CloseAllDialogWindows (called by the still-C sas.c +
        // the Rust winlogon.rs); sas.c owns LuidNone (wlx.rs externs it). Only
        // sas.c (the SAS state machine) remains C.
        .c_from_compile_db("winlogon_c", &["base/system/winlogon/sas.c"])
        // The winreg RPC server stub is now pure RUST: the winreg-stub crate
        // transliterates widl's generated `winreg_s.c` to Rust (against midl-rt)
        // at build time and exports `winreg_v1_0_s_ifspec`. No C stub linked.
        .rust_lib("winreg-stub", &[], true)
        // Load-bearing resource (winlogon.ico app icon), compiled IN-cargo by
        // replaying the winlogon_res windres command from compile_commands.json.
        .resource_from_rc("base/system/winlogon/winlogon.rc", "winlogon_res")
        // 10 import libs generated in-cargo from .spec.
        .import_lib("win32ss/gdi/gdi32/gdi32.spec", "gdi32.dll", "gdi32")
        .import_lib("win32ss/user/user32/user32.spec", "user32.dll", "user32")
        .import_lib("dll/win32/advapi32/advapi32.spec", "advapi32.dll", "advapi32")
        .import_lib("dll/win32/userenv/userenv.spec", "userenv.dll", "userenv")
        .import_lib("dll/win32/secur32/secur32.spec", "secur32.dll", "secur32")
        .import_lib("dll/win32/rpcrt4/rpcrt4.spec", "rpcrt4.dll", "rpcrt4")
        .import_lib("dll/win32/mpr/mpr.spec", "mpr.dll", "mpr")
        .import_lib("dll/win32/msvcrt/msvcrt.spec", "msvcrt.dll", "msvcrt")
        .import_lib("dll/win32/kernel32/kernel32.spec", "kernel32.dll", "kernel32")
        .import_lib("dll/ntdll/def/ntdll.spec", "ntdll.dll", "ntdll")
        // libwine (3rdparty C) + libpseh + CRT statics, reused from cmake.
        .static_libs(&[
            "sdk/lib/3rdparty/libwine/libwine.a",
            "sdk/lib/pseh/libpseh.a",
            "sdk/lib/crt/libchkstk.a",
            "sdk/lib/crt/libmsvcrtex.a",
            "sdk/lib/crt/CMakeFiles/oldnames.dir/oldnames.a",
        ])
        .link(ModuleKind::Win32Exe {
            entry: entry.into(),
            subsystem: "windows:5.01".into(),
        });
}
