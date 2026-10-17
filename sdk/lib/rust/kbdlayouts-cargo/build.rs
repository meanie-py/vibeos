//! Cargo-driven link of every keyboard layout DLL (dll/keyboard), one bin each.
//! Each link is cmake's: the layout's .spec exports `<layout>_KbdLayerDescriptor` from
//! the shared body under its real name, its .rc is compiled with cmake's windres rule (Linker::rc), and
//! the sections merge into one executable `.data` (a layout has no code section).

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry) = match arch.as_str() {
        "x86" => ("i686-pc-windows-gnu", "___ImageBase"),
        "x86_64" => ("x86_64-pc-windows-gnu", "__ImageBase"),
        "aarch64" => ("aarch64-pc-windows-gnullvm", "__ImageBase"),
        o => panic!("kbdlayouts-cargo: unsupported arch {o}"),
    };
    let manifest = std::fs::read_to_string("Cargo.toml").expect("read Cargo.toml");
    // Every `name = "…"` but the package's own (a bin name has no `-`).
    let layouts: Vec<&str> = manifest
        .lines()
        .filter_map(|l| l.strip_prefix("name = \"")?.strip_suffix('"'))
        .filter(|n| !n.contains('-'))
        .collect();
    assert!(!layouts.is_empty(), "kbdlayouts-cargo: no [[bin]] layouts in Cargo.toml");
    // The layouts' part of cmake's windres command, the same for every layout (they were
    // one OBJECT library in dll/keyboard); reactos-link adds what every module shares
    // from reactos-link/cflags/rc.<arch>.flags (Linker::rc).
    let rc_includes: &[&str] = &[
        "@build/dll/keyboard",
        "dll/keyboard",
        "@std",
        "sdk/lib/pseh/include",
        "vendor/opengl-headers/include",
        "vendor/libdxg/include",
    ];
    let rc_flags: &[&str] = &[];
    let rc_defines: &[&str] = &["WINVER=0x502", "_WIN32_WINNT=0x502"];
    for layout in layouts {
        Linker::new(target_json)
            .for_bin(layout)
            .rust_staticlib("kbdlayouts-rust")
            .rc(
                &format!("dll/keyboard/{layout}/{layout}.rc"),
                rc_includes,
                rc_flags,
                rc_defines,
            )
            .spec2def(&format!("dll/keyboard/{layout}/{layout}.spec"), &format!("{layout}.dll"))
            // Each directive is its own -Xlinker pair: `-Wl,` would split /SECTION's comma.
            // .pdata is the one addition to cmake's list: the bin's panic handler is a
            // rooted lang item, and on amd64/arm64 its unwind entry would otherwise be
            // a section of its own in a module that is meant to be one .data.
            .link_args(&[
                "-Xlinker", "/MERGE:.pdata=.data",
                "-Xlinker", "/MERGE:.text=.data",
                "-Xlinker", "/MERGE:.rdata=.data",
                "-Xlinker", "/MERGE:.bss=.data",
                "-Xlinker", "/MERGE:.edata=.data",
                "-Xlinker", "/SECTION:.data,ER",
            ])
            .link(ModuleKind::Win32Dll {
                entry: entry.into(),
                image_base: Some(0x5fff_0000),
                subsystem: Some("native:5.01".into()),
            });
    }
}
