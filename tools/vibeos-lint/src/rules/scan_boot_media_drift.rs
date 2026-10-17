use crate::{Corpus, Rule, SourceSet};
use regex::Regex;
use std::sync::LazyLock;
pub struct ScanBootMediaDrift;
const CMAKE: &str = "sdk/cmake/CMakeMacros.cmake";
const RAM: &str = "qemu/mkramdisk.sh";
const SETS: &[SourceSet] = &[SourceSet::tree(
    "boot-media-inputs",
    &["sdk/cmake/CMakeMacros.cmake", "qemu/mkramdisk.sh"],
    &["cmake", "sh"],
    &[],
)
.with_inputs()];
static BLOCK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\(").unwrap());
static HIVE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"boot/bootdata/([A-Za-z0-9_.]+)").unwrap());
static ASK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"native-mkhive\s+-h:([A-Za-z0-9_,]+)").unwrap());
static NEXT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(COMMAND|DEPENDS|WORKING_DIRECTORY|COMMENT|MAIN_DEPENDENCY|IMPLICIT_DEPENDS|BYPRODUCTS|VERBATIM|APPEND|USES_TERMINAL|COMMAND_EXPAND_LISTS|TARGET|DESTINATION|FOR|NAME_ON_CD|NO_CAB|OUTPUT|FILE)\b").unwrap()
});
static DEST: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bDESTINATION\s+(\S+)").unwrap());
static LIVE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bFOR\b[^)]*\blivecd\b").unwrap());
fn line_at(s: &str, i: usize) -> usize {
    s[..i].bytes().filter(|b| *b == b'\n').count() + 1
}
fn blocks<'a>(text: &'a str, command: &str) -> Vec<(&'a str, usize)> {
    let mut out = Vec::new();
    for m in BLOCK.find_iter(text) {
        let whole = m.as_str();
        let name = whole.split('(').next().unwrap_or("").trim();
        if name != command {
            continue;
        }
        let open = m.end() - 1;
        let mut depth = 0i32;
        let mut end = text.len();
        for (off, ch) in text[open..].char_indices() {
            if ch == '(' {
                depth += 1;
            } else if ch == ')' {
                depth -= 1;
                if depth == 0 {
                    end = open + off;
                    break;
                }
            }
        }
        out.push((&text[open + 1..end], line_at(text, m.start())));
    }
    out
}
fn files(body: &str, section: Option<&str>) -> Vec<String> {
    let mut s = body;
    if let Some(sec) = section {
        let Some(m) = Regex::new(&format!(r"\b{}\b", sec)).unwrap().find(body) else {
            return vec![];
        };
        let rest = &body[m.end()..];
        s = if let Some(n) = NEXT.find(rest) {
            &rest[..n.start()]
        } else {
            rest
        };
    }
    HIVE.captures_iter(s)
        .filter_map(|c| {
            let n = c[1].to_string();
            (!n.to_lowercase().ends_with(".inf")).then_some(n)
        })
        .collect()
}
fn audit(cmake: &str, ram: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut generated = std::collections::BTreeMap::<String, usize>::new();
    let mut count = 0;
    for (body, line) in blocks(cmake, "add_custom_command") {
        let Some(m) = ASK.captures(body) else {
            continue;
        };
        let asked: Vec<_> = m[1].split(',').filter(|s| !s.is_empty()).collect();
        let outputs = files(body, Some("OUTPUT"));
        if asked.len() != outputs.len() {
            out.push(format!("{CMAKE}:{line}: [HIVE_LIST_DRIFT] native-mkhive is told to emit {} hive(s) ({}) but the command declares {} OUTPUT file(s) ({}) — an undeclared output is never a dependency, so it goes stale silently",asked.len(),asked.join(","),outputs.len(),if outputs.is_empty(){"none".into()}else{outputs.join(",")}));
            count += 1;
        }
        for n in outputs {
            generated.insert(n, line);
        }
    }
    let mut live = std::collections::BTreeMap::<String, usize>::new();
    for (body, line) in blocks(cmake, "add_cd_file") {
        if !LIVE.is_match(body) {
            continue;
        }
        let Some(d) = DEST.captures(body) else {
            continue;
        };
        if d[1].trim().to_lowercase() != "reactos/system32/config" {
            continue;
        }
        for n in files(body, Some("FILE")) {
            if generated.contains_key(&n) {
                live.insert(n, line);
            }
        }
    }
    if live.is_empty() {
        return vec![format!("{CMAKE}:1: [NO_REFERENCE] found no add_cd_file(... FOR livecd) block staging generated hives — this audit cannot see the reference set, so it would pass vacuously; the parser or the cmake layout has changed")];
    }
    let lays =
        Regex::new(r"livecd\.\w+\.lst").unwrap().is_match(ram) && ram.contains("ramdisk_tree.py");
    if !lays {
        let mut h: Vec<_> = live.keys().cloned().collect();
        h.sort();
        out.push(format!("{RAM}:1: [RAMDISK_NOT_FROM_LIVECD] the ramdisk is not built from the livecd list (no livecd.<config>.lst laid out by tools/ramdisk_tree.py), so nothing makes it carry the {} hive(s) the livecd stages ({}); a missing one is FABRICATED empty at boot",live.len(),h.join(", ")));
        count += 1;
    }
    out.push(String::new());
    out.push(format!(
        "SUMMARY: generated={}  livecd={}  ramdisk_from_livecd={}  findings={count}",
        generated.len(),
        live.len(),
        if lays { "yes" } else { "NO" }
    ));
    out
}
impl Rule for ScanBootMediaDrift {
    fn name(&self) -> &'static str {
        "scan_boot_media_drift"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        SETS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let Some(cm) = c
            .sources("boot-media-inputs")
            .find(|f| f.path.ends_with(CMAKE))
        else {
            return Ok(vec![format!(
                "{CMAKE}:1: [MISSING] cannot audit boot media without this file"
            )]);
        };
        let Some(ram) = c
            .sources("boot-media-inputs")
            .find(|f| f.path.ends_with(RAM))
        else {
            return Ok(vec![format!(
                "{RAM}:1: [MISSING] cannot audit boot media without this file"
            )]);
        };
        Ok(audit(&cm.text, &ram.text))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Corpus, SourceFile};
    use std::path::PathBuf;
    fn f(p: &str, s: &str) -> SourceFile {
        SourceFile::new(PathBuf::from(p), s.into())
    }
    const CM: &str = r#"
    add_custom_command(
        OUTPUT ${CMAKE_BINARY_DIR}/boot/bootdata/system
               ${CMAKE_BINARY_DIR}/boot/bootdata/sam
        COMMAND native-mkhive -h:SYSTEM,SAM -d:${CMAKE_BINARY_DIR}/boot/bootdata ${_inf}
        DEPENDS native-mkhive ${_inf})

    add_cd_file(
        FILE ${CMAKE_BINARY_DIR}/boot/bootdata/system
             ${CMAKE_BINARY_DIR}/boot/bootdata/sam
        TARGET livecd_hives
        DESTINATION reactos/system32/config
        FOR livecd)

    add_cd_file(
        FILE ${CMAKE_BINARY_DIR}/boot/bootdata/BCD
        TARGET bcd_hive
        DESTINATION efi/boot
        FOR bootcd regtest livecd)
