use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

const MC: &str = "sdk/include/reactos/mc/bugcodes.mc";
const HEADER: &str = "sdk/lib/rust/reactos-link/committed-headers/bugcodes.h";
const DEFAULT_FACILITY_COMMENT: (&str, u32) = ("Application", 0xFFF);

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).expect("static regex")
}
fn parse_num(s: &str) -> Result<u32, String> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).map_err(|e| format!("invalid integer {s:?}: {e}"))
    } else {
        s.parse().map_err(|e| format!("invalid integer {s:?}: {e}"))
    }
}

fn parse_mc_bugcodes(text: &str) -> Result<(BTreeMap<String, u32>, BTreeSet<String>), String> {
    let id_re = re(r"^MessageId\s*=\s*(0x[0-9A-Fa-f]+|\d+)\s*$");
    let sev_re = re(r"^Severity\s*=\s*(\w+)\s*$");
    let sym_re = re(r"^SymbolicName\s*=\s*(\w+)\s*$");
    let mut codes = BTreeMap::new();
    let mut stops = BTreeSet::new();
    let (mut id, mut severity) = (None, "Success".to_string());
    for line in text.lines() {
        let s = line.trim();
        if let Some(c) = id_re.captures(s) {
            id = Some(parse_num(&c[1])?);
            severity = "Success".into();
            continue;
        }
        if let Some(c) = sev_re.captures(s) {
            severity = c[1].into();
            continue;
        }
        if let Some(c) = sym_re.captures(s) {
            if let Some(mid) = id.take() {
                let sev = match severity.as_str() {
                    "Success" => 0,
                    "Informational" => 1,
                    "Warning" => 2,
                    "Error" => 3,
                    _ => 0,
                };
                let name = c[1].to_string();
                codes.insert(name.clone(), (sev << 30) | mid);
                if severity == "Success" {
                    stops.insert(name);
                }
            }
        }
    }
    Ok((codes, stops))
}

fn parse_header_codes(text: &str) -> Result<BTreeMap<String, u32>, String> {
    let define = re(r"^#define\s+(\w+)\s+\(ULONG\)\s*(0x[0-9A-Fa-f]+|\d+)");
    let mut codes = BTreeMap::new();
    for line in text.lines() {
        if let Some(c) = define.captures(line.trim()) {
            codes.insert(c[1].into(), parse_num(&c[2])?);
        }
    }
    Ok(codes)
}

fn validate_sources(
    mc: &BTreeMap<String, u32>,
    header: &BTreeMap<String, u32>,
) -> Result<(), String> {
    if mc.is_empty() {
        return Err("gen_bugcodes: parsed no codes from sdk/include/reactos/mc/bugcodes.mc".into());
    }
    let mut problems = Vec::new();
    for (name, val) in mc {
        if let Some(h) = header.get(name) {
            if h != val {
                problems.push(format!(
                    "  {name}: bugcodes.mc=0x{val:X} committed-header=0x{h:X}"
                ));
            }
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "gen_bugcodes: SOURCES DISAGREE — refusing to generate:\n{}",
            problems.join("\n")
        ))
    }
}

