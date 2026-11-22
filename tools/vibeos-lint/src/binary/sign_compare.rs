//! On-demand audit: mixed-sign `min`/`max` in the inert (cohab-flipped) C.
//!
//! C's usual arithmetic conversions make `min(int, UINT)` an UNSIGNED compare;
//! a translation that casts one side so Rust's `min` type-checks can flip it to
//! a signed one (rebar's 8 px taskbar, user32 listbox's `LB_SETCOUNT` clamp).
//! No source rule can see this without the C operand types, so this parses
//! every flipped `.c` that calls min/max with clang `-Wsign-compare`, borrowing
//! the flags of the nearest live TU in `build-<arch>/compile_commands.json`
//! (a flipped module usually has none of its own), and lists the warnings that
//! land on a min/max. It is a REVIEW list, not a gate: most hits are loop
//! bounds or operands guarded non-negative, so it always exits 0. Run all
//! three arches, since SIZE_T/LONG widths move which compares are mixed-sign.
//!
//! ```text
//! ./vibeos-cli lint --arch amd64 --rules scan_sign_compare_minmax
//! ```
//!
//! A TU the nearest donor cannot parse is retried under a driver and a
//! user-mode donor, keeping the parse with the fewest errors; `#ERR n file`
//! marks one that still only partly parsed: reduced coverage.
use super::{BinaryContext, CheckOutput};
use rayon::prelude::*;
use regex::Regex;
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};

const CANON: &str = "/tmp/vibeos"; // builds run at the canonical bind-mount path
const AREAS: &[&str] = &[
    "ntoskrnl",
    "hal",
    "win32ss",
    "dll",
    "base",
    "drivers",
    "boot",
    "sdk/lib",
    "subsystems",
];

fn minmax() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\b(min|max|MIN|MAX|min_c|max_c)\s*\(").unwrap())
}
fn macro_note() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"expanded from macro '(min|max|MIN|MAX|min_c|max_c)'").unwrap())
}

/// POSIX-shell word splitting for compile_commands `command` strings.
pub(crate) fn shell_split(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut have = false;
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '\'' => {
                have = true;
                for d in it.by_ref() {
                    if d == '\'' {
                        break;
                    }
                    cur.push(d);
                }
            }
            '"' => {
                have = true;
                while let Some(d) = it.next() {
                    match d {
                        '"' => break,
                        '\\' if matches!(it.peek(), Some('"' | '\\' | '$' | '`')) => {
                            cur.push(it.next().unwrap())
                        }
                        _ => cur.push(d),
                    }
                }
            }
            '\\' => {
                have = true;
                if let Some(d) = it.next() {
                    cur.push(d);
                }
            }
            c if c.is_whitespace() => {
                if have {
                    out.push(std::mem::take(&mut cur));
                    have = false;
                }
            }
            c => {
                have = true;
                cur.push(c);
            }
        }
    }
    if have {
        out.push(cur);
    }
    out
}

struct Entry {
    dir: String,
    file: String,
    args: Vec<String>,
}

fn real(root: &Path, p: &str) -> String {
    // anywhere in the arg, not just as a prefix: `-I/tmp/vibeos/sdk/include`
    p.replacen(CANON, &root.display().to_string(), 1)
}

fn load(ctx: &BinaryContext) -> Result<(HashSet<PathBuf>, BTreeMap<PathBuf, Entry>), String> {
    let db = ctx.build_dir.join("compile_commands.json");
    let text = fs::read_to_string(&db).map_err(|e| {
        format!(
            "{}: {e}\n  fix: ./vibeos-cli build --arch {}",
            db.display(),
            ctx.arch
        )
    })?;
    let v: Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", db.display()))?;
    let mut live = HashSet::new();
    let mut by_dir = BTreeMap::new();
    for e in v
        .as_array()
        .ok_or("compile_commands.json is not an array")?
    {
        let file = real(&ctx.root, e["file"].as_str().unwrap_or_default());
        let p = PathBuf::from(&file);
        if file.ends_with(".c") {
            if let Some(d) = p.parent() {
                if !by_dir.contains_key(d) {
                    let args = match e.get("arguments").and_then(Value::as_array) {
                        Some(a) => a
                            .iter()
                            .filter_map(|x| x.as_str().map(String::from))
                            .collect(),
                        None => shell_split(e["command"].as_str().unwrap_or_default()),
                    };
                    let dir = real(&ctx.root, e["directory"].as_str().unwrap_or_default());
                    by_dir.insert(
                        d.to_path_buf(),
                        Entry {
                            dir,
                            file: file.clone(),
                            args,
                        },
                    );
                }
            }
        }
        live.insert(p);
    }
    Ok((live, by_dir))
}

