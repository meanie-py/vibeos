//! Cargo-driven `arm64input.sys` link (arm64-only). Manifest for the shared
//! `reactos-link` engine. Sibling of `null-cargo`. The cmake driver only calls
//! add_cargo_module for this when ARCH==arm64, so only the aarch64 branch is
//! ever exercised.
//!
//! Mirrors the cmake `arm64input.sys` link (set_module_type kernelmodedriver):
//!   subsystem `native:5.01`, `-shared` (kerneldll), `--exclude-all-symbols`
//!             + 0x1000 align, no `--gc-sections`.
//!   entry  `DriverEntry` (arm64 undecorated). base `0x7FFB00000000`.
//!   libs   ntoskrnl + hal import libs (cmake `add_importlibs(arm64input
//!          ntoskrnl hal)`). No leaf static libs.
//!
//! Run `ninja` in build-arm64/ first.

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "aarch64" => ("aarch64-pc-windows-gnullvm", "DriverEntry", 0x7FFB_0000_0000),
        other => panic!("arm64input-cargo: arm64-only driver, unsupported arch {other}"),
    };

    Linker::new(target_json)
        .rust_staticlib("arm64input-rust")
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .import_lib("hal/hal.spec", "hal.dll", "hal")
        // cmake's kernel-mode links keep every section.
        .keep_all_sections()
        .link(ModuleKind::KernelDll {
            entry: entry.into(),
            image_base,
        });
}
