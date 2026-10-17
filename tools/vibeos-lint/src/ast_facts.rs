//! Compiler facts requested by rules, captured once by the runner.
use serde_json::Value;
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

pub(crate) fn c_signature_ast(root: &Path, arch: &str) -> Result<Value, String> {
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
    serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("clang {arch} returned an invalid AST: {e}"))
}
