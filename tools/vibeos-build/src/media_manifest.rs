//! Boot-media manifests and explicit mastering plans.
use crate::{arch, cell_manifest, local, read, Error, Result};
use regex::Regex;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    process::Command,
};
pub const CDS: [&str; 3] = ["livecd", "bootcd", "bootcdregtest"];
pub const CAB: &str = "build:reactos.cab";
pub const CAB_INF: &str = "build:boot/bootdata/packages/reactos.inf";
pub const PACKAGES: &str = "{BUILD}/boot/bootdata/packages";
pub fn manifest_path(a: &str, root: &Path) -> PathBuf {
    root.join(format!("boot/media/{a}.json"))
}
pub fn enc(path: &str, a: &str, cells: &BTreeSet<String>) -> Result<String> {
    let base = format!("/tmp/vibeos/build-{a}/");
    if let Some(rel) = path.strip_prefix(&base) {
        return Ok(format!(
            "{}:{rel}",
            if cells.contains(rel) { "cell" } else { "build" }
        ));
    }
    if let Some(rel) = path.strip_prefix("/tmp/vibeos/") {
        return Ok(format!("repo:{rel}"));
    }
    Err(Error::new(format!("media source outside the tree: {path}")))
}
pub fn enc_cmd(cmd: &str, a: &str) -> String {
    cmd.replace(&format!("/tmp/vibeos/build-{a}"), "{BUILD}")
        .replace("/tmp/vibeos", "{REPO}")
}
pub fn command(ninja: &str, cd: &str) -> String {
    let p = format!(
        r"\nbuild boot/CMakeFiles/{} [^\n]*\n  COMMAND = ([^\n]*)\n",
        regex::escape(cd)
    );
    Regex::new(&p)
        .unwrap()
        .captures(ninja)
        .map(|c| c[1].into())
        .unwrap_or_default()
}
pub fn output_command(ninja: &str, out: &str) -> Result<String> {
    let p = format!(
        r"\nbuild {} [^\n]*\n  COMMAND = ([^\n]*)\n",
        regex::escape(out)
    );
    Regex::new(&p)?
        .captures(ninja)
        .map(|c| c[1].into())
        .ok_or_else(|| Error::new(format!("build.ninja: no command makes {out}")))
}
pub fn derive_cab(
    a: &str,
    build: &Path,
    ninja: &str,
    cells: &BTreeSet<String>,
    root: &Path,
) -> Result<Value> {
    let dynfile = local(
        root,
        &build.join("boot/bootdata/packages/reactos.dff.Debug.dyn"),
    )?;
    let pat = Regex::new(r#"^"([^"]+)" (.+)$"#)?;
    let mut files = Vec::new();
    for line in read(&dynfile)?.lines() {
        let c = pat.captures(line).ok_or_else(|| {
            Error::new(format!("{}: unmodelled line {line:?}", dynfile.display()))
        })?;
        files.push(json!([enc(&c[1], a, cells)?, &c[2]]));
    }
    Ok(
        json!({"dff":"repo:boot/bootdata/packages/reactos.dff.in","inf_command":enc_cmd(&output_command(ninja,"boot/bootdata/packages/reactos.inf")?,a),"cab_command":enc_cmd(&output_command(ninja,"reactos.cab")?,a),"files":files}),
    )
}
pub fn derive(a: &str, root: &Path) -> Result<Value> {
    arch(a)?;
    let build = root.join(format!("build-{a}"));
    let ninja = read(&local(root, &build.join("build.ninja"))?)?;
    let mut cells = BTreeSet::new();
    for cell in cell_manifest::current(a, root)? {
        for o in array(&cell["outputs"], "outputs")? {
            cells.insert(string(&o["out"], "cell output")?.to_owned());
        }
    }
    let mut cds = serde_json::Map::new();
    for cd in CDS {
        let p = build.join(format!("boot/{cd}.Debug.lst"));
        if !p.exists() {
            return Err(Error::new(format!(
                "{} missing: configure with ./vibeos-cli build --arch {a}",
                p.display()
            )));
        }
        let mut entries = Vec::new();
        for line in read(&local(root, &p)?)?.lines() {
            let (dest, src) = match line.rsplit_once('=') {
                Some((d, s)) => (Some(d), s),
                None => (None, line),
            };
            entries.push(json!([dest, enc(src, a, &cells)?]));
        }
        cds.insert(
            cd.into(),
            json!({"entries":entries,"command":enc_cmd(&command(&ninja,cd),a)}),
        );
    }
    let mut out = json!({"config":"Debug","cds":cds});
    if cds_values(&out)?.iter().any(|c| {
        c["entries"].as_array().is_some_and(|e| {
            e.iter()
                .any(|e| e.get(1).and_then(Value::as_str) == Some(CAB))
        })
    }) {
        out["cab"] = derive_cab(a, &build, &ninja, &cells, root)?;
    }
    Ok(out)
}
fn array<'a>(v: &'a Value, field: &str) -> Result<&'a Vec<Value>> {
    v.as_array()
        .ok_or_else(|| Error::new(format!("manifest missing {field} array")))
}
fn string<'a>(v: &'a Value, field: &str) -> Result<&'a str> {
    v.as_str()
        .ok_or_else(|| Error::new(format!("manifest missing {field} string")))
}
fn cds_values(v: &Value) -> Result<Vec<&Value>> {
    Ok(v["cds"]
        .as_object()
        .ok_or_else(|| Error::new("manifest missing cds object"))?
        .values()
        .collect())
}
pub fn committed(m: &Value, a: &str) -> Result<Value> {
    let p = Regex::new(r"(?:^|/)modules/rostests/([^/]+)/")?;
    let mut hits = BTreeSet::new();
    for spec in cds_values(m)? {
        for entry in array(&spec["entries"], "entries")? {
            let s = string(&entry[1], "source")?;
            let rel = s.split_once(':').map(|x| x.1).unwrap_or("");
            if let Some(c) = p.captures(rel) {
                if !["kmtests", "testdata"].contains(&&c[1]) {
                    hits.insert(c[1].to_owned());
                }
            }
        }
    }
    if !hits.is_empty() {
        let arch = if a.is_empty() { "<arch>" } else { a };
        let mut msg=format!("build-{arch} carries payload rostests ({}): the committed lists carry kmtests and testdata only, and these options stay on once a payload build sets them. Reset them (at the canonical mount, as every build runs), rebuild, then write: (cd build-{arch} && bash ../tools/canon-run.sh cmake -DENABLE_ROSTESTS=OFF -DENABLE_ROSTESTS_APITESTS=OFF .) && ./vibeos-cli build --arch {arch}",hits.iter().cloned().collect::<Vec<_>>().join(", "));
        if hits.contains("winetests") {
            msg.push_str(". The winetest A/B control (boot/bootdata/CMakeLists.txt) also stages build-i386's user32_winetest.exe onto i386 livecd and amd64 syswow64 whenever that file exists, so remove build-i386/modules/rostests/winetests and rebuild i386 and amd64 too");
        }
        return Err(Error::new(msg));
    }
    Ok(m.clone())
}
pub fn render(m: &Value) -> Result<String> {
    let mut out = vec![format!(
        "{{\"config\": {}, \"cds\": {{",
        super::json::dumps(&m["config"], false)
    )];
    let cds = m["cds"]
        .as_object()
        .ok_or_else(|| Error::new("manifest missing cds object"))?;
    let mut list: Vec<_> = cds.iter().collect();
    list.sort_by(|a, b| a.0.cmp(b.0));
    for (i, (cd, spec)) in list.iter().enumerate() {
        out.push(format!(" {}: {{", super::json::dumps(&json!(cd), false)));
        out.push(format!(
            "  \"command\": {},",
            super::json::dumps(&spec["command"], false)
        ));
        out.push("  \"entries\": [".into());
        out.push(
            array(&spec["entries"], "entries")?
                .iter()
                .map(|e| format!("   {}", super::json::dumps(e, false)))
                .collect::<Vec<_>>()
                .join(",\n"),
        );
        out.push("  ]".into());
        out.push(format!(" }}{}", if i + 1 < list.len() { "," } else { "" }));
    }
    if m.get("cab").is_none() {
        out.push("}}".into());
        return Ok(out.join("\n") + "\n");
    }
    out.push("}, \"cab\": {".into());
    let cab = &m["cab"];
    for k in ["dff", "inf_command", "cab_command"] {
        out.push(format!(
            "  {}: {},",
            super::json::dumps(&json!(k), false),
            super::json::dumps(&cab[k], false)
        ));
    }
    out.push("  \"files\": [".into());
    out.push(
        array(&cab["files"], "files")?
            .iter()
            .map(|e| format!("   {}", super::json::dumps(e, false)))
            .collect::<Vec<_>>()
            .join(",\n"),
    );
    out.push("  ]".into());
    out.push("}}".into());
    Ok(out.join("\n") + "\n")
}
pub fn load(a: &str, root: &Path) -> Result<Value> {
    arch(a)?;
    let p = manifest_path(a, root);
    if !p.exists() {
        return Err(Error::new(format!(
            "{} missing: python3 tools/media_manifest.py --write",
            p.display()
        )));
    }
    Ok(serde_json::from_str(&read(&local(root, &p)?)?)?)
}
pub fn current(a: &str, root: &Path) -> Result<Value> {
    if root.join(format!("build-{a}/build.ninja")).exists() {
        derive(a, root)
    } else {
        load(a, root)
    }
}
fn safe_rel(rel: &str) -> Result<&Path> {
    let p = Path::new(rel);
    if p.is_absolute()
        || p.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        })
    {
        return Err(Error::new(format!("unsafe media source path: {rel}")));
    }
    Ok(p)
}
pub fn resolve(src: &str, build: &Path, root: &Path, cells: Option<&Path>) -> Result<PathBuf> {
    let (kind, rel) = src.split_once(':').unwrap_or((src, ""));
    let base = match kind {
        "cell" => cells.unwrap_or(build),
        "build" => build,
        "repo" => root,
        _ => return Err(Error::new(format!("unknown media source {src:?}"))),
    };
    Ok(base.join(safe_rel(rel)?))
}
pub fn write_list(entries: &[Value], locations: &BTreeMap<String, PathBuf>) -> Result<String> {
    let mut lines = Vec::new();
    for e in entries {
        let src = string(&e[1], "source")?;
        let path = locations
            .get(src)
            .ok_or_else(|| Error::new(format!("no resolved source {src}")))?;
        lines.push(if e[0].is_null() {
            path.display().to_string()
        } else {
            format!("{}={}", string(&e[0], "destination")?, path.display())
        });
    }
    Ok(lines.join("\n") + "\n")
}
pub fn cells_source(cells: &Path, a: &str) -> Result<String> {
    let p = cells.join(".cells.json");
    if !p.exists() {
        return Err(Error::new(format!(
            "{} missing: build every cell with cargo_orchestrate.py --arch {a} --manifest",
            p.display()
        )));
    }
    let v: Value = serde_json::from_str(&read(&p)?)?;
    if v["arch"].as_str() != Some(a) {
        return Err(Error::new(format!(
            "{} holds {} cells, not {a}",
            cells.display(),
            v["arch"].as_str().unwrap_or("None")
        )));
    }
    Ok(string(&v["source"], "source")?.to_owned())
}
fn git(root: &Path, args: &[&str], index: Option<&Path>) -> Result<Vec<u8>> {
    let mut c = Command::new("git");
    c.current_dir(root).args(args);
    if let Some(p) = index {
        c.env("GIT_INDEX_FILE", p);
    }
    let out = c.output()?;
    if !out.status.success() {
        return Err(Error::new(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    Ok(out.stdout)
}
struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
pub fn source_hash(root: &Path) -> Result<String> {
    let raw = git(root, &["rev-parse", "--git-path", "index"], None)?;
    let s = String::from_utf8(raw).map_err(|e| Error::new(e.to_string()))?;
    let path = Path::new(s.trim());
    let index = if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    };
    let mut suffix = 0u64;
    let dir = loop {
        let p = std::env::temp_dir().join(format!("vibeos-prov-{}-{suffix}", std::process::id()));
        match std::fs::create_dir(&p) {
            Ok(()) => break p,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => suffix += 1,
            Err(e) => return Err(e.into()),
        }
    };
    let guard = Scratch(dir);
    let temp = guard.0.join("index");
    if index.is_file() {
        std::fs::copy(index, &temp)?;
    }
    git(root, &["add", "-A"], Some(&temp))?;
    let data = git(
        root,
        &[
            "ls-files",
            "-s",
            "--",
            ".",
            ":(exclude)docs",
            ":(exclude)*.md",
            ":(exclude).github",
            ":(exclude).claude",
            ":(exclude)tools/lint-baseline",
            ":(exclude)tools/cohab-census.txt",
            ":(exclude)tools/pi4",
            ":(exclude)tools/pi5",
            ":(exclude,glob)tests/**/test_*.py",
            ":(exclude,glob)tools/test_*.py",
            ":(exclude)tests/qemu/vm.py",
        ],
        Some(&temp),
    )?;
    Ok(format!("{:x}", Sha256::digest(data)))
}
pub fn check_tools(cmd: &str) -> Result<()> {
    let r = Regex::new(r"(\S+/host-tools/bin/\w+)")?;
    for c in r.captures_iter(cmd) {
        if !Path::new(&c[1]).exists() {
            return Err(Error::new(format!("{} missing: a cmake-off build's generated SDK makes it (cargo_orchestrate.py --manifest --cmake-off), a cmake build makes it too",&c[1])));
        }
    }
    Ok(())
}
pub fn run(cmd: &str) -> Result<()> {
    check_tools(cmd)?;
    let result = Command::new("bash").arg("-c").arg(cmd).output()?;
    if !result.status.success() {
        return Err(Error::new(format!(
            "media command failed ({}): {cmd}\n{}{}",
            result.status,
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        )));
    }
    Ok(())
}
fn checked_sources(root: &Path, build: &Path, cells: Option<&Path>, src: &str) -> Result<PathBuf> {
    let p = resolve(src, build, root, cells)?;
    let kind = src.split(':').next().unwrap_or("");
    if kind == "build" || (kind == "cell" && cells.is_none()) || kind == "repo" {
        return local(root, &p);
    }
    if kind == "cell" {
        return local(cells.expect("checked cell root"), &p);
    }
    Ok(p)
}
pub fn recab(
    m: &Value,
    out: &Path,
    build: &Path,
    root: &Path,
    cells: &Path,
) -> Result<BTreeMap<String, PathBuf>> {
    let build = local(root, build)?;
    let build = build.as_path();
    let cab = &m["cab"];
    let dir = out.join("cab");
    let mut rows = Vec::new();
    for e in array(&cab["files"], "cab files")? {
        let src = string(&e[0], "cab source")?;
        let p = checked_sources(root, build, Some(cells), src)?;
        if !p.exists() {
            return Err(Error::new(format!(
                "reactos.cab: source missing: {}",
                p.display()
            )));
        }
        rows.push(format!(
            "\"{}\" {}\n",
            p.display(),
            string(&e[1], "cab flags")?
        ));
    }
    let dff = checked_sources(root, build, Some(cells), string(&cab["dff"], "cab dff")?)?;
    let head = read(&dff)?;
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("reactos.dff"), head + &rows.concat())?;
    for key in ["inf_command", "cab_command"] {
        let cmd = string(&cab[key], key)?
            .replace(PACKAGES, &dir.to_string_lossy())
            .replace(
                "{BUILD}/reactos.inf",
                &dir.join("reactos.inf").to_string_lossy(),
            )
            .replace("cd {BUILD} ", &format!("cd {} ", dir.display()))
            .replace("{BUILD}", &build.to_string_lossy())
            .replace("{REPO}", &root.to_string_lossy());
        run(&cmd)?;
    }
    let made = BTreeMap::from([
        (CAB.into(), dir.join("reactos.cab")),
        (CAB_INF.into(), dir.join("reactos.inf")),
    ]);
    for p in made.values() {
        if !p.exists() {
            return Err(Error::new(format!("cabman made no {}", p.display())));
        }
    }
    Ok(made)
}
pub fn assemble(
    a: &str,
    cd: &str,
    out: &Path,
    cells: Option<&Path>,
    root: &Path,
) -> Result<PathBuf> {
    arch(a)?;
    if !CDS.contains(&cd) {
        return Err(Error::new(format!("unknown CD {cd}")));
    }
    let source = source_hash(root)?;
    if let Some(c) = cells {
        if cells_source(c, a)? != source {
            return Err(Error::new(format!("{} was built from another source tree: rebuild it with cargo_orchestrate.py --arch {a} --manifest",c.display())));
        }
    }
    let m = current(a, root)?;
    let spec = m["cds"]
        .get(cd)
        .ok_or_else(|| Error::new(format!("manifest has no {cd}")))?;
    let build = local(root, &root.join(format!("build-{a}")))?;
    std::fs::create_dir_all(out)?;
    let out = out.canonicalize()?;
    for arch in crate::ARCHES {
        let dir = root.join(format!("build-{arch}"));
        if dir.exists() && out.starts_with(dir.canonicalize()?) {
            return Err(Error::new("media output must be outside the build tree"));
        }
    }
    let entries = array(&spec["entries"], "entries")?;
    let mut where_ = BTreeMap::new();
    for e in entries {
        let src = string(&e[1], "source")?;
        let path = if cells.is_some() && (src == CAB || src == CAB_INF) {
            resolve(src, &build, root, cells)?
        } else {
            checked_sources(root, &build, cells, src)?
        };
        where_.insert(src.to_owned(), path);
    }
    if let Some(c) = cells {
        if where_.contains_key(CAB) {
            where_.extend(recab(&m, &out, &build, root, c)?);
        }
    }
    let missing: Vec<_> = where_.values().filter(|p| !p.exists()).take(3).collect();
    if !missing.is_empty() {
        return Err(Error::new(format!(
            "{cd}: sources missing, first {missing:?}"
        )));
    }
    let lst = out.join(format!("{cd}.lst"));
    std::fs::write(&lst, write_list(entries, &where_)?)?;
    let command = string(&spec["command"], "command")?;
    let image;
    if !command.is_empty() {
        let cfg = string(&m["config"], "config")?;
        image = out.join(format!("{cd}.iso"));
        let cmd = command
            .replace(
                &format!("{{BUILD}}/boot/{cd}.{cfg}.lst"),
                &lst.to_string_lossy(),
            )
            .replace(
                &format!("{{BUILD}}/boot/{cd}.stripped.lst"),
                &out.join(format!("{cd}.stripped.lst")).to_string_lossy(),
            )
            .replace(
                "{BUILD}/boot/stripped ",
                &format!("{} ", out.join("stripped").display()),
            )
            .replace(&format!("{{BUILD}}/{cd}.iso"), &image.to_string_lossy())
            .replace("{BUILD}", &build.to_string_lossy())
            .replace("{REPO}", &root.to_string_lossy());
        if image.exists() {
            std::fs::remove_file(&image)?;
        }
        run(&cmd)?;
    } else if cd == "livecd" && a == "arm64" {
        image = out.join("livecd.img");
        if image.exists() {
            std::fs::remove_file(&image)?;
        }
        let status = Command::new("bash")
            .arg(root.join("qemu/mkramdisk.sh"))
            .arg("--lst")
            .arg(&lst)
            .arg("--out-dir")
            .arg(&out)
            .status()?;
        if !status.success() {
            return Err(Error::new(format!("mkramdisk failed: {status}")));
        }
    } else {
        return Ok(lst);
    }
    if source_hash(root)? != source {
        return Err(Error::new(
            "the source tree changed while the image was mastered",
        ));
    }
    let digest = format!("{:x}", Sha256::digest(std::fs::read(&image)?));
    let stamp = json!({"arch":a,"cd":cd,"image":image.file_name().unwrap_or_default().to_string_lossy(),"sha256":digest,"source":source,"cells":cells.map(|p|p.to_string_lossy().into_owned())});
    let fields = ["arch", "cd", "image", "sha256", "source", "cells"];
    let lines = fields
        .iter()
        .map(|k| {
            format!(
                " {}: {}",
                super::json::dumps(&json!(k), false),
                super::json::dumps(&stamp[*k], false)
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    std::fs::write(
        out.join(format!("{cd}.provenance.json")),
        format!("{{\n{lines}\n}}\n"),
    )?;
    Ok(image)
}
pub fn write_plan(root: &Path, arches: &[String]) -> Result<BTreeMap<PathBuf, String>> {
    let mut out = BTreeMap::new();
    for a in arches {
        out.insert(
            manifest_path(a, root),
            render(&committed(&derive(a, root)?, a)?)?,
        );
    }
    Ok(out)
}
