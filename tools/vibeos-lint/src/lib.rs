//! Statically linked lint plugins over an immutable, shared source snapshot.
pub mod rules;

use rayon::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub trait Rule: Sync {
    fn name(&self) -> &'static str;
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String>;
    fn needs_c_sources(&self) -> bool {
        false
    }
}

pub struct SourceFile {
    pub path: PathBuf,
    pub text: String,
    resolved: Option<PathBuf>,
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

impl Corpus {
    /// Match source_files(root): prune target before entering it, never follow
    /// directory symlinks, include symlinks to files, propagate read failures.
    pub fn load(root: &Path) -> Result<Self, String> {
        Self::load_with_c(root, false)
    }

    pub fn load_with_c(root: &Path, include_c: bool) -> Result<Self, String> {
        Self::load_paths(root, &[], include_c)
    }

    pub fn load_paths(root: &Path, inputs: &[PathBuf], include_c: bool) -> Result<Self, String> {
        let root = root
            .canonicalize()
            .map_err(|e| format!("{}: {e}", root.display()))?;
        let mut paths = Vec::new();
        let defaults = [root.join("sdk/lib/rust")];
        for input in if inputs.is_empty() {
            &defaults[..]
        } else {
            inputs
        } {
            let input = if input.is_absolute() {
                input.clone()
            } else {
                root.join(input)
            };
            let metadata = fs::metadata(&input).map_err(|e| format!("{}: {e}", input.display()))?;
            if metadata.is_dir() {
                collect(&input, &mut paths).map_err(|e| format!("source traversal: {e}"))?;
            } else {
                paths.push(input);
            }
        }
        if include_c {
            collect_c(&root, &root, &mut paths).map_err(|e| format!("C source traversal: {e}"))?;
            paths.push(root.join("sdk/include/psdk/winerror.h"));
        }
        paths.sort();
        paths.dedup();
        let files: Result<Vec<_>, String> = paths
            .par_iter()
            .map(|path| {
                let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
                let mut file = SourceFile::new(
                    path.strip_prefix(&root).unwrap_or(path).to_owned(),
                    String::from_utf8_lossy(&bytes).into_owned(),
                );
                let resolved = path
                    .canonicalize()
                    .map_err(|e| format!("{}: {e}", path.display()))?;
                file.resolved = Some(resolved.strip_prefix(&root).unwrap_or(path).to_owned());
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
            .filter(|f| f.path.extension().is_some_and(|ext| ext == "rs"))
    }
}

fn collect_c(root: &Path, dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(root).unwrap().to_string_lossy();
        let kind = entry.file_type()?;
        if kind.is_dir() {
            if entry.file_name() != "target"
                && !["build-", "modules/rostests", "sdk/lib/rust", ".git"]
                    .iter()
                    .any(|prefix| relative.starts_with(prefix))
            {
                collect_c(root, &path, files)?;
            }
        } else if path.extension().is_some_and(|ext| ext == "c") {
            if kind.is_symlink() && fs::metadata(&path)?.is_dir() {
                continue;
            }
            files.push(path);
        }
    }
    Ok(())
}

fn collect(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if dir.components().any(|part| part.as_os_str() == "target") {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let kind = entry.file_type()?;
        if kind.is_dir() {
            if entry.file_name() != "target" {
                collect(&path, files)?;
            }
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            // os.walk lists symlinked directories in dirs, not files.
            if kind.is_symlink() && fs::metadata(&path)?.is_dir() {
                continue;
            }
            files.push(path);
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
    }
}
