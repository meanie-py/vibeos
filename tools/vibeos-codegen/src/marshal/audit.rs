use super::*;
use crate::wow64::compute_thunk_table;
use std::{
    fs,
    path::{Path, PathBuf},
};
fn read(root: &Path, rel: &str) -> Result<String, String> {
    fs::read_to_string(root.join(rel)).map_err(|e| {
        format!(
            "marshalgen audit: cannot read {}: {e}",
            root.join(rel).display()
        )
    })
}
pub fn parse_table_artifact(text: &str) -> HashMap<String, bool> {
    // The committed Rust table row, or the C one the generator can also write.
    let rs = re(r#"^\s*WOW64_THUNK_ENTRY \{ Name: c"([^"]+)"\.as_ptr\(\), Thunk: (None|Some\(\w+\)),"#);
    let pat = re(r#"^\s*\{\s*"([^"]+)"\s*,\s*(\w+),\s*\d+\s*\},"#);
    let mut out = HashMap::new();
    for line in text.lines() {
        if let Some(m) = rs.captures(line) {
            out.insert(m[1].to_owned(), &m[2] != "None");
        } else if let Some(m) = pat.captures(line) {
            out.insert(m[1].to_owned(), &m[2] != "0");
        }
    }
    out
}
pub fn crosscheck(
    computed: &HashMap<String, bool>,
    gen_text: &str,
    path: &Path,
    tname: &str,
) -> Result<(), String> {
    let built = parse_table_artifact(gen_text);
    if built.is_empty() {
        return Ok(());
    }
    let mut diffs: Vec<_> = computed
        .keys()
        .filter_map(|n| {
            built
                .get(n)
                .filter(|b| **b != computed[n])
                .map(|b| (n.clone(), computed[n], *b))
        })
        .collect();
    diffs.sort_by(|a, b| a.0.cmp(&b.0));
    if diffs.is_empty() {
        return Ok(());
    }
    let lines = diffs
        .iter()
        .take(12)
        .map(|(n, c, b)| {
            format!(
                "    {n}: sources say {}, table says {}",
                if *c { "FLAT" } else { "marshalled" },
                if *b { "FLAT" } else { "marshalled" }
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let more = if diffs.len() > 12 {
        format!("\n    ... and {} more", diffs.len() - 12)
    } else {
        String::new()
    };
    Err(format!("audit {tname}: the committed thunk table disagrees with its sources on {} syscall(s) — {} is STALE.\n{lines}{more}\nRegenerate it before trusting this lint (`tools/codegen wow64-thunks --emit-rs`). Reporting stale findings here is how already-fixed sites get re-blessed into the baseline.",diffs.len(),path.display()))
}

fn build_dir(root: &Path, requested: Option<&str>, candidates: &[&str]) -> Option<PathBuf> {
    let list: Vec<_> = requested
        .map(|x| vec![x])
        .unwrap_or_else(|| candidates.to_vec());
    list.into_iter()
        .map(|x| {
            let p = Path::new(x);
            if p.is_absolute() {
                p.to_owned()
            } else {
                root.join(p)
            }
        })
        .find(|p| p.join("compile_commands.json").is_file())
}

pub fn audit(
    root: &Path,
    build64: Option<&str>,
    build32: Option<&str>,
    table_sel: &str,
) -> Result<Vec<String>, String> {
    let b64 = build_dir(root, build64, &["build-amd64", "build-arm64"]);
    let b32 = build_dir(root, build32, &["build-i386"]);
    let (Some(b64), Some(b32)) = (b64, b32) else {
        return Ok(vec![]);
    };
    crate::probe::local(root, &b64)?;
    crate::probe::local(root, &b32)?;
    let clang = root.join(".build-tools/llvm-mingw/bin/clang");
    if !clang.is_file() {
        return Ok(vec![]);
    }
    let tables: Vec<&str> = if table_sel == "both" {
        vec!["win32k", "ntdll"]
    } else {
        vec![table_sel]
    };
    let arch = if b64
        .file_name()
        .is_some_and(|n| n.to_string_lossy().ends_with("arm64"))
    {
        "arm64"
    } else {
        "amd64"
    };
    let mut findings = vec![];
    for tname in tables {
        let cfg = metadata()["AUDIT_TABLES"]
            .get(tname)
            .ok_or_else(|| format!("marshalgen audit: unknown table {tname}"))?;
        let gen_rel = cfg["gen"].as_str().ok_or("missing generated table path")?;
        let headers = cfg["headers"]
            .as_array()
            .ok_or("missing headers")?
            .iter()
            .map(|p| p.as_str().ok_or("invalid header"))
            .collect::<Result<Vec<_>, _>>()?;
        let tus = cfg["tus"]
            .as_array()
            .ok_or("missing probes")?
            .iter()
            .map(|p| {
                Ok((
                    p[0].as_str().ok_or("invalid TU suffix")?,
                    p[1].as_str().ok_or("invalid TU source")?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let table = compute_thunk_table(root, tname, arch)?;
        // The committed table, so it is checked whether or not this tree is built.
        let genpath = root.join(gen_rel);
        if genpath.is_file() {
            crate::probe::local(root, &genpath)?;
            let text = fs::read_to_string(&genpath)
                .map_err(|e| format!("cannot read {}: {e}", genpath.display()))?;
            crosscheck(&table, &text, &genpath, tname)?;
        }
        let (mut d32, mut d64) = (HashMap::new(), HashMap::new());
        for (suffix, tu) in tus {
            for (db, bdir) in [(&mut d32, &b32), (&mut d64, &b64)] {
                let flags=extract_flags(root,bdir,suffix).map_err(|e|if e==format!("NO_TU:{suffix}"){format!("audit {tname}: no TU matching '{suffix}' in {}/compile_commands.json — the layout probe cannot run, so this table would go UNCHECKED. Point cfg['tus'] at a TU that still exists.",bdir.display())}else{e})?;
                for (k, v) in dump_layouts(root, &flags, tu)? {
                    db.entry(k).or_insert(v);
                }
            }
        }
        if d32.is_empty() || d64.is_empty() {
            return Err(format!("audit {tname}: layout dump came back empty (32:{} 64:{}) — refusing to report a clean table from no data.",d32.len(),d64.len()));
        }
        let mut header_text = String::new();
        for h in headers {
            header_text.push_str(&read(root, h)?);
            header_text.push('\n')
        }
        let protos = extract_prototypes(&header_text, r"Nt\w+")?;
        let mut names: Vec<_> = table.keys().collect();
        names.sort();
        for name in names {
            if !table[name] {
                continue;
            }
            let Some(p) = protos.get(name) else { continue };
            for prm in &p.params {
                let (cls, det) = classify_param(prm, &d32, &d64);
                let v = audit_verdict(&cls, &det);
                let line = format!("{} {}({}) {} {}", tname, name, prm.name, cls, det);
                if v == "DIVERGENT" {
                    findings.push(format!("FLAT-DIVERGENT: {line}"))
                }
            }
        }
    }
    Ok(findings)
}
