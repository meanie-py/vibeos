//! Cargo-driven link of the BIOS FreeLDR body (`freeldr_pe`). BIOS sibling of
//! `uefildr-cargo`'s build.rs; mirrors the pre-prune cmake link in pcat.cmake
//! (deleted 2026-05-29 with the BIOS prune, restored 2026-10-05):
//!   subsystem  native, image base 0x10000, file/section alignment 0x200
//!   entry      `RealEntryPoint` (arch/amd64/entry.S; frldr16 jumps to it)
//!   exports    freeldr.spec via `.spec2def()`: rosload.exe imports the loader
//!              services from freeldr.sys BY NAME, so the BIOS loader needs an
//!              export table (uefildr does not: it calls RunLoader directly)
//!
//! freeldr-rust is built with NO `uefi` feature, which selects the PC-AT
//! machine (machpc/pcmem/pcdisk/pcvideo/pcvesa/pchw/hwacpi...) and BootMain ->
//! LaunchSecondStageLoader. The loader's asm (entry/int386/pnpbios, plus the
//! llvm-mingw bss markers) is assembled by cmake into libfreeldr_bios_asm.a
//! with freeldr's own include set and linked below.
//!
//! amd64 only for now: i386 also needs drvmap.S, which is 16-bit code that
//! only GNU as assembles (CMAKE_ASM16_GCC), and i386 is on the deprecation
//! track (roadmap section 9).

use reactos_link::{Linker, ModuleKind};
use std::env;

