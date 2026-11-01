use super::{valid_arch, within, Generated};
use regex::Regex;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    path::Path,
};
pub const DEFAULT_SUITES: &[&str] = &["oleaut32", "ole32"];
pub const DEFAULT_REGISTER: &[&str] = &["ole32", "oleaut32", "windowscodecs", "urlmon", "shell32"];
/// Parsed CLI options; vectors preserve input order and repetitions. `out_dir` supplies the header tag.
pub struct Options<'a> {
    pub root: &'a Path,
    pub arch: &'a str,
    pub out_dir: &'a Path,
    pub suites: &'a [String],
    pub register: &'a [String],
    pub only: &'a [String],
    pub skip: &'a [String],
    pub global_flag: &'a [String],
    pub repeat: &'a [String],
    pub timeout: &'a [String],
}
fn valid_suite(suite: &str) -> Result<(), String> {
    if suite.is_empty() || suite == "." || suite == ".." || suite.contains(['/', '\\']) {
        return Err(format!("invalid winetest suite name: {suite:?}"));
    }
    Ok(())
}
/// Source discovery reads only this repository root and its suite directory.
pub fn subtests(root: &Path, suite: &str) -> Result<Vec<String>, String> {
    valid_suite(suite)?;
    let dir = root.join("modules/rostests/winetests").join(suite);
    if dir.exists() {
        within(&dir, root)?;
    }
    let testlist = dir.join("testlist.c");
    if testlist.is_file() {
        within(&testlist, root)?;
        let text =
            fs::read_to_string(&testlist).map_err(|e| format!("{}: {e}", testlist.display()))?;
        let re = Regex::new(r#"(?m)^\s*\{\s*"([A-Za-z0-9_]+)"\s*,\s*func_\w+\s*\}"#).unwrap();
        return Ok(re.captures_iter(&text).map(|c| c[1].to_owned()).collect());
    }
    let mut tests = Vec::new();
    let re = Regex::new(r"(?m)^START_TEST\((\w+)\)").unwrap();
    let rd = match fs::read_dir(&dir) {
        Ok(v) => v,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(tests),
        Err(e) => return Err(format!("{}: {e}", dir.display())),
    };
    for entry in rd {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().is_some_and(|v| v == "c") {
            within(&path, root)?;
            let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            tests.extend(re.captures_iter(&text).map(|c| c[1].to_owned()));
        }
    }
    tests.sort();
    Ok(tests)
}
fn pair(v: &str) -> (String, String) {
    let (s, t) = v.split_once(':').unwrap_or((v, ""));
    let s = s.strip_suffix("_winetest.exe").unwrap_or(s);
    let s = s.strip_suffix("_winetest").unwrap_or(s);
    (s.to_owned(), t.to_owned())
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
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if c.is_control() => out.push_str(&format!("\\x{:02x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}
fn positive(s: &str) -> Option<usize> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse::<usize>().ok().filter(|n| *n > 0)
}
/// Render winetest list and skip file; no files are written.
pub fn generate(o: &Options<'_>) -> Result<Generated, String> {
    valid_arch(o.arch)?;
    for suite in o.suites {
        valid_suite(suite)?;
    }
    let build = o
        .root
        .join(format!("build-{}/modules/rostests/winetests", o.arch));
    let missing: Vec<_> = o
        .suites
        .iter()
        .filter(|s| !build.join(s).join(format!("{s}_winetest.exe")).is_file())
        .cloned()
        .collect();
    if !missing.is_empty() {
        return Err(format!("gen_suites: {} not built under {}.\n  fix: ./vibeos-cli build --arch {} --rostests --keep-going",missing.join(", "),build.display(),o.arch));
    }
    for s in o.suites {
        within(&build.join(s).join(format!("{s}_winetest.exe")), o.root)?;
    }
    let mut table = HashMap::new();
    let mut order = Vec::new();
    for s in o.suites {
        let tests = subtests(o.root, s)?;
        if !table.contains_key(s) {
            order.push(s.clone());
        }
        table.insert(s.clone(), tests);
    }
    let only: Vec<_> = o.only.iter().map(|s| pair(s)).collect();
    let skip: Vec<_> = o.skip.iter().map(|s| pair(s)).collect();
    for (s, t) in only.iter().chain(skip.iter()) {
        if !table
            .get(s)
            .is_some_and(|tests| t.is_empty() || tests.contains(t))
        {
            return Err(format!("gen_suites: no such suite/subtest {s}:{t}"));
        }
    }
    let skip: HashSet<_> = skip.into_iter().collect();
    let mut repeat = HashMap::new();
    for spec in o.repeat {
        let (p, c) = spec.split_once('=').unwrap_or((spec, ""));
        let (s, t) = pair(p);
        let n = positive(c);
        if !table.get(&s).is_some_and(|ts| ts.contains(&t)) || n.is_none() {
            return Err(format!(
                "gen_suites: bad --repeat {} (want SUITE:Subtest=N)",
                py_repr(spec)
            ));
        }
        repeat.insert((s, t), n.unwrap());
    }
    let mut timeouts = BTreeMap::new();
    if table.contains_key("dwrite") {
        timeouts.insert("dwrite".to_owned(), 1800);
    }
    for spec in o.timeout {
        let (s, n) = spec.split_once('=').unwrap_or((spec, ""));
        let val = positive(n);
        if !table.contains_key(s) || val.is_none() {
            return Err(format!(
                "gen_suites: bad --timeout {} (want SUITE=SECONDS)",
                py_repr(spec)
            ));
        }
        timeouts.insert(s.to_owned(), val.unwrap());
    }
    let tag = o.out_dir.file_name().and_then(|x| x.to_str()).unwrap_or("");
    let mut lines = vec![
        format!("# {tag} sweep list - {}. GENERATED by", o.arch),
        "# tools/payloads/olewinetest/gen_suites.py from each suite's testlist.c.".into(),
    ];
    if !only.is_empty() {
        lines.push(format!("# RESTRICTED by --only to: {}", o.only.join(", ")));
    }
    if !repeat.is_empty() {
        lines.push(format!("# REPEATED by --repeat: {}", o.repeat.join(", ")));
    }
    let ifeo =
        "HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Image File Execution Options";
    for gf in o.global_flag {
        let (s, flags) = gf.split_once('=').unwrap_or((gf, ""));
        let s = s.strip_suffix("_winetest.exe").unwrap_or(s);
        let s = s.strip_suffix("_winetest").unwrap_or(s).to_owned();
        if !o.suites.contains(&s) || flags.is_empty() {
            return Err(format!(
                "gen_suites: bad --global-flag {} (want SUITE=HEX)",
                py_repr(gf)
            ));
        }
        lines.push(format!(
            "reg.exe add \"{ifeo}\\{s}_winetest.exe\" /v GlobalFlag /t REG_DWORD /d {flags} /f"
        ));
        lines.push(format!(
            "# GlobalFlag {flags} on {s}: a DEBUG run, not a baseline"
        ));
    }
    for (s, n) in timeouts {
        lines.push(format!("timeout {s}_winetest.exe {n}"));
    }
    for dll in o.register {
        lines.push(format!("regsvr32.exe /s {dll}.dll"));
    }
    let mut count = 0;
    for s in order {
        for sub in &table[&s] {
            if !only.is_empty()
                && !only
                    .iter()
                    .any(|(name, t)| name == &s && (t.is_empty() || t == sub))
            {
                continue;
            }
            if skip.contains(&(s.clone(), sub.clone())) {
                continue;
            }
            let times = *repeat.get(&(s.clone(), sub.clone())).unwrap_or(&1);
            for _ in 0..times {
                lines.push(format!("{s}_winetest.exe {sub}"));
            }
            count += times;
        }
    }
    Ok(Generated {
        suites: lines.join("\n") + "\n",
        skips: "# none: skips are applied when the list is generated\n".into(),
        warnings: vec![],
        status: format!(
            "gen_suites: {count} subtest(s) -> winetest-suites-{}.txt",
            o.arch
        ),
    })
}
