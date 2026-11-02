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


class MediaError(NativeError):
    pass


_error_type = MediaError
_invoke = partial(_native_invoke, error_type=_error_type)
_metadata_names = {
    "ARCHES": "tuple",
    "CDS": "tuple",
    "CANON": "plain",
    "CONFIG": "plain",
    "CAB": "plain",
    "CAB_INF": "plain",
    "PACKAGES": "plain",
    "CELLS_STAMP": "plain",
    "COMMITTED_ROSTESTS": "tuple",
    "_ROSTEST_SRC": "regex",
}


def __getattr__(name):
    if name not in _metadata_names:
        raise AttributeError(name)
    data = _invoke("media_constants", {}, REPO)
    for key, kind in _metadata_names.items():
        value = data[key]
        globals()[key] = (
            tuple(value) if kind == "tuple" else re.compile(value) if kind == "regex" else value
        )
    return globals()[name]


def manifest_path(arch: str, repo: Path = REPO) -> Path:
    return _coerce(
        _invoke("media_manifest_path", locals(), repo),
        typing.get_type_hints(manifest_path).get("return", typing.Any),
    )


def _enc(path: str, arch: str, cells: set[str]) -> str:
    return _coerce(
        _invoke("media__enc", locals(), None), typing.get_type_hints(_enc).get("return", typing.Any)
    )


def _enc_cmd(cmd: str, arch: str) -> str:
    return _coerce(
        _invoke("media__enc_cmd", locals(), None),
        typing.get_type_hints(_enc_cmd).get("return", typing.Any),
    )


def _command(ninja: str, cd: str) -> str:
    return _coerce(
        _invoke("media__command", locals(), None),
        typing.get_type_hints(_command).get("return", typing.Any),
    )


def _output_command(ninja: str, out: str) -> str:
    return _coerce(
        _invoke("media__output_command", locals(), None),
        typing.get_type_hints(_output_command).get("return", typing.Any),
    )


def derive_cab(arch: str, build: Path, ninja: str, cells: set[str]) -> dict:
    return _coerce(
        _invoke("media_derive_cab", locals(), None),
        typing.get_type_hints(derive_cab).get("return", typing.Any),
    )


def derive(arch: str, repo: Path = REPO) -> dict:
    return _coerce(
        _invoke("media_derive", locals(), repo),
        typing.get_type_hints(derive).get("return", typing.Any),
    )


def committed(m: dict, arch: str = "") -> dict:
    _invoke("media_committed", locals())
    return m


def render(m: dict) -> str:
    return _coerce(
        _invoke("media_render", locals(), None),
        typing.get_type_hints(render).get("return", typing.Any),
    )


def load(arch: str, repo: Path = REPO) -> dict:
    return _coerce(
        _invoke("media_load", locals(), repo), typing.get_type_hints(load).get("return", typing.Any)
    )


def current(arch: str, repo: Path = REPO) -> dict:
    return _coerce(
        _invoke("media_current", locals(), repo),
        typing.get_type_hints(current).get("return", typing.Any),
    )


def resolve(src: str, build: Path, repo: Path, cells: Path | None) -> Path:
    return _coerce(
        _invoke("media_resolve", locals(), repo),
        typing.get_type_hints(resolve).get("return", typing.Any),
    )


def write_list(entries: list, where: dict[str, Path]) -> str:
    return _coerce(
        _invoke("media_write_list", locals(), None),
        typing.get_type_hints(write_list).get("return", typing.Any),
    )


def source_hash(repo: Path = REPO) -> str:
    return _coerce(
        _invoke("media_source_hash", locals(), repo),
        typing.get_type_hints(source_hash).get("return", typing.Any),
    )


def cells_source(cells: Path, arch: str) -> str:
    return _coerce(
        _invoke("media_cells_source", locals(), None),
        typing.get_type_hints(cells_source).get("return", typing.Any),
    )


def _check_tools(cmd: str) -> None:
    return _coerce(
        _invoke("media__check_tools", locals(), None),
        typing.get_type_hints(_check_tools).get("return", typing.Any),
    )


def _run(cmd: str) -> None:
    return _coerce(
        _invoke("media__run", locals(), None), typing.get_type_hints(_run).get("return", typing.Any)
    )


def recab(m: dict, out: Path, build: Path, repo: Path, cells: Path) -> dict[str, Path]:
    return {k: Path(v) for k, v in _invoke("media_recab", locals(), repo).items()}


def assemble(arch: str, cd: str, out: Path, cells: Path | None, repo: Path = REPO) -> Path:
    return _coerce(
        _invoke("media_assemble", locals(), repo),
        typing.get_type_hints(assemble).get("return", typing.Any),
    )


def main(argv=None):
    return _native_main("media-manifest", argv)


if __name__ == "__main__":
    sys.exit(main())
