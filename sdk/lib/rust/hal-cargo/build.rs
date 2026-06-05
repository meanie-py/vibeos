//! Cargo-driven `hal.dll` link (i386 + amd64). Manifest for the shared
//! `reactos-link` engine — all machinery lives there.
//!
//! There is exactly one `hal.dll` flavor built (cmake target `hal`); the
//! `halacpi.dll`/`halapic.dll`/`halaacpi.dll` names in txtsetup.sif are
//! install-time aliases of it. i386 is the pic-flavor (generic/legacy/pic
//! components), amd64 the acpi/apic-flavor (generic/acpi/apic) — but both
//! reduce to: libhal_rust.a (the whole HAL translation) + a couple of
//! self-hosted asm files + the shared leaf libs.
//!
//! Per-arch differences (from each build-<arch>/build.ninja `hal.dll` line):
//!   i386 : entry `_HalInitSystem@8` (stdcall `_`/`@8`); asm generic/systimer.S
//!          + generic/trap.S + pic/pic.S; static libs add libchkstk.a; a
//!          checked-in resource (pic/halpic.rc → data/hal-i386.rc.res).
//!   amd64: entry `HalInitSystem` (no decoration); asm apic/apictrap.S +
//!          apic/tsccal.S; static libs add libcntpr_rust.a (the cnt-rust
//!          cohabitation twin), no chkstk; NO resource (the amd64 link line
//!          carries no .res).
//! Both: image base 0x10000, native subsystem, 0x1000 file/section align,
//! `--exclude-all-symbols` (KernelDll bakes these in); the ntoskrnl import
//! lib is self-hosted from ntoskrnl.spec; hal.spec → hal.def via spec2def.
//!
//! The asm is assembled from SOURCE `.S` by the engine (HAL define set +
//! the kernel asm `-I` chain + the HAL-extra `-I` dirs below) — no offline
//! pre-expansion. Out-of-band input still checked in:
//!   - `tools/gen_hal_res.py` → data/hal-i386.rc.res (i386 only)
//!
//! Post-link pefixup / fix_lld_forwarding.py are applied by add_cargo_module
//! (KIND kerneldll) when this is driven from the main build.

use reactos_link::{AsmDefineSet, Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");

    // Per-arch: (target json, entry, asm sources, extra static libs). The
    // common leaf libs (cntpr/setjmp/arbiter/pseh/memcmp) are shared; each arch
    // appends what its cmake link line carries (i386 chkstk; amd64 cntpr_rust).
    let (target_json, entry, asm, extra_libs): (&str, &str, &[&str], &[&str]) = match arch.as_str()
    {
        // i386 pic-flavor (hal/halx86/pic.cmake): generic/systimer.S +
        // generic/trap.S + pic/pic.S. v86.S is GONE (pre-Vista V86 INT10 BIOS
        // path retired with fast486/x86bios, commit ebf1ac53a99).
        "x86" => (
            "i686-kernel.json",
            "_HalInitSystem@8",
            &[
                "hal/halx86/generic/systimer.S",
                "hal/halx86/generic/trap.S",
                "hal/halx86/pic/pic.S",
            ],
            // i386 cmake link adds libchkstk.a (no transitive propagation in
            // the cargo link, so list it explicitly). libcntpr_rust.a (the
            // cnt-rust cohabitation twin) is also needed: the cnt-rust
            // MATH-TIERC flip moved strnlen/wcsnlen into the Rust twin, so the C
            // liblibcntpr.a now references them undefined — same as amd64, which
            // already links libcntpr_rust.a.
            &["sdk/lib/crt/libchkstk.a", "sdk/lib/crt/libcntpr_rust.a"],
        ),
        // amd64 acpi/apic-flavor: the only non-empty component contributions
        // are apic/apictrap.S + apic/tsccal.S (generic + acpi are empty.c).
        "x86_64" => (
            "x86_64-kernel.json",
            "HalInitSystem",
            &["hal/halx86/apic/apictrap.S", "hal/halx86/apic/tsccal.S"],
            // amd64 cmake link adds libcntpr_rust.a (the cnt-rust cohabitation
            // twin); no chkstk.
            &["sdk/lib/crt/libcntpr_rust.a"],
        ),
        other => panic!("hal-cargo: unsupported arch {other} (i386/amd64 only; arm64 HAL is halarm64)"),
    };

    // Shared leaf static libs, in the cmake hal.dll link order. libhal_rust.a is
    // emitted first by the engine (rust_staticlib). The ntoskrnl import lib is
    // self-generated from ntoskrnl.spec (.import_lib below) — no cmake-built
    // libntoskrnl.a. clang_rt.builtins is added by the engine.
    let mut libs: Vec<&str> = vec![
        "sdk/lib/crt/liblibcntpr.a",
        "sdk/lib/vcruntime/libsetjmp.a",
        "sdk/lib/drivers/arbiter/libarbiter.a",
        "sdk/lib/pseh/libpseh.a",
    ];
    libs.extend_from_slice(extra_libs);
    libs.push("sdk/lib/crt/libmemcmp.a");

    let mut linker = Linker::new(target_json)
        .rust_staticlib("hal-rust")
        // Self-hosted asm: the engine assembles the SOURCE `.S` directly via the
        // HAL define set (AsmDefineSet::Hal) + the engine's kernel asm -I chain
        // (asm.inc + the generated build-<arch> ks*.inc + ntoskrnl/include) PLUS
        // the HAL-extra `-I` dirs below. No offline pre-expansion. Run `ninja
        // hal` in build-<arch> once first so the generated ks*.inc / sysfuncs.h
        // exist.
        .asm(asm, AsmDefineSet::Hal)
        // HAL-extra `-I` dirs (beyond the engine's built-in asm.inc + generated
        // ks* + ntoskrnl/include + -isystem crt): the broader header chain the
        // HAL `.S` resolve against (hal/halx86's headers, ddk/ndk/psdk/reactos,
        // pseh). Mirrors the old gen_preprocessed_asm_hal.py -I list.
        .asm_include_dirs(&[
            "sdk/include",
            "sdk/include/ddk",
            "sdk/include/ndk",
            "sdk/include/psdk",
            "sdk/include/reactos",
            "sdk/include/reactos/libs",
            "sdk/include/vcruntime",
            "hal/halx86/include",
            "sdk/lib/pseh/include",
        ])
        .spec2def("hal/hal.spec", "hal.dll")
        .static_libs(&libs)
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl");

    // i386 hal.dll carries the pic/halpic.rc resource (checked-in .res). amd64's
    // cmake link line has no .res, so skip it there.
    if arch == "x86" {
        linker = linker.resource("data/hal-i386.rc.res");
    }

    linker.link(ModuleKind::KernelDll {
        entry: entry.into(),
        image_base: 0x0001_0000,
    });
}
