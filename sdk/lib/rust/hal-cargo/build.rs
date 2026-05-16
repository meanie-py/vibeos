//! Drive the cargo-side link of `hal.dll` (i386 PoC).
//!
//! HAL sibling of `ntoskrnl-cargo/build.rs`, but simpler: HAL has no C
//! shims. cargo owns the `hal-rust` staticlib build, the 4 pre-expanded
//! asm blobs (cc-rs), staging the checked-in `.res`, self-hosting
//! spec2def for `hal.def`, and the final link. cmake still owns the
//! sibling static libs (libcntpr, setjmp, arbiter, fast486, pseh) and
//! the ntoskrnl import lib — run `ninja hal` once first so they exist.
//!
//! Out-of-band inputs (regenerate when their sources change):
//!   - `tools/gen_preprocessed_asm_hal.py` → data/asm/hal-*-i386.s
//!   - `tools/gen_hal_res.py`              → data/hal-i386.rc.res
//!
//! Post-link pefixup / fix_lld_forwarding.py run separately (see
//! build-and-test.sh), same as ntoskrnl-cargo.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let arch = match target_arch.as_str() {
        "x86" => "i386",
        other => panic!("hal-cargo PoC supports i386 only for now (got {other})"),
    };

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    // sdk/lib/rust/hal-cargo → repo root is 4 levels up.
    let repo_root = manifest_dir.ancestors().nth(4).expect("repo root").to_path_buf();
    let build_dir = repo_root.join(format!("build-{arch}"));
    let workspace = repo_root.join("sdk/lib/rust");

    if !build_dir.exists() {
        panic!(
            "cmake build dir {} does not exist — run `cd build-{arch} && ninja hal` first",
            build_dir.display()
        );
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=i686-kernel.json");

    // ---- Build hal-rust as a staticlib ourselves ---------------------------
    // Mirrors ntoskrnl-cargo's kernel-rust build: custom kernel target
    // json + build-std on nightly. Cargo can't consume a `staticlib`
    // crate as a Rust dep, so shell out and emit the .a path.
    let target_json = manifest_dir.join("i686-kernel.json");
    let hal_target_dir = workspace.join("target/hal-cargo-halrust");
    let status = Command::new(env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .arg("build")
        .arg("--release")
        .arg("--target").arg(&target_json)
        .arg("-Z").arg("build-std=core,alloc,compiler_builtins")
        .arg("-Z").arg("build-std-features=compiler-builtins-mem")
        .arg("-p").arg("hal-rust")
        .arg("--manifest-path").arg(workspace.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", &hal_target_dir)
        .env("RUSTUP_TOOLCHAIN", "nightly")
        .status()
        .expect("failed to invoke cargo for hal-rust");
    if !status.success() {
        panic!("cargo build -p hal-rust failed");
    }
    let hal_rust_a = hal_target_dir.join("i686-kernel/release/libhal_rust.a");
    if !hal_rust_a.exists() {
        panic!("hal-rust build produced no libhal_rust.a at {}", hal_rust_a.display());
    }

    // ---- Asm compile (cc-rs) ------------------------------------------------
    // The 4 HAL i386 asm files are pre-expanded (all `#include`s resolved
    // — asm.inc, ks386.inc, asmmacro.S) by tools/gen_preprocessed_asm_hal.py
    // so cc-rs compiles them with NO kernel `-I` chain. `.code16` in
    // v86.S survives — clang's integrated assembler handles it (exactly
    // as cmake's `clang -x assembler-with-cpp` does today).
    let asm_sources: &[&str] = &[
        "sdk/lib/rust/hal-cargo/data/asm/hal-v86-i386.s",
        "sdk/lib/rust/hal-cargo/data/asm/hal-trap-i386.s",
        "sdk/lib/rust/hal-cargo/data/asm/hal-systimer-i386.s",
        "sdk/lib/rust/hal-cargo/data/asm/hal-pic-i386.s",
    ];
    for relpath in asm_sources {
        let abs = repo_root.join(relpath);
        if !abs.exists() {
            panic!(
                "missing pre-processed asm: {} — regenerate via `python3 tools/gen_preprocessed_asm_hal.py`",
                abs.display()
            );
        }
        compile_asm(&repo_root, &abs);
        println!("cargo:rerun-if-changed={}", abs.display());
    }

    // ---- Resource (.res) — checked-in, regen via tools/gen_hal_res.py ------
    let res_out = stage_hal_res(&manifest_dir);
    println!("cargo:rustc-link-arg={}", res_out.display());
    println!("cargo:rerun-if-changed={}", manifest_dir.join("data/hal-i386.rc.res").display());

    // ---- Self-host spec2def + generate hal.def -----------------------------
    let def_file = generate_def_file(&repo_root);
    println!("cargo:rerun-if-changed={}", repo_root.join("hal/hal.spec").display());
    println!("cargo:rerun-if-changed={}", repo_root.join("sdk/tools/spec2def/spec2def.c").display());
    println!("cargo:rustc-link-arg={}", def_file.display());

    // Static libs, cmake link order (from the `hal.dll` ninja link line).
    // libhal_rust.a goes first inside the group so its symbols are visible
    // to the cmake-built libs that reference them; libntoskrnl.a is the
    // ntoskrnl *import* lib (HAL calls kernel exports).
    let static_libs = [
        "sdk/lib/crt/liblibcntpr.a",
        "sdk/lib/vcruntime/libsetjmp.a",
        "sdk/lib/drivers/arbiter/libarbiter.a",
        "sdk/lib/fast486/libfast486.a",
        "sdk/lib/pseh/libpseh.a",
        "ntoskrnl/CMakeFiles/libntoskrnl.dir/libntoskrnl.a",
        "sdk/lib/crt/libchkstk.a",
        "sdk/lib/crt/libmemcmp.a",
    ];
    println!("cargo:rustc-link-arg=-Wl,--start-group");
    println!("cargo:rustc-link-arg={}", hal_rust_a.display());
    for lib in static_libs.iter() {
        let full = build_dir.join(lib);
        if !full.exists() {
            panic!("missing static lib: {} — run `ninja hal` first", full.display());
        }
        println!("cargo:rustc-link-arg={}", full.display());
    }
    println!("cargo:rustc-link-arg=-Wl,--end-group");

    let llvm_mingw = repo_root.join(".build-tools/llvm-mingw");
    let clang_rt = llvm_mingw.join("lib/clang/19/lib/windows/libclang_rt.builtins-i386.a");
    println!("cargo:rustc-link-arg={}", clang_rt.display());

    // Kernel-mode PE-COFF DLL link flags — from cmake's `hal.dll` link
    // line (set_module_type kerneldll ENTRYPOINT HalInitSystem 8).
    // Differs from ntoskrnl: -shared (MODULE/DLL), entry _HalInitSystem@8,
    // image-base 0x00010000.
    let link_flags = [
        "-nostdlib",
        "-fuse-ld=lld",
        "-Wl,--enable-auto-image-base,--disable-auto-import",
        "-Wl,--disable-stdcall-fixup,--allow-multiple-definition",
        "-Wl,--subsystem,native:5.01",
        "-Wl,-entry,_HalInitSystem@8",
        "-Wl,--image-base,0x00010000",
        "-Wl,--major-image-version,5",
        "-Wl,--minor-image-version,01",
        "-Wl,--major-os-version,5",
        "-Wl,--minor-os-version,01",
        "-Wl,--exclude-all-symbols,-file-alignment=0x1000,-section-alignment=0x1000",
        "-shared",
    ];
    for f in link_flags.iter() {
        println!("cargo:rustc-link-arg={f}");
    }
}

/// -D set cmake applies to every HAL asm TU (from the `v86.S.obj` ninja
/// line). HAL flavor: `_NTHALDLL_`/`_NTHAL_`, no `_NTOSKRNL_`/KDBG.
const HAL_ASM_DEFINES: &[(&str, Option<&str>)] = &[
    ("D3D_UMD_INTERFACE_VERSION", Some("0x000C")),
    ("DBG", Some("1")),
    ("DLL_EXPORT_VERSION", Some("0x502")),
    ("DXGKDDI_INTERFACE_VERSION", Some("0x1052")),
    ("MINGW_HAS_SECURE_API", Some("1")),
    ("USE_COMPILER_EXCEPTIONS", None),
    ("WINVER", Some("0x502")),
    ("_CRT_NON_CONFORMING_SWPRINTFS", None),
    ("_CRT_SUPPRESS_RESTRICT", None),
    ("_GLIBCXX_HAVE_BROKEN_VSWPRINTF", None),
    ("_M_IX86", None),
    ("_NEW_DELETE_OPERATORS_", None),
    ("_NTHALDLL_", None),
    ("_NTHAL_", None),
    ("_SEH_ENABLE_TRACE", None),
    ("_SETUPAPI_VER", Some("0x502")),
    ("_USE_32BIT_TIME_T", None),
    ("_USE_DUMMY_PSEH", Some("1")),
    ("_WIN32_IE", Some("0x603")),
    ("_WIN32_WINDOWS", Some("0x502")),
    ("_WIN32_WINNT", Some("0x502")),
    ("_X86_", None),
    ("__REACTOS__", None),
    ("__i386__", None),
    ("i386", None),
];

fn apply_hal_asm_base(build: &mut cc::Build, clang: &Path) {
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
        .flag("-Wno-unknown-warning-option");
    for (name, value) in HAL_ASM_DEFINES {
        build.define(name, *value);
    }
}

fn shim_obj_dir() -> PathBuf {
    let dir = PathBuf::from(env::var("OUT_DIR").unwrap()).join("objs");
    std::fs::create_dir_all(&dir).expect("mkdir OUT_DIR/objs");
    dir
}

/// Compile one fully-pre-expanded `.s` with kernel-mode flags. No kernel
/// `-I` chain — the blobs are self-contained.
fn compile_asm(repo_root: &Path, source: &Path) {
    let llvm_mingw = repo_root.join(".build-tools/llvm-mingw");
    let clang = llvm_mingw.join("bin/i686-w64-mingw32-clang");

    let mut build = cc::Build::new();
    apply_hal_asm_base(&mut build, &clang);
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

    let objs = build.compile_intermediates();
    if objs.is_empty() {
        panic!("cc-rs produced no objects for {}", source.display());
    }
    for obj in objs {
        println!("cargo:rustc-link-arg={}", obj.display());
    }
}

/// Stage the checked-in `data/hal-i386.rc.res` for the link line.
/// Out-of-band regen: `python3 tools/gen_hal_res.py`.
fn stage_hal_res(manifest_dir: &Path) -> PathBuf {
    let src = manifest_dir.join("data/hal-i386.rc.res");
    if !src.exists() {
        panic!(
            "missing {} — regenerate via `python3 tools/gen_hal_res.py`",
            src.display()
        );
    }
    let dst = shim_obj_dir().join("hal.rc.res");
    std::fs::copy(&src, &dst).unwrap_or_else(|e| panic!("stage hal.rc.res: {e}"));
    dst
}

/// Compile `sdk/tools/spec2def/spec2def.c` with the host C compiler.
fn compile_spec2def_host(repo_root: &Path) -> PathBuf {
    let host = env::var("HOST").expect("HOST env var");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap()).join("host-tools");
    std::fs::create_dir_all(&out_dir).expect("mkdir host-tools");
    let exe = out_dir.join(if cfg!(windows) { "spec2def.exe" } else { "spec2def" });

    let cc_tool = cc::Build::new().target(&host).host(&host).opt_level(2).get_compiler();
    let src = repo_root.join("sdk/tools/spec2def/spec2def.c");
    let mut cmd = Command::new(cc_tool.path());
    cmd.args(cc_tool.args());
    cmd.arg("-O2").arg("-o").arg(&exe).arg(&src);
    let status = cmd
        .status()
        .unwrap_or_else(|e| panic!("failed to invoke host cc ({}): {e}", cc_tool.path().display()));
    if !status.success() {
        panic!("host cc failed to compile spec2def.c (exit {:?})", status.code());
    }
    exe
}

/// Run self-built `spec2def` on `hal/hal.spec` → `hal.def`. Mirrors
/// cmake's spec2def invocation (`-n=hal.dll -a=i386 --version=0x502
/// --dbg`). hal.spec has no `-stub` exports, so the stubs.c is empty
/// and not compiled.
fn generate_def_file(repo_root: &Path) -> PathBuf {
    let spec2def = compile_spec2def_host(repo_root);
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap()).join("spec-gen");
    std::fs::create_dir_all(&out_dir).expect("mkdir spec-gen");

    let spec = repo_root.join("hal/hal.spec");
    if !spec.exists() {
        panic!("missing hal.spec at {}", spec.display());
    }
    let def = out_dir.join("hal.def");
    let stubs = out_dir.join("hal_stubs.c");

    let status = Command::new(&spec2def)
        .arg("-n=hal.dll")
        .arg("-a=i386")
        .arg(format!("-d={}", def.display()))
        .arg(format!("-s={}", stubs.display()))
        .arg("--version=0x502")
        .arg("--dbg")
        .arg(&spec)
        .status()
        .unwrap_or_else(|e| panic!("failed to invoke {}: {e}", spec2def.display()));
    if !status.success() {
        panic!("spec2def failed for hal.spec (exit {:?})", status.code());
    }
    if !def.exists() {
        panic!("spec2def ran but produced no .def at {}", def.display());
    }
    def
}
