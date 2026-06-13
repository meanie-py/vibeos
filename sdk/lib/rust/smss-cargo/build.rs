//! Cargo-driven `smss.exe` (Session Manager) link (i386 + amd64 + arm64).
//! Manifest for the shared reactos-link engine. Fifth spine binary moved
//! cmake→cargo (after lsass + userinit + winlogon + services); the first
//! NATIVE-subsystem one (ModuleKind::NativeExe, like rosload).
//!
//! Mirrors the cmake smss.exe link (build-<arch>/build.ninja):
//!   subsystem native:5.01 ; entry _NtProcessStartup@4 (i386) / NtProcessStartup
//!   (amd64/arm64) ; --gc-sections ; image base 0x400000 (i386) /
//!   0x140000000 (amd64/arm64).
//!   inputs: libsmss_csrss_rust.a (FULL flip — all 9 smss .c, built
//!   `smss_exe_only`) + libpseh/libsmlib/libchkstk + the ntdll import lib.
//!   (libnt's _NtProcessStartup@4 is now a Rust port in smss-csrss-rust →
//!   crate::entry_point, so libnt.a is no longer linked.)
//!
//! smss is a FULL flip (all 9 .c → Rust; the lone C `smss_rust_stub.c` is an
//! empty TU, omitted). The body crate smss-csrss-rust builds the STANDARD
//! user-mode target (.rust_user_target — hardfloat, like lsass; NOT the kernel
//! json) with `--features smss_exe_only`. That feature compiles only smss/* +
//! smlib and excludes csrsrv/csrlib/csrss/basesrv (no duplicate `_main`, no
//! stray csrsrv data statics). The crate's lone default feature `ntddi_ws03`
//! stays on (additive), so this is exactly cmake's
//! `--no-default-features --features smss_exe_only,ntddi_ws03`. smss.rc is
//! version-only → .res omitted (like lsass).

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "_NtProcessStartup@4", 0x40_0000),
        "x86_64" => ("x86_64-kernel.json", "NtProcessStartup", 0x1_4000_0000),
        "aarch64" => ("aarch64-kernel.json", "NtProcessStartup", 0x1_4000_0000),
        o => panic!("smss-cargo: unsupported arch {o}"),
    };

    Linker::new(target_json)
        .rust_staticlib("smss-csrss-rust")
        // smss_exe_only: only smss/* + smlib; excludes csrsrv/csrlib/csrss/
        // basesrv. ntddi_ws03 stays on as the default (additive) → matches
        // cmake's `--no-default-features --features smss_exe_only,ntddi_ws03`.
        .rust_features(&["smss_exe_only"])
        // User-mode native app: standard hardfloat target (like lsass), NOT the
        // kernel json — NativeExe would otherwise build the kernel json.
        .rust_user_target()
        .import_lib("dll/ntdll/def/ntdll.spec", "ntdll.dll", "ntdll")
        // libnt.a DROPPED — the native CRT startup _NtProcessStartup@4 is now a
        // Rust port in smss-csrss-rust (crate::entry_point, smss_exe_only). pseh,
        // the Sm* client lib (libsmlib, real C debt), and chkstk are still reused
        // from cmake. No msvcrtex/oldnames (native exe, minimal CRT).
        .static_libs(&[
            "sdk/lib/pseh/libpseh.a",
            "sdk/lib/smlib/libsmlib.a",
            "sdk/lib/crt/libchkstk.a",
        ])
        .link(ModuleKind::NativeExe {
            entry: entry.into(),
            image_base,
        });
}
