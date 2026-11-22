//! A `size_of::<X>()` standing where the C twin says `sizeof(Y)`: the wrong type sized.
//!
//! crypt32's `CRYPT_CopyChainToElement` allocated `sizeof(CertificateChain)` in the C and
//! `size_of::<CERT_CHAIN_CONTEXT>()` in the Rust, the struct the wrapper embeds. Every field
//! store past it overran the heap block; it surfaced as a crash in ntdll's free-list code, and
//! only page heap put it back at the allocation (2026-11-21).
//!
//! Per function, against the same-module C function of the same name: a type the Rust sizes
//! MORE often than the C does, while a type the C sizes is sized LESS often in the Rust. The
//! missing C type must also appear in the Rust body as a type (`as *mut Y`, `: *mut Y`), and
//! the swap must be one-for-one: as many extra sizings as missing ones.
//! Without that, the C's `sizeof(var)`, which the Rust spells as the variable's type, would
//! read as a swap. Type names compare without a leading underscore or `struct`, so
//! `_CERT_CONTEXT` and `CERT_CONTEXT` are one type. Scalar and pointer-width names (`WCHAR`,
//! `u16`, `ULONG`, `PVOID`, ...) are set aside: one width has many spellings, and only a
//! struct-for-struct swap is the bug.
//! `// nolint: sizeof-vs-c (why)` on the function's `fn` line suppresses.
use crate::{twins, Corpus, Rule};
use rayon::prelude::*;
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::LazyLock;

pub struct SizeofVsC;
impl Rule for SizeofVsC {
    fn name(&self) -> &'static str {
        "scan_sizeof_vs_c"
    }
    fn needs_c_twins(&self) -> bool {
        true
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let index = c.c_twins();
        let files: Vec<_> = c
            .rust_files()
            .filter(|f| f.text.contains("size_of::<"))
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
static RUST_SIZEOF: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"size_of::<\s*(?:[\w:]+::)?([A-Za-z_]\w*)\s*>\s*\(\s*\)").unwrap()
});
static C_SIZEOF: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bsizeof\s*\(\s*(?:struct\s+|union\s+)?([A-Za-z_]\w*)\s*\)").unwrap()
});
static NOLINT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bnolint:[\w\s,-]*\bsizeof-vs-c\b").unwrap());

fn norm(name: &str) -> String {
    name.trim_start_matches('_').to_owned()
}

/// Scalar and pointer-width names: `WCHAR`/`u16`, `ULONG`/`DWORD`/`u32`, `PVOID`/`usize` are one
/// width under many spellings, so they never decide a swap. Only struct-like types do.
const SCALARS: &[&str] = &[
    "u8", "i8", "u16", "i16", "u32", "i32", "u64", "i64", "usize", "isize", "f32", "f64", "bool",
    "c_char", "c_uchar", "c_schar", "c_short", "c_ushort", "c_int", "c_uint", "c_long",
    "c_ulong", "c_longlong", "c_ulonglong", "c_void", "char", "wchar_t", "time_t", "size_t",
    "CHAR", "UCHAR", "BYTE", "BOOLEAN", "WCHAR", "WORD", "USHORT", "SHORT", "ULONG", "LONG",
    "DWORD", "UINT", "INT", "BOOL", "NTSTATUS", "HRESULT", "ACCESS_MASK", "ULONGLONG",
    "LONGLONG", "ULONG64", "LONG64", "DWORD64", "DWORDLONG", "ULONG_PTR", "LONG_PTR",
    "DWORD_PTR", "UINT_PTR", "INT_PTR", "SIZE_T", "SSIZE_T", "ACPI_INTEGER", "PVOID", "LPVOID",
    "HANDLE", "PSTR", "LPSTR", "PCSTR", "LPCSTR", "PCHAR", "PWSTR", "LPWSTR", "PCWSTR",
    "LPCWSTR", "PWCHAR", "BSTR", "LPITEMIDLIST", "HFONT", "HWND", "HKEY", "HMODULE",
    "HINSTANCE", "HDC", "HBITMAP", "HICON", "HMENU", "PUCHAR", "PULONG", "LPDWORD", "LPBYTE",
    "PBYTE", "FLOAT", "DOUBLE",
];

fn is_scalar(name: &str) -> bool {
    SCALARS.contains(&name)
}

fn counts(re: &Regex, text: &str) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for c in re.captures_iter(text) {
        *out.entry(norm(c.get(1).unwrap().as_str())).or_insert(0) += 1;
    }
    out
}

/// `name` used as a type in Rust text: a cast target or a pointer annotation.
fn used_as_type(body: &str, name: &str) -> bool {
    let pat = format!(
        r"(?:\bas\s+(?:\*\s*(?:mut|const)\s+)?|:\s*\*\s*(?:mut|const)\s+)_?{}\b",
        regex::escape(name)
    );
    Regex::new(&pat).unwrap().is_match(body)
}

/// `name` is a variable in the Rust function (a `let` or a parameter) or in its C twin (a
/// declaration or a parameter): then the C's `sizeof(name)` sizes an object, and the Rust
/// rightly spells it as that object's type.
fn is_variable(name: &str, rust_params: &str, rust_body: &str, c_params: &str, c_body: &str) -> bool {
    let n = regex::escape(name);
    let rust = Regex::new(&format!(r"\blet\s+(?:mut\s+)?{n}\b|(?:^|[,(])\s*(?:mut\s+)?{n}\s*:")).unwrap();
    let c = Regex::new(&format!(r"[\w*]\s*\*?\s*\b{n}\s*(?:[;=,)\[]|$)")).unwrap();
    rust.is_match(rust_body) || rust.is_match(rust_params) || c.is_match(c_params) || c.is_match(c_body)
}

