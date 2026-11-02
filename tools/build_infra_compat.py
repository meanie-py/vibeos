"""Transport for legacy Python importers of native build mechanisms."""

from __future__ import annotations

import dataclasses
import inspect
import json
import subprocess
import sys
import types
import typing
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parent.parent


class NativeError(ValueError):
    pass


def _json(value: Any) -> Any:
    if isinstance(value, Path):
        return str(value)
    if dataclasses.is_dataclass(value) and not isinstance(value, type):
        return _json(dataclasses.asdict(value))
    if isinstance(value, dict):
        return {str(k): _json(v) for k, v in value.items()}
    if isinstance(value, (list, tuple, set, frozenset)):
        return [_json(v) for v in value]
    return value


def call(op: str, kwargs: dict, root: Path = REPO) -> Any:
    tool = REPO / "tools/build-infra"
    request = json.dumps({"op": op, "kwargs": _json(kwargs)}, ensure_ascii=False)
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
