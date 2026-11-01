#!/usr/bin/env python3
"""Legacy WoW64 generator API; parsing, classification and rendering live in Rust."""

from __future__ import annotations

import json
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

_TOOL = Path(__file__).resolve().parents[1] / "codegen"


def _rpc(op: str, **fields):
    result = subprocess.run(
        [str(_TOOL), "wow64-thunks", "--api-json"],
        input=json.dumps({"op": op, **fields}),
        capture_output=True,
        text=True,
        check=False,
        timeout=180,
    )
    if result.returncode:
        raise RuntimeError(result.stderr.strip() or "native wow64 generator failed")
    return json.loads(result.stdout)


@dataclass(frozen=True)
class ArgKind:
    dwords: int
    c_type: str
    widen: str
    manual: bool = False


ARG_KINDS = {name: ArgKind(**value) for name, value in _rpc("arg_kinds").items()}


@dataclass
class Syscall:
    name: str
    args: list[str]
    ordinal: int = -1
    arch: str | None = None
    manual_reason: str = ""
    slots: int = 0

    @property
    def needs_manual(self) -> bool:
        return bool(self.manual_reason)


def _update(sc: Syscall, row: dict) -> Syscall:
    for key in Syscall.__dataclass_fields__:
        setattr(sc, key, row[key])
    return sc


def _row(row: dict) -> Syscall:
    return _update(Syscall(row["name"], row["args"]), row)


def parse_sysfuncs(path: str) -> list[tuple[str, int]]:
    return [tuple(row) for row in _rpc("parse_sysfuncs", path=str(path))]


def load_manual_list(path: str | None) -> set[str]:
    return set(_rpc("load_manual_list", paths=path or ""))


def parse_spec(path: str, arch: str, manual: set[str] | None = None) -> list[Syscall]:
    return [
        _row(row)
        for row in _rpc("parse_spec", path=str(path), arch=arch, manual=sorted(manual or set()))
    ]


def order_by_sysfuncs(
    sysfuncs: list[tuple[str, int]], spec_syscalls: list[Syscall]
) -> list[Syscall]:
    rows = _rpc("order", sysfuncs=sysfuncs, syscalls=[asdict(sc) for sc in spec_syscalls])
    original = {sc.name: sc for sc in spec_syscalls}
    # Preserve the legacy API's object identities and in-place updates.
    return [
        _update(original[row["name"]], row) if row["name"] in original else _row(row)
        for row in rows
    ]


def _classify(sc: Syscall) -> None:
    _update(sc, _rpc("classify", syscall=asdict(sc)))


def emit_c(
    syscalls: list[Syscall], spec: str, arch: str, out: str, symbol_prefix: str = "Wow64Thunk"
) -> str:
    return _rpc(
        "emit",
        syscalls=[asdict(sc) for sc in syscalls],
        spec=spec,
        arch=arch,
        out=out,
        symbol_prefix=symbol_prefix,
    )


def main(argv: list[str]) -> int:
    return subprocess.run([str(_TOOL), "wow64-thunks", *argv], check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
