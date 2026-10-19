use crate::binary::{BinaryContext, CheckOutput};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

const ARCHES: &[&str] = &["i386", "amd64", "arm64"];
const GENINC: &[(&str, &str, &str)] = &[
    ("i386", "i686-w64-mingw32", "ks386.inc"),
    ("amd64", "x86_64-w64-mingw32", "ksamd64.inc"),
    ("arm64", "aarch64-w64-mingw32", "ksarm64.h"),
];
const WIDL_STUBS: &[(&str, &str, &str, &str)] = &[
    (
        "svcctl_s",
        "sdk/lib/rust/svcctl-stub/data/svcctl_s.{arch}.rs",
        "build-{arch}/base/system/services/svcctl_s.c",
        "build-{arch}/base/system/services/CMakeFiles",
    ),
    (
        "winreg_s",
        "sdk/lib/rust/winreg-stub/data/winreg_s.{arch}.rs",
        "build-{arch}/base/system/winlogon/winreg_s.c",
        "build-{arch}/base/system/winlogon/CMakeFiles",
    ),
];

const OK: &str = "OK";
const DRIFT: &str = "DRIFT";
const SKIP: &str = "SKIP";
type Row = (String, String, &'static str, String);

fn selected_arches(ctx: &BinaryContext) -> Vec<&'static str> {
    if ctx.arch == "all" {
        ARCHES.to_vec()
    } else {
        ARCHES.iter().copied().filter(|a| *a == ctx.arch).collect()
    }
}

