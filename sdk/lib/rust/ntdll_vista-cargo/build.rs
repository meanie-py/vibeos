//! Cargo-driven `ntdll_vista.dll` link (i386). Manifest for the shared
//! `reactos-link` engine — companion to ntdll-cargo.
//!
//! cmake still owns the sibling static libs; run `ninja` in build-i386
//! once first. `.res` is a copy of cmake's ntdll_vista.rc.res (regen tool
//! owed, mirroring gen_ntoskrnl_res.py).

use reactos_link::{Linker, ModuleKind};

fn main() {
    Linker::new("i686-kernel.json")
        // ntdll-rust with `vista` feature → DllMain + LdrpHeap + ldrnotify
        // (the nt_0600 module set). Built into this crate's own rust target
        // dir, distinct from ntdll-cargo's default-feature build.
        .rust_staticlib("ntdll-rust")
        .rust_features(&["vista"])
        .resource("data/ntdll_vista-i386.rc.res")
        .spec2def("dll/ntdll/nt_0600/ntdll_vista.spec", "ntdll_vista.dll")
        // ntdll_vista imports Rtl*/Nt*/… from ntdll.dll
        // (cmake `add_importlibs(ntdll_vista ntdll)`) — self-host the import
        // lib from ntdll.spec, no cmake-built libntdll.a needed.
        .import_lib("dll/ntdll/def/ntdll.spec", "ntdll.dll", "ntdll")
        // Reused cmake-built .a: the lean smlib archive (smlib_only) +
        // rtl_vista + utf8 + pseh + chkstk. Everything else is imported
        // from ntdll.dll above.
        .static_libs(&[
            "dll/ntdll/nt_0600/libsmlib_rust.a",
            "sdk/lib/rtl/librtl_vista.a",
            "sdk/lib/rtl/libutf8.a",
            "sdk/lib/pseh/libpseh.a",
            "sdk/lib/crt/libchkstk.a",
        ])
        // ntdll_vista.dll: real DllMain entry (RVA 0x1000), image base
        // 0x6fc70000 (dwarf baseaddress). No --subsystem → lld default
        // (GUI 6.0), matching cmake's ntdll_vista.dll.
        .link(ModuleKind::Win32Dll {
            entry: "_DllMain@12".into(),
            image_base: 0x6fc7_0000,
            subsystem: None,
        });
}
