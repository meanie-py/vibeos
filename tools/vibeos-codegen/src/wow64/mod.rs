//! Source-derived WoW64 stage-1 thunk classification, service order and C output.
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArgKind {
    pub dwords: usize,
    pub c_type: String,
    pub widen: String,
    pub manual: bool,
}
pub fn arg_kind(token: &str) -> Option<ArgKind> {
    let (d, c, w, m) = match token {
        "long" => (1, "ULONG_PTR", "(ULONG_PTR)(LONG_PTR)(LONG)a[{s}]", false),
        "s_long" => (1, "LONG_PTR", "(LONG_PTR)(LONG)a[{s}]", false),
        "word" => (1, "ULONG_PTR", "(ULONG_PTR)(USHORT)a[{s}]", false),
        "ptr" | "str" | "wstr" => (1, "PVOID", "(PVOID)(LONG_PTR)(LONG)a[{s}]", false),
        "int64" => (
            2,
            "ULONGLONG",
            "((ULONGLONG)a[{s}] | ((ULONGLONG)a[{s1}] << 32))",
            false,
        ),
        "double" => (2, "double", "wow64_bits_to_double(a[{s}], a[{s1}])", false),
        "int128" => (4, "PVOID", "", true),
        _ => return None,
    };
    Some(ArgKind {
        dwords: d,
        c_type: c.into(),
        widen: w.into(),
        manual: m,
    })
}
/// All public argument kinds, keyed by the original spec token.
pub fn arg_kinds() -> std::collections::BTreeMap<String, ArgKind> {
    [
        "long", "s_long", "word", "ptr", "str", "wstr", "int64", "double", "int128",
    ]
    .into_iter()
    .map(|token| (token.to_owned(), arg_kind(token).unwrap()))
    .collect()
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Syscall {
    pub name: String,
    pub args: Vec<String>,
    pub ordinal: isize,
    pub arch: Option<String>,
    pub manual_reason: String,
    pub slots: usize,
}
impl Serialize for Syscall {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("Syscall", 7)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("args", &self.args)?;
        s.serialize_field("ordinal", &self.ordinal)?;
        s.serialize_field("arch", &self.arch)?;
        s.serialize_field("manual_reason", &self.manual_reason)?;
        s.serialize_field("slots", &self.slots)?;
        s.serialize_field("needs_manual", &self.needs_manual())?;
        s.end()
    }
}
impl Syscall {
    pub fn needs_manual(&self) -> bool {
        !self.manual_reason.is_empty()
    }
}
/// Apply the legacy classification operation in place. An unknown token returns
/// immediately, preserving the input slots exactly as the original function did.
pub fn classify(sc: &mut Syscall) {
    let mut slots = 0;
    for tok in &sc.args {
        let Some(kind) = arg_kind(tok) else {
            sc.manual_reason = format!("unknown arg token '{tok}'");
            return;
        };
        if kind.manual {
            sc.manual_reason = format!("arg '{tok}' needs deep marshalling");
        }
        slots += kind.dwords;
    }
    sc.slots = slots;
}
/// Parse spec text in source order. Unknown tokens and deep-marshalling cases become explicit manual stubs.
pub fn parse_spec(text: &str, arch: &str, manual: &HashSet<String>) -> Vec<Syscall> {
    let re =
        Regex::new(r"^\s*@\s+stdcall\s+(?:-arch=(\S+)\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)")
            .unwrap();
    let spec_arch = if arch == "amd64" { "x86_64" } else { arch };
    let mut out = Vec::new();
    for line in text.lines() {
        let Some(c) = re.captures(line) else { continue };
        let name = &c[2];
        if !name.starts_with("Nt") {
            continue;
        }
        let entry = c.get(1).map(|v| v.as_str());
        if let Some(e) = entry {
            let arches: Vec<_> = e.split(',').collect();
            let negative = arches[0].starts_with('!');
            let listed = arches
                .iter()
                .any(|s| s.trim_start_matches('!') == spec_arch);
            if (negative && listed) || (!negative && !listed) {
                continue;
            }
        }
        let mut sc = Syscall {
            name: name.into(),
            args: c[3].split_whitespace().map(str::to_owned).collect(),
            ordinal: out.len() as isize,
            arch: entry.map(str::to_owned),
            manual_reason: String::new(),
            slots: 0,
        };
        classify(&mut sc);
        if !sc.needs_manual() && manual.contains(name) {
            sc.manual_reason = "listed in --manual-list (struct layout)".into();
        }
        out.push(sc);
    }
    out
}
/// Parse service order for a 32-bit guest, retaining both branches of non-WIN64 conditionals.
pub fn parse_sysfuncs(text: &str) -> Vec<(String, usize)> {
    let cpp = Regex::new(r"^\s*#\s*(ifdef|ifndef|if|else|elif|endif)\b(.*)$").unwrap();
    let svc = Regex::new(r"^\s*SVC_\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*,\s*(\d+)\s*\)").unwrap();
    let w64 = Regex::new(r"\b_WIN64\b").unwrap();
    let mut stack: Vec<(bool, bool)> = Vec::new();
    let mut out = Vec::new();
    for line in text.lines() {
        if let Some(c) = cpp.captures(line) {
            let k = &c[1];
            let rest = &c[2];
            if ["ifdef", "ifndef", "if"].contains(&k) {
                let has = w64.is_match(rest);
                let suppressed = has && (k == "ifdef" || (k == "if" && !rest.contains('!')));
                stack.push((has, suppressed));
            } else if k == "else" {
                if let Some((true, s)) = stack.last_mut() {
                    *s = !*s;
                }
            } else if k == "endif" {
                stack.pop();
            }
            continue;
        }
        if stack.iter().any(|(_, s)| *s) {
            continue;
        }
        if let Some(c) = svc.captures(line) {
            let n = c[2]
                .parse()
                .expect("SVC_ argument count exceeds platform limits");
            out.push((format!("Nt{}", &c[1]), n));
        }
    }
    out
}
pub fn parse_manual_list(text: &str) -> HashSet<String> {
    text.lines()
        .map(|s| s.split('#').next().unwrap_or("").trim())
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}
/// Order by authoritative service list; missing/mismatched rows are manual.
pub fn order_by_sysfuncs(sys: &[(String, usize)], spec: &[Syscall]) -> Vec<Syscall> {
    let mut by_name: HashMap<_, _> = spec.iter().map(|s| (s.name.as_str(), s.clone())).collect();
    let mut earlier: HashMap<&str, Vec<usize>> = HashMap::new();
    let mut out: Vec<Syscall> = Vec::new();
    for (i, (name, argc)) in sys.iter().enumerate() {
        let mut s = by_name
            .get(name.as_str())
            .cloned()
            .unwrap_or_else(|| Syscall {
                name: name.clone(),
                args: vec![],
                ordinal: -1,
                arch: None,
                manual_reason: "in sysfuncs.h but not in .spec (no type info)".into(),
                slots: *argc,
            });
        if !s.needs_manual() && s.slots != *argc {
            s.manual_reason = format!("spec arg dwords {} != sysfuncs.h argcount {argc}", s.slots);
            s.slots = *argc;
        }
        s.ordinal = i as isize;
        // Python reuses a spec object's identity for repeated SVC names. Its later
        // mutation changes all prior table references, so mirror that final state.
        if let Some(current) = by_name.get_mut(name.as_str()) {
            *current = s.clone();
            if let Some(indices) = earlier.get(name.as_str()) {
                for &j in indices {
                    out[j] = s.clone();
                }
            }
            earlier.entry(name.as_str()).or_default().push(i);
        }
        out.push(s);
    }
    out
}
fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}
pub fn read_spec(
    path: &Path,
    arch: &str,
    manual: &HashSet<String>,
) -> Result<Vec<Syscall>, String> {
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(parse_spec(&String::from_utf8_lossy(&bytes), arch, manual))
}
pub fn read_sysfuncs(path: &Path) -> Result<Vec<(String, usize)>, String> {
    Ok(parse_sysfuncs(&read(path)?))
}
pub fn load_manual_lists(paths: &[impl AsRef<Path>]) -> Result<HashSet<String>, String> {
    let mut out = HashSet::new();
    for p in paths {
        out.extend(parse_manual_list(&read(p.as_ref())?));
    }
    Ok(out)
}
/// Convenience for the shared source-derived marshalgen audit.
pub fn compute_thunk_table(
    root: &Path,
    table: &str,
    arch: &str,
) -> Result<HashMap<String, bool>, String> {
    let cfg = metadata()["AUDIT_TABLES"]
        .get(table)
        .ok_or_else(|| format!("unknown thunk table {table}"))?;
    let spec = cfg["spec"].as_str().ok_or("missing table spec")?;
    let sys = cfg["sysfuncs"].as_str().ok_or("missing table services")?;
    let manuals = cfg["manual_lists"]
        .as_array()
        .ok_or("missing manual lists")?
        .iter()
        .map(|p| p.as_str().ok_or("invalid manual list"))
        .collect::<Result<Vec<_>, _>>()?;
    let paths: Vec<_> = manuals.iter().map(|p| root.join(p)).collect();
    let manual = load_manual_lists(&paths)?;
    let specs = read_spec(&root.join(spec), arch, &manual)?;
    let services = read_sysfuncs(&root.join(sys))?;
    Ok(order_by_sysfuncs(&services, &specs)
        .into_iter()
        .map(|s| {
            let live = !s.needs_manual();
            (s.name, live)
        })
        .collect())
}
/// Emit the exact C text; `spec` and `out` are caller-supplied provenance spellings.
pub fn emit_c(
    syscalls: &[Syscall],
    spec: &str,
    arch: &str,
    out: &str,
    symbol_prefix: &str,
) -> Result<String, String> {
    let n_manual = syscalls.iter().filter(|s| s.needs_manual()).count();
    let header = include_str!("header.txt")
        .replace("@@SPEC@@", spec)
        .replace("@@ARCH@@", arch)
        .replace("@@OUT@@", out)
        .replace("@@NAUTO@@", &(syscalls.len() - n_manual).to_string())
        .replace("@@NMANUAL@@", &n_manual.to_string());
    let mut buf = vec![header];
    for sc in syscalls {
        if sc.needs_manual() {
            buf.push(format!(
                "/* {}: WOW64_NEEDS_MANUAL — {}. Implement by hand; table entry left 0. */\n",
                sc.name, sc.manual_reason
            ));
            continue;
        }
        let mut types = Vec::new();
        let mut args = Vec::new();
        let mut slot = 0;
        for tok in &sc.args {
            let k = arg_kind(tok)
                .ok_or_else(|| format!("{}: unknown auto argument '{tok}'", sc.name))?;
            if k.manual {
                return Err(format!(
                    "{}: manual argument '{tok}' in auto thunk",
                    sc.name
                ));
            }
            types.push(k.c_type);
            args.push(
                k.widen
                    .replace("{s1}", &(slot + 1).to_string())
                    .replace("{s}", &slot.to_string()),
            );
            slot += k.dwords;
        }
        let params = if types.is_empty() {
            "void".into()
        } else {
            types.join(", ")
        };
        buf.extend([
            format!("extern NTSTATUS {}({params});", sc.name),
            format!("static NTSTATUS wow64_thunk_{}(ULONG *a)", sc.name),
            "{".into(),
            format!("    (void)a;\n    return {}({});", sc.name, args.join(", ")),
            "}".into(),
            String::new(),
        ]);
    }
    buf.push(format!(
        "static const WOW64_THUNK_ENTRY {symbol_prefix}Table[] = {{"
    ));
    for sc in syscalls {
        if sc.needs_manual() {
            buf.push(format!(
                "    {{ \"{}\", 0, {} }}, /* [{}] MANUAL: {} */",
                sc.name, sc.slots, sc.ordinal, sc.manual_reason
            ));
        } else {
            buf.push(format!(
                "    {{ \"{}\", wow64_thunk_{}, {} }}, /* [{}] */",
                sc.name, sc.name, sc.slots, sc.ordinal
            ));
        }
    }
    buf.extend(["};".into(),String::new(),format!("static const unsigned {symbol_prefix}Count = sizeof({symbol_prefix}Table) / sizeof({symbol_prefix}Table[0]);"),String::new()]);
    Ok(buf.join("\n"))
}

#[cfg(test)]
mod tests;

/// Shared source/probe recipes for flat thunks, deep marshalling and audits.
pub fn metadata() -> &'static serde_json::Value {
    static CONFIG: std::sync::OnceLock<serde_json::Value> = std::sync::OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg: serde_json::Value =
            serde_json::from_str(include_str!("config.json")).expect("committed WoW64 recipes");
        cfg["WIN32K_TUS"] = cfg["AUDIT_TABLES"]["win32k"]["tus"].clone();
        cfg["NTDLL_TU"] = cfg["AUDIT_TABLES"]["ntdll"]["tus"][0][1].clone();
        cfg
    })
}