static TEMP_ID: AtomicU64 = AtomicU64::new(0);
struct TempDir(PathBuf);
impl TempDir {
    fn new() -> Result<Self, String> {
        let id = TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("vibeos-lint-drift-{}-{id}", std::process::id()));
        fs::create_dir(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(Self(path))
    }
    fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn python() -> String {
    std::env::var("VIBEOS_LINT_PYTHON").unwrap_or_else(|_| "python3".to_owned())
}
fn command(
    root: &Path,
    program: &str,
    args: &[String],
    cwd: Option<&Path>,
) -> Result<Output, String> {
    let mut cmd = Command::new(program);
    cmd.args(args).current_dir(cwd.unwrap_or(root));
    cmd.output()
        .map_err(|e| format!("cannot run {program}: {e}"))
}
fn cmd_args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| (*s).to_owned()).collect()
}
fn fmt_arch(template: &str, arch: &str) -> String {
    template.replace("{arch}", arch)
}
fn add(
    rows: &mut Vec<Row>,
    kind: &str,
    name: impl Into<String>,
    status: &'static str,
    detail: impl Into<String>,
) {
    rows.push((kind.to_owned(), name.into(), status, detail.into()));
}
fn compare_text(committed: &Path, fresh: &str) -> Result<bool, String> {
    Ok(
        fs::read_to_string(committed).map_err(|e| format!("{}: {e}", committed.display()))?
            == fresh,
    )
}
fn read_bytes(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|e| format!("{}: {e}", path.display()))
}
fn write_json_command(root: &Path, script: &str, args: &[String]) -> Result<Value, String> {
    let output = command(
        root,
        &python(),
        &[
            vec![root.join(script).to_string_lossy().into_owned()],
            args.to_vec(),
        ]
        .concat(),
        Some(root),
    )?;
    if !output.status.success() {
        return Err(format!(
            "{} failed: {}",
            script,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("{} returned invalid JSON: {e}", script))
}

fn check_geninc_arch(ctx: &BinaryContext, arch: &str, rows: &mut Vec<Row>) -> Result<(), String> {
    let Some((_, triple, ksf)) = GENINC.iter().find(|(a, _, _)| *a == arch) else {
        return Ok(());
    };
    let bd = ctx.root.join(format!("build-{arch}"));
    let cmake_ks = bd.join("sdk/include/asm").join(ksf);
    let flags_f = ctx.root.join(format!(
        "sdk/lib/rust/reactos-link/cflags/genincdata.{arch}.flags"
    ));
    let clang = ctx
        .root
        .join(format!(".build-tools/llvm-mingw/bin/{triple}-clang"));
    if !cmake_ks.exists() || !flags_f.exists() || !clang.exists() {
        add(
            rows,
            "geninc",
            *ksf,
            SKIP,
            format!("no build-{arch} / flags / clang"),
        );
        return Ok(());
    }
    let flags = fs::read_to_string(&flags_f).map_err(|e| format!("{}: {e}", flags_f.display()))?;
    let flags: Vec<String> = flags
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.replace("{REPO}", &ctx.root.to_string_lossy()))
        .collect();
    let tmp = TempDir::new()?;
    let mc = tmp.path().join("mc");
    fs::create_dir(&mc).map_err(|e| e.to_string())?;
    fs::copy(
        bd.join("sdk/include/reactos/mc/bugcodes.h"),
        mc.join("bugcodes.h"),
    )
    .map_err(|e| e.to_string())?;
    let obj = tmp.path().join("genincdata.o");
    let mut args = flags;
    args.extend([
        "-I".into(),
        bd.join("sdk/include/psdk").to_string_lossy().into_owned(),
    ]);
    args.extend([
        "-I".into(),
        bd.join("sdk/include/ddk").to_string_lossy().into_owned(),
    ]);
    args.extend(["-I".into(), mc.to_string_lossy().into_owned()]);
    args.push(
        ctx.root
            .join("sdk/include/asm/genincdata.c")
            .to_string_lossy()
            .into_owned(),
    );
    args.extend(["-o".into(), obj.to_string_lossy().into_owned()]);
    let output = command(&ctx.root, &clang.to_string_lossy(), &args, Some(&ctx.root))?;
    if !output.status.success() {
        add(rows, "geninc", *ksf, SKIP, "genincdata compile failed");
        return Ok(());
    }
    let text = emit_ks(&read_coff_section(&obj, ".asmdef")?)?;
    let expected =
        fs::read_to_string(&cmake_ks).map_err(|e| format!("{}: {e}", cmake_ks.display()))?;
    add(
        rows,
        "geninc",
        *ksf,
        if text == expected { OK } else { DRIFT },
        "self-host vs build dir",
    );
    Ok(())
}

fn check_geninc(ctx: &BinaryContext, rows: &mut Vec<Row>) -> Result<(), String> {
    for arch in selected_arches(ctx) {
        check_geninc_arch(ctx, arch, rows)?;
    }
    Ok(())
}

fn read_coff_section(obj: &Path, wanted: &str) -> Result<Vec<u8>, String> {
    vibeos_codegen::asm::read_coff_section(&read_bytes(obj)?, wanted)
}
fn emit_ks(asmdef: &[u8]) -> Result<String, String> {
    vibeos_codegen::asm::emit_ks(asmdef)
}

fn check_cflags_arch(ctx: &BinaryContext, arch: &str, rows: &mut Vec<Row>) -> Result<(), String> {
    for target in ["blrtl", "libcntpr", "win32k_residue", "genincdata"] {
        let args = cmd_args(&["--stdout", "--target", target, "--arch", arch]);
        let value = write_json_command(&ctx.root, "tools/gen_cflags.py", &args)?;
        let name = format!("{target}.{arch}");
        if value.is_null() {
            add(rows, "cflags", name, SKIP, format!("no build-{arch}"));
            continue;
        }
        let expected = value
            .as_str()
            .ok_or("gen_cflags JSON output must be a string or null")?;
        let committed = ctx.root.join(format!(
            "sdk/lib/rust/reactos-link/cflags/{target}.{arch}.flags",
        ));
        if !committed.exists() {
            add(rows, "cflags", name, DRIFT, "committed copy MISSING");
        } else {
            add(
                rows,
                "cflags",
                name,
                if compare_text(&committed, expected)? {
                    OK
                } else {
                    DRIFT
                },
                "regen vs committed",
            );
        }
    }
    Ok(())
}

fn check_cflags(ctx: &BinaryContext, rows: &mut Vec<Row>) -> Result<(), String> {
    let arches = selected_arches(ctx);
    let mut generated = Vec::new();
    for arch in &arches {
        check_cflags_arch(ctx, arch, &mut generated)?;
    }
    for target in ["blrtl", "libcntpr", "win32k_residue", "genincdata"] {
        for arch in &arches {
            let name = format!("{target}.{arch}");
            let i = generated
                .iter()
                .position(|row| row.1 == name)
                .ok_or("missing cflags row")?;
            rows.push(generated.remove(i));
        }
    }
    Ok(())
}

fn check_res_arch(ctx: &BinaryContext, arch: &str, rows: &mut Vec<Row>) -> Result<String, String> {
    let recipes = vibeos_codegen::resources::committed_jobs(&ctx.root, arch)?;
    let messages = recipes.messages;
    let Some(jobs) = recipes.jobs else {
        add(
            rows,
            ".res",
            format!("*-{arch}"),
            SKIP,
            format!("no build-{arch}"),
        );
        return Ok(messages);
    };
    for raw in jobs {
        let stem = &raw.0;
        let tmp = TempDir::new()?;
        let job = match vibeos_codegen::resources::localize_job(&ctx.root, &raw, tmp.path()) {
            Ok(job) => job,
            Err(error) => {
                add(rows, ".res", format!("{stem}-{arch}"), SKIP, error);
                continue;
            }
        };
        let fresh = match vibeos_codegen::resources::execute(&job, tmp.path()) {
            Ok(bytes) => bytes,
            Err(error) => {
                add(rows, ".res", format!("{stem}-{arch}"), SKIP, error);
                continue;
            }
        };
        if !job.output.exists() {
            add(
                rows,
                ".res",
                format!("{stem}-{arch}"),
                DRIFT,
                "committed copy MISSING",
            );
            continue;
        }
        add(
            rows,
            ".res",
            format!("{stem}-{arch}"),
            if read_bytes(&job.output)? == fresh {
                OK
            } else {
                DRIFT
            },
            "regen vs committed",
        );
    }
    Ok(messages)
}

fn check_res(ctx: &BinaryContext, rows: &mut Vec<Row>) -> Result<String, String> {
    let mut messages = String::new();
    for arch in selected_arches(ctx) {
        messages += &check_res_arch(ctx, arch, rows)?;
    }
    Ok(messages)
}

fn check_bugcodes(ctx: &BinaryContext, rows: &mut Vec<Row>) -> Result<(), String> {
    let committed = ctx
        .root
        .join("sdk/lib/rust/reactos-link/committed-headers/bugcodes.h");
    let fresh = vibeos_codegen::messages::bugcodes_h(&ctx.root)?.into_bytes();
    if !committed.exists() {
        add(
            rows,
            "bugcodes.h",
            "committed",
            DRIFT,
            "committed copy MISSING",
        );
    } else {
        add(
            rows,
            "bugcodes.h",
            "gen vs committed",
            if read_bytes(&committed)? == fresh {
                OK
            } else {
                DRIFT
            },
            "self-host regen",
        );
    }
    for arch in selected_arches(ctx) {
        let gen = ctx
            .root
            .join(format!("build-{arch}/sdk/include/reactos/mc/bugcodes.h"));
        if gen.exists() {
            add(
                rows,
                "bugcodes.h",
                format!("vs {arch}"),
                if committed.exists() && read_bytes(&committed)? == read_bytes(&gen)? {
                    OK
                } else {
                    DRIFT
                },
                "byte-diff vs windmc",
            );
            break;
        }
    }
    Ok(())
}

fn check_gendib(ctx: &BinaryContext, rows: &mut Vec<Row>) -> Result<(), String> {
    let src = ctx.root.join("sdk/tools/gendib/gendib.c");
    let Some((arch, cmdir)) = selected_arches(ctx).into_iter().find_map(|arch| {
        let cmdir = ctx.root.join(format!("build-{arch}/win32ss/gdi/dib"));
        cmdir.join("dib8gen.c").exists().then_some((arch, cmdir))
    }) else {
        add(rows, "gendib", "*", SKIP, "no build dir");
        return Ok(());
    };
    if !src.exists() {
        add(rows, "gendib", arch, SKIP, "no build dir");
        return Ok(());
    }
    let cc = std::env::var("CC").unwrap_or_else(|_| "cc".to_owned());
    let tmp = TempDir::new()?;
    let tool = tmp.path().join("native-gendib");
    let compile = command(
        &ctx.root,
        &cc,
        &cmd_args(&["-O2", "-o", &tool.to_string_lossy(), &src.to_string_lossy()]),
        Some(&ctx.root),
    )?;
    if !compile.status.success() {
        add(rows, "gendib", arch, SKIP, "tool build failed");
        return Ok(());
    }
    let generated = command(
        &ctx.root,
        &tool.to_string_lossy(),
        &[tmp.path().to_string_lossy().into_owned()],
        Some(&ctx.root),
    )?;
    if !generated.status.success() {
        add(rows, "gendib", arch, SKIP, "gendib run failed");
        return Ok(());
    }
    let mut bad = Vec::new();
    for name in ["dib8gen.c", "dib16gen.c", "dib32gen.c"] {
        if read_bytes(&tmp.path().join(name))? != read_bytes(&cmdir.join(name))? {
            bad.push(name);
        }
    }
    add(
        rows,
        "gendib",
        arch,
        if bad.is_empty() { OK } else { DRIFT },
        if bad.is_empty() {
            "3 .c vs build dir".to_owned()
        } else {
            bad.join(",")
        },
    );
    Ok(())
}

fn first_obj(dir: &Path, filename: &str) -> Result<Option<PathBuf>, String> {
    if !dir.exists() {
        return Ok(None);
    }
    for entry in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            let candidate = path.join(filename);
            if candidate.is_file() {
                return Ok(Some(candidate));
            }
        }
    }
    Ok(None)
}
fn check_widl_arch(ctx: &BinaryContext, arch: &str, rows: &mut Vec<Row>) -> Result<(), String> {
    let nm = ctx.root.join(".build-tools/llvm-mingw/bin/llvm-nm");
    let py = python();
    let w2r = ctx.root.join("tools/widl_to_rust.py");
    for (stub, rs_template, c_template, obj_dir) in WIDL_STUBS {
        let src = ctx.root.join(fmt_arch(c_template, arch));
        let object_root = ctx.root.join(fmt_arch(obj_dir, arch));
        let obj = first_obj(&object_root, &format!("{stub}.c.obj"))?;
        let committed = ctx.root.join(fmt_arch(rs_template, arch));
        let Some(obj) = obj else {
            add(
                rows,
                "widl .rs",
                format!("{stub}.{arch}"),
                SKIP,
                "no build-dir _s.c/.obj",
            );
            continue;
        };
        if !src.exists() || !nm.exists() {
            add(
                rows,
                "widl .rs",
                format!("{stub}.{arch}"),
                SKIP,
                "no build-dir _s.c/.obj",
            );
            continue;
        }
        let tmp = TempDir::new()?;
        let dst = tmp.path().join("out.rs");
        let args = vec![
            w2r.to_string_lossy().into_owned(),
            src.to_string_lossy().into_owned(),
            "--symobj".into(),
            obj.to_string_lossy().into_owned(),
            "--arch".into(),
            arch.to_owned(),
            "--nm".into(),
            nm.to_string_lossy().into_owned(),
            "-o".into(),
            dst.to_string_lossy().into_owned(),
        ];
        let output = command(&ctx.root, &py, &args, Some(&ctx.root))?;
        if !output.status.success() || !dst.exists() {
            add(
                rows,
                "widl .rs",
                format!("{stub}.{arch}"),
                SKIP,
                "widl_to_rust replay failed",
            );
            continue;
        }
        if !committed.exists() {
            add(
                rows,
                "widl .rs",
                format!("{stub}.{arch}"),
                DRIFT,
                "committed copy MISSING",
            );
            continue;
        }
        add(
            rows,
            "widl .rs",
            format!("{stub}.{arch}"),
            if compare_text(
                &committed,
                &fs::read_to_string(&dst).map_err(|e| e.to_string())?,
            )? {
                OK
            } else {
                DRIFT
            },
            "regen vs committed",
        );
    }
    Ok(())
}
fn check_widl(ctx: &BinaryContext, rows: &mut Vec<Row>) -> Result<(), String> {
    let arches = selected_arches(ctx);
    let mut per_arch = Vec::new();
    for arch in &arches {
        let mut arch_rows = Vec::new();
        check_widl_arch(ctx, arch, &mut arch_rows)?;
        per_arch.push(arch_rows);
    }
    for (stub, _, _, _) in WIDL_STUBS {
        for (i, arch) in arches.iter().enumerate() {
            let name = format!("{stub}.{arch}");
            if let Some(index) = per_arch[i].iter().position(|row| row.1 == name) {
                rows.push(per_arch[i].remove(index));
            }
        }
    }
    Ok(())
}
fn check_layout(ctx: &BinaryContext, rows: &mut Vec<Row>) -> Result<(), String> {
    let missing: Vec<&str> = ARCHES
        .iter()
        .copied()
        .filter(|a| {
            !ctx.root
                .join(format!("build-{a}/sdk/include/psdk/ntdef.h"))
                .is_file()
        })
        .collect();
    if !missing.is_empty() {
        add(
            rows,
            "layout",
            "ntkrnl-sys",
            SKIP,
            format!("needs build-{}", missing.join(",")),
        );
        return Ok(());
    }
    for (name, rel) in [
        ("size_check", "sdk/lib/rust/ntkrnl-sys/verify_sizes.py"),
        ("offset_check", "tools/verify_offsets.py"),
    ] {
        let temporary = TempDir::new()?;
        let args = vec![
            ctx.root.join(rel).to_string_lossy().into_owned(),
            "--output-dir".into(),
            temporary.path().to_string_lossy().into_owned(),
        ];
        let output = command(&ctx.root, &python(), &args, Some(&ctx.root))?;
        let matches = output.status.success()
            && ARCHES.iter().all(|arch| {
                let filename = format!("{name}_{arch}.rs");
                match (
                    fs::read(temporary.path().join(&filename)),
                    fs::read(ctx.root.join("sdk/lib/rust/ntkrnl-sys/src").join(&filename)),
                ) {
                    (Ok(fresh), Ok(committed)) => fresh == committed,
                    _ => false,
                }
            });
        add(
            rows,
            "layout",
            name,
            if matches { OK } else { DRIFT },
            "C headers vs committed asserts",
        );
    }
    Ok(())
}

