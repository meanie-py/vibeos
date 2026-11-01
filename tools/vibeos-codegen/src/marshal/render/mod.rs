//! Pure WoW64 stage-two renderers; the parent module owns the model and classifier.
use super::{
    classify_param, diff_struct, is_divergent_scalar, is_safe_scalar, is_string_pointee, norm_type,
    resolve_exact, resolve_pointee, scalar_base, Param, Proto, Record,
};
use regex::Regex;
use std::collections::{HashMap, HashSet};
type Db = HashMap<String, Record>;
/// Insertion-ordered mapping (cycle guard and output/header declaration order).
#[derive(Default, Clone, Debug)]
pub struct Emitted(pub Vec<(String, (usize, usize))>);
impl Emitted {
    fn contains(&self, tag: &str) -> bool {
        self.0.iter().any(|(t, _)| t == tag)
    }
    fn add(&mut self, tag: String, sizes: (usize, usize)) {
        self.0.push((tag, sizes))
    }
    fn remove(&mut self, tag: &str) {
        self.0.retain(|(t, _)| t != tag)
    }
}
fn scalar_size(t: &str) -> Option<usize> {
    match t {
        "CHAR" | "UCHAR" | "BOOLEAN" | "BYTE" | "char" | "unsigned char" | "signed char" => Some(1),
        "WORD" | "USHORT" | "SHORT" | "WCHAR" | "wchar_t" | "ATOM" | "LANGID" | "short"
        | "unsigned short" => Some(2),
        "INT" | "UINT" | "LONG" | "ULONG" | "DWORD" | "BOOL" | "FLOAT" | "NTSTATUS"
        | "ACCESS_MASK" | "COLORREF" | "LCID" | "HRESULT" | "int" | "unsigned" | "unsigned int"
        | "long" | "unsigned long" | "float" => Some(4),
        "LONGLONG" | "ULONGLONG" | "DWORD64" | "UINT64" | "INT64" | "LARGE_INTEGER"
        | "ULARGE_INTEGER" | "double" | "__int64" | "long long" | "unsigned long long" => Some(8),
        _ => None,
    }
}

