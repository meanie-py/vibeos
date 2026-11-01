//! Cross-compiler probes transported through bounded COFF reads, never host ABI guesses.
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub struct Recipe {
    pub arch: String,
    pub includes: Vec<String>,
    pub include_dirs: Vec<String>,
    pub defines: Vec<String>,
    pub expressions: Vec<(String, String)>,
}
fn u32_at(bytes: &[u8], at: usize) -> Result<u32, String> {
    Ok(u32::from_le_bytes(
        bytes
            .get(at..at.checked_add(4).ok_or("COFF offset overflow")?)
            .ok_or("truncated COFF integer")?
            .try_into()
            .unwrap(),
    ))
}
fn c_string(bytes: &[u8]) -> Result<String, String> {
    let end = bytes
        .iter()
        .position(|b| *b == 0)
        .ok_or("unterminated COFF name")?;
    std::str::from_utf8(&bytes[..end])
        .map(str::to_owned)
        .map_err(|e| e.to_string())
}

/// Symbol name -> (section number, byte offset), including defined static data.
pub fn symbols(bytes: &[u8]) -> Result<BTreeMap<String, (i16, u32)>, String> {
    if bytes.len() < 20 {
        return Err("truncated COFF header".into());
    }
    let start = u32_at(bytes, 8)? as usize;
    let count = u32_at(bytes, 12)? as usize;
    let end = start
        .checked_add(count.checked_mul(18).ok_or("COFF symbol count overflow")?)
        .ok_or("COFF symbol offset overflow")?;
    if start < 20 || end > bytes.len() {
        return Err("truncated COFF symbol table".into());
    }
    let string_len = u32_at(bytes, end)? as usize;
    let strings = bytes
        .get(end..end.checked_add(string_len).ok_or("COFF strings overflow")?)
        .filter(|_| string_len >= 4)
        .ok_or("truncated COFF strings")?;
    let mut result = BTreeMap::new();
    let mut i = 0;
    while i < count {
        let row = &bytes[start + i * 18..start + (i + 1) * 18];
        let name = if row[..4] == [0; 4] {
            let off = u32_at(row, 4)? as usize;
            if off < 4 {
                return Err("invalid COFF string offset".into());
            }
            c_string(strings.get(off..).ok_or("COFF name outside string table")?)?
        } else {
            let end = row[..8].iter().position(|b| *b == 0).unwrap_or(8);
            std::str::from_utf8(&row[..end])
                .map_err(|e| e.to_string())?
                .to_owned()
        };
        let section = i16::from_le_bytes(row[12..14].try_into().unwrap());
        if section > 0 {
            result.insert(name, (section, u32_at(row, 8)?));
        }
        i = i
            .checked_add(1 + row[17] as usize)
            .filter(|n| *n <= count)
            .ok_or("COFF auxiliary symbols exceed table")?;
    }
    Ok(result)
}
fn section_name(bytes: &[u8], number: i16) -> Result<String, String> {
    if number <= 0
        || number as usize
            > u16::from_le_bytes(
                bytes
                    .get(2..4)
                    .ok_or("truncated COFF header")?
                    .try_into()
                    .unwrap(),
            ) as usize
    {
        return Err("invalid COFF symbol section".into());
    }
    let optional = u16::from_le_bytes(
        bytes
            .get(16..18)
            .ok_or("truncated COFF header")?
            .try_into()
            .unwrap(),
    ) as usize;
    let at = 20 + optional + (number as usize - 1) * 40;
    let name = bytes.get(at..at + 8).ok_or("truncated COFF section name")?;
    let end = name.iter().position(|b| *b == 0).unwrap_or(8);
    std::str::from_utf8(&name[..end])
        .map(str::to_owned)
        .map_err(|e| e.to_string())
}
pub fn values(bytes: &[u8], names: &[(String, String)]) -> Result<BTreeMap<String, i64>, String> {
    let symbols = symbols(bytes)?;
    let (section, offset) = symbols
        .get("__harvest")
        .or_else(|| symbols.get("___harvest"))
        .ok_or("COFF probe lacks __harvest symbol")?;
    let name = section_name(bytes, *section)?;
    if name != ".rdata" {
        return Err(format!("__harvest is in {name}, expected .rdata"));
    }
    let data = crate::asm::read_coff_section(bytes, &name)?;
    let start = *offset as usize;
    let len = names.len().checked_mul(8).ok_or("probe size overflow")?;
    let data = data
        .get(start..start.checked_add(len).ok_or("probe offset overflow")?)
        .ok_or("truncated __harvest values")?;
    let mut result = BTreeMap::new();
    for ((key, _), slot) in names.iter().zip(data.chunks_exact(8)) {
        if result
            .insert(key.clone(), i64::from_le_bytes(slot.try_into().unwrap()))
            .is_some()
        {
            return Err(format!("duplicate probe key: {key}"));
        }
    }
    Ok(result)
}
pub fn source(recipe: &Recipe) -> String {
    let mut text = recipe
        .includes
        .iter()
        .map(|h| {
            if h.starts_with('"') {
                format!("#include {h}\n")
            } else {
                format!("#include <{h}>\n")
            }
        })
        .collect::<String>();
    text += "__attribute__((used)) const long long __harvest[] = {\n";
    for (_, expr) in &recipe.expressions {
        text += &format!("  (long long)({expr}),\n")
    }
    text += "};\n";
    text
}
pub fn local(root: &Path, path: &Path) -> Result<PathBuf, String> {
    let root = root
        .canonicalize()
        .map_err(|e| format!("{}: {e}", root.display()))?;
    let path = path
        .canonicalize()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    if !path.starts_with(&root) {
        return Err(format!("probe input escapes checkout: {}", path.display()));
    }
    Ok(path)
}
pub fn harvest(root: &Path, recipe: &Recipe) -> Result<BTreeMap<String, i64>, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let triple = match recipe.arch.as_str() {
        "i386" => "i686-w64-mingw32",
        "amd64" => "x86_64-w64-mingw32",
        "arm64" => "aarch64-w64-mingw32",
        _ => return Err(format!("invalid probe arch: {}", recipe.arch)),
    };
    let build = root.join(format!("build-{}", recipe.arch));
    local(&root, &build.join("sdk/include/reactos")).map_err(|e| {
        format!(
            "{e}; prepare this checkout: ./vibeos-cli build --arch {} --headers-only",
            recipe.arch
        )
    })?;
    let temp = crate::resources::TempDir::new()?;
    let input = temp.path().join("probe.c");
    let object = temp.path().join("probe.obj");
    fs::write(&input, source(recipe)).map_err(|e| e.to_string())?;
    let clang = root.join(".build-tools/llvm-mingw/bin/clang");
    let mut command = Command::new(&clang);
    command
        .args(["-target", triple, "-c", "-o"])
        .arg(&object)
        .arg(&input)
        .args(["-nostdinc", "-w"]);
    for dir in &recipe.include_dirs {
        let path = root.join(dir);
        // Nonexistent optional include directories match clang's normal behavior;
        // existing generated directories must belong to this checkout.
        if dir.starts_with("build-") && path.exists() {
            local(&root, &path)?;
        }
        command.arg(format!("-I{}", path.display()));
    }
    for define in &recipe.defines {
        command.arg(format!("-D{define}"));
    }
    let result = command
        .output()
        .map_err(|e| format!("{}: {e}", clang.display()))?;
    if !result.status.success() {
        return Err(format!(
            "{} probe compiler failed ({}): {}",
            recipe.arch,
            result.status,
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    values(
        &fs::read(&object).map_err(|e| format!("compiler produced no probe object: {e}"))?,
        &recipe.expressions,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_truncated_and_inconsistent_coff() {
        assert!(symbols(&[]).is_err());
        let mut b = vec![0; 40];
        b[8..12].copy_from_slice(&20u32.to_le_bytes());
        b[12..16].copy_from_slice(&5u32.to_le_bytes());
        assert!(symbols(&b).is_err());
        assert!(values(&b, &[]).is_err());
    }
    #[test]
    fn probe_keeps_expression_and_include_order() {
        let r = Recipe {
            arch: "amd64".into(),
            includes: vec!["stdio.h".into(), "\"local.h\"".into()],
            include_dirs: vec![],
            defines: vec![],
            expressions: vec![
                ("x".into(), "sizeof(int)".into()),
                ("y".into(), "-1".into()),
            ],
        };
        assert_eq!(source(&r),"#include <stdio.h>\n#include \"local.h\"\n__attribute__((used)) const long long __harvest[] = {\n  (long long)(sizeof(int)),\n  (long long)(-1),\n};\n");
    }
}
