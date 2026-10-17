use crate::{Corpus, Rule, SourceSet};
use regex::Regex;
use std::{collections::BTreeMap, sync::LazyLock};

pub struct ExternStaticWidth;

static SOURCES: &[SourceSet] = &[SourceSet::tree(
    "static_width_c",
    &[
        "drivers",
        "dll",
        "ntoskrnl",
        "base",
        "win32ss",
        "subsystems",
        "hal",
        "boot",
        "sdk/lib",
    ],
    &["c"],
    &[],
)];
static C_DEF: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r#"^\s*(?:static\s+)?(?:const\s+)?(?P<ctype>[A-Za-z_][A-Za-z0-9_]*(?:\s+int)?)\s*(?P<ptr>\*?)\s*(?:const\s+)?[A-Za-z_]\w*\s+__asm__\s*\(\s*(?:\w+\s*\(\s*)?\"_?(?P<name>[A-Za-z_]\w*)\""#).unwrap()
});
static RUST_DECL: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"^\s*(?:pub(?:\([^)]*\))?\s+)?static\s+(?:mut\s+)?(?P<name>[A-Za-z_]\w*)\s*:\s*(?P<ty>[^=;]+);").unwrap()
});

fn c_width(ty: &str) -> Option<usize> {
    Some(match ty {
        "UCHAR" | "CHAR" | "BOOLEAN" | "CCHAR" | "BYTE" => 1,
        "USHORT" | "SHORT" | "CSHORT" | "WCHAR" | "WORD" => 2,
        "ULONG" | "LONG" | "INT" | "UINT" | "DWORD" | "NTSTATUS" | "int" | "unsigned"
        | "unsigned int" => 4,
        "LONGLONG" | "ULONGLONG" | "LONG64" | "ULONG64" | "SIZE_T" | "ULONG_PTR" | "LONG_PTR"
        | "PVOID" => 8,
        _ => return None,
    })
}

fn rust_width(ty: &str) -> Option<usize> {
    let ty = ty.trim();
    if ty.starts_with("*mut ") || ty.starts_with("*const ") || ty.starts_with('&') {
        return Some(8);
    }
    let ty = ty.rsplit("::").next().unwrap_or(ty);
    Some(match ty {
        "u8" | "i8" | "c_char" | "c_uchar" | "c_schar" | "BOOLEAN" | "UCHAR" => 1,
        "u16" | "i16" | "c_short" | "c_ushort" | "USHORT" | "WCHAR" | "CSHORT" => 2,
        "u32" | "i32" | "c_int" | "c_uint" | "c_long" | "c_ulong" | "ULONG" | "LONG"
        | "NTSTATUS" | "DWORD" => 4,
        "u64" | "i64" | "LONGLONG" | "ULONGLONG" | "usize" | "isize" | "SIZE_T" | "ULONG_PTR"
        | "PVOID" => 8,
        _ => return None,
    })
}

fn c_definitions(text: &str) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for line in text.lines() {
        let Some(m) = C_DEF.captures(line) else {
            continue;
        };
        let width = if &m["ptr"] == "*" {
            Some(8)
        } else {
            c_width(&m["ctype"])
        };
        if let Some(width) = width {
            out.insert(m["name"].to_owned(), width);
        }
    }
    out
}

fn scan_rust(text: &str, c_widths: &BTreeMap<String, usize>) -> Vec<(usize, String, usize, usize)> {
    let mut hits = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.split_once("//").map_or(line, |(before, _)| before);
        let Some(m) = RUST_DECL.captures(line) else {
            continue;
        };
        let name = &m["name"];
        let Some(&cw) = c_widths.get(name) else {
            continue;
        };
        let Some(rw) = rust_width(&m["ty"]) else {
            continue;
        };
        if rw > cw {
            hits.push((n + 1, name.to_owned(), rw, cw));
        }
    }
    hits
}

impl Rule for ExternStaticWidth {
    fn name(&self) -> &'static str {
        "scan_extern_static_width"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        SOURCES
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut c_widths = BTreeMap::new();
        for file in corpus.sources("static_width_c") {
            if file.text.contains("__asm__") {
                c_widths.extend(c_definitions(&file.text));
            }
        }
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            let rel = file.path.to_string_lossy().replace('\\', "/");
            if rel.contains("/target/") {
                continue;
            }
            for (line, name, rw, cw) in scan_rust(&file.text, &c_widths) {
                out.push(format!("{rel}:{line}: [EXTERN_STATIC_OVERREAD] {name} is read as {rw} bytes but its C definition is {cw} -- declare the C width, or use the crate constant"));
            }
        }
        out.sort();
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceFile;
    use std::path::PathBuf;
    const SHIM: &str = "#define FF_SYM(s) s\nconst ULONG __c_MAX_LFN_CHARACTERS __asm__(FF_SYM(\"MAX_LFN_CHARACTERS\")) = 260;\nconst LONGLONG __c_FCB_LOOKUP_AS_HINT __asm__(FF_SYM(\"FCB_LOOKUP_ALLOCATIONSIZE_HINT\")) = -1;\nconst ULONG __c_FAT_NTC_DCB __asm__(FF_SYM(\"FAT_NTC_DCB\")) = 0x503;\nstatic PVOID __c_Slot __asm__(\"_Slot\") = 0;\n";

    #[test]
    fn asm_symbol_and_type_widths() {
        assert_eq!(
            c_definitions(SHIM),
            BTreeMap::from([
                ("MAX_LFN_CHARACTERS".into(), 4),
                ("FCB_LOOKUP_ALLOCATIONSIZE_HINT".into(), 8),
                ("FAT_NTC_DCB".into(), 4),
                ("Slot".into(), 8),
            ])
        );
    }

    #[test]
    fn overread_positive_and_clean_width_cases() {
        let widths = c_definitions(SHIM);
        assert_eq!(
            scan_rust(
                "extern \"system\" {\n static MAX_LFN_CHARACTERS: usize;\n}\n",
                &widths
            ),
            vec![(2, "MAX_LFN_CHARACTERS".into(), 8, 4)]
        );
        let good = "static FCB_LOOKUP_ALLOCATIONSIZE_HINT: i64;\nstatic FAT_NTC_DCB: u16;\npub static mut Slot: *mut c_void;\nstatic MAX_LFN_CHARACTERS: SomeAlias;\nstatic NOT_IN_C: usize;\n// static MAX_LFN_CHARACTERS: usize;\nstatic OWN: usize = 4;\n";
        assert!(scan_rust(good, &widths).is_empty());
    }

    #[test]
    fn corpus_pairs_rust_and_c_snapshots() {
        let rust = SourceFile::new(
            "sdk/lib/rust/fastfat-rust/src/lib.rs".into(),
            "extern \"system\" {\nstatic MAX_LFN_CHARACTERS: usize;\n}".into(),
        );
        let c = SourceFile::new(
            "drivers/filesystems/fastfat/rust_cohab.c".into(),
            "const ULONG __c_MAX_LFN_CHARACTERS __asm__(FF_SYM(\"MAX_LFN_CHARACTERS\")) = 260;"
                .into(),
        );
        let corpus = Corpus::new(PathBuf::from("/repo"), vec![rust, c]);
        assert_eq!(ExternStaticWidth.run(&corpus).unwrap().len(), 1);
    }
}
