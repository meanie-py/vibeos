//! The textual C-twin oracle shared by source rules, ported from tools/c_twin.py.
//! Inputs are already-loaded source snapshots; tracking and traversal belong to Corpus.
use crate::SourceFile;
use rayon::prelude::*;
use regex::Regex;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub path: PathBuf,
    pub line: usize,
    pub body: String,
    pub body_start: usize,
    pub params: String,
    pub ret: String,
}
static C_IGNORED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)//[^\n]*|/\*.*?\*/|"(?:\\.|[^"\\\n])*"|'(?:\\.|[^'\\\n])+'"#).unwrap()
});
static RS_IGNORED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)//[^\n]*|/\*.*?\*/|b?"(?:\\.|[^"\\])*"|b?'(?:\\.|[^'\\\n])'"#).unwrap()
});
static RS_FN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>{]*>)?\s*\(").unwrap());
static NAME_PAREN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b([A-Za-z_]\w*)\s*\(").unwrap());
static C_DECOR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:WINAPIV?|NTAPI|APIENTRY|FASTCALL|STDCALL|CALLBACK|PASCAL|CDECL|EXTERN_C|NTSYSAPI|NTKERNELAPI|NTHALAPI|W32KAPI|DECLSPEC_\w+|INIT_FUNCTION|CODE_SEG|static|inline|extern|__\w+|_\w+_|_In\w*|_Out\w*|_Must_\w+|_Success_|_Check_return_|_IRQL_\w+)$").unwrap()
});
static PARENS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\([^()]*\)").unwrap());
static RETURN_WORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[A-Za-z_]\w*|\*").unwrap());
static WORD_SPLIT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^a-z0-9]+").unwrap());
const C_KEYWORDS: &[&str] = &[
    "if", "while", "for", "switch", "return", "sizeof", "do", "else", "case",
];
fn blank(text: &str, pattern: &Regex) -> String {
    let mut bytes = text.as_bytes().to_vec();
    for m in pattern.find_iter(text) {
        for b in &mut bytes[m.range()] {
            if *b != b'\n' {
                *b = b' ';
            }
        }
    }
    String::from_utf8(bytes).expect("masking entire matches preserves UTF-8")
}
pub fn blank_c(text: &str) -> String {
    blank(text, &C_IGNORED)
}
pub fn blank_rust(text: &str) -> String {
    blank(text, &RS_IGNORED)
}
/// The index just past the matching `}`, or None for an incomplete body.
pub fn match_brace(text: &str, open_at: usize) -> Option<usize> {
    matching(text, open_at, b'{', b'}').map(|at| at + 1)
}
fn matching(text: &str, open_at: usize, opener: u8, closer: u8) -> Option<usize> {
    let mut depth: isize = 0;
    for (offset, &b) in text.as_bytes().get(open_at..)?.iter().enumerate() {
        if b == opener {
            depth += 1;
        } else if b == closer {
            depth -= 1;
            if depth == 0 {
                return Some(open_at + offset);
            }
        }
    }
    None
}
fn return_type(text: &str, name_at: usize) -> String {
    let prefix = &text[..name_at];
    let lo = prefix.rfind([';', '}', '{']).map_or(0, |at| at + 1);
    let seg = text[lo..name_at]
        .split('\n')
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let seg = PARENS.replace_all(&seg, " ");
    RETURN_WORD
        .find_iter(&seg)
        .filter(|word| !C_DECOR.is_match(word.as_str()))
        .last()
        .map_or(String::new(), |word| word.as_str().into())
}
pub fn rust_functions(text: &str, path: &Path) -> Vec<Function> {
    let newlines: Vec<_> = text.match_indices('\n').map(|(at, _)| at).collect();
    let mut out = Vec::new();
    for m in RS_FN.captures_iter(text) {
        let whole = m.get(0).unwrap();
        let Some(close) = matching(text, whole.end() - 1, b'(', b')') else {
            continue;
        };
        let Some(brace_offset) = text[close..].find('{') else {
            continue;
        };
        let brace = close + brace_offset;
        if text[close..]
            .find(';')
            .is_some_and(|semi| close + semi < brace)
        {
            continue;
        }
        let Some(end) = match_brace(text, brace) else {
            continue;
        };
        out.push(Function {
            name: m[1].into(),
            path: path.into(),
            line: newlines.partition_point(|&at| at < whole.start()) + 1,
            body: text[brace..end].into(),
            body_start: brace,
            params: text[whole.end()..close].into(),
            ret: return_type(text, whole.start()),
        });
    }
    out
}
pub fn enclosing(functions: &[Function], at: usize) -> Option<&Function> {
    functions
        .iter()
        .filter(|f| f.body_start <= at && at < f.body_start + f.body.len())
        .max_by_key(|f| f.body_start)
}
fn definition_brace(text: &str, close: usize) -> Option<usize> {
    let at = text.as_bytes()[close + 1..]
        .iter()
        .position(|b| b"{;=,)".contains(b))?
        + close
        + 1;
    (text.as_bytes()[at] == b'{').then_some(at)
}
pub fn c_functions(text: &str, path: &Path, names: &HashSet<String>) -> Vec<Function> {
    if names.is_empty() {
        return Vec::new();
    }
    // All names harvested by defined_names use this identifier grammar. Match
    // once and filter names rather than compiling a large alternation per file.
    // Preserve the public helper's behavior for unusual explicitly supplied names.
    let fallback = if names.iter().all(|name| {
        let mut bytes = name.bytes();
        bytes
            .next()
            .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
            && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
    }) {
        None
    } else {
        let mut sorted: Vec<_> = names.iter().collect();
        sorted.sort();
        let alternatives = sorted
            .into_iter()
            .map(|name| regex::escape(name))
            .collect::<Vec<_>>()
            .join("|");
        Some(Regex::new(&format!(r"\b({alternatives})\s*\(")).expect("escaped C function names"))
    };
    let pattern = fallback.as_ref().unwrap_or(&NAME_PAREN);
    let newlines: Vec<_> = text.match_indices('\n').map(|(at, _)| at).collect();
    let mut out = Vec::new();
    for m in pattern.captures_iter(text) {
        let name = &m[1];
        if !names.contains(name) {
            continue;
        }
        if C_KEYWORDS.contains(&name) {
            continue;
        }
        let whole = m.get(0).unwrap();
        let before: String = text[..whole.start()]
            .chars()
            .rev()
            .take(3)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        let before = before.trim_end();
        if [".", "->", "=", ",", "(", "!", "&", "|", "+", "-", "?", ":"]
            .iter()
            .any(|suffix| before.ends_with(suffix))
        {
            continue;
        }
        let Some(close) = matching(text, whole.end() - 1, b'(', b')') else {
            continue;
        };
        let Some(brace) = definition_brace(text, close) else {
            continue;
        };
        let Some(end) = match_brace(text, brace) else {
            continue;
        };
        out.push(Function {
            name: name.into(),
            path: path.into(),
            line: newlines.partition_point(|&at| at < whole.start()) + 1,
            body: text[brace..end].into(),
            body_start: brace,
            params: text[whole.end()..close].into(),
            ret: return_type(text, whole.start()),
        });
    }
    out
}
/// Definitions at file scope. Skipping bodies prevents calls from seeding the oracle.
pub fn defined_names(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut pos = 0;
    let mut depth: isize = 0;
    while let Some(m) = NAME_PAREN.captures_at(text, pos) {
        let whole = m.get(0).unwrap();
        let gap = &text[pos..whole.start()];
        depth += gap.bytes().filter(|&b| b == b'{').count() as isize
            - gap.bytes().filter(|&b| b == b'}').count() as isize;
        depth = depth.max(0);
        pos = whole.end();
        if depth != 0 || C_KEYWORDS.contains(&&m[1]) {
            continue;
        }
        let Some(close) = matching(text, whole.end() - 1, b'(', b')') else {
            continue;
        };
        if let Some(brace) = definition_brace(text, close) {
            let Some(end) = match_brace(text, brace) else {
                break;
            };
            out.push(m[1].into());
            pos = end;
        } else {
            pos = close + 1;
        }
    }
    out
}
fn crate_of(path: &Path) -> String {
    let mut parts = path.components();
    while let Some(part) = parts.next() {
        if part.as_os_str() == "rust" {
            return parts.next().map_or(String::new(), |p| {
                p.as_os_str().to_string_lossy().into_owned()
            });
        }
    }
    String::new()
}
fn words(text: &str, excluded: &[&str]) -> HashSet<String> {
    WORD_SPLIT
        .split(&text.to_lowercase())
        .filter(|word| !excluded.contains(word))
        .map(str::to_owned)
        .collect()
}
pub fn affinity(rust_path: &Path, twin: &Function) -> usize {
    let mut common = words(
        &rust_path.to_string_lossy(),
        &["", "src", "rust", "rs", "sdk", "lib"],
    );
    common.extend(words(&crate_of(rust_path), &["", "rust"]));
    let c_words = words(&twin.path.to_string_lossy(), &[]);
    common.intersection(&c_words).count()
}
pub fn same_module(rust_path: &Path, twin: &Function) -> bool {
    const EXCLUDED: &[&str] = &["", "rust", "sys", "src", "lib", "sdk"];
    let mut crate_words = words(&crate_of(rust_path), EXCLUDED);
    if crate_words.contains("win32k") {
        crate_words.insert("win32ss".into());
    }
    if crate_words.contains("hal") {
        crate_words.insert("halx86".into());
    }
    !crate_words.is_disjoint(&words(&twin.path.to_string_lossy(), EXCLUDED))
}
pub fn prefer<'a>(rust_path: &Path, twins: &'a [Function]) -> Vec<&'a Function> {
    let mut out: Vec<_> = twins.iter().collect();
    out.sort_by(|a, b| {
        affinity(rust_path, b)
            .cmp(&affinity(rust_path, a))
            .then_with(|| a.path.cmp(&b.path))
    });
    out
}

