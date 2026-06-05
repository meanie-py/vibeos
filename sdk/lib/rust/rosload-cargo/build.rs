//! Cargo-driven `rosload.exe` link (i386 + amd64). Manifest for the shared
//! `reactos-link` engine. Native-subsystem sibling of `uefildr-cargo`'s
//! build.rs.
//!
//! Mirrors the cmake `rosload.exe` link (from build-<arch>/build.ninja,
//! target `boot/freeldr/freeldr/rosload.exe`):
//!   subsystem  `native:5.01`                  (set_subsystem native)
//!   entry      i386 `_RunLoader`, amd64 `RunLoader`  (set_entrypoint RunLoader)
//!   imagebase  `0x10000`                       (set_image_base 0x10000)
//!   gc         `--gc-sections`, NOT -shared    (it's an exe)
//!   exports    `rosload.def` from rosload.spec (one export: RtlAssert) — so
//!              NO `--exclude-all-symbols` (ENABLE_EXPORTS TRUE). spec2def
//!              `-n=rosload.exe`.
//!   align      DEFAULT (cmake passes no --file/-section-alignment).
//!   i386 only  `-u _HalDispatchTable` / `-u _HalPrivateDispatchTable`
//!              force-undefined (pull the Rust halstub HAL dispatch tables;
//!              unreferenced+unexported → GC'd without -u) + the i386 mini-HAL
//!              libs (libmini_hal.a + libhal_rust_mini.a) linked first.
//!
//! Inputs are the cmake-built artifacts (run `ninja rosload` in build-<arch>/
//! first): the prebuilt objects compiled into rosload.dir (strsafe_shim.c.obj,
//! cmboot.c.obj, per-arch asm), `libfreeldr_rust_rosload.a` (the freeldr-rust
//! body + the RunLoader entry, built `--features rosload`), the sibling leaf
//! libs, and the freeldr import lib (`libfreeldr.dir/libfreeldr.a`).
//! reactos-link wraps them in --start-group/--end-group.

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");

    // Per-arch prebuilt objects + leaf libs (verbatim from each
    // build-<arch>/build.ninja's rosload.exe inputs). The objects are
    // reused-prebuilt (header-dependent, compiled by cmake into rosload.dir),
    // matching uefildr-cargo's strsafe_shim reuse.
    let (target_json, entry, mut libs): (&str, &str, Vec<&str>) = match arch.as_str() {
        "x86" => (
            "i686-kernel.json",
            "_RunLoader",
            vec![
                // i386 objects: drvmap is gcc-compiled (16-bit asm), the rest
                // clang-compiled by the cmake OBJECT lib `rosload_objs`.
                // drvmap is 16-bit gcc asm (clang can't do .code16) — stays a
                // cmake-built reused .obj. strsafe_shim.c is compiled directly
                // by reactos-link via .c_shims() below (self-contained leaf, no
                // freeldr env needed); cmboot is Rust (cmboot-rust in the
                // rosload-feature archive). Neither is a reused .obj anymore.
                "boot/freeldr/freeldr/drvmap_gcc.obj",
                "boot/freeldr/freeldr/CMakeFiles/rosload_objs.dir/arch/i386/linux.S.obj",
                // i386 mini-HAL linked BEFORE the freeldr libs (rosload.cmake
                // `if(ARCH STREQUAL i386)`).
                "hal/halx86/minihal/libmini_hal.a",
                "hal/halx86/libhal_rust_mini.a",
            ],
        ),
        "x86_64" => (
            "x86_64-kernel.json",
            "RunLoader",
            vec![
                // strsafe_shim.c → reactos-link .c_shims() (see i386 note);
                // cmboot is Rust. Only the per-arch asm remains a reused .obj.
                "boot/freeldr/freeldr/CMakeFiles/rosload_objs.dir/arch/amd64/misc.S.obj",
                "boot/freeldr/freeldr/CMakeFiles/rosload_objs.dir/arch/amd64/linux.S.obj",
            ],
        ),
        other => panic!("rosload-cargo: unsupported arch {other} (i386/amd64 only; arm64 is UEFI-only, no rosload)"),
    };

    // Common libs, in cmake link order: the rosload Rust body, the sibling C
    // leaf libs, the C libcntpr, the freeldr import lib, then libcntpr_rust.
    libs.extend_from_slice(&[
        "boot/freeldr/freeldr/libfreeldr_rust_rosload.a",
        "sdk/lib/cmlib/libblcmlib.a",
        "boot/rtl/libblrtl.a",
        "sdk/lib/crt/liblibcntpr.a",
        // freeldr import lib (add_importlibs(rosload freeldr)); rosload's
        // freeldr.sys imports resolve at runtime against uefildr.efi.
        "boot/freeldr/freeldr/CMakeFiles/libfreeldr.dir/libfreeldr.a",
        "sdk/lib/crt/libcntpr_rust.a",
    ]);

    let mut linker = Linker::new(target_json)
        // No .rust_staticlib(): reuse the cmake-built libfreeldr_rust_rosload.a
        // (identical codegen + carries the `rosload` feature).
        .static_libs(&libs)
        // strsafe_shim.c compiled directly (self-contained leaf). rosload is
        // i386/amd64 only — both x86 — so -mno-sse always applies (freeldr
        // pre-boot has SSE disabled; an emitted movaps would #UD).
        .c_shims(&["boot/freeldr/freeldr/lib/strsafe_shim.c"])
        .c_shim_flags(&["-mno-sse"])
        // rosload exports RtlAssert via rosload.def (ENABLE_EXPORTS TRUE).
        .spec2def("boot/freeldr/freeldr/rosload.spec", "rosload.exe");

    if arch == "x86" {
        // The C halstub.c always linked the Rust HAL dispatch tables as a
        // source object; as archive members they'd be GC'd. Pull them.
        linker = linker.force_undefined(&["_HalDispatchTable", "_HalPrivateDispatchTable"]);
    }

    linker.link(ModuleKind::NativeExe {
        entry: entry.into(),
        image_base: 0x1_0000,
    });
}
