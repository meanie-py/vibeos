//! The cargo-drive manifests a build with no cmake reads must match the build graph:
//! `sdk/lib/rust/cells/<arch>.jsonl` (each cell's crate and outputs, from the
//! `Cargo-linking` ninja edges; tools/cell_manifest.py) and `boot/media/<arch>.json` (each
//! CD's path list; tools/media_manifest.py). A module converted, removed or moved on the CD
//! without regenerating them leaves cargo_orchestrate --manifest and media_manifest
//! --assemble building and mastering the old set. Reads build-<arch>/, so it is deferred,
//! and an arch with no configured build is skipped.
use crate::{Corpus, Rule};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

pub struct ScanCargoManifests;

const ARCHES: [&str; 3] = ["i386", "amd64", "arm64"];
const CDS: [&str; 3] = ["livecd", "bootcd", "bootcdregtest"];
const CANON: &str = "/tmp/vibeos";

static CELL_EDGE: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(
        r"\nbuild ([^\n]*?): CUSTOM_COMMAND[^\n]*\n  COMMAND = ([^\n]*)\n  DESC = Cargo-linking",
    )
    .expect("valid cargo-link regex")
});
static CRATE_ARG: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r" -p (\S+)").expect("valid -p regex"));

type Cells = BTreeMap<String, BTreeSet<String>>;

/// crate -> outputs (build-relative), from the ninja graph.
fn graph_cells(ninja: &str) -> Cells {
    let mut out = Cells::new();
    for c in CELL_EDGE.captures_iter(ninja) {
        let Some(krate) = CRATE_ARG.captures(&c[2]).map(|m| m[1].to_string()) else {
            continue;
        };
        let outs = c[1]
            .split('|')
            .next()
            .unwrap_or("")
            .split_whitespace()
            .map(str::to_owned);
        out.entry(krate).or_default().extend(outs);
    }
    out
}

fn manifest_cells(text: &str, path: &str) -> Result<Cells, String> {
    let mut out = Cells::new();
    for (n, line) in text
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty())
    {
        let v: serde_json::Value =
            serde_json::from_str(line).map_err(|e| format!("{path}:{}: {e}", n + 1))?;
        let krate = v["crate"]
            .as_str()
            .ok_or(format!("{path}:{}: no crate", n + 1))?;
        let outs = v["outputs"]
            .as_array()
            .ok_or(format!("{path}:{}: no outputs", n + 1))?;
        let set = out.entry(krate.to_string()).or_default();
        for o in outs {
            set.insert(o["out"].as_str().unwrap_or_default().to_string());
        }
    }
    Ok(out)
}

fn decode(src: &str, arch: &str) -> String {
    match src.split_once(':') {
        Some(("cell" | "build", rel)) => format!("{CANON}/build-{arch}/{rel}"),
        Some(("repo", rel)) => format!("{CANON}/{rel}"),
        _ => format!("<bad source {src}>"),
    }
}

fn media_lines(entries: &[serde_json::Value], arch: &str) -> Vec<String> {
    entries
        .iter()
        .map(|e| {
            let src = decode(e[1].as_str().unwrap_or_default(), arch);
            match e[0].as_str() {
                Some(dest) => format!("{dest}={src}"),
                None => src,
            }
        })
        .collect()
}

