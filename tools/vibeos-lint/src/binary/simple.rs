use crate::binary::{BinaryContext, CheckOutput, BASIS};
use regex::Regex;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const PE_SUFFIXES: &[&str] = &[".sys", ".dll", ".exe", ".drv", ".cpl", ".ocx"];
const SELF_SUFFIXES: &[&str] = &[".sys", ".dll", ".exe", ".drv"];

fn re(slot: &'static OnceLock<Regex>, pattern: &str) -> &'static Regex {
    slot.get_or_init(|| Regex::new(pattern).expect("valid scanner regex"))
}

fn lines(path: &Path) -> Result<Vec<String>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    Ok(text
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .map(|line| line.trim().to_owned())
        .collect())
}

fn run_objdump(
    ctx: &BinaryContext,
    args: &[&str],
    pe: &Path,
    failure: &str,
) -> Result<String, String> {
    let mut argv: Vec<String> = args.iter().map(|s| (*s).to_owned()).collect();
    argv.push(pe.to_string_lossy().into_owned());
    ctx.command(&ctx.objdump(), &argv)
        .map_err(|e| format!("{failure} {}: {e}", pe.display()))
}

fn corpus(ctx: &BinaryContext, suffixes: &[&str]) -> Result<Vec<PathBuf>, String> {
    ctx.binaries(
        suffixes,
        &[
            "CMakeFiles",
            "host-tools",
            "/boot/stripped/",
            "/boot/staging/",
            "/boot/ramdisk/",
            "/boot/ramdisk-overlay/",
        ],
    )
}

fn self_recursive_pe(ctx: &BinaryContext, pe: &Path) -> Result<Vec<(String, String)>, String> {
    let out = run_objdump(ctx, &["-d", "--no-show-raw-insn"], pe, "cannot disassemble")?;
    Ok(self_jumps(
        &out,
        pe.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .as_ref(),
    ))
}

fn self_jumps(disassembly: &str, module: &str) -> Vec<(String, String)> {
    static SYM: OnceLock<Regex> = OnceLock::new();
    static JMP: OnceLock<Regex> = OnceLock::new();
    let rows: Vec<&str> = disassembly.lines().collect();
    let mut found = Vec::new();
    for i in 0..rows.len().saturating_sub(1) {
        let Some(m) = re(&SYM, r"^([0-9a-f]+) <([^>]+)>:").captures(rows[i]) else {
            continue;
        };
        let Ok(addr) = u64::from_str_radix(&m[1], 16) else {
            continue;
        };
        let sym = m[2].to_owned();
        if let Some(j) = re(
            &JMP,
            r"^\s*[0-9a-f]+:\s+(?:jmp|jmpq|b)\s+(?:0x)?([0-9a-f]+)\b",
        )
        .captures(rows[i + 1])
        {
            if u64::from_str_radix(&j[1], 16).ok() == Some(addr) {
                found.push((module.to_owned(), sym));
            }
        }
    }
    found
}

fn baseline(ctx: &BinaryContext, name: &str) -> Result<BTreeSet<String>, String> {
    let path = ctx.root.join("tools/lint-baseline").join(name);
    if !path.is_file() {
        return Ok(BTreeSet::new());
    }
    Ok(lines(&path)?.into_iter().collect())
}