fn render_bugcodes(codes: &BTreeMap<String, u32>, stops: &BTreeSet<String>) -> String {
    let width = codes.keys().map(String::len).max().unwrap_or(0);
    let mut rows = Vec::new();
    let mut sorted: Vec<_> = codes.iter().collect();
    sorted.sort_by_key(|(n, v)| (**v, n.as_str()));
    for (name, val) in sorted {
        let tail = if stops.contains(name.as_str()) {
            ""
        } else {
            "  // message string, NOT a stop code"
        };
        rows.push(format!(
            "pub const {name:<width$} : ULONG = 0x{val:X};{tail}"
        ));
    }
    let body = rows.join("\n");
    let n_stop = stops.len();
    let n_msg = codes.len() - n_stop;
    format!("//! Kernel bugcheck (stop) codes — GENERATED, do not edit by hand.\n//!\n//! Regenerate with `python3 tools/gen_bugcodes.py` after touching\n//! `sdk/include/reactos/mc/bugcodes.mc`.\n//!\n//! Pass these to `KeBugCheck`/`KeBugCheckEx` instead of a numeric literal. A\n//! hand-written number is how 61 translated stop codes ended up naming the\n//! wrong subsystem (field manual §9.113); `tools/scan_bugcheck_codes.py` keeps\n//! new ones out.\n//!\n//! Source: sdk/include/reactos/mc/bugcodes.mc ({n_stop} stop codes + {n_msg}\n//! bugcheck-screen message strings, which keep their message-compiler severity\n//! bits), cross-verified against\n//! sdk/lib/rust/reactos-link/committed-headers/bugcodes.h.\n\n#![allow(dead_code)]\n\nuse crate::ddk::ULONG;\n\n{body}\n\n/// Real-NT stop code that ReactOS's bugcodes.mc does not define. Used by our own\n/// IPI/spinlock stall guards, which have no C original to inherit a code from.\n/// Kept here so those call sites stay symbolic like every other one.\npub const CLOCK_WATCHDOG_TIMEOUT: ULONG = 0x101;\n")
}

pub fn bugcodes(root: &Path) -> Result<String, String> {
    let mc = crate::read(&root.join(MC))?;
    let hdr = crate::read(&root.join(HEADER))?;
    let (codes, stops) = parse_mc_bugcodes(&mc)?;
    let header = parse_header_codes(&hdr)?;
    validate_sources(&codes, &header)?;
    Ok(render_bugcodes(&codes, &stops))
}

