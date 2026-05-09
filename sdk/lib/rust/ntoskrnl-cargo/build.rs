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
        compile_shim(&repo_root, &abs);
        println!("cargo:rerun-if-changed={}", abs.display());
    }
    // io_tables.c retired 2026-05-08; replaced by Rust iomgr/io_tables.rs.
    // The 8 Iop*Operation* tables are now `#[no_mangle] pub static` arrays
    // in kernel-rust, computed from `core::mem::size_of::<>()` against
    // ntkrnl-sys bindgen types.

    // ---- Self-host the hpp host tool + generate xdk kernel headers --------
    // cmake builds 4 kernel headers (ddk/ntddk.h, ddk/ntifs.h, ddk/wdm.h,
    // psdk/ntdef.h) by running the `hpp` host tool on
    // `sdk/include/xdk/*.template.h`. We compile hpp.c with the host
    // compiler (via cc-rs `host()`) and run it ourselves; outputs go to
    // OUT_DIR/xdk-gen/{ddk,psdk}/. After this, the cargo PoC has zero
    // `build-i386/*` dependencies — only the source tree + OUT_DIR.
    let (xdk_ddk_dir, xdk_psdk_dir) = generate_xdk_headers(&repo_root);
    println!("cargo:rerun-if-changed={}", repo_root.join("sdk/tools/hpp/hpp.c").display());

    // ---- Generate bugcodes.h/rc via windmc ---------------------------------
    // cmake produces these by converting bugcodes.mc to UTF-16LE then
    // running windmc on it; we do both steps from build.rs and emit into
    // OUT_DIR/mc-gen/. The .h is consumed by genincdata.c's compile, the
    // .rc by ntoskrnl.rc, and the .bin sidecars by the .rc → .res step.
    let mc_gen_dir = generate_bugcodes_mc(&repo_root);
    println!("cargo:rerun-if-changed={}", repo_root.join("sdk/include/reactos/mc/bugcodes.mc").display());

    // ---- Stage checked-in ks386.inc into OUT_DIR ---------------------------
    // The 8 asm sources all `#include <ks386.inc>` (constants + struct
    // offsets for the i386 trap/context-switch chain). Historically this
    // file was generated at build time by compiling
    // `sdk/include/asm/genincdata.c` and walking its `.asmdef` PE section
    // — that pulled the entire `<ntoskrnl.h>` chain (`<ntndk.h>`,
    // `<arc/arc.h>`, the full ddk/ndk tree) just to compute a static list
    // of offsets. We've moved that regeneration out-of-band to
    // `tools/gen_ks386_inc.py`; the resulting `ks386-i386.inc` is checked
    // in to `data/`. Re-run that script when kernel layouts change
    // (`python3 tools/gen_ks386_inc.py --check` validates the checked-in
    // copy against a fresh regen). Drops 5 `KERNEL_INCLUDES` paths
    // (`sdk/include`, `sdk/include/ddk`, `sdk/include/ndk`,
    // `sdk/include/reactos`, `sdk/lib/pseh/include`) from every cargo build.
    let ks386_inc_dir = stage_ks386_inc(&manifest_dir);
    println!("cargo:rerun-if-changed={}", manifest_dir.join("data/ks386-i386.inc").display());

    // ---- Asm compile (cc-rs) ------------------------------------------------
    // 6 of the 8 historical asm files now live as pre-processed `.s` blobs
    // under `data/asm/` (regenerate via `python3 tools/gen_preprocessed_asm.py`).
    // Each is fully self-contained — every `#include` (`asm.inc`,
    // `ks386.inc`, `asmmacro.S`, `syscalls.inc`, `sysfuncs.h`) is expanded
    // — so cc-rs compiles them with NO kernel `-I` chain (only the
    // `-isystem sdk/include/crt` shim from `apply_kernel_includes_asmonly`).
    // The remaining 2 (`zeropage.S`, `prefetch.S`) became `global_asm!`
    // blocks in `ntoskrnl-rust/src/{ke/i386/zeropage_asm,rtl/prefetch_asm}.rs`
    // (milestone 14).
    let asm_sources: &[&str] = &[
        "sdk/lib/rust/ntoskrnl-cargo/data/asm/zw-i386.s",
        "sdk/lib/rust/ntoskrnl-cargo/data/asm/fastinterlck-i386.s",
        "sdk/lib/rust/ntoskrnl-cargo/data/asm/ioport-i386.s",
        "sdk/lib/rust/ntoskrnl-cargo/data/asm/ctxswitch-i386.s",
        "sdk/lib/rust/ntoskrnl-cargo/data/asm/trap-i386.s",
        "sdk/lib/rust/ntoskrnl-cargo/data/asm/usercall-i386.s",
    ];
    for relpath in asm_sources {
        let abs = repo_root.join(relpath);
        if !abs.exists() {
            panic!("missing pre-processed asm source: {} — regenerate via `python3 tools/gen_preprocessed_asm.py`", abs.display());
        }
        compile_asm(&repo_root, &abs);
        println!("cargo:rerun-if-changed={}", abs.display());
    }
    let _ = ks386_inc_dir;
    // ---- Resource (.res) — checked-in, regen via tools/gen_ntoskrnl_res.py
    // ntoskrnl.rc.res (PE-COFF resource section: VS_VERSION_INFO with 136
    // per-language StringFileInfo blocks + bugcodes RC + bootanim BITMAPs)
    // is regenerated out-of-band by `python3 tools/gen_ntoskrnl_res.py` and
    // checked in to `data/ntoskrnl-i386.rc.res`. Same pattern as
    // `data/ks386-i386.inc` — keeps the kernel header chain
    // (`<winver.h>`, `<ntverp.h>`, `common.ver`) out of every cargo build.
    // Re-run the script when version info or bootanim resources change.
    // Drops 3 source-tree `KERNEL_INCLUDES` paths (`ntoskrnl` root,
    // `sdk/include/psdk`, `sdk/include/vcruntime`) plus the in-process
    // `compile_resource()` helper + 136-entry `LANGUAGES` const.
    let res_out = stage_ntoskrnl_res(&manifest_dir);
    println!("cargo:rustc-link-arg={}", res_out.display());
    println!("cargo:rerun-if-changed={}", manifest_dir.join("data/ntoskrnl-i386.rc.res").display());
    let _ = cmake_obj_dir;

    // ---- Self-host spec2def + generate ntoskrnl.def -----------------------
    // cmake builds spec2def as a host C tool, then runs it on
    // `ntoskrnl/ntoskrnl.spec` to produce `ntoskrnl.def` (drives the PE
    // export table) and `ntoskrnl_stubs.c` (empty for ntoskrnl — the spec
    // has no unimplemented stubs). We compile spec2def with the host cc
    // (same pattern as compile_hpp_host), run it ourselves, and use the
    // resulting `.def` in the link line. Drops the LAST cmake-built file
    // the cargo PoC consumes outside the ~30 sibling static libs.
    let def_file = generate_def_file(&repo_root);
    println!("cargo:rerun-if-changed={}", repo_root.join("ntoskrnl/ntoskrnl.spec").display());
    println!("cargo:rerun-if-changed={}", repo_root.join("sdk/tools/spec2def/spec2def.c").display());
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
///
/// `build-i386/sdk/include/reactos/mc` is intentionally NOT here — that
/// directory holds windmc-generated headers (`bugcodes.h`, `bugcodes.rc`)
/// which we now produce in-process via `generate_bugcodes_mc()` and pass
/// as an extra `-I` to all three consumers.
// `KERNEL_INCLUDES` and `apply_kernel_includes()` are gone — every cargo
// PoC compile input (8 header-free C shims + 6 pre-processed `.s` files +
// the checked-in `.rc.res` resource) is now self-contained and needs at
// most `-isystem sdk/include/crt` plus clang's own resource dir. The
// last source-tree kernel header path (`ntoskrnl/include`) was dropped
// in milestone 16 (2026-05-09) when the remaining 6 asm files moved to
// pre-processed-via-cc-rs.