"#;
    #[test]
    fn livecd_list_passes() {
        let x = audit(
            CM,
            "LIVECD_LST=livecd.Debug.lst\npython3 tools/ramdisk_tree.py",
        );
        assert!(x.last().unwrap().contains("findings=0"));
        assert!(!x.join("\n").contains("BCD"));
    }
    #[test]
    fn hand_staged_ramdisk_and_output_drift_report() {
        let x = audit(CM, "RMC boot/bootdata/system");
        assert!(x.iter().any(|l| l.contains("[RAMDISK_NOT_FROM_LIVECD]")));
        assert!(x.iter().any(|l| l.contains("sam")));
        let bad = CM.replace("-h:SYSTEM,SAM", "-h:SYSTEM,SAM,SECURITY");
        assert!(audit(&bad, "livecd.Debug.lst ramdisk_tree.py")
            .iter()
            .any(|l| l.contains("[HIVE_LIST_DRIFT]")));
    }
    #[test]
    fn no_reference_fails_loudly() {
        let x = audit("# moved", "livecd.x.lst ramdisk_tree.py");
        assert!(x[0].contains("[NO_REFERENCE]"));
    }
    #[test]
    fn rule_reads_two_snapshots() {
        let c = Corpus::new(
            PathBuf::from("."),
            vec![f(CMAKE, CM), f(RAM, "livecd.x.lst ramdisk_tree.py")],
        );
        assert!(ScanBootMediaDrift
            .run(&c)
            .unwrap()
            .last()
            .unwrap()
            .contains("findings=0"));
    }
}
