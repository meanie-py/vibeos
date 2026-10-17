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

    // The module's part of cmake's windres command; reactos-link adds what every
    // module shares from reactos-link/cflags/rc.<arch>.flags (Linker::rc).
    let rc_includes: &[&str] = &[
        "@build/drivers/base/kdgdb",
        "drivers/base/kdgdb",
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
        other => panic!("kdgdb-cargo: unsupported arch {other} (i386/amd64 only; kdgdb has no arm64)"),
    };

    // cnt-rust-lib float feature: i386 softfloat omits it, amd64 enables it.
    let cnt_feats: &[&str] = if arch == "x86" { &[] } else { &["float"] };

    Linker::new(target_json)
        .rust_staticlib("kdgdb-rust")
        .spec2def("drivers/base/kdgdb/kdgdb.spec", "kdcom.dll")
        // Cp* serial → `cport-rust` cargo sibling (subsumes libcportlib.a stub +
        // libcport_rust.a); cntpr_rust → `cnt-rust-lib`; memcmp →
        // compiler-builtins-mem.
        .rust_lib("cport-rust", &[], true)
        .rust_lib("cnt-rust-lib", cnt_feats, true)
        // ZERO cmake .a: liblibcntpr.a DROPPED. kdgdb formats GDB packets with
        // sprintf/_snprintf, and its scanf path needs cnt_va_next_ptr — the three
        // symbols it pulled from liblibcntpr. The printf FORMAT ENGINE
        // (streamout) is already Rust in cnt-rust-lib; only the thin variadic
        // `va_start; streamout(...)` shells (sprintf.c/_snprintf.c, each #include
        // _sxprintf.c) + the scanf va_arg helper stay C — Rust can't DEFINE
        // variadics on stable. WS1 cmake-off (2026-06-21): compile them as
        // self-contained headerless c_shims (the shared _sxprintf.c engine behind
        // -DCRT_PRINTF_HEADERLESS; scanf uses stdarg/stddef only) instead of a
        // compile_commands.json replay. Verified disasm-identical (WS1 harness) —
        // kdgdb now links free of cmake's compile_commands.json.
        .c_shims(&[
            "sdk/lib/crt/printf/sprintf.c",
            "sdk/lib/crt/printf/_snprintf.c",
            "sdk/lib/rust/cnt-rust/src/cabi/scanf_trampoline.c",
        ])
        .c_shim_flags(&["-DCRT_PRINTF_HEADERLESS"])
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .import_lib("hal/hal.spec", "hal.dll", "hal")
        .rc("drivers/base/kdgdb/kdgdb.rc", rc_includes, rc_flags, rc_defines)
        // cmake's kernel-mode links keep every section.
        .keep_all_sections()
        .link(ModuleKind::KernelDll {
            entry: entry.into(),
            image_base,
        });
}
