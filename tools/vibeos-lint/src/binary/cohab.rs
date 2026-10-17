use crate::binary::{BinaryContext, CheckOutput};
use regex::Regex;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

const FAMILY_NAME: &str = "msvcrt";
const MOD_RS: &str = "sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/mod.rs";
const RUST_DIR: &str = "sdk/lib/rust/msvcrt-crt-rust/src/msvcrt";
const OBJ_DIR: &str = "dll/win32/msvcrt/CMakeFiles/msvcrt_static.dir";
const OBJ_ROOT: &str = "dll/win32/msvcrt";
const SPEC: &str = "dll/win32/msvcrt/msvcrt.spec";
const ARCHIVE: &str = "dll/win32/msvcrt/libmsvcrt_crt_cargo.a";
const LINK_TARGET: &str = "msvcrt";
const STATIC_LIB: &str = "dll/win32/msvcrt/libmsvcrt_static.a";
const ARCHIVE_SOURCES: &[&str] = &[
    "sdk/lib/rust/msvcrt-crt-rust/src",
    "sdk/lib/rust/msvcrt-crt-cargo/src",
    "sdk/lib/rust/cnt-rust/src",
];
const NO_C_TWIN: &[(&str, &str)] = &[
    (
        "cpp_rtti",
        "the RTTI half of cpp.c; cpp.c.obj carries both halves' symbols",
    ),
    (
        "seh_i386",
        "i386 SEH substrate written directly in Rust, no C original",
    ),
];
const OBJECT_ALIASES: &[(&str, &str)] = &[
    ("reactos_misc", "reactos/misc.c.obj"),
    ("reactos_system", "reactos/system.cpp.obj"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct Symbol {
    kind: String,
    name: String,
}

fn is_global(kind: &str) -> bool {
    let has_case = kind.chars().any(char::is_alphabetic);
    has_case
        && kind
            .chars()
            .filter(|c| c.is_alphabetic())
            .all(char::is_uppercase)
}
fn spellings(name: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::from([
        name.to_owned(),
        name.split('@').next().unwrap_or(name).to_owned(),
    ]);
    let without = out.clone();
    if name.starts_with('_') {
        for value in without {
            if let Some(stripped) = value.strip_prefix('_') {
                out.insert(stripped.to_owned());
            }
        }
    }
    out
}
fn symbols_from_nm(output: &str) -> Vec<Symbol> {
    output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() || parts.last() == Some(&"@feat.00") {
                return None;
            }
            Some(Symbol {
                kind: if parts.len() > 1 {
                    parts[parts.len() - 2].to_owned()
                } else {
                    "?".to_owned()
                },
                name: parts[parts.len() - 1].to_owned(),
            })
        })
        .collect()
}
fn provider_symbols(symbols: &[Symbol]) -> BTreeSet<String> {
    let mut providers = BTreeSet::new();
    for symbol in symbols {
        if is_global(&symbol.kind) {
            providers.extend(spellings(&symbol.name));
        }
    }
    providers
}
fn parse_flipped_modules(text: &str) -> Vec<String> {
    let re = Regex::new(r"^\s*pub mod (\w+);").unwrap();
    text.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .filter_map(|line| re.captures(line).map(|m| m[1].to_owned()))
        .collect()
}
fn spec_references(text: &str) -> BTreeSet<String> {
    let export = Regex::new(r"(?m)^\s*(?:@|\d+)\s+\S+(?:\s+-\S+)*\s+(\w+)").unwrap();
    let extern_export =
        Regex::new(r"(?m)^\s*(?:@|\d+)\s+extern(?:\s+-\S+)*\s+\w+\s+(\w+)").unwrap();
    let mut out = BTreeSet::new();
    for cap in export.captures_iter(text) {
        out.insert(cap[1].to_owned());
        out.insert(format!("_{}", &cap[1]));
    }
    for cap in extern_export.captures_iter(text) {
        out.extend(spellings(&cap[1]));
    }
    out
}
fn missing_obligation(
    kind: &str,
    raw: &str,
    providers: &BTreeSet<String>,
    referenced: &BTreeSet<String>,
) -> bool {
    if !is_global(kind) {
        return false;
    }
    let names = spellings(raw);
    !names.iter().any(|name| providers.contains(name))
        && names.iter().any(|name| referenced.contains(name))
}
fn empty_provider_refusal(arch: &str) -> String {
    format!("verify_cohab_coverage: nothing links ahead of {OBJ_DIR} in build-{arch} — with no Rust archive and no module objects every symbol would look missing, which is a false verdict, not a finding.\n  fix: ./vibeos-cli build --arch {arch}")
}
fn tree_rs_files(root: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    for item in fs::read_dir(root).map_err(|e| format!("{}: {e}", root.display()))? {
        let item = item.map_err(|e| e.to_string())?;
        let path = item.path();
        let ty = item.file_type().map_err(|e| e.to_string())?;
        if ty.is_dir() {
            tree_rs_files(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    Ok(())
}
fn relative_display(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}
fn refusal_error(message: &str) -> bool {
    [
        "llvm-nm not found at ",
        " missing — cannot tell what the Rust side defines.",
        " is OLDER than ",
        "could not ask ninja for ",
        "`ninja -t commands ",
        " missing — generated objects must come from THIS worktree",
        "nothing links ahead of ",
    ]
    .iter()
    .any(|marker| message.contains(marker))
}
fn refusal_output(message: String) -> CheckOutput {
    CheckOutput {
        stdout: String::new(),
        stderr: format!("{message}\n"),
        returncode: 1,
    }
}
fn require_fresh(ctx: &BinaryContext, build: &Path) -> Result<(), String> {
    let archive = build.join(ARCHIVE);
    if !archive.is_file() {
        return Err(format!("verify_cohab_coverage: {} missing — cannot tell what the Rust side defines.\n  fix: ./vibeos-cli build --arch {}", archive.display(), ctx.arch));
    }
    let archive_mtime = archive
        .metadata()
        .map_err(|e| format!("{}: {e}", archive.display()))?
        .modified()
        .map_err(|e| e.to_string())?;
    let mut newest: Option<(SystemTime, PathBuf)> = None;
    for rel in ARCHIVE_SOURCES {
        let source_root = ctx.root.join(rel);
        if !source_root.is_dir() {
            continue;
        }
        let mut files = Vec::new();
        tree_rs_files(&source_root, &mut files)?;
        for file in files {
            let mtime = file
                .metadata()
                .map_err(|e| format!("{}: {e}", file.display()))?
                .modified()
                .map_err(|e| e.to_string())?;
            if newest.as_ref().is_none_or(|(old, _)| mtime > *old) {
                newest = Some((mtime, file));
            }
        }
    }
    if let Some((mtime, path)) = newest {
        if mtime > archive_mtime {
            return Err(format!("verify_cohab_coverage: {} for {} is OLDER than {} — every finding would be staleness, not a missing symbol.\n  fix: ./vibeos-cli build --arch {}", archive.file_name().unwrap_or_default().to_string_lossy(), ctx.arch, relative_display(&ctx.root, &path), ctx.arch));
        }
    }
    Ok(())
}
fn ninja_last_command(ctx: &BinaryContext, build: &Path, target: &str) -> Result<String, String> {
    let script = ctx.root.join("tools/canon-run.sh");
    let mut child = Command::new("bash")
        .arg(script)
        .arg("ninja")
        .arg("-C")
        .arg(build)
        .args(["-t", "commands", target])
        .current_dir(&ctx.root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("verify_cohab_coverage: could not ask ninja for {target}: {e}"))?;
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let out_thread = thread::spawn(move || {
        let mut b = Vec::new();
        std::io::Read::read_to_end(&mut stdout, &mut b).map(|_| b)
    });
    let err_thread = thread::spawn(move || {
        let mut b = Vec::new();
        std::io::Read::read_to_end(&mut stderr, &mut b).map(|_| b)
    });
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if start.elapsed() <= Duration::from_secs(300) => {
                thread::sleep(Duration::from_millis(20))
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("verify_cohab_coverage: could not ask ninja for {target}: timed out after 300 seconds"));
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "verify_cohab_coverage: could not ask ninja for {target}: {e}"
                ));
            }
        }
    };
    let stdout = out_thread
        .join()
        .map_err(|_| "ninja stdout reader panicked")?
        .map_err(|e| e.to_string())?;
    let stderr = err_thread
        .join()
        .map_err(|_| "ninja stderr reader panicked")?
        .map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&stdout);
    let stderr = String::from_utf8_lossy(&stderr);
    let lines: Vec<&str> = stdout.lines().filter(|ln| !ln.trim().is_empty()).collect();
    if !status.success() || lines.is_empty() {
        return Err(format!("verify_cohab_coverage: `ninja -t commands {target}` failed in {} (exit {}): {}\n  fix: ./vibeos-cli build --arch <arch>", build.display(), status.code().map_or_else(|| "signal".to_owned(), |c| c.to_string()), stderr.trim().chars().take(1000).collect::<String>()));
    }
    Ok(lines.last().unwrap().to_string())
}
fn command_objects(command: &str, build: &Path) -> Vec<PathBuf> {
    command
        .split_whitespace()
        .filter(|t| t.ends_with(".obj"))
        .map(|t| build.join(t))
        .collect()
}
fn link_inputs(ctx: &BinaryContext, build: &Path) -> Result<(Vec<PathBuf>, Vec<PathBuf>), String> {
    let direct = command_objects(&ninja_last_command(ctx, build, LINK_TARGET)?, build);
    let members = command_objects(&ninja_last_command(ctx, build, STATIC_LIB)?, build);
    Ok((direct, members))
}
fn nm_symbols(
    ctx: &BinaryContext,
    nm: &str,
    obj: &Path,
    mode: &str,
) -> Result<Vec<Symbol>, String> {
    let args = vec![format!("--{mode}-only"), obj.to_string_lossy().into_owned()];
    let output = ctx.command(nm, &args).map_err(|e| {
        format!(
            "verify_cohab_coverage: llvm-nm failed for {}: {e}",
            obj.display()
        )
    })?;
    Ok(symbols_from_nm(&output))
}
fn providers(
    ctx: &BinaryContext,
    nm: &str,
    build: &Path,
    direct: &[PathBuf],
) -> Result<BTreeSet<String>, String> {
    let archive = build.join(ARCHIVE);
    let mut sources = Vec::new();
    if archive.is_file() {
        sources.push(archive);
    }
    sources.extend_from_slice(direct);
    let mut found = Vec::new();
    for source in sources {
        found.extend(nm_symbols(ctx, nm, &source, "defined")?);
    }
    Ok(provider_symbols(&found))
}
fn referenced(
    ctx: &BinaryContext,
    nm: &str,
    objects: &[PathBuf],
) -> Result<BTreeSet<String>, String> {
    let mut result = BTreeSet::new();
    for obj in objects {
        for symbol in nm_symbols(ctx, nm, obj, "undefined")? {
            result.extend(spellings(&symbol.name));
        }
    }
    let spec = ctx.root.join(SPEC);
    if spec.is_file() {
        let text = fs::read_to_string(&spec).map_err(|e| format!("{}: {e}", spec.display()))?;
        result.extend(spec_references(&text));
    }
    Ok(result)
}
fn object_relative(module: &str) -> &str {
    OBJECT_ALIASES
        .iter()
        .find(|(name, _)| *name == module)
        .map(|(_, object)| *object)
        .unwrap_or("")
}
fn missing_object_finding(
    ctx: &BinaryContext,
    module: &str,
    obj_rel: &str,
    live_members: &BTreeSet<PathBuf>,
    obj_dir: &Path,
) -> Option<String> {
    let obj = obj_dir.join(obj_rel);
    if live_members.contains(&obj) {
        return None;
    }
    let src_rel = obj_rel.strip_suffix(".obj").unwrap_or(obj_rel);
    if ctx.root.join(OBJ_ROOT).join(src_rel).is_file() {
        return None;
    }
    Some(format!("{RUST_DIR}/{module}.rs: [{}] flipped, but no C object and no C source at {OBJ_ROOT}/{src_rel} — this module is being checked against nothing. Add it to the family's `objects` map if its twin is named differently, or to `no_c_twin` if it has none", ctx.arch))
}
fn check_family(ctx: &BinaryContext, nm: &str, build: &Path) -> Result<Vec<String>, String> {
    let obj_dir = build.join(OBJ_DIR);
    if !obj_dir.is_dir() {
        return Err(format!("verify_cohab_coverage: {} missing — generated objects must come from THIS worktree, never a sibling's.\n  fix: ./vibeos-cli build --arch {}", obj_dir.display(), ctx.arch));
    }
    require_fresh(ctx, build)?;
    let (direct, members) = link_inputs(ctx, build)?;
    let live_members: BTreeSet<PathBuf> = members.iter().cloned().collect();
    let defined_before = providers(ctx, nm, build, &direct)?;
    if defined_before.is_empty() {
        return Err(empty_provider_refusal(&ctx.arch));
    }
    let mut all_objects = direct;
    all_objects.extend(members);
    let needed = referenced(ctx, nm, &all_objects)?;
    let mod_rs = ctx.root.join(MOD_RS);
    let modules = parse_flipped_modules(
        &fs::read_to_string(&mod_rs).map_err(|e| format!("{}: {e}", mod_rs.display()))?,
    );
    let mut findings = Vec::new();
    for module in modules {
        if NO_C_TWIN.iter().any(|(name, _)| *name == module) {
            continue;
        }
        let alias = object_relative(&module);
        let obj_rel = if alias.is_empty() {
            format!("{module}.c.obj")
        } else {
            alias.to_owned()
        };
        let obj = obj_dir.join(&obj_rel);
        if !live_members.contains(&obj) {
            if let Some(finding) =
                missing_object_finding(ctx, &module, &obj_rel, &live_members, &obj_dir)
            {
                findings.push(finding);
            }
            continue;
        }
        for symbol in nm_symbols(ctx, nm, &obj, "defined")? {
            if !missing_obligation(&symbol.kind, &symbol.name, &defined_before, &needed) {
                continue;
            }
            findings.push(format!("{RUST_DIR}/{module}.rs: [{}] nothing that links ahead of {} defines `{}`, which {} exports and something references — the C object gets pulled back in and duplicates every other symbol in this file", ctx.arch, Path::new(OBJ_DIR).file_name().unwrap_or_default().to_string_lossy(), symbol.name, obj_rel));
        }
    }
    Ok(findings)
}

