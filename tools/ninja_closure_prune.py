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
_error_type = NativeError
_invoke = partial(_native_invoke, error_type=_error_type)
_ESC = re.compile(r"\$(.)")


def _unescape(tok: str) -> str:
    return _coerce(
        _invoke("ninja__unescape", locals(), None),
        typing.get_type_hints(_unescape).get("return", typing.Any),
    )


def _split(s: str) -> list[str]:
    return _coerce(
        _invoke("ninja__split", locals(), None),
        typing.get_type_hints(_split).get("return", typing.Any),
    )


def _logical_lines(text: str):
    return iter(_invoke("ninja__logical_lines", locals()))


def _colon(s: str) -> int:
    return _coerce(
        _invoke("ninja__colon", locals(), None),
        typing.get_type_hints(_colon).get("return", typing.Any),
    )


def parse_edges(manifest: Path) -> list[tuple[list[str], list[str]]]:
    return [
        (list(outputs), list(inputs)) for outputs, inputs in _invoke("ninja_parse_edges", locals())
    ]


def closure_outputs(edges, targets: list[str], norm) -> tuple[set[str], list[str]]:
    # Evaluate the supplied normalizer at the boundary; native traversal consumes
    # only this deterministic mapping and does not call Python.
    names = dict.fromkeys([*targets, *(x for edge in edges for group in edge for x in group)])
    mapping = {name: norm(name) for name in names}
    keep, unknown = _invoke(
        "ninja_closure_outputs", {"edges": edges, "targets": targets, "norm": mapping}
    )
    return set(keep), unknown


def logged_outputs(build: Path) -> list[str]:
    return _coerce(
        _invoke("ninja_logged_outputs", locals(), None),
        typing.get_type_hints(logged_outputs).get("return", typing.Any),
    )


def plan(build: Path, targets: list[str], canon_root: str | None = None):
    victims, unknown = _invoke("ninja_plan", locals())
    return victims, unknown


def main(argv=None):
    return _native_main("ninja-prune", argv)


if __name__ == "__main__":
    sys.exit(main())
