//! Cargo-driven `kdcom.dll` link for the kdgdb (GDB-stub) KD transport.
//! Sibling of kdcom-cargo's build.rs. i386 + amd64 (kdgdb has no arm64).
//!
//! Mirrors the cmake `kdgdb` link (drivers/base/kdgdb/CMakeLists.txt:
//! `set_module_type(kdcom kerneldll ENTRYPOINT 0)`, native subsystem,
//! `add_importlibs(kdcom ntoskrnl hal)`, `target_link_libraries(kdcom
//! cportlib)`), but driven by reactos-link with the kdgdb-rust staticlib.
//!
//! Beyond the kdrosdbg stub, kdgdb pulls real CRT (sprintf/_snprintf/strcmp/
//! strstr/_strupr/atol/strlen) from libcntpr + the cnt-rust twin, and the
//! cportlib serial Cp* symbols. The ntoskrnl + hal import libs resolve its
//! kernel/HAL imports.
//!
//! Run `ninja` in build-<arch>/ first so the leaf static libs + import-lib
//! host tooling exist (reactos-link consumes them).

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "___ImageBase", 0x0001_0000),
        "x86_64" => ("x86_64-kernel.json", "__ImageBase", 0x0001_0000),
        other => panic!("kdgdb-cargo: unsupported arch {other} (i386/amd64 only; kdgdb has no arm64)"),
    };

    // cnt-rust-lib float feature: i386 softfloat omits it, amd64 enables it.
    let cnt_feats: &[&str] = if arch == "x86" { &[] } else { &["float"] };

    Linker::new(target_json)
        .rust_staticlib("kdgdb-rust")
        .spec2def("drivers/base/kdgdb/kdgdb.spec", "kdcom.dll")
        // Cp* serial → `cport-rust` cargo sibling (subsumes libcportlib.a stub +
        // libcport_rust.a); cntpr_rust → `cnt-rust-lib`; memcmp →
        // compiler-builtins-mem. kdgdb formats GDB packets with sprintf/_snprintf
        // — those live ONLY in the C printf tail, so liblibcntpr.a is KEPT
        // (cnt-rust-lib supplies strcmp/strstr/_strupr/atol/strlen, not sprintf).
        .rust_lib("cport-rust", &[], true)
        .rust_lib("cnt-rust-lib", cnt_feats, true)
        .static_libs(&["sdk/lib/crt/liblibcntpr.a"])
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .import_lib("hal/hal.spec", "hal.dll", "hal")
        .link(ModuleKind::KernelDll {
            entry: entry.into(),
            image_base,
        });
}
