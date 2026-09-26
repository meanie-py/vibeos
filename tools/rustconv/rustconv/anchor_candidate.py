"""Persist a compiler candidate from one proven CMake TU, without a model rewrite.

Raw anchors are never regenerated to test a later lowering change. Generation uses
one pinned architecture and records failures; adaptation operates on saved Rust.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import time
from pathlib import Path

from . import c2rust_anchor, profile_derive
from .attempts import durable_json
from .config import REACTOS_ROOT
from .cost_ledger import discover_banks


def prior_output(source: Path, target: str) -> dict:
    relative = str(source.relative_to(REACTOS_ROOT))
    banks, absent = discover_banks(REACTOS_ROOT)
    findings = []
    for bank in banks:
        ledger = bank / "outcomes.jsonl"
        if ledger.is_file():
            for number, line in enumerate(ledger.read_text().splitlines(), 1):
                event = json.loads(line)
                if event.get("source_file") == relative:
                    findings.append(f"{ledger}:{number}")
        for path in bank.glob("*/meta.json"):
            meta = json.loads(path.read_text())
            if meta.get("target") == target and source.stem in meta.get("label", ""):
                findings.append(str(path))
        for path in bank.glob("*/request.json"):
            request = json.loads(path.read_text())
            if request.get("source") == relative:
                findings.append(str(path))
    crate = REACTOS_ROOT / "sdk/lib/rust" / f"{target}-rust/src"
    findings.extend(str(p) for p in crate.glob(f"**/{source.stem}.rs"))
    return {
        "source": relative,
        "banks": [str(b) for b in banks],
        "worktrees_without_banks": absent,
        "findings": findings,
    }


def generate(source: Path, module: Path, arch: str, output: Path) -> Path:
    source, module, output = source.resolve(), module.resolve(), output.resolve()
    if output.exists():
        raise ValueError(f"Existing compiler output: replay {output}; never regenerate it")
    started = time.monotonic()
    profile = profile_derive.derive(module, REACTOS_ROOT, arch=arch, sources=(source,))
    if profile.errors:
        raise ValueError(profile_derive.emit_report(profile))
    audit = prior_output(source, profile.module.name)
    if audit["findings"]:
        raise ValueError(f"Existing translation must be reused: {audit['findings']}")
    found = c2rust_anchor._find_db_entry(source, arch)
    if found is None:
        raise ValueError(f"Missing {arch} compile command; build that architecture first")
    entry, directory = found
    flags = c2rust_anchor._extract_clang_flags(entry)
    binary = shutil.which("c2rust")
    if binary is None:
        raise ValueError("c2rust is required; no model fallback")
    output.mkdir(parents=True, exist_ok=False)
    derived = profile_derive.to_json(profile)
    durable_json(output / "profile.json", derived)
    durable_json(output / "prior-output.json", audit)
    extra = [] if c2rust_anchor._targets_i386(flags) else ["-D__thiscall="]
    argv = [
        binary,
        "transpile",
        "--emit-no-std",
        "--emit-c-decl-map",
        "--fail-on-error",
        "--output-dir",
        str(output / "raw"),
        str(source),
        "--",
        *c2rust_anchor._ATTR_NEUTRALIZERS,
        *extra,
        *flags,
    ]
    durable_json(
        output / "request.json",
        {
            "source": str(source.relative_to(REACTOS_ROOT)),
            "arch": arch,
            "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
            "argv": argv,
            "cwd": str(directory),
            "compile_command": entry,
            "transpiler_sha256": hashlib.sha256(Path(binary).read_bytes()).hexdigest(),
            "profile_header_sha256": {
                item["path"]: hashlib.sha256((REACTOS_ROOT / item["path"]).read_bytes()).hexdigest()
                for item in derived["closure"]
                if item.get("path")
            },
            "preparation_seconds": time.monotonic() - started,
        },
    )
    started = time.monotonic()
    try:
        proc = subprocess.run(argv, cwd=directory, capture_output=True, text=True, timeout=180)
    except subprocess.TimeoutExpired as exc:
        for name, content in (("stdout", exc.stdout), ("stderr", exc.stderr)):
            (output / f"{name}.log").write_bytes(
                content.encode() if isinstance(content, str) else content or b""
            )
        durable_json(
            output / "result.json",
            {
                "exit": None,
                "error": "compiler timeout",
                "elapsed_seconds": time.monotonic() - started,
            },
        )
        raise ValueError(f"Compiler timed out; preserved partial output at {output}") from exc
    (output / "stdout.log").write_text(proc.stdout)
    (output / "stderr.log").write_text(proc.stderr)
    raw = output / "raw/src" / f"{source.stem}.rs"
    durable_json(
        output / "result.json",
        {
            "exit": proc.returncode,
            "elapsed_seconds": time.monotonic() - started,
            "raw_sha256": hashlib.sha256(raw.read_bytes()).hexdigest() if raw.exists() else None,
        },
    )
    if proc.returncode or not raw.is_file():
        raise ValueError(
            f"Compiler candidate failed; preserved diagnostics: {output / 'stderr.log'}"
        )
    return raw


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("--module", required=True, type=Path)
    parser.add_argument("--arch", required=True, choices=("i386", "amd64", "arm64"))
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    print(generate(args.source, args.module, args.arch, args.output))


if __name__ == "__main__":
    main()
