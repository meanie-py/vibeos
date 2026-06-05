//! Cargo-driven `hal.dll` link (i386). Manifest for the shared
//! `reactos-link` engine — all machinery lives there.
//!
//! The asm is assembled from SOURCE `.S` by the engine (HAL asm -I chain),
//! so there is no offline pre-expansion. Out-of-band input still checked in:
//!   - `tools/gen_hal_res.py`              → data/hal-i386.rc.res
//!
//! Post-link pefixup / fix_lld_forwarding.py run separately (see
//! build-and-test.sh).

use reactos_link::{AsmDefineSet, Linker, ModuleKind};

fn main() {
    Linker::new("i686-kernel.json")
        .rust_staticlib("hal-rust")
        // Self-hosted asm: the engine assembles the SOURCE `.S` directly via the
        // HAL define set (AsmDefineSet::Hal) + the engine's kernel asm -I chain
        // (asm.inc + the generated build-i386 ks386.inc + ntoskrnl/include) PLUS
        // the HAL-extra `-I` dirs below — exactly the chain the old
        // tools/gen_preprocessed_asm_hal.py used. No offline pre-expansion (the
        // retired data/asm/hal-*-i386.s). Run `ninja hal` in build-i386 once
        // first so the generated ks386.inc / sysfuncs.h exist.
        //
        // The set mirrors the cmake pic-flavor HAL (hal/halx86/pic.cmake):
        // generic/systimer.S + generic/trap.S + pic/pic.S. v86.S is GONE — the
        // pre-Vista V86-mode INT10 BIOS path was retired with fast486/x86bios
        // (commit ebf1ac53a99); the old data/asm/hal-v86-i386.s blob was stale
        // dead code that cmake no longer builds.
        .asm(
            &[
                "hal/halx86/generic/systimer.S",
                "hal/halx86/generic/trap.S",
                "hal/halx86/pic/pic.S",
            ],
            AsmDefineSet::Hal,
        )
        // HAL-extra `-I` dirs (beyond the engine's built-in sdk/include/asm +
        // generated ks386 + ntoskrnl/include + the -isystem sdk/include/crt):
        // the broader header chain the HAL `.S` resolve against (hal/halx86's
        // own headers, the ddk/ndk/psdk/reactos trees, pseh). Mirrors the -I
        // list the old gen_preprocessed_asm_hal.py passed.
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
        .resource("data/hal-i386.rc.res")
        .spec2def("hal/hal.spec", "hal.dll")
        // cmake link order (from the `hal.dll` ninja line). libhal_rust.a
        // is emitted first by the engine. The ntoskrnl import lib is
        // self-generated from ntoskrnl.spec (.import_lib below) — no
        // cmake-built libntoskrnl.a needed. The remaining leaf static
        // libs still come from cmake (deferred mechanical pass).
        .static_libs(&[
            "sdk/lib/crt/liblibcntpr.a",
            "sdk/lib/vcruntime/libsetjmp.a",
            "sdk/lib/drivers/arbiter/libarbiter.a",
            // fast486 dropped — the fast486/x86bios emulation engine was retired
            // from the HAL (commit ebf1ac53a99); the cmake hal.dll link no longer
            // references it (target_link_libraries: libcntpr setjmp arbiter pseh
            // + hal_rust). libchkstk/libmemcmp stay — the cargo link lacks
            // cmake's transitive propagation, so they're listed explicitly.
            "sdk/lib/pseh/libpseh.a",
            "sdk/lib/crt/libchkstk.a",
            "sdk/lib/crt/libmemcmp.a",
        ])
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .link(ModuleKind::KernelDll {
            entry: "_HalInitSystem@8".into(),
            image_base: 0x0001_0000,
        });
}
