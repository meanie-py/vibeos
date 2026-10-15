//! Cargo-driven `lsass.exe` link (i386 first mover). Manifest for the shared
//! `reactos-link` engine. User-mode Win32 GUI exe — first of OUR translated
//! spine binaries to move its link cmake→cargo.
//!
//! Mirrors the cmake lsass.exe link (from build-i386/build.ninja, target
//! `base/system/lsass/lsass.exe`):
//!   subsystem  `windows:5.01`            (set_module_type win32gui)
//!   entry      `_wWinMainCRTStartup`     (CRT startup in libmsvcrtex.a)
//!   sections   `--gc-sections`, NOT -shared (it's an exe), no exports
//!   image base auto (`--enable-auto-image-base`; cmake passes no --image-base)
//!   inputs     liblsass_rust.a (wWinMain) + lsass.rc.res + the importlib group
//!              advapi32/lsasrv/samsrv/msvcrt/kernel32/ntdll + libmsvcrtex.a
//!              (CRT) + oldnames.a.
//!
//! The importlibs/CRT/.res are REUSED from cmake (build-i386/...; run `ninja`
//! once first) — first-pass like rosload-cargo. The Rust body IS cargo-built
//! (`.rust_staticlib("lsass-rust")`, user-mode target). amd64/arm64 follow
//! (entry undecorated there; subsystem identical).

use reactos_link::{Linker, ModuleKind, MsvcrtexKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    // Subsystem (windows:5.01) is identical on all arches; only the CRT entry's
    // decoration differs — i386 prefixes `_`, amd64/arm64 are undecorated.
    let (target_json, entry) = match arch.as_str() {
        "x86" => ("i686-pc-windows-gnu", "_wWinMainCRTStartup"),
        "x86_64" => ("x86_64-pc-windows-gnu", "wWinMainCRTStartup"),
        "aarch64" => ("aarch64-pc-windows-gnullvm", "wWinMainCRTStartup"),
        o => panic!("lsass-cargo: unsupported arch {o}"),
    };

    Linker::new(target_json)
        // Body: cargo-build lsass-rust (user-mode target, default features).
        .rust_staticlib("lsass-rust")
        // NOTE: lsass.rc (pure version info — FILE_DESCRIPTION + version.rc, no
        // manifest/icons, non-load-bearing) is omitted in this first mover:
        // retiring cmake's add_executable removes its .rc compile rule, and
        // re-hosting it (gen_lsass_res.py or a cmake rc target) is owed follow-up.
        // Import libs GENERATED in-cargo from each DLL's .spec (spec2def +
        // dlltool, self-hosted by the engine) — no cmake importlib reuse, no
        // `ninja` prereq. These supply lsass's imports: advapi32 (LSA helpers),
        // lsasrv (LsapInitLsa/ServiceInit), samsrv (SamIInitialize), msvcrt
        // (CRT), kernel32, ntdll.
        .import_lib("dll/win32/advapi32/advapi32.spec", "advapi32.dll", "advapi32")
        .import_lib("dll/win32/lsasrv/lsasrv.spec", "lsasrv.dll", "lsasrv")
        .import_lib("dll/win32/samsrv/samsrv.spec", "samsrv.dll", "samsrv")
        .import_lib("dll/win32/msvcrt/msvcrt.spec", "msvcrt.dll", "msvcrt")
        .import_lib("dll/win32/kernel32/kernel32.spec", "kernel32.dll", "kernel32")
        .import_lib("dll/ntdll/def/ntdll.spec", "ntdll.dll", "ntdll")
        // oldnames.a (_name aliases) still reused from cmake; the CRT startup
        // (libmsvcrtex.a — _wWinMainCRTStartup → wWinMain) is now compiled
        // in-cargo via .msvcrtex_crt(MsvcrtexKind::ExeWide) (msvcrtex.cmake sources replayed).
        // oldnames.a (CRT _name aliases) DROPPED: nm -u on liblsass_rust.a ×3
        // shows zero underscore-oldname refs (the Rust body calls the canonical
        // CRT names via the msvcrt import lib). lsass.exe is a NON-shared exe
        // link (--gc-sections, no -shared) so a clean cold link is itself proof
        // the alias bridge is unreferenced. CRT startup stays in-cargo below.
        .msvcrtex_crt(MsvcrtexKind::ExeWide)
        .link(ModuleKind::Win32Exe {
            entry: entry.into(),
            subsystem: "windows:5.01".into(),
        });
}
