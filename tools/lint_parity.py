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
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO / "tests/qemu"))
sys.path.insert(0, str(REPO))
from cli import lints, rustlints  # noqa: E402

_ORACLE_REPLAY = r"""
import importlib.abc
import importlib.util
import os
import subprocess
import sys
import tempfile
from pathlib import Path

__file__, revision, repo_arg = sys.argv[1:4]
sys.argv = sys.argv[1:2] + sys.argv[4:]
repo = Path(repo_arg)
code_repo = Path(os.environ.get("VIBEOS_LINT_ORACLE_GIT", repo_arg))
sys.path.insert(0, str(repo / "tools"))
tracked = set(subprocess.check_output(
    ["git", "ls-tree", "-r", "--name-only", revision, "--", "tools"], cwd=code_repo
).decode().splitlines())

# BinaryFacts fingerprints its implementation files even with caching disabled.
# Keep those fingerprints bound to the oracle Git bytes after scanner retirement;
# detector __file__ and source/build paths retain the selected corpus root.
cache_sources = tempfile.TemporaryDirectory(prefix="vibeos-lint-oracle-code-")
def snapshot(relative):
    dest = Path(cache_sources.name) / relative
    if not dest.exists():
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_bytes(subprocess.check_output(
            ["git", "show", revision + ":" + relative], cwd=code_repo))
    return dest

class OriginalTools(importlib.abc.MetaPathFinder, importlib.abc.Loader):
    def find_spec(self, fullname, path=None, target=None):
        if fullname.startswith("tools."):
            stem = fullname.replace(".", "/")
        elif fullname == "rustconv" or fullname.startswith("rustconv."):
            stem = "tools/rustconv/" + fullname.replace(".", "/")
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
        source = subprocess.check_output(["git", "show", revision + ":" + relative], cwd=code_repo)
        exec(compile(source, module.__file__, "exec"), module.__dict__)
        if relative == "tools/binary_facts.py":
            # Preserve original path identities while hashing immutable oracle bytes.
            # The original cache still verifies real tool/binary contents and records.
            original_stamp, original_digest = module._stamp, module._digest
            def implementation_path(path):
                try:
                    rel = path.relative_to(repo).as_posix()
                except ValueError:
                    return path
                return snapshot(rel) if rel in tracked and rel.endswith(".py") else path
            module._stamp = lambda path: original_stamp(implementation_path(path))
            module._digest = lambda path: original_digest(implementation_path(path))

sys.meta_path.insert(0, OriginalTools())
try:
    exec(compile(sys.stdin.read(), __file__, "exec"))
finally:
    cache_sources.cleanup()
"""


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
    helper = "tests/qemu/cli/rustlints.py"
    if helper in tracked and "rustlints" in namespace:
        # The registry's ported set must also belong to its historical revision.
        # Otherwise retiring a plugin today rewrites yesterday's oracle command.
        from types import ModuleType

        original_runner = ModuleType("cli.parity_rustlints")
        original_runner.__file__ = str(root / helper)
        original_runner.__package__ = "cli"
        helper_source = subprocess.check_output(["git", "show", f"{revision}:{helper}"], cwd=REPO)
        exec(compile(helper_source, original_runner.__file__, "exec"), original_runner.__dict__)
        namespace["rustlints"] = original_runner
    return {
        name: (cmd, stream)
        for name, cmd, stream in namespace["source_lint_specs"](OraclePath(root))
    }


