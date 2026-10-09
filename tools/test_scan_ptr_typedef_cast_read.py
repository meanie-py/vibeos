"""Unit tests for scan_ptr_typedef_cast_read: the shipped shape, its fix, and nolint."""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
_spec = importlib.util.spec_from_file_location(
    "scan_ptr_typedef_cast_read", REPO / "tools" / "scan_ptr_typedef_cast_read.py"
)
assert _spec is not None and _spec.loader is not None
spc = importlib.util.module_from_spec(_spec)
sys.modules["scan_ptr_typedef_cast_read"] = spc
_spec.loader.exec_module(spc)


def _scan(tmp_path: Path, body: str) -> list[tuple[int, str]]:
    f = tmp_path / "x.rs"
    f.write_text(body)
    return [(line, name) for _, line, name in spc.scan([f])]


def test_flags_the_double_deref(tmp_path: Path) -> None:
    body = "\nLocaleId = (*(GetListEntryData(ListEntry) as *mut PGENENTRY).read()).Id;\n"
    assert _scan(tmp_path, body) == [(2, "PGENENTRY")]


def test_quiet_on_the_fix_a_plain_slot_read_and_nolint(tmp_path: Path) -> None:
    body = """
LocaleId = (*(GetListEntryData(ListEntry) as PGENENTRY)).Id;
let e = (h as *mut PVOID).read();
let x = (*(slot as *mut PFOO).read()).Next; // nolint: ptr-typedef-cast-read (slot holds a PFOO)
"""
    assert _scan(tmp_path, body) == []
