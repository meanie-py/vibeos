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
    SourceSet::tree(
        "inline-flag-ctl-headers",
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
    SourceSet::tree(
        "inline-flag-extra-header-trees",
        &[
            "sdk/include/xdk",
            "sdk/include/ndk",
            "sdk/include/psdk",
            "ntoskrnl/include/internal",
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
            if !potential_annotation(&f.text, &catalog) {
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
                        let total = names.iter().fold(0i128, |v, n| v | catalog[n]);
                        if !values(rowl[0], &f.text).iter().any(|v| *v == total) {
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
                        if side
                            .iter()
                            .any(|n| values(*n, &f.text).iter().any(|v| *v == *want))
                        {
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
                    if vals.iter().any(|v| *v == want) {
                        continue;
                    }
                    let value = literal(lit, &f.text);
                    if want != 0
                        && want.count_ones() == 1
                        && value.is_some_and(|v| want > 0 && (v as i128) & want == want)
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
                    let want_hex = if want < 0 {
                        format!("-{:#x}", want.unsigned_abs())
                    } else {
                        format!("{want:#x}")
                    };
                    out.push(format!("{}:{line}: literal {v:#x} tagged /* {name} */ but {name} = {want_hex} ({want})",f.path.display()));
                }
            }
        }
        Ok(out)
    }
}
// Consider every opener, including openers inside strings or comments. False
// positives only request parsing; actual comment nodes still decide findings.
fn potential_annotation(text: &str, catalog: &BTreeMap<String, i128>) -> bool {
    for (start, _) in text.match_indices("//") {
        let end = text[start..].find('\n').map_or(text.len(), |n| start + n);
        if comment_names(&text[start..end])
            .iter()
            .any(|n| catalog.contains_key(n))
        {
            return true;
        }
    }
    for (start, _) in text.match_indices("/*") {
        if let Some(end) = text[start + 2..].find("*/") {
            if comment_names(&text[start..start + 2 + end + 2])
                .iter()
                .any(|n| catalog.contains_key(n))
            {
                return true;
            }
        }
    }
    false
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
    // Match Python's `_SUFFIX_RE` after it removes literal separators. Stripping
    // alphabetic characters alone leaves the digits in `u32`/`i64` behind, so
    // every typed Rust integer silently became an unparseable value (and then
    // the caller displayed it as zero).
    static SUFFIX: OnceLock<Regex> = OnceLock::new();
    let suffix = SUFFIX.get_or_init(|| {
        crate::regexes::cached(r"(?:u8|u16|u32|u64|usize|i8|i16|i32|i64|isize)$").unwrap()
    });
    let clean = s.trim().replace('_', "");
    let x = suffix.replace(&clean, "");
    let x = x.as_ref();
    if let Some(h) = x.strip_prefix("0x").or_else(|| x.strip_prefix("0X")) {
        u64::from_str_radix(h, 16).ok()
    } else {
        x.parse().ok()
    }
}
fn literal(n: tree_sitter::Node<'_>, s: &str) -> Option<u64> {
    litval(&s[n.start_byte()..n.end_byte()])
}
fn values(n: tree_sitter::Node<'_>, s: &str) -> BTreeSet<i128> {
    let Some(v) = literal(n, s) else {
        return BTreeSet::new();
    };
    let mut x = BTreeSet::from([v as i128]);
    if n.parent().is_some_and(|p| {
        p.kind() == "unary_expression"
            && s[p.start_byte()..p.end_byte()]
                .trim_start()
                .starts_with('-')
    }) {
        // The source scanner models Rust's negative literal at both C-width and host-width,
        // then compares negative C catalog values against those unsigned two's-complement
        // representations as well.
        x.insert((v.wrapping_neg() & 0xffff_ffff) as i128);
        x.insert(v.wrapping_neg() as i128);
        x.insert(-(v as i128));
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
fn catalog(c: &Corpus) -> BTreeMap<String, i128> {
    // Keep the seed, direct-header, enum, downgrade, and CTL_CODE phases in the
    // same order as verify_constants.build_c_catalog(include_enums=True).
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
    let fixed: Vec<_> = c.sources("inline-flag-fixed-headers").collect();
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
                if let Some(v) = c_header_value(m.get(2).unwrap().as_str()) {
                    out.insert(m[1].to_owned(), v);
                }
            }
        }
    }

    let mut enum_values: BTreeMap<String, BTreeSet<u64>> = BTreeMap::new();
    for f in c.sources("inline-flag-enum-headers") {
        harvest_enums(&f.text, &mut enum_values);
    }
    for (name, values) in enum_values {
        if values.len() == 1 {
            out.entry(name)
                .or_insert(*values.iter().next().unwrap() as i128);
        }
    }
    out.insert("POOL_NX_ALLOCATION".into(), 0);
    out.insert("MdlMappingNoExecute".into(), 0);

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
    let ctl_headers: Vec<_> = c.sources("inline-flag-ctl-headers").collect();
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

    // scan_inline_flag_literals.add_headers runs after build_c_catalog and
    // harvests numeric defines from these four trees. The curated build values
    // (including CTL_CODE results) win collisions via setdefault semantics.
    static STRIP: OnceLock<Regex> = OnceLock::new();
    static DEFINE: OnceLock<Regex> = OnceLock::new();
    let strip_comments = STRIP.get_or_init(|| crate::regexes::cached(r"(?s)/\*.*?\*/").unwrap());
    let define = DEFINE.get_or_init(|| {
        crate::regexes::cached(r"^#define\s+([A-Z_][A-Z0-9_]*)\s+(.+?)(?:\s*//.*)?$").unwrap()
    });
    let ordered = c.ordered_sources("inline-flag-extra-header-trees");
    for root in [
        "sdk/include/xdk",
        "sdk/include/ndk",
        "sdk/include/psdk",
        "ntoskrnl/include/internal",
    ] {
        for f in ordered
            .iter()
            .copied()
            .filter(|f| f.path.strip_prefix(root).is_ok())
        {
            let clean = strip_comments.replace_all(&f.text, " ");
            for raw in clean.lines() {
                if let Some(m) = define.captures(raw.trim()) {
                    if let Some(v) = inline_header_value(m.get(2).unwrap().as_str()) {
                        out.entry(m[1].to_owned()).or_insert(v);
                    }
                }
            }
        }
    }
    out
}