/// The nearest live TU: same dir, then an ancestor, then the same area.
fn donor<'a>(root: &Path, d: &Path, by_dir: &'a BTreeMap<PathBuf, Entry>) -> Option<&'a Entry> {
    let mut x = d;
    while x.starts_with(root) && x != root {
        if let Some(e) = by_dir.get(x) {
            return Some(e);
        }
        x = x.parent()?;
    }
    let rel: Vec<_> = d.strip_prefix(root).ok()?.components().collect();
    for k in [2, 1] {
        let pre: PathBuf = root.join(rel.iter().take(k).collect::<PathBuf>());
        if let Some((_, e)) = by_dir.iter().find(|(dd, _)| dd.starts_with(&pre)) {
            return Some(e);
        }
    }
    None
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut ents: Vec<_> = rd.flatten().map(|e| e.path()).collect();
    ents.sort();
    for p in ents {
        if p.is_dir() && !p.is_symlink() {
            walk(&p, out);
        } else if p.extension().is_some_and(|e| e == "c") {
            out.push(p);
        }
    }
}

fn command(ctx: &BinaryContext, p: &Path, e: &Entry) -> Vec<String> {
    let mut out = Vec::new();
    let mut skip = false;
    for a in e.args.iter().map(|a| real(&ctx.root, a)) {
        if skip {
            skip = false;
            continue;
        }
        if matches!(a.as_str(), "-o" | "-MF" | "-MT" | "-MQ") {
            skip = true;
            continue;
        }
        if matches!(a.as_str(), "-c" | "-MD" | "-MMD")
            || a == e.file
            || (a.starts_with("-W") && a.contains("error"))
        {
            continue;
        }
        out.push(a);
    }
    let mut x = p.parent().unwrap_or(p).to_path_buf();
    for _ in 0..4 {
        let rel = x.strip_prefix(&ctx.root).unwrap_or(&x).to_path_buf();
        for i in [
            x.clone(),
            x.join("include"),
            x.join("include/internal"),
            ctx.build_dir.join(rel),
        ] {
            out.push(format!("-I{}", i.display()));
        }
        match x.parent() {
            Some(parent) if parent != ctx.root => x = parent.to_path_buf(),
            _ => break,
        }
    }
    let rel = p.strip_prefix(&ctx.root).unwrap_or(p);
    if rel.starts_with("ntoskrnl") || rel.starts_with("hal") {
        // no live TU of their own: ntoskrnl/ntos.cmake's include set
        for d in [
            "ntoskrnl",
            "ntoskrnl/include",
            "sdk/lib/cmlib",
            "sdk/lib/drivers/arbiter",
            "sdk/lib/pseh/include",
            "sdk/include/reactos/libs",
            "sdk/include/reactos/drivers",
            "sdk/lib/rtl",
        ] {
            out.push(format!("-I{}", ctx.root.join(d).display()));
        }
        for d in ["ntoskrnl", "ntoskrnl/include", "ntoskrnl/include/internal"] {
            out.push(format!("-I{}", ctx.build_dir.join(d).display()));
        }
    }
    if rel.starts_with("boot/freeldr/freeldr") {
        // all Rust now, so no live freeldr TU to borrow: boot/freeldr/freeldr's CMake
        // BEFORE the donor's dirs, as its CMake's `include_directories(BEFORE include)`:
        // freeldr's own debug.h (DBG_DEFAULT_CHANNEL) must shadow sdk/include/reactos's.
        // The parent dir is for freeldr.h's `<include/ntldr/winldr.h>`.
        for (i, d) in ["boot/freeldr/freeldr/include", "boot/freeldr/freeldr"]
            .iter()
            .enumerate()
        {
            out.insert(1 + i, format!("-I{}", ctx.root.join(d).display()));
        }
        for d in ["-D_NTHAL_", "-D_BLDR_", "-D_NTSYSTEM_"] {
            out.push(d.into());
        }
    }
    if rel.starts_with("ntoskrnl") {
        for d in [
            "-D_NTOSKRNL_",
            "-D__NTOSKRNL__",
            "-D_NTSYSTEM_",
            "-DNTDDI_VERSION=0x05020400",
            "-include",
            "ntoskrnl.h",
        ] {
            out.push(d.into());
        }
    }
    for f in [
        "-fsyntax-only",
        "-Wno-everything",
        "-Wsign-compare",
        "-ferror-limit=0",
    ] {
        out.push(f.into());
    }
    out.push(p.display().to_string());
    out
}

