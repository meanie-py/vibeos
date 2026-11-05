use serde_json::{json, Value};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
};
use vibeos_codegen::{
    marshal::{self as m, Param, Proto, Record},
    wow64,
};
pub const NAMES: &[&str] = &["marshal"];
/// The shipping Rust twins of the two C outputs (wow64-rust modules).
const RS_AUTOMARSHAL: &str = "sdk/lib/rust/wow64-rust/src/wow64/wow64_automarshal.rs";
const RS_CALLBACK: &str = "sdk/lib/rust/wow64-rust/src/wow64/wow64_cb_automarshal.rs";
type Db = HashMap<String, Record>;
fn field<'a>(v: &'a Value, k: &str) -> Result<&'a str, String> {
    v.get(k)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string field {k}"))
}
fn decode<T: serde::de::DeserializeOwned>(v: &Value, k: &str) -> Result<T, String> {
    serde_json::from_value(
        v.get(k)
            .cloned()
            .ok_or_else(|| format!("missing field {k}"))?,
    )
    .map_err(|e| e.to_string())
}
fn path(root: &Path, p: &str) -> PathBuf {
    let p = Path::new(p);
    if p.is_absolute() {
        p.to_owned()
    } else {
        root.join(p)
    }
}
fn source_table(root: &Path, cfg: &Value, arch: &str) -> Result<Vec<(String, bool)>, String> {
    let manuals: Vec<String> = decode(cfg, "manual_lists")?;
    let files = manuals.iter().map(|p| root.join(p)).collect::<Vec<_>>();
    let manual = wow64::load_manual_lists(&files)?;
    let spec = wow64::read_spec(&root.join(field(cfg, "spec")?), arch, &manual)?;
    let sys = wow64::read_sysfuncs(&root.join(field(cfg, "sysfuncs")?))?;
    let rows = wow64::order_by_sysfuncs(&sys, &spec);
    let mut order = Vec::new();
    let mut values = HashMap::new();
    for sc in rows {
        if !values.contains_key(&sc.name) {
            order.push(sc.name.clone());
        }
        values.insert(sc.name.clone(), !sc.needs_manual());
    }
    Ok(order
        .into_iter()
        .map(|name| {
            let live = values[&name];
            (name, live)
        })
        .collect())
}
fn api(root: &Path) -> Result<u8, String> {
    let mut text = String::new();
    std::io::stdin()
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    let v: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let result = match field(&v, "op")? {
        "metadata" => m::metadata().clone(),
        "parse_layout_dump" => json!(m::parse_layout_dump(field(&v, "text")?)),
        "extract_prototypes" => json!(m::extract_prototypes(
            field(&v, "text")?,
            field(&v, "pattern")?
        )?),
        "strip_annotations" => json!(m::strip_annotations(field(&v, "text")?)),
        "strip_comments" => json!(m::strip_comments(field(&v, "text")?)),
        "norm_type" => json!(m::norm_type(field(&v, "text")?)),
        "resolve_exact" | "resolve_pointee" => {
            let db: Db = decode(&v, "db")?;
            let t = field(&v, "ctype")?;
            json!(if field(&v, "op")? == "resolve_exact" {
                m::resolve_exact(t, &db)
            } else {
                m::resolve_pointee(t, &db)
            })
        }
        "diff_struct" => {
            let a: Db = decode(&v, "db32")?;
            let b: Db = decode(&v, "db64")?;
            json!(m::diff_struct(field(&v, "name")?, &a, &b))
        }
        "diff_structs" => {
            let a: Db = decode(&v, "db32")?;
            let b: Db = decode(&v, "db64")?;
            let names: Vec<String> = decode(&v, "names")?;
            json!(names
                .into_iter()
                .map(|n| {
                    let verdict = m::diff_struct(&n, &a, &b);
                    (n, verdict)
                })
                .collect::<HashMap<_, _>>())
        }
        "classify_param" => {
            let a: Db = decode(&v, "db32")?;
            let b: Db = decode(&v, "db64")?;
            let p: Param = decode(&v, "param")?;
            json!(m::classify_param(&p, &a, &b))
        }
        "extract_flags" => json!(m::extract_flags(
            root,
            &path(root, field(&v, "build")?),
            field(&v, "suffix")?
        )?),
        "dump_layouts" => {
            let flags: Vec<String> = decode(&v, "flags")?;
            json!(m::dump_layouts(root, &flags, field(&v, "tu")?)?)
        }
        "source_table" => {
            let cfg = v.get("cfg").ok_or("missing cfg")?;
            json!(source_table(root, cfg, field(&v, "arch")?)?)
        }
        "load_thunk_table" => {
            let text = String::from_utf8_lossy(
                &fs::read(path(root, field(&v, "file")?)).map_err(|e| e.to_string())?,
            )
            .into_owned();
            json!(m::parse_table_artifact(&text))
        }
        "crosscheck" => {
            let computed: HashMap<String, bool> = decode(&v, "computed")?;
            let file = path(root, field(&v, "file")?);
            let text = vibeos_codegen::read(&file)?;
            m::crosscheck(&computed, &text, &file, field(&v, "table")?)?;
            Value::Null
        }
        "audit_verdict" => json!(m::audit_verdict(field(&v, "cls")?, field(&v, "detail")?)),
        "audit" => json!(m::audit(
            root,
            v["build64"].as_str(),
            v["build32"].as_str(),
            field(&v, "table")?
        )?),
        "generate" => {
            let a: Db = decode(&v, "db32")?;
            let b: Db = decode(&v, "db64")?;
            let p: HashMap<String, Proto> = decode(&v, "protos")?;
            let manual: Vec<String> = decode(&v, "manual")?;
            let linked: HashSet<String> = decode(&v, "linkable")?;
            let hand: HashSet<String> = decode(&v, "hand_written")?;
            json!(m::render::generate(
                &a,
                &b,
                &p,
                &manual,
                &linked,
                &hand,
                field(&v, "export_spec")?
            ))
        }
        "generate_callback" => {
            let a: Db = decode(&v, "db32")?;
            let b: Db = decode(&v, "db64")?;
            json!(m::render::generate_callback(&a, &b)?)
        }
        _ => return Err("unknown marshal API operation".into()),
    };
    println!("{result}");
    Ok(0)
}
fn windows_target(flags: &[String]) -> Result<&'static str, String> {
    let triple = flags
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.strip_prefix("--target=")
                .or_else(|| s.strip_prefix("-target="))
                .or_else(|| {
                    if ["-target", "--target"].contains(&s.as_str()) {
                        flags.get(i + 1).map(String::as_str)
                    } else {
                        None
                    }
                })
        })
        .ok_or("layout generation requires an explicit Windows target in the compile recipe")?;
    if !(triple.contains("windows") || triple.contains("mingw")) {
        return Err(format!(
            "layout generation requires a Windows target, found {triple}"
        ));
    }
    if triple.starts_with("aarch64") {
        Ok("arm64")
    } else if triple.starts_with("x86_64") {
        Ok("amd64")
    } else if triple.starts_with("i686") || triple.starts_with("i386") {
        Ok("i386")
    } else {
        Err(format!("unsupported layout target {triple}"))
    }
}

