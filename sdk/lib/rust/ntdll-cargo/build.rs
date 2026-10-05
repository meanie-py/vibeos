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
            // Moved up from 0x7c92_0000 to widen kernel32's preferred slot: the
            // kernel32-rust cohab grows kernel32.dll past its old 0x440000 slot
            // (kernel32@0x7c4e0000 .. ntdll@0x7c920000), and kernel32 overrunning
            // ntdll tripped STATUS_ILLEGAL_DLL_RELOCATION. ntdll is the only DLL
            // based above 0x7c4e0000 and 0x7c92..0x7ffe is free, so placing it high
            // gives kernel32 ~27MB of headroom for the rest of the campaign.
            // (amd64/arm64 ntdll sit at 0x7ffb_7000_0000 — no 32-bit crunch there.)
            0x7e00_0000,
            "___ImageBase",
        ),
        "x86_64" => (
            "x86_64-pc-windows-gnu",
            "ntdll-amd64.rc.res",
            0x7ff_b700_0000,
            "__ImageBase",
        ),
        "aarch64" => (
            "aarch64-pc-windows-gnullvm",
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
    // cnt-rust C floor — EMPTY as of the WS1 cmake-off batches (2026-06-21).
    // Every former member is now a self-contained/headerless c_shim in ndll_shims:
    //   - i386 _CIxxx math wrappers + amd64 SEH personality + _fltused (LEAF batch);
    //   - the C-variadic printf shells + _swprintf + _vscwprintf (printf batch,
    //     -DCRT_PRINTF_HEADERLESS);
    //   - scanf_trampoline.c (stdarg/stddef only — toolchain headers, no cmake).
    // So ntdll no longer calls c_from_compile_db — its cargo link is free of
    // cmake's compile_commands.json. (i386 integer/ftol/chkstk asm: cntpr_asm.)
    let cntpr_residue: Vec<&str> = Vec::new();
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
            // x87 pow/fmod (ntdll exports pow): this cell's cnt-rust-lib is the builtin
            // no-SSE i686 build, which leaves them to the asm (cnt-rust/src/cabi/math.rs).
            "sdk/lib/crt/math/i386/pow_asm.s",
            "sdk/lib/crt/math/i386/fmod_asm.s",
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
    // integer/chkstk asm, arm64 = shells + scanf only).
    // librtl.a DROPPED 2026-06-20 — the whole sdk/lib/rtl/*.c surface is Rust
    // (rtl-rust .rust_lib above); librtl.a was down to the irreducible floor
    // (C-variadic DbgPrint family + per-arch except/mem/swap/debug asm), now
    // compiled in-cargo via the rtl c_from_compile_db + rtl_asm asm_archive below
    // (the cntpr pattern). ntdll now links ZERO cmake .a.
    let static_libs: Vec<&str> = vec![];

    // librtl.a's irreducible asm floor (the `.c` surface is all rtl-rust now),
    // assembled in-cargo via asm_archive (lazy in-group, same asm_set as .asm()
    // = Kernel + the kernel asm -I chain). Per-arch except/mem/swap/debug asm.
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
    // NOTE: libntdllsys.a dropped — the Nt*/Zw* syscall stubs are now assembled
    // from the SOURCE ntoskrnl/ntdll.S (asm_archive below). libapisets.a / libuuid.a
    // dropped — now Rust (ntdll-rust apisets/uuid). libsetjmp.a dropped — amd64
    // builds setjmp/longjmp from pre-expanded asm + Rust glue; i386/arm64 never
    // referenced it. libchkstk.a dropped — __chkstk comes from clang_rt.builtins.

    // c_shims: the Tier-A header-free C floor compiled directly by cc-rs.
    //   - dummy.c (DUMMY_PSEH dead-code globals)
    //   - dbgprint_variadic.c (cmake-off spike, 2026-06-20)
    //   - WS1 cmake-off LEAF batch (2026-06-21): the CRT-floor _CIxxx math
    //     wrappers / SEH personality / _fltused, now self-contained/headerless
    //     and moved out of the cntpr_residue c_from_compile_db list (each
    //     verified symbol+disasm-identical vs the cmake object). i386 = ci*.c +
    //     fltused; amd64 = ehandler.c + fltused; arm64 = none (no _CIxxx/SEH/
    //     _fltused floor — see cntpr_residue).
    // sehtry.c: the kernel's __try/__except trampolines, built for user mode
    // (-DSEHTRY_USER_MODE) and exported as RtlSehTryExcept/RtlSehTryFinally, so
    // user-mode Rust can catch an RPC exception (ntdll_sys::seh).
    let mut ndll_shims: Vec<&str> = vec![
        "sdk/lib/pseh/dummy.c",
        "sdk/lib/rtl/dbgprint_variadic.c",
        "ntoskrnl/sehtry.c",
    ];
    match arch.as_str() {
        "x86" => ndll_shims.extend_from_slice(&[
            "sdk/lib/crt/math/i386/cicos.c",
            "sdk/lib/crt/math/i386/cisin.c",
            "sdk/lib/crt/math/i386/cisqrt.c",
            "sdk/lib/crt/math/i386/cilog.c",
            "sdk/lib/crt/math/i386/cipow.c",
            "sdk/lib/crt/misc/fltused.c",
        ]),
        "x86_64" => ndll_shims.extend_from_slice(&[
            "sdk/lib/crt/except/amd64/ehandler.c",
            "sdk/lib/crt/misc/fltused.c",
        ]),
        // arm64: user-mode SEH / fault-delivery self-test (FEX bring-up rung 0e,
        // docs/engineering/fex-bringup-plan.md). Native clang __try/__except +
        // a VEH-on-hardware-AV probe; needs -fms-extensions (added to
        // c_shim_flags below). Headerless so no pseh2.h macro layer interferes.
        // Called once from LdrpInit, gated to smss.exe.
        "aarch64" => ndll_shims.push("dll/ntdll/seh_poc_user.c"),
        _ => {}
    }
    // WS1 cmake-off printf+scanf batch (2026-06-21), all arches: the C-variadic
    // printf shells (#include the shared headerless _sxprintf.c engine behind
    // -DCRT_PRINTF_HEADERLESS; see c_shim_flags) + _vscwprintf (separate engine,
    // same guard) + the scanf trampoline (stdarg/stddef only). Each verified
    // disasm-identical vs the header build (WS1 harness). Moving them empties
    // cntpr_residue → ntdll drops c_from_compile_db entirely.
    ndll_shims.extend_from_slice(&[
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
    ]);

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
        // dbgprint_variadic.c: now self-contained/headerless (cmake-off spike
        // 2026-06-20) → compiled by cc-rs here, dropping the c_from_compile_db("rtl")
        // path that needed cmake's compile_commands.json.
        .c_shims(&ndll_shims)
        // -DCRT_PRINTF_HEADERLESS selects the hand-defined FILE/_TCHAR/flags in
        // _sxprintf.c / _vscwprintf.c (harmless to the other shims).
        // -fms-extensions: real clang __try/__except in seh_poc_user.c (arm64);
        // benign for the printf shells and dummy.c.
        .c_shim_flags(&["-DCRT_PRINTF_HEADERLESS", "-fms-extensions", "-DSEHTRY_USER_MODE"])
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
    // librtl.a residual, compiled in-cargo (replaces the dropped librtl.a):
    // dbgprint_variadic.c → moved to .c_shims() above (cmake-off spike 2026-06-20:
    // self-contained/headerless, no compile_commands.json needed). Only the
    // per-arch asm floor remains via asm_archive (lazy in-group).
    if !rtl_asm.is_empty() {
        linker = linker.asm_archive("rtl_asm", rtl_asm);
    }
    linker.static_libs(&static_libs).link(ModuleKind::Win32Dll {
        entry: entry.into(),
        image_base,
        subsystem: Some("console:5.01".into()),
    });
}
