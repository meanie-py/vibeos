"""Compatibility DTOs for the native content-receipt store.

Python supplies interpreter/package discovery facts. Rust fingerprints the bytes,
checks declared inputs and owns locks, reuse, validation and atomic publication.
"""

from __future__ import annotations

import importlib.metadata
import os
import platform
import shutil
import subprocess
import sys
from pathlib import Path

from tools import build_infra_compat as native

_MAX_AGE = 24 * 3600


def _call(op, **kwargs):
    try:
        return native.call("tooling_receipts_" + op, kwargs)
    except native.NativeError as error:
        raise RuntimeError(str(error)) from error


_POLICY = _call("policy")
GENERATED_INPUT_CHECKS = frozenset(_POLICY["GENERATED_INPUT_CHECKS"])


def eligibility(label):
    return _call("eligible", label=label)


def _file(path):
    return _call("file", path=path)


def _tree(path):
    return _call("tree", path=path)


def source_identity(repo):
    return _call("source", repo=repo)


def generated_identity(repo):
    return _call("generated", repo=repo)


def runtime_identity(repo):
    # These are discovery facts provided by the interpreter being tested; all
    # content hashing, environment filtering and normalization is native.
    files = []
    trees = []
    facts = {
        "python": sys.version,
        "packages": [],
        "system": [
            platform.platform(),
            platform.machine(),
            os.cpu_count(),
            sorted(os.sched_getaffinity(0)),
        ],
    }
    for name in [
        *_POLICY["_TOOLS"],
        str(Path(sys.executable).parent / "ruff"),
        str(Path(sys.executable).parent / "pyright"),
    ]:
        files.append([name, shutil.which(name)])
    rustup = shutil.which("rustup")
    if rustup:
        selected = {}
        for name in ("rustc", "cargo"):
            proc = subprocess.run(
                [rustup, "which", name], cwd=repo, capture_output=True, text=True, check=False
            )
            if proc.returncode == 0:
                selected[name] = Path(proc.stdout.strip())
                files.append(["selected:" + name, str(selected[name])])
            else:
                facts["selected:" + name] = proc.stderr
        if "rustc" in selected:
            sysroot = selected["rustc"].parent.parent
            for p in sorted((sysroot / "lib").glob("*.so*")):
                files.append(["rustc-shared:" + str(p.relative_to(sysroot)), str(p)])
            output = subprocess.check_output([str(selected["rustc"]), "-vV"], text=True)
            host = next(
                s.removeprefix("host: ") for s in output.splitlines() if s.startswith("host: ")
            )
            trees.append(["rustc-host-sysroot", str(sysroot / "lib/rustlib" / host / "lib")])
    packages = []
    for dist in importlib.metadata.distributions():
        name = dist.metadata["Name"]
        version = dist.version
        packages.append([name, version])
        for entry in sorted(dist.files or []):
            if not str(entry).endswith((".pyc", ".pyo")):
                files.append([f"package:{name}:{version}:{entry}", str(dist.locate_file(entry))])
    facts["packages"] = sorted(packages)
    files.append(["python", sys.executable])
    cargo_home = Path(os.environ.get("CARGO_HOME", str(Path.home() / ".cargo")))
    for name in ("config", "config.toml"):
        files.append([str(cargo_home / name), str(cargo_home / name)])
    return _call("runtime", repo=repo, env=dict(os.environ), files=files, trees=trees, facts=facts)


class ReceiptSession:
    def __init__(self, repo):
        self.repo = repo.resolve()
        self.closed = False
        self.rerun = set()
        self._session = native.DedicatedSession("--api-jsonl")
        try:
            result = self._request("begin", repo=self.repo, identity=self._identity())
            self.root = Path(result["root"])
            self.key = result["key"]
        except BaseException:
            self.close()
            raise

    def _request(self, op, **kwargs):
        try:
            return self._session.call("tooling_receipts_" + op, kwargs)
        except native.NativeError as error:
            raise RuntimeError(str(error)) from error

    def _identity(self):
        return [source_identity(self.repo), runtime_identity(self.repo)]

    @property
    def hits(self):
        return self._final_hits if hasattr(self, "_final_hits") else set(self._request("hits"))

    def run(self, label, cmd, compute, *, cwd=None, env_pp=None):
        result = self._request("lookup", label=label, cmd=cmd, cwd=cwd or self.repo, env_pp=env_pp)
        if "result" in result:
            return tuple(result["result"])
        value = compute()
        if result["eligible"]:
            self._request("put", key=result["key"], result=value)
        return value

    def stale_generated_hits(self):
        return set(self._request("stale"))

    def mark_rerun(self, label):
        self.rerun.add(label)
        self._request("rerun", label=label)

    def finish(self):
        try:
            self._final_hits = set(self._request("finish", identity=self._identity())["hits"])
        finally:
            self.close()

    def close(self):
        if not self.closed:
            self.closed = True
            self._session.close()

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.close()
