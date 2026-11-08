use super::{constants, CFlags, Cell, Error, RcFlags, Result, ARCHES};
use std::collections::BTreeMap;

fn policy(map: &str, arch: &str) -> &'static str {
    constants()[map][arch]
        .as_str()
        .expect("cell architecture constant")
}
fn triple(a: &str) -> &'static str {
    policy("TRIPLE", a)
}
fn quoted(xs: &[String]) -> String {
    xs.iter()
        .map(|x| format!("\"{x}\""))
        .collect::<Vec<_>>()
        .join(", ")
}
pub fn rs_list(items: &[String], indent: usize) -> String {
    let one = format!("&[{}]", quoted(items));
    if indent + one.chars().count() < 92 {
        return one;
    }
    format!(
        "&[\n{}{}]",
        items
            .iter()
            .map(|x| format!("{}\"{}\",\n", " ".repeat(indent + 4), x))
            .collect::<String>(),
        " ".repeat(indent)
    )
}
fn binding(
    prefix: &str,
    comments: &[&str],
    tables: Vec<(&str, BTreeMap<String, Vec<String>>)>,
) -> Vec<String> {
    let mut out: Vec<String> = comments.iter().map(|s| (*s).into()).collect();
    for (name, vals) in tables {
        // The arches the table carries: all of them, or a gated cell's own.
        let arches: Vec<&str> = ARCHES.into_iter().filter(|a| vals.contains_key(*a)).collect();
        let reference = &vals[arches[0]];
        if arches.iter().all(|a| &vals[*a] == reference) {
            out.push(format!(
                "    let {prefix}_{name}: &[&str] = {};",
                rs_list(reference, 4)
            ));
        } else {
            out.push(format!(
                "    let {prefix}_{name}: &[&str] = match arch.as_str() {{"
            ));
            for a in arches {
                out.push(format!(
                    "        \"{}\" => {},",
                    triple(a),
                    rs_list(&vals[a], 8)
                ));
            }
            out.extend(["        _ => unreachable!(),".into(), "    };".into()]);
        }
    }
    out
}
pub fn rc_binding(flags: &BTreeMap<String, RcFlags>) -> Vec<String> {
    let tables = ["includes", "flags", "defines"]
        .into_iter()
        .map(|name| {
            (
                name,
                ARCHES
                    .into_iter()
                    .filter(|a| flags.contains_key(*a))
                    .map(|a| {
                        (
                            a.to_owned(),
                            match name {
                                "includes" => &flags[a].includes,
                                "flags" => &flags[a].flags,
                                _ => &flags[a].defines,
                            }
                            .clone(),
                        )
                    })
                    .collect(),
            )
        })
        .collect();
    let mut out = binding(
        "rc",
        &[
            "    // The module's part of cmake's windres command; reactos-link adds what every",
            "    // module shares from reactos-link/cflags/rc.<arch>.flags (Linker::rc).",
        ],
        tables,
    );
    out.push(String::new());
    out
}
pub fn c_binding(
    srcs: &BTreeMap<String, Vec<String>>,
    flags: &BTreeMap<String, CFlags>,
) -> Vec<String> {
    let mut tables = vec![("sources", srcs.clone())];
    tables.extend(["includes", "flags", "defines"].into_iter().map(|name| {
        (
            name,
            ARCHES
                .into_iter()
                .filter(|a| flags.contains_key(*a))
                .map(|a| {
                    (
                        a.to_owned(),
                        match name {
                            "includes" => &flags[a].includes,
                            "flags" => &flags[a].flags,
                            _ => &flags[a].defines,
                        }
                        .clone(),
                    )
                })
                .collect(),
        )
    }));
    binding(
        "c",
        &[
            "    // The C left, and its part of cmake's C rule; reactos-link adds what every C",
            "    // compile shares from reactos-link/cflags/c.<arch>.flags (Linker::c_compiled).",
        ],
        tables,
    )
}
pub fn rc_call(rc: &str) -> String {
    format!("        .rc(\"{rc}\", rc_includes, rc_flags, rc_defines)")
}
pub fn cfg(arches: &[&str]) -> String {
    let p: Vec<_> = arches
        .iter()
        .map(|a| format!("target_arch = \"{}\"", policy("CFG_ARCH", a)))
        .collect();
    if p.len() == 1 {
        p[0].clone()
    } else {
        format!("any({})", p.join(", "))
    }
}
pub fn user_panic_deps(cell: &Cell) -> String {
    let arches = cell.arches();
    let full: Vec<_> = arches.iter().copied().filter(|a| cell.panics[*a]).collect();
    let common = "panic-common = { path = \"../panic-common\", features = [\"user\"] }\n";
    let lean = "panic-lean = { path = \"../panic-lean\" }\n";
    if full.len() == arches.len() {
        return format!("[dependencies]\n{common}");
    }
    let mut out = format!("[dependencies]\n{lean}");
    if !full.is_empty() {
        out += &format!("[target.'cfg({})'.dependencies]\n{common}", cfg(&full));
    }
    out
}
pub fn panic_deps(cell: &Cell) -> String {
    if cell.shell_less {
        return "[dependencies]\npanic-lean = { path = \"../panic-lean\" }\n".into();
    }
    if cell.module_kind == "KernelDll" {
        if cell.panics.values().all(|v| *v) {
            "[dependencies]\npanic-common = { path = \"../panic-common\", features = [\"kernel\"] }\n".into()
        } else {
            "[dependencies]\npanic-lean = { path = \"../panic-lean\" }\n".into()
        }
    } else {
        user_panic_deps(cell)
    }
}
pub fn cargo_toml(cell: &Cell) -> Result<String> {
    validate_cell(cell, false)?;
    let name = cell.crate_name();
    let bin = name.strip_suffix("-cargo").unwrap_or(&name);
    Ok(format!("# Cargo-driven `{}` link, generated by tools/cargo_cell_gen.py.\n\n[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"{}\"\npath = \"src/main.rs\"\n\n{}[build-dependencies]\nreactos-link = {{ path = \"../reactos-link\" }}\n",cell.output_name,name,bin,panic_deps(cell)))
}
pub const MAIN_RS_HEAD: &str = "//! Empty bin shell: the PE's entry (the CRT startup, DllMain, DriverEntry) is in the\n//! linked body, and nothing here runs. Cargo needs a `main.rs` to drive the link.\n//!\n//! The handler is panic_common only where cmake's link carried the body's handler;\n//! elsewhere panic-lean, which keeps all of `core` out of a module that cannot panic.\n\n#![no_std]\n#![no_main]\n#![allow(internal_features)]\n\n#[panic_handler]\nfn panic(info: &core::panic::PanicInfo) -> ! {\n";
pub fn main_rs(cell: &Cell) -> Result<String> {
    validate_cell(cell, false)?;
    let arches = cell.arches();
    let full: Vec<_> = arches.iter().copied().filter(|a| cell.panics[*a]).collect();
    if cell.shell_less {
        return Ok(format!(
            "{MAIN_RS_HEAD}    // A shell-less cell: link-guard drops this bin's objects from the link\n    // (reactos-link Linker::shell_less), so this handler is never in the PE.\n    let _ = info;\n    panic_lean::panic_fastfail()\n}}\n"
        ));
    }
    if cell.module_kind == "KernelDll" && full.is_empty() {
        return Ok(format!(
            "{MAIN_RS_HEAD}    let _ = info;\n    panic_lean::panic_fastfail()\n}}\n"
        ));
    }
    let common = "    panic_common::panic_common(env!(\"CARGO_PKG_NAME\"), info)\n";
    let lean = format!(
        "    panic_lean::panic_lean(b\"{}\\0\", info)\n",
        cell.crate_name()
    );
    Ok(if full.len() == arches.len() {
        format!("{MAIN_RS_HEAD}{common}}}\n")
    } else if full.is_empty() {
        format!("{MAIN_RS_HEAD}{lean}}}\n")
    } else {
        format!(
            "{MAIN_RS_HEAD}    #[cfg({})]\n{};\n    #[cfg(not({}))]\n{};\n}}\n",
            cfg(&full),
            common
                .trim_end()
                .replacen("    panic_common", "    return panic_common", 1),
            cfg(&full),
            lean.trim_end()
                .replacen("    panic_lean", "    return panic_lean", 1)
        )
    })
}
pub fn build_rs(cell: &Cell) -> Result<String> {
    validate_cell(cell, true)?;
    let arches = cell.arches();
    let mut l = vec![
        format!(
            "//! Cargo-driven `{}` link, generated by tools/cargo_cell_gen.py",
            cell.output_name
        ),
        "//! from the cmake link it replaces.".into(),
        if cell.body_crate.is_empty() {
            "//! The module is C, compiled and linked by reactos-link: there is no Rust body."
                .into()
        } else {
            format!(
                "//! The body is {}; reactos-link builds the PE.",
                cell.body_crate
            )
        },
        "".into(),
        format!(
            "use reactos_link::{{Linker, ModuleKind{}}};",
            if cell.msvcrtex.is_some() {
                ", MsvcrtexKind"
            } else {
                ""
            }
        ),
        "use std::env;".into(),
        "".into(),
        "fn main() {".into(),
        "    let arch = env::var(\"CARGO_CFG_TARGET_ARCH\").expect(\"CARGO_CFG_TARGET_ARCH\");"
            .into(),
    ];
    let dll = cell.module_kind == "Win32Dll";
    let kernel = cell.module_kind == "KernelDll";
    l.push(format!(
        "    let (target_json, entry{}) = match arch.as_str() {{",
        if dll || kernel { ", image_base" } else { "" }
    ));
    for a in arches.iter().copied() {
        let base = cell.image_base[a];
        let tail = if dll {
            base.map(|v| format!(", Some({v:#x}u64)"))
                .unwrap_or(", None".into())
        } else if kernel {
            format!(", {:#x}u64", base.expect("kernel image base"))
        } else {
            String::new()
        };
        l.push(format!(
            "        \"{}\" => (\"{}\", \"{}\"{}),",
            triple(a),
            policy("TARGET", a),
            cell.entries[a],
            tail
        ));
    }
    l.extend([
        format!(
            "        o => panic!(\"{}: unsupported arch {{o}}\"),",
            cell.crate_name()
        ),
        "    };".into(),
        "".into(),
    ]);
    if !cell.subsystems.is_empty() {
        l.extend([
            "    // The arches link different subsystem versions, as cmake's links did.".into(),
            "    let subsystem = match arch.as_str() {".into(),
        ]);
        for a in arches.iter().copied() {
            l.push(format!(
                "        \"{}\" => \"{}\",",
                triple(a),
                cell.subsystems[a]
            ));
        }
        l.extend([
            "        _ => unreachable!(),".into(),
            "    };".into(),
            "".into(),
        ]);
    }
    let late: BTreeMap<String, Vec<String>> = arches
        .iter()
        .copied()
        .map(|a| (a.into(), cell.late_libs.get(a).cloned().unwrap_or_default()))
        .collect();
    let early: BTreeMap<String, Vec<String>> = arches
        .iter()
        .copied()
        .map(|a| {
            (
                a.into(),
                cell.static_libs[a]
                    .iter()
                    .filter(|x| !late[a].contains(x))
                    .cloned()
                    .collect(),
            )
        })
        .collect();
    let tables = if !cell.link_order.values().any(|v| !v.is_empty()) {
        vec![("static_libs", &early), ("late_libs", &late)]
    } else {
        vec![("link_order", &cell.link_order)]
    };
    for (var, table) in tables {
        if !table.values().any(|v| !v.is_empty()) {
            continue;
        }
        l.push(format!("    let {var}: &[&str] = match arch.as_str() {{"));
        for a in arches.iter().copied() {
            l.push(format!(
                "        \"{}\" => &[{}],",
                triple(a),
                quoted(&table[a])
            ));
        }
        l.extend([
            "        _ => unreachable!(),".into(),
            "    };".into(),
            "".into(),
        ]);
    }
    if !cell.require_defined.is_empty() {
        l.extend([
            "    // target_link_options(LINKER:--require-defined=...): defined up front, so the"
                .into(),
            "    // body's definition wins over an import thunk of the same name.".into(),
            "    let require_defined: &[&str] = match arch.as_str() {".into(),
        ]);
        for a in arches.iter().copied() {
            let ops: Vec<_> = cell.require_defined[a]
                .iter()
                .map(|x| format!("-Wl,--require-defined={x}"))
                .collect();
            l.push(format!("        \"{}\" => &[{}],", triple(a), quoted(&ops)));
        }
        l.extend([
            "        _ => unreachable!(),".into(),
            "    };".into(),
            "".into(),
        ]);
    }
    if cell.rc.is_some() {
        let flags = cell
            .rc_flags
            .iter()
            .filter(|(a, _)| arches.contains(&a.as_str()))
            .map(|(a, f)| (a.clone(), f.clone()))
            .collect();
        l.extend(rc_binding(&flags));
    }
    if !cell.body_rustflags.is_empty() {
        l.extend([
            "    // cmake built the body with these RUSTFLAGS (its rust_staticlib_variant ENV)."
                .into(),
            "    let body_rustflags = match arch.as_str() {".into(),
        ]);
        for a in arches.iter().copied() {
            if let Some(flags) = cell.body_rustflags.get(a) {
                l.push(format!("        \"{}\" => Some(\"{}\"),", triple(a), flags));
            }
        }
        l.extend([
            "        _ => None,".into(),
            "    };".into(),
            "    let mut link = Linker::new(target_json);".into(),
            "    if let Some(flags) = body_rustflags {".into(),
            "        link = link.rust_build_rustflags(flags);".into(),
            "    }".into(),
            "    link".into(),
        ]);
    } else {
        l.push("    Linker::new(target_json)".into());
    }
    if !cell.body_crate.is_empty() {
        if !cell.body_features.is_empty() || !cell.body_defaults {
            l.push(format!(
                "        .rust_lib(\"{}\", &[{}], {})",
                cell.body_crate,
                quoted(&cell.body_features),
                cell.body_defaults
            ));
        } else {
            l.push(format!("        .rust_staticlib(\"{}\")", cell.body_crate));
        }
        if kernel {
            l.push("        .rust_user_target()".into());
        }
    }
    if !cell.arch_shims.is_empty() {
        l.extend([
            "        .c_from_compile_db(".into(),
            format!("            \"{}_shims\",", cell.target),
            "            match arch.as_str() {".into(),
        ]);
        for a in arches.iter().copied() {
            let mut shims = cell.c_shims.clone();
            shims.extend(cell.arch_shims[a].clone());
            l.push(format!(
                "                \"{}\" => &[{}],",
                triple(a),
                quoted(&shims)
            ));
        }
        l.extend([
            "                _ => unreachable!(),".into(),
            "            },".into(),
            "        )".into(),
        ]);
    } else if !cell.c_shims.is_empty() {
        l.push(format!(
            "        .c_from_compile_db(\"{}_shims\", &[{}])",
            cell.target,
            quoted(&cell.c_shims)
        ));
    }
    if let Some(rc) = &cell.rc {
        l.push(rc_call(rc));
    }
    for rc in &cell.extra_rc {
        l.push(rc_call(rc));
    }
    if let Some((spec, dll)) = &cell.spec {
        l.push(format!("        .spec2def(\"{spec}\", \"{dll}\")"));
    }
    if cell.spec_stubs {
        l.push("        .spec_stubs()".into());
    }
    let regular: BTreeMap<_, _> = cell.import_libs.iter().map(|x| (&x.2, x)).collect();
    let delayed: BTreeMap<_, _> = cell.delay_import_libs.iter().map(|x| (&x.2, x)).collect();
    let order = if cell.import_order.is_empty() {
        let mut v: Vec<_> = cell
            .import_libs
            .iter()
            .map(|x| (x.2.clone(), false))
            .collect();
        v.extend(cell.delay_import_libs.iter().map(|x| (x.2.clone(), true)));
        v
    } else {
        cell.import_order.clone()
    };
    for (base, delay) in order {
        let (spec, dll, _) = if delay {
            delayed[&base]
        } else {
            regular[&base]
        };
        l.push(format!(
            "        .{}(\"{}\", \"{}\", \"{}\")",
            if delay {
                "delay_import_lib"
            } else {
                "import_lib"
            },
            spec,
            dll,
            base
        ));
    }
    if !cell.link_order.values().any(|v| !v.is_empty()) && early.values().any(|v| !v.is_empty()) {
        l.extend([
            "        // cmake-built static libraries, reused until each has a cargo home.".into(),
            "        .static_libs(static_libs)".into(),
        ]);
    }
    if !cell.link_order.values().any(|v| !v.is_empty()) && late.values().any(|v| !v.is_empty()) {
        l.extend([
            "        // ...and those cmake linked after the body, in the same order.".into(),
            "        .static_libs_after_body(late_libs)".into(),
        ]);
    }
    if !!cell.link_order.values().any(|v| !v.is_empty()) {
        l.extend([
            "        // The body, import libs and static libs in cmake's link order: the first"
                .into(),
            "        // to define a symbol wins. cmake-built ones are reused until each has a"
                .into(),
            "        // cargo home; `engine:` ones reactos-link builds.".into(),
            "        .link_order(link_order)".into(),
        ]);
    }
    if kernel {
        l.extend([
            "        // cmake's kernel-mode links keep every section.".into(),
            "        .keep_all_sections()".into(),
        ]);
    }
    if let Some(m) = &cell.msvcrtex {
        l.push(format!("        .msvcrtex_crt(MsvcrtexKind::{m})"));
    }
    if cell.shell_less {
        l.extend([
            "        // No code of its own: link nothing from rustc's bin (its panic".into(),
            "        // handler, i386's rsbegin/rsend), so the PE is cmake's.".into(),
            "        .shell_less()".into(),
        ]);
    }
    if let Some(s) = cell.stack.filter(|s| *s != 0) {
        l.push(format!("        .stack_reserve({s})"));
    }
    if !cell.require_defined.is_empty() {
        l.push("        .link_args(require_defined)".into());
    }
    if kernel {
        l.extend([
            "        .link(ModuleKind::KernelDll {".into(),
            "            entry: entry.into(),".into(),
            "            image_base,".into(),
            "        });".into(),
        ]);
    } else if dll {
        let sub = if !cell.subsystems.is_empty() {
            "Some(subsystem.into())".into()
        } else {
            cell.subsystem
                .as_ref()
                .map(|s| format!("Some(\"{s}\".into())"))
                .unwrap_or("None".into())
        };
        l.extend([
            "        .link(ModuleKind::Win32Dll {".into(),
            "            entry: entry.into(),".into(),
            "            image_base,".into(),
            format!("            subsystem: {sub},"),
            "        });".into(),
        ]);
    } else {
        let sub = if cell.subsystems.is_empty() {
            format!("\"{}\"", cell.subsystem.as_deref().unwrap_or("None"))
        } else {
            "subsystem".into()
        };
        l.extend([
            "        .link(ModuleKind::Win32Exe {".into(),
            "            entry: entry.into(),".into(),
            format!("            subsystem: {sub}.into(),"),
            "        });".into(),
        ]);
    }
    l.push("}".into());
    Ok(l.join("\n") + "\n")
}

