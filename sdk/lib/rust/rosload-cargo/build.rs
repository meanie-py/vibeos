//! Cargo-driven `rosload.exe` link (i386 + amd64). Manifest for the shared
//! `reactos-link` engine. Native-subsystem sibling of `uefildr-cargo`'s
//! build.rs.
//!
//! Mirrors the cmake `rosload.exe` link (from build-<arch>/build.ninja,
//! target `boot/freeldr/freeldr/rosload.exe`):
//!   subsystem  `native:5.01`                  (set_subsystem native)
//!   entry      i386 `_RunLoader`, amd64 `RunLoader`  (set_entrypoint RunLoader)
//!   imagebase  `0x10000`                       (set_image_base 0x10000)
//!   gc         `--gc-sections`, NOT -shared    (it's an exe)
//!   exports    `rosload.def` from rosload.spec (one export: RtlAssert) — so
//!              NO `--exclude-all-symbols` (ENABLE_EXPORTS TRUE). spec2def
//!              `-n=rosload.exe`.
//!   align      DEFAULT (cmake passes no --file/-section-alignment).
//!   i386 only  `-u _HalDispatchTable` / `-u _HalPrivateDispatchTable`
//!              force-undefined (pull the Rust halstub HAL dispatch tables;
//!              unreferenced+unexported → GC'd without -u) + the i386 mini-HAL
//!              libs (libmini_hal.a + libhal_rust_mini.a) linked first.
//!
//! Inputs are the cmake-built artifacts (run `ninja rosload` in build-<arch>/
//! first): the prebuilt objects compiled into rosload.dir (strsafe_shim.c.obj,
//! cmboot.c.obj, per-arch asm), `libfreeldr_rust_rosload.a` (the freeldr-rust
//! body + the RunLoader entry, built `--features rosload`), and the sibling
//! leaf libs. The freeldr import lib is self-hosted by reactos-link from
//! freeldr.spec (.import_lib()) — no cmake-built libfreeldr.a needed.
//! reactos-link wraps them in --start-group/--end-group.

use reactos_link::{Linker, ModuleKind};
use std::env;

