//! Cargo-driven kdvbox.dll link for the VirtualBox KD transport. Mirrors
//! kdgdb-cargo but spec2def kdvm.spec → kdvbox.dll, no cportlib (kdvm talks to
//! the VBox backdoor via the in-crate KdVmExchange port I/O, not a UART).
//! CRT + memcmp leaf libs, ntoskrnl + hal imports. i386 + amd64. Run `ninja`
//! in build-<arch>/ first so the leaf libs exist.

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "___ImageBase", 0x0001_0000),
        "x86_64" => ("x86_64-kernel.json", "__ImageBase", 0x0001_0000),
        other => panic!("kdvm-cargo: unsupported arch {other} (i386/amd64 only)"),
    };

    Linker::new(target_json)
        .rust_staticlib("kdvm-rust")
        .spec2def("drivers/base/kdvm/kdvm.spec", "kdvbox.dll")
        .static_libs(&[
            "sdk/lib/crt/liblibcntpr.a",
            "sdk/lib/crt/libcntpr_rust.a",
            "sdk/lib/crt/libmemcmp.a",
        ])
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .import_lib("hal/hal.spec", "hal.dll", "hal")
        .link(ModuleKind::KernelDll {
            entry: entry.into(),
            image_base,
        });
}
