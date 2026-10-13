//! Cargo-driven `wpprecorderum.dll` link (amd64 + arm64): a user-mode DLL, entry the
//! crate's own `DllMain`, EXPORTS from dll/win32/wpprecorderum/wpprecorderum.spec, and
//! ntdll the only import (panic-common's user-mode exit). 64-bit only, like
//! wpprecorder.sys: every importer is x64.

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let target = match arch.as_str() {
        "x86_64" => "x86_64-pc-windows-gnu",
        "aarch64" => "aarch64-pc-windows-gnullvm",
        other => {
            // Only a workspace-wide `cargo check` gets here: the CMake declares this cell
            // on 64-bit alone.
            println!("cargo:warning=wpprecorderum-cargo: 64-bit-only cell, nothing to link for {other}");
            return;
        }
    };

    Linker::new(target)
        .rust_staticlib("wpprecorderum-rust")
        .spec2def("dll/win32/wpprecorderum/wpprecorderum.spec", "wpprecorderum.dll")
        .import_lib("dll/ntdll/def/ntdll.spec", "ntdll.dll", "ntdll")
        .link(ModuleKind::Win32Dll {
            entry: "DllMain".into(),
            image_base: None,
            subsystem: None,
        });
}
