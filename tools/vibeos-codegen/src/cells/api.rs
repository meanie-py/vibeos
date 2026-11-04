//! JSON bridge for pure Cargo cell operations; writer operations are owned by the CLI.
use super::{
    graph, mutate, pe, proof, recipes, render, rewrite, CFlags, CTemplate, Cell, Error, Link,
    RcFlags, RcTemplate, Result,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

fn val<T: Serialize>(v: T) -> Result<Value> {
    Ok(serde_json::to_value(v)?)
}
struct Args<'a> {
    op: &'a str,
    args: &'a [Value],
    kwargs: &'a serde_json::Map<String, Value>,
}
impl<'a> Args<'a> {
    fn new(req: &'a Value) -> Result<Self> {
        let obj = req
            .as_object()
            .ok_or_else(|| Error::new("request must be an object"))?;
        let op = obj
            .get("op")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::new("request op must be a string"))?;
        let args = match obj.get("args") {
            None => &[][..],
            Some(v) => v
                .as_array()
                .ok_or_else(|| Error::new("args must be an array"))?,
        };
        let kwargs = match obj.get("kwargs") {
            None => {
                static EMPTY: std::sync::OnceLock<serde_json::Map<String, Value>> =
                    std::sync::OnceLock::new();
                EMPTY.get_or_init(Default::default)
            }
            Some(v) => v
                .as_object()
                .ok_or_else(|| Error::new("kwargs must be an object"))?,
        };
        Ok(Self { op, args, kwargs })
    }
    fn bind(&self, names: &[&str]) -> Result<()> {
        if self.args.len() > names.len() {
            return Err(Error::new(format!(
                "{}: too many positional arguments",
                self.op
            )));
        }
        for key in self.kwargs.keys() {
            let Some(i) = names.iter().position(|n| *n == key) else {
                return Err(Error::new(format!(
                    "{}: unexpected keyword argument '{key}'",
                    self.op
                )));
            };
            if i < self.args.len() {
                return Err(Error::new(format!(
                    "{}: multiple values for argument '{key}'",
                    self.op
                )));
            }
        }
        Ok(())
    }
    fn get(&self, names: &[&str], name: &str) -> Option<&'a Value> {
        let i = names.iter().position(|n| *n == name)?;
        self.args.get(i).or_else(|| self.kwargs.get(name))
    }
    fn req<T: DeserializeOwned>(&self, names: &[&str], name: &str) -> Result<T> {
        let v = self.get(names, name).ok_or_else(|| {
            Error::new(format!("{}: missing required argument '{name}'", self.op))
        })?;
        serde_json::from_value(v.clone())
            .map_err(|e| Error::new(format!("{}: invalid {name}: {e}", self.op)))
    }
    fn opt<T: DeserializeOwned>(&self, names: &[&str], name: &str, default: T) -> Result<T> {
        match self.get(names, name) {
            None => Ok(default),
            Some(v) => serde_json::from_value(v.clone())
                .map_err(|e| Error::new(format!("{}: invalid {name}: {e}", self.op))),
        }
    }
    fn repo(&self, names: &[&str], root: &Path) -> Result<PathBuf> {
        let path: Option<PathBuf> = self.opt(names, "repo", None)?;
        if let Some(path) = path {
            let selected = root.canonicalize()?;
            let given = path.canonicalize()?;
            if selected != given {
                return Err(Error::new(format!(
                    "{}: repo differs from selected root",
                    self.op
                )));
            }
        }
        Ok(root.to_owned())
    }
}
macro_rules! bind {($a:expr,$($n:literal),* $(,)?)=>{{let names=&[$($n),*];$a.bind(names)?;names}}}
fn plan(p: mutate::Plan) -> Value {
    json!({"edits":p.edits,"changed":p.changed,"skipped":p.skipped,"provenance":p.provenance})
}

