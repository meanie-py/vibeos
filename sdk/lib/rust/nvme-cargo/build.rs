//! Cargo-driven `nvme.sys` link (i386 + amd64 + arm64). Manifest for the shared
//! `reactos-link` engine, the same shape as beep-cargo: a kernel-mode driver,
//! entry `DriverEntry` (`_DriverEntry@8` on i386), ntoskrnl + hal import libs
//! (KeStallExecutionProcessor and KeQueryPerformanceCounter are hal exports).

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "_DriverEntry@8", 0x0001_0000),
        "x86_64" => ("x86_64-pc-windows-gnu", "DriverEntry", 0x0001_0000),
        "aarch64" => ("aarch64-pc-windows-gnullvm", "DriverEntry", 0x7FFB_0000_0000),
        other => panic!("nvme-cargo: unsupported arch {other} (i386/amd64/arm64 only)"),
    };

    Linker::new(target_json)
        .rust_staticlib("nvme-rust")
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .import_lib("hal/hal.spec", "hal.dll", "hal")
        .link(ModuleKind::KernelDll { entry: entry.into(), image_base });
}
