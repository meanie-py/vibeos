use crate::{Corpus, Rule, SourceFile, SourceSet};
use std::collections::BTreeSet;
use tree_sitter::Node;

pub struct SilentFailures;
const SOURCES: &[SourceSet] =
    &[
        SourceSet::tree("silent-rust", &["sdk/lib/rust"], &["rs"], &["Cargo.toml"])
            .with_pruning(&["target", "tests", "examples", "benches"])
            .with_inputs(),
    ];
fn excluded(path: &std::path::Path) -> bool {
    path.components().any(|c| {
        matches!(
            c.as_os_str().to_str(),
            Some("target" | "tests" | "examples" | "benches")
        )
    })
}
fn scan(f: &SourceFile) -> Result<Vec<String>, String> {
    let tree = f.rust_tree()?;
    let text = f.text.as_bytes();
    let lines: Vec<&str> = f.text.lines().collect();
    let mut hits = Vec::new();
    fn empty(n: Node<'_>) -> bool {
        let mut c = n.walk();
        let result = n
            .children(&mut c)
            .all(|x| matches!(x.kind(), "{" | "}" | "line_comment" | "block_comment"));
        result
    }
    fn rec(
        n: Node<'_>,
        attrs: &[Node<'_>],
        src: &[u8],
        lines: &[&str],
        f: &SourceFile,
        hits: &mut Vec<String>,
    ) {
        if matches!(n.kind(), "const_item" | "static_item") {
            return;
        }
        if matches!(n.kind(), "mod_item" | "function_item")
            && attrs
                .iter()
                .any(|a| src[a.byte_range()].windows(4).any(|w| w == b"test"))
        {
            return;
        }
        let line = n.start_position().row + 1;
        let suppressed = |ln: usize| {
            lines.get(ln.saturating_sub(1)).is_some_and(|s| {
                let scoped = regex::Regex::new(r"//\s*nolint:\s*([a-z-]+)").unwrap();
                scoped.captures(s).map_or_else(
                    || s.contains("//") && regex::Regex::new(r"//\s*nolint\b").unwrap().is_match(s),
                    |m| &m[1] == "silent-failure",
                )
            })
        };
        let mut add = |ln: usize, k: &str, msg: &str| {
            if !suppressed(ln) {
                hits.push(format!("{}:{ln}: [{k}] {msg}", f.path.display()));
            }
        };
        match n.kind() {
            "loop_expression" => {
                if let Some(b) = n.child_by_field_name("body") {
                    if empty(b) {
                        add(
                            line,
                            "SILENT_LOOP",
                            "empty `loop {}` — silent busy-spin; print a self-report first",
                        )
                    }
                }
            }
            "call_expression" => {
                if let Some(fun) = n.child_by_field_name("function") {
                    if fun.kind() == "field_expression" {
                        if let Some(field) = fun.child_by_field_name("field") {
                            let name = &src[field.byte_range()];
                            if name == b"unwrap" || name == b"expect" {
                                let ln = field.start_position().row + 1;
                                add(
                                    ln,
                                    "UNWRAP",
                                    &format!(
                                        ".{}() — panics on Err/None in kernel code",
                                        String::from_utf8_lossy(name)
                                    ),
                                );
                            }
                        }
                    }
                    if let Some(args) = n.child_by_field_name("arguments") {
                        let mut cur = args.walk();
                        let a: Vec<_> = args.named_children(&mut cur).collect();
                        let fname = &src[fun.byte_range()];
                        if a.len() == 3
                            && (fname.ends_with(b"write_bytes")
                                || fname.ends_with(b"copy_nonoverlapping")
                                || fname.ends_with(b"ptr::copy"))
                        {
                            let mut p = src[a[0].byte_range()].to_vec();
                            if !fname.ends_with(b"write_bytes") {
                                p.extend_from_slice(&src[a[1].byte_range()]);
                            }
                            let ct = &src[a[2].byte_range()];
                            let size1 = [
                                b"*mut u8".as_slice(),
                                b"*const u8",
                                b"*mut i8",
                                b"*const i8",
                                b"*mut CHAR",
                                b"*const CHAR",
                                b"*mut UCHAR",
                                b"*const UCHAR",
                                b"PUCHAR",
                                b"PCHAR",
                                b"LPBYTE",
                                b"PBYTE",
                                b"*mut BYTE",
                                b"*const BYTE",
                                b"c_void",
                                b"PVOID",
                                b"<u8>",
                            ]
                            .iter()
                            .any(|x| p.windows(x.len()).any(|w| w == *x));
                            if ct.windows(7).any(|w| w == b"size_of")
                                && !size1
                                && !ct.contains(&b'/')
                            {
                                add(line,"WRITE_BYTES_SIZEOF","write_bytes(ptr, _, size_of::<T>()) — count is in ELEMENTS, zeroes sizeof(T)^2 bytes; cast ptr `as *mut u8`")
                            }
                        }
                    }
                }
            }
            "match_arm" => {
                if let (Some(p), Some(v)) = (
                    n.child_by_field_name("pattern"),
                    n.child_by_field_name("value"),
                ) {
                    if src[p.byte_range()].starts_with(b"Err(")
                        && (v.kind() == "unit_expression" || (v.kind() == "block" && empty(v)))
                    {
                        add(
                            line,
                            "SWALLOWED_ERR",
                            "Err(...) arm does nothing — error silently discarded",
                        )
                    }
                }
            }
            "if_expression" => {
                if let (Some(cond), Some(cons)) = (
                    n.child_by_field_name("condition"),
                    n.child_by_field_name("consequence"),
                ) {
                    let raw = &src[cond.byte_range()];
                    if cond.kind() == "let_condition"
                        && raw
                            .split(|b| *b == b'=')
                            .next()
                            .unwrap_or_default()
                            .windows(4)
                            .any(|w| w == b"Err(")
                        && empty(cons)
                        && n.child_by_field_name("alternative").is_none()
                    {
                        add(
                            line,
                            "SWALLOWED_ERR",
                            "`if let Err(...) {}` — error silently discarded",
                        )
                    }
                }
            }
            _ => {}
        }
        let mut attrs = Vec::new();
        let mut cur = n.walk();
        for ch in n.children(&mut cur) {
            if ch.kind() == "attribute_item" {
                attrs.push(ch)
            } else {
                rec(ch, &attrs, src, lines, f, hits);
                attrs.clear()
            }
        }
    }
    rec(tree.root_node(), &[], text, &lines, f, &mut hits);
    Ok(hits)
}
impl Rule for SilentFailures {
    fn name(&self) -> &'static str {
        "scan_silent_failures"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        SOURCES
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        if !c.explicit_paths().is_empty() {
            let explicit: BTreeSet<_> = c.explicit_paths().iter().cloned().collect();
            let mut out = Vec::new();
            for f in c
                .sources("silent-rust")
                .filter(|f| explicit.contains(f.resolved_path()))
            {
                out.extend(scan(f)?);
            }
            out.sort();
            return Ok(out);
        }
        let mut manifests = BTreeSet::new();
        let mut rust: Vec<_> = c
            .sources("silent-rust")
            .filter(|f| {
                f.path.extension().is_some_and(|x| x == "rs")
                    && !excluded(&f.path)
                    && f.path.file_name().is_some_and(|n| {
                        n != "build.rs" && !n.to_string_lossy().starts_with("bindgen_")
                    })
            })
            .collect();
        for m in c
            .sources("silent-rust")
            .filter(|f| f.path.file_name().is_some_and(|n| n == "Cargo.toml") && !excluded(&f.path))
        {
            let src = m.path.parent().unwrap().join("src");
            let root = ["lib.rs", "main.rs"]
                .iter()
                .find_map(|n| c.sources("silent-rust").find(|f| f.path == src.join(n)));
            if let Some(r) = root {
                if r.text
                    .chars()
                    .take(4096)
                    .collect::<String>()
                    .contains("no_std")
                {
                    manifests.insert(src);
                }
            }
        }
        // ORPHAN-BY-DESIGN and the crate-root discovery are lexical legacy gates.
        rust.retain(|f| {
            manifests.iter().any(|r| f.path.starts_with(r))
                && !f
                    .text
                    .chars()
                    .take(2048)
                    .collect::<String>()
                    .contains("ORPHAN-BY-DESIGN:")
        });
        rust.sort_by(|a, b| a.path.cmp(&b.path));
        let mut out = Vec::new();
        for f in rust {
            out.extend(scan(f)?)
        }
        out.sort();
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Corpus, SourceFile};
    use std::path::PathBuf;
    fn file(s: &str) -> SourceFile {
        SourceFile::new(PathBuf::from("sdk/lib/rust/x/src/lib.rs"), s.into())
    }
    #[test]
    fn controls() {
        let f = file(
            "pub unsafe fn spin(){ let v:Result<u32,u32>=Err(1); let _x=v.unwrap(); loop {} }\n",
        );
        let x = scan(&f).unwrap();
        assert!(x.iter().any(|a| a.contains("[UNWRAP]")));
        assert!(x.iter().any(|a| a.contains("[SILENT_LOOP]")));
        let f = file("pub unsafe fn spin()->Result<u32,u32>{let v:Result<u32,u32>=Err(1);v}\n");
        assert!(scan(&f).unwrap().is_empty());
    }
    #[test]
    fn explicit_inputs_bypass_default_no_std_filters() {
        let source = SourceFile::new(
            PathBuf::from("tools/plain.rs"),
            "pub fn f(){ loop {} }".into(),
        );
        let corpus = Corpus::new(PathBuf::from("."), vec![source])
            .with_explicit_paths(vec![PathBuf::from("tools/plain.rs")]);
        let findings = SilentFailures.run(&corpus).unwrap();
        assert_eq!(findings.len(), 1);
        assert!(findings[0].contains("[SILENT_LOOP]"));
    }
    #[test]
    fn explicit_directory_does_not_expand_in_rule() {
        let source = SourceFile::new(
            PathBuf::from("tools/plain.rs"),
            "pub fn f(){ loop {} }".into(),
        );
        let corpus = Corpus::new(PathBuf::from("."), vec![source])
            .with_explicit_paths(vec![PathBuf::from("tools")]);
        assert!(SilentFailures.run(&corpus).unwrap().is_empty());
    }
    #[test]
    fn suppressions_test_subtree_and_swallowed() {
        let f=file("pub fn f(){ loop {} // nolint: silent-failure\n if let Err(e)=f() {}\n match x { Err(_)=>{}, _=>{} } }\n#[cfg(test)] mod tests { fn t(){x.unwrap();} }\n");
        let x = scan(&f).unwrap();
        assert!(!x.iter().any(|a| a.contains("SILENT_LOOP")));
        assert!(x.iter().any(|a| a.contains("SWALLOWED_ERR")));
        assert!(!x.iter().any(|a| a.contains("UNWRAP")));
    }
}
