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
    let io_tables = repo_root.join("ntoskrnl/io/iomgr/io_tables.c");
    if !io_tables.exists() {
        panic!("missing shim source: {}", io_tables.display());
    }
    compile_shim(&repo_root, &io_tables, /*kernel_includes=*/ true);
    println!("cargo:rerun-if-changed={}", io_tables.display());

    // ---- Asm objects (still cmake-owned) ------------------------------------
    let asm_objs = [
        "ex/zw.S.obj",
        "ex/i386/fastinterlck_asm.S.obj",
        "ex/i386/ioport.S.obj",
        "ke/i386/ctxswitch.S.obj",
        "ke/i386/trap.s.obj",
        "ke/i386/usercall_asm.S.obj",
        "ke/i386/zeropage.S.obj",
        "rtl/i386/prefetch.S.obj",
    ];
    for o in asm_objs.iter() {
        let full = cmake_obj_dir.join(o);
        if !full.exists() {
            panic!("missing input: {}\n(run ninja in build-{arch} first)", full.display());
        }
        println!("cargo:rustc-link-arg={}", full.display());
    }
    let res_full = cmake_obj_dir.join("ntoskrnl.rc.res");
    if !res_full.exists() {
        panic!("missing resource file: {}", res_full.display());
    }
    println!("cargo:rustc-link-arg={}", res_full.display());

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

/// Compile one C shim with the same kernel-mode flags + DEFINES cmake uses,
/// emit it as a `.o`, and add it to the rustc link line. Set
/// `kernel_includes` true for the one shim (`io_tables.c`) that still needs
/// `<ntoskrnl.h>` + `internal/io_i.h`; false (no `-I`) for the rest, which
/// only need `<stdarg.h>`/`<stddef.h>`/`<stdint.h>` from clang's own
/// resource directory.
fn compile_shim(repo_root: &Path, source: &Path, kernel_includes: bool) {
    let llvm_mingw = repo_root.join(".build-tools/llvm-mingw");
    let clang = llvm_mingw.join("bin/i686-w64-mingw32-clang");

    // Per-file output path under OUT_DIR. cc-rs would auto-name, but we use
    // its raw `.compile_intermediates()` mode below for finer control.
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("shim");
    let obj_dir = out_dir.join("shims");
    std::fs::create_dir_all(&obj_dir).expect("mkdir OUT_DIR/shims");

    let mut build = cc::Build::new();
    build
        .compiler(&clang)
        .file(source)
        .out_dir(&obj_dir)
        .warnings(false)
        .extra_warnings(false)
        // Match cmake's C compile flags exactly. The set below mirrors the
        // `FLAGS = …` line ninja prints for any ntoskrnl C source.
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
        .flag("-std=gnu2x")
        .flag("-Wno-unknown-warning-option")
        // Kernel-mode + ReactOS-build DEFINES. Mirrors the `DEFINES = …`
        // line cmake emits.
        .define("D3D_UMD_INTERFACE_VERSION", "0x000C")
        .define("DBG", "1")
        .define("DLL_EXPORT_VERSION", "0x502")
        .define("DXGKDDI_INTERFACE_VERSION", "0x1052")
        .define("KDBG", None)
        .define("MINGW_HAS_SECURE_API", "1")
        .define("NTDDI_VERSION", "0x05020400")
        .define("NTKRNLVISTA", None)
        .define("USE_COMPILER_EXCEPTIONS", None)
        .define("WINVER", "0x502")
        .define("_CRT_NON_CONFORMING_SWPRINTFS", None)
        .define("_CRT_SUPPRESS_RESTRICT", None)
        .define("_GLIBCXX_HAVE_BROKEN_VSWPRINTF", None)
        .define("_M_IX86", None)
        .define("_NEW_DELETE_OPERATORS_", None)
        .define("_NTOSKRNL_", None)
        .define("_NTSYSTEM_", None)
        .define("_SEH_ENABLE_TRACE", None)
        .define("_SETUPAPI_VER", "0x502")
        .define("_USE_32BIT_TIME_T", None)
        .define("_USE_DUMMY_PSEH", "1")
        .define("_WIN32_IE", "0x603")
        .define("_WIN32_WINDOWS", "0x502")
        .define("_WIN32_WINNT", "0x502")
        .define("_X86_", None)
        .define("__REACTOS__", None)
        .define("__ROS_ROSSYM__", None)
        .define("__i386__", None)
        .define("i386", None);

    if kernel_includes {
        // Mirror cmake's `INCLUDES = …` for ntoskrnl C sources. Only the one
        // remaining shim that drags `<ntoskrnl.h>` in (`io_tables.c`) needs
        // these. The other 8 shims are header-free and don't.
        let build_dir = repo_root.join("build-i386");
        for inc in [
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
        ] {
            build.include(repo_root.join(inc));
        }
        // Required to suppress the user-mode CRT search path. Matches the
        // `-nostdinc -isystem<crt> -isystem<clang>` triplet cmake passes.
        build
            .flag("-nostdinc")
            .flag("-isystem")
            .flag(repo_root.join("sdk/include/crt").to_str().unwrap())
            .flag("-isystem")
            .flag(
                llvm_mingw
                    .join("lib/clang/19/include")
                    .to_str()
                    .unwrap(),
            );
        // Sanity-check the build dir exists when this code path runs.
        assert!(
            build_dir.exists(),
            "kernel_includes path needs build-i386/ to exist for generated headers"
        );
    }

    // Drive cc-rs ourselves: cc::Build::compile() emits a static archive,
    // but we want raw .o files we can pass on the rustc link line.
    let objs = build.compile_intermediates();
    if objs.is_empty() {
        panic!("cc-rs produced no objects for {}", source.display());
    }
    let _ = stem; // currently unused but kept for future per-shim diagnostics
    for obj in objs {
        println!("cargo:rustc-link-arg={}", obj.display());
    }
}
