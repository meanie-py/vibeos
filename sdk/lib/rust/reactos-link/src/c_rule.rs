//! C without cmake: the engine compiles a module's leftover C itself.
//!
//! cmake's C rule (the Ninja generator's `C_COMPILER` rule) is
//!
//!   <CC> --target=<triple> <DEFINES> <INCLUDES> <FLAGS> -o <OBJECT> -c <SOURCE>
//!
//! with `<DEFINES>` sorted. What every C compile in the tree shares (the compiler, the
//! standard include run, the core flag run, the global defines) is the committed
//! `cflags/c.<arch>.flags`; the library's own part (`Linker::c_compiled`) is what
//! tools/cargo_cell_gen.py read from cmake's commands, refusing any it could not rebuild
//! token for token. In a cmake build the command is the one cmake ran, from the same
//! directory and through ccache when cmake uses it, so it hits cmake's own cache entries.

use super::{Linker, TracedStatus};
use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) struct CSpec {
    pub(crate) name: String,
    pub(crate) sources: Vec<String>,
    pub(crate) includes: Vec<String>,
    pub(crate) flags: Vec<String>,
    pub(crate) defines: Vec<String>,
}

#[derive(Default)]
struct CTemplate {
    cc: String,
    target: String,
    std: Vec<String>,
    core: Vec<String>,
    defines: Vec<String>,
}

fn parse_template(path: &Path) -> CTemplate {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!(
            "read {}: {e} (regenerate: python3 tools/cargo_cell_gen.py --c-template)",
            path.display()
        )
    });
    let mut t = CTemplate::default();
    let mut section = String::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = name.to_string();
            continue;
        }
        let tok = line.to_string();
        match section.as_str() {
            "cc" => t.cc = tok,
            "target" => t.target = tok,
            "std" => t.std.push(tok),
            "core" => t.core.push(tok),
            "defines" => t.defines.push(tok),
            o => panic!("{}: unknown section [{o}]", path.display()),
        }
    }
    if t.cc.is_empty() || t.std.is_empty() || t.core.is_empty() {
        panic!("{}: incomplete C template", path.display());
    }
    t
}

impl Linker {
    /// `lib<name>.a` from `sources` (repo-relative, or `@build/<rel>` for a generated
    /// one), each compiled with cmake's C rule rebuilt from the template and this part.
    pub(crate) fn compile_c_spec(&self, spec: &CSpec, ar: &Path) -> PathBuf {
        let tmpl_path = self
            .repo_root
            .join(format!("sdk/lib/rust/reactos-link/cflags/c.{}.flags", self.arch.name));
        let t = parse_template(&tmpl_path);
        let root = self.rc_build_root();
        let repo = self.repo_root.to_str().expect("utf8 repo root");
        let build = root.to_str().expect("utf8 build root");
        let sub = |s: &str| s.replace("{REPO}", repo).replace("{BUILD}", build);

        let mut head: Vec<String> = vec![t.target.clone()];
        let mut defines: Vec<String> = t.defines.clone();
        defines.extend(spec.defines.iter().map(|d| format!("-D{d}")));
        defines.sort();
        head.extend(defines);
        for inc in &spec.includes {
            let dirs = if inc == "@std" { t.std.clone() } else { vec![inc.clone()] };
            for d in dirs {
                head.push(format!("-I{}", self.rc_path(&d, &root).display()));
            }
        }
        for f in &spec.flags {
            if f == "@core" {
                head.extend(t.core.iter().map(|x| sub(x)));
            } else {
                head.push(sub(f));
            }
        }

        let out_sub = self.obj_dir().join(format!("cc-{}", spec.name));
        std::fs::create_dir_all(&out_sub).expect("mkdir cc objs");
        let ccache = root == self.build_dir && self.cmake_uses_ccache();
        let mut objs = Vec::new();
        for rel in &spec.sources {
            let src = self.rc_path(rel, &root);
            let stem = src.file_stem().and_then(|s| s.to_str()).expect("source stem");
            let obj = out_sub.join(format!("{stem}.o"));
            if objs.contains(&obj) {
                panic!("{}: two sources compile to {}", spec.name, obj.display());
            }
            let mut c = if ccache {
                let mut c = Command::new("ccache");
                c.arg(sub(&t.cc));
                c
            } else {
                Command::new(sub(&t.cc))
            };
            let status = c
                .args(&head)
                .arg("-o")
                .arg(&obj)
                .arg("-c")
                .arg(&src)
                .current_dir(&root)
                .native_status(&src)
                .unwrap_or_else(|e| panic!("clang for {rel} (lib {}): {e}", spec.name));
            if !status.success() {
                panic!("c_compiled: compile failed for {rel} (lib {})", spec.name);
            }
            println!("cargo:rerun-if-changed={}", src.display());
            objs.push(obj);
        }
        println!("cargo:rerun-if-changed={}", tmpl_path.display());
        let lib_a = self.obj_dir().join(format!("lib{}.a", spec.name));
        let _ = std::fs::remove_file(&lib_a); // ar rcs appends; start fresh
        let status = Command::new(ar)
            .arg("rcs")
            .arg(&lib_a)
            .args(&objs)
            .traced_status()
            .unwrap_or_else(|e| panic!("ar ({}): {e}", ar.display()));
        if !status.success() {
            panic!("ar failed for c_compiled {}", spec.name);
        }
        lib_a
    }
}
