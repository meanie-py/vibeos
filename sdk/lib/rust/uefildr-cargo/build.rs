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
                // i386: mini_hal + libhal_rust_mini are linked BEFORE the
                // freeldr libs (uefi.cmake `if(ARCH STREQUAL i386 AND NOT MSVC)`).
                libs.push("hal/halx86/minihal/libmini_hal.a");
                libs.push("hal/halx86/libhal_rust_mini.a");
                // uefiasm.S + i386trap.S flipped to Rust global_asm!
                // (freeldr-rust/uefiasm.rs + loader_asm.rs) — no asm left here.
                ("i686-kernel.json", "_EfiEntry", Some(0x0040_0000), 0x200)
            }
            "x86_64" => {
                // uefiasm.S + misc.S flipped to Rust global_asm!
                // (freeldr-rust/uefiasm.rs + loader_asm.rs) — no asm left here.
                ("x86_64-kernel.json", "EfiEntry", Some(0x1_0000), 0x200)
            }
            "aarch64" => {
                // uefiasm.S flipped to Rust global_asm! — arm64 has no other asm.
                ("aarch64-kernel.json", "EfiEntry", None, 0x1000)
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
        "sdk/lib/cportlib/libcportlib.a",
        // cport.c is Rust now (cohabitation); libcportlib.a is just the stub,
        // the Cp* serial symbols come from libcport_rust.a.
        "sdk/lib/cportlib/libcport_rust.a",
        // blcmlib (CM hive boot lib) is now compiled IN-cargo via
        // .c_from_compile_db() below (13 C sources, no asm), not reused .a.
        "boot/rtl/libblrtl.a",
        "sdk/lib/crt/liblibcntpr.a",
        "sdk/lib/crt/libchkstk.a",
        // cnt-rust-lib (the Rust cnt cohabitation twin) is now a DIRECT cargo
        // sibling (.rust_lib() below), not the cmake-copied libcntpr_rust.a.
    ]);

    // blcmlib's 13 C sources (sdk/lib/cmlib/CMakeLists.txt) compiled in-cargo by
    // replaying cmake's clang cmd (flag-exact incl. _BLDR_ + the cmlib.h PCH) —
    // produces a symbol-identical libblcmlib.a without `ninja blcmlib`. C-only,
    // no asm. cmboot-rust (in freeldr-rust) calls these Hv*/Cm* symbols.
    const BLCMLIB_C: &[&str] = &[
        "sdk/lib/cmlib/cmcheck.c",
        "sdk/lib/cmlib/cminit.c",
        "sdk/lib/cmlib/cmheal.c",
        "sdk/lib/cmlib/cmindex.c",
        "sdk/lib/cmlib/cmkeydel.c",
        "sdk/lib/cmlib/cmname.c",
        "sdk/lib/cmlib/cmse.c",
        "sdk/lib/cmlib/cmvalue.c",
        "sdk/lib/cmlib/hivebin.c",
        "sdk/lib/cmlib/hivecell.c",
        "sdk/lib/cmlib/hiveinit.c",
        "sdk/lib/cmlib/hivesum.c",
        "sdk/lib/cmlib/hivewrt.c",
    ];

    // cnt-rust-lib features mirror sdk/lib/crt/libcntpr.cmake: i386 builds
    // WITHOUT `float` (no-SSE pre-boot, no by-value f64 ABI); amd64/arm64 with
    // `float`. build_rust_crate's cnt-rust-lib special case supplies the exact
    // per-arch RUSTFLAGS (i386 -sse,-sse2 -C no-prepopulate-passes; else
    // -C no-prepopulate-passes). Same .rust_lib() path ntdll-cargo uses.
    let cnt_feats: &[&str] = if arch == "x86" { &[] } else { &["float"] };

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
        // blcmlib C compiled in-cargo (compile_commands.json replay) — drops the
        // cmake libblcmlib.a reuse. In the C-archive position (before rust libs).
        .c_from_compile_db("blcmlib", BLCMLIB_C)
        // cnt cohabitation twin built directly (was reused cmake libcntpr_rust.a).
        .rust_lib("cnt-rust-lib", cnt_feats, true)
        .static_libs(&libs)
        .c_shims(&["boot/freeldr/freeldr/lib/strsafe_shim.c"]);
    // i386 freeldr runs in protected mode before CR4.OSFXSR — no SSE. Build
    // freeldr-rust with -sse,-sse2 (the retired custom_command's i386 RUSTFLAGS;
    // amd64/arm64 ran with empty RUSTFLAGS → SSE on, as in long mode).
    if arch == "x86" {
        linker = linker.rust_build_rustflags("-C target-feature=-sse,-sse2");
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
