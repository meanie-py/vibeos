#!/usr/bin/env python3
"""Compatibility API for native PE media stripping and checksum publication."""

from __future__ import annotations

import os
import sys
from pathlib import Path

if __package__ in {None, ""}:
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tools import build_infra_compat as bridge

MANIFEST = "strip-manifest.json"


def parse_pe(data: bytes):
    value = bridge.call("stage_parse_pe", {"data": list(data)})
    return (value[0], value[1]) if value is not None else None


def has_debug_sections(names: list[str]) -> bool:
    return bridge.call("stage_has_debug", locals())


def pe_checksum(data: bytearray, csum_off: int) -> int:
    # Preserve the caller-visible field clearing, while Rust owns the calculation.
    result = bridge.call("stage_checksum", {"data": list(data), "csum_off": csum_off})
    data[csum_off : csum_off + 4] = bytes(4)
    return result


def strip_one(src: Path, dst: Path, objcopy: str) -> None:
    bridge.call("stage_strip_one", locals())


def twin_for(src: Path, cache: Path, objcopy: str, manifest: dict) -> Path | None:
    value = bridge.call("stage_twin_for", locals())
    manifest.clear()
    manifest.update(value["manifest"])
    return Path(value["twin"]) if value["twin"] is not None else None


def main() -> int:
    os.execv(
        str(bridge.REPO / "tools/build-infra"), ["build-infra", "strip-cd-media", *sys.argv[1:]]
    )


if __name__ == "__main__":
    sys.exit(main())
