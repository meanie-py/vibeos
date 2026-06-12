//! Cargo-driven `ntdll.dll` link (i386 / amd64 / arm64). Manifest for the
//! shared `reactos-link` engine.
//!
//! Phase-2 of the cargo-drive: cargo now also BUILDS the sibling Rust crates
//! (Tier-B: rtl-rust / utf8 / cnt-rust-lib / smss-csrss-rust csrlib) directly
//! via `rust_lib()`, and ASSEMBLES the asm from SOURCE `.S` (dispatch.S +
//! ntoskrnl/ntdll.S syscall stubs + amd64 setjmp) via the engine's kernel asm
//! -I chain — no offline pre-expansion. cmake still owns only the deep-C bodies
//! (Tier-C: librtl / librtl_vista / libcntpr — deferred to the rtl-priv +
//! libcntpr-crt flip tracks). Run `ninja` in build-<arch> once first so the
//! reused Tier-C `.a` + the generated asm includes (ks*.inc / sysfuncs.h) exist.
//! Out-of-band input still checked in:
//!   - `tools/gen_ntdll_res.py`  → data/ntdll-<arch>.rc.res

use reactos_link::{AsmDefineSet, Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    // Per-arch: (target json, res basename, image base, entry).
    // ntdll.dll is console:5.01 on all arches; AddressOfEntryPoint 0 via the
    // lld __ImageBase symbol (i386 decorates it ___ImageBase).
    let (target_json, res, image_base, entry): (&str, &str, u64, &str) = match arch.as_str() {
        "x86" => (
            "i686-kernel.json",
            "ntdll-i386.rc.res",
            0x7c92_0000,
            "___ImageBase",
        ),
        "x86_64" => (
            "x86_64-kernel.json",
            "ntdll-amd64.rc.res",
            0x7ff_b700_0000,
            "__ImageBase",
        ),
        "aarch64" => (
            "aarch64-kernel.json",
            "ntdll-arm64.rc.res",
            0x7ff_b700_0000,
            "__ImageBase",
        ),
        o => panic!("ntdll-cargo: unsupported arch {o}"),
    };
    // cnt-rust-lib `float` feature, mirroring libcntpr.cmake EXACTLY: amd64 +
    // arm64 build the Rust format_float/format_floatw that streamout/wstreamout
    // call (without it the arm64 link fails `undefined symbol: format_float`).
    // i386 (x86) omits float — its no-SSE softfloat build takes the C residual
    // format_float.c/format_floatw.c from liblibcntpr.a instead.
    let cnt_feats: &[&str] = if arch == "x86" { &[] } else { &["float"] };

    let stem = match arch.as_str() {
        "x86" => "i386",
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        o => panic!("ntdll-cargo: unsupported arch {o}"),
    };
    // Self-hosted asm: the engine assembles the SOURCE `.S` directly (kernel asm
    // -I chain: asm.inc + the generated ks*.inc + ntoskrnl/include), so there is
    // no offline pre-expansion / checked-in `.s` (the old tools/gen_ntdll_asm.py).
    // dispatch.S = ntdll's kernel→user trampolines (force-included object, like
    // cmake compiles it into the ntdll target).
    let mut asm_paths = vec![format!("dll/ntdll/dispatch/{stem}/dispatch.S")];
    // ntdllsys = ntoskrnl/ntdll.S (Nt*/Zw* syscall stubs; #includes syscalls.inc
    // + the generated sysfuncs.h). In-group ARCHIVE (see .asm_archive below) —
    // force-including these stubs as an object relocated ~5% of the i386 image
    // and tripped a latent layout-sensitive hivewrt miscompile; the archive
    // preserves the cmake link layout.
    let ntdllsys_path = "ntoskrnl/ntdll.S".to_string();
    // amd64 ntdll.dll references setjmp/_setjmpex/longjmp (i386/arm64 don't —
    // libsetjmp.a is dead there). Assemble the register save/restore asm here
    // (longjmp's RtlUnwind glue is Rust: ntdll-rust src/setjmp.rs), replacing
    // the cmake libsetjmp.a reuse.
    if arch == "x86_64" {
        asm_paths.push("sdk/lib/vcruntime/amd64/_setjmp.s".to_string());
        asm_paths.push("sdk/lib/vcruntime/amd64/__longjmp_noframe.s".to_string());
        // __chkstk: the MSVC-name stack probe (clang emits a call for frames
        // >1 page). clang_rt.builtins-x86_64 only ships __chkstk_ms, NOT
        // __chkstk, so dropping liblibcntpr.a (its old provider via this same
        // chkstk_ms.s) leaves __chkstk undefined — assemble it here. (i386 stays
        // on liblibcntpr.a; arm64's __chkstk comes from clang_rt.builtins-aarch64.)
        asm_paths.push("sdk/lib/crt/except/amd64/chkstk_ms.s".to_string());
    }
    let asm_refs: Vec<&str> = asm_paths.iter().map(String::as_str).collect();

    // cnt-rust integration (amd64): the irreducible C floor cnt-rust-lib can't
    // supply, compiled in-cargo from the libcntpr cmake target's compile_commands
    // entries (symbol-identical to liblibcntpr.a's objects). cnt-rust-lib already
    // provides the whole translatable CRT surface (string/mem/ctype/search + the
    // printf/scanf format ENGINES: streamout/wstreamout/cnt_v*scanf). The floor:
    //   - the amd64 SEH personality (ehandler.c → __C_specific_handler /
    //     _local_unwind),
    //   - _fltused (the float-used marker the MSVC ABI expects),
    //   - the thin variadic printf shells (Rust stable can't DEFINE C variadics):
    //     sprintf/swprintf/vsprintf + the _sn*/_vsn*/_vscwprintf family — each a
    //     `va_start; streamout(...); va_end` shell over the Rust engine,
    //   - the scanf variadic trampolines (sscanf/swscanf/… → cnt_v*scanf),
    //     cnt-rust's own cabi/scanf_trampoline.c (REMOVE'd from libcntpr.cmake and
    //     re-added here for the cargo drive).
    // __chkstk is NOT listed: it comes from the clang_rt.builtins the engine links
    // (the canonical clang stack probe), same as ntoskrnl-cargo amd64. i386/arm64
    // keep liblibcntpr.a — their residue (i386: _CIxxx + integer asm → asm_archive)
    // is a follow-up. See ntoskrnl-cargo build.rs +
    // docs/engineering/cargo-drive-unification-plan.md.
    let cntpr_residue: Vec<&str> = match arch.as_str() {
        "x86_64" => vec![
            "sdk/lib/crt/except/amd64/ehandler.c",
            "sdk/lib/crt/misc/fltused.c",
            "sdk/lib/crt/printf/sprintf.c",
            "sdk/lib/crt/printf/swprintf.c",
            "sdk/lib/crt/printf/_swprintf.c",
            "sdk/lib/crt/printf/vsprintf.c",
            "sdk/lib/crt/printf/_snprintf.c",
            "sdk/lib/crt/printf/_snwprintf.c",
            "sdk/lib/crt/printf/_vsnprintf.c",
            "sdk/lib/crt/printf/_vsnwprintf.c",
            "sdk/lib/crt/printf/_vscwprintf.c",
            "sdk/lib/rust/cnt-rust/src/cabi/scanf_trampoline.c",
        ],
        // i386: the _CIxxx x87 MSVC math intrinsic wrappers (ci*.c CALL the
        // now-Rust libm) + _fltused + the variadic printf shells. The 64-bit
        // integer / ftol / chkstk asm go through cntpr_asm below. The C floor is
        // shared with ntoskrnl-cargo (lazy archive pull) plus ntdll's own spec
        // exports (_vscwprintf + scanf trampolines). No SEH personality on i386
        // (DUMMY_PSEH).
        "x86" => vec![
            "sdk/lib/crt/math/i386/cicos.c",
            "sdk/lib/crt/math/i386/cisin.c",
            "sdk/lib/crt/math/i386/cisqrt.c",
            "sdk/lib/crt/math/i386/cilog.c",
            "sdk/lib/crt/math/i386/cipow.c",
            "sdk/lib/crt/misc/fltused.c",
            "sdk/lib/crt/printf/sprintf.c",
            "sdk/lib/crt/printf/swprintf.c",
            "sdk/lib/crt/printf/_swprintf.c",
            "sdk/lib/crt/printf/vsprintf.c",
            "sdk/lib/crt/printf/_snprintf.c",
            "sdk/lib/crt/printf/_snwprintf.c",
            "sdk/lib/crt/printf/_vsnprintf.c",
            "sdk/lib/crt/printf/_vsnwprintf.c",
            // ntdll spec exports (pulled by the export directive, not the body):
            // _vscwprintf (wide char-count printf) + the scanf trampolines
            // (sscanf/swscanf → cnt_vsscanf, which cnt-rust provides on i386 too —
            // the baseline i386 ntdll links these from libcntpr.cmake's copy).
            "sdk/lib/crt/printf/_vscwprintf.c",
            "sdk/lib/rust/cnt-rust/src/cabi/scanf_trampoline.c",
        ],
        // arm64: just the variadic printf shells + _vscwprintf + the scanf
        // trampolines (drop-test). No SEH personality / _fltused / chkstk / _CIxxx
        // / integer asm — arm64's cdecl variadic ABI + clang_rt.builtins-aarch64
        // (__chkstk) + cnt-rust's libm cover those.
        "aarch64" => vec![
            "sdk/lib/crt/printf/sprintf.c",
            "sdk/lib/crt/printf/swprintf.c",
            "sdk/lib/crt/printf/_swprintf.c",
            "sdk/lib/crt/printf/vsprintf.c",
            "sdk/lib/crt/printf/_snprintf.c",
            "sdk/lib/crt/printf/_snwprintf.c",
            "sdk/lib/crt/printf/_vsnprintf.c",
            "sdk/lib/crt/printf/_vsnwprintf.c",
            "sdk/lib/crt/printf/_vscwprintf.c",
            "sdk/lib/rust/cnt-rust/src/cabi/scanf_trampoline.c",
        ],
        _ => Vec::new(),
    };
    // i386 asm floor (lazy archive, layout-safe): 64-bit integer intrinsics
    // (_alldiv/…/_aullshr), float→int (_ftol), stack probe (_chkstk /
    // _alloca_probe / ___alloca). amd64's __chkstk is the chkstk_ms.s in
    // asm_paths above; arm64 gets __chkstk from clang_rt.builtins-aarch64.
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

    // Tier-C deep-C (deferred — follows rtl-priv / libcntpr-crt flips). librtl is
    // always reused; liblibcntpr.a is reused only where its residue isn't wired in
    // yet (i386/arm64). librtl_vista.a is gone (compiled in-cargo below).
    // liblibcntpr.a is DROPPED on every arch (cntpr_residue + cntpr_asm cover the
    // floor: amd64 = SEH + printf shells + scanf, i386 = _CIxxx + shells + scanf +
    // integer/chkstk asm, arm64 = shells + scanf only). librtl.a stays (Tier-C).
    let static_libs: Vec<&str> = vec!["sdk/lib/rtl/librtl.a"];
    // NOTE: libntdllsys.a dropped — the Nt*/Zw* syscall stubs are now assembled
    // from the SOURCE ntoskrnl/ntdll.S (asm_archive below). libapisets.a / libuuid.a
    // dropped — now Rust (ntdll-rust apisets/uuid). libsetjmp.a dropped — amd64
    // builds setjmp/longjmp from pre-expanded asm + Rust glue; i386/arm64 never
    // referenced it. libchkstk.a dropped — __chkstk comes from clang_rt.builtins.

    let mut linker = Linker::new(target_json);
    linker = linker
        .rust_staticlib("ntdll-rust") // default features = ntdll.dll proper
        // Tier-B sibling Rust crates (was reuse of cmake librtl_rust.a /
        // libutf8.a / libcnt_rust_lib.a / libcsrlib_rust.a). smss-csrss-rust
        // is built csrlib_only (--no-default-features) like cmake's cohab.
        .rust_lib("smss-csrss-rust", &["csrlib_only", "ntddi_ws03"], false)
        .rust_lib("rtl-rust", &[], true)
        .rust_lib("utf8", &[], true)
        .rust_lib("cnt-rust-lib", cnt_feats, true)
        // Tier-A leaf C, cargo-compiled (was reuse of cmake libpseh.a). DUMMY_PSEH
        // dead code: header-free, two globals (_SEH2_Volatile0 / *ExceptionCode).
        .c_shims(&["sdk/lib/pseh/dummy.c"])
        .asm(&asm_refs, AsmDefineSet::Kernel)
        // ntdllsys (Nt*/Zw* syscall stubs) as an in-group archive, replacing
        // the cmake libntdllsys.a — preserves the cmake archive link layout.
        .asm_archive("ntdllsys", &[&ntdllsys_path])
        .resource(&format!("data/{res}"))
        .spec2def("dll/ntdll/def/ntdll.spec", "ntdll.dll")
        .spec_stubs() // 48 Etw* -stub bodies
        // Tier-C deep-C compiled IN-cargo via compile_commands.json replay
        // (flag-exact; drops the `ninja rtl_vista` prerequisite). Needs cmake
        // configured with -DCMAKE_EXPORT_COMPILE_COMMANDS=ON.
        // rtl_vista is now 100% Rust (condvar/srw/runonce/locale → rtl-sync, via
        // .rust_lib("rtl-rust")); no rtl_vista C files remain to compile in-cargo,
        // so the chain ends here. (The deepen-branch's rtl_vista c_from_compile_db
        // is intentionally NOT carried over — superseded by rtl-rust on this branch.)
        ;
    // cnt-rust C floor — name = the libcntpr cmake target so the engine finds
    // these files' CMakeFiles/libcntpr.dir compile_commands entries. Per-arch:
    // amd64 = SEH + _fltused + printf shells + scanf, i386 = _CIxxx + shells +
    // scanf (+ integer/chkstk via cntpr_asm), arm64 = shells + scanf only.
    if !cntpr_residue.is_empty() {
        linker = linker.c_from_compile_db("libcntpr", &cntpr_residue);
    }
    if !cntpr_asm.is_empty() {
        linker = linker.asm_archive("cntpr_asm", cntpr_asm);
    }
    linker.static_libs(&static_libs).link(ModuleKind::Win32Dll {
        entry: entry.into(),
        image_base,
        subsystem: Some("console:5.01".into()),
    });
}
