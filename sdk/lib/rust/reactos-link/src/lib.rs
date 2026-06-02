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
    /// `win32dll` — user-mode PE DLL (`-shared`), exports via `.def` so
    /// NO `--exclude-all-symbols`, default PE alignment (file 0x200), no
    /// `--gc-sections`. `entry` is the i386-decorated entry symbol —
    /// `___ImageBase` for ntdll.dll (AddressOfEntryPoint = 0) or
    /// `_DllMain@12` for ntdll_vista.dll. `subsystem`: `Some("console:5.01")`
    /// for ntdll.dll (the win32dll set_subsystem hack); `None` lets lld
    /// default (GUI 6.0) — matches ntdll_vista.dll, which passes no
    /// `--subsystem`.
    Win32Dll { entry: String, image_base: u64, subsystem: Option<String> },
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
    rust_features: Vec<String>,
    c_shims: Vec<String>,
    asm: Vec<String>,
    asm_set: AsmDefineSet,
    resource: Option<String>,
    spec: Option<(String, String)>,
    link_stubs: bool,
    static_libs: Vec<String>,
    import_libs: Vec<(String, String, String)>,
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
            rust_features: Vec::new(),
            c_shims: Vec::new(),
            asm: Vec::new(),
            asm_set: AsmDefineSet::Kernel,
            resource: None,
            spec: None,
            link_stubs: false,
            static_libs: Vec::new(),
            import_libs: Vec::new(),
        }
    }

    /// Workspace crate (`crate-type = ["staticlib"]`) that is the module
    /// body. build.rs shells out to build it (custom target json +
    /// build-std nightly) since cargo can't link a staticlib as a dep.
    pub fn rust_staticlib(mut self, crate_name: &str) -> Self {
        self.rust_crate = Some(crate_name.to_string());
        self
    }

    /// Extra cargo `--features` for the `rust_staticlib` build (kept
    /// additive — default features stay on, mirroring cmake's
    /// `cargo build … -p ntdll-rust --features vista`). e.g. ntdll_vista.dll
    /// builds ntdll-rust with `["vista"]` (DllMain/LdrpHeap/ldrnotify).
    pub fn rust_features(mut self, feats: &[&str]) -> Self {
        self.rust_features = feats.iter().map(|s| s.to_string()).collect();
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

    /// Also compile + link the spec2def-generated stub bodies
    /// (`module_stubs.c`, the `-stub` spec entries — e.g. ntdll's 48
    /// `Etw*`/`Rtl*` unimplemented exports). Opt-in: kernel modules don't
    /// link them (their `-stub` symbols resolve elsewhere), but a user-mode
    /// DLL whose `.def` exports them needs the bodies or the link fails
    /// undefined. The stub TU only `#include <stubs.h>` (self-contained:
    /// declares its own types + `DbgPrint`/`RtlRaiseException`), so it
    /// compiles header-free with a single `-I sdk/include/reactos`.
    pub fn spec_stubs(mut self) -> Self {
        self.link_stubs = true;
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
    /// Chainable — call once per imported DLL (e.g. bootvid imports both
    /// `ntoskrnl` and `hal`, mirroring cmake `add_importlibs(...)`).
    pub fn import_lib(mut self, spec_rel: &str, dll_name: &str, basename: &str) -> Self {
        self.import_libs.push((
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
            let mut cmd = Command::new(env::var("CARGO").unwrap_or_else(|_| "cargo".into()));
            cmd.arg("build")
                .arg("--release")
                .arg("--target")
                .arg(&self.target_json)
                .arg("-Z")
                .arg("build-std=core,alloc,compiler_builtins")
                .arg("-Z")
                .arg("build-std-features=compiler-builtins-mem")
                .arg("-p")
                .arg(crate_name);
            if !self.rust_features.is_empty() {
                cmd.arg("--features").arg(self.rust_features.join(","));
            }
            let status = cmd
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

        // ---- spec2def → .def (+ optional -stub bodies) ---------------
        if let Some((spec_rel, dll_name)) = &self.spec {
            let (def, stubs) = self.generate_def(spec_rel, dll_name);
            println!("cargo:rustc-link-arg={}", def.display());
            println!(
                "cargo:rerun-if-changed={}",
                self.repo_root.join(spec_rel).display()
            );
            println!(
                "cargo:rerun-if-changed={}",
                self.repo_root.join("sdk/tools/spec2def/spec2def.c").display()
            );
            if self.link_stubs {
                if !stubs.exists() {
                    panic!("spec_stubs() set but spec2def produced no stubs at {}", stubs.display());
                }
                let mut b = cc::Build::new();
                apply_base(&mut b, &clang, AsmDefineSet::Kernel);
                // module_stubs.c only `#include <stubs.h>` (self-contained).
                b.file(&stubs)
                    .out_dir(self.obj_dir())
                    .include(self.repo_root.join("sdk/include/reactos"))
                    .flag("-std=gnu2x");
                emit_objs(&b, &stubs);
            }
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
        for (spec_rel, dll_name, basename) in &self.import_libs {
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

        // ---- PE link flags (common to every module kind) -------------
        for f in [
            "-nostdlib",
            "-fuse-ld=lld",
            "-Wl,--enable-auto-image-base,--disable-auto-import",
            "-Wl,--major-image-version,5",
            "-Wl,--minor-image-version,01",
            "-Wl,--major-os-version,5",
            "-Wl,--minor-os-version,01",
            "-Wl,--allow-multiple-definition",
            "-Wl,--disable-stdcall-fixup",
        ] {
            println!("cargo:rustc-link-arg={f}");
        }
        // ---- kind-specific subsystem / alignment / sections ----------
        let (entry, image_base) = match &kind {
            ModuleKind::KernelExe { entry, image_base } => {
                // native subsystem; GC unreferenced sections (main.rs anchors
                // the entry to hold the root); no -shared.
                println!("cargo:rustc-link-arg=-Wl,--subsystem,native:5.01");
                println!("cargo:rustc-link-arg=-Wl,--exclude-all-symbols,-file-alignment=0x1000,-section-alignment=0x1000");
                println!("cargo:rustc-link-arg=-Wl,--gc-sections");
                (entry, *image_base)
            }
            ModuleKind::KernelDll { entry, image_base } => {
                // native subsystem PE DLL; keep all sections, 0x1000 align.
                println!("cargo:rustc-link-arg=-Wl,--subsystem,native:5.01");
                println!("cargo:rustc-link-arg=-Wl,--exclude-all-symbols,-file-alignment=0x1000,-section-alignment=0x1000");
                println!("cargo:rustc-link-arg=-shared");
                (entry, *image_base)
            }
            ModuleKind::Win32Dll { entry, image_base, subsystem } => {
                // user-mode PE DLL: exports come from the .def so NO
                // --exclude-all-symbols, default PE alignment (file 0x200),
                // -shared, keep all sections. Subsystem is explicit for
                // ntdll.dll (console:5.01) and lld-default (GUI 6.0) for
                // ntdll_vista.dll, which passes no --subsystem.
                if let Some(s) = subsystem {
                    println!("cargo:rustc-link-arg=-Wl,--subsystem,{s}");
                }
                println!("cargo:rustc-link-arg=-shared");
                (entry, *image_base)
            }
        };
        println!("cargo:rustc-link-arg=-Wl,-e,{entry}");
        println!("cargo:rustc-link-arg=-Wl,--image-base,0x{:08x}", image_base);
    }

    fn obj_dir(&self) -> PathBuf {
        let d = PathBuf::from(env::var("OUT_DIR").unwrap()).join("objs");
        std::fs::create_dir_all(&d).expect("mkdir OUT_DIR/objs");
        d
    }

    /// Returns `(def, stubs)` — the `.def` and the generated `-stub`
    /// bodies TU (`module_stubs.c`). The caller links `def` always and
    /// `stubs` only when `spec_stubs()` opted in.
    fn generate_def(&self, spec_rel: &str, dll_name: &str) -> (PathBuf, PathBuf) {
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
        (def, stubs)
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
