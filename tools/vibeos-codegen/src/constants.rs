//! Deterministic C enum and IOCTL constant extraction for Rust bindings.
//!
//! Text-oriented counterpart to `tools/gen_enum_consts.py` and
//! `tools/gen_ioctl_consts.py`; callers own file discovery and CLI handling.
use regex::Regex;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct EnumOptions {
    pub rust_type: String,
    pub only: Vec<String>,
    pub by_member: bool,
    pub source_name: String,
}
impl Default for EnumOptions {
    fn default() -> Self {
        Self {
            rust_type: "u32".into(),
            only: vec![],
            by_member: false,
            source_name: "<header>".into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct IoctlOptions {
    pub prefixes: Vec<String>,
    pub rust_type: String,
    pub source_name: String,
}
impl Default for IoctlOptions {
    fn default() -> Self {
        Self {
            prefixes: vec![],
            rust_type: "u32".into(),
            source_name: "<header>".into(),
        }
    }
}

pub fn enum_consts(header: &str, tag: &str, opts: &EnumOptions) -> Result<String, String> {
    let clean = strip_comments(header);
    let body = if opts.by_member {
        enum_body_for_member(&clean, tag)?
    } else {
        enum_body(&clean, tag)?
    }
    .ok_or_else(|| {
        format!(
            "no {} named {tag} in {}",
            if opts.by_member {
                "member of any enum"
            } else {
                "enum"
            },
            opts.source_name
        )
    })?;
    let mut members = enum_members(&body)?;
    if members.is_empty() {
        return Err(format!("empty enum body for {tag}"));
    }
    if !opts.only.is_empty() {
        let names: std::collections::HashSet<_> = members.iter().map(|(n, _)| n.as_str()).collect();
        let missing: Vec<_> = opts
            .only
            .iter()
            .filter(|n| !names.contains(n.as_str()))
            .cloned()
            .collect();
        if !missing.is_empty() {
            return Err(format!(
                "not members of {tag} in {}: {}",
                opts.source_name,
                missing.join(", ")
            ));
        }
        members.retain(|(n, _)| opts.only.contains(n));
    }
    let mut out = format!("// enum {tag} ({}), transcribed by tools/gen_enum_consts.py.\n// Values COMPUTED from the header — never hand-typed (§9.113). These arms\n// were bare identifiers with no definition in scope, i.e. catch-all\n// bindings that made the whole dispatch take the first arm (§9.112a).\n", opts.source_name);
    for (name, value) in members {
        out.push_str(&format!(
            "pub const {name}: {} = {value};\n",
            opts.rust_type
        ));
    }
    Ok(out)
}

fn strip_comments(s: &str) -> String {
    let block = Regex::new(r"(?s)/\*.*?\*/").unwrap();
    let line = Regex::new(r"//[^\n]*").unwrap();
    line.replace_all(&block.replace_all(s, " "), " ")
        .into_owned()
}
fn matching_body(s: &str, brace: usize) -> Result<String, String> {
    let b = s.as_bytes();
    let (mut i, mut d) = (brace + 1, 1usize);
    let start = i;
    while i < b.len() && d > 0 {
        if b[i] == b'{' {
            d += 1
        } else if b[i] == b'}' {
            d -= 1
        }
        i += 1;
    }
    if d == 0 {
        Ok(s[start..i - 1].into())
    } else {
        Err("unterminated enum body".into())
    }
}
fn enum_body(s: &str, tag: &str) -> Result<Option<String>, String> {
    let re = Regex::new(&format!(r"\benum\s+{}\s*\{{", regex::escape(tag))).unwrap();
    re.find(s)
        .map(|m| matching_body(s, m.end() - 1))
        .transpose()
}
fn enum_body_for_member(s: &str, member: &str) -> Result<Option<String>, String> {
    let re = Regex::new(r"\benum\b[^{{;]*\{").unwrap();
    let mem = Regex::new(&format!(r"(^|,)\s*{}\s*(=|,|$)", regex::escape(member))).unwrap();
    let mut hits = Vec::new();
    for m in re.find_iter(s) {
        let body = matching_body(s, m.end() - 1)?;
        if mem.is_match(&body) {
            hits.push(body)
        }
    }
    if hits.len() > 1 {
        return Err(format!(
            "{member:?} appears in {} enums here — name the tag instead",
            hits.len()
        ));
    }
    Ok(hits.into_iter().next())
}
fn enum_members(body: &str) -> Result<Vec<(String, i128)>, String> {
    let mut out = Vec::new();
    let mut syms: HashMap<String, i128> = HashMap::new();
    let empty_obj = Obj::new();
    let empty_func = Func::new();
    let no_macros = (&empty_obj, &empty_func);
    let mut next = 0i128;
    for raw in body.split(',') {
        let item = raw.trim();
        if item.is_empty() {
            continue;
        }
        let mut p = item.splitn(2, '=');
        let name = p.next().unwrap().trim();
        if !is_ident(name) {
            return Err(format!("cannot parse enumerator {item:?}"));
        }
        let value = if let Some(expr) = p.next() {
            eval_expr(expr, &syms, &no_macros, 0)?
        } else {
            next
        };
        out.push((name.into(), value));
        syms.insert(name.into(), value);
        next = value
            .checked_add(1)
            .ok_or_else(|| "enum implicit increment overflow".to_string())?;
    }
    Ok(out)
}
fn is_ident(s: &str) -> bool {
    let mut c = s.chars();
    matches!(c.next(),Some(x) if x=='_'||x.is_ascii_alphabetic())
        && c.all(|x| x == '_' || x.is_ascii_alphanumeric())
}

type Obj = HashMap<String, (String, String)>;
type Func = HashMap<String, (Vec<String>, String)>;
pub fn ioctl_consts(
    header: &str,
    other_headers: &[&str],
    opts: &IoctlOptions,
) -> Result<String, String> {
    let (own, _) = read_defines(&[header]);
    let mut alltexts = Vec::with_capacity(other_headers.len() + 1);
    alltexts.push(header);
    alltexts.extend_from_slice(other_headers);
    let (table, funcs) = read_defines(&alltexts);
    let mut out = Vec::new();
    let mut failures = Vec::new();
    let mut skipped = 0usize;
    let mut entries: Vec<_> = own.iter().collect();
    entries.sort_by_key(|(n, _)| *n);
    for (name, (rhs, _)) in entries {
        if !opts.prefixes.is_empty() && !opts.prefixes.iter().any(|p| name.starts_with(p)) {
            continue;
        }
        let call = macro_call(rhs);
        if !call.as_ref().is_some_and(|(n, _)| funcs.contains_key(n)) {
            if let Some(v) = literal(rhs) {
                out.push((name.clone(), v, rhs.clone()));
            } else {
                skipped += 1;
            }
            continue;
        }
        match eval_expr(rhs, &HashMap::new(), &(&table, &funcs), 0) {
            Ok(v) => out.push((name.clone(), (v as u32) as i128, rhs.clone())),
            Err(e) => failures.push(format!("{name}: {e}")),
        }
    }
    if !failures.is_empty() {
        return Err(format!(
            "could not evaluate these definitions:\n  {}",
            failures.join("\n  ")
        ));
    }
    if out.is_empty() {
        return Err(format!(
            "no evaluable macro defines matched in {} ({skipped} non-numeric define(s) skipped)",
            opts.source_name
        ));
    }
    let mut text = format!(
        "// Generated by tools/gen_ioctl_consts.py from {}.\n// bindgen cannot emit function-like macros, so these CTL_CODE() IOCTLs\n// can never come from the bindgen island (field manual §9.112a). Values\n// are COMPUTED from the header, never hand-typed — see §9.113.\n",
        opts.source_name
    );
    for (name, value, rhs) in out {
        text.push_str(&format!(
            "pub const {name}: {} = {}; // {rhs}\n",
            opts.rust_type,
            format_hex(value)
        ));
    }
    Ok(text)
}

fn read_defines(texts: &[&str]) -> (Obj, Func) {
    let (mut obj, mut fun) = (Obj::new(), Func::new());
    let fr =
        Regex::new(r"(?m)^[ \t]*#[ \t]*define[ \t]+([A-Za-z_]\w*)\(([^)]*)\)[ \t]+(.*?)[ \t]*$")
            .unwrap();
    let or = Regex::new(r"(?m)^[ \t]*#[ \t]*define[ \t]+([A-Za-z_]\w*)[ \t]+(.*?)[ \t]*$").unwrap();
    for (idx, src) in texts.iter().enumerate() {
        let folded = src.replace("\\\n", " ");
        for c in fr.captures_iter(&folded) {
            let n = &c[1];
            if !fun.contains_key(n) {
                let params = c[2]
                    .split(',')
                    .map(str::trim)
                    .filter(|x| !x.is_empty())
                    .map(str::to_owned)
                    .collect();
                fun.insert(n.into(), (params, trim_comment(&c[3])));
            }
        }
        for c in or.captures_iter(&folded) {
            let n = &c[1];
            if !obj.contains_key(n) {
                obj.insert(n.into(), (trim_comment(&c[2]), format!("header[{idx}]")));
            }
        }
    }
    (obj, fun)
}
fn trim_comment(s: &str) -> String {
    let a = s.split("/*").next().unwrap_or(s);
    a.split("//").next().unwrap_or(a).trim().into()
}
fn macro_call(s: &str) -> Option<(String, String)> {
    let re = Regex::new(r"(?s)^\s*([A-Za-z_]\w*)\s*\((.*)\)\s*$").unwrap();
    let c = re.captures(s)?;
    Some((c[1].into(), c[2].into()))
}
fn split_args(s: &str) -> Vec<String> {
    let (mut out, mut depth, mut start) = (Vec::new(), 0i32, 0usize);
    for (i, c) in s.char_indices() {
        if c == ',' && depth == 0 {
            out.push(s[start..i].trim().into());
            start = i + 1
        } else if c == '(' {
            depth += 1
        } else if c == ')' {
            depth -= 1
        }
    }
    out.push(s[start..].trim().into());
    out
}
fn literal(s: &str) -> Option<i128> {
    let t = s
        .trim()
        .trim_end_matches(|c: char| matches!(c, 'u' | 'U' | 'l' | 'L'));
    parse_int(t)
}
fn parse_int(s: &str) -> Option<i128> {
    let normalized = s.trim().replace('_', "");
    let t = normalized.as_str();
    if let Some(x) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        i128::from_str_radix(x, 16).ok()
    } else {
        t.parse().ok()
    }
}

// Small precedence parser for the integer operators used by these headers.
fn eval_expr(
    expr: &str,
    syms: &HashMap<String, i128>,
    macros: &(&Obj, &Func),
    depth: u8,
) -> Result<i128, String> {
    if depth > 24 {
        return Err(format!("recursion limit on {expr:?}"));
    }
    let mut e = expr.trim();
    while e.starts_with('(') && e.ends_with(')') && balanced(&e[1..e.len() - 1]) {
        e = e[1..e.len() - 1].trim();
    }
    if let Some(v) = literal(e) {
        return Ok(v);
    }
    if is_ident(e) {
        if let Some(v) = syms.get(e) {
            return Ok(*v);
        }
        let (obj, _func) = macros;
        if let Some((rhs, _)) = obj.get(e) {
            return eval_expr(rhs, syms, macros, depth + 1);
        }
        return Err(format!("undefined symbol {e}"));
    }
    // Ignore conventional scalar C casts, as the Python generator does.
    let cast = Regex::new(r"^\(\s*(?:unsigned\s+|signed\s+)?[A-Za-z_]\w*\s*\)\s*(.+)$").unwrap();
    if let Some(c) = cast.captures(e) {
        return eval_expr(&c[1], syms, macros, depth + 1);
    }
    if let Some((name, args)) = macro_call(e) {
        let (_, func) = macros;
        if let Some((params, body)) = func.get(&name) {
            let values = split_args(&args);
            if values.len() != params.len() {
                return Err(format!(
                    "{name} takes {} args, got {}",
                    params.len(),
                    values.len()
                ));
            }
            let binds: HashMap<_, _> = params
                .iter()
                .zip(values)
                .map(|(p, v)| (p.as_str(), v))
                .collect();
            let re = Regex::new(r"\b[A-Za-z_]\w*\b").unwrap();
            let expanded = re
                .replace_all(body, |c: &regex::Captures| {
                    binds
                        .get(&c[0])
                        .map(|v| format!("({v})"))
                        .unwrap_or_else(|| c[0].into())
                })
                .into_owned();
            return eval_expr(&expanded, syms, macros, depth + 1);
        }
    }
    let toks = tokenize(e)?;
    let mut p = Parser {
        toks,
        pos: 0,
        syms,
        macros,
        depth,
    };
    let v = p.parse(0)?;
    if p.pos != p.toks.len() {
        return Err(format!("cannot evaluate {e:?}"));
    }
    Ok(v)
}

/// Evaluate an already-rendered integer constant using the same checked
/// integer-expression rules used by enum and IOCTL extraction.
pub fn evaluate_integer(expr: &str) -> Result<i128, String> {
    let obj = Obj::new();
    let funcs = Func::new();
    eval_expr(expr, &HashMap::new(), &(&obj, &funcs), 0)
}
fn format_hex(v: i128) -> String {
    if v < 0 {
        format!("-0x{:07x}", v.unsigned_abs())
    } else {
        format!("0x{:08x}", v)
    }
}
fn balanced(s: &str) -> bool {
    let mut d = 0;
    for c in s.chars() {
        if c == '(' {
            d += 1
        } else if c == ')' {
            d -= 1;
            if d < 0 {
                return false;
            }
        }
    }
    d == 0
}
#[derive(Debug, Clone)]
enum Tok {
    Num(i128),
    Id(String),
    Op(String),
    Lp,
    Rp,
}
fn tokenize(s: &str) -> Result<Vec<Tok>, String> {
    let b = s.as_bytes();
    let (mut i, mut v) = (0, Vec::new());
    while i < b.len() {
        if b[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if b[i].is_ascii_digit() {
            let st = i;
            i += 1;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1
            }
            let t = s[st..i].trim_end_matches(|c: char| matches!(c, 'u' | 'U' | 'l' | 'L'));
            v.push(Tok::Num(
                parse_int(t).ok_or_else(|| format!("invalid integer {t}"))?,
            ));
            continue;
        }
        if b[i].is_ascii_alphabetic() || b[i] == b'_' {
            let st = i;
            i += 1;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1
            }
            v.push(Tok::Id(s[st..i].into()));
            continue;
        }
        if b[i] == b'(' {
            v.push(Tok::Lp);
            i += 1;
            continue;
        }
        if b[i] == b')' {
            v.push(Tok::Rp);
            i += 1;
            continue;
        }
        let mut found = None;
        for op in ["<<", ">>", "+", "-", "*", "/", "%", "|", "&", "^", "~"] {
            if s[i..].starts_with(op) {
                found = Some(op);
                break;
            }
        }
        if let Some(op) = found {
            v.push(Tok::Op(op.into()));
            i += op.len()
        } else {
            return Err(format!("unsupported syntax in {s:?}"));
        }
    }
    Ok(v)
}
struct Parser<'a> {
    toks: Vec<Tok>,
    pos: usize,
    syms: &'a HashMap<String, i128>,
    macros: &'a (&'a Obj, &'a Func),
    depth: u8,
}
impl Parser<'_> {
    fn parse(&mut self, min: u8) -> Result<i128, String> {
        let mut lhs = match self.toks.get(self.pos).cloned() {
            Some(Tok::Num(n)) => {
                self.pos += 1;
                n
            }
            Some(Tok::Id(n)) => {
                self.pos += 1;
                eval_expr(&n, self.syms, self.macros, self.depth + 1)?
            }
            Some(Tok::Op(ref o)) if o == "~" || o == "+" || o == "-" => {
                let op = o.clone();
                self.pos += 1;
                let x = self.parse(11)?;
                match op.as_str() {
                    "~" => !x,
                    "-" => x.checked_neg().ok_or("integer overflow")?,
                    _ => x,
                }
            }
            Some(Tok::Lp) => {
                self.pos += 1;
                let v = self.parse(0)?;
                if !matches!(self.toks.get(self.pos), Some(Tok::Rp)) {
                    return Err("unbalanced parentheses".into());
                }
                self.pos += 1;
                v
            }
            _ => return Err("expected integer expression".into()),
        };
        loop {
            let op = match self.toks.get(self.pos) {
                Some(Tok::Op(o)) => o.as_str(),
                _ => break,
            };
            let prec = match op {
                "|" => 1,
                "^" => 2,
                "&" => 3,
                "<<" | ">>" => 4,
                "+" | "-" => 5,
                "*" | "/" | "%" => 6,
                _ => break,
            };
            if prec < min {
                break;
            }
            let op = op.to_string();
            self.pos += 1;
            let rhs = self.parse(prec + 1)?;
            lhs = match op.as_str() {
                "|" => lhs | rhs,
                "^" => lhs ^ rhs,
                "&" => lhs & rhs,
                "<<" => {
                    let n = u32::try_from(rhs).map_err(|_| "invalid shift count")?;
                    let v = lhs.checked_shl(n).ok_or("shift overflow")?;
                    if n >= 127 || (v >> n) != lhs {
                        return Err("shift overflow".into());
                    }
                    v
                }
                ">>" => {
                    let n = u32::try_from(rhs).map_err(|_| "invalid shift count")?;
                    lhs.checked_shr(n).ok_or("invalid shift count")?
                }
                "+" => lhs.checked_add(rhs).ok_or("integer overflow")?,
                "-" => lhs.checked_sub(rhs).ok_or("integer overflow")?,
                "*" => lhs.checked_mul(rhs).ok_or("integer overflow")?,
                "/" => lhs.checked_div(rhs).ok_or("division by zero")?,
                "%" => lhs.checked_rem(rhs).ok_or("division by zero")?,
                _ => unreachable!(),
            }
        }
        Ok(lhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn enum_implicit_alias_and_expression() {
        let h = "enum E { A, B = A + 4, C, D = (B << 2) | 3, };";
        let s = enum_consts(h, "E", &EnumOptions::default()).unwrap();
        assert!(s.contains("A: u32 = 0"));
        assert!(s.contains("B: u32 = 4"));
        assert!(s.contains("C: u32 = 5"));
        assert!(s.contains("D: u32 = 19"));
    }
    #[test]
    fn enum_and_ioctl_banners_match_scripts() {
        let e = enum_consts("enum E { A };", "E", &EnumOptions::default()).unwrap();
        assert!(e.contains("Values COMPUTED from the header — never hand-typed (§9.113). These arms\n// were bare identifiers with no definition in scope, i.e. catch-all\n// bindings that made the whole dispatch take the first arm (§9.112a)."));
        let i = ioctl_consts(
            "#define F(x) (x)\n#define IOCTL_X F(1)\n",
            &[],
            &IoctlOptions::default(),
        )
        .unwrap();
        assert!(i.contains("// bindgen cannot emit function-like macros, so these CTL_CODE() IOCTLs\n// can never come from the bindgen island (field manual §9.112a). Values\n// are COMPUTED from the header, never hand-typed — see §9.113."));
    }
    #[test]
    fn evaluator_checks_overflow_and_parses_i128() {
        assert_eq!(
            evaluate_integer("0x7fff_ffff_ffff_ffff").unwrap(),
            0x7fff_ffff_ffff_ffff
        );
        assert!(evaluate_integer("170141183460469231731687303715884105727 + 1").is_err());
        assert!(evaluate_integer("1 << 127").is_err());
        assert!(enum_consts("enum E {", "E", &EnumOptions::default())
            .unwrap_err()
            .contains("unterminated"));
        assert!(enum_consts("enum E {}", "E", &EnumOptions::default())
            .unwrap_err()
            .contains("empty"));
    }
    #[test]
    fn enum_rejects_unknown_and_ambiguous() {
        assert!(enum_consts(
            "enum E{A};",
            "E",
            &EnumOptions {
                only: vec!["MISSING".into()],
                ..Default::default()
            }
        )
        .unwrap_err()
        .contains("not members"));
        assert!(enum_consts(
            "enum { X }; enum { X };",
            "X",
            &EnumOptions {
                by_member: true,
                ..Default::default()
            }
        )
        .unwrap_err()
        .contains("appears in 2 enums"));
    }
    #[test]
    fn ioctl_expands_function_macro_aliases() {
        let h="#define DEV 0x22\n#define M 0\n#define ACCESS 3\n#define CODE(d,f,m,a) (((d)<<16)|((a)<<14)|((f)<<2)|(m))\n#define IOCTL_X CODE(DEV, 7, M, ACCESS)\n#define IOCTL_ALIAS 0x1234U\n#define IOCTL_SKIP \"str\"\n";
        let s = ioctl_consts(
            h,
            &[],
            &IoctlOptions {
                prefixes: vec!["IOCTL_".into()],
                ..Default::default()
            },
        )
        .unwrap();
        assert!(s.contains("IOCTL_X: u32 = 0x0022c01c"));
        assert!(s.contains("IOCTL_ALIAS: u32 = 0x00001234"));
        assert!(!s.contains("IOCTL_SKIP"));
    }
    #[test]
    fn negative_literal_keeps_python_rendering() {
        let s = ioctl_consts("#define IOCTL_NEG -1\n", &[], &IoctlOptions::default()).unwrap();
        assert!(s.contains("IOCTL_NEG: u32 = -0x0000001;"));
    }
    #[test]
    fn ioctl_fails_on_bad_macro_arity() {
        let h = "#define F(a,b) ((a)|(b))\n#define IOCTL_BAD F(1)\n";
        assert!(ioctl_consts(h, &[], &IoctlOptions::default())
            .unwrap_err()
            .contains("takes 2 args"));
    }
}
