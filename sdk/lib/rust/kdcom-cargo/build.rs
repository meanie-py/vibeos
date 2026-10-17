//! Cargo-driven `kdcom.dll` link (i386 + amd64 + arm64). Manifest for the
//! shared `reactos-link` engine — all machinery lives there. KD-COM sibling
//! of `bootvid-cargo` / `hal-cargo` build.rs.
//!
//! Mirrors the cmake `kdcom.dll` link line (from each build-<arch>/build.ninja,
//! target `drivers/base/kdrosdbg/kdcom.dll`):
//!   subsystem `native:5.01`, `-shared` (kerneldll), `--exclude-all-symbols`
//!             + 0x1000 align (the 8 kdrosdbg.spec exports come from the .def,
//!             not auto-export), no `--gc-sections`.
//!   entry  i386 `-Wl,-e,___ImageBase` (C `_`→triple `_` on i686 GNU);
//!          amd64/arm64 `-Wl,-e,__ImageBase` (no cdecl `_` prefix).
//!   base   i386/amd64 `--image-base,0x00010000`; arm64
//!          `--image-base,0x7FFB00000000` (verbatim from build-arm64).
//!   libs   ntoskrnl import lib (self-generated from ntoskrnl.spec —
//!          only `DbgPrint` is referenced) + the Cp* serial port symbols.
//!          ZERO cmake .a: cportlib/cport_rust → the cargo-sibling `cport-rust`
//!          (.rust_lib, like uefildr); memcmp → compiler-builtins-mem.
//!
//! The C `kdrosdbg` target also imports `hal`, but the stub references zero
//! hal symbols, so no hal import lib is needed.
//!
//! Run `ninja` in build-<arch>/ first (reactos-link consumes cmake-built
//! leaf static libs + the spec2def host tool from there).

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");

    // The module's part of cmake's windres command; reactos-link adds what every
    // module shares from reactos-link/cflags/rc.<arch>.flags (Linker::rc).
    let rc_includes: &[&str] = &[
        "@build/drivers/base/kdrosdbg",
        "drivers/base/kdrosdbg",
        "@std",
        "sdk/lib/pseh/include",
        "vendor/opengl-headers/include",
        "vendor/libdxg/include",
    ];
    let rc_flags: &[&str] = &[];
    let rc_defines: &[&str] = &["WINVER=0x502", "_WIN32_WINNT=0x502"];

    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "___ImageBase", 0x0001_0000),
        "x86_64" => ("x86_64-pc-windows-gnu", "__ImageBase", 0x0001_0000),
        "aarch64" => ("aarch64-pc-windows-gnullvm", "__ImageBase", 0x7FFB_0000_0000),
        other => panic!("kdcom-cargo: unsupported arch {other} (i386/amd64/arm64 only)"),
    };

    Linker::new(target_json)
        .rust_staticlib("kdcom-rust")
        .spec2def("drivers/base/kdrosdbg/kdrosdbg.spec", "kdcom.dll")
        // ZERO cmake .a. The Cp* serial port symbols come from `cport-rust` as a
        // DIRECT cargo sibling (.rust_lib) — exactly what uefildr-cargo does —
        // which subsumes BOTH the C cohab stub libcportlib.a (its only unique
        // symbol is the unreferenced __cportlib_rust_cohab_stub) AND the
        // cmake-copied libcport_rust.a. libmemcmp.a is dropped: `memcmp` comes
        // from `-Z build-std-features=compiler-builtins-mem` (same one-line drop
        // the cargo-cut driver/DLL fleet already took).
        .rust_lib("cport-rust", &[], true)
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .rc("drivers/base/kdrosdbg/kdrosdbg.rc", rc_includes, rc_flags, rc_defines)
        // cmake's kernel-mode links keep every section.
        .keep_all_sections()
        .link(ModuleKind::KernelDll {
            // ENTRYPOINT 0: the DLL has no DllMain (ntoskrnl resolves the 8
            // spec exports by name). The entry symbol is lld's image-base
            // symbol, arch-decorated (i386 `___ImageBase` / else `__ImageBase`).
            entry: entry.into(),
            image_base,
        });
}
