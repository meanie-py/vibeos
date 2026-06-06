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
    }
    let asm_refs: Vec<&str> = asm_paths.iter().map(String::as_str).collect();

    Linker::new(target_json)
        .rust_staticlib("ntdll-rust") // default features = ntdll.dll proper
        // Tier-B sibling Rust crates (was reuse of cmake librtl_rust.a /
        // libutf8.a / libcnt_rust_lib.a / libcsrlib_rust.a). smss-csrss-rust
        // is built csrlib_only (--no-default-features) like cmake's cohab.
        .rust_lib("smss-csrss-rust", &["csrlib_only", "ntddi_ws03"], false)
        .rust_lib("rtl-rust", &["user-atom-table"], true)
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
        // (flag-exact; drops the `ninja rtl_vista` prerequisite). First mover of
        // the residue migration — librtl / libcntpr follow. Needs cmake
        // configured with -DCMAKE_EXPORT_COMPILE_COMMANDS=ON.
        .c_from_compile_db(
            "rtl_vista",
            &[
                "sdk/lib/rtl/condvar.c",
                "sdk/lib/rtl/locale.c",
                "sdk/lib/rtl/runonce.c",
                "sdk/lib/rtl/srw.c",
            ],
        )
        .static_libs(&[
            // Tier-C deep-C (deferred — follows rtl-priv / libcntpr-crt flips):
            "sdk/lib/rtl/librtl.a",
            // librtl_vista.a — now compiled in-cargo (.c_from_compile_db above)
            "sdk/lib/crt/liblibcntpr.a",
            // NOTE: libntdllsys.a dropped — the Nt*/Zw* syscall stubs are now
            // assembled from the SOURCE ntoskrnl/ntdll.S (asm_archive above; the
            // engine resolves syscalls.inc + the generated sysfuncs.h directly).
            // NOTE: libapisets.a dropped — ApiSetResolveToHost + the api-set
            // table are now Rust (ntdll-rust src/apisets.rs + apisets_table.rs
            // generated at build time by ntdll-rust/build.rs from apisets.table.c).
            // NOTE: libuuid.a dropped — ntdll references only 3 fixed COM IIDs
            // (IID_IUnknown/IStream/ISequentialStream), now provided by Rust
            // (ntdll-rust src/uuid.rs); the widl-generated archive is overkill.
            // NOTE: libsetjmp.a dropped — amd64 now builds setjmp/longjmp from
            // pre-expanded asm + Rust glue (above); i386/arm64 never referenced
            // it (dead archive members).
            // NOTE: libchkstk.a dropped — __chkstk/_alloca_probe/___alloca are
            // already provided by the clang_rt.builtins the engine links (the
            // canonical stack-probe for clang-emitted code), so the cmake
            // ReactOS chkstk_asm.s reuse-.a is redundant here.
        ])
        .link(ModuleKind::Win32Dll {
            entry: entry.into(),
            image_base,
            subsystem: Some("console:5.01".into()),
        });
}
