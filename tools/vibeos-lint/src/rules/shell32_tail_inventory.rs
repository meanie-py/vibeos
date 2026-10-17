use crate::{Corpus, Rule, SourceFile, SourceSet};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

pub struct Shell32TailInventory;
const CMAKE_FILES: &[&str] = &[
    "dll/win32/shell32/CMakeLists.txt",
    "dll/win32/shell32/shelldesktop/CMakeLists.txt",
    "dll/win32/shell32/shellmenu/CMakeLists.txt",
    "dll/win32/shell32/shellrecyclebin/CMakeLists.txt",
];
const INPUTS: &[SourceSet] = &[SourceSet::tree(
    "shell32-tail",
    &["dll/win32/shell32"],
    &["c", "cpp"],
    &["CMakeLists.txt"],
)
.with_inputs()];
const LEDGER: &str = "tools/rustconv/data/shell32_cpp_tail.json";
#[derive(Clone, Debug, PartialEq, Eq)]
struct Unit {
    path: String,
    language: String,
    cohort: String,
    status: String,
}
#[derive(Default, Debug)]
struct Report {
    live: BTreeSet<String>,
    expected_live: BTreeSet<String>,
    unclassified: BTreeSet<String>,
    unexpectedly_live: BTreeSet<String>,
    expected_but_absent: BTreeSet<String>,
    missing_files: BTreeSet<String>,
    duplicates: BTreeSet<String>,
    cpp_files: usize,
    cpp_lines: usize,
    c_files: usize,
    c_lines: usize,
}
impl Report {
    fn ok(&self) -> bool {
        self.unclassified.is_empty()
            && self.unexpectedly_live.is_empty()
            && self.expected_but_absent.is_empty()
            && self.missing_files.is_empty()
            && self.duplicates.is_empty()
    }
}