/// The diagnostics in clang's stderr that are `-Wsign-compare` on a min/max.
pub(crate) fn hits(path: &str, rel: &str, src: &str, stderr: &str) -> (Vec<String>, usize, String) {
    let lines: Vec<&str> = stderr.lines().collect();
    let src: Vec<&str> = src.lines().collect();
    let prefix = format!("{path}:");
    let mut found = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if !line.contains("-Wsign-compare") || !line.starts_with(&prefix) {
            continue;
        }
        let Some(ln) = line[prefix.len()..]
            .split(':')
            .next()
            .and_then(|n| n.parse::<usize>().ok())
        else {
            continue;
        };
        let text = src.get(ln.wrapping_sub(1)).map_or("", |s| s.trim());
        // only this diagnostic's own notes, up to the next warning/error
        let notes = lines[i + 1..]
            .iter()
            .take_while(|l| !l.contains(": warning: ") && !l.contains(": error: "))
            .copied()
            .collect::<Vec<_>>()
            .join("\n");
        if minmax().is_match(text) || macro_note().is_match(&notes) {
            let why = line
                .split("warning: ")
                .last()
                .unwrap_or("")
                .replace(" [-Wsign-compare]", "");
            found.push(format!("{rel}:{ln}: {text}  || {why}"));
        }
    }
    let errors: Vec<&&str> = lines.iter().filter(|l| l.contains(" error: ")).collect();
    let first = errors.first().map_or(String::new(), |l| {
        l.split(" error: ").last().unwrap_or("").to_string()
    });
    (found, errors.len(), first)
}

fn missing_header() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"fatal error: '([^']+)' file not found").unwrap())
}

/// Every header in the tree (and this build's generated ones), by file name.
fn header_index(ctx: &BinaryContext) -> HashMap<String, Vec<PathBuf>> {
    fn go(dir: &Path, idx: &mut HashMap<String, Vec<PathBuf>>) {
        let Ok(rd) = fs::read_dir(dir) else { return };
        for p in rd.flatten().map(|e| e.path()) {
            let name = p.file_name().map(|n| n.to_string_lossy().into_owned());
            let Some(name) = name else { continue };
            if p.is_dir() && !p.is_symlink() {
                if !(name.starts_with('.') || name == "target" || name == "CMakeFiles") {
                    go(&p, idx);
                }
            } else if name.ends_with(".h") {
                idx.entry(name).or_default().push(p);
            }
        }
    }
    let mut idx = HashMap::new();
    for d in AREAS.iter().copied().chain(["sdk/include", "modules"]) {
        go(&ctx.root.join(d), &mut idx);
    }
    go(&ctx.build_dir, &mut idx);
    idx
}

/// The include dir that supplies `missing` (e.g. `afd/shared.h`), choosing the
/// candidate sharing the longest path prefix with the TU being parsed.
fn resolve(idx: &HashMap<String, Vec<PathBuf>>, missing: &str, tu: &Path) -> Option<PathBuf> {
    let want = Path::new(missing);
    let name = want.file_name()?.to_string_lossy();
    let depth = want.components().count();
    idx.get(name.as_ref())?
        .iter()
        .filter(|c| c.ends_with(want))
        .max_by_key(|c| {
            c.components()
                .zip(tu.components())
                .take_while(|(a, b)| a == b)
                .count()
        })
        .and_then(|c| c.ancestors().nth(depth).map(Path::to_path_buf))
}

