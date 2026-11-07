//! Content-based tooling receipts. The session owns its lock until publication or drop.
use crate::{Error, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::{fd::AsRawFd, unix::fs::MetadataExt},
    path::{Path, PathBuf},
    process::Command,
    sync::{LazyLock, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
static POLICY: LazyLock<Value> =
    LazyLock::new(|| serde_json::from_str(include_str!("tooling_policy.json")).unwrap());
pub const GENERATED_INPUT_CHECKS: &[&str] = &[
    "pytest:tools/rustconv/tests",
    "pytest:tools/test_compiler_codegen.py",
];
fn listed(key: &str, label: &str) -> bool {
    POLICY[key].as_array().unwrap().iter().any(|x| x == label)
}
pub fn eligibility(label: &str) -> bool {
    !listed("_LIVE", label)
        && (label.starts_with("ruff:")
            || label.starts_with("fmt:")
            || label.starts_with("pyright:")
            || label == "generated:ddk_enum_constants"
            || listed("_AUDITED_PYTEST", label))
}
fn now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}
fn digest(v: &Value) -> String {
    format!("{:x}", Sha256::digest(serde_json::to_vec(v).unwrap()))
}
pub fn file(path: &Path) -> Result<Value> {
    let m = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(json!([0, "missing"])),
        Err(e) => return Err(e.into()),
    };
    if m.is_symlink() {
        let target = match path.canonicalize() {
            Ok(target) => target,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => path.to_owned(),
            Err(error) => return Err(error.into()),
        };
        if target.is_dir() {
            return Err(format!("Tooling receipt input {} is a directory symlink. Use a real checkout-local input directory before caching tooling checks.",path.display()).into());
        }
        let link = fs::read_link(path)?;
        let resolved = if link.is_absolute() {
            link.clone()
        } else {
            path.parent().unwrap().join(&link)
        };
        return Ok(json!([m.mode(), digest(&json!([link, file(&resolved)?]))]));
    }
    if !m.is_file() {
        return Ok(json!([
            m.mode(),
            if m.is_dir() { "directory" } else { "special" }
        ]));
    }
    let mut hash = Sha256::new();
    let mut f = File::open(path)?;
    let mut buf = [0u8; 1024 * 64];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hash.update(&buf[..n]);
    }
    Ok(json!([m.mode(), format!("{:x}", hash.finalize())]))
}
pub fn tree(root: &Path) -> Result<Value> {
    for p in root.ancestors().skip(1) {
        if p.is_symlink() {
            file(p)?;
        }
    }
    let first = file(root)?;
    let mut out = vec![json!([".", first])];
    if first[1] == "missing" {
        return Ok(json!(out));
    }
    fn walk(base: &Path, dir: &Path, out: &mut Vec<Value>) -> Result<()> {
        let mut entries = fs::read_dir(dir)?.collect::<std::io::Result<Vec<_>>>()?;
        entries.sort_by_key(|x| x.file_name());
        let mut dirs = vec![];
        for e in entries {
            let p = e.path();
            if p.is_dir() {
                if p.is_symlink() {
                    file(&p)?;
                }
                if e.file_name() != "__pycache__" {
                    dirs.push(p)
                }
            } else {
                let n = p.to_string_lossy();
                if !n.ends_with(".pyc") && !n.ends_with(".pyo") {
                    out.push(json!([p.strip_prefix(base).unwrap(), file(&p)?]))
                }
            }
        }
        for d in dirs {
            walk(base, &d, out)?
        }
        Ok(())
    }
    walk(root, root, &mut out)?;
    Ok(json!(out))
}
fn git(repo: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let p = Command::new("git").args(args).current_dir(repo).output()?;
    if !p.status.success() {
        return Err(String::from_utf8_lossy(&p.stderr).into_owned().into());
    }
    Ok(p.stdout)
}
pub fn source(repo: &Path) -> Result<String> {
    let mut names = BTreeSet::new();
    for (args, untracked) in [
        (vec!["ls-files", "-z"], false),
        (
            vec!["ls-files", "--others", "--exclude-standard", "-z"],
            true,
        ),
    ] {
        for b in git(repo, &args)?.split(|b| *b == 0) {
            if b.is_empty() {
                continue;
            }
            let n = String::from_utf8_lossy(b).into_owned();
            if !untracked || !n.ends_with(".md") {
                names.insert(n);
            }
        }
    }
    Ok(digest(&json!(names
        .into_iter()
        .map(|n| Ok(json!([n, file(&repo.join(&n))?])))
        .collect::<Result<Vec<_>>>()?)))
}
pub fn generated(repo: &Path) -> Result<String> {
    Ok(digest(&json!(crate::ARCHES
        .iter()
        .map(|a| Ok(json!([
            a,
            tree(&repo.join(format!("build-{a}/sdk/include")))?
        ])))
        .collect::<Result<Vec<_>>>()?)))
}
/// Caller supplies runtime discovery facts, not fingerprints; file/tree bytes are read here.
pub fn runtime(a: &Value) -> Result<String> {
    let repo = a["repo"].as_str().ok_or("missing runtime repo")?;
    let mut env = a["env"]
        .as_object()
        .ok_or("missing runtime environment")?
        .clone();
    env.retain(|k, _| !listed("_VOLATILE_ENV", k));
    for v in env.values_mut() {
        if let Some(s) = v.as_str() {
            *v = json!(s.replace(repo, "<repo>"))
        }
    }
    let mut files = vec![];
    for p in a["files"].as_array().ok_or("missing runtime files")? {
        files.push(json!([
            p[0],
            if let Some(path) = p[1].as_str() {
                file(Path::new(path))?
            } else {
                Value::Null
            }
        ]))
    }
    let mut trees = vec![];
    for p in a["trees"].as_array().ok_or("missing runtime trees")? {
        trees.push(json!([
            p[0],
            tree(Path::new(p[1].as_str().ok_or("invalid runtime tree")?))?
        ]))
    }
    Ok(digest(
        &json!({"env":env,"files":files,"trees":trees,"facts":a["facts"]}),
    ))
}
pub struct Session {
    repo: PathBuf,
    root: PathBuf,
    before: Value,
    generated_before: String,
    generated_checked: Option<String>,
    key: String,
    _lock: File,
    pending: BTreeMap<String, Value>,
    generated_keys: BTreeSet<String>,
    hits: BTreeSet<String>,
}
impl Session {
    pub fn new(repo: &Path, before: Value) -> Result<Self> {
        let common = String::from_utf8(git(repo, &["rev-parse", "--git-common-dir"])?)
            .map_err(|e| Error::new(e.to_string()))?;
        let root = repo
            .join(common.trim())
            .canonicalize()?
            .join("vibeos-tooling-receipts");
        fs::create_dir_all(&root)?;
        let generated_before = generated(repo)?;
        let key = digest(&json!([1, before]));
        let lock = OpenOptions::new()
            .create(true)
            .append(true)
            .open(root.join(format!("{key}.lock")))?;
        if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX) } != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(Self {
            repo: repo.into(),
            root,
            before,
            generated_before,
            generated_checked: None,
            key,
            _lock: lock,
            pending: BTreeMap::new(),
            generated_keys: BTreeSet::new(),
            hits: BTreeSet::new(),
        })
    }
    pub fn hits(&self) -> &BTreeSet<String> {
        &self.hits
    }
    pub fn mark_rerun(&mut self, label: &str) {
        self.hits.remove(label);
    }
    pub fn lookup(&mut self, a: &Value) -> Result<Value> {
        let label = a["label"].as_str().ok_or("missing receipt label")?;
        if !eligibility(label) {
            return Ok(json!({"eligible":false}));
        }
        let normalize = |s: &str| s.replace(&*self.repo.to_string_lossy(), "<repo>");
        let argv = a["cmd"]
            .as_array()
            .ok_or("missing receipt command")?
            .iter()
            .map(|v| normalize(v.as_str().unwrap_or("")))
            .collect::<Vec<_>>();
        let gen = if listed("GENERATED_INPUT_CHECKS", label) {
            Some(&self.generated_before)
        } else {
            None
        };
        let key = digest(&json!([
            self.key,
            label,
            argv,
            normalize(a["cwd"].as_str().unwrap_or(&self.repo.to_string_lossy())),
            a["env_pp"],
            gen
        ]));
        if gen.is_some() {
            self.generated_keys.insert(key.clone());
        }
        let path = self.root.join(format!("{key}.json"));
        let mut out = json!({"eligible":true,"key":key});
        if path.exists() {
            let data = fs::read(&path)
                .and_then(|b| serde_json::from_slice::<Value>(&b).map_err(std::io::Error::other))
                .map_err(|e| {
                    Error::new(format!("Corrupt tooling receipt {}: {e}", path.display()))
                })?;
            if data["version"] != 1
                || data["key"] != key
                || data["label"] != label
                || data["ok"] != true
                || !data["output"].is_string()
                || !data["created"].is_number()
            {
                return Err(format!(
                    "Corrupt tooling receipt {}: invalid receipt fields",
                    path.display()
                )
                .into());
            }
            let age = now() - data["created"].as_f64().unwrap_or(0.);
            if (0. ..86400.).contains(&age) {
                self.hits.insert(label.into());
                out["result"] = json!([label, true, data["output"]]);
            }
        }
        Ok(out)
    }
    pub fn put(&mut self, a: &Value) -> Result<Value> {
        let result = &a["result"];
        if result[1] == true {
            self.pending.insert(
                a["key"].as_str().ok_or("missing receipt key")?.into(),
                result.clone(),
            );
        }
        Ok(Value::Null)
    }
    pub fn stale(&mut self) -> Result<Value> {
        let current = generated(&self.repo)?;
        let changed = current != self.generated_before;
        self.generated_checked = Some(current);
        Ok(json!(self
            .hits
            .iter()
            .filter(|l| changed && listed("GENERATED_INPUT_CHECKS", l))
            .collect::<Vec<_>>()))
    }
    pub fn finish(&mut self, before: &Value) -> Result<Value> {
        if before != &self.before {
            return Err("Tooling inputs changed during checks; no receipts published. Rerun check-tooling. Usual cause: files were edited or committed WHILE these checks ran — run `land`/`check-tooling` in the foreground and make no edits until it returns.".into());
        }
        let changed = generated(&self.repo)? != self.generated_before;
        let validated = self.generated_checked.as_ref() == Some(&self.generated_before);
        if changed && !validated && self.hits.contains("pytest:tools/rustconv/tests") {
            return Err(
                "Generated tooling inputs changed after receipt reuse; rerun check-tooling.".into(),
            );
        }
        for (key, r) in &self.pending {
            if changed && self.generated_keys.contains(key) {
                continue;
            }
            let mut tmp = tempfile::NamedTempFile::new_in(&self.root)?;
            serde_json::to_writer(
                &mut tmp,
                &json!({"version":1,"key":key,"label":r[0],"ok":true,"output":r[2],"created":now()}),
            )?;
            tmp.flush()?;
            tmp.persist(self.root.join(format!("{key}.json")))
                .map_err(|e| Error::new(e.to_string()))?;
        }
        for e in fs::read_dir(&self.root)? {
            let p = e?.path();
            if p.extension().is_some_and(|s| s == "json")
                && fs::metadata(&p)
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| SystemTime::now().duration_since(t).ok())
                    .is_some_and(|age| age.as_secs() > 86400)
            {
                let _ = fs::remove_file(p);
            }
        }
        Ok(json!({"hits":self.hits}))
    }
}
static SESSION: Mutex<Option<Session>> = Mutex::new(None);
pub fn dispatch(root: &Path, r: &Value) -> Result<Value> {
    let a = &r["kwargs"];
    let op = r["op"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches("tooling_receipts_");
    let repo = a["repo"].as_str().map(Path::new).unwrap_or(root);
    match op {
        "policy" => Ok(POLICY.clone()),
        "eligible" => Ok(json!(eligibility(a["label"].as_str().unwrap_or("")))),
        "file" => file(Path::new(a["path"].as_str().ok_or("missing receipt path")?)),
        "tree" => tree(Path::new(a["path"].as_str().ok_or("missing receipt path")?)),
        "source" => Ok(json!(source(repo)?)),
        "generated" => Ok(json!(generated(repo)?)),
        "runtime" => Ok(json!(runtime(a)?)),
        "begin" => {
            let s = Session::new(repo, a["identity"].clone())?;
            let info = json!({"root":s.root,"key":s.key});
            *SESSION.lock().unwrap() = Some(s);
            Ok(info)
        }
        _ => {
            let mut guard = SESSION.lock().unwrap();
            if op == "close" {
                *guard = None;
                return Ok(Value::Null);
            }
            let s = guard.as_mut().ok_or("tooling receipt session closed")?;
            match op {
                "lookup" => s.lookup(a),
                "put" => s.put(a),
                "stale" => s.stale(),
                "rerun" => {
                    s.hits.remove(a["label"].as_str().unwrap_or(""));
                    Ok(Value::Null)
                }
                "hits" => Ok(json!(s.hits)),
                "finish" => {
                    let v = s.finish(&a["identity"]);
                    *guard = None;
                    v
                }
                _ => Err(format!("unknown tooling receipt operation {op}").into()),
            }
        }
    }
}