pub fn self_recursive(ctx: &BinaryContext) -> Result<CheckOutput, String> {
    if !ctx.build_dir.is_dir() {
        return Ok(CheckOutput {
            stdout: String::new(),
            stderr: format!(
                "scan_self_recursive: no such build dir: {}\n",
                ctx.build_dir.display()
            ),
            returncode: 2,
        });
    }
    let paths = corpus(ctx, SELF_SUFFIXES)?;
    let mut hits = Vec::new();
    let mut scanned = 0usize;
    for pe in paths.iter().filter(|p| p.is_file()) {
        hits.extend(self_recursive_pe(ctx, pe)?);
        scanned += 1;
    }
    if scanned == 0 {
        return Ok(CheckOutput {
            stdout: String::new(),
            stderr: format!(
                "scan_self_recursive: no PE inputs in {} (corpus basis: {BASIS})\n",
                ctx.build_dir.display(),
            ),
            returncode: 2,
        });
    }
    let base = baseline(ctx, "scan_self_recursive.txt")?;
    let found: BTreeSet<String> = hits.into_iter().map(|(m, s)| format!("{m}: {s}")).collect();
    let new: Vec<String> = found.difference(&base).cloned().collect();
    if found.is_empty() {
        return Ok(CheckOutput::new(
            format!(
                "scan_self_recursive: {} — clean (no self-jump functions)\n",
                ctx.build_dir.display()
            ),
            0,
        ));
    }
    let mut out = format!(
        "scan_self_recursive: {} self-jump function(s) ({} NEW, {} baselined):\n\n",
        found.len(),
        new.len(),
        found.len() - new.len()
    );
    for entry in &found {
        out.push_str(&format!(
            "  [{}] {entry}\n",
            if new.contains(entry) {
                "NEW  "
            } else {
                "known"
            }
        ));
    }
    if !new.is_empty() {
        out.push_str(&format!("\n{} NEW INFINITE SELF-JUMP(S) — these hang the CPU silently (no bugcheck, no output).\nA Rust `#[no_mangle] fn memcpy/memset/...` whose body uses `copy_nonoverlapping`/`write_bytes`\nis the usual cause: LLVM lowers the body back into a call to the function ITSELF.\nDon't define mem* in Rust — import ntoskrnl's exports. See this file's docstring.\n", new.len()));
    }
    Ok(CheckOutput::new(out, if !new.is_empty() { 1 } else { 0 }))
}

fn exports_of(headers: &str) -> BTreeSet<String> {
    static EXPORT: OnceLock<Regex> = OnceLock::new();
    let mut names = BTreeSet::new();
    let mut inside = false;
    for line in headers.lines() {
        if line.starts_with("Export Table:") {
            inside = true;
            continue;
        }
        if inside {
            if let Some(m) = re(&EXPORT, r"^\s+\d+\s+0x[0-9a-f]+\s+(\S+)$").captures(line) {
                names.insert(m[1].to_owned());
            } else if line.trim().is_empty() && !names.is_empty() {
                break;
            }
        }
    }
    names
}

fn normalized(sym: &str) -> String {
    sym.trim_start_matches('_')
        .split('@')
        .next()
        .unwrap_or(sym)
        .to_owned()
}

fn self_thunks(disasm: &str, exports: &BTreeSet<String>) -> Vec<String> {
    static SYM: OnceLock<Regex> = OnceLock::new();
    static IMP: OnceLock<Regex> = OnceLock::new();
    let rows: Vec<&str> = disasm.lines().collect();
    let mut hits = BTreeSet::new();
    for (i, line) in rows.iter().enumerate() {
        let Some(m) = re(&SYM, r"^[0-9a-f]+ <([^>]+)>:").captures(line) else {
            continue;
        };
        let sym = m[1].to_owned();
        let bare = normalized(&sym);
        if !exports.contains(&sym) && !exports.contains(&bare) {
            continue;
        }
        for next in rows.iter().skip(i + 1).take(3) {
            if re(&SYM, r"^[0-9a-f]+ <([^>]+)>:").is_match(next) {
                break;
            }
            if let Some(im) = re(&IMP, r"<__imp_([^>+]+)>").captures(next) {
                if normalized(&im[1]) == bare {
                    hits.insert(bare.clone());
                }
                break;
            }
        }
    }
    hits.into_iter().collect()
}

fn scan_export_pe(ctx: &BinaryContext, pe: &Path) -> Result<Vec<(String, String)>, String> {
    let headers = run_objdump(ctx, &["-p"], pe, "cannot read")?;
    let exports = exports_of(&headers);
    if exports.is_empty() {
        return Ok(Vec::new());
    }
    let disasm = run_objdump(ctx, &["-d", "--no-show-raw-insn"], pe, "cannot read")?;
    Ok(self_thunks(&disasm, &exports)
        .into_iter()
        .map(|sym| {
            (
                pe.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                sym,
            )
        })
        .collect())
}

