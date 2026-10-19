//! Cargo-driven `uefildr.efi` link (i386 + amd64 + arm64). Manifest for the
//! shared `reactos-link` engine. UEFI sibling of `kdcom-cargo`'s build.rs.
//!
//! Mirrors the cmake `uefildr.efi` link line (from build-<arch>/build.ninja,
//! target `boot/freeldr/freeldr/uefildr.efi`):
//!   subsystem  `efi_application:5.01`         (set_subsystem 10)
//!   align      file/section 0x200             (UEFI app)
//!   gc         `--gc-sections`, NOT -shared   (it's an exe)
//!   exports    none (`--exclude-all-symbols`; the .def/freeldr.spec is only
//!              for the freeldr *import lib* rosload consumes, NOT for
//!              uefildr's own exports — so NO .spec2def() here. Passing it
//!              forces every spec export as required, and the UEFI build
//!              genuinely lacks a few (`PxeCallApi`,
//!              `DriveMapMapDrivesInSection` — gated out of the Rust uefi
//!              feature, defined only in the C BIOS libfreeldr.a) →
//!              `<root>: undefined symbol`. cmake's link never feeds the
//!              .def to the linker either; matching it = no .def.)
//!
//! Per-arch differences (verbatim from each build.ninja):
//!   i386 : entry `_EfiEntry` (cdecl `_` prefix), auto image base
//!          (0x00400000), 0x200 align, links mini_hal + libhal_rust_mini
//!          (the i386-only freeldr mini-HAL).
//!   amd64: entry `EfiEntry` (no decoration), `--image-base 0x10000` (keep
//!          the loader in the low 1GB freeldr maps), 0x200 align, NO mini_hal.
//!   arm64: entry `EfiEntry`, NO `--image-base` (auto-hash base), 0x1000
//!          file/section align, NO mini_hal (same lib set as amd64).
//!
//! reactos-link builds everything uefildr-specific from source: the freeldr-rust
//! body comes from `libfreeldr_rust_uefi.a` (the EfiEntry entry, built `--features
//! uefi,uefildr-rust-entry`), strsafe_shim.c via `.c_shims()`, and the per-arch
//! asm (uefiasm.S + i386:i386trap.S + amd64:misc.S) via `.asm()`. The old
//! `uefifreeldr_common` cmake aggregate is retired (all its C is Rust, its asm
//! moved here). Only genuinely-shared leaf libs (blcmlib/blrtl/libcntpr/cport/…)
//! stay cmake-built. reactos-link wraps them in --start-group/--end-group.

use reactos_link::{Linker, ModuleKind};
use std::env;

