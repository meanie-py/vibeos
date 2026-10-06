"""Content receipts for tooling checks with declared source/generated/tool inputs.

Receipts contain successful output only. The whole session holds a lock for its
input identity, and publishes only after a second content fingerprint. Generated
headers belong to this checkout; neither cache lookup nor publication reads a
sibling's mutable build artifacts.
"""

from __future__ import annotations

import fcntl
import hashlib
import importlib.metadata
import json
import os
import platform
import shutil
import stat
import subprocess
import sys
import threading
import time
from collections.abc import Callable
from pathlib import Path

Result = tuple[str, bool, str]
_VERSION = 1
_MAX_AGE = 24 * 3600
# These exercise real runtime admission/process behavior or live object files.
_LIVE = {
    "pytest:tests/qemu/cli/test_portlease.py",
    "pytest:tools/test_host_jobs.py",
    "pytest:tools/test_verify_ucrtbase_cohab.py",
    "pytest:tools/test_tooling_receipts.py",
    # These compile temporary projects using mutable external Cargo registry/sysroot inputs.
    "pytest:tools/test_build_invalidation.py",
    "pytest:tools/test_cargo_link_context.py",
    "pytest:tools/test_nested_cargo_deps.py",
    "pytest:tools/test_bindgen_generation.py",
}
# Audited source/fixture suites. Adding a suite to the runner does not establish
# its input contract: new suites run live until explicitly reviewed here.
_AUDITED_PYTEST = {
    "pytest:tests/qemu/cli",
    "pytest:tools/fex/test_wow64_closure.py",
    "pytest:tools/rustconv/tests",
    "pytest:tools/test_cargo_depgraph.py",
    "pytest:tools/test_conformance_probe.py",
    "pytest:tools/test_export_census.py",
    "pytest:tools/test_gate_source_partitions.py",
    "pytest:tools/test_gen_ctype_tables.py",
    "pytest:tools/test_hostdisk.py",
    "pytest:tools/test_kernel_probe.py",
    "pytest:tools/test_link_guard.py",
    "pytest:tools/test_link_pulls.py",
    "pytest:tools/test_ninja_closure_prune.py",
    "pytest:tools/test_lint_facts.py",
    "pytest:tools/test_migrate_debug_assert.py",
    "pytest:tools/test_retarget_cargo_depfile.py",
    "pytest:tools/test_rust_module_tree.py",
    "pytest:tools/test_scan_arg_side_effects.py",
    "pytest:tools/test_scan_dropped_asserts.py",
    "pytest:tools/test_scan_flex_array_panic.py",
    "pytest:tools/test_scan_import_cc.py",
    "pytest:tools/test_scan_invented_constants.py",
    "pytest:tools/test_scan_octal_literals.py",
    "pytest:tools/test_scan_raw_uart_writes.py",
    "pytest:tools/test_scan_res_named_types.py",
    "pytest:tools/test_scan_bindgen_padding_index.py",
    "pytest:tools/test_scan_extern_static_width.py",
    "pytest:tools/test_ramdisk_tree.py",
    "pytest:tools/test_scan_const_vs_island.py",
    "pytest:tools/test_cohab_census.py",
    "pytest:tools/test_scan_orphan_modules.py",
    "pytest:tools/test_scan_self_recursive.py",
    "pytest:tools/test_scan_export_thunks.py",
    "pytest:tools/test_scan_wide_literals.py",
    "pytest:tools/test_scanner_controls.py",
    "pytest:tools/test_source_walk.py",
    "pytest:tools/test_vendored.py",
    "pytest:tools/test_verify_c_signatures.py",
    "pytest:tools/test_verify_cohab_coverage.py",
    "pytest:tools/test_verify_lost_exports.py",
    "pytest:tools/test_verify_translation_literals.py",
    "pytest:tools/wow64gen/test_wow64gen.py",
}
_TOOLS = ("git", "rustc", "cargo", "cmake", "ninja", "cc", "c++", "clang", "ar", "objdump", "node")
_VOLATILE_ENV = {
    "PWD",
    "OLDPWD",
    "SHLVL",
    "_",
    "VIBEOS_JOB_OWNER_PID",
    "VIBEOS_JOB_RUN",
    "VIBEOS_JOB_LAUNCHER_PID",
    "VIBEOS_JOB_CLASS",
    "VIBEOS_JOB_REPO",
    "VIBEOS_ARTIFACT_ROOT",
}


def eligibility(label: str) -> bool:
    """Only explicitly audited input contracts are eligible; new suites run live."""
    if label in _LIVE:
        return False
    if label.startswith(("ruff:", "fmt:", "pyright:")) or label == "generated:ddk_enum_constants":
        return True
    return label in _AUDITED_PYTEST


def _digest(value: object) -> str:
    return hashlib.sha256(json.dumps(value, sort_keys=True).encode()).hexdigest()


