"""Compatibility data classes for the native debugcon decoder."""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from pathlib import Path

from tools import qemu_infra_compat as native

CATALOG_DIR = Path(__file__).resolve().parents[2] / "tools/rustconv/data"


@dataclass
class TracePoint:
    char: str
    function: str
    file: str
    line: int
    description: str


@dataclass
class DebugconResult:
    raw_bytes: bytes
    milestones_reached: list[TracePoint] = field(default_factory=list)
    last_milestone: TracePoint | None = None
    next_expected: TracePoint | None = None
    hex_dumps: dict[str, str] = field(default_factory=dict)


def load_catalog(arch: str) -> dict | None:
    return native.call("verdict_debugcon_load_catalog", {"arch": arch}, root=CATALOG_DIR.parents[2])


def decode_debugcon(raw: bytes, catalog: dict) -> DebugconResult:
    dto = native.call("verdict_debugcon_decode", {"raw": list(raw), "catalog": catalog})
    dto["raw_bytes"] = bytes(dto["raw_bytes"])
    dto["milestones_reached"] = [TracePoint(**p) for p in dto["milestones_reached"]]
    for key in ("last_milestone", "next_expected"):
        if dto[key] is not None:
            dto[key] = TracePoint(**dto[key])
    order = dto.pop("hex_dump_order", list(dto["hex_dumps"]))
    dto["hex_dumps"] = {name: dto["hex_dumps"][name] for name in order}
    return DebugconResult(**dto)


def format_debugcon_report(result: DebugconResult) -> str:
    dto = asdict(result)
    dto["raw_bytes"] = list(result.raw_bytes)
    dto["hex_dump_order"] = list(result.hex_dumps)
    return native.call("verdict_debugcon_format", {"result": dto})


def debugcon_report(raw: bytes, arch: str) -> str | None:
    return native.call(
        "verdict_debugcon_report", {"raw": list(raw), "arch": arch}, root=CATALOG_DIR.parents[2]
    )
