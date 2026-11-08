//! Render the CMake rewrite for a newly converted cargo cell.
use super::graph::{args, branches, enclosing_ifs, exclusive, inlinable_source, statements};
use super::{constants, Cell, Error, Result};
use regex::Regex;
use std::{
    collections::{BTreeSet, HashSet},
    path::Path,
};
fn re(s: &str) -> Regex {
    Regex::new(s).expect("constant CMake regex")
}
fn unique(v: &mut Vec<String>, s: String) {
    if !v.contains(&s) {
        v.push(s)
    }
}
fn names_only(a: &[String]) -> bool {
    a.len() >= 4
        && a[1] == "PROPERTIES"
        && a.len() % 2 == 0
        && a[2..].iter().step_by(2).all(|k| {
            constants()["NAME_PROPERTIES"]
                .as_array()
                .is_some_and(|xs| xs.iter().any(|v| v.as_str() == Some(k)))
        })
}
fn ssp(a: &[String]) -> bool {
    a.len() == 3 && a[1] == "PRIVATE" && re(r"^\$<TARGET_OBJECTS:gcc_ssp_\w+>$").is_match(&a[2])
}
fn link_only(a: &[String], pat: &str) -> bool {
    let opts: Vec<_> = a
        .iter()
        .skip(1)
        .filter(|s| *s != "PRIVATE" && *s != "PUBLIC")
        .collect();
    !opts.is_empty() && opts.iter().all(|s| re(pat).is_match(s.trim_matches('"')))
}
fn rc_deps(cmd: &str, raw: &str) -> Option<Vec<String>> {
    let a = args(raw);
    if cmd == "add_rc_deps" && a.first().is_some_and(|v| v.ends_with(".rc")) {
        return Some(a[1..].to_vec());
    }
    if cmd == "set_source_files_properties" {
        let i = a.iter().position(|s| s == "PROPERTIES")?;
        if i > 0
            && i + 1 < a.len()
            && a[..i].iter().all(|s| s.ends_with(".rc"))
            && a[i + 1] == "OBJECT_DEPENDS"
        {
            return Some(
                a[i + 2..]
                    .iter()
                    .map(|s| s.trim_matches('"').to_owned())
                    .collect(),
            );
        }
    }
    None
}
fn retarget(cmd: &str, raw: &str, new: &str) -> String {
    let first = args(raw).first().cloned().unwrap_or_default();
    format!("{cmd}({})", raw.trim().replacen(&first, new, 1))
}
fn basename(s: &str) -> String {
    Path::new(s)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}
