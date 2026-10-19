//! A Rust GUID literal whose name the SDK defines with a different value.
//!
//! A translation that cannot see a GUID's declaration writes the GUID itself, from memory.
//! stobject's model did that for shlguid.h's `CGID_ShellServiceObject` and wrote Data1
//! `0x000214D4` for `0x000214D2`: every `IOleCommandTarget::Exec` explorer sent it failed
//! the group check, so the tray thread never started and no volume, power or hotplug icon
//! appeared, with nothing failing loudly. The SDK's `DEFINE_GUID` / `DEFINE_OLEGUID` /
//! `DEFINE_SHLGUID` is the authority; a same-named Rust `GUID { Data1, Data2, Data3, Data4 }`
//! literal must match one of its definitions. Suppress a verified false positive (a module
//! that deliberately shadows an SDK name) with `// nolint: guid-values (reason)` on the line.
use crate::{Corpus, Rule, SourceSet};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

pub struct GuidValues;
impl Rule for GuidValues {
    fn name(&self) -> &'static str {
        "scan_guid_values"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        const SETS: &[SourceSet] = &[SourceSet::tree("guid-headers", &["sdk/include"], &["h"], &[])];
        SETS
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut sdk: BTreeMap<String, BTreeSet<Guid>> = BTreeMap::new();
        for file in corpus.sources("guid-headers") {
            for (name, g) in sdk_guids(&file.text) {
                sdk.entry(name).or_default().insert(g);
            }
        }
        let mut findings = Vec::new();
        for file in corpus.rust_files() {
            for (name, g, line) in rust_guids(&file.text) {
                let Some(known) = sdk.get(&name) else { continue };
                if known.contains(&g) || file.line_text(line).contains("nolint: guid-values") {
                    continue;
                }
                let want: Vec<String> = known.iter().map(Guid::show).collect();
                findings.push(format!(
                    "{}:{line}: {name} is {} here but the SDK defines it as {}; use the \
                     island's declaration instead of a hand-written literal",
                    file.path.display(),
                    g.show(),
                    want.join(" or ")
                ));
            }
        }
        findings.sort();
        Ok(findings)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Guid(u32, u16, u16, [u8; 8]);

impl Guid {
    fn show(&self) -> String {
        let d4: Vec<String> = self.3.iter().map(|b| format!("{b:02X}")).collect();
        format!("{{{:08X}-{:04X}-{:04X}-{}}}", self.0, self.1, self.2, d4.join(""))
    }
}

/// `0x000214D2`, `0x46u8`, `70`, `0x0000_u16` -> the value.
fn number(text: &str) -> Option<u64> {
    let t = text.trim().replace('_', "");
    let t = ["u8", "u16", "u32", "u64", "usize", "i32", "UL", "L", "U"]
        .iter()
        .find_map(|s| t.strip_suffix(s))
        .unwrap_or(&t);
    if let Some(h) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        u64::from_str_radix(h, 16).ok()
    } else {
        t.parse().ok()
    }
}

fn guid_of(parts: &[&str]) -> Option<Guid> {
    let n: Option<Vec<u64>> = parts.iter().map(|p| number(p)).collect();
    let n = n?;
    if n.len() != 11 {
        return None;
    }
    let mut d4 = [0u8; 8];
    for (i, b) in n[3..].iter().enumerate() {
        d4[i] = u8::try_from(*b).ok()?;
    }
    Some(Guid(u32::try_from(n[0]).ok()?, u16::try_from(n[1]).ok()?, u16::try_from(n[2]).ok()?, d4))
}

static SDK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bDEFINE_(GUID|OLEGUID|SHLGUID)\s*\(\s*(\w+)\s*,([^)]*)\)").unwrap()
});

fn sdk_guids(text: &str) -> Vec<(String, Guid)> {
    let mut out = Vec::new();
    for c in SDK_RE.captures_iter(text) {
        let args: Vec<&str> = c[3].split(',').map(str::trim).collect();
        let g = match &c[1] {
            "GUID" => guid_of(&args),
            // guiddef.h: an OLE GUID is {l, w1, w2, {0xC0,0,0,0,0,0,0,0x46}}; shlguid.h's
            // DEFINE_SHLGUID is the same shape.
            _ if args.len() == 3 => {
                let mut full = args.clone();
                full.extend(["0xC0", "0", "0", "0", "0", "0", "0", "0x46"]);
                guid_of(&full)
            }
            _ => None,
        };
        if let Some(g) = g {
            out.push((c[2].to_string(), g));
        }
    }
    out
}

static RUST_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?s)\b(?:const|static)\s+(?:mut\s+)?(\w+)\s*:\s*(?:\w+::)*_?(?:GUID|IID|CLSID)\s*=\s*(?:\w+::)*_?GUID\s*\{\s*Data1\s*:\s*([^,]+),\s*Data2\s*:\s*([^,]+),\s*Data3\s*:\s*([^,]+),\s*Data4\s*:\s*\[([^\]]*)\]",
    )
    .unwrap()
});

fn rust_guids(text: &str) -> Vec<(String, Guid, usize)> {
    let mut out = Vec::new();
    for c in RUST_RE.captures_iter(text) {
        let mut parts = vec![&c[2], &c[3], &c[4]];
        parts.extend(c[5].split(',').map(str::trim).filter(|s| !s.is_empty()));
        if let Some(g) = guid_of(&parts) {
            let line = text[..c.get(0).unwrap().start()].matches('\n').count() + 1;
            out.push((c[1].to_string(), g, line));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const RS: &str = "const CGID_ShellServiceObject: GUID = GUID { Data1: 0x000214D4, Data2: \
                      0x0000, Data3: 0x0000, Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, \
                      0x46] };\n";

    #[test]
    fn an_ole_guid_expands_its_tail_and_a_wrong_digit_is_seen() {
        let sdk = sdk_guids("DEFINE_OLEGUID(CGID_ShellServiceObject, 0x000214D2, 0, 0);");
        assert_eq!(sdk.len(), 1);
        let (name, g, line) = rust_guids(RS).remove(0);
        assert_eq!((name.as_str(), line), ("CGID_ShellServiceObject", 1));
        assert_ne!(g, sdk[0].1);
        let fixed = RS.replace("0x000214D4", "0x000214D2");
        assert_eq!(rust_guids(&fixed)[0].1, sdk[0].1);
    }

    #[test]
    fn a_full_guid_and_suffixed_rust_literals_parse() {
        let sdk = sdk_guids(
            "DEFINE_GUID(CLSID_SysTray, 0x35CEC8A3, 0x2BE6, 0x11D2, 0x87, 0x73, 0x92, 0xE2, \
             0x20, 0x52, 0x41, 0x53);",
        );
        let rs = "pub static CLSID_SysTray: GUID = GUID { Data1: 0x35CEC8A3u32, Data2: \
                  0x2BE6, Data3: 0x11D2_u16, Data4: [0x87, 0x73, 0x92, 0xE2, 0x20, 0x52, \
                  0x41, 0x53u8] };";
        assert_eq!(rust_guids(rs)[0].1, sdk[0].1);
    }
}
