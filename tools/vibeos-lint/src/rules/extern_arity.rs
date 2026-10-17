//! Crate-local extern declarations whose call shape disagrees with a live definition.
//! Port of tools/scan_extern_arity.py; lexical masking is byte-for-byte crate_repair's
//! `_mask_lexical`, since all parser offsets and regex matches are byte offsets.
#[cfg(test)]
use crate::SourceFile;
use crate::{Corpus, Rule};
use regex::bytes::{Regex, RegexBuilder};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub struct ExternArity;

impl Rule for ExternArity {
    fn name(&self) -> &'static str {
        "scan_extern_arity"
    }
    fn needs_module_tree(&self) -> bool {
        true
    }

    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let tree = corpus.module_tree();
        let mut crates: BTreeMap<PathBuf, CrateFacts> = BTreeMap::new();

        // os.walk visits each directory's sorted files before recursing into sorted child
        // directories. Sorting by (parent components, filename) is that same preorder.
        // The module-tree rule's selected snapshot includes the default sdk/lib/rust
        // tree and explicit fixture files; rust_files() is the corpus-wide source view.
        let mut files: Vec<_> = corpus.rust_files().collect();
        files.sort_by(|a, b| walk_key(&a.path).cmp(&walk_key(&b.path)));
        for file in files {
            let Some(crate_root) = tree.crate_root_of(&file.path) else {
                continue;
            };
            let bytes = file.bytes();
            if bytes
                .get(..bytes.len().min(2048))
                .unwrap_or(bytes)
                .windows(17)
                .any(|w| w == b"ORPHAN-BY-DESIGN:")
            {
                continue;
            }
            if !contains_token(bytes, b"extern") && !contains_token(bytes, b"no_mangle") {
                continue;
            }
            let facts = parse_facts(bytes);
            let entry = crates.entry(crate_root.to_path_buf()).or_default();
            for (symbol, sites) in facts.decls {
                entry
                    .decls
                    .entry(symbol)
                    .or_default()
                    .extend(sites.into_iter().map(|(name, sig)| Site {
                        path: file.path.clone(),
                        resolved: file.resolved_path().to_path_buf(),
                        name,
                        sig,
                    }));
            }
            for (symbol, sig) in facts.defs {
                entry.defs.insert(symbol, (file.path.clone(), sig));
            }
        }

        let mut output = Vec::new();
        for (crate_root, facts) in crates {
            let live = tree.live_files(&crate_root);
            let live: HashSet<PathBuf> = live.iter().cloned().collect();
            output.extend(render_findings(&facts, &live));
        }
        output.sort_by(|a, b| finding_sort_key(a).cmp(&finding_sort_key(b)));
        Ok(output)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Sig {
    raw: String,
    masked: String,
    ret: Ret,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ret {
    Unit,
    Value,
}
#[derive(Default)]
struct Facts {
    decls: BTreeMap<String, Vec<(String, Sig)>>,
    defs: BTreeMap<String, Sig>,
}
#[derive(Default)]
struct CrateFacts {
    decls: BTreeMap<String, Vec<Site>>,
    defs: BTreeMap<String, (PathBuf, Sig)>,
}
#[derive(Clone)]
struct Site {
    path: PathBuf,
    resolved: PathBuf,
    name: String,
    sig: Sig,
}

fn walk_key(path: &Path) -> (Vec<String>, String) {
    let parent = path.parent().unwrap_or(Path::new(""));
    let dirs = parent
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    (
        dirs,
        path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
    )
}

fn regexes() -> &'static RegexSet {
    static SET: OnceLock<RegexSet> = OnceLock::new();
    SET.get_or_init(|| RegexSet {
        extern_block: byte_regex(r#"(?:unsafe\s+)?extern\s+[^\{;()]*\{"#),
        decl: byte_regex(r"\bfn\s+(\w+)\s*\("),
        def: byte_regex(r#"#\[no_mangle\]\s*(?:#\[[^\]]*\]\s*)*pub\s+(?:unsafe\s+)?(?:extern\s+"[A-Za-z-]+"\s+)?fn\s+(\w+)\s*\("#),
        link: byte_regex(r#"#\s*\[\s*link_name\s*=\s*"([^"]+)"\s*\]\s*(?:pub\s+)?(?:unsafe\s+)?$"#),
        export: byte_regex(r#"#\s*\[\s*export_name\s*=\s*"([^"]+)"\s*\]"#),
    })
}
struct RegexSet {
    extern_block: Regex,
    decl: Regex,
    def: Regex,
    link: Regex,
    export: Regex,
}
fn byte_regex(pattern: &str) -> Regex {
    RegexBuilder::new(pattern).unicode(false).build().unwrap()
}
fn contains_token(data: &[u8], needle: &[u8]) -> bool {
    data.windows(needle.len()).any(|w| w == needle)
}

// Exact crate_repair._mask_lexical behavior: preserve every byte and newline, only mask
// line/block comments, quoted strings, and complete char literals (not lifetimes).
fn mask_lexical(data: &[u8]) -> Vec<u8> {
    let mut out = data.to_vec();
    let mut i = 0;
    while i < data.len() {
        if data.get(i..i + 2) == Some(b"//") {
            let end = data[i + 2..]
                .iter()
                .position(|&b| b == b'\n')
                .map_or(data.len(), |n| i + 2 + n);
            out[i..end].fill(b' ');
            i = end;
        } else if data.get(i..i + 2) == Some(b"/*") {
            let end = data[i + 2..]
                .windows(2)
                .position(|w| w == b"*/")
                .map_or(data.len(), |n| i + 2 + n + 2);
            for p in i..end {
                if out[p] != b'\n' {
                    out[p] = b' ';
                }
            }
            i = end;
        } else if data[i] == b'"' {
            let mut p = i + 1;
            while p < data.len() {
                if data[p] == b'\\' {
                    p = (p + 2).min(data.len());
                    continue;
                }
                if data[p] == b'"' {
                    p += 1;
                    break;
                }
                p += 1;
            }
            for q in i..p {
                if out[q] != b'\n' {
                    out[q] = b' ';
                }
            }
            i = p;
        } else if data[i] == b'\'' {
            if let Some(end) = char_literal_end(data, i) {
                out[i..end].fill(b' ');
                i = end;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    out
}
fn char_literal_end(s: &[u8], i: usize) -> Option<usize> {
    let mut p = i + 1;
    if p >= s.len() || s[p] == b'\n' {
        return None;
    }
    if s[p] == b'\\' {
        p += 1;
        if p >= s.len() || s[p] == b'\n' {
            return None;
        }
        p += 1;
    } else {
        p += 1;
    }
    (s.get(p) == Some(&b'\'')).then_some(p + 1)
}
fn match_forward(s: &[u8], start: usize) -> Option<usize> {
    match *s.get(start)? {
        b'(' | b'[' | b'{' => (),
        _ => return None,
    }
    let mut depth = 0isize;
    for (p, &b) in s.iter().enumerate().skip(start) {
        if matches!(b, b'(' | b'[' | b'{') {
            depth += 1
        } else if matches!(b, b')' | b']' | b'}') {
            depth -= 1;
            if depth == 0 {
                return Some(p + 1);
            }
        }
    }
    None
}
fn text<'a>(s: &'a [u8]) -> &'a str {
    std::str::from_utf8(s).unwrap_or("")
}
fn symbol(region: &[u8], rust_name: &[u8], re: &Regex) -> String {
    re.captures(region)
        .and_then(|c| c.get(1))
        .map(|m| String::from_utf8_lossy(m.as_bytes()).into_owned())
        .unwrap_or_else(|| String::from_utf8_lossy(rust_name).into_owned())
}
fn parse_sig(raw: &[u8], masked: &[u8], open: usize) -> Option<Sig> {
    let close = match_forward(masked, open)?;
    let raw_params = &raw[open + 1..close - 1];
    let masked_params = &masked[open + 1..close - 1];
    let tail = &masked[close..(close + 200).min(masked.len())];
    let tail = text(tail)
        .split('{')
        .next()
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("");
    let ret = if let Some((_, v)) = tail.split_once("->") {
        let v = v.trim();
        if v.is_empty() || v == "()" {
            Ret::Unit
        } else {
            Ret::Value
        }
    } else {
        Ret::Unit
    };
    Some(Sig {
        raw: String::from_utf8_lossy(raw_params).into_owned(),
        masked: String::from_utf8_lossy(masked_params).into_owned(),
        ret,
    })
}
fn nargs(s: &str) -> usize {
    let chars: Vec<char> = s.replace("->", "→").chars().collect();
    let mut depth = 0isize;
    let mut count = 0;
    let mut nonempty = false;
    for ch in chars {
        if matches!(ch, '<' | '(' | '[' | '{') {
            depth += 1
        } else if matches!(ch, '>' | ')' | ']' | '}') {
            depth -= 1
        }
        if ch == ',' && depth == 0 {
            if nonempty {
                count += 1
            }
            nonempty = false;
        } else if !ch.is_whitespace() {
            nonempty = true;
        }
    }
    if nonempty {
        count += 1
    }
    count
}
fn parse_facts(data: &[u8]) -> Facts {
    let masked = mask_lexical(data);
    let r = regexes();
    let mut facts = Facts::default();
    for bm in r.extern_block.find_iter(&masked) {
        let Some(end) = match_forward(&masked, bm.end() - 1) else {
            continue;
        };
        let mut prev = bm.end();
        for dm in r.decl.captures_iter(&masked[bm.end()..end]) {
            let m = dm.get(0).unwrap();
            let start = bm.end() + m.start();
            let open = bm.end() + m.end() - 1;
            if let Some(sig) = parse_sig(data, &masked, open) {
                let name = String::from_utf8_lossy(dm.get(1).unwrap().as_bytes()).into_owned();
                let sym = symbol(&data[prev..start], dm.get(1).unwrap().as_bytes(), &r.link);
                facts.decls.entry(sym).or_default().push((name, sig));
            }
            prev = bm.end() + m.end();
        }
    }
    for dm in r.def.captures_iter(data) {
        let m = dm.get(0).unwrap();
        let open = m.end() - 1;
        if let Some(sig) = parse_sig(data, &masked, open) {
            let name = dm.get(1).unwrap().as_bytes();
            let sym = symbol(m.as_bytes(), name, &r.export);
            facts.defs.insert(sym, sig);
        }
    }
    facts
}
fn render_findings(facts: &CrateFacts, live: &HashSet<PathBuf>) -> Vec<String> {
    let mut found = Vec::new();
    for (symbol, sites) in &facts.decls {
        let Some((dpath, dsig)) = facts.defs.get(symbol) else {
            continue;
        };
        for site in sites {
            if site.sig.raw.contains("...") || dsig.raw.contains("...") {
                continue;
            }
            if !live.is_empty() && !live.contains(&site.resolved) {
                continue;
            }
            let a = nargs(&site.sig.masked);
            let b = nargs(&dsig.masked);
            if a != b {
                found.push((
                    "ARITY",
                    site.path.clone(),
                    site.name.clone(),
                    a,
                    b,
                    dpath.clone(),
                ));
            } else if site.sig.ret == Ret::Value && dsig.ret == Ret::Unit {
                found.push((
                    "RETVOID",
                    site.path.clone(),
                    site.name.clone(),
                    0,
                    0,
                    dpath.clone(),
                ));
            }
        }
    }
    found.sort();
    found.into_iter().map(|(kind,path,name,a,b,dpath)| {
        let path=path.display(); let dpath=dpath.display();
        if kind=="ARITY" {format!("{path}:1: [ARITY] {name}: declared with {a} parameter(s), defined with {b} in {dpath} — the caller and callee disagree about the call shape")}
        else {format!("{path}:1: [RETVOID] {name}: declared as returning a value, defined as returning nothing in {dpath} — the caller reads a register the callee never set")}
    }).collect()
}
fn finding_sort_key(line: &str) -> (u8, String, String, usize, usize, String) {
    let rank = if line.contains("[ARITY]") { 0 } else { 1 };
    let (path, rest) = line.split_once(":1: ").unwrap_or((line, ""));
    let name = rest
        .split_once("] ")
        .map(|(_, s)| s.split_once(':').map(|(n, _)| n).unwrap_or(s))
        .unwrap_or("");
    let (a, b) = if rank == 0 {
        let a = rest
            .split_once("declared with ")
            .and_then(|(_, s)| s.split_once(" parameter(s)"))
            .and_then(|(s, _)| s.parse().ok())
            .unwrap_or(0);
        let b = rest
            .split_once("defined with ")
            .and_then(|(_, s)| s.split_once(" in "))
            .and_then(|(s, _)| s.parse().ok())
            .unwrap_or(0);
        (a, b)
    } else {
        (0, 0)
    };
    let dpath = rest
        .rsplit_once(" in ")
        .map(|(_, s)| s.split_once(" — ").map(|(p, _)| p).unwrap_or(s))
        .unwrap_or("");
    (
        rank,
        path.to_owned(),
        name.to_owned(),
        a,
        b,
        dpath.to_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    const BAD_SHIM:&str="#[no_mangle] pub unsafe extern \"C\" fn ScsiGetNextSenseDescriptorByType(_buf: *mut u8, _len: u32, _ty: u8, found: *mut u8,) -> *mut u8 { core::ptr::null_mut() }";
    const BAD_DECL:&str="extern \"C\" { fn ScsiGetNextSenseDescriptorByType(start: *mut u8, start_len: u8, types: *const u8, type_count: u32, out_type: *mut u8, out_buf: *mut *mut u8, out_len: *mut u8) -> u8; }";
    #[test]
    fn original_scanner_control_bad_and_good() {
        let mut f = parse_facts(BAD_SHIM.as_bytes());
        let d = parse_facts(BAD_DECL.as_bytes());
        f.decls = d.decls;
        let mut cf = CrateFacts::default();
        for (sym, ss) in f.decls {
            for (name, sig) in ss {
                cf.decls.entry(sym.clone()).or_default().push(Site {
                    path: "src/caller.rs".into(),
                    resolved: "src/caller.rs".into(),
                    name,
                    sig,
                });
            }
        }
        for (sym, sig) in f.defs {
            cf.defs.insert(sym, ("src/shim.rs".into(), sig));
        }
        assert_eq!(render_findings(&cf,&HashSet::new()),vec!["src/caller.rs:1: [ARITY] ScsiGetNextSenseDescriptorByType: declared with 7 parameter(s), defined with 4 in src/shim.rs — the caller and callee disagree about the call shape"]);
        let good_shim="#[no_mangle] pub unsafe extern \"C\" fn Matching(_a: *mut u8, _b: u8, // comment comma, ->\n            _cb: Option<unsafe extern \"system\" fn(u32) -> u32>) -> u8 { 0 }";
        let good_decl="extern \"C\" { fn Matching(a: *mut u8, b: u8, /* comma, -> */ cb: Option<unsafe extern \"system\" fn(u32) -> u32>) -> u8; }";
        assert!(parse_facts(good_shim.as_bytes()).defs["Matching"]
            .masked
            .contains("Option"));
        assert_eq!(
            nargs(
                &parse_facts(good_decl.as_bytes()).decls["Matching"][0]
                    .1
                    .masked
            ),
            3
        );
        assert_eq!(
            nargs(&parse_facts(good_shim.as_bytes()).defs["Matching"].masked),
            3
        );
    }
    #[test]
    fn linkage_aliases_and_variadics_are_resolved() {
        let d = parse_facts(
            b"extern \"C\" { #[link_name = \"Real\"] fn Alias(a:u8, b:u8); fn Var(a:u8, ...); }",
        );
        let f=parse_facts(b"#[no_mangle] #[export_name = \"Real\"] pub extern \"C\" fn Impl(a:u8) {} #[no_mangle] pub extern \"C\" fn Var(a:u8) {}");
        assert!(d.decls.contains_key("Real"));
        assert!(f.defs.contains_key("Real"));
        assert!(d.decls["Var"][0].1.raw.contains("..."));
    }
    #[test]
    fn retvoid_only_flags_value_decl_to_unit_definition() {
        let d = parse_facts(b"extern \"C\" { fn A() -> u32; fn B(); }");
        let f=parse_facts(b"#[no_mangle] pub extern \"C\" fn A() {} #[no_mangle] pub extern \"C\" fn B() -> u32 { 0 }");
        let mut cf = CrateFacts::default();
        for (sym, sites) in d.decls {
            let entry = cf.decls.entry(sym).or_default();
            for (name, sig) in sites {
                entry.push(Site {
                    path: "src/lib.rs".into(),
                    resolved: "src/lib.rs".into(),
                    name,
                    sig,
                });
            }
        }
        for (sym, sig) in f.defs {
            cf.defs.insert(sym, ("src/lib.rs".into(), sig));
        }
        assert_eq!(render_findings(&cf, &HashSet::new()).len(), 1);
        assert!(render_findings(&cf, &HashSet::new())[0].contains("[RETVOID] A"));
    }
    #[test]
    fn unreachable_declaration_is_ignored_but_live_one_is_retained() {
        let d = parse_facts(b"extern \"C\" { fn F(a:u8); }");
        let f = parse_facts(b"#[no_mangle] pub extern \"C\" fn F() {}");
        let mut cf = CrateFacts::default();
        let sig = d.decls["F"][0].1.clone();
        cf.decls.entry("F".into()).or_default().push(Site {
            path: "src/dead.rs".into(),
            resolved: "src/dead.rs".into(),
            name: "F".into(),
            sig,
        });
        cf.defs
            .insert("F".into(), ("src/def.rs".into(), f.defs["F"].clone()));
        assert!(render_findings(&cf, &HashSet::from([PathBuf::from("src/live.rs")])).is_empty());
        assert_eq!(
            render_findings(&cf, &HashSet::from([PathBuf::from("src/dead.rs")])).len(),
            1
        );
    }
    #[test]
    fn corpus_fixture_uses_module_reachability() {
        let files = vec![
            SourceFile::new(
                "Cargo.toml".into(),
                "[package]\nname=\"fixture\"\nversion=\"0.0.0\"\nedition=\"2021\"\n".into(),
            ),
            SourceFile::new("src/lib.rs".into(), "pub mod caller; pub mod shim;".into()),
            SourceFile::new(
                "src/caller.rs".into(),
                "extern \"C\" { fn F(a:u8); }".into(),
            ),
            SourceFile::new(
                "src/shim.rs".into(),
                "#[no_mangle] pub extern \"C\" fn F() {}".into(),
            ),
            SourceFile::new("src/dead.rs".into(), "extern \"C\" { fn F(a:u8); }".into()),
        ];
        let corpus = Corpus::new(PathBuf::from("fixture"), files);
        let out = ExternArity.run(&corpus).unwrap();
        assert_eq!(out.len(), 1);
        assert!(out[0].starts_with("src/caller.rs:1: [ARITY] F:"));
    }
    #[test]
    fn lexical_mask_is_byte_stable_and_lifetimes_survive() {
        let src = "'a // λ\nfn x(a: &'a str, b: u8 /* -> , */) {}";
        let masked = mask_lexical(src.as_bytes());
        assert_eq!(masked.len(), src.len());
        assert!(String::from_utf8(masked).unwrap().contains("'a"));
    }
    #[test]
    fn no_mangle_prefilter_checks_all_nine_bytes() {
        assert!(contains_token(b"#[no_mangle] pub fn f() {}", b"no_mangle"));
        assert!(!contains_token(b"#[no_mangl] pub fn f() {}", b"no_mangle"));
    }
    #[test]
    fn byte_regex_word_classes_match_python_bytes_ascii() {
        let re = &regexes().decl;
        assert!(re.is_match(b"fn ascii_name()"));
        assert!(!re.is_match("fn naïve()".as_bytes()));
    }
}
