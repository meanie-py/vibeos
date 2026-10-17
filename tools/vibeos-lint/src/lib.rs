//! Statically linked lint plugins over an immutable, shared source snapshot.
mod integers;
pub mod lexical;
pub mod rules;

use rayon::prelude::*;
use regex::Regex;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

pub trait Rule: Sync {
    fn name(&self) -> &'static str;
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String>;
    fn needs_c_sources(&self) -> bool {
        false
    }
    fn needs_sdk_rust(&self) -> bool {
        false
    }
    fn needs_cdecl_sources(&self) -> bool {
        false
    }
}

pub struct SourceFile {
    pub path: PathBuf,
    pub text: String,
    resolved: Option<PathBuf>,
    scopes: u8,
    noise: OnceLock<Arc<str>>,
    lines: Vec<usize>,
    views: Mutex<HashMap<(String, Option<String>, bool), Arc<str>>>,
}

impl SourceFile {
    pub fn new(path: PathBuf, text: String) -> Self {
        let mut lines = vec![0];
        lines.extend(text.match_indices('\n').map(|(at, _)| at + 1));
        Self {
            path,
            text,
            resolved: None,
            scopes: u8::MAX,
            noise: OnceLock::new(),
            lines,
            views: Mutex::new(HashMap::new()),
        }
    }

    /// Some legacy scanners resolve symlinked file names for diagnostics while
    /// others intentionally retain lexical paths. Capture both in the snapshot.
    pub fn resolved_path(&self) -> &Path {
        self.resolved.as_deref().unwrap_or(&self.path)
    }

    /// Legacy rules disagree about which lexemes to mask and about whether
    /// offsets count UTF-8 bytes or Unicode characters. Cache each exact view;
    /// equivalent users share its allocation and never tokenize it again.
    pub fn masked(&self, regex: &Regex, cutoff: Option<&str>, characters: bool) -> Arc<str> {
        let key = (
            regex.as_str().to_owned(),
            cutoff.map(str::to_owned),
            characters,
        );
        let mut views = self.views.lock().unwrap();
        views
            .entry(key)
            .or_insert_with(|| {
                let raw = cutoff
                    .and_then(|marker| self.text.split_once(marker).map(|p| p.0))
                    .unwrap_or(&self.text);
                regex
                    .replace_all(raw, |caps: &regex::Captures<'_>| {
                        let text = caps.get(0).unwrap().as_str();
                        if characters {
                            text.chars()
                                .map(|c| if c == '\n' { '\n' } else { ' ' })
                                .collect::<String>()
                        } else {
                            text.bytes()
                                .map(|b| if b == b'\n' { '\n' } else { ' ' })
                                .collect::<String>()
                        }
                    })
                    .into_owned()
                    .into()
            })
            .clone()
    }

    pub fn rust_noise(&self) -> &str {
        self.noise
            .get_or_init(|| lexical::strip_noise(&self.text).into())
    }

    pub fn line(&self, byte: usize) -> usize {
        self.lines.partition_point(|&at| at <= byte)
    }

    pub fn line_text(&self, line: usize) -> &str {
        let Some(&start) = line.checked_sub(1).and_then(|i| self.lines.get(i)) else {
            return "";
        };
        let end = self.lines.get(line).copied().unwrap_or(self.text.len());
        self.text[start..end].trim_end_matches(['\r', '\n'])
    }
}

pub struct Corpus {
    pub root: PathBuf,
    pub files: Vec<SourceFile>,
}

const RUST: u8 = 1;
const SDK_RUST: u8 = 2;
const MODULE_C: u8 = 4;
const CDECL: u8 = 8;

#[derive(Clone, Copy)]
pub struct SourceNeeds {
    pub workspace_rust: bool,
    pub module_c: bool,
    pub sdk_rust: bool,
    pub cdecl_sources: bool,
}
impl Default for SourceNeeds {
    fn default() -> Self {
        Self {
            workspace_rust: true,
            module_c: false,
            sdk_rust: false,
            cdecl_sources: false,
        }
    }
}

