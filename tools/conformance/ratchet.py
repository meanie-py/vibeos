"""Compatibility facade for the native assertion-identity ratchet."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from tools import qemu_infra_compat as native

BAD = {"CRASH", "TIMEOUT", "FAILSTART", "NO-END", "INCOMPLETE", "ENUM-TIMEOUT"}


def _json(value):
    if isinstance(value, dict):
        return {k: _json(v) for k, v in value.items()}
    if isinstance(value, (list, tuple, set, frozenset)):
        return [_json(v) for v in value]
    return value


def _call(op, **kwargs):
    return native.call("conformance_parse_" + op, _json(kwargs))


def _rows(rows):
    for row in rows:
        row["asserts"] = set(row.get("asserts") or ())
    return rows


def fmt(r, flaky=()):
    return _call("format", result=r, flaky=flaky)


def load_baseline(path):
    base = dict(_call("baseline_entries", path=str(path)))
    for row in base.values():
        for key in ("asserts", "flaky"):
            if key in row:
                row[key] = set(row[key])
    return base


def truncated_identity(ident, known):
    return _call("truncated_identity", ident=ident, known=known)


def compare(results, base):
    return _call("compare", results=results, base=base, base_order=list(base))