fn scan(text: &str, path: &std::path::Path, index: &twins::CIndex) -> Vec<String> {
    let blank = twins::blank_rust(text);
    let newlines: Vec<_> = blank.match_indices('\n').map(|(at, _)| at).collect();
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    for fun in twins::rust_functions(&blank, path) {
        if !fun.body.contains("size_of::<") {
            continue;
        }
        let twins_here: Vec<_> =
            index.get(&fun.name).iter().filter(|t| twins::same_module(path, t)).collect();
        let [twin] = twins_here.as_slice() else {
            continue; // no twin, or an ambiguous one: say nothing
        };
        let rust = counts(&RUST_SIZEOF, &fun.body);
        let c = counts(&C_SIZEOF, &twins::blank_c(&twin.body));
        let extra: Vec<&String> = rust
            .iter()
            .filter(|(k, v)| **v > c.get(*k).copied().unwrap_or(0))
            .map(|(k, _)| k)
            .filter(|k| !is_scalar(k))
            .collect();
        let missing: Vec<&String> = c
            .iter()
            .filter(|(k, v)| **v > rust.get(*k).copied().unwrap_or(0))
            .map(|(k, _)| k)
            .filter(|k| !is_scalar(k))
            .filter(|k| used_as_type(&fun.body, k))
            .filter(|k| !is_variable(k, &fun.params, &fun.body, &twin.params, &twin.body))
            .collect();
        if extra.is_empty() || missing.is_empty() {
            continue;
        }
        // A swap is one-for-one: as many extra sizings of the Rust's types as missing ones of
        // the C's. Unequal deltas are restructured code (a memset become a struct literal, a
        // size computed in a helper), which is not this bug.
        let delta = |m: &BTreeMap<String, usize>, o: &BTreeMap<String, usize>, ks: &[&String]| -> usize {
            ks.iter().map(|k| m[*k] - o.get(*k).copied().unwrap_or(0)).sum()
        };
        if delta(&rust, &c, &extra) != delta(&c, &rust, &missing) {
            continue;
        }
        let line = newlines.partition_point(|&nl| nl < fun.body_start) + 1;
        let fn_line = (1..=line).rev().find(|&l| lines.get(l - 1).is_some_and(|s| s.contains("fn ")));
        if fn_line.is_some_and(|l| lines.get(l - 1).is_some_and(|s| NOLINT.is_match(s))) {
            continue;
        }
        let join = |v: &[&String]| v.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
        out.push(format!(
            "{}: sizeof-vs-c {}: Rust sizes {} where the C ({}:{}) sizes {}",
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
    use std::path::Path;

    fn index(c: &str) -> twins::CIndex {
        let file = crate::SourceFile::new(std::path::PathBuf::from("dll/win32/crypt32/chain.c"), c.into());
        twins::CIndex::from_files([&file])
    }

    const C: &str = "static CertificateChain *CRYPT_CopyChainToElement(CertificateChain *chain)\n{\n    CertificateChain *copy = CryptMemAlloc(sizeof(CertificateChain));\n    copy->context.cbSize = sizeof(CERT_CHAIN_CONTEXT);\n    return copy;\n}\n";

    #[test]
    fn the_wrong_struct_sized_is_flagged() {
        let rust = "unsafe extern \"C\" fn CRYPT_CopyChainToElement(chain: *mut CertificateChain) -> *mut CertificateChain {\n    let copy = CryptMemAlloc(size_of::<CERT_CHAIN_CONTEXT>() as ULONG) as *mut CertificateChain;\n    (*copy).context.cbSize = size_of::<CERT_CHAIN_CONTEXT>() as DWORD;\n    copy\n}\n";
        let got = scan(rust, Path::new("sdk/lib/rust/crypt32-rust/src/crypt32/chain.rs"), &index(C));
        assert_eq!(got.len(), 1, "{got:?}");
        assert!(got[0].contains("sizes CERT_CHAIN_CONTEXT where the C") && got[0].ends_with("sizes CertificateChain"));
    }

    #[test]
    fn the_right_struct_and_a_resolved_sizeof_var_pass() {
        let rust = "unsafe extern \"C\" fn CRYPT_CopyChainToElement(chain: *mut CertificateChain) -> *mut CertificateChain {\n    let copy = CryptMemAlloc(size_of::<CertificateChain>() as ULONG) as *mut CertificateChain;\n    (*copy).context.cbSize = size_of::<_CERT_CHAIN_CONTEXT>() as DWORD;\n    copy\n}\n";
        let p = Path::new("sdk/lib/rust/crypt32-rust/src/crypt32/chain.rs");
        assert!(scan(rust, p, &index(C)).is_empty());
        // `sizeof(*info)` in the C is the variable's type in the Rust: not a swap.
        let c2 = "static BOOL f(LPSTR *info)\n{\n    DWORD n = sizeof(*info) + sizeof(dataLen);\n    return n;\n}\n";
        let r2 = "fn f(info: *mut LPSTR) -> BOOL {\n    let n = size_of::<LPSTR>() + size_of::<DWORD>();\n    n\n}\n";
        assert!(scan(r2, p, &index(c2)).is_empty());
        let nolint = "unsafe extern \"C\" fn CRYPT_CopyChainToElement(chain: *mut CertificateChain) -> *mut CertificateChain { // nolint: sizeof-vs-c (checked)\n    let copy = CryptMemAlloc(size_of::<CERT_CHAIN_CONTEXT>() as ULONG) as *mut CertificateChain;\n    copy\n}\n";
        assert!(scan(nolint, p, &index(C)).is_empty());
    }
}