pub fn dispatch(root: &Path, request: &Value) -> Result<Value> {
    let a = Args::new(request)?;
    match a.op {
        "_read" => {
            let n = bind!(a, "path");
            val(super::read(&a.req::<PathBuf>(n, "path")?)?)
        }
        "_ccdb" => {
            let n = bind!(a, "path");
            let text = super::read(&a.req::<PathBuf>(n, "path")?)?;
            let parsed: Value = serde_json::from_str(&text)?;
            if !parsed.is_array() {
                return Err(Error::new("compile database must be an array"));
            }
            Ok(parsed)
        }
        "_source_list" => {
            let n = bind!(a, "text");
            let mut out = Vec::new();
            for (_, _, cmd, raw) in graph::statements(&a.req::<String>(n, "text")?) {
                let args = graph::args(&raw);
                if cmd == "list" && args.len() >= 2 && args[0] == "APPEND" && args[1] == "SOURCE" {
                    out.extend_from_slice(&args[2..]);
                }
            }
            val(out)
        }
        "_cd_for" => {
            let n = bind!(a, "a");
            let v = a.req::<Vec<String>>(n, "a")?;
            val(v
                .iter()
                .position(|x| x == "FOR")
                .map(|i| v[i + 1..].to_vec())
                .unwrap_or_default())
        }
        "_drop_target" => {
            let n = bind!(a, "a", "target");
            let mut v = a.req::<Vec<String>>(n, "a")?;
            let target = a.req::<String>(n, "target")?;
            let i = v
                .iter()
                .position(|x| x == "TARGET")
                .ok_or_else(|| Error::new(format!("{target}: add_cd_file has no TARGET")))?;
            if v.get(i + 1) != Some(&target) {
                return Err(Error::new(format!(
                    "{target}: add_cd_file names TARGET {}",
                    v.get(i + 1).map(String::as_str).unwrap_or("")
                )));
            }
            v.drain(i..i + 2);
            val(v)
        }
        "_c_commands" | "_asm_commands" => {
            let n = bind!(a, "arch", "repo");
            let arch = a.req::<String>(n, "arch")?;
            let root = a.repo(n, root)?;
            let mut rows = super::compile_db(&root, &arch)?;
            rows.retain(|e| {
                let f = e["file"].as_str().unwrap_or("");
                let cmd = e["command"].as_str().unwrap_or("");
                if a.op == "_c_commands" {
                    f.ends_with(".c") && cmd.contains(" --target=")
                } else {
                    (f.ends_with(".s") || f.ends_with(".S"))
                        && cmd.contains(" -x assembler-with-cpp ")
                }
            });
            val(rows)
        }
        "_why_by_arch" => {
            let n = bind!(a, "miss");
            let miss = a.req::<BTreeMap<String, Value>>(n, "miss")?;
            let mut order: Vec<&str> = super::ARCHES
                .into_iter()
                .filter(|a| miss.contains_key(*a))
                .collect();
            order.extend(
                miss.keys()
                    .map(String::as_str)
                    .filter(|a| !super::ARCHES.contains(a)),
            );
            let mut out = Vec::new();
            for arch in order {
                let why = miss[arch]
                    .get("why")
                    .and_then(Value::as_str)
                    .ok_or_else(|| Error::new("_why_by_arch: missing why"))?;
                let template = super::constants()["_NO_EDGE_TEXT"]
                    .get(why)
                    .and_then(Value::as_str)
                    .ok_or_else(|| Error::new("_why_by_arch: invalid why"))?;
                out.push(format!("{arch}: {}", template.replace("{arch}", arch)));
            }
            val(out.join(", "))
        }
        "_ssp_objects" | "_subsystem_only" | "_require_defined_only" | "_names_only" => {
            let n = bind!(a, "a");
            let x = a.req::<Vec<String>>(n, "a")?;
            let re = |p: &str| regex::Regex::new(p).expect("constant regex");
            let value = match a.op {
                "_ssp_objects" => {
                    x.len() == 3
                        && x[1] == "PRIVATE"
                        && re(r"^\$<TARGET_OBJECTS:gcc_ssp_\w+>$").is_match(&x[2])
                }
                "_names_only" => {
                    x.len() >= 4
                        && x[1] == "PROPERTIES"
                        && x.len() % 2 == 0
                        && x[2..].iter().step_by(2).all(|v| {
                            super::constants()["NAME_PROPERTIES"]
                                .as_array()
                                .is_some_and(|a| a.iter().any(|x| x.as_str() == Some(v)))
                        })
                }
                _ => {
                    let opts: Vec<_> = x
                        .iter()
                        .skip(1)
                        .filter(|v| *v != "PRIVATE" && *v != "PUBLIC")
                        .collect();
                    let p = if a.op == "_subsystem_only" {
                        r"^-Wl,--subsystem,[\w:.]+$"
                    } else {
                        r"^(?:LINKER:|-Wl,)--require-defined=[^,\s]+(?:,--require-defined=[^,\s]+)*$"
                    };
                    !opts.is_empty() && opts.iter().all(|v| re(p).is_match(v.trim_matches('"')))
                }
            };
            val(value)
        }
        "_retarget" => {
            let n = bind!(a, "cmd", "raw", "new");
            let cmd = a.req::<String>(n, "cmd")?;
            let raw = a.req::<String>(n, "raw")?;
            let new = a.req::<String>(n, "new")?;
            let first = graph::args(&raw)
                .into_iter()
                .next()
                .ok_or_else(|| Error::new("_retarget: empty args"))?;
            val(format!("{cmd}({})", raw.trim().replacen(&first, &new, 1)))
        }
        "_engine_deps" => {
            let n = bind!(a, "name");
            let name = a.req::<String>(n, "name")?;
            if let Some(v) = super::constants()["ENGINE_LIB_DEPS"]
                .get(&name)
                .and_then(Value::as_str)
            {
                return val(v.split_whitespace().collect::<Vec<_>>());
            }
            if let Some(target) = name.strip_prefix("engine:c:") {
                let index = graph::library_index(root)?;
                return val(graph::engine_lib(&format!("lib{target}.a"), &index)
                    .map(|x| x.1)
                    .unwrap_or_default());
            }
            val(Vec::<String>::new())
        }
        "txtsetup_files" => {
            let n = bind!(a, "repo");
            let root = a.repo(n, root)?;
            let path = super::local(&root, &root.join("boot/bootdata/txtsetup.sif"))?;
            let text = super::read(&path)?;
            let re = regex::Regex::new(r"(?m)^([A-Za-z0-9_.-]+)[ \t]*=")?;
            val(re
                .captures_iter(&text)
                .map(|c| c[1].to_ascii_lowercase())
                .collect::<std::collections::BTreeSet<_>>())
        }
        "_one_list" => {
            let n = bind!(a, "arch", "stem", "repo");
            let arch = a.req::<String>(n, "arch")?;
            let stem = a.req::<String>(n, "stem")?;
            let root = a.repo(n, root)?;
            let dir = super::local(&root, &root.join(format!("build-{arch}/boot")))?;
            let mut files = Vec::new();
            for entry in std::fs::read_dir(dir)? {
                let p = entry?.path();
                let name = p.file_name().and_then(|v| v.to_str()).unwrap_or("");
                if name.starts_with(&format!("{stem}."))
                    && name.ends_with(".lst")
                    && name != format!("{stem}.cmake.lst")
                    && name != format!("{stem}.stripped.lst")
                {
                    files.push(super::local(&root, &p)?);
                }
            }
            files.sort();
            if files.len() > 1 {
                return Err(Error::new(format!(
                    "build-{arch}/boot: several {stem}.<config>.lst: {files:?}"
                )));
            }
            val(files.into_iter().next())
        }
        "link_edge" => {
            let n = bind!(a, "arch", "target", "repo");
            val(graph::link_edge(
                &a.repo(n, root)?,
                &a.req::<String>(n, "arch")?,
                &a.req::<String>(n, "target")?,
            )?)
        }
        "_link_edges" => {
            let n = bind!(a, "target", "repo");
            val(graph::link_edges(
                &a.repo(n, root)?,
                &a.req::<String>(n, "target")?,
            )?)
        }
        "_no_edge_why" => {
            let n = bind!(a, "ninja", "target");
            val(graph::no_edge_why(
                &a.req::<String>(n, "ninja")?,
                &a.req::<String>(n, "target")?,
            ))
        }
        "spec_index" => {
            let n = bind!(a, "repo");
            val(graph::spec_index(&a.repo(n, root)?)?)
        }
        "cd_dirs" | "cd_placement" => {
            let n = bind!(a, "arch", "output", "repo");
            let r = a.repo(n, root)?;
            let arch = a.req::<String>(n, "arch")?;
            let output = a.req::<String>(n, "output")?;
            if a.op == "cd_dirs" {
                val(graph::cd_dirs(&r, &arch, &output)?)
            } else {
                val(graph::cd_placement(&r, &arch, &output)?)
            }
        }
        "source_dir" => {
            let n = bind!(a, "arch", "output", "target", "repo");
            val(graph::source_dir(
                &a.repo(n, root)?,
                &a.req::<String>(n, "arch")?,
                &a.req::<String>(n, "output")?,
                &a.req::<String>(n, "target")?,
            )?)
        }
        "model" => {
            let n = bind!(a, "target", "repo");
            val(graph::model(
                &a.repo(n, root)?,
                &a.req::<String>(n, "target")?,
            )?)
        }
        "cmake_facts" => {
            let n = bind!(a, "cmake", "target", "libs");
            let libs: Option<Vec<String>> = a.opt(n, "libs", None)?;
            val(graph::cmake_facts(
                &a.req::<PathBuf>(n, "cmake")?,
                &a.req::<String>(n, "target")?,
                libs.as_deref().unwrap_or(&[]),
            )?)
        }
        "rewrite_cmake" => {
            let n = bind!(a, "cmake", "cell");
            val(rewrite::rewrite_cmake(
                &a.req::<PathBuf>(n, "cmake")?,
                &a.req::<Cell>(n, "cell")?,
            )?)
        }
        "engine_lib" => {
            let n = bind!(a, "lib", "repo");
            let index = graph::library_index(&a.repo(n, root)?)?;
            val(graph::engine_lib(&a.req::<String>(n, "lib")?, &index))
        }
        "_library_index" => {
            let n = bind!(a, "repo");
            val(graph::library_index(&a.repo(n, root)?)?)
        }
        "_obj_source" => {
            let n = bind!(a, "cmake_dir", "obj");
            val(graph::obj_source(
                &a.req::<PathBuf>(n, "cmake_dir")?,
                &a.req::<String>(n, "obj")?,
            ))
        }
        "_shim_name" => {
            let n = bind!(a, "c", "cell");
            val(graph::shim_name(
                root,
                &a.req::<String>(n, "c")?,
                &a.req::<Cell>(n, "cell")?,
            ))
        }
        "_target_file_refs" => {
            let n = bind!(a, "repo");
            val(graph::target_file_refs(&a.repo(n, root)?)?)
        }
        "_list_vars" => {
            let n = bind!(a, "text");
            val(graph::list_vars(&a.req::<String>(n, "text")?))
        }
        "_expand" => {
            let n = bind!(a, "items", "lists");
            val(graph::expand(
                &a.req::<Vec<String>>(n, "items")?,
                &a.req::<BTreeMap<String, Vec<String>>>(n, "lists")?,
            )?)
        }
        "missing_cell_recipes" => {
            let n = bind!(a, "targets", "repo");
            val(super::driver::missing_cell_recipes(
                &a.repo(n, root)?,
                &a.req::<Vec<String>>(n, "targets")?,
            )?)
        }
        "_statements" => {
            let n = bind!(a, "text");
            val(graph::statements(&a.req::<String>(n, "text")?))
        }
        "_args" => {
            let n = bind!(a, "s");
            val(graph::args(&a.req::<String>(n, "s")?))
        }
        "_branches" => {
            let n = bind!(a, "stmts");
            val(graph::branches(
                &a.req::<Vec<graph::Statement>>(n, "stmts")?,
            ))
        }
        "_exclusive" => {
            let n = bind!(a, "p", "q");
            val(graph::exclusive(
                &a.req::<Vec<(usize, usize)>>(n, "p")?,
                &a.req::<Vec<(usize, usize)>>(n, "q")?,
            ))
        }
        "_included_functions" => {
            let n = bind!(a, "cmake", "text");
            val(graph::included_functions(
                &a.req::<PathBuf>(n, "cmake")?,
                &a.req::<String>(n, "text")?,
            )?)
        }
        "_enclosing_ifs" => {
            let n = bind!(a, "text", "target");
            val(graph::enclosing_ifs(
                &a.req::<String>(n, "text")?,
                &a.req::<String>(n, "target")?,
            )?)
        }
        "_custom_body" => {
            let n = bind!(a, "text");
            val(graph::custom_body(&a.req::<String>(n, "text")?))
        }
        "_body_env" => {
            let n = bind!(a, "text", "extras");
            val(graph::body_env(
                &a.req::<String>(n, "text")?,
                &a.req::<Vec<String>>(n, "extras")?,
            ))
        }
        "_arch_values" => {
            let n = bind!(a, "text", "var");
            val(graph::arch_values(
                &a.req::<String>(n, "text")?,
                &a.req::<String>(n, "var")?,
            ))
        }
        "_inlinable_source" => {
            let n = bind!(a, "text", "cmake_dir");
            val(graph::inlinable_source(
                &a.req::<String>(n, "text")?,
                &a.req::<PathBuf>(n, "cmake_dir")?,
            )?)
        }
        "_entry_symbol" => {
            let n = bind!(a, "arch", "sym", "stack");
            let stack: Option<String> = a.opt(n, "stack", None)?;
            val(graph::entry_symbol(
                &a.req::<String>(n, "arch")?,
                &a.req::<String>(n, "sym")?,
                stack.as_deref(),
            ))
        }
        "_subsystem_flag" => {
            let n = bind!(a, "sub");
            val(graph::subsystem_flag(&a.req::<String>(n, "sub")?))
        }
        "_links_delayimp" => {
            let n = bind!(a, "static");
            val(graph::links_delayimp(&a.req::<Vec<String>>(n, "static")?))
        }
        "_placeholder" => {
            let n = bind!(a, "path");
            val(graph::placeholder(&a.req::<PathBuf>(n, "path")?)?)
        }
        "_placeholder_archive" => {
            let n = bind!(a, "arch", "lib", "repo");
            val(graph::placeholder_archive(
                &a.repo(n, root)?,
                &a.req::<String>(n, "arch")?,
                &a.req::<String>(n, "lib")?,
            )?)
        }
        "_spec_imported_exports" => {
            let n = bind!(a, "spec");
            val(graph::spec_imported_exports(&a.req::<PathBuf>(n, "spec")?)?)
        }
        "_check_statement_effects" => {
            let n = bind!(a, "target", "facts", "links");
            graph::check_statement_effects(
                &a.req::<String>(n, "target")?,
                &a.req::<Value>(n, "facts")?,
                &a.req::<BTreeMap<String, Link>>(n, "links")?,
            )?;
            Ok(Value::Null)
        }
        "_cmakelists" => {
            let n = bind!(a, "repo");
            val(graph::cmakelists(&a.repo(n, root)?)?)
        }
        "_call_span" => {
            let n = bind!(a, "text", "start");
            val(graph::call_span(
                &a.req::<String>(n, "text")?,
                a.req::<usize>(n, "start")?,
            )?)
        }
        "_rc_parts" | "_c_parts" | "_asm_parts" => {
            let n = bind!(a, "cmd");
            let cmd = a.req::<String>(n, "cmd")?;
            match a.op {
                "_rc_parts" => val(recipes::rc_parts(&cmd)?),
                "_c_parts" => val(recipes::c_parts(&cmd)?),
                _ => val(recipes::asm_parts(&cmd)?),
            }
        }
        "_rc_enc" | "_rc_dec" | "_flag_enc" | "_flag_dec" => {
            let n = if a.op == "_rc_enc" {
                bind!(a, "path", "arch")
            } else {
                bind!(a, "tok", "arch")
            };
            let tok = a.req::<String>(n, n[0])?;
            let arch = a.req::<String>(n, "arch")?;
            match a.op {
                "_rc_enc" => val(recipes::rc_enc(&tok, &arch)?),
                "_rc_dec" => val(recipes::rc_dec(&tok, &arch)),
                "_flag_enc" => val(recipes::flag_enc(&tok, &arch)),
                _ => val(recipes::flag_dec(&tok, &arch)),
            }
        }
        "_majority_run" => {
            let n = bind!(a, "lists", "share");
            let lists = a.req::<Vec<Vec<String>>>(n, "lists")?;
            let share = a.opt::<f64>(n, "share", 0.95)?;
            val(recipes::majority_run(&lists, share))
        }
        "_with_marker" => {
            let n = bind!(a, "seq", "run", "marker");
            val(recipes::with_marker(
                &a.req::<Vec<String>>(n, "seq")?,
                &a.req::<Vec<String>>(n, "run")?,
                &a.req::<String>(n, "marker")?,
            ))
        }
        "rc_decode" => {
            let n = bind!(a, "f", "tmpl", "arch", "src");
            val(recipes::rc_decode(
                &a.req::<RcFlags>(n, "f")?,
                &a.req::<RcTemplate>(n, "tmpl")?,
                &a.req::<String>(n, "arch")?,
                &a.req::<String>(n, "src")?,
            ))
        }
        "rc_encode" => {
            let n = bind!(a, "cmd", "tmpl", "arch");
            val(recipes::rc_encode(
                &a.req::<String>(n, "cmd")?,
                &a.req::<RcTemplate>(n, "tmpl")?,
                &a.req::<String>(n, "arch")?,
            )?)
        }
        "c_decode" => {
            let n = bind!(a, "f", "tmpl", "arch");
            val(recipes::c_decode(
                &a.req::<CFlags>(n, "f")?,
                &a.req::<CTemplate>(n, "tmpl")?,
                &a.req::<String>(n, "arch")?,
            ))
        }
        "c_encode" => {
            let n = bind!(a, "cmd", "tmpl", "arch");
            val(recipes::c_encode(
                &a.req::<String>(n, "cmd")?,
                &a.req::<CTemplate>(n, "tmpl")?,
                &a.req::<String>(n, "arch")?,
            )?)
        }
        "asm_decode" => {
            let n = bind!(a, "f", "tmpl", "arch", "obj");
            val(recipes::asm_decode(
                &a.req::<CFlags>(n, "f")?,
                &a.req::<CTemplate>(n, "tmpl")?,
                &a.req::<String>(n, "arch")?,
                &a.req::<String>(n, "obj")?,
            ))
        }
        "asm_encode" => {
            let n = bind!(a, "cmd", "tmpl", "arch");
            val(recipes::asm_encode(
                &a.req::<String>(n, "cmd")?,
                &a.req::<CTemplate>(n, "tmpl")?,
                &a.req::<String>(n, "arch")?,
            )?)
        }
        "_rc_command" => {
            let n = bind!(a, "arch", "rc_obj", "repo");
            val(recipes::rc_command(
                &a.req::<String>(n, "arch")?,
                &a.req::<String>(n, "rc_obj")?,
                &a.repo(n, root)?,
            )?)
        }
        "rc_template_path"
        | "c_template_path"
        | "asm_template_path"
        | "read_rc_template"
        | "read_c_template"
        | "read_asm_template"
        | "derive_rc_template"
        | "derive_c_template"
        | "derive_asm_template" => {
            let n = bind!(a, "arch", "repo");
            let arch = a.req::<String>(n, "arch")?;
            let root = a.repo(n, root)?;
            match a.op {
                "rc_template_path" => val(recipes::rc_template_path(&arch, &root)),
                "c_template_path" => val(recipes::c_template_path(&arch, &root)),
                "asm_template_path" => val(recipes::asm_template_path(&arch, &root)),
                "read_rc_template" => val(recipes::read_rc_template(&arch, &root)?),
                "read_c_template" => val(recipes::read_c_template(&arch, &root)?),
                "read_asm_template" => val(recipes::read_asm_template(&arch, &root)?),
                "derive_rc_template" => val(recipes::derive_rc_template(&arch, &root)?),
                "derive_c_template" => val(recipes::derive_c_template(&arch, &root)?),
                _ => val(recipes::derive_asm_template(&arch, &root)?),
            }
        }
        "rc_flags_for" => {
            let n = bind!(a, "rc_rel", "marker", "repo");
            val(recipes::rc_flags_for(
                &a.req::<String>(n, "rc_rel")?,
                &a.req::<String>(n, "marker")?,
                &a.repo(n, root)?,
            )?)
        }
        "c_lib_for" => {
            let n = bind!(a, "target", "repo");
            val(recipes::c_lib_for(
                &a.req::<String>(n, "target")?,
                &a.repo(n, root)?,
            )?)
        }
        "clib_recipe" | "clib_recipe_path" => {
            let n = bind!(a, "arch", "target", "repo");
            let arch = a.req::<String>(n, "arch")?;
            let target = a.req::<String>(n, "target")?;
            let repo = a.repo(n, root)?;
            if a.op == "clib_recipe" {
                val(recipes::clib_recipe(&arch, &target, &repo)?)
            } else {
                val(recipes::clib_recipe_path(&arch, &target, &repo))
            }
        }
        "engine_c_targets" => {
            let n = bind!(a, "repo");
            val(recipes::engine_c_targets(&a.repo(n, root)?)?)
        }
        "_rc_commands" => {
            let n = bind!(a, "arch", "repo");
            val(recipes::rc_commands(
                &a.repo(n, root)?,
                &a.req::<String>(n, "arch")?,
            )?)
        }
        "build_rs" | "cargo_toml" | "main_rs" | "_panic_deps" | "_user_panic_deps" => {
            let n = bind!(a, "cell");
            let cell = a.req::<Cell>(n, "cell")?;
            render::validate_cell(&cell, a.op == "build_rs")?;
            match a.op {
                "build_rs" => val(render::build_rs(&cell)?),
                "cargo_toml" => val(render::cargo_toml(&cell)?),
                "main_rs" => val(render::main_rs(&cell)?),
                "_panic_deps" => val(render::panic_deps(&cell)),
                _ => val(render::user_panic_deps(&cell)),
            }
        }
        "_rs_list" => {
            let n = bind!(a, "items", "indent");
            val(render::rs_list(
                &a.req::<Vec<String>>(n, "items")?,
                a.req::<usize>(n, "indent")?,
            ))
        }
        "_cfg" => {
            let n = bind!(a, "arches");
            let arches = a.req::<Vec<String>>(n, "arches")?;
            if arches
                .iter()
                .any(|arch| !super::ARCHES.contains(&arch.as_str()))
            {
                return Err(Error::new("_cfg: unsupported architecture"));
            }
            val(render::cfg(
                &arches.iter().map(String::as_str).collect::<Vec<_>>(),
            ))
        }
        "rc_binding" => {
            let n = bind!(a, "flags");
            let flags = a.req::<BTreeMap<String, RcFlags>>(n, "flags")?;
            for arch in super::ARCHES {
                if !flags.contains_key(arch) {
                    return Err(Error::new(format!("rc_binding: missing {arch}")));
                }
            }
            val(render::rc_binding(&flags))
        }
        "c_binding" => {
            let n = bind!(a, "srcs", "flags");
            let srcs = a.req::<BTreeMap<String, Vec<String>>>(n, "srcs")?;
            let flags = a.req::<BTreeMap<String, CFlags>>(n, "flags")?;
            for arch in super::ARCHES {
                if !flags.contains_key(arch) || !srcs.contains_key(arch) {
                    return Err(Error::new(format!("c_binding: missing {arch}")));
                }
            }
            val(render::c_binding(&srcs, &flags))
        }
        "rc_call" => {
            let n = bind!(a, "rc");
            val(render::rc_call(&a.req::<String>(n, "rc")?))
        }
        "constants" => {
            a.bind(&[])?;
            let mut c = super::constants().clone();
            let obj = c
                .as_object_mut()
                .ok_or_else(|| Error::new("invalid constants"))?;
            obj.insert("ARCHES".into(), json!(super::ARCHES));
            obj.insert("MAIN_RS_HEAD".into(), json!(render::MAIN_RS_HEAD));
            obj.insert(
                "_NOT_STATIC".into(),
                json!([
                    "SHARED",
                    "MODULE",
                    "OBJECT",
                    "INTERFACE",
                    "IMPORTED",
                    "ALIAS"
                ]),
            );
            obj.insert(
                "_SPEC_IMPORT_ENTRY".into(),
                json!(r"^\s*(?:\d+|@)\s+\w+\s+(?:-\S+\s+)*?-import\b(?:\s+-\S+)*\s+(\w+)"),
            );
            obj.insert(
                "_SPEC_SELF_TARGET".into(),
                json!(r"^\s*(?:\d+|@)\s+\w+\s+(?:-\S+\s+)*(\w+)\([^)]*\)\s+(\w+)\s*$"),
            );
            obj.insert(
                "RES_CALL".into(),
                json!(r#"\.resource_from_rc\("([^"]+)",\s*"([^"]+)"\)"#),
            );
            obj.insert(
                "SHIMS_CALL".into(),
                json!(r#"\.c_from_compile_db\(\s*"(\w+)""#),
            );
            obj.insert("REPO".into(), json!(root));
            obj.insert("RUST".into(), json!(root.join("sdk/lib/rust")));
            Ok(c)
        }
        "rc_template_text" => {
            let n = bind!(a, "template", "arch");
            val(a
                .req::<RcTemplate>(n, "template")?
                .text(&a.req::<String>(n, "arch")?))
        }
        "c_template_text" => {
            let n = bind!(a, "template", "arch");
            val(a
                .req::<CTemplate>(n, "template")?
                .text(&a.req::<String>(n, "arch")?))
        }
        "_export_is_import_thunk" => {
            let n = bind!(a, "code", "machine");
            val(pe::export_is_import_thunk(
                &a.req::<Vec<u8>>(n, "code")?,
                a.req::<u16>(n, "machine")?,
            ))
        }
        "_exports_bound_to_imports" | "carries_panic_handler" => {
            let n = bind!(a, "pe_path");
            let p = a.req::<PathBuf>(n, "pe_path")?;
            if a.op == "carries_panic_handler" {
                val(pe::carries_panic_handler(&p)?)
            } else {
                val(pe::exports_bound_to_imports(&p)?)
            }
        }
        "_rc_object_depends" => {
            let n = bind!(a, "cmd", "raw");
            val(mutate::rc_object_depends(
                &a.req::<String>(n, "cmd")?,
                &a.req::<String>(n, "raw")?,
            ))
        }
        "_rc_flags_by_analogy" => {
            let n = bind!(a, "rc_rel", "repo");
            val(mutate::rc_flags_by_analogy(&a.req::<String>(n, "rc_rel")?, &a.repo(n, root)?)?.0)
        }
        "plan_remap" | "plan_own_c" | "plan_own_rc" => {
            let n = bind!(a, "crates", "repo");
            let crates: Option<Vec<String>> = a.opt(n, "crates", None)?;
            let repo = a.repo(n, root)?;
            Ok(plan(match a.op {
                "plan_remap" => mutate::remap(&repo, crates.as_deref())?,
                "plan_own_c" => mutate::own_c(&repo, crates.as_deref())?,
                _ => mutate::own_rc(&repo, crates.as_deref())?,
            }))
        }
        "plan_drop_shims_in" => {
            let n = bind!(a, "text", "cmake", "target", "generated");
            val(mutate::drop_shims_in(
                &a.req::<String>(n, "text")?,
                &a.req::<PathBuf>(n, "cmake")?,
                &a.req::<String>(n, "target")?,
                &a.req::<Vec<String>>(n, "generated")?,
            )?)
        }
        "plan_drop_res_in" => {
            let n = bind!(a, "text", "cmake", "target");
            val(mutate::drop_res_in(
                &a.req::<String>(n, "text")?,
                &a.req::<PathBuf>(n, "cmake")?,
                &a.req::<String>(n, "target")?,
            )?)
        }
        "cell_crate" => {
            let n = bind!(a, "target", "repo");
            val(proof::cell_crate(
                &a.repo(n, root)?,
                &a.req::<String>(n, "target")?,
            )?)
        }
        "_cell_is_bodyless" => {
            let n = bind!(a, "target", "repo");
            val(proof::cell_is_bodyless(
                &a.repo(n, root)?,
                &a.req::<String>(n, "target")?,
            )?)
        }
        "baseline_reusable" => {
            let n = bind!(a, "targets", "baseline");
            val(proof::baseline_reusable(
                &a.req::<Vec<String>>(n, "targets")?,
                &a.req::<PathBuf>(n, "baseline")?,
            )?)
        }
        "_cell_uses_panic_common" => {
            let n = bind!(a, "target", "arch", "repo");
            val(proof::cell_uses_panic_common(
                &a.repo(n, root)?,
                &a.req::<String>(n, "target")?,
                &a.req::<String>(n, "arch")?,
            )?)
        }
        "_baseline_pe" => {
            let n = bind!(a, "baseline", "arch", "target");
            val(proof::baseline_pe(
                &a.req::<PathBuf>(n, "baseline")?,
                &a.req::<String>(n, "arch")?,
                &a.req::<String>(n, "target")?,
            )?)
        }
        "plan_reshell" => {
            let n = bind!(a, "targets", "baseline", "repo");
            let p = proof::reshell(
                &a.repo(n, root)?,
                &a.req::<Vec<String>>(n, "targets")?,
                &a.req::<PathBuf>(n, "baseline")?,
            )?;
            Ok(json!({"edits":p.edits,"notes":p.notes}))
        }
        "plan_save_baseline" => {
            let n = bind!(a, "targets", "baseline", "repo");
            let p = proof::save_baseline(
                &a.repo(n, root)?,
                &a.req::<Vec<String>>(n, "targets")?,
                &a.req::<PathBuf>(n, "baseline")?,
            )?;
            Ok(
                json!({"copies":p.copies.into_iter().map(|c|json!({"source":c.source,"destination":c.destination})).collect::<Vec<_>>(),"index_path":p.index_path,"index_text":p.index_text}),
            )
        }
        "plan_check" => {
            let n = bind!(a, "targets", "baseline", "repo");
            let p = proof::check(
                &a.repo(n, root)?,
                &a.req::<Vec<String>>(n, "targets")?,
                &a.req::<PathBuf>(n, "baseline")?,
            )?;
            let report = p.report();
            Ok(
                json!({"records":p.records.into_iter().map(|r|json!({"target":r.target,"arch":r.arch,"status":r.status,"diffs":r.diffs,"note":r.note})).collect::<Vec<_>>(),"bad":p.bad,"report":report}),
            )
        }
        _ => Err(Error::new(format!(
            "unknown cargo-cell operation '{}'",
            a.op
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_arguments_and_calls_python_names() {
        let root = std::env::current_dir().unwrap();
        assert_eq!(dispatch(&root,&json!({"op":"_majority_run","args":[[["a","b"],["a","b"],["a","c"]]],"kwargs":{"share":0.6}})).unwrap(),json!(["a", "b"]));
        assert_eq!(
            dispatch(
                &root,
                &json!({"op":"_no_edge_why","kwargs":{"ninja":"", "target":"m"}})
            )
            .unwrap(),
            "absent"
        );
        assert!(dispatch(&root, &json!({"op":"_entry_symbol","args":["i386"]})).is_err());
        assert!(dispatch(
            &root,
            &json!({"op":"_entry_symbol","args":["i386"],"kwargs":{"arch":"i386"}})
        )
        .is_err());
        assert!(dispatch(
            &root,
            &json!({"op":"_entry_symbol","kwargs":{"arch":"i386","sym":"m","unknown":1}})
        )
        .is_err());
        assert!(dispatch(&root, &json!({"op":"constants","args":[1]})).is_err());
    }
    #[test]
    fn serializes_link_and_resource_dtos() {
        let root = std::env::current_dir().unwrap();
        let f = json!({"includes":["@std"],"flags":[],"defines":[]});
        let t = json!({"windres":"windres","std":["@std"],"flags":[],"defs":[],"defines":[]});
        assert!(dispatch(
            &root,
            &json!({"op":"rc_template_text","kwargs":{"template":t,"arch":"amd64"}})
        )
        .unwrap()
        .as_str()
        .unwrap()
        .contains("[windres]"));
        assert!(dispatch(
            &root,
            &json!({"op":"rc_decode","args":[f,t,"amd64","x.rc"]})
        )
        .unwrap()
        .is_array());
    }
}
