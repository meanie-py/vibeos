use super::signature::rs_ty;
use toml::Value;
type Params = Vec<(String, String)>;
fn field<'a>(m: &'a Value, k: &str) -> Result<&'a str, String> {
    m.get(k)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing {k}"))
}
pub fn render(name: &str, m: &Value, params: &Params, rt: &str) -> Result<String, String> {
    let sp = field(m, "slice_pattern")?;
    let rc = field(m, "return_collapse")?;
    let cf = field(m, "core_fn")?;
    let typed = params
        .iter()
        .map(|(t, n)| Ok((rs_ty(t)?, n.as_str())))
        .collect::<Result<Vec<_>, String>>()?;
    let decl = params
        .iter()
        .map(|(t, n)| {
            Ok(format!(
                "{}: {}",
                if t.replace(' ', "") == "_locale_t" {
                    format!("_{n}")
                } else {
                    n.clone()
                },
                rs_ty(t)?
            ))
        })
        .collect::<Result<Vec<_>, String>>()?
        .join(", ");
    let ptrs: Vec<_> = typed
        .iter()
        .filter(|(t, _)| t.starts_with('*'))
        .map(|(_, n)| *n)
        .collect();
    let scal: Vec<_> = typed
        .iter()
        .filter(|(t, _)| !t.starts_with('*') && *t != "usize")
        .map(|(_, n)| *n)
        .collect();
    let sz = typed
        .iter()
        .find(|(t, _)| *t == "usize")
        .map(|(_, n)| *n)
        .unwrap_or("None");
    fn get<'a>(p: &[&'a str], i: usize) -> Result<&'a str, String> {
        p.get(i)
            .copied()
            .ok_or_else(|| format!("missing argument {i}"))
    }
    let find = |tys: &[&str]| {
        typed
            .iter()
            .find(|(t, _)| tys.contains(t))
            .map(|(_, n)| *n)
            .ok_or_else(|| format!("{name}: missing type {tys:?}"))
    };
    let mut b: Vec<String> = Vec::new();
    if m.get("null_guard_nls")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        b.push(format!(
            "    if {} {{ return if {sz} != 0 {{ 0x7fff_ffff }} else {{ 0 }}; }}",
            ptrs.iter()
                .map(|p| format!("{p}.is_null()"))
                .collect::<Vec<_>>()
                .join(" || ")
        ));
    }
    match sp {
        "nul_in" | "wnul_in" | "nul_in_n" | "wnul_in_n" => {
            let f = match sp {
                "nul_in" => "cstr_slice",
                "wnul_in" => "wcstr_slice",
                "nul_in_n" => "cstr_slice_bounded",
                _ => "wcstr_slice_bounded",
            };
            for p in &ptrs {
                let extra = if sp.ends_with("_n") {
                    format!(", {sz}")
                } else {
                    String::new()
                };
                b.push(format!("    let {p}_s = {f}({p}{extra});"));
            }
        }
        "nul_in_maxlen" | "wnul_in_maxlen" => {
            let p = get(&ptrs, 0)?;
            let f = if sp.starts_with('w') {
                "wcstr_slice_bounded"
            } else {
                "cstr_slice_bounded"
            };
            b.push(format!("    let {p}_s = {f}({p}, {sz});"));
        }
        "dst_fixed_n" | "wdst_fixed_n" => {
            let d = params.first().ok_or("missing destination")?.1.as_str();
            let s = params.get(1).ok_or("missing source")?.1.as_str();
            let (raw, cstr) = if sp.starts_with('w') {
                ("wraw_mut_slice", "wcstr_slice_bounded")
            } else {
                ("raw_mut_slice", "cstr_slice_bounded")
            };
            b.push(format!("    let {d}_s = {raw}({d}, {sz});"));
            b.push(format!("    let {s}_s = {cstr}({s}, {sz});"));
        }
        "ptr_len" => {
            for p in &ptrs {
                b.push(format!(
                    "    let {p}_s = raw_slice({p} as *const u8, {sz});"
                ));
            }
        }
        "nul_endptr_base" | "wnul_endptr_base" => {
            let wide = sp.starts_with('w');
            let p = find(&[if wide { "*const u16" } else { "*const u8" }])?;
            let f = if wide { "wcstr_slice" } else { "cstr_slice" };
            b.push(format!("    let {p}_s = {f}({p});"));
        }
        "set_raw" | "mem_raw" => {}
        _ => return Err(format!("{name}: unsupported slice_pattern {sp}")),
    }
    let s0 = format!(
        "{}_s",
        if matches!(sp, "dst_fixed_n" | "wdst_fixed_n") {
            params.first().ok_or("missing arg")?.1.as_str()
        } else {
            get(&ptrs, 0)?
        }
    );
    let s1 = ptrs.get(1).map(|p| format!("{p}_s"));
    let core = |a: &str| format!("crate::core_rs::{cf}({a})");
    match rc {
        "usize" => b.push(format!("    {} as {rt}", core(&s0))),
        "usize2" => b.push(format!(
            "    {} as {rt}",
            core(&format!(
                "{s0}, {}",
                s1.as_ref().ok_or("missing second pointer")?
            ))
        )),
        "i32_passthrough" | "i32_cmp_n" => {
            let s = s1.as_ref().ok_or("missing second pointer")?;
            let extra = if rc == "i32_cmp_n" {
                format!(", {sz}")
            } else {
                String::new()
            };
            b.push(format!("    {}", core(&format!("{s0}, {s}{extra}"))));
        }
        "ptr_or_null_from_opt" => {
            let arg = match &s1 {
                Some(s) => format!("{s0}, {s}"),
                None => format!("{s0}, {} as i32", get(&scal, 0)?),
            };
            let base = if sp == "nul_in" {
                params.first().ok_or("missing arg")?.1.as_str()
            } else {
                get(&ptrs, 0)?
            };
            b.extend([
                format!("    match {} {{", core(&arg)),
                format!("        Some(i) => {base}.add(i) as {rt},"),
                "        None => core::ptr::null_mut(),".into(),
                "    }".into(),
            ]);
        }
        "ret_dst" => {
            let d = get(&ptrs, 0)?;
            match sp {
                "dst_fixed_n" | "wdst_fixed_n" => {
                    let s = params.get(1).ok_or("missing source")?.1.as_str();
                    b.push(format!("    let _ = {};", core(&format!("{d}_s, {s}_s"))));
                }
                "set_raw" => b.push(format!(
                    "    raw_fill({d} as *mut u8, {} as u8, {sz});",
                    get(&scal, 0)?
                )),
                "mem_raw" => {
                    let s = params.get(1).ok_or("missing source")?.1.as_str();
                    let n = params.get(2).ok_or("missing size")?.1.as_str();
                    b.push(format!(
                        "    raw_move({d} as *mut u8, {s} as *const u8, {n});"
                    ));
                }
                _ => return Err(format!("{name}: ret_dst with {sp}")),
            }
            b.push(format!("    {d}"));
        }
        "i64_trunc_i32" | "i64_passthrough" => {
            b.push(format!("    let (v, _of) = {};", core(&s0)));
            b.push(format!(
                "    v as {}{rt}",
                if rc == "i64_trunc_i32" { "i32 as " } else { "" }
            ));
        }
        "wtoi64_w" | "wtol_w" => {
            let np = find(&["*const u16"])?;
            if rc == "wtoi64_w" {
                b.push(format!(
                    "    let (v, _of) = crate::core_rs::radix::parse_i64_wrapping_cu({np}_s);"
                ));
                b.push(format!("    v as {rt}"));
            } else {
                b.push(format!(
                    "    crate::core_rs::radix::parse_u32_dec_w({np}_s) as {rt}"
                ));
            }
        }
        "i64_endptr" | "ulong_strtoul" | "wcstoul" | "long_bsd" | "u64_bsd" | "wcstoi64_w"
        | "wcstoui64_w" => {
            let narrow = matches!(rc, "i64_endptr" | "ulong_strtoul");
            let np = if narrow {
                find(&["*const u8"])?
            } else if matches!(rc, "long_bsd" | "u64_bsd") {
                find(&["*const u8", "*const u16"])?
            } else {
                find(&["*const u16"])?
            };
            let ep = if narrow {
                find(&["*mut *mut u8"])?
            } else if matches!(rc, "long_bsd" | "u64_bsd") {
                find(&["*mut *mut u8", "*mut *mut u16"])?
            } else {
                find(&["*mut *mut u16"])?
            };
            let bs = find(&["i32"])?;
            let func = match rc {
                "i64_endptr" | "ulong_strtoul" => "parse_i64_clamped",
                "wcstoul" => "parse_ulong_wide",
                "long_bsd" => "parse_long_bsd",
                "u64_bsd" => "parse_u64_bsd",
                "wcstoi64_w" => "parse_i64_clamped_w",
                _ => "parse_u64_clamped_w",
            };
            if matches!(rc, "wcstoi64_w" | "wcstoui64_w") {
                b.push(format!("    if {np}.is_null() {{ return 0 as {rt}; }}"));
            }
            b.push(format!(
                "    let r = crate::core_rs::radix::{func}({np}_s, {bs});"
            ));
            let guard = if matches!(
                rc,
                "i64_endptr" | "ulong_strtoul" | "wcstoi64_w" | "wcstoui64_w"
            ) {
                "r.base_ok && "
            } else {
                ""
            };
            b.push(format!("    if {guard}!{ep}.is_null() {{"));
            let cast = if narrow {
                "u8"
            } else if rc == "wcstoul" {
                "u16"
            } else {
                "_"
            };
            b.push(format!("        *{ep} = {np}.add(r.end) as *mut {cast};"));
            b.push("    }".into());
            if rc == "ulong_strtoul" {
                b.extend([
                    "    let v = if r.value > 4294967295i64 {".into(),
                    "        4294967295i64".into(),
                    "    } else if r.value < -4294967295i64 {".into(),
                    "        1i64".into(),
                    "    } else { r.value };".into(),
                    format!("    v as u32 as {rt}"),
                ]);
            } else {
                b.push(format!("    r.value as {rt}"));
            }
        }
        _ => return Err(format!("{name}: unsupported return_collapse {rc}")),
    }
    let sig = field(m, "c_signature")?;
    let misf = m
        .get("preserved_misfeature")
        .and_then(Value::as_str)
        .unwrap_or("");
    Ok(format!("/// `{sig}`\n/// preserved misfeature: {misf}\n#[no_mangle]\npub unsafe extern \"C\" fn {name}({decl}) -> {rt} {{\n{}\n}}\n",b.join("\n")))
}
