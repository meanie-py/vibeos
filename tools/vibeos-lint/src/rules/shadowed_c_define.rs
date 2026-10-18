//! A Rust `const` that re-declares a C `#define` with a different value.
//!
//! Translations often carry a C macro across as a local `const`, and a model that
//! writes the value from memory invents one: ke/balmgr.rs had
//! `THREAD_BOOST_PRIORITY = 23` where balmgr.c defines `(LOW_REALTIME_PRIORITY - 1)`,
//! so the starvation boost lifted dynamic threads into the realtime range
//! (2026-10-17, found by a restored ASSERT under kmtest). A name is checked only when
//! every C definition of it VISIBLE to the crate evaluates to the same integer: the
//! shared headers (`sdk/include/`) plus the C tree of the crate's own module (path
//! component = crate stem; the kernel subsystem crates map to `ntoskrnl`). That drops
//! per-arch `#ifdef` variants and a `MAX_CPUS` in some unrelated Wine DLL. The Rust value
//! is compared at the width of its declared type; a type this rule cannot size is
//! skipped rather than guessed.
use crate::{Corpus, Rule};
use rayon::prelude::*;
use regex::Regex;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::LazyLock;

pub struct ShadowedCDefine;
impl Rule for ShadowedCDefine {
    fn name(&self) -> &'static str {
        "scan_shadowed_c_define"
    }
    fn needs_cdecl_sources(&self) -> bool {
        true
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let c_sources: Vec<(String, &str)> = corpus
            .cdecl_files()
            .filter(|f| f.path.extension().is_some_and(|e| e == "c" || e == "h"))
            .filter(|f| !C_SKIP.is_match(&f.path.to_string_lossy()))
            .map(|f| (f.path.to_string_lossy().into_owned(), f.text.as_str()))
            .collect();
        let defines = CDefines::collect(c_sources.iter().map(|(p, t)| (p.as_str(), *t)));
        // A cdecl rule receives its Rust through the cdecl scope, not rust_files().
        let rust: Vec<_> = corpus
            .cdecl_files()
            .filter(|f| f.path.extension().is_some_and(|e| e == "rs"))
            .filter(|f| f.path.starts_with("sdk/lib/rust"))
            .filter(|f| !RUST_SKIP.is_match(&f.path.to_string_lossy()))
            .collect();
        let mut out: Vec<String> = rust
            .par_iter()
            .flat_map_iter(|f| {
                let scope = crate_scope(&f.path.to_string_lossy());
                scan_rust(&f.text, &defines, scope.as_deref())
                    .into_iter()
                    .map(move |x| format!("{}:{}", f.path.display(), x))
            })
            .collect();
        out.sort();
        Ok(out)
    }
}

static C_SKIP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(^|/)(build-[^/]*|\.git|modules/rostests|sdk/lib/rust)/").unwrap()
});
static RUST_SKIP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/island_[a-z0-9_]+\.rs$)").unwrap()
});
static C_COMMENTS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/|//[^\n]*").unwrap());
static DEFINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^[ \t]*#[ \t]*define[ \t]+([A-Za-z_]\w*)[ \t]+([^\n]*?)[ \t]*$").unwrap()
});
static RUST_CONST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^[ \t]*(?:pub(?:\([\w: ]+\))?[ \t]+)?const[ \t]+([A-Za-z_]\w*)[ \t]*:[ \t]*([\w:]+)[ \t]*=[ \t]*([^;]+);",
    )
    .unwrap()
});

