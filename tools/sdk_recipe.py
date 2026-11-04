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


class RecipeError(NativeError):
    pass


_error_type = RecipeError
_invoke = partial(_native_invoke, error_type=_error_type)
_metadata_names = {
    "ARCHES": "tuple",
    "CANON": "plain",
    "GENERATORS": "tuple",
    "HOST_TOOLS": "tuple",
    "MEDIA_TOOLS": "tuple",
}


def __getattr__(name):
    if name not in _metadata_names:
        raise AttributeError(name)
    data = _invoke("sdk_constants", {}, REPO)
    for key, kind in _metadata_names.items():
        value = data[key]
        globals()[key] = (
            tuple(value) if kind == "tuple" else re.compile(value) if kind == "regex" else value
        )
    return globals()[name]


CELL_EDGE = re.compile(
    r"\nbuild ([^\n]*?): CUSTOM_COMMAND([^\n]*)\n  COMMAND = [^\n]*\n  DESC = Cargo-linking"
)
HOST_JOBS = re.compile(r'"?\S*/\.cache/host-jobs/host-jobs"? --kind \w+ -- ')


def recipe_path(arch: str, host: bool = False, repo: Path = REPO) -> Path:
    return _coerce(
        _invoke("sdk_recipe_path", locals(), repo),
        typing.get_type_hints(recipe_path).get("return", typing.Any),
    )


def _ninja_commands(build: Path, targets: list[str]) -> list[str]:
    return _coerce(
        _invoke("sdk__ninja_commands", locals(), None),
        typing.get_type_hints(_ninja_commands).get("return", typing.Any),
    )


def _tool(line: str) -> str:
    return _coerce(
        _invoke("sdk__tool", locals(), None), typing.get_type_hints(_tool).get("return", typing.Any)
    )


def _keep(line: str) -> bool:
    return _coerce(
        _invoke("sdk__keep", locals(), None), typing.get_type_hints(_keep).get("return", typing.Any)
    )


def _unwrap(line: str) -> str:
    return _coerce(
        _invoke("sdk__unwrap", locals(), None),
        typing.get_type_hints(_unwrap).get("return", typing.Any),
    )


def _cell_deps(ninja: str) -> list[str]:
    return _coerce(
        _invoke("sdk__cell_deps", locals(), None),
        typing.get_type_hints(_cell_deps).get("return", typing.Any),
    )


def _encode(line: str, arch: str) -> str:
    return _coerce(
        _invoke("sdk__encode", locals(), None),
        typing.get_type_hints(_encode).get("return", typing.Any),
    )


def _factor(lines: list[str]) -> list[str]:
    return _coerce(
        _invoke("sdk__factor", locals(), None),
        typing.get_type_hints(_factor).get("return", typing.Any),
    )


def derive(arch: str, repo: Path = REPO) -> tuple[list[str], list[str]]:
    sdk, host = _invoke("sdk_derive", locals(), repo)
    return sdk, host


def run(arch: str, gen: Path, repo: Path = REPO) -> None:
    return _coerce(
        _invoke("sdk_run", locals(), repo), typing.get_type_hints(run).get("return", typing.Any)
    )


def main(argv=None):
    return _native_main("sdk-recipe", argv)


if __name__ == "__main__":
    sys.exit(main())
