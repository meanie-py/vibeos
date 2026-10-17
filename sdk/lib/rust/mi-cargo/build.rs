//! Cargo-driven `mi.dll` link. The body is mi-rust; reactos-link builds the PE.

use reactos_link::{Linker, ModuleKind, MsvcrtexKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base) = match arch.as_str() {
        "x86" => ("i686-pc-windows-gnu", "_DllMainCRTStartup@12", None),
        "x86_64" => ("x86_64-pc-windows-gnu", "DllMainCRTStartup", None),
        "aarch64" => ("aarch64-pc-windows-gnullvm", "DllMainCRTStartup", None),
        o => panic!("mi-cargo: unsupported arch {o}"),
    };

    let link_order: &[&str] =
        &["body", "import:ole32", "import:oleaut32", "import:msvcrt", "import:kernel32", "import:ntdll", "engine:oldnames"];

    Linker::new(target_json)
        .rust_staticlib("mi-rust")
        .spec2def("dll/win32/mi/mi.spec", "mi.dll")
        .spec_stubs()
        .import_lib("dll/win32/ole32/ole32.spec", "ole32.dll", "ole32")
        .import_lib("dll/win32/oleaut32/oleaut32.spec", "oleaut32.dll", "oleaut32")
        .import_lib("dll/win32/msvcrt/msvcrt.spec", "msvcrt.dll", "msvcrt")
        .import_lib("dll/win32/kernel32/kernel32.spec", "kernel32.dll", "kernel32")
        .import_lib("dll/ntdll/def/ntdll.spec", "ntdll.dll", "ntdll")
        .link_order(link_order)
        .msvcrtex_crt(MsvcrtexKind::Dll)
        .link(ModuleKind::Win32Dll {
            entry: entry.into(),
            image_base,
            subsystem: None,
        });
}
