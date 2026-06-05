//! Cargo-driven `winsrv.dll` link (i386 + amd64 + arm64). Manifest for the
//! shared reactos-link engine. Seventh spine binary moved cmake→cargo (after
//! lsass + userinit + winlogon + services + smss + csrsrv) — the last of the
//! SMSS/CSRSS spine, and the one with the largest C residue.
//!
//! Mirrors the cmake winsrv.dll link (build-<arch>/build.ninja):
//!   subsystem console:5.01 ; entry _DllMain@12 (i386) / DllMain (amd64/arm64) ;
//!   -shared ; keep all sections ; exports via the spec2def-generated
//!   winsrv.def ; image base 0x78860000 (i386) / 0x7ff86000000 (amd64/arm64).
//!
//! winsrv is a PARTIAL flip: init.c is Rust (winsrv-rust, `winsrv_dll_only`)
//! but the consrv + usersrv + concfg C component libraries stay C. Rather than
//! c_from_compile_db every consrv/usersrv source (Axis B), this reuses the
//! whole cmake-built component .a (libconsrv/libusersrv/libconcfg/
//! libconcfg_rust/libwin32ksys) + the import libs via static_libs — moving
//! just the LINK. winsrv-rust pulls concfg-rust as a Cargo dep, so
//! libconcfg_rust.a overlaps (kept for link parity; --allow-multiple-definition
//! covers it). winsrv-rust's only default feature ntddi_ws03 stays on
//! (additive) → exactly cmake's `--no-default-features --features
//! winsrv_dll_only,ntddi_ws03`. winsrv.rc/.res is consumed by the cmake link
//! today via the .def path; the resource is reused below.

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "_DllMain@12", 0x7886_0000),
        "x86_64" => ("x86_64-kernel.json", "DllMain", 0x7ff_8600_0000),
        "aarch64" => ("aarch64-kernel.json", "DllMain", 0x7ff_8600_0000),
        o => panic!("winsrv-cargo: unsupported arch {o}"),
    };

    Linker::new(target_json)
        // winsrv_dll_only gates init.rs's DllMain/GetTimeouts/ShutdownSettings.
        // ntddi_ws03 stays on as the default (additive) → matches cmake's
        // `--no-default-features --features winsrv_dll_only,ntddi_ws03`.
        // Win32Dll → user-mode target (winsrv-rust builds i686-pc-windows-gnu,
        // pulling concfg-rust as a Cargo dep).
        .rust_staticlib("winsrv-rust")
        .rust_features(&["winsrv_dll_only"])
        // winsrv-rust has cross-module extern "C"-def / extern "system"-call
        // ABI mismatches (e.g. HistoryReshapeAllBuffers: history.rs defines it
        // extern "C", settings.rs calls it extern "system"). The cmake link
        // resolves these via lld's stdcall-fixup (it passes no
        // --disable-stdcall-fixup); match that so the cargo link is equivalent.
        .allow_stdcall_fixup()
        // Exports via spec2def-generated winsrv.def (ConServerDllInitialization
        // / UserServerDllInitialization / _UserSoundSentry / …).
        .spec2def("win32ss/user/winsrv/winsrv.spec", "winsrv.dll")
        // The C component libs + leaf statics + import libs, reused from cmake
        // in the cmake link order (libwinsrv_rust.a is emitted first by the
        // engine, so it is omitted here). --allow-multiple-definition (added by
        // the engine) covers the concfg-rust / cntpr overlaps.
        .static_libs(&[
            "win32ss/user/winsrv/libconsrv.a",
            "win32ss/user/winsrv/libusersrv.a",
            "win32ss/user/winsrv/concfg/libconcfg.a",
            "sdk/lib/uuid/libuuid.a",
            "win32ss/libwin32ksys.a",
            "sdk/lib/crt/liblibcntpr.a",
            "sdk/lib/pseh/libpseh.a",
            "dll/win32/shell32/CMakeFiles/libshell32.dir/libshell32.a",
            "dll/win32/ole32/CMakeFiles/libole32.dir/libole32.a",
            "dll/win32/psapi/CMakeFiles/libpsapi.dir/libpsapi.a",
            "dll/win32/advapi32/CMakeFiles/libadvapi32.dir/libadvapi32.a",
            "subsystems/win/basesrv/CMakeFiles/libbasesrv.dir/libbasesrv.a",
            "subsystems/csr/csrsrv/CMakeFiles/libcsrsrv.dir/libcsrsrv.a",
            "win32ss/gdi/gdi32/CMakeFiles/libgdi32.dir/libgdi32.a",
            "win32ss/user/user32/CMakeFiles/libuser32.dir/libuser32.a",
            "dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a",
            "dll/ntdll/CMakeFiles/libntdll.dir/libntdll.a",
            "sdk/lib/crt/libchkstk.a",
            "win32ss/user/winsrv/concfg/libconcfg_rust.a",
            "sdk/lib/crt/libcntpr_rust.a",
        ])
        .link(ModuleKind::Win32Dll {
            entry: entry.into(),
            image_base,
            subsystem: Some("console:5.01".into()),
        });
}
