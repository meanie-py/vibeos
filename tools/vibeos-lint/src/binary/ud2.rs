use crate::binary::{BinaryContext, CheckOutput};
use rayon::prelude::*;
use regex::Regex;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const EXPECTED_UD2: &[&str] = &[
    "unwrap_unchecked",
    "unreachable_unchecked",
    "panic_cold",
    "panic_cold_display",
    "panic_nounwind_cold",
    "panic_nounwind",
    "abort",
    "rust_begin_unwind",
    "copy_from_slice_impl",
    "NeverShortCircuit",
    "17compiler_builtins3int3big",
];
const EXPECTED_SPIN: &[&str] = &[
    "rust_begin_unwind",
    "KeBugCheckWithTf",
    "KeBugCheck",
    "xHalHaltSystem",
    "HvValidateHive",
    "ExitProcess",
    "FatalAppExit",
    "FatalExit",
];
const EXPECTED_SPIN_EXACT: &[&str] = &["exit"];
const DRIVER_INTRINSIC_SKIP: &[&str] = &[
    "compiler_builtins",
    "__add",
    "__sub",
    "__mul",
    "__div",
    "__mod",
    "__udiv",
    "__umod",
    "__divmod",
    "__udivmod",
    "__fix",
    "__float",
    "__trunc",
    "__extend",
    "fmodf",
    "lgamma",
    "D0Ev",
    "D1Ev",
    "D2Ev",
    ".llvm.",
];
const NORETURN_CALLEES: &[&str] = &[
    "panic",
    "KeBugCheck",
    "FatBugCheck",
    "RtlAssert",
    "abort",
    "_assert",
    "ExRaiseStatus",
    "RaiseStatus",
    "ExRaiseAccessViolation",
    "KiBugCheck",
    "HalHaltSystem",
    "rust_begin_unwind",
    "Terminate",
    "unreachable",
];
fn known_driver(name: &str) -> &'static [&'static str] {
    if name == "rosload.exe" || name == "freeldr.exe" {
        &["LoadAndBootWindowsCommon"]
    } else {
        &[]
    }
}