fn shim_obj_dir() -> PathBuf {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dir = out_dir.join("shims");
    std::fs::create_dir_all(&dir).expect("mkdir OUT_DIR/shims");
    dir
}

/// Compile one C shim with kernel-mode flags + DEFINES, emit it as a `.o`,
/// and add it to the rustc link line. All current shims are header-free —
/// they only need `<stdarg.h>` / `<stddef.h>` / `<stdint.h>` from clang's
/// resource dir.
fn compile_shim(repo_root: &Path, source: &Path) {
    let llvm_mingw = repo_root.join(".build-tools/llvm-mingw");
    let clang = llvm_mingw.join("bin/i686-w64-mingw32-clang");

    let mut build = cc::Build::new();
    apply_kernel_base(&mut build, &clang);
    build
        .file(source)
        .out_dir(shim_obj_dir())
        .flag("-std=gnu2x");

    emit_objects(&build, source);
}

/// Compile one .S/.s file with kernel-mode flags + DEFINES. Asm files all
/// Compile a fully-pre-processed `.s` file with kernel-mode flags. The
/// asm sources under `data/asm/` have all `#include`s expanded out-of-band
/// (`tools/gen_preprocessed_asm.py`), so cc-rs needs no kernel `-I` chain
/// — just the universal `-isystem sdk/include/crt` mingw shim and clang's
/// own resource dir. `-x assembler-with-cpp` keeps `.macro`/`.equ` etc.
/// happy under cc-rs's extension sniffing for lowercase `.s`.
fn compile_asm(repo_root: &Path, source: &Path) {
    let llvm_mingw = repo_root.join(".build-tools/llvm-mingw");
    let clang = llvm_mingw.join("bin/i686-w64-mingw32-clang");

    let mut build = cc::Build::new();
    apply_kernel_base(&mut build, &clang);
    build
        .file(source)
        .out_dir(shim_obj_dir())
        .define("__ASM__", None)
        .flag("-nostdinc")
        .flag("-isystem")
        .flag(repo_root.join("sdk/include/crt").to_str().unwrap())
        .flag("-isystem")
        .flag(llvm_mingw.join("lib/clang/19/include").to_str().unwrap())
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

/// Stage the checked-in `data/ntoskrnl-i386.rc.res` into `OUT_DIR/shims/`
/// for the link line. Out-of-band regen: `python3 tools/gen_ntoskrnl_res.py`
/// runs windres on `ntoskrnl/ntoskrnl.rc` with the kernel `-I` chain (from
/// cmake's `build-i386/sdk/include/`); the byte-identical-to-cmake `.res`
/// is checked in to `data/`. Same pattern as `stage_ks386_inc()`.
fn stage_ntoskrnl_res(manifest_dir: &Path) -> PathBuf {
    let src = manifest_dir.join("data/ntoskrnl-i386.rc.res");
    if !src.exists() {
        panic!(
            "missing {} — regenerate via `python3 tools/gen_ntoskrnl_res.py`",
            src.display()
        );
    }
    let dst = shim_obj_dir().join("ntoskrnl.rc.res");
    std::fs::copy(&src, &dst).unwrap_or_else(|e| panic!("stage ntoskrnl.rc.res: {e}"));
    dst
}

// ---- ks386.inc staging (out-of-band regen via tools/gen_ks386_inc.py) ------

/// Stage the checked-in `data/ks386-i386.inc` into
/// `OUT_DIR/asm-gen/ks386.inc` so the asm sources can pick it up via
/// `-I OUT_DIR/asm-gen` and the canonical name `<ks386.inc>`. Returns the
/// include-path dir.
///
/// Out-of-band regen: `python3 tools/gen_ks386_inc.py` (compiles
/// `sdk/include/asm/genincdata.c` once, walks its `.asmdef` COFF section,
/// emits `data/ks386-i386.inc` byte-identical to cmake's). Re-run that
/// script when kernel struct layouts or the templates
/// (`ks386.template.h` / `ksx.template.h`) change. Day-to-day cargo builds
/// just read the checked-in file — no kernel `-I` chain, no `genincdata.c`
/// compile, no `.obj` parsing.
fn stage_ks386_inc(manifest_dir: &Path) -> PathBuf {
    let src = manifest_dir.join("data/ks386-i386.inc");
    if !src.exists() {
        panic!(
            "missing {} — regenerate via `python3 tools/gen_ks386_inc.py`",
            src.display()
        );
    }
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap()).join("asm-gen");
    std::fs::create_dir_all(&out_dir).expect("mkdir asm-gen");
    let dst = out_dir.join("ks386.inc");
    std::fs::copy(&src, &dst).unwrap_or_else(|e| panic!("stage ks386.inc: {e}"));
    out_dir
}

