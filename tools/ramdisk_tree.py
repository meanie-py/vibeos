#!/usr/bin/env python3
"""Compatibility API for the native ramdisk graft-list staging mechanism."""

from __future__ import annotations

import os
import sys
from pathlib import Path

if __package__ in {None, ""}:
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tools import build_infra_compat as bridge


def parse_lst(path: Path, canon: tuple[str, str] | None) -> list[tuple[str, str]]:
    return [tuple(row) for row in bridge.call("stage_parse_lst", locals())]


def build(lists: list[Path], out: Path, canon: tuple[str, str] | None) -> int:
    try:
        return bridge.call("stage_ramdisk_tree", locals())
    except bridge.NativeError as error:
        print(f"ramdisk_tree: {error}; refusing to build a partial tree", file=sys.stderr)
        return -1


def main() -> int:
    os.execv(str(bridge.REPO / "tools/build-infra"), ["build-infra", "ramdisk-tree", *sys.argv[1:]])


if __name__ == "__main__":
    sys.exit(main())