fn regexes() -> &'static [Regex; 7] {
    static R: OnceLock<[Regex; 7]> = OnceLock::new();
    R.get_or_init(|| {
        [
            Regex::new(r"^([0-9a-f]+)\s+<(.+)>:").unwrap(),
            Regex::new(r"^\s*([0-9a-f]+):\s+[0-9a-f]{8}\s+b\s+0x([0-9a-f]+)\b").unwrap(),
            Regex::new(r"^\s*([0-9a-f]+):\s+0f 0b\s+ud2").unwrap(),
            Regex::new(r"\b(?:call\w*|bl)\s+.*<(.+?)>").unwrap(),
            Regex::new(r"\bint\s+\$0x29\b").unwrap(),
            Regex::new(r"^\s*[0-9a-f]+:").unwrap(),
            Regex::new(r"\baddl?\s+\$0x[0-9a-f]+, %esp").unwrap(),
        ]
    })
}
fn has_any(s: &str, items: &[&str]) -> bool {
    items.iter().any(|x| s.contains(x))
}
fn is_mangled(s: &str) -> bool {
    let b = s.trim_start_matches(|c| c == '_' || c == '@');
    b.starts_with("ZN") || (b.starts_with('R') && !b.starts_with("Rtl"))
}
fn objdump(
    ctx: &BinaryContext,
    tool: &str,
    path: &Path,
    timeout_label: &str,
) -> Result<String, String> {
    let args = vec!["-d".to_string(), path.to_string_lossy().into_owned()];
    ctx.command(tool, &args)
        .map_err(|e| format!("{}: {}", timeout_label, e))
}
fn facts(lines: impl Iterator<Item = String>, arch: &str, binary: Option<&str>) -> Vec<Value> {
    let r = regexes();
    let driver = binary.is_some();
    let mut out = Vec::new();
    let mut current = String::new();
    let mut prev_call: Option<String> = None;
    let mut prev_fast = false;
    for line in lines {
        if let Some(c) = r[0].captures(&line) {
            current = c[2].to_string();
            prev_call = None;
            // scan_ud2's kernel path carries its immediately previous instruction across labels;
            // the driver path resets call adjacency but deliberately retains fastfail state.
            continue;
        }
        if current.is_empty() {
            continue;
        }
        let (hit, addr) = if arch == "arm64" {
            if let Some(c) = r[1].captures(&line) {
                (c[1] == c[2], c[1].to_string())
            } else {
                (false, String::new())
            }
        } else if let Some(c) = r[2].captures(&line) {
            (true, c[1].to_string())
        } else {
            (false, String::new())
        };
        if !hit {
            if r[5].is_match(&line) {
                prev_fast = r[4].is_match(&line);
            }
            if let Some(c) = r[3].captures(&line) {
                prev_call = Some(c[1].to_string());
            } else if !line.trim().is_empty() && !r[6].is_match(&line) && !line.contains("nop") {
                prev_call = None;
            }
            continue;
        }
        let expected = if driver {
            has_any(&current, EXPECTED_UD2)
                || (arch == "arm64"
                    && (has_any(&current, EXPECTED_SPIN)
                        || EXPECTED_SPIN_EXACT.contains(&current.as_str())))
                || has_any(&current, DRIVER_INTRINSIC_SKIP)
                || is_mangled(&current)
                || prev_call
                    .as_deref()
                    .is_some_and(|s| has_any(s, NORETURN_CALLEES))
                || prev_fast
        } else if arch == "arm64" {
            has_any(&current, EXPECTED_SPIN) || EXPECTED_SPIN_EXACT.contains(&current.as_str())
        } else {
            // Kernel x86 is intentionally narrow: only named unreachable stubs and an
            // immediately preceding int $0x29 are trusted here.
            has_any(&current, EXPECTED_UD2) || prev_fast
        };
        let known =
            driver && binary.is_some_and(|b| known_driver(b).iter().any(|x| current.contains(x)));
        let mut item =
            json!({"address":addr,"function":current,"expected":expected,"known_cgu":known});
        if let Some(b) = binary {
            item["binary"] = json!(b);
        }
        out.push(item);
        // scan_ud2 stores every instruction in prev_insn, including a hit; driver facts keep
        // prev_fastfail unchanged when the current instruction itself is a hit.
        if !driver && r[5].is_match(&line) {
            prev_fast = r[4].is_match(&line);
        }
    }
    out
}
fn scan_one(
    ctx: &BinaryContext,
    tool: &str,
    path: &Path,
    arch: &str,
) -> Result<Vec<Value>, String> {
    Ok(facts(
        objdump(ctx, tool, path, "objdump failed")?
            .lines()
            .map(str::to_string),
        arch,
        Some(
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .as_ref(),
        ),
    ))
}
fn tool_names(ctx: &BinaryContext, arch: &str, nm: bool) -> Vec<String> {
    let base = match (arch, nm) {
        ("arm64", false) => vec!["aarch64-w64-mingw32-objdump", "llvm-objdump"],
        ("amd64", false) => vec!["x86_64-w64-mingw32-objdump", "llvm-objdump"],
        ("i386", false) => vec!["i686-w64-mingw32-objdump", "llvm-objdump"],
        ("arm64", true) => vec!["aarch64-w64-mingw32-nm", "llvm-nm"],
        ("amd64", true) => vec!["x86_64-w64-mingw32-nm", "llvm-nm"],
        ("i386", true) => vec!["i686-w64-mingw32-nm", "llvm-nm"],
        (_, false) => vec!["llvm-objdump"],
        (_, true) => vec!["llvm-nm"],
    };
    let mut out: Vec<String> = base.iter().map(|x| x.to_string()).collect();
    let bundled = ctx_tool_dir(ctx);
    for x in base {
        out.push(format!("{}/{}", bundled, x));
    }
    out
}
fn ctx_tool_dir(ctx: &BinaryContext) -> String {
    ctx.root
        .join(".build-tools/llvm-mingw/bin")
        .to_string_lossy()
        .into_owned()
}
fn find_tool(ctx: &BinaryContext, arch: &str, nm: bool) -> Option<String> {
    for p in tool_names(ctx, arch, nm) {
        if ctx.command(&p, &["--version".into()]).is_ok() {
            return Some(p);
        }
    }
    None
}
fn find_rust_binaries(
    ctx: &BinaryContext,
    nm: &str,
) -> Result<(Vec<PathBuf>, Vec<PathBuf>), String> {
    let corpus = ctx.binaries(
        &[".sys", ".dll", ".exe", ".drv"],
        &[
            "boot/stripped",
            "boot/ramdisk",
            "boot/ramdisk-overlay",
            "livecd",
            "bootcd",
            "CMakeFiles",
            "host-tools",
        ],
    )?;
    let corpus_paths = corpus.clone();
    let paths: Vec<PathBuf> = corpus
        .into_iter()
        .filter(|p| p.file_name().is_none_or(|n| n != "ntoskrnl.exe"))
        .collect();
    let found: Vec<Result<Option<PathBuf>, String>> = paths
        .par_iter()
        .map(|p| {
            let o = ctx.command(nm, &[p.to_string_lossy().into_owned()])?;
            Ok(if o.contains("rust_begin_unwind") {
                Some(p.clone())
            } else {
                None
            })
        })
        .collect();
    let mut bins = Vec::new();
    for v in found {
        if let Some(p) = v? {
            bins.push(p)
        }
    }
    Ok((bins, corpus_paths))
}
fn classify(findings: &[Value]) -> (Vec<&Value>, Vec<&Value>, Vec<&Value>) {
    let mut e = vec![];
    let mut k = vec![];
    let mut u = vec![];
    for f in findings {
        if f["expected"] == true {
            e.push(f)
        } else if f["known_cgu"] == true {
            k.push(f)
        } else {
            u.push(f)
        }
    }
    (e, k, u)
}
fn print_driver(out: &mut String, title: &str, fs: &[Value]) {
    if fs.is_empty() {
        return;
    }
    out.push_str(&format!("  {} ({}):\n", title, fs.len()));
    for f in fs {
        out.push_str(&format!(
            "    {}: 0x{}: {}{}\n",
            f["binary"].as_str().unwrap_or(""),
            f["address"].as_str().unwrap_or(""),
            f["function"].as_str().unwrap_or(""),
            if f["known_cgu"] == true {
                "  (known)"
            } else {
                "  *** BUG ***"
            }
        ));
    }
}

