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

    let asm_path = format!("sdk/lib/rust/ntdll-cargo/data/asm/{asm}");

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
        .asm(&[&asm_path], AsmDefineSet::Kernel)
        .resource(&format!("data/{res}"))
        .spec2def("dll/ntdll/def/ntdll.spec", "ntdll.dll")
        .spec_stubs() // 48 Etw* -stub bodies
        .static_libs(&[
            // Tier-C deep-C (deferred — follows rtl-priv / libcntpr-crt flips):
            "sdk/lib/rtl/librtl.a",
            "sdk/lib/rtl/librtl_vista.a",
            "sdk/lib/crt/liblibcntpr.a",
            // Tier-A asm/data/generated (still cmake-built):
            "ntoskrnl/libntdllsys.a",
            "sdk/lib/apisets/libapisets.a",
            "sdk/lib/uuid/libuuid.a",
            "sdk/lib/vcruntime/libsetjmp.a",
            "sdk/lib/crt/libchkstk.a",
        ])
        .link(ModuleKind::Win32Dll {
            entry: entry.into(),
            image_base,
            subsystem: Some("console:5.01".into()),
        });
}
