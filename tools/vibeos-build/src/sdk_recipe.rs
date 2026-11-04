//! Capture and replay cmake's generated-SDK and host-tool Ninja recipes.
use crate::{arch, local, read, Error, Result, ARCHES};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, HashSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};
pub const GENERATORS: &[&str] = &["widl", "hpp", "utf16le", "wmc", "bin2c", "txt2nls"];
pub const HOST_TOOLS: &[&str] = GENERATORS;
pub const MEDIA_TOOLS: &[&str] = &["cabman", "isohybrid", "mkisofs"];
pub const CANON: &str = "/tmp/vibeos";
fn re(s: &str) -> Regex {
    Regex::new(s).expect("constant SDK recipe regex")
}
pub fn recipe_path(root: &Path, arch_name: &str, host: bool) -> PathBuf {
    root.join(format!(
        "sdk/lib/rust/generated/{arch_name}{}.cmds",
        if host { ".host" } else { "" }
    ))
}
/// Invoke the project's canonical Ninja entrypoint in this selected checkout.
pub fn ninja_commands(root: &Path, build: &Path, targets: &[String]) -> Result<Vec<String>> {
    let script = local(root, &root.join("tools/canon-run.sh"))?;
    let build = local(root, build)?;
    let o = Command::new("bash")
        .arg(script)
        .arg("ninja")
        .arg("-C")
        .arg(&build)
        .args(["-t", "commands"])
        .args(targets)
        .output()
        .map_err(|e| Error::new(format!("ninja -t commands in {}: {e}", build.display())))?;
    if !o.status.success() {
        let err = String::from_utf8_lossy(&o.stderr);
        let tail = err
            .chars()
            .rev()
            .take(400)
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>();
        return Err(Error::new(format!(
            "ninja -t commands in {}: {}",
            build.display(),
            tail.trim()
        )));
    }
    Ok(String::from_utf8(o.stdout)
        .map_err(|e| Error::new(format!("ninja output is not UTF-8: {e}")))?
        .lines()
        .map(str::to_owned)
        .collect())
}
pub fn tool(line: &str) -> Result<String> {
    let body = if line.starts_with("cd ") {
        line.split_once(" && ")
            .map(|(_, b)| b)
            .ok_or_else(|| Error::new("malformed cd command: missing &&"))?
    } else {
        line
    };
    if body.trim().is_empty() {
        return Ok(String::new());
    }
    let args =
        shlex::split(body).ok_or_else(|| Error::new(format!("malformed shell quoting: {body}")))?;
    let Some(first) = args.first() else {
        return Ok(String::new());
    };
    if first == "/usr/bin/cmake" {
        let cmd = body
            .split_whitespace()
            .nth(2)
            .ok_or_else(|| Error::new(format!("malformed cmake command: {body}")))?;
        return Ok(format!("cmake -E {cmd}"));
    }
    Ok(Path::new(first)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned())
}
pub fn keep(line: &str) -> Result<bool> {
    if line.contains("host-jobs") || line.contains("ExternalProject") {
        return Ok(false);
    }
    let t = tool(line)?;
    Ok(t.ends_with("-windmc")
        || (t.ends_with("-windres") && re(r" -O coff -i \S+\.res ").is_match(line))
        || (t.ends_with("gcc") && line.contains(" -x assembler-with-cpp "))
        || GENERATORS.contains(&t.as_str())
        || [
            "flex",
            "bison",
            "cmake -E copy",
            "cmake -E copy_if_different",
            "cmake -E touch",
            "cmake -E make_directory",
        ]
        .contains(&t.as_str()))
}
pub fn unwrap(line: &str) -> String {
    re(r#""?\S*/\.cache/host-jobs/host-jobs"? --kind \w+ -- "#)
        .replace_all(line, "")
        .into_owned()
}
pub fn cell_deps(ninja: &str) -> Vec<String> {
    let re = re(
        r"\nbuild ([^\n]*?): CUSTOM_COMMAND([^\n]*)\n  COMMAND = [^\n]*\n  DESC = Cargo-linking",
    );
    let mut out = Vec::new();
    for c in re.captures_iter(ninja) {
        for x in c[2].replace("||", " ").replace('|', " ").split_whitespace() {
            if !x.starts_with('/') && !x.contains("rust") && !out.iter().any(|s| s == x) {
                out.push(x.to_owned())
            }
        }
    }
    out
}
pub fn encode(line: &str, a: &str) -> String {
    let build = format!("{CANON}/build-{a}");
    line.replace(&format!("{build}/host-tools/bin"), "{TOOLS}")
        .replace(&build, "{BUILD}")
        .replace(CANON, "{REPO}")
}
/// Longest contiguous run present in the requested share; ties use the first longest input.
pub fn majority_run(lists: &[Vec<String>], share: f64) -> Vec<String> {
    let Some(first) = lists
        .iter()
        .reduce(|a, b| if b.len() > a.len() { b } else { a })
    else {
        return vec![];
    };
    let mut best = Vec::new();
    for start in 0..first.len() {
        let mut end = start + best.len() + 1;
        if end > first.len() {
            continue;
        }
        let count = |end: usize| {
            lists
                .iter()
                .filter(|xs| xs.windows(end - start).any(|w| w == &first[start..end]))
                .count() as f64
        };
        if count(end) < share * lists.len() as f64 {
            continue;
        }
        while end < first.len() && count(end + 1) >= share * lists.len() as f64 {
            end += 1
        }
        best = first[start..end].to_vec();
    }
    best
}
pub fn factor(lines: &[String]) -> Result<Vec<String>> {
    let mut widl = Vec::new();
    for ln in lines {
        if tool(ln)? == "widl" {
            widl.push(ln.split(' ').map(str::to_owned).collect())
        }
    }
    let run = majority_run(&widl, 0.9);
    if run.len() < 4 {
        return Ok(lines.to_vec());
    }
    let mut out = vec![format!("{{WIDL}} = {}", run.join(" "))];
    for ln in lines {
        let mut toks: Vec<_> = ln.split(' ').map(str::to_owned).collect();
        if toks.len() >= run.len() {
            if let Some(i) = toks.windows(run.len()).position(|w| w == run) {
                toks.splice(i..i + run.len(), ["{WIDL}".to_owned()]);
            }
        }
        out.push(toks.join(" "));
    }
    Ok(out)
}
/// Derive in-memory recipes from the current selected build's Ninja graph.
pub fn derive(root: &Path, a: &str) -> Result<(Vec<String>, Vec<String>)> {
    arch(a)?;
    let build = local(root, &root.join(format!("build-{a}")))?;
    let ninja = read(&local(root, &build.join("build.ninja"))?)?;
    let mut targets = vec!["sdk/include/psdk/psdk".into(), "sdk/include/xdk/xdk".into()];
    targets.extend(cell_deps(&ninja));
    let (mut sdk, mut seen) = (Vec::new(), HashSet::new());
    for line in ninja_commands(root, &build, &targets)? {
        if keep(&line)? && seen.insert(line.clone()) {
            sdk.push(encode(&line, a));
        }
    }
    let sdk = factor(&sdk)?;
    let used_re = re(r"\{TOOLS\}/(\w+)");
    let used: BTreeSet<_> = sdk
        .iter()
        .flat_map(|l| used_re.captures_iter(l).map(|c| c[1].to_owned()))
        .collect();
    let unknown: Vec<_> = used
        .iter()
        .filter(|s| !HOST_TOOLS.contains(&s.as_str()))
        .cloned()
        .collect();
    if !unknown.is_empty() {
        return Err(Error::new(format!(
            "build-{a}: generator commands run host tools not recorded: {{{}}}",
            unknown
                .iter()
                .map(|s| format!("'{s}'"))
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }
    let mut host_targets: Vec<_> = used.into_iter().collect();
    host_targets.extend(MEDIA_TOOLS.iter().map(|s| s.to_string()));
    let host = ninja_commands(root, &build.join("host-tools/bin"), &host_targets)?
        .into_iter()
        .map(|s| {
            unwrap(&s)
                .replace(&format!("{CANON}/build-{a}/host-tools/bin"), "{TOOLS}")
                .replace(CANON, "{REPO}")
        })
        .collect();
    Ok((sdk, host))
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecipeOutput {
    pub arch: String,
    pub host: bool,
    pub path: PathBuf,
    pub text: String,
    pub commands: usize,
}
/// Fully derive every selected architecture before callers write any files.
pub fn write_plan(root: &Path, arches: &[String]) -> Result<Vec<RecipeOutput>> {
    let mut out = Vec::new();
    for a in arches {
        let (sdk, host) = derive(root, a)?;
        for (host, lines) in [(false, sdk), (true, host)] {
            out.push(RecipeOutput {
                arch: a.clone(),
                host,
                path: recipe_path(root, a, host),
                text: lines.join("\n") + "\n",
                commands: lines.len(),
            });
        }
    }
    Ok(out)
}
pub fn check(root: &Path, arches: &[String]) -> Result<Vec<PathBuf>> {
    let plans = write_plan(root, arches)?;
    let mut stale = Vec::new();
    for p in plans {
        match fs::read_to_string(&p.path) {
            Ok(existing) if existing == p.text => {}
            Ok(_) => stale.push(p.path),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => stale.push(p.path),
            Err(e) => return Err(Error::new(format!("{}: {e}", p.path.display()))),
        }
    }
    Ok(stale)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayCommand {
    pub recipe: PathBuf,
    pub line: String,
    pub cwd: PathBuf,
    pub directories: Vec<PathBuf>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayPlan {
    pub root: PathBuf,
    pub arch: String,
    pub gen: PathBuf,
    pub commands: Vec<ReplayCommand>,
}
/// Read both recipes and validate every command before executing any of them.
pub fn replay_plan(root: &Path, a: &str, gen: &Path) -> Result<ReplayPlan> {
    arch(a)?;
    let root = root
        .canonicalize()
        .map_err(|e| Error::new(format!("{}: {e}", root.display())))?;
    let gen = if gen.is_absolute() {
        gen.to_owned()
    } else {
        std::env::current_dir()?.join(gen)
    };
    let tools = gen.join("host-tools/bin");
    let mut commands = Vec::new();
    for host in [true, false] {
        let path = recipe_path(&root, a, host);
        let path=local(&root,&path).map_err(|e|Error::new(format!("{} missing or outside selected checkout: python3 tools/sdk_recipe.py --write ({e})",path.display())))?;
        let source = read(&path)?;
        let mut lines = source.lines();
        let first = lines.next();
        let (widl, rest) = match first {
            Some(l) if l.starts_with("{WIDL} = ") => (&l[9..], lines.collect::<Vec<_>>()),
            Some(l) => ("", std::iter::once(l).chain(lines).collect()),
            None => ("", vec![]),
        };
        let cwd = if host { tools.clone() } else { gen.clone() };
        let genstr = gen.to_string_lossy();
        let pat = Regex::new(&format!(r"(?:^cd |[ =])({}/\S*)", regex::escape(&genstr)))?;
        for line in rest {
            let expanded = line
                .replace("{WIDL}", widl)
                .replace("{TOOLS}", &tools.to_string_lossy())
                .replace("{BUILD}", &genstr)
                .replace("{REPO}", &root.to_string_lossy());
            let mut dirs = Vec::new();
            for c in pat.captures_iter(&expanded) {
                let d = &c[1];
                let p = PathBuf::from(d);
                let mkdir = if expanded.starts_with(&format!("cd {d} ")) {
                    p
                } else {
                    p.parent().unwrap_or(Path::new("")).to_owned()
                };
                if !dirs.contains(&mkdir) {
                    dirs.push(mkdir);
                }
            }
            let toks = shlex::split(&expanded.replace("&&", " ")).ok_or_else(|| {
                Error::new(format!(
                    "{}: invalid shell quoting: {expanded}",
                    path.display()
                ))
            })?;
            for tok in toks {
                let rel = if tok.starts_with('-') {
                    tok.split_once('=').map_or(tok.as_str(), |(_, v)| v)
                } else {
                    tok.as_str()
                };
                if rel.contains('/') && !rel.starts_with(['/', '-', '$']) {
                    let p = cwd.join(rel);
                    let parent = p.parent().unwrap_or(&cwd).to_owned();
                    if !dirs.contains(&parent) {
                        dirs.push(parent)
                    }
                }
            }
            if !dirs.contains(&cwd) {
                dirs.push(cwd.clone())
            }
            commands.push(ReplayCommand {
                recipe: path.clone(),
                line: expanded,
                cwd: cwd.clone(),
                directories: dirs,
            });
        }
    }
    Ok(ReplayPlan {
        root,
        arch: a.into(),
        gen,
        commands,
    })
}
/// Execute exactly the prevalidated plan in recipe order, stopping on first failure.
pub fn run(plan: &ReplayPlan) -> Result<()> {
    for cmd in &plan.commands {
        for dir in &cmd.directories {
            fs::create_dir_all(dir).map_err(|e| Error::new(format!("{}: {e}", dir.display())))?;
        }
        let o = Command::new("bash")
            .args(["-c", &cmd.line])
            .current_dir(&cmd.cwd)
            .output()
            .map_err(|e| Error::new(format!("{}: {e}", cmd.cwd.display())))?;
        if !o.status.success() {
            let err = String::from_utf8_lossy(&o.stderr);
            let tail = err
                .chars()
                .rev()
                .take(800)
                .collect::<String>()
                .chars()
                .rev()
                .collect::<String>();
            return Err(Error::new(format!(
                "{}: failed: {}\n{tail}",
                cmd.recipe.file_name().unwrap_or_default().to_string_lossy(),
                cmd.line.chars().take(300).collect::<String>()
            )));
        }
    }
    Ok(())
}
pub fn selected_arches(arches: &[String]) -> Result<Vec<String>> {
    if arches.is_empty() {
        return Ok(ARCHES.iter().map(|s| s.to_string()).collect());
    }
    for a in arches {
        arch(a)?;
    }
    Ok(arches.to_vec())
}