pub fn param_slots32(p: &Param, db: &Db) -> usize {
    let t = norm_type(&p.ctype);
    if t.contains('*') {
        return 1;
    }
    if let Some(r) = resolve_exact(&t, db) {
        return ((r.size + 3) / 4).max(1);
    }
    if scalar_size(&t) == Some(8) {
        2
    } else {
        1
    }
}
pub fn element_size(ctype: &str, db: &Db, width: usize) -> Option<usize> {
    let t = norm_type(ctype);
    let re = Regex::new(r"\[(\d+)\]$").unwrap();
    if let Some(c) = re.captures(&t) {
        let n: usize = c[1].parse().ok()?;
        return element_size(t[..c.get(0)?.start()].trim(), db, width)?.checked_mul(n);
    }
    if t.contains('*') {
        return Some(width);
    }
    if let Some(r) = resolve_exact(&t, db) {
        return Some(r.size);
    }
    if let Some(s) = scalar_size(&t) {
        return Some(s);
    }
    if is_divergent_scalar(&t) {
        return Some(width);
    }
    let sb = scalar_base(&t);
    if sb != t
        && (scalar_size(sb).is_some()
            || is_string_pointee(sb)
            || is_safe_scalar(sb)
            || is_divergent_scalar(sb)
            || resolve_exact(sb, db).is_some())
    {
        return Some(width);
    }
    None
}
pub fn flat_arg_exprs(proto: &Proto, db: &Db) -> Option<(Vec<String>, usize)> {
    let (mut out, mut slot) = (Vec::new(), 0);
    for p in &proto.params {
        let n = param_slots32(p, db);
        if n == 1 {
            out.push(format!("(ULONGLONG)(LONGLONG)(LONG)a[{slot}]"));
        } else if n == 2 {
            out.push(format!(
                "((ULONGLONG)a[{slot}] | ((ULONGLONG)a[{}] << 32))",
                slot + 1
            ));
        } else if n <= 4 {
            out.push(format!(
                "((ULONGLONG)a[{slot}] | ((ULONGLONG)a[{}] << 32))",
                slot + 1
            ));
            out.push(if n == 4 {
                format!(
                    "((ULONGLONG)a[{}] | ((ULONGLONG)a[{}] << 32))",
                    slot + 2,
                    slot + 3
                )
            } else {
                format!("(ULONGLONG)a[{}]", slot + 2)
            });
        } else {
            return None;
        }
        slot += n;
    }
    Some((out, slot))
}
pub fn is_deref_pointer(ctype: &str, db: &Db) -> bool {
    let t = norm_type(ctype);
    let re = Regex::new(r"\[\d*\]$").unwrap();
    let t = re.replace(&t, "").trim().to_owned();
    if is_divergent_scalar(&t) {
        return false;
    }
    if t.contains('*') {
        return true;
    }
    let sb = scalar_base(&t);
    sb != t
        && (is_safe_scalar(sb)
            || is_string_pointee(sb)
            || is_divergent_scalar(sb)
            || resolve_exact(sb, db).is_some())
}
fn ctag(r: &Record) -> &str {
    r.name
        .split_once(char::is_whitespace)
        .map(|(_, v)| v)
        .unwrap_or("")
}
fn span(fields: &[&super::Field], i: usize, total: usize) -> (usize, usize) {
    let off = fields[i].offset;
    let next = fields.get(i + 1).map_or(total, |f| f.offset);
    (off, next - off)
}
/// Emit field-copy marshallers, reserving tags before recursing to break cycles.
pub fn gen_struct_marshaller(
    r32: &Record,
    r64: &Record,
    d32: &Db,
    d64: &Db,
    emitted: &mut Emitted,
    out: &mut Vec<String>,
) -> bool {
    pair(r32, r64, d32, d64, emitted, out, false)
}
/// Emit callback down/up pair. Pointer fields are preserved on the up leg.
pub fn gen_cb_pair(
    r32: &Record,
    r64: &Record,
    d32: &Db,
    d64: &Db,
    emitted: &mut Emitted,
    out: &mut Vec<String>,
) -> bool {
    pair(r32, r64, d32, d64, emitted, out, true)
}
fn pair(
    r32: &Record,
    r64: &Record,
    d32: &Db,
    d64: &Db,
    emitted: &mut Emitted,
    out: &mut Vec<String>,
    callback: bool,
) -> bool {
    let tag = ctag(r32);
    if emitted.contains(tag) {
        return true;
    }
    emitted.add(tag.into(), (r32.size, r64.size));
    let f32 = r32.top_fields();
    let f64 = r64.top_fields();
    let (mut first, mut second) = (Vec::new(), Vec::new());
    for (i, (a, b)) in f32.iter().zip(f64.iter()).enumerate() {
        let (o32, mut s32) = span(&f32, i, r32.size);
        let (o64, mut s64) = span(&f64, i, r64.size);
        if let (Some(e32), Some(e64)) = (
            element_size(&a.ctype, d32, 4),
            element_size(&b.ctype, d64, 8),
        ) {
            s32 = e32;
            s64 = e64;
        }
        let cmt = format!(" /* {} */", a.name);
        if s32 == s64 {
            if !callback
                && i == 0
                && a.offset == 0
                && s32 == 4
                && ["cbSize", "lStructSize", "dwSize", "Size", "Length"].contains(&a.name.as_str())
            {
                first.push(format!(
                    "    WowAmSizeIn(d + {o64}, s + {o32}, {}, {});{cmt}",
                    r32.size, r64.size
                ));
                second.push(format!(
                    "    WowAmSizeOut(d + {o32}, s + {o64}, {}, {});{cmt}",
                    r32.size, r64.size
                ));
            } else if callback {
                first.push(format!("    WowCbCpy(d + {o32}, s + {o64}, {s32});{cmt}"));
                second.push(format!("    WowCbCpy(d + {o64}, s + {o32}, {s32});{cmt}"));
            } else {
                first.push(format!("    WowAmCpy(d + {o64}, s + {o32}, {s32});{cmt}"));
                second.push(format!("    WowAmCpy(d + {o32}, s + {o64}, {s32});{cmt}"));
            }
            continue;
        }
        if (s32, s64) == (4, 8) {
            if callback {
                first.push(format!("    WowCbNx(d + {o32}, s + {o64});{cmt}"));
                if is_deref_pointer(&a.ctype, d32) {
                    second.push(format!(
                        "    /* {}: kernel pointer preserved (guest value is scratch) */",
                        a.name
                    ));
                } else {
                    second.push(format!("    WowCbSx(d + {o64}, s + {o32});{cmt}"));
                }
            } else {
                if let Some(p) = resolve_pointee(&a.ctype, d32) {
                    if ["WIDENING", "SEMANTIC"]
                        .contains(&diff_struct(&p.name, d32, d64).cls.as_str())
                    {
                        emitted.remove(tag);
                        return false;
                    }
                }
                first.push(format!("    WowAmSx(d + {o64}, s + {o32});{cmt}"));
                second.push(format!("    WowAmNx(d + {o32}, s + {o64});{cmt}"));
            }
            continue;
        }
        let nr32 = resolve_exact(&a.ctype, d32);
        let nr64 = resolve_exact(if callback { &b.ctype } else { &a.ctype }, d64);
        if let (Some(n32), Some(n64)) = (nr32, nr64) {
            if callback || (n32.size, n64.size) == (s32, s64) {
                if !pair(&n32, &n64, d32, d64, emitted, out, callback) {
                    emitted.remove(tag);
                    return false;
                }
                let nt = ctag(&n32);
                if callback {
                    first.push(format!("    WowCbDown_{nt}(s + {o64}, d + {o32});{cmt}"));
                    second.push(format!("    WowCbUp_{nt}(s + {o32}, d + {o64});{cmt}"));
                } else {
                    first.push(format!("    WowAmIn_{nt}(s + {o32}, d + {o64});{cmt}"));
                    second.push(format!("    WowAmOut_{nt}(s + {o64}, d + {o32});{cmt}"));
                }
                continue;
            }
        }
        emitted.remove(tag);
        return false;
    }
    if callback {
        out.push(format!("/* {}: 32-bit {}b <-> 64-bit {}b */\nvoid WowCbDown_{tag}(const unsigned char *s, unsigned char *d)\n{{\n    WowCbZero(d, {});\n{}\n}}\n\nvoid WowCbUp_{tag}(const unsigned char *s, unsigned char *d)\n{{\n{}\n}}\n\n",r32.name,r32.size,r64.size,r32.size,first.join("\n"),second.join("\n")));
    } else {
        out.push(format!("/* {}: {} -> {} bytes */\nstatic void WowAmIn_{tag}(const unsigned char *s, unsigned char *d)\n{{\n    WowAmZero(d, {});\n{}\n}}\n\nstatic void WowAmOut_{tag}(const unsigned char *s, unsigned char *d)\n{{\n{}\n}}\n",r32.name,r32.size,r64.size,r64.size,first.join("\n"),second.join("\n")));
    }
    true
}
#[derive(Debug, serde::Serialize)]
pub struct Generated {
    pub c: String,
    pub thunks: usize,
    pub struct_pairs: usize,
    pub skipped: Vec<String>,
}
/// `manual` preserves the service-table order; export_spec is used in diagnostics.
pub fn generate(
    d32: &Db,
    d64: &Db,
    protos: &HashMap<String, Proto>,
    manual: &[String],
    linkable: &HashSet<String>,
    hand_written: &HashSet<String>,
    export_spec: &str,
) -> Generated {
    let mut emitted = Emitted::default();
    let (mut structs, mut thunks, mut dispatch, mut skipped) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for name in manual {
        if hand_written.contains(name) {
            continue;
        }
        let Some(p) = protos.get(name) else {
            skipped.push(format!("{name}: no prototype"));
            continue;
        };
        if !linkable.contains(name) {
            skipped.push(format!("{name}: not exported by {export_spec}"));
            continue;
        }
        let mut plan = Vec::new();
        let mut ok = true;
        for prm in &p.params {
            let (cls, _) = classify_param(prm, d32, d64);
            if cls == "FLAT" || cls == "IDENTICAL" {
                plan.push(None);
            } else if cls == "WIDENING" {
                let r32 =
                    resolve_pointee(&prm.ctype, d32).or_else(|| resolve_exact(&prm.ctype, d32));
                let r64 =
                    resolve_pointee(&prm.ctype, d64).or_else(|| resolve_exact(&prm.ctype, d64));
                if let (Some(r32), Some(r64)) = (r32, r64) {
                    if !gen_struct_marshaller(&r32, &r64, d32, d64, &mut emitted, &mut structs) {
                        skipped.push(format!(
                            "{name}: {} not generatable (deep pointee)",
                            ctag(&r32)
                        ));
                        ok = false;
                        break;
                    }
                    plan.push(Some((r32, r64)));
                } else {
                    skipped.push(format!("{name}: {} pointee unresolved", prm.name));
                    ok = false;
                    break;
                }
            } else {
                skipped.push(format!("{name}: param {} is {cls}", prm.name));
                ok = false;
                break;
            }
        }
        if !ok {
            continue;
        }
        let Some((mut exprs, _)) = flat_arg_exprs(p, d32) else {
            skipped.push(format!("{name}: by-value struct > 16 bytes"));
            continue;
        };
        let (mut decls, mut pre, mut post) = (Vec::new(), Vec::new(), Vec::new());
        let mut slot = 0;
        for (i, (prm, info)) in p.params.iter().zip(plan.iter()).enumerate() {
            let n = param_slots32(prm, d32);
            if let Some((r32, r64)) = info {
                let tag = ctag(r32);
                let is_const =
                    prm.ctype.contains("CONST") || prm.ctype.contains("const") || prm.sal == "in";
                decls.push(format!(
                    "    unsigned char b{i}[{}] __attribute__((aligned(16)));",
                    r64.size
                ));
                decls.push(format!(
                    "    const unsigned char *g{i} = (const unsigned char *)(ULONG_PTR)a[{slot}];"
                ));
                pre.push(format!("    if (g{i}) WowAmIn_{tag}(g{i}, b{i});"));
                if !is_const {
                    post.push(format!("    if (g{i}) WowAmOut_{tag}(b{i}, (unsigned char *)(ULONG_PTR)a[{slot}]);"));
                }
                exprs[i] = format!("(g{i} ? (ULONGLONG)(ULONG_PTR)b{i} : 0)");
            }
            slot += n;
        }
        let proto = if exprs.is_empty() {
            "void".into()
        } else {
            vec!["ULONGLONG"; exprs.len()].join(", ")
        };
        let mut thunk=format!("static NTSTATUS WowAmThunk_{name}(ULONG *a)\n{{\n    extern ULONGLONG {name}({proto});\n");
        if !decls.is_empty() {
            thunk.push_str(&(decls.join("\n") + "\n"));
        }
        thunk.push_str("    ULONGLONG r;\n    (void)a;\n");
        if !pre.is_empty() {
            thunk.push_str(&(pre.join("\n") + "\n"));
        }
        thunk.push_str(&format!("    r = {name}({});\n", exprs.join(", ")));
        if !post.is_empty() {
            thunk.push_str(&(post.join("\n") + "\n"));
        }
        thunk.push_str("    return (NTSTATUS)(ULONG)r;\n}\n");
        thunks.push(thunk);
        dispatch.push(format!(
            "    if (WowAmStrEq(Name, \"{name}\"))\n        return WowAmThunk_{name}(Arguments);"
        ));
    }
    let mut c = include_str!("_gen_prelude.txt").to_owned();
    c.push_str(&structs.concat());
    c.push('\n');
    c.push_str(&thunks.concat());
    c.push_str(&format!("\nNTSTATUS Wow64AutoMarshalDispatch(const char *Name, ULONG *Arguments)\n{{\n{}\n    return WOW64_AUTOMARSHAL_UNIMPL;\n}}\n",dispatch.join("\n")));
    if !skipped.is_empty() {
        let mut sorted = skipped.clone();
        sorted.sort();
        c.push_str(&format!(
            "\n/* NOT generated (still loud NEEDS_MANUAL stubs):\n{}\n */\n",
            sorted
                .iter()
                .map(|s| format!(" *   {s}"))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }
    Generated {
        c,
        thunks: thunks.len(),
        struct_pairs: emitted.0.len(),
        skipped,
    }
}
const CB_TABLE: &[(&str, u32, &str)] = &[
    ("WM_DRAWITEM", 0x002b, "DRAWITEMSTRUCT"),
    ("WM_MEASUREITEM", 0x002c, "MEASUREITEMSTRUCT"),
    ("WM_HELP", 0x0053, "HELPINFO"),
    ("WM_NEXTMENU", 0x0213, "MDINEXTMENU"),
    ("WM_WINDOWPOSCHANGING", 0x0046, "WINDOWPOS"),
    ("WM_WINDOWPOSCHANGED", 0x0047, "WINDOWPOS"),
];
const CB_EXTRA: &[&str] = &["NCCALCSIZE_PARAMS", "WINDOWPOS", "COPYDATASTRUCT"];
const CB_TYPE:&str="typedef struct _WOW64_CB_MARSHAL {\n    unsigned int Msg;\n    unsigned int Size32;\n    unsigned int Size64;\n    void (*Down)(const unsigned char *, unsigned char *);\n    void (*Up)(const unsigned char *, unsigned char *);\n} WOW64_CB_MARSHAL;\n\n";
#[derive(Debug, serde::Serialize)]
pub struct CallbackGenerated {
    pub c: String,
    pub header: String,
    pub table_messages: usize,
    pub struct_pairs: usize,
}
pub fn generate_callback(d32: &Db, d64: &Db) -> Result<CallbackGenerated, String> {
    let mut emitted = Emitted::default();
    let (mut pairs, mut rows, mut failed) = (Vec::new(), Vec::new(), Vec::new());
    for (name, value, st) in CB_TABLE {
        let (Some(r32), Some(r64)) = (resolve_exact(st, d32), resolve_exact(st, d64)) else {
            failed.push(format!("{st}: not in layout dump"));
            continue;
        };
        let v = diff_struct(&r32.name, d32, d64);
        if v.cls == "SEMANTIC" {
            failed.push(format!("{st}: SEMANTIC ({})", v.detail));
            continue;
        }
        if !gen_cb_pair(&r32, &r64, d32, d64, &mut emitted, &mut pairs) {
            failed.push(format!("{st}: pair not generatable"));
            continue;
        }
        let tag = ctag(&r32);
        rows.push(format!(
            "    {{ 0x{value:04X} /* {name} */, {}, {}, WowCbDown_{tag}, WowCbUp_{tag} }},",
            r32.size, r64.size
        ));
    }
    for st in CB_EXTRA {
        let (Some(r32), Some(r64)) = (resolve_exact(st, d32), resolve_exact(st, d64)) else {
            failed.push(format!("{st}: not in layout dump"));
            continue;
        };
        if !gen_cb_pair(&r32, &r64, d32, d64, &mut emitted, &mut pairs) {
            failed.push(format!("{st}: pair not generatable"));
        }
    }
    if !failed.is_empty() {
        return Err(format!(
            "gen-callback failed on:\n  {}",
            failed.join("\n  ")
        ));
    }
    let decls=emitted.0.iter().map(|(t,(s32,s64))|format!("void WowCbDown_{t}(const unsigned char *s, unsigned char *d);\nvoid WowCbUp_{t}(const unsigned char *s, unsigned char *d);\n#define WOW64_CB_SIZE32_{t} {s32}\n#define WOW64_CB_SIZE64_{t} {s64}")).collect::<Vec<_>>().join("\n");
    let header=format!("/* wow64_cb_automarshal.h — GENERATED by marshalgen. DO NOT EDIT. */\n#ifndef WOW64_CB_AUTOMARSHAL_H\n#define WOW64_CB_AUTOMARSHAL_H\n\n{CB_TYPE}const WOW64_CB_MARSHAL *Wow64CbFindMarshal(unsigned int Msg);\n\n{decls}\n\n#endif /* WOW64_CB_AUTOMARSHAL_H */\n");
    let c=format!("{}{}{CB_TYPE}static const WOW64_CB_MARSHAL Wow64CbMarshalTable[] = {{\n{}\n}};\n\nconst WOW64_CB_MARSHAL *Wow64CbFindMarshal(unsigned int Msg)\n{{\n    unsigned int i;\n    for (i = 0; i < sizeof(Wow64CbMarshalTable) /\n                    sizeof(Wow64CbMarshalTable[0]); i++)\n        if (Wow64CbMarshalTable[i].Msg == Msg)\n            return &Wow64CbMarshalTable[i];\n    return 0;\n}}\n",include_str!("_cb_prelude.txt"),pairs.concat(),rows.join("\n"));
    Ok(CallbackGenerated {
        c,
        header,
        table_messages: rows.len(),
        struct_pairs: emitted.0.len(),
    })
}