/// A complete, immutable-snapshot C oracle plus a cache for known-file lookups.
/// `from_files` accepts only tracked sources supplied by the owner of the corpus.
/// Definitions are gated by defined_names before c_functions, as in Python's load().
pub struct CIndex {
    definitions: HashMap<String, Vec<Function>>,
    blanked: BTreeMap<PathBuf, String>,
    local: Mutex<HashMap<(PathBuf, String), Vec<Function>>>,
}
impl CIndex {
    pub fn from_files<'a>(files: impl IntoIterator<Item = &'a SourceFile>) -> Self {
        let mut blanked = BTreeMap::new();
        for file in files {
            if file.path.starts_with("sdk/lib/rust")
                || !file
                    .path
                    .extension()
                    .is_some_and(|ext| ext == "c" || ext == "cpp")
            {
                continue;
            }
            // Python strips REPO from C paths before computing affinity. Keep
            // snapshot keys relative so the checkout's own name cannot pair
            // unrelated functions merely because both live in that checkout.
            blanked.insert(file.path.clone(), blank_c(&file.text));
        }
        let parsed: Vec<Vec<Function>> = blanked
            .par_iter()
            .map(|(path, text)| {
                let names = defined_names(text).into_iter().collect();
                c_functions(text, path, &names)
            })
            .collect();
        let mut definitions: HashMap<String, Vec<Function>> = HashMap::new();
        for functions in parsed {
            for function in functions {
                definitions
                    .entry(function.name.clone())
                    .or_default()
                    .push(function);
            }
        }
        Self {
            definitions,
            blanked,
            local: Mutex::new(HashMap::new()),
        }
    }
    pub fn get(&self, name: &str) -> &[Function] {
        self.definitions.get(name).map_or(&[], Vec::as_slice)
    }
    /// Match Python's local lookup: unlike global get, this intentionally has no
    /// defined_names gate, because its caller has already identified the C file.
    pub fn in_file(&self, path: &Path, name: &str) -> Vec<Function> {
        let key = (path.to_owned(), name.to_owned());
        self.local
            .lock()
            .expect("C local index poisoned")
            .entry(key)
            .or_insert_with(|| {
                self.blanked
                    .get(path)
                    .filter(|text| text.contains(name))
                    .map_or_else(Vec::new, |text| {
                        c_functions(text, path, &HashSet::from([name.to_owned()]))
                    })
            })
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn function(path: &str) -> Function {
        Function {
            name: "DriverEntry".into(),
            path: path.into(),
            line: 1,
            body: "{}".into(),
            body_start: 0,
            params: String::new(),
            ret: String::new(),
        }
    }
    #[test]
    fn masks_preserve_offsets_and_newlines() {
        let c = "// é\nBOOL Foo() { return '\\n'; /* two\nlines */ }";
        let masked = blank_c(c);
        assert_eq!(masked.len(), c.len());
        assert_eq!(
            masked
                .match_indices('\n')
                .map(|(i, _)| i)
                .collect::<Vec<_>>(),
            c.match_indices('\n').map(|(i, _)| i).collect::<Vec<_>>()
        );
        assert!(!masked.contains("é"));
        assert!(!blank_rust("b\"fn Fake() {}\"; b'x'; fn Real() {}").contains("Fake"));
    }
    #[test]
    fn definitions_skip_calls_prototypes_and_nested_bodies() {
        let text = blank_c("BOOL NTAPI Foo(int a) { Bar(a); if (a) { Baz(); } }\nvoid Prototype();\nvoid Bar(int a) { Foo(a); }\nstruct S { int (*member)(int); };\n");
        assert_eq!(defined_names(&text), vec!["Foo", "Bar"]);
        let names = HashSet::from(["Foo".into(), "Bar".into(), "Prototype".into()]);
        let parsed = c_functions(&text, Path::new("dll/example.c"), &names);
        assert_eq!(
            parsed
                .iter()
                .map(|f| (f.name.as_str(), f.ret.as_str()))
                .collect::<Vec<_>>(),
            vec![("Foo", "BOOL"), ("Bar", "void")]
        );
        assert_eq!(parsed[0].params, "int a");
        assert_eq!(parsed[0].body, "{ Bar(a); if (a) { Baz(); } }");
    }
    #[test]
    fn c_return_type_annotations_and_pointer_returns() {
        let source = "#define NOISE x\nBOOL\nCODE_SEG(\"INIT\") NTAPI _In_reads_(n) Foo(int n) {}\nchar * WINAPI Ptr(void) {}";
        let funcs = c_functions(
            &blank_c(source),
            Path::new("a.c"),
            &HashSet::from(["Foo".into(), "Ptr".into()]),
        );
        assert_eq!(
            funcs.iter().map(|f| f.ret.as_str()).collect::<Vec<_>>(),
            vec!["BOOL", "*"]
        );
    }
    #[test]
    fn rust_declarations_nested_functions_and_enclosing() {
        let text = blank_rust("extern \"C\" { fn Prototype(a: u32); }\nfn Outer<T>(a: T) { fn Inner() { call(); } }\nfn Broken( {");
        let functions = rust_functions(&text, Path::new("sdk/lib/rust/test-rust/src/lib.rs"));
        assert_eq!(
            functions
                .iter()
                .map(|f| f.name.as_str())
                .collect::<Vec<_>>(),
            vec!["Outer", "Inner"]
        );
        assert_eq!(
            enclosing(&functions, text.find("call").unwrap())
                .unwrap()
                .name,
            "Inner"
        );
        assert!(enclosing(&functions, 0).is_none());
        assert_eq!(match_brace("{ x { y } }", 0), Some(11));
        assert_eq!(match_brace("{", 0), None);
    }
    #[test]
    fn preference_uses_stable_path_ties_and_module_aliases() {
        let rust = Path::new("sdk/lib/rust/win32k-rust/src/main.rs");
        let twins = vec![
            function("drivers/bus/acpi/main.c"),
            function("win32ss/main.c"),
            function("win32ss/other/main.c"),
        ];
        assert!(!same_module(rust, &twins[0]));
        assert!(same_module(rust, &twins[1]));
        assert!(same_module(
            Path::new("sdk/lib/rust/hal-rust/src/lib.rs"),
            &function("hal/halx86/generic.c")
        ));
        let order = prefer(rust, &twins);
        // main adds affinity but cannot establish same_module on its own.
        assert_eq!(order[0].path, Path::new("drivers/bus/acpi/main.c"));
        let tied = vec![function("dll/z.c"), function("dll/a.c")];
        assert_eq!(
            prefer(Path::new("sdk/lib/rust/unknown/src/lib.rs"), &tied)[0].path,
            Path::new("dll/a.c")
        );
    }
    #[test]
    fn global_index_keeps_file_scope_gate() {
        let file = SourceFile::new(
            "dll/a.c".into(),
            "void Outer() { callback() { } }\nvoid Real() {}".into(),
        );
        let index = CIndex::from_files([&file]);
        assert!(index.get("callback").is_empty());
        assert_eq!(index.get("Real").len(), 1);
        assert_eq!(
            affinity(
                Path::new("/checkout/sdk/lib/rust/unknown/src/lib.rs"),
                &index.get("Real")[0]
            ),
            0
        );
        assert_eq!(index.in_file(Path::new("dll/a.c"), "callback").len(), 1);
    }
}