def compare(root: Path, revision: str, names: list[str]) -> dict:
    specs = oracle_specs(root, revision)
    for name in names:
        if name not in specs:
            raise RuntimeError(f"{name}: absent from the historical registry")
        cmd, _stream = specs[name]
        if name not in {"shellcheck", "py-syntax"} and (len(cmd) < 2 or not cmd[1].endswith(".py")):
            raise RuntimeError(f"{name}: oracle is not a standalone Python source command: {cmd}")
    runner = rustlints.ensure(REPO)
    started = time.monotonic()
    rust_cmd = [str(runner), "--root", str(root), "--rules", ",".join(names)]
    if "py-syntax" in names:
        rust_cmd.extend(["--python", sys.executable])
    if "shellcheck" in names:
        shellcheck = lints._shellcheck_bin()
        if shellcheck is None:
            raise RuntimeError("shellcheck is unavailable")
        rust_cmd.extend(["--shellcheck", shellcheck])
    cp = subprocess.run(
        rust_cmd,
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
        "rust_binary_sha256": hashlib.sha256(runner.read_bytes()).hexdigest(),
        "rules": {},
    }

    def oracle_result(name: str) -> tuple[str, dict]:
        rust = results[name]
        if lints._scanner_error(rust, "stdout"):
            raise RuntimeError(rust.stderr)
        cmd, stream = specs[name]
        if name in {"shellcheck", "py-syntax"}:
            started = time.monotonic()
            python = subprocess.run(cmd, cwd=root, capture_output=True, text=True, check=False)
            return name, result_for(python, stream, rust, time.monotonic() - started)
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
        replay = _ORACLE_REPLAY

        python = subprocess.run(
            [sys.executable, "-c", replay, cmd[1], revision, str(root), *cmd[2:]],
            cwd=root,
            input=source.decode(),
            env={
                **os.environ,
                "VIBEOS_LINT_CACHE": "0",
                "VIBEOS_LINT_ORACLE_GIT": str(REPO),
            },
            capture_output=True,
            text=True,
            check=False,
        )
        return name, result_for(python, stream, rust, time.monotonic() - started)

    def result_for(
        python: subprocess.CompletedProcess,
        stream: str,
        rust: subprocess.CompletedProcess,
        python_seconds: float,
    ) -> dict:
        if lints._scanner_error(python, stream):
            raise RuntimeError(f"Python oracle did not complete: {python.stderr}")
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
        return result

    with ThreadPoolExecutor(max_workers=min(lints.lint_jobs(), len(names))) as pool:
        report["rules"] = dict(pool.map(oracle_result, names))
    return report


