use crate::binary::{BinaryContext, CheckOutput};
use regex::Regex;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

const DENIED: &[&str] = &[
    "invalid_value",
    "deref_nullptr",
    "static_mut_refs",
    "improper_ctypes_definitions",
    "improper_ctypes",
];
const CANON: &str = "/tmp/vibeos/";

#[derive(Clone, Debug)]
struct Site {
    path: String,
    line: u64,
    col: u64,
    lint: String,
    msg: String,
}
impl Site {
    fn key(&self) -> (String, u64, u64, String) {
        (self.path.clone(), self.line, self.col, template(&self.msg))
    }
}
#[derive(Clone, Debug)]
struct Unit {
    crate_name: String,
    fpdir: PathBuf,
    stale: String,
    sites: Vec<Site>,
}

fn regexes() -> &'static (Regex, Regex, Regex, Regex) {
    static R: OnceLock<(Regex, Regex, Regex, Regex)> = OnceLock::new();
    R.get_or_init(|| {
        (
            Regex::new(r"^(?P<crate>.+)-(?P<hash>[0-9a-f]{16})$").unwrap(),
            Regex::new(r#"(?m)^\[package\][^\[]*?^name\s*=\s*"([^"]+)""#).unwrap(),
            Regex::new(r"`[^`]*`").unwrap(),
            Regex::new(r"\b\d+\b").unwrap(),
        )
    })
}
fn template(msg: &str) -> String {
    let (_, _, backtick, number) = regexes();
    let no_ticks = backtick.replace_all(msg, "`_`");
    let no_nums = number.replace_all(&no_ticks, "N");
    let list = list_regex().replace_all(&no_nums, "`_`");
    let plural = plural_regex().replace_all(&list, "$1");
    plural.trim().to_string()
}
fn list_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"`_`(?:(?:, | and |, and | or |, or )`_`)+").unwrap())
}
fn plural_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r"\b(import|variant|field|function|method|constant|associated item|macro|label)s\b",
        )
        .unwrap()
    })
}
fn denied(lint: &str) -> bool {
    DENIED.contains(&lint)
}

fn canonical_site_path(raw: &str) -> String {
    let path = raw.strip_prefix(CANON).unwrap_or(raw);
    let path = path.strip_prefix("sdk/lib/rust/").unwrap_or(path);
    normalize_path(path)
}
fn normalize_path(raw: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    let owned = raw.replace('\\', "/");
    let absolute = owned.starts_with('/');
    for p in owned.split('/') {
        match p {
            "" | "." => {}
            ".." => {
                if !parts.is_empty() && parts.last() != Some(&"..") {
                    parts.pop();
                } else if !absolute {
                    parts.push("..");
                }
            }
            _ => parts.push(p),
        }
    }
    let joined = parts.join("/");
    if absolute {
        format!("/{joined}")
    } else {
        joined
    }
}
fn site_of(diag: &Value) -> Option<Site> {
    if diag["level"].as_str()? != "warning" {
        return None;
    }
    let spans = diag["spans"].as_array()?;
    let span = spans.iter().find(|s| s["is_primary"] == true)?;
    let code = diag["code"]["code"].as_str()?;
    if !denied(code) {
        return None;
    }
    let path = canonical_site_path(span["file_name"].as_str()?);
    let line = span["line_start"].as_u64()?;
    let col = span["column_start"].as_u64()?;
    let msg = diag["message"].as_str()?.to_string();
    Some(Site {
        path,
        line,
        col,
        lint: code.to_string(),
        msg,
    })
}

fn dep_paths(dfile: &Path) -> Result<Vec<String>, String> {
    let contents = fs::read_to_string(dfile).map_err(|e| format!("{}: {e}", dfile.display()))?;
    let first = contents.split('\n').next().unwrap_or("");
    let deps = first.split_once(": ").map(|(_, d)| d).unwrap_or("").trim();
    let mut out = Vec::new();
    let mut start = 0;
    for (index, ch) in deps.char_indices() {
        if ch == ' ' && (index == 0 || deps.as_bytes()[index - 1] != b'\\') {
            if index > start {
                out.push(deps[start..index].replace("\\ ", " "));
            }
            start = index + 1;
        }
    }
    if start < deps.len() {
        out.push(deps[start..].replace("\\ ", " "));
    }
    Ok(out)
}

