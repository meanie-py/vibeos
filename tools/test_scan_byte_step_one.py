"""Unit tests for scan_byte_step_one.

Pins the shipped shapes (a header pointer, and a `(*x).field` deref group) and the two
look-alikes that are NOT the cast: a call's own paren around a cast argument, and a
call nested inside the cast group.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
_spec = importlib.util.spec_from_file_location(
    "scan_byte_step_one", REPO / "tools" / "scan_byte_step_one.py"
)
assert _spec is not None and _spec.loader is not None
sbs = importlib.util.module_from_spec(_spec)
sys.modules["scan_byte_step_one"] = sbs
_spec.loader.exec_module(sbs)


def _scan(tmp_path: Path, body: str) -> list[tuple[int, str]]:
    f = tmp_path / "x.rs"
    f.write_text(body)
    return [(line, expr) for _, line, expr in sbs.scan([f])]


def test_flags_header_steps(tmp_path: Path) -> None:
    body = """
Buffer = ((*CabinetContext).PCABHeader as *mut u8).add(1);
let p = (pHeader as *mut u8).add(1 as usize) as *mut DELETED_FILE_RECORD;
"""
    assert _scan(tmp_path, body) == [(2, "(*CabinetContext).PCABHeader"), (3, "pHeader")]


def test_quiet_on_calls_and_the_fix(tmp_path: Path) -> None:
    body = """
let n = strlen(p as *const i8).wrapping_add(1);
let t = strstr(buf.as_ptr(), b".\\0".as_ptr() as *const i8).offset(1);
Buffer = (*CabinetContext).PCABHeader.add(1) as *mut u8;
// Buffer = ((*CabinetContext).PCABHeader as *mut u8).add(1);
"""
    assert _scan(tmp_path, body) == []
