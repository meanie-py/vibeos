//! llvm-mingw delay-import descriptors, architecture thunks, and archive construction.
use std::fmt::Write;
use std::path::Path;
use std::process::Command;

pub struct Export {
    pub name: String,
    pub data: bool,
    pub ordinal: Option<u32>,
}
pub fn triple(arch: &str) -> Result<&'static str, String> {
    match arch {
        "i386" => Ok("i686-w64-windows-gnu"),
        "amd64" => Ok("x86_64-w64-windows-gnu"),
        "arm64" => Ok("aarch64-w64-windows-gnu"),
        _ => Err(format!("unknown delaylib architecture: {arch}")),
    }
}
pub fn parse_def(source: &str) -> Result<(String, Vec<Export>), String> {
    let mut dll = None;
    let mut exports = Vec::new();
    let mut inside = false;
    let mut seen = std::collections::HashSet::new();
    for line in source.lines() {
        let line = line.split(';').next().unwrap().trim();
        let parts: Vec<_> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        if ["NAME", "LIBRARY"].contains(&parts[0].to_ascii_uppercase().as_str()) {
            dll = Some(parts.get(1).ok_or("missing DLL name")?.to_string());
            continue;
        }
        if parts[0].eq_ignore_ascii_case("EXPORTS") {
            inside = true;
            continue;
        }
        if !inside || parts[0].starts_with('@') {
            continue;
        }
        let name = parts[0].split('=').next().unwrap().to_string();
        if !seen.insert(name.clone()) {
            continue;
        }
        let ordinal = if parts[1..].contains(&"NONAME") {
            let value = parts[1..]
                .iter()
                .find_map(|s| {
                    s.strip_prefix('@')
                        .filter(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()))
                })
                .ok_or_else(|| format!("NONAME export {name} has no @ordinal"))?;
            Some(value.parse::<u32>().map_err(|_| "invalid export ordinal")?)
        } else {
            None
        };
        exports.push(Export {
            name,
            data: parts[1..].contains(&"DATA"),
            ordinal,
        });
    }
    Ok((dll.ok_or("no NAME/LIBRARY line")?, exports))
}
fn q(s: &str) -> String {
    if s.contains('@') {
        format!("\"{s}\"")
    } else {
        s.into()
    }
}
pub fn emit(source: &str, arch: &str) -> Result<String, String> {
    triple(arch)?;
    let (dll, exports) = parse_def(source)?;
    let functions: Vec<_> = exports.iter().filter(|e| !e.data).collect();
    let datas: Vec<_> = exports
        .iter()
        .filter(|e| e.data)
        .map(|e| e.name.as_str())
        .collect();
    let desc = format!(
        "___DELAY_DESC_{}",
        dll.chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            })
            .collect::<String>()
    );
    let ptrdir = if arch == "i386" { ".long" } else { ".quad" };
    let align = if arch == "i386" { 2 } else { 3 };
    let mut out = format!("\t.section .rdata,\"dr\"\n\t.p2align 2\n\t.rva __vibeos_delayimp_head\n\t.rva __vibeos_delayimp_tail\n__dll_name: .asciz \"{dll}\"\n");
    for (i, e) in functions.iter().enumerate() {
        if e.ordinal.is_none() {
            writeln!(
                out,
                "\t.p2align 1\n__hn_{i}: .short 0\n\t.asciz \"{}\"",
                e.name.split('@').next().unwrap()
            )
            .unwrap();
        }
    }
    writeln!(out, "\t.p2align {align}\n__delay_int:").unwrap();
    for (i, e) in functions.iter().enumerate() {
        match e.ordinal {
            Some(n) if arch == "i386" => writeln!(out, "\t.long {:#x}", 0x80000000u32 | n).unwrap(),
            Some(n) => writeln!(out, "\t.long {n:#x}\n\t.long 0x80000000").unwrap(),
            None => {
                writeln!(out, "\t.rva __hn_{i}").unwrap();
                if arch != "i386" {
                    out.push_str("\t.long 0\n");
                }
            }
        }
    }
    out.push_str("\t.long 0\n");
    if arch != "i386" {
        out.push_str("\t.long 0\n");
    }
    writeln!(
        out,
        "\t.section .data,\"dw\"\n\t.p2align {align}\n__delay_iat:"
    )
    .unwrap();
    let imp = |s: &str| {
        q(&format!(
            "{}{}",
            if arch == "i386" { "__imp__" } else { "__imp_" },
            s
        ))
    };
    for (i, e) in functions.iter().enumerate() {
        let symbol = imp(&e.name);
        writeln!(out, "\t.globl {symbol}\n{symbol}:\n\t{ptrdir} __thunk_{i}").unwrap();
    }
    writeln!(out, "\t{ptrdir} 0\n__delay_hmod:\n\t{ptrdir} 0").unwrap();
    for (suffix, symbol, body) in [
        (
            "A",
            "__vibeos_delayimp_head",
            "\t.ascii \"VIBEOS-DLYIMP-01\"",
        ),
        ("C", "__vibeos_delayimp_tail", "\t.zero 32"),
    ] {
        writeln!(out,"\t.section .CRT$VBDLYIMP{suffix},\"dr\",discard,{symbol}\n\t.p2align 2\n\t.globl {symbol}\n{symbol}:\n{body}").unwrap();
    }
    writeln!(out,"\t.section .CRT$VBDLYIMPB,\"dr\"\n\t.p2align 2\n\t.globl {desc}\n{desc}:\n\t.long 1\n\t.rva __dll_name\n\t.rva __delay_hmod\n\t.rva __delay_iat\n\t.rva __delay_int\n\t.long 0\n\t.long 0\n\t.long 0\n\t.section .text,\"xr\"").unwrap();
    for (i, e) in functions.iter().enumerate() {
        let public = q(&format!(
            "{}{}",
            if arch == "i386" { "_" } else { "" },
            e.name
        ));
        let symbol = imp(&e.name);
        writeln!(out, "\t.globl {public}\n{public}:").unwrap();
        match arch {
            "i386" => writeln!(out, "\tjmp  *{symbol}").unwrap(),
            "amd64" => writeln!(out, "\tjmp  *{symbol}(%rip)").unwrap(),
            _ => writeln!(
                out,
                "\tadrp x16, {symbol}\n\tldr  x16, [x16, :lo12:{symbol}]\n\tbr   x16"
            )
            .unwrap(),
        }
        writeln!(out, "__thunk_{i}:").unwrap();
        let template = match arch {
            "i386" => include_str!("delay_thunk_i386.txt"),
            "amd64" => include_str!("delay_thunk_amd64.txt"),
            _ => include_str!("delay_thunk_arm64.txt"),
        };
        out.push_str(&template.replace("{IMP}", &symbol).replace("{DESC}", &desc));
    }
    if !datas.is_empty() {
        writeln!(
            out,
            "\t/* NOTE: {} DATA export(s) not delay-bound: {} */",
            datas.len(),
            datas.join(", ")
        )
        .unwrap();
    }
    Ok(out)
}

