//! A comparison against a different named constant than the C twin's: a permuted constant.
//!
//! crypt32's `CertAddStoreToCollection` checks `collection->hdr.type != StoreTypeCollection`;
//! the Rust checked `type_ != E_INVALIDARG`, a constant the function uses three lines later.
//! The fact seal's invariant is a closed SET of constants per function, and a permutation
//! satisfies it, so every collection add was refused (2026-11-21, found by Wine's suite).
//!
//! Per function, against the same-module C function of the same name: the named constants each
//! side compares with `==`/`!=` (either operand). A constant the Rust compares more often than
//! the C, while the C compares another more often than the Rust, is a swap when the two excess
//! counts are equal and the C's constant is compared NOWHERE in the Rust function. Names are
//! constant-like: no lowercase letter, or a CamelCase enum member (in the C, any capitalized
//! name that is not one of the function's variables; in the Rust, one the C compares too).
//! Casts are skipped. `// nolint: compare-const-vs-c (why)` on the `fn` line suppresses.
use crate::{twins, Corpus, Rule};
use rayon::prelude::*;
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::LazyLock;

pub struct CompareConstVsC;
impl Rule for CompareConstVsC {
    fn name(&self) -> &'static str {
        "scan_compare_const_vs_c"
    }
    fn needs_c_twins(&self) -> bool {
        true
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let index = c.c_twins();
        let files: Vec<_> = c
            .rust_files()
            .filter(|f| f.text.contains("==") || f.text.contains("!="))
            .filter(|f| !SKIP.is_match(&f.path.to_string_lossy()))
            .collect();
        let mut out: Vec<String> = files
            .par_iter()
            .flat_map_iter(|f| {
                scan(&f.text, &f.path, index)
                    .into_iter()
                    .map(move |x| format!("{}:{x}", f.path.display()))
            })
            .collect();
        out.sort();
        Ok(out)
    }
}

