//! Cargo-driven `pci.sys` link (i386 / amd64 / arm64). The ReactOS PCI bus
//! driver (drivers/bus/pci), fully translated to `pci-rust` and loaded at
//! every boot for PCI device enumeration. Replaces the cmake
//! `add_library(pci MODULE rust_cohab_stub.c pci.rc)` +
//! `set_module_type(pci kernelmodedriver)` + `add_importlibs(pci ntoskrnl hal)`
//! cohab — the Rust body IS the driver; the version resource is dropped (as in
//! every other driver cargo cut).
//!
//! Mirrors the cmake link (build-<arch>/build.ninja `drivers/bus/pci/pci.sys`):
//! subsystem native:5.01, -shared, --exclude-all-symbols + 0x1000 align; entry
//! _DriverEntry@8 (i386) / DriverEntry; base 0x10000 (x86) / 0x7FFB00000000
//! (arm64); imports ntoskrnl + hal; reuses libmemcmp.a. pci-rust builds with
//! the STANDARD user target + i386 -sse,-sse2 (.rust_user_target() — exactly
//! the cmake custom_command's config, NOT the softfloat kernel json).
//!
//! Run `ninja` in build-<arch>/ first (the cargo link reuses cmake's
//! libmemcmp.a + the spec2def host tool for the import libs).

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "_DriverEntry@8", 0x0001_0000),
        "x86_64" => ("x86_64-kernel.json", "DriverEntry", 0x0001_0000),
        "aarch64" => ("aarch64-kernel.json", "DriverEntry", 0x7FFB_0000_0000),
        o => panic!("pci-cargo: unsupported arch {o} (i386/amd64/arm64 only)"),
    };

    let mut linker = Linker::new(target_json)
        .rust_staticlib("pci-rust")
        .rust_user_target();
    if arch == "x86" {
        linker = linker.rust_build_rustflags("-C target-feature=-sse,-sse2");
    }
    linker
        .static_libs(&["sdk/lib/crt/libmemcmp.a"])
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .import_lib("hal/hal.spec", "hal.dll", "hal")
        .link(ModuleKind::KernelDll {
            entry: entry.into(),
            image_base,
        });
}