// The checked-in ledger uses JSON only as a serialization format. A small complete JSON
// value parser keeps the rule hermetic and rejects malformed ledgers just as Python's json
// loader does, without bringing a runtime dependency into the lint crate.
#[derive(Debug)]
enum Json {
    Null,
    Bool,
    Number,
    String(String),
    Array(Vec<Json>),
    Object(BTreeMap<String, Json>),
}
struct JParser<'a> {
    s: &'a str,
    i: usize,
}
impl<'a> JParser<'a> {
    fn ws(&mut self) {
        while self.i < self.s.len() && self.s.as_bytes()[self.i].is_ascii_whitespace() {
            self.i += 1
        }
    }
    fn byte(&self) -> Option<u8> {
        self.s.as_bytes().get(self.i).copied()
    }
    fn value(&mut self) -> Result<Json, String> {
        self.ws();
        match self.byte() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => Ok(Json::String(self.string()?)),
            Some(b't') if self.s[self.i..].starts_with("true") => {
                self.i += 4;
                Ok(Json::Bool)
            }
            Some(b'f') if self.s[self.i..].starts_with("false") => {
                self.i += 5;
                Ok(Json::Bool)
            }
            Some(b'n') if self.s[self.i..].starts_with("null") => {
                self.i += 4;
                Ok(Json::Null)
            }
            Some(b'-' | b'0'..=b'9') => {
                self.number()?;
                Ok(Json::Number)
            }
            _ => Err(format!("invalid JSON at byte {}", self.i)),
        }
    }
    fn string(&mut self) -> Result<String, String> {
        if self.byte() != Some(b'"') {
            return Err("expected JSON string".into());
        }
        self.i += 1;
        let mut o = String::new();
        while let Some(b) = self.byte() {
            match b {
                b'"' => {
                    self.i += 1;
                    return Ok(o);
                }
                b'\\' => {
                    self.i += 1;
                    let e = self.byte().ok_or("trailing JSON escape")?;
                    self.i += 1;
                    match e {
                        b'"' => o.push('"'),
                        b'\\' => o.push('\\'),
                        b'/' => o.push('/'),
                        b'b' => o.push('\u{8}'),
                        b'f' => o.push('\u{c}'),
                        b'n' => o.push('\n'),
                        b'r' => o.push('\r'),
                        b't' => o.push('\t'),
                        b'u' => {
                            let cp = self.hex4()?;
                            let scalar = if (0xd800..=0xdbff).contains(&cp) {
                                if self.s.as_bytes().get(self.i..self.i + 2) != Some(b"\\u") {
                                    return Err("missing low surrogate".into());
                                }
                                self.i += 2;
                                let lo = self.hex4()?;
                                if !(0xdc00..=0xdfff).contains(&lo) {
                                    return Err("invalid low surrogate".into());
                                }
                                0x10000 + ((cp - 0xd800) << 10) + (lo - 0xdc00)
                            } else {
                                cp
                            };
                            o.push(char::from_u32(scalar).ok_or("invalid unicode scalar")?);
                        }
                        _ => return Err("invalid JSON escape".into()),
                    }
                }
                0..=0x1f => return Err("control byte in JSON string".into()),
                _ => {
                    let ch = self.s[self.i..].chars().next().ok_or("invalid UTF-8")?;
                    o.push(ch);
                    self.i += ch.len_utf8();
                }
            }
        }
        Err("unterminated JSON string".into())
    }
    fn hex4(&mut self) -> Result<u32, String> {
        if self.i + 4 > self.s.len() {
            return Err("short unicode escape".into());
        }
        let n = u32::from_str_radix(&self.s[self.i..self.i + 4], 16)
            .map_err(|_| "bad unicode escape")?;
        self.i += 4;
        Ok(n)
    }
    fn number(&mut self) -> Result<(), String> {
        let start = self.i;
        while self
            .byte()
            .is_some_and(|b| b.is_ascii_digit() || b"-+.eE".contains(&b))
        {
            self.i += 1
        }
        if self.i == start {
            Err("bad JSON number".into())
        } else {
            Ok(())
        }
    }
    fn array(&mut self) -> Result<Json, String> {
        self.i += 1;
        let mut a = vec![];
        self.ws();
        if self.byte() == Some(b']') {
            self.i += 1;
            return Ok(Json::Array(a));
        }
        loop {
            a.push(self.value()?);
            self.ws();
            match self.byte() {
                Some(b',') => self.i += 1,
                Some(b']') => {
                    self.i += 1;
                    break;
                }
                _ => return Err(format!("expected , or ] at byte {}", self.i)),
            }
        }
        Ok(Json::Array(a))
    }
    fn object(&mut self) -> Result<Json, String> {
        self.i += 1;
        let mut m = BTreeMap::new();
        self.ws();
        if self.byte() == Some(b'}') {
            self.i += 1;
            return Ok(Json::Object(m));
        }
        loop {
            self.ws();
            let k = self.string()?;
            self.ws();
            if self.byte() != Some(b':') {
                return Err("expected : in object".into());
            }
            self.i += 1;
            let v = self.value()?;
            m.insert(k, v);
            self.ws();
            match self.byte() {
                Some(b',') => self.i += 1,
                Some(b'}') => {
                    self.i += 1;
                    break;
                }
                _ => return Err(format!("expected , or }} at byte {}", self.i)),
            }
        }
        Ok(Json::Object(m))
    }
}
fn parse_ledger(s: &str) -> Result<(Vec<Unit>, BTreeSet<String>), String> {
    let mut p = JParser { s, i: 0 };
    let root = p.value()?;
    p.ws();
    if p.i != s.len() {
        return Err("trailing JSON data".into());
    }
    let Json::Object(obj) = root else {
        return Err("ledger root must be object".into());
    };
    let valid_status = ["active", "removed", "sanctioned"];
    let mut units = vec![];
    for status in valid_status {
        if let Some(Json::Object(cohorts)) = obj.get(status) {
            for (cohort, paths) in cohorts {
                if let Json::Array(paths) = paths {
                    for path in paths {
                        if let Json::String(path) = path {
                            units.push(Unit {
                                path: path.clone(),
                                language: if path.ends_with(".cpp") { "cpp" } else { "c" }.into(),
                                cohort: cohort.clone(),
                                status: status.into(),
                            });
                        }
                    }
                }
            }
        }
    }
    if let Some(Json::Array(rows)) = obj.get("unit") {
        let mut seen = BTreeSet::new();
        let mut duplicates = BTreeSet::new();
        for row in rows {
            let Json::Object(m) = row else { continue };
            let get = |k: &str| {
                m.get(k)
                    .and_then(|v| {
                        if let Json::String(s) = v {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default()
            };
            let path = get("path");
            let language = get("language");
            let cohort = get("cohort");
            let status = get("status");
            if !["c", "cpp"].contains(&language.as_str()) {
                return Err(format!("{path}: invalid language {language:?}"));
            }
            if !valid_status.contains(&status.as_str()) {
                return Err(format!("{path}: invalid status {status:?}"));
            }
            let suffix = if language == "cpp" { ".cpp" } else { ".c" };
            if !path.ends_with(suffix) {
                return Err(format!(
                    "{path}: language {language:?} disagrees with suffix"
                ));
            }
            if !seen.insert(path.clone()) {
                duplicates.insert(path.clone());
            }
            units.push(Unit {
                path,
                language,
                cohort,
                status,
            });
        }
        Ok((units, duplicates))
    } else {
        Ok((units, BTreeSet::new()))
    }
}
fn normalized(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            x => out.push(x.as_os_str()),
        }
    }
    out
}
fn source_tokens(text: &str) -> Vec<String> {
    let mut found = vec![];
    for raw in text.split('\n') {
        let code = raw.split('#').next().unwrap_or("");
        for token in code.replace('(', " ").replace(')', " ").split_whitespace() {
            if token.contains('$') || !(token.ends_with(".c") || token.ends_with(".cpp")) {
                continue;
            }
            found.push(token.to_string())
        }
    }
    found
}
fn rel_live(c: &Corpus, cmake: &SourceFile, token: &str) -> Option<String> {
    let joined = normalized(&cmake.path.parent().unwrap_or(Path::new("")).join(token));
    if joined.is_absolute() || joined.starts_with("..") {
        return None;
    }
    let resolved = c
        .files
        .iter()
        .find(|f| f.path == joined || f.resolved_path() == joined.as_path());
    let f = resolved?;
    let path = f.resolved_path();
    if path.is_absolute() {
        path.strip_prefix(&c.root)
            .ok()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
    } else {
        Some(path.to_string_lossy().replace('\\', "/"))
    }
}
fn line_count(s: &str) -> usize {
    s.lines().count()
}
fn audit(c: &Corpus) -> Result<Report, String> {
    audit_with_cmake(c, CMAKE_FILES)
}
fn audit_with_cmake(c: &Corpus, cmake_files: &[&str]) -> Result<Report, String> {
    let ledger = c
        .files
        .iter()
        .find(|f| f.path == Path::new(LEDGER))
        .ok_or_else(|| format!("missing required input {LEDGER}"))?;
    let (units, duplicates) = parse_ledger(&ledger.text)?;
    let mut live = BTreeSet::new();
    for path in cmake_files {
        let cmake = c
            .files
            .iter()
            .find(|f| f.path == Path::new(path))
            .ok_or_else(|| format!("missing required input {path}"))?;
        for token in source_tokens(&cmake.text) {
            if let Some(rel) = rel_live(c, cmake, &token) {
                live.insert(rel);
            }
        }
    }
    let by_path: BTreeMap<_, _> = units.iter().map(|u| (u.path.as_str(), u)).collect();
    let expected_live: BTreeSet<_> = units
        .iter()
        .filter(|u| u.status == "active" || u.status == "sanctioned")
        .map(|u| u.path.clone())
        .collect();
    let removed: BTreeSet<_> = units
        .iter()
        .filter(|u| u.status == "removed")
        .map(|u| u.path.clone())
        .collect();
    let unclassified = live
        .difference(&by_path.keys().map(|p| (*p).to_string()).collect())
        .cloned()
        .collect();
    let unexpectedly_live = live.intersection(&removed).cloned().collect();
    let expected_but_absent = expected_live.difference(&live).cloned().collect();
    let present: BTreeSet<_> = c
        .files
        .iter()
        .map(|f| f.resolved_path().to_string_lossy().replace('\\', "/"))
        .collect();
    let missing_files = units
        .iter()
        .filter(|u| !present.contains(&u.path))
        .map(|u| u.path.clone())
        .collect();
    let cpp: Vec<_> = live
        .iter()
        .filter(|p| p.ends_with(".cpp"))
        .cloned()
        .collect();
    let cs: Vec<_> = live.iter().filter(|p| p.ends_with(".c")).cloned().collect();
    let count = |p: &String| {
        c.files
            .iter()
            .find(|f| f.resolved_path().to_string_lossy().replace('\\', "/") == *p)
            .map(|f| line_count(&f.text))
            .unwrap_or(0)
    };
    let cpp_lines = cpp.iter().map(|p| count(p)).sum();
    let c_lines = cs.iter().map(|p| count(p)).sum();
    Ok(Report {
        live,
        expected_live,
        unclassified,
        unexpectedly_live,
        expected_but_absent,
        missing_files,
        duplicates,
        cpp_files: cpp.len(),
        cpp_lines,
        c_files: cs.len(),
        c_lines,
    })
}
impl Rule for Shell32TailInventory {
    fn name(&self) -> &'static str {
        "shell32_tail_inventory"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn extra_files(&self) -> &'static [&'static str] {
        &[LEDGER]
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let r = audit(c)?;
        let mut out = vec![];
        if !r.ok() {
            for (label, paths) in [
                ("UNCLASSIFIED", &r.unclassified),
                ("STILL_LIVE", &r.unexpectedly_live),
                ("ABSENT", &r.expected_but_absent),
                ("MISSING", &r.missing_files),
                ("DUPLICATE", &r.duplicates),
            ] {
                for p in paths {
                    out.push(format!("{label}: {p}"));
                }
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const MINI_CMAKE: &[&str] = &[
        "dll/win32/shell32/CMakeLists.txt",
        "dll/win32/shell32/sub/CMakeLists.txt",
    ];
    fn mini() -> Corpus {
        let entries = [
            ("dll/win32/shell32/CMakeLists.txt", "list(APPEND SOURCE\n one.cpp\n # dead.cpp\n ${CMAKE_CURRENT_BINARY_DIR}/gen.c)\nlist(APPEND PCH_SKIP_SOURCE shim.c)\n"),
            ("dll/win32/shell32/sub/CMakeLists.txt", "list(APPEND SOURCE two.cpp)\n"),
            ("dll/win32/shell32/shelldesktop/CMakeLists.txt", ""),
            ("dll/win32/shell32/shellmenu/CMakeLists.txt", ""),
            ("dll/win32/shell32/shellrecyclebin/CMakeLists.txt", ""),
            ("dll/win32/shell32/one.cpp", "// source\n"),
            ("dll/win32/shell32/sub/two.cpp", "// source\n// line\n"),
            ("dll/win32/shell32/shim.c", "// source\n"),
            (LEDGER, r#"{"unit":[{"path":"dll/win32/shell32/one.cpp","language":"cpp","cohort":"pilot","status":"active"},{"path":"dll/win32/shell32/sub/two.cpp","language":"cpp","cohort":"pilot","status":"active"},{"path":"dll/win32/shell32/shim.c","language":"c","cohort":"shim","status":"sanctioned"}]}"#),
        ];
        Corpus::new(
            PathBuf::from("."),
            entries
                .into_iter()
                .map(|(p, t)| SourceFile::new(p.into(), t.into()))
                .collect(),
        )
    }
    #[test]
    fn original_extraction_fixture_ignores_comment_variable_generated() {
        let c = mini();
        let r = audit_with_cmake(&c, MINI_CMAKE).unwrap();
        assert_eq!(
            r.live,
            BTreeSet::from([
                "dll/win32/shell32/one.cpp".into(),
                "dll/win32/shell32/sub/two.cpp".into(),
                "dll/win32/shell32/shim.c".into()
            ])
        );
    }
    #[test]
    fn original_inventory_fixture_matches_counts() {
        let r = audit_with_cmake(&mini(), MINI_CMAKE).unwrap();
        assert!(r.ok());
        assert_eq!(
            (r.cpp_files, r.cpp_lines, r.c_files, r.c_lines),
            (2, 3, 1, 1)
        );
    }
    #[test]
    fn stale_removed_unclassified_and_absent_are_distinct() {
        let mut c = mini();
        let ledger = r#"{"unit":[{"path":"dll/win32/shell32/one.cpp","language":"cpp","cohort":"pilot","status":"removed"},{"path":"dll/win32/shell32/shim.c","language":"c","cohort":"shim","status":"active"},{"path":"dll/win32/shell32/missing.cpp","language":"cpp","cohort":"pilot","status":"active"}]}"#;
        c.files.retain(|f| {
            f.path != Path::new(LEDGER) && f.path != Path::new("dll/win32/shell32/shim.c")
        });
        c.files.push(SourceFile::new(LEDGER.into(), ledger.into()));
        let r = audit_with_cmake(&c, MINI_CMAKE).unwrap();
        assert!(r.unexpectedly_live.contains("dll/win32/shell32/one.cpp"));
        assert!(r.unclassified.contains("dll/win32/shell32/sub/two.cpp"));
        assert!(r.expected_but_absent.contains("dll/win32/shell32/shim.c"));
        assert!(r.missing_files.contains("dll/win32/shell32/missing.cpp"));
    }
    #[test]
    fn json_rejects_wrong_language_suffix_and_records_duplicates() {
        let (u,d)=parse_ledger(r#"{"unit":[{"path":"x.c","language":"c","cohort":"a","status":"active"},{"path":"x.c","language":"c","cohort":"b","status":"removed"}]}"#).unwrap();
        assert_eq!(u.len(), 2);
        assert!(d.contains("x.c"));
        assert!(parse_ledger(
            r#"{"unit":[{"path":"x.cpp","language":"c","cohort":"a","status":"active"}]}"#
        )
        .is_err());
    }
    #[test]
    fn check_lines_are_finding_only() {
        let mut c = mini();
        let ledger = c
            .files
            .iter_mut()
            .find(|f| f.path == Path::new(LEDGER))
            .unwrap();
        ledger.text = ledger
            .text
            .replace("\"status\":\"active\"", "\"status\":\"removed\"");
        let out = Shell32TailInventory.run(&c).unwrap();
        assert!(out.contains(&"STILL_LIVE: dll/win32/shell32/one.cpp".to_string()));
    }
}