pub fn validate_cell(cell: &Cell, build: bool) -> Result<()> {
    for arch in cell.arches() {
        if !cell.panics.contains_key(arch) {
            return Err(Error::new(format!(
                "{}: missing panic setting for {arch}",
                cell.target
            )));
        }
        if build {
            if !cell.entries.contains_key(arch)
                || !cell.image_base.contains_key(arch)
                || !cell.static_libs.contains_key(arch)
            {
                return Err(Error::new(format!(
                    "{}: incomplete build settings for {arch}",
                    cell.target
                )));
            }
            if cell.module_kind == "KernelDll" && cell.image_base[arch].is_none() {
                return Err(Error::new(format!(
                    "{}: missing kernel image base for {arch}",
                    cell.target
                )));
            }
            if !cell.subsystems.is_empty() && !cell.subsystems.contains_key(arch)
                || !!cell.link_order.values().any(|v| !v.is_empty())
                    && !cell.link_order.contains_key(arch)
                || !cell.require_defined.is_empty() && !cell.require_defined.contains_key(arch)
                || !cell.arch_shims.is_empty() && !cell.arch_shims.contains_key(arch)
                || cell.rc.is_some() && !cell.rc_flags.contains_key(arch)
            {
                return Err(Error::new(format!(
                    "{}: incomplete architecture settings for {arch}",
                    cell.target
                )));
            }
        }
    }
    if build {
        for (name, delay) in &cell.import_order {
            let valid = if *delay {
                cell.delay_import_libs.iter().any(|v| &v.2 == name)
            } else {
                cell.import_libs.iter().any(|v| &v.2 == name)
            };
            if !valid {
                return Err(Error::new(format!(
                    "{}: unknown import {name}",
                    cell.target
                )));
            }
        }
    }
    Ok(())
}
