//! Drive the cargo-side link of `ntoskrnl.exe`.
//!
//! cmake still owns the asm objects, the .res resource, the .def file, and
//! the ~30 sibling static libs (rtl, cmlib, hal, etc.). cargo owns the C
//! shim compile (via cc-rs), the kernel-rust staticlib build, and the
//! final link.
//!
//! The user must run `ninja ntoskrnl` once before `cargo build` so the
//! cmake-side artifacts exist. Future iterations will subsume the asm and
//! the static libs as well.
//!
//! `kernel-rust` is `crate-type = ["staticlib"]` — cargo can't link
//! staticlibs as Rust deps, so build.rs invokes `cargo build -p kernel-rust`
//! manually for the same custom target and emits the resulting
//! `libkernel_rust.a` as an absolute-path link arg.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let arch = match target_arch.as_str() {
        "x86" => "i386",
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        other => panic!("unsupported target arch: {other}"),
    };

    if arch != "i386" {
        panic!("ntoskrnl-cargo PoC supports i386 only for now");
    }

    // Resolve workspace root + cmake build dir.
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    // sdk/lib/rust/ntoskrnl-cargo → repo root is 4 levels up.
    let repo_root = manifest_dir
        .ancestors()
        .nth(4)
        .expect("repo root")
        .to_path_buf();
    let build_dir = repo_root.join(format!("build-{arch}"));
    let cmake_obj_dir = build_dir.join("ntoskrnl/CMakeFiles/ntoskrnl.dir");
    let workspace = repo_root.join("sdk/lib/rust");

    if !build_dir.exists() {
        panic!(
            "cmake build dir {} does not exist — run `cd build-{arch} && ninja ntoskrnl` first",
            build_dir.display()
        );
    }

    // ---- Build kernel-rust as a staticlib ourselves --------------------------
    // Cargo can't pick up `crate-type = ["staticlib"]` as a Rust dep, so we
    // shell out and then emit the absolute path of the resulting .a file.
    let target_json = manifest_dir.join("i686-kernel.json");
    let krust_target_dir = workspace.join("target/ntoskrnl-cargo-krust");
    let status = Command::new(env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .arg("build")
        .arg("--release")
        .arg("--target").arg(&target_json)
        .arg("-Z").arg("build-std=core,alloc,compiler_builtins")
        .arg("-Z").arg("build-std-features=compiler-builtins-mem")
        .arg("-p").arg("kernel-rust")
        .arg("--manifest-path").arg(workspace.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", &krust_target_dir)
        // Force the nightly toolchain so `-Z build-std` is accepted.
        // `+nightly` doesn't work via Command (rustup proxy only); the
        // env-var route is the equivalent.
        .env("RUSTUP_TOOLCHAIN", "nightly")
        .status()
        .expect("failed to invoke cargo for kernel-rust");
    if !status.success() {
        panic!("cargo build -p kernel-rust failed");
    }
    let kernel_rust_a = krust_target_dir.join("i686-kernel/release/libkernel_rust.a");
    if !kernel_rust_a.exists() {
        panic!("kernel-rust build produced no libkernel_rust.a at {}", kernel_rust_a.display());
    }

    // Cause re-link if any cmake .obj or static lib changes.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=i686-kernel.json");
    println!("cargo:rerun-if-changed={}", cmake_obj_dir.display());
    println!("cargo:rerun-if-changed={}", build_dir.join("ntoskrnl/ntoskrnl.def").display());

    // ---- C shim compile (cc-rs) ---------------------------------------------
    // 8 header-free shims compile with no -I (clang's own stdarg/stddef/stdint
    // is enough). io_tables.c is the lone shim that still pulls
    // <ntoskrnl.h> + internal/io_i.h to materialize the 8 Iop*Operation*
    // tables consumed by Rust iofunc.rs — give it the kernel include chain.
    let header_free_shims: &[&str] = &[
        "ntoskrnl/io/iomgr/io_rust_shims.c",
        "ntoskrnl/ke/ke_rust_shims.c",
        "ntoskrnl/mm/mm_rust_shims.c",   // body is _M_ARM64-gated; empty .o on i386
        "ntoskrnl/variadic_shims.c",
        "ntoskrnl/lto_workaround.c",
        "ntoskrnl/mm_lock_outlines.c",
        "ntoskrnl/ntprintf_trampoline.c",
        "ntoskrnl/kdbg/kdbg_rust_shims.c",
    ];
    for relpath in header_free_shims {
        let abs = repo_root.join(relpath);
        if !abs.exists() {
            panic!("missing shim source: {}", abs.display());
        }
        compile_shim(&repo_root, &abs, /*kernel_includes=*/ false);
        println!("cargo:rerun-if-changed={}", abs.display());
    }
    // io_tables.c retired 2026-05-08; replaced by Rust iomgr/io_tables.rs.
    // The 8 Iop*Operation* tables are now `#[no_mangle] pub static` arrays
    // in kernel-rust, computed from `core::mem::size_of::<>()` against
    // ntkrnl-sys bindgen types.

    // ---- Generate ks386.inc ourselves --------------------------------------
    // The 8 asm sources all `#include <ks386.inc>`, which cmake produces by
    // compiling `sdk/include/asm/genincdata.c` to a custom `.asmdef`-section
    // PE blob and walking that section with the `geninc` host tool. We do
    // both steps in-process: cc-rs compiles `genincdata.c` to a `.obj`, and
    // a small Rust COFF parser walks the `.asmdef` section and emits
    // `ks386.inc` to OUT_DIR.
    let ks386_inc_dir = generate_ks386_inc(&repo_root);
    println!("cargo:rerun-if-changed={}", repo_root.join("sdk/include/asm/genincdata.c").display());
    println!("cargo:rerun-if-changed={}", repo_root.join("sdk/include/asm/ks386.template.h").display());
    println!("cargo:rerun-if-changed={}", repo_root.join("sdk/include/asm/ksx.template.h").display());

    // ---- Asm compile (cc-rs) ------------------------------------------------
    // The 8 .S/.s files all `#include <asm.inc>` + `<ks386.inc>` + an
    // `internal/i386/asmmacro.S` shim — they need the asm-specific include
    // dir (`sdk/include/asm`) + our generated `OUT_DIR/asm-gen` (for the
    // freshly-emitted `ks386.inc`) + the full kernel `-I` chain.
    let asm_sources: &[&str] = &[
        "ntoskrnl/ex/zw.S",
        "ntoskrnl/ex/i386/fastinterlck_asm.S",
        "ntoskrnl/ex/i386/ioport.S",
        "ntoskrnl/ke/i386/ctxswitch.S",
        "ntoskrnl/ke/i386/trap.s",
        "ntoskrnl/ke/i386/usercall_asm.S",
        "ntoskrnl/ke/i386/zeropage.S",
        "ntoskrnl/rtl/i386/prefetch.S",
    ];
    for relpath in asm_sources {
        let abs = repo_root.join(relpath);
        if !abs.exists() {
            panic!("missing asm source: {}", abs.display());
        }
        compile_asm(&repo_root, &abs, &ks386_inc_dir);
        println!("cargo:rerun-if-changed={}", abs.display());
    }
    // ---- Resource (.res) — cc-rs-style windres invocation -------------------
    let rc_source = repo_root.join("ntoskrnl/ntoskrnl.rc");
    let res_out = compile_resource(&repo_root, &rc_source);
    println!("cargo:rustc-link-arg={}", res_out.display());
    println!("cargo:rerun-if-changed={}", rc_source.display());
    let _ = cmake_obj_dir;

    // Export def file (drives the PE export table).
    let def_file = build_dir.join("ntoskrnl/ntoskrnl.def");
    if !def_file.exists() {
        panic!("missing def file: {}", def_file.display());
    }
    println!("cargo:rustc-link-arg={}", def_file.display());

    // Static libs (in cmake link order). libkernel_rust.a goes first inside
    // the group so its symbols are visible to all the cmake-built libs that
    // reference them.
    let static_libs = [
        "sdk/lib/drivers/ntoskrnl_vista/libntoskrnl_vista.a",
        "sdk/lib/cportlib/libcportlib.a",
        "sdk/lib/drivers/csq/libcsq.a",
        "sdk/lib/pseh/libpseh.a",
        "sdk/lib/drivers/arbiter/libarbiter.a",
        "sdk/lib/cmlib/libcmlib.a",
        "sdk/lib/lsalib/libntlsalib.a",
        "sdk/lib/rtl/librtl.a",
        "sdk/lib/rossym/librossym.a",
        "sdk/lib/crt/liblibcntpr.a",
        "sdk/lib/vcruntime/libsetjmp.a",
        "sdk/lib/wdmguid/libwdmguid.a",
        "sdk/lib/poguid/libpoguid.a",
        "sdk/lib/ioevent/libioevent.a",
        "hal/halx86/CMakeFiles/libhal.dir/libhal.a",
        "drivers/base/kdrosdbg/CMakeFiles/libkdcom.dir/libkdcom.a",
        "drivers/base/bootvid/CMakeFiles/libbootvid.dir/libbootvid.a",
        "sdk/lib/crt/libchkstk.a",
        "sdk/lib/rtl/libreactos_avltable.a",
        "sdk/lib/rtl/libreactos_bitmap.a",
        "sdk/lib/rtl/libreactos_slist.a",
        "sdk/lib/rtl/libreactos_compress.a",
        "sdk/lib/rtl/libreactos_crc32.a",
        "sdk/lib/rtl/libreactos_random.a",
        "sdk/lib/rtl/libreactos_encode.a",
        "sdk/lib/rtl/libreactos_largeint.a",
        "sdk/lib/rtl/libreactos_access.a",
        "sdk/lib/rtl/libreactos_byteswap.a",
        "sdk/lib/rtl/libreactos_interlck.a",
        "sdk/lib/rtl/libreactos_luid.a",
        "sdk/lib/rtl/libreactos_prefix.a",
        "sdk/lib/rtl/libreactos_mem.a",
        "sdk/lib/rtl/libreactos_handle.a",
        "sdk/lib/rtl/libreactos_time.a",
        "sdk/lib/rtl/libreactos_splaytree.a",
        "sdk/lib/rtl/libreactos_generictable.a",
    ];
    println!("cargo:rustc-link-arg=-Wl,--start-group");
    println!("cargo:rustc-link-arg={}", kernel_rust_a.display());
    for lib in static_libs.iter() {
        let full = build_dir.join(lib);
        if !full.exists() {
            panic!("missing static lib: {}", full.display());
        }
        println!("cargo:rustc-link-arg={}", full.display());
    }
    println!("cargo:rustc-link-arg=-Wl,--end-group");

    // libclang_rt.builtins-i386.a — supplied by llvm-mingw clang.
    let llvm_mingw = repo_root.join(".build-tools/llvm-mingw");
    let clang_rt = llvm_mingw.join("lib/clang/19/lib/windows/libclang_rt.builtins-i386.a");
    println!("cargo:rustc-link-arg={}", clang_rt.display());

    // Kernel-mode PE-COFF link flags (from cmake set_module_type).
    let link_flags = [
        "-nostdlib",
        "-fuse-ld=lld",
        "-Wl,--enable-auto-image-base,--disable-auto-import",
        "-Wl,--disable-stdcall-fixup,--gc-sections",
        "-Wl,--subsystem,native:5.01",
        "-Wl,-entry,_KiSystemStartup@4",
        "-Wl,--image-base,0x00400000",
        "-Wl,--major-image-version,5",
        "-Wl,--minor-image-version,01",
        "-Wl,--major-os-version,5",
        "-Wl,--minor-os-version,01",
        "-Wl,--exclude-all-symbols,-file-alignment=0x1000,-section-alignment=0x1000",
        "-Wl,--allow-multiple-definition",
    ];
    for f in link_flags.iter() {
        println!("cargo:rustc-link-arg={f}");
    }
}

/// Kernel-mode + ReactOS-build DEFINES applied to every ntoskrnl C/asm/RC
/// translation unit. Mirrors cmake's `DEFINES = …` line for any ntoskrnl
/// source. Extracted to a const so the C, asm, and RC paths share one
/// source of truth.
const KERNEL_DEFINES: &[(&str, Option<&str>)] = &[
    ("D3D_UMD_INTERFACE_VERSION", Some("0x000C")),
    ("DBG", Some("1")),
    ("DLL_EXPORT_VERSION", Some("0x502")),
    ("DXGKDDI_INTERFACE_VERSION", Some("0x1052")),
    ("KDBG", None),
    ("MINGW_HAS_SECURE_API", Some("1")),
    ("NTDDI_VERSION", Some("0x05020400")),
    ("NTKRNLVISTA", None),
    ("USE_COMPILER_EXCEPTIONS", None),
    ("WINVER", Some("0x502")),
    ("_CRT_NON_CONFORMING_SWPRINTFS", None),
    ("_CRT_SUPPRESS_RESTRICT", None),
    ("_GLIBCXX_HAVE_BROKEN_VSWPRINTF", None),
    ("_M_IX86", None),
    ("_NEW_DELETE_OPERATORS_", None),
    ("_NTOSKRNL_", None),
    ("_NTSYSTEM_", None),
    ("_SEH_ENABLE_TRACE", None),
    ("_SETUPAPI_VER", Some("0x502")),
    ("_USE_32BIT_TIME_T", None),
    ("_USE_DUMMY_PSEH", Some("1")),
    ("_WIN32_IE", Some("0x603")),
    ("_WIN32_WINDOWS", Some("0x502")),
    ("_WIN32_WINNT", Some("0x502")),
    ("_X86_", None),
    ("__REACTOS__", None),
    ("__ROS_ROSSYM__", None),
    ("__i386__", None),
    ("i386", None),
];

/// Common kernel-mode flags + DEFINES applied to every ntoskrnl C/asm
/// translation unit. Mirrors the `FLAGS = …` and `DEFINES = …` lines ninja
/// emits for any ntoskrnl source.
fn apply_kernel_base(build: &mut cc::Build, clang: &Path) {
    build
        .compiler(clang)
        .warnings(false)
        .extra_warnings(false)
        .flag("--target=i686-w64-mingw32")
        .flag("-pipe")
        .flag("-fms-extensions")
        .flag("-fno-strict-aliasing")
        .flag("-fno-common")
        .flag("-mlong-double-64")
        .flag("-Wno-microsoft")
        .flag("-Wno-pragma-pack")
        .flag("-fno-associative-math")
        .flag("-fno-builtin-stpcpy")
        .flag("-gdwarf-2")
        .flag("-gstrict-dwarf")
        .flag("-march=pentium")
        .flag("-mtune=generic")
        .flag("-O1")
        .flag("-fno-optimize-sibling-calls")
        .flag("-fno-omit-frame-pointer")
        .flag("-mstackrealign")
        .flag("-mxsave")
        .flag("-Wno-unknown-warning-option");
    for (name, value) in KERNEL_DEFINES {
        build.define(name, *value);
    }
}

/// Kernel `-I` paths cmake adds to every ntoskrnl C/asm/RC translation
/// unit. Mirrors cmake's `INCLUDES = …` line; consumed by both the C/asm
/// path (`apply_kernel_includes` via `cc::Build::include`) and the RC path
/// (`compile_resource` via raw `-I` args).
const KERNEL_INCLUDES: &[&str] = &[
    "build-i386/ntoskrnl",
    "ntoskrnl",
    "sdk/include",
    "sdk/include/crt",
    "sdk/include/ddk",
    "sdk/include/ndk",
    "sdk/include/psdk",
    "sdk/include/reactos",
    "sdk/include/reactos/libs",
    "sdk/include/vcruntime",
    "sdk/include/winrt",
    "build-i386/sdk/include",
    "build-i386/sdk/include/psdk",
    "build-i386/sdk/include/ddk",
    "build-i386/sdk/include/dxsdk",
    "build-i386/sdk/include/reactos",
    "build-i386/sdk/include/reactos/mc",
    "sdk/include/dxsdk",
    "sdk/lib/pseh/include",
    "",
    "sdk/lib/drivers/arbiter",
    "sdk/lib/cmlib",
    "ntoskrnl/include",
    "build-i386/ntoskrnl/include",
    "build-i386/ntoskrnl/include/internal",
    "sdk/include/reactos/drivers",
];

/// Adds the full kernel `-I` chain + the `-nostdinc -isystem<crt>
/// -isystem<clang>` triplet that suppresses mingw's user-mode CRT search.
/// Required by the one C shim that still drags `<ntoskrnl.h>` in
/// (`io_tables.c`) and by every asm source (which `#include <asm.inc>` +
/// `<ks386.inc>`).
fn apply_kernel_includes(build: &mut cc::Build, repo_root: &Path, llvm_mingw: &Path) {
    for inc in KERNEL_INCLUDES {
        build.include(repo_root.join(inc));
    }
    build
        .flag("-nostdinc")
        .flag("-isystem")
        .flag(repo_root.join("sdk/include/crt").to_str().unwrap())
        .flag("-isystem")
        .flag(llvm_mingw.join("lib/clang/19/include").to_str().unwrap());
}

fn shim_obj_dir() -> PathBuf {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dir = out_dir.join("shims");
    std::fs::create_dir_all(&dir).expect("mkdir OUT_DIR/shims");
    dir
}

/// Compile one C shim with kernel-mode flags + DEFINES, emit it as a `.o`,
/// and add it to the rustc link line. All current shims pass
/// `kernel_includes=false` and only need `<stdarg.h>` / `<stddef.h>` /
/// `<stdint.h>` from clang's resource dir; the parameter is kept as an
/// escape hatch for future shims that may need to pull `<ntoskrnl.h>`.
fn compile_shim(repo_root: &Path, source: &Path, kernel_includes: bool) {
    let llvm_mingw = repo_root.join(".build-tools/llvm-mingw");
    let clang = llvm_mingw.join("bin/i686-w64-mingw32-clang");

    let mut build = cc::Build::new();
    apply_kernel_base(&mut build, &clang);
    build
        .file(source)
        .out_dir(shim_obj_dir())
        .flag("-std=gnu2x");
    if kernel_includes {
        apply_kernel_includes(&mut build, repo_root, &llvm_mingw);
        assert!(
            repo_root.join("build-i386").exists(),
            "kernel_includes path needs build-i386/ to exist for generated headers"
        );
    }

    emit_objects(&build, source);
}

/// Compile one .S/.s file with kernel-mode flags + DEFINES. Asm files all
/// `#include <asm.inc>` + `<ks386.inc>` + `internal/i386/asmmacro.S`. They
/// need the asm-specific include dir (`sdk/include/asm`) + the generated
/// `ks386.inc` dir (`generated_inc_dir`, which build.rs produces fresh via
/// `generate_ks386_inc()`) + the full kernel `-I` chain + `-D__ASM__`.
fn compile_asm(repo_root: &Path, source: &Path, generated_inc_dir: &Path) {
    let llvm_mingw = repo_root.join(".build-tools/llvm-mingw");
    let clang = llvm_mingw.join("bin/i686-w64-mingw32-clang");

    let mut build = cc::Build::new();
    apply_kernel_base(&mut build, &clang);
    apply_kernel_includes(&mut build, repo_root, &llvm_mingw);
    build
        .file(source)
        .out_dir(shim_obj_dir())
        .define("__ASM__", None)
        // Asm-specific include dirs (cmake adds these as the first two -I
        // before the C-side INCLUDES).
        .include(repo_root.join("sdk/include/asm"))
        .include(generated_inc_dir)
        // Lowercase .s files don't get `-x assembler-with-cpp` from cc-rs's
        // extension sniffing; force it for parity with cmake's rule.
        .flag("-x")
        .flag("assembler-with-cpp");

    emit_objects(&build, source);
}

fn emit_objects(build: &cc::Build, source: &Path) {
    let objs = build.compile_intermediates();
    if objs.is_empty() {
        panic!("cc-rs produced no objects for {}", source.display());
    }
    for obj in objs {
        println!("cargo:rustc-link-arg={}", obj.display());
    }
}

/// 136 LANGUAGE_* defines the cmake RC rule passes to windres. Lifted
/// verbatim from `RC_COMPILER__ntoskrnl_Debug` in `build.ninja`.
const LANGUAGES: &[&str] = &[
    "LANGUAGE_AF_ZA", "LANGUAGE_AR_AE", "LANGUAGE_AR_BH", "LANGUAGE_AR_DZ",
    "LANGUAGE_AR_EG", "LANGUAGE_AR_IQ", "LANGUAGE_AR_JO", "LANGUAGE_AR_KW",
    "LANGUAGE_AR_LB", "LANGUAGE_AR_LY", "LANGUAGE_AR_MA", "LANGUAGE_AR_OM",
    "LANGUAGE_AR_QA", "LANGUAGE_AR_SA", "LANGUAGE_AR_SY", "LANGUAGE_AR_TN",
    "LANGUAGE_AR_YE", "LANGUAGE_AZ_AZ", "LANGUAGE_BE_BY", "LANGUAGE_BG_BG",
    "LANGUAGE_BN_BD", "LANGUAGE_BN_IN", "LANGUAGE_CA_ES", "LANGUAGE_CS_CZ",
    "LANGUAGE_DA_DK", "LANGUAGE_DE_AT", "LANGUAGE_DE_CH", "LANGUAGE_DE_DE",
    "LANGUAGE_DE_LI", "LANGUAGE_DE_LU", "LANGUAGE_DV_MV", "LANGUAGE_EL_GR",
    "LANGUAGE_EN_AU", "LANGUAGE_EN_BZ", "LANGUAGE_EN_CA", "LANGUAGE_EN_CB",
    "LANGUAGE_EN_GB", "LANGUAGE_EN_IE", "LANGUAGE_EN_JM", "LANGUAGE_EN_NZ",
    "LANGUAGE_EN_PH", "LANGUAGE_EN_TT", "LANGUAGE_EN_US", "LANGUAGE_EN_ZA",
    "LANGUAGE_EN_ZW", "LANGUAGE_EO_AA", "LANGUAGE_ES_AR", "LANGUAGE_ES_BO",
    "LANGUAGE_ES_CL", "LANGUAGE_ES_CO", "LANGUAGE_ES_CR", "LANGUAGE_ES_DO",
    "LANGUAGE_ES_EC", "LANGUAGE_ES_ES", "LANGUAGE_ES_GT", "LANGUAGE_ES_HN",
    "LANGUAGE_ES_MX", "LANGUAGE_ES_NI", "LANGUAGE_ES_PA", "LANGUAGE_ES_PE",
    "LANGUAGE_ES_PR", "LANGUAGE_ES_PY", "LANGUAGE_ES_SV", "LANGUAGE_ES_UY",
    "LANGUAGE_ES_VE", "LANGUAGE_ET_EE", "LANGUAGE_EU_ES", "LANGUAGE_FA_IR",
    "LANGUAGE_FI_FI", "LANGUAGE_FO_FO", "LANGUAGE_FR_BE", "LANGUAGE_FR_CA",
    "LANGUAGE_FR_CH", "LANGUAGE_FR_FR", "LANGUAGE_FR_LU", "LANGUAGE_FR_MC",
    "LANGUAGE_GL_ES", "LANGUAGE_GU_IN", "LANGUAGE_HE_IL", "LANGUAGE_HI_IN",
    "LANGUAGE_HR_HR", "LANGUAGE_HU_HU", "LANGUAGE_HY_AM", "LANGUAGE_ID_ID",
    "LANGUAGE_IS_IS", "LANGUAGE_IT_CH", "LANGUAGE_IT_IT", "LANGUAGE_JA_JP",
    "LANGUAGE_KA_GE", "LANGUAGE_KK_KZ", "LANGUAGE_KN_IN", "LANGUAGE_KOK_IN",
    "LANGUAGE_KO_KR", "LANGUAGE_KY_KG", "LANGUAGE_LT_LT", "LANGUAGE_LV_LV",
    "LANGUAGE_MK_MK", "LANGUAGE_MN_MN", "LANGUAGE_MR_IN", "LANGUAGE_MS_BN",
    "LANGUAGE_MS_MY", "LANGUAGE_MY_MM", "LANGUAGE_NL_BE", "LANGUAGE_NL_NL",
    "LANGUAGE_NN_NO", "LANGUAGE_PA_IN", "LANGUAGE_PL_PL", "LANGUAGE_PT_BR",
    "LANGUAGE_PT_PT", "LANGUAGE_RM_CH", "LANGUAGE_RO_RO", "LANGUAGE_RU_RU",
    "LANGUAGE_SA_IN", "LANGUAGE_SK_SK", "LANGUAGE_SL_SI", "LANGUAGE_SQ_AL",
    "LANGUAGE_SR_SP", "LANGUAGE_SV_FI", "LANGUAGE_SV_SE", "LANGUAGE_SW_KE",
    "LANGUAGE_SYR_SY", "LANGUAGE_TA_IN", "LANGUAGE_TE_IN", "LANGUAGE_TH_TH",
    "LANGUAGE_TR_TR", "LANGUAGE_TT_TA", "LANGUAGE_UK_UA", "LANGUAGE_UR_PK",
    "LANGUAGE_UZ_UZ", "LANGUAGE_VI_VN", "LANGUAGE_WA_BE", "LANGUAGE_ZH_CN",
    "LANGUAGE_ZH_HK", "LANGUAGE_ZH_MO", "LANGUAGE_ZH_SG", "LANGUAGE_ZH_TW",
];

/// Compile `ntoskrnl.rc` to a PE-COFF resource file via llvm-mingw's
/// `i686-w64-mingw32-windres`, mirroring the cmake RC rule. Returns the
/// path to the resulting `.res` for `cargo:rustc-link-arg=`.
fn compile_resource(repo_root: &Path, source: &Path) -> PathBuf {
    let llvm_mingw = repo_root.join(".build-tools/llvm-mingw");
    let windres = llvm_mingw.join("bin/i686-w64-mingw32-windres");
    let clang = llvm_mingw.join("bin/i686-w64-mingw32-clang");

    let out_dir = shim_obj_dir();
    let out_path = out_dir.join("ntoskrnl.rc.res");
    let dep_path = out_dir.join("ntoskrnl.rc.res.d");

    let mut cmd = Command::new(&windres);
    cmd.arg("-O").arg("coff");

    // INCLUDES — full kernel chain (cmake's RC FLAGS line is just an extra
    // `-I sdk/include/crt`, but `sdk/include/crt` is already in
    // `KERNEL_INCLUDES`, so the chain is sufficient).
    for inc in KERNEL_INCLUDES {
        cmd.arg("-I").arg(repo_root.join(inc));
    }

    // Preprocessor: clang in -E mode with ms-extensions disabled.
    let out_str = out_path.to_str().expect("utf8 OUT path");
    let dep_str = dep_path.to_str().expect("utf8 DEP path");
    cmd.arg(format!("--preprocessor={}", clang.display()))
        .arg("--preprocessor-arg=--target=i686-w64-mingw32")
        .arg("--preprocessor-arg=-E")
        .arg("--preprocessor-arg=-nostdinc")
        .arg("--preprocessor-arg=-xc-header")
        .arg("--preprocessor-arg=-MMD")
        .arg("--preprocessor-arg=-MF")
        .arg(format!("--preprocessor-arg={dep_str}"))
        .arg("--preprocessor-arg=-MT")
        .arg(format!("--preprocessor-arg={out_str}"));

    // RC-specific defines.
    cmd.arg("-DRC_INVOKED")
        .arg("-D__WIN32__=1")
        .arg("-D__FLAT__=1");

    // 136 LANGUAGE_* defines — required for common.ver to emit every
    // language's StringFileInfo block.
    for lang in LANGUAGES {
        cmd.arg(format!("-D{lang}"));
    }

    // Standard ntoskrnl C-side defines (KERNEL_DEFINES).
    for (name, value) in KERNEL_DEFINES {
        match value {
            Some(v) => cmd.arg(format!("-D{name}={v}")),
            None => cmd.arg(format!("-D{name}")),
        };
    }

    cmd.arg(source).arg(&out_path);

    let status = cmd.status().expect("failed to invoke windres");
    if !status.success() {
        panic!(
            "windres failed for {} (exit {:?})",
            source.display(),
            status.code()
        );
    }
    if !out_path.exists() {
        panic!("windres ran but produced no output at {}", out_path.display());
    }
    out_path
}

// ---- ks386.inc generation (replaces cmake's geninc + genincdata.dll) -------

/// Compile `sdk/include/asm/genincdata.c` to a PE-COFF `.obj`, parse its
/// `.asmdef` section, and emit `ks386.inc` to `OUT_DIR/asm-gen/`. Returns
/// the include-path directory the asm sources should add via `-I`.
///
/// `genincdata.c` materializes an `ASMGENDATA Table[]` into a custom
/// `.asmdef` PE section by `__attribute__((section(".asmdef")))`. Each
/// record is 64 bytes:
///   - byte 0:    Type (0=END, 1=RAW, 2=CONSTANT, 3=HEADER)
///   - bytes 1–55: Name (null-terminated, padded)
///   - bytes 56–63: Value (uint64 little-endian)
///
/// cmake builds this as a DLL via the `genincdata` target and runs the
/// `geninc` host tool to walk the section. We skip the DLL link and parse
/// the `.obj` directly — the section data is identical (it's all
/// compile-time constants from `<ks386.template.h>` + `<ksx.template.h>`,
/// no relocations needed).
fn generate_ks386_inc(repo_root: &Path) -> PathBuf {
    let llvm_mingw = repo_root.join(".build-tools/llvm-mingw");
    let clang = llvm_mingw.join("bin/i686-w64-mingw32-clang");
    let src = repo_root.join("sdk/include/asm/genincdata.c");
    if !src.exists() {
        panic!("missing genincdata.c at {}", src.display());
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let gen_dir = out_dir.join("asm-gen");
    let obj_subdir = out_dir.join("geninc-obj");
    std::fs::create_dir_all(&gen_dir).expect("mkdir asm-gen");
    std::fs::create_dir_all(&obj_subdir).expect("mkdir geninc-obj");

    // Compile genincdata.c to a .obj — kernel-mode flags + DEFINES + the
    // full kernel `-I` chain (it pulls <ntdef.h> / <ntndk.h> / <bugcodes.h>).
    // No PE link needed; we read the .obj directly.
    let mut build = cc::Build::new();
    apply_kernel_base(&mut build, &clang);
    apply_kernel_includes(&mut build, repo_root, &llvm_mingw);
    build.file(&src).out_dir(&obj_subdir).flag("-std=gnu2x");
    let objs = build.compile_intermediates();
    let obj = objs.into_iter().next().expect("genincdata.c produced no object");

    // Parse the .obj's `.asmdef` section.
    let asmdef = read_coff_section(&obj, ".asmdef");

    // Walk ASMGENDATA records and emit ks386.inc.
    let inc_path = gen_dir.join("ks386.inc");
    let mut text = String::new();
    for chunk in asmdef.chunks_exact(64) {
        let ty = chunk[0];
        let name = {
            let nb = &chunk[1..56];
            let end = nb.iter().position(|&b| b == 0).unwrap_or(55);
            std::str::from_utf8(&nb[..end])
                .unwrap_or_else(|e| panic!("non-utf8 in .asmdef name: {e}"))
        };
        let value = u64::from_le_bytes(chunk[56..64].try_into().unwrap());
        match ty {
            0 => break,                                              // TYPE_END
            1 => text.push_str(&format!("{name}\n")),                // TYPE_RAW
            2 => text.push_str(&format!("{name} = 0x{value:x}\n")), // TYPE_CONSTANT
            3 => text.push_str(&format!("\n/* {name} */\n")),        // TYPE_HEADER
            other => panic!("unknown ASMGENDATA type {other} in .asmdef"),
        }
    }
    std::fs::write(&inc_path, &text).expect("write ks386.inc");
    println!(
        "cargo:warning=ntoskrnl-cargo: generated {} ({} bytes)",
        inc_path.display(),
        text.len()
    );
    gen_dir
}

/// Minimal PE-COFF object parser: read a named section's raw bytes from a
/// `.obj` file (no DOS header, no PE signature — the COFF File Header lives
/// at offset 0).
fn read_coff_section(obj: &Path, section_name: &str) -> Vec<u8> {
    let bytes = std::fs::read(obj).unwrap_or_else(|e| panic!("read {}: {e}", obj.display()));
    if bytes.len() < 20 {
        panic!("{} is too small to be a COFF object", obj.display());
    }
    // COFF File Header: Machine(2) NumSections(2) TimeDateStamp(4)
    // PtrToSymTable(4) NumSymbols(4) SizeOfOptionalHeader(2) Characteristics(2)
    let num_sections = u16::from_le_bytes([bytes[2], bytes[3]]) as usize;
    let opt_header_size = u16::from_le_bytes([bytes[16], bytes[17]]) as usize;
    let sec_table = 20 + opt_header_size;

    for i in 0..num_sections {
        let off = sec_table + i * 40;
        if off + 40 > bytes.len() {
            panic!("section table {i} out of bounds in {}", obj.display());
        }
        // Section name is 8 bytes, null-padded. Long names use a `/N` slot
        // pointing into the string table — not used for `.asmdef`.
        let name_bytes = &bytes[off..off + 8];
        let nul = name_bytes.iter().position(|&b| b == 0).unwrap_or(8);
        let name = std::str::from_utf8(&name_bytes[..nul]).unwrap_or("");
        if name == section_name {
            let raw_size = u32::from_le_bytes(bytes[off + 16..off + 20].try_into().unwrap()) as usize;
            let raw_addr = u32::from_le_bytes(bytes[off + 20..off + 24].try_into().unwrap()) as usize;
            if raw_addr == 0 || raw_size == 0 {
                panic!("section '{section_name}' has no raw data in {}", obj.display());
            }
            if raw_addr + raw_size > bytes.len() {
                panic!(
                    "section '{section_name}' extends past EOF in {}",
                    obj.display()
                );
            }
            return bytes[raw_addr..raw_addr + raw_size].to_vec();
        }
    }
    panic!(
        "section '{section_name}' not found in {} (had {num_sections} sections)",
        obj.display()
    );
}
