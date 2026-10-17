//! Cargo-driven kdvbox.dll link for the VirtualBox KD transport. Mirrors
//! kdgdb-cargo but spec2def kdvm.spec → kdvbox.dll, no cportlib (kdvm talks to
//! the VBox backdoor via the in-crate KdVmExchange port I/O, not a UART).
//! CRT + memcmp leaf libs, ntoskrnl + hal imports. i386 + amd64. Run `ninja`
//! in build-<arch>/ first so the leaf libs exist.

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");

    // The module's part of cmake's windres command; reactos-link adds what every
    // module shares from reactos-link/cflags/rc.<arch>.flags (Linker::rc).
    let rc_includes: &[&str] = &[
        "@build/drivers/base/kdvm",
        "drivers/base/kdvm",
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
        // stable). WS1 cmake-off (2026-06-21): it uses only <stdarg.h>/<stddef.h>
        // (toolchain headers, no cmake), so it's a plain self-contained c_shim now
        // — no compile_commands.json replay. kdvm now links free of cmake's
        // compile_commands.json.
        .c_shims(&["sdk/lib/rust/cnt-rust/src/cabi/scanf_trampoline.c"])
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .import_lib("hal/hal.spec", "hal.dll", "hal")
        .rc("drivers/base/kdvm/kdvm.rc", rc_includes, rc_flags, rc_defines)
        // cmake's kernel-mode links keep every section.
        .keep_all_sections()
        .link(ModuleKind::KernelDll {
            entry: entry.into(),
            image_base,
        });
}
