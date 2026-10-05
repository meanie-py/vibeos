//! Cargo-driven `win32k.sys` link (i386 / amd64 / arm64). Manifest for the
//! shared `reactos-link` engine — the Win32 kernel-mode subsystem driver
//! (GDI + NTUSER + DirectDraw), cmake `set_module_type(win32k
//! kernelmodedriver)` moved to `add_cargo_module(... KIND kernelmodedriver)`.
//!
//! Mirrors the cmake `win32k.sys` link (verified against each
//! build-<arch>/build.ninja `win32ss/win32k.sys` line):
//!   subsystem `native:5.01`, `-shared`, `--exclude-all-symbols` + 0x1000
//!   align (ModuleKind::KernelDll == the kernelmodedriver flag set; the
//!   init-section.lds ordering script is dropped exactly as in every other
//!   kernel-mode cargo cut — pefixup makes INIT discardable regardless).
//!   entry  `DriverEntry@8` — i386 `_DriverEntry@8`; amd64/arm64 `DriverEntry`
//!          (win32k-rust/src/main.rs).
//!   base   i386/amd64 `0x00010000`; arm64 `0x7FFB00000000`.
//!
//! The Rust body (`win32k-kernel-rust` → libwin32k_kernel_rust.a) builds with
//! the STANDARD user-mode target via `.rust_user_target()` — i386
//! `i686-pc-windows-gnu` + `RUSTFLAGS=-C target-feature=-sse,-sse2` — exactly
//! the cmake custom_command's proven configuration (NOT the softfloat kernel
//! json; switching targets would change codegen of the entire translated
//! body). The C residue (DirectDraw, gendib tables, FORCEINLINE shims,
//! safe_user_io, user/rtl, i386 floatobj + dib asm) stays cmake-compiled as
//! `libwin32k_residue.a` and is reused FIRST in the static-lib group so the
//! C-before-Rust cohab first-match precedence matches the old link, where the
//! residue was force-linked objects ahead of the Rust archive.
//!
//! Run `ninja` in build-<arch>/ first (the cargo link reuses only cmake's
//! genuine C residue — libwin32k_residue.a + libdxguid.a). The former CRT-floor
//! reuses are all dropped: libpseh.a + liblibcntpr.a (win32k references nothing
//! from pseh; the libcntpr C-variadic floor is compiled in-cargo via
//! c_from_compile_db), and now libcntpr_rust.a (→ `.rust_lib("cnt-rust-lib")`),
//! libmemcmp.a (→ compiler-builtins-mem via build-std) and libchkstk.a (→
//! clang_rt.builtins) — the classpnp-cargo fully-cargo-native CRT pattern. The
//! version resource is the
//! checked-in per-arch `data/win32k-<arch>.rc.res` (byte-identical to cmake's
//! win32k.rc.res; regenerate via tools/gen_win32k_res.py when version blocks
//! change). Post-link pefixup --kernelmodedriver / fix_lld_forwarding.py run
//! from add_cargo_module.

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");

    // Per-arch facts, verbatim from each build-<arch>/build.ninja:
    //   - hal_dir: hal import lib spec dir is shared (hal/hal.spec) but the
    //     arch matters only for the .a path, which we don't reuse — the engine
    //     self-hosts the import lib from the spec.
    let (target_json, res, entry, image_base): (&str, &str, &str, u64) = match arch.as_str() {
        "x86" => (
            "i686-kernel.json",
            "win32k-i386.rc.res",
            "_DriverEntry@8",
            0x0001_0000,
        ),
        "x86_64" => (
            "x86_64-pc-windows-gnu",
            "win32k-amd64.rc.res",
            "DriverEntry",
            0x0001_0000,
        ),
        "aarch64" => (
            "aarch64-pc-windows-gnullvm",
            "win32k-arm64.rc.res",
            "DriverEntry",
            0x7FFB_0000_0000,
        ),
        o => panic!("win32k-cargo: unsupported arch {o}"),
    };

    // cnt-rust-lib feature set: amd64/arm64 enable `float` (the forbid-unsafe
    // floatcvt/scanf cabi surface); i386 omits it (no-SSE-illegal by-value f64
    // returns stay C). Mirrors libcntpr.cmake's CNT_RUST_FEATURES + classpnp.
    let cnt_feats: &[&str] = if arch == "x86" { &[] } else { &["float"] };

    // cmake C / cohab static libs, in the cmake link-line order. The residue
    // archive comes FIRST (it was force-linked objects in the cmake link, so
    // archive-position-first preserves the C-first --allow-multiple-definition
    // cohab precedence over libwin32k_kernel_rust.a, which the engine emits
    // after this group via rust_staticlib).
    // WS5 cmake-off (2026-06-21): libwin32k_residue.a is now compiled IN-cargo
    // (the last win32k cmake .a). Its 13 members split by provenance:
    //   - 5 *_rust_shims.c + (i386) floatobj.c → c_from_compile_db (replay when
    //     cmake configured; committed-flags + native-hpp xdk + committed
    //     bugcodes.h when cmake-off — the blrtl pattern).
    //   - 3 gendib DIB tables (dib{8,16,32}gen.c, GENERATED) → the `.gendib()`
    //     marker: replay finds them under build-<arch>/; cmake-off runs the
    //     self-hosted native-gendib (byte-identical to cmake's).
    //   - (i386) 4 asm (3 dib*_hline/colorfill.s + floatobj.S) → asm_archive.
    // amd64/arm64 have only the 5 shims + 3 gendib (no floatobj/asm).
    let mut residue_c: Vec<&str> = vec![
        "win32ss/user/ntuser/main_rust_shims.c",
        "win32ss/gdi/ntgdi/ntbitblt_rust_shims.c",
        "win32ss/gdi/ntgdi/font_rust_shims.c",
        "win32ss/gdi/ntgdi/path_rust_shims.c",
        "win32ss/gdi/ntgdi/freetype_rust_shims.c",
        // gendib-generated (build-<arch>-relative; resolved per branch):
        "win32ss/gdi/dib/dib8gen.c",
        "win32ss/gdi/dib/dib16gen.c",
        "win32ss/gdi/dib/dib32gen.c",
    ];
    let residue_asm: &[&str] = if arch == "x86" {
        residue_c.push("win32ss/gdi/eng/i386/floatobj.c");
        &[
            "win32ss/gdi/dib/i386/dib24bpp_hline.s",
            "win32ss/gdi/dib/i386/dib32bpp_hline.s",
            "win32ss/gdi/dib/i386/dib32bpp_colorfill.s",
            "win32ss/gdi/eng/i386/floatobj.S",
            // x87 pow/fmod: this cell's cnt-rust-lib is the builtin no-SSE i686 build,
            // which leaves them to the asm (cnt-rust/src/cabi/math.rs, `pow`).
            "sdk/lib/crt/math/i386/pow_asm.s",
            "sdk/lib/crt/math/i386/fmod_asm.s",
        ]
    } else {
        &[]
    };

    let static_libs: Vec<&str> = vec![
        // libwin32k_residue.a → c_from_compile_db("win32k_residue") + .gendib()
        // + asm_archive (above/below) — 2026-06-21, the last win32k cmake .a.
        // libpseh.a DROPPED — win32k references NOTHING from it. Offline nm
        // drop-test (undefined across libwin32k_residue + libwin32k_kernel_rust
        // ∩ libpseh defs) = EMPTY on both i386 (full seh.s runtime + volatiles)
        // and amd64 (volatiles-only): win32k's DUMMY_PSEH handlers are dead code
        // and its body never references the `_SEH2_Volatile*` data symbols, so
        // the cmake reuse was vestigial. The SEH runtime export ABI lives in
        // ntoskrnl (pseh-rust), not here. C kept on disk. arm64 (volatiles-only,
        // same arch-independent refs as amd64) verified at the gate's cold link.
        // libdxguid.a DROP-TEST 2026-06-20 — DirectX/Direct3DRM GUID *data* (1228
        // .rdata CLSID_/IID_ consts, 0 text). Offline residue (win32k undefined ∩
        // dxguid-defined) = EMPTY → win32k.sys doesn't reference these user-mode
        // DX GUIDs. Empirical build+boot is the oracle (vista pattern).
        // liblibcntpr.a (cmake C CRT) DROPPED — cnt-rust-lib (.rust_lib below)
        // provides the whole translatable surface; the only irreducible C floor
        // win32k pulls is the C-variadic printf shells (Rust stable can't define
        // C variadics), compiled in-cargo via c_from_compile_db below. Offline
        // drop-test (nm: win32k undefined ∩ liblibcntpr defs − libcntpr_rust
        // defs) = i386 needs only _vsnwprintf; the full shell set is lazy-pull +
        // cross-arch safe (nothing else defines them — zero dup risk).
        //
        // The remaining three cmake .a are now ALSO dropped — win32k links a
        // fully cargo-native CRT floor, exactly the classpnp-cargo pattern (the
        // largest driver after win32k, booting ×3 with the same drops):
        //   - libcntpr_rust.a → `.rust_lib("cnt-rust-lib", cnt_feats, true)`
        //     (cmake built the same Rust crate into libcntpr_rust.a; building it
        //     as a cargo-native sibling is symbol-identical, ntdll/classpnp/hal
        //     pattern).
        //   - libmemcmp.a → compiler-builtins-mem (build-std, pulled in by the
        //     `.rust_lib(.., true)` calls). The one C `memcmp` fn is now a Rust
        //     builtin.
        //   - libchkstk.a → clang_rt.builtins provides `__chkstk` on all 3 arches
        //     (the engine links clang_rt into every kernel DLL; classpnp's cold
        //     link confirmed nothing else referenced the cmake stack-probe asm).
        //     The win32k cold link is the oracle if any arch still pulls it.
    ];

    // The irreducible C-variadic printf shells (sprintf/_snwprintf/… = thin
    // `va_start; vsnwprintf(...)` over the now-Rust streamout engine). Arch-
    // independent. Mirrors ntoskrnl-cargo's cntpr_residue.
    let cntpr_residue: &[&str] = &[
        "sdk/lib/crt/printf/sprintf.c",
        "sdk/lib/crt/printf/swprintf.c",
        "sdk/lib/crt/printf/vsprintf.c",
        "sdk/lib/crt/printf/_snprintf.c",
        "sdk/lib/crt/printf/_snwprintf.c",
        "sdk/lib/crt/printf/_vsnprintf.c",
        "sdk/lib/crt/printf/_vsnwprintf.c",
        // _vscwprintf (wide char-count) — win32k's body pulls it transitively;
        // its only dep is wstreamout, which the Rust CRT provides.
        "sdk/lib/crt/printf/_vscwprintf.c",
    ];

    let mut linker = Linker::new(target_json)
        // The translated body — built with the user hardfloat target exactly
        // as cmake's custom_command did (see module doc above).
        .rust_staticlib("win32k-kernel-rust")
        .rust_user_target()
        // cnt-rust-lib (libcntpr's Rust half) built as a cargo-native sibling
        // staticlib instead of reusing cmake's libcntpr_rust.a — ntdll/classpnp/
        // hal pattern. The `true` (build-std) also pulls compiler-builtins-mem,
        // which is what lets libmemcmp.a be dropped.
        .rust_lib("cnt-rust-lib", cnt_feats, true)
        // win32k-kernel-rust is an umbrella; the actual translation lives in
        // win32k-rust (a path dep) — watch it so edits re-run this build.
        .rust_watch(&["win32k-rust/src"])
        .resource(&format!("data/{res}"))
        // Exports (Eng*/FLOATOBJ_*/XLATEOBJ_*/…) via the spec2def-generated
        // win32k.def. The cmake-side spec2def(... ADD_IMPORTLIB) stays — the
        // libwin32k import lib consumers (framebuf etc.) are independent.
        .spec2def("win32ss/win32k.spec", "win32k.sys");
    if arch == "x86" {
        // cmake: RUST_WIN32K_EXTRA_ENV "RUSTFLAGS=-C target-feature=-sse,-sse2"
        // — kernel-mode code must not touch SSE state on i386.
        linker = linker.rust_build_rustflags("-C target-feature=-sse,-sse2");
    }
    linker = linker
        .static_libs(&static_libs)
        // libwin32k_residue.a compiled in-cargo (WS5): the 5 shims + (i386)
        // floatobj.c + 3 gendib tables, replay/committed dual-branch + native-
        // gendib. C-first in the link group (before the Rust body), preserving
        // the cohab precedence the old residue-archive-first reuse had.
        .c_from_compile_db("win32k_residue", &residue_c)
        .gendib("win32k_residue");
    if !residue_asm.is_empty() {
        // i386 dib/floatobj asm (was in libwin32k_residue.a) → in-group archive.
        linker = linker.asm_archive("win32k_residue_asm", residue_asm);
    }
    linker
        // Compile the cntpr C-variadic printf floor in-cargo as self-contained
        // headerless c_shims (WS1 cmake-off, 2026-06-21): the shells #include the
        // shared _sxprintf.c / _vscwprintf.c engine, now headerless behind
        // -DCRT_PRINTF_HEADERLESS — cc-rs compiles them directly, no
        // compile_commands.json replay. Each verified disasm-identical vs the
        // header build (WS1 harness). This is win32k's ONLY former C floor, so it
        // now links free of cmake's compile_commands.json.
        .c_shims(cntpr_residue)
        .c_shim_flags(&["-DCRT_PRINTF_HEADERLESS"])
        // cmake add_importlibs(win32k ntoskrnl hal ftfd), self-hosted from
        // each .spec (spec2def + dlltool) like every other cargo cut.
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .import_lib("hal/hal.spec", "hal.dll", "hal")
        .import_lib("win32ss/drivers/font/ftfd/ftfd.spec", "ftfd.dll", "ftfd")
        .link(ModuleKind::KernelDll {
            entry: entry.into(),
            image_base,
        });
}