// ---- bugcodes.h/.rc generation (replaces cmake's mc + utf16le step) --------

/// Run `windmc` on `bugcodes.mc` to produce `bugcodes.h` + `bugcodes.rc` +
/// the per-language `.bin` sidecars in `OUT_DIR/mc-gen/`. Returns the gen
/// dir path so other consumers (`apply_kernel_includes`, `compile_resource`)
/// can `-I` it for `<bugcodes.h>` / `<bugcodes.rc>` resolution.
///
/// `windmc -u` requires UTF-16LE input (no BOM). cmake stages this via the
/// `native-utf16le` host tool; we do the same conversion in pure Rust
/// (bugcodes.mc is ASCII, so each byte zero-extends to one little-endian
/// u16 — verified vs cmake's converted file byte-for-byte during dev).
fn generate_bugcodes_mc(repo_root: &Path) -> PathBuf {
    let llvm_mingw = repo_root.join(".build-tools/llvm-mingw");
    let windmc = llvm_mingw.join("bin/i686-w64-mingw32-windmc");
    if !windmc.exists() {
        // Fall back to PATH (Homebrew mingw / system mingw).
        // Same fallback cmake uses via find_program in toolchain-llvm-mingw.cmake.
    }
    let windmc_exe = if windmc.exists() {
        windmc
    } else {
        PathBuf::from("i686-w64-mingw32-windmc")
    };

    let src = repo_root.join("sdk/include/reactos/mc/bugcodes.mc");
    if !src.exists() {
        panic!("missing bugcodes.mc at {}", src.display());
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let gen_dir = out_dir.join("mc-gen");
    std::fs::create_dir_all(&gen_dir).expect("mkdir mc-gen");

    // ASCII → UTF-16LE conversion (no BOM, matches cmake's `nobom` arg).
    let ascii = std::fs::read(&src).unwrap_or_else(|e| panic!("read {}: {e}", src.display()));
    let mut utf16: Vec<u8> = Vec::with_capacity(ascii.len() * 2);
    for &b in &ascii {
        utf16.push(b);
        utf16.push(0);
    }
    let utf16_path = gen_dir.join("bugcodes.mc");
    std::fs::write(&utf16_path, &utf16).expect("write utf16 bugcodes.mc");

    // Run windmc — same flags as cmake's macro_mc:
    //   windmc -u -A -b -h <gen_dir>/ -r <gen_dir>/ <utf16_input>
    let gen_dir_with_slash = format!("{}/", gen_dir.display());
    let status = Command::new(&windmc_exe)
        .arg("-u") // input is UTF-16LE
        .arg("-A") // ANSI output (matches cmake's ANSI_SOURCE for bugcodes.mc)
        .arg("-b") // .bin filename prefixed with .mc filename for uniqueness
        .arg("-h").arg(&gen_dir_with_slash)
        .arg("-r").arg(&gen_dir_with_slash)
        .arg(&utf16_path)
        .status()
        .unwrap_or_else(|e| panic!("failed to invoke {}: {e}", windmc_exe.display()));
    if !status.success() {
        panic!("windmc failed for bugcodes.mc (exit {:?})", status.code());
    }

    let h = gen_dir.join("bugcodes.h");
    let rc = gen_dir.join("bugcodes.rc");
    if !h.exists() || !rc.exists() {
        panic!(
            "windmc ran but produced no bugcodes.h/.rc in {}",
            gen_dir.display()
        );
    }
    gen_dir
}

// ---- xdk header generation (replaces cmake's hpp host tool + add_xdk_header)

/// Compile `sdk/tools/hpp/hpp.c` with the host compiler and return the
/// path to the resulting `hpp` binary in `OUT_DIR/host-tools/`. cc-rs
/// finds the host C compiler via `cc::Build::host()`; we then drive a
/// raw link step ourselves (`cc::Build::compile()` produces an archive,
/// not an executable).
fn compile_hpp_host(repo_root: &Path) -> PathBuf {
    let host = env::var("HOST").expect("HOST env var");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap()).join("host-tools");
    std::fs::create_dir_all(&out_dir).expect("mkdir host-tools");
    let exe = out_dir.join(if cfg!(windows) { "hpp.exe" } else { "hpp" });

    let cc_tool = cc::Build::new()
        .target(&host)
        .host(&host)
        .opt_level(2)
        .get_compiler();

    let src = repo_root.join("sdk/tools/hpp/hpp.c");
    let mut cmd = Command::new(cc_tool.path());
    // Forward platform-specific flags cc-rs would have added (e.g. arch).
    cmd.args(cc_tool.args());
    cmd.arg("-O2").arg("-o").arg(&exe).arg(&src);
    let status = cmd.status().unwrap_or_else(|e| {
        panic!("failed to invoke host cc ({}): {e}", cc_tool.path().display())
    });
    if !status.success() {
        panic!("host cc failed to compile hpp.c (exit {:?})", status.code());
    }
    if !exe.exists() {
        panic!("hpp build produced no executable at {}", exe.display());
    }
    exe
}

