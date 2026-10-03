#!/usr/bin/env python3
"""Generate a separate, stable-Rust c2rust base64 candidate for host grading."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
from pathlib import Path

MANIFEST = Path(__file__).parent.parent / "data/pilots/base64.json"


def version(executable: str) -> str:
    result = subprocess.run([executable, "--version"], check=True, capture_output=True, text=True)
    return result.stdout.splitlines()[0]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--repo", type=Path, default=Path.cwd(), help="repository root (default: cwd)"
    )
    parser.add_argument(
        "--output", required=True, type=Path, help="new directory; existing output is refused"
    )
    parser.add_argument(
        "--c2rust", default=shutil.which("c2rust") or str(Path.home() / ".cargo/bin/c2rust")
    )
    args = parser.parse_args()
    repo = args.repo.resolve()
    output = args.output.resolve()
    if output.exists():
        parser.error(f"output exists: {output}; choose a new directory to preserve raw anchors")
    manifest = json.loads(MANIFEST.read_text())
    for relative, expected in manifest["dependencies_sha256"].items():
        observed = hashlib.sha256((repo / relative).read_bytes()).hexdigest()
        if observed != expected:
            parser.error(
                f"dependency changed: {relative}: expected {expected}, observed {observed}"
            )
    source = repo / manifest["source"]
    includes = [
        repo / "vendor/mbedtls/include",
        repo / "vendor/mbedtls/include/mbedtls",
        repo / "vendor/mbedtls",
    ]
    raw_dir = output / "raw"
    output.mkdir(parents=True)
    transpile = [
        args.c2rust,
        "transpile",
        "--output-dir",
        str(raw_dir),
        str(source),
        "--",
        "-std=c11",
        "-DMBEDTLS_BASE64_C",
        *(arg for inc in includes for arg in ("-I", str(inc))),
    ]
    result = subprocess.run(transpile, cwd=repo, capture_output=True, text=True)
    if result.returncode:
        raise SystemExit(f"c2rust failed ({result.returncode}):\n{result.stdout}\n{result.stderr}")
    raw_path = raw_dir / "src/base64.rs"
    raw = raw_path.read_text()
    feature = "#![feature(asm)]\n"
    empty_asm = 'asm!("\\n", inlateout(reg) x, options(preserves_flags, att_syntax));'
    if raw.count(feature) != 1 or raw.count(empty_asm) != 1:
        raise SystemExit(
            "raw anchor has unexpected stable adaptation surface; inspect it before proceeding"
        )
    adapted = raw.replace(feature, "").replace(
        empty_asm, 'asm!("/* {0} */", inlateout(reg) x, options(preserves_flags, att_syntax));'
    )
    adapted_path = output / "adapted_base64.rs"
    adapted_path.write_text(adapted)
    library_path = output / "libbase64-anchor.so"
    rustc = shutil.which("rustc") or "rustc"
    compile_rust = [
        rustc,
        "--edition",
        "2021",
        "--crate-type",
        "cdylib",
        "-O",
        str(adapted_path),
        "-o",
        str(library_path),
    ]
    subprocess.run(compile_rust, check=True)
    report = {
        "versions": {
            "cc": version(shutil.which("cc") or "cc"),
            "clang": version(shutil.which("clang") or "clang"),
            "c2rust": version(args.c2rust),
            "rustc": version(rustc),
        },
        "transpile_argv": transpile,
        "rust_compile_argv": compile_rust,
        "adaptations": [
            {"find": feature, "replace": "", "count": 1},
            {
                "find": empty_asm,
                "replace": (
                    'asm!("/* {0} */", inlateout(reg) x, options(preserves_flags, att_syntax));'
                ),
                "count": 1,
            },
        ],
        "raw_sha256": hashlib.sha256(raw.encode()).hexdigest(),
        "adapted_sha256": hashlib.sha256(adapted.encode()).hexdigest(),
        "library_sha256": hashlib.sha256(library_path.read_bytes()).hexdigest(),
        "c2rust_stdout": result.stdout,
        "c2rust_stderr": result.stderr,
    }
    (output / "generation.json").write_text(json.dumps(report, indent=2) + "\n")
    print(
        json.dumps(
            {
                "library": str(library_path),
                "raw_sha256": report["raw_sha256"],
                "adapted_sha256": report["adapted_sha256"],
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