/// Every object-like C `#define` with a value all of its definitions agree on.
pub(crate) struct CDefines {
    raw: HashMap<String, Vec<(String, usize, String)>>,
    memo: std::sync::Mutex<HashMap<(Option<String>, String), Option<i128>>>,
}
impl CDefines {
    pub(crate) fn collect<'a>(files: impl Iterator<Item = (&'a str, &'a str)>) -> Self {
        let mut raw: HashMap<String, Vec<(String, usize, String)>> = HashMap::new();
        for (path, text) in files {
            if !text.contains("define") {
                continue;
            }
            // Join backslash continuations, then drop comments, keeping line numbers.
            let joined = text.replace("\\\r\n", "  ").replace("\\\n", " ");
            let code = C_COMMENTS.replace_all(&joined, |c: &regex::Captures| {
                c[0].chars()
                    .map(|ch| if ch == '\n' { '\n' } else { ' ' })
                    .collect::<String>()
            });
            for c in DEFINE.captures_iter(&code) {
                let line = code[..c.get(0).unwrap().start()].matches('\n').count() + 1;
                raw.entry(c[1].to_string()).or_default().push((
                    path.to_string(),
                    line,
                    c[2].to_string(),
                ));
            }
        }
        Self {
            raw,
            memo: Default::default(),
        }
    }
    /// The single value every definition of `name` visible from `scope` evaluates
    /// to, if there is one. `None` scope sees every definition (tests).
    pub(crate) fn value(&self, name: &str, scope: Option<&str>) -> Option<i128> {
        self.value_inner(name, scope, &mut HashSet::new())
    }
    fn value_inner(
        &self,
        name: &str,
        scope: Option<&str>,
        active: &mut HashSet<String>,
    ) -> Option<i128> {
        let key = (scope.map(str::to_string), name.to_string());
        if let Some(v) = self.memo.lock().unwrap().get(&key) {
            return *v;
        }
        let defs: Vec<_> = self
            .raw
            .get(name)?
            .iter()
            .filter(|(p, _, _)| visible(p, scope))
            .collect();
        if defs.is_empty() {
            return None;
        }
        if !active.insert(name.to_string()) {
            return None;
        }
        let mut agreed: Option<i128> = None;
        let mut ok = true;
        for (_, _, body) in defs {
            let v = eval(
                body,
                &mut |id| self.value_inner(id, scope, active),
                Dialect::C,
            );
            match (v, agreed) {
                (Some(v), None) => agreed = Some(v),
                (Some(v), Some(a)) if v == a => {}
                _ => {
                    ok = false;
                    break;
                }
            }
        }
        active.remove(name);
        let result = if ok { agreed } else { None };
        self.memo.lock().unwrap().insert(key, result);
        result
    }
    fn site(&self, name: &str, scope: Option<&str>) -> Option<(&str, usize)> {
        self.raw
            .get(name)?
            .iter()
            .find(|(p, _, _)| visible(p, scope))
            .map(|(p, l, _)| (p.as_str(), *l))
    }
}

/// The C module a Rust crate translates, as a path component: `ex-rust` -> `ntoskrnl`.
fn crate_scope(path: &str) -> Option<String> {
    let name = path.strip_prefix("sdk/lib/rust/")?.split('/').next()?;
    let stem = name
        .trim_end_matches("-rust")
        .trim_end_matches("-sys")
        .trim_end_matches("-cargo");
    Some(
        match stem {
            "ntkrnl" | "ntoskrnl" | "ex" | "ps" | "cc" | "ob" | "se" | "iomgr" | "config"
            | "mm" | "ke" | "ke-inlines" | "kernel" | "fsrtl" | "kd64" | "kdbg" | "inbv" | "po"
            | "lpc" | "dbgk" | "wmi" => "ntoskrnl",
            "rtlk" => "rtl",
            "win32k" => "win32ss",
            other => other,
        }
        .to_string(),
    )
}

fn visible(c_path: &str, scope: Option<&str>) -> bool {
    match scope {
        None => true,
        Some(stem) => {
            c_path.starts_with("sdk/include/") || c_path.split('/').any(|part| part == stem)
        }
    }
}

fn scan_rust(text: &str, defines: &CDefines, scope: Option<&str>) -> Vec<String> {
    let code = crate::lexical::strip_noise(text);
    let mut locals: HashMap<String, String> = HashMap::new();
    let mut sites = Vec::new();
    for c in RUST_CONST.captures_iter(&code) {
        locals.insert(c[1].to_string(), c[3].to_string());
        let line = code[..c.get(0).unwrap().start()].matches('\n').count() + 1;
        sites.push((c[1].to_string(), c[2].to_string(), c[3].to_string(), line));
    }
    let mut out = Vec::new();
    let mut seen = BTreeMap::new();
    for (name, ty, rhs, line) in sites {
        let Some(bits) = type_bits(&ty) else { continue };
        let Some(c_value) = defines.value(&name, scope) else {
            continue;
        };
        let mut active = HashSet::from([name.clone()]);
        let Some(r_value) = eval_rust(&rhs, &locals, defines, scope, &mut active) else {
            continue;
        };
        let mask: i128 = (1i128 << bits) - 1;
        if r_value & mask == c_value & mask {
            continue;
        }
        if seen.insert((name.clone(), line), ()).is_some() {
            continue;
        }
        let (cp, cl) = defines.site(&name, scope).unwrap_or(("?", 0));
        out.push(format!(
            "{line}: shadowed-c-define {name} rust={} c={} ({cp}:{cl})",
            show(r_value & mask),
            show(c_value & mask)
        ));
    }
    out
}

