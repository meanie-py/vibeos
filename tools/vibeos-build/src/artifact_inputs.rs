//! Conservative, discovery-only Cargo input projection. A projection never authorizes a hit.
use crate::{Error, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::os::unix::fs::MetadataExt;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
    time::Instant,
};

pub const SCHEMA: &str = "vibeos-body-input-projection-v1-spike";
pub const PACKAGE_KEYS: &[&str] = &[
    "name",
    "version",
    "edition",
    "rust-version",
    "authors",
    "description",
    "homepage",
    "repository",
    "license",
    "license-file",
    "readme",
    "keywords",
    "categories",
    "publish",
    "include",
    "exclude",
    "metadata",
    "build",
    "autolib",
    "autobins",
    "autoexamples",
    "autotests",
    "autobenches",
];
pub const TOP_KEYS: &[&str] = &[
    "package",
    "lib",
    "dependencies",
    "dev-dependencies",
    "features",
    "lints",
];
pub const LIB_KEYS: &[&str] = &[
    "name",
    "path",
    "crate-type",
    "required-features",
    "test",
    "doctest",
    "bench",
    "doc",
    "harness",
    "proc-macro",
    "edition",
];
pub const DEP_KEYS: &[&str] = &[
    "path",
    "version",
    "package",
    "features",
    "default-features",
    "optional",
];
pub const PROFILE_KEYS: &[&str] = &[
    "opt-level",
    "debug",
    "split-debuginfo",
    "strip",
    "debug-assertions",
    "overflow-checks",
    "lto",
    "panic",
    "incremental",
    "codegen-units",
    "rpath",
];
pub const CONTEXT_KEYS: &[&str] = &[
    "target",
    "effective_flags",
    "effective_config",
    "toolchain",
    "environment",
    "execution_policy",
];
pub const PACKAGE_ENV: &[&str] = &[
    "CARGO_PKG_NAME",
    "CARGO_PKG_VERSION",
    "CARGO_PKG_VERSION_MAJOR",
    "CARGO_PKG_VERSION_MINOR",
    "CARGO_PKG_VERSION_PATCH",
    "CARGO_PKG_VERSION_PRE",
    "CARGO_PKG_AUTHORS",
    "CARGO_PKG_DESCRIPTION",
    "CARGO_PKG_HOMEPAGE",
    "CARGO_PKG_REPOSITORY",
    "CARGO_PKG_LICENSE",
    "CARGO_PKG_LICENSE_FILE",
    "CARGO_PKG_RUST_VERSION",
    "CARGO_CRATE_NAME",
    "CARGO_PRIMARY_PACKAGE",
];
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct DiscoverOptions {
    pub features: Vec<String>,
    pub default_features: bool,
    pub profile: String,
    pub force_staticlib: bool,
    pub cargo: String,
    pub workspace: Option<PathBuf>,
    pub scratch: Option<PathBuf>,
}
impl Default for DiscoverOptions {
    fn default() -> Self {
        Self {
            features: vec![],
            default_features: true,
            profile: "release".into(),
            force_staticlib: false,
            cargo: "cargo".into(),
            workspace: None,
            scratch: None,
        }
    }
}
fn rx(pattern: &str) -> Regex {
    static PATTERNS: std::sync::OnceLock<std::sync::Mutex<BTreeMap<String, Regex>>> =
        std::sync::OnceLock::new();
    let mut patterns = PATTERNS
        .get_or_init(Default::default)
        .lock()
        .expect("regex cache lock");
    patterns
        .entry(pattern.to_owned())
        .or_insert_with(|| Regex::new(pattern).expect("constant artifact projection regex"))
        .clone()
}
pub fn canonical(value: &Value) -> Result<Vec<u8>> {
    Ok(crate::json::canonical(value)?.into_bytes())
}
pub fn digest(value: &Value) -> Result<String> {
    Ok(hash(&canonical(value)?))
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn object(v: &Value) -> Result<&Map<String, Value>> {
    v.as_object()
        .ok_or_else(|| Error::new("expected TOML table"))
}
fn strings(v: &Value) -> Result<Vec<String>> {
    v.as_array()
        .ok_or("expected list")?
        .iter()
        .map(|s| Ok(s.as_str().ok_or("expected string")?.to_owned()))
        .collect()
}
fn string<'a>(v: &'a Value) -> Result<&'a str> {
    v.as_str().ok_or_else(|| Error::new("expected string"))
}
fn table(v: &Value, name: &str) -> Value {
    v.get(name).cloned().unwrap_or_else(|| json!({}))
}
pub fn check_keys(value: &Value, allowed: &[&str], location: &str) -> Result<()> {
    let unknown: Vec<_> = object(value)?
        .keys()
        .filter(|k| !allowed.contains(&k.as_str()))
        .collect();
    if !unknown.is_empty() {
        return Err(Error::new(format!(
            "{location}: unsupported keys {unknown:?}"
        )));
    }
    Ok(())
}
pub fn toml_literal(value: &Value) -> Result<String> {
    Ok(match value {
        Value::Object(m) => format!(
            "{{ {} }}",
            m.iter()
                .map(|(k, v)| Ok(format!(
                    "{} = {}",
                    crate::json::dumps(&json!(k), true),
                    toml_literal(v)?
                )))
                .collect::<Result<Vec<_>>>()?
                .join(", ")
        ),
        Value::Array(xs) => format!(
            "[{}]",
            xs.iter()
                .map(toml_literal)
                .collect::<Result<Vec<_>>>()?
                .join(", ")
        ),
        Value::Null => return Err("unsupported TOML value NoneType".into()),
        _ => crate::json::canonical(value)?,
    })
}
/// Inline TOML spelling retained for the isolated artifact executor.
pub fn toml_value(value: &Value) -> Result<String> {
    toml_literal(value)
}
fn parse_toml(data: &[u8]) -> Result<Value> {
    let text = std::str::from_utf8(data).map_err(|e| Error::new(e.to_string()))?;
    let value: toml::Value = toml::from_str(text).map_err(|e| Error::new(e.to_string()))?;
    fn convert(v: toml::Value) -> Result<Value> {
        Ok(match v {
            toml::Value::Datetime(_) => return Err("unsupported TOML value datetime".into()),
            toml::Value::Table(t) => Value::Object(
                t.into_iter()
                    .map(|(k, v)| Ok((k, convert(v)?)))
                    .collect::<Result<_>>()?,
            ),
            toml::Value::Array(a) => {
                Value::Array(a.into_iter().map(convert).collect::<Result<_>>()?)
            }
            v => serde_json::to_value(v)?,
        })
    }
    convert(value)
}
/// Resolve symlinks like pathlib.resolve(strict=False), retaining missing path suffixes.
fn resolve(path: &Path) -> Result<PathBuf> {
    let input = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    fn walk(path: &Path, depth: usize) -> Result<PathBuf> {
        if depth > 40 {
            return Err("symlink loop during path resolution".into());
        }
        let mut out = PathBuf::new();
        for part in path.components() {
            match part {
                Component::ParentDir => {
                    out.pop();
                }
                Component::CurDir => {}
                other => {
                    out.push(other.as_os_str());
                    if let Ok(target) = fs::read_link(&out) {
                        let new = if target.is_absolute() {
                            target
                        } else {
                            out.parent().unwrap_or(Path::new("/")).join(target)
                        };
                        out = walk(&new, depth + 1)?;
                    }
                }
            }
        }
        Ok(out)
    }
    walk(&input, 0)
}
pub fn without_comments(text: &str) -> Result<String> {
    let tokens = rx(r##"//|/\*|r(?:#+)?\"|[\"']"##);
    let comment = rx(r"/\*|\*/");
    let character = rx(r"^'(?:\\(?:x[0-9A-Fa-f]{2}|u\{[0-9A-Fa-f_]+\}|.)|[^'\\\n])'");
    let raw = rx(r##"^r(#+)?\""##);
    let mut out = String::new();
    let mut index = 0;
    while index < text.len() {
        let Some(token) = tokens.find_at(text, index) else {
            out.push_str(&text[index..]);
            break;
        };
        out.push_str(&text[index..token.start()]);
        index = token.start();
        let start = index;
        if text[index..].starts_with("//") {
            index = text[index..]
                .find('\n')
                .map(|n| index + n)
                .unwrap_or(text.len());
            out.push_str(&" ".repeat(text[start..index].chars().count()));
        } else if text[index..].starts_with("/*") {
            let mut depth = 1;
            index += 2;
            while index < text.len() && depth > 0 {
                let Some(boundary) = comment.find_at(text, index) else {
                    break;
                };
                if boundary.as_str() == "/*" {
                    depth += 1;
                } else {
                    depth -= 1;
                }
                index = boundary.end();
            }
            if depth > 0 {
                return Err("unterminated Rust block comment".into());
            }
            out.push_str(&" ".repeat(text[start..index].chars().count()));
        } else if let Some(c) = character
            .find(&text[index..])
            .filter(|_| text[index..].starts_with('\''))
        {
            index += c.end();
            out.push_str(&text[start..index]);
        } else if text[index..].starts_with('"') {
            index += 1;
            while index < text.len() {
                let c = text[index..].chars().next().unwrap();
                index += c.len_utf8();
                if c == '\\' {
                    if let Some(next) = text[index..].chars().next() {
                        index += next.len_utf8();
                    }
                } else if c == '"' {
                    break;
                }
            }
            out.push_str(&text[start..index]);
        } else if let Some(c) = raw.captures(&text[index..]) {
            let terminal = format!("\"{}", c.get(1).map(|m| m.as_str()).unwrap_or(""));
            let begin = index + c[0].len();
            let end = text[begin..]
                .find(&terminal)
                .ok_or("unterminated Rust raw string")?;
            index = begin + end + terminal.len();
            out.push_str(&text[start..index]);
        } else {
            let c = text[index..].chars().next().unwrap();
            out.push(c);
            index += c.len_utf8();
        }
    }
    Ok(out)
}
pub fn explain(before: &Value, after: &Value) -> Value {
    fn walk(a: &Value, b: &Value, path: &str, out: &mut Vec<Value>) {
        if let (Some(a), Some(b)) = (a.as_object(), b.as_object()) {
            let keys: BTreeSet<_> = a.keys().chain(b.keys()).collect();
            for k in keys {
                let field = format!("{path}/{k}");
                match(a.get(k),b.get(k)) {
                (Some(a),Some(b))=>walk(a,b,&field,out),
                (a,b)=>out.push(json!({"field":field,"before":a,"after":b,"absent_before":a.is_none(),"absent_after":b.is_none()})),
            }
            }
        } else if a != b {
            out.push(json!({"field":path,"before":a,"after":b}));
        }
    }
    let mut out = vec![];
    walk(before, after, "", &mut out);
    json!(out)
}
pub fn stamp(m: &fs::Metadata) -> Value {
    json!([
        m.dev(),
        m.ino(),
        m.size(),
        m.mtime() as i128 * 1_000_000_000 + m.mtime_nsec() as i128,
        m.ctime() as i128 * 1_000_000_000 + m.ctime_nsec() as i128
    ])
}
pub fn unchanged(discovery: &Value) -> bool {
    if discovery["eligible"] != true {
        return false;
    }
    let Some(stamps) = discovery["provenance"]["stamps"]
        .as_object()
        .filter(|m| !m.is_empty())
    else {
        return false;
    };
    stamps.iter().all(|(path, expected)| {
        let p = path.trim_end_matches('/');
        fs::metadata(if p.is_empty() { "/" } else { p })
            .map(|m| stamp(&m) == *expected)
            .unwrap_or(false)
    })
}
#[derive(Debug)]
pub struct Discovery {
    pub repo: PathBuf,
    pub workspace: PathBuf,
    pub context: Value,
    pub reads: BTreeMap<String, Value>,
    pub stamps: BTreeMap<String, Value>,
    pub manifests: BTreeMap<String, Value>,
    pub roots: BTreeMap<String, PathBuf>,
    pub raw_manifests: BTreeMap<String, String>,
    pub evidence: Value,
    pub data_inputs: BTreeMap<String, Value>,
}
impl Discovery {
    pub fn new(repo: &Path, workspace: &Path, context: Value) -> Result<Self> {
        Ok(Self {
            repo: resolve(repo)?,
            workspace: resolve(workspace)?,
            context,
            reads: BTreeMap::new(),
            stamps: BTreeMap::new(),
            manifests: BTreeMap::new(),
            roots: BTreeMap::new(),
            raw_manifests: BTreeMap::new(),
            evidence: json!({}),
            data_inputs: BTreeMap::new(),
        })
    }
    pub fn logical(&self, path: &Path) -> Result<String> {
        let path = resolve(path)?;
        path.strip_prefix(&self.repo)
            .map(|p| p.to_string_lossy().into_owned())
            .map_err(|_| Error::new(format!("outside repository root: {}", path.display())))
    }
    pub fn read(&mut self, path: &Path) -> Result<Vec<u8>> {
        let logical = self.logical(path)?;
        let before = fs::metadata(path)?;
        let data = fs::read(path)?;
        let after = fs::metadata(path)?;
        let st = stamp(&before);
        if st != stamp(&after) {
            return Err(Error::new(format!(
                "input changed while reading: {logical}"
            )));
        }
        self.stamps
            .insert(resolve(path)?.to_string_lossy().into_owned(), st);
        let record = json!({"sha256":hash(&data),"bytes":data.len()});
        if self.reads.get(&logical).is_some_and(|r| r != &record) {
            return Err(Error::new(format!(
                "input changed during discovery: {logical}"
            )));
        }
        self.reads.insert(logical, record);
        Ok(data)
    }
    pub fn load(&mut self, manifest: &Path) -> Result<String> {
        let manifest = resolve(manifest)?;
        let parent = manifest.parent().ok_or("manifest has no parent")?;
        let logical = self.logical(parent)?;
        if self.manifests.contains_key(&logical) {
            return Ok(logical);
        }
        let data = self.read(&manifest)?;
        let raw = parse_toml(&data)?;
        check_keys(&raw, TOP_KEYS, &self.logical(&manifest)?)?;
        let package = table(&raw, "package");
        check_keys(&package, PACKAGE_KEYS, &format!("{logical}/package"))?;
        if !package["name"].is_string() || !package["version"].is_string() {
            return Err(Error::new(format!(
                "{logical}: package name/version must be explicit strings"
            )));
        }
        if object(&package)?
            .values()
            .any(|v| v.get("workspace").is_some_and(truthy))
        {
            return Err(Error::new(format!(
                "{logical}: inherited workspace package fields unsupported"
            )));
        }
        check_keys(&table(&raw, "lib"), LIB_KEYS, &format!("{logical}/lib"))?;
        let lints = table(&raw, "lints");
        if !lints.is_object()
            || lints.get("workspace").is_some_and(truthy)
            || object(&lints)?.values().any(|v| !v.is_object())
        {
            return Err(Error::new(format!(
                "{logical}: only package-local [lints.<tool>] tables supported"
            )));
        }
        self.manifests.insert(logical.clone(), raw.clone());
        self.roots.insert(logical.clone(), parent.to_owned());
        self.raw_manifests.insert(
            logical.clone(),
            String::from_utf8(data).map_err(|e| Error::new(e.to_string()))?,
        );
        for (name, dep) in object(&table(&raw, "dependencies"))? {
            if !dep.is_object() {
                return Err(Error::new(format!(
                    "{logical}/{name}: only path dependencies supported"
                )));
            }
            check_keys(dep, DEP_KEYS, &format!("{logical}/{name}"))?;
            let Some(path) = dep.get("path") else {
                return Err(Error::new(format!(
                    "{logical}/{name}: only explicit path dependencies supported"
                )));
            };
            let child = self.load(&parent.join(string(path)?).join("Cargo.toml"))?;
            if self.manifests[&child]["package"]["name"]
                != dep.get("package").cloned().unwrap_or_else(|| json!(name))
            {
                return Err(Error::new(format!(
                    "{logical}/{name}: dependency package name mismatch"
                )));
            }
        }
        Ok(logical)
    }
    pub fn metadata(
        &mut self,
        selected: &str,
        features: &[String],
        default_features: bool,
        cargo: &str,
        resolver: &str,
        scratch: Option<&Path>,
    ) -> Result<Value> {
        let mut builder = tempfile::Builder::new();
        builder.prefix("vibeos-artifact-resolution-");
        let private = match scratch {
            Some(p) => builder.tempdir_in(p)?,
            None => builder.tempdir()?,
        };
        let dir = private.path();
        let ids: BTreeMap<String, String> = self
            .manifests
            .keys()
            .enumerate()
            .map(|(i, k)| (k.clone(), format!("p{i}")))
            .collect();
        let excluded: Vec<_> = ids
            .iter()
            .filter(|(k, _)| k.as_str() != selected)
            .map(|(_, v)| v.clone())
            .collect();
        fs::write(
            dir.join("Cargo.toml"),
            format!(
                "[workspace]\nresolver = {}\nmembers = {}\nexclude = {}\n",
                toml_literal(&json!(resolver))?,
                toml_literal(&json!([ids[selected]]))?,
                toml_literal(&json!(excluded))?
            ),
        )?;
        let mut lock_entries = vec![];
        for (key, raw) in &self.manifests {
            let dest = dir.join(&ids[key]);
            fs::create_dir(&dest)?;
            let package = &raw["package"];
            let mut lines = vec![
                "[package]".to_owned(),
                format!("name = {}", toml_literal(&package["name"])?),
                format!("version = {}", toml_literal(&package["version"])?),
                format!(
                    "edition = {}",
                    toml_literal(package.get("edition").unwrap_or(&json!("2015")))?
                ),
                "build = false".into(),
                "[lib]".into(),
                "path = \"lib.rs\"".into(),
            ];
            if let Some(name) = raw.get("lib").and_then(|v| v.get("name")) {
                lines.push(format!("name = {}", toml_literal(name)?));
            }
            fs::write(dest.join("lib.rs"), "pub fn resolution_stub() {}\n")?;
            let deps = table(raw, "dependencies");
            let mut dep_names = vec![];
            if !object(&deps)?.is_empty() {
                lines.push("[dependencies]".into());
            }
            for (name, dep) in object(&deps)? {
                let child = self.logical(&self.roots[key].join(string(&dep["path"])?))?;
                let mut replacement = dep.clone();
                replacement["path"] = json!(format!("../{}", ids[&child]));
                lines.push(format!(
                    "{} = {}",
                    toml_literal(&json!(name))?,
                    toml_literal(&replacement)?
                ));
                dep_names.push(self.manifests[&child]["package"]["name"].clone());
            }
            if raw.get("features").is_some_and(truthy) {
                lines.push("[features]".into());
                for (name, values) in object(&raw["features"])? {
                    lines.push(format!(
                        "{} = {}",
                        toml_literal(&json!(name))?,
                        toml_literal(values)?
                    ));
                }
            }
            fs::write(dest.join("Cargo.toml"), lines.join("\n") + "\n")?;
            dep_names.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
            lock_entries.push(json!({"name":package["name"],"version":package["version"],"dependencies":dep_names}));
        }
        lock_entries.sort_by(|a, b| {
            (a["name"].as_str(), a["version"].as_str())
                .cmp(&(b["name"].as_str(), b["version"].as_str()))
        });
        let mut lock = vec!["version = 4".to_owned()];
        for entry in lock_entries {
            lock.extend([
                "[[package]]".into(),
                format!("name = {}", toml_literal(&entry["name"])?),
                format!("version = {}", toml_literal(&entry["version"])?),
            ]);
            if truthy(&entry["dependencies"]) {
                lock.push(format!(
                    "dependencies = {}",
                    toml_literal(&entry["dependencies"])?
                ));
            }
        }
        fs::write(dir.join("Cargo.lock"), lock.join("\n") + "\n")?;
        let mut arguments = vec![
            "metadata".to_owned(),
            "--format-version".into(),
            "1".into(),
            "--offline".into(),
            "--locked".into(),
            "--manifest-path".into(),
            dir.join(&ids[selected])
                .join("Cargo.toml")
                .to_string_lossy()
                .into_owned(),
        ];
        if !features.is_empty() {
            arguments.extend(["--features".into(), features.join(",")]);
        }
        if !default_features {
            arguments.push("--no-default-features".into());
        }
        let mut environment: BTreeMap<String, String> = std::env::vars()
            .filter(|(k, _)| {
                [
                    "PATH",
                    "HOME",
                    "RUSTUP_HOME",
                    "RUSTUP_TOOLCHAIN",
                    "LD_LIBRARY_PATH",
                ]
                .contains(&k.as_str())
            })
            .collect();
        environment.insert(
            "CARGO_HOME".into(),
            dir.join("cargo-home").to_string_lossy().into_owned(),
        );
        environment.insert(
            "CARGO_TARGET_DIR".into(),
            dir.join("target").to_string_lossy().into_owned(),
        );
        for ancestor in dir.ancestors() {
            if ["config", "config.toml"]
                .iter()
                .any(|n| ancestor.join(".cargo").join(n).exists())
            {
                return Err("private metadata has ambient ancestor Cargo configuration".into());
            }
        }
        let version = Command::new(cargo)
            .arg("-Vv")
            .current_dir(dir)
            .env_clear()
            .envs(&environment)
            .output()?;
        let identity = String::from_utf8(version.stdout).map_err(|e| Error::new(e.to_string()))?;
        if !version.status.success() || !identity.starts_with("cargo 1.92.0 ") {
            return Err(Error::new(format!(
                "resolution requires audited pinned Cargo 1.92.0: {}",
                identity.trim()
            )));
        }
        let completed = Command::new(cargo)
            .args(&arguments)
            .current_dir(dir)
            .env_clear()
            .envs(&environment)
            .output()?;
        let stderr = String::from_utf8(completed.stderr).map_err(|e| Error::new(e.to_string()))?;
        let command: Vec<_> = std::iter::once(cargo.to_owned())
            .chain(arguments)
            .map(|arg| arg.replace(&dir.to_string_lossy().to_string(), "<private-resolution>"))
            .collect();
        self.evidence = json!({"command":command,"exit_code":completed.status.code().unwrap_or(-1),"stderr":stderr,"cargo_identity":identity,"mirror_policy":"selected-member-only; normal path edges; stub sources; no dev edges; Cargo locked/offline"});
        if !completed.status.success() {
            return Err(Error::new(format!(
                "private locked Cargo resolution failed: {}",
                stderr.trim()
            )));
        }
        let metadata: Value = serde_json::from_slice(&completed.stdout)?;
        let reverse: BTreeMap<_, _> = ids
            .iter()
            .map(|(k, v)| {
                (
                    dir.join(v)
                        .join("Cargo.toml")
                        .to_string_lossy()
                        .into_owned(),
                    k.clone(),
                )
            })
            .collect();
        let mut package_keys = BTreeMap::new();
        for package in metadata["packages"]
            .as_array()
            .ok_or("metadata packages missing")?
        {
            let path = string(&package["manifest_path"])?;
            let key = reverse
                .get(path)
                .ok_or("metadata package outside private mirror")?;
            package_keys.insert(string(&package["id"])?.to_owned(), key.clone());
        }
        let mut nodes = BTreeMap::new();
        for node in metadata["resolve"]["nodes"]
            .as_array()
            .ok_or("metadata resolution missing")?
        {
            let id = string(&node["id"])?;
            let key = package_keys
                .get(id)
                .ok_or("metadata node outside private mirror")?;
            nodes.insert(key.clone(), node.clone());
        }
        let mut active = BTreeSet::new();
        let mut pending = vec![selected.to_owned()];
        while let Some(key) = pending.pop() {
            if !active.insert(key.clone()) {
                continue;
            }
            for edge in nodes.get(&key).ok_or("metadata node missing")?["deps"]
                .as_array()
                .ok_or("metadata deps missing")?
            {
                pending.push(
                    package_keys
                        .get(string(&edge["pkg"])?)
                        .ok_or("metadata edge outside mirror")?
                        .clone(),
                );
            }
        }
        let mut out = Map::new();
        for key in active {
            let node = &nodes[&key];
            let mut features = strings(&node["features"])?;
            features.sort();
            for feature in &features {
                if self.manifests[&key]
                    .get("features")
                    .and_then(|f| f.get(feature))
                    .map(strings)
                    .transpose()?
                    .unwrap_or_default()
                    .iter()
                    .any(|t| t.contains("?/"))
                {
                    return Err(Error::new(format!("{key}: active weak optional feature forwarding unsupported; Cargo metadata over-activates it")));
                }
            }
            let mut edges = vec![];
            for edge in node["deps"].as_array().ok_or("metadata deps missing")? {
                edges.push(json!({"name":edge["name"],"package":package_keys.get(string(&edge["pkg"])?) .ok_or("metadata edge outside mirror")?}));
            }
            edges.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
            out.insert(key, json!({"features":features,"edges":edges}));
        }
        Ok(Value::Object(out))
    }
    pub fn inventory(&mut self, key: &str) -> Result<Value> {
        let root = self.roots[key].clone();
        let raw = self.manifests[key].clone();
        let library = table(&raw, "lib");
        if raw["package"]
            .get("build")
            .is_some_and(|v| !v.is_null() && *v != false)
            || root.join("build.rs").exists()
        {
            return Err(Error::new(format!("{key}: build script unsupported")));
        }
        if library.get("proc-macro").is_some_and(truthy) {
            return Err(Error::new(format!("{key}: proc macro unsupported")));
        }
        let source = root.join(
            library
                .get("path")
                .map(string)
                .transpose()?
                .unwrap_or("src/lib.rs"),
        );
        if !resolve(&source)?.starts_with(&root) || !source.is_file() {
            return Err(Error::new(format!(
                "{key}: library source missing or outside package"
            )));
        }
        let first = source
            .strip_prefix(&root)
            .map_err(|_| Error::new("source outside package"))?
            .components()
            .next();
        if first.is_some_and(|c| c.as_os_str() == "target" || c.as_os_str() == ".git") {
            return Err(Error::new(format!(
                "{key}: output/VCS directory used as source root"
            )));
        }
        let mut files = vec![];
        let mut dirs = vec![];
        let mut pending = vec![root.clone()];
        while let Some(current) = pending.pop() {
            let relative = current.strip_prefix(&root).unwrap();
            dirs.push(if relative.as_os_str().is_empty() {
                ".".to_owned()
            } else {
                relative.to_string_lossy().into_owned()
            });
            self.stamps.insert(
                format!("{}/", resolve(&current)?.display()),
                stamp(&fs::metadata(&current)?),
            );
            let mut entries = fs::read_dir(&current)?.collect::<std::io::Result<Vec<_>>>()?;
            entries.sort_by_key(|e| e.file_name());
            let mut children = vec![];
            for entry in entries {
                let path = entry.path();
                let name = entry.file_name();
                // os.walk's top-level excluded directories are never source inputs.
                if current == root && ["target", ".git"].iter().any(|n| name == *n) && path.is_dir()
                {
                    continue;
                }
                if path.is_symlink() {
                    return Err(Error::new(format!(
                        "{key}: source symlink unsupported: {}",
                        path.strip_prefix(&root).unwrap().display()
                    )));
                }
                if entry.file_type()?.is_dir() {
                    children.push(path);
                    continue;
                }
                if !path.is_file() {
                    return Err(Error::new(format!(
                        "{key}: non-regular source input {}",
                        name.to_string_lossy()
                    )));
                }
                if path == root.join("Cargo.toml") {
                    continue;
                }
                let data = self.read(&path)?;
                let rel = path
                    .strip_prefix(&root)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                if path.extension().is_some_and(|e| e == "rs") {
                    self.audit_source(key, &path, &data)?;
                }
                files.push(json!({"path":rel,"sha256":hash(&data),"bytes":data.len()}));
            }
            pending.extend(children.into_iter().rev());
        }
        dirs.sort();
        Ok(
            json!({"directories":dirs,"files":files,"exclusions":["top-level target/","top-level .git/"]}),
        )
    }
    pub fn audit_source(&mut self, key: &str, path: &Path, data: &[u8]) -> Result<()> {
        let mut text =
            without_comments(std::str::from_utf8(data).map_err(|e| Error::new(e.to_string()))?)?;
        let root = self.roots[key].clone();
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if rx(r"#\s*\[\s*cfg_attr\b[^]]*\blink\s*\(").is_match(&text) {
            return Err(Error::new(format!(
                "{key}: conditional native link attribute unsupported in {name}"
            )));
        }
        for native in rx(r"#\s*\[\s*link\s*\(([^]]*)\)\s*\]").captures_iter(&text) {
            let mut attributes = BTreeMap::new();
            for field in native[1].trim().trim_end_matches(',').split(',') {
                let pattern = rx(r#"^\s*([a-z_]+)\s*=\s*"([^"\\]*)"\s*$"#);
                let Some(cap) = pattern.captures(field) else {
                    return Err(Error::new(format!(
                        "{key}: native library attribute unsupported in {name}"
                    )));
                };
                if attributes
                    .insert(cap[1].to_owned(), cap[2].to_owned())
                    .is_some()
                {
                    return Err(Error::new(format!(
                        "{key}: native library attribute unsupported in {name}"
                    )));
                }
            }
            let kind = attributes
                .get("kind")
                .map(|s| s.as_str())
                .unwrap_or("dylib");
            if attributes.get("name").is_none_or(|s| s.is_empty())
                || attributes
                    .keys()
                    .any(|k| !["name", "kind", "modifiers"].contains(&k.as_str()))
                || !["dylib", "raw-dylib"].contains(&kind)
                || attributes
                    .get("modifiers")
                    .is_some_and(|m| kind != "raw-dylib" || m != "+verbatim")
            {
                return Err(Error::new(format!(
                    "{key}: native library input unsupported in {name}"
                )));
            }
        }
        if rx(r"\.(?:incbin|include)\s").is_match(&text) {
            return Err(Error::new(format!(
                "{key}: assembly external input unsupported in {name}"
            )));
        }
        let embeds = rx(concat!(
            r"\b(include_str|include_bytes)\s*!\s*\(\s*concat\s*!\s*\(\s*env\s*!\s*",
            r#"\(\s*"CARGO_MANIFEST_DIR"\s*\)\s*,\s*"([^"\\]*)"\s*,?\s*\)\s*,?\s*\)"#
        ));
        let matches: Vec<_> = embeds
            .captures_iter(&text)
            .map(|c| (c[0].to_owned(), c[2].to_owned()))
            .collect();
        for (matched, suffix) in matches {
            let target = resolve(Path::new(&format!("{}{suffix}", root.display())))?;
            let rel = target.strip_prefix(&self.repo).ok();
            let components: Vec<_> = rel
                .into_iter()
                .flat_map(|p| p.components())
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect();
            if components.is_empty()
                || ["target", ".git"].contains(&components[0].as_str())
                || components[0].starts_with("build-")
                || components.iter().any(|c| c == "target")
                || !target.is_file()
                || target.is_symlink()
            {
                return Err(Error::new(format!(
                    "{key}: data include outside the repository sources in {name}"
                )));
            }
            if !target.starts_with(&root) {
                let bytes = self.read(&target)?;
                let logical = self.logical(&target)?;
                self.data_inputs
                    .insert(logical, json!({"sha256":hash(&bytes),"bytes":bytes.len()}));
            }
            text = text.replacen(&matched, &" ".repeat(matched.chars().count()), 1);
        }
        let includes = rx(r"\b(include|include_str|include_bytes)\s*!\s*[({\[]([^)}\]]*)[)}\]]");
        for cap in includes.captures_iter(&text) {
            let value = cap[2].trim();
            if !rx(r#"^"[^"\\]*"\s*,?$"#).is_match(value) {
                return Err(Error::new(format!(
                    "{key}: computed source include unsupported in {name}"
                )));
            }
            let literal = value.trim_end_matches(',').trim();
            let target = resolve(&path.parent().unwrap().join(&literal[1..literal.len() - 1]))?;
            let rel = target.strip_prefix(&root).ok();
            if rel.is_none()
                || target
                    .file_name()
                    .is_some_and(|n| n == "Cargo.toml" || n == "Cargo.lock")
                || rel.is_some_and(|p| {
                    p.components()
                        .any(|c| c.as_os_str() == "target" || c.as_os_str() == ".git")
                })
            {
                return Err(Error::new(format!(
                    "{key}: external/manifest/output include unsupported in {name}"
                )));
            }
        }
        for cap in rx(r#"\bpath\s*=\s*"([^"\\]*)""#).captures_iter(&text) {
            if !resolve(&path.parent().unwrap().join(&cap[1]))?.starts_with(&root) {
                return Err(Error::new(format!(
                    "{key}: external module path unsupported in {name}"
                )));
            }
        }
        for cap in rx(r"\b(?:env|option_env)\s*!\s*[({\[]([^)}\]]*)[)}\]]").captures_iter(&text) {
            let pattern = rx(r#"^\s*"([^"\\]*)""#);
            let Some(literal) = pattern.captures(&cap[1]) else {
                return Err(Error::new(format!(
                    "{key}: computed environment observation unsupported"
                )));
            };
            let variable = &literal[1];
            if !PACKAGE_ENV.contains(&variable)
                && !self
                    .context
                    .get("environment")
                    .and_then(|v| v.as_object())
                    .is_some_and(|m| m.contains_key(variable))
            {
                return Err(Error::new(format!(
                    "{key}: environment observation not declared: {variable}"
                )));
            }
        }
        Ok(())
    }
}
fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(m) => !m.is_empty(),
        Value::Number(n) => n.as_f64() != Some(0.0),
    }
}
fn project(
    state: &mut Discovery,
    body: &str,
    options: &DiscoverOptions,
    result: &mut Value,
) -> Result<()> {
    let workspace_data = state.read(&state.workspace.join("Cargo.toml"))?;
    let root = parse_toml(&workspace_data)?;
    check_keys(&root, &["workspace", "profile"], "workspace manifest")?;
    check_keys(
        &table(&root, "workspace"),
        &[
            "members",
            "exclude",
            "default-members",
            "resolver",
            "package",
            "dependencies",
            "metadata",
        ],
        "workspace",
    )?;
    let workspace_table = table(&root, "workspace");
    let resolver = workspace_table
        .get("resolver")
        .cloned()
        .unwrap_or_else(|| json!("1"));
    if resolver != "2" {
        return Err("only explicit Cargo resolver 2 is audited".into());
    }
    let selected = state.load(&state.workspace.join(body).join("Cargo.toml"))?;
    let names: Vec<String> = state
        .manifests
        .values()
        .map(|raw| Ok(string(&raw["package"]["name"])?.to_owned()))
        .collect::<Result<_>>()?;
    if names.iter().collect::<BTreeSet<_>>().len() != names.len() {
        return Err("duplicate package names/version disambiguation unsupported".into());
    }
    let lock_data = state.read(&state.workspace.join("Cargo.lock"))?;
    let lock = parse_toml(&lock_data)?;
    if ![Some(3), Some(4)].contains(&lock["version"].as_i64()) {
        return Err("unsupported Cargo lock schema".into());
    }
    let features: Vec<_> = options
        .features
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let resolved = state.metadata(
        &selected,
        &features,
        options.default_features,
        &options.cargo,
        string(&resolver)?,
        options.scratch.as_deref(),
    )?;
    let mut lock_projection = vec![];
    for (key, node) in object(&resolved)? {
        let package = &state.manifests[key]["package"];
        let candidates: Vec<_> = lock["package"]
            .as_array()
            .ok_or("lock packages missing")?
            .iter()
            .filter(|entry| {
                entry["name"] == package["name"]
                    && entry["version"] == package["version"]
                    && entry.get("source").is_none()
            })
            .collect();
        if candidates.len() != 1 {
            return Err(Error::new(format!(
                "{key}: existing lock does not pin the path package uniquely"
            )));
        }
        for edge in node["edges"].as_array().ok_or("resolution edges missing")? {
            let child = &state.manifests[string(&edge["package"])?]["package"];
            let name = string(&child["name"])?;
            let prefix = format!("{} {}", name, string(&child["version"])?);
            if !candidates[0]
                .get("dependencies")
                .map(strings)
                .transpose()?
                .unwrap_or_default()
                .iter()
                .any(|d| d == name || d.starts_with(&prefix))
            {
                return Err(Error::new(format!(
                    "{key}: existing lock is missing resolved edge to {name}"
                )));
            }
        }
        lock_projection.push(json!({"name":package["name"],"version":package["version"],"source":format!("path:{key}"),"edges":node["edges"]}));
    }
    if !["release", "dev"].contains(&options.profile.as_str()) {
        return Err("only built-in release/dev profiles are audited".into());
    }
    let profiles = table(&root, "profile");
    let selected_profile = profiles
        .get(&options.profile)
        .cloned()
        .unwrap_or_else(|| json!({}));
    let mut profile_keys = PROFILE_KEYS.to_vec();
    profile_keys.extend(["package", "build-override"]);
    check_keys(
        &selected_profile,
        &profile_keys,
        &format!("profile.{}", options.profile),
    )?;
    let overrides = table(&selected_profile, "package");
    if overrides.get("*").is_some() {
        return Err("wildcard profile override workspace membership semantics unsupported".into());
    }
    let mut projected_profile = Map::new();
    for (k, v) in object(&selected_profile)? {
        if PROFILE_KEYS.contains(&k.as_str()) {
            projected_profile.insert(k.clone(), v.clone());
        }
    }
    let mut package_profiles = Map::new();
    for key in object(&resolved)?.keys() {
        let name = string(&state.manifests[key]["package"]["name"])?;
        if let Some(value) = overrides.get(name) {
            check_keys(
                value,
                PROFILE_KEYS,
                &format!("profile.{}.package.{name}", options.profile),
            )?;
            package_profiles.insert(name.to_owned(), value.clone());
        }
    }
    projected_profile.insert("package".into(), Value::Object(package_profiles));
    for selector in object(&overrides)?.keys() {
        if !names.contains(selector)
            && names.iter().any(|name| {
                selector.starts_with(&format!("{name}:"))
                    || selector.starts_with(&format!("{name}@"))
            })
        {
            return Err("version-qualified profile package override unsupported".into());
        }
    }
    let mut packages = Map::new();
    for (key, node) in object(&resolved)? {
        let raw = state.manifests[key].clone();
        let inventory = state.inventory(key)?;
        packages.insert(key.clone(),json!({"package":raw["package"],"lib":table(&raw,"lib"),"lints":table(&raw,"lints"),"features":node["features"],"edges":node["edges"],"inventory":inventory}));
    }
    let mut descriptor = json!({"schema":SCHEMA,"request":{"package":selected,"features":features,"default_features":options.default_features,"profile":options.profile,"invocation":if options.force_staticlib{"cargo rustc --lib --crate-type staticlib"}else{"cargo build --lib"}},"resolver":resolver,"resolution_cargo":state.evidence["cargo_identity"],"profiles":projected_profile,"packages":packages,"lock":lock_projection});
    if !state.data_inputs.is_empty() {
        descriptor["data_inputs"] = json!(state.data_inputs);
    }
    result["source_projection_key"] = json!(digest(&descriptor)?);
    let mut full = descriptor.clone();
    full["context"] = state.context.clone();
    result["descriptor"] = full;
    let manifests: BTreeMap<_, _> = state
        .raw_manifests
        .iter()
        .map(|(k, v)| (k.clone(), hash(v.as_bytes())))
        .collect();
    result["whole_manifest_key"] = json!(digest(
        &json!({"source_projection":descriptor,"workspace_manifest":hash(&workspace_data),"lockfile":hash(&lock_data),"package_manifests":manifests,"context":state.context})
    )?);
    result["eligible"] = json!(true);
    let missing: Vec<_> = CONTEXT_KEYS
        .iter()
        .filter(|key| state.context.get(**key).is_none())
        .cloned()
        .collect();
    if missing.is_empty() {
        result["action_key"] = json!(digest(&result["descriptor"])?);
    }
    result["missing_context"] = json!(missing);
    result["files"] = json!(packages
        .values()
        .map(|p| p["inventory"]["files"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0))
        .sum::<usize>());
    result["bytes"] = json!(packages
        .values()
        .flat_map(|p| p["inventory"]["files"].as_array().into_iter().flatten())
        .map(|f| f["bytes"].as_u64().unwrap_or(0))
        .sum::<u64>());
    result["package_count"] = json!(packages.len());
    Ok(())
}
/// Project live inputs. Even an eligible result remains discovery evidence, not authorization.
pub fn discover(repo: &Path, body: &str, context: Value, options: &DiscoverOptions) -> Value {
    let started = Instant::now();
    let context = if context.is_null() {
        json!({})
    } else {
        context
    };
    let workspace = options
        .workspace
        .clone()
        .unwrap_or_else(|| repo.join("sdk/lib/rust"));
    let mut result = json!({"schema":SCHEMA,"eligible":false,"reasons":[],"action_key":null,"source_projection_key":null,"whole_manifest_key":null,"descriptor":null});
    let state = Discovery::new(repo, &workspace, context);
    let mut state = match state {
        Ok(s) => s,
        Err(e) => {
            result["reasons"] = json!([e.message]);
            return result;
        }
    };
    if let Err(e) = project(&mut state, body, options, &mut result) {
        result["eligible"] = json!(false);
        result["action_key"] = Value::Null;
        result["reasons"] = json!([e.message]);
    }
    result["provenance"] = json!({"reads":state.reads,"stamps":state.stamps,"resolution":state.evidence,"elapsed_seconds":started.elapsed().as_secs_f64(),"identity_status":"projected live inputs; no immutable snapshot or executor authorization","normalization":"repository-relative package roots; private resolution paths excluded","limitations":["normal path dependencies only; no target/build/registry/git edges","build --lib only; no scripts/proc macros/lints/inherited workspace fields","pinned Cargo 1.92.0; resolver 2; no active weak optional forwarding; release/dev; no wildcard/version-qualified profile overrides","source text audit is conservative, not an execution read sandbox","caller must supply complete actual execution context; discovery does not establish toolchain identity","live discovery cannot establish snapshot coherence against concurrent writers"]});
    result
}
