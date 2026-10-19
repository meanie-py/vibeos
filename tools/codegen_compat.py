"""Legacy Python entry points delegate generation to the native host tool."""

from __future__ import annotations

import os
import sys
from pathlib import Path
from typing import NoReturn


def main(generator: str) -> NoReturn:
    args = sys.argv[1:]
    if generator.endswith("-res"):
        expanded = []
        for arg in args:
            key, separator, value = arg.partition("=")
            expanded.extend([key, value] if separator and key in {"--arch", "--module"} else [arg])
        args = expanded
        if generator == "committed-res":
            # The original replay helper read compile_commands relative to cwd.
            args = ["--root", str(Path.cwd()), *args]
        if "--arch" not in args and "--jobs-json" not in args:
            default = "i386,amd64,arm64" if generator in {"committed-res", "ntdll-res"} else "i386"
            args.extend(["--arch", default])
    if generator in {"error-tables", "winerror-consts"}:
        minimum = 3 if generator == "winerror-consts" else 2
        if len(args) < minimum or any(arg.startswith("-") for arg in args):
            raise SystemExit(
                f"usage: {sys.argv[0]} INPUT OUTPUT"
                + (" HEADER [HEADER ...]" if minimum == 3 else "")
            )
        source, output, *headers = args
        args = ["--input", source, "--output", output]
        # The checkout's RTL recipe preserves intentional Rust divergences. Do not
        # let its old positional invocation erase them by selecting literal mode.
        repo = Path(__file__).resolve().parent.parent
        if generator == "error-tables" and Path(source).resolve() == repo / "sdk/lib/rtl/error.c":
            args = ["--output", output]
        for header in headers:
            args.extend(["--header", header])
    tool = Path(__file__).resolve().parent / "codegen"
    try:
        os.execv(str(tool), [str(tool), generator, *args])
    except OSError as error:
        raise SystemExit(f"cannot launch native code generator {tool}: {error}") from error
