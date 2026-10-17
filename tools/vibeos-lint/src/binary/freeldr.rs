use super::{BinaryContext, CheckOutput};
use regex::Regex;
use std::fs;
pub fn limit(text: &str) -> Result<i64, String> {
    let defs = Regex::new(r"#define\s+(\w+)\s+HEX\(([0-9A-Fa-f]+)\)")
        .unwrap()
        .captures_iter(text)
        .map(|m| (m[1].to_string(), i64::from_str_radix(&m[2], 16).unwrap()))
        .collect::<std::collections::HashMap<_, _>>();
    let re = Regex::new(r"#define\s+MAX_FREELDR_PE_SIZE\s+\(([^)]*)\)").unwrap();
    let expr = re
        .captures(text)
        .ok_or("x86common.h no longer defines MAX_FREELDR_PE_SIZE as expected")?;
    if expr[1].split_whitespace().collect::<Vec<_>>()
        != ["MEMORY_MARGIN", "-", "FREELDR_PE_BASE", "-", "PAGE_SIZE"]
    {
        return Err(format!(
            "MAX_FREELDR_PE_SIZE changed shape: {:?}; update this tool",
            &expr[1]
        ));
    }
    Ok(defs.get("MEMORY_MARGIN").ok_or("missing MEMORY_MARGIN")?
        - defs
            .get("FREELDR_PE_BASE")
            .ok_or("missing FREELDR_PE_BASE")?
        - 0x1000)
}
pub fn u32_at(data: &[u8], off: usize) -> Result<u32, String> {
    Ok(u32::from_le_bytes(
        data.get(off..off.checked_add(4).ok_or("PE offset overflow")?)
            .ok_or("truncated PE")?
            .try_into()
            .unwrap(),
    ))
}
pub fn size(data: &[u8]) -> Result<u32, String> {
    if data.get(..2) != Some(b"MZ") {
        return Err("not a PE (no MZ)".into());
    }
    let pe = u32_at(data, 0x3c)? as usize;
    if data.get(pe..pe + 4) != Some(b"PE\0\0") {
        return Err("not a PE (no PE signature)".into());
    }
    u32_at(data, pe + 24 + 56)
}
pub fn run(ctx: &BinaryContext) -> Result<CheckOutput, String> {
    let pe = ctx.build_dir.join("boot/freeldr/freeldr/freeldr_pe.exe");
    if !pe.is_file() {
        return Ok(CheckOutput::new(
            format!(
                "SKIP: {} not built (no BIOS loader on this arch)\n",
                pe.display()
            ),
            0,
        ));
    }
    let limit = limit(
        &fs::read_to_string(
            ctx.root
                .join("boot/freeldr/freeldr/include/arch/pc/x86common.h"),
        )
        .map_err(|e| e.to_string())?,
    )?;
    let size = size(&fs::read(pe).map_err(|e| e.to_string())?)?;
    let room = limit - i64::from(size);
    let msg =
        format!("freeldr_pe SizeOfImage 0x{size:x} ({size}) vs MAX_FREELDR_PE_SIZE 0x{limit:x}");
    Ok(if room < 0 {
        CheckOutput::new(format!("FAIL: {msg} -- {} bytes OVER: every BIOS boot bug-checks in\n  MmCheckFreeldrImageFile. Shrink the BIOS image (freeldr-rust without\n  `uefi`/`rosload` is the one with the budget).\n",-room),1)
    } else {
        CheckOutput::new(
            format!(
                "{}: {msg} -- {room} bytes of headroom\n",
                if room < 0x3000 { "WARN" } else { "OK" }
            ),
            0,
        )
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn budget_shape() {
        let hdr="#define FREELDR_PE_BASE HEX(10000)\n#define MEMORY_MARGIN HEX(88000)\n#define MAX_FREELDR_PE_SIZE (MEMORY_MARGIN - FREELDR_PE_BASE - PAGE_SIZE)";
        assert_eq!(limit(hdr).unwrap(), 0x77000);
        assert!(limit(&hdr.replace("- PAGE_SIZE", "- PAGE_SIZE * 2")).is_err());
    }
    #[test]
    fn header_and_truncation() {
        let mut pe = vec![0; 512];
        pe[..2].copy_from_slice(b"MZ");
        pe[0x3c..0x40].copy_from_slice(&0x80u32.to_le_bytes());
        pe[0x80..0x84].copy_from_slice(b"PE\0\0");
        pe[0xd0..0xd4].copy_from_slice(&0x70200u32.to_le_bytes());
        assert_eq!(size(&pe).unwrap(), 0x70200);
        assert!(size(&pe[..0x90]).is_err());
    }
}