pub fn export_thunks(ctx: &BinaryContext) -> Result<CheckOutput, String> {
    if !ctx.build_dir.is_dir() {
        return Ok(CheckOutput {
            stdout: String::new(),
            stderr: format!(
                "scan_export_thunks: no such build dir: {}\n",
                ctx.build_dir.display()
            ),
            returncode: 2,
        });
    }
    if ctx.arch != "amd64" {
        return Ok(CheckOutput { stdout: String::new(), stderr: format!("scan_export_thunks: {} is not an amd64 build; the IAT-name annotation this scan reads exists only there (see the docstring)\n", ctx.build_dir.display()), returncode: 2 });
    }
    let paths = corpus(ctx, PE_SUFFIXES)?;
    let mut hits = Vec::new();
    let mut scanned = 0;
    for pe in paths.iter().filter(|p| p.is_file()) {
        hits.extend(scan_export_pe(ctx, pe)?);
        scanned += 1;
    }
    if scanned == 0 {
        return Ok(CheckOutput {
            stdout: String::new(),
            stderr: format!(
                "scan_export_thunks: no PE inputs in {} (corpus basis: {BASIS})\n",
                ctx.build_dir.display(),
            ),
            returncode: 2,
        });
    }
    let found: BTreeSet<String> = hits.into_iter().map(|(m, s)| format!("{m}: {s}")).collect();
    let base = baseline(ctx, "scan_export_thunks.txt")?;
    let new: Vec<_> = found.difference(&base).cloned().collect();
    if found.is_empty() {
        return Ok(CheckOutput::new(
            format!("scan_export_thunks: {} — clean\n", ctx.build_dir.display()),
            0,
        ));
    }
    let mut out = format!(
        "scan_export_thunks: {} export(s) that are an import thunk of themselves ({} NEW):\n\n",
        found.len(),
        new.len()
    );
    for entry in &found {
        out.push_str(&format!(
            "  [{}] {entry}\n",
            if new.contains(entry) {
                "NEW  "
            } else {
                "known"
            }
        ));
    }
    if !new.is_empty() {
        out.push_str("\nA NEW export resolves to another module's same-named import. If the module was\nmeant to define it (a Rust #[no_mangle] body, say), an import library won the\nlink: force the local one with LINKER:--require-defined=<sym> (not -u, which lld\nmaps to -includeoptional). If the re-export is deliberate, add it to the baseline.\n");
    }
    Ok(CheckOutput::new(out, if !new.is_empty() { 1 } else { 0 }))
}

fn xchg_findings(disasm: &str) -> Vec<(String, String)> {
    static SYM: OnceLock<Regex> = OnceLock::new();
    static INSN: OnceLock<Regex> = OnceLock::new();
    static XCHG: OnceLock<Regex> = OnceLock::new();
    static EBX: OnceLock<Regex> = OnceLock::new();
    let mut hits = Vec::new();
    let mut func = "?".to_owned();
    let mut window = 0usize;
    for line in disasm.lines() {
        if let Some(m) = re(&SYM, r"^[0-9a-f]+ <([^>]+)>:").captures(line) {
            func = m[1].to_owned();
            window = 0;
            continue;
        }
        let Some(m) = re(&INSN, r"^\s*([0-9a-f]+):\s+(.*)$").captures(line) else {
            continue;
        };
        let addr = m[1].to_owned();
        let insn = m[2].split('#').next().unwrap_or("").trim();
        let swap = re(&XCHG, r"^xchgl\s+(%ebx,\s*%e\w+|%e\w+,\s*%ebx)\s*$")
            .captures(insn)
            .and_then(|m| m.get(1))
            .map(|m| m.as_str());
        let is_ebx_swap = swap.is_some_and(|operands| {
            let other = operands
                .split(',')
                .map(str::trim)
                .find(|operand| *operand != "%ebx");
            other.is_some_and(|operand| operand != "%eax")
        });
        if is_ebx_swap {
            window = if window == 0 { 4 } else { 0 };
            continue;
        }
        if window > 0 {
            if re(&EBX, r"\(%ebx[,)]").is_match(insn) {
                hits.push((func.clone(), addr));
            }
            window -= 1;
        }
    }
    hits
}

