//! Cargo-driven `netstack.sys` link (i386 + amd64 + arm64). Manifest for the
//! shared `reactos-link` engine. Sibling of `beep-cargo` / `null-cargo`.
//!
//! Mirrors a cmake `set_module_type(kernelmodedriver)` link:
//!   subsystem native, `-shared` (kerneldll), `--exclude-all-symbols`, no
//!   `--gc-sections` (so `DriverEntry` is retained).
//!   entry  `DriverEntry@8` — i386 `_DriverEntry@8`; amd64/arm64 `DriverEntry`.
//!   base   i386/amd64 `0x00010000`; arm64 `0x7FFB00000000`.
//!   libs   ntoskrnl + hal + ndis import libs (HalGetBusDataByOffset / MmMapIoSpace /
//!          KeStallExecutionProcessor are hal/ntoskrnl exports).
//!
//! The body crate `netstack-rust` pulls in smoltcp + virtio-drivers; reactos-link
//! cargo-builds it (build-std nightly, kernel json) and links the staticlib.
//!
//! Run `ninja` in build-<arch>/ first (the link reuses the spec2def host tool +
//! ntoskrnl/hal spec files).

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "_DriverEntry@8", 0x0001_0000),
        "x86_64" => ("x86_64-pc-windows-gnu", "DriverEntry", 0x0001_0000),
        "aarch64" => ("aarch64-pc-windows-gnullvm", "DriverEntry", 0x7FFB_0000_0000),
        other => panic!("netstack-cargo: unsupported arch {other} (i386/amd64/arm64 only)"),
    };

    Linker::new(target_json)
        .rust_staticlib("netstack-rust")
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .import_lib("hal/hal.spec", "hal.dll", "hal")
        // The NDIS protocol shim (netstack-rust ndis.rs) for NICs a miniport drives.
        .import_lib("drivers/network/ndis/ndis.spec", "ndis.sys", "ndis")
        .link(ModuleKind::KernelDll {
            entry: entry.into(),
            image_base,
        });
}
