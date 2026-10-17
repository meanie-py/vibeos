//! Cargo-driven `services.exe` (SCM) link (i386 + amd64 + arm64). Manifest for
//! the shared reactos-link engine. Fourth spine binary moved cmake→cargo
//! (after lsass + userinit + winlogon); the winlogon twin (partial flip + C
//! residue via c_from_compile_db) but with a WIDE entry.
//!
//! Mirrors the cmake services.exe link (build-<arch>/build.ninja):
//!   subsystem windows:5.01 ; entry _wWinMainCRTStartup (i386) / undecorated
//!   (amd64/arm64) — win32gui UNICODE, wide wWinMain ; --gc-sections ; auto
//!   image base.
//!   inputs: libservices_rust.a (the 8 flipped SCM .c) + the still-C residue
//!   (services.c main + services_rust_stub.c + the GENERATED svcctl_s RPC
//!   stub) + services.rc.res + libpseh/libchkstk/libmsvcrtex/oldnames + the 8
//!   import libs (userenv/user32/advapi32/advapi32_vista/rpcrt4/msvcrt/
//!   kernel32/ntdll).
//!
//! The C residue is replayed flag-exact from compile_commands.json (the cmake
//! `services_c` OBJECT lib) — including the generated `svcctl_s.c` (build-dir
//! source). Import libs are generated in-cargo from each .spec. CRT statics +
//! the load-bearing .res (localized STRINGTABLEs) are reused from cmake.

use reactos_link::{Linker, ModuleKind, MsvcrtexKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");

    // The module's part of cmake's windres command; reactos-link adds what every
    // module shares from reactos-link/cflags/rc.<arch>.flags (Linker::rc).
    let rc_includes: &[&str] = &[
        "@build/base/system/services",
        "base/system/services",
        "@std",
        "sdk/lib/pseh/include",
        "vendor/opengl-headers/include",
        "vendor/libdxg/include",
        "sdk/include/reactos/subsys",
        "sdk/include/reactos/idl",
    ];
    let rc_flags: &[&str] = &[];
    let rc_defines: &[&str] = &["UNICODE", "WINVER=0x502", "_UNICODE", "_WIN32_WINNT=0x502"];

    // services is win32gui UNICODE → wide wWinMain (like lsass/userinit).
    let (target_json, entry) = match arch.as_str() {
        "x86" => ("i686-pc-windows-gnu", "_wWinMainCRTStartup"),
        "x86_64" => ("x86_64-pc-windows-gnu", "wWinMainCRTStartup"),
        "aarch64" => ("aarch64-pc-windows-gnullvm", "wWinMainCRTStartup"),
        o => panic!("services-cargo: unsupported arch {o}"),
    };

    Linker::new(target_json)
        .rust_staticlib("services-rust")
        // services.exe (SCM) is now a FULL Rust flip — services.c (the wWinMain
        // SCM bootstrap, the last C file) is translated to services-rust/src/
        // services.rs, so the link compiles ZERO C residue. No c_from_compile_db.
        // The svcctl RPC server stub is pure RUST: the svcctl-stub crate
        // transliterates widl's generated `svcctl_s.c` to Rust (against midl-rt)
        // at build time and exports `svcctl_v2_0_s_ifspec`. No C stub linked.
        .rust_lib("svcctl-stub", &[], true)
        // Load-bearing resource (localized STRINGTABLEs + version), compiled
        // IN-cargo by replaying the services_res windres command from
        // compile_commands.json.
        .rc("base/system/services/services.rc", rc_includes, rc_flags, rc_defines)
        // 8 import libs generated in-cargo from each .spec.
        .import_lib("dll/win32/userenv/userenv.spec", "userenv.dll", "userenv")
        .import_lib("win32ss/user/user32/user32.spec", "user32.dll", "user32")
        .import_lib("dll/win32/advapi32/advapi32.spec", "advapi32.dll", "advapi32")
        .import_lib(
            "dll/win32/advapi32_vista/advapi32_vista.spec",
            "advapi32_vista.dll",
            "advapi32_vista",
        )
        .import_lib("dll/win32/rpcrt4/rpcrt4.spec", "rpcrt4.dll", "rpcrt4")
        .import_lib("dll/win32/msvcrt/msvcrt.spec", "msvcrt.dll", "msvcrt")
        .import_lib("dll/win32/kernel32/kernel32.spec", "kernel32.dll", "kernel32")
        .import_lib("dll/ntdll/def/ntdll.spec", "ntdll.dll", "ntdll")
        // ZERO cmake .a: libpseh + chkstk + oldnames all dropped — each proved
        // INERT via the residue probe (nm -u <body> ∩ nm <.a> ' T ' = empty ×3).
        // The body emits no PSEH (DUMMY_PSEH), no undefined stack-probe (i386
        // chkstk now rides the msvcrtex_asm archive), and references nothing from
        // the oldnames CRT aliases. libmsvcrtex (CRT startup) is now Rust via
        // .msvcrtex_crt(MsvcrtexKind::ExeWide).
        .msvcrtex_crt(MsvcrtexKind::ExeWide)
        .link(ModuleKind::Win32Exe {
            entry: entry.into(),
            subsystem: "windows:5.01".into(),
        });
}