impl Corpus {
    pub fn load(root: &Path) -> Result<Self, String> {
        Self::load_with_c(root, false)
    }
    pub fn load_with_c(root: &Path, include_c: bool) -> Result<Self, String> {
        Self::load_paths(root, &[], include_c)
    }
    pub fn load_paths(root: &Path, paths: &[PathBuf], include_c: bool) -> Result<Self, String> {
        Self::load_scopes(
            root,
            paths,
            SourceNeeds {
                module_c: include_c,
                ..SourceNeeds::default()
            },
        )
    }
    pub fn load_scopes(
        root: &Path,
        inputs: &[PathBuf],
        needs: SourceNeeds,
    ) -> Result<Self, String> {
        let root = root
            .canonicalize()
            .map_err(|e| format!("{}: {e}", root.display()))?;
        let mut paths = BTreeMap::<PathBuf, u8>::new();
        if inputs.is_empty() {
            collect_scopes(&root, &root, needs, false, &mut paths)
                .map_err(|e| format!("source traversal: {e}"))?;
            // Missing the default Rust corpus is an incomplete check, even if
            // another scope happens to exist and supplies files.
            if needs.workspace_rust {
                fs::read_dir(root.join("sdk/lib/rust"))
                    .map_err(|e| format!("Rust source root: {e}"))?;
            }
            if needs.sdk_rust {
                fs::read_dir(root.join("sdk")).map_err(|e| format!("SDK source root: {e}"))?;
            }
        } else {
            for input in inputs {
                let input = if input.is_absolute() {
                    input.clone()
                } else {
                    root.join(input)
                };
                let metadata =
                    fs::metadata(&input).map_err(|e| format!("{}: {e}", input.display()))?;
                if metadata.is_dir() {
                    collect_scopes(&input, &input, needs, true, &mut paths)
                        .map_err(|e| format!("source traversal: {e}"))?;
                } else {
                    paths.insert(input, RUST | SDK_RUST | CDECL);
                }
            }
            if needs.module_c {
                // Fixture paths narrow Rust inputs, but the HRESULT oracle still
                // comes from this repository just as in the original scanner.
                let c_only = SourceNeeds {
                    module_c: true,
                    workspace_rust: false,
                    ..SourceNeeds::default()
                };
                let mut extra = BTreeMap::new();
                collect_scopes(&root, &root, c_only, false, &mut extra)
                    .map_err(|e| format!("C source traversal: {e}"))?;
                for (path, scope) in extra {
                    if scope & MODULE_C != 0 {
                        *paths.entry(path).or_default() |= MODULE_C;
                    }
                }
            }
        }
        if needs.module_c {
            *paths
                .entry(root.join("sdk/include/psdk/winerror.h"))
                .or_default() |= MODULE_C;
        }
        let entries: Vec<_> = paths.into_iter().collect();
        let files: Result<Vec<_>, String> = entries
            .par_iter()
            .map(|(path, scopes)| {
                let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
                // Path.read_text() uses universal newlines in the Python oracles.
                let text = String::from_utf8_lossy(&bytes);
                let text = if text.contains('\r') {
                    text.replace("\r\n", "\n").replace('\r', "\n")
                } else {
                    text.into_owned()
                };
                let mut file =
                    SourceFile::new(path.strip_prefix(&root).unwrap_or(path).to_owned(), text);
                let resolved = path
                    .canonicalize()
                    .map_err(|e| format!("{}: {e}", path.display()))?;
                file.resolved = Some(resolved.strip_prefix(&root).unwrap_or(path).to_owned());
                file.scopes = *scopes;
                Ok(file)
            })
            .collect();
        Ok(Self {
            root,
            files: files?,
        })
    }
    pub fn rust_files(&self) -> impl Iterator<Item = &SourceFile> {
        self.files
            .iter()
            .filter(|f| f.scopes & RUST != 0 && f.path.extension().is_some_and(|ext| ext == "rs"))
    }
    pub fn sdk_rust_files(&self) -> impl Iterator<Item = &SourceFile> {
        self.files.iter().filter(|f| {
            f.scopes & SDK_RUST != 0 && f.path.extension().is_some_and(|ext| ext == "rs")
        })
    }
    pub fn cdecl_files(&self) -> impl Iterator<Item = &SourceFile> {
        self.files.iter().filter(|f| f.scopes & CDECL != 0)
    }
}

