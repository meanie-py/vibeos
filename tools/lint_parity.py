#!/usr/bin/env python3
"""Compare Rust plugins with the original Python bytes at a specified git revision.

The original source is compiled from stdin with its original __file__, preserving
root discovery without creating source files while other checks run. It scans
the same corpus as Rust, even after Python removal.
Use --root on a detached older worktree to prove a second source revision too.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO / "tests/qemu"))
sys.path.insert(0, str(REPO))
from cli import lints, rustlints  # noqa: E402


def compare(root: Path, revision: str, names: list[str]) -> dict:
    runner = rustlints.ensure(REPO)
    started = time.monotonic()
    cp = subprocess.run(
        [str(runner), "--root", str(root), "--rules", ",".join(names)],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    rust_seconds = time.monotonic() - started
    results = rustlints.split_output(cp, names)
    report: dict = {
        "root": str(root),
        "oracle_revision": revision,
        "source_revision": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=root, text=True
        ).strip(),
        "rust_seconds": rust_seconds,
        "rules": {},
    }

    def oracle_result(name: str) -> tuple[str, dict]:
        rust = results[name]
        if lints._scanner_error(rust, "stdout"):
            raise RuntimeError(rust.stderr)
        source = subprocess.run(
            ["git", "show", f"{revision}:tools/{name}.py"],
            cwd=REPO,
            capture_output=True,
            check=True,
        ).stdout
        started = time.monotonic()
        replay = (
            "import sys; __file__ = sys.argv[1]; sys.argv = sys.argv[1:]; "
            "exec(compile(sys.stdin.read(), __file__, 'exec'))"
        )
        python = subprocess.run(
            [sys.executable, "-c", replay, str(root / "tools" / f"{name}.py"), "--lint"],
            cwd=root,
            input=source.decode(),
            capture_output=True,
            text=True,
            check=False,
        )
        python_seconds = time.monotonic() - started
        if lints._scanner_error(python, "stdout"):
            raise RuntimeError(f"{name}: Python oracle did not complete: {python.stderr}")
        py_lines = lints._finding_lines(lints._rel(python.stdout, root))
        rs_lines = lints._finding_lines(lints._rel(rust.stdout, root))
        result = {
            "python": len(py_lines),
            "rust": len(rs_lines),
            "python_seconds": python_seconds,
            "python_only": sorted(py_lines - rs_lines),
            "rust_only": sorted(rs_lines - py_lines),
            "python_sha256": hashlib.sha256("\n".join(sorted(py_lines)).encode()).hexdigest(),
            "rust_sha256": hashlib.sha256("\n".join(sorted(rs_lines)).encode()).hexdigest(),
        }
        return name, result

    with ThreadPoolExecutor(max_workers=min(lints.lint_jobs(), len(names))) as pool:
        report["rules"] = dict(pool.map(oracle_result, names))
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=REPO)
    parser.add_argument("--oracle-revision", default="34d3d7e1941")
    parser.add_argument("--rules", required=True, help="comma-separated ported rule names")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    try:
        report = compare(args.root.resolve(), args.oracle_revision, args.rules.split(","))
    except (OSError, RuntimeError, subprocess.SubprocessError) as error:
        print(f"FATAL: {error}", file=sys.stderr)
        return 2
    rendered = json.dumps(report, indent=2) + "\n"
    print(rendered, end="")
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered)
    return int(any(r["python_only"] or r["rust_only"] for r in report["rules"].values()))


if __name__ == "__main__":
    sys.exit(main())
