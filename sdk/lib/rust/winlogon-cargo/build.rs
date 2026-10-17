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

use reactos_link::{Linker, ModuleKind, MsvcrtexKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");

    // The module's part of cmake's windres command; reactos-link adds what every
    // module shares from reactos-link/cflags/rc.<arch>.flags (Linker::rc).
    let rc_includes: &[&str] = &[
        "@build/base/system/winlogon",
        "base/system/winlogon",
        "@std",
        "sdk/lib/pseh/include",
        "vendor/opengl-headers/include",
        "vendor/libdxg/include",
        "sdk/include/reactos/idl",
    ];
    let rc_flags: &[&str] = &[];
    let rc_defines: &[&str] = &["WINVER=0x502", "_WIN32_WINNT=0x502"];

    // winlogon is set_module_type win32gui WITHOUT UNICODE, so its body uses
    // ANSI `WinMain` (`_WinMain@16`) and the CRT entry is `WinMainCRTStartup`
    // (NOT the wide `wWinMainCRTStartup` that lsass/userinit use).
    let (target_json, entry) = match arch.as_str() {
        "x86" => ("i686-pc-windows-gnu", "_WinMainCRTStartup"),
        "x86_64" => ("x86_64-pc-windows-gnu", "WinMainCRTStartup"),
        "aarch64" => ("aarch64-pc-windows-gnullvm", "WinMainCRTStartup"),
        o => panic!("winlogon-cargo: unsupported arch {o}"),
    };

    Linker::new(target_json)
        .rust_staticlib("winlogon-rust")
        // Still-C residue of the partial flip, compiled in-cargo by replaying
        // cmake's clang (the `winlogon_c` OBJECT lib).
        // winlogon.exe is now a FULL Rust flip — ALL 4 GINA/SAS/RPC cores
        // (rpcserver/winlogon/wlx/sas) are Rust, so the link compiles ZERO C.
        // sas.rs owns LuidNone (wlx.rs externs it); the winreg RPC server stub
        // is the Rust winreg-stub crate. No c_from_compile_db.
        // The winreg RPC server stub is now pure RUST: the winreg-stub crate
        // transliterates widl's generated `winreg_s.c` to Rust (against midl-rt)
        // at build time and exports `winreg_v1_0_s_ifspec`. No C stub linked.
        .rust_lib("winreg-stub", &[], true)
        // Load-bearing resource (winlogon.ico app icon), compiled IN-cargo by
        // replaying the winlogon_res windres command from compile_commands.json.
        .rc("base/system/winlogon/winlogon.rc", rc_includes, rc_flags, rc_defines)
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
        // ZERO cmake .a: libwine + libpseh + chkstk + oldnames all dropped —
        // each proved INERT via the residue probe (nm -u <body> ∩ nm <.a> ' T '
        // = empty ×3). The body emits no PSEH (DUMMY_PSEH), no undefined
        // stack-probe (i386 chkstk now rides the msvcrtex_asm archive), and
        // references nothing from wine's debug stubs or the oldnames CRT aliases.
        // libmsvcrtex (CRT startup) is now Rust via
        // .msvcrtex_crt(MsvcrtexKind::ExeAnsi).
        .msvcrtex_crt(MsvcrtexKind::ExeAnsi)
        .link(ModuleKind::Win32Exe {
            entry: entry.into(),
            subsystem: "windows:5.01".into(),
        });
}
