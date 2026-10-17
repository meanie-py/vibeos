//! Rust translations of mm.h section-segment-entry helpers checked on representative values.
use crate::{Corpus, Rule};
use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

pub struct SseHelpers;
impl Rule for SseHelpers {
    fn name(&self) -> &'static str {
        "scan_sse_helpers"
    }
    fn extra_files(&self) -> &'static [&'static str] {
        &["ntoskrnl/include/internal/mm.h"]
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let header = corpus
            .files
            .iter()
            .find(|f| f.path == std::path::Path::new("ntoskrnl/include/internal/mm.h"))
            .ok_or(
                "scan_sse_helpers requires ntoskrnl/include/internal/mm.h in the source corpus",
            )?;
        let macros = parse_macros(&header.text);
        let mut findings = Vec::new();
        for file in corpus.rust_files() {
            if patterns().skip.is_match(&file.path.to_string_lossy()) {
                continue;
            }
            let text = file
                .text
                .split("ORIGINAL C SOURCE")
                .next()
                .unwrap_or(&file.text);
            let funcs = rust_functions(text);
            for f in &funcs {
                let Some(def) = macros.get(&f.name) else {
                    continue;
                };
                if f.params.len() != def.params.len() {
                    continue;
                }
                let Some(got_expr) = rust_expr(&f.body) else {
                    continue;
                };
                // The original Python lambda rejects Rust-qualified calls with
                // SyntaxError and skips them. Cast paths have already been removed.
                if got_expr.contains("::") {
                    continue;
                }
                let samples = if def.params.len() == 1 {
                    samples_one()
                } else {
                    samples_two()
                };
                for args in samples {
                    let want = eval_macro(&f.name, &args, &macros, 0).map_err(|e| {
                        format!(
                            "{}: {} reference evaluation failed: {e}",
                            file.path.display(),
                            f.name
                        )
                    })?;
                    let got = eval_impl(&got_expr, &f.params, &args, &macros).map_err(|e| {
                        format!(
                            "{}: {} implementation evaluation failed: {e}",
                            file.path.display(),
                            f.name
                        )
                    })?;
                    let same = if f.name.starts_with("IS_") || f.name.starts_with("MM_IS_") {
                        (want != 0) == (got != 0)
                    } else {
                        (want as u128 as u64) == (got as u128 as u64)
                    };
                    if !same {
                        let line = text[..f.start].bytes().filter(|b| *b == b'\n').count() + 1;
                        let shown = args
                            .iter()
                            .map(|a| format!("{a:#x}"))
                            .collect::<Vec<_>>()
                            .join(", ");
                        findings.push(format!("{}:{line}: sse-helper-drift {}({shown}) = {got:#x}, mm.h says {want:#x}", file.path.display(), f.name));
                        break;
                    }
                }
            }
        }
        findings.sort();
        Ok(findings)
    }
}
fn eval_impl(
    expr: &str,
    params: &[String],
    args: &[i128],
    defs: &HashMap<String, MacroDef>,
) -> Result<i128, String> {
    if params.len() != args.len() {
        return Err("Rust implementation arity mismatch".into());
    }
    let vars: HashMap<_, _> = params.iter().cloned().zip(args.iter().copied()).collect();
    eval_expr(expr, &vars, defs, 0)
}
struct Patterns {
    skip: Regex,
    define: Regex,
    rust_fn: Regex,
    c_cast: Regex,
    suffix: Regex,
    rust_cast: Regex,
    rust_suffix: Regex,
    cfg64: Regex,
}
fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| Patterns {
        skip: crate::regexes::cached(
            r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/island_[a-z0-9_]+\.rs$)",
        )
        .unwrap(),
        define: crate::regexes::cached(
            r"^#define\s+(\w+_SSE|SHARE_COUNT_FROM_SSE)\((\w+)(?:,\s*(\w+))?\)\s+(.+)$",
        )
        .unwrap(),
        rust_fn: crate::regexes::cached(r"\bfn\s+(\w+)\s*\(([^)]*)\)\s*(?:->\s*[\w:]+\s*)?\{")
            .unwrap(),
        c_cast: crate::regexes::cached(r"\((?:PFN_NUMBER|ULONG_PTR|ULONG|SWAPENTRY)\)").unwrap(),
        suffix: crate::regexes::cached(r"\b(0x[0-9A-Fa-f]+|\d+)(?:ULL|UL|U|L)\b").unwrap(),
        rust_cast: crate::regexes::cached(r"\bas\s+[\w:]+").unwrap(),
        rust_suffix: crate::regexes::cached(
            r"\b(0x[0-9A-Fa-f_]+|\d[\d_]*)(?:u8|u16|u32|u64|usize|i32|i64|isize)\b",
        )
        .unwrap(),
        cfg64: crate::regexes::cached(r#"#\[cfg\(target_pointer_width\s*=\s*"64"\)\]\s*\{(.*?)\}"#)
            .unwrap(),
    })
}
#[derive(Clone)]
struct MacroDef {
    params: Vec<String>,
    expr: String,
}
fn parse_macros(header: &str) -> HashMap<String, MacroDef> {
    let normalized = header.replace("\\\n", " ");
    let mut in_else = false;
    let mut defs = HashMap::new();
    for line in normalized.lines() {
        let s = line.trim();
        if s.starts_with("#else") {
            in_else = true;
            continue;
        }
        if s.starts_with("#endif") {
            in_else = false;
            continue;
        }
        if in_else {
            continue;
        }
        let Some(cap) = patterns().define.captures(s) else {
            continue;
        };
        let name = cap[1].to_owned();
        let mut params = vec![cap[2].to_owned()];
        if let Some(p) = cap.get(3) {
            params.push(p.as_str().to_owned());
        }
        let expr = c_expr(&cap[4]);
        defs.entry(name).or_insert(MacroDef { params, expr });
    }
    defs
}
fn c_expr(raw: &str) -> String {
    let no_casts = patterns().c_cast.replace_all(raw, "");
    let no_suffix = patterns().suffix.replace_all(&no_casts, "$1");
    no_suffix.replace("PAGE_SHIFT", "12")
}
struct RustFn {
    name: String,
    params: Vec<String>,
    body: String,
    start: usize,
}
fn rust_functions(text: &str) -> Vec<RustFn> {
    let mut out = Vec::new();
    for cap in patterns().rust_fn.captures_iter(text) {
        let m = cap.get(0).unwrap();
        let args = cap[2]
            .split(',')
            .filter_map(|s| {
                let name = s.trim().split(':').next().unwrap_or("").trim();
                (!name.is_empty()).then(|| name.to_owned())
            })
            .collect();
        let open = m.end() - 1;
        let Some(end) = matching_brace(text, open) else {
            continue;
        };
        out.push(RustFn {
            name: cap[1].to_owned(),
            params: args,
            body: text[open + 1..end].to_owned(),
            start: m.start(),
        });
    }
    out
}
fn matching_brace(text: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (i, b) in text.bytes().enumerate().skip(open) {
        if b == b'{' {
            depth += 1;
        } else if b == b'}' {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}
fn rust_expr(raw: &str) -> Option<String> {
    let mut body = raw.to_owned();
    if let Some(c) = patterns().cfg64.captures(&body) {
        body = c[1].to_owned();
    }
    body = body
        .lines()
        .map(|l| l.split_once("//").map_or(l, |x| x.0))
        .collect::<Vec<_>>()
        .join("\n");
    let body = body.trim().trim_end_matches(';').trim();
    if ["let ", "return", "if ", "match ", "unsafe"]
        .iter()
        .any(|x| body.contains(x))
    {
        return None;
    }
    let no_casts = patterns().rust_cast.replace_all(body, "");
    let no_suffix = patterns().rust_suffix.replace_all(&no_casts, "$1");
    let mut out = no_suffix.replace("PAGE_SHIFT", "12");
    let chars: Vec<char> = out.chars().collect();
    out.clear();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '!' && chars.get(i + 1) != Some(&'=') {
            out.push('~');
        } else {
            out.push(chars[i]);
        }
        i += 1;
    }
    Some(out)
}
fn samples_one() -> Vec<Vec<i128>> {
    let pages = [0i128, 0x1000, 0x12345000, 0xFD4000, 0x3D0AD000];
    let mut out = Vec::new();
    for p in pages {
        for c in 0..6i128 {
            for f in 0..8i128 {
                out.push(vec![p | (c << 3) | f]);
            }
        }
    }
    out
}
fn samples_two() -> Vec<Vec<i128>> {
    [0x1000i128, 0x12345000]
        .into_iter()
        .flat_map(|p| (0..6i128).map(move |c| vec![p, c]))
        .collect()
}
fn eval_macro(
    name: &str,
    args: &[i128],
    defs: &HashMap<String, MacroDef>,
    depth: usize,
) -> Result<i128, String> {
    if depth > 32 {
        return Err("recursive macro".into());
    }
    let d = defs
        .get(name)
        .ok_or_else(|| format!("unknown macro {name}"))?;
    if args.len() != d.params.len() {
        return Err("macro arity mismatch".into());
    }
    let vars: HashMap<_, _> = d.params.iter().cloned().zip(args.iter().copied()).collect();
    eval_expr(&d.expr, &vars, defs, depth + 1)
}
fn eval_expr(
    expr: &str,
    vars: &HashMap<String, i128>,
    defs: &HashMap<String, MacroDef>,
    depth: usize,
) -> Result<i128, String> {
    let tokens = tokenize(expr)?;
    let mut parser = Parser {
        tokens,
        at: 0,
        vars,
        defs,
        depth,
    };
    let v = parser.parse(0)?;
    if parser.at != parser.tokens.len() {
        return Err(format!("trailing expression tokens: {}", expr));
    }
    Ok(v)
}
#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(i128),
    Id(String),
    Op(&'static str),
    Lp,
    Rp,
    Comma,
}
fn tokenize(s: &str) -> Result<Vec<Tok>, String> {
    let c: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    while i < c.len() {
        if c[i].is_whitespace() {
            i += 1;
            continue;
        }
        if c[i].is_ascii_digit() {
            let st = i;
            i += 1;
            while i < c.len()
                && (c[i].is_ascii_hexdigit() || c[i] == 'x' || c[i] == 'X' || c[i] == '_')
            {
                i += 1;
            }
            let raw: String = c[st..i].iter().collect();
            let clean = raw.replace('_', "");
            let (digits, radix) = if let Some(x) = clean
                .strip_prefix("0x")
                .or_else(|| clean.strip_prefix("0X"))
            {
                (x, 16)
            } else {
                (clean.as_str(), 10)
            };
            out.push(Tok::Num(
                i128::from_str_radix(digits, radix).map_err(|e| e.to_string())?,
            ));
            continue;
        }
        if c[i].is_ascii_alphabetic() || c[i] == '_' {
            let st = i;
            i += 1;
            while i < c.len() && (c[i].is_alphanumeric() || c[i] == '_') {
                i += 1;
            }
            let id: String = c[st..i].iter().collect();
            match id.as_str() {
                "and" => out.push(Tok::Op("&&")),
                "or" => out.push(Tok::Op("||")),
                _ => out.push(Tok::Id(id)),
            }
            continue;
        }
        if c[i] == '(' {
            out.push(Tok::Lp);
            i += 1;
            continue;
        }
        if c[i] == ')' {
            out.push(Tok::Rp);
            i += 1;
            continue;
        }
        if c[i] == ',' {
            out.push(Tok::Comma);
            i += 1;
            continue;
        }
        let mut found = None;
        for op in ["&&", "||", "==", "!=", "<=", ">=", "<<", ">>"] {
            let o: Vec<char> = op.chars().collect();
            if c[i..].starts_with(&o) {
                found = Some(op);
                break;
            }
        }
        if let Some(op) = found {
            out.push(Tok::Op(op));
            i += op.len();
            continue;
        }
        let one = match c[i] {
            '+' => Some("+"),
            '-' => Some("-"),
            '|' => Some("|"),
            '&' => Some("&"),
            '^' => Some("^"),
            '~' => Some("~"),
            '!' => Some("!"),
            '<' => Some("<"),
            '>' => Some(">"),
            _ => None,
        };
        if let Some(op) = one {
            out.push(Tok::Op(op));
            i += 1;
        } else {
            return Err(format!("unsupported expression character {}", c[i]));
        }
    }
    Ok(out)
}
struct Parser<'a> {
    tokens: Vec<Tok>,
    at: usize,
    vars: &'a HashMap<String, i128>,
    defs: &'a HashMap<String, MacroDef>,
    depth: usize,
}
impl Parser<'_> {
    fn parse(&mut self, min: u8) -> Result<i128, String> {
        let mut lhs = self.prefix()?;
        loop {
            let Some(Tok::Op(op)) = self.tokens.get(self.at) else {
                break;
            };
            let prec = precedence(op);
            if prec < min {
                break;
            }
            let op = *op;
            self.at += 1;
            let rhs = self.parse(prec + 1)?;
            lhs = apply(op, lhs, rhs)?;
        }
        Ok(lhs)
    }
    fn prefix(&mut self) -> Result<i128, String> {
        let tok = self
            .tokens
            .get(self.at)
            .cloned()
            .ok_or("missing expression operand")?;
        self.at += 1;
        match tok {
            Tok::Num(n) => Ok(n),
            Tok::Id(id) => {
                if self.tokens.get(self.at) == Some(&Tok::Lp) {
                    self.at += 1;
                    let mut args = Vec::new();
                    if self.tokens.get(self.at) != Some(&Tok::Rp) {
                        loop {
                            args.push(self.parse(0)?);
                            if self.tokens.get(self.at) == Some(&Tok::Comma) {
                                self.at += 1;
                                continue;
                            }
                            break;
                        }
                    }
                    if self.tokens.get(self.at) != Some(&Tok::Rp) {
                        return Err("missing call close".into());
                    }
                    self.at += 1;
                    eval_macro(&id, &args, self.defs, self.depth + 1)
                } else {
                    self.vars
                        .get(&id)
                        .copied()
                        .ok_or_else(|| format!("unknown identifier {id}"))
                }
            }
            Tok::Lp => {
                let v = self.parse(0)?;
                if self.tokens.get(self.at) != Some(&Tok::Rp) {
                    return Err("missing close paren".into());
                }
                self.at += 1;
                Ok(v)
            }
            Tok::Op("~") => Ok(!self.parse(13)?),
            Tok::Op("!") => Ok((self.parse(13)? == 0) as i128),
            Tok::Op("-") => Ok(-self.parse(13)?),
            Tok::Op("+") => self.parse(13),
            _ => Err("invalid expression prefix".into()),
        }
    }
}
fn precedence(op: &str) -> u8 {
    match op {
        "||" => 1,
        "&&" => 2,
        "==" | "!=" | "<" | ">" | "<=" | ">=" => 3,
        "|" => 4,
        "^" => 5,
        "&" => 6,
        "<<" | ">>" => 7,
        "+" | "-" => 8,
        _ => 0,
    }
}
fn apply(op: &str, a: i128, b: i128) -> Result<i128, String> {
    Ok(match op {
        "||" => (a != 0 || b != 0) as i128,
        "&&" => (a != 0 && b != 0) as i128,
        "==" => (a == b) as i128,
        "!=" => (a != b) as i128,
        "<" => (a < b) as i128,
        ">" => (a > b) as i128,
        "<=" => (a <= b) as i128,
        ">=" => (a >= b) as i128,
        "|" => a | b,
        "^" => a ^ b,
        "&" => a & b,
        "<<" => a.checked_shl(b as u32).ok_or("shift overflow")?,
        ">>" => a.checked_shr(b as u32).ok_or("shift overflow")?,
        "+" => a.checked_add(b).ok_or("addition overflow")?,
        "-" => a.checked_sub(b).ok_or("subtraction overflow")?,
        _ => return Err(format!("unknown operator {op}")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceFile;
    use std::path::PathBuf;
    const MM_H: &str = "#define IS_DIRTY_SSE(E)          ((E) & 2)\n#ifdef _WIN64\n#define PAGE_FROM_SSE(E)         ((E) & 0xFFFFFFF000ULL)\n#else\n#define PAGE_FROM_SSE(E)         ((E) & 0xFFFFF000)\n#endif\n#define SHARE_COUNT_FROM_SSE(E)  (((E) & 0x00000FFC) >> 3)\n#define BUMPREF_SSE(E)           (PAGE_FROM_SSE(E) | ((SHARE_COUNT_FROM_SSE(E) + 1) << 3) | ((E) & 0x7))\n";
    const BAD: &str = "#[inline]\npub fn IS_DIRTY_SSE(entry: ULONG_PTR) -> bool {\n    (entry & 0x00000008) != 0\n}\n";
    const GOOD: &str = "#[inline]\nfn IS_DIRTY_SSE(sse: ULONG_PTR) -> bool {\n    (sse & 0x2) != 0\n}\n#[inline]\nfn PAGE_FROM_SSE(sse: ULONG_PTR) -> ULONG_PTR {\n    #[cfg(target_pointer_width = \"64\")]\n    { sse & 0xFFFFFFFFF000 }\n    #[cfg(target_pointer_width = \"32\")]\n    { sse & 0xFFFFF000 }\n}\n#[inline]\nfn BUMPREF_SSE(sse: ULONG_PTR) -> ULONG_PTR {\n    PAGE_FROM_SSE(sse) | ((SHARE_COUNT_FROM_SSE(sse) + 1) << 3) | (sse & 0x7)\n}\n";
    fn corpus(rust: &str) -> Corpus {
        Corpus::new(
            PathBuf::from("/fixture"),
            vec![
                SourceFile::new("ntoskrnl/include/internal/mm.h".into(), MM_H.into()),
                SourceFile::new("sdk/lib/rust/x/src/sse.rs".into(), rust.into()),
            ],
        )
    }
    #[test]
    fn reports_original_drift_fixture() {
        assert_eq!(SseHelpers.run(&corpus(BAD)).unwrap(),vec!["sdk/lib/rust/x/src/sse.rs:2: sse-helper-drift IS_DIRTY_SSE(0x2) = 0x0, mm.h says 0x2"]);
    }
    #[test]
    fn mm_h_encodings_and_reference_helper_are_clean() {
        assert!(SseHelpers.run(&corpus(GOOD)).unwrap().is_empty());
    }
    #[test]
    fn missing_oracle_is_actionable() {
        let c = Corpus::new(PathBuf::from("/fixture"), vec![]);
        assert!(SseHelpers
            .run(&c)
            .unwrap_err()
            .contains("ntoskrnl/include/internal/mm.h"));
    }
    #[test]
    fn rust_qualified_wrapper_has_the_oracles_syntax_exemption() {
        assert!(SseHelpers
            .run(&corpus(
                "fn IS_DIRTY_SSE(sse: ULONG_PTR) -> bool { crate::mm::IS_DIRTY_SSE(sse) }"
            ))
            .unwrap()
            .is_empty());
    }
    #[test]
    fn implementation_parameters_receive_each_sampled_argument_value() {
        let params = vec!["page".to_owned(), "color".to_owned()];
        let args = [0x12345000i128, 5i128];
        let got = eval_impl("page | (color << 3)", &params, &args, &HashMap::new()).unwrap();
        assert_eq!(got, 0x12345028);
    }
}
