use super::{executable, BinaryContext, CheckOutput};
pub fn assess(text: &str, arch: &str) -> CheckOutput {
    let mut out = String::new();
    let mut failed = false;
    let primitives = if arch == "amd64" {
        vec!["KeAcquireSpinLock", "KeReleaseSpinLock"]
    } else {
        vec![
            "KfRaiseIrql",
            "KfLowerIrql",
            "KeAcquireSpinLock",
            "KeReleaseSpinLock",
        ]
    };
    for p in primitives {
        let n = text.lines().filter(|l| l.contains(p)).count();
        if n == 0 {
            out += &format!("FAIL: {p} — not found in disassembly! LLVM may have eliminated it.\n");
            failed = true
        } else {
            out += &format!("  OK: {p} — {n} references\n")
        }
    }
    if arch == "amd64" {
        let n = text.lines().filter(|l| l.contains("cr8")).count();
        if n == 0 {
            out += "FAIL: No CR8 accesses found — IRQL primitives may be eliminated\n";
            failed = true
        } else {
            out += &format!("  OK: CR8 accesses — {n} references\n")
        }
    }
    out.push('\n');
    out += if failed {
        "WARNING: Some sacred primitives missing from binary!\nCheck --emit=llvm-ir output for the affected functions.\n"
    } else {
        "All sacred primitives verified.\n"
    };
    CheckOutput::new(out, i32::from(failed))
}
pub fn run(ctx: &BinaryContext) -> Result<CheckOutput, String> {
    let pe = ctx.build_dir.join("ntoskrnl/ntoskrnl.exe");
    if !pe.is_file() {
        return Ok(CheckOutput::new(
            format!("ERROR: {} not found. Build first.\n", pe.display()),
            1,
        ));
    }
    let triple = match ctx.arch.as_str() {
        "amd64" => "x86_64",
        "i386" => "i686",
        "arm64" => "aarch64",
        _ => return Err("unknown arch".into()),
    };
    let name = format!("{triple}-w64-mingw32-objdump");
    let candidates = [
        name.clone(),
        "llvm-objdump".into(),
        ctx.root
            .join(".build-tools/llvm-mingw/bin")
            .join(&name)
            .to_string_lossy()
            .into(),
        ctx.objdump(),
    ];
    let obj = candidates
        .into_iter()
        .find(|s| executable(s).is_some())
        .ok_or("no objdump found for sacred primitives")?;
    let text = ctx.command(&obj, &["-d".into(), pe.to_string_lossy().into()])?;
    let mut output = assess(&text, &ctx.arch);
    output.stdout = format!(
        "Checking {} ({}) with {obj}...\n\n{}",
        pe.display(),
        ctx.arch,
        output.stdout
    );
    Ok(output)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arch_coverage() {
        let text = "KeAcquireSpinLock\nKeReleaseSpinLock\ncr8\n";
        assert_eq!(assess(text, "amd64").returncode, 0);
        assert_eq!(assess(text, "arm64").returncode, 1);
        assert_eq!(
            assess("KeAcquireSpinLock\nKeReleaseSpinLock", "amd64").returncode,
            1
        );
    }
}
