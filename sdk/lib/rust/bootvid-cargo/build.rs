//! Cargo-driven `bootvid.dll` link (i386 + amd64 + arm64). Manifest for the
//! shared `reactos-link` engine. Boot-video sibling of `kdcom-cargo` /
//! `hal-cargo` build.rs.
//!
//! Mirrors the cmake `bootvid.dll` link line (each build-<arch>/build.ninja,
//! target `drivers/base/bootvid/bootvid.dll`):
//!   subsystem `native:5.01`, `-shared` (kerneldll), `--exclude-all-symbols`
//!             + 0x1000 align (the 11 bootvid.spec `Vid*` exports come from
//!             the .def), no `--gc-sections`.
//!   entry  i386 `-Wl,-e,___ImageBase`; amd64/arm64 `-Wl,-e,__ImageBase`.
//!   base   i386/amd64 `--image-base,0x00010000`; arm64
//!          `--image-base,0x7FFB00000000`.
//!   libs   ntoskrnl + hal import libs (self-generated from their specs;
//!          cmake `add_importlibs(bootvid ntoskrnl hal)`) + memcmp leaf
//!          static lib (cmake LINK_LIBRARIES).
//!
//! Run `ninja` in build-<arch>/ first (reactos-link consumes the cmake-built
//! leaf static lib + spec2def host tool from there).

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "___ImageBase", 0x0001_0000),
        "x86_64" => ("x86_64-pc-windows-gnu", "__ImageBase", 0x0001_0000),
        "aarch64" => ("aarch64-pc-windows-gnullvm", "__ImageBase", 0x7FFB_0000_0000),
        other => panic!("bootvid-cargo: unsupported arch {other} (i386/amd64/arm64 only)"),
    };

    Linker::new(target_json)
        .rust_staticlib("bootvid-rust")
        .spec2def("drivers/base/bootvid/bootvid.spec", "bootvid.dll")
        // ZERO cmake .a: libmemcmp.a dropped (cold-link confirms bootvid.sys
        // references no memcmp; compiler-builtins-mem covers it if ever).
        // Chainable multi-import (reactos-link Vec<import_lib>): exactly
        // cmake's `add_importlibs(bootvid ntoskrnl hal)`.
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .import_lib("hal/hal.spec", "hal.dll", "hal")
        // cmake's kernel-mode links keep every section.
        .keep_all_sections()
        .link(ModuleKind::KernelDll {
            // ENTRYPOINT 0: no DllMain (ntoskrnl resolves the Vid* exports by
            // name). Entry = lld's image-base symbol, arch-decorated.
            entry: entry.into(),
            image_base,
        });
}
