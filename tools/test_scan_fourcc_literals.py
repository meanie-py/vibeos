"""Tests for scan_fourcc_literals — a MAKEFOURCC folded to the wrong integer (evr's MFIsFormatYUV)."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from scan_fourcc_literals import scan_text


def test_the_evr_misfolds_fire_with_the_right_value():
    text = (
        "        | 808530521 // MAKEFOURCC('I','M','C','1')\n"
        "        | 842094158 // MAKEFOURCC('N','V','1','1')\n"
    )
    assert scan_text(text) == [(1, "IMC1", 808530521, 826494281), (2, "NV11", 842094158, 825316942)]


def test_correct_folds_in_any_spelling_are_silent():
    text = (
        "| 842094158 // MAKEFOURCC('N','V','1','2')\n"
        "const Y: u32 = 0x3231_564e; // MAKEFOURCC('N', 'V', '1', '2')\n"
        "x = 808530521u32 as DWORD; // mmioFOURCC('Y','2','1','0')\n"
    )
    assert scan_text(text) == []


def test_a_literal_without_the_comment_is_not_a_candidate():
    assert scan_text("| 808530521 // IMC1\n| 808530521\n") == []