def compare_binary(
    root: Path, revision: str, names: list[str], arch: str, inputs: list[Path] | None = None
) -> dict:
    """Compare gate outputs/exit codes using the exact same built artifacts.

    This checks scanner parity, not artifact freshness or boot health.
    """
    supported = {
        *rustlints.binary_names(arch),
        "committed_drift",
        "verify_cohab_coverage",
        "scan_i386_sse_abi",
        "lint_wide_strings",
        "scan_captures",
        "scan_ir_mismatch",
        "verify_arm64_unwind",
    }
    if not names or not set(names) <= supported:
        raise RuntimeError(f"unsupported binary rules for {arch}: {set(names) - supported}")
    runner = rustlints.ensure(REPO)
    started = time.monotonic()
    command = [
        str(runner),
        "--root",
        str(root),
        "--binary-rules",
        ",".join(names),
        "--arch",
        arch,
        "--python",
        sys.executable,
    ]
    for path in inputs or []:
        command += ["--path", str(path)]
    print("parity: running native checks", file=sys.stderr, flush=True)
    native = rustlints.split_binary_output(
        subprocess.run(command, cwd=root, capture_output=True, text=True), names
    )
    report = {
        "root": str(root),
        "oracle_revision": revision,
        "arch": arch,
        "artifact_basis": "same existing build tree; freshness not asserted",
        "rust_binary_sha256": hashlib.sha256(runner.read_bytes()).hexdigest(),
        "rust_seconds": time.monotonic() - started,
        "rules": {},
    }
    build = root / f"build-{arch}"
    for name in names:
        if native[name].returncode >= 2:
            raise RuntimeError(f"native {name} did not complete: {native[name].stderr}")
        script = (
            "tools/check_committed_drift.py"
            if name == "committed_drift"
            else "tools/check_freeldr_size.py"
            if name == "freeldr_size"
            else "tools/rust_warning_census.py"
            if name == "rust_ub_deny"
            else "tools/verify_sacred_primitives.sh"
            if name == "sacred_primitives"
            else f"tools/{name}.py"
        )
        source = subprocess.check_output(["git", "show", f"{revision}:{script}"], cwd=REPO)
        if name == "scan_captures":
            args = [str(p) for p in inputs or []] + ["--lint"]
        elif name == "scan_ir_mismatch":
            args = [str(p) for p in inputs or []] + ["--check"]
        elif name == "verify_cohab_coverage":
            args = ["--arch", arch, "--check"]
        elif name == "scan_i386_sse_abi":
            args = []
        elif name == "lint_wide_strings":
            args = ["--check", *[str(p) for p in (inputs or [root / "sdk/lib/rust"])]]
        elif name == "committed_drift":
            args = ["--arch", arch]
        elif name == "rust_ub_deny":
            args = [
                str(build),
                "--allow-stale",
                "--deny",
                "invalid_value,deref_nullptr,static_mut_refs,improper_ctypes_definitions,improper_ctypes",
            ]
        elif name == "scan_flex_array_panic":
            args = ["--lint", "--arch", arch]
        else:
            args = ["--check", str(build)]
        started = time.monotonic()
        if name == "sacred_primitives":
            with tempfile.TemporaryDirectory(prefix="vibeos-lint-oracle-") as directory:
                base = Path(directory)
                (base / "tools").mkdir()
                (base / ".build-tools").symlink_to(root / ".build-tools")
                tool = base / script
                tool.write_bytes(source)
                old = subprocess.run(
                    ["bash", str(tool), str(build)], cwd=root, capture_output=True, text=True
                )
                old.stdout = old.stdout.replace(str(base), str(root))
                old.stderr = old.stderr.replace(str(base), str(root))
        else:
            old = subprocess.run(
                [
                    sys.executable,
                    "-c",
                    _ORACLE_REPLAY,
                    str(root / script),
                    revision,
                    str(root),
                    *args,
                ],
                input=source.decode(),
                cwd=root,
                env={
                    **os.environ,
                    "VIBEOS_LINT_CACHE": "0",
                    "VIBEOS_LINT_ORACLE_GIT": str(REPO),
                },
                capture_output=True,
                text=True,
            )
        if old.returncode >= 2 or lints._scanner_error(old, "both"):
            raise RuntimeError(f"oracle {name} did not complete: {old.stderr}")
        print(
            f"parity: oracle {name} completed in {time.monotonic() - started:.2f}s",
            file=sys.stderr,
            flush=True,
        )
        rs = native[name]
        old_text = lints._rel(old.stdout + "\n" + old.stderr, root)
        rs_text = lints._rel(rs.stdout + "\n" + rs.stderr, root)
        old_lines, rs_lines = old_text.splitlines(), rs_text.splitlines()
        report["rules"][name] = {
            "python": len(old_lines),
            "rust": len(rs_lines),
            "python_exit": old.returncode,
            "rust_exit": rs.returncode,
            "python_seconds": time.monotonic() - started,
            "python_only": sorted(set(old_lines) - set(rs_lines)),
            "rust_only": sorted(set(rs_lines) - set(old_lines)),
            "python_sha256": hashlib.sha256(old_text.encode()).hexdigest(),
            "rust_sha256": hashlib.sha256(rs_text.encode()).hexdigest(),
            "exact": old_text == rs_text and old.returncode == rs.returncode,
        }
        if name == "scan_ir_mismatch":
            # Python iterates a set of call signatures; its line order varies by hash seed.
            report["rules"][name]["comparison"] = (
                "line multiset and exit code; oracle signature-set order is unspecified"
            )
            report["rules"][name]["equivalent"] = (
                sorted(old_lines) == sorted(rs_lines) and old.returncode == rs.returncode
            )
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=REPO)
    parser.add_argument("--oracle-revision", default="34d3d7e1941")
    selection = parser.add_mutually_exclusive_group(required=True)
    selection.add_argument("--rules", help="comma-separated ported rule names")
    selection.add_argument("--all-ported", action="store_true")
    selection.add_argument("--binary-rules", help="comma-separated post-link rule names")
    parser.add_argument("--arch", choices=("i386", "amd64", "arm64"))
    parser.add_argument("--path", action="append", type=Path, default=[])
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    try:
        names = (
            args.rules.split(",")
            if args.rules
            else []
            if args.binary_rules
            else subprocess.check_output(
                [str(rustlints.ensure(REPO)), "--list-rules"], text=True
            ).splitlines()
        )
        if args.binary_rules:
            if not args.arch:
                parser.error("--binary-rules requires --arch")
            report = compare_binary(
                args.root.resolve(),
                args.oracle_revision,
                args.binary_rules.split(","),
                args.arch,
                args.path,
            )
        else:
            report = compare(args.root.resolve(), args.oracle_revision, names)
    except (OSError, RuntimeError, subprocess.SubprocessError) as error:
        print(f"FATAL: {error}", file=sys.stderr)
        return 2
    rendered = json.dumps(report, indent=2) + "\n"
    print(rendered, end="")
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered)
    return int(
        any(
            r["python_only"] or r["rust_only"] or not r.get("equivalent", r.get("exact", True))
            for r in report["rules"].values()
        )
    )


if __name__ == "__main__":
    sys.exit(main())
