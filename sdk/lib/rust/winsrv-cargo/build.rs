//! Cargo-driven `winsrv.dll` link (i386 + amd64 + arm64). Manifest for the
//! shared reactos-link engine. Seventh spine binary moved cmake→cargo (after
//! lsass + userinit + winlogon + services + smss + csrsrv) — the last of the
//! SMSS/CSRSS spine, and the one with the largest C residue.
//!
//! Mirrors the cmake winsrv.dll link (build-<arch>/build.ninja):
//!   subsystem console:5.01 ; entry _DllMain@12 (i386) / DllMain (amd64/arm64) ;
//!   -shared ; keep all sections ; exports via the spec2def-generated
//!   winsrv.def ; image base 0x78860000 (i386) / 0x7ff86000000 (amd64/arm64).
//!
//! winsrv is a PARTIAL flip: init.c is Rust (winsrv-rust, `winsrv_dll_only`)
//! but consrv + usersrv + concfg stay C. Those component libs are now almost
//! entirely Rust-flipped — each cmake `.a` carries only a tiny shim/stub TU:
//!   libconsrv.a  = consrv/consrv_rust_shim.c        (1 TU)
//!   libusersrv.a = usersrv/{usersrv_rust_stub,usersrv_rust_strsafe_shim}.c (2)
//!   libconcfg.a  = concfg/concfg_rust_stub.c        (1 TU)
//! Axis B: rather than reuse those whole cmake `.a`, compile the shims IN-cargo
//! via `c_from_compile_db` (replaying cmake's flag-exact clang command per the
//! existing consrv/usersrv/concfg cmake targets — all SKIP_PRECOMPILE_HEADERS,
//! no PCH). Likewise the 8 import libs (ntdll/kernel32/user32/gdi32/advapi32/
//! ole32/psapi/shell32) are self-hosted from each `.spec` via `.import_lib()`
//! (like the other 6 spine binaries), not reused as cmake `lib<dll>.a`. Only
//! the genuine shared leaf/runtime statics (uuid/win32ksys/libcntpr/pseh/
//! basesrv/csrsrv/chkstk/concfg_rust/cntpr_rust) remain reused — the same tier
//! every spine binary reuses. This brings winsrv to parity with the other six.
//!
//! winsrv-rust pulls concfg-rust as a Cargo dep, so libconcfg_rust.a overlaps
//! (kept for link parity; --allow-multiple-definition covers it). winsrv-rust's
//! only default feature ntddi_ws03 stays on (additive) → exactly cmake's
//! `--no-default-features --features winsrv_dll_only,ntddi_ws03`.