def _file(path: Path) -> tuple[int, str]:
    try:
        mode = path.lstat().st_mode
        if stat.S_ISLNK(mode):
            resolved = path.resolve()
            if resolved.is_dir():
                raise RuntimeError(
                    f"Tooling receipt input {path} is a directory symlink. "
                    "Use a real checkout-local input directory before caching tooling checks."
                )
            return mode, _digest([os.readlink(path), _file(resolved)])
        if not stat.S_ISREG(mode):
            return mode, "directory" if path.is_dir() else "special"
        digest = hashlib.sha256()
        with path.open("rb") as stream:
            for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                digest.update(chunk)
        return mode, digest.hexdigest()
    except FileNotFoundError:
        return 0, "missing"


def _tree(root: Path) -> list[tuple[str, tuple[int, str]]]:
    """Walk one explicitly declared generated/package tree, never build siblings."""
    for parent in root.parents:
        if parent.is_symlink():
            _file(parent)  # A symlinked build root must not expose sibling headers either.
    result = [(".", _file(root))]
    if result[0][1][1] == "missing":
        return result

    def fail(error: OSError) -> None:
        raise error

    for directory, dirs, files in os.walk(root, onerror=fail):
        for name in dirs:
            path = Path(directory) / name
            if path.is_symlink():
                _file(path)  # Reject a directory symlink before os.walk silently skips it.
        dirs[:] = sorted(d for d in dirs if d != "__pycache__")
        for name in sorted(files):
            if name.endswith((".pyc", ".pyo")):
                continue
            path = Path(directory) / name
            result.append((str(path.relative_to(root)), _file(path)))
    return result


def source_identity(repo: Path) -> str:
    tracked = subprocess.check_output(["git", "ls-files", "-z"], cwd=repo).split(b"\0")
    untracked = subprocess.check_output(
        ["git", "ls-files", "--others", "--exclude-standard", "-z"], cwd=repo
    ).split(b"\0")
    names = {os.fsdecode(name) for name in tracked if name}
    # User handoff notes do not affect checks; untracked executable/source/config
    # inputs do, even before git add. Tracked documentation is still included.
    names.update(os.fsdecode(name) for name in untracked if name and not name.endswith(b".md"))
    return _digest([(name, _file(repo / name)) for name in sorted(names)])


def runtime_identity(repo: Path) -> str:
    def normalize(value: str) -> str:
        return value.replace(str(repo), "<repo>")

    env = {key: normalize(value) for key, value in os.environ.items() if key not in _VOLATILE_ENV}
    tools = []
    for name in (
        *_TOOLS,
        str(Path(sys.executable).parent / "ruff"),
        str(Path(sys.executable).parent / "pyright"),
    ):
        found = shutil.which(name)
        tools.append((name, _file(Path(found)) if found else None))
    # rustup proxies do not identify the compiler; include the selected binaries.
    rustup = shutil.which("rustup")
    if rustup:
        for name in ("rustc", "cargo"):
            proc = subprocess.run([rustup, "which", name], cwd=repo, capture_output=True, text=True)
            tools.append(
                (
                    f"selected:{name}",
                    _file(Path(proc.stdout.strip())) if proc.returncode == 0 else proc.stderr,
                )
            )
    if rustup:
        proc = subprocess.run([rustup, "which", "rustc"], cwd=repo, capture_output=True, text=True)
        if proc.returncode == 0:
            sysroot = Path(proc.stdout.strip()).parent.parent
            # rustc itself is a small loader; its driver and host std libraries
            # are executable test inputs too. Exclude unrelated cross targets.
            tools.append(
                (
                    "rustc-shared-libraries",
                    [
                        (str(path.relative_to(sysroot)), _file(path))
                        for path in sorted((sysroot / "lib").glob("*.so*"))
                    ],
                )
            )
            target = subprocess.check_output([str(sysroot / "bin/rustc"), "-vV"], text=True)
            host = next(
                line.removeprefix("host: ")
                for line in target.splitlines()
                if line.startswith("host: ")
            )
            tools.append(("rustc-host-sysroot", _tree(sysroot / "lib/rustlib" / host / "lib")))
    packages = []
    for dist in importlib.metadata.distributions():
        files = []
        for entry in dist.files or []:
            # RECORD alone cannot detect an in-place edit of an installed module.
            if str(entry).endswith((".pyc", ".pyo")):
                continue
            files.append((str(entry), _file(Path(str(dist.locate_file(entry))))))
        packages.append((dist.metadata["Name"], dist.version, sorted(files)))
    cargo_home = Path(os.environ.get("CARGO_HOME", str(Path.home() / ".cargo")))
    configs = [
        (str(cargo_home / name), _file(cargo_home / name)) for name in ("config", "config.toml")
    ]
    return _digest(
        {
            "environment": env,
            "python": [sys.version, _file(Path(sys.executable))],
            "packages": sorted(packages),
            "tools": tools,
            "configs": configs,
            "system": [
                platform.platform(),
                platform.machine(),
                os.cpu_count(),
                sorted(os.sched_getaffinity(0)),
            ],
        }
    )