// blrtl-rust's path-deps (its own src is tracked by the .rust_lib build).
const BLRTL_RUST_DEPS: &[&str] =
    &["bitmap/src", "compress/src", "crc32/src", "largeint/src", "random/src", "byteswap/src", "mem/src", "rtl-diag/src"];

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");

    // strsafe_shim.c — the variadic RtlStringCbPrintfA shim Rust stable can't
    // host. Despite living under freeldr's 30-include compile env in cmake, it
    // is a self-contained LEAF: zero #includes (it self-defines NTSTATUS /
    // va_list / STATUS_* and only externs libcntpr's `_vsnprintf`). So
    // reactos-link compiles it directly via .c_shims() below — no cmake OBJECT
    // lib (`uefildr_shim`), no fragile reused-.obj path into the freeldr env.
    let mut libs: Vec<&str> = Vec::new();

    // Per-arch link knobs (verbatim from each build-<arch>/build.ninja's
    // uefildr.efi LINK_FLAGS):
    //   entry      i386 `_EfiEntry` (cdecl `_`); amd64/arm64 `EfiEntry`.
    //   image_base i386 cmake auto-resolves to the exe default 0x00400000;
    //              amd64 cmake pins 0x10000 (low 1GB freeldr maps); arm64
    //              cmake passes no --image-base → None (auto-hash base).
    //   alignment  x86 (i386/amd64) 0x200; arm64 0x1000.
    // All of uefildr's asm (uefiasm.S, i386:i386trap.S, amd64:misc.S) is now
    // Rust global_asm! in freeldr-rust (uefiasm.rs + loader_asm.rs), so it flows
    // through libfreeldr_rust_uefi.a — no .S, no reactos-link .asm() here.
    let (target_json, entry, image_base, alignment): (&str, &str, Option<u64>, u64) =
        match arch.as_str() {
            "x86" => {
                // libmini_hal.a DROP-TEST 2026-06-20 — its Rust HAL impl is a
                // direct cargo sibling (.rust_lib("hal-rust",["minihal"]) below).
                // The cmake asm-half (systimer.S → _HalpCalibrateStallExecution@0)
                // is INERT: complete offline residue across ALL uefildr i686 rlibs
                // (incl hal-rust[minihal]) = 0 — nothing references it. Empirical
                // UEFI-boot is the oracle.
                // libs.push("hal/halx86/minihal/libmini_hal.a");
                // uefiasm.S + i386trap.S flipped to Rust global_asm!
                // (freeldr-rust/uefiasm.rs + loader_asm.rs) — no asm left here.
                ("i686-kernel.json", "_EfiEntry", Some(0x0040_0000), 0x200)
            }
            "x86_64" => {
                // uefiasm.S + misc.S flipped to Rust global_asm!
                // (freeldr-rust/uefiasm.rs + loader_asm.rs) — no asm left here.
                ("x86_64-pc-windows-gnu", "EfiEntry", Some(0x1_0000), 0x200)
            }
            "aarch64" => {
                // uefiasm.S flipped to Rust global_asm! — arm64 has no other asm.
                ("aarch64-pc-windows-gnullvm", "EfiEntry", None, 0x1000)
            }
            other => {
                panic!("uefildr-cargo: unsupported arch {other} (i386/amd64/arm64 only)")
            }
        };

    libs.extend_from_slice(&[
        // libuefifreeldr_common.a retired — it was asm-only (all freeldr C is
        // Rust now), and that asm is assembled directly above. The freeldr-rust
        // body is now a DIRECT cargo build (.rust_staticlib() below), no longer
        // a cmake-built libfreeldr_rust_uefi.a reused here.
        // cport: libcportlib.a is NOT linked here — cport-rust (.rust_lib below)
        // supplies all the Cp* NTAPI serial symbols directly. The cmake
        // libcportlib.a bundles its own copy of cport_rust (PUBLIC link), so
        // linking it too was a redundant double-cport (masked by
        // --allow-multiple-definition); the cohab stub symbol it also carries
        // (__cportlib_rust_cohab_stub) and the C file-global RingIndicator are
        // unreferenced (cport-rust folds RingIndicator into internal state).
        // blcmlib (CM hive boot lib): the C is GONE — the loader now reads the
        // SYSTEM hive via `cmlib-rust` built with the `bldr` feature (mirrors the
        // old C blcmlib's -D_BLDR_ reduced surface). See .rust_lib("cmlib-rust",
        // &["bldr"]) below + cmlib_loader_stubs.c (da9c72b240d).
        // blrtl (boot RTL leaf) is compiled IN-cargo below: its C via
        // .c_from_compile_db("blrtl", …) + its per-arch asm via
        // .asm_archive("blrtl_asm", …), not reused boot/rtl/libblrtl.a.
        // libcntpr (cntpr C floor) likewise compiled IN-cargo below via
        // .c_from_compile_db("libcntpr", …) (cmake-off WS1 DEEP) — was the
        // reused cmake "sdk/lib/crt/liblibcntpr.a". Its i386/amd64 MSVC asm
        // intrinsics (_alldiv/_ftol2/_chkstk/…) are unreferenced by freeldr-rust
        // (a *-windows-gnu target that uses gnu-named helpers __divdi3/
        // ___chkstk_ms from compiler_builtins), so no asm_archive is needed —
        // offline residue confirms zero undefined libcntpr-asm symbols.
        // libchkstk.a DROP-TEST 2026-06-20 — INERT: complete offline residue
        // (chkstk defs __chkstk/___alloca/__alloca_probe{,_16} ∩ truly-unresolved
        // across ALL uefildr i686 rlibs) = 0, no >4KB-frame refs; clang_rt.builtins
        // backstops __chkstk on all 3 arches. Empirical UEFI-boot is the oracle.
        // "sdk/lib/crt/libchkstk.a",
        // cnt-rust-lib (the Rust cnt cohabitation twin) is now a DIRECT cargo
        // sibling (.rust_lib() below), not the cmake-copied libcntpr_rust.a.
    ]);

    // blrtl (boot-loader RTL, boot/rtl/blrtl.cmake): the blrtl-rust crate (the kernel's
    // RTL rlibs + nlsboot, with blrtl.cmake's per-arch membership) as a .rust_lib below;
    // what is still C compiles in-cargo via compile_commands.json replay (target `blrtl`
    // disambiguates the shared sdk/lib/rtl/*.c sources); the per-arch asm via
    // .asm_archive() below.
    let blrtl_c: &[&str] = &["sdk/lib/rtl/unicode.c"];
    // blrtl's per-arch asm (asm.inc only — AsmDefineSet::Kernel). i386:
    // debug_asm/rtlmem/rtlswap; amd64: debug_asm; arm64: none (no ASM_SOURCE).
    let blrtl_asm: &[&str] = match arch.as_str() {
        "x86" => &[
            "sdk/lib/rtl/i386/debug_asm.S",
            "sdk/lib/rtl/i386/rtlmem.s",
            "sdk/lib/rtl/i386/rtlswap.S",
        ],
        "x86_64" => &["sdk/lib/rtl/amd64/debug_asm.S"],
        _ => &[],
    };

    // libcntpr (cntpr C floor, sdk/lib/crt/libcntpr.cmake) compiled in-cargo via
    // c_from_compile_db — was the reused cmake liblibcntpr.a (cmake-off WS1 DEEP).
    // The common C set (math _invoke_matherr/exp2f, mbstring, misc fltused, the 12
    // printf shells, string atol/wctype/winesup, scanf_trampoline); i386 adds the
    // x87 math wrappers (ci*/libm_sse2/nexttowardf), amd64 adds ehandler.c. Source
    // list is a subset of the cmake `libcntpr` target's objects (with no cmake build,
    // their lines of clibs/<arch>/libcntpr.recipe). asm intrinsics are
    // GC'd (gnu target) — no asm_archive.
    let mut libcntpr_c: Vec<&str> = vec![
        "sdk/lib/crt/math/_invoke_matherr.c",
        "sdk/lib/crt/math/exp2f.c",
        "sdk/lib/crt/mbstring/mbstrlen.c",
        "sdk/lib/crt/misc/fltused.c",
        "sdk/lib/crt/printf/_snprintf.c",
        "sdk/lib/crt/printf/_snwprintf.c",
        "sdk/lib/crt/printf/_swprintf.c",
        "sdk/lib/crt/printf/_vscprintf.c",
        "sdk/lib/crt/printf/_vscwprintf.c",
        "sdk/lib/crt/printf/_vsnprintf.c",
        "sdk/lib/crt/printf/_vsnwprintf.c",
        "sdk/lib/crt/printf/sprintf.c",
        "sdk/lib/crt/printf/swprintf.c",
        "sdk/lib/crt/printf/vprintf.c",
        "sdk/lib/crt/printf/vsprintf.c",
        "sdk/lib/crt/printf/vswprintf.c",
        "sdk/lib/crt/string/atol.c",
        "sdk/lib/crt/string/wctype.c",
        "sdk/lib/crt/string/winesup.c",
        "sdk/lib/rust/cnt-rust/src/cabi/scanf_trampoline.c",
    ];
    match arch.as_str() {
        "x86" => libcntpr_c.extend_from_slice(&[
            "sdk/lib/crt/math/nexttowardf.c",
            "sdk/lib/crt/math/i386/ci.c",
            "sdk/lib/crt/math/i386/cicos.c",
            "sdk/lib/crt/math/i386/cilog.c",
            "sdk/lib/crt/math/i386/cipow.c",
            "sdk/lib/crt/math/i386/cisin.c",
            "sdk/lib/crt/math/i386/cisqrt.c",
            "sdk/lib/crt/math/i386/libm_sse2.c",
        ]),
        "x86_64" => libcntpr_c.push("sdk/lib/crt/except/amd64/ehandler.c"),
        _ => {}
    }

    // cnt-rust-lib features mirror sdk/lib/crt/libcntpr.cmake: i386 builds
    // WITHOUT `float` (no-SSE pre-boot, no by-value f64 ABI); amd64/arm64 with
    // `float`. build_rust_crate's cnt-rust-lib special case supplies the exact
    // per-arch RUSTFLAGS (i386 -sse,-sse2 -C no-prepopulate-passes; else
    // -C no-prepopulate-passes). Same .rust_lib() path ntdll-cargo uses.
    // `freeldr`: the loader's own iswctype (freeldr-rust) is the only one.
    let cnt_feats: &[&str] = if arch == "x86" { &["freeldr"] } else { &["float", "freeldr"] };

    let mut linker = Linker::new(target_json)
        // freeldr-rust body built DIRECTLY by reactos-link (was the cmake
        // custom_command → libfreeldr_rust_uefi.a). .rust_user_target() builds
        // it on the STANDARD target (i686/x86_64-pc-windows-gnu / aarch64-
        // gnullvm, stable, no build-std) exactly as the custom_command did —
        // NOT the x86-softfloat kernel json, whose float ABI would diverge.
        .rust_staticlib("freeldr-rust")
        .rust_features(&["uefi", "uefildr-rust-entry"])
        .rust_user_target()
        // cmboot-rust is a path-dep of freeldr-rust — watch it so an edit there
        // re-runs build.rs and rebuilds the body (freeldr-rust's own src is
        // tracked by build_rust_crate).
        // cmboot-rust (freeldr-rust dep) + cnt-rust/cnt-ctype (cnt-rust-lib
        // path-deps) — watch so an edit re-runs build.rs (each crate's OWN src
        // is auto-tracked by build_rust_crate; these are the path-deps).
        .rust_watch(&["cmboot-rust/src", "cnt-rust/src", "cnt-ctype/src"])
        .rust_watch(BLRTL_RUST_DEPS)
        // Bootloader cmlib: Rust crate built with the `bldr` feature, matching
        // the old C blcmlib's -D_BLDR_ reduced hive surface.
        .rust_lib("cmlib-rust", &["bldr"], true)
        // blrtl C compiled in-cargo (target `blrtl` disambiguates the shared
        // rtl sources) — drops the cmake boot/rtl/libblrtl.a reuse.
        .rust_lib("blrtl-rust", &[], true)
        .c_from_compile_db("blrtl", blrtl_c)
        // libcntpr C compiled in-cargo — drops the cmake liblibcntpr.a reuse
        // (cmake-off WS1 DEEP). cmake-off path uses cflags/libcntpr.<arch>.flags.
        .c_from_compile_db("libcntpr", &libcntpr_c)
        // cnt cohabitation twin built directly (was reused cmake libcntpr_rust.a).
        .rust_lib("cnt-rust-lib", cnt_feats, true)
        // cport-rust (NS16550 serial) built directly (was reused cmake
        // libcport_rust.a). Default features, all arches; build_rust_crate keys
        // the i386 -sse,-sse2 (cmake got it from .cargo/config.toml).
        .rust_lib("cport-rust", &[], true)
        .static_libs(&libs)
        .c_shims(&[
            "boot/freeldr/freeldr/lib/strsafe_shim.c",
            "boot/freeldr/freeldr/lib/cmlib_loader_stubs.c",
        ]);
    // i386 freeldr runs in protected mode before CR4.OSFXSR — no SSE. Build
    // freeldr-rust with -sse,-sse2 (the retired custom_command's i386 RUSTFLAGS;
    // amd64/arm64 ran with empty RUSTFLAGS → SSE on, as in long mode).
    if arch == "x86" {
        linker = linker.rust_build_rustflags("-C target-feature=-sse,-sse2");
        // hal-rust minihal variant (i386-only) built directly (was reused cmake
        // libhal_rust_mini.a). Default features + `minihal`; build_rust_crate
        // keys the i386 -sse,-sse2 (cmake's HAL_RUST_EXTRA_ENV). The systimer.S
        // asm half (libmini_hal.a) is still reused via static_libs above.
        linker = linker.rust_lib("hal-rust", &["minihal"], true);
    }
    // blrtl's per-arch asm as an in-group archive (mirrors blrtl.cmake's
    // add_asm_files+add_library, where the asm was part of libblrtl.a). arm64
    // has no blrtl asm — skip (an empty asm_archive would panic).
    if !blrtl_asm.is_empty() {
        linker = linker.asm_archive("blrtl_asm", blrtl_asm);
    }
    // strsafe_shim.c: an emitted movaps pre-boot would #UD. Match
    // uefifreeldr_common's -mno-sse (x86 only; invalid on aarch64).
    if arch == "x86" || arch == "x86_64" {
        linker = linker.c_shim_flags(&["-mno-sse"]);
    }
    linker.link(ModuleKind::EfiApplication {
        entry: entry.into(),
        image_base,
        alignment,
    });
}
