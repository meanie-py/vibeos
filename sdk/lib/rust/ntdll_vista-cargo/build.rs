//! Cargo-driven `ntdll_vista.dll` link (i386 / amd64 / arm64). Manifest for
//! the shared `reactos-link` engine — companion to ntdll-cargo.
//!
//! cmake still owns the sibling static libs; run `ninja` in build-<arch>
//! once first. `.res` are regenerated out-of-band by `tools/gen_ntdll_res.py`
//! (windres on ntdll_vista.rc; canonical REPO llvm-windres, all 3 arches).

use std::env;
use reactos_link::{Linker, ModuleKind};

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    // ntdll_vista.dll: real DllMain entry; no --subsystem (lld default GUI
    // 6.0). i386 decorates the entry (_DllMain@12) + has a 32-bit base.
    let (target_json, res, image_base, entry): (&str, &str, u64, &str) = match arch.as_str() {
        "x86" => (
            "i686-kernel.json", "ntdll_vista-i386.rc.res", 0x6fc7_0000, "_DllMain@12",
        ),
        "x86_64" => (
            "x86_64-kernel.json", "ntdll_vista-amd64.rc.res", 0x7feb_2000_0000, "DllMain",
        ),
        "aarch64" => (
            "aarch64-kernel.json", "ntdll_vista-arm64.rc.res", 0x7feb_2000_0000, "DllMain",
        ),
        o => panic!("ntdll_vista-cargo: unsupported arch {o}"),
    };

    Linker::new(target_json)
        .rust_staticlib("ntdll-rust")
        .rust_features(&["vista"]) // DllMain + LdrpHeap + ldrnotify
        .resource(&format!("data/{res}"))
        .spec2def("dll/ntdll/nt_0600/ntdll_vista.spec", "ntdll_vista.dll")
        // imports Rtl*/Nt*/… from ntdll.dll (add_importlibs(ntdll_vista ntdll))
        .import_lib("dll/ntdll/def/ntdll.spec", "ntdll.dll", "ntdll")
        .static_libs(&[
            "dll/ntdll/nt_0600/libsmlib_rust.a",
            "sdk/lib/rtl/librtl_vista.a",
            "sdk/lib/rtl/libutf8.a",
            "sdk/lib/pseh/libpseh.a",
            "sdk/lib/crt/libchkstk.a",
        ])
        .link(ModuleKind::Win32Dll {
            entry: entry.into(),
            image_base,
            subsystem: None, // lld default → GUI 6.0, matches cmake
        });
}
