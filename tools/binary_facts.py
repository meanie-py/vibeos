"""Worktree-local, content-addressed facts from expensive binary extraction.

Create one context per scanner invocation. Only extraction results belong here;
apply current baselines and gate policy after each get(), including cache hits.
Failures and changing inputs never publish a record. Corrupt records fail loudly.
"""

from __future__ import annotations

import fcntl
import hashlib
import json
import os
import platform
import shutil
import sys
import tempfile
from collections.abc import Callable, Mapping, Sequence
from pathlib import Path
from typing import Any


class BinaryFactsError(RuntimeError):
    """The requested extraction could not be verified."""


def _stamp(path: Path) -> tuple[int, int, int, int, int]:
    stat = path.stat()
    return stat.st_dev, stat.st_ino, stat.st_size, stat.st_mtime_ns, stat.st_ctime_ns


def _digest(path: Path) -> tuple[str, tuple[int, int, int, int, int]]:
    before = _stamp(path)
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    if _stamp(path) != before:
        raise BinaryFactsError(f"binary cache input changed while hashing: {path}; retry the scan")
    return digest.hexdigest(), before


def _json(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


class BinaryFacts:
    """Fingerprint implementation/tools once; extract each binary once per key.

    tools are executable paths or names resolved through PATH. implementation must
    include the scanner and all helpers affecting its facts. options holds flags
    such as architecture. JSON normalization applies on both misses and hits.
    """

    def __init__(
        self,
        repo: Path,
        *,
        namespace: str,
        implementation: Sequence[Path],
        tools: Sequence[str | Path],
        options: Mapping[str, Any] | None = None,
    ) -> None:
        self.enabled = os.environ.get("VIBEOS_BINARY_CACHE") != "0"
        self.root = repo.resolve() / ".cache" / "binary-facts"
        self.dependencies: dict[Path, tuple[int, int, int, int, int]] = {}
        identities = []
        paths = [Path(__file__), *implementation, Path(sys.executable)]
        for tool in tools:
            executable = shutil.which(str(tool))
            if executable is None:
                raise BinaryFactsError(f"binary cache cannot resolve executable: {tool}")
            paths.append(Path(executable))
        self.bindings = {path.absolute(): path.resolve() for path in paths}
        for path in dict.fromkeys(self.bindings.values()):
            digest, stamp = _digest(path)
            self.dependencies[path] = stamp
            identities.append([str(path), digest])
        environment = {
            key: value
            for key, value in os.environ.items()
            if key.startswith(("LC_", "LD_", "DYLD_", "LLVM_"))
            or key in {"LANG", "LANGUAGE", "PATH", "SYSTEMROOT", "TZ"}
        }
        self.identity = hashlib.sha256(
            _json(
                {
                    "schema": 1,
                    "namespace": namespace,
                    "implementation_and_tools": identities,
                    "options": dict(options or {}),
                    "python": sys.version,
                    "platform": platform.platform(),
                    "environment": environment,
                }
            )
        ).hexdigest()

    def _verify_dependencies(self) -> None:
        for path, resolved in self.bindings.items():
            if path.resolve() != resolved:
                raise BinaryFactsError(f"binary cache tool symlink changed: {path}; retry the scan")
        for path, stamp in self.dependencies.items():
            if _stamp(path) != stamp:
                raise BinaryFactsError(
                    f"binary cache implementation/tool changed during scan: {path}; retry the scan"
                )

    def get(self, binary: Path, extract: Callable[[], Any]) -> Any:
        """Get JSON facts, raising if extraction fails or any input changes."""
        requested = binary.absolute()
        binary = binary.resolve()
        self._verify_dependencies()
        content, initial = _digest(binary)
        key = hashlib.sha256(
            _json([self.identity, str(requested), str(binary), content])
        ).hexdigest()
        if not self.enabled:
            value = json.loads(_json(extract()))
            self._verify_input(binary, initial, requested)
            return value
        self.root.mkdir(parents=True, exist_ok=True)
        record = self.root / f"{key}.json"
        with (self.root / f"{key}.lock").open("a+b") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
            self._verify_input(binary, initial, requested)
            if record.exists():
                try:
                    data = json.loads(record.read_bytes())
                    if (
                        not isinstance(data, dict)
                        or set(data) != {"key", "facts", "sha256"}
                        or data["key"] != key
                        or data["sha256"] != hashlib.sha256(_json(data["facts"])).hexdigest()
                    ):
                        raise ValueError("record key or checksum mismatch")
                    value = data["facts"]
                except (OSError, ValueError, TypeError) as error:
                    raise BinaryFactsError(
                        f"corrupt binary facts cache {record}: {error}; "
                        "remove this record and retry"
                    ) from error
                self._verify_input(binary, initial, requested)
                return value
            value = json.loads(_json(extract()))
            self._verify_input(binary, initial, requested)
            payload = _json(
                {
                    "key": key,
                    "facts": value,
                    "sha256": hashlib.sha256(_json(value)).hexdigest(),
                }
            )
            descriptor, temporary = tempfile.mkstemp(prefix=f".{key}-", dir=self.root)
            try:
                with os.fdopen(descriptor, "wb") as stream:
                    stream.write(payload)
                    stream.flush()
                    os.fsync(stream.fileno())
                self._verify_input(binary, initial, requested)
                os.replace(temporary, record)
            finally:
                Path(temporary).unlink(missing_ok=True)
            return value

    def _verify_input(
        self, binary: Path, initial: tuple[int, int, int, int, int], requested: Path
    ) -> None:
        self._verify_dependencies()
        if requested.resolve() != binary:
            raise BinaryFactsError(
                f"binary cache input symlink changed: {requested}; retry the scan"
            )
        if _stamp(binary) != initial:
            raise BinaryFactsError(
                f"binary cache input changed during scan: {binary}; retry the scan"
            )