fn list_has(name: &str, key: &str) -> bool {
    constants()[key]
        .as_array()
        .is_some_and(|a| a.iter().any(|v| v.as_str() == Some(name)))
}
/// Return a new CMakeLists body; read-only, never writes to disk.
pub fn rewrite_cmake(cmake: &Path, cell: &Cell) -> Result<String> {
    let input = std::fs::read_to_string(cmake)
        .map_err(|e| Error::new(format!("{}: {e}", cmake.display())))?;
    rewrite_text(cmake, &input, cell)
}
pub fn rewrite_text(cmake: &Path, input: &str, cell: &Cell) -> Result<String> {
    let text = if cell.body_env_block.is_empty() {
        input.to_owned()
    } else {
        input.replacen(&cell.body_env_block, "", 1)
    };
    let mut keep = String::new();
    let mut pos = 0;
    let mut body_lib = None;
    let body_var = if cell.body_var.is_empty() {
        format!("{}_rust", cell.target)
    } else {
        cell.body_var.clone()
    };
    let body_re = Regex::new(&format!(
        r"\$\{{{}_LIB\}}|rust_{}\b",
        regex::escape(&body_var),
        regex::escape(&body_var)
    ))?;
    let stmts = statements(&text);
    let shared = stmts.iter().any(|(_, _, cmd, raw)| {
        args(raw).first() != Some(&cell.target)
            && cmd != "rust_staticlib_variant"
            && body_re.is_match(raw)
    });
    // A custom-command body whose archive another statement still reads (ws2help's
    // INTERFACE ws2help_static forwards ${WS2HELP_RUST_LIB} to other modules) keeps its
    // command, custom target and variables; the cell builds the body itself either way.
    let custom_shared = cell.custom_body["output_var"].as_str().is_some_and(|v| {
        let used = format!("${{{v}}}");
        let own_target = cell.custom_body["custom_target"].as_str().unwrap_or("");
        stmts.iter().any(|(_, _, cmd, raw)| {
            let a = args(raw);
            raw.contains(&used)
                && a.first() != Some(&cell.target)
                && !(cmd == "add_custom_command" && re(r"\bcargo build\b").is_match(raw))
                && !(cmd == "add_custom_target"
                    && a.first().map(String::as_str) == Some(own_target))
                && !(["set", "rust_storage_target", "rust_crate_sources"].contains(&cmd.as_str())
                    && a.first().map(String::as_str) == Some(v))
        })
    });
    let guards = enclosing_ifs(&text, &cell.target)?;
    let paths = branches(&stmts);
    let rc_lists = super::graph::list_vars(&text);
    let if_raw: Vec<_> = stmts
        .iter()
        .filter(|(_, _, c, _)| c == "if")
        .map(|(_, _, _, r)| r)
        .collect();
    let shim_root = if cell.cmake_dir.is_absolute() {
        cell.cmake_dir
            .ancestors()
            .find(|p| p.join("sdk/lib/rust").is_dir())
    } else {
        None
    };
    let home = stmts
        .iter()
        .enumerate()
        .find(|(_, (_, _, c, r))| {
            ["add_executable", "add_library"].contains(&c.as_str())
                && args(r).first() == Some(&cell.target)
        })
        .map(|(i, _)| paths[i].clone());
    let src_comment =
        re(r"^\s*#.*?[A-Za-z0-9_${}/.+-]*[A-Za-z0-9_+-]\.(?:c|cpp|cxx|cc|S|s|asm|inc)\b");
    let (mut carried, mut transferred, mut build_deps, mut rc_gen_deps, mut usage) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut pefixup: Vec<String> = Vec::new();
    for (idx, (s, e, cmd, raw)) in stmts.iter().enumerate() {
        let a = args(raw);
        let target = a.first() == Some(&cell.target);
        let other = home.as_ref().is_some_and(|h| exclusive(&paths[idx], h));
        let mut drop = list_has(cmd, "REPLACED")
            && !a.is_empty()
            && (target || (cmd == "add_cd_file" && a.contains(&cell.target)));
        let mut keep_comment = false;
        // A TRANSFERRED statement is re-emitted whole, comments included, so its inner
        // comments are not dead sources to carry: d3dx9's `# math.c wants M_PI` inside
        // its compile definitions was carried as one and doubled.
        let mut moved = false;
        // `add_custom_command(TARGET t POST_BUILD COMMAND native-pefixup <args>
        // $<TARGET_FILE:t>)` (pcix, legide) becomes add_cargo_module's PEFIXUP. Any other
        // command on the target is refused: left in place it names a target the cell
        // removed, and the whole tree's configure fails.
        if cmd == "add_custom_command" && a.len() > 1 && a[0] == "TARGET" && a[1] == cell.target {
            let file = format!("$<TARGET_FILE:{}>", cell.target);
            let rest: Vec<&String> = a.iter().skip(5).filter(|x| *x != "VERBATIM").collect();
            if !(a.len() > 5
                && a[2] == "POST_BUILD"
                && a[3] == "COMMAND"
                && a[4] == "native-pefixup"
                && rest.last() == Some(&&file)
                && rest.iter().filter(|x| ***x == file).count() == 1)
            {
                return Err(Error::new(format!(
                    "{}: add_custom_command(TARGET {} ...) other than one POST_BUILD native-pefixup is not modelled",
                    cell.target, cell.target
                )));
            }
            pefixup.extend(rest[..rest.len() - 1].iter().map(|x| (*x).clone()));
            drop = true;
        }
        if target && cmd == "add_importlibs" {
            // `${_forward_targets}` is a list: `lib${_forward_targets}` would prefix only its
            // first element (d3dx10's forwarders). Import libs carry no usage requirements
            // for a list to lose, so skip variables as target_link_libraries does.
            usage.extend(
                a.iter()
                    .skip(1)
                    .filter(|x| !x.starts_with("${"))
                    .map(|x| format!("lib{x}")),
            );
        }
        if target && cmd == "target_link_libraries" {
            usage.extend(
                a.iter()
                    .skip(1)
                    .filter(|x| {
                        // Linker flags (cdrom's -Wl,--whole-archive) are no usage requirement.
                        !x.starts_with("${") && !x.starts_with('-') && *x != "PRIVATE" && *x != "PUBLIC"
                    })
                    .cloned(),
            );
        }
        if target && cmd == "add_dependencies" {
            for d in a.iter().skip(1).filter(|x| !x.starts_with("rust_")) {
                unique(&mut build_deps, d.clone());
            }
        }
        if cell.rc.is_some() {
            if let Some(deps) = rc_deps(cmd, raw) {
                drop = true;
                rc_gen_deps.extend(
                    super::graph::expand(&deps, &rc_lists)?
                        .into_iter()
                        .filter(|d| d.contains("CMAKE_CURRENT_BINARY_DIR")),
                );
            }
        }
        if target && cmd == "add_idl_reg_scripts" && a.len() > 1 {
            if let Some(suffix) = constants()["REG_SUFFIX"]
                .get(&a[1])
                .and_then(|v| v.as_str())
            {
                keep.push_str(&text[pos..*s]);
                keep.push_str(&text[*s..*e].replacen(
                    "add_idl_reg_scripts",
                    "add_idl_reg_objects",
                    1,
                ));
                // comdlg32 registers its classes only `if(DLL_EXPORT_VERSION GREATER_EQUAL
                // 0x600)`: under a false if() the reg target does not exist, and a plain name
                // in the cell's DEPENDS is a missing file to ninja.
                let reg = format!("{}{suffix}", cell.target);
                rc_gen_deps.push(if paths[idx].is_empty() {
                    reg
                } else {
                    format!("$<TARGET_NAME_IF_EXISTS:{reg}>")
                });
                pos = *e;
                continue;
            }
        }
        if target && list_has(&cmd.to_lowercase(), "TRANSFERRED") {
            drop = true;
            moved = true;
            let h = home.as_deref().unwrap_or(&[]);
            if !paths[idx].starts_with(h) {
                return Err(Error::new(format!(
                    "{}: `{cmd}` sits outside its target's if() branch",
                    cell.target
                )));
            }
            let extra = &paths[idx][h.len()..];
            if extra.iter().any(|(_, branch)| *branch != 0) {
                return Err(Error::new(format!(
                    "{}: `{cmd}` on the target is in an elseif/else branch",
                    cell.target
                )));
            }
            let conditions: Vec<String> = extra.iter().map(|(i, _)| if_raw[*i].clone()).collect();
            transferred.push((
                cmd.clone(),
                raw.clone(),
                conditions,
                same_line_comment(&text, *e),
            ));
        }
        if target && cmd == "set_target_properties" && names_only(&a) {
            drop = true
        }
        if target && cmd == "set_target_cpp_properties" {
            drop = true
        }
        if target
            && cmd == "target_link_options"
            && (link_only(&a, r"^-Wl,--subsystem,[\w:.]+$")
                || link_only(
                    &a,
                    r"^(?:LINKER:|-Wl,)--require-defined=[^,\s]+(?:,--require-defined=[^,\s]+)*$",
                ))
        {
            drop = true
        }
        if target && ["set_subsystem", "set_entrypoint"].contains(&cmd.as_str()) {
            drop = true
        }
        if target && cmd == "target_sources" && ssp(&a) {
            keep.push_str(&text[pos..*s]);
            let old = &text[*s..*e];
            keep.push_str(&old.replacen(old.trim(),&format!("message(FATAL_ERROR \"{} is linked by cargo, which does not link {} (STACK_PROTECTOR)\")",cell.target,a[2]),1));
            pos = *e;
            continue;
        }
        let vars = cell.custom_body.get("vars").and_then(|v| v.as_array());
        let custom = !custom_shared
            && !cell.custom_body.is_null()
            && cell.custom_body.as_object().is_some_and(|o| !o.is_empty());
        if custom
            && ((["set", "rust_storage_target", "rust_crate_sources"].contains(&cmd.as_str())
                && !a.is_empty()
                && vars.is_some_and(|vs| vs.iter().any(|v| v.as_str() == Some(&a[0]))))
                // oleaut32's per-arch `list(APPEND OLEAUT32_RUST_ENV "RUSTFLAGS=...")`.
                || (cmd == "list"
                    && a.len() > 1
                    && a[0] == "APPEND"
                    && vars.is_some_and(|vs| vs.iter().any(|v| v.as_str() == Some(&a[1]))))
                || (cmd == "add_custom_command"
                    && re(r"\bcargo build\b").is_match(raw)
                    && cell.custom_body["crate"].as_str().is_some_and(|k| {
                        // Only the body's own command: a module's other cargo commands
                        // (kernel32's vista/shared archives) may feed other modules.
                        re(&format!(r"\s-p\s+{}\s", regex::escape(k))).is_match(raw)
                    }))
                || (cmd == "add_custom_target"
                    && a.first()
                        .is_some_and(|s| cell.custom_body["custom_target"].as_str() == Some(s))))
        {
            drop = true;
            keep_comment = true;
        }
        if cmd == "rust_staticlib_variant" && a.first() == Some(&body_var) {
            drop = !shared;
            body_lib = Some(a[0].clone());
        }
        if cmd == "list"
            && a.len() > 2
            && a[0] == "APPEND"
            && ["SOURCE", "PCH_SKIP_SOURCE"].contains(&a[1].as_str())
            && !other
        {
            drop = true
        }
        if !drop && raw.contains("${SOURCE}") && !target && !other {
            let src = inlinable_source(&text, cmake.parent().unwrap_or(Path::new(".")))?
                .ok_or_else(|| {
                    Error::new(format!("{}: {cmd} shares ${{SOURCE}}", cmake.display()))
                })?;
            let new = raw.replace("${SOURCE}", &src.join(" "));
            keep.push_str(&text[pos..*s]);
            keep.push_str(&text[*s..*e].replacen(raw, &new, 1));
            pos = *e;
            continue;
        }
        let mut gap = text[pos..*s].to_owned();
        if drop && !keep_comment {
            let mut lines: Vec<_> = gap.split('\n').map(str::to_owned).collect();
            let mut block = Vec::new();
            while lines.len() > 1 && lines[lines.len() - 2].trim_start().starts_with('#') {
                let line = lines.remove(lines.len() - 2);
                block.insert(0, line);
            }
            if block.iter().any(|line| src_comment.is_match(line)) {
                carried.extend(block.iter().map(|line| line.trim().to_owned()));
            }
            gap = lines.join("\n");
        }
        if drop
            && keep_comment
            && gap.trim_end_matches([' ', '\t']).ends_with('\n')
            && gap.contains('#')
        {
            gap.push('\n');
        }
        keep.push_str(&gap);
        if !drop {
            keep.push_str(&text[*s..*e]);
        } else if !moved {
            carried.extend(
                text[*s..*e]
                    .lines()
                    .filter(|l| src_comment.is_match(l))
                    .map(|s| s.trim().to_owned()),
            );
        }
        pos = *e;
        if drop {
            // `set_wine_module_FIXME(t) # CORE-5743: No ARRAY_SIZE macro`: the statement span
            // ends at `)`, so the comment used to survive the drop as an indented orphan line
            // (76 cells). A transferred call carries it to its re-emitted copy instead.
            if let Some(c) = same_line_comment(&text, pos) {
                pos += text[pos..].find('#').unwrap_or(0) + c.len();
            }
        }
    }
    keep.push_str(&text[pos..]);
    let mut out = keep.trim_start_matches('\n').to_owned();
    if let Some(b) = body_lib {
        if !shared
            && Regex::new(&format!(
                r"\$\{{{}_LIB\}}|rust_{}\b",
                regex::escape(&b),
                regex::escape(&b)
            ))?
            .is_match(&out)
        {
            return Err(Error::new(format!(
                "{}: something else still uses {b}",
                cmake.display()
            )));
        }
    }
    if let Some(vs) = cell
        .custom_body
        .get("vars")
        .and_then(|v| v.as_array())
        .filter(|_| !custom_shared)
    {
        for v in vs.iter().filter_map(|v| v.as_str()) {
            if out.contains(&format!("${{{v}}}")) {
                return Err(Error::new(format!(
                    "{}: something else still uses ${{{v}}}",
                    cmake.display()
                )));
            }
        }
    }
    let empty = re(
        r"(?m)^(?:[ \t]*#[^\n]*\n)*[ \t]*if\([^\n]*\)\n(?:[ \t]*\n)*(?:[ \t]*(?:elseif\([^\n]*\)|else\(\))\n(?:[ \t]*\n)*)*[ \t]*endif\(\)\n",
    );
    loop {
        let next = empty.replace_all(&out, "").into_owned();
        if next == out {
            break;
        }
        out = next;
    }
    let glob = re(r"file\(GLOB (\w+)[^)]*\)\n");
    let matches: Vec<_> = glob
        .captures_iter(&out)
        .map(|c| (c[0].to_owned(), c[1].to_owned()))
        .collect();
    for (full, name) in matches {
        if !out.contains(&format!("${{{name}}}")) {
            out = out.replacen(&full, "", 1);
        }
    }
    out = re(r"\n{3,}")
        .replace_all(&out, "\n\n")
        .trim_end()
        .to_owned()
        + "\n";
    let mut block = vec![
        String::new(),
        if cell.body_crate.is_empty() {
            format!(
                "# {} is C, and cargo compiles and links it:",
                cell.output_name
            )
        } else {
            format!(
                "# {} is all Rust ({}) and cargo links it:",
                cell.output_name, cell.body_crate
            )
        },
        format!(
            "# sdk/lib/rust/{} (reactos-link, generated by tools/cargo_cell_gen.py).",
            cell.crate_name()
        ),
    ];
    if !cell.body_crate.is_empty() {
        block.push("# The C stays on disk, uncompiled.".into())
    }
    if !carried.is_empty() {
        block.push("# The translated C, kept on disk:".into());
        let mut seen = HashSet::new();
        block.extend(carried.into_iter().filter(|s| seen.insert(s.clone())));
    }
    let mut extra = Vec::new();
    let shims = format!("{}_shims", cell.target);
    let mut shim_lines = Vec::new();
    let guarded = !cell.arch_shims.is_empty();
    if !cell.c_shims.is_empty() || guarded {
        block.push(
            "# The C still compiled, in an OBJECT library whose compile the cargo link replays."
                .into(),
        );
        let names: Vec<_> = cell
            .c_shims
            .iter()
            .map(|c| {
                if cell.generated.contains(c) {
                    format!("${{CMAKE_CURRENT_BINARY_DIR}}/{}", basename(c))
                } else if cell.foreign.contains(c) {
                    format!("${{REACTOS_SOURCE_DIR}}/{c}")
                } else {
                    shim_root
                        .map(|root| super::graph::shim_name(root, c, cell))
                        .unwrap_or_else(|| basename(c))
                }
            })
            .collect();
        if guarded {
            let var = format!("{}_SHIMS", cell.target.to_uppercase());
            block.push(format!("set({var} {})", names.join(" ")).replace(" )", ")"));
            for arch in cell.arches() {
                let list = cell.arch_shims.get(arch).ok_or_else(|| {
                    Error::new(format!("{}: missing arch shims for {arch}", cell.target))
                })?;
                if !list.is_empty() {
                    block.push(format!("if(ARCH STREQUAL \"{arch}\")"));
                    block.push(format!(
                        "    list(APPEND {var} {})",
                        list.iter()
                            .map(|c| shim_root
                                .map(|root| super::graph::shim_name(root, c, cell))
                                .unwrap_or_else(|| basename(c)))
                            .collect::<Vec<_>>()
                            .join(" ")
                    ));
                    block.push("endif()".into());
                }
            }
            block.push(format!("set({var}_TARGET)"));
            shim_lines.push(format!("add_library({shims} OBJECT ${{{var}}})"));
            shim_lines.push(format!("set({var}_TARGET {shims})"));
        } else {
            shim_lines.push(format!("add_library({shims} OBJECT {})", names.join(" ")));
        }
        // set_module_type(kmdfdriver) linked the wdfdriverentry INTERFACE library, whose usage
        // (the KMDF include dir and WDF defines) cdrom's sources compiled with; the cell drops
        // set_module_type, so the OBJECT library takes that usage itself.
        if cell.kernel_kind == "kmdfdriver" {
            usage.push("wdfdriverentry".into());
        }
        if !usage.is_empty() {
            let mut seen = HashSet::new();
            let vals: Vec<_> = usage
                .into_iter()
                .filter(|x| seen.insert(x.clone()))
                .collect();
            shim_lines.push(format!(
                "target_link_libraries({shims} PRIVATE {})",
                vals.join(" ")
            ));
        }
        if !cell.rc_defines.is_empty() {
            shim_lines.push(format!(
                "target_compile_definitions({shims} PRIVATE {})",
                cell.rc_defines.join(" ")
            ));
        }
        extra.push(shims.clone());
    }
    if cell.rc.is_some() && !extra.contains(&"psdk".to_owned()) {
        extra.push("psdk".into())
    }
    for d in rc_gen_deps {
        if !extra.contains(&d) {
            extra.push(d)
        }
    }
    let objlibs: Vec<_> = extra.iter().filter(|x| *x == &shims).cloned().collect();
    if !transferred.is_empty() && !objlibs.is_empty() {
        block
            .push("# How cmake compiled the target's sources, now on the OBJECT libraries.".into());
        for lib in &objlibs {
            for (cmd, raw, conditions, comment) in &transferred {
                let mut ln = retarget(cmd, raw, lib);
                if let Some(c) = comment {
                    ln = format!("{ln} {c}");
                }
                for condition in conditions.iter().rev() {
                    ln = format!(
                        "if({})\n    {}\nendif()",
                        condition.trim(),
                        ln.replace('\n', "\n    ")
                    );
                }
                if lib == &shims {
                    shim_lines.push(ln)
                } else {
                    block.push(ln)
                }
            }
        }
    }
    if !build_deps.is_empty() && !objlibs.is_empty() {
        block.push("# Generated inputs those compiles read, ordered before them.".into());
        for lib in &objlibs {
            let ln = format!("add_dependencies({lib} {})", build_deps.join(" "));
            if lib == &shims {
                shim_lines.push(ln)
            } else {
                block.push(ln)
            }
        }
    }
    if guarded {
        let var = cell.target.to_uppercase();
        block.push(format!("if({var}_SHIMS)"));
        block.extend(shim_lines.into_iter().map(|s| format!("    {s}")));
        block.push("endif()".into());
        let idx = extra
            .iter()
            .position(|x| x == &shims)
            .ok_or_else(|| Error::new("missing guarded shims target"))?;
        extra[idx] = format!("${{{var}_SHIMS_TARGET}}");
    } else {
        block.extend(shim_lines)
    }
    let mut by_arch = std::collections::BTreeMap::new();
    for arch in cell.arches() {
        let libs = cell.static_libs.get(arch).ok_or_else(|| {
            Error::new(format!("{}: missing static libs for {arch}", cell.target))
        })?;
        by_arch.insert(
            arch,
            libs.iter()
                .filter(|x| !x.starts_with("engine:") && !x.starts_with('-'))
                .cloned()
                .collect::<BTreeSet<_>>(),
        );
    }
    let every: BTreeSet<_> = by_arch.values().flat_map(|s| s.iter().cloned()).collect();
    let common: Vec<_> = every
        .iter()
        .filter(|x| cell.arches().iter().all(|a| by_arch[a].contains(*x)))
        .cloned()
        .collect();
    if !every.is_empty() {
        let var = format!("{}_REUSED", cell.target.to_uppercase());
        if common.is_empty() {
            block.push(format!("set({var})"));
        } else {
            block.push(format!("set({var}"));
            for lib in &common {
                block.push(format!("    ${{CMAKE_BINARY_DIR}}/{lib}"));
            }
            block.last_mut().unwrap().push(')');
        }
        for arch in cell.arches() {
            let only: Vec<_> = by_arch[arch]
                .iter()
                .filter(|l| !common.contains(*l))
                .collect();
            if !only.is_empty() {
                block.push(format!("if(ARCH STREQUAL \"{arch}\")"));
                for lib in only {
                    block.push(format!(
                        "    list(APPEND {var} ${{CMAKE_BINARY_DIR}}/{lib})"
                    ));
                }
                block.push("endif()".into());
            }
        }
        extra.push(format!("${{{var}}}"));
    }
    let named: BTreeSet<_> = cell.static_libs.values().flatten().collect();
    let mut deps = BTreeSet::new();
    for lib in named {
        if let Some(s) = constants()["ENGINE_LIB_DEPS"]
            .get(lib)
            .and_then(|v| v.as_str())
        {
            deps.extend(s.split_whitespace().map(str::to_owned));
        } else if lib.starts_with("engine:c:") {
            let target = &lib[9..];
            let root = cmake
                .ancestors()
                .find(|p| p.join("sdk/lib/rust").is_dir())
                .ok_or_else(|| {
                    Error::new(format!(
                        "{}: cannot resolve engine dependency {lib} without repository root",
                        cmake.display()
                    ))
                })?;
            let index = super::graph::library_index(root)?;
            let (_, found) = super::graph::engine_lib(&format!("lib{target}.a"), &index)
                .ok_or_else(|| Error::new(format!("cannot resolve engine dependency {lib}")))?;
            deps.extend(found);
        }
    }
    extra.extend(deps);
    let kind = if !cell.kernel_kind.is_empty() {
        cell.kernel_kind.as_str()
    } else if cell.module_kind == "Win32Exe" {
        "win32exe"
    } else {
        "win32dll"
    };
    let mut call = vec![
        format!(
            "add_cargo_module({} {} {}",
            cell.target,
            cell.crate_name(),
            cell.output_name
        ),
        format!("    KIND {kind}"),
    ];
    if !cell.body_crate.is_empty() {
        call.push(format!("    BODY_CRATE {}", cell.body_crate));
    }
    if !cell.arch_destinations.is_empty() {
        let var = format!("{}_DESTINATION", cell.target.to_uppercase());
        for (i, arch) in cell.arches().iter().enumerate() {
            block.push(format!(
                "{}(ARCH STREQUAL \"{arch}\")",
                if i == 0 { "if" } else { "elseif" }
            ));
            let d = cell.arch_destinations.get(*arch).ok_or_else(|| {
                Error::new(format!("{}: missing destination for {arch}", cell.target))
            })?;
            block.push(format!("    set({var} {})", d.join(" ")));
        }
        block.push("endif()".into());
        call.push(format!("    DESTINATION ${{{var}}}"));
    } else if !cell.destinations.is_empty() {
        call.push(format!("    DESTINATION {}", cell.destinations.join(" ")));
    } else {
        call.push("    NO_CD".into());
    }
    if let Some(name) = &cell.name_on_cd {
        if !name.is_empty() {
            call.push(format!("    NAME_ON_CD {name}"));
        }
    }
    if !cell.delay_import_libs.is_empty() {
        call.push("    DELAY_IMPORTS".into())
    }
    if cell.no_cab {
        call.push("    NO_CAB".into())
    }
    if !pefixup.is_empty() {
        call.push(format!("    PEFIXUP {}", pefixup.join(" ")));
    }
    extra.extend(cell.body_depends.iter().cloned());
    let mut seen = HashSet::new();
    extra.retain(|x| seen.insert(x.clone()));
    if !extra.is_empty() {
        call.push(format!("    EXTRA_DEPENDS {})", extra.join(" ")));
    } else {
        call.last_mut().unwrap().push(')');
    }
    if !build_deps.is_empty() && objlibs.is_empty() {
        call.push(format!(
            "add_dependencies({} {})",
            cell.target,
            build_deps.join(" ")
        ));
    }
    for c in &cell.extra_cd {
        call.push(format!("add_cargo_cd_file({} {c})", cell.target));
    }
    block.extend(call);
    for cond in guards.iter().rev() {
        let mut lines = vec![block[0].clone(), format!("if({cond})")];
        lines.extend(block[1..].iter().map(|l| {
            if l.is_empty() {
                String::new()
            } else {
                format!("    {l}")
            }
        }));
        lines.push("endif()".into());
        block = lines;
    }
    Ok(out + &block.join("\n") + "\n")
}

/// The `# ...` that ends the line a statement closed on (`foo(x) # why`), without its newline;
/// None when the statement's own end consumed the newline or anything but blanks precedes `#`.
fn same_line_comment(text: &str, end: usize) -> Option<String> {
    let rest = &text[end..];
    let line = &rest[..rest.find('\n').unwrap_or(rest.len())];
    let trimmed = line.trim_start_matches([' ', '\t']);
    (end > 0 && !text[..end].ends_with('\n') && trimmed.starts_with('#'))
        .then(|| trimmed.trim_end().to_owned())
}

#[cfg(test)]
mod tests {
    use super::same_line_comment;
    #[test]
    fn same_line_comment_is_only_the_closing_lines_comment() {
        let t = "f(a) # why\ng(b)\nh(c)  #x  \n";
        assert_eq!(same_line_comment(t, 4).as_deref(), Some("# why"));
        assert_eq!(same_line_comment(t, 20).as_deref(), Some("#x"));
        assert_eq!(same_line_comment(t, 16), None); // g(b)'s span took its newline
        assert_eq!(same_line_comment("f(a) g(b) # no\n", 4), None);
    }
}
