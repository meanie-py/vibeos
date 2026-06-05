//! Cargo-driven `ntdll.dll` link (i386 / amd64 / arm64). Manifest for the
//! shared `reactos-link` engine.
//!
//! Phase-2 of the cargo-drive: cargo now also BUILDS the sibling Rust crates
//! (Tier-B: rtl-rust / utf8 / cnt-rust-lib / smss-csrss-rust csrlib) directly
//! via `rust_lib()`, instead of reusing their cmake-built `.a`. cmake still
//! owns only the deep-C bodies (Tier-C: librtl / librtl_vista / libcntpr —
//! deferred to the rtl-priv + libcntpr-crt flip tracks) and the asm/data libs
//! (Tier-A: ntdllsys / apisets / uuid / setjmp / pseh / chkstk). Run `ninja`
//! in build-<arch> once first for those. Out-of-band inputs:
//!   - `tools/gen_ntdll_asm.py`  → data/asm/dispatch-<arch>.s
//!   - `tools/gen_ntdll_res.py`  → data/ntdll-<arch>.rc.res

use std::env;
use reactos_link::{AsmDefineSet, Linker, ModuleKind};

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    // Per-arch: (target json, asm basename, res basename, image base, entry).
    // ntdll.dll is console:5.01 on all arches; AddressOfEntryPoint 0 via the
    // lld __ImageBase symbol (i386 decorates it ___ImageBase).
    let (target_json, asm, res, image_base, entry): (&str, &str, &str, u64, &str) =
        match arch.as_str() {
            "x86" => (
                "i686-kernel.json", "dispatch-i386.s", "ntdll-i386.rc.res",
                0x7c92_0000, "___ImageBase",
            ),
            "x86_64" => (
                "x86_64-kernel.json", "dispatch-amd64.s", "ntdll-amd64.rc.res",
                0x7ff_b700_0000, "__ImageBase",
            ),
            "aarch64" => (
                "aarch64-kernel.json", "dispatch-arm64.s", "ntdll-arm64.rc.res",
                0x7ff_b700_0000, "__ImageBase",
            ),
            o => panic!("ntdll-cargo: unsupported arch {o}"),
        };
    // cnt-rust-lib gets the `float` feature on x86 (by-value f64 ABI); arm64
    // omits it (mirrors libcntpr.cmake).
    let cnt_feats: &[&str] = if arch == "aarch64" { &[] } else { &["float"] };

    let stem = match arch.as_str() {
        "x86" => "i386",
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        o => panic!("ntdll-cargo: unsupported arch {o}"),
    };
    let asm_dir = "sdk/lib/rust/ntdll-cargo/data/asm";
    // dispatch (ntdll's own kernel→user trampolines) — force-included object,
    // exactly as cmake compiles dispatch.S into the ntdll target. ntdllsys (the
    // Nt*/Zw* syscall stubs) is handled separately as an in-group ARCHIVE (see
    // .asm_archive below) — force-including it as an object relocated ~5% of the
    // i386 image and tripped a latent layout-sensitive hivewrt miscompile.
    let mut asm_paths = vec![format!("{asm_dir}/{asm}")];
    let ntdllsys_path = format!("{asm_dir}/ntdllsys-{stem}.s");
    // amd64 ntdll.dll references setjmp/_setjmpex/longjmp (i386/arm64 don't —
    // libsetjmp.a is dead there). Provide the register save/restore asm here
    // (longjmp's RtlUnwind glue is Rust: ntdll-rust src/setjmp.rs), replacing
    // the cmake libsetjmp.a reuse. Pre-expanded by tools/gen_ntdll_asm.py.
    if arch == "x86_64" {
        asm_paths.push(format!("{asm_dir}/setjmp-amd64.s"));
        asm_paths.push(format!("{asm_dir}/longjmp_noframe-amd64.s"));
    }
    let asm_refs: Vec<&str> = asm_paths.iter().map(String::as_str).collect();

    Linker::new(target_json)
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
        .static_libs(&[
            // Tier-C deep-C (deferred — follows rtl-priv / libcntpr-crt flips):
            "sdk/lib/rtl/librtl.a",
            "sdk/lib/rtl/librtl_vista.a",
            "sdk/lib/crt/liblibcntpr.a",
            // Tier-A asm/data/generated (still cmake-built):
            // NOTE: libntdllsys.a dropped — the Nt*/Zw* syscall stubs are now
            // pre-expanded from ntoskrnl/ntdll.S (asm above), reusing the same
            // checked-in syscalls.inc + sysfuncs.h (numbers identical).
            // NOTE: libapisets.a dropped — ApiSetResolveToHost + the api-set
            // table are now Rust (ntdll-rust src/apisets.rs + the generated
            // apisets_table.rs from tools/gen_apisets_rs.py).
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
