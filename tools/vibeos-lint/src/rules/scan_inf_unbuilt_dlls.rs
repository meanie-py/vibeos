//! An INF that registers a DLL the install media does not carry.
//!
//! GUI setup runs each `RegisterDlls` section of `media/inf/syssetup.inf`, and a device install
//! runs its INF's. For every entry setup calls LoadLibrary on the file, and when the file is not
//! there it puts up a modal "LoadLibrary failed" box. An unattended install then waits on that box
//! until the stage times out. This happened for real: an upstream merge kept syssetup.inf's
//! `11,,uiautomationcore.dll,1` while the tree declined the restructure that builds that DLL, and
//! every unattended install hung on the Finishing page (install-smoke amd64,
//! 20261106-042127-fzhjtst3). Nothing failed loudly, because the INF is plain data that nothing
//! cross-checks against the build.
//!
//! The reference is the cab file list in each committed `boot/media/<arch>.json` (what an install
//! copies, kept honest by `media_manifest.py --check`), with the directory ids from
//! `reactos.dff.in`. Every entry whose directory resolves is checked on every arch; an arch that
//! does not ship the file is named in the finding. Entries in a directory this audit cannot
//! resolve (a `%string%`, a shell-folder id such as 16422) are counted in the summary, not
//! guessed at.
use crate::{Corpus, Rule, SourceSet};
use std::collections::{BTreeMap, BTreeSet};

pub struct ScanInfUnbuiltDlls;

const DFF: &str = "boot/bootdata/packages/reactos.dff.in";
const ARCHES: &[&str] = &["i386", "amd64", "arm64"];
const SETS: &[SourceSet] = &[SourceSet::tree(
    "inf-register-dlls",
    &["media/inf", "boot/media", "boot/bootdata/packages"],
    &["inf", "json", "in"],
    &[],
)
.with_inputs()];

/// The cab's directory table: `2 = system32` etc., between `.InfBegin` and the first section.
fn cab_dirs(dff: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for line in dff.lines() {
        let line = line.split(';').next().unwrap_or("").trim();
        let Some((id, path)) = line.split_once('=') else {
            continue;
        };
        let id = id.trim();
        if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let path = path.trim().trim_matches('"').replace('\\', "/");
        out.insert(id.to_string(), path.trim_matches('/').to_lowercase());
    }
    out
}

fn join(dir: &str, name: &str) -> String {
    if dir.is_empty() {
        name.to_lowercase()
    } else {
        format!("{dir}/{}", name.to_lowercase())
    }
}

/// Install-relative paths (lowercase) of every file in one arch's cab.
fn cab_files(manifest: &str, dirs: &BTreeMap<String, String>) -> Result<BTreeSet<String>, String> {
    let v: serde_json::Value = serde_json::from_str(manifest).map_err(|e| e.to_string())?;
    let files = v
        .pointer("/cab/files")
        .and_then(|f| f.as_array())
        .ok_or("no cab.files array")?;
    let mut out = BTreeSet::new();
    for f in files {
        let (Some(src), Some(dir)) = (f.get(0).and_then(|s| s.as_str()), f.get(1).and_then(|s| s.as_str())) else {
            continue;
        };
        let Some(d) = dirs.get(dir) else { continue };
        let name = src.rsplit(['/', ':']).next().unwrap_or(src);
        out.insert(join(d, name));
    }
    Ok(out)
}

/// The INF standard directory ids, as install-relative paths. 10 is the Windows directory.
fn inf_dir(id: &str, sub: &str) -> Option<String> {
    let base = match id.trim() {
        "10" => "",
        "11" => "system32",
        "12" => "system32/drivers",
        "17" => "inf",
        "20" => "fonts",
        _ => return None,
    };
    let sub = sub.trim().trim_matches('"').replace('\\', "/").to_lowercase();
    if sub.contains('%') {
        return None;
    }
    let sub = sub.trim_matches('/');
    Some(match (base.is_empty(), sub.is_empty()) {
        (_, true) => base.to_string(),
        (true, false) => sub.to_string(),
        (false, false) => format!("{base}/{sub}"),
    })
}

struct Entry {
    line: usize,
    raw: String,
    path: Option<String>,
}

/// Every entry of every section some `RegisterDlls =` directive in this INF names.
fn register_entries(inf: &str) -> Vec<Entry> {
    let mut sections: BTreeMap<String, Vec<(usize, String)>> = BTreeMap::new();
    let mut wanted = BTreeSet::new();
    let mut current = String::new();
    for (i, raw) in inf.lines().enumerate() {
        let line = raw.split(';').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.split(']').next()) {
            current = name.trim().to_lowercase();
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            if k.trim().eq_ignore_ascii_case("RegisterDlls") {
                for s in v.split(',') {
                    let s = s.trim().to_lowercase();
                    if !s.is_empty() {
                        wanted.insert(s);
                    }
                }
                continue;
            }
        }
        sections.entry(current.clone()).or_default().push((i + 1, line.to_string()));
    }
    let mut out = Vec::new();
    for s in wanted {
        for (line, text) in sections.get(&s).into_iter().flatten() {
            let f: Vec<&str> = text.split(',').collect();
            if f.len() < 3 {
                continue;
            }
            let name = f[2].trim().trim_matches('"');
            let path = if name.contains('%') { None } else { inf_dir(f[0], f[1]).map(|d| join(&d, name)) };
            out.push(Entry { line: *line, raw: text.clone(), path });
        }
    }
    out
}