// blrtl-rust's path-deps (its own src is tracked by the .rust_lib build).
const BLRTL_RUST_DEPS: &[&str] =
    &["bitmap/src", "compress/src", "crc32/src", "largeint/src", "random/src", "byteswap/src", "mem/src", "rtl-diag/src"];

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");

    // rosload-specific compilation is now all reactos-link's: strsafe_shim.c via
    // .c_shims(), and ALL the asm (linux.S + amd64:misc.S) flipped to Rust
    // global_asm! in freeldr-rust (loader_asm.rs → libfreeldr_rust_rosload.a).
    // cmboot is Rust (cmboot-rust). The SOLE remaining cmake-built reused object
    // is the i386 16-bit drvmap_gcc.obj (clang can't .code16); rosload_objs gone.
    let (target_json, entry, mut libs): (&str, &str, Vec<&str>) = match arch.as_str() {
        "x86" => {
            // linux.S flipped to Rust global_asm! (freeldr-rust/loader_asm.rs).
            (
                "i686-kernel.json",
                "_RunLoader",
                vec![
                    // drvmap: 16-bit gcc asm, reused directly.
                    "boot/freeldr/freeldr/drvmap_gcc.obj",
                    // libmini_hal.a DROP-TEST 2026-06-20 — INERT: complete offline
                    // residue across ALL rosload i686 rlibs = 0
                    // (_HalpCalibrateStallExecution@0 unreferenced; the Rust
                    // mini-HAL .rust_lib("hal-rust",["minihal"]) below covers it).
                    // Empirical boot is the oracle.
                    // "hal/halx86/minihal/libmini_hal.a",
                ],
            )
        }
        "x86_64" => {
            // misc.S + linux.S flipped to Rust global_asm! (loader_asm.rs).
            ("x86_64-pc-windows-gnu", "RunLoader", Vec::new())
        }
        other => panic!("rosload-cargo: unsupported arch {other} (i386/amd64 only; arm64 is UEFI-only, no rosload)"),
    };

    // Common C leaf libs (cm/rtl/cntpr) + the cnt-rust cohabitation twin. The
    // rosload Rust body is now a DIRECT cargo build (.rust_staticlib() below),
    // not a reused cmake .a. The freeldr import lib is also self-hosted by
    // reactos-link from freeldr.spec (.import_lib() below), so cmake's
    // spec2def(freeldr.sys ... ADD_IMPORTLIB) is retired.
    libs.extend_from_slice(&[
        // blcmlib (CM hive boot lib): the C is GONE — the loader reads the SYSTEM
        // hive via `cmlib-rust` built with the `bldr` feature (mirrors the old C
        // blcmlib's -D_BLDR_ reduced surface). See .rust_lib("cmlib-rust",
        // &["bldr"]) below + cmlib_loader_stubs.c (da9c72b240d).
        // blrtl (boot RTL leaf) is compiled IN-cargo below (C via
        // .c_from_compile_db("blrtl", …) + asm via .asm_archive("blrtl_asm", …)),
        // not reused boot/rtl/libblrtl.a.
        // libcntpr (cntpr C floor) likewise compiled IN-cargo via
        // .c_from_compile_db("libcntpr", …) (cmake-off WS1 DEEP) — was the reused
        // cmake "sdk/lib/crt/liblibcntpr.a". Its i386 MSVC asm intrinsics
        // (_alldiv/_ftol2/_chkstk/…) are unreferenced by freeldr-rust (an
        // i686-pc-windows-gnu target using gnu helpers __divdi3/___chkstk_ms), so
        // no asm_archive — chkstk still comes from libchkstk.a below.
        // cnt-rust-lib is now a DIRECT cargo sibling (.rust_lib() below), not
        // the cmake-copied libcntpr_rust.a.
    ]);

    // blrtl (boot-loader RTL, boot/rtl/blrtl.cmake): the blrtl-rust crate (the kernel's
    // RTL rlibs + nlsboot, with blrtl.cmake's per-arch membership) as a .rust_lib below;
    // what is still C compiles in-cargo via compile_commands.json replay (target `blrtl`
    // disambiguates the shared sdk/lib/rtl/*.c sources); the per-arch asm via
    // .asm_archive() below.
    let blrtl_c: &[&str] = &["sdk/lib/rtl/unicode.c"];
    // blrtl's per-arch asm (asm.inc only — AsmDefineSet::Kernel). i386:
    // debug_asm/rtlmem/rtlswap; amd64: debug_asm. (rosload is i386/amd64 only.)
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
    // Common C set + i386 x87 math wrappers / amd64 ehandler.c; mirrors the cmake
    // `libcntpr` target objects (tools/gen_cflags.py + cflags/libcntpr.<arch>.flags).
    // asm intrinsics GC'd (gnu target) — no asm_archive. (rosload is i386/amd64.)
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

    // cnt-rust-lib features mirror sdk/lib/crt/libcntpr.cmake: i386 no `float`,
    // amd64 `float`. build_rust_crate's cnt-rust-lib special case supplies the
    // per-arch RUSTFLAGS. Same path ntdll-cargo uses. (rosload is i386/amd64.)
    // `freeldr`: the loader's own iswctype (freeldr-rust) is the only one.
    let cnt_feats: &[&str] = if arch == "x86" { &["freeldr"] } else { &["float", "freeldr"] };

    let mut linker = Linker::new(target_json)
        // freeldr-rust body built DIRECTLY by reactos-link (was the cmake
        // custom_command → libfreeldr_rust_rosload.a). .rust_user_target()
        // builds the STANDARD target (stable, no build-std) as that command
        // did — NOT the x86-softfloat kernel json. The `rosload` feature
        // selects the 2nd-stage-loader module set + the RunLoader entry.
        .rust_staticlib("freeldr-rust")
        .rust_features(&["rosload"])
        .rust_user_target()
        // cmboot-rust is a path-dep of freeldr-rust — watch it so an edit there
        // re-runs build.rs (freeldr-rust's own src is tracked by build_rust_crate).
        // cmboot-rust (freeldr-rust dep) + cnt-rust/cnt-ctype (cnt-rust-lib
        // path-deps) — watch so an edit re-runs build.rs.
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
        .static_libs(&libs)
        // strsafe_shim.c compiled directly (self-contained leaf). rosload is
        // i386/amd64 only — both x86 — so -mno-sse always applies (freeldr
        // pre-boot has SSE disabled; an emitted movaps would #UD).
        .c_shims(&[
            "boot/freeldr/freeldr/lib/strsafe_shim.c",
            "boot/freeldr/freeldr/lib/cmlib_loader_stubs.c",
        ])
        .c_shim_flags(&["-mno-sse"])
        // rosload exports RtlAssert via rosload.def (ENABLE_EXPORTS TRUE).
        .spec2def("boot/freeldr/freeldr/rosload.spec", "rosload.exe")
        // freeldr import lib, self-hosted from freeldr.spec (was the cmake
        // spec2def(freeldr.sys freeldr.spec ADD_IMPORTLIB) → libfreeldr.a).
        // rosload's freeldr.sys imports bind at runtime against uefildr.efi,
        // which the PE loader registers as "freeldr.sys" and which exports the
        // same freeldr.spec symbols. Same spec → identical import names/dll.
        .import_lib("boot/freeldr/freeldr/freeldr.spec", "freeldr.sys", "freeldr");

    if arch == "x86" {
        // i386 freeldr runs in protected mode before CR4.OSFXSR — no SSE.
        // Build freeldr-rust with -sse,-sse2 (the retired custom_command's i386
        // RUSTFLAGS; amd64 ran empty → SSE on, as in long mode).
        linker = linker.rust_build_rustflags("-C target-feature=-sse,-sse2");
        // The C halstub.c always linked the Rust HAL dispatch tables as a
        // source object; as archive members they'd be GC'd. Pull them.
        linker = linker.force_undefined(&["_HalDispatchTable", "_HalPrivateDispatchTable"]);
        // hal-rust minihal variant (i386-only) built directly (was reused cmake
        // libhal_rust_mini.a). build_rust_crate keys the i386 -sse,-sse2. The
        // systimer.S asm half (libmini_hal.a) stays reused via the libs above.
        linker = linker.rust_lib("hal-rust", &["minihal"], true);
    }
    // blrtl's per-arch asm as an in-group archive (mirrors blrtl.cmake's
    // add_asm_files+add_library, where the asm lived in libblrtl.a).
    if !blrtl_asm.is_empty() {
        linker = linker.asm_archive("blrtl_asm", blrtl_asm);
    }

    linker.link(ModuleKind::NativeExe {
        entry: entry.into(),
        image_base: 0x1_0000,
    });
}
