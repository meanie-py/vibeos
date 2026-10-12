//! Cargo-driven `sysfb.sys` link (i386 + amd64 + arm64). Manifest for the
//! shared `reactos-link` engine. Sibling of `null-cargo`.
//!
//! Despite the historical name, sysfb is the live GOP/linear-framebuffer
//! video device on ALL three UEFI arches (it renders the desktop), so it links
//! per-arch like any kernelmodedriver.
//!
//! Mirrors the cmake `sysfb.sys` link (set_module_type kernelmodedriver):
//!   subsystem `native:5.01`, `-shared` (kerneldll), `--exclude-all-symbols`
//!             + 0x1000 align, no `--gc-sections`.
//!   entry  `DriverEntry@8` — i386 `_DriverEntry@8`; amd64/arm64 `DriverEntry`.
//!   base   i386/amd64 `0x00010000`; arm64 `0x7FFB00000000`.
//!   libs   ntoskrnl + hal import libs (cmake `add_importlibs(sysfb ntoskrnl
//!          hal)`). No leaf static libs (RtlZeroMemory etc → compiler-builtins).
//!
//! The cohab build used the user target `i686-pc-windows-gnu -sse,-sse2`; the
//! kernel json used here is softfloat, which is correct for a kernel driver
//! (no float crosses the integer/pointer ntoskrnl FFI boundary).
//!
//! Run `ninja` in build-<arch>/ first.

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "_DriverEntry@8", 0x0001_0000),
        "x86_64" => ("x86_64-pc-windows-gnu", "DriverEntry", 0x0001_0000),
        "aarch64" => ("aarch64-pc-windows-gnullvm", "DriverEntry", 0x7FFB_0000_0000),
        other => panic!("sysfb-cargo: unsupported arch {other} (i386/amd64/arm64 only)"),
    };

    Linker::new(target_json)
        .rust_staticlib("sysfb-rust")
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .import_lib("hal/hal.spec", "hal.dll", "hal")
        .link(ModuleKind::KernelDll {
            entry: entry.into(),
            image_base,
        });
}
