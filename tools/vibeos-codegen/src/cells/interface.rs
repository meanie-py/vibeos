//! A fully-Rust STATIC library becomes a CMake INTERFACE library over its Rust archive.
//!
//! A cohab-flipped static library still archives one placeholder TU
//! (`rust_cohab_stub.c`, a lone typedef) and links `rust_staticlib_variant`'s archive
//! through it. The stub archive is empty: every consumer's real input is the Rust archive.
//! A cargo cell consumer already skips the stub (`graph::placeholder_archive`) and links
//! the crate (`engine:crate:`); a cmake-linked consumer gets the archive transitively.
//! An INTERFACE library hands every consumer the same archive with no stub step, so
//! nothing of the module is "linked by cmake" any more, and its cmake consumers keep
//! linking it by name until they become cells themselves.
//!
//! The rewrite is proven, never guessed: the archive's only object on every arch it is
//! built for is the placeholder stub, and every statement on the target is one of the
//! modelled shapes below. Anything else refuses with the reason.
use super::graph::{args, list_vars, placeholder, source_dir, statements, target_file_refs};
use super::{local, read, Error, Result, ARCHES};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

fn re(s: &str) -> Regex {
    Regex::new(s).expect("constant regex")
}

#[derive(Debug, Default)]
pub struct InterfacePlan {
    pub target: String,
    pub cmake: PathBuf,
    pub text: String,
    /// Placeholder stub TUs nothing compiles once the target is INTERFACE, and the
    /// `clibs/<arch>/<target>.recipe`s that compiled only them for a cargo cell.
    pub removed: Vec<PathBuf>,
    /// Cargo cells' build.rs with the stub's `engine:c:<target>` link token dropped.
    pub cell_edits: BTreeMap<PathBuf, String>,
}

/// The static archive edges of `target`: (arch, output, object inputs).
fn archive_edges(root: &Path, target: &str) -> Result<Vec<(String, String, Vec<String>)>> {
    let pat = re(&format!(
        r"(?m)^build (\S+): C_STATIC_LIBRARY_LINKER__{}_(?:Debug|Release|RelWithDebInfo|MinSizeRel) ([^|\n]*)",
        regex::escape(target)
    ));
    let mut out = Vec::new();
    for arch in ARCHES {
        let ninja = local(root, &root.join(format!("build-{arch}/build.ninja")))?;
        if let Some(c) = pat.captures(&read(&ninja)?) {
            let objs = c[2].split_whitespace().map(str::to_owned).collect();
            out.push((arch.to_owned(), c[1].to_owned(), objs));
        }
    }
    if out.is_empty() {
        return Err(Error::new(format!(
            "{target}: no static archive edge on any arch (not a STATIC library, or not configured)"
        )));
    }
    Ok(out)
}

