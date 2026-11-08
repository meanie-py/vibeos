//! Native payload selection and staging helpers.
use crate::{payload_providers, payload_spec, Error, Result};
use serde_json::{json, Value};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};
fn err(s: impl ToString) -> Error {
    Error::new(s.to_string())
}
fn field<'a>(v: &'a Value, k: &str) -> Result<&'a str> {
    v.get(k)
        .and_then(Value::as_str)
        .ok_or_else(|| err(format!("missing string {k}")))
}
fn names(v: &Value) -> Result<Vec<String>> {
    v.as_array()
        .ok_or_else(|| err("names must be an array"))?
        .iter()
        .map(|x| {
            x.as_str()
                .map(str::to_owned)
                .ok_or_else(|| err("name must be a string"))
        })
        .collect()
}
fn resolved(root: &Path, path: &str) -> PathBuf {
    let p = root.join(path);
    if let Ok(p) = p.canonicalize() {
        return p;
    }
    let mut parts = PathBuf::new();
    for c in p.components() {
        match c {
            std::path::Component::ParentDir => {
                parts.pop();
            }
            std::path::Component::CurDir => {}
            _ => parts.push(c.as_os_str()),
        }
    }
    parts
}
fn entries(root: &Path, spec: &Value, arch: &str) -> Result<Vec<Value>> {
    let mut out = spec["binaries"]
        .as_array()
        .ok_or_else(|| err("missing binaries"))?
        .clone();
    if let Some(provider) = spec
        .get("binaries_from")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
    {
        out.extend(payload_providers::stage_entries(root, provider, arch)?);
    }
    for b in &out {
        if b.get("src").is_none() || b.get("dest").is_none() {
            return Err(err("provider staged entry without src/dest"));
        }
    }
    Ok(out)
}
fn src(root: &Path, b: &Value, arch: &str) -> Result<PathBuf> {
    Ok(resolved(
        root,
        &field(b, "src")?.replace(
            "{arch}",
            b.get("build_arch").and_then(Value::as_str).unwrap_or(arch),
        ),
    ))
}
pub fn resolve_env(env: &Value) -> Vec<String> {
    let mut out: Vec<_> = env
        .get("VIBEOS_PAYLOAD")
        .and_then(Value::as_str)
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect();
    if env.get("VIBEOS_DOOM").and_then(Value::as_str) == Some("1")
        && !out.iter().any(|n| n == "doom")
    {
        out.push("doom".into());
    }
    out
}
pub fn read_stamp(dir: &Path) -> Result<Vec<String>> {
    let p = dir.join("payload-selection.json");
    let data = match fs::read(&p) {
        Ok(x) => x,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(err(format!("{}: {e}", p.display()))),
    };
    let v: Value =
        serde_json::from_slice(&data).map_err(|e| err(format!("corrupt {}: {e}", p.display())))?;
    names(v.get("payloads").unwrap_or(&json!([]))).map_err(|_| {
        err(format!(
            "corrupt {}: `payloads` must be a list of strings",
            p.display()
        ))
    })
}
/// The payload's source files: everything under its directory that is not a build output
/// (gitignored, or a binary extension), plus the shared `tools/payloads/*.h`.
fn payload_sources(root: &Path, name: &str) -> Result<Vec<PathBuf>> {
    let dir = root.join("tools/payloads").join(name);
    let ignored = Command::new("git")
        .args([
            "ls-files",
            "--others",
            "--ignored",
            "--exclude-standard",
            "--",
            ".",
        ])
        .current_dir(&dir)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|s| resolved(&dir, s))
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();
    let mut files = Vec::new();
    walk_sources(&dir, &mut files)?;
    for e in fs::read_dir(root.join("tools/payloads")).map_err(err)? {
        let p = e.map_err(err)?.path();
        if p.extension().is_some_and(|x| x == "h") {
            files.push(p);
        }
    }
    files.retain(|p| !ignored.contains(&resolved(root, &p.to_string_lossy())));
    Ok(files)
}
/// The newest source and its mtime: what a "still stale" output is measured against, so the
/// error can name it (a concurrent build of another arch once tripped this, and the message
/// said only which output was stale -- not which file was newer).
fn newest_source(root: &Path, name: &str) -> Result<Option<(PathBuf, std::time::SystemTime)>> {
    Ok(payload_sources(root, name)?
        .into_iter()
        .filter_map(|p| Some((fs::metadata(&p).ok()?.modified().ok()?, p)))
        .max_by_key(|(t, _)| *t)
        .map(|(t, p)| (p, t)))
}
fn stale(root: &Path, name: &str, arch: &str) -> Result<Vec<PathBuf>> {
    let spec = payload_spec::load(root, name)?;
    let dir = root.join("tools/payloads").join(name);
    let files = payload_sources(root, name)?;
    let newest = files
        .iter()
        .filter_map(|p| fs::metadata(p).ok()?.modified().ok())
        .max();
    let sources: HashSet<_> = files
        .iter()
        .map(|p| resolved(root, &p.to_string_lossy()))
        .collect();
    let parent = resolved(root, &dir.to_string_lossy());
    let mut out = Vec::new();
    for b in entries(root, &spec, arch)? {
        let p = src(root, &b, arch)?;
        if p.starts_with(&parent)
            && !sources.contains(&p)
            && (!p.is_file()
                || newest.is_some_and(|n| {
                    fs::metadata(&p)
                        .and_then(|m| m.modified())
                        .map_or(true, |t| t < n)
                }))
        {
            out.push(p);
        }
    }
    Ok(out)
}
fn walk_sources(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for e in fs::read_dir(dir).map_err(err)? {
        let e = e.map_err(err)?;
        let p = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if name == "__pycache__" {
            continue;
        }
        if e.file_type().map_err(err)?.is_dir() {
            walk_sources(&p, out)?;
        } else if p.is_file()
            && !name.starts_with('.')
            && !["exe", "dll", "png", "log"].contains(
                &p.extension()
                    .and_then(|x| x.to_str())
                    .unwrap_or("")
                    .to_lowercase()
                    .as_str(),
            )
        {
            out.push(p);
        }
    }
    Ok(())
}
fn borrowed(root: &Path, name: &str, arch: &str) -> Result<Vec<(String, String)>> {
    let parent = resolved(root, "tools/payloads");
    let mut out = Vec::new();
    for b in entries(root, &payload_spec::load(root, name)?, arch)? {
        let p = src(root, &b, arch)?;
        if let Ok(rel) = p.strip_prefix(&parent) {
            if let Some(owner) = rel.components().next() {
                let owner = owner.as_os_str().to_string_lossy().into_owned();
                let pair = (
                    owner.clone(),
                    b.get("build_arch")
                        .and_then(Value::as_str)
                        .unwrap_or(arch)
                        .to_owned(),
                );
                if owner != name
                    && parent.join(&owner).join("payload.yaml").is_file()
                    && !out.contains(&pair)
                {
                    out.push(pair);
                }
            }
        }
    }
    Ok(out)
}
fn ensure(root: &Path, selected: &[String], arch: &str) -> Result<Vec<String>> {
    let mut builds = Vec::new();
    let mut wanted: HashMap<(String, String), HashSet<PathBuf>> = HashMap::new();
    let own: HashSet<_> = selected
        .iter()
        .map(|n| (n.clone(), arch.to_owned()))
        .collect();
    let parent = resolved(root, "tools/payloads");
    for name in selected {
        for pair in borrowed(root, name, arch)?
            .into_iter()
            .chain(std::iter::once((name.clone(), arch.to_owned())))
        {
            if !builds.contains(&pair) {
                builds.push(pair);
            }
        }
        for b in entries(root, &payload_spec::load(root, name)?, arch)? {
            let a = b
                .get("build_arch")
                .and_then(Value::as_str)
                .unwrap_or(arch)
                .to_owned();
            let p = src(root, &b, arch)?;
            if let Ok(rel) = p.strip_prefix(&parent) {
                if let Some(o) = rel.components().next() {
                    wanted
                        .entry((o.as_os_str().to_string_lossy().into_owned(), a))
                        .or_default()
                        .insert(p);
                }
            }
        }
    }
    let mut rebuilt = Vec::new();
    for (name, a) in builds {
        let pair = (name.clone(), a.clone());
        let mut outdated = stale(root, &name, &a)?;
        if !own.contains(&pair) {
            outdated.retain(|p| wanted.get(&pair).is_some_and(|w| w.contains(p)));
        }
        if outdated.is_empty()
            && payload_spec::load(root, &name)?
                .get("rebuild")
                .and_then(Value::as_str)
                != Some("always")
        {
            continue;
        }
        let script = parent.join(&name).join("build.sh");
        if !script.is_file() {
            return Err(err(format!(
                "payload {name:?}: stale or missing output and no {}",
                script.display()
            )));
        }
        let o = Command::new("bash")
            .arg(&script)
            .arg(&a)
            .current_dir(root)
            .output()
            .map_err(err)?;
        if !o.status.success() {
            let text = if o.stderr.is_empty() {
                &o.stdout
            } else {
                &o.stderr
            };
            let s = String::from_utf8_lossy(text);
            let lines: Vec<_> = s.lines().collect();
            return Err(err(format!(
                "payload {name:?}: build.sh {a} failed: {}",
                lines[lines.len().saturating_sub(5)..].join("\n")
            )));
        }
        let mut still = stale(root, &name, &a)?;
        if !own.contains(&pair) {
            still.retain(|p| wanted.get(&pair).is_some_and(|w| w.contains(p)));
        }
        if let Some(p) = still.first() {
            let age = |t: std::time::SystemTime| {
                t.duration_since(std::time::UNIX_EPOCH).map_or(0.0, |d| d.as_secs_f64())
            };
            let out_t = fs::metadata(p).and_then(|m| m.modified()).ok().map(age);
            let newer = newest_source(root, &name)?
                .map(|(s, t)| format!("{} (mtime {:.3})", s.display(), age(t)))
                .unwrap_or_else(|| "no source".into());
            return Err(err(format!(
                "payload {name:?}: build.sh {a} succeeded but {} is still stale (mtime {}): \
                 newer than it is {newer}",
                p.display(),
                out_t.map_or("missing".into(), |t| format!("{t:.3}"))
            )));
        }
        rebuilt.push(name);
    }
    Ok(rebuilt)
}
fn walk_depfiles(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for e in fs::read_dir(dir).map_err(err)? {
        let p = e.map_err(err)?.path();
        if p.is_dir() {
            walk_depfiles(&p, out)?;
        } else if p.extension().is_some_and(|x| x == "d") {
            out.push(p);
        }
    }
    Ok(())
}
/// The tool crates that only ever read a finished build: the smoke and payload harness, the
/// lint gate (its build-time scans report, they write nothing the media carries) and the bench
/// driver. An edit to one cannot change what a build-<arch> stages, so it never makes that
/// build stale; counting them did, and made every harness edit force a full amd64 rebuild
/// before an arm64 payload could stage (smoke_run.rs, 2026-11-08). Unlike the scopes below,
/// this needs no build record: it is a fact about the tool, not about one build.
const NEVER_BUILD_INPUTS: [&str; 3] = [
    "tools/vibeos-test/",
    "tools/vibeos-lint/",
    "tools/vibeos-bench/",
];

