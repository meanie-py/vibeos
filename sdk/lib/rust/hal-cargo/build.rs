//! Cargo-driven `hal.dll` link (i386 + amd64 + arm64). Manifest for the
//! shared `reactos-link` engine — all machinery lives there.
//!
//! There is exactly one `hal.dll` flavor built per arch (cmake target `hal`,
//! in hal/halx86 for x86 and hal/halarm64 for arm64); the
//! `halacpi.dll`/`halapic.dll`/`halaacpi.dll` names in txtsetup.sif are
//! install-time aliases of it. i386 is the pic-flavor (generic/legacy/pic
//! components), amd64 the acpi/apic-flavor (generic/acpi/apic) — both reduce
//! to libhal_rust.a (the whole x86 HAL translation) + a couple of self-hosted
//! asm files + the shared leaf libs. arm64 is pure-Rust stubs
//! (hal-rust/src/arm64) → libhal_rust.a + libmemcmp.a, no asm/resource.
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

    // Per-arch: (target json, entry, image base, asm sources, static libs in
    // cmake hal.dll link order). x86/amd64 (halx86) carry the full leaf chain
    // (cntpr/setjmp/arbiter/pseh + per-arch extras + memcmp). arm64 (halarm64)
    // is pure-Rust stubs whose authoritative build.ninja link line is just
    // libhal_rust.a + the ntoskrnl import lib + libmemcmp.a — no asm, no
    // resource, none of the x86 leaf libs. image base: x86 0x10000; arm64
    // 0x7FFB00000000 (verbatim from build-arm64/build.ninja, == kdcom/uefildr).
    // libhal_rust.a is emitted first by the engine (rust_staticlib); the
    // ntoskrnl import lib is self-generated from ntoskrnl.spec (.import_lib
    // below), not a cmake libntoskrnl.a; clang_rt.builtins is added by the engine.
    let (target_json, entry, image_base, asm, libs): (&str, &str, u64, &[&str], Vec<&str>) =
        match arch.as_str()
    {
        // i386 pic-flavor (hal/halx86/pic.cmake): generic/systimer.S +
        // generic/trap.S + pic/pic.S. v86.S is GONE (pre-Vista V86 INT10 BIOS
        // path retired with fast486/x86bios, commit ebf1ac53a99). i386 cmake
        // link adds libchkstk.a (no transitive propagation in the cargo link).
        // libcntpr_rust.a (the cnt-rust cohabitation twin) is also needed:
        // cnt-rust now owns symbols the C liblibcntpr.a still references
        // (strnlen/wcsnlen from the CORE-OPS flip; streamout/wstreamout from the
        // printf wrappers, pulled when a cgu shift makes hal-rust reach sprintf).
        "x86" => (
            "i686-kernel.json",
            "_HalInitSystem@8",
            0x0001_0000,
            &[
                "hal/halx86/generic/systimer.S",
                "hal/halx86/generic/trap.S",
                "hal/halx86/pic/pic.S",
            ],
            vec![
                "sdk/lib/crt/liblibcntpr.a",
                "sdk/lib/vcruntime/libsetjmp.a",
                "sdk/lib/drivers/arbiter/libarbiter.a",
                "sdk/lib/pseh/libpseh.a",
                "sdk/lib/crt/libchkstk.a",
                "sdk/lib/crt/libcntpr_rust.a",
                "sdk/lib/crt/libmemcmp.a",
            ],
        ),
        // amd64 acpi/apic-flavor: the only non-empty component contributions
        // are apic/apictrap.S + apic/tsccal.S (generic + acpi are empty.c).
        // cmake link adds libcntpr_rust.a (cnt-rust twin); no chkstk.
        "x86_64" => (
            "x86_64-kernel.json",
            "HalInitSystem",
            0x0001_0000,
            &["hal/halx86/apic/apictrap.S", "hal/halx86/apic/tsccal.S"],
            vec![
                "sdk/lib/crt/liblibcntpr.a",
                "sdk/lib/vcruntime/libsetjmp.a",
                "sdk/lib/drivers/arbiter/libarbiter.a",
                "sdk/lib/pseh/libpseh.a",
                "sdk/lib/crt/libcntpr_rust.a",
                "sdk/lib/crt/libmemcmp.a",
            ],
        ),
        // arm64 (hal/halarm64): pure-Rust stubs (src/arm64/halstubs.rs). The
        // build-arm64/build.ninja hal.dll link line is just libhal_rust.a +
        // libntoskrnl.a (import lib, self-hosted below) + libmemcmp.a — no asm,
        // no resource, none of the x86 CRT/setjmp/arbiter/pseh leaf libs.
        "aarch64" => (
            "aarch64-kernel.json",
            "HalInitSystem",
            0x7FFB_0000_0000,
            &[],
            vec!["sdk/lib/crt/libmemcmp.a"],
        ),
        other => panic!("hal-cargo: unsupported arch {other}"),
    };

    let mut linker = Linker::new(target_json).rust_staticlib("hal-rust");

    // Self-hosted asm (x86 only): the engine assembles the SOURCE `.S` directly
    // via the HAL define set (AsmDefineSet::Hal) + the engine's kernel asm -I
    // chain (asm.inc + the generated build-<arch> ks*.inc + ntoskrnl/include)
    // PLUS the HAL-extra `-I` dirs below. No offline pre-expansion. Run `ninja
    // hal` in build-<arch> once first so the generated ks*.inc / sysfuncs.h
    // exist. arm64 has no HAL asm → skip (an empty .asm() has nothing to do).
    if !asm.is_empty() {
        linker = linker
            .asm(asm, AsmDefineSet::Hal)
            // HAL-extra `-I` dirs (beyond the engine's built-in asm.inc +
            // generated ks* + ntoskrnl/include + -isystem crt): the broader
            // header chain the HAL `.S` resolve against (hal/halx86's headers,
            // ddk/ndk/psdk/reactos, pseh). Mirrors the old
            // gen_preprocessed_asm_hal.py -I list.
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
            ]);
    }

    linker = linker
        .spec2def("hal/hal.spec", "hal.dll")
        .static_libs(&libs)
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl");

    // i386 hal.dll carries the pic/halpic.rc resource (checked-in .res). amd64
    // and arm64 link lines carry no .res, so skip it there.
    if arch == "x86" {
        linker = linker.resource("data/hal-i386.rc.res");
    }

    linker.link(ModuleKind::KernelDll {
        entry: entry.into(),
        image_base,
    });
}