pub fn run(ctx: &BinaryContext) -> Result<CheckOutput, String> {
    let (live, by_dir) = load(ctx)?;
    let headers = header_index(ctx);
    let mut files = Vec::new();
    for area in AREAS {
        walk(&ctx.root.join(area), &mut files);
    }
    // Fallbacks for a TU the nearest donor cannot parse (a user-mode file
    // beside kernel code, or ntoskrnl/hal, which have no live C left at all).
    let area_donor = |pre: &str| {
        let pre = ctx.root.join(pre);
        by_dir
            .iter()
            .find(|(d, _)| d.starts_with(&pre))
            .map(|(_, e)| e)
    };
    let fallbacks: Vec<&Entry> = [area_donor("drivers"), area_donor("dll/win32")]
        .into_iter()
        .flatten()
        .collect();
    let mut scan = Vec::new();
    for p in files {
        if live.contains(&p) {
            continue;
        }
        let Ok(src) = fs::read_to_string(&p)
            .or_else(|_| fs::read(&p).map(|b| String::from_utf8_lossy(&b).into()))
        else {
            continue;
        };
        if !minmax().is_match(&src) {
            continue;
        }
        let mut donors: Vec<&Entry> = p
            .parent()
            .and_then(|d| donor(&ctx.root, d, &by_dir))
            .into_iter()
            .collect();
        for f in &fallbacks {
            if !donors.iter().any(|d| std::ptr::eq(*d, *f)) {
                donors.push(f);
            }
        }
        scan.push((p, src, donors));
    }
    let results: Vec<_> = scan
        .par_iter()
        .map(|(p, src, donors)| {
            let rel = p.strip_prefix(&ctx.root).unwrap_or(p).display().to_string();
            let mut best: Option<(Vec<String>, usize, String)> = None;
            for e in donors {
                // a borrowed flag set lacks the module's own include dirs: add
                // the dir that supplies each missing header and parse again
                let mut extra: Vec<String> = Vec::new();
                let mut stderr = String::new();
                for _ in 0..8 {
                    let mut args = command(ctx, p, e);
                    let tu = args.pop().unwrap_or_default();
                    args.extend(extra.iter().cloned());
                    args.push(tu);
                    let o = Command::new(&args[0])
                        .args(&args[1..])
                        .current_dir(&e.dir)
                        .output()
                        .map_err(|err| format!("{}: {err}", args[0]))?;
                    stderr = String::from_utf8_lossy(&o.stderr).into_owned();
                    let add = missing_header()
                        .captures(&stderr)
                        .and_then(|c| resolve(&headers, &c[1], p))
                        .map(|d| format!("-I{}", d.display()));
                    match add {
                        Some(flag) if !extra.contains(&flag) => extra.push(flag),
                        _ => break,
                    }
                }
                let r = hits(&p.display().to_string(), &rel, src, &stderr);
                let clean = r.1 == 0;
                if best.as_ref().is_none_or(|b| r.1 < b.1) {
                    best = Some(r);
                }
                if clean {
                    break;
                }
            }
            let (h, errs, first) = best.unwrap_or_default();
            Ok((rel, h, errs, first))
        })
        .collect::<Result<_, String>>()?;
    let mut stdout = String::new();
    let (mut n_hits, mut n_err) = (0, 0);
    for (rel, h, errs, first) in &results {
        if *errs > 0 {
            n_err += 1;
            stdout.push_str(&format!("#ERR {errs} {rel}  first: {first}\n"));
        }
        for line in h {
            n_hits += 1;
            stdout.push_str(line);
            stdout.push('\n');
        }
    }
    stdout.push_str(&format!(
        "# {}: {} flipped .c scanned ({n_err} partly parsed), \
         {n_hits} mixed-sign min/max sites (review list, not a gate)\n",
        ctx.arch,
        results.len(),
    ));
    Ok(CheckOutput::new(stdout, 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_split_handles_quotes_and_escapes() {
        assert_eq!(
            shell_split(r#"clang -DA="x y" -DB='\"q\"' a\ b  -c"#),
            ["clang", "-DA=x y", r#"-DB=\"q\""#, "a b", "-c"]
        );
    }

    #[test]
    fn keeps_only_sign_compare_on_a_minmax() {
        let src = "int f(void){\n x = min(a, b);\n if (i < n) {}\n y = CLAMP(a);\n}\n";
        let err = "\
/r/x.c:2:6: warning: comparison of integers of different signs: 'int' and 'UINT' [-Wsign-compare]
/r/x.c:3:7: warning: comparison of integers of different signs: 'int' and 'UINT' [-Wsign-compare]
/r/x.c:4:6: warning: comparison of integers of different signs: 'int' and 'UINT' [-Wsign-compare]
/r/h.h:9:1: note: expanded from macro 'max'
/r/x.c:5:1: error: boom
";
        let (h, errs, _) = hits("/r/x.c", "x.c", src, err);
        assert_eq!(errs, 1);
        assert_eq!(h.len(), 2, "{h:?}");
        assert!(h[0].starts_with("x.c:2: x = min(a, b);  || comparison"));
        assert!(
            h[1].starts_with("x.c:4: "),
            "macro note attributes line 4: {h:?}"
        );
    }
    #[test]
    fn resolve_picks_the_candidate_nearest_the_tu() {
        let mut idx = HashMap::new();
        idx.insert(
            "shared.h".to_string(),
            vec![
                PathBuf::from("/r/sdk/include/reactos/drivers/afd/shared.h"),
                PathBuf::from("/r/drivers/network/afd/include/afd/shared.h"),
                PathBuf::from("/r/dll/other/shared.h"),
            ],
        );
        let tu = Path::new("/r/drivers/network/afd/afd/main.c");
        assert_eq!(
            resolve(&idx, "afd/shared.h", tu),
            Some(PathBuf::from("/r/drivers/network/afd/include"))
        );
        assert_eq!(resolve(&idx, "nope.h", tu), None);
    }
}
