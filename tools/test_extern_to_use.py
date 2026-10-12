"""Pins extern_to_use and apply_agent_edits, the fastfat clash-burndown pair."""

from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import apply_agent_edits as ae
import extern_to_use as e2u

DEFINING = (
    '#[no_mangle]\npub unsafe extern "system" fn FatRaiseStatus(c: *mut u8, s: i32) -> ! {\n}\n'
)
CALLER = (
    "use crate::types::*;\n"
    'extern "system" {\n'
    "    fn FatRaiseStatus(IrpContext: *mut u8,\n        Status: u32);\n"
    "    fn Other(x: u32);\n"
    "}\n"
)


def test_own_function_redeclaration_becomes_a_use_of_the_definition():
    files = {"seh": DEFINING, "allocsup": CALLER}
    changed, report = e2u.convert(files, e2u.find_defs(files, ["FatRaiseStatus"]), e2u.MARK_OWN)
    assert set(changed) == {"allocsup"}  # the defining module is left alone
    out = changed["allocsup"]
    assert "use crate::seh::FatRaiseStatus;" in out
    assert "fn FatRaiseStatus" not in out  # the multi-line decl went, newline included
    assert "    fn Other(x: u32);\n}" in out
    assert report == ["allocsup.rs:3: fn FatRaiseStatus(IrpContext: *mut u8, Status: u32);"]


def test_a_name_defined_twice_or_nowhere_is_refused():
    with pytest.raises(SystemExit):
        e2u.find_defs({"a": DEFINING, "b": DEFINING}, ["FatRaiseStatus"])
    with pytest.raises(SystemExit):
        e2u.find_defs({"a": CALLER}, ["Nowhere"])


def test_external_symbols_go_to_the_named_module():
    changed, _ = e2u.convert(
        {"allocsup": CALLER}, {"Other": "externs"}, e2u.MARK_TO.format(to="externs")
    )
    assert "use crate::externs::Other;" in changed["allocsup"]


def test_agent_edit_applies_only_on_a_unique_match(tmp_path: Path):
    (tmp_path / "a.rs").write_text("let x = 1;\nlet y = 1;\nlet y = 1;\n")
    rows = [
        {"id": "A1", "edit": {"file": "a.rs", "old": "let x = 1;", "new": "let x = 2;"}},
        {"id": "A2", "edit": {"file": "a.rs", "old": "let y = 1;", "new": "let y = 2;"}},
    ]
    p = tmp_path / "out.jsonl"
    p.write_text("\n".join(json.dumps(r) for r in rows))
    ok, skipped = ae.apply(ae.load([p]), tmp_path)
    assert ok == 1
    assert skipped == ["SKIP A2 a.rs: old occurs 2x"]
    assert (tmp_path / "a.rs").read_text().startswith("let x = 2;")
