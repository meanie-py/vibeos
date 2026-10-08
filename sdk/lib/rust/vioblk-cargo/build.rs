//! Cargo-driven `vioblk.sys` link (i386 + amd64 + arm64). Manifest for the shared
//! `reactos-link` engine; a sibling of `netstack-cargo`, which it mirrors.
//!
//! Mirrors a cmake `set_module_type(kernelmodedriver)` link: subsystem native,
//! `-shared` (kerneldll), no `--gc-sections` so `DriverEntry` is retained.
//!   entry  i386 `_DriverEntry@8`; amd64/arm64 `DriverEntry`.
//!   libs   scsiport (ScsiPort*), hal (HalGet/SetBusDataByOffset, MmMapIoSpace's
//!          neighbours), ntoskrnl (Mm contiguous memory, DbgPrint, spinlocks).

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "_DriverEntry@8", 0x0001_0000),
        "x86_64" => ("x86_64-pc-windows-gnu", "DriverEntry", 0x0001_0000),
        "aarch64" => ("aarch64-pc-windows-gnullvm", "DriverEntry", 0x7FFB_0000_0000),
        other => panic!("vioblk-cargo: unsupported arch {other} (i386/amd64/arm64 only)"),
    };

    Linker::new(target_json)
        .rust_staticlib("vioblk-rust")
        .import_lib("drivers/storage/port/scsiport/scsiport.spec", "scsiport.sys", "scsiport")
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .import_lib("hal/hal.spec", "hal.dll", "hal")
        .link(ModuleKind::KernelDll {
            entry: entry.into(),
            image_base,
        });
}