fn unreferenced(build: &Path, path: &str) -> Result<bool> {
    if NEVER_BUILD_INPUTS.iter().any(|dir| path.starts_with(dir)) {
        return Ok(true);
    }
    let parts: Vec<_> = path.split('/').collect();
    let needle = if parts.len() >= 5 && parts[..3] == ["sdk", "lib", "rust"] {
        format!("/{}/", parts[..4].join("/"))
    } else if parts.len() >= 3
        && (parts[..2] == ["tools", "fex"]
            || parts[..2] == ["tools", "payloads"]
            || parts[..2] == ["vendor", "fex"])
    {
        // Each named by its own path wherever a build reads it. A payload's sources build
        // through its own build.sh and vendored FEX is arm64's alone, so an edit to either
        // made every arm64 payload iteration rebuild amd64 for nothing.
        format!("/{path}")
    } else {
        return Ok(false);
    };
    let mut ninja = Vec::new();
    for e in fs::read_dir(build).map_err(err)? {
        let p = e.map_err(err)?.path();
        if p.extension().is_some_and(|x| x == "ninja") {
            ninja.push(p);
        }
    }
    let mut deps = Vec::new();
    walk_depfiles(build, &mut deps)?;
    if ninja.is_empty() || deps.is_empty() {
        return Ok(false);
    }
    for p in ninja {
        if String::from_utf8_lossy(&fs::read(p).map_err(err)?).contains(&needle) {
            return Ok(false);
        }
    }
    let output = Command::new("ninja")
        .args(["-C", &build.to_string_lossy(), "-t", "deps"])
        .output()
        .map_err(err)?;
    if !output.status.success() {
        return Err(err(format!(
            "cannot read {}'s ninja dependency log",
            build.display()
        )));
    }
    if String::from_utf8_lossy(&output.stdout).contains(&needle) {
        return Ok(false);
    }
    for p in deps {
        if String::from_utf8_lossy(&fs::read(p).map_err(err)?).contains(&needle) {
            return Ok(false);
        }
    }
    Ok(true)
}
fn git(root: &Path, args: &[&str], index: Option<&Path>) -> Result<String> {
    let mut c = Command::new("git");
    c.current_dir(root).args(args);
    if let Some(i) = index {
        c.env("GIT_INDEX_FILE", i);
    }
    let o = c.output().map_err(err)?;
    if !o.status.success() {
        return Err(err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&o.stderr)
        )));
    }
    Ok(String::from_utf8_lossy(&o.stdout).trim().to_owned())
}
fn reachable_python(root: &Path) -> Option<HashSet<String>> {
    let tracked = git(root, &["ls-files", "--", "tools/*.py", "tests/*.py"], None).ok()?;
    let o = Command::new("git")
        .current_dir(root)
        .args([
            "grep",
            "-h",
            "-o",
            "-I",
            "-E",
            r"[A-Za-z0-9_-]+\.py|-m[[:space:]]+[A-Za-z_][A-Za-z0-9_.]*",
            "--",
            "*CMakeLists.txt",
            "*.cmake",
            "*build.rs",
            "sdk/lib/rust/reactos-link",
            "sdk/tools",
            "*.sh",
            "*.cmd",
        ])
        .output()
        .ok()?;
    if ![Some(0), Some(1)].contains(&o.status.code()) {
        return None;
    }
    let mut text = String::from_utf8_lossy(&o.stdout).into_owned();
    for e in fs::read_dir(root).ok()? {
        let p = e.ok()?.path();
        if p.file_name()?.to_string_lossy().starts_with("build-") && p.join("build.ninja").is_file()
        {
            text.push_str(&fs::read_to_string(p.join("build.ninja")).ok()?);
        }
    }
    let py = regex::Regex::new(r"([A-Za-z0-9_-]+)\.py\b|-m\s+([A-Za-z_][\w.]*)").ok()?;
    let imports = regex::Regex::new(
        r"(?m)^\s*(?:from\s+\.*([\w.]*)\s+import\s+([\w, ()]+)|import\s+([\w., ]+))",
    )
    .ok()?;
    let mut by: HashMap<String, Vec<String>> = HashMap::new();
    for p in tracked.lines() {
        let parts: Vec<_> = p.trim_end_matches(".py").split('/').collect();
        let base = *parts.last()?;
        by.entry(base.into()).or_default().push(p.into());
        if base == "__init__" && parts.len() > 1 {
            by.entry(parts[parts.len() - 2].into())
                .or_default()
                .push(p.into());
        }
    }
    let split = |s: &str| -> Vec<String> {
        s.split(|c: char| c == '.' || c == ',' || c.is_whitespace() || c == '(' || c == ')')
            .filter(|s| !s.is_empty() && *s != "as")
            .map(str::to_owned)
            .collect()
    };
    let mut queue = Vec::new();
    for c in py.captures_iter(&text) {
        let wanted = if let Some(m) = c.get(1) {
            vec![m.as_str().to_owned()]
        } else {
            split(c.get(2)?.as_str())
        };
        for n in wanted {
            if let Some(p) = by.get(&n) {
                queue.extend(p.iter().cloned());
            }
        }
    }
    let mut found = HashSet::new();
    while let Some(p) = queue.pop() {
        if !found.insert(p.clone()) {
            continue;
        }
        if let Ok(src) = fs::read_to_string(root.join(&p)) {
            for c in imports.captures_iter(&src) {
                for i in 1..=3 {
                    if let Some(m) = c.get(i) {
                        for n in split(m.as_str()) {
                            if let Some(p) = by.get(&n) {
                                queue.extend(p.iter().cloned());
                            }
                        }
                    }
                }
            }
        }
    }
    Some(found)
}
fn guest_current(root: &Path, name: &str, build: &Path) -> Result<()> {
    let leaf = build.file_name().and_then(|x| x.to_str()).unwrap_or("");
    let arch = leaf
        .strip_prefix("build-")
        .filter(|a| ["i386", "amd64", "arm64"].contains(a))
        .ok_or_else(|| {
            err(format!(
                "payload {name:?}: guest set build_dir {leaf:?} is not build-<arch>"
            ))
        })?;
    let tmp = tempfile::tempdir().map_err(err)?;
    let index = tmp.path().join("index");
    let real = git(root, &["rev-parse", "--git-path", "index"], None)?;
    let real = root.join(real);
    if real.is_file() {
        // Keeps the real index's mtime: a fresh one made a same-size edit in the last
        // `git add`'s second look clean, and a stale guest was accepted.
        crate::copy_git_index(&real, &index).map_err(err)?;
    }
    git(root, &["add", "-A"], Some(&index))?;
    let tree = git(root, &["write-tree"], Some(&index))?;
    let common = git(root, &["rev-parse", "--git-common-dir"], None)?;
    let dir = root.join(common).join("vibeos-build-receipts");
    let mut receipts = Vec::new();
    if let Ok(iter) = fs::read_dir(dir) {
        for e in iter.flatten() {
            let n = e.file_name().to_string_lossy().into_owned();
            if n.ends_with(&format!(".{arch}.json")) {
                let t = n.split('.').next().unwrap_or("").to_owned();
                let m = e.metadata().and_then(|x| x.modified()).ok();
                receipts.push((m, t));
            }
        }
    }
    receipts.sort_by(|a, b| b.0.cmp(&a.0));
    if receipts.iter().any(|(_, t)| *t == tree) {
        return Ok(());
    }
    let mut reach = None;
    for (_, t) in receipts.into_iter().take(20) {
        let Ok(diff) = git(root, &["diff", "--name-only", &t, &tree], None) else {
            continue;
        };
        let mut ok = true;
        for p in diff.lines() {
            let lower = p.to_lowercase();
            if (lower != "cmakelists.txt"
                && !lower.ends_with("/cmakelists.txt")
                && [".md", ".txt", ".rst", ".png", ".jpg", ".jpeg", ".gif"]
                    .iter()
                    .any(|x| lower.ends_with(x)))
                || unreferenced(build, p)?
            {
                continue;
            }
            if p.ends_with(".py") && (p.starts_with("tools/") || p.starts_with("tests/")) {
                if reach.is_none() {
                    reach = Some(reachable_python(root));
                }
                if reach
                    .as_ref()
                    .and_then(|r| r.as_ref())
                    .is_some_and(|r| !r.contains(p))
                {
                    continue;
                }
            }
            ok = false;
            break;
        }
        if ok {
            return Ok(());
        }
    }
    Err(err(format!("payload {name:?}: {leaf} was not built from this tree, so its guest set would stage stale programs. Build it first: ./vibeos-cli build --arch {arch}")))
}
fn manifest(root: &Path, selected: &[String], arch: &str) -> Result<Value> {
    if !["i386", "amd64", "arm64"].contains(&arch) {
        return Err(err(format!("unknown arch {arch:?}")));
    }
    let mut out = serde_json::Map::new();
    let mut owners = HashMap::new();
    for name in selected {
        let spec = payload_spec::load(root, name)?;
        if let Some(a) = spec.get("arches").and_then(Value::as_array) {
            if !a.iter().any(|x| x.as_str() == Some(arch)) {
                return Err(err(format!(
                    "payload {name:?} does not support arch {arch:?}"
                )));
            }
        }
        let hint = spec
            .get("build_hint")
            .and_then(Value::as_str)
            .unwrap_or("")
            .replace("{arch}", arch);
        for b in entries(root, &spec, arch)? {
            let p = src(root, &b, arch)?;
            let prefix = if b.get("root").and_then(Value::as_str).unwrap_or("reactos") == "reactos"
            {
                "reactos/"
            } else {
                ""
            };
            let d = format!(
                "{}{}",
                prefix,
                field(&b, "dest")?
                    .replace('\\', "/")
                    .trim_start_matches('/')
            );
            if out.contains_key(&d) {
                return Err(err(format!(
                    "dest collision on {d:?}: payloads {:?} and {name:?} both graft it",
                    owners.get(&d)
                )));
            }
            if !p.is_file() {
                return Err(err(format!(
                    "payload {name:?}: src missing: {}\n  build it first: {hint}",
                    p.display()
                )));
            }
            if let Some(g) = stale_generator(root, &p)? {
                return Err(err(format!(
                    "payload {name:?}: {} is older than its generator {}; regenerate it\n  {hint}",
                    p.display(),
                    g.display()
                )));
            }
            owners.insert(d.clone(), name.clone());
            out.insert(d, json!(p));
        }
        if let Some(sets) = spec.get("guest_sets").and_then(Value::as_array) {
            for g in sets {
                let build = root.join(field(g, "build_dir")?);
                if !build.is_dir() {
                    return Err(err(format!(
                        "payload {name:?}: guest set needs {}",
                        build.display()
                    )));
                }
                guest_current(root, name, &build)?;
                let roots = names(
                    g.get("roots")
                        .ok_or_else(|| err("guest set missing roots"))?,
                )?;
                for (fname, p) in
                    crate::payload_closure::closure(root, &build, &roots, true, true, false, false)?
                {
                    let dest = format!(
                        "reactos/{}/{}",
                        field(g, "dest")?.replace('\\', "/").trim_matches('/'),
                        fname
                    );
                    if let Some(existing) = out.get(&dest) {
                        if existing.as_str() != p.to_str() {
                            return Err(err(format!("dest collision on {dest:?}")));
                        }
                    }
                    owners.insert(dest.clone(), name.clone());
                    out.insert(dest, json!(p));
                }
            }
        }
    }
    Ok(Value::Object(out))
}
fn stale_generator(root: &Path, src: &Path) -> Result<Option<PathBuf>> {
    if src
        .extension()
        .and_then(|s| s.to_str())
        .map(str::to_lowercase)
        .as_deref()
        != Some("txt")
    {
        return Ok(None);
    }
    let data = match fs::read(src) {
        Ok(data) => data,
        Err(_) => return Ok(None),
    };
    let text = String::from_utf8_lossy(&data);
    let head = text.lines().take(4).collect::<Vec<_>>().join("\n");
    if !head.contains("GENERATED by") {
        return Ok(None);
    }
    let re = regex::Regex::new(r"tools/\S+?\.py").map_err(err)?;
    if let Some(m) = re.find(&head) {
        let p = root.join(m.as_str());
        if p.is_file()
            && fs::metadata(&p).and_then(|m| m.modified()).map_err(err)?
                > fs::metadata(src).and_then(|m| m.modified()).map_err(err)?
        {
            return Ok(Some(p));
        }
    }
    Ok(None)
}
pub fn media_manifest(root: &Path, output: &Path, env: &Value) -> Result<Value> {
    let path = resolved(root, &output.to_string_lossy());
    let re = regex::Regex::new(r"^(.*build-(amd64|i386|arm64))")?;
    let path_text = path.to_string_lossy();
    let captures = re.captures(&path_text);
    let selected =
        if env.get("VIBEOS_PAYLOAD").is_some() || env["VIBEOS_DOOM"].as_str() == Some("1") {
            resolve_env(env)
        } else if let Some(cap) = &captures {
            read_stamp(Path::new(&cap[1]))?
        } else {
            Vec::new()
        };
    if selected.is_empty() {
        return Ok(json!({}));
    }
    let captures = captures.ok_or_else(|| {
        err(format!(
            "payloads selected but cannot infer media arch from {}",
            output.display()
        ))
    })?;
    manifest(root, &selected, &captures[2])
}
pub fn dispatch(root: &Path, req: &Value) -> Result<Value> {
    let op = field(req, "op")?
        .strip_prefix("payload_build_")
        .unwrap_or(field(req, "op")?);
    let kw = req.get("kwargs").unwrap_or(req);
    let arch = kw.get("arch").and_then(Value::as_str).unwrap_or("");
    match op {
        "media_manifest" => media_manifest(
            root,
            Path::new(field(kw, "output")?),
            kw.get("environ").unwrap_or(&Value::Null),
        ),
        "available" => Ok(json!(payload_spec::available(root)?)),
        "load_spec" => payload_spec::load(root, field(kw, "name")?),
        "resolve_env_payloads" => Ok(json!(resolve_env(
            kw.get("environ").unwrap_or(&Value::Null)
        ))),
        "read_stamp" => Ok(json!(read_stamp(Path::new(field(kw, "build_dir")?))?)),
        "write_stamp" => {
            let n = resolve_env(kw.get("environ").unwrap_or(&Value::Null));
            let p = Path::new(field(kw, "build_dir")?).join("payload-selection.json");
            fs::write(p, format!("{}\n", json!({"payloads": n}))).map_err(err)?;
            Ok(json!(n))
        }
        "resolve_payloads" => {
            let env = kw.get("environ").unwrap_or(&Value::Null);
            if env.get("VIBEOS_PAYLOAD").is_some()
                || env.get("VIBEOS_DOOM").and_then(Value::as_str) == Some("1")
            {
                Ok(json!(resolve_env(env)))
            } else if let Some(p) = kw.get("build_dir").and_then(Value::as_str) {
                Ok(json!(read_stamp(Path::new(p))?))
            } else {
                Ok(json!([]))
            }
        }
        "binaries" => Ok(json!(entries(
            root,
            kw.get("spec").ok_or_else(|| err("missing spec"))?,
            arch
        )?)),
        "stale_binaries" => Ok(json!(stale(root, field(kw, "name")?, arch)?)),
        "borrowed_builds" => Ok(json!(borrowed(root, field(kw, "name")?, arch)?)),
        "borrowed_owners" => {
            let mut owners = Vec::new();
            for (o, _) in borrowed(root, field(kw, "name")?, arch)? {
                if !owners.contains(&o) {
                    owners.push(o);
                }
            }
            Ok(json!(owners))
        }
        "stale_generator" => Ok(json!(stale_generator(root, Path::new(field(kw, "src")?))?)),
        "guest_set" => {
            let name = field(kw, "name")?;
            let g = kw.get("guest").ok_or_else(|| err("missing guest"))?;
            let build = root.join(field(g, "build_dir")?);
            if !build.is_dir() {
                return Err(err(format!(
                    "payload {name:?}: guest set needs {}",
                    build.display()
                )));
            }
            guest_current(root, name, &build)?;
            let roots = names(
                g.get("roots")
                    .ok_or_else(|| err("guest set missing roots"))?,
            )?;
            Ok(json!(crate::payload_closure::closure(
                root, &build, &roots, true, true, false, false
            )?))
        }
        "guest_build_current" => {
            guest_current(root, field(kw, "name")?, Path::new(field(kw, "build_dir")?))?;
            Ok(Value::Null)
        }
        "unreferenced_by_build" => Ok(json!(unreferenced(
            Path::new(field(kw, "build_dir")?),
            field(kw, "path")?
        )?)),
        "manifest" => manifest(
            root,
            &names(kw.get("names").ok_or_else(|| err("missing names"))?)?,
            arch,
        ),
        "ensure_built" => Ok(json!(ensure(
            root,
            &names(kw.get("names").ok_or_else(|| err("missing names"))?)?,
            arch
        )?)),
        "ninja_targets" => {
            let mut out = Vec::new();
            for n in names(kw.get("names").ok_or_else(|| err("missing names"))?)? {
                let s = payload_spec::load(root, &n)?;
                let mut targets: Vec<String> = s
                    .get("ninja_targets")
                    .and_then(Value::as_array)
                    .map(|a| {
                        a.iter()
                            .filter_map(Value::as_str)
                            .map(str::to_owned)
                            .collect()
                    })
                    .unwrap_or_default();
                if let Some(p) = s.get("binaries_from").and_then(Value::as_str) {
                    targets.extend(payload_providers::ninja_targets(root, p, arch)?);
                }
                for t in targets {
                    if !out.contains(&t) {
                        out.push(t);
                    }
                }
            }
            Ok(json!(out))
        }
        _ => Err(err(format!("unknown payload build operation {op}"))),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn env_empty_overrides_and_alias_deduplicates() {
        assert_eq!(
            resolve_env(&json!({"VIBEOS_PAYLOAD":" doom, x ","VIBEOS_DOOM":"1"})),
            vec!["doom", "x"]
        );
        assert!(resolve_env(&json!({"VIBEOS_PAYLOAD":""})).is_empty());
    }
    #[test]
    fn stamp_and_explicit_empty_selection() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("payload-selection.json"),
            r#"{"payloads":["doom"]}"#,
        )
        .unwrap();
        assert_eq!(read_stamp(tmp.path()).unwrap(), vec!["doom"]);
        let result=dispatch(tmp.path(),&json!({"op":"payload_build_resolve_payloads","kwargs":{"build_dir":tmp.path(),"environ":{"VIBEOS_PAYLOAD":""}}})).unwrap();
        assert_eq!(result, json!([]));
        fs::write(tmp.path().join("payload-selection.json"), "broken").unwrap();
        assert!(read_stamp(tmp.path()).is_err());
    }
    #[test]
    fn no_build_records_prove_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(!unreferenced(tmp.path(), "sdk/lib/rust/foo/src/lib.rs").unwrap());
    }
    #[test]
    fn harness_crates_are_never_build_inputs() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(unreferenced(tmp.path(), "tools/vibeos-test/src/smoke_run.rs").unwrap());
        assert!(unreferenced(tmp.path(), "tools/vibeos-lint/src/rules/mod.rs").unwrap());
        // Payload sources and vendored FEX are judged by the records, as tools/fex is:
        // with none, nothing is proven.
        assert!(!unreferenced(tmp.path(), "tools/payloads/appstorm/appstorm.c").unwrap());
        assert!(!unreferenced(
            tmp.path(),
            "vendor/fex/FEXCore/Source/Interface/Core/Core.cpp"
        )
        .unwrap());
        // The codegen and build crates do write what a build stages.
        assert!(!unreferenced(tmp.path(), "tools/vibeos-codegen/src/lib.rs").unwrap());
        assert!(!unreferenced(tmp.path(), "tools/vibeos-build/src/lib.rs").unwrap());
    }
    #[test]
    fn unsupported_arch_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(manifest(tmp.path(), &[], "mips").is_err());
    }
    fn fixture() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        assert!(Command::new("git")
            .args(["init", "-q"])
            .current_dir(root)
            .status()
            .unwrap()
            .success());
        for (p, content) in [
            (".gitignore", "build-amd64/\n"),
            ("CMakeLists.txt", "project(example)\n"),
            ("readme.md", "original\n"),
            ("code.c", "int x;\n"),
            ("sdk/lib/rust/guest/src/lib.rs", "pub fn x() {}\n"),
        ] {
            let path = root.join(p);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, content).unwrap();
        }
        git(root, &["add", "-A"], None).unwrap();
        let tree = git(root, &["write-tree"], None).unwrap();
        let receipts = root.join(".git/vibeos-build-receipts");
        fs::create_dir_all(&receipts).unwrap();
        fs::write(receipts.join(format!("{tree}.amd64.json")), "{}").unwrap();
        let build = root.join("build-amd64");
        fs::create_dir_all(build.join("obj")).unwrap();
        (tmp, build)
    }
    #[test]
    fn receipt_sees_a_same_size_edit_made_with_the_last_git_add() {
        let (tmp, build) = fixture();
        let root = tmp.path();
        fs::write(root.join("code.c"), "int y;\n").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        // The edit lands in the same second as the fixture's `git add`; the check runs in a
        // later one. A fresh-mtime index copy made git trust the stale stat data here.
        assert!(guest_current(root, "demo", &build).is_err(), "same-size edit missed");
    }
    #[test]
    fn receipt_rejects_code_and_cmake_but_accepts_docs() {
        let (tmp, build) = fixture();
        let root = tmp.path();
        assert!(guest_current(root, "demo", &build).is_ok());
        fs::write(root.join("readme.md"), "changed\n").unwrap();
        assert!(guest_current(root, "demo", &build).is_ok());
        fs::write(root.join("code.c"), "int y;\n").unwrap();
        assert!(guest_current(root, "demo", &build).is_err());
        fs::write(root.join("code.c"), "int x;\n").unwrap();
        fs::write(root.join("CMakeLists.txt"), "project(changed)\n").unwrap();
        assert!(guest_current(root, "demo", &build).is_err());
    }
    #[test]
    fn receipt_accepts_unbuilt_crate_only_with_complete_build_records() {
        let (tmp, build) = fixture();
        let root = tmp.path();
        fs::write(
            root.join("sdk/lib/rust/guest/src/lib.rs"),
            "pub fn changed() {}\n",
        )
        .unwrap();
        assert!(guest_current(root, "demo", &build).is_err());
        fs::write(build.join("build.ninja"), "rule x\n  command = true\n").unwrap();
        fs::write(build.join("obj/x.d"), "x: unrelated\n").unwrap();
        assert!(guest_current(root, "demo", &build).is_ok());
        fs::write(
            build.join("obj/x.d"),
            "x: /another/checkout/sdk/lib/rust/guest/src/lib.rs\n",
        )
        .unwrap();
        assert!(guest_current(root, "demo", &build).is_err());
    }
}