// ---- spec2def host build + ntoskrnl.def generation ------------------------

/// Compile `sdk/tools/spec2def/spec2def.c` with the host C compiler. Returns
/// the path to the resulting host binary in `OUT_DIR/host-tools/`. Mirrors
/// `compile_hpp_host()` exactly (single .c file, no special deps).
fn compile_spec2def_host(repo_root: &Path) -> PathBuf {
    let host = env::var("HOST").expect("HOST env var");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap()).join("host-tools");
    std::fs::create_dir_all(&out_dir).expect("mkdir host-tools");
    let exe = out_dir.join(if cfg!(windows) { "spec2def.exe" } else { "spec2def" });

    let cc_tool = cc::Build::new()
        .target(&host)
        .host(&host)
        .opt_level(2)
        .get_compiler();

    let src = repo_root.join("sdk/tools/spec2def/spec2def.c");
    let mut cmd = Command::new(cc_tool.path());
    cmd.args(cc_tool.args());
    cmd.arg("-O2").arg("-o").arg(&exe).arg(&src);
    let status = cmd.status().unwrap_or_else(|e| {
        panic!("failed to invoke host cc ({}): {e}", cc_tool.path().display())
    });
    if !status.success() {
        panic!("host cc failed to compile spec2def.c (exit {:?})", status.code());
    }
    if !exe.exists() {
        panic!("spec2def build produced no executable at {}", exe.display());
    }
    exe
}

