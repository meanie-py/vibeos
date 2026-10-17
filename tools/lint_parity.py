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
import os
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO / "tests/qemu"))
sys.path.insert(0, str(REPO))
from cli import lints, rustlints  # noqa: E402


def oracle_specs(root: Path, revision: str) -> dict[str, tuple[list[str], str]]:
    """Use the original registry, including its flags and output streams.

    Its file-existence guards consult the selected git tree, so a removed Python
    scanner still has its original command. We execute only registry construction;
    scanner source is independently fetched with git show below.
    """
    tracked = set(
        subprocess.check_output(["git", "ls-tree", "-r", "--name-only", "-z", revision], cwd=REPO)
        .decode()
        .split("\0")
    )

    class OraclePath(type(root)):
        def is_file(self) -> bool:
            try:
                return self.relative_to(root).as_posix() in tracked
            except ValueError:
                return super().is_file()

    registry = subprocess.check_output(
        ["git", "show", f"{revision}:tests/qemu/cli/lints.py"], cwd=REPO
    )
    namespace = {
        "__name__": "cli.parity_registry",
        "__package__": "cli",
        "__file__": str(root / "tests/qemu/cli/lints.py"),
    }
    exec(compile(registry, namespace["__file__"], "exec"), namespace)
    return {
        name: (cmd, stream)
        for name, cmd, stream in namespace["source_lint_specs"](OraclePath(root))
    }


def compare(root: Path, revision: str, names: list[str]) -> dict:
    specs = oracle_specs(root, revision)
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
        cmd, stream = specs[name]
        if len(cmd) < 2 or not cmd[1].endswith(".py"):
            raise RuntimeError(f"{name}: oracle is not a standalone Python source command: {cmd}")
        script = Path(cmd[1]).relative_to(root)
        source = subprocess.run(
            ["git", "show", f"{revision}:{script.as_posix()}"],
            cwd=REPO,
            capture_output=True,
            check=True,
        ).stdout
        started = time.monotonic()
        # Imports must also come from the oracle revision: both a scanner and
        # its shared Python helper may already have been retired in this lane.
        replay = r"""
import importlib.abc
import importlib.util
import subprocess
import sys
from pathlib import Path

__file__, revision = sys.argv[1:3]
sys.argv = sys.argv[1:2] + sys.argv[3:]
repo = Path(__file__).parent.parent
sys.path.insert(0, str(repo / "tools"))
tracked = set(subprocess.check_output(
    ["git", "ls-tree", "-r", "--name-only", revision, "--", "tools"], cwd=repo
).decode().splitlines())

class OriginalTools(importlib.abc.MetaPathFinder, importlib.abc.Loader):
    def find_spec(self, fullname, path=None, target=None):
        if fullname.startswith("tools."):
            stem = fullname.replace(".", "/")
        elif "." not in fullname:
            stem = "tools/" + fullname
        else:
            return None
        for relative in (stem + ".py", stem + "/__init__.py"):
            if relative in tracked:
                spec = importlib.util.spec_from_loader(
                    fullname, self, is_package=relative.endswith("/__init__.py")
                )
                spec.loader_state = relative
                return spec
        return None

    def create_module(self, spec):
        return None

    def exec_module(self, module):
        relative = module.__spec__.loader_state
        module.__file__ = str(repo / relative)
        source = subprocess.check_output(["git", "show", revision + ":" + relative], cwd=repo)
        exec(compile(source, module.__file__, "exec"), module.__dict__)

sys.meta_path.insert(0, OriginalTools())
exec(compile(sys.stdin.read(), __file__, "exec"))
"""
        python = subprocess.run(
            [sys.executable, "-c", replay, cmd[1], revision, *cmd[2:]],
            cwd=root,
            input=source.decode(),
            env={**os.environ, "VIBEOS_LINT_CACHE": "0"},
            capture_output=True,
            text=True,
            check=False,
        )
        python_seconds = time.monotonic() - started
        if lints._scanner_error(python, stream):
            raise RuntimeError(f"{name}: Python oracle did not complete: {python.stderr}")
        output = python.stdout if stream == "stdout" else python.stdout + "\n" + python.stderr
        py_lines = lints._finding_lines(lints._rel(output, root))
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
    selection = parser.add_mutually_exclusive_group(required=True)
    selection.add_argument("--rules", help="comma-separated ported rule names")
    selection.add_argument("--all-ported", action="store_true")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    try:
        names = (
            args.rules.split(",")
            if args.rules
            else subprocess.check_output(
                [str(rustlints.ensure(REPO)), "--list-rules"], text=True
            ).splitlines()
        )
        report = compare(args.root.resolve(), args.oracle_revision, names)
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
