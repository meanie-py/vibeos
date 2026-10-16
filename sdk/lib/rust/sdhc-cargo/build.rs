//! Cargo-driven `sdhc.sys` link (arm64 only: the SD host is published by the
//! arm64 HAL's ACPI PnP root). Manifest for the shared `reactos-link` engine,
//! the same shape as beep-cargo: a kernel-mode driver, entry `DriverEntry`,
//! ntoskrnl + hal import libs (KeStallExecutionProcessor and
//! KeQueryPerformanceCounter are hal exports).

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "aarch64" => ("aarch64-pc-windows-gnullvm", "DriverEntry", 0x7FFB_0000_0000),
        other => panic!("sdhc-cargo: unsupported arch {other} (arm64 only)"),
    };

    Linker::new(target_json)
        .rust_staticlib("sdhc-rust")
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .import_lib("hal/hal.spec", "hal.dll", "hal")
        // cmake's kernel-mode links keep every section.
        .keep_all_sections()
        .link(ModuleKind::KernelDll { entry: entry.into(), image_base });
}
