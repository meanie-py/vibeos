#!/usr/bin/env python3
"""Python compatibility facade for native build infrastructure."""

from __future__ import annotations

import re
import sys
import typing
from functools import partial
from pathlib import Path

if __package__:
    from .build_infra_compat import NativeError
    from .build_infra_compat import coerce as _coerce
    from .build_infra_compat import invoke as _native_invoke
    from .build_infra_compat import main as _native_main
else:
    from build_infra_compat import NativeError
    from build_infra_compat import coerce as _coerce
    from build_infra_compat import invoke as _native_invoke
    from build_infra_compat import main as _native_main

REPO = Path(__file__).resolve().parent.parent


class ManifestError(NativeError):
    pass


_error_type = ManifestError
_invoke = partial(_native_invoke, error_type=_error_type)
_metadata_names = {"ARCHES": "tuple", "CANON": "plain", "CELL_ENV": "tuple", "STMT": "regex"}


def __getattr__(name):
    if name not in _metadata_names:
        raise AttributeError(name)
    data = _invoke("cell_constants", {}, REPO)
    for key, kind in _metadata_names.items():
        value = data[key]
        globals()[key] = (
            tuple(value) if kind == "tuple" else re.compile(value) if kind == "regex" else value
        )
    return globals()[name]


def manifest_path(arch: str, repo: Path = REPO) -> Path:
    return _coerce(
        _invoke("cell_manifest_path", locals(), repo),
        typing.get_type_hints(manifest_path).get("return", typing.Any),
    )


def _build_rel(path: str, arch: str) -> str:
    return _coerce(
        _invoke("cell__build_rel", locals(), None),
        typing.get_type_hints(_build_rel).get("return", typing.Any),
    )


def _post(cmd: list[str], out: str, arch: str) -> str:
    return _coerce(
        _invoke("cell__post", locals(), None),
        typing.get_type_hints(_post).get("return", typing.Any),
    )


def parse_cell(outs: str, command: str, arch: str) -> dict:
    return _coerce(
        _invoke("cell_parse_cell", locals(), None),
        typing.get_type_hints(parse_cell).get("return", typing.Any),
    )


def derive(arch: str, repo: Path = REPO) -> list[dict]:
    return _coerce(
        _invoke("cell_derive", locals(), repo),
        typing.get_type_hints(derive).get("return", typing.Any),
    )


def render(cells: list[dict]) -> str:
    return _coerce(
        _invoke("cell_render", locals(), None),
        typing.get_type_hints(render).get("return", typing.Any),
    )


def load(arch: str, repo: Path = REPO) -> list[dict]:
    return _coerce(
        _invoke("cell_load", locals(), repo), typing.get_type_hints(load).get("return", typing.Any)
    )


def current(arch: str, repo: Path = REPO) -> list[dict]:
    return _coerce(
        _invoke("cell_current", locals(), repo),
        typing.get_type_hints(current).get("return", typing.Any),
    )


def drift(committed: list[dict], derived: list[dict]) -> list[str]:
    return _coerce(
        _invoke("cell_drift", locals(), None),
        typing.get_type_hints(drift).get("return", typing.Any),
    )


def main(argv=None):
    return _native_main("cell-manifest", argv)


if __name__ == "__main__":
    sys.exit(main())
