"""Provision the host linter and demultiplex its single shared invocation."""

from __future__ import annotations

import fcntl
import hashlib
import os
import subprocess
from collections.abc import Callable
from pathlib import Path

from tools.source_walk import source_files

# Only parity-proven plugins belong here. The original registry owns stage ordering.
PORTED = frozenset(
    {
        "scan_list_walk_from_head",
        "scan_dowhile_continue",
        "scan_variadic_array_arg",
        "scan_let_shadow",
        "scan_double_fetch",
        "scan_hresult_literals",
        "scan_init_then_move",
        "scan_read_after_clear",
        "scan_const_narrowing_cast",
        "scan_info_class_literals",
        "scan_misaligned_cast",
        "scan_fnptr_abi_transmute",
    }
)
SENTINEL = "VIBEOS_LINT"


def ensure(repo: Path) -> Path:
    source = repo / "tools/vibeos-lint"
    target = repo / ".cache/vibeos-lint/vibeos-lint"
    target.parent.mkdir(parents=True, exist_ok=True)
    files = [
        source / "Cargo.toml",
        source / "Cargo.lock",
        repo / "rust-toolchain.toml",
        *sorted(source_files(source / "src")),
    ]
    digest = hashlib.sha256()
    for path in files:
        digest.update(str(path.relative_to(repo)).encode() + b"\0" + path.read_bytes() + b"\0")
    key = digest.hexdigest()
    stamp = target.with_suffix(".sha256")
    with target.with_suffix(".lock").open("w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        if target.is_file() and stamp.is_file() and stamp.read_text() == key:
            return target
        env = {
            k: v
            for k, v in os.environ.items()
            if k
            not in {
                "RUSTUP_TOOLCHAIN",
                "RUSTC_WRAPPER",
                "CARGO_BUILD_RUSTC_WRAPPER",
                "CARGO_BUILD_TARGET",
                "RUSTFLAGS",
                "CARGO_ENCODED_RUSTFLAGS",
                "RUSTC_BOOTSTRAP",
            }
        }
        build = target.parent / "target"
        cmd = [
            "cargo",
            "build",
            "--offline",
            "--locked",
            "--release",
            "--manifest-path",
            str(source / "Cargo.toml"),
            "--target-dir",
            str(build),
        ]
        cp = subprocess.run(cmd, cwd=repo, env=env, capture_output=True, text=True)
        artifact = build / "release/vibeos-lint"
        if cp.returncode or not artifact.is_file():
            raise RuntimeError(f"vibeos-lint bootstrap failed: {cp.stderr}")
        temporary = target.with_suffix(f".{os.getpid()}.tmp")
        try:
            temporary.write_bytes(artifact.read_bytes())
            temporary.chmod(0o755)
            temporary.replace(target)
            stamp.write_text(key)
        finally:
            temporary.unlink(missing_ok=True)
    return target


def split_output(
    cp: subprocess.CompletedProcess, names: list[str]
) -> dict[str, subprocess.CompletedProcess]:
    """A shared failure invalidates all requested plugins, even with partial output."""
    lines: dict[str, list[str]] = {name: [] for name in names}
    error = ""
    if cp.returncode or any(
        line.startswith(("FATAL:", "Traceback (most recent call last):"))
        for line in cp.stderr.splitlines()
    ):
        error = cp.stderr or f"shared scanner exited {cp.returncode}"
    else:
        for line in cp.stdout.splitlines():
            name, tab, finding = line.partition("\t")
            if not tab or name not in lines or not finding:
                error = f"invalid tagged output: {line!r}"
                break
            lines[name].append(finding)
    return {
        name: subprocess.CompletedProcess(
            [SENTINEL, name],
            2 if error else 0,
            "" if error else "\n".join(findings),
            f"FATAL: vibeos-lint: {error}" if error else "",
        )
        for name, findings in lines.items()
    }


def run_many(
    repo: Path, names: list[str], invoke: Callable[[list[str]], subprocess.CompletedProcess]
) -> dict[str, subprocess.CompletedProcess]:
    try:
        runner = ensure(repo)
        cp = invoke([str(runner), "--root", str(repo), "--rules", ",".join(names)])
    except (OSError, RuntimeError, subprocess.SubprocessError) as error:
        cp = subprocess.CompletedProcess([], 2, "", f"FATAL: {error}")
    return split_output(cp, names)
