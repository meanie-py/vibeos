//! Cargo-driven `wowx64.dll` link — the OS side of BTCpu64, x86-64 guest
//! processes on an arm64 host (docs/engineering/wow64-x64-guest-design.md,
//! Fork B). arm64 ONLY: on amd64 an x64 exe is native and on i386 there is no
//! such thing, so dll/win32/wowx64/CMakeLists.txt only declares it there.
//!
//! ntdll-only, deliberately, like wow64.dll: it is loaded into every x64 guest
//! process during process init, and must not drag a native kernel32/msvcrt
//! into a process whose user mode is about to be the GUEST's. So no CRT
//! startup: the entry is the crate's own `DllMain` (cf. ntdll_vista-cargo),
//! exports via spec2def(wowx64.spec), and the only import library is ntdll.

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    if arch != "aarch64" {
        // Only a workspace-wide `cargo check` gets here — the CMake declares
        // this cell on arm64 alone, so no build ever asks for this link. Say so
        // and link nothing, rather than red every lane's x86 check.
        println!("cargo:warning=wowx64-cargo: arm64-only cell, nothing to link for {arch}");
        return;
    }

    Linker::new("aarch64-pc-windows-gnullvm")
        .rust_staticlib("wowx64-rust")
        .spec2def("dll/win32/wowx64/wowx64.spec", "wowx64.dll")
        .import_lib("dll/ntdll/def/ntdll.spec", "ntdll.dll", "ntdll")
        .link(ModuleKind::Win32Dll {
            entry: "DllMain".into(),
            image_base: Some(0x7ff_7c00_0000),
            subsystem: None, // lld default -> GUI 6.0
        });
}
