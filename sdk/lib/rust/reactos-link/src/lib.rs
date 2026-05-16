//! Shared engine for cargo-driven ReactOS PE-module links (i386).
//!
//! Lifts the machinery duplicated across `ntoskrnl-cargo` and
//! `hal-cargo` build.rs files into one builder. A per-module build.rs
//! becomes a manifest:
//!
//! ```ignore
//! reactos_link::Linker::new("i686-kernel.json")
//!     .rust_staticlib("hal-rust")
//!     .asm(&["sdk/lib/rust/hal-cargo/data/asm/hal-v86-i386.s", ..],
//!          reactos_link::AsmDefineSet::Hal)
//!     .resource("data/hal-i386.rc.res")
//!     .spec2def("hal/hal.spec", "hal.dll")
//!     .static_libs(&["sdk/lib/crt/liblibcntpr.a", ..])
//!     .link(reactos_link::ModuleKind::KernelDll {
//!         entry: "_HalInitSystem@8".into(), image_base: 0x0001_0000 });
//! ```
//!
//! Runs inside the consumer's build.rs, so env vars
//! (`CARGO_MANIFEST_DIR`, `OUT_DIR`, `CARGO_PKG_NAME`, `CARGO`, `HOST`,
//! `CARGO_CFG_TARGET_ARCH`) refer to the consumer crate.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Final PE-module flavor. Carries the entry symbol (already i386
/// stdcall-decorated, e.g. `_HalInitSystem@8`) and preferred image base.
pub enum ModuleKind {
    /// `kernelexe` — not `-shared`, `--gc-sections` (e.g. ntoskrnl.exe:
    /// `_KiSystemStartup@4`, 0x00400000).
    KernelExe { entry: String, image_base: u64 },
    /// `kerneldll` — `-shared`, no `--gc-sections` (e.g. hal.dll:
    /// `_HalInitSystem@8`, 0x00010000).
    KernelDll { entry: String, image_base: u64 },
}

/// Which `-D` set the cc-rs asm/C compiles get. Mirrors cmake's
/// per-subsystem DEFINES line.
#[derive(Clone, Copy)]
pub enum AsmDefineSet {
    /// `_NTOSKRNL_`/`_NTSYSTEM_`/`KDBG`/`NTDDI_VERSION` (ntoskrnl).
    Kernel,
    /// `_NTHALDLL_`/`_NTHAL_`, no `KDBG`/`NTDDI` (hal).
    Hal,
}

/// ntoskrnl C/asm DEFINES (cmake's `DEFINES =` line for any ntoskrnl TU).
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

/// HAL asm DEFINES (cmake's line for `hal/halx86/**/*.S.obj`).
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

fn defines(set: AsmDefineSet) -> &'static [(&'static str, Option<&'static str>)] {
    match set {
        AsmDefineSet::Kernel => KERNEL_DEFINES,
        AsmDefineSet::Hal => HAL_ASM_DEFINES,
    }
}

/// Builder for one cargo-driven module link.
pub struct Linker {
    repo_root: PathBuf,
    manifest_dir: PathBuf,
    build_dir: PathBuf,
    pkg_name: String,
    target_json: PathBuf,
    rust_crate: Option<String>,
    c_shims: Vec<String>,
    asm: Vec<String>,
    asm_set: AsmDefineSet,
    resource: Option<String>,
    spec: Option<(String, String)>,
    static_libs: Vec<String>,
    import_lib: Option<(String, String, String)>,
}

impl Linker {
    /// `target_json_rel` is relative to the consumer crate dir
    /// (e.g. `"i686-kernel.json"`). Asserts i386 (only arch supported
    /// today).
    pub fn new(target_json_rel: &str) -> Linker {
        let target_arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
        if target_arch != "x86" {
            panic!("reactos-link supports i386 only for now (got {target_arch})");
        }
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        // sdk/lib/rust/<crate> → repo root is 4 levels up.
        let repo_root = manifest_dir
            .ancestors()
            .nth(4)
            .expect("repo root")
            .to_path_buf();
        let build_dir = repo_root.join("build-i386");
        if !build_dir.exists() {
            panic!(
                "cmake build dir {} does not exist — run `ninja` there first",
                build_dir.display()
            );
        }
        let pkg_name = env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME");
        let target_json = manifest_dir.join(target_json_rel);
        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-changed={}", target_json.display());
        Linker {
            repo_root,
            manifest_dir,
            build_dir,
            pkg_name,
            target_json,
            rust_crate: None,
            c_shims: Vec::new(),
            asm: Vec::new(),
            asm_set: AsmDefineSet::Kernel,
            resource: None,
            spec: None,
            static_libs: Vec::new(),
            import_lib: None,
        }
    }

