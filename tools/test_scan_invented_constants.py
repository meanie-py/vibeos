"""Unit tests for scan_invented_constants' C-expression oracle.

A scanner of this shape is only as good as its evaluator, and BOTH failure
directions have already happened in this file:

  * a name regex without a lookbehind found "x0501" inside `0x0501`, failed to
    resolve it, and reported EVERY hex constant in every header as unresolvable
    -- the scanner printed a confident "0 MISMATCH" over a population it could
    not read at all;
  * ignoring C's implicit enum numbering after an explicit initialiser
    (`FcbGood = 1, FcbBad,`) reported the CORRECT `FcbNeedsToBeVerified = 3` as a
    mismatch against a phantom 2.

A false negative makes the gate decorative; a false positive gets it ignored.
Both are pinned here rather than left to a whole-tree fixture, which is why this
scanner is listed UNCOVERED in test_scanner_controls.py: its finding requires the
real C module a crate translates, so a synthetic tree cannot exercise it.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parent.parent
_spec = importlib.util.spec_from_file_location(
    "scan_invented_constants", REPO / "tools" / "scan_invented_constants.py"
)
assert _spec is not None and _spec.loader is not None
sic = importlib.util.module_from_spec(_spec)
sys.modules["scan_invented_constants"] = sic
_spec.loader.exec_module(sic)


@pytest.mark.parametrize(
    ("expr", "want"),
    [
        # The regression that made the scanner blind: a cast wrapping a hex literal.
        ("((NODE_TYPE_CODE)0x0501)", 0x0501),
        ("((ULONG) 5)", 5),
        ("((LONGLONG) -1)", -1),
        ("0x00000008", 8),
        ("(0x12 | 0x34)", 0x36),
        ("1 << 4", 16),
        ("0x10UL", 0x10),
        # C multi-character literals are BIG-endian by definition. Getting this
        # backwards is its own recurring pool-tag bug.
        ("'vtaF'", 0x76746146),
        ("'cDwH'", 0x63447748),
        # CTL_CODE: (Device<<16)|(Access<<14)|(Function<<2)|Method
        ("CTL_CODE(2, 0, 0, 1)", 0x24000),
    ],
)
def test_eval_c_shapes(expr: str, want: int) -> None:
    assert sic.eval_c(expr, {}) == want


def test_eval_c_resolves_named_operands() -> None:
    table = {"BASE": "0x10", "OFFSET": "(BASE + 4)"}
    assert sic.eval_c("OFFSET", table) == 0x14


def test_eval_c_refuses_an_unresolvable_name() -> None:
    """Silence beats a guess: an unknown operand must not evaluate to anything."""
    assert sic.eval_c("SOME_UNKNOWN_THING + 1", {}) is None


def test_harvest_continues_implicit_enum_numbering(tmp_path: Path) -> None:
    """`FcbGood = 1, FcbBad,` makes FcbBad 2 -- numbering resumes FROM the
    initialiser, not from zero."""
    h = tmp_path / "fatstruc.h"
    h.write_text(
        "typedef enum _FCB_CONDITION {\n"
        "    FcbGood = 1,\n"
        "    FcbBad,\n"
        "    FcbNeedsToBeVerified\n"
        "} FCB_CONDITION;\n"
    )
    got = sic.harvest([h])
    assert {v for v, _ in got["FcbGood"]} == {1}
    assert {v for v, _ in got["FcbBad"]} == {2}
    assert {v for v, _ in got["FcbNeedsToBeVerified"]} == {3}


def test_harvest_reports_true_line_numbers(tmp_path: Path) -> None:
    """Comment stripping must preserve newlines, or every citation is wrong."""
    h = tmp_path / "x.h"
    h.write_text("/* a\n   multi-line\n   comment */\n#define WANTED 0x7\n")
    (value, site) = next(iter(sic.harvest([h])["WANTED"]))
    assert value == 7
    assert site.endswith(":4"), site


def test_same_compares_at_the_declared_width() -> None:
    """`0xC0000052_u32 as i32` and the header's `0xC0000052L` are one value."""
    assert sic.same(-0x3FFFFFAE, 0xC0000052, "i32")
    assert not sic.same(0x10, 0x20, "u32")