fn package_dirs(repo: &Path) -> BTreeMap<String, PathBuf> {
    let mut found = BTreeMap::new();
    // Match the Python census authority exactly: only indexed Cargo.toml files, in git's
    // path order. An untracked manifest must not silently change dep resolution.
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["ls-files", "-z", "*Cargo.toml"])
        .output();
    let Ok(output) = output else { return found };
    if !output.status.success() {
        return found;
    }
    for raw in output.stdout.split(|b| *b == 0).filter(|p| !p.is_empty()) {
        let rel = PathBuf::from(String::from_utf8_lossy(raw).as_ref());
        let path = repo.join(&rel);
        let Ok(contents) = fs::read_to_string(&path) else {
            continue;
        };
        if let Some(m) = regexes().1.captures(&contents) {
            found
                .entry(m[1].to_string())
                .or_insert_with(|| path.parent().unwrap_or(repo).to_path_buf());
        }
    }
    found
}
fn resolve_dep(
    dep: &str,
    repo: &Path,
    crate_name: &str,
    packages: &BTreeMap<String, PathBuf>,
) -> Option<PathBuf> {
    if let Some(rest) = dep.strip_prefix(CANON) {
        return Some(repo.join(rest));
    }
    let p = Path::new(dep);
    if p.is_absolute() {
        return Some(p.to_path_buf());
    }
    let bases = [
        packages.get(crate_name).map(PathBuf::as_path),
        Some(&repo.join("sdk/lib/rust")),
        Some(repo),
    ];
    for base in bases.into_iter().flatten() {
        let candidate = base.join(dep);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}
fn dep_info(fpdir: &Path, crate_name: &str, hash: &str) -> PathBuf {
    let mut libs: Vec<String> = fs::read_dir(fpdir)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with("lib-") && !name.ends_with(".json") {
                Some(name[4..].to_string())
            } else {
                None
            }
        })
        .collect();
    libs.sort();
    let stem = if libs.len() == 1 {
        libs[0].clone()
    } else {
        crate_name.replace('-', "_")
    };
    fpdir
        .parent()
        .unwrap_or(fpdir)
        .parent()
        .unwrap_or(fpdir)
        .join("deps")
        .join(format!("{stem}-{hash}.d"))
}
fn unit_staleness(fpdir: &Path, repo: &Path, packages: &BTreeMap<String, PathBuf>) -> String {
    let Some(stamp_meta) = fs::metadata(fpdir.join("invoked.timestamp")).ok() else {
        return "no invoked.timestamp".into();
    };
    let fpname = fpdir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let Some(m) = regexes().0.captures(&fpname) else {
        return "no invoked.timestamp".into();
    };
    let crate_name = m["crate"].to_string();
    let hash = m["hash"].to_string();
    let dfile = dep_info(fpdir, &crate_name, &hash);
    if !dfile.is_file() {
        return format!(
            "no dep-info {}",
            dfile.file_name().unwrap_or_default().to_string_lossy()
        );
    }
    let Ok(built) = stamp_meta.modified() else {
        return "no invoked.timestamp".into();
    };
    if let Some(manifest_dir) = packages.get(&crate_name) {
        let toml = manifest_dir.join("Cargo.toml");
        if fs::metadata(&toml)
            .and_then(|m| m.modified())
            .is_ok_and(|t| t > built)
        {
            let display = toml
                .strip_prefix(repo)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| toml.to_string_lossy().into_owned());
            return format!("{display} changed since the build");
        }
    }
    let Ok(deps) = dep_paths(&dfile) else {
        return format!(
            "no dep-info {}",
            dfile.file_name().unwrap_or_default().to_string_lossy()
        );
    };
    for dep in deps {
        let Some(src) = resolve_dep(&dep, repo, &crate_name, packages) else {
            return format!("source gone: {dep}");
        };
        let Ok(meta) = fs::metadata(&src) else {
            return format!("source gone: {dep}");
        };
        if meta.modified().is_ok_and(|t| t > built) {
            return format!("{dep} changed since the build");
        }
    }
    String::new()
}
fn parse_unit_sites(fpdir: &Path) -> Vec<Site> {
    let mut sites = Vec::new();
    let Ok(entries) = fs::read_dir(fpdir) else {
        return sites;
    };
    let mut outputs: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("output-lib-"))
        })
        .collect();
    outputs.sort();
    for output in outputs {
        let Ok(text) = fs::read_to_string(output) else {
            continue;
        };
        for line in text.lines() {
            let Ok(value) = serde_json::from_str::<Value>(line) else {
                continue;
            };
            if let Some(site) = site_of(&value) {
                sites.push(site)
            }
        }
    }
    sites
}
fn read_units(
    build_dir: &Path,
    repo: &Path,
    packages: &BTreeMap<String, PathBuf>,
) -> Result<Vec<Unit>, String> {
    if !build_dir.is_dir() {
        return Err(format!("no such build dir: {}", build_dir.display()));
    }
    fn read_fp(
        fp_root: &Path,
        repo: &Path,
        packages: &BTreeMap<String, PathBuf>,
        units: &mut Vec<Unit>,
    ) -> Result<(), String> {
        let mut fps: Vec<PathBuf> = fs::read_dir(fp_root)
            .map_err(|e| format!("{}: {e}", fp_root.display()))?
            .map(|e| e.map(|e| e.path()).map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        fps.sort();
        for fp in fps {
            let Some(name) = fp.file_name().map(|n| n.to_string_lossy().into_owned()) else {
                continue;
            };
            let Some(m) = regexes().0.captures(&name) else {
                continue;
            };
            if !fp.is_dir()
                || !fs::read_dir(&fp)
                    .map(|it| {
                        it.flatten().any(|e| {
                            e.file_name().to_string_lossy().starts_with("lib-")
                                && e.file_name().to_string_lossy().ends_with(".json")
                        })
                    })
                    .unwrap_or(false)
            {
                continue;
            }
            let crate_name = m["crate"].to_string();
            let stale = unit_staleness(&fp, repo, packages);
            let sites = parse_unit_sites(&fp);
            units.push(Unit {
                crate_name,
                fpdir: fp,
                stale,
                sites,
            });
        }
        Ok(())
    }
    fn walk(
        dir: &Path,
        repo: &Path,
        packages: &BTreeMap<String, PathBuf>,
        units: &mut Vec<Unit>,
    ) -> Result<(), String> {
        let entries: Vec<_> = fs::read_dir(dir)
            .map_err(|e| format!("{}: {e}", dir.display()))?
            .map(|e| e.map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        let mut children = Vec::new();
        let mut fingerprints = None;
        for entry in entries {
            let path = entry.path();
            if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
                if entry.file_name() == ".fingerprint" {
                    fingerprints = Some(path);
                } else {
                    children.push(path);
                }
            }
        }
        // os.walk visits the current root's .fingerprint entries before descending.
        if let Some(fp) = fingerprints {
            read_fp(&fp, repo, packages, units)?;
        }
        for child in children {
            walk(&child, repo, packages, units)?;
        }
        Ok(())
    }
    let mut units = Vec::new();
    walk(build_dir, repo, packages, &mut units)?;
    Ok(units)
}
fn coverage(units: &[Unit]) -> (BTreeSet<String>, BTreeMap<String, String>) {
    let fresh: BTreeSet<String> = units
        .iter()
        .filter(|u| u.stale.is_empty())
        .map(|u| u.crate_name.clone())
        .collect();
    let mut stale = BTreeMap::new();
    let mut ordered: Vec<&Unit> = units.iter().filter(|u| !u.stale.is_empty()).collect();
    ordered.sort_by_key(|u| {
        fs::metadata(u.fpdir.join("invoked.timestamp"))
            .and_then(|m| m.modified())
            .ok()
    });
    for u in ordered {
        if !fresh.contains(&u.crate_name) {
            stale.insert(u.crate_name.clone(), u.stale.clone());
        }
    }
    (fresh, stale)
}
fn merge_sites(units: &[Unit]) -> Vec<Site> {
    let mut merged: Vec<Site> = Vec::new();
    let mut indices: HashMap<(String, u64, u64, String), usize> = HashMap::new();
    for unit in units.iter().filter(|u| u.stale.is_empty()) {
        for site in &unit.sites {
            let key = site.key();
            if let Some(index) = indices.get(&key).copied() {
                if merged[index].lint.is_empty() && !site.lint.is_empty() {
                    merged[index].lint = site.lint.clone();
                }
            } else {
                indices.insert(key, merged.len());
                merged.push(site.clone());
            }
        }
    }
    // Python's stable sort orders only path/line/column; template ties retain first-seen order.
    merged.sort_by_key(|s| (s.path.clone(), s.line, s.col));
    merged
}
pub fn run(ctx: &BinaryContext) -> Result<CheckOutput, String> {
    let packages = package_dirs(&ctx.root);
    let units = read_units(&ctx.build_dir, &ctx.root, &packages)?;
    if units.is_empty() {
        return Ok(CheckOutput {
            stdout: String::new(),
            stderr: format!(
                "NO CARGO UNITS under {}: not a build dir?\n",
                ctx.build_dir.display()
            ),
            returncode: 2,
        });
    }
    let (fresh, stale) = coverage(&units);
    let arch = ctx.arch.clone();
    let mut stderr = String::new();
    if !stale.is_empty() {
        stderr.push_str(&format!(
            "STALE: {} crate(s) edited since their build:\n",
            stale.len()
        ));
        for (crate_name, why) in &stale {
            stderr.push_str(&format!("   {arch}:{crate_name}: {why}\n"));
        }
    }
    let sites = merge_sites(&units);
    let hits: Vec<&Site> = sites.iter().filter(|s| denied(&s.lint)).collect();
    let mut stdout = String::new();
    for s in &hits {
        stdout.push_str(&format!(
            "{}:{}:{}: {}: {} [{}]\n",
            s.path, s.line, s.col, s.lint, s.msg, arch
        ));
    }
    let mut deny = DENIED.iter().copied().collect::<Vec<_>>();
    deny.sort();
    stderr.push_str(&format!(
        "{} site(s) of a denied lint ({}); {} fresh unit(s) read\n",
        hits.len(),
        deny.join(", "),
        fresh.len()
    ));
    Ok(CheckOutput {
        stdout,
        stderr,
        returncode: if hits.is_empty() { 0 } else { 1 },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    fn temp() -> PathBuf {
        std::env::temp_dir().join(format!(
            "binary-warnings-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }
    fn write(path: &Path, text: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    fn git_add(root: &Path, files: &[&str]) {
        let init = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["init", "-q"])
            .status()
            .unwrap();
        assert!(init.success());
        let add = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["add", "--"])
            .args(files)
            .status()
            .unwrap();
        assert!(add.success());
    }
    fn diag(path: &str, line: u64, lint: &str, msg: &str) -> String {
        serde_json::json!({"$message_type":"diagnostic","level":"warning","message":msg,"code":{"code":lint,"explanation":null},"spans":[{"file_name":path,"line_start":line,"column_start":5,"is_primary":true,"text":[{"text":"    x = 0;"}]}],"children":[]}).to_string()
    }
    fn unit(root: &Path, crate_name: &str, hash: &str, dep: &str, output: Option<&str>, time: u64) {
        let target = root.join("build-i386/t/i686/release");
        let fp = target
            .join(".fingerprint")
            .join(format!("{crate_name}-{hash}"));
        write(
            &fp.join(format!("lib-{}.json", crate_name.replace('-', "_"))),
            "{}",
        );
        if let Some(o) = output {
            write(
                &fp.join(format!("output-lib-{}", crate_name.replace('-', "_"))),
                &(o.to_string() + "\n"),
            );
        }
        write(
            &target
                .join("deps")
                .join(format!("{}-{hash}.d", crate_name.replace('-', "_"))),
            &format!("/tmp/vibeos/build-i386/x/deps/{crate_name}-{hash}.d: {dep}\n\n"),
        );
        write(&fp.join("invoked.timestamp"), "");
        let _ = time;
    }
    #[test]
    fn reads_json_primary_denials_and_sorts_deduplicated_sites() {
        let root = temp();
        let src = root.join("sdk/lib/rust/good-rust/src/lib.rs");
        write(&src, "");
        let d = diag(
            "/tmp/vibeos/sdk/lib/rust/good-rust/src/lib.rs",
            7,
            "invalid_value",
            "creating an uninitialized value of type `Thing` is undefined behavior",
        );
        unit(
            &root,
            "good-rust",
            "0000000000000000",
            "good-rust/src/lib.rs",
            Some(&d),
            2000,
        );
        unit(
            &root,
            "good-rust",
            "0000000000000001",
            "good-rust/src/lib.rs",
            Some(&d),
            2000,
        );
        let ctx = BinaryContext::new(root.clone(), root.join("build-i386"), "i386".into());
        let out = run(&ctx).unwrap();
        assert_eq!(out.returncode, 1);
        assert!(out
            .stdout
            .starts_with("good-rust/src/lib.rs:7:5: invalid_value:"));
        assert_eq!(out.stdout.lines().count(), 1);
        assert!(out.stderr.contains("1 fresh unit(s) read"));
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn ignores_non_denied_and_non_primary_diagnostics() {
        let root = temp();
        let src = root.join("sdk/lib/rust/clean-rust/src/lib.rs");
        write(&src, "");
        let mut d: Value =
            serde_json::from_str(&diag("clean-rust/src/lib.rs", 3, "dead_code", "unused")).unwrap();
        d["code"]["code"] = Value::String("dead_code".into());
        unit(
            &root,
            "clean-rust",
            "1111111111111111",
            "clean-rust/src/lib.rs",
            Some(&d.to_string()),
            2000,
        );
        let ctx = BinaryContext::new(root.clone(), root.join("build-i386"), "i386".into());
        let out = run(&ctx).unwrap();
        assert_eq!(out.returncode, 0);
        assert!(out.stdout.is_empty());
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn freshness_checks_manifest_and_make_escaped_paths_and_stale_latest_reason() {
        let root = temp();
        let crate_dir = root.join("sdk/lib/rust/crate-rust");
        write(
            &crate_dir.join("Cargo.toml"),
            "[package]\nname = \"crate-rust\"\n",
        );
        let src = crate_dir.join("src/file with space.rs");
        write(&src, "");
        let tool_crate = root.join("tools/spec-stubs-core");
        write(
            &tool_crate.join("Cargo.toml"),
            "[package]\nname = \"spec-stubs-core\"\n",
        );
        write(&tool_crate.join("src/lib.rs"), "");
        let untracked = root.join("untracked/Cargo.toml");
        write(&untracked, "[package]\nname = \"untracked-crate\"\n");
        git_add(
            &root,
            &[
                "sdk/lib/rust/crate-rust/Cargo.toml",
                "tools/spec-stubs-core/Cargo.toml",
            ],
        );
        unit(
            &root,
            "crate-rust",
            "2222222222222222",
            "crate-rust/src/file\\ with\\ space.rs",
            None,
            2000,
        );
        let target = root.join("build-i386/t/i686/release");
        let fp = target.join(".fingerprint/crate-rust-2222222222222222");
        let packages = package_dirs(&root);
        assert_eq!(packages.get("spec-stubs-core").unwrap(), &tool_crate);
        assert!(!packages.contains_key("untracked-crate"));
        assert_eq!(
            resolve_dep("src/lib.rs", &root, "spec-stubs-core", &packages).unwrap(),
            tool_crate.join("src/lib.rs")
        );
        assert!(unit_staleness(&fp, &root, &packages).is_empty());
        assert!(unit_staleness(&fp, &root, &packages).is_empty());
        std::thread::sleep(std::time::Duration::from_millis(5));
        write(
            &crate_dir.join("Cargo.toml"),
            "[package]\nname = \"crate-rust\"\n# changed\n",
        );
        assert!(
            unit_staleness(&fp, &root, &packages).contains("Cargo.toml changed since the build")
        );
        let renamed = target.join(".fingerprint/atlclasses-rust-3333333333333333");
        write(&renamed.join("lib-atl_rust.json"), "{}");
        write(&renamed.join("lib-atl_rust"), "");
        assert_eq!(
            dep_info(&renamed, "atlclasses-rust", "3333333333333333"),
            target.join("deps/atl_rust-3333333333333333.d")
        );
        let stale_old = target.join(".fingerprint/old-crate-4444444444444444");
        write(&stale_old.join("invoked.timestamp"), "");
        std::thread::sleep(std::time::Duration::from_millis(5));
        let missing_dep = root.join("build-i386/t/i686/release");
        let latest_fp = missing_dep.join(".fingerprint/old-crate-5555555555555555");
        write(&latest_fp.join("lib-old_crate.json"), "{}");
        write(&latest_fp.join("lib-old_crate"), "");
        write(&latest_fp.join("invoked.timestamp"), "");
        write(
            &missing_dep.join("deps/old_crate-5555555555555555.d"),
            "/tmp/vibeos/build-i386/out.d: gone.rs\n",
        );
        let units = read_units(&root.join("build-i386"), &root, &packages).unwrap();
        let (_, stale) = coverage(&units);
        assert_eq!(stale.get("old-crate").unwrap(), "source gone: gone.rs");
        let _ = fs::remove_dir_all(root);
    }
}