    /// Workspace crate (`crate-type = ["staticlib"]`) that is the module
    /// body. build.rs shells out to build it (custom target json +
    /// build-std nightly) since cargo can't link a staticlib as a dep.
    pub fn rust_staticlib(mut self, crate_name: &str) -> Self {
        self.rust_crate = Some(crate_name.to_string());
        self
    }

    /// Header-free C shims (repo-relative). Compiled with the Kernel
    /// define set + clang's own stdarg/stddef only.
    pub fn c_shims(mut self, rels: &[&str]) -> Self {
        self.c_shims = rels.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Pre-expanded asm `.s` blobs (repo-relative; all `#include`s
    /// resolved out-of-band). Compiled with `set`'s define list, no
    /// kernel `-I` chain.
    pub fn asm(mut self, rels: &[&str], set: AsmDefineSet) -> Self {
        self.asm = rels.iter().map(|s| s.to_string()).collect();
        self.asm_set = set;
        self
    }

    /// Checked-in `.res` (consumer-crate-relative), regenerated
    /// out-of-band (`tools/gen_*_res.py`).
    pub fn resource(mut self, manifest_rel: &str) -> Self {
        self.resource = Some(manifest_rel.to_string());
        self
    }

    /// `.spec` (repo-relative) → `.def` via self-hosted spec2def.
    /// `dll_name` is the `-n=` arg (e.g. `hal.dll`, `ntoskrnl.exe`).
    pub fn spec2def(mut self, spec_rel: &str, dll_name: &str) -> Self {
        self.spec = Some((spec_rel.to_string(), dll_name.to_string()));
        self
    }

    /// cmake-built sibling static libs (build-i386-relative), in link
    /// order. Wrapped in `--start-group`/`--end-group` with the Rust
    /// staticlib first.
    pub fn static_libs(mut self, builddir_rels: &[&str]) -> Self {
        self.static_libs = builddir_rels.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Generate a PE *import* lib from a `.spec` instead of consuming a
    /// cmake-built one (e.g. HAL importing ntoskrnl). `spec_rel`
    /// repo-relative; `dll_name` the `-n=` arg; `basename` the dlltool
    /// `-t` prefix / the imported DLL's logical name. Self-hosts
    /// spec2def `--implib` then runs `i686-w64-mingw32-dlltool` — no
    /// cmake-built `libNAME.a` needed. Emitted inside the link group.
    pub fn import_lib(mut self, spec_rel: &str, dll_name: &str, basename: &str) -> Self {
        self.import_lib = Some((
            spec_rel.to_string(),
            dll_name.to_string(),
            basename.to_string(),
        ));
        self
    }

    /// Run everything and emit the cargo: link directives.
    pub fn link(self, kind: ModuleKind) {
        let llvm_mingw = self.repo_root.join(".build-tools/llvm-mingw");
        let clang = llvm_mingw.join("bin/i686-w64-mingw32-clang");

        // ---- Rust staticlib (separate target dir, build-std nightly) --
        let rust_a = self.rust_crate.as_ref().map(|crate_name| {
            let lib_file = format!("lib{}.a", crate_name.replace('-', "_"));
            let workspace = self.repo_root.join("sdk/lib/rust");
            let tgt_dir = workspace.join(format!("target/{}-rustlib", self.pkg_name));
            let status = Command::new(env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
                .arg("build")
                .arg("--release")
                .arg("--target")
                .arg(&self.target_json)
                .arg("-Z")
                .arg("build-std=core,alloc,compiler_builtins")
                .arg("-Z")
                .arg("build-std-features=compiler-builtins-mem")
                .arg("-p")
                .arg(crate_name)
                .arg("--manifest-path")
                .arg(workspace.join("Cargo.toml"))
                .env("CARGO_TARGET_DIR", &tgt_dir)
                .env("RUSTUP_TOOLCHAIN", "nightly")
                .status()
                .unwrap_or_else(|e| panic!("cargo build -p {crate_name}: {e}"));
            if !status.success() {
                panic!("cargo build -p {crate_name} failed");
            }
            let a = tgt_dir.join(format!("i686-kernel/release/{lib_file}"));
            if !a.exists() {
                panic!("{crate_name} produced no {lib_file} at {}", a.display());
            }
            a
        });

        // ---- C shims (cc-rs) -----------------------------------------
        for rel in &self.c_shims {
            let abs = self.repo_root.join(rel);
            if !abs.exists() {
                panic!("missing C shim: {}", abs.display());
            }
            let mut b = cc::Build::new();
            apply_base(&mut b, &clang, AsmDefineSet::Kernel);
            b.file(&abs).out_dir(self.obj_dir()).flag("-std=gnu2x");
            emit_objs(&b, &abs);
            println!("cargo:rerun-if-changed={}", abs.display());
        }

        // ---- Pre-expanded asm (cc-rs, no kernel -I) ------------------
        for rel in &self.asm {
            let abs = self.repo_root.join(rel);
            if !abs.exists() {
                panic!(
                    "missing pre-processed asm: {} — regenerate via the matching tools/gen_preprocessed_asm*.py",
                    abs.display()
                );
            }
            let mut b = cc::Build::new();
            apply_base(&mut b, &clang, self.asm_set);
            b.file(&abs)
                .out_dir(self.obj_dir())
                .define("__ASM__", None)
                .flag("-nostdinc")
                .flag("-isystem")
                .flag(self.repo_root.join("sdk/include/crt").to_str().unwrap())
                .flag("-isystem")
                .flag(llvm_mingw.join("lib/clang/19/include").to_str().unwrap())
                .flag("-x")
                .flag("assembler-with-cpp");
            emit_objs(&b, &abs);
            println!("cargo:rerun-if-changed={}", abs.display());
        }

        // ---- Resource (.res) -----------------------------------------
        if let Some(res_rel) = &self.resource {
            let src = self.manifest_dir.join(res_rel);
            if !src.exists() {
                panic!(
                    "missing {} — regenerate via the matching tools/gen_*_res.py",
                    src.display()
                );
            }
            let dst = self.obj_dir().join("module.rc.res");
            std::fs::copy(&src, &dst).unwrap_or_else(|e| panic!("stage .res: {e}"));
            println!("cargo:rustc-link-arg={}", dst.display());
            println!("cargo:rerun-if-changed={}", src.display());
        }

        // ---- spec2def → .def -----------------------------------------
        if let Some((spec_rel, dll_name)) = &self.spec {
            let def = self.generate_def(spec_rel, dll_name);
            println!("cargo:rustc-link-arg={}", def.display());
            println!(
                "cargo:rerun-if-changed={}",
                self.repo_root.join(spec_rel).display()
            );
            println!(
                "cargo:rerun-if-changed={}",
                self.repo_root.join("sdk/tools/spec2def/spec2def.c").display()
            );
        }

        // ---- Static-lib group ----------------------------------------
        println!("cargo:rustc-link-arg=-Wl,--start-group");
        if let Some(a) = &rust_a {
            println!("cargo:rustc-link-arg={}", a.display());
        }
        for lib in &self.static_libs {
            let full = self.build_dir.join(lib);
            if !full.exists() {
                panic!("missing static lib: {} — run ninja first", full.display());
            }
            println!("cargo:rustc-link-arg={}", full.display());
        }
        if let Some((spec_rel, dll_name, basename)) = &self.import_lib {
            let a = self.generate_import_lib(spec_rel, dll_name, basename);
            println!("cargo:rustc-link-arg={}", a.display());
            println!(
                "cargo:rerun-if-changed={}",
                self.repo_root.join(spec_rel).display()
            );
        }
        println!("cargo:rustc-link-arg=-Wl,--end-group");

        let clang_rt =
            llvm_mingw.join("lib/clang/19/lib/windows/libclang_rt.builtins-i386.a");
        println!("cargo:rustc-link-arg={}", clang_rt.display());

        // ---- Kernel-mode PE link flags -------------------------------
        for f in [
            "-nostdlib",
            "-fuse-ld=lld",
            "-Wl,--enable-auto-image-base,--disable-auto-import",
            "-Wl,--subsystem,native:5.01",
            "-Wl,--major-image-version,5",
            "-Wl,--minor-image-version,01",
            "-Wl,--major-os-version,5",
            "-Wl,--minor-os-version,01",
            "-Wl,--exclude-all-symbols,-file-alignment=0x1000,-section-alignment=0x1000",
            "-Wl,--allow-multiple-definition",
        ] {
            println!("cargo:rustc-link-arg={f}");
        }
        let (entry, image_base, exe) = match &kind {
            ModuleKind::KernelExe { entry, image_base } => (entry, *image_base, true),
            ModuleKind::KernelDll { entry, image_base } => (entry, *image_base, false),
        };
        println!("cargo:rustc-link-arg=-Wl,-entry,{entry}");
        println!("cargo:rustc-link-arg=-Wl,--image-base,0x{:08x}", image_base);
        if exe {
            // ntoskrnl.exe: GC unreferenced sections (main.rs anchors
            // the entry to hold the root); no -shared.
            println!("cargo:rustc-link-arg=-Wl,--disable-stdcall-fixup,--gc-sections");
        } else {
            // kerneldll: PE DLL; keep all sections.
            println!("cargo:rustc-link-arg=-Wl,--disable-stdcall-fixup");
            println!("cargo:rustc-link-arg=-shared");
        }
    }

    fn obj_dir(&self) -> PathBuf {
        let d = PathBuf::from(env::var("OUT_DIR").unwrap()).join("objs");
        std::fs::create_dir_all(&d).expect("mkdir OUT_DIR/objs");
        d
    }

    fn generate_def(&self, spec_rel: &str, dll_name: &str) -> PathBuf {
        let spec2def = self.compile_host_tool("sdk/tools/spec2def/spec2def.c", "spec2def");
        let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("spec-gen");
        std::fs::create_dir_all(&out).expect("mkdir spec-gen");
        let spec = self.repo_root.join(spec_rel);
        if !spec.exists() {
            panic!("missing spec {}", spec.display());
        }
        let def = out.join("module.def");
        let stubs = out.join("module_stubs.c");
        let status = Command::new(&spec2def)
            .arg(format!("-n={dll_name}"))
            .arg("-a=i386")
            .arg(format!("-d={}", def.display()))
            .arg(format!("-s={}", stubs.display()))
            .arg("--version=0x502")
            .arg("--dbg")
            .arg(&spec)
            .status()
            .unwrap_or_else(|e| panic!("spec2def: {e}"));
        if !status.success() {
            panic!("spec2def failed for {spec_rel}");
        }
        if !def.exists() {
            panic!("spec2def produced no .def at {}", def.display());
        }
        def
    }

    /// `spec --implib → .def → i686-w64-mingw32-dlltool → libNAME.a`.
    /// Mirrors cmake's `spec2def --implib` + `dlltool -m i386 -d ... -k
    /// -l libNAME.a -t NAME` exactly — produces a PE import lib without
    /// a cmake-built one (the spec carries the export names; the target
    /// DLL need not exist).
    fn generate_import_lib(&self, spec_rel: &str, dll_name: &str, basename: &str) -> PathBuf {
        let spec2def = self.compile_host_tool("sdk/tools/spec2def/spec2def.c", "spec2def");
        let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("implib-gen");
        std::fs::create_dir_all(&out).expect("mkdir implib-gen");
        let spec = self.repo_root.join(spec_rel);
        if !spec.exists() {
            panic!("missing spec {}", spec.display());
        }
        let implib_def = out.join(format!("{basename}_implib.def"));
        let status = Command::new(&spec2def)
            .arg("--version=0x502")
            .arg("--dbg")
            .arg(format!("-n={dll_name}"))
            .arg("-a=i386")
            .arg("--implib")
            .arg(format!("-d={}", implib_def.display()))
            .arg(&spec)
            .status()
            .unwrap_or_else(|e| panic!("spec2def --implib: {e}"));
        if !status.success() {
            panic!("spec2def --implib failed for {spec_rel}");
        }
        let dlltool = self
            .repo_root
            .join(".build-tools/llvm-mingw/bin/i686-w64-mingw32-dlltool");
        let lib_a = out.join(format!("lib{basename}.a"));
        let status = Command::new(&dlltool)
            .arg("-m")
            .arg("i386")
            .arg("-d")
            .arg(&implib_def)
            .arg("-k")
            .arg("-l")
            .arg(&lib_a)
            .arg("-t")
            .arg(basename)
            .status()
            .unwrap_or_else(|e| panic!("dlltool ({}): {e}", dlltool.display()));
        if !status.success() {
            panic!("dlltool failed for {basename}");
        }
        if !lib_a.exists() {
            panic!("dlltool produced no {}", lib_a.display());
        }
        lib_a
    }

    /// Compile a single-.c host tool with the host cc, return its path.
    fn compile_host_tool(&self, src_rel: &str, name: &str) -> PathBuf {
        let host = env::var("HOST").expect("HOST");
        let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("host-tools");
        std::fs::create_dir_all(&out).expect("mkdir host-tools");
        let exe = out.join(if cfg!(windows) {
            format!("{name}.exe")
        } else {
            name.to_string()
        });
        let cc_tool = cc::Build::new()
            .target(&host)
            .host(&host)
            .opt_level(2)
            .get_compiler();
        let src = self.repo_root.join(src_rel);
        let mut cmd = Command::new(cc_tool.path());
        cmd.args(cc_tool.args());
        cmd.arg("-O2").arg("-o").arg(&exe).arg(&src);
        let status = cmd
            .status()
            .unwrap_or_else(|e| panic!("host cc ({}): {e}", cc_tool.path().display()));
        if !status.success() {
            panic!("host cc failed to compile {src_rel}");
        }
        exe
    }
}

/// Common kernel-mode clang flags + the chosen DEFINES (cmake's
/// `FLAGS =`/`DEFINES =` lines for any ntoskrnl/HAL TU).
fn apply_base(build: &mut cc::Build, clang: &Path, set: AsmDefineSet) {
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
    for (n, v) in defines(set) {
        build.define(n, *v);
    }
}

fn emit_objs(build: &cc::Build, source: &Path) {
    let objs = build.compile_intermediates();
    if objs.is_empty() {
        panic!("cc-rs produced no objects for {}", source.display());
    }
    for obj in objs {
        println!("cargo:rustc-link-arg={}", obj.display());
    }
}