const CDECL_ROOTS: &[&str] = &[
    "boot",
    "base",
    "dll",
    "drivers",
    "hal",
    "modules",
    "ntoskrnl",
    "sdk",
    "subsystems",
    "win32ss",
];
const CDECL_EXTS: &[&str] = &["c", "h", "s", "S", "asm", "inc", "rs", "cpp"];
fn directory_scopes(relative: &Path, needs: SourceNeeds, explicit: bool) -> u8 {
    if relative.components().any(|p| p.as_os_str() == "target") {
        return 0;
    }
    let rust = Path::new("sdk/lib/rust");
    let mut scopes = 0;
    if needs.workspace_rust
        && (explicit || relative.starts_with(rust) || rust.starts_with(relative))
    {
        scopes |= RUST;
    }
    if needs.sdk_rust
        && (explicit || relative.starts_with("sdk") || Path::new("sdk").starts_with(relative))
    {
        scopes |= SDK_RUST;
    }
    let rel = relative.to_string_lossy();
    if needs.module_c
        && !["build-", "modules/rostests", "sdk/lib/rust", ".git"]
            .iter()
            .any(|p| rel.starts_with(p))
    {
        scopes |= MODULE_C;
    }
    let cdecl_root = relative
        .components()
        .next()
        .map(|p| p.as_os_str().to_string_lossy());
    let cdecl_skipped = relative.components().any(|p| {
        let name = p.as_os_str().to_string_lossy();
        name.starts_with('.') || name.starts_with("build-") || name == "node_modules"
    });
    if needs.cdecl_sources
        && !cdecl_skipped
        && (explicit || cdecl_root.is_none_or(|p| CDECL_ROOTS.contains(&p.as_ref())))
    {
        scopes |= CDECL;
    }
    scopes
}
fn collect_scopes(
    base: &Path,
    dir: &Path,
    needs: SourceNeeds,
    explicit: bool,
    paths: &mut BTreeMap<PathBuf, u8>,
) -> std::io::Result<()> {
    if base.components().any(|p| p.as_os_str() == "target") {
        return Ok(());
    }
    let relative = dir.strip_prefix(base).unwrap();
    let scopes = directory_scopes(relative, needs, explicit);
    if scopes == 0 {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let kind = entry.file_type()?;
        if kind.is_dir() {
            collect_scopes(base, &path, needs, explicit, paths)?;
            continue;
        }
        let ext = path.extension().unwrap_or_default().to_string_lossy();
        let file_relative = path.strip_prefix(base).unwrap();
        let mut selected = 0;
        if ext == "rs" {
            if scopes & RUST != 0 && (explicit || file_relative.starts_with("sdk/lib/rust")) {
                selected |= RUST;
            }
            if scopes & SDK_RUST != 0 && (explicit || file_relative.starts_with("sdk")) {
                selected |= SDK_RUST;
            }
        }
        if ext == "c" && scopes & MODULE_C != 0 {
            selected |= MODULE_C;
        }
        if scopes & CDECL != 0
            && CDECL_EXTS.contains(&ext.as_ref())
            && (explicit
                || file_relative.components().next().is_some_and(|p| {
                    CDECL_ROOTS.contains(&p.as_os_str().to_string_lossy().as_ref())
                }))
        {
            selected |= CDECL;
        }
        if selected != 0 {
            if kind.is_symlink() && fs::metadata(&path)?.is_dir() {
                continue;
            }
            *paths.entry(path).or_default() |= selected;
        }
    }
    Ok(())
}

