//! Cargo-driven `hidparse.sys` link (i386 / amd64 / arm64). The ReactOS HID
//! parser driver (drivers/hid/hidparse), translated to `hidparse-rust` (the
//! driver shell) + `hidparser-rust` (the `hidparser_km` parser library, a
//! sibling static lib). Replaces the cmake
//! `add_library(hidparse MODULE rust_cohab_stub.c hidparse.rc hidparse.def)` +
//! `set_module_type(hidparse kernelmodedriver)` +
//! `spec2def(hidparse.sys hidparse.spec)` +
//! `add_importlibs(hidparse ntoskrnl)` cohab.
//!
//! native:5.01, -shared, --exclude-all-symbols + 0x1000 align; entry
//! _DriverEntry@8 / DriverEntry; base 0x10000 (x86) / 0x7FFB00000000 (arm64);
//! EXPORTS the HidP_* API via the spec2def-generated hidparse.def (hidclass
//! etc. import it); imports ntoskrnl only; reuses libmemcmp.a. Both rust crates
//! build with the standard user target + i386 -sse,-sse2 (the sibling
//! hidparser-rust gets the i386 no-SSE baseline automatically via the engine's
//! user-target path). The cmake `spec2def(hidparse.sys hidparse.spec
//! ADD_IMPORTLIB)` STAYS.

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "_DriverEntry@8", 0x0001_0000),
        "x86_64" => ("x86_64-pc-windows-gnu", "DriverEntry", 0x0001_0000),
        "aarch64" => ("aarch64-pc-windows-gnullvm", "DriverEntry", 0x7FFB_0000_0000),
        o => panic!("hidparse-cargo: unsupported arch {o} (i386/amd64/arm64 only)"),
    };

    let mut linker = Linker::new(target_json)
        .rust_staticlib("hidparse-rust")
        .rust_user_target()
        // hidparser_km parser library (translated) — a sibling static lib in
        // the same link, built user-target like the body.
        .rust_lib("hidparser-rust", &[], true);
    if arch == "x86" {
        linker = linker.rust_build_rustflags("-C target-feature=-sse,-sse2");
    }
    linker
        .spec2def("drivers/hid/hidparse/hidparse.spec", "hidparse.sys")
        // memcmp comes from compiler-builtins-mem (build-std); libmemcmp.a dropped.
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .resource_from_rc("drivers/hid/hidparse/hidparse.rc", "hidparse_res")
        // cmake's kernel-mode links keep every section.
        .keep_all_sections()
        .link(ModuleKind::KernelDll {
            entry: entry.into(),
            image_base,
        });
}