use reactos_link::{Linker, ModuleKind};
use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let (target_json, entry, image_base): (&str, &str, u64) = match arch.as_str() {
        "x86" => ("i686-kernel.json", "_DllMain@12", 0x7886_0000),
        "x86_64" => ("x86_64-pc-windows-gnu", "DllMain", 0x7ff_8600_0000),
        "aarch64" => ("aarch64-pc-windows-gnullvm", "DllMain", 0x7ff_8600_0000),
        o => panic!("winsrv-cargo: unsupported arch {o}"),
    };

    // cnt-rust-lib `float` feature, mirroring libcntpr.cmake / ntdll-cargo: amd64
    // + arm64 build the Rust format_float/format_floatw; i386 (no-SSE) omits it
    // and takes the C residual from liblibcntpr.a (still reused below).
    let cnt_feats: &[&str] = if arch == "x86" { &[] } else { &["float"] };

    Linker::new(target_json)
        // winsrv_dll_only gates init.rs's DllMain/GetTimeouts/ShutdownSettings.
        // ntddi_ws03 stays on as the default (additive) → matches cmake's
        // `--no-default-features --features winsrv_dll_only,ntddi_ws03`.
        // Win32Dll → user-mode target (winsrv-rust builds i686-pc-windows-gnu,
        // pulling concfg-rust as a Cargo dep).
        .rust_staticlib("winsrv-rust")
        .rust_features(&["winsrv_dll_only"])
        // cntpr's Rust half as a cargo-native sibling (was the cmake-copied
        // libcntpr_rust.a) + the DUMMY_PSEH _SEH2_Volatile* symbols compiled
        // in-cargo from dummy.c (was libpseh.a). Both proven on the other spine
        // binaries / ntdll / hal; dummy.c provides exactly libpseh.a's 2 symbols.
        .rust_lib("cnt-rust-lib", cnt_feats, true)
        // c_shims: dummy.c (DUMMY_PSEH _SEH2_Volatile* globals, was libpseh.a) +
        // the usersrv strsafe shim (the real strsafe/_Interlocked/IsListEmpty
        // provider harderror.rs needs). The strsafe shim was ALREADY headerless
        // (only <stdarg.h>; it hand-defines NTSTATUS/WCHAR/UNICODE_STRING etc. on
        // purpose to avoid colliding with ntstrsafe.h's __forceinline decls), so
        // WS1 cmake-off (2026-06-21) just moved it off c_from_compile_db onto
        // c_shims — verified symbol-identical (WS1 harness). winsrv now links free
        // of cmake's compile_commands.json. (consrv_rust_shim.c / usersrv_rust_stub
        // / concfg_rust_stub were already dropped — Rust-provided / vestigial.)
        .c_shims(&[
            "sdk/lib/pseh/dummy.c",
            "win32ss/user/winsrv/usersrv/usersrv_rust_strsafe_shim.c",
            // WS5 cmake-off: the 3 shell GUIDs winsrv pulls from the cmake
            // libuuid.a (CLSID_ShellLink / IID_IShellLinkW / IID_IPersistFile —
            // shortcut creation). Verbatim midl values, selectany → byte- and
            // section-identical to libuuid.a's members. See the file header.
            "sdk/lib/rust/winsrv-cargo/winsrv_guids.c",
        ])
        // Exports via spec2def-generated winsrv.def (ConServerDllInitialization
        // / UserServerDllInitialization / _UserSoundSentry / …).
        .spec2def("win32ss/user/winsrv/winsrv.spec", "winsrv.dll")
        // WS5 cmake-off: the win32k syscall stubs (was cmake `libwin32ksys.a` =
        // sys-stubs.S). winsrv calls 8 NtUser* stubs (Initialize/ProcessConnect/
        // ConsoleControl/…). Assemble sys-stubs.S IN-cargo as an in-group archive
        // (the ntdll `ntdllsys` pattern): it #includes the checked-in syscall
        // tables (w32ksvc32.h/w32ksvc64.h, quote-resolved next to the .S) +
        // syscalls.inc (kernel asm -I chain, WS2 committed ks*.inc fallback). The
        // linker GCs the 1466 unreferenced stubs, same as the cmake archive.
        .asm_archive("win32ksys", &["win32ss/sys-stubs.S"])
        // The genuine shared leaf/runtime statics, reused from cmake (the same
        // tier every spine binary reuses). libwinsrv_rust.a is emitted first by
        // the engine (rust_staticlib). --allow-multiple-definition (engine)
        // covers the concfg-rust / cntpr overlaps.
        // libpseh.a → dummy.c c_shim, libcntpr_rust.a → .rust_lib(cnt-rust-lib)
        // (both above). libchkstk.a KEPT: i386 winsrv-rust references __alloca.
        // liblibcntpr.a DROPPED — cnt-rust-lib (.rust_lib above) provides winsrv's
        // ENTIRE libcntpr surface. Offline nm drop-test (undefined across ALL
        // winsrv inputs — libwinsrv_rust + basesrv/csrsrv/win32ksys/concfg_rust —
        // ∩ liblibcntpr defs − cnt-rust defs) = EMPTY, so no C-variadic/float
        // residue is needed (unlike win32k, winsrv calls printf only via the
        // Rust-provided entry points). The csr server bodies stay.
        // libconcfg_rust.a DROPPED 2026-06-20 — REDUNDANT: winsrv-rust already
        // pulls concfg-rust as a Cargo dep, so all 50 of libconcfg_rust.a's syms
        // are provided by the cargo dep (offline residue: concfg_rust defs −
        // winsrv-rlib defs = EMPTY). Same pattern as libcntpr_rust→cnt-rust-lib.
        // libbasesrv.a / libcsrsrv.a self-hosted via .import_lib below (2026-06-20)
        // — both are cmake-built IMPORT libs (members basesrv.dll/csrsrv.dll +
        // __imp_ thunks), NOT bodies (basesrv/csrsrv are 100% Rust DLLs now).
        // winsrv genuinely imports them (basesrv: BaseSetProcessCreateNotify;
        // csrsrv: 21 Csr* syms), so they're self-hosted from each .spec (the
        // kdcom/bootvid/hal pattern), symbol-identical (the cmake .a is itself
        // spec2def-generated from these specs).
        //
        // WS5 cmake-off (2026-06-21): the LAST 3 cmake static libs are gone, so
        // winsrv joins the cmake-off spine. libuuid.a → winsrv_guids.c c_shim
        // (3 GUIDs); libwin32ksys.a → asm_archive(sys-stubs.S) above; libchkstk.a
        // DROPPED — an offline drop-test shows winsrv references NO chkstk symbol
        // (__alloca/__chkstk come from cnt-rust-lib / clang_rt.builtins), so the
        // cmake libchkstk.a was already inert (the userinit libwine.a pattern).
        .static_libs(&[])
        // The 8 import libs self-hosted from each `.spec` (spec2def + dlltool),
        // not reused as cmake lib<dll>.a — same as the other 6 spine binaries.
        // (shell32/ole32/psapi were cmake DELAY imports; delay-imports collapse
        // to regular imports on llvm-mingw, so plain import_lib is equivalent.)
        .import_lib("dll/ntdll/def/ntdll.spec", "ntdll.dll", "ntdll")
        .import_lib("dll/win32/kernel32/kernel32.spec", "kernel32.dll", "kernel32")
        .import_lib("win32ss/user/user32/user32.spec", "user32.dll", "user32")
        .import_lib("win32ss/gdi/gdi32/gdi32.spec", "gdi32.dll", "gdi32")
        .import_lib("dll/win32/advapi32/advapi32.spec", "advapi32.dll", "advapi32")
        .import_lib("dll/win32/ole32/ole32.spec", "ole32.dll", "ole32")
        .import_lib("dll/win32/psapi/psapi.spec", "psapi.dll", "psapi")
        .import_lib("dll/win32/shell32/shell32.spec", "shell32.dll", "shell32")
        .import_lib("subsystems/win/basesrv/basesrv.spec", "basesrv.dll", "basesrv")
        .import_lib("subsystems/csr/csrsrv/csrsrv.spec", "csrsrv.dll", "csrsrv")
        .link(ModuleKind::Win32Dll {
            entry: entry.into(),
            image_base: Some(image_base),
            subsystem: Some("console:5.01".into()),
        });
}
