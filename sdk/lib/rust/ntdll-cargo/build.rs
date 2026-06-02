//! Cargo-driven `ntdll.dll` link (i386 / amd64 / arm64). Manifest for the
//! shared `reactos-link` engine.
//!
//! cmake still owns the sibling static libs; run `ninja` in build-<arch>
//! once first. Out-of-band inputs (regenerate when their sources change):
//!   - `tools/gen_ntdll_asm.py`  → data/asm/dispatch-<arch>.s
//!   - data/ntdll-<arch>.rc.res  (today copies of cmake's ntdll.rc.res;
//!     a gen_ntdll_res.py is owed)

use std::env;
use reactos_link::{AsmDefineSet, Linker, ModuleKind};

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    // Per-arch: (target json, asm basename, res basename, cnt-rust triple,
    //            image base, entry symbol). ntdll.dll is console:5.01 on all
    //            arches; AddressOfEntryPoint 0 via the lld __ImageBase symbol
    //            (i386 decorates it ___ImageBase).
    let (target_json, asm, res, cnt_triple, image_base, entry): (
        &str, &str, &str, &str, u64, &str,
    ) = match arch.as_str() {
        "x86" => (
            "i686-kernel.json", "dispatch-i386.s", "ntdll-i386.rc.res",
            "i686-pc-windows-gnu", 0x7c92_0000, "___ImageBase",
        ),
        "x86_64" => (
            "x86_64-kernel.json", "dispatch-amd64.s", "ntdll-amd64.rc.res",
            "x86_64-pc-windows-gnu", 0x7ffb_7000_0000, "__ImageBase",
        ),
        "aarch64" => (
            "aarch64-kernel.json", "dispatch-arm64.s", "ntdll-arm64.rc.res",
            "aarch64-pc-windows-gnullvm", 0x7ffb_7000_0000, "__ImageBase",
        ),
        o => panic!("ntdll-cargo: unsupported arch {o}"),
    };

    let asm_path = format!("sdk/lib/rust/ntdll-cargo/data/asm/{asm}");
    let cnt_rust = format!("sdk/lib/crt/cnt-rust-target/{cnt_triple}/release/libcnt_rust_lib.a");

    Linker::new(target_json)
        .rust_staticlib("ntdll-rust") // default features = ntdll.dll proper
        .asm(&[&asm_path], AsmDefineSet::Kernel)
        .resource(&format!("data/{res}"))
        .spec2def("dll/ntdll/def/ntdll.spec", "ntdll.dll")
        .spec_stubs() // 48 Etw* -stub bodies
        .static_libs(&[
            "dll/ntdll/libcsrlib_rust.a",
            "sdk/lib/rtl/librtl.a",
            "sdk/lib/rtl/librtl_vista.a",
            "sdk/lib/rtl/librtl_rust.a",
            "sdk/lib/rtl/libutf8.a",
            "sdk/lib/crt/liblibcntpr.a",
            "sdk/lib/crt/libcntpr_rust.a",
            &cnt_rust,
            "ntoskrnl/libntdllsys.a",
            "sdk/lib/apisets/libapisets.a",
            "sdk/lib/uuid/libuuid.a",
            "sdk/lib/vcruntime/libsetjmp.a",
            "sdk/lib/pseh/libpseh.a",
            "sdk/lib/crt/libchkstk.a",
        ])
        .link(ModuleKind::Win32Dll {
            entry: entry.into(),
            image_base,
            subsystem: Some("console:5.01".into()),
        });
}
