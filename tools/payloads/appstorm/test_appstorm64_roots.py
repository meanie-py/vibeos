"""Every program appstorm launches must be staged as an amd64 guest by appstorm64.

appstorm64 runs appstorm's whole app table as x86-64 guests, but its guest set is a
separate hand-kept list (tools/payloads/appstorm64/payload.yaml `roots`). An app added to
the table and not to the list is not missing: the launch resolves to the native arm64
copy, appstorm sees the machine differ from its own and scores it FAIL. certmgr, mmc_new
and drwtsn32 sat that way from aa6a2359ed3 until 2026-10-15, reading as three app failures.
"""
from __future__ import annotations

import re
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[3]
APPSTORM_C = ROOT / "tools/payloads/appstorm/appstorm.c"
APPSTORM64 = ROOT / "tools/payloads/appstorm64/payload.yaml"
# `{ "name", "program.exe", ...`: the second string is what CreateProcess runs.
_ENTRY = re.compile(r'\{\s*"[^"]+"\s*,\s*"([^"]+\.exe)"')


def _launched() -> set[str]:
    text = APPSTORM_C.read_text()
    table = text[text.index("static const app_t apps[]") :]
    table = table[: table.index("};")]
    return {m.group(1).lower() for m in _ENTRY.finditer(table)}


def _staged() -> set[str]:
    doc = yaml.safe_load(APPSTORM64.read_text())
    return {
        Path(root).name.lower()
        for guest_set in doc["guest_sets"]
        for root in guest_set["roots"]
    }


def test_table_parses() -> None:
    # A regex that silently matched nothing would make the real test vacuous.
    assert {"notepad.exe", "control.exe", "rundll32.exe"} <= _launched()


def test_every_launched_program_is_staged() -> None:
    missing = sorted(_launched() - _staged())
    assert not missing, (
        f"appstorm launches {missing} but appstorm64 does not stage them as amd64 guests; "
        "add their build paths to guest_sets roots in tools/payloads/appstorm64/payload.yaml"
    )
