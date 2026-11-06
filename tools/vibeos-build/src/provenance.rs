//! Source/content and media freshness authority shared by build and QEMU clients.
//! Architecture DTOs cross the adapter explicitly; generated inputs stay lane-local.
use crate::{Error, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

thread_local! { static REQUEST_ENV: std::cell::RefCell<Option<BTreeMap<String,String>>> = const { std::cell::RefCell::new(None) }; }
struct EnvGuard(Option<BTreeMap<String, String>>);
impl Drop for EnvGuard {
    fn drop(&mut self) {
        REQUEST_ENV.with(|v| *v.borrow_mut() = self.0.take());
    }
}
fn request_env() -> BTreeMap<String, String> {
    REQUEST_ENV
        .with(|v| v.borrow().clone())
        .unwrap_or_else(|| std::env::vars().collect())
}
pub const STAMP_NAME: &str = ".vibeos-build-stamp.json";
pub const NON_BUILD_PATHSPEC: &[&str] = &[
    ":(exclude)docs",
    ":(exclude)*.md",
    ":(exclude).github",
    ":(exclude).claude",
    ":(exclude)tools/lint-baseline",
    ":(exclude)tools/cohab-census.txt",
    ":(exclude)tools/pi4",
    ":(exclude)tools/pi5",
    ":(exclude,glob)tests/**/test_*.py",
    ":(exclude,glob)tools/test_*.py",
    ":(exclude)tests/qemu/vm.py",
];
const PE_SUFFIXES: &[&str] = &[
    "exe", "dll", "sys", "drv", "cpl", "ocx", "efi", "acm", "ax", "tlb", "mui", "scr",
];
fn error(message: impl Into<String>) -> Error {
    Error::classified("ProvenanceError", message)
}
fn text(path: &Path) -> Result<String> {
    fs::read(path)
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .map_err(|e| error(format!("{}: {e}", path.display())))
}
fn name(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}
fn pathstr(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
fn resolve(path: &Path) -> Result<PathBuf> {
    crate::artifact_fs::resolve(path)
}
fn mt(path: &Path) -> Result<f64> {
    let m = fs::metadata(path)?;
    Ok(m.mtime() as f64 + m.mtime_nsec() as f64 / 1e9)
}
fn mt_ns(path: &Path) -> Result<i128> {
    let m = fs::metadata(path)?;
    Ok(m.mtime() as i128 * 1_000_000_000 + m.mtime_nsec() as i128)
}
fn base_arch(s: &str) -> String {
    s.replace("-net", "")
        .replace("-uefi", "")
        .replace("-bios", "")
        .replace("-test", "")
}
fn stem(arch: &str) -> Option<&'static str> {
    match arch {
        "i386" => Some("i686-kernel"),
        "amd64" => Some("x86_64-pc-windows-gnu"),
        "arm64" => Some("aarch64-pc-windows-gnullvm"),
        _ => None,
    }
}
fn build_dir(repo: &Path, arch: &str) -> PathBuf {
    repo.join(format!("build-{}", base_arch(arch)))
}
fn pe(path: &Path) -> bool {
    path.extension()
        .is_some_and(|s| PE_SUFFIXES.contains(&s.to_string_lossy().to_lowercase().as_str()))
}
fn unique<T: Ord>(xs: impl IntoIterator<Item = T>) -> Vec<T> {
    xs.into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Bounded capture retains Git's raw output bytes, including non-UTF8 diffs.
pub fn run_bytes(
    cmd: &[String],
    cwd: &Path,
    ok: &[i32],
    env: Option<&BTreeMap<String, String>>,
) -> Result<Vec<u8>> {
    let (program, args) = cmd
        .split_first()
        .ok_or_else(|| error("empty provenance command"))?;
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let effective_env = env
        .cloned()
        .or_else(|| REQUEST_ENV.with(|v| v.borrow().clone()));
    if let Some(env) = effective_env {
        command.env_clear().envs(env);
    }
    let mut child = command
        .spawn()
        .map_err(|e| error(format!("{} (cwd={}): {e}", cmd.join(" "), cwd.display())))?;
    let mut stdout = child.stdout.take().ok_or("missing command stdout")?;
    let mut stderr = child.stderr.take().ok_or("missing command stderr")?;
    let out = std::thread::spawn(move || {
        let mut v = Vec::new();
        stdout.read_to_end(&mut v).map(|_| v)
    });
    let err = std::thread::spawn(move || {
        let mut v = Vec::new();
        stderr.read_to_end(&mut v).map(|_| v)
    });
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if start.elapsed() > Duration::from_secs(120) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error(format!(
                "{} (cwd={}): timed out after 120s",
                cmd.join(" "),
                cwd.display()
            )));
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let stdout = out.join().map_err(|_| error("stdout reader failed"))??;
    let stderr = err.join().map_err(|_| error("stderr reader failed"))??;
    let code = status.code().unwrap_or(-1);
    if !ok.contains(&code) {
        let stderr = String::from_utf8_lossy(&stderr);
        let line = stderr.trim().lines().next().unwrap_or("<no stderr>");
        return Err(error(format!(
            "{} (cwd={}) exited {code}: {}",
            cmd.join(" "),
            cwd.display(),
            line.chars().take(200).collect::<String>()
        )));
    }
    Ok(stdout)
}
fn command(repo: &Path, args: &[&str], ok: &[i32]) -> Result<Vec<u8>> {
    run_bytes(
        &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        repo,
        ok,
        None,
    )
}
fn git_text(repo: &Path, args: &[&str], ok: &[i32]) -> Result<String> {
    Ok(String::from_utf8_lossy(&command(repo, args, ok)?).into_owned())
}
pub fn git_head(repo: &Path) -> Result<String> {
    Ok(git_text(repo, &["git", "rev-parse", "HEAD"], &[0])?
        .trim()
        .to_owned())
}
pub fn git_fingerprint(repo: &Path, scope: &[String]) -> Result<String> {
    let committed = if scope.is_empty() {
        git_head(repo)?.into_bytes()
    } else {
        let mut args = vec!["git".into(), "ls-files".into(), "-s".into(), "--".into()];
        args.extend_from_slice(scope);
        run_bytes(&args, repo, &[0], None)?
    };
    let mut diff = vec!["git".into(), "diff".into(), "HEAD".into(), "--".into()];
    diff.extend_from_slice(scope);
    let mut status = vec![
        "git".into(),
        "status".into(),
        "--porcelain".into(),
        "--".into(),
    ];
    status.extend_from_slice(scope);
    let mut h = Sha256::new();
    h.update(committed);
    h.update([0]);
    h.update(run_bytes(&diff, repo, &[0], None)?);
    h.update([0]);
    h.update(run_bytes(&status, repo, &[0], None)?);
    Ok(format!("{:x}", h.finalize()))
}
pub fn tree_content_hash(repo: &Path, scope: &[String]) -> Result<String> {
    let index =
        PathBuf::from(git_text(repo, &["git", "rev-parse", "--git-path", "index"], &[0])?.trim());
    let index = if index.is_absolute() {
        index
    } else {
        repo.join(index)
    };
    let tmp = tempfile::Builder::new().prefix("vibeos-prov-").tempdir()?;
    let scratch = tmp.path().join("index");
    if index.is_file() {
        crate::copy_git_index(&index, &scratch)?;
    }
    let mut env: BTreeMap<String, String> = request_env();
    env.insert("GIT_INDEX_FILE".into(), pathstr(&scratch));
    run_bytes(
        &["git".into(), "add".into(), "-A".into()],
        repo,
        &[0],
        Some(&env),
    )?;
    let mut args = vec!["git".into(), "ls-files".into(), "-s".into(), "--".into()];
    args.extend_from_slice(scope);
    Ok(format!(
        "{:x}",
        Sha256::digest(run_bytes(&args, repo, &[0], Some(&env))?)
    ))
}
fn sha_file(path: &Path) -> String {
    let Ok(mut file) = fs::File::open(path) else {
        return String::new();
    };
    let mut h = Sha256::new();
    let mut buf = vec![0; 1 << 20];
    loop {
        match file.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => h.update(&buf[..n]),
            Err(_) => return String::new(),
        }
    }
    format!("{:x}", h.finalize())
}
#[derive(Clone)]
enum GlobToken {
    Literal(char),
    Any,
    Star,
    Class(bool, Vec<(char, char)>),
}
fn glob_tokens(pattern: &str) -> Vec<GlobToken> {
    let chars: Vec<char> = pattern.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '*' => {
                out.push(GlobToken::Star);
                i += 1;
            }
            '?' => {
                out.push(GlobToken::Any);
                i += 1;
            }
            '[' => {
                let mut j = i + 1;
                let negated = chars.get(j) == Some(&'!');
                if negated {
                    j += 1;
                }
                let start = j;
                if chars.get(j) == Some(&']') {
                    j += 1;
                }
                while j < chars.len() && chars[j] != ']' {
                    j += 1;
                }
                if j == chars.len() {
                    out.push(GlobToken::Literal('['));
                    i += 1;
                    continue;
                }
                let mut ranges = Vec::new();
                let mut k = start;
                while k < j {
                    if k + 2 < j && chars[k + 1] == '-' {
                        if chars[k] <= chars[k + 2] {
                            ranges.push((chars[k], chars[k + 2]));
                        }
                        k += 3;
                    } else {
                        ranges.push((chars[k], chars[k]));
                        k += 1;
                    }
                }
                out.push(GlobToken::Class(negated, ranges));
                i = j + 1;
            }
            c => {
                out.push(GlobToken::Literal(c));
                i += 1;
            }
        }
    }
    out
}
fn match_component(tokens: &[GlobToken], value: &str) -> bool {
    fn run(
        p: &[GlobToken],
        v: &[char],
        i: usize,
        j: usize,
        memo: &mut BTreeMap<(usize, usize), bool>,
    ) -> bool {
        if let Some(found) = memo.get(&(i, j)) {
            return *found;
        }
        let found = if i == p.len() {
            j == v.len()
        } else {
            match &p[i] {
                GlobToken::Star => {
                    run(p, v, i + 1, j, memo) || (j < v.len() && run(p, v, i, j + 1, memo))
                }
                GlobToken::Any => j < v.len() && run(p, v, i + 1, j + 1, memo),
                GlobToken::Literal(c) => j < v.len() && v[j] == *c && run(p, v, i + 1, j + 1, memo),
                GlobToken::Class(negated, ranges) => {
                    j < v.len()
                        && (ranges.iter().any(|(a, b)| *a <= v[j] && v[j] <= *b) != *negated)
                        && run(p, v, i + 1, j + 1, memo)
                }
            }
        };
        memo.insert((i, j), found);
        found
    }
    run(
        tokens,
        &value.chars().collect::<Vec<_>>(),
        0,
        0,
        &mut BTreeMap::new(),
    )
}
fn path_glob(parts: &[(bool, Vec<GlobToken>)], values: &[String], anchored: bool) -> bool {
    fn run(parts: &[(bool, Vec<GlobToken>)], v: &[String], i: usize, j: usize) -> bool {
        if i == parts.len() {
            return j == v.len();
        }
        if parts[i].0 {
            return run(parts, v, i + 1, j) || (j < v.len() && run(parts, v, i, j + 1));
        }
        j < v.len() && match_component(&parts[i].1, &v[j]) && run(parts, v, i + 1, j + 1)
    }
    if anchored {
        run(parts, values, 0, 0)
    } else {
        (0..=values.len()).any(|start| run(parts, values, 0, start))
    }
}
fn walk_once(root: &Path, pattern: &str, out: &mut Vec<PathBuf>, anchored: bool) -> Result<()> {
    fn walk(
        base: &Path,
        dir: &Path,
        parts: &[(bool, Vec<GlobToken>)],
        dir_only: bool,
        anchored: bool,
        out: &mut Vec<PathBuf>,
    ) -> Result<()> {
        match fs::metadata(dir) {
            Ok(meta) if meta.is_dir() => {}
            Ok(_) => return Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e.into()),
        }
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            let meta = fs::symlink_metadata(&path)?;
            let rel = path
                .strip_prefix(base)
                .map_err(|_| error("walk escaped its selected root"))?;
            let values: Vec<String> = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect();
            if path_glob(parts, &values, anchored) && (!dir_only || path.is_dir()) {
                out.push(path.clone());
            }
            if meta.is_dir() {
                walk(base, &path, parts, dir_only, anchored, out)?;
            }
        }
        Ok(())
    }
    if pattern.is_empty() || pattern.starts_with('/') || pattern.split('/').any(|p| p == "..") {
        return Err(error(
            "glob pattern must be nonempty and confined to its selected root",
        ));
    }
    let recursive_dirs = pattern.trim_end_matches('/').split('/').next_back() == Some("**");
    let dir_only = pattern.ends_with('/') || recursive_dirs;
    let parts: Vec<_> = pattern
        .trim_end_matches('/')
        .split('/')
        .filter(|p| !p.is_empty())
        .map(|p| (p == "**", glob_tokens(p)))
        .collect();
    if recursive_dirs && parts.iter().all(|(recursive, _)| *recursive) && root.is_dir() {
        out.push(root.to_owned());
    }
    walk(root, root, &parts, dir_only, anchored, out)
}
// Replace production rglob_stable with this helper + wrapper, then append tests.
// Fault injection is native test-only; no request operation accepts fake walk results.
fn retry_walk(mut attempt: impl FnMut(&mut Vec<PathBuf>) -> Result<()>) -> Result<Vec<PathBuf>> {
    for i in 0..5 {
        let mut out = Vec::new();
        match attempt(&mut out) {
            Ok(()) => return Ok(out),
            Err(_) if i < 4 => {}
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
pub fn rglob_stable(root: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    retry_walk(|out| walk_once(root, pattern, out, false))
}

/// Anchored Path.glob semantics for declarative host application staging.
pub fn glob_stable(root: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    retry_walk(|out| walk_once(root, pattern, out, true))
}

pub fn discover_rust_targets(build: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for pattern in ["rust-*-target", "rust-target"] {
        out.extend(
            rglob_stable(build, pattern)?
                .into_iter()
                .filter(|p| p.is_dir()),
        );
    }
    Ok(unique(out))
}
pub fn discover_rust_archives(build: &Path) -> Result<Vec<PathBuf>> {
    let mut out = rglob_stable(build, "lib*_rust.a")?;
    out.sort();
    Ok(out)
}
pub fn workspace_cargo_target(repo: &Path, arch: &str) -> Option<PathBuf> {
    let p = repo
        .join("sdk/lib/rust/target")
        .join(stem(&base_arch(arch))?);
    p.is_dir().then_some(p)
}
pub fn workspace_cargo_targets(repo: &Path, arch: &str) -> Result<Vec<PathBuf>> {
    let base = base_arch(arch);
    let Some(stem) = stem(&base) else {
        return Ok(Vec::new());
    };
    let root = repo.join("sdk/lib/rust/target");
    if resolve(&root)? != resolve(repo)?.join("sdk/lib/rust/target") {
        return Err(Error::new(format!(
            "refusing symlinked workspace target: {}",
            root.display()
        )));
    }
    let mut parents = vec![root.clone()];
    if root.is_dir() {
        for e in fs::read_dir(&root)? {
            let p = e?.path();
            let n = name(&p);
            if n.ends_with("-rustlib")
                || n.ends_with("-cargo-link")
                || n.starts_with("cell-link-v1-")
            {
                parents.push(p);
            }
        }
    }
    parents.sort();
    parents.dedup();
    let stems = if base == "i386" {
        vec![stem, "i686-pc-windows-gnu"]
    } else {
        vec![stem]
    };
    let mut out = Vec::new();
    for p in parents {
        for s in &stems {
            let path = p.join(s);
            if p.is_symlink() || path.is_symlink() {
                return Err(Error::new(format!(
                    "refusing symlinked nested Cargo target: {}",
                    path.display()
                )));
            }
            if path.is_dir() {
                out.push(path);
            }
        }
    }
    out.sort();
    Ok(out)
}
#[derive(Clone, Debug, Deserialize)]
struct Config {
    boot_media_rel: String,
    #[serde(default)]
    staged_bins_rel: Vec<String>,
    #[serde(default)]
    needs_mkramdisk: bool,
}
fn config(v: &Value) -> Result<Config> {
    serde_json::from_value(v.clone()).map_err(Into::into)
}
fn discover_rust_module_outputs(repo: &Path, arch: &str, cfg: &Config) -> Vec<PathBuf> {
    let mut rels = cfg.staged_bins_rel.clone();
    rels.extend(
        [
            "ntoskrnl/ntoskrnl.exe",
            "ntoskrnl/ntkrnlmp.exe",
            "win32ss/win32k.sys",
            "dll/win32/kernel32/kernel32.dll",
        ]
        .map(str::to_owned),
    );
    unique(rels)
        .into_iter()
        .map(|p| build_dir(repo, arch).join(p))
        .filter(|p| p.is_file())
        .collect()
}
fn workspace_members(repo: &Path) -> Option<BTreeSet<String>> {
    let txt = fs::read_to_string(repo.join("sdk/lib/rust/Cargo.toml")).ok()?;
    let re = Regex::new(r"(?s)members\s*=\s*\[(.*?)\]").ok()?;
    let block = re.captures(&txt)?.get(1)?.as_str();
    Some(
        Regex::new(r#""([^"]+)""#)
            .ok()?
            .captures_iter(block)
            .map(|c| c[1].to_owned())
            .collect(),
    )
}
fn cmake_named_crates(repo: &Path) -> Option<BTreeSet<String>> {
    let out = command(
        repo,
        &[
            "git",
            "grep",
            "-ohE",
            r"[A-Za-z0-9_-]+-rust\b",
            "--",
            "*CMakeLists.txt",
            "*.cmake",
        ],
        &[0, 1],
    )
    .ok()?;
    let names: BTreeSet<_> = String::from_utf8_lossy(&out)
        .split_whitespace()
        .map(str::to_owned)
        .collect();
    (!names.is_empty()).then_some(names)
}
struct CrateMap {
    members: Option<BTreeSet<String>>,
    linked: Option<BTreeSet<String>>,
}
impl CrateMap {
    fn new(repo: &Path) -> Self {
        Self {
            members: workspace_members(repo),
            linked: cmake_named_crates(repo),
        }
    }
    fn stem(&self, repo: &Path, stem: &str) -> Option<PathBuf> {
        for n in [
            format!("{stem}-rust"),
            format!("{}-rust", stem.replace('_', "-")),
        ] {
            let p = repo.join("sdk/lib/rust").join(&n);
            if p.join("Cargo.toml").is_file()
                && self.members.as_ref().is_none_or(|x| x.contains(&n))
                && self.linked.as_ref().is_none_or(|x| x.contains(&n))
            {
                return Some(p);
            }
        }
        None
    }
    fn rel(&self, repo: &Path, rel: &str) -> Option<PathBuf> {
        let p = Path::new(rel);
        let stem = p.file_stem()?.to_string_lossy();
        if let Some(ext) = p.extension() {
            if let Some(found) = self.stem(
                repo,
                &format!("{}_{}", stem, ext.to_string_lossy().to_lowercase()),
            ) {
                return Some(found);
            }
        }
        self.stem(repo, &stem)
    }
}
fn crate_for_stem(repo: &Path, stem: &str) -> Option<PathBuf> {
    CrateMap::new(repo).stem(repo, stem)
}
fn crate_for_rel(repo: &Path, rel: &str) -> Option<PathBuf> {
    CrateMap::new(repo).rel(repo, rel)
}
fn newest_rs_mtime(crate_dir: &Path) -> Result<f64> {
    let root = if crate_dir.join("src").is_dir() {
        crate_dir.join("src")
    } else {
        crate_dir.to_owned()
    };
    let mut newest: f64 = 0.0;
    for p in rglob_stable(&root, "*.rs")? {
        if let Ok(t) = mt(&p) {
            newest = newest.max(t)
        }
    }
    Ok(newest)
}
fn depinfo_rs_mtime(repo: &Path, cargo: &Path, crate_dir: &Path) -> Result<Option<f64>> {
    let Ok(txt) = fs::read_to_string(cargo.with_extension("d")) else {
        return Ok(None);
    };
    let crate_dir = resolve(crate_dir)?;
    let mut newest: Option<f64> = None;
    for line in txt.lines() {
        let Some((_, deps)) = line.split_once(": ") else {
            continue;
        };
        for tok in deps.replace("\\ ", "\0").split_whitespace() {
            let tok = tok.replace('\0', " ");
            if !tok.ends_with(".rs") {
                continue;
            }
            let p = canonical_input(repo, repo, &tok);
            if resolve(&p)?.starts_with(&crate_dir) {
                if let Ok(t) = mt(&p) {
                    newest = Some(newest.map_or(t, |v| v.max(t)));
                }
            }
        }
    }
    Ok(newest)
}
fn mkramdisk_required_rels(repo: &Path) -> Vec<String> {
    let Ok(txt) = fs::read_to_string(repo.join("build-arm64/boot/livecd.Debug.lst")) else {
        return Vec::new();
    };
    let prefixes = [
        "/tmp/vibeos/build-arm64/".to_owned(),
        format!("{}/build-arm64/", repo.display()),
    ];
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for line in txt.lines() {
        let src = line.split_once('=').map(|x| x.1).unwrap_or(line);
        for prefix in &prefixes {
            if let Some(rel) = src.strip_prefix(prefix) {
                if seen.insert(rel.to_owned()) {
                    out.push(rel.to_owned());
                }
                break;
            }
        }
    }
    out
}
fn boot_set_cohab_with_map(repo: &Path, cfg: &Config, map: &CrateMap) -> Vec<String> {
    if !cfg.needs_mkramdisk {
        return Vec::new();
    }
    let mut rels = mkramdisk_required_rels(repo);
    rels.extend(cfg.staged_bins_rel.clone());
    unique(rels)
        .into_iter()
        .filter(|p| pe(Path::new(p)) && map.rel(repo, p).is_some())
        .collect()
}
fn boot_set_cohab_rels(repo: &Path, _arch: &str, cfg: &Config) -> Vec<String> {
    boot_set_cohab_with_map(repo, cfg, &CrateMap::new(repo))
}

fn store_root(v: &Value) -> PathBuf {
    v.get("store_root")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("VIBEOS_ARTIFACT_STORE_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                        .join("vibeos-artifact-store")
                })
        })
}
fn store_refs(repo: &Path, arch: &str, store: &Path) -> Result<BTreeMap<String, PathBuf>> {
    let refs = store.join("v1/refs");
    if !refs.is_dir() {
        return Ok(BTreeMap::new());
    }
    let lane = pathstr(&resolve(repo)?);
    let markers = [
        format!("-{arch}-cargo-link/"),
        format!("/cell-link-v1-{arch}-"),
    ];
    let mut newest: BTreeMap<String, (f64, PathBuf)> = BTreeMap::new();
    for entry in fs::read_dir(refs)? {
        let path = entry?.path();
        if path.extension().is_none_or(|s| s != "json") {
            continue;
        }
        let Ok(raw) = fs::read(&path) else { continue };
        let Ok(record) = serde_json::from_slice::<Value>(&raw) else {
            continue;
        };
        let consumer = record["consumer"].as_str().unwrap_or("");
        let fields: Vec<_> = consumer.split('|').collect();
        if record["lane"].as_str() == Some(lane.as_str())
            && consumer.starts_with("cargo:")
            && fields.len() > 2
            && markers.iter().any(|m| fields[1].contains(m))
        {
            let Ok(mtime) = mt(&path) else { continue };
            let body = fields[2].to_owned();
            if newest.get(&body).is_none_or(|(old, _)| mtime > *old) {
                newest.insert(body, (mtime, path));
            }
        }
    }
    Ok(newest.into_iter().map(|(k, (_, v))| (k, v)).collect())
}
#[derive(Default)]
struct StaticlibIndex {
    libs: BTreeMap<String, Vec<PathBuf>>,
    refs: BTreeMap<String, PathBuf>,
}
impl StaticlibIndex {
    fn new(repo: &Path, build: &Path, store: &Path) -> Result<Self> {
        let mut s = Self::default();
        let selected = resolve(repo)?;
        for root in [build.to_owned(), repo.join("sdk/lib/rust/target")] {
            if !resolve(&root)?.starts_with(&selected) {
                return Err(error(format!(
                    "generated archive root {} resolves into another checkout",
                    root.display()
                )));
            }
            for p in rglob_stable(&root, "lib*.a")? {
                if !resolve(&p)?.starts_with(&selected) {
                    return Err(error(format!(
                        "generated archive {} resolves into another checkout",
                        p.display()
                    )));
                }
                s.libs.entry(name(&p)).or_default().push(p);
            }
        }
        s.refs = store_refs(repo, &name(build).trim_start_matches("build-"), store)?;
        Ok(s)
    }
}
fn crate_staticlibs(
    crate_dir: &Path,
    index: &StaticlibIndex,
) -> Result<(Option<PathBuf>, Option<PathBuf>)> {
    let lib = format!("lib{}.a", name(crate_dir).replace('-', "_"));
    let candidates = index.libs.get(&lib).cloned().unwrap_or_default();
    let mut cargo: Option<(f64, PathBuf)> = None;
    let mut copied: Option<(f64, PathBuf)> = None;
    for p in candidates {
        if !p.is_file() {
            continue;
        }
        let t = mt(&p)?;
        let dst = if p.components().any(|c| {
            let x = c.as_os_str().to_string_lossy();
            x == "target" || x.ends_with("-target")
        }) {
            &mut cargo
        } else {
            &mut copied
        };
        if dst.as_ref().is_none_or(|(old, _)| t > *old) {
            *dst = Some((t, p));
        }
    }
    if let Some(p) = index.refs.get(&name(crate_dir)) {
        let t = mt(p)?;
        if cargo.as_ref().is_none_or(|(old, _)| t > *old) {
            cargo = Some((t, p.clone()));
        }
    }
    Ok((cargo.map(|(_, p)| p), copied.map(|(_, p)| p)))
}
fn belongs_to_sibling(copied: &Path, artifact: &Path) -> Result<bool> {
    if copied.parent() == artifact.parent() {
        return Ok(false);
    }
    let stem = artifact
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    let Some(parent) = copied.parent() else {
        return Ok(false);
    };
    for e in fs::read_dir(parent)? {
        let p = e?.path();
        if p.is_file()
            && p.file_stem()
                .is_some_and(|s| s.to_string_lossy().to_lowercase() == stem)
            && pe(&p)
        {
            return Ok(true);
        }
    }
    Ok(false)
}
fn lag(older: &Path, newer: &Path) -> String {
    let (Ok(a), Ok(b)) = (mt(older), mt(newer)) else {
        return String::new();
    };
    let d = b - a;
    if d < 2.0 {
        format!(" [input {:.0}ms newer — bytes changed AFTER the link; a later pass (ninja-optional:*?) re-emitted this staticlib. NOT a benign ordering race]",d*1000.0)
    } else if d < 120.0 {
        format!(" [input {d:.1}s newer]")
    } else {
        format!(
            " [input {:.0}min newer — a genuinely missed rebuild]",
            d / 60.0
        )
    }
}
fn cargo_module_rels(repo: &Path) -> Result<Vec<String>> {
    let files = git_text(
        repo,
        &[
            "git",
            "grep",
            "-lE",
            r"add_cargo_(module|kernel_dll)\(",
            "--",
            "*.txt",
            "*.cmake",
        ],
        &[0, 1],
    )?;
    let re = Regex::new(r"(?m)^\s*add_cargo_(?:module|kernel_dll)\(\s*(\S+)\s+(\S+)\s+(\S+)")?;
    let mut rels = Vec::new();
    for file in files.lines() {
        let p = repo.join(file);
        if name(&p) != "CMakeLists.txt" && p.extension().is_none_or(|x| x != "cmake") {
            continue;
        }
        let Ok(bytes) = fs::read(p) else {
            continue;
        };
        let txt = String::from_utf8_lossy(&bytes);
        for c in re.captures_iter(&txt) {
            let out = c[3].trim_end_matches(')');
            if c[1].contains('$') || out.contains('$') {
                continue;
            }
            let rel = Path::new(file).parent().unwrap_or(Path::new("")).join(out);
            rels.push(pathstr(&rel));
        }
    }
    Ok(unique(rels))
}
fn canonical_input(repo: &Path, build: &Path, word: &str) -> PathBuf {
    let word = word.replace("$$", "$");
    let path = Path::new(&word);
    if let Ok(rel) = path.strip_prefix("/tmp/vibeos") {
        repo.join(rel)
    } else if path.is_absolute() {
        path.to_owned()
    } else {
        build.join(path)
    }
}
// Python shlex.split(..., comments=False): '#' is a filename byte, never a comment.
fn dep_words(input: &str) -> Result<Vec<String>> {
    let mut out = Vec::new();
    let mut word = String::new();
    let mut active = false;
    let mut quote = None;
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        match quote {
            Some('\'') => {
                if c == '\'' {
                    quote = None;
                } else {
                    word.push(c);
                }
            }
            Some('"') => {
                if c == '"' {
                    quote = None;
                } else if c == '\\' {
                    let next = chars.next().ok_or("invalid quoted dependency rule")?;
                    if next != '"' && next != '\\' {
                        word.push('\\');
                    }
                    word.push(next);
                } else {
                    word.push(c);
                }
            }
            _ => {
                if c.is_whitespace() {
                    if active {
                        out.push(std::mem::take(&mut word));
                        active = false;
                    }
                } else if c == '\'' || c == '"' {
                    quote = Some(c);
                    active = true;
                } else if c == '\\' {
                    let next = chars.next().ok_or("invalid quoted dependency rule")?;
                    word.push(next);
                    active = true;
                } else {
                    word.push(c);
                    active = true;
                }
            }
        }
    }
    if quote.is_some() {
        return Err("invalid quoted dependency rule".into());
    }
    if active {
        out.push(word);
    }
    Ok(out)
}
fn cargo_link_inputs(repo: &Path, build: &Path, artifact: &Path) -> Result<Vec<PathBuf>> {
    let depfile = PathBuf::from(format!("{}.d", artifact.display()));
    let result = (|| -> Result<Vec<PathBuf>> {
        let txt = text(&depfile)?.replace("\\\n", " ");
        let rule = txt.lines().next().ok_or("empty dependency file")?;
        let rel = artifact
            .strip_prefix(repo)
            .map_err(|_| error("artifact outside selected checkout"))?;
        let targets = [
            artifact.to_owned(),
            Path::new("/tmp/vibeos").join(rel),
            artifact
                .strip_prefix(build)
                .map_err(|_| error("artifact outside build directory"))?
                .to_owned(),
        ];
        let mut body = None;
        for target in targets {
            let prefix = format!("{}: ", pathstr(&target).replace(' ', "\\ "));
            if let Some(rest) = rule.strip_prefix(&prefix) {
                body = Some(rest);
                break;
            }
        }
        let words = dep_words(body.ok_or("missing expected output rule")?)?;
        if words.is_empty() {
            return Err("empty input closure".into());
        }
        let mut inputs = Vec::new();
        for word in words {
            let path = canonical_input(repo, build, &word);
            let generated = path.components().any(|c| {
                matches!(
                    c.as_os_str().to_str(),
                    Some("build-i386" | "build-amd64" | "build-arm64")
                )
            }) || pathstr(&path).contains("/sdk/lib/rust/target/");
            if generated && !resolve(&path)?.starts_with(&resolve(repo)?) {
                return Err(error(format!(
                    "generated dependency {} resolves into another checkout",
                    path.display()
                )));
            }
            if !path.exists() {
                return Err(format!("missing dependency {}", path.display()).into());
            }
            inputs.push(path.clone());
            if path.is_dir() {
                inputs.extend(rglob_stable(&path, "*")?);
            }
        }
        Ok(inputs)
    })();
    result.map_err(|e| {
        error(format!(
            "{}: cannot verify cargo link inputs: {e}; run `build`",
            depfile.display()
        ))
    })
}
fn media_manifest_rels(repo: &Path, arch: &str, cfg: &Config) -> Option<BTreeSet<String>> {
    let stem = Path::new(&cfg.boot_media_rel)
        .file_stem()?
        .to_string_lossy();
    let txt = fs::read_to_string(
        build_dir(repo, arch)
            .join("boot")
            .join(format!("{stem}.cmake.lst")),
    )
    .ok()?;
    let prefix = format!("/tmp/vibeos/build-{}/", base_arch(arch));
    Some(
        txt.lines()
            .filter_map(|line| line.split_once('=').map(|(_, s)| s.trim()))
            .filter_map(|s| s.strip_prefix(&prefix).map(str::to_owned))
            .collect(),
    )
}
fn cargo_link_staleness_reasons(repo: &Path, arch: &str, cfg: &Config) -> Result<Vec<String>> {
    let build = build_dir(repo, arch);
    let scope = if cfg.needs_mkramdisk {
        let mut rels = mkramdisk_required_rels(repo);
        rels.extend(cfg.staged_bins_rel.clone());
        rels.into_iter().collect()
    } else {
        let Some(scope) = media_manifest_rels(repo, arch, cfg) else {
            return Ok(Vec::new());
        };
        scope
    };
    crate::depfile::check_compile_snapshot(&build)
        .map_err(|e| error(format!("cannot verify compile commands: {e}; run `build`")))?;
    let mut reasons = Vec::new();
    for rel in cargo_module_rels(repo)? {
        if !scope.contains(&rel) {
            continue;
        }
        let artifact = build.join(&rel);
        if !artifact.is_file() {
            continue;
        }
        let inputs = cargo_link_inputs(repo, &build, &artifact)?;
        let mut newest: Option<(i128, PathBuf)> = None;
        for p in inputs {
            let n = mt_ns(&p)?;
            if newest.as_ref().is_none_or(|(old, _)| n > *old) {
                newest = Some((n, p));
            }
        }
        let (_, p) = newest.ok_or("empty cargo input closure")?;
        let newest = mt(&p)?;
        let artifact_mt = mt(&artifact)?;
        if artifact_mt < newest {
            let d = newest - artifact_mt;
            let note = if d < 2.0 {
                " [<2s — possibly same-run ordering; confirm the binary is newer than the SOURCE it compiled, not merely close]".into()
            } else if d >= 120.0 {
                format!(" [{:.0}min newer — a genuinely missed rebuild]", d / 60.0)
            } else {
                format!(" [{d:.1}s newer]")
            };
            let p = pathstr(p.strip_prefix(repo).unwrap_or(&p));
            reasons.push(format!("{rel}: stale — {p} newer than the cargo-linked binary (cargo link not re-run). ninja's graph can MISS this edge, in which case `build` reports the same failure forever and `ninja <target>` says 'no work to do'. If a rebuild does not clear it: `rm {rel}` and build again. That works PRECISELY BECAUSE the edge is missing — deleting the output is the only way to make ninja rebuild something it has no reason to rebuild — so if you need it, the cargo module is probably missing a BODY_CRATES declaration. Once that is declared, the rm is a ONE-TIME cleanup for an already-stale tree, not a standing remedy.{note}"));
        }
    }
    Ok(reasons)
}
fn cohab_staleness_reasons(
    repo: &Path,
    arch: &str,
    cfg: &Config,
    store: &Path,
) -> Result<Vec<String>> {
    let build = build_dir(repo, arch);
    let mut reasons = Vec::new();
    let map = CrateMap::new(repo);
    let rels = boot_set_cohab_with_map(repo, cfg, &map);
    if rels.is_empty() {
        return Ok(reasons);
    }
    let index = StaticlibIndex::new(repo, &build, store)?;
    let cargo_rels: BTreeSet<_> = cargo_module_rels(repo)?.into_iter().collect();
    for rel in rels {
        let crate_dir = map
            .rel(repo, &rel)
            .ok_or("mapped cohab crate disappeared")?;
        let crate_name = name(&crate_dir);
        let artifact = build.join(&rel);
        if !artifact.is_file() {
            reasons.push(format!(
                "{rel}: {crate_name} cohab crate but build artifact absent — never built"
            ));
            continue;
        }
        let newest_rs = newest_rs_mtime(&crate_dir)?;
        let (cargo, copied) = crate_staticlibs(&crate_dir, &index)?;
        if (cargo.is_none() && copied.is_none()) || cargo_rels.contains(&rel) {
            if newest_rs > 0.0 && mt(&artifact)? < newest_rs {
                reasons.push(format!("{rel}: stale — {crate_name} has .rs newer than the built binary (cohab flip not rebuilt; `build` force-rebuilds it, or add to ninja_targets)"));
            }
            continue;
        }
        if newest_rs > 0.0 && cargo.is_none() {
            reasons.push(format!("{rel}: stale — {crate_name} has a staged staticlib but cargo's own output is missing for this arch (cohab crate never compiled here)"));
            continue;
        }
        if newest_rs > 0.0 {
            if let Some(cargo) = &cargo {
                let effective = depinfo_rs_mtime(repo, cargo, &crate_dir)?.unwrap_or(newest_rs);
                if mt(cargo)? < effective {
                    reasons.push(format!("{rel}: stale — {crate_name} .rs is newer than cargo's own output ({}); the crate was not rebuilt (add to ninja_targets?)",name(cargo)));
                    continue;
                }
            }
        }
        if let Some(copied) = copied {
            if belongs_to_sibling(&copied, &artifact)? {
                continue;
            }
            if mt(&artifact)? < mt(&copied)? {
                reasons.push(format!("{rel}: stale — {} is newer than the built binary; the cohab staticlib changed but {} was never relinked (not in ninja_targets?){}",name(&copied),name(Path::new(&rel)),lag(&artifact,&copied)));
            }
        }
    }
    Ok(reasons)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stamp {
    pub arch: String,
    pub ts: i64,
    pub git_fingerprint: String,
    pub rust_src_hash: String,
    pub boot_media: String,
    pub boot_media_sha: String,
    pub cold: bool,
    #[serde(default)]
    pub contributing: BTreeMap<String, String>,
    #[serde(default)]
    pub rust_targets: Vec<String>,
    #[serde(default)]
    pub build_src_hash: String,
    #[serde(default)]
    pub head_sha: String,
    #[serde(default)]
    pub rust_tree_hash: String,
    #[serde(default)]
    pub build_tree_hash: String,
}
impl Stamp {
    pub fn read(build: &Path) -> Option<Self> {
        serde_json::from_slice(&fs::read(build.join(STAMP_NAME)).ok()?).ok()
    }
    pub fn write(&self, build: &Path) -> Result<PathBuf> {
        let path = build.join(STAMP_NAME);
        let tmp = build.join(format!("{STAMP_NAME}.tmp"));
        fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
        fs::rename(tmp, &path)?;
        Ok(path)
    }
}
fn build_scope() -> Vec<String> {
    let mut paths = vec![".".to_owned()];
    paths.extend(NON_BUILD_PATHSPEC.iter().map(|s| s.to_string()));
    paths
}
fn rust_scope() -> Vec<String> {
    vec!["sdk/lib/rust".into()]
}
fn contributing_hashes(repo: &Path, arch: &str, cfg: &Config) -> BTreeMap<String, String> {
    let build = build_dir(repo, arch);
    let mut rels = cfg.staged_bins_rel.clone();
    if matches!(base_arch(arch).as_str(), "i386" | "amd64") {
        rels.extend(
            [
                "ntoskrnl/ntoskrnl.exe",
                "hal/halx86/hal.dll",
                "boot/freeldr/freeldr/freeldr.sys",
            ]
            .map(str::to_owned),
        );
    }
    rels.into_iter()
        .filter_map(|rel| {
            let p = build.join(&rel);
            p.is_file().then(|| (rel, sha_file(&p)))
        })
        .collect()
}
fn make_stamp(repo: &Path, arch: &str, cold: bool, ts: i64, cfg: &Config) -> Result<Stamp> {
    let build = build_dir(repo, arch);
    let targets = discover_rust_targets(&build)?
        .iter()
        .map(|p| pathstr(p.strip_prefix(repo).unwrap_or(p)))
        .collect();
    Ok(Stamp {
        arch: base_arch(arch),
        ts,
        git_fingerprint: git_fingerprint(repo, &[])?,
        rust_src_hash: git_fingerprint(repo, &rust_scope())?,
        boot_media: cfg.boot_media_rel.clone(),
        boot_media_sha: sha_file(&repo.join(&cfg.boot_media_rel)),
        cold,
        contributing: contributing_hashes(repo, arch, cfg),
        rust_targets: targets,
        build_src_hash: git_fingerprint(repo, &build_scope())?,
        head_sha: git_head(repo)?,
        rust_tree_hash: tree_content_hash(repo, &rust_scope())?,
        build_tree_hash: tree_content_hash(repo, &build_scope())?,
    })
}
fn src_delta(repo: &Path, stamp: &Stamp) -> String {
    match src_delta_inner(repo, stamp) {
        Ok(s) => s,
        Err(e) => format!(" — (could not explain: {e})"),
    }
}
fn src_delta_inner(repo: &Path, stamp: &Stamp) -> Result<String> {
    let now = git_head(repo)?;
    let moved = !stamp.head_sha.is_empty() && now != stamp.head_sha;
    let base = if moved {
        stamp.head_sha.as_str()
    } else {
        "HEAD"
    };
    let mut args = vec![
        "git".into(),
        "diff".into(),
        "--name-only".into(),
        base.into(),
        "--".into(),
    ];
    args.extend(build_scope());
    let names = run_bytes(&args, repo, &[0], None)?;
    let names = String::from_utf8_lossy(&names);
    let mut args = vec![
        "git".into(),
        "status".into(),
        "--porcelain".into(),
        "--".into(),
    ];
    args.extend(build_scope());
    let dirty = run_bytes(&args, repo, &[0], None)?;
    let dirty = String::from_utf8_lossy(&dirty);
    let mut changed: BTreeSet<String> = names.split_whitespace().map(str::to_owned).collect();
    changed.extend(dirty.lines().map(|l| l.get(3..).unwrap_or("").to_owned()));
    let head = if moved {
        format!(
            " (HEAD also moved: {} -> {})",
            stamp.head_sha.chars().take(12).collect::<String>(),
            now.chars().take(12).collect::<String>()
        )
    } else {
        String::new()
    };
    if changed.is_empty() {
        return Ok(if moved {
            format!(" — but NO build-relevant path differs from the build's commit{head}. Nothing in scope changed, so this is a STALE STAMP, not stale media: rebuild once to re-stamp.")
        } else {
            " — but NOTHING is currently modified vs HEAD, and HEAD did not move. The fingerprint moved without a source change: suspect the oracle, not the tree (see ProvenanceError / _run)".into()
        });
    }
    let count = changed.len();
    Ok(format!(
        " — {count} path(s) vs the build's commit{head}: {}{}",
        changed.into_iter().take(6).collect::<Vec<_>>().join(", "),
        if count > 6 { " …" } else { "" }
    ))
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Staleness {
    pub fresh: bool,
    pub reasons: Vec<String>,
    pub warm: bool,
    pub media_missing: bool,
}
impl Staleness {
    fn stale(reasons: Vec<String>, warm: bool, missing: bool) -> Self {
        Self {
            fresh: false,
            reasons,
            warm,
            media_missing: missing,
        }
    }
}
fn check_staleness(repo: &Path, arch: &str, cfg: &Config, store: &Path) -> Staleness {
    match check_staleness_inner(repo, arch, cfg, store) {
        Ok(s) => s,
        Err(e) => Staleness::stale(
            vec![format!(
                "provenance oracle FAILED (verdict unknown, not stale): {e}"
            )],
            false,
            false,
        ),
    }
}
fn check_staleness_inner(repo: &Path, arch: &str, cfg: &Config, store: &Path) -> Result<Staleness> {
    let build = build_dir(repo, arch);
    let media = repo.join(&cfg.boot_media_rel);
    if !media.is_file() {
        return Ok(Staleness::stale(
            vec!["boot media absent — run `build`".into()],
            false,
            true,
        ));
    }
    let Some(stamp) = Stamp::read(&build) else {
        return Ok(Staleness::stale(
            vec!["no build stamp — never built via vibeos-cli".into()],
            false,
            false,
        ));
    };
    let warm = !stamp.cold;
    let mut reasons = cohab_staleness_reasons(repo, arch, cfg, store)?;
    reasons.extend(cargo_link_staleness_reasons(repo, arch, cfg)?);
    if git_fingerprint(repo, &[])? == stamp.git_fingerprint {
        return Ok(Staleness {
            fresh: reasons.is_empty(),
            reasons,
            warm,
            media_missing: false,
        });
    }
    let (rust_moved, build_moved) =
        if !stamp.rust_tree_hash.is_empty() && !stamp.build_tree_hash.is_empty() {
            (
                tree_content_hash(repo, &rust_scope())? != stamp.rust_tree_hash,
                tree_content_hash(repo, &build_scope())? != stamp.build_tree_hash,
            )
        } else {
            (
                git_fingerprint(repo, &rust_scope())? != stamp.rust_src_hash,
                !stamp.build_src_hash.is_empty()
                    && git_fingerprint(repo, &build_scope())? != stamp.build_src_hash,
            )
        };
    if rust_moved {
        reasons.push(format!(
            "rust source changed since build{}",
            if warm {
                " (last build WARM — may mask)"
            } else {
                ""
            }
        ));
    }
    if stamp.build_src_hash.is_empty() {
        reasons.push("stamp predates build-source fingerprinting — cannot verify non-Rust sources; run `build` to re-stamp".into());
    } else if build_moved {
        reasons.push(format!(
            "build source changed since build (C/asm/cmake/spec/rc){}",
            src_delta(repo, &stamp)
        ));
    }
    if cfg.needs_mkramdisk {
        let media_mt = mt(&media)?;
        for rel in &cfg.staged_bins_rel {
            let p = build.join(rel);
            if p.is_file() && mt(&p)? > media_mt {
                reasons.push(format!("{rel} newer than boot.img — ramdisk not re-staged"));
                break;
            }
        }
    }
    if sha_file(&media) != stamp.boot_media_sha {
        reasons.push("boot media changed since stamp (rebuilt out of band?)".into());
    }
    if reasons.is_empty() {
        Ok(Staleness {
            fresh: true,
            reasons: vec!["clean-diff (non-build change)".into()],
            warm,
            media_missing: false,
        })
    } else {
        Ok(Staleness::stale(reasons, warm, false))
    }
}
fn external_media_problems(repo: &Path, arch: &str, image: &Path) -> Result<Vec<String>> {
    let image_stem = image.file_stem().unwrap_or_default().to_string_lossy();
    let stamp = image.with_file_name(format!("{image_stem}.provenance.json"));
    if !image.is_file() {
        return Ok(vec![format!("{}: no such image", image.display())]);
    }
    if !stamp.is_file() {
        return Ok(vec![format!(
            "{} missing: master it with tools/media_manifest.py --assemble",
            stamp.display()
        )]);
    }
    let st: Value = serde_json::from_slice(&fs::read(stamp)?)?;
    let mut problems = Vec::new();
    if st["arch"].as_str() != Some(arch) {
        problems.push(format!(
            "{} is {} media, not {arch}",
            image.display(),
            st["arch"].as_str().unwrap_or("None")
        ));
    }
    if st["cd"].as_str() != Some("livecd") {
        problems.push(format!(
            "{} is the {} image: smoke boots the livecd",
            image.display(),
            st["cd"].as_str().unwrap_or("None")
        ));
    }
    if st["sha256"].as_str() != Some(&sha_file(image)) {
        problems.push(format!(
            "{} changed since it was mastered (sha256 differs from its stamp)",
            image.display()
        ));
    }
    if st["source"].as_str() != Some(&tree_content_hash(repo, &build_scope())?) {
        problems.push(format!(
            "{} was mastered from another source tree: master it again",
            image.display()
        ));
    }
    Ok(problems)
}
fn media_paths(repo: &Path, arch: &str, cfg: &Config) -> Vec<PathBuf> {
    let mut paths = vec![repo.join(&cfg.boot_media_rel)];
    if base_arch(arch) == "arm64" {
        paths.push(repo.join("qemu/ramdisk.img"));
    }
    paths
}
fn count_in_file(path: &Path, needles: &[Vec<u8>]) -> Vec<u64> {
    let mut counts = vec![0; needles.len()];
    let overlap = needles
        .iter()
        .map(Vec::len)
        .max()
        .unwrap_or(1)
        .saturating_sub(1);
    let Ok(mut file) = fs::File::open(path) else {
        return counts;
    };
    let mut tail = Vec::new();
    let mut chunk = vec![0; 8 << 20];
    loop {
        let n = match file.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => return counts,
        };
        let mut buf = tail;
        buf.extend_from_slice(&chunk[..n]);
        for (needle, count) in needles.iter().zip(&mut counts) {
            if needle.is_empty() {
                *count += buf.len() as u64 + 1;
                continue;
            }
            let mut i = 0;
            while i + needle.len() <= buf.len() {
                if &buf[i..i + needle.len()] == needle.as_slice() {
                    *count += 1;
                    i += needle.len();
                } else {
                    i += 1;
                }
            }
        }
        let begin = buf.len().saturating_sub(overlap);
        tail = buf[begin..].to_vec();
    }
    counts
}
fn mtime_str(path: &Path) -> String {
    let Ok(m) = fs::metadata(path) else {
        return "?".into();
    };
    let t = m.mtime() as libc::time_t;
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    if unsafe { libc::localtime_r(&t, &mut tm) }.is_null() {
        return "?".into();
    };
    let fmt = b"%Y-%m-%d %H:%M:%S\0";
    let mut out = [0u8; 64];
    let n = unsafe { libc::strftime(out.as_mut_ptr().cast(), out.len(), fmt.as_ptr().cast(), &tm) };
    String::from_utf8_lossy(&out[..n]).into_owned()
}
fn py_repr(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::from(quote);
    for c in s.chars() {
        match c {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\\' => out.push_str("\\\\"),
            c if c == quote => {
                out.push('\\');
                out.push(c)
            }
            c if c.is_control() => {
                let n = c as u32;
                if n <= 0xff {
                    out.push_str(&format!("\\x{n:02x}"));
                } else {
                    out.push_str(&format!("\\u{n:04x}"));
                }
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}
fn probes_missing_from_media(
    repo: &Path,
    arch: &str,
    literals: &[String],
    cfg: &Config,
) -> Vec<String> {
    let needles: Vec<Vec<u8>> = literals
        .iter()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.as_bytes().to_vec())
        .collect();
    if needles.is_empty() {
        return Vec::new();
    }
    let mut reasons = Vec::new();
    for media in media_paths(repo, arch, cfg) {
        if !media.is_file() {
            reasons.push(format!(
                "{} does not exist — build {arch} before requiring a probe",
                media.display()
            ));
            continue;
        }
        let counts = count_in_file(&media, &needles);
        for (n, count) in needles.iter().zip(counts) {
            if count == 0 {
                reasons.push(format!("{} appears 0 times in {} (built {}) — the probe is NOT in the media this boot would open, so its silence would prove nothing",py_repr(&String::from_utf8_lossy(n)),name(&media),mtime_str(&media)));
            }
        }
    }
    reasons
}
fn req<'a>(v: &'a Value, name: &str) -> Result<&'a Value> {
    v.get(name)
        .ok_or_else(|| error(format!("missing provenance argument {name}")))
}
fn string<'a>(v: &'a Value, name: &str) -> Result<&'a str> {
    req(v, name)?
        .as_str()
        .ok_or_else(|| error(format!("{name} must be a string")))
}
fn paths(v: Vec<PathBuf>) -> Value {
    json!(v)
}
fn as_path(root: &Path, v: &Value, name: &str) -> Result<PathBuf> {
    let p = PathBuf::from(string(v, name)?);
    Ok(if p.is_absolute() { p } else { root.join(p) })
}
fn strings(v: &Value, name: &str) -> Result<Vec<String>> {
    serde_json::from_value(req(v, name)?.clone()).map_err(Into::into)
}

