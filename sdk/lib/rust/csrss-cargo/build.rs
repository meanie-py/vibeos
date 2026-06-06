//! Cargo-driven `csrss.exe` link (i386 + amd64 + arm64). Manifest for the shared
//! reactos-link engine. The LAST csrss-spine binary moved cmake→cargo, and (like
//! basesrv) the one still on the legacy pattern: cmake `add_executable(csrss)` +
//! a standalone `add_custom_command` cargo build whose `.a` the link reused.
//!
//! Mirrors the cmake csrss.exe link (build-<arch>/build.ninja):
//!   subsystem native:5.01 ; entry _NtProcessStartup@4 (i386) / NtProcessStartup
//!   (amd64/arm64) ; --gc-sections ; auto image base (lld resolves 0x400000 i386
//!   / 0x140000000 amd64). arm64 reserves a 2MB stack (deep LdrpLoadDll chain).
//!
//! csrss.c is flipped to Rust (smss-csrss-rust/src/csrss.rs); the only C residue
//! is csrss_rust_stub.c, compiled IN-cargo (c_from_compile_db). The LEAN
//! `csrss_main_only` feature ships ONLY `_main` + CsrpSetDefaultProcessHardError
//! Mode (additive over the sole default ntddi_ws03 → exactly cmake's
//! `--no-default-features --features csrss_main_only,ntddi_ws03`) so csrss.exe
//! does NOT carry its own copies of csrsrv.dll's data exports. NativeExe builds
//! the user hardfloat target (.rust_user_target), like smss. csrss.rc is
//! version-only (non-load-bearing) → omitted. csrsrv + ntdll import libs
//! self-hosted; libnt.a reused.

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "_NtProcessStartup@4", 0x40_0000),
        "x86_64" => ("x86_64-kernel.json", "NtProcessStartup", 0x1_4000_0000),
        "aarch64" => ("aarch64-kernel.json", "NtProcessStartup", 0x1_4000_0000),
        o => panic!("csrss-cargo: unsupported arch {o}"),
    };

    let mut linker = Linker::new(target_json)
        // csrss_main_only → lean archive (_main + CsrpSetDefaultProcessHardError
        // Mode only). ntddi_ws03 stays on (additive). NativeExe would otherwise
        // build the kernel json — csrss is user hardfloat like smss/lsass.
        .rust_staticlib("smss-csrss-rust")
        .rust_features(&["csrss_main_only"])
        .rust_user_target()
        // csrss_rust_stub.c macro/intrinsic shim compiled IN-cargo (replays the
        // csrss_c cmake compile command; SKIP_PRECOMPILE_HEADERS, self-contained).
        .c_from_compile_db("csrss_c", &["subsystems/csr/csrss/csrss_rust_stub.c"])
        // csrsrv + ntdll import libs self-hosted from each .spec.
        .import_lib("subsystems/csr/csrsrv/csrsrv.spec", "csrsrv.dll", "csrsrv")
        .import_lib("dll/ntdll/def/ntdll.spec", "ntdll.dll", "ntdll")
        // libnt.a (native CRT startup _NtProcessStartup@4) reused from cmake.
        .static_libs(&["sdk/lib/nt/libnt.a"]);

    // arm64 only: llvm-dlltool collapses delay-imports to regular imports, so
    // every dependency loads at once and the LdrpLoadDll recursion overflows the
    // default 1MB stack. Reserve 2MB (cmake's `-Wl,--stack,2097152`).
    if arch == "aarch64" {
        linker = linker.stack_reserve(2 * 1024 * 1024);
    }

    linker.link(ModuleKind::NativeExe {
        entry: entry.into(),
        image_base,
    });
}