/// Run our self-built `spec2def` on `ntoskrnl/ntoskrnl.spec` to produce
/// `ntoskrnl.def` (export table) and `ntoskrnl_stubs.c` (empty for ntoskrnl
/// — the spec has no `-stub` exports). Returns the path to the .def file
/// for the link line. Mirrors cmake's exact ninja invocation:
///
///   spec2def -n=ntoskrnl.exe -a=i386 -d=...ntoskrnl.def
///            -s=...ntoskrnl_stubs.c --version=0x502 --dbg ntoskrnl.spec
fn generate_def_file(repo_root: &Path) -> PathBuf {
    let spec2def = compile_spec2def_host(repo_root);
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap()).join("spec-gen");
    std::fs::create_dir_all(&out_dir).expect("mkdir spec-gen");

    let spec = repo_root.join("ntoskrnl/ntoskrnl.spec");
    if !spec.exists() {
        panic!("missing ntoskrnl.spec at {}", spec.display());
    }
    let def = out_dir.join("ntoskrnl.def");
    let stubs = out_dir.join("ntoskrnl_stubs.c");

    let status = Command::new(&spec2def)
        .arg("-n=ntoskrnl.exe")
        .arg("-a=i386")
        .arg(format!("-d={}", def.display()))
        .arg(format!("-s={}", stubs.display()))
        .arg("--version=0x502")
        .arg("--dbg")
        .arg(&spec)
        .status()
        .unwrap_or_else(|e| panic!("failed to invoke {}: {e}", spec2def.display()));
    if !status.success() {
        panic!("spec2def failed for ntoskrnl.spec (exit {:?})", status.code());
    }
    if !def.exists() {
        panic!("spec2def ran but produced no .def at {}", def.display());
    }
    def
}

