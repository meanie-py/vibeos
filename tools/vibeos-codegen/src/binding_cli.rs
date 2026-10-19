use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
};
use vibeos_codegen::{asm, com, constants, delay, read, ref_protos, rpc};

pub const NAMES: &[&str] = &[
    "com-macros",
    "enum-consts",
    "ioctl-consts",
    "ks386-inc",
    "ref-protos",
    "rpc-guards",
    "delaylib",
];
pub fn selected(args: &[String]) -> bool {
    args.iter().any(|a| NAMES.contains(&a.as_str()))
}

#[derive(Default)]
struct Args {
    root: PathBuf,
    name: String,
    positional: Vec<String>,
    values: BTreeMap<String, Vec<String>>,
    check: bool,
    stdout: bool,
}
impl Args {
    fn value(&self, key: &str) -> Option<&str> {
        self.values
            .get(key)
            .and_then(|v| v.last())
            .map(String::as_str)
    }
    fn required(&self, key: &str) -> Result<&str, String> {
        self.value(key)
            .ok_or_else(|| format!("{} requires {key}", self.name))
    }
    fn flag(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
    fn many(&self, key: &str) -> Vec<String> {
        self.values.get(key).cloned().unwrap_or_default()
    }
}
fn parse(raw: Vec<String>) -> Result<Args, String> {
    let mut a = Args {
        root: std::env::current_dir().map_err(|e| e.to_string())?,
        ..Args::default()
    };
    let expanded = raw
        .into_iter()
        .flat_map(|x| {
            if x.starts_with("--") && x.contains('=') {
                let (k, v) = x.split_once('=').unwrap();
                vec![k.to_owned(), v.to_owned()]
            } else {
                vec![x]
            }
        })
        .collect::<Vec<_>>();
    let mut it = expanded.into_iter();
    while let Some(x) = it.next() {
        match x.as_str() {
            "--root" => a.root = it.next().ok_or("--root requires a directory")?.into(),
            "--stdout" => a.stdout = true,
            "--check" => {
                a.check = true;
                if matches!(a.name.as_str(), "enum-consts" | "ioctl-consts") {
                    a.values
                        .insert(x, vec![it.next().ok_or("--check requires RSFILE")?]);
                }
            }
            "--by-member" | "--parse-json" | "--keep-asm" => {
                a.values.insert(x, vec![]);
            }
            "--output-dir" | "--bindgen" | "--header" | "--out-c" | "--out-h" | "--prelude"
            | "--output" | "-o" | "--only" | "--type" | "--prefix" | "--def" | "--arch"
            | "--out" | "--clang" | "--ar" => {
                let k = if x == "-o" { "--output".into() } else { x };
                let v = it.next().ok_or_else(|| format!("{k} requires a value"))?;
                a.values.entry(k).or_default().push(v);
            }
            s if !s.starts_with('-') => {
                if a.name.is_empty() {
                    if !NAMES.contains(&s) {
                        return Err(format!("unknown binding generator: {s}"));
                    }
                    a.name = x
                } else {
                    a.positional.push(x)
                }
            }
            _ => return Err(format!("unexpected argument: {x}")),
        }
    }
    let allowed: &[&str] = match a.name.as_str() {
        "com-macros" => &["--bindgen", "--output", "--only", "--parse-json"],
        "rpc-guards" => &["--header", "--out-c", "--out-h", "--prelude"],
        "enum-consts" => &["--only", "--type", "--by-member", "--check", "--output"],
        "ioctl-consts" => &["--prefix", "--type", "--check", "--output"],
        "ks386-inc" => &["--arch", "--output"],
        "ref-protos" => &["--output-dir"],
        "delaylib" => &["--def", "--arch", "--out", "--clang", "--ar", "--keep-asm"],
        _ => return Err("missing generator".into()),
    };
    for k in a.values.keys() {
        if !allowed.contains(&k.as_str()) {
            return Err(format!("{} does not accept {k}", a.name));
        }
    }
    let count = match a.name.as_str() {
        "enum-consts" => 2,
        "ioctl-consts" | "ref-protos" => 1,
        _ => 0,
    };
    if a.positional.len() != count {
        return Err(format!("{} requires {count} positional arguments", a.name));
    }
    if a.stdout && a.check {
        return Err("--stdout and --check are mutually exclusive".into());
    }
    Ok(a)
}
fn repo_path(root: &Path, p: &str) -> PathBuf {
    let p = Path::new(p);
    if p.is_absolute() {
        p.into()
    } else {
        root.join(p)
    }
}
fn header_text(path: &Path) -> Result<String, String> {
    fs::read(path)
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .map_err(|e| format!("{}: {e}", path.display()))
}
fn header_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let e = entry.map_err(|e| e.to_string())?;
        let t = e.file_type().map_err(|e| e.to_string())?;
        if t.is_dir() {
            header_files(&e.path(), out)?
        } else if t.is_file() && e.path().extension().is_some_and(|x| x == "h") {
            out.push(e.path())
        }
    }
    Ok(())
}
fn constant_check(generated: &str, existing: &str) -> Result<u8, String> {
    let re = regex::Regex::new(r"\b([A-Za-z_]\w*)\s*:\s*\w+\s*=\s*([^;]+);").unwrap();
    let expected = re
        .captures_iter(generated)
        .map(|c| Ok((c[1].to_owned(), constants::evaluate_integer(c[2].trim())?)))
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    let mut checked = 0;
    let mut bad = false;
    for c in re.captures_iter(existing) {
        if let Some(want) = expected.get(&c[1]) {
            checked += 1;
            let have =
                constants::evaluate_integer(c[2].trim()).map_err(|e| format!("{}: {e}", &c[1]))?;
            if &have != want {
                eprintln!("MISMATCH {}: file={have} header={want}", &c[1]);
                bad = true
            }
        }
    }
    if checked == 0 {
        return Err("no generated constants found in check file".into());
    }
    println!("checked {checked} constants");
    Ok(u8::from(bad))
}
pub fn run(raw: Vec<String>) -> Result<u8, String> {
    let a = parse(raw)?;
    let mut outputs: Vec<(PathBuf, Vec<u8>)> = Vec::new();
    match a.name.as_str() {
        "com-macros" => {
            let input = a.required("--bindgen")?;
            let source = if input == "-" {
                let mut s = String::new();
                std::io::stdin()
                    .read_to_string(&mut s)
                    .map_err(|e| e.to_string())?;
                s
            } else {
                read(Path::new(input))?
            };
            if a.flag("--parse-json") {
                if a.check || a.value("--output").is_some() || a.value("--only").is_some() {
                    return Err("--parse-json cannot be combined with output/check/filter".into());
                }
                let mut data = BTreeMap::new();
                for i in com::parse_vtbls(&source)? {
                    data.insert(
                        i.name,
                        i.methods
                            .into_iter()
                            .map(|m| serde_json::json!([m.name, m.args, m.ret]))
                            .collect::<Vec<_>>(),
                    );
                }
                println!(
                    "{}",
                    serde_json::to_string(&data).map_err(|e| e.to_string())?
                );
                return Ok(0);
            }
            let text = com::generate(&source, a.value("--only"))?;
            outputs.push((a.value("--output").unwrap_or("-").into(), text.into_bytes()));
        }
        "rpc-guards" => {
            let input = Path::new(a.required("--header")?);
            let header = input
                .file_name()
                .ok_or("header has no filename")?
                .to_string_lossy();
            let h = a.required("--out-h")?;
            let c = a.required("--out-c")?;
            let (ht, ct) = rpc::render(
                &rpc::parse_header(&read(input)?)?,
                &header,
                &a.many("--prelude"),
                h,
            );
            outputs.push((h.into(), ht.into_bytes()));
            outputs.push((c.into(), ct.into_bytes()));
        }
        "enum-consts" | "ioctl-consts" => {
            let input = repo_path(&a.root, &a.positional[0]);
            let source = header_text(&input)?;
            let label = input
                .strip_prefix(&a.root)
                .unwrap_or(&input)
                .to_string_lossy()
                .into_owned();
            let ty = a.value("--type").unwrap_or("u32").to_owned();
            let text = if a.name == "enum-consts" {
                constants::enum_consts(
                    &source,
                    &a.positional[1],
                    &constants::EnumOptions {
                        rust_type: ty,
                        only: a
                            .value("--only")
                            .map(|s| {
                                s.split(',')
                                    .map(str::trim)
                                    .filter(|s| !s.is_empty())
                                    .map(str::to_owned)
                                    .collect()
                            })
                            .unwrap_or_default(),
                        by_member: a.flag("--by-member"),
                        source_name: label,
                    },
                )?
            } else {
                let mut files = Vec::new();
                header_files(&a.root.join("sdk/include"), &mut files)?;
                files.sort();
                let headers = files
                    .iter()
                    .map(|p| header_text(p))
                    .collect::<Result<Vec<_>, _>>()?;
                constants::ioctl_consts(
                    &source,
                    &headers.iter().map(String::as_str).collect::<Vec<_>>(),
                    &constants::IoctlOptions {
                        rust_type: ty,
                        prefixes: a.many("--prefix"),
                        source_name: label,
                    },
                )?
            };
            if a.check {
                return constant_check(&text, &read(&repo_path(&a.root, a.required("--check")?))?);
            }
            outputs.push((a.value("--output").unwrap_or("-").into(), text.into_bytes()));
        }
        "ks386-inc" => {
            if a.value("--arch").is_some_and(|s| s != "i386") {
                return Err("ks386-inc only supports i386".into());
            }
            outputs.push((
                a.value("--output").map(PathBuf::from).unwrap_or_else(|| {
                    a.root
                        .join("sdk/lib/rust/ntoskrnl-cargo/data/ks386-i386.inc")
                }),
                asm::ks386(&a.root)?.into_bytes(),
            ));
        }
        "ref-protos" => {
            let stem = Path::new(&a.positional[0])
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or("invalid source name")?;
            outputs.extend(
                ref_protos::generate(&a.root, stem)?
                    .into_iter()
                    .map(|(p, t)| {
                        (
                            a.value("--output-dir")
                                .map(|d| Path::new(d).join(p.file_name().unwrap()))
                                .unwrap_or(p),
                            t.into_bytes(),
                        )
                    }),
            );
        }
        "delaylib" => {
            let source = read(Path::new(a.required("--def")?))?;
            let arch = a.required("--arch")?;
            let text = delay::emit(&source, arch)?;
            if a.stdout {
                print!("{text}");
                return Ok(0);
            }
            let out = PathBuf::from(a.required("--out")?);
            let bytes = delay::archive(
                &text,
                arch,
                &out,
                a.value("--clang").unwrap_or("clang"),
                a.value("--ar").unwrap_or("llvm-ar"),
            )?;
            outputs.push((out.clone(), bytes));
            if a.flag("--keep-asm") {
                outputs.push((out.with_extension("delay.s"), text.into_bytes()))
            }
        }
        _ => unreachable!(),
    }
    if a.stdout || outputs.iter().any(|(p, _)| p == Path::new("-")) {
        if outputs.len() != 1 || a.check {
            return Err("stdout requires exactly one output and no --check".into());
        }
        use std::io::Write;
        std::io::stdout()
            .write_all(&outputs[0].1)
            .map_err(|e| e.to_string())?;
        return Ok(0);
    }
    let mut stale = false;
    for (i, (path, bytes)) in outputs.iter().enumerate() {
        if a.check {
            if fs::read(path).ok().as_deref() != Some(bytes.as_slice()) {
                eprintln!("DRIFT: {}", path.display());
                stale = true
            }
        } else {
            crate::write_atomic(path, bytes, i)?;
        }
    }
    Ok(u8::from(stale))
}
