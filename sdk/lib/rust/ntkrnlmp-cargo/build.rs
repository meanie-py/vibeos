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
    //   - hal_dir: hal/halx86 (i386+amd64) vs hal/halarm64 (arm64).
    //   - chkstk: cmake links the ReactOS libchkstk.a on i386+arm64 (amd64 takes
    //     __chkstk from the clang_rt.builtins the engine already links).
    let (rust_target, res, entry, asm, cnt_float, hal_dir, chkstk): (
        &str,
        &str,
        &str,
        Vec<&str>,
        bool,
        &str,
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
            "halx86",
            true,
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
            "halx86",
            false,
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
            "halarm64",
            true,
        ),
        o => panic!("ntkrnlmp-cargo: unsupported arch {o}"),
    };

    let cnt_feats: &[&str] = if cnt_float { &["float"] } else { &[] };
    let hal_lib = format!("hal/{hal_dir}/CMakeFiles/libhal.dir/libhal.a");

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
        "sdk/lib/cportlib/libcportlib.a",
        "sdk/lib/drivers/csq/libcsq.a",
        "sdk/lib/pseh/libpseh.a",
        "sdk/lib/drivers/arbiter/libarbiter.a",
        "sdk/lib/cmlib/libcmlib.a",
        "sdk/lib/lsalib/libntlsalib.a",
        "sdk/lib/rtl/librtl.a",
        "sdk/lib/rossym/librossym.a",
        // liblibcntpr.a (the cmake C CRT) is GONE — cnt-rust-lib provides the
        // whole translatable surface (string/mem/ctype/search/printf+scanf
        // engines/full math); the irreducible C/asm floor is compiled in-cargo
        // (cntpr_residue / cntpr_asm below). See cargo-drive-unification-plan.md.
        "sdk/lib/vcruntime/libsetjmp.a",
        "sdk/lib/wdmguid/libwdmguid.a",
        "sdk/lib/poguid/libpoguid.a",
        "sdk/lib/ioevent/libioevent.a",
        &hal_lib,
        "drivers/base/kdrosdbg/CMakeFiles/libkdcom.dir/libkdcom.a",
        "drivers/base/bootvid/CMakeFiles/libbootvid.dir/libbootvid.a",
    ];
    if chkstk {
        static_libs.push("sdk/lib/crt/libchkstk.a");
    }

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
        // Header-free C shims (built above; kdbg shim is x86-only).
        .c_shims(&c_shims)
        // Self-hosted asm: the engine assembles the SOURCE `.S` directly via its
        // kernel asm -I chain (asm.inc + the generated build-<arch> ks*.inc +
        // ntoskrnl/include) — no offline pre-expansion. The set is per-arch
        // (see the match above).
        .asm(&asm, AsmDefineSet::Kernel)
        // ntkrnlmp = the SMP kernel: assemble the kernel .S with CONFIG_SMP
        // (the #ifdef CONFIG_SMP spinlock/fast-path blocks in ks*.inc), mirroring
        // cmake's target_compile_definitions(ntkrnlmp PRIVATE CONFIG_SMP). The
        // Rust body (kernel-rust) is identical UP/MP — only the asm differs.
        .asm_extra_defines(&[("CONFIG_SMP", None)])
        .resource(&format!("data/{res}"))
        .spec2def("ntoskrnl/ntoskrnl.spec", "ntkrnlmp.exe");
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
    linker
        .static_libs(&static_libs)
        .link(ModuleKind::KernelExe {
            entry: entry.into(),
            image_base: 0x0040_0000,
        });
}
