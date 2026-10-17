//! Reject COFF resource objects that encode a predefined RT_* type as a string name.
use crate::{Corpus, Rule, SourceSet};
use std::collections::BTreeMap;

pub struct ResNamedTypes;
const INPUTS: &[SourceSet] = &[SourceSet::tracked("res-named-types", &["res"], &[]).with_inputs()];
impl Rule for ResNamedTypes {
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn name(&self) -> &'static str {
        "scan_res_named_types"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        // The shared loader supplies this tracked binary source set; rule code never opens files.
        for file in corpus.sources("res-named-types") {
            let rel = file.path.to_string_lossy();
            match scan_bytes(file.bytes()) {
                Ok(names) => {
                    for name in names {
                        let id = predefined()[name.to_uppercase().as_str()];
                        out.push(format!("{rel}: [NAMED_RT_TYPE] resource type is the STRING \"{name}\", not RT id {id} -- regenerate it (tools/gen_*_res.py)"));
                    }
                }
                Err(e) => out.push(format!("{rel}: [UNREADABLE] {e}")),
            }
        }
        Ok(out)
    }
}
fn predefined() -> &'static BTreeMap<&'static str, u16> {
    static M: std::sync::OnceLock<BTreeMap<&'static str, u16>> = std::sync::OnceLock::new();
    M.get_or_init(|| {
        [
            ("CURSOR", 1),
            ("BITMAP", 2),
            ("ICON", 3),
            ("MENU", 4),
            ("DIALOG", 5),
            ("STRING", 6),
            ("STRINGTABLE", 6),
            ("FONTDIR", 7),
            ("FONT", 8),
            ("ACCELERATORS", 9),
            ("ACCELERATOR", 9),
            ("RCDATA", 10),
            ("MESSAGETABLE", 11),
            ("GROUP_CURSOR", 12),
            ("GROUP_ICON", 14),
            ("VERSIONINFO", 16),
            ("VERSION", 16),
            ("DLGINCLUDE", 17),
            ("PLUGPLAY", 19),
            ("VXD", 20),
            ("ANICURSOR", 21),
            ("ANIICON", 22),
            ("HTML", 23),
            ("MANIFEST", 24),
        ]
        .into_iter()
        .collect()
    })
}
fn u16le(b: &[u8], at: usize) -> Result<u16, String> {
    let x=b.get(at..at+2).ok_or_else(||format!("unpack_from requires a buffer of at least {} bytes for unpacking 2 bytes at offset {} (actual buffer size is {})",at+2,at,b.len()))?;
    Ok(u16::from_le_bytes([x[0], x[1]]))
}
fn u32le(b: &[u8], at: usize) -> Result<u32, String> {
    let x=b.get(at..at+4).ok_or_else(||format!("unpack_from requires a buffer of at least {} bytes for unpacking 4 bytes at offset {} (actual buffer size is {})",at+4,at,b.len()))?;
    Ok(u32::from_le_bytes([x[0], x[1], x[2], x[3]]))
}
fn rsrc_section(buf: &[u8]) -> Result<&[u8], String> {
    let nsect = u16le(buf, 2)? as usize;
    let opt = u16le(buf, 16)? as usize;
    let base = 20usize.checked_add(opt).ok_or("section offset overflow")?;
    for i in 0..nsect {
        let h = base
            .checked_add(40usize.checked_mul(i).ok_or("section offset overflow")?)
            .ok_or("section offset overflow")?;
        let name=buf.get(h..h+8).ok_or_else(||format!("unpack_from requires a buffer of at least {} bytes for unpacking 40 bytes at offset {} (actual buffer size is {})",h+40,h,buf.len()))?;
        let end = name.iter().position(|b| *b == 0).unwrap_or(8);
        let name = String::from_utf8_lossy(&name[..end]);
        let size = u32le(buf, h + 16)? as usize;
        let ptr = u32le(buf, h + 20)? as usize;
        if name == ".rsrc" || name == ".rsrc$01" {
            let end = ptr
                .checked_add(size)
                .ok_or("resource section offset overflow")?;
            return buf.get(ptr..end).ok_or_else(|| {
                format!(
                    "resource section extends past object: {ptr}..{end} (size {})",
                    buf.len()
                )
            });
        }
    }
    Err("no .rsrc section".into())
}
fn resource_types(buf: &[u8]) -> Result<Vec<StringOrId>, String> {
    let r = rsrc_section(buf)?;
    let named = u16le(r, 12)? as usize;
    let ids = u16le(r, 14)? as usize;
    let mut out = Vec::new();
    for i in 0..named + ids {
        let at = 16 + 8 * i;
        let name = u32le(r, at)?;
        if name & 0x80000000 != 0 {
            let off = (name & 0x7fffffff) as usize;
            let len = u16le(r, off)? as usize;
            let bytes = r
                .get(off + 2..off + 2 + 2 * len)
                .ok_or_else(|| "resource type name extends past section".to_owned())?;
            let units = bytes
                .chunks_exact(2)
                .map(|x| u16::from_le_bytes([x[0], x[1]]))
                .collect::<Vec<_>>();
            out.push(StringOrId::Name(
                String::from_utf16(&units)
                    .map_err(|e| format!("invalid UTF-16 resource type: {e}"))?,
            ));
        } else {
            out.push(StringOrId::Id((name & 0xffff) as u16));
        }
    }
    Ok(out)
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum StringOrId {
    Name(String),
    Id(u16),
}
fn scan_bytes(buf: &[u8]) -> Result<Vec<String>, String> {
    let mut names = BTreeMap::<String, ()>::new();
    for t in resource_types(buf)? {
        if let StringOrId::Name(s) = t {
            if predefined().contains_key(s.to_uppercase().as_str()) {
                names.insert(s, ());
            }
        }
    }
    Ok(names.into_keys().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(types: &[StringOrId]) -> Vec<u8> {
        let named = types
            .iter()
            .filter_map(|x| {
                if let StringOrId::Name(s) = x {
                    Some(s)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let ids = types
            .iter()
            .filter_map(|x| {
                if let StringOrId::Id(n) = x {
                    Some(*n)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let end = 16 + 8 * types.len();
        let mut entries = Vec::new();
        let mut strings = Vec::new();
        for n in &named {
            let off = end + strings.len();
            entries.extend_from_slice(&((0x80000000u32 | off as u32).to_le_bytes()));
            entries.extend_from_slice(&0u32.to_le_bytes());
            let utf: nVec = n.encode_utf16().collect();
            strings.extend_from_slice(&(utf.len() as u16).to_le_bytes());
            for c in utf {
                strings.extend_from_slice(&c.to_le_bytes());
            }
        }
        for id in ids {
            entries.extend_from_slice(&(id as u32).to_le_bytes());
            entries.extend_from_slice(&0u32.to_le_bytes());
        }
        let mut r = Vec::new();
        r.extend_from_slice(&0u32.to_le_bytes());
        r.extend_from_slice(&0u32.to_le_bytes());
        r.extend_from_slice(&0u16.to_le_bytes());
        r.extend_from_slice(&0u16.to_le_bytes());
        r.extend_from_slice(&(named.len() as u16).to_le_bytes());
        r.extend_from_slice(&((types.len() - named.len()) as u16).to_le_bytes());
        r.extend(entries);
        r.extend(strings);
        let sectptr = 60u32;
        let mut b = Vec::new();
        b.extend_from_slice(&0x14cu16.to_le_bytes());
        b.extend_from_slice(&1u16.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&0u16.to_le_bytes());
        b.extend_from_slice(&0u16.to_le_bytes());
        b.extend_from_slice(b".rsrc$01");
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&(r.len() as u32).to_le_bytes());
        b.extend_from_slice(&sectptr.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&0u16.to_le_bytes());
        b.extend_from_slice(&0u16.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend(r);
        b
    }
    #[test]
    fn string_messagetable_reports_but_integer_and_custom_types_do_not() {
        let b = fixture(&[StringOrId::Name("MESSAGETABLE".into()), StringOrId::Id(16)]);
        assert_eq!(scan_bytes(&b).unwrap(), vec!["MESSAGETABLE"]);
        assert!(scan_bytes(&fixture(&[
            StringOrId::Id(2),
            StringOrId::Id(11),
            StringOrId::Id(16)
        ]))
        .unwrap()
        .is_empty());
        assert!(scan_bytes(&fixture(&[
            StringOrId::Name("WAVE".into()),
            StringOrId::Id(16)
        ]))
        .unwrap()
        .is_empty());
    }
    #[test]
    fn resource_types_preserve_directory_order() {
        let b = fixture(&[
            StringOrId::Name("MESSAGETABLE".into()),
            StringOrId::Id(2),
            StringOrId::Id(16),
        ]);
        assert_eq!(
            resource_types(&b).unwrap(),
            vec![
                StringOrId::Name("MESSAGETABLE".into()),
                StringOrId::Id(2),
                StringOrId::Id(16)
            ]
        );
    }
}
type nVec = Vec<u16>;
