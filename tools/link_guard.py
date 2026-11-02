#!/usr/bin/env python3
"""Python compatibility facade for native build infrastructure."""

from __future__ import annotations

import re
import sys
import typing
from dataclasses import dataclass
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
DEFAULT_LLD = REPO / ".build-tools/llvm-mingw/bin/ld.lld"
_error_type = NativeError
_invoke = partial(_native_invoke, error_type=_error_type)
DUP_RE = re.compile(r"warning: duplicate symbol: (.+)$")
RUST_RUNTIME_SYMBOL = re.compile(
    r"^_?(__rustc::)?(rust_begin_unwind|rust_eh_personality|rust_oom|__rust_[a-z_]+|_fltused"
    r"|vibeos_unwind_resume_fallback)$"
)
VARARGS_PULLER = re.compile(r"^_?vibeos_va_arg_(i32|u32|i64|u64|f64|ptr)$")
VARARGS_SHIM_ORIGIN = re.compile(r"rust_varargs_shim\.c")
MEM_SYMBOL = re.compile(r"^_?(memcpy|memmove|memset|memcmp|bcmp|strlen)$")
RUNTIME_ORIGIN = re.compile(
    r"compiler_builtins-|compiler-builtins|\.[/\\]lib/builtins[/\\]|libclang_rt\.builtins|libunwind\.a\("
)
IMPORT_ORIGIN = re.compile(
    r"\.delay\.o\)?$|\([^()/]+\.(dll|exe|sys|drv|ocx|cpl|acm|efi|tsp)\)$|^[^()/\s]+\.(dll|exe|sys|drv|ocx|cpl|acm|efi|tsp)$",
    re.IGNORECASE,
)


@dataclass
class Dup:
    symbol: str
    kept: str
    dropped: str
    kept_kind: str
    dropped_kind: str
    verdict: str


def origin_kind(origin: str) -> str:
    return _coerce(
        _invoke("link_origin_kind", locals(), None),
        typing.get_type_hints(origin_kind).get("return", typing.Any),
    )


def parse(stderr: str) -> list[tuple[str, str, str]]:
    return [tuple(row) for row in _invoke("link_parse", locals())]


def judge(symbol: str, kept: str, dropped: str) -> Dup:
    return Dup(**_invoke("link_judge", locals()))


def output_of(argv: list[str]) -> str:
    return _coerce(
        _invoke("link_output_of", locals(), None),
        typing.get_type_hints(output_of).get("return", typing.Any),
    )


def main(argv=None):
    return _native_main("link-guard", argv)


if __name__ == "__main__":
    sys.exit(main())