static SKIP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/types_[a-z0-9]+\.rs$)").unwrap()
});
/// One `==`/`!=` and the text around it: the left operand's last identifier with what precedes
/// it, the right operand's first identifier with what follows it.
static CMP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?P<lpre>[.>*]?)\b(?:[A-Za-z_]\w*::)*(?P<lhs>[A-Za-z_]\w*)\s*(?:\s+as\s+[\w:]+)*\s*\)*\s*(?:[=!]=)\s*(?:\(\s*[A-Za-z_][\w\s*]*\)\s*)?\(*\s*(?P<rpre>[*&]?)(?:[A-Za-z_]\w*::)*(?P<rhs>[A-Za-z_]\w*|\d)(?P<rpost>\s*(?:\.|->|\(|\[|!)?)").unwrap()
});
static RUST_CAST_TAIL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s+as\s+[\w:]+(?:\s+as\s+[\w:]+)*").unwrap());
static NOLINT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bnolint:[\w\s,-]*\bcompare-const-vs-c\b").unwrap());

/// A name that reads as a constant: no lowercase letter (`E_INVALIDARG`, `TRUE`), or a CamelCase
/// enum member (`StoreTypeCollection`) when the other side also spells it.
fn constant_like(name: &str) -> bool {
    name.chars().any(|c| c.is_ascii_uppercase())
        && !name.chars().any(|c| c.is_ascii_lowercase())
        && name.len() > 1
}

/// The named constants a function compares with `==`/`!=`. A right operand counts unless it is
/// the base of a field access, call or index (`== p->x`, `== f(..)`). A left operand counts only
/// when it is ALL_CAPS and the right one is not a number: `(x & FLAG) != 0` is the Rust spelling
/// of C's truthiness test `if (x & FLAG)`, not a comparison with FLAG. Neither counts after `.`
/// or `->` (a field). `camel_ok(name)` decides a CamelCase name: in the C, any capitalized name
/// that is not one of the function's variables; in the Rust, one the C compares too, since the
/// Rust spells locals and fields in CamelCase as often as enum members.
fn compared(text: &str, rust: bool, camel_ok: &dyn Fn(&str) -> bool) -> BTreeMap<String, usize> {
    let text = if rust {
        RUST_CAST_TAIL.replace_all(text, "").into_owned()
    } else {
        text.to_owned()
    };
    let mut out = BTreeMap::new();
    let mut take = |name: &str| {
        let name = name.trim_start_matches('_');
        let camel = name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            && name.chars().any(|c| c.is_ascii_lowercase());
        if constant_like(name) || (camel && camel_ok(name)) {
            *out.entry(name.to_owned()).or_insert(0) += 1;
        }
    };
    for c in CMP.captures_iter(&text) {
        let rhs = &c["rhs"];
        let rpost = c["rpost"].trim();
        let numeric = rhs.chars().next().is_some_and(|ch| ch.is_ascii_digit());
        if !numeric && rpost.is_empty() && c["rpre"].is_empty() {
            take(rhs);
        }
        let lhs = &c["lhs"];
        if c["lpre"].is_empty() && !numeric && constant_like(lhs) {
            take(lhs);
        }
    }
    out
}

/// A variable the C function declares (`TYPE name;`, `TYPE *name =`, a parameter).
fn c_variable(name: &str, params: &str, body: &str) -> bool {
    let re = Regex::new(&format!(
        r"[\w*]\s*\*?\s*\b{}\s*(?:[;=,)\[]|$)",
        regex::escape(name)
    ))
    .unwrap();
    re.is_match(params) || re.is_match(body)
}

/// The letters and digits of a name, uppercased: `TableFoundNode` and `TABLE_FOUND_NODE` agree.
fn letters(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

/// `name` is used as a type: a C cast `(NAME)` / `(NAME *)`, or a Rust `as NAME`.
fn is_cast_type(text: &str, name: &str, rust: bool) -> bool {
    let n = regex::escape(name);
    let pat = if rust {
        format!(r"\bas\s+(?:\*\s*(?:mut|const)\s+)?_?{n}\b")
    } else {
        format!(r"\(\s*_?{n}\s*\**\s*\)")
    };
    Regex::new(&pat).unwrap().is_match(text)
}

/// Never a swap: booleans, success codes, and `MAX`/`MIN` (the Rust spells `INVALID_HANDLE_VALUE`
/// and `WAIT_FAILED` as `usize::MAX`/`u32::MAX`).
const IGNORED: &[&str] = &[
    "NULL",
    "TRUE",
    "FALSE",
    "S_OK",
    "ERROR_SUCCESS",
    "STATUS_SUCCESS",
    "MAX",
    "MIN",
];

fn scan(text: &str, path: &std::path::Path, index: &twins::CIndex) -> Vec<String> {
    let blank = twins::blank_rust(text);
    let newlines: Vec<_> = blank.match_indices('\n').map(|(at, _)| at).collect();
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    for fun in twins::rust_functions(&blank, path) {
        let twins_here: Vec<_> = index
            .get(&fun.name)
            .iter()
            .filter(|t| twins::same_module(path, t))
            .collect();
        let [twin] = twins_here.as_slice() else {
            continue;
        };
        let c_body = twins::blank_c(&twin.body);
        let c = compared(&c_body, false, &|n| !c_variable(n, &twin.params, &c_body));
        let rust = compared(&fun.body, true, &|n| c.contains_key(n));
        let keep = |k: &&String| !IGNORED.contains(&k.as_str());
        // A cast type is not a constant: `(UINT)x == ..` in the C, `as SIZE_T` in the Rust.
        let c_type = |k: &&String| !is_cast_type(&c_body, k, false);
        let r_type = |k: &&String| !is_cast_type(&fun.body, k, true);
        let mut extra: Vec<&String> = rust
            .iter()
            .filter(|(k, v)| **v > c.get(*k).copied().unwrap_or(0))
            .map(|(k, _)| k)
            .filter(keep)
            .filter(r_type)
            .collect();
        let mut missing: Vec<&String> = c
            .iter()
            .filter(|(k, _)| !rust.contains_key(*k))
            .map(|(k, _)| k)
            .filter(keep)
            .filter(c_type)
            .collect();
        // One constant under two spellings is no swap: `TableFoundNode`/`TABLE_FOUND_NODE`,
        // `GDILoObjType_LO_DC_TYPE`/`LO_DC_TYPE`, `STATUS_X`/`STATUS_X_C`. Pair them off.
        let mut i = 0;
        while i < missing.len() {
            let m = letters(missing[i]);
            if let Some(j) = extra.iter().position(|e| {
                let e = letters(e);
                e.contains(&m) || m.contains(&e)
            }) {
                extra.remove(j);
                missing.remove(i);
            } else {
                i += 1;
            }
        }
        if extra.is_empty() || missing.is_empty() {
            continue;
        }
        if extra.is_empty() || missing.is_empty() {
            continue;
        }
        let ex: usize = extra
            .iter()
            .map(|k| rust[*k] - c.get(*k).copied().unwrap_or(0))
            .sum();
        let mi: usize = missing.iter().map(|k| c[*k]).sum();
        if ex != mi {
            continue;
        }
        let line = newlines.partition_point(|&nl| nl < fun.body_start) + 1;
        let fn_line = (1..=line)
            .rev()
            .find(|&l| lines.get(l - 1).is_some_and(|s| s.contains("fn ")));
        if fn_line.is_some_and(|l| lines.get(l - 1).is_some_and(|s| NOLINT.is_match(s))) {
            continue;
        }
        let join = |v: &[&String]| v.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
        out.push(format!(
            "{}: compare-const-vs-c {}: Rust compares with {} where the C ({}:{}) compares with {}",
            fn_line.unwrap_or(line),
            fun.name,
            join(&extra),
            twin.path.display(),
            twin.line,
            join(&missing)
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn index(c: &str) -> twins::CIndex {
        let file = crate::SourceFile::new(
            PathBuf::from("dll/win32/crypt32/collectionstore.c"),
            c.into(),
        );
        twins::CIndex::from_files([&file])
    }

    const C: &str = "BOOL WINAPI CertAddStoreToCollection(HCERTSTORE hCollectionStore, HCERTSTORE hSiblingStore)\n{\n    WINE_COLLECTIONSTORE *collection = hCollectionStore;\n    if (collection->hdr.dwMagic != WINE_CRYPTCERTSTORE_MAGIC)\n    {\n        SetLastError(E_INVALIDARG);\n        return FALSE;\n    }\n    if (collection->hdr.type != StoreTypeCollection)\n    {\n        SetLastError(E_INVALIDARG);\n        return FALSE;\n    }\n    return TRUE;\n}\n";
    const P: &str = "sdk/lib/rust/crypt32-rust/src/crypt32/collectionstore.rs";

    #[test]
    fn the_permuted_constant_is_flagged() {
        let rust = "pub unsafe extern \"system\" fn CertAddStoreToCollection(hCollectionStore: HCERTSTORE, hSiblingStore: HCERTSTORE) -> BOOL {\n    let collection = hCollectionStore as *mut WINE_COLLECTIONSTORE;\n    if (*collection).hdr.dwMagic != WINE_CRYPTCERTSTORE_MAGIC as DWORD {\n        SetLastError(E_INVALIDARG as DWORD);\n        return FALSE;\n    }\n    if (*collection).hdr.type_ as c_uint\n        != E_INVALIDARG as c_int as c_uint\n    {\n        SetLastError(E_INVALIDARG as DWORD);\n        return FALSE;\n    }\n    TRUE\n}\n";
        let got = scan(rust, Path::new(P), &index(C));
        assert_eq!(got.len(), 1, "{got:?}");
        assert!(
            got[0].contains("compares with E_INVALIDARG where the C")
                && got[0].ends_with("compares with StoreTypeCollection"),
            "{got:?}"
        );
    }

    #[test]
    fn a_path_qualified_enum_member_is_the_member() {
        // freeldr's `Reason == BAD_HIVE_REASON::CorruptHive` is the C's `Reason == CorruptHive`.
        let c = "BOOLEAN WinLdrInitSystemHive(PVOID p)\n{\n    if (Reason == CorruptHive || Reason == NoHive)\n        return FALSE;\n    return TRUE;\n}\n";
        let r = "pub unsafe fn WinLdrInitSystemHive(p: PVOID) -> BOOLEAN {\n    if Reason == BAD_HIVE_REASON::CorruptHive || Reason == BAD_HIVE_REASON::NoHive {\n        return FALSE;\n    }\n    TRUE\n}\n";
        let file = crate::SourceFile::new(
            PathBuf::from("boot/freeldr/freeldr/ntldr/wlregistry.c"),
            c.into(),
        );
        let idx = twins::CIndex::from_files([&file]);
        assert!(scan(
            r,
            Path::new("sdk/lib/rust/freeldr-rust/src/wlregistry.rs"),
            &idx
        )
        .is_empty());
    }

    #[test]
    fn the_faithful_body_passes() {
        let rust = "pub unsafe extern \"system\" fn CertAddStoreToCollection(hCollectionStore: HCERTSTORE, hSiblingStore: HCERTSTORE) -> BOOL {\n    let collection = hCollectionStore as *mut WINE_COLLECTIONSTORE;\n    if (*collection).hdr.dwMagic != WINE_CRYPTCERTSTORE_MAGIC as DWORD {\n        SetLastError(E_INVALIDARG as DWORD);\n        return FALSE;\n    }\n    if (*collection).hdr.type_ != StoreTypeCollection {\n        SetLastError(E_INVALIDARG as DWORD);\n        return FALSE;\n    }\n    TRUE\n}\n";
        assert!(scan(rust, Path::new(P), &index(C)).is_empty());
    }
}
