//! Cargo-driven HAL link (i386 + amd64 + arm64), shared by both HAL cells.
//! Manifest for the shared `reactos-link` engine — all machinery lives there.
//!
//! **Two flavors, one recipe.** `hal-cargo` calls `link(Flavor::Up)` and
//! produces `hal.dll`; `halmp-cargo` calls `link(Flavor::Mp)` and produces
//! `halmp.dll`, which is the same HAL built with hal-rust's `smp` feature
//! (`HalpBuildType` drops the UNIPROCESSOR bit, `SYNCH_LEVEL` takes its MP
//! value). Everything else — per-arch entry point, image base, asm set, CRT
//! floor, spec2def, ntoskrnl import lib, resource — is identical BY
//! CONSTRUCTION, because there is only one copy of it: this function.
//!
//! Both ship on every media; freeldr picks one per boot entry via `/HAL=`
//! (`winldr.c` loads the named file and registers it in the loader block as
//! `hal.dll`, so imports resolve regardless of the on-disk name — the same
//! mechanism `[LiveCD_Aacpi]`'s `/HAL=halaacpi.dll` has always used). The
//! `halacpi.dll`/`halapic.dll`/`halaacpi.dll` names in txtsetup.sif remain
//! install-time aliases of `hal.dll`.
//!
//! i386 is the apic-flavor by default (generic/apic components; the legacy pic
//! flavor is the `VIBEOS_PIC_HAL` escape hatch), amd64 the acpi/apic-flavor —
//! both reduce to libhal_rust.a (the whole x86 HAL translation) + a couple of
//! self-hosted asm files + the in-cargo cnt-rust-lib/printf-shell CRT floor.
//! arm64 is pure-Rust stubs (hal-rust/src/arm64) → just libhal_rust.a, no
//! asm/resource.
//!
//! ZERO cmake `.static_libs()` .a on every arch (2026-06-20): the former leaf
//! reuses are all gone — liblibcntpr.a → cnt-rust-lib (.rust_lib) + in-cargo
//! printf shells; libpseh.a → dummy.c c_shim; libmemcmp.a → compiler-builtins-
//! mem; and libsetjmp.a / libarbiter.a / libchkstk.a DROPPED as inert vestigial
//! reuses (drop-test EMPTY — hal references no setjmp/arbiter/chkstk; clang_rt
//! covers __chkstk). The HAL now links a fully cargo-native floor.
//!
//! Per-arch differences (from each build-<arch>/build.ninja `hal.dll` line):
//!   i386 : entry `_HalInitSystem@8` (stdcall `_`/`@8`); asm generic/systimer.S
//!          + generic/trap.S + pic/pic.S; no cmake static libs (libchkstk.a
//!          dropped as inert); a checked-in resource (pic/halpic.rc →
//!          hal-cargo/data/hal-i386.rc.res).
//!   amd64: entry `HalInitSystem` (no decoration); asm apic/apictrap.S +
//!          apic/tsccal.S; no cmake static libs; NO resource (the amd64 link
//!          line carries no .res).
//! Both: image base 0x10000, native subsystem, 0x1000 file/section align,
//! `--exclude-all-symbols` (KernelDll bakes these in); the ntoskrnl import
//! lib is self-hosted from ntoskrnl.spec; hal.spec → the .def via spec2def.
//!
//! The asm is assembled from SOURCE `.S` by the engine (HAL define set +
//! the kernel asm `-I` chain + the HAL-extra `-I` dirs below) — no offline
//! pre-expansion. Out-of-band input still checked in:
//!   - `tools/gen_hal_res.py` → hal-cargo/data/hal-i386.rc.res (i386 only)
//!
//! Post-link pefixup / fix_lld_forwarding.py are applied by add_cargo_module
//! (KIND kerneldll) when this is driven from the main build.

use reactos_link::{AsmDefineSet, Linker, ModuleKind};
use std::env;

/// Which HAL this cell builds. The UP/MP split is expressed by WHICH CRATE you
/// build, not by an env var read inside a shared one: a `VIBEOS_SMP_HAL=1` that
/// silently turned `hal.dll` into an MP HAL was a footgun (a UP-named binary
/// with MP semantics passes every name-based check), and it forced a full HAL
/// relink + cmake reconfigure to switch flavors. Both flavors are now built
/// unconditionally and shipped side by side.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Flavor {
    /// `hal.dll` — uniprocessor, pairs with `ntoskrnl.exe`.
    Up,
    /// `halmp.dll` — hal-rust `smp` feature, pairs with `ntkrnlmp.exe`.
    Mp,
}

