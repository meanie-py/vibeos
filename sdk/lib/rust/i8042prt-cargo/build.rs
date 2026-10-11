//! Cargo-driven `i8042prt.sys` link (i386 + amd64 + arm64). Manifest for the
//! shared `reactos-link` engine. Sibling of `null-cargo` / `sysfb-cargo`.
//!
//! Mirrors the cmake `i8042prt.sys` link (set_module_type kernelmodedriver):
//!   subsystem `native:5.01`, `-shared` (kerneldll), `--exclude-all-symbols`
//!             + 0x1000 align, no `--gc-sections`.
//!   entry  `DriverEntry@8` — i386 `_DriverEntry@8`; amd64/arm64 `DriverEntry`.
//!   base   i386/amd64 `0x00010000`; arm64 `0x7FFB00000000`.
//!   libs   ntoskrnl + hal import libs (`add_importlibs(i8042prt ntoskrnl
//!          hal)`; strcmp is an ntoskrnl cdecl export). cmake's
//!          `target_link_libraries(i8042prt dmilib …)` is GONE — ParseSMBiosTables
//!          is now a Rust port in i8042prt-rust (crate::dmi), so libdmilib.a is
//!          no longer linked.
//!
//! No guid.c / c_from_compile_db: i8042prt-rust defines its sole poclass GUID
//! (GUID_DEVICE_SYS_BUTTON) as a Rust-internal `static` used via addr_of!, so
//! the C guid.c GUID symbol is unreferenced (dead residue, kept on disk).
//! The version resource (i8042prt.rc) is dropped, as in the other cargo cuts;
//! the registry inf (i8042prt_reg.inf) is independent of the link and stays in
//! cmake. The cohab body built with the user target i686-pc-windows-gnu
//! -sse,-sse2; the kernel softfloat json here is correct for a kernel driver.
//!
//! Run `ninja` in build-<arch>/ first (the cargo link reuses the spec2def host
//! tool). ZERO cmake .a: libdmilib.a dropped (ParseSMBiosTables ported to Rust).

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "_DriverEntry@8", 0x0001_0000),
        "x86_64" => ("x86_64-pc-windows-gnu", "DriverEntry", 0x0001_0000),
        "aarch64" => ("aarch64-pc-windows-gnullvm", "DriverEntry", 0x7FFB_0000_0000),
        other => panic!("i8042prt-cargo: unsupported arch {other} (i386/amd64/arm64 only)"),
    };

    Linker::new(target_json)
        .rust_staticlib("i8042prt-rust")
        // ZERO cmake .a: libdmilib.a dropped — ParseSMBiosTables is a Rust port
        // in i8042prt-rust (crate::dmi). The host tool udmihelp keeps the C copy.
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .import_lib("hal/hal.spec", "hal.dll", "hal")
        .link(ModuleKind::KernelDll {
            entry: entry.into(),
            image_base,
        });
}