/// Run our self-built `hpp` host tool on the 4 kernel-header templates
/// from `sdk/include/xdk/`, producing the headers cmake puts in
/// `build-i386/sdk/include/{ddk,psdk}/`. Returns `(ddk_dir, psdk_dir)`
/// for `-I` plumbing. hpp resolves `#include`d fragments relative to its
/// CWD, so we run it in `sdk/include/xdk/`.
fn generate_xdk_headers(repo_root: &Path) -> (PathBuf, PathBuf) {
    let hpp = compile_hpp_host(repo_root);
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let xdk_gen = out_dir.join("xdk-gen");
    let ddk_dir = xdk_gen.join("ddk");
    let psdk_dir = xdk_gen.join("psdk");
    std::fs::create_dir_all(&ddk_dir).expect("mkdir xdk-gen/ddk");
    std::fs::create_dir_all(&psdk_dir).expect("mkdir xdk-gen/psdk");

    let xdk_src = repo_root.join("sdk/include/xdk");
    if !xdk_src.exists() {
        panic!("missing sdk/include/xdk source dir: {}", xdk_src.display());
    }

    let runs: &[(&str, &Path, &str)] = &[
        ("wdm.template.h", &ddk_dir, "wdm.h"),
        ("ntddk.template.h", &ddk_dir, "ntddk.h"),
        ("ntifs.template.h", &ddk_dir, "ntifs.h"),
        ("ntdef.template.h", &psdk_dir, "ntdef.h"),
    ];
    for (template, dir, out_name) in runs {
        let out = dir.join(out_name);
        // hpp writes to its CWD if given a bare filename; pass an absolute
        // output path so it lands in OUT_DIR/xdk-gen/.
        let status = Command::new(&hpp)
            .arg(template)
            .arg(&out)
            .current_dir(&xdk_src)
            .status()
            .unwrap_or_else(|e| panic!("failed to invoke {}: {e}", hpp.display()));
        if !status.success() {
            panic!(
                "hpp failed for {template} (exit {:?})",
                status.code()
            );
        }
        if !out.exists() {
            panic!("hpp ran but produced no output at {}", out.display());
        }
        println!("cargo:rerun-if-changed={}", xdk_src.join(template).display());
    }
    // The xdk fragments (ccfuncs.h, cmtypes.h, etc.) all live in xdk_src;
    // any change there should trigger regen. Re-run if any .h in xdk
    // changes.
    println!("cargo:rerun-if-changed={}", xdk_src.display());

    (ddk_dir, psdk_dir)
}
