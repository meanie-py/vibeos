//! Media-scoped WoW64 ladder and Wine-subtest verdicts. UNKNOWN media stays unknown.
use crate::verdict::{args, py_lines, re, string};
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    process::Command,
};
pub const PROGRAM_DONE: &str = "[WINLOGON] graded program complete";
pub const TIMEOUT_SECONDS: f64 = 420.0;
pub const OPTIONAL_EXTRA_SECONDS: f64 = 360.0;
pub const WINETEST_EXTRA_SECONDS: f64 = 200.0;
const BUILDER: &str = "tools/fex/build-hello-compute.sh";
const PREREQ: &str = "build-i386/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a";
const PREREQ64: &str = "build-amd64/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a";
const TOOLCHAIN: &str = ".build-tools/llvm-mingw/bin/i686-w64-mingw32-clang";
const REQUIRED: &[&str] = &[
    "hello32",
    "gui32",
    "win32",
    "file32",
    "async32",
    "marshal32",
    "thread32",
    "fault32",
    "selfmod32",
    "reg32",
    "spawn32",
    "apc32",
    "hello64",
];
const STAGED: &[&str] = &[
    "gui64",
    "win64",
    "file64",
    "async64",
    "marshal64",
    "fault64",
    "thread64",
    "selfmod64",
    "reg64",
    "spawn64",
    "apc64",
];
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub ladder: Vec<(String, u64)>,
    pub optional: Vec<(String, u64)>,
    pub diagnostic: Vec<(String, u64)>,
    pub staged_ladder: Vec<(String, u64)>,
    #[serde(default)]
    pub also_accepted: BTreeMap<String, BTreeSet<u64>>,
    pub guest_source: BTreeMap<String, String>,
    #[serde(default)]
    pub guest_helpers: BTreeMap<String, Vec<String>>,
    pub optional_source: Vec<(String, (String, Vec<String>))>,
    pub guest_builder: String,
    pub guest_prereq: String,
    pub guest_prereq_x64: String,
    pub guest_toolchain: String,
    pub ramdisk_rel: String,
}
impl Default for Config {
    fn default() -> Self {
        let sources = [
            ("hello32", "Hello32.c"),
            ("gui32", "Gui32.c"),
            ("win32", "Win32App.c"),
            ("file32", "File32.c"),
            ("async32", "Async32.c"),
            ("marshal32", "Marshal32.c"),
            ("thread32", "Thread32.c"),
            ("fault32", "Fault32.c"),
            ("selfmod32", "Selfmod32.c"),
            ("reg32", "Reg32.c"),
            ("spawn32", "Spawn32.c"),
            ("apc32", "Apc64.c"),
            ("hello64", "Hello64.c"),
            ("gui64", "Gui32.c"),
            ("win64", "Win32App.c"),
            ("file64", "File32.c"),
            ("async64", "Async32.c"),
            ("marshal64", "Marshal32.c"),
            ("thread64", "Thread32.c"),
            ("fault64", "Fault32.c"),
            ("selfmod64", "Selfmod32.c"),
            ("reg64", "Reg32.c"),
            ("spawn64", "Spawn32.c"),
            ("apc64", "Apc64.c"),
        ];
        Self {
            ladder: REQUIRED.iter().map(|n| (n.to_string(), 0x600d)).collect(),
            optional: vec![("sqlite32".into(), 0x600d), ("lua32".into(), 0x600d)],
            diagnostic: Vec::new(),
            staged_ladder: STAGED.iter().map(|n| (n.to_string(), 0x600d)).collect(),
            also_accepted: BTreeMap::new(),
            guest_source: sources
                .iter()
                .map(|(g, p)| (g.to_string(), format!("tools/fex/hello-compute/{p}")))
                .collect(),
            guest_helpers: [(
                "spawn64".into(),
                vec!["tools/fex/hello-compute/XInject.c".into()],
            )]
            .into_iter()
            .collect(),
            optional_source: vec![
                (
                    "lua32".into(),
                    (
                        "tools/stress/lua/lua32.exe".into(),
                        vec![
                            "tools/stress/lua/Lua32.c".into(),
                            "tools/stress/lua/build.sh".into(),
                        ],
                    ),
                ),
                (
                    "sqlite32".into(),
                    (
                        "tools/stress/sqlite/sqlite32.exe".into(),
                        vec![
                            "tools/stress/sqlite/Sqlite32.c".into(),
                            "tools/stress/sqlite/build.sh".into(),
                        ],
                    ),
                ),
            ],
            guest_builder: BUILDER.into(),
            guest_prereq: PREREQ.into(),
            guest_prereq_x64: PREREQ64.into(),
            guest_toolchain: TOOLCHAIN.into(),
            ramdisk_rel: "qemu/ramdisk.img".into(),
        }
    }
}
pub fn constants() -> Value {
    let c = Config::default();
    json!({"timeout_seconds":TIMEOUT_SECONDS,"optional_extra_seconds":OPTIONAL_EXTRA_SECONDS,"winetest_extra_seconds":WINETEST_EXTRA_SECONDS,"program_done":PROGRAM_DONE,"drain_floor_env":"VIBEOS_LADDER_DRAIN_FLOOR","staged_dir":"syswow64","x64_staged_dir":"system32","cpu_layers":["wow64cpu.dll","libwow64fex.dll"],"config":c})
}
pub fn verdict_ok(c: &Config, guest: &str, got: u64, want: u64) -> bool {
    got == want || c.also_accepted.get(guest).is_some_and(|s| s.contains(&got))
}
pub fn staged_guests(c: &Config) -> Vec<String> {
    c.ladder.iter().map(|(g, _)| g.clone()).collect()
}
pub fn markers(c: &Config) -> Vec<String> {
    c.ladder
        .iter()
        .map(|(g, _)| format!("{g}.exe exit = "))
        .collect()
}
pub fn verdicts(text: &str) -> Result<BTreeMap<String, u64>> {
    let mut seen = BTreeMap::new();
    for cap in re(r"\[WINLOGON\] ([A-Za-z0-9_]+)\.exe exit = 0x([0-9a-fA-F]+)").captures_iter(text)
    {
        let code = u64::from_str_radix(&cap[2], 16)
            .map_err(|_| Error::new("ladder exit code exceeds u64 range"))?;
        seen.insert(cap[1].into(), code);
    }
    Ok(seen)
}
pub fn winetests(text: &str) -> Result<BTreeMap<String, u64>> {
    let mut out = BTreeMap::new();
    for c in re(r"\[WT\] ([A-Za-z0-9_/]+:[A-Za-z0-9_]+): exit = (\d+) failure").captures_iter(text)
    {
        out.insert(
            c[1].into(),
            c[2].parse()
                .map_err(|_| Error::new("winetest failure count exceeds u64 range"))?,
        );
    }
    Ok(out)
}
pub fn winetests_crashed(text: &str) -> BTreeMap<String, String> {
    re(r"\[WT\] ([A-Za-z0-9_/]+:[A-Za-z0-9_]+): CRASHED, exit = (0x[0-9a-fA-F]+)")
        .captures_iter(text)
        .map(|c| (c[1].into(), c[2].into()))
        .collect()
}
pub fn winetests_timed_out(text: &str) -> Vec<String> {
    re(r"\[WT\] ([A-Za-z0-9_/]+:[A-Za-z0-9_]+): TIMED OUT,")
        .captures_iter(text)
        .map(|c| c[1].into())
        .collect()
}
pub fn winetests_lost(text: &str) -> Result<Vec<String>> {
    let mut done: BTreeSet<_> = winetests(text)?.into_keys().collect();
    done.extend(winetests_crashed(text).into_keys());
    done.extend(winetests_timed_out(text));
    Ok(re(r"\[WT\] ([A-Za-z0-9_/]+:[A-Za-z0-9_]+): started ")
        .captures_iter(text)
        .map(|c| c[1].to_owned())
        .filter(|n| !done.contains(n))
        .collect())
}
pub fn sweeps_unfinished(text: &str) -> Vec<String> {
    let done: BTreeSet<_> =
        re(r"\[WT\] ([A-Za-z0-9_/]+): (?:sweep complete|--list produced no subtest)")
            .captures_iter(text)
            .map(|c| c[1].to_owned())
            .collect();
    re(r"\[WT\] ([A-Za-z0-9_/]+):--list: started ")
        .captures_iter(text)
        .map(|c| c[1].to_owned())
        .filter(|s| !done.contains(s))
        .collect()
}
pub fn optional_in_flight(c: &Config, text: &str) -> Vec<String> {
    c.optional
        .iter()
        .filter(|(g, _)| {
            text.contains(&format!("{g}.exe (")) && !text.contains(&format!("{g}.exe exit = "))
        })
        .map(|(g, _)| g.clone())
        .collect()
}
pub fn watch_complete(
    c: &Config,
    text: &str,
    optional_staged: &[String],
    program_done_expected: bool,
) -> Result<bool> {
    if program_done_expected {
        return Ok(text.contains(PROGRAM_DONE));
    }
    if !markers(c).iter().all(|m| text.contains(m)) {
        return Ok(false);
    }
    let mut pending = optional_in_flight(c, text);
    for g in optional_staged {
        if !text.contains(&format!("{g}.exe exit = ")) && !pending.contains(g) {
            pending.push(g.clone());
        }
    }
    Ok(
        pending.is_empty()
            && winetests_lost(text)?.is_empty()
            && sweeps_unfinished(text).is_empty(),
    )
}
pub fn check(c: &Config, text: &str, optional_staged: &[String]) -> Result<Value> {
    let seen = verdicts(text)?;
    let mut missing: Vec<_> = c
        .ladder
        .iter()
        .filter(|(g, _)| !seen.contains_key(g))
        .map(|(g, _)| g.clone())
        .collect();
    let mut wrong: Vec<_> = c
        .ladder
        .iter()
        .filter_map(|(g, want)| {
            seen.get(g)
                .filter(|got| !verdict_ok(c, g, **got, *want))
                .map(|got| (g.clone(), *got, *want))
        })
        .collect();
    for (g, want) in c.optional.iter().chain(c.staged_ladder.iter()) {
        if let Some(got) = seen.get(g).filter(|got| **got != *want) {
            wrong.push((g.clone(), *got, *want));
        }
    }
    for g in optional_staged {
        if !seen.contains_key(g) && !missing.contains(g) {
            missing.push(g.clone());
        }
    }
    for g in optional_in_flight(c, text) {
        if !missing.contains(&g) {
            missing.push(g);
        }
    }
    let mut lost = winetests_lost(text)?;
    lost.extend(
        sweeps_unfinished(text)
            .into_iter()
            .map(|s| format!("{s}:--list")),
    );
    if missing.is_empty() && wrong.is_empty() && lost.is_empty() {
        return Ok(Value::Null);
    }
    let mut salient: Vec<_> = lost
        .iter()
        .map(|l| format!("winetest {l} started and never reported — the guest crashed or hung"))
        .collect();
    if missing.len() == c.ladder.len() {
        return Ok(
            json!({"classification":"LADDER_NOT_STAGED","salient":["no ladder guest reported a verdict — the media has no WoW64 guests staged","arm64 stages them only under VIBEOS_FEX=1: `VIBEOS_FEX=1 ./vibeos-cli build --arch arm64`","amd64 stages from dll/win32/wow64cpu/CMakeLists.txt; guests build via tools/fex/build-hello-compute.sh"]}),
        );
    }
    salient.extend(wrong.into_iter().map(|(g, got, want)| {
        format!("{g}.exe exit = 0x{got:04x} (want 0x{want:04x}) — see [{g}] serial lines")
    }));
    salient.extend(
        missing
            .into_iter()
            .map(|g| format!("{g}.exe never reported a verdict (crashed, hung, or not staged)")),
    );
    Ok(json!({"classification":"LADDER_FAIL","salient":salient}))
}
pub fn diagnostic_tail(c: &Config, seen: &BTreeMap<String, u64>) -> String {
    let mut diagnostic = c.diagnostic.clone();
    diagnostic.sort();
    let bits: Vec<_> = diagnostic
        .into_iter()
        .filter_map(|(g, want)| {
            seen.get(&g).map(|got| {
                if *got == want {
                    format!("{g} PASSES (0x{got:04x}) — gap closed, promote it to LADDER")
                } else {
                    format!("{g}=0x{got:04x}")
                }
            })
        })
        .collect();
    if bits.is_empty() {
        String::new()
    } else {
        format!("; diagnostic: {}", bits.join(", "))
    }
}
pub fn summary(c: &Config, text: &str) -> Result<String> {
    let seen = verdicts(text)?;
    let ok = c
        .ladder
        .iter()
        .filter(|(g, w)| seen.get(g).is_some_and(|got| verdict_ok(c, g, *got, *w)))
        .count();
    let extra: Vec<_> = seen
        .keys()
        .filter(|g| {
            !c.ladder
                .iter()
                .chain(c.diagnostic.iter())
                .chain(c.staged_ladder.iter())
                .any(|(n, _)| n == *g)
        })
        .cloned()
        .collect();
    let x64: Vec<_> = c
        .staged_ladder
        .iter()
        .filter(|(g, _)| seen.contains_key(g))
        .collect();
    let x64_ok = x64
        .iter()
        .filter(|(g, w)| seen.get(g).is_some_and(|v| *v == *w))
        .count();
    let mut wt: BTreeSet<_> = winetests(text)?
        .into_iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect();
    wt.extend(
        winetests_crashed(text)
            .into_iter()
            .map(|(k, v)| format!("{k}=CRASH({v})")),
    );
    wt.extend(
        winetests_timed_out(text)
            .into_iter()
            .map(|k| format!("{k}=TIMEOUT")),
    );
    Ok(format!(
        "{ok}/{} guests PASS{}{}{}{}",
        c.ladder.len(),
        if x64.is_empty() {
            String::new()
        } else {
            format!("; x86-64 ladder {x64_ok}/{} PASS", x64.len())
        },
        if extra.is_empty() {
            String::new()
        } else {
            format!(" (+{} non-parity: {})", extra.len(), extra.join(", "))
        },
        diagnostic_tail(c, &seen),
        if wt.is_empty() {
            String::new()
        } else {
            format!(
                "; winetests {}",
                wt.into_iter().collect::<Vec<_>>().join(", ")
            )
        }
    ))
}
pub fn drain_floor(raw: Option<&str>) -> Result<f64> {
    let Some(raw) = raw.filter(|s| !s.is_empty()) else {
        return Ok(0.0);
    };
    let value = raw.parse::<f64>().map_err(|_| {
        Error::new(format!(
            "VIBEOS_LADDER_DRAIN_FLOOR={raw:?} is not a number of seconds"
        ))
    })?;
    if !value.is_finite() {
        return Err(Error::new(format!(
            "VIBEOS_LADDER_DRAIN_FLOOR={raw:?} is not a finite number of seconds"
        )));
    }
    if value < 0.0 {
        return Err(Error::new(format!(
            "VIBEOS_LADDER_DRAIN_FLOOR={raw:?} must not be negative"
        )));
    }
    Ok(value)
}
fn base_arch(a: &str) -> String {
    a.replace("-net", "")
        .replace("-uefi", "")
        .replace("-bios", "")
        .replace("-test", "")
}
fn media_rel(arch: &str, override_rel: Option<&str>) -> Option<String> {
    if let Some(p) = override_rel {
        return if p.is_empty() { None } else { Some(p.into()) };
    }
    let base = base_arch(arch);
    match base.as_str() {
        "i386" | "amd64" => Some(format!("build-{base}/livecd.iso")),
        "arm64" => Some("qemu/boot.img".into()),
        _ => None,
    }
}
pub fn uses_ramdisk(arch: &str, media: Option<&str>) -> bool {
    media_rel(arch, media)
        .is_some_and(|m| !m.split('/').next().is_some_and(|p| p.starts_with("build-")))
}
pub fn staged_per_manifest(
    repo: &Path,
    arch: &str,
    subdir: &str,
    names: &BTreeSet<String>,
    media: Option<&str>,
) -> Option<BTreeSet<String>> {
    let rel = media_rel(arch, media)?;
    let stem = Path::new(&rel).file_stem()?.to_str()?;
    let dir = repo.join(format!("build-{}/boot", base_arch(arch)));
    let mut paths: Vec<_> = fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(&format!("{stem}.")) && n.ends_with(".lst"))
        })
        .collect();
    paths.sort();
    let mut text = Vec::new();
    for p in paths {
        text.push(String::from_utf8_lossy(&fs::read(p).ok()?).into_owned());
    }
    let text = text.join("\n");
    if text.is_empty() {
        return None;
    }
    let mut found = BTreeSet::new();
    for line in py_lines(&text) {
        let path = line
            .split('=')
            .next()?
            .trim()
            .replace('\\', "/")
            .to_lowercase();
        let parts: Vec<_> = path.split('/').collect();
        if parts.len() >= 2
            && parts[parts.len() - 2] == subdir
            && names.contains(parts[parts.len() - 1])
        {
            found.insert(parts[parts.len() - 1].to_owned());
        }
    }
    Some(found)
}
pub fn staged_per_ramdisk(
    c: &Config,
    repo: &Path,
    arch: &str,
    subdir: &str,
    names: &BTreeSet<String>,
    media: Option<&str>,
) -> Option<BTreeSet<String>> {
    if !uses_ramdisk(arch, media) {
        return None;
    }
    let image = repo.join(&c.ramdisk_rel);
    if !image.is_file() {
        return None;
    }
    let mut cmd = Command::new("mdir");
    cmd.args(["-b", "-i"])
        .arg(image)
        .arg(format!("::/reactos/{subdir}"));
    let output = vibeos_build::process::capture(&mut cmd, 60).ok()?;
    Some(
        py_lines(&output.stdout)
            .iter()
            .map(|l| l.trim().rsplit('/').next().unwrap_or("").to_lowercase())
            .filter(|base| names.contains(base))
            .collect(),
    )
}
pub fn read_media(
    c: &Config,
    repo: &Path,
    arch: &str,
    subdir: &str,
    names: &BTreeSet<String>,
    media: Option<&str>,
) -> Option<BTreeSet<String>> {
    staged_per_manifest(repo, arch, subdir, names, media)
        .or_else(|| staged_per_ramdisk(c, repo, arch, subdir, names, media))
}
fn names(entries: &[(String, u64)]) -> BTreeSet<String> {
    entries
        .iter()
        .map(|(g, _)| format!("{g}.exe").to_lowercase())
        .collect()
}
fn guest_names(c: &Config) -> BTreeSet<String> {
    let mut n = names(&c.ladder);
    n.extend(names(&c.diagnostic));
    n
}
pub fn supports_wow64(c: &Config, repo: &Path, arch: &str, media: Option<&str>) -> Option<bool> {
    read_media(
        c,
        repo,
        arch,
        "system32",
        &["wow64cpu.dll".into(), "libwow64fex.dll".into()]
            .into_iter()
            .collect(),
        media,
    )
    .map(|s| !s.is_empty())
}
pub fn media_has_guests(c: &Config, repo: &Path, arch: &str, media: Option<&str>) -> bool {
    read_media(c, repo, arch, "syswow64", &guest_names(c), media).is_some_and(|s| !s.is_empty())
}
pub fn media_optional_staged(
    c: &Config,
    repo: &Path,
    arch: &str,
    media: Option<&str>,
) -> Vec<String> {
    let found =
        read_media(c, repo, arch, "syswow64", &names(&c.optional), media).unwrap_or_default();
    let mut staged: Vec<_> = c
        .optional
        .iter()
        .filter(|(g, _)| found.contains(&format!("{g}.exe").to_lowercase()))
        .map(|(g, _)| g.clone())
        .collect();
    staged.sort();
    let found =
        read_media(c, repo, arch, "system32", &names(&c.staged_ladder), media).unwrap_or_default();
    let mut x64: Vec<_> = c
        .staged_ladder
        .iter()
        .filter(|(g, _)| found.contains(&format!("{g}.exe").to_lowercase()))
        .map(|(g, _)| g.clone())
        .collect();
    x64.sort();
    staged.extend(x64);
    staged
}
pub fn media_missing_runtime(
    c: &Config,
    repo: &Path,
    arch: &str,
    media: Option<&str>,
) -> Vec<String> {
    let mut missing = Vec::new();
    let ntdll = ["ntdll.dll".into()].into_iter().collect();
    if read_media(c, repo, arch, "syswow64", &guest_names(c), media).is_some_and(|s| !s.is_empty())
        && !read_media(c, repo, arch, "syswow64", &ntdll, media).is_some_and(|s| !s.is_empty())
    {
        missing.push("syswow64\\ntdll.dll (build-i386)".into());
    }
    if uses_ramdisk(arch, media)
        && read_media(c, repo, arch, "system32", &names(&c.staged_ladder), media)
            .is_some_and(|s| !s.is_empty())
        && !read_media(c, repo, arch, "sysamd64", &ntdll, media).is_some_and(|s| !s.is_empty())
    {
        missing.push("sysamd64\\ntdll.dll (build-amd64)".into());
    }
    missing
}
pub fn newer_than(repo: &Path, guest: &str, exe: &Path, sources: &[String]) -> Result<Vec<String>> {
    let built = fs::metadata(exe)?.modified()?;
    let mut out = Vec::new();
    for rel in sources {
        let source = repo.join(rel);
        if !source.is_file() {
            out.push(format!("{guest} (source {rel} missing)"));
        } else if fs::metadata(source)?.modified()? > built {
            out.push(format!("{guest} ({rel} is newer)"));
        }
    }
    Ok(out)
}
pub fn stale_guests(c: &Config, repo: &Path) -> Result<Vec<String>> {
    let mut out = Vec::new();
    for (g, _) in c
        .ladder
        .iter()
        .chain(c.diagnostic.iter())
        .chain(c.staged_ladder.iter())
    {
        let exe = repo.join(format!("build-fex-hello/{g}.exe"));
        if !exe.is_file() {
            continue;
        }
        let Some(source) = c.guest_source.get(g) else {
            out.push(format!("{g} (no source mapping)"));
            continue;
        };
        let mut sources = vec![source.clone(), c.guest_builder.clone()];
        sources.extend(c.guest_helpers.get(g).cloned().unwrap_or_default());
        out.extend(newer_than(repo, g, &exe, &sources)?);
    }
    for (g, (exe, sources)) in &c.optional_source {
        let exe = repo.join(exe);
        if exe.is_file() {
            out.extend(newer_than(repo, g, &exe, sources)?);
        }
    }
    Ok(out)
}
pub fn x64_refresh_deferred(c: &Config, repo: &Path, todo: &[String]) -> Vec<String> {
    if repo.join(&c.guest_prereq_x64).is_file() {
        return Vec::new();
    }
    todo.iter()
        .filter(|t| {
            c.staged_ladder
                .iter()
                .any(|(g, _)| t.split(' ').next() == Some(g.as_str()))
        })
        .cloned()
        .collect()
}
pub fn guests_to_refresh(c: &Config, repo: &Path) -> Result<Vec<String>> {
    let stale = stale_guests(c, repo)?;
    let mut out = Vec::new();
    for (g, _) in c
        .ladder
        .iter()
        .chain(c.diagnostic.iter())
        .chain(c.staged_ladder.iter())
    {
        if c.ladder.iter().any(|(n, _)| n == g)
            && !repo.join(format!("build-fex-hello/{g}.exe")).is_file()
        {
            out.push(format!("{g} (absent)"));
        } else {
            out.extend(
                stale
                    .iter()
                    .filter(|s| s.starts_with(&format!("{g} (")))
                    .cloned(),
            );
        }
    }
    Ok(out)
}
pub fn refresh_blocker(c: &Config, repo: &Path) -> Option<String> {
    if !repo.join(&c.guest_toolchain).is_file() {
        return Some(format!("no i386 toolchain at {}", c.guest_toolchain));
    }
    if !repo.join(&c.guest_prereq).is_file() {
        return Some("no build-i386 importlibs to link the guests against — build i386 first: ./vibeos-cli build --arch i386".into());
    }
    None
}
pub fn gate_decision(
    c: &Config,
    repo: &Path,
    arch: &str,
    media: Option<&str>,
) -> Result<(Vec<String>, bool, String)> {
    if media_has_guests(c, repo, arch, media) {
        let stale = stale_guests(c, repo)?;
        if !stale.is_empty() {
            return Ok((Vec::new(),false,format!("staged guest binaries are older than their sources ({}) — the guests are NOT in git and a normal build does not rebuild them, so the ladder would grade stale code; rebuild with: {}",stale.join(", "),c.guest_builder)));
        }
        return Ok((vec!["--ladder".into()],true,format!("{} parity + {} diagnostic WoW64 guests staged — boot watched past the desktop marker",c.ladder.len(),c.diagnostic.len())));
    }
    match supports_wow64(c,repo,arch,media){None=>return Ok((Vec::new(),false,format!("no media to read for {arch} — neither a build-{arch}/ manifest nor a ramdisk this arch uses, so whether it can host 32-bit guests is UNKNOWN, not no; build the arch first: ./vibeos-cli build --arch {arch}"))),Some(false)=>return Ok((Vec::new(),false,"no WoW64 CPU layer on this arch's media — 32-bit guests cannot run here, so the ladder does not apply".into())),Some(true)=>{}}
    let mut built: Vec<_> = staged_guests(c)
        .into_iter()
        .filter(|g| repo.join(format!("build-fex-hello/{g}.exe")).is_file())
        .collect();
    built.sort();
    if !built.is_empty() && built.len() < c.ladder.len() {
        let absent = c
            .ladder
            .iter()
            .filter(|(g, _)| !built.contains(g))
            .map(|(g, _)| format!("{g}.exe"))
            .collect::<Vec<_>>()
            .join(", ");
        let fix = if uses_ramdisk(arch, media) {
            c.guest_builder.clone()
        } else {
            format!(
                "./vibeos-cli build --arch {arch} (it runs {})",
                c.guest_builder
            )
        };
        return Ok((Vec::new(),false,format!("{}/{} ladder guests built — {absent} missing, and staging is all-or-nothing, so NOTHING is staged; build the guests with: {fix}",built.len(),c.ladder.len())));
    }
    if !built.is_empty() {
        let fix = if uses_ramdisk(arch, media) {
            "VIBEOS_FEX=1 ./vibeos-cli build --arch arm64".into()
        } else {
            format!("./vibeos-cli build --arch {arch}")
        };
        return Ok((Vec::new(),false,format!("{}/{} guests are BUILT but none is staged on this arch's media — staging is snapshotted when the media is assembled; restage with: {fix}",built.len(),c.ladder.len())));
    }
    Ok((Vec::new(),false,format!("no staged syswow64/ guest in this arch's media manifest — WoW64 guests NOT checked by this gate{}",if uses_ramdisk(arch,media){""}else{" (the amd64 build's wow64-guests stage says why)"})))
}
pub fn dispatch(root: &Path, request: &Value) -> Result<Value> {
    let a = args(request);
    let op = string(request, "op")?;
    let config: Config = a
        .get("config")
        .cloned()
        .map(serde_json::from_value)
        .transpose()?
        .unwrap_or_default();
    let serial = || {
        a.get("serial")
            .or_else(|| a.get("text"))
            .or_else(|| a.get("seen"))
            .and_then(Value::as_str)
            .ok_or_else(|| Error::new("missing ladder serial"))
    };
    let media = a.get("media_rel").and_then(Value::as_str);
    let staged: Vec<String> = a
        .get("optional_staged")
        .filter(|v| !v.is_null())
        .cloned()
        .map(serde_json::from_value)
        .transpose()?
        .unwrap_or_default();
    Ok(match op {
        "ladder_constants" => constants(),
        "ladder_staged_guests" => json!(staged_guests(&config)),
        "ladder_markers" => json!(markers(&config)),
        "ladder_verdict_ok" => json!(verdict_ok(
            &config,
            string(a, "guest")?,
            a["got"].as_u64().ok_or("missing guest exit code")?,
            a["want"].as_u64().ok_or("missing expected exit code")?
        )),
        "ladder_verdicts" => json!(verdicts(serial()?)?),
        "ladder_winetests" => json!(winetests(serial()?)?),
        "ladder_winetests_crashed" => json!(winetests_crashed(serial()?)),
        "ladder_winetests_timed_out" => json!(winetests_timed_out(serial()?)),
        "ladder_winetests_lost" => json!(winetests_lost(serial()?)?),
        "ladder_sweeps_unfinished" => json!(sweeps_unfinished(serial()?)),
        "ladder_optional_in_flight" => json!(optional_in_flight(&config, serial()?)),
        "ladder_check" => check(&config, serial()?, &staged)?,
        "ladder_summary" => json!(summary(&config, serial()?)?),
        "ladder_diagnostic_tail" => json!(diagnostic_tail(
            &config,
            &serde_json::from_value(a["seen"].clone())?
        )),
        "ladder_watch_complete" => json!(watch_complete(
            &config,
            serial()?,
            &staged,
            a["program_done_expected"].as_bool().unwrap_or(false)
        )?),
        "ladder_watch_state" => {
            let text = serial()?;
            let mut pending = optional_in_flight(&config, text);
            for g in &staged {
                if !text.contains(&format!("{g}.exe exit = ")) && !pending.contains(g) {
                    pending.push(g.clone());
                }
            }
            json!({"complete":watch_complete(&config,text,&staged,a["program_done_expected"].as_bool().unwrap_or(false))?,"optional_in_flight":pending,"winetests_lost":winetests_lost(text)?,"sweeps_unfinished":sweeps_unfinished(text),"fatal":crate::verdict::post_boot_check(text,a["debugcon_raw"].as_str().unwrap_or(""),a.get("known_flakes").unwrap_or(&crate::verdict::known_flakes()))})
        }
        "ladder_drain_floor" => json!(drain_floor(a.get("raw").and_then(Value::as_str))?),
        "ladder_uses_ramdisk" => json!(uses_ramdisk(string(a, "arch")?, media)),
        "ladder_staged_per_manifest" => json!(staged_per_manifest(
            root,
            string(a, "arch")?,
            string(a, "subdir")?,
            &serde_json::from_value(a["names"].clone())?,
            media
        )),
        "ladder_staged_per_ramdisk" => json!(staged_per_ramdisk(
            &config,
            root,
            string(a, "arch")?,
            string(a, "subdir")?,
            &serde_json::from_value(a["names"].clone())?,
            media
        )),
        "ladder_read_media" => json!(read_media(
            &config,
            root,
            string(a, "arch")?,
            string(a, "subdir")?,
            &serde_json::from_value(a["names"].clone())?,
            media
        )),
        "ladder_supports_wow64" => json!(supports_wow64(&config, root, string(a, "arch")?, media)),
        "ladder_media_has_guests" => {
            json!(media_has_guests(&config, root, string(a, "arch")?, media))
        }
        "ladder_media_missing_runtime" => json!(media_missing_runtime(
            &config,
            root,
            string(a, "arch")?,
            media
        )),
        "ladder_media_optional_staged" => json!(media_optional_staged(
            &config,
            root,
            string(a, "arch")?,
            media
        )),
        "ladder_stale_guests" => json!(stale_guests(&config, root)?),
        "ladder_guests_to_refresh" => json!(guests_to_refresh(&config, root)?),
        "ladder_x64_refresh_deferred" => json!(x64_refresh_deferred(
            &config,
            root,
            &serde_json::from_value::<Vec<String>>(a["todo"].clone())?
        )),
        "ladder_refresh_blocker" => json!(refresh_blocker(&config, root)),
        "ladder_newer_than" => json!(newer_than(
            root,
            string(a, "guest")?,
            Path::new(string(a, "exe")?),
            &serde_json::from_value::<Vec<String>>(a["sources"].clone())?
        )?),
        "ladder_guest_names" => json!(guest_names(&config)),
        "ladder_gate_decision" => json!(gate_decision(&config, root, string(a, "arch")?, media)?),
        _ => return Err(Error::new(format!("unknown ladder operation {op}"))),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn staged_unreported_is_owed_and_no_early_finish() {
        let c = Config::default();
        let text = c
            .ladder
            .iter()
            .map(|(g, _)| format!("[WINLOGON] {g}.exe exit = 0x600d\n"))
            .collect::<String>();
        assert!(check(&c, &text, &[]).unwrap().is_null());
        assert!(!check(&c, &text, &["lua32".into()]).unwrap().is_null());
        assert!(!watch_complete(&c, &text, &["lua32".into()], false).unwrap());
        assert!(!watch_complete(&c, &text, &[], true).unwrap());
        assert!(watch_complete(&c, &format!("{text}{PROGRAM_DONE}"), &[], true).unwrap());
    }
    #[test]
    fn native_x86_never_borrows_arm64_media() {
        assert!(!uses_ramdisk("amd64", None));
        assert!(uses_ramdisk("arm64", None));
        assert!(!uses_ramdisk("unknown", None));
    }
    #[test]
    fn crashed_timed_out_and_unreported_winetests_are_distinct() {
        let s="[WT] native/user32:class: started \n[WT] native/user32:class: TIMED OUT,\n[WT] user32:other: started \n";
        assert_eq!(winetests_lost(s).unwrap(), vec!["user32:other"]);
        assert_eq!(winetests_timed_out(s), vec!["native/user32:class"]);
        assert!(winetests(s).unwrap().is_empty());
    }
}
