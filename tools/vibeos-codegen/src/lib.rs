//! Deterministic host-side generators. Rendering is shared with drift checks.
pub mod data;
pub mod messages;
pub mod tables;

use std::path::{Path, PathBuf};

pub fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

pub const GENERATORS: &[&str] = &[
    "apisets-table",
    "bugcodes",
    "bugcodes-h",
    "error-tables",
    "winerror-consts",
    "lcid",
    "ssdt",
    "nls",
    "msvcrt-errlist",
];

pub struct Output {
    pub path: PathBuf,
    pub text: String,
}

/// Render the dependent RTL constants from the newly rendered tables, so --all
/// never reads yesterday's table when a C header/table introduces a new constant.
pub fn generate_all(root: &Path) -> Result<Vec<Output>, String> {
    let mut outputs: Vec<Output> = Vec::new();
    for name in GENERATORS {
        if *name == "winerror-consts" {
            let table = outputs
                .iter()
                .find(|o| o.path.ends_with("rtl-error/src/tables.rs"))
                .ok_or("error tables must be rendered before constants")?;
            let text = tables::winerror_consts(
                &table.text,
                &[
                    read(&root.join("sdk/include/psdk/winerror.h"))?,
                    read(&root.join("sdk/include/psdk/ntstatus.h"))?,
                ],
            )?;
            outputs.push(Output {
                path: root.join("sdk/lib/rust/rtl-error/src/winerr.rs"),
                text,
            });
        } else {
            outputs.extend(generate(root, name, None, None, &[])?);
        }
    }
    Ok(outputs)
}

pub fn replace_block(text: &str, begin: &str, end: &str, block: &str) -> Result<String, String> {
    if text.matches(begin).count() != 1 || text.matches(end).count() != 1 {
        return Err(format!("expected exactly one {begin} .. {end} block"));
    }
    let (head, rest) = text.split_once(begin).unwrap();
    let (_, tail) = rest
        .split_once(end)
        .ok_or("generated-block markers are out of order")?;
    Ok(format!("{head}{block}{tail}"))
}

/// Resolve only declared inputs under this checkout. Explicit overrides are for fixtures
/// and one-off extraction; relative override paths are relative to the caller's cwd.
pub fn generate(
    root: &Path,
    name: &str,
    input: Option<&Path>,
    output: Option<&Path>,
    headers: &[PathBuf],
) -> Result<Vec<Output>, String> {
    if !GENERATORS.contains(&name) {
        return Err(format!("unknown generator: {name}"));
    }
    if input.is_some() && matches!(name, "bugcodes" | "bugcodes-h" | "nls") {
        return Err(format!(
            "{name} uses its declared checkout inputs; --input is unsupported"
        ));
    }
    if !headers.is_empty() && name != "winerror-consts" {
        return Err("--header is only valid for winerror-consts".into());
    }
    if name == "nls" {
        if output.is_some() {
            return Err("nls emits three files; --output is unsupported".into());
        }
        return ["casemap", "collation", "digitmap"]
            .into_iter()
            .map(|stem| {
                let filename = format!("{stem}.c");
                Ok(Output {
                    text: data::nls(
                        &read(
                            &root
                                .join("dll/win32/kernel32/winnls/string")
                                .join(&filename),
                        )?,
                        &filename,
                    )?,
                    path: root.join(format!("sdk/lib/rust/kernel32-rust/src/string/{stem}.rs")),
                })
            })
            .collect();
    }
    let (source, destination) = match name {
        "apisets-table" => (
            "sdk/lib/apisets/apisets.table.c",
            "sdk/lib/rust/ntdll-rust/src/apisets_table.rs",
        ),
        "bugcodes" => ("", "sdk/lib/rust/ntkrnl-sys/src/bugcodes.rs"),
        "bugcodes-h" => ("", "sdk/lib/rust/reactos-link/committed-headers/bugcodes.h"),
        "error-tables" => (
            "sdk/lib/rtl/error.c",
            "sdk/lib/rust/rtl-error/src/tables.rs",
        ),
        "winerror-consts" => (
            "sdk/lib/rust/rtl-error/src/tables.rs",
            "sdk/lib/rust/rtl-error/src/winerr.rs",
        ),
        "lcid" => (
            "dll/win32/kernel32/kernel32_vista/LocaleNameToLCID.c",
            "sdk/lib/rust/kernel32-vista-rust/src/localenametolcid_table.rs",
        ),
        "ssdt" => (
            "ntoskrnl/include/sysfuncs.h",
            "sdk/lib/rust/ntoskrnl-rust/src/ke/ssdt.rs",
        ),
        "msvcrt-errlist" => (
            "dll/win32/msvcrt/errno.c",
            "sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/errno.rs",
        ),
        _ => unreachable!(),
    };
    let path = output
        .map(Path::to_owned)
        .unwrap_or_else(|| root.join(destination));
    let source_path = input
        .map(Path::to_owned)
        .unwrap_or_else(|| root.join(source));
    let text = match name {
        "bugcodes" => messages::bugcodes(root)?,
        "bugcodes-h" => messages::bugcodes_h(root)?,
        "apisets-table" => data::apisets(&read(&source_path)?)?,
        "error-tables" if input.is_none() => tables::rtl_error_tables(
            &read(&source_path)?,
            &read(&root.join("sdk/include/psdk/winerror.h"))?,
            &read(&root.join("sdk/include/psdk/ntstatus.h"))?,
        )?,
        "error-tables" => tables::error_tables(&read(&source_path)?)?,
        "winerror-consts" => {
            let headers = if headers.is_empty() {
                vec![
                    root.join("sdk/include/psdk/winerror.h"),
                    root.join("sdk/include/psdk/ntstatus.h"),
                ]
            } else {
                headers.to_vec()
            };
            let headers: Vec<_> = headers.iter().map(|p| read(p)).collect::<Result<_, _>>()?;
            tables::winerror_consts(&read(&source_path)?, &headers)?
        }
        "lcid" => tables::lcid(&read(&source_path)?)?,
        "ssdt" => data::ssdt(&read(&source_path)?)?,
        "msvcrt-errlist" => replace_block(
            &read(&path)?,
            data::ERRLIST_BEGIN,
            data::ERRLIST_END,
            &data::errlist(&read(&source_path)?)?,
        )?,
        _ => unreachable!(),
    };
    Ok(vec![Output { path, text }])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn block_replacement_preserves_surrounding_code_and_rejects_ambiguity() {
        assert_eq!(
            replace_block("headBEGINoldENDtail", "BEGIN", "END", "BEGINnewEND").unwrap(),
            "headBEGINnewENDtail"
        );
        for text in ["BEGIN old", "END BEGIN", "BEGIN BEGIN END"] {
            assert!(replace_block(text, "BEGIN", "END", "BEGINnewEND").is_err());
        }
    }
}
