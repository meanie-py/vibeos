//! Cargo-driven `ntoskrnl.exe` link (i386 / amd64 / arm64). Manifest for the
//! shared `reactos-link` engine — all machinery lives there.
//!
//! Tier-B sibling Rust crates (rtl-rust / cnt-rust-lib) are cargo-built by the
//! engine via `rust_lib()`; cmake still owns the deep-C / cohab static libs
//! (librtl.a, liblibcntpr.a, libhal.a, …). Run `ninja ntoskrnl` in the matching
//! build-<arch> once first (also generates the asm includes: ks*.inc /
//! sysfuncs.h). The asm is assembled from SOURCE `.S` by the engine (kernel asm
//! -I chain), so there is no offline pre-expansion. Only out-of-band input:
//!   - `tools/gen_ntoskrnl_res.py --arch <a>` → data/ntoskrnl-<arch>.rc.res
//!
//! Post-link pefixup / fix_lld_forwarding.py run separately.

use reactos_link::{AsmDefineSet, Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");

    // Per-arch facts, all verbatim from each build-<arch>/build.ninja's
    // `ntoskrnl/ntoskrnl.exe` link line:
    //   - Rust target spec (i386 custom kernel target for x86-softfloat;
    //     amd64/arm64 builtin triples for cmake's precompiled-std path)
    //   - res basename (gen_ntoskrnl_res.py output)
    //   - entry: i386 stdcall-decorated `_KiSystemStartup@4`; amd64/arm64 cdecl
    //     `KiSystemStartup`. image-base is 0x00400000 on every arch.
    //   - asm: the per-arch `.S` set the cmake kernel target compiles
    //   - cnt_float: cnt-rust-lib `float` feature — i386 omits it (no-SSE
    //     softfloat kernel), amd64/arm64 enable it (libcntpr.cmake CNT_RUST_*).
    // (libchkstk.a DROPPED 2026-06-20: the i386 cntpr_asm below assembles
    // chkstk_asm.s — which PUBLICs all 4 stack-probe symbols __chkstk/
    // __alloca_probe/__alloca_probe_16/___alloca — and clang_rt.builtins covers
    // __chkstk on amd64/arm64, so the ReactOS libchkstk.a was redundant.)
    let (rust_target, res, entry, asm, cnt_float): (
        &str,
        &str,
        &str,
        Vec<&str>,
        bool,
    ) = match arch.as_str() {
        "x86" => (
            "i686-kernel.json",
            "ntoskrnl-i386.rc.res",
            "_KiSystemStartup@4",
            vec![
                "ntoskrnl/ex/zw.S",
                "ntoskrnl/ex/i386/fastinterlck_asm.S",
                "ntoskrnl/ex/i386/ioport.S",
                "ntoskrnl/ke/i386/ctxswitch.S",
                "ntoskrnl/ke/i386/trap.s",
                "ntoskrnl/ke/i386/usercall_asm.S",
            ],
            false,
        ),
        "x86_64" => (
            "x86_64-kernel.json",
            "ntoskrnl-amd64.rc.res",
            "KiSystemStartup",
            vec![
                "ntoskrnl/ex/zw.S",
                "ntoskrnl/ke/amd64/boot.S",
                "ntoskrnl/ke/amd64/ctxswitch.S",
                "ntoskrnl/ke/amd64/trap.S",
                "ntoskrnl/ke/amd64/usercall_asm.S",
                "ntoskrnl/ke/amd64/zeropage.S",
            ],
            true,
        ),
        "aarch64" => (
            "aarch64-pc-windows-gnullvm",
            "ntoskrnl-arm64.rc.res",
            "KiSystemStartup",
            vec![
                "ntoskrnl/ex/zw.S",
                "ntoskrnl/ke/arm64/ctxswitch.S",
                "ntoskrnl/ke/arm64/trap.S",
                "ntoskrnl/ke/arm64/usercall_asm.S",
                "ntoskrnl/ke/arm64/zwdispatch.S",
            ],
            true,
        ),
        o => panic!("ntoskrnl-cargo: unsupported arch {o}"),
    };

    let cnt_feats: &[&str] = if cnt_float { &["float"] } else { &[] };

    // Header-free C shims (no kernel -I; clang's own stdarg/stddef/stdint).
    // kdbg_rust_shims.c is x86-only — it references the kdbg pager symbols
    // (KdbOutputAborted / KdbpPagerInternal) that ntoskrnl-rust's kdb_cli.rs
    // defines only on x86 (arm64 has no integrated kdb). cmake's arm64 link
    // drops it for the same reason; including it on arm64 leaves those symbols
    // undefined at link.
    let mut c_shims: Vec<&str> = vec![
        "ntoskrnl/io/iomgr/io_rust_shims.c",
        "ntoskrnl/ke/ke_rust_shims.c",
        "ntoskrnl/mm/mm_rust_shims.c",
        "ntoskrnl/variadic_shims.c",
        "ntoskrnl/lto_workaround.c",
        "ntoskrnl/mm_lock_outlines.c",
        "ntoskrnl/ntprintf_trampoline.c",
        "sdk/lib/crt/mem/memcpy.c",
        "sdk/lib/crt/mem/memmove.c",
        "sdk/lib/crt/mem/memset.c",
    ];
    if arch != "aarch64" {
        c_shims.push("ntoskrnl/kdbg/kdbg_rust_shims.c");
    }

    // cmake C / cohab static libs (the rust archives — libkernel_rust.a,
    // librtl_rust.a, libcntpr_rust.a — are emitted by the engine AFTER this
    // group via rust_staticlib/rust_lib below, preserving the C-first
    // --allow-multiple-definition cohab precedence). Order mirrors the cmake
    // link line; the engine wraps the whole set in --start-group so inter-C
    // order is non-critical.
    let mut static_libs: Vec<&str> = vec![
        "sdk/lib/drivers/ntoskrnl_vista/libntoskrnl_vista.a",
        // cportlib/csq/arbiter/lsalib/rossym cohab → cargo-native `.rust_lib`
        // (the 5 calls below) 2026-06-20. Both halves of each cohab dropped from
        // this cmake-.a list: the C bodies (libcportlib/libcsq/libarbiter/
        // libntlsalib/librossym .a) were empty rust_cohab_stub placeholders —
        // 0 real symbols (libcportlib's lone def, ___cportlib_rust_cohab_stub,
        // is unreferenced) — and the cmake-BUILT libX_rust.a are superseded by
        // building those same crates (cport-rust/csq-rust/arbiter-rust/
        // lsalib-rust/rossym-rust) cargo-native, exactly the rtl-rust/
        // cnt-rust-lib/cmlib-rust pattern. The `bacd1fc64d3` x86-ungate block
        // (which re-added the cmake libX_rust.a on x86 too) is likewise gone —
        // the `.rust_lib` calls below serve all 3 arches uniformly.
        // libpseh.a DROPPED → `pseh-rust` (.rust_lib below, all arches). It
        // provides the 2 DUMMY_PSEH `_SEH2_Volatile*` data symbols (referenced
        // by the C cohab archives' pseh2.h) on every arch, plus the i386 SEH
        // runtime (`_except_handler2/3`/`_global_unwind2`/`_local_unwind2`/
        // `_abnormal_termination`, exported by ntoskrnl.spec for -arch=i386,arm)
        // via global_asm! disasm-identical to the cmake seh.s. C kept on disk.
        // libcmlib.a DROPPED 2026-06-20 — it was an empty placeholder TU (0
        // defined symbols; all 13 cmlib .c are REMOVE_ITEM'd) since the kernel
        // cmlib body is 100% Rust via `.rust_lib("cmlib-rust")` below. The
        // placeholder contributed nothing to the link.
        // librtl.a DROPPED 2026-06-20 — the whole sdk/lib/rtl/*.c surface is Rust
        // (rtl-rust .rust_lib below, 5681 syms); librtl.a was down to the
        // irreducible floor (34 syms i386 / 18 amd64): the C-variadic DbgPrint
        // family (dbgprint_variadic.c) + per-arch asm (except/mem/swap/debug),
        // now compiled in-cargo via the rtl c_from_compile_db + rtl_asm
        // asm_archive below (the cntpr pattern). No translation gap.
        // liblibcntpr.a (the cmake C CRT) is GONE — cnt-rust-lib provides the
        // whole translatable surface (string/mem/ctype/search/printf+scanf
        // engines/full math); the irreducible C/asm floor is compiled in-cargo
        // (cntpr_residue / cntpr_asm below). See cargo-drive-unification-plan.md.
        // libsetjmp.a DROPPED → `setjmp-rust` (amd64 .rust_lib below). i386
        // (DUMMY_PSEH) + arm64 don't reference setjmp at all (drop-test: link
        // clean without it); amd64's SEH unwind path needs _setjmp/_setjmpex/
        // longjmp. C kept on disk for hal/ftfd cmake consumers.
        // wdmguid/poguid/ioevent (the 3 cmake C GUID libs) → `guids-rust`
        // (.rust_lib below): 217 deduped #[no_mangle] GUID data constants. C
        // kept on disk for any remaining cmake consumers.
        // libhal.a (the cmake-built hal.dll IMPORT lib) DROPPED 2026-06-20 — the
        // kernel now self-hosts the hal import from hal.spec via .import_lib
        // below (the classpnp/win32k pattern). hal.dll itself is 100% hal-rust
        // (cargo-driven); this was only the import library (95 short-import
        // members → hal.dll), not a HAL body.
        // libkdcom.a / libbootvid.a (the cmake-built kdcom.dll / bootvid.dll
        // IMPORT libs) DROPPED 2026-06-20 — the kernel now self-hosts both
        // imports from their .spec via .import_lib below (the hal pattern).
        // kdcom.spec = 8 stdcall exports (Kd{D0,D3}Transition/DebuggerInitialize
        // {0,1}/ReceivePacket/Restore/Save/SendPacket); bootvid.spec = 11 Vid*.
        // Symbol-identical: the cmake .a is itself spec2def-generated from these
        // same specs. bootvid.dll is cargo-driven (bootvid-cargo); these were
        // only the import libraries, not bodies.
    ];

    // (cportlib/csq/arbiter/lsalib/rossym are now cargo-native `.rust_lib` — see
    // the calls in the linker chain below. Historically these were cmake-built
    // libX_rust.a added to static_libs, gated arm64-only then x86-ungated in
    // `bacd1fc64d3`; the layout shift once tipped a latent cmlib Hv* hive-write
    // miscompile [cold amd64 HANG at hivewrt.c "Failed to write the primary
    // hive"], now guarded by the cmlib-rust no-prepopulate-passes class-guard in
    // reactos-link. Building the crates cargo-native instead of reusing cmake's
    // copy keeps the same guard in force; the amd64 cold boot is the oracle.)

    // cnt-rust integration: the irreducible C floor cnt-rust-lib can't provide
    // (enumerated by a drop-test against liblibcntpr.a). Compiled in-cargo via
    // c_from_compile_db (replays compile_commands.json, symbol-identical to the
    // cmake objects). i386 additionally needs an asm archive — see cntpr_asm.
    //   - amd64: the variadic printf shells (Rust stable can't define C
    //     variadics) + the amd64 SEH personality (except/amd64/ehandler.c →
    //     __C_specific_handler/_local_unwind).
    //   - i386: the _CIxxx x87 MSVC math intrinsic wrappers (ci*.c — they CALL
    //     the now-Rust libm, e.g. _CIsqrt→sqrt) + the variadic printf shells +
    //     _fltused. (No SEH personality — i386 uses DUMMY_PSEH.) The 64-bit
    //     integer + ftol + chkstk asm go through cntpr_asm below.
    // Archive members pull lazily, so the per-arch list is a SUPERSET safe for
    // both ntoskrnl and ntdll (ntoskrnl gets _fltused/chkstk from hal etc., so
    // those objects simply aren't pulled here). arm64 needs NO residue at all —
    // its drop-test came back with zero undefined (cnt-rust-lib + kernel-rust
    // cover the whole surface; clang_rt.builtins-aarch64 covers __chkstk).
    let cntpr_residue: Vec<&str> = match arch.as_str() {
        "x86_64" => vec![
            "sdk/lib/crt/except/amd64/ehandler.c",
            "sdk/lib/crt/printf/sprintf.c",
            "sdk/lib/crt/printf/swprintf.c",
            "sdk/lib/crt/printf/vsprintf.c",
            "sdk/lib/crt/printf/_snprintf.c",
            "sdk/lib/crt/printf/_snwprintf.c",
            "sdk/lib/crt/printf/_vsnprintf.c",
            "sdk/lib/crt/printf/_vsnwprintf.c",
        ],
        "x86" => vec![
            "sdk/lib/crt/math/i386/cicos.c",
            "sdk/lib/crt/math/i386/cisin.c",
            "sdk/lib/crt/math/i386/cisqrt.c",
            "sdk/lib/crt/math/i386/cilog.c",
            "sdk/lib/crt/math/i386/cipow.c",
            "sdk/lib/crt/misc/fltused.c",
            "sdk/lib/crt/printf/sprintf.c",
            "sdk/lib/crt/printf/swprintf.c",
            "sdk/lib/crt/printf/vsprintf.c",
            "sdk/lib/crt/printf/_snprintf.c",
            "sdk/lib/crt/printf/_snwprintf.c",
            "sdk/lib/crt/printf/_vsnprintf.c",
            "sdk/lib/crt/printf/_vsnwprintf.c",
        ],
        // arm64: the variadic printf shells only. The original 93d5ba8 drop-test
        // found arm64 needed NOTHING — but that predated linking cmlib-rust here.
        // cmlib's hive code pulls sprintf/swprintf/vsprintf/_sn*/_vsn*, which Rust
        // stable can't define (C variadics), so compile the shells in-cargo. No
        // SEH/_fltused/_CIxxx/chkstk (cdecl variadic ABI + clang_rt.builtins-aarch64
        // + cnt-rust libm cover the rest), and no scanf/_vscwprintf (ntdll-only).
        "aarch64" => vec![
            "sdk/lib/crt/printf/sprintf.c",
            "sdk/lib/crt/printf/swprintf.c",
            "sdk/lib/crt/printf/vsprintf.c",
            "sdk/lib/crt/printf/_snprintf.c",
            "sdk/lib/crt/printf/_snwprintf.c",
            "sdk/lib/crt/printf/_vsnprintf.c",
            "sdk/lib/crt/printf/_vsnwprintf.c",
        ],
        _ => Vec::new(),
    };
    // i386 asm floor: the 64-bit integer intrinsics (_alldiv/_allmul/…/_aullshr),
    // the float→int conversion (_ftol), and the stack probe (_chkstk /
    // _alloca_probe / ___alloca). Assembled from SOURCE .s via the engine's
    // kernel asm -I chain into an ARCHIVE (lazy-pull, layout-safe — the object
    // form relocates the image and trips the i386 hivewrt miscompile; see
    // ntdll-cargo's ntdllsys note). cnt-rust owns the libm x87 (sin/cos/sqrt/…),
    // so only the non-math asm stays.
    let cntpr_asm: &[&str] = if arch == "x86" {
        &[
            "sdk/lib/crt/math/i386/alldiv_asm.s",
            "sdk/lib/crt/math/i386/alldvrm_asm.s",
            "sdk/lib/crt/math/i386/allmul_asm.s",
            "sdk/lib/crt/math/i386/allrem_asm.s",
            "sdk/lib/crt/math/i386/allshl_asm.s",
            "sdk/lib/crt/math/i386/allshr_asm.s",
            "sdk/lib/crt/math/i386/aulldiv_asm.s",
            "sdk/lib/crt/math/i386/aulldvrm_asm.s",
            "sdk/lib/crt/math/i386/aullrem_asm.s",
            "sdk/lib/crt/math/i386/aullshr_asm.s",
            "sdk/lib/crt/math/i386/ftol_asm.s",
            "sdk/lib/crt/except/i386/chkstk_asm.s",
        ]
    } else {
        &[]
    };
    // liblibcntpr.a is DROPPED on every arch: cnt-rust-lib + the per-arch C/asm
    // residue above cover the whole surface (drop-test oracle: amd64 needs the
    // SEH + printf shells, i386 needs _CIxxx + printf shells + integer/chkstk
    // asm, arm64 needs NOTHING extra — cnt-rust-lib + kernel-rust already provide
    // it all). Keeping it would only make cnt-rust dead-cohab (C-first wins).

    // librtl.a's irreducible asm floor (the `.c` surface is all rtl-rust now):
    // per-arch except/mem/swap/debug asm. Assembled in-cargo via asm_archive
    // (lazy in-group archive, same asm_set as .asm() = Kernel + the kernel asm
    // -I chain — identical to how cmake compiled them into librtl.a). The
    // C-variadic DbgPrint family (dbgprint_variadic.c) goes via c_from_compile_db
    // below. Together these replace the dropped librtl.a.
    let rtl_asm: &[&str] = match arch.as_str() {
        "x86" => &[
            "sdk/lib/rtl/i386/debug_asm.S",
            "sdk/lib/rtl/i386/except_asm.s",
            "sdk/lib/rtl/i386/rtlmem.s",
            "sdk/lib/rtl/i386/rtlswap.S",
        ],
        "x86_64" => &[
            "sdk/lib/rtl/amd64/debug_asm.S",
            "sdk/lib/rtl/amd64/except_asm.S",
            "sdk/lib/rtl/amd64/unwind-asm.s",
        ],
        "aarch64" => &["sdk/lib/rtl/arm64/debug_asm.S"],
        _ => &[],
    };

    let mut linker = Linker::new(rust_target);
    linker = linker
        .rust_staticlib("kernel-rust")
        // Tier-B sibling Rust crates: cargo-compiled directly by the engine (was
        // reuse of the cmake-built librtl_rust.a / libcntpr_rust.a). Built
        // against the same target mode as the main kernel-rust crate: i386 uses
        // the softfloat kernel json + build-std; amd64/arm64 use cmake's builtin
        // precompiled-std targets. The engine emits these AFTER the cmake C
        // archives (C-first cohab precedence) and applies the per-crate RUSTFLAGS
        // cmake gives them (i386 -sse + the class-wide -C no-prepopulate-passes;
        // see reactos-link build_rust_crate). cnt-rust-lib `float` is per-arch.
        .rust_lib("rtl-rust", &[], true)
        .rust_lib("cnt-rust-lib", cnt_feats, true)
        // cmlib hive library: flipped C→Rust (kernel cmlib is 100% Rust). The
        // cmake `libcmlib.a` in static_libs is now just the cohab placeholder —
        // the real Hv*/Cmp*/Cm* symbols live in cmlib-rust (DEFAULT features, the
        // full kernel variant; NOT `bldr`, which is the loader's read-only build).
        .rust_lib("cmlib-rust", &[], true)
        // The kernel's GUID data libs (wdmguid/poguid/ioevent C) → Rust.
        .rust_lib("guids-rust", &[], true)
        // The 5 cohab driver/serial libs, now cargo-native (was the cmake-built
        // libX_rust.a in static_libs + the empty C-body stubs, both dropped
        // above). Same crates cmake compiled into libX_rust.a — building them
        // here via the engine is symbol-identical. All 3 arches uniformly.
        .rust_lib("cport-rust", &[], true)
        .rust_lib("csq-rust", &[], true)
        .rust_lib("arbiter-rust", &[], true)
        .rust_lib("lsalib-rust", &[], true)
        .rust_lib("rossym-rust", &[], true)
        // Header-free C shims (built above; kdbg shim is x86-only).
        .c_shims(&c_shims)
        // Self-hosted asm: the engine assembles the SOURCE `.S` directly via its
        // kernel asm -I chain (asm.inc + the generated build-<arch> ks*.inc +
        // ntoskrnl/include) — no offline pre-expansion. The set is per-arch
        // (see the match above).
        .asm(&asm, AsmDefineSet::Kernel)
        .resource(&format!("data/{res}"))
        .spec2def("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe");
    // cnt-rust integration: compile the cntpr C floor in-cargo (C-first, before
    // the static libs, mirroring ntdll-cargo's rtl_vista c_from_compile_db).
    if !cntpr_residue.is_empty() {
        // name = the cmake target that compiled these (libcntpr), so the engine
        // finds their CMakeFiles/libcntpr.dir entries in compile_commands.json.
        linker = linker.c_from_compile_db("libcntpr", &cntpr_residue);
    }
    if !cntpr_asm.is_empty() {
        linker = linker.asm_archive("cntpr_asm", cntpr_asm);
    }
    // librtl.a residual, compiled in-cargo (replaces the dropped librtl.a):
    //   - dbgprint_variadic.c (the C-variadic DbgPrint/DbgPrintEx/wine_dbg_*
    //     trampolines) via c_from_compile_db — name = the cmake `rtl` target so
    //     the engine finds CMakeFiles/rtl.dir in compile_commands.json.
    //   - the per-arch asm floor via asm_archive (lazy in-group, Kernel define
    //     set + kernel asm -I chain).
    linker = linker.c_from_compile_db("rtl", &["sdk/lib/rtl/dbgprint_variadic.c"]);
    if !rtl_asm.is_empty() {
        linker = linker.asm_archive("rtl_asm", rtl_asm);
    }
    // All arches: pseh-rust (replaces the dropped libpseh.a). Provides the 2
    // DUMMY_PSEH `_SEH2_Volatile*` data symbols everywhere; the i386 SEH runtime
    // (the 5 ntoskrnl.spec-exported entry points) self-gates inside the crate
    // via cfg(target_arch="x86"), so amd64/arm64 link only the volatiles. Its
    // panic_handler/eh_personality are absorbed by --allow-multiple-definition.
    linker = linker.rust_lib("pseh-rust", &[], true);
    // amd64 only: the SEH unwind path links _setjmp/_setjmpex/longjmp →
    // setjmp-rust (global_asm! save/restore, disasm-identical to the cmake asm,
    // + Rust longjmp). libsetjmp.a dropped. i386/arm64 don't reference setjmp.
    if arch == "x86_64" {
        linker = linker.rust_lib("setjmp-rust", &[], true);
    }
    // Self-host the hal.dll import lib from hal.spec (replaces the dropped
    // cmake-built libhal.a import lib) — the classpnp/win32k pattern. The shared
    // hal/hal.spec serves all 3 arches (halx86 + halarm64 both spec2def it).
    linker = linker.import_lib("hal/hal.spec", "hal.dll", "hal");
    // Self-host the kdcom.dll / bootvid.dll import libs from their specs
    // (replaces the dropped cmake-built libkdcom.a / libbootvid.a import libs).
    linker = linker.import_lib("drivers/base/kdcom/kdcom.spec", "kdcom.dll", "kdcom");
    linker = linker.import_lib("drivers/base/bootvid/bootvid.spec", "bootvid.dll", "bootvid");
    linker
        .static_libs(&static_libs)
        .link(ModuleKind::KernelExe {
            entry: entry.into(),
            image_base: 0x0040_0000,
        });
}
