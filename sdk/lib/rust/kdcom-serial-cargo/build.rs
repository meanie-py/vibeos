//! Cargo-driven kdcom.dll link for the classic serial KD transport. Mirrors
//! kdgdb-cargo (same cportlib + CRT leaf set, ntoskrnl+hal imports). i386 +
//! amd64. Run `ninja` in build-<arch>/ first so the leaf libs exist.

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "___ImageBase", 0x0001_0000),
        "x86_64" => ("x86_64-kernel.json", "__ImageBase", 0x0001_0000),
        other => panic!("kdcom-serial-cargo: unsupported arch {other} (i386/amd64 only)"),
    };

    // cnt-rust-lib float feature: i386's softfloat kernel target omits it,
    // amd64 enables it (mirrors libcntpr.cmake CNT_RUST_FEATURES + uefildr).
    let cnt_feats: &[&str] = if arch == "x86" { &[] } else { &["float"] };

    Linker::new(target_json)
        .rust_staticlib("kdcom-serial-rust")
        .spec2def("drivers/base/kdcom/kdcom.spec", "kdcom.dll")
        // Cp* serial port → `cport-rust` cargo sibling (.rust_lib), subsuming the
        // C cohab stub libcportlib.a + the cmake-copied libcport_rust.a (uefildr
        // precedent). cntpr_rust → `cnt-rust-lib` cargo sibling. libmemcmp →
        // compiler-builtins-mem. liblibcntpr.a is KEPT: this transport formats
        // packets with sprintf/_snprintf, which live ONLY in the C printf tail
        // (cnt-rust-lib provides strcmp/strlen/strstr/strupr/atol, not sprintf).
        // That C float/printf residual is the same one hal/uefildr/kernel share —
        // a separate, larger translation, not KD-specific.
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