pub fn xchg_ebx_deref(ctx: &BinaryContext) -> Result<CheckOutput, String> {
    if !ctx.build_dir.is_dir() {
        return Ok(CheckOutput {
            stdout: String::new(),
            stderr: format!(
                "scan_xchg_ebx_deref: no such build dir: {}\n",
                ctx.build_dir.display()
            ),
            returncode: 2,
        });
    }
    if ctx.arch != "i386" {
        return Ok(CheckOutput {
            stdout: String::new(),
            stderr: format!(
                "scan_xchg_ebx_deref: {} is not an i386 build; the defect needs an allocatable ebx (see the docstring)\n",
                ctx.build_dir.display()
            ),
            returncode: 2,
        });
    }
    let paths = corpus(ctx, PE_SUFFIXES)?;
    let mut hits = Vec::new();
    let mut scanned = 0;
    for pe in paths.iter().filter(|p| p.is_file()) {
        let out = run_objdump(ctx, &["-d", "--no-show-raw-insn"], pe, "cannot read")?;
        for (f, a) in xchg_findings(&out) {
            hits.push((
                pe.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                f,
                a,
            ));
        }
        scanned += 1;
    }
    if scanned == 0 {
        return Ok(CheckOutput {
            stdout: String::new(),
            stderr: format!(
                "scan_xchg_ebx_deref: no PE inputs in {} (corpus basis: {BASIS})\n",
                ctx.build_dir.display(),
            ),
            returncode: 2,
        });
    }
    if hits.is_empty() {
        return Ok(CheckOutput::new(
            format!(
                "scan_xchg_ebx_deref: {} — clean ({} binaries)\n",
                ctx.build_dir.display(),
                scanned
            ),
            0,
        ));
    }
    let count = hits.len();
    let found: BTreeSet<String> = hits
        .into_iter()
        .map(|(m, f, a)| format!("{m}: {f} @ {a}"))
        .collect();
    let mut out = format!(
        "scan_xchg_ebx_deref: {} ebx dereference(s) inside an ebx swap:\n\n",
        count
    );
    for h in &found {
        out.push_str(&format!("  {h}\n"));
    }
    out.push_str("\nThe instruction dereferences ebx after ebx was swapped out, so it uses the\nswapped-in VALUE as the address. In Rust inline asm this means LLVM allocated\nthe pointer operand to ebx: pin it to a fixed register (in(\"edi\") addr) and\nname that register in the template.\n");
    Ok(CheckOutput::new(out, 1))
}

#[cfg(test)]
mod fixtures {
    use super::*;
    #[test]
    fn detects_first_instruction_self_jump() {
        let input = "00001000 <loop>:\n    1000: jmp 0x1000 <loop>\n";
        assert_eq!(
            self_jumps(input, "kernel.exe"),
            vec![("kernel.exe".into(), "loop".into())]
        );
    }
    #[test]
    fn exported_self_import_thunk_is_found() {
        let exports = exports_of("Export Table:\n     105  0x3d33a  CreateErrorInfo\n\nData");
        let dis="000007ffa803d33a <CreateErrorInfo>:\n7ffa803d33a: jmpq *0x79768(%rip) # <__imp_CreateErrorInfo>\n";
        assert_eq!(self_thunks(dis, &exports), vec!["CreateErrorInfo"]);
    }
    #[test]
    fn ebx_swap_dereference_detected_but_fixed_operand_is_clean() {
        let broken="7e0803d0 <push>:\n7e080401: xchgl %ebx, %edi\n7e080403: lock\n7e080404: cmpxchg8b (%ebx)\n7e080407: xchgl %ebx, %edi\n";
        assert_eq!(
            xchg_findings(broken),
            vec![("push".into(), "7e080404".into())]
        );
        assert!(xchg_findings(&broken.replace("(%ebx)", "(%edi)")).is_empty());
        let cpuid = "func:\n0000: xchgl %ebx, %esi\n0001: cpuid\n0002: xchgl %esi, %ebx\n0003: movl %ebx, (%edi)\n";
        assert!(xchg_findings(cpuid).is_empty());
        let jump_table =
            "func:\n0000: xchgl %ebx, %eax\n0001: addl %eax, (%eax)\n0002: addb %dl, -4(%ebx)\n";
        assert!(xchg_findings(jump_table).is_empty());
    }
}