pub fn archive(
    asm: &str,
    arch: &str,
    output: &Path,
    clang: &str,
    ar: &str,
) -> Result<Vec<u8>, String> {
    let work = crate::resources::TempDir::new()?;
    let stem = output
        .file_stem()
        .ok_or("delay archive needs a file name")?
        .to_string_lossy();
    let source = work.path().join(format!("{stem}.delay.s"));
    let obj = work.path().join(format!("{stem}.delay.o"));
    let library = work.path().join(output.file_name().unwrap());
    std::fs::write(&source, asm).map_err(|e| e.to_string())?;
    let result = Command::new(clang)
        .args([
            format!("--target={}", triple(arch)?),
            "-c".into(),
            "-o".into(),
        ])
        .arg(&obj)
        .arg(&source)
        .output()
        .map_err(|e| format!("cannot run {clang}: {e}"))?;
    if !result.status.success() {
        return Err(format!(
            "delaylib compiler failed: {}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    if !obj.is_file() {
        return Err("delaylib compiler produced no object".into());
    }
    let result = Command::new(ar)
        .arg("crs")
        .arg(&library)
        .arg(&obj)
        .output()
        .map_err(|e| format!("cannot run {ar}: {e}"))?;
    if !result.status.success() {
        return Err(format!(
            "delaylib archiver failed: {}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    let bytes = std::fs::read(&library)
        .map_err(|e| format!("delaylib archiver produced no archive: {e}"))?;
    if bytes.is_empty() {
        return Err("delaylib archiver produced an empty archive".into());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordinal_imports_never_emit_a_name() {
        let text="LIBRARY browseui.dll\nEXPORTS\nByName @101\nSHOnCWMCommandLine @127 NONAME\nData @130 DATA\n";
        for arch in ["i386", "amd64", "arm64"] {
            let out = emit(text, arch).unwrap();
            assert!(out.contains(".asciz \"ByName\""));
            assert!(!out.contains(".asciz \"SHOnCWMCommandLine\""));
            assert!(out.contains(if arch == "i386" {
                "\t.long 0x8000007f"
            } else {
                "\t.long 0x7f\n\t.long 0x80000000"
            }));
        }
        assert!(parse_def("LIBRARY a.dll\nEXPORTS\nBroken NONAME\n").is_err());
    }
}
