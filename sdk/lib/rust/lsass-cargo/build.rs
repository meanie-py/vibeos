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

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "_wWinMainCRTStartup"),
        o => panic!("lsass-cargo: unsupported arch {o} (i386 first mover)"),
    };

    Linker::new(target_json)
        // Body: cargo-build lsass-rust (user-mode target, default features).
        .rust_staticlib("lsass-rust")
        // NOTE: lsass.rc (pure version info — FILE_DESCRIPTION + version.rc, no
        // manifest/icons, non-load-bearing) is omitted in this first mover:
        // retiring cmake's add_executable removes its .rc compile rule, and
        // re-hosting it (gen_lsass_res.py or a cmake rc target) is owed follow-up.
        // Importlib group + CRT, reused from cmake (build-i386 relative). The
        // engine wraps these + the Rust .a in one --start-group, so the circular
        // refs (CRT entry → wWinMain → Rtl*/kernel32) resolve. libmsvcrtex.a is
        // the "hidden" CRT startup that provides _wWinMainCRTStartup.
        .static_libs(&[
            "dll/win32/advapi32/CMakeFiles/libadvapi32.dir/libadvapi32.a",
            "dll/win32/lsasrv/CMakeFiles/liblsasrv.dir/liblsasrv.a",
            "dll/win32/samsrv/CMakeFiles/libsamsrv.dir/libsamsrv.a",
            "dll/win32/msvcrt/CMakeFiles/libmsvcrt.dir/libmsvcrt.a",
            "dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a",
            "dll/ntdll/CMakeFiles/libntdll.dir/libntdll.a",
            "sdk/lib/crt/libmsvcrtex.a",
            "sdk/lib/crt/CMakeFiles/oldnames.dir/oldnames.a",
        ])
        .link(ModuleKind::Win32Exe {
            entry: entry.into(),
            subsystem: "windows:5.01".into(),
        });
}
