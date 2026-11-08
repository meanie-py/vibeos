#!/usr/bin/env python3
"""Compatibility API/CLI for the native WoW64 deep-marshaller generator."""

from __future__ import annotations

import json
import subprocess
import sys
from dataclasses import asdict, dataclass, field, is_dataclass
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
CLANG = REPO / ".build-tools/llvm-mingw/bin/clang"
_TOOL = REPO / "tools/codegen"


def _encode(value):
    if is_dataclass(value):
        return asdict(value)
    if isinstance(value, Path):
        return str(value)
    if isinstance(value, set):
        return sorted(value)
    raise TypeError(f"unsupported marshal API value: {type(value).__name__}")


def _rpc(op, **values):
    result = subprocess.run(
        [str(_TOOL), "--root", str(REPO), "marshal", "--api-json"],
        input=json.dumps({"op": op, **values}, default=_encode),
        capture_output=True,
        text=True,
        check=False,
        timeout=180,
    )
    if result.returncode:
        raise RuntimeError(result.stderr.strip() or "native marshal generator failed")
    return json.loads(result.stdout)


@dataclass
class Field:
    offset: int
    depth: int
    ctype: str
    name: str
    is_bitfield: bool
    bits: str = ""


@dataclass
class Record:
    name: str
    size: int
    align: int
    fields: list[Field] = field(default_factory=list)

    @property
    def has_union(self):
        return self.name.startswith("union") or any(
            f.ctype.startswith("union") for f in self.fields
        )

    @property
    def has_bitfield(self):
        return any(f.is_bitfield for f in self.fields)

    def top_fields(self):
        return [f for f in self.fields if f.depth == 1]


@dataclass
class Param:
    ctype: str
    name: str
    sal: str


@dataclass
class Proto:
    name: str
    ret: str
    params: list[Param]


@dataclass
class StructVerdict:
    cls: str
    detail: str
    rec32: Record | None = None
    rec64: Record | None = None


def _record(row):
    return (
        None
        if row is None
        else Record(row["name"], row["size"], row["align"], [Field(**f) for f in row["fields"]])
    )


def _records(rows):
    return {name: _record(row) for name, row in rows.items()}


def parse_layout_dump(text):
    return _records(_rpc("parse_layout_dump", text=text))


def extract_flags(build_dir, tu_suffix):
    return _rpc("extract_flags", build=str(build_dir), suffix=tu_suffix)


def dump_layouts(flags, tu_source):
    return _records(_rpc("dump_layouts", flags=flags, tu=tu_source))


def strip_annotations(raw):
    return tuple(_rpc("strip_annotations", text=raw))


def strip_comments(text):
    return _rpc("strip_comments", text=text)


def extract_prototypes(text, name_re):
    return {
        name: Proto(row["name"], row["ret"], [Param(**p) for p in row["params"]])
        for name, row in _rpc("extract_prototypes", text=text, pattern=name_re).items()
    }


def _norm_type(ctype):
    return _rpc("norm_type", text=ctype)


def resolve_exact(ctype, records):
    row = _rpc("resolve_exact", ctype=ctype, db=records)
    return records.get(row["name"], _record(row)) if row is not None else None


def resolve_pointee(ctype, records):
    row = _rpc("resolve_pointee", ctype=ctype, db=records)
    return records.get(row["name"], _record(row)) if row is not None else None


def diff_struct(name, db32, db64, _seen=None):
    row = _rpc("diff_struct", name=name, db32=db32, db64=db64)
    return StructVerdict(row["cls"], row["detail"], _record(row["rec32"]), _record(row["rec64"]))


def diff_structs(names, db32, db64):
    rows = _rpc("diff_structs", names=list(names), db32=db32, db64=db64)
    return {
        name: StructVerdict(row["cls"], row["detail"], _record(row["rec32"]), _record(row["rec64"]))
        for name, row in rows.items()
    }


def classify_param(p, db32, db64):
    return tuple(_rpc("classify_param", param=p, db32=db32, db64=db64))


_metadata = _rpc("metadata")
AUDIT_TABLES = _metadata["AUDIT_TABLES"]
WIN32K_TUS = [tuple(row) for row in _metadata["WIN32K_TUS"]]
NTDLL_TU = _metadata["NTDLL_TU"]
PARAM_RANK = _metadata["PARAM_RANK"]


def compute_thunk_table(cfg, arch):
    return dict(_rpc("source_table", cfg=cfg, arch=arch))


def needs_manual_from_sources(arch, table="win32k"):
    return [
        name for name, live in _rpc("source_table", cfg=AUDIT_TABLES[table], arch=arch) if not live
    ]


def load_thunk_table(gen_c):
    return _rpc("load_thunk_table", file=str(gen_c))


def crosscheck_artifact(computed, gen_c, tname):
    _rpc("crosscheck", computed=computed, file=str(gen_c), table=tname)


def audit_verdict(cls, detail):
    return _rpc("audit_verdict", cls=cls, detail=detail)


def cmd_audit(args):
    if args.show_unknown:
        raise RuntimeError("marshal audit diagnostics moved to ./vibeos-cli check")
    findings = _rpc("audit", build64=args.build_dir, build32=args.build_dir32, table=args.table)
    for line in findings:
        print(line)
    return int(bool(findings))


def main(argv=None):
    return subprocess.run(
        [str(_TOOL), "marshal", *(sys.argv[1:] if argv is None else argv)], check=False
    ).returncode


if __name__ == "__main__":
    raise SystemExit(main())
