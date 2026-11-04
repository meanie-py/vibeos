#!/usr/bin/env python3
"""Compatibility facade for the native Cargo cell orchestrator."""

from __future__ import annotations

import os
import sys
from dataclasses import dataclass, field
from pathlib import Path

if __package__:
    from .build_infra_compat import invoke
    from .build_infra_compat import main as native_main
else:
    from build_infra_compat import invoke
    from build_infra_compat import main as native_main

REPO = Path(__file__).resolve().parent.parent

if __name__ == "__main__":
    tool = REPO / "tools/build-infra"
    os.execv(tool, [str(tool), "cargo-orchestrate", *sys.argv[1:]])


@dataclass(frozen=True)
class Cell:
    target: str
    crate: str
    output: str
    kind: str
    deps: tuple[str, ...] = field(default=())
    arches: tuple[str, ...] = ("i386", "amd64", "arm64")


def _cell(value):
    value = dict(value)
    value["crate"] = value.pop("crate_name")
    value["deps"] = tuple(value["deps"])
    value["arches"] = tuple(value["arches"])
    return Cell(**value)


def _arg(cell):
    return {
        "target": cell.target,
        "crate_name": cell.crate,
        "output": cell.output,
        "kind": cell.kind,
        "deps": cell.deps,
        "arches": cell.arches,
    }


CELLS = {c.target: c for c in map(_cell, invoke("cargo_orchestrate_cells", {}))}


def build_std(cell: Cell, arch: str) -> bool:
    return invoke("cargo_orchestrate_build_std", {"cell": _arg(cell), "arch": arch})


def cell_stem(cell: Cell, arch: str) -> str:
    return invoke("cargo_orchestrate_cell_stem", {"cell": _arg(cell), "arch": arch})


def target_json(root: Path, cell: Cell, arch: str) -> Path:
    return Path(invoke("cargo_orchestrate_target_json", {"cell": _arg(cell), "arch": arch}, root))


def cargo_target_dir(root: Path, cell: Cell, arch: str) -> Path:
    return Path(invoke("cargo_orchestrate_target_dir", {"cell": _arg(cell), "arch": arch}, root))


def built_exe(root: Path, cell: Cell, arch: str) -> Path:
    return Path(invoke("cargo_orchestrate_built_exe", {"cell": _arg(cell), "arch": arch}, root))


def topo(targets: list[str]) -> list[Cell]:
    return list(map(_cell, invoke("cargo_orchestrate_topo", {"targets": targets})))


def main(argv: list[str] | None = None) -> int:
    return native_main("cargo-orchestrate", argv)
