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
//! body + the RunLoader entry, built `--features rosload`), and the sibling
//! leaf libs. The freeldr import lib is self-hosted by reactos-link from
//! freeldr.spec (.import_lib()) — no cmake-built libfreeldr.a needed.
//! reactos-link wraps them in --start-group/--end-group.

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");

    // rosload-specific compilation is now all reactos-link's: strsafe_shim.c via
    // .c_shims(), and ALL the asm (linux.S + amd64:misc.S) flipped to Rust
    // global_asm! in freeldr-rust (loader_asm.rs → libfreeldr_rust_rosload.a).
    // cmboot is Rust (cmboot-rust). The SOLE remaining cmake-built reused object
    // is the i386 16-bit drvmap_gcc.obj (clang can't .code16); rosload_objs gone.
    let (target_json, entry, mut libs): (&str, &str, Vec<&str>) = match arch.as_str() {
        "x86" => {
            // linux.S flipped to Rust global_asm! (freeldr-rust/loader_asm.rs).
            (
                "i686-kernel.json",
                "_RunLoader",
                vec![
                    // drvmap: 16-bit gcc asm, reused directly.
                    "boot/freeldr/freeldr/drvmap_gcc.obj",
                    // i386 mini-HAL linked BEFORE the freeldr libs.
                    "hal/halx86/minihal/libmini_hal.a",
                    "hal/halx86/libhal_rust_mini.a",
                ],
            )
        }
        "x86_64" => {
            // misc.S + linux.S flipped to Rust global_asm! (loader_asm.rs).
            ("x86_64-kernel.json", "RunLoader", Vec::new())
        }
        other => panic!("rosload-cargo: unsupported arch {other} (i386/amd64 only; arm64 is UEFI-only, no rosload)"),
    };

    // Common libs, in cmake link order: the rosload Rust body, the sibling C
    // leaf libs, the C libcntpr, then libcntpr_rust. The freeldr import lib is
    // no longer a cmake-built reused .a — reactos-link self-hosts it from
    // freeldr.spec (.import_lib() below), so cmake's spec2def(freeldr.sys ...
    // ADD_IMPORTLIB) is retired.
    libs.extend_from_slice(&[
        "boot/freeldr/freeldr/libfreeldr_rust_rosload.a",
        "sdk/lib/cmlib/libblcmlib.a",
        "boot/rtl/libblrtl.a",
        "sdk/lib/crt/liblibcntpr.a",
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
        .spec2def("boot/freeldr/freeldr/rosload.spec", "rosload.exe")
        // freeldr import lib, self-hosted from freeldr.spec (was the cmake
        // spec2def(freeldr.sys freeldr.spec ADD_IMPORTLIB) → libfreeldr.a).
        // rosload's freeldr.sys imports bind at runtime against uefildr.efi,
        // which the PE loader registers as "freeldr.sys" and which exports the
        // same freeldr.spec symbols. Same spec → identical import names/dll.
        .import_lib("boot/freeldr/freeldr/freeldr.spec", "freeldr.sys", "freeldr");

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
