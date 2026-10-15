//! Cargo-driven `basesrv.dll` link (i386 + amd64 + arm64). Manifest for the
//! shared reactos-link engine. The LAST csrss-spine server moved cmake→cargo,
//! and the one still on the pre-Axis-A pattern (cmake `add_library(basesrv
//! MODULE)` + a standalone `add_custom_command` cargo build whose `.a` the link
//! reused). Now driven end-to-end by reactos-link, like csrsrv/winsrv.
//!
//! Mirrors the cmake basesrv.dll link (build-<arch>/build.ninja):
//!   subsystem console:5.01 ; entry _DllMain@12 (i386) / DllMain (amd64/arm64) ;
//!   -shared ; keep all sections ; exports via the spec2def-generated
//!   basesrv.def ; image base 0x78960000 (i386) / 0x7ff87000000 (amd64/arm64).
//!
//! basesrv is a FULL Rust flip (smss-csrss-rust/src/basesrv/*, basesrv_dll_only
//! feature — all 7 .c gone). The only C residue is basesrv_rust_stub.c (the
//! macro/intrinsic shim) and basesrv.rc, both compiled IN-cargo here
//! (c_from_compile_db + resource_from_rc). basesrv_dll_only is additive over the
//! sole default ntddi_ws03 → exactly cmake's `--no-default-features --features
//! basesrv_dll_only,ntddi_ws03`. csrsrv + ntdll import libs self-hosted from
//! each .spec; chkstk + pseh leaf statics reused.

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-pc-windows-gnu", "_DllMain@12", 0x7896_0000),
        "x86_64" => ("x86_64-pc-windows-gnu", "DllMain", 0x7ff_8700_0000),
        "aarch64" => ("aarch64-pc-windows-gnullvm", "DllMain", 0x7ff_8700_0000),
        o => panic!("basesrv-cargo: unsupported arch {o}"),
    };

    Linker::new(target_json)
        // basesrv_dll_only gates the basesrv DllMain/ServerDllInitialization
        // entry surface; ntddi_ws03 stays on (additive) → matches cmake's
        // `--no-default-features --features basesrv_dll_only,ntddi_ws03`.
        .rust_staticlib("smss-csrss-rust")
        .rust_features(&["basesrv_dll_only"])
        // 4 basesrv.spec exports (ServerDllInitialization / BaseSetProcessCreate
        // Notify / BaseSrvNlsLogon / BaseSrvNlsUpdateRegistryCache) via a
        // spec2def-generated .def. No -stub entries.
        .spec2def("subsystems/win/basesrv/basesrv.spec", "basesrv.dll")
        // basesrv_rust_stub.c was a cmake-era cohabitation stub (an empty TU to
        // force archive extraction under add_executable); the cargo link pulls
        // the Rust body archive directly via the CRT's symbol references, so it
        // is vestigial and dropped — the basesrv link now compiles ZERO C.
        // basesrv.rc compiled IN-cargo by replaying the basesrv_res windres
        // command (drops the cmake-built .res reuse).
        .resource_from_rc("subsystems/win/basesrv/basesrv.rc", "basesrv_res")
        // csrsrv + ntdll import libs self-hosted from each .spec.
        .import_lib("subsystems/csr/csrsrv/csrsrv.spec", "csrsrv.dll", "csrsrv")
        .import_lib("dll/ntdll/def/ntdll.spec", "ntdll.dll", "ntdll")
        // ZERO cmake .a: libchkstk.a + libpseh.a both dropped. Probe (nm -u on
        // libsmss_csrss_rust.a ×3 arches) confirms the Rust body references
        // neither __chkstk/___alloca (no >4KB frames in codegen) nor any PSEH
        // runtime symbol (__except_handler3/__SEH_prolog — DUMMY_PSEH makes the
        // handlers dead code, and Rust translations never emit PSEH at all).
        // basesrv now compiles and links ZERO C.
        .link(ModuleKind::Win32Dll {
            entry: entry.into(),
            image_base: Some(image_base),
            subsystem: Some("console:5.01".into()),
        });
}
