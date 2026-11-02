"""Transport for legacy Python importers of native build mechanisms."""

from __future__ import annotations

import atexit
import contextlib
import dataclasses
import inspect
import json
import os
import subprocess
import sys
import tempfile
import threading
import types
import typing
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parent.parent


class NativeError(ValueError):
    pass


def _json(value: Any) -> Any:
    if isinstance(value, bytes):
        return list(value)
    if isinstance(value, Path):
        return str(value)
    if dataclasses.is_dataclass(value) and not isinstance(value, type):
        return _json(dataclasses.asdict(value))
    if isinstance(value, dict):
        return {str(k): _json(v) for k, v in value.items()}
    if isinstance(value, (list, tuple, set, frozenset)):
        return [_json(v) for v in value]
    return value


class _ArtifactSession:
    def __init__(self, root):
        self.owner = os.getpid()
        self.signature = _source_signature()
        self.environment = dict(os.environ)
        self.resources = contextlib.ExitStack()
        # The stream outlives this method; session.close owns the ExitStack.
        self.errors = self.resources.enter_context(tempfile.TemporaryFile(mode="w+t"))  # noqa: SIM115
        self.process = subprocess.Popen(
            [str(REPO / "tools/build-infra"), "--root", str(root), "--api-jsonl"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=self.errors,
            text=True,
            bufsize=1,
            env=self.environment,
        )

    def close(self):
        process = self.process
        if process.stdin:
            process.stdin.close()
        if self.owner == os.getpid():
            try:
                process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                process.terminate()
                try:
                    process.wait(timeout=2)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
        if process.stdout:
            process.stdout.close()
        self.resources.close()

    def request(self, payload):
        process = self.process
        if process.stdin is None or process.stdout is None:
            raise NativeError("native artifact transport has no pipes")
        process.stdin.write(payload + "\n")
        process.stdin.flush()
        line = process.stdout.readline()
        if not line:
            self.errors.seek(0)
            raise NativeError("native artifact transport ended: " + self.errors.read().strip())
        if self.signature != _source_signature():
            raise NativeError(
                "native build sources changed during artifact request; retry after edits finish"
            )
        return line


def _source_signature():
    # Transport freshness only: the native tool owns descriptors and input policy.
    sources = REPO / "tools/vibeos-build"
    paths = [REPO / "tools/build-infra", Path(__file__)]
    if sources.is_dir():
        paths.extend([REPO / "rust-toolchain.toml", sources / "Cargo.toml", sources / "Cargo.lock"])
        paths.extend(p for p in (sources / "src").rglob("*") if p.is_file())
    records = []
    for path in sorted(paths):
        info = path.stat()
        records.append(
            (str(path), info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns, info.st_ctime_ns)
        )
    return tuple(records)


_sessions = {}
_session_lock = threading.RLock()


def _close_sessions():
    for session in list(_sessions.values()):
        try:
            session.close()
        except (OSError, ValueError):
            pass
    _sessions.clear()


def _after_fork():
    global _session_lock
    # Close inherited pipe copies, without terminating the parent's server.
    _close_sessions()
    _session_lock = threading.RLock()


atexit.register(_close_sessions)
if hasattr(os, "register_at_fork"):
    os.register_at_fork(after_in_child=_after_fork)


def _artifact_request(root, payload):
    with _session_lock:
        key = str(root)
        session = _sessions.get(key)
        if session and (
            session.signature != _source_signature() or session.environment != dict(os.environ)
        ):
            session.close()
            del _sessions[key]
            session = None
        if session is None:
            session = _ArtifactSession(root)
            _sessions[key] = session
        try:
            response = json.loads(session.request(payload))
            if not isinstance(response, dict) or set(response) not in ({"ok"}, {"error"}):
                raise NativeError("native artifact response must contain exactly ok or error")
            if "error" in response and (
                not isinstance(response["error"], dict)
                or not isinstance(response["error"].get("message"), str)
            ):
                raise NativeError("native artifact response has an invalid error envelope")
            return response
        except BaseException:
            # An interrupted read may leave a complete earlier response queued.
            # Reusing that stream could attribute it to the next request.
            _sessions.pop(key, None)
            try:
                session.close()
            except (OSError, ValueError):
                pass
            raise


def call(op: str, kwargs: dict, root: Path = REPO) -> Any:
    tool = REPO / "tools/build-infra"
    request = json.dumps({"op": op, "kwargs": _json(kwargs)}, ensure_ascii=False)
    if op.startswith("artifact_"):
        try:
            response = _artifact_request(root, request)
        except (OSError, ValueError) as exc:
            if isinstance(exc, NativeError):
                raise
            raise NativeError(f"native build {op}: invalid response: {exc}") from exc
        if not isinstance(response, dict):
            raise NativeError(f"native build {op}: response must be an object")
        if "error" in response:
            raise NativeError(response["error"]["message"])
        if "ok" not in response:
            raise NativeError(f"native build {op}: response lacks result")
        return response["ok"]
    try:
        proc = subprocess.run(
            [str(tool), "--root", str(root), "--api-json"],
            input=request,
            text=True,
            capture_output=True,
            check=False,
        )
    except OSError as exc:
        raise NativeError(f"cannot launch native build tool: {exc}") from exc
    try:
        response = json.loads(proc.stdout)
    except ValueError as exc:
        raise NativeError(
            f"native build {op}: invalid response (exit {proc.returncode}): {proc.stderr.strip()}"
        ) from exc
    if not isinstance(response, dict):
        raise NativeError(f"native build {op}: response must be an object")
    if "error" in response:
        raise NativeError(response["error"]["message"])
    if proc.returncode or "ok" not in response:
        raise NativeError(f"native build {op}: exit {proc.returncode}: {proc.stderr.strip()}")
    return response["ok"]


def main(command: str, argv: list[str] | None = None) -> int:
    return subprocess.call(
        [str(REPO / "tools/build-infra"), command, *(sys.argv[1:] if argv is None else argv)]
    )


def coerce(value, annotation) -> Any:
    if value is None or annotation is inspect.Signature.empty or annotation is typing.Any:
        return value
    origin = typing.get_origin(annotation)
    params = typing.get_args(annotation)
    if origin in (typing.Union, types.UnionType):
        choices = [x for x in params if x is not type(None)]
        return coerce(value, choices[0]) if len(choices) == 1 else value
    if annotation is Path:
        return Path(value)
    if isinstance(annotation, type) and dataclasses.is_dataclass(annotation):
        hints = typing.get_type_hints(annotation)
        return annotation(**{k: coerce(v, hints.get(k, typing.Any)) for k, v in value.items()})
    if origin in (list, set, frozenset):
        return origin(coerce(v, params[0]) for v in value) if params else origin(value)
    if origin is tuple:
        if not params:
            return tuple(value)
        if len(params) == 2 and params[1] is Ellipsis:
            return tuple(coerce(v, params[0]) for v in value)
        return tuple(coerce(v, t) for v, t in zip(value, params, strict=True))
    if origin is dict:
        return (
            {coerce(k, params[0]): coerce(v, params[1]) for k, v in value.items()}
            if params
            else value
        )
    return value


def invoke(
    op: str, args: dict, root: Path | None = None, *, error_type: type[NativeError] = NativeError
) -> Any:
    try:
        return call(op, args, root=Path(root) if root is not None else REPO)
    except NativeError as exc:
        if error_type is NativeError:
            raise
        raise error_type(str(exc)) from exc


def implementation_identity() -> dict:
    """Native policy identity; source changes deliberately invalidate old artifact actions."""
    result = call("artifact_implementation_identity", {})
    result.pop("executable")
    return result


def implementation_watches() -> list[str]:
    """Watch the executing binary and source inputs available in source distributions."""
    result = call("artifact_implementation_identity", {})
    paths = [Path(result["executable"]), Path(__file__)]
    paths.extend(REPO / name for name in result["sources"] if (REPO / name).is_file())
    return [str(path) for path in paths]
