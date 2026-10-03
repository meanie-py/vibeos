//! Cargo-driven `userinit.exe` link (i386 + amd64 + arm64). Manifest for the
//! shared reactos-link engine. Second spine binary moved cmake→cargo (after
//! lsass); the Win32Exe sibling with a LOAD-BEARING resource.
//!
//! Mirrors the cmake userinit.exe link (build-<arch>/build.ninja):
//!   subsystem windows:5.01 ; entry _wWinMainCRTStartup (i386) / undecorated
//!   (amd64/arm64) ; --gc-sections ; auto image base.
//!   inputs: libuserinit_rust.a (wWinMain) + userinit.rc.res + libwine.a +
//!   the 7 import libs (user32/gdi32/advapi32/shell32/msvcrt/kernel32/ntdll) +
//!   CRT (libmsvcrtex/oldnames).
//!
//! Import libs are GENERATED in-cargo from each .spec (.import_lib). libwine.a
//! (3rdparty C) and the CRT statics are reused from cmake. The .res is reused
//! from cmake (resource-only OBJECT lib userinit_res) — it is load-bearing
//! (manifest + logo bitmaps + dialogs), so unlike lsass it is NOT omitted.

use reactos_link::{Linker, ModuleKind, MsvcrtexKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "_wWinMainCRTStartup"),
        "x86_64" => ("x86_64-pc-windows-gnu", "wWinMainCRTStartup"),
        "aarch64" => ("aarch64-pc-windows-gnullvm", "wWinMainCRTStartup"),
        o => panic!("userinit-cargo: unsupported arch {o}"),
    };

    Linker::new(target_json)
        .rust_staticlib("userinit-rust")
        // Load-bearing resource (manifest + IDB_ROSLOGO bitmaps + localized
        // dialogs), compiled IN-cargo by replaying the userinit_res windres
        // command from compile_commands.json (drops the cmake-built .res reuse).
        .resource_from_rc("base/system/userinit/userinit.rc", "userinit_res")
        // 7 import libs generated in-cargo from .spec.
        .import_lib("win32ss/user/user32/user32.spec", "user32.dll", "user32")
        .import_lib("win32ss/gdi/gdi32/gdi32.spec", "gdi32.dll", "gdi32")
        .import_lib("dll/win32/advapi32/advapi32.spec", "advapi32.dll", "advapi32")
        .import_lib("dll/win32/userenv/userenv.spec", "userenv.dll", "userenv")
        .import_lib("dll/win32/shell32/shell32.spec", "shell32.dll", "shell32")
        .import_lib("dll/win32/msvcrt/msvcrt.spec", "msvcrt.dll", "msvcrt")
        .import_lib("dll/win32/kernel32/kernel32.spec", "kernel32.dll", "kernel32")
        .import_lib("dll/ntdll/def/ntdll.spec", "ntdll.dll", "ntdll")
        // ZERO cmake .a: libwine.a + oldnames.a dropped — both proved INERT via
        // the residue probe (nm -u <body> ∩ nm <.a> ' T ' = empty ×3). The body's
        // undefined symbols are all satisfied by the import libs + compiler-builtins;
        // it referenced nothing from wine's debug-channel stubs or the oldnames
        // CRT aliases. libmsvcrtex (CRT startup) is now Rust via
        // .msvcrtex_crt(MsvcrtexKind::ExeWide).
        .msvcrtex_crt(MsvcrtexKind::ExeWide)
        .link(ModuleKind::Win32Exe {
            entry: entry.into(),
            subsystem: "windows:5.01".into(),
        });
}
