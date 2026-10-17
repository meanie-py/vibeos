//! Exact no-enums verify_constants header catalog used by scan_mask_drift.
use crate::{Corpus, SourceSet};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

pub const SOURCES: &[SourceSet] = &[
    SourceSet::tree(
        "mask-fixed-headers",
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
    SourceSet::tree(
        "mask-define-headers",
        &[
            "sdk/include",
            "sdk/lib",
            "ntoskrnl/include",
            "drivers",
            "win32ss",
            "hal",
        ],
        &["h"],
        &[],
    ),
    SourceSet::tree(
        "mask-ctl-headers",
        &[
            "sdk/include/xdk/iotypes.h",
            "sdk/include/psdk/winioctl.h",
            "sdk/include/psdk/ntdddisk.h",
            "sdk/include/psdk/ntddcdrm.h",
            "sdk/include/psdk/ntddstor.h",
            "sdk/include/psdk/ntddscsi.h",
            "sdk/include/psdk/ntddvol.h",
            "sdk/include/reactos/drivers/ntddrdsk.h",
            "sdk/include/xdk/ntifs.template.h",
            "sdk/include/reactos/drivers/afd/shared.h",
            "sdk/include/psdk/ntddtdi.h",
            "sdk/include/psdk/ntddndis.h",
            "sdk/include/psdk/usbiodef.h",
        ],
        &["h"],
        &[],
    ),
];

pub fn catalog(c: &Corpus) -> BTreeMap<String, i128> {
    // Keep the seed, direct-header, downgrade, and CTL_CODE phases in the
    // same direct-header and CTL phases as verify_constants.build_c_catalog(include_enums=False).
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
        crate::regexes::cached(r"^#define\s+([A-Z_][A-Z0-9_]*)\s+(.+?)(?:\s*//.*)?$").unwrap();
    const FIXED_ORDER: &[&str] = &[
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
    ];
    let fixed: Vec<_> = c.sources("mask-fixed-headers").collect();
    for path in FIXED_ORDER {
        let Some(f) = fixed
            .iter()
            .copied()
            .find(|f| f.path == std::path::Path::new(path))
        else {
            continue;
        };
        for raw in f.text.lines() {
            let line = raw.trim();
            if let Some(m) = define.captures(line) {
                if let Some(v) = header_value(m.get(2).unwrap().as_str()) {
                    out.insert(m[1].to_owned(), v);
                }
            }
        }
    }

    // Only the explicit Python CTL_CODE_HEADERS participate here. The prior
    // broad header-tree scan both expanded the oracle and repeatedly parsed
    // thousands of irrelevant headers.
    let mut funcs: BTreeMap<String, (Vec<String>, String)> = BTreeMap::new();
    let mut defs = Vec::new();
    let fdef = crate::regexes::cached(r"^#define\s+([A-Za-z_]\w*)\(([^)]*)\)\s+(.+?)\s*$").unwrap();
    let od = crate::regexes::cached(r"^#define\s+([A-Z_][A-Z0-9_]*)\s+(.+?)\s*$").unwrap();
    let mut texts = Vec::new();
    const CTL_ORDER: &[&str] = &[
        "sdk/include/xdk/iotypes.h",
        "sdk/include/psdk/winioctl.h",
        "sdk/include/psdk/ntdddisk.h",
        "sdk/include/psdk/ntddcdrm.h",
        "sdk/include/psdk/ntddstor.h",
        "sdk/include/psdk/ntddscsi.h",
        "sdk/include/psdk/ntddvol.h",
        "sdk/include/reactos/drivers/ntddrdsk.h",
        "sdk/include/xdk/ntifs.template.h",
        "sdk/include/reactos/drivers/afd/shared.h",
        "sdk/include/psdk/ntddtdi.h",
        "sdk/include/psdk/ntddndis.h",
        "sdk/include/psdk/usbiodef.h",
    ];
    let ctl_headers: Vec<_> = c.sources("mask-ctl-headers").collect();
    for path in CTL_ORDER {
        let Some(f) = ctl_headers
            .iter()
            .copied()
            .find(|f| f.path == std::path::Path::new(path))
        else {
            continue;
        };
        let joined = crate::regexes::cached(r"\\\n")
            .unwrap()
            .replace_all(&f.text, " ")
            .into_owned();
        texts.push(joined);
    }
    for text in &texts {
        for raw in text.lines() {
            let line = raw.trim();
            if let Some(m) = fdef.captures(line) {
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
            if let Some(m) = od.captures(line) {
                defs.push((
                    m[1].to_owned(),
                    m[2].split("//").next().unwrap_or("").trim().to_owned(),
                ));
            }
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

    let mut scope: BTreeMap<String, u64> =
        out.iter().map(|(n, v)| (n.clone(), *v as u64)).collect();
    let alias =
        crate::regexes::cached(r"^#define\s+([A-Z_][A-Z0-9_]*)\s+([A-Z_][A-Z0-9_]*)\s*(?://.*)?$")
            .unwrap();
    let mut aliases = Vec::new();
    for (name, expr) in &defs {
        if let Some(v) = c_simple_value(expr) {
            scope.entry(name.clone()).or_insert(v as u64);
        } else if let Some(m) = alias.captures(&format!("#define {name} {expr}")) {
            aliases.push((m[1].to_owned(), m[2].to_owned()));
        }
    }
    for _ in 0..8 {
        let mut progressed = false;
        for (name, target) in &aliases {
            if !scope.contains_key(name) {
                if let Some(&v) = scope.get(target) {
                    scope.insert(name.clone(), v);
                    progressed = true;
                }
            }
        }
        if !progressed {
            break;
        }
    }
    for (name, value) in [
        ("METHOD_BUFFERED", 0),
        ("METHOD_IN_DIRECT", 1),
        ("METHOD_OUT_DIRECT", 2),
        ("METHOD_NEITHER", 3),
        ("FILE_ANY_ACCESS", 0),
        ("FILE_READ_ACCESS", 1),
        ("FILE_WRITE_ACCESS", 2),
    ] {
        scope.insert(name.into(), value);
    }

    let mut ctl: BTreeMap<String, BTreeSet<u64>> = BTreeMap::new();
    for (name, expr) in defs {
        if name.starts_with("IOCTL_") || name.starts_with("FSCTL_") {
            if let Some(v) = eval_macro(&expr, &funcs, &scope, 0) {
                ctl.entry(name).or_default().insert(v & 0xffff_ffff);
            }
        }
    }
    for (name, vals) in ctl {
        if vals.len() == 1 {
            out.entry(name)
                .or_insert(*vals.iter().next().unwrap() as i128);
        }
    }

    out
}

pub fn header_value(s: &str) -> Option<i128> {
    // parse_c_hex strips C block comments; parse_c_multichar and
    // parse_ntstatus_cast receive the unmodified RHS and therefore reject
    // literals followed by comments, exactly as verify_constants does.
    let raw = s.trim().trim_end_matches(';').trim();
    let mut x = raw;
    x = x.trim_end_matches(['U', 'L', 'u', 'l']).trim();
    if let Some(negative) = x.strip_prefix('-') {
        return header_value(negative).map(|v| -v);
    }
    if let Some(v) = c_simple_value(x) {
        return Some(v);
    }
    if raw.starts_with('\'') && raw.ends_with('\'') {
        let chars: Vec<_> = raw[1..raw.len() - 1].chars().collect();
        if (1..=4).contains(&chars.len()) {
            return Some(
                (chars.into_iter().fold(0u64, |v, c| (v << 8) | c as u64) & 0xffff_ffff) as i128,
            );
        }
    }
    if ![",", "|", "+", "<<", ">>"]
        .iter()
        .any(|op| raw.contains(op))
    {
        static HEX: OnceLock<Regex> = OnceLock::new();
        if let Some(m) = HEX
            .get_or_init(|| crate::regexes::cached(r"0x[0-9A-Fa-f]+").unwrap())
            .find(raw)
        {
            if let Ok(v) = u64::from_str_radix(&m.as_str()[2..], 16) {
                return Some(v as i128);
            }
        }
    }
    None
}

// Python parse_c_hex: C 0x values, leading-zero octal, then signed/unsigned
// decimal. Used separately from parse_c_multichar/parse_ntstatus_cast in the
// ordinary catalog phase and alone for CTL_CODE scope literals.
fn c_simple_value(s: &str) -> Option<i128> {
    let x = s.split("/*").next()?.trim().trim_end_matches(';').trim();
    let x = x.trim_end_matches(['U', 'L', 'u', 'l']).trim();
    if let Some(negative) = x.strip_prefix('-') {
        return c_simple_value(negative).map(|v| -v);
    }
    if let Some(v) = x.strip_prefix("0x").or_else(|| x.strip_prefix("0X")) {
        if let Ok(n) = i128::from_str_radix(v, 16) {
            return Some(n);
        }
    } else if x.starts_with('0') && x.len() > 1 && x[1..].chars().all(|c| c.is_ascii_digit()) {
        if let Ok(n) = i128::from_str_radix(&x[1..], 8) {
            return Some(n);
        }
    } else if x.chars().all(|c| c.is_ascii_digit()) {
        if let Ok(n) = x.parse::<i128>() {
            return Some(n);
        }
    }
    None
}

fn inline_header_value(value: &str) -> Option<i128> {
    // Exact shape of scan_inline_flag_literals.parse_c_value: parenthesis peel,
    // optional C type cast, Rust-like integer suffix peel, decimal/hex parse,
    // then verify_constants.parse_c_hex fallback (including octal).
    let mut x = value.trim();
    while x.starts_with('(') && x.ends_with(')') {
        x = x[1..x.len() - 1].trim();
    }
    static CAST: OnceLock<Regex> = OnceLock::new();
    let cast =
        CAST.get_or_init(|| crate::regexes::cached(r"^\(\s*[A-Za-z_][A-Za-z0-9_ ]*\)\s*").unwrap());
    let casted = cast.replace(x, "");
    x = casted.trim();
    static SUFFIX: OnceLock<Regex> = OnceLock::new();
    let suffix = SUFFIX.get_or_init(|| {
        crate::regexes::cached(r"(u8|u16|u32|u64|usize|i8|i16|i32|i64|isize)$").unwrap()
    });
    let unsuffixed = suffix.replace(x, "");
    x = unsuffixed.trim_end_matches(['u', 'U', 'l', 'L']);
    if let Some(h) = x.strip_prefix("0x").or_else(|| x.strip_prefix("0X")) {
        if let Ok(v) = i128::from_str_radix(h, 16) {
            return Some(v);
        }
    } else if let Ok(v) = x.parse::<i128>() {
        return Some(v);
    }
    c_simple_value(x)
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
