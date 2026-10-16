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
            "i686-pc-windows-gnu", "ntdll_vista-i386.rc.res", 0x6fc7_0000, "_DllMain@12",
        ),
        "x86_64" => (
            "x86_64-pc-windows-gnu", "ntdll_vista-amd64.rc.res", 0x7fe_b200_0000, "DllMain",
        ),
        "aarch64" => (
            "aarch64-pc-windows-gnullvm", "ntdll_vista-arm64.rc.res", 0x7fe_b200_0000, "DllMain",
        ),
        o => panic!("ntdll_vista-cargo: unsupported arch {o}"),
    };

    Linker::new(target_json)
        .rust_staticlib("ntdll-rust")
        .rust_features(&["vista"]) // DllMain + LdrpHeap + ldrnotify
        // Tier-B sibling Rust crates (was reuse of cmake libsmlib_rust.a /
        // libutf8.a). smss-csrss-rust built smlib_only (--no-default-features).
        .rust_lib("smss-csrss-rust", &["smlib_only", "ntddi_ws03"], false)
        .rust_lib("utf8", &[], true)
        // rtl_vista is now 100% Rust: srw/condvar/runonce/locale come from the
        // rtl-vista-rust staticlib (a thin wrapper re-exporting rtl-sync's `vista`
        // modules — was the Tier-C reuse of librtl_vista.a, which compiled
        // condvar/runonce/srw/locale.c). condvar's Rtl*CriticalSection calls
        // resolve as ntdll.dll imports (critical.rs is NOT in `vista`).
        .rust_lib("rtl-vista-rust", &[], true)
        // Tier-A leaf C, cargo-compiled (was reuse of cmake libpseh.a). DUMMY_PSEH
        // dead code: header-free, two globals (_SEH2_Volatile0 / *ExceptionCode).
        .c_shims(&["sdk/lib/pseh/dummy.c"])
        .resource(&format!("data/{res}"))
        .spec2def("dll/ntdll/nt_0600/ntdll_vista.spec", "ntdll_vista.dll")
        // imports Rtl*/Nt*/… from ntdll.dll (add_importlibs(ntdll_vista ntdll))
        .import_lib("dll/ntdll/def/ntdll.spec", "ntdll.dll", "ntdll")
        // NOTE: librtl_vista.a dropped — condvar/runonce/srw/locale now come from
        // .rust_lib("rtl-sync", ["vista"]) above. libchkstk.a likewise dropped —
        // __chkstk/_alloca_probe/___alloca come from the clang_rt.builtins the
        // engine links (both were redundant reuse-.a entries).
        .link(ModuleKind::Win32Dll {
            entry: entry.into(),
            image_base: Some(image_base),
            subsystem: None, // lld default → GUI 6.0, matches cmake
        });
}
