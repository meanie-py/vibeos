//! Compare integer literals annotated with C flag names against the captured header catalog.
use crate::{Corpus, Rule, SourceSet};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

pub struct InlineFlagLiterals;
const RUST: &[SourceSet] = &[
    SourceSet::tree("inline-flag-rust", &["sdk/lib/rust"], &["rs"], &[])
        .with_pruning(&["target", ".responses"])
        .with_inputs(),
    SourceSet::tree(
        "inline-flag-fixed-headers",
        &[
            "sdk/include/xdk/iotypes.h",
            "sdk/include/xdk/setypes.h",
            "sdk/include/psdk/ntstatus.h",
            "ntoskrnl/include/internal/tag.h",
            "ntoskrnl/include/internal/io.h",
            "sdk/include/xdk/ketypes.h",
            "sdk/include/ndk/ketypes.h",
            "sdk/include/ndk/pstypes.h",
            "sdk/include/ndk/obtypes.h",
            "sdk/include/ndk/mmtypes.h",
            "sdk/include/ndk/extypes.h",
            "sdk/include/ndk/lpctypes.h",
            "sdk/include/ndk/cmtypes.h",
            "sdk/include/ndk/iotypes.h",
            "sdk/include/reactos/drivers/ntddrdsk.h",
            "sdk/include/reactos/srmp.h",
            "sdk/include/psdk/ntiologc.h",
            "sdk/include/reactos/exeformat.h",
            "sdk/lib/rust/reactos-link/committed-headers/bugcodes.h",
            "sdk/include/xdk/ntbasedef.h",
            "sdk/include/psdk/winioctl.h",
            "sdk/include/psdk/ntdddisk.h",
            "sdk/include/psdk/ntddcdrm.h",
            "sdk/include/psdk/ntddstor.h",
            "sdk/include/psdk/ntddscsi.h",
            "sdk/include/psdk/ntddvol.h",
            "sdk/include/xdk/ntifs.template.h",
            "sdk/include/reactos/drivers/afd/shared.h",
            "sdk/include/psdk/ntddtdi.h",
            "sdk/include/psdk/ntddndis.h",
            "sdk/include/psdk/usbiodef.h",
        ],
        &["h"],
        &[],
    ),
    SourceSet::tree(
        "inline-flag-header-trees",
        &[
            "sdk/include/xdk",
            "sdk/include/ndk",
            "sdk/include/psdk",
            "ntoskrnl/include/internal",
        ],
        &["h"],
        &[],
    ),
    SourceSet::tree(
        "inline-flag-enum-headers",
        &[
            "sdk/include/xdk/iotypes.h",
            "sdk/include/xdk/setypes.h",
            "sdk/include/psdk/ntstatus.h",
            "ntoskrnl/include/internal/tag.h",
            "ntoskrnl/include/internal/io.h",
            "sdk/include/xdk/ketypes.h",
            "sdk/include/ndk/ketypes.h",
            "sdk/include/ndk/pstypes.h",
            "sdk/include/ndk/obtypes.h",
            "sdk/include/ndk/mmtypes.h",
            "sdk/include/ndk/extypes.h",
            "sdk/include/ndk/lpctypes.h",
            "sdk/include/ndk/cmtypes.h",
            "sdk/include/ndk/iotypes.h",
            "sdk/include/reactos/drivers/ntddrdsk.h",
            "sdk/include/reactos/srmp.h",
            "sdk/include/psdk/ntiologc.h",
            "sdk/include/reactos/exeformat.h",
            "sdk/lib/rust/reactos-link/committed-headers/bugcodes.h",
            "sdk/include/xdk/ntbasedef.h",
        ],
        &["h"],
        &[],
    ),
];
impl Rule for InlineFlagLiterals {
    fn name(&self) -> &'static str {
        "scan_inline_flag_literals"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        RUST
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let catalog = catalog(c);
        let mut out = Vec::new();
        for f in c.sources("inline-flag-rust") {
            if f.path.to_string_lossy().contains("/tests/")
                || f.path.to_string_lossy().contains("/target/")
            {
                continue;
            }
            let tree = f.rust_tree()?;
            let mut lits = Vec::new();
            let mut comments = Vec::new();
            collect(tree.root_node(), &mut lits, &mut comments);
            lits.sort_by_key(|n| n.start_byte());
            comments.sort_by_key(|n| n.start_byte());
            let lines: Vec<&str> = f.text.split('\n').collect();
            for cmt in comments {
                let names = comment_names(&f.text[cmt.start_byte()..cmt.end_byte()]);
                if names.is_empty() {
                    continue;
                }
                let original_line = cmt.start_position().row;
                let mut row = original_line;
                let mut line_src = lines.get(row).copied().unwrap_or("").trim().to_owned();
                let mut opener = false;
                if line_src.starts_with("//") || line_src.starts_with("/*") {
                    let nxt = lines.get(row + 1).copied().unwrap_or("").trim();
                    let prev = if row > 0 {
                        lines[row - 1].split("//").next().unwrap_or("").trim_end()
                    } else {
                        ""
                    };
                    if lone_lit().is_match(nxt) {
                        row += 1;
                        line_src = nxt.to_owned();
                    } else if prev.ends_with('{') {
                        row -= 1;
                        line_src = prev.trim().to_owned();
                        opener = true;
                    }
                }
                let mut pairs: Vec<(tree_sitter::Node<'_>, String)> = Vec::new();
                if names.len() >= 2 {
                    let field = if row != original_line {
                        0
                    } else {
                        [",", "{", "(", ";"]
                            .iter()
                            .filter_map(|s| f.text[..cmt.start_byte()].rfind(s))
                            .max()
                            .unwrap_or(0)
                    };
                    let mut rowl: Vec<_> = lits
                        .iter()
                        .copied()
                        .filter(|n| {
                            n.start_position().row == row
                                && n.start_byte() > field
                                && n.start_byte() < cmt.start_byte()
                        })
                        .collect();
                    rowl.sort_by_key(|n| n.start_byte());
                    if rowl.len() == 1 && names.iter().all(|n| catalog.contains_key(n)) {
                        let total = names.iter().fold(0u64, |v, n| v | catalog[n]);
                        if !values(rowl[0], &f.text).contains(&total) {
                            for (n, name) in rowl.into_iter().zip(names) {
                                pairs.push((n, name));
                            }
                        }
                    } else if rowl.len() != 1 {
                        for (n, name) in rowl.into_iter().zip(names) {
                            pairs.push((n, name));
                        }
                    }
                } else {
                    let name = &names[0];
                    let before: Vec<_> = lits
                        .iter()
                        .copied()
                        .filter(|n| {
                            n.end_byte() <= cmt.start_byte()
                                && n.start_position().row == row
                                && (!opener || !values(*n, &f.text).contains(&0))
                        })
                        .collect();
                    let after: Vec<_> = lits
                        .iter()
                        .copied()
                        .filter(|n| {
                            n.start_byte() >= cmt.end_byte() && n.start_position().row == row
                        })
                        .collect();
                    let side = if !before.is_empty() { &before } else { &after };
                    if let Some(want) = catalog.get(name) {
                        if side.iter().any(|n| values(*n, &f.text).contains(want)) {
                            continue;
                        }
                    }
                    if let Some(n) = before.iter().max_by_key(|n| n.end_byte()) {
                        pairs.push((*n, name.clone()));
                    } else if let Some(n) = after.iter().min_by_key(|n| n.start_byte()) {
                        pairs.push((*n, name.clone()));
                    }
                }
                for (lit, name) in pairs {
                    let Some(&want) = catalog.get(&name) else {
                        continue;
                    };
                    if name.starts_with("COLOR_") {
                        continue;
                    }
                    let vals = values(lit, &f.text);
                    if vals.contains(&want) {
                        continue;
                    }
                    let value = literal(lit, &f.text);
                    if want != 0
                        && want.count_ones() == 1
                        && value.is_some_and(|v| v & want == want)
                    {
                        continue;
                    }
                    if value.is_some_and(|v| v > 0xffff && want <= 0xff) {
                        continue;
                    }
                    let ll = lines.get(lit.start_position().row).copied().unwrap_or("");
                    if suppressed(ll) || suppressed(&line_src) {
                        continue;
                    }
                    let v = value.unwrap_or(0);
                    let line = lit.start_position().row + 1;
                    out.push(format!("{}:{line}: literal {v:#x} tagged /* {name} */ but {name} = {want:#x} ({want})",f.path.display()));
                }
            }
        }
        Ok(out)
    }
}
fn collect<'a>(
    n: tree_sitter::Node<'a>,
    l: &mut Vec<tree_sitter::Node<'a>>,
    c: &mut Vec<tree_sitter::Node<'a>>,
) {
    if n.kind() == "integer_literal" {
        l.push(n);
    } else if n.kind() == "line_comment" || n.kind() == "block_comment" {
        c.push(n);
    }
    let mut cur = n.walk();
    for ch in n.children(&mut cur) {
        collect(ch, l, c);
    }
}
fn lone_lit() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        crate::regexes::cached(r"^-?(?:0x[0-9a-fA-F_]+|\d[\d_]*)\w*\s*[,;]?$").unwrap()
    })
}
fn comment_names(s: &str) -> Vec<String> {
    let body = s
        .trim()
        .strip_prefix("/*")
        .and_then(|x| x.strip_suffix("*/"))
        .or_else(|| s.trim().strip_prefix("//"))
        .unwrap_or(s)
        .trim();
    let mut out = Vec::new();
    for p in crate::regexes::cached(r"\|\|?").unwrap().split(body) {
        let p = p.trim();
        let n = p.split_once('(').map(|x| x.0.trim()).unwrap_or(p);
        if n.is_empty()
            || !n
                .chars()
                .next()
                .is_some_and(|c| c == '_' || c.is_ascii_alphabetic())
            || !n.chars().all(|c| c == '_' || c.is_ascii_alphanumeric())
        {
            return vec![];
        }
        if p.contains('(') && !p.ends_with(')') {
            return vec![];
        }
        out.push(n.to_owned());
    }
    out
}
fn litval(s: &str) -> Option<u64> {
    let x = s.trim().replace('_', "");
    let x = x.trim_end_matches(|c: char| c.is_ascii_alphabetic());
    if let Some(h) = x.strip_prefix("0x").or_else(|| x.strip_prefix("0X")) {
        u64::from_str_radix(h, 16).ok()
    } else {
        x.parse().ok()
    }
}
fn literal(n: tree_sitter::Node<'_>, s: &str) -> Option<u64> {
    litval(&s[n.start_byte()..n.end_byte()])
}
fn values(n: tree_sitter::Node<'_>, s: &str) -> BTreeSet<u64> {
    let Some(v) = literal(n, s) else {
        return BTreeSet::new();
    };
    let mut x = BTreeSet::from([v]);
    if n.parent().is_some_and(|p| {
        p.kind() == "unary_expression"
            && s[p.start_byte()..p.end_byte()]
                .trim_start()
                .starts_with('-')
    }) {
        x.insert(v.wrapping_neg() & 0xffff_ffff);
        x.insert(v.wrapping_neg());
    }
    x
}
fn suppressed(s: &str) -> bool {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| crate::regexes::cached(r"(?i)\bnolint\b\s*[:(\[]?\s*([a-z0-9_-]*)").unwrap())
        .captures(s)
        .is_some_and(|m| {
            matches!(
                m.get(1).unwrap().as_str().to_ascii_lowercase().as_str(),
                "" | "inline_literal"
                    | "inline-literal"
                    | "inline_flag_literal"
                    | "inline-flag-literal"
                    | "scan_inline_flag_literals"
                    | "all"
            )
        })
}
fn catalog(c: &Corpus) -> BTreeMap<String, u64> {
    let mut out = BTreeMap::from([
        ("NonPagedPoolNx".into(), 0),
        ("NonPagedPoolNxCacheAligned".into(), 4),
        ("NonPagedPoolSessionNx".into(), 544),
        ("METHOD_BUFFERED".into(), 0),
        ("METHOD_IN_DIRECT".into(), 1),
        ("METHOD_OUT_DIRECT".into(), 2),
        ("METHOD_NEITHER".into(), 3),
        ("FILE_ANY_ACCESS".into(), 0),
        ("FILE_READ_ACCESS".into(), 1),
        ("FILE_WRITE_ACCESS".into(), 2),
        ("POOL_NX_ALLOCATION".into(), 0),
        ("MdlMappingNoExecute".into(), 0),
    ]);
    let define =
        crate::regexes::cached(r"(?m)^#define\s+([A-Z_][A-Z0-9_]*)\s+(.+?)\s*(?://.*)?$").unwrap();
    let mut fixed_headers = Vec::new();
    for f in c.sources("inline-flag-fixed-headers") {
        fixed_headers.push(f.text.clone());
        for m in define.captures_iter(&f.text) {
            if let Some(v) = cvalue(m.get(2).unwrap().as_str()) {
                out.insert(m[1].to_owned(), v);
            }
        }
    }
    let mut all_headers = fixed_headers.clone();
    for f in c.sources("inline-flag-header-trees") {
        all_headers.push(f.text.clone());
        for m in define.captures_iter(&f.text) {
            if let Some(v) = cvalue(m.get(2).unwrap().as_str()) {
                out.entry(m[1].to_owned()).or_insert(v);
            }
        }
    }
    // The enum oracle intentionally keeps only names with one value across the
    // curated headers, matching verify_constants' --enums ambiguity rule.
    let mut enum_values: BTreeMap<String, BTreeSet<u64>> = BTreeMap::new();
    for f in c.sources("inline-flag-enum-headers") {
        harvest_enums(&f.text, &out, &mut enum_values);
    }
    for (name, values) in enum_values {
        if values.len() == 1 {
            out.entry(name).or_insert(*values.iter().next().unwrap());
        }
    }
    // Evaluate object aliases and function-like IOCTL wrappers from the captured
    // CTL_CODE header family. Unknown operands/macros stay unknown rather than guessed.
    let mut funcs: BTreeMap<String, (Vec<String>, String)> = BTreeMap::new();
    let mut defs = Vec::new();
    let fdef =
        crate::regexes::cached(r"(?m)^\s*#define\s+([A-Za-z_]\w*)\(([^)]*)\)\s+(.+?)\s*$").unwrap();
    let od = crate::regexes::cached(r"(?m)^\s*#define\s+([A-Z_][A-Z0-9_]*)\s+(.+?)\s*$").unwrap();
    for text in &all_headers {
        let joined = crate::regexes::cached(r"\\\n")
            .unwrap()
            .replace_all(text, " ")
            .into_owned();
        for m in fdef.captures_iter(&joined) {
            funcs.insert(
                m[1].to_owned(),
                (
                    m[2].split(',')
                        .map(|x| x.trim().to_owned())
                        .filter(|x| !x.is_empty())
                        .collect(),
                    m[3].split("//").next().unwrap_or("").trim().to_owned(),
                ),
            );
        }
        for m in od.captures_iter(&joined) {
            defs.push((
                m[1].to_owned(),
                m[2].split("//").next().unwrap_or("").trim().to_owned(),
            ));
        }
    }
    funcs.entry("CTL_CODE".into()).or_insert((
        vec![
            "DeviceType".into(),
            "Function".into(),
            "Method".into(),
            "Access".into(),
        ],
        "(((DeviceType) << 16) | ((Access) << 14) | ((Function) << 2) | (Method))".into(),
    ));
    for _ in 0..8 {
        let mut changed = false;
        for (name, expr) in &defs {
            if !out.contains_key(name) {
                if let Some(v) = eval_macro(expr, &funcs, &out, 0) {
                    out.insert(name.clone(), v);
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    let mut ctl: BTreeMap<String, BTreeSet<u64>> = BTreeMap::new();
    for (name, expr) in defs {
        if name.starts_with("IOCTL_") || name.starts_with("FSCTL_") {
            if let Some(v) = eval_macro(&expr, &funcs, &out, 0) {
                ctl.entry(name).or_default().insert(v);
            }
        }
    }
    for (name, vals) in ctl {
        if vals.len() == 1 {
            out.entry(name).or_insert(*vals.iter().next().unwrap());
        }
    }
    out
}
fn eval_macro(
    expr: &str,
    funcs: &BTreeMap<String, (Vec<String>, String)>,
    scope: &BTreeMap<String, u64>,
    depth: u8,
) -> Option<u64> {
    if depth > 12 {
        return None;
    }
    let call = crate::regexes::cached(r"^\s*([A-Za-z_]\w*)\s*\((.*)\)\s*$").unwrap();
    if let Some(m) = call.captures(expr) {
        let name = &m[1];
        let inside = &m[2];
        let mut args = Vec::new();
        let (mut d, mut last) = (0i32, 0usize);
        for (i, ch) in inside.char_indices() {
            match ch {
                '(' => d += 1,
                ')' => d -= 1,
                ',' if d == 0 => {
                    args.push(inside[last..i].trim().to_owned());
                    last = i + 1;
                }
                _ => {}
            }
        }
        if !inside[last..].trim().is_empty() {
            args.push(inside[last..].trim().to_owned());
        }
        let (params, body) = funcs.get(name)?;
        if params.len() != args.len() {
            return None;
        }
        let mut expanded = body.clone();
        for (p, a) in params.iter().zip(args) {
            let value = eval_macro(&a, funcs, scope, depth + 1).or_else(|| eval_expr(&a, scope))?;
            expanded = crate::regexes::cached(&format!(r"\b{}\b", regex::escape(p)))
                .ok()?
                .replace_all(&expanded, value.to_string().as_str())
                .into_owned();
        }
        return eval_macro(&expanded, funcs, scope, depth + 1)
            .or_else(|| eval_expr(&expanded, scope));
    }
    eval_expr(expr, scope)
}
fn harvest_enums(
    text: &str,
    scope: &BTreeMap<String, u64>,
    out: &mut BTreeMap<String, BTreeSet<u64>>,
) {
    let no_comments = crate::regexes::cached(r"(?s)/\*.*?\*/|//[^\n]*")
        .unwrap()
        .replace_all(text, " ")
        .into_owned();
    let clean = no_comments
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let en = crate::regexes::cached(r"\benum\b[^{};]*\{").unwrap();
    for m in en.find_iter(&clean) {
        let start = m.end();
        let Some(end) = clean[start..].find('}').map(|i| i + start) else {
            continue;
        };
        let mut val = Some(0u64);
        let mut local = BTreeMap::new();
        for item in clean[start..end].split(',') {
            let item = item.trim();
            if item.is_empty() {
                continue;
            }
            let item = item
                .lines()
                .filter(|l| !l.trim_start().starts_with('#'))
                .collect::<Vec<_>>()
                .join(" ");
            let (name, next) = if let Some((n, e)) = item.split_once('=') {
                (n.trim(), eval_expr(e.trim(), &local))
            } else {
                (item.trim(), val)
            };
            if name
                .chars()
                .next()
                .is_some_and(|c| c == '_' || c.is_ascii_alphabetic())
                && name.chars().all(|c| c == '_' || c.is_ascii_alphanumeric())
            {
                if let Some(next) = next {
                    let next = next & 0xffff_ffff;
                    out.entry(name.to_owned()).or_default().insert(next);
                    local.insert(name.to_owned(), next);
                    val = Some(next.wrapping_add(1) & 0xffff_ffff);
                } else {
                    val = None;
                }
            }
        }
    }
}
fn eval_expr(expr: &str, scope: &BTreeMap<String, u64>) -> Option<u64> {
    let toks =
        crate::regexes::cached(r"0[xX][0-9a-fA-F]+|\d+|[A-Za-z_]\w*|<<|>>|[|&+\-()~^]").unwrap();
    let mut ts = Vec::new();
    let mut last = 0;
    for m in toks.find_iter(expr) {
        if !expr[last..m.start()].trim().is_empty() {
            return None;
        }
        ts.push(m.as_str().to_owned());
        last = m.end();
    }
    if !expr[last..].trim().is_empty() {
        return None;
    }
    let (mut i,) = (0usize,);
    let v = parse_expr(&ts, &mut i, 0, scope)?;
    if i == ts.len() {
        Some(v & 0xffff_ffff)
    } else {
        None
    }
}
fn parse_expr(ts: &[String], i: &mut usize, min: u8, scope: &BTreeMap<String, u64>) -> Option<u64> {
    let t = ts.get(*i)?.as_str();
    *i += 1;
    let mut lhs = match t {
        "(" => {
            let x = parse_expr(ts, i, 0, scope)?;
            if ts.get(*i)?.as_str() != ")" {
                return None;
            }
            *i += 1;
            x
        }
        "~" => !parse_expr(ts, i, 12, scope)?,
        "-" => parse_expr(ts, i, 12, scope)?.wrapping_neg(),
        _ => number(t).or_else(|| scope.get(t).copied())?,
    };
    loop {
        let Some(op) = ts.get(*i).map(String::as_str) else {
            break;
        };
        let (prec, kind) = match op {
            "|" => (1, 0),
            "^" => (2, 1),
            "&" => (3, 2),
            "<<" => (4, 3),
            ">>" => (4, 4),
            "+" => (5, 5),
            "-" => (5, 6),
            _ => (0, 0),
        };
        if prec < min || prec == 0 {
            break;
        }
        *i += 1;
        let rhs = parse_expr(ts, i, prec + 1, scope)?;
        lhs = match kind {
            0 => lhs | rhs,
            1 => lhs ^ rhs,
            2 => lhs & rhs,
            3 => lhs.wrapping_shl(rhs as u32),
            4 => lhs.wrapping_shr(rhs as u32),
            5 => lhs.wrapping_add(rhs),
            _ => lhs.wrapping_sub(rhs),
        };
    }
    Some(lhs)
}
fn number(s: &str) -> Option<u64> {
    if let Some(x) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(x, 16).ok()
    } else if s.starts_with('0') && s.len() > 1 && s.chars().all(|c| c.is_ascii_digit()) {
        u64::from_str_radix(&s[1..], 8).ok()
    } else {
        s.parse().ok()
    }
}
fn cvalue(s: &str) -> Option<u64> {
    let mut x = s
        .split("/*")
        .next()?
        .trim()
        .trim_end_matches(';')
        .trim()
        .trim_end_matches(['U', 'L', 'u', 'l']);
    while x.starts_with('(') && x.ends_with(')') {
        x = x[1..x.len() - 1].trim();
    }
    static CAST: OnceLock<Regex> = OnceLock::new();
    let casted = CAST
        .get_or_init(|| crate::regexes::cached(r"^\(\s*[A-Za-z_][A-Za-z0-9_ ]*\)\s*").unwrap())
        .replace(x, "");
    x = casted.trim();
    if let Some(rest) = x.strip_prefix('-') {
        return cvalue(rest).map(|v| v.wrapping_neg() & 0xffff_ffff);
    }
    if x.starts_with('\'') && x.ends_with('\'') {
        let chars: Vec<_> = x[1..x.len() - 1].chars().collect();
        if !chars.is_empty() && chars.len() <= 4 {
            return Some(chars.into_iter().fold(0u64, |v, c| (v << 8) | c as u64) & 0xffff_ffff);
        }
        return None;
    }
    if let Some(v) = x.strip_prefix("0x").or_else(|| x.strip_prefix("0X")) {
        u64::from_str_radix(v, 16).ok()
    } else if x.starts_with('0') && x.len() > 1 && x.chars().all(|c| c.is_ascii_digit()) {
        u64::from_str_radix(&x[1..], 8).ok()
    } else {
        x.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_annotation_fixtures() {
        let file = crate::SourceFile::new(
            "sdk/lib/rust/a.rs".into(),
            "fn f(){ let x = 0x1 /* METHOD_IN_DIRECT */; }\n".into(),
        );
        let good = Corpus::new(".".into(), vec![file]);
        assert!(InlineFlagLiterals.run(&good).unwrap().is_empty());
        let file = crate::SourceFile::new(
            "sdk/lib/rust/a.rs".into(),
            "fn f(){ let x = 0x4 /* METHOD_BUFFERED */; }\n".into(),
        );
        let bad = Corpus::new(".".into(), vec![file]);
        assert_eq!(InlineFlagLiterals.run(&bad).unwrap(),vec!["sdk/lib/rust/a.rs:1: literal 0x4 tagged /* METHOD_BUFFERED */ but METHOD_BUFFERED = 0x0 (0)"]);
        assert!(comment_names("/* FLAG */") == vec!["FLAG"]);
        assert!(comment_names("// FLAG (reason)") == vec!["FLAG"]);
        assert!(comment_names("// A || B") == vec!["A", "B"]);
        assert!(comment_names("// NULL / no '=' => -1").is_empty());
        assert!(suppressed("// nolint: inline-literal (verified)"));
        assert!(!suppressed("// nolint: other-lint"));
    }
    #[test]
    fn scanner_controls_comment_pairs_within_struct_field_only() {
        let header = crate::SourceFile::new(
            "sdk/include/xdk/fixture.h".into(),
            "#define VK_MEDIA_PREV_TRACK 0xB1\n#define VK_HOME 0x24\n".into(),
        );
        let source=crate::SourceFile::new("sdk/lib/rust/t.rs".into(),"static T: [VSC_VK; 2] = [\n    VSC_VK { Vsc: 0x10 as BYTE, Vk: 433 as USHORT /* VK_MEDIA_PREV_TRACK | KBDEXT */ },\n    VSC_VK { Vsc: 0x47 as BYTE, Vk: 3108 as USHORT /* VK_HOME | KNUMS */ },\n];\nconst BAD: u32 = 0x25; // VK_HOME\n".into());
        let corpus = Corpus::new(".".into(), vec![header, source]);
        assert_eq!(
            InlineFlagLiterals.run(&corpus).unwrap(),
            vec!["sdk/lib/rust/t.rs:5: literal 0x25 tagged /* VK_HOME */ but VK_HOME = 0x24 (36)"]
        );
    }
}
