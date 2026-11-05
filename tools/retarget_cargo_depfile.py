#!/usr/bin/env python3
"""Python compatibility facade for native build infrastructure."""

from __future__ import annotations

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


def escaped_output(path):
    return _coerce(
        _invoke("depfile_escaped_output", locals(), None),
        typing.get_type_hints(escaped_output).get("return", typing.Any),
    )


def retarget(text, cargo_output, copied_output):
    return _coerce(
        _invoke("depfile_retarget", locals(), None),
        typing.get_type_hints(retarget).get("return", typing.Any),
    )


def main(argv=None):
    return _native_main("retarget-depfile", argv)


if __name__ == "__main__":
    sys.exit(main())