fn eval_rust(
    rhs: &str,
    locals: &HashMap<String, String>,
    defines: &CDefines,
    scope: Option<&str>,
    active: &mut HashSet<String>,
) -> Option<i128> {
    eval(
        rhs,
        &mut |id: &str| {
            if let Some(body) = locals.get(id) {
                if !active.insert(id.to_string()) {
                    return None;
                }
                let v = eval_rust(body, locals, defines, scope, active);
                active.remove(id);
                v
            } else {
                defines.value(id, scope)
            }
        },
        Dialect::Rust,
    )
}

fn show(v: i128) -> String {
    if v > 9 {
        format!("{v:#x}")
    } else {
        v.to_string()
    }
}

fn type_bits(ty: &str) -> Option<u32> {
    let ty = ty.rsplit("::").next().unwrap_or(ty);
    Some(match ty {
        "u8" | "i8" | "UCHAR" | "CHAR" | "BOOLEAN" | "BYTE" | "c_uchar" | "c_char" | "SCHAR" => 8,
        "u16" | "i16" | "USHORT" | "SHORT" | "WORD" | "WCHAR" | "c_ushort" | "c_short" => 16,
        "u32" | "i32" | "ULONG" | "LONG" | "DWORD" | "UINT" | "INT" | "NTSTATUS" | "KPRIORITY"
        | "c_ulong" | "c_long" | "c_uint" | "c_int" | "HRESULT" | "BOOL" | "ACCESS_MASK" => 32,
        "u64" | "i64" | "ULONGLONG" | "LONGLONG" | "ULONG64" | "LONG64" | "DWORD64" => 64,
        // Pointer-width: compare at 32 bits, which every target shares.
        "usize" | "isize" | "ULONG_PTR" | "LONG_PTR" | "SIZE_T" | "SSIZE_T" | "UINT_PTR"
        | "INT_PTR" | "DWORD_PTR" => 32,
        _ => return None,
    })
}

#[derive(Clone, Copy, PartialEq)]
enum Dialect {
    C,
    Rust,
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(i128),
    Id(String),
    Op(&'static str),
}

fn lex(s: &str, d: Dialect) -> Option<Vec<Tok>> {
    const OPS: &[&str] = &[
        "<<", ">>", "::", "+", "-", "*", "/", "%", "&", "|", "^", "~", "(", ")", "!",
    ];
    let b = s.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();
    'outer: while i < b.len() {
        let ch = b[i];
        if ch.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if ch.is_ascii_digit() {
            let start = i;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            out.push(Tok::Num(number(&s[start..i], d)?));
            continue;
        }
        if ch.is_ascii_alphabetic() || ch == b'_' {
            let start = i;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            out.push(Tok::Id(s[start..i].to_string()));
            continue;
        }
        for op in OPS {
            if s[i..].starts_with(op) {
                out.push(Tok::Op(op));
                i += op.len();
                continue 'outer;
            }
        }
        return None;
    }
    Some(out)
}

fn number(raw: &str, d: Dialect) -> Option<i128> {
    let clean = raw.replace('_', "");
    let lower = clean.to_ascii_lowercase();
    let (digits, radix) = if let Some(h) = lower.strip_prefix("0x") {
        (h.to_string(), 16)
    } else if let Some(bin) = lower.strip_prefix("0b").filter(|_| d == Dialect::Rust) {
        (bin.to_string(), 2)
    } else if d == Dialect::C && lower.len() > 1 && lower.starts_with('0') {
        (lower[1..].to_string(), 8)
    } else {
        (lower.clone(), 10)
    };
    let digits = match d {
        Dialect::C => digits
            .trim_end_matches(|c| c == 'u' || c == 'l')
            .to_string(),
        Dialect::Rust => {
            let mut s = digits;
            for suf in [
                "usize", "isize", "u128", "i128", "u64", "i64", "u32", "i32", "u16", "i16", "u8",
                "i8",
            ] {
                if radix != 16 || suf.len() > 2 {
                    if let Some(x) = s.strip_suffix(suf) {
                        s = x.to_string();
                        break;
                    }
                }
            }
            s
        }
    };
    if digits.is_empty() {
        return None;
    }
    i128::from_str_radix(&digits, radix).ok()
}

