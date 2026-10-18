//! Compiler facts requested by rules, captured once by the runner.
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

fn clang_path(root: &Path) -> Result<PathBuf, String> {
    let mut candidates = vec![root.to_path_buf()];
    if let Some(parent) = root.parent() {
        let mut siblings = std::fs::read_dir(parent)
            .map_err(|e| format!("clang toolchain discovery: {e}"))?
            .map(|e| e.map(|e| e.path()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("clang toolchain discovery: {e}"))?;
        siblings.sort();
        candidates.extend(
            siblings
                .into_iter()
                .filter(|p| p != root && p.is_dir() && p.join(".build-tools").exists()),
        );
    }
    candidates
        .into_iter()
        .map(|p| p.join(".build-tools/llvm-mingw/bin/clang"))
        .find(|p| p.exists())
        .ok_or_else(|| {
            "could not find .build-tools/llvm-mingw/bin/clang in this repo or a sibling checkout"
                .into()
        })
}
const WRAPPER:&str="#define _NTOSKRNL_\n#define _NTSYSTEM_\n#define _USE_DUMMY_PSEH 1\n#define __REACTOS__\n#define _INTRIN_H_\n#define _INTRIN_X86_H_\n#ifndef __int64\n#define __int64 long long\n#endif\n#undef FORCEINLINE\n#define FORCEINLINE\n#include <ntoskrnl.h>\n";

pub(crate) fn c_signature_facts(root: &Path, arch: &str) -> Result<Signatures, String> {
    let (target, defs): (&str, &[&str]) = match arch {
        "amd64" => (
            "x86_64-pc-windows-gnu",
            &[
                "-D_WIN64",
                "-D_M_AMD64",
                "-D__x86_64__",
                "-D_AMD64_",
                "-mxsave",
            ],
        ),
        "i386" => (
            "i686-pc-windows-gnu",
            &["-D_M_IX86", "-D__i386__", "-D_X86_"],
        ),
        "arm64" => (
            "aarch64-pc-windows-gnu",
            &[
                "-D_WIN64",
                "-D_M_ARM64",
                "-D__aarch64__",
                "-D__arm64__",
                "-D_ARM64_",
            ],
        ),
        _ => return Err(format!("unknown C signature architecture {arch}")),
    };
    let build = root.join(format!("build-{arch}"));
    if !build.is_dir() {
        return Err(format!("{}/build-{arch} not found: C signatures require generated headers from this worktree; run ./vibeos-cli build --arch {arch}",root.display()));
    }
    let clang = clang_path(root)?;
    let temp = std::env::temp_dir().join(format!(
        "vibeos-lint-c-signatures-{}-{arch}",
        std::process::id()
    ));
    std::fs::create_dir(&temp).map_err(|e| format!("{}: {e}", temp.display()))?;
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(temp.clone());
    let wrap = temp.join("wrap.c");
    std::fs::write(&wrap, WRAPPER).map_err(|e| format!("{}: {e}", wrap.display()))?;
    let mut cmd = Command::new(&clang);
    cmd.args(["-Xclang", "-ast-dump=json", "-fsyntax-only", "-nostdlibinc"])
        .arg(format!("--target={target}"))
        .args(defs)
        .args([
            "-D_NTOSKRNL_",
            "-D_NTSYSTEM_",
            "-D_USE_DUMMY_PSEH=1",
            "-D__REACTOS__",
            "-DDBG=1",
            "-DWINVER=0x502",
            "-D_WIN32_WINNT=0x502",
            "-ferror-limit=0",
            "-Wno-everything",
        ]);
    let includes = vec![
        build.join("ntoskrnl"),
        root.join("ntoskrnl"),
        root.join("sdk/include"),
        root.join("sdk/include/ddk"),
        root.join("sdk/include/ndk"),
        root.join("sdk/include/psdk"),
        root.join("sdk/include/xdk"),
        root.join("sdk/include/reactos"),
        root.join("sdk/include/reactos/libs"),
        root.join("sdk/include/reactos/drivers"),
        root.join("sdk/include/vcruntime"),
        root.join("sdk/include/winrt"),
        root.join("sdk/lib/pseh/include"),
        root.to_path_buf(),
        root.join("sdk/lib/drivers/arbiter"),
        root.join("sdk/lib/cmlib"),
        root.join("ntoskrnl/include"),
        build.join("ntoskrnl/include"),
        build.join("ntoskrnl/include/internal"),
        build.join("sdk/include"),
        build.join("sdk/include/psdk"),
        build.join("sdk/include/ddk"),
        build.join("sdk/include/dxsdk"),
        build.join("sdk/include/reactos"),
        build.join("sdk/include/reactos/mc"),
        root.join("sdk/include/dxsdk"),
    ];
    for p in includes {
        cmd.arg("-I").arg(p);
    }
    cmd.arg("-isystem").arg(root.join("sdk/include/crt"));
    let versions = clang.parent().unwrap().parent().unwrap().join("lib/clang");
    if versions.is_dir() {
        let mut entries = std::fs::read_dir(&versions)
            .map_err(|e| format!("{}: {e}", versions.display()))?
            .map(|e| e.map(|e| e.path()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("clang builtin headers: {e}"))?;
        entries.sort();
        if let Some(last) = entries.last() {
            cmd.arg("-isystem").arg(last.join("include"));
        }
    }
    let output = cmd
        .arg(wrap)
        .output()
        .map_err(|e| format!("{}: {e}", clang.display()))?;
    if output.stdout.iter().all(u8::is_ascii_whitespace) {
        return Err(format!(
            "clang produced no AST for {arch}: {}",
            String::from_utf8_lossy(&output.stderr)
                .chars()
                .take(4000)
                .collect::<String>()
        ));
    }
    // As in the original oracle, clang may diagnose headers while still returning
    // the declaration AST this verifier consumes. Empty/malformed ASTs are fatal.
    signature_facts(&output.stdout)
        .map_err(|e| format!("clang {arch} returned an invalid AST: {e}"))
}

/// Only the facts consumed by signature verification survive initialization.
/// The full Clang JSON tree and captured stdout are dropped before rules fan out.
pub(crate) struct Signatures {
    pub protos: BTreeMap<String, Proto>,
    pub ctypes: BTreeMap<String, (String, String)>,
    pub json_bytes: usize,
}
impl Signatures {
    /// Owned string payloads, excluding map/vector headers and allocator overhead.
    pub fn string_bytes(&self) -> usize {
        self.protos
            .iter()
            .map(|(name, proto)| {
                name.capacity()
                    + proto.ret.0.capacity()
                    + proto.ret.1.capacity()
                    + proto
                        .params
                        .iter()
                        .map(|(q, d)| q.capacity() + d.capacity())
                        .sum::<usize>()
            })
            .sum::<usize>()
            + self
                .ctypes
                .iter()
                .map(|(name, (q, d))| name.capacity() + q.capacity() + d.capacity())
                .sum::<usize>()
    }
}
fn signature_facts(json: &[u8]) -> Result<Signatures, serde_json::Error> {
    let ast: Value = serde_json::from_slice(json)?;
    let (protos, ctypes) = extract(&ast);
    Ok(Signatures {
        protos,
        ctypes,
        json_bytes: json.len(),
    })
}

#[derive(Clone)]
pub(crate) struct Proto {
    pub ret: (String, String),
    pub params: Vec<(String, String)>,
    pub variadic: bool,
}
fn type_pair(v: &Value) -> (String, String) {
    let q = v
        .get("qualType")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned();
    let d = v
        .get("desugaredQualType")
        .and_then(Value::as_str)
        .unwrap_or(&q)
        .to_owned();
    (q, d)
}
pub(crate) fn extract(
    ast: &Value,
) -> (BTreeMap<String, Proto>, BTreeMap<String, (String, String)>) {
    fn walk(
        v: &Value,
        ps: &mut BTreeMap<String, Proto>,
        ts: &mut BTreeMap<String, (String, String)>,
    ) {
        let Some(o) = v.as_object() else { return };
        match o.get("kind").and_then(Value::as_str).unwrap_or("") {
            "TypedefDecl" => {
                if let Some(n) = o.get("name").and_then(Value::as_str) {
                    ts.insert(n.into(), type_pair(o.get("type").unwrap_or(&Value::Null)));
                }
            }
            "FunctionDecl" => {
                if let Some(n) = o.get("name").and_then(Value::as_str) {
                    let qt = o
                        .get("type")
                        .and_then(|x| x.get("qualType"))
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let ret = (
                        qt.split('(').next().unwrap_or(qt).trim().to_string(),
                        qt.split('(').next().unwrap_or(qt).trim().to_string(),
                    );
                    let params = o
                        .get("inner")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter(|x| x.get("kind").and_then(Value::as_str) == Some("ParmVarDecl"))
                        .map(|x| type_pair(x.get("type").unwrap_or(&Value::Null)))
                        .collect();
                    ps.insert(
                        n.into(),
                        Proto {
                            ret,
                            params,
                            variadic: qt.contains("..."),
                        },
                    );
                }
            }
            _ => {}
        }
        for c in o
            .get("inner")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            walk(c, ps, ts)
        }
    }
    let (mut p, mut t) = (BTreeMap::new(), BTreeMap::new());
    walk(ast, &mut p, &mut t);
    (p, t)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_facts_keep_nested_declarations_and_last_declaration_wins() {
        let facts = {
            let input = serde_json::json!({"kind":"TranslationUnitDecl", "inner":[
                {"kind":"TypedefDecl", "name":"Alias", "type":{"qualType":"old"}},
                {"kind":"FunctionDecl", "name":"F", "type":{"qualType":"int (int)"}},
                {"kind":"LinkageSpecDecl", "inner":[
                    {"kind":"TypedefDecl", "name":"Alias", "type":{"qualType":"ULONG", "desugaredQualType":"unsigned int"}},
                    {"kind":"FunctionDecl", "name":"F", "type":{"qualType":"void (Alias, ...)"}, "inner":[
                        {"kind":"ParmVarDecl", "type":{"qualType":"Alias", "desugaredQualType":"unsigned int"}},
                        {"kind":"CompoundStmt", "inner":[{"kind":"ParmVarDecl", "type":{"qualType":"ignored"}}]}
                    ]},
                    {"kind":"TypedefDecl", "name":"MissingType"},
                    {"kind":"TypedefDecl", "type":{"qualType":"unnamed"}}
                ]}
            ]});
            let bytes = serde_json::to_vec(&input).unwrap();
            let facts = signature_facts(&bytes).unwrap();
            assert_eq!(facts.json_bytes, bytes.len());
            facts
        };
        assert_eq!(facts.ctypes.len(), 2);
        assert_eq!(
            facts.ctypes["Alias"],
            ("ULONG".into(), "unsigned int".into())
        );
        assert_eq!(facts.ctypes["MissingType"], (String::new(), String::new()));
        assert_eq!(facts.protos.len(), 1);
        let proto = &facts.protos["F"];
        assert_eq!(proto.ret, ("void".into(), "void".into()));
        assert_eq!(proto.params, vec![("Alias".into(), "unsigned int".into())]);
        assert!(proto.variadic);
        assert!(facts.string_bytes() > 0);
    }

    #[test]
    fn malformed_json_remains_fatal_and_empty_ast_has_no_facts() {
        assert!(signature_facts(br#"{"kind":"TranslationUnitDecl","inner":["#).is_err());
        let facts = signature_facts(br#"{"kind":"TranslationUnitDecl"}"#).unwrap();
        assert!(facts.protos.is_empty());
        assert!(facts.ctypes.is_empty());
        assert_eq!(facts.string_bytes(), 0);
    }
}