pub fn execute(
    corpus: &Corpus,
    selected: &[&dyn Rule],
) -> Result<Vec<(&'static str, Vec<String>)>, String> {
    let results: Vec<_> = selected
        .par_iter()
        .map(|rule| {
            let started = std::time::Instant::now();
            let mut lines = rule
                .run(corpus)
                .map_err(|e| format!("{}: {e}", rule.name()))?;
            if std::env::var_os("VIBEOS_LINT_TIMINGS").is_some() {
                eprintln!(
                    "TIMING\t{}\t{:.3}",
                    rule.name(),
                    started.elapsed().as_secs_f64()
                );
            }
            if lines
                .iter()
                .any(|line| line.contains(['\n', '\r']) || line.is_empty())
            {
                return Err(format!("{}: invalid finding in tagged output", rule.name()));
            }
            lines.sort();
            lines.dedup();
            Ok((rule.name(), lines))
        })
        .collect();
    results.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_offsets_and_unicode() {
        let file = SourceFile::new("x.rs".into(), "é\r\nnext\n".into());
        assert_eq!(file.line(4), 2);
        assert_eq!(file.line_text(1), "é");
        assert_eq!(file.line_text(2), "next");
        assert_eq!(file.line_text(0), "");
    }

    #[test]
    fn lexical_views_share_work_without_changing_legacy_offsets() {
        let file = SourceFile::new("x.rs".into(), "/* é */\nfn f() {} MARKER ignored".into());
        let regex = Regex::new(r"(?s)/\*.*?\*/").unwrap();
        let bytes = file.masked(&regex, Some("MARKER"), false);
        assert_eq!(bytes.as_ref(), "        \nfn f() {} ");
        assert!(Arc::ptr_eq(
            &bytes,
            &file.masked(&regex, Some("MARKER"), false)
        ));
        assert_eq!(
            file.masked(&regex, Some("MARKER"), true).as_ref(),
            "       \nfn f() {} "
        );
    }

    #[test]
    fn source_walk_prunes_targets_and_does_not_follow_directory_links() {
        let temp = std::env::temp_dir().join(format!(
            "vibeos-lint-corpus-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(temp.clone());
        let rust = temp.join("sdk/lib/rust");
        fs::create_dir_all(rust.join("crate/src")).unwrap();
        fs::create_dir_all(rust.join("target")).unwrap();
        fs::write(rust.join("crate/src/lib.rs"), "fn f() {}\n").unwrap();
        fs::write(rust.join("target/ignored.rs"), "ignored").unwrap();
        fs::write(rust.join("other.txt"), "ignored").unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&rust, rust.join("cycle")).unwrap();
            std::os::unix::fs::symlink(rust.join("crate/src/lib.rs"), rust.join("alias.rs"))
                .unwrap();
        }
        let corpus = Corpus::load(&temp).unwrap();
        assert!(corpus
            .files
            .iter()
            .all(|f| !f.path.components().any(|p| p.as_os_str() == "target")));
        assert!(corpus
            .files
            .iter()
            .any(|f| f.path == Path::new("sdk/lib/rust/crate/src/lib.rs")));
        #[cfg(unix)]
        assert_eq!(corpus.files.len(), 2);
        #[cfg(unix)]
        assert_eq!(
            corpus
                .files
                .iter()
                .find(|f| f.path.ends_with("alias.rs"))
                .unwrap()
                .resolved_path(),
            Path::new("sdk/lib/rust/crate/src/lib.rs")
        );
        assert!(Corpus::load_paths(&temp, &[temp.join("absent")], false).is_err());
        assert!(Corpus::load_paths(&temp, &[rust.join("target")], false)
            .unwrap()
            .files
            .is_empty());
    }

    #[test]
    fn union_scopes_share_files_without_widening_rule_inputs() {
        let temp = std::env::temp_dir().join(format!("vibeos-lint-scopes-{}", std::process::id()));
        fs::create_dir_all(&temp).unwrap();
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(temp.clone());
        for name in [
            "sdk/lib/rust/a/src/lib.rs",
            "sdk/extra.rs",
            "dll/alias.c",
            "dll/target/ignored.rs",
            "dll/.hidden/ignored.asm",
            "dll/build-test/ignored.c",
            "tools/ignored.c",
            "root.asm",
        ] {
            let path = temp.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "source").unwrap();
        }
        let corpus = Corpus::load_scopes(
            &temp,
            &[],
            SourceNeeds {
                sdk_rust: true,
                cdecl_sources: true,
                ..SourceNeeds::default()
            },
        )
        .unwrap();
        let names = |files: Vec<&SourceFile>| {
            files
                .into_iter()
                .map(|f| f.path.to_string_lossy().into_owned())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            names(corpus.rust_files().collect()),
            ["sdk/lib/rust/a/src/lib.rs"]
        );
        assert_eq!(
            names(corpus.sdk_rust_files().collect()),
            ["sdk/extra.rs", "sdk/lib/rust/a/src/lib.rs"]
        );
        assert_eq!(
            names(corpus.cdecl_files().collect()),
            ["dll/alias.c", "sdk/extra.rs", "sdk/lib/rust/a/src/lib.rs"]
        );
        assert_eq!(corpus.files.len(), 3);
        let explicit = Corpus::load_scopes(
            &temp,
            &[PathBuf::from("root.asm")],
            SourceNeeds {
                workspace_rust: false,
                cdecl_sources: true,
                ..SourceNeeds::default()
            },
        )
        .unwrap();
        assert_eq!(names(explicit.cdecl_files().collect()), ["root.asm"]);
        fs::write(temp.join("root.asm"), "first\rsecond\r\nthird\n").unwrap();
        let newlines = Corpus::load_scopes(
            &temp,
            &[PathBuf::from("root.asm")],
            SourceNeeds {
                workspace_rust: false,
                cdecl_sources: true,
                ..SourceNeeds::default()
            },
        )
        .unwrap();
        assert_eq!(newlines.files[0].text, "first\nsecond\nthird\n");
    }
}
