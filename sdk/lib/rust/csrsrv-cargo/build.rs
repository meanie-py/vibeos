//! Cargo-driven `csrsrv.dll` link (i386 + amd64 + arm64). Manifest for the
//! shared reactos-link engine. Sixth spine binary moved cmake→cargo (after
//! lsass + userinit + winlogon + services + smss); the first native DLL.
//!
//! Mirrors the cmake csrsrv.dll link (build-<arch>/build.ninja):
//!   subsystem native:5.01 ; entry _DllMain@12 (i386) / DllMain (amd64/arm64) ;
//!   -shared ; keep all sections (no --gc-sections) ; exports via the
//!   spec2def-generated csrsrv.def ; image base 0x10000000 (i386) /
//!   0x180000000 (amd64/arm64).
//!   inputs: libsmss_csrss_rust.a (FULL flip — all 7 csrsrv .c, default
//!   features) + the ntdll import lib. ZERO cmake .a (libsmlib/libpseh dropped
//!   — Sm* is body-defined via the smclient.rs port; see note below).
//!
//! csrsrv is a FULL flip; the lone C csrsrv_rust_stub.c is an empty TU →
//! omitted (cargo links the whole staticlib). csrsrv.rc is version-only →
//! omitted. The crate is built with DEFAULT features (the kitchen-sink
//! csrsrv_dll + csrlib + smlib union) exactly as cmake does — the unreferenced
//! exe mains are dead code (csrsrv exports only its .def symbols).
//! ModuleKind::Win32Dll gives the user-mode target + keep-all-sections; the
//! native subsystem + DllMain entry are passed explicitly.

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-pc-windows-gnu", "_DllMain@12", 0x1000_0000),
        "x86_64" => ("x86_64-pc-windows-gnu", "DllMain", 0x1_8000_0000),
        "aarch64" => ("aarch64-pc-windows-gnullvm", "DllMain", 0x1_8000_0000),
        o => panic!("csrsrv-cargo: unsupported arch {o}"),
    };

    // ZERO cmake .a. The Sm* client (libsmlib) is no longer reused: smclient.c
    // is already a Rust port (smss-csrss-rust/src/smlib/smclient.rs), compiled
    // into THIS body under default features (`pub mod smlib`). The body defines
    // all 7 Sm* exports + SmBaseTag itself, so the cmake libsmlib.a was inert
    // (its members never extracted — an .a member only loads to satisfy an
    // UNDEFINED symbol, and these are all body-defined). Residue probe (nm -u
    // body ∩ nm libsmlib.a) = empty ×3. libpseh.a was already dropped (DUMMY_PSEH
    // dead handlers + Rust body never emits PSEH). csrsrv is now fully cargo-native.
    Linker::new(target_json)
        // Default (kitchen-sink) features — same as cmake's bare
        // `cargo build -p smss-csrss-rust`. Win32Dll → user-mode target.
        .rust_staticlib("smss-csrss-rust")
        // 35 csrsrv.spec exports via a spec2def-generated .def.
        .spec2def("subsystems/csr/csrsrv/csrsrv.spec", "csrsrv.dll")
        .import_lib("dll/ntdll/def/ntdll.spec", "ntdll.dll", "ntdll")
        .link(ModuleKind::Win32Dll {
            entry: entry.into(),
            image_base: Some(image_base),
            subsystem: Some("native:5.01".into()),
        });
}
