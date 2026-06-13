//! Cargo-driven `mlang.dll` link (i386 / amd64 / arm64). The Wine-derived
//! MLang multi-language-support DLL (dll/win32/mlang), fully translated to
//! `mlang-rust`. Replaces the cmake `add_library(mlang MODULE rust_cohab_stub.c
//! mlang_stubs.c mlang.def mlang.rc)` + `set_module_type(mlang win32dll UNICODE)`
//! + `target_link_libraries(mlang uuid wine oldnames)` +
//! `add_delay_importlibs(mlang oleaut32)` + `add_importlibs(mlang gdi32 advapi32
//! msvcrt kernel32 ntdll)` cohab. Version resource (mlang.rc) dropped.
//!
//! Win32Dll (user-mode): entry _DllMainCRTStartup@12 (i386) / DllMainCRTStartup
//! (from libmsvcrtex.a); image base 0x72420000 (i386) / 0x7fee8000000
//! (amd64/arm64); lld-default subsystem (GUI 6.0); exports via the
//! spec2def-generated mlang.def. mlang.spec has `stub`-only entries → the
//! spec2def-generated mlang_stubs.c is compiled + linked via `.spec_stubs()`.
//! mlang-rust builds with the STANDARD user target (Win32Dll → kernel_rust=false);
//! the engine restates the i386 no-SSE baseline (matching cmake's
//! MLANG_RUST_RUSTFLAGS -sse,-sse2). rust_cohab_stub.c (wine TRACE/WARN/FIXME +
//! IsEqualGUID/IsEqualIID + min + the wine list_* helpers + vtbl dispatch that
//! mlang-rust calls) is header-free → compiled in-cargo via c_shims.
//!
//! oleaut32 was a cmake DELAY import; delay-imports collapse to regular imports
//! on llvm-mingw (see the winsrv note), so a plain import_lib is equivalent. The
//! cmake `spec2def(mlang.dll mlang.spec ADD_IMPORTLIB)` STAYS.
//!
//! Run `ninja` in build-<arch>/ first (the cargo link reuses cmake's libuuid.a
//! / libwine.a / oldnames.a / libmsvcrtex.a + the spec2def host tool).

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "_DllMainCRTStartup@12", 0x7242_0000),
        "x86_64" => ("x86_64-kernel.json", "DllMainCRTStartup", 0x7fe_e800_0000),
        "aarch64" => ("aarch64-kernel.json", "DllMainCRTStartup", 0x7fe_e800_0000),
        o => panic!("mlang-cargo: unsupported arch {o} (i386/amd64/arm64 only)"),
    };

    Linker::new(target_json)
        .rust_staticlib("mlang-rust")
        // wine debug no-ops + IsEqualGUID/min/list_*/vtbl helpers mlang-rust calls.
        .c_shims(&["dll/win32/mlang/rust_cohab_stub.c"])
        .spec2def("dll/win32/mlang/mlang.spec", "mlang.dll")
        // mlang.spec has `stub`-only entries → compile + link mlang_stubs.c.
        .spec_stubs()
        .static_libs(&[
            "sdk/lib/uuid/libuuid.a",
            // libwine.a dropped — mlang-rust's wine surface (TRACE/WARN/FIXME +
            // IsEqualGUID/min/list_*/vtbl helpers) is already provided
            // header-free by rust_cohab_stub.c (c_shims above).
            "sdk/lib/crt/CMakeFiles/oldnames.dir/oldnames.a",
            "sdk/lib/crt/libmsvcrtex.a",
        ])
        .import_lib("dll/win32/oleaut32/oleaut32.spec", "oleaut32.dll", "oleaut32")
        .import_lib("win32ss/gdi/gdi32/gdi32.spec", "gdi32.dll", "gdi32")
        .import_lib("dll/win32/advapi32/advapi32.spec", "advapi32.dll", "advapi32")
        .import_lib("dll/win32/msvcrt/msvcrt.spec", "msvcrt.dll", "msvcrt")
        .import_lib("dll/win32/kernel32/kernel32.spec", "kernel32.dll", "kernel32")
        .import_lib("dll/ntdll/def/ntdll.spec", "ntdll.dll", "ntdll")
        .link(ModuleKind::Win32Dll {
            entry: entry.into(),
            image_base,
            subsystem: None,
        });
}
