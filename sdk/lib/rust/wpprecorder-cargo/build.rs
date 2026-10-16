//! Cargo-driven `wpprecorder.sys` link (amd64 + arm64), on the `null-cargo` /
//! `hidparse-cargo` pattern: a kernel-mode export driver, entry `DriverEntry`, EXPORTS
//! from drivers/base/wpprecorder/wpprecorder.spec, imports ntoskrnl only. 64-bit only:
//! every driver that imports it is (see the spec).

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, image_base): (&str, u64) = match arch.as_str() {
        "x86_64" => ("x86_64-pc-windows-gnu", 0x0001_0000),
        "aarch64" => ("aarch64-pc-windows-gnullvm", 0x7FFB_0000_0000),
        other => panic!("wpprecorder-cargo: unsupported arch {other} (amd64/arm64 only)"),
    };

    Linker::new(target_json)
        .rust_staticlib("wpprecorder-rust")
        .spec2def("drivers/base/wpprecorder/wpprecorder.spec", "wpprecorder.sys")
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        // cmake's kernel-mode links keep every section.
        .keep_all_sections()
        .link(ModuleKind::KernelDll {
            entry: "DriverEntry".into(),
            image_base,
        });
}