/// The comment lines inside a statement's argument text (the flipped TUs' records).
fn comment_lines(raw: &str) -> Vec<String> {
    raw.lines()
        .map(str::trim)
        .filter(|l| l.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

fn entry_is_cell(build: &Path) -> bool {
    build.is_file()
        && build
            .parent()
            .and_then(|p| p.file_name())
            .is_some_and(|n| n.to_string_lossy().ends_with("-cargo"))
}
/// Rewrap one `# ...` line to `width` columns at word boundaries, keeping its indent.
pub fn wrap_comment(line: &str, width: usize) -> String {
    let nl = if line.ends_with('\n') { "\n" } else { "" };
    let body = line.trim_end_matches('\n');
    let indent = &body[..body.len() - body.trim_start().len()];
    let Some(text) = body.trim_start().strip_prefix("# ") else {
        return line.to_owned();
    };
    if body.len() <= width {
        return line.to_owned();
    }
    let prefix = format!("{indent}# ");
    let mut out = Vec::new();
    let mut cur = prefix.clone();
    for word in text.split(' ') {
        if cur.len() > prefix.len() && cur.len() + 1 + word.len() > width {
            out.push(cur);
            cur = prefix.clone();
        }
        if cur.len() > prefix.len() {
            cur.push(' ');
        }
        cur.push_str(word);
    }
    out.push(cur);
    out.join("\n") + nl
}
pub fn plan(root: &Path, target: &str) -> Result<InterfacePlan> {
    plan_on(root, target, &BTreeMap::new())
}
/// `plan`, reading a CMakeLists an earlier plan of the same wave already rewrote.
pub fn plan_on(root: &Path, target: &str, pending: &BTreeMap<PathBuf, String>) -> Result<InterfacePlan> {
    let edges = archive_edges(root, target)?;
    let (arch, output, _) = &edges[0];
    let dir = source_dir(root, arch, output, target)?;
    let cmake = local(root, &dir.join("CMakeLists.txt"))?;
    for (a, _, objs) in &edges {
        let live: Vec<_> = objs
            .iter()
            .filter(|o| o.rsplit('/').next() != Some("rust_cohab_stub.c.obj"))
            .collect();
        if !live.is_empty() || objs.is_empty() {
            return Err(Error::new(format!(
                "{target}: {a} archives more than the placeholder stub: {live:?}"
            )));
        }
    }
    let stub = dir.join("rust_cohab_stub.c");
    if !stub.is_file() || !placeholder(&local(root, &stub)?)? {
        return Err(Error::new(format!(
            "{target}: {} is not a placeholder (it defines something the archive carries)",
            stub.display()
        )));
    }
    if let Some(refs) = target_file_refs(root)?.get(target) {
        return Err(Error::new(format!(
            "{target}: read as a file via $<TARGET_FILE...> in {refs:?}"
        )));
    }
    let text = match pending.get(&cmake) {
        Some(t) => t.clone(),
        None => read(&cmake)?,
    };
    if re(&format!(r"\$<TARGET_OBJECTS:{}>", regex::escape(target))).is_match(&text) {
        return Err(Error::new(format!("{target}: its objects are reused ($<TARGET_OBJECTS>)")));
    }
    let lists = list_vars(&text);
    let mut out = String::new();
    let mut pos = 0;
    let mut body_lib = None;
    let mut source_vars = BTreeSet::new();
    let mut seen_add = false;
    for (s, e, cmd, raw) in statements(&text) {
        let a = args(&raw);
        if a.first().map(String::as_str) != Some(target) {
            if cmd == "rust_staticlib_variant" && a.len() >= 2 {
                body_lib.get_or_insert_with(BTreeMap::new).insert(a[0].clone(), a[1].clone());
            }
            continue;
        }
        let head_ws = &text[s..s + (text[s..].len() - text[s..].trim_start().len())];
        let rest = &a[1..];
        let kw = |x: &str| ["PRIVATE", "PUBLIC", "INTERFACE"].contains(&x);
        let replacement: Option<String> = match cmd.as_str() {
            "add_library" => {
                if seen_add {
                    return Err(Error::new(format!("{target}: two add_library statements")));
                }
                seen_add = true;
                let mut srcs = rest.to_vec();
                if srcs.first().is_some_and(|x| x == "STATIC") {
                    srcs.remove(0);
                }
                if srcs.iter().any(|x| ["SHARED", "MODULE", "OBJECT", "INTERFACE"].contains(&x.as_str())) {
                    return Err(Error::new(format!("{target}: add_library({target} {}) is not a static library", srcs.join(" "))));
                }
                for src in &srcs {
                    if let Some(v) = src.strip_prefix("${").and_then(|x| x.strip_suffix('}')) {
                        source_vars.insert(v.to_owned());
                    }
                }
                let mut lines = comment_lines(&raw);
                lines.push(format!("add_library({target} INTERFACE)"));
                Some(lines.into_iter().map(|l| format!("{head_ws}{l}\n")).collect())
            }
            "target_link_libraries" => {
                // A static library hands consumers every link item: PUBLIC and plain ones
                // with their usage requirements, PRIVATE ones as $<LINK_ONLY:..> (link
                // only). INTERFACE spells the same thing, so the consumers' links do not move.
                let mut items = Vec::new();
                let mut scope = "PUBLIC";
                for x in rest {
                    if kw(x) {
                        scope = x.as_str();
                    } else if scope == "PRIVATE" {
                        items.push(format!("$<LINK_ONLY:{x}>"));
                    } else {
                        items.push(x.clone());
                    }
                }
                Some(format!("{head_ws}target_link_libraries({target} INTERFACE {})\n", items.join(" ")))
            }
            "add_importlibs" => Some(format!(
                "{head_ws}target_link_libraries({target} INTERFACE {})\n",
                rest.iter().map(|l| format!("lib{l}")).collect::<Vec<_>>().join(" ")
            )),
            "add_dependencies" => None,
            "add_pch" => Some(String::new()),
            "target_include_directories" | "target_compile_definitions" | "target_compile_options" => {
                // Nothing compiles in an INTERFACE library: its own (PRIVATE) usage goes,
                // what consumers got (PUBLIC/INTERFACE) stays theirs.
                let mut scope = "";
                let mut keep = Vec::new();
                let mut iter = rest.iter().peekable();
                let mut before = true;
                while let Some(x) = iter.next() {
                    if before && (x == "BEFORE" || x == "SYSTEM" || x == "AFTER") {
                        keep.push(x.clone());
                        continue;
                    }
                    before = false;
                    if kw(x) {
                        scope = x.as_str();
                        if scope != "PRIVATE" {
                            keep.push("INTERFACE".into());
                        }
                        continue;
                    }
                    if scope.is_empty() {
                        return Err(Error::new(format!("{target}: {cmd} without a scope keyword")));
                    }
                    if scope != "PRIVATE" {
                        keep.push(x.clone());
                    }
                }
                if keep.iter().all(|x| ["BEFORE", "SYSTEM", "AFTER", "INTERFACE"].contains(&x.as_str())) {
                    Some(String::new())
                } else {
                    Some(format!("{head_ws}{cmd}({target} {})\n", keep.join(" ")))
                }
            }
            _ => {
                return Err(Error::new(format!(
                    "{target}: {cmd}({target} ...) is not modelled for an INTERFACE library"
                )))
            }
        };
        if let Some(mut rep) = replacement {
            // A trailing `# comment` after the `)` stays on the statement's line.
            let trailing = !text[e..].starts_with('\n') && !text[e..].is_empty() && !text[s..e].ends_with('\n');
            if trailing && rep.ends_with('\n') {
                rep.pop();
            }
            out.push_str(&text[pos..s]);
            out.push_str(&rep);
            pos = e;
        }
    }
    if !seen_add {
        return Err(Error::new(format!("{target}: no add_library({target} ...) in {}", cmake.display())));
    }
    out.push_str(&text[pos..]);
    // The body: at least one Rust archive the target links, either a
    // rust_staticlib_variant's ${V_LIB} or the OUTPUT of a custom command that runs
    // `cargo build` (the shape before rust_staticlib_variant existed).
    let mut archives: BTreeSet<String> = body_lib
        .unwrap_or_default()
        .keys()
        .map(|v| format!("${{{v}_LIB}}"))
        .collect();
    for (_, _, cmd, raw) in statements(&text) {
        if cmd == "add_custom_command" && re(r"\bcargo build\b").is_match(&raw) {
            let a = args(&raw);
            if let Some(i) = a.iter().position(|x| x == "OUTPUT") {
                if let Some(o) = a.get(i + 1) {
                    archives.insert(o.clone());
                }
            }
        }
    }
    let links: Vec<String> = statements(&out)
        .into_iter()
        .filter(|(_, _, cmd, raw)| cmd == "target_link_libraries" && args(raw).first().map(String::as_str) == Some(target))
        .flat_map(|(_, _, _, raw)| args(&raw))
        .map(|x| x.trim_start_matches("$<LINK_ONLY:").trim_end_matches('>').to_owned())
        .collect();
    if !links.iter().any(|x| archives.contains(x)) {
        return Err(Error::new(format!(
            "{target}: links no Rust archive (no rust_staticlib_variant or `cargo build` OUTPUT among {links:?})"
        )));
    }
    // A source list only this target read loses its live entries; its comments stay.
    let mut removed = Vec::new();
    for var in &source_vars {
        let used = re(&format!(r"\$\{{{}\}}", regex::escape(var))).find_iter(&out).count();
        if used != 0 {
            return Err(Error::new(format!("{target}: ${{{var}}} is read by another statement too")));
        }
        let entries = lists.get(var).cloned().unwrap_or_default();
        if entries.iter().any(|e| !(e.ends_with(".h") || e == "rust_cohab_stub.c")) {
            return Err(Error::new(format!("{target}: ${{{var}}} lists more than the stub and headers: {entries:?}")));
        }
        let mut text2 = String::new();
        let mut p2 = 0;
        for (s, e, cmd, raw) in statements(&out) {
            let a = args(&raw);
            let ours = (cmd == "set" && a.first() == Some(var))
                || (cmd == "list" && a.len() > 1 && a[0] == "APPEND" && &a[1] == var);
            if ours {
                let ws = &out[s..s + (out[s..].len() - out[s..].trim_start().len())];
                text2.push_str(&out[p2..s]);
                for l in comment_lines(&raw) {
                    text2.push_str(&format!("{ws}{l}\n"));
                }
                p2 = e;
            }
        }
        text2.push_str(&out[p2..]);
        out = text2;
    }
    // The stub goes only when nothing else in the file still compiles it.
    let live_stub = out
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .any(|l| l.contains("rust_cohab_stub.c"));
    if !live_stub {
        removed.push(stub);
    }
    // Say what the target is now: rewrite the flip's "keeps a stub TU" sentence, or add
    // a note above the target when there is none.
    let stale = re(r"keeps(\s+#)?\s+a\s+stub\s+TU(\s+#)?\s+and(\s+#)?\s+links(\s+#)?\s+the(\s+#)?\s+Rust(\s+#)?\s+archive(s?)");
    if stale.is_match(&out) {
        out = stale
            .replace(&out, "is an INTERFACE library over the Rust archive$7 (`cargo-cell --interface`)")
            .into_owned();
        // The sentence may have spanned comment lines: rewrap what grew past the margin.
        out = out
            .split_inclusive('\n')
            .map(|l| if l.contains("cargo-cell --interface") { wrap_comment(l, 92) } else { l.to_owned() })
            .collect();
    } else {
        let note = format!(
            "# {target} is all Rust: an INTERFACE library over its Rust archive, so consumers link\n# it directly and no placeholder archive is built (`tools/codegen cargo-cell --interface`).\n"
        );
        let at = out
            .find(&format!("add_library({target} INTERFACE)"))
            .ok_or("rewritten add_library not found")?;
        let mut start = out[..at].rfind('\n').map_or(0, |i| i + 1);
        while start > 0 {
            let prev = out[..start - 1].rfind('\n').map_or(0, |i| i + 1);
            if out[prev..start].trim_start().starts_with('#') {
                start = prev;
            } else {
                break;
            }
        }
        out.insert_str(start, &note);
    }
    out = re(r"\n{3,}").replace_all(&out, "\n\n").into_owned();
    // A cell that compiled the stub itself (`engine:c:<target>`, a recipe of only the
    // placeholder) links nothing from it: the Rust comes in under its own token.
    let token = format!("\"engine:c:{target}\"");
    let mut cell_edits = BTreeMap::new();
    let rust = root.join("sdk/lib/rust");
    for entry in std::fs::read_dir(&rust)? {
        let build = entry?.path().join("build.rs");
        if !entry_is_cell(&build) {
            continue;
        }
        let text = match pending.get(&build) {
            Some(t) => t.clone(),
            None => read(&local(root, &build)?)?,
        };
        if !text.contains(&token) {
            continue;
        }
        let edited = re(&format!(r"{}, |, {}", regex::escape(&token), regex::escape(&token)))
            .replace_all(&text, "")
            .into_owned();
        if edited.contains(&token) {
            return Err(Error::new(format!("{target}: {} names {token} in a shape the rewrite does not drop", build.display())));
        }
        cell_edits.insert(build, edited);
    }
    for arch in ARCHES {
        let recipe = super::recipes::clib_recipe_path(arch, target, root);
        if !recipe.is_file() {
            continue;
        }
        let body = read(&local(root, &recipe)?)?;
        let srcs: Vec<_> = body
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .map(|l| l.split('\t').nth(1).unwrap_or(""))
            .collect();
        if srcs.is_empty() || srcs.iter().any(|s| !s.ends_with("/rust_cohab_stub.c")) {
            return Err(Error::new(format!(
                "{target}: {} compiles more than the placeholder: {srcs:?}",
                recipe.display()
            )));
        }
        removed.push(recipe);
    }
    Ok(InterfacePlan {
        target: target.into(),
        cmake,
        text: out,
        removed,
        cell_edits,
    })
}
