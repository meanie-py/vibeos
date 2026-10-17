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
    if t.cc.is_empty() || t.core.is_empty() {
        panic!("{}: incomplete C template", path.display());
    }
    t
}

/// One source of a shared library's recipe (`clibs/<arch>/<target>.recipe`).
struct RecipeLine {
    asm: bool,
    src: String,
    includes: Vec<String>,
    flags: Vec<String>,
    defines: Vec<String>,
}

fn parse_recipe(path: &Path) -> Vec<RecipeLine> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!(
            "read {}: {e} (regenerate: python3 tools/cargo_cell_gen.py --clib-recipes)",
            path.display()
        )
    });
    let words = |s: &str| s.split(' ').filter(|w| !w.is_empty()).map(String::from).collect();
    text.lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            if f.len() != 5 || !matches!(f[0], "c" | "asm") {
                panic!("{}: malformed recipe line {l:?}", path.display());
            }
            RecipeLine {
                asm: f[0] == "asm",
                src: f[1].to_string(),
                includes: words(f[2]),
                flags: words(f[3]),
                defines: words(f[4]),
            }
        })
        .collect()
}

impl Linker {
    fn template(&self, kind: &str) -> (PathBuf, CTemplate) {
        let path = self
            .repo_root
            .join(format!("sdk/lib/rust/reactos-link/cflags/{kind}.{}.flags", self.arch.name));
        let t = parse_template(&path);
        (path, t)
    }

    fn subst(&self, s: &str, root: &Path) -> String {
        s.replace("{REPO}", self.repo_root.to_str().expect("utf8 repo root"))
            .replace("{BUILD}", root.to_str().expect("utf8 build root"))
    }

    /// cmake's C rule after the compiler: `--target <sorted defines> <includes> <flags>`.
    fn c_head(
        &self,
        t: &CTemplate,
        includes: &[String],
        flags: &[String],
        defines: &[String],
        root: &Path,
    ) -> Vec<String> {
        let mut head: Vec<String> = vec![t.target.clone()];
        let mut all: Vec<String> = t.defines.clone();
        all.extend(defines.iter().map(|d| format!("-D{d}")));
        all.sort();
        head.extend(all);
        head.extend(self.c_includes(t, includes, root));
        for f in flags {
            if f == "@core" {
                head.extend(t.core.iter().map(|x| self.subst(x, root)));
            } else {
                head.push(self.subst(f, root));
            }
        }
        head
    }

    fn c_includes(&self, t: &CTemplate, includes: &[String], root: &Path) -> Vec<String> {
        let mut out = Vec::new();
        for inc in includes {
            let dirs = if inc == "@std" { t.std.clone() } else { vec![inc.clone()] };
            for d in dirs {
                out.push(format!("-I{}", self.rc_path(&d, root).display()));
            }
        }
        out
    }

    /// cmake's ASM rule after `-o <obj>`: `<includes> <flags> <sorted defines> -D__ASM__`,
    /// the defines where the flags' `@defines` marker puts them (else last).
    fn asm_tail(&self, t: &CTemplate, line: &RecipeLine, root: &Path) -> Vec<String> {
        let mut out = self.c_includes(t, &line.includes, root);
        let mut all: Vec<String> = t.defines.clone();
        all.extend(line.defines.iter().map(|d| format!("-D{d}")));
        all.sort();
        let mut flags = line.flags.clone();
        if !flags.iter().any(|f| f == "@defines") {
            flags.push("@defines".into());
        }
        for f in &flags {
            match f.as_str() {
                "@core" => out.extend(t.core.iter().map(|x| self.subst(x, root))),
                "@defines" => out.extend(all.iter().cloned()),
                _ => out.push(self.subst(f, root)),
            }
        }
        out.push("-D__ASM__".into());
        out
    }

