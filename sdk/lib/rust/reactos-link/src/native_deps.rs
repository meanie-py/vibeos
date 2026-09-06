//! Compiler-discovered native inputs, including transitive and system headers.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

/// Reuse generator output only within this build-script invocation. A persisted
/// output file is not evidence that it reflects the current inputs.
#[derive(Default)]
pub struct InvocationOutput(std::cell::OnceCell<PathBuf>);

impl InvocationOutput {
    pub fn get_or_generate(&self, generate: impl FnOnce() -> PathBuf) -> PathBuf {
        self.0.get_or_init(generate).clone()
    }
}

/// Add dependency discovery without changing a replayed compiler's cwd/flags.
/// The executor hook lets the production caller retain command tracing.
pub fn run(
    command: &mut Command,
    source: &Path,
    execute: impl FnOnce(&mut Command) -> std::io::Result<ExitStatus>,
) -> std::io::Result<ExitStatus> {
    let depfile = prepare(source);
    command
        .args(["-MD", "-MF"])
        .arg(&depfile)
        .args(["-MT", "vibeos-native"]);
    let base = command
        .get_current_dir()
        .map(Path::to_path_buf)
        .unwrap_or(std::env::current_dir()?);
    let status = execute(command)?;
    if status.success() {
        watch(&depfile, &base);
    }
    Ok(status)
}

/// Run native-hpp with runtime template discovery, resolved against its cwd.
pub fn run_hpp(
    command: &mut Command,
    source: &Path,
    execute: impl FnOnce(&mut Command) -> std::io::Result<ExitStatus>,
) -> std::io::Result<ExitStatus> {
    let depfile = prepare(source);
    command.arg("--depfile").arg(&depfile);
    let base = command
        .get_current_dir()
        .map(Path::to_path_buf)
        .unwrap_or(std::env::current_dir()?);
    let status = execute(command)?;
    if status.success() {
        watch(&depfile, &base);
    }
    Ok(status)
}

/// Compile one configured source and expose its actual include closure to Cargo.
/// Keep this on the compile itself: a separate preprocessor can disagree with
/// the compiler's target, defines, include ordering, or language flags.
pub fn compile(build: &cc::Build, source: &Path) -> Vec<PathBuf> {
    let depfile = prepare(source);
    let mut tracked = build.clone();
    tracked
        .flag("-MD") // Unlike -MMD, include SDK/toolchain system headers too.
        .flag("-MF")
        .flag(depfile.to_str().expect("native depfile path must be UTF-8"))
        .flag("-MT")
        .flag("vibeos-native");
    let objects = tracked.compile_intermediates();
    assert_eq!(
        objects.len(),
        1,
        "native include discovery requires one source"
    );
    watch(
        &depfile,
        &std::env::current_dir().expect("native compiler cwd"),
    );
    objects
}

fn prepare(source: &Path) -> PathBuf {
    let mut hash = DefaultHasher::new();
    source.hash(&mut hash);
    let depfile = PathBuf::from(std::env::var_os("OUT_DIR").expect("native compile needs OUT_DIR"))
        .join(format!("native-{:016x}.d", hash.finish()));
    // Never accept an old depfile if a compiler ignores the dependency flags.
    match std::fs::remove_file(&depfile) {
        Ok(()) => (),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => panic!("remove stale native depfile {}: {e}", depfile.display()),
    }
    depfile
}

fn watch(depfile: &Path, base: &Path) {
    let raw = std::fs::read_to_string(&depfile)
        .unwrap_or_else(|e| panic!("read compiler depfile {}: {e}", depfile.display()));
    let inputs = parse(&raw).unwrap_or_else(|e| panic!("{}: {e}", depfile.display()));
    for input in inputs {
        // A compile-db entry's cwd differs from Cargo's build-script cwd.
        let input = base.join(input);
        assert!(
            input.is_file(),
            "native prerequisite missing: {}",
            input.display()
        );
        println!("cargo:rerun-if-changed={}", input.display());
    }
}

/// Parse the single Make rule emitted by GCC/Clang with our fixed `-MT`.
/// Reject unsupported syntax rather than silently losing an input edge.
fn parse(raw: &str) -> Result<Vec<String>, &'static str> {
    let body = raw
        .strip_prefix("vibeos-native:")
        .ok_or("unexpected native depfile target")?;
    let mut chars = body.trim_end().chars().peekable();
    let mut inputs = Vec::new();
    let mut token = String::new();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some('\n') => (),
                Some('\r') if chars.next() == Some('\n') => (),
                Some(c @ (' ' | '\t' | '#' | '\\')) => token.push(c),
                _ => return Err("unsupported native depfile escape"),
            },
            '$' if chars.next() == Some('$') => token.push('$'),
            '$' | '#' | '\n' | '\r' | '\0' => return Err("unsupported native depfile syntax"),
            ' ' | '\t' => {
                if !token.is_empty() {
                    inputs.push(std::mem::take(&mut token));
                }
            }
            _ => token.push(c),
        }
    }
    if !token.is_empty() {
        inputs.push(token);
    }
    if inputs.is_empty() {
        return Err("empty native dependency closure");
    }
    inputs.sort();
    inputs.dedup();
    Ok(inputs)
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn escaped_paths_and_continuations() {
        assert_eq!(
            parse("vibeos-native: a.c space\\ name.h \\\n hash\\#name.h dollar$$name.h a.c\n")
                .unwrap(),
            ["a.c", "dollar$name.h", "hash#name.h", "space name.h"]
        );
        assert_eq!(
            parse("vibeos-native: a.c \\\r\n b.h\r\n").unwrap(),
            ["a.c", "b.h"]
        );
    }

    #[test]
    fn malformed_rules_fail_loudly() {
        for raw in [
            "",
            "other: a.c",
            "vibeos-native:",
            "vibeos-native: a.c\nb.h:",
            "vibeos-native: a$var",
            "vibeos-native: a#comment",
            "vibeos-native: a\\q",
        ] {
            assert!(parse(raw).is_err(), "accepted {raw:?}");
        }
    }
}