pub fn run(raw: Vec<String>) -> Result<u8, String> {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (mut mode, mut b64, mut b32, mut out, mut out_header) = (
        None,
        "build-arm64".to_owned(),
        "build-i386".to_owned(),
        "dll/win32/wow64/wow64_automarshal.c".to_owned(),
        "dll/win32/wow64/wow64_cb_automarshal.h".to_owned(),
    );
    let (mut check, mut api_json) = (false, false);
    let mut headers =
        "win32ss/include/ntuser.h,sdk/include/psdk/ntgdi.h,win32ss/include/ntgdibad.h".to_owned();
    let mut exports = "win32ss/win32u/win32u.spec".to_owned();
    let mut manual_list = "tools/wow64gen/manual-win32k.txt".to_owned();
    let mut out_set = false;
    let mut out_rs: Option<String> = None;
    let mut supplied = Vec::new();
    let expanded = raw
        .into_iter()
        .flat_map(|arg| {
            if arg.starts_with("--") && arg.contains('=') {
                let (k, v) = arg.split_once('=').unwrap();
                vec![k.to_owned(), v.to_owned()]
            } else {
                vec![arg]
            }
        })
        .collect::<Vec<_>>();
    let mut args = expanded.into_iter();
    while let Some(arg) = args.next() {
        if [
            "--build-dir",
            "--build-dir32",
            "--headers",
            "--export-spec",
            "--manual-list",
            "--out",
            "--out-header",
            "--out-rs",
            "--tu-suffix",
        ]
        .contains(&arg.as_str())
        {
            supplied.push(arg.clone());
        }
        match arg.as_str() {
            "--root" => root = args.next().ok_or("--root requires DIR")?.into(),
            "marshal" => {}
            "gen" | "gen-callback" | "report" | "audit" if mode.is_none() => mode = Some(arg),
            "--api-json" => api_json = true,
            "--check" => check = true,
            "--build-dir" => b64 = args.next().ok_or("--build-dir requires DIR")?,
            "--build-dir32" => b32 = args.next().ok_or("--build-dir32 requires DIR")?,
            "--headers" => headers = args.next().ok_or("--headers requires FILES")?,
            "--export-spec" => exports = args.next().ok_or("--export-spec requires FILE")?,
            "--manual-list" => manual_list = args.next().ok_or("--manual-list requires FILE")?,
            "--out" => {
                out = args.next().ok_or("--out requires FILE")?;
                out_set = true;
            }
            "--out-header" => out_header = args.next().ok_or("--out-header requires FILE")?,
            "--out-rs" => out_rs = Some(args.next().ok_or("--out-rs requires FILE")?),
            "--tu-suffix" => {
                let value = args.next().ok_or("--tu-suffix requires NAME")?;
                if value != "user32_layout_probe.c" {
                    return Err(
                        "--tu-suffix is obsolete; both configured Win32k probes are required"
                            .into(),
                    );
                }
            }
            _ => return Err(format!("unexpected marshal argument: {arg}")),
        }
    }
    if api_json {
        if mode.is_some() || check || !supplied.is_empty() {
            return Err("--api-json cannot combine with a mode/check".into());
        }
        return api(&root);
    }
    let mode = mode.ok_or("marshal requires report, gen or gen-callback")?;
    if mode == "audit" {
        return Err(
            "marshal audit moved to ./vibeos-cli check (Rust rule: marshalgen_audit)".into(),
        );
    }
    if mode == "report" && check {
        return Err("report does not support --check".into());
    }
    for flag in &supplied {
        let allowed = match mode.as_str() {
            "report" => ["--build-dir", "--build-dir32", "--headers", "--tu-suffix"]
                .contains(&flag.as_str()),
            "gen" => [
                "--build-dir",
                "--build-dir32",
                "--headers",
                "--export-spec",
                "--manual-list",
                "--out",
                "--out-rs",
            ]
            .contains(&flag.as_str()),
            _ => ["--build-dir", "--build-dir32", "--out", "--out-header", "--out-rs"]
                .contains(&flag.as_str()),
        };
        if !allowed {
            return Err(format!("{flag} is not supported by marshal {mode}"));
        }
    }
    if mode == "report" {
        eprintln!("== dumping layouts (32-bit + 64-bit) ...");
    }
    let build64 = path(&root, &b64);
    let build32 = path(&root, &b32);
    let (mut d32, mut d64) = (HashMap::new(), HashMap::new());
    let mut host_arch = None;
    for (suffix, tu) in m::win32k_tus() {
        for (db, build, width) in [(&mut d32, &build32, 4), (&mut d64, &build64, 8)] {
            let flags = m::extract_flags(&root, build, suffix)?;
            let target = windows_target(&flags)?;
            let records = m::dump_expected(&root, &flags, tu, width)?;
            if width == 8 {
                if host_arch.is_some_and(|a| a != target) {
                    return Err("64-bit layout probes use conflicting target architectures".into());
                }
                host_arch = Some(target);
            }
            for (k, v) in records {
                db.entry(k).or_insert(v);
            }
        }
    }
    if mode == "report" {
        eprintln!("   {} / {} records", d32.len(), d64.len());
    }
    let mut outputs = Vec::new();
    let status;
    if mode == "gen-callback" {
        if !out_set {
            out = "dll/win32/wow64/wow64_cb_automarshal.c".into();
        }
        let g = m::render::generate_callback(&d32, &d64)?;
        status = format!(
            "wrote {out} + {out_header}: {} table messages, {} struct pairs",
            g.table_messages, g.struct_pairs
        );
        outputs.push((path(&root, &out), g.c));
        outputs.push((path(&root, &out_header), g.header));
        let rs = out_rs.as_deref().unwrap_or(RS_CALLBACK);
        outputs.push((path(&root, rs), g.rs));
    } else {
        let mut text = Vec::new();
        for h in headers.split(',') {
            text.push(vibeos_codegen::read(&path(&root, h))?);
        }
        let protos = m::extract_prototypes(&text.join("\n"), r"Nt\w+")?;
        let cfg = &m::metadata()["AUDIT_TABLES"]["win32k"];
        let arch = host_arch.ok_or("missing host layout target")?;
        let manual = source_table(&root, cfg, arch)?
            .into_iter()
            .filter(|(_, live)| !*live)
            .map(|(n, _)| n)
            .collect::<Vec<_>>();
        if mode == "report" {
            eprintln!("   {} prototypes", protos.len());
            eprintln!(
                "   {} NEEDS_MANUAL stubs (from the checked-in lists)",
                manual.len()
            );
            let mut lines = Vec::new();
            let names = [
                "FLAT-OK",
                "SCALAR_WIDEN",
                "WIDENING",
                "SEMANTIC",
                "NO-PROTO",
            ];
            let mut counts = [0usize; 5];
            for name in &manual {
                let Some(p) = protos.get(name) else {
                    lines.push(format!("{:13} {name}", "NO-PROTO"));
                    counts[4] += 1;
                    continue;
                };
                let (mut rank, mut details) = (0usize, Vec::new());
                for prm in &p.params {
                    let (cls, det) = m::classify_param(prm, &d32, &d64);
                    let r = match cls.as_str() {
                        "FLAT" | "IDENTICAL" => 0,
                        "SCALAR_WIDEN" => 1,
                        "WIDENING" => 2,
                        _ => 3,
                    };
                    rank = rank.max(r);
                    if r > 0 {
                        details.push(format!("{}:{cls}{det}", prm.name));
                    }
                }
                counts[rank] += 1;
                lines.push(format!(
                    "{:13} {name:40} {}",
                    names[rank],
                    details.join("; ")
                ));
            }
            lines.sort();
            println!("{}\n", lines.join("\n"));
            let total: usize = counts.iter().sum();
            for (n, c) in names.iter().zip(counts) {
                println!("{n:13} {c:4}  ({}%)", 100 * c / total.max(1));
            }
            println!("{:13} {total:4}", "TOTAL");
            return Ok(0);
        }
        let export_text = vibeos_codegen::read(&path(&root, &exports))?;
        let re = regex::Regex::new(r"(?m)^@ stdcall (?:-arch=\S+ )?(\w+)\(").unwrap();
        let linked = re
            .captures_iter(&export_text)
            .map(|c| c[1].to_owned())
            .collect();
        let hand = vibeos_codegen::read(&path(&root, &manual_list))?
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .filter_map(|l| l.split_whitespace().next().map(str::to_owned))
            .collect();
        let g = m::render::generate(&d32, &d64, &protos, &manual, &linked, &hand, &exports);
        let dest = path(&root, &out);
        status = format!(
            "wrote {}: {} thunks, {} struct marshaller pairs, {} skipped",
            dest.display(),
            g.thunks,
            g.struct_pairs,
            g.skipped.len()
        );
        outputs.push((dest, g.c));
        let rs = out_rs.as_deref().unwrap_or(RS_AUTOMARSHAL);
        outputs.push((path(&root, rs), g.rs));
    }
    let mut stale = false;
    for (i, (file, text)) in outputs.iter().enumerate() {
        if check {
            if fs::read(file).ok().as_deref() != Some(text.as_bytes()) {
                eprintln!("DRIFT: {}", file.display());
                stale = true;
            }
        } else {
            if let Some(dir) = file.parent() {
                fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
            }
            crate::write_atomic(file, text.as_bytes(), i)?;
        }
    }
    if !check {
        eprintln!("{status}");
    }
    Ok(u8::from(stale))
}