    /// `lib<target>.a` from the committed recipe of a shared cmake C library, for a build
    /// with no cmake: what `replay_target` replays from compile_commands.json otherwise.
    pub(crate) fn compile_recipe(&self, target: &str, ar: &Path) -> PathBuf {
        let rpath = self.repo_root.join(format!(
            "sdk/lib/rust/reactos-link/clibs/{}/{target}.recipe",
            self.arch.name
        ));
        let lines = parse_recipe(&rpath);
        let (cpath, ct) = self.template("c");
        let (apath, at) = self.template("asm");
        let root = self.rc_build_root();
        // An asm source reading the generated ks*.inc (`@build/sdk/include/asm`), with no
        // cmake build to have made it: the engine's self-hosted geninc output, put where
        // the command looks, so the command stays cmake's. Cells share the tree and write
        // the same bytes, so it goes in by rename.
        let ks_dir = root.join("sdk/include/asm");
        if root != self.build_dir
            && lines.iter().any(|l| l.asm && l.includes.iter().any(|i| i == "@build/sdk/include/asm"))
            && !ks_dir.join(self.ks_inc_name()).exists()
        {
            let made = self.generate_ks_inc().join(self.ks_inc_name());
            std::fs::create_dir_all(&ks_dir).expect("mkdir generated asm include dir");
            let tmp = ks_dir.join(format!("{}.{}.tmp", self.ks_inc_name(), std::process::id()));
            std::fs::copy(&made, &tmp).expect("stage ks inc");
            std::fs::rename(&tmp, ks_dir.join(self.ks_inc_name())).expect("place ks inc");
        }
        let out_sub = self.obj_dir().join(format!("recipe-{target}"));
        std::fs::create_dir_all(&out_sub).expect("mkdir recipe objs");
        let mut objs = Vec::new();
        for line in &lines {
            let src = self.rc_path(&line.src, &root);
            let name = src.file_name().and_then(|s| s.to_str()).expect("source name");
            let obj = out_sub.join(format!("{name}.o"));
            if objs.contains(&obj) {
                panic!("{target}: two sources compile to {}", obj.display());
            }
            let mut c = if line.asm {
                let mut c = Command::new(self.subst(&at.cc, &root));
                c.args(["-x", "assembler-with-cpp", "-o"]).arg(&obj);
                c.args(self.asm_tail(&at, line, &root));
                c
            } else {
                let mut c = Command::new(self.subst(&ct.cc, &root));
                c.args(self.c_head(&ct, &line.includes, &line.flags, &line.defines, &root));
                c.arg("-o").arg(&obj);
                c
            };
            let status = c
                .arg("-c")
                .arg(&src)
                .current_dir(&root)
                .native_status(&src)
                .unwrap_or_else(|e| panic!("compile {} (engine:{target}): {e}", line.src));
            if !status.success() {
                panic!("engine:{target}: compile failed for {}", line.src);
            }
            println!("cargo:rerun-if-changed={}", src.display());
            objs.push(obj);
        }
        for p in [&rpath, &cpath, &apath] {
            println!("cargo:rerun-if-changed={}", p.display());
        }
        let lib_a = self.obj_dir().join(format!("lib{target}.a"));
        let _ = std::fs::remove_file(&lib_a);
        let status = Command::new(ar)
            .arg("rcs")
            .arg(&lib_a)
            .args(&objs)
            .traced_status()
            .unwrap_or_else(|e| panic!("ar ({}): {e}", ar.display()));
        if !status.success() {
            panic!("ar failed for engine:{target}");
        }
        lib_a
    }

    /// `lib<name>.a` from `sources` (repo-relative, or `@build/<rel>` for a generated
    /// one), each compiled with cmake's C rule rebuilt from the template and this part.
    pub(crate) fn compile_c_spec(&self, spec: &CSpec, ar: &Path) -> PathBuf {
        let (tmpl_path, t) = self.template("c");
        let root = self.rc_build_root();
        let sub = |s: &str| self.subst(s, &root);

        let head = self.c_head(&t, &spec.includes, &spec.flags, &spec.defines, &root);

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