/// `kwargs` contain explicit filesystem paths/configuration from the CLI adapter.
pub fn dispatch(root: &Path, request: &Value) -> Result<Value> {
    let op = request["op"]
        .as_str()
        .ok_or("missing provenance operation")?;
    let a = request.get("kwargs").unwrap_or(request);
    let env: Option<BTreeMap<String, String>> = a
        .get("env")
        .filter(|v| !v.is_null())
        .map(|v| serde_json::from_value(v.clone()))
        .transpose()?;
    let previous = REQUEST_ENV.with(|v| v.replace(env));
    let _env = EnvGuard(previous);
    let repo = if a.get("repo").is_some() {
        as_path(root, a, "repo")?
    } else {
        root.to_owned()
    };
    let cfg = || config(req(a, "cfg")?);
    let arch = || string(a, "arch");
    let result = match op {
        "provenance_non_build_pathspec" => json!(NON_BUILD_PATHSPEC),
        "provenance_run_bytes" => {
            let cmd = strings(a, "cmd")?;
            let ok: Vec<i32> = a
                .get("ok_codes")
                .map(|v| serde_json::from_value(v.clone()))
                .transpose()?
                .unwrap_or_else(|| vec![0]);
            let env: Option<BTreeMap<String, String>> = a
                .get("env")
                .filter(|v| !v.is_null())
                .map(|v| serde_json::from_value(v.clone()))
                .transpose()?;
            json!(run_bytes(
                &cmd,
                &as_path(root, a, "cwd")?,
                &ok,
                env.as_ref()
            )?)
        }
        "provenance_git_head" => json!(git_head(&repo)?),
        "provenance_git_fingerprint" => {
            let scope: Vec<String> = a
                .get("paths")
                .filter(|v| !v.is_null())
                .map(|v| serde_json::from_value(v.clone()))
                .transpose()?
                .unwrap_or_default();
            json!(git_fingerprint(&repo, &scope)?)
        }
        "provenance_tree_content_hash" => json!(tree_content_hash(&repo, &strings(a, "paths")?)?),
        "provenance_sha256_file" => json!(sha_file(&as_path(root, a, "path")?)),
        "provenance_rglob_stable" => paths(rglob_stable(
            &as_path(root, a, "path")?,
            string(a, "pattern")?,
        )?),
        "provenance_discover_rust_targets" => {
            paths(discover_rust_targets(&as_path(root, a, "build_dir")?)?)
        }
        "provenance_discover_rust_archives" => {
            paths(discover_rust_archives(&as_path(root, a, "build_dir")?)?)
        }
        "provenance_workspace_cargo_target" => json!(workspace_cargo_target(&repo, arch()?)),
        "provenance_workspace_cargo_targets" => paths(workspace_cargo_targets(&repo, arch()?)?),
        "provenance_discover_rust_module_outputs" => {
            paths(discover_rust_module_outputs(&repo, arch()?, &cfg()?))
        }
        "provenance_workspace_members" => json!(workspace_members(&repo)),
        "provenance_cmake_named_crates" => json!(cmake_named_crates(&repo)),
        "provenance_crate_for_stem" => json!(crate_for_stem(&repo, string(a, "stem")?)),
        "provenance_crate_for_rel" => json!(crate_for_rel(&repo, string(a, "rel")?)),
        "provenance_newest_rs_mtime" => json!(newest_rs_mtime(&as_path(root, a, "crate")?)?),
        "provenance_depinfo_rs_mtime" => json!(depinfo_rs_mtime(
            &repo,
            &as_path(root, a, "cargo_out")?,
            &as_path(root, a, "crate")?
        )?),
        "provenance_mkramdisk_required_rels" => json!(mkramdisk_required_rels(&repo)),
        "provenance_boot_set_cohab_rels" => json!(boot_set_cohab_rels(&repo, arch()?, &cfg()?)),
        "provenance_cohab_staleness_reasons" => json!(cohab_staleness_reasons(
            &repo,
            arch()?,
            &cfg()?,
            &store_root(a)
        )?),
        "provenance_belongs_to_sibling" => json!(belongs_to_sibling(
            &as_path(root, a, "copied")?,
            &as_path(root, a, "artifact")?
        )?),
        "provenance_lag" => json!(lag(
            &as_path(root, a, "older")?,
            &as_path(root, a, "newer")?
        )),
        "provenance_staticlib_index" => {
            let build = as_path(root, a, "build_dir")?;
            let index = StaticlibIndex::new(&repo, &build, &store_root(a))?;
            json!({"libs":index.libs,"refs":index.refs})
        }
        "provenance_crate_staticlibs" => {
            let build = as_path(root, a, "build_dir")?;
            let index = if let Some(index) = a.get("index").filter(|v| !v.is_null()) {
                StaticlibIndex {
                    libs: serde_json::from_value(index["libs"].clone())?,
                    refs: serde_json::from_value(index["refs"].clone())?,
                }
            } else {
                StaticlibIndex::new(&repo, &build, &store_root(a))?
            };
            json!(crate_staticlibs(&as_path(root, a, "crate")?, &index)?)
        }
        "provenance_store_refs" => json!(store_refs(&repo, arch()?, &store_root(a))?),
        "provenance_store_ref" => {
            json!(store_refs(&repo, arch()?, &store_root(a))?.get(string(a, "body")?))
        }
        "provenance_cargo_module_rels" => json!(cargo_module_rels(&repo)?),
        "provenance_cargo_link_inputs" => paths(cargo_link_inputs(
            &repo,
            &as_path(root, a, "build_dir")?,
            &as_path(root, a, "artifact")?,
        )?),
        "provenance_media_manifest_rels" => json!(media_manifest_rels(&repo, arch()?, &cfg()?)),
        "provenance_cargo_link_staleness_reasons" => {
            json!(cargo_link_staleness_reasons(&repo, arch()?, &cfg()?)?)
        }
        "provenance_stamp_write" => {
            let stamp: Stamp = serde_json::from_value(req(a, "stamp")?.clone())?;
            json!(stamp.write(&as_path(root, a, "build_dir")?)?)
        }
        "provenance_stamp_read" => json!(Stamp::read(&as_path(root, a, "build_dir")?)),
        "provenance_rust_src_hash" => json!(git_fingerprint(&repo, &rust_scope())?),
        "provenance_build_src_hash" => json!(git_fingerprint(&repo, &build_scope())?),
        "provenance_contributing_hashes" => json!(contributing_hashes(&repo, arch()?, &cfg()?)),
        "provenance_make_stamp" => json!(make_stamp(
            &repo,
            arch()?,
            a["cold"].as_bool().ok_or("missing cold flag")?,
            a["ts"].as_i64().ok_or("missing stamp timestamp")?,
            &cfg()?
        )?),
        "provenance_src_delta" => json!(src_delta(
            &repo,
            &serde_json::from_value(req(a, "stamp")?.clone())?
        )),
        "provenance_external_media_problems" => json!(external_media_problems(
            &repo,
            arch()?,
            &as_path(root, a, "image")?
        )?),
        "provenance_check_staleness" => {
            json!(check_staleness(&repo, arch()?, &cfg()?, &store_root(a)))
        }
        "provenance_check_staleness_inner" => json!(check_staleness_inner(
            &repo,
            arch()?,
            &cfg()?,
            &store_root(a)
        )?),
        "provenance_media_paths" => paths(media_paths(&repo, arch()?, &cfg()?)),
        "provenance_count_in_file" => {
            let needles: Vec<Vec<u8>> = serde_json::from_value(req(a, "needles")?.clone())?;
            json!(count_in_file(&as_path(root, a, "path")?, &needles))
        }
        "provenance_probes_missing_from_media" => json!(probes_missing_from_media(
            &repo,
            arch()?,
            &strings(a, "literals")?,
            &cfg()?
        )),
        "provenance_mtime_str" => json!(mtime_str(&as_path(root, a, "path")?)),
        _ => return Err(error(format!("unknown provenance operation {op}"))),
    };
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn glob_matching_preserves_unicode_classes_and_relative_components() {
        assert!(match_component(&glob_tokens("x[0-9]?.rs"), "x3é.rs"));
        assert!(!match_component(&glob_tokens("x[!0-9]*.rs"), "x3.rs"));
        assert!(match_component(&glob_tokens("x[!0-9]*.rs"), "xé.rs"));
        let parts = vec![(false, glob_tokens("nested")), (false, glob_tokens("*.rs"))];
        assert!(path_glob(
            &parts,
            &["prefix".into(), "nested".into(), "item.rs".into()],
            false
        ));
    }
    #[test]
    fn scoped_git_hashes_are_commit_invariant_and_preserve_raw_diff_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();
        let git = |args: &[&str]| command(repo, args, &[0]).unwrap();
        git(&["git", "init", "-q"]);
        git(&["git", "config", "user.name", "test"]);
        git(&["git", "config", "user.email", "test@example.invalid"]);
        fs::create_dir(repo.join("src")).unwrap();
        fs::write(repo.join("src/input.c"), b"initial\n").unwrap();
        git(&["git", "add", "-A"]);
        git(&["git", "commit", "-qm", "initial"]);
        let scope = vec!["src".to_owned()];
        let before = tree_content_hash(repo, &scope).unwrap();
        fs::write(repo.join("src/input.c"), b"\xff\n").unwrap();
        let dirty = tree_content_hash(repo, &scope).unwrap();
        assert_ne!(before, dirty);
        git(&["git", "add", "-A"]);
        git(&["git", "commit", "-qm", "bytes"]);
        assert_eq!(dirty, tree_content_hash(repo, &scope).unwrap());
        let fp = git_fingerprint(repo, &scope).unwrap();
        fs::write(repo.join("notes.md"), "outside").unwrap();
        git(&["git", "add", "-A"]);
        git(&["git", "commit", "-qm", "docs"]);
        assert_eq!(fp, git_fingerprint(repo, &scope).unwrap());
    }
    #[test]
    fn copied_pe_depfile_requires_current_complete_local_evidence() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();
        let build = repo.join("build-amd64");
        fs::create_dir(&build).unwrap();
        let artifact = build.join("fixture.exe");
        fs::write(&artifact, "PE").unwrap();
        let input = repo.join("space name.bin");
        fs::write(&input, "data").unwrap();
        let dep = PathBuf::from(format!("{}.d", artifact.display()));
        fs::write(
            &dep,
            "/tmp/vibeos/build-amd64/fixture.exe: /tmp/vibeos/space\\ name.bin\n",
        )
        .unwrap();
        assert_eq!(
            cargo_link_inputs(repo, &build, &artifact).unwrap(),
            vec![input.clone()]
        );
        fs::remove_file(&input).unwrap();
        assert!(cargo_link_inputs(repo, &build, &artifact)
            .unwrap_err()
            .message
            .contains("missing dependency"));
        fs::write(&dep, "wrong: source.rs\n").unwrap();
        assert!(cargo_link_inputs(repo, &build, &artifact).is_err());
    }
    #[test]
    fn recursive_discovery_does_not_follow_sibling_symlinks() {
        use std::os::unix::fs::symlink;
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        let sibling = dir.path().join("sibling");
        fs::create_dir_all(repo.join("build")).unwrap();
        fs::create_dir(&sibling).unwrap();
        fs::write(sibling.join("libwrong_rust.a"), "bad").unwrap();
        symlink(&sibling, repo.join("build/link")).unwrap();
        assert!(discover_rust_archives(&repo.join("build"))
            .unwrap()
            .is_empty());
        fs::create_dir_all(repo.join("sdk/lib/rust")).unwrap();
        symlink(&sibling, repo.join("sdk/lib/rust/target")).unwrap();
        assert!(workspace_cargo_targets(&repo, "amd64").is_err());
    }
    #[test]
    fn byte_stream_probe_handles_boundaries_and_reports_all_images() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("media");
        let mut bytes = vec![b'x'; (8 << 20) - 3];
        bytes.extend_from_slice(b"[MARKER]");
        fs::write(&path, bytes).unwrap();
        assert_eq!(count_in_file(&path, &[b"[MARKER]".to_vec()]), vec![1]);
    }
}
#[cfg(test)]
mod walk_retry_controls {
    use super::*;
    #[test]
    fn transient_missing_directory_retries_without_retaining_partial_evidence() {
        let mut attempts = 0;
        let out = retry_walk(|out| {
            attempts += 1;
            if attempts == 1 {
                out.push(PathBuf::from("vanished/libfoo_rust.a"));
                return Err(std::io::Error::from(std::io::ErrorKind::NotFound).into());
            }
            out.push(PathBuf::from("stable/libfoo_rust.a"));
            Ok(())
        })
        .unwrap();
        assert_eq!(attempts, 2);
        assert_eq!(out, vec![PathBuf::from("stable/libfoo_rust.a")]);
    }
    #[test]
    fn permanently_unreadable_directory_exhausts_retries_and_fails() {
        let mut attempts = 0;
        let err = retry_walk(|out| {
            attempts += 1;
            out.push(PathBuf::from("partial/libfoo_rust.a"));
            Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied).into())
        })
        .unwrap_err();
        assert_eq!(attempts, 5);
        assert!(err.to_string().to_lowercase().contains("permission"));
    }
}