// blrtl-rust's path-deps (its own src is tracked by the .rust_lib build).
const BLRTL_RUST_DEPS: &[&str] =
    &["bitmap/src", "compress/src", "crc32/src", "largeint/src", "random/src", "byteswap/src", "mem/src", "rtl-diag/src", "rtl-strings/src"];

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    if arch != "x86_64" {
        panic!("freeldr-cargo: the BIOS loader is built for amd64 only (got {arch}); see build.rs");
    }
    let target_json = "x86_64-pc-windows-gnu";
    // The BIOS entry/int386/pnpbios asm, assembled with freeldr's own include set: the
    // engine replays cmake's compile of `freeldr_bios_asm` (pcat.cmake), or with no cmake
    // build compiles its recipe (reactos-link/clibs).
    let libs: Vec<&str> = vec!["engine:c:freeldr_bios_asm"];

    // blrtl (boot-loader RTL, boot/rtl/blrtl.cmake) is the blrtl-rust crate (the kernel's
    // RTL rlibs + nlsboot, with blrtl.cmake's per-arch membership; no C left), a .rust_lib
    // below; its per-arch asm goes in via .asm_archive() below.
    // blrtl's per-arch asm (asm.inc only — AsmDefineSet::Kernel). i386:
    // debug_asm/rtlmem/rtlswap; amd64: debug_asm; arm64: none (no ASM_SOURCE).
    let blrtl_asm: &[&str] = match arch.as_str() {
        "x86" => &[
            "sdk/lib/rtl/i386/debug_asm.S",
            "sdk/lib/rtl/i386/rtlmem.s",
            "sdk/lib/rtl/i386/rtlswap.S",
        ],
        "x86_64" => &["sdk/lib/rtl/amd64/debug_asm.S"],
        _ => &[],
    };

    // libcntpr (cntpr C floor, sdk/lib/crt/libcntpr.cmake) compiled in-cargo via
    // c_from_compile_db — was the reused cmake liblibcntpr.a (cmake-off WS1 DEEP).
    // The common C set (math _invoke_matherr/exp2f, mbstring, misc fltused, the 12
    // printf shells, string atol/wctype/winesup, scanf_trampoline); i386 adds the
    // x87 math wrappers (ci*/libm_sse2/nexttowardf), amd64 adds ehandler.c. Source
    // list is a subset of the cmake `libcntpr` target's objects (with no cmake build,
    // their lines of clibs/<arch>/libcntpr.recipe). asm intrinsics are
    // GC'd (gnu target) — no asm_archive.
    let mut libcntpr_c: Vec<&str> = vec![
        "sdk/lib/crt/math/_invoke_matherr.c",
        "sdk/lib/crt/math/exp2f.c",
        "sdk/lib/crt/mbstring/mbstrlen.c",
        "sdk/lib/crt/misc/fltused.c",
        "sdk/lib/crt/printf/_snprintf.c",
        "sdk/lib/crt/printf/_snwprintf.c",
        "sdk/lib/crt/printf/_swprintf.c",
        "sdk/lib/crt/printf/_vscprintf.c",
        "sdk/lib/crt/printf/_vscwprintf.c",
        "sdk/lib/crt/printf/_vsnprintf.c",
        "sdk/lib/crt/printf/_vsnwprintf.c",
        "sdk/lib/crt/printf/sprintf.c",
        "sdk/lib/crt/printf/swprintf.c",
        "sdk/lib/crt/printf/vprintf.c",
        "sdk/lib/crt/printf/vsprintf.c",
        "sdk/lib/crt/printf/vswprintf.c",
        "sdk/lib/crt/string/atol.c",
        "sdk/lib/crt/string/wctype.c",
        "sdk/lib/crt/string/winesup.c",
        "sdk/lib/rust/cnt-rust/src/cabi/scanf_trampoline.c",
    ];
    match arch.as_str() {
        "x86" => libcntpr_c.extend_from_slice(&[
            "sdk/lib/crt/math/nexttowardf.c",
            "sdk/lib/crt/math/i386/ci.c",
            "sdk/lib/crt/math/i386/cicos.c",
            "sdk/lib/crt/math/i386/cilog.c",
            "sdk/lib/crt/math/i386/cipow.c",
            "sdk/lib/crt/math/i386/cisin.c",
            "sdk/lib/crt/math/i386/cisqrt.c",
            "sdk/lib/crt/math/i386/libm_sse2.c",
        ]),
        "x86_64" => libcntpr_c.push("sdk/lib/crt/except/amd64/ehandler.c"),
        _ => {}
    }

    // cnt-rust-lib features mirror sdk/lib/crt/libcntpr.cmake: i386 builds
    // WITHOUT `float` (no-SSE pre-boot, no by-value f64 ABI); amd64/arm64 with
    // `float`. build_rust_crate's cnt-rust-lib special case supplies the exact
    // per-arch RUSTFLAGS (i386 -sse,-sse2 -C no-prepopulate-passes; else
    // -C no-prepopulate-passes). Same .rust_lib() path ntdll-cargo uses.
    // `freeldr`: the loader's own iswctype (freeldr-rust) is the only one.
    let cnt_feats: &[&str] = if arch == "x86" { &["freeldr"] } else { &["float", "freeldr"] };

    let linker = Linker::new(target_json)
        // No `uefi` feature: the PC-AT machine and BootMain.
        .rust_staticlib("freeldr-rust")
        .rust_user_target()
        .rust_watch(&["cmboot-rust/src", "cnt-rust/src", "cnt-ctype/src"])
        .rust_watch(BLRTL_RUST_DEPS)
        .rust_lib("cmlib-rust", &["bldr"], true)
        .rust_lib("blrtl-rust", &[], true)
        .c_from_compile_db("libcntpr", &libcntpr_c)
        .rust_lib("cnt-rust-lib", cnt_feats, true)
        .rust_lib("cport-rust", &[], true)
        .static_libs(&libs)
        .c_shims(&[
            "boot/freeldr/freeldr/lib/strsafe_shim.c",
            "boot/freeldr/freeldr/lib/cmlib_loader_stubs.c",
        ])
        .c_shim_flags(&["-mno-sse"])
        .asm_archive("blrtl_asm", blrtl_asm)
        .spec2def("boot/freeldr/freeldr/freeldr.spec", "freeldr.sys");
    // The whole image, .bss included, must fit in conventional memory below
    // MEMORY_MARGIN (x86common.h: MAX_FREELDR_PE_SIZE = 0x77000), which leaves
    // room above it for the INT 13h bounce buffer under the EBDA. At the default
    // opt-level the Rust body overflowed that; a loader is never the hot path.
    let linker = linker.rust_build_rustflags("-C opt-level=s");
    linker.link(ModuleKind::BiosLoader {
        entry: "RealEntryPoint".into(),
        image_base: 0x1_0000,
    });
}
