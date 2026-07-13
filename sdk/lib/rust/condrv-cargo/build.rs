//! Cargo-driven `condrv.sys` link (i386 + amd64 + arm64). Manifest for the
//! shared `reactos-link` engine. Sibling of `beep-cargo` / `null-cargo`.
//!
//! Mirrors the cmake `condrv.sys` link (set_module_type kernelmodedriver):
//!   subsystem `native:5.01`, `-shared` (kerneldll), `--exclude-all-symbols`
//!             + 0x1000 align, no `--gc-sections`.
//!   entry  `DriverEntry@8` — i386 `_DriverEntry@8`; amd64/arm64 `DriverEntry`.
//!   base   i386/amd64 `0x00010000`; arm64 `0x7FFB00000000`.
//!   libs   ntoskrnl + hal import libs (`add_importlibs(condrv ntoskrnl hal)`;
//!          condrv only calls ntoskrnl Io*/Rtl*, but hal is kept to match cmake).
//!
//! The version resource (condrv.rc) is dropped, as in the other cargo cuts.
//! The kernel softfloat json is correct for a kernel-mode driver.
//!
//! Run `ninja` in build-<arch>/ first (the cargo link reuses the spec2def host
//! tool + ntoskrnl/hal spec files).

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "_DriverEntry@8", 0x0001_0000),
        "x86_64" => ("x86_64-pc-windows-gnu", "DriverEntry", 0x0001_0000),
        "aarch64" => ("aarch64-pc-windows-gnullvm", "DriverEntry", 0x7FFB_0000_0000),
        other => panic!("condrv-cargo: unsupported arch {other} (i386/amd64/arm64 only)"),
    };

    Linker::new(target_json)
        .rust_staticlib("condrv-rust")
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .import_lib("hal/hal.spec", "hal.dll", "hal")
        .link(ModuleKind::KernelDll {
            entry: entry.into(),
            image_base,
        });
}
