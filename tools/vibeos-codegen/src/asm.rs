use std::path::Path;
use std::process::Command;

pub fn read_coff_section(raw: &[u8], wanted: &str) -> Result<Vec<u8>, String> {
    if raw.len() < 20 {
        return Err("truncated COFF header".into());
    }
    let n = u16::from_le_bytes([raw[2], raw[3]]) as usize;
    let optional = u16::from_le_bytes([raw[16], raw[17]]) as usize;
    for i in 0..n {
        let at = 20 + optional + i * 40;
        let row = raw.get(at..at + 40).ok_or("truncated COFF section table")?;
        let end = row[..8].iter().position(|b| *b == 0).unwrap_or(8);
        if &row[..end] != wanted.as_bytes() {
            continue;
        }
        let len = u32::from_le_bytes(row[16..20].try_into().unwrap()) as usize;
        let start = u32::from_le_bytes(row[20..24].try_into().unwrap()) as usize;
        if len == 0 || start == 0 {
            return Err(format!("section {wanted} has no raw data"));
        }
        return raw
            .get(start..start.checked_add(len).ok_or("COFF section overflow")?)
            .map(|s| s.to_vec())
            .ok_or("truncated COFF section data".into());
    }
    Err(format!("section {wanted} not in COFF object"))
}

pub fn emit_ks(asmdef: &[u8]) -> Result<String, String> {
    let mut out = String::new();
    for (i, row) in asmdef.chunks_exact(64).enumerate() {
        let end = row[1..56].iter().position(|b| *b == 0).unwrap_or(55);
        let name = std::str::from_utf8(&row[1..end + 1]).map_err(|e| e.to_string())?;
        let value = u64::from_le_bytes(row[56..].try_into().unwrap());
        match row[0] {
            0 => break,
            1 => out += &format!("{name}\n"),
            2 => out += &format!("{name} = 0x{value:x}\n"),
            3 => out += &format!("\n/* {name} */\n"),
            n => return Err(format!("unknown ASMGENDATA type {n} at offset {}", i * 64)),
        }
    }
    Ok(out)
}

pub fn ks386(root: &Path) -> Result<String, String> {
    let work = crate::resources::TempDir::new()?;
    let mc = work.path().join("mc-stage");
    std::fs::create_dir(&mc).map_err(|e| e.to_string())?;
    let bugcodes = root.join("build-i386/sdk/include/reactos/mc/bugcodes.h");
    let local = std::fs::canonicalize(root).map_err(|e| e.to_string())?;
    if !std::fs::canonicalize(&bugcodes)
        .map_err(|e| format!("{}: {e}", bugcodes.display()))?
        .starts_with(&local)
    {
        return Err("ks386 generated header resolves outside checkout".into());
    }
    for include in ["build-i386/sdk/include/ddk", "build-i386/sdk/include/psdk"] {
        let dir = root.join(include);
        if !dir
            .canonicalize()
            .map_err(|e| format!("{}: {e}", dir.display()))?
            .starts_with(&local)
        {
            return Err(format!(
                "ks386 generated include escapes checkout: {include}"
            ));
        }
    }
    std::fs::copy(bugcodes, mc.join("bugcodes.h")).map_err(|e| e.to_string())?;
    let recipe: Vec<String> =
        serde_json::from_str(include_str!("ks386_command.json")).expect("ks386 command template");
    let argv: Vec<_> = recipe
        .iter()
        .map(|s| {
            s.replace("{ROOT}", &root.to_string_lossy())
                .replace("{WORK}", &work.path().to_string_lossy())
        })
        .collect();
    let result = Command::new(&argv[0])
        .args(&argv[1..])
        .current_dir(root)
        .output()
        .map_err(|e| e.to_string())?;
    if !result.status.success() {
        return Err(format!(
            "ks386 compiler failed: {}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    let obj = std::fs::read(work.path().join("genincdata.obj"))
        .map_err(|e| format!("ks386 produced no object: {e}"))?;
    emit_ks(&read_coff_section(&obj, ".asmdef")?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn constants_headers_and_unknown_records() {
        let mut bytes = vec![0u8; 256];
        bytes[0] = 3;
        bytes[1..7].copy_from_slice(b"header");
        bytes[64] = 2;
        bytes[65..69].copy_from_slice(b"FLAG");
        bytes[120..128].copy_from_slice(&17u64.to_le_bytes());
        bytes[128] = 1;
        bytes[129..132].copy_from_slice(b"raw");
        assert_eq!(
            emit_ks(&bytes).unwrap(),
            "\n/* header */\nFLAG = 0x11\nraw\n"
        );
        bytes[128] = 9;
        assert!(emit_ks(&bytes).is_err());
        assert!(read_coff_section(&[0u8; 19], ".asmdef").is_err());
    }
}