/// Integer value of a C macro body or Rust const initializer, or None when it is not
/// a closed integer expression (a string, a call, a struct, an unknown name).
fn eval(s: &str, lookup: &mut dyn FnMut(&str) -> Option<i128>, d: Dialect) -> Option<i128> {
    let toks = lex(s, d)?;
    if toks.is_empty() {
        return None;
    }
    let mut p = Parser {
        toks,
        pos: 0,
        lookup,
        d,
    };
    let v = p.expr(0)?;
    (p.pos == p.toks.len()).then_some(v)
}

struct Parser<'a> {
    toks: Vec<Tok>,
    pos: usize,
    lookup: &'a mut dyn FnMut(&str) -> Option<i128>,
    d: Dialect,
}
impl Parser<'_> {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }
    fn expr(&mut self, min: u8) -> Option<i128> {
        let mut lhs = self.unary()?;
        loop {
            // Rust `as T` casts: the comparison already happens at the declared width.
            if self.d == Dialect::Rust && matches!(self.peek(), Some(Tok::Id(k)) if k == "as") {
                self.pos += 1;
                self.path()?;
                continue;
            }
            let Some(Tok::Op(op)) = self.peek() else {
                break;
            };
            let (prec, op) = match *op {
                "*" | "/" | "%" => (6, *op),
                "+" | "-" => (5, *op),
                "<<" | ">>" => (4, *op),
                "&" => (3, *op),
                "^" => (2, *op),
                "|" => (1, *op),
                _ => break,
            };
            if prec < min {
                break;
            }
            self.pos += 1;
            let rhs = self.expr(prec + 1)?;
            lhs = match op {
                "*" => lhs.checked_mul(rhs)?,
                "/" => lhs.checked_div(rhs)?,
                "%" => lhs.checked_rem(rhs)?,
                "+" => lhs.checked_add(rhs)?,
                "-" => lhs.checked_sub(rhs)?,
                "<<" => lhs.checked_shl(u32::try_from(rhs).ok().filter(|s| *s < 100)?)?,
                ">>" => lhs.checked_shr(u32::try_from(rhs).ok().filter(|s| *s < 100)?)?,
                "&" => lhs & rhs,
                "^" => lhs ^ rhs,
                _ => lhs | rhs,
            };
        }
        Some(lhs)
    }
    fn path(&mut self) -> Option<String> {
        let Some(Tok::Id(mut last)) = self.peek().cloned() else {
            return None;
        };
        self.pos += 1;
        while matches!(self.peek(), Some(Tok::Op("::"))) {
            self.pos += 1;
            let Some(Tok::Id(next)) = self.peek().cloned() else {
                return None;
            };
            last = next;
            self.pos += 1;
        }
        Some(last)
    }
    fn unary(&mut self) -> Option<i128> {
        match self.peek()?.clone() {
            Tok::Op("-") => {
                self.pos += 1;
                Some(-self.unary()?)
            }
            Tok::Op("+") => {
                self.pos += 1;
                self.unary()
            }
            Tok::Op("~") | Tok::Op("!")
                if self.d == Dialect::C || matches!(self.peek(), Some(Tok::Op("!"))) =>
            {
                self.pos += 1;
                // 64-bit complement; the width mask applies at comparison.
                Some(!self.unary()? & ((1i128 << 64) - 1))
            }
            Tok::Op("(") => {
                self.pos += 1;
                // A C cast `(TYPE)x`: a lone identifier in parens followed by an operand.
                if self.d == Dialect::C {
                    if let (Some(Tok::Id(_)), Some(Tok::Op(")"))) = (
                        self.toks.get(self.pos).cloned(),
                        self.toks.get(self.pos + 1).cloned(),
                    ) {
                        let operand_follows = matches!(
                            self.toks.get(self.pos + 2),
                            Some(Tok::Num(_))
                                | Some(Tok::Id(_))
                                | Some(Tok::Op("("))
                                | Some(Tok::Op("-"))
                                | Some(Tok::Op("~"))
                        );
                        if operand_follows {
                            self.pos += 2;
                            return self.unary();
                        }
                    }
                }
                let v = self.expr(0)?;
                matches!(self.peek(), Some(Tok::Op(")"))).then(|| self.pos += 1)?;
                Some(v)
            }
            Tok::Num(n) => {
                self.pos += 1;
                Some(n)
            }
            Tok::Id(_) => {
                let id = self.path()?;
                if matches!(self.peek(), Some(Tok::Op("("))) {
                    return None; // a call or function-like macro
                }
                (self.lookup)(&id)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defs(files: &[(&str, &str)]) -> CDefines {
        CDefines::collect(files.iter().copied())
    }

    #[test]
    fn invented_boost_priority_is_reported() {
        let d = defs(&[
            (
                "sdk/include/ndk/ketypes.h",
                "#define LOW_REALTIME_PRIORITY 16\n",
            ),
            (
                "ntoskrnl/ke/balmgr.c",
                "#define THREAD_BOOST_PRIORITY (LOW_REALTIME_PRIORITY - 1)\n",
            ),
        ]);
        let out = scan_rust("const THREAD_BOOST_PRIORITY: ULONG = 23;\n", &d, None);
        assert_eq!(
            out,
            ["1: shadowed-c-define THREAD_BOOST_PRIORITY rust=0x17 c=0xf (ntoskrnl/ke/balmgr.c:1)"]
        );
    }
    #[test]
    fn the_fix_is_clean_and_resolves_through_rust_locals() {
        let d = defs(&[
            ("a.h", "#define LOW_REALTIME_PRIORITY 16\n"),
            (
                "b.c",
                "#define THREAD_BOOST_PRIORITY (LOW_REALTIME_PRIORITY - 1)\n",
            ),
        ]);
        let src = "const LOW_REALTIME_PRIORITY: ULONG = 16;\nconst THREAD_BOOST_PRIORITY: ULONG = LOW_REALTIME_PRIORITY - 1;\n";
        assert!(scan_rust(src, &d, None).is_empty());
    }
    #[test]
    fn another_modules_macro_is_not_visible() {
        let d = defs(&[
            ("dll/win32/advapi32/wine/crypt.c", "#define MAX_CPUS 256\n"),
            ("hal/halarm/gic.c", "#define OTHER 1\n"),
        ]);
        let scope = crate_scope("sdk/lib/rust/hal-rust/src/arm64/gic3.rs");
        assert_eq!(scope.as_deref(), Some("hal"));
        assert!(scan_rust("const MAX_CPUS: usize = 64;", &d, scope.as_deref()).is_empty());
        let d = defs(&[("ntoskrnl/ex/work.c", "#define EX_DELAYED_WORK_THREADS 12\n")]);
        let scope = crate_scope("sdk/lib/rust/ex-rust/src/work.rs");
        assert_eq!(
            scan_rust(
                "const EX_DELAYED_WORK_THREADS: ULONG = 3;",
                &d,
                scope.as_deref()
            )
            .len(),
            1
        );
    }
    #[test]
    fn disagreeing_c_definitions_are_skipped() {
        let d = defs(&[("a.h", "#ifdef X\n#define N 1\n#else\n#define N 2\n#endif\n")]);
        assert!(scan_rust("const N: u32 = 3;", &d, None).is_empty());
    }
    #[test]
    fn width_and_casts_compare_like_the_type() {
        let d = defs(&[(
            "a.h",
            "#define STATUS_X ((NTSTATUS)0xC0000005L)\n#define ALL (~0)\n",
        )]);
        assert!(scan_rust(
            "pub const STATUS_X: NTSTATUS = 0xC0000005u32 as i32;\nconst ALL: u32 = 0xFFFF_FFFF;",
            &d,
            None
        )
        .is_empty());
        assert_eq!(
            scan_rust(
                "const STATUS_X: NTSTATUS = 0xC0000006u32 as NTSTATUS;",
                &d,
                None
            )
            .len(),
            1
        );
    }
    #[test]
    fn unsized_types_strings_and_calls_are_skipped() {
        let d = defs(&[("a.h", "#define A 1\n#define B 2\n#define NAME \"x\"\n")]);
        assert!(scan_rust(
            "const A: Foo = 5;\nconst B: u32 = f(1);\nconst NAME: &str = \"y\";",
            &d,
            None
        )
        .is_empty());
    }
    #[test]
    fn comments_and_continuations_in_c() {
        let d = defs(&[(
            "a.h",
            "/* #define A 9 */\n#define A \\\n  (1 << 4) // sixteen\n",
        )]);
        assert!(scan_rust("const A: u32 = 16;", &d, None).is_empty());
        assert_eq!(scan_rust("const A: u32 = 15;", &d, None).len(), 1);
    }
}
