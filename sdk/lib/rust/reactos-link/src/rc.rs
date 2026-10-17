//! Resources without cmake: the engine compiles a module's `.rc` itself.
//!
//! cmake's windres rule (`CMAKE_RC_COMPILE_OBJECT`, sdk/cmake/gcc.cmake) is
//!
//!   windres -O coff --codepage=65001 <INCLUDES> <FLAGS>
//!           -DRC_INVOKED -D__WIN32__=1 -D__FLAT__=1 ${I18N_DEFS} <DEFINES> <SOURCE> <OBJECT>
//!
//! where `<DEFINES>` comes out sorted. What every module shares (the windres binary, the
//! standard include run, `CMAKE_RC_FLAGS`, the RC_INVOKED/LANGUAGE_* block, the global
//! defines) is the committed `cflags/rc.<arch>.flags`; what the module adds is in its
//! build.rs (`Linker::rc`). tools/cargo_cell_gen.py writes both from cmake's own
//! commands and refuses a command it cannot rebuild token for token, so the `.res` is
//! byte-identical to the one cmake compiled.

use super::{Linker, TracedStatus};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) struct RcSpec {
    pub(crate) rc_rel: String,
    pub(crate) includes: Vec<String>,
    pub(crate) flags: Vec<String>,
    pub(crate) defines: Vec<String>,
}

/// `cflags/rc.<arch>.flags`: `[section]` headers, one token per line, `{REPO}` for the
/// repo root and `@build/` / repo-relative paths in `[std]` as in a module's includes.
#[derive(Default)]
struct RcTemplate {
    windres: String,
    std: Vec<String>,
    flags: Vec<String>,
    defs: Vec<String>,
    defines: Vec<String>,
}

fn parse_template(path: &Path) -> RcTemplate {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!(
            "read {}: {e} (regenerate: python3 tools/cargo_cell_gen.py --rc-template)",
            path.display()
        )
    });
    let mut t = RcTemplate::default();
    let mut section = String::new();
    for line in text.lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = name.to_string();
            continue;
        }
        let tok = line.to_string();
        match section.as_str() {
            "windres" => t.windres = tok,
            "std" => t.std.push(tok),
            "flags" => t.flags.push(tok),
            "defs" => t.defs.push(tok),
            "defines" => t.defines.push(tok),
            o => panic!("{}: unknown section [{o}]", path.display()),
        }
    }
    if t.windres.is_empty() || t.std.is_empty() || t.defs.is_empty() {
        panic!("{}: incomplete rc template", path.display());
    }
    t
}

impl Linker {
    /// The tree `@build/` names: `build-<arch>` when cmake configured one (and it is
    /// not forced off), else a per-crate tree the engine generates.
    fn rc_build_root(&self) -> PathBuf {
        if !super::cmake_off_forced() && self.build_dir.join("build.ninja").exists() {
            return self.build_dir.clone();
        }
        let root = PathBuf::from(env::var("OUT_DIR").unwrap()).join("cmake-off-build");
        // winnt.h (and the other xdk headers) are native-hpp output, regenerated here.
        self.generate_xdk_headers_into(&root.join("sdk/include"));
        // version.h / buildno.h are CONFIGURE-time in cmake (sdk/include/reactos/
        // version.cmake: the date, `git describe`), so they come from the one configure
        // step the orchestrator runs, not from every build script at its own moment.
        let gen = env::var_os("VIBEOS_GEN_DIR").map(PathBuf::from).unwrap_or_else(|| {
            panic!(
                "resources without a cmake build need VIBEOS_GEN_DIR (version.h/buildno.h): \
                 build through tools/cargo_orchestrate.py, which configures it"
            )
        });
        for rel in ["sdk/include/reactos/version.h", "sdk/include/reactos/buildno.h"] {
            let src = gen.join(rel);
            let dst = root.join(rel);
            std::fs::create_dir_all(dst.parent().unwrap()).expect("mkdir cmake-off-build");
            std::fs::copy(&src, &dst)
                .unwrap_or_else(|e| panic!("stage {} from VIBEOS_GEN_DIR: {e}", src.display()));
            println!("cargo:rerun-if-changed={}", src.display());
        }
        println!("cargo:rerun-if-env-changed=VIBEOS_GEN_DIR");
        root
    }

    fn rc_path(&self, tok: &str, build_root: &Path) -> PathBuf {
        match tok.strip_prefix("@build/") {
            Some(rel) => build_root.join(rel),
            None if tok == "@build" => build_root.to_path_buf(),
            None => self.repo_root.join(tok),
        }
    }

    pub(crate) fn compile_rc(&self, spec: &RcSpec) -> PathBuf {
        let tmpl_path = self
            .repo_root
            .join(format!("sdk/lib/rust/reactos-link/cflags/rc.{}.flags", self.arch.name));
        let t = parse_template(&tmpl_path);
        let repo = self.repo_root.to_str().expect("utf8 repo root");
        let sub = |s: &str| s.replace("{REPO}", repo);
        let root = self.rc_build_root();
        let rc_abs = self.repo_root.join(&spec.rc_rel);
        let dst = self.obj_dir().join("module.rc.res");

        let mut args: Vec<String> = vec!["-O".into(), "coff".into(), "--codepage=65001".into()];
        for inc in &spec.includes {
            let dirs = if inc == "@std" { t.std.clone() } else { vec![inc.clone()] };
            for d in dirs {
                args.push("-I".into());
                args.push(self.rc_path(&d, &root).to_str().expect("utf8 include").to_string());
            }
        }
        args.extend(t.flags.iter().map(|f| sub(f)));
        args.extend(spec.flags.iter().map(|f| sub(f)));
        args.extend(t.defs.iter().cloned());
        let mut defines: Vec<String> = t.defines.clone();
        defines.extend(spec.defines.iter().map(|d| format!("-D{d}")));
        defines.sort();
        args.extend(defines);
        args.push(rc_abs.to_str().expect("utf8 rc").to_string());
        args.push(dst.to_str().expect("utf8 res").to_string());

        let status = Command::new(sub(&t.windres))
            .args(&args)
            .current_dir(&root)
            .traced_status()
            .unwrap_or_else(|e| panic!("windres for {}: {e}", spec.rc_rel));
        if !status.success() {
            panic!("windres failed for {}", spec.rc_rel);
        }

        // What the .rc reads: its #includes and the icons, bitmaps and language files
        // its statements name sit beside it, so its directory tree is the dependency.
        println!("cargo:rerun-if-changed={}", tmpl_path.display());
        watch_tree(rc_abs.parent().expect("rc dir"));
        // ...and what the build generates for it into the module's own build directories
        // (wscript's typelib, an mc step's .rc): the generated inputs, not the module's
        // outputs, which would rerun every build.
        for inc in spec.includes.iter().filter(|i| i.starts_with("@build/")) {
            watch_generated(&self.rc_path(inc, &root));
        }
        dst
    }
}

const GENERATED_RC_INPUTS: &[&str] = &["tlb", "rc", "h", "bin", "mc", "rgs", "manifest"];

fn watch_generated(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        let ext = p.extension().and_then(|x| x.to_str()).unwrap_or("");
        if p.is_file() && GENERATED_RC_INPUTS.contains(&ext) {
            println!("cargo:rerun-if-changed={}", p.display());
        }
    }
}

fn watch_tree(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        match e.file_type() {
            Ok(ft) if ft.is_dir() => watch_tree(&p),
            Ok(ft) if ft.is_file() => println!("cargo:rerun-if-changed={}", p.display()),
            _ => {}
        }
    }
}