pub fn run(ctx: &BinaryContext) -> Result<CheckOutput, String> {
    let mut rows = Vec::new();
    check_geninc(ctx, &mut rows)?;
    check_cflags(ctx, &mut rows)?;
    let messages = check_res(ctx, &mut rows)?;
    check_widl(ctx, &mut rows)?;
    check_bugcodes(ctx, &mut rows)?;
    check_gendib(ctx, &mut rows)?;
    check_layout(ctx, &mut rows)?;
    let drift = rows.iter().filter(|row| row.2 == DRIFT).count();
    let checked = rows.iter().filter(|row| row.2 == OK).count();
    let skipped = rows.iter().filter(|row| row.2 == SKIP).count();
    let mut out = messages;
    for (kind, name, status, detail) in &rows {
        let mark = match *status {
            OK => "✅",
            DRIFT => "❌",
            _ => "··",
        };
        out.push_str(&format!(
            "  {mark} {status:5} {kind:9} {name:22} {detail}\n"
        ));
    }
    out.push_str(&format!(
        "\n  {checked} OK, {drift} DRIFT, {skipped} skipped (no build dir)\n"
    ));
    if drift > 0 {
        out.push_str("\nDRIFT — committed fallback(s) are STALE vs cmake. Regenerate:\n  cflags  → python3 tools/gen_cflags.py [--arch …]\n  .res    → python3 tools/gen_committed_res.py [--arch …]\n  geninc  → self-hosted; if it drifts, the genincdata flags are stale\n            (python3 tools/gen_cflags.py --target genincdata) or a\n            ks*.template.h / genincdata.c change altered the .asmdef\n  widl .rs→ reactos-link RPC replay (see cargo-drive-cmake-off-roadmap WS3)\n  layout  → sdk/lib/rust/ntkrnl-sys/generate_bindings.sh, install the staged\n            bindgen_*.rs, then verify_sizes.py + tools/verify_offsets.py\n");
        return Ok(CheckOutput::new(out, 1));
    }
    if checked == 0 {
        out.push_str(
            "\n(no build-<arch>/ configured — nothing checked; configure a build to verify)\n",
        );
    }
    Ok(CheckOutput::new(out, 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coff_section_and_asmdef_are_decoded() {
        let mut obj = vec![0u8; 20 + 40 + 64];
        obj[2..4].copy_from_slice(&1u16.to_le_bytes());
        obj[20..28].copy_from_slice(b".asmdef\0");
        obj[20 + 16..20 + 20].copy_from_slice(&64u32.to_le_bytes());
        obj[20 + 20..20 + 24].copy_from_slice(&60u32.to_le_bytes());
        obj[60] = 2;
        obj[61..66].copy_from_slice(b"Field");
        obj[116..124].copy_from_slice(&0x2au64.to_le_bytes());
        let path = std::env::temp_dir().join(format!("drift-coff-{}.obj", std::process::id()));
        fs::write(&path, obj).unwrap();
        assert_eq!(read_coff_section(&path, ".asmdef").unwrap().len(), 64);
        assert_eq!(
            emit_ks(&read_coff_section(&path, ".asmdef").unwrap()).unwrap(),
            "Field = 0x2a\n"
        );
        fs::remove_file(path).unwrap();
    }
    #[test]
    fn missing_asmdef_is_an_error() {
        let path = std::env::temp_dir().join(format!("drift-empty-{}.obj", std::process::id()));
        fs::write(&path, vec![0u8; 20]).unwrap();
        assert!(read_coff_section(&path, ".asmdef")
            .unwrap_err()
            .contains("not in"));
        fs::remove_file(path).unwrap();
    }
}

pub fn layout(ctx: &BinaryContext) -> Result<CheckOutput, String> {
    let mut rows = Vec::new();
    check_layout(ctx, &mut rows)?;
    let mut stdout = String::new();
    let mut failure = false;
    for (kind, name, status, detail) in rows {
        stdout += &format!("{status}: {kind} {name}: {detail}\n");
        failure |= status == DRIFT;
    }
    Ok(CheckOutput::new(stdout, i32::from(failure)))
}
