//! Cargo-driven `uefildr.efi` link (i386 + amd64 + arm64). Manifest for the
//! shared `reactos-link` engine. UEFI sibling of `kdcom-cargo`'s build.rs.
//!
//! Mirrors the cmake `uefildr.efi` link line (from build-<arch>/build.ninja,
//! target `boot/freeldr/freeldr/uefildr.efi`):
//!   subsystem  `efi_application:5.01`         (set_subsystem 10)
//!   align      file/section 0x200             (UEFI app)
//!   gc         `--gc-sections`, NOT -shared   (it's an exe)
//!   exports    none (`--exclude-all-symbols`; the .def/freeldr.spec is only
//!              for the freeldr *import lib* rosload consumes, NOT for
//!              uefildr's own exports — so NO .spec2def() here. Passing it
//!              forces every spec export as required, and the UEFI build
//!              genuinely lacks a few (`PxeCallApi`,
//!              `DriveMapMapDrivesInSection` — gated out of the Rust uefi
//!              feature, defined only in the C BIOS libfreeldr.a) →
//!              `<root>: undefined symbol`. cmake's link never feeds the
//!              .def to the linker either; matching it = no .def.)
//!
//! Per-arch differences (verbatim from each build.ninja):
//!   i386 : entry `_EfiEntry` (cdecl `_` prefix), auto image base
//!          (0x00400000), 0x200 align, links mini_hal + libhal_rust_mini
//!          (the i386-only freeldr mini-HAL).
//!   amd64: entry `EfiEntry` (no decoration), `--image-base 0x10000` (keep
//!          the loader in the low 1GB freeldr maps), 0x200 align, NO mini_hal.
//!   arm64: entry `EfiEntry`, NO `--image-base` (auto-hash base), 0x1000
//!          file/section align, NO mini_hal (same lib set as amd64).
//!
//! Inputs are the cmake-built artifacts (run `ninja uefildr` in build-<arch>/
//! first): the freeldr C aggregate `libuefifreeldr_common.a`, the prebuilt
//! `strsafe_shim.c.obj` (variadic RtlStringCb* shim Rust stable can't host),
//! and `libfreeldr_rust_uefi.a` (the freeldr-rust body + the EfiEntry entry,
//! built `--features uefi,uefildr-rust-entry`) plus the leaf libs.
//! reactos-link wraps them in --start-group/--end-group, so the cmake link's
//! duplicate-listing trick to satisfy circular refs is unneeded.

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");

    // strsafe_shim.c.obj — the C variadic shim, compiled into the cmake
    // uefildr.dir (header-dependent, so reused prebuilt rather than
    // recompiled as a header-free c_shim).
    let mut libs: Vec<&str> =
        vec!["boot/freeldr/freeldr/CMakeFiles/uefildr.dir/lib/strsafe_shim.c.obj"];

    // Per-arch link knobs (verbatim from each build-<arch>/build.ninja's
    // uefildr.efi LINK_FLAGS):
    //   entry      i386 `_EfiEntry` (cdecl `_`); amd64/arm64 `EfiEntry`.
    //   image_base i386 cmake auto-resolves to the exe default 0x00400000;
    //              amd64 cmake pins 0x10000 (low 1GB freeldr maps); arm64
    //              cmake passes no --image-base → None (auto-hash base).
    //   alignment  x86 (i386/amd64) 0x200; arm64 0x1000.
    let (target_json, entry, image_base, alignment): (&str, &str, Option<u64>, u64) =
        match arch.as_str() {
            "x86" => {
                // i386: mini_hal + libhal_rust_mini are linked BEFORE the
                // freeldr libs (uefi.cmake `if(ARCH STREQUAL i386 AND NOT MSVC)`).
                libs.push("hal/halx86/minihal/libmini_hal.a");
                libs.push("hal/halx86/libhal_rust_mini.a");
                ("i686-kernel.json", "_EfiEntry", Some(0x0040_0000), 0x200)
            }
            "x86_64" => ("x86_64-kernel.json", "EfiEntry", Some(0x1_0000), 0x200),
            "aarch64" => ("aarch64-kernel.json", "EfiEntry", None, 0x1000),
            other => {
                panic!("uefildr-cargo: unsupported arch {other} (i386/amd64/arm64 only)")
            }
        };

    libs.extend_from_slice(&[
        "boot/freeldr/freeldr/libuefifreeldr_common.a",
        "boot/freeldr/freeldr/libfreeldr_rust_uefi.a",
        "sdk/lib/cportlib/libcportlib.a",
        "sdk/lib/cmlib/libblcmlib.a",
        "boot/rtl/libblrtl.a",
        "sdk/lib/crt/liblibcntpr.a",
        "sdk/lib/crt/libchkstk.a",
        "sdk/lib/crt/libcntpr_rust.a",
    ]);

    Linker::new(target_json)
        // No .rust_staticlib(): reuse the cmake-built libfreeldr_rust_uefi.a
        // (identical codegen + carries the uefi/uefildr-rust-entry features).
        .static_libs(&libs)
        .link(ModuleKind::EfiApplication {
            entry: entry.into(),
            image_base,
            alignment,
        });
}