fn c_header_value(s: &str) -> Option<i128> {
    // parse_c_hex strips C block comments; parse_c_multichar and
    // parse_ntstatus_cast receive the unmodified RHS and therefore reject
    // literals followed by comments, exactly as verify_constants does.
    let raw = s.trim().trim_end_matches(';').trim();
    let mut x = raw;
    x = x.trim_end_matches(['U', 'L', 'u', 'l']).trim();
    if let Some(negative) = x.strip_prefix('-') {
        return c_header_value(negative).map(|v| -v);
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
fn harvest_enums(text: &str, out: &mut BTreeMap<String, BTreeSet<u64>>) {
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
                (n.trim(), eval_enum_expr(e.trim(), &local))
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
fn eval_enum_expr(expr: &str, scope: &BTreeMap<String, u64>) -> Option<u64> {
    let cleaned = expr.replace("<<", "L");
    if cleaned.contains('&')
        || cleaned.contains('^')
        || cleaned.contains('~')
        || cleaned.contains('>')
    {
        return None;
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

#[doc(hidden)]
pub fn export_catalog_for_audit(c: &Corpus) -> BTreeMap<String, i128> {
    catalog(c)
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
        assert_eq!(
            InlineFlagLiterals.run(&bad).unwrap(),
            vec![
                "sdk/lib/rust/a.rs:1: literal 0x4 tagged /* METHOD_BUFFERED */ but METHOD_BUFFERED = 0x0 (0)"
            ]
        );
        assert!(comment_names("/* FLAG */") == vec!["FLAG"]);
        assert!(comment_names("// FLAG (reason)") == vec!["FLAG"]);
        assert!(comment_names("// A || B") == vec!["A", "B"]);
        assert!(comment_names("// NULL / no '=' => -1").is_empty());
        assert!(suppressed("// nolint: inline-literal (verified)"));
        assert!(!suppressed("// nolint: other-lint"));
        // `integer_literal` includes these suffixes in its source text. Python
        // strips the complete suffix; do not leave its trailing width digits.
        assert_eq!(litval("0xC0000095_u32"), Some(0xC0000095));
        assert_eq!(litval("0x80000016u32"), Some(0x80000016));
        assert_eq!(litval("1u8"), Some(1));
        assert_eq!(litval("1073741670i32"), Some(1073741670));
        // Hexadecimal A-F digits are not type suffixes.
        assert_eq!(litval("0x0f"), Some(15));
        assert_eq!(litval("0x2C05C"), Some(0x2C05C));
        assert_eq!(litval("0xFFFF_FFFF"), Some(0xFFFF_FFFF));
        assert_eq!(litval("0x007F"), Some(0x7f));
        assert_eq!(litval("0xC000_007B"), Some(0xC000_007B));
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

    #[test]
    fn negative_catalog_values_match_rust_unary_minus_twos_complement() {
        let header = crate::SourceFile::new(
            "sdk/include/psdk/windef.h".into(),
            "#define HFILE_ERROR -1\n#define INVALID_HANDLE_VALUE -1\n".into(),
        );
        let rust = crate::SourceFile::new(
            "sdk/lib/rust/a.rs".into(),
            "fn a() { return -1; // HFILE_ERROR\n}\nfn b() { let h = (-1isize) as HANDLE; // INVALID_HANDLE_VALUE\n}\n".into(),
        );
        let corpus = Corpus::new(".".into(), vec![header, rust]);
        assert!(InlineFlagLiterals.run(&corpus).unwrap().is_empty());
        assert_eq!(c_header_value("-1"), Some(-1));
    }
    #[test]
    fn signed_catalog_mismatch_uses_python_hex_display() {
        let c = Corpus::new(
            ".".into(),
            vec![
                crate::SourceFile::new(
                    "sdk/include/xdk/fixture.h".into(),
                    "#define HFILE_ERROR -1\n".into(),
                ),
                crate::SourceFile::new(
                    "sdk/lib/rust/a.rs".into(),
                    "fn f(){ let x = 0; // HFILE_ERROR\n}\n".into(),
                ),
            ],
        );
        assert_eq!(InlineFlagLiterals.run(&c).unwrap(),vec!["sdk/lib/rust/a.rs:1: literal 0x0 tagged /* HFILE_ERROR */ but HFILE_ERROR = -0x1 (-1)"]);
    }
}