fn scan(repo: &Path) -> Result<Vec<String>, String> {
    let mut rows = Vec::new();
    for arch in ARCHES {
        let build = repo.join(format!("build-{arch}"));
        let Ok(ninja) = fs::read_to_string(build.join("build.ninja")) else {
            continue;
        };
        let cells_rel = format!("sdk/lib/rust/cells/{arch}.jsonl");
        let fix = "(python3 tools/cell_manifest.py --write)";
        match fs::read_to_string(repo.join(&cells_rel)) {
            Err(_) => rows.push(format!("{cells_rel}: missing {fix}")),
            Ok(text) => {
                let want = graph_cells(&ninja);
                let have = manifest_cells(&text, &cells_rel)?;
                for k in want.keys().filter(|k| !have.contains_key(*k)) {
                    rows.push(format!(
                        "{cells_rel}: {k} links in the build but is not listed {fix}"
                    ));
                }
                for k in have.keys().filter(|k| !want.contains_key(*k)) {
                    rows.push(format!(
                        "{cells_rel}: {k} is listed but the build links no such cell {fix}"
                    ));
                }
                for (k, outs) in &want {
                    if have.get(k).is_some_and(|h| h != outs) {
                        rows.push(format!(
                            "{cells_rel}: {k}'s outputs differ from the build's {fix}"
                        ));
                    }
                }
            }
        }
        let media_rel = format!("boot/media/{arch}.json");
        let fix = "(python3 tools/media_manifest.py --write)";
        let Ok(text) = fs::read_to_string(repo.join(&media_rel)) else {
            rows.push(format!("{media_rel}: missing {fix}"));
            continue;
        };
        let media: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| format!("{media_rel}: {e}"))?;
        let config = media["config"].as_str().unwrap_or("Debug");
        for cd in CDS {
            let Ok(lst) = fs::read_to_string(build.join(format!("boot/{cd}.{config}.lst"))) else {
                continue;
            };
            let want: Vec<&str> = lst.lines().collect();
            let entries = media["cds"][cd]["entries"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            let have = media_lines(&entries, arch);
            if want.len() != have.len() || want.iter().zip(&have).any(|(w, h)| w != h) {
                let differ = want.iter().zip(&have).filter(|(w, h)| *w != h).count()
                    + want.len().abs_diff(have.len());
                rows.push(format!(
                    "{media_rel}: {cd} differs from build-{arch}/boot/{cd}.{config}.lst at {differ} entries {fix}"
                ));
            }
        }
    }
    Ok(rows)
}

impl Rule for ScanCargoManifests {
    fn needs_workspace_rust(&self) -> bool {
        false
    }
    fn name(&self) -> &'static str {
        "scan_cargo_manifests"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        scan(&corpus.root)
    }
    fn source_sets(&self) -> &'static [crate::SourceSet] {
        &[]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let id = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let p =
                std::env::temp_dir().join(format!("vibeos-manifests-{}-{id}", std::process::id()));
            fs::create_dir_all(&p).unwrap();
            Self(p)
        }
        fn put(&self, rel: &str, text: &str) {
            let p = self.0.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, text).unwrap();
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    const NINJA: &str = "\nbuild m/m.exe | x: CUSTOM_COMMAND a\n  COMMAND = cd /tmp/vibeos/sdk/lib/rust && cargo build -p m-cargo --bin m\n  DESC = Cargo-linking m.exe (x) [reactos-link]\n";

    fn fixture(cells: &str, media_entries: &str, lst: &str) -> Temp {
        let t = Temp::new();
        t.put("build-amd64/build.ninja", NINJA);
        t.put("sdk/lib/rust/cells/amd64.jsonl", cells);
        t.put(
            "boot/media/amd64.json",
            &format!(r#"{{"config": "Debug", "cds": {{"livecd": {{"command": "", "entries": [{media_entries}]}}}}}}"#),
        );
        t.put("build-amd64/boot/livecd.Debug.lst", lst);
        t
    }

    const CELL: &str = r#"{"crate": "m-cargo", "outputs": [{"out": "m/m.exe"}]}"#;

    #[test]
    fn current_manifests_are_clean() {
        let t = fixture(
            CELL,
            r#"[null, "build:boot/empty"], ["reactos/m.exe", "cell:m/m.exe"]"#,
            "/tmp/vibeos/build-amd64/boot/empty\nreactos/m.exe=/tmp/vibeos/build-amd64/m/m.exe\n",
        );
        assert_eq!(scan(&t.0).unwrap(), Vec::<String>::new());
    }

    #[test]
    fn a_cell_the_build_links_but_the_manifest_lacks_is_reported() {
        let t = fixture(
            "",
            r#"[null, "build:boot/empty"]"#,
            "/tmp/vibeos/build-amd64/boot/empty\n",
        );
        assert_eq!(
            scan(&t.0).unwrap(),
            vec!["sdk/lib/rust/cells/amd64.jsonl: m-cargo links in the build but is not listed (python3 tools/cell_manifest.py --write)"]
        );
    }

    #[test]
    fn a_moved_cd_placement_is_reported() {
        let t = fixture(
            CELL,
            r#"["reactos/m.exe", "cell:m/m.exe"]"#,
            "reactos/system32/m.exe=/tmp/vibeos/build-amd64/m/m.exe\n",
        );
        let rows = scan(&t.0).unwrap();
        assert_eq!(rows.len(), 1);
        assert!(
            rows[0].starts_with("boot/media/amd64.json: livecd differs"),
            "{rows:?}"
        );
    }

    #[test]
    fn an_arch_with_no_configured_build_is_skipped() {
        let t = Temp::new();
        assert_eq!(scan(&t.0).unwrap(), Vec::<String>::new());
    }
}
