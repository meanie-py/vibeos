use crate::{Corpus, Rule};
use regex::Regex;
use std::collections::HashSet;
use std::path::Path;
pub struct OpaquePtrArith;
struct Patterns {
    skip: Regex,
    nolint: Regex,
    opaque: Regex,
    alias: Regex,
    pointer_field: Regex,
    alias_field: Regex,
    usage: Regex,
}
impl Patterns {
    fn new() -> Self {
        let re = |s| Regex::new(s).expect("valid opaque-ptr-arith regex");
        Self {
            skip: re(r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/island_[a-z0-9_]+\.rs$)"),
            nolint: re(r"//\s*nolint(?::\s*opaque-ptr-arith\b|\s*$|:?\s*\()"),
            opaque: re(r"pub struct (\w+) \{\s*pub _address: u8,\s*\}"),
            alias: re(r"pub type (\w+) = \*(?:mut|const) (\w+);"),
            pointer_field: re(r"pub (\w+): \*(?:mut|const) (\w+),"),
            alias_field: re(r"pub (\w+): (\w+),"),
            usage: re(r"\.(\w+)\.(add|offset|sub|wrapping_add|wrapping_sub)\("),
        }
    }
}
fn opaque_fields<'a>(texts: impl IntoIterator<Item = &'a str>, p: &Patterns) -> HashSet<String> {
    let mut fields = HashSet::new();
    for text in texts {
        let opaque: HashSet<_> = p
            .opaque
            .captures_iter(text)
            .map(|m| m[1].to_owned())
            .collect();
        if opaque.is_empty() {
            continue;
        }
        let aliases: HashSet<_> = p
            .alias
            .captures_iter(text)
            .filter(|m| opaque.contains(&m[2]))
            .map(|m| m[1].to_owned())
            .collect();
        fields.extend(
            p.pointer_field
                .captures_iter(text)
                .filter(|m| opaque.contains(&m[2]))
                .map(|m| m[1].to_owned()),
        );
        fields.extend(
            p.alias_field
                .captures_iter(text)
                .filter(|m| aliases.contains(&m[2]))
                .map(|m| m[1].to_owned()),
        );
    }
    fields.remove("_address");
    fields
}
fn bindgen_source(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
        return false;
    };
    if name == "opaque_fixture_bindgen.rs" {
        return true;
    }
    name.starts_with("bindgen_")
        && name.ends_with(".rs")
        && path.parent().is_some_and(|parent| {
            parent.file_name().is_some_and(|n| n == "src")
                && parent.parent().and_then(Path::parent) == Some(Path::new("sdk/lib/rust"))
        })
}
fn scan(text: &str, fields: &HashSet<String>, p: &Patterns) -> Vec<(usize, String)> {
    let lines: Vec<_> = text.lines().collect();
    let mut out = Vec::new();
    for m in p.usage.captures_iter(text) {
        if !fields.contains(&m[1]) {
            continue;
        }
        let line = text[..m.get(0).unwrap().start()]
            .bytes()
            .filter(|b| *b == b'\n')
            .count()
            + 1;
        if !p.nolint.is_match(lines[line - 1]) {
            out.push((line, m[1].to_owned()));
        }
    }
    out
}
impl Rule for OpaquePtrArith {
    fn name(&self) -> &'static str {
        "scan_opaque_ptr_arith"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let p = Patterns::new();
        let fields = opaque_fields(
            corpus
                .rust_files()
                .filter(|f| bindgen_source(&f.path))
                .map(|f| f.text.as_str()),
            &p,
        );
        if fields.is_empty() {
            return Ok(Vec::new());
        }
        let mut rows = Vec::new();
        for f in corpus.rust_files() {
            if p.skip.is_match(&f.path.to_string_lossy())
                || f.path
                    .file_name()
                    .is_some_and(|s| s == "opaque_fixture_bindgen.rs")
            {
                continue;
            }
            for (line, field) in scan(&f.text, &fields, &p) {
                rows.push((f.path.clone(), line, field));
            }
        }
        rows.sort();
        Ok(rows
            .into_iter()
            .map(|(path, line, field)| {
                format!("{}:{line}: opaque-ptr-arith .{field}", path.display())
            })
            .collect())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    const BINDGEN:&str="pub struct Opaque { pub _address: u8, }\npub type POpaque = *mut Opaque;\npub struct Table { pub direct: *mut Opaque, pub alias: POpaque, pub plain: *mut u32, }";
    #[test]
    fn pointer_and_alias_fields() {
        let p = Patterns::new();
        let fields = opaque_fields([BINDGEN], &p);
        assert_eq!(
            fields,
            HashSet::from(["direct".to_owned(), "alias".to_owned()])
        );
        assert_eq!(
            scan(
                "p.direct.add(i); p.alias.offset(i); p.plain.add(i);",
                &fields,
                &p
            ),
            vec![(1, "direct".to_owned()), (1, "alias".to_owned())]
        );
    }
    #[test]
    fn casts_spacing_and_suppression() {
        let p = Patterns::new();
        let fields = opaque_fields([BINDGEN], &p);
        assert!(scan(
            "(p.direct as *mut Real).add(i); p.direct .add(i);",
            &fields,
            &p
        )
        .is_empty());
        assert!(scan(
            "p.direct.add(i); // nolint: opaque-ptr-arith aligned",
            &fields,
            &p
        )
        .is_empty());
    }
    #[test]
    fn bindgen_scope_and_file_local_opaque_index() {
        assert!(bindgen_source(Path::new(
            "sdk/lib/rust/one-sys/src/bindgen_amd64.rs"
        )));
        assert!(!bindgen_source(Path::new(
            "sdk/lib/rust/one-sys/src/sub/bindgen_amd64.rs"
        )));
        assert!(bindgen_source(Path::new(
            "sdk/lib/rust/fixture/opaque_fixture_bindgen.rs"
        )));
        let p = Patterns::new();
        assert!(opaque_fields(
            [
                "pub struct Opaque { pub _address: u8, }",
                "pub struct Table { pub direct: *mut Opaque, }"
            ],
            &p
        )
        .is_empty());
    }
}