fn audit(dff: &str, manifests: &[(&str, &str)], infs: &[(&str, &str)]) -> Vec<String> {
    let dirs = cab_dirs(dff);
    let mut out = Vec::new();
    if !dirs.values().any(|d| d == "system32") {
        return vec![format!("{DFF}:1: [NO_REFERENCE] no `N = system32` directory in the cab table — the parser or the dff layout changed, and this audit would pass vacuously")];
    }
    let mut cabs = Vec::new();
    for (arch, text) in manifests {
        match cab_files(text, &dirs) {
            Ok(set) if set.iter().any(|p| p.starts_with("system32/")) => cabs.push((*arch, set)),
            Ok(_) => out.push(format!("boot/media/{arch}.json:1: [NO_REFERENCE] its cab lists nothing in system32 — this audit would pass vacuously")),
            Err(e) => out.push(format!("boot/media/{arch}.json:1: [NO_REFERENCE] unreadable cab file list: {e}")),
        }
    }
    let (mut checked, mut unresolved, mut findings) = (0, 0, 0);
    for (path, text) in infs {
        for e in register_entries(text) {
            let Some(target) = e.path else {
                unresolved += 1;
                continue;
            };
            checked += 1;
            let missing: Vec<&str> = cabs
                .iter()
                .filter(|(_, set)| !set.contains(&target))
                .map(|(a, _)| *a)
                .collect();
            if !missing.is_empty() {
                findings += 1;
                out.push(format!(
                    "{path}:{}: [UNBUILT_REGISTER_DLL] `{}` registers {target}, which the {} install media does not carry; setup's LoadLibrary fails with a modal box and an unattended install hangs on it. Build the module, or comment the line out with the reason",
                    e.line,
                    e.raw,
                    missing.join("/")
                ));
            }
        }
    }
    out.push(String::new());
    out.push(format!(
        "SUMMARY: arches={}  checked={checked}  unresolved_dirs={unresolved}  findings={findings}",
        cabs.len()
    ));
    out
}

impl Rule for ScanInfUnbuiltDlls {
    fn name(&self) -> &'static str {
        "scan_inf_unbuilt_dlls"
    }
    fn needs_workspace_rust(&self) -> bool {
        false
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        SETS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let files: Vec<_> = c.sources("inf-register-dlls").collect();
        let rel = |f: &&crate::SourceFile| f.path.to_string_lossy().replace('\\', "/");
        let Some(dff) = files.iter().find(|f| rel(f).ends_with(DFF)) else {
            return Ok(vec![format!("{DFF}:1: [MISSING] cannot resolve cab directories without this file")]);
        };
        let mut manifests = Vec::new();
        for arch in ARCHES {
            let want = format!("boot/media/{arch}.json");
            match files.iter().find(|f| rel(f).ends_with(&want)) {
                Some(f) => manifests.push((*arch, f.text.as_str())),
                None => return Ok(vec![format!("{want}:1: [MISSING] the committed media manifest is the reference; without it this arch is unchecked")]),
            }
        }
        let paths: Vec<String> = files.iter().map(rel).collect();
        let infs: Vec<(&str, &str)> = files
            .iter()
            .zip(&paths)
            .filter(|(_, p)| p.contains("media/inf/") && p.to_lowercase().ends_with(".inf"))
            .map(|(f, p)| {
                let p = p.as_str();
                (&p[p.find("media/inf/").unwrap()..], f.text.as_str())
            })
            .collect();
        Ok(audit(&dff.text, &manifests, &infs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const DFF_T: &str = ".InfBegin\n[Directories]\n1 = \"\\\"\n2 = system32\n17 = system32\\wbem\n.InfEnd\n";
    fn manifest(files: &[(&str, &str)]) -> String {
        let f: Vec<String> = files.iter().map(|(s, d)| format!("[\"{s}\", \"{d}\"]")).collect();
        format!("{{\"cab\": {{\"files\": [{}]}}}}", f.join(","))
    }
    const INF: &str = "[EarlyPhaseSetup]\nRegisterDlls = Early\n\n[Early]\n11,,ole32.dll,1\n11,,uiautomationcore.dll,1 ; upstream\n11,wbem,wbemprox.dll,1\n16422,Internet Explorer,iexplore.exe\n\n[NotRegistered]\n11,,ghost.dll,1\n";
    #[test]
    fn unbuilt_dll_is_named_per_arch() {
        let full = manifest(&[("cell:dll/win32/ole32/ole32.dll", "2"), ("cell:dll/win32/wbemprox/wbemprox.dll", "17")]);
        let partial = manifest(&[("cell:dll/win32/ole32/ole32.dll", "2")]);
        let x = audit(DFF_T, &[("amd64", &full), ("arm64", &partial)], &[("media/inf/syssetup.inf", INF)]);
        let text = x.join("\n");
        assert!(text.contains("media/inf/syssetup.inf:6: [UNBUILT_REGISTER_DLL]"), "{text}");
        assert!(text.contains("system32/uiautomationcore.dll, which the amd64/arm64"));
        assert!(text.contains("system32/wbem/wbemprox.dll, which the arm64 install"));
        assert!(!text.contains("ghost.dll"), "a section no RegisterDlls names is not run");
        assert!(text.contains("checked=3  unresolved_dirs=1  findings=2"));
    }
    #[test]
    fn missing_reference_fails_loudly() {
        assert!(audit("nothing", &[], &[])[0].contains("[NO_REFERENCE]"));
        let empty = manifest(&[]);
        assert!(audit(DFF_T, &[("i386", &empty)], &[])[0].contains("[NO_REFERENCE]"));
    }
}
