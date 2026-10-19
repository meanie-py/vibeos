#!/usr/bin/env python3
"""Compatibility CLI and native COM parser bridge used by rustconv."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path


def parse_vtbls(src: str) -> dict[str, list[tuple[str, list[tuple[str, str]], str | None]]]:
    tool = Path(__file__).resolve().parent / "codegen"
    result = subprocess.run(
        [str(tool), "com-macros", "--bindgen", "-", "--parse-json"],
        input=src,
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode:
        raise ValueError(f"native COM parser failed: {result.stderr.strip()}")
    return {
        name: [(method, [(arg, ty) for arg, ty in args], ret) for method, args, ret in methods]
        for name, methods in json.loads(result.stdout).items()
    }


if __name__ == "__main__":
    from codegen_compat import main

    main("com-macros")
