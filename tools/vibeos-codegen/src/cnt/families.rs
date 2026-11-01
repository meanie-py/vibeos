//! Bespoke shim families. Parameter and type validation happens before rendering.
use super::signature::rs_ty;
use toml::Value;

type Params = Vec<(String, String)>;
fn field<'a>(m: &'a Value, k: &str) -> Result<&'a str, String> {
    m.get(k)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing or invalid {k}"))
}
fn args<'a>(params: &'a Params, count: usize, name: &str) -> Result<Vec<&'a str>, String> {
    if params.len() != count {
        return Err(format!(
            "{name}: expected {count} parameters, got {}",
            params.len()
        ));
    }
    Ok(params.iter().map(|(_, n)| n.as_str()).collect())
}
fn finish(
    name: &str,
    m: &Value,
    params: &Params,
    rt: &str,
    body: Vec<String>,
) -> Result<String, String> {
    let decl = params
        .iter()
        .map(|(t, n)| Ok(format!("{n}: {}", rs_ty(t)?)))
        .collect::<Result<Vec<_>, String>>()?
        .join(", ");
    let sig = field(m, "c_signature")?;
    let misf = m
        .get("preserved_misfeature")
        .and_then(Value::as_str)
        .unwrap_or("");
    Ok(format!("/// `{sig}`\n/// preserved misfeature: {misf}\n#[no_mangle]\npub unsafe extern \"C\" fn {name}({decl}) -> {rt} {{\n{}\n}}\n", body.join("\n")))
}
pub fn render(name: &str, m: &Value, params: &Params, rt: &str) -> Result<Option<String>, String> {
    let sp = field(m, "slice_pattern")?;
    let b: Vec<String> = match sp {
        "strcpy_like" | "strcat_like" | "strncat_like" => {
            let p = args(params, if sp == "strncat_like" { 3 } else { 2 }, name)?;
            let wide = rs_ty(&params[0].0)? == "*mut u16";
            let el = if wide { "u16" } else { "u8" };
            let cstr = if wide { "wcstr_slice" } else { "cstr_slice" };
            let rawm = if wide {
                "wraw_mut_slice"
            } else {
                "raw_mut_slice"
            };
            let d = p[0];
            let s = p[1];
            match sp {
                "strcpy_like" => vec![
                    format!("    let s = {cstr}({s});"),
                    "    let m = s.len();".into(),
                    format!("    let d = {rawm}({d}, m + 1);"),
                    "    crate::core_rs::string::cpy(d, s);".into(),
                    format!("    {d}"),
                ],
                "strcat_like" => vec![
                    format!("    let dl = {cstr}({d} as *const {el}).len();"),
                    format!("    let a = {cstr}({s});"),
                    "    let m = a.len();".into(),
                    format!("    let tail = {rawm}({d}.add(dl), m + 1);"),
                    "    crate::core_rs::string::cat(tail, a);".into(),
                    format!("    {d}"),
                ],
                _ => {
                    let n = p[2];
                    let bounded = if wide {
                        "wcstr_slice_bounded"
                    } else {
                        "cstr_slice_bounded"
                    };
                    vec![
                        format!("    if {n} == 0 {{ return {d}; }}"),
                        format!("    let dl = {cstr}({d} as *const {el}).len();"),
                        format!("    let sc = {bounded}({s}, {n});"),
                        format!("    let m = if sc.len() < {n} {{ sc.len() }} else {{ {n} }};"),
                        format!("    let tail = {rawm}({d}.add(dl), m + 1);"),
                        format!("    crate::core_rs::string::ncat(tail, sc, {n});"),
                        format!("    {d}"),
                    ]
                }
            }
        }
        "mbtowc" | "mbstowcs" | "wctomb" | "wcstombs" | "toupper_mb" => {
            let p = args(
                params,
                match sp {
                    "wctomb" => 2,
                    "toupper_mb" => 1,
                    _ => 3,
                },
                name,
            )?;
            match sp {
                "mbtowc" => {
                    let (wc, mb, cnt) = (p[0], p[1], p[2]);
                    vec![
                        format!("    if {mb}.is_null() || {wc}.is_null() {{ return 0; }}"),
                        format!("    let take = if {cnt} < 2 {{ {cnt} }} else {{ 2 }};"),
                        format!("    let s = core::slice::from_raw_parts({mb}, take);"),
                        "    let r = crate::core_rs::mbcp::mbtowc(s);".into(),
                        "    if r.ret == -1 { return -1; }".into(),
                        format!("    *{wc} = r.wc;"),
                        "    r.ret".into(),
                    ]
                }
                "mbstowcs" | "wcstombs" => {
                    let (out, input, cnt) = (p[0], p[1], p[2]);
                    let cstr = if sp == "mbstowcs" {
                        "cstr_slice"
                    } else {
                        "wcstr_slice"
                    };
                    let core = sp;
                    vec![
                        format!("    let s = {cstr}({input});"),
                        format!("    if {out}.is_null() {{ return s.len() as {rt}; }}"),
                        format!("    let n = if s.len() < {cnt} {{ s.len() }} else {{ {cnt} }};"),
                        format!("    let d = core::slice::from_raw_parts_mut({out}, n);"),
                        format!("    crate::core_rs::mbcp::{core}(d, s, {cnt}) as {rt}"),
                    ]
                }
                "wctomb" => {
                    let (mb, wc) = (p[0], p[1]);
                    vec![
                        format!("    if {mb}.is_null() {{ return 0; }}"),
                        format!("    *{mb} = crate::core_rs::mbcp::wctomb({wc});"),
                        "    1".into(),
                    ]
                }
                _ => vec![format!("    crate::core_rs::mbcp::toupper_nt_mb({})", p[0])],
            }
        }
        "memccpy" => {
            let p = args(params, 4, name)?;
            let (to, frm, c, count) = (p[0], p[1], p[2], p[3]);
            vec![
                format!("    let d = raw_mut_slice({to} as *mut u8, {count});"),
                format!("    let s = raw_slice({frm} as *const u8, {count});"),
                format!("    match crate::core_rs::mem::ccpy(d, s, {c}) {{"),
                format!("        Some(i) => ({to} as *mut u8).add(i) as {rt},"),
                "        None => core::ptr::null_mut(),".into(),
                "    }".into(),
            ]
        }
        "rand_step" => {
            args(params, 0, name)?;
            vec![
                "    use core::sync::atomic::Ordering;".into(),
                "    let s = super::RAND_STATE.load(Ordering::Relaxed);".into(),
                "    let (ns, r) = crate::core_rs::rng::step(s);".into(),
                "    super::RAND_STATE.store(ns, Ordering::Relaxed);".into(),
                "    r".into(),
            ]
        }
        "rand_seed" => {
            let seed = args(params, 1, name)?[0];
            vec!["    use core::sync::atomic::Ordering;".into(),format!("    super::RAND_STATE.store(crate::core_rs::rng::seed({seed}), Ordering::Relaxed);")]
        }
        "tok_static" | "wtok_static" | "tok_ctx" | "wtok_ctx" => {
            let wide = sp.starts_with('w');
            let stat = sp.ends_with("static");
            let p = args(params, if stat { 2 } else { 3 }, name)?;
            let (s, delim) = (p[0], p[1]);
            let el = if wide { "u16" } else { "u8" };
            let cstr = if wide { "wcstr_slice" } else { "cstr_slice" };
            let null = "core::ptr::null_mut()";
            let mut b = Vec::new();
            if stat {
                b.extend([
                    "    use core::sync::atomic::{AtomicPtr, Ordering};".into(),
                    format!("    static SAVED: AtomicPtr<{el}> = AtomicPtr::new({null});"),
                    format!("    if {delim}.is_null() {{ return {null}; }}"),
                    format!("    let base = if {s}.is_null() {{"),
                    "        let p = SAVED.load(Ordering::Relaxed);".into(),
                    format!("        if p.is_null() {{ return {null}; }}"),
                    "        p".into(),
                    format!("    }} else {{ {s} }};"),
                ]);
            } else {
                let ctx = p[2];
                b.extend([
                    format!("    if {delim}.is_null() {{ return {null}; }}"),
                    format!("    if {ctx}.is_null() {{ return {null}; }}"),
                    format!("    let base = if {s}.is_null() {{"),
                    format!("        if (*{ctx}).is_null() {{ return {null}; }}"),
                    format!("        *{ctx}"),
                    format!("    }} else {{ {s} }};"),
                ]);
            }
            b.extend([
                format!("    let n = {cstr}(base as *const {el}).len();"),
                "    let s = core::slice::from_raw_parts_mut(base, n);".into(),
                format!("    match crate::core_rs::strtok::next_token(s, {cstr}({delim})) {{"),
            ]);
            if sp == "tok_ctx" {
                b.push(format!("        crate::core_rs::strtok::Tok::Empty {{ stop }} => {{ *{} = base.add(stop); {null} }},",p[2]));
            } else {
                b.push(format!(
                    "        crate::core_rs::strtok::Tok::Empty {{ .. }} => {null},"
                ));
            }
            b.push("        crate::core_rs::strtok::Tok::Found { start, next } => {".into());
            if stat {
                b.push("            SAVED.store(base.add(next), Ordering::Relaxed);".into());
            } else {
                b.push(format!("            *{} = base.add(next);", p[2]));
            }
            b.extend([
                "            base.add(start)".into(),
                "        }".into(),
                "    }".into(),
            ]);
            b
        }
        "splitpath" | "makepath" => {
            let p = args(params, 5, name)?;
            let (path, drive, dir, fname, ext) = (p[0], p[1], p[2], p[3], p[4]);
            let wide = rs_ty(&params[0].0)?.contains("u16");
            let el = if wide { "u16" } else { "u8" };
            let cstr = if wide { "wcstr_slice" } else { "cstr_slice" };
            let rawm = if wide {
                "wraw_mut_slice"
            } else {
                "raw_mut_slice"
            };
            let nul = if wide { "0u16" } else { "0u8" };
            let colon = if wide { "b':' as u16" } else { "b':'" };
            if sp == "splitpath" {
                vec![
                    format!("    let p = {cstr}({path});"),
                    format!("    if !{drive}.is_null() {{ *{drive} = {nul}; }}"),
                    format!("    if !{dir}.is_null() {{ *{dir} = {nul}; }}"),
                    format!("    if !{fname}.is_null() {{ *{fname} = {nul}; }}"),
                    format!("    if !{ext}.is_null() {{ *{ext} = {nul}; }}"),
                    "    if p.is_empty() { return; }".into(),
                    "    let r = crate::core_rs::path::split(p);".into(),
                    format!("    if r.has_drive && !{drive}.is_null() {{"),
                    format!("        *{drive} = p[0];"),
                    format!("        *{drive}.add(1) = {colon};"),
                    format!("        *{drive}.add(2) = {nul};"),
                    "    }".into(),
                    "    for (out, (a, e)) in [".into(),
                    format!("        ({dir}, r.dir), ({fname}, r.fname), ({ext}, r.ext),"),
                    "    ] {".into(),
                    "        if !out.is_null() {".into(),
                    format!("            let d = {rawm}(out, e - a + 1);"),
                    "            let mut k = 0usize;".into(),
                    "            while k < e - a { d[k] = p[a + k]; k += 1; }".into(),
                    format!("            d[e - a] = {nul};"),
                    "        }".into(),
                    "    }".into(),
                ]
            } else {
                vec![
                    format!("    if {path}.is_null() {{ return; }}"),
                    format!("    let dr = {cstr}({drive} as *const {el});"),
                    format!("    let di = {cstr}({dir} as *const {el});"),
                    format!("    let fl = {cstr}({fname} as *const {el});"),
                    format!("    let ex = {cstr}({ext} as *const {el});"),
                    "    let cap = dr.len() + di.len() + fl.len() + ex.len() + 5;".into(),
                    format!("    let out = {rawm}({path}, cap);"),
                    "    crate::core_rs::path::make(out, dr, di, fl, ex);".into(),
                ]
            }
        }
        "int_to_str" | "int_to_wstr" | "s_erange_n" | "s_erange_w" | "s_einval_n"
        | "s_einval_w" => {
            let secure = sp.starts_with("s_");
            let p = args(params, if secure { 4 } else { 3 }, name)?;
            let v = p[0];
            let rad = p[p.len() - 1];
            let dst = params
                .iter()
                .find_map(|(t, n)| {
                    rs_ty(t)
                        .ok()
                        .filter(|t| t.starts_with("*mut"))
                        .map(|_| n.as_str())
                })
                .ok_or_else(|| format!("{name}: missing destination"))?;
            let size = params
                .iter()
                .find_map(|(t, n)| rs_ty(t).ok().filter(|t| *t == "usize").map(|_| n.as_str()))
                .unwrap_or("None");
            let fr = "crate::core_rs::radix::format_radix";
            let mag=match rs_ty(&params[0].0)? {"i64"=>format!("    let neg = {rad} == 10 && {v} < 0;\n    let mag = if neg {{ ({v} as i64).unsigned_abs() }} else {{ {v} as u64 }};"),"i32"=>format!("    let neg = {rad} == 10 && {v} < 0;\n    let mag = if neg {{ ({v} as i32).unsigned_abs() as u64 }} else {{ {v} as u32 as u64 }};"),"u64"=>format!("    let neg = false;\n    let mag = {v};"),"u32"=>format!("    let neg = false;\n    let mag = {v} as u64;"),t=>return Err(format!("int2str: unsupported value type {t}"))};
            let mut b = Vec::new();
            if !secure {
                b.push(mag);
                b.push(format!("    let f = {fr}(mag, {rad} as u32, neg);"));
                let wide = sp == "int_to_wstr";
                if wide {
                    b.push(format!("    if !{dst}.is_null() {{"));
                }
                let indent = if wide { "        " } else { "    " };
                let cast = if wide { " as u16" } else { "" };
                b.extend([format!("{indent}let mut o = 0usize;"),format!("{indent}if f.negative {{ *{dst} = b'-'{cast}; o = 1; }}"),format!("{indent}let mut i = 0;"),format!("{indent}while i < f.ndigits {{ *{dst}.add(o + i) = f.digits[i]{cast}; i += 1; }}"),format!("{indent}*{dst}.add(o + f.ndigits) = 0;")]);
                if wide {
                    b.push("    }".into());
                }
                b.push(format!("    {dst}"));
            } else {
                let erange = sp.starts_with("s_erange");
                let wide = sp.ends_with("_w");
                b.push(format!(
                    "    if {dst}.is_null() || {size} == 0 || {rad} < 2 || {rad} > 36 {{"
                ));
                if erange {
                    b.push(format!(
                        "        if !{dst}.is_null() && {size} != 0 {{ *{dst} = 0; }}"
                    ));
                }
                b.extend(["        return 22;".into(), "    }".into()]);
                if erange {
                    b.push(mag);
                    b.push(format!("    let f = {fr}(mag, {rad} as u32, neg);"));
                    b.push("    let need = f.ndigits + f.negative as usize + 1;".into());
                    b.push(format!("    if need > {size} {{"));
                    let cast = if wide { " as u16" } else { "" };
                    b.extend(["        let o = f.negative as usize;".into(),format!("        let cnt = {size} - o;"),"        let mut i = 0;".into(),"        while i < cnt && i < f.ndigits {".into(),format!("            *{dst}.add(o + i) = f.digits[f.ndigits - 1 - i]{cast};"),"            i += 1;".into(),"        }".into(),format!("        *{dst} = 0;"),"        return 34;".into(),"    }".into(),"    let mut o = 0usize;".into(),format!("    if f.negative {{ *{dst} = b'-'{cast}; o = 1; }}"),"    let mut i = 0;".into(),format!("    while i < f.ndigits {{ *{dst}.add(o + i) = f.digits[i]{cast}; i += 1; }}"),format!("    *{dst}.add(o + f.ndigits) = 0;"),"    0".into()]);
                } else {
                    b.extend([
                        format!("    let f = {fr}({v}, {rad} as u32, false);"),
                        "    let need = f.ndigits + 1;".into(),
                        format!("    if need > {size} {{ return 22; }}"),
                    ]);
                    if wide {
                        b.extend(["    let mut tmp = [0u8; 2 * crate::core_rs::radix::RADIX_DIGITS_MAX + 2];".into(),"    let mut t = 0;".into(),"    let mut i = 0;".into(),"    while i < f.ndigits { tmp[t] = f.digits[i]; t += 2; i += 1; }".into(),format!("    let db = {dst} as *mut u8;"),"    let mut i = 0;".into(),"    while i < need { *db.add(i) = tmp[i]; i += 1; }".into()]);
                    } else {
                        b.extend(["    let mut i = 0;".into(),format!("    while i < f.ndigits {{ *{dst}.add(i) = f.digits[i]; i += 1; }}"),format!("    *{dst}.add(f.ndigits) = 0;")]);
                    }
                    b.push("    0".into());
                }
            }
            b
        }
        "case_inplace" | "case_tolower" | "case_towupper" => {
            let p = args(params, 1, name)?[0];
            match sp {
                "case_inplace" => {
                    let wide = rs_ty(&params[0].0)? == "*mut u16";
                    let el = if wide { "u16" } else { "u8" };
                    let cstr = if wide { "wcstr_slice" } else { "cstr_slice" };
                    let rawm = if wide {
                        "wraw_mut_slice"
                    } else {
                        "raw_mut_slice"
                    };
                    let cf = field(m, "core_fn")?;
                    let mut b = Vec::new();
                    if m.get("null_guard")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                    {
                        b.push(format!(
                            "    if {p}.is_null() {{ return core::ptr::null_mut(); }}"
                        ));
                    }
                    b.extend([
                        format!("    let n = {cstr}({p} as *const {el}).len();"),
                        format!("    let s = {rawm}({p}, n);"),
                        format!("    crate::core_rs::{cf}(s);"),
                        format!("    {p}"),
                    ]);
                    b
                }
                "case_tolower" => vec![format!("    crate::core_rs::casemap::tolower_ascii({p})")],
                _ => vec![format!("    crate::core_rs::mbcp::upcase({p}) as {rt}")],
            }
        }
        "mut_reverse" | "mut_set" | "mut_nset" => {
            let p = args(
                params,
                match sp {
                    "mut_reverse" => 1,
                    "mut_set" => 2,
                    _ => 3,
                },
                name,
            )?;
            let wide = rs_ty(&params[0].0)? == "*mut u16";
            let el = if wide { "u16" } else { "u8" };
            let cstr = if wide { "wcstr_slice" } else { "cstr_slice" };
            let rawm = if wide {
                "wraw_mut_slice"
            } else {
                "raw_mut_slice"
            };
            let dst = p[0];
            let mut b = vec![format!("    let __n = {cstr}({dst} as *const {el}).len();")];
            if sp == "mut_nset" {
                b.push(format!("    let __lim = {} as i32;", p[2]));
                b.push("    let __take = if __lim <= 0 { 0 } else { core::cmp::min(__n, __lim as usize) };".into());
            }
            let take = if sp == "mut_nset" { "__take" } else { "__n" };
            b.push(format!("    let __buf = {rawm}({dst}, {take});"));
            if sp == "mut_reverse" {
                b.push("    crate::core_rs::strmut::reverse(__buf);".into());
            } else {
                b.push(format!(
                    "    crate::core_rs::strmut::fill(__buf, {} as {el});",
                    p[1]
                ));
            }
            b.push(format!("    {dst}"));
            b
        }
        _ => return Ok(None),
    };
    Ok(Some(finish(name, m, params, rt, b)?))
}