def generated_identity(repo: Path) -> str:
    # cpp_com resolves generated WIDL interfaces and their include closure. Hash
    # complete include trees, including absence, rather than guess leaf headers.
    return _digest(
        [
            (arch, _tree(repo / f"build-{arch}" / "sdk" / "include"))
            for arch in ("i386", "amd64", "arm64")
        ]
    )


class ReceiptSession:
    """One check-tooling invocation; call finish even when a check fails.

    Use as a context manager so exceptions release the lock without publishing.
    ``run`` is thread-safe and preserves the existing tooling result tuple.
    """

    def __init__(self, repo: Path):
        self.repo = repo.resolve()
        common = subprocess.check_output(
            ["git", "rev-parse", "--git-common-dir"], cwd=repo, text=True
        ).strip()
        self.root = (repo / common).resolve() / "vibeos-tooling-receipts"
        self.root.mkdir(parents=True, exist_ok=True)
        self.before = self._identity()
        self.generated_before = generated_identity(self.repo)
        self.key = _digest([_VERSION, self.before])
        self.lock = (self.root / f"{self.key}.lock").open("a")
        fcntl.flock(self.lock, fcntl.LOCK_EX)
        self.pending: dict[str, Result] = {}
        self.generated_keys: set[str] = set()
        self.hits: set[str] = set()
        self.guard = threading.Lock()
        self.closed = False

    def _identity(self) -> tuple[str, str]:
        return source_identity(self.repo), runtime_identity(self.repo)

    def run(
        self,
        label: str,
        cmd: list[str],
        compute: Callable[[], Result],
        *,
        cwd: Path | None = None,
        env_pp: str | None = None,
    ) -> Result:
        if not eligibility(label):
            return compute()

        def normalize(value: object) -> str:
            return str(value).replace(str(self.repo), "<repo>")

        generated = self.generated_before if label == "pytest:tools/rustconv/tests" else None
        key = _digest(
            [
                self.key,
                label,
                [normalize(arg) for arg in cmd],
                normalize(cwd or self.repo),
                env_pp,
                generated,
            ]
        )
        if generated is not None:
            with self.guard:
                self.generated_keys.add(key)
        path = self.root / f"{key}.json"
        if path.exists():
            try:
                data = json.loads(path.read_text())
                if (
                    data["version"] != _VERSION
                    or data["key"] != key
                    or data["label"] != label
                    or data["ok"] is not True
                    or not isinstance(data["output"], str)
                    or not isinstance(data["created"], (int, float))
                ):
                    raise ValueError("invalid receipt fields")
            except (ValueError, KeyError, TypeError) as error:
                raise RuntimeError(f"Corrupt tooling receipt {path}: {error}") from error
            if 0 <= time.time() - data["created"] < _MAX_AGE:
                with self.guard:
                    self.hits.add(label)
                return label, True, data["output"]
        result = compute()
        if result[1]:
            with self.guard:
                self.pending[key] = result
        return result

    def finish(self) -> None:
        try:
            if self._identity() != self.before:
                # Name the usual cause, because the message alone reads as a
                # mysterious infrastructure failure and is not one. It fires
                # ~always for the same reason: the checks were started in the
                # BACKGROUND and their caller kept editing (or committing) while
                # they ran. Three aborted land submissions in one session, same
                # cause each time, before anyone wrote this sentence down.
                raise RuntimeError(
                    "Tooling inputs changed during checks; no receipts published. "
                    "Rerun check-tooling. Usual cause: files were edited or "
                    "committed WHILE these checks ran — run `land`/`check-tooling` "
                    "in the foreground and make no edits until it returns."
                )
            generated_changed = generated_identity(self.repo) != self.generated_before
            if generated_changed and "pytest:tools/rustconv/tests" in self.hits:
                raise RuntimeError(
                    "Generated tooling inputs changed after receipt reuse; rerun check-tooling."
                )
            for key, (label, ok, output) in self.pending.items():
                if generated_changed and key in self.generated_keys:
                    continue
                path = self.root / f"{key}.json"
                temp = path.with_suffix(f".{os.getpid()}.tmp")
                temp.write_text(
                    json.dumps(
                        {
                            "version": _VERSION,
                            "key": key,
                            "label": label,
                            "ok": ok,
                            "output": output,
                            "created": time.time(),
                        }
                    )
                )
                os.replace(temp, path)
        finally:
            self.close()

    def close(self) -> None:
        if not self.closed:
            self.lock.close()
            self.closed = True

    def __enter__(self) -> ReceiptSession:
        return self

    def __exit__(self, exc_type, exc_value, traceback) -> None:
        self.close()
