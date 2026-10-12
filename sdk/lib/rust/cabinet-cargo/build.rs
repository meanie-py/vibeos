//! Cargo-driven `cabinet.dll` link (i386 / amd64 / arm64). The Wine-derived
//! CAB archive DLL (dll/win32/cabinet), fully translated to `cabinet-rust`
//! (FCI/FDI + the LZX/QTM/MSZIP decompression engine). Replaces the cmake
//! `add_library(cabinet MODULE rust_cohab_stub.c cabinet.rc cabinet.def)` +
//! `set_module_type(cabinet win32dll)` + `target_link_libraries(cabinet wine
//! zlib)` + `add_importlibs(cabinet msvcrt kernel32 ntdll)` cohab. Version
//! resource (cabinet.rc) dropped, as in the other DLL cargo cuts.
//!
//! Win32Dll (user-mode): entry _DllMainCRTStartup@12 (i386) / DllMainCRTStartup
//! (from libmsvcrtex.a); image base 0x79160000 (i386) / 0x7ff90000000
//! (amd64/arm64); lld-default subsystem (GUI 6.0); exports via the
//! spec2def-generated cabinet.def. cabinet-rust builds with the STANDARD user
//! target (Win32Dll → kernel_rust=false); the engine restates the i386 no-SSE
//! baseline (matching cmake's CABINET_RUST_RUSTFLAGS -sse,-sse2). rust_cohab_stub.c
//! (the wine TRACE/WARN/ERR/FIXME no-ops + HRESULT_FROM_WIN32 that cabinet-rust
//! calls) is header-free → compiled in-cargo via c_shims. The cmake
//! `spec2def(cabinet.dll cabinet.spec ADD_IMPORTLIB)` STAYS.
//!
//! Run `ninja` in build-<arch>/ first (the cargo link reuses cmake's libwine.a
//! / libzlib.a / libmsvcrtex.a / oldnames.a + the spec2def host tool).

use reactos_link::{Linker, ModuleKind, MsvcrtexKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "_DllMainCRTStartup@12", 0x7916_0000),
        "x86_64" => ("x86_64-pc-windows-gnu", "DllMainCRTStartup", 0x7ff_9000_0000),
        "aarch64" => ("aarch64-pc-windows-gnullvm", "DllMainCRTStartup", 0x7ff_9000_0000),
        o => panic!("cabinet-cargo: unsupported arch {o} (i386/amd64/arm64 only)"),
    };

    Linker::new(target_json)
        .rust_staticlib("cabinet-rust")
        // wine debug no-ops + HRESULT_FROM_WIN32 cabinet-rust calls (header-free).
        .c_shims(&["dll/win32/cabinet/rust_cohab_stub.c"])
        .spec2def("dll/win32/cabinet/cabinet.spec", "cabinet.dll")
        // ZERO cmake-built .a: wine→c_shim (step 6), zlib→libz-rs-sys (step 7),
        // msvcrtex→.msvcrtex_crt(MsvcrtexKind::Dll), and oldnames.a dropped — the cold-link
        // confirms cabinet-rust references none of its _name aliases. The
        // floor is now fully cargo-native (+ msvcrt/kernel32/ntdll imports).
        // The mingw CRT-startup floor (entry _DllMainCRTStartup@12 + _CRT_INIT +
        // security cookie + pei386 relocator + i386 chkstk/integer intrinsics),
        // compiled in-cargo from msvcrtex.cmake's sources instead of the cmake .a.
        .msvcrtex_crt(MsvcrtexKind::Dll)
        .import_lib("dll/win32/msvcrt/msvcrt.spec", "msvcrt.dll", "msvcrt")
        .import_lib("dll/win32/kernel32/kernel32.spec", "kernel32.dll", "kernel32")
        .import_lib("dll/ntdll/def/ntdll.spec", "ntdll.dll", "ntdll")
        .link(ModuleKind::Win32Dll {
            entry: entry.into(),
            image_base: Some(image_base),
            subsystem: None,
        });
}