type Named = Vec<(String, u32, Option<String>)>;
fn parse_names(block: &str) -> Result<Named, String> {
    let pat = re(r"(\w+)=(0x[0-9A-Fa-f]+|\d+)(?::(\w+))?");
    let mut out = Vec::new();
    for c in pat.captures_iter(block) {
        out.push((
            c[1].into(),
            parse_num(&c[2])?,
            c.get(3).map(|m| m.as_str().to_string()),
        ))
    }
    Ok(out)
}
fn parse_mc_header(
    text: &str,
) -> Result<(Option<String>, Named, Named, Vec<(String, u32)>), String> {
    let typedef = re(r"(?m)^MessageIdTypedef\s*=\s*(\w+)")
        .captures(text)
        .map(|c| c[1].to_string());
    let grab = |kw: &str| -> Result<Named, String> {
        let pat = re(&format!(r"(?s){kw}\s*=\s*\((.*?)\)"));
        match pat.captures(text) {
            Some(c) => parse_names(&c[1]),
            None => Ok(Vec::new()),
        }
    };
    let sevs = grab("SeverityNames")?;
    let facs = grab("FacilityNames")?;
    let sv: BTreeMap<_, _> = sevs.iter().map(|(n, v, _)| (n.as_str(), *v)).collect();
    let fv: BTreeMap<_, _> = facs.iter().map(|(n, v, _)| (n.as_str(), *v)).collect();
    let mut msgs = Vec::new();
    let starts: Vec<_> = re(r"(?m)^MessageId\s*=")
        .find_iter(text)
        .map(|m| m.start())
        .collect();
    let field = re(r"(?m)^(MessageId|Severity|Facility|SymbolicName)\s*=\s*(.*)$");
    for (i, start) in starts.iter().enumerate() {
        let blk = &text[*start..starts.get(i + 1).copied().unwrap_or(text.len())];
        let mut fields = BTreeMap::new();
        for c in field.captures_iter(blk) {
            fields.insert(c[1].to_string(), c[2].trim().to_string());
        }
        if let Some(sym) = fields.get("SymbolicName") {
            let mid = parse_num(fields.get("MessageId").ok_or("message missing MessageId")?)?;
            let sev = fields
                .get("Severity")
                .and_then(|x| sv.get(x.as_str()))
                .copied()
                .unwrap_or(0);
            let fac = fields
                .get("Facility")
                .and_then(|x| fv.get(x.as_str()))
                .copied()
                .unwrap_or(0);
            msgs.push((sym.clone(), (sev << 30) | (fac << 16) | mid));
        }
    }
    Ok((typedef, sevs, facs, msgs))
}
const PREAMBLE: &str = include_str!("bugcodes_preamble.txt");
fn render_bugcodes_h(text: &str) -> Result<String, String> {
    let (typedef, sevs, facs, msgs) = parse_mc_header(text)?;
    if msgs.is_empty() {
        return Err("no message IDs parsed from bugcodes.mc".into());
    }
    let cast = typedef.map(|t| format!("({t}) ")).unwrap_or_default();
    let mut out = PREAMBLE.to_string();
    for (n, v, sym) in sevs {
        out.push_str(&format!("//           {n} - {v:02x}\n"));
        if let Some(s) = sym {
            out.push_str(&format!("#define {s} 0x{v:x}\n"));
        }
    }
    out.push_str("//\n//      Facility - is the facility code\n//\n");
    for (n, v, sym) in facs {
        out.push_str(&format!("//           {n} - {v:04x}\n"));
        if let Some(s) = sym {
            out.push_str(&format!("#define {s} 0x{v:x}\n"));
        }
    }
    let (dn, dv) = DEFAULT_FACILITY_COMMENT;
    out.push_str(&format!("//           {dn} - {dv:04x}\n//\n"));
    for (sym, code) in msgs {
        out.push_str(&format!(
            "\n//\n// MessageId: {sym}\n//\n#define {sym} {cast}0x{code:x}\n"
        ));
    }
    out.push('\n');
    Ok(out)
}
pub fn bugcodes_h(root: &Path) -> Result<String, String> {
    render_bugcodes_h(&crate::read(&root.join(MC))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn severity_and_facility_are_composed() {
        let s="SeverityNames=(Success=0x0 Informational=0x1 Warning=0x2 Error=0x3)\nFacilityNames=(System=0x0 Io=0x4)\nMessageId=0x12\nSeverity=Warning\nFacility=Io\nSymbolicName=IO_TEST\nLanguage=English\ntext\n\nMessageId=0x13\nSymbolicName=STOP_TEST\n";
        let (t, sv, fa, m) = parse_mc_header(s).unwrap();
        assert!(t.is_none());
        assert_eq!(sv[1].1, 1);
        assert_eq!(fa[1].1, 4);
        assert_eq!(m[0].1, 0x8004_0012);
        assert_eq!(m[1].1, 0x13);
    }
    #[test]
    fn bugcodes_severity_marks_only_success_as_stop() {
        let s="MessageId=0x7e\nSeverity=Informational\nSymbolicName=SCREEN\nMessageId=0x8e\nSymbolicName=STOP\n";
        let (c, st) = parse_mc_bugcodes(s).unwrap();
        assert_eq!(c["SCREEN"], 0x4000_007e);
        assert!(!st.contains("SCREEN"));
        assert!(st.contains("STOP"));
    }
    #[test]
    fn cross_source_mismatch_fails_loudly() {
        let mc = BTreeMap::from([("A".to_string(), 0x10)]);
        let h = BTreeMap::from([("A".to_string(), 0x11)]);
        assert!(validate_sources(&mc, &h)
            .unwrap_err()
            .contains("SOURCES DISAGREE"));
    }
    #[test]
    fn generated_header_keeps_typedef_and_facility_format() {
        let s="MessageIdTypedef=ULONG\nSeverityNames=(Success=0x0:SEV)\nFacilityNames=(Io=0x4:FAC)\nMessageId=0x12\nSeverity=Success\nFacility=Io\nSymbolicName=IO_TEST\n";
        let h = render_bugcodes_h(s).unwrap();
        assert!(h.contains("#define IO_TEST (ULONG) 0x40012\n"));
        assert!(h.contains("//           Application - 0fff\n"));
        assert!(render_bugcodes_h("not a message file").is_err());
    }
}