impl Flavor {
    fn dll_name(self) -> &'static str {
        match self {
            Self::Up => "hal.dll",
            Self::Mp => "halmp.dll",
        }
    }
}

/// Drive the HAL link for `flavor`. Call from the cell's `build.rs` `main()`.
pub fn link(flavor: Flavor) {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");

    // i386 HAL interrupt-model flavor. As of the i386-apic-hal lane landing the
    // APIC flavor (LAPIC/IOAPIC/TPR, lazy-IRQL, RTC clock) is the DEFAULT on
    // i386; the legacy 8259 PIC flavor is the opt-out escape hatch, selected by
    // VIBEOS_PIC_HAL=1 (cmake -DVIBEOS_PIC_HAL=ON via `vibeos-cli build
    // --hal-pic`). The APIC flavor builds hal-rust with the `apic` feature AND
    // assembles the APIC asm set (apictrap.S + tsccal.S) instead of the PIC set
    // (generic/systimer.S + generic/trap.S + pic/pic.S). amd64 is always the
    // APIC flavor regardless of this env. Ignored on arm64. (The PIC modules
    // stay in-tree, gated off — never deleted — so the flip is one
    // `git revert` away.) Applies to BOTH flavors: an MP PIC HAL is a valid
    // combination, so this composes with `smp` rather than overriding it.
    let apic_hal = arch == "x86" && env::var("VIBEOS_PIC_HAL").is_err();

    // cnt-rust-lib `float` feature, mirroring libcntpr.cmake CNT_RUST_FEATURES:
    // i386 omits it (no-SSE softfloat kernel), amd64/arm64 enable it. Used only
    // on x86/amd64 (the halx86 flavors that reuse libcntpr); arm64 halstubs
    // reference no cntpr.
    let is_x86 = arch == "x86" || arch == "x86_64";
    let cnt_feats: &[&str] = if arch == "x86" { &[] } else { &["float"] };

    // Per-arch: (target json, entry, image base, asm sources, static libs).
    // The cmake hal.dll leaf chain (cntpr/setjmp/arbiter/pseh/chkstk/memcmp) is
    // now FULLY cargo-native: cnt-rust-lib via .rust_lib + dummy.c c_shim +
    // compiler-builtins-mem cover cntpr/pseh/memcmp, and setjmp/arbiter/chkstk
    // were dropped as inert (drop-test EMPTY) → `libs` is empty on every arch.
    // x86/amd64 (halx86) still carry the self-hosted asm + the i386 printf
    // shells; arm64 (halarm64) is pure-Rust stubs (just libhal_rust.a + the
    // ntoskrnl import lib). image base: x86 0x10000; arm64
    // 0x7FFB00000000 (verbatim from build-arm64/build.ninja, == kdcom/uefildr).
    // libhal_rust.a is emitted first by the engine (rust_staticlib); the
    // ntoskrnl import lib is self-generated from ntoskrnl.spec (.import_lib
    // below), not a cmake libntoskrnl.a; clang_rt.builtins is added by the engine.
    //
    // The target json is manifest-relative, so each cell carries its own copy —
    // that is the established pattern (42 identical `i686-kernel.json` across
    // the workspace) AND a hard requirement: cmake's add_cargo_module builds the
    // i386 `--target` path as `${ws}/${crate}/i686-kernel.json`.
    let (target_json, entry, image_base, asm, libs): (&str, &str, u64, &[&str], Vec<&str>) =
        match arch.as_str()
    {
        // i386 pic-flavor (hal/halx86/pic.cmake): generic/systimer.S +
        // generic/trap.S + pic/pic.S. v86.S is GONE (pre-Vista V86 INT10 BIOS
        // path retired with fast486/x86bios, commit ebf1ac53a99).
        // cntpr's Rust half (was the cmake-copied libcntpr_rust.a) now comes
        // from the cargo-native sibling `cnt-rust-lib` via .rust_lib() below
        // (classpnp/framebuf pattern); libpseh.a → the dummy.c c_shim; memcmp →
        // compiler-builtins-mem (build-std). liblibcntpr.a (C residual) is
        // DROPPED — the sprintf/swprintf C-variadic shells come in-cargo via
        // c_from_compile_db (see below); setjmp/arbiter/chkstk DROPPED as inert
        // (see the `libs` comment below) → no cmake static libs.
        "x86" => (
            "i686-kernel.json",
            "_HalInitSystem@8",
            0x0001_0000,
            if apic_hal {
                // APIC flavor (i386-apic-hal lane), mirroring amd64's set:
                // apic/apictrap.S provides ALL FIVE i386 trap stubs
                // (HalpClock/ClockIpi/Profile/Apc/DispatchInterrupt) + the
                // spurious service, and apic/tsccal.S the TSC calibration ISR.
                // This set DELIBERATELY EXCLUDES the PIC set below
                // (generic/systimer.S + generic/trap.S + pic/pic.S) — trap.S and
                // pic.S define HalpApc/Clock/Profile/DispatchInterrupt too and
                // would collide (see the i386-apic-hal lane's asm audit).
                &["hal/halx86/apic/apictrap.S", "hal/halx86/apic/tsccal.S"] as &[&str]
            } else {
                &[
                    "hal/halx86/generic/systimer.S",
                    "hal/halx86/generic/trap.S",
                    "hal/halx86/pic/pic.S",
                ] as &[&str]
            },
            // liblibcntpr.a (C CRT) DROPPED — cnt-rust-lib (.rust_lib below)
            // provides hal's translatable libcntpr surface; the only irreducible
            // C floor i386 hal pulls is the two C-variadic printf shells
            // sprintf/swprintf (Rust stable can't define C variadics), compiled
            // in-cargo via c_from_compile_db below. Offline drop-test (nm: hal
            // undefined across libhal_rust + setjmp/arbiter/chkstk ∩ liblibcntpr
            // defs − cnt-rust-lib defs) = {_sprintf,_swprintf} on i386, EMPTY on
            // amd64.
            //
            // libsetjmp.a + libarbiter.a + libchkstk.a DROPPED (2026-06-20) — all
            // three were INERT vestigial reuses: the offline drop-test (undefined
            // across libhal_rust + the engine-assembled trap/timer/pic asm ∩ each
            // .a's defs) is EMPTY. i386 hal uses DUMMY_PSEH so it references no
            // setjmp/longjmp (setjmp-rust's own doc: "i386 uses DUMMY_PSEH... so
            // libsetjmp.a is inert there"); the body calls no Arb* arbiter API;
            // and clang_rt.builtins covers __chkstk (the engine links it into
            // every kernel DLL — win32k proved this ×3 incl arm64 cold-relink).
            // → ZERO cmake .static_libs() .a for hal.dll, all 3 arches.
            vec![],
        ),
        // amd64 acpi/apic-flavor: the only non-empty component contributions
        // are apic/apictrap.S + apic/tsccal.S (generic + acpi are empty.c).
        // cntpr Rust half → cnt-rust-lib (.rust_lib); pseh → dummy.c c_shim;
        // memcmp → compiler-builtins-mem; no chkstk (amd64 takes __chkstk from
        // clang_rt.builtins).
        "x86_64" => (
            "x86_64-pc-windows-gnu",
            "HalInitSystem",
            0x0001_0000,
            &["hal/halx86/apic/apictrap.S", "hal/halx86/apic/tsccal.S"],
            // liblibcntpr.a DROPPED — cnt-rust-lib provides amd64 hal's ENTIRE
            // libcntpr surface; drop-test residue is EMPTY (no C-variadic shell
            // needed; amd64 hal calls printf only via Rust-provided entry points).
            // libsetjmp.a + libarbiter.a DROPPED (2026-06-20) — INERT: drop-test
            // (undefined across libhal_rust + asm ∩ each .a's defs) is EMPTY.
            // amd64 hal references no setjmp/longjmp (only the kernel's SEH unwind
            // path does — setjmp-rust serves that; hal's body doesn't) and no Arb*
            // arbiter API. amd64 already had no chkstk (clang_rt.builtins). → ZERO
            // cmake .static_libs() .a for amd64 hal.dll.
            vec![],
        ),
        // arm64 (hal/halarm64): pure-Rust stubs (src/arm64/halstubs.rs). The
        // build-arm64/build.ninja hal.dll link line is just libhal_rust.a +
        // libntoskrnl.a (import lib, self-hosted below) + memcmp — no asm,
        // no resource, none of the x86 CRT/setjmp/arbiter/pseh leaf libs.
        // memcmp now comes from compiler-builtins-mem (build-std), so the last
        // cmake .a (libmemcmp.a) is dropped → arm64 hal.dll is ZERO cmake .a.
        "aarch64" => (
            "aarch64-pc-windows-gnullvm",
            "HalInitSystem",
            0x7FFB_0000_0000,
            &[],
            vec![],
        ),
        other => panic!("hal-link: unsupported arch {other}"),
    };

    let mut linker = Linker::new(target_json).rust_staticlib("hal-rust");

    // Flavor features. `smp` comes from WHICH CELL is building (Flavor::Mp),
    // never from an env var — see the Flavor doc comment. `apic` still comes
    // from VIBEOS_PIC_HAL's absence, and the two COMPOSE (an SMP APIC HAL sets
    // both). rust_features REPLACES the set, so pass one combined list rather
    // than two calls.
    println!("cargo:rerun-if-env-changed=VIBEOS_PIC_HAL");
    let mut hal_features: Vec<&str> = Vec::new();
    if flavor == Flavor::Mp {
        hal_features.push("smp");
    }
    if apic_hal {
        hal_features.push("apic");
    }
    if !hal_features.is_empty() {
        linker = linker.rust_features(&hal_features);
    }

    // x86/amd64 only: cntpr's Rust half as a cargo-native sibling staticlib
    // (was the cmake-copied libcntpr_rust.a) + the DUMMY_PSEH _SEH2_Volatile*
    // symbols compiled in-cargo from dummy.c (was libpseh.a). Both are
    // mechanism-identical reuses turned cargo-native — exactly classpnp/ntdll.
    // arm64 halstubs reference neither cntpr nor pseh.
    if is_x86 {
        // i386 also needs the two C-variadic printf shells (sprintf/swprintf =
        // thin `va_start; vsxprintf(...)` over the now-Rust streamout engine that
        // cnt-rust-lib provides). As of the WS1 cmake-off printf batch (2026-06-21)
        // they are self-contained/headerless c_shims (the shared _sxprintf.c engine
        // behind -DCRT_PRINTF_HEADERLESS, see c_shim_flags below) instead of a
        // compile_commands.json replay; verified disasm-identical (WS1 harness).
        // amd64 drop-test residue is empty → no shells (arm64 never linked cntpr).
        let mut hal_shims: Vec<&str> = vec!["sdk/lib/pseh/dummy.c"];
        if arch == "x86" {
            hal_shims.extend_from_slice(&[
                "sdk/lib/crt/printf/sprintf.c",
                "sdk/lib/crt/printf/swprintf.c",
            ]);
        }
        linker = linker
            .rust_lib("cnt-rust-lib", cnt_feats, true)
            .c_shims(&hal_shims)
            .c_shim_flags(&["-DCRT_PRINTF_HEADERLESS"]);
    }

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

    // One hal.spec serves both flavors — halmp.dll exports the identical HAL
    // surface, it is only compiled differently. `dll_name` is spec2def's `-n=`
    // (the name in the PE export directory); freeldr registers whichever file
    // it loads as `hal.dll` in the loader block, so nothing resolves against it.
    linker = linker
        .spec2def("hal/hal.spec", flavor.dll_name())
        .static_libs(&libs);

    // (the i386 printf shells moved up into the is_x86 c_shims block above —
    // WS1 cmake-off printf batch.)

    linker = linker.import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl");

    // i386 hal.dll carries the pic/halpic.rc resource (checked-in .res). amd64
    // and arm64 link lines carry no .res, so skip it there. The path is
    // manifest-relative and deliberately reaches into hal-cargo from BOTH cells
    // (`hal-cargo/../hal-cargo/…` is just `hal-cargo/…`), so there is one
    // checked-in binary blob rather than a copy per flavor to drift.
    if arch == "x86" {
        linker = linker.resource("../hal-cargo/data/hal-i386.rc.res");
    }

    // cmake's kernel-mode links keep every section.
    linker.keep_all_sections().link(ModuleKind::KernelDll {
        entry: entry.into(),
        image_base,
    });
}
