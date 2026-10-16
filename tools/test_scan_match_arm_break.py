"""Tests for scan_match_arm_break: a C switch's `break;` translated so it leaves the loop."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from scan_match_arm_break import scan_text


def test_the_mf_session_shape_fires():
    text = (
        "fn f() {\n"
        "    while i < n {\n"
        "        match kind {\n"
        "            SOURCE => {\n"
        "                check(node);\n"
        "                break;\n"
        "            }\n"
        "            _ => {}\n"
        "        }\n"
        "        release(node);\n"
        "    }\n"
        "}\n"
    )
    assert scan_text(text) == [6]


def test_meant_breaks_are_silent():
    text = (
        "fn f() {\n"
        "    loop {\n"
        "        match x {\n"
        "            A => { a(); break; }\n"  # once-through loop standing in for a switch
        "            _ => break,\n"
        "        }\n"
        "    }\n"
        "    for c in s {\n"
        "        match c {\n"
        "            0 => break,\n"  # expression arm: written on purpose
        "            _ => { for d in t { g(d); break; } }\n"  # break of the inner loop
        "        }\n"
        "    }\n"
        '    match y { B => { h(); "break;"; } _ => {} }\n'  # no loop, and a string
        "}\n"
    )
    assert scan_text(text) == []