pub fn run(ctx: &BinaryContext) -> Result<CheckOutput, String> {
    let arch = ctx.arch.as_str();
    let kernel = ctx.build_dir.join("ntoskrnl/ntoskrnl.exe");
    if !kernel.exists() {
        return Err(format!("ERROR: {} not found", kernel.display()));
    }
    let obj = find_tool(ctx, arch, false).ok_or_else(|| "ERROR: No objdump found".to_string())?;
    let (findings, driver_bad, driver_known, corpus_count, bins_count) =
        rayon::scope(|_| -> Result<_, String> {
            let k = facts(
                objdump(ctx, &obj, &kernel, "objdump failed")?
                    .lines()
                    .map(str::to_string),
                arch,
                None,
            );
            let nm = find_tool(ctx, arch, true)
                .ok_or_else(|| "no nm found — driver sweep cannot run".to_string())?;
            let (bins, corpus) = find_rust_binaries(ctx, &nm)?;
            let all: Vec<Result<Vec<Value>, String>> = bins
                .par_iter()
                .map(|p| scan_one(ctx, &obj, p, arch))
                .collect();
            let mut bad = Vec::new();
            let mut known = Vec::new();
            for x in all {
                for f in x?.into_iter().filter(|f| f["expected"] != true) {
                    if f["known_cgu"] == true {
                        known.push(f)
                    } else {
                        bad.push(f)
                    }
                }
            }
            Ok((k, bad, known, corpus.len(), bins.len()))
        })?;
    let (expected, known, unexpected) = classify(&findings);
    let mut stdout = String::new();
    let kind = if arch == "arm64" {
        "branch-to-self spins"
    } else {
        "ud2 instructions"
    };
    let hint = if arch == "arm64" {
        "unintended infinite loop (arm64 'idle hang' signature)"
    } else {
        "likely LLVM miscompilation"
    };
    stdout.push_str(&format!(
        "Scanning {} ({}) for {}...\n",
        kernel.display(),
        arch,
        kind
    ));
    if !expected.is_empty() {
        stdout.push_str(&format!(
            "\n  Expected ({} — {}):\n",
            expected.len(),
            if arch == "arm64" {
                "panic/halt spins"
            } else {
                "unreachable panic paths"
            }
        ));
        for f in &expected {
            stdout.push_str(&format!(
                "    0x{}: {}\n",
                f["address"].as_str().unwrap_or(""),
                f["function"].as_str().unwrap_or("")
            ));
        }
    }
    if !known.is_empty() {
        stdout.push_str(&format!(
            "\n  Known CGU ud2 ({} — rustc CGU partitioning bug, non-blocking):\n",
            known.len()
        ));
        for f in &known {
            stdout.push_str(&format!(
                "    0x{}: {}  (known)\n",
                f["address"].as_str().unwrap_or(""),
                f["function"].as_str().unwrap_or("")
            ));
        }
    }
    if !unexpected.is_empty() {
        stdout.push_str(&format!(
            "\n  UNEXPECTED ({} — {}):\n",
            unexpected.len(),
            hint
        ));
        for f in &unexpected {
            stdout.push_str(&format!(
                "    0x{}: {}  *** BUG ***\n",
                f["address"].as_str().unwrap_or(""),
                f["function"].as_str().unwrap_or("")
            ));
        }
    }
    if findings.is_empty() {
        stdout.push_str(&format!("  No {} found.\n", kind));
    }
    if arch == "arm64" {
        stdout.push_str("\n  NOTE: arm64 has no instruction-level UB trap (trap-unreachable off);\n  the captures(none)/UB class is SILENT here and is gated at source by\n  tools/lint_kernel.py — this scan only covers branch-to-self spins.\n");
    }
    stdout.push_str(&format!(
        "\nTotal: {} {} ({} expected, {} known CGU, {} unexpected)\n",
        findings.len(),
        kind.split_whitespace().next().unwrap_or(""),
        expected.len(),
        known.len(),
        unexpected.len()
    ));
    let mut driver_out = String::new();
    print_driver(&mut driver_out, "Known residuals", &driver_known);
    print_driver(
        &mut driver_out,
        &format!(
            "UNEXPECTED driver {} (§9.99 CC/sig-mismatch fold)",
            if arch == "arm64" { "spins" } else { "ud2" }
        ),
        &driver_bad,
    );
    stdout.push_str(&format!("\nDriver sweep: {} Rust-containing binaries scanned of {} in corpus\n  corpus basis: build tree (the livecd DAG re-rolls every shipped binary each build)\n",bins_count,corpus_count));
    stdout.push_str(&driver_out);
    if driver_bad.is_empty() && driver_known.is_empty() {
        stdout.push_str(&format!(
            "  No unexpected driver {}.\n",
            if arch == "arm64" { "spins" } else { "ud2" }
        ));
    }
    if corpus_count == 0 {
        return Err(format!(
            "empty binary corpus for {} — nothing was swept.",
            ctx.build_dir.display()
        ));
    }
    let failed = !unexpected.is_empty() || !driver_bad.is_empty();
    if failed {
        stdout.push_str(&format!(
            "\nFAILED: Unexpected {} found — {}!\n",
            if arch == "arm64" {
                "branch-to-self spin"
            } else {
                "ud2"
            },
            hint
        ));
    } else {
        stdout.push_str("\nPASSED: All findings are in expected or known-CGU locations.\n");
    }
    Ok(CheckOutput::new(stdout, if failed { 1 } else { 0 }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn x86_expected_and_unexpected_ud2() {
        let input="0046b350 <_panic_cold>:\n  46b350: 0f 0b ud2\n0046b360 <MmBad>:\n  46b360: 0f 0b ud2\n";
        let f = facts(input.lines().map(str::to_owned), "i386", None);
        assert_eq!(f.len(), 2);
        assert_eq!(f[0]["expected"], true);
        assert_eq!(f[1]["expected"], false);
    }
    #[test]
    fn arm64_self_branch_classifies_terminal_only() {
        let input="00001000 <KeBugCheck>:\n    1000: d503201f b 0x1000\n00001004 <Worker>:\n    1004: d503201f b 0x1004\n00001008 <exit>:\n    1008: d503201f b 0x1008\n";
        let f = facts(input.lines().map(str::to_owned), "arm64", None);
        assert_eq!(f.len(), 3);
        assert_eq!(f[0]["expected"], true);
        assert_eq!(f[1]["expected"], false);
        assert_eq!(f[2]["expected"], true);
    }
    #[test]
    fn driver_noreturn_and_known_exception() {
        let input="0001 <Bad>:\n  1: e8 00 call 2 <KeBugCheck>\n  6: 0f 0b ud2\n0002 <LoadAndBootWindowsCommon>:\n  2: 0f 0b ud2\n";
        let f = facts(
            input.lines().map(str::to_owned),
            "i386",
            Some("rosload.exe"),
        );
        assert_eq!(f.len(), 2);
        assert_eq!(f[0]["expected"], true);
        assert_eq!(f[1]["known_cgu"], true);
    }
    #[test]
    fn fastfail_preceding_ud2_is_expected() {
        let input = "0001 <Bad>:\n  1: cd 29 int $0x29\n  3: 0f 0b ud2\n";
        let f = facts(input.lines().map(str::to_owned), "i386", None);
        assert_eq!(f[0]["expected"], true);
    }
    #[test]
    fn kernel_x86_does_not_inherit_driver_allowances() {
        let input = "0001 <_ZN7mangledE>:\n  1: 0f 0b ud2\n0002 <Bad>:\n  2: e8 00 call 3 <KeBugCheck>\n  7: 0f 0b ud2\n";
        let kernel = facts(input.lines().map(str::to_owned), "i386", None);
        let driver = facts(input.lines().map(str::to_owned), "i386", Some("x.sys"));
        assert_eq!(
            kernel
                .iter()
                .map(|f| f["expected"].as_bool().unwrap())
                .collect::<Vec<_>>(),
            vec![false, false]
        );
        assert_eq!(
            driver
                .iter()
                .map(|f| f["expected"].as_bool().unwrap())
                .collect::<Vec<_>>(),
            vec![true, true]
        );
    }

    #[test]
    fn arm64_kernel_and_driver_have_distinct_allow_sets() {
        let input = "0001 <abort>:\n    1: d503201f b 0x1\n";
        let kernel = facts(input.lines().map(str::to_owned), "arm64", None);
        let driver = facts(input.lines().map(str::to_owned), "arm64", Some("x.sys"));
        assert_eq!(kernel[0]["expected"], false);
        assert_eq!(driver[0]["expected"], true);
    }
}
