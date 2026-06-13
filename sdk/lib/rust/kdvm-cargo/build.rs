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

    // cnt-rust-lib float feature: i386 softfloat omits it, amd64 enables it.
    let cnt_feats: &[&str] = if arch == "x86" { &[] } else { &["float"] };

    Linker::new(target_json)
        .rust_staticlib("kdvm-rust")
        .spec2def("drivers/base/kdvm/kdvm.spec", "kdvbox.dll")
        // cntpr_rust → `cnt-rust-lib` cargo sibling; memcmp →
        // compiler-builtins-mem. kdvm has no cportlib (it talks to the VBox
        // backdoor via in-crate KdVmExchange port I/O).
        .rust_lib("cnt-rust-lib", cnt_feats, true)
        // ZERO cmake .a: liblibcntpr.a DROPPED. The ONLY symbol kdvm pulled from
        // it is `cnt_va_next_ptr` (cnt-rust-lib's scanf-engine va_arg helper from
        // cnt-rust's own cabi/scanf_trampoline.c — Rust can't DEFINE variadics on
        // stable). Compiled in-cargo via c_from_compile_db (ntdll-cargo pattern).
        .c_from_compile_db("libcntpr", &["sdk/lib/rust/cnt-rust/src/cabi/scanf_trampoline.c"])
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .import_lib("hal/hal.spec", "hal.dll", "hal")
        .link(ModuleKind::KernelDll {
            entry: entry.into(),
            image_base,
        });
}