pub fn run(ctx: &BinaryContext) -> Result<CheckOutput, String> {
    let build = ctx.root.join(format!("build-{}", ctx.arch));
    let obj_dir = build.join(OBJ_DIR);
    let nm_path = ctx.root.join(".build-tools/llvm-mingw/bin/llvm-nm");
    if !nm_path.is_file() {
        return Ok(refusal_output(format!(
            "verify_cohab_coverage: llvm-nm not found at {}",
            nm_path.display()
        )));
    }
    let nm = nm_path.to_string_lossy().into_owned();
    let mut findings = Vec::new();
    let mut pairs = 0usize;
    if obj_dir.is_dir() {
        pairs = 1;
        match check_family(ctx, &nm, &build) {
            Ok(found) => findings = found,
            Err(error) if refusal_error(&error) => return Ok(refusal_output(error)),
            Err(error) => return Err(error),
        }
    }
    let mut stdout = String::new();
    for finding in &findings {
        stdout.push_str(finding);
        stdout.push('\n');
    }
    let mut stderr = format!("verify_cohab_coverage: {} finding(s); {} family/arch pair(s) compared against their C objects\n", findings.len(), pairs);
    if pairs == 0 {
        stderr.push_str("verify_cohab_coverage: --check with 0 pairs compared — the build layout moved or no arch was built, so this run proves nothing.\n  fix: ./vibeos-cli build --arch i386\n");
        return Ok(CheckOutput {
            stdout,
            stderr,
            returncode: 1,
        });
    }
    let _ = FAMILY_NAME; // retained as named family metadata for report/debug contexts
    Ok(CheckOutput {
        stdout,
        stderr,
        returncode: if findings.is_empty() { 0 } else { 1 },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_root(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("cohab-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }
    fn fixture_ctx(root: &Path) -> BinaryContext {
        BinaryContext::new(root.to_path_buf(), root.join("build-i386"), "i386".into())
    }
    fn named_symbols(defs: &[(&str, &str)]) -> Vec<Symbol> {
        defs.iter()
            .map(|(kind, name)| Symbol {
                kind: (*kind).into(),
                name: (*name).into(),
            })
            .collect()
    }

    #[test]
    fn a_symbol_the_c_exports_and_the_rust_side_does_not_is_a_finding() {
        let providers = provider_symbols(&named_symbols(&[("T", "_msvcrt_rust_baseline")]));
        let referenced = spec_references("@ cdecl _MSVCRT__rotl64(long long long)\n");
        assert!(missing_obligation(
            "T",
            "_MSVCRT__rotl64",
            &providers,
            &referenced
        ));
        assert!(missing_obligation(
            "T",
            "_invoke_matherr",
            &BTreeSet::new(),
            &spellings("_invoke_matherr")
        ));
    }

    #[test]
    fn matcher_is_alive_across_every_provider_category() {
        let refs = spellings("_MSVCRT__rotl64");
        let archive_provider = provider_symbols(&named_symbols(&[("T", "_MSVCRT__rotl64")]));
        let direct_object_provider = provider_symbols(&named_symbols(&[("T", "_MSVCRT__rotl64")]));
        for provider in [&archive_provider, &direct_object_provider] {
            assert!(!missing_obligation(
                "T",
                "_MSVCRT__rotl64",
                &provider,
                &refs
            ));
        }
    }

    #[test]
    fn decoration_is_not_a_mismatch() {
        let provider = provider_symbols(&named_symbols(&[("T", "_adj_fdiv_m16i")]));
        let referenced = spellings("__adj_fdiv_m16i@4");
        assert!(!missing_obligation(
            "T",
            "__adj_fdiv_m16i@4",
            &provider,
            &referenced
        ));
        assert!(spellings("__adj_fdiv_m16i@4").contains("_adj_fdiv_m16i"));
    }

    #[test]
    fn defined_but_dead_is_not_a_finding() {
        let empty = BTreeSet::new();
        let dead = spellings("_asm_sqrt");
        assert!(!missing_obligation(
            "T",
            "_asm_sqrt",
            &empty,
            &BTreeSet::new()
        ));
        assert!(missing_obligation("T", "_asm_sqrt", &empty, &dead));
    }

    #[test]
    fn file_local_symbols_cannot_pull_the_object_in() {
        let empty = BTreeSet::new();
        let refs = spellings("_helper");
        assert!(!missing_obligation("t", "_helper", &empty, &refs));
        assert!(!missing_obligation(
            "d",
            "_table",
            &empty,
            &spellings("_table")
        ));
        assert!(missing_obligation("T", "_helper", &empty, &refs));
        assert!(missing_obligation(
            "D",
            "_table",
            &empty,
            &spellings("_table")
        ));
    }

    #[test]
    fn a_module_that_is_not_flipped_is_not_checked() {
        let flipped = parse_flipped_modules("pub mod math;\n// pub mod notflipped;\n");
        assert_eq!(flipped, vec!["math"]);
        assert!(!flipped.contains(&"notflipped".to_owned()));
        assert!(missing_obligation(
            "T",
            "_gone",
            &BTreeSet::new(),
            &spellings("_gone")
        ));
    }

    #[test]
    fn a_stale_object_on_disk_is_not_a_provider() {
        let root = fixture_root("stale-direct");
        let build = root.join("build-i386");
        let stale = build.join("dll/win32/msvcrt/CMakeFiles/msvcrt.dir/stale_old_shim.c.obj");
        fs::create_dir_all(stale.parent().unwrap()).unwrap();
        fs::write(&stale, "stale").unwrap();
        let live = command_objects("link.exe live_shim.c.obj", &build);
        assert!(!live.contains(&stale));
        let providers = provider_symbols(&named_symbols(&[("T", "_live_baseline")]));
        let refs = spellings("_MSVCRT__rotl64");
        assert!(missing_obligation(
            "T",
            "_MSVCRT__rotl64",
            &providers,
            &refs
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_stale_twin_that_links_into_nothing_is_not_checked() {
        let root = fixture_root("stale-twin");
        let ctx = fixture_ctx(&root);
        let obj_dir = ctx.build_dir.join(OBJ_DIR);
        let obj = obj_dir.join("math.c.obj");
        fs::create_dir_all(obj.parent().unwrap()).unwrap();
        fs::write(&obj, "stale twin").unwrap();
        let source = root.join(OBJ_ROOT).join("math.c");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, "/* live C source, omitted on this link */").unwrap();
        assert!(
            missing_object_finding(&ctx, "math", "math.c.obj", &BTreeSet::new(), &obj_dir)
                .is_none()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_stale_archive_is_refused_not_reported() {
        let root = fixture_root("stale-archive");
        let build = root.join("build-i386");
        let archive = build.join(ARCHIVE);
        fs::create_dir_all(archive.parent().unwrap()).unwrap();
        fs::write(&archive, "archive").unwrap();
        let source = root.join(ARCHIVE_SOURCES[0]).join("msvcrt/math.rs");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, "// changed").unwrap();
        let later = SystemTime::now() + Duration::from_secs(10);
        fs::File::options()
            .write(true)
            .open(&source)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(later))
            .unwrap();
        let ctx = fixture_ctx(&root);
        assert!(require_fresh(&ctx, &build)
            .unwrap_err()
            .contains("OLDER than"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn nothing_linking_ahead_is_a_refusal_not_a_clean_sweep() {
        assert!(empty_provider_refusal("i386").contains("nothing links ahead"));
        let providers = provider_symbols(&[]);
        assert!(providers.is_empty());
    }

    #[test]
    fn check_over_zero_pairs_is_not_green() {
        let root = fixture_root("zero-pairs");
        let nm = root.join(".build-tools/llvm-mingw/bin/llvm-nm");
        fs::create_dir_all(nm.parent().unwrap()).unwrap();
        fs::write(&nm, "fixture").unwrap();
        let ctx = fixture_ctx(&root);
        let result = run(&ctx).unwrap();
        assert_eq!(result.returncode, 1);
        assert!(result.stderr.contains("0 pairs compared"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_module_named_differently_from_its_c_file_is_still_checked() {
        let root = fixture_root("mapped-twin");
        let ctx = fixture_ctx(&root);
        let obj_dir = ctx.build_dir.join(OBJ_DIR);
        let mapped = object_relative("reactos_misc");
        assert_eq!(mapped, "reactos/misc.c.obj");
        assert_eq!(object_relative("reactos_system"), "reactos/system.cpp.obj");
        let c_object = obj_dir.join(mapped);
        let refs = spellings("__mbcasemap");
        assert!(missing_obligation(
            "D",
            "__mbcasemap",
            &BTreeSet::new(),
            &refs
        ));
        assert!(format!("{RUST_DIR}/reactos_misc.rs").ends_with("reactos_misc.rs"));
        assert!(c_object.ends_with("reactos/misc.c.obj"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_module_whose_c_file_this_arch_does_not_compile_is_not_a_finding() {
        let root = fixture_root("arch-excluded");
        let ctx = fixture_ctx(&root);
        let source = root.join(OBJ_ROOT).join("except_i386.c");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, "/* only compiled on i386 */").unwrap();
        assert!(missing_object_finding(
            &ctx,
            "except_i386",
            "except_i386.c.obj",
            &BTreeSet::new(),
            &ctx.build_dir.join(OBJ_DIR)
        )
        .is_none());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_module_matching_no_c_file_anywhere_is_a_finding() {
        let root = fixture_root("missing-twin");
        let ctx = fixture_ctx(&root);
        let finding = missing_object_finding(
            &ctx,
            "raectos_misc",
            "raectos_misc.c.obj",
            &BTreeSet::new(),
            &ctx.build_dir.join(OBJ_DIR),
        )
        .unwrap();
        assert!(finding.contains("checked against nothing"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_declared_rust_only_module_is_not_a_finding() {
        for module in ["cpp_rtti", "seh_i386"] {
            assert!(NO_C_TWIN
                .iter()
                .any(|(name, reason)| *name == module && !reason.is_empty()));
        }
    }

    #[test]
    fn spec_exports_and_extern_backing_symbols_are_referenced() {
        let refs =
            spec_references("@ cdecl _rotl64(long long long)\n@ extern _mbctype MSVCRT_mbctype\n");
        assert!(refs.contains("_rotl64") && refs.contains("__rotl64"));
        assert!(refs.contains("MSVCRT_mbctype"));
    }

    #[test]
    fn nm_parsing_ignores_pe_feature_marker_and_retains_symbol_codes() {
        assert_eq!(
            symbols_from_nm("00000000 T _foo\n00000001 t local\n00000000 A @feat.00\n"),
            vec![
                Symbol {
                    kind: "T".into(),
                    name: "_foo".into()
                },
                Symbol {
                    kind: "t".into(),
                    name: "local".into()
                }
            ]
        );
    }
}
