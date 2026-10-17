//! Git facts captured centrally before rule execution.
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

const CACHE_SCHEMA: u64 = 1;

#[derive(Default)]
pub struct VersionHistory {
    pub ever_versioned_cmake: BTreeSet<PathBuf>,
    pub last_parent_text: BTreeMap<PathBuf, String>,
    pub last_parent_revision: BTreeMap<PathBuf, String>,
}
fn git(root: &Path, args: &[&str]) -> Result<Output, String> {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|e| format!("version history: git: {e}"))
}
impl VersionHistory {
    pub(crate) fn capture(root: &Path) -> Result<Self, String> {
        let key = CacheKey::discover(root)?;
        let enabled = cache_enabled();
        let cache_path = cache_path(root, &key);
        if enabled {
            match fs::read(&cache_path) {
                Ok(bytes) => {
                    if let Some(snapshot) = decode_cache(&bytes, &key) {
                        return Ok(snapshot);
                    }
                    // A truncated/old/mismatched cache is never treated as an empty
                    // history. Recompute from Git, then replace it atomically.
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => {
                    return Err(format!(
                        "version history: read cache {}: {e}",
                        cache_path.display()
                    ))
                }
            }
        }
        let snapshot = Self::capture_git(root)?;
        let after = current_head(root)?;
        if after != key.head {
            return Err(format!(
                "version history: HEAD changed during capture ({} -> {}); retry the lint",
                key.head, after
            ));
        }
        if enabled {
            write_cache(&cache_path, &key, &snapshot)?;
        }
        Ok(snapshot)
    }

    fn capture_git(root: &Path) -> Result<Self, String> {
        let paths = git(
            root,
            &[
                "log",
                "--format=",
                "--name-only",
                "-S_WIN32_WINNT",
                "--",
                "*CMakeLists.txt",
            ],
        )?;
        if !paths.status.success() {
            return Err(format!(
                "version history: {}",
                String::from_utf8_lossy(&paths.stderr)
            ));
        }
        let mut out = Self::default();
        out.ever_versioned_cmake = String::from_utf8_lossy(&paths.stdout)
            .split_whitespace()
            .map(PathBuf::from)
            .collect();
        // Each path's Git queries are independent. Bound concurrency rather than
        // spawning one process per path or serializing every cold cache miss.
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(8)
            .build()
            .map_err(|e| format!("version history: worker pool: {e}"))?;
        let rows: Result<Vec<_>, String> = pool.install(|| {
            out.ever_versioned_cmake
                .par_iter()
                .map(|path| {
                    let p = path
                        .to_str()
                        .ok_or("version history: non-UTF-8 CMake path")?;
                    let last = git(
                        root,
                        &["log", "-1", "--format=%h", "-S_WIN32_WINNT", "--", p],
                    )?;
                    if !last.status.success() {
                        return Err(format!(
                            "version history: {}",
                            String::from_utf8_lossy(&last.stderr)
                        ));
                    }
                    let commit = String::from_utf8_lossy(&last.stdout).trim().to_owned();
                    if commit.is_empty() {
                        return Ok(None);
                    }
                    let revision = format!("{commit}^");
                    let old = git(root, &["show", &format!("{revision}:{p}")])?;
                    // A newly created file has no copy in the parent. The old scanner
                    // treats this as no historical override, not an empty source file.
                    if old.status.success() {
                        return Ok(Some((
                            path.clone(),
                            String::from_utf8_lossy(&old.stdout).into_owned(),
                            revision,
                        )));
                    }
                    Ok(None)
                })
                .collect()
        });
        for (path, text, revision) in rows?.into_iter().flatten() {
            out.last_parent_text.insert(path.clone(), text);
            out.last_parent_revision.insert(path, revision);
        }
        Ok(out)
    }
}

#[derive(Debug)]
struct CacheKey {
    repository: String,
    head: String,
}
impl CacheKey {
    fn discover(root: &Path) -> Result<Self, String> {
        let top = git(root, &["rev-parse", "--show-toplevel"])?;
        if !top.status.success() {
            return Err(format!(
                "version history: {}",
                String::from_utf8_lossy(&top.stderr)
            ));
        }
        let top = PathBuf::from(String::from_utf8_lossy(&top.stdout).trim());
        let top = top
            .canonicalize()
            .map_err(|e| format!("version history: canonicalize repository root: {e}"))?;
        let common = git(root, &["rev-parse", "--git-common-dir"])?;
        if !common.status.success() {
            return Err(format!(
                "version history: {}",
                String::from_utf8_lossy(&common.stderr)
            ));
        }
        let common = PathBuf::from(String::from_utf8_lossy(&common.stdout).trim());
        let common = if common.is_absolute() {
            common
        } else {
            top.join(common)
        };
        let common = common
            .canonicalize()
            .map_err(|e| format!("version history: canonicalize Git directory: {e}"))?;
        let head = current_head(root)?;
        Ok(Self {
            repository: format!("{}|{}", top.display(), common.display()),
            head,
        })
    }
}
fn current_head(root: &Path) -> Result<String, String> {
    let out = git(root, &["rev-parse", "--verify", "HEAD"])?;
    if !out.status.success() {
        return Err(format!(
            "version history: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let head = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    if head.is_empty() {
        return Err("version history: Git returned an empty HEAD".into());
    }
    Ok(head)
}
fn cache_enabled() -> bool {
    std::env::var("VIBEOS_LINT_CACHE").map_or(true, |v| v != "0")
}
fn cache_path(root: &Path, key: &CacheKey) -> PathBuf {
    root.join(".cache")
        .join("vibeos-lint")
        .join(format!("history-{}-v{}.json", key.head, CACHE_SCHEMA))
}
fn decode_cache(bytes: &[u8], key: &CacheKey) -> Option<VersionHistory> {
    let value: serde_json::Value = serde_json::from_slice(bytes).ok()?;
    if value.get("schema")?.as_u64()? != CACHE_SCHEMA
        || value.get("head")?.as_str()? != key.head
        || value.get("repository")?.as_str()? != key.repository
    {
        return None;
    }
    let mut out = VersionHistory::default();
    for value in value.get("ever_versioned_cmake")?.as_array()? {
        let path = Path::new(value.as_str()?);
        if path.is_absolute() {
            return None;
        }
        out.ever_versioned_cmake.insert(path.to_owned());
    }
    for entry in value.get("last_parent")?.as_array()? {
        let path = Path::new(entry.get("path")?.as_str()?);
        let text = entry.get("text")?.as_str()?;
        let revision = entry.get("revision")?.as_str()?;
        if path.is_absolute() || revision.is_empty() {
            return None;
        }
        out.last_parent_text
            .insert(path.to_owned(), text.to_owned());
        out.last_parent_revision
            .insert(path.to_owned(), revision.to_owned());
    }
    Some(out)
}
fn write_cache(path: &Path, key: &CacheKey, history: &VersionHistory) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("version history: cache has no parent: {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|e| {
        format!(
            "version history: create cache directory {}: {e}",
            parent.display()
        )
    })?;
    let mut parent_rows = Vec::with_capacity(history.last_parent_text.len());
    for (path, text) in &history.last_parent_text {
        let path = path
            .to_str()
            .ok_or("version history: non-UTF-8 path while writing cache")?;
        let revision = history
            .last_parent_revision
            .get(Path::new(path))
            .ok_or_else(|| format!("version history: missing revision for {path}"))?;
        parent_rows.push(serde_json::json!({"path": path, "text": text, "revision": revision}));
    }
    let ever: Vec<_> = history
        .ever_versioned_cmake
        .iter()
        .map(|p| {
            p.to_str()
                .ok_or("version history: non-UTF-8 path while writing cache")
        })
        .collect::<Result<_, _>>()?;
    let bytes = serde_json::to_vec(&serde_json::json!({
        "schema": CACHE_SCHEMA,
        "repository": key.repository,
        "head": key.head,
        "ever_versioned_cmake": ever,
        "last_parent": parent_rows,
    }))
    .map_err(|e| format!("version history: encode cache: {e}"))?;
    let mut temp = None;
    for attempt in 0..100u32 {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let candidate = parent.join(format!(
            ".history-{}-{}-{nonce}-{attempt}.tmp",
            std::process::id(),
            key.head
        ));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(mut file) => {
                file.write_all(&bytes)
                    .and_then(|_| file.sync_all())
                    .map_err(|e| {
                        let _ = fs::remove_file(&candidate);
                        format!("version history: write cache {}: {e}", candidate.display())
                    })?;
                temp = Some(candidate);
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("version history: create cache temp: {e}")),
        }
    }
    let temp = temp.ok_or("version history: could not allocate unique cache temp file")?;
    if let Err(e) = fs::rename(&temp, path) {
        let _ = fs::remove_file(&temp);
        return Err(format!(
            "version history: publish cache {}: {e}",
            path.display()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn historical_override_uses_the_last_change_parent_and_handles_new_files() {
        let root = std::env::temp_dir().join(format!("vibeos-lint-history-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(root.clone());
        let run = |args: &[&str]| {
            let o = git(&root, args).unwrap();
            assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
            o
        };
        run(&["init", "-q", "-b", "history-fixture"]);
        run(&["config", "user.name", "lint fixture"]);
        run(&["config", "user.email", "lint@example.invalid"]);
        std::fs::write(
            root.join("CMakeLists.txt"),
            "add_definitions(-D_WIN32_WINNT=0x502)\n",
        )
        .unwrap();
        std::fs::create_dir_all(root.join("new")).unwrap();
        std::fs::write(
            root.join("new/CMakeLists.txt"),
            "add_definitions(-D_WIN32_WINNT=0x600)\n",
        )
        .unwrap();
        run(&["add", "CMakeLists.txt", "new/CMakeLists.txt"]);
        run(&["commit", "-qm", "initial version profiles"]);
        std::fs::write(
            root.join("CMakeLists.txt"),
            "add_cargo_module(example example-cargo example.dll)\n",
        )
        .unwrap();
        run(&["add", "CMakeLists.txt"]);
        run(&["commit", "-qm", "replace C link"]);
        let snapshot = VersionHistory::capture(&root).unwrap();
        assert!(snapshot
            .ever_versioned_cmake
            .contains(Path::new("CMakeLists.txt")));
        assert!(snapshot
            .ever_versioned_cmake
            .contains(Path::new("new/CMakeLists.txt")));
        assert_eq!(
            snapshot.last_parent_text[Path::new("CMakeLists.txt")],
            "add_definitions(-D_WIN32_WINNT=0x502)\n"
        );
        assert!(snapshot.last_parent_revision[Path::new("CMakeLists.txt")].ends_with('^'));
        assert!(!snapshot
            .last_parent_text
            .contains_key(Path::new("new/CMakeLists.txt")));
        let key = CacheKey::discover(&root).unwrap();
        let cache = cache_path(&root, &key);
        assert!(cache.is_file(), "capture should publish a disk cache");
        std::fs::write(&cache, b"{ truncated").unwrap();
        let refreshed = VersionHistory::capture(&root).unwrap();
        assert_eq!(
            refreshed.ever_versioned_cmake,
            snapshot.ever_versioned_cmake
        );
        assert_eq!(refreshed.last_parent_text, snapshot.last_parent_text);
        assert_eq!(
            refreshed.last_parent_revision,
            snapshot.last_parent_revision
        );
        let bytes = std::fs::read(cache).unwrap();
        assert!(
            decode_cache(&bytes, &key).is_some(),
            "bad cache must be recomputed and replaced"
        );
    }
}
